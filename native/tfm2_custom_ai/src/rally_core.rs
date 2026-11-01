//! Rally's deterministic simulation. No host handles, clocks, filesystem or combat RNG.
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
#[path = "rally_data.rs"]
pub mod data;
use data::*;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Point {
    pub x: i64,
    pub y: i64,
}
impl Point {
    pub fn distance(self, b: Self) -> i64 {
        isqrt(
            (self.x - b.x) as i128 * (self.x - b.x) as i128
                + (self.y - b.y) as i128 * (self.y - b.y) as i128,
        )
    }
    pub fn toward(self, b: Self, n: i64) -> Self {
        let d = self.distance(b);
        if d == 0 {
            return self;
        }
        let n = n.min(d).max(0);
        Self {
            x: self.x + (b.x - self.x) * n / d,
            y: self.y + (b.y - self.y) * n / d,
        }
    }
}
fn isqrt(n: i128) -> i64 {
    if n <= 0 {
        return 0;
    }
    let (mut lo, mut hi) = (0i128, n.min(i64::MAX as i128));
    while lo < hi {
        let m = (lo + hi + 1) / 2;
        if m * m <= n {
            lo = m
        } else {
            hi = m - 1
        }
    }
    lo as i64
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
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
    pub fn touches(self, p: Point, r: i64) -> bool {
        p.x + r >= self.min.x
            && p.y + r >= self.min.y
            && p.x - r <= self.max.x
            && p.y - r <= self.max.y
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    #[default]
    Spiker,
    Setter,
    Receiver,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionKind {
    Receive,
    SelfSet,
    AllySet,
    Approach,
    Takeoff,
    Spike,
    Ultimate,
    BlackThunder,
    Spiker,
    Setter,
    Receiver,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Action {
    pub kind: ActionKind,
    #[serde(default)]
    pub target: Option<usize>,
    #[serde(default)]
    pub aim: Point,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Unit {
    pub id: usize,
    pub team: usize,
    pub pos: Point,
    #[serde(default)]
    pub velocity: Point,
    pub hp: i64,
    pub max_hp: i64,
    pub attack: i64,
    pub radius: i64,
    pub champion: bool,
    #[serde(default = "default_damageable")]
    pub damageable: bool,
    pub visible: bool,
    pub targetable: bool,
    #[serde(default)]
    pub farmable: bool,
    #[serde(default)]
    pub controlled: bool,
    #[serde(default)]
    pub cc_immune: bool,
    #[serde(default = "default_damageable")]
    pub control_eligible: bool,
}
fn default_damageable() -> bool {
    true
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Basic {
    pub serial: u64,
    pub target: usize,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Receipt {
    pub serial: u64,
    pub source: usize,
    pub delivered: i64,
    pub reduction: i64,
    pub champion: bool,
    pub verified: bool,
    #[serde(default)]
    pub reflected: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SupportReceipt {
    pub serial: u64,
    pub ally: usize,
    pub champion: bool,
    pub positive: bool,
    pub verified: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Hit {
    pub serial: u64,
    pub target: usize,
    pub delivered: bool,
    #[serde(default)]
    pub champion: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Disk {
    pub pos: Point,
    pub radius: i64,
}
impl Disk {
    fn intersects(&self, a: Point, b: Point) -> bool {
        let (dx, dy) = ((b.x - a.x) as i128, (b.y - a.y) as i128);
        let (vx, vy) = ((self.pos.x - a.x) as i128, (self.pos.y - a.y) as i128);
        let n = dx * dx + dy * dy;
        if n == 0 {
            return vx * vx + vy * vy <= (self.radius as i128).pow(2);
        }
        let k = (vx * dx + vy * dy).clamp(0, n);
        (vx * n - dx * k).pow(2) + (vy * n - dy * k).pow(2) <= (self.radius as i128).pow(2) * n * n
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Frame {
    pub version: u8,
    pub tick: u64,
    pub owner: usize,
    pub seed: u64,
    pub units: Vec<Unit>,
    pub blocks: Vec<Rect>,
    pub smoke: Vec<Rect>,
    pub smoke_disks: Vec<Disk>,
    pub bounds: Rect,
    pub profile: [i64; 13],
    pub manual: bool,
    pub action: Option<Action>,
    pub steer: Point,
    pub move_step: i64,
    pub skill1: bool,
    pub skill2: bool,
    pub ultimate: bool,
    pub interrupted: bool,
    pub rooted: bool,
    pub danger: i64,
    pub uncertainty: i64,
    pub basics: Vec<Basic>,
    pub receipts: Vec<Receipt>,
    pub support_receipts: Vec<SupportReceipt>,
    pub hits: Vec<Hit>,
    pub incoming_verified: bool,
    pub support_verified: bool,
}
impl Default for Frame {
    fn default() -> Self {
        Self {
            version: 1,
            tick: 0,
            owner: 1,
            seed: 1,
            units: vec![],
            blocks: vec![],
            smoke: vec![],
            smoke_disks: vec![],
            bounds: Rect {
                min: Point { x: 0, y: 0 },
                max: Point { x: MAP, y: MAP },
            },
            profile: [50; 13],
            manual: true,
            action: None,
            steer: Point::default(),
            move_step: 1050,
            skill1: true,
            skill2: true,
            ultimate: true,
            interrupted: false,
            rooted: false,
            danger: 0,
            uncertainty: 0,
            basics: vec![],
            receipts: vec![],
            support_receipts: vec![],
            hits: vec![],
            incoming_verified: false,
            support_verified: false,
        }
    }
}
impl Frame {
    pub fn me(&self) -> Option<&Unit> {
        self.units.iter().find(|u| u.id == self.owner)
    }
    pub fn legal(&self, p: Point, r: i64) -> bool {
        self.bounds.contains(p, r) && !self.blocks.iter().any(|b| b.touches(p, r))
    }
    pub fn clear(&self, a: Point, b: Point, r: i64, sight: bool) -> bool {
        if sight && self.smoke_disks.iter().any(|s| s.intersects(a, b)) {
            return false;
        }
        let n = (a.distance(b) / 1000 + 1).max(1);
        (0..=n).all(|i| {
            let p = Point {
                x: a.x + (b.x - a.x) * i / n,
                y: a.y + (b.y - a.y) * i / n,
            };
            self.legal(p, r) && (!sight || !self.smoke.iter().any(|s| s.touches(p, 0)))
        })
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Phase {
    Idle,
    Prepare {
        action: Action,
        until: u64,
    },
    Approach {
        start: u64,
        origin: Point,
        aim: Point,
        enhanced: bool,
        black: bool,
        intended: u64,
    },
    Air {
        start: u64,
        origin: Point,
        aim: Point,
        max_height: i64,
        perfect: bool,
        enhanced: bool,
        black: bool,
        attempted: bool,
        intended: u64,
        hold_start: Option<u64>,
        hold_ticks: u64,
        serve: bool,
        white: bool,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Ball {
    Held,
    Toss {
        start: u64,
        origin: Point,
        to: Point,
        peak: i64,
    },
    Flying {
        serial: u64,
        origin: Point,
        pos: Point,
        to: Point,
        amount: i64,
        basic: bool,
        speed: i64,
        caught: i64,
        height: i64,
        black: bool,
        thunder: bool,
        awakened: bool,
        white: bool,
        serve: bool,
    },
    Returning {
        pos: Point,
    },
    Grounded {
        pos: Point,
    },
    Absent,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Buff {
    pub target: usize,
    pub channel: String,
    pub attack: i64,
    pub speed: i64,
    pub reduce: i64,
    pub until: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Effect {
    Transform,
    WhiteTransform,
    KnockUp {
        target: usize,
        ticks: u64,
    },
    Slow {
        target: usize,
        percent: i64,
        ticks: u64,
    },
    Damage {
        serial: u64,
        target: usize,
        amount: i64,
        basic: bool,
        source: String,
    },
    Move {
        pos: Point,
    },
    Shield {
        amount: i64,
        ticks: u64,
    },
    Buff {
        buff: Buff,
        remove: bool,
    },
    Warning {
        serial: u64,
        pos: Point,
        radius: i64,
        until: u64,
    },
    ClearWarning {
        serial: u64,
    },
    Visual {
        name: String,
        pos: Point,
        height: i64,
    },
    Reject {
        reason: String,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Plan {
    pub actions: Vec<ActionKind>,
    pub score: i64,
    pub reason: String,
    pub aim: Point,
    pub target: Option<usize>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Serve {
    pub origin: Point,
    pub aim: Point,
    pub until: u64,
    pub committed: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Crater {
    pub pos: Point,
    pub radius: i64,
    pub until: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Core {
    pub transformed: bool,
    pub white_storm: bool,
    pub serve: Option<Serve>,
    pub craters: Vec<Crater>,
    pub role: Role,
    pub phase: Phase,
    pub ball: Ball,
    pub flow: i64,
    pub charges: i64,
    pub charge_progress: u64,
    pub caught: i64,
    pub caught_until: u64,
    pub receive_start: Option<u64>,
    pub cds: [u64; 5],
    pub set_ready: u64,
    pub helper: Option<(Point, u64)>,
    pub support: Option<(usize, u64, u64)>,
    pub ultimate_until: u64,
    pub awakened: bool,
    pub black_until: u64,
    pub contact_height: i64,
    pub last_spike_height: i64,
    pub last_tick: Option<u64>,
    pub alive: bool,
    pub serial: u64,
    pub last_basic: u64,
    pub last_receipt: u64,
    pub last_support: u64,
    pub last_hit: u64,
    pub buffs: BTreeMap<String, Buff>,
    pub pending_hits: BTreeMap<u64, (bool, bool, bool, bool)>,
    pub rewarded: BTreeSet<u64>,
    pub plans: Vec<Plan>,
    pub reaction_until: u64,
    pub situation: Option<usize>,
    pub rejection: String,
    pub incoming_verified: bool,
    pub support_verified: bool,
}
impl Default for Core {
    fn default() -> Self {
        Self {
            transformed: false,
            white_storm: false,
            serve: None,
            craters: vec![],
            role: Role::Spiker,
            phase: Phase::Idle,
            ball: Ball::Held,
            flow: 0,
            charges: 0,
            charge_progress: 0,
            caught: 0,
            caught_until: 0,
            receive_start: None,
            cds: [0; 5],
            set_ready: 0,
            helper: None,
            support: None,
            ultimate_until: 0,
            awakened: false,
            black_until: 0,
            contact_height: 2000,
            last_spike_height: 0,
            last_tick: None,
            alive: false,
            serial: 0,
            last_basic: 0,
            last_receipt: 0,
            last_support: 0,
            last_hit: 0,
            buffs: BTreeMap::new(),
            pending_hits: BTreeMap::new(),
            rewarded: BTreeSet::new(),
            plans: vec![],
            reaction_until: 0,
            situation: None,
            rejection: String::new(),
            incoming_verified: false,
            support_verified: false,
        }
    }
}
impl Core {
    pub fn died(&mut self) -> Vec<Effect> {
        let mut out = vec![];
        for k in self.buffs.keys().cloned().collect::<Vec<_>>() {
            self.remove_buff(&k, &mut out)
        }
        if let Ball::Flying { serial, .. } = self.ball {
            out.push(Effect::ClearWarning { serial })
        }
        self.phase = Phase::Idle;
        self.last_spike_height = 0;
        self.ball = Ball::Absent;
        self.flow = 0;
        self.charges = 0;
        self.charge_progress = 0;
        self.caught = 0;
        self.set_ready = 0;
        self.helper = None;
        self.support = None;
        self.ultimate_until = 0;
        self.awakened = false;
        self.transformed = false;
        self.white_storm = false;
        self.serve = None;
        self.craters.clear();
        self.black_until = 0;
        self.pending_hits.clear();
        self.plans.clear();
        self.alive = false;
        out
    }
    pub fn busy(&self) -> bool {
        !matches!(self.phase, Phase::Idle)
    }
    pub fn flash_blocked(&self) -> bool {
        self.busy()
    }
    pub fn height(&self, t: u64) -> i64 {
        if let Phase::Air {
            start,
            max_height,
            hold_start,
            hold_ticks,
            white,
            ..
        } = self.phase
        {
            let paused = hold_start.map_or(0, |s| t.saturating_sub(s).min(hold_ticks));
            let a = t
                .saturating_sub(start)
                .saturating_sub(paused)
                .min(FLIGHT_TICKS as u64) as i64;
            if white {
                // White Storm blinks to the apex. It never gains invulnerability.
                if a < FLIGHT_TICKS / 2 {
                    2000
                } else {
                    max_height
                }
            } else {
                2000 + 4 * (max_height - 2000) * a * (FLIGHT_TICKS - a)
                    / (FLIGHT_TICKS * FLIGHT_TICKS)
            }
        } else {
            2000
        }
    }
    pub fn reduction(&self, t: u64) -> i64 {
        let mut n = if self.role == Role::Receiver {
            RECEIVER_REDUCTION
                + self.charges * CHARGE_REDUCTION
                + if self.white_storm {
                    WHITE_ROLE_RECEIVE
                } else if self.transformed {
                    BLACK_ROLE_RECEIVE
                } else {
                    0
                }
        } else {
            0
        };
        if let Some(s) = self.receive_start {
            let age = t.saturating_sub(s);
            if age < GUARD_TICKS as u64 {
                n = n.max(GUARD_REDUCTION)
            }
            let perfect = PERFECT_RECEIVE;
            if self.role == Role::Receiver && age < perfect as u64 {
                n = PERFECT_REDUCTION
            }
        }
        n.min(PERFECT_REDUCTION)
    }
    fn role_jump(&self) -> i64 {
        if self.white_storm {
            WHITE_ROLE_JUMP
        } else if self.transformed {
            BLACK_ROLE_JUMP
        } else {
            0
        }
    }
    fn select_role(&mut self, role: Role) {
        if self.role == Role::Receiver && role != Role::Receiver {
            self.caught = 0;
        }
        self.role = role;
    }
    fn end_attempt(&mut self) {
        self.phase = Phase::Idle;
        self.contact_height = 2000;
        self.serve = None;
        self.ultimate_until = 0;
        if self.black_until > 0 {
            self.charge_progress = 0;
        }
        self.black_until = 0;
        self.awakened = false;
    }
    fn next(&mut self) -> u64 {
        self.serial += 1;
        self.serial
    }
    fn reject(&mut self, reason: &str, out: &mut Vec<Effect>) {
        self.rejection = reason.into();
        out.push(Effect::Reject {
            reason: reason.into(),
        });
    }
    fn buff(&mut self, b: Buff, out: &mut Vec<Effect>) {
        let key = format!("{}:{}", b.target, b.channel);
        if self.buffs.get(&key) != Some(&b) {
            self.buffs.insert(key, b.clone());
            out.push(Effect::Buff {
                buff: b,
                remove: false,
            })
        }
    }
    fn remove_buff(&mut self, key: &str, out: &mut Vec<Effect>) {
        if let Some(buff) = self.buffs.remove(key) {
            out.push(Effect::Buff { buff, remove: true })
        }
    }
    fn gain(&mut self, serial: u64) {
        if self.awakened || self.black_until > 0 || !self.rewarded.insert(serial) {
            return;
        }
        self.flow = (self.flow + FLOW_GAIN).min(FLOW_CAP)
    }
    pub fn step(&mut self, f: &Frame) -> Vec<Effect> {
        let mut out = vec![];
        if self.last_tick.is_some_and(|t| f.tick <= t) {
            return out;
        }
        let mut dt = self.last_tick.map_or(0, |t| f.tick - t);
        self.last_tick = Some(f.tick);
        self.incoming_verified = f.incoming_verified;
        self.support_verified = f.support_verified;
        let Some(me) = f.me() else { return out };
        if me.hp <= 0 {
            if self.alive {
                out.extend(self.died());
            }
            return out;
        }
        if !self.alive {
            dt = 0;
            self.alive = true;
            self.ball = Ball::Held;
            self.charge_progress = 0;
        }
        if self.black_until > 0 && f.tick >= self.black_until {
            self.black_until = 0;
            self.awakened = false;
            self.charge_progress = 0;
        }
        if self.ultimate_until > 0 && f.tick >= self.ultimate_until {
            self.ultimate_until = 0;
            self.serve = None;
            self.awakened = self.black_until > f.tick;
            if matches!(
                self.phase,
                Phase::Approach { .. } | Phase::Air { serve: true, .. }
            ) {
                self.phase = Phase::Idle;
            }
        }
        let black_preparing = matches!(&self.phase, Phase::Prepare { action, .. } if action.kind == ActionKind::BlackThunder);
        if self.black_until == 0 && !black_preparing && self.charges < CHARGE_MAX {
            self.charge_progress += dt;
            while self.charge_progress >= CHARGE_TICKS as u64 && self.charges < CHARGE_MAX {
                self.charges += 1;
                self.charge_progress -= CHARGE_TICKS as u64;
            }
            if self.charges == CHARGE_MAX {
                self.charge_progress = 0
            }
        }
        if self.role != Role::Receiver || f.tick >= self.caught_until {
            self.caught = 0
        }
        let cap = BANK_CAP;
        self.caught = self.caught.min(me.max_hp * cap / 100);
        for r in &f.receipts {
            if r.serial <= self.last_receipt {
                continue;
            }
            self.last_receipt = r.serial;
            if self.role != Role::Receiver
                || !f.incoming_verified
                || !r.verified
                || !r.champion
                || r.reflected
                || r.source == f.owner
                || r.delivered <= 0
                || r.reduction < 0
                || r.reduction > PERFECT_REDUCTION
            {
                continue;
            }
            self.caught = (self.caught
                + (r.delivered * 100 / (100 - r.reduction)).min(me.max_hp * BANK_PER_HIT / 100))
            .min(me.max_hp * cap / 100);
            self.caught_until = f.tick + BANK_TICKS as u64;
            if self
                .receive_start
                .is_some_and(|s| f.tick >= s && f.tick - s < PERFECT_RECEIVE as u64)
            {
                self.gain(0x1000000000000000 | self.receive_start.unwrap())
            }
        }
        for h in &f.hits {
            // Impacts can arrive out of release order (a travelling spike and a later palm).
            self.last_hit = self.last_hit.max(h.serial);
            if let Some((eligible, awakened, black, white)) = self.pending_hits.remove(&h.serial) {
                if white && h.delivered && !self.white_storm {
                    self.white_storm = true;
                    self.transformed = true;
                    out.push(Effect::WhiteTransform);
                }
                if black && h.delivered && !self.transformed {
                    self.transformed = true;
                    out.push(Effect::Transform);
                }
                if eligible && !awakened && h.delivered && h.champion {
                    self.gain(h.serial)
                }
            }
        }
        for r in &f.support_receipts {
            if r.serial <= self.last_support {
                continue;
            }
            self.last_support = r.serial;
            if f.support_verified && r.verified && r.champion && r.positive {
                if let Some((ally, end, serial)) = self.support {
                    if ally == r.ally && f.tick < end {
                        self.gain(serial)
                    }
                }
            }
        }
        for key in self
            .buffs
            .iter()
            .filter(|(_, b)| {
                (b.until > 0 && f.tick >= b.until)
                    || ((b.channel == "support" || b.channel == "support_ball")
                        && !f
                            .units
                            .iter()
                            .any(|u| u.id == b.target && u.hp > 0 && u.team == me.team))
            })
            .map(|(k, _)| k.clone())
            .collect::<Vec<_>>()
        {
            self.remove_buff(&key, &mut out)
        }
        if self.support.is_some_and(|(id, end, _)| {
            f.tick >= end
                || !f
                    .units
                    .iter()
                    .any(|u| u.id == id && u.hp > 0 && u.team == me.team)
        }) {
            let legal = self.support.is_some_and(|(id, _, _)| {
                f.units
                    .iter()
                    .any(|u| u.id == id && u.hp > 0 && u.team == me.team)
            });
            self.end_support(&mut out, f.tick, legal);
        }
        if f.interrupted {
            if matches!(
                self.phase,
                Phase::Prepare { .. } | Phase::Approach { .. } | Phase::Air { .. }
            ) {
                self.phase = Phase::Idle;
                self.serve = None;
                self.ultimate_until = 0;
                self.black_until = 0;
                self.awakened = false;
                self.charge_progress = 0;
                self.contact_height = 2000;
                if let Ball::Toss { to, .. } = self.ball {
                    self.ball = Ball::Grounded {
                        pos: self.nearest(f, to),
                    }
                }
                self.reject("Interrupted; commitments remain spent", &mut out);
            }
        }
        if self.helper.is_some_and(|(_, end)| f.tick >= end) {
            self.helper = None;
        }
        self.craters.retain(|c| f.tick < c.until);
        for crater in &self.craters {
            for u in &f.units {
                if u.team != me.team
                    && u.hp > 0
                    && u.targetable
                    && u.damageable
                    && u.control_eligible
                    && !u.cc_immune
                    && crater.pos.distance(u.pos) <= crater.radius + u.radius
                {
                    out.push(Effect::Slow {
                        target: u.id,
                        percent: CRATER_SLOW,
                        ticks: 2,
                    });
                }
            }
        }
        if self.serve.is_some()
            || matches!(&self.phase, Phase::Prepare { action, .. } if action.kind == ActionKind::Ultimate)
        {
            out.push(Effect::Move { pos: me.pos });
        }
        self.update_phase(f, &mut out);
        self.update_ball(f, &mut out);
        if !f.manual && !self.busy() && !f.rooted && !f.interrupted && f.danger < 50 {
            if let Ball::Grounded { pos } = self.ball {
                let to = me.pos.toward(pos, f.move_step);
                if me.pos.distance(pos) > RETRIEVE_RANGE && f.clear(me.pos, to, me.radius, false) {
                    out.push(Effect::Move { pos: to });
                }
            }
        }
        for b in &f.basics {
            if b.serial <= self.last_basic {
                continue;
            }
            self.last_basic = b.serial;
            if self.busy() {
                continue;
            }
            let Some(u) = f
                .units
                .iter()
                .find(|u| u.id == b.target && u.hp > 0 && u.team != me.team)
            else {
                continue;
            };
            let edge = (me.pos.distance(u.pos) - me.radius - u.radius).max(0);
            let serial = self.next();
            if edge <= PALM_RANGE {
                out.push(Effect::Damage {
                    serial,
                    target: u.id,
                    amount: me.attack,
                    basic: true,
                    source: "palm".into(),
                })
            } else if self.ball == Ball::Held {
                self.ball = Ball::Flying {
                    serial,
                    origin: me.pos,
                    pos: me.pos,
                    to: u.pos,
                    amount: me.attack * VOLLEY_PERCENT / 100,
                    basic: true,
                    speed: VOLLEY_SPEED,
                    caught: 0,
                    height: 2000,
                    black: false,
                    thunder: false,
                    awakened: self.awakened,
                    white: false,
                    serve: false,
                };
            }
        }
        let action = if f.manual {
            f.action.clone()
        } else {
            self.decide(f)
        };
        if let Some(a) = action {
            self.accept(f, a, &mut out)
        }
        self.update_buffs(f, &mut out);
        self.contact_height = self.height(f.tick);
        // Receipt/decision history is bounded; serials prevent duplicated old records from paying again.
        if self.rewarded.len() > 256 {
            let floor = self.serial.saturating_sub(128);
            self.rewarded
                .retain(|s| *s >= floor && *s < 0x1000000000000000)
        }
        while self.pending_hits.len() > 64 {
            let k = *self.pending_hits.keys().next().unwrap();
            self.pending_hits.remove(&k);
        }
        out
    }
    fn accept(&mut self, f: &Frame, a: Action, out: &mut Vec<Effect>) {
        let me = f.me().unwrap();
        let t = f.tick;
        if a.kind == ActionKind::Takeoff {
            let Phase::Approach {
                start,
                origin,
                aim,
                enhanced,
                black,
                ..
            } = self.phase.clone()
            else {
                self.reject("No active run-up or Serve", out);
                return;
            };
            if f.rooted {
                self.reject("Movement disabled", out);
                return;
            }
            if self.ball != Ball::Held {
                self.reject("Takeoff requires held combat ball", out);
                return;
            }
            let serve = self
                .serve
                .as_ref()
                .is_some_and(|v| t < v.until && !v.committed);
            let age = t - start;
            let perfect = age >= PERFECT_START as u64
                && age <= PERFECT_END as u64
                && (serve || origin.distance(me.pos) >= RUN_REQUIRED);
            let base = if perfect {
                PERFECT_HEIGHT
            } else if age >= GOOD_START as u64 {
                GOOD_HEIGHT
            } else {
                WEAK_HEIGHT
            };
            let enhanced = enhanced && t < self.set_ready;
            if enhanced {
                self.set_ready = 0;
            }
            let normal = base
                + self.role_jump()
                + self.charges * CHARGE_HEIGHT
                + if enhanced { SET_HEIGHT } else { 0 };
            let mut max_height = if black && perfect {
                BLACK_HEIGHT + self.role_jump()
            } else {
                normal
            };
            if serve {
                max_height = max_height * SERVE_HEIGHT_PERCENT / 100;
            }
            let white = serve && perfect && max_height >= WHITE_MIN_HEIGHT;
            let dir = if serve {
                self.serve.as_ref().unwrap().aim
            } else if f.steer.x != 0 || f.steer.y != 0 {
                Point {
                    x: me.pos.x + f.steer.x * 1000000,
                    y: me.pos.y + f.steer.y * 1000000,
                }
            } else {
                aim
            };
            let contact = if serve {
                me.pos
            } else {
                me.pos.toward(dir, FLIGHT_RANGE / 2)
            };
            let clone = self.nearest(
                f,
                Point {
                    x: (me.pos.x - 18000).max(0),
                    y: (me.pos.y + 12000).min(MAP),
                },
            );
            let to = Point {
                x: (2 * contact.x - clone.x).clamp(f.bounds.min.x, f.bounds.max.x),
                y: (2 * contact.y - clone.y).clamp(f.bounds.min.y, f.bounds.max.y),
            };
            self.ball = Ball::Toss {
                start: t,
                origin: clone,
                to,
                peak: max_height + 100,
            };
            self.helper = Some((clone, t + 24));
            out.push(Effect::Visual {
                name: "toss".into(),
                pos: clone,
                height: 2000,
            });
            let micro = if f.manual { 100 } else { f.profile[10] };
            let error = (hash(f.seed, start) % 11) as i64 - 5;
            let intended = (24 + error * (100 - micro) / 100).max(1) as u64;
            self.phase = Phase::Air {
                start: t,
                origin: me.pos,
                aim: dir,
                max_height,
                perfect,
                enhanced,
                black,
                attempted: false,
                intended,
                hold_start: None,
                hold_ticks: 0,
                serve,
                white,
            };
            if let Some(v) = &mut self.serve {
                v.committed = true;
            }
            return;
        }
        if a.kind == ActionKind::Spike {
            let Phase::Air {
                start,
                max_height,
                perfect,
                enhanced,
                black,
                attempted,
                serve,
                white,
                ..
            } = self.phase.clone()
            else {
                self.reject("Spike requires flight", out);
                return;
            };
            if attempted {
                self.reject("Spike already attempted", out);
                return;
            }
            if serve
                && !self
                    .serve
                    .as_ref()
                    .is_some_and(|v| in_serve_cone(v.origin, v.aim, a.aim))
            {
                self.reject("Serve aim outside the committed V", out);
                return;
            }
            if let Phase::Air { attempted, .. } = &mut self.phase {
                *attempted = true
            }
            let h = self.height(t);
            let ball = match self.ball {
                Ball::Toss {
                    start,
                    origin,
                    to,
                    peak,
                } => Some(toss(start, origin, to, peak, t)),
                _ => None,
            };
            if !ball.is_some_and(|(p, bh)| {
                p.distance(me.pos) <= CONTACT_RADIUS && (bh - h).abs() <= CONTACT_HEIGHT
            }) {
                self.reject("Contact missed: ball outside hand reach", out);
                return;
            }
            self.last_spike_height = h;
            let is_white = white && perfect && h >= WHITE_MIN_HEIGHT;
            let black_height = (BLACK_HEIGHT + self.role_jump())
                * if serve { SERVE_HEIGHT_PERCENT } else { 100 }
                / 100;
            let is_black =
                black && perfect && h >= black_height - 200 && max_height == black_height;
            if self.role == Role::Spiker && h > 4000 {
                if let Phase::Air {
                    hold_start,
                    hold_ticks,
                    ..
                } = &mut self.phase
                {
                    *hold_start = Some(t);
                    *hold_ticks = if is_black || is_white {
                        BLACK_CONTACT_HANG
                    } else {
                        CONTACT_HANG
                    } as u64;
                }
            }
            let setup = if enhanced { 100 + SET_DAMAGE } else { 100 };
            let mult = if is_white {
                WHITE_MULTIPLIER
            } else if is_black {
                BLACK_MULTIPLIER
            } else if self.role == Role::Spiker {
                thunder(h)
            } else {
                100
            };
            let amount = (SPIKE_BASE + me.attack * SPIKE_PERCENT / 100) * setup * mult / 10000;
            let to = me
                .pos
                .toward(a.aim, if serve { SERVE_RANGE } else { SPIKE_RANGE });
            let serial = self.next();
            let target_champion = a
                .target
                .and_then(|id| f.units.iter().find(|u| u.id == id))
                .is_some_and(|u| u.champion && u.team != me.team && u.visible);
            let bank = if target_champion {
                std::mem::take(&mut self.caught)
            } else {
                0
            };
            let speed = SPIKE_SPEED * (if enhanced { 100 + SET_BALL_SPEED } else { 100 }) / 100;
            self.ball = Ball::Flying {
                serial,
                origin: me.pos,
                pos: me.pos,
                to,
                amount,
                basic: false,
                speed,
                caught: bank,
                height: h,
                black: is_black,
                thunder: self.role == Role::Spiker && h > 4000,
                awakened: self.awakened || black,
                white: is_white,
                serve,
            };
            if self.role == Role::Spiker && h > 4000 {
                out.push(Effect::Visual {
                    name: if is_white {
                        "white_blink"
                    } else if is_black {
                        "stroke_black_contact"
                    } else {
                        "stroke_contact"
                    }
                    .into(),
                    pos: me.pos,
                    height: h,
                });
            }
            let until = t + (me.pos.distance(to) + speed - 1) as u64 / speed as u64;
            out.push(Effect::Warning {
                serial,
                pos: to,
                radius: impact_radius(serve, is_black, is_white),
                until,
            });
            out.push(Effect::Visual {
                name: if is_white {
                    "white_release"
                } else if is_black {
                    "black_release"
                } else if self.role == Role::Spiker && h > 4000 {
                    "spike_release"
                } else {
                    "contact"
                }
                .into(),
                pos: me.pos,
                height: h,
            });
            let _ = start;
            return;
        }
        if self.busy() {
            self.reject("Another technique is preparing", out);
            return;
        }
        if f.interrupted {
            self.reject("Controlled or silenced", out);
            return;
        }
        let (slot, cd, prep) = match a.kind {
            ActionKind::Receive => (0, RECEIVE_CD, 0),
            ActionKind::SelfSet | ActionKind::AllySet => (1, SET_CD, SET_PREP),
            ActionKind::Approach => (2, APPROACH_CD, 0),
            ActionKind::Ultimate => (3, ULT_CD, ULT_PREP),
            ActionKind::BlackThunder => (4, BLACK_CD, ULT_PREP),
            ActionKind::Spiker | ActionKind::Setter | ActionKind::Receiver => {
                self.reject(
                    "Roles follow Receive, Set and Run-up; manual stance commands are retired",
                    out,
                );
                return;
            }
            _ => return,
        };
        if t < self.cds[slot] {
            self.reject("Cooldown unavailable", out);
            return;
        }
        if slot <= 2 && !(if slot == 1 { f.skill2 } else { f.skill1 }) {
            self.reject("Skill not learned", out);
            return;
        }
        if slot >= 3 && !f.ultimate {
            self.reject("Ultimate not learned", out);
            return;
        }
        if a.kind == ActionKind::AllySet
            && !a
                .target
                .and_then(|id| f.units.iter().find(|u| u.id == id))
                .is_some_and(|u| {
                    u.id != me.id
                        && u.team == me.team
                        && u.champion
                        && u.hp > 0
                        && u.visible
                        && me.pos.distance(u.pos) <= SET_RANGE
                        && f.clear(me.pos, u.pos, 0, true)
                })
        {
            self.reject("Invalid ally Set: sight, range or realm", out);
            return;
        }
        if a.kind == ActionKind::Approach && f.rooted {
            self.reject("Run-up requires movement", out);
            return;
        }
        if a.kind == ActionKind::SelfSet && self.ball != Ball::Held {
            self.reject("Self Set requires held combat ball", out);
            return;
        }
        if a.kind == ActionKind::BlackThunder
            && (self.charges != CHARGE_MAX
                || self.flow < BLACK_FLOW
                || f.rooted
                || self.ball != Ball::Held
                || !f.skill1
                || !f.skill2
                || (t < self.cds[1] && self.set_ready <= t)
                || t < self.cds[2]
                || self.black_until > t)
        {
            self.reject(
                "Black Thunder needs 3 charges, 60 Flow, held ball and learned/ready Set/S3",
                out,
            );
            return;
        }
        if a.kind == ActionKind::Ultimate
            && (self.ball != Ball::Held || !f.skill1 || a.aim == me.pos)
        {
            self.reject(
                "Serve requires a held ball, Run-up unlock and a firing direction",
                out,
            );
            return;
        }
        self.cds[slot] = t + cd as u64;
        self.rejection.clear();
        match a.kind {
            ActionKind::Receive => self.select_role(Role::Receiver),
            ActionKind::SelfSet | ActionKind::AllySet => self.select_role(Role::Setter),
            ActionKind::Approach | ActionKind::Ultimate | ActionKind::BlackThunder => {
                self.select_role(Role::Spiker)
            }
            _ => {}
        }
        if a.kind == ActionKind::BlackThunder {
            self.charges = 0;
            self.charge_progress = 0;
            self.flow = 0;
        }
        if prep > 0 {
            self.phase = Phase::Prepare {
                action: a,
                until: t + prep as u64,
            };
            return;
        }
        if a.kind == ActionKind::Receive {
            self.receive_start = Some(t);
            out.push(Effect::Shield {
                amount: SHIELD_BASE + me.attack * SHIELD_PERCENT / 100,
                ticks: SHIELD_TICKS as u64,
            });
            out.push(Effect::Visual {
                name: "receive".into(),
                pos: me.pos,
                height: 2000,
            });
        }
        if a.kind == ActionKind::Approach {
            let jitter = (hash(f.seed, t) % 13) as i64 - 6;
            let intended = (PERFECT_START
                + jitter * (100 - if f.manual { 100 } else { f.profile[10] }) / 100)
                .clamp(1, GAUGE_TICKS - 1) as u64;
            self.phase = Phase::Approach {
                start: t,
                origin: me.pos,
                aim: a.aim,
                enhanced: t < self.set_ready,
                black: self.black_until > t,
                intended,
            };
        }
    }
    fn update_phase(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        let me = f.me().unwrap();
        let t = f.tick;
        match self.phase.clone() {
            Phase::Prepare { action, until } if t >= until => {
                self.phase = Phase::Idle;
                match action.kind {
                    ActionKind::SelfSet => {
                        self.helper = Some((
                            self.nearest(
                                f,
                                Point {
                                    x: (me.pos.x - 18000).max(0),
                                    y: (me.pos.y + 12000).min(MAP),
                                },
                            ),
                            t + SELF_SET_TICKS as u64,
                        ));
                        self.set_ready = t + SELF_SET_TICKS as u64;
                        out.push(Effect::Visual {
                            name: "setter_ready".into(),
                            pos: self.helper.map_or(me.pos, |h| h.0),
                            height: 2000,
                        });
                    }
                    ActionKind::AllySet => {
                        self.end_support(out, t, true);
                        if let Some(u) = action.target.and_then(|id| {
                            f.units.iter().find(|u| {
                                u.id == id
                                    && u.hp > 0
                                    && u.team == me.team
                                    && u.champion
                                    && u.visible
                                    && me.pos.distance(u.pos) <= SET_RANGE
                                    && f.clear(me.pos, u.pos, 0, true)
                            })
                        }) {
                            let serial = self.next();
                            self.support = Some((u.id, t + ALLY_TICKS as u64, serial));
                            self.buff(
                                Buff {
                                    target: u.id,
                                    channel: "support_ball".into(),
                                    attack: 0,
                                    speed: 0,
                                    reduce: 0,
                                    until: t + ALLY_TICKS as u64,
                                },
                                out,
                            );
                            self.buff(
                                Buff {
                                    target: u.id,
                                    channel: "support".into(),
                                    attack: ALLY_ATTACK,
                                    speed: ALLY_SPEED,
                                    reduce: 0,
                                    until: t + ALLY_TICKS as u64 + SUPPORT_LINGER as u64,
                                },
                                out,
                            )
                        } else {
                            self.reject("Ally Set invalidated before commitment", out)
                        }
                    }
                    ActionKind::Ultimate => {
                        self.ultimate_until = t + SERVE_TICKS as u64;
                        self.awakened = true;
                        self.serve = Some(Serve {
                            origin: me.pos,
                            aim: action.aim,
                            until: self.ultimate_until,
                            committed: false,
                        });
                        let jitter = (hash(f.seed, t) % 13) as i64 - 6;
                        self.phase = Phase::Approach {
                            start: t,
                            origin: me.pos,
                            aim: action.aim,
                            enhanced: t < self.set_ready,
                            black: self.black_until > t,
                            intended: (PERFECT_START
                                + jitter * (100 - if f.manual { 100 } else { f.profile[10] }) / 100)
                                .clamp(1, GAUGE_TICKS - 1)
                                as u64,
                        };
                        out.push(Effect::Visual {
                            name: "serve_ready".into(),
                            pos: me.pos,
                            height: 2000,
                        });
                    }
                    ActionKind::BlackThunder => {
                        self.black_until = t + BLACK_READY_TICKS as u64;
                        self.awakened = true;
                        out.push(Effect::Visual {
                            name: "black_ready".into(),
                            pos: me.pos,
                            height: 2000,
                        });
                    }
                    _ => {}
                }
            }
            Phase::Approach {
                start,
                aim,
                enhanced,
                ..
            } => {
                if t - start >= RUN_TICKS as u64 && self.serve.is_none() {
                    self.phase = Phase::Idle;
                    if self.serve.is_some() {
                        self.serve = None;
                        self.ultimate_until = 0;
                    }
                } else if self.serve.is_none() && !f.rooted {
                    let dir = if f.steer.x != 0 || f.steer.y != 0 {
                        Point {
                            x: me.pos.x + f.steer.x * 1000000,
                            y: me.pos.y + f.steer.y * 1000000,
                        }
                    } else if f.manual {
                        me.pos
                    } else {
                        aim
                    };
                    let step = f.move_step * RUN_SPEED_PERCENT / 100
                        * (if enhanced && t < self.set_ready {
                            100 + SET_RUN_SPEED
                        } else {
                            100
                        })
                        / 100;
                    let to = me.pos.toward(dir, step);
                    if f.clear(me.pos, to, me.radius, false) {
                        out.push(Effect::Move { pos: to });
                    }
                }
            }
            Phase::Air {
                start,
                origin,
                aim,
                hold_start,
                hold_ticks,
                serve,
                white,
                ..
            } => {
                let paused = hold_start.map_or(0, |s| t.saturating_sub(s).min(hold_ticks));
                if t - start - paused >= FLIGHT_TICKS as u64 {
                    self.end_attempt();
                } else if serve {
                    out.push(Effect::Move { pos: origin });
                    if white
                        && t - start >= FLIGHT_TICKS as u64 / 2
                        && self.contact_height < WHITE_MIN_HEIGHT
                    {
                        out.push(Effect::Visual {
                            name: "white_blink".into(),
                            pos: origin,
                            height: self.height(t),
                        });
                    }
                } else if hold_start.is_some_and(|s| t < s + hold_ticks) {
                    // Keep the host's ordinary walking from competing with the contact hold.
                    out.push(Effect::Move { pos: me.pos });
                } else if !f.rooted {
                    let dir = if f.steer.x != 0 || f.steer.y != 0 {
                        Point {
                            x: me.pos.x + f.steer.x * 1000,
                            y: me.pos.y + f.steer.y * 1000,
                        }
                    } else {
                        aim
                    };
                    let to = me.pos.toward(dir, FLIGHT_RANGE / FLIGHT_TICKS);
                    if origin.distance(to) <= FLIGHT_RANGE && f.clear(me.pos, to, me.radius, false)
                    {
                        out.push(Effect::Move { pos: to })
                    } else {
                        self.end_attempt();
                        self.reject("Flight ended at terrain", out)
                    }
                }
            }
            _ => {}
        }
    }
    fn end_support(&mut self, out: &mut Vec<Effect>, tick: u64, linger: bool) {
        let Some((target, _, _)) = self.support.take() else {
            return;
        };
        let old = self
            .buffs
            .iter()
            .filter(|(_, b)| {
                b.target == target && (b.channel == "support" || b.channel == "support_ball")
            })
            .map(|(k, b)| (k.clone(), b.clone()))
            .collect::<Vec<_>>();
        for (key, mut b) in old {
            if linger && b.channel == "support" {
                b.until = tick + SUPPORT_LINGER as u64;
                self.buff(b, out);
            } else {
                self.remove_buff(&key, out);
            }
        }
    }
    fn nearest(&self, f: &Frame, p: Point) -> Point {
        if f.legal(p, BALL_RADIUS) {
            return p;
        }
        for r in 1..=32 {
            for (x, y) in [
                (r, 0),
                (-r, 0),
                (0, r),
                (0, -r),
                (r, r),
                (-r, r),
                (r, -r),
                (-r, -r),
            ] {
                let q = Point {
                    x: (p.x + x * 1000).clamp(BALL_RADIUS, MAP - BALL_RADIUS),
                    y: (p.y + y * 1000).clamp(BALL_RADIUS, MAP - BALL_RADIUS),
                };
                if f.legal(q, BALL_RADIUS) {
                    return q;
                }
            }
        }
        f.me().unwrap().pos
    }
    fn update_ball(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        let me = f.me().unwrap();
        match self.ball.clone() {
            Ball::Grounded { pos } => {
                if pos.distance(me.pos) <= RETRIEVE_RANGE {
                    self.ball = Ball::Held
                }
            }
            Ball::Returning { pos } => {
                let p = pos.toward(me.pos, RETURN_SPEED);
                self.ball = if p == me.pos {
                    Ball::Held
                } else {
                    Ball::Returning { pos: p }
                }
            }
            Ball::Toss { start, to, .. } => {
                if f.tick >= start + TOSS_TICKS as u64 {
                    self.ball = Ball::Grounded {
                        pos: self.nearest(f, to),
                    }
                }
            }
            Ball::Flying {
                serial,
                origin,
                pos,
                to,
                amount,
                basic,
                speed,
                caught,
                height,
                black,
                awakened,
                thunder,
                white,
                serve,
            } => {
                let next = pos.toward(to, speed);
                let steps = (pos.distance(next) / 500 + 1).max(1);
                let mut impact = None;
                let mut hit = None;
                for i in 1..=steps {
                    let p = Point {
                        x: pos.x + (next.x - pos.x) * i / steps,
                        y: pos.y + (next.y - pos.y) * i / steps,
                    };
                    if !f.legal(p, BALL_RADIUS) {
                        impact = Some(p);
                        break;
                    }
                    if let Some(u) = f
                        .units
                        .iter()
                        .filter(|u| u.team != me.team && u.hp > 0 && u.targetable)
                        .filter(|u| p.distance(u.pos) <= u.radius + BALL_RADIUS)
                        .min_by_key(|u| u.id)
                    {
                        impact = Some(p);
                        hit = Some(u.id);
                        break;
                    }
                }
                if impact.is_none() && next == to {
                    impact = Some(to)
                }
                if let Some(p) = impact {
                    let victims = f
                        .units
                        .iter()
                        .filter(|u| {
                            u.team != me.team
                                && u.hp > 0
                                && u.targetable
                                && (basic || u.damageable)
                                && if basic {
                                    hit == Some(u.id)
                                } else {
                                    p.distance(u.pos)
                                        <= impact_radius(serve, black, white) + u.radius
                                }
                        })
                        .collect::<Vec<_>>();
                    let mut bank = caught;
                    for u in victims {
                        let extra = if u.champion {
                            std::mem::take(&mut bank)
                        } else {
                            0
                        };
                        out.push(Effect::Damage {
                            serial,
                            target: u.id,
                            amount: amount + extra,
                            basic,
                            source: if basic {
                                "volley"
                            } else if white {
                                "white_storm"
                            } else if black {
                                "black_thunder"
                            } else {
                                "spike"
                            }
                            .into(),
                        });
                        if !basic {
                            self.pending_hits
                                .insert(serial, (height > 4000, awakened, black, white));
                        }
                    }
                    if !basic {
                        if serve || black || white {
                            let radius = impact_radius(serve, black, white);
                            for u in &f.units {
                                if u.team != me.team
                                    && u.hp > 0
                                    && u.damageable
                                    && u.targetable
                                    && u.control_eligible
                                    && !u.cc_immune
                                    && p.distance(u.pos) <= radius + u.radius
                                {
                                    out.push(Effect::KnockUp {
                                        target: u.id,
                                        ticks: CRATER_KNOCKUP as u64,
                                    });
                                }
                            }
                            self.craters.push(Crater {
                                pos: p,
                                radius,
                                until: f.tick + CRATER_TICKS as u64,
                            });
                            if self.craters.len() > 4 {
                                self.craters.remove(0);
                            }
                        }
                        if serve {
                            out.push(Effect::Visual {
                                name: "serve_impact_ring".into(),
                                pos: p,
                                height: 0,
                            });
                        }
                        out.push(Effect::ClearWarning { serial });
                        if thunder && !white {
                            out.push(Effect::Visual {
                                name: if black {
                                    "stroke_black_impact"
                                } else {
                                    "stroke_impact"
                                }
                                .into(),
                                pos: p,
                                height: 0,
                            });
                        }
                        out.push(Effect::Visual {
                            name: if white {
                                "white_impact"
                            } else if black {
                                "black_impact"
                            } else {
                                "impact"
                            }
                            .into(),
                            pos: p,
                            height: 0,
                        });
                    }
                    self.ball = if hit.is_some() {
                        Ball::Returning { pos: p }
                    } else {
                        Ball::Grounded {
                            pos: self.nearest(f, p),
                        }
                    };
                } else {
                    self.ball = Ball::Flying {
                        serial,
                        origin,
                        pos: next,
                        to,
                        amount,
                        basic,
                        speed,
                        caught,
                        height,
                        black,
                        awakened,
                        thunder,
                        white,
                        serve,
                    };
                }
            }
            _ => {}
        }
    }
    fn update_buffs(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        let me = f.me().unwrap();
        let reduce = self.reduction(f.tick);
        let rk = format!("{}:reduction", me.id);
        if reduce > 0 {
            self.buff(
                Buff {
                    target: me.id,
                    channel: "reduction".into(),
                    attack: 0,
                    speed: 0,
                    reduce,
                    until: 0,
                },
                out,
            )
        } else {
            self.remove_buff(&rk, out)
        }
        let (atk, spd, red) = (AURA_ATTACK, AURA_SPEED, AURA_REDUCTION);
        let role_bonus = if self.white_storm {
            WHITE_ROLE_SETTER
        } else if self.transformed {
            BLACK_ROLE_SETTER
        } else {
            0
        };
        let eligible = f
            .units
            .iter()
            .filter(|u| {
                self.role == Role::Setter
                    && u.team == me.team
                    && u.champion
                    && u.hp > 0
                    && me.pos.distance(u.pos) <= AURA_RANGE
            })
            .map(|u| u.id)
            .collect::<BTreeSet<_>>();
        for id in &eligible {
            self.buff(
                Buff {
                    target: *id,
                    channel: "aura".into(),
                    attack: atk + self.charges + role_bonus,
                    speed: spd + self.charges + role_bonus,
                    reduce: red + self.charges + role_bonus,
                    until: 0,
                },
                out,
            )
        }
        let leave = self
            .buffs
            .iter()
            .filter(|(_, b)| b.channel == "aura" && b.until == 0 && !eligible.contains(&b.target))
            .map(|(k, b)| (k.clone(), b.clone()))
            .collect::<Vec<_>>();
        for (k, mut b) in leave {
            if b.target == me.id
                || !f
                    .units
                    .iter()
                    .any(|u| u.id == b.target && u.hp > 0 && u.team == me.team)
            {
                self.remove_buff(&k, out)
            } else {
                b.until = f.tick + AURA_LINGER as u64;
                self.buff(b, out)
            }
        }
    }
    fn decide(&mut self, f: &Frame) -> Option<Action> {
        if let Phase::Approach {
            start, intended, ..
        } = self.phase
        {
            if f.tick >= start + intended {
                return Some(Action {
                    kind: ActionKind::Takeoff,
                    target: None,
                    aim: Point::default(),
                });
            }
            return None;
        }
        if let Phase::Air {
            start,
            intended,
            attempted,
            ..
        } = self.phase
        {
            if !attempted && f.tick >= start + intended {
                let enemy = f
                    .units
                    .iter()
                    .filter(|u| {
                        u.team != f.me().unwrap().team
                            && u.visible
                            && u.hp > 0
                            && u.targetable
                            && (u.champion || u.farmable)
                            && self.serve.as_ref().is_none_or(|v| {
                                let aim = predict(u, f);
                                in_serve_cone(v.origin, v.aim, aim)
                                    && v.origin.distance(aim) <= SERVE_RANGE
                            })
                    })
                    .min_by_key(|u| u.pos.distance(f.me().unwrap().pos));
                if let Some(u) = enemy {
                    return Some(Action {
                        kind: ActionKind::Spike,
                        target: Some(u.id),
                        aim: predict(u, f),
                    });
                }
            }
            return None;
        }
        if self.busy() {
            return None;
        }
        let me = f.me().unwrap();
        let enemies = f
            .units
            .iter()
            .filter(|u| {
                u.team != me.team
                    && u.visible
                    && u.hp > 0
                    && u.targetable
                    && (u.champion || u.farmable)
            })
            .collect::<Vec<_>>();
        let nearest = enemies
            .iter()
            .min_by_key(|u| u.pos.distance(me.pos))
            .map(|u| u.id);
        if nearest != self.situation {
            self.situation = nearest;
            self.reaction_until = f.tick + (24 - f.profile[3] * 18 / 100) as u64;
        }
        if f.tick < self.reaction_until {
            return None;
        }
        let mut plans = vec![];
        let p = f.profile;
        if f.danger > 0 && f.tick >= self.cds[0] && f.skill1 {
            plans.push(Plan {
                actions: vec![ActionKind::Receive],
                score: 35 + f.danger / 10 + p[5] / 2 + p[1] / 4 - p[0] / 4,
                reason: "Visible incoming danger: receive".into(),
                aim: me.pos,
                target: None,
            });
        }
        for u in enemies.into_iter().take(16) {
            let distance = me.pos.distance(u.pos);
            if distance > SERVE_RANGE.max(SPIKE_RANGE + FLIGHT_RANGE) {
                continue;
            }
            let incoming = f.danger + (f.uncertainty * p[11] / 100);
            let danger = (incoming + if u.champion { 40 } else { 0 }) * (160 - p[0]) / 100;
            let error = ((hash(f.seed, u.id as u64) % 41) as i64 - 20) * (100 - p[1]) / 100;
            let position = (100 - p[2]) * ((distance - 50000).abs() / 10000) / 5;
            let value = if u.champion {
                70 + p[6] / 3
            } else {
                20 + p[9] / 2
            };
            let coordinated = f
                .units
                .iter()
                .filter(|a| {
                    a.team == me.team
                        && a.champion
                        && a.id != me.id
                        && a.hp > 0
                        && a.pos.distance(u.pos) < 80000
                })
                .count() as i64
                * p[7]
                / 10;
            let rotation = if distance > 65000 {
                p[8] / 5 - p[9] / 8
            } else {
                0
            };
            let horizon = 1 + 2 * p[12] / 100;
            let aim = predict(u, f);
            if f.skill1
                && distance <= SPIKE_RANGE + FLIGHT_RANGE
                && f.tick >= self.cds[2]
                && self.ball == Ball::Held
                && !f.rooted
                && f.clear(me.pos, me.pos.toward(aim, RUN_REQUIRED), me.radius, false)
            {
                let with_set = f.skill2 && f.tick >= self.cds[1] && self.set_ready <= f.tick;
                let mut actions = if with_set {
                    vec![ActionKind::SelfSet, ActionKind::Approach, ActionKind::Spike]
                } else {
                    vec![ActionKind::Approach, ActionKind::Spike]
                };
                let score = value - danger - position
                    + coordinated
                    + rotation
                    + error
                    + if with_set { horizon * 7 } else { 8 };
                actions.truncate(HORIZON as usize);
                plans.push(Plan{actions,score,reason:format!("Spike opportunity: value {value}, danger {danger}, position {position}, follow-up {coordinated}, review {horizon}"),aim,target:Some(u.id)});
            }
            if u.champion && f.ultimate {
                let enough = me.hp > me.max_hp / 3 && distance <= SERVE_RANGE && danger < 70;
                if enough
                    && f.tick >= self.cds[3]
                    && f.tick >= self.cds[1]
                    && f.skill1
                    && f.skill2
                    && self.ball == Ball::Held
                    && self.set_ready <= f.tick
                    && f.clear(me.pos, aim, 0, true)
                {
                    plans.push(Plan {
                        actions: vec![ActionKind::SelfSet, ActionKind::Ultimate, ActionKind::Spike],
                        score: value + 15 + coordinated - danger + error,
                        reason: "Prepare a set for a useful long-range Serve".into(),
                        aim,
                        target: Some(u.id),
                    });
                }
                if enough
                    && f.tick >= self.cds[4]
                    && self.charges == CHARGE_MAX
                    && self.flow >= BLACK_FLOW
                    && self.black_until == 0
                    && self.ball == Ball::Held
                    && f.skill1
                    && f.skill2
                    && (f.tick >= self.cds[1] || self.set_ready > f.tick)
                    && f.tick >= self.cds[2]
                {
                    plans.push(Plan {
                        actions: vec![
                            ActionKind::BlackThunder,
                            ActionKind::SelfSet,
                            ActionKind::Ultimate,
                        ],
                        score: value + 30 + coordinated - danger + error,
                        reason: "Arm resource finisher; compare Serve and a safe close spike"
                            .into(),
                        aim,
                        target: Some(u.id),
                    });
                }
                if enough
                    && f.tick >= self.cds[3]
                    && self.ball == Ball::Held
                    && f.skill1
                    && (self.set_ready > f.tick || self.black_until > f.tick)
                    && f.clear(me.pos, aim, 0, true)
                {
                    plans.push(Plan {
                        actions: vec![ActionKind::Ultimate, ActionKind::Takeoff, ActionKind::Spike],
                        score: value + 20 + coordinated - danger
                            + error
                            + if self.transformed { 15 } else { 0 },
                        reason: "Prepared long-range Serve; safe stationary commitment".into(),
                        aim,
                        target: Some(u.id),
                    });
                }
            }
        }
        if f.skill2 && f.tick >= self.cds[1] {
            for ally in f
                .units
                .iter()
                .filter(|u| {
                    u.id != me.id
                        && u.team == me.team
                        && u.champion
                        && u.hp > 0
                        && u.pos.distance(me.pos) <= SET_RANGE
                        && f.clear(me.pos, u.pos, 0, true)
                })
                .take(8)
            {
                let fighting = f.units.iter().any(|u| {
                    u.champion
                        && u.visible
                        && u.team != me.team
                        && u.hp > 0
                        && u.pos.distance(ally.pos) < 80000
                });
                if fighting {
                    plans.push(Plan {
                        actions: vec![ActionKind::AllySet],
                        score: 15
                            + p[7] / 2
                            + p[12] / 5
                            + if self.role == Role::Setter { 20 } else { 0 },
                        reason: "Support ally's visible champion engagement".into(),
                        aim: ally.pos,
                        target: Some(ally.id),
                    })
                }
            }
        }
        plans.truncate(PLAN_MAX as usize);
        plans.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.target.cmp(&b.target)));
        plans.truncate(BEAM_WIDTH as usize);
        self.plans = plans;
        let best = self.plans.first()?;
        if best.score <= 0 {
            return None;
        }
        Some(Action {
            kind: *best.actions.first()?,
            target: best.target,
            aim: best.aim,
        })
    }
}
/// Fixed sixty-degree Serve sector. Uses integer dot products, never presentation state.
pub fn in_serve_cone(origin: Point, axis: Point, p: Point) -> bool {
    let (ax, ay) = ((axis.x - origin.x) as i128, (axis.y - origin.y) as i128);
    let (bx, by) = ((p.x - origin.x) as i128, (p.y - origin.y) as i128);
    let dot = ax * bx + ay * by;
    let aa = ax * ax + ay * ay;
    let bb = bx * bx + by * by;
    dot > 0 && aa > 0 && bb > 0 && dot * dot * 10000 >= aa * bb * SERVE_CONE_COS_SQUARED as i128
}
pub fn impact_radius(serve: bool, black: bool, white: bool) -> i64 {
    if serve {
        SERVE_RADIUS
    } else if black || white {
        BLACK_RADIUS
    } else {
        SPIKE_RADIUS
    }
}
pub fn flight_height(origin: Point, pos: Point, to: Point, height: i64) -> i64 {
    let total = origin.distance(to).max(1);
    2000 + (height - 2000) * pos.distance(to).min(total) / total
}
pub fn thunder(h: i64) -> i64 {
    if h <= 2000 {
        100
    } else if h <= 4000 {
        100 + (h - 2000) * 10 / 2000
    } else {
        180 + ((h - 4000) * 50 / 800).min(50)
    }
}
pub fn toss(start: u64, origin: Point, to: Point, peak: i64, t: u64) -> (Point, i64) {
    let a = t.saturating_sub(start).min(TOSS_TICKS as u64) as i64;
    let p = Point {
        x: origin.x + (to.x - origin.x) * a / TOSS_TICKS,
        y: origin.y + (to.y - origin.y) * a / TOSS_TICKS,
    };
    (
        p,
        2000 + 4 * (peak - 2000) * a * (TOSS_TICKS - a) / (TOSS_TICKS * TOSS_TICKS),
    )
}
fn hash(seed: u64, salt: u64) -> u64 {
    let mut x = seed ^ salt.wrapping_mul(0x9e3779b97f4a7c15);
    x ^= x >> 30;
    x = x.wrapping_mul(0xbf58476d1ce4e5b9);
    x ^ (x >> 27)
}
fn predict(u: &Unit, f: &Frame) -> Point {
    let lead = 8 * (30 + 70 * f.profile[4] / 100) / 100;
    let e = 6000 * (100 - f.profile[4]) / 100;
    let h = hash(f.seed, u.id as u64);
    Point {
        x: (u.pos.x + u.velocity.x * lead + (h % 3) as i64 * e - e).clamp(0, MAP),
        y: (u.pos.y + u.velocity.y * lead + ((h / 3) % 3) as i64 * e - e).clamp(0, MAP),
    }
}
