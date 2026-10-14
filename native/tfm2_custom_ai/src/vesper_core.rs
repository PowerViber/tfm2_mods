//! Vesper, the Rift Marksman: every rule as a pure, deterministic engine.
//!
//! `Core::step` takes one tick's snapshot (fighters, Vesper's state) and returns the commands the adapter
//! (`vesper.rs`) applies through the stable API. Nothing in here reads the simulation, so the native tests and the
//! Rift Lab (editor/vesper-combat.js) run exactly the same rules. Mastery only changes the planner (`think`): every
//! rift type, technique and dimension is available at every rank with identical numbers.
use crate::unified_theory_math::{ChargeBank, Window};
use crate::vesper_data::*;
use crate::vesper_math::*;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

/// Rank look for the art: 0..7 per rank, 8 for the #1 athlete.
pub fn look(rank: usize, top: Option<usize>) -> usize {
    if rank >= 7 && top == Some(1) {
        8
    } else {
        rank.min(7)
    }
}
pub fn art_tier(rank: usize) -> usize {
    [0, 0, 0, 1, 1, 2, 2, 3][rank.min(7)]
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fighter {
    pub id: usize,
    pub team: usize,
    pub x: i64,
    pub y: i64,
    pub hp: i64,
    pub max_hp: i64,
    pub attack: i64,
    pub cc_immune: bool,
    pub controlled: bool,
}

/// One tick of input.
pub struct Input<'a> {
    pub tick: usize,
    pub me: usize,
    /// Champions in the fight (Vesper included). Enemies are only the visible ones.
    pub fighters: &'a [Fighter],
    /// Ticks between Vesper's basic attacks (after attack-speed buffs).
    pub attack_interval: usize,
    pub stunned: bool,
    pub wall: &'a dyn Fn(i64, i64) -> bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dmg {
    Phys,
    Magic,
    True,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cc {
    Stun,
    Root,
    Silence,
    Disarm,
}
/// Buff fields the adapter maps onto `BuffV1`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Mods {
    pub move_speed: i32,
    pub attack_speed: i32,
    pub amplify: i32,
    pub reduce: i32,
    pub heal_reduce: i32,
    pub attack: i32,
    pub defence: i32,
    pub mr: i32,
    pub ap: i32,
    pub regen: i32,
    pub crit: i32,
    pub range: i32,
    pub ba_reduce: i32,
    pub reflect: i32,
    pub cooldown: i32,
    pub vamp: i32,
    pub ignore_wall: bool,
    pub undying: bool,
    pub cc_immune: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Cmd {
    Damage { target: usize, amount: i64, kind: Dmg },
    Heal { target: usize, amount: i64 },
    Shield { target: usize, amount: i64, ticks: usize },
    Cc { target: usize, kind: Cc, ticks: usize },
    Buff { target: usize, name: String, ticks: usize, mods: Mods },
    SetPos { target: usize, x: i64, y: i64 },
    Banish { target: usize, ticks: usize },
    Invisible { target: usize, ticks: usize },
    ClearCc { target: usize },
    ClearShield { target: usize },
    /// Non-lethal self harm (a Breach).
    LoseHp { target: usize, amount: i64 },
    /// A view effect `vesper_<tag>` at a point, or following a unit.
    Fx { tag: String, x: i64, y: i64, unit: Option<usize>, ticks: usize },
    Log(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rift {
    pub uid: u32,
    pub kind: usize,
    pub tech: usize,
    pub dim: usize,
    /// Stitch: a second dimension carried by the same rift.
    pub dim2: Option<usize>,
    /// Borrow: (dimension, until tick).
    pub borrowed: Option<(usize, usize)>,
    pub x: i64,
    pub y: i64,
    /// Second end of a Slash or Mirror Pane, or the end of a Wandering Rift's path.
    pub ex: i64,
    pub ey: i64,
    pub radius: i64,
    pub opens: usize,
    pub expires: usize,
    /// Anchor host (follows it).
    pub host: Option<usize>,
    /// The paired rift (Gate / Kidnap / Twin exit and the like).
    pub peer: Option<u32>,
    /// Fighter this rift was opened for (Kidnap victim, Rescue ally, Hook target, ...).
    pub subject: Option<usize>,
    /// Chain order (chain nodes only).
    pub chain: Option<u32>,
    pub entrance: bool,
    pub opened: bool,
    /// One-shot techniques (Step, Phase, Kidnap, Rescue, Swap) fire once both ends are open.
    pub fired: bool,
    pub dwell: BTreeMap<usize, usize>,
    pub touched: BTreeSet<usize>,
    pub sides: BTreeMap<usize, i32>,
    pub inside: BTreeSet<usize>,
    pub done: bool,
}
impl Rift {
    pub fn live(&self) -> bool {
        self.opened && !self.done
    }
    pub fn dims(&self, tick: usize) -> Vec<usize> {
        let mut v = vec![match self.borrowed {
            Some((d, until)) if tick < until => d,
            _ => self.dim,
        }];
        if let Some(d) = self.dim2 {
            v.push(d);
        }
        v
    }
    pub fn hidden(&self, tick: usize) -> bool {
        self.kind == R_SEED && tick < self.opens + SEED_HIDDEN as usize
    }
    /// Distance from a point to the rift's body (a segment for Slash / Mirror Pane).
    pub fn reach(&self, x: i64, y: i64) -> i64 {
        if matches!(self.kind, R_SLASH | R_MIRROR) {
            seg_dist(self.x, self.y, self.ex, self.ey, x, y)
        } else {
            dist(self.x, self.y, x, y)
        }
    }
    pub fn covers(&self, x: i64, y: i64) -> bool {
        self.reach(x, y) <= self.radius
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Packet {
    pub x: i64,
    pub y: i64,
    /// Where the current link started (its full length is what costs efficiency).
    pub hop: (i64, i64),
    /// Remaining waypoints: (rift uid or 0 for the target, x, y).
    pub path: VecDeque<(u32, i64, i64)>,
    pub target: usize,
    pub eff: i64,
    pub bonus: i64,
    pub dims: Vec<usize>,
    pub last_families: Vec<usize>,
    pub kaleido: bool,
    pub true_pct: i64,
    pub ignore_wall: bool,
    pub split: bool,
    pub echo: bool,
    pub source: &'static str,
    pub base: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Later {
    /// A packet copy fired at a target (Echo, Paradox).
    Repeat { at: usize, target: usize, amount: i64, families: Vec<usize> },
    /// The next rift in a Domino collapses.
    Domino { at: usize, uid: u32 },
    /// One Rift Rain drop.
    Drop { at: usize, x: i64, y: i64, amount: i64 },
    /// A swallowed fighter reappears (Kidnap, Seed, Sinkhole, Hook) at the exit.
    Spit { at: usize, target: usize, x: i64, y: i64, ally: bool },
    /// Vesper leaves a Phase Out.
    Unphase { at: usize, x: i64, y: i64 },
    /// A Sky Drop landing stun.
    Land { at: usize, x: i64, y: i64, ally: usize },
}
impl Later {
    fn at(&self) -> usize {
        match self {
            Later::Repeat { at, .. }
            | Later::Domino { at, .. }
            | Later::Drop { at, .. }
            | Later::Spit { at, .. }
            | Later::Unphase { at, .. }
            | Later::Land { at, .. } => *at,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Dot {
    pub target: usize,
    pub per_pulse: i64,
    pub until: usize,
    pub drain: bool,
}

/// Counters the fixtures and the Rift Lab report per rank.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Stats {
    pub tech_uses: [u32; 28],
    pub rifts_opened: u32,
    pub rifts_touching: u32,
    pub breaches: u32,
    pub breaks: u32,
    pub packets: u32,
    pub eff_sum: i64,
    pub damage: i64,
    pub kidnaps: u32,
    pub useful_kidnaps: u32,
    pub gates: u32,
    pub engaged_gates: u32,
    pub lonely_gates: u32,
    pub traps: u32,
    pub trap_catches: u32,
    pub risky_moves: u32,
    pub self_deaths: u32,
    pub implodes: u32,
    pub implode_hits: u32,
    pub habits: BTreeMap<&'static str, u32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RiftSpec {
    pub kind: usize,
    pub x: i64,
    pub y: i64,
    pub ex: i64,
    pub ey: i64,
    pub host: Option<usize>,
    pub chain: bool,
    pub entrance: bool,
    /// Spine upkeep: a refreshed node keeps the chain position of the node it replaces.
    pub order: Option<u32>,
}
impl RiftSpec {
    pub fn at(kind: usize, x: i64, y: i64) -> RiftSpec {
        RiftSpec { kind, x, y, ex: x, ey: y, host: None, chain: false, entrance: false, order: None }
    }
    pub fn line(kind: usize, x: i64, y: i64, ex: i64, ey: i64) -> RiftSpec {
        RiftSpec { ex, ey, ..RiftSpec::at(kind, x, y) }
    }
    pub fn node(kind: usize, x: i64, y: i64) -> RiftSpec {
        RiftSpec { chain: true, ..RiftSpec::at(kind, x, y) }
    }
    pub fn entrance(kind: usize, x: i64, y: i64) -> RiftSpec {
        RiftSpec { entrance: true, ..RiftSpec::at(kind, x, y) }
    }
}
/// A planned technique: the rifts to open plus what they are for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Action {
    pub tech: usize,
    pub rifts: Vec<RiftSpec>,
    pub subject: Option<usize>,
    /// Existing rifts the technique acts on (Implode, Domino, Stitch, Crossfire, Overload, Borrow).
    pub uses: Vec<u32>,
    pub score: i64,
    pub habit: &'static str,
}

#[derive(Clone, Debug)]
pub struct Core {
    pub rank: usize,
    pub top: Option<usize>,
    pub rng: u64,
    pub atlas: Atlas,
    pub charges: ChargeBank<5>,
    pub strain: i64,
    pub rifts: Vec<Rift>,
    pub next_uid: u32,
    pub chain_seq: u32,
    pub packets: Vec<Packet>,
    later: Vec<Later>,
    pub dots: Vec<Dot>,
    pub history: BTreeMap<usize, VecDeque<(i64, i64)>>,
    pub velocity: BTreeMap<usize, (i64, i64)>,
    damage: BTreeMap<usize, Window>,
    controls: BTreeMap<usize, Window>,
    moves: BTreeMap<usize, Window>,
    support: BTreeMap<usize, Window>,
    pub cooldowns: BTreeMap<(u32, usize), usize>,
    pub self_move_until: usize,
    pub next_attack: usize,
    pub aiming: bool,
    pub chain_target: Option<usize>,
    pub crossfire_until: usize,
    pub overload: i64,
    pub plan: VecDeque<usize>,
    pub plan_until: usize,
    pub last_think: usize,
    pub phased: bool,
    pub risky_until: usize,
    pub displaced: BTreeMap<usize, usize>,
    pub stats: Stats,
    pub fx_left: usize,
    pub last_tick: Option<usize>,
    pub hand_placed: bool,
}

impl Core {
    pub fn new(seed: u64, owner: usize, rank: usize, top: Option<usize>) -> Core {
        Core {
            rank: rank.min(7),
            top,
            rng: (seed ^ (owner as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15) ^ 0x5645_5350_4552).max(1),
            atlas: Atlas::new(seed),
            charges: ChargeBank::default(),
            strain: 0,
            rifts: vec![],
            next_uid: 1,
            chain_seq: 0,
            packets: vec![],
            later: vec![],
            dots: vec![],
            history: BTreeMap::new(),
            velocity: BTreeMap::new(),
            damage: BTreeMap::new(),
            controls: BTreeMap::new(),
            moves: BTreeMap::new(),
            support: BTreeMap::new(),
            cooldowns: BTreeMap::new(),
            self_move_until: 0,
            next_attack: 0,
            aiming: false,
            chain_target: None,
            crossfire_until: 0,
            overload: 0,
            plan: VecDeque::new(),
            plan_until: 0,
            last_think: 0,
            phased: false,
            risky_until: 0,
            displaced: BTreeMap::new(),
            stats: Stats::default(),
            fx_left: FX_PER_UPDATE as usize,
            last_tick: None,
            hand_placed: false,
        }
    }
    fn random(&mut self, n: u64) -> u64 {
        roll(&mut self.rng, n)
    }
    pub fn look(&self) -> usize {
        look(self.rank, self.top)
    }
    pub fn table(&self, t: &[i64; 8]) -> i64 {
        t[self.rank.min(7)]
    }
    pub fn gate(&self, g: usize) -> bool {
        self.rank >= g
    }
    fn fx(&mut self, out: &mut Vec<Cmd>, tag: String, x: i64, y: i64, unit: Option<usize>, ticks: usize) {
        if self.fx_left > 0 {
            self.fx_left -= 1;
            out.push(Cmd::Fx { tag, x, y, unit, ticks });
        }
    }
    fn habit(&mut self, h: &'static str) {
        if !h.is_empty() && self.rank >= habit_gate(h) {
            *self.stats.habits.entry(h).or_default() += 1;
        }
    }
    pub fn rift(&self, uid: u32) -> Option<&Rift> {
        self.rifts.iter().find(|r| r.uid == uid && !r.done)
    }
    fn rift_mut(&mut self, uid: u32) -> Option<&mut Rift> {
        self.rifts.iter_mut().find(|r| r.uid == uid && !r.done)
    }
    pub fn open_count(&self) -> usize {
        self.rifts.iter().filter(|r| !r.done).count()
    }
    fn cooldown_ok(&self, key: u32, id: usize, tick: usize) -> bool {
        self.cooldowns.get(&(key, id)).map_or(true, |until| tick >= *until)
    }
    fn set_cooldown(&mut self, key: u32, id: usize, until: usize) {
        self.cooldowns.insert((key, id), until);
    }

    // ---------------------------------------------------------------- shared limits
    /// All of Vesper's damage: 35% of the target's max HP per 2 s.
    fn emit(&mut self, out: &mut Vec<Cmd>, f: &Fighter, amount: i64, kind: Dmg, tick: usize) -> i64 {
        if amount <= 0 || f.hp <= 0 {
            return 0;
        }
        let cap = (f.max_hp * DAMAGE_PCT / 100).max(1) as usize;
        let n = self.damage.entry(f.id).or_default().allow(tick, DAMAGE_WINDOW as usize, cap, amount as usize) as i64;
        if n > 0 {
            out.push(Cmd::Damage { target: f.id, amount: n, kind });
            self.stats.damage += n;
        }
        n
    }
    /// Stuns, roots, silences, disarms and banishes share 60 ticks per 3 s per target.
    fn control(&mut self, out: &mut Vec<Cmd>, f: &Fighter, kind: Option<Cc>, ticks: i64, tick: usize) -> bool {
        if f.cc_immune || ticks <= 0 {
            return false;
        }
        let n = self.controls.entry(f.id).or_default().allow(tick, CONTROL_WINDOW as usize, CONTROL_CAP as usize, ticks as usize);
        if n == 0 {
            return false;
        }
        match kind {
            Some(k) => out.push(Cmd::Cc { target: f.id, kind: k, ticks: n }),
            None => out.push(Cmd::Banish { target: f.id, ticks: n }),
        }
        true
    }
    /// Forced movement: two per 2 s per target, never for crowd-control immune enemies.
    fn displace(&mut self, out: &mut Vec<Cmd>, f: &Fighter, x: i64, y: i64, tick: usize, wall: &dyn Fn(i64, i64) -> bool, enemy: bool) -> bool {
        if enemy && f.cc_immune {
            return false;
        }
        if self.moves.entry(f.id).or_default().allow(tick, MOVE_WINDOW as usize, MOVE_CAP as usize, 1) == 0 {
            return false;
        }
        let (x, y) = land(f.x, f.y, x, y, wall);
        out.push(Cmd::SetPos { target: f.id, x, y });
        self.displaced.insert(f.id, tick);
        true
    }
    fn shield(&mut self, out: &mut Vec<Cmd>, f: &Fighter, amount: i64, ticks: usize, tick: usize) {
        let cap = (f.max_hp * SUPPORT_PCT / 100).max(1) as usize;
        let n = self.support.entry(f.id).or_default().allow(tick, SUPPORT_WINDOW as usize, cap, amount.max(0) as usize);
        if n > 0 {
            out.push(Cmd::Shield { target: f.id, amount: n as i64, ticks });
        }
    }
    fn buff(&self, out: &mut Vec<Cmd>, target: usize, dim: usize, ticks: i64, mods: Mods) {
        if ticks > 0 {
            out.push(Cmd::Buff { target, name: format!("vsd_{}", DIMS[dim].id), ticks: ticks as usize, mods });
        }
    }
}
/// The rank from which a habit label is credited (the Rift Lab labels and the fixtures read these).
pub fn habit_gate(h: &str) -> usize {
    match h {
        "led target" => 1,
        "collapsed a rift before it broke" => GATE_SAFETY,
        "solved the link count" | "implode on a pile" | "rain on a cluster" | "crossfire split" | "domino through the pile" | "bank shot around a wall" => GATE_GEOMETRY,
        "Lifeline set before the fight" | "peeled a diver" | "saved an ally" | "rescued ally" | "phased out of a burst" | "escaped a dive" => GATE_SURVIVAL,
        "Escort on the diver" | "Gate synced with the tank" | "Fling toward the team" | "swapped the target into the team" | "dropped aim for a diver" | "borrowed a far dimension" | "rotated an ally into the fight" => GATE_TEAM,
        "Kidnapped the carry into the team" | "Sinkhole at the chokepoint" | "Seed on the carry's path" | "stitched two worlds" => GATE_PRIORITY,
        "spine refreshed" | "rode strain into Overload" | "planned a 5-step sequence" => GATE_SPINE,
        "planned 2 steps ahead" => 3,
        "planned 3 steps ahead" => 5,
        "planned 4 steps ahead" => 6,
        _ => 0,
    }
}
/// A landing spot: inside the map and out of walls (walked back toward where the fighter stood).
pub fn land(fx: i64, fy: i64, x: i64, y: i64, wall: &dyn Fn(i64, i64) -> bool) -> (i64, i64) {
    let (x, y) = clamp_map(x, y);
    if !wall(x, y) {
        return (x, y);
    }
    let steps = (dist(fx, fy, x, y) / 3000).max(1);
    for i in 0..=steps {
        let (px, py) = (x + (fx - x) * i / steps, y + (fy - y) * i / steps);
        if !wall(px, py) {
            return (px, py);
        }
    }
    (fx, fy)
}

/// The tick being simulated: who is where.
pub struct Tick<'a> {
    pub tick: usize,
    pub me: Fighter,
    pub fighters: &'a [Fighter],
    pub wall: &'a dyn Fn(i64, i64) -> bool,
    pub attack_interval: usize,
    pub stunned: bool,
}
impl<'a> Tick<'a> {
    pub fn get(&self, id: usize) -> Option<&Fighter> {
        self.fighters.iter().find(|f| f.id == id && f.hp > 0)
    }
    pub fn enemies(&self) -> impl Iterator<Item = &Fighter> {
        let team = self.me.team;
        self.fighters.iter().filter(move |f| f.team != team && f.hp > 0)
    }
    pub fn allies(&self) -> impl Iterator<Item = &Fighter> {
        let (team, me) = (self.me.team, self.me.id);
        self.fighters.iter().filter(move |f| f.team == team && f.id != me && f.hp > 0)
    }
    pub fn friends(&self) -> impl Iterator<Item = &Fighter> {
        let team = self.me.team;
        self.fighters.iter().filter(move |f| f.team == team && f.hp > 0)
    }
    pub fn danger(&self, x: i64, y: i64, r: i64) -> usize {
        self.enemies().filter(|e| dist2(e.x, e.y, x, y) <= (r as i128) * (r as i128)).count()
    }
}
/// Sixteen unit directions (x10000) for seeded shoves.
const DIRS: [(i64, i64); 16] = [
    (10000, 0), (9239, 3827), (7071, 7071), (3827, 9239), (0, 10000), (-3827, 9239), (-7071, 7071), (-9239, 3827),
    (-10000, 0), (-9239, -3827), (-7071, -7071), (-3827, -9239), (0, -10000), (3827, -9239), (7071, -7071), (9239, -3827),
];
pub fn heading8(dx: i64, dy: i64) -> usize {
    if dx == 0 && dy == 0 {
        return 0;
    }
    const D8: [(i64, i64); 8] = [(10000, 0), (7071, 7071), (0, 10000), (-7071, 7071), (-10000, 0), (-7071, -7071), (0, -10000), (7071, -7071)];
    D8.iter()
        .enumerate()
        .max_by_key(|(_, (x, y))| *x as i128 * dx as i128 + *y as i128 * dy as i128)
        .map_or(0, |(i, _)| i)
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Src {
    Zone,
    Imbue,
    Breach,
}

impl Core {
    // ---------------------------------------------------------------- dimensions
    /// Applies one dimension to one fighter. `s` is the strength in percent; `c` is the rift centre and radius.
    #[allow(clippy::too_many_arguments)]
    fn apply_dim(&mut self, out: &mut Vec<Cmd>, t: &Tick, dim: usize, f: &Fighter, s: i64, c: (i64, i64, i64), src: Src) {
        let d = DIMS[dim];
        let p = d.prim;
        let side = PRIM_SIDE[p as usize];
        let enemy = f.team != t.me.team;
        let me = f.id == t.me.id;
        if src != Src::Breach {
            let ok = match side {
                b'E' => enemy,
                b'A' => !enemy,
                b'S' => me,
                b'X' => true,
                _ => false,
            };
            if !ok {
                return;
            }
        }
        let tick = t.tick;
        let sc = |v: i64| v * s / 100;
        let key = 1000 + p as u32;
        let mut m = Mods::default();
        match p {
            P_BANISH => {
                if self.cooldown_ok(key, f.id, tick) && self.control(out, f, None, sc(d.a).max(20), tick) {
                    self.set_cooldown(key, f.id, tick + 480);
                }
            }
            P_STUN | P_ROOT | P_SILENCE | P_DISARM | P_STATICBURST => {
                let kind = match p {
                    P_STUN | P_STATICBURST => Cc::Stun,
                    P_ROOT => Cc::Root,
                    P_SILENCE => Cc::Silence,
                    _ => Cc::Disarm,
                };
                let ticks = if p == P_STATICBURST { self.random(d.a as u64 + 1) as i64 } else { sc(d.a).max(10) };
                if self.cooldown_ok(key, f.id, tick) && self.control(out, f, Some(kind), ticks, tick) {
                    self.set_cooldown(key, f.id, tick + 240);
                }
            }
            P_SLOW => {
                m.move_speed = -(sc(d.a) as i32);
                m.attack_speed = -(sc(d.b) as i32);
                self.buff(out, f.id, dim, d.c, m);
            }
            P_AMPLIFY => {
                m.amplify = sc(d.a) as i32;
                self.buff(out, f.id, dim, d.b, m);
            }
            P_DOT | P_DRAIN | P_ENTROPY => {
                let per_mille = if p == P_ENTROPY { self.random(d.a as u64 + 1) as i64 } else { d.a };
                let per_pulse = f.max_hp * sc(per_mille) / 2000;
                if src == Src::Imbue && d.b > 0 {
                    self.dots.retain(|x| !(x.target == f.id && x.drain == (p == P_DRAIN)));
                    self.dots.push(Dot { target: f.id, per_pulse, until: tick + d.b as usize, drain: p == P_DRAIN });
                } else {
                    let n = self.emit(out, f, per_pulse, Dmg::Magic, tick);
                    if p == P_DRAIN && n > 0 {
                        out.push(Cmd::Heal { target: t.me.id, amount: n });
                    }
                }
            }
            P_SHATTER => {
                if self.cooldown_ok(key, f.id, tick) {
                    out.push(Cmd::ClearShield { target: f.id });
                    self.set_cooldown(key, f.id, tick + 240);
                }
            }
            P_STATDOWN | P_STATUP => {
                let v = sc(d.b) as i32 * if p == P_STATDOWN { -1 } else { 1 };
                match d.a {
                    0 => m.attack = v,
                    1 => m.defence = v,
                    2 => m.mr = v,
                    3 => m.ap = v,
                    _ => {
                        if p == P_STATDOWN {
                            m.regen = -(d.b as i32 / 10).max(1)
                        } else {
                            m.crit = v
                        }
                    }
                }
                self.buff(out, f.id, dim, d.c, m);
            }
            P_REFLECT | P_PULL | P_KNOCK | P_SHOVE | P_STORM | P_GRAB | P_REWIND => {
                if !self.cooldown_ok(key, f.id, tick) {
                    return;
                }
                let (cx, cy) = (c.0, c.1);
                let to = match p {
                    P_REFLECT => (2 * cx - f.x, 2 * cy - f.y),
                    P_PULL => lerp(f.x, f.y, cx, cy, sc(d.a).min(100)),
                    P_KNOCK => {
                        let (dx, dy) = if (f.x, f.y) == (cx, cy) { (1, 0) } else { (f.x - cx, f.y - cy) };
                        along(f.x, f.y, dx, dy, sc(d.a))
                    }
                    P_SHOVE | P_STORM => {
                        let mut v = (0i64, 0i64);
                        for _ in 0..if p == P_STORM { 2 } else { 1 } {
                            let (ux, uy) = DIRS[self.random(16) as usize];
                            v.0 += ux * sc(d.a) / 10000;
                            v.1 += uy * sc(d.a) / 10000;
                        }
                        (f.x + v.0, f.y + v.1)
                    }
                    P_GRAB => along(f.x, f.y, t.me.x - f.x, t.me.y - f.y, sc(d.a).min(dist(f.x, f.y, t.me.x, t.me.y))),
                    _ => {
                        let Some(h) = self.history.get(&f.id) else { return };
                        let back = (d.a as usize * s.clamp(30, 100) as usize / 100 / 6).max(1);
                        if h.len() <= back {
                            return;
                        }
                        h[h.len() - 1 - back]
                    }
                };
                let enemy_move = enemy || src == Src::Breach;
                if self.displace(out, f, to.0, to.1, tick, t.wall, enemy_move) {
                    self.set_cooldown(key, f.id, tick + if p == P_REWIND { 480 } else { 240 });
                }
            }
            P_CDUP | P_CDDOWN => {
                m.cooldown = sc(d.a) as i32 * if p == P_CDUP { 1 } else { -1 };
                self.buff(out, f.id, dim, d.b, m);
            }
            P_HEALCUT => {
                m.heal_reduce = sc(d.a) as i32;
                self.buff(out, f.id, dim, d.b, m);
            }
            P_SHELTER => {
                m.reduce = sc(d.a) as i32;
                self.buff(out, f.id, dim, d.b, m);
            }
            P_HEAL => {
                if d.b == 0 {
                    let k = 3000 + dim as u32;
                    if self.cooldown_ok(k, f.id, tick) {
                        out.push(Cmd::Heal { target: f.id, amount: f.max_hp * sc(d.a) / 1000 });
                        self.set_cooldown(k, f.id, tick + 600);
                    }
                } else {
                    out.push(Cmd::Heal { target: f.id, amount: f.max_hp * sc(d.a) / 2000 });
                }
            }
            P_SHIELD => {
                if self.cooldown_ok(key, f.id, tick) {
                    self.shield(out, f, f.max_hp * sc(d.a) / 1000, d.b as usize, tick);
                    self.set_cooldown(key, f.id, tick + 180);
                }
            }
            P_UNDYING | P_CCIMMUNE => {
                if self.cooldown_ok(key, f.id, tick) {
                    if p == P_UNDYING {
                        m.undying = true;
                    } else {
                        m.cc_immune = true;
                    }
                    self.buff(out, f.id, dim, sc(d.a).max(10), m);
                    self.set_cooldown(key, f.id, tick + 1200);
                }
            }
            P_HASTE => {
                m.move_speed = sc(d.a) as i32;
                m.attack_speed = sc(d.b) as i32;
                self.buff(out, f.id, dim, d.c, m);
            }
            P_RANGE => {
                m.range = sc(d.a) as i32;
                self.buff(out, f.id, dim, d.b, m);
            }
            P_GHOST => {
                m.ignore_wall = true;
                self.buff(out, f.id, dim, d.a, m);
            }
            P_CLEANSE => {
                if self.cooldown_ok(key, f.id, tick) {
                    out.push(Cmd::ClearCc { target: f.id });
                    self.set_cooldown(key, f.id, tick + 240);
                }
            }
            P_ALLYINVIS | P_SELFINVIS => {
                if self.cooldown_ok(key, f.id, tick) {
                    out.push(Cmd::Invisible { target: f.id, ticks: sc(d.a).max(10) as usize });
                    self.set_cooldown(key, f.id, tick + if p == P_ALLYINVIS { 600 } else { 30 });
                }
            }
            P_BADR => {
                m.ba_reduce = sc(d.a) as i32;
                self.buff(out, f.id, dim, d.b, m);
            }
            P_REFLECTDMG => {
                m.reflect = sc(d.a) as i32;
                self.buff(out, f.id, dim, d.b, m);
            }
            P_VAMP => {
                m.vamp = sc(d.a) as i32;
                self.buff(out, t.me.id, dim, d.b, m);
            }
            P_TITHE => {
                if enemy && self.cooldown_ok(key, f.id, tick) {
                    out.push(Cmd::ClearShield { target: f.id });
                    self.set_cooldown(key, f.id, tick + 240);
                    let me = t.me.clone();
                    self.shield(out, &me, me.max_hp * 5 / 100, 180, tick);
                }
            }
            P_COINFLIP => {
                if self.random(2) == 0 {
                    if enemy {
                        m.amplify = sc(d.a) as i32;
                        self.buff(out, f.id, dim, 40, m);
                    }
                } else if !enemy {
                    m.reduce = sc(d.a) as i32;
                    self.buff(out, f.id, dim, 40, m);
                }
            }
            P_DICE => {
                let fam = d.family;
                let mut other = self.random(90) as usize;
                if other / 10 >= fam {
                    other += 10;
                }
                if DIMS[other].prim != P_DICE {
                    self.apply_dim(out, t, other, f, s, c, src);
                }
            }
            _ => {}
        }
    }

    // ---------------------------------------------------------------- rifts
    /// Opens a rift: charges are already spent; it opens after the travel delay.
    fn open(&mut self, out: &mut Vec<Cmd>, t: &Tick, spec: &RiftSpec, tech: usize, subject: Option<usize>) -> u32 {
        let rt = RIFTS[spec.kind];
        let uid = self.next_uid;
        self.next_uid += 1;
        let d = dist(t.me.x, t.me.y, spec.x, spec.y);
        let delay = if spec.kind == R_ANCHOR && spec.host.is_some() { open_delay(0, 100) } else { open_delay(d, rt.delay_pct) };
        let dim = self.atlas.dim_at(spec.x, spec.y, t.tick + delay);
        let chain = if spec.chain {
            Some(spec.order.unwrap_or_else(|| {
                self.chain_seq += 1;
                self.chain_seq
            }))
        } else {
            None
        };
        let opens = t.tick + delay;
        self.rifts.push(Rift {
            uid,
            kind: spec.kind,
            tech,
            dim,
            dim2: None,
            borrowed: None,
            x: spec.x,
            y: spec.y,
            ex: spec.ex,
            ey: spec.ey,
            radius: rt.radius,
            opens,
            expires: opens + rt.life,
            host: spec.host,
            peer: None,
            subject,
            chain,
            entrance: spec.entrance,
            opened: false,
            fired: false,
            dwell: BTreeMap::new(),
            touched: BTreeSet::new(),
            sides: BTreeMap::new(),
            inside: BTreeSet::new(),
            done: false,
        });
        self.stats.rifts_opened += 1;
        let look = self.look();
        self.fx(out, format!("open_l{look}"), spec.x, spec.y, None, delay);
        uid
    }
    fn close(&mut self, out: &mut Vec<Cmd>, uid: u32, why: &str) {
        let look = self.look();
        let Some(r) = self.rifts.iter_mut().find(|r| r.uid == uid && !r.done) else { return };
        r.done = true;
        let (x, y) = (r.x, r.y);
        self.fx(out, format!("{why}_l{look}"), x, y, None, 24);
    }
    fn implode(&mut self, out: &mut Vec<Cmd>, t: &Tick, uid: u32) {
        let Some(r) = self.rift(uid).cloned() else { return };
        let reach = r.radius * 3 / 2;
        let amount = IMPLODE_BASE + t.me.attack * IMPLODE_AD / 100;
        let mut hits = 0;
        let targets: Vec<Fighter> = t.enemies().filter(|e| r.reach(e.x, e.y) <= reach).cloned().collect();
        for e in &targets {
            hits += 1;
            self.displace(out, e, (e.x + r.x) / 2, (e.y + r.y) / 2, t.tick, t.wall, true);
            self.emit(out, e, amount, Dmg::Phys, t.tick);
            for dim in r.dims(t.tick) {
                self.apply_dim(out, t, dim, e, 200, (r.x, r.y, r.radius), Src::Imbue);
            }
        }
        self.stats.implodes += 1;
        self.stats.implode_hits += hits;
        // closing a rift on purpose releases part of the strain it cost
        self.add_strain(-STRAIN_TEAR / 2);
        let tier = self.tier4();
        self.fx(out, format!("tech_implode_t{tier}"), r.x, r.y, None, 30);
        self.close(out, uid, "close");
    }
    /// Art tier 0..3, or 4 for the #1 athlete.
    pub fn tier4(&self) -> usize {
        if self.look() == 8 {
            4
        } else {
            art_tier(self.rank)
        }
    }
    fn breach(&mut self, out: &mut Vec<Cmd>, t: &Tick) {
        let dim = self.atlas.dim_at(t.me.x, t.me.y, t.tick);
        self.stats.breaches += 1;
        out.push(Cmd::Log(format!("tick {} breach strain {} dimension {} ({})", t.tick, self.strain, DIMS[dim].id, DIMS[dim].name)));
        let hit: Vec<Fighter> = t
            .fighters
            .iter()
            .filter(|f| f.hp > 0 && dist2(f.x, f.y, t.me.x, t.me.y) <= (BREACH_RADIUS as i128).pow(2))
            .cloned()
            .collect();
        for f in &hit {
            self.apply_dim(out, t, dim, f, 100, (t.me.x, t.me.y, BREACH_RADIUS), Src::Breach);
        }
        out.push(Cmd::LoseHp { target: t.me.id, amount: t.me.max_hp * BREACH_HP_PCT / 100 });
        out.push(Cmd::Cc { target: t.me.id, kind: Cc::Stun, ticks: BREACH_STAGGER as usize });
        let tier = self.tier4();
        self.fx(out, format!("breach_t{tier}"), t.me.x, t.me.y, Some(t.me.id), 36);
        let uids: Vec<u32> = self.rifts.iter().filter(|r| !r.done).map(|r| r.uid).collect();
        for uid in uids {
            self.close(out, uid, "break");
        }
        self.packets.clear();
        self.strain = BREACH_RESET;
        self.aiming = false;
    }
    fn add_strain(&mut self, n: i64) {
        self.strain = (self.strain + n).max(0);
    }
}

impl Core {
    /// One simulation tick. Returns the commands to apply, in order.
    pub fn step(&mut self, input: &Input) -> Vec<Cmd> {
        let mut out = Vec::new();
        if self.last_tick == Some(input.tick) {
            return out;
        }
        self.last_tick = Some(input.tick);
        self.fx_left = FX_PER_UPDATE as usize;
        let Some(me) = input.fighters.iter().find(|f| f.id == input.me).cloned() else { return out };
        let t = Tick { tick: input.tick, me, fighters: input.fighters, wall: input.wall, attack_interval: input.attack_interval.max(1), stunned: input.stunned };
        let tick = t.tick;
        self.charges.advance(tick);
        self.observe(&t);
        // strain bleeds off while at most one rift is open
        if self.open_count() <= 1 && tick.is_multiple_of(STRAIN_DECAY_TICKS as usize) {
            self.add_strain(-1);
        }
        self.cooldowns.retain(|_, until| *until > tick);
        self.step_rifts(&mut out, &t);
        self.step_techniques(&mut out, &t);
        self.step_zones(&mut out, &t);
        self.step_breaking(&mut out, &t);
        self.step_later(&mut out, &t);
        self.step_dots(&mut out, &t);
        self.step_chain(&mut out, &t);
        self.step_packets(&mut out, &t);
        if self.strain >= BREACH_AT {
            self.breach(&mut out, &t);
        }
        if !self.hand_placed && !self.phased && !t.stunned && tick >= self.last_think + self.table(&THINK_TICKS) as usize {
            self.last_think = tick;
            self.think(&mut out, &t);
        }
        self.render(&mut out, &t);
        self.rifts.retain(|r| !r.done);
        out
    }
    /// Positions every 6 ticks: forecasting velocity and Rewind history.
    fn observe(&mut self, t: &Tick) {
        if !t.tick.is_multiple_of(6) {
            return;
        }
        for f in t.fighters.iter().filter(|f| f.hp > 0) {
            let h = self.history.entry(f.id).or_default();
            if let Some(&(px, py)) = h.back() {
                self.velocity.insert(f.id, ((f.x - px) / 6, (f.y - py) / 6));
            }
            h.push_back((f.x, f.y));
            while h.len() > (HISTORY as usize / 6) + 1 {
                h.pop_front();
            }
        }
        let alive: BTreeSet<usize> = t.fighters.iter().filter(|f| f.hp > 0).map(|f| f.id).collect();
        self.history.retain(|id, _| alive.contains(id));
        self.velocity.retain(|id, _| alive.contains(id));
    }
    pub fn forecast(&self, f: &Fighter, ticks: i64) -> (i64, i64) {
        let (vx, vy) = self.velocity.get(&f.id).copied().unwrap_or((0, 0));
        clamp_map(f.x + (vx * ticks).clamp(-60000, 60000), f.y + (vy * ticks).clamp(-60000, 60000))
    }
    fn step_rifts(&mut self, out: &mut Vec<Cmd>, t: &Tick) {
        let tick = t.tick;
        for i in 0..self.rifts.len() {
            if self.rifts[i].done {
                continue;
            }
            // anchors ride their host; a dead or missing host ends them
            if let Some(h) = self.rifts[i].host {
                match t.get(h) {
                    Some(f) => {
                        self.rifts[i].x = f.x;
                        self.rifts[i].y = f.y;
                    }
                    None => {
                        let uid = self.rifts[i].uid;
                        self.close(out, uid, "close");
                        continue;
                    }
                }
            }
            if !self.rifts[i].opened && tick >= self.rifts[i].opens {
                self.rifts[i].opened = true;
                let r = &self.rifts[i];
                let touching = t.enemies().any(|e| r.reach(e.x, e.y) <= r.radius * 2);
                if touching {
                    self.stats.rifts_touching += 1;
                }
            }
            if self.rifts[i].opened && self.rifts[i].kind == R_WANDER {
                let r = &mut self.rifts[i];
                let (nx, ny) = if dist(r.x, r.y, r.ex, r.ey) <= WANDER_SPEED { (r.ex, r.ey) } else { along(r.x, r.y, r.ex - r.x, r.ey - r.y, WANDER_SPEED) };
                r.x = nx;
                r.y = ny;
            }
            if tick >= self.rifts[i].expires {
                let r = self.rifts[i].clone();
                if r.kind == R_MAW {
                    self.implode(out, t, r.uid);
                } else if r.tech == T_HOOK {
                    self.reel(out, t, &r);
                } else if r.tech == T_FLING || r.tech == T_SWEEP {
                    self.drop_carried(out, t, &r);
                } else {
                    self.close(out, r.uid, "close");
                }
            }
        }
        // Borrowed dimensions run out on their own
        for r in self.rifts.iter_mut() {
            if r.borrowed.is_some_and(|(_, until)| tick >= until) {
                r.borrowed = None;
            }
        }
    }
    fn reel(&mut self, out: &mut Vec<Cmd>, t: &Tick, r: &Rift) {
        if let Some(target) = r.host.and_then(|h| t.get(h)).cloned() {
            let exit = r.peer.and_then(|p| self.rift(p)).map(|p| (p.x, p.y)).unwrap_or((t.me.x, t.me.y));
            if self.cooldown_ok(2000 + T_HOOK as u32, target.id, t.tick) && self.displace(out, &target, exit.0, exit.1, t.tick, t.wall, true) {
                self.control(out, &target, None, 20, t.tick);
                self.set_cooldown(2000 + T_HOOK as u32, target.id, t.tick + TECHS[T_HOOK].cooldown);
                self.score_kidnap(t, exit, "hook");
                let tier = self.tier4();
                self.fx(out, format!("tech_hook_t{tier}"), exit.0, exit.1, None, 24);
            }
        }
        self.close(out, r.uid, "close");
    }
    fn drop_carried(&mut self, out: &mut Vec<Cmd>, t: &Tick, r: &Rift) {
        // carried fighters reappear at the path's end (their Spit is already scheduled)
        let _ = t;
        self.close(out, r.uid, "close");
    }
    /// Kidnap-style landings count as useful when they deliver an enemy into Vesper's team, not onto Vesper.
    fn score_kidnap(&mut self, t: &Tick, at: (i64, i64), _what: &str) {
        self.stats.kidnaps += 1;
        let allies = t.allies().filter(|a| dist2(a.x, a.y, at.0, at.1) <= (45000i128).pow(2)).count();
        if allies >= 1 && dist(t.me.x, t.me.y, at.0, at.1) > 25000 {
            self.stats.useful_kidnaps += 1;
        }
    }
    /// A one-shot technique has fired: its entrance seals at once and its exit lingers one more second.
    fn fire(&mut self, uid: u32) {
        let mut peer = None;
        if let Some(r) = self.rift_mut(uid) {
            r.fired = true;
            r.expires = r.expires.min(r.opens.max(1));
            peer = r.peer;
        }
        if let Some(p) = peer.and_then(|p| self.rift_mut(p)) {
            p.expires = p.expires.min(p.opens.max(p.expires.saturating_sub(RIFTS[p.kind].life)) + 60);
        }
    }
    fn exit_of(&self, r: &Rift) -> Option<(i64, i64)> {
        r.peer.and_then(|p| self.rift(p)).filter(|p| p.live()).map(|p| (p.x, p.y))
    }
    fn step_techniques(&mut self, out: &mut Vec<Cmd>, t: &Tick) {
        let tick = t.tick;
        let uids: Vec<u32> = self.rifts.iter().filter(|r| r.live()).map(|r| r.uid).collect();
        for uid in uids {
            let Some(r) = self.rift(uid).cloned() else { continue };
            let just_opened = tick == r.opens;
            let tier = self.tier4();
            match r.tech {
                T_GATE | T_SKYDROP | T_ROTATION if r.entrance => {
                    let Some(exit) = self.exit_of(&r) else { continue };
                    let movers: Vec<Fighter> = t.allies().filter(|a| r.covers(a.x, a.y)).cloned().collect();
                    for a in movers {
                        let key = 2000 + T_GATE as u32;
                        if !self.cooldown_ok(key, a.id, tick) {
                            continue;
                        }
                        self.set_cooldown(key, a.id, tick + TECHS[T_GATE].cooldown);
                        let (x, y) = land(a.x, a.y, exit.0, exit.1, t.wall);
                        out.push(Cmd::SetPos { target: a.id, x, y });
                        self.stats.gates += 1;
                        let idle = t.danger(a.x, a.y, 50000) == 0;
                        let foes = t.danger(x, y, 40000);
                        let friends = t.friends().filter(|f| f.id != a.id && dist2(f.x, f.y, x, y) <= (70000i128).pow(2)).count();
                        if idle && foes >= 1 && friends >= 1 {
                            self.stats.engaged_gates += 1;
                        } else if foes >= 3 {
                            self.stats.lonely_gates += 1;
                        }
                        self.fx(out, format!("tech_gate_t{tier}"), x, y, None, 24);
                        if r.tech == T_SKYDROP {
                            self.later.push(Later::Land { at: tick + 12, x, y, ally: a.id });
                        }
                    }
                }
                T_STEP if r.entrance && !r.fired => {
                    let Some(exit) = self.exit_of(&r) else { continue };
                    self.fire(uid);
                    if dist(t.me.x, t.me.y, r.x, r.y) <= STEP_REACH && tick >= self.self_move_until {
                        self.self_move_until = tick + SELF_MOVE_CD as usize;
                        let (x, y) = land(t.me.x, t.me.y, exit.0, exit.1, t.wall);
                        out.push(Cmd::SetPos { target: t.me.id, x, y });
                        self.fx(out, format!("tech_step_t{tier}"), x, y, None, 18);
                        if t.danger(x, y, 35000) >= 3 {
                            self.stats.risky_moves += 1;
                            self.risky_until = tick + 180;
                        }
                    }
                }
                T_PHASE if r.entrance && !r.fired => {
                    let Some(exit) = self.exit_of(&r) else { continue };
                    self.fire(uid);
                    if dist(t.me.x, t.me.y, r.x, r.y) <= STEP_REACH && tick >= self.self_move_until {
                        self.self_move_until = tick + SELF_MOVE_CD as usize;
                        out.push(Cmd::Banish { target: t.me.id, ticks: 45 });
                        self.phased = true;
                        self.aiming = false;
                        self.later.push(Later::Unphase { at: tick + 45, x: exit.0, y: exit.1 });
                        self.fx(out, format!("tech_phase_t{tier}"), r.x, r.y, None, 30);
                    }
                }
                T_KIDNAP if r.entrance && !r.fired => {
                    let Some(exit) = self.exit_of(&r) else { continue };
                    self.fire(uid);
                    let Some(v) = r.subject.and_then(|s| t.get(s)).cloned() else { continue };
                    let key = 2000 + T_KIDNAP as u32;
                    if r.reach(v.x, v.y) <= r.radius * 13 / 10 && self.cooldown_ok(key, v.id, tick) && self.displace(out, &v, exit.0, exit.1, tick, t.wall, true) {
                        self.set_cooldown(key, v.id, tick + TECHS[T_KIDNAP].cooldown);
                        self.control(out, &v, None, 30, tick);
                        self.score_kidnap(t, exit, "kidnap");
                        self.fx(out, format!("tech_kidnap_t{tier}"), r.x, r.y, None, 24);
                        self.fx(out, format!("tech_kidnap_t{tier}"), exit.0, exit.1, None, 24);
                    }
                }
                T_SINKTRAP => {
                    let inside: Vec<Fighter> = t.enemies().filter(|e| r.covers(e.x, e.y)).cloned().collect();
                    let key = 2000 + T_SINKTRAP as u32;
                    for e in inside {
                        let n = {
                            let rr = self.rift_mut(uid).unwrap();
                            let n = rr.dwell.entry(e.id + 1_000_000).or_default();
                            *n += 1;
                            *n
                        };
                        if tick.is_multiple_of(PULSE as usize) {
                            self.buff(out, e.id, 0, 40, Mods { move_speed: -30, ..Mods::default() });
                        }
                        if n >= 60 && self.cooldown_ok(key, e.id, tick) {
                            self.set_cooldown(key, e.id, tick + TECHS[T_SINKTRAP].cooldown);
                            self.stats.trap_catches += 1;
                            let exit = self.exit_of(&r);
                            if let Some(exit) = exit {
                                if self.displace(out, &e, exit.0, exit.1, tick, t.wall, true) {
                                    self.score_kidnap(t, exit, "sinkhole");
                                }
                            }
                            self.control(out, &e, None, 30, tick);
                            self.fx(out, format!("tech_sinktrap_t{tier}"), e.x, e.y, None, 24);
                        }
                    }
                }
                T_SWAP if r.entrance && !r.fired => {
                    let Some(peer) = r.peer.and_then(|p| self.rift(p)).filter(|p| p.live()).cloned() else { continue };
                    self.fire(uid);
                    let near = |x: i64, y: i64, rad: i64| t.fighters.iter().filter(|f| f.hp > 0 && dist(f.x, f.y, x, y) <= rad).min_by_key(|f| (dist2(f.x, f.y, x, y), f.id)).cloned();
                    let (Some(a), Some(b)) = (near(r.x, r.y, r.radius * 2), near(peer.x, peer.y, peer.radius * 2)) else { continue };
                    if a.id == b.id || (a.team == t.me.team && b.team == t.me.team) {
                        continue;
                    }
                    let key = 2000 + T_SWAP as u32;
                    let enemies_ok = [&a, &b].iter().all(|f| f.team == t.me.team || (!f.cc_immune && self.cooldown_ok(key, f.id, tick)));
                    if !enemies_ok {
                        continue;
                    }
                    for f in [&a, &b] {
                        if f.team != t.me.team {
                            self.set_cooldown(key, f.id, tick + TECHS[T_SWAP].cooldown);
                        }
                    }
                    let (ax, ay) = land(a.x, a.y, b.x, b.y, t.wall);
                    let (bx, by) = land(b.x, b.y, a.x, a.y, t.wall);
                    out.push(Cmd::SetPos { target: a.id, x: ax, y: ay });
                    out.push(Cmd::SetPos { target: b.id, x: bx, y: by });
                    for (f, x, y) in [(&a, ax, ay), (&b, bx, by)] {
                        self.displaced.insert(f.id, tick);
                        if f.id == t.me.id && t.danger(x, y, 35000) >= 3 {
                            self.stats.risky_moves += 1;
                            self.risky_until = tick + 180;
                        }
                        if f.team != t.me.team {
                            self.score_kidnap(t, (x, y), "swap");
                        }
                    }
                    self.fx(out, format!("tech_swap_t{tier}"), r.x, r.y, None, 24);
                    self.fx(out, format!("tech_swap_t{tier}"), peer.x, peer.y, None, 24);
                }
                T_FLING => {
                    let key = 2000 + T_FLING as u32;
                    let caught: Vec<Fighter> = t.enemies().filter(|e| r.covers(e.x, e.y) && !r.touched.contains(&e.id)).cloned().collect();
                    for e in caught {
                        if let Some(rr) = self.rift_mut(uid) {
                            rr.touched.insert(e.id);
                        }
                        if e.cc_immune || !self.cooldown_ok(key, e.id, tick) {
                            continue;
                        }
                        let travel = (dist(r.x, r.y, r.ex, r.ey) / WANDER_SPEED).clamp(10, 60) as usize;
                        if self.control(out, &e, None, travel as i64, tick) {
                            self.set_cooldown(key, e.id, tick + TECHS[T_FLING].cooldown);
                            self.later.push(Later::Spit { at: tick + travel, target: e.id, x: r.ex, y: r.ey, ally: false });
                            self.fx(out, format!("tech_fling_t{tier}"), e.x, e.y, None, 24);
                        }
                    }
                }
                T_RESCUE if r.entrance && !r.fired => {
                    let Some(exit) = self.exit_of(&r) else { continue };
                    self.fire(uid);
                    if let Some(a) = r.subject.and_then(|s| t.get(s)).cloned() {
                        let key = 2000 + T_RESCUE as u32;
                        if r.covers(a.x, a.y) && self.cooldown_ok(key, a.id, tick) {
                            self.set_cooldown(key, a.id, tick + TECHS[T_RESCUE].cooldown);
                            let (x, y) = land(a.x, a.y, exit.0, exit.1, t.wall);
                            out.push(Cmd::SetPos { target: a.id, x, y });
                            self.fx(out, format!("tech_rescue_t{tier}"), x, y, None, 24);
                            self.habit("rescued ally");
                        }
                    }
                }
                T_POCKET if just_opened => {
                    if let Some(a) = r.subject.and_then(|s| t.get(s)).cloned() {
                        let key = 2000 + T_POCKET as u32;
                        if r.covers(a.x, a.y) && self.cooldown_ok(key, a.id, tick) {
                            self.set_cooldown(key, a.id, tick + TECHS[T_POCKET].cooldown);
                            out.push(Cmd::Banish { target: a.id, ticks: 60 });
                            self.fx(out, format!("tech_pocket_t{tier}"), a.x, a.y, None, 30);
                        }
                    }
                }
                T_LIFELINE => {
                    if t.me.hp * 4 < t.me.max_hp && tick >= self.self_move_until {
                        self.self_move_until = tick + SELF_MOVE_CD as usize;
                        let (x, y) = land(t.me.x, t.me.y, r.x, r.y, t.wall);
                        out.push(Cmd::SetPos { target: t.me.id, x, y });
                        self.fx(out, format!("tech_lifeline_t{tier}"), x, y, None, 24);
                        self.close(out, uid, "close");
                        self.aiming = false;
                    }
                }
                T_SEEDTRAP if !r.hidden(tick) => {
                    let victim = t.enemies().filter(|e| r.covers(e.x, e.y)).min_by_key(|e| (dist2(e.x, e.y, r.x, r.y), e.id)).cloned();
                    if let Some(v) = victim {
                        let key = 2000 + T_SEEDTRAP as u32;
                        if self.cooldown_ok(key, v.id, tick) {
                            self.set_cooldown(key, v.id, tick + TECHS[T_SEEDTRAP].cooldown);
                            self.stats.trap_catches += 1;
                            for dim in r.dims(tick) {
                                self.apply_dim(out, t, dim, &v, 200, (r.x, r.y, r.radius), Src::Imbue);
                            }
                            match self.exit_of(&r) {
                                Some(exit) => {
                                    if self.displace(out, &v, exit.0, exit.1, tick, t.wall, true) {
                                        self.score_kidnap(t, exit, "seed");
                                    }
                                    self.control(out, &v, None, 30, tick);
                                }
                                None => {
                                    self.control(out, &v, None, 45, tick);
                                }
                            }
                            self.fx(out, format!("tech_seedtrap_t{tier}"), r.x, r.y, None, 30);
                            self.close(out, uid, "close");
                        }
                    }
                }
                T_RAIN if just_opened => {
                    let amount = 25 + t.me.attack * 25 / 100;
                    for k in 0..6usize {
                        let (ux, uy) = DIRS[self.random(16) as usize];
                        let reach = self.random(r.radius as u64) as i64;
                        self.later.push(Later::Drop { at: tick + 4 + k * 10, x: r.x + ux * reach / 10000, y: r.y + uy * reach / 10000, amount });
                    }
                }
                T_VOLLEY if just_opened => {
                    let amount = 20 + t.me.attack * 30 / 100;
                    for k in 0..5i64 {
                        let (px, py) = (r.x + (r.ex - r.x) * k / 4, r.y + (r.ey - r.y) * k / 4);
                        if let Some(e) = t.enemies().filter(|e| dist(e.x, e.y, px, py) <= 40000).min_by_key(|e| (dist2(e.x, e.y, px, py), e.id)).cloned() {
                            self.emit(out, &e, amount, Dmg::Phys, tick);
                            let dir = heading8(e.x - px, e.y - py);
                            self.fx(out, format!("packet_l{}_a{dir}", self.look()), px, py, None, 8);
                        }
                    }
                }
                T_IMPLODE | T_DOMINO => {}
                _ => {}
            }
            // shape behaviours: Slash curtains and Mirror Panes throw crossers back
            if matches!(r.kind, R_SLASH | R_MIRROR) || (r.tech == T_CURTAIN && r.peer.is_some()) {
                self.crossing(out, t, uid);
            }
            // a Maw swallows everyone inside the moment it opens
            if r.kind == R_MAW && just_opened && r.tech != T_POCKET {
                let inside: Vec<Fighter> = t.enemies().filter(|e| r.covers(e.x, e.y)).cloned().collect();
                for e in inside {
                    self.control(out, &e, None, 45, tick);
                }
                self.fx(out, format!("tech_maw_t{tier}"), r.x, r.y, None, 30);
            }
        }
    }
    /// Enemies crossing a curtain line (a Slash, a Mirror Pane, or two linked curtain Tears) are thrown back.
    fn crossing(&mut self, out: &mut Vec<Cmd>, t: &Tick, uid: u32) {
        let Some(r) = self.rift(uid).cloned() else { return };
        let (ax, ay, bx, by) = if r.tech == T_CURTAIN && r.kind == R_TEAR {
            match r.peer.and_then(|p| self.rift(p)) {
                Some(p) => (r.x, r.y, p.x, p.y),
                None => return,
            }
        } else {
            (r.x, r.y, r.ex, r.ey)
        };
        let reach = r.radius.max(9000) * 3;
        let near: Vec<Fighter> = t.enemies().filter(|e| seg_dist(ax, ay, bx, by, e.x, e.y) <= reach).cloned().collect();
        for e in near {
            let s = side(ax, ay, bx, by, e.x, e.y);
            let prev = self.rift_mut(uid).and_then(|rr| rr.sides.insert(e.id, s));
            if let Some(p) = prev {
                if p != 0 && s != 0 && p != s {
                    // pushed back across, a little past the line
                    let (lx, ly) = (bx - ax, by - ay);
                    let (nx, ny) = (-ly, lx);
                    let push = if r.kind == R_MIRROR { 26000 } else { 14000 };
                    let dir = if p > 0 { 1 } else { -1 };
                    let to = along(e.x, e.y, nx * dir, ny * dir, push);
                    if self.displace(out, &e, to.0, to.1, t.tick, t.wall, true) {
                        if let Some(rr) = self.rift_mut(uid) {
                            rr.sides.insert(e.id, p);
                        }
                        let tier = self.tier4();
                        let tag = if r.kind == R_MIRROR { "reflect" } else { "curtain" };
                        self.fx(out, format!("tech_{tag}_t{tier}"), e.x, e.y, None, 18);
                    }
                }
            }
        }
    }
    fn step_zones(&mut self, out: &mut Vec<Cmd>, t: &Tick) {
        let tick = t.tick;
        let uids: Vec<u32> = self.rifts.iter().filter(|r| r.live() && !r.hidden(tick)).map(|r| r.uid).collect();
        for uid in uids {
            let Some(r) = self.rift(uid).cloned() else { continue };
            let dims = r.dims(tick);
            let centre = (r.x, r.y, r.radius);
            let pulse = (tick + uid as usize).is_multiple_of(PULSE as usize);
            let inside: Vec<Fighter> = t.fighters.iter().filter(|f| f.hp > 0 && r.covers(f.x, f.y)).cloned().collect();
            let inside_ids: BTreeSet<usize> = inside.iter().map(|f| f.id).collect();
            for f in &inside {
                let entering = !r.inside.contains(&f.id);
                for &dim in &dims {
                    let p = DIMS[dim].prim;
                    if PRIM_ONCE[p as usize] {
                        if entering && !r.touched.contains(&(f.id ^ (dim << 24))) {
                            if let Some(rr) = self.rift_mut(uid) {
                                rr.touched.insert(f.id ^ (dim << 24));
                            }
                            if p == P_FOLD {
                                self.fold(out, t, &r, f);
                            } else if !matches!(p, P_SWAPSPACE | P_SCRAMBLE | P_WALL | P_TETHER | P_KALEIDO) && PRIM_SIDE[p as usize] != b'P' {
                                self.apply_dim(out, t, dim, f, 100, centre, Src::Zone);
                            }
                        }
                    } else if pulse && !matches!(p, P_VENT | P_UNSTABLE | P_SIPHON) {
                        self.apply_dim(out, t, dim, f, 100, centre, Src::Zone);
                    }
                }
            }
            // tethered enemies who walk out are snapped back to the edge, once
            for &dim in &dims {
                if DIMS[dim].prim == P_TETHER {
                    let left: Vec<usize> = r.inside.iter().filter(|id| !inside_ids.contains(id)).copied().collect();
                    for id in left {
                        if let Some(e) = t.get(id).filter(|e| e.team != t.me.team).cloned() {
                            if !r.touched.contains(&(id ^ (dim << 24))) {
                                if let Some(rr) = self.rift_mut(uid) {
                                    rr.touched.insert(id ^ (dim << 24));
                                }
                                let edge = along(r.x, r.y, e.x - r.x, e.y - r.y, r.radius * 8 / 10);
                                self.displace(out, &e, edge.0, edge.1, tick, t.wall, true);
                            }
                        }
                    }
                }
            }
            if pulse {
                for &dim in &dims {
                    match DIMS[dim].prim {
                        P_VENT => self.add_strain(-DIMS[dim].a),
                        P_SWAPSPACE | P_SCRAMBLE if !r.touched.contains(&(usize::MAX ^ dim)) => {
                            let enemies_only = DIMS[dim].prim == P_SCRAMBLE;
                            let mut pool: Vec<&Fighter> = inside.iter().filter(|f| !enemies_only || f.team != t.me.team).collect();
                            pool.sort_by_key(|f| (dist2(f.x, f.y, r.x, r.y), f.id));
                            if pool.len() >= 2 && pool.iter().take(2).any(|f| f.team != t.me.team) {
                                let (a, b) = (pool[0].clone(), pool[1].clone());
                                if let Some(rr) = self.rift_mut(uid) {
                                    rr.touched.insert(usize::MAX ^ dim);
                                }
                                let oka = self.displace(out, &a, b.x, b.y, tick, t.wall, a.team != t.me.team);
                                if oka {
                                    self.displace(out, &b, a.x, a.y, tick, t.wall, b.team != t.me.team);
                                }
                            }
                        }
                        P_WALL => {}
                        _ => {}
                    }
                }
            }
            // Glass Wall: a pane across the rift's width
            if dims.iter().any(|d| DIMS[*d].prim == P_WALL) {
                let (ax, ay, bx, by) = (r.x - r.radius, r.y, r.x + r.radius, r.y);
                let near: Vec<Fighter> = t.enemies().filter(|e| seg_dist(ax, ay, bx, by, e.x, e.y) <= r.radius).cloned().collect();
                for e in near {
                    let s = side(ax, ay, bx, by, e.x, e.y);
                    let prev = self.rift_mut(uid).and_then(|rr| rr.sides.insert(e.id + 2_000_000, s));
                    if let Some(p) = prev {
                        if p != 0 && s != 0 && p != s {
                            let to = (e.x, e.y - 12000 * s as i64);
                            if self.displace(out, &e, to.0, to.1, tick, t.wall, true) {
                                if let Some(rr) = self.rift_mut(uid) {
                                    rr.sides.insert(e.id + 2_000_000, p);
                                }
                            }
                        }
                    }
                }
            }
            if let Some(rr) = self.rift_mut(uid) {
                rr.inside = inside_ids;
            }
        }
    }
    /// Folded Map: an ally stepping in comes out of the nearest other live rift.
    fn fold(&mut self, out: &mut Vec<Cmd>, t: &Tick, r: &Rift, f: &Fighter) {
        if f.team != t.me.team {
            return;
        }
        let key = 2000 + T_GATE as u32;
        if !self.cooldown_ok(key, f.id, t.tick) {
            return;
        }
        let exit = self.rifts.iter().filter(|o| o.live() && o.uid != r.uid && !o.hidden(t.tick)).min_by_key(|o| (dist2(o.x, o.y, r.x, r.y), o.uid)).map(|o| (o.x, o.y));
        if let Some((x, y)) = exit {
            self.set_cooldown(key, f.id, t.tick + 180);
            let (x, y) = land(f.x, f.y, x, y, t.wall);
            out.push(Cmd::SetPos { target: f.id, x, y });
        }
    }
    fn step_breaking(&mut self, out: &mut Vec<Cmd>, t: &Tick) {
        let tick = t.tick;
        let mut broken = vec![];
        for r in self.rifts.iter_mut().filter(|r| r.opened && !r.done) {
            if r.hidden(tick) || r.kind == R_MAW {
                continue;
            }
            let here: Vec<usize> = t
                .enemies()
                .filter(|e| Some(e.id) != r.host && r.reach(e.x, e.y) <= BREAK_CORE)
                .map(|e| e.id)
                .collect();
            r.dwell.retain(|id, _| *id >= 1_000_000 || here.contains(id));
            let mut snapped = false;
            for id in here {
                let n = r.dwell.entry(id).or_default();
                *n += 1;
                if *n >= BREAK_TICKS as usize {
                    snapped = true;
                }
            }
            if snapped {
                broken.push(r.uid);
            }
        }
        for uid in broken {
            self.stats.breaks += 1;
            self.add_strain(STRAIN_BREAK);
            out.push(Cmd::Log(format!("tick {tick} rift {uid} broken by an enemy (+{STRAIN_BREAK} strain)")));
            self.close(out, uid, "break");
        }
    }
    fn step_later(&mut self, out: &mut Vec<Cmd>, t: &Tick) {
        let tick = t.tick;
        let (due, rest): (Vec<Later>, Vec<Later>) = std::mem::take(&mut self.later).into_iter().partition(|l| l.at() <= tick);
        self.later = rest;
        let tier = self.tier4();
        for l in due {
            match l {
                Later::Repeat { target, amount, families, .. } => {
                    if let Some(f) = t.get(target).cloned() {
                        self.emit(out, &f, amount, Dmg::Phys, tick);
                        if families.len() == 2 {
                            let _ = families;
                        }
                        self.fx(out, format!("impact_l{}", self.look()), f.x, f.y, None, 12);
                    }
                }
                Later::Domino { uid, .. } => self.implode(out, t, uid),
                Later::Drop { x, y, amount, .. } => {
                    let hit: Vec<Fighter> = t.enemies().filter(|e| dist(e.x, e.y, x, y) <= 12000).cloned().collect();
                    for e in hit {
                        self.emit(out, &e, amount, Dmg::Phys, tick);
                    }
                    self.fx(out, format!("tech_rain_t{tier}"), x, y, None, 12);
                }
                Later::Spit { target, x, y, ally, .. } => {
                    if let Some(f) = t.get(target).cloned() {
                        if ally {
                            let (x, y) = land(f.x, f.y, x, y, t.wall);
                            out.push(Cmd::SetPos { target, x, y });
                        } else if self.displace(out, &f, x, y, tick, t.wall, true) {
                            self.score_kidnap(t, (x, y), "fling");
                        }
                    }
                }
                Later::Unphase { x, y, .. } => {
                    self.phased = false;
                    let (x, y) = land(t.me.x, t.me.y, x, y, t.wall);
                    out.push(Cmd::SetPos { target: t.me.id, x, y });
                    self.fx(out, format!("tech_phase_t{tier}"), x, y, None, 24);
                }
                Later::Land { x, y, ally, .. } => {
                    let hit: Vec<Fighter> = t.enemies().filter(|e| dist(e.x, e.y, x, y) <= 20000).cloned().collect();
                    for e in hit {
                        self.control(out, &e, Some(Cc::Stun), 24, tick);
                    }
                    let _ = ally;
                    self.fx(out, format!("tech_skydrop_t{tier}"), x, y, None, 24);
                }
            }
        }
    }
    fn step_dots(&mut self, out: &mut Vec<Cmd>, t: &Tick) {
        let tick = t.tick;
        if !tick.is_multiple_of(PULSE as usize) {
            self.dots.retain(|d| d.until > tick);
            return;
        }
        let dots = std::mem::take(&mut self.dots);
        for d in &dots {
            if d.until <= tick {
                continue;
            }
            if let Some(f) = t.get(d.target).cloned() {
                let n = self.emit(out, &f, d.per_pulse, Dmg::Magic, tick);
                if d.drain && n > 0 {
                    out.push(Cmd::Heal { target: t.me.id, amount: n });
                }
            }
        }
        self.dots = dots.into_iter().filter(|d| d.until > tick && t.get(d.target).is_some()).collect();
    }
}

impl Core {
    /// Live chain nodes in chain order.
    pub fn chain_nodes(&self, tick: usize) -> Vec<Rift> {
        let mut v: Vec<Rift> = self.rifts.iter().filter(|r| r.live() && r.chain.is_some() && !r.hidden(tick)).cloned().collect();
        v.sort_by_key(|r| r.chain);
        v
    }
    fn step_chain(&mut self, out: &mut Vec<Cmd>, t: &Tick) {
        let tick = t.tick;
        let nodes = self.chain_nodes(tick);
        let Some(exit) = nodes.last().cloned() else {
            self.aiming = false;
            self.chain_target = None;
            return;
        };
        let in_range = |e: &Fighter| dist(e.x, e.y, exit.x, exit.y) <= ATTACK_RANGE;
        let keep = self.chain_target.and_then(|id| t.get(id)).filter(|e| e.team != t.me.team && in_range(e)).cloned();
        let target = keep.or_else(|| t.enemies().filter(|e| in_range(e)).min_by_key(|e| (dist2(e.x, e.y, exit.x, exit.y), e.id)).cloned());
        let Some(target) = target else {
            self.aiming = false;
            return;
        };
        self.chain_target = Some(target.id);
        let diver = t.danger(t.me.x, t.me.y, DIVE_RADIUS) > 0;
        if diver && self.gate(GATE_TEAM) {
            if self.aiming {
                self.habit("dropped aim for a diver");
            }
            self.aiming = false;
            return;
        }
        self.aiming = true;
        if tick.is_multiple_of(6) {
            out.push(Cmd::Cc { target: t.me.id, kind: Cc::Disarm, ticks: 8 });
            out.push(Cmd::Buff { target: t.me.id, name: "vs_aim".into(), ticks: 8, mods: Mods { move_speed: -100, ..Mods::default() } });
        }
        if tick < self.next_attack {
            return;
        }
        self.next_attack = tick + t.attack_interval;
        if self.packets.len() >= PACKET_LIMIT as usize {
            return;
        }
        let mut path = VecDeque::new();
        let (mut px, mut py) = (t.me.x, t.me.y);
        for n in &nodes {
            path.push_back((n.uid, n.x, n.y));
            let _ = (px, py);
            (px, py) = (n.x, n.y);
        }
        path.push_back((0, target.x, target.y));
        let bonus = std::mem::take(&mut self.overload);
        self.packets.push(Packet {
            x: t.me.x,
            y: t.me.y,
            hop: (t.me.x, t.me.y),
            path,
            target: target.id,
            eff: 10000,
            bonus,
            dims: vec![],
            last_families: vec![],
            kaleido: false,
            true_pct: 0,
            ignore_wall: false,
            split: false,
            echo: false,
            source: "chain",
            base: t.me.attack,
        });
        if tick < self.crossfire_until {
            let others: Vec<Rift> = self.rifts.iter().filter(|r| r.live() && r.uid != exit.uid && !r.hidden(tick)).cloned().collect();
            let mut used = vec![target.id];
            for r in others.iter().take(4) {
                if self.packets.len() >= PACKET_LIMIT as usize || used.len() >= 3 {
                    break;
                }
                let foe = t.enemies().filter(|e| !used.contains(&e.id) && dist(e.x, e.y, r.x, r.y) <= ATTACK_RANGE).min_by_key(|e| (dist2(e.x, e.y, r.x, r.y), e.id)).cloned();
                if let Some(e) = foe {
                    used.push(e.id);
                    self.packets.push(Packet {
                        x: r.x,
                        y: r.y,
                        hop: (r.x, r.y),
                        path: VecDeque::from([(0, e.x, e.y)]),
                        target: e.id,
                        eff: CROSSFIRE_PCT * 100,
                        bonus: 0,
                        dims: vec![],
                        last_families: vec![],
                        kaleido: false,
                        true_pct: 0,
                        ignore_wall: true,
                        split: true,
                        echo: false,
                        source: "crossfire",
                        base: t.me.attack,
                    });
                }
            }
        }
    }
    fn step_packets(&mut self, out: &mut Vec<Cmd>, t: &Tick) {
        let tick = t.tick;
        let packets = std::mem::take(&mut self.packets);
        let mut keep = vec![];
        for mut p in packets {
            let mut budget = PACKET_SPEED;
            let mut alive = true;
            let mut hit = false;
            let mut bank = false;
            while budget > 0 && alive && !hit {
                let Some(&(uid, wx, wy)) = p.path.front() else {
                    alive = false;
                    break;
                };
                let (wx, wy) = if uid == 0 { t.get(p.target).map(|f| (f.x, f.y)).unwrap_or((wx, wy)) } else { (wx, wy) };
                let d = dist(p.x, p.y, wx, wy);
                if d > budget {
                    let (nx, ny) = along(p.x, p.y, wx - p.x, wy - p.y, budget);
                    p.x = nx;
                    p.y = ny;
                    break;
                }
                budget -= d;
                let len = dist(p.hop.0, p.hop.1, wx, wy);
                p.hop = (wx, wy);
                p.x = wx;
                p.y = wy;
                p.path.pop_front();
                if uid == 0 {
                    hit = true;
                    break;
                }
                let Some(r) = self.rift(uid).filter(|r| r.live()).cloned() else {
                    alive = false;
                    self.fx(out, format!("close_l{}", self.look()), wx, wy, None, 12);
                    break;
                };
                let mut keep_bp_v = keep_bp(len);
                if bank {
                    keep_bp_v = (keep_bp_v * 12 / 10).min(9500);
                }
                p.eff = p.eff * keep_bp_v / 10000;
                bank = r.kind == R_MIRROR;
                let mut unstable = 100;
                for dim in r.dims(tick) {
                    let dd = DIMS[dim];
                    match dd.prim {
                        P_TRUE => p.true_pct += dd.a,
                        P_LENS => p.bonus += dd.a,
                        P_ECHO | P_WILDECHO => p.echo = true,
                        P_PRISM => p.split = true,
                        P_KALEIDO => p.kaleido = true,
                        P_UNSTABLE => {
                            unstable = dd.a;
                            self.add_strain(dd.b);
                        }
                        _ => match PRIM_SIDE[dd.prim as usize] {
                            b'E' => p.dims.push(dim),
                            b'S' => {
                                let me = t.me.clone();
                                self.apply_dim(out, t, dim, &me, 100, (r.x, r.y, r.radius), Src::Imbue);
                            }
                            _ => {}
                        },
                    }
                    if dd.prim == P_ECHO || dd.prim == P_WILDECHO || dd.prim == P_PRISM {
                        p.dims.push(dim);
                    }
                }
                if unstable != 100 {
                    p.bonus += unstable - 100;
                }
                p.last_families.push(DIMS[r.dims(tick)[0]].family);
                if p.last_families.len() > 2 {
                    p.last_families.remove(0);
                }
            }
            if hit {
                self.hit(out, t, &p);
            } else if alive {
                if (tick + p.target).is_multiple_of(4) {
                    let dir = p.path.front().map_or(0, |w| heading8(w.1 - p.x, w.2 - p.y));
                    let look = self.look();
                    self.fx(out, format!("packet_l{look}_a{dir}"), p.x, p.y, None, 4);
                }
                keep.push(p);
            }
        }
        self.packets = keep;
    }
    fn hit(&mut self, out: &mut Vec<Cmd>, t: &Tick, p: &Packet) {
        let tick = t.tick;
        let Some(f) = t.get(p.target).cloned() else { return };
        let amount = p.base * p.eff / 10000 * (100 + p.bonus).max(10) / 100;
        let dealt = self.emit(out, &f, amount, Dmg::Phys, tick);
        if p.true_pct > 0 {
            self.emit(out, &f, amount * p.true_pct / 100, Dmg::True, tick);
        }
        self.stats.packets += 1;
        self.stats.eff_sum += p.eff;
        let s = (p.eff / 100).clamp(30, 100);
        for &dim in &p.dims {
            match DIMS[dim].prim {
                P_ECHO => self.later.push(Later::Repeat { at: tick + DIMS[dim].b as usize, target: f.id, amount: amount * DIMS[dim].a / 100, families: vec![] }),
                P_WILDECHO => {
                    let pct = 20 + self.random(61) as i64;
                    self.later.push(Later::Repeat { at: tick + 30, target: f.id, amount: amount * pct / 100, families: vec![] });
                }
                P_PRISM => {
                    let others: Vec<Fighter> = t.enemies().filter(|e| e.id != f.id && dist(e.x, e.y, f.x, f.y) <= 30000).take(DIMS[dim].a as usize - 1).cloned().collect();
                    for e in others {
                        self.emit(out, &e, amount * DIMS[dim].b / 100, Dmg::Phys, tick);
                    }
                }
                _ => self.apply_dim(out, t, dim, &f, s, (f.x, f.y, 0), Src::Imbue),
            }
        }
        // Siphon / Feast: allies inside a siphon rift around the target heal from the hit
        let siphons: Vec<(i64, i64, i64, i64)> = self
            .rifts
            .iter()
            .filter(|r| r.live() && r.covers(f.x, f.y))
            .flat_map(|r| r.dims(tick).into_iter().filter(|d| DIMS[*d].prim == P_SIPHON).map(move |d| (r.x, r.y, r.radius, DIMS[d].a)))
            .collect();
        for (x, y, rad, pct) in siphons {
            let allies: Vec<usize> = t.friends().filter(|a| dist(a.x, a.y, x, y) <= rad).map(|a| a.id).collect();
            for a in allies {
                out.push(Cmd::Heal { target: a, amount: dealt * pct / 100 });
            }
        }
        if p.last_families.len() == 2 {
            let idx = if p.kaleido { self.random(55) as usize } else { twist_index(p.last_families[0], p.last_families[1]) };
            self.twist(out, t, idx, &f, amount);
        }
        let look = self.look();
        self.fx(out, format!("impact_l{look}"), f.x, f.y, None, 12);
    }
    fn twist(&mut self, out: &mut Vec<Cmd>, t: &Tick, idx: usize, f: &Fighter, amount: i64) {
        let tick = t.tick;
        let w = TWISTS[idx];
        let nodes = self.chain_nodes(tick);
        let exit = nodes.last().map(|n| (n.x, n.y)).unwrap_or((f.x, f.y));
        let first = nodes.first().map(|n| (n.x, n.y));
        let mag = w.mag;
        let within = |x: i64, y: i64, cx: i64, cy: i64, r: i64| dist2(x, y, cx, cy) <= (r as i128) * (r as i128);
        match w.kind {
            0 => {
                self.control(out, f, Some(Cc::Root), mag, tick);
            }
            1 => {
                if f.controlled {
                    self.emit(out, f, amount * mag / 100, Dmg::Phys, tick);
                }
            }
            2 => {
                if self.displaced.get(&f.id).is_some_and(|d| tick <= d + 60) {
                    self.control(out, f, Some(Cc::Stun), mag, tick);
                }
            }
            3 => {
                let allies: Vec<usize> = t.friends().filter(|a| within(a.x, a.y, exit.0, exit.1, 20000)).map(|a| a.id).collect();
                for a in allies {
                    if self.cooldown_ok(5003, a, tick) {
                        self.set_cooldown(5003, a, tick + 600);
                        out.push(Cmd::Buff { target: a, name: "vst_sanctuary".into(), ticks: mag as usize, mods: Mods { cc_immune: true, ..Mods::default() } });
                    }
                }
            }
            4 => {
                if self.control(out, f, Some(Cc::Root), mag, tick) {
                    let to = lerp(f.x, f.y, exit.0, exit.1, 50);
                    self.displace(out, f, to.0, to.1, tick, t.wall, true);
                }
            }
            5 => {
                let second = t.enemies().filter(|e| e.id != f.id && within(e.x, e.y, f.x, f.y, 30000)).min_by_key(|e| (dist2(e.x, e.y, f.x, f.y), e.id)).cloned();
                if let Some(e) = second {
                    self.emit(out, &e, amount * mag / 100, Dmg::Phys, tick);
                }
            }
            6 => {
                if self.displaced.get(&f.id).is_some_and(|d| tick <= d + 60) {
                    self.emit(out, f, f.max_hp * mag / 1000, Dmg::Phys, tick);
                }
            }
            7 | 12 => {
                let allies: Vec<Fighter> = t.friends().filter(|a| within(a.x, a.y, exit.0, exit.1, 20000)).cloned().collect();
                for a in allies {
                    let n = if w.kind == 7 { amount * mag / 100 } else { a.max_hp * mag / 1000 };
                    out.push(Cmd::Heal { target: a.id, amount: n });
                }
            }
            8 => {
                let far = self.rifts.iter().filter(|r| r.live()).max_by_key(|r| (dist2(r.x, r.y, exit.0, exit.1), r.uid)).cloned();
                if let (Some(r), Some(last)) = (far, nodes.last()) {
                    let dim = last.dims(tick)[0];
                    let near: Vec<Fighter> = t.enemies().filter(|e| r.covers(e.x, e.y)).cloned().collect();
                    for e in near {
                        self.apply_dim(out, t, dim, &e, 100, (r.x, r.y, r.radius), Src::Zone);
                    }
                }
            }
            9 => {
                let near: Vec<Fighter> = t.enemies().filter(|e| e.id != f.id && within(e.x, e.y, f.x, f.y, 20000)).cloned().collect();
                for e in near {
                    let to = lerp(e.x, e.y, f.x, f.y, mag.min(100));
                    self.displace(out, &e, to.0, to.1, tick, t.wall, true);
                }
            }
            10 => {
                let allies: Vec<usize> = t.friends().filter(|a| within(a.x, a.y, exit.0, exit.1, 20000)).map(|a| a.id).collect();
                for a in allies {
                    out.push(Cmd::Buff { target: a, name: "vst_escort".into(), ticks: 60, mods: Mods { move_speed: mag as i32, ..Mods::default() } });
                }
            }
            11 => {
                if let Some((x, y)) = first {
                    self.displace(out, f, x, y, tick, t.wall, true);
                }
            }
            13 => {
                let allies: Vec<Fighter> = t.friends().filter(|a| within(a.x, a.y, exit.0, exit.1, 20000)).cloned().collect();
                for a in allies {
                    self.shield(out, &a, a.max_hp * mag / 1000, 180, tick);
                }
            }
            _ => self.later.push(Later::Repeat { at: tick + 12, target: f.id, amount: amount * mag / 100, families: vec![] }),
        }
        self.fx(out, format!("twist{}", w.kind), f.x, f.y, None, 18);
    }
    /// Rift layers are refreshed on a 48-tick loop, staggered so a busy tick never spends the effect budget at once.
    fn render(&mut self, out: &mut Vec<Cmd>, t: &Tick) {
        let tick = t.tick;
        let look = self.look();
        let tier = self.tier4().min(3);
        let rifts: Vec<Rift> = self.rifts.iter().filter(|r| r.live()).cloned().collect();
        for r in &rifts {
            let host = r.host;
            for layer in 0..3usize {
                if !(tick + r.uid as usize * 11 + layer * 16).is_multiple_of(48) {
                    continue;
                }
                if r.hidden(tick) && layer > 0 {
                    continue;
                }
                let dim = r.dims(tick)[0];
                let tag = match layer {
                    0 if r.hidden(tick) => format!("seedhint_l{look}"),
                    0 if matches!(r.kind, R_SLASH | R_MIRROR) => format!("rift_{}_l{look}_a{}", RIFTS[r.kind].id, heading8(r.ex - r.x, r.ey - r.y) % 4),
                    0 => format!("rift_{}_l{look}", RIFTS[r.kind].id),
                    1 => format!("core_f{}_t{tier}", DIMS[dim].family),
                    _ => format!("glyph{dim:02}"),
                };
                let (x, y) = if matches!(r.kind, R_SLASH | R_MIRROR) { ((r.x + r.ex) / 2, (r.y + r.ey) / 2) } else { (r.x, r.y) };
                self.fx(out, tag, x, y, host, 48);
            }
        }
        // link threads along the chain, from Vesper through every node
        let nodes = self.chain_nodes(tick);
        let mut prev = (t.me.x, t.me.y);
        for (i, n) in nodes.iter().enumerate() {
            if (tick + i * 7).is_multiple_of(48) {
                let dir = heading8(n.x - prev.0, n.y - prev.1);
                self.fx(out, format!("link_l{look}_a{dir}"), (n.x + prev.0) / 2, (n.y + prev.1) / 2, None, 48);
            }
            prev = (n.x, n.y);
        }
        if tick % DRIFT_TICKS as usize == (DRIFT_TICKS - DRIFT_WARN) as usize {
            self.fx(out, "atlas_warn".into(), t.me.x, t.me.y, Some(t.me.id), DRIFT_WARN as usize);
        }
    }
}

/// What the planner sees when it thinks.
#[derive(Clone, Debug)]
pub struct View {
    pub enemies: Vec<Fighter>,
    pub allies: Vec<Fighter>,
    pub focus: Fighter,
    pub tank: usize,
    pub ally_centre: (i64, i64),
    pub enemy_centre: (i64, i64),
    pub front: Option<Fighter>,
    pub diver: bool,
}
fn centre(v: &[Fighter], or: (i64, i64)) -> (i64, i64) {
    if v.is_empty() {
        return or;
    }
    let n = v.len() as i64;
    (v.iter().map(|f| f.x).sum::<i64>() / n, v.iter().map(|f| f.y).sum::<i64>() / n)
}

impl Core {
    fn view(&self, t: &Tick, enemies: Vec<Fighter>) -> View {
        let allies: Vec<Fighter> = t.allies().cloned().collect();
        let tank = enemies.iter().max_by_key(|e| (e.max_hp, e.id)).map_or(0, |e| e.id);
        let focus = if self.gate(GATE_PRIORITY) {
            enemies.iter().min_by_key(|e| (self.priority_rank(e, tank, t), e.id)).cloned()
        } else {
            enemies.iter().min_by_key(|e| (dist2(e.x, e.y, t.me.x, t.me.y), e.id)).cloned()
        }
        .unwrap_or_else(|| t.me.clone());
        let me = (t.me.x, t.me.y);
        let front = allies.iter().max_by_key(|a| (a.max_hp, a.id)).cloned();
        View {
            ally_centre: centre(&allies, me),
            enemy_centre: centre(&enemies, me),
            diver: t.danger(t.me.x, t.me.y, DIVE_RADIUS) > 0,
            focus,
            tank,
            front,
            allies,
            enemies,
        }
    }
    /// Lower is a better target: low health, high damage, never the tank.
    fn priority_rank(&self, e: &Fighter, tank: usize, t: &Tick) -> i64 {
        let hp = e.hp * 100 / e.max_hp.max(1);
        hp - e.attack / 6 + if e.id == tank { 80 } else { 0 } + dist(e.x, e.y, t.me.x, t.me.y) / 20000
    }
    fn prio(&self, e: &Fighter, v: &View) -> i64 {
        if self.gate(GATE_PRIORITY) {
            30 + (100 - e.hp * 100 / e.max_hp.max(1)) / 2 + if e.id == v.tank { -40 } else { 0 }
        } else {
            20
        }
    }
    /// Where a fighter will be when a rift opening `delay` ticks from now arrives: a Spark aims at where they stand.
    fn lead(&self, f: &Fighter, delay: usize) -> (i64, i64) {
        if self.rank == 0 {
            return (f.x, f.y);
        }
        self.forecast(f, (delay as i64).min(self.table(&LOOK_AHEAD)))
    }
    fn delay_to(&self, t: &Tick, x: i64, y: i64, kind: usize) -> usize {
        open_delay(dist(t.me.x, t.me.y, x, y), RIFTS[kind].delay_pct)
    }
    /// From Safety up: rifts stay off the paths enemies are walking (they would be broken).
    fn off_path(&self, t: &Tick, x: i64, y: i64) -> bool {
        if !self.gate(GATE_SAFETY) {
            return true;
        }
        !t.enemies().any(|e| {
            let (fx, fy) = self.forecast(e, 60);
            seg_dist(e.x, e.y, fx, fy, x, y) <= BREAK_CORE * 2
        })
    }
    fn exit_ok(&self, t: &Tick, x: i64, y: i64) -> bool {
        !self.gate(GATE_SAFETY) || t.danger(x, y, 35000) < 3
    }
    /// How much the planner wants a dimension on its route (atlas routing).
    pub fn dim_value(&self, dim: usize) -> i64 {
        let p = DIMS[dim].prim;
        match p {
            P_BANISH | P_STUN | P_ROOT | P_SILENCE | P_DISARM => 18,
            P_SLOW | P_CDUP => 10,
            P_AMPLIFY | P_DOT | P_TRUE | P_STATDOWN | P_HEALCUT | P_DRAIN | P_SHATTER => 14,
            P_REFLECT | P_PULL | P_KNOCK | P_SHOVE | P_GRAB | P_REWIND | P_TETHER => 6,
            P_STATICBURST | P_ENTROPY | P_STORM => 8,
            P_LENS | P_ECHO | P_PRISM | P_WILDECHO => 16,
            P_UNSTABLE => {
                if self.strain > 60 {
                    -15
                } else {
                    10
                }
            }
            P_VENT => {
                if self.strain > 50 {
                    14
                } else {
                    4
                }
            }
            P_SWAPSPACE | P_SCRAMBLE => -6,
            _ => match PRIM_SIDE[p as usize] {
                b'A' => 6,
                b'S' => 8,
                _ => 5,
            },
        }
    }
    /// A chain from Vesper to a firing spot within range of `target`.
    fn plan_chain(&self, t: &Tick, target: &Fighter, kind_mid: usize) -> Option<(Vec<RiftSpec>, i64, usize)> {
        let guess = self.lead(target, 40);
        let reach = ATTACK_RANGE * 7 / 10;
        let d = dist(t.me.x, t.me.y, guess.0, guess.1);
        if d <= reach + 20000 {
            return None;
        }
        let exit = along(guess.0, guess.1, t.me.x - guess.0, t.me.y - guess.1, reach);
        let span = dist(t.me.x, t.me.y, exit.0, exit.1);
        let max = if self.gate(GATE_GEOMETRY) { self.table(&MAX_LINKS) as usize } else { 1 };
        let mut best = (i64::MIN, 1usize, 0i64);
        for n in 1..=max {
            let eff = chain_bp(&vec![span / n as i64; n]);
            let score = eff - n as i64 * 400;
            if score > best.0 {
                best = (score, n, eff);
            }
        }
        let n = best.1;
        let (px, py) = (-(exit.1 - t.me.y), exit.0 - t.me.x);
        let mut nodes = vec![];
        for i in 1..=n as i64 {
            let mut p = (t.me.x + (exit.0 - t.me.x) * i / n as i64, t.me.y + (exit.1 - t.me.y) * i / n as i64);
            let routing = self.table(&ATLAS_ROUTING);
            if routing >= 1 {
                let here = self.dim_value(self.atlas.dim_at(p.0, p.1, t.tick));
                let mut pick = (here, p);
                for off in [-ATLAS_CELL / 2, ATLAS_CELL / 2] {
                    let q = clamp_map(p.0 + along(0, 0, px, py, off).0, p.1 + along(0, 0, px, py, off).1);
                    let v = self.dim_value(self.atlas.dim_at(q.0, q.1, t.tick));
                    let better = if routing >= 2 { v > pick.0 } else { here < 0 && v > pick.0 };
                    if better && !(t.wall)(q.0, q.1) {
                        pick = (v, q);
                    }
                }
                p = pick.1;
            }
            if !self.off_path(t, p.0, p.1) {
                p = clamp_map(p.0 + along(0, 0, px, py, 24000).0, p.1 + along(0, 0, px, py, 24000).1);
            }
            let kind = if i == 1 && kind_mid != R_TEAR && n > 1 { kind_mid } else { R_TEAR };
            nodes.push(if kind == R_MIRROR {
                let (hx, hy) = along(0, 0, px, py, 20000);
                RiftSpec { chain: true, ..RiftSpec::line(R_MIRROR, p.0 - hx, p.1 - hy, p.0 + hx, p.1 + hy) }
            } else {
                RiftSpec::node(kind, p.0, p.1)
            });
        }
        // the real links, Vesper to the exit
        let mut links = vec![];
        let mut prev = (t.me.x, t.me.y);
        for s in &nodes {
            links.push(dist(prev.0, prev.1, s.x, s.y));
            prev = (s.x, s.y);
        }
        Some((nodes, chain_bp(&links), n))
    }
    fn wall_between(&self, t: &Tick, a: (i64, i64), b: (i64, i64)) -> bool {
        let steps = (dist(a.0, a.1, b.0, b.1) / 16000).max(1);
        (1..steps).any(|i| (t.wall)(a.0 + (b.0 - a.0) * i / steps, a.1 + (b.1 - a.1) * i / steps))
    }
    fn safe_back(&self, v: &View, t: &Tick, len: i64) -> (i64, i64) {
        let (dx, dy) = (v.ally_centre.0 - v.enemy_centre.0, v.ally_centre.1 - v.enemy_centre.1);
        let (dx, dy) = if dx == 0 && dy == 0 { (t.me.x - v.enemy_centre.0, t.me.y - v.enemy_centre.1) } else { (dx, dy) };
        clamp_map(along(t.me.x, t.me.y, dx, dy, len).0, along(t.me.x, t.me.y, dx, dy, len).1)
    }
    fn random_near(&mut self, x: i64, y: i64, r: i64) -> (i64, i64) {
        let (ux, uy) = DIRS[self.random(16) as usize];
        let k = self.random(r.max(1) as u64) as i64;
        clamp_map(x + ux * k / 10000, y + uy * k / 10000)
    }
    /// Candidate actions for one technique. Every technique can be proposed at every rank; the gates and tables
    /// only shape where it goes and how much the planner likes it.
    pub fn propose(&mut self, t: &Tick, v: &View, tech: usize) -> Vec<Action> {
        let tick = t.tick;
        let me = (t.me.x, t.me.y);
        let f = v.focus.clone();
        let act = |rifts: Vec<RiftSpec>, subject: Option<usize>, uses: Vec<u32>, score: i64, habit: &'static str| Action { tech, rifts, subject, uses, score, habit };
        let live: Vec<Rift> = self.rifts.iter().filter(|r| r.live() && !r.hidden(tick)).cloned().collect();
        let nodes = self.chain_nodes(tick);
        let chain_live = !nodes.is_empty();
        let mut out = vec![];
        match tech {
            T_CHAIN => {
                if chain_live {
                    if self.gate(GATE_SPINE) {
                        if let Some(n) = nodes.iter().find(|n| n.expires <= tick + 60) {
                            out.push(act(vec![RiftSpec { order: n.chain, ..RiftSpec::node(R_TEAR, n.x, n.y) }], None, vec![], 46, "spine refreshed"));
                        }
                    }
                } else if let Some((rifts, eff, n)) = self.plan_chain(t, &f, R_TEAR) {
                    let habit = if n >= 2 { "solved the link count" } else if self.rank >= 1 { "led target" } else { "" };
                    out.push(act(rifts, Some(f.id), vec![], 28 + eff / 300, habit));
                }
            }
            T_BANK => {
                if !chain_live {
                    let walled = self.wall_between(t, me, (f.x, f.y));
                    if let Some((mut rifts, eff, n)) = self.plan_chain(t, &f, R_MIRROR) {
                        if n == 1 {
                            // one link: the pane itself is the node
                            let p = (rifts[0].x, rifts[0].y);
                            let (ox, oy) = if self.gate(GATE_SAFETY) { along(0, 0, -(p.1 - me.1), p.0 - me.0, 20000) } else { DIRS[self.random(16) as usize] };
                            let (ox, oy) = if self.gate(GATE_SAFETY) { (ox, oy) } else { (ox * 2, oy * 2) };
                            rifts[0] = RiftSpec { chain: true, ..RiftSpec::line(R_MIRROR, p.0 - ox, p.1 - oy, p.0 + ox, p.1 + oy) };
                        }
                        let score = if self.gate(GATE_GEOMETRY) { if walled { 40 + eff / 400 } else { 0 } } else { 12 + eff / 600 };
                        out.push(act(rifts, Some(f.id), vec![], score, if walled { "bank shot around a wall" } else { "" }));
                    }
                }
            }
            T_CROSSFIRE => {
                if chain_live && live.len() >= 2 && tick >= self.crossfire_until {
                    let covered = live.iter().filter(|r| t.enemies().any(|e| dist(e.x, e.y, r.x, r.y) <= ATTACK_RANGE)).count() as i64;
                    let score = if self.gate(GATE_GEOMETRY) { 12 * covered.min(3) } else { 18 };
                    out.push(act(vec![], None, vec![], score, if covered >= 2 { "crossfire split" } else { "" }));
                }
            }
            T_RAIN => {
                let mut spot = (f.x, f.y);
                let mut count = 0i64;
                if self.gate(GATE_GEOMETRY) {
                    for e in &v.enemies {
                        let p = self.lead(e, self.delay_to(t, e.x, e.y, R_SKY));
                        let c = v.enemies.iter().filter(|o| dist(o.x, o.y, p.0, p.1) <= 26000).count() as i64;
                        if c > count {
                            count = c;
                            spot = p;
                        }
                    }
                } else {
                    count = v.enemies.iter().filter(|o| dist(o.x, o.y, spot.0, spot.1) <= 26000).count() as i64;
                }
                out.push(act(vec![RiftSpec::at(R_SKY, spot.0, spot.1)], None, vec![], 14 * count - 4, if count >= 2 { "rain on a cluster" } else { "" }));
            }
            T_VOLLEY => {
                let p = self.lead(&f, self.delay_to(t, f.x, f.y, R_SLASH));
                let (dx, dy) = if self.gate(GATE_GEOMETRY) { (-(p.1 - me.1), p.0 - me.0) } else { DIRS[self.random(16) as usize] };
                let a = along(p.0, p.1, dx, dy, 30000);
                let b = along(p.0, p.1, -dx, -dy, 30000);
                let n = v.enemies.iter().filter(|e| seg_dist(a.0, a.1, b.0, b.1, e.x, e.y) <= 22000).count() as i64;
                out.push(act(vec![RiftSpec::line(R_SLASH, a.0, a.1, b.0, b.1)], None, vec![], 10 * n + 2, ""));
            }
            T_IMPLODE => {
                if !live.is_empty() {
                    let doomed = live.iter().filter(|r| r.dwell.iter().any(|(id, n)| *id < 1_000_000 && *n >= 20)).max_by_key(|r| r.uid).cloned();
                    if let (true, Some(r)) = (self.gate(GATE_SAFETY), doomed) {
                        let n = t.enemies().filter(|e| r.reach(e.x, e.y) <= r.radius * 3 / 2).count() as i64;
                        out.push(act(vec![], None, vec![r.uid], 34 + 6 * n, "collapsed a rift before it broke"));
                    }
                    let pick = if self.gate(GATE_GEOMETRY) {
                        live.iter().max_by_key(|r| (t.enemies().filter(|e| r.reach(e.x, e.y) <= r.radius * 3 / 2).count(), r.uid)).cloned()
                    } else {
                        let i = self.random(live.len() as u64) as usize;
                        Some(live[i].clone())
                    };
                    if let Some(r) = pick {
                        let n = t.enemies().filter(|e| r.reach(e.x, e.y) <= r.radius * 3 / 2).count() as i64;
                        let low = t.enemies().any(|e| r.reach(e.x, e.y) <= r.radius * 3 / 2 && e.hp * 4 < e.max_hp);
                        let score = if self.gate(GATE_GEOMETRY) { if n >= 2 || low { 16 * n + 6 } else { -20 } } else { 10 + 8 * n };
                        out.push(act(vec![], None, vec![r.uid], score, if n >= 2 { "implode on a pile" } else { "" }));
                    }
                }
            }
            T_DOMINO => {
                let linked: Vec<&Rift> = live.iter().filter(|r| r.chain.is_some() || r.kind == R_TWIN).collect();
                if linked.len() >= 2 {
                    let total = linked.iter().map(|r| t.enemies().filter(|e| r.reach(e.x, e.y) <= r.radius * 3 / 2).count()).sum::<usize>() as i64;
                    let score = if self.gate(GATE_GEOMETRY) { if total >= 2 { 9 * total + 8 } else { -20 } } else { 12 };
                    let mut uses: Vec<u32> = linked.iter().map(|r| r.uid).collect();
                    uses.sort();
                    out.push(act(vec![], None, uses, score, if total >= 2 { "domino through the pile" } else { "" }));
                }
            }
            T_OVERLOAD => {
                if chain_live && self.strain >= 25 && self.overload == 0 {
                    let spend = self.strain.min(OVERLOAD_MAX);
                    let score = if self.gate(GATE_SPINE) && self.strain >= 80 { spend / 3 + 25 } else if self.gate(GATE_SAFETY) { spend / 3 } else { 14 };
                    out.push(act(vec![], None, vec![], score, if self.gate(GATE_SPINE) && self.strain >= 80 { "rode strain into Overload" } else { "" }));
                }
            }
            T_STEP => {
                if tick >= self.self_move_until {
                    if self.gate(GATE_SURVIVAL) && v.diver {
                        let to = self.safe_back(v, t, 60000);
                        out.push(act(vec![RiftSpec::entrance(R_TEAR, me.0, me.1), RiftSpec::at(R_TEAR, to.0, to.1)], None, vec![], 55, "escaped a dive"));
                    } else {
                        let to = along(f.x, f.y, me.0 - f.x, me.1 - f.y, 15000);
                        if self.exit_ok(t, to.0, to.1) {
                            let score = if self.gate(GATE_SURVIVAL) { 0 } else { 16 };
                            out.push(act(vec![RiftSpec::entrance(R_TEAR, me.0, me.1), RiftSpec::at(R_TEAR, to.0, to.1)], None, vec![], score, ""));
                        }
                    }
                }
            }
            T_GATE | T_SKYDROP => {
                let exit_kind = if tech == T_SKYDROP { R_SKY } else { R_TEAR };
                if self.gate(GATE_TEAM) {
                    if let Some(fr) = v.front.clone() {
                        let (vx, vy) = self.velocity.get(&fr.id).copied().unwrap_or((0, 0));
                        let toward = vx as i128 * (v.enemy_centre.0 - fr.x) as i128 + vy as i128 * (v.enemy_centre.1 - fr.y) as i128 > 0;
                        let close = dist(fr.x, fr.y, v.enemy_centre.0, v.enemy_centre.1) < 170000 && t.danger(fr.x, fr.y, 45000) == 0;
                        let wait = self.table(&ALLY_WAIT) as usize;
                        let ent = self.lead(&fr, self.delay_to(t, fr.x, fr.y, R_TEAR) + wait);
                        let (dx, dy) = (v.enemy_centre.0 - v.ally_centre.0, v.enemy_centre.1 - v.ally_centre.1);
                        let behind = clamp_map(along(v.enemy_centre.0, v.enemy_centre.1, dx, dy, 18000).0, along(v.enemy_centre.0, v.enemy_centre.1, dx, dy, 18000).1);
                        let cluster = v.enemies.iter().filter(|e| dist(e.x, e.y, behind.0, behind.1) <= 40000).count() as i64;
                        let score = if toward && close { if tech == T_SKYDROP { 40 + 6 * cluster } else { 50 } } else { 8 };
                        out.push(act(vec![RiftSpec::entrance(R_TEAR, ent.0, ent.1), RiftSpec::at(exit_kind, behind.0, behind.1)], Some(fr.id), vec![], score, if toward && close { "Gate synced with the tank" } else { "" }));
                    }
                } else if !v.allies.is_empty() {
                    let i = self.random(v.allies.len() as u64) as usize;
                    let a = v.allies[i].clone();
                    out.push(act(vec![RiftSpec::entrance(R_TEAR, a.x, a.y), RiftSpec::at(exit_kind, f.x, f.y)], Some(a.id), vec![], 16, ""));
                }
            }
            T_KIDNAP => {
                let subject = f.clone();
                if !(self.gate(GATE_SAFETY) && subject.cc_immune) {
                    let ent = self.lead(&subject, self.delay_to(t, subject.x, subject.y, R_TEAR));
                    let exit = if self.gate(GATE_TEAM) && !v.allies.is_empty() {
                        let front = v.allies.iter().min_by_key(|a| (dist2(a.x, a.y, v.enemy_centre.0, v.enemy_centre.1), a.id)).unwrap();
                        Some(along(front.x, front.y, v.ally_centre.0 - front.x, v.ally_centre.1 - front.y, 8000))
                    } else if self.gate(GATE_SAFETY) {
                        let p = self.safe_back(v, t, 45000);
                        if dist(p.0, p.1, me.0, me.1) > 25000 {
                            Some(p)
                        } else {
                            None
                        }
                    } else {
                        Some(me)
                    };
                    if let Some(exit) = exit {
                        let score = if self.gate(GATE_TEAM) { 44 + self.prio(&subject, v) / 2 } else { 20 };
                        let habit = if self.gate(GATE_PRIORITY) && subject.id != v.tank { "Kidnapped the carry into the team" } else { "" };
                        out.push(act(vec![RiftSpec::entrance(R_TEAR, ent.0, ent.1), RiftSpec::at(R_TEAR, exit.0, exit.1)], Some(subject.id), vec![], score, habit));
                    }
                }
            }
            T_SINKTRAP => {
                let spot = if self.gate(GATE_PRIORITY) { lerp(v.enemy_centre.0, v.enemy_centre.1, v.ally_centre.0, v.ally_centre.1, 35) } else { self.lead(&f, self.delay_to(t, f.x, f.y, R_SINKHOLE)) };
                let n = v.enemies.iter().filter(|e| dist(e.x, e.y, spot.0, spot.1) <= 40000).count() as i64;
                let exit = if self.gate(GATE_TEAM) { v.ally_centre } else { me };
                out.push(act(vec![RiftSpec::entrance(R_SINKHOLE, spot.0, spot.1), RiftSpec::at(R_TEAR, exit.0, exit.1)], None, vec![], 9 * n + 10, if self.gate(GATE_PRIORITY) { "Sinkhole at the chokepoint" } else { "" }));
            }
            T_SWAP => {
                if self.gate(GATE_TEAM) {
                    if let Some(fr) = v.front.clone() {
                        let a = self.lead(&f, self.delay_to(t, f.x, f.y, R_TWIN));
                        let b = self.lead(&fr, self.delay_to(t, fr.x, fr.y, R_TWIN));
                        out.push(act(vec![RiftSpec::entrance(R_TWIN, a.0, a.1), RiftSpec::at(R_TWIN, b.0, b.1)], Some(f.id), vec![], 36 + self.prio(&f, v) / 3, "swapped the target into the team"));
                    }
                } else if !self.gate(GATE_SAFETY) || t.danger(f.x, f.y, 35000) < 3 {
                    out.push(act(vec![RiftSpec::entrance(R_TWIN, me.0, me.1), RiftSpec::at(R_TWIN, f.x, f.y)], Some(f.id), vec![], 18, ""));
                }
            }
            T_FLING => {
                let start = self.lead(&f, self.delay_to(t, f.x, f.y, R_WANDER));
                let (dx, dy) = if self.gate(GATE_TEAM) { (v.ally_centre.0 - start.0, v.ally_centre.1 - start.1) } else { DIRS[self.random(16) as usize] };
                let end = clamp_map(along(start.0, start.1, dx, dy, 110000).0, along(start.0, start.1, dx, dy, 110000).1);
                let n = v.enemies.iter().filter(|e| dist(e.x, e.y, start.0, start.1) <= 30000).count() as i64;
                out.push(act(vec![RiftSpec::line(R_WANDER, start.0, start.1, end.0, end.1)], None, vec![], 10 * n + 4, if self.gate(GATE_TEAM) { "Fling toward the team" } else { "" }));
            }
            T_HOOK => {
                if !(self.gate(GATE_SAFETY) && f.cc_immune) {
                    let anchor = RiftSpec { host: Some(f.id), ..RiftSpec::at(R_ANCHOR, f.x, f.y) };
                    let mut rifts = vec![anchor];
                    if self.gate(GATE_TEAM) {
                        rifts.push(RiftSpec::at(R_TEAR, v.ally_centre.0, v.ally_centre.1));
                    }
                    out.push(act(rifts, Some(f.id), vec![], 22 + self.prio(&f, v) / 3, ""));
                }
            }
            T_PHASE => {
                if tick >= self.self_move_until {
                    let hurt = t.me.hp * 2 < t.me.max_hp && t.danger(me.0, me.1, 40000) >= 1;
                    let to = if self.gate(GATE_SURVIVAL) { self.safe_back(v, t, 50000) } else { self.random_near(me.0, me.1, 50000) };
                    let score = if self.gate(GATE_SURVIVAL) { if hurt { 55 } else { -10 } } else { 12 };
                    out.push(act(vec![RiftSpec::entrance(R_TEAR, me.0, me.1), RiftSpec::at(R_TEAR, to.0, to.1)], None, vec![], score, if hurt { "phased out of a burst" } else { "" }));
                }
            }
            T_CURTAIN | T_REFLECT => {
                let kind = if tech == T_CURTAIN { R_SLASH } else { R_MIRROR };
                let near = v.enemies.iter().filter(|e| dist(e.x, e.y, me.0, me.1) <= 90000).min_by_key(|e| (dist2(e.x, e.y, me.0, me.1), e.id)).cloned();
                if let Some(e) = near {
                    let mid = lerp(me.0, me.1, e.x, e.y, 45);
                    let half = RIFTS[kind].length / 2;
                    let (dx, dy) = if self.gate(if tech == T_REFLECT { GATE_SAFETY } else { GATE_SURVIVAL }) { (-(e.y - me.1), e.x - me.0) } else { DIRS[self.random(16) as usize] };
                    let a = along(mid.0, mid.1, dx, dy, half);
                    let b = along(mid.0, mid.1, -dx, -dy, half);
                    let score = if self.gate(GATE_SURVIVAL) { if v.diver { 40 } else { 0 } } else { 12 };
                    out.push(act(vec![RiftSpec::line(kind, a.0, a.1, b.0, b.1)], None, vec![], score, if v.diver { "peeled a diver" } else { "" }));
                }
            }
            T_RESCUE | T_POCKET => {
                let low = v.allies.iter().filter(|a| a.hp * 100 / a.max_hp.max(1) < if tech == T_RESCUE { 25 } else { 40 }).min_by_key(|a| (a.hp, a.id)).cloned();
                let pick = if self.gate(GATE_SURVIVAL) { low } else if v.allies.is_empty() { None } else { let i = self.random(v.allies.len() as u64) as usize; Some(v.allies[i].clone()) };
                if let Some(a) = pick {
                    let pressed = t.danger(a.x, a.y, 40000) >= if tech == T_RESCUE { 1 } else { 2 };
                    let ent = self.lead(&a, self.delay_to(t, a.x, a.y, R_TEAR));
                    let score = if self.gate(GATE_SURVIVAL) { if pressed { 58 } else { 10 } } else { 12 };
                    let mut rifts = vec![RiftSpec::entrance(R_TEAR, ent.0, ent.1)];
                    if tech == T_RESCUE {
                        let safe = self.safe_back(v, t, 30000);
                        rifts.push(RiftSpec::at(R_TEAR, safe.0, safe.1));
                    }
                    out.push(act(rifts, Some(a.id), vec![], score, if pressed { "saved an ally" } else { "" }));
                }
            }
            T_LIFELINE => {
                if !self.rifts.iter().any(|r| !r.done && r.tech == T_LIFELINE) {
                    let close = v.enemies.iter().any(|e| dist(e.x, e.y, me.0, me.1) <= 200000);
                    let spot = if self.gate(GATE_SURVIVAL) { self.safe_back(v, t, 80000) } else { self.random_near(me.0, me.1, 80000) };
                    let score = if self.gate(GATE_SURVIVAL) { if close { 32 } else { 0 } } else { 10 };
                    out.push(act(vec![RiftSpec::at(R_ANCHOR, spot.0, spot.1)], Some(t.me.id), vec![], score, if close { "Lifeline set before the fight" } else { "" }));
                }
            }
            T_ESCORT => {
                if !chain_live {
                    let pick = if self.gate(GATE_TEAM) { v.front.clone() } else if v.allies.is_empty() { None } else { let i = self.random(v.allies.len() as u64) as usize; Some(v.allies[i].clone()) };
                    if let Some(a) = pick {
                        let diving = dist(a.x, a.y, v.enemy_centre.0, v.enemy_centre.1) < 90000;
                        let score = if self.gate(GATE_TEAM) { if diving { 42 } else { 6 } } else { 12 };
                        out.push(act(vec![RiftSpec { host: Some(a.id), chain: true, ..RiftSpec::at(R_ANCHOR, a.x, a.y) }], Some(a.id), vec![], score, if diving { "Escort on the diver" } else { "" }));
                    }
                }
            }
            T_ROTATION => {
                let far = v.allies.iter().filter(|a| dist(a.x, a.y, v.enemy_centre.0, v.enemy_centre.1) > 220000).min_by_key(|a| a.id).cloned();
                let fighting = v.allies.iter().filter(|a| dist(a.x, a.y, v.enemy_centre.0, v.enemy_centre.1) <= 120000).count();
                if let Some(a) = far {
                    let score = if self.gate(GATE_TEAM) { if fighting >= 1 { 30 } else { 4 } } else { 8 };
                    let ent = self.lead(&a, self.delay_to(t, a.x, a.y, R_TWIN));
                    let near_fight = lerp(v.enemy_centre.0, v.enemy_centre.1, me.0, me.1, 50);
                    out.push(act(vec![RiftSpec::entrance(R_TWIN, ent.0, ent.1), RiftSpec::at(R_TWIN, near_fight.0, near_fight.1)], Some(a.id), vec![], score, if fighting >= 1 { "rotated an ally into the fight" } else { "" }));
                }
            }
            T_SEEDTRAP => {
                let subject = f.clone();
                let spot = if self.rank == 0 {
                    self.random_near(subject.x, subject.y, 60000)
                } else {
                    self.forecast(&subject, self.table(&LOOK_AHEAD) * 2)
                };
                let mut rifts = vec![RiftSpec::at(R_SEED, spot.0, spot.1)];
                if self.gate(GATE_TEAM) {
                    rifts.push(RiftSpec::at(R_TEAR, v.ally_centre.0, v.ally_centre.1));
                }
                let score = 22 + if self.gate(GATE_PRIORITY) { 10 } else { 0 };
                out.push(act(rifts, Some(subject.id), vec![], score, if self.gate(GATE_PRIORITY) { "Seed on the carry's path" } else { "" }));
            }
            T_SWEEP => {
                let c = v.enemy_centre;
                let (dx, dy) = (-(c.1 - me.1), c.0 - me.0);
                let a = along(c.0, c.1, dx, dy, 50000);
                let b = along(c.0, c.1, -dx, -dy, 50000);
                let n = v.enemies.iter().filter(|e| seg_dist(a.0, a.1, b.0, b.1, e.x, e.y) <= 20000).count() as i64;
                out.push(act(vec![RiftSpec::line(R_WANDER, a.0, a.1, b.0, b.1)], None, vec![], 8 * n, ""));
            }
            T_BORROW => {
                if let Some(exit) = nodes.last() {
                    let (cell, val) = if self.table(&ATLAS_ROUTING) >= 2 {
                        let mut best = ((0, 0), i64::MIN);
                        for d in 0..100 {
                            let c = self.atlas.centre_of(d, tick);
                            if dist(c.0, c.1, me.0, me.1) <= 320000 && !(t.wall)(c.0, c.1) {
                                let val = self.dim_value(d);
                                if val > best.1 {
                                    best = (c, val);
                                }
                            }
                        }
                        best
                    } else {
                        let c = self.random_near(me.0, me.1, 300000);
                        (c, self.dim_value(self.atlas.dim_at(c.0, c.1, tick)))
                    };
                    if val > i64::MIN {
                        let score = if self.table(&ATLAS_ROUTING) >= 2 { val * 2 } else { 10 };
                        out.push(act(vec![RiftSpec::at(R_TEAR, cell.0, cell.1)], None, vec![exit.uid], score, if self.table(&ATLAS_ROUTING) >= 2 { "borrowed a far dimension" } else { "" }));
                    }
                }
            }
            T_STITCH => {
                let tears: Vec<&Rift> = live.iter().filter(|r| r.kind == R_TEAR && r.dim2.is_none()).collect();
                let mut pair = None;
                for (i, a) in tears.iter().enumerate() {
                    for b in tears.iter().skip(i + 1) {
                        if dist(a.x, a.y, b.x, b.y) <= 40000 {
                            let fresh = DIMS[a.dim].family != DIMS[b.dim].family;
                            if pair.is_none() || (fresh && self.gate(GATE_PRIORITY)) {
                                pair = Some((a.uid, b.uid, fresh));
                            }
                        }
                    }
                }
                if let Some((a, b, fresh)) = pair {
                    let score = if self.gate(GATE_PRIORITY) { if fresh { 26 } else { 4 } } else { 12 };
                    out.push(act(vec![], None, vec![a, b], score, if fresh { "stitched two worlds" } else { "" }));
                }
            }
            _ => {}
        }
        out
    }
    fn cost(a: &Action) -> usize {
        a.rifts.iter().map(|s| RIFTS[s.kind].cost).sum()
    }
    fn strain_of(&self, a: &Action, me: (i64, i64)) -> i64 {
        let mut s = TECHS[a.tech].strain + STRAIN_TEAR * a.rifts.len() as i64;
        let mut prev = me;
        for r in a.rifts.iter().filter(|r| r.chain) {
            if dist(prev.0, prev.1, r.x, r.y) > LINK_SCALE {
                s += STRAIN_LONG;
            }
            prev = (r.x, r.y);
        }
        s
    }
    pub fn affordable(&self, a: &Action, me: (i64, i64)) -> bool {
        let cost = Core::cost(a);
        let reserve = if TECHS[a.tech].group == "defense" { 0 } else { self.table(&RESERVE) as usize };
        if self.charges.available() < cost + if cost > 0 { reserve } else { 0 } {
            return false;
        }
        if self.open_count() + a.rifts.len() > MAX_RIFTS as usize {
            return false;
        }
        if a.uses.iter().any(|u| self.rift(*u).is_none()) {
            return false;
        }
        let buffer = if self.gate(GATE_SURVIVAL) { STRAIN_BREAK } else { 0 };
        !self.gate(GATE_SAFETY) || self.strain + self.strain_of(a, me) + buffer <= self.table(&STRAIN_CAP)
    }
    fn think(&mut self, out: &mut Vec<Cmd>, t: &Tick) {
        let tick = t.tick;
        let enemies: Vec<Fighter> = t.enemies().cloned().collect();
        if enemies.is_empty() || enemies.iter().all(|e| dist(e.x, e.y, t.me.x, t.me.y) > 320000) {
            return;
        }
        let v = self.view(t, enemies);
        let k = (self.table(&TECH_OPTIONS) as usize).min(28);
        let mut order: Vec<usize> = (0..28).collect();
        for i in 0..k {
            let j = i + self.random((28 - i) as u64) as usize;
            order.swap(i, j);
        }
        let mut pool: Vec<usize> = order[..k].to_vec();
        if tick >= self.plan_until {
            self.plan.clear();
        }
        let next = self.plan.front().copied();
        if let Some(n) = next {
            if !pool.contains(&n) {
                pool.push(n);
            }
        }
        pool.sort();
        let noise = self.table(&NOISE_PCT);
        let me = (t.me.x, t.me.y);
        let mut best: Option<Action> = None;
        for tech in pool {
            for mut a in self.propose(t, &v, tech) {
                if !self.affordable(&a, me) {
                    continue;
                }
                if Some(tech) == next {
                    a.score += 30;
                }
                // deep planners like opening a whole known sequence from its first step
                let depth = self.table(&COMBO_DEPTH);
                if next.is_none() && depth >= 3 && a.score >= 20 && COMBOS.iter().any(|c| c[0] == tech) {
                    a.score += 4 * (depth - 1);
                }
                if noise > 0 {
                    a.score += self.random((2 * noise + 1) as u64) as i64 - noise;
                }
                if best.as_ref().is_none_or(|b| a.score > b.score) {
                    best = Some(a);
                }
            }
        }
        let Some(a) = best else { return };
        if a.score < 12 {
            return;
        }
        let tech = a.tech;
        self.execute(out, t, a);
        // sequencing: deeper athletes commit to the rest of a known combination
        let depth = self.table(&COMBO_DEPTH) as usize;
        if self.plan.front() == Some(&tech) {
            self.plan.pop_front();
        } else if depth > 1 && self.plan.is_empty() {
            for combo in COMBOS.iter() {
                if let Some(i) = combo.iter().position(|s| *s == tech) {
                    let rest: Vec<usize> = combo[i + 1..].iter().take(depth - 1).copied().collect();
                    if !rest.is_empty() {
                        self.plan = rest.into_iter().collect();
                        self.plan_until = tick + 360;
                        self.habit(match depth {
                            2 => "planned 2 steps ahead",
                            3 => "planned 3 steps ahead",
                            4 => "planned 4 steps ahead",
                            _ => "planned a 5-step sequence",
                        });
                        break;
                    }
                }
            }
        }
    }
    /// Carries out an action: spends charges and strain, opens rifts, or acts on existing ones.
    /// The Rift Lab and the hand-placed fixtures call this directly.
    pub fn execute(&mut self, out: &mut Vec<Cmd>, t: &Tick, a: Action) {
        let tick = t.tick;
        let cost = Core::cost(&a);
        if cost > 0 && !self.charges.spend(tick, cost) {
            return;
        }
        let strain = self.strain_of(&a, (t.me.x, t.me.y));
        self.add_strain(strain);
        self.stats.tech_uses[a.tech] += 1;
        self.habit(a.habit);
        out.push(Cmd::Log(format!("tick {tick} rank {} {} score {} rifts {} strain {}", RANKS[self.rank], TECHS[a.tech].name, a.score, a.rifts.len(), self.strain)));
        // every technique has its own cast flourish on Vesper
        let flourish = self.tier4();
        self.fx(out, format!("tech_{}_t{flourish}", TECHS[a.tech].id), t.me.x, t.me.y, Some(t.me.id), 24);
        let uids: Vec<u32> = a.rifts.iter().map(|s| self.open(out, t, s, a.tech, a.subject)).collect();
        if uids.len() >= 2 && a.tech != T_CHAIN && a.tech != T_BANK {
            let (first, last) = (uids[0], uids[uids.len() - 1]);
            if let Some(r) = self.rift_mut(first) {
                r.peer = Some(last);
            }
            if let Some(r) = self.rift_mut(last) {
                r.peer = Some(first);
            }
        }
        if a.tech == T_HOOK && uids.len() == 2 {
            if let Some(r) = self.rift_mut(uids[0]) {
                r.expires = r.opens + 90;
            }
        }
        if matches!(a.tech, T_SEEDTRAP | T_SINKTRAP) {
            self.stats.traps += 1;
        }
        if a.tech == T_LIFELINE {
            if let Some(r) = uids.first().and_then(|u| self.rift_mut(*u)) {
                r.expires = r.opens + 1200;
            }
        }
        let tier = self.tier4();
        match a.tech {
            T_IMPLODE => {
                if let Some(&u) = a.uses.first() {
                    self.implode(out, t, u);
                }
            }
            T_DOMINO => {
                for (i, &u) in a.uses.iter().enumerate() {
                    if i == 0 {
                        self.implode(out, t, u);
                    } else {
                        self.later.push(Later::Domino { at: tick + 15 * i, uid: u });
                    }
                }
            }
            T_CROSSFIRE => {
                self.crossfire_until = tick + 180;
                self.fx(out, format!("tech_crossfire_t{tier}"), t.me.x, t.me.y, Some(t.me.id), 30);
            }
            T_OVERLOAD => {
                let spend = self.strain.min(OVERLOAD_MAX);
                self.overload = spend;
                self.add_strain(-spend);
                self.fx(out, format!("tech_overload_t{tier}"), t.me.x, t.me.y, Some(t.me.id), 30);
            }
            T_BORROW => {
                if let (Some(&near), Some(&far)) = (a.uses.first(), uids.first()) {
                    let dim = self.rift(far).map(|r| r.dim);
                    if let (Some(d), Some(r)) = (dim, self.rift_mut(near)) {
                        r.borrowed = Some((d, tick + 120));
                    }
                }
            }
            T_STITCH => {
                if let (Some(&ua), Some(&ub)) = (a.uses.first(), a.uses.get(1)) {
                    let b = self.rift(ub).cloned();
                    if let (Some(b), Some(ra)) = (b, self.rift_mut(ua)) {
                        ra.x = (ra.x + b.x) / 2;
                        ra.y = (ra.y + b.y) / 2;
                        ra.radius = ra.radius * 14 / 10;
                        ra.dim2 = Some(b.dim);
                        ra.expires = ra.expires.max(b.expires);
                        let (x, y) = (ra.x, ra.y);
                        self.close(out, ub, "close");
                        self.fx(out, format!("tech_stitch_t{tier}"), x, y, None, 30);
                    }
                }
            }
            _ => {}
        }
    }
    /// Hand placement (Rift Lab and parity fixtures): the planner never runs.
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn place(&mut self, input: &Input, a: Action) -> Vec<Cmd> {
        let mut out = vec![];
        let Some(me) = input.fighters.iter().find(|f| f.id == input.me).cloned() else { return out };
        let t = Tick { tick: input.tick, me, fighters: input.fighters, wall: input.wall, attack_interval: input.attack_interval.max(1), stunned: input.stunned };
        self.fx_left = FX_PER_UPDATE as usize;
        self.execute(&mut out, &t, a);
        out
    }
    /// Vesper died: everything she owns closes. Deaths soon after a risky self-move count as self-inflicted.
    pub fn on_dead(&mut self, tick: usize) {
        if tick <= self.risky_until {
            self.stats.self_deaths += 1;
        }
        for r in self.rifts.iter_mut() {
            r.done = true;
        }
        self.rifts.clear();
        self.packets.clear();
        self.later.clear();
        self.dots.clear();
        self.aiming = false;
        self.phased = false;
        self.chain_target = None;
        self.plan.clear();
        self.overload = 0;
        self.damage.clear();
        self.controls.clear();
        self.moves.clear();
    }
}
