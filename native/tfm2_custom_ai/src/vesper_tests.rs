//! Vesper, the Rift Gauntlets: rule tests on the pure core, plus duel and 5v5 fixtures per rank in a small arena.
use crate::vesper_core::*;
use crate::vesper_data::*;
use crate::vesper_math::*;
use std::collections::BTreeMap;

const NO_WALL: fn(i64, i64) -> bool = |_, _| false;

#[test]
fn farming_uses_rifts_without_major_moves_or_champion_takedown_credit() {
    let me=Unit::new(0,0,400000,400000,1000,120,66000);
    let mut units=vec![me];
    for id in 1..=3 {
        let mut creep=Unit::new(id,1,420000+id as i64*3000,400000,5000,0,20000);
        creep.f.champion=false;creep.f.hp=600;creep.speed=0;creep.next_attack=100000;
        units.push(creep);
    }
    let mut arena=Arena::new(42,7,units);arena.core.farm_only=true;
    arena.run(1200);
    assert!(arena.core.stats.damage>0);
    assert_eq!(arena.core.stats.takedowns,0);
    for tech in [M_SHATTER,M_OVERLOAD,M_AWAKEN,M_OVERCLOCK] {
        assert_eq!(arena.core.stats.move_uses[tech-28],0);
    }
    assert_eq!(arena.core.stats.kidnaps,0);
}

/// A tiny deterministic arena: scripted movement and basic attacks for everyone, Vesper's rules from the core.
#[derive(Clone, Debug)]
pub struct Unit {
    pub f: Fighter,
    pub speed: i64,
    pub range: i64,
    pub interval: usize,
    pub next_attack: usize,
    pub stun: usize,
    pub root: usize,
    pub disarm: usize,
    pub banish: usize,
    pub invisible: usize,
    pub shield: i64,
    pub buffs: BTreeMap<String, (usize, Mods)>,
    pub dealt: i64,
}
impl Unit {
    pub fn new(id: usize, team: usize, x: i64, y: i64, max_hp: i64, attack: i64, range: i64) -> Unit {
        Unit {
            f: Fighter { champion:true, id, team, x, y, hp: max_hp, max_hp, attack, cc_immune: false, controlled: false },
            speed: 900,
            range,
            interval: 70,
            next_attack: 0,
            stun: 0,
            root: 0,
            disarm: 0,
            banish: 0,
            invisible: 0,
            shield: 0,
            buffs: BTreeMap::new(),
            dealt: 0,
        }
    }
    fn mods(&self, tick: usize) -> Mods {
        let mut m = Mods::default();
        for (until, b) in self.buffs.values() {
            if *until > tick {
                m.move_speed += b.move_speed;
                m.attack_speed += b.attack_speed;
                m.amplify += b.amplify;
                m.reduce += b.reduce;
                m.attack += b.attack;
                m.cc_immune |= b.cc_immune;
                m.undying |= b.undying;
            }
        }
        m
    }
}
pub struct Arena {
    pub tick: usize,
    pub units: Vec<Unit>,
    pub core: Core,
    pub wall: fn(i64, i64) -> bool,
    pub fx: usize,
    pub cmds: Vec<Cmd>,
    pub keep_cmds: bool,
    pub vesper_alive: bool,
    pub frozen: bool,
}
impl Arena {
    pub fn new(seed: u64, rank: usize, units: Vec<Unit>) -> Arena {
        Arena { tick: 0, units, core: Core::new(seed, 0, rank, None), wall: NO_WALL, fx: 0, cmds: vec![], keep_cmds: false, vesper_alive: true, frozen: false }
    }
    pub fn unit(&self, id: usize) -> &Unit {
        self.units.iter().find(|u| u.f.id == id).unwrap()
    }
    pub fn unit_mut(&mut self, id: usize) -> &mut Unit {
        self.units.iter_mut().find(|u| u.f.id == id).unwrap()
    }
    pub fn fighters(&self) -> Vec<Fighter> {
        let tick = self.tick;
        self.units
            .iter()
            .filter(|u| u.f.hp > 0 && u.banish <= tick && (u.f.team == 0 || u.invisible <= tick))
            .map(|u| {
                let mut f = u.f.clone();
                let m = u.mods(tick);
                f.cc_immune = m.cc_immune;
                f.controlled = u.stun > tick || u.root > tick || u.disarm > tick;
                f
            })
            .collect()
    }
    pub fn input_cmds(&mut self, a: Action) -> Vec<Cmd> {
        let fighters = self.fighters();
        let wall = self.wall;
        let w = move |x: i64, y: i64| wall(x, y);
        let input = Input { tick: self.tick, me: 0, fighters: &fighters, attack_interval: 70, stunned: false, wall: &w };
        let cmds = self.core.place(&input, a);
        self.apply(cmds.clone());
        cmds
    }
    fn hurt(&mut self, target: usize, amount: i64, from: Option<usize>) {
        let tick = self.tick;
        let Some(u) = self.units.iter_mut().find(|u| u.f.id == target && u.f.hp > 0) else { return };
        if u.banish > tick {
            return;
        }
        let m = u.mods(tick);
        let mut n = amount * (100 + m.amplify as i64 - m.reduce as i64).max(10) / 100;
        let absorbed = n.min(u.shield);
        u.shield -= absorbed;
        n -= absorbed;
        u.f.hp -= n;
        if m.undying && u.f.hp < 1 {
            u.f.hp = 1;
        }
        if let Some(src) = from {
            if let Some(s) = self.units.iter_mut().find(|x| x.f.id == src) {
                s.dealt += n;
            }
        }
    }
    pub fn apply(&mut self, cmds: Vec<Cmd>) {
        let tick = self.tick;
        for c in cmds {
            match &c {
                Cmd::Damage { target, amount, .. } => self.hurt(*target, *amount, Some(0)),
                Cmd::Heal { target, amount } => {
                    if let Some(u) = self.units.iter_mut().find(|u| u.f.id == *target && u.f.hp > 0) {
                        u.f.hp = (u.f.hp + amount).min(u.f.max_hp);
                    }
                }
                Cmd::Shield { target, amount, .. } => self.unit_mut(*target).shield += amount,
                Cmd::Cc { target, kind, ticks } => {
                    let u = self.unit_mut(*target);
                    let until = tick + ticks;
                    match kind {
                        Cc::Stun => u.stun = u.stun.max(until),
                        Cc::Root => u.root = u.root.max(until),
                        Cc::Disarm => u.disarm = u.disarm.max(until),
                        Cc::Silence => {}
                    }
                }
                Cmd::Buff { target, name, ticks, mods } => {
                    self.unit_mut(*target).buffs.insert(name.clone(), (tick + ticks, *mods));
                }
                Cmd::SetPos { target, x, y } => {
                    let u = self.unit_mut(*target);
                    u.f.x = *x;
                    u.f.y = *y;
                }
                Cmd::Banish { target, ticks } => self.unit_mut(*target).banish = tick + ticks,
                Cmd::Invisible { target, ticks } => self.unit_mut(*target).invisible = tick + ticks,
                Cmd::ClearCc { target } => {
                    let u = self.unit_mut(*target);
                    u.stun = 0;
                    u.root = 0;
                    u.disarm = 0;
                }
                Cmd::ClearShield { target } => self.unit_mut(*target).shield = 0,
                Cmd::LoseHp { target, amount } => {
                    let u = self.unit_mut(*target);
                    u.f.hp = (u.f.hp - amount).max(1);
                }
                Cmd::Fx { .. } => self.fx += 1,
                Cmd::Log(_) => {}
            }
            if self.keep_cmds {
                self.cmds.push(c);
            }
        }
    }
    /// One tick: Vesper's core, then everyone else's scripted movement and attacks.
    pub fn step(&mut self) {
        let tick = self.tick;
        let fighters = self.fighters();
        let vesper = self.unit(0).clone();
        if vesper.f.hp > 0 {
            let wall = self.wall;
            let w = move |x: i64, y: i64| wall(x, y);
            let input = Input { tick, me: 0, fighters: &fighters, attack_interval: vesper.interval, stunned: vesper.stun > tick || vesper.banish > tick, wall: &w };
            let cmds = self.core.step(&input);
            self.apply(cmds);
        } else if self.vesper_alive {
            self.vesper_alive = false;
            self.core.on_dead(tick);
        }
        if !self.frozen {
            let ids: Vec<usize> = self.units.iter().filter(|u| u.f.hp > 0).map(|u| u.f.id).collect();
            for id in ids {
                self.act(id);
            }
        }
        self.tick += 1;
    }
    fn act(&mut self, id: usize) {
        let tick = self.tick;
        let u = self.unit(id).clone();
        if u.f.hp <= 0 || u.stun > tick || u.banish > tick {
            return;
        }
        let aiming = id == 0 && self.core.aiming;
        let foes: Vec<Unit> = self.units.iter().filter(|o| o.f.team != u.f.team && o.f.hp > 0 && o.banish <= tick && o.invisible <= tick).cloned().collect();
        let Some(target) = foes.iter().min_by_key(|o| (dist2(o.f.x, o.f.y, u.f.x, u.f.y), o.f.id)) else { return };
        let d = dist(u.f.x, u.f.y, target.f.x, target.f.y);
        let m = u.mods(tick);
        if d > u.range {
            if u.root <= tick && !aiming {
                let speed = u.speed * (100 + m.move_speed as i64).max(0) / 100;
                // Vesper's enemies follow the game's input-AI rule: out of her rifts and Kidnap fields, never into them
                let zones = if u.f.team != 0 { crate::vesper::zones_of(&self.core, tick) } else { vec![] };
                let (nx, ny) = match crate::vesper::steer(&zones, (u.f.x, u.f.y), (target.f.x, target.f.y)) {
                    Some(g) => along(u.f.x, u.f.y, g.0 - u.f.x, g.1 - u.f.y, speed.min(dist(u.f.x, u.f.y, g.0, g.1))),
                    None => along(u.f.x, u.f.y, target.f.x - u.f.x, target.f.y - u.f.y, speed.min(d - u.range + 1)),
                };
                let w = self.wall;
                let (nx, ny) = land(u.f.x, u.f.y, nx, ny, &move |x, y| w(x, y));
                let me = self.unit_mut(id);
                me.f.x = nx;
                me.f.y = ny;
            }
        } else if u.disarm <= tick && tick >= u.next_attack {
            let interval = (u.interval as i64 * 100 / (100 + m.attack_speed as i64).max(20)) as usize;
            self.unit_mut(id).next_attack = tick + interval.max(10);
            let dmg = u.f.attack * (100 + m.attack as i64).max(0) / 100;
            let tid = target.f.id;
            self.hurt(tid, dmg, Some(id));
            if id == 0 {
                self.core.stats.damage += dmg;
                self.core.stats.jab_damage += dmg;
            }
        }
    }
    pub fn run(&mut self, ticks: usize) {
        for _ in 0..ticks {
            self.step();
            if self.wiped().is_some() {
                break;
            }
        }
    }
    pub fn wiped(&self) -> Option<usize> {
        for team in 0..2 {
            if !self.units.iter().any(|u| u.f.team == team && u.f.hp > 0) {
                return Some(team);
            }
        }
        None
    }
}


fn duel(seed: u64, rank: usize) -> Arena {
    let units = vec![
        Unit::new(0, 0, 200_000, 480_000, 1100, 90, NORMAL_RANGE),
        Unit::new(5, 1, 520_000, 480_000, 2400, 60, 30_000),
    ];
    Arena::new(seed, rank, units)
}
/// Vesper plus a tank, a bruiser, a mage and a healer-ish support against a mirrored line-up.
pub fn teamfight(seed: u64, rank: usize) -> Arena {
    // odd seeds start close (the engage window is open from the first think), even seeds start far apart
    let (home, away) = if seed % 2 == 1 { (300_000, 600_000) } else { (260_000, 700_000) };
    let mut units = vec![Unit::new(0, 0, home - 80_000, 480_000, 1100, 90, NORMAL_RANGE)];
    let roles = [(2600, 55, 25_000), (1800, 80, 28_000), (1200, 95, 60_000), (1300, 60, 55_000)];
    for (i, (hp, atk, range)) in roles.iter().enumerate() {
        let y = 400_000 + i as i64 * 50_000;
        units.push(Unit::new(1 + i, 0, home - (*range / 3), y, *hp, *atk, *range));
    }
    let foes = [(2600, 55, 25_000), (1800, 80, 28_000), (1200, 95, 60_000), (1300, 60, 55_000), (1100, 90, ATTACK_RANGE)];
    // the enemy line-up carries +45% HP and +20% attack: an even fight for a mid-rank Vesper, so decisions decide it
    // (0.10.48 cut her damage, control and heals by a fifth, so the line-up came down from +55%/+30%;
    // VS_FOE=hp,attack in percent overrides it for calibration)
    let (hp_pct, atk_pct): (i64, i64) = std::env::var("VS_FOE").ok().and_then(|v| {
        let mut it = v.split(',').filter_map(|x| x.parse().ok());
        Some((it.next()?, it.next()?))
    }).unwrap_or((145, 120));
    for (i, (hp, atk, range)) in foes.iter().enumerate() {
        let y = 380_000 + i as i64 * 50_000;
        units.push(Unit::new(10 + i, 1, away + (*range / 3), y, *hp * hp_pct / 100, *atk * atk_pct / 100, *range));
    }
    // a little jitter per seed so fixtures cover many shapes
    let mut rng = seed.max(1);
    for u in units.iter_mut().skip(1) {
        u.f.y += roll(&mut rng, 40_000) as i64 - 20_000;
        u.f.x += roll(&mut rng, 30_000) as i64 - 15_000;
    }
    Arena::new(seed, rank, units)
}

