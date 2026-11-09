//! Wright, the aircraft engineer: a deterministic core (snapshot in, effects out). `wright.rs` builds the frame from the
//! sim and applies the effects; the Lab (`native/wright_lab`) runs this same file.
//!
//!   passive Workshop: an airframe every 4 s (up to 3); S1-S3 each use one. Every 14 s a scout drone (a real unit) flies
//!           to the nearest enemy champion, hovers and taunts enemies near it onto itself once.
//!   S1      Fighter Wing: 2 fighters (real, targetable units) for 20 s, at most 4. They fly sorties from him: commit to
//!           where an enemy is (champions first, then minions and creeps), strike once (every 3rd sortie a rocket) and
//!           fly back; no homing. A kamikaze (knockback, burn) when their time ends or they are nearly shot down.
//!   S2      Carpet Bomber: three large, overlapping explosions advance along a committed line; the last stuns.
//!   S3      Repair Wing: a repair chopper circles him and a repair plane flies to the most hurt ally anywhere; both drop
//!           kits (heal + capped shield) for 6 s, and the chopper repairs his fighters.
//!   Ult     Sky Carrier: drops on Wright (no damage; knockback stronger the closer), then follows him for 10 s,
//!           guarding him, launching fighters/repair choppers and stocking ready bombers.
//!
//! Planes are simulated here; fighters and the drone also have a host carrier unit (their position and HP come back in
//! the frame, and a carrier that dies means the plane was shot down).
use crate::wright_data as d;
use serde::{Deserialize, Serialize};

pub const MAP: i64 = 960_000;
pub const CELL: i64 = 32_000;
pub const CELLS: i64 = 30;

/// Aim selects the middle strike/direction. Keep the entire centre-line in the
/// existing range. Near a map edge, shorten its spacing along the same ray.
/// All three centres are committed once; later movement cannot retarget them.
pub fn bomber_centres(from: Point, aim: Point) -> Option<[Point; 3]> {
    let from=from.clamp_map();
    let direction=if aim==from {Point::new(from.x+1,from.y)}else{aim};
    let end_distance=((from.dist(direction) as i64)+d::BOMBER_SPACING)
        .clamp(2*d::BOMBER_SPACING,d::BOMBER_RANGE);
    let raw=from.toward(direction,end_distance);
    let mut fraction=1.0_f64;
    for (base,delta) in [(from.x,raw.x-from.x),(from.y,raw.y-from.y)] {
        if delta>0 {fraction=fraction.min((MAP-1000-base) as f64/delta as f64);}
        if delta<0 {fraction=fraction.min((1000-base) as f64/delta as f64);}
    }
    let end=Point::new(from.x+((raw.x-from.x) as f64*fraction).trunc() as i64,
        from.y+((raw.y-from.y) as f64*fraction).trunc() as i64).clamp_map();
    let first=from.toward(end,((from.dist(end) as i64)-2*d::BOMBER_SPACING).max(0));
    let middle=first.toward(end,(first.dist(end) as i64)/2);
    if first==middle||middle==end {None}else{Some([first,middle,end])}
}

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
            return self;
        }
        let k = len as f64 / dd;
        Point::new(self.x + ((to.x - self.x) as f64 * k).round() as i64, self.y + ((to.y - self.y) as f64 * k).round() as i64)
    }
    /// at most `len` toward `to`
    pub fn reach(self, to: Point, len: i64) -> Point {
        self.toward(to, (self.dist(to) as i64).min(len))
    }
    pub fn clamp_map(self) -> Point {
        Point::new(self.x.clamp(1_000, MAP - 1_000), self.y.clamp(1_000, MAP - 1_000))
    }
    pub fn on_map(self) -> bool {
        (0..=MAP).contains(&self.x) && (0..=MAP).contains(&self.y)
    }
    /// the point at `r` from self at angle `mdeg` (milli-degrees)
    pub fn around(self, r: i64, mdeg: i64) -> Point {
        let a = (mdeg as f64 / 1000.0).to_radians();
        Point::new(self.x + (r as f64 * a.cos()).round() as i64, self.y + (r as f64 * a.sin()).round() as i64)
    }
}

fn sec(t: i64) -> u64 {
    t.max(0) as u64
}
fn pct(base: i64, ratio: i64, ap: i64) -> i64 {
    base + ratio * ap / 100
}
/// one of 8 headings (0 = east, clockwise in screen space) from a direction
pub fn heading(from: Point, to: Point) -> u8 {
    if from == to {
        return 0;
    }
    let mdeg = (((to.y - from.y) as f64).atan2((to.x - from.x) as f64).to_degrees() * 1000.0).round() as i64;
    (((mdeg + 22_500).rem_euclid(360_000)) / 45_000) as u8
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
    pub champion: bool,
    pub tower: bool,
    pub visible: bool,
    pub targetable: bool,
    #[serde(default)]
    pub farmable: bool,
}

/// A plane's host carrier as the host sees it (fighters and the drone).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlaneView {
    pub key: u32,
    pub pos: Point,
    pub hp: i64,
    pub max_hp: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "aim", rename_all = "snake_case")]
