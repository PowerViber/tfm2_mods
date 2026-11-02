//! Round 150: is Rally viable, and does her athlete AI play varied, safe volleyball? Duels, a lane, a jungle camp and
//! a long fight sequence, all host-free.
//!
//! Rally is the real shared core (`Core::step`, the same code as the game and the Lab), driven in automatic mode
//! (`manual: false`). The engine side is modelled here, as in `raze_viability.rs`: her basic attack (the engine
//! fires it every BASIC_CD at its range, the core turns it into a palm or a volley), her walk to that range while the
//! core is idle, shields, armour, and the knock-ups and slows her spikes put on the peer.
//!
//! The peers are scripted models built from their shipped data plus their Mod Power line, exactly the models of
//! `raze_viability.rs` (Levi, V1, Minato, David) plus Raze and Contender (from `rules/contender.json`). Damage that
//! only lives in native code is estimated. Peers walk to their range, land PEER_ACCURACY of their skills after a
//! seeded reaction, and walk out of a public Rally warning (Impossible, Serve, storm) WARNING_DODGE ticks after it
//! appears, the way the game's avoid layer reads `rlyw:` zones.
//!
//! `cargo test --release rally_viability -- --ignored --nocapture` prints the tables; the smoke test keeps it alive.
use crate::rally_core::data::*;
use crate::rally_core::*;
use std::collections::BTreeMap;

const DUEL_TICKS: u64 = 1800;
const SEEDS: u64 = 24;
const PEER_ACCURACY: u64 = 80;
const WARNING_DODGE: u64 = 20;

