//! Unified Theory equation kit: rules, every pair and signature, and rank fixtures.
use crate::unified_core::*;
use crate::unified_eq_data::*;

/// A tiny deterministic arena: scripted movement and basic attacks; Unified cores drive their owners.
#[derive(Clone)]
struct Unit {
    f: Fighter,
    speed: i64,
    range: i64,
    attack: i64,
    interval: usize,
    next_attack: usize,
    shield: i64,
    rooted_until: usize,
    speed_pct: i64,
    speed_until: usize,
    /// Strafes sideways (a moving target) instead of walking in.
    strafe: bool,
}
impl Unit {
    fn new(id: usize, team: usize, x: i64, y: i64, hp: i64) -> Unit {
        Unit {
            f: Fighter { id, team, x, y, hp, max_hp: hp, ad: 70, ap: 120, champion: true, visible: true, targetable: true, cc_immune: false },
            speed: 700,
            range: 22000,
            attack: 28,
            interval: 70,
            next_attack: 40,
            shield: 0,
            rooted_until: 0,
            speed_pct: 0,
            speed_until: 0,
            strafe: false,
        }
    }
}
struct Arena {
    tick: usize,
    units: Vec<Unit>,
    cores: Vec<(usize, Core)>,
    ult: bool,
    dealt: Vec<i64>,
}
impl Arena {
    fn new(units: Vec<Unit>) -> Arena {
        let n = units.len();
        Arena { tick: 0, units, cores: vec![], ult: true, dealt: vec![0; n + 2] }
    }
    fn with_core(mut self, owner: usize, seed: u64, rank: usize) -> Arena {
        self.cores.push((owner, Core::new(seed, owner, rank)));
        self
    }
    fn fighters(&self) -> Vec<Fighter> {
        self.units.iter().filter(|u| u.f.hp > 0).map(|u| u.f.clone()).collect()
    }
    fn unit(&mut self, id: usize) -> Option<&mut Unit> {
        self.units.iter_mut().find(|u| u.f.id == id)
    }
    fn apply(&mut self, owner: usize, cmds: Vec<Cmd>) {
        let team = self.units.iter().find(|u| u.f.id == owner).map_or(0, |u| u.f.team);
        for c in cmds {
            match c {
                Cmd::Damage { target, amount, .. } => {
                    if let Some(u) = self.unit(target) {
                        let a = amount - u.shield.min(amount);
                        u.shield -= amount - a;
                        u.f.hp = (u.f.hp - a).max(0);
                        if team < 2 {
                            self.dealt[team] += amount;
                        }
                    }
                }
                Cmd::Heal { target, amount } => {
                    if let Some(u) = self.unit(target) {
                        u.f.hp = (u.f.hp + amount).min(u.f.max_hp);
                    }
                }
                Cmd::Shield { target, amount, .. } => {
                    if let Some(u) = self.unit(target) {
                        u.shield += amount;
                    }
                }
                Cmd::Root { target, ticks } => {
                    let t = self.tick;
                    if let Some(u) = self.unit(target) {
                        u.rooted_until = u.rooted_until.max(t + ticks);
                    }
                }
                Cmd::Speed { target, pct, ticks } => {
                    let t = self.tick;
                    if let Some(u) = self.unit(target) {
                        u.speed_pct = pct;
                        u.speed_until = t + ticks;
                    }
                }
                Cmd::SetPos { target, x, y } => {
                    if let Some(u) = self.unit(target) {
                        u.f.x = x;
                        u.f.y = y;
                    }
                }
                Cmd::Fx { .. } | Cmd::Log(_) => {}
            }
        }
    }
    fn step_with(&mut self, action: Option<(usize, Action)>, automatic: bool) {
        let tick = self.tick;
        let fighters = self.fighters();
        let wall = |_: i64, _: i64| false;
        for i in 0..self.cores.len() {
            let owner = self.cores[i].0;
            if !fighters.iter().any(|f| f.id == owner) {
                continue;
            }
            let inp = Input {
                tick,
                me: owner,
                fighters: &fighters,
                automatic,
                action: action.filter(|(o, _)| *o == owner).map(|(_, a)| a),
                ult_unlocked: self.ult,
                stunned: false,
                wall: &wall,
            };
            let cmds = self.cores[i].1.step(&inp);
            self.apply(owner, cmds);
        }
        // Scripted bodies: walk toward the nearest enemy (or strafe), attack in range.
        let snapshot = self.fighters();
        for u in self.units.iter_mut().filter(|u| u.f.hp > 0) {
            let Some(t) = snapshot.iter().filter(|e| e.team != u.f.team).min_by_key(|e| ((e.x - u.f.x).abs() + (e.y - u.f.y).abs(), e.id)) else { continue };
            let d = (((t.x - u.f.x) as f64).hypot((t.y - u.f.y) as f64)) as i64;
            let pct = if tick < u.speed_until { u.speed_pct } else { 0 };
            let v = u.speed * (100 + pct).max(10) / 100;
            if tick >= u.rooted_until {
                if u.strafe {
                    let phase = (tick / 90) % 2;
                    u.f.y += if phase == 0 { v } else { -v };
                } else if d > u.range * 8 / 10 {
                    u.f.x += (t.x - u.f.x) * v / d.max(1);
                    u.f.y += (t.y - u.f.y) * v / d.max(1);
                } else if u.f.team == 1 {
                    // In range, enemies keep dancing sideways (champions rarely stand still).
                    let side = if (tick / 50 + u.f.id) % 2 == 0 { 1 } else { -1 };
                    u.f.x += -(t.y - u.f.y) * v / d.max(1) * side;
                    u.f.y += (t.x - u.f.x) * v / d.max(1) * side;
                }
            }
            if d <= u.range && tick >= u.next_attack {
                u.next_attack = tick + u.interval;
                let (id, a) = (t.id, u.attack);
                let _ = (id, a);
            }
        }
        // Basic attacks resolved after movement.
        let attacks: Vec<(usize, usize, i64)> = self
            .units
            .iter()
            .filter(|u| u.f.hp > 0 && u.next_attack == tick + u.interval)
            .filter_map(|u| snapshot.iter().filter(|e| e.team != u.f.team).min_by_key(|e| ((e.x - u.f.x).abs() + (e.y - u.f.y).abs(), e.id)).map(|t| (u.f.team, t.id, u.attack)))
            .collect();
        for (team, id, a) in attacks {
            if let Some(t) = self.unit(id) {
                let n = a - t.shield.min(a);
                t.shield -= a - n;
                t.f.hp = (t.f.hp - n).max(0);
                if team < 2 {
                    self.dealt[team] += a;
                }
            }
        }
        self.tick += 1;
    }
    fn run(&mut self, ticks: usize) {
        for _ in 0..ticks {
            self.step_with(None, true);
        }
    }
    fn core(&self, owner: usize) -> &Core {
        &self.cores.iter().find(|c| c.0 == owner).unwrap().1
    }
    fn alive(&self, team: usize) -> usize {
        self.units.iter().filter(|u| u.f.team == team && u.f.hp > 0).count()
    }
}