fn tick_input<'a>(fighters: &'a [Fighter], tick: usize, w: &'a dyn Fn(i64, i64) -> bool) -> Input<'a> {
    Input { tick, me: 0, fighters, attack_interval: 70, stunned: false, wall: w }
}
fn place(a: &mut Arena, tech: usize, rifts: Vec<RiftSpec>, subject: Option<usize>, uses: Vec<u32>) {
    a.input_cmds(Action { subject, uses, ..Action::new(tech, rifts) });
}
/// A rift spec with its dimension chosen by hand (Rift Lab style).
fn with(dim: usize, s: RiftSpec) -> RiftSpec {
    RiftSpec { dim: Some(dim), ..s }
}
fn hand(seed: u64, rank: usize, team: bool) -> Arena {
    let mut a = if team { teamfight(seed, rank) } else { duel(seed, rank) };
    a.core.hand_placed = true;
    // effect tests repeat one dimension; the cooldowns have their own test
    a.core.free_casts = true;
    a.frozen = true;
    a
}
fn damage_to(a: &Arena, id: usize) -> i64 {
    a.cmds.iter().filter_map(|c| if let Cmd::Damage { target, amount, .. } = c { (*target == id).then_some(*amount) } else { None }).sum()
}
fn has_fx(a: &Arena, prefix: &str) -> bool {
    a.cmds.iter().any(|c| matches!(c, Cmd::Fx { tag, .. } if tag.starts_with(prefix)))
}
/// Every pure orb and hexorb spent, recovering far in the future.
fn drain(a: &mut Arena) {
    for s in a.core.pure.iter_mut() {
        s.ready_at = 1_000_000;
    }
    a.core.hex = [1_000_000; 6];
}

// ------------------------------------------------------------------ orbs and rifts

#[test]
fn orbs_open_rifts_cap_and_recover() {
    let mut a = hand(7, 7, false);
    assert_eq!((a.core.pure_ready(0), a.core.hex_ready(0)), (5, 2), "5 pure orbs and 2 hexorbs at a 70% Hexcore");
    for i in 0..5 {
        place(&mut a, T_CHAIN, vec![with(12, RiftSpec::node(R_TEAR, 250_000 + i * 30_000, 480_000))], None, vec![]);
    }
    assert_eq!(a.core.pure_ready(0), 0, "pure orbs are spent first by hand");
    assert_eq!(a.core.open_count(), 5);
    place(&mut a, T_CHAIN, vec![with(12, RiftSpec::node(R_TEAR, 400_000, 480_000))], None, vec![]);
    assert_eq!(a.core.open_count(), 5, "at most 5 rifts are open");
    assert_eq!(a.core.completion, HEX_START + 5 * RESEARCH_ORB, "every pure orb researches the Hexcore");
    drain_hex_only(&mut a);
    a.run(ORB_CD as usize + 100);
    assert_eq!(a.core.open_count(), 0, "rifts close by themselves");
    assert_eq!(a.core.pure_ready(a.tick), 5, "each orb recovers in 4 s");
}
fn drain_hex_only(a: &mut Arena) {
    a.core.hex = [1_000_000; 6];
}

#[test]
fn dimension_cooldowns_and_family_recovery() {
    let mut a = hand(7, 7, false);
    a.core.free_casts = false;
    let tear = |dim: usize, x: i64| with(dim, RiftSpec::node(R_TEAR, x, 480_000));
    place(&mut a, T_CHAIN, vec![tear(12, 250_000)], None, vec![]);
    assert_eq!(a.core.open_count(), 1);
    assert_eq!(a.core.dim_ready[12], DIMS[12].cooldown, "the dimension rests for its own cooldown");
    assert_eq!(a.core.family_ready[1], FAMILY_RECOVERY as usize, "and its family for 2 s");
    place(&mut a, T_CHAIN, vec![tear(12, 280_000)], None, vec![]);
    assert_eq!(a.core.open_count(), 1, "a resting dimension is refused");
    place(&mut a, T_CHAIN, vec![tear(13, 280_000)], None, vec![]);
    assert_eq!(a.core.open_count(), 1, "so is another dimension of a resting family");
    assert!(!a.core.available_dims(a.tick).iter().any(|d| DIMS[*d].family == 1), "the planner never sees them");
    // one cast's rifts lock together: both ends open, then both rest
    place(&mut a, T_GATE, vec![tear(20, 300_000), tear(21, 340_000)], Some(1), vec![]);
    assert_eq!(a.core.open_count(), 3, "two dimensions of one family open in one cast");
    assert!(!a.core.cast_ok(20, a.tick) && !a.core.cast_ok(21, a.tick));
    assert!(a.core.cast_ok(0, a.tick), "other families are free");
    drain_hex_only(&mut a);
    a.run(FAMILY_RECOVERY as usize + 1);
    assert!(a.core.cast_ok(13, a.tick), "the family is back after 2 s");
    assert!(!a.core.cast_ok(12, a.tick), "the dimension itself still rests");
    // the classes: signature dimensions rest longest
    assert_eq!(DIMS[0].cooldown, CD_SIGNATURE as usize);
    assert_eq!(DIMS[12].cooldown, CD_DAMAGE as usize);
    assert_eq!(DIMS[9].cooldown, CD_LIGHT as usize);
    assert_eq!(DIMS[21].cooldown, CD_MOVE as usize);
    assert_eq!(DIMS[32].cooldown, CD_STRONG as usize);
    // death clears every timer
    a.core.on_dead(a.tick);
    assert!(a.core.cast_ok(12, a.tick));
}

#[test]
fn a_resting_dimension_leaves_her_the_plain_jab() {
    let mut a = hand(3, 7, false);
    a.core.free_casts = false;
    a.core.armed = Some(12);
    a.unit_mut(5).f.x = 200_000 + 40_000;
    a.run(20);
    assert_eq!(a.core.stats.punches, 1, "one rift punch, then the dimension rests");
    assert!(!a.core.punching, "she jabs while it rests");
    let hp = a.unit(5).f.hp;
    a.frozen = false;
    a.run(120);
    assert!(a.unit(5).f.hp < hp, "and the jab still hits");
    assert_eq!(a.core.stats.punches, 1);
}

#[test]
fn heals_share_an_allowance() {
    let mut a = hand(5, 7, true);
    let fighters = a.fighters();
    let me = fighters.iter().find(|f| f.id == 0).cloned().unwrap();
    let ally = fighters.iter().find(|f| f.id == 1).cloned().unwrap();
    let w = |_: i64, _: i64| false;
    let healed = |out: &[Cmd]| -> i64 { out.iter().filter_map(|c| if let Cmd::Heal { target: 1, amount } = c { Some(*amount) } else { None }).sum() };
    let mut out = vec![];
    let t = Tick { tick: 100, me: me.clone(), fighters: &fighters, wall: &w, attack_interval: 70, stunned: false };
    for _ in 0..10 {
        a.core.heal(&mut out, &t, 1, ally.max_hp / 10, 100);
    }
    assert_eq!(healed(&out), ally.max_hp * HEAL_PCT / 100, "every heal she gives shares 20% of max HP per 3 s");
    let mut later = vec![];
    let t = Tick { tick: 100 + HEAL_WINDOW as usize, me, fighters: &fighters, wall: &w, attack_interval: 70, stunned: false };
    a.core.heal(&mut later, &t, 1, ally.max_hp / 10, t.tick);
    assert_eq!(healed(&later), ally.max_hp / 10, "and the allowance comes back");
}

/// The first dimension of `family` whose rift zone is `prim`.
fn zone_dim(family: usize, prim: u8) -> usize {
    (FAMILY_START[family]..FAMILY_START[family] + FAMILY_LEN[family]).find(|&d| DIMS[d].prim == prim).expect("a dimension with that zone")
}

#[test]
fn gravity_wells_draw_in_and_repulsors_push_out() {
    for (prim, closer) in [(P_WELL, true), (P_REPEL, false)] {
        let dim = zone_dim(10, prim);
        let mut a = hand(7, 7, true);
        let e = a.unit(10).f.clone();
        let spot = (e.x + 8000, e.y);
        place(&mut a, T_CHAIN, vec![with(dim, RiftSpec::node(R_TEAR, spot.0, spot.1))], None, vec![]);
        a.run(100);
        let (before, after) = (dist(e.x, e.y, spot.0, spot.1), dist(a.unit(10).f.x, a.unit(10).f.y, spot.0, spot.1));
        assert!(if closer { after < before } else { after > before }, "{}: {before} -> {after}", DIMS[dim].name);
    }
}

#[test]
fn frost_stacks_freeze_and_fade_when_she_steps_out() {
    let dim = zone_dim(11, P_CHILL);
    let need = DIMS[dim].c;
    let mut a = hand(7, 7, true);
    a.keep_cmds = true;
    let e = a.unit(10).f.clone();
    // off the core (an enemy on the core breaks the rift in 1 s), inside the radius
    place(&mut a, T_CHAIN, vec![with(dim, RiftSpec::node(R_TEAR, e.x + 10_000, e.y))], None, vec![]);
    drain(&mut a);
    a.run(PULSE as usize * need as usize + 60);
    assert!(a.cmds.iter().any(|c| matches!(c, Cmd::Cc { target: 10, kind: Cc::Root, .. })), "{need} stacks freeze");
    let (n, _) = a.core.chill[&10];
    assert!(n < need, "and the stacks clear");
    // out of the zone the stacks fade: the next chill starts again at one
    a.unit_mut(10).f.x += 200_000;
    a.run(CHILL_FADE as usize + PULSE as usize);
    let back = a.tick;
    a.unit_mut(10).f.x = e.x;
    a.run(PULSE as usize + 2);
    let (n, last) = a.core.chill[&10];
    assert!(last >= back && n == 1, "stepping out sheds the stacks ({n})");
}

#[test]
fn pressure_stores_charges_and_bursts_on_close() {
    let dim = zone_dim(13, P_PRESSURE);
    let cap = DIMS[dim].c;
    let mut a = hand(7, 7, true);
    a.keep_cmds = true;
    let e = a.unit(10).f.clone();
    place(&mut a, T_CHAIN, vec![with(dim, RiftSpec::node(R_TEAR, e.x + 10_000, e.y))], None, vec![]);
    drain(&mut a); // no punches through the chain: only the zone acts
    let mut most = 0;
    for _ in 0..(RIFTS[R_TEAR].life + 120) {
        a.step();
        if let Some(r) = a.core.rifts.first() {
            most = most.max(r.charge);
        }
        if a.core.rifts.is_empty() {
            break;
        }
    }
    assert_eq!(most, cap, "charges stop at the cap");
    let burst = e.max_hp * DIMS[dim].a * cap / 1000;
    assert!(a.cmds.iter().any(|c| matches!(c, Cmd::Damage { target: 10, amount, kind: Dmg::Magic } if *amount == burst)), "the closing rift bursts for {burst}");
}

#[test]
fn momentum_is_bounded_and_zero_standing_still() {
    let a = hand(7, 7, false);
    let fighters = a.fighters();
    let mut me = fighters.iter().find(|f| f.id == 0).cloned().unwrap();
    let w = |_: i64, _: i64| false;
    let mut core = a.core.clone();
    core.history.insert(0, std::collections::VecDeque::from(vec![(me.x, me.y, me.hp); 12]));
    let t = Tick { tick: 100, me: me.clone(), fighters: &fighters, wall: &w, attack_interval: 70, stunned: false };
    assert_eq!(core.momentum(&t, 12, 48), 0, "standing still adds nothing");
    me.x += 30_000;
    let t = Tick { tick: 100, me: me.clone(), fighters: &fighters, wall: &w, attack_interval: 70, stunned: false };
    assert_eq!(core.momentum(&t, 12, 48), 36, "+12% per 10000 moved");
    me.x += 300_000;
    let t = Tick { tick: 100, me, fighters: &fighters, wall: &w, attack_interval: 70, stunned: false };
    assert_eq!(core.momentum(&t, 12, 48), 48, "and never past the cap");
}

