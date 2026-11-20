//! Shared Contender combat rules, fight selection and Shadow Link lifecycle.
use crate::contender_shadow_data as d;
use serde::{Deserialize, Serialize};

pub const MAP: i64 = 960_000;
pub const CELL: i64 = 32_000;
pub const CELLS: i64 = 30;
pub const EGO: usize = 0;
pub const AGGRESSIVE: usize = 6;

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
/// Round 190: how long Codex's Shadow Grab latch and burst play (their fades are baked by
/// tools/build-contender-grab-art.py; a test checks these match the packed `.fanim`).
pub const GRAB_LATCH_TICKS: u64 = 8;
pub const GRAB_BURST_TICKS: u64 = 18;

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
    pub radius: i64,
    pub champion: bool,
    pub tower: bool,
    pub visible: bool,
    pub targetable: bool,
    /// a minion or jungle monster the shared farming rules let him spend skills on (and it's calm)
    #[serde(default)]
    pub farmable: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Order {
    S1,
    S2,
    Ult { aim: Point },
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct IncomingHit {
    pub attacker: usize,
    pub from: Point,
    #[serde(default)]
    pub counter: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HitSource {
    Echo,
    Merge,
    Counter,
    Flurry,
    Bonus,
    /// contender-3.6: Shadow Grab's hit
    Grab,
}

impl HitSource {
    pub fn credits_flurry(self) -> bool {
        matches!(self, Self::Echo | Self::Merge | Self::Counter | Self::Grab)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
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
    #[serde(default)]
    pub ult_unlocked: bool,
    /// Champion level (0: unknown); S2 at 2 and the ultimate at 5 (`unlocks.rs`)
    #[serde(default)]
    pub level: i64,
    #[serde(default)]
    pub home: Point,
    pub units: Vec<Unit>,
    /// wall cells (`CELLS` x `CELLS`)
    #[serde(default)]
    pub terrain: Vec<bool>,
    pub profile: [i64; 13],
    pub manual: bool,
    pub order: Option<Order>,
    /// enemies his own (engine) basic attacks landed on since the last frame
    #[serde(default)]
    pub basic_hits: Vec<usize>,
    #[serde(default)]
    pub incoming: Vec<IncomingHit>,
    /// Confirmed non-basic damage impacts from the adapter's previous step.
    #[serde(default)]
    pub landed_hits: u16,
    /// Confirmed Shadow Cross hits, which grant a larger recharge credit.
    #[serde(default)]
    pub landed_merge_hits: u16,
    /// Units that are immune or otherwise ineligible for Link.
    #[serde(default)]
    pub link_excluded: Vec<usize>,
    /// Targets with this owner's active Shadow Link knock-up.
    #[serde(default)]
    pub link_airborne: Vec<usize>,
    /// Units which cannot interact across the current cage or realm.
    #[serde(default)]
    pub separated: Vec<usize>,
    /// Additional wall segments (player-built walls and cage edges).
    #[serde(default)]
    pub barriers: Vec<Barrier>,
    #[serde(default)]
    pub defences: Vec<TargetDefence>,
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
    fn link_target(&self, unit: &Unit) -> bool {
        self.enemy(unit) && !unit.tower && !self.link_excluded.contains(&unit.id)
            && !self.separated.contains(&unit.id)
    }

    fn path_until(&self, from: Point, to: Point, avoid_towers: bool) -> Point {
        let steps = (from.dist(to) as i64 / 1_000 + 1).max(1);
        let origin_tower = if avoid_towers { self.enemy_tower_at(from) } else { None };
        let mut last = from;
        for i in 1..=steps {
            let p = Point::new(from.x + (to.x - from.x) * i / steps, from.y + (to.y - from.y) * i / steps);
            if self.terrain_at(p) || self.barriers.iter().any(|wall| wall.contains(p)) {
                break;
            }
            if avoid_towers && self.units.iter().any(|tower| {
                tower.tower && tower.team != self.team && tower.hp > 0 && tower.pos.within(p, d::AI_TOWER_RANGE)
                    && !origin_tower.is_some_and(|origin| origin.id == tower.id && p.d2(tower.pos) > last.d2(tower.pos))
            }) {
                break;
            }
            last = p;
        }
        last
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
            for k in 0..24 {
                let a = (k as f64 * 15.0).to_radians();
                let q = p.along((a.cos(), a.sin()), r).clamp_map();
                if !self.terrain_at(q) {
                    return q;
                }
            }
        }
        p
    }
    /// The last open point on the straight path from `a` toward `b` (stops before terrain).
    pub fn clear_path(&self, a: Point, b: Point) -> Point {
        let steps = (a.dist(b) as i64 / 4_000).max(1);
        let mut last = a;
        for i in 1..=steps {
            let q = Point::new(a.x + (b.x - a.x) * i / steps, a.y + (b.y - a.y) * i / steps);
            if self.terrain_at(q) {
                return last;
            }
            last = q;
        }
        last
    }
}

// ------------------------------------------------------------------ effects

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Effect {
    Damage { target: usize, amount: i64, source: HitSource },
    /// Shadow Grab's knock-up (`ct_link_air_{owner}`)
    LinkAirborne { target: usize, ticks: u64 },
    /// contender-3.6: Shadow Break's knock-up (`ct_break_air_{owner}`)
    BreakAirborne { target: usize, ticks: u64 },
    /// Pull an enemy along open ground using the host glide.
    Knock { target: usize, to: Point, ticks: u64 },
    /// a deliberate move of his own (an engine glide): dodge slips, the step-out, the self-cleave
    Glide { to: Point, ticks: u64 },
    /// set him there at once (the ult's reappearance)
    /// the ult: unseen, unhurt, rooted and CC immune (a short lease, renewed)
    Vanish { ticks: u64 },
    /// Shadow Merge on him: move speed for `ticks` (`ticks` 1 ends it)
    /// contender-3.5: Merge is a self-buff (move speed, attack speed, Attack, CC immunity)
    Merge { ticks: u64, move_pct: i64, #[serde(default)] attack_speed_pct: i64, #[serde(default)] attack_pct: i64, #[serde(default)] cc_immune: bool },
    /// Shadow Rhythm's buffs on himself
    Rhythm { attack_speed: i64, attack_permille: i64, move_permille: i64, ticks: u64 },
    /// his shadow dodges (for the pips)
    Charges { count: i64 },
    /// a purposeful walk the shared input AI turns into a move order
    Steer { to: Point },
    Fx {
        name: String,
        at: Point,
        ticks: u64,
        /// Optional presentation heading: clockwise eighth-turns from east.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        heading: Option<u8>,
    },
    HoldForward { threat: Point, cover: Point },
    Log { text: String },
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Echo {
    pub target: usize,
    pub at: u64,
    pub amount: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Flurry {
    pub start: u64,
    pub next: u64,
    pub done: i64,
    pub first: usize,
    pub last: Option<(usize, Point)>,
    pub vanished_at: Point,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Stack {
    pub id: usize,
    pub count: i64,
    pub until: u64,
    pub champion: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct LinkPull {
    pub target: usize,
    pub from: Point,
    pub to: Point,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "phase", rename_all = "snake_case")]
/// contender-3.6: the ult is Shadow Grab. The shadow flies in a line, grabs the first enemy it touches, drags it to him
/// and knocks it up. (The type keeps its old name: the bridge and Lab read it.)
pub enum ShadowLink {
    Flight { cast: u64, from: Point, at: Point, cursor: Point, landing: u64 },
    Pull { at: Point, until: u64, pulled: Vec<LinkPull> },
    Airborne { until: u64, targets: Vec<usize> },
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct TargetDefence {
    pub id: usize,
    pub armour: i64,
    pub shield: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Barrier {
    pub from: Point,
    pub to: Point,
    pub half_width: i64,
}

impl Barrier {
    fn contains(self, point: Point) -> bool {
        let dx = (self.to.x - self.from.x) as f64;
        let dy = (self.to.y - self.from.y) as f64;
        let x = (point.x - self.from.x) as f64;
        let y = (point.y - self.from.y) as f64;
        let length = dx * dx + dy * dy;
        let offset = if length > 0.0 { ((x * dx + y * dy) / length).clamp(0.0, 1.0) } else { 0.0 };
        let distance_sq = (x - offset * dx).powi(2) + (y - offset * dy).powi(2);
        distance_sq <= (self.half_width as f64).powi(2)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Core {
    pub charges: i64,
    pub charge_at: u64,
    pub last_dodge: u64,
    pub pending_dodge: Option<IncomingHit>,
    pub merge_at: u64,
    pub merge_until: u64,
    pub merge_cross_at: u64,
    pub shadow_link: Option<ShadowLink>,
    /// contender-3.6: when each enemy may be knocked up by Shadow Break again
    #[serde(default)]
    pub break_knock_ready: Vec<(usize, u64)>,
    pub ult_at: u64,
    pub flurry_cast_at: u64,
    pub flurry: Option<Flurry>,
    pub haymaker_until: u64,
    pub echoes: Vec<Echo>,
    pub stacks: Vec<Stack>,
    pub focus: Option<(usize, i64)>,
    pub linger: Option<(i64, u64)>,
    pub rhythm_shown: (i64, u64),
    pub charges_shown: i64,
    pub steer_at: u64,
    #[serde(default)]
    pub reason: String,
    #[serde(default)]
    pub rejection: String,
}
impl Default for Core {
    fn default() -> Self {
        Core {
            charges: d::CHARGES_MAX,
            charge_at: 0,
            last_dodge: 0,
            pending_dodge: None,
            merge_at: 0,
            merge_until: 0,
            merge_cross_at: 0,
            shadow_link: None,
            break_knock_ready: Vec::new(),
            ult_at: 0,
            flurry_cast_at: 0,
            flurry: None,
            haymaker_until: 0,
            echoes: vec![],
            stacks: vec![],
            focus: None,
            linger: None,
            rhythm_shown: (0, 0),
            charges_shown: -1,
            steer_at: 0,
            reason: String::new(),
            rejection: String::new(),
        }
    }
}

impl Core {
    pub fn respawn(&mut self) {
        self.flurry = None;
        self.echoes.clear();
        self.merge_until = 0;
        self.merge_cross_at = 0;
        self.shadow_link = None;
        self.break_knock_ready.clear();
        self.pending_dodge = None;
        self.haymaker_until = 0;
        self.stacks.clear();
        self.linger = None;
        self.focus = None;
    }

    pub fn merged(&self, tick: u64) -> bool {
        tick < self.merge_until
    }

    pub fn flurrying(&self) -> bool {
        self.flurry.is_some()
    }

    pub fn ult_ready(&self, f: &Frame) -> bool {
        crate::unlocks::ult_unlocked(f.level, f.ult_unlocked) && f.tick >= self.ult_at && self.shadow_link.is_none()
    }

    pub fn flurry_ready(&self, f: &Frame) -> bool {
        crate::unlocks::skill_unlocked(f.level, 1) && f.tick >= self.flurry_cast_at
            && self.charges >= d::CHARGES_FLURRY_COST
    }

    pub fn stacks_on(&self, id: usize, tick: u64) -> i64 {
        self.stacks.iter().find(|s| s.id == id && tick < s.until).map_or(0, |s| s.count)
    }

    pub fn vuln_permille(&self, id: usize, tick: u64) -> i64 {
        self.stacks_on(id, tick) * d::RHYTHM_VULNERABILITY_PERMILLE_PER_STACK
    }

    pub fn try_dodge(&mut self, tick: u64, hit: IncomingHit) -> bool {
        if self.flurrying() || self.charges <= 0
            || (self.last_dodge > 0 && tick < self.last_dodge + t(d::CHARGES_DODGE_GAP_TICKS)) {
            return false;
        }
        self.spend_dodge(tick);
        self.last_dodge = tick.max(1);
        self.pending_dodge = Some(hit);
        true
    }

    fn spend_dodge(&mut self, tick: u64) {
        self.spend_shadows(tick, 1);
    }

    /// contender-3.6: dodges and Shadow Flurry share one pool of shadows (a dodge costs 1, a Flurry 2)
    fn spend_shadows(&mut self, tick: u64, n: i64) {
        if self.charges == d::CHARGES_MAX {
            self.charge_at = tick + t(d::CHARGES_RECHARGE_TICKS);
        }
        self.charges -= n;
    }

    pub fn step(&mut self, f: &Frame) -> Vec<Effect> {
        let mut out = Vec::new();
        self.rejection.clear();
        if !f.alive {
            self.respawn();
            return out;
        }
        for &hit in &f.incoming {
            self.try_dodge(f.tick, hit);
        }
        self.recharge(f.tick);
        self.refund_shadows(f.tick, f.landed_hits);
        self.refund_shadows(f.tick, f.landed_merge_hits);
        self.stacks.retain(|s| f.tick < s.until);
        self.slip(f, &mut out);
        for &id in &f.basic_hits {
            self.refund_shadows(f.tick, 1);
            if let Some(u) = f.unit(id).filter(|u| !u.tower && u.team != f.team) {
                self.basic_landed(f, u, &mut out);
            }
        }
        self.run_echoes(f, &mut out);
        self.run_merge(f, &mut out);
        self.advance_shadow(f, &mut out);
        let was_flurrying = self.flurrying();
        self.run_flurry(f, &mut out);
        if !was_flurrying {
            let order = if f.manual { f.order } else { self.decide(f, &mut out) };
            if let Some(order) = order {
                if f.stunned {
                    self.reject("controlled");
                } else {
                    self.execute(f, order, &mut out);
                }
            }
        }
        self.rhythm(f, &mut out);
        if self.charges_shown != self.charges {
            self.charges_shown = self.charges;
            out.push(Effect::Charges { count: self.charges });
        }
        out
    }

    fn recharge(&mut self, tick: u64) {
        while self.charges < d::CHARGES_MAX && tick >= self.charge_at {
            self.charges += 1;
            self.charge_at += t(d::CHARGES_RECHARGE_TICKS);
        }
    }

    /// contender-3.6 (Reihan): each landed hit shortens the next shadow, more the fewer he has left
    /// (`CHARGES_HIT_REFUND_TICKS_PER_MISSING` per missing shadow).
    fn refund_shadows(&mut self, tick: u64, hits: u16) {
        for _ in 0..hits {
            if self.charges >= d::CHARGES_MAX {
                break;
            }
            let missing = (d::CHARGES_MAX - self.charges).max(0);
            let cut = t(missing * d::CHARGES_HIT_REFUND_TICKS_PER_MISSING).min(self.charge_at.saturating_sub(tick));
            self.charge_at -= cut;
            self.recharge(tick);
        }
    }

    fn reject(&mut self, why: &str) {
        self.rejection = why.into();
    }

    fn slip(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        let Some(hit) = self.pending_dodge.take() else { return };
        let away = unit_dir(hit.from, f.pos);
        let side = if (f.tick / 7) % 2 == 0 { (-away.1, away.0) } else { (away.1, -away.0) };
        let dir = ((away.0 + side.0) * 0.7071, (away.1 + side.1) * 0.7071);
        let to = f.path_until(f.pos, f.pos.along(dir, d::CHARGES_DODGE_SLIP).clamp_map(), false);
        out.push(Effect::Glide { to, ticks: t(d::CHARGES_DODGE_SLIP_TICKS) });
        out.push(Effect::Fx { name: "ct_swap".into(), at: f.pos, ticks: 18, heading: None });
        let reason = match f.unit(hit.attacker) {
            _ if hit.counter => "a Counterpunch cannot trigger another counter",
            Some(u) if !u.champion || u.team == f.team || u.hp <= 0 || !u.targetable
                || f.separated.contains(&u.id) => "attacker ineligible",
            Some(u) if !u.pos.within(f.pos, d::COUNTER_RANGE) => "attacker out of range",
            Some(u) if f.path_until(f.pos, u.pos, false) != u.pos => "path blocked",
            Some(u) => {
                let amount = self.scaled_hit(f, u, pct(d::COUNTER_BASE, d::COUNTER_AD_RATIO, f.attack))
                    + self.take_break(f, u, out);
                out.push(Effect::Damage { target: u.id, amount, source: HitSource::Counter });
                out.push(Effect::Fx { name: "ct_cross".into(), at: u.pos, ticks: 18, heading: None });
                out.push(Effect::Log { text: format!("Counterpunch: attacker #{}", u.id) });
                return;
            }
            None => "attacker missing",
        };
        out.push(Effect::Log { text: format!("Counterpunch: attacker #{} — {reason}", hit.attacker) });
    }

    fn scaled_hit(&self, f: &Frame, u: &Unit, base: i64) -> i64 {
        base * (1_000 + self.vuln_permille(u.id, f.tick)) / 1_000
    }

    fn take_break(&mut self, f: &Frame, u: &Unit, out: &mut Vec<Effect>) -> i64 {
        let max = d::RHYTHM_MAX_STACKS;
        if self.stacks_on(u.id, f.tick) < max {
            return 0;
        }
        self.stacks.retain(|s| s.id != u.id);
        let end = f.tick + t(d::RHYTHM_BREAK_BUFF_TICKS);
        self.linger = Some((max, self.linger.filter(|l| l.0 >= max).map_or(end, |l| l.1.max(end))));
        out.push(Effect::Fx { name: "ct_break".into(), at: u.pos, ticks: 18, heading: None });
        // contender-3.6: Shadow Break knocks up, once per cooldown on each enemy
        self.break_knock_ready.retain(|&(_, ready)| ready > f.tick);
        if !u.tower && !self.break_knock_ready.iter().any(|&(id, _)| id == u.id) {
            out.push(Effect::BreakAirborne { target: u.id, ticks: t(d::RHYTHM_BREAK_KNOCK_TICKS) });
            self.break_knock_ready.push((u.id, f.tick + t(d::RHYTHM_BREAK_KNOCK_COOLDOWN_TICKS)));
        }
        u.max_hp * d::RHYTHM_BREAK_PERMILLE_MAX_HP / 1_000 + d::RHYTHM_BREAK_AD_RATIO * f.attack / 100
    }

    fn add_stack(&mut self, f: &Frame, u: &Unit) {
        let max = d::RHYTHM_MAX_STACKS;
        let until = f.tick + t(d::RHYTHM_STACK_TICKS);
        let gain = if self.merged(f.tick) { d::MERGE_STACKS_PER_HIT } else { 1 };
        let carry = u.champion && self.stacks.iter().any(|s| !s.champion && s.count >= max && f.tick < s.until);
        match self.stacks.iter_mut().find(|s| s.id == u.id) {
            Some(s) => {
                s.count = (s.count + gain).min(max);
                s.until = until;
            }
            None => self.stacks.push(Stack { id: u.id, count: if carry { d::RHYTHM_CARRY_STACKS } else { gain.min(max) }, until, champion: u.champion }),
        }
    }

    fn run_echoes(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        let mut echoes = std::mem::take(&mut self.echoes);
        echoes.retain(|e| {
            if f.tick < e.at {
                return true;
            }
            if e.amount > 0 {
                if let Some(u) = f.unit(e.target).filter(|u| f.skill_target(u)) {
                    out.push(Effect::Damage { target: u.id, amount: self.scaled_hit(f, u, e.amount), source: HitSource::Echo });
                    out.push(Effect::Fx { name: "ct_echo".into(), at: u.pos, ticks: 12, heading: None });
                }
            }
            false
        });
        self.echoes = echoes;
    }

    fn basic_landed(&mut self, f: &Frame, u: &Unit, out: &mut Vec<Effect>) {
        let haymaker = f.tick < self.haymaker_until;
        let mut bonus = 0;
        if haymaker {
            self.haymaker_until = 0;
            bonus = self.scaled_hit(f, u, pct(d::FLURRY_HAYMAKER_BASE, d::FLURRY_HAYMAKER_AD_RATIO, f.attack));
            out.push(Effect::Fx { name: "ct_haymaker".into(), at: u.pos, ticks: 18, heading: None });
        }
        if self.stacks_on(u.id, f.tick) >= d::RHYTHM_MAX_STACKS {
            bonus += self.take_break(f, u, out);
        } else {
            self.add_stack(f, u);
        }
        if bonus > 0 {
            out.push(Effect::Damage { target: u.id, amount: bonus, source: HitSource::Bonus });
        }
        let echoes = if self.merged(f.tick) { 2 } else { 1 };
        for i in 1..=echoes {
            self.echoes.push(Echo {
                target: u.id,
                at: f.tick + t(d::PACT_DELAY_TICKS) * i,
                amount: f.attack * d::PACT_PCT / 100,
            });
        }
    }

    /// Shadow Rhythm: buffs from the most-stacked enemy (or the kill's linger).
    fn rhythm(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        let alive = |id: usize| f.unit(id).is_some_and(|u| u.hp > 0);
        if let Some((id, n)) = self.focus {
            if !alive(id) && n > 0 {
                self.linger = Some((n, f.tick + t(d::RHYTHM_LINGER_TICKS)));
            }
        }
        self.focus = self.stacks.iter().filter(|s| alive(s.id)).max_by_key(|s| (s.count, s.champion, std::cmp::Reverse(s.id))).map(|s| (s.id, s.count));
        if self.linger.is_some_and(|l| f.tick >= l.1) {
            self.linger = None;
        }
        let n = self.focus.map_or(0, |x| x.1).max(self.linger.map_or(0, |l| l.0));
        if n != self.rhythm_shown.0 || (n > 0 && f.tick >= self.rhythm_shown.1) {
            self.rhythm_shown = (n, f.tick + 30);
            out.push(Effect::Rhythm {
                attack_speed: n * d::RHYTHM_ATTACK_SPEED_PERMILLE_PER_STACK / 10,
                attack_permille: n * d::RHYTHM_ATTACK_PERMILLE_PER_STACK,
                move_permille: n * d::RHYTHM_MOVE_SPEED_PERMILLE_PER_STACK,
                ticks: if n > 0 { 40 } else { 1 },
            });
        }
    }

    /// enemies (never towers) near `p`, champions first then nearest
    fn targets_near(f: &Frame, p: Point, r: i64) -> Vec<Unit> {
        let mut v: Vec<Unit> = f.units.iter().filter(|u| f.skill_target(u) && !f.separated.contains(&u.id)
            && u.pos.within(p, r + u.radius) && f.path_until(p, u.pos, false) == u.pos).cloned().collect();
        v.sort_by_key(|u| (!u.champion, u.pos.d2(p), u.id));
        v
    }

    // -------------------------------------------------------------- orders

    pub fn execute(&mut self, f: &Frame, order: Order, out: &mut Vec<Effect>) {
        match order {
            Order::S1 if f.tick >= self.merge_at => {
                self.merge_at = f.tick + t(d::MERGE_COOLDOWN_TICKS);
                self.merge_until = f.tick + t(d::MERGE_TICKS);
                self.merge_cross_at = f.tick + t(d::MERGE_CROSS_EVERY_TICKS);
                out.push(Effect::Merge { ticks: t(d::MERGE_TICKS), move_pct: d::MERGE_MOVE_SPEED_PCT, attack_speed_pct: d::MERGE_ATTACK_SPEED_PCT, attack_pct: d::MERGE_ATTACK_PCT, cc_immune: d::MERGE_CC_IMMUNE != 0 });
                out.push(Effect::Fx { name: "ct_launch".into(), at: f.pos, ticks: 18, heading: None });
            }
            Order::S1 => self.reject("s1: Shadow Merge cooling down"),
            Order::S2 if !crate::unlocks::skill_unlocked(f.level, 1) => self.reject("s2: locked until level 2"),
            Order::S2 if !self.flurry_ready(f) => self.reject(if self.charges < d::CHARGES_FLURRY_COST { "s2: not enough shadows" } else { "s2: locked or cast gap active" }),
            Order::S2 => self.start_flurry(f, out),
            Order::Ult { aim } if self.ult_ready(f) => self.cast_grab(f, aim),
            Order::Ult { .. } => self.reject("ult: locked or Shadow Grab cooling down"),
        }
    }

    fn run_merge(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        if self.merge_cross_at != 0 && f.tick >= self.merge_cross_at
            && f.tick <= self.merge_until {
            self.merge_cross_at += t(d::MERGE_CROSS_EVERY_TICKS);
            out.push(Effect::Fx { name: "ct_cross".into(), at: f.pos, ticks: 18, heading: None });
            for u in f.units.iter().filter(|u| f.skill_target(u) && !f.separated.contains(&u.id)
                && u.pos.within(f.pos, d::MERGE_CROSS_RADIUS) && f.path_until(f.pos, u.pos, false) == u.pos) {
                let amount = self.scaled_hit(f, u, pct(d::MERGE_CROSS_BASE, d::MERGE_CROSS_AD_RATIO, f.attack));
                out.push(Effect::Damage { target: u.id, amount, source: HitSource::Merge });
            }
        }
        if self.merge_until > 0 && f.tick >= self.merge_until {
            self.merge_until = 0;
            self.merge_cross_at = 0;
        }
    }

    /// contender-3.6: Shadow Grab. The shadow flies in a line toward the aim, up to its range.
    fn cast_grab(&mut self, f: &Frame, aim: Point) {
        let cast = f.tick;
        let destination = f.pos.toward(aim.clamp_map(), d::SHADOW_GRAB_RANGE);
        let at = f.path_until(f.pos, destination, false);
        if at == f.pos && destination != f.pos {
            self.reject("ult: path blocked");
            return;
        }
        let distance = f.pos.dist(at).ceil() as i64;
        let ticks = (distance * d::SHADOW_GRAB_FLIGHT_TICKS + d::SHADOW_GRAB_RANGE - 1) / d::SHADOW_GRAB_RANGE;
        self.ult_at = cast + t(d::SHADOW_GRAB_COOLDOWN_TICKS);
        self.shadow_link = Some(ShadowLink::Flight { cast, from: f.pos, at, cursor: f.pos, landing: cast + t(ticks).max(1) });
    }

    /// The first enemy the shadow touches between `from` and `to` (closest to `from`).
    fn grab_victim<'a>(f: &'a Frame, from: Point, to: Point) -> Option<&'a Unit> {
        let lane = Barrier { from, to, half_width: d::SHADOW_GRAB_WIDTH };
        f.units.iter()
            .filter(|u| f.link_target(u) && u.hp > 0 && lane.contains(u.pos))
            .min_by_key(|u| (u.pos.d2(from), u.id))
    }

    fn advance_shadow(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        let Some(phase) = self.shadow_link.take() else { return };
        match phase {
            ShadowLink::Flight { cast, from, at, cursor, landing } => {
                let elapsed = f.tick.saturating_sub(cast).min(landing - cast) as i64;
                let total = (landing - cast) as i64;
                let step = Point::new(from.x + (at.x - from.x) * elapsed / total, from.y + (at.y - from.y) * elapsed / total);
                let next = f.path_until(cursor, step, false);
                if let Some(u) = Self::grab_victim(f, cursor, next) {
                    let amount = self.scaled_hit(f, u, pct(d::SHADOW_GRAB_BASE, d::SHADOW_GRAB_AD_RATIO, f.attack));
                    out.push(Effect::Damage { target: u.id, amount, source: HitSource::Grab });
                    out.push(Effect::Fx { name: "ct_grab_latch".into(), at: u.pos, ticks: GRAB_LATCH_TICKS, heading: None });
                    out.push(Effect::Log { text: format!("Shadow Grab: #{}", u.id) });
                    self.add_stack(f, u);
                    let to = f.path_until(u.pos, f.pos.toward(u.pos, d::SHADOW_GRAB_PULL_STOP), false);
                    let pulled = vec![LinkPull { target: u.id, from: u.pos, to }];
                    self.shadow_link = Some(ShadowLink::Pull { at: f.pos, until: f.tick + t(d::SHADOW_GRAB_PULL_TICKS), pulled });
                    return;
                }
                if next == step && f.tick < landing {
                    self.shadow_link = Some(ShadowLink::Flight { cast, from, at, cursor: next, landing });
                }
            }
            ShadowLink::Pull { at, until, mut pulled } => {
                pulled.retain(|p| f.unit(p.target).is_some_and(|u| f.link_target(u)));
                if f.tick <= until && !pulled.is_empty() {
                    let total = d::SHADOW_GRAB_PULL_TICKS;
                    let elapsed = total - (until - f.tick) as i64 + 1;
                    if elapsed <= total {
                        for pull in &pulled {
                            if let Some(u) = f.unit(pull.target) {
                                let step = Point::new(pull.from.x + (pull.to.x - pull.from.x) * elapsed / total,
                                    pull.from.y + (pull.to.y - pull.from.y) * elapsed / total);
                                let to = f.path_until(u.pos, step, false);
                                if to != u.pos && to.d2(at) < u.pos.d2(at) {
                                    out.push(Effect::Knock { target: u.id, to, ticks: 1 });
                                }
                            }
                        }
                        self.shadow_link = Some(ShadowLink::Pull { at, until, pulled });
                        return;
                    }
                }
                let ticks = t(d::SHADOW_GRAB_KNOCK_TICKS);
                let targets: Vec<_> = pulled.iter().map(|p| p.target).collect();
                for &target in &targets {
                    out.push(Effect::LinkAirborne { target, ticks });
                    if let Some(u) = f.unit(target) {
                        out.push(Effect::Fx { name: "ct_grab_burst".into(), at: u.pos, ticks: GRAB_BURST_TICKS, heading: None });
                    }
                }
                if !targets.is_empty() {
                    self.shadow_link = Some(ShadowLink::Airborne { until: f.tick + ticks, targets });
                }
            }
            ShadowLink::Airborne { until, targets } => {
                if f.tick < until {
                    self.shadow_link = Some(ShadowLink::Airborne { until, targets });
                }
            }
        }
    }

    fn start_flurry(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        let Some(first) = Self::targets_near(f, f.pos, d::FLURRY_START_RANGE).first().cloned() else {
            return self.reject("s2: nothing in reach");
        };
        if self.charges < d::CHARGES_FLURRY_COST {
            return self.reject("s2: not enough shadows");
        }
        self.spend_shadows(f.tick, d::CHARGES_FLURRY_COST);
        self.flurry = Some(Flurry {
            start: f.tick, next: f.tick + t(d::FLURRY_GAP_TICKS), done: 0,
            first: first.id, last: None, vanished_at: f.pos,
        });
        out.push(Effect::Vanish { ticks: 2 });
        out.push(Effect::Fx { name: "ct_vanish".into(), at: f.pos, ticks: 24, heading: None });
    }

    pub fn flurry_heading(owner: usize, punch: i64) -> u8 {
        ((owner % 8 + 3 * punch.rem_euclid(8) as usize) % 8) as u8
    }

    fn run_flurry(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        let Some(mut fl) = self.flurry.take() else { return };
        let end = fl.start + t(d::FLURRY_TICKS);
        if f.tick >= fl.next && fl.next <= end && fl.done < d::FLURRY_PUNCHES {
            let candidates = fl.last.map_or_else(
                || Self::targets_near(f, fl.vanished_at, d::FLURRY_START_RANGE),
                |(_, at)| Self::targets_near(f, at, d::FLURRY_CHAIN_RANGE),
            );
            let victim = match fl.last {
                None => candidates.iter().find(|u| u.id == fl.first),
                Some((previous, _)) => candidates.iter().find(|u| u.id != previous).or_else(|| candidates.first()),
            };
            if let Some(u) = victim {
                let amount = self.scaled_hit(f, u, pct(d::FLURRY_BASE, d::FLURRY_AD_RATIO, f.attack));
                out.push(Effect::Damage { target: u.id, amount, source: HitSource::Flurry });
                self.add_stack(f, u);
                out.push(Effect::Fx { name: "ct_flurry".into(), at: u.pos, ticks: 8, heading: Some(Self::flurry_heading(f.owner, fl.done)) });
                fl.last = Some((u.id, u.pos));
                fl.next += t(d::FLURRY_GAP_TICKS);
                fl.done += 1;
            } else {
                fl.next = 0;
            }
        }
        if f.tick >= end || fl.next == 0 {
            out.push(Effect::Fx { name: "ct_appear".into(), at: f.pos, ticks: 18, heading: None });
            if fl.last.is_some() {
                self.haymaker_until = f.tick + t(d::FLURRY_HAYMAKER_TICKS);
            }
            self.flurry_cast_at = f.tick + t(d::FLURRY_CAST_GAP_TICKS);
        } else {
            out.push(Effect::Vanish { ticks: (end - f.tick).min(2) });
            self.flurry = Some(fl);
        }
    }

    // -------------------------------------------------------------- AI

    fn hurt_pct(u: &Unit) -> i64 {
        u.hp * 100 / u.max_hp.max(1)
    }
    fn nearby_threats(f: &Frame, at: Point) -> usize {
        f.units.iter().filter(|unit| unit.champion && unit.team != f.team && unit.hp > 0 && unit.visible
            && !f.separated.contains(&unit.id) && unit.pos.within(at, d::AI_THREAT_RANGE)).count()
    }

    fn support(f: &Frame, at: Point) -> usize {
        f.allies().filter(|ally| Self::hurt_pct(ally) >= d::AI_SUPPORT_HP
            && !f.separated.contains(&ally.id) && ally.pos.within(at, d::AI_SUPPORT_RANGE)
            && f.path_until(ally.pos, at, true) == at).count()
    }

    fn cover(f: &Frame, at: Point) -> Option<Point> {
        f.allies().filter(|ally| Self::hurt_pct(ally) >= d::AI_SUPPORT_HP && !f.separated.contains(&ally.id))
            .map(|ally| ally.pos).chain(std::iter::once(f.home))
            .map(|goal| f.path_until(at, at.toward(goal, 24_000), true))
            .filter(|point| at.dist(*point) >= 8_000.0 && f.enemy_tower_at(*point).is_none())
            .min_by_key(|point| (Self::nearby_threats(f, *point), point.d2(f.home)))
    }

    fn solo_finish(&self, f: &Frame, target: &Unit) -> bool {
        let Some(defence) = f.defences.iter().find(|defence| defence.id == target.id) else { return false };
        let hit = f.attack * 100 / (100 + defence.armour.max(0));
        target.pos.within(f.pos, d::BASIC_RANGE + target.radius)
            && f.path_until(f.pos, target.pos, true) == target.pos
            && Self::nearby_threats(f, target.pos) == 1
            && hit >= target.hp.saturating_add(defence.shield.max(0))
    }

    fn safe_commit(f: &Frame, at: Point) -> bool {
        let allies = Self::support(f, at);
        f.hp * 100 >= (d::AI_COMMIT_HP - f.p(6) / d::AI_AGGRESSIVE_HP_DIVISOR) * f.max_hp.max(1)
            && allies > 0 && allies + 1 >= Self::nearby_threats(f, at)
            && f.enemy_tower_at(at).is_none() && f.path_until(f.pos, at, true) == at
            && Self::cover(f, at).is_some()
    }

    /// Retain the ordering of healthy legal combat targets.
    pub fn pick(&self, f: &Frame) -> Option<Unit> {
        f.enemy_champions().filter(|unit| unit.pos.within(f.pos, d::AI_AWARENESS_RANGE))
            .min_by_key(|unit| (Self::hurt_pct(unit) + (unit.pos.dist(f.pos) as i64 / 2_000), unit.id))
            .cloned()
    }

    fn steer(&mut self, f: &Frame, to: Point, out: &mut Vec<Effect>) {
        if f.tick < self.steer_at {
            return;
        }
        self.steer_at = f.tick + 6;
        out.push(Effect::Steer { to });
    }

    fn hold_forward(&self, f: &Frame, threat: Point, out: &mut Vec<Effect>) {
        out.push(Effect::HoldForward { threat, cover: Self::cover(f, f.pos).unwrap_or(f.pos) });
    }

    /// Lab AI only: grab the nearest enemy champion in reach whose line is clear. `true` when it peels a hurt ally.
    fn shadow_aim(&self, f: &Frame) -> Option<(Point, bool)> {
        if !self.ult_ready(f) {
            return None;
        }
        f.enemy_champions()
            .filter(|enemy| f.link_target(enemy) && enemy.pos.within(f.pos, d::SHADOW_GRAB_RANGE)
                && f.path_until(f.pos, enemy.pos, false) == enemy.pos
                && Self::grab_victim(f, f.pos, enemy.pos).is_some_and(|u| u.id == enemy.id))
            .map(|enemy| (enemy.pos, f.allies().any(|ally| Self::hurt_pct(ally) < 45 && enemy.pos.within(ally.pos, 30_000))))
            .min_by_key(|(at, peel)| (!*peel, at.d2(f.pos)))
    }

    fn safe_flurry(&self, f: &Frame) -> bool {
        let Some(first) = Self::targets_near(f, f.pos, d::FLURRY_START_RANGE).into_iter().next() else { return false };
        if !first.champion || self.solo_finish(f, &first) {
            return false;
        }
        // Follow the currently visible chain component conservatively. Any member may be the last
        // strike after deaths or movement. Check each possible landing before committing.
        let mut candidates = vec![first.clone()];
        let mut checked = 0;
        while checked < candidates.len() {
            let unit = candidates[checked].clone();
            checked += 1;
            let land = f.path_until(f.pos, unit.pos.toward(f.pos, 10_000), false);
            if !Self::safe_commit(f, land) {
                return false;
            }
            for next in Self::targets_near(f, unit.pos, d::FLURRY_CHAIN_RANGE) {
                if !candidates.iter().any(|candidate| candidate.id == next.id) {
                    candidates.push(next);
                }
            }
        }
        candidates.iter().filter(|unit| unit.champion).count() >= 2 || Self::hurt_pct(&first) <= 35
    }

    pub fn decide(&mut self, f: &Frame, out: &mut Vec<Effect>) -> Option<Order> {
        if f.stunned || self.flurrying() {
            return None;
        }
        let target = self.pick(f);
        if let Some((aim, peel)) = self.shadow_aim(f) {
            self.reason = if peel { "Peel: Shadow Grab" } else { "Commit: Shadow Grab" }.into();
            if let Some(foe) = &target {
                self.hold_forward(f, foe.pos, out);
            }
            return Some(Order::Ult { aim });
        }
        if f.hp * 100 < d::AI_RETREAT_HP * f.max_hp.max(1) {
            if !self.reason.starts_with("Retreat") {
                self.steer_at = 0;
            }
            self.reason = "Retreat: low HP".into();
            if let Some(foe) = &target {
                self.hold_forward(f, foe.pos, out);
            }
            if let Some(cover) = Self::cover(f, f.pos) {
                self.steer(f, cover, out);
            }
            return (target.is_some() && f.tick >= self.merge_at && !self.merged(f.tick)).then_some(Order::S1);
        }
        if let Some(foe) = target {
            if self.solo_finish(f, &foe) && Self::cover(f, f.pos).is_some() {
                self.reason = "Commit: basic can finish".into();
                return None;
            }
            if self.flurry_ready(f) && self.safe_flurry(f) {
                self.reason = "Commit: supported Shadow Flurry".into();
                return Some(Order::S2);
            }
            let dist = f.pos.dist(foe.pos) as i64;
            let limit = d::AI_CHASE_RANGE + f.p(6) * d::AI_AGGRESSIVE_CHASE_PER_POINT;
            if dist <= limit && dist > d::BASIC_RANGE && Self::safe_commit(f, foe.pos) {
                self.reason = "Commit: supported approach".into();
                self.steer(f, foe.pos, out);
                if f.tick >= self.merge_at && dist <= d::AI_MERGE_RANGE {
                    return Some(Order::S1);
                }
            } else {
                self.reason = "Hold: unsafe or unsupported approach".into();
                self.hold_forward(f, foe.pos, out);
            }
            if f.tick >= self.merge_at && dist <= d::MERGE_CROSS_RADIUS {
                return Some(Order::S1);
            }
            return None;
        }
        self.reason.clear();
        if f.tick >= self.merge_at && f.units.iter().filter(|unit| unit.farmable && f.skill_target(unit)
            && unit.pos.within(f.pos, d::MERGE_CROSS_RADIUS)).count() >= 2 {
            self.reason = "Farm: Shadow Merge".into();
            return Some(Order::S1);
        }
        None
    }
}