#[derive(Clone, Copy)]
struct Burst {
    damage: i64,
    ratio: i64,
    cd: u64,
    range: i64,
    gap: Option<i64>,
    from: i64,
    hits: u64,
    every: u64,
    /// Stun on Rally when it lands (ticks).
    stun: u64,
}
const fn burst(damage: i64, ratio: i64, cd: u64, range: i64, gap: Option<i64>, from: i64) -> Burst {
    Burst { damage, ratio, cd, range, gap, from, hits: 1, every: 1, stun: 0 }
}
const fn channel(b: Burst, hits: u64, every: u64) -> Burst {
    Burst { damage: b.damage / hits as i64, ratio: b.ratio / hits as i64, hits, every, ..b }
}
const fn stunning(b: Burst, stun: u64) -> Burst {
    Burst { stun, ..b }
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
        burst(30, 50, 60, 200000, Some(12000), 1),
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
        burst(0, 0, 600, 200000, Some(20000), 1),
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
        burst(40, 60, 420, 160000, Some(15000), 1),
        burst(60, 120, 840, 25000, None, 1),
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
        burst(0, 0, 45, 60000, Some(15000), 1),
        channel(burst(210, 380, 480, 90000, Some(15000), 1), 10, 6),
        channel(burst(200, 200, 3600, 80000, None, 5), 3, 20),
    ],
};
/// Raze from her shipped data and Mod Power [35, 0, 10, 10, 10, 0, 10]; her gun as a ranged basic, Paint Shells and
/// Showstopper as estimated bursts.
const RAZE: Model = Model {
    name: "Raze",
    hp: (850, 90),
    attack: (50, 8),
    defence: (25, 6),
    speed: (1020, 8),
    power: [35, 10, 10, 10],
    basic_ratio: 100,
    basic_cd: 40,
    range: 45000,
    melee: false,
    skills: &[
        // Paint Shells (main blast plus clusters, est.)
        burst(70, 90, 600, 90000, None, 1),
        // Blast Pack hop
        burst(0, 0, 480, 120000, Some(30000), 1),
        // Showstopper (est.)
        burst(300, 200, 3600, 150000, None, 5),
    ],
};
/// Contender from `rules/contender.json` and Mod Power attack 30 / hp 5 / defence 10 / attack speed 10.
const CONTENDER: Model = Model {
    name: "Contender",
    hp: (1050, 95),
    attack: (85, 10),
    defence: (24, 4),
    speed: (1050, 10),
    power: [30, 5, 10, 10],
    basic_ratio: 100,
    basic_cd: 54,
    range: 23000,
    melee: true,
    skills: &[
        // Combination: 20 + 55%, then 30 + 75%, dash 14,000
        channel(burst(50, 130, 240, 37000, Some(15000), 1), 2, 18),
        // Uppercut: 55 + 85%, airborne 24 ticks
        stunning(burst(55, 85, 480, 32000, Some(15000), 1), 24),
        // Main Event (est.: eight seconds of 4/3 recovery, as a channel)
        channel(burst(240, 300, 4500, 65000, Some(15000), 5), 6, 60),
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
const PEERS: [Model; 6] = [LEVI, V1, MINATO, DAVID, RAZE, CONTENDER];

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
fn unit(id: usize, team: usize, pos: Point, hp: i64, attack: i64) -> Unit {
    Unit {
        id,
        team,
        pos,
        velocity: Point::default(),
        hp,
        max_hp: hp,
        attack,
        radius: 10000,
        champion: true,
        damageable: true,
        visible: true,
        targetable: true,
        farmable: false,
        controlled: false,
        cc_immune: false,
        control_eligible: true,
    }
}
/// Rally's stats at a level, read from the shipped `tfm2_custom_rally.data_champion` (stat + growth per level) and
/// her Mod Power line [attack, magic, hp, armour, resistance, speed, attack speed]. `RALLY_TUNE=attack,armour`
/// overrides the two growth values for a tuning sweep without regenerating.
fn rally_stats(level: i64) -> (i64, i64, i64, i64) {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../mods/tfm2_custom/champion/tfm2_custom_rally.data_champion");
    let data: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).expect("Rally champion data")).expect("Rally JSON");
    let get = |block: &str, key: &str| data[block][key].as_i64().unwrap_or(0);
    let mut growth = (get("growth", "attack"), get("growth", "defence"));
    if let Ok(t) = std::env::var("RALLY_TUNE") {
        let v: Vec<i64> = t.split(',').filter_map(|x| x.trim().parse().ok()).collect();
        if v.len() == 2 {
            growth = (v[0], v[1]);
        }
    }
    let l = level - 1;
    let p = MOD_POWER;
    (
        (get("stat", "hp") + get("growth", "hp") * l) * (100 + p[2] as i64) / 100,
        (get("stat", "attack") + growth.0 * l) * (100 + p[0] as i64) / 100,
        (get("stat", "defence") + growth.1 * l) * (100 + p[3] as i64) / 100,
        get("stat", "move_speed") + get("growth", "move_speed") * l,
    )
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
    plays: BTreeMap<String, u32>,
    impossible_tries: u32,
    impossible_lands: u32,
    charge_deaths: u32,
    /// Pace lost on death, and ticks spent at full Pace (hoarded).
    wasted: i64,
    full_ticks: u64,
    /// Champion damage by core source (palm, volley, spike, black_thunder, white_storm, impossible_spike).
    by_source: BTreeMap<String, i64>,
}
impl Tally {
    fn add(&mut self, o: &Tally) {
        self.duels += o.duels;
        self.wins += o.wins;
        self.kills += o.kills;
        self.deaths += o.deaths;
        self.ticks += o.ticks;
        self.dealt += o.dealt;
        self.taken += o.taken;
        for (k, v) in &o.plays {
            *self.plays.entry(k.clone()).or_default() += v;
        }
        self.impossible_tries += o.impossible_tries;
        self.impossible_lands += o.impossible_lands;
        self.charge_deaths += o.charge_deaths;
        self.wasted += o.wasted;
        self.full_ticks += o.full_ticks;
        for (k, v) in &o.by_source {
            *self.by_source.entry(k.clone()).or_default() += v;
        }
    }
}

