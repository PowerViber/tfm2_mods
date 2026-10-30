//! Round 144: is Raze viable next to her mobile peers? Duels, a lane and an objective, all host-free.
//!
//! Raze is the real shared core (`Core::step`, the same code as the game and the Lab), driven in automatic mode.
//! Everything the engine does for her is modelled here: her gun (the core picks it, the engine fires it), armour,
//! shields, omnivamp, crowd control and pushes on the peer.
//!
//! The peers are a scripted model, not their native brains: Levi, V1, Minato and David need the live engine
//! (cables, buffs, projectiles), so each is built from its shipped data (`mods/*/champion/*.data_champion`: stats,
//! growth, basic attack ratio, cooldown and range, data-driven skill damage) plus its Mod Power line in `lib.rs`.
//! Damage that only lives in native code is estimated and marked `est.` below. Peers walk straight at her to their
//! range (ranged peers then kite to 80% of it), never dodge her grenades, and land PEER_ACCURACY (80%) of their skills
//! with a seeded reaction of 15–60 ticks. The stock ranged champion is a generic ranged base-class model (no Mod Power).
//!
//! Round 145: peers dodge her grenades the way the game's avoid layer does (they walk out of a `rzw` warning zone
//! GRENADE_DODGE_REACTION ticks after the throw), every unit reports its velocity to the core, ranged laners shuffle
//! sideways between trades, and Paint Shells' hit rate on champions is counted.
//!
//! `cargo test --release raze_viability -- --ignored --nocapture` prints the tables; the smoke test keeps it alive.
use super::*;
use crate::valorant_guns::{current_basic_raw, Gun};
use std::collections::{BTreeMap, BTreeSet};

const DUEL_TICKS: u64 = 1800;
const SEEDS: u64 = 24;

#[derive(Clone, Copy)]
struct Burst {
    damage: i64,
    ratio: i64,
    cd: u64,
    range: i64,
    /// Closes the gap to this distance when cast (dashes, blinks, cables).
    gap: Option<i64>,
    /// Level the burst is available from (ultimates at 5).
    from: i64,
    /// The damage above lands this many times, `every` ticks apart (combos, channels, Rampage).
    hits: u64,
    every: u64,
}
const fn burst(damage: i64, ratio: i64, cd: u64, range: i64, gap: Option<i64>, from: i64) -> Burst {
    Burst { damage, ratio, cd, range, gap, from, hits: 1, every: 1 }
}
const fn channel(b: Burst, hits: u64, every: u64) -> Burst {
    Burst { damage: b.damage / hits as i64, ratio: b.ratio / hits as i64, hits, every, ..b }
}

#[derive(Clone, Copy)]
struct Model {
    name: &'static str,
    hp: (i64, i64),
    attack: (i64, i64),
    defence: (i64, i64),
    speed: (i64, i64),
    /// Mod Power: attack, hp, armour and attack speed percent.
    power: [i64; 4],
    basic_ratio: i64,
    basic_cd: u64,
    range: i64,
    melee: bool,
    skills: &'static [Burst],
}

