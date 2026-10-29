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
pub fn facing(direction: Point) -> usize {
    if direction.x.abs() > direction.y.abs() * 2 {
        if direction.x >= 0 {
            0
        } else {
            4
        }
    } else if direction.y.abs() > direction.x.abs() * 2 {
        if direction.y >= 0 {
            2
        } else {
            6
        }
    } else {
        match (direction.x >= 0, direction.y >= 0) {
            (true, true) => 1,
            (false, true) => 3,
            (false, false) => 5,
            _ => 7,
        }
    }
}
pub fn rocket_damage(raw: i64, distance: i64) -> i64 {
    if distance > ROCKET_RADIUS {
        0
    } else {
        raw * (4 * ROCKET_RADIUS - 3 * distance.max(0)) / (4 * ROCKET_RADIUS)
    }
}
/// Enemy-only multiplier, applied after the original rounding and falloff.
pub fn skill_raw(raw: i64) -> i64 {
    raw * SKILL_DAMAGE_PERCENT / 100
}
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct LaunchProfile {
    pub distance: i64,
    pub horizontal_speed: i64,
    pub peak: i64,
    pub ticks: u64,
}
/// Authored dome curve, shared by manual physics, AI placement and Lab guides.
pub fn launch_profile(d: i64) -> LaunchProfile {
    let d = d.max(0);
    let (horizontal_speed, peak, ticks) = if d <= DOME_CENTER_RADIUS {
        (0, DOME_CENTER_PEAK, DOME_CENTER_TICKS)
    } else if d < DOME_SWEET_RADIUS {
        let n = d - DOME_CENTER_RADIUS;
        let span = DOME_SWEET_RADIUS - DOME_CENTER_RADIUS;
        (
            LAUNCH_SPEED * n / span,
            DOME_CENTER_PEAK + (DOME_SWEET_PEAK - DOME_CENTER_PEAK) * n / span,
            DOME_CENTER_TICKS + (FLIGHT_TICKS - DOME_CENTER_TICKS) * n / span,
        )
    } else if d < PACK_RADIUS {
        let n = PACK_RADIUS - d;
        let span = PACK_RADIUS - DOME_SWEET_RADIUS;
        (
            LAUNCH_SPEED * n / span,
            DOME_SWEET_PEAK * n / span,
            FLIGHT_TICKS * n / span,
        )
    } else {
        (0, 0, 0)
    };
    LaunchProfile {
        distance: d,
        horizontal_speed,
        peak,
        ticks: ticks as u64,
    }
}
pub fn launch_distance_for_travel(travel: i64) -> i64 {
    (1..=9)
        .map(|n| DOME_CENTER_RADIUS * n)
        .min_by_key(|d| {
            let p = launch_profile(*d);
            (
                (p.horizontal_speed * p.ticks as i64 / TPS - travel).abs(),
                *d,
            )
        })
        .unwrap_or(DOME_SWEET_RADIUS)
}
pub fn landing_protection_ticks(enemies: usize) -> u64 {
    if enemies == 0 {
        0
    } else {
        (AIR_LINGER_BASE_TICKS
            + AIR_LINGER_PER_ENEMY * (enemies.min(AIR_LINGER_MAX_ENEMIES as usize) as i64 - 1))
            .min(AIR_LINGER_MAX_TICKS) as u64
    }
}
fn default_arc_peak() -> i64 {
    16000
}
/// Distance in tick n, using the midpoint of that tick's linear acceleration.
pub fn rocket_tick_distance(age: u64) -> i64 {
    let n = age.min(ROCKET_ACCEL_TICKS as u64) as i64;
    let speed2 = if n < ROCKET_ACCEL_TICKS {
        2 * ROCKET_START_SPEED
            + (ROCKET_SPEED - ROCKET_START_SPEED) * (2 * n + 1) / ROCKET_ACCEL_TICKS
    } else {
        2 * ROCKET_SPEED
    };
    speed2 / (2 * TPS)
}
pub fn rocket_eta(mut remaining: i64, mut age: u64) -> u64 {
    let mut ticks = 0;
    while remaining > 0 && ticks < 600 {
        remaining -= rocket_tick_distance(age);
        age += 1;
        ticks += 1;
    }
    ticks.max(1)
}
/// Round 143: the air-steering target turns at most AIR_STRAFE_MAX_DEG (155) away from the flight heading, so an
/// opposite input curves her round instead of braking through zero and reversing outright.
pub fn strafe_limit(velocity: Point, desired: Point) -> Point {
    let (vl, dl) = (velocity.length(), desired.length());
    if vl == 0 || dl == 0 {
        return desired;
    }
    let dot = velocity.x as i128 * desired.x as i128 + velocity.y as i128 * desired.y as i128;
    if dot * 1000 >= AIR_STRAFE_COS as i128 * vl as i128 * dl as i128 {
        return desired;
    }
    let cross = velocity.x as i128 * desired.y as i128 - velocity.y as i128 * desired.x as i128;
    let s = if cross < 0 {
        -AIR_STRAFE_SIN
    } else {
        AIR_STRAFE_SIN
    };
    let h = velocity.scale(1000, vl);
    Point::new(
        (h.x * AIR_STRAFE_COS - h.y * s) / 1000,
        (h.x * s + h.y * AIR_STRAFE_COS) / 1000,
    )
    .scale(dl, 1000)
}
/// Round 143: the most speed that still stops within `distance` at the air acceleration.
pub fn arrival_speed(distance: i64) -> i64 {
    ((2 * AIR_ACCEL as u128 * distance.max(0) as u128).isqrt() as i64).min(ROCKET_MAX_SPEED)
}
fn segment_near(a: Point, b: Point, p: Point, r: i64) -> bool {
    let d = b.sub(a);
    let len = d.x * d.x + d.y * d.y;
    if len == 0 {
        return distance(a, p) <= r;
    }
    let t = ((p.x - a.x) * d.x + (p.y - a.y) * d.y).clamp(0, len);
    distance(a.add(d.scale(t, len)), p) <= r
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
    pub mr: i64,
    pub visible: bool,
    pub targetable: bool,
    pub kind: u8,
    pub farmable: bool,
    pub controlled: bool,
    pub melee: bool,
    pub push_immune: bool,
    pub move_speed: i64,
    pub defence: i64,
    pub attack_range: i64,
    pub shield: i64,
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
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Surface {
    pub id: u64,
    pub bounds: Rect,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Attachment {
    pub surface: Option<u64>,
    pub kind: String,
    pub point: Point,
    pub normal: Point,
    pub offset: Point,
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
    /// Round 144: the champion a kill (kind 0) took down; 0 when unknown (assists).
    #[serde(default)]
    pub victim: usize,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Defender {
    pub id: usize,
    pub profile: Profile,
    pub observed: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DodgeMemory {
    pub rocket: u64,
    pub noticed: u64,
    pub ready: u64,
    pub responds: bool,
    pub goal: Option<Point>,
    pub reason: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Action {
    Buy,
    DropPack,
    Grenade { aim: Point },
    Pack { aim: Point },
    Detonate { id: u64 },
    Bot { aim: Point },
    EquipRocket,
    FireRocket { aim: Point },
    Steer { input: Point },
    /// Round 144: Bucky's alternate fire, one slug at one visible target.
    Slug { target: usize },
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
    pub steering: Option<Point>,
    pub units: Vec<Unit>,
    pub blocks: Vec<Rect>,
    pub surfaces: Vec<Surface>,
    pub defenders: Vec<Defender>,
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
    /// Round 143: where the game is walking her (its latest fresh move order), for the takeoff gate.
    pub walk: Option<Point>,
    /// Round 144: her champion level (skills gain SKILL_LEVEL_BASE base damage per level after the first).
    pub level: i64,
    /// Round 144: the athlete's Raze mastery, 0 Iron to 7 Radiant. It changes planning only, never raw power.
    pub mastery: u8,
}
impl Default for Frame {
    fn default() -> Self {
        Self {
            version: 3,
            tick: 0,
            owner: 1,
            seed: 1,
            automatic: false,
            profile: Profile::default(),
            action: None,
            steering: None,
            units: vec![],
            blocks: vec![],
            surfaces: vec![],
            defenders: vec![],
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
            walk: None,
            level: 1,
            mastery: 0,
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
            || self.surfaces.iter().any(|s| s.bounds.contains(p, r))
    }
    /// Round 143: a guide near a wall is pulled back toward `from` onto the first free point (as
    /// `walls::pull_back`), with 2,000 padding, instead of being dropped.
    pub fn free_point(&self, from: Point, to: Point) -> Point {
        let steps = (distance(from, to) / 3000).max(1);
        (0..=steps)
            .map(|i| to.add(from.sub(to).scale(i, steps)))
            .find(|p| !self.blocked(*p, 2000))
            .unwrap_or(from)
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
    /// Swept wall reflection, shared by the grenade and bot. No unit collision.
    /// A bounded number of face contacts prevents loops in narrow corners.
    pub fn ricochet(
        &self,
        from: Point,
        direction: Point,
        travel: i64,
        r: i64,
    ) -> (Point, Point, u32) {
        let mut pos = from;
        let mut dir = direction;
        let mut left = travel;
        let mut bounces = 0;
        if self.blocked(pos, r) {
            return (pos, dir, 0);
        }
        for _ in 0..8 {
            if left <= 0 || dir.length() == 0 {
                break;
            }
            let delta = dir.scale(left, dir.length());
            let mut hit = 1_000_001i64;
            let mut normal = Point::default();
            let mut consider = |t: i64, n: Point| {
                if (0..=1_000_000).contains(&t) {
                    if t < hit {
                        hit = t;
                        normal = n;
                    } else if t == hit {
                        if n.x != 0 {
                            normal.x = n.x;
                        }
                        if n.y != 0 {
                            normal.y = n.y;
                        }
                    }
                }
            };
            // Map edges are solid faces too.
            for (start, step, n) in [
                (pos.x, delta.x, Point::new(1, 0)),
                (pos.y, delta.y, Point::new(0, 1)),
            ] {
                if step > 0 && start + step >= MAP_SIZE - r {
                    consider((MAP_SIZE - r - start) * 1_000_000 / step, n.scale(-1, 1));
                } else if step < 0 && start + step <= r {
                    consider((start - r) * 1_000_000 / -step, n);
                }
            }
            for rect in self
                .blocks
                .iter()
                .chain(self.surfaces.iter().map(|s| &s.bounds))
            {
                let mut enter = 0;
                let mut exit = 1_000_000;
                let mut face = Point::default();
                let mut valid = true;
                for (start, step, lo, hi, axis) in [
                    (
                        pos.x,
                        delta.x,
                        rect.min.x - r,
                        rect.max.x + r,
                        Point::new(1, 0),
                    ),
                    (
                        pos.y,
                        delta.y,
                        rect.min.y - r,
                        rect.max.y + r,
                        Point::new(0, 1),
                    ),
                ] {
                    if step == 0 {
                        if start < lo || start > hi {
                            valid = false;
                            break;
                        }
                        continue;
                    }
                    let (a, b, n) = if step > 0 {
                        (lo - start, hi - start, axis.scale(-1, 1))
                    } else {
                        (start - hi, start - lo, axis)
                    };
                    let t = -(-a * 1_000_000).div_euclid(step.abs());
                    let end = (b * 1_000_000).div_euclid(step.abs());
                    if t > enter {
                        enter = t;
                        face = n;
                    } else if t == enter {
                        if n.x != 0 {
                            face.x = n.x;
                        }
                        if n.y != 0 {
                            face.y = n.y;
                        }
                    }
                    exit = exit.min(end);
                    if enter > exit {
                        valid = false;
                        break;
                    }
                }
                if valid && exit >= 0 && face != Point::default() {
                    consider(enter, face);
                }
            }
            if hit > 1_000_000 {
                pos = pos.add(delta);
                break;
            }
            // Stop just outside the solid, retaining the untravelled distance.
            let used = (left * hit / 1_000_000).max(0);
            let contact = pos.add(delta.scale(hit, 1_000_000)).add(normal.scale(2, 1));
            if self.blocked(contact, r) {
                break;
            }
            pos = contact;
            left = (left - used).saturating_sub(2);
            if normal.x != 0 {
                dir.x = -dir.x;
            }
            if normal.y != 0 {
                dir.y = -dir.y;
            }
            bounces += 1;
        }
        (pos, dir, bounces)
    }
    pub fn visible_enemies<'a>(&'a self, me: &'a Unit) -> impl Iterator<Item = &'a Unit> {
        self.units
            .iter()
            .filter(move |u| u.team != me.team && u.visible && u.damageable())
    }
    /// First solid face along a throw. Unit positions are deliberately not attachment surfaces.
    pub fn pack_attachment(&self, from: Point, aim: Point) -> Option<(Point, Attachment)> {
        if self.blocked(from, PACK_CARRIER_RADIUS) {
            return None;
        }
        let delta = aim.sub(from);
        let mut first: Option<(i64, u64, Point, Option<u64>, Point)> = None;
        for (rect, id) in self
            .blocks
            .iter()
            .map(|b| (b, None))
            .chain(self.surfaces.iter().map(|s| (&s.bounds, Some(s.id))))
        {
            let mut enter = 0;
            let mut exit = 1_000_000;
            let mut normal = Point::default();
            let mut valid = true;
            for (start, step, lo, hi, axis) in [
                (
                    from.x,
                    delta.x,
                    rect.min.x - PACK_CARRIER_RADIUS,
                    rect.max.x + PACK_CARRIER_RADIUS,
                    Point::new(1, 0),
                ),
                (
                    from.y,
                    delta.y,
                    rect.min.y - PACK_CARRIER_RADIUS,
                    rect.max.y + PACK_CARRIER_RADIUS,
                    Point::new(0, 1),
                ),
            ] {
                if step == 0 {
                    if start < lo || start > hi {
                        valid = false;
                        break;
                    }
                    continue;
                }
                let (a, b, n) = if step > 0 {
                    (lo - start, hi - start, axis.scale(-1, 1))
                } else {
                    (start - hi, start - lo, axis)
                };
                let den = step.abs();
                let t = -(-a * 1_000_000).div_euclid(den);
                let end = (b * 1_000_000).div_euclid(den);
                if t > enter {
                    enter = t;
                    normal = n;
                }
                exit = exit.min(end);
                if enter > exit {
                    valid = false;
                    break;
                }
            }
            if valid && enter <= 1_000_000 && exit >= 0 && normal != Point::default() {
                let key = id.unwrap_or(u64::MAX);
                if first.as_ref().is_none_or(|v| (enter, key) < (v.0, v.1)) {
                    first = Some((enter, key, normal, id, rect.min));
                }
            }
        }
        if let Some((t, _, normal, id, origin)) = first {
            let pos = from.add(delta.scale(t, 1_000_000)).add(normal.scale(2, 1));
            if self.blocked(pos, PACK_CARRIER_RADIUS) {
                return None;
            }
            let point = pos.sub(normal.scale(PACK_CARRIER_RADIUS + 2, 1));
            Some((
                pos,
                Attachment {
                    surface: id,
                    kind: if id.is_some() { "structure" } else { "wall" }.into(),
                    point,
                    normal,
                    offset: point.sub(origin),
                },
            ))
        } else if !self.blocked(aim, PACK_CARRIER_RADIUS) {
            Some((
                aim,
                Attachment {
                    surface: None,
                    kind: "ground".into(),
                    point: aim,
                    normal: Point::default(),
                    offset: Point::default(),
                },
            ))
        } else {
            None
        }
    }
    fn drop_pack(&self, from: Point) -> Option<Point> {
        if !self.blocked(from, PACK_CARRIER_RADIUS) {
            return Some(from);
        }
        for radius in (1000..=32000).step_by(1000) {
            let mut choices = vec![];
            for dx in -1..=1 {
                for dy in -1..=1 {
                    if dx != 0 || dy != 0 {
                        let p = from.add(Point::new(dx * radius, dy * radius));
                        if !self.blocked(p, PACK_CARRIER_RADIUS) {
                            choices.push(p);
                        }
                    }
                }
            }
            choices.sort_by_key(|p| (distance(from, *p), p.x, p.y));
            if let Some(p) = choices.first() {
                return Some(*p);
            }
        }
        None
    }
    pub fn rocket_impact(
        &self,
        team: usize,
        from: Point,
        to: Point,
        visible_only: bool,
    ) -> (Point, bool) {
        let (wall, blocked) = self.sweep(from, to, 1000);
        let d = to.sub(from);
        let den = d.x as i128 * d.x as i128 + d.y as i128 * d.y as i128;
        let hit = self
            .units
            .iter()
            .filter(|u| u.team != team && u.alive() && u.targetable && (!visible_only || u.visible))
            .filter_map(|u| {
                let q = u.pos.sub(from);
                let dot = q.x as i128 * d.x as i128 + q.y as i128 * d.y as i128;
                let c = q.x as i128 * q.x as i128 + q.y as i128 * q.y as i128 - 8000i128.pow(2);
                let disc = dot * dot - den * c;
                if disc < 0 || den == 0 {
                    return None;
                }
                let t = if c <= 0 {
                    0
                } else {
                    dot - (disc as u128).isqrt() as i128
                };
                if t < 0 || t > den {
                    return None;
                }
                let p = from.add(d.scale(t as i64, den as i64));
                (distance(from, p) <= distance(from, wall)).then_some((t, u.id, p))
            })
            .min_by_key(|v| (v.0, v.1));
        hit.map_or((wall, blocked), |v| (v.2, true))
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Cmd {
    AirProtection {
        active: bool,
    },
    Damage {
        id: u64,
        target: usize,
        raw: i64,
        self_hit: bool,
        #[serde(default)]
        source: String,
        /// Round 144: the percent of the damage actually dealt that heals her (omnivamp).
        #[serde(default)]
        vamp: i64,
        /// Round 144: true damage (Combustion), past armour.
        #[serde(default)]
        fixed: bool,
    },
    /// Round 144: one shield layer on her (the air shield).
    Shield {
        amount: i64,
        ticks: u64,
    },
    Slow {
        target: usize,
        percent: i64,
        ticks: u64,
    },
    Stun {
        target: usize,
        ticks: u64,
    },
    KnockUp {
        target: usize,
        ticks: u64,
    },
    Reveal {
        target: usize,
        ticks: u64,
    },
    /// Bucky's slug: a cosmetic tracer from her to the target.
    Tracer {
        to: Point,
    },
    Push {
        target: usize,
        to: Point,
    },
    Knockback {
        target: usize,
        to: Point,
        ticks: u64,
    },
    ClearWarning {
        id: u64,
    },
    EnemyGuide {
        target: usize,
        rocket: u64,
        to: Point,
        until: u64,
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
    pub attachment: Attachment,
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
    pub attack: i64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Flight {
    pub satchel_count: u8,
    pub grounded: bool,
    pub boosted: bool,
    pub arc_born: u64,
    pub arc_height: i64,
    #[serde(default = "default_arc_peak")]
    pub arc_peak: i64,
    pub velocity: Point,
    pub remainder: Point,
    pub born: u64,
    pub until: u64,
    pub input: Point,
    pub next_input: u64,
    pub goal: Point,
}
impl Flight {
    pub fn height(&self, tick: u64) -> i64 {
        if self.grounded {
            return 0;
        }
        let total = self.until.saturating_sub(self.arc_born).max(1);
        let elapsed = tick.saturating_sub(self.arc_born).min(total);
        self.arc_height * (total - elapsed) as i64 / total as i64
            + (4 * self.arc_peak as u64 * elapsed * (total - elapsed) / (total * total)) as i64
    }
    fn cap(&self) -> i64 {
        if self.boosted {
            ROCKET_MAX_SPEED
        } else {
            MAX_SPEED
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Cast {
    pub action: Action,
    pub release: u64,
    pub attack: i64,
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
    pub attack: i64,
    pub main_done: bool,
    pub centres: Vec<Point>,
    pub pos: Point,
    pub direction: Point,
    pub remaining: i64,
    pub bounces: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Rocket {
    pub id: u64,
    pub pos: Point,
    pub direction: Point,
    pub endpoint: Point,
    pub remaining: i64,
    pub attack: i64,
    pub born: u64,
    pub age: u64,
    pub speed: i64,
    pub impact: Point,
    pub impact_tick: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Sighting {
    pub pos: Point,
    pub velocity: Point,
    pub tick: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Reflection {
    pub axis: u8,
    pub coordinate: i64,
}
impl Reflection {
    fn apply(&self, p: Point) -> Point {
        if self.axis == 0 {
            Point::new(2 * self.coordinate - p.x, p.y)
        } else {
            Point::new(p.x, 2 * self.coordinate - p.y)
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlannedStep {
    pub at: u64,
    pub action: Action,
    pub relative: bool,
    pub intended: u64,
    pub target_offset: Point,
    #[serde(default)]
    pub reflection: Option<Reflection>,
    #[serde(default)]
    pub landing: Option<Point>,
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
    #[serde(default)]
    pub reflection: Option<Reflection>,
    #[serde(default)]
    pub hazard: Option<u64>,
    #[serde(default)]
    pub entry: Option<EntryReview>,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct EntryReview {
    pub allowed: bool,
    pub reason: String,
    pub ally: Option<usize>,
    pub ally_eta: Option<u64>,
    pub escape: Option<Point>,
    pub retained_packs: u8,
    pub predicted_hp: i64,
    pub incoming: i64,
    pub expected_damage: i64,
    pub solo_pick: bool,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct WeaponSaving {
    pub goal: Option<usize>,
    pub reserve: i64,
    pub review_at: u64,
    pub close: bool,
    pub reason: String,
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
    pub rocket_shots: u64,
    pub rocket_entries: u64,
    pub triple_boosts: u64,
    pub rocket_objective_hits: u64,
    pub protected_ticks: u64,
    #[serde(default)]
    pub damage_by_source: BTreeMap<String, i64>,
}
/// Round 143: the ground movement she holds between decisions (re-issued every tick, held on arrival).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Guide {
    pub family: String,
    pub target: Option<usize>,
    pub to: Point,
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
    pub air_protection: bool,
    #[serde(default)]
    pub air_protection_until: u64,
    #[serde(default)]
    pub air_protection_enemies: u8,
    #[serde(default)]
    pub last_launch: Option<LaunchProfile>,
    pub last_weapon_buy: Option<u64>,
    pub dodges: BTreeMap<usize, DodgeMemory>,
    /// Round 143: the free Blast Pack charge returns at this tick (grenade: cooldowns[0], bot: cooldowns[1]).
    #[serde(default)]
    pub pack_cooldown: u64,
    #[serde(default)]
    pub last_think: u64,
    #[serde(default)]
    pub guide: Option<Guide>,
    /// The hazard being evaded and the one direction kept until it ends.
    #[serde(default)]
    pub evade: Option<(u64, Point)>,
    #[serde(default)]
    pub savings: WeaponSaving,
    #[serde(default)]
    pub last_entry: Option<EntryReview>,
    /// Round 144: the level and mastery of the current frame.
    #[serde(default)]
    pub level: i64,
    #[serde(default)]
    pub mastery: u8,
    /// Round 144: Hype (0..HYPE_MAX) fills from her explosions; at full she bursts into Carnaval.
    #[serde(default)]
    pub hype: i64,
    #[serde(default)]
    pub carnaval_until: u64,
    #[serde(default)]
    pub last_combat: u64,
    #[serde(default)]
    pub air_shield: Option<AirShield>,
    /// Graffiti splashes left by Paint Shells.
    #[serde(default)]
    pub paints: Vec<Zone>,
    /// Showstopper burning zones.
    #[serde(default)]
    pub burn_zones: Vec<Zone>,
    /// Ticks of her recent explosions (Encore counts them).
    #[serde(default)]
    pub explosions: Vec<u64>,
    #[serde(default)]
    pub encore_ready: u64,
    /// Paint Shells hits on champions toward Hot Hands.
    #[serde(default)]
    pub hot_hands: u8,
    /// Combustion: (stacks, until) per enemy champion.
    #[serde(default)]
    pub burns: BTreeMap<usize, (u8, u64)>,
    /// The last tick she damaged each enemy champion (the Blast Pack reset).
    #[serde(default)]
    pub last_hits: BTreeMap<usize, u64>,
    /// Primed by a Boom Bot hit, until this tick.
    #[serde(default)]
    pub primed: BTreeMap<usize, u64>,
    /// The tick a satchel last pushed each enemy (the satchel combo).
    #[serde(default)]
    pub pushes: BTreeMap<usize, u64>,
    /// Recent Paint Shells main blasts (point, tick), for the satchel combo.
    #[serde(default)]
    pub paint_blasts: Vec<(Point, u64)>,
    #[serde(default)]
    pub slug_ready: u64,
    #[serde(default)]
    pub sheriff_given: bool,
    /// The tick the rocket was last armed (its pose is held ROCKET_HOLD ticks before an AI shot).
    #[serde(default)]
    pub armed_at: u64,
}
/// Round 144: the air shield. One engine layer at a time: `granted` is what the current layer gave, `base` the shield
/// she had besides it, `layer_end` when that layer expires and `end` when the whole shield ends (set on landing).
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct AirShield {
    pub granted: i64,
    pub base: i64,
    pub left: i64,
    pub layer_end: u64,
    pub end: Option<u64>,
}
/// Round 144: a timed circle (Graffiti paint, Showstopper burn).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Zone {
    pub id: u64,
    pub pos: Point,
    pub radius: i64,
    pub born: u64,
    pub until: u64,
    pub attack: i64,
}
impl Default for Core {
    fn default() -> Self {
        Self {
            credits: START_CREDITS,
            gun: START_GUN as usize,
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
            air_protection: false,
            air_protection_until: 0,
            air_protection_enemies: 0,
            last_launch: None,
            last_weapon_buy: None,
            dodges: BTreeMap::new(),
            pack_cooldown: 0,
            last_think: 0,
            guide: None,
            evade: None,
            savings: WeaponSaving::default(),
            last_entry: None,
            level: 1,
            mastery: 0,
            hype: 0,
            carnaval_until: 0,
            last_combat: 0,
            air_shield: None,
            paints: vec![],
            burn_zones: vec![],
            explosions: vec![],
            encore_ready: 0,
            hot_hands: 0,
            burns: BTreeMap::new(),
            last_hits: BTreeMap::new(),
            primed: BTreeMap::new(),
            pushes: BTreeMap::new(),
            paint_blasts: vec![],
            slug_ready: 0,
            sheriff_given: false,
            armed_at: 0,
        }
    }
}
impl Core {
    pub fn airborne(&self) -> bool {
        self.flight.as_ref().is_some_and(|v| !v.grounded)
    }
    pub fn protected_damage(&self, raw: i64) -> i64 {
        raw * if self.air_protection {
            100 - AIR_REDUCTION
        } else {
            100
        } / 100
    }
    pub fn carnaval(&self, tick: u64) -> bool {
        tick < self.carnaval_until
    }
    /// Round 144: a skill's base damage plus SKILL_LEVEL_BASE per level after the first.
    pub fn base(&self, n: i64) -> i64 {
        n + SKILL_LEVEL_BASE * (self.level.max(1) - 1)
    }
    /// Round 144: the air shield, doubled during Carnaval.
    pub fn air_shield_amount(&self, me: &Unit, tick: u64) -> i64 {
        (me.max_hp * AIR_SHIELD_HP_PERCENT / 100 + me.attack * AIR_SHIELD_ATTACK / 100)
            * if self.carnaval(tick) { 2 } else { 1 }
    }
    fn gain_hype(&mut self, n: i64, tick: u64) {
        self.hype = (self.hype + n).clamp(0, HYPE_MAX);
        self.last_combat = tick;
    }
    fn explode(&mut self, tick: u64) {
        self.explosions.push(tick);
        self.explosions
            .retain(|t| tick < t + ENCORE_WINDOW as u64);
    }
    /// Round 144: a launch tops the air shield up to full. The first launch grants a layer that ends with the
    /// flight; while a shield is up, the next launch adds only what was lost, ending with the current layer.
    fn grant_air_shield(&mut self, f: &Frame, me: &Unit, until: u64, out: &mut Vec<Cmd>) {
        let full = self.air_shield_amount(me, f.tick);
        match self.air_shield.as_mut() {
            Some(s) => {
                s.end = None;
                let add = full - s.left;
                if add > 0 && s.layer_end > f.tick {
                    out.push(Cmd::Shield {
                        amount: add,
                        ticks: s.layer_end - f.tick,
                    });
                    s.granted += add;
                    s.left = full;
                }
            }
            None => {
                let ticks = until.saturating_sub(f.tick).max(1);
                out.push(Cmd::Shield {
                    amount: full,
                    ticks,
                });
                self.air_shield = Some(AirShield {
                    granted: full,
                    base: f.shield,
                    left: full,
                    layer_end: f.tick + ticks,
                    end: None,
                });
            }
        }
    }
    /// Round 144: keeps the air shield's single layer alive through the flight and the landing linger, granting
    /// what is left of it again when a layer runs out, and drops it once broken or over.
    fn keep_air_shield(&mut self, f: &Frame, out: &mut Vec<Cmd>) {
        let flight_end = self
            .flight
            .as_ref()
            .filter(|v| !v.grounded)
            .map(|v| v.until);
        let Some(s) = self.air_shield.as_mut() else {
            return;
        };
        if f.tick < s.layer_end {
            s.left = (f.shield - s.base).clamp(0, s.granted);
        }
        let end = flight_end.or(s.end).unwrap_or(s.layer_end);
        if s.left <= 0 || f.tick >= end {
            self.air_shield = None;
            return;
        }
        if f.tick >= s.layer_end {
            out.push(Cmd::Shield {
                amount: s.left,
                ticks: end - f.tick,
            });
            s.granted = s.left;
            s.layer_end = end;
        }
    }
    fn sync_air(&mut self, tick: u64, out: &mut Vec<Cmd>) {
        let active = self.airborne() || tick < self.air_protection_until;
        if active != self.air_protection {
            self.air_protection = active;
            out.push(Cmd::AirProtection { active });
        }
    }
    fn finish_flight(
        &mut self,
        f: &Frame,
        me: &Unit,
        at: Point,
        was_airborne: bool,
        out: &mut Vec<Cmd>,
    ) {
        self.flight = None;
        if was_airborne && me.alive() {
            let count = f
                .units
                .iter()
                .filter(|u| {
                    u.team != me.team
                        && u.visible
                        && u.alive()
                        && u.champion()
                        && distance(u.pos, at) <= AIR_LINGER_RADIUS
                })
                .count();
            self.air_protection_enemies = count.min(AIR_LINGER_MAX_ENEMIES as usize) as u8;
            self.air_protection_until = self
                .air_protection_until
                .max(f.tick + landing_protection_ticks(count));
            // Round 144: the air shield lingers for the same window as landing defence.
            if let Some(s) = self.air_shield.as_mut() {
                s.end = Some(f.tick + landing_protection_ticks(count));
            }
            self.encore(f, me, at, out);
        }
        self.sync_air(f.tick, out);
    }
    /// Round 144, Encore: her first landing after ENCORE_EXPLOSIONS explosions within ENCORE_WINDOW sends out a
    /// small shockwave that knocks enemy champions up.
    fn encore(&mut self, f: &Frame, me: &Unit, at: Point, out: &mut Vec<Cmd>) {
        self.explosions
            .retain(|t| f.tick < t + ENCORE_WINDOW as u64);
        if f.tick < self.encore_ready || (self.explosions.len() as i64) < ENCORE_EXPLOSIONS {
            return;
        }
        self.encore_ready = f.tick + ENCORE_COOLDOWN as u64;
        self.explosions.clear();
        let id = self.id();
        for u in f
            .units
            .iter()
            .filter(|u| u.team != me.team && u.damageable() && distance(u.pos, at) <= ENCORE_RADIUS)
        {
            self.hit(f, id, u, me.attack * ENCORE_ATTACK / 100, "encore", out);
            if u.champion() && !u.push_immune {
                out.push(Cmd::KnockUp {
                    target: u.id,
                    ticks: ENCORE_LIFT_TICKS as u64,
                });
            }
        }
        out.push(Cmd::Visual {
            id,
            tag: "encore".into(),
            pos: at,
            height: 0,
            until: f.tick + 18,
        });
    }
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
    /// Omen's Buy Phase (round 143): each utility (0 pack, 1 grenade, 2 bot) has one free charge that returns on
    /// its cooldown, like his free smoke, plus at most one bought charge in `stock`, like his second smoke.
    fn free_clock(&self, n: usize) -> u64 {
        match n {
            0 => self.pack_cooldown,
            1 => self.cooldowns[0],
            _ => self.cooldowns[1],
        }
    }
    fn set_free_clock(&mut self, n: usize, until: u64) {
        match n {
            0 => self.pack_cooldown = until,
            1 => self.cooldowns[0] = until,
            _ => self.cooldowns[1] = until,
        }
    }
    pub fn charges(&self, n: usize, tick: u64) -> u8 {
        self.stock[n] + (tick >= self.free_clock(n)) as u8
    }
    /// Spends the free charge first (it starts its cooldown), else the bought one.
    fn spend(&mut self, n: usize, tick: u64, cooldown: i64, haste: i64) {
        if tick >= self.free_clock(n) {
            self.set_free_clock(
                n,
                tick + (cooldown * 100 / (100 + haste.clamp(-90, 900))) as u64,
            );
        } else {
            self.stock[n] = self.stock[n].saturating_sub(1);
        }
    }
    /// Gives back a charge whose carrier never appeared.
    fn refund(&mut self, n: usize, tick: u64) {
        if self.free_clock(n) > tick {
            self.set_free_clock(n, 0);
        } else {
            self.stock[n] = 1;
        }
    }
    /// Round 144: a reward returns the free charge now; it never adds a bought one.
    fn restore(&mut self, n: usize, tick: u64) {
        if self.free_clock(n) > tick {
            self.set_free_clock(n, tick);
        }
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
        self.air_protection_until = 0;
        self.air_protection_enemies = 0;
        self.last_launch = None;
        self.sync_air(0, &mut out);
        self.cast = None;
        self.plan = None;
        for g in self.grenades.drain(..) {
            out.push(Cmd::ClearWarning { id: g.id });
            for n in 1..=5 {
                out.push(Cmd::ClearWarning {
                    id: g.id + (n << 40),
                });
            }
        }
        if let Some(r) = self.rocket.take() {
            out.push(Cmd::ClearWarning { id: r.id });
        }
        self.dodges.clear();
        self.armed_until = 0;
        self.armor = 0;
        self.was_alive = false;
        self.last_buy = None;
        // Omen's Buy Phase: bought charges are lost, the free ones come back immediately.
        self.stock = [0; 3];
        self.cooldowns[0] = 0;
        self.cooldowns[1] = 0;
        self.pack_cooldown = 0;
        self.guide = None;
        self.evade = None;
        self.sightings.clear();
        // Round 144: Hype, Carnaval and the air shield end with her; her paint and burning ground stay.
        self.hype = 0;
        self.carnaval_until = 0;
        self.air_shield = None;
        self.explosions.clear();
        self.hot_hands = 0;
        self.pushes.clear();
        out
    }
    /// Round 144: a kill or assist. Hype, Hot Hands' Paint Shells charge, armour, the Showstopper refund, and a
    /// Blast Pack charge back when she damaged the victim (any champion, for an assist) within RESET_WINDOW.
    fn takedown(&mut self, f: &Frame, me: &Unit, victim: usize, out: &mut Vec<Cmd>) {
        self.gain_hype(HYPE_TAKEDOWN, f.tick);
        self.hot_hands = 0;
        self.restore(1, f.tick);
        let recent = |t: &u64| f.tick <= t + RESET_WINDOW as u64;
        let reset = if victim == 0 {
            self.last_hits.values().any(recent)
        } else {
            self.last_hits.get(&victim).is_some_and(recent)
        };
        if reset {
            self.restore(0, f.tick);
        }
        if self.cooldowns[2] > f.tick {
            self.cooldowns[2] = self.cooldowns[2]
                .saturating_sub((ROCKET_COOLDOWN * ULT_TAKEDOWN_REFUND / 100) as u64)
                .max(f.tick);
        }
        if self.armor > 0 {
            let full = me.max_hp * if self.armor == 2 { 25 } else { 12 } / 100;
            let amount = (f.shield + full * TAKEDOWN_ARMOR_PERCENT / 100).min(full.max(f.shield));
            if amount > f.shield {
                out.push(Cmd::Armor { amount });
            }
        }
        self.last_hits.remove(&victim);
    }
    /// Round 144: the per-tick upkeep of her new kit: the level-6 Sheriff, Hype and Carnaval, Graffiti slows,
    /// Showstopper burns and the air shield.
    fn upkeep(&mut self, f: &Frame, me: &Unit, out: &mut Vec<Cmd>) {
        if self.level >= FREE_SHERIFF_LEVEL && self.gun == START_GUN as usize && !self.sheriff_given {
            self.sheriff_given = true;
            self.gun = Gun::Sheriff.idx();
            out.push(Cmd::Gun { index: self.gun });
        }
        if self.hype >= HYPE_MAX && !self.carnaval(f.tick) {
            // A Gold-or-better athlete saves a full meter for a fight or an objective; others burst at once.
            let fight = f.objective.is_some()
                || f.visible_enemies(me)
                    .any(|u| u.champion() && distance(u.pos, me.pos) <= CARNAVAL_FIGHT_RANGE);
            if !(f.automatic && self.mastery as i64 >= CARNAVAL_HOLD_RANK && !fight) {
                self.hype = 0;
                self.carnaval_until = f.tick + CARNAVAL_TICKS as u64;
                out.push(Cmd::Visual {
                    id: 0,
                    tag: "carnaval".into(),
                    pos: me.pos,
                    height: 0,
                    until: f.tick + 30,
                });
            }
        } else if f.tick > self.last_combat + HYPE_COMBAT_TICKS as u64
            && f.tick % HYPE_DECAY_TICKS as u64 == 0
        {
            self.hype = (self.hype - 1).max(0);
        }
        if self.carnaval(f.tick) {
            // skill cooldowns recover twice as fast (not the Showstopper's)
            for n in 0..3 {
                let c = self.free_clock(n);
                if c > f.tick {
                    self.set_free_clock(n, c - 1);
                }
            }
        }
        self.paints.retain(|z| f.tick < z.until);
        for z in &self.paints {
            if (f.tick - z.born) % 6 == 0 {
                for u in f.units.iter().filter(|u| {
                    u.team != me.team && u.damageable() && distance(u.pos, z.pos) <= z.radius
                }) {
                    out.push(Cmd::Slow {
                        target: u.id,
                        percent: GRAFFITI_SLOW,
                        ticks: 6,
                    });
                }
            }
        }
        let zones = std::mem::take(&mut self.burn_zones);
        for z in &zones {
            if f.tick > z.born && (f.tick - z.born) % TPS as u64 == 0 {
                for u in f.units.iter().filter(|u| {
                    u.team != me.team && u.damageable() && distance(u.pos, z.pos) <= z.radius
                }) {
                    self.hit(f, z.id, u, z.attack * BURN_ZONE_ATTACK / 100, "burn", out);
                }
            }
        }
        self.burn_zones = zones.into_iter().filter(|z| f.tick < z.until).collect();
        self.keep_air_shield(f, out);
        self.burns.retain(|_, (_, until)| f.tick < *until);
        self.primed.retain(|_, until| f.tick <= *until);
        self.pushes
            .retain(|_, t| f.tick <= *t + COMBO_WINDOW as u64);
        self.paint_blasts
            .retain(|(_, t)| f.tick <= t + COMBO_WINDOW as u64);
        self.last_hits
            .retain(|_, t| f.tick <= *t + RESET_WINDOW as u64);
    }
    /// Round 144: one skill hit on an enemy. Farm, structure, Graffiti and Primed bonuses, then Hype, Hot Hands,
    /// Combustion and omnivamp. Skill damage stays exactly 3x the original after these.
    fn hit(&mut self, f: &Frame, id: u64, u: &Unit, raw: i64, source: &str, out: &mut Vec<Cmd>) {
        if raw <= 0 {
            return;
        }
        let mut raw = raw;
        if source == "cluster" && matches!(u.kind, 1 | 2) {
            raw = raw * CLUSTER_FARM_PERCENT / 100;
        }
        if source == "bot" && matches!(u.kind, 3 | 4) {
            raw = raw * BOT_STRUCTURE_PERCENT / 100;
        }
        if self
            .paints
            .iter()
            .any(|z| f.tick < z.until && distance(z.pos, u.pos) <= z.radius)
        {
            raw = raw * (100 + GRAFFITI_AMPLIFY) / 100;
        }
        if matches!(source, "grenade" | "rocket")
            && self.primed.remove(&u.id).is_some_and(|t| f.tick <= t)
        {
            raw = raw * (100 + PRIMED_PERCENT) / 100;
        }
        let champion = u.champion();
        let vamp = if champion && (self.air_protection || self.air_shield.is_some()) {
            OMNIVAMP_PROTECTED
        } else {
            OMNIVAMP
        };
        self.emit_damage(id, u.id, skill_raw(raw), false, source, vamp, false, out);
        if champion {
            self.gain_hype(HYPE_CHAMPION_HIT, f.tick);
            self.last_hits.insert(u.id, f.tick);
            if source == "bot" {
                self.primed.insert(u.id, f.tick + PRIMED_TICKS as u64);
            }
            if source == "grenade" {
                self.hot_hands += 1;
                if self.hot_hands as i64 >= HOT_HANDS_HITS {
                    self.hot_hands = 0;
                    self.restore(1, f.tick);
                }
            }
            if source != "burn" {
                let e = self.burns.entry(u.id).or_insert((0, 0));
                if f.tick >= e.1 {
                    *e = (0, 0);
                }
                e.0 += 1;
                e.1 = f.tick + BURN_TICKS as u64;
                if e.0 as i64 >= BURN_STACKS {
                    self.burns.remove(&u.id);
                    self.emit_damage(
                        id,
                        u.id,
                        u.max_hp * BURN_MAX_HP_PERCENT / 100,
                        false,
                        "combustion",
                        vamp,
                        true,
                        out,
                    );
                    out.push(Cmd::Reveal {
                        target: u.id,
                        ticks: BURN_REVEAL_TICKS as u64,
                    });
                    out.push(Cmd::Visual {
                        id,
                        tag: "combustion".into(),
                        pos: u.pos,
                        height: 0,
                        until: f.tick + 18,
                    });
                }
            }
        } else if matches!(u.kind, 1..=3) {
            self.gain_hype(HYPE_UNIT_HIT, f.tick);
        }
    }
    /// Round 143: exactly Omen's buy (valorant.rs `Omen::buy`): the gun first, armour only from what the Vandal's
    /// price leaves until she owns one, then one bought charge per utility (his second smoke).
    pub fn entry_review(
        &self,
        f: &Frame,
        me: &Unit,
        target: &Unit,
        landing: Point,
        spends: u8,
        rocket: bool,
    ) -> EntryReview {
        let mut r = EntryReview::default();
        r.reason = "No useful legal landing".into();
        if !target.visible
            || !target.alive()
            || !target.targetable
            || !target.champion()
            || f.blocked(landing, BODY_RADIUS)
            || f.sweep(me.pos, landing, BODY_RADIUS).1
        {
            return r;
        }
        let gun = Gun::from_index(self.gun);
        if distance(landing, target.pos) > gun.reach() + 10000 && !rocket {
            return r;
        }
        let flight = (distance(me.pos, landing) * TPS / LAUNCH_SPEED).min(MAX_FLIGHT_TICKS) as u64;
        let reduce = |raw: i64, def: i64| raw.max(0) * 100 / (100 + def.max(0));
        for enemy in f.units.iter().filter(|u| {
            u.team != me.team && u.visible && u.alive() && (u.champion() || u.kind == 4)
        }) {
            let range = if enemy.kind == 4 {
                90000
            } else {
                enemy
                    .attack_range
                    .max(if enemy.melee { 23000 } else { 30000 })
            };
            let speed = enemy.move_speed.max(1);
            let reach = (distance(enemy.pos, landing) - range).max(0) / speed;
            if reach <= ENTRY_REVIEW_TICKS {
                let hits = 1 + (ENTRY_REVIEW_TICKS - reach) / 60;
                r.incoming += reduce(enemy.attack * hits, me.defence);
                if enemy.champion() {
                    r.incoming += reduce(enemy.ap / 2, me.mr);
                }
            }
        }
        for h in f.hazards.iter().filter(|h| {
            h.until > f.tick + flight && distance(h.pos, landing) < h.radius + BODY_RADIUS
        }) {
            r.incoming += me.max_hp * h.severity.max(0) / 100;
        }
        if rocket {
            r.incoming += reduce(
                rocket_damage(
                    self.base(ROCKET_DAMAGE) + me.attack * ROCKET_ATTACK / 100,
                    distance(landing, target.pos),
                ) * SELF_DAMAGE_PERCENT
                    / 100,
                me.defence,
            );
        }
        r.predicted_hp = me.hp + f.shield - r.incoming;
        if r.predicted_hp < me.max_hp * ENTRY_SURVIVE_PERCENT / 100 {
            r.reason = "Landing survival below 20% HP".into();
            return r;
        }
        let range_to = |ally: &Unit| {
            let range = ally.attack_range;
            if range <= 0 || ally.controlled {
                return None;
            }
            let d = distance(ally.pos, target.pos);
            let fire_from = if d > range {
                target
                    .pos
                    .toward(ally.pos, range.saturating_sub(BODY_RADIUS))
            } else {
                ally.pos
            };
            if f.sweep(ally.pos, fire_from, BODY_RADIUS).1 || f.sweep(fire_from, target.pos, 1000).1
            {
                return None;
            }
            let travel = distance(ally.pos, fire_from);
            if travel > 0 && ally.move_speed <= 0 {
                return None;
            }
            let eta = (travel + ally.move_speed.max(1) - 1) / ally.move_speed.max(1);
            (eta as u64 <= flight + ENTRY_REVIEW_TICKS as u64)
                .then_some(((eta - flight as i64).max(0) as u64, ally.id))
        };
        if let Some((eta, id)) = f
            .units
            .iter()
            .filter(|u| u.team == me.team && u.id != me.id && u.champion() && u.alive())
            .filter_map(range_to)
            .min()
        {
            r.ally = Some(id);
            r.ally_eta = Some(eta);
        }
        let away = landing.sub(target.pos);
        let away = if away.length() == 0 {
            me.pos.sub(target.pos).add(Point::new(1, 0))
        } else {
            away
        };
        let exits = [
            me.pos,
            landing
                .add(away.scale(75000, away.length().max(1)))
                .bounded(),
            landing
                .add(Point::new(-away.y, away.x).scale(75000, away.length().max(1)))
                .bounded(),
            landing
                .add(Point::new(away.y, -away.x).scale(75000, away.length().max(1)))
                .bounded(),
        ];
        r.escape = exits
            .into_iter()
            .filter(|p| {
                distance(landing, *p) >= 24000
                    && !f.sweep(landing, *p, BODY_RADIUS).1
                    && !f.hazards.iter().any(|h| {
                        h.until > f.tick + flight && distance(h.pos, *p) <= h.radius + BODY_RADIUS
                    })
                    && !f
                        .units
                        .iter()
                        .filter(|u| {
                            u.team != me.team
                                && u.visible
                                && u.alive()
                                && (u.champion() || u.kind == 4)
                        })
                        .any(|u| {
                            distance(u.pos, *p)
                                < if u.kind == 4 {
                                    90000
                                } else {
                                    u.attack_range.max(if u.melee { 23000 } else { 30000 })
                                        + BODY_RADIUS
                                }
                        })
            })
            .min_by_key(|p| (distance(landing, *p), p.x, p.y));
        // A ready pack at takeoff can pay for a queued launch. Remote deployed
        // packs are not an escape reserve: she cannot carry them to the landing.
        let staged = self
            .packs
            .iter()
            .filter(|p| {
                p.ready && p.hp > 0 && p.until > f.tick && distance(me.pos, p.pos) <= PACK_RADIUS
            })
            .count()
            .min(1) as u8;
        r.retained_packs = self
            .charges(0, f.tick)
            .saturating_sub(spends.saturating_sub(staged));
        let accuracy = if target.controlled {
            90
        } else {
            45 + f.profile.get(4) * 35 / 100
        };
        let d = distance(landing, target.pos);
        if d <= gun.reach() {
            let hits = (ENTRY_REVIEW_TICKS * (100 + gun.modifiers().1 as i64) / 6000).max(1);
            r.expected_damage += reduce(
                crate::valorant_guns::current_basic_raw(gun, me.attack, d) * hits,
                target.defence,
            );
        }
        if self.charges(1, f.tick) > 0 && d <= GRENADE_RANGE {
            let impact = f.ricochet(landing, target.pos.sub(landing), d, 1000).0;
            let delay = GRENADE_PREP + GRENADE_FUSE + d * TPS / GRENADE_SPEED;
            if delay <= ENTRY_REVIEW_TICKS && distance(impact, target.pos) <= GRENADE_RADIUS {
                r.expected_damage += reduce(
                    skill_raw(self.base(GRENADE_DAMAGE) + me.attack * GRENADE_ATTACK / 100) * accuracy / 100,
                    target.defence,
                );
            }
        }
        if self.charges(2, f.tick) > 0
            && self.bot.is_none()
            && BOT_PREP + BOT_FUSE + d * TPS / BOT_CHASE_SPEED <= ENTRY_REVIEW_TICKS
        {
            r.expected_damage += reduce(
                skill_raw(self.base(BOT_DAMAGE) + me.attack * BOT_ATTACK / 100) * accuracy / 100,
                target.defence,
            );
        }
        if rocket && f.unlocked[2] && (self.armed_until > f.tick || f.tick >= self.cooldowns[2]) {
            let from = me.pos.toward(landing, distance(me.pos, landing).min(70000));
            let end = from.toward(target.pos, distance(from, target.pos).min(ROCKET_RANGE));
            let impact = f.rocket_impact(me.team, from, end, true).0;
            r.expected_damage += reduce(
                skill_raw(rocket_damage(
                    self.base(ROCKET_DAMAGE) + me.attack * ROCKET_ATTACK / 100,
                    distance(impact, target.pos),
                )) * accuracy
                    / 100,
                target.defence,
            );
        }
        r.solo_pick = r.expected_damage >= (target.hp + target.shield) * 110 / 100
            && r.escape.is_some()
            && !f
                .visible_enemies(me)
                .any(|u| u.champion() && u.id != target.id && distance(u.pos, target.pos) <= 80000);
        r.allowed = r.ally.is_some() || r.retained_packs > 0 && r.escape.is_some() || r.solo_pick;
        r.reason = if r.ally.is_some() {
            "Champion follow-up within 1.5 seconds"
        } else if r.retained_packs > 0 && r.escape.is_some() {
            "Retained satchel and clear exit"
        } else if r.solo_pick {
            "Isolated finishing entry with walking retreat"
        } else {
            "No champion follow-up or viable retained escape"
        }
        .into();
        r
    }
    fn refresh_savings(&mut self, f: &Frame, me: &Unit) {
        let seen: Vec<_> = f
            .visible_enemies(me)
            .filter(|u| u.champion() && distance(u.pos, me.pos) <= 200000)
            .take(4)
            .collect();
        let close = if seen.is_empty() {
            true
        } else {
            seen.iter().any(|u| {
                let landing = u.pos.toward(me.pos, 9000).bounded();
                distance(me.pos, landing) <= LAUNCH_SPEED * FLIGHT_TICKS / TPS
                    && self.entry_review(f, me, u, landing, 1, false).allowed
            })
        };
        if f.tick < self.savings.review_at && close == self.savings.close {
            return;
        }
        let old = Gun::from_index(self.gun);
        let want = if close { Gun::Judge } else { Gun::Vandal };
        let range = if close { 9000 } else { 35000 };
        let value = |g: Gun| {
            if range > g.reach() {
                0
            } else {
                crate::valorant_guns::current_basic_raw(
                    g,
                    me.attack * (100 + g.modifiers().0 as i64)
                        / (100 + old.modifiers().0 as i64).max(1),
                    range,
                ) * (100 + g.modifiers().1 as i64)
            }
        };
        let improving =
            want != old && want.tier() >= old.tier() && value(want) * 100 >= value(old) * 120;
        self.savings = WeaponSaving {
            goal: improving.then_some(want.idx()),
            reserve: if improving { want.price() } else { 0 },
            review_at: f.tick + UPGRADE_TICKS as u64,
            close,
            reason: if improving {
                format!(
                    "Saving for {}: {}",
                    if close { "Judge" } else { "Vandal" },
                    if close {
                        "viable close entry"
                    } else {
                        "safer ranged pressure"
                    }
                )
            } else {
                "Current weapon meets this approach; surplus may buy protection and utility".into()
            },
        };
    }
    fn buy(&mut self, f: &Frame, me: &Unit, out: &mut Vec<Cmd>) {
        if !f.buy_allowed || self.flight.is_some() || self.cast.is_some() {
            self.reject(out, "Buying requires the same safe location as Omen");
            return;
        }
        let mut neutral;
        let f = if f.automatic {
            f
        } else {
            neutral = f.clone();
            neutral.profile = Profile::default();
            &neutral
        };
        self.refresh_savings(f, me);
        let before = self.credits;
        if let Some(gun) = self.savings.goal.map(Gun::from_index).filter(|g| {
            self.credits >= g.price()
                && self
                    .last_weapon_buy
                    .is_none_or(|t| f.tick >= t + UPGRADE_TICKS as u64)
        }) {
            self.credits -= gun.price();
            self.gun = gun.idx();
            self.last_weapon_buy = Some(f.tick);
            out.push(Cmd::Gun { index: self.gun });
            self.savings.goal = None;
            self.savings.reserve = 0;
            self.savings.reason = "Weapon purchased; spending only the remaining credits".into();
        }
        let need = |level: usize| me.max_hp * if level == 2 { 25 } else { 12 } / 100;
        let spare = self.credits - self.savings.reserve;
        let level = if spare >= 1000 {
            2
        } else if spare >= 400 {
            1
        } else {
            0
        };
        if level > 0 && (self.armor < level || f.shield < need(self.armor.max(1)) / 2) {
            self.credits -= if level == 2 { 1000 } else { 400 };
            self.armor = level;
            out.push(Cmd::Armor {
                amount: need(level),
            });
        }
        for (n, price) in [(0, PACK_PRICE), (1, GRENADE_PRICE), (2, BOT_PRICE)] {
            if self.stock[n] == 0 && self.credits - self.savings.reserve >= price {
                self.credits -= price;
                self.stock[n] = 1;
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
            self.stock = [1, 1, 1];
        }
        self.last_tick = Some(f.tick);
        let Some(me) = f.me().cloned() else {
            return out;
        };
        // Round 144: mastery sharpens her aim, timing and plans (Skill Accuracy, Micro and Strategy move toward
        // 100), never her damage or cooldowns.
        let mastered;
        let f = if f.automatic && f.mastery > 0 {
            let mut m = f.clone();
            for n in [4, 10, 12] {
                let v = m.profile.get(n);
                m.profile.values[n] = v + (100 - v) * f.mastery.min(7) as i64 / 16;
            }
            mastered = m;
            &mastered
        } else {
            f
        };
        self.level = f.level.max(1);
        self.mastery = f.mastery.min(MASTERY_RANKS as u8 - 1);
        let mut takedowns = vec![];
        for e in &f.credits {
            if e.id > self.credit_floor && self.seen_credits.insert(e.id) {
                self.earn(match e.kind {
                    0 => KILL_CREDITS + KILL_BOUNTY,
                    1 => ASSIST_CREDITS,
                    2 => TEAM_KILL_CREDITS,
                    3 => DEATH_CREDITS,
                    _ => 0,
                });
                if e.kind <= 1 {
                    takedowns.push(if e.kind == 0 { e.victim } else { 0 });
                }
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
        for victim in takedowns {
            self.takedown(f, &me, victim, &mut out);
        }
        self.upkeep(f, &me, &mut out);
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
                    self.refund(0, f.tick);
                    self.reject(&mut out, "Satchel carrier spawn failed; charge refunded");
                } else {
                    self.packs[n].ready = true;
                    self.packs[n].hp = s.hp;
                }
            }
            if self.bot.as_ref().is_some_and(|b| b.id == s.id) {
                if s.hp < 0 {
                    self.bot = None;
                    self.refund(2, f.tick);
                    self.reject(&mut out, "Boom Bot carrier spawn failed; charge restored");
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
        let mut retained = vec![];
        for mut p in std::mem::take(&mut self.packs) {
            if let Some(id) = p.attachment.surface {
                let follow = f
                    .surfaces
                    .iter()
                    .find(|s| s.id == id)
                    .map(|s| {
                        let point = s.bounds.min.add(p.attachment.offset);
                        let pos = point.add(p.attachment.normal.scale(PACK_CARRIER_RADIUS + 2, 1));
                        (point, pos)
                    })
                    .filter(|(_, pos)| !f.blocked(*pos, PACK_CARRIER_RADIUS));
                if let Some((point, pos)) = follow {
                    if p.pos != pos {
                        p.pos = pos;
                        out.push(Cmd::MoveUnit { id: p.id, to: pos });
                    }
                    p.attachment.point = point;
                } else if let Some(pos) = f.drop_pack(p.pos) {
                    p.pos = pos;
                    p.attachment = Attachment {
                        surface: None,
                        kind: "ground".into(),
                        point: pos,
                        normal: Point::default(),
                        offset: Point::default(),
                    };
                    out.push(Cmd::MoveUnit { id: p.id, to: pos });
                } else {
                    out.push(Cmd::Remove { id: p.id });
                    continue;
                }
            }
            retained.push(p);
        }
        self.packs = retained;
        if self
            .bot
            .as_ref()
            .is_some_and(|b| b.hp <= 0 || f.tick >= b.until)
        {
            if let Some(b) = self.bot.take() {
                out.push(Cmd::Remove { id: b.id });
            }
        }
        if let Some(c) = self
            .cast
            .take_if(|c| (f.stunned || f.silenced) && f.tick < c.release)
        {
            self.metrics.wasted_preparation += 1;
            self.plan = None;
            // The ultimate is charged when the rocket fires, so an interrupted equip costs nothing.
            self.reject(
                &mut out,
                if matches!(c.action, Action::EquipRocket) {
                    "Rocket equip interrupted; the ultimate is not charged"
                } else {
                    "Preparation interrupted; the committed charge is spent"
                },
            );
        }
        let flying = self.airborne();
        if f.bound || f.stunned || self.flight.as_ref().is_some_and(|v| f.tick >= v.until) {
            self.finish_flight(f, &me, me.pos, flying, &mut out);
        }
        self.sync_air(f.tick, &mut out);
        if !f.automatic {
            if let (Some(v), Some(input)) = (self.flight.as_mut(), f.steering) {
                v.input = input.capped(1000);
            }
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
        if f.automatic {
            self.hold_guide(f, &me, flying, &mut out);
        }
        if self.air_protection {
            self.metrics.protected_ticks += 1;
        }
        out
    }
    /// Round 143: the ground movement is re-issued every tick from the current decision, held on arrival (no
    /// hand-back to the engine mid-plan), and after a landing it holds where she landed until the next decision.
    fn hold_guide(&mut self, f: &Frame, me: &Unit, flying: bool, out: &mut Vec<Cmd>) {
        if flying && !self.airborne() {
            let at = out
                .iter()
                .rev()
                .find_map(|c| match c {
                    Cmd::MoveOwner { to } => Some(*to),
                    _ => None,
                })
                .unwrap_or(me.pos);
            self.guide = Some(Guide {
                family: "landing".into(),
                target: None,
                to: at,
            });
        }
        if self.flight.is_some() || f.stunned || f.bound {
            return;
        }
        let Some(mut g) = self.guide.take() else {
            return;
        };
        match g.family.as_str() {
            "shoot" => {
                if let Some(u) = f
                    .units
                    .iter()
                    .find(|u| Some(u.id) == g.target && u.visible && u.alive())
                {
                    let gun = Gun::from_index(self.gun);
                    let offset =
                        (gun.reach() * (60 + f.profile.get(2) * 25 / 100) / 100).max(10000);
                    g.to = u.pos.toward(me.pos, offset).bounded();
                }
            }
            "rotate" => match f.assignment {
                Some(to) => g.to = to,
                // team guidance handed the job back: the engine walks her again
                None => return,
            },
            _ => {}
        }
        g.to = f.free_point(me.pos, g.to);
        out.push(Cmd::Guide { to: g.to });
        self.guide = Some(g);
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
                if !f.unlocked[0] || self.charges(1, f.tick) == 0 {
                    self.reject(out, "Grenade locked, or no free or bought charge");
                    return false;
                }
                if distance(me.pos, *aim) > GRENADE_RANGE
                    || distance(me.pos, *aim) < 1000
                    || f.blocked(me.pos, 1000)
                {
                    self.reject(out, "Invalid grenade throw");
                    return false;
                }
                self.spend(1, f.tick, GRENADE_COOLDOWN, f.haste);
                GRENADE_PREP
            }
            Action::Pack { .. } | Action::DropPack => {
                let aim = match &a {
                    Action::Pack { aim } => *aim,
                    _ => me.pos,
                };
                if !f.unlocked[1] || self.charges(0, f.tick) == 0 || self.packs.len() >= 2 {
                    self.reject(
                        out,
                        "Satchel locked, no free or bought charge, or deployment limit reached",
                    );
                    return false;
                }
                if distance(me.pos, aim) > PACK_RANGE || f.pack_attachment(me.pos, aim).is_none() {
                    self.reject(out, "Satchel throw is obstructed");
                    return false;
                }
                self.spend(0, f.tick, PACK_COOLDOWN, f.haste);
                PACK_THROW
            }
            Action::Bot { aim } => {
                if !f.unlocked[0] || self.charges(2, f.tick) == 0 || self.bot.is_some() {
                    self.reject(out, "Bot locked, active, or no free or bought charge");
                    return false;
                }
                if f.blocked(me.pos, 1000) || distance(me.pos, *aim) < 1000 {
                    self.reject(out, "Invalid bot direction");
                    return false;
                }
                self.spend(2, f.tick, BOT_COOLDOWN, f.haste);
                BOT_PREP
            }
            Action::EquipRocket => {
                if !f.unlocked[2] || f.tick < self.cooldowns[2] || f.tick < self.armed_until {
                    self.reject(out, "Ultimate locked, armed or cooling down");
                    return false;
                }
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
                // Round 143: the cooldown starts when the rocket fires, not on the equip.
                self.cooldowns[2] =
                    f.tick + (ROCKET_COOLDOWN * 100 / (100 + f.ult_haste.clamp(-90, 900))) as u64;
                self.release_rocket(f, me, *aim, out);
                self.metrics.casts += 1;
                return true;
            }
            Action::Slug { target } => {
                let Some(u) = f.units.iter().find(|u| {
                    u.id == *target && u.team != me.team && u.visible && u.damageable()
                }) else {
                    self.reject(out, "No visible slug target");
                    return false;
                };
                if self.gun != Gun::Bucky.idx()
                    || f.tick < self.slug_ready
                    || distance(me.pos, u.pos) > SLUG_RANGE
                    || f.sweep(me.pos, u.pos, 1000).1
                {
                    self.reject(out, "Slug needs the Bucky, its cooldown and a clear line in range");
                    return false;
                }
                // A gun shot: physical, not a skill, so neither tripled nor feeding Hype or Combustion.
                self.slug_ready = f.tick + SLUG_COOLDOWN as u64;
                let id = self.id();
                self.emit_damage(id, u.id, me.attack * SLUG_ATTACK / 100, false, "slug", 0, false, out);
                out.push(Cmd::Tracer { to: u.pos });
                return true;
            }
            _ => return false,
        };
        self.metrics.casts += 1;
        let id = self.id();
        let (tag, p, r) = match &a {
            Action::Grenade { aim } => (
                "grenade_prep",
                f.ricochet(me.pos, aim.sub(me.pos), distance(me.pos, *aim), 1000)
                    .0,
                GRENADE_RADIUS,
            ),
            Action::Pack { aim } => ("pack_throw", *aim, PACK_RADIUS),
            Action::DropPack => ("pack_throw", me.pos, PACK_RADIUS),
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
            attack: me.attack,
            max_hp: me.max_hp,
        });
        true
    }
    fn release(&mut self, f: &Frame, me: &Unit, c: Cast, out: &mut Vec<Cmd>) {
        // Foot placement follows the caster through preparation, then stays put.
        let c = if matches!(c.action, Action::DropPack) {
            Cast {
                action: Action::Pack { aim: me.pos },
                ..c
            }
        } else {
            c
        };
        let id = self.id();
        match c.action {
            Action::Pack { aim } => {
                let attachment = f.pack_attachment(me.pos, aim);
                if distance(me.pos, aim) > PACK_RANGE || attachment.is_none() {
                    self.reject(out, "Satchel landing invalidated before release");
                    return;
                }
                let (pos, attachment) = attachment.unwrap();
                self.packs.push(Pack {
                    id,
                    pos,
                    born: f.tick,
                    until: f.tick + PACK_LIFETIME as u64,
                    hp: PACK_HP,
                    ready: false,
                    attachment,
                });
                out.push(Cmd::Spawn {
                    id,
                    bot: false,
                    pos,
                    hp: PACK_HP,
                    ticks: PACK_LIFETIME as u64,
                });
            }
            Action::Grenade { aim } => {
                let remaining = distance(me.pos, aim);
                let direction = aim.sub(me.pos).capped(1_000_000);
                let landing = f.ricochet(me.pos, direction, remaining, 1000).0;
                let travel = ((remaining * TPS + GRENADE_SPEED - 1) / GRENADE_SPEED).max(1) as u64;
                let arrival = f.tick + travel;
                self.grenades.push(Grenade {
                    id,
                    from: me.pos,
                    aim: landing,
                    arrival,
                    explosion: arrival + GRENADE_FUSE as u64,
                    cluster: arrival + (GRENADE_FUSE + CLUSTER_FUSE) as u64,
                    born: f.tick,
                    attack: c.attack,
                    main_done: false,
                    centres: vec![],
                    pos: me.pos,
                    direction,
                    remaining,
                    bounces: 0,
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
                    attack: c.attack,
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
                self.armed_at = f.tick;
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
            pos: p.attachment.point,
            height: 0,
            until: f.tick + 12,
        });
        self.explode(f.tick);
        for u in f.units.iter().filter(|u| {
            u.team != me.team
                && u.alive()
                && u.targetable
                && distance(u.pos, p.attachment.point) <= PACK_RADIUS
        }) {
            if f.tick >= p.born + PACK_ARM as u64 {
                self.hit(
                    f,
                    p.id,
                    u,
                    self.base(PACK_DAMAGE) + me.attack * PACK_ATTACK / 100,
                    "satchel",
                    out,
                );
            }
            let to = u.pos.toward(p.attachment.point, -PACK_PUSH);
            let (to, _) = f.sweep(u.pos, to, BODY_RADIUS);
            out.push(Cmd::Push { target: u.id, to });
            if u.champion() {
                // Round 144, the satchel combo: pushed into a fresh Paint Shells blast, the enemy is stunned
                self.pushes.insert(u.id, f.tick);
                if self
                    .paint_blasts
                    .iter()
                    .any(|(at, t)| f.tick <= t + COMBO_WINDOW as u64 && distance(*at, to) <= GRENADE_RADIUS)
                {
                    out.push(Cmd::Stun {
                        target: u.id,
                        ticks: COMBO_STUN as u64,
                    });
                }
            }
        }
        let centre = p.attachment.point;
        let profile = launch_profile(distance(me.pos, centre));
        self.last_launch = Some(profile.clone());
        if profile.ticks > 0 && !f.bound {
            let dir = me.pos.sub(centre);
            let impulse = dir.scale(profile.horizontal_speed, dir.length().max(1));
            let old = self.flight.take();
            let born = old
                .as_ref()
                .filter(|v| !v.grounded)
                .map_or(f.tick, |v| v.born);
            let velocity = old
                .as_ref()
                .map_or(me.velocity.add(impulse), |v| v.velocity.add(impulse))
                .capped(if old.as_ref().is_some_and(|v| v.boosted) {
                    ROCKET_MAX_SPEED
                } else {
                    MAX_SPEED
                });
            let until = (f.tick + profile.ticks)
                .max(old.as_ref().map_or(0, |v| v.until))
                .min(
                    born + if old.as_ref().is_some_and(|v| v.boosted) {
                        ROCKET_MAX_FLIGHT
                    } else {
                        MAX_FLIGHT_TICKS
                    } as u64,
                );
            let goal = self
                .plan
                .as_ref()
                .map_or(me.pos.add(velocity.scale(profile.ticks as i64, TPS)), |p| {
                    p.landing
                });
            self.flight = Some(Flight {
                satchel_count: old
                    .as_ref()
                    .filter(|v| !v.grounded)
                    .map_or(1, |v| v.satchel_count + 1),
                grounded: false,
                boosted: old.as_ref().is_some_and(|v| v.boosted),
                arc_born: f.tick,
                arc_height: old.as_ref().map_or(0, |v| v.height(f.tick)),
                arc_peak: profile.peak,
                velocity,
                remainder: Point::default(),
                born,
                until,
                input: Point::default(),
                next_input: f.tick,
                goal,
            });
            self.metrics.launches += 1;
            self.gain_hype(HYPE_LAUNCH, f.tick);
            self.grant_air_shield(f, me, until, out);
            self.sync_air(f.tick, out);
        }
        self.actual_detonation = Some(f.tick);
        true
    }
    fn advance_flight(&mut self, f: &Frame, me: &Unit, out: &mut Vec<Cmd>) {
        let Some(mut v) = self.flight.take() else {
            return;
        };
        if f.tick >= v.until || f.bound {
            self.finish_flight(f, me, me.pos, !v.grounded, out);
            return;
        }
        if f.automatic && !v.grounded && !f.stunned && f.tick >= v.next_input {
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
        if !f.stunned && !v.grounded && v.input.length() > 0 {
            // Accelerate toward the held direction, turned at most AIR_STRAFE_MAX_DEG from the heading
            // (round 143): an opposite input curves her round instead of reversing; releasing input keeps
            // all existing momentum.
            let mut speed = v.cap();
            if f.automatic {
                // Round 143: brake into the goal every tick: the speed that still stops in the distance left,
                // so she lands on it instead of overshooting and walking back.
                // (measured along the held direction: once past the goal she only brakes)
                let ahead = f.intent.unwrap_or(v.goal).sub(me.pos);
                let along = (ahead.x as i128 * v.input.x as i128
                    + ahead.y as i128 * v.input.y as i128)
                    / v.input.length().max(1) as i128;
                speed = speed.min(arrival_speed(along as i64));
            }
            let desired = strafe_limit(v.velocity, v.input.capped(1000).scale(speed, 1000));
            let acceleration = desired.sub(v.velocity).capped(AIR_ACCEL / TPS);
            v.velocity = v.velocity.add(acceleration).capped(v.cap());
        }
        v.remainder = v.remainder.add(v.velocity);
        let step = v.remainder.scale(1, TPS);
        v.remainder = v.remainder.sub(step.scale(TPS, 1));
        let (to, hit) = f.sweep(me.pos, me.pos.add(step), BODY_RADIUS);
        out.push(Cmd::MoveOwner { to });
        let height = v.height(f.tick);
        if !v.grounded {
            out.push(Cmd::Visual {
                id: 0,
                tag: "airborne".into(),
                pos: to,
                height,
                until: f.tick + 2,
            });
        }
        if hit {
            self.metrics.collisions += 1;
            out.push(Cmd::Visual {
                id: 0,
                tag: "landing".into(),
                pos: to,
                height: 0,
                until: f.tick + 8,
            });
            self.finish_flight(f, me, to, !v.grounded, out);
        } else {
            self.flight = Some(v)
        }
        self.sync_air(f.tick, out);
    }
    fn damage(
        &mut self,
        id: u64,
        target: usize,
        raw: i64,
        self_hit: bool,
        source: &str,
        out: &mut Vec<Cmd>,
    ) {
        let raw = if self_hit { raw } else { skill_raw(raw) };
        self.emit_damage(id, target, raw, self_hit, source, 0, false, out);
    }
    #[allow(clippy::too_many_arguments)]
    fn emit_damage(
        &mut self,
        id: u64,
        target: usize,
        raw: i64,
        self_hit: bool,
        source: &str,
        vamp: i64,
        fixed: bool,
        out: &mut Vec<Cmd>,
    ) {
        if raw > 0 {
            if !self_hit {
                self.metrics.requested_damage += raw;
                *self
                    .metrics
                    .damage_by_source
                    .entry(source.into())
                    .or_default() += raw;
            }
            out.push(Cmd::Damage {
                id,
                target,
                raw,
                self_hit,
                source: source.into(),
                vamp,
                fixed,
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
        source: &str,
        out: &mut Vec<Cmd>,
    ) {
        // Impact uses current targetable entities, including concealed enemies already caught by an area attack.
        // Round 144: the Boom Bot also hits structures.
        self.explode(f.tick);
        for u in f.units.iter().filter(|u| {
            u.team != me.team
                && (u.damageable() || source == "bot" && u.kind == 4 && u.targetable && u.alive())
        }) {
            let d = distance(p, u.pos);
            if d <= r {
                let amount = if falloff { rocket_damage(raw, d) } else { raw };
                self.hit(f, id, u, amount, source, out);
            }
        }
        if self_damage && distance(p, me.pos) <= r {
            let d = distance(p, me.pos);
            let amount = if falloff { rocket_damage(raw, d) } else { raw };
            self.damage(
                id,
                me.id,
                amount * SELF_DAMAGE_PERCENT / 100,
                true,
                source,
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
            if f.tick > g.born && g.remaining > 0 {
                let step = (GRENADE_SPEED / TPS).min(g.remaining);
                let (pos, direction, bounces) = f.ricochet(g.pos, g.direction, step, 1000);
                g.pos = pos;
                g.direction = direction;
                g.remaining -= step;
                g.bounces += bounces;
                let landing = f.ricochet(g.pos, g.direction, g.remaining, 1000).0;
                g.aim = landing;
            }
            if !g.main_done && f.tick < g.explosion {
                out.push(Cmd::Warning {
                    id: g.id,
                    tag: "grenade_warning".into(),
                    pos: g.aim,
                    radius: GRENADE_RADIUS,
                    until: g.explosion,
                });
            }
            if f.tick < g.arrival {
                let n = f.tick - g.born;
                let d = (g.arrival - g.born).max(1);
                out.push(Cmd::Visual {
                    id: g.id,
                    tag: "grenade_flight".into(),
                    pos: g.pos,
                    height: (32000 * n * (d - n) / (d * d)) as i64,
                    until: f.tick + 2,
                });
            }
            if f.tick >= g.explosion && !g.main_done {
                out.push(Cmd::ClearWarning { id: g.id });
                self.blast(
                    f,
                    me,
                    g.id,
                    g.aim,
                    GRENADE_RADIUS,
                    self.base(GRENADE_DAMAGE) + g.attack * GRENADE_ATTACK / 100,
                    true,
                    false,
                    "grenade",
                    out,
                );
                g.main_done = true;
                // Round 144, Graffiti: the main blast leaves a paint splash that slows and marks enemies.
                self.paints.push(Zone {
                    id: g.id,
                    pos: g.aim,
                    radius: GRENADE_RADIUS,
                    born: f.tick,
                    until: f.tick + GRAFFITI_TICKS as u64,
                    attack: 0,
                });
                out.push(Cmd::Visual {
                    id: g.id,
                    tag: "graffiti".into(),
                    pos: g.aim,
                    height: 0,
                    until: f.tick + GRAFFITI_TICKS as u64,
                });
                self.paint_blasts.push((g.aim, f.tick));
                // the satchel combo: an enemy a satchel pushed within COMBO_WINDOW is stunned by the blast
                for u in f.units.iter().filter(|u| {
                    u.team != me.team
                        && u.champion()
                        && u.damageable()
                        && distance(u.pos, g.aim) <= GRENADE_RADIUS
                        && self.pushes.contains_key(&u.id)
                }) {
                    out.push(Cmd::Stun {
                        target: u.id,
                        ticks: COMBO_STUN as u64,
                    });
                }
                let mut offsets = vec![
                    Point::new(CLUSTER_OFFSET, CLUSTER_OFFSET),
                    Point::new(-CLUSTER_OFFSET, CLUSTER_OFFSET),
                    Point::new(-CLUSTER_OFFSET, -CLUSTER_OFFSET),
                    Point::new(CLUSTER_OFFSET, -CLUSTER_OFFSET),
                ];
                // Carnaval drops an extra cluster in the middle
                if self.carnaval(f.tick) {
                    offsets.push(Point::default());
                }
                g.centres = offsets
                    .into_iter()
                    .map(|d| f.sweep(g.aim, g.aim.add(d), 1000).0)
                    .collect();
                for (n, p) in g.centres.iter().copied().enumerate() {
                    out.push(Cmd::Warning {
                        id: g.id + ((n as u64 + 1) << 40),
                        tag: "cluster_warning".into(),
                        pos: p,
                        radius: CLUSTER_RADIUS,
                        until: g.cluster,
                    });
                }
            }
            if f.tick >= g.cluster {
                let centres = &g.centres;
                let raw = CLUSTER_DAMAGE + g.attack * CLUSTER_ATTACK_NUM / CLUSTER_ATTACK_DEN;
                self.explode(f.tick);
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
                        self.hit(f, g.id, u, raw * n as i64, "cluster", out);
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
                        "cluster",
                        out,
                    );
                }
                for p in centres.iter().copied() {
                    out.push(Cmd::Visual {
                        id: g.id,
                        tag: "cluster_blast".into(),
                        pos: p,
                        height: 0,
                        until: f.tick + 12,
                    });
                }
            } else {
                if g.main_done {
                    for (n, p) in g.centres.iter().enumerate() {
                        let t = (f.tick - g.explosion).min(CLUSTER_FUSE as u64) as i64;
                        out.push(Cmd::Visual {
                            id: g.id + ((n as u64 + 1) << 40),
                            tag: "grenade_flight".into(),
                            pos: g.aim.add(p.sub(g.aim).scale(t, CLUSTER_FUSE)),
                            height: 0,
                            until: f.tick + 2,
                        });
                    }
                }
                self.grenades.push(g)
            }
        }
    }
    fn release_rocket(&mut self, f: &Frame, me: &Unit, aim: Point, out: &mut Vec<Cmd>) {
        self.metrics.rocket_shots += 1;
        if self.airborne() {
            self.metrics.rocket_entries += 1;
        }
        let id = self.id();
        let endpoint = me.pos.toward(
            aim.bounded(),
            distance(me.pos, aim.bounded()).min(ROCKET_RANGE),
        );
        let delta = endpoint.sub(me.pos);
        let direction = delta.scale(1000000, delta.length().max(1));
        let impact = f.rocket_impact(me.team, me.pos, endpoint, true).0;
        let impact_tick = f.tick + rocket_eta(distance(me.pos, impact), 0);
        self.rocket = Some(Rocket {
            id,
            pos: me.pos,
            direction,
            endpoint,
            remaining: delta.length(),
            attack: me.attack,
            born: f.tick,
            age: 0,
            speed: ROCKET_START_SPEED,
            impact,
            impact_tick,
        });
        out.push(Cmd::Visual {
            id,
            tag: "rocket_release".into(),
            pos: me.pos,
            height: 0,
            until: f.tick + 12,
        });
        out.push(Cmd::Warning {
            id,
            tag: "rocket_impact_warning".into(),
            pos: impact,
            radius: ROCKET_RADIUS,
            until: impact_tick,
        });
        if let Some(r) = self.rocket.clone() {
            self.guide_defenders(f, &r, out);
        }
        if !f.bound && !f.stunned {
            if let Some(v) = self.flight.as_mut().filter(|v| !v.grounded) {
                if v.satchel_count >= 2
                    && (v.velocity.x as i128 * direction.x as i128
                        + v.velocity.y as i128 * direction.y as i128)
                        < 0
                {
                    self.metrics.triple_boosts += 1;
                }
                let height = v.height(f.tick);
                v.velocity = v
                    .velocity
                    .sub(direction.scale(ROCKET_RECOIL, 1000000))
                    .capped(ROCKET_MAX_SPEED);
                v.until = (v.until + ROCKET_EXTEND as u64).min(v.born + ROCKET_MAX_FLIGHT as u64);
                v.boosted = true;
                v.arc_born = f.tick;
                v.arc_height = height;
            } else {
                self.flight = Some(Flight {
                    satchel_count: 0,
                    grounded: true,
                    boosted: false,
                    arc_born: f.tick,
                    arc_height: 0,
                    arc_peak: 0,
                    velocity: direction.scale(-GROUND_RECOIL_SPEED, 1000000),
                    remainder: Point::default(),
                    born: f.tick,
                    until: f.tick + GROUND_RECOIL_TICKS as u64,
                    input: Point::default(),
                    next_input: f.tick,
                    goal: me.pos.sub(
                        direction.scale(GROUND_RECOIL_SPEED * GROUND_RECOIL_TICKS / TPS, 1000000),
                    ),
                });
            }
        }
    }
    fn advance_rocket(&mut self, f: &Frame, me: &Unit, out: &mut Vec<Cmd>) {
        let Some(mut r) = self.rocket.take() else {
            return;
        };
        let travel = rocket_tick_distance(r.age).min(r.remaining);
        r.speed = (ROCKET_START_SPEED
            + (ROCKET_SPEED - ROCKET_START_SPEED) * (r.age as i64 + 1).min(ROCKET_ACCEL_TICKS)
                / ROCKET_ACCEL_TICKS)
            .min(ROCKET_SPEED);
        r.age += 1;
        let end = if travel >= r.remaining {
            r.endpoint
        } else {
            r.pos.add(r.direction.scale(travel, 1000000))
        };
        let (point, blocked) = f.rocket_impact(me.team, r.pos, end, false);
        r.remaining -= travel;
        let impact = (blocked || r.remaining <= 0).then_some(point);
        if let Some(p) = impact {
            out.push(Cmd::ClearWarning { id: r.id });
            self.dodges.clear();
            self.metrics.rocket_objective_hits += f
                .units
                .iter()
                .filter(|u| {
                    u.kind == 3 && u.alive() && u.targetable && distance(u.pos, p) <= ROCKET_RADIUS
                })
                .count() as u64;
            self.blast(
                f,
                me,
                r.id,
                p,
                ROCKET_RADIUS,
                self.base(ROCKET_DAMAGE) + r.attack * ROCKET_ATTACK / 100,
                true,
                true,
                "rocket",
                out,
            );
            // Round 144: the impact scorches the ground and leaves a burning zone.
            self.burn_zones.push(Zone {
                id: r.id,
                pos: p,
                radius: BURN_ZONE_RADIUS,
                born: f.tick,
                until: f.tick + BURN_ZONE_TICKS as u64,
                attack: r.attack,
            });
            for (tag, ticks) in [("rocket_scorch", SCORCH_TICKS), ("burn_zone", BURN_ZONE_TICKS)] {
                out.push(Cmd::Visual {
                    id: r.id,
                    tag: tag.into(),
                    pos: p,
                    height: 0,
                    until: f.tick + ticks as u64,
                });
            }
            for u in f.units.iter().filter(|u| {
                u.team != me.team
                    && u.damageable()
                    && matches!(u.kind, 0 | 1 | 5)
                    && !u.push_immune
                    && distance(u.pos, p) <= ROCKET_RADIUS
            }) {
                let push = rocket_damage(ROCKET_PUSH, distance(u.pos, p));
                let dir = if u.pos == p {
                    r.direction
                } else {
                    u.pos.sub(p)
                };
                let to = f
                    .sweep(
                        u.pos,
                        u.pos.add(dir.scale(push, dir.length().max(1))),
                        BODY_RADIUS,
                    )
                    .0;
                if to != u.pos {
                    out.push(Cmd::Knockback {
                        target: u.id,
                        to,
                        ticks: ROCKET_PUSH_TICKS as u64,
                    });
                }
            }
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
            r.impact = f.rocket_impact(me.team, r.pos, r.endpoint, true).0;
            r.impact_tick = f.tick + rocket_eta(distance(r.pos, r.impact), r.age);
            out.push(Cmd::Warning {
                id: r.id,
                tag: "rocket_impact_warning".into(),
                pos: r.impact,
                radius: ROCKET_RADIUS,
                until: r.impact_tick,
            });
            self.guide_defenders(f, &r, out);
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
    fn guide_defenders(&mut self, f: &Frame, r: &Rocket, out: &mut Vec<Cmd>) {
        for d in &f.defenders {
            let Some(u) = f
                .units
                .iter()
                .find(|u| u.id == d.id && u.alive() && u.champion())
            else {
                continue;
            };
            if self.dodges.get(&d.id).is_none_or(|m| m.rocket != r.id) {
                if !d.observed {
                    continue;
                }
                self.dodges.insert(
                    d.id,
                    DodgeMemory {
                        rocket: r.id,
                        noticed: f.tick,
                        ready: f.tick + d.profile.reaction(),
                        responds: hash(f.seed ^ r.id ^ d.id as u64) % 100
                            < (50 + d.profile.get(5)).min(100) as u64,
                        goal: None,
                        reason: "Recognised rocket; reacting".into(),
                    },
                );
            }
            let m = self.dodges.get_mut(&d.id).unwrap();
            m.goal = None;
            if !m.responds {
                m.reason = "Missed warning".into();
                continue;
            }
            if f.tick < m.ready {
                m.reason = "Reaction pending".into();
                continue;
            }
            if u.controlled {
                m.reason = "Movement disabled".into();
                continue;
            }
            if distance(u.pos, r.impact) > ROCKET_RADIUS + BODY_RADIUS {
                m.reason = "Outside blast".into();
                continue;
            }
            let delta = if u.pos == r.impact {
                Point::new(-r.direction.y, r.direction.x)
            } else {
                u.pos.sub(r.impact)
            };
            let mut dirs = vec![
                delta,
                Point::new(-delta.y, delta.x),
                Point::new(delta.y, -delta.x),
            ];
            dirs.extend([
                Point::new(1, 0),
                Point::new(-1, 0),
                Point::new(0, 1),
                Point::new(0, -1),
                Point::new(1, 1),
                Point::new(1, -1),
                Point::new(-1, 1),
                Point::new(-1, -1),
            ]);
            let speed = if u.move_speed > 0 { u.move_speed } else { 1020 };
            let remaining = r.impact_tick.saturating_sub(f.tick) as i64;
            let choice = dirs
                .into_iter()
                .map(|dir| {
                    r.impact
                        .add(dir.scale(ROCKET_RADIUS + BODY_RADIUS + 2000, dir.length().max(1)))
                        .bounded()
                })
                .filter(|p| !f.blocked(*p, BODY_RADIUS) && !f.sweep(u.pos, *p, BODY_RADIUS).1)
                .min_by_key(|p| {
                    let reachable = distance(u.pos, *p) <= speed * remaining;
                    let other = f
                        .hazards
                        .iter()
                        .filter(|h| {
                            h.id != r.id
                                && h.until > f.tick
                                && distance(h.pos, *p) < h.radius + BODY_RADIUS
                        })
                        .map(|h| h.severity * 10000)
                        .sum::<i64>();
                    let crosses = segment_near(r.pos, r.endpoint, *p, 8000);
                    (if reachable { 0 } else { 1_000_000 })
                        + distance(u.pos, *p)
                        + other
                        + if crosses { 20000 } else { 0 }
                });
            if let Some(to) = choice {
                m.reason = if distance(u.pos, to) <= speed * remaining {
                    "Escape reachable"
                } else {
                    "Late escape; reduce blast exposure"
                }
                .into();
                m.goal = Some(to);
                out.push(Cmd::EnemyGuide {
                    target: u.id,
                    rocket: r.id,
                    to,
                    until: r.impact_tick,
                });
            } else {
                m.reason = "No clear escape route".into();
            }
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
                && distance(u.pos, b.pos)
                    <= if b.target == Some(u.id) {
                        BOT_TRACK_RANGE
                    } else {
                        BOT_SIGHT
                    }
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
            .min_by_key(|u| (b.target != Some(u.id), distance(u.pos, b.pos), u.id));
        // Round 144: with no enemy champion in sight it hunts the nearest wave, camp or structure.
        let target = target.or_else(|| {
            if f.visible_enemies(me)
                .any(|u| u.champion() && distance(u.pos, b.pos) <= BOT_TRACK_RANGE)
            {
                return None;
            }
            f.units
                .iter()
                .filter(|u| {
                    u.team != me.team
                        && u.visible
                        && u.alive()
                        && u.targetable
                        && !u.champion()
                        && distance(u.pos, b.pos) <= BOT_TRACK_RANGE
                        && !f.sweep(b.pos, u.pos, 1000).1
                })
                .min_by_key(|u| (distance(u.pos, b.pos), u.id))
        });
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
                self.base(BOT_DAMAGE) + b.attack * BOT_ATTACK / 100,
                false,
                false,
                "bot",
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
            let (to, direction, _) = f.ricochet(b.pos, b.direction, speed / TPS, 2000);
            b.direction = direction;
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
            if !f.automatic {
                continue;
            }
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
            .filter(|u| {
                u.champion()
                    || (u.farmable && f.farm_window)
                    || Some(u.id) == f.objective && u.kind == 3
            })
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
                u.team == me.team
                    && u.id != me.id
                    && u.champion()
                    && u.alive()
                    && distance(u.pos, me.pos) < 150000
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
            // Round 143: the rocket's crowd is counted over its own 50,000 blast, not the grenade's 30,000.
            let blast_cluster = f
                .visible_enemies(me)
                .filter(|v| distance(v.pos, aim) < ROCKET_RADIUS)
                .count() as i64;
            let is_wave = u.kind == 1;
            let farm_value = if farm {
                p.get(9) * (cluster * 4 + if Some(u.id) == f.objective { 18 } else { 0 }) / 100
            } else {
                0
            };
            let useful_clear = !farm || cluster >= GRENADE_FARM_MIN || u.kind == 2 || u.kind == 3;
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
                        && a.champion()
                        && a.alive()
                        && distance(a.pos, landing) < 90000
                })
                .count() as i64;
            // The authored two-pack sequence fires before reaching its final
            // landing. Do not price a shot from a destination not yet reached.
            let entry_shot = me.pos.toward(landing, distance(me.pos, landing).min(70000));
            // Round 143 takeoff gate (as Levi's): no launch more than 90 degrees away from where the game is
            // walking her, unless she is already there. Escapes are exempt.
            let walk_agrees = f.walk.map_or(true, |w| {
                let (a, b) = (landing.sub(me.pos), w.sub(me.pos));
                distance(me.pos, w) < 20000
                    || a.x as i128 * b.x as i128 + a.y as i128 * b.y as i128 >= 0
            });
            let packs = self.charges(0, f.tick) as i64;
            let armed = self.armed_until > f.tick;
            for family in [
                "shoot",
                "slug",
                "grenade",
                "bot",
                "single_launch",
                "double_launch",
                "rocket",
                "rocket_entry",
                "triple_boost",
                "escape",
            ] {
                let entry = family == "single_launch"
                    || family == "double_launch"
                    || family == "rocket_entry"
                    || family == "triple_boost";
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
                    "shoot" => {
                        target_value
                            + if dist <= gun.reach() {
                                10 + crate::valorant_guns::current_basic_raw(gun, me.attack, dist)
                                    / 15
                            } else {
                                -20
                            }
                    }
                    "grenade" => {
                        target_value + cluster * 8 + farm_value - 8
                            + skill_raw(self.base(GRENADE_DAMAGE) + me.attack * GRENADE_ATTACK / 100) / 12
                    }
                    "bot" => {
                        target_value + if dist > 50000 { 0 } else { 8 } - 12
                            + skill_raw(self.base(BOT_DAMAGE) + me.attack * BOT_ATTACK / 100) / 12
                    }
                    "slug" => target_value - 6 + me.attack * SLUG_ATTACK / 100 / 12,
                    "single_launch" => target_value + if dist > gun.reach() { 18 } else { -12 },
                    "double_launch" => target_value + if dist > 85000 { 25 } else { -20 },
                    "rocket" => {
                        target_value + blast_cluster * 18 + target_low / 3 - 22
                            + if self.airborne() { 20 } else { 0 }
                            + if armed { 60 } else { 0 }
                    }
                    "rocket_entry" => {
                        target_value + blast_cluster * 18 + target_low / 3 - 12
                            + if armed { 60 } else { 0 }
                    }
                    "triple_boost" => {
                        target_value
                            + (dist - 100000) / 3000
                            + (self.danger(f, me, me.pos) - self.danger(f, me, landing)) * 2
                            - 35
                    }
                    "escape" => {
                        self.danger(f, me, me.pos) * 2 + (55 - me.hp * 100 / me.max_hp.max(1)) * 2
                    }
                    _ => 0,
                };
                if entry {
                    // Fifteen percent protection applies to travel, never the exposed landing.
                    benefit += self.danger(f, me, me.pos) * AIR_REDUCTION / 200;
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
                // Round 144: skills on champions build Hype (worth more near a full meter) and are cheap while
                // Carnaval recovers them twice as fast; the rocket is kept for objective fights.
                if champion && matches!(family, "grenade" | "bot" | "rocket" | "rocket_entry") {
                    benefit += self.hype * 10 / HYPE_MAX
                        + if self.carnaval(f.tick) { 10 } else { 0 };
                }
                if family == "rocket" && champion && f.objective.is_some() {
                    benefit += 15;
                }
                if (family == "rocket" || family == "rocket_entry") && champion {
                    benefit += p.get(12) * blast_cluster / 10;
                    if distance(if entry { landing } else { me.pos }, aim) < ROCKET_RADIUS {
                        benefit -= (self.base(ROCKET_DAMAGE) + me.attack * ROCKET_ATTACK / 100)
                            * SELF_DAMAGE_PERCENT
                            / me.max_hp.max(1);
                    }
                }
                let follow = 0;
                let mut cost = match family {
                    "grenade" => {
                        if farm && cluster < GRENADE_FARM_MIN {
                            p.get(9) / 4
                        } else {
                            0
                        }
                    }
                    "double_launch" => 8 + if allies == 0 { p.get(1) / 8 } else { 0 },
                    "rocket" => {
                        if blast_cluster < 2 {
                            p.get(12) / 5
                        } else {
                            0
                        }
                    }
                    _ => 0,
                };
                // Round 143: keep an escape. An entry that spends every pack with no ally near the landing costs more.
                let spends = match family {
                    "single_launch" => 1,
                    "double_launch" | "rocket_entry" | "triple_boost" => 2,
                    _ => 0,
                };
                if spends > 0 && packs <= spends && near_allies == 0 {
                    cost += 20 + (100 - p.get(0)) / 5;
                }
                let rocket_ready = f.tick >= self.cooldowns[2] || armed;
                let (blocked_at, blocked) =
                    f.sweep(if entry { entry_shot } else { me.pos }, aim, 1000);
                let firing_line = !blocked || distance(blocked_at, u.pos) <= ROCKET_RADIUS;
                let legal = match family {
                    "shoot" => true,
                    // Round 144: Bucky's slug for mid-range trades, only while she is safe
                    "slug" => {
                        self.gun == Gun::Bucky.idx()
                            && f.tick >= self.slug_ready
                            && dist > SLUG_MIN_RANGE
                            && dist <= SLUG_RANGE
                            && u.damageable()
                            && !f.sweep(me.pos, u.pos, 1000).1
                            && self.danger(f, me, me.pos) < 40
                    }
                    "grenade" => {
                        f.unlocked[0]
                            && self.charges(1, f.tick) > 0
                            && dist <= GRENADE_RANGE
                            && useful_clear
                    }
                    "bot" => f.unlocked[0] && self.charges(2, f.tick) > 0 && self.bot.is_none(),
                    "single_launch" => {
                        f.unlocked[1] && packs > 0 && !f.bound && champion && walk_agrees
                    }
                    "escape" => f.unlocked[1] && packs > 0 && !f.bound && champion,
                    "double_launch" => {
                        f.unlocked[1] && packs >= 2 && !f.bound && champion && walk_agrees
                    }
                    "rocket" => {
                        f.unlocked[2]
                            && rocket_ready
                            && dist <= ROCKET_RANGE
                            && firing_line
                            && (champion || u.kind == 3 && f.objective == Some(u.id))
                    }
                    "rocket_entry" | "triple_boost" => {
                        f.unlocked[1]
                            && f.unlocked[2]
                            && packs >= 2
                            && !f.bound
                            && champion
                            && rocket_ready
                            && walk_agrees
                            && distance(entry_shot, aim) <= ROCKET_RANGE
                            && (family == "triple_boost" || firing_line)
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
                // Round 143: a rocket from where she stands does not pay for the danger she is already in; an
                // entry rocket pays only for what the landing adds.
                let danger = match family {
                    "rocket" => 0,
                    "rocket_entry" => (danger - self.danger(f, me, me.pos)).max(0),
                    _ => danger,
                };
                let score = benefit + follow - cost - danger * (160 - p.get(0)) / 100 + error;
                let aim = if family == "slug" {
                    u.pos
                } else if family == "triple_boost" {
                    landing.sub(axis.scale(60000, 1000)).bounded()
                } else if family == "rocket" || family == "rocket_entry" {
                    p.aim(
                        me.pos,
                        u,
                        rocket_eta(dist, 0)
                            + if self.armed_until > f.tick {
                                0
                            } else {
                                ROCKET_PREP as u64
                            }
                            + if entry { 30 } else { 0 },
                        f.seed,
                        self.decision ^ u.id as u64,
                    )
                } else {
                    aim
                };
                let mut c=Candidate{family:family.into(),target:Some(u.id),landing:pos,score,immediate:benefit-cost,follow_up:follow,legal:legal&&clear,reason:format!("{family}: value {benefit}, danger {danger}, resource cost {cost}, estimation {error}"),aim,line:vec![family.into()],reflection:None,hazard:None,entry:None};
                if entry {
                    let review =
                        self.entry_review(f, me, u, pos, spends as u8, family == "rocket_entry");
                    c.legal &= review.allowed;
                    c.score += review.expected_damage / 16;
                    c.immediate += review.expected_damage / 16;
                    c.reason += &format!(
                        "; {}: hp {}, ally ETA {:?}, packs retained {}",
                        review.reason, review.predicted_hp, review.ally_eta, review.retained_packs
                    );
                    c.entry = Some(review);
                }
                if family == "grenade" {
                    let mut walls: Vec<_> = f
                        .blocks
                        .iter()
                        .chain(f.surfaces.iter().map(|s| &s.bounds))
                        .collect();
                    walls.sort_by_key(|r| {
                        let middle = r.min.add(r.max).scale(1, 2);
                        (
                            distance(me.pos, middle) + distance(aim, middle),
                            r.min.x,
                            r.min.y,
                            r.max.x,
                            r.max.y,
                        )
                    });
                    let mut throws = vec![(aim, None)];
                    for r in walls.into_iter().take(2) {
                        for x in [r.min.x - 1000, r.max.x + 1000] {
                            let reflection = Reflection {
                                axis: 0,
                                coordinate: x,
                            };
                            throws.push((reflection.apply(aim), Some(reflection)));
                        }
                        for y in [r.min.y - 1000, r.max.y + 1000] {
                            let reflection = Reflection {
                                axis: 1,
                                coordinate: y,
                            };
                            throws.push((reflection.apply(aim), Some(reflection)));
                        }
                    }
                    let mut options = vec![];
                    for (point, reflection) in throws {
                        let travel = distance(me.pos, point);
                        if travel > GRENADE_RANGE {
                            continue;
                        }
                        let (impact, _, bounces) =
                            f.ricochet(me.pos, point.sub(me.pos), travel, 1000);
                        if reflection.is_some() && bounces == 0 {
                            continue;
                        }
                        let hits = f
                            .visible_enemies(me)
                            .filter(|v| distance(v.pos, impact) <= GRENADE_RADIUS)
                            .count() as i64;
                        let mut variant = c.clone();
                        variant.aim = point;
                        variant.reflection = reflection;
                        variant.legal &= hits > 0;
                        let own = if distance(me.pos, impact) < GRENADE_RADIUS {
                            20
                        } else {
                            0
                        };
                        variant.score += hits * 4 - own;
                        variant.immediate += hits * 4 - own;
                        variant.reason+=&format!("; predicted landing {},{}, {bounces} wall bounces, {hits} main-blast targets",impact.x,impact.y);
                        options.push(variant);
                    }
                    options.sort_by_key(|v| {
                        (
                            !v.legal,
                            -v.score,
                            distance(me.pos, v.aim),
                            v.aim.x,
                            v.aim.y,
                        )
                    });
                    options.dedup_by_key(|v| v.aim);
                    all.extend(options.into_iter().take(3));
                } else if family == "rocket" || family == "rocket_entry" {
                    let from = if entry { entry_shot } else { me.pos };
                    let mut points = vec![aim];
                    if let Some(v) = f
                        .visible_enemies(me)
                        .filter(|v| {
                            v.id != u.id && v.champion() && distance(v.pos, aim) < ROCKET_RADIUS * 2
                        })
                        .min_by_key(|v| v.id)
                    {
                        points.push(aim.add(v.pos).scale(1, 2));
                    }
                    let wall = f.sweep(from, aim, 1000);
                    if wall.1 {
                        points.push(wall.0);
                    }
                    for point in points {
                        let impact = f.rocket_impact(me.team, from, point, true).0;
                        let raw = self.base(ROCKET_DAMAGE) + me.attack * ROCKET_ATTACK / 100;
                        let delivered: i64 = f
                            .visible_enemies(me)
                            .map(|v| skill_raw(rocket_damage(raw, distance(impact, v.pos))))
                            .sum();
                        let self_cost =
                            rocket_damage(raw, distance(from, impact)) * SELF_DAMAGE_PERCENT / 100
                                * if entry { 100 - AIR_REDUCTION } else { 100 }
                                / 100;
                        let champion_hits = f
                            .visible_enemies(me)
                            .filter(|v| v.champion() && distance(impact, v.pos) <= ROCKET_RADIUS)
                            .count();
                        let secure = u.kind == 3
                            && Some(u.id) == f.objective
                            && (u.hp
                                <= skill_raw(rocket_damage(raw, distance(impact, u.pos))) * 100
                                    / (100 + u.defence.max(0))
                                || champion_hits > 0);
                        let mut variant = c.clone();
                        variant.aim = point;
                        variant.legal &= distance(from, point) <= ROCKET_RANGE
                            && (champion || secure)
                            && delivered > 0;
                        // Round 143: +25 for every enemy champion inside the blast.
                        let crowd = champion_hits as i64 * 25;
                        variant.score += delivered / 12 - self_cost / 8 + crowd;
                        variant.immediate += delivered / 12 - self_cost / 8 + crowd;
                        if secure {
                            variant.score += 100;
                            variant.immediate += 100;
                        }
                        variant.reason+=&format!("; blast raw {delivered}, self {self_cost}, champion hits {champion_hits}, objective secure {secure}");
                        all.push(variant);
                    }
                } else {
                    if family == "triple_boost" {
                        c.legal &= (dist > 100000 || self.danger(f, me, me.pos) > 60)
                            && !f.sweep(landing, aim, 1000).1;
                        c.reason+="; two paid packs and a real backward rocket; exposed landing still costs full danger";
                    }
                    all.push(c);
                }
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
                legal: distance(me.pos, goal) > 2000,
                reason: format!(
                    "Team guidance step: roaming {}, order {}, missed-wave cost {}",
                    p.get(8),
                    p.get(7),
                    f.missed_wave * p.get(9) / 100
                ),
                aim: goal,
                line: vec!["rotate".into()],
                reflection: None,
                hazard: None,
                entry: None,
            });
        }
        if let Some(h) = f
            .hazards
            .iter()
            .filter(|h| {
                // Round 143: only a hazard actually reaching her (its forecast covers her), not one passing by.
                h.until > f.tick
                    && distance(h.pos, me.pos) < h.radius + BODY_RADIUS
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
            // ...and she keeps the one direction she chose until that hazard ends.
            let goal = self
                .evade
                .filter(|(id, kept)| *id == h.id && !f.blocked(*kept, BODY_RADIUS))
                .map(|(_, kept)| kept)
                .or(goal);
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
                    reflection: None,
                    hazard: Some(h.id),
                    entry: None,
                });
            }
        }
        let retreat_hp = 30 + (50 - p.get(0)).max(0) * 3 / 5;
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
                    reflection: None,
                    hazard: None,
                    entry: None,
                });
            }
        }
        // Round 143: rank first, then keep the best PLANNER_CANDIDATES (it used to cut before ranking).
        all.sort_by(|a, b| {
            b.legal
                .cmp(&a.legal)
                .then(b.score.cmp(&a.score))
                .then(a.family.cmp(&b.family))
                .then(a.target.cmp(&b.target))
        });
        all.truncate(PLANNER_CANDIDATES as usize);
        all
    }
    pub(crate) fn make_plan(&mut self, f: &Frame, me: &Unit, c: &Candidate, start: u64) -> Plan {
        let mut steps = vec![];
        let dir = c.landing.sub(me.pos).capped(1000);
        let pack = dir.scale(
            -launch_distance_for_travel(distance(me.pos, c.landing)),
            1000,
        );
        let mut at = start;
        let target_offset = c
            .target
            .and_then(|id| f.units.iter().find(|u| u.id == id))
            .map_or(Point::default(), |u| {
                if matches!(c.family.as_str(), "rocket" | "rocket_entry") {
                    let travel = rocket_eta(distance(me.pos, u.pos), 0)
                        + if self.armed_until > f.tick {
                            0
                        } else {
                            ROCKET_PREP as u64
                        }
                        + if c.family == "rocket_entry" { 30 } else { 0 };
                    c.aim.sub(
                        f.profile
                            .aim(me.pos, u, travel, f.seed, self.decision ^ u.id as u64),
                    )
                } else {
                    c.aim.sub(u.pos)
                }
            });
        let mut add = |at: u64, a: Action, relative: bool, intended: u64| {
            steps.push(PlannedStep {
                at,
                action: a,
                relative,
                intended,
                target_offset,
                reflection: c.reflection.clone(),
                landing: Some(c.landing),
            })
        };
        match c.family.as_str() {
            "grenade" => add(at, Action::Grenade { aim: c.aim }, false, at),
            "bot" => add(at, Action::Bot { aim: c.aim }, false, at),
            "slug" => {
                if let Some(target) = c.target {
                    add(at, Action::Slug { target }, false, at)
                }
            }
            "rocket" => {
                // Round 144: the armed pose is held ROCKET_HOLD ticks before the shot, so it can be seen.
                if self.armed_until <= at {
                    add(at, Action::EquipRocket, false, at);
                    at += (ROCKET_PREP + ROCKET_HOLD) as u64 + 1;
                } else {
                    at = at.max(self.armed_at + ROCKET_HOLD as u64);
                }
                add(at, Action::FireRocket { aim: c.aim }, false, at);
            }
            "rocket_entry" | "triple_boost" => {
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
                add(
                    at,
                    Action::FireRocket { aim: c.aim },
                    c.family == "triple_boost",
                    at,
                );
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
        // Round 143: only the nearest enemy champion and her own HP quarter. Bullet ids, hazards and CC flags
        // changed every few ticks and kept re-arming the reaction, so she never decided in a fight.
        let enemy = f
            .visible_enemies(me)
            .filter(|u| u.champion())
            .min_by_key(|u| (distance(me.pos, u.pos), u.id));
        let situation = hash(
            enemy.map_or(0, |u| u.id as u64 + 1)
                ^ ((me.hp * 4 / me.max_hp.max(1)).clamp(0, 4) as u64) << 40,
        );
        if situation != self.observed_situation {
            self.observed_situation = situation;
            // the timer is re-armed only once the previous reaction has run out
            if f.tick >= self.reaction_until {
                self.reaction_until = f.tick + f.profile.reaction();
                self.reaction_events += 1;
            }
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
                // Round 143: an armed rocket re-targets the best visible champion at once instead of being dropped.
                if self.armed_until > f.tick {
                    self.next_think = f.tick;
                }
            }
        }
        if let Some(mut p) = self.plan.take() {
            if let Some(step) = p.steps.first().cloned() {
                if f.tick >= step.at && self.cast.is_none() {
                    p.steps.remove(0);
                    let landing = step.landing.unwrap_or(p.landing);
                    if matches!(step.action, Action::Pack { .. } | Action::Detonate { .. })
                        && p.family != "escape"
                    {
                        if let Some(target) = p
                            .target
                            .and_then(|id| f.visible_enemies(me).find(|u| u.id == id))
                        {
                            let spends = p
                                .steps
                                .iter()
                                .filter(|s| matches!(s.action, Action::Detonate { .. }))
                                .count() as u8
                                + u8::from(matches!(step.action, Action::Detonate { .. }));
                            let review = self.entry_review(
                                f,
                                me,
                                target,
                                landing,
                                spends,
                                p.steps.iter().any(|s| {
                                    matches!(s.action, Action::FireRocket { .. }) && !s.relative
                                }),
                            );
                            self.last_entry = Some(review.clone());
                            if !review.allowed {
                                if let Some(v) = self.flight.as_mut() {
                                    v.goal = review.escape.unwrap_or(me.pos);
                                    v.next_input = f.tick;
                                }
                                self.continuation.clear();
                                self.reject(out, &format!("Entry cancelled: {}", review.reason));
                                return;
                            }
                        }
                    }
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
                            Action::FireRocket { .. } => {
                                let direction = self
                                    .flight
                                    .as_ref()
                                    .map_or(p.landing.sub(me.pos), |v| v.velocity);
                                Action::FireRocket {
                                    aim: me
                                        .pos
                                        .sub(direction.scale(60000, direction.length().max(1)))
                                        .bounded(),
                                }
                            }
                            a => a,
                        }
                    } else {
                        step.action
                    };
                    if let Some(u) = p.target.filter(|_| !step.relative).and_then(|id| {
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
                                if matches!(action, Action::FireRocket { .. }) {
                                    rocket_eta(distance(me.pos, u.pos), 0)
                                } else {
                                    (distance(me.pos, u.pos) * TPS / speed + prep) as u64
                                },
                                f.seed,
                                step.intended ^ u.id as u64,
                            );
                            action = match action {
                                Action::Grenade { .. } => Action::Grenade {
                                    aim: step.reflection.as_ref().map_or(aim, |r| r.apply(aim)),
                                },
                                Action::FireRocket { .. } => Action::FireRocket {
                                    aim: aim.add(step.target_offset).bounded(),
                                },
                                a => a,
                            };
                        }
                    }
                    if matches!(action, Action::Detonate { .. }) {
                        self.intended_detonation = Some(step.intended);
                        if p.family != "escape" {
                            let selected = self
                                .packs
                                .iter()
                                .filter(|pack| {
                                    pack.ready
                                        && pack.hp > 0
                                        && distance(me.pos, pack.pos) <= PACK_RADIUS
                                })
                                .min_by_key(|pack| (distance(me.pos, pack.pos), pack.id));
                            let Some(pack) = selected else {
                                self.continuation.clear();
                                self.reject(
                                    out,
                                    "Entry cancelled: no ready launch satchel in reach",
                                );
                                return;
                            };
                            action = Action::Detonate { id: pack.id };
                        }
                    }
                    p.landing = landing;
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
        // Round 143: a decision at least every reaction + 12 ticks, and at once while a rocket is armed.
        let forced = f.tick >= self.last_think + self.profile.reaction() + 12;
        if f.tick < self.reaction_until && !forced && self.armed_until <= f.tick
            || f.tick < self.next_think
        {
            return;
        }
        self.next_think = f.tick + 12;
        self.last_think = f.tick;
        self.decision += 1;
        self.candidates = self.evaluate(f, me);
        // counts the follow-up steps searched (it used to start at the candidate count, so none were)
        self.evaluated = 0;
        self.search(f, me);
        let shortlist: Vec<_> = self
            .candidates
            .iter()
            .filter(|c| c.legal)
            .take(BEAM_WIDTH as usize)
            .cloned()
            .collect();
        let Some(best) = shortlist.first().filter(|c| c.score > 0).cloned() else {
            self.guide = None;
            return;
        };
        self.continuation = best.line.clone();
        self.last_entry = best.entry.clone();
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
            if best.family == "evade" {
                self.evade = best.hazard.map(|id| (id, to));
            }
            // held and re-issued every tick by hold_guide (round 143)
            self.guide = Some(Guide {
                family: best.family.clone(),
                target: best.target,
                to,
            });
            return;
        }
        self.guide = None;
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
    fn search(&mut self, f: &Frame, me: &Unit) {
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
            "rocket_entry" | "triple_boost" => {
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
                    stock: [0, 1, 2].map(|n| self.charges(n, f.tick)),
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
                        "rocket_entry" | "triple_boost" => n.stock[0] >= 2 && !n.spent[2],
                        "single_launch" | "escape" => n.stock[0] > 0,
                        "double_launch" => n.stock[0] >= 2,
                        "shoot" => true,
                        "slug" => distance(n.pos, c.aim) <= SLUG_RANGE,
                        _ => false,
                    };
                    if !allowed {
                        continue;
                    }
                    self.evaluated += 1;
                    expanded += 1;
                    let value = if c.family == "shoot" || c.family == "slug" {
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