#[test]
fn an_exposed_target_detonates_on_the_next_punch() {
    let dim = zone_dim(14, P_EXPOSE);
    let mut a = hand(3, 7, false);
    a.core.armed = Some(dim);
    a.unit_mut(5).f.x = 200_000 + 40_000;
    let max_hp = a.unit(5).f.max_hp;
    a.keep_cmds = true;
    while a.core.stats.landed < 1 && a.tick < 300 {
        a.step();
    }
    let (_, pm) = *a.core.exposed.get(&5).expect("the first punch exposes");
    a.cmds.clear();
    while a.core.stats.landed < 2 && a.tick < 600 {
        a.step();
    }
    assert!(a.cmds.iter().any(|c| matches!(c, Cmd::Damage { target: 5, amount, kind: Dmg::Magic } if *amount == max_hp * pm / 1000)), "the second punch detonates it");
}

/// The first dimension of `family` whose punch's main effect is `prim`.
fn punch_dim(family: usize, prim: u8) -> usize {
    (FAMILY_START[family]..FAMILY_START[family] + FAMILY_LEN[family]).find(|&d| PUNCH[d].main.prim == prim).expect("a dimension with that punch")
}
fn hit_on(a: &Arena, id: usize, kind: Dmg) -> Vec<i64> {
    a.cmds.iter().filter_map(|c| if let Cmd::Damage { target, amount, kind: k } = c { (*target == id && *k == kind).then_some(*amount) } else { None }).collect()
}

#[test]
fn resonance_pays_the_next_hit_inside_its_window() {
    let dim = punch_dim(15, P_RESONATE);
    let mut a = hand(3, 7, false);
    a.core.armed = Some(dim);
    a.unit_mut(5).f.x = 200_000 + 40_000;
    while a.core.stats.landed < 1 && a.tick < 300 {
        a.step();
    }
    let (until, pct) = a.core.resonance[&5];
    assert!(until > a.tick && pct > 0, "the punch leaves the target resonating");
    a.keep_cmds = true;
    while a.core.stats.landed < 2 && a.tick < 600 {
        a.step();
    }
    let phys = hit_on(&a, 5, Dmg::Phys);
    assert!(phys.len() >= 2 && phys.iter().any(|x| phys.iter().any(|y| *y == *x * pct / 100 && *y > 0)), "the next hit deals {pct}% again: {phys:?}");
    // outside the window nothing happens
    let mut b = hand(3, 7, false);
    b.core.resonance.insert(5, (0, 50));
    b.unit_mut(5).f.x = 200_000 + 40_000;
    b.core.armed = Some(12);
    b.keep_cmds = true;
    while b.core.stats.landed < 1 && b.tick < 300 {
        b.step();
    }
    assert_eq!(hit_on(&b, 5, Dmg::Phys).len(), 1, "a lapsed resonance adds nothing");
}

#[test]
fn polarity_needs_two_marks_in_range() {
    let dim = zone_dim(16, P_POLE);
    let range = DIMS[dim].a;
    let mut a = hand(7, 7, true);
    a.keep_cmds = true;
    let e1 = a.unit(10).f.clone();
    a.unit_mut(11).f.x = e1.x + 400_000;
    // one mark alone: polarised, nothing else
    place(&mut a, T_CHAIN, vec![with(dim, RiftSpec::node(R_TEAR, e1.x + 10_000, e1.y))], None, vec![]);
    drain(&mut a);
    a.run(80);
    assert!(a.core.poles.contains_key(&10) && hit_on(&a, 10, Dmg::Magic).is_empty(), "one pole waits for a partner");
    // the second enemy, within range, is marked: both snap together and take the damage
    let near = (e1.x + range / 2, e1.y);
    a.unit_mut(11).f.x = near.0;
    a.unit_mut(11).f.y = near.1;
    for s in a.core.pure.iter_mut() {
        s.ready_at = 0; // orbs back for the second mark
    }
    place(&mut a, T_CHAIN, vec![with(dim, RiftSpec::node(R_TEAR, near.0 + 10_000, near.1))], None, vec![]);
    drain(&mut a);
    a.run(80);
    assert!(!hit_on(&a, 10, Dmg::Magic).is_empty() && !hit_on(&a, 11, Dmg::Magic).is_empty(), "both poles take the damage");
    assert!(dist(a.unit(10).f.x, a.unit(10).f.y, a.unit(11).f.x, a.unit(11).f.y) < range / 2, "and snap together");
}

#[test]
fn circuit_relays_a_module_and_arcs_a_punch() {
    let dim = zone_dim(17, P_CIRCUIT);
    let mut a = hand(7, 7, true);
    a.keep_cmds = true;
    let e = a.unit(10).f.clone();
    let other = (e.x + 60_000, e.y);
    place(&mut a, T_CHAIN, vec![with(dim, RiftSpec::node(R_TEAR, e.x + 10_000, e.y)), with(12, RiftSpec::node(R_TEAR, other.0, other.1))], None, vec![]);
    drain(&mut a);
    a.run(60);
    // a module going off inside the Circuit rift also goes off at the other rift
    a.cmds.clear();
    let fighters = a.fighters();
    let me = fighters.iter().find(|f| f.id == 0).cloned().unwrap();
    let w = |_: i64, _: i64| false;
    let t = Tick { tick: a.tick, me, fighters: &fighters, wall: &w, attack_interval: 70, stunned: false };
    let mut out = vec![];
    a.core.fx_left = 10;
    a.core.fx_tokens = 12;
    a.core.module_test(&mut out, &t, 2, e.x + 10_000, e.y);
    let at: Vec<(i64, i64)> = out.iter().filter_map(|c| if let Cmd::Fx { tag, x, y, .. } = c { tag.starts_with("mod2_").then_some((*x, *y)) } else { None }).collect();
    assert_eq!(at, vec![(e.x + 10_000, e.y), other], "the module relays to the nearest other rift");
    // a Circuit punch arcs part of the hit into the nearest live rift
    let pdim = punch_dim(17, P_CIRCUIT);
    let mut b = hand(3, 7, true);
    b.core.armed = Some(pdim);
    let foe = b.unit(10).f.clone();
    place(&mut b, T_CHAIN, vec![with(12, RiftSpec::at(R_TEAR, foe.x - 60_000, foe.y))], None, vec![]);
    b.run(60); // the rift opens before she punches
    b.unit_mut(11).f.x = foe.x - 60_000;
    b.unit_mut(11).f.y = foe.y;
    b.unit_mut(0).f.x = foe.x - 30_000;
    b.unit_mut(0).f.y = foe.y;
    b.keep_cmds = true;
    while b.core.stats.landed < 1 && b.tick < 300 {
        b.step();
    }
    assert!(!hit_on(&b, 11, Dmg::Phys).is_empty(), "the enemy inside the rift takes the arc");
}

#[test]
fn an_oath_strikes_back_once_and_ends_out_of_range() {
    let dim = zone_dim(18, P_OATH);
    let mut a = hand(7, 7, true);
    a.keep_cmds = true;
    let ally = a.unit(1).f.clone();
    place(&mut a, T_CHAIN, vec![with(dim, RiftSpec::node(R_TEAR, ally.x + 5000, ally.y))], None, vec![]);
    drain(&mut a);
    a.run(40);
    assert!(a.core.oaths.contains_key(&1), "the ally is sworn");
    assert!(a.cmds.iter().any(|c| matches!(c, Cmd::Shield { target: 1, .. })), "with a shield");
    // the first hit the ally takes strikes the nearest enemy
    let foe = a.unit(10).f.clone();
    a.unit_mut(10).f.x = ally.x + 20_000;
    a.unit_mut(10).f.y = ally.y;
    a.cmds.clear();
    a.unit_mut(1).f.hp -= 50;
    a.run(2);
    assert!(!hit_on(&a, 10, Dmg::Magic).is_empty() && !a.core.oaths.contains_key(&1), "once");
    let _ = foe;
    // walking away from where it was sworn ends it
    let mut b = hand(7, 7, true);
    let ally = b.unit(1).f.clone();
    place(&mut b, T_CHAIN, vec![with(dim, RiftSpec::node(R_TEAR, ally.x + 5000, ally.y))], None, vec![]);
    drain(&mut b);
    b.run(40);
    assert!(b.core.oaths.contains_key(&1));
    b.unit_mut(1).f.x += OATH_RANGE + 20_000;
    b.run(2);
    assert!(!b.core.oaths.contains_key(&1), "the oath holds only near where it was sworn");
}

#[test]
fn genesis_grows_seed_bloom_storm() {
    let dim = zone_dim(19, P_GENESIS);
    let mut a = hand(7, 7, true);
    a.keep_cmds = true;
    let e = a.unit(10).f.clone();
    place(&mut a, T_CHAIN, vec![with(dim, RiftSpec::node(R_TEAR, e.x + 10_000, e.y))], None, vec![]);
    drain(&mut a);
    let life = RIFTS[R_TEAR].life;
    let opens = a.core.rifts[0].opens;
    let (mut slowed, mut burned, mut stunned) = (None, None, None);
    for _ in 0..(life + 80) {
        a.cmds.clear();
        a.step();
        for c in &a.cmds {
            match c {
                Cmd::Buff { target: 10, name, .. } if name == &format!("vsd_{}", DIMS[dim].id) => slowed = slowed.or(Some(a.tick)),
                Cmd::Damage { target: 10, kind: Dmg::Magic, .. } => burned = burned.or(Some(a.tick)),
                Cmd::Cc { target: 10, kind: Cc::Stun, .. } => stunned = stunned.or(Some(a.tick)),
                _ => {}
            }
        }
        if a.core.rifts.is_empty() {
            break;
        }
    }
    let (s, b, st) = (slowed.expect("seed"), burned.expect("bloom"), stunned.expect("storm"));
    assert!(s < b && b < st, "the stages come in order: {s} {b} {st}");
    assert!(b >= opens + life / 3 && st >= opens + 2 * life / 3, "a third of the rift's life each");
}

#[test]
fn far_tears_open_later() {
    let mut a = hand(7, 0, false);
    place(&mut a, T_CHAIN, vec![with(12, RiftSpec::node(R_TEAR, 210_000, 480_000)), with(12, RiftSpec::node(R_TEAR, 900_000, 480_000))], None, vec![]);
    let near = a.core.rifts[0].opens;
    let far = a.core.rifts[1].opens;
    assert_eq!(near, open_delay(10_000, 100));
    assert_eq!(far, open_delay(700_000, 100));
    assert!(far > near + 50);
}

#[test]
fn rift_punches_go_through_the_chain_and_lose_per_link() {
    let mut a = hand(3, 7, false);
    a.core.armed = Some(12);
    // the target stands 320000 away: far beyond her reach
    place(&mut a, T_CHAIN, vec![with(12, RiftSpec::node(R_TEAR, 300_000, 480_000)), with(12, RiftSpec::node(R_TEAR, 460_000, 480_000))], None, vec![]);
    a.keep_cmds = true;
    a.run(200);
    assert!(a.core.aiming && a.core.punching, "she plants her feet and punches through the chain");
    assert!(damage_to(&a, 5) > 0, "punches reach a target far out of her reach");
    assert!(a.core.stats.landed > 0);
    assert_eq!(a.core.stats.eff_sum / a.core.stats.landed as i64, chain_bp(&[100_000, 160_000]));
    assert!(a.cmds.iter().any(|c| matches!(c, Cmd::Cc { target: 0, kind: Cc::Disarm, .. })), "the engine attack is held while punching");
    assert!(has_fx(&a, "hole_l7"), "every punch breaks a hole at the target");
}

#[test]
fn direct_punches_and_the_normal_attack() {
    let mut a = hand(3, 7, false);
    a.core.armed = Some(12);
    a.unit_mut(5).f.x = 200_000 + 50_000;
    a.keep_cmds = true;
    a.run(10);
    assert!(a.core.punching && !a.core.aiming, "within punch reach she punches without planting her feet");
    a.run(30);
    assert_eq!(a.core.stats.eff_sum / a.core.stats.landed.max(1) as i64, keep_bp(50_000));
    // out of orbs: back to the normal basic attack
    let mut b = hand(3, 7, false);
    drain(&mut b);
    b.unit_mut(5).f.x = 200_000 + 30_000;
    b.frozen = false;
    b.keep_cmds = true;
    b.run(80);
    assert!(!b.core.punching);
    assert!(!b.cmds.iter().any(|c| matches!(c, Cmd::Cc { target: 0, kind: Cc::Disarm, .. })));
    assert!(b.unit(5).f.hp < b.unit(5).f.max_hp, "the normal attack still hits");
}