/// Unified (id 1) at the origin and a tight group of dummies 40000 east.
fn range_arena() -> Arena {
    let mut units = vec![Unit::new(1, 0, 400000, 400000, 3000)];
    for (k, (dx, dy)) in [(40000, 0), (46000, 4000), (46000, -4000), (52000, 0)].into_iter().enumerate() {
        let mut d = Unit::new(10 + k, 1, 400000 + dx, 400000 + dy, 5000);
        d.speed = 0;
        d.attack = 0;
        units.push(d);
    }
    let mut a = Arena::new(units).with_core(1, 7, 7);
    a.units[0].speed = 0;
    a.units[0].attack = 0;
    a
}
fn manual(a: &mut Arena, action: Action) {
    a.step_with(Some((1, action)), false);
}
fn idle(a: &mut Arena, ticks: usize) {
    for _ in 0..ticks {
        a.step_with(None, false);
    }
}
fn enemy_hp(a: &Arena) -> i64 {
    a.units.iter().filter(|u| u.f.team == 1).map(|u| u.f.hp).sum()
}

#[test]
fn catalogue_is_complete() {
    assert_eq!(EQS.len(), 18);
    assert_eq!((0..3).map(|s| EQS.iter().filter(|e| e.science == s).count()).collect::<Vec<_>>(), vec![6, 6, 6]);
    assert_eq!(SIGNATURES.len(), 36);
    for s in &SIGNATURES {
        assert!((3..=5).contains(&s.steps.len()), "{}", s.name);
    }
}

