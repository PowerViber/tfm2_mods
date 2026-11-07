//! Grimhald's deterministic core: snapshot in (`Frame`), commands out (`Effect`). No host handles live here, so every
//! rule can be tested with hand-built frames; `grimhald.rs` builds the frame from the sim and applies the effects, and
//! `grimhald_cage.rs` (the match hook) holds everyone to the pentagon the ultimate raises.
//!
//! Kit (numbers in rules/grimhald.json → grimhald_data.rs):
//!   passive  Bloodrage: damage and move speed grow with missing HP (floor 30% HP); basics +20% on controlled enemies.
//!   S1       Iron Ghost: 1 s banished smoke walk (untargetable, no damage), recast ends it early, then a fading speed
//!            tail; cleanses CC on cast (22 s cooldown when it did, 14 s otherwise).
//!   S2       Stone Bulwark: fissure line (damage + slow) ending in a rock pillar (an `sbw:` wall); recast once to
//!            relocate the pillar (pushes nobody), leaving a slowing fissure trail.
//!   S3       Skull Bash: charge; the first enemy is knocked back and stunned. Into a wall: slam damage and a long stun.
//!            Into the pillar (enemy knocked into it, or Grimhald himself charging into it): the pillar shatters with
//!            an area knock-up.
//!   Ult      Warden's Verdict: axe thrown up to 480,000; landing knocks up the area. If it knocked up a champion, a
//!            pentagon cage rises round the axe and he leaps in; otherwise the axe returns (full cooldown).
use crate::grimhald_data as d;
use serde::{Serialize,Deserialize};
use std::collections::BTreeSet;

pub const WALL_HALF: i64 = 8_000;
pub const CELL: i64 = 32_000;
pub const CELLS: i64 = 30;
pub const MAP: i64 = 960_000;
const KNOCK_STEP_SAMPLES: i64 = 4;
pub const TOWER_RANGE: i64 = d::ULT_TURRET_RANGE;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Point {
    pub x: i64,
    pub y: i64,
}
impl Point {
    pub const fn new(x: i64, y: i64) -> Self {
        Point { x, y }
    }
    pub fn d2(self, o: Point) -> i128 {
        let (dx, dy) = ((self.x - o.x) as i128, (self.y - o.y) as i128);
        dx * dx + dy * dy
    }
    pub fn dist(self, o: Point) -> f64 {
        (self.d2(o) as f64).sqrt()
    }
    pub fn within(self, o: Point, r: i64) -> bool {
        self.d2(o) <= (r as i128) * (r as i128)
    }
    /// `len` from self toward `to` (self when they coincide).
    pub fn toward(self, to: Point, len: i64) -> Point {
        let dd = self.dist(to);
        if dd < 1.0 {
            return self;
        }
        let k = len as f64 / dd;
        Point::new(self.x + ((to.x - self.x) as f64 * k).round() as i64, self.y + ((to.y - self.y) as f64 * k).round() as i64)
    }
    pub fn lerp(self, to: Point, num: i64, den: i64) -> Point {
        let den = den.max(1);
        Point::new(self.x + (to.x - self.x) * num / den, self.y + (to.y - self.y) * num / den)
    }
    pub fn offset(self, dir: (f64, f64), len: f64) -> Point {
        Point::new(self.x + (dir.0 * len).round() as i64, self.y + (dir.1 * len).round() as i64)
    }
    pub fn clamp_map(self) -> Point {
        Point::new(self.x.clamp(1_000, MAP - 1_000), self.y.clamp(1_000, MAP - 1_000))
    }
}

pub fn dir(a: Point, b: Point) -> (f64, f64) {
    let (dx, dy) = ((b.x - a.x) as f64, (b.y - a.y) as f64);
    let l = dx.hypot(dy);
    if l < 1.0 {
        (1.0, 0.0)
    } else {
        (dx / l, dy / l)
    }
}

/// Distance² from p to the segment a–b.
pub fn seg_d2(p: Point, a: Point, b: Point) -> f64 {
    let (vx, vy) = ((b.x - a.x) as f64, (b.y - a.y) as f64);
    let l2 = vx * vx + vy * vy;
    let k = if l2 == 0.0 { 0.0 } else { (((p.x - a.x) as f64 * vx + (p.y - a.y) as f64 * vy) / l2).clamp(0.0, 1.0) };
    let (qx, qy) = (a.x as f64 + vx * k, a.y as f64 + vy * k);
    let (dx, dy) = (p.x as f64 - qx, p.y as f64 - qy);
    dx * dx + dy * dy
}

/// A wall segment (the `sbw:` format: built from a to b, standing until `until`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Seg {
    pub a: Point,
    pub b: Point,
}
impl Seg {
    pub fn blocks(&self, p: Point, radius: i64) -> bool {
        seg_d2(p, self.a, self.b) < ((WALL_HALF + radius / 2) as f64).powi(2)
    }
    pub fn mid(&self) -> Point {
        Point::new((self.a.x + self.b.x) / 2, (self.a.y + self.b.y) / 2)
    }
}

// ------------------------------------------------------------------ the pentagon

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cage {
    pub owner: usize,
    pub center: Point,
    pub radius: i64,
    pub until: u64,
}

