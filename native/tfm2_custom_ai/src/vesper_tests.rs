//! Vesper: rule tests on the pure core, plus duel and 5v5 fixtures per rank in a small deterministic arena.
use crate::vesper_core::*;
use crate::vesper_data::*;
use crate::vesper_math::*;
use std::collections::BTreeMap;

const NO_WALL: fn(i64, i64) -> bool = |_, _| false;

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
            f: Fighter { id, team, x, y, hp: max_hp, max_hp, attack, cc_immune: false, controlled: false },
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
                let (nx, ny) = along(u.f.x, u.f.y, target.f.x - u.f.x, target.f.y - u.f.y, speed.min(d - u.range + 1));
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
        Unit::new(0, 0, 200_000, 480_000, 1100, 90, ATTACK_RANGE),
        Unit::new(5, 1, 520_000, 480_000, 2400, 60, 30_000),
    ];
    Arena::new(seed, rank, units)
}
/// Vesper plus a tank, a bruiser, a mage and a healer-ish support against a mirrored line-up.
pub fn teamfight(seed: u64, rank: usize) -> Arena {
    // odd seeds start close (the engage window is open from the first think), even seeds start far apart
    let (home, away) = if seed % 2 == 1 { (300_000, 600_000) } else { (260_000, 700_000) };
    let mut units = vec![Unit::new(0, 0, home - 80_000, 480_000, 1100, 90, ATTACK_RANGE)];
    let roles = [(2600, 55, 25_000), (1800, 80, 28_000), (1200, 95, 60_000), (1300, 60, 55_000)];
    for (i, (hp, atk, range)) in roles.iter().enumerate() {
        let y = 400_000 + i as i64 * 50_000;
        units.push(Unit::new(1 + i, 0, home - (*range / 3), y, *hp, *atk, *range));
    }
    let foes = [(2600, 55, 25_000), (1800, 80, 28_000), (1200, 95, 60_000), (1300, 60, 55_000), (1100, 90, ATTACK_RANGE)];
    for (i, (hp, atk, range)) in foes.iter().enumerate() {
        let y = 380_000 + i as i64 * 50_000;
        units.push(Unit::new(10 + i, 1, away + (*range / 3), y, *hp, *atk, *range));
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

// ------------------------------------------------------------------ rules

#[test]
fn five_charges_cap_and_expiry() {
    let mut a = duel(7, 0);
    a.core.hand_placed = true;
    a.frozen = true;
    for i in 0..5 {
        a.input_cmds(Action { tech: T_CHAIN, rifts: vec![RiftSpec::node(R_TEAR, 250_000 + i * 30_000, 480_000)], subject: None, uses: vec![], score: 0, habit: "" });
    }
    assert_eq!(a.core.charges.available(), 0);
    assert_eq!(a.core.open_count(), 5);
    a.input_cmds(Action { tech: T_CHAIN, rifts: vec![RiftSpec::node(R_TEAR, 400_000, 480_000)], subject: None, uses: vec![], score: 0, habit: "" });
    assert_eq!(a.core.open_count(), 5, "a sixth tear needs a sixth charge");
    a.run(400);
    assert_eq!(a.core.open_count(), 0, "tears close after 5 s");
    assert!(a.core.charges.available() == 5, "charges recover in 5 s");
}

#[test]
fn far_tears_open_later() {
    let mut a = duel(7, 0);
    a.core.hand_placed = true;
    a.frozen = true;
    a.input_cmds(Action { tech: T_CHAIN, rifts: vec![RiftSpec::node(R_TEAR, 210_000, 480_000), RiftSpec::node(R_TEAR, 900_000, 480_000)], subject: None, uses: vec![], score: 0, habit: "" });
    let near = a.core.rifts[0].opens;
    let far = a.core.rifts[1].opens;
    assert_eq!(near, open_delay(10_000, 100));
    assert_eq!(far, open_delay(700_000, 100));
    assert!(far > near + 50);
}

#[test]
fn chain_fire_reroutes_attacks_and_loses_per_link() {
    let mut a = duel(3, 7);
    a.core.hand_placed = true;
    a.frozen = true;
    // the target stands 320000 away: far beyond her own range
    a.input_cmds(Action { tech: T_CHAIN, rifts: vec![RiftSpec::node(R_TEAR, 300_000, 480_000), RiftSpec::node(R_TEAR, 460_000, 480_000)], subject: None, uses: vec![], score: 0, habit: "" });
    a.keep_cmds = true;
    a.run(200);
    assert!(a.core.aiming, "she plants her feet while the chain is live");
    let hits: Vec<i64> = a.cmds.iter().filter_map(|c| if let Cmd::Damage { target: 5, amount, kind: Dmg::Phys } = c { Some(*amount) } else { None }).collect();
    assert!(!hits.is_empty(), "packets reach a target out of her own range");
    let expected = chain_bp(&[100_000, 160_000]);
    assert!(a.core.stats.packets > 0);
    assert_eq!(a.core.stats.eff_sum / a.core.stats.packets as i64, expected);
    assert!(a.cmds.iter().any(|c| matches!(c, Cmd::Cc { target: 0, kind: Cc::Disarm, .. })), "the engine attack is held while aiming");
}

#[test]
fn breaking_cuts_the_chain_and_adds_strain() {
    let mut a = duel(3, 7);
    a.core.hand_placed = true;
    a.frozen = true;
    a.input_cmds(Action { tech: T_CHAIN, rifts: vec![RiftSpec::node(R_TEAR, 520_000, 480_000)], subject: None, uses: vec![], score: 0, habit: "" });
    let strain = a.core.strain;
    a.run(40 + BREAK_TICKS as usize);
    assert_eq!(a.core.stats.breaks, 1);
    assert_eq!(a.core.open_count(), 0);
    let _ = strain;
    assert!(a.core.strain > 0 && a.core.strain >= STRAIN_BREAK - 2, "a broken rift costs strain");
}

#[test]
fn strain_breach_hits_both_teams_and_collapses_everything() {
    let mut a = teamfight(5, 0);
    a.core.hand_placed = true;
    a.frozen = true;
    // put an ally and an enemy right next to Vesper
    a.unit_mut(1).f.x = 185_000;
    a.unit_mut(10).f.x = 190_000;
    a.unit_mut(10).f.y = 480_000;
    a.unit_mut(1).f.y = 480_000;
    a.input_cmds(Action { tech: T_CHAIN, rifts: vec![RiftSpec::node(R_TEAR, 300_000, 480_000)], subject: None, uses: vec![], score: 0, habit: "" });
    a.core.strain = 120;
    a.keep_cmds = true;
    a.run(2);
    assert_eq!(a.core.stats.breaches, 1);
    assert_eq!(a.core.strain, BREACH_RESET);
    assert_eq!(a.core.open_count(), 0);
    assert!(a.cmds.iter().any(|c| matches!(c, Cmd::LoseHp { target: 0, .. })));
    assert!(a.unit(0).f.hp >= 1);
}

#[test]
fn every_dimension_fires_inside_its_limits() {
    for dim in 0..100 {
        let mut a = teamfight(11, 7);
        a.core.hand_placed = true;
        a.frozen = true;
        let (cx, cy) = (480_000, 480_000);
        for id in [1usize, 10, 11] {
            let u = a.unit_mut(id);
            u.f.x = cx + 10_000 + (id as i64 % 3) * 2000;
            u.f.y = cy;
        }
        a.input_cmds(Action { tech: T_CHAIN, rifts: vec![RiftSpec::node(R_TEAR, cx, cy)], subject: None, uses: vec![], score: 0, habit: "" });
        a.core.rifts[0].dim = dim;
        a.keep_cmds = true;
        a.run(240);
        // shared limits per target
        let mut moves: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
        for c in &a.cmds {
            if let Cmd::SetPos { target, .. } = c {
                if *target >= 10 {
                    moves.entry(*target).or_default().push(0);
                }
            }
        }
        for (id, m) in moves {
            assert!(m.len() <= 6, "{} moved {id} {} times", DIMS[dim].name, m.len());
        }
        let control: i64 = a.cmds.iter().filter_map(|c| match c {
            Cmd::Cc { target: 10, ticks, kind } if *kind != Cc::Disarm || DIMS[dim].prim != P_DISARM => Some(*ticks as i64),
            Cmd::Banish { target: 10, ticks } => Some(*ticks as i64),
            _ => None,
        }).sum();
        assert!(control <= CONTROL_CAP * 2, "{} controlled for {control} ticks in 4 s", DIMS[dim].name);
        let damage: i64 = a.cmds.iter().filter_map(|c| if let Cmd::Damage { target: 10, amount, .. } = c { Some(*amount) } else { None }).sum();
        assert!(damage <= a.unit(10).f.max_hp * DAMAGE_PCT / 100 * 2 + 1, "{} dealt {damage}", DIMS[dim].name);
        let side = PRIM_SIDE[DIMS[dim].prim as usize];
        let touched = a.cmds.iter().any(|c| match c {
            Cmd::Damage { .. } | Cmd::Cc { target: 10 | 11 | 1, .. } | Cmd::Banish { .. } | Cmd::SetPos { .. } | Cmd::Heal { .. } | Cmd::Shield { .. } | Cmd::Invisible { .. } | Cmd::ClearCc { .. } | Cmd::ClearShield { .. } => true,
            Cmd::Buff { name, .. } => name.starts_with("vsd_"),
            _ => false,
        });
        let silent_ok = matches!(side, b'P') || matches!(DIMS[dim].prim, P_VENT | P_UNSTABLE | P_KALEIDO | P_SIPHON | P_WALL | P_TETHER | P_FOLD | P_SELFINVIS | P_CLEANSE | P_DICE | P_STATICBURST);
        assert!(touched || silent_ok, "{} ({}) did nothing", DIMS[dim].name, PRIM_IDS[DIMS[dim].prim as usize]);
    }
}

#[test]
fn imbues_and_family_bonuses_ride_the_packet() {
    let mut a = duel(3, 7);
    a.core.hand_placed = true;
    a.frozen = true;
    a.input_cmds(Action { tech: T_CHAIN, rifts: vec![RiftSpec::node(R_TEAR, 300_000, 480_000), RiftSpec::node(R_TEAR, 470_000, 480_000)], subject: None, uses: vec![], score: 0, habit: "" });
    // Static Realm then Anchor: Ruin + Control -> Execute Window; the packet also roots and amplifies
    a.core.rifts[0].dim = 10;
    a.core.rifts[1].dim = 4;
    a.keep_cmds = true;
    a.run(160);
    assert!(a.cmds.iter().any(|c| matches!(c, Cmd::Buff { target: 5, name, .. } if name == "vsd_D10")), "Static Realm imbued");
    assert!(a.cmds.iter().any(|c| matches!(c, Cmd::Cc { target: 5, kind: Cc::Root, .. })), "Anchor imbued");
    assert!(a.cmds.iter().any(|c| matches!(c, Cmd::Fx { tag, .. } if tag == &format!("twist{}", TWISTS[twist_index(1, 0)].kind))));
}

fn place(a: &mut Arena, tech: usize, rifts: Vec<RiftSpec>, subject: Option<usize>, uses: Vec<u32>) {
    a.input_cmds(Action { tech, rifts, subject, uses, score: 0, habit: "" });
}

#[test]
fn kidnap_moves_the_victim_and_respects_immunity_and_cooldown() {
    let mut a = teamfight(5, 7);
    a.core.hand_placed = true;
    a.frozen = true;
    let v = a.unit(12).f.clone();
    place(&mut a, T_KIDNAP, vec![RiftSpec::entrance(R_TEAR, v.x, v.y), RiftSpec::at(R_TEAR, 260_000, 450_000)], Some(12), vec![]);
    a.run(120);
    let after = a.unit(12).f.clone();
    assert!(dist(after.x, after.y, 260_000, 450_000) < 5000, "kidnapped to the exit");
    assert_eq!(a.core.stats.kidnaps, 1);
    // again at once: the 10 s per-target cooldown holds
    let v = a.unit(12).f.clone();
    place(&mut a, T_KIDNAP, vec![RiftSpec::entrance(R_TEAR, v.x, v.y), RiftSpec::at(R_TEAR, 600_000, 450_000)], Some(12), vec![]);
    a.run(60);
    assert!(dist(a.unit(12).f.x, a.unit(12).f.y, 600_000, 450_000) > 5000);
    // crowd-control immune enemies cannot be kidnapped
    let mut b = teamfight(5, 7);
    b.core.hand_placed = true;
    b.frozen = true;
    b.unit_mut(13).buffs.insert("unstoppable".into(), (100_000, Mods { cc_immune: true, ..Mods::default() }));
    let w = b.unit(13).f.clone();
    place(&mut b, T_KIDNAP, vec![RiftSpec::entrance(R_TEAR, w.x, w.y), RiftSpec::at(R_TEAR, 260_000, 450_000)], Some(13), vec![]);
    b.run(120);
    assert!(dist(b.unit(13).f.x, b.unit(13).f.y, w.x, w.y) < 1000);
}

#[test]
fn gates_move_allies_never_enemies() {
    let mut a = teamfight(5, 7);
    a.core.hand_placed = true;
    a.frozen = true;
    let ally = a.unit(1).f.clone();
    a.unit_mut(10).f.x = ally.x + 3000;
    a.unit_mut(10).f.y = ally.y;
    place(&mut a, T_GATE, vec![RiftSpec::entrance(R_TEAR, ally.x, ally.y), RiftSpec::at(R_TEAR, 600_000, 300_000)], Some(1), vec![]);
    a.run(60);
    assert!(dist(a.unit(1).f.x, a.unit(1).f.y, 600_000, 300_000) < 3000);
    assert!(dist(a.unit(10).f.x, a.unit(10).f.y, 600_000, 300_000) > 100_000);
    assert_eq!(a.core.stats.gates, 1);
}

#[test]
fn step_and_phase_share_a_cooldown() {
    let mut a = duel(5, 7);
    a.core.hand_placed = true;
    a.frozen = true;
    place(&mut a, T_STEP, vec![RiftSpec::entrance(R_TEAR, 200_000, 480_000), RiftSpec::at(R_TEAR, 260_000, 400_000)], None, vec![]);
    a.run(20);
    assert!(dist(a.unit(0).f.x, a.unit(0).f.y, 260_000, 400_000) < 3000);
    let at = (a.unit(0).f.x, a.unit(0).f.y);
    place(&mut a, T_PHASE, vec![RiftSpec::entrance(R_TEAR, at.0, at.1), RiftSpec::at(R_TEAR, 300_000, 300_000)], None, vec![]);
    a.run(80);
    assert!(dist(a.unit(0).f.x, a.unit(0).f.y, at.0, at.1) < 3000, "Phase Out waits for the shared self-move cooldown");
    assert!(!a.core.phased);
}

#[test]
fn implode_consumes_and_domino_travels() {
    let mut a = teamfight(5, 7);
    a.core.hand_placed = true;
    a.frozen = true;
    for id in [10usize, 11] {
        let u = a.unit_mut(id);
        u.f.x = 500_000;
        u.f.y = 480_000 + id as i64 * 1000;
    }
    place(&mut a, T_CHAIN, vec![RiftSpec::node(R_TEAR, 500_000, 480_000), RiftSpec::node(R_TEAR, 520_000, 500_000)], None, vec![]);
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
    // Seed: hidden for 120 ticks, then a trap
    let mut a = teamfight(5, 7);
    a.core.hand_placed = true;
    a.frozen = true;
    let e = a.unit(11).f.clone();
    place(&mut a, T_SEEDTRAP, vec![RiftSpec::at(R_SEED, e.x, e.y)], Some(11), vec![]);
    a.run(100);
    assert_eq!(a.core.stats.trap_catches, 0, "hidden seeds do nothing");
    assert_eq!(a.core.stats.breaks, 0, "and cannot be broken");
    a.run(80);
    assert_eq!(a.core.stats.trap_catches, 1);
    // Wandering Rift moves along its path
    let mut b = duel(5, 7);
    b.core.hand_placed = true;
    b.frozen = true;
    place(&mut b, T_SWEEP, vec![RiftSpec::line(R_WANDER, 300_000, 300_000, 300_000, 600_000)], None, vec![]);
    b.run(40);
    let y = b.core.rifts[0].y;
    assert!(y > 300_000 + WANDER_SPEED * 10);
    // Anchor follows its host; Maw caves in by itself
    let mut c = teamfight(5, 7);
    c.core.hand_placed = true;
    c.frozen = true;
    place(&mut c, T_ESCORT, vec![RiftSpec { host: Some(1), chain: true, ..RiftSpec::at(R_ANCHOR, 0, 0) }], Some(1), vec![]);
    c.run(10);
    c.unit_mut(1).f.x += 50_000;
    c.run(2);
    assert_eq!(c.core.rifts[0].x, c.unit(1).f.x);
    let e = c.unit(12).f.clone();
    place(&mut c, T_CHAIN, vec![RiftSpec::at(R_MAW, e.x, e.y)], None, vec![]);
    c.run(200);
    assert!(c.core.stats.implodes >= 1, "a Maw caves in when it closes");
}

#[test]
fn hook_breaks_when_torn_off() {
    let mut a = teamfight(5, 7);
    a.core.hand_placed = true;
    a.frozen = true;
    place(&mut a, T_HOOK, vec![RiftSpec { host: Some(12), ..RiftSpec::at(R_ANCHOR, 0, 0) }], Some(12), vec![]);
    // another enemy stands on the anchor
    let e = a.unit(12).f.clone();
    a.unit_mut(13).f.x = e.x;
    a.unit_mut(13).f.y = e.y;
    a.run(80);
    assert_eq!(a.core.stats.breaks, 1);
    assert_eq!(a.core.stats.kidnaps, 0);
}

#[test]
fn overload_borrow_and_stitch() {
    let mut a = duel(3, 7);
    a.core.hand_placed = true;
    a.frozen = true;
    place(&mut a, T_CHAIN, vec![RiftSpec::node(R_TEAR, 300_000, 480_000), RiftSpec::node(R_TEAR, 320_000, 480_000)], None, vec![]);
    a.run(30);
    a.core.strain = 70;
    place(&mut a, T_OVERLOAD, vec![], None, vec![]);
    assert_eq!(a.core.overload, OVERLOAD_MAX);
    assert_eq!(a.core.strain, 70 - OVERLOAD_MAX);
    let exit = a.core.rifts[1].uid;
    place(&mut a, T_BORROW, vec![RiftSpec::at(R_TEAR, 900_000, 900_000)], None, vec![exit]);
    let far_dim = a.atlas_dim(900_000, 900_000);
    a.run(1);
    assert_eq!(a.core.rift(exit).unwrap().borrowed.map(|b| b.0), Some(far_dim));
    let (u0, u1) = (a.core.rifts[0].uid, a.core.rifts[1].uid);
    let (d0, d1) = (a.core.rifts[0].dim, a.core.rifts[1].dim);
    place(&mut a, T_STITCH, vec![], None, vec![u0, u1]);
    a.run(1);
    let r = a.core.rift(u0).unwrap();
    assert_eq!((r.dim, r.dim2), (d0, Some(d1)));
    assert!(a.core.rift(u1).is_none());
}

#[test]
fn landings_stay_out_of_walls() {
    let mut a = teamfight(5, 7);
    a.core.hand_placed = true;
    a.frozen = true;
    a.wall = |x, _| (590_000..=630_000).contains(&x);
    let ally = a.unit(1).f.clone();
    place(&mut a, T_GATE, vec![RiftSpec::entrance(R_TEAR, ally.x, ally.y), RiftSpec::at(R_TEAR, 610_000, 300_000)], Some(1), vec![]);
    a.run(60);
    assert!(!(a.wall)(a.unit(1).f.x, a.unit(1).f.y));
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
fn hand_placed_damage_is_identical_at_every_rank() {
    let mut totals = vec![];
    for rank in 0..8 {
        let mut a = duel(9, rank);
        a.core.hand_placed = true;
        a.frozen = true;
        place(&mut a, T_CHAIN, vec![RiftSpec::node(R_TEAR, 300_000, 480_000), RiftSpec::node(R_TEAR, 470_000, 480_000)], None, vec![]);
        a.run(280);
        totals.push(a.core.stats.damage);
    }
    assert!(totals.iter().all(|d| *d == totals[0] && *d > 0), "{totals:?}");
}

impl Arena {
    fn atlas_dim(&self, x: i64, y: i64) -> usize {
        self.core.atlas.dim_at(x, y, self.tick)
    }
}

// ------------------------------------------------------------------ fixtures per rank

#[derive(Debug, Default, Clone)]
pub struct Report {
    pub damage: i64,
    pub eff: i64,
    pub breaches: u32,
    pub breaks: u32,
    pub kidnaps: (u32, u32),
    pub gates: (u32, u32),
    pub traps: (u32, u32),
    pub risky: u32,
    pub wins: u32,
    pub length: usize,
    pub uses: [u32; 28],
    pub habits: BTreeMap<&'static str, u32>,
}
pub fn fixture(rank: usize, seeds: &[u64]) -> Report {
    let mut r = Report::default();
    for &seed in seeds {
        let mut a = teamfight(seed, rank);
        a.run(3600);
        let s = &a.core.stats;
        r.damage += s.damage;
        r.eff += if s.packets > 0 { s.eff_sum / s.packets as i64 } else { 0 };
        r.breaches += s.breaches;
        r.breaks += s.breaks;
        r.kidnaps.0 += s.useful_kidnaps;
        r.kidnaps.1 += s.kidnaps;
        r.gates.0 += s.engaged_gates;
        r.gates.1 += s.gates;
        r.traps.0 += s.trap_catches;
        r.traps.1 += s.traps;
        r.risky += s.risky_moves;
        for (i, u) in s.tech_uses.iter().enumerate() {
            r.uses[i] += u;
        }
        for (h, n) in &s.habits {
            *r.habits.entry(h).or_default() += n;
        }
        if a.wiped() == Some(1) {
            r.wins += 1;
        }
        r.length += a.tick;
    }
    r.eff /= seeds.len().max(1) as i64;
    r.length /= seeds.len().max(1);
    r
}

#[test]
fn rank_fixtures_report() {
    let seeds: Vec<u64> = (1..=12).collect();
    let mut lines = vec![];
    let reports: Vec<Report> = (0..8).map(|rank| fixture(rank, &seeds)).collect();
    for (rank, r) in reports.iter().enumerate() {
        lines.push(format!(
            "{rank} {:<12} damage {:>6} eff {:>4}bp breaches {:>3} breaks {:>3} kidnaps {}/{} gates {}/{} traps {}/{} risky {} wins {}/{} fight {:.1}s techniques used {}",
            RANKS[rank], r.damage, r.eff, r.breaches, r.breaks, r.kidnaps.0, r.kidnaps.1, r.gates.0, r.gates.1, r.traps.0, r.traps.1, r.risky, r.wins, seeds.len(), r.length as f64 / 60.0, r.uses.iter().filter(|u| **u > 0).count()
        ));
    }
    println!("{}", lines.join("\n"));
    if std::env::var("VESPER_REPORT").is_ok() {
        std::fs::write(concat!(env!("CARGO_MANIFEST_DIR"), "/target/vesper_fixture_report.txt"), lines.join("\n") + "\n").unwrap();
    }
    let (spark, wb) = (&reports[0], &reports[7]);
    assert!(wb.damage * 10 >= spark.damage * 14, "Worldbreaker deals at least 1.4x a Spark's damage");
    assert!(wb.breaches * 5 <= spark.breaches, "Worldbreaker avoids Breaches");
    assert!(wb.breaks * 2 <= spark.breaks, "and keeps rifts off enemy paths");
    assert!(wb.kidnaps.0 * 100 >= wb.kidnaps.1 * 60 && spark.kidnaps.0 * 100 <= spark.kidnaps.1 * 20, "Kidnaps land in the team at the top, onto Vesper at the bottom");
    assert!(wb.wins >= spark.wins && wb.length < spark.length, "and the team wins faster");
    assert!(spark.uses.iter().filter(|u| **u > 0).count() >= 20, "a Spark still uses (almost) everything");
}

#[test]
fn habits_start_at_their_rank() {
    let seeds: Vec<u64> = (1..=6).collect();
    let reports: Vec<Report> = (0..8).map(|rank| fixture(rank, &seeds)).collect();
    // each habit is never seen below its rank
    let gated: [(&str, usize); 8] = [
        ("solved the link count", GATE_GEOMETRY),
        ("Gate synced with the tank", GATE_TEAM),
        ("Kidnapped the carry into the team", GATE_PRIORITY),
        ("escaped a dive", GATE_SURVIVAL),
        ("spine refreshed", GATE_SPINE),
        ("rode strain into Overload", GATE_SPINE),
        ("dropped aim for a diver", GATE_TEAM),
        ("borrowed a far dimension", GATE_TEAM),
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
    for rank in [0usize, 3, 5, 7] {
        let r = fixture(rank, &seeds);
        let uses: Vec<String> = r.uses.iter().enumerate().filter(|(_, u)| **u > 0).map(|(i, u)| format!("{}:{u}", TECHS[i].id)).collect();
        println!("{rank} {}", uses.join(" "));
        println!("   habits {:?}", r.habits);
    }
}

#[test]
#[ignore]
fn breach_trace() {
    for seed in 1..=12u64 {
        let mut a = teamfight(seed, 7);
        let mut last = 0;
        for _ in 0..3600 {
            let before = a.core.strain;
            let breaches = a.core.stats.breaches;
            let fighters = a.fighters();
            let _ = fighters;
            a.keep_cmds = true;
            a.cmds.clear();
            a.step();
            if a.core.strain != before && a.core.strain > 60 {
                let logs: Vec<String> = a.cmds.iter().filter_map(|c| if let Cmd::Log(s) = c { Some(s.clone()) } else { None }).collect();
                if a.tick - last < 400 || a.core.stats.breaches > breaches {
                    println!("seed {seed} tick {} strain {before}->{} open {} {:?}", a.tick, a.core.strain, a.core.open_count(), logs);
                }
                last = a.tick;
            }
            if a.core.stats.breaches > breaches {
                let logs: Vec<String> = a.cmds.iter().filter_map(|c| if let Cmd::Log(s) = c { Some(s.clone()) } else { None }).collect();
                println!("---- BREACH at {} from {before}: {:?}", a.tick, logs);
            }
            if a.wiped().is_some() { break; }
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
        assert!(buffs.iter().any(|b| b.starts_with("vs_outfit_r0") || b.starts_with("vs_aim_r0")), "{buffs:?}");
        assert!(buffs.iter().any(|b| b == "vs_rank0"));
        assert!(buffs.iter().any(|b| b.starts_with("vs_charges")));
        assert!(buffs.iter().any(|b| b.starts_with("vs_strain")));
        assert!(host.effects.borrow().iter().all(|e| e.starts_with("tfm2_custom_vesper_")));
        assert!(!host.effects.borrow().is_empty(), "rifts and packets are drawn");
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
            let hp: Vec<String> = a.units.iter().map(|u| format!("{}:{}:{}:{}", u.f.id, u.f.hp, u.f.x, u.f.y)).collect();
            rows.push(format!(
                "{seed}|{rank}|end tick {} strain {} damage {} packets {} eff {} breaches {} breaks {} kidnaps {}/{} gates {}/{} traps {}/{} implodes {} uses {:?} units {}",
                a.tick, a.core.strain, s.damage, s.packets, s.eff_sum, s.breaches, s.breaks, s.useful_kidnaps, s.kidnaps, s.engaged_gates, s.gates, s.trap_catches, s.traps, s.implodes, s.tech_uses, hp.join(",")
            ));
        }
    }
    let text = rows.join("\n") + "\n";
    if std::env::var("VESPER_SIM_VECTORS").as_deref() == Ok("write") {
        std::fs::write(concat!(env!("CARGO_MANIFEST_DIR"), "/src/vesper_sim_vectors.txt"), text).unwrap();
    } else {
        assert_eq!(text, include_str!("vesper_sim_vectors.txt").replace("\r\n", "\n"));
    }
}