#[test]
fn breaking_cuts_the_chain_and_adds_stress() {
    let mut a = hand(3, 7, false);
    place(&mut a, T_CHAIN, vec![with(12, RiftSpec::node(R_TEAR, 520_000, 480_000))], None, vec![]);
    a.run(40 + BREAK_TICKS as usize);
    assert_eq!(a.core.stats.breaks, 1);
    assert_eq!(a.core.open_count(), 0);
    assert!(a.core.stress >= STRESS_BREAK - 2, "a broken rift costs stress");
}

// ------------------------------------------------------------------ the left gauntlet

#[test]
fn overload_heats_and_enemy_hits_crack_the_gauntlet() {
    let mut a = hand(3, 7, false);
    let mv = |m: usize| Action::new(m, vec![]);
    for _ in 0..4 {
        a.input_cmds(mv(M_OVERLOAD));
    }
    assert_eq!(a.core.sockets(), 9);
    assert_eq!(a.core.over_sockets(), 4);
    assert_eq!(a.core.stress, 4 * OVER_STRESS);
    a.input_cmds(mv(M_OVERLOAD));
    assert_eq!(a.core.sockets(), 9, "nine sockets at most");
    let before = a.core.stress;
    a.run(OVER_HEAT_TICKS as usize * 3);
    assert!(a.core.stress >= before + 8, "held overloaded orbs heat the gauntlet");
    // deep into a fight, an enemy hammering her while overloaded drives stress to the cap: forced Shatter, the left gauntlet breaks
    a.core.stress = a.core.stress.max(STRESS_CAP - 40);
    a.keep_cmds = true;
    for _ in 0..12 {
        let u = a.unit_mut(0);
        u.f.hp -= u.f.max_hp / 20;
        a.run(1);
        if a.core.left_broken(a.tick) {
            break;
        }
    }
    assert!(a.core.left_broken(a.tick), "stress {}", a.core.stress);
    assert_eq!(a.core.stats.left_breaks, 1);
    assert_eq!(a.core.pure_ready(a.tick), 0, "no pure orbs while broken");
    assert_eq!(a.core.sockets(), ORBS_BASE as usize, "overload sockets are gone");
    assert!(a.cmds.iter().any(|c| matches!(c, Cmd::LoseHp { target: 0, .. })));
    assert!(a.unit(0).f.hp >= 1);
    a.run(LEFT_BREAK_TICKS as usize + 2);
    assert_eq!(a.core.pure_ready(a.tick), 5, "the gauntlet comes back with full orbs");
}

#[test]
fn shatter_rips_rifts_wide_and_never_hurts_allies() {
    let mut a = hand(5, 7, true);
    let (cx, cy) = (500_000, 480_000);
    for (id, dx) in [(10usize, 4000i64), (11, -6000), (1, 2000)] {
        let u = a.unit_mut(id);
        u.f.x = cx + dx;
        u.f.y = cy + 3000;
    }
    place(&mut a, T_CHAIN, vec![with(12, RiftSpec::node(R_TEAR, cx, cy))], None, vec![]);
    a.run(40);
    let ally_hp = a.unit(1).f.hp;
    let stress = a.core.stress;
    let held = a.core.pure_ready(a.tick) as i64;
    a.keep_cmds = true;
    a.input_cmds(Action::new(M_SHATTER, vec![]));
    assert!(damage_to(&a, 10) > 0 && damage_to(&a, 11) > 0, "enemies inside the rip take the Shatter");
    assert!(a.cmds.iter().any(|c| matches!(c, Cmd::Banish { target: 10, .. })));
    assert_eq!(damage_to(&a, 1), 0);
    assert_eq!(a.unit(1).f.hp, ally_hp, "allies are never hurt");
    assert!(dist(a.unit(1).f.x, a.unit(1).f.y, cx, cy) > RIFTS[R_TEAR].radius * 2, "allies are pushed out of the rip");
    assert_eq!(a.core.open_count(), 0, "every rift collapses");
    assert_eq!(a.core.stress, stress + held * SHATTER_LOAD);
    assert_eq!(a.core.stats.useful_shatters, 1);
    assert!(!a.core.left_broken(a.tick), "the gauntlet held");
    // the same Shatter past the stress cap breaks the left gauntlet
    let mut b = hand(5, 7, true);
    place(&mut b, T_CHAIN, vec![with(12, RiftSpec::node(R_TEAR, cx, cy))], None, vec![]);
    b.run(40);
    b.core.stress = STRESS_CAP - 10;
    b.input_cmds(Action::new(M_SHATTER, vec![]));
    assert!(b.core.left_broken(b.tick));
}

#[test]
fn orbs_combo_and_pair_bonuses() {
    let mut a = hand(3, 7, false);
    a.keep_cmds = true;
    // Static Realm (Ruin) then Anchor (Control) within a second: a combo step and the Ruin + Control bonus
    place(&mut a, T_CHAIN, vec![with(10, RiftSpec::node(R_TEAR, 300_000, 480_000))], None, vec![]);
    a.unit_mut(5).f.x = 470_000;
    a.run(10);
    place(&mut a, T_CHAIN, vec![with(4, RiftSpec::node(R_TEAR, 450_000, 480_000))], None, vec![]);
    assert_eq!(a.core.combo, 1);
    assert_eq!(a.core.stats.twists, 1);
    assert!(has_fx(&a, &format!("twist{}", TWISTS[twist_index(1, 0)].kind)));
    drain(&mut a);
    a.run(COMBO_WINDOW as usize + 5);
    assert_eq!(a.core.combo, 0, "the combo ends a second after the last orb");
}

// ------------------------------------------------------------------ the right gauntlet

#[test]
fn hexcore_researches_awakens_and_fuses() {
    let mut a = hand(3, 7, false);
    assert_eq!(hex_slots(a.core.completion), 2);
    a.core.completion = 99;
    a.input_cmds(Action::new(M_AWAKEN, vec![]));
    assert!(!a.core.awakened(a.tick), "the Hexcore awakens only at 100%");
    a.core.completion = 100;
    assert_eq!(a.core.hex_slots(a.tick), 6);
    a.input_cmds(Action::new(M_AWAKEN, vec![]));
    assert!(a.core.awakened(a.tick));
    assert_eq!(a.core.hex_power(a.tick), 100);
    // a punch while awakened fuses a pure orb and a hexorb
    a.core.armed = Some(12);
    a.unit_mut(5).f.x = 250_000;
    a.keep_cmds = true;
    a.run(90);
    assert!(a.core.stats.fused >= 1, "pure + hex fuse while awakened");
    // rifts cannot be broken while awakened
    place(&mut a, T_CHAIN, vec![with(12, RiftSpec::node(R_TEAR, 250_000, 480_000))], None, vec![]);
    a.run(BREAK_TICKS as usize + 30);
    assert_eq!(a.core.stats.breaks, 0);
    a.unit_mut(5).f.x = 900_000;
    a.run(AWAKEN_TICKS as usize);
    assert!(!a.core.awakened(a.tick));
    assert_eq!(a.core.completion, AWAKEN_AFTER);
}

#[test]
fn overclock_corrupts_and_breaks_the_right_gauntlet() {
    let mut a = hand(5, 7, true);
    a.unit_mut(10).f.x = 225_000;
    a.unit_mut(10).f.y = 480_000;
    a.unit_mut(1).f.x = 215_000;
    a.unit_mut(1).f.y = 480_000;
    a.core.completion = 100;
    a.input_cmds(Action::new(M_AWAKEN, vec![]));
    a.keep_cmds = true;
    let ally = a.unit(1).f.hp;
    // 15 corruption an orb: seven overclocks, so two overload sockets first
    for _ in 0..2 {
        a.input_cmds(Action::new(M_OVERLOAD, vec![]));
    }
    for i in 0..7 {
        a.input_cmds(Action::new(M_OVERCLOCK, vec![]));
        if i < 6 {
            assert_eq!(a.core.corruption, CORRUPT_PER_ORB * (i + 1));
            assert_eq!(a.core.hex_power(a.tick), 100 + CORRUPT_POWER * (i + 1));
        }
    }
    assert!(a.core.right_broken(a.tick), "100 corruption breaks the right gauntlet");
    assert_eq!(a.core.completion, HEX_START);
    assert_eq!(a.core.hex_slots(a.tick), 0);
    assert!(damage_to(&a, 10) > 0, "the hex-burst hits enemies");
    assert_eq!(a.unit(1).f.hp, ally, "never allies");
    assert!(a.cmds.iter().any(|c| matches!(c, Cmd::LoseHp { target: 0, .. })));
    assert_eq!(a.core.stats.right_breaks, 1);
}

#[test]
fn hexorbs_carry_the_athletes_tech_modules() {
    let mut a = hand(3, 7, false);
    a.core.modules = [5, 1, 2, 3, 4, 0];
    drain(&mut a);
    a.core.hex = [0; 6];
    a.core.armed = Some(12);
    a.unit_mut(5).f.x = 250_000;
    a.keep_cmds = true;
    a.run(10);
    assert!(a.core.stats.hexorbs >= 1);
    assert!(a.cmds.iter().any(|c| matches!(c, Cmd::Buff { target: 5, name, .. } if name == "vsm_scanner")), "slot 0 carries Scanner");
    // Infuse: a hexorb's module pulses from an open rift
    let mut b = hand(3, 7, false);
    place(&mut b, T_CHAIN, vec![with(12, RiftSpec::node(R_TEAR, 515_000, 480_000))], None, vec![]);
    b.run(30);
    let uid = b.core.rifts[0].uid;
    b.core.modules = [2, 1, 0, 3, 4, 5];
    place(&mut b, T_INFUSE, vec![], None, vec![uid]);
    assert_eq!(b.core.rift(uid).unwrap().infused, Some(2));
    b.keep_cmds = true;
    b.run(70);
    assert!(b.cmds.iter().any(|c| matches!(c, Cmd::Buff { target: 5, name, .. } if name == "vsm_stasis")));
    assert_eq!(crate::vesper::parse_modules("7 arc bolt 0 1 2 3\n", 7), Some([6, 7, 0, 1, 2, 3]));
    assert_eq!(crate::vesper::parse_modules("7 arc arc 0 1 2 3\n", 7), None, "six different modules");
}

// ------------------------------------------------------------------ the behaviours (three per dimension)

#[test]
fn every_dimension_has_three_distinct_behaviours() {
    let mut seen = std::collections::BTreeSet::new();
    for d in 0..N_DIMS {
        let z = DIMS[d];
        seen.insert((0, z.prim, z.a, z.b, z.c, 255, 0, 0, 0));
        for (k, dl) in [(1, PUNCH[d]), (2, SWIPE[d])] {
            seen.insert((k, dl.main.prim, dl.main.a, dl.main.b, dl.main.c, dl.rider.prim, dl.rider.a, dl.rider.b, dl.rider.c));
        }
    }
    assert_eq!(seen.len(), 3 * N_DIMS);
}

fn limits_hold(a: &Arena, what: &str) {
    let mut moves: BTreeMap<usize, usize> = BTreeMap::new();
    for c in &a.cmds {
        if let Cmd::SetPos { target, .. } = c {
            if *target >= 10 {
                *moves.entry(*target).or_default() += 1;
            }
        }
    }
    for (id, m) in moves {
        assert!(m <= 6, "{what} moved {id} {m} times");
    }
    let control: i64 = a
        .cmds
        .iter()
        .filter_map(|c| match c {
            Cmd::Cc { target: 10, ticks, .. } => Some(*ticks as i64),
            Cmd::Banish { target: 10, ticks } => Some(*ticks as i64),
            _ => None,
        })
        .sum();
    assert!(control <= CONTROL_CAP * 2, "{what} controlled for {control} ticks in 4 s");
    let damage = damage_to(a, 10);
    assert!(damage <= a.unit(10).f.max_hp * DAMAGE_PCT / 100 * 2 + 1, "{what} dealt {damage}");
}
fn touched(a: &Arena) -> bool {
    a.cmds.iter().any(|c| match c {
        Cmd::Damage { target, .. } => *target != 10 || true,
        Cmd::Cc { target: 10 | 11 | 1 | 0, .. } | Cmd::Banish { .. } | Cmd::SetPos { .. } | Cmd::Heal { .. } | Cmd::Shield { .. } | Cmd::Invisible { .. } | Cmd::ClearCc { .. } | Cmd::ClearShield { .. } => true,
        Cmd::Buff { name, .. } => name.starts_with("vsd_"),
        _ => false,
    })
}