#[test]
fn five_charges_recharge_independently() {
    let mut a = range_arena();
    for k in 0..5 {
        manual(&mut a, Action::Cast([APPLE, PRISM, ALPHA, BETA, GAMMA][k], Aim::Unit(10)));
        idle(&mut a, 10);
    }
    let tick = a.tick;
    let c = a.core(1);
    assert_eq!(c.ready_charges(tick), 0, "all five spent");
    // The first one spent comes back first, on its own timer.
    let mut back: Vec<usize> = c.charges.to_vec();
    back.sort();
    assert!(back.windows(2).all(|w| w[1] - w[0] == 11), "{back:?}");
    // Casts at ticks 0, 11, .. 44; the first returns at 360, the second at 371.
    let wait = 360 - a.tick + 3;
    idle(&mut a, wait);
    assert_eq!(a.core(1).ready_charges(a.tick), 1);
    manual(&mut a, Action::Cast(CHAIN, Aim::Unit(10)));
    assert_eq!(a.core(1).ready_charges(a.tick), 0);
}

#[test]
fn reserve_then_cast_carries_the_traits_in_order() {
    let r = |s: usize| Step { skill: s, at: 0, insight: false };
    let m = Core::new(1, 1, 7).compose(&[r(PRISM)], WELL as u8, false);
    assert_eq!(m.has(PRISM), 1);
    assert_eq!(m.mult, 100 + LINK_PCT);
    let doubled = Core::new(1, 1, 7).compose(&[r(PRISM), r(PRISM), r(PRISM)], ALPHA as u8, false);
    assert_eq!(doubled.has(PRISM), 2, "the same trait at most doubles");
    // Prism ▸ Well and Well ▸ Prism are different casts.
    let a = Core::new(1, 1, 7).compose(&[r(PRISM)], WELL as u8, false);
    let b = Core::new(1, 1, 7).compose(&[r(WELL)], PRISM as u8, false);
    assert_ne!(a.traits, b.traits);
    // A known signature is detected only for its exact order.
    let sig = Core::signature(&[RADIUM, CHAIN], GAMMA as u8);
    assert_eq!(sig.map(|s| SIGNATURES[s].name), Some("Criticality"));
    assert_eq!(Core::signature(&[CHAIN, RADIUM], GAMMA as u8), None);
}

#[test]
fn cooldowns_start_when_the_combo_resolves() {
    let mut a = range_arena();
    manual(&mut a, Action::Reserve(PRISM, Aim::Unit(10)));
    idle(&mut a, 5);
    assert!(a.core(1).usable(PRISM, a.tick), "reserving does not start the cooldown");
    manual(&mut a, Action::Reserve(PRISM, Aim::Unit(10)));
    assert_eq!(a.core(1).reserve.len(), 2, "the same skill can be reserved twice in one combo");
    manual(&mut a, Action::Cast(ALPHA, Aim::Unit(10)));
    let t = a.tick;
    assert!(!a.core(1).usable(PRISM, t) && !a.core(1).usable(ALPHA, t));
    assert!(a.core(1).reserve.is_empty());
    manual(&mut a, Action::Cast(PRISM, Aim::Unit(10)));
    assert_eq!(a.core(1).stats.casts, 1, "a skill on cooldown is refused");
}

