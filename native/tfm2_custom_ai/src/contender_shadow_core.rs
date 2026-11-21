//! Contender 4.0, the Shadow Boxer rebuilt (round 192): the shadow pool and dodges, Shadow Rhythm, Shadowborne,
//! S1 Shadow Grab, S2 Shadow Anchor, S3 Smother (self-cast) and the Clone Army ult. The anchor and the clones mimic
//! his grab and Smother. Frame in, effects out; the game bridge (`contender.rs`) and the Lab share it.
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
    /// Shadow Grab toward `aim`
    S1 { #[serde(default)] aim: Point },
    /// Shadow Anchor: drop at `aim`, or teleport to the anchor when it exists
    S2 { #[serde(default)] aim: Point },
    /// Smother toward `aim` (normally self-cast by the core)
    S3 { #[serde(default)] aim: Point },
    /// Clone Army toward `aim`
    Ult { #[serde(default)] aim: Point },
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
    Counter,
    /// the grab's hook and the Haymaker
    Grab,
    Smother,
    /// a Shadowborne flurry punch (not credited as a landed hit)
    ShadowFlurry,
}

impl HitSource {
    /// hits that shorten the next shadow (`landed_hits`)
    pub fn credits_flurry(self) -> bool {
        matches!(self, Self::Counter | Self::Grab | Self::Smother)
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
    /// Champion level (0: unknown); S2 at 2, S3 at 3 and the ultimate at 5 (`unlocks.rs`)
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
    /// Units that are CC-immune or otherwise not to be grabbed or knocked up.
    #[serde(default)]
    pub link_excluded: Vec<usize>,
    /// Units which cannot interact across the current cage or realm.
    #[serde(default)]
    pub separated: Vec<usize>,
    /// Additional wall segments (player-built walls and cage edges).
    #[serde(default)]
    pub barriers: Vec<Barrier>,
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
    pub fn link_target(&self, unit: &Unit) -> bool {
        self.enemy(unit) && !unit.tower && !self.link_excluded.contains(&unit.id)
            && !self.separated.contains(&unit.id)
    }

    pub fn path_until(&self, from: Point, to: Point, avoid_towers: bool) -> Point {
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

/// Round 190: how long Codex's Shadow Grab latch and burst play (their fades are baked by
/// tools/build-contender-grab-art.py; a test checks these match the packed `.fanim`).
pub const GRAB_LATCH_TICKS: u64 = 8;
pub const GRAB_BURST_TICKS: u64 = 18;
/// his dash after the pull, 3/8 of the grab's range
pub const GRAB_DASH: i64 = d::SHADOW_GRAB_RANGE * d::SHADOW_GRAB_DASH_PERMILLE / 1_000;
/// where the pull leaves the target: one dash plus the Haymaker's reach away from the caster
pub const GRAB_MEET: i64 = GRAB_DASH + d::SHADOW_GRAB_HAYMAKER_REACH;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Effect {
    Damage { target: usize, amount: i64, source: HitSource },
    /// a knock-up (the Haymaker and Shadowborne's finisher; `ct_air_{owner}`)
    Airborne { target: usize, ticks: u64 },
    /// the grab drags its target straight through walls (placed each tick, not a force-move)
    Drag { target: usize, to: Point },
    /// the grabbed target is held while the caster dashes in
    Hold { target: usize, ticks: u64 },
    /// a deliberate move of his own (an engine glide): dodge slips and the grab's dash
    Glide { to: Point, ticks: u64 },
    /// set him there at once (Shadowborne's teleport, the anchor teleport)
    Blink { to: Point },
    Fear { target: usize, ticks: u64 },
    Silence { target: usize, ticks: u64 },
    Slow { target: usize, pct: i64, ticks: u64 },
    /// Smother's debuff on an enemy: Armour and attack speed down (percent)
    Smothered { target: usize, armour_pct: i64, attack_speed_pct: i64, ticks: u64 },
    /// Smother's haste on him
    Haste { attack_speed_pct: i64, move_pct: i64, ticks: u64 },
    /// a shield on him
    Shield { amount: i64, ticks: u64 },
    /// his shadows (for the pips)
    Charges { count: i64 },
    Fx {
        name: String,
        at: Point,
        ticks: u64,
        /// Optional presentation heading: clockwise eighth-turns from east.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        heading: Option<u8>,
    },
    Log { text: String },
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Stack {
    pub id: usize,
    pub count: i64,
    pub until: u64,
    pub champion: bool,
}

/// A shadow that stands somewhere: the S2 anchor or a Clone Army clone. Both mimic his grab and Smother.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Shadow {
    pub id: u32,
    pub pos: Point,
    pub until: u64,
    pub anchor: bool,
}

/// Who throws a cast: Contender himself, or one of his shadows (by id).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "by", rename_all = "snake_case")]
pub enum Caster {
    Me,
    Shadow { id: u32 },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "phase", rename_all = "snake_case")]
pub enum Grab {
    Flight { from: Point, at: Point, cursor: Point, cast: u64, landing: u64 },
    Pull { target: usize, from: Point, to: Point, until: u64 },
    Dash { target: usize, to: Point, until: u64 },
    Airborne { target: usize, until: u64 },
}

/// One running cast. `pct` scales its damage (100, or the mimic share for a shadow's copy).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Cast {
    Grab { caster: Caster, pct: i64, grab: Grab },
    Smother { caster: Caster, pct: i64, from: Point, to: Point, cursor: Point, cast: u64, landing: u64, hit: Vec<usize> },
    /// a Shadowborne flurry from `at` (starts at `start`, reaching `reach` for its first target)
    Flurry { caster: Caster, pct: i64, at: Point, reach: i64, start: u64, next: u64, done: i64, last: Option<(usize, Point)> },
}

impl Cast {
    pub fn caster(&self) -> Caster {
        match self {
            Cast::Grab { caster, .. } | Cast::Smother { caster, .. } | Cast::Flurry { caster, .. } => *caster,
        }
    }
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
    pub fn contains(self, point: Point) -> bool {
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
    pub stacks: Vec<Stack>,
    pub grab_at: u64,
    pub anchor_at: u64,
    pub smother_at: u64,
    pub ult_at: u64,
    pub shadows: Vec<Shadow>,
    pub casts: Vec<Cast>,
    next_shadow: u32,
    /// shield already granted by the current Smother (his cast and its copies share the cap)
    smother_shield: i64,
    pub charges_shown: i64,
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
            stacks: vec![],
            grab_at: 0,
            anchor_at: 0,
            smother_at: 0,
            ult_at: 0,
            shadows: vec![],
            casts: vec![],
            next_shadow: 1,
            smother_shield: 0,
            charges_shown: -1,
            reason: String::new(),
            rejection: String::new(),
        }
    }
}

impl Core {
    pub fn respawn(&mut self) {
        self.pending_dodge = None;
        self.stacks.clear();
        self.shadows.clear();
        self.casts.clear();
    }

    pub fn anchor(&self) -> Option<&Shadow> {
        self.shadows.iter().find(|s| s.anchor)
    }

    pub fn grabbing(&self) -> bool {
        self.casts.iter().any(|c| matches!(c, Cast::Grab { caster: Caster::Me, .. }))
    }

    pub fn grab_ready(&self, f: &Frame) -> bool {
        crate::unlocks::skill_unlocked(f.level, 0) && f.tick >= self.grab_at && !self.grabbing()
    }

    /// S2 is wanted when the anchor can be dropped, or when one exists to teleport to.
    pub fn anchor_ready(&self, f: &Frame) -> bool {
        crate::unlocks::skill_unlocked(f.level, 1) && (self.anchor().is_some() || f.tick >= self.anchor_at)
    }

    pub fn smother_ready(&self, f: &Frame) -> bool {
        crate::unlocks::skill_unlocked(f.level, 2) && f.tick >= self.smother_at
    }

    pub fn ult_ready(&self, f: &Frame) -> bool {
        crate::unlocks::ult_unlocked(f.level, f.ult_unlocked) && f.tick >= self.ult_at
    }

    pub fn stacks_on(&self, id: usize, tick: u64) -> i64 {
        self.stacks.iter().find(|s| s.id == id && tick < s.until).map_or(0, |s| s.count)
    }

    pub fn vuln_permille(&self, id: usize, tick: u64) -> i64 {
        self.stacks_on(id, tick) * d::RHYTHM_VULNERABILITY_PERMILLE_PER_STACK
    }

    fn pos_of(&self, f: &Frame, c: Caster) -> Option<Point> {
        match c {
            Caster::Me => Some(f.pos),
            Caster::Shadow { id } => self.shadows.iter().find(|s| s.id == id).map(|s| s.pos),
        }
    }

    // -------------------------------------------------------------- shadows (the passive pool)

    pub fn try_dodge(&mut self, tick: u64, hit: IncomingHit) -> bool {
        if self.charges <= 0 || (self.last_dodge > 0 && tick < self.last_dodge + t(d::CHARGES_DODGE_GAP_TICKS)) {
            return false;
        }
        self.spend_shadows(tick, 1);
        self.last_dodge = tick.max(1);
        self.pending_dodge = Some(hit);
        true
    }

    fn spend_shadows(&mut self, tick: u64, n: i64) {
        if self.charges == d::CHARGES_MAX {
            self.charge_at = tick + t(d::CHARGES_RECHARGE_TICKS);
        }
        self.charges -= n;
    }

    fn recharge(&mut self, tick: u64) {
        while self.charges < d::CHARGES_MAX && tick >= self.charge_at {
            self.charges += 1;
            self.charge_at += t(d::CHARGES_RECHARGE_TICKS);
        }
    }

    /// each landed hit shortens the next shadow, more the fewer he has left
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
                let amount = self.scaled_hit(f, u, pct(d::COUNTER_BASE, d::COUNTER_AD_RATIO, f.attack));
                out.push(Effect::Damage { target: u.id, amount, source: HitSource::Counter });
                out.push(Effect::Fx { name: "ct_cross".into(), at: u.pos, ticks: 18, heading: None });
                out.push(Effect::Log { text: format!("Counterpunch: attacker #{}", u.id) });
                self.add_stack(f, u);
                return;
            }
            None => "attacker missing",
        };
        out.push(Effect::Log { text: format!("Counterpunch: attacker #{} — {reason}", hit.attacker) });
    }

    // -------------------------------------------------------------- Shadow Rhythm

    fn scaled_hit(&self, f: &Frame, u: &Unit, base: i64) -> i64 {
        base * (1_000 + self.vuln_permille(u.id, f.tick)) / 1_000
    }

    fn add_stack(&mut self, f: &Frame, u: &Unit) {
        let until = f.tick + t(d::RHYTHM_STACK_TICKS);
        match self.stacks.iter_mut().find(|s| s.id == u.id) {
            Some(s) => {
                s.count = (s.count + 1).min(d::RHYTHM_MAX_STACKS);
                s.until = until;
            }
            None => self.stacks.push(Stack { id: u.id, count: 1, until, champion: u.champion }),
        }
    }

    /// a hit from one of his casts: Rhythm-scaled, shared by the cast's damage share, and it builds a stack
    fn hit(&mut self, f: &Frame, u: &Unit, base: i64, share: i64, source: HitSource, out: &mut Vec<Effect>) {
        let amount = self.scaled_hit(f, u, base) * share / 100;
        out.push(Effect::Damage { target: u.id, amount, source });
        self.add_stack(f, u);
    }

    // -------------------------------------------------------------- the step

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
        self.stacks.retain(|s| f.tick < s.until);
        self.slip(f, &mut out);
        for &id in &f.basic_hits {
            self.refund_shadows(f.tick, 1);
            if let Some(u) = f.unit(id).filter(|u| !u.tower && u.team != f.team) {
                self.add_stack(f, u);
            }
        }
        self.expire_shadows(f);
        let order = if f.manual { f.order } else { self.decide(f) };
        if let Some(order) = order {
            if f.stunned {
                self.reject("controlled");
            } else {
                self.execute(f, order, &mut out);
            }
        }
        // S3 has no engine button: his passive casts Smother at the nearest enemy champion in reach
        if !f.stunned && self.smother_ready(f) && !matches!(order, Some(Order::S3 { .. })) {
            if let Some(foe) = f.enemy_champions().filter(|u| !f.separated.contains(&u.id) && u.pos.within(f.pos, d::SMOTHER_RANGE))
                .min_by_key(|u| (u.pos.d2(f.pos), u.id)) {
                let aim = foe.pos;
                self.cast_smother(f, aim, &mut out);
            }
        }
        // the anchor saves him when he drops low
        if self.anchor().is_some() && f.hp * 100 <= d::ANCHOR_LOW_HP_PCT * f.max_hp.max(1) {
            self.teleport_to_anchor(f, &mut out);
        }
        self.run_casts(f, &mut out);
        if self.charges_shown != self.charges {
            self.charges_shown = self.charges;
            out.push(Effect::Charges { count: self.charges });
        }
        out
    }

    fn expire_shadows(&mut self, f: &Frame) {
        let before = self.anchor().is_some();
        self.shadows.retain(|s| f.tick < s.until);
        if before && self.anchor().is_none() {
            self.anchor_at = f.tick + t(d::ANCHOR_COOLDOWN_TICKS);
        }
    }

    pub fn execute(&mut self, f: &Frame, order: Order, out: &mut Vec<Effect>) {
        match order {
            Order::S1 { aim } if self.grab_ready(f) => self.cast_grab(f, aim, out),
            Order::S1 { .. } => self.reject("s1: locked or Shadow Grab cooling down"),
            Order::S2 { .. } if !crate::unlocks::skill_unlocked(f.level, 1) => self.reject(&crate::unlocks::locked_reason(1)),
            Order::S2 { .. } if self.anchor().is_some() => self.teleport_to_anchor(f, out),
            Order::S2 { aim } if f.tick >= self.anchor_at => self.drop_anchor(f, aim, out),
            Order::S2 { .. } => self.reject("s2: Shadow Anchor cooling down"),
            Order::S3 { aim } if self.smother_ready(f) => self.cast_smother(f, aim, out),
            Order::S3 { .. } => self.reject("s3: locked or Smother cooling down"),
            Order::Ult { aim } if self.ult_ready(f) => self.clone_army(f, aim, out),
            Order::Ult { .. } => self.reject("ult: locked or Clone Army cooling down"),
        }
    }

    // -------------------------------------------------------------- S1 Shadow Grab

    fn cast_grab(&mut self, f: &Frame, aim: Point, out: &mut Vec<Effect>) {
        if aim == f.pos {
            return self.reject("s1: no direction");
        }
        self.grab_at = f.tick + t(d::SHADOW_GRAB_COOLDOWN_TICKS);
        let dir = unit_dir(f.pos, aim);
        self.start_grab(f, Caster::Me, f.pos, dir, 100);
        // the anchor and the clones mimic it from where they stand
        for s in self.shadows.clone() {
            self.start_grab(f, Caster::Shadow { id: s.id }, s.pos, dir, d::CLONE_ARMY_MIMIC_DAMAGE_PCT);
        }
        out.push(Effect::Log { text: format!("Shadow Grab ({} casters)", 1 + self.shadows.len()) });
    }

    fn start_grab(&mut self, f: &Frame, caster: Caster, from: Point, dir: (f64, f64), pct: i64) {
        // the shadow flies through walls
        let at = from.along(dir, d::SHADOW_GRAB_RANGE).clamp_map();
        let distance = from.dist(at).ceil() as i64;
        let ticks = (distance * d::SHADOW_GRAB_FLIGHT_TICKS + d::SHADOW_GRAB_RANGE - 1) / d::SHADOW_GRAB_RANGE;
        let grab = Grab::Flight { from, at, cursor: from, cast: f.tick, landing: f.tick + t(ticks).max(1) };
        self.casts.push(Cast::Grab { caster, pct, grab });
    }

    /// enemies another of his grabs already holds (one grab per enemy)
    fn held(&self) -> Vec<usize> {
        self.casts.iter().filter_map(|c| match c {
            Cast::Grab { grab: Grab::Pull { target, .. } | Grab::Dash { target, .. } | Grab::Airborne { target, .. }, .. } => Some(*target),
            _ => None,
        }).collect()
    }

    /// The first enemy the shadow touches between `from` and `to` (closest to `from`).
    pub fn grab_victim<'a>(f: &'a Frame, from: Point, to: Point, held: &[usize]) -> Option<&'a Unit> {
        let lane = Barrier { from, to, half_width: d::SHADOW_GRAB_WIDTH };
        f.units.iter()
            .filter(|u| f.link_target(u) && !held.contains(&u.id) && lane.contains(u.pos))
            .min_by_key(|u| (u.pos.d2(from), u.id))
    }

    /// One grab's tick. Returns false when it is over.
    fn run_grab(&mut self, f: &Frame, caster: Caster, pct: i64, grab: &mut Grab, out: &mut Vec<Effect>) -> bool {
        let Some(origin) = self.pos_of(f, caster) else { return false };
        match grab.clone() {
            Grab::Flight { from, at, cursor, cast, landing } => {
                let elapsed = f.tick.saturating_sub(cast).min(landing - cast) as i64;
                let total = (landing - cast) as i64;
                let next = Point::new(from.x + (at.x - from.x) * elapsed / total, from.y + (at.y - from.y) * elapsed / total);
                let held = self.held();
                if let Some(u) = Self::grab_victim(f, cursor, next, &held).cloned() {
                    self.hit(f, &u, pct_of(d::SHADOW_GRAB_BASE, d::SHADOW_GRAB_AD_RATIO, f.attack), pct, HitSource::Grab, out);
                    out.push(Effect::Fx { name: "ct_grab_latch".into(), at: u.pos, ticks: GRAB_LATCH_TICKS, heading: None });
                    out.push(Effect::Log { text: format!("Shadow Grab: #{}", u.id) });
                    if u.pos.dist(origin) > GRAB_MEET as f64 {
                        // the meeting point is open ground the caster reaches; the drag itself ignores walls
                        let to = f.path_until(origin, origin.toward(u.pos, GRAB_MEET), false);
                        *grab = Grab::Pull { target: u.id, from: u.pos, to, until: f.tick + t(d::SHADOW_GRAB_PULL_TICKS) };
                    } else {
                        *grab = self.dash(f, caster, origin, &u, out);
                    }
                    return true;
                }
                if f.tick < landing {
                    *grab = Grab::Flight { from, at, cursor: next, cast, landing };
                    return true;
                }
                // a miss: the shadow throws a Shadowborne flurry where it landed
                out.push(Effect::Log { text: "Shadow Grab missed: shadow flurry".into() });
                self.casts.push(Cast::Flurry { caster, pct, at: next, reach: d::SHADOWBORNE_START_RANGE, start: f.tick, next: f.tick, done: 0, last: None });
                false
            }
            Grab::Pull { target, from, to, until } => {
                let Some(u) = f.unit(target).filter(|u| f.link_target(u)).cloned() else { return false };
                let total = d::SHADOW_GRAB_PULL_TICKS.max(1);
                let elapsed = (total - until.saturating_sub(f.tick) as i64 + 1).clamp(1, total);
                let step = Point::new(from.x + (to.x - from.x) * elapsed / total, from.y + (to.y - from.y) * elapsed / total);
                out.push(Effect::Drag { target, to: step });
                if f.tick >= until {
                    let landed = Unit { pos: step, ..u };
                    *grab = self.dash(f, caster, origin, &landed, out);
                }
                true
            }
            Grab::Dash { target, to, until } => {
                let Some(u) = f.unit(target).filter(|u| f.link_target(u)).cloned() else { return false };
                if f.tick < until {
                    return true;
                }
                // reach from where the caster is or where the dash was aimed (the engine's glide can land a tick late)
                let reach = d::SHADOW_GRAB_HAYMAKER_REACH + u.radius + d::BASIC_RANGE / 2;
                if !u.pos.within(origin, reach) && !u.pos.within(to, reach) {
                    out.push(Effect::Log { text: format!("Haymaker: #{} out of reach", u.id) });
                    return false;
                }
                self.hit(f, &u, pct_of(d::SHADOW_GRAB_HAYMAKER_BASE, d::SHADOW_GRAB_HAYMAKER_AD_RATIO, f.attack), pct, HitSource::Grab, out);
                let ticks = t(d::SHADOW_GRAB_HAYMAKER_KNOCK_TICKS);
                out.push(Effect::Airborne { target, ticks });
                out.push(Effect::Fx { name: "ct_grab_burst".into(), at: u.pos, ticks: GRAB_BURST_TICKS, heading: None });
                out.push(Effect::Log { text: format!("Haymaker: #{}", u.id) });
                *grab = Grab::Airborne { target, until: f.tick + ticks };
                true
            }
            Grab::Airborne { until, .. } => f.tick < until,
        }
    }

    /// The caster dashes at the grabbed target (3/8 of the grab's range at most, stopping at Haymaker reach and at
    /// walls) while it is held. A shadow simply steps there.
    fn dash(&mut self, f: &Frame, caster: Caster, origin: Point, u: &Unit, out: &mut Vec<Effect>) -> Grab {
        let gap = (u.pos.dist(origin).round() as i64 - d::SHADOW_GRAB_HAYMAKER_REACH).clamp(0, GRAB_DASH);
        let ticks = t(d::SHADOW_GRAB_DASH_TICKS);
        let to = if gap > 0 { f.path_until(origin, origin.toward(u.pos, gap), false) } else { origin };
        match caster {
            Caster::Me if to != origin => out.push(Effect::Glide { to, ticks }),
            Caster::Shadow { id } => {
                if let Some(s) = self.shadows.iter_mut().find(|s| s.id == id) {
                    s.pos = to;
                }
            }
            _ => {}
        }
        out.push(Effect::Hold { target: u.id, ticks });
        Grab::Dash { target: u.id, to, until: f.tick + ticks }
    }

    // -------------------------------------------------------------- S2 Shadow Anchor

    fn drop_anchor(&mut self, f: &Frame, aim: Point, out: &mut Vec<Effect>) {
        let at = f.nearest_ground(f.pos.toward(aim, d::ANCHOR_RANGE));
        let id = self.new_shadow(at, f.tick + t(d::ANCHOR_LIFE_TICKS), true);
        let ticks = t(d::ANCHOR_FEAR_TICKS);
        for u in f.units.iter().filter(|u| f.link_target(u) && u.pos.within(at, d::ANCHOR_FEAR_RADIUS)) {
            out.push(Effect::Fear { target: u.id, ticks });
        }
        out.push(Effect::Fx { name: "ct_launch".into(), at, ticks: 18, heading: None });
        // after the fear, the shadow flurries (reaching the feared, who ran up to a fear radius further)
        self.casts.push(Cast::Flurry {
            caster: Caster::Shadow { id }, pct: 100, at, reach: d::ANCHOR_FEAR_RADIUS + d::SHADOWBORNE_START_RANGE,
            start: f.tick + ticks, next: f.tick + ticks, done: 0, last: None,
        });
        out.push(Effect::Log { text: "Shadow Anchor dropped".into() });
    }

    fn teleport_to_anchor(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        let Some(a) = self.anchor().copied() else { return };
        out.push(Effect::Blink { to: a.pos });
        out.push(Effect::Fx { name: "ct_swap".into(), at: f.pos, ticks: 18, heading: None });
        self.shadows.retain(|s| !s.anchor);
        self.anchor_at = f.tick + t(d::ANCHOR_COOLDOWN_TICKS);
        out.push(Effect::Log { text: "Shadow Anchor: teleport".into() });
    }

    fn new_shadow(&mut self, pos: Point, until: u64, anchor: bool) -> u32 {
        let id = self.next_shadow;
        self.next_shadow += 1;
        self.shadows.push(Shadow { id, pos, until, anchor });
        id
    }

    // -------------------------------------------------------------- S3 Smother

    fn cast_smother(&mut self, f: &Frame, aim: Point, out: &mut Vec<Effect>) {
        if aim == f.pos {
            return;
        }
        self.smother_at = f.tick + t(d::SMOTHER_COOLDOWN_TICKS);
        self.smother_shield = 0;
        let dir = unit_dir(f.pos, aim);
        self.start_smother(f, Caster::Me, f.pos, dir, 100);
        for s in self.shadows.clone() {
            self.start_smother(f, Caster::Shadow { id: s.id }, s.pos, dir, d::CLONE_ARMY_MIMIC_DAMAGE_PCT);
        }
        let ticks = t(d::SMOTHER_HASTE_TICKS);
        out.push(Effect::Haste { attack_speed_pct: d::SMOTHER_HASTE_ATTACK_SPEED_PCT, move_pct: d::SMOTHER_HASTE_MOVE_SPEED_PCT, ticks });
        out.push(Effect::Log { text: "Smother".into() });
    }

    fn start_smother(&mut self, f: &Frame, caster: Caster, from: Point, dir: (f64, f64), pct: i64) {
        let to = from.along(dir, d::SMOTHER_RANGE).clamp_map();
        self.casts.push(Cast::Smother { caster, pct, from, to, cursor: from, cast: f.tick, landing: f.tick + t(d::SMOTHER_FLIGHT_TICKS).max(1), hit: vec![] });
    }

    /// One Smother line's tick. Returns false when it is over.
    #[allow(clippy::too_many_arguments)]
    fn run_smother(&mut self, f: &Frame, caster: Caster, pct: i64, from: Point, to: Point, cursor: &mut Point, cast: u64, landing: u64,
                   hit: &mut Vec<usize>, out: &mut Vec<Effect>) -> bool {
        let total = (landing - cast) as i64;
        let elapsed = f.tick.saturating_sub(cast).min(landing - cast) as i64;
        let next = Point::new(from.x + (to.x - from.x) * elapsed / total, from.y + (to.y - from.y) * elapsed / total);
        let lane = Barrier { from: *cursor, to: next, half_width: d::SMOTHER_WIDTH };
        let fresh: Vec<Unit> = f.units.iter().filter(|u| f.skill_target(u) && !f.separated.contains(&u.id)
            && !hit.contains(&u.id) && lane.contains(u.pos)).cloned().collect();
        for u in fresh {
            hit.push(u.id);
            self.hit(f, &u, pct_of(d::SMOTHER_BASE, d::SMOTHER_AD_RATIO, f.attack), pct, HitSource::Smother, out);
            if !f.link_excluded.contains(&u.id) {
                out.push(Effect::Silence { target: u.id, ticks: t(d::SMOTHER_SILENCE_TICKS) });
                out.push(Effect::Slow { target: u.id, pct: d::SMOTHER_SLOW_PCT, ticks: t(d::SMOTHER_SLOW_TICKS) });
            }
            out.push(Effect::Smothered { target: u.id, armour_pct: d::SMOTHER_ARMOUR_SHRED_PCT,
                attack_speed_pct: d::SMOTHER_ATTACK_SPEED_SHRED_PCT, ticks: t(d::SMOTHER_DEBUFF_TICKS) });
            out.push(Effect::Fx { name: "ct_echo".into(), at: u.pos, ticks: 12, heading: None });
            if u.champion {
                let cap = f.max_hp * d::SMOTHER_SHIELD_CAP_PERMILLE / 1_000;
                let add = (f.max_hp * d::SMOTHER_SHIELD_PERMILLE_PER_HIT / 1_000).min(cap - self.smother_shield);
                if add > 0 {
                    self.smother_shield += add;
                    out.push(Effect::Shield { amount: add, ticks: t(d::SMOTHER_HASTE_TICKS) });
                }
            }
        }
        *cursor = next;
        if f.tick < landing {
            return true;
        }
        // then it flurries from the first enemy hit
        if let Some(first) = hit.first().and_then(|&id| f.unit(id)) {
            let at = first.pos;
            self.casts.push(Cast::Flurry { caster, pct, at, reach: d::SHADOWBORNE_START_RANGE, start: f.tick, next: f.tick, done: 0, last: None });
        }
        false
    }

    // -------------------------------------------------------------- ult Clone Army

    fn clone_army(&mut self, f: &Frame, aim: Point, out: &mut Vec<Effect>) {
        self.ult_at = f.tick + t(d::CLONE_ARMY_COOLDOWN_TICKS);
        self.charges = d::CHARGES_MAX;
        let until = f.tick + t(d::CLONE_ARMY_LIFE_TICKS);
        let dir = unit_dir(f.pos, if aim == f.pos { Point::new(f.pos.x + 1, f.pos.y) } else { aim });
        let mut origins = vec![(f.pos, dir)];
        if let Some(a) = self.anchor().copied() {
            let toward = f.enemy_champions().min_by_key(|u| (u.pos.d2(a.pos), u.id)).map_or(dir, |u| unit_dir(a.pos, u.pos));
            origins.push((a.pos, toward));
        }
        for (from, dir) in origins {
            for p in Self::kunai_fan(from, dir) {
                self.new_shadow(p, until, false);
                out.push(Effect::Fx { name: "ct_launch".into(), at: p, ticks: 18, heading: None });
            }
        }
        out.push(Effect::Log { text: format!("Clone Army: {} clones", self.shadows.iter().filter(|s| !s.anchor).count()) });
    }

    /// One clone `CLONE_ARMY_AHEAD` in front, and one `CLONE_ARMY_SIDE` to each side of it.
    pub fn kunai_fan(from: Point, dir: (f64, f64)) -> [Point; 3] {
        let front = from.along(dir, d::CLONE_ARMY_AHEAD);
        let side = (-dir.1, dir.0);
        [front.clamp_map(), front.along(side, d::CLONE_ARMY_SIDE).clamp_map(), front.along(side, -d::CLONE_ARMY_SIDE).clamp_map()]
    }

    // -------------------------------------------------------------- Shadowborne flurries

    /// enemies (never towers) near `p`, champions first then nearest; shadows ignore walls
    fn targets_around(f: &Frame, p: Point, r: i64) -> Vec<Unit> {
        let mut v: Vec<Unit> = f.units.iter().filter(|u| f.skill_target(u) && !f.separated.contains(&u.id)
            && u.pos.within(p, r + u.radius)).cloned().collect();
        v.sort_by_key(|u| (!u.champion, u.pos.d2(p), u.id));
        v
    }

    pub fn flurry_heading(owner: usize, punch: i64) -> u8 {
        ((owner % 8 + 3 * punch.rem_euclid(8) as usize) % 8) as u8
    }

    /// One flurry's tick. Returns false when it is over (after the finisher).
    #[allow(clippy::too_many_arguments)]
    fn run_flurry(&mut self, f: &Frame, caster: Caster, pct: i64, at: Point, reach: i64, start: u64, next: &mut u64, done: &mut i64,
                  last: &mut Option<(usize, Point)>, out: &mut Vec<Effect>) -> bool {
        if f.tick < start {
            return true;
        }
        let end = start + t(d::SHADOWBORNE_TICKS);
        let mut over = f.tick >= end || *done >= d::SHADOWBORNE_PUNCHES;
        if !over && f.tick >= *next {
            let candidates = last.map_or_else(|| Self::targets_around(f, at, reach), |(_, p)| Self::targets_around(f, p, d::SHADOWBORNE_CHAIN_RANGE));
            let victim = match *last {
                None => candidates.first(),
                Some((previous, _)) => candidates.iter().find(|u| u.id != previous).or_else(|| candidates.first()),
            };
            match victim.cloned() {
                Some(u) => {
                    self.hit(f, &u, pct_of(d::SHADOWBORNE_BASE, d::SHADOWBORNE_AD_RATIO, f.attack), pct, HitSource::ShadowFlurry, out);
                    out.push(Effect::Fx { name: "ct_flurry".into(), at: u.pos, ticks: 8, heading: Some(Self::flurry_heading(f.owner, *done)) });
                    *last = Some((u.id, u.pos));
                    *next = f.tick + t(d::SHADOWBORNE_GAP_TICKS);
                    *done += 1;
                    over = *done >= d::SHADOWBORNE_PUNCHES;
                }
                None => over = true,
            }
        }
        if !over {
            return true;
        }
        match *last {
            Some((id, p)) => self.finisher(f, caster, id, p, out),
            None => out.push(Effect::Log { text: "Shadowborne: nobody near the shadow".into() }),
        }
        false
    }

    /// Shadowborne: the last punch knocks up every enemy around that enemy; his own flurry also teleports him behind it.
    fn finisher(&mut self, f: &Frame, caster: Caster, id: usize, seen_at: Point, out: &mut Vec<Effect>) {
        let (centre, radius) = f.unit(id).map_or((seen_at, 0), |u| (u.pos, u.radius));
        let ticks = t(d::SHADOWBORNE_KNOCK_TICKS);
        for u in f.units.iter().filter(|u| f.link_target(u) && u.pos.within(centre, d::SHADOWBORNE_KNOCK_RADIUS)) {
            out.push(Effect::Airborne { target: u.id, ticks });
        }
        out.push(Effect::Fx { name: "ct_grab_burst".into(), at: centre, ticks: GRAB_BURST_TICKS, heading: None });
        if caster == Caster::Me {
            let behind = f.nearest_ground(centre.along(unit_dir(f.pos, centre), radius + d::SHADOWBORNE_BEHIND));
            out.push(Effect::Blink { to: behind });
        }
        out.push(Effect::Log { text: format!("Shadowborne: #{id}") });
    }

    fn run_casts(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        let mut i = 0;
        // casts started during this loop (a miss's flurry, Smother's flurry) run from the next tick
        let mut count = self.casts.len();
        while i < count {
            let mut cast = self.casts[i].clone();
            let alive = match &mut cast {
                Cast::Grab { caster, pct, grab } => {
                    let (c, p) = (*caster, *pct);
                    self.run_grab(f, c, p, grab, out)
                }
                Cast::Smother { caster, pct, from, to, cursor, cast, landing, hit } => {
                    let (c, p, fr, t_, ca, la) = (*caster, *pct, *from, *to, *cast, *landing);
                    self.run_smother(f, c, p, fr, t_, cursor, ca, la, hit, out)
                }
                Cast::Flurry { caster, pct, at, reach, start, next, done, last } => {
                    let (c, p, a, r, s) = (*caster, *pct, *at, *reach, *start);
                    self.run_flurry(f, c, p, a, r, s, next, done, last, out)
                }
            };
            if alive {
                self.casts[i] = cast;
                i += 1;
            } else {
                self.casts.remove(i);
                count -= 1;
            }
        }
    }

    // -------------------------------------------------------------- Lab autoplay (not used in the game)

    /// Lab-only: use each skill on the nearest enemy champion. In the game the engine's AI presses S1, S2 and the
    /// ult (cast markers), and S3 is self-cast above.
    pub fn decide(&mut self, f: &Frame) -> Option<Order> {
        let foe = f.enemy_champions().filter(|u| !f.separated.contains(&u.id)).min_by_key(|u| (u.pos.d2(f.pos), u.id))?.clone();
        let near = f.enemy_champions().filter(|u| u.pos.within(f.pos, d::CLONE_ARMY_RANGE)).count();
        self.reason = format!("Lab: nearest champion #{}", foe.id);
        if self.ult_ready(f) && near >= 2 {
            return Some(Order::Ult { aim: foe.pos });
        }
        if self.grab_ready(f) && foe.pos.within(f.pos, d::SHADOW_GRAB_RANGE) {
            return Some(Order::S1 { aim: foe.pos });
        }
        if crate::unlocks::skill_unlocked(f.level, 1) && self.anchor().is_none() && f.tick >= self.anchor_at
            && foe.pos.within(f.pos, d::ANCHOR_RANGE) {
            return Some(Order::S2 { aim: foe.pos });
        }
        None
    }
}

fn pct_of(base: i64, ratio: i64, attack: i64) -> i64 {
    pct(base, ratio, attack)
}