#[test]
fn every_dimension_fires_as_rift_punch_and_swipe_inside_the_limits() {
    for dim in 0..N_DIMS {
        let (cx, cy) = (480_000, 480_000);
        let setup = |a: &mut Arena| {
            for id in [1usize, 10, 11] {
                let u = a.unit_mut(id);
                u.f.x = cx + 10_000 + (id as i64 % 3) * 2000;
                u.f.y = cy;
            }
            a.unit_mut(0).f.x = cx - 40_000;
            a.unit_mut(0).f.y = cy;
        };
        // the rift's zone
        let mut a = hand(11, 7, true);
        setup(&mut a);
        a.keep_cmds = true;
        place(&mut a, T_CHAIN, vec![with(dim, RiftSpec::node(R_TEAR, cx, cy))], None, vec![]);
        a.core.armed = Some(12);
        a.run(240);
        limits_hold(&a, &format!("{} zone", DIMS[dim].name));
        let p = DIMS[dim].prim;
        let silent = PRIM_SIDE[p as usize] == b'P' || matches!(p, P_VENT | P_UNSTABLE | P_KALEIDO | P_SIPHON | P_WALL | P_TETHER | P_FOLD | P_SELFINVIS | P_CLEANSE | P_DICE | P_STATICBURST);
        assert!(touched(&a) || silent, "{} zone did nothing", DIMS[dim].name);
        // the punch
        let mut b = hand(11, 7, true);
        setup(&mut b);
        b.keep_cmds = true;
        b.core.armed = Some(dim);
        b.run(240);
        assert!(b.core.stats.landed >= 1, "{} punch never landed", DIMS[dim].name);
        limits_hold(&b, &format!("{} punch", DIMS[dim].name));
        assert!(damage_to(&b, 10) + damage_to(&b, 11) > 0, "{} punch did no damage", DIMS[dim].name);
        // the swipe
        let mut c = hand(11, 7, true);
        setup(&mut c);
        c.keep_cmds = true;
        c.input_cmds(Action { point: Some((cx + 12_000, cy)), dim: Some(dim), subject: Some(10), ..Action::new(M_SWIPE, vec![]) });
        c.run(120);
        assert_eq!(c.core.stats.swipes, 1, "{} swipe", DIMS[dim].name);
        assert!(c.core.stats.swipe_hits >= 2, "{} swipe hit {}", DIMS[dim].name, c.core.stats.swipe_hits);
        limits_hold(&c, &format!("{} swipe", DIMS[dim].name));
    }
}

#[test]
fn rewind_punch_at_full_strength_moves_the_target() {
    // Rewind's punch looks 180 ticks back; the core keeps 3 s of positions, so a full-strength punch has its sample
    assert_eq!(PUNCH[50].main.prim, P_REWIND);
    assert!(HISTORY >= PUNCH[50].main.a, "the history covers the longest Rewind");
    let mut a = hand(3, 7, false);
    a.keep_cmds = true;
    a.unit_mut(0).f.x = 200_000;
    a.unit_mut(5).f.x = 210_000;
    a.unit_mut(5).f.y = a.unit(0).f.y;
    // the target walks right for 4 s, leaving a track
    for _ in 0..240 {
        a.unit_mut(5).f.x += 150;
        a.step();
    }
    let walked_to = a.unit(5).f.x;
    a.cmds.clear();
    a.core.armed = Some(50);
    a.run(60);
    let sent = a.cmds.iter().find_map(|c| if let Cmd::SetPos { target: 5, x, .. } = c { Some(*x) } else { None });
    let x = sent.expect("the Rewind punch sends the target back");
    assert!(x <= walked_to - 15_000, "back along its track: {x} vs {walked_to}");
    assert!(x >= 210_000, "never further back than it has been");
}

#[test]
fn signature_punches_and_swipes() {
    let wall_at = |x: i64, _: i64| x >= 270_000;
    // Gale: the punch throws the target into a wall and the wall slam stuns
    let mut a = hand(3, 7, false);
    a.wall = |x, _| x >= 270_000;
    a.unit_mut(5).f.x = 255_000;
    a.core.armed = Some(22);
    a.keep_cmds = true;
    a.run(30);
    assert!(!wall_at(a.unit(5).f.x, 0));
    assert!(a.cmds.iter().any(|c| matches!(c, Cmd::Cc { target: 5, kind: Cc::Stun, .. })), "Gale slams into the wall");
    // Mirror Realm: knocked out of a mirror on the far side of Vesper
    let mut b = hand(3, 7, false);
    b.unit_mut(5).f.x = 250_000;
    b.core.armed = Some(20);
    b.run(30);
    assert!(b.unit(5).f.x < 200_000, "reflected to her other side ({})", b.unit(5).f.x);
    // Static Realm swipe marks the line; the next punch arcs to every mark
    let mut c = hand(5, 7, true);
    for (id, y) in [(10usize, 0i64), (11, 4000), (12, -4000)] {
        let u = c.unit_mut(id);
        u.f.x = 250_000 + y.abs() * 3;
        u.f.y = 480_000 + y;
    }
    c.input_cmds(Action { point: Some((300_000, 480_000)), dim: Some(10), subject: Some(10), ..Action::new(M_SWIPE, vec![]) });
    assert!(c.core.marks.len() >= 2, "{:?}", c.core.marks);
    c.core.armed = Some(12);
    c.keep_cmds = true;
    c.run(40);
    assert!(c.core.marks.is_empty(), "the punch spent the marks");
    // Glass Wall's swipe leaves a tear twice as long; Swap Space's swipe trades the two ends
    let mut d = hand(5, 7, true);
    d.input_cmds(Action { point: Some((300_000, 480_000)), dim: Some(70), ..Action::new(M_SWIPE, vec![]) });
    let r = d.core.rifts[0].clone();
    assert_eq!(r.expires - r.opens, 2 * RIFTS[R_SLASH].life);
    let mut e = hand(5, 7, true);
    e.unit_mut(10).f.x = 225_000;
    e.unit_mut(10).f.y = 480_000;
    e.unit_mut(11).f.x = 270_000;
    e.unit_mut(11).f.y = 480_000;
    e.input_cmds(Action { point: Some((300_000, 480_000)), dim: Some(25), ..Action::new(M_SWIPE, vec![]) });
    assert!(e.unit(10).f.x > 260_000 && e.unit(11).f.x < 235_000, "swapped end for end");
}

// ------------------------------------------------------------------ techniques

#[test]
fn kidnap_blinks_arms_a_stun_field_and_drops_both_through() {
    // caught: she blinks in front of the target, the field stuns, and both drop through to the exit
    let mut a = hand(5, 7, true);
    a.keep_cmds = true;
    let v = a.unit(12).f.clone();
    a.unit_mut(0).f.x = v.x - 80_000;
    let me = a.unit(0).f.clone();
    // the drop is a teleport too: the exit within TELEPORT_MAX of the catch
    let exit = (v.x - 70_000, v.y - 30_000);
    assert!(dist(me.x, me.y, v.x, v.y) <= KIDNAP_REACH, "the fixture target is in blink reach");
    place(&mut a, T_KIDNAP, vec![with(12, RiftSpec::entrance(R_TEAR, me.x, me.y)), with(12, RiftSpec::at(R_TEAR, exit.0, exit.1))], Some(12), vec![]);
    let mut blinked = None;
    for _ in 0..120 {
        a.step();
        if blinked.is_none() && dist(a.unit(0).f.x, a.unit(0).f.y, me.x, me.y) > 10_000 {
            blinked = Some((a.unit(0).f.x, a.unit(0).f.y));
            assert!(dist(a.unit(0).f.x, a.unit(0).f.y, v.x, v.y) <= KIDNAP_FRONT + 2000, "she lands in front of the target");
            assert!(has_fx(&a, "kidnapwarn_t"), "the armed field is shown");
            assert_eq!(a.core.kidnap_zones().len(), 1, "and published for the game's AI");
        }
    }
    assert!(blinked.is_some(), "she blinked");
    assert!(a.cmds.iter().any(|c| matches!(c, Cmd::Cc { target: 12, kind: Cc::Stun, .. })), "the field stuns the target");
    assert!(a.cmds.iter().any(|c| matches!(c, Cmd::SetPos { target: 12, x, y } if (*x, *y) == exit)), "the target drops onto the exit");
    assert!(dist(a.unit(0).f.x, a.unit(0).f.y, exit.0, exit.1) <= 20_000, "and she lands beside it");
    assert_eq!((a.core.stats.kidnaps, a.core.stats.kidnap_dodges), (1, 0));
    assert!(a.core.kidnap_ready >= KIDNAP_CD as usize, "Kidnap is on its 20 s cooldown");
    assert!(a.core.kidnap_zones().is_empty());

    // dodged: the target walks out before the field closes; no stun, no kidnap, and she stays where she blinked
    let mut b = hand(5, 7, true);
    b.keep_cmds = true;
    let bx = b.unit(12).f.x - 80_000;
    b.unit_mut(0).f.x = bx;
    let me = b.unit(0).f.clone();
    place(&mut b, T_KIDNAP, vec![with(12, RiftSpec::entrance(R_TEAR, me.x, me.y)), with(12, RiftSpec::at(R_TEAR, 260_000, 450_000))], Some(12), vec![]);
    let mut spot = None;
    for _ in 0..120 {
        b.step();
        if spot.is_none() && dist(b.unit(0).f.x, b.unit(0).f.y, me.x, me.y) > 10_000 {
            let s = (b.unit(0).f.x, b.unit(0).f.y);
            spot = Some(s);
            let u = b.unit_mut(12);
            u.f.x = (s.0 + KIDNAP_RADIUS + 20_000).min(900_000);
            u.f.y = s.1;
        }
    }
    let spot = spot.expect("she blinked");
    assert!(!b.cmds.iter().any(|c| matches!(c, Cmd::Cc { target: 12, kind: Cc::Stun, .. })), "a dodged field stuns nobody it missed");
    assert!(dist(b.unit(12).f.x, b.unit(12).f.y, 260_000, 450_000) > 100_000, "no kidnap");
    assert!(dist(b.unit(0).f.x, b.unit(0).f.y, spot.0, spot.1) < 2000, "she stays where she blinked");
    assert_eq!(b.core.stats.kidnap_dodges, 1);
    assert!(has_fx(&b, "kidnapmiss_t"));

    // crowd-control immune targets shrug the field off
    let mut c = hand(5, 7, true);
    c.unit_mut(12).buffs.insert("unstoppable".into(), (100_000, Mods { cc_immune: true, ..Mods::default() }));
    let cx = c.unit(12).f.x - 80_000;
    c.unit_mut(0).f.x = cx;
    let me = c.unit(0).f.clone();
    let w = c.unit(12).f.clone();
    place(&mut c, T_KIDNAP, vec![with(12, RiftSpec::entrance(R_TEAR, me.x, me.y)), with(12, RiftSpec::at(R_TEAR, 260_000, 450_000))], Some(12), vec![]);
    c.run(120);
    assert!(dist(c.unit(12).f.x, c.unit(12).f.y, w.x, w.y) < 1000, "crowd-control immune enemies cannot be kidnapped");
    assert_eq!(c.core.stats.kidnap_dodges, 1);
}

#[test]
fn rifts_are_published_and_her_enemies_steer_around_them() {
    use crate::vesper::{avoid_rifts, rifts_up, steer, zone_names, Zone};
    let mut a = hand(5, 7, true);
    place(&mut a, T_CHAIN, vec![with(12, RiftSpec::node(R_TEAR, 500_000, 480_000))], None, vec![]);
    a.run(40);
    let names = zone_names(&a.core, a.tick);
    assert_eq!(names.len(), 1, "one live rift, one published zone");
    assert!(names[0].0.starts_with("vsr:") && names[0].1 > 0);
    let champ = |id: usize, team: usize, x: i64, y: i64, buffs: Vec<mod_api_stable::BuffV1>| crate::Champ {
        id, team, x, y, buffs, stunned: false, pushed: false, hp: 1000, max_hp: 1000, attack: 50, name: String::new(),
    };
    let all = vec![
        champ(0, 0, 300_000, 480_000, vec![crate::timed(&names[0].0, names[0].1)]),
        champ(10, 1, 400_000, 480_000, vec![]),
        champ(1, 0, 420_000, 480_000, vec![]),
        champ(11, 1, 500_000, 482_000, vec![]),
    ];
    let up = rifts_up(&all, a.tick);
    assert_eq!(up.len(), 1);
    assert_eq!(up[0].1, 0, "owned by Vesper's team");
    let r = up[0].0;
    assert!(!r.field && dist(r.x, r.y, 500_000, 480_000) <= 4000 && r.r >= RIFTS[R_TEAR].radius);
    let p = avoid_rifts(&all, &all[1], (500_000, 480_000), a.tick).expect("an enemy's move into the rift is redirected");
    assert!(dist(p.0, p.1, r.x, r.y) > r.r, "to just outside it");
    assert!(avoid_rifts(&all, &all[2], (500_000, 480_000), a.tick).is_none(), "her allies walk through her rifts");
    assert!(avoid_rifts(&all, &all[3], (520_000, 480_000), a.tick).is_none(), "a rift that opened under an enemy is fought in, not fled");
    assert!(crate::batch2::danger_zones(&all, 1).iter().any(|z| dist(z.0, z.1, r.x, r.y) <= 4000), "the other mod AIs see it too");
    // an armed Kidnap field: walk straight out of it, that is the dodge
    let field = Zone { x: 600_000, y: 480_000, r: KIDNAP_RADIUS, until: 1_000, field: true };
    let out = steer(&[field], (610_000, 480_000), (600_000, 480_000)).expect("out of the field");
    assert!(out.0 > 600_000 + KIDNAP_RADIUS);
}