const LEVI: Model = Model {
    name: "Levi",
    hp: (880, 82),
    attack: (110, 26),
    defence: (28, 8),
    speed: (1050, 12),
    power: [30, 10, 10, 10],
    basic_ratio: 100,
    basic_cd: 55,
    range: 22000,
    melee: true,
    skills: &[
        // a cable cut every AIR_DASH_CD (levi.rs: 15 + 25%, scaled by speed, est. ×2)
        burst(30, 50, 60, 200000, Some(12000), 1),
        // Rampage (levi.rs: RAMPAGE_T 480 ticks of cuts at 30 + 45%, est. one pass every 40 ticks at ×2)
        channel(burst(720, 1080, 3600, 110000, Some(12000), 5), 12, 40),
    ],
};
const V1: Model = Model {
    name: "V1",
    hp: (870, 82),
    attack: (94, 20),
    defence: (22, 6),
    speed: (1100, 12),
    power: [45, 10, 10, 10],
    basic_ratio: 100,
    basic_cd: 50,
    range: 30000,
    melee: false,
    skills: &[
        burst(30, 45, 420, 60000, None, 1),
        // slide (no data damage): a gap closer
        burst(0, 0, 600, 200000, Some(20000), 1),
        // ult (native, est.)
        channel(burst(200, 200, 3300, 260000, None, 5), 3, 20),
    ],
};
const MINATO: Model = Model {
    name: "Minato",
    hp: (950, 85),
    attack: (92, 20),
    defence: (25, 7),
    speed: (1120, 14),
    power: [25, 5, 10, 10],
    basic_ratio: 70,
    basic_cd: 50,
    range: 50000,
    melee: false,
    skills: &[
        // kunai teleport strike
        burst(40, 60, 420, 160000, Some(15000), 1),
        // Rasengan (mid charge of its 90–165% ladder)
        burst(60, 120, 840, 25000, None, 1),
        // ult (native, est.)
        channel(burst(200, 200, 3600, 60000, Some(15000), 5), 3, 20),
    ],
};
const DAVID: Model = Model {
    name: "David",
    hp: (950, 90),
    attack: (95, 22),
    defence: (25, 7),
    speed: (1100, 12),
    power: [40, 5, 5, 10],
    basic_ratio: 100,
    basic_cd: 55,
    range: 23000,
    melee: true,
    skills: &[
        // Sandevistan dash
        burst(0, 0, 45, 60000, Some(15000), 1),
        // the skill2 combo: 7 × (8 + 18%) + 50 + 95% + 40 + 70% + 60 + 90%
        channel(burst(210, 380, 480, 90000, Some(15000), 1), 10, 6),
        // ult (native, est.)
        channel(burst(200, 200, 3600, 80000, None, 5), 3, 20),
    ],
};
const STOCK: Model = Model {
    name: "Stock ranged",
    hp: (820, 85),
    attack: (62, 11),
    defence: (20, 5),
    speed: (1000, 8),
    power: [0, 0, 0, 0],
    basic_ratio: 100,
    basic_cd: 60,
    range: 45000,
    melee: false,
    skills: &[burst(60, 80, 360, 60000, None, 1), burst(150, 150, 3600, 70000, None, 5)],
};
/// A generic melee laner (fighter base class, no Mod Power), for the lane table.
const STOCK_MELEE: Model = Model {
    name: "Stock melee",
    hp: (900, 90),
    attack: (66, 12),
    defence: (25, 6),
    speed: (1050, 8),
    power: [0, 0, 0, 0],
    basic_ratio: 100,
    basic_cd: 55,
    range: 22000,
    melee: true,
    skills: &[burst(70, 90, 420, 22000, Some(15000), 1), burst(150, 150, 3600, 30000, None, 5)],
};
const PEERS: [Model; 4] = [LEVI, V1, MINATO, DAVID];

impl Model {
    fn at(&self, level: i64) -> (i64, i64, i64, i64) {
        let l = level - 1;
        let hp = (self.hp.0 + self.hp.1 * l) * (100 + self.power[1]) / 100;
        let attack = (self.attack.0 + self.attack.1 * l) * (100 + self.power[0]) / 100;
        let defence = (self.defence.0 + self.defence.1 * l) * (100 + self.power[2]) / 100;
        (hp, attack, defence, self.speed.0 + self.speed.1 * l)
    }
    fn cd(&self) -> u64 {
        self.basic_cd * 100 / (100 + self.power[3]) as u64
    }
}

fn mitigate(raw: i64, defence: i64) -> i64 {
    raw * 100 / (100 + defence.max(0))
}

/// Raze's stats at a level, from her shipped data and Mod Power [35, 0, 10, 10, 10, 0, 10].
fn raze_unit(level: i64) -> Unit {
    let l = level - 1;
    Unit {
        id: 1,
        team: 0,
        hp: (850 + 90 * l) * 110 / 100,
        max_hp: (850 + 90 * l) * 110 / 100,
        attack: (50 + 8 * l) * 135 / 100,
        defence: (25 + 6 * l) * 110 / 100,
        mr: (20 + 4 * l) * 110 / 100,
        move_speed: 1020 + 8 * l,
        attack_range: 22000,
        visible: true,
        targetable: true,
        ..Default::default()
    }
}

#[derive(Default, Clone)]
struct Tally {
    duels: u32,
    wins: f64,
    kills: u32,
    deaths: u32,
    ticks: u64,
    dealt: i64,
    taken: i64,
    thrown: u64,
    landed: u64,
}