/// The engine side of one Rally.
struct Sim {
    core: Core,
    f: Frame,
    defence: i64,
    /// Peer armour by id (Rally's Unit has no armour field; the engine mitigates).
    armour: BTreeMap<usize, i64>,
    layers: Vec<(i64, u64)>,
    basic_ready: u64,
    stunned_until: BTreeMap<usize, u64>,
    slowed: BTreeMap<usize, (i64, u64)>,
    /// Public warnings: (pos, radius, appeared, until).
    warnings: BTreeMap<u64, (Point, i64, u64, u64)>,
    last: BTreeMap<usize, Point>,
    t: Tally,
    cds: [u64; 4],
    was_impossible: bool,
    impossible_serials: Vec<u64>,
}
impl Sim {
    fn new(level: i64, seed: u64, profile: i64, mut units: Vec<Unit>) -> Self {
        let (hp, attack, defence, speed) = rally_stats(level);
        units.insert(0, unit(1, 0, Point { x: 300000, y: 300000 }, hp, attack));
        Self {
            core: Core::default(),
            f: Frame {
                owner: 1,
                seed,
                units,
                manual: false,
                profile: [profile; 13],
                move_step: speed,
                skill1: true,
                skill2: level >= 3,
                ultimate: level >= 5,
                ..Frame::default()
            },
            defence,
            armour: BTreeMap::new(),
            layers: vec![],
            basic_ready: 0,
            stunned_until: BTreeMap::new(),
            slowed: BTreeMap::new(),
            warnings: BTreeMap::new(),
            last: BTreeMap::new(),
            t: Tally::default(),
            cds: [0; 4],
            was_impossible: false,
            impossible_serials: vec![],
        }
    }
    fn me(&self) -> &Unit {
        &self.f.units[0]
    }
    fn unit(&mut self, id: usize) -> Option<&mut Unit> {
        self.f.units.iter_mut().find(|u| u.id == id)
    }
    fn controlled(&self, id: usize) -> bool {
        self.stunned_until.get(&id).is_some_and(|t| self.f.tick < *t)
    }
    fn speed(&self, id: usize, base: i64) -> i64 {
        match self.slowed.get(&id) {
            Some((p, until)) if self.f.tick < *until => base * (100 - p) / 100,
            _ => base,
        }
    }
    fn hit_rally(&mut self, raw: i64) {
        let mut n = mitigate(raw, self.defence);
        self.t.taken += n;
        for l in self.layers.iter_mut() {
            let used = l.0.min(n);
            l.0 -= used;
            n -= used;
        }
        self.f.units[0].hp -= n;
    }
    fn alive(&self) -> bool {
        self.me().hp > 0
    }
    /// One tick: the engine's basic and walk, then the core and its effects.
    fn step(&mut self) {
        self.f.tick += 1;
        let now = self.f.tick;
        self.layers.retain(|(a, end)| *a > 0 && now < *end);
        self.warnings.retain(|_, w| now < w.3);
        for u in self.f.units.iter_mut() {
            u.velocity = self.last.get(&u.id).map_or(Point::default(), |p| Point { x: u.pos.x - p.x, y: u.pos.y - p.y });
            self.last.insert(u.id, u.pos);
        }
        let stunned = self.controlled(1);
        self.f.interrupted = stunned;
        let ids: Vec<usize> = self.f.units.iter().map(|u| u.id).collect();
        for id in ids {
            let c = self.controlled(id);
            if let Some(u) = self.unit(id) {
                u.controlled = c;
            }
        }
        let me = self.me().clone();
        self.f.danger = self
            .f
            .units
            .iter()
            .filter(|u| u.champion && u.team != 0 && u.hp > 0 && u.pos.distance(me.pos) < u.radius + me.radius + 35000)
            .map(|u| u.attack / 3)
            .sum();
        // the engine's basic attack on the nearest damageable enemy (champions first), and her walk to its range
        self.f.basics.clear();
        let target = self
            .f
            .units
            .iter()
            .filter(|u| u.team != 0 && u.hp > 0 && u.damageable)
            .min_by_key(|u| (!u.champion, u.pos.distance(me.pos)))
            .cloned();
        if let Some(t) = target {
            let reach = PALM_RANGE + if self.core.ball == Ball::Held { VOLLEY_RANGE - PALM_RANGE } else { 0 };
            let edge = (me.pos.distance(t.pos) - me.radius - t.radius).max(0);
            if !self.core.busy() && !stunned {
                if edge <= reach && now >= self.basic_ready {
                    self.f.basics.push(Basic { serial: now, target: t.id });
                    self.basic_ready = now + (BASIC_CD as u64) * 100 / (100 + MOD_POWER[6] as u64);
                } else if edge > reach - 2000 {
                    let step = self.speed(1, self.f.move_step).min(edge - reach + 2000);
                    self.f.units[0].pos = me.pos.toward(t.pos, step);
                }
            }
        }
        let out = self.core.step(&self.f);
        self.f.hits.clear();
        for e in out {
            match e {
                Effect::Damage { serial, target, amount, source, .. } => {
                    let defence = self.armour.get(&target).copied().unwrap_or(10);
                    let Some(u) = self.unit(target) else { continue };
                    if u.hp <= 0 || u.team == 0 {
                        continue;
                    }
                    let champion = u.champion;
                    let n = mitigate(amount, defence).min(u.hp);
                    u.hp -= n;
                    if champion {
                        self.t.dealt += n;
                        *self.t.by_source.entry(source.clone()).or_default() += n;
                        if source == "impossible_spike" && !self.impossible_serials.contains(&serial) {
                            self.impossible_serials.push(serial);
                            self.t.impossible_lands += 1;
                        }
                    }
                    self.f.hits.push(Hit { serial, target, delivered: n > 0, champion });
                }
                Effect::Move { pos } => self.f.units[0].pos = pos,
                Effect::Shield { amount, ticks } => self.layers.push((amount, now + ticks)),
                Effect::KnockUp { target, ticks } => {
                    let e = self.stunned_until.entry(target).or_default();
                    *e = (*e).max(now + ticks);
                }
                Effect::Slow { target, percent, ticks } => {
                    self.slowed.insert(target, (percent, now + ticks));
                }
                Effect::Warning { public: true, serial, pos, radius, until, .. } => {
                    let appeared = self.warnings.get(&serial).map_or(now, |w| w.2);
                    self.warnings.insert(serial, (pos, radius, appeared, until));
                }
                Effect::ClearWarning { serial } => {
                    self.warnings.remove(&serial);
                }
                _ => {}
            }
        }
        // a play starts when one of her skills goes on cooldown (Receive, Self Set, the run-up, Serve)
        if self.core.cds.iter().zip(&self.cds).any(|(now, before)| now > before) {
            *self.t.plays.entry(format!("{:?}", self.core.play)).or_default() += 1;
        }
        self.cds = self.core.cds;
        let impossible = matches!(self.core.phase, Phase::Impossible { .. });
        if impossible && !self.was_impossible {
            self.t.impossible_tries += 1;
        }
        self.was_impossible = impossible;
        if self.core.pace >= 100 {
            self.t.full_ticks += 1;
        }
    }
    /// Her death: Pace lost, a death during the Impossible charge, and the core's own cleanup.
    fn rally_died(&mut self) {
        self.t.wasted += self.core.pace;
        if matches!(self.core.phase, Phase::Impossible { .. }) {
            self.t.charge_deaths += 1;
        }
    }
}

