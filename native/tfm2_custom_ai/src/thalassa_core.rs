//! Thalassa, the mermaid harpist: a deterministic core (snapshot in, effects out). `thalassa.rs` builds the frame from
//! the sim and applies the effects; the Lab (`native/thalassa_lab`) runs this same file.
//!
//! Water is the kit: her pools, whirlpools, the clone's rescue pool and the trident's landing pool all count as water,
//! and so does a short "wet" window after a slide on land.
//!   passive Tidebound: in water +move speed, +range, bouncing harp bolts, 1% HP a second (a shield, capped at 10% max
//!           HP, once she's full). On land: no penalty.
//!   S1      Tide Pool: on land a pool forms below her; in water the pool she stands in surges to the target and becomes
//!           a bigger whirlpool (slow, pull, damage; 2+ enemy champions bump and are stunned), and a new whirlpool forms
//!           below her.
//!   S2      Dive: in water, a pool opens at the target and she dives across; on land, a short slide that leaves her wet.
//!   S3      Tide Twin: a water clone (12 s, can be killed) copies her S1, S2 and harp bolts at half power to the same
//!           point; an ally (or she) stunned near it gets a rescue pool (cleanse, shield, CC immunity); spawned on water
//!           a tide link lets her swap with it once when engaged, leaving a whirlpool.
//!   Ult     Trident of the Deep: a slow, global, steerable (25 degrees in total) trident; the first enemy champion hit
//!           is stunned (longer from water); it leaves a pool (a whirlpool from water) where it lands.
use crate::thalassa_data as d;
use serde::{Deserialize, Serialize};

pub const MAP: i64 = 960_000;
pub const CELL: i64 = 32_000;
pub const CELLS: i64 = 30;

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
}

fn sec(t: i64) -> u64 {
    t.max(0) as u64
}
fn pct(base: i64, ratio: i64, ap: i64) -> i64 {
    base + ratio * ap / 100
}
fn jitter(tick: u64, salt: u64, n: i64) -> i64 {
    let mut h = tick.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ salt.wrapping_mul(0xC2B2_AE3D_27D4_EB4F);
    h ^= h >> 31;
    (h % (n.max(1) as u64)) as i64
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
    /// stunned, knocked up or otherwise held by someone else's crowd control
    #[serde(default)]
    pub stunned: bool,
    #[serde(default)]
    pub velocity: Point,
}

/// Her water clone as the host sees it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CloneView {
    pub id: usize,
    pub pos: Point,
    pub hp: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "aim", rename_all = "snake_case")]
pub enum Order {
    TidePool(Point),
    Dive(Point),
    Twin(Point),
    Ult(Point),
    Steer(Point),
    Swap,
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
    /// her ability power
    pub ap: i64,
    /// her current shield total (any source); her overheal never counts more than this
    #[serde(default)]
    pub shield: i64,
    /// stunned (someone else's hard crowd control; her own named motion never counts)
    pub stunned: bool,
    pub ult_unlocked: bool,
    pub units: Vec<Unit>,
    #[serde(default)]
    pub terrain: Vec<bool>,
    pub profile: [i64; 13],
    /// her clone, if one stands
    #[serde(default)]
    pub clone: Option<CloneView>,
    /// her basic attacks the game accepted since the last frame (targets)
    #[serde(default)]
    pub basic_hits: Vec<usize>,
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
}

pub const PROFILE_JUDGEMENT: usize = 1;
pub const PROFILE_SKILL_HIT: usize = 4;
pub const PROFILE_AGGRESSIVE: usize = 6;