#[test]
fn a_reserve_left_too_long_fizzles() {
    let mut a = range_arena();
    manual(&mut a, Action::Reserve(RADIUM, Aim::Unit(10)));
    idle(&mut a, RESERVE_TICKS + 5);
    let c = a.core(1);
    assert!(c.reserve.is_empty());
    assert_eq!(c.stats.fizzles, 1);
    assert_eq!(c.ready_charges(a.tick), 4, "the charge stays spent");
}

#[test]
fn every_skill_does_something_on_its_own() {
    for s in 0..18 {
        let mut a = range_arena();
        // Twin Paradox needs history and a loss to restore.
        if s == THIRD_LAW {
            a.units[1].f.x = 410000;
        }
        if s == TWIN {
            idle(&mut a, 60);
            a.units[0].f.hp -= 800;
            idle(&mut a, 12);
        }
        let before = (enemy_hp(&a), a.units[0].f.hp, a.units.iter().map(|u| (u.f.x, u.f.y)).collect::<Vec<_>>());
        let aim = match EQS[s].shape {
            Shape::Zone | Shape::Lob | Shape::Planet => Aim::Point(440000, 400000),
            Shape::Ally => Aim::Unit(1),
            _ => Aim::Unit(10),
        };
        manual(&mut a, Action::Cast(s, aim));
        // Third Law reacts to damage taken.
        if s == THIRD_LAW {
            a.units[0].f.hp -= 300;
        }
        idle(&mut a, 200);
        let after = (enemy_hp(&a), a.units[0].f.hp, a.units.iter().map(|u| (u.f.x, u.f.y)).collect::<Vec<_>>());
        let moved = before.2 != after.2;
        let hurt = after.0 < before.0;
        let healed = after.1 > before.1 - if s == THIRD_LAW { 300 } else { 0 };
        assert!(hurt || moved || healed || s == TIME_FIELD, "{} did nothing", EQS[s].name);
        if s == TIME_FIELD {
            assert!(a.core(1).marks.values().any(|m| m.until[1] > 0), "Time Field slows");
        }
        assert_eq!(a.core(1).stats.casts, 1, "{}", EQS[s].name);
    }
}

#[test]
fn every_two_step_combo_resolves() {
    for t in 0..18 {
        for s in 0..18 {
            let mut a = range_arena();
            idle(&mut a, 30);
            let aim = |k: usize| match EQS[k].shape {
                Shape::Zone | Shape::Lob | Shape::Planet => Aim::Point(440000, 400000),
                Shape::Ally => Aim::Unit(1),
                _ => Aim::Unit(10),
            };
            manual(&mut a, Action::Reserve(t, aim(t)));
            manual(&mut a, Action::Cast(s, aim(s)));
            let c = a.core(1);
            assert_eq!(c.stats.casts, 1, "{} ▸ {}", EQS[t].name, EQS[s].name);
            assert_eq!(c.stats.longest, 2);
            idle(&mut a, 240);
            assert!(a.core(1).live.len() <= 96);
        }
    }
}

#[test]
fn every_signature_fires_by_name() {
    for (g, sig) in SIGNATURES.iter().enumerate() {
        let mut a = range_arena();
        idle(&mut a, 30);
        let steps: Vec<usize> = sig.steps.iter().map(|k| usize::from(*k)).collect();
        let aim = |k: usize| match EQS[k.min(17)].shape {
            Shape::Zone | Shape::Lob | Shape::Planet => Aim::Point(440000, 400000),
            Shape::Ally => Aim::Unit(1),
            _ => Aim::Unit(10),
        };
        for (i, k) in steps.iter().enumerate() {
            let last = i + 1 == steps.len();
            if *k == usize::from(ULT_STEP) {
                // The ultimate replays the previous combo; give it one.
                let tick = a.tick;
                let _ = tick;
                manual(&mut a, Action::Ult(Aim::Unit(10)));
                idle(&mut a, ULT_WARN_TICKS + 2);
            } else if last {
                manual(&mut a, Action::Cast(*k, aim(*k)));
            } else {
                manual(&mut a, Action::Reserve(*k, aim(*k)));
            }
        }
        let c = a.core(1);
        assert_eq!(c.stats.signatures, 1, "signature {g} {} did not fire", sig.name);
        assert_eq!(c.stats.sig_uses, vec![g]);
        idle(&mut a, 300);
    }
}