/// The engine side of one Raze: shields, control on others and her gun.
struct Sim {
    core: Core,
    f: Frame,
    layers: Vec<(i64, u64)>,
    gun_ready: u64,
    stunned_until: [u64; 64],
    slowed: [(i64, u64); 64],
    dealt: i64,
    taken: i64,
    next_credit: u64,
    /// Positions at the start of the previous tick, for the velocities the core reads.
    last: BTreeMap<usize, Point>,
    /// Paint Shells thrown while an enemy champion was alive, and those whose main blast hit one.
    seen: BTreeSet<u64>,
    aimed: BTreeSet<u64>,
    thrown: u64,
    landed: BTreeSet<u64>,
}

impl Sim {
    fn new(level: i64, seed: u64, mut units: Vec<Unit>) -> Self {
        let mut me = raze_unit(level);
        me.pos = Point::new(300000, 300000);
        units.insert(0, me);
        Self {
            core: Core { stock: [1, 1, 1], ..Default::default() },
            f: Frame {
                units,
                seed,
                automatic: true,
                level,
                mastery: 3,
                unlocked: [true, true, level >= 5],
                ..Default::default()
            },
            layers: vec![],
            gun_ready: 0,
            stunned_until: [0; 64],
            slowed: [(0, 0); 64],
            dealt: 0,
            taken: 0,
            next_credit: 1,
            last: BTreeMap::new(),
            seen: BTreeSet::new(),
            aimed: BTreeSet::new(),
            thrown: 0,
            landed: BTreeSet::new(),
        }
    }
    fn unit(&mut self, id: usize) -> Option<&mut Unit> {
        self.f.units.iter_mut().find(|u| u.id == id)
    }
    fn controlled(&self, id: usize) -> bool {
        self.f.tick < self.stunned_until[id]
    }
    fn speed(&self, u: &Unit) -> i64 {
        let (p, until) = self.slowed[u.id];
        if self.f.tick < until { u.move_speed * (100 - p) / 100 } else { u.move_speed }
    }
    /// Damage on Raze from others: air protection, then her shield layers, then HP.
    fn hit_raze(&mut self, raw: i64) {
        let mut n = self.core.protected_damage(mitigate(raw, self.f.units[0].defence));
        self.taken += n;
        for l in self.layers.iter_mut() {
            let used = l.0.min(n);
            l.0 -= used;
            n -= used;
        }
        self.f.units[0].hp -= n;
    }
    /// One tick of the core and the engine. Returns the ids of units her commands took down.
    fn step(&mut self) -> Vec<usize> {
        self.f.tick += 1;
        self.layers.retain(|(a, end)| *a > 0 && self.f.tick < *end);
        self.f.shield = self.layers.iter().map(|l| l.0).sum();
        let gun = Gun::from_index(self.core.gun);
        self.f.units[0].attack_range = gun.reach();
        for u in self.f.units.iter_mut() {
            u.velocity = self.last.get(&u.id).map_or(Point::default(), |p| u.pos.sub(*p).scale(TPS, 1));
            self.last.insert(u.id, u.pos);
        }
        let out = self.core.step(&self.f);
        // a throw counts as aimed at a champion when it lands near one and not on a wave (wave clears are farming)
        for g in &self.core.grenades {
            let near = |kind: u8, r: i64| {
                self.f.units.iter().any(|u| u.team != 0 && u.kind == kind && u.alive() && distance(u.pos, g.aim) <= r)
            };
            let at_champion = near(0, GRENADE_RADIUS + BODY_RADIUS)
                || near(0, GRENADE_RADIUS + 20000) && !near(1, GRENADE_RADIUS);
            if self.seen.insert(g.id) && at_champion {
                self.thrown += 1;
                self.aimed.insert(g.id);
            }
        }
        self.f.credits.clear();
        let mut down = vec![];
        for cmd in &out {
            match cmd {
                Cmd::MoveOwner { to } => self.f.units[0].pos = *to,
                Cmd::Spawn { id, hp, .. } => self.f.utility.push(UtilityStatus { id: *id, hp: *hp }),
                Cmd::Remove { id } => self.f.utility.retain(|s| s.id != *id),
                Cmd::Shield { amount, ticks } => self.layers.push((*amount, self.f.tick + ticks)),
                Cmd::Damage { target, raw, self_hit, vamp, fixed, .. } => {
                    if *self_hit || *target == 1 {
                        let n = mitigate(*raw, self.f.units[0].defence);
                        self.f.units[0].hp -= n;
                        continue;
                    }
                    let t = *target;
                    if let Cmd::Damage { id, source, .. } = cmd {
                        if source == "grenade"
                            && self.aimed.contains(id)
                            && self.f.units.iter().any(|u| u.id == t && u.kind == 0)
                        {
                            self.landed.insert(*id);
                        }
                    }
                    let Some(u) = self.unit(t) else { continue };
                    if !u.alive() {
                        continue;
                    }
                    let n = if *fixed { *raw } else { mitigate(*raw, u.defence) }.min(u.hp);
                    u.hp -= n;
                    let (kind, dead) = (u.kind, !u.alive());
                    if kind == 0 {
                        self.dealt += n;
                    }
                    let me = &mut self.f.units[0];
                    me.hp = (me.hp + n * vamp / 100).min(me.max_hp);
                    if dead {
                        down.push(t);
                    }
                }
                Cmd::Stun { target, ticks } | Cmd::KnockUp { target, ticks } if *target < 64 => {
                    self.stunned_until[*target] = self.stunned_until[*target].max(self.f.tick + ticks)
                }
                Cmd::Slow { target, percent, ticks } if *target < 64 => {
                    self.slowed[*target] = (*percent, self.f.tick + ticks)
                }
                Cmd::Push { target, to } | Cmd::Knockback { target, to, .. } => {
                    if let Some(u) = self.unit(*target) {
                        u.pos = *to;
                    }
                }
                _ => {}
            }
        }
        for id in &down {
            if self.f.units.iter().any(|u| u.id == *id && u.kind == 0) {
                self.f.credits.push(CreditEvent { id: self.next_credit, kind: 0, victim: *id });
                self.next_credit += 1;
            }
        }
        down
    }
    /// Her gun on the nearest damageable enemy (the engine's basic attack), and her walk when grounded.
    fn gun_and_walk(&mut self, hold: i64) -> Vec<usize> {
        let mut down = vec![];
        let me = self.f.units[0].clone();
        let Some(t) = self
            .f
            .units
            .iter()
            .filter(|u| u.team != me.team && u.damageable())
            .min_by_key(|u| (u.kind != 0, distance(me.pos, u.pos)))
            .cloned()
        else {
            return down;
        };
        let d = distance(me.pos, t.pos);
        let busy = self.core.airborne() || self.core.cast.is_some() || self.f.stunned;
        let gun = Gun::from_index(self.core.gun);
        if !busy && d <= gun.reach() + BODY_RADIUS && self.f.tick >= self.gun_ready {
            let (atk, aspd) = gun.modifiers();
            let n = me.attack * (100 + atk as i64) / 100;
            let raw = current_basic_raw(gun, n, d);
            let u = self.unit(t.id).unwrap();
            let dealt = mitigate(raw, u.defence).min(u.hp);
            u.hp -= dealt;
            if !u.alive() {
                down.push(t.id);
            }
            if t.kind == 0 {
                self.dealt += dealt;
            }
            self.gun_ready = self.f.tick + 60 * 100 / (100 + aspd as i64 + 10).max(20) as u64;
        }
        if !busy && d > hold {
            let speed = self.speed(&me);
            self.f.units[0].pos = me.pos.toward(t.pos, speed.min(d - hold));
        }
        down
    }
}