#[test]
fn gates_move_allies_never_enemies() {
    let mut a = hand(5, 7, true);
    let ally = a.unit(1).f.clone();
    a.unit_mut(10).f.x = ally.x + 3000;
    a.unit_mut(10).f.y = ally.y;
    let exit = (ally.x + 60_000, ally.y - 50_000);
    place(&mut a, T_GATE, vec![with(12, RiftSpec::entrance(R_TEAR, ally.x, ally.y)), with(12, RiftSpec::at(R_TEAR, exit.0, exit.1))], Some(1), vec![]);
    a.run(60);
    assert!(dist(a.unit(1).f.x, a.unit(1).f.y, exit.0, exit.1) < 3000);
    assert!(dist(a.unit(10).f.x, a.unit(10).f.y, exit.0, exit.1) > 60_000);
    assert_eq!(a.core.stats.gates, 1);
}

#[test]
fn no_teleport_goes_farther_than_teleport_max() {
    // a Gate exit 400000 away carries the ally TELEPORT_MAX toward it, no farther
    let mut a = hand(5, 7, true);
    let ally = a.unit(1).f.clone();
    place(&mut a, T_GATE, vec![with(12, RiftSpec::entrance(R_TEAR, ally.x, ally.y)), with(12, RiftSpec::at(R_TEAR, ally.x + 400_000, ally.y))], Some(1), vec![]);
    a.run(60);
    let moved = dist(a.unit(1).f.x, a.unit(1).f.y, ally.x, ally.y);
    assert!(moved > TELEPORT_MAX - 3000 && moved <= TELEPORT_MAX, "moved {moved}");
    assert_eq!(land(0, 0, 500_000, 0, &|_, _| false), (TELEPORT_MAX, 0));
}

#[test]
fn step_and_phase_share_a_cooldown() {
    let mut a = hand(5, 7, false);
    place(&mut a, T_STEP, vec![with(12, RiftSpec::entrance(R_TEAR, 200_000, 480_000)), with(12, RiftSpec::at(R_TEAR, 250_000, 420_000))], None, vec![]);
    a.run(20);
    assert!(dist(a.unit(0).f.x, a.unit(0).f.y, 250_000, 420_000) < 3000);
    let at = (a.unit(0).f.x, a.unit(0).f.y);
    place(&mut a, T_PHASE, vec![with(12, RiftSpec::entrance(R_TEAR, at.0, at.1)), with(12, RiftSpec::at(R_TEAR, 300_000, 300_000))], None, vec![]);
    a.run(80);
    assert!(dist(a.unit(0).f.x, a.unit(0).f.y, at.0, at.1) < 3000, "Phase Out waits for the shared self-move cooldown");
    assert!(!a.core.phased);
}

#[test]
fn implode_consumes_and_domino_travels() {
    let mut a = hand(5, 7, true);
    for id in [10usize, 11] {
        let u = a.unit_mut(id);
        u.f.x = 500_000;
        u.f.y = 480_000 + id as i64 * 1000;
    }
    place(&mut a, T_CHAIN, vec![with(12, RiftSpec::node(R_TEAR, 500_000, 480_000)), with(12, RiftSpec::node(R_TEAR, 520_000, 500_000))], None, vec![]);
    a.run(60);
    let uids: Vec<u32> = a.core.rifts.iter().map(|r| r.uid).collect();
    a.keep_cmds = true;
    place(&mut a, T_DOMINO, vec![], None, uids.clone());
    assert_eq!(a.core.open_count(), 1, "the first rift caves in at once");
    a.run(14);
    assert_eq!(a.core.open_count(), 1);
    a.run(2);
    assert_eq!(a.core.open_count(), 0, "the next one 15 ticks later");
    assert_eq!(a.core.stats.implodes, 2);
    assert!(a.core.stats.implode_hits >= 2);
}

#[test]
fn rift_shapes_behave() {
    let mut a = hand(5, 7, true);
    let e = a.unit(11).f.clone();
    place(&mut a, T_SEEDTRAP, vec![with(12, RiftSpec::at(R_SEED, e.x, e.y))], Some(11), vec![]);
    a.run(100);
    assert_eq!(a.core.stats.trap_catches, 0, "hidden seeds do nothing");
    assert_eq!(a.core.stats.breaks, 0, "and cannot be broken");
    a.run(80);
    assert_eq!(a.core.stats.trap_catches, 1);
    let mut b = hand(5, 7, false);
    place(&mut b, T_SWEEP, vec![with(12, RiftSpec::line(R_WANDER, 300_000, 300_000, 300_000, 600_000))], None, vec![]);
    b.run(40);
    assert!(b.core.rifts[0].y > 300_000 + WANDER_SPEED * 10);
    let mut c = hand(5, 7, true);
    place(&mut c, T_ESCORT, vec![with(12, RiftSpec { host: Some(1), chain: true, ..RiftSpec::at(R_ANCHOR, 0, 0) })], Some(1), vec![]);
    c.run(10);
    c.unit_mut(1).f.x += 50_000;
    c.run(2);
    assert_eq!(c.core.rifts[0].x, c.unit(1).f.x);
    c.run(ORB_CD as usize);
    let e = c.unit(12).f.clone();
    place(&mut c, T_CHAIN, vec![with(12, RiftSpec::at(R_MAW, e.x, e.y))], None, vec![]);
    c.run(200);
    assert!(c.core.stats.implodes >= 1, "a Maw caves in when it closes");
}

#[test]
fn hook_breaks_when_torn_off() {
    let mut a = hand(5, 7, true);
    place(&mut a, T_HOOK, vec![with(12, RiftSpec { host: Some(12), ..RiftSpec::at(R_ANCHOR, 0, 0) })], Some(12), vec![]);
    let e = a.unit(12).f.clone();
    a.unit_mut(13).f.x = e.x;
    a.unit_mut(13).f.y = e.y;
    a.run(80);
    assert_eq!(a.core.stats.breaks, 1);
    assert_eq!(a.core.stats.kidnaps, 0);
}

#[test]
fn overload_punch_and_stitch() {
    let mut a = hand(3, 7, false);
    place(&mut a, T_CHAIN, vec![with(10, RiftSpec::node(R_TEAR, 300_000, 480_000)), with(4, RiftSpec::node(R_TEAR, 320_000, 480_000))], None, vec![]);
    a.run(30);
    a.core.stress = 70;
    place(&mut a, T_OPUNCH, vec![], None, vec![]);
    assert_eq!(a.core.opunch, STRESS_MAX_OVERLOAD);
    assert_eq!(a.core.stress, 70 - STRESS_MAX_OVERLOAD);
    let (u0, u1) = (a.core.rifts[0].uid, a.core.rifts[1].uid);
    place(&mut a, T_STITCH, vec![], None, vec![u0, u1]);
    a.run(1);
    let r = a.core.rift(u0).unwrap();
    assert_eq!((r.dim, r.dim2), (10, Some(4)));
    assert!(a.core.rift(u1).is_none());
}

#[test]
fn landings_stay_out_of_walls() {
    let mut a = hand(5, 7, true);
    a.wall = |x, _| (590_000..=630_000).contains(&x);
    let ally = a.unit(1).f.clone();
    place(&mut a, T_GATE, vec![with(12, RiftSpec::entrance(R_TEAR, ally.x, ally.y)), with(12, RiftSpec::at(R_TEAR, 610_000, 300_000))], Some(1), vec![]);
    a.run(60);
    assert!(!(a.wall)(a.unit(1).f.x, a.unit(1).f.y));
}

#[test]
fn pose_looks_and_rank_tags() {
    use crate::vesper::{pose_look, style_names};
    assert_eq!((0..7).map(|r| pose_look(r, None)).collect::<Vec<_>>(), vec![0, 1, 2, 3, 4, 5, 6]);
    assert_eq!([pose_look(7, Some(1)), pose_look(7, Some(2)), pose_look(7, Some(3)), pose_look(7, Some(7)), pose_look(7, None)], [8, 10, 9, 7, 7]);
    let c = Core::new(1, 0, 7, Some(1));
    let names = style_names(7, Some(1), &c, 0, false);
    assert!(names.contains(&"vs_outfit_r8".to_string()) && names.contains(&"vs_top1".to_string()) && names.contains(&"vs_back_top1".to_string()), "{names:?}");
    let names = style_names(6, None, &c, 0, false);
    assert!(names.contains(&"vs_back_wound".to_string()) && names.contains(&"vs_rank6".to_string()) && !names.iter().any(|n| n.starts_with("vs_top")), "{names:?}");
    let names = style_names(3, None, &c, 0, false);
    assert!(names.contains(&"vs_rank3".to_string()) && !names.iter().any(|n| n.starts_with("vs_back") || n.starts_with("vs_top")), "{names:?}");
    // facing left: the mirrored body and back scar, the emblem and HUD unchanged
    let names = style_names(7, Some(1), &c, 0, true);
    assert!(names.contains(&"vs_outfit_r8_l".to_string()) && names.contains(&"vs_back_top1_l".to_string()) && names.contains(&"vs_top1".to_string()), "{names:?}");
    assert!(names.iter().filter(|n| n.ends_with("_l")).count() == 2, "{names:?}");
}

#[test]
fn pose_overlays_follow_their_priority() {
    use crate::vesper::pose_of;
    let mut c = Core::new(1, 0, 3, None);
    assert_eq!(pose_of(&c, 100), "outfit");
    c.block_until = 118;
    assert_eq!(pose_of(&c, 100), "block");
    c.jab_until = 124;
    assert_eq!(pose_of(&c, 100), "jab");
    c.punching = true;
    assert_eq!(pose_of(&c, 100), "punch");
    c.swipe_until = 124;
    assert_eq!(pose_of(&c, 100), "swipe");
    c.shatter_until = 130;
    assert_eq!(pose_of(&c, 100), "shatter");
    assert_eq!(pose_of(&c, 140), "punch", "one-shot poses end on their tick");
    // the moves set their own pose timers
    let mut a = teamfight(3, 7);
    for _ in 0..1800 {
        a.step();
    }
    assert!(a.core.swipe_until > 0 && a.core.shatter_until > 0, "a fight shows swipes and Shatters");
}

#[test]
fn deterministic_across_clones() {
    let mut a = teamfight(77, 5);
    a.run(300);
    let mut b_core = a.core.clone();
    let fighters = a.fighters();
    let w = |_: i64, _: i64| false;
    let x = a.core.step(&tick_input(&fighters, a.tick, &w));
    let y = b_core.step(&tick_input(&fighters, a.tick, &w));
    assert_eq!(x, y);
}

#[test]
fn profiling_observer_preserves_all_commands_and_randomness() {
    for rank in 0..8 {
        let mut arena=teamfight(77,rank);arena.run(300);
        let mut observed=arena.core.clone();
        let fighters=arena.fighters();let wall=|_:i64,_:i64|false;
        let mut phases=vec![];
        for tick in 300..660 {
            let input=tick_input(&fighters,tick,&wall);
            let expected=arena.core.step(&input);
            let actual=observed.step_observed(&input,&mut|name,start|phases.push((name,start)));
            assert_eq!(expected,actual,"rank {rank}, tick {tick}");
            // The pre-existing planner stopwatch is diagnostic wall time, not simulation state.
            arena.core.stats.pick_nanos=0;observed.stats.pick_nanos=0;
            assert_eq!(format!("{:?}",arena.core),format!("{:?}",observed));
        }
        assert!(phases.iter().any(|p|*p==("render_schedule",true)));
    }
}

#[test]
fn hand_placed_damage_is_identical_at_every_rank() {
    let mut totals = vec![];
    for rank in 0..8 {
        let mut a = hand(9, rank, false);
        a.core.armed = Some(12);
        place(&mut a, T_CHAIN, vec![with(12, RiftSpec::node(R_TEAR, 300_000, 480_000)), with(12, RiftSpec::node(R_TEAR, 470_000, 480_000))], None, vec![]);
        a.run(280);
        totals.push(a.core.stats.damage);
    }
    assert!(totals.iter().all(|d| *d == totals[0] && *d > 0), "{totals:?}");
}

// ------------------------------------------------------------------ fixtures per rank

