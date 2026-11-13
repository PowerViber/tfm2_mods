//! Kicker, the rooftop street fighter: a deterministic core (snapshot in, effects out). `kicker.rs` builds the frame
//! from the sim and applies the effects; the Lab (`native/kicker_lab`) runs this same file.
//!
//!   P1 Rooftop Runner: he stands and runs on wall cells (a named `ignore_wall` buff): faster, 15% less damage, hidden
//!      from enemies farther than 40,000. Every jump down is a Drop-in (invulnerable while dropping, a knock-up on
//!      landing, +15% on his next hit; 6 s). Landing on the ground shields him (10 s). A stun on a wall drops him.
//!   P2 Marked Dash (2 charges): his hits mark units (never towers); he dashes next to a marked unit within 75,000.
//!   P3 Bruised: everyone he crowd-controls: no HP regen, less healing, +3% damage taken per hit of his (max 5).
//!   S1 Wall Hop / Drop Kick: onto a wall (its own cooldown), wall to wall (4 charges), off a wall onto an enemy
//!      (no charge). No wall near: a short hop.
//!   S2 Rock Throw / Rock Rain: a rock through everything that marks every unit it hits; from a wall, 3 in a fan.
//!   S3 Homecoming Kick / Skyfall Kick (2 charges): on the ground, dash in and kick an enemy champion away from him
//!      (the flying body knocks up 2 more; a wall stops it and stuns). From a wall: a long dive that carries the first
//!      enemy champion in its line to the end, through walls; he lands behind it.
//!   Ult Second Wind: a tag where he stands; for 8 s a death sends him back to it at full HP; when the time runs out he
//!      goes back with the HP he had.
//!
//! His AI is his own. It reads what the bridge puts in the frame and the 13 athlete values shape its choices.
//! Every skill of his skips towers.
use crate::kicker_data as d;
use serde::{Deserialize, Serialize};

pub const MAP: i64 = 960_000;
pub const CELL: i64 = 32_000;
pub const CELLS: i64 = 30;

// the 13 athlete values in the frame's profile
pub const EGO: usize = 0;
pub const JUDGEMENT: usize = 1;
pub const POSITIONING: usize = 2;
pub const CONTROL_SPEED: usize = 3;
pub const SKILL_HIT: usize = 4;
pub const SKILL_AVOID: usize = 5;
pub const AGGRESSIVE: usize = 6;
pub const ORDER: usize = 7;
pub const ROAMING: usize = 8;
pub const MACRO: usize = 9;
pub const MICRO: usize = 10;
pub const AWARENESS: usize = 11;
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
        self.d2(o) <= (r as i128) * (r as i128)
    }
    pub fn toward(self, to: Point, len: i64) -> Point {
        let dd = self.dist(to);
        if dd < 1.0 {
            return Point::new(self.x + len, self.y);
        }
        let k = len as f64 / dd;
        Point::new(self.x + ((to.x - self.x) as f64 * k).round() as i64, self.y + ((to.y - self.y) as f64 * k).round() as i64)
    }
    /// `len` along the unit direction `u`
    pub fn along(self, u: (f64, f64), len: i64) -> Point {
        Point::new(self.x + (u.0 * len as f64).round() as i64, self.y + (u.1 * len as f64).round() as i64)
    }
    pub fn clamp_map(self) -> Point {
        Point::new(self.x.clamp(1_000, MAP - 1_000), self.y.clamp(1_000, MAP - 1_000))
    }
    pub fn on_map(self) -> bool {
        (0..=MAP).contains(&self.x) && (0..=MAP).contains(&self.y)
    }
    /// distance from self to the segment a-b, and how far along it (0..=1)
    pub fn seg(self, a: Point, b: Point) -> (f64, f64) {
        let (vx, vy) = ((b.x - a.x) as f64, (b.y - a.y) as f64);
        let len2 = vx * vx + vy * vy;
        let k = if len2 == 0.0 { 0.0 } else { (((self.x - a.x) as f64 * vx + (self.y - a.y) as f64 * vy) / len2).clamp(0.0, 1.0) };
        let q = (a.x as f64 + vx * k, a.y as f64 + vy * k);
        (((self.x as f64 - q.0).powi(2) + (self.y as f64 - q.1).powi(2)).sqrt(), k)
    }
}

/// The unit direction from `a` to `b` (east when they coincide).
pub fn unit_dir(a: Point, b: Point) -> (f64, f64) {
    let (dx, dy) = ((b.x - a.x) as f64, (b.y - a.y) as f64);
    let l = dx.hypot(dy);
    if l < 1.0 {
        (1.0, 0.0)
    } else {
        (dx / l, dy / l)
    }
}
fn rotate(u: (f64, f64), deg: f64) -> (f64, f64) {
    let (s, c) = deg.to_radians().sin_cos();
    (u.0 * c - u.1 * s, u.0 * s + u.1 * c)
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
    pub radius: i64,
    #[serde(default)]
    pub attack: i64,
    pub champion: bool,
    pub tower: bool,
    pub visible: bool,
    pub targetable: bool,
    #[serde(default)]
    pub farmable: bool,
    #[serde(default)]
    pub stunned: bool,
    /// V1's parry is up: a committed strike would be returned
    #[serde(default)]
    pub parry: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "aim", rename_all = "snake_case")]