struct Peer {
    m: Model,
    id: usize,
    defence: i64,
    speed: i64,
    ready: Vec<u64>,
    basic_ready: u64,
    pending: Vec<(u64, i64, u64)>,
    seed: u64,
    reaction: u64,
}
impl Peer {
    fn new(m: Model, id: usize, level: i64, seed: u64) -> Self {
        let (_, _, defence, speed) = m.at(level);
        Self {
            m,
            id,
            defence,
            speed,
            ready: vec![0; m.skills.len()],
            basic_ready: 0,
            pending: vec![],
            seed,
            reaction: 15 + seed * 13 % 45,
        }
    }
    fn unit(&self, level: i64, pos: Point) -> Unit {
        let (hp, attack, _, _) = self.m.at(level);
        unit(self.id, 1, pos, hp, attack)
    }
    fn lands(&self, tick: u64, skill: usize) -> bool {
        let h = (self.seed ^ tick.wrapping_mul(0x9e37_79b9_7f4a_7c15) ^ (skill as u64) << 32)
            .wrapping_mul(0xbf58_476d_1ce4_e5b9);
        (h >> 33) % 100 < PEER_ACCURACY
    }
    fn act(&mut self, s: &mut Sim, level: i64) {
        let Some(me) = s.f.units.iter().find(|u| u.id == self.id).cloned() else { return };
        if me.hp <= 0 || s.controlled(self.id) {
            return;
        }
        let rally = s.me().clone();
        let mut d = me.pos.distance(rally.pos);
        let now = s.f.tick;
        for (_, raw, stun) in self.pending.iter().filter(|(t, _, _)| *t <= now) {
            s.hit_rally(*raw);
            if *stun > 0 {
                let e = s.stunned_until.entry(1).or_default();
                *e = (*e).max(now + stun);
            }
        }
        self.pending.retain(|(t, _, _)| *t > now);
        // the avoid layer: walk out of a public Rally warning once it has been up WARNING_DODGE ticks
        if let Some(pos) = s
            .warnings
            .values()
            .find(|w| now >= w.2 + WARNING_DODGE && me.pos.distance(w.0) <= w.1 + me.radius)
            .map(|w| w.0)
        {
            let away = if me.pos == pos { Point { x: me.pos.x + 1000, y: me.pos.y } } else { Point { x: 2 * me.pos.x - pos.x, y: 2 * me.pos.y - pos.y } };
            let step = s.speed(self.id, self.speed);
            s.unit(self.id).unwrap().pos = me.pos.toward(away, step);
            return;
        }
        for (i, b) in self.m.skills.iter().enumerate() {
            if level < b.from || now < self.ready[i] || d > b.range || now < self.reaction {
                continue;
            }
            if let Some(gap) = b.gap {
                if d > gap {
                    s.unit(self.id).unwrap().pos = rally.pos.toward(me.pos, gap);
                    d = gap;
                }
            }
            if b.damage + b.ratio > 0 && self.lands(now, i) {
                for n in 0..b.hits {
                    self.pending.push((now + n * b.every, b.damage + me.attack * b.ratio / 100, if n == 0 { b.stun } else { 0 }));
                }
            }
            self.ready[i] = now + b.cd;
        }
        if d <= self.m.range + 20000 {
            if now >= self.basic_ready.max(self.reaction) {
                s.hit_rally(me.attack * self.m.basic_ratio / 100);
                self.basic_ready = now + self.m.cd();
            }
            if !self.m.melee && d < self.m.range * 8 / 10 {
                let away = Point { x: 2 * me.pos.x - rally.pos.x, y: 2 * me.pos.y - rally.pos.y };
                let step = s.speed(self.id, self.speed);
                s.unit(self.id).unwrap().pos = me.pos.toward(away, step);
            }
        } else {
            let step = s.speed(self.id, self.speed).min(d - self.m.range);
            s.unit(self.id).unwrap().pos = me.pos.toward(rally.pos, step);
        }
    }
}