#[test]
fn photoelectric_needs_two_marks() {
    let mut weak = range_arena();
    let hp = enemy_hp(&weak);
    manual(&mut weak, Action::Cast(PHOTO, Aim::Unit(10)));
    idle(&mut weak, 60);
    let below = hp - enemy_hp(&weak);
    let mut strong = range_arena();
    // Root (Apple) and slow (Time Field) on the target first.
    manual(&mut strong, Action::Cast(TIME_FIELD, Aim::Point(440000, 400000)));
    manual(&mut strong, Action::Cast(APPLE, Aim::Point(440000, 400000)));
    idle(&mut strong, 60);
    assert!(strong.core(1).marks_on(10, strong.tick) >= 2);
    let hp = enemy_hp(&strong);
    manual(&mut strong, Action::Cast(PHOTO, Aim::Unit(10)));
    idle(&mut strong, 60);
    let above = hp - enemy_hp(&strong);
    assert!(below > 0 && above > below * 4, "below threshold {below}, above {above}");
}

#[test]
fn chain_reaction_is_critical_inside_radium() {
    let measure = |radium: bool| {
        let mut a = range_arena();
        if radium {
            manual(&mut a, Action::Cast(RADIUM, Aim::Point(440000, 400000)));
            idle(&mut a, 2);
        }
        idle(&mut a, 3);
        let hp: Vec<i64> = a.units.iter().map(|u| u.f.hp).collect();
        let before = a.core(1).stats.damage;
        manual(&mut a, Action::Cast(CHAIN, Aim::Unit(10)));
        idle(&mut a, 30);
        let _ = hp;
        a.core(1).stats.damage - before
    };
    let (sub, crit) = (measure(false), measure(true));
    assert!(crit > sub, "k = 1 inside Radium ({crit}) beats k = 0.7 ({sub})");
}

#[test]
fn radium_dose_halves_every_half_life() {
    let mut a = range_arena();
    manual(&mut a, Action::Cast(RADIUM, Aim::Point(440000, 400000)));
    let mut per_second = vec![];
    let mut last = 0;
    for _ in 0..4 {
        idle(&mut a, 60);
        let d = a.core(1).stats.damage;
        per_second.push(d - last);
        last = d;
    }
    assert!(per_second[0] > 0);
    assert!(per_second[3] * 2 <= per_second[0] + 10, "{per_second:?}");
}

#[test]
fn unification_follows_the_order_of_the_scientists() {
    let mut a = range_arena();
    // Newton → Curie → Einstein: Unified Burst (index 0).
    manual(&mut a, Action::Reserve(APPLE, Aim::Point(440000, 400000)));
    manual(&mut a, Action::Reserve(RADIUM, Aim::Point(440000, 400000)));
    manual(&mut a, Action::Cast(EMC2, Aim::Unit(10)));
    assert_eq!(a.core(1).stats.unifications, 1);
    // Cooldown: a second three-science combo right away does not unify again.
    manual(&mut a, Action::Reserve(PRISM, Aim::Unit(10)));
    manual(&mut a, Action::Reserve(ALPHA, Aim::Unit(10)));
    manual(&mut a, Action::Cast(PHOTO, Aim::Unit(10)));
    assert_eq!(a.core(1).stats.unifications, 1);
    assert_eq!(UNIFICATION[0].0, [1, 2, 0]);
}