/// A scripted peer: walks to its range, uses gap closers, bursts on cooldown and attacks.
struct Peer {
    m: Model,
    id: usize,
    ready: Vec<u64>,
    basic_ready: u64,
    /// Channelled hits still to land: (tick, raw).
    pending: Vec<(u64, i64)>,
    /// Seeded per duel: how long it takes to react to her, and which of its skills land.
    seed: u64,
    reaction: u64,
    /// Ranged laners shuffle sideways between trades (lane only).
    shuffle: bool,
}
/// Peers land PEER_ACCURACY percent of their skills (Raze's own aim error comes from her profile in the core).
const PEER_ACCURACY: u64 = 80;
impl Peer {
    fn new(m: Model, seed: u64) -> Self {
        let reaction = 15 + seed * 13 % 45;
        Self { m, id: 2, ready: vec![0; m.skills.len()], basic_ready: 0, pending: vec![], seed, reaction, shuffle: false }
    }
    fn lands(&self, tick: u64, skill: usize) -> bool {
        let h = (self.seed ^ tick.wrapping_mul(0x9e37_79b9_7f4a_7c15) ^ (skill as u64) << 32)
            .wrapping_mul(0xbf58_476d_1ce4_e5b9);
        (h >> 33) % 100 < PEER_ACCURACY
    }
    fn unit(&self, level: i64, pos: Point) -> Unit {
        let (hp, attack, defence, speed) = self.m.at(level);
        Unit {
            id: self.id,
            team: 1,
            kind: 0,
            pos,
            hp,
            max_hp: hp,
            attack,
            defence,
            mr: defence,
            move_speed: speed,
            attack_range: self.m.range,
            melee: self.m.melee,
            visible: true,
            targetable: true,
            ..Default::default()
        }
    }
    fn act(&mut self, s: &mut Sim, level: i64) {
        let Some(me) = s.f.units.iter().find(|u| u.id == self.id).cloned() else { return };
        if !me.alive() || s.controlled(self.id) {
            return;
        }
        let raze = s.f.units[0].clone();
        let mut d = distance(me.pos, raze.pos);
        let now = s.f.tick;
        for (_, raw) in self.pending.iter().filter(|(t, _)| *t <= now) {
            s.hit_raze(*raw);
        }
        self.pending.retain(|(t, _)| *t > now);
        // The avoid layer: once a Paint Shells warning has been up GRENADE_DODGE_REACTION ticks, walk out of it.
        if let Some(aim) = s
            .core
            .grenades
            .iter()
            .find(|g| {
                !g.main_done
                    && now >= g.born + GRENADE_DODGE_REACTION as u64
                    && distance(me.pos, g.aim) <= GRENADE_RADIUS + BODY_RADIUS
            })
            .map(|g| g.aim)
        {
            let away = if me.pos == aim { me.pos.add(Point::new(1000, 0)) } else { me.pos.add(me.pos.sub(aim)) };
            let speed = s.speed(&me);
            s.unit(self.id).unwrap().pos = me.pos.toward(away.bounded(), speed);
            return;
        }
        for (i, b) in self.m.skills.iter().enumerate() {
            if level < b.from || s.f.tick < self.ready[i] || d > b.range || s.f.tick < self.reaction {
                continue;
            }
            if let Some(gap) = b.gap {
                if d > gap {
                    let to = raze.pos.toward(me.pos, gap);
                    s.unit(self.id).unwrap().pos = to;
                    d = gap;
                }
            }
            if b.damage + b.ratio > 0 && self.lands(s.f.tick, i) {
                for n in 0..b.hits {
                    self.pending.push((s.f.tick + n * b.every, b.damage + me.attack * b.ratio / 100));
                }
            }
            self.ready[i] = s.f.tick + b.cd;
        }
        if d <= self.m.range + BODY_RADIUS {
            if s.f.tick >= self.basic_ready.max(self.reaction) {
                s.hit_raze(me.attack * self.m.basic_ratio / 100);
                self.basic_ready = s.f.tick + self.m.cd();
            }
        } else {
            let speed = s.speed(&me);
            s.unit(self.id).unwrap().pos = me.pos.toward(raze.pos, speed.min(d - self.m.range + 2000));
            return;
        }
        // ranged peers kite: they back off to keep her at 80% of their range
        if !self.m.melee && d < self.m.range * 8 / 10 {
            let speed = s.speed(&me);
            let away = me.pos.add(me.pos.sub(raze.pos)).bounded();
            s.unit(self.id).unwrap().pos = me.pos.toward(away, speed);
        } else if self.shuffle {
            // between basic attacks a laner steps sideways, flipping every 30-60 ticks
            let flip = (now / (30 + self.seed % 31)) % 2 == 0;
            let axis = raze.pos.sub(me.pos);
            let side = if flip { Point::new(-axis.y, axis.x) } else { Point::new(axis.y, -axis.x) };
            let to = me.pos.add(side.capped(1000).scale(30, 1)).bounded();
            let speed = s.speed(&me) / 2;
            s.unit(self.id).unwrap().pos = me.pos.toward(to, speed);
        }
    }
}

