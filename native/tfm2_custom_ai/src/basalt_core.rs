//! Basalt, the Wallbreaker: a deterministic core (snapshot in, effects out). `basalt.rs` builds the frame from the sim,
//! applies the effects and turns his walls into the shared `sbw:` walls everyone collides with; the Lab
//! (`native/basalt_lab`) runs this same file.
//!
//! A stone-shaper brawler with a whip. Passives (no skill slot, they fire on their own):
//!   Bedrock: up to 5 walls, each a big square block (32,000 a side, like Grimhald's blocks but bigger). A wall stands while he is within 150,000 of it and fades 3 s
//!     after he leaves. Only he can hit his walls: each takes 3 of his hits (a punch, a throw landing, an enemy
//!     slammed into it) and shatters (rubble: +15% on his next whip). An enemy his skills drive into any wall (his,
//!     the map's, Steve's) is slammed: a 1 s stun and damage, once every 2 s per target.
//!   Jump: 2 charges; a 90,000 leap, 0.6 s airborne (CC immune, passes every wall), one free double jump; kicking off
//!     one of his walls goes 140,000 and restores the double jump (each wall once per airborne chain).
//!   Whip Pull: his whip catches any wall within 120,000 and slingshots him over it, faster the farther he was, landing
//!     half the distance past it (up to 60,000), airborne all the way; enemies he flies through are knocked aside.
//! Eight skills: each slot has a ground version and a jump (airborne) version, sharing one cooldown.
//!   S1 Raise & Punch / Air Dash (2 charges)   S2 Hurl / Slab Drop   S3 Crushing Clap (the V slam) / Whip Swing
//!   Ult Boulder Throw / Meteor Punch: a 0.5 s charge, then a global rock (knock-up, slow) or, from the air, a punched
//!     rock raining burning meteors (burn, slow, a little knock-up); a recast whips him to where it landed.
//! His AI is Kicker's flat priority list (ult, escape, peel, follow-up, help, best target, farm), engaging mobility
//! first and skills second. Nothing of his hits a tower.
use crate::basalt_data as d;
use serde::{Deserialize, Serialize};

pub const MAP: i64 = 960_000;
pub const CELL: i64 = 32_000;
pub const CELLS: i64 = 30;
/// half the thickness of a shared wall (`steve::WALL_HALF`)
pub const WALL_HALF: i64 = 8_000;

pub const EGO: usize = 0;
pub const JUDGEMENT: usize = 1;
pub const AGGRESSIVE: usize = 6;
pub const STRATEGY: usize = 12;

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
        self.d2(o) <= r as i128 * r as i128
    }
    pub fn along(self, u: (f64, f64), len: i64) -> Point {
        Point::new(self.x + (u.0 * len as f64).round() as i64, self.y + (u.1 * len as f64).round() as i64)
    }
    pub fn toward(self, to: Point, len: i64) -> Point {
        if self.within(to, len) {
            return to;
        }
        self.along(unit_dir(self, to), len)
    }
    pub fn clamp_map(self) -> Point {
        Point::new(self.x.clamp(2_000, MAP - 2_000), self.y.clamp(2_000, MAP - 2_000))
    }
    pub fn on_map(self) -> bool {
        (0..=MAP).contains(&self.x) && (0..=MAP).contains(&self.y)
    }
    pub fn mid(self, o: Point) -> Point {
        Point::new((self.x + o.x) / 2, (self.y + o.y) / 2)
    }
}
pub fn unit_dir(a: Point, b: Point) -> (f64, f64) {
    let (dx, dy) = ((b.x - a.x) as f64, (b.y - a.y) as f64);
    let l = dx.hypot(dy);
    if l < 1.0 {
        (1.0, 0.0)
    } else {
        (dx / l, dy / l)
    }
}
pub fn rotate(u: (f64, f64), deg: f64) -> (f64, f64) {
    let (s, c) = deg.to_radians().sin_cos();
    (u.0 * c - u.1 * s, u.0 * s + u.1 * c)
}
fn perp(u: (f64, f64)) -> (f64, f64) {
    (-u.1, u.0)
}
/// distance from `p` to the segment a-b
pub fn seg_dist(p: Point, a: Point, b: Point) -> f64 {
    let (vx, vy) = ((b.x - a.x) as f64, (b.y - a.y) as f64);
    let len2 = vx * vx + vy * vy;
    let k = if len2 == 0.0 { 0.0 } else { (((p.x - a.x) as f64 * vx + (p.y - a.y) as f64 * vy) / len2).clamp(0.0, 1.0) };
    let q = (a.x as f64 + vx * k, a.y as f64 + vy * k);
    ((p.x as f64 - q.0).powi(2) + (p.y as f64 - q.1).powi(2)).sqrt()
}
fn pct(base: i64, ratio: i64, ad: i64) -> i64 {
    base + ratio * ad / 100
}
fn t(n: i64) -> u64 {
    n.max(0) as u64
}

// ------------------------------------------------------------------ frame

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Unit {
    pub id: usize,
    pub team: usize,
    pub pos: Point,
    pub hp: i64,
    pub max_hp: i64,
    #[serde(default)]
    pub attack: i64,
    pub champion: bool,
    pub tower: bool,
    pub visible: bool,
    pub targetable: bool,
    /// a minion or jungle monster his farming may spend skills on (the shared farming rules say so, and it's calm)
    #[serde(default)]
    pub farmable: bool,
}

/// Another champion's shared wall (Steve's boat walls, Grimhald's cage edges): he can whip and slam into it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Seg {
    pub a: Point,
    pub b: Point,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "aim", rename_all = "snake_case")]
pub enum Order {
    /// a jump (a double jump in the air, a wall kick beside one of his walls) toward the point
    Jump(Point),
    /// Whip Pull: slingshot over the wall nearest this point
    Pull(Point),
    /// S1: Raise & Punch toward the point (ground) / Air Dash to it (air)
    S1(Point),
    /// S2: Hurl onto the point (ground) / Slab Drop onto it (air)
    S2(Point),
    /// S3: Crushing Clap toward the point (ground) / Whip Swing on the enemy nearest it (air)
    S3(Point),
    /// Ult: Boulder Throw (ground) / Meteor Punch (air) onto the point; recast: the slingshot to where it landed
    Ult(Point),
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
    #[serde(default)]
    pub stunned: bool,
    pub ult_unlocked: bool,
    #[serde(default)]
    pub home: Point,
    pub units: Vec<Unit>,
    /// wall cells (`CELLS` x `CELLS`)
    #[serde(default)]
    pub terrain: Vec<bool>,
    /// other champions' shared walls standing now
    #[serde(default)]
    pub other_walls: Vec<Seg>,
    pub profile: [i64; 13],
    pub manual: bool,
    pub order: Option<Order>,
}
impl Frame {
    pub fn enemy(&self, u: &Unit) -> bool {
        u.team != self.team && u.visible && u.targetable && u.hp > 0
    }
    pub fn skill_target(&self, u: &Unit) -> bool {
        self.enemy(u) && !u.tower
    }
    pub fn enemy_champions(&self) -> impl Iterator<Item = &Unit> {
        self.units.iter().filter(move |u| u.champion && self.enemy(u))
    }
    pub fn allies(&self) -> impl Iterator<Item = &Unit> {
        self.units.iter().filter(move |u| u.champion && u.team == self.team && u.hp > 0 && u.id != self.owner)
    }
    pub fn unit(&self, id: usize) -> Option<&Unit> {
        self.units.iter().find(|u| u.id == id)
    }
    pub fn p(&self, i: usize) -> i64 {
        self.profile.get(i).copied().unwrap_or(50).clamp(0, 100)
    }
    pub fn terrain_at(&self, p: Point) -> bool {
        !p.on_map() || self.terrain.get(((p.y / CELL) * CELLS + p.x / CELL) as usize).copied().unwrap_or(false)
    }
    pub fn enemy_tower_at(&self, p: Point) -> Option<&Unit> {
        self.units.iter().find(|u| u.tower && u.team != self.team && u.hp > 0 && u.pos.within(p, d::AI_TOWER_RANGE))
    }
    /// The nearest open (non-terrain, on-map) point to `p`.
    pub fn nearest_ground(&self, p: Point) -> Point {
        let p = p.clamp_map();
        if !self.terrain_at(p) {
            return p;
        }
        for r in (4_000..=96_000).step_by(4_000) {
            let mut best: Option<Point> = None;
            for k in 0..24 {
                let q = p.along(rotate((1.0, 0.0), k as f64 * 15.0), r).clamp_map();
                if !self.terrain_at(q) && best.is_none_or(|b| q.d2(p) < b.d2(p)) {
                    best = Some(q);
                }
            }
            if let Some(b) = best {
                return b;
            }
        }
        p
    }
    /// Terrain cell centres within `range` of `p`, nearest first (at most `cap`).
    pub fn terrain_near(&self, p: Point, range: i64, cap: usize) -> Vec<Point> {
        let mut out: Vec<Point> = (0..CELLS * CELLS)
            .filter(|i| self.terrain.get(*i as usize).copied().unwrap_or(false))
            .map(|i| Point::new(i % CELLS * CELL + CELL / 2, i / CELLS * CELL + CELL / 2))
            .filter(|c| c.within(p, range + CELL / 2))
            .collect();
        out.sort_by_key(|c| (c.d2(p), c.x, c.y));
        out.truncate(cap);
        out
    }
    /// The point of a terrain cell nearest `p` (a 2,000 inset from its edges).
    pub fn cell_point(c: Point, p: Point) -> Point {
        let inset = 2_000;
        Point::new(p.x.clamp(c.x - CELL / 2 + inset, c.x + CELL / 2 - inset), p.y.clamp(c.y - CELL / 2 + inset, c.y + CELL / 2 - inset))
    }
}

