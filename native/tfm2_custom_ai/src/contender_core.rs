//! Shared deterministic boxing rules. Host acceptance/impact receipts are explicit inputs.
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, sync::OnceLock};
pub const RULES: &str = include_str!("../../../rules/contender.json");
pub fn canonical() -> &'static serde_json::Value {
    static VALUE: OnceLock<serde_json::Value> = OnceLock::new();
    VALUE.get_or_init(|| serde_json::from_str(RULES).expect("canonical boxing rules"))
}
pub fn number(section: &str, key: &str) -> i64 {
    canonical()[section][key]
        .as_i64()
        .expect("canonical numeric rule")
}
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Point {
    pub x: i64,
    pub y: i64,
}
impl Point {
    pub fn distance2(self, b: Self) -> i128 {
        (self.x - b.x) as i128 * (self.x - b.x) as i128
            + (self.y - b.y) as i128 * (self.y - b.y) as i128
    }
    pub fn toward(self, b: Self, length: i64) -> Self {
        let dx = b.x - self.x;
        let dy = b.y - self.y;
        let len = isqrt((dx as i128 * dx as i128 + dy as i128 * dy as i128) as u128).max(1) as i64;
        let n = length.min(len);
        Self {
            x: self.x + dx * n / len,
            y: self.y + dy * n / len,
        }
    }
}
fn isqrt(n: u128) -> u128 {
    if n < 2 {
        return n;
    }
    let mut x = n;
    let mut y = x.div_ceil(2);
    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }
    x
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Rect {
    pub min: Point,
    pub max: Point,
}
impl Rect {
    pub fn contains(self, p: Point, r: i64) -> bool {
        p.x >= self.min.x + r
            && p.y >= self.min.y + r
            && p.x <= self.max.x - r
            && p.y <= self.max.y - r
    }
    pub fn clamp(self, p: Point, r: i64) -> Point {
        Point {
            x: p.x.clamp(self.min.x + r, self.max.x - r),
            y: p.y.clamp(self.min.y + r, self.max.y - r),
        }
    }
}
pub fn map() -> Rect {
    Rect {
        min: Point::default(),
        max: Point {
            x: number("arena", "mapLimit"),
            y: number("arena", "mapLimit"),
        },
    }
}
pub fn arena(owner: usize) -> Rect {
    let side = number("arena", "interiorSide");
    let x = canonical()["arena"]["prototypeOrigin"][0].as_i64().unwrap()
        + owner as i64 * number("arena", "slotStride");
    let y = canonical()["arena"]["prototypeOrigin"][1].as_i64().unwrap();
    Rect {
        min: Point { x, y },
        max: Point {
            x: x + side,
            y: y + side,
        },
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Unit {
    pub id: usize,
    pub team: usize,
    pub pos: Point,
    pub hp: i64,
    pub max_hp: i64,
    pub attack: i64,
    pub radius: i64,
    pub champion: bool,
    pub visible: bool,
    pub targetable: bool,
    pub cc_immune: bool,
    pub farmable: bool,
    #[serde(default)]
    pub velocity: Point,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Technique {
    Combination,
    BodyShot,
    Uppercut,
    Slip,
    Guard,
    MainEvent,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Action {
    pub technique: Technique,
    pub target: Option<usize>,
    pub aim: Point,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Receipt {
    pub serial: u64,
    pub source: usize,
    pub champion: bool,
    pub positive_damage: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Basic {
    pub serial: u64,
    pub target: usize,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Frame {
    pub version: u8,
    pub tick: u64,
    pub owner: usize,
    pub units: Vec<Unit>,
    pub blocks: Vec<Rect>,
    pub bounds: Rect,
    pub profile: [i64; 13],
    pub manual: bool,
    pub action: Option<Action>,
    pub skill1: bool,
    pub skill2: bool,
    pub ultimate: bool,
    pub interrupted: bool,
    pub rooted: bool,
    #[serde(default)]
    pub receipts: Vec<Receipt>,
    #[serde(default)]
    pub basics: Vec<Basic>,
    #[serde(default)]
    pub arena_started: Option<usize>,
    #[serde(default)]
    pub arena_ended: bool,
    #[serde(default)]
    pub basics_validated_by_host: bool,
    #[serde(default)]
    pub dealt_at: Option<u64>,
    #[serde(default)]
    pub duel_unavailable: Vec<usize>,
}
impl Frame {
    pub fn me(&self) -> Option<&Unit> {
        self.units.iter().find(|u| u.id == self.owner)
    }
    pub fn enemy(&self, id: usize) -> Option<&Unit> {
        let m = self.me()?;
        self.units
            .iter()
            .find(|u| u.id == id && u.team != m.team && u.hp > 0 && u.visible && u.targetable)
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Effect {
    Damage {
        target: usize,
        amount: i64,
        basic: bool,
        identity: u64,
    },
    Airborne {
        target: usize,
        ticks: u64,
    },
    AttackSlow {
        target: usize,
        percent: i64,
        ticks: u64,
    },
    Shield {
        amount: i64,
        ticks: u64,
    },
    Defence {
        percent: i64,
        move_penalty: i64,
        ticks: u64,
    },
    Dodge {
        ticks: u64,
    },
    Move {
        pos: Point,
    },
    Warn {
        from: Point,
        to: Point,
        radius: i64,
        until: u64,
        label: String,
    },
    ClearWarnings,
    Transfer {
        target: usize,
        bounds: Rect,
        duration: u64,
    },
    Pose {
        name: String,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Cast {
    pub technique: Technique,
    pub begin: u64,
    pub until: u64,
    pub target: Option<usize>,
    pub from: Point,
    pub aim: Point,
    pub hit_mask: u8,
    pub reaimed: bool,
    pub attack: i64,
    /// Round 148: where a dash punch carries him during its wind-up (None: a punch from where he stands).
    #[serde(default)]
    pub dash_to: Option<Point>,
}
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Decision {
    pub selected: Vec<Technique>,
    pub reason: String,
    pub candidates: usize,
    pub reaction_until: u64,
    /// Round 146: the continuation the planner is in: approach, pressure, punish or defend ("" when idle).
    #[serde(default)]
    pub intent: String,
    /// Round 146: enemies the planner did not consider this decision, with why (Boxing Lab v2 diagnostics).
    #[serde(default)]
    pub rejected: Vec<Rejected>,
}
#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct Rejected {
    pub id: usize,
    pub reason: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Core {
    pub stamina: i64,
    pub cooldowns: [i64; 4],
    pub cast: Option<Cast>,
    pub counter: Option<(usize, u64)>,
    pub arena_target: Option<usize>,
    pub arena_until: u64,
    pub last_champion_damage: u64,
    pub last_tick: Option<u64>,
    pub alive: bool,
    pub cross: bool,
    pub rejection: String,
    pub decision: Decision,
    pub last_basic: u64,
    pub last_receipt: u64,
    pub situation: Option<usize>,
    pub next_plan: u64,
    pub next_basic: u64,
    /// Round 146: the last tick an enemy champion's hit landed on him (an observed reason to defend).
    #[serde(default)]
    pub last_hit_taken: u64,
    /// Round 146: until when an opening he made (a landed uppercut or body shot, a guarded hit) is worth punishing.
    #[serde(default)]
    pub punish_until: u64,
}
impl Default for Core {
    fn default() -> Self {
        Self {
            stamina: 100000,
            cooldowns: [0; 4],
            cast: None,
            counter: None,
            arena_target: None,
            arena_until: 0,
            last_champion_damage: 0,
            last_tick: None,
            alive: true,
            cross: false,
            rejection: String::new(),
            decision: Decision::default(),
            last_basic: 0,
            last_receipt: 0,
            situation: None,
            next_plan: 0,
            next_basic: 0,
            last_hit_taken: 0,
            punish_until: 0,
        }
    }
}
fn group(t: Technique) -> usize {
    match t {
        Technique::Combination | Technique::BodyShot => 0,
        Technique::Slip | Technique::Guard => 1,
        Technique::Uppercut => 2,
        Technique::MainEvent => 3,
    }
}
fn section(t: Technique) -> &'static str {
    match t {
        Technique::Combination => "combination",
        Technique::BodyShot => "bodyShot",
        Technique::Uppercut => "uppercut",
        Technique::Slip => "slip",
        Technique::Guard => "guard",
        Technique::MainEvent => "ultimate",
    }
}
pub fn legal_move(f: &Frame, from: Point, to: Point, r: i64) -> bool {
    if !f.bounds.contains(to, r) {
        return false;
    }
    let length = isqrt(from.distance2(to) as u128) as i64;
    let steps = (length / 2000).max(1);
    (0..=steps).all(|i| {
        let p = Point {
            x: from.x + (to.x - from.x) * i / steps,
            y: from.y + (to.y - from.y) * i / steps,
        };
        !f.blocks.iter().any(|b| {
            p.x >= b.min.x - r && p.x <= b.max.x + r && p.y >= b.min.y - r && p.y <= b.max.y + r
        })
    })
}
fn segment_distance2(p: Point, a: Point, b: Point) -> i128 {
    let x = (b.x - a.x) as i128;
    let y = (b.y - a.y) as i128;
    let n = x * x + y * y;
    if n == 0 {
        return p.distance2(a);
    }
    let dot = ((p.x - a.x) as i128 * x + (p.y - a.y) as i128 * y).clamp(0, n);
    let q = Point {
        x: a.x + (x * dot / n) as i64,
        y: a.y + (y * dot / n) as i64,
    };
    p.distance2(q)
}
pub fn capsule_target<'a>(f: &'a Frame, from: Point, to: Point) -> Option<&'a Unit> {
    let m = f.me()?;
    let end = from.toward(to, isqrt(from.distance2(to) as u128) as i64);
    f.units
        .iter()
        .filter(|u| u.team != m.team && u.hp > 0 && u.visible && u.targetable)
        .filter(|u| segment_distance2(u.pos, from, end) <= ((u.radius + 6000) as i128).pow(2))
        .filter(|u| clear_line(f, from, u.pos))
        .min_by_key(|u| (from.distance2(u.pos), u.id))
}
/// Round 148: a hit's line of sight. Only wall cells strictly between the two bodies block it: the coarse 32,000
/// cells under the striker or the target (camps sit in walled pockets) don't, so a hit the walls don't really cut
/// lands.
pub fn clear_line(f: &Frame, from: Point, to: Point) -> bool {
    let length = isqrt(from.distance2(to) as u128) as i64;
    let steps = (length / 1000).max(1);
    let own = |b: &Rect, p: Point| p.x >= b.min.x && p.x <= b.max.x && p.y >= b.min.y && p.y <= b.max.y;
    (0..=steps).all(|i| {
        let p = Point {
            x: from.x + (to.x - from.x) * i / steps,
            y: from.y + (to.y - from.y) * i / steps,
        };
        !f.blocks.iter().any(|b| own(b, p) && !own(b, from) && !own(b, to))
    })
}
/// Round 148: the gap between two bodies (the game measures attack range edge to edge, not centre to centre).
pub fn edge_distance(a: &Unit, b: &Unit) -> i64 {
    (isqrt(a.pos.distance2(b.pos) as u128) as i64 - a.radius - b.radius).max(0)
}
/// How far a technique's wind-up dash carries him (0 for the ones that stand still).
pub fn dash_travel(t: Technique) -> i64 {
    match t {
        Technique::Combination | Technique::BodyShot | Technique::Uppercut => {
            canonical()[section(t)]["dashTravel"].as_i64().unwrap_or(0)
        }
        _ => 0,
    }
}
/// Ticks the dash takes: it ends the tick before the first hit, so the hit lands from where it stopped.
fn dash_ticks(t: Technique) -> u64 {
    match t {
        Technique::Combination => 8,
        Technique::BodyShot | Technique::Uppercut => {
            (number(section(t), "preparationTicks") as u64).saturating_sub(1).max(1)
        }
        _ => 0,
    }
}
impl Core {
    pub fn cooldown_ticks(&self, n: usize) -> u64 {
        ((self.cooldowns[n].max(0) + 2) / 3) as u64
    }
    pub fn busy(&self) -> bool {
        self.cast.is_some()
    }
    pub fn respawn(&mut self, tick: u64) {
        if let Some(last) = self.last_tick {
            let dt = tick.saturating_sub(last) as i64;
            for c in &mut self.cooldowns {
                *c = (*c - 3 * dt).max(0)
            }
        }
        self.stamina = number("stamina", "maximum") * 1000;
        self.cast = None;
        self.counter = None;
        self.arena_target = None;
        self.arena_until = 0;
        self.alive = true;
        self.last_tick = Some(tick);
        self.situation = None;
    }
    pub fn dealt(&mut self, tick: u64, champion: bool, positive: bool) {
        if champion && positive {
            self.last_champion_damage = tick
        }
    }
    fn valid(&self, f: &Frame, a: &Action) -> Result<(), String> {
        let m = f.me().ok_or("Owner missing")?;
        if m.hp <= 0 {
            return Err("Dead".into());
        }
        if self.busy() {
            return Err("A technique is already running".into());
        }
        if f.interrupted {
            return Err("Interrupted or silenced".into());
        }
        let t = a.technique;
        let s = section(t);
        let unlock = match t {
            Technique::Combination | Technique::BodyShot | Technique::Uppercut => f.skill1,
            Technique::Slip | Technique::Guard => f.skill2,
            Technique::MainEvent => f.ultimate,
        };
        if !unlock {
            return Err("Skill is not learned".into());
        }
        if self.cooldowns[group(t)] > 0 {
            return Err("Shared cooldown is recovering".into());
        }
        if self.stamina < number(s, "stamina") * 1000 {
            return Err("Insufficient stamina".into());
        }
        if t == Technique::Slip {
            if f.rooted {
                return Err("Movement is disabled".into());
            }
            let to = m.pos.toward(a.aim, number(s, "travel"));
            if !legal_move(f, m.pos, to, m.radius) {
                return Err("Dodge path is blocked".into());
            }
        }
        if t == Technique::MainEvent {
            if f.duel_unavailable.contains(&f.owner)
                || a.target.is_some_and(|id| f.duel_unavailable.contains(&id))
            {
                return Err("Participant is already committed to Main Event".into());
            }
            if self.arena_target.is_some() {
                return Err("Already in Main Event".into());
            }
            let target = a
                .target
                .and_then(|id| f.enemy(id))
                .filter(|u| u.champion)
                .ok_or("Visible enemy champion required")?;
            if m.pos.distance2(target.pos) > (number(s, "range") as i128).pow(2) {
                return Err("Target outside ultimate range".into());
            }
        }
        Ok(())
    }
    fn accept(&mut self, f: &Frame, a: Action, out: &mut Vec<Effect>) -> bool {
        if let Err(e) = self.valid(f, &a) {
            self.rejection = e;
            return false;
        }
        let m = f.me().unwrap();
        let t = a.technique;
        let s = section(t);
        self.stamina -= number(s, "stamina") * 1000;
        self.cooldowns[group(t)] = number(s, "cooldownTicks") * 3;
        self.rejection.clear();
        let duration = match t {
            Technique::Combination => number(s, "durationTicks"),
            Technique::Uppercut => number(s, "preparationTicks") + number(s, "recoveryTicks"),
            Technique::Slip | Technique::Guard => number(s, "durationTicks"),
            _ => number(
                s,
                if t == Technique::MainEvent {
                    "warningTicks"
                } else {
                    "preparationTicks"
                },
            ),
        } as u64;
        let aim = if t == Technique::Slip {
            m.pos.toward(a.aim, number(s, "travel"))
        } else {
            a.aim
        };
        // Round 148: the punches dash in (up to dashTravel), stopping at the target's edge and short of walls.
        let dash_to = (dash_travel(t) > 0 && !f.rooted).then(|| {
            let stop = a
                .target
                .and_then(|id| f.enemy(id))
                .map_or(0, |u| (edge_distance(m, u) - 1500).max(0));
            let goal = a.target.and_then(|id| f.enemy(id)).map_or(aim, |u| u.pos);
            let mut travel = dash_travel(t).min(if a.target.is_some() { stop } else { dash_travel(t) });
            while travel > 0 && !legal_move(f, m.pos, m.pos.toward(goal, travel), m.radius) {
                travel -= 2000;
            }
            m.pos.toward(goal, travel.max(0))
        })
        .filter(|p| *p != m.pos);
        self.cast = Some(Cast {
            technique: t,
            begin: f.tick,
            until: f.tick + duration,
            target: a.target,
            from: m.pos,
            aim,
            hit_mask: 0,
            reaimed: false,
            attack: m.attack,
            dash_to,
        });
        if t == Technique::Guard {
            out.push(Effect::Shield {
                amount: number(s, "shieldBase") + m.attack * number(s, "shieldAttackPercent") / 100,
                ticks: number(s, "shieldTicks") as u64,
            });
            out.push(Effect::Defence {
                percent: number(s, "damageReductionPercent"),
                move_penalty: number(s, "moveSpeedReductionPercent"),
                ticks: duration,
            })
        }
        if t == Technique::Slip {
            out.push(Effect::Dodge { ticks: duration })
        }
        out.push(Effect::Pose { name: s.into() });
        if matches!(
            t,
            Technique::Combination | Technique::BodyShot | Technique::Uppercut
        ) {
            let reach = number(s, "reach");
            let until = f.tick
                + if t == Technique::Combination {
                    9
                } else {
                    number(s, "preparationTicks") as u64
                };
            let from = dash_to.unwrap_or(m.pos);
            out.push(Effect::Warn {
                from,
                to: from.toward(aim, reach + m.radius),
                radius: 6000,
                until,
                label: s.into(),
            })
        }
        if t == Technique::MainEvent {
            out.push(Effect::Warn {
                from: m.pos,
                to: aim,
                radius: 10000,
                until: f.tick + duration,
                label: "Main Event".into(),
            })
        }
        true
    }
    pub fn step(&mut self, f: &Frame) -> Vec<Effect> {
        let mut out = vec![];
        // v2 (Boxing Lab v2) only adds diagnostics the core ignores; both versions play identically.
        if !(1..=2).contains(&f.version) || self.last_tick.is_some_and(|t| f.tick <= t) {
            return out;
        }
        let Some(me) = f.me() else { return out };
        if let Some(tick) = f.dealt_at.filter(|&t| t <= f.tick) {
            self.last_champion_damage = tick;
        }
        let dt = self.last_tick.map_or(0, |t| f.tick - t);
        let arena_elapsed = if self.arena_target.is_some() {
            f.tick
                .min(self.arena_until)
                .saturating_sub(self.last_tick.unwrap_or(f.tick))
        } else {
            0
        };
        self.last_tick = Some(f.tick);
        if f.arena_ended {
            self.arena_target = None;
            self.arena_until = 0
        }
        if let Some(t) = f.arena_started {
            self.arena_target = Some(t);
            self.arena_until = f.tick + number("ultimate", "durationTicks") as u64
        }
        for (i, c) in self.cooldowns.iter_mut().enumerate() {
            *c = (*c - dt as i64 * 3 - if i < 3 { arena_elapsed as i64 } else { 0 }).max(0)
        }
        if me.hp <= 0 {
            if self.alive {
                out.push(Effect::ClearWarnings)
            }
            self.alive = false;
            self.cast = None;
            self.counter = None;
            self.arena_target = None;
            return out;
        }
        if !self.alive {
            self.respawn(f.tick)
        }
        let mut seen = BTreeSet::new();
        for r in &f.receipts {
            if r.serial <= self.last_receipt || !seen.insert(r.serial) {
                continue;
            }
            self.last_receipt = r.serial;
            if r.champion && r.positive_damage {
                self.last_champion_damage = f.tick;
                self.last_hit_taken = f.tick;
                if self
                    .cast
                    .as_ref()
                    .is_some_and(|c| c.technique == Technique::Guard && f.tick < c.until)
                    && self.counter.is_none()
                {
                    self.counter =
                        Some((r.source, f.tick + number("counter", "windowTicks") as u64));
                    self.punish_until = f.tick + 45
                }
            }
        }
        if self.counter.is_some_and(|(_, t)| f.tick >= t) {
            self.counter = None
        }
        if self.cast.is_none() {
            let regen = if f.tick.saturating_sub(self.last_champion_damage)
                >= number("stamina", "outOfCombatTicks") as u64
            {
                number("stamina", "outOfCombatRecoveryPerSecond")
            } else {
                number("stamina", "recoveryPerSecond")
            };
            self.stamina = (self.stamina + regen * 1000 * dt as i64 / 60)
                .min(number("stamina", "maximum") * 1000)
        }
        if f.interrupted
            && self
                .cast
                .as_ref()
                .is_some_and(|c| c.technique != Technique::Slip)
        {
            self.cast = None;
            out.push(Effect::ClearWarnings)
        }
        if let Some(mut c) = self.cast.take() {
            if c.technique == Technique::Slip {
                let n = (f.tick - c.begin).min(c.until - c.begin) as i64;
                let total = (c.until - c.begin) as i64;
                out.push(Effect::Move {
                    pos: Point {
                        x: c.from.x + (c.aim.x - c.from.x) * n / total,
                        y: c.from.y + (c.aim.y - c.from.y) * n / total,
                    },
                })
            }
            if let Some(to) = c.dash_to.filter(|_| !f.rooted) {
                let total = dash_ticks(c.technique).max(1);
                let n = (f.tick - c.begin).min(total);
                if n > 0 && f.tick - c.begin <= total {
                    out.push(Effect::Move {
                        pos: Point {
                            x: c.from.x + (to.x - c.from.x) * n as i64 / total as i64,
                            y: c.from.y + (to.y - c.from.y) * n as i64 / total as i64,
                        },
                    })
                }
            }
            if c.technique == Technique::Combination && !c.reaimed && f.tick >= c.begin + 18 {
                c.reaimed = true;
                if let Some(u) = c.target.and_then(|id| f.enemy(id)) {
                    c.aim = u.pos
                }
                out.push(Effect::Warn {
                    from: me.pos,
                    to: me.pos.toward(c.aim, 23000),
                    radius: 6000,
                    until: c.begin + 27,
                    label: "Cross".into(),
                })
            }
            let schedule: Vec<(u64, i64, i64, u8)> = match c.technique {
                Technique::Combination => canonical()["combination"]["hits"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .enumerate()
                    .map(|(i, h)| {
                        (
                            h["tick"].as_u64().unwrap(),
                            h["base"].as_i64().unwrap(),
                            h["attackPercent"].as_i64().unwrap(),
                            1u8 << i,
                        )
                    })
                    .collect(),
                Technique::BodyShot | Technique::Uppercut => vec![(
                    number(section(c.technique), "preparationTicks") as u64,
                    number(section(c.technique), "base"),
                    number(section(c.technique), "attackPercent"),
                    1,
                )],
                _ => vec![],
            };
            for (delay, base, ratio, bit) in schedule {
                if f.tick >= c.begin + delay && c.hit_mask & bit == 0 {
                    c.hit_mask |= bit;
                    let reach = if c.technique == Technique::Combination {
                        23000
                    } else {
                        20000
                    };
                    if let Some(u) = capsule_target(f, me.pos, me.pos.toward(c.aim, reach + me.radius)) {
                        out.push(Effect::Damage {
                            target: u.id,
                            amount: base + c.attack * ratio / 100,
                            basic: false,
                            identity: c.begin * 8 + bit as u64,
                        });
                        if matches!(c.technique, Technique::Uppercut | Technique::BodyShot) {
                            self.punish_until = f.tick + 30;
                        }
                        if c.technique == Technique::Uppercut && !u.cc_immune {
                            out.push(Effect::Airborne {
                                target: u.id,
                                ticks: 24,
                            })
                        }
                        if c.technique == Technique::BodyShot {
                            out.push(Effect::AttackSlow {
                                target: u.id,
                                percent: 15,
                                ticks: 90,
                            })
                        }
                    }
                }
            }
            if c.technique == Technique::MainEvent && f.tick >= c.until {
                if let Some(u) = c
                    .target
                    .and_then(|id| f.enemy(id))
                    .filter(|u| u.champion && u.pos.distance2(me.pos) <= 65000_i128.pow(2))
                {
                    out.push(Effect::Transfer {
                        target: u.id,
                        bounds: arena(f.owner),
                        duration: 480,
                    })
                } else {
                    self.rejection = "Ultimate target lost before transfer".into()
                }
            }
            if f.tick < c.until {
                self.cast = Some(c)
            } else {
                out.push(Effect::ClearWarnings)
            }
        }
        if !self.busy() {
            for b in &f.basics {
                if b.serial <= self.last_basic
                    || (!f.basics_validated_by_host && f.tick < self.next_basic)
                {
                    continue;
                }
                self.last_basic = b.serial;
                // Round 148: the game has already accepted its own basic attack (range edge to edge, its own line of
                // sight), so the hit lands on its target. Only the Boxing Lab, with no host, re-checks it.
                let hit = if f.basics_validated_by_host {
                    f.enemy(b.target)
                } else {
                    f.enemy(b.target)
                        .filter(|u| me.pos.distance2(u.pos) <= 23000_i128.pow(2))
                        .and_then(|u| {
                            capsule_target(f, me.pos, me.pos.toward(u.pos, number("basic", "range")))
                        })
                };
                if let Some(u) = hit {
                    let bonus = if self
                        .counter
                        .is_some_and(|(id, until)| id == u.id && f.tick < until)
                    {
                        self.counter = None;
                        number("counter", "base")
                            + me.attack * number("counter", "attackPercent") / 100
                    } else {
                        0
                    };
                    self.next_basic = f.tick + number("basic", "cooldownTicks") as u64;
                    self.cross = !self.cross;
                    out.push(Effect::Damage {
                        target: u.id,
                        amount: me.attack + bonus,
                        basic: true,
                        identity: b.serial,
                    });
                    out.push(Effect::Pose {
                        name: if self.cross { "cross" } else { "jab" }.into(),
                    });
                }
            }
        }
        if let Some(a) = f.action.clone() {
            self.accept(f, a, &mut out);
        } else if !f.manual && !self.busy() {
            if let Some(a) = self.plan(f) {
                self.accept(f, a, &mut out);
            }
        }
        out
    }
    fn plan(&mut self, f: &Frame) -> Option<Action> {
        let m = f.me()?;
        let p = f.profile.map(|x| x.clamp(0, 100));
        // In Main Event the opponent is the only target that matters, however far across the ring.
        let duel = self.arena_target.and_then(|id| f.enemy(id));
        let enemy = match duel {
            Some(u) => u,
            None => f
                .units
                .iter()
                .filter(|u| {
                    u.hp > 0
                        && u.team != m.team
                        && u.visible
                        && u.targetable
                        && (u.champion || u.farmable)
                        && m.pos.distance2(u.pos) <= 80000_i128.pow(2)
                })
                .min_by_key(|u| (!u.champion, m.pos.distance2(u.pos), u.id))?,
        };
        let in_duel = duel.is_some();
        self.decision.rejected = f
            .units
            .iter()
            .filter(|u| u.team != m.team && u.hp > 0 && u.id != enemy.id)
            .map(|u| Rejected {
                id: u.id,
                reason: if in_duel {
                    "outside the duel"
                } else if !u.visible {
                    "concealed"
                } else if !u.targetable {
                    "untargetable"
                } else if !u.champion && !u.farmable {
                    "not farmable now"
                } else if m.pos.distance2(u.pos) > 80000_i128.pow(2) {
                    "out of reach"
                } else {
                    "a closer target came first"
                }
                .into(),
            })
            .take(16)
            .collect();
        if self.situation != Some(enemy.id) {
            self.situation = Some(enemy.id);
            self.decision.reaction_until = f.tick + (24 - 18 * p[3] / 100) as u64
        }
        if f.tick < self.decision.reaction_until || f.tick < self.next_plan {
            return None;
        }
        self.next_plan = f.tick + (12 - 9 * p[10] / 100) as u64;
        let centre = isqrt(m.pos.distance2(enemy.pos) as u128) as i64;
        // Round 148: reach is edge to edge, as the game measures it (centre distance left every punch "out of reach").
        let distance = edge_distance(m, enemy);
        let danger = f
            .units
            .iter()
            .filter(|u| {
                u.champion
                    && u.team != m.team
                    && u.visible
                    && u.pos.distance2(m.pos) < 80000_i128.pow(2)
            })
            .count() as i64;
        let allies = f
            .units
            .iter()
            .filter(|u| {
                u.champion
                    && u.team == m.team
                    && u.id != m.id
                    && u.hp > 0
                    && u.pos.distance2(m.pos) < 80000_i128.pow(2)
            })
            .count() as i64;
        // Round 146: approach → pressure → punish → defend. Defend needs an observed reason (a hit just landed on
        // him); punish follows an opening he made; out of reach he approaches; in reach he keeps the pressure on.
        let hp_pct = m.hp * 100 / m.max_hp.max(1);
        let threatened = self.last_hit_taken > 0 && f.tick.saturating_sub(self.last_hit_taken) <= 45;
        let punish = f.tick < self.punish_until;
        let reach = number("combination", "reach") + dash_travel(Technique::Combination);
        let intent = if threatened && (hp_pct < 50 || !punish && distance < 25000 && p[5] > 60) {
            "defend"
        } else if distance > reach {
            "approach"
        } else if punish {
            "punish"
        } else {
            "pressure"
        };
        self.decision.intent = intent.into();
        let hash = (f.tick.wrapping_mul(0x9e3779b97f4a7c15) ^ (enemy.id as u64)).rotate_left(17);
        let error = ((hash % 41) as i64 - 20) * (100 - p[1]) / 100;
        let lead = 6 + (p[4] * 12 / 100);
        let aim = Point {
            x: enemy.pos.x + enemy.velocity.x * lead,
            y: enemy.pos.y + enemy.velocity.y * lead,
        };
        let mut candidates = vec![];
        for t in [
            Technique::Combination,
            Technique::BodyShot,
            Technique::Uppercut,
            Technique::Slip,
            Technique::Guard,
            Technique::MainEvent,
        ] {
            let retreat = Point {
                x: m.pos.x + (m.pos.x - enemy.pos.x),
                y: m.pos.y + (m.pos.y - enemy.pos.y),
            };
            let place = if t == Technique::Slip {
                if m.hp * 100 / m.max_hp < 35 || danger > allies + 1 {
                    retreat
                } else {
                    aim
                }
            } else {
                aim
            };
            let a = Action {
                technique: t,
                target: Some(enemy.id),
                aim: f.bounds.clamp(place, m.radius),
            };
            if self.valid(f, &a).is_err() {
                continue;
            }
            let score = match t {
                // Guard only against an observed threat: a hit has just landed on him (or he is badly hurt).
                Technique::Guard => {
                    if distance < 25000 && (threatened || hp_pct < 35) {
                        30 + (100 - hp_pct) / 2 + p[5] / 5 + p[10] / 8
                            + if intent == "defend" { 20 } else { 0 }
                    } else {
                        -100
                    }
                }
                // Slip either gives access (closing a gap he can't punch across) or avoids a threat.
                Technique::Slip => {
                    let access = distance > reach;
                    let avoid = distance < 6000 && (threatened || hp_pct < 35);
                    if access || avoid {
                        25 + p[2] / 5 + p[5] / 6 - p[7] / 10
                            + if hp_pct < 35 { 45 } else { 0 }
                            + if access && in_duel { 35 } else { 0 }
                    } else {
                        -50
                    }
                }
                Technique::Combination => {
                    if distance <= number("combination", "reach") + dash_travel(t) {
                        65 + p[6] / 5 + p[10] / 8 + if in_duel { 15 } else { 0 }
                    } else {
                        -100
                    }
                }
                Technique::BodyShot => {
                    if distance <= number("bodyShot", "reach") + dash_travel(t) {
                        55 + p[7] / 8 + p[12] / 10
                            + if in_duel { 15 } else { 0 }
                            + if punish { 20 } else { 0 }
                    } else {
                        -100
                    }
                }
                Technique::Uppercut => {
                    if distance <= number("uppercut", "reach") + dash_travel(t) {
                        60 + p[10] / 6 + p[12] / 8
                            + if in_duel { 15 } else { 0 }
                            + if punish { 20 } else { 0 }
                    } else {
                        -100
                    }
                }
                Technique::MainEvent => {
                    if enemy.champion && centre <= 65000 && self.arena_target.is_none() {
                        25 + p[0] / 3 + p[8] / 8 + p[9] / 10 + p[11] / 10 + p[12] / 5
                            - (danger - allies).max(0) * (150 - p[0]) / 10
                            + error
                            - if m.hp * 100 / m.max_hp < 30 { 100 } else { 0 }
                    } else {
                        -150
                    }
                }
            };
            candidates.push((score, a));
        }
        candidates.sort_by_key(|x| std::cmp::Reverse(x.0));
        let horizon = (1 + 2 * p[12] / 100) as usize;
        // Every continuation is applied to a separate core/frame, so charge use and shared
        // cooldowns are real constraints. Forecasts never mutate the authoritative world.
        let mut beam = vec![(self.clone(), f.clone(), 0i64, Vec::<Action>::new())];
        let mut evaluated = 0usize;
        for depth in 0..horizon {
            let quota = (64 - evaluated) / (horizon - depth).max(1);
            let mut expanded = vec![];
            let mut level_count = 0;
            for (node, world, total, line) in &beam {
                for (base, action) in &candidates {
                    if level_count >= quota || evaluated >= 64 {
                        break;
                    }
                    if *base <= 0 || node.valid(world, action).is_err() {
                        continue;
                    }
                    if !matches!(action.technique, Technique::Guard | Technique::Slip)
                        && action.target.and_then(|id| world.enemy(id)).is_none()
                    {
                        continue;
                    }
                    let mut next = node.clone();
                    let mut forecast = world.clone();
                    forecast.manual = true;
                    forecast.basics.clear();
                    forecast.receipts.clear();
                    forecast.action = None;
                    let mut commands = vec![];
                    if !next.accept(&forecast, action.clone(), &mut commands) {
                        continue;
                    }
                    evaluated += 1;
                    level_count += 1;
                    let end = next.cast.as_ref().unwrap().until;
                    let duration = end - forecast.tick;
                    forecast.tick = end;
                    for u in &mut forecast.units {
                        if u.id != f.owner && u.visible {
                            u.pos.x += u.velocity.x * duration as i64;
                            u.pos.y += u.velocity.y * duration as i64;
                        }
                    }
                    commands.extend(next.step(&forecast));
                    let mut delivered = 0;
                    let mut kills = 0;
                    for command in commands {
                        match command {
                            Effect::Damage { target, amount, .. } => {
                                if let Some(u) = forecast.units.iter_mut().find(|u| u.id == target)
                                {
                                    let n = amount.min(u.hp.max(0));
                                    u.hp = (u.hp - amount).max(0);
                                    delivered += n;
                                    kills += i64::from(u.hp == 0);
                                }
                            }
                            Effect::Move { pos } => {
                                if let Some(u) = forecast.units.iter_mut().find(|u| u.id == f.owner)
                                {
                                    u.pos = pos;
                                }
                            }
                            _ => {}
                        }
                    }
                    let own = forecast.me().unwrap();
                    let exposure = forecast
                        .units
                        .iter()
                        .filter(|u| {
                            u.team != own.team
                                && u.champion
                                && u.visible
                                && u.hp > 0
                                && u.pos.distance2(own.pos) < 35000_i128.pow(2)
                        })
                        .count() as i64;
                    let defensive = matches!(action.technique, Technique::Slip | Technique::Guard);
                    let risk = exposure * (160 - p[0]) * (if defensive { 1 } else { 2 }) / 20;
                    let choice_error = ((hash.rotate_left(evaluated as u32 % 63) % 41) as i64 - 20)
                        * (100 - p[1])
                        / 100;
                    let score = total + base + delivered / 8 + kills * 80 - risk + choice_error
                        - duration as i64 / 6;
                    let mut next_line = line.clone();
                    next_line.push(action.clone());
                    expanded.push((next, forecast, score, next_line));
                }
            }
            if expanded.is_empty() {
                break;
            }
            expanded.sort_by_key(|v| std::cmp::Reverse(v.2));
            expanded.truncate(8);
            beam = expanded;
        }
        self.decision.candidates = evaluated;
        let chosen = beam
            .iter()
            .max_by_key(|n| n.2)
            .map(|n| n.3.clone())
            .unwrap_or_default();
        self.decision.selected = chosen.iter().map(|a| a.technique).collect();
        self.decision.reason = format!(
            "{} {} target {}; {} threats, {} allied follow-up; stamina {}; Macro {} Awareness {}",
            intent,
            if in_duel { "duel" } else { "visible" },
            enemy.id,
            danger,
            allies,
            self.stamina / 1000,
            p[9],
            p[11]
        );
        chosen.into_iter().next()
    }
}