#[test]
fn the_ultimate_replays_the_last_combo_without_charges() {
    let mut a = range_arena();
    manual(&mut a, Action::Reserve(PRISM, Aim::Unit(10)));
    manual(&mut a, Action::Cast(ALPHA, Aim::Unit(10)));
    idle(&mut a, 30);
    let charges = a.core(1).ready_charges(a.tick);
    manual(&mut a, Action::Ult(Aim::Unit(10)));
    idle(&mut a, ULT_WARN_TICKS + 2);
    let c = a.core(1);
    assert_eq!(c.stats.ults, 1);
    assert_eq!(c.stats.casts, 2);
    assert_eq!(c.ready_charges(a.tick), charges);
    assert!(c.ult_at >= a.tick + ULT_COOLDOWN_TICKS - ULT_WARN_TICKS - 3);
    manual(&mut a, Action::Ult(Aim::Unit(10)));
    idle(&mut a, ULT_WARN_TICKS + 2);
    assert_eq!(a.core(1).stats.ults, 1, "on cooldown");
}

#[test]
fn twin_paradox_restores_position_and_hp() {
    let mut a = range_arena();
    idle(&mut a, 60);
    let (x0, hp0) = (a.units[0].f.x, a.units[0].f.hp);
    a.units[0].f.x += 20000;
    a.units[0].f.hp -= 500;
    idle(&mut a, 12);
    manual(&mut a, Action::Cast(TWIN, Aim::Unit(1)));
    assert_eq!(a.units[0].f.x, x0);
    assert_eq!(a.units[0].f.hp, hp0);
    idle(&mut a, EQS[TWIN].cooldown + 2);
    a.units[0].f.hp -= 500;
    idle(&mut a, 12);
    let hp = a.units[0].f.hp;
    manual(&mut a, Action::Cast(TWIN, Aim::Unit(1)));
    assert_eq!(a.units[0].f.hp, hp, "the same ally travels once a minute");
}

#[test]
fn damage_is_limited_per_target() {
    let mut a = range_arena();
    let cap = a.units[1].f.max_hp * LIMIT_PCT / 100;
    for s in [PHOTO, ALPHA, EMC2, BETA, PRISM] {
        manual(&mut a, Action::Cast(s, Aim::Unit(10)));
    }
    idle(&mut a, 40);
    let lost = a.units[1].f.max_hp - a.units[1].f.hp;
    assert!(lost <= cap, "lost {lost} cap {cap}");
}

#[test]
fn deterministic_and_bounded() {
    let mut a = fight(5, 7, 4);
    let mut b = fight(5, 7, 4);
    a.run(1800);
    b.run(1800);
    assert_eq!(a.core(1).stats, b.core(1).stats);
    assert!(a.core(1).live.len() <= 96);
}

/// Unified (rank r) and two allies against three moving enemies, one strafing.
fn fight(seed: u64, rank: usize, foes: usize) -> Arena {
    let mut units = vec![Unit::new(1, 0, 380000, 400000, 2600)];
    units.push(Unit::new(2, 0, 375000, 385000, 2600));
    for k in 0..foes {
        let mut e = Unit::new(20 + k, 1, 470000 + 4000 * k as i64, 392000 + 8000 * k as i64, 2600);
        e.strafe = k % 2 == 1;
        e.f.max_hp = 2600 + 400 * k as i64;
        e.f.hp = e.f.max_hp;
        units.push(e);
    }
    Arena::new(units).with_core(1, seed, rank)
}