#[derive(Debug, Default, Clone)]
pub struct Report {
    pub damage: i64,
    /// Damage from her rifts, punches, swipes and moves: everything but the plain jab.
    pub rift_damage: i64,
    pub eff: i64,
    pub punches: u32,
    pub swipes: (u32, u32),
    pub shatters: (u32, u32),
    pub left_breaks: u32,
    pub right_breaks: u32,
    pub awakens: (u32, u32),
    pub twists: u32,
    pub fused: u32,
    pub breaks: u32,
    pub kidnaps: (u32, u32),
    pub gates: (u32, u32),
    pub traps: (u32, u32),
    pub risky: u32,
    pub wins: u32,
    pub length: usize,
    pub uses: [u32; 28],
    pub moves: [u32; 6],
    pub habits: BTreeMap<&'static str, u32>,
    /// View effects started per second of fight (the game's cost).
    pub eps: usize,
    pub peak: usize,
    pub dodges: u32,
    pub deaths: u32,
    pub effects: usize,
    pub ticks: usize,
    /// Distinct dimensions opened per fight (averaged), and the planner's mean wall time per pick.
    pub dims: usize,
    pub picks: u64,
    pub pick_nanos: u64,
}
pub fn fixture(rank: usize, seeds: &[u64]) -> Report {
    let mut r = Report::default();
    for &seed in seeds {
        let mut a = teamfight(seed, rank);
        // the busiest second: effects started in any 60-tick window
        let mut window = std::collections::VecDeque::new();
        for _ in 0..3600 {
            let before = a.core.stats.effects;
            a.step();
            window.push_back(a.core.stats.effects - before);
            if window.len() > 60 {
                window.pop_front();
            }
            r.peak = r.peak.max(window.iter().sum::<u32>() as usize);
            if a.wiped().is_some() {
                break;
            }
        }
        let s = &a.core.stats;
        r.damage += s.damage;
        r.rift_damage += s.damage - s.jab_damage;
        r.eff += if s.landed > 0 { s.eff_sum / s.landed as i64 } else { 0 };
        r.punches += s.punches;
        r.swipes.0 += s.swipe_hits;
        r.swipes.1 += s.swipes;
        r.shatters.0 += s.useful_shatters;
        r.shatters.1 += s.shatters;
        r.left_breaks += s.left_breaks;
        r.right_breaks += s.right_breaks;
        r.awakens.0 += s.good_awakens;
        r.awakens.1 += s.awakens;
        r.twists += s.twists;
        r.fused += s.fused;
        r.breaks += s.breaks;
        r.kidnaps.0 += s.useful_kidnaps;
        r.kidnaps.1 += s.kidnaps;
        r.dodges += s.kidnap_dodges;
        r.deaths += s.self_deaths;
        r.gates.0 += s.engaged_gates;
        r.gates.1 += s.gates;
        r.traps.0 += s.trap_catches;
        r.traps.1 += s.traps;
        r.risky += s.risky_moves;
        for (i, u) in s.tech_uses.iter().enumerate() {
            r.uses[i] += u;
        }
        for (i, u) in s.move_uses.iter().enumerate() {
            r.moves[i] += u;
        }
        for (h, n) in &s.habits {
            *r.habits.entry(h).or_default() += n;
        }
        if a.wiped() == Some(1) {
            r.wins += 1;
        }
        r.length += a.tick;
        r.effects += s.effects as usize;
        r.ticks += a.tick;
        r.dims += s.dims.len();
        r.picks += s.picks as u64;
        r.pick_nanos += s.pick_nanos;
    }
    r.eps = r.effects * 60 / r.ticks.max(1);
    r.dims /= seeds.len().max(1);
    r.eff /= seeds.len().max(1) as i64;
    r.length /= seeds.len().max(1);
    r
}

#[test]
fn rank_fixtures_report() {
    // 192 seeds: at 200 dimensions the Worldbreaker's margin over the win bar is a few fights in a hundred, so 48 or
    // 96 seeds pass or fail on luck; 192 measure the expectation. VESPER_SEEDS changes the sample for tuning; the win
    // margin scales with it.
    let n: u64 = std::env::var("VESPER_SEEDS").ok().and_then(|v| v.parse().ok()).unwrap_or(192);
    let seeds: Vec<u64> = (1..=n).collect();
    let mut lines = vec![];
    let reports: Vec<Report> = (0..8).map(|rank| fixture(rank, &seeds)).collect();
    for (rank, r) in reports.iter().enumerate() {
        lines.push(format!(
            "{rank} {:<12} damage {:>6} punches {:>3} eff {:>4}bp swipes {}/{} shatters {}/{} breaks L{} R{} awaken {}/{} twists {} fused {} rifts_broken {} kidnaps {}/{} gates {}/{} traps {}/{} wins {}/{} fight {:.1}s techniques {} top {}% (#{}) fx/s {} peak {} dodges {} deaths {} moves {:?} dims {} pick {}us rift {}",
            RANKS[rank], r.damage, r.punches, r.eff, r.swipes.0, r.swipes.1, r.shatters.0, r.shatters.1, r.left_breaks, r.right_breaks, r.awakens.0, r.awakens.1, r.twists, r.fused, r.breaks, r.kidnaps.0, r.kidnaps.1, r.gates.0, r.gates.1, r.traps.0, r.traps.1, r.wins, seeds.len(), r.length as f64 / 60.0, r.uses.iter().filter(|u| **u > 0).count(), top_share(r), r.uses.iter().enumerate().max_by_key(|(_, u)| **u).map(|(i, _)| i).unwrap_or(0), r.eps, r.peak, r.dodges, r.deaths, r.moves, r.dims, r.pick_nanos / r.picks.max(1) / 1000, r.rift_damage
        ));
    }
    println!("{}", lines.join("\n"));
    if std::env::var("VESPER_REPORT").is_ok() {
        std::fs::write(concat!(env!("CARGO_MANIFEST_DIR"), "/target/vesper_fixture_report.txt"), lines.join("\n") + "\n").unwrap();
    }
    let (spark, wb) = (&reports[0], &reports[7]);
    // her rifts, punches, swipes and moves: the plain jab (most of a Spark's damage) is not part of the kit
    assert!(wb.rift_damage >= spark.rift_damage * 2, "Worldbreaker's rifts deal at least 2x a Spark's ({} vs {})", wb.rift_damage, spark.rift_damage);
    assert!(wb.left_breaks * 5 <= spark.left_breaks && spark.left_breaks >= 12, "a Spark cracks her gauntlet; a Worldbreaker keeps it");
    assert!(wb.right_breaks <= spark.right_breaks, "and never corrupts the Hexcore more");
    assert!(wb.breaks * 2 <= spark.breaks, "rifts stay off enemy paths");
    assert!(wb.shatters.0 * 100 >= wb.shatters.1 * 90 && spark.shatters.0 * 100 <= spark.shatters.1 * 50, "Shatters land on piles at the top, on nothing at the bottom");
    // a Spark's swipes go anywhere (about one hit in two); a Worldbreaker's cut whole lines: two or more a swipe, 3x her rate
    assert!(wb.swipes.0 >= wb.swipes.1 * 2 && wb.swipes.0 * spark.swipes.1 >= 3 * spark.swipes.0 * wb.swipes.1, "swipes cut through lines of enemies at the top");
    // every pull that lands an enemy (Hook, Sinkhole, Seed, Swap, Fling and the Kidnap blink) counts here; with the
    // enemies stepping around her rifts, half of a Worldbreaker's land in her team against a tenth of a Spark's
    assert!(wb.kidnaps.0 * 100 >= wb.kidnaps.1 * 50 && spark.kidnaps.0 * 100 <= spark.kidnaps.1 * 30, "pulls land enemies in the team at the top");
    assert!(wb.wins as u64 >= spark.wins as u64 + 8 * n / 12, "the same team wins with a Worldbreaker and loses with a Spark");
    assert!(spark.uses.iter().filter(|u| **u > 0).count() >= 20, "a Spark still uses (almost) everything");
    assert!(spark.moves.iter().filter(|m| **m > 0).count() >= 4, "and the gauntlet moves");
    // without noise the top still reads the target and the spot, so she opens several dimensions a fight, not one
    assert!(wb.dims >= 8, "a Worldbreaker opens at least 8 different dimensions a fight ({})", wb.dims);
    for (rank, r) in reports.iter().enumerate() {
        // generous for slow machines: a pick takes 1-5 us (17.5 us before the fight was read once per pick)
        let us = r.pick_nanos / r.picks.max(1) / 1000;
        assert!(us <= 250, "{}: the planner takes {us} us per dimension pick", RANKS[rank]);
    }
    for (rank, r) in reports.iter().enumerate() {
        // the top ranks rotate; the middle ranks lean on their stress dump (Overload Punch) but never past a quarter
        // the effect budget: never more than one bucket plus its refill in any second
        assert!(r.peak <= (60 / FX_REFILL + FX_TOKENS) as usize && r.eps <= 30, "{}: {} effects in the busiest second, {}/s", RANKS[rank], r.peak, r.eps);
        let cap = if rank >= 5 { 20 } else { 25 };
        assert!(top_share(r) <= cap, "{}: no technique takes more than {cap}% of her plays ({}%)", RANKS[rank], top_share(r));
    }
}

/// The most-used technique's share of all technique uses, in percent.
fn top_share(r: &Report) -> usize {
    let total: usize = r.uses.iter().map(|u| *u as usize).sum();
    r.uses.iter().map(|u| *u as usize).max().unwrap_or(0) * 100 / total.max(1)
}

#[test]
fn habits_start_at_their_rank() {
    let seeds: Vec<u64> = (1..=6).collect();
    let reports: Vec<Report> = (0..8).map(|rank| fixture(rank, &seeds)).collect();
    let gated: [(&str, usize); 8] = [
        ("solved the link count", GATE_GEOMETRY),
        ("Gate synced with the tank", GATE_TEAM),
        ("Kidnapped the carry into the team", GATE_PRIORITY),
        ("escaped a dive", GATE_SURVIVAL),
        ("spine refreshed", GATE_SPINE),
        ("dimension picked for the target", GATE_PRIORITY),
        ("dropped aim for a diver", GATE_TEAM),
        ("Awakening timed with the engage", GATE_TEAM),
    ];
    for (habit, from) in gated {
        for (rank, r) in reports.iter().enumerate() {
            if rank < from {
                assert_eq!(r.habits.get(habit).copied().unwrap_or(0), 0, "{habit} at rank {rank}");
            }
        }
    }
    let wb = &reports[7];
    assert!(wb.habits.values().sum::<u32>() > reports[0].habits.values().sum::<u32>());
}

#[test]
#[ignore]
fn usage_dump() {
    let seeds: Vec<u64> = (1..=12).collect();
    for rank in [0usize, 3, 5, 6, 7] {
        let r = fixture(rank, &seeds);
        let uses: Vec<String> = r.uses.iter().enumerate().filter(|(_, u)| **u > 0).map(|(i, u)| format!("{}:{u}", TECHS[i].id)).collect();
        println!("{rank} {} moves {:?}", uses.join(" "), r.moves);
        println!("   habits {:?}", r.habits);
    }
}

#[test]
#[ignore]
fn gauntlet_trace() {
    for seed in 1..=4u64 {
        for rank in [0usize, 7] {
            let mut a = teamfight(seed, rank);
            a.keep_cmds = true;
            for _ in 0..3600 {
                a.cmds.clear();
                a.step();
                for c in &a.cmds {
                    if let Cmd::Log(s) = c {
                        if !s.contains("rank") || s.contains("shatter") || s.contains("overload") || s.contains("awaken") || s.contains("overclock") {
                            println!("seed {seed} rank {rank} {s}");
                        }
                    }
                }
                if a.wiped().is_some() {
                    break;
                }
            }
            println!("seed {seed} rank {rank} end {:?}", (a.core.stats.left_breaks, a.core.stats.right_breaks, a.core.stats.shatters, a.core.stats.damage));
        }
    }
}