/// One duel from a seeded start. Returns Raze's score (1 win, 0 loss, HP share on a draw), the ticks, her
/// champion damage dealt and taken.
/// Also returns (Paint Shells thrown at a live champion, those that hit one).
fn duel_hits(m: Model, level: i64, seed: u64) -> (f64, u64, i64, i64, (u64, u64)) {
    let mut peer = Peer::new(m, seed);
    let angle = (seed * 37 % 360) as f64 * std::f64::consts::PI / 180.0;
    let gap = 40000 + (seed * 7919 % 50) as i64 * 1000;
    let pos = Point::new(
        300000 + (gap as f64 * angle.cos()) as i64,
        300000 + (gap as f64 * angle.sin()) as i64,
    );
    let mut s = Sim::new(level, seed, vec![peer.unit(level, pos)]);
    s.f.profile.values = [50 + (seed % 5) as i64 * 5; 13];
    for _ in 0..DUEL_TICKS {
        s.step();
        s.gun_and_walk(9000);
        peer.act(&mut s, level);
        let (me, them) = (&s.f.units[0], &s.f.units[1]);
        if !me.alive() || !them.alive() {
            let score = if me.alive() { 1.0 } else if them.alive() { 0.0 } else { 0.5 };
            return (score, s.f.tick, s.dealt, s.taken, (s.thrown, s.landed.len() as u64));
        }
    }
    let (me, them) = (&s.f.units[0], &s.f.units[1]);
    let a = me.hp as f64 / me.max_hp as f64;
    let b = them.hp as f64 / them.max_hp as f64;
    (if a > b { 1.0 } else if a < b { 0.0 } else { 0.5 }, s.f.tick, s.dealt, s.taken, (s.thrown, s.landed.len() as u64))
}