pub enum Order {
    /// S1 from the ground: onto the nearest wall within reach
    Mount,
    /// S1: wall to wall (on a wall), or a short hop (on the ground)
    Hop(Point),
    /// S1 from a wall: down toward a point, kicking whoever is at the landing
    DropKick(Point),
    /// S2 toward a point (a fan of three from a wall)
    Rock(Point),
    /// Marked Dash to this marked unit
    Dash(usize),
    /// S3 on the ground: kick this enemy champion toward the point
    Kick(usize, Point),
    /// S3 with Flash: blink to the kick spot first (or during the wind-up: re-aim)
    FlashKick(usize, Point),
    /// S3 from a wall: dive toward the point
    Skyfall(Point),
    Ult,
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
    pub stunned: bool,
    pub ult_unlocked: bool,
    #[serde(default)]
    pub flash_ready: bool,
    #[serde(default)]
    pub home: Point,
    pub units: Vec<Unit>,
    /// wall cells (`CELLS` x `CELLS`)
    #[serde(default)]
    pub terrain: Vec<bool>,
    pub profile: [i64; 13],
    /// units his accepted basic attacks hit since the last frame
    #[serde(default)]
    pub basic_hits: Vec<usize>,
    pub manual: bool,
    pub order: Option<Order>,
}
impl Frame {
    pub fn enemy(&self, u: &Unit) -> bool {
        u.team != self.team && u.visible && u.targetable && u.hp > 0
    }
    /// enemies his skills may hit: never a tower
    pub fn skill_target(&self, u: &Unit) -> bool {
        self.enemy(u) && !u.tower
    }
    pub fn enemy_champions(&self) -> impl Iterator<Item = &Unit> {
        self.units.iter().filter(move |u| u.champion && self.enemy(u))
    }
    pub fn allies(&self) -> impl Iterator<Item = &Unit> {
        self.units.iter().filter(move |u| u.champion && u.team == self.team && u.hp > 0 && u.id != self.owner)
    }
    pub fn p(&self, i: usize) -> i64 {
        self.profile.get(i).copied().unwrap_or(50).clamp(0, 100)
    }
    pub fn unit(&self, id: usize) -> Option<&Unit> {
        self.units.iter().find(|u| u.id == id)
    }
    pub fn wall_at(&self, p: Point) -> bool {
        !p.on_map() || self.terrain.get(((p.y / CELL) * CELLS + p.x / CELL) as usize).copied().unwrap_or(false)
    }
    pub fn enemy_tower_at(&self, p: Point) -> Option<&Unit> {
        self.units.iter().find(|u| u.tower && u.team != self.team && u.hp > 0 && u.pos.within(p, d::AI_TOWER_RANGE))
    }
    /// The nearest open (non-wall, on-map) point to `p`, searching outward in rings.
    pub fn nearest_ground(&self, p: Point) -> Point {
        let p = p.clamp_map();
        if !self.wall_at(p) {
            return p;
        }
        for r in (4_000..=96_000).step_by(4_000) {
            let mut best: Option<Point> = None;
            for k in 0..24 {
                let u = rotate((1.0, 0.0), k as f64 * 15.0);
                let q = p.along(u, r).clamp_map();
                if !self.wall_at(q) && best.is_none_or(|b| q.d2(p) < b.d2(p)) {
                    best = Some(q);
                }
            }
            if let Some(b) = best {
                return b;
            }
        }
        p
    }
    /// Wall cell centres within `range` of `p`, nearest first.
    pub fn walls_near(&self, p: Point, range: i64) -> Vec<Point> {
        let mut out: Vec<Point> = (0..CELLS * CELLS)
            .filter(|i| self.terrain.get(*i as usize).copied().unwrap_or(false))
            .map(|i| Point::new(i % CELLS * CELL + CELL / 2, i / CELLS * CELL + CELL / 2))
            .filter(|c| c.within(p, range))
            .collect();
        out.sort_by_key(|c| (c.d2(p), c.x, c.y));
        out
    }
    /// A body flying from `from` along `u` for `len`: where it stops (before a wall or the map edge), and whether a
    /// wall stopped it.
    pub fn flight(&self, from: Point, u: (f64, f64), len: i64) -> (Point, bool) {
        let mut last = from;
        let steps = (len / 2_000).max(1);
        for i in 1..=steps {
            let q = from.along(u, len * i / steps);
            if !q.on_map() || q.x < 2_000 || q.y < 2_000 || q.x > MAP - 2_000 || q.y > MAP - 2_000 {
                return (last, false);
            }
            if self.wall_at(q) {
                return (last, true);
            }
            last = q;
        }
        (last, false)
    }
    /// Through walls: the first open point at or past `p` along `u` (then back toward `back`), never off the map.
    pub fn land_past(&self, p: Point, u: (f64, f64), back: Point) -> Point {
        let p = p.clamp_map();
        if !self.wall_at(p) {
            return p;
        }
        for k in 1..=24 {
            let q = p.along(u, k * 2_000).clamp_map();
            if !self.wall_at(q) {
                return q;
            }
        }
        let b = unit_dir(p, back);
        for k in 1..=60 {
            let q = p.along(b, k * 2_000).clamp_map();
            if !self.wall_at(q) {
                return q;
            }
        }
        self.nearest_ground(p)
    }
}

// ------------------------------------------------------------------ effects

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case")]
pub enum Effect {
    /// `direct`: a single committed strike on one target (a V1 parry may cancel it and what follows up to `StrikeEnd`)
    Damage { target: usize, amount: i64, direct: bool },
    /// what follows comes from the strike on `target` (dropped by the bridge if that strike was parried)
    Strike { target: usize },
    StrikeEnd,
    KnockUp { target: usize, ticks: u64 },
    Stun { target: usize, ticks: u64 },
    /// the kick flight: `target` is pushed to `to` over `ticks`
    Knock { target: usize, to: Point, ticks: u64 },
    /// Skyfall: `target` is carried to `to` over `ticks`, through walls
    Carry { target: usize, to: Point, ticks: u64 },
    /// Bruised stacks shown on the target (refreshes its window; 0 stacks still Bruised)
    Bruise { target: usize, stacks: i64, ticks: u64 },
    /// the end of a Bruised window
    BruiseEnd { target: usize },
    Mark { target: usize, ticks: u64 },
    /// he's on a wall (true) or back on the ground
    Roof { on: bool },
    /// on a wall with no enemy champion close: hidden for a few ticks
    Hide,
    Shield { amount: i64, ticks: u64 },
    Invulnerable { ticks: u64 },
    /// he moves himself to `to` over `ticks` (an engine glide)
    Glide { to: Point, ticks: u64 },
    /// he passes through walls for `ticks` (hops onto and between walls, the Skyfall dive)
    PassWalls { ticks: u64 },
    /// Flash (the summoner blink) to `to`
    Flash { to: Point },
    /// Second Wind: lethal damage leaves him at 1 HP for `ticks`
    Undying { ticks: u64 },
    UndyingEnd,
    /// back to the tag with this HP (a teleport)
    Return { to: Point, hp: i64 },
    /// the AI walks him toward a point (the bridge turns it into a move order)
    Steer { to: Point },
    Fx { name: String, at: Point, ticks: u64, ambient: bool },
    Pose { name: &'static str, ticks: u64 },
    Log(String),
}

// ------------------------------------------------------------------ state

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MoveKind {
    Mount,
    WallHop,
    GroundHop,
    Drop,
    Dash,
    KickDash,
    Skyfall,
    Fall,
}

/// A move in progress: he's busy until `until`, landing at `land`.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Move {
    pub kind: MoveKind,
    pub until: u64,
    pub land: Point,
    /// the Drop Kick's or Skyfall's target
    pub target: Option<usize>,
}

/// A kick about to land: the strike at `strike_at`, sending `target` along `dir`.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct KickPlan {
    pub target: usize,
    pub dir: (f64, f64),
    pub strike_at: u64,
    pub flashed: bool,
}

/// A kicked body in the air: it knocks up who it passes (up to the cap) and may hit a wall at the end.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Flight {
    pub target: usize,
    pub from: Point,
    pub to: Point,
    pub start: u64,
    pub end: u64,
    pub wall: bool,
    pub hit: Vec<usize>,
    pub attack: i64,
}