/// One duel from a seeded start. Rally's score: 1 win, 0 loss, HP share on a timeout.
fn duel(m: Model, level: i64, seed: u64, profile: i64) -> Tally {
    let mut peer = Peer::new(m, 2, level, seed);
    let angle = (seed * 37 % 360) as f64 * std::f64::consts::PI / 180.0;
    let gap = 40000 + (seed * 7919 % 50) as i64 * 1000;
    let pos = Point { x: 300000 + (gap as f64 * angle.cos()) as i64, y: 300000 + (gap as f64 * angle.sin()) as i64 };
    let mut s = Sim::new(level, seed, profile, vec![peer.unit(level, pos)]);
    s.armour.insert(peer.id, peer.defence);
    s.t.duels = 1;
    for _ in 0..DUEL_TICKS {
        s.step();
        peer.act(&mut s, level);
        let (a, b) = (s.alive(), s.f.units[1].hp > 0);
        if !a {
            s.rally_died();
        }
        if !a || !b {
            s.t.wins = if a { 1.0 } else if b { 0.0 } else { 0.5 };
            s.t.kills = (!b) as u32;
            s.t.deaths = (!a) as u32;
            s.t.ticks = s.f.tick;
            return s.t;
        }
    }
    let a = s.me().hp as f64 / s.me().max_hp as f64;
    let b = s.f.units[1].hp as f64 / s.f.units[1].max_hp as f64;
    s.t.wins = if a > b { 1.0 } else if a < b { 0.0 } else { 0.5 };
    s.t.ticks = s.f.tick;
    s.t
}
fn series(m: Model, level: i64, seeds: u64, profile: impl Fn(u64) -> i64) -> Tally {
    let mut t = Tally::default();
    for seed in 1..=seeds {
        t.add(&duel(m, level, seed, profile(seed)));
    }
    t
}
/// The raze harness's athlete spread: 50, 55, 60, 65, 70 by seed.
fn spread(seed: u64) -> i64 {
    50 + (seed % 5) as i64 * 5
}