fn series(m: Model, level: i64, seeds: u64) -> Tally {
    let mut t = Tally::default();
    for seed in 1..=seeds {
        let (score, ticks, dealt, taken, (thrown, landed)) = duel_hits(m, level, seed);
        t.thrown += thrown;
        t.landed += landed;
        t.duels += 1;
        t.wins += score;
        t.kills += (score == 1.0) as u32;
        t.deaths += (score == 0.0) as u32;
        t.ticks += ticks;
        t.dealt += dealt;
        t.taken += taken;
    }
    t
}

/// A six-minion wave (three melee, three ranged) at her lane position, scaled by level.
fn wave(level: i64, first: usize, x: i64) -> Vec<Unit> {
    (0..6)
        .map(|i| {
            let melee = i < 3;
            let hp = if melee { 450 } else { 300 } + 22 * (level - 1);
            Unit {
                id: first + i,
                team: 1,
                kind: 1,
                farmable: true,
                pos: Point::new(x + if melee { 0 } else { 6000 }, 296000 + (i as i64 % 3) * 4000),
                hp,
                max_hp: hp,
                attack: 20,
                defence: if melee { 10 } else { 5 },
                move_speed: 500,
                visible: true,
                targetable: true,
                ..Default::default()
            }
        })
        .collect()
}

/// Ticks for Raze to clear one wave alone.
fn raze_wave_clear(level: i64, seed: u64) -> u64 {
    let mut s = Sim::new(level, seed, wave(level, 10, 330000));
    s.f.farm_window = true;
    for _ in 0..3600 {
        s.step();
        s.gun_and_walk(8000);
        if s.f.units.iter().all(|u| u.kind != 1 || !u.alive()) {
            return s.f.tick;
        }
    }
    3600
}

/// The same wave for a scripted model: basic attacks on the nearest minion plus its bursts, each burst hitting
/// `aoe` minions (Levi's spin and cuts are SPIN_R 18,000 wide, est. 3 minions).
fn model_wave_clear(m: Model, level: i64, aoe: usize) -> u64 {
    let (_, attack, _, _) = m.at(level);
    let mut minions = wave(level, 10, 330000);
    let mut ready = vec![0u64; m.skills.len()];
    let (mut basic, mut walk) = (0u64, 30000 - m.range.min(30000));
    for tick in 1..=3600u64 {
        if minions.iter().all(|u| !u.alive()) {
            return tick;
        }
        if walk > 0 {
            walk = (walk - m.at(level).3).max(0);
            continue;
        }
        for (i, b) in m.skills.iter().enumerate() {
            if level < b.from || tick < ready[i] || b.damage + b.ratio == 0 {
                continue;
            }
            for u in minions.iter_mut().filter(|u| u.alive()).take(aoe) {
                u.hp -= mitigate(b.damage + attack * b.ratio / 100, u.defence) * b.hits as i64;
            }
            ready[i] = tick + b.cd;
        }
        if tick >= basic {
            if let Some(u) = minions.iter_mut().find(|u| u.alive()) {
                u.hp -= mitigate(attack * m.basic_ratio / 100, u.defence);
            }
            basic = tick + m.cd();
        }
    }
    3600
}