struct Report {
    damage: i64,
    combo_len: f64,
    signatures: usize,
    insights: usize,
    efficiency: f64,
    misplays: usize,
}
fn report(rank: usize, seeds: u64) -> Report {
    let (mut damage, mut steps, mut casts, mut sig, mut ins, mut eff, mut measured, mut miss) = (0, 0, 0, 0, 0, 0, 0, 0);
    for seed in 0..seeds {
        let mut a = fight(seed + 1, rank, 3);
        a.run(1500);
        let c = a.core(1);
        damage += c.stats.damage;
        steps += c.stats.steps;
        casts += c.stats.casts;
        sig += c.stats.signatures;
        ins += c.stats.insights;
        eff += c.stats.efficiency;
        measured += c.stats.measured;
        miss += c.stats.misplays;
    }
    Report {
        damage: damage / seeds as i64,
        combo_len: steps as f64 / casts.max(1) as f64,
        signatures: sig,
        insights: ins,
        efficiency: eff as f64 / measured.max(1) as f64,
        misplays: miss,
    }
}

#[test]
fn rank_fixtures() {
    let seeds = std::env::var("UNIFIED_SEEDS").ok().and_then(|s| s.parse().ok()).unwrap_or(12);
    let reports: Vec<Report> = (0..8).map(|r| report(r, seeds)).collect();
    for (r, x) in reports.iter().enumerate() {
        eprintln!("rank {r} {:<14} damage {:>6} combo {:.2} signatures {:>3} insights {:>4} efficiency {:.0} misplays {}", RANKS[r], x.damage, x.combo_len, x.signatures, x.insights, x.efficiency, x.misplays);
    }
    let (lo, hi) = (&reports[0], &reports[7]);
    assert!(hi.damage * 10 >= lo.damage * 15, "rank 7 deals ≥ 1.5 × rank 0: {} vs {}", hi.damage, lo.damage);
    assert!(hi.combo_len > lo.combo_len + 0.8, "longer combos at the top");
    assert!(reports[0].signatures == 0 && hi.signatures > 0, "only skilled ranks plan signatures");
    assert!(hi.insights > lo.insights * 3 + 3);
    assert!(lo.misplays > hi.misplays * 3 + 2, "Students waste casts: {} vs {}", lo.misplays, hi.misplays);
    // Monotone enough: every rank at least matches the one two below it.
    for r in 2..8 {
        assert!(reports[r].damage * 100 >= reports[r - 2].damage * 95, "rank {r} vs {}", r - 2);
    }
}

#[test]
fn rank_seven_beats_rank_zero_head_to_head() {
    let seeds = std::env::var("UNIFIED_SEEDS").ok().and_then(|s| s.parse().ok()).unwrap_or(20u64);
    let mut wins = 0;
    for seed in 0..seeds {
        // Two Unified duelists with one escort each; sides alternate to cancel positional bias.
        let flip = seed % 2 == 1;
        let (x_hi, x_lo) = if flip { (480000, 380000) } else { (380000, 480000) };
        let units = vec![
            Unit::new(1, 0, x_hi, 400000, 2600),
            Unit::new(3, 0, x_hi - 5000, 390000, 2000),
            Unit::new(2, 1, x_lo, 400000, 2600),
            Unit::new(4, 1, x_lo + 5000, 410000, 2000),
        ];
        let mut a = Arena::new(units).with_core(1, seed + 1, 7).with_core(2, seed + 101, 0);
        for _ in 0..3600 {
            a.step_with(None, true);
            if a.alive(0) == 0 || a.alive(1) == 0 {
                break;
            }
        }
        let hp = |team: usize| a.units.iter().filter(|u| u.f.team == team).map(|u| u.f.hp * 1000 / u.f.max_hp).sum::<i64>();
        if a.alive(1) == 0 || (a.alive(0) > 0 && hp(0) > hp(1)) {
            wins += 1;
        }
    }
    eprintln!("rank 7 won {wins}/{seeds}");
    assert!(wins * 100 >= seeds * 80, "rank 7 won {wins}/{seeds}");
}