pub enum Order {
    Fighters(Point),
    Bomber(Point),
    Repair(Point),
    Carrier(Point),
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
    pub ap: i64,
    pub stunned: bool,
    pub ult_unlocked: bool,
    pub units: Vec<Unit>,
    #[serde(default)]
    pub terrain: Vec<bool>,
    pub profile: [i64; 13],
    /// his fighters' and drone's carriers, alive
    #[serde(default)]
    pub planes: Vec<PlaneView>,
    /// the unit his last accepted basic attack hit (his fighters join it)
    #[serde(default)]
    pub attack_target: Option<usize>,
    /// damage he actually took since the last frame (after the carrier's guard)
    #[serde(default)]
    pub damage_taken: i64,
    pub manual: bool,
    pub order: Option<Order>,
}
impl Frame {
    fn enemy(&self, u: &Unit) -> bool {
        u.team != self.team && u.visible && u.targetable && u.hp > 0
    }
    pub fn enemy_champions(&self) -> impl Iterator<Item = &Unit> {
        self.units.iter().filter(move |u| u.champion && self.enemy(u))
    }
    fn profile(&self, i: usize) -> i64 {
        self.profile.get(i).copied().unwrap_or(50).clamp(0, 100)
    }
    pub fn terrain_at(&self, p: Point) -> bool {
        if !p.on_map() {
            return true;
        }
        self.terrain.get(((p.y / CELL) * CELLS + p.x / CELL) as usize).copied().unwrap_or(false)
    }
    /// no enemy champion near him: his planes may farm
    pub fn calm(&self) -> bool {
        !self.enemy_champions().any(|u| u.pos.within(self.pos, 90_000))
    }
    /// allies (champions of his team, him included): (id, pos, hp, max_hp)
    fn allies(&self) -> Vec<(usize, Point, i64, i64)> {
        std::iter::once((self.owner, self.pos, self.hp, self.max_hp))
            .chain(self.units.iter().filter(|u| u.champion && u.team == self.team && u.hp > 0).map(|u| (u.id, u.pos, u.hp, u.max_hp)))
            .collect()
    }
}

pub const PROFILE_AGGRESSIVE: usize = 6;