/// A long fight sequence: three minutes against a peer that respawns three seconds after each death, Rally keeping
/// her Pace and forms (and respawning herself after a death, Pace lost).
fn long_fight(m: Model, level: i64, seed: u64, profile: i64) -> Tally {
    let mut peer = Peer::new(m, 2, level, seed);
    let start = Point { x: 350000, y: 300000 };
    let mut s = Sim::new(level, seed, profile, vec![peer.unit(level, start)]);
    s.armour.insert(peer.id, peer.defence);
    let (mut peer_back, mut rally_back) = (0u64, 0u64);
    for _ in 0..10800 {
        let now = s.f.tick;
        if s.f.units[1].hp <= 0 && now >= peer_back {
            if peer_back == 0 {
                peer_back = now + 180;
                s.t.kills += 1;
            } else {
                let fresh = peer.unit(level, start);
                s.f.units[1] = fresh;
                peer = Peer::new(m, 2, level, seed.wrapping_add(now));
                peer_back = 0;
            }
        }
        if !s.alive() {
            if rally_back == 0 {
                s.rally_died();
                s.t.deaths += 1;
                let _ = s.core.died();
                rally_back = now + 300;
                s.core.pace = 0;
            } else if now >= rally_back {
                let (hp, ..) = rally_stats(level);
                s.f.units[0].hp = hp;
                s.f.units[0].pos = Point { x: 300000, y: 300000 };
                s.core.phase = Phase::Idle;
                s.core.ball = Ball::Held;
                rally_back = 0;
            }
            s.f.tick += 1;
            continue;
        }
        s.step();
        if s.f.units[1].hp > 0 {
            peer.act(&mut s, level);
        }
    }
    s.t.ticks = s.f.tick;
    s.t
}

/// One minute of laning against a stock laner with a wave in front of him: Rally's and the laner's champion damage.
fn lane(m: Model, level: i64, seed: u64) -> (i64, i64) {
    let mut peer = Peer::new(m, 2, level, seed);
    let mut units = vec![peer.unit(level, Point { x: 360000, y: 300000 })];
    for i in 0..6 {
        let hp = if i < 3 { 450 } else { 300 } + 22 * (level - 1);
        let mut u = unit(10 + i, 1, Point { x: 335000 + (i as i64 / 3) * 6000, y: 296000 + (i as i64 % 3) * 4000 }, hp, 20);
        u.champion = false;
        u.farmable = true;
        units.push(u);
    }
    let mut s = Sim::new(level, seed, 60, units);
    s.armour.insert(peer.id, peer.defence);
    for _ in 0..3600 {
        s.step();
        peer.act(&mut s, level);
        if !s.alive() || s.f.units[1].hp <= 0 {
            break;
        }
    }
    (s.t.dealt, s.t.taken)
}

/// Seconds for Rally alone to clear a three-monster jungle camp (1,000 + 120 per level HP each, 30 attack).
fn jungle(level: i64, seed: u64) -> u64 {
    let units = (0..3)
        .map(|i| {
            let mut u = unit(20 + i, 2, Point { x: 340000 + i as i64 * 9000, y: 300000 }, 1000 + 120 * level, 30);
            u.champion = false;
            u.farmable = true;
            u
        })
        .collect();
    let mut s = Sim::new(level, seed, 60, units);
    for _ in 0..7200 {
        s.step();
        let now = s.f.tick;
        // the camp fights back at whoever is closest
        if now % 60 == 0 {
            let near = s.f.units[1..].iter().filter(|u| u.hp > 0).count() as i64;
            s.hit_rally(30 * near);
        }
        if s.f.units[1..].iter().all(|u| u.hp <= 0) {
            return now;
        }
        if !s.alive() {
            return 7200;
        }
    }
    7200
}

const LEVELS: [i64; 4] = [1, 6, 10, 15];

fn plays_line(t: &Tally) -> String {
    let total: u32 = t.plays.values().sum();
    t.plays
        .iter()
        .map(|(k, v)| format!("{k} {:.0}%", 100.0 * *v as f64 / total.max(1) as f64))
        .collect::<Vec<_>>()
        .join(", ")
}

#[test]
fn harness_runs_duels_a_lane_a_camp_and_a_long_fight() {
    for m in [LEVI, CONTENDER] {
        let t = series(m, 6, 3, spread);
        assert_eq!(t.duels, 3);
        assert!(t.dealt > 0, "Rally never hurt {}", m.name);
        assert!(t.taken > 0, "{} never hurt Rally", m.name);
        assert!(t.plays.values().sum::<u32>() > 0, "Rally never started a play against {}", m.name);
    }
    assert!(lane(STOCK, 6, 1).0 > 0);
    assert!(jungle(6, 1) < 7200, "Rally clears a camp");
}