/// One minute of laning: both waves and the enemy champion. Returns Raze's and the enemy's champion damage.
fn lane(m: Model, level: i64, seed: u64) -> (i64, i64) {
    let (a, b, _) = lane_hits(m, level, seed);
    (a, b)
}
/// As `lane`, also returning (Paint Shells thrown at a live champion, those that hit one).
fn lane_hits(m: Model, level: i64, seed: u64) -> (i64, i64, (u64, u64)) {
    let mut peer = Peer::new(m, seed);
    peer.shuffle = !m.melee;
    let mut units = vec![peer.unit(level, Point::new(345000, 300000))];
    units.extend(wave(level, 10, 325000));
    let mut s = Sim::new(level, seed, units);
    for _ in 0..3600 {
        s.step();
        s.gun_and_walk(14000);
        peer.act(&mut s, level);
        if !s.f.units[0].alive() || !s.f.units[1].alive() {
            break;
        }
    }
    (s.dealt, s.taken, (s.thrown, s.landed.len() as u64))
}

/// Ticks for Raze alone to take an objective (kind 3, 4,000 + 400 per level HP, 40 armour).
fn objective(level: i64, seed: u64) -> u64 {
    let hp = 4000 + 400 * level;
    let o = Unit {
        id: 2,
        team: 1,
        kind: 3,
        pos: Point::new(330000, 300000),
        hp,
        max_hp: hp,
        defence: 40,
        visible: true,
        targetable: true,
        ..Default::default()
    };
    let mut s = Sim::new(level, seed, vec![o]);
    s.f.objective = Some(2);
    for _ in 0..7200 {
        s.step();
        s.gun_and_walk(9000);
        if !s.f.units[1].alive() {
            return s.f.tick;
        }
    }
    7200
}

fn model_objective(m: Model, level: i64) -> u64 {
    let (_, attack, _, _) = m.at(level);
    let mut hp = 4000 + 400 * level;
    let mut ready = vec![0u64; m.skills.len()];
    let mut basic = 0;
    for tick in 1..=7200u64 {
        for (i, b) in m.skills.iter().enumerate() {
            if level >= b.from && tick >= ready[i] && b.damage + b.ratio > 0 {
                hp -= mitigate(b.damage + attack * b.ratio / 100, 40) * b.hits as i64;
                ready[i] = tick + b.cd;
            }
        }
        if tick >= basic {
            hp -= mitigate(attack * m.basic_ratio / 100, 40);
            basic = tick + m.cd();
        }
        if hp <= 0 {
            return tick;
        }
    }
    7200
}

const LEVELS: [i64; 4] = [1, 6, 10, 15];

#[test]
fn harness_runs_a_few_duels_lanes_and_objectives() {
    for m in [LEVI, STOCK] {
        let t = series(m, 6, 3);
        assert_eq!(t.duels, 3);
        assert!(t.dealt > 0, "{} never took damage", m.name);
        assert!(t.taken > 0, "Raze never took damage from {}", m.name);
    }
    assert!(raze_wave_clear(6, 1) < 3600, "Raze clears a wave");
    assert!(model_wave_clear(LEVI, 6, 3) < 3600);
    assert!(lane(STOCK, 6, 1).0 > 0);
    assert!(objective(6, 1) < 7200, "Raze takes an objective");
}