// ------------------------------------------------------------------ stable-API adapter smoke test
mod adapter {
    use crate::vesper::Vesper;
    use mod_api_stable::{AttackTypeV1, BuffV1, CcV1, EntityHandleV1, SimCtxV1, SimVtableV1, StablePassive, StableSim, StatV1};
    use std::cell::{Cell, RefCell};
    use std::collections::BTreeMap;
    use std::ffi::c_void;
    use std::mem::size_of;
    #[derive(Default)]
    struct Host {
        tick: Cell<usize>,
        pos: RefCell<BTreeMap<usize, (u64, u64)>>,
        hp: RefCell<BTreeMap<usize, usize>>,
        buffs: RefCell<BTreeMap<usize, Vec<BuffV1>>>,
        effects: RefCell<Vec<String>>,
        hits: RefCell<Vec<(usize, usize)>>,
        cc: RefCell<Vec<(usize, u32)>>,
    }
    fn h<'a>(p: *const c_void) -> &'a Host {
        unsafe { &*(p as *const Host) }
    }
    unsafe extern "C" fn tick(p: *const c_void) -> usize { h(p).tick.get() }
    unsafe extern "C" fn seed(_: *const c_void) -> u64 { 4242 }
    unsafe extern "C" fn valid(_: *const c_void, _: EntityHandleV1) -> bool { true }
    unsafe extern "C" fn alive(p: *const c_void, e: EntityHandleV1) -> bool { h(p).hp.borrow().get(&e.id().unwrap()).copied().unwrap_or(1) > 0 }
    unsafe extern "C" fn hp(p: *const c_void, e: EntityHandleV1, c: *mut usize, m: *mut usize) -> bool {
        *c = h(p).hp.borrow().get(&e.id().unwrap()).copied().unwrap_or(1000);
        *m = 1000;
        true
    }
    unsafe extern "C" fn pos(p: *const c_void, e: EntityHandleV1, x: *mut u64, y: *mut u64) -> bool {
        let (px, py) = h(p).pos.borrow().get(&e.id().unwrap()).copied().unwrap_or((0, 0));
        *x = px;
        *y = py;
        true
    }
    unsafe extern "C" fn stat(_: *const c_void, _: EntityHandleV1, out: *mut StatV1) -> bool {
        *out = StatV1 { attack: 120, ..StatV1::default() };
        true
    }
    unsafe extern "C" fn team(_: *const c_void, e: EntityHandleV1) -> usize { usize::from(e.id().unwrap() >= 5) }
    unsafe extern "C" fn count(_: *const c_void) -> usize { 2 }
    unsafe extern "C" fn at(_: *const c_void, i: usize) -> usize { [0, 5][i] }
    unsafe extern "C" fn visible(_: *const c_void, _: usize, _: usize) -> bool { true }
    unsafe extern "C" fn damage(p: *mut c_void, _: usize, id: usize, ad: usize, ap: usize, kind: u32) {
        assert_eq!(kind, AttackTypeV1::Skill.code());
        let host = h(p);
        host.hits.borrow_mut().push((id, ad + ap));
        let mut hp = host.hp.borrow_mut();
        let v = hp.entry(id).or_insert(1000);
        *v = v.saturating_sub(ad + ap);
    }
    unsafe extern "C" fn buff(p: *mut c_void, id: usize, b: *const BuffV1) {
        let mut all = h(p).buffs.borrow_mut();
        let v = all.entry(id).or_default();
        v.retain(|o| o.name() != (*b).name());
        v.push(*b);
    }
    unsafe extern "C" fn buff_count(p: *const c_void, e: EntityHandleV1) -> usize { h(p).buffs.borrow().get(&e.id().unwrap()).map_or(0, Vec::len) }
    unsafe extern "C" fn buff_at(p: *const c_void, e: EntityHandleV1, i: usize, out: *mut BuffV1) -> bool {
        match h(p).buffs.borrow().get(&e.id().unwrap()).and_then(|v| v.get(i)) {
            Some(b) => {
                *out = *b;
                true
            }
            None => false,
        }
    }
    unsafe extern "C" fn remove_buff(p: *mut c_void, e: EntityHandleV1, name: *const u8, len: usize) -> usize {
        let name = std::str::from_utf8(std::slice::from_raw_parts(name, len)).unwrap();
        let mut all = h(p).buffs.borrow_mut();
        let v = all.entry(e.id().unwrap()).or_default();
        let n = v.len();
        v.retain(|b| b.name() != name);
        n - v.len()
    }
    unsafe extern "C" fn effect(p: *mut c_void, name: *const u8, len: usize, _: usize, _: *const mod_api_stable::InputTargetV1, _: u64, _: u64, _: u64) -> bool {
        h(p).effects.borrow_mut().push(std::str::from_utf8(std::slice::from_raw_parts(name, len)).unwrap().to_owned());
        true
    }
    unsafe extern "C" fn cc(p: *mut c_void, id: usize, c: *const CcV1) {
        h(p).cc.borrow_mut().push((id, (*c).kind));
    }
    unsafe extern "C" fn interval(_: *const c_void, _: EntityHandleV1) -> usize { 60 }
    #[test]
    fn adapter_applies_core_commands() {
        let host = Host::default();
        host.pos.borrow_mut().insert(0, (300_000, 480_000));
        host.pos.borrow_mut().insert(5, (420_000, 480_000));
        let mut v: SimVtableV1 = unsafe { std::mem::zeroed() };
        v.size = size_of::<SimVtableV1>();
        v.tick = Some(tick);
        v.seed = Some(seed);
        v.entity_is_valid = Some(valid);
        v.entity_is_alive = Some(alive);
        v.entity_hp = Some(hp);
        v.entity_pos = Some(pos);
        v.entity_stat = Some(stat);
        v.entity_team = Some(team);
        v.champion_count = Some(count);
        v.champion_id_at = Some(at);
        v.is_visible = Some(visible);
        v.deal_damage = Some(damage);
        v.add_buff = Some(buff);
        v.entity_buff_count = Some(buff_count);
        v.entity_buff_at = Some(buff_at);
        v.entity_remove_buff = Some(remove_buff);
        v.play_view_effect = Some(effect);
        v.apply_cc = Some(cc);
        v.entity_attack_interval = Some(interval);
        let mut raw = SimCtxV1 { size: size_of::<SimCtxV1>(), sim: &v, frame: std::ptr::null(), state: (&host as *const Host).cast_mut().cast() };
        let mut sim = unsafe { StableSim::from_raw(&mut raw).unwrap() };
        let mut vesper = Vesper::default();
        for t in 0..900 {
            host.tick.set(t);
            vesper.on_update(&mut sim, 0, 0, 0);
        }
        let buffs: Vec<String> = host.buffs.borrow().get(&0).unwrap().iter().map(|b| b.name().to_string()).collect();
        let poses = ["outfit", "jab", "block", "punch", "swipe", "shatter"];
        assert!(buffs.iter().any(|b| poses.iter().any(|p| b == &format!("vs_{p}_r0"))), "{buffs:?}");
        assert!(buffs.iter().any(|b| b == "vs_rank0") && !buffs.iter().any(|b| b.starts_with("vs_back") || b.starts_with("vs_top")), "a Spark wears her emblem and no scar behind her: {buffs:?}");
        assert!(buffs.iter().any(|b| b.starts_with("vs_orbs")));
        assert!(buffs.iter().any(|b| b.starts_with("vs_hex")));
        assert!(buffs.iter().any(|b| b.starts_with("vs_core")));
        assert!(buffs.iter().any(|b| b.starts_with("vs_stress")));
        assert!(host.effects.borrow().iter().all(|e| e.starts_with("tfm2_custom_vesper_")));
        assert!(!host.effects.borrow().is_empty(), "rifts and punches are drawn");
        assert!(!host.hits.borrow().is_empty(), "the core's damage reaches the target");
    }
}

/// Rift Lab parity: the JS port (editor/vesper-combat.js) must replay these fights decision for decision.
/// `VESPER_SIM_VECTORS=write` regenerates `vesper_sim_vectors.txt`.
#[test]
fn sim_vectors() {
    let mut rows = vec![];
    for seed in [1u64, 2, 3] {
        for rank in [0usize, 3, 5, 7] {
            let mut a = teamfight(seed, rank);
            a.keep_cmds = true;
            for _ in 0..900 {
                a.cmds.clear();
                a.step();
                for c in &a.cmds {
                    if let Cmd::Log(s) = c {
                        rows.push(format!("{seed}|{rank}|{s}"));
                    }
                }
                if a.wiped().is_some() {
                    break;
                }
            }
            let s = &a.core.stats;
            let c = &a.core;
            let hp: Vec<String> = a.units.iter().map(|u| format!("{}:{}:{}:{}", u.f.id, u.f.hp, u.f.x, u.f.y)).collect();
            rows.push(format!(
                "{seed}|{rank}|end tick {} stress {} hexcore {} corruption {} orbs {}/{} damage {} punches {} landed {} eff {} swipes {}/{} shatters {}/{} breaks L{} R{} awakens {}/{} twists {} fused {} rifts_broken {} kidnaps {}/{} dodges {} gates {}/{} traps {}/{} implodes {} effects {} uses {:?} moves {:?} units {}",
                a.tick, c.stress, c.completion, c.corruption, c.pure_ready(a.tick), c.hex_ready(a.tick), s.damage, s.punches, s.landed, s.eff_sum, s.swipe_hits, s.swipes, s.useful_shatters, s.shatters, s.left_breaks, s.right_breaks, s.good_awakens, s.awakens, s.twists, s.fused, s.breaks, s.useful_kidnaps, s.kidnaps, s.kidnap_dodges, s.engaged_gates, s.gates, s.trap_catches, s.traps, s.implodes, s.effects, s.tech_uses, s.move_uses, hp.join(",")
            ));
        }
    }
    // hand-placed scenarios: every dimension's punch and swipe, and the gauntlet moves
    let digest = |a: &Arena| {
        let units: Vec<String> = a.units.iter().map(|u| format!("{}:{}:{}:{}", u.f.id, u.f.hp, u.f.x, u.f.y)).collect();
        let s = &a.core.stats;
        format!("tick {} stress {} hexcore {} corruption {} orbs {}/{} damage {} landed {} twists {} swipes {}/{} units {}", a.tick, a.core.stress, a.core.completion, a.core.corruption, a.core.pure_ready(a.tick), a.core.hex_ready(a.tick), s.damage, s.landed, s.twists, s.swipe_hits, s.swipes, units.join(","))
    };
    for dim in 0..N_DIMS {
        let mut a = teamfight(11, 7);
        a.core.hand_placed = true;
        for (i, id) in [10usize, 11, 12, 1].iter().enumerate() {
            let u = a.unit_mut(*id);
            u.f.x = 260_000 + i as i64 * 7000;
            u.f.y = 480_000 + (i as i64 % 2) * 5000;
        }
        a.unit_mut(0).f.x = 220_000;
        a.unit_mut(0).f.y = 480_000;
        a.core.armed = Some(dim);
        a.run(150);
        rows.push(format!("hand|punch|{dim}|{}", digest(&a)));
        let mut b = teamfight(11, 7);
        b.core.hand_placed = true;
        b.frozen = true;
        for (i, id) in [10usize, 11, 12, 1].iter().enumerate() {
            let u = b.unit_mut(*id);
            u.f.x = 240_000 + i as i64 * 9000;
            u.f.y = 480_000 + (i as i64 % 2) * 4000;
        }
        b.unit_mut(0).f.x = 220_000;
        b.unit_mut(0).f.y = 480_000;
        b.input_cmds(Action { point: Some((300_000, 480_000)), dim: Some(dim), subject: Some(10), ..Action::new(M_SWIPE, vec![]) });
        b.run(150);
        rows.push(format!("hand|swipe|{dim}|{}", digest(&b)));
    }
    let mut g = teamfight(5, 7);
    g.core.hand_placed = true;
    g.frozen = true;
    for id in [10usize, 11] {
        let u = g.unit_mut(id);
        u.f.x = 500_000 + id as i64 * 1000;
        u.f.y = 480_000;
    }
    g.input_cmds(Action::new(T_CHAIN, vec![RiftSpec { dim: Some(11), ..RiftSpec::node(R_TEAR, 500_000, 480_000) }]));
    for _ in 0..3 {
        g.input_cmds(Action::new(M_OVERLOAD, vec![]));
    }
    g.run(60);
    g.input_cmds(Action::new(M_SHATTER, vec![]));
    rows.push(format!("hand|shatter|{}", digest(&g)));
    let mut h = teamfight(5, 7);
    h.core.hand_placed = true;
    h.frozen = true;
    h.unit_mut(10).f.x = 225_000;
    h.unit_mut(10).f.y = 480_000;
    h.core.completion = 100;
    h.input_cmds(Action::new(M_OVERLOAD, vec![]));
    h.input_cmds(Action::new(M_OVERLOAD, vec![]));
    h.input_cmds(Action::new(M_AWAKEN, vec![]));
    h.run(30);
    for _ in 0..5 {
        h.input_cmds(Action::new(M_OVERCLOCK, vec![]));
    }
    h.run(30);
    rows.push(format!("hand|overclock|{}", digest(&h)));
    let text = rows.join("\n") + "\n";
    if std::env::var("VESPER_SIM_VECTORS").as_deref() == Ok("write") {
        std::fs::write(concat!(env!("CARGO_MANIFEST_DIR"), "/src/vesper_sim_vectors.txt"), text).unwrap();
    } else {
        assert_eq!(text, include_str!("vesper_sim_vectors.txt").replace("\r\n", "\n"));
    }
}