/// A rock in the air (non-homing; Attack and endpoint fixed on release).
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Rock {
    pub from: Point,
    pub end: Point,
    pub pos: Point,
    pub attack: i64,
    pub pct: i64,
    pub hit: Vec<usize>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SecondWind {
    pub anchor: Point,
    pub hp: i64,
    pub until: u64,
    pub roof: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct Core {
    pub roof: bool,
    pub mount_ready: u64,
    pub hops: i64,
    pub hop_next: u64,
    pub dashes: i64,
    pub dash_next: u64,
    pub kicks: i64,
    pub kick_next: u64,
    pub rock_ready: u64,
    pub ult_ready: u64,
    /// (unit, until)
    pub marks: Vec<(usize, u64)>,
    /// (unit, until, stacks)
    pub bruised: Vec<(usize, u64, i64)>,
    pub drop_ready: u64,
    /// his next hit deals more until then (the Drop-in bonus)
    pub bonus_until: u64,
    pub shield_ready: u64,
    pub moving: Option<Move>,
    pub kick: Option<KickPlan>,
    pub flights: Vec<Flight>,
    pub rocks: Vec<Rock>,
    pub ult: Option<SecondWind>,
    /// (target, until): a kick just landed on it (the AI may spend the reserve charge to finish it)
    pub last_kicked: Option<(usize, u64)>,
    /// on a wall since (the AI leaves an idle wall)
    pub roof_since: u64,
    /// his HP over the last few ticks (tick, hp), to tell real damage from danger nearby
    pub hp_log: Vec<(u64, i64)>,
    pub rejection: String,
    pub last_skill: &'static str,
}
impl Default for Core {
    fn default() -> Self {
        Core {
            roof: false,
            mount_ready: 0,
            hops: d::HOP_CHARGES,
            hop_next: 0,
            dashes: d::DASH_CHARGES,
            dash_next: 0,
            kicks: d::KICK_CHARGES,
            kick_next: 0,
            rock_ready: 0,
            ult_ready: 0,
            marks: vec![],
            bruised: vec![],
            drop_ready: 0,
            bonus_until: 0,
            shield_ready: 0,
            moving: None,
            kick: None,
            flights: vec![],
            rocks: vec![],
            ult: None,
            last_kicked: None,
            roof_since: 0,
            hp_log: vec![],
            rejection: String::new(),
            last_skill: "",
        }
    }
}

fn travel_ticks(from: Point, to: Point, speed: i64, min: i64) -> u64 {
    ((from.dist(to) as i64) / speed.max(1)).max(min).max(1) as u64
}

impl Core {
    pub fn respawn(&mut self, tick: u64) {
        let keep = (self.mount_ready, self.rock_ready, self.ult_ready, self.drop_ready, self.shield_ready);
        *self = Core::default();
        (self.mount_ready, self.rock_ready, self.ult_ready, self.drop_ready, self.shield_ready) =
            (keep.0.max(tick), keep.1.max(tick), keep.2.max(tick), keep.3, keep.4);
    }
    pub fn busy(&self, tick: u64) -> bool {
        self.moving.as_ref().is_some_and(|m| tick < m.until) || self.kick.is_some()
    }
    pub fn marked(&self, id: usize, tick: u64) -> bool {
        self.marks.iter().any(|m| m.0 == id && tick < m.1)
    }
    pub fn bruise_stacks(&self, id: usize, tick: u64) -> Option<i64> {
        self.bruised.iter().find(|b| b.0 == id && tick < b.1).map(|b| b.2)
    }
    /// Basic attacks: +15% for the first hit after a Drop-in (the bridge scales them and calls `take_bonus`).
    pub fn basic_multiplier(&self, tick: u64) -> i64 {
        if tick < self.bonus_until {
            100 + d::DROP_IN_BONUS_PCT
        } else {
            100
        }
    }
    pub fn take_bonus(&mut self) {
        self.bonus_until = 0;
    }

    pub fn step(&mut self, f: &Frame) -> Vec<Effect> {
        let mut out = vec![];
        self.rejection.clear();
        if !f.alive {
            if self.roof {
                out.push(Effect::Roof { on: false });
            }
            self.moving = None;
            self.kick = None;
            self.flights.clear();
            self.rocks.clear();
            self.roof = false;
            self.ult = None;
            return out;
        }
        self.recharge(f);
        self.hp_log.retain(|h| f.tick < h.0 + t(d::AI_HURT_TICKS));
        self.hp_log.push((f.tick, f.hp));
        self.marks.retain(|m| f.tick < m.1);
        let ended: Vec<usize> = self.bruised.iter().filter(|b| f.tick >= b.1).map(|b| b.0).collect();
        for id in ended {
            out.push(Effect::BruiseEnd { target: id });
        }
        self.bruised.retain(|b| f.tick < b.1);
        if self.last_kicked.is_some_and(|k| f.tick >= k.1) {
            self.last_kicked = None;
        }
        // his accepted basic attacks: marks and Bruised stacks (the bridge scales the Drop-in bonus)
        for &id in &f.basic_hits {
            if let Some(u) = f.unit(id).cloned() {
                if !u.tower && f.skill_target(&u) {
                    self.mark(f, u.id, &mut out);
                    self.bruise_hit(f, u.id, &mut out);
                }
            }
        }
        self.run_second_wind(f, &mut out);
        // a move that ends this tick lands first, then the wall checks see where he really is
        self.run_move(f, &mut out);
        self.run_roof(f, &mut out);
        self.run_kick(f, &mut out);
        self.run_flights(f, &mut out);
        self.run_rocks(f, &mut out);
        let order = if f.manual { f.order } else { self.decide(f, &mut out) };
        if let Some(o) = order {
            self.execute(f, o, &mut out);
        }
        out
    }

    fn recharge(&mut self, f: &Frame) {
        let hop_ticks = t(if self.roof { d::HOP_WALL_RECHARGE_TICKS } else { d::HOP_GROUND_RECHARGE_TICKS });
        for (n, next, max, every) in [
            (&mut self.hops, &mut self.hop_next, d::HOP_CHARGES, hop_ticks),
            (&mut self.dashes, &mut self.dash_next, d::DASH_CHARGES, t(d::DASH_RECHARGE_TICKS)),
            (&mut self.kicks, &mut self.kick_next, d::KICK_CHARGES, t(d::KICK_RECHARGE_TICKS)),
        ] {
            if *n >= max {
                *next = f.tick + every;
                continue;
            }
            if f.tick >= *next {
                *n += 1;
                *next = f.tick + every;
            }
        }
    }
    fn spend(n: &mut i64, next: &mut u64, max: i64, tick: u64, every: u64) {
        if *n >= max {
            *next = tick + every;
        }
        *n -= 1;
    }

    // ---------------------------------------------------------- passives

    fn mark(&mut self, f: &Frame, id: usize, out: &mut Vec<Effect>) {
        self.marks.retain(|m| m.0 != id);
        self.marks.push((id, f.tick + t(d::DASH_MARK_TICKS)));
        out.push(Effect::Mark { target: id, ticks: t(d::DASH_MARK_TICKS) });
    }
    /// Bruised from one of his CCs: refreshed (its stacks kept), only on champions, minions and creeps.
    fn bruise(&mut self, f: &Frame, id: usize, out: &mut Vec<Effect>) {
        let stacks = self.bruise_stacks(id, f.tick).unwrap_or(0);
        self.bruised.retain(|b| b.0 != id);
        self.bruised.push((id, f.tick + t(d::BRUISED_TICKS), stacks));
        out.push(Effect::Bruise { target: id, stacks, ticks: t(d::BRUISED_TICKS) });
    }
    /// One of his hits on a Bruised unit: one more stack (capped), same window.
    fn bruise_hit(&mut self, f: &Frame, id: usize, out: &mut Vec<Effect>) {
        if let Some(b) = self.bruised.iter_mut().find(|b| b.0 == id && f.tick < b.1) {
            if b.2 < d::BRUISED_MAX_STACKS {
                b.2 += 1;
                out.push(Effect::Bruise { target: id, stacks: b.2, ticks: b.1 - f.tick });
            }
        }
    }
    /// A skill hit: damage (with the Drop-in bonus once), a mark and a Bruised stack.
    fn hit(&mut self, f: &Frame, u: &Unit, amount: i64, direct: bool, out: &mut Vec<Effect>) {
        let amount = if f.tick < self.bonus_until {
            self.bonus_until = 0;
            amount * (100 + d::DROP_IN_BONUS_PCT) / 100
        } else {
            amount
        };
        out.push(Effect::Damage { target: u.id, amount, direct });
        self.mark(f, u.id, out);
        self.bruise_hit(f, u.id, out);
    }

    fn land(&mut self, f: &Frame, at: Point, drop: bool, out: &mut Vec<Effect>) {
        if self.roof {
            self.roof = false;
            out.push(Effect::Roof { on: false });
        }
        out.push(Effect::Fx { name: "kk_land".into(), at, ticks: 18, ambient: true });
        if drop && f.tick >= self.drop_ready {
            // Drop-in: a knock-up around the landing and his next hit deals more
            self.drop_ready = f.tick + t(d::DROP_IN_COOLDOWN_TICKS);
            self.bonus_until = f.tick + t(d::DROP_IN_BONUS_TICKS);
            out.push(Effect::Fx { name: "kk_drop_in".into(), at, ticks: 24, ambient: false });
            let hit: Vec<Unit> = f
                .units
                .iter()
                .filter(|u| f.skill_target(u) && u.pos.within(at, d::DROP_IN_KNOCK_RADIUS + u.radius))
                .cloned()
                .collect();
            for u in hit {
                out.push(Effect::KnockUp { target: u.id, ticks: t(d::DROP_IN_KNOCK_TICKS) });
                self.bruise(f, u.id, out);
            }
        }
        if f.tick >= self.shield_ready {
            self.shield_ready = f.tick + t(d::ROOF_GROUND_SHIELD_COOLDOWN_TICKS);
            out.push(Effect::Shield { amount: f.max_hp * d::ROOF_GROUND_SHIELD_PCT_MAX_HP / 100, ticks: t(d::ROOF_GROUND_SHIELD_TICKS) });
        }
    }

    fn run_roof(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        let moving = self.busy(f.tick);
        if self.roof {
            if f.stunned && !self.moving.as_ref().is_some_and(|m| m.kind == MoveKind::Fall) {
                // knocked off: down to the nearest open ground, no Drop-in
                let to = f.nearest_ground(f.pos);
                let ticks = travel_ticks(f.pos, to, d::HOP_SPEED, 6);
                out.push(Effect::PassWalls { ticks: ticks + 2 });
                out.push(Effect::Glide { to, ticks });
                self.moving = Some(Move { kind: MoveKind::Fall, until: f.tick + ticks, land: to, target: None });
                self.kick = None;
                out.push(Effect::Log("knocked off the wall".into()));
                return;
            }
            // Roof is an accepted stance, not inferred from one coarse terrain cell.
            // Walking across an art ledge or a host path correction must not drop him.
            // Only hard control, a deliberate Drop-in technique, death or the
            // recorded Second Wind return can change this state.
            if !f.enemy_champions().any(|u| u.pos.within(f.pos, d::ROOF_HIDE_RANGE)) {
                out.push(Effect::Hide);
            }
        } else if !moving && f.wall_at(f.pos) {
            // never left inside terrain
            let to = f.nearest_ground(f.pos);
            out.push(Effect::PassWalls { ticks: 8 });
            out.push(Effect::Glide { to, ticks: 6 });
            self.moving = Some(Move { kind: MoveKind::Fall, until: f.tick + 6, land: to, target: None });
        }
    }

    fn run_move(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        let Some(m) = self.moving.clone() else { return };
        if f.tick < m.until {
            return;
        }
        self.moving = None;
        match m.kind {
            MoveKind::Mount | MoveKind::WallHop => {
                if !self.roof {
                    self.roof = true;
                    self.roof_since = f.tick;
                    out.push(Effect::Roof { on: true });
                }
                out.push(Effect::Fx { name: "kk_land".into(), at: m.land, ticks: 12, ambient: true });
            }
            MoveKind::GroundHop | MoveKind::Dash | MoveKind::KickDash => {}
            MoveKind::Fall => {
                if self.roof {
                    self.roof = false;
                    out.push(Effect::Roof { on: false });
                }
            }
            MoveKind::Drop => {
                self.land(f, m.land, true, out);
                if let Some(u) = m.target.and_then(|id| f.unit(id)).cloned() {
                    if f.skill_target(&u) && u.pos.within(m.land, d::HOP_DROP_KICK_RADIUS + u.radius) {
                        out.push(Effect::Strike { target: u.id });
                        self.hit(f, &u, pct(d::HOP_DROP_BASE, d::HOP_DROP_AD_RATIO, f.attack), true, out);
                        out.push(Effect::StrikeEnd);
                    }
                }
            }
            MoveKind::Skyfall => {
                self.land(f, m.land, true, out);
            }
        }
    }

    // ---------------------------------------------------------- S3

    /// Where he must stand to kick `u` along `dir`: just behind it.
    pub fn kick_spot(u: &Unit, dir: (f64, f64)) -> Point {
        u.pos.along(dir, -(u.radius + d::KICK_REACH))
    }

    fn run_kick(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        let Some(k) = self.kick.clone() else { return };
        if self.moving.as_ref().is_some_and(|m| f.tick < m.until) || f.tick < k.strike_at {
            return;
        }
        self.kick = None;
        let Some(u) = f.unit(k.target).cloned() else { return };
        if !f.skill_target(&u) || !u.champion || !u.pos.within(f.pos, u.radius + d::KICK_REACH + 8_000) {
            out.push(Effect::Log("the kick missed (out of reach)".into()));
            return;
        }
        let (to, wall) = f.flight(u.pos, k.dir, d::KICK_DISTANCE);
        out.push(Effect::Strike { target: u.id });
        self.hit(f, &u, pct(d::KICK_BASE, d::KICK_AD_RATIO, f.attack), true, out);
        let ticks = t(d::KICK_FLIGHT_TICKS);
        out.push(Effect::Knock { target: u.id, to, ticks });
        self.bruise(f, u.id, out);
        out.push(Effect::Fx { name: "kk_kick".into(), at: u.pos, ticks: 18, ambient: false });
        out.push(Effect::StrikeEnd);
        self.flights.push(Flight { target: u.id, from: u.pos, to, start: f.tick, end: f.tick + ticks, wall, hit: vec![], attack: f.attack });
        self.last_kicked = Some((u.id, f.tick + 180));
        out.push(Effect::Log(format!("kick {} -> ({}, {}){}", u.id, to.x, to.y, if wall { " into a wall" } else { "" })));
    }

    fn run_flights(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        let mut flights = std::mem::take(&mut self.flights);
        for fl in flights.iter_mut() {
            let span = (fl.end - fl.start).max(1) as f64;
            let k = ((f.tick - fl.start) as f64 / span).clamp(0.0, 1.0);
            let at = Point::new(fl.from.x + ((fl.to.x - fl.from.x) as f64 * k) as i64, fl.from.y + ((fl.to.y - fl.from.y) as f64 * k) as i64);
            // the body knocks up up to two other enemy champions it passes
            if (fl.hit.len() as i64) < d::KICK_SECONDARY_MAX {
                let mut near: Vec<Unit> = f
                    .enemy_champions()
                    .filter(|o| o.id != fl.target && !fl.hit.contains(&o.id))
                    .filter(|o| o.pos.seg(fl.from, at).0 <= (d::KICK_SECONDARY_RADIUS + o.radius) as f64)
                    .cloned()
                    .collect();
                near.sort_by_key(|o| (o.pos.d2(fl.from), o.id));
                for o in near.into_iter().take((d::KICK_SECONDARY_MAX - fl.hit.len() as i64) as usize) {
                    fl.hit.push(o.id);
                    let amount = pct(d::KICK_BASE, d::KICK_AD_RATIO, fl.attack) * d::KICK_SECONDARY_PCT / 100;
                    self.hit(f, &o, amount, false, out);
                    out.push(Effect::KnockUp { target: o.id, ticks: t(d::KICK_SECONDARY_KNOCK_TICKS) });
                    self.bruise(f, o.id, out);
                }
            }
            if f.tick >= fl.end && fl.wall {
                out.push(Effect::Stun { target: fl.target, ticks: t(d::KICK_WALL_STUN_TICKS) });
                self.bruise(f, fl.target, out);
                out.push(Effect::Fx { name: "kk_wall_slam".into(), at: fl.to, ticks: 18, ambient: false });
            }
        }
        flights.retain(|fl| f.tick < fl.end);
        self.flights = flights;
    }

    // ---------------------------------------------------------- S2

    fn run_rocks(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        let mut rocks = std::mem::take(&mut self.rocks);
        for r in rocks.iter_mut() {
            let next = if r.pos.within(r.end, d::ROCK_SPEED) { r.end } else { r.pos.toward(r.end, d::ROCK_SPEED) };
            let hit: Vec<Unit> = f
                .units
                .iter()
                .filter(|u| f.skill_target(u) && !r.hit.contains(&u.id) && u.pos.seg(r.pos, next).0 <= (d::ROCK_WIDTH + u.radius) as f64)
                .cloned()
                .collect();
            for u in hit {
                r.hit.push(u.id);
                let amount = pct(d::ROCK_BASE, d::ROCK_AD_RATIO, r.attack) * r.pct / 100;
                self.hit(f, &u, amount, false, out);
                out.push(Effect::Fx { name: "kk_rock_hit".into(), at: u.pos, ticks: 12, ambient: true });
            }
            r.pos = next;
        }
        rocks.retain(|r| r.pos != r.end);
        self.rocks = rocks;
    }

    // ---------------------------------------------------------- ult

    fn run_second_wind(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        let Some(w) = self.ult.clone() else { return };
        let died = f.hp <= 1;
        if !died && f.tick < w.until {
            return;
        }
        self.ult = None;
        let hp = if died { f.max_hp } else { w.hp.clamp(1, f.max_hp) };
        out.push(Effect::UndyingEnd);
        out.push(Effect::Return { to: w.anchor, hp });
        out.push(Effect::Fx { name: "kk_return".into(), at: w.anchor, ticks: 24, ambient: false });
        self.moving = None;
        self.kick = None;
        if self.roof != w.roof {
            self.roof = w.roof;
            out.push(Effect::Roof { on: w.roof });
        }
        out.push(Effect::Log(if died { "Second Wind: back to the tag at full HP".into() } else { "Second Wind: time's up, back to the tag".into() }));
    }

    // ---------------------------------------------------------- orders

    fn reject(&mut self, why: &str) {
        self.rejection = why.to_string();
    }

    pub fn execute(&mut self, f: &Frame, o: Order, out: &mut Vec<Effect>) {
        if f.stunned {
            return self.reject("stunned");
        }
        // Flash may re-aim a kick in its wind-up; the tag is instant (no movement), so it may go down mid-leap
        if self.busy(f.tick) && !matches!(o, Order::FlashKick(..) | Order::Ult) {
            return self.reject("busy");
        }
        match o {
            Order::Mount => {
                if self.roof {
                    return self.reject("already on a wall");
                }
                if f.tick < self.mount_ready {
                    return self.reject("mount cooling down");
                }
                let Some(cell) = f.walls_near(f.pos, d::HOP_MOUNT_RANGE).into_iter().next() else {
                    return self.reject("no wall in reach");
                };
                self.mount_ready = f.tick + t(d::HOP_MOUNT_COOLDOWN_TICKS);
                self.leap(f, cell, MoveKind::Mount, None, out);
                self.last_skill = "mount";
            }
            Order::Hop(p) => {
                if self.hops <= 0 {
                    return self.reject("no hop charge");
                }
                let every = t(if self.roof { d::HOP_WALL_RECHARGE_TICKS } else { d::HOP_GROUND_RECHARGE_TICKS });
                if self.roof {
                    let Some(cell) = f.walls_near(p, CELL).into_iter().next().filter(|c| c.within(f.pos, d::HOP_WALL_RANGE)) else {
                        return self.reject("no wall there in reach");
                    };
                    Self::spend(&mut self.hops, &mut self.hop_next, d::HOP_CHARGES, f.tick, every);
                    self.leap(f, cell, MoveKind::WallHop, None, out);
                } else {
                    let to = f.nearest_ground(f.pos.toward(p, d::HOP_GROUND_HOP));
                    Self::spend(&mut self.hops, &mut self.hop_next, d::HOP_CHARGES, f.tick, every);
                    self.leap(f, to, MoveKind::GroundHop, None, out);
                }
                self.last_skill = "hop";
            }
            Order::DropKick(p) => {
                if !self.roof {
                    return self.reject("not on a wall");
                }
                let aim = if p.within(f.pos, d::HOP_DROP_RANGE) { p } else { f.pos.toward(p, d::HOP_DROP_RANGE) };
                let target = f
                    .units
                    .iter()
                    .filter(|u| f.skill_target(u) && u.pos.within(aim, d::HOP_DROP_KICK_RADIUS + u.radius))
                    .min_by_key(|u| (!u.champion, u.pos.d2(aim), u.id))
                    .map(|u| u.id);
                let land = f.nearest_ground(aim);
                // invulnerable while dropping (Drop-in), whether or not the knock-up is ready
                out.push(Effect::Invulnerable { ticks: t(d::DROP_IN_INVULN_TICKS) });
                self.leap(f, land, MoveKind::Drop, target, out);
                self.last_skill = "drop_kick";
            }
            Order::Rock(p) => {
                if f.tick < self.rock_ready {
                    return self.reject("rock cooling down");
                }
                self.rock_ready = f.tick + t(d::ROCK_COOLDOWN_TICKS);
                let u = unit_dir(f.pos, p);
                if self.roof {
                    let n = d::ROCK_RAIN_COUNT.max(1);
                    for i in 0..n {
                        let deg = (i - (n - 1) / 2) as f64 * d::ROCK_RAIN_SPREAD_DEG as f64;
                        let end = f.pos.along(rotate(u, deg), d::ROCK_RAIN_RANGE);
                        self.rocks.push(Rock { from: f.pos, end, pos: f.pos, attack: f.attack, pct: d::ROCK_RAIN_PCT, hit: vec![] });
                    }
                } else {
                    let end = f.pos.along(u, d::ROCK_RANGE);
                    self.rocks.push(Rock { from: f.pos, end, pos: f.pos, attack: f.attack, pct: 100, hit: vec![] });
                }
                out.push(Effect::Fx { name: "kk_rock".into(), at: f.pos, ticks: 12, ambient: false });
                out.push(Effect::Pose { name: "throw", ticks: 12 });
                self.last_skill = "rock";
            }
            Order::Dash(id) => {
                if self.dashes <= 0 {
                    return self.reject("no dash charge");
                }
                let Some(u) = f.unit(id).cloned() else { return self.reject("no such unit") };
                if !self.marked(id, f.tick) || !f.enemy(&u) || u.tower {
                    return self.reject("not marked");
                }
                if !u.pos.within(f.pos, d::DASH_RANGE) {
                    return self.reject("out of dash range");
                }
                let stop = f.nearest_ground(u.pos.toward(f.pos, u.radius + d::DASH_GAP));
                Self::spend(&mut self.dashes, &mut self.dash_next, d::DASH_CHARGES, f.tick, t(d::DASH_RECHARGE_TICKS));
                let ticks = travel_ticks(f.pos, stop, d::DASH_SPEED, 6);
                if self.roof {
                    out.push(Effect::PassWalls { ticks: ticks + 2 });
                }
                out.push(Effect::Glide { to: stop, ticks });
                out.push(Effect::Shield { amount: f.max_hp * d::DASH_SHIELD_PCT_MAX_HP / 100, ticks: t(d::DASH_SHIELD_TICKS) });
                out.push(Effect::Fx { name: "kk_dash".into(), at: f.pos, ticks: 12, ambient: true });
                self.moving = Some(Move { kind: MoveKind::Dash, until: f.tick + ticks, land: stop, target: Some(id) });
                if self.roof {
                    // a dash off a wall is a jump down
                    self.moving = Some(Move { kind: MoveKind::Drop, until: f.tick + ticks, land: stop, target: None });
                }
                self.last_skill = "dash";
            }
            Order::Kick(id, toward) => {
                if self.roof {
                    return self.execute(f, Order::Skyfall(toward), out);
                }
                if self.kicks <= 0 {
                    return self.reject("no kick charge");
                }
                let Some(u) = f.unit(id).cloned().filter(|u| u.champion && f.skill_target(u)) else {
                    return self.reject("not an enemy champion");
                };
                let dir = unit_dir(u.pos, toward);
                let spot = f.nearest_ground(Self::kick_spot(&u, dir));
                if !spot.within(f.pos, d::KICK_DASH_RANGE + d::KICK_REACH) {
                    return self.reject("kick spot out of reach");
                }
                Self::spend(&mut self.kicks, &mut self.kick_next, d::KICK_CHARGES, f.tick, t(d::KICK_RECHARGE_TICKS));
                let ticks = travel_ticks(f.pos, spot, d::DASH_SPEED, 4);
                if !spot.within(f.pos, 1_000) {
                    out.push(Effect::Glide { to: spot, ticks });
                    self.moving = Some(Move { kind: MoveKind::KickDash, until: f.tick + ticks, land: spot, target: Some(id) });
                }
                let strike_at = f.tick + if spot.within(f.pos, 1_000) { 0 } else { ticks } + t(d::KICK_WINDUP_TICKS);
                self.kick = Some(KickPlan { target: id, dir, strike_at, flashed: false });
                out.push(Effect::Pose { name: "kick", ticks: t(d::KICK_WINDUP_TICKS) + 6 });
                self.last_skill = "kick";
            }
            Order::FlashKick(id, toward) => {
                if !f.flash_ready {
                    return self.reject("Flash not ready");
                }
                let Some(u) = f.unit(id).cloned().filter(|u| u.champion && f.skill_target(u)) else {
                    return self.reject("not an enemy champion");
                };
                let dir = unit_dir(u.pos, toward);
                let spot = f.nearest_ground(Self::kick_spot(&u, dir));
                if !spot.within(f.pos, d::KICK_FLASH_RANGE) {
                    return self.reject("kick spot out of Flash range");
                }
                match &mut self.kick {
                    // during the wind-up: Flash re-aims the kick from the new spot (no extra charge)
                    Some(k) if k.target == id => {
                        k.dir = dir;
                        k.flashed = true;
                    }
                    _ => {
                        if self.busy(f.tick) {
                            return self.reject("busy");
                        }
                        if self.roof {
                            return self.reject("on a wall");
                        }
                        if self.kicks <= 0 {
                            return self.reject("no kick charge");
                        }
                        Self::spend(&mut self.kicks, &mut self.kick_next, d::KICK_CHARGES, f.tick, t(d::KICK_RECHARGE_TICKS));
                        self.kick = Some(KickPlan { target: id, dir, strike_at: f.tick + t(d::KICK_WINDUP_TICKS), flashed: true });
                        out.push(Effect::Pose { name: "kick", ticks: t(d::KICK_WINDUP_TICKS) + 6 });
                    }
                }
                self.moving = None;
                out.push(Effect::Flash { to: spot });
                self.last_skill = "flash_kick";
            }
            Order::Skyfall(p) => {
                if !self.roof {
                    return self.reject("not on a wall");
                }
                if self.kicks <= 0 {
                    return self.reject("no kick charge");
                }
                let u = unit_dir(f.pos, p);
                let end = f.pos.along(u, d::SKYFALL_RANGE);
                let target = Self::skyfall_target(f, f.pos, end);
                Self::spend(&mut self.kicks, &mut self.kick_next, d::KICK_CHARGES, f.tick, t(d::KICK_RECHARGE_TICKS));
                let ticks = t(d::SKYFALL_TICKS);
                let target_end = f.land_past(end, u, f.pos);
                let mine = if target.is_some() { f.land_past(target_end.along(u, d::SKYFALL_BEHIND), u, target_end) } else { target_end };
                out.push(Effect::PassWalls { ticks: ticks + 2 });
                out.push(Effect::Invulnerable { ticks: t(d::DROP_IN_INVULN_TICKS) });
                out.push(Effect::Glide { to: mine, ticks });
                out.push(Effect::Fx { name: "kk_skyfall".into(), at: f.pos, ticks: 24, ambient: false });
                if let Some(v) = target.and_then(|id| f.unit(id)).cloned() {
                    out.push(Effect::Strike { target: v.id });
                    self.hit(f, &v, pct(d::SKYFALL_BASE, d::SKYFALL_AD_RATIO, f.attack), true, out);
                    out.push(Effect::Carry { target: v.id, to: target_end, ticks });
                    self.bruise(f, v.id, out);
                    out.push(Effect::StrikeEnd);
                    self.last_kicked = Some((v.id, f.tick + ticks + 180));
                }
                self.moving = Some(Move { kind: MoveKind::Skyfall, until: f.tick + ticks, land: mine, target });
                self.last_skill = "skyfall";
            }
            Order::Ult => {
                if !f.ult_unlocked {
                    return self.reject("ult not learned");
                }
                if f.tick < self.ult_ready {
                    return self.reject("ult cooling down");
                }
                self.ult_ready = f.tick + t(d::ULT_COOLDOWN_TICKS);
                self.ult = Some(SecondWind { anchor: f.pos, hp: f.hp, until: f.tick + t(d::ULT_TICKS), roof: self.roof });
                out.push(Effect::Undying { ticks: t(d::ULT_TICKS) + 2 });
                out.push(Effect::Fx { name: "kk_tag".into(), at: f.pos, ticks: t(d::ULT_TICKS), ambient: false });
                self.last_skill = "second_wind";
            }
        }
    }

    /// The first enemy champion along a Skyfall line.
    pub fn skyfall_target(f: &Frame, from: Point, end: Point) -> Option<usize> {
        f.enemy_champions()
            .filter(|u| u.pos.seg(from, end).0 <= (d::SKYFALL_WIDTH + u.radius) as f64)
            .min_by(|a, b| a.pos.seg(from, end).1.total_cmp(&b.pos.seg(from, end).1).then(a.id.cmp(&b.id)))
            .map(|u| u.id)
    }

    /// A leap (an arc: a glide under a jump pose), through walls when it starts or ends on one.
    fn leap(&mut self, f: &Frame, to: Point, kind: MoveKind, target: Option<usize>, out: &mut Vec<Effect>) {
        let ticks = travel_ticks(f.pos, to, d::HOP_SPEED, d::HOP_MIN_TICKS);
        out.push(Effect::PassWalls { ticks: ticks + 2 });
        out.push(Effect::Glide { to, ticks });
        out.push(Effect::Pose { name: if f.pos.within(to, 30_000) { "hop_low" } else { "hop_high" }, ticks });
        out.push(Effect::Fx { name: "kk_hop".into(), at: f.pos, ticks: 12, ambient: true });
        self.moving = Some(Move { kind, until: f.tick + ticks, land: to, target });
        self.kick = None;
    }

    // ---------------------------------------------------------- AI

    pub fn hp_pct(f: &Frame) -> i64 {
        f.hp * 100 / f.max_hp.max(1)
    }
    /// Real damage taken: more than `AI_HURT_PCT` of his max HP in the last half second.
    pub fn hurt(&self, f: &Frame) -> bool {
        let top = self.hp_log.iter().map(|h| h.1).max().unwrap_or(f.hp);
        (top - f.hp) * 100 >= d::AI_HURT_PCT * f.max_hp.max(1)
    }
    /// Enemy champions near a point (towers count double).
    pub fn danger_at(f: &Frame, p: Point, skip: Option<usize>) -> i64 {
        let champs = f.enemy_champions().filter(|u| Some(u.id) != skip && u.pos.within(p, 60_000)).count() as i64;
        champs + if f.enemy_tower_at(p).is_some() { 2 } else { 0 }
    }
    pub fn isolated(f: &Frame, u: &Unit) -> bool {
        !f.enemy_champions().any(|o| o.id != u.id && o.pos.within(u.pos, 50_000))
    }
    /// Would he start a fight with `u`? Ego leads; his HP, the target's HP, help on both sides and its tower weigh.
    pub fn would_start(&self, f: &Frame, u: &Unit) -> bool {
        let ego = f.p(EGO);
        let target_pct = u.hp * 100 / u.max_hp.max(1);
        let allies = f.allies().filter(|a| a.pos.within(u.pos, d::AI_ENGAGE_RANGE)).count() as i64;
        let enemies = f.enemy_champions().filter(|o| o.id != u.id && o.pos.within(u.pos, 60_000)).count() as i64;
        let mut score = ego - 50 + (100 - target_pct) / 2 + (Self::hp_pct(f) - 50) / 2 + allies * 15 - enemies * 20;
        if Self::isolated(f, u) {
            score += 15;
        }
        if f.enemy_tower_at(u.pos).is_some() && !(self.kicks >= 1 && f.p(AGGRESSIVE) >= 40) {
            score -= 40;
        }
        // Skyfall made this 1v1: he takes the fight he created
        if self.last_kicked.is_some_and(|k| k.0 == u.id) && Self::isolated(f, u) {
            score += 30;
        }
        score >= 0
    }
    pub fn hunt(&self, f: &Frame, range: i64) -> Option<Unit> {
        f.enemy_champions()
            .filter(|u| u.pos.within(f.pos, range) && self.would_start(f, u))
            .min_by_key(|u| (u.hp * 100 / u.max_hp.max(1), u.pos.d2(f.pos), u.id))
            .cloned()
    }

    /// Aim error from the athlete's skill-hit value: up to about 12 degrees, deterministic per tick.
    pub fn aim_error(f: &Frame) -> f64 {
        let spread = (100 - f.p(SKILL_HIT)) as f64 * 0.12;
        let h = (f.tick.wrapping_mul(2_654_435_761) ^ f.owner as u64) % 1_000;
        (h as f64 / 999.0 * 2.0 - 1.0) * spread
    }

    /// Kick directions for `u`, best first: (score, direction, where it lands). Directions that land it in his danger,
    /// under an enemy tower or among its team are left out. `peel`: an ally it was on (kick it away from them).
    pub fn kick_options(&self, f: &Frame, u: &Unit, peel: Option<Point>) -> Vec<(i64, (f64, f64), Point)> {
        let mut cands: Vec<(i64, (f64, f64))> = vec![];
        // peeling: never toward a teammate (the one it was on is the nearest)
        if let Some(a) = f.allies().filter(|_| peel.is_none()).filter(|a| a.pos.within(u.pos, d::AI_ALLY_RANGE + d::KICK_DISTANCE)).min_by_key(|a| (a.pos.d2(u.pos), a.id)) {
            cands.push((100, unit_dir(u.pos, a.pos)));
        }
        if let Some(p) = peel {
            cands.push((90, unit_dir(p, u.pos)));
        }
        if let Some(tw) = f.units.iter().find(|w| w.tower && w.team != f.team && w.hp > 0 && w.pos.within(u.pos, d::AI_TOWER_RANGE)) {
            cands.push((80, unit_dir(tw.pos, u.pos)));
        }
        cands.push((60, unit_dir(u.pos, f.home)));
        for k in 0..16 {
            let dir = rotate((1.0, 0.0), k as f64 * 22.5);
            if f.flight(u.pos, dir, d::KICK_DISTANCE).1 {
                cands.push((50, dir));
            }
        }
        let others: Vec<&Unit> = f.enemy_champions().filter(|o| o.id != u.id).collect();
        let mut out = vec![];
        for (score, dir) in cands {
            let (land, _) = f.flight(u.pos, dir, d::KICK_DISTANCE);
            if f.enemy_tower_at(land).is_some() {
                continue;
            }
            let near_team = others.iter().filter(|o| o.pos.within(land, 40_000)).count();
            let closer = others.iter().any(|o| o.pos.within(land, 30_000) && o.pos.d2(land) < o.pos.d2(u.pos));
            if near_team >= 2 || closer {
                continue;
            }
            out.push((score, dir, land));
        }
        out.sort_by(|a, b| b.0.cmp(&a.0));
        out
    }

    /// May he spend a kick charge on `u`? One is held for picks: the second only finishes a target he just kicked,
    /// or peels.
    fn kick_charge_ok(&self, u: &Unit, peel: bool) -> bool {
        self.kicks >= 2 || (self.kicks >= 1 && (peel || self.last_kicked.is_some_and(|k| k.0 == u.id)))
    }

    /// A kick order for `u` if a safe direction exists: plain (the dash reaches the spot) or with Flash.
    pub fn plan_kick(&self, f: &Frame, u: &Unit, peel: Option<Point>) -> Option<Order> {
        // never at a body still flying from his last kick: it would be gone before the strike
        if !self.kick_charge_ok(u, peel.is_some()) || u.parry || self.flights.iter().any(|fl| fl.target == u.id) {
            return None;
        }
        let err = Self::aim_error(f);
        let options = self.kick_options(f, u, peel);
        for &(_, dir, _) in &options {
            let dir = rotate(dir, err);
            let spot = f.nearest_ground(Self::kick_spot(u, dir));
            if spot.within(f.pos, d::KICK_DASH_RANGE + d::KICK_REACH) {
                return Some(Order::Kick(u.id, u.pos.along(dir, d::KICK_DISTANCE)));
            }
        }
        if f.flash_ready {
            for &(_, dir, _) in &options {
                let spot = f.nearest_ground(Self::kick_spot(u, dir));
                if spot.within(f.pos, d::KICK_FLASH_RANGE) {
                    return Some(Order::FlashKick(u.id, u.pos.along(dir, d::KICK_DISTANCE)));
                }
            }
        }
        None
    }

    /// A Skyfall at `u`: only if it's the first champion in the line and where it lands is safe.
    pub fn plan_skyfall(&self, f: &Frame, u: &Unit) -> Option<Order> {
        if !self.kick_charge_ok(u, false) || u.parry || !u.pos.within(f.pos, d::SKYFALL_RANGE) {
            return None;
        }
        let dir = unit_dir(f.pos, u.pos);
        let end = f.pos.along(dir, d::SKYFALL_RANGE);
        if Self::skyfall_target(f, f.pos, end) != Some(u.id) {
            return None;
        }
        let land = f.land_past(end, dir, f.pos);
        if f.enemy_tower_at(land).is_some() || Self::danger_at(f, land, Some(u.id)) >= 2 {
            return None;
        }
        Some(Order::Skyfall(u.pos))
    }

    /// The ally his kicks should protect: one with an enemy champion on it and hurt.
    fn peel_target(f: &Frame) -> Option<(Unit, Unit)> {
        f.allies()
            .filter(|a| a.hp * 100 < a.max_hp.max(1) * 70)
            .filter_map(|a| {
                f.enemy_champions()
                    .filter(|e| e.pos.within(a.pos, d::AI_PEEL_RANGE + e.radius + a.radius) && e.pos.within(f.pos, d::DASH_RANGE))
                    .min_by_key(|e| (e.pos.d2(a.pos), e.id))
                    .map(|e| (a.clone(), e.clone()))
            })
            .min_by_key(|(a, _)| (a.hp * 100 / a.max_hp.max(1), a.id))
    }

    pub fn decide(&mut self, f: &Frame, out: &mut Vec<Effect>) -> Option<Order> {
        if f.stunned || self.busy(f.tick) {
            return None;
        }
        let hurt = self.hurt(f);
        let here = Self::danger_at(f, f.pos, None);
        // 1. escape only after real damage (or a tower on him): a marked minion or creep away from danger, or a wall
        if (hurt && here >= 2 && Self::hp_pct(f) < 45) || (hurt && f.enemy_tower_at(f.pos).is_some()) {
            if self.dashes > 0 {
                if let Some(u) = f
                    .units
                    .iter()
                    .filter(|u| !u.champion && !u.tower && self.marked(u.id, f.tick) && u.pos.within(f.pos, d::DASH_RANGE))
                    .filter(|u| Self::danger_at(f, u.pos, None) < here)
                    .max_by_key(|u| (u.pos.d2(f.pos), u.id))
                {
                    return Some(Order::Dash(u.id));
                }
            }
            if !self.roof && f.tick >= self.mount_ready && !f.walls_near(f.pos, d::HOP_MOUNT_RANGE).is_empty() {
                return Some(Order::Mount);
            }
            if self.roof && self.hops > 0 {
                if let Some(c) = f.walls_near(f.pos, d::HOP_WALL_RANGE).into_iter().filter(|c| Self::danger_at(f, *c, None) < here).last() {
                    return Some(Order::Hop(c));
                }
            }
        }
        let target = self.hunt(f, if self.roof { d::AI_GANK_RANGE } else { d::AI_ENGAGE_RANGE });
        // 2. Second Wind before a dive or a teamfight, with the tag somewhere safe
        if f.ult_unlocked && f.tick >= self.ult_ready && self.ult.is_none() {
            if let Some(u) = &target {
                let fight = f.enemy_champions().filter(|o| o.pos.within(u.pos, d::AI_ENGAGE_RANGE)).count() >= 2;
                let dive = f.enemy_tower_at(u.pos).is_some();
                let safe = !f.enemy_champions().any(|o| o.pos.within(f.pos, d::ULT_ANCHOR_SAFE_RANGE));
                if (fight || dive) && safe && u.pos.within(f.pos, d::AI_ENGAGE_RANGE) {
                    return Some(Order::Ult);
                }
            }
        }
        // 3. peel: an enemy on a hurt teammate is kicked away from them
        if let Some((ally, diver)) = Self::peel_target(f) {
            if !self.roof {
                if let Some(o) = self.plan_kick(f, &diver, Some(ally.pos)) {
                    return Some(o);
                }
                if self.dashes > 0 && self.marked(diver.id, f.tick) && !diver.pos.within(f.pos, 25_000) {
                    return Some(Order::Dash(diver.id));
                }
            }
        }
        if let Some(u) = target {
            if self.roof {
                // the rooftop gank: mark it, carry it home, or drop on it
                if f.tick >= self.rock_ready && !self.marked(u.id, f.tick) && u.pos.within(f.pos, d::ROCK_RAIN_RANGE) {
                    return Some(Order::Rock(u.pos));
                }
                if let Some(o) = self.plan_skyfall(f, &u) {
                    return Some(o);
                }
                if u.pos.within(f.pos, d::HOP_DROP_RANGE) && f.enemy_tower_at(u.pos).is_none() {
                    return Some(Order::DropKick(u.pos));
                }
                if self.hops > 0 {
                    if let Some(c) = f.walls_near(u.pos, d::HOP_WALL_RANGE).into_iter().find(|c| c.within(f.pos, d::HOP_WALL_RANGE) && c.d2(u.pos) < f.pos.d2(u.pos)) {
                        return Some(Order::Hop(c));
                    }
                }
                return None;
            }
            if let Some(o) = self.plan_kick(f, &u, None) {
                return Some(o);
            }
            if f.tick >= self.rock_ready && u.pos.within(f.pos, d::ROCK_RANGE) && !self.marked(u.id, f.tick) {
                return Some(Order::Rock(u.pos));
            }
            if self.dashes > 0 && self.marked(u.id, f.tick) && !u.pos.within(f.pos, 30_000) && u.pos.within(f.pos, d::DASH_RANGE) {
                let tower = f.enemy_tower_at(u.pos).is_some();
                if !tower || self.kicks >= 1 {
                    return Some(Order::Dash(u.id));
                }
            }
            out.push(Effect::Steer { to: u.pos });
            return None;
        }
        if self.roof {
            // an idle wall: back down to the lane after a while
            if f.tick >= self.roof_since + t(d::AI_IDLE_ROOF_TICKS) && !f.enemy_champions().any(|u| u.pos.within(f.pos, d::AI_GANK_RANGE)) {
                let ground = f.nearest_ground(f.pos.toward(f.home, 20_000));
                return Some(Order::DropKick(ground));
            }
            return None;
        }
        // 4. set up a rooftop gank: enemies around, none to start on yet, a wall close by
        let enemies_around = f.enemy_champions().any(|u| u.pos.within(f.pos, d::AI_GANK_RANGE));
        if enemies_around && f.tick >= self.mount_ready && f.p(ROAMING) >= 40 && !f.walls_near(f.pos, d::HOP_MOUNT_RANGE).is_empty() && Self::hp_pct(f) >= 50 {
            return Some(Order::Mount);
        }
        // 5. farming: Rock Throw through the wave (S1 and S2 only; S3 and the ult never on minions)
        let calm = !f.enemy_champions().any(|u| u.pos.within(f.pos, d::AI_CALM_RANGE));
        if calm && f.tick >= self.rock_ready {
            let farm: Vec<&Unit> = f.units.iter().filter(|u| u.farmable && f.skill_target(u) && u.pos.within(f.pos, d::ROCK_RANGE)).collect();
            if let Some(best) = farm
                .iter()
                .map(|u| {
                    let end = f.pos.along(unit_dir(f.pos, u.pos), d::ROCK_RANGE);
                    let n = farm.iter().filter(|o| o.pos.seg(f.pos, end).0 <= (d::ROCK_WIDTH + o.radius) as f64).count() as i64;
                    (n, u.pos)
                })
                .max_by_key(|(n, p)| (*n, -(p.d2(f.pos) as i64 / 1_000_000)))
            {
                if best.0 >= d::AI_FARM_MIN_HITS {
                    return Some(Order::Rock(best.1));
                }
            }
        }
        None
    }
}