// ------------------------------------------------------------------ walls

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Slide {
    pub a0: Point,
    pub b0: Point,
    pub delta: Point,
    pub start: u64,
    pub ticks: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct StoneWall {
    pub id: u32,
    pub a: Point,
    pub b: Point,
    pub integrity: u8,
    pub born: u64,
    /// he has been out of the keep range since this tick
    pub out_since: Option<u64>,
    pub slide: Option<Slide>,
}
/// half the side of his square walls
pub fn square_half() -> i64 {
    (d::WALL_SIZE / 2).max(WALL_HALF)
}
impl StoneWall {
    /// the square's centre (`a` and `b` are both the centre: a square block, axis-aligned)
    pub fn mid(&self) -> Point {
        self.a.mid(self.b)
    }
    /// distance from `p` to the square's surface (0 inside)
    pub fn dist(&self, p: Point) -> f64 {
        let c = self.mid();
        let h = square_half();
        let dx = ((p.x - c.x).abs() - h).max(0) as f64;
        let dy = ((p.y - c.y).abs() - h).max(0) as f64;
        dx.hypot(dy)
    }
    /// the point of the square nearest `p`
    pub fn nearest(&self, p: Point) -> Point {
        let c = self.mid();
        let h = square_half();
        Point::new(p.x.clamp(c.x - h, c.x + h), p.y.clamp(c.y - h, c.y + h))
    }
}

/// What a forced move ran into
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Hit {
    None,
    Terrain,
    /// one of his walls
    Mine(u32),
    /// someone else's shared wall
    Other,
}

// ------------------------------------------------------------------ effects

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Effect {
    Damage { target: usize, amount: i64 },
    Stun { target: usize, ticks: u64 },
    KnockUp { target: usize, ticks: u64 },
    Slow { target: usize, pct: i64, ticks: u64 },
    /// he holds still (the ult's charge)
    Root { ticks: u64 },
    /// an enemy pushed to `to` over `ticks` (an engine Knock)
    Knock { target: usize, to: Point, ticks: u64 },
    /// he moves himself to `to` over `ticks`
    Glide { to: Point, ticks: u64 },
    /// airborne for at least this many more ticks (CC immune, basics miss, passes every wall)
    Air { ticks: u64 },
    /// the whip's +15% on his next basic
    Rubble { ticks: u64 },
    Steer { to: Point },
    Fx { name: String, at: Point, ticks: u64 },
    Log { text: String },
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Air {
    pub land: Point,
    pub until: u64,
    pub double_left: bool,
    /// an air skill (Slab Drop, Whip Swing, Meteor Punch) closed the chain: no more air plays this flight
    pub spent: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct PendingSlam {
    pub target: usize,
    pub at: u64,
    pub hit: Hit,
    /// extra damage on top of the slam (the skill's own hit)
    pub damage: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Throw {
    Hurl,
    Slab,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Flying {
    pub kind: Throw,
    pub to: Point,
    pub dir: (f64, f64),
    pub land_at: u64,
    pub integrity: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Clap {
    pub apex: Point,
    pub dir: (f64, f64),
    pub start: u64,
    pub left: u32,
    pub right: u32,
    pub r0: i64,
    pub r1: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Swing {
    pub target: usize,
    pub center: Point,
    /// heading of the victim from the centre at the start, degrees
    pub start_deg: f64,
    /// total turn (360 + x), degrees, signed
    pub turn_deg: f64,
    pub r0: i64,
    pub r1: i64,
    pub start: u64,
    pub next: u64,
    /// slammed into an existing wall at the end (else a new wall is raised there)
    pub into: Hit,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Core {
    pub walls: Vec<StoneWall>,
    pub next_wall: u32,
    pub air: Option<Air>,
    pub jumps: i64,
    pub jump_at: u64,
    /// walls kicked off in this airborne chain
    pub kicked: Vec<u32>,
    pub pull_at: u64,
    pub punches: i64,
    pub punch_at: u64,
    pub s2_at: u64,
    pub s3_at: u64,
    pub ult_at: u64,
    /// the ult's charge: (air, aim, release at)
    pub charging: Option<(bool, Point, u64)>,
    /// a rock in flight: (air, aim, lands at)
    pub boulders: Vec<(bool, Point, u64)>,
    /// meteors still to land: (spot, at)
    pub meteors: Vec<(Point, u64)>,
    /// burning enemies: (target, until, next tick)
    pub burns: Vec<(usize, u64, u64)>,
    /// the recast: where the rock landed, and until when
    pub recast: Option<(Point, u64)>,
    pub slams: Vec<PendingSlam>,
    /// (target, no slam stun again before)
    pub slam_guard: Vec<(usize, u64)>,
    pub flying: Vec<Flying>,
    pub claps: Vec<Clap>,
    pub swing: Option<Swing>,
    pub rubble_until: u64,
    /// the target he last slammed (follow it up) and until when
    pub last_slammed: Option<(usize, u64)>,
    pub hp_log: Vec<(u64, i64)>,
    pub steer_at: u64,
    /// a purposeful walk his AI asked for this tick
    #[serde(default)]
    pub pending_walk: Option<Point>,
    /// cast recovery: no skill, jump or pull before this tick (the ult's recast excepted)
    #[serde(default)]
    pub ready_at: u64,
    #[serde(default)]
    pub reason: String,
    #[serde(default)]
    pub rejection: String,
}
impl Default for Core {
    fn default() -> Self {
        Core {
            walls: vec![],
            next_wall: 1,
            air: None,
            jumps: d::JUMP_CHARGES,
            jump_at: 0,
            kicked: vec![],
            pull_at: 0,
            punches: d::PUNCH_CHARGES,
            punch_at: 0,
            s2_at: 0,
            s3_at: 0,
            ult_at: 0,
            charging: None,
            boulders: vec![],
            meteors: vec![],
            burns: vec![],
            recast: None,
            slams: vec![],
            slam_guard: vec![],
            flying: vec![],
            claps: vec![],
            swing: None,
            rubble_until: 0,
            last_slammed: None,
            hp_log: vec![],
            steer_at: 0,
            pending_walk: None,
            ready_at: 0,
            reason: String::new(),
            rejection: String::new(),
        }
    }
}

fn travel(from: Point, to: Point, speed: i64) -> u64 {
    ((from.dist(to) / speed.max(1) as f64).ceil() as u64).max(1)
}

impl Core {
    pub fn respawn(&mut self, _tick: u64) {
        self.air = None;
        self.kicked.clear();
        self.slams.clear();
        self.flying.clear();
        self.claps.clear();
        self.swing = None;
        self.hp_log.clear();
        self.last_slammed = None;
        self.charging = None;
        self.recast = None;
        self.ready_at = 0;
    }
    pub fn airborne(&self, f: &Frame) -> bool {
        self.air.is_some_and(|a| f.tick < a.until)
    }
    pub fn wall_max(&self, _f: &Frame) -> usize {
        d::WALL_MAX.max(1) as usize
    }
    pub fn ult_ready(&self, f: &Frame) -> bool {
        f.ult_unlocked && f.tick >= self.ult_at && self.charging.is_none()
    }
    /// recovering from his last skill, jump or pull
    pub fn recovering(&self, f: &Frame) -> bool {
        f.tick < self.ready_at
    }
    pub fn recast_ready(&self, f: &Frame) -> bool {
        self.recast.is_some_and(|r| f.tick < r.1)
    }
    /// his +15% whip after a wall breaks
    pub fn basic_multiplier(&self, tick: u64) -> i64 {
        if tick < self.rubble_until {
            100 + d::WALL_RUBBLE_PCT
        } else {
            100
        }
    }
    pub fn take_rubble(&mut self) {
        self.rubble_until = 0;
    }

    fn recharge(&mut self, f: &Frame) {
        if self.jumps < d::JUMP_CHARGES && f.tick >= self.jump_at {
            self.jumps += 1;
            if self.jumps < d::JUMP_CHARGES {
                self.jump_at = f.tick + t(d::JUMP_RECHARGE_TICKS);
            }
        }
        if self.punches < d::PUNCH_CHARGES && f.tick >= self.punch_at {
            self.punches += 1;
            if self.punches < d::PUNCH_CHARGES {
                self.punch_at = f.tick + t(d::PUNCH_RECHARGE_TICKS);
            }
        }
    }
    fn spend_jump(&mut self, f: &Frame) {
        if self.jumps == d::JUMP_CHARGES {
            self.jump_at = f.tick + t(d::JUMP_RECHARGE_TICKS);
        }
        self.jumps -= 1;
    }
    fn spend_punch(&mut self, f: &Frame) {
        if self.punches == d::PUNCH_CHARGES {
            self.punch_at = f.tick + t(d::PUNCH_RECHARGE_TICKS);
        }
        self.punches -= 1;
    }

    // -------------------------------------------------------------- step

    pub fn step(&mut self, f: &Frame) -> Vec<Effect> {
        let mut out = vec![];
        self.rejection.clear();
        if !f.alive {
            self.respawn(f.tick);
            return out;
        }
        self.recharge(f);
        self.hp_log.retain(|h| f.tick < h.0 + t(d::AI_HURT_TICKS));
        self.hp_log.push((f.tick, f.hp));
        self.slam_guard.retain(|g| f.tick < g.1);
        if self.last_slammed.is_some_and(|l| f.tick >= l.1) {
            self.last_slammed = None;
        }
        self.run_walls(f, &mut out);
        self.run_claps(f, &mut out);
        self.run_flying(f, &mut out);
        self.run_swing(f, &mut out);
        self.run_slams(f, &mut out);
        self.run_ult(f, &mut out);
        self.run_air(f, &mut out);
        if self.recast.is_some_and(|r| f.tick >= r.1) {
            self.recast = None;
        }
        self.pending_walk = None;
        let order = if self.charging.is_some() { None } else if f.manual { f.order } else { self.decide(f) };
        if let Some(o) = order {
            let recast = matches!(o, Order::Ult(_)) && self.recast_ready(f);
            if self.recovering(f) && !recast {
                if f.manual {
                    self.reject("recover");
                } else if !matches!(o, Order::Pull(_) | Order::Ult(_)) && self.pending_walk.is_none() {
                    // the AI walks on toward its target while it recovers
                    let (Order::Jump(p) | Order::S1(p) | Order::S2(p) | Order::S3(p) | Order::Pull(p) | Order::Ult(p)) = o;
                    if !p.within(f.pos, d::BASIC_RANGE) {
                        self.walk(f, p);
                    }
                }
            } else if !f.stunned || self.airborne(f) {
                self.execute(f, o, &mut out);
                if self.rejection.is_empty() {
                    let air = self.airborne(f) || self.air.is_some_and(|a| a.until > f.tick);
                    self.ready_at = f.tick + t(if air { d::RECOVER_AIR_RECOVER_TICKS } else { d::RECOVER_RECOVER_TICKS });
                }
            }
        }
        if let Some(to) = self.pending_walk.take() {
            out.push(Effect::Steer { to });
        }
        out
    }

    // -------------------------------------------------------------- walls

    /// Where a wall stands right now (slides interpolate)
    fn place_walls(&mut self, tick: u64) {
        for w in self.walls.iter_mut() {
            if let Some(s) = w.slide {
                let k = ((tick.saturating_sub(s.start)) as f64 / s.ticks.max(1) as f64).min(1.0);
                let dx = (s.delta.x as f64 * k) as i64;
                let dy = (s.delta.y as f64 * k) as i64;
                w.a = Point::new(s.a0.x + dx, s.a0.y + dy);
                w.b = Point::new(s.b0.x + dx, s.b0.y + dy);
                if k >= 1.0 {
                    w.slide = None;
                }
            }
        }
    }

    fn run_walls(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        self.place_walls(f.tick);
        // out of his keep range: fade 3 s later (back in range cancels it)
        let mut gone = vec![];
        for w in self.walls.iter_mut() {
            if w.dist(f.pos) > d::WALL_KEEP_RANGE as f64 {
                let since = *w.out_since.get_or_insert(f.tick);
                if f.tick >= since + t(d::WALL_FADE_TICKS) {
                    gone.push(w.id);
                }
            } else {
                w.out_since = None;
            }
        }
        for id in gone {
            self.crumble(f, id, false, out);
        }
        // over the limit: the oldest crumble
        let max = self.wall_max(f);
        while self.walls.len() > max {
            let oldest = self.walls.iter().min_by_key(|w| (w.born, w.id)).map(|w| w.id).unwrap();
            self.crumble(f, oldest, false, out);
        }
    }

    /// a square wall centred on `c`
    fn raise(&mut self, f: &Frame, c: Point, integrity: u8, out: &mut Vec<Effect>) -> u32 {
        let id = self.next_wall;
        self.next_wall += 1;
        let c = c.clamp_map();
        let (a, b) = (c, c);
        self.walls.push(StoneWall {
            id,
            a,
            b,
            integrity,
            born: f.tick,
            out_since: None,
            slide: None,
        });
        out.push(Effect::Fx {
            name: "bs_rise".into(),
            at: a.mid(b),
            ticks: 24,
        });
        id
    }
    /// a square wall centred on `c` (the heading only matters to callers placing it)
    fn raise_across(&mut self, f: &Frame, c: Point, _dir: (f64, f64), out: &mut Vec<Effect>) -> u32 {
        self.raise(f, c, d::WALL_INTEGRITY.max(1) as u8, out)
    }
    /// A wall breaks (`shatter`: his hits used it up; otherwise it crumbled away). Either way: rubble.
    fn crumble(&mut self, f: &Frame, id: u32, shatter: bool, out: &mut Vec<Effect>) {
        let Some(i) = self.walls.iter().position(|w| w.id == id) else { return };
        let w = self.walls.remove(i);
        self.rubble_until = f.tick + t(d::WALL_RUBBLE_TICKS);
        out.push(Effect::Rubble { ticks: t(d::WALL_RUBBLE_TICKS) });
        out.push(Effect::Fx {
            name: if shatter { "bs_break".into() } else { "bs_dust".into() },
            at: w.mid(),
            ticks: 24,
        });
    }
    /// One of his hits on his own wall: integrity down by one; at 0 it shatters.
    fn hit_wall(&mut self, f: &Frame, id: u32, out: &mut Vec<Effect>) {
        let Some(w) = self.walls.iter_mut().find(|w| w.id == id) else { return };
        w.integrity = w.integrity.saturating_sub(1);
        if w.integrity == 0 {
            self.crumble(f, id, true, out);
        }
    }

    /// What stands at `p` (within `margin` of its surface): terrain, his wall, another wall
    pub fn blocked(&self, f: &Frame, p: Point, margin: i64, skip: Option<u32>) -> Hit {
        if f.terrain_at(p) {
            return Hit::Terrain;
        }
        let m = (WALL_HALF + margin) as f64;
        if let Some(w) = self.walls.iter().find(|w| Some(w.id) != skip && w.dist(p) < margin as f64) {
            return Hit::Mine(w.id);
        }
        if f.other_walls.iter().any(|s| seg_dist(p, s.a, s.b) < m) {
            return Hit::Other;
        }
        Hit::None
    }

    /// A body pushed from `from` along `u` for `len`: where it stops (before a wall, terrain or the edge), and what
    /// stopped it.
    pub fn push_path(&self, f: &Frame, from: Point, u: (f64, f64), len: i64, skip: Option<u32>) -> (Point, Hit) {
        let mut last = from;
        let steps = (len / 2_000).max(1);
        for i in 1..=steps {
            let q = from.along(u, len * i / steps);
            if !q.on_map() || q.x < 2_000 || q.y < 2_000 || q.x > MAP - 2_000 || q.y > MAP - 2_000 {
                return (last, Hit::None);
            }
            let hit = self.blocked(f, q, 3_000, skip);
            if hit != Hit::None {
                return (last, hit);
            }
            last = q;
        }
        (last, Hit::None)
    }

    /// Push `u` to `to`; if something stopped it, slam it on arrival.
    fn shove(&mut self, f: &Frame, target: &Unit, dir: (f64, f64), len: i64, ticks: u64, damage: i64, skip: Option<u32>, out: &mut Vec<Effect>) {
        let (to, hit) = self.push_path(f, target.pos, dir, len, skip);
        if damage > 0 {
            out.push(Effect::Damage { target: target.id, amount: damage });
        }
        if to != target.pos {
            out.push(Effect::Knock { target: target.id, to, ticks: ticks.max(1) });
        }
        if hit != Hit::None {
            let k = (target.pos.dist(to) / len.max(1) as f64).clamp(0.0, 1.0);
            self.slams.push(PendingSlam {
                target: target.id,
                at: f.tick + (ticks as f64 * k).round().max(1.0) as u64,
                hit,
                damage: 0,
            });
        }
    }

    fn run_slams(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        let due: Vec<PendingSlam> = self.slams.iter().filter(|s| f.tick >= s.at).copied().collect();
        self.slams.retain(|s| f.tick < s.at);
        for s in due {
            self.slam(f, s.target, s.hit, s.damage, out);
        }
    }

    /// The slam: a stun (once every 2 s per target), damage, his wall takes a hit.
    pub fn slam(&mut self, f: &Frame, target: usize, hit: Hit, extra: i64, out: &mut Vec<Effect>) {
        let Some(u) = f.unit(target).filter(|u| f.skill_target(u)).cloned() else { return };
        out.push(Effect::Damage {
            target,
            amount: pct(d::SLAM_BASE, d::SLAM_AD_RATIO, f.attack) + extra,
        });
        if !self.slam_guard.iter().any(|g| g.0 == target) {
            out.push(Effect::Stun { target, ticks: t(d::SLAM_STUN_TICKS) });
            self.slam_guard.push((target, f.tick + t(d::SLAM_REPEAT_TICKS)));
        }
        out.push(Effect::Fx {
            name: "bs_wall_slam".into(),
            at: u.pos,
            ticks: 24,
        });
        if let Hit::Mine(id) = hit {
            self.hit_wall(f, id, out);
        }
        self.last_slammed = Some((target, f.tick + 150));
    }

    // -------------------------------------------------------------- movement: Jump, double jump, wall kick, Whip Pull

    /// a landing spot that is open ground, outside every wall
    pub fn safe_land(&self, f: &Frame, p: Point, dir: (f64, f64)) -> Point {
        let mut q = f.nearest_ground(p.clamp_map());
        for k in 0..30 {
            if self.blocked(f, q, 4_000, None) == Hit::None {
                return q;
            }
            q = f.nearest_ground(p.along(dir, (k + 1) * 3_000).clamp_map());
        }
        f.nearest_ground(p)
    }

    /// his wall within kick range he hasn't kicked off in this chain
    pub fn kick_wall(&self, f: &Frame) -> Option<u32> {
        self.walls
            .iter()
            .filter(|w| w.dist(f.pos) <= d::JUMP_KICK_RANGE as f64 && !self.kicked.contains(&w.id))
            .min_by_key(|w| (w.dist(f.pos) as i64, w.id))
            .map(|w| w.id)
    }

    /// What a Jump order would do now: (kind, range); None when no jump is possible.
    pub fn jump_kind(&self, f: &Frame) -> Option<(&'static str, i64)> {
        if self.air.is_some_and(|a| a.spent && f.tick < a.until) {
            return None;
        }
        if self.kick_wall(f).is_some() {
            return Some(("kick", d::JUMP_KICK_JUMP));
        }
        if self.airborne(f) {
            return self.air.filter(|a| a.double_left).map(|_| ("double", d::JUMP_DOUBLE_RANGE));
        }
        (self.jumps > 0).then_some(("jump", d::JUMP_RANGE))
    }

    fn fly(&mut self, f: &Frame, to: Point, ticks: u64, double_left: bool, out: &mut Vec<Effect>) {
        let air_ticks = ticks.max(1);
        out.push(Effect::Glide { to, ticks: air_ticks });
        out.push(Effect::Air { ticks: air_ticks + 2 });
        self.air = Some(Air {
            land: to,
            until: f.tick + air_ticks,
            double_left,
            spent: false,
        });
    }

    fn jump(&mut self, f: &Frame, aim: Point, out: &mut Vec<Effect>) {
        let Some((kind, range)) = self.jump_kind(f) else { return self.reject("jump") };
        let dir = unit_dir(f.pos, aim);
        let to = self.safe_land(f, f.pos.toward(aim, range).clamp_map(), dir);
        match kind {
            "kick" => {
                let w = self.kick_wall(f).unwrap();
                self.kicked.push(w);
                out.push(Effect::Fx {
                    name: "bs_dust".into(),
                    at: f.pos,
                    ticks: 18,
                });
                self.fly(f, to, t(d::JUMP_AIR_TICKS) * range as u64 / d::JUMP_RANGE.max(1) as u64, true, out);
            }
            "double" => {
                self.fly(f, to, (t(d::JUMP_AIR_TICKS) * range as u64 / d::JUMP_RANGE.max(1) as u64).max(t(d::JUMP_AIR_TICKS) * 2 / 3), false, out);
            }
            _ => {
                self.spend_jump(f);
                self.kicked.clear();
                self.fly(f, to, t(d::JUMP_AIR_TICKS), true, out);
            }
        }
        out.push(Effect::Fx {
            name: "bs_leap".into(),
            at: f.pos,
            ticks: 18,
        });
    }

    /// Whip Pull's slingshot over a wall point `w`: (landing, flight ticks, speed)
    pub fn sling(&self, f: &Frame, w: Point) -> (Point, u64, i64) {
        let dist = f.pos.dist(w) as i64;
        let dir = unit_dir(f.pos, w);
        let speed = (d::PULL_SPEED_BASE + dist / d::PULL_SPEED_DIVISOR.max(1)).min(d::PULL_SPEED_CAP);
        let over = (dist * d::PULL_OVERSHOOT_PCT / 100).min(d::PULL_OVERSHOOT_MAX);
        let land = self.safe_land(f, w.along(dir, over.max(WALL_HALF + 6_000)), dir);
        (land, travel(f.pos, land, speed), speed)
    }

    /// Every wall point he could whip within range: his walls, other walls, terrain cells (nearest first, capped).
    pub fn whip_points(&self, f: &Frame, range: i64) -> Vec<Point> {
        let near = |a: Point, b: Point| -> Point {
            let (vx, vy) = ((b.x - a.x) as f64, (b.y - a.y) as f64);
            let len2 = (vx * vx + vy * vy).max(1.0);
            let k = (((f.pos.x - a.x) as f64 * vx + (f.pos.y - a.y) as f64 * vy) / len2).clamp(0.0, 1.0);
            Point::new(a.x + (vx * k) as i64, a.y + (vy * k) as i64)
        };
        let mut pts: Vec<Point> = self.walls.iter().map(|w| w.nearest(f.pos)).collect();
        pts.extend(f.other_walls.iter().map(|s| near(s.a, s.b)));
        pts.extend(f.terrain_near(f.pos, range, 16).into_iter().map(|c| Frame::cell_point(c, f.pos)));
        pts.retain(|p| p.within(f.pos, range) && !p.within(f.pos, 12_000));
        pts
    }

    fn pull(&mut self, f: &Frame, aim: Point, out: &mut Vec<Effect>) {
        if f.tick < self.pull_at || self.air.is_some_and(|a| a.spent && f.tick < a.until) {
            return self.reject("pull");
        }
        let Some(w) = self.whip_points(f, d::PULL_RANGE).into_iter().min_by_key(|p| (p.d2(aim), p.x, p.y)) else {
            return self.reject("pull");
        };
        self.pull_at = f.tick + t(d::PULL_COOLDOWN_TICKS);
        let (land, ticks, speed) = self.sling(f, w);
        // fly-through: knocked aside, harder the faster he flies
        let scale = (speed * 100 / 1_500).clamp(100, 150);
        let dir = unit_dir(f.pos, land);
        for u in f.units.iter().filter(|u| f.skill_target(u)) {
            if seg_dist(u.pos, f.pos, land) <= d::PULL_HIT_RADIUS as f64 {
                let side = if (u.pos.x - f.pos.x) as f64 * -dir.1 + (u.pos.y - f.pos.y) as f64 * dir.0 >= 0.0 { perp(dir) } else { perp((-dir.0, -dir.1)) };
                let target = u.clone();
                self.shove(f, &target, side, 15_000, 10, pct(d::PULL_HIT_BASE, d::PULL_HIT_AD_RATIO, f.attack) * scale / 100, None, out);
            }
        }
        let double = self.air.is_none_or(|a| a.double_left || f.tick >= a.until);
        self.kicked.clear();
        self.fly(f, land, ticks, double, out);
        out.push(Effect::Fx {
            name: "bs_dash".into(),
            at: w,
            ticks: 18,
        });
    }

    fn run_air(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        let Some(a) = self.air else { return };
        if f.tick < a.until {
            out.push(Effect::Air { ticks: 3 });
            return;
        }
        // landed: the shockwave
        self.air = None;
        self.kicked.clear();
        for u in f.units.iter().filter(|u| f.skill_target(u) && u.pos.within(f.pos, d::JUMP_LAND_RADIUS)) {
            out.push(Effect::Damage {
                target: u.id,
                amount: pct(d::JUMP_LAND_BASE, d::JUMP_LAND_AD_RATIO, f.attack),
            });
        }
        out.push(Effect::Fx {
            name: "bs_land".into(),
            at: f.pos,
            ticks: 18,
        });
    }

    /// an air skill closes the chain (Air Dash never does)
    fn spend_air(&mut self) {
        if let Some(a) = self.air.as_mut() {
            a.spent = true;
            a.double_left = false;
        }
    }

    // -------------------------------------------------------------- the eight skills

    pub fn execute(&mut self, f: &Frame, o: Order, out: &mut Vec<Effect>) {
        let air = self.airborne(f);
        match o {
            Order::Jump(p) => self.jump(f, p, out),
            Order::Pull(p) => self.pull(f, p, out),
            Order::S1(p) => {
                if self.punches < 1 {
                    return self.reject("s1");
                }
                if air {
                    self.air_dash(f, p, out)
                } else {
                    self.raise_punch(f, p, out)
                }
            }
            Order::S2(p) => {
                if f.tick < self.s2_at {
                    return self.reject("s2");
                }
                if air {
                    self.slab_drop(f, p, out)
                } else {
                    self.hurl(f, p, out)
                }
            }
            Order::S3(p) => {
                if f.tick < self.s3_at {
                    return self.reject("s3");
                }
                if air {
                    self.whip_swing(f, p, out)
                } else {
                    self.crushing_clap(f, p, out)
                }
            }
            Order::Ult(p) => {
                if self.recast_ready(f) {
                    return self.ult_recast(f, out);
                }
                if !self.ult_ready(f) {
                    return self.reject("ult");
                }
                self.ult_charge(f, p, air, out)
            }
        }
    }

    fn reject(&mut self, why: &str) {
        self.rejection = why.into();
    }

    /// S1 ground: a wall rises right in front of him, his punch launches it forward, pushing (and slamming) enemies.
    fn raise_punch(&mut self, f: &Frame, aim: Point, out: &mut Vec<Effect>) {
        let dir = unit_dir(f.pos, aim);
        let c = f.pos.along(dir, d::PUNCH_RAISE_AHEAD);
        if f.terrain_at(c) {
            return self.reject("s1");
        }
        self.spend_punch(f);
        let id = self.raise_across(f, c, dir, out);
        let (a0, b0) = {
            let w = self.walls.iter().find(|w| w.id == id).unwrap();
            (w.a, w.b)
        };
        // the wall slides until terrain or another wall stops its centre
        let (stop, _) = self.push_path(f, c, dir, d::PUNCH_SLIDE, Some(id));
        let slide = c.dist(stop) as i64;
        let ticks = (t(d::PUNCH_SLIDE_TICKS) * slide.max(1) as u64 / d::PUNCH_SLIDE.max(1) as u64).max(1);
        if let Some(w) = self.walls.iter_mut().find(|w| w.id == id) {
            w.slide = Some(Slide {
                a0,
                b0,
                delta: Point::new(stop.x - c.x, stop.y - c.y),
                start: f.tick,
                ticks,
            });
        }
        let half = square_half();
        let n = perp(dir);
        let targets: Vec<Unit> = f.units.iter().filter(|u| f.skill_target(u)).cloned().collect();
        for u in targets {
            let rel = Point::new(u.pos.x - c.x, u.pos.y - c.y);
            let ahead = rel.x as f64 * dir.0 + rel.y as f64 * dir.1;
            let side = (rel.x as f64 * n.0 + rel.y as f64 * n.1).abs();
            if side > (half + 4_000) as f64 || ahead < -(half as f64) || ahead > slide as f64 + half as f64 {
                continue;
            }
            if ahead.abs() <= (half + 4_000) as f64 {
                out.push(Effect::KnockUp { target: u.id, ticks: t(d::PUNCH_KNOCK_TICKS) });
            }
            // carried ahead of the wall to where it stops
            let len = (slide as f64 - ahead).max(0.0) as i64 + half + 4_000;
            self.shove(f, &u, dir, len, ticks, pct(d::PUNCH_BASE, d::PUNCH_AD_RATIO, f.attack), Some(id), out);
        }
        // the punch itself is one of his hits on the wall
        self.hit_wall(f, id, out);
        out.push(Effect::Fx {
            name: "bs_rock".into(),
            at: c,
            ticks: 18,
        });
    }

    /// S1 air: Air Dash, a fast aerial dash that keeps him airborne and never ends the chain.
    fn air_dash(&mut self, f: &Frame, aim: Point, out: &mut Vec<Effect>) {
        self.spend_punch(f);
        let dir = unit_dir(f.pos, aim);
        let to = self.safe_land(f, f.pos.toward(aim, d::AIR_DASH_RANGE).clamp_map(), dir);
        let ticks = t(d::AIR_DASH_TICKS);
        for u in f.units.iter().filter(|u| f.skill_target(u) && seg_dist(u.pos, f.pos, to) <= d::AIR_DASH_HIT_RADIUS as f64) {
            out.push(Effect::Damage {
                target: u.id,
                amount: pct(d::AIR_DASH_BASE, d::AIR_DASH_AD_RATIO, f.attack),
            });
        }
        let a = self.air.unwrap();
        out.push(Effect::Glide { to, ticks });
        let until = (f.tick + ticks + t(d::AIR_DASH_EXTRA_AIR_TICKS)).max(a.until);
        out.push(Effect::Air { ticks: until - f.tick + 2 });
        self.air = Some(Air { land: to, until, ..a });
        out.push(Effect::Fx {
            name: "bs_dash".into(),
            at: f.pos,
            ticks: 12,
        });
    }

    /// the wall a throw uses: his nearest wall within grab range, else a chunk ripped off terrain (a new wall)
    fn grab_wall(&mut self, f: &Frame, out: &mut Vec<Effect>) -> Option<u8> {
        if let Some(w) = self
            .walls
            .iter()
            .filter(|w| w.dist(f.pos) <= d::HURL_GRAB_RANGE as f64 && w.slide.is_none())
            .min_by_key(|w| (w.dist(f.pos) as i64, w.id))
            .copied()
        {
            self.walls.retain(|x| x.id != w.id);
            out.push(Effect::Fx {
                name: "bs_dust".into(),
                at: w.mid(),
                ticks: 18,
            });
            return Some(w.integrity);
        }
        let chunk = f.terrain_near(f.pos, d::HURL_GRAB_RANGE, 1).first().map(|c| Frame::cell_point(*c, f.pos))?;
        out.push(Effect::Fx {
            name: "bs_break".into(),
            at: chunk,
            ticks: 18,
        });
        Some(d::WALL_INTEGRITY.max(1) as u8)
    }

    /// S2 ground: Hurl. A wall is thrown onto a point (no line): impact damage, an outward knock, slams into walls.
    fn hurl(&mut self, f: &Frame, aim: Point, out: &mut Vec<Effect>) {
        let to = f.pos.toward(aim, d::HURL_RANGE).clamp_map();
        let count = 1;
        let mut thrown = 0;
        for k in 0..count {
            let Some(integrity) = self.grab_wall(f, out) else { break };
            let spot = if k == 0 { to } else { f.pos.along(rotate(unit_dir(f.pos, to), if k == 1 { 25.0 } else { -25.0 }), f.pos.dist(to) as i64) };
            self.flying.push(Flying {
                kind: Throw::Hurl,
                to: f.nearest_ground(spot),
                dir: unit_dir(f.pos, spot),
                land_at: f.tick + t(d::HURL_FLIGHT_TICKS),
                integrity,
            });
            thrown += 1;
        }
        if thrown == 0 {
            return self.reject("s2");
        }
        self.s2_at = f.tick + t(d::HURL_COOLDOWN_TICKS);
    }

    /// S2 air: Slab Drop. A wall thrown down onto a point: an area knock-up.
    fn slab_drop(&mut self, f: &Frame, aim: Point, out: &mut Vec<Effect>) {
        let Some(integrity) = self.grab_wall(f, out) else { return self.reject("s2") };
        let to = f.nearest_ground(f.pos.toward(aim, d::SLAB_RANGE).clamp_map());
        self.flying.push(Flying {
            kind: Throw::Slab,
            to,
            dir: unit_dir(f.pos, to),
            land_at: f.tick + t(d::SLAB_FLIGHT_TICKS),
            integrity,
        });
        self.s2_at = f.tick + t(d::HURL_COOLDOWN_TICKS);
        self.spend_air();
    }

    fn run_flying(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        let due: Vec<Flying> = self.flying.iter().filter(|x| f.tick >= x.land_at).copied().collect();
        self.flying.retain(|x| f.tick < x.land_at);
        for x in due {
            // the landing is one of his hits on that wall
            let integrity = x.integrity.saturating_sub(1);
            let id = self.raise_across(f, x.to, x.dir, out);
            if let Some(w) = self.walls.iter_mut().find(|w| w.id == id) {
                w.integrity = integrity;
            }
            let targets: Vec<Unit> = f.units.iter().filter(|u| f.skill_target(u)).cloned().collect();
            match x.kind {
                Throw::Hurl => {
                    for u in targets.iter().filter(|u| u.pos.within(x.to, d::HURL_RADIUS)) {
                        let away = unit_dir(x.to, u.pos);
                        self.shove(f, u, away, d::HURL_KNOCK, 12, pct(d::HURL_BASE, d::HURL_AD_RATIO, f.attack), Some(id), out);
                    }
                }
                Throw::Slab => {
                    for u in targets.iter().filter(|u| u.pos.within(x.to, d::SLAB_RADIUS)) {
                        out.push(Effect::Damage {
                            target: u.id,
                            amount: pct(d::SLAB_BASE, d::SLAB_AD_RATIO, f.attack),
                        });
                        out.push(Effect::KnockUp {
                            target: u.id,
                            ticks: t(d::SLAB_KNOCK_TICKS),
                        });
                    }
                }
            }
            out.push(Effect::Fx {
                name: "bs_slam".into(),
                at: x.to,
                ticks: 24,
            });
            if integrity == 0 {
                self.crumble(f, id, true, out);
            }
        }
    }

    /// The V's two square walls at `k` (0 = open, 1 = shut): their centres on the left and right arms
    fn clap_walls(c: &Clap, k: f64) -> [Point; 2] {
        let half = d::CLAP_CONE_DEG as f64 / 2.0 * (1.0 - k);
        // The two square blocks occupy the far corners of the warned cone.
        [c.apex.along(rotate(c.dir, half), c.r1), c.apex.along(rotate(c.dir, -half), c.r1)]
    }

    /// S3 ground: Crushing Clap. Two walls rise at the upper left and right of a V and the whip snaps them shut.
    fn crushing_clap(&mut self, f: &Frame, aim: Point, out: &mut Vec<Effect>) {
        let dir = unit_dir(f.pos, aim);
        self.s3_at = f.tick + t(d::CLAP_COOLDOWN_TICKS);
        let vs = 1;
        for v in 0..vs {
            let apex = f.pos.along(dir, v * (d::CLAP_REACH * 2 / 3));
            let mut c = Clap {
                apex,
                dir,
                start: f.tick,
                left: 0,
                right: 0,
                r0: 15_000,
                r1: d::CLAP_REACH,
            };
            let [l, r] = Self::clap_walls(&c, 0.0);
            c.left = self.raise(f, l, d::WALL_INTEGRITY.max(1) as u8, out);
            c.right = self.raise(f, r, d::WALL_INTEGRITY.max(1) as u8, out);
            // who is caught inside the V: swept onto the centre line as it closes
            let half = d::CLAP_CONE_DEG as f64 / 2.0;
            let ticks = t(d::CLAP_CLOSE_TICKS);
            for u in f.units.iter().filter(|u| f.skill_target(u)) {
                let rel = unit_dir(apex, u.pos);
                let ang = (rel.0 * dir.0 + rel.1 * dir.1).clamp(-1.0, 1.0).acos().to_degrees();
                let r = apex.dist(u.pos) as i64;
                if ang <= half && r >= c.r0 - 4_000 && r <= c.r1 + 4_000 {
                    let to = apex.along(dir, r);
                    out.push(Effect::Knock { target: u.id, to, ticks });
                    out.push(Effect::Damage {
                        target: u.id,
                        amount: pct(d::CLAP_BASE, d::CLAP_AD_RATIO, f.attack),
                    });
                    let stun = d::CLAP_STUN_TICKS;
                    out.push(Effect::Stun {
                        target: u.id,
                        ticks: ticks + t(stun),
                    });
                    self.slam_guard.retain(|g| g.0 != u.id);
                    self.slam_guard.push((u.id, f.tick + ticks + t(d::SLAM_REPEAT_TICKS)));
                    self.last_slammed = Some((u.id, f.tick + 150));
                }
            }
            out.push(Effect::Fx {
                name: "bs_slam".into(),
                at: apex.along(dir, c.r1),
                ticks: 30,
            });
            self.claps.push(c);
        }
    }

    fn run_claps(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        let claps = std::mem::take(&mut self.claps);
        for c in claps {
            let k = (f.tick.saturating_sub(c.start) as f64 / d::CLAP_CLOSE_TICKS.max(1) as f64).min(1.0);
            let [l, r] = Self::clap_walls(&c, k);
            for (id, at) in [(c.left, l), (c.right, r)] {
                if let Some(w) = self.walls.iter_mut().find(|w| w.id == id) {
                    w.a = at;
                    w.b = at;
                }
            }
            if k >= 1.0 {
                // shut: the two walls fuse into one on the centre line
                self.walls.retain(|w| w.id != c.right);
                out.push(Effect::Fx {
                    name: "bs_wall_slam".into(),
                    at: c.apex.along(c.dir, c.r1),
                    ticks: 24,
                });
            } else {
                self.claps.push(c);
            }
        }
    }

    /// The release heading for a Whip Swing on `u`: toward his nearest teammate in reach (a gank), else away from the
    /// victim's own team.
    pub fn swing_heading(f: &Frame, u: &Unit) -> (f64, f64) {
        if let Some(a) = f
            .allies()
            .filter(|a| a.pos.within(f.pos, d::SWING_ALLY_REACH))
            .min_by_key(|a| (a.pos.d2(f.pos), a.id))
        {
            return unit_dir(f.pos, a.pos);
        }
        let mates: Vec<&Unit> = f.enemy_champions().filter(|e| e.id != u.id).collect();
        if mates.is_empty() {
            return unit_dir(f.pos, u.pos);
        }
        let (sx, sy) = mates.iter().fold((0i64, 0i64), |s, e| (s.0 + e.pos.x, s.1 + e.pos.y));
        let c = Point::new(sx / mates.len() as i64, sy / mates.len() as i64);
        unit_dir(c, f.pos)
    }

    /// S3 air: Whip Swing. Snare, swing 360° + x with the whip growing to 2x, slam at x into a wall (raised if none).
    fn whip_swing(&mut self, f: &Frame, aim: Point, out: &mut Vec<Effect>) {
        // an enemy champion first; with none in reach, a minion or jungle monster (every skill but the ult works on creeps)
        let near = |champ: bool| {
            f.units
                .iter()
                .filter(|u| f.skill_target(u) && u.champion == champ && u.pos.within(f.pos, d::SWING_RANGE))
                .min_by_key(|u| (u.pos.d2(aim), u.id))
                .cloned()
        };
        let Some(u) = near(true).or_else(|| near(false)) else {
            return self.reject("s3");
        };
        self.s3_at = f.tick + t(d::CLAP_COOLDOWN_TICKS);
        self.spend_air();
        let head = Self::swing_heading(f, &u);
        let start = unit_dir(f.pos, u.pos);
        let start_deg = start.1.atan2(start.0).to_degrees();
        let mut x = head.1.atan2(head.0).to_degrees() - start_deg;
        while x < 0.0 {
            x += 360.0;
        }
        let r0 = (f.pos.dist(u.pos) as i64).max(12_000);
        let r1 = (r0 * 2).min(d::SWING_MAX_RADIUS);
        // a wall on that heading within the whip's reach takes the slam
        let (_, into) = self.push_path(f, f.pos, head, r1 + square_half(), None);
        self.swing = Some(Swing {
            target: u.id,
            center: f.pos,
            start_deg,
            turn_deg: 360.0 + x,
            r0,
            r1,
            start: f.tick,
            next: f.tick,
            into,
        });
        out.push(Effect::Fx {
            name: "bs_dash".into(),
            at: u.pos,
            ticks: 12,
        });
    }

    fn run_swing(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        let Some(s) = self.swing else { return };
        let Some(u) = f.unit(s.target).filter(|u| f.skill_target(u)).cloned() else {
            self.swing = None;
            return;
        };
        let total = t(d::SWING_SPIN_TICKS).max(6);
        let end = s.start + total;
        if f.tick >= end {
            self.swing = None;
            let head = rotate((1.0, 0.0), s.start_deg + s.turn_deg);
            let hit = match s.into {
                Hit::None => {
                    // no wall there: he raises one and slams them into it
                    let id = self.raise_across(f, s.center.along(head, s.r1 + square_half() + 4_000), head, out);
                    Hit::Mine(id)
                }
                h => h,
            };
            self.slam(f, u.id, hit, pct(d::SWING_BASE, d::SWING_AD_RATIO, f.attack), out);
            return;
        }
        if f.tick < s.next {
            return;
        }
        // the swing in 6-tick legs along the growing circle
        let k1 = ((f.tick + 6 - s.start) as f64 / total as f64).min(1.0);
        let deg = s.start_deg + s.turn_deg * k1;
        let r = s.r0 + ((s.r1 - s.r0) as f64 * k1) as i64;
        let mut to = s.center.along(rotate((1.0, 0.0), deg), r).clamp_map();
        if k1 >= 1.0 {
            // the last leg stops at the wall's face
            let head = rotate((1.0, 0.0), deg);
            to = self.push_path(f, s.center, head, r, None).0;
        }
        out.push(Effect::Knock { target: u.id, to, ticks: 6 });
        out.push(Effect::Stun { target: u.id, ticks: 8 });
        if let Some(sw) = self.swing.as_mut() {
            sw.next = f.tick + 6;
        }
    }

    /// The ult's 0.5 s charge: he holds still (in the air he hangs there, airborne), then throws or punches the rock.
    fn ult_charge(&mut self, f: &Frame, aim: Point, air: bool, out: &mut Vec<Effect>) {
        let ticks = t(d::ULT_CHARGE_TICKS);
        self.ult_at = f.tick + t(d::ULT_COOLDOWN_TICKS);
        self.charging = Some((air, aim.clamp_map(), f.tick + ticks));
        self.recast = None;
        out.push(Effect::Root { ticks });
        if air {
            // he hangs in the air through the charge
            if let Some(a) = self.air.as_mut() {
                a.until = f.tick + ticks + 2;
                a.land = f.pos;
                a.spent = true;
                a.double_left = false;
            }
            out.push(Effect::Glide { to: f.pos, ticks: 1 });
            out.push(Effect::Air { ticks: ticks + 4 });
        }
        out.push(Effect::Fx {
            name: "bs_rise".into(),
            at: f.pos,
            ticks: 30,
        });
        out.push(Effect::Log {
            text: if air { "Meteor Punch: charging".into() } else { "Boulder Throw: charging".into() },
        });
    }

    fn run_ult(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        // the charge ends: the throw (or the punch)
        if let Some((air, aim, at)) = self.charging {
            if f.tick >= at {
                self.charging = None;
                let flight = (travel(f.pos, aim, d::ULT_ROCK_SPEED)).max(t(d::ULT_MIN_FLIGHT_TICKS));
                self.boulders.push((air, aim, f.tick + flight));
                out.push(Effect::Fx {
                    name: if air { "bs_rock".into() } else { "bs_dust".into() },
                    at: f.pos,
                    ticks: 18,
                });
            } else if air {
                out.push(Effect::Air { ticks: 3 });
            }
        }
        // the rock lands
        let due: Vec<(bool, Point, u64)> = self.boulders.iter().filter(|b| f.tick >= b.2).copied().collect();
        self.boulders.retain(|b| f.tick < b.2);
        for (air, at, _) in due {
            let at = f.nearest_ground(at);
            if air {
                // punched apart: burning meteors rain over the area, one every few ticks
                for k in 0..d::ULT_METEOR_COUNT.max(1) {
                    let spot = if k == 0 { at } else { at.along(rotate((1.0, 0.0), k as f64 * 360.0 / (d::ULT_METEOR_COUNT - 1).max(1) as f64), d::ULT_METEOR_SPREAD) };
                    self.meteors.push((spot.clamp_map(), f.tick + k as u64 * t(d::ULT_METEOR_EVERY_TICKS)));
                }
            } else {
                for u in f.units.iter().filter(|u| f.skill_target(u) && u.pos.within(at, d::ULT_RADIUS)) {
                    out.push(Effect::Damage {
                        target: u.id,
                        amount: pct(d::ULT_BASE, d::ULT_AD_RATIO, f.attack),
                    });
                    out.push(Effect::KnockUp { target: u.id, ticks: t(d::ULT_KNOCK_TICKS) });
                    out.push(Effect::Slow {
                        target: u.id,
                        pct: d::ULT_SLOW_PCT,
                        ticks: t(d::ULT_SLOW_TICKS),
                    });
                }
                out.push(Effect::Fx {
                    name: "bs_slam".into(),
                    at,
                    ticks: 36,
                });
            }
            // the rock stands as one of his walls; the recast whips him to it
            self.raise_across(f, at, unit_dir(f.pos, at), out);
            self.recast = Some((at, f.tick + t(d::ULT_RECAST_TICKS)));
        }
        let landed: Vec<(Point, u64)> = self.meteors.iter().filter(|m| f.tick >= m.1).copied().collect();
        self.meteors.retain(|m| f.tick < m.1);
        for (spot, _) in landed {
            for u in f.units.iter().filter(|u| f.skill_target(u) && u.pos.within(spot, d::ULT_METEOR_HIT_RADIUS)) {
                out.push(Effect::Damage {
                    target: u.id,
                    amount: pct(d::ULT_METEOR_BASE, d::ULT_METEOR_AD_RATIO, f.attack),
                });
                out.push(Effect::KnockUp {
                    target: u.id,
                    ticks: t(d::ULT_METEOR_KNOCK_TICKS),
                });
                out.push(Effect::Slow {
                    target: u.id,
                    pct: d::ULT_METEOR_SLOW_PCT,
                    ticks: t(d::ULT_METEOR_SLOW_TICKS),
                });
                // the burn: renewed, not stacked
                self.burns.retain(|b| b.0 != u.id);
                self.burns.push((u.id, f.tick + t(d::ULT_BURN_TICKS), f.tick + 60));
            }
            out.push(Effect::Fx {
                name: "bs_rock".into(),
                at: spot,
                ticks: 24,
            });
        }
        // burning: damage once a second
        let burn = pct(d::ULT_BURN_BASE, d::ULT_BURN_AD_RATIO, f.attack);
        for b in self.burns.iter_mut() {
            if f.tick >= b.2 && f.tick <= b.1 {
                b.2 = f.tick + 60;
                if f.unit(b.0).is_some_and(|u| f.skill_target(u)) {
                    out.push(Effect::Damage { target: b.0, amount: burn });
                }
            }
        }
        self.burns.retain(|b| f.tick < b.1);
    }

    /// The recast: his whip catches the rock and slingshots him to where it landed (airborne, the chain open).
    fn ult_recast(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        let Some((at, _)) = self.recast.take() else { return };
        let dir = unit_dir(f.pos, at);
        // he lands at the rock, on his side of it
        let land = self.safe_land(f, at.along(dir, -(square_half() + 10_000)), (-dir.0, -dir.1));
        let dist = f.pos.dist(land) as i64;
        let speed = (d::PULL_SPEED_BASE + dist / d::PULL_SPEED_DIVISOR.max(1)).min(d::ULT_RECAST_SPEED_CAP);
        let ticks = travel(f.pos, land, speed);
        self.kicked.clear();
        self.fly(f, land, ticks, true, out);
        out.push(Effect::Fx {
            name: "bs_dash".into(),
            at,
            ticks: 18,
        });
    }

    // -------------------------------------------------------------- AI (Kicker's flat list)

    pub fn hp_pct(f: &Frame) -> i64 {
        f.hp * 100 / f.max_hp.max(1)
    }
    pub fn hurt(&self, f: &Frame) -> bool {
        let top = self.hp_log.iter().map(|h| h.1).max().unwrap_or(f.hp);
        (top - f.hp) * 100 >= f.max_hp * d::AI_HURT_PCT
    }
    pub fn danger_at(f: &Frame, p: Point) -> i64 {
        f.enemy_champions()
            .filter(|u| u.pos.within(p, d::AI_FIGHT_RANGE))
            .map(|u| (d::AI_FIGHT_RANGE as f64 - u.pos.dist(p)) as i64)
            .sum()
    }
    pub fn outnumbered(f: &Frame, at: Point) -> bool {
        let enemies = f.enemy_champions().filter(|u| u.pos.within(at, 60_000)).count();
        let ours = 1 + f.allies().filter(|a| a.pos.within(at, 60_000)).count();
        enemies >= ours + 2 || (enemies >= 3 && ours < 2)
    }
    pub fn isolated(f: &Frame, u: &Unit) -> bool {
        !f.enemy_champions().any(|o| o.id != u.id && o.pos.within(u.pos, 60_000))
    }
    pub fn in_fight(f: &Frame, u: &Unit) -> bool {
        f.allies().any(|a| a.pos.within(u.pos, d::AI_FIGHT_RANGE))
    }
    /// Is the target slammable: a wall (any) within reach behind it from him?
    pub fn wall_behind(&self, f: &Frame, u: &Unit, reach: i64) -> bool {
        self.push_path(f, u.pos, unit_dir(f.pos, u.pos), reach, None).1 != Hit::None
    }
    pub fn would_start(&self, f: &Frame, u: &Unit) -> bool {
        if Self::in_fight(f, u) && !Self::outnumbered(f, u.pos) {
            return true;
        }
        let target_pct = u.hp * 100 / u.max_hp.max(1);
        let allies = f.allies().filter(|a| a.pos.within(u.pos, d::AI_ENGAGE_RANGE)).count() as i64;
        let enemies = f.enemy_champions().filter(|o| o.id != u.id && o.pos.within(u.pos, 60_000)).count() as i64;
        let mut score = f.p(EGO) - 50 + (100 - target_pct) / 2 + (Self::hp_pct(f) - 50) / 2 + allies * 15 - enemies * 20;
        if Self::isolated(f, u) {
            score += 15;
        }
        if self.last_slammed.is_some_and(|l| l.0 == u.id) {
            score += 30;
        }
        score >= 0
    }
    /// Kicker's target score: re-read every tick (no lock-on); under its tower never; nearer, lower, alone, fighting
    /// his team, slammable: better.
    pub fn score(&self, f: &Frame, u: &Unit, range: i64) -> Option<i64> {
        if !u.pos.within(f.pos, range) || Self::outnumbered(f, u.pos) || !self.would_start(f, u) || f.enemy_tower_at(u.pos).is_some() {
            return None;
        }
        let pct = u.hp * 100 / u.max_hp.max(1);
        let near = 60 - (u.pos.dist(f.pos) as i64 * 60 / range.max(1));
        let enemies = f.enemy_champions().filter(|o| o.id != u.id && o.pos.within(u.pos, 60_000)).count() as i64;
        let allies = f.allies().filter(|a| a.pos.within(u.pos, d::AI_ENGAGE_RANGE)).count() as i64;
        let mut s = (100 - pct) + near + allies * 10 - enemies * 15;
        if Self::isolated(f, u) {
            s += 20;
        }
        if Self::in_fight(f, u) {
            s += 20;
        }
        if self.last_slammed.is_some_and(|l| l.0 == u.id) {
            s += 30;
        }
        if u.pos.within(f.pos, d::CLAP_REACH) && self.wall_behind(f, u, 40_000) {
            s += 10;
        }
        Some(s)
    }
    pub fn pick(&self, f: &Frame, range: i64) -> Option<Unit> {
        f.enemy_champions()
            .filter_map(|u| self.score(f, u, range).map(|s| (s, u)))
            .max_by(|a, b| a.0.cmp(&b.0).then(b.1.id.cmp(&a.1.id)))
            .map(|(_, u)| u.clone())
    }

    /// A Whip Pull that brings him toward `at`: the wall whose slingshot lands nearest it, at least 20,000 nearer than
    /// he is, never under an enemy tower unless `at` is.
    pub fn pull_toward(&self, f: &Frame, at: Point) -> Option<Order> {
        if f.tick < self.pull_at {
            return None;
        }
        let now = f.pos.dist(at);
        let tower_ok = f.enemy_tower_at(at).is_some();
        self.whip_points(f, d::PULL_RANGE)
            .into_iter()
            .map(|w| (w, self.sling(f, w).0))
            .filter(|(_, l)| l.dist(at) + 20_000.0 < now && (tower_ok || f.enemy_tower_at(*l).is_none()))
            .min_by_key(|(_, l)| (l.d2(at), l.x, l.y))
            .map(|(w, _)| Order::Pull(w))
    }
    /// A jump toward `at` that brings him at least 20,000 nearer and never under an enemy tower unless `at` is.
    pub fn jump_toward(&self, f: &Frame, at: Point) -> Option<Order> {
        let (_, range) = self.jump_kind(f)?;
        let land = f.pos.toward(at, range);
        let tower_ok = f.enemy_tower_at(at).is_some();
        (land.dist(at) + 20_000.0 < f.pos.dist(at) && (tower_ok || f.enemy_tower_at(land).is_none())).then_some(Order::Jump(at))
    }
    /// mobility toward a spot: the slingshot first (it's free of charges), then a jump
    pub fn move_toward(&self, f: &Frame, at: Point) -> Option<Order> {
        self.pull_toward(f, at).or_else(|| self.jump_toward(f, at))
    }

    /// enemies inside a Crushing Clap aimed at `aim`
    pub fn clap_hits(f: &Frame, aim: Point) -> usize {
        let dir = unit_dir(f.pos, aim);
        let half = d::CLAP_CONE_DEG as f64 / 2.0;
        f.units
            .iter()
            .filter(|u| f.skill_target(u))
            .filter(|u| {
                let rel = unit_dir(f.pos, u.pos);
                let ang = (rel.0 * dir.0 + rel.1 * dir.1).clamp(-1.0, 1.0).acos().to_degrees();
                let r = f.pos.dist(u.pos) as i64;
                ang <= half && (11_000..=d::CLAP_REACH + 4_000).contains(&r)
            })
            .count()
    }

    fn engage(&mut self, f: &Frame, u: &Unit) -> Option<Order> {
        let dist = f.pos.dist(u.pos) as i64;
        let air = self.airborne(f);
        let a_free = self.air.is_some_and(|a| !a.spent);
        if air && a_free {
            // airborne: Whip Swing in reach, Slab Drop on a group, else Air Dash in (it chains into anything)
            if f.tick >= self.s3_at && dist <= d::SWING_RANGE {
                self.reason = "Whip Swing".into();
                return Some(Order::S3(u.pos));
            }
            let group = f.enemy_champions().filter(|e| e.pos.within(u.pos, d::SLAB_RADIUS)).count();
            if f.tick >= self.s2_at && dist <= d::SLAB_RANGE && group >= 2 {
                self.reason = "Slab Drop on a group".into();
                return Some(Order::S2(u.pos));
            }
            if self.punches > 0 && dist > d::SWING_RANGE && dist <= d::AIR_DASH_RANGE + d::SWING_RANGE {
                self.reason = "Air Dash in".into();
                return Some(Order::S1(u.pos));
            }
            if f.tick >= self.s2_at && dist <= d::SLAB_RANGE {
                self.reason = "Slab Drop".into();
                return Some(Order::S2(u.pos));
            }
            return None;
        }
        if air {
            return None;
        }
        // on the ground: the V slam on a group (or a low target in the cone)
        if f.tick >= self.s3_at && dist <= d::CLAP_REACH {
            let hits = Self::clap_hits(f, u.pos);
            if hits >= 2 || (hits >= 1 && u.hp * 100 < u.max_hp * 50) || hits >= 1 && self.last_slammed.is_some_and(|l| l.0 == u.id) {
                self.reason = "Crushing Clap".into();
                return Some(Order::S3(u.pos));
            }
        }
        // the punch: a wall launched into them, best with a wall behind them (a slam)
        if self.punches > 0 && dist <= d::PUNCH_RAISE_AHEAD + d::PUNCH_SLIDE - 20_000 && (self.wall_behind(f, u, 60_000) || dist <= 45_000) {
            self.reason = "Raise & Punch".into();
            return Some(Order::S1(u.pos));
        }
        // a throw onto them (a wall to throw in reach)
        if f.tick >= self.s2_at && dist <= d::HURL_RANGE && (!self.walls.is_empty() || !f.terrain_near(f.pos, d::HURL_GRAB_RANGE, 1).is_empty()) {
            self.reason = "Hurl".into();
            return Some(Order::S2(u.pos));
        }
        // mobility first: jump in for the air skills (Whip Swing), or slingshot over a wall toward them
        if f.tick >= self.s3_at && dist > d::SWING_RANGE {
            if let Some(o) = self.jump_toward(f, u.pos) {
                self.reason = "Jump in for Whip Swing".into();
                return Some(o);
            }
        }
        if dist > d::CLAP_REACH {
            if let Some(o) = self.pull_toward(f, u.pos) {
                self.reason = "Whip Pull engage".into();
                return Some(o);
            }
        }
        None
    }

    pub fn decide(&mut self, f: &Frame) -> Option<Order> {
        if f.stunned && !self.airborne(f) {
            return None;
        }
        // in the middle of a swing, nothing else
        if self.swing.is_some() {
            return None;
        }
        // 0. the ult. Recast first: whip to the rock to follow up a hit there (or to escape to it).
        if let Some((rock, _)) = self.recast.filter(|r| f.tick < r.1) {
            let fight = f.enemy_champions().filter(|u| u.pos.within(rock, d::ULT_RADIUS + 20_000)).count();
            let pressed = f.enemy_champions().filter(|u| u.pos.within(f.pos, 40_000)).count();
            let follow = fight > 0 && !Self::outnumbered(f, rock) && Self::hp_pct(f) >= 40;
            let escape = pressed > 0 && self.hurt(f) && Self::danger_at(f, rock) < Self::danger_at(f, f.pos) / 2;
            if follow || escape {
                self.reason = if follow { "Ult recast: whip to the rock".into() } else { "Ult recast: escape to the rock".into() };
                return Some(Order::Ult(rock));
            }
        }
        // The charge holds him for 0.5 s: only with no enemy champion within his fight range. Anywhere on the map: a group of three
        // (or two his team is fighting), else a low champion trying to get away. In the air it's Meteor Punch.
        if self.ult_ready(f) && !f.enemy_champions().any(|u| u.pos.within(f.pos, d::AI_FIGHT_RANGE)) && self.air.is_none_or(|a| !a.spent || f.tick >= a.until) {
            let around = |u: &Unit| f.enemy_champions().filter(|o| o.pos.within(u.pos, d::ULT_RADIUS)).count() as i64;
            let best = f
                .enemy_champions()
                .filter(|u| f.enemy_tower_at(u.pos).is_none() || around(u) >= d::AI_TEAMFIGHT_COUNT)
                .map(|u| (around(u), Self::in_fight(f, u), u))
                .filter(|(n, fight, u)| *n >= d::AI_TEAMFIGHT_COUNT || (*n >= 2 && *fight) || (u.hp * 100 < u.max_hp * 25 && *fight))
                .max_by_key(|(n, fight, u)| (*n, *fight, std::cmp::Reverse(u.id)));
            if let Some((_, _, u)) = best {
                self.reason = if self.airborne(f) { "Meteor Punch".into() } else { "Boulder Throw".into() };
                return Some(Order::Ult(u.pos));
            }
        }
        // 1. escape after real damage: the slingshot or a jump toward safety, else a wall in their way
        let pressed = f.enemy_champions().filter(|u| u.pos.within(f.pos, 40_000)).count();
        if pressed > 0 && self.hurt(f) && Self::hp_pct(f) < 45 {
            let safe = f.allies().filter(|a| !a.pos.within(f.pos, 20_000)).min_by_key(|a| (Self::danger_at(f, a.pos), a.id)).map_or(f.home, |a| a.pos);
            if let Some(o) = self.move_toward(f, safe) {
                self.reason = "Escape".into();
                return Some(o);
            }
            if self.punches > 0 && !self.airborne(f) {
                if let Some(d) = f.enemy_champions().min_by_key(|u| (u.pos.d2(f.pos), u.id)) {
                    self.reason = "Punch a wall into the diver".into();
                    return Some(Order::S1(d.pos));
                }
            }
        }
        // 2. peel: an enemy on a hurt teammate
        let peel = f
            .allies()
            .filter(|a| a.hp * 100 < a.max_hp * 50)
            .filter_map(|a| f.enemy_champions().filter(|e| e.pos.within(a.pos, d::AI_PEEL_RANGE)).min_by_key(|e| (e.pos.d2(a.pos), e.id)).map(|e| (a.clone(), e.clone())))
            .min_by_key(|(a, _)| (a.hp * 100 / a.max_hp.max(1), a.id));
        if let Some((_, diver)) = peel {
            if !Self::outnumbered(f, diver.pos) {
                if let Some(o) = self.engage(f, &diver) {
                    return Some(o);
                }
                if let Some(o) = self.move_toward(f, diver.pos) {
                    self.reason = "Peel".into();
                    return Some(o);
                }
            }
        }
        // 3. follow-up: go hit what he slammed
        if let Some(u) = self.last_slammed.and_then(|l| f.unit(l.0)).filter(|u| u.champion && f.skill_target(u)).cloned() {
            if let Some(o) = self.engage(f, &u) {
                return Some(o);
            }
            if !u.pos.within(f.pos, d::BASIC_RANGE + 8_000) {
                if let Some(o) = self.move_toward(f, u.pos) {
                    self.reason = "Follow up".into();
                    return Some(o);
                }
                self.walk(f, u.pos);
                return None;
            }
        }
        // 4. help a teammate's fight farther off
        let help = f
            .allies()
            .filter(|a| a.pos.within(f.pos, d::AI_HELP_RANGE) && !a.pos.within(f.pos, d::AI_FIGHT_RANGE))
            .filter(|a| f.enemy_champions().any(|e| e.pos.within(a.pos, d::AI_FIGHT_RANGE)) && !Self::outnumbered(f, a.pos))
            .min_by_key(|a| (a.hp * 100 / a.max_hp.max(1), a.pos.d2(f.pos), a.id))
            .map(|a| a.pos);
        if let Some(at) = help {
            if let Some(o) = self.move_toward(f, at) {
                self.reason = "Help".into();
                return Some(o);
            }
            self.walk(f, at);
            return None;
        }
        // 5. the best target: mobility first, then skills
        if let Some(u) = self.pick(f, d::AI_ENGAGE_RANGE) {
            if let Some(o) = self.engage(f, &u) {
                return Some(o);
            }
            self.walk(f, u.pos);
            return None;
        }
        // 6. farm: every skill but the ult works on minions and jungle monsters (only when calm: no enemy champion near)
        if f.enemy_champions().all(|u| !u.pos.within(f.pos, d::AI_ENGAGE_RANGE)) {
            if let Some(o) = self.farm(f) {
                return Some(o);
            }
        }
        self.reason.clear();
        None
    }

    /// a camp monster or big creep: worth a single-target skill
    fn big(u: &Unit) -> bool {
        u.max_hp >= 1_000
    }
    /// His farming (basalt-1.4): all six non-ult skills on farmable units. Ground: Crushing Clap on two creeps or a
    /// monster, Raise & Punch into a camp, Hurl on two creeps or a monster. Airborne: Whip Swing a monster, Slab Drop on
    /// a group or a monster, Air Dash onto a camp. A jump onto a camp in reach opens the air skills.
    pub fn farm(&mut self, f: &Frame) -> Option<Order> {
        let farm: Vec<Unit> = f.units.iter().filter(|u| u.farmable && f.skill_target(u) && u.pos.within(f.pos, d::HURL_RANGE)).cloned().collect();
        if farm.is_empty() {
            return None;
        }
        let around = |p: Point, r: i64| farm.iter().filter(|o| o.pos.within(p, r)).count();
        let best = |pick: &dyn Fn(&Unit) -> bool, r: i64| -> Option<Unit> {
            farm.iter()
                .filter(|u| pick(u))
                .max_by_key(|u| (around(u.pos, r), Self::big(u), std::cmp::Reverse(u.pos.d2(f.pos)), std::cmp::Reverse(u.id)))
                .cloned()
        };
        let air = self.airborne(f);
        if air {
            if !self.air.is_some_and(|a| !a.spent) {
                return None;
            }
            if f.tick >= self.s3_at {
                if let Some(u) = best(&|u| Self::big(u) && u.pos.within(f.pos, d::SWING_RANGE), 0) {
                    self.reason = "Farm: Whip Swing".into();
                    return Some(Order::S3(u.pos));
                }
            }
            if f.tick >= self.s2_at {
                if let Some(u) = best(&|u| u.pos.within(f.pos, d::SLAB_RANGE), d::SLAB_RADIUS).filter(|u| around(u.pos, d::SLAB_RADIUS) >= 2 || Self::big(u)) {
                    self.reason = "Farm: Slab Drop".into();
                    return Some(Order::S2(u.pos));
                }
            }
            if self.punches > 0 {
                if let Some(u) = best(&|u| Self::big(u) && !u.pos.within(f.pos, d::SWING_RANGE) && u.pos.within(f.pos, d::AIR_DASH_RANGE + d::SWING_RANGE), 0) {
                    self.reason = "Farm: Air Dash".into();
                    return Some(Order::S1(u.pos));
                }
            }
            return None;
        }
        // a jump onto a camp opens the air skills (only with both charges, so one stays for fights)
        if self.jumps >= d::JUMP_CHARGES && (f.tick >= self.s3_at || f.tick >= self.s2_at) {
            if let Some(u) = best(&|u| Self::big(u) && u.pos.within(f.pos, d::JUMP_RANGE + d::SWING_RANGE) && !u.pos.within(f.pos, 30_000), 0) {
                if f.enemy_tower_at(u.pos).is_none() {
                    self.reason = "Farm: jump onto the camp".into();
                    return Some(Order::Jump(u.pos));
                }
            }
        }
        if f.tick >= self.s3_at {
            if let Some(u) = best(&|u| u.pos.within(f.pos, d::CLAP_REACH), 0).filter(|u| Self::clap_hits(f, u.pos) >= 2 || Self::big(u) && Self::clap_hits(f, u.pos) >= 1) {
                self.reason = "Farm: Crushing Clap".into();
                return Some(Order::S3(u.pos));
            }
        }
        if self.punches > 0 {
            if let Some(u) = best(&|u| Self::big(u) && u.pos.within(f.pos, 45_000), 0) {
                self.reason = "Farm: Raise & Punch".into();
                return Some(Order::S1(u.pos));
            }
        }
        let throwable = !self.walls.is_empty() || !f.terrain_near(f.pos, d::HURL_GRAB_RANGE, 1).is_empty();
        if f.tick >= self.s2_at && throwable {
            if let Some(u) = best(&|_| true, d::HURL_RADIUS).filter(|u| around(u.pos, d::HURL_RADIUS) >= 2 || Self::big(u)) {
                self.reason = "Farm: Hurl".into();
                return Some(Order::S2(u.pos));
            }
        }
        None
    }

    /// a purposeful walk (the shared input AI turns it into a move order)
    fn walk(&mut self, f: &Frame, to: Point) {
        if f.tick < self.steer_at || self.airborne(f) || f.enemy_tower_at(to).is_some() && f.enemy_tower_at(f.pos).is_none() {
            return;
        }
        self.steer_at = f.tick + 6;
        self.pending_walk = Some(to);
    }
}