#[test]
#[ignore]
fn rally_viability() {
    println!("\nDuels ({SEEDS} seeded starts each, 30 s cap; win% counts timeouts by HP share; athletes 50-70)");
    println!(
        "{:<11} {:>5} {:>6} {:>7} {:>7} {:>6} {:>9} {:>6} {:>7} {:>7}",
        "Opponent", "Level", "Win %", "K/min", "D/min", "Dmg %", "Imp try/ok", "ChgD", "Wasted", "Full %"
    );
    let mut peers = vec![];
    let mut all = Tally::default();
    for m in PEERS {
        for level in LEVELS {
            let t = series(m, level, SEEDS, spread);
            let minutes = t.ticks as f64 / 3600.0;
            let win = 100.0 * t.wins / t.duels as f64;
            peers.push(win);
            println!(
                "{:<11} {:>5} {:>6.1} {:>7.2} {:>7.2} {:>6.1} {:>5}/{:<4} {:>6} {:>7} {:>6.1}",
                m.name,
                level,
                win,
                t.kills as f64 / minutes,
                t.deaths as f64 / minutes,
                100.0 * t.dealt as f64 / (t.dealt + t.taken).max(1) as f64,
                t.impossible_tries,
                t.impossible_lands,
                t.charge_deaths,
                t.wasted / t.duels as i64,
                100.0 * t.full_ticks as f64 / t.ticks.max(1) as f64
            );
            all.add(&t);
        }
    }
    let avg = peers.iter().sum::<f64>() / peers.len() as f64;
    println!("Average against peers: {avg:.1}% (target 45-55%)");
    println!("Plays started in duels: {}", plays_line(&all));
    let dealt: i64 = all.by_source.values().sum();
    println!(
        "Champion damage by source in duels: {}",
        all.by_source.iter().map(|(k, v)| format!("{k} {:.0}%", 100.0 * *v as f64 / dealt.max(1) as f64)).collect::<Vec<_>>().join(", ")
    );
    println!(
        "Impossible in duels: {} attempts, {} landed, {} deaths during the charge",
        all.impossible_tries, all.impossible_lands, all.charge_deaths
    );

    println!("\nLong fight sequence (3 minutes, the peer respawns), level 15");
    println!("{:<11} {:>8} {:>7} {:>7} {:>10} {:>6} {:>7} {:>7}", "Opponent", "Athlete", "Kills", "Deaths", "Imp try/ok", "ChgD", "Wasted", "Full %");
    let mut long = Tally::default();
    for m in PEERS {
        for profile in [60, 90] {
            let mut t = Tally::default();
            for seed in 1..=4 {
                t.add(&long_fight(m, 15, seed, profile));
            }
            println!(
                "{:<11} {:>8} {:>7.2} {:>7.2} {:>5}/{:<4} {:>6} {:>7} {:>6.1}",
                m.name,
                profile,
                t.kills as f64 / 4.0,
                t.deaths as f64 / 4.0,
                t.impossible_tries,
                t.impossible_lands,
                t.charge_deaths,
                t.wasted / 4,
                100.0 * t.full_ticks as f64 / t.ticks.max(1) as f64
            );
            if profile == 90 {
                long.add(&t);
            }
        }
    }
    println!(
        "Top athlete (90) per 3-minute sequence: {:.2} Impossible attempts, {:.2} landed, charge deaths {:.0}% of attempts",
        long.impossible_tries as f64 / 24.0,
        long.impossible_lands as f64 / 24.0,
        100.0 * long.charge_deaths as f64 / long.impossible_tries.max(1) as f64
    );
    println!("Plays started (top athlete, long fights): {}", plays_line(&long));

    println!("\nLane: one minute against the stock ranged laner (damage to champions)");
    println!("{:>5} {:>8} {:>8} {:>7}", "Level", "Rally", "Laner", "Ratio");
    for level in LEVELS {
        let (mut a, mut b) = (0, 0);
        for seed in 1..=8 {
            let (x, y) = lane(STOCK, level, seed);
            a += x;
            b += y;
        }
        println!("{:>5} {:>8} {:>8} {:>7.2}", level, a / 8, b / 8, a as f64 / b.max(1) as f64);
    }
    println!("\nJungle: one three-monster camp (seconds)");
    for level in LEVELS {
        let t = (1..=4).map(|s| jungle(level, s)).sum::<u64>() / 4;
        println!("{:>5} {:>6.1}", level, t as f64 / 60.0);
    }
}