// ------------------------------------------------------------------ effects

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Fighter,
    Drone,
    Bomber,
    Chopper,
    Repair,
    Rocket,
}
impl Kind {
    pub fn tag(self) -> &'static str {
        match self {
            Kind::Fighter => "fighter",
            Kind::Drone => "drone",
            Kind::Bomber => "bomber",
            Kind::Chopper => "chopper",
            Kind::Repair => "repair",
            Kind::Rocket => "rocket",
        }
    }
    /// planes with a host carrier unit (targetable, can be shot down)
    pub fn real(self) -> bool {
        matches!(self, Kind::Fighter | Kind::Drone)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case")]
pub enum Effect {
    /// `direct`: a single strike on one target (a V1 parry may cancel it)
    Damage { target: usize, amount: i64, direct: bool },
    Slow { target: usize, percent: i64, ticks: u64 },
    Stun { target: usize, ticks: u64 },
    /// the Sky Carrier's guard on him: it takes `pct`% of the damage he takes (on) or stops (off)
    Guard { on: bool, pct: i64 },
    /// pushed to `to` over `ticks` (an engine glide)
    Knock { target: usize, to: Point, ticks: u64 },
    Heal { target: usize, amount: i64 },
    Shield { target: usize, amount: i64, ticks: u64 },
    /// the burn's look on the target
    Burn { target: usize, ticks: u64 },
    /// forced to attack the plane `key` (the drone)
    Taunt { target: usize, key: u32, ticks: u64 },
    /// a plane with a host carrier unit appears (fighters, the drone)
    SpawnPlane { key: u32, kind: Kind, pos: Point, ticks: u64, hp_pct: i64 },
    PlaneMove { key: u32, to: Point, ticks: u64 },
    RemovePlane { key: u32 },
    /// restore a share of a carrier plane's HP
    RepairPlane { key: u32, pct: i64 },
    /// the workshop's airframe count changed
    Stock { count: i64 },
    Fx { name: String, at: Point, ticks: u64, ambient: bool },
    Pose { name: &'static str, ticks: u64 },
    Log(String),
}

// ------------------------------------------------------------------ state

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Anchor {
    /// circles Wright
    Owner,
    /// circles a point
    At(Point),
    /// follows (and circles) an allied champion
    Ally(usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Plane {
    pub key: u32,
    pub kind: Kind,
    pub pos: Point,
    pub heading: u8,
    pub born: u64,
    /// active from (a warning before)
    pub start: u64,
    pub until: u64,
    pub anchor: Anchor,
    pub orbit: i64,
    pub angle_mdeg: i64,
    /// launched by the Sky Carrier (does not count toward his own fighter cap)
    pub from_carrier: bool,
    pub target: Option<usize>,
    pub moving_until: u64,
    pub next_kit: u64,
    pub next_bomb: u64,
    /// a fighter diving in (no longer has a carrier)
    pub kamikaze: bool,
    /// the drone: arrived and taunted already
    pub taunted: bool,
    pub hover_until: u64,
    pub goal: Point,
    /// a jet's dogfight phase: patrol, run (boosting in), break (banking away), until `phase_until`
    pub phase: Phase,
    pub phase_until: u64,
    /// carpet waves dropped so far (the bomber)
    pub waves: i64,
    /// Accepted bomber geometry; other aircraft have no strike line.
    pub strikes: Option<[Point; 3]>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Phase {
    Patrol,
    Run,
    Break,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Carrier {
    /// over Wright (it follows him)
    pub center: Point,
    pub land_at: u64,
    pub until: u64,
    pub next_fighter: u64,
    pub next_bomber: u64,
    pub next_chopper: u64,
    /// what it can still absorb for him, and its full size
    pub pool: i64,
    pub max_pool: i64,
    /// bombers readied on deck for his Carpet Bomber
    pub ready_bombers: i64,
    pub guarding: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct Core {
    /// tick each skill is ready again: [S1, S2, S3, ult]
    pub ready: [u64; 4],
    pub stock: i64,
    pub next_build: u64,
    pub next_drone: u64,
    pub planes: Vec<Plane>,
    pub next_key: u32,
    pub carrier: Option<Carrier>,
    /// burns: (target, until, damage per half second)
    pub burns: Vec<(usize, u64, i64)>,
    /// repair shields given per ally: (ally, amount, until)
    pub repair_shields: Vec<(usize, i64, u64)>,
    pub last_seen: Vec<(usize, Point)>,
    pub rejection: String,
    pub last_skill: &'static str,
    pub started: bool,
}
impl Default for Core {
    fn default() -> Self {
        Core {
            ready: [0; 4],
            stock: d::WORKSHOP_START_STOCK,
            next_build: 0,
            next_drone: 0,
            planes: vec![],
            next_key: 1,
            carrier: None,
            burns: vec![],
            repair_shields: vec![],
            last_seen: vec![],
            rejection: String::new(),
            last_skill: "",
            started: false,
        }
    }
}

impl Core {
    pub fn respawn(&mut self, tick: u64) {
        let ready = self.ready;
        let next_key = self.next_key;
        *self = Core::default();
        self.ready = ready.map(|r| r.max(tick));
        self.next_key = next_key;
    }

    pub fn ready(&self, slot: usize, tick: u64) -> bool {
        tick >= self.ready[slot]
    }
    /// his own S1 fighters out
    pub fn own_fighters(&self) -> usize {
        self.planes.iter().filter(|p| p.kind == Kind::Fighter && !p.from_carrier && !p.kamikaze).count()
    }

    pub fn step(&mut self, f: &Frame) -> Vec<Effect> {
        let mut out = vec![];
        if !f.alive {
            return out;
        }
        if !self.started {
            self.started = true;
            self.next_build = f.tick + sec(d::WORKSHOP_BUILD_TICKS);
            self.next_drone = f.tick + sec(d::WORKSHOP_DRONE_EVERY_TICKS);
            out.push(Effect::Stock { count: self.stock });
        }
        self.workshop(f, &mut out);
        self.sync_carriers(f, &mut out);
        self.run_planes(f, &mut out);
        self.run_carrier(f, &mut out);
        self.run_burns(f, &mut out);
        let order = if f.manual { f.order } else { self.decide(f) };
        if let Some(o) = order {
            self.execute(f, o, &mut out);
        }
        for u in f.enemy_champions() {
            match self.last_seen.iter_mut().find(|s| s.0 == u.id) {
                Some(s) => s.1 = u.pos,
                None => self.last_seen.push((u.id, u.pos)),
            }
        }
        out
    }

    // ---------------------------------------------------------------- passive: the workshop

    fn workshop(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        if self.stock >= d::WORKSHOP_MAX_STOCK {
            self.next_build = f.tick + sec(d::WORKSHOP_BUILD_TICKS);
        } else if f.tick >= self.next_build {
            self.stock += 1;
            self.next_build = f.tick + sec(d::WORKSHOP_BUILD_TICKS);
            out.push(Effect::Stock { count: self.stock });
        }
        if f.tick >= self.next_drone {
            // to the nearest enemy champion in reach, else where one was last seen; none: try again in 2 s
            let goal = f
                .enemy_champions()
                .filter(|u| u.pos.within(f.pos, d::DRONE_RANGE))
                .min_by_key(|u| (u.pos.d2(f.pos), u.id))
                .map(|u| u.pos)
                .or_else(|| self.last_seen.iter().filter(|s| s.1.within(f.pos, d::DRONE_RANGE)).min_by_key(|s| (s.1.d2(f.pos), s.0)).map(|s| s.1));
            match goal {
                Some(goal) => {
                    self.next_drone = f.tick + sec(d::WORKSHOP_DRONE_EVERY_TICKS);
                    let key = self.launch(f, Kind::Drone, f.pos, sec(d::DRONE_HOVER_TICKS) + 600, Anchor::At(goal), 0, false, out);
                    if let Some(p) = self.planes.iter_mut().find(|p| p.key == key) {
                        p.goal = goal;
                    }
                    out.push(Effect::Fx { name: "wrt_drone_up".into(), at: f.pos, ticks: 18, ambient: true });
                }
                None => self.next_drone = f.tick + 120,
            }
        }
    }

    fn take_airframe(&mut self, out: &mut Vec<Effect>) -> bool {
        if self.stock <= 0 {
            return false;
        }
        self.stock -= 1;
        out.push(Effect::Stock { count: self.stock });
        true
    }

    #[allow(clippy::too_many_arguments)]
    fn launch(&mut self, f: &Frame, kind: Kind, at: Point, ticks: u64, anchor: Anchor, orbit: i64, from_carrier: bool, out: &mut Vec<Effect>) -> u32 {
        let key = self.next_key;
        self.next_key += 1;
        let at = if kind.real() { free_spot(f, f.pos, at) } else { at.clamp_map() };
        self.planes.push(Plane {
            key,
            kind,
            pos: at,
            heading: 0,
            born: f.tick,
            start: f.tick,
            until: f.tick + ticks,
            anchor,
            orbit,
            angle_mdeg: (key as i64 * 97_000) % 360_000,
            from_carrier,
            target: None,
            moving_until: 0,
            next_kit: f.tick + 20,
            next_bomb: f.tick,
            kamikaze: false,
            taunted: false,
            hover_until: 0,
            goal: at,
            phase: Phase::Patrol,
            phase_until: f.tick + (key as u64 % 2) * 6,
            waves: 0,
            strikes: None,
        });
        if kind.real() {
            let hp_pct = if kind == Kind::Drone { d::DRONE_HP_PCT } else { d::FIGHTER_HP_PCT };
            out.push(Effect::SpawnPlane { key, kind, pos: at, ticks: ticks + 2, hp_pct });
        }
        key
    }

    /// Real planes take their position from their carriers; a carrier that's gone means the plane was shot down.
    fn sync_carriers(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        let mut down = vec![];
        for p in self.planes.iter_mut().filter(|p| p.kind.real() && !p.kamikaze) {
            match f.planes.iter().find(|v| v.key == p.key) {
                Some(v) => p.pos = v.pos,
                None if f.tick > p.born + 1 => down.push(p.key),
                None => {}
            }
        }
        for key in down {
            if let Some(i) = self.planes.iter().position(|p| p.key == key) {
                let p = self.planes.remove(i);
                out.push(Effect::Fx { name: "wrt_bomb".into(), at: p.pos, ticks: 18, ambient: false });
                out.push(Effect::Log(format!("{} {key} shot down", p.kind.tag())));
            }
        }
    }

    // ---------------------------------------------------------------- planes

    fn run_planes(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        let keys: Vec<u32> = self.planes.iter().map(|p| p.key).collect();
        for key in keys {
            let Some(i) = self.planes.iter().position(|p| p.key == key) else { continue };
            let mut p = self.planes[i];
            let keep = match p.kind {
                Kind::Fighter => self.fighter(f, &mut p, out),
                Kind::Drone => self.drone(f, &mut p, out),
                Kind::Bomber => self.bomber(f, &mut p, out),
                Kind::Chopper | Kind::Repair => self.repairer(f, &mut p, out),
                Kind::Rocket => false,
            };
            let Some(i) = self.planes.iter().position(|q| q.key == key) else { continue };
            if keep {
                // every plane is drawn every tick at its heading (single-frame phases)
                if f.tick >= p.start {
                    let name=if p.kind==Kind::Fighter {
                        crate::wright_art::fighter_name(p.heading as usize,match p.phase {Phase::Run=>"Run",Phase::Break=>"Break",Phase::Patrol=>"Patrol"},f.tick,p.born)
                    }else{format!("wrt_{}_p{}",p.kind.tag(),p.heading)};
                    out.push(Effect::Fx { name, at: p.pos, ticks: 1, ambient: false });
                }
                self.planes[i] = p;
            } else {
                self.planes.remove(i);
                if p.kind.real() && !p.kamikaze {
                    out.push(Effect::RemovePlane { key });
                }
            }
        }
        self.repair_shields.retain(|s| f.tick < s.2);
    }

    /// Glide a carrier plane (or move a simulated one) toward `goal` by `step` over the next 6 ticks.
    fn fly(&self, f: &Frame, p: &mut Plane, goal: Point, step: i64, out: &mut Vec<Effect>) {
        if p.kind.real() && !p.kamikaze {
            if f.tick < p.moving_until || f.tick % 6 != 0 {
                return;
            }
            let to = free_spot(f, p.pos, p.pos.reach(goal, step * 6));
            if !to.within(p.pos, 300) {
                p.heading = heading(p.pos, to);
                p.moving_until = f.tick + 6;
                out.push(Effect::PlaneMove { key: p.key, to, ticks: 6 });
            }
        } else {
            let to = p.pos.reach(goal, step).clamp_map();
            if !to.within(p.pos, 50) {
                p.heading = heading(p.pos, to);
            }
            p.pos = to;
        }
    }

    /// His target first, then the nearest enemy champion in reach, then minions and camps (they clear like normal).
    /// Never a tower.
    fn fighter_target<'a>(&self, f: &'a Frame, p: &Plane) -> Option<&'a Unit> {
        let reach = d::FIGHTER_AGGRO_RANGE;
        let ok = |u: &&Unit| f.enemy(u) && !u.tower && u.pos.within(f.pos, reach);
        f.attack_target
            .and_then(|t| f.units.iter().find(|u| u.id == t))
            .filter(|u| ok(u) && (u.champion || !f.enemy_champions().any(|c| c.pos.within(f.pos, reach))))
            .or_else(|| f.units.iter().filter(|u| u.champion).filter(ok).min_by_key(|u| (u.pos.d2(p.pos), u.id)))
            .or_else(|| f.units.iter().filter(|u| !u.champion).filter(ok).min_by_key(|u| (u.pos.d2(p.pos), u.id)))
    }

    /// A jet flies sorties from him: it waits at home (circling him), commits to where its target is, flies there, strikes
    /// once (a gun burst; every few sorties a rocket) and flies back. No homing: a target that moves away is missed. The
    /// closer he is to the fight, the shorter the round trip, so the faster they attack. Always targetable.
    fn fighter(&mut self, f: &Frame, p: &mut Plane, out: &mut Vec<Effect>) -> bool {
        let speed = d::FIGHTER_SORTIE_SPEED;
        if p.kamikaze {
            // diving onto the spot it committed to, then the blast
            let goal = p.goal;
            self.fly(f, p, goal, speed, out);
            if p.pos.within(goal, 4_000) || f.tick >= p.until {
                self.kamikaze_blast(f, p.pos, out);
                return false;
            }
            return true;
        }
        let low = f.planes.iter().find(|v| v.key == p.key).is_some_and(|v| v.max_hp > 0 && v.hp * 100 <= v.max_hp * d::FIGHTER_KAMIKAZE_HP_PCT);
        // time's up or nearly shot down: dive at the nearest enemy's spot (committed); no one near: it just leaves
        if f.tick >= p.until || low {
            let near = f.units.iter().filter(|u| f.enemy(u) && !u.tower && u.pos.within(p.pos, 80_000)).min_by_key(|u| (u.pos.d2(p.pos), u.id));
            return match near {
                Some(u) => {
                    p.kamikaze = true;
                    p.goal = u.pos;
                    p.until = f.tick + 40;
                    out.push(Effect::RemovePlane { key: p.key });
                    out.push(Effect::Log(format!("fighter {} kamikaze", p.key)));
                    true
                }
                None => false,
            };
        }
        match p.phase {
            Phase::Run => {
                // outbound to the committed spot
                let goal = p.goal;
                self.fly(f, p, goal, speed, out);
                if p.pos.within(goal, 4_000) {
                    self.strike(f, p, out);
                    p.phase = Phase::Break;
                }
            }
            Phase::Break => {
                // home to him
                self.fly(f, p, f.pos, speed, out);
                if p.pos.within(f.pos, d::FIGHTER_HOME_RADIUS + 4_000) {
                    p.phase = Phase::Patrol;
                    p.phase_until = f.tick + sec(d::FIGHTER_REST_TICKS);
                }
            }
            Phase::Patrol => {
                let target = (f.tick >= p.phase_until).then(|| self.fighter_target(f, p).map(|u| (u.id, u.pos))).flatten();
                match target {
                    Some((id, at)) => {
                        // launch: commit to where the target is now
                        p.phase = Phase::Run;
                        p.target = Some(id);
                        p.goal = at;
                        self.fly(f, p, at, speed, out);
                    }
                    None => {
                        // at home: tight laps around him
                        p.angle_mdeg = (p.angle_mdeg + 8_000).rem_euclid(360_000);
                        let goal = f.pos.around(d::FIGHTER_HOME_RADIUS, p.angle_mdeg + if p.from_carrier { 60_000 } else { 0 });
                        self.fly(f, p, goal, d::FIGHTER_SPEED * 2, out);
                    }
                }
            }
        }
        true
    }

    /// One sortie's strike at its committed spot: the committed target if it's still there, else the nearest enemy
    /// within the strike radius, else a miss. Every few sorties it's a rocket (a direct hit, with splash).
    fn strike(&mut self, f: &Frame, p: &mut Plane, out: &mut Vec<Effect>) {
        p.waves += 1;
        let rocket = d::FIGHTER_ROCKET_EVERY > 0 && p.waves % d::FIGHTER_ROCKET_EVERY == 0;
        let spot = p.goal;
        let in_reach = |u: &&Unit| f.enemy(u) && !u.tower && u.pos.within(spot, d::FIGHTER_GUN_RANGE + u.radius);
        let hit = p
            .target
            .and_then(|t| f.units.iter().find(|u| u.id == t))
            .filter(in_reach)
            .or_else(|| f.units.iter().filter(in_reach).min_by_key(|u| (u.pos.d2(spot), u.id)));
        let Some(u) = hit else {
            out.push(Effect::Log(format!("fighter {} sortie missed", p.key)));
            return;
        };
        if rocket {
            let dmg = pct(d::FIGHTER_ROCKET_BASE, d::FIGHTER_ROCKET_AP_RATIO, f.ap);
            out.push(Effect::Damage { target: u.id, amount: dmg, direct: true });
            for o in f.units.iter().filter(|o| o.id != u.id && f.enemy(o) && !o.tower && o.pos.within(u.pos, d::FIGHTER_ROCKET_SPLASH + o.radius)) {
                out.push(Effect::Damage { target: o.id, amount: dmg / 2, direct: false });
            }
            out.push(Effect::Fx { name: "wrt_bomb".into(), at: u.pos, ticks: 16, ambient: false });
        } else {
            out.push(Effect::Damage { target: u.id, amount: pct(d::FIGHTER_SORTIE_BASE, d::FIGHTER_SORTIE_AP_RATIO, f.ap), direct: false });
            out.push(Effect::Fx { name: "wrt_mg".into(), at: u.pos, ticks: 8, ambient: true });
        }
    }

    fn kamikaze_blast(&mut self, f: &Frame, at: Point, out: &mut Vec<Effect>) {
        out.push(Effect::Fx { name: "wrt_kamikaze".into(), at, ticks: 24, ambient: false });
        let dmg = pct(d::FIGHTER_KAMIKAZE_BASE, d::FIGHTER_KAMIKAZE_AP_RATIO, f.ap);
        for u in f.units.iter().filter(|u| f.enemy(u) && !u.tower && u.pos.within(at, d::FIGHTER_KAMIKAZE_RADIUS + u.radius)) {
            out.push(Effect::Damage { target: u.id, amount: dmg, direct: true });
            // away from the blast (straight east when it lands right on them)
            let from = if u.pos == at { Point::new(at.x - 1, at.y) } else { at };
            let to = free_spot(f, u.pos, u.pos.toward(from, -d::FIGHTER_KAMIKAZE_KNOCK));
            out.push(Effect::Knock { target: u.id, to, ticks: 12 });
            let per_half = (u.max_hp * d::FIGHTER_BURN_PERMILLE_PER_SECOND / 1000 / 2).max(1);
            self.burns.retain(|b| b.0 != u.id);
            self.burns.push((u.id, f.tick + sec(d::FIGHTER_BURN_TICKS), per_half));
            out.push(Effect::Burn { target: u.id, ticks: sec(d::FIGHTER_BURN_TICKS) });
        }
    }

    fn run_burns(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        self.burns.retain(|b| f.tick < b.1);
        if f.tick % 30 != 0 {
            return;
        }
        for &(t, _, amount) in &self.burns {
            if f.units.iter().any(|u| u.id == t && u.hp > 0) {
                out.push(Effect::Damage { target: t, amount, direct: false });
            }
        }
    }

    fn drone(&mut self, f: &Frame, p: &mut Plane, out: &mut Vec<Effect>) -> bool {
        if f.tick >= p.until || (p.hover_until > 0 && f.tick >= p.hover_until) {
            return false;
        }
        if !p.taunted {
            self.fly(f, p, p.goal, d::DRONE_SPEED, out);
            if p.pos.within(p.goal, 6_000) {
                p.taunted = true;
                p.hover_until = f.tick + sec(d::DRONE_HOVER_TICKS);
                for u in f.enemy_champions().filter(|u| u.pos.within(p.pos, d::DRONE_TAUNT_RADIUS + u.radius)) {
                    out.push(Effect::Taunt { target: u.id, key: p.key, ticks: sec(d::DRONE_TAUNT_TICKS) });
                }
                out.push(Effect::Fx { name: "wrt_taunt".into(), at: p.pos, ticks: 30, ambient: false });
            }
        }
        true
    }

    /// The Carpet Bomber: after its ready-up it sweeps across the marked area dropping carpet waves; the first ones slow,
    /// the last stuns. Every enemy in the area (never a tower) is hit once per wave.
    fn bomber(&mut self, f: &Frame, p: &mut Plane, out: &mut Vec<Effect>) -> bool {
        let Some(strikes)=p.strikes else {return false};
        let center=strikes[(p.waves as usize).min(strikes.len()-1)];
        if p.waves >= d::BOMBER_WAVES {
            // flying out
            let out_to = strikes[2].toward(strikes[0], -60_000);
            self.fly(f, p, out_to, d::BOMBER_SPEED, out);
            return f.tick < p.until;
        }
        if f.tick < p.start {
            // flying in toward the area for the first wave
            let step = (p.pos.dist(center) as i64 / (p.start - f.tick).max(1) as i64).max(1);
            self.fly(f, p, center, step, out);
            return true;
        }
        self.fly(f, p, center, d::BOMBER_SPEED, out);
        if f.tick >= p.next_bomb {
            p.next_bomb = f.tick + sec(d::BOMBER_WAVE_EVERY_TICKS);
            p.waves += 1;
            let last = p.waves >= d::BOMBER_WAVES;
            // Each warned centre gets one large explosion and one packet per
            // victim. The line is committed; it never follows later target motion.
            out.push(Effect::Fx { name: format!("wrt_bomber_wave{}", p.waves), at: center, ticks: 14, ambient: false });
            let dmg = pct(d::BOMBER_WAVE_BASE, d::BOMBER_WAVE_AP_RATIO, f.ap);
            for u in f.units.iter().filter(|u| f.enemy(u) && !u.tower && u.pos.within(center, d::BOMBER_RADIUS + u.radius)) {
                out.push(Effect::Damage { target: u.id, amount: dmg, direct: false });
                if last {
                    out.push(Effect::Stun { target: u.id, ticks: sec(d::BOMBER_STUN_TICKS) });
                } else {
                    out.push(Effect::Slow { target: u.id, percent: d::BOMBER_SLOW_PCT, ticks: sec(d::BOMBER_SLOW_TICKS) });
                }
            }
            if last {
                p.until = f.tick + 40;
            }
        }
        true
    }

    /// The repair chopper (circles him, or the carrier) and the repair plane (flies to an ally anywhere and circles them).
    fn repairer(&mut self, f: &Frame, p: &mut Plane, out: &mut Vec<Effect>) -> bool {
        if f.tick >= p.until {
            return false;
        }
        let allies = f.allies();
        let center = match p.anchor {
            Anchor::Owner => f.pos,
            Anchor::At(c) => c,
            Anchor::Ally(id) => match allies.iter().find(|a| a.0 == id) {
                Some(a) => a.1,
                None => return false,
            },
        };
        p.angle_mdeg = (p.angle_mdeg + 3_000).rem_euclid(360_000);
        let goal = center.around(p.orbit, p.angle_mdeg);
        let speed = if p.kind == Kind::Repair { d::REPAIR_PLANE_SPEED } else { d::REPAIR_PLANE_SPEED / 2 };
        self.fly(f, p, goal, speed, out);
        let reach = if p.from_carrier { d::CARRIER_KIT_RADIUS } else { d::REPAIR_KIT_RANGE + p.orbit };
        if f.tick < p.next_kit || !p.pos.within(center, reach) {
            return true;
        }
        p.next_kit = f.tick + sec(d::REPAIR_KIT_EVERY_TICKS);
        // the lowest-HP ally near it (the repair plane: its ally)
        let near = |a: &&(usize, Point, i64, i64)| a.1.within(p.pos, d::REPAIR_KIT_RANGE);
        let pick = match p.anchor {
            Anchor::Ally(id) => allies.iter().find(|a| a.0 == id),
            Anchor::At(c) => allies.iter().filter(|a| a.1.within(c, d::CARRIER_KIT_RADIUS)).min_by_key(|a| (a.2 * 1000 / a.3.max(1), a.0)),
            Anchor::Owner if p.from_carrier => allies.iter().filter(|a| a.1.within(f.pos, d::CARRIER_KIT_RADIUS)).min_by_key(|a| (a.2 * 1000 / a.3.max(1), a.0)),
            Anchor::Owner => allies.iter().filter(near).min_by_key(|a| (a.2 * 1000 / a.3.max(1), a.0)),
        };
        if let Some(&(id, pos, _, max_hp)) = pick {
            self.kit(f, id, pos, max_hp, out);
        }
        // the chopper also patches up his fighters around it
        if p.kind == Kind::Chopper {
            for q in self.planes.iter().filter(|q| q.kind == Kind::Fighter && !q.kamikaze && q.pos.within(p.pos, d::REPAIR_KIT_RANGE)) {
                out.push(Effect::RepairPlane { key: q.key, pct: d::REPAIR_FIGHTER_REPAIR_PCT });
            }
        }
        true
    }

    fn kit(&mut self, f: &Frame, ally: usize, at: Point, max_hp: i64, out: &mut Vec<Effect>) {
        out.push(Effect::Heal { target: ally, amount: (max_hp * d::REPAIR_HEAL_PERMILLE / 1000).max(1) });
        let cap = pct(d::REPAIR_SHIELD_CAP_BASE, d::REPAIR_SHIELD_CAP_AP_RATIO, f.ap);
        let have: i64 = self.repair_shields.iter().filter(|s| s.0 == ally).map(|s| s.1).sum();
        let add = pct(d::REPAIR_SHIELD_BASE, d::REPAIR_SHIELD_AP_RATIO, f.ap).min(cap - have);
        if add > 0 {
            let ticks = sec(d::REPAIR_SHIELD_TICKS);
            self.repair_shields.push((ally, add, f.tick + ticks));
            out.push(Effect::Shield { target: ally, amount: add, ticks });
        }
        out.push(Effect::Fx { name: "wrt_repair_kit".into(), at, ticks: 20, ambient: true });
    }

    // ---------------------------------------------------------------- ult: the Sky Carrier

    fn run_carrier(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        let Some(mut c) = self.carrier else { return };
        // it follows him
        c.center = f.pos;
        if f.tick < c.land_at {
            self.carrier = Some(c);
            return;
        }
        if f.tick == c.land_at {
            // the drop on him: no damage, a knockback away from him (the closer, the farther) and a small slow
            out.push(Effect::Fx { name: "wrt_carrier_land".into(), at: c.center, ticks: 30, ambient: false });
            out.push(Effect::Fx { name: "wrt_knock_ring".into(), at: c.center, ticks: 24, ambient: false });
            for u in f.units.iter().filter(|u| f.enemy(u) && !u.tower && u.pos.within(c.center, d::CARRIER_KNOCK_RADIUS + u.radius)) {
                let dist = (u.pos.dist(c.center) as i64).min(d::CARRIER_KNOCK_RADIUS);
                let push = d::CARRIER_KNOCK_MAX * (d::CARRIER_KNOCK_RADIUS - dist) / d::CARRIER_KNOCK_RADIUS;
                let from = if u.pos == c.center { Point::new(c.center.x - 1, c.center.y) } else { c.center };
                if push > 0 {
                    let to = free_spot(f, u.pos, u.pos.toward(from, -push));
                    out.push(Effect::Knock { target: u.id, to, ticks: sec(d::CARRIER_KNOCK_TICKS) });
                }
                out.push(Effect::Slow { target: u.id, percent: d::CARRIER_SLOW_PCT, ticks: sec(d::CARRIER_SLOW_TICKS) });
            }
            c.guarding = true;
            out.push(Effect::Guard { on: true, pct: d::CARRIER_GUARD_PCT });
            out.push(Effect::Log("the sky carrier lands on him".into()));
        } else if c.guarding && f.damage_taken > 0 {
            // it took guardPct of what he was hit for: what got through is the rest
            let through = (100 - d::CARRIER_GUARD_PCT).max(1);
            c.pool -= f.damage_taken * d::CARRIER_GUARD_PCT / through;
        }
        if f.tick >= c.until || c.pool <= 0 {
            self.carrier = None;
            if c.guarding {
                out.push(Effect::Guard { on: false, pct: 0 });
            }
            out.push(Effect::Log(if c.pool <= 0 { "the sky carrier is shot out of the sky".into() } else { "the sky carrier leaves".into() }));
            return;
        }
        out.push(Effect::Fx { name: "wrt_carrier".into(), at: c.center, ticks: 1, ambient: false });
        let carrier_fighters = self.planes.iter().filter(|p| p.kind == Kind::Fighter && p.from_carrier && !p.kamikaze).count() as i64;
        if f.tick >= c.next_fighter {
            c.next_fighter = f.tick + sec(d::CARRIER_FIGHTER_EVERY_TICKS);
            if carrier_fighters < d::CARRIER_MAX_FIGHTERS {
                self.launch(f, Kind::Fighter, c.center, sec(d::CARRIER_FIGHTER_TICKS), Anchor::Owner, 0, true, out);
            }
        }
        if f.tick >= c.next_bomber {
            c.next_bomber = f.tick + sec(d::CARRIER_BOMBER_EVERY_TICKS);
            if c.ready_bombers < d::CARRIER_MAX_READY_BOMBERS {
                c.ready_bombers += 1;
                out.push(Effect::Log(format!("bomber ready on deck ({})", c.ready_bombers)));
            }
        }
        if f.tick >= c.next_chopper {
            c.next_chopper = f.tick + sec(d::CARRIER_CHOPPER_EVERY_TICKS);
            self.launch(f, Kind::Chopper, c.center, sec(d::REPAIR_TICKS), Anchor::Owner, d::REPAIR_CHOPPER_ORBIT, true, out);
        }
        self.carrier = Some(c);
    }

    /// Ready bombers on the carrier's deck (S2 uses them while it's up).
    pub fn ready_bombers(&self, tick: u64) -> i64 {
        self.carrier.filter(|c| tick >= c.land_at).map_or(0, |c| c.ready_bombers)
    }

    // ---------------------------------------------------------------- orders

    fn execute(&mut self, f: &Frame, o: Order, out: &mut Vec<Effect>) {
        let ok = match o {
            Order::Fighters(_) => self.fighters(f, out),
            Order::Bomber(p) => self.bomber_cast(f, p, out),
            Order::Repair(_) => self.repair_cast(f, out),
            Order::Carrier(p) => self.carrier_cast(f, p, out),
        };
        if !ok {
            self.rejection = format!("{o:?} rejected at tick {}", f.tick);
        }
    }

    fn fighters(&mut self, f: &Frame, out: &mut Vec<Effect>) -> bool {
        if f.stunned || !self.ready(0, f.tick) || !self.take_airframe(out) {
            return false;
        }
        self.ready[0] = f.tick + sec(d::FIGHTER_COOLDOWN_TICKS);
        for _ in 0..d::FIGHTER_PER_CAST {
            // at most maxAlive: the oldest of his own leaves
            while self.own_fighters() as i64 >= d::FIGHTER_MAX_ALIVE {
                let Some(i) = self.planes.iter().enumerate().filter(|(_, p)| p.kind == Kind::Fighter && !p.from_carrier && !p.kamikaze).min_by_key(|(_, p)| (p.born, p.key)).map(|(i, _)| i) else { break };
                let old = self.planes.remove(i);
                out.push(Effect::RemovePlane { key: old.key });
            }
            self.launch(f, Kind::Fighter, f.pos, sec(d::FIGHTER_TICKS), Anchor::Owner, 0, false, out);
        }
        out.push(Effect::Pose { name: "skill1", ticks: 18 });
        self.last_skill = "fighters";
        true
    }

    fn bomber_cast(&mut self, f: &Frame, aim: Point, out: &mut Vec<Effect>) -> bool {
        if f.stunned {
            return false;
        }
        let Some(strikes)=bomber_centres(f.pos,aim) else{return false};
        // a ready bomber from the carrier: no airframe, no cooldown, a short mark
        let from_deck = self.ready_bombers(f.tick) > 0;
        if from_deck {
            if let Some(c) = self.carrier.as_mut() {
                c.ready_bombers -= 1;
            }
        } else {
            if !self.ready(1, f.tick) || !self.take_airframe(out) {
                return false;
            }
            self.ready[1] = f.tick + sec(d::BOMBER_COOLDOWN_TICKS);
        }
        let mark = sec(if from_deck { d::BOMBER_READY_MARK_TICKS } else { d::BOMBER_WARN_TICKS });
        let key = self.launch(f, Kind::Bomber, f.pos, mark + sec(d::BOMBER_WAVES * d::BOMBER_WAVE_EVERY_TICKS) + 90, Anchor::At(strikes[0]), 0, from_deck, out);
        if let Some(p) = self.planes.iter_mut().find(|p| p.key == key) {
            p.start = f.tick + mark;
            p.next_bomb = p.start;
            p.strikes=Some(strikes);
            p.goal=strikes[2];
        }
        for (i,center) in strikes.into_iter().enumerate() {
            out.push(Effect::Fx { name: format!("wrt_warn{}",i+1), at:center, ticks:mark+sec(i as i64*d::BOMBER_WAVE_EVERY_TICKS), ambient:false });
        }
        out.push(Effect::Pose { name: "skill2", ticks: 18 });
        out.push(Effect::Log(if from_deck { "a ready bomber from the carrier".into() } else { "carpet bomber called".into() }));
        self.last_skill = "bomber";
        true
    }

    fn repair_cast(&mut self, f: &Frame, out: &mut Vec<Effect>) -> bool {
        if f.stunned || !self.ready(2, f.tick) || !self.take_airframe(out) {
            return false;
        }
        self.ready[2] = f.tick + sec(d::REPAIR_COOLDOWN_TICKS);
        self.launch(f, Kind::Chopper, f.pos, sec(d::REPAIR_TICKS), Anchor::Owner, d::REPAIR_CHOPPER_ORBIT, false, out);
        // the plane goes to the most hurt ally anywhere (him included)
        let allies = f.allies();
        let most_hurt = allies.iter().min_by_key(|a| (a.2 * 1000 / a.3.max(1), a.0)).map_or(f.owner, |a| a.0);
        self.launch(f, Kind::Repair, f.pos, sec(d::REPAIR_TICKS), Anchor::Ally(most_hurt), d::REPAIR_CHOPPER_ORBIT / 2, false, out);
        out.push(Effect::Pose { name: "skill3", ticks: 18 });
        out.push(Effect::Log(format!("repair plane to {most_hurt}")));
        self.last_skill = "repair";
        true
    }

    fn carrier_cast(&mut self, f: &Frame, _aim: Point, out: &mut Vec<Effect>) -> bool {
        if f.stunned || !f.ult_unlocked || !self.ready(3, f.tick) || self.carrier.is_some() {
            return false;
        }
        self.ready[3] = f.tick + sec(d::CARRIER_COOLDOWN_TICKS);
        let land_at = f.tick + sec(d::CARRIER_WARN_TICKS);
        let pool = (f.max_hp * d::CARRIER_POOL_PCT / 100).max(1);
        self.carrier = Some(Carrier {
            center: f.pos,
            land_at,
            until: land_at + sec(d::CARRIER_TICKS),
            next_fighter: land_at + 10,
            next_bomber: land_at + sec(d::CARRIER_BOMBER_EVERY_TICKS),
            next_chopper: land_at + 20,
            pool,
            max_pool: pool,
            ready_bombers: 0,
            guarding: false,
        });
        out.push(Effect::Fx { name: "wrt_carrier_shadow".into(), at: f.pos, ticks: sec(d::CARRIER_WARN_TICKS), ambient: false });
        out.push(Effect::Pose { name: "ult", ticks: 24 });
        out.push(Effect::Log("sky carrier called down on him".into()));
        self.last_skill = "carrier";
        true
    }

    // ---------------------------------------------------------------- AI

    fn decide(&self, f: &Frame) -> Option<Order> {
        if f.stunned {
            return None;
        }
        let near: Vec<&Unit> = f.enemy_champions().filter(|u| u.pos.within(f.pos, 120_000)).collect();
        let allies = f.allies();
        let hurt = allies.iter().filter(|a| a.2 * 100 < a.3 * 60).min_by_key(|a| (a.2 * 1000 / a.3.max(1), a.0)).copied();
        // the carrier is a peel and a guard: when he's under pressure
        if f.ult_unlocked && self.ready(3, f.tick) && self.carrier.is_none() {
            let close = near.iter().filter(|u| u.pos.within(f.pos, d::CARRIER_PRESSURE_RANGE)).count();
            let low = f.hp * 100 < f.max_hp * 50;
            if close >= 2 || (low && close >= 1) || (close >= 1 && f.profile(PROFILE_AGGRESSIVE) >= 80) {
                return Some(Order::Carrier(f.pos));
            }
        }
        let best_bomb = || {
            near.iter()
                .filter(|u| u.pos.within(f.pos, d::BOMBER_RANGE))
                .filter_map(|u|bomber_centres(f.pos,u.pos).map(|centres|
                    (near.iter().filter(|o|centres.iter().any(|c|o.pos.within(*c,d::BOMBER_RADIUS+o.radius))).count(),*u)))
                .max_by_key(|(n, u)| (*n, std::cmp::Reverse(u.id)))
                .map(|(_, u)| u.pos)
        };
        // a ready bomber on the carrier's deck: use it on the best spot
        if self.ready_bombers(f.tick) > 0 {
            if let Some(at) = best_bomb() {
                return Some(Order::Bomber(at));
            }
        }
        if self.stock <= 0 {
            return None;
        }
        // keep an airframe for repairs when someone is hurt
        if let Some(a) = hurt {
            if self.ready(2, f.tick) {
                return Some(Order::Repair(a.1));
            }
        }
        if self.ready(1, f.tick) {
            if let Some(at) = best_bomb() {
                return Some(Order::Bomber(at));
            }
            // calm: carpet-bomb a camp or a wave (3+ units)
            if f.calm() {
                let farm: Vec<&Unit> = f.units.iter().filter(|u| f.enemy(u) && u.farmable && u.pos.within(f.pos, d::BOMBER_RANGE)).collect();
                if let Some(u) = farm.iter().find(|u|bomber_centres(f.pos,u.pos).is_some_and(|centres|
                    farm.iter().filter(|o|centres.iter().any(|c|o.pos.within(*c,d::BOMBER_RADIUS+o.radius))).count()>=3)) {
                    return Some(Order::Bomber(u.pos));
                }
            }
        }
        if self.ready(0, f.tick) && (self.own_fighters() as i64) < d::FIGHTER_MAX_ALIVE {
            let target = !near.is_empty() || (f.calm() && f.units.iter().any(|u| f.enemy(u) && u.farmable && u.pos.within(f.pos, d::FIGHTER_AGGRO_RANGE)));
            if target {
                return Some(Order::Fighters(f.pos));
            }
        }
        None
    }
}

/// The nearest open ground to `to`, stepping back toward `from` (never terrain or off the map).
pub fn free_spot(f: &Frame, from: Point, to: Point) -> Point {
    let mut p = to.clamp_map();
    for _ in 0..200 {
        if !f.terrain_at(p) {
            return p;
        }
        if p.within(from, 2_000) {
            break;
        }
        p = p.toward(from, 2_000);
    }
    from.clamp_map()
}