#[test]
#[ignore]
fn usage_dump() {
    for rank in [0, 3, 7] {
        let mut a = fight(3, rank, 3);
        let mut died = None;
        for t in 0..1500 {
            a.step_with(None, true);
            if died.is_none() && a.units[0].f.hp == 0 {
                died = Some(t);
            }
        }
        let c = a.core(1);
        eprintln!("rank {rank}: died {died:?} casts {} steps {} combos {} sigs {:?} uses {:?} fizzles {} ults {} dmg {} enemies alive {} hp {:?}",
            c.stats.casts, c.stats.steps, c.stats.combos, c.stats.sig_uses, c.stats.uses, c.stats.fizzles, c.stats.ults, c.stats.damage, a.alive(1),
            a.units.iter().map(|u| u.f.hp).collect::<Vec<_>>());
    }
}

#[test]
fn planner_cost_stays_small() {
    for rank in [0, 7] {
        let mut a = fight(9, rank, 3);
        let start = std::time::Instant::now();
        a.run(1500);
        let per_tick = start.elapsed().as_micros() as f64 / 1500.0;
        eprintln!("rank {rank}: {per_tick:.1} us per tick (core + arena)");
        // Generous for debug builds and slow CI; release runs far below.
        assert!(per_tick < if cfg!(debug_assertions) { 4000.0 } else { 400.0 }, "{per_tick}");
    }
}

#[test]
fn every_effect_the_core_draws_is_shipped() {
    let data = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../mods/tfm2_custom/champion/tfm2_custom_unified_theory.data_champion")).unwrap();
    let mut tags = std::collections::BTreeSet::new();
    let mut collect = |a: &mut Arena, owner: usize, action: Option<Action>, auto: bool| {
        let tick = a.tick;
        let fighters = a.fighters();
        let wall = |_: i64, _: i64| false;
        let i = a.cores.iter().position(|c| c.0 == owner).unwrap();
        let inp = Input { tick, me: owner, fighters: &fighters, automatic: auto, action, ult_unlocked: true, stunned: false, wall: &wall };
        let cmds = a.cores[i].1.step(&inp);
        for c in &cmds {
            if let Cmd::Fx { tag, .. } = c {
                tags.insert(tag.clone());
            }
        }
        a.apply(owner, cmds);
        a.tick += 1;
    };
    let aim = |k: usize| match EQS[k].shape {
        Shape::Zone | Shape::Lob | Shape::Planet => Aim::Point(440000, 400000),
        Shape::Ally => Aim::Unit(1),
        _ => Aim::Unit(10),
    };
    for t in 0..18 {
        for s in 0..18 {
            let mut a = range_arena();
            for _ in 0..30 {
                collect(&mut a, 1, None, false);
            }
            collect(&mut a, 1, Some(Action::Reserve(t, aim(t))), false);
            collect(&mut a, 1, Some(Action::Cast(s, aim(s))), false);
            for _ in 0..200 {
                collect(&mut a, 1, None, false);
            }
        }
    }
    for sig in SIGNATURES.iter() {
        let mut a = range_arena();
        let steps: Vec<usize> = sig.steps.iter().map(|k| usize::from(*k)).collect();
        for (i, k) in steps.iter().enumerate() {
            let act = if *k == usize::from(ULT_STEP) { Action::Ult(Aim::Unit(10)) } else if i + 1 == steps.len() { Action::Cast(*k, aim(*k)) } else { Action::Reserve(*k, aim(*k)) };
            collect(&mut a, 1, Some(act), false);
        }
        for _ in 0..300 {
            collect(&mut a, 1, None, false);
        }
    }
    for rank in [0, 7] {
        let mut a = fight(4, rank, 3);
        for _ in 0..1500 {
            collect(&mut a, 1, None, true);
        }
    }
    let missing: Vec<&String> = tags.iter().filter(|t| !data.contains(&format!("\"tfm2_custom_unified_theory_{t}\""))).collect();
    assert!(missing.is_empty(), "effects without art: {missing:?}");
    assert!(tags.len() > 150, "{} tags exercised", tags.len());
}
