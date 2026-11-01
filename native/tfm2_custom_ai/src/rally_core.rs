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
/// Attack tier is chosen only from exact contact height. Permanent form is separate.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttackTier {
    #[default]
    Normal,
    Thunder,
    BlackThunder,
    WhiteStorm,
}
pub fn attack_tier(height: i64) -> AttackTier {
    if height >= WHITE_MIN_HEIGHT {
        AttackTier::WhiteStorm
    } else if height >= BLACK_MIN_HEIGHT {
        AttackTier::BlackThunder
    } else if height >= THUNDER_MIN_HEIGHT {
        AttackTier::Thunder
    } else {
        AttackTier::Normal
    }
}
pub fn tier_speed(tier: AttackTier) -> i64 {
    match tier {
        AttackTier::Normal => SPIKE_SPEED,
        AttackTier::Thunder => THUNDER_SPEED,
        AttackTier::BlackThunder => BLACK_SPEED,
        AttackTier::WhiteStorm => WHITE_SPEED,
    }
}
/// Labels round down so 5.999 m never displays as a 6.0 m Black contact.
pub fn height_label(height: i64) -> i64 {
    (height / 100 * 100).clamp(2000, MAX_HEIGHT_LABEL)
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
    ArmBurst,
    CancelBurst,
    BlackThunder, // Retired reader token; never executes the old command.
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
    /// Separately calibrated force before Rally reduction, after unrelated mitigation.
    #[serde(default)]
    pub incoming_force: Option<i64>,
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
    pub realm: Option<usize>,
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
    pub aim: Option<Point>,
    pub burst_bars: u8,
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
            version: 2,
            realm: None,
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
            aim: None,
            burst_bars: 1,
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
        burst: u8,
    },
    Approach {
        start: u64,
        origin: Point,
        aim: Point,
        enhanced: bool,
        intended: u64,
    },
    Air {
        start: u64,
        origin: Point,
        aim: Point,
        max_height: i64,
        perfect: bool,
        enhanced: bool,
        set_bars: u8,
        spike_bars: u8,
        white_before: bool,
        attempted: bool,
        intended: u64,
        hold_start: Option<u64>,
        hold_ticks: u64,
        serve: bool,
        white: bool,
    },
    Impossible {
        start: u64,
        origin: Point,
        height: i64,
        aim: Point,
        locked: bool,
        warned: Option<(Point, bool)>,
        shot: Shot,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Shot {
    pub serial: u64,
    pub amount: i64,
    pub base_damage: i64,
    pub starting_power: i64,
    pub base_speed: i64,
    pub speed: i64,
    pub caught: i64,
    pub bank_percent: i64,
    pub height: i64,
    pub power: i64,
    pub tier: AttackTier,
    pub serve: bool,
    pub impossible: bool,
    pub burst: bool,
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
        burst: bool,
        white: bool,
        serve: bool,
        impossible: bool,
        power: i64,
        radius: i64,
        overpressure: bool,
        hit: BTreeSet<usize>,
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
        public: bool,
        locked: bool,
        serial: u64,
        pos: Point,
        radius: i64,
        until: u64,
    },
    LineWarning {
        /// A trajectory guide, never an interaction or movement hazard.
        harmless: bool,
        public: bool,
        locked: bool,
        serial: u64,
        from: Point,
        to: Point,
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
    pub bars: u8,
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
    pub tier: AttackTier,
    pub impossible: bool,
    pub pos: Point,
    pub radius: i64,
    pub until: u64,
    pub slow: i64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StormPatch {
    pub target: usize,
    pub serial: u64,
    pub shot: u64,
    pub pos: Point,
    pub radius: i64,
    pub active_at: u64,
    pub until: u64,
}
/// Presentation-only snapshots: no target data or gameplay RNG, and no renewal clock.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PresentationCue {
    pub frame: u8,
    pub serial: u64,
    pub phase: String,
    pub start: u64,
    pub pos: Point,
    pub to: Point,
    pub height: i64,
    pub radius: i64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BurstPlay {
    pub id: u64,
    pub action: ActionKind,
    pub bars: u8,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Core {
    pub charge_presentation: Option<ChargePresentation>,
    pub thunder_earned: bool,
    pub last_spike_tier: AttackTier,
    pub gauge_origin: Option<Point>,
    pub gauge_cycle: u64,
    pub transformed: bool,
    pub white_storm: bool,
    pub serve: Option<Serve>,
    pub craters: Vec<Crater>,
    pub storm: Vec<StormPatch>,
    pub storm_sequence: u64,
    pub storm_shot: u64,
    pub next_storm_tick: u64,
    pub presentation: Vec<PresentationCue>,
    pub role: Role,
    pub phase: Phase,
    pub ball: Ball,
    pub pace: i64,
    pub burst_play: Option<BurstPlay>,
    pub armed_bars: u8,
    pub planned_bars: u8,
    pub armed_until: u64,
    pub exhaustion_until: u64,
    pub receive_until: u64,
    pub receive_perfect_until: u64,
    pub receive_burst: u8,
    pub receive_serial: u64,
    pub bank_base: i64,
    pub bank_burst: bool,
    pub realm: Option<usize>,
    pub caught: i64,
    pub caught_until: u64,
    pub receive_start: Option<u64>,
    pub cds: [u64; 4],
    pub set_ready: u64,
    pub set_bars: u8,
    pub support_burst: bool,
    pub helper: Option<(Point, u64)>,
    pub support: Option<(usize, u64, u64)>,
    pub ultimate_until: u64,
    pub awakened: bool,
    pub last_power: i64,
    pub last_speed: i64,
    pub impossible_ready: bool,
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
            thunder_earned: false,
            last_spike_tier: AttackTier::Normal,
            gauge_origin: None,
            gauge_cycle: 0,
            transformed: false,
            white_storm: false,
            serve: None,
            craters: vec![],
            storm: vec![],
            storm_sequence: 0,
            storm_shot: 0,
            next_storm_tick: 0,
            presentation: vec![],
            charge_presentation: None,
            role: Role::Spiker,
            phase: Phase::Idle,
            ball: Ball::Held,
            pace: 0,
            burst_play: None,
            armed_bars: 0,
            planned_bars: 0,
            armed_until: 0,
            exhaustion_until: 0,
            receive_until: 0,
            receive_perfect_until: 0,
            receive_burst: 0,
            receive_serial: 0,
            bank_base: 0,
            bank_burst: false,
            realm: None,
            caught: 0,
            caught_until: 0,
            receive_start: None,
            cds: [0; 4],
            set_ready: 0,
            set_bars: 0,
            support_burst: false,
            helper: None,
            support: None,
            ultimate_until: 0,
            awakened: false,
            last_power: 100,
            last_speed: SPIKE_SPEED,
            impossible_ready: false,
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
            self.remove_buff(&k, &mut out);
        }
        if let Ball::Flying { serial, .. } = self.ball {
            out.push(Effect::ClearWarning { serial });
        }
        if let Phase::Impossible { ref shot, .. } = self.phase {
            out.push(Effect::ClearWarning {
                serial: shot.serial,
            });
        }
        self.clear_storm(&mut out);
        let storm_sequence = self.storm_sequence;
        let cds = self.cds;
        let tick = self.last_tick;
        let serial = self.serial;
        *self = Self::default();
        self.storm_sequence = storm_sequence;
        self.cds = cds;
        self.last_tick = tick;
        self.serial = serial;
        self.ball = Ball::Absent;
        out
    }
    pub fn busy(&self) -> bool {
        !matches!(self.phase, Phase::Idle)
    }
    pub fn flash_blocked(&self) -> bool {
        self.busy()
    }
    pub fn height(&self, t: u64) -> i64 {
        match self.phase {
            Phase::Impossible { height, .. } => height,
            Phase::Air {
                start,
                max_height,
                hold_start,
                hold_ticks,
                ..
            } => {
                let paused = hold_start.map_or(0, |s| t.saturating_sub(s).min(hold_ticks));
                let a = t
                    .saturating_sub(start)
                    .saturating_sub(paused)
                    .min(FLIGHT_TICKS as u64) as i64;
                2000 + 4 * (max_height - 2000) * a * (FLIGHT_TICKS - a)
                    / (FLIGHT_TICKS * FLIGHT_TICKS)
            }
            _ => 2000,
        }
    }
    pub fn permanent_tier(&self) -> AttackTier {
        if self.white_storm {
            AttackTier::WhiteStorm
        } else if self.transformed {
            AttackTier::BlackThunder
        } else if self.thunder_earned {
            AttackTier::Thunder
        } else {
            AttackTier::Normal
        }
    }
    pub fn ball_speed(&self) -> i64 {
        tier_speed(self.permanent_tier())
    }
    fn earn_contact(&mut self, tier: AttackTier, out: &mut Vec<Effect>) {
        let previous = self.permanent_tier();
        self.thunder_earned |= tier >= AttackTier::Thunder;
        self.transformed |= tier >= AttackTier::BlackThunder;
        self.white_storm |= tier >= AttackTier::WhiteStorm;
        if self.permanent_tier() > previous {
            if tier == AttackTier::WhiteStorm {
                out.push(Effect::WhiteTransform);
            } else if tier == AttackTier::BlackThunder {
                out.push(Effect::Transform);
            }
        }
    }
    pub fn reduction(&self, t: u64) -> i64 {
        let mut n = if self.role == Role::Receiver {
            RECEIVER_REDUCTION
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
        if matches!(self.phase, Phase::Impossible { .. }) {
            n = n.max(IMPOSSIBLE_REDUCTION);
        }
        if self
            .receive_start
            .is_some_and(|s| t >= s && t - s < GUARD_TICKS as u64)
        {
            n = n.max(GUARD_REDUCTION);
        }
        if t < self.receive_perfect_until {
            n = PERFECT_REDUCTION;
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
        self.role = role;
    }
    fn end_attempt(&mut self, t: u64) {
        if self
            .burst_play
            .as_ref()
            .is_some_and(|p| p.action == ActionKind::SelfSet)
        {
            self.set_ready = 0;
            self.set_bars = 0;
        }
        self.resolve_spike(t);
        self.phase = Phase::Idle;
        self.contact_height = 2000;
        self.serve = None;
        self.ultimate_until = 0;
        self.awakened = false;
        self.impossible_ready = false;
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
    fn gain(&mut self, serial: u64, t: u64) {
        if t < self.exhaustion_until || !self.rewarded.insert(serial) {
            return;
        }
        self.pace = (self.pace + PACE_GAIN).min(PACE_CAP);
    }
    pub fn bank_percent(&self) -> i64 {
        if self.bank_base > 0 {
            (self.caught * 100 / self.bank_base).clamp(0, BANK_MAX_PERCENT)
        } else {
            0
        }
    }
    pub fn power(&self, enhanced: bool, set_bars: u8, spike_bars: u8) -> i64 {
        POWER_NORMAL
            + match self.permanent_tier() {
                AttackTier::Normal => 0,
                AttackTier::Thunder => POWER_THUNDER,
                AttackTier::BlackThunder => POWER_BLACK,
                AttackTier::WhiteStorm => POWER_WHITE,
            }
            + if enhanced {
                POWER_SET + set_bars as i64 * BURST_SET_POWER
            } else {
                0
            }
            + spike_bars as i64 * BURST_SPIKE_POWER
    }
    pub fn velocity(&self, power: i64, bank_percent: i64) -> i64 {
        Self::shot_velocity(
            tier_speed(self.permanent_tier()),
            power,
            bank_percent,
            false,
        )
    }
    pub fn power_at(&self, enhanced: bool, set_bars: u8, spike_bars: u8, height: i64) -> i64 {
        self.power(enhanced, set_bars, spike_bars)
            + (height - 2000).max(0) * HEIGHT_POWER_PER_M / 1000
    }
    fn shot_velocity(base: i64, power: i64, bank: i64, impossible: bool) -> i64 {
        (base
            * TICKS
            * power
            * (BANK_MAX_PERCENT + bank.clamp(0, BANK_MAX_PERCENT))
            * if impossible {
                IMPOSSIBLE_SPEED_PERCENT
            } else {
                100
            }
            / (10000 * BANK_MAX_PERCENT))
            .clamp(
                1,
                if impossible {
                    IMPOSSIBLE_MAX_SPEED
                } else {
                    MAX_SPEED
                },
            )
            / TICKS
    }
    pub fn charge_shot(shot: &Shot, elapsed: u64) -> Shot {
        let mut result = shot.clone();
        result.power = shot.starting_power
            + IMPOSSIBLE_POWER_GAIN * elapsed.min(IMPOSSIBLE_CHARGE as u64) as i64
                / IMPOSSIBLE_CHARGE;
        result.amount = shot.base_damage * result.power * IMPOSSIBLE_MULTIPLIER / 10000;
        result.speed = Self::shot_velocity(shot.base_speed, result.power, shot.bank_percent, true);
        result
    }
    fn warn_charge(
        &self,
        shot: &Shot,
        origin: Point,
        to: Point,
        start: u64,
        locked: bool,
        out: &mut Vec<Effect>,
    ) {
        let final_shot = Self::charge_shot(shot, IMPOSSIBLE_CHARGE as u64);
        let until = start
            + IMPOSSIBLE_CHARGE as u64
            + ((origin.distance(to) + final_shot.speed - 1) / final_shot.speed) as u64;
        let radius = impossible_radius(shot.serve)
            * if final_shot.speed * TICKS >= OVERPRESSURE_SPEED {
                OVERPRESSURE_RADIUS
            } else {
                100
            }
            / 100;
        out.push(Effect::Warning {
            public: true,
            locked,
            serial: shot.serial,
            pos: to,
            radius,
            until,
        });
        out.push(Effect::LineWarning {
            harmless: true,
            public: true,
            locked,
            serial: shot.serial,
            from: origin,
            to,
            radius: BALL_RADIUS,
            until,
        });
    }
    pub fn kmh_tenths(speed: i64) -> i64 {
        speed * TICKS * 36 / HEIGHT_WORLD_PER_M
    }
    fn exhaust(&mut self, t: u64, bars: u8) {
        if bars > 0 {
            self.exhaustion_until = self.exhaustion_until.max(t + EXHAUST_TICKS as u64);
        }
    }
    fn resolve_burst(&mut self, t: u64, kinds: &[ActionKind]) {
        if self
            .burst_play
            .as_ref()
            .is_some_and(|p| kinds.contains(&p.action))
        {
            let p = self.burst_play.take().unwrap();
            self.exhaust(t, p.bars);
        }
    }
    fn resolve_spike(&mut self, t: u64) {
        self.resolve_burst(t, &[ActionKind::SelfSet, ActionKind::Spike]);
    }
    fn take_burst(&mut self, t: u64, action: ActionKind) -> u8 {
        if self.burst_play.is_some() || t < self.exhaustion_until || t >= self.armed_until {
            return 0;
        }
        let n = self.armed_bars;
        if ![1, 3, 5].contains(&n) || self.pace < n as i64 * PACE_BAR {
            return 0;
        }
        self.pace -= n as i64 * PACE_BAR;
        self.armed_bars = 0;
        self.armed_until = 0;
        let id = self.next();
        self.burst_play = Some(BurstPlay {
            id,
            action,
            bars: n,
        });
        n
    }
    fn reserve_bank(&mut self) -> (i64, i64, bool) {
        let percent = self.bank_percent();
        let amount = std::mem::take(&mut self.caught);
        self.bank_base = 0;
        self.caught_until = 0;
        let burst = std::mem::take(&mut self.bank_burst);
        (amount, percent, burst)
    }
    pub fn step(&mut self, f: &Frame) -> Vec<Effect> {
        let mut out = vec![];
        if f.version != 2 {
            self.reject("Rally Lab v2 / rally-3.3 required", &mut out);
            return out;
        }
        if self.last_tick.is_some_and(|t| f.tick <= t) {
            return out;
        }
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
        if self.alive && self.realm != f.realm {
            if let Ball::Flying { serial, .. } = self.ball {
                out.push(Effect::ClearWarning { serial });
            }
            if let Phase::Impossible { ref shot, .. } = self.phase {
                out.push(Effect::ClearWarning {
                    serial: shot.serial,
                });
                self.exhaustion_until = self
                    .exhaustion_until
                    .max(f.tick + IMPOSSIBLE_EXHAUST as u64);
            }
            self.end_attempt(f.tick);
            self.ball = Ball::Held;
            self.craters.clear();
            self.presentation.clear();
            self.charge_presentation = None;
            self.helper = None;
            self.pending_hits.clear();
        }
        self.realm = f.realm;
        if !self.alive {
            self.alive = true;
            self.ball = Ball::Held;
        }
        if f.tick >= self.armed_until {
            self.armed_bars = 0;
            self.armed_until = 0;
        }
        if self.receive_burst > 0 && f.tick >= self.receive_until {
            self.resolve_burst(self.receive_until, &[ActionKind::Receive]);
            self.receive_burst = 0;
        }
        if f.tick >= self.caught_until {
            self.caught = 0;
            self.bank_base = 0;
            self.bank_burst = false;
        }
        if self.set_ready > 0 && f.tick >= self.set_ready {
            self.set_bars = 0;
            self.set_ready = 0;
            self.resolve_burst(f.tick, &[ActionKind::SelfSet]);
        }
        if self.ultimate_until > 0 && f.tick >= self.ultimate_until {
            self.ultimate_until = 0;
            self.serve = None;
            if matches!(
                self.phase,
                Phase::Approach { .. } | Phase::Air { serve: true, .. }
            ) {
                self.end_attempt(f.tick);
            }
        }
        let mut receipts = f.receipts.iter().collect::<Vec<_>>();
        receipts.sort_by_key(|r| r.serial);
        for r in receipts {
            if r.serial <= self.last_receipt {
                continue;
            }
            self.last_receipt = r.serial;
            if f.tick >= self.receive_until
                || !f.incoming_verified
                || !r.verified
                || !r.champion
                || r.reflected
                || r.source == f.owner
                || f.units
                    .iter()
                    .any(|u| u.id == r.source && u.team == me.team)
            {
                continue;
            }
            // Do not infer pre-reduction force from aggregate HP/shield loss.
            let Some(force) = r.incoming_force.filter(|n| *n > 0) else {
                continue;
            };
            if self.caught == 0 {
                self.bank_base = SPIKE_BASE + me.attack * SPIKE_PERCENT / 100;
            }
            let cap = self.bank_base * BANK_MAX_PERCENT / 100;
            self.caught = (self.caught + force).min(cap);
            self.bank_burst |= self.receive_burst > 0;
            self.caught_until = f.tick + BANK_TICKS as u64;
            if f.tick < self.receive_perfect_until && self.receive_burst == 0 {
                self.gain(0x1000000000000000 | self.receive_serial, f.tick);
            }
        }
        for h in &f.hits {
            self.last_hit = self.last_hit.max(h.serial);
            if let Some((eligible, burst, _, _)) = self.pending_hits.remove(&h.serial) {
                if eligible && !burst && h.delivered && h.champion {
                    self.gain(h.serial, f.tick);
                }
            }
        }
        for r in &f.support_receipts {
            if r.serial <= self.last_support {
                continue;
            }
            self.last_support = r.serial;
            if f.support_verified && r.verified && r.champion && r.positive && !self.support_burst {
                if let Some((ally, end, serial)) = self.support {
                    if ally == r.ally && f.tick < end {
                        self.gain(serial, f.tick);
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
            self.remove_buff(&key, &mut out);
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
            match self.phase.clone() {
                Phase::Impossible { shot, .. } => {
                    out.push(Effect::ClearWarning {
                        serial: shot.serial,
                    });
                    self.exhaustion_until = self
                        .exhaustion_until
                        .max(f.tick + IMPOSSIBLE_EXHAUST as u64);
                    self.end_attempt(f.tick);
                }
                Phase::Prepare { burst, .. } => {
                    self.resolve_burst(f.tick, &[ActionKind::SelfSet, ActionKind::AllySet]);
                    let _ = burst;
                    self.end_attempt(f.tick);
                }
                Phase::Approach { .. } | Phase::Air { .. } => {
                    self.end_attempt(f.tick);
                }
                _ => {}
            }
            if let Ball::Toss { to, .. } = self.ball {
                self.ball = Ball::Grounded {
                    pos: self.nearest(f, to),
                };
            }
            self.reject("Interrupted; committed costs remain spent", &mut out);
        }
        if self.helper.is_some_and(|(_, end)| f.tick >= end) {
            self.helper = None;
        }
        self.craters.retain(|c| f.tick < c.until);
        let mut slows = BTreeMap::new();
        for crater in &self.craters {
            area_slow(f, crater.pos, crater.radius, crater.slow, &mut slows);
        }
        if self.serve.is_some()
            || matches!(&self.phase,Phase::Prepare{action,..}if action.kind==ActionKind::Ultimate)
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
                });
            } else if self.ball == Ball::Held {
                let (bank, percent, bank_burst) = self.reserve_bank();
                let speed = (VOLLEY_SPEED * TICKS * (BANK_MAX_PERCENT + percent)
                    / BANK_MAX_PERCENT)
                    .min(MAX_SPEED)
                    / TICKS;
                self.ball = Ball::Flying {
                    serial,
                    origin: me.pos,
                    pos: me.pos,
                    to: u.pos,
                    amount: me.attack * VOLLEY_PERCENT / 100,
                    basic: true,
                    speed,
                    caught: bank,
                    height: 2000,
                    black: false,
                    thunder: false,
                    burst: bank_burst,
                    white: false,
                    serve: false,
                    impossible: false,
                    power: 100,
                    radius: 0,
                    overpressure: false,
                    hit: BTreeSet::new(),
                };
            }
        }
        let action = if f.manual {
            f.action.clone()
        } else {
            self.decide(f)
        };
        if let Some(a) = action {
            self.accept(f, a, &mut out);
        }
        self.update_storm(f, &mut out, &mut slows);
        for (target, percent) in slows {
            out.push(Effect::Slow {
                target,
                percent,
                ticks: 2,
            });
        }
        self.presentation
            .retain(|cue| f.tick < cue.start + (IMPACT_DRAWINGS * IMPACT_FRAME_TICKS) as u64);
        for cue in &mut self.presentation {
            cue.frame = ((f.tick - cue.start) / IMPACT_FRAME_TICKS as u64) as u8;
        }
        self.charge_presentation = charge_presentation(&self.phase, f.tick);
        self.update_buffs(f, &mut out);
        self.contact_height = self.height(f.tick);
        self.awakened = self.serve.is_some()
            || self.armed_bars > 0
            || (self.set_bars > 0 && f.tick < self.set_ready)
            || (self.receive_burst > 0 && f.tick < self.receive_until);
        if self.rewarded.len() > 256 {
            let floor = self.serial.saturating_sub(128);
            self.rewarded
                .retain(|n| *n >= floor && *n < 0x1000000000000000);
        }
        while self.pending_hits.len() > 64 {
            let k = *self.pending_hits.keys().next().unwrap();
            self.pending_hits.remove(&k);
        }
        out
    }
    fn clear_storm(&mut self, out: &mut Vec<Effect>) {
        for patch in self.storm.drain(..) {
            out.push(Effect::ClearWarning {
                serial: patch.serial,
            });
        }
        self.storm_shot = 0;
        self.next_storm_tick = 0;
    }
    fn update_storm(&mut self, f: &Frame, out: &mut Vec<Effect>, slows: &mut BTreeMap<usize, i64>) {
        let context = match &self.phase {
            Phase::Impossible {
                start, aim, shot, ..
            } => {
                let final_shot = Self::charge_shot(shot, IMPOSSIBLE_CHARGE as u64);
                let radius = impossible_radius(shot.serve)
                    * if final_shot.speed * TICKS >= OVERPRESSURE_SPEED {
                        OVERPRESSURE_RADIUS
                    } else {
                        100
                    }
                    / 100;
                Some((shot.serial, *start, *aim, radius))
            }
            _ => match &self.ball {
                Ball::Flying {
                    serial,
                    to,
                    radius,
                    impossible: true,
                    ..
                } => Some((*serial, f.tick, *to, *radius)),
                _ => None,
            },
        };
        let Some((shot, start, _centre, _radius)) = context else {
            self.clear_storm(out);
            return;
        };
        if self.storm_shot != shot {
            self.clear_storm(out);
            self.storm_shot = shot;
            self.next_storm_tick = start;
        }
        self.storm.retain(|p| f.tick < p.until);
        if f.tick >= self.next_storm_tick {
            // Skip missed pulses rather than releasing an unbounded catch-up burst.
            self.next_storm_tick = f.tick + STORM_INTERVAL as u64;
            let origin = match &self.phase {
                Phase::Impossible { origin, .. } => *origin,
                _ => match self.ball {
                    Ball::Flying { origin, .. } => origin,
                    _ => f.me().unwrap().pos,
                },
            };
            let global = match &self.phase {
                Phase::Impossible { shot, .. } => shot.serve,
                _ => matches!(self.ball, Ball::Flying { serve: true, .. }),
            };
            let mut targets = f
                .units
                .iter()
                .filter(|u| {
                    u.team != f.me().unwrap().team
                        && u.champion
                        && u.hp > 0
                        && u.visible
                        && u.targetable
                        && u.damageable
                        && f.bounds.contains(u.pos, 0)
                        && (global || origin.distance(u.pos) <= IMPOSSIBLE_RANGE)
                })
                .collect::<Vec<_>>();
            targets.sort_by_key(|u| u.id);
            // Normal teams have five champions. Larger fixtures rotate fairly within the bound.
            let offset = ((f.tick.saturating_sub(start) / STORM_INTERVAL as u64) as usize
                * STORM_COUNT as usize)
                % targets.len().max(1);
            let count = targets.len().min(STORM_COUNT as usize);
            for n in 0..count {
                if self.storm.len() >= STORM_LIMIT as usize {
                    break;
                }
                let target = targets[(offset + n) % targets.len()];
                self.storm_sequence += 1;
                let serial = (1u64 << 52) | self.storm_sequence;
                let salt = hash(
                    f.seed ^ f.owner as u64,
                    shot ^ self.storm_sequence ^ target.id as u64,
                );
                let mut chosen = None;
                for attempt in 0..8 {
                    let h = hash(salt, attempt);
                    let span = 2 * STORM_JITTER + 1;
                    let pos = Point {
                        x: target.pos.x + (h % span as u64) as i64 - STORM_JITTER,
                        y: target.pos.y + ((h >> 24) % span as u64) as i64 - STORM_JITTER,
                    };
                    if f.legal(pos, 0) {
                        chosen = Some(pos);
                        break;
                    }
                }
                let Some(pos) = chosen else { continue };
                let active_at = f.tick + STORM_WARNING as u64;
                let until = active_at + STORM_ACTIVE as u64;
                self.storm.push(StormPatch {
                    target: target.id,
                    serial,
                    shot,
                    pos,
                    radius: STORM_RADIUS,
                    active_at,
                    until,
                });
                out.push(Effect::Warning {
                    public: true,
                    locked: true,
                    serial,
                    pos,
                    radius: STORM_RADIUS,
                    until,
                });
            }
        }
        for patch in &self.storm {
            if f.tick >= patch.active_at {
                area_slow(f, patch.pos, patch.radius, STORM_SLOW, slows);
            }
        }
    }
    fn shot(
        &mut self,
        f: &Frame,
        height: i64,
        enhanced: bool,
        set_bars: u8,
        spike_bars: u8,
        serve: bool,
        impossible: bool,
        out: &mut Vec<Effect>,
    ) -> Shot {
        let tier = attack_tier(height);
        self.earn_contact(tier, out);
        let power = self.power_at(enhanced, set_bars, spike_bars, height);
        let (caught, bank_percent, bank_burst) = self.reserve_bank();
        let base_speed = tier_speed(self.permanent_tier());
        let speed = Self::shot_velocity(base_speed, power, bank_percent, impossible);
        let d = SPIKE_BASE + f.me().unwrap().attack * SPIKE_PERCENT / 100;
        let mult = if impossible {
            IMPOSSIBLE_MULTIPLIER
        } else {
            thunder(height)
        };
        self.last_power = power;
        self.last_speed = speed;
        self.last_spike_tier = tier;
        self.last_spike_height = height;
        Shot {
            serial: self.next(),
            amount: d * power * mult / 10000,
            base_damage: d,
            starting_power: power,
            base_speed,
            speed,
            caught,
            bank_percent,
            height,
            power,
            tier,
            serve,
            impossible,
            burst: set_bars > 0 || spike_bars > 0 || bank_burst,
        }
    }
    fn endpoint(&self, f: &Frame, aim: Point, serve: bool, impossible: bool) -> Point {
        let origin = match self.phase {
            Phase::Impossible { origin, .. } => origin,
            _ => f.me().unwrap().pos,
        };
        let range = if impossible && serve {
            f.bounds.min.distance(f.bounds.max)
        } else if impossible {
            IMPOSSIBLE_RANGE
        } else if serve {
            SERVE_RANGE
        } else {
            SPIKE_RANGE
        };
        let p = origin.toward(aim, range);
        // Clip the ray, not its axes independently: a locked shot must keep its
        // direction inside the fixed Serve V when it reaches map/realm edges.
        let scale = 1_000_000i128;
        let mut fraction = scale;
        for (at, delta, low, high) in [
            (
                origin.x,
                p.x - origin.x,
                f.bounds.min.x + BALL_RADIUS,
                f.bounds.max.x - BALL_RADIUS,
            ),
            (
                origin.y,
                p.y - origin.y,
                f.bounds.min.y + BALL_RADIUS,
                f.bounds.max.y - BALL_RADIUS,
            ),
        ] {
            if delta > 0 {
                fraction =
                    fraction.min(((high - at) as i128 * scale / delta as i128).clamp(0, scale));
            } else if delta < 0 {
                fraction =
                    fraction.min(((low - at) as i128 * scale / delta as i128).clamp(0, scale));
            }
        }
        Point {
            x: origin.x + ((p.x - origin.x) as i128 * fraction / scale) as i64,
            y: origin.y + ((p.y - origin.y) as i128 * fraction / scale) as i64,
        }
    }
    fn launch(&mut self, f: &Frame, aim: Point, shot: Shot, out: &mut Vec<Effect>) {
        let origin = match self.phase {
            Phase::Impossible { origin, .. } => origin,
            _ => f.me().unwrap().pos,
        };
        let to = self.endpoint(f, aim, shot.serve, shot.impossible);
        let overpressure = shot.speed * TICKS >= OVERPRESSURE_SPEED;
        let base_radius = if shot.impossible {
            impossible_radius(shot.serve)
        } else {
            impact_radius(
                shot.serve,
                shot.tier == AttackTier::BlackThunder,
                shot.tier == AttackTier::WhiteStorm,
            )
        };
        let radius = base_radius
            * if overpressure {
                OVERPRESSURE_RADIUS
            } else {
                100
            }
            / 100;
        let until = f.tick + ((origin.distance(to) + shot.speed - 1) / shot.speed) as u64;
        if !shot.impossible {
            out.push(Effect::Warning {
                public: false,
                locked: true,
                serial: shot.serial,
                pos: to,
                radius,
                until,
            });
        }
        out.push(Effect::Visual {
            name: if shot.impossible {
                "impossible_release"
            } else if overpressure {
                "overpressure_release"
            } else if shot.tier == AttackTier::WhiteStorm {
                "white_release"
            } else if shot.tier == AttackTier::BlackThunder {
                "black_release"
            } else if shot.tier == AttackTier::Thunder {
                "spike_release"
            } else {
                "contact"
            }
            .into(),
            pos: origin,
            height: shot.height,
        });
        if shot.impossible {
            self.presentation.push(PresentationCue {
                frame: 0,
                serial: shot.serial,
                phase: "release".into(),
                start: f.tick,
                pos: origin,
                to,
                height: shot.height,
                radius,
            });
        }
        self.ball = Ball::Flying {
            serial: shot.serial,
            origin: origin,
            pos: origin,
            to,
            amount: shot.amount,
            basic: false,
            speed: shot.speed,
            caught: shot.caught,
            height: shot.height,
            black: shot.tier == AttackTier::BlackThunder,
            thunder: shot.tier >= AttackTier::Thunder,
            burst: shot.burst,
            white: shot.tier == AttackTier::WhiteStorm,
            serve: shot.serve,
            impossible: shot.impossible,
            power: shot.power,
            radius,
            overpressure,
            hit: BTreeSet::new(),
        };
    }
    fn accept(&mut self, f: &Frame, a: Action, out: &mut Vec<Effect>) {
        let me = f.me().unwrap();
        let t = f.tick;
        if f.interrupted {
            self.reject("Enemy control prevents new actions", out);
            return;
        }
        if a.kind == ActionKind::CancelBurst {
            self.armed_bars = 0;
            self.armed_until = 0;
            return;
        }
        if a.kind == ActionKind::ArmBurst {
            if matches!(
                self.phase,
                Phase::Prepare { .. } | Phase::Air { .. } | Phase::Impossible { .. }
            ) || t < self.exhaustion_until
                || self.burst_play.is_some()
            {
                self.reject("Burst unavailable during commitment or exhaustion", out);
                return;
            }
            let bars = if f.manual {
                f.burst_bars
            } else {
                self.planned_bars
            };
            if ![1, 3, 5].contains(&bars) || self.pace < (bars as i64) * PACE_BAR {
                self.reject("Choose 1, 3 or 5 complete available Pace bars", out);
                return;
            }
            if !f.skill1 && !f.skill2 {
                self.reject("Learn a volleyball skill first", out);
                return;
            }
            self.armed_bars = bars;
            self.armed_until = t + BURST_ARM_TICKS as u64;
            self.rejection.clear();
            return;
        }
        if a.kind == ActionKind::Takeoff {
            let Phase::Approach {
                start,
                origin,
                aim,
                enhanced,
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
            let age = (t - start) % GAUGE_TICKS as u64;
            let perfect = age >= PERFECT_START as u64
                && age <= PERFECT_END as u64
                && (serve || self.gauge_origin.unwrap_or(origin).distance(me.pos) >= RUN_REQUIRED);
            let base = if perfect {
                PERFECT_HEIGHT
            } else if age >= GOOD_START as u64 {
                GOOD_HEIGHT
            } else {
                WEAK_HEIGHT
            };
            let enhanced = enhanced && t < self.set_ready;
            let set_bars = if enhanced { self.set_bars } else { 0 };
            let spike_bars = self.take_burst(t, ActionKind::Spike);
            if enhanced {
                self.set_ready = 0;
                self.set_bars = 0;
            }
            let mut max_height = base
                + self.role_jump()
                + if enhanced {
                    SET_HEIGHT + set_bars as i64 * BURST_SET_HEIGHT
                } else {
                    0
                }
                + spike_bars as i64 * BURST_SPIKE_HEIGHT;
            if serve {
                max_height = max_height * SERVE_HEIGHT_PERCENT / 100;
            }
            let dir = if serve {
                self.serve.as_ref().unwrap().aim
            } else if f.steer != Point::default() {
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
                    x: (me.pos.x - 18000).max(f.bounds.min.x),
                    y: (me.pos.y + 12000).min(f.bounds.max.y),
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
            let intended =
                (24 + ((hash(f.seed, start) % 11) as i64 - 5) * (100 - micro) / 100).max(1) as u64;
            self.impossible_ready = self.white_storm
                && perfect
                && (set_bars == 5 || spike_bars == 5)
                && (serve || max_height >= IMPOSSIBLE_MIN_HEIGHT);
            self.phase = Phase::Air {
                start: t,
                origin: me.pos,
                aim: dir,
                max_height,
                perfect,
                enhanced,
                set_bars,
                spike_bars,
                white_before: self.white_storm,
                attempted: false,
                intended,
                hold_start: None,
                hold_ticks: 0,
                serve,
                white: false,
            };
            if let Some(v) = &mut self.serve {
                v.committed = true;
            }
            return;
        }
        if a.kind == ActionKind::Spike {
            let Phase::Air {
                max_height,
                perfect,
                enhanced,
                set_bars,
                spike_bars,
                white_before,
                attempted,
                serve,
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
            let impossible = white_before
                && perfect
                && (set_bars == 5 || spike_bars == 5)
                && (serve || self.height(t) >= IMPOSSIBLE_MIN_HEIGHT);
            if let Phase::Air {
                attempted,
                spike_bars,
                ..
            } = &mut self.phase
            {
                *attempted = true;
                *spike_bars = 0;
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
                self.resolve_spike(t);
                self.reject("Contact missed: ball outside hand reach", out);
                return;
            }
            let height = h;
            let _ = max_height;
            let shot = self.shot(
                f, height, enhanced, set_bars, spike_bars, serve, impossible, out,
            );
            if impossible {
                // Contact ends horizontal flight immediately. Do not commit a queued
                // flight step before beginning the stationary charge at this point.
                out.retain(|e| !matches!(e, Effect::Move { .. }));
                out.push(Effect::Move { pos: me.pos });
                let end = t + IMPOSSIBLE_CHARGE as u64 + FLIGHT_TICKS as u64 / 2 + 1;
                if let Some(v) = &mut self.serve {
                    self.ultimate_until = self.ultimate_until.max(end);
                    v.until = v.until.max(end);
                }
                let aim = self.endpoint(f, a.aim, serve, true);
                self.warn_charge(&shot, me.pos, aim, t, false, out);
                self.phase = Phase::Impossible {
                    start: t,
                    origin: me.pos,
                    height,
                    aim,
                    locked: false,
                    warned: Some((aim, false)),
                    shot,
                };
                self.ball = Ball::Held; // Owned, unavailable while the charging phase blocks basics/takeoff.
                out.push(Effect::Visual {
                    name: "impossible_charge".into(),
                    pos: me.pos,
                    height,
                });
            } else {
                if let Phase::Air {
                    white,
                    hold_start,
                    hold_ticks,
                    ..
                } = &mut self.phase
                {
                    *white = shot.tier == AttackTier::WhiteStorm;
                    if shot.tier >= AttackTier::Thunder {
                        *hold_start = Some(t);
                        *hold_ticks = if shot.tier >= AttackTier::BlackThunder {
                            BLACK_CONTACT_HANG
                        } else {
                            CONTACT_HANG
                        } as u64;
                    }
                }
                if shot.tier == AttackTier::WhiteStorm {
                    out.push(Effect::Visual {
                        name: "white_blink".into(),
                        pos: me.pos,
                        height,
                    });
                } else if shot.tier >= AttackTier::Thunder {
                    out.push(Effect::Visual {
                        name: if shot.tier == AttackTier::BlackThunder {
                            "stroke_black_contact"
                        } else {
                            "stroke_contact"
                        }
                        .into(),
                        pos: me.pos,
                        height,
                    });
                }
                self.resolve_spike(t);
                self.launch(f, a.aim, shot, out);
            }
            self.rejection.clear();
            return;
        }
        if self.busy() {
            self.reject("Another technique is committed", out);
            return;
        }
        let (slot, cd, prep) = match a.kind {
            ActionKind::Receive => (0, RECEIVE_CD, 0),
            ActionKind::SelfSet | ActionKind::AllySet => (1, SET_CD, SET_PREP),
            ActionKind::Approach => (2, APPROACH_CD, 0),
            ActionKind::Ultimate => (3, ULT_CD, ULT_PREP),
            _ => {
                self.reject(
                    "Retired action: use skills, Aura Burst and height-based attacks",
                    out,
                );
                return;
            }
        };
        if t < self.cds[slot] {
            self.reject("Cooldown unavailable", out);
            return;
        }
        if slot <= 2 && !(if slot == 1 { f.skill2 } else { f.skill1 }) {
            self.reject("Skill not learned", out);
            return;
        }
        if slot == 3 && !f.ultimate {
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
        if a.kind == ActionKind::Ultimate
            && (self.ball != Ball::Held || !f.skill1 || a.aim == me.pos)
        {
            self.reject("Serve requires held ball, Run-up unlock and direction", out);
            return;
        }
        if a.kind == ActionKind::SelfSet && self.set_ready > t {
            self.resolve_burst(t, &[ActionKind::SelfSet]);
            self.set_ready = 0;
            self.set_bars = 0;
        }
        let burst = if matches!(
            a.kind,
            ActionKind::Receive | ActionKind::SelfSet | ActionKind::AllySet
        ) {
            self.take_burst(t, a.kind.clone())
        } else {
            0
        };
        self.cds[slot] = t + cd as u64;
        self.rejection.clear();
        self.select_role(match a.kind {
            ActionKind::Receive => Role::Receiver,
            ActionKind::SelfSet | ActionKind::AllySet => Role::Setter,
            _ => Role::Spiker,
        });
        if prep > 0 {
            self.phase = Phase::Prepare {
                action: a,
                until: t + prep as u64,
                burst,
            };
            return;
        }
        if a.kind == ActionKind::Receive {
            self.receive_start = Some(t);
            self.receive_serial = self.next();
            self.receive_until =
                t + (RECEIVE_CAPTURE + BURST_RECEIVE_CAPTURE * burst as i64) as u64;
            self.receive_perfect_until =
                t + ((PERFECT_RECEIVE * 10 + BURST_RECEIVE_PERFECT * burst as i64 + 9) / 10) as u64;
            self.receive_burst = burst;
            let amount = (SHIELD_BASE + me.attack * SHIELD_PERCENT / 100)
                * (100 + BURST_RECEIVE_SHIELD * burst as i64)
                / 100;
            out.push(Effect::Shield {
                amount,
                ticks: self.receive_until - t,
            });
            out.push(Effect::Visual {
                name: "receive".into(),
                pos: me.pos,
                height: 2000,
            });
        }
        if a.kind == ActionKind::Approach {
            self.gauge_origin = Some(me.pos);
            self.gauge_cycle = 0;
            let jitter = (hash(f.seed, t) % 13) as i64 - 6;
            self.phase = Phase::Approach {
                start: t,
                origin: me.pos,
                aim: a.aim,
                enhanced: t < self.set_ready,
                intended: (PERFECT_START
                    + jitter * (100 - if f.manual { 100 } else { f.profile[10] }) / 100)
                    .clamp(1, GAUGE_TICKS - 1) as u64,
            };
        }
    }
    fn update_phase(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        let me = f.me().unwrap();
        let t = f.tick;
        match self.phase.clone() {
            Phase::Prepare {
                action,
                until,
                burst,
            } if t >= until => {
                self.phase = Phase::Idle;
                if action.kind == ActionKind::AllySet {
                    self.resolve_burst(t, &[ActionKind::AllySet]);
                }
                match action.kind {
                    ActionKind::SelfSet => {
                        let clone = self.nearest(
                            f,
                            Point {
                                x: (me.pos.x - 18000).max(f.bounds.min.x),
                                y: (me.pos.y + 12000).min(f.bounds.max.y),
                            },
                        );
                        self.helper = Some((clone, t + SELF_SET_TICKS as u64));
                        self.set_ready = t + SELF_SET_TICKS as u64;
                        self.set_bars = burst;
                        out.push(Effect::Visual {
                            name: "setter_ready".into(),
                            pos: clone,
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
                            self.support_burst = burst > 0;
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
                                    attack: ALLY_ATTACK + burst as i64 * BURST_ALLY_ATTACK,
                                    speed: ALLY_SPEED + burst as i64 * BURST_ALLY_SPEED,
                                    reduce: 0,
                                    until: t + (ALLY_TICKS + SUPPORT_LINGER) as u64,
                                },
                                out,
                            );
                        } else {
                            self.reject("Ally Set invalidated before commitment", out);
                        }
                    }
                    ActionKind::Ultimate => {
                        self.ultimate_until = t + SERVE_TICKS as u64;
                        self.serve = Some(Serve {
                            origin: me.pos,
                            aim: action.aim,
                            until: self.ultimate_until,
                            committed: false,
                        });
                        self.gauge_origin = Some(me.pos);
                        self.gauge_cycle = 0;
                        let jitter = (hash(f.seed, t) % 13) as i64 - 6;
                        self.phase = Phase::Approach {
                            start: t,
                            origin: me.pos,
                            aim: action.aim,
                            enhanced: t < self.set_ready,
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
                    _ => {}
                }
            }
            Phase::Approach {
                start,
                aim,
                enhanced,
                ..
            } => {
                let cycle = (t - start) / GAUGE_TICKS as u64;
                if cycle != self.gauge_cycle {
                    self.gauge_cycle = cycle;
                    self.gauge_origin = Some(me.pos);
                }
                if t - start >= RUN_TICKS as u64 && self.serve.is_none() {
                    self.end_attempt(t);
                } else if self.serve.is_none() && !f.rooted {
                    let dir = if f.steer != Point::default() {
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
                        * if enhanced && t < self.set_ready {
                            100 + SET_RUN_SPEED
                        } else {
                            100
                        }
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
                ..
            } => {
                let paused = hold_start.map_or(0, |s| t.saturating_sub(s).min(hold_ticks));
                if t - start - paused >= FLIGHT_TICKS as u64 {
                    self.end_attempt(t);
                } else if serve {
                    out.push(Effect::Move { pos: origin });
                } else if hold_start.is_some_and(|s| t < s + hold_ticks) {
                    out.push(Effect::Move { pos: me.pos });
                } else if !f.rooted {
                    let dir = if f.steer != Point::default() {
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
                        out.push(Effect::Move { pos: to });
                    } else {
                        self.end_attempt(t);
                        self.reject("Flight ended at terrain", out);
                    }
                }
            }
            Phase::Impossible {
                start,
                origin,
                height,
                mut aim,
                mut locked,
                mut warned,
                shot,
            } => {
                out.push(Effect::Move { pos: origin });
                let age = t.saturating_sub(start);
                if age < (IMPOSSIBLE_CHARGE - IMPOSSIBLE_LOCK) as u64 {
                    let proposed = if f.manual {
                        f.aim.unwrap_or(aim)
                    } else {
                        f.units
                            .iter()
                            .filter(|u| {
                                u.team != me.team
                                    && u.visible
                                    && u.targetable
                                    && u.hp > 0
                                    && u.champion
                                    && (shot.serve
                                        || origin.distance(u.pos)
                                            <= IMPOSSIBLE_RANGE + impossible_radius(false))
                            })
                            .take(PLAN_MAX as usize)
                            .max_by_key(|u| {
                                f.units
                                    .iter()
                                    .filter(|v| {
                                        v.team != me.team
                                            && v.visible
                                            && v.hp > 0
                                            && u.pos.distance(v.pos) < IMPOSSIBLE_RADIUS
                                    })
                                    .count()
                            })
                            .map_or(aim, |u| predict(u, f))
                    };
                    aim = self.endpoint(f, proposed, shot.serve, true);
                } else {
                    locked = true;
                }
                let shot = Self::charge_shot(&shot, age);
                self.last_power = shot.power;
                self.last_speed = shot.speed;
                if warned != Some((aim, locked)) {
                    self.warn_charge(&shot, origin, aim, start, locked, out);
                    warned = Some((aim, locked));
                }
                if age >= IMPOSSIBLE_CHARGE as u64 {
                    self.launch(f, aim, shot, out);
                    self.resolve_spike(t);
                    self.phase = Phase::Air {
                        start: t - FLIGHT_TICKS as u64 / 2,
                        origin,
                        aim: origin,
                        max_height: height,
                        perfect: true,
                        enhanced: false,
                        set_bars: 0,
                        spike_bars: 0,
                        white_before: true,
                        attempted: true,
                        intended: 0,
                        hold_start: None,
                        hold_ticks: 0,
                        serve: false,
                        white: true,
                    };
                    self.impossible_ready = false;
                } else {
                    self.phase = Phase::Impossible {
                        start,
                        origin,
                        height,
                        aim,
                        locked,
                        warned,
                        shot,
                    };
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
                    self.ball = Ball::Held;
                }
            }
            Ball::Returning { pos } => {
                let p = pos.toward(me.pos, RETURN_SPEED);
                self.ball = if p == me.pos {
                    Ball::Held
                } else {
                    Ball::Returning { pos: p }
                };
            }
            Ball::Toss { start, to, .. } => {
                if f.tick >= start + TOSS_TICKS as u64 {
                    self.ball = Ball::Grounded {
                        pos: self.nearest(f, to),
                    };
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
                mut caught,
                height,
                black,
                thunder,
                burst,
                white,
                serve,
                impossible,
                power,
                radius,
                overpressure,
                mut hit,
            } => {
                let next = pos.toward(to, speed);
                let wall = if serve || impossible {
                    None
                } else {
                    f.blocks
                        .iter()
                        .filter_map(|r| segment_rect(pos, next, *r, BALL_RADIUS))
                        .min()
                };
                let mut crossings = f
                    .units
                    .iter()
                    .filter(|u| {
                        u.team != me.team && u.hp > 0 && u.targetable && !hit.contains(&u.id)
                    })
                    .filter_map(|u| {
                        segment_disk(pos, next, u.pos, u.radius + BALL_RADIUS).map(|at| (at, u.id))
                    })
                    .collect::<Vec<_>>();
                crossings.sort_unstable();
                let mut impact = None;
                if !impossible {
                    if let Some(&(at, id)) = crossings.first() {
                        if wall.is_none_or(|w| at < w) {
                            impact = Some((segment_point(pos, next, at), Some(id)));
                        }
                    }
                }
                if impact.is_none() {
                    if let Some(at) = wall {
                        impact = Some((segment_point(pos, next, at), None));
                    } else if next == to {
                        impact = Some((to, None));
                    }
                }
                if let Some((p, direct)) = impact {
                    let mut victims = f
                        .units
                        .iter()
                        .filter(|u| {
                            u.team != me.team
                                && u.hp > 0
                                && u.targetable
                                && !hit.contains(&u.id)
                                && if basic {
                                    direct == Some(u.id)
                                } else {
                                    u.damageable && p.distance(u.pos) <= radius + u.radius
                                }
                        })
                        .collect::<Vec<_>>();
                    victims.sort_by_key(|u| (p.distance(u.pos), u.id));
                    for u in victims {
                        hit.insert(u.id);
                        let extra = if u.champion {
                            std::mem::take(&mut caught)
                        } else {
                            0
                        };
                        out.push(Effect::Damage {
                            serial,
                            target: u.id,
                            amount: if impossible {
                                blast_damage(amount, p.distance(u.pos), radius)
                            } else {
                                amount
                            } + extra,
                            basic,
                            source: if basic {
                                "volley"
                            } else if impossible {
                                "impossible_spike"
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
                            self.pending_hits.insert(
                                serial,
                                (height >= THUNDER_MIN_HEIGHT, burst, black, white),
                            );
                        }
                    }
                    if !basic {
                        if serve || black || white || impossible {
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
                                tier: attack_tier(height),
                                impossible,
                                pos: p,
                                radius,
                                until: f.tick
                                    + if black && !impossible && !white {
                                        BLACK_CRATER_TICKS
                                    } else {
                                        CRATER_TICKS
                                    } as u64,
                                slow: if black && !impossible && !white {
                                    BLACK_CRATER_SLOW
                                } else {
                                    CRATER_SLOW
                                },
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
                        if overpressure {
                            out.push(Effect::Visual {
                                name: "overpressure_impact".into(),
                                pos: p,
                                height: 0,
                            });
                        }
                        if thunder && !white && !impossible {
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
                            name: if impossible {
                                "impossible_impact"
                            } else if white {
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
                    self.ball = if !basic || direct.is_some() {
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
                        thunder,
                        burst,
                        white,
                        serve,
                        impossible,
                        power,
                        radius,
                        overpressure,
                        hit,
                    };
                }
            }
            _ => {}
        }
    }
    fn update_buffs(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        let me = f.me().unwrap();
        let ek = format!("{}:exhaustion", me.id);
        if f.tick < self.exhaustion_until {
            self.buff(
                Buff {
                    target: me.id,
                    channel: "exhaustion".into(),
                    attack: 0,
                    speed: -EXHAUST_SLOW,
                    reduce: 0,
                    until: self.exhaustion_until,
                },
                out,
            );
        } else {
            self.remove_buff(&ek, out);
        }
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
                    attack: atk + role_bonus,
                    speed: spd + role_bonus,
                    reduce: red + role_bonus,
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
                            && f.me().unwrap().pos.distance(predict(u, f))
                                <= if self.impossible_ready {
                                    if self.serve.is_some() {
                                        f.bounds.min.distance(f.bounds.max)
                                    } else {
                                        IMPOSSIBLE_RANGE
                                    }
                                } else if self.serve.is_some() {
                                    SERVE_RANGE
                                } else {
                                    SPIKE_RANGE
                                }
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
        let burst_review = self.armed_bars == 0
            && f.tick >= self.exhaustion_until
            && self.pace >= PACE_BAR
            && self.burst_play.is_none();
        let base_budget = PLAN_MAX as usize - if burst_review { 15 } else { 0 };
        let p = f.profile;
        if f.danger > 0 && f.tick >= self.cds[0] && f.skill1 {
            plans.push(Plan {
                bars: 0,
                actions: vec![ActionKind::Receive],
                score: 35 + f.danger / 10 + p[5] / 2 + p[1] / 4 - p[0] / 4,
                reason: "Visible incoming danger: receive".into(),
                aim: me.pos,
                target: None,
            });
        }
        for u in enemies.into_iter().take(16) {
            // Reserve eight support alternatives and all fifteen Burst alternatives before scoring.
            if plans.len() + 5 > base_budget.saturating_sub(8) {
                break;
            }
            let distance = me.pos.distance(u.pos);
            if distance
                > if self.white_storm
                    && (self.pace == PACE_CAP || self.armed_bars == 5 || self.set_bars == 5)
                {
                    f.bounds.min.distance(f.bounds.max)
                } else {
                    SERVE_RANGE.max(SPIKE_RANGE + FLIGHT_RANGE)
                }
            {
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
                plans.push(Plan{bars:0,actions,score,reason:format!("Spike opportunity: value {value}, danger {danger}, position {position}, follow-up {coordinated}, review {horizon}"),aim,target:Some(u.id)});
            }
            let can_serve = self.serve.is_some() || (f.ultimate && f.tick >= self.cds[3]);
            let has_five_set = self.set_bars == 5 && self.set_ready > f.tick;
            let can_set = f.skill2 && f.tick >= self.cds[1] && self.set_ready <= f.tick;
            if u.champion
                && self.white_storm
                && f.skill1
                && self.ball == Ball::Held
                && f.tick >= self.cds[2]
                && f.danger < 70
                && me.hp > me.max_hp / 3
                && (self.pace == PACE_CAP || self.armed_bars == 5 || has_five_set)
                && (can_set || has_five_set || self.serve.is_some())
                && (distance <= IMPOSSIBLE_RANGE || can_serve)
            {
                let armed = self.armed_bars == 5 || has_five_set;
                let next = if !armed {
                    ActionKind::ArmBurst
                } else if !has_five_set && self.serve.is_none() && can_set {
                    ActionKind::SelfSet
                } else if distance > IMPOSSIBLE_RANGE && self.serve.is_none() {
                    ActionKind::Ultimate
                } else {
                    ActionKind::Approach
                };
                plans.push(Plan {
                    bars: if armed { 0 } else { 5 },
                    actions: vec![
                        next,
                        if next == ActionKind::ArmBurst {
                            ActionKind::SelfSet
                        } else {
                            ActionKind::Takeoff
                        },
                        ActionKind::Spike,
                    ],
                    score: value + 25 + p[12] / 3 + coordinated - danger + error,
                    reason: if distance > IMPOSSIBLE_RANGE {
                        "Serve Impossible: prepare global endpoint blast"
                    } else {
                        "Local Impossible: five-bar Set, perfect 10 m contact"
                    }
                    .into(),
                    aim,
                    target: Some(u.id),
                });
            }
            if u.champion && f.ultimate {
                let enough = me.hp > me.max_hp / 3
                    && danger < 70
                    && (distance <= SERVE_RANGE || (self.white_storm && self.pace == PACE_CAP));
                if enough
                    && f.tick >= self.cds[3]
                    && f.tick >= self.cds[1]
                    && f.skill1
                    && f.skill2
                    && self.ball == Ball::Held
                    && self.set_ready <= f.tick
                    && u.visible
                {
                    plans.push(Plan {
                        bars: 0,
                        actions: vec![ActionKind::SelfSet, ActionKind::Ultimate, ActionKind::Spike],
                        score: value + 15 + coordinated - danger + error,
                        reason: "Prepare a set for a useful long-range Serve".into(),
                        aim,
                        target: Some(u.id),
                    });
                }
                if enough
                    && f.tick >= self.cds[3]
                    && self.ball == Ball::Held
                    && f.skill1
                    && self.set_ready > f.tick
                    && u.visible
                {
                    plans.push(Plan {
                        bars: 0,
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
                if plans.len() >= base_budget {
                    break;
                }
                let fighting = f.units.iter().any(|u| {
                    u.champion
                        && u.visible
                        && u.team != me.team
                        && u.hp > 0
                        && u.pos.distance(ally.pos) < 80000
                });
                if fighting {
                    plans.push(Plan {
                        bars: 0,
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
        // Three authored families x five spend counts fit within the 64-candidate budget.
        if burst_review {
            let base = plans.clone();
            for family in [
                ActionKind::Receive,
                ActionKind::SelfSet,
                ActionKind::Approach,
            ] {
                let best = base
                    .iter()
                    .filter(|p| match family {
                        ActionKind::Receive => p.actions.first() == Some(&ActionKind::Receive),
                        ActionKind::SelfSet => {
                            p.actions.first() == Some(&ActionKind::SelfSet)
                                || p.actions.first() == Some(&ActionKind::AllySet)
                        }
                        _ => {
                            p.actions.first() == Some(&ActionKind::Approach)
                                || p.actions.first() == Some(&ActionKind::SelfSet)
                        }
                    })
                    .max_by_key(|p| p.score);
                if let Some(best) = best {
                    for bars in [1u8, 3, 5]
                        .into_iter()
                        .filter(|n| *n as i64 * PACE_BAR <= self.pace)
                    {
                        if plans.len() >= PLAN_MAX as usize {
                            break;
                        }
                        let first = if family == ActionKind::Approach {
                            ActionKind::Approach
                        } else {
                            best.actions[0]
                        };
                        let follow = if first == ActionKind::SelfSet
                            && self.white_storm
                            && bars == 5
                            && f.ultimate
                            && f.tick >= self.cds[3]
                        {
                            ActionKind::Ultimate
                        } else if first == ActionKind::SelfSet {
                            ActionKind::Approach
                        } else {
                            ActionKind::Spike
                        };
                        let benefit = bars as i64 * (5 + p[10] / 20 + p[12] / 25);
                        let reserve = bars as i64 * (100 - p[0]) / 25;
                        let impossible =
                            first == ActionKind::SelfSet && self.white_storm && bars == 5;
                        plans.push(Plan {
                            bars,
                            actions: if matches!(first, ActionKind::Receive | ActionKind::AllySet) {
                                vec![ActionKind::ArmBurst, first]
                            } else {
                                vec![ActionKind::ArmBurst, first, follow]
                            },
                            score: best.score + benefit - reserve
                                + if impossible {
                                    p[12] / 3 - f.danger / 2
                                } else {
                                    0
                                },
                            reason: format!(
                                "Spend {bars} Pace bars on {first:?}; reserve cost {reserve}; {}",
                                if impossible {
                                    "review range, five-second exposure and charged Impossible commitment"
                                } else {
                                    "ordinary Burst"
                                }
                            ),
                            aim: best.aim,
                            target: best.target,
                        });
                    }
                }
            }
        }
        plans.truncate(PLAN_MAX as usize);
        plans.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.target.cmp(&b.target)));
        plans.truncate(BEAM_WIDTH as usize);
        self.plans = plans;
        let best = self.plans.first()?;
        self.planned_bars = best.bars;
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
    match attack_tier(h) {
        AttackTier::Normal => 100,
        AttackTier::Thunder => 180 + ((h - THUNDER_MIN_HEIGHT) * 50 / 800).min(50),
        AttackTier::BlackThunder => BLACK_MULTIPLIER,
        AttackTier::WhiteStorm => WHITE_MULTIPLIER,
    }
}
pub fn toss(start: u64, origin: Point, to: Point, peak: i64, t: u64) -> (Point, i64) {
    let a = t.saturating_sub(start).min(TOSS_TICKS as u64) as i64;
    let p = Point {
        // The clone sends the toss over the contact point early, while the ball
        // keeps rising. This permits lower contacts on a White-capable arc.
        x: origin.x + (to.x - origin.x) * toss_travel(a) / TOSS_TICKS,
        y: origin.y + (to.y - origin.y) * toss_travel(a) / TOSS_TICKS,
    };
    (
        p,
        2000 + 4 * (peak - 2000) * a * (TOSS_TICKS - a) / (TOSS_TICKS * TOSS_TICKS),
    )
}
pub fn toss_travel(age: i64) -> i64 {
    let near = TOSS_TICKS / 2 - 4;
    if age < SET_ARRIVAL_TICKS {
        age * near / SET_ARRIVAL_TICKS
    } else if age <= TOSS_TICKS - SET_ARRIVAL_TICKS {
        near + (age - SET_ARRIVAL_TICKS) * 8 / (TOSS_TICKS - 2 * SET_ARRIVAL_TICKS)
    } else {
        TOSS_TICKS / 2 + 4 + (age - (TOSS_TICKS - SET_ARRIVAL_TICKS)) * near / SET_ARRIVAL_TICKS
    }
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

const SEGMENT_SCALE: i64 = 1_000_000;
pub fn segment_point(a: Point, b: Point, t: i64) -> Point {
    Point {
        x: a.x + (b.x - a.x) * t / SEGMENT_SCALE,
        y: a.y + (b.y - a.y) * t / SEGMENT_SCALE,
    }
}
pub fn segment_disk(a: Point, b: Point, c: Point, r: i64) -> Option<i64> {
    if c.x + r < a.x.min(b.x)
        || c.x - r > a.x.max(b.x)
        || c.y + r < a.y.min(b.y)
        || c.y - r > a.y.max(b.y)
    {
        return None;
    }
    let (dx, dy, fx, fy) = (
        (b.x - a.x) as i128,
        (b.y - a.y) as i128,
        (a.x - c.x) as i128,
        (a.y - c.y) as i128,
    );
    let aa = dx * dx + dy * dy;
    let cc = fx * fx + fy * fy - (r as i128).pow(2);
    if cc <= 0 {
        return Some(0);
    }
    if aa == 0 {
        return None;
    }
    let bb = 2 * (fx * dx + fy * dy);
    let discriminant = bb * bb - 4 * aa * cc;
    if discriminant < 0 {
        return None;
    }
    let n = -bb - isqrt(discriminant) as i128;
    if n < 0 || n > 2 * aa {
        None
    } else {
        Some((n * SEGMENT_SCALE as i128 / (2 * aa)) as i64)
    }
}
pub fn segment_rect(a: Point, b: Point, r: Rect, margin: i64) -> Option<i64> {
    let mut lo = 0i64;
    let mut hi = SEGMENT_SCALE;
    for (p, d, min, max) in [
        (a.x, b.x - a.x, r.min.x - margin, r.max.x + margin),
        (a.y, b.y - a.y, r.min.y - margin, r.max.y + margin),
    ] {
        if d == 0 {
            if p < min || p > max {
                return None;
            }
        } else {
            let x = (min - p) * SEGMENT_SCALE / d;
            let y = (max - p) * SEGMENT_SCALE / d;
            lo = lo.max(x.min(y));
            hi = hi.min(x.max(y));
            if lo > hi {
                return None;
            }
        }
    }
    if hi < 0 || lo > SEGMENT_SCALE {
        None
    } else {
        Some(lo.max(0))
    }
}

pub fn closest_point(a: Point, b: Point, p: Point) -> Point {
    let dx = (b.x - a.x) as i128;
    let dy = (b.y - a.y) as i128;
    let n = dx * dx + dy * dy;
    if n == 0 {
        return a;
    }
    let k = (((p.x - a.x) as i128 * dx + (p.y - a.y) as i128 * dy) * SEGMENT_SCALE as i128 / n)
        .clamp(0, SEGMENT_SCALE as i128) as i64;
    segment_point(a, b, k)
}

pub fn impossible_radius(serve: bool) -> i64 {
    if serve {
        IMPOSSIBLE_RADIUS
    } else {
        IMPOSSIBLE_LOCAL_RADIUS
    }
}
pub fn blast_damage(amount: i64, distance: i64, radius: i64) -> i64 {
    let d = distance.clamp(0, radius.max(1));
    (amount as i128 * (100 * radius.max(1) - (100 - IMPOSSIBLE_EDGE_DAMAGE) * d) as i128
        / (100 * radius.max(1)) as i128) as i64
}
fn area_slow(f: &Frame, pos: Point, radius: i64, percent: i64, slows: &mut BTreeMap<usize, i64>) {
    let Some(me) = f.me() else { return };
    for u in &f.units {
        if u.team != me.team
            && u.hp > 0
            && u.targetable
            && u.damageable
            && u.control_eligible
            && !u.cc_immune
            && pos.distance(u.pos) <= radius + u.radius
        {
            slows
                .entry(u.id)
                .and_modify(|p| *p = (*p).max(percent))
                .or_insert(percent);
        }
    }
}

pub fn crater_style(tier: AttackTier, impossible: bool) -> &'static str {
    if impossible {
        "impossible"
    } else {
        match tier {
            AttackTier::Normal => "normal",
            AttackTier::Thunder => "yellow",
            AttackTier::BlackThunder => "black",
            AttackTier::WhiteStorm => "white",
        }
    }
}

/// Presentation only: the same eight fracture locations in native and Lab output.
pub fn crater_fragments(c: &Crater) -> Vec<Point> {
    let directions = [
        (1000, 0),
        (707, 707),
        (0, 1000),
        (-707, 707),
        (-1000, 0),
        (-707, -707),
        (0, -1000),
        (707, -707),
    ];
    directions
        .iter()
        .map(|(x, y)| Point {
            x: c.pos.x + x * c.radius * 3 / 4000,
            y: c.pos.y + y * c.radius * 3 / 4000,
        })
        .collect()
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChargePresentation {
    pub ball: Point,
    pub height: i64,
    pub stage: u8,
    pub ring_radius: i64,
    pub bolts: Vec<Point>,
}
pub fn charge_presentation(phase: &Phase, tick: u64) -> Option<ChargePresentation> {
    let Phase::Impossible {
        origin,
        aim,
        height,
        start,
        ..
    } = phase
    else {
        return None;
    };
    let stage = (tick.saturating_sub(*start) * 7 / IMPOSSIBLE_CHARGE as u64).min(6) as u8;
    let ball = Point {
        x: origin.x + if aim.x < origin.x { -18000 } else { 18000 },
        y: origin.y,
    };
    let ring_radius = 12000 + stage as i64 * 8000;
    let dirs = [
        (1000, 0),
        (707, 707),
        (0, 1000),
        (-707, 707),
        (-1000, 0),
        (-707, -707),
        (0, -1000),
        (707, -707),
    ];
    let bolts = (0..stage as usize + 2)
        .map(|n| {
            let (x, y) = dirs[(n + (tick / 2) as usize) % 8];
            Point {
                x: ball.x + x * ring_radius / 1000,
                y: ball.y + y * ring_radius / 1000,
            }
        })
        .collect();
    Some(ChargePresentation {
        ball,
        height: *height + 1500,
        stage,
        ring_radius,
        bolts,
    })
}