// ------------------------------------------------------------------ effects

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case")]
pub enum Effect {
    /// magic damage from her (or her clone: `from_clone`)
    Damage { target: usize, amount: i64, from_clone: bool },
    Stun { target: usize, ticks: u64 },
    Slow { target: usize, percent: i64, ticks: u64 },
    /// an engine push (crate::motion::glide): whirlpool pulls
    Pull { target: usize, to: Point, ticks: u64 },
    /// she moves herself through the engine (dive, slide)
    Glide { to: Point, ticks: u64 },
    /// unseen while under water (the dive)
    Submerge { ticks: u64 },
    Heal { amount: i64 },
    Shield { target: usize, amount: i64, ticks: u64 },
    Cleanse { target: usize },
    CcImmune { target: usize, ticks: u64 },
    /// water mode on or off (the passive's stat buff)
    Water { on: bool },
    PoolUp { id: u32, center: Point, radius: i64, until: u64, whirl: bool },
    PoolDown { id: u32 },
    SpawnClone { pos: Point, ticks: u64 },
    /// her clone flies or dives over `ticks` (an engine glide of its hidden carrier; its body is drawn as an effect)
    CloneMove { to: Point, ticks: u64 },
    /// the trident impales a champion: stun unless V1 parries it (the bridge checks; a parry also cancels damage)
    TridentHit { target: usize, at: Point, ticks: u64 },
    RemoveClone,
    /// trade places with her clone (a deliberate swap)
    Swap { owner_to: Point, clone_to: Point },
    Fx { name: String, at: Point, ticks: u64, ambient: bool },
    Pose { name: &'static str, ticks: u64 },
    Log(String),
}

// ------------------------------------------------------------------ state

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pool {
    pub id: u32,
    pub center: Point,
    pub radius: i64,
    pub until: u64,
    pub whirl: bool,
}
impl Pool {
    pub fn holds(&self, p: Point) -> bool {
        p.within(self.center, self.radius)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Twin {
    pub until: u64,
    /// the twin is flying or diving until this tick (its body is drawn every tick meanwhile)
    pub moving_until: u64,
    pub linked: bool,
    pub swapped: bool,
    /// flying to rescue this ally, arriving at tick
    pub rescue: Option<(usize, u64)>,
    /// its own harp bolt is ready again at this tick (it attacks on its own, like someone playing her)
    pub next_bolt: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Trident {
    pub at: Point,
    pub heading_mdeg: i64,
    pub spent_mdeg: i64,
    pub speed: i64,
    pub travelled: i64,
    pub from_water: bool,
    pub target: Option<usize>,
    pub guide: Option<Point>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Cast {
    TidePool { until: u64, aim: Point },
    Trident { until: u64, aim: Point, from_water: bool, target: Option<usize> },
}

#[derive(Clone, Debug, Serialize)]
pub struct Core {
    /// tick each skill is ready again: [S1, S2, S3, ult]
    pub ready: [u64; 4],
    pub pools: Vec<Pool>,
    pub next_pool: u32,
    pub wet_until: u64,
    pub in_water: bool,
    /// her overheal right now (own unexpired increments, never above her actual shield)
    pub overheal: i64,
    /// her own overheal increments: (amount, expires)
    pub overheal_parts: Vec<(i64, u64)>,
    pub cast: Option<Cast>,
    /// diving or sliding until
    pub moving_until: u64,
    pub twin: Option<Twin>,
    pub trident: Option<Trident>,
    /// recent HP for the tide link's "engaged" read: (tick, hp)
    pub hp_log: Vec<(u64, i64)>,
    pub last_positions: Vec<(usize, Point)>,
    pub rejection: String,
    pub last_skill: &'static str,
}
impl Default for Core {
    fn default() -> Self {
        Core {
            ready: [0; 4],
            pools: vec![],
            next_pool: 1,
            wet_until: 0,
            in_water: false,
            overheal: 0,
            overheal_parts: vec![],
            cast: None,
            moving_until: 0,
            twin: None,
            trident: None,
            hp_log: vec![],
            last_positions: vec![],
            rejection: String::new(),
            last_skill: "",
        }
    }
}

impl Core {
    pub fn respawn(&mut self, tick: u64) {
        let ready = self.ready;
        let next_pool = self.next_pool;
        *self = Core::default();
        self.ready = ready.map(|r| r.max(tick));
        self.next_pool = next_pool;
    }

    pub fn ready(&self, slot: usize, tick: u64) -> bool {
        tick >= self.ready[slot]
    }
    pub fn busy(&self, tick: u64) -> bool {
        self.cast.is_some() || tick < self.moving_until
    }
    pub fn pool_at(&self, p: Point) -> Option<&Pool> {
        self.pools.iter().filter(|q| q.holds(p)).min_by_key(|q| (q.center.d2(p), q.id))
    }
    /// In water: inside one of her pools, or still wet.
    pub fn water_at(&self, p: Point, tick: u64) -> bool {
        self.pool_at(p).is_some() || tick < self.wet_until
    }

    pub fn step(&mut self, f: &Frame) -> Vec<Effect> {
        let mut out = vec![];
        if !f.alive {
            return out;
        }
        self.expire(f, &mut out);
        self.run_cast(f, &mut out);
        self.passive(f, &mut out);
        self.run_whirlpools(f, &mut out);
        self.run_twin(f, &mut out);
        self.run_trident(f, &mut out);
        self.bounce(f, &mut out);
        let order = if f.manual { f.order } else { self.decide(f) };
        if let Some(o) = order {
            self.execute(f, o, &mut out);
        }
        self.hp_log.push((f.tick, f.hp));
        self.hp_log.retain(|(t, _)| f.tick < t + sec(d::TWIN_ENGAGE_WINDOW_TICKS));
        self.last_positions = f.enemy_champions().map(|u| (u.id, u.pos)).collect();
        out
    }

    // ---------------------------------------------------------------- pools and the passive

    fn add_pool(&mut self, f: &Frame, center: Point, radius: i64, whirl: bool, out: &mut Vec<Effect>) -> u32 {
        let id = self.next_pool;
        self.next_pool += 1;
        let until = f.tick + sec(d::WATER_POOL_TICKS);
        let center = center.clamp_map();
        self.pools.push(Pool { id, center, radius, until, whirl });
        out.push(Effect::PoolUp { id, center, radius, until, whirl });
        out.push(Effect::Fx { name: "tls_pool_rise".into(), at: center, ticks: 18, ambient: false });
        id
    }

    fn expire(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        self.pools.retain(|p| {
            let keep = f.tick < p.until;
            if !keep {
                out.push(Effect::PoolDown { id: p.id });
            }
            keep
        });
    }

    fn passive(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        let wet = self.water_at(f.pos, f.tick);
        if wet != self.in_water {
            self.in_water = wet;
            out.push(Effect::Water { on: wet });
        }
        // overheal: only her own unexpired increments count, and never more than the shield she still has (absorbed
        // shielding is gone); each increment lasts the linger plus one refresh, so it holds in water and fades ~2 s after
        self.overheal_parts.retain(|(_, until)| f.tick < *until);
        let own = self.overheal_parts.iter().map(|p| p.0).sum::<i64>().min(f.shield.max(0));
        self.overheal = own;
        // twice a second: half of the per-second regen
        if wet && f.tick % 30 == 0 {
            let amount = f.max_hp * d::WATER_REGEN_PER_MILLE_PER_SECOND / 2000;
            if f.hp < f.max_hp {
                out.push(Effect::Heal { amount });
            } else {
                let cap = f.max_hp * d::WATER_OVERHEAL_CAP_PCT / 100;
                let add = amount.min(cap - own);
                if add > 0 {
                    let ticks = sec(d::WATER_OVERHEAL_LINGER_TICKS) + 30;
                    self.overheal_parts.push((add, f.tick + ticks));
                    self.overheal = own + add;
                    out.push(Effect::Shield { target: f.owner, amount: add, ticks });
                }
            }
        }
    }

    /// Her accepted harp bolts: in water each bounces to a second enemy; the clone echoes them.
    fn bounce(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        let base = pct(d::BASIC_BASE, d::BASIC_AP_RATIO, f.ap);
        for &t in &f.basic_hits {
            let Some(target) = f.units.iter().find(|u| u.id == t && u.hp > 0) else { continue };
            if self.in_water {
                if let Some(next) = f
                    .units
                    .iter()
                    .filter(|u| u.id != t && f.enemy(u) && !u.tower && u.pos.within(target.pos, d::WATER_BOUNCE_RANGE + u.radius))
                    .min_by_key(|u| (!u.champion, u.pos.d2(target.pos), u.id))
                {
                    out.push(Effect::Damage { target: next.id, amount: base * d::WATER_BOUNCE_PCT / 100, from_clone: false });
                    out.push(Effect::Fx { name: "tls_bounce".into(), at: next.pos, ticks: 10, ambient: true });
                }
            }
        }
    }

    // ---------------------------------------------------------------- S1 Tide Pool

    fn start_tide_pool(&mut self, f: &Frame, aim: Point, out: &mut Vec<Effect>) -> bool {
        if !self.ready(0, f.tick) || self.busy(f.tick) {
            return false;
        }
        self.ready[0] = f.tick + sec(d::TIDE_POOL_COOLDOWN_TICKS);
        let aim = f.pos.reach(aim, d::TIDE_POOL_RANGE);
        self.cast = Some(Cast::TidePool { until: f.tick + sec(d::TIDE_POOL_CAST_TICKS), aim });
        out.push(Effect::Pose { name: "skill1", ticks: sec(d::TIDE_POOL_CAST_TICKS) });
        self.last_skill = "tide_pool";
        true
    }

    /// The pool (or wet spot) at `from` surges to `aim` as a whirlpool; a whirlpool forms back at `from`.
    fn surge(&mut self, f: &Frame, from: Point, aim: Point, power: i64, out: &mut Vec<Effect>) {
        let radius = d::WATER_POOL_RADIUS * d::TIDE_POOL_WHIRL_SCALE_PCT / 100;
        if let Some(i) = self.pools.iter().position(|q| q.holds(from)) {
            let old = self.pools.remove(i);
            out.push(Effect::PoolDown { id: old.id });
        }
        let id = self.add_pool(f, aim, radius, true, out);
        out.push(Effect::Fx { name: "tls_surge".into(), at: aim, ticks: 20, ambient: false });
        let center = self.pools.iter().find(|p| p.id == id).map_or(aim, |p| p.center);
        let dmg = pct(d::TIDE_POOL_BASE, d::TIDE_POOL_AP_RATIO, f.ap) * power / 100;
        let caught: Vec<&Unit> = f.units.iter().filter(|u| f.enemy(u) && !u.tower && u.pos.within(center, radius + u.radius)).collect();
        for u in &caught {
            out.push(Effect::Damage { target: u.id, amount: dmg, from_clone: power < 100 });
            out.push(Effect::Slow { target: u.id, percent: d::TIDE_POOL_SLOW_PCT, ticks: 30 });
            out.push(Effect::Pull { target: u.id, to: u.pos.toward(center, (u.pos.dist(center) as i64 / 2).max(1)), ticks: 10 });
        }
        if caught.iter().filter(|u| u.champion).count() >= d::TIDE_POOL_BUMP_MIN_CHAMPIONS as usize {
            out.push(Effect::Fx { name: "tls_bump".into(), at: center, ticks: 20, ambient: false });
            for u in caught.iter().filter(|u| u.champion) {
                out.push(Effect::Stun { target: u.id, ticks: sec(d::TIDE_POOL_BUMP_STUN_TICKS) });
            }
        }
        self.add_pool(f, from, d::WATER_POOL_RADIUS, true, out);
    }

    fn tide_pool_from(&mut self, f: &Frame, at: Point, aim: Point, power: i64, out: &mut Vec<Effect>) {
        if self.water_at(at, f.tick) {
            self.surge(f, at, aim, power, out);
        } else {
            self.add_pool(f, at, d::WATER_POOL_RADIUS, false, out);
        }
    }

    fn run_cast(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        match self.cast {
            Some(Cast::TidePool { until, aim }) if f.tick >= until => {
                self.cast = None;
                let clone_at = f.clone.map(|c| c.pos);
                self.tide_pool_from(f, f.pos, aim, 100, out);
                // the twin echoes it to the same point
                if let (Some(c), Some(_)) = (clone_at, self.twin) {
                    self.tide_pool_from(f, c, aim, d::TWIN_COPY_PCT, out);
                }
            }
            Some(Cast::Trident { until, aim, from_water, target }) if f.tick >= until => {
                self.cast = None;
                let heading = ((aim.y - f.pos.y) as f64).atan2((aim.x - f.pos.x) as f64).to_degrees() * 1000.0;
                let speed = if from_water { d::ULT_SPEED * d::ULT_WATER_SPEED_PCT / 100 } else { d::ULT_SPEED };
                self.trident = Some(Trident { at: f.pos, heading_mdeg: heading.round() as i64, spent_mdeg: 0, speed, travelled: 0, from_water, target, guide: None });
            }
            _ => {}
        }
    }

    /// Whirlpools keep slowing and drawing in the enemies inside them.
    fn run_whirlpools(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        if f.tick % sec(d::TIDE_POOL_PULL_EVERY_TICKS).max(1) != 0 {
            return;
        }
        for p in self.pools.iter().filter(|p| p.whirl) {
            for u in f.units.iter().filter(|u| f.enemy(u) && !u.tower && u.pos.within(p.center, p.radius + u.radius)) {
                out.push(Effect::Slow { target: u.id, percent: d::TIDE_POOL_SLOW_PCT, ticks: sec(d::TIDE_POOL_PULL_EVERY_TICKS) + 2 });
                if !u.pos.within(p.center, 4_000) {
                    out.push(Effect::Pull { target: u.id, to: u.pos.toward(p.center, d::TIDE_POOL_PULL_STEP.min(u.pos.dist(p.center) as i64)), ticks: 4 });
                }
            }
        }
    }

    // ---------------------------------------------------------------- S2 Dive

    fn dive(&mut self, f: &Frame, aim: Point, out: &mut Vec<Effect>) -> bool {
        if !self.ready(1, f.tick) || self.busy(f.tick) {
            return false;
        }
        self.ready[1] = f.tick + sec(d::DIVE_COOLDOWN_TICKS);
        let (destination, travel_ticks) = if self.water_at(f.pos, f.tick) {
            // into one of her own pools (the trident's, anywhere on the map) she dives all the way; otherwise in range
            let to = if self.pool_at(aim).is_some() { free_spot(f, f.pos, aim) } else { free_spot(f, f.pos, f.pos.reach(aim, d::DIVE_RANGE)) };
            if self.pool_at(to).is_none() {
                self.add_pool(f, to, d::WATER_POOL_RADIUS, false, out);
            }
            let ticks = sec(d::DIVE_DIVE_TICKS + f.pos.dist(to) as i64 / 20_000).min(90);
            out.push(Effect::Fx { name: "tls_dive_in".into(), at: f.pos, ticks: 18, ambient: false });
            out.push(Effect::Submerge { ticks });
            out.push(Effect::Glide { to, ticks });
            out.push(Effect::Pose { name: "skill2", ticks });
            self.moving_until = f.tick + ticks;
            self.last_skill = "dive";
            (to, ticks)
        } else {
            let to = free_spot(f, f.pos, f.pos.reach(aim, d::DIVE_SLIDE_RANGE));
            let ticks = sec(d::DIVE_SLIDE_TICKS);
            out.push(Effect::Glide { to, ticks });
            out.push(Effect::Fx { name: "tls_slide".into(), at: f.pos, ticks: 14, ambient: false });
            out.push(Effect::Pose { name: "skill2", ticks });
            self.moving_until = f.tick + ticks;
            self.wet_until = f.tick + ticks + sec(d::WATER_WET_TICKS);
            self.last_skill = "slide";
            (to, ticks)
        };
        // Copy the committed owner destination and clock, including far owned-pool dives.
        // The clone still resolves its own legal landing if its approach is obstructed.
        if let (Some(c), Some(t)) = (f.clone, self.twin) {
            let to = free_spot(f, c.pos, destination);
            let ticks = travel_ticks;
            self.twin = Some(Twin { moving_until: f.tick + ticks, ..t });
            out.push(Effect::CloneMove { to, ticks });
        }
        true
    }

    // ---------------------------------------------------------------- S3 Tide Twin

    fn spawn_twin(&mut self, f: &Frame, aim: Point, out: &mut Vec<Effect>) -> bool {
        if !self.ready(2, f.tick) || self.busy(f.tick) {
            return false;
        }
        self.ready[2] = f.tick + sec(d::TWIN_COOLDOWN_TICKS);
        if self.twin.is_some() {
            out.push(Effect::RemoveClone);
        }
        let at = free_spot(f, f.pos, f.pos.reach(aim, d::TWIN_RANGE));
        let linked = self.pool_at(at).is_some();
        let ticks = sec(d::TWIN_TICKS);
        self.twin = Some(Twin { until: f.tick + ticks, moving_until: 0, linked, swapped: false, rescue: None, next_bolt: f.tick + 12 });
        out.push(Effect::SpawnClone { pos: at, ticks });
        out.push(Effect::Fx { name: "tls_twin_spawn".into(), at, ticks: 20, ambient: false });
        out.push(Effect::Pose { name: "skill3", ticks: 18 });
        if linked {
            out.push(Effect::Log("tide twin spawned on water: tide link".into()));
        }
        self.last_skill = "twin";
        true
    }

    fn end_twin(&mut self, out: &mut Vec<Effect>) {
        if self.twin.take().is_some() {
            out.push(Effect::RemoveClone);
        }
    }

    fn engaged(&self, f: &Frame) -> bool {
        // a heavy hit in the window
        let drop = self.hp_log.first().map_or(0, |(_, hp)| hp - f.hp);
        if drop * 100 >= f.max_hp * d::TWIN_ENGAGE_DROP_PCT {
            return true;
        }
        // an enemy champion dashed onto her (moved far in one tick, landing close)
        f.enemy_champions().any(|u| {
            u.pos.within(f.pos, d::TWIN_ENGAGE_DASH_RANGE + u.radius)
                && self.last_positions.iter().any(|(id, p)| *id == u.id && !p.within(u.pos, 6_000))
        })
    }

    fn swap(&mut self, f: &Frame, out: &mut Vec<Effect>) -> bool {
        let (Some(c), Some(t)) = (f.clone, self.twin) else { return false };
        if !t.linked || t.swapped || t.rescue.is_some() {
            return false;
        }
        self.twin = Some(Twin { swapped: true, ..t });
        out.push(Effect::Swap { owner_to: free_spot(f, f.pos, c.pos), clone_to: f.pos });
        out.push(Effect::Fx { name: "tls_swap".into(), at: f.pos, ticks: 18, ambient: false });
        self.add_pool(f, f.pos, d::WATER_POOL_RADIUS, true, out);
        self.last_skill = "swap";
        true
    }

    fn run_twin(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        let Some(t) = self.twin else { return };
        let Some(c) = f.clone else {
            // killed: it leaves a pool behind
            self.twin = None;
            return;
        };
        if f.tick >= t.until {
            self.end_twin(out);
            return;
        }
        // its body: every tick while it moves (so a flight reads as motion), else every 6 ticks
        let moving = f.tick < t.moving_until;
        if moving || f.tick % 6 == 0 {
            out.push(Effect::Fx { name: format!("tls_twin_p{}", (f.tick / 6) % 8), at: c.pos, ticks: if moving { 1 } else { 6 }, ambient: false });
        }
        // The presentation adapter connects live carrier endpoints every tick.
        // the rescue: arrived?
        if let Some((ally, at)) = t.rescue {
            if f.tick >= at {
                let pos = if ally == f.owner { f.pos } else { f.units.iter().find(|u| u.id == ally).map_or(c.pos, |u| u.pos) };
                self.rescue(f, ally, pos, format!("tide twin rescued {ally}"), out);
            }
            return;
        }
        // a twin cast on water reaches Thalassa herself anywhere on the map, instantly
        if t.linked && d::TWIN_SELF_RESCUE_GLOBAL != 0 && f.stunned && !c.pos.within(f.pos, d::TWIN_RESCUE_RANGE) {
            self.rescue(f, f.owner, f.pos, "tide twin rescued Thalassa from afar".into(), out);
            return;
        }
        // an ally (or she) stunned in range: fly to them
        let stunned_ally = std::iter::once((f.owner, f.pos, f.stunned))
            .chain(f.units.iter().filter(|u| u.champion && u.team == f.team && u.hp > 0).map(|u| (u.id, u.pos, u.stunned)))
            .filter(|(_, p, s)| *s && p.within(c.pos, d::TWIN_RESCUE_RANGE))
            .min_by_key(|(id, p, _)| (p.d2(c.pos), *id));
        if let Some((ally, pos, _)) = stunned_ally {
            let arrive = f.tick + sec(d::TWIN_FLY_TICKS);
            self.twin = Some(Twin { rescue: Some((ally, arrive)), moving_until: arrive, ..t });
            out.push(Effect::CloneMove { to: free_spot(f, c.pos, pos), ticks: sec(d::TWIN_FLY_TICKS) });
            out.push(Effect::Fx { name: "tls_twin_fly".into(), at: c.pos, ticks: 12, ambient: false });
            return;
        }
        // the tide link: engaged on, she swaps with it
        if t.linked && !t.swapped && !f.manual && self.engaged(f) {
            self.swap(f, out);
            return;
        }
        // otherwise it plays on its own: harp bolts and its own footwork (skills only when she casts hers)
        self.twin_play(f, c, out);
    }

    fn rescue(&mut self, f: &Frame, ally: usize, pos: Point, log: String, out: &mut Vec<Effect>) {
        self.add_pool(f, pos, d::WATER_POOL_RADIUS, false, out);
        out.push(Effect::Cleanse { target: ally });
        out.push(Effect::Shield { target: ally, amount: pct(d::TWIN_SHIELD_BASE, d::TWIN_SHIELD_AP_RATIO, f.ap), ticks: sec(d::TWIN_SHIELD_TICKS) });
        out.push(Effect::CcImmune { target: ally, ticks: sec(d::TWIN_IMMUNE_TICKS) });
        out.push(Effect::Fx { name: "tls_rescue".into(), at: pos, ticks: 24, ambient: false });
        out.push(Effect::Log(log));
        self.end_twin(out);
    }

    /// The twin's own target: the nearest enemy champion it can see, or, when no champion is near, a farmable unit.
    fn twin_target<'a>(&self, f: &'a Frame, c: CloneView) -> Option<&'a Unit> {
        let near = |u: &&Unit| f.enemy(u) && !u.tower && u.pos.within(c.pos, d::TWIN_AGGRO_RANGE) && u.pos.within(f.pos, d::TWIN_LEASH + d::TWIN_AGGRO_RANGE);
        f.units
            .iter()
            .filter(|u| u.champion)
            .filter(near)
            .min_by_key(|u| (u.pos.d2(c.pos), u.id))
            .or_else(|| f.units.iter().filter(|u| !u.champion && u.farmable).filter(near).min_by_key(|u| (u.pos.d2(c.pos), u.id)))
    }

    fn twin_play(&mut self, f: &Frame, c: CloneView, out: &mut Vec<Effect>) {
        let Some(mut t) = self.twin else { return };
        let wet = self.water_at(c.pos, f.tick);
        let range = d::BASIC_RANGE * (100 + if wet { d::WATER_RANGE_PCT } else { 0 }) / 100;
        let target = self.twin_target(f, c);
        // harp bolts on its own clock, at copyPct of hers
        if let Some(u) = target {
            if f.tick >= t.next_bolt && c.pos.within(u.pos, range + u.radius) {
                t.next_bolt = f.tick + sec(d::BASIC_COOLDOWN_TICKS);
                out.push(Effect::Damage { target: u.id, amount: pct(d::BASIC_BASE, d::BASIC_AP_RATIO, f.ap) * d::TWIN_COPY_PCT / 100, from_clone: true });
                out.push(Effect::Fx { name: "tls_bounce".into(), at: u.pos, ticks: 10, ambient: true });
            }
        }
        // footwork every 6 ticks, unless a flight or a dive is still carrying it
        if f.tick >= t.moving_until && f.tick % 6 == 0 {
            let step = d::TWIN_WALK_SPEED * (100 + if wet { d::WATER_MOVE_SPEED_PCT } else { 0 }) / 100 * 6;
            let goal = match target {
                Some(u) => {
                    let gap = c.pos.dist(u.pos) as i64 - u.radius;
                    if gap > range {
                        c.pos.toward(u.pos, step.min(gap - range * 9 / 10))
                    } else if gap < range * d::TWIN_KITE_PCT / 100 {
                        c.pos.toward(u.pos, -step)
                    } else {
                        c.pos
                    }
                }
                None if !c.pos.within(f.pos, 20_000) => c.pos.toward(f.pos, step.min(c.pos.dist(f.pos) as i64 - 20_000)),
                None => c.pos,
            };
            // never past the leash
            let goal = if goal.within(f.pos, d::TWIN_LEASH) { goal } else { f.pos.reach(goal, d::TWIN_LEASH) };
            let to = free_spot(f, c.pos, goal);
            if !to.within(c.pos, 500) {
                t.moving_until = f.tick + 6;
                out.push(Effect::CloneMove { to, ticks: 6 });
            }
        }
        self.twin = Some(t);
    }

    // ---------------------------------------------------------------- Ult: Trident of the Deep

    fn throw(&mut self, f: &Frame, aim: Point, out: &mut Vec<Effect>) -> bool {
        if !f.ult_unlocked || !self.ready(3, f.tick) || self.busy(f.tick) || self.trident.is_some() {
            return false;
        }
        self.ready[3] = f.tick + sec(d::ULT_COOLDOWN_TICKS);
        let from_water = self.water_at(f.pos, f.tick);
        let target = f.enemy_champions().filter(|u| u.pos.within(aim, 20_000 + u.radius)).min_by_key(|u| (u.pos.d2(aim), u.id)).map(|u| u.id);
        self.cast = Some(Cast::Trident { until: f.tick + sec(d::ULT_WINDUP_TICKS), aim, from_water, target });
        out.push(Effect::Pose { name: "ult", ticks: sec(d::ULT_WINDUP_TICKS) });
        out.push(Effect::Log(format!("trident thrown at {},{} (from water: {from_water})", aim.x, aim.y)));
        self.last_skill = "ult";
        true
    }

    fn run_trident(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        let Some(mut t) = self.trident else { return };
        // steer toward the guide (a recast) or the chosen target, within the 25 degree budget
        let guide = t.guide.or_else(|| t.target.and_then(|id| f.units.iter().find(|u| u.id == id && f.enemy(u)).map(|u| u.pos)));
        if let Some(g) = guide {
            let want = ((g.y - t.at.y) as f64).atan2((g.x - t.at.x) as f64).to_degrees() * 1000.0;
            let delta = (want.round() as i64 - t.heading_mdeg + 180_000).rem_euclid(360_000) - 180_000;
            let budget = (d::ULT_CURVE_MILLI_DEGREES - t.spent_mdeg).max(0).min(d::ULT_CURVE_RATE_MILLI_DEGREES);
            let turn = delta.clamp(-budget, budget);
            t.heading_mdeg += turn;
            t.spent_mdeg += turn.abs();
        }
        let rad = (t.heading_mdeg as f64 / 1000.0).to_radians();
        let next = Point::new(t.at.x + (rad.cos() * t.speed as f64).round() as i64, t.at.y + (rad.sin() * t.speed as f64).round() as i64);
        // the first enemy champion in the swept step
        let hit = f
            .enemy_champions()
            .filter(|u| seg_d2(u.pos, t.at, next) <= ((d::ULT_HIT_RADIUS + u.radius) as f64).powi(2))
            .min_by_key(|u| (u.pos.d2(t.at), u.id));
        out.push(Effect::Fx { name: format!("tls_trident_p{}", f.tick % 8), at: next, ticks: 1, ambient: false });
        if let Some(u) = hit {
            let stun = if t.from_water { d::ULT_WATER_STUN_TICKS } else { d::ULT_STUN_TICKS };
            out.push(Effect::TridentHit { target: u.id, at: u.pos, ticks: sec(stun) });
            let dmg = pct(d::ULT_BASE, d::ULT_AP_RATIO, f.ap);
            for e in f.units.iter().filter(|e| f.enemy(e) && !e.tower && e.pos.within(u.pos, d::ULT_SPLASH_RADIUS + e.radius)) {
                out.push(Effect::Damage { target: e.id, amount: dmg, from_clone: false });
            }
            out.push(Effect::Fx { name: "tls_trident_hit".into(), at: u.pos, ticks: 24, ambient: false });
            self.land(f, u.pos, t.from_water, out);
            return;
        }
        t.travelled += t.speed;
        if !next.on_map() || t.travelled >= d::ULT_MAX_DISTANCE {
            self.land(f, t.at.clamp_map(), t.from_water, out);
            return;
        }
        t.at = next;
        self.trident = Some(t);
    }

    fn land(&mut self, f: &Frame, at: Point, from_water: bool, out: &mut Vec<Effect>) {
        self.trident = None;
        out.push(Effect::Fx { name: "tls_trident_land".into(), at, ticks: 24, ambient: false });
        let radius = if from_water { d::WATER_POOL_RADIUS * d::TIDE_POOL_WHIRL_SCALE_PCT / 100 } else { d::WATER_POOL_RADIUS };
        self.add_pool(f, at, radius, from_water, out);
    }

    // ---------------------------------------------------------------- orders

    fn execute(&mut self, f: &Frame, o: Order, out: &mut Vec<Effect>) {
        let ok = match o {
            Order::TidePool(p) => self.start_tide_pool(f, p, out),
            Order::Dive(p) => self.dive(f, p, out),
            Order::Twin(p) => self.spawn_twin(f, p, out),
            Order::Ult(p) => self.throw(f, p, out),
            Order::Steer(p) => match self.trident.as_mut() {
                Some(t) => {
                    t.guide = Some(p);
                    t.target = None;
                    true
                }
                None => false,
            },
            Order::Swap => self.swap(f, out),
        };
        if !ok {
            self.rejection = format!("{o:?} rejected at tick {}", f.tick);
        }
    }

    // ---------------------------------------------------------------- AI

    fn decide(&self, f: &Frame) -> Option<Order> {
        if f.stunned || self.busy(f.tick) {
            return None;
        }
        let low = f.hp * 100 <= f.max_hp * 35;
        let wet = self.water_at(f.pos, f.tick);
        let near: Vec<&Unit> = f.enemy_champions().filter(|u| u.pos.within(f.pos, 90_000)).collect();
        // escape: dive or slide away when someone is on her
        if self.ready(1, f.tick) && low {
            if let Some(t) = near.iter().filter(|u| u.pos.within(f.pos, 30_000)).min_by_key(|u| u.pos.d2(f.pos)) {
                let away = Point::new(f.pos.x * 2 - t.pos.x, f.pos.y * 2 - t.pos.y);
                return Some(Order::Dive(f.pos.toward(away, d::DIVE_RANGE).clamp_map()));
            }
        }
        if let Some(o) = self.decide_ult(f, wet) {
            return Some(o);
        }
        // the twin at the edge of a fight (on water if she can) when 2+ enemies are close or an ally is pressed
        if self.ready(2, f.tick) && self.twin.is_none() && near.len() >= 2 {
            let spot = self.pools.iter().filter(|p| p.center.within(f.pos, d::TWIN_RANGE)).min_by_key(|p| p.center.d2(f.pos)).map(|p| p.center);
            return Some(Order::Twin(spot.unwrap_or_else(|| f.pos.toward(near[0].pos, -10_000))));
        }
        if self.ready(0, f.tick) {
            if wet {
                // surge onto the spot that catches the most champions (2+ for the bump)
                let r = d::WATER_POOL_RADIUS * d::TIDE_POOL_WHIRL_SCALE_PCT / 100;
                let best = near
                    .iter()
                    .filter(|u| u.pos.within(f.pos, d::TIDE_POOL_RANGE))
                    .map(|u| (near.iter().filter(|o| o.pos.within(u.pos, r)).count(), u))
                    .max_by_key(|(n, u)| (*n, std::cmp::Reverse(u.id)));
                if let Some((n, u)) = best {
                    if n >= 2 || f.profile(PROFILE_AGGRESSIVE) >= 40 {
                        return Some(Order::TidePool(u.pos));
                    }
                }
            } else if !near.is_empty() {
                return Some(Order::TidePool(f.pos));
            }
        }
        // follow up into the trident's pool with a dive
        if self.ready(1, f.tick) && wet && !low {
            if let Some(p) = self.pools.iter().filter(|p| !p.holds(f.pos)).filter(|p| f.enemy_champions().any(|u| p.holds(u.pos))).min_by_key(|p| (p.center.d2(f.pos), p.id)) {
                return Some(Order::Dive(p.center));
            }
        }
        self.decide_farm(f, wet)
    }

    fn decide_ult(&self, f: &Frame, wet: bool) -> Option<Order> {
        if !f.ult_unlocked || !self.ready(3, f.tick) || self.trident.is_some() {
            return None;
        }
        // prefer to throw from water when a pool is a second away
        if !wet && self.ready(0, f.tick) {
            return None;
        }
        let skill = f.profile(PROFILE_SKILL_HIT);
        let target = f
            .enemy_champions()
            .map(|u| {
                let grouped = f.enemy_champions().filter(|o| o.pos.within(u.pos, 40_000)).count() as i64;
                let low = (100 - u.hp * 100 / u.max_hp.max(1)).max(0);
                (grouped * 40 + low, u)
            })
            .filter(|(score, _)| *score >= 70)
            .max_by_key(|(score, u)| (*score, std::cmp::Reverse(u.id)))?
            .1;
        let travel = f.pos.dist(target.pos) as i64 / d::ULT_SPEED.max(1);
        let lead = Point::new(target.pos.x + target.velocity.x * travel, target.pos.y + target.velocity.y * travel);
        let err = (100 - skill) * 100;
        Some(Order::Ult(Point::new(lead.x + jitter(f.tick, 3, err * 2 + 1) - err, lead.y + jitter(f.tick, 4, err * 2 + 1) - err).clamp_map()))
    }

    fn decide_farm(&self, f: &Frame, wet: bool) -> Option<Order> {
        if !self.ready(0, f.tick) || f.enemy_champions().any(|u| u.pos.within(f.pos, 70_000)) {
            return None;
        }
        let farm: Vec<&Unit> = f.units.iter().filter(|u| u.farmable && !u.champion && u.pos.within(f.pos, d::TIDE_POOL_RANGE)).collect();
        let r = d::WATER_POOL_RADIUS * d::TIDE_POOL_WHIRL_SCALE_PCT / 100;
        let (n, u) = farm.iter().map(|u| (farm.iter().filter(|o| o.pos.within(u.pos, r)).count(), u)).max_by_key(|(n, u)| (*n, std::cmp::Reverse(u.id)))?;
        if n >= 3 || u.max_hp >= 1_500 {
            return Some(Order::TidePool(if wet { u.pos } else { f.pos }));
        }
        None
    }
}

/// `to`, or the nearest point back toward `from` (2,000 at a time) that's on the map and off terrain.
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

pub fn seg_d2(p: Point, a: Point, b: Point) -> f64 {
    let (vx, vy) = ((b.x - a.x) as f64, (b.y - a.y) as f64);
    let l2 = vx * vx + vy * vy;
    let k = if l2 == 0.0 { 0.0 } else { (((p.x - a.x) as f64 * vx + (p.y - a.y) as f64 * vy) / l2).clamp(0.0, 1.0) };
    let (dx, dy) = (p.x as f64 - (a.x as f64 + vx * k), p.y as f64 - (a.y as f64 + vy * k));
    dx * dx + dy * dy
}
