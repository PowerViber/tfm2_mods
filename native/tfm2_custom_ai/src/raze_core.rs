//! Raze: deterministic purchases, combat, momentum physics and thirteen-attribute decisions.
//! No host calls, floating point, global mutable state or combat RNG.
use crate::raze_data::*;
use crate::valorant_guns::Gun;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const PROFILE_NAMES: [&str; 13] = [
    "Ego",
    "Judgement",
    "Positioning",
    "Control Speed",
    "Skill Accuracy",
    "Skill Dodge",
    "Aggressive",
    "Order",
    "Roaming",
    "Macro",
    "Micro",
    "Map Awareness",
    "Strategy",
];
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Point {
    pub x: i64,
    pub y: i64,
}
impl Point {
    pub fn new(x: i64, y: i64) -> Self {
        Self { x, y }
    }
    pub fn add(self, p: Self) -> Self {
        Self::new(self.x + p.x, self.y + p.y)
    }
    pub fn sub(self, p: Self) -> Self {
        Self::new(self.x - p.x, self.y - p.y)
    }
    pub fn scale(self, n: i64, d: i64) -> Self {
        Self::new(self.x * n / d, self.y * n / d)
    }
    pub fn length(self) -> i64 {
        ((self.x as i128 * self.x as i128 + self.y as i128 * self.y as i128) as u128).isqrt() as i64
    }
    pub fn toward(self, to: Self, n: i64) -> Self {
        let d = to.sub(self);
        self.add(d.scale(n, d.length().max(1)))
    }
    pub fn capped(self, n: i64) -> Self {
        let l = self.length();
        if l > n {
            self.scale(n, l)
        } else {
            self
        }
    }
}
pub fn distance(a: Point, b: Point) -> i64 {
    a.sub(b).length()
}
fn hash(mut x: u64) -> u64 {
    x ^= x >> 30;
    x = x.wrapping_mul(0xbf58476d1ce4e5b9);
    x ^= x >> 27;
    x = x.wrapping_mul(0x94d049bb133111eb);
    x ^ (x >> 31)
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Profile {
    pub values: [i64; 13],
    pub known: bool,
}
impl Default for Profile {
    fn default() -> Self {
        Self {
            values: [50; 13],
            known: false,
        }
    }
}
impl Profile {
    pub fn get(&self, n: usize) -> i64 {
        self.values[n].clamp(0, 100)
    }
    pub fn reaction(&self) -> u64 {
        (24 - self.get(3) * 18 / 100) as u64
    }
    pub fn steering_ticks(&self) -> u64 {
        (12 - self.get(10) * 9 / 100) as u64
    }
    pub fn horizon(&self) -> usize {
        1 + (2 * self.get(12) / 100) as usize
    }
    pub fn timing_error(&self, seed: u64, event: u64) -> i64 {
        let n = 6 * (100 - self.get(10)) / 100;
        if n == 0 {
            0
        } else {
            (hash(seed ^ event.wrapping_mul(41)) % (2 * n + 1) as u64) as i64 - n
        }
    }
    pub fn estimation_error(&self, seed: u64, event: u64) -> i64 {
        let n = 20 * (100 - self.get(1)) / 100;
        if n == 0 {
            0
        } else {
            (hash(seed ^ event.wrapping_mul(97)) % (2 * n + 1) as u64) as i64 - n
        }
    }
    pub fn aim(&self, from: Point, u: &Unit, travel: u64, seed: u64, event: u64) -> Point {
        let lead = if u.controlled {
            0
        } else {
            30 + self.get(4) * 70 / 100
        };
        let n = 6000 * (100 - self.get(4)) / 100;
        let noise = |salt: u64| {
            if n == 0 {
                0
            } else {
                (hash(seed ^ event.wrapping_mul(131) ^ salt) % (2 * n + 1) as u64) as i64 - n
            }
        };
        let p = u
            .pos
            .add(u.velocity.scale(travel as i64 * lead, TPS * 100))
            .add(Point::new(noise(1), noise(2)));
        // A prediction never changes the projectile's committed route after release.
        from.toward(p, distance(from, p)).bounded()
    }
}
impl Point {
    pub fn bounded(self) -> Self {
        Self::new(
            self.x.clamp(BODY_RADIUS, MAP_SIZE - BODY_RADIUS),
            self.y.clamp(BODY_RADIUS, MAP_SIZE - BODY_RADIUS),
        )
    }
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Unit {
    pub id: usize,
    pub team: usize,
    pub pos: Point,
    pub velocity: Point,
    pub hp: i64,
    pub max_hp: i64,
    pub ap: i64,
    pub attack: i64,
    pub visible: bool,
    pub targetable: bool,
    pub kind: u8,
    pub farmable: bool,
    pub controlled: bool,
    pub melee: bool,
}
impl Unit {
    pub fn champion(&self) -> bool {
        self.kind == 0
    }
    pub fn alive(&self) -> bool {
        self.hp > 0
    }
    pub fn damageable(&self) -> bool {
        self.targetable && self.kind != 4 && self.alive()
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Rect {
    pub min: Point,
    pub max: Point,
}
impl Rect {
    fn contains(&self, p: Point, r: i64) -> bool {
        p.x >= self.min.x - r
            && p.x <= self.max.x + r
            && p.y >= self.min.y - r
            && p.y <= self.max.y + r
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Hazard {
    pub id: u64,
    pub pos: Point,
    pub radius: i64,
    pub until: u64,
    pub severity: i64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UtilityStatus {
    pub id: u64,
    pub hp: i64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreditEvent {
    pub id: u64,
    pub kind: u8,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Action {
    Buy,
    Grenade { aim: Point },
    Pack { aim: Point },
    Detonate { id: u64 },
    Bot { aim: Point },
    EquipRocket,
    FireRocket { aim: Point },
    Steer { input: Point },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Frame {
    pub version: u32,
    pub tick: u64,
    pub owner: usize,
    pub seed: u64,
    pub automatic: bool,
    pub profile: Profile,
    pub action: Option<Action>,
    pub units: Vec<Unit>,
    pub blocks: Vec<Rect>,
    pub hazards: Vec<Hazard>,
    pub utility: Vec<UtilityStatus>,
    pub credits: Vec<CreditEvent>,
    pub unlocked: [bool; 3],
    pub stunned: bool,
    pub silenced: bool,
    pub bound: bool,
    pub farm_window: bool,
    pub buy_allowed: bool,
    pub at_spawn: bool,
    pub shield: i64,
    pub assignment: Option<Point>,
    pub objective: Option<usize>,
    pub missed_wave: i64,
    pub intent: Option<Point>,
    pub practice_inventory: bool,
    pub haste: i64,
    pub ult_haste: i64,
}
impl Default for Frame {
    fn default() -> Self {
        Self {
            version: 1,
            tick: 0,
            owner: 1,
            seed: 1,
            automatic: false,
            profile: Profile::default(),
            action: None,
            units: vec![],
            blocks: vec![],
            hazards: vec![],
            utility: vec![],
            credits: vec![],
            unlocked: [true; 3],
            stunned: false,
            silenced: false,
            bound: false,
            farm_window: true,
            buy_allowed: false,
            at_spawn: false,
            shield: 0,
            assignment: None,
            objective: None,
            missed_wave: 0,
            intent: None,
            practice_inventory: false,
            haste: 0,
            ult_haste: 0,
        }
    }
}
impl Frame {
    pub fn me(&self) -> Option<&Unit> {
        self.units.iter().find(|u| u.id == self.owner)
    }
    pub fn blocked(&self, p: Point, r: i64) -> bool {
        p.x < r
            || p.y < r
            || p.x > MAP_SIZE - r
            || p.y > MAP_SIZE - r
            || self.blocks.iter().any(|b| b.contains(p, r))
    }
    pub fn sweep(&self, from: Point, to: Point, r: i64) -> (Point, bool) {
        let n = (distance(from, to) / 1000 + 1).min(2000);
        let mut prev = from;
        for k in 1..=n {
            let p = from.add(to.sub(from).scale(k, n));
            if self.blocked(p, r) {
                return (prev, true);
            }
            prev = p;
        }
        (to, false)
    }
    pub fn visible_enemies<'a>(&'a self, me: &'a Unit) -> impl Iterator<Item = &'a Unit> {
        self.units
            .iter()
            .filter(move |u| u.team != me.team && u.visible && u.damageable())
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Cmd {
    Damage {
        id: u64,
        target: usize,
        raw: i64,
        self_hit: bool,
    },
    Push {
        target: usize,
        to: Point,
    },
    MoveOwner {
        to: Point,
    },
    Guide {
        to: Point,
    },
    Spawn {
        id: u64,
        bot: bool,
        pos: Point,
        hp: i64,
        ticks: u64,
    },
    MoveUnit {
        id: u64,
        to: Point,
    },
    Remove {
        id: u64,
    },
    Warning {
        id: u64,
        tag: String,
        pos: Point,
        radius: i64,
        until: u64,
    },
    Visual {
        id: u64,
        tag: String,
        pos: Point,
        height: i64,
        until: u64,
    },
    Gun {
        index: usize,
    },
    Armor {
        amount: i64,
    },
    Buy {
        spent: i64,
    },
    Rejected {
        reason: String,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Pack {
    pub id: u64,
    pub pos: Point,
    pub born: u64,
    pub until: u64,
    pub hp: i64,
    pub ready: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Bot {
    pub id: u64,
    pub pos: Point,
    pub direction: Point,
    pub until: u64,
    pub hp: i64,
    pub ready: bool,
    pub target: Option<usize>,
    pub fuse: Option<u64>,
    pub ap: i64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Flight {
    pub velocity: Point,
    pub remainder: Point,
    pub born: u64,
    pub until: u64,
    pub input: Point,
    pub next_input: u64,
    pub goal: Point,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Cast {
    pub action: Action,
    pub release: u64,
    pub ap: i64,
    pub max_hp: i64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Grenade {
    pub id: u64,
    pub from: Point,
    pub aim: Point,
    pub arrival: u64,
    pub explosion: u64,
    pub cluster: u64,
    pub born: u64,
    pub ap: i64,
    pub main_done: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Rocket {
    pub id: u64,
    pub pos: Point,
    pub direction: Point,
    pub remaining: i64,
    pub ap: i64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Sighting {
    pub pos: Point,
    pub velocity: Point,
    pub tick: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlannedStep {
    pub at: u64,
    pub action: Action,
    pub relative: bool,
    pub intended: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Plan {
    pub family: String,
    pub target: Option<usize>,
    pub landing: Point,
    pub steps: Vec<PlannedStep>,
    pub score: i64,
    pub reason: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Candidate {
    pub family: String,
    pub target: Option<usize>,
    pub landing: Point,
    pub score: i64,
    pub immediate: i64,
    pub follow_up: i64,
    pub legal: bool,
    pub reason: String,
    pub aim: Point,
    pub line: Vec<String>,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Metrics {
    pub requested_damage: i64,
    pub casts: u64,
    pub launches: u64,
    pub collisions: u64,
    pub wasted_preparation: u64,
    pub spent: i64,
    pub grenade_hits: u64,
    pub rocket_hits: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Core {
    pub credits: i64,
    pub gun: usize,
    pub armor: usize,
    pub stock: [u8; 3],
    pub cooldowns: [u64; 3],
    pub packs: Vec<Pack>,
    pub bot: Option<Bot>,
    pub flight: Option<Flight>,
    pub cast: Option<Cast>,
    pub grenades: Vec<Grenade>,
    pub rocket: Option<Rocket>,
    pub armed_until: u64,
    pub profile: Profile,
    pub plan: Option<Plan>,
    pub candidates: Vec<Candidate>,
    pub continuation: Vec<String>,
    pub reaction_until: u64,
    pub reaction_events: u64,
    pub evaluated: usize,
    pub horizon: usize,
    pub last_rejection: String,
    pub intended_detonation: Option<u64>,
    pub actual_detonation: Option<u64>,
    pub metrics: Metrics,
    pub sightings: BTreeMap<usize, Sighting>,
    pub last_tick: Option<u64>,
    pub alive_since: u64,
    pub paid_alive: u64,
    pub last_buy: Option<u64>,
    pub next_id: u64,
    pub decision: u64,
    pub next_think: u64,
    pub observed_situation: u64,
    pub seen_credits: BTreeSet<u64>,
    pub credit_floor: u64,
    pub was_alive: bool,
}
impl Default for Core {
    fn default() -> Self {
        Self {
            credits: START_CREDITS,
            gun: 0,
            armor: 0,
            stock: [0; 3],
            cooldowns: [0; 3],
            packs: vec![],
            bot: None,
            flight: None,
            cast: None,
            grenades: vec![],
            rocket: None,
            armed_until: 0,
            profile: Profile::default(),
            plan: None,
            candidates: vec![],
            continuation: vec![],
            reaction_until: 0,
            reaction_events: 0,
            evaluated: 0,
            horizon: 1,
            last_rejection: String::new(),
            intended_detonation: None,
            actual_detonation: None,
            metrics: Metrics::default(),
            sightings: BTreeMap::new(),
            last_tick: None,
            alive_since: 0,
            paid_alive: 0,
            last_buy: None,
            next_id: 1,
            decision: 0,
            next_think: 0,
            observed_situation: 0,
            seen_credits: BTreeSet::new(),
            credit_floor: 0,
            was_alive: false,
        }
    }
}
impl Core {
    fn id(&mut self) -> u64 {
        let n = self.next_id;
        self.next_id += 1;
        n
    }
    fn reject(&mut self, out: &mut Vec<Cmd>, s: &str) {
        self.last_rejection = s.into();
        out.push(Cmd::Rejected { reason: s.into() });
    }
    fn earn(&mut self, n: i64) {
        self.credits = (self.credits + n).clamp(0, MAX_CREDITS)
    }
    pub fn died(&mut self) -> Vec<Cmd> {
        let mut out = vec![];
        for p in self.packs.drain(..) {
            out.push(Cmd::Remove { id: p.id })
        }
        if let Some(b) = self.bot.take() {
            out.push(Cmd::Remove { id: b.id })
        }
        self.flight = None;
        self.cast = None;
        self.plan = None;
        self.grenades.clear();
        self.rocket = None;
        self.armed_until = 0;
        self.armor = 0;
        self.was_alive = false;
        self.last_buy = None;
        self.sightings.clear();
        out
    }
    fn buy(&mut self, f: &Frame, me: &Unit, out: &mut Vec<Cmd>) {
        if !f.buy_allowed || self.flight.is_some() || self.cast.is_some() {
            self.reject(out, "Buying requires the same safe location as Omen");
            return;
        }
        let before = self.credits;
        let old = Gun::from_index(self.gun);
        let close = f
            .units
            .iter()
            .filter(|u| u.champion() && u.team != me.team && u.melee)
            .count();
        let gun = Gun::upgrade(self.credits, close, old);
        if gun != old {
            self.credits -= gun.price();
            self.gun = gun.idx();
            out.push(Cmd::Gun { index: self.gun });
        }
        let spare = if gun.tier() >= Gun::Vandal.tier() {
            self.credits
        } else {
            self.credits - Gun::Vandal.price()
        };
        let level = if spare >= 1000 {
            2
        } else if spare >= 400 {
            1
        } else {
            0
        };
        let amount = me.max_hp * if self.armor.max(1) == 2 { 25 } else { 12 } / 100;
        if level > 0 && (self.armor < level || f.shield < amount / 2) {
            self.credits -= if level == 2 { 1000 } else { 400 };
            self.armor = level;
            out.push(Cmd::Armor {
                amount: me.max_hp * if level == 2 { 25 } else { 12 } / 100,
            });
        }
        for (n, max, price) in [(0, 2, PACK_PRICE), (1, 1, GRENADE_PRICE), (2, 1, BOT_PRICE)] {
            while self.stock[n] < max && self.credits >= price {
                self.credits -= price;
                self.stock[n] += 1;
            }
        }
        self.last_buy = Some(f.tick);
        if before != self.credits {
            let spent = before - self.credits;
            self.metrics.spent += spent;
            out.push(Cmd::Buy { spent });
        }
    }
    pub fn step(&mut self, f: &Frame) -> Vec<Cmd> {
        let mut out = vec![];
        if self.last_tick.is_some_and(|t| f.tick <= t) {
            return out;
        }
        if self.last_tick.is_none() && f.practice_inventory {
            self.stock = [2, 1, 1];
        }
        self.last_tick = Some(f.tick);
        let Some(me) = f.me().cloned() else {
            return out;
        };
        for e in &f.credits {
            if e.id > self.credit_floor && self.seen_credits.insert(e.id) {
                self.earn(match e.kind {
                    0 => KILL_CREDITS,
                    1 => ASSIST_CREDITS,
                    2 => TEAM_KILL_CREDITS,
                    3 => DEATH_CREDITS,
                    _ => 0,
                });
            }
        }
        while self.seen_credits.len() > 256 {
            let k = *self.seen_credits.first().unwrap();
            self.seen_credits.remove(&k);
            self.credit_floor = self.credit_floor.max(k);
        }
        if !me.alive() {
            if self.was_alive {
                self.earn(DEATH_CREDITS);
                out.extend(self.died());
            }
            return out;
        }
        if !self.was_alive {
            self.was_alive = true;
            self.alive_since = f.tick;
            self.paid_alive = 0;
            self.observed_situation = 0;
        }
        let paid = f.tick.saturating_sub(self.alive_since) / ALIVE_TICKS as u64;
        if paid > self.paid_alive {
            self.earn((paid - self.paid_alive) as i64 * ALIVE_CREDITS);
            self.paid_alive = paid;
        }
        self.profile = f.profile.clone();
        self.horizon = self.profile.horizon();
        if f.automatic {
            self.observe_situation(f, &me);
        }
        for u in f
            .units
            .iter()
            .filter(|u| u.visible && u.alive() && u.team != me.team && u.champion())
        {
            self.sightings.insert(
                u.id,
                Sighting {
                    pos: u.pos,
                    velocity: u.velocity,
                    tick: f.tick,
                },
            );
        }
        let memory = (120 + self.profile.get(11) * 480 / 100) as u64;
        self.sightings
            .retain(|_, s| f.tick.saturating_sub(s.tick) <= memory);
        for s in &f.utility {
            if let Some(n) = self.packs.iter().position(|p| p.id == s.id) {
                if s.hp < 0 {
                    self.packs.remove(n);
                    self.stock[0] = (self.stock[0] + 1).min(2);
                    self.reject(&mut out, "Satchel carrier spawn failed; stock refunded");
                } else {
                    self.packs[n].ready = true;
                    self.packs[n].hp = s.hp;
                }
            }
            if self.bot.as_ref().is_some_and(|b| b.id == s.id) {
                if s.hp < 0 {
                    self.bot = None;
                    self.stock[2] = 1;
                    self.cooldowns[1] = 0;
                    self.reject(
                        &mut out,
                        "Boom Bot carrier spawn failed; stock and cooldown restored",
                    );
                } else if let Some(b) = self.bot.as_mut() {
                    b.ready = true;
                    b.hp = s.hp;
                }
            }
        }
        self.packs.retain(|p| {
            let keep = p.hp > 0 && f.tick < p.until;
            if !keep {
                out.push(Cmd::Remove { id: p.id });
            }
            keep
        });
        if self
            .bot
            .as_ref()
            .is_some_and(|b| b.hp <= 0 || f.tick >= b.until)
        {
            if let Some(b) = self.bot.take() {
                out.push(Cmd::Remove { id: b.id });
            }
        }
        if (f.stunned || f.silenced) && self.cast.take_if(|c| f.tick < c.release).is_some() {
            self.metrics.wasted_preparation += 1;
            self.plan = None;
            self.reject(
                &mut out,
                "Preparation interrupted; committed stock and cooldown are spent",
            );
        }
        if f.bound || f.stunned {
            self.flight = None;
        }
        if f.automatic
            && self.cast.is_none()
            && self.flight.is_none()
            && f.tick % 20 == 0
            && f.buy_allowed
            && self
                .last_buy
                .map_or(true, |t| f.tick >= t + if f.at_spawn { 120 } else { 300 })
        {
            self.buy(f, &me, &mut out);
        }
        if let Some(c) = self.cast.take() {
            if f.tick >= c.release {
                if f.stunned || f.silenced {
                    self.metrics.wasted_preparation += 1;
                    self.reject(&mut out, "Preparation interrupted");
                } else {
                    self.release(f, &me, c, &mut out);
                }
            } else {
                self.cast = Some(c)
            }
        }
        self.advance_grenades(f, &me, &mut out);
        self.advance_rocket(f, &me, &mut out);
        self.advance_bot(f, &me, &mut out);
        if f.automatic {
            self.think(f, &me, &mut out);
        } else if let Some(a) = f.action.clone() {
            self.accept(f, &me, a, &mut out);
        }
        self.advance_flight(f, &me, &mut out);
        out
    }
    fn accept(&mut self, f: &Frame, me: &Unit, a: Action, out: &mut Vec<Cmd>) -> bool {
        self.last_rejection.clear();
        if let Action::Buy = a {
            self.buy(f, me, out);
            return true;
        }
        if let Action::Steer { input } = a {
            if f.stunned {
                self.reject(out, "Stunned: steering unavailable");
                return false;
            }
            if let Some(v) = self.flight.as_mut() {
                v.input = input.capped(1000);
                return true;
            }
            self.reject(out, "No active satchel flight");
            return false;
        }
        if f.stunned || f.silenced {
            self.reject(out, "Control prevents a new ability");
            return false;
        }
        if let Action::Detonate { id } = a {
            return self.detonate(f, me, id, out);
        }
        if self.cast.is_some() {
            self.reject(out, "Another preparation is active");
            return false;
        }
        let prep = match &a {
            Action::Grenade { aim } => {
                if !f.unlocked[0] || self.stock[1] == 0 || f.tick < self.cooldowns[0] {
                    self.reject(out, "Grenade locked, unpurchased or cooling down");
                    return false;
                }
                if distance(me.pos, *aim) > GRENADE_RANGE || f.blocked(*aim, 0) {
                    self.reject(out, "Invalid grenade landing");
                    return false;
                }
                self.stock[1] -= 1;
                self.cooldowns[0] =
                    f.tick + (GRENADE_COOLDOWN * 100 / (100 + f.haste.clamp(-90, 900))) as u64;
                GRENADE_PREP
            }
            Action::Pack { aim } => {
                if !f.unlocked[1] || self.stock[0] == 0 || self.packs.len() >= 2 {
                    self.reject(
                        out,
                        "Satchel locked, unpurchased or deployment limit reached",
                    );
                    return false;
                }
                if distance(me.pos, *aim) > PACK_RANGE
                    || f.blocked(*aim, 1000)
                    || f.sweep(me.pos, *aim, 1000).1
                {
                    self.reject(out, "Satchel throw is obstructed");
                    return false;
                }
                self.stock[0] -= 1;
                PACK_THROW
            }
            Action::Bot { aim } => {
                if !f.unlocked[0]
                    || self.stock[2] == 0
                    || f.tick < self.cooldowns[1]
                    || self.bot.is_some()
                {
                    self.reject(out, "Bot locked, unpurchased, active or cooling down");
                    return false;
                }
                if f.blocked(me.pos, 1000) || distance(me.pos, *aim) < 1000 {
                    self.reject(out, "Invalid bot direction");
                    return false;
                }
                self.stock[2] -= 1;
                self.cooldowns[1] =
                    f.tick + (BOT_COOLDOWN * 100 / (100 + f.haste.clamp(-90, 900))) as u64;
                BOT_PREP
            }
            Action::EquipRocket => {
                if !f.unlocked[2] || f.tick < self.cooldowns[2] || f.tick < self.armed_until {
                    self.reject(out, "Ultimate locked, armed or cooling down");
                    return false;
                }
                self.cooldowns[2] =
                    f.tick + (ROCKET_COOLDOWN * 100 / (100 + f.ult_haste.clamp(-90, 900))) as u64;
                ROCKET_PREP
            }
            Action::FireRocket { aim } => {
                if self.armed_until <= f.tick
                    || self.rocket.is_some()
                    || distance(me.pos, *aim) < 1000
                {
                    self.reject(out, "No armed rocket or invalid firing direction");
                    return false;
                }
                self.armed_until = 0;
                self.release_rocket(f, me, *aim, out);
                self.metrics.casts += 1;
                return true;
            }
            _ => return false,
        };
        self.metrics.casts += 1;
        let id = self.id();
        let (tag, p, r) = match &a {
            Action::Grenade { aim } => ("grenade_prep", *aim, GRENADE_RADIUS),
            Action::Pack { aim } => ("pack_throw", *aim, PACK_RADIUS),
            Action::Bot { .. } => ("bot_prep", me.pos, 0),
            _ => ("rocket_equip", me.pos, 0),
        };
        out.push(Cmd::Warning {
            id,
            tag: tag.into(),
            pos: p,
            radius: r,
            until: f.tick + prep as u64,
        });
        self.cast = Some(Cast {
            action: a,
            release: f.tick + prep as u64,
            ap: me.ap,
            max_hp: me.max_hp,
        });
        true
    }
    fn release(&mut self, f: &Frame, me: &Unit, c: Cast, out: &mut Vec<Cmd>) {
        let id = self.id();
        match c.action {
            Action::Pack { aim } => {
                if distance(me.pos, aim) > PACK_RANGE || f.blocked(aim, 1000) {
                    self.reject(out, "Satchel landing invalidated before release");
                    return;
                }
                self.packs.push(Pack {
                    id,
                    pos: aim,
                    born: f.tick,
                    until: f.tick + PACK_LIFETIME as u64,
                    hp: PACK_HP,
                    ready: false,
                });
                out.push(Cmd::Spawn {
                    id,
                    bot: false,
                    pos: aim,
                    hp: PACK_HP,
                    ticks: PACK_LIFETIME as u64,
                });
            }
            Action::Grenade { aim } => {
                let travel = (distance(me.pos, aim) * TPS / GRENADE_SPEED).max(1) as u64;
                let arrival = f.tick + travel;
                self.grenades.push(Grenade {
                    id,
                    from: me.pos,
                    aim,
                    arrival,
                    explosion: arrival + GRENADE_FUSE as u64,
                    cluster: arrival + (GRENADE_FUSE + CLUSTER_FUSE) as u64,
                    born: f.tick,
                    ap: c.ap,
                    main_done: false,
                });
                out.push(Cmd::Warning {
                    id,
                    tag: "grenade_warning".into(),
                    pos: aim,
                    radius: GRENADE_RADIUS,
                    until: arrival + GRENADE_FUSE as u64,
                });
            }
            Action::Bot { aim } => {
                let direction = aim.sub(me.pos).capped(1000);
                self.bot = Some(Bot {
                    id,
                    pos: me.pos,
                    direction,
                    until: f.tick + BOT_LIFETIME as u64,
                    hp: c.max_hp / 4,
                    ready: false,
                    target: None,
                    fuse: None,
                    ap: c.ap,
                });
                out.push(Cmd::Spawn {
                    id,
                    bot: true,
                    pos: me.pos,
                    hp: c.max_hp / 4,
                    ticks: BOT_LIFETIME as u64,
                });
            }
            Action::EquipRocket => {
                self.armed_until = f.tick + ROCKET_LIFETIME as u64;
                out.push(Cmd::Visual {
                    id,
                    tag: "rocket_ready".into(),
                    pos: me.pos,
                    height: 0,
                    until: self.armed_until,
                });
            }
            _ => {}
        }
    }
    fn detonate(&mut self, f: &Frame, me: &Unit, id: u64, out: &mut Vec<Cmd>) -> bool {
        let selected = if id == 0 {
            self.packs.iter().find(|p| p.ready).map(|p| p.id)
        } else {
            Some(id)
        };
        let Some(n) = self
            .packs
            .iter()
            .position(|p| Some(p.id) == selected && p.ready && p.hp > 0)
        else {
            self.reject(out, "Satchel is missing, destroyed or awaiting its carrier");
            return false;
        };
        let p = self.packs.remove(n);
        out.push(Cmd::Remove { id: p.id });
        out.push(Cmd::Visual {
            id: p.id,
            tag: "satchel_blast".into(),
            pos: p.pos,
            height: 0,
            until: f.tick + 12,
        });
        for u in f.units.iter().filter(|u| {
            u.team != me.team && u.alive() && u.targetable && distance(u.pos, p.pos) <= PACK_RADIUS
        }) {
            if f.tick >= p.born + PACK_ARM as u64 {
                self.damage(p.id, u.id, PACK_DAMAGE + me.ap * PACK_AP / 100, false, out);
            }
            let to = u.pos.toward(p.pos, -PACK_PUSH);
            let (to, _) = f.sweep(u.pos, to, BODY_RADIUS);
            out.push(Cmd::Push { target: u.id, to });
        }
        if distance(me.pos, p.pos) <= PACK_RADIUS && !f.bound {
            let dir = if distance(me.pos, p.pos) < 500 {
                self.plan
                    .as_ref()
                    .map(|p| p.landing.sub(me.pos))
                    .filter(|d| d.length() > 0)
                    .unwrap_or(Point::new(1000, 0))
            } else {
                me.pos.sub(p.pos)
            };
            let impulse = dir.scale(LAUNCH_SPEED, dir.length().max(1));
            let old = self.flight.take();
            let born = old.as_ref().map_or(f.tick, |v| v.born);
            let velocity = old
                .as_ref()
                .map_or(impulse, |v| v.velocity.add(impulse))
                .capped(MAX_SPEED);
            let until = (f.tick + FLIGHT_TICKS as u64).min(born + MAX_FLIGHT_TICKS as u64);
            let goal = self
                .plan
                .as_ref()
                .map_or(me.pos.add(velocity.scale(FLIGHT_TICKS, TPS)), |p| p.landing);
            self.flight = Some(Flight {
                velocity,
                remainder: Point::default(),
                born,
                until,
                input: Point::default(),
                next_input: f.tick,
                goal,
            });
            self.metrics.launches += 1;
        }
        self.actual_detonation = Some(f.tick);
        true
    }
    fn advance_flight(&mut self, f: &Frame, me: &Unit, out: &mut Vec<Cmd>) {
        let Some(mut v) = self.flight.take() else {
            return;
        };
        if f.tick >= v.until || f.bound {
            return;
        }
        if f.automatic && !f.stunned && f.tick >= v.next_input {
            let mut goal = f.intent.unwrap_or(v.goal);
            if f.tick >= self.reaction_until {
                let forecast = me.pos.add(v.velocity.scale(8, TPS));
                if f.hazards.iter().any(|h| {
                    h.until > f.tick
                        && distance(h.pos, forecast) < h.radius + BODY_RADIUS
                        && hash(f.seed ^ h.id ^ me.id as u64) % 100
                            < (50 + f.profile.get(5)).min(100) as u64
                }) {
                    let side = Point::new(-v.velocity.y, v.velocity.x).capped(50000);
                    goal = [me.pos.add(side), me.pos.sub(side)]
                        .into_iter()
                        .filter(|p| !f.sweep(me.pos, *p, BODY_RADIUS).1)
                        .min_by_key(|p| self.danger(f, me, *p) * 1000 + distance(*p, v.goal) / 100)
                        .unwrap_or(goal);
                }
            }
            let delta = goal.sub(me.pos);
            v.input = delta.capped(1000);
            v.next_input = f.tick + f.profile.steering_ticks();
        }
        if !f.stunned {
            let speed = v.velocity.length().max(1);
            let tangent = Point::new(-v.velocity.y, v.velocity.x);
            let sideways = (v.input.x as i128 * tangent.x as i128
                + v.input.y as i128 * tangent.y as i128)
                / speed as i128;
            let accel = tangent.scale(AIR_ACCEL * sideways as i64 / (1000 * TPS), speed);
            v.velocity = v.velocity.add(accel).capped(MAX_SPEED);
            if (v.input.x as i128 * v.velocity.x as i128 + v.input.y as i128 * v.velocity.y as i128)
                < 0
            {
                v.velocity = v.velocity.scale(96, 100);
            }
        }
        v.remainder = v.remainder.add(v.velocity);
        let step = v.remainder.scale(1, TPS);
        v.remainder = v.remainder.sub(step.scale(TPS, 1));
        let (to, hit) = f.sweep(me.pos, me.pos.add(step), BODY_RADIUS);
        out.push(Cmd::MoveOwner { to });
        let elapsed = f.tick - v.born;
        let total = (v.until - v.born).max(1);
        let height = (64000 * elapsed * (total - elapsed) / (total * total)) as i64;
        out.push(Cmd::Visual {
            id: 0,
            tag: "airborne".into(),
            pos: to,
            height,
            until: f.tick + 2,
        });
        if hit {
            self.metrics.collisions += 1;
            out.push(Cmd::Visual {
                id: 0,
                tag: "landing".into(),
                pos: to,
                height: 0,
                until: f.tick + 8,
            });
        } else {
            self.flight = Some(v)
        }
    }
    fn damage(&mut self, id: u64, target: usize, raw: i64, self_hit: bool, out: &mut Vec<Cmd>) {
        if raw > 0 {
            if !self_hit {
                self.metrics.requested_damage += raw;
            }
            out.push(Cmd::Damage {
                id,
                target,
                raw,
                self_hit,
            });
        }
    }
    fn blast(
        &mut self,
        f: &Frame,
        me: &Unit,
        id: u64,
        p: Point,
        r: i64,
        raw: i64,
        self_damage: bool,
        falloff: bool,
        out: &mut Vec<Cmd>,
    ) {
        // Impact uses current targetable entities, including concealed enemies already caught by an area attack.
        for u in f
            .units
            .iter()
            .filter(|u| u.team != me.team && u.damageable())
        {
            let d = distance(p, u.pos);
            if d <= r {
                let pct = if falloff && d > ROCKET_INNER {
                    100 - (d - ROCKET_INNER) * 60 / (r - ROCKET_INNER)
                } else {
                    100
                };
                self.damage(id, u.id, raw * pct / 100, false, out);
            }
        }
        if self_damage && distance(p, me.pos) <= r {
            let d = distance(p, me.pos);
            let pct = if falloff && d > ROCKET_INNER {
                100 - (d - ROCKET_INNER) * 60 / (r - ROCKET_INNER)
            } else {
                100
            };
            self.damage(
                id,
                me.id,
                raw * pct / 100 * SELF_DAMAGE_PERCENT / 100,
                true,
                out,
            );
        }
        out.push(Cmd::Visual {
            id,
            tag: if falloff {
                "rocket_blast"
            } else {
                "grenade_blast"
            }
            .into(),
            pos: p,
            height: 0,
            until: f.tick + 18,
        });
    }
    fn advance_grenades(&mut self, f: &Frame, me: &Unit, out: &mut Vec<Cmd>) {
        let gs = std::mem::take(&mut self.grenades);
        for mut g in gs {
            if f.tick < g.arrival {
                let n = f.tick - g.born;
                let d = (g.arrival - g.born).max(1);
                let p = g.from.add(g.aim.sub(g.from).scale(n as i64, d as i64));
                out.push(Cmd::Visual {
                    id: g.id,
                    tag: "grenade_flight".into(),
                    pos: p,
                    height: (32000 * n * (d - n) / (d * d)) as i64,
                    until: f.tick + 2,
                });
            }
            if f.tick >= g.explosion && !g.main_done {
                self.blast(
                    f,
                    me,
                    g.id,
                    g.aim,
                    GRENADE_RADIUS,
                    GRENADE_DAMAGE + g.ap * GRENADE_AP / 100,
                    true,
                    false,
                    out,
                );
                g.main_done = true;
                for d in [
                    Point::new(10000, 10000),
                    Point::new(-10000, 10000),
                    Point::new(-10000, -10000),
                    Point::new(10000, -10000),
                ] {
                    let (p, _) = f.sweep(g.aim, g.aim.add(d), 1000);
                    out.push(Cmd::Warning {
                        id: g.id,
                        tag: "cluster_warning".into(),
                        pos: p,
                        radius: CLUSTER_RADIUS,
                        until: g.cluster,
                    });
                }
            }
            if f.tick >= g.cluster {
                let centres = [
                    Point::new(10000, 10000),
                    Point::new(-10000, 10000),
                    Point::new(-10000, -10000),
                    Point::new(10000, -10000),
                ]
                .map(|d| f.sweep(g.aim, g.aim.add(d), 1000).0);
                let raw = CLUSTER_DAMAGE + g.ap * CLUSTER_AP_NUM / CLUSTER_AP_DEN;
                for u in f
                    .units
                    .iter()
                    .filter(|u| u.team != me.team && u.damageable())
                {
                    let n = centres
                        .iter()
                        .filter(|p| distance(**p, u.pos) <= CLUSTER_RADIUS)
                        .count()
                        .min(2);
                    if n > 0 {
                        self.damage(g.id, u.id, raw * n as i64, false, out);
                        self.metrics.grenade_hits += 1;
                    }
                }
                let n = centres
                    .iter()
                    .filter(|p| distance(**p, me.pos) <= CLUSTER_RADIUS)
                    .count()
                    .min(2);
                if n > 0 {
                    self.damage(
                        g.id,
                        me.id,
                        raw * n as i64 * SELF_DAMAGE_PERCENT / 100,
                        true,
                        out,
                    );
                }
                for p in centres {
                    out.push(Cmd::Visual {
                        id: g.id,
                        tag: "cluster_blast".into(),
                        pos: p,
                        height: 0,
                        until: f.tick + 12,
                    });
                }
            } else {
                self.grenades.push(g)
            }
        }
    }
    fn release_rocket(&mut self, f: &Frame, me: &Unit, aim: Point, out: &mut Vec<Cmd>) {
        let id = self.id();
        let delta = aim.sub(me.pos);
        let direction = delta.scale(1000000, delta.length().max(1));
        self.rocket = Some(Rocket {
            id,
            pos: me.pos,
            direction,
            remaining: ROCKET_RANGE,
            ap: me.ap,
        });
        out.push(Cmd::Visual {
            id,
            tag: "rocket_release".into(),
            pos: me.pos,
            height: 0,
            until: f.tick + 12,
        });
        if let Some(v) = self.flight.as_mut() {
            v.velocity = v
                .velocity
                .sub(direction.scale(ROCKET_RECOIL, 1000000))
                .capped(MAX_SPEED);
        }
    }
    fn advance_rocket(&mut self, f: &Frame, me: &Unit, out: &mut Vec<Cmd>) {
        let Some(mut r) = self.rocket.take() else {
            return;
        };
        let travel = (ROCKET_SPEED / TPS).min(r.remaining);
        let end = r.pos.add(r.direction.scale(travel, 1000000));
        let (wall, blocked) = f.sweep(r.pos, end, 1000);
        let hit = f
            .units
            .iter()
            .filter(|u| u.team != me.team && u.alive() && u.targetable)
            .filter_map(|u| {
                let d = end.sub(r.pos);
                let den = d.x as i128 * d.x as i128 + d.y as i128 * d.y as i128;
                let q = u.pos.sub(r.pos);
                let t =
                    (q.x as i128 * d.x as i128 + q.y as i128 * d.y as i128).clamp(0, den.max(1));
                let p = r.pos.add(d.scale(t as i64, den.max(1) as i64));
                (distance(p, u.pos) <= 8000 && distance(r.pos, p) <= distance(r.pos, wall))
                    .then_some((t, u.id, p))
            })
            .min_by_key(|(t, id, _)| (*t, *id));
        r.remaining -= travel;
        let impact = hit
            .map(|(_, _, p)| p)
            .or_else(|| (blocked || r.remaining <= 0).then_some(wall));
        if let Some(p) = impact {
            self.blast(
                f,
                me,
                r.id,
                p,
                ROCKET_RADIUS,
                ROCKET_DAMAGE + r.ap * ROCKET_AP / 100,
                true,
                true,
                out,
            );
            self.metrics.rocket_hits += f
                .units
                .iter()
                .filter(|u| {
                    u.team != me.team
                        && u.alive()
                        && u.targetable
                        && distance(u.pos, p) <= ROCKET_RADIUS
                })
                .count() as u64;
        } else {
            r.pos = end;
            out.push(Cmd::Visual {
                id: r.id,
                tag: "rocket_flight".into(),
                pos: end,
                height: 0,
                until: f.tick + 2,
            });
            self.rocket = Some(r);
        }
    }
    fn advance_bot(&mut self, f: &Frame, me: &Unit, out: &mut Vec<Cmd>) {
        let Some(mut b) = self.bot.take() else { return };
        if !b.ready {
            self.bot = Some(b);
            return;
        }
        let eligible = |u: &&Unit| {
            u.team != me.team
                && u.visible
                && u.damageable()
                && distance(u.pos, b.pos) <= BOT_SIGHT
                && !f.sweep(b.pos, u.pos, 1000).1
        };
        let target = f
            .units
            .iter()
            .filter(eligible)
            .filter(|u| {
                let d = u.pos.sub(b.pos);
                b.target == Some(u.id)
                    || d.x as i128 * b.direction.x as i128 + d.y as i128 * b.direction.y as i128
                        >= d.length() as i128 * 700
            })
            .min_by_key(|u| (distance(u.pos, b.pos), u.id));
        b.target = target.map(|u| u.id);
        if let Some(u) = target {
            if distance(u.pos, b.pos) <= BOT_TRIGGER {
                if b.fuse.is_none() {
                    b.fuse = Some(f.tick + BOT_FUSE as u64);
                    out.push(Cmd::Warning {
                        id: b.id,
                        tag: "bot_warning".into(),
                        pos: b.pos,
                        radius: BOT_RADIUS,
                        until: f.tick + BOT_FUSE as u64,
                    });
                }
            } else if b.fuse.is_none() {
                b.direction = u.pos.sub(b.pos).capped(1000);
            }
        }
        if b.fuse.is_some_and(|t| f.tick >= t) {
            self.blast(
                f,
                me,
                b.id,
                b.pos,
                BOT_RADIUS,
                BOT_DAMAGE + b.ap * BOT_AP / 100,
                false,
                false,
                out,
            );
            out.push(Cmd::Remove { id: b.id });
            return;
        }
        if b.fuse.is_none() {
            let speed = if b.target.is_some() {
                BOT_CHASE_SPEED
            } else {
                BOT_SPEED
            };
            let next = b.pos.add(b.direction.scale(speed, 1000 * TPS));
            let (to, hit) = f.sweep(b.pos, next, 2000);
            if hit {
                let x = Point::new(next.x, b.pos.y);
                if f.blocked(x, 2000) {
                    b.direction.x = -b.direction.x
                } else {
                    b.direction.y = -b.direction.y
                }
            }
            b.pos = to;
            out.push(Cmd::MoveUnit {
                id: b.id,
                to: b.pos,
            });
        }
        self.bot = Some(b);
    }
    fn danger(&self, f: &Frame, me: &Unit, p: Point) -> i64 {
        let mut danger = 0;
        for u in f.visible_enemies(me).filter(|u| u.champion()) {
            let d = distance(u.pos, p);
            if d < 100000 {
                danger += (100000 - d) * 40 / 100000;
            }
        }
        for h in f
            .hazards
            .iter()
            .filter(|h| h.until > f.tick && distance(h.pos, p) < h.radius + BODY_RADIUS)
        {
            let notices =
                hash(f.seed ^ h.id ^ me.id as u64) % 100 < (50 + f.profile.get(5)).min(100) as u64;
            if notices {
                danger += h.severity;
            }
        }
        for (id, s) in &self.sightings {
            if f.units.iter().any(|u| u.id == *id && u.visible) {
                continue;
            }
            let age = f.tick.saturating_sub(s.tick) as i64;
            let uncertainty = age * 1500;
            let d = distance(s.pos, p);
            if d < 40000 + uncertainty {
                danger += f.profile.get(11) * (40000 + uncertainty - d).min(40000) / 100000;
            }
        }
        danger + (100 - me.hp * 100 / me.max_hp.max(1)).max(0) / 3
    }
    pub fn evaluate(&self, f: &Frame, me: &Unit) -> Vec<Candidate> {
        let p = &f.profile;
        let gun = Gun::from_index(self.gun);
        let mut all = vec![];
        let mut enemies: Vec<_> = f
            .visible_enemies(me)
            .filter(|u| u.champion() || (u.farmable && f.farm_window))
            .collect();
        enemies.sort_by_key(|u| {
            (
                if Some(u.id) == f.objective {
                    0
                } else if u.champion() {
                    1
                } else {
                    2
                },
                distance(me.pos, u.pos),
                u.id,
            )
        });
        enemies.truncate(4);
        let allies = f
            .units
            .iter()
            .filter(|u| {
                u.team == me.team && u.id != me.id && u.alive() && distance(u.pos, me.pos) < 150000
            })
            .count() as i64;
        for u in enemies {
            let dist = distance(me.pos, u.pos);
            let champion = u.champion();
            let farm = !champion;
            let pressure = if champion {
                28 + if u.controlled { 10 } else { 0 }
            } else {
                12
            };
            let aim = p.aim(
                me.pos,
                u,
                (dist * TPS / GRENADE_SPEED + GRENADE_PREP) as u64,
                f.seed,
                self.decision ^ u.id as u64,
            );
            let cluster = f
                .visible_enemies(me)
                .filter(|v| distance(v.pos, aim) < GRENADE_RADIUS)
                .count() as i64;
            let is_wave = u.kind == 1;
            let farm_value = if farm {
                p.get(9) * (cluster * 4 + if Some(u.id) == f.objective { 18 } else { 0 }) / 100
            } else {
                0
            };
            let useful_clear = !farm || cluster >= 3 || u.kind == 2 || u.kind == 3;
            let target_low = 100 - u.hp * 100 / u.max_hp.max(1);
            let target_value = pressure + target_low / 5;
            let base = me
                .pos
                .toward(
                    u.pos,
                    (dist - gun.reach() * (60 + p.get(2) / 4) / 100).clamp(20000, 110000),
                )
                .bounded();
            let axis = u.pos.sub(me.pos).capped(1000);
            let side = Point::new(-axis.y, axis.x).scale(24000, 1000);
            let landing = [base, base.add(side).bounded(), base.sub(side).bounded()]
                .into_iter()
                .filter(|q| !f.sweep(me.pos, *q, BODY_RADIUS).1)
                .min_by_key(|q| {
                    self.danger(f, me, *q) * (50 + p.get(2))
                        + distance(*q, base) * (100 - p.get(2)) / 1000
                })
                .unwrap_or(base);
            let escape = if let Some(a) = f
                .units
                .iter()
                .filter(|a| a.team == me.team && a.id != me.id && a.champion() && a.alive())
                .min_by_key(|a| (distance(me.pos, a.pos), a.id))
            {
                a.pos
            } else {
                me.pos.toward(u.pos, -75000).bounded()
            };
            let near_allies = f
                .units
                .iter()
                .filter(|a| {
                    a.team == me.team
                        && a.id != me.id
                        && a.alive()
                        && distance(a.pos, landing) < 90000
                })
                .count() as i64;
            for family in [
                "shoot",
                "grenade",
                "bot",
                "single_launch",
                "double_launch",
                "rocket",
                "rocket_entry",
                "escape",
            ] {
                let entry = family == "single_launch"
                    || family == "double_launch"
                    || family == "rocket_entry";
                let danger = self.danger(
                    f,
                    me,
                    if family == "escape" {
                        escape
                    } else if entry {
                        landing
                    } else {
                        me.pos
                    },
                );
                let mut benefit = match family {
                    "shoot" => target_value + if dist <= gun.reach() { 10 } else { -20 },
                    "grenade" => target_value + cluster * 8 + farm_value - 8,
                    "bot" => target_value + if dist > 50000 { 0 } else { 8 } - 12,
                    "single_launch" => target_value + if dist > gun.reach() { 18 } else { -12 },
                    "double_launch" => target_value + if dist > 85000 { 25 } else { -20 },
                    "rocket" => target_value + cluster * 18 + target_low / 3 - 22,
                    "rocket_entry" => target_value + cluster * 18 + target_low / 3 - 12,
                    "escape" => {
                        self.danger(f, me, me.pos) * 2 + (40 - me.hp * 100 / me.max_hp.max(1)) * 2
                    }
                    _ => 0,
                };
                if entry {
                    benefit = benefit * (75 + p.get(6) / 2) / 100;
                    benefit += (near_allies * 10 - 15) * p.get(7) / 100;
                    benefit += p.get(2) * (40 - self.danger(f, me, landing)) / 100;
                }
                if farm {
                    benefit += farm_value;
                    if is_wave {
                        benefit += p.get(9) / 5;
                    }
                    if entry || family == "rocket" {
                        benefit -= 30 + p.get(9) / 3;
                    }
                }
                if family == "escape" {
                    benefit += ((100 - p.get(0)) * 25 / 100) - 35;
                }
                if family == "bot" {
                    benefit += p.get(2) / 8;
                }
                if (family == "rocket" || family == "rocket_entry") && champion {
                    benefit += p.get(12) * cluster / 10;
                    if distance(if entry { landing } else { me.pos }, aim) < ROCKET_RADIUS {
                        benefit -= (ROCKET_DAMAGE + me.ap * ROCKET_AP / 100) * SELF_DAMAGE_PERCENT
                            / me.max_hp.max(1);
                    }
                }
                let follow = 0;
                let cost = match family {
                    "grenade" => {
                        if farm && cluster < 3 {
                            p.get(9) / 4
                        } else {
                            0
                        }
                    }
                    "double_launch" => 8 + if allies == 0 { p.get(1) / 8 } else { 0 },
                    "rocket" => {
                        if cluster < 2 {
                            p.get(12) / 5
                        } else {
                            0
                        }
                    }
                    _ => 0,
                };
                let rocket_ready = f.tick >= self.cooldowns[2] || self.armed_until > f.tick;
                let (blocked_at, blocked) =
                    f.sweep(if entry { landing } else { me.pos }, aim, 1000);
                let firing_line = !blocked || distance(blocked_at, u.pos) <= ROCKET_RADIUS;
                let legal = match family {
                    "shoot" => true,
                    "grenade" => {
                        f.unlocked[0]
                            && self.stock[1] > 0
                            && f.tick >= self.cooldowns[0]
                            && dist <= GRENADE_RANGE
                            && useful_clear
                    }
                    "bot" => {
                        f.unlocked[0]
                            && self.stock[2] > 0
                            && self.bot.is_none()
                            && f.tick >= self.cooldowns[1]
                    }
                    "single_launch" | "escape" => {
                        f.unlocked[1] && self.stock[0] > 0 && !f.bound && champion
                    }
                    "double_launch" => f.unlocked[1] && self.stock[0] >= 2 && !f.bound && champion,
                    "rocket" => {
                        f.unlocked[2]
                            && rocket_ready
                            && dist <= ROCKET_RANGE
                            && firing_line
                            && (champion || u.kind == 3 && f.objective == Some(u.id))
                    }
                    "rocket_entry" => {
                        f.unlocked[1]
                            && f.unlocked[2]
                            && self.stock[0] >= 2
                            && !f.bound
                            && champion
                            && rocket_ready
                            && dist <= ROCKET_RANGE
                            && firing_line
                    }
                    _ => false,
                };
                let pos = if family == "escape" {
                    escape
                } else if entry {
                    landing
                } else {
                    me.pos
                };
                let clear = !entry || !f.sweep(me.pos, pos, BODY_RADIUS).1;
                let error = p.estimation_error(
                    f.seed,
                    self.decision
                        ^ (u.id as u64).wrapping_mul(31)
                        ^ family
                            .bytes()
                            .fold(0u64, |a, b| a.wrapping_mul(31) + b as u64),
                );
                let score = benefit + follow - cost - danger * (160 - p.get(0)) / 100 + error;
                let aim = if family == "rocket" || family == "rocket_entry" {
                    p.aim(
                        me.pos,
                        u,
                        (dist * TPS / ROCKET_SPEED + ROCKET_PREP + if entry { 30 } else { 0 })
                            as u64,
                        f.seed,
                        self.decision ^ u.id as u64,
                    )
                } else {
                    aim
                };
                all.push(Candidate{family:family.into(),target:Some(u.id),landing:pos,score,immediate:benefit-cost,follow_up:follow,legal:legal&&clear,reason:format!("{family}: value {benefit}, danger {danger}, resource cost {cost}, estimation {error}"),aim,line:vec![family.into()]});
            }
        }
        if let Some(goal) = f.assignment {
            let danger = self.danger(f, me, goal);
            let score = 12 + p.get(8) / 3 + p.get(7) / 5 + p.get(12) / 8
                - f.missed_wave * p.get(9) / 100
                - danger * (160 - p.get(0)) / 100;
            all.push(Candidate {
                family: "rotate".into(),
                target: None,
                landing: goal,
                score,
                immediate: score,
                follow_up: 0,
                legal: distance(me.pos, goal) > 20000,
                reason: format!(
                    "Team assignment: roaming {}, order {}, missed-wave cost {}",
                    p.get(8),
                    p.get(7),
                    f.missed_wave * p.get(9) / 100
                ),
                aim: goal,
                line: vec!["rotate".into()],
            });
        }
        if let Some(h) = f
            .hazards
            .iter()
            .filter(|h| {
                h.until > f.tick
                    && distance(h.pos, me.pos) < h.radius + 20000
                    && hash(f.seed ^ h.id ^ me.id as u64) % 100 < (50 + p.get(5)).min(100) as u64
            })
            .min_by_key(|h| h.id)
        {
            let away = if distance(me.pos, h.pos) > 0 {
                me.pos.sub(h.pos)
            } else {
                Point::new(1000, 0)
            };
            let goal = [
                away,
                Point::new(-away.y, away.x),
                Point::new(away.y, -away.x),
            ]
            .map(|d| {
                me.pos
                    .add(d.scale(h.radius + 12000, d.length().max(1)))
                    .bounded()
            })
            .into_iter()
            .filter(|q| !f.sweep(me.pos, *q, BODY_RADIUS).1)
            .min_by_key(|q| self.danger(f, me, *q) * 1000 + distance(*q, me.pos) / 100);
            if let Some(goal) = goal {
                let score = self.danger(f, me, me.pos) * 2
                    - self.danger(f, me, goal)
                    - 10
                    - if distance(goal, me.pos) / 1020 > h.until.saturating_sub(f.tick) as i64 {
                        60
                    } else {
                        0
                    };
                all.push(Candidate {
                    family: "evade".into(),
                    target: None,
                    landing: goal,
                    score,
                    immediate: score,
                    follow_up: 0,
                    legal: true,
                    reason: "Visible warning: compare ground strafe with paid escape".into(),
                    aim: goal,
                    line: vec!["evade".into()],
                });
            }
        }
        let retreat_hp = 15 + (50 - p.get(0)).max(0) * 4 / 5;
        if me.hp * 100 / me.max_hp.max(1) <= retreat_hp {
            if let Some(enemy) = f
                .visible_enemies(me)
                .filter(|u| u.champion() && distance(u.pos, me.pos) < 75000)
                .min_by_key(|u| (distance(u.pos, me.pos), u.id))
            {
                let goal = f
                    .units
                    .iter()
                    .filter(|u| {
                        u.team == me.team
                            && u.id != me.id
                            && u.champion()
                            && u.alive()
                            && self.danger(f, me, u.pos) < self.danger(f, me, me.pos)
                    })
                    .min_by_key(|u| (distance(u.pos, me.pos), u.id))
                    .map_or_else(|| me.pos.toward(enemy.pos, -75000).bounded(), |u| u.pos);
                all.push(Candidate {
                    family: "retreat".into(),
                    target: None,
                    landing: goal,
                    score: 100 + self.danger(f, me, me.pos) - self.danger(f, me, goal),
                    immediate: 100,
                    follow_up: 0,
                    legal: !f.blocked(goal, BODY_RADIUS),
                    reason: "Low health: preserve life even when both satchels are spent".into(),
                    aim: goal,
                    line: vec!["retreat".into()],
                });
            }
        }
        all.truncate(PLANNER_CANDIDATES as usize);
        all.sort_by(|a, b| {
            b.legal
                .cmp(&a.legal)
                .then(b.score.cmp(&a.score))
                .then(a.family.cmp(&b.family))
                .then(a.target.cmp(&b.target))
        });
        all
    }
    fn make_plan(&mut self, f: &Frame, me: &Unit, c: &Candidate, start: u64) -> Plan {
        let mut steps = vec![];
        let dir = c.landing.sub(me.pos).capped(1000);
        let pack = dir.scale(-10000, 1000);
        let mut at = start;
        let mut add = |at: u64, a: Action, relative: bool, intended: u64| {
            steps.push(PlannedStep {
                at,
                action: a,
                relative,
                intended,
            })
        };
        match c.family.as_str() {
            "grenade" => add(at, Action::Grenade { aim: c.aim }, false, at),
            "bot" => add(at, Action::Bot { aim: c.aim }, false, at),
            "rocket" => {
                if self.armed_until <= at {
                    add(at, Action::EquipRocket, false, at);
                    at += ROCKET_PREP as u64 + 1;
                }
                add(at, Action::FireRocket { aim: c.aim }, false, at);
            }
            "rocket_entry" => {
                if self.armed_until <= at {
                    add(at, Action::EquipRocket, false, at);
                    at += ROCKET_PREP as u64 + 1;
                }
                for n in 0..2 {
                    add(at, Action::Pack { aim: pack }, true, at);
                    at += PACK_THROW as u64 + 2;
                    let intended = at;
                    at = (at as i64 + f.profile.timing_error(f.seed, self.decision + n))
                        .max(at as i64 - 1) as u64;
                    add(at, Action::Detonate { id: 0 }, false, intended);
                    at += 6;
                }
                add(at, Action::FireRocket { aim: c.aim }, false, at);
            }
            "single_launch" | "double_launch" | "escape" => {
                add(at, Action::Pack { aim: pack }, true, at);
                at += PACK_THROW as u64 + 2;
                let intended = at;
                let error = f.profile.timing_error(f.seed, self.decision);
                at = (at as i64 + error).max(start as i64 + PACK_THROW + 1) as u64;
                add(at, Action::Detonate { id: 0 }, false, intended);
                if c.family == "double_launch" {
                    at += 6;
                    add(at, Action::Pack { aim: pack }, true, at);
                    at += PACK_THROW as u64 + 2;
                    let intended = at;
                    at = (at as i64 + f.profile.timing_error(f.seed, self.decision + 1))
                        .max(at as i64 - 1) as u64;
                    add(at, Action::Detonate { id: 0 }, false, intended);
                }
                if self.armed_until > at {
                    add(at + 1, Action::FireRocket { aim: c.aim }, false, at + 1);
                }
            }
            _ => {}
        }
        Plan {
            family: c.family.clone(),
            target: c.target,
            landing: c.landing,
            steps,
            score: c.score,
            reason: c.reason.clone(),
        }
    }
    fn observe_situation(&mut self, f: &Frame, me: &Unit) {
        let enemy = f
            .visible_enemies(me)
            .filter(|u| u.champion())
            .min_by_key(|u| (distance(me.pos, u.pos), u.id));
        let hazard = f
            .hazards
            .iter()
            .filter(|h| h.until > f.tick && distance(h.pos, me.pos) < h.radius + 40000)
            .min_by_key(|h| h.id)
            .map_or(0, |h| h.id);
        let situation = hash(
            enemy.map_or(0, |u| (u.id as u64 + 1) ^ ((u.controlled as u64) << 40))
                ^ hazard.wrapping_mul(17)
                ^ if me.hp * 100 / me.max_hp.max(1) < 35 {
                    0x100000
                } else {
                    0
                }
                ^ f.objective.unwrap_or(0) as u64,
        );
        if situation != self.observed_situation {
            self.observed_situation = situation;
            self.reaction_until = f.tick + f.profile.reaction();
            self.reaction_events += 1;
        }
    }
    fn think(&mut self, f: &Frame, me: &Unit, out: &mut Vec<Cmd>) {
        if let Some(p) = &self.plan {
            if p.target.is_some_and(|id| {
                !f.units
                    .iter()
                    .any(|u| u.id == id && u.visible && u.alive() && u.targetable)
            }) {
                self.plan = None;
                self.continuation.clear();
            }
        }
        if let Some(mut p) = self.plan.take() {
            if let Some(step) = p.steps.first().cloned() {
                if f.tick >= step.at && self.cast.is_none() {
                    p.steps.remove(0);
                    let mut action = if step.relative {
                        match step.action {
                            Action::Pack { aim } => {
                                let lead = self.flight.as_ref().map_or(Point::default(), |v| {
                                    v.velocity.scale(PACK_THROW + 2, TPS)
                                });
                                Action::Pack {
                                    aim: me.pos.add(aim).add(lead).bounded(),
                                }
                            }
                            a => a,
                        }
                    } else {
                        step.action
                    };
                    if let Some(u) = p.target.and_then(|id| {
                        f.units
                            .iter()
                            .find(|u| u.id == id && u.visible && u.alive())
                    }) {
                        let (speed, prep) = match action {
                            Action::Grenade { .. } => (GRENADE_SPEED, GRENADE_PREP + GRENADE_FUSE),
                            Action::FireRocket { .. } => (ROCKET_SPEED, 0),
                            _ => (0, 0),
                        };
                        if speed > 0 {
                            let aim = f.profile.aim(
                                me.pos,
                                u,
                                (distance(me.pos, u.pos) * TPS / speed + prep) as u64,
                                f.seed,
                                step.intended ^ u.id as u64,
                            );
                            action = match action {
                                Action::Grenade { .. } => Action::Grenade { aim },
                                Action::FireRocket { .. } => Action::FireRocket { aim },
                                a => a,
                            };
                        }
                    }
                    if matches!(action, Action::Detonate { .. }) {
                        self.intended_detonation = Some(step.intended);
                    }
                    let landing = p.landing;
                    self.plan = Some(p.clone());
                    if !self.accept(f, me, action, out) {
                        self.plan = None;
                        return;
                    }
                    if let Some(v) = self.flight.as_mut() {
                        v.goal = landing;
                    }
                    self.plan = None;
                }
            }
            if !p.steps.is_empty() || self.flight.is_some() {
                self.plan = Some(p);
                return;
            }
        }
        if self.flight.is_some() || self.cast.is_some() || f.stunned || f.silenced {
            return;
        }
        if f.tick < self.reaction_until || f.tick < self.next_think {
            return;
        }
        self.next_think = f.tick + 12;
        self.decision += 1;
        self.candidates = self.evaluate(f, me);
        self.evaluated = self.candidates.len();
        self.search(f, me);
        let shortlist: Vec<_> = self
            .candidates
            .iter()
            .filter(|c| c.legal)
            .take(BEAM_WIDTH as usize)
            .cloned()
            .collect();
        let Some(best) = shortlist.first().filter(|c| c.score > 0).cloned() else {
            return;
        };
        self.continuation = best.line.clone();
        if best.family == "shoot"
            || best.family == "rotate"
            || best.family == "evade"
            || best.family == "retreat"
        {
            let to = if best.family == "shoot" {
                let u = f.units.iter().find(|u| Some(u.id) == best.target).unwrap();
                let gun = Gun::from_index(self.gun);
                let offset = (gun.reach() * (60 + f.profile.get(2) * 25 / 100) / 100).max(10000);
                u.pos.toward(me.pos, offset).bounded()
            } else {
                best.landing
            };
            if !f.blocked(to, BODY_RADIUS) {
                out.push(Cmd::Guide { to });
            }
            return;
        }
        let mut plan = self.make_plan(f, me, &best, f.tick);
        let mut at = plan.steps.last().map_or(f.tick, |s| s.at)
            + if best.family == "rocket" { 12 } else { 24 };
        // The preview is the selected resource-valid branch. Queue its uncommitted stages too.
        for family in best.line.iter().skip(1) {
            if family == "shoot" {
                continue;
            }
            if let Some(c) = self
                .candidates
                .iter()
                .find(|c| c.family == *family && c.target == best.target)
                .cloned()
            {
                let more = self.make_plan(f, me, &c, at);
                at = more.steps.last().map_or(at, |s| s.at) + 24;
                plan.steps.extend(more.steps);
            }
        }
        self.plan = Some(plan);
        // Acceptance occurs on the next tick, preserving an inspectable selected continuation.
    }
    fn search(&mut self, _f: &Frame, me: &Unit) {
        // Beam nodes contain only a resource/position forecast. No cloning combat cores or host calls.
        #[derive(Clone)]
        struct Node {
            first: usize,
            line: Vec<String>,
            score: i64,
            pos: Point,
            stock: [u8; 3],
            spent: [bool; 3],
        }
        let apply = |n: &mut Node, family: &str, landing: Point| match family {
            "single_launch" | "escape" => {
                n.stock[0] = n.stock[0].saturating_sub(1);
                n.pos = landing
            }
            "double_launch" => {
                n.stock[0] = n.stock[0].saturating_sub(2);
                n.pos = landing
            }
            "rocket_entry" => {
                n.stock[0] = n.stock[0].saturating_sub(2);
                n.pos = landing;
                n.spent[2] = true
            }
            "grenade" => {
                n.stock[1] = n.stock[1].saturating_sub(1);
                n.spent[0] = true
            }
            "bot" => {
                n.stock[2] = n.stock[2].saturating_sub(1);
                n.spent[1] = true
            }
            "rocket" => n.spent[2] = true,
            _ => {}
        };
        let mut beam: Vec<_> = self
            .candidates
            .iter()
            .enumerate()
            .filter(|(_, c)| c.legal)
            .take(BEAM_WIDTH as usize)
            .map(|(i, c)| {
                let mut n = Node {
                    first: i,
                    line: vec![c.family.clone()],
                    score: c.score,
                    pos: me.pos,
                    stock: self.stock,
                    spent: [false; 3],
                };
                apply(&mut n, &c.family, c.landing);
                n
            })
            .collect();
        for depth in 1..self.horizon {
            let mut next = vec![];
            for n in &beam {
                let mut expanded = 0;
                for c in self
                    .candidates
                    .iter()
                    .filter(|c| c.legal && c.target == self.candidates[n.first].target)
                {
                    if self.evaluated >= PLANNER_CANDIDATES as usize || expanded >= 2 {
                        break;
                    }
                    let allowed = match c.family.as_str() {
                        "grenade" => {
                            n.stock[1] > 0 && !n.spent[0] && distance(n.pos, c.aim) <= GRENADE_RANGE
                        }
                        "bot" => n.stock[2] > 0 && !n.spent[1],
                        "rocket" => !n.spent[2] && distance(n.pos, c.aim) <= ROCKET_RANGE,
                        "rocket_entry" => n.stock[0] >= 2 && !n.spent[2],
                        "single_launch" | "escape" => n.stock[0] > 0,
                        "double_launch" => n.stock[0] >= 2,
                        "shoot" => true,
                        _ => false,
                    };
                    if !allowed {
                        continue;
                    }
                    self.evaluated += 1;
                    expanded += 1;
                    let value = if c.family == "shoot" {
                        if distance(n.pos, c.aim) <= Gun::from_index(self.gun).reach() {
                            36
                        } else {
                            3
                        }
                    } else {
                        c.immediate.max(0) / 2
                    };
                    let mut v = n.clone();
                    v.score += value / (depth as i64 + 1);
                    v.line.push(c.family.clone());
                    apply(&mut v, &c.family, c.landing);
                    next.push(v);
                }
            }
            next.sort_by(|a, b| b.score.cmp(&a.score).then(a.line.cmp(&b.line)));
            next.truncate(BEAM_WIDTH as usize);
            if next.is_empty() {
                break;
            }
            beam = next;
        }
        for n in beam {
            let c = &mut self.candidates[n.first];
            if n.score >= c.score {
                c.follow_up = n.score - c.score;
                c.score = n.score;
                c.line = n.line;
            }
        }
        self.candidates.sort_by(|a, b| {
            b.legal
                .cmp(&a.legal)
                .then(b.score.cmp(&a.score))
                .then(a.family.cmp(&b.family))
                .then(a.target.cmp(&b.target))
        });
    }
}
