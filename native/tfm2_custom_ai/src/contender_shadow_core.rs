//! Contender 2.3, the elusive Shadow Boxer: a deterministic core (snapshot in, effects out). `contender.rs` builds the
//! frame from the sim and applies the effects; the Lab (`native/contender_shadow_lab`) runs this same file.
//!
//! Reihan's 2.3 ("like a shadow, really hard to hit since he dodges, but his punches don't do a lot, focusing on the
//! stacks"):
//!   P1 Shadows: up to 5 dodges (one every 6 s; each basic attack that lands takes 0.5 s off). An enemy champion's hit
//!      spends one: he slips aside and the hit is undone (one dodge per 0.4 s). Skills cost nothing; each can be recast
//!      once, free, in its window.
//!   P2 Shadow Pact: every hit echoes 1 s later for 40%, its crowd control halved (at most 1.5 s more per target every
//!      6 s). Recast hits and every hit while merged echo twice; the flurry echoes when cast while merged.
//!   P3 Shadow Rhythm: hits stack their target (10 max, 4 s); +4% damage from him per stack; the most-stacked enemy
//!      buffs him. At 10 stacks his next hit is a Shadow Break (6% max HP + 30% Attack) and the target drops to 5.
//!   S1 Shadow Merge: 6 s merged with his shadow: dodges recharge twice as fast, +2 dodges, every hit echoes twice and
//!      adds 2 stacks, +15% move speed. Recast: he steps out, the shadow's cross knocks the nearest enemy back and he
//!      slips away.
//!   S2 Shadow Cyclone: a clone cleaves a circle (it can't be hit). Recast: he cleaves anywhere within 60,000,
//!      invulnerable, in a bigger circle that pulls enemies in, the last hit knocking up.
//!   S3 Shadow Tether: a shadow latches onto every enemy within 50,000 (5 at most) for 4 s and copies each of his hits
//!      onto them (30%; flurry punches 15%). Recast: every tethered enemy is yanked toward him and uppercut.
//!   Ult Shadow Flurry: cast with 20 banked stacks (every hit outside the flurry banks one); 5 s unseen and unhurt, 36
//!      accelerating chained punches (each prefers a new enemy), a knock-up every second, an early end when nothing is
//!      in reach; he reappears by the last enemy hit with a haymaker ready and his dodges full.
//!
//! His AI is his own (Kicker's flat list); the 13 athlete values shape it. Nothing of his hits a tower.
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
    /// S3 Shadow Tether (the point is ignored: every enemy in reach); recast: the yank and uppercut
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
    /// his banked shadow stacks toward the ult
    Bank { count: i64 },
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

/// Where a hit came from (it decides echoes, copies and the bank).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Src {
    Basic,
    Skill,
    Recast,
    Echo,
    Flurry,
    /// a Shadow Tether copy (`true`: of a flurry punch)
    Copy(bool),
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Echo {
    pub target: usize,
    pub at: u64,
    pub amount: i64,
    pub cc: Option<Cc>,
    /// an echo of a flurry punch never banks
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
    /// tethered enemies: (id, until)
    #[serde(default)]
    pub tethers: Vec<(usize, u64)>,
    /// banked shadow stacks toward the ult
    #[serde(default)]
    pub bank: i64,
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
    #[serde(default)]
    pub bank_shown: i64,
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
            tethers: vec![],
            bank: 0,
            flurry: None,
            haymaker_until: 0,
            echoes: vec![],
            cc_budget: vec![],
            stacks: vec![],
            focus: None,
            linger: None,
            rhythm_shown: (0, 0),
            charges_shown: -1,
            bank_shown: -1,
            steer_at: 0,
            reason: String::new(),
            rejection: String::new(),
        }
    }
}