/// The five corners, the first straight up (screen y grows down), going round.
pub fn pentagon(center: Point, radius: i64) -> [Point; 5] {
    let mut out = [Point::default(); 5];
    for (k, o) in out.iter_mut().enumerate() {
        let a = -std::f64::consts::FRAC_PI_2 + k as f64 * 2.0 * std::f64::consts::PI / 5.0;
        *o = Point::new(center.x + (a.cos() * radius as f64).round() as i64, center.y + (a.sin() * radius as f64).round() as i64);
    }
    out
}
pub fn edges(center: Point, radius: i64) -> [Seg; 5] {
    let p = pentagon(center, radius);
    std::array::from_fn(|k| Seg { a: p[k], b: p[(k + 1) % 5] })
}
/// Signed distance from p to the pentagon's boundary: positive inside (how far in), negative outside.
pub fn depth(center: Point, radius: i64, p: Point) -> f64 {
    let pts = pentagon(center, radius);
    let mut m = f64::MAX;
    for k in 0..5 {
        let (a, b) = (pts[k], pts[(k + 1) % 5]);
        // inward normal: toward the centre
        let (ex, ey) = ((b.x - a.x) as f64, (b.y - a.y) as f64);
        let l = ex.hypot(ey).max(1.0);
        let (mut nx, mut ny) = (-ey / l, ex / l);
        if nx * (center.x - a.x) as f64 + ny * (center.y - a.y) as f64 <= 0.0 {
            nx = -nx;
            ny = -ny;
        }
        let s = nx * (p.x - a.x) as f64 + ny * (p.y - a.y) as f64;
        m = m.min(s);
    }
    m
}
pub fn inside(center: Point, radius: i64, p: Point, margin: i64) -> bool {
    depth(center, radius, p) >= margin as f64
}
/// The nearest point at least `margin` inside the pentagon (p itself when it already is).
pub fn clamp_inside(center: Point, radius: i64, p: Point, margin: i64) -> Point {
    if inside(center, radius, p, margin) {
        return p;
    }
    // walk from p toward the centre until deep enough (convex, so the first such point is the nearest along it)
    let total = p.dist(center);
    let mut lo = 0.0f64;
    let mut hi = total;
    for _ in 0..24 {
        let mid = (lo + hi) / 2.0;
        let q = p.toward(center, mid as i64);
        if inside(center, radius, q, margin) {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    p.toward(center, hi.ceil() as i64)
}
/// The nearest point at least `margin` outside the pentagon, straight out from the centre.
pub fn push_outside(center: Point, radius: i64, p: Point, margin: i64) -> Point {
    if depth(center, radius, p) <= -(margin as f64) {
        return p;
    }
    let d = if p == center { Point::new(center.x + 1, center.y) } else { p };
    let mut lo = 0.0f64;
    let mut hi = (radius + margin * 2 + 2_000) as f64;
    for _ in 0..24 {
        let mid = (lo + hi) / 2.0;
        let q = center.toward(d, mid as i64);
        if depth(center, radius, q) <= -(margin as f64) {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    center.toward(d, hi.ceil() as i64)
}

// ------------------------------------------------------------------ frame and effects

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Unit {
    pub id: usize,
    pub team: usize,
    pub pos: Point,
    pub hp: i64,
    pub max_hp: i64,
    pub radius: i64,
    pub champion: bool,
    pub tower: bool,
    pub visible: bool,
    pub targetable: bool,
    /// fair game for farming (farming::Context: eligible and calm)
    pub farmable: bool,
    /// knocked up, stunned or otherwise held (for the bruiser basic bonus and bash choices)
    pub controlled: bool,
    pub velocity: Point,
    /// the cage (owner id) this champion is held in
    pub caged_by: Option<usize>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Frame {
    pub tick: u64,
    pub owner: usize,
    pub team: usize,
    pub alive: bool,
    pub pos: Point,
    pub hp: i64,
    pub max_hp: i64,
    pub attack: i64,
    pub move_speed: i64,
    /// ticks left of the longest hard CC on him (stun, knock-up, root…), his own named locks excluded
    pub hard_cc_ticks: u64,
    pub ult_unlocked: bool,
    pub units: Vec<Unit>,
    /// terrain cells (30 x 30 of 32,000), index cy * 30 + cx; empty = no terrain known
    pub terrain: Vec<bool>,
    /// every standing `sbw:` wall that isn't his own pillar (Steve, Scribble, cages…)
    pub walls: Vec<Seg>,
    pub profile: [i64; 13],
    /// an enemy danger zone (ult, warned skill) covers him
    pub danger: bool,
    /// the cage he stands in, if any
    pub cage: Option<Cage>,
    /// AI off: tests drive the kit with `order`
    pub manual: bool,
    pub order: Option<Order>,
}

/// A command a test (or a future Lab) can give instead of the AI.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag="kind",content="aim",rename_all="snake_case")]
pub enum Order {
    Ghost(Point),
    GhostEnd,
    Bulwark(Point),
    Relocate(Point),
    Bash(Point),
    Ult(Point),
    Steer(Point),
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag="kind",content="data",rename_all="snake_case")]
pub enum Effect {
    Damage { target: usize, amount: i64 },
    Airborne { target: usize, ticks: u64 },
    Stun { target: usize, ticks: u64 },
    Slow { target: usize, percent: i64, ticks: u64 },
    /// move a unit (knockbacks); `hold` keeps it from acting meanwhile
    Place { target: usize, pos: Point, hold: u64 },
    /// move himself (ghost walk, charge, leap)
    Move { pos: Point },
    Banish { ticks: u64 },
    Invulnerable { ticks: u64 },
    ClearCc,
    Speed { percent: i64, ticks: u64 },
    Passive { damage_pct: i64, move_pct: i64, tier: u8 },
    WallUp { a: Point, b: Point, rise: u64, until: u64 },
    WallDown { a: Point, b: Point },
    CageUp { cage: Cage },
    CageDown,
    Fx { name: String, at: Point, ticks: u64, ambient: bool },
    Pose { name: &'static str, ticks: u64 },
    Log(String),
}

// ------------------------------------------------------------------ state

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Ghost {
    pub start: u64,
    pub until: u64,
    pub dest: Point,
    pub cleansed: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Pillar {
    pub seg: Seg,
    pub center: Point,
    pub until: u64,
    pub relocated: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Trail {
    pub a: Point,
    pub b: Point,
    pub until: u64,
}

#[derive(Clone, Debug, Serialize)]
pub struct Rupture {
    pub from: Point,
    pub to: Point,
    pub start: u64,
    pub arrival: u64,
    pub pillar_at: u64,
    pub last: Point,
    pub hits: BTreeSet<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Bash {
    pub from: Point,
    pub to: Point,
    pub t0: u64,
    pub t1: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Knock {
    pub target: usize,
    pub radius: i64,
    pub from: Point,
    pub to: Point,
    pub t0: u64,
    pub t1: u64,
    pub last: Point,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(tag="kind",rename_all="snake_case")]
pub enum Ult {
    Idle,
    Windup { until: u64, aim: Point, target: Option<usize> },
    Flight { from: Point, to: Point, at: Point, heading_mdeg:i64, spent_mdeg:i64, target:Option<usize>, guide:Option<Point>, t0:u64, t1:u64, last_tick:u64 },
    Return { from: Point, to: Point, t0: u64, t1: u64 },
    Leap { from: Point, to: Point, t0: u64, t1: u64, cage_until: u64 },
    Caged { center: Point, until: u64 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(tag="kind",rename_all="snake_case")]
pub enum Cast {
    /// Stone Bulwark's slam before the fissure runs
    Bulwark { until: u64, aim: Point },
}

/// Observable projection from the completed flight state, not hidden target data.
/// Future steering may update this warning; it never moves the projectile itself.
pub fn ult_projection(ult:Ult)->Option<(Point,u64)> {
    match ult {
        Ult::Windup{aim,until,..}=>Some((aim,until+sec(d::ULT_FLIGHT_TICKS))),
        Ult::Flight{from,to,at,heading_mdeg,t0,t1,last_tick,..}=>{
            let remaining=from.dist(to)*t1.saturating_sub(last_tick) as f64/(t1-t0).max(1) as f64;
            let angle=(heading_mdeg as f64/1000.).to_radians();
            Some((at.offset((angle.cos(),angle.sin()),remaining).clamp_map(),t1))
        },_=>None
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Core {
    /// tick each skill is ready again: [S1, S2, S3, ult]
    pub ready: [u64; 4],
    pub ghost: Option<Ghost>,
    pub tail: Option<(u64, u64)>,
    pub cast: Option<Cast>,
    pub rupture: Option<Rupture>,
    pub pillar: Option<Pillar>,
    pub trails: Vec<Trail>,
    pub bash: Option<Bash>,
    pub knocks: Vec<Knock>,
    pub ult: Ult,
    pub passive: (i64, i64, u8),
    pub rejection: String,
    /// the last skill used, for logs and tests
    pub last_skill: &'static str,
}
impl Default for Core {
    fn default() -> Self {
        Core {
            ready: [0; 4],
            ghost: None,
            tail: None,
            cast: None,
            rupture: None,
            pillar: None,
            trails: vec![],
            bash: None,
            knocks: vec![],
            ult: Ult::Idle,
            passive: (-1, -1, 0),
            rejection: String::new(),
            last_skill: "",
        }
    }
}

fn pct(base: i64, ratio: i64, attack: i64) -> i64 {
    base + ratio * attack / 100
}
fn sec(t: i64) -> u64 {
    t.max(0) as u64
}

impl Frame {
    pub fn terrain_at(&self, p: Point) -> bool {
        if p.x < 0 || p.y < 0 || p.x >= MAP || p.y >= MAP {
            return true;
        }
        let (cx, cy) = (p.x / CELL, p.y / CELL);
        self.terrain.get((cy * CELLS + cx) as usize).copied().unwrap_or(false)
    }
    fn enemy(&self, u: &Unit) -> bool {
        u.team != self.team && u.visible && u.targetable && u.hp > 0
    }
    pub fn enemy_champions(&self) -> impl Iterator<Item = &Unit> {
        self.units.iter().filter(move |u| u.champion && self.enemy(u))
    }
    fn can_reach(&self, u: &Unit) -> bool {
        match self.cage {
            Some(c) => u.caged_by == Some(c.owner),
            None => u.caged_by.is_none(),
        }
    }
    fn profile(&self, i: usize) -> i64 {
        self.profile.get(i).copied().unwrap_or(50).clamp(0, 100)
    }
}

/// Deterministic 0..n from the tick and a salt (aim error and tie-breaks without an RNG).
fn jitter(tick: u64, salt: u64, n: i64) -> i64 {
    let mut h = tick.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ salt.wrapping_mul(0xC2B2_AE3D_27D4_EB4F);
    h ^= h >> 31;
    (h % (n.max(1) as u64)) as i64
}

pub const PROFILE_JUDGEMENT: usize = 1;
pub const PROFILE_SKILL_HIT: usize = 4;
pub const PROFILE_AGGRESSIVE: usize = 6;

impl Core {
    pub fn respawn(&mut self, tick: u64) {
        let ready = self.ready;
        *self = Core::default();
        self.ready = ready.map(|r| r.max(tick));
    }

    pub fn busy(&self) -> bool {
        self.ghost.is_some() || self.cast.is_some() || self.bash.is_some() || matches!(self.ult, Ult::Windup { .. } | Ult::Leap { .. })
    }

    pub fn ready(&self, slot: usize, tick: u64) -> bool {
        tick >= self.ready[slot]
    }

    /// The pillar as a wall segment, if one stands.
    pub fn pillar_seg(&self) -> Option<Seg> {
        self.pillar.map(|p| p.seg)
    }

    pub fn step(&mut self, f: &Frame) -> Vec<Effect> {
        let mut out = vec![];
        if !f.alive {
            return out;
        }
        self.passive(f, &mut out);
        self.expire(f, &mut out);
        self.run_ghost(f, &mut out);
        self.run_bulwark(f, &mut out);
        self.run_bash(f, &mut out);
        self.run_knocks(f, &mut out);
        self.run_trails(f, &mut out);
        self.run_ult(f, &mut out);
        let order = if f.manual { f.order } else { self.decide(f) };
        if let Some(o) = order {
            self.execute(f, o, &mut out);
        }
        out
    }

    // ---------------------------------------------------------------- passive

    fn passive(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        let hp_pct = (f.hp * 100 / f.max_hp.max(1)).clamp(0, 100);
        let missing = (100 - hp_pct.max(d::PASSIVE_FLOOR_HP_PCT)).max(0);
        let dmg = missing * d::PASSIVE_PER_MISSING_PCT_DAMAGE / 100;
        let ms = missing * d::PASSIVE_PER_MISSING_PCT_MOVE / 100;
        let tier = d::PASSIVE_TIERS.iter().filter(|&&t| hp_pct <= t).count() as u8;
        if (dmg, ms, tier) != self.passive {
            self.passive = (dmg, ms, tier);
            out.push(Effect::Passive { damage_pct: dmg, move_pct: ms, tier });
        }
    }

    fn expire(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        if let Some(p) = self.pillar {
            if f.tick >= p.until {
                out.push(Effect::WallDown { a: p.seg.a, b: p.seg.b });
                out.push(Effect::Fx { name: "grh_pillar_break".into(), at: p.center, ticks: 20, ambient: true });
                self.pillar = None;
            }
        }
        self.trails.retain(|t| f.tick < t.until);
        if let Some((_, until)) = self.tail {
            if f.tick >= until {
                self.tail = None;
            }
        }
    }

    // ---------------------------------------------------------------- S1 Iron Ghost

    fn start_ghost(&mut self, f: &Frame, dest: Point, out: &mut Vec<Effect>) -> bool {
        if !self.ready(0, f.tick) || self.ghost.is_some() {
            return false;
        }
        let dest = f.pos.toward(dest, (f.pos.dist(dest) as i64).min(d::GHOST_MAX_DISTANCE)).clamp_map();
        let cleansed = f.hard_cc_ticks > 0;
        if cleansed {
            out.push(Effect::ClearCc);
        }
        // a recast is free: the cooldown starts now, longer when it cleansed
        let cd = if cleansed { d::GHOST_CLEANSE_COOLDOWN_TICKS } else { d::GHOST_COOLDOWN_TICKS };
        self.ready[0] = f.tick + sec(cd);
        // whatever he was doing is dropped (a cleanse out of a bash or a cast)
        self.cast = None;
        self.bash = None;
        self.ghost = Some(Ghost { start: f.tick, until: f.tick + sec(d::GHOST_TICKS), dest, cleansed });
        out.push(Effect::Banish { ticks: 6 });
        out.push(Effect::Invulnerable { ticks: 7 });
        out.push(Effect::Fx { name: "grh_ghost".into(), at: f.pos, ticks: 18, ambient: false });
        self.last_skill = "ghost";
        true
    }

    fn end_ghost(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        let Some(_) = self.ghost.take() else { return };
        out.push(Effect::Fx { name: "grh_ghost_end".into(), at: f.pos, ticks: 18, ambient: false });
        // the fading tail starts the moment he reappears
        self.tail = Some((f.tick, f.tick + sec(d::GHOST_TAIL_TICKS)));
        out.push(Effect::Speed { percent: d::GHOST_TAIL_SPEED_PCT, ticks: sec(d::GHOST_TAIL_TICKS) });
        // reappearing next to an enemy readies the bash sooner
        if f.enemy_champions().any(|e| e.pos.within(f.pos, d::GHOST_RESET_RANGE + e.radius)) {
            self.ready[2] = self.ready[2].saturating_sub(sec(d::GHOST_RESET_TICKS)).max(f.tick.min(self.ready[2]));
        }
    }

    fn run_ghost(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        let Some(g) = self.ghost else { return };
        if f.tick >= g.until {
            self.end_ghost(f, out);
            return;
        }
        // banished in short slices, renewed while the ghost lasts: ending it (recast) just stops renewing
        if f.tick > g.start && (f.tick - g.start) % 6 == 0 {
            let left = (g.until - f.tick).min(6);
            out.push(Effect::Banish { ticks: left });
            out.push(Effect::Invulnerable { ticks: left + 1 });
        }
        let speed = f.move_speed.max(600) * (100 + d::GHOST_GHOST_SPEED_PCT) / 100;
        if !f.pos.within(g.dest, 1_000) {
            let mut next = f.pos.toward(g.dest, speed.min(f.pos.dist(g.dest) as i64));
            // walls still stop him (his own cage too)
            if self.blocked(f, next, 6_000, None) {
                next = f.pos;
            }
            if let Some(c) = f.cage {
                next = clamp_inside(c.center, c.radius, next, WALL_HALF + 3_000);
            }
            out.push(Effect::Move { pos: next });
            if (f.tick - g.start) % 4 == 0 {
                out.push(Effect::Fx { name: "grh_ghost".into(), at: next, ticks: 12, ambient: true });
            }
        }
    }

    // ---------------------------------------------------------------- S2 Stone Bulwark

    fn start_bulwark(&mut self, f: &Frame, aim: Point, out: &mut Vec<Effect>) -> bool {
        if !self.ready(1, f.tick) || self.busy() {
            return false;
        }
        // a new fissure replaces the old pillar
        if let Some(p) = self.pillar.take() {
            out.push(Effect::WallDown { a: p.seg.a, b: p.seg.b });
        }
        self.ready[1] = f.tick + sec(d::BULWARK_COOLDOWN_TICKS);
        self.cast = Some(Cast::Bulwark { until: f.tick + sec(d::BULWARK_CAST_TICKS), aim });
        out.push(Effect::Pose { name: "skill1", ticks: sec(d::BULWARK_CAST_TICKS) });
        self.last_skill = "bulwark";
        true
    }

    /// The fissure from `from` toward `aim`: where it ends (early at terrain or a wall).
    pub fn fissure_end(&self, f: &Frame, from: Point, aim: Point) -> Point {
        let full = from.toward(aim, d::BULWARK_LENGTH);
        let n = 19;
        let mut last = from;
        for k in 1..=n {
            let p = from.lerp(full, k, n);
            if f.terrain_at(p) || f.walls.iter().any(|w| w.blocks(p, 0)) {
                return last;
            }
            last = p;
        }
        full
    }

    fn pillar_at(center: Point, along: (f64, f64)) -> Seg {
        // across the line
        let n = (-along.1, along.0);
        let h = d::BULWARK_PILLAR_HALF as f64;
        Seg { a: center.offset(n, -h), b: center.offset(n, h) }
    }

    fn run_bulwark(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        if let Some(Cast::Bulwark { until, aim })=self.cast {
            if f.tick>=until {
                self.cast=None;
                let to=self.fissure_end(f,f.pos,aim);
                self.rupture=Some(Rupture { from:f.pos,to,start:f.tick,arrival:f.tick+sec(d::BULWARK_FISSURE_TICKS),pillar_at:f.tick+sec(d::BULWARK_FISSURE_TICKS+d::BULWARK_PILLAR_DELAY_TICKS),last:f.pos,hits:BTreeSet::new() });
            }
        }
        let Some(mut r)=self.rupture.take() else{return};
        let age=f.tick.saturating_sub(r.start).min(r.arrival-r.start);
        let front=r.from.lerp(r.to,age as i64,(r.arrival-r.start) as i64);
        if f.tick<=r.arrival {
            let dmg=pct(d::BULWARK_BASE,d::BULWARK_RATIO,f.attack);
            for u in f.units.iter().filter(|u|f.enemy(u)&&f.can_reach(u)&&!u.tower) {
                if !r.hits.contains(&u.id)&&seg_d2(u.pos,r.last,front)<=((d::BULWARK_WIDTH+u.radius) as f64).powi(2) {
                    r.hits.insert(u.id);
                    out.push(Effect::Damage{target:u.id,amount:dmg});
                    out.push(Effect::Slow{target:u.id,percent:d::BULWARK_SLOW_PCT,ticks:sec(d::BULWARK_SLOW_TICKS)});
                }
            }
            if f.tick%3==0 {out.push(Effect::Fx{name:"grh_fissure".into(),at:front,ticks:18,ambient:false});}
            r.last=front;
        }
        if f.tick>=r.pillar_at {self.raise_pillar(f,r.to,dir(r.from,r.to),out);}
        else {self.rupture=Some(r);}
    }

    fn raise_pillar(&mut self, f: &Frame, center: Point, along: (f64, f64), out: &mut Vec<Effect>) {
        // never on top of a champion: shift along the line to the nearest free spot
        let mut c = center;
        for k in 0..6 {
            let seg = Self::pillar_at(c, along);
            if !f.units.iter().any(|u| u.champion && u.hp > 0 && seg.blocks(u.pos, u.radius)) && !seg.blocks(f.pos, 10_000) {
                break;
            }
            let s = if k % 2 == 0 { 1.0 } else { -1.0 };
            c = center.offset(along, s * 6_000.0 * (k / 2 + 1) as f64);
        }
        let until = self.pillar.map_or(f.tick + sec(d::BULWARK_PILLAR_TICKS), |p| p.until);
        let seg = Self::pillar_at(c, along);
        self.pillar = Some(Pillar { seg, center: c, until, relocated: self.pillar.is_some_and(|p| p.relocated) });
        out.push(Effect::WallUp { a: seg.a, b: seg.b, rise: 4, until });
        out.push(Effect::Fx { name: "grh_pillar_rise".into(), at: c, ticks: 18, ambient: false });
    }

    fn relocate(&mut self, f: &Frame, to: Point, out: &mut Vec<Effect>) -> bool {
        let Some(p) = self.pillar else { return false };
        if p.relocated {
            return false;
        }
        let to = p.center.toward(to, (p.center.dist(to) as i64).min(d::BULWARK_RELOCATE_RANGE)).clamp_map();
        if f.terrain_at(to) {
            return false;
        }
        out.push(Effect::WallDown { a: p.seg.a, b: p.seg.b });
        // the fissure trail along the way: slows, never pushes
        self.trails.push(Trail { a: p.center, b: to, until: f.tick + sec(d::BULWARK_TRAIL_TICKS) });
        let steps = (p.center.dist(to) / 7_600.0).ceil().max(1.0) as i64;
        for k in 0..=steps {
            out.push(Effect::Fx { name: "grh_trail".into(), at: p.center.lerp(to, k, steps), ticks: 40, ambient: k % 2 == 1 });
        }
        let along = { let s = dir(p.seg.a, p.seg.b); (s.1, -s.0) };
        self.pillar = Some(Pillar { relocated: true, ..p });
        self.raise_pillar(f, to, along, out);
        self.last_skill = "relocate";
        true
    }

    fn run_trails(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        if self.trails.is_empty() || f.tick % 10 != 0 {
            return;
        }
        for u in f.units.iter().filter(|u| f.enemy(u) && f.can_reach(u) && !u.tower) {
            if self.trails.iter().any(|t| seg_d2(u.pos, t.a, t.b) <= ((d::BULWARK_TRAIL_WIDTH + u.radius) as f64).powi(2)) {
                out.push(Effect::Slow { target: u.id, percent: d::BULWARK_TRAIL_SLOW_PCT, ticks: 14 });
            }
        }
    }

    fn shatter(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        let Some(p) = self.pillar.take() else { return };
        out.push(Effect::WallDown { a: p.seg.a, b: p.seg.b });
        out.push(Effect::Fx { name: "grh_pillar_break".into(), at: p.center, ticks: 24, ambient: false });
        let dmg = pct(d::BULWARK_SHATTER_BASE, d::BULWARK_SHATTER_RATIO, f.attack);
        for u in f.units.iter().filter(|u| f.enemy(u) && f.can_reach(u) && !u.tower) {
            if u.pos.within(p.center, d::BULWARK_SHATTER_RADIUS + u.radius) {
                out.push(Effect::Damage { target: u.id, amount: dmg });
                out.push(Effect::Airborne { target: u.id, ticks: sec(d::BULWARK_SHATTER_TICKS) });
            }
        }
        out.push(Effect::Log(format!("pillar shattered at {},{}", p.center.x, p.center.y)));
    }

    // ---------------------------------------------------------------- S3 Skull Bash

    /// Is p blocked (terrain, any wall, his pillar)? Returns which: Some(true) = his pillar.
    fn blocked(&self, f: &Frame, p: Point, radius: i64, cage: Option<Cage>) -> bool {
        self.wall_hit(f, p, radius, cage).is_some()
    }
    fn wall_hit(&self, f: &Frame, p: Point, radius: i64, cage: Option<Cage>) -> Option<bool> {
        if self.pillar.is_some_and(|pl| pl.seg.blocks(p, radius)) {
            return Some(true);
        }
        if f.terrain_at(p) || f.walls.iter().any(|w| w.blocks(p, radius)) {
            return Some(false);
        }
        if let Some(c) = cage {
            if !inside(c.center, c.radius, p, WALL_HALF / 2) {
                return Some(false);
            }
        }
        None
    }

    fn start_bash(&mut self, f: &Frame, aim: Point, out: &mut Vec<Effect>) -> bool {
        if !self.ready(2, f.tick) || self.busy() {
            return false;
        }
        self.ready[2] = f.tick + sec(d::BASH_COOLDOWN_TICKS);
        let to = f.pos.toward(aim, d::BASH_RANGE);
        self.bash = Some(Bash { from: f.pos, to, t0: f.tick, t1: f.tick + sec(d::BASH_DASH_TICKS) });
        out.push(Effect::Pose { name: "skill2", ticks: sec(d::BASH_DASH_TICKS) });
        self.last_skill = "bash";
        true
    }

    fn run_bash(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        let Some(b) = self.bash else { return };
        let k = (f.tick.saturating_sub(b.t0) + 1).min(b.t1 - b.t0) as i64;
        let next = b.from.lerp(b.to, k, (b.t1 - b.t0) as i64);
        // charging straight into his own pillar shatters it
        match self.wall_hit(f, next, 8_000, f.cage) {
            Some(true) => {
                self.bash = None;
                out.push(Effect::Fx { name: "grh_slam".into(), at: f.pos, ticks: 20, ambient: false });
                self.shatter(f, out);
                return;
            }
            Some(false) => {
                self.bash = None;
                return;
            }
            None => {}
        }
        out.push(Effect::Move { pos: next });
        if k % 3 == 0 {
            out.push(Effect::Fx { name: "grh_bash_dust".into(), at: f.pos, ticks: 14, ambient: true });
        }
        // the first enemy in his way: champions first, then anything farmable
        let along = dir(b.from, b.to);
        let hit = f
            .units
            .iter()
            .filter(|u| f.enemy(u) && f.can_reach(u) && !u.tower && (u.champion || u.farmable))
            .filter(|u| u.pos.within(next, u.radius + 9_000))
            .min_by_key(|u| (!u.champion, u.pos.d2(next), u.id));
        if let Some(u) = hit {
            self.bash = None;
            out.push(Effect::Damage { target: u.id, amount: pct(d::BASH_BASE, d::BASH_RATIO, f.attack) });
            let to = u.pos.offset(along, d::BASH_KNOCKBACK as f64);
            self.knocks.push(Knock { target: u.id, radius: u.radius, from: u.pos, to, t0: f.tick, t1: f.tick + sec(d::BASH_KNOCK_TICKS), last: u.pos });
            out.push(Effect::Stun { target: u.id, ticks: sec(d::BASH_KNOCK_TICKS) + 1 });
        } else if f.tick >= b.t1 {
            self.bash = None;
        }
    }

    fn run_knocks(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        let knocks = std::mem::take(&mut self.knocks);
        for mut k in knocks {
            let Some(u) = f.units.iter().find(|u| u.id == k.target && u.hp > 0) else { continue };
            let n = (f.tick.saturating_sub(k.t0) + 1).min(k.t1 - k.t0) as i64;
            let next = k.from.lerp(k.to, n, (k.t1 - k.t0) as i64);
            // sample between last and next so fast knockbacks don't tunnel through thin walls
            let mut stop: Option<(Point, bool)> = None;
            for s in 1..=KNOCK_STEP_SAMPLES {
                let p = k.last.lerp(next, s, KNOCK_STEP_SAMPLES);
                if let Some(pillar) = self.wall_hit(f, p, u.radius.min(8_000), f.units.iter().find(|x| x.id == u.id).and(cage_of(f, u))) {
                    stop = Some((k.last.lerp(next, s - 1, KNOCK_STEP_SAMPLES), pillar));
                    break;
                }
            }
            if let Some((at, pillar)) = stop {
                out.push(Effect::Place { target: k.target, pos: at, hold: 1 });
                out.push(Effect::Damage { target: k.target, amount: pct(d::BASH_SLAM_BASE, d::BASH_SLAM_RATIO, f.attack) });
                out.push(Effect::Fx { name: "grh_slam".into(), at, ticks: 20, ambient: false });
                if pillar {
                    self.shatter(f, out);
                } else {
                    out.push(Effect::Stun { target: k.target, ticks: sec(d::BASH_SLAM_STUN_TICKS) });
                }
                continue;
            }
            out.push(Effect::Place { target: k.target, pos: next, hold: 2 });
            k.last = next;
            if f.tick + 1 >= k.t1 {
                out.push(Effect::Stun { target: k.target, ticks: sec(d::BASH_LAND_STUN_TICKS) });
                continue;
            }
            self.knocks.push(k);
        }
    }

    // ---------------------------------------------------------------- Ult: Warden's Verdict

    fn start_ult(&mut self, f: &Frame, aim: Point, out: &mut Vec<Effect>) -> bool {
        if !f.ult_unlocked || !self.ready(3, f.tick) || self.busy() || self.ult != Ult::Idle || f.cage.is_some() {
            return false;
        }
        let aim = f.pos.toward(aim, (f.pos.dist(aim) as i64).min(d::ULT_RANGE)).clamp_map();
        self.ready[3] = f.tick + sec(d::ULT_COOLDOWN_TICKS);
        let target=f.enemy_champions().filter(|u|f.can_reach(u)&&u.pos.within(aim,d::ULT_KNOCK_RADIUS+u.radius)).min_by_key(|u|(u.pos.d2(aim),u.id)).map(|u|u.id);
        self.ult = Ult::Windup { until: f.tick + sec(d::ULT_WINDUP_TICKS), aim, target };
        out.push(Effect::Pose { name: "ult", ticks: sec(d::ULT_WINDUP_TICKS) });
        out.push(Effect::Log(format!("ult thrown at {},{}", aim.x, aim.y)));
        self.last_skill = "ult";
        true
    }

    fn run_ult(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        match self.ult {
            Ult::Idle => {}
            Ult::Windup { until, aim, target } => {
                if f.tick >= until {
                    let flight = sec(d::ULT_FLIGHT_TICKS).max(1);
                    let heading=((aim.y-f.pos.y) as f64).atan2((aim.x-f.pos.x) as f64).to_degrees()*1000.;
                    self.ult = Ult::Flight {from:f.pos,to:aim,at:f.pos,heading_mdeg:heading.round() as i64,spent_mdeg:0,target,guide:None,t0:f.tick,t1:f.tick+flight,last_tick:f.tick};
                }
            }
            Ult::Flight {from,to,mut at,mut heading_mdeg,mut spent_mdeg,target,guide:manual_guide,t0,t1,last_tick} => {
                let guide=manual_guide.unwrap_or_else(||target.and_then(|id|f.units.iter().find(|u|u.id==id&&f.enemy(u)&&f.can_reach(u))).map_or(to,|u|u.pos));
                let end=f.tick.min(t1);
                for step in last_tick..end {
                    let desired=((guide.y-at.y) as f64).atan2((guide.x-at.x) as f64).to_degrees()*1000.;
                    let delta=(desired.round() as i64-heading_mdeg+180000).rem_euclid(360000)-180000;
                    let budget=(d::ULT_CURVE_MILLI_DEGREES-spent_mdeg).max(0).min(d::ULT_CURVE_RATE_MILLI_DEGREES);
                    let turn=delta.clamp(-budget,budget);
                    heading_mdeg+=turn;spent_mdeg+=turn.abs();
                    let len=from.dist(to)*(step-t0+1) as f64/(t1-t0) as f64-from.dist(to)*(step-t0) as f64/(t1-t0) as f64;
                    let rad=(heading_mdeg as f64/1000.).to_radians();
                    at=at.offset((rad.cos(),rad.sin()),len).clamp_map();
                }
                out.push(Effect::Fx{name:format!("grh_axe_p{}",f.tick%8),at,ticks:1,ambient:false});
                if f.tick>=t1 {self.land(f,at,out);}
                else {self.ult=Ult::Flight{from,to,at,heading_mdeg,spent_mdeg,target,guide:manual_guide,t0,t1,last_tick:end};}
            }
            Ult::Return { from, to, t0, t1 } => {
                let at = from.lerp(to, (f.tick - t0) as i64, (t1 - t0) as i64);
                out.push(Effect::Fx { name: format!("grh_axe_p{}", 7 - f.tick % 8), at, ticks: 1, ambient: false });
                if f.tick >= t1 {
                    self.ult = Ult::Idle;
                }
            }
            Ult::Leap { from, to, t0, t1, cage_until } => {
                let n = (f.tick - t0 + 1).min(t1 - t0) as i64;
                out.push(Effect::Move { pos: from.lerp(to, n, (t1 - t0) as i64) });
                out.push(Effect::Invulnerable { ticks: 2 });
                if f.tick + 1 >= t1 {
                    out.push(Effect::Fx { name: "grh_land".into(), at: to, ticks: 24, ambient: false });
                    self.ult = Ult::Caged { center: to, until: cage_until };
                }
            }
            Ult::Caged { until, .. } => {
                if f.tick >= until {
                    out.push(Effect::CageDown);
                    self.ult = Ult::Idle;
                }
            }
        }
    }

    fn land(&mut self, f: &Frame, at: Point, out: &mut Vec<Effect>) {
        out.push(Effect::Fx { name: "grh_land".into(), at, ticks: 30, ambient: false });
        let dmg = pct(d::ULT_BASE, d::ULT_RATIO, f.attack);
        let mut knocked_champion = false;
        for u in f.units.iter().filter(|u| u.team != f.team && u.targetable && u.hp > 0 && !u.tower && u.caged_by.is_none()) {
            if u.pos.within(at, d::ULT_KNOCK_RADIUS + u.radius) {
                out.push(Effect::Damage { target: u.id, amount: dmg });
                out.push(Effect::Airborne { target: u.id, ticks: sec(d::ULT_KNOCK_TICKS) });
                knocked_champion |= u.champion;
            }
        }
        if !knocked_champion {
            // nobody to cage: the axe flies home, the cooldown already runs in full
            let back = sec(d::ULT_FLIGHT_TICKS / 2).max(1);
            self.ult = Ult::Return { from: at, to: f.pos, t0: f.tick, t1: f.tick + back };
            out.push(Effect::Log("ult missed: axe returns".into()));
            return;
        }
        let until = f.tick + sec(d::ULT_CAGE_RISE_TICKS + d::ULT_CAGE_TICKS);
        let cage = Cage { owner: f.owner, center: at, radius: d::ULT_CAGE_RADIUS, until };
        out.push(Effect::CageUp { cage });
        for (k, e) in edges(at, d::ULT_CAGE_RADIUS).iter().enumerate() {
            out.push(Effect::WallUp { a: e.a, b: e.b, rise: sec(d::ULT_CAGE_RISE_TICKS), until });
            out.push(Effect::Fx { name: "grh_cage_rise".into(), at: e.mid(), ticks: 20, ambient: k % 2 == 1 });
        }
        // he leaps to the axe (it lands on free ground, so he does too)
        let to = at;
        out.push(Effect::Fx { name: "grh_leap".into(), at: f.pos, ticks: 18, ambient: false });
        self.ult = Ult::Leap { from: f.pos, to, t0: f.tick, t1: f.tick + sec(d::ULT_LEAP_TICKS), cage_until: until };
    }

    // ---------------------------------------------------------------- orders

    fn execute(&mut self, f: &Frame, o: Order, out: &mut Vec<Effect>) {
        let ok = match o {
            Order::Ghost(p) => self.start_ghost(f, p, out),
            Order::GhostEnd => {
                let had = self.ghost.is_some();
                self.end_ghost(f, out);
                had
            }
            Order::Bulwark(p) => self.start_bulwark(f, p, out),
            Order::Relocate(p) => self.relocate(f, p, out),
            Order::Bash(p) => self.start_bash(f, p, out),
            Order::Ult(p) => self.start_ult(f, p, out),
            Order::Steer(p) => {
                if let Ult::Flight {ref mut guide,ref mut target,..}=self.ult {
                    // Recast supplies a steering intention, never another cast or cooldown.
                    *guide=Some(p);*target=None;true
                } else {false}
            },
        };
        if !ok {
            self.rejection = format!("{o:?} rejected at tick {}", f.tick);
        }
    }

    // ---------------------------------------------------------------- AI

    fn decide(&mut self, f: &Frame) -> Option<Order> {
        // the ghost first: it is the only thing he can do while held
        if let Some(g) = self.ghost {
            if f.pos.within(g.dest, 3_000) || self.ghost_ready_to_strike(f) {
                return Some(Order::GhostEnd);
            }
            return None;
        }
        if f.hard_cc_ticks > 0 {
            let threatened = f.enemy_champions().filter(|e| e.pos.within(f.pos, 60_000)).count();
            let long = f.hard_cc_ticks >= d::GHOST_AI_MIN_CC_TICKS as u64;
            if self.ready(0, f.tick) && threatened > 0 && (long || f.hp * 100 <= f.max_hp * 35 || f.danger) {
                return Some(Order::Ghost(self.ghost_escape_point(f)));
            }
            return None;
        }
        if self.busy() || matches!(self.ult, Ult::Leap { .. }) {
            return None;
        }
        let low = f.hp * 100 <= f.max_hp * 35;
        if f.danger && self.ready(0, f.tick) {
            return Some(Order::Ghost(self.ghost_escape_point(f)));
        }
        if let Some(o) = self.decide_ult(f) {
            return Some(o);
        }
        if let Some(o) = self.decide_combo(f) {
            return Some(o);
        }
        if let Some(o) = self.decide_bulwark(f, low) {
            return Some(o);
        }
        if let Some(o) = self.decide_bash(f, low) {
            return Some(o);
        }
        if let Some(o) = self.decide_engage_ghost(f, low) {
            return Some(o);
        }
        self.decide_farm(f)
    }

    fn nearest_enemy<'a>(&self, f: &'a Frame, r: i64) -> Option<&'a Unit> {
        f.enemy_champions().filter(|e| f.can_reach(e) && e.pos.within(f.pos, r)).min_by_key(|e| (e.pos.d2(f.pos), e.id))
    }

    fn ghost_ready_to_strike(&self, f: &Frame) -> bool {
        // engage ghost: reappear as soon as a bash would connect
        self.ready(2, f.tick) && self.nearest_enemy(f, 9_000).is_some()
    }

    fn ghost_escape_point(&self, f: &Frame) -> Point {
        // away from the enemies' centre (or straight back toward the map centre if alone)
        let threats: Vec<Point> = f.enemy_champions().filter(|e| e.pos.within(f.pos, 80_000)).map(|e| e.pos).collect();
        let away = if threats.is_empty() {
            Point::new(MAP / 2, MAP / 2)
        } else {
            let n = threats.len() as i64;
            let c = Point::new(threats.iter().map(|p| p.x).sum::<i64>() / n, threats.iter().map(|p| p.y).sum::<i64>() / n);
            f.pos.offset(dir(c, f.pos), 60_000.0)
        };
        let mut dest = f.pos.toward(away, d::GHOST_MAX_DISTANCE).clamp_map();
        if let Some(c) = f.cage {
            dest = clamp_inside(c.center, c.radius, dest, WALL_HALF + 3_000);
        }
        dest
    }

    fn predict(u: &Unit, ticks: i64) -> Point {
        Point::new(u.pos.x + u.velocity.x * ticks, u.pos.y + u.velocity.y * ticks)
    }

    /// Ult: 2+ enemies in the knock-up, or one low enough to finish; never into turret range; not while nearly dead
    /// with nobody to help.
    fn decide_ult(&self, f: &Frame) -> Option<Order> {
        if !f.ult_unlocked || !self.ready(3, f.tick) || f.cage.is_some() {
            return None;
        }
        let lead = d::ULT_WINDUP_TICKS + d::ULT_FLIGHT_TICKS;
        let skill = f.profile(PROFILE_SKILL_HIT);
        let mut best: Option<(i64, Point)> = None;
        for e in f.enemy_champions().filter(|e| e.caged_by.is_none() && e.pos.within(f.pos, d::ULT_RANGE)) {
            let p = Self::predict(e, lead).clamp_map();
            let Some(p) = self.turret_safe(f, p) else { continue };
            let hit: Vec<&Unit> = f.enemy_champions().filter(|o| o.caged_by.is_none() && Self::predict(o, lead).within(p, d::ULT_KNOCK_RADIUS)).collect();
            let low = hit.iter().any(|o| o.hp * 100 <= o.max_hp * d::ULT_AI_EXECUTE_PCT);
            if hit.len() < d::ULT_AI_MIN_ENEMIES as usize && !low {
                continue;
            }
            let allies = f.units.iter().filter(|a| a.champion && a.team == f.team && a.hp > 0 && a.pos.within(p, 90_000)).count();
            if f.hp * 100 <= f.max_hp * d::ULT_AI_SELF_MIN_PCT && allies < 3 {
                continue;
            }
            let score = hit.len() as i64 * 100 + if low { 60 } else { 0 } + allies as i64 * 10 - (f.pos.dist(p) / 20_000.0) as i64;
            if best.is_none_or(|(s, _)| score > s) {
                best = Some((score, p));
            }
        }
        let (_, p) = best?;
        // aim error shrinks with skill_hit (0..100 → up to 12,000 off)
        let err = (100 - skill) * 120;
        let off = Point::new(jitter(f.tick, 1, err * 2 + 1) - err, jitter(f.tick, 2, err * 2 + 1) - err);
        Some(Order::Ult(Point::new(p.x + off.x, p.y + off.y).clamp_map()))
    }

    /// The landing point moved so the pentagon stays out of every turret's range, or None.
    pub fn turret_safe(&self, f: &Frame, p: Point) -> Option<Point> {
        let clear = |q: Point| f.units.iter().filter(|t| t.tower && t.hp > 0).all(|t| !t.pos.within(q, TOWER_RANGE + d::ULT_CAGE_RADIUS));
        if clear(p) {
            return Some(p);
        }
        // slide away from the nearest tower up to 20,000 while the target stays in the knock-up
        let t = f.units.iter().filter(|t| t.tower && t.hp > 0).min_by_key(|t| t.pos.d2(p))?;
        for k in 1..=4 {
            let q = p.offset(dir(t.pos, p), 5_000.0 * k as f64).clamp_map();
            if clear(q) && q.within(p, d::ULT_KNOCK_RADIUS - 6_000) {
                return Some(q);
            }
        }
        None
    }

    /// S3 with the pillar standing: bash someone into it, or charge into it himself when it would catch 2+.
    fn decide_combo(&self, f: &Frame) -> Option<Order> {
        let pillar = self.pillar?;
        if !self.ready(2, f.tick) {
            // relocate the pillar behind the best target, ready for the bash
            if !pillar.relocated && self.ready(2, f.tick + 90) {
                let t = self.nearest_enemy(f, 26_000)?;
                let want = t.pos.offset(dir(f.pos, t.pos), 12_000.0);
                if want.within(pillar.center, d::BULWARK_RELOCATE_RANGE) && !pillar.center.within(want, 6_000) {
                    return Some(Order::Relocate(want));
                }
            }
            return None;
        }
        // an enemy between him and the pillar: the knockback line runs into it
        for e in f.enemy_champions().filter(|e| f.can_reach(e) && e.pos.within(f.pos, d::BASH_RANGE + e.radius)) {
            let line_end = e.pos.offset(dir(f.pos, e.pos), d::BASH_KNOCKBACK as f64);
            if seg_d2(pillar.center, e.pos, line_end) <= ((d::BULWARK_PILLAR_HALF + e.radius) as f64).powi(2) {
                return Some(Order::Bash(e.pos));
            }
        }
        // relocate to make that line if he can
        if !pillar.relocated {
            if let Some(t) = self.nearest_enemy(f, d::BASH_RANGE) {
                let want = t.pos.offset(dir(f.pos, t.pos), 12_000.0);
                if want.within(pillar.center, d::BULWARK_RELOCATE_RANGE) {
                    return Some(Order::Relocate(want));
                }
            }
        }
        // charge into it himself when it catches 2+ (1 if they're low)
        let near = f.enemy_champions().filter(|e| f.can_reach(e) && e.pos.within(pillar.center, d::BULWARK_SHATTER_RADIUS)).count();
        if pillar.center.within(f.pos, d::BASH_RANGE) && near >= 2 {
            return Some(Order::Bash(pillar.center));
        }
        None
    }

    /// S2 smart placement: score candidate aims so the pillar rises behind the target (away from him, on its
    /// escape side) or among the most enemies; in a cage, against an edge; escaping, in the chaser's path.
    fn decide_bulwark(&self, f: &Frame, low: bool) -> Option<Order> {
        if !self.ready(1, f.tick) {
            return None;
        }
        let targets: Vec<&Unit> = f.enemy_champions().filter(|e| f.can_reach(e) && e.pos.within(f.pos, d::BULWARK_LENGTH + 6_000)).collect();
        if targets.is_empty() {
            return None;
        }
        if low {
            // escaping: the pillar between him and the nearest chaser
            let c = targets.iter().min_by_key(|e| e.pos.d2(f.pos))?;
            return Some(Order::Bulwark(c.pos));
        }
        let judgement = f.profile(PROFILE_JUDGEMENT);
        let mut best: Option<(i64, Point)> = None;
        for t in &targets {
            // aim through the target so the line ends ~10,000 behind them
            let aim = t.pos.offset(dir(f.pos, t.pos), 10_000.0);
            let end = self.fissure_end(f, f.pos, aim);
            let on_line = targets.iter().filter(|o| seg_d2(o.pos, f.pos, end) <= ((d::BULWARK_WIDTH + o.radius) as f64).powi(2)).count() as i64;
            let near_end = targets.iter().filter(|o| o.pos.within(end, d::BULWARK_SHATTER_RADIUS)).count() as i64;
            let behind = end.d2(f.pos) > t.pos.d2(f.pos);
            let mut score = on_line * 30 + near_end * 25 + if behind { 20 } else { 0 };
            if let Some(c) = f.cage {
                // against an edge: the bash pins them between pillar and wall
                if depth(c.center, c.radius, t.pos) < 20_000.0 {
                    score += 15;
                }
            }
            // judgement weighs the setup; a poor reader takes the first target in reach
            score = score * (50 + judgement) / 100 - (f.pos.dist(t.pos) / 10_000.0) as i64;
            if best.is_none_or(|(s, _)| score > s) {
                best = Some((score, aim));
            }
        }
        best.filter(|(s, _)| *s >= 20).map(|(_, a)| Order::Bulwark(a))
    }

    fn decide_bash(&self, f: &Frame, low: bool) -> Option<Order> {
        if !self.ready(2, f.tick) {
            return None;
        }
        let aggressive = f.profile(PROFILE_AGGRESSIVE);
        let mut best: Option<(i64, Point)> = None;
        for e in f.enemy_champions().filter(|e| f.can_reach(e) && e.pos.within(f.pos, d::BASH_RANGE + e.radius)) {
            let along = dir(f.pos, e.pos);
            // does the knockback line meet a wall within reach?
            let walled = (1..=6).any(|k| self.blocked(f, e.pos.offset(along, (d::BASH_KNOCKBACK * k / 6) as f64), e.radius.min(8_000), f.cage));
            let mut score = if walled { 80 } else { 20 + aggressive / 3 };
            if low && e.pos.within(f.pos, 14_000) {
                score += 40; // knock the chaser away
            }
            if e.hp * 100 <= e.max_hp * 30 {
                score += 15;
            }
            if best.is_none_or(|(s, _)| score > s) {
                best = Some((score, e.pos));
            }
        }
        best.filter(|(s, _)| *s >= 35).map(|(_, p)| Order::Bash(p))
    }

    fn decide_engage_ghost(&self, f: &Frame, low: bool) -> Option<Order> {
        if low || !self.ready(0, f.tick) || !self.ready(2, f.tick) {
            return None;
        }
        // close the gap for the combo: reappear behind the target
        let t = f.enemy_champions().filter(|e| f.can_reach(e) && !e.pos.within(f.pos, 26_000) && e.pos.within(f.pos, 55_000)).min_by_key(|e| (e.hp * 100 / e.max_hp.max(1), e.id))?;
        if f.profile(PROFILE_AGGRESSIVE) < 35 {
            return None;
        }
        Some(Order::Ghost(t.pos.offset(dir(f.pos, t.pos), 6_000.0)))
    }

    fn decide_farm(&self, f: &Frame) -> Option<Order> {
        if f.enemy_champions().any(|e| e.pos.within(f.pos, 70_000)) {
            return None;
        }
        let farm: Vec<&Unit> = f.units.iter().filter(|u| u.farmable && !u.champion && u.pos.within(f.pos, d::BULWARK_LENGTH)).collect();
        if farm.is_empty() {
            return None;
        }
        if self.ready(1, f.tick) {
            // the line through the most of them
            let best = farm.iter().max_by_key(|t| {
                let end = f.pos.toward(t.pos, d::BULWARK_LENGTH);
                (farm.iter().filter(|o| seg_d2(o.pos, f.pos, end) <= ((d::BULWARK_WIDTH + o.radius) as f64).powi(2)).count(), std::cmp::Reverse(t.id))
            })?;
            let end = f.pos.toward(best.pos, d::BULWARK_LENGTH);
            let n = farm.iter().filter(|o| seg_d2(o.pos, f.pos, end) <= ((d::BULWARK_WIDTH + o.radius) as f64).powi(2)).count();
            if n >= 2 || best.max_hp >= 1_500 {
                return Some(Order::Bulwark(best.pos));
            }
        }
        if self.ready(2, f.tick) {
            let t = farm.iter().filter(|u| u.pos.within(f.pos, d::BASH_RANGE)).max_by_key(|u| (u.max_hp, std::cmp::Reverse(u.id)))?;
            if t.max_hp >= 1_500 {
                return Some(Order::Bash(t.pos));
            }
        }
        None
    }
}

fn cage_of(f: &Frame, u: &Unit) -> Option<Cage> {
    let owner = u.caged_by?;
    if f.cage.is_some_and(|c| c.owner == owner) {
        return f.cage;
    }
    None
}