#[test]
#[ignore]
fn raze_viability() {
    println!("\nDuels ({SEEDS} seeded starts each, 30 s cap; win% counts draws by HP share)");
    println!("{:<14} {:>5} {:>6} {:>7} {:>7} {:>7} {:>7}", "Opponent", "Level", "Win %", "K/min", "D/min", "Dmg %", "Hit %");
    let (mut all_thrown, mut all_landed) = (0, 0);
    let mut peers = vec![];
    for m in PEERS.iter().chain([STOCK].iter()) {
        for level in LEVELS {
            let t = series(*m, level, SEEDS);
            let minutes = t.ticks as f64 / 3600.0;
            let win = 100.0 * t.wins / t.duels as f64;
            if m.name != STOCK.name {
                peers.push(win);
            }
            all_thrown += t.thrown;
            all_landed += t.landed;
            println!(
                "{:<14} {:>5} {:>6.1} {:>7.2} {:>7.2} {:>7.1} {:>7.1}",
                m.name,
                level,
                win,
                t.kills as f64 / minutes,
                t.deaths as f64 / minutes,
                100.0 * t.dealt as f64 / (t.dealt + t.taken).max(1) as f64,
                100.0 * t.landed as f64 / t.thrown.max(1) as f64
            );
        }
    }
    let avg = peers.iter().sum::<f64>() / peers.len() as f64;
    println!("Average against peers: {avg:.1}% (target 45–55%)");
    println!(
        "Paint Shells on champions in duels: {all_landed}/{all_thrown} hit ({:.1}%)",
        100.0 * all_landed as f64 / all_thrown.max(1) as f64
    );

    for m in [STOCK, STOCK_MELEE] {
        println!("\nLane: one minute against the {} laner (damage to champions)", m.name.to_lowercase());
        println!("{:>5} {:>8} {:>8} {:>7} {:>10}", "Level", "Raze", "Laner", "Ratio", "Hit/thrown");
        let (mut tt, mut tl) = (0, 0);
        for level in LEVELS {
            let (mut a, mut b, mut thrown, mut landed) = (0, 0, 0, 0);
            for seed in 1..=8 {
                let (x, y, (t, l)) = lane_hits(m, level, seed);
                a += x;
                b += y;
                thrown += t;
                landed += l;
            }
            tt += thrown;
            tl += landed;
            println!("{:>5} {:>8} {:>8} {:>7.2} {:>5}/{:<4}", level, a / 8, b / 8, a as f64 / b.max(1) as f64, landed, thrown);
        }
        println!("Paint Shells hit rate: {:.1}%", 100.0 * tl as f64 / tt.max(1) as f64);
    }

    println!("\nWave clear (six minions, seconds) and objective (seconds)");
    println!("{:>5} {:>6} {:>6} {:>6} {:>7} {:>7} {:>7}", "Level", "Raze", "Levi", "Stock", "Obj R", "Obj L", "Obj S");
    for level in LEVELS {
        let raze = (1..=4).map(|s| raze_wave_clear(level, s)).sum::<u64>() / 4;
        let obj = (1..=4).map(|s| objective(level, s)).sum::<u64>() / 4;
        println!(
            "{:>5} {:>6.1} {:>6.1} {:>6.1} {:>7.1} {:>7.1} {:>7.1}",
            level,
            raze as f64 / 60.0,
            model_wave_clear(LEVI, level, 3) as f64 / 60.0,
            model_wave_clear(STOCK, level, 1) as f64 / 60.0,
            obj as f64 / 60.0,
            model_objective(LEVI, level) as f64 / 60.0,
            model_objective(STOCK, level) as f64 / 60.0,
        );
    }
}

#[test]
#[ignore]
fn raze_viability_breakdown() {
    for (m, level) in [(MINATO, 10), (STOCK, 6), (LEVI, 10), (DAVID, 6)] {
        let mut peer = Peer::new(m, 3);
        let pos = Point::new(360000, 300000);
        let mut s = Sim::new(level, 3, vec![peer.unit(level, pos)]);
        let mut gun = 0;
        for _ in 0..DUEL_TICKS {
            s.step();
            let mid = s.dealt;
            s.gun_and_walk(9000);
            gun += s.dealt - mid;
            peer.act(&mut s, level);
            if !s.f.units[0].alive() || !s.f.units[1].alive() {
                break;
            }
        }
        println!(
            "{} L{level}: tick {} raze hp {}/{} peer hp {}/{} gun {gun} sources {:?}",
            m.name,
            s.f.tick,
            s.f.units[0].hp,
            s.f.units[0].max_hp,
            s.f.units[1].hp,
            s.f.units[1].max_hp,
            s.core.metrics.damage_by_source
        );
    }
}