impl Core {
    pub fn respawn(&mut self) {
        self.cleaves.clear();
        self.tethers.clear();
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
        f.ult_unlocked && self.bank >= d::ULT_BANK_MAX && self.flurry.is_none()
    }
    pub fn ready(&self, slot: usize, f: &Frame) -> bool {
        f.tick >= self.cd[slot]
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
        self.tethers.retain(|(id, until)| f.tick < *until && f.unit(*id).is_some_and(|u| f.skill_target(u) && u.pos.within(f.pos, d::TETHER_BREAK_RANGE)));
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
        if self.bank != self.bank_shown {
            self.bank_shown = self.bank;
            out.push(Effect::Bank { count: self.bank });
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

    // -------------------------------------------------------------- hits, echoes, stacks, copies, the bank

    /// One hit of his (or his shadows'). `apply`: deal the damage here (basic attacks are dealt by the engine).
    fn hit(&mut self, f: &Frame, u: &Unit, base: i64, cc: Option<Cc>, src: Src, apply: bool, out: &mut Vec<Effect>) {
        let amount = base * (1_000 + self.vuln_permille(u.id, f.tick)) / 1_000;
        if apply && amount > 0 {
            out.push(Effect::Damage { target: u.id, amount });
        }
        if let Some(c) = cc {
            self.cc(f, u.id, c, src == Src::Echo, out);
        }
        self.add_stacks(f, u, out);
        let flurryish = matches!(src, Src::Flurry | Src::Copy(true));
        if !flurryish && !(src == Src::Echo && self.flurry.is_some()) {
            self.bank = (self.bank + 1).min(d::ULT_BANK_MAX);
        }
        let echoes = match src {
            Src::Echo | Src::Copy(_) => 0,
            Src::Flurry => i64::from(self.flurry.is_some_and(|fl| fl.echo)),
            Src::Recast => 2,
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
        // Shadow Tether: every tethered enemy takes a copy (copies and echoes are never copied again; basics copy in
        // basic_landed, where his Attack is known)
        if matches!(src, Src::Skill | Src::Recast | Src::Flurry) && !self.tethers.is_empty() {
            let pct = if src == Src::Flurry { d::TETHER_FLURRY_COPY_PCT } else { d::TETHER_COPY_PCT };
            let copies: Vec<Unit> = self.tethers.iter().filter(|(id, _)| *id != u.id).filter_map(|(id, _)| f.unit(*id).filter(|v| f.skill_target(v)).cloned()).collect();
            for v in copies {
                self.hit(f, &v, base * pct / 100, None, Src::Copy(src == Src::Flurry), true, out);
                out.push(Effect::Fx { name: "ct_echo".into(), at: v.pos, ticks: 12, heading: None });
            }
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
    fn add_stacks(&mut self, f: &Frame, u: &Unit, out: &mut Vec<Effect>) {
        let max = d::RHYTHM_MAX_STACKS;
        let until = f.tick + t(d::RHYTHM_STACK_TICKS);
        if self.stacks_on(u.id, f.tick) >= max {
            let burst = u.max_hp * d::RHYTHM_BREAK_PERMILLE_MAX_HP / 1_000 + d::RHYTHM_BREAK_AD_RATIO * f.attack / 100;
            out.push(Effect::Damage { target: u.id, amount: burst });
            out.push(Effect::Fx { name: "ct_haymaker".into(), at: u.pos, ticks: 18, heading: None });
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
                let src = if e.flurry { Src::Copy(true) } else { Src::Echo };
                // a flurry echo counts as flurry damage (never banks), still no echo of its own
                self.hit(f, &u, e.amount, e.cc, src, true, out);
                out.push(Effect::Fx { name: "ct_echo".into(), at: u.pos, ticks: 12, heading: None });
            }
        }
    }

    /// The engine's basic attack (his body's) landed.
    fn basic_landed(&mut self, f: &Frame, u: &Unit, out: &mut Vec<Effect>) {
        self.hit(f, u, 0, None, Src::Basic, false, out);
        // the echo of a basic is 40% of his Attack; tether copies of it are 30% of his Attack
        if let Some(e) = self.echoes.iter_mut().rev().find(|e| e.target == u.id && e.amount == 0) {
            e.amount = f.attack * d::PACT_PCT / 100;
        }
        if !self.tethers.is_empty() {
            let copies: Vec<Unit> = self.tethers.iter().filter(|(id, _)| *id != u.id).filter_map(|(id, _)| f.unit(*id).filter(|v| f.skill_target(v)).cloned()).collect();
            for v in copies {
                self.hit(f, &v, f.attack * d::TETHER_COPY_PCT / 100, None, Src::Copy(false), true, out);
                out.push(Effect::Fx { name: "ct_echo".into(), at: v.pos, ticks: 12, heading: None });
            }
        }
        if self.charges < d::CHARGES_MAX && self.charge_at > 0 {
            self.charge_at = self.charge_at.saturating_sub(t(d::CHARGES_BASIC_CUT_TICKS));
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
                if !self.ready(0, f) {
                    return self.reject("s1");
                }
                self.merge(f, out)
            }
            Order::S2 { aim } => {
                if self.recast_open(1, f) {
                    return self.cyclone_recast(f, aim, out);
                }
                if !self.ready(1, f) {
                    return self.reject("s2");
                }
                self.cyclone(f, aim, out)
            }
            Order::S3 { .. } => {
                if self.recast_open(2, f) && !self.tethers.is_empty() {
                    return self.yank(f, out);
                }
                if !self.ready(2, f) {
                    return self.reject("s3");
                }
                self.tether(f, out)
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
        if let Some(u) = near {
            let dir = unit_dir(f.pos, u.pos);
            let knock = f.clear_path(u.pos, u.pos.along(dir, d::MERGE_CROSS_KNOCK));
            self.hit(f, &u, pct(d::MERGE_CROSS_BASE, d::MERGE_CROSS_AD_RATIO, f.attack), None, Src::Recast, true, out);
            out.push(Effect::Knock { target: u.id, to: knock, ticks: t(d::MERGE_CROSS_KNOCK_TICKS) });
            out.push(Effect::Fx { name: "ct_cross".into(), at: u.pos, ticks: 18, heading: None });
        }
        let dir = if away_from == f.pos { (1.0, 0.0) } else { unit_dir(away_from, f.pos) };
        let to = f.nearest_ground(f.clear_path(f.pos, f.pos.along(dir, d::MERGE_SLIP)));
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

    // -------------------------------------------------------------- S3 Shadow Tether

    fn tether(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        let targets: Vec<Unit> = Self::targets_near(f, f.pos, d::TETHER_RADIUS).into_iter().take(d::TETHER_MAX.max(1) as usize).collect();
        if targets.is_empty() {
            return self.reject("s3: nobody in reach");
        }
        let until = f.tick + t(d::TETHER_TICKS);
        self.cd[2] = f.tick + t(d::TETHER_COOLDOWN_TICKS);
        self.window[2] = until;
        self.used[2] = false;
        self.tethers = targets.iter().map(|u| (u.id, until)).collect();
        for u in &targets {
            out.push(Effect::Fx { name: "ct_echo".into(), at: u.pos, ticks: 12, heading: None });
        }
    }

    /// S3 recast: every tethered enemy is yanked toward him and its shadow uppercuts it.
    fn yank(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        self.used[2] = true;
        self.window[2] = 0;
        let tethered: Vec<Unit> = self.tethers.iter().filter_map(|(id, _)| f.unit(*id).filter(|u| f.skill_target(u)).cloned()).collect();
        self.tethers.clear();
        for u in tethered {
            let gap = u.pos.dist(f.pos) as i64;
            let len = d::TETHER_YANK.min((gap - 10_000).max(0));
            if len > 0 {
                let to = f.clear_path(u.pos, u.pos.toward(f.pos, len));
                out.push(Effect::Knock { target: u.id, to, ticks: t(d::TETHER_YANK_TICKS) });
            }
            self.hit(f, &u, pct(d::TETHER_BASE, d::TETHER_AD_RATIO, f.attack), Some(Cc::KnockUp(t(d::TETHER_KNOCK_TICKS))), Src::Recast, true, out);
            out.push(Effect::Fx { name: "ct_uppercut".into(), at: u.pos, ticks: 20, heading: None });
        }
    }

    // -------------------------------------------------------------- Ult Shadow Flurry

    fn vanish(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        let Some(first) = Self::targets_near(f, f.pos, d::ULT_START_RANGE).first().cloned() else {
            return self.reject("ult: nothing in reach");
        };
        // cast while merged: the shadow's echoes ride the whole flurry
        let echo = self.merged(f.tick);
        self.bank = 0;
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
            // he comes back with his dodges full
            self.charges = self.charges.max(d::CHARGES_MAX);
            self.charge_at = 0;
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
    fn outnumbered(f: &Frame, p: Point) -> bool {
        let enemies = f.enemy_champions().filter(|u| u.pos.within(p, 70_000)).count() as i64;
        let allies = f.allies().filter(|a| a.pos.within(p, 90_000)).count() as i64 + 1;
        enemies >= allies + 2
    }

    /// The champion to fight now: nearer, lower, alone better; never under its tower unless low.
    pub fn pick(&self, f: &Frame) -> Option<Unit> {
        f.enemy_champions()
            .filter(|u| u.pos.within(f.pos, d::AI_ENGAGE_RANGE) && !Self::outnumbered(f, u.pos))
            .filter(|u| f.enemy_tower_at(u.pos).is_none() || Self::hurt_pct(u) < d::AI_COMMIT_HP_PCT)
            .min_by_key(|u| (Self::hurt_pct(u) + (u.pos.dist(f.pos) as i64 / 2_000), u.id))
            .cloned()
    }

    fn steer(&mut self, f: &Frame, to: Point, out: &mut Vec<Effect>) {
        if f.tick < self.steer_at {
            return;
        }
        self.steer_at = f.tick + 6;
        out.push(Effect::Steer { to });
    }

    pub fn decide(&mut self, f: &Frame, out: &mut Vec<Effect>) -> Option<Order> {
        if f.stunned || self.flurry.is_some() {
            return None;
        }
        let me_pct = f.hp * 100 / f.max_hp.max(1);
        // recasts first: the yank when the tethered group is spread or low, or the tether is about to lapse
        if self.recast_open(2, f) && !self.tethers.is_empty() {
            let tethered: Vec<&Unit> = self.tethers.iter().filter_map(|(id, _)| f.unit(*id)).collect();
            let spread = tethered.iter().any(|u| !u.pos.within(f.pos, 35_000));
            let low = tethered.iter().any(|u| u.champion && Self::hurt_pct(u) < d::AI_COMMIT_HP_PCT);
            let lapsing = f.tick + 30 >= self.window[2];
            if (spread && tethered.iter().any(|u| u.champion)) || low || (lapsing && tethered.len() >= 2) {
                self.reason = "S3 recast: yank".into();
                return Some(Order::S3 { aim: f.pos });
            }
        }
        // stepping out of the merge: an escape when hurt and pressed
        if self.merged(f.tick) && self.recast_open(0, f) && me_pct < 30 && f.enemy_champions().any(|u| u.pos.within(f.pos, 25_000)) {
            self.reason = "S1 recast: step out".into();
            let threat = f.enemy_champions().min_by_key(|u| (u.pos.d2(f.pos), u.id)).map_or(f.home, |u| u.pos);
            return Some(Order::S1 { aim: threat });
        }
        // the self-cleave onto a champion (or a pack while farming with charges full)
        if self.recast_open(1, f) {
            if let Some(u) = f.enemy_champions().filter(|u| u.pos.within(f.pos, d::CYCLONE_RECAST_RANGE)).min_by_key(|u| (Self::hurt_pct(u), u.id)) {
                self.reason = "S2 recast: cleave on them".into();
                return Some(Order::S2 { aim: u.pos });
            }
        }
        // the fight
        if let Some(u) = self.pick(f) {
            let dist = u.pos.dist(f.pos) as i64;
            if self.ult_ready(f) && u.pos.within(f.pos, d::ULT_START_RANGE) {
                let group = f.enemy_champions().filter(|v| v.pos.within(u.pos, d::ULT_CHAIN_RANGE)).count();
                let tethered = self.tethers.len() >= 2;
                if group >= 2 || tethered || Self::hurt_pct(&u) < d::AI_COMMIT_HP_PCT || self.merged(f.tick) {
                    self.reason = "Shadow Flurry".into();
                    return Some(Order::Ult);
                }
            }
            if !self.merged(f.tick) && self.ready(0, f) && dist <= d::AI_MERGE_RANGE {
                self.reason = "S1: merge".into();
                return Some(Order::S1 { aim: u.pos });
            }
            if self.ready(2, f) && Self::targets_near(f, f.pos, d::TETHER_RADIUS).len() >= 2 {
                self.reason = "S3: tether".into();
                return Some(Order::S3 { aim: u.pos });
            }
            if self.ready(1, f) && dist <= d::CYCLONE_RANGE + d::CYCLONE_RADIUS / 2 {
                self.reason = "S2: Shadow Cyclone".into();
                return Some(Order::S2 { aim: u.pos });
            }
            if !u.pos.within(f.pos, d::BASIC_RANGE) {
                self.steer(f, u.pos, out);
            }
            return None;
        }
        // farming (calm): his skills work on creeps and bank stacks for the ult
        if f.enemy_champions().all(|u| !u.pos.within(f.pos, d::AI_CALM_RANGE)) {
            let farm: Vec<Unit> = f.units.iter().filter(|u| u.farmable && f.skill_target(u) && u.pos.within(f.pos, 90_000)).cloned().collect();
            let big = |u: &Unit| u.max_hp >= d::AI_FARM_BIG_HP;
            if let Some(m) = farm.iter().min_by_key(|u| (u.pos.d2(f.pos), u.id)).cloned() {
                let around = farm.iter().filter(|v| v.pos.within(m.pos, d::CYCLONE_RADIUS)).count();
                let dist = m.pos.dist(f.pos) as i64;
                if self.recast_open(1, f) && (around >= 3 || big(&m)) {
                    self.reason = "Farm: S2 recast".into();
                    return Some(Order::S2 { aim: m.pos });
                }
                if self.ready(1, f) && dist <= d::CYCLONE_RANGE && (around >= 2 || big(&m)) {
                    self.reason = "Farm: Shadow Cyclone".into();
                    return Some(Order::S2 { aim: m.pos });
                }
                if self.ready(2, f) && farm.iter().filter(|v| v.pos.within(f.pos, d::TETHER_RADIUS)).count() >= 2 {
                    self.reason = "Farm: tether".into();
                    return Some(Order::S3 { aim: m.pos });
                }
                if self.ult_ready(f) && m.max_hp >= 2 * d::AI_FARM_BIG_HP && m.pos.within(f.pos, d::ULT_START_RANGE) {
                    self.reason = "Farm: Shadow Flurry on a big camp".into();
                    return Some(Order::Ult);
                }
            }
        }
        self.reason.clear();
        None
    }
}
