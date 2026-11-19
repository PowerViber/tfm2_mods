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
    /// S1 Shadow Merge; recast (while merged): step out of the shadow
    S1 { aim: Point },
    /// S2 Shadow Cyclone: the clone's cleave at the point; recast: he cleaves himself at the point
    S2 { aim: Point },
    /// S3 Shadow Link: send the shadow toward the aimed point.
    S3 { aim: Point },
    Ult,
    /// older Lab transport: nothing to steer any more (ignored)
    Move { aim: Point },
    /// end Shadow Merge early (no step-out cross)
    End,
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
    /// his champion level (0: unknown, everything unlocked); S2 at 2, S3 at 3, the ult at 5 (`unlocks.rs`)
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
    /// where the enemy champions whose hits landed on him since the last frame stood (the Lab and tests; in the game
    /// the bridge asks `try_dodge` as the hit lands)
    #[serde(default)]
    pub incoming: Vec<Point>,
    #[serde(default)]
    pub shadow_link_enabled: bool,
    #[serde(default)]
    pub shadow_observation: Option<ShadowObservation>,
    /// Immune or otherwise ineligible for the arrival slow.
    #[serde(default)]
    pub link_excluded: Vec<usize>,
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
        let origin_tower = self.enemy_tower_at(from);
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

    fn step_out_destination(&self, away_from: Point) -> Point {
        let dir = if away_from == self.pos { (1.0, 0.0) } else { unit_dir(away_from, self.pos) };
        self.nearest_ground(self.clear_path(self.pos, self.pos.along(dir, d::MERGE_SLIP)))
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
    Damage { target: usize, amount: i64 },
    Stun { target: usize, ticks: u64 },
    KnockUp { target: usize, ticks: u64 },
    Slow { target: usize, pct: i64, ticks: u64 },
    /// push a unit along open ground (an engine glide)
    Knock { target: usize, to: Point, ticks: u64 },
    /// a deliberate move of his own (an engine glide): dodge slips, the step-out, the self-cleave
    Glide { to: Point, ticks: u64 },
    /// set him there at once (the ult's reappearance)
    Teleport { to: Point },
    /// the ult: unseen, unhurt, rooted and CC immune (a short lease, renewed)
    Vanish { ticks: u64 },
    /// unhurt for a moment: the self-cleave, the step-out slip
    Invulnerable { ticks: u64 },
    /// Shadow Merge on him: move speed for `ticks` (`ticks` 1 ends it)
    Merge { ticks: u64, move_pct: i64 },
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
    ShadowLand { cast: u64, at: Point, candidates: Vec<LinkTarget> },
    ShadowStrike { cast: u64, amount: i64 },
    ShadowEnd { cast: u64, at: Point, stored: i64, paid: bool },
    ShadowPayout { target: usize, amount: i64 },
    PreferShadow,
    HoldForward { threat: Point, cover: Point },
    Log { text: String },
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Cc {
    Stun(u64),
    KnockUp(u64),
    Slow(i64, u64),
}
impl Cc {
    fn halved(self) -> Cc {
        let h = |n: u64| n * d::PACT_CC_PCT as u64 / 100;
        match self {
            Cc::Stun(n) => Cc::Stun(h(n)),
            Cc::KnockUp(n) => Cc::KnockUp(h(n)),
            Cc::Slow(p, n) => Cc::Slow(p, h(n)),
        }
    }
    fn ticks(self) -> u64 {
        match self {
            Cc::Stun(n) | Cc::KnockUp(n) | Cc::Slow(_, n) => n,
        }
    }
    fn with_ticks(self, n: u64) -> Cc {
        match self {
            Cc::Stun(_) => Cc::Stun(n),
            Cc::KnockUp(_) => Cc::KnockUp(n),
            Cc::Slow(p, _) => Cc::Slow(p, n),
        }
    }
}

/// Where a hit came from (it governs echoes and Rhythm).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Src {
    Basic,
    Skill,
    Recast,
    Echo,
    Flurry,
    FlurryEcho,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Echo {
    pub target: usize,
    pub at: u64,
    pub amount: i64,
    pub cc: Option<Cc>,
    /// an echo of a flurry punch does not shorten the cooldown
    #[serde(default)]
    pub flurry: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Cleave {
    pub at: Point,
    pub next: u64,
    pub left: i64,
    pub every: u64,
    pub base: i64,
    pub ratio: i64,
    pub recast: bool,
    #[serde(default)]
    pub radius: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Flurry {
    pub start: u64,
    pub next: u64,
    pub done: i64,
    pub first: usize,
    pub last: Option<(usize, Point)>,
    pub knock_next: u64,
    pub vanished_at: Point,
    /// cast while merged: every punch echoes
    #[serde(default)]
    pub echo: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Stack {
    pub id: usize,
    pub count: i64,
    pub until: u64,
    pub champion: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct LinkTarget {
    pub id: usize,
    pub max_hp: i64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ShadowObservation {
    Landed { cast: u64, targets: Vec<usize> },
    Health { cast: u64, hp: i64 },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "phase", rename_all = "snake_case")]
pub enum ShadowLink {
    Flight { cast: u64, from: Point, at: Point, cursor: Point, landing: u64 },
    Pending { cast: u64, at: Point, landed: u64, candidates: Vec<LinkTarget> },
    Active { cast: u64, at: Point, until: u64, max_hp: i64, hp: i64, linked: Vec<usize> },
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
    /// shadow dodges (above the maximum only while merged)
    pub charges: i64,
    pub charge_at: u64,
    pub last_dodge: u64,
    /// a dodge to show and slip on the next step: where the hit came from
    #[serde(default)]
    pub pending_dodge: Option<Point>,
    /// S1-S3 ready ticks
    pub cd: [u64; 3],
    /// recast windows (end ticks) and whether each recast was used
    pub window: [u64; 3],
    #[serde(default)]
    pub used: [bool; 3],
    /// Shadow Merge until this tick
    #[serde(default)]
    pub merge_until: u64,
    pub cleaves: Vec<Cleave>,
    #[serde(default)]
    pub shadow_link: Option<ShadowLink>,
    /// the ult is ready from this tick, less the cut (contender-2.7: a 120 s cooldown)
    #[serde(default)]
    pub ult_at: u64,
    /// hundredths of a second cut off the ult cooldown by his hits since the last cast
    #[serde(default)]
    pub ult_cut_centis: u64,
    pub flurry: Option<Flurry>,
    pub haymaker_until: u64,
    pub echoes: Vec<Echo>,
    /// echoed crowd control per target: (target, window start, ticks used)
    pub cc_budget: Vec<(usize, u64, u64)>,
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
            cd: [0; 3],
            window: [0; 3],
            used: [false; 3],
            merge_until: 0,
            cleaves: vec![],
            shadow_link: None,
            ult_at: 0,
            ult_cut_centis: 0,
            flurry: None,
            haymaker_until: 0,
            echoes: vec![],
            cc_budget: vec![],
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
        self.cleaves.clear();
        self.shadow_link = None;
        self.flurry = None;
        self.echoes.clear();
        self.window = [0; 3];
        self.used = [false; 3];
        self.merge_until = 0;
        self.pending_dodge = None;
        self.haymaker_until = 0;
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
crate::unlocks::ult_unlocked(f.level, f.ult_unlocked) && f.tick >= self.ult_ready_at() && self.flurry.is_none()
    }
    /// the tick the ult is ready from: its cooldown less 0.01 s per hit since the last cast
    pub fn ult_ready_at(&self) -> u64 {
        self.ult_at.saturating_sub(self.ult_cut_centis * 60 / 100)
    }
    pub fn ready(&self, slot: usize, f: &Frame) -> bool {
        f.tick >= self.cd[slot] && crate::unlocks::skill_unlocked(f.level, slot)
    }
    /// a free recast is open for this slot
    pub fn recast_open(&self, slot: usize, f: &Frame) -> bool {
        f.tick < self.window[slot] && !self.used[slot]
    }
    pub fn stacks_on(&self, id: usize, tick: u64) -> i64 {
        self.stacks.iter().find(|s| s.id == id && tick < s.until).map_or(0, |s| s.count)
    }
    /// extra damage he deals to `id`, in permille (Shadow Rhythm's vulnerability)
    pub fn vuln_permille(&self, id: usize, tick: u64) -> i64 {
        self.stacks_on(id, tick) * d::RHYTHM_VULNERABILITY_PERMILLE_PER_STACK
    }
    fn dodge_gap(&self, tick: u64) -> u64 {
        t(if self.merged(tick) { d::MERGE_DODGE_GAP_TICKS } else { d::CHARGES_DODGE_GAP_TICKS })
    }
    fn recharge_ticks(&self, tick: u64) -> u64 {
        t(d::CHARGES_RECHARGE_TICKS / if self.merged(tick) { d::MERGE_RECHARGE_MULT.max(1) } else { 1 })
    }

    /// An enemy champion's hit (from `from`) is landing on him now: spend a dodge if one is ready. The bridge undoes the
    /// damage when this says yes; the slip and its shadow are drawn on the next step.
    pub fn try_dodge(&mut self, tick: u64, from: Point) -> bool {
        if self.flurry.is_some() || self.charges <= 0 || (self.last_dodge > 0 && tick < self.last_dodge + self.dodge_gap(tick)) {
            return false;
        }
        if self.charges == d::CHARGES_MAX {
            self.charge_at = tick + self.recharge_ticks(tick);
        }
        self.charges -= 1;
        self.last_dodge = tick.max(1);
        self.pending_dodge = Some(from);
        true
    }

    // -------------------------------------------------------------- step

    pub fn step(&mut self, f: &Frame) -> Vec<Effect> {
        let mut out = vec![];
        self.rejection.clear();
        if !f.alive {
            self.cancel_shadow(&mut out);
            self.respawn();
            return out;
        }
        for from in f.incoming.clone() {
            self.try_dodge(f.tick, from);
        }
        self.recharge(f);
        self.stacks.retain(|s| f.tick < s.until);
        self.cc_budget.retain(|b| f.tick < b.1 + t(d::PACT_CC_WINDOW_TICKS));
        if self.merge_until > 0 && f.tick >= self.merge_until {
            self.merge_until = 0;
            self.charges = self.charges.min(d::CHARGES_MAX);
        }
        self.advance_shadow(f, &mut out);
        self.slip(f, &mut out);
        for id in f.basic_hits.clone() {
            if let Some(u) = f.unit(id).filter(|u| u.team != f.team && !u.tower).cloned() {
                self.basic_landed(f, &u, &mut out);
            }
        }
        self.run_cleaves(f, &mut out);
        self.run_flurry(f, &mut out);
        self.run_echoes(f, &mut out);
        self.rhythm(f, &mut out);
        let order = if self.flurry.is_some() { None } else if f.manual { f.order } else { self.decide(f, &mut out) };
        if let Some(o) = order {
            if !f.stunned || matches!(o, Order::End) {
                self.execute(f, o, &mut out);
            }
        }
        if self.charges != self.charges_shown {
            self.charges_shown = self.charges;
            out.push(Effect::Charges { count: self.charges });
        }
        out
    }

    fn recharge(&mut self, f: &Frame) {
        if self.charges >= d::CHARGES_MAX {
            self.charge_at = 0;
            return;
        }
        if self.charge_at == 0 {
            self.charge_at = f.tick + self.recharge_ticks(f.tick);
        }
        if f.tick >= self.charge_at {
            self.charges += 1;
            self.charge_at = if self.charges < d::CHARGES_MAX { f.tick + self.recharge_ticks(f.tick) } else { 0 };
        }
    }
    fn reject(&mut self, why: &str) {
        self.rejection = why.into();
    }

    /// a dodge that landed: he slips aside, away from the hit, and a shadow stays where he stood
    fn slip(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        let Some(from) = self.pending_dodge.take() else { return };
        let away = unit_dir(from, f.pos);
        let side = if (f.tick / 7) % 2 == 0 { (-away.1, away.0) } else { (away.1, -away.0) };
        let dir = ((away.0 + side.0) * 0.7071, (away.1 + side.1) * 0.7071);
        let to = f.nearest_ground(f.clear_path(f.pos, f.pos.along(dir, d::CHARGES_DODGE_SLIP)));
        out.push(Effect::Glide { to, ticks: t(d::CHARGES_DODGE_SLIP_TICKS) });
        out.push(Effect::Fx { name: "ct_swap".into(), at: f.pos, ticks: 18, heading: None });
    }

    // -------------------------------------------------------------- hits, echoes and Rhythm

    /// One hit of his (or his shadows'). `apply`: deal the damage here (basic attacks are dealt by the engine).
    fn hit(&mut self, f: &Frame, u: &Unit, base: i64, cc: Option<Cc>, src: Src, apply: bool, out: &mut Vec<Effect>) {
        let amount = base * (1_000 + self.vuln_permille(u.id, f.tick)) / 1_000;
        if apply && amount > 0 {
            out.push(Effect::Damage { target: u.id, amount });
        }
        if let Some(c) = cc {
            self.cc(f, u.id, c, src == Src::Echo, out);
        }
        // echoes deal damage, carry crowd control and bank, but never add Rhythm stacks (contender-2.5)
        // contender-2.7: flurry punches still stack, but never set off a Shadow Break
        let flurryish = matches!(src, Src::Flurry | Src::FlurryEcho) || (src == Src::Echo && self.flurry.is_some());
        if src != Src::Echo {
            self.add_stacks(f, u, !flurryish, out);
        }
        // contender-2.7: every hit outside the flurry, echoes included, cuts the ult cooldown by 0.01 s
        if !flurryish {
            self.ult_cut_centis += d::ULT_CUT_CENTIS_PER_HIT as u64;
        }
        let echoes = match src {
            Src::Echo | Src::FlurryEcho => 0,
            Src::Flurry => i64::from(self.flurry.is_some_and(|fl| fl.echo)),
            Src::Recast => 1, // contender-2.6: recasts echo once, like any hit
            _ if self.merged(f.tick) => 2,
            _ => 1,
        };
        for k in 1..=echoes {
            self.echoes.push(Echo {
                target: u.id,
                at: f.tick + t(d::PACT_DELAY_TICKS) * k as u64,
                amount: base * d::PACT_PCT / 100,
                cc: cc.map(Cc::halved),
                flurry: src == Src::Flurry,
            });
        }
    }

    fn cc(&mut self, f: &Frame, target: usize, c: Cc, echo: bool, out: &mut Vec<Effect>) {
        let mut c = c;
        if echo {
            let cap = t(d::PACT_CC_CAP_TICKS);
            let i = match self.cc_budget.iter().position(|b| b.0 == target) {
                Some(i) => i,
                None => {
                    self.cc_budget.push((target, f.tick, 0));
                    self.cc_budget.len() - 1
                }
            };
            let left = cap.saturating_sub(self.cc_budget[i].2);
            let n = c.ticks().min(left);
            if n == 0 {
                return;
            }
            self.cc_budget[i].2 += n;
            c = c.with_ticks(n);
        }
        out.push(match c {
            Cc::Stun(n) => Effect::Stun { target, ticks: n },
            Cc::KnockUp(n) => Effect::KnockUp { target, ticks: n },
            Cc::Slow(p, n) => Effect::Slow { target, pct: p, ticks: n },
        });
    }

    /// Shadow Rhythm: a hit on a full target is a Shadow Break; otherwise it stacks (2 while merged).
    fn add_stacks(&mut self, f: &Frame, u: &Unit, can_break: bool, out: &mut Vec<Effect>) {
        let max = d::RHYTHM_MAX_STACKS;
        let until = f.tick + t(d::RHYTHM_STACK_TICKS);
        if self.stacks_on(u.id, f.tick) >= max {
            if !can_break {
                // a flurry punch on a full target: it holds at full; his next normal hit Breaks
                if let Some(s) = self.stacks.iter_mut().find(|s| s.id == u.id) {
                    s.until = until;
                }
                return;
            }
            let burst = u.max_hp * d::RHYTHM_BREAK_PERMILLE_MAX_HP / 1_000 + d::RHYTHM_BREAK_AD_RATIO * f.attack / 100;
            out.push(Effect::Damage { target: u.id, amount: burst });
            out.push(Effect::Fx { name: "ct_break".into(), at: u.pos, ticks: 18, heading: None });
            // the target's stacks reset to 0, but his Rhythm buffs stay at full for a while (the kill-linger path)
            self.stacks.retain(|s| s.id != u.id);
            let end = f.tick + t(d::RHYTHM_BREAK_BUFF_TICKS);
            self.linger = Some((max, self.linger.filter(|l| l.0 >= max).map_or(end, |l| l.1.max(end))));
            return;
        }
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
        let due: Vec<Echo> = self.echoes.iter().filter(|e| f.tick >= e.at).copied().collect();
        self.echoes.retain(|e| f.tick < e.at);
        for e in due {
            if let Some(u) = f.unit(e.target).filter(|u| f.skill_target(u)).cloned() {
                let src = if e.flurry { Src::FlurryEcho } else { Src::Echo };
                // a flurry echo counts as flurry damage (does not shorten the cooldown), still no echo of its own
                self.hit(f, &u, e.amount, e.cc, src, true, out);
                out.push(Effect::Fx { name: "ct_echo".into(), at: u.pos, ticks: 12, heading: None });
            }
        }
    }

    /// The engine's basic attack (his body's) landed.
    fn basic_landed(&mut self, f: &Frame, u: &Unit, out: &mut Vec<Effect>) {
        self.hit(f, u, 0, None, Src::Basic, false, out);
        // A basic echo uses Contender\x27s Attack rather than the zero placeholder.
        // (every echo this basic queued: two while merged)
        for e in self.echoes.iter_mut().filter(|e| e.target == u.id && e.amount == 0) {
            e.amount = f.attack * d::PACT_PCT / 100;
        }
        if f.tick < self.haymaker_until {
            self.haymaker_until = 0;
            let amount = pct(d::ULT_HAYMAKER_BASE, d::ULT_HAYMAKER_AD_RATIO, f.attack);
            self.hit(f, u, amount, Some(Cc::Stun(t(d::ULT_HAYMAKER_STUN_TICKS))), Src::Skill, true, out);
            out.push(Effect::Fx { name: "ct_haymaker".into(), at: u.pos, ticks: 18, heading: None });
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
        let mut v: Vec<Unit> = f.units.iter().filter(|u| f.skill_target(u) && u.pos.within(p, r + u.radius)).cloned().collect();
        v.sort_by_key(|u| (!u.champion, u.pos.d2(p), u.id));
        v
    }

    // -------------------------------------------------------------- orders

    pub fn execute(&mut self, f: &Frame, o: Order, out: &mut Vec<Effect>) {
        match o {
            Order::S1 { aim } => {
                if self.merged(f.tick) {
                    if !self.recast_open(0, f) {
                        return self.reject("s1 recast: used");
                    }
                    return self.step_out(f, aim, out);
                }
                if !crate::unlocks::skill_unlocked(f.level, 0) {
                    return self.reject(&crate::unlocks::locked_reason(0));
                }
                if !self.ready(0, f) {
                    return self.reject("s1");
                }
                self.merge(f, out)
            }
            Order::S2 { aim } => {
                if self.recast_open(1, f) {
                    return self.cyclone_recast(f, aim, out);
                }
                if !crate::unlocks::skill_unlocked(f.level, 1) {
                    return self.reject(&crate::unlocks::locked_reason(1));
                }
                if !self.ready(1, f) {
                    return self.reject("s2");
                }
                self.cyclone(f, aim, out)
            }
            Order::S3 { aim } => {
                if !crate::unlocks::skill_unlocked(f.level, 2) {
                    return self.reject(&crate::unlocks::locked_reason(2));
                }
                if !f.shadow_link_enabled {
                    return self.reject("s3: Shadow Link host verification pending");
                }
                if !self.ready(2, f) || self.shadow_link.is_some() {
                    return self.reject("s3: not ready");
                }
                self.cast_shadow(f, aim)
            }
            Order::Ult => {
                if !self.ult_ready(f) {
                    return self.reject("ult");
                }
                self.vanish(f, out)
            }
            Order::Move { .. } => self.reject("move: nothing to steer"),
            Order::End => {
                if self.merged(f.tick) {
                    self.merge_until = 0;
                    self.charges = self.charges.min(d::CHARGES_MAX);
                    out.push(Effect::Merge { ticks: 1, move_pct: 0 });
                }
            }
        }
    }

    // -------------------------------------------------------------- S1 Shadow Merge

    fn merge(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        let ticks = t(d::MERGE_TICKS);
        self.cd[0] = f.tick + t(d::MERGE_COOLDOWN_TICKS);
        self.merge_until = f.tick + ticks;
        self.window[0] = f.tick + ticks;
        self.used[0] = false;
        self.charges += d::MERGE_BONUS_DODGES;
        out.push(Effect::Merge { ticks, move_pct: d::MERGE_MOVE_SPEED_PCT });
        out.push(Effect::Fx { name: "ct_launch".into(), at: f.pos, ticks: 18, heading: None });
    }

    /// S1 recast: he steps out of the shadow; the shadow's cross knocks the nearest enemy back and he slips away.
    fn step_out(&mut self, f: &Frame, aim: Point, out: &mut Vec<Effect>) {
        self.used[0] = true;
        self.merge_until = 0;
        self.charges = self.charges.min(d::CHARGES_MAX);
        out.push(Effect::Merge { ticks: 1, move_pct: 0 });
        out.push(Effect::Fx { name: "ct_swap".into(), at: f.pos, ticks: 18, heading: None });
        let near = Self::targets_near(f, f.pos, d::MERGE_CROSS_RADIUS).first().cloned();
        let away_from = near.as_ref().map_or(aim, |u| u.pos);
        self.strike_shadow(f.pos, d::MERGE_CROSS_RADIUS, pct(d::MERGE_CROSS_BASE, d::MERGE_CROSS_AD_RATIO, f.attack), out);
        if let Some(u) = near {
            let dir = unit_dir(f.pos, u.pos);
            let knock = f.clear_path(u.pos, u.pos.along(dir, d::MERGE_CROSS_KNOCK));
            self.hit(f, &u, pct(d::MERGE_CROSS_BASE, d::MERGE_CROSS_AD_RATIO, f.attack), None, Src::Recast, true, out);
            out.push(Effect::Knock { target: u.id, to: knock, ticks: t(d::MERGE_CROSS_KNOCK_TICKS) });
            out.push(Effect::Fx { name: "ct_cross".into(), at: u.pos, ticks: 18, heading: None });
        }
        let to = f.step_out_destination(away_from);
        out.push(Effect::Glide { to, ticks: t(d::MERGE_SLIP_TICKS) });
        out.push(Effect::Invulnerable { ticks: t(d::MERGE_SLIP_TICKS) + 7 });
    }

    // -------------------------------------------------------------- S2 Shadow Cyclone

    fn cyclone(&mut self, f: &Frame, aim: Point, out: &mut Vec<Effect>) {
        let spot = f.nearest_ground(f.clear_path(f.pos, f.pos.toward(aim, d::CYCLONE_RANGE)));
        self.cd[1] = f.tick + t(d::CYCLONE_COOLDOWN_TICKS);
        self.window[1] = f.tick + t(d::CHARGES_RECAST_WINDOW_TICKS);
        self.used[1] = false;
        self.cleaves.push(Cleave {
            at: spot,
            next: f.tick + t(d::CYCLONE_EVERY_TICKS),
            left: d::CYCLONE_HITS,
            every: t(d::CYCLONE_EVERY_TICKS),
            base: d::CYCLONE_BASE,
            ratio: d::CYCLONE_AD_RATIO,
            recast: false,
            radius: d::CYCLONE_RADIUS,
        });
        out.push(Effect::Fx { name: "ct_shadow".into(), at: spot, ticks: t(d::CYCLONE_HITS * d::CYCLONE_EVERY_TICKS) + 6, heading: None });
    }

    /// S2 recast: he cleaves himself anywhere within reach, invulnerable, in a bigger circle that pulls enemies in.
    fn cyclone_recast(&mut self, f: &Frame, aim: Point, out: &mut Vec<Effect>) {
        self.used[1] = true;
        self.window[1] = 0;
        // the clone stops: his own cleave takes over
        self.cleaves.retain(|c| c.recast);
        let spot = f.nearest_ground(f.clear_path(f.pos, f.pos.toward(aim, d::CYCLONE_RECAST_RANGE)));
        let span = t(d::CYCLONE_RECAST_HITS * d::CYCLONE_RECAST_EVERY_TICKS);
        out.push(Effect::Glide { to: spot, ticks: 8 });
        out.push(Effect::Invulnerable { ticks: span + 12 });
        self.cleaves.push(Cleave {
            at: spot,
            next: f.tick + 8 + t(d::CYCLONE_RECAST_EVERY_TICKS),
            left: d::CYCLONE_RECAST_HITS,
            every: t(d::CYCLONE_RECAST_EVERY_TICKS),
            base: d::CYCLONE_RECAST_BASE,
            ratio: d::CYCLONE_RECAST_AD_RATIO,
            recast: true,
            radius: d::CYCLONE_RECAST_RADIUS,
        });
    }

    fn run_cleaves(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        let mut cleaves = std::mem::take(&mut self.cleaves);
        for c in cleaves.iter_mut() {
            if f.tick < c.next || c.left <= 0 {
                continue;
            }
            c.left -= 1;
            c.next = f.tick + c.every;
            let last = c.left == 0;
            let radius = if c.radius > 0 { c.radius } else { d::CYCLONE_RADIUS };
            out.push(Effect::Fx { name: "ct_cleave".into(), at: c.at, ticks: c.every + 2, heading: None });
            self.strike_shadow(c.at, radius, pct(c.base, c.ratio, f.attack), out);
            for u in Self::targets_near(f, c.at, radius) {
                let cc = if c.recast {
                    last.then_some(Cc::KnockUp(t(d::CYCLONE_RECAST_KNOCK_TICKS)))
                } else {
                    Some(Cc::Slow(d::CYCLONE_SLOW_PCT, t(d::CYCLONE_SLOW_TICKS)))
                };
                self.hit(f, &u, pct(c.base, c.ratio, f.attack), cc, if c.recast { Src::Recast } else { Src::Skill }, true, out);
                // the self-cleave drags them toward its centre
                if c.recast && !last {
                    let gap = u.pos.dist(c.at) as i64;
                    if gap > 3_000 {
                        let to = f.clear_path(u.pos, u.pos.toward(c.at, d::CYCLONE_RECAST_PULL.min(gap - 2_000)));
                        out.push(Effect::Knock { target: u.id, to, ticks: t(d::CYCLONE_RECAST_PULL_TICKS) });
                    }
                }
            }
        }
        cleaves.retain(|c| c.left > 0);
        self.cleaves.extend(cleaves);
    }

    // -------------------------------------------------------------- S3 Shadow Link

    fn cast_shadow(&mut self, f: &Frame, aim: Point) {
        let destination = f.pos.toward(aim, d::SHADOW_LINK_RANGE);
        let at = f.path_until(f.pos, destination, false);
        if at == f.pos {
            return self.reject("s3: no legal dash");
        }
        let distance = f.pos.dist(at).ceil() as i64;
        let ticks = (distance * d::SHADOW_LINK_FLIGHT_TICKS + d::SHADOW_LINK_RANGE - 1) / d::SHADOW_LINK_RANGE;
        self.cd[2] = f.tick + t(d::SHADOW_LINK_COOLDOWN_TICKS);
        self.shadow_link = Some(ShadowLink::Flight { cast: f.tick, from: f.pos, at, cursor: f.pos, landing: f.tick + t(ticks.max(1)) });
    }

    fn advance_shadow(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        if !f.shadow_link_enabled {
            self.cancel_shadow(out);
            return;
        }
        let Some(shadow) = self.shadow_link.take() else { return };
        match shadow {
            ShadowLink::Flight { cast, from, at, cursor, landing } => {
                let elapsed = f.tick.saturating_sub(cast).min(landing - cast) as i64;
                let duration = (landing - cast) as i64;
                let expected = Point::new(from.x + (at.x - from.x) * elapsed / duration, from.y + (at.y - from.y) * elapsed / duration);
                let cleared = f.path_until(cursor, expected, false);
                if f.tick < landing && cleared == expected {
                    self.shadow_link = Some(ShadowLink::Flight { cast, from, at, cursor: cleared, landing });
                    return;
                }
                let at = cleared;
                let mut candidates: Vec<_> = f.units.iter()
                    .filter(|unit| f.link_target(unit) && unit.pos.within(at, d::SHADOW_LINK_RADIUS + unit.radius))
                    .map(|unit| LinkTarget { id: unit.id, max_hp: unit.max_hp })
                    .collect();
                candidates.sort_by_key(|target| target.id);
                candidates.dedup_by_key(|target| target.id);
                out.push(Effect::ShadowLand { cast, at, candidates: candidates.clone() });
                if candidates.is_empty() {
                    out.push(Effect::ShadowEnd { cast, at, stored: 0, paid: false });
                } else {
                    self.shadow_link = Some(ShadowLink::Pending { cast, at, landed: f.tick, candidates });
                }
            }
            ShadowLink::Pending { cast, at, landed, candidates } => {
                if let Some(ShadowObservation::Landed { cast: observed, targets }) = &f.shadow_observation {
                    if *observed == cast {
                        let confirmed: Vec<_> = candidates.iter().filter(|target| targets.contains(&target.id)).collect();
                        let max_hp = confirmed.iter().map(|target| target.max_hp).max().unwrap_or(0);
                        if max_hp > 0 {
                            self.shadow_link = Some(ShadowLink::Active {
                                cast, at, until: landed + t(d::SHADOW_LINK_TICKS), max_hp, hp: max_hp,
                                linked: confirmed.iter().map(|target| target.id).collect(),
                            });
                            return;
                        }
                    }
                }
                out.push(Effect::ShadowEnd { cast, at, stored: 0, paid: false });
            }
            ShadowLink::Active { cast, at, until, max_hp, hp, mut linked } => {
                linked.retain(|id| !f.separated.contains(id) && f.unit(*id)
                    .is_some_and(|unit| unit.hp > 0 && unit.pos.within(at, d::SHADOW_LINK_BREAK_RANGE)));
                let observed = match f.shadow_observation {
                    Some(ShadowObservation::Health { cast: observed, hp }) if observed == cast => Some(hp.clamp(0, max_hp)),
                    _ => None,
                };
                if linked.is_empty() || observed.is_none() {
                    out.push(Effect::ShadowEnd { cast, at, stored: max_hp - hp, paid: false });
                    return;
                }
                let hp = hp.min(observed.unwrap_or(hp));
                if hp == 0 || f.tick >= until {
                    let stored = max_hp - hp;
                    let mut paid = false;
                    for id in linked {
                        if let Some(unit) = f.unit(id).filter(|unit| unit.targetable && unit.team != f.team) {
                            let amount = stored.min(unit.max_hp * d::SHADOW_LINK_MAX_HP_DAMAGE_PCT / 100);
                            if amount > 0 {
                                paid = true;
                                out.push(Effect::ShadowPayout { target: id, amount });
                            }
                        }
                    }
                    out.push(Effect::ShadowEnd { cast, at, stored, paid });
                } else {
                    self.shadow_link = Some(ShadowLink::Active { cast, at, until, max_hp, hp, linked });
                }
            }
        }
    }

    fn cancel_shadow(&mut self, out: &mut Vec<Effect>) {
        if let Some(shadow) = self.shadow_link.take() {
            let (cast, at, stored) = match shadow {
                ShadowLink::Flight { cast, at, .. } | ShadowLink::Pending { cast, at, .. } => (cast, at, 0),
                ShadowLink::Active { cast, at, max_hp, hp, .. } => (cast, at, max_hp - hp),
            };
            out.push(Effect::ShadowEnd { cast, at, stored, paid: false });
        }
    }

    fn strike_shadow(&self, center: Point, radius: i64, amount: i64, out: &mut Vec<Effect>) {
        if let Some(ShadowLink::Active { cast, at, .. }) = self.shadow_link {
            if amount > 0 && center.within(at, radius) {
                out.push(Effect::ShadowStrike { cast, amount });
            }
        }
    }

    // -------------------------------------------------------------- Ult Shadow Flurry

    fn vanish(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        let Some(first) = Self::targets_near(f, f.pos, d::ULT_START_RANGE).first().cloned() else {
            return self.reject("ult: nothing in reach");
        };
        // contender-2.7: a merged cast no longer echoes; the cooldown starts and the cut resets
        let echo = false;
        self.ult_at = f.tick + t(d::ULT_COOLDOWN_TICKS);
        self.ult_cut_centis = 0;
        self.flurry = Some(Flurry {
            start: f.tick,
            next: f.tick + 6,
            done: 0,
            first: first.id,
            last: None,
            knock_next: f.tick + t(d::ULT_KNOCK_EVERY_TICKS),
            vanished_at: f.pos,
            echo,
        });
        out.push(Effect::Vanish { ticks: 3 });
        out.push(Effect::Fx { name: "ct_vanish".into(), at: f.pos, ticks: 24, heading: None });
    }

    /// Read-only visual sequence: covers every direction without combat RNG.
    pub fn flurry_heading(owner: usize, punch: i64) -> u8 {
        ((owner % 8 + 3 * punch.rem_euclid(8) as usize) % 8) as u8
    }

    /// the gap before punch `k` (0-based): slow at first, fast at the end
    pub fn flurry_gap(k: i64) -> u64 {
        let n = (d::ULT_PUNCHES - 1).max(1);
        let span = d::ULT_FIRST_GAP_TICKS - d::ULT_LAST_GAP_TICKS;
        t(d::ULT_FIRST_GAP_TICKS - (span * k.min(n) + n / 2) / n)
    }

    fn run_flurry(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        let Some(mut fl) = self.flurry else { return };
        let end = fl.start + t(d::ULT_TICKS);
        let mut over = f.tick >= end || fl.done >= d::ULT_PUNCHES;
        while !over && f.tick >= fl.next && fl.done < d::ULT_PUNCHES {
            let pick = match fl.last {
                None => f.unit(fl.first).filter(|u| f.skill_target(u)).cloned(),
                Some((last, at)) => {
                    let nearby = Self::targets_near(f, at, d::ULT_CHAIN_RANGE);
                    // The last victim is usually distance zero. Prefer another
                    // eligible enemy before applying the existing deterministic
                    // champion/distance/id ordering; retain the solo-target fallback.
                    nearby.iter().find(|u| u.id != last).or_else(|| nearby.first()).cloned()
                }
            };
            let Some(u) = pick else {
                over = true;
                break;
            };
            self.flurry = Some(fl);
            self.hit(f, &u, pct(d::ULT_BASE, d::ULT_AD_RATIO, f.attack), None, Src::Flurry, true, out);
            fl = self.flurry.unwrap_or(fl);
            out.push(Effect::Fx { name: "ct_flurry".into(), at: u.pos, ticks: 8, heading: Some(Self::flurry_heading(f.owner, fl.done)) });
            fl.last = Some((u.id, u.pos));
            fl.next += Self::flurry_gap(fl.done);
            fl.done += 1;
        }
        if !over && f.tick >= fl.knock_next {
            fl.knock_next += t(d::ULT_KNOCK_EVERY_TICKS);
            if let Some((id, _)) = fl.last {
                if f.unit(id).is_some_and(|u| f.skill_target(u)) {
                    out.push(Effect::KnockUp { target: id, ticks: t(d::ULT_KNOCK_TICKS) });
                }
            }
        }
        if over || fl.done >= d::ULT_PUNCHES {
            self.flurry = None;
            let back = fl.last.and_then(|(id, at)| f.unit(id).map(|u| u.pos).or(Some(at)));
            if let Some(p) = back {
                let to = f.nearest_ground(p.toward(fl.vanished_at, 10_000));
                out.push(Effect::Teleport { to });
                out.push(Effect::Fx { name: "ct_appear".into(), at: to, ticks: 18, heading: None });
            }
            // contender-2.6: his dodges no longer refill when he comes back
            self.haymaker_until = f.tick + t(d::ULT_HAYMAKER_TICKS);
            return;
        }
        out.push(Effect::Vanish { ticks: 3 });
        self.flurry = Some(fl);
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

    fn shadow_aim(&self, f: &Frame, peeling: bool) -> Option<Point> {
        if !f.shadow_link_enabled || !self.ready(2, f) || self.shadow_link.is_some() {
            return None;
        }
        f.units.iter().filter(|unit| f.link_target(unit) && (unit.champion || (!peeling && unit.farmable)))
            .filter(|unit| unit.pos.within(f.pos, d::SHADOW_LINK_RANGE + d::SHADOW_LINK_RADIUS + unit.radius))
            // Try close placements too: the shadow can cover opponents from the edge of his punch range.
            .flat_map(|unit| [d::AI_SHADOW_STEP, d::BASIC_RANGE, d::SHADOW_LINK_RANGE]
                .map(|range| f.pos.toward(unit.pos, range)))
            .map(|at| f.path_until(f.pos, at, false))
            .filter(|at| *at != f.pos)
            .filter_map(|at| {
                let champions = f.enemy_champions().filter(|unit| f.link_target(unit)
                    && unit.pos.within(at, d::SHADOW_LINK_RADIUS + unit.radius)
                    && (!peeling || unit.pos.within(f.pos, d::AI_THREAT_RANGE))).count();
                let farm = f.units.iter().filter(|unit| f.link_target(unit) && unit.farmable
                    && unit.pos.within(at, d::SHADOW_LINK_RADIUS + unit.radius)).count();
                (champions > 0 || (!peeling && Self::nearby_threats(f, f.pos) == 0 && farm > 0))
                    .then_some((champions, farm, at))
            })
            .max_by_key(|(champions, farm, at)| (*champions, *farm, std::cmp::Reverse(at.d2(f.pos))))
            .map(|(_, _, at)| at)
    }

    fn safe_flurry(&self, f: &Frame) -> bool {
        let Some(first) = Self::targets_near(f, f.pos, d::ULT_START_RANGE).into_iter().next() else { return false };
        if !first.champion || self.solo_finish(f, &first) {
            return false;
        }
        // Follow the currently visible chain component conservatively. Any member may be the last
        // strike after deaths or movement. Check each possible landing before committing.
        let mut candidates = vec![&first];
        let mut checked = 0;
        while checked < candidates.len() {
            let unit = candidates[checked];
            checked += 1;
            let land = f.nearest_ground(unit.pos.toward(f.pos, 10_000));
            if !Self::safe_commit(f, land) {
                return false;
            }
            for next in f.units.iter().filter(|next| f.skill_target(next)
                && next.pos.within(unit.pos, d::ULT_CHAIN_RANGE + next.radius)) {
                if !candidates.iter().any(|candidate| candidate.id == next.id) {
                    candidates.push(next);
                }
            }
        }
        candidates.iter().filter(|unit| unit.champion).count() >= 2 || Self::hurt_pct(&first) <= 35
    }

    pub fn decide(&mut self, f: &Frame, out: &mut Vec<Effect>) -> Option<Order> {
        if f.stunned || self.flurry.is_some() {
            return None;
        }
        let target = self.pick(f);
        if f.hp * 100 < d::AI_RETREAT_HP * f.max_hp.max(1) {
            if !self.reason.starts_with("Retreat") && !self.reason.starts_with("Peel") {
                self.steer_at = 0;
            }
            self.reason = "Retreat: low HP".into();
            if let Some(foe) = target.as_ref() {
                self.hold_forward(f, foe.pos, out);
                if foe.pos.within(f.pos, 25_000) && self.merged(f.tick) && self.recast_open(0, f) {
                    let nearest = Self::targets_near(f, f.pos, d::MERGE_CROSS_RADIUS).into_iter().next();
                    let source = nearest.as_ref().map_or(foe.pos, |unit| unit.pos);
                    let to = f.step_out_destination(source);
                    if to.d2(source) > f.pos.d2(source) && f.path_until(f.pos, to, true) == to
                        && Self::nearby_threats(f, to) <= Self::nearby_threats(f, f.pos) {
                        self.reason = "Retreat: Merge step-out".into();
                        return Some(Order::S1 { aim: source });
                    }
                }
                if let Some(cover) = Self::cover(f, f.pos) {
                    self.steer(f, cover, out);
                }
                if let Some(aim) = self.shadow_aim(f, true) {
                    self.reason = "Peel: Shadow Link".into();
                    return Some(Order::S3 { aim });
                }
            } else if let Some(cover) = Self::cover(f, f.pos) {
                self.steer(f, cover, out);
            }
            return None;
        }

        if let Some(ShadowLink::Active { at, .. }) = self.shadow_link {
            if at.within(f.pos, d::BASIC_RANGE) && f.enemy_tower_at(f.pos).is_none()
                && f.path_until(f.pos, at, false) == at {
                out.push(Effect::PreferShadow);
                self.reason = "Hold: attack the shadow".into();
                if let Some(foe) = &target { self.hold_forward(f, foe.pos, out); }
                return None;
            }
            let step = f.pos.toward(at, d::AI_SHADOW_STEP);
            if at.within(step, d::BASIC_RANGE) && f.path_until(f.pos, step, true) == step
                && Self::nearby_threats(f, step) <= Self::nearby_threats(f, f.pos)
                && f.enemy_champions().all(|enemy| !enemy.pos.within(step, 25_000))
                && Self::cover(f, step).is_some() {
                self.reason = "Hold: safe step to the shadow".into();
                self.steer(f, step, out);
                return None;
            }
        }

        if let Some(aim) = self.shadow_aim(f, false) {
            self.reason = if target.is_some() { "Poke: Shadow Link" } else { "Farm: Shadow Link" }.into();
            if let Some(foe) = &target { self.hold_forward(f, foe.pos, out); }
            return Some(Order::S3 { aim });
        }

        if let Some(foe) = target {
            if self.solo_finish(f, &foe) && Self::cover(f, f.pos).is_some() {
                self.reason = "Commit: basic can finish".into();
                return None;
            }
            let dist = f.pos.dist(foe.pos) as i64;
            let limit = d::AI_CHASE_RANGE + f.p(6) * d::AI_AGGRESSIVE_CHASE_PER_POINT;
            let landing = f.nearest_ground(f.clear_path(f.pos, f.pos.toward(foe.pos, d::CYCLONE_RECAST_RANGE)));
            if self.recast_open(1, f) && dist <= d::CYCLONE_RECAST_RANGE
                && foe.pos.within(landing, d::CYCLONE_RECAST_RADIUS + foe.radius) && Self::safe_commit(f, landing) {
                self.reason = "Commit: supported Cyclone recast".into();
                return Some(Order::S2 { aim: foe.pos });
            }
            if self.ult_ready(f) && self.safe_flurry(f) {
                self.reason = "Commit: supported Shadow Flurry".into();
                return Some(Order::Ult);
            }
            if !self.merged(f.tick) && self.ready(0, f) && dist <= d::AI_MERGE_RANGE && Self::safe_commit(f, foe.pos) {
                self.reason = "Commit: supported Merge".into();
                return Some(Order::S1 { aim: foe.pos });
            }
            if self.ready(1, f) && dist <= d::CYCLONE_RANGE + d::CYCLONE_RADIUS / 2 {
                self.reason = "Poke: Shadow Cyclone".into();
                self.hold_forward(f, foe.pos, out);
                return Some(Order::S2 { aim: foe.pos });
            }
            if dist <= limit && dist > d::BASIC_RANGE && Self::safe_commit(f, foe.pos) {
                self.reason = "Commit: supported approach".into();
                self.steer(f, foe.pos, out);
            } else {
                self.reason = "Hold: unsafe or unsupported approach".into();
                self.hold_forward(f, foe.pos, out);
            }
            return None;
        }

        if f.enemy_champions().all(|unit| !unit.pos.within(f.pos, d::AI_CALM_RANGE)) {
            let farm = f.units.iter().filter(|unit| unit.farmable && f.skill_target(unit)
                && unit.pos.within(f.pos, 90_000)).min_by_key(|unit| (unit.pos.d2(f.pos), unit.id));
            if let Some(unit) = farm {
                let clustered = f.units.iter().filter(|other| other.farmable && f.skill_target(other)
                    && other.pos.within(unit.pos, d::CYCLONE_RADIUS)).count() >= 2;
                if self.ready(1, f) && unit.pos.within(f.pos, d::CYCLONE_RANGE)
                    && (clustered || unit.max_hp >= d::AI_FARM_BIG_HP) {
                    self.reason = "Farm: Shadow Cyclone".into();
                    return Some(Order::S2 { aim: unit.pos });
                }
            }
        }
        self.reason.clear();
        None
    }
}
