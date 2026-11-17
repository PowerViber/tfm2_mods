//! Contender 2.0, the Shadow Boxer: a deterministic core (snapshot in, effects out). `contender.rs` builds the frame
//! from the sim and applies the effects; the Lab (`native/contender_shadow_lab`) runs this same file.
//!
//!   P1 Shadows: up to 5 shadow charges (one every 6 s; each basic attack takes 0.5 s off). For 3 s after S2 or S3
//!      (and while S1's projection lasts), casting it again spends a charge: his shadow recasts it, stronger. Every hit
//!      of his, on anything, takes 1 s off the ult's cooldown.
//!   P2 Shadow Pact: every punch and skill hits again 1 s later for 40%, its crowd control halved (at most 1.5 s more per
//!      target every 6 s). Recast hits, and every hit while projected, echo twice. The ult's punches echo only when
//!      primed (S2 cast from the projected shadow).
//!   P3 Shadow Rhythm: each hit stacks on its target (10 max, 4 s). The most-stacked enemy gives him attack speed,
//!      Attack and move speed per stack and takes more damage from him per stack. A max-stacked creep carries 4 stacks
//!      onto the next champion he hits; a stacked kill keeps the buffs 5 s longer.
//!   S1 Shadow Projection: his body holds still and can't be hurt; his shadow is launched up to 100,000 and fights as
//!      him for 5 s (S2 and S3 come from it). 70% of the damage it takes reaches him; if it falls, that's 70% of his
//!      max HP. Recast: his body swaps to it and the shadow's cross knocks the nearest enemy back.
//!   S2 Shadow Cyclone: a shadow clone cleaves a circle at a spot within 60,000 (4 hits, a slow; it can't be hit).
//!      Recast: he goes there and cleaves 7 hits, invulnerable, the last knocking up.
//!   S3 Slam & Uppercut: grab and slam (a stun, an impact ring). Recast: an uppercut; from the projection it launches
//!      the target toward his body.
//!   Ult Shadow Flurry: 5 s unseen and unhurt, charges refilled; 36 accelerating shadow punches chained from enemy to
//!      enemy within 30,000 of the last hit, a knock-up every second, an early end when nothing is in reach; he
//!      reappears beside the last enemy hit with a haymaker ready.
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

/// His projected shadow as the host sees it (a carrier unit).
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ShadowView {
    pub id: usize,
    pub pos: Point,
    pub hp: i64,
    pub max_hp: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Order {
    /// S1: launch the shadow toward the point; recast (while projected): swap his body to it
    S1 { aim: Point },
    /// S2: the clone's cleave at the point; recast: he cleaves there himself
    S2 { aim: Point },
    /// S3: slam the enemy nearest the point; recast: the uppercut
    S3 { aim: Point },
    Ult,
    /// steer the projected shadow toward the point
    Move { aim: Point },
    /// let the projection go now
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
    /// his projected shadow, while it stands
    #[serde(default)]
    pub shadow: Option<ShadowView>,
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
    /// a deliberate move of his own (an engine glide)
    Glide { to: Point, ticks: u64 },
    /// set him there at once (the projection swap, the ult's reappearance)
    Teleport { to: Point },
    /// his body during a projection: rooted, unhurt, no basic attacks (a short lease, renewed)
    Hold { ticks: u64 },
    /// the ult: unseen, unhurt, rooted and CC immune (a short lease, renewed)
    Vanish { ticks: u64 },
    /// S2's self-cleave: unhurt
    Invulnerable { ticks: u64 },
    /// the projected shadow: a carrier unit with his max HP
    SpawnShadow { at: Point, hp: i64, ticks: u64 },
    MoveShadow { to: Point, ticks: u64 },
    DespawnShadow,
    /// the 70% relay of the shadow's damage (raw, to himself)
    SelfDamage { amount: i64 },
    /// Shadow Rhythm's buffs on himself
    Rhythm { attack_speed: i64, attack_permille: i64, move_permille: i64, ticks: u64 },
    /// his shadow charges (for the pips)
    Charges { count: i64 },
    /// a purposeful walk the shared input AI turns into a move order
    Steer { to: Point },
    Fx { name: String, at: Point, ticks: u64 },
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

/// Where a hit came from (it decides the echoes).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Src {
    Basic,
    Skill,
    Recast,
    Echo,
    Flurry,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Echo {
    pub target: usize,
    pub at: u64,
    pub amount: i64,
    pub cc: Option<Cc>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Projection {
    pub until: u64,
    /// the shadow's position (its launch target until the host reports it)
    pub pos: Point,
    pub shadow_hp: i64,
    pub spawned: bool,
    pub next_basic: u64,
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
    pub charges: i64,
    pub charge_at: u64,
    /// S1-S3 ready ticks
    pub cd: [u64; 3],
    pub ult_at: u64,
    /// S2 / S3 recast windows (end ticks)
    pub window: [u64; 3],
    pub proj: Option<Projection>,
    pub cleaves: Vec<Cleave>,
    pub cyclone_spot: Option<Point>,
    pub slammed: Option<usize>,
    pub primed_until: u64,
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
            cd: [0; 3],
            ult_at: 0,
            window: [0; 3],
            proj: None,
            cleaves: vec![],
            cyclone_spot: None,
            slammed: None,
            primed_until: 0,
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
        self.proj = None;
        self.cleaves.clear();
        self.cyclone_spot = None;
        self.slammed = None;
        self.flurry = None;
        self.echoes.clear();
        self.window = [0; 3];
        self.haymaker_until = 0;
        self.linger = None;
        self.focus = None;
    }
    pub fn projecting(&self) -> bool {
        self.proj.is_some()
    }
    pub fn flurrying(&self) -> bool {
        self.flurry.is_some()
    }
    pub fn ult_ready(&self, f: &Frame) -> bool {
        f.ult_unlocked && f.tick >= self.ult_at && self.flurry.is_none()
    }
    pub fn ready(&self, slot: usize, f: &Frame) -> bool {
        f.tick >= self.cd[slot]
    }
    /// S2/S3: a recast is open (S1's is the projection itself)
    pub fn recast_open(&self, slot: usize, f: &Frame) -> bool {
        match slot {
            0 => self.proj.is_some(),
            _ => f.tick < self.window[slot],
        }
    }
    /// where his skills come from: the projected shadow, else him
    pub fn origin(&self, f: &Frame) -> Point {
        self.proj.map_or(f.pos, |p| p.pos)
    }
    pub fn stacks_on(&self, id: usize, tick: u64) -> i64 {
        self.stacks.iter().find(|s| s.id == id && tick < s.until).map_or(0, |s| s.count)
    }
    /// extra damage he deals to `id`, in permille (Shadow Rhythm's vulnerability)
    pub fn vuln_permille(&self, id: usize, tick: u64) -> i64 {
        self.stacks_on(id, tick) * d::RHYTHM_VULNERABILITY_PERMILLE_PER_STACK
    }

    // -------------------------------------------------------------- step

    pub fn step(&mut self, f: &Frame) -> Vec<Effect> {
        let mut out = vec![];
        self.rejection.clear();
        if !f.alive {
            if self.proj.is_some() {
                out.push(Effect::DespawnShadow);
            }
            self.respawn();
            return out;
        }
        self.recharge(f);
        self.stacks.retain(|s| f.tick < s.until);
        self.cc_budget.retain(|b| f.tick < b.1 + t(d::PACT_CC_WINDOW_TICKS));
        if self.cyclone_spot.is_some() && f.tick >= self.window[1] {
            self.cyclone_spot = None;
        }
        if self.slammed.is_some() && f.tick >= self.window[2] {
            self.slammed = None;
        }
        for id in f.basic_hits.clone() {
            if let Some(u) = f.unit(id).filter(|u| f.skill_target(u) || (u.team != f.team && !u.tower)).cloned() {
                self.basic_landed(f, &u, &mut out);
            }
        }
        self.run_projection(f, &mut out);
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
            self.charge_at = f.tick + t(d::CHARGES_RECHARGE_TICKS);
        }
        if f.tick >= self.charge_at {
            self.charges += 1;
            self.charge_at = if self.charges < d::CHARGES_MAX { f.tick + t(d::CHARGES_RECHARGE_TICKS) } else { 0 };
        }
    }
    fn spend(&mut self, f: &Frame) -> bool {
        if self.charges <= 0 {
            return false;
        }
        if self.charges == d::CHARGES_MAX {
            self.charge_at = f.tick + t(d::CHARGES_RECHARGE_TICKS);
        }
        self.charges -= 1;
        true
    }
    fn reject(&mut self, why: &str) {
        self.rejection = why.into();
    }

    // -------------------------------------------------------------- hits, echoes, stacks

    /// One hit of his (or his shadows'). `apply`: deal the damage here (basic attacks are dealt by the engine).
    fn hit(&mut self, f: &Frame, u: &Unit, base: i64, cc: Option<Cc>, src: Src, apply: bool, out: &mut Vec<Effect>) {
        let amount = base * (1_000 + self.vuln_permille(u.id, f.tick)) / 1_000;
        if apply && amount > 0 {
            out.push(Effect::Damage { target: u.id, amount });
        }
        if let Some(c) = cc {
            self.cc(f, u.id, c, src == Src::Echo, out);
        }
        self.add_stack(f, u);
        self.ult_at = self.ult_at.saturating_sub(t(d::ULT_CUT_PER_HIT_TICKS));
        let echoes = match src {
            Src::Echo => 0,
            Src::Flurry => i64::from(f.tick < self.primed_until),
            Src::Recast => 2,
            _ if self.proj.is_some() => 2,
            _ => 1,
        };
        for k in 1..=echoes {
            self.echoes.push(Echo {
                target: u.id,
                at: f.tick + t(d::PACT_DELAY_TICKS) * k as u64,
                amount: base * d::PACT_PCT / 100,
                cc: cc.map(Cc::halved),
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

    fn add_stack(&mut self, f: &Frame, u: &Unit) {
        let max = d::RHYTHM_MAX_STACKS;
        let carry = u.champion && self.stacks.iter().any(|s| !s.champion && s.count >= max && f.tick < s.until);
        let until = f.tick + t(d::RHYTHM_STACK_TICKS);
        match self.stacks.iter_mut().find(|s| s.id == u.id) {
            Some(s) => {
                s.count = (s.count + 1).min(max);
                s.until = until;
            }
            None => self.stacks.push(Stack { id: u.id, count: if carry { d::RHYTHM_CARRY_STACKS } else { 1 }, until, champion: u.champion }),
        }
    }

    fn run_echoes(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        let due: Vec<Echo> = self.echoes.iter().filter(|e| f.tick >= e.at).copied().collect();
        self.echoes.retain(|e| f.tick < e.at);
        for e in due {
            if let Some(u) = f.unit(e.target).filter(|u| f.skill_target(u)).cloned() {
                self.hit(f, &u, e.amount, e.cc, Src::Echo, true, out);
                out.push(Effect::Fx { name: "ct_echo".into(), at: u.pos, ticks: 12 });
            }
        }
    }

    /// The engine's basic attack (his body's) landed.
    fn basic_landed(&mut self, f: &Frame, u: &Unit, out: &mut Vec<Effect>) {
        self.hit(f, u, 0, None, Src::Basic, false, out);
        // the echo of a basic is 40% of his Attack
        if let Some(e) = self.echoes.iter_mut().rev().find(|e| e.target == u.id && e.amount == 0) {
            e.amount = f.attack * d::PACT_PCT / 100;
        }
        self.charge_cut();
        if f.tick < self.haymaker_until {
            self.haymaker_until = 0;
            let amount = pct(d::ULT_HAYMAKER_BASE, d::ULT_HAYMAKER_AD_RATIO, f.attack);
            self.hit(f, u, amount, Some(Cc::Stun(t(d::ULT_HAYMAKER_STUN_TICKS))), Src::Skill, true, out);
            out.push(Effect::Fx { name: "ct_haymaker".into(), at: u.pos, ticks: 18 });
        }
    }
    fn charge_cut(&mut self) {
        if self.charges < d::CHARGES_MAX && self.charge_at > 0 {
            self.charge_at = self.charge_at.saturating_sub(t(d::CHARGES_BASIC_CUT_TICKS));
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
                attack_speed: n * d::RHYTHM_ATTACK_SPEED_PCT_PER_STACK,
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
                if self.proj.is_some() {
                    if !self.spend(f) {
                        return self.reject("s1 recast: no charge");
                    }
                    return self.swap(f, out);
                }
                if !self.ready(0, f) {
                    return self.reject("s1");
                }
                self.project(f, aim, out)
            }
            Order::S2 { aim } => {
                if self.recast_open(1, f) && self.cyclone_spot.is_some() {
                    if !self.spend(f) {
                        return self.reject("s2 recast: no charge");
                    }
                    return self.cyclone_recast(f, out);
                }
                if !self.ready(1, f) {
                    return self.reject("s2");
                }
                self.cyclone(f, aim, out)
            }
            Order::S3 { aim } => {
                if self.recast_open(2, f) && self.slammed.is_some() {
                    if !self.spend(f) {
                        return self.reject("s3 recast: no charge");
                    }
                    return self.uppercut(f, out);
                }
                if !self.ready(2, f) {
                    return self.reject("s3");
                }
                self.slam(f, aim, out)
            }
            Order::Ult => {
                if !self.ult_ready(f) {
                    return self.reject("ult");
                }
                self.vanish(f, out)
            }
            Order::Move { aim } => {
                let Some(p) = self.proj else { return self.reject("move: not projected") };
                let step = (d::PROJECTION_SHADOW_SPEED * 6).max(1);
                let to = f.clear_path(p.pos, p.pos.toward(aim, step));
                if to != p.pos {
                    out.push(Effect::MoveShadow { to, ticks: 6 });
                    if let Some(p) = self.proj.as_mut() {
                        p.pos = to;
                    }
                }
            }
            Order::End => {
                if self.proj.is_some() {
                    self.end_projection(f, out);
                }
            }
        }
    }

    // -------------------------------------------------------------- S1 Shadow Projection

    fn project(&mut self, f: &Frame, aim: Point, out: &mut Vec<Effect>) {
        let to = f.nearest_ground(f.clear_path(f.pos, f.pos.toward(aim, d::PROJECTION_RANGE)));
        self.cd[0] = f.tick + t(d::PROJECTION_COOLDOWN_TICKS);
        let ticks = t(d::PROJECTION_TICKS);
        out.push(Effect::SpawnShadow { at: f.pos, hp: f.max_hp, ticks: ticks + 30 });
        let launch = ((f.pos.dist(to) as i64 / d::PROJECTION_LAUNCH_SPEED.max(1)) as u64).max(1);
        out.push(Effect::MoveShadow { to, ticks: launch });
        out.push(Effect::Hold { ticks: 3 });
        out.push(Effect::Fx { name: "ct_launch".into(), at: f.pos, ticks: 18 });
        self.proj = Some(Projection { until: f.tick + ticks, pos: to, shadow_hp: f.max_hp, spawned: false, next_basic: f.tick + launch });
    }

    fn run_projection(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        let Some(mut p) = self.proj else { return };
        if f.tick >= p.until {
            return self.end_projection(f, out);
        }
        match f.shadow {
            Some(s) => {
                if p.spawned && s.hp < p.shadow_hp {
                    out.push(Effect::SelfDamage { amount: (p.shadow_hp - s.hp) * d::PROJECTION_RELAY_PCT / 100 });
                }
                p.spawned = true;
                p.shadow_hp = s.hp;
                p.pos = s.pos;
            }
            None if p.spawned => {
                // it fell: the rest of its HP comes through the relay
                out.push(Effect::SelfDamage { amount: p.shadow_hp.max(0) * d::PROJECTION_RELAY_PCT / 100 });
                out.push(Effect::Fx { name: "ct_appear".into(), at: f.pos, ticks: 18 });
                self.proj = None;
                return;
            }
            None => {}
        }
        out.push(Effect::Hold { ticks: 3 });
        // the shadow boxes like him: basic attacks on the nearest enemy in reach (they echo twice)
        if f.tick >= p.next_basic {
            if let Some(u) = Self::targets_near(f, p.pos, d::BASIC_RANGE).first().cloned() {
                let speed = 100 + self.stacks_rhythm(f) * d::RHYTHM_ATTACK_SPEED_PCT_PER_STACK;
                p.next_basic = f.tick + (t(d::BASIC_COOLDOWN_TICKS) * 100 / speed.max(1) as u64).max(10);
                self.proj = Some(p);
                self.hit(f, &u, f.attack, None, Src::Basic, true, out);
                self.charge_cut();
                out.push(Effect::Fx { name: "ct_cross".into(), at: u.pos, ticks: 10 });
                return;
            }
        }
        self.proj = Some(p);
    }
    fn stacks_rhythm(&self, f: &Frame) -> i64 {
        self.focus.map_or(0, |x| x.1).max(self.linger.filter(|l| f.tick < l.1).map_or(0, |l| l.0))
    }

    fn end_projection(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        if let Some(p) = self.proj.take() {
            out.push(Effect::DespawnShadow);
            out.push(Effect::Fx { name: "ct_appear".into(), at: p.pos, ticks: 12 });
            let _ = f;
        }
    }

    /// S1 recast: his body swaps to the shadow; the shadow, now where he stood, throws a cross.
    fn swap(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        let Some(p) = self.proj.take() else { return };
        let to = f.nearest_ground(p.pos);
        out.push(Effect::DespawnShadow);
        out.push(Effect::Invulnerable { ticks: 15 });
        out.push(Effect::Teleport { to });
        out.push(Effect::Fx { name: "ct_swap".into(), at: to, ticks: 18 });
        out.push(Effect::Fx { name: "ct_swap".into(), at: f.pos, ticks: 18 });
        if let Some(u) = Self::targets_near(f, f.pos, d::PROJECTION_CROSS_RADIUS).first().cloned() {
            let dir = unit_dir(f.pos, u.pos);
            let knock = f.clear_path(u.pos, u.pos.along(dir, d::PROJECTION_CROSS_KNOCK));
            self.hit(f, &u, pct(d::PROJECTION_CROSS_BASE, d::PROJECTION_CROSS_AD_RATIO, f.attack), None, Src::Recast, true, out);
            out.push(Effect::Knock { target: u.id, to: knock, ticks: t(d::PROJECTION_CROSS_KNOCK_TICKS) });
            out.push(Effect::Fx { name: "ct_cross".into(), at: u.pos, ticks: 18 });
        }
    }

    // -------------------------------------------------------------- S2 Shadow Cyclone

    fn cyclone(&mut self, f: &Frame, aim: Point, out: &mut Vec<Effect>) {
        let o = self.origin(f);
        let spot = f.nearest_ground(f.clear_path(o, o.toward(aim, d::CYCLONE_RANGE)));
        self.cd[1] = f.tick + t(d::CYCLONE_COOLDOWN_TICKS);
        self.window[1] = f.tick + t(d::CHARGES_RECAST_WINDOW_TICKS);
        self.cyclone_spot = Some(spot);
        if self.proj.is_some() {
            self.primed_until = f.tick + t(d::CYCLONE_PRIME_TICKS);
        }
        self.cleaves.push(Cleave {
            at: spot,
            next: f.tick + t(d::CYCLONE_EVERY_TICKS),
            left: d::CYCLONE_HITS,
            every: t(d::CYCLONE_EVERY_TICKS),
            base: d::CYCLONE_BASE,
            ratio: d::CYCLONE_AD_RATIO,
            recast: false,
        });
        out.push(Effect::Fx { name: "ct_shadow".into(), at: spot, ticks: t(d::CYCLONE_HITS * d::CYCLONE_EVERY_TICKS) + 6 });
    }

    fn cyclone_recast(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        let Some(spot) = self.cyclone_spot.take() else { return };
        self.window[1] = 0;
        let span = t(d::CYCLONE_RECAST_HITS * d::CYCLONE_RECAST_EVERY_TICKS);
        if let Some(p) = self.proj.as_mut() {
            p.pos = spot;
            out.push(Effect::MoveShadow { to: spot, ticks: 8 });
        } else {
            out.push(Effect::Glide { to: spot, ticks: 8 });
            out.push(Effect::Invulnerable { ticks: span + 12 });
        }
        self.cleaves.push(Cleave {
            at: spot,
            next: f.tick + 8 + t(d::CYCLONE_RECAST_EVERY_TICKS),
            left: d::CYCLONE_RECAST_HITS,
            every: t(d::CYCLONE_RECAST_EVERY_TICKS),
            base: d::CYCLONE_RECAST_BASE,
            ratio: d::CYCLONE_RECAST_AD_RATIO,
            recast: true,
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
            out.push(Effect::Fx { name: "ct_cleave".into(), at: c.at, ticks: c.every + 2 });
            for u in Self::targets_near(f, c.at, d::CYCLONE_RADIUS) {
                let cc = if c.recast {
                    last.then_some(Cc::KnockUp(t(d::CYCLONE_RECAST_KNOCK_TICKS)))
                } else {
                    Some(Cc::Slow(d::CYCLONE_SLOW_PCT, t(d::CYCLONE_SLOW_TICKS)))
                };
                self.hit(f, &u, pct(c.base, c.ratio, f.attack), cc, if c.recast { Src::Recast } else { Src::Skill }, true, out);
            }
        }
        cleaves.retain(|c| c.left > 0);
        self.cleaves.extend(cleaves);
    }

    // -------------------------------------------------------------- S3 Slam & Uppercut

    fn slam(&mut self, f: &Frame, aim: Point, out: &mut Vec<Effect>) {
        let o = self.origin(f);
        let Some(u) = f
            .units
            .iter()
            .filter(|u| f.skill_target(u) && u.pos.within(o, d::SLAM_GRAB + u.radius))
            .min_by_key(|u| (!u.champion, u.pos.d2(aim), u.id))
            .cloned()
        else {
            return self.reject("s3: nothing in reach");
        };
        self.cd[2] = f.tick + t(d::SLAM_COOLDOWN_TICKS);
        self.window[2] = f.tick + t(d::CHARGES_RECAST_WINDOW_TICKS);
        self.slammed = Some(u.id);
        self.hit(f, &u, pct(d::SLAM_BASE, d::SLAM_AD_RATIO, f.attack), Some(Cc::Stun(t(d::SLAM_STUN_TICKS))), Src::Skill, true, out);
        out.push(Effect::Fx { name: "ct_slam".into(), at: u.pos, ticks: 20 });
        let ring = pct(d::SLAM_BASE, d::SLAM_AD_RATIO, f.attack) * d::SLAM_RING_PCT / 100;
        for v in Self::targets_near(f, u.pos, d::SLAM_RING_RADIUS).into_iter().filter(|v| v.id != u.id) {
            self.hit(f, &v, ring, None, Src::Skill, true, out);
        }
    }

    fn uppercut(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        let Some(id) = self.slammed.take() else { return };
        self.window[2] = 0;
        let o = self.origin(f);
        let Some(u) = f.unit(id).filter(|u| f.skill_target(u) && u.pos.within(o, d::SLAM_GRAB * 2 + u.radius)).cloned() else {
            return self.reject("s3 recast: target gone");
        };
        self.hit(f, &u, pct(d::SLAM_UPPERCUT_BASE, d::SLAM_UPPERCUT_AD_RATIO, f.attack), Some(Cc::KnockUp(t(d::SLAM_KNOCK_TICKS))), Src::Recast, true, out);
        out.push(Effect::Fx { name: "ct_uppercut".into(), at: u.pos, ticks: 20 });
        if self.proj.is_some() {
            // launched toward his rooted body, landing just short of it, never through terrain
            let gap = u.pos.dist(f.pos) as i64;
            let len = (gap - 12_000).clamp(0, d::SLAM_LAUNCH);
            if len > 0 {
                let to = f.clear_path(u.pos, u.pos.toward(f.pos, len));
                out.push(Effect::Knock { target: u.id, to, ticks: t(d::SLAM_LAUNCH_TICKS) });
            }
        }
    }

    // -------------------------------------------------------------- Ult Shadow Flurry

    fn vanish(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        let Some(first) = Self::targets_near(f, f.pos, d::ULT_START_RANGE).first().cloned() else {
            return self.reject("ult: nothing in reach");
        };
        if self.proj.is_some() {
            self.end_projection(f, out);
        }
        self.ult_at = f.tick + t(d::ULT_COOLDOWN_TICKS);
        self.charges = d::CHARGES_MAX;
        self.charge_at = 0;
        self.flurry = Some(Flurry { start: f.tick, next: f.tick + 6, done: 0, first: first.id, last: None, knock_next: f.tick + t(d::ULT_KNOCK_EVERY_TICKS), vanished_at: f.pos });
        out.push(Effect::Vanish { ticks: 3 });
        out.push(Effect::Fx { name: "ct_vanish".into(), at: f.pos, ticks: 24 });
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
                Some((_, at)) => Self::targets_near(f, at, d::ULT_CHAIN_RANGE).into_iter().next(),
            };
            let Some(u) = pick else {
                over = true;
                break;
            };
            self.flurry = Some(fl);
            self.hit(f, &u, pct(d::ULT_BASE, d::ULT_AD_RATIO, f.attack), None, Src::Flurry, true, out);
            fl = self.flurry.unwrap_or(fl);
            out.push(Effect::Fx { name: "ct_flurry".into(), at: u.pos, ticks: 8 });
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
                out.push(Effect::Fx { name: "ct_appear".into(), at: to, ticks: 18 });
            }
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
        let o = self.origin(f);
        f.enemy_champions()
            .filter(|u| u.pos.within(o, d::AI_ENGAGE_RANGE) && !Self::outnumbered(f, u.pos))
            .filter(|u| f.enemy_tower_at(u.pos).is_none() || Self::hurt_pct(u) < d::AI_COMMIT_HP_PCT)
            .min_by_key(|u| (Self::hurt_pct(u) + (u.pos.dist(o) as i64 / 2_000), u.id))
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
        let o = self.origin(f);
        // recasts first: the uppercut on a slammed target
        if let Some(id) = self.slammed.filter(|_| self.recast_open(2, f) && self.charges > 0) {
            if let Some(u) = f.unit(id).filter(|u| f.skill_target(u)) {
                if u.champion || self.charges >= d::CHARGES_MAX {
                    self.reason = "S3 recast: uppercut".into();
                    return Some(Order::S3 { aim: u.pos });
                }
            }
        }
        // the projection: lapse when the shadow is losing, commit (swap in) on a low target or with help there
        if let Some(p) = self.proj {
            let shadow_pct = f.shadow.map_or(100, |s| s.hp * 100 / s.max_hp.max(1));
            if shadow_pct < d::AI_LAPSE_SHADOW_PCT {
                self.reason = "Projection lapses (the shadow is losing)".into();
                return Some(Order::End);
            }
            let near = f.enemy_champions().filter(|u| u.pos.within(p.pos, 40_000)).min_by_key(|u| (Self::hurt_pct(u), u.id)).cloned();
            if let Some(u) = &near {
                let help = f.allies().any(|a| a.pos.within(p.pos, 60_000));
                if self.charges > 0 && (Self::hurt_pct(u) < d::AI_COMMIT_HP_PCT || help) && !Self::outnumbered(f, p.pos) {
                    self.reason = "S1 recast: swap in".into();
                    return Some(Order::S1 { aim: p.pos });
                }
            }
        }
        // the self-cleave: on a champion in the spot, or a wave with charges to spare
        if let Some(spot) = self.cyclone_spot.filter(|_| self.recast_open(1, f) && self.charges > 0) {
            let there = Self::targets_near(f, spot, d::CYCLONE_RADIUS);
            if there.iter().any(|u| u.champion) || (there.len() >= 3 && self.charges >= d::CHARGES_MAX) {
                self.reason = "S2 recast: cleave there".into();
                return Some(Order::S2 { aim: spot });
            }
        }
        // the fight
        if let Some(u) = self.pick(f) {
            let dist = u.pos.dist(o) as i64;
            if self.ult_ready(f) && u.pos.within(f.pos, d::ULT_START_RANGE) {
                let group = f.enemy_champions().filter(|v| v.pos.within(u.pos, d::ULT_CHAIN_RANGE)).count();
                if group >= 2 || Self::hurt_pct(&u) < d::AI_COMMIT_HP_PCT || f.tick < self.primed_until {
                    self.reason = "Shadow Flurry".into();
                    return Some(Order::Ult);
                }
            }
            if self.proj.is_none() && self.ready(0, f) && dist >= 40_000 && dist <= d::PROJECTION_RANGE + 20_000 {
                self.reason = "S1: project onto the target".into();
                return Some(Order::S1 { aim: u.pos });
            }
            if self.ready(1, f) && dist <= d::CYCLONE_RANGE + d::CYCLONE_RADIUS / 2 {
                self.reason = "S2: Shadow Cyclone".into();
                return Some(Order::S2 { aim: u.pos });
            }
            if self.ready(2, f) && dist <= d::SLAM_GRAB + u.radius {
                self.reason = "S3: Slam".into();
                return Some(Order::S3 { aim: u.pos });
            }
            if self.proj.is_some() {
                return Some(Order::Move { aim: u.pos });
            }
            if !u.pos.within(f.pos, d::BASIC_RANGE) {
                self.steer(f, u.pos, out);
            }
            return None;
        }
        // farming (calm): every skill works on creeps
        if f.enemy_champions().all(|u| !u.pos.within(f.pos, d::AI_CALM_RANGE)) {
            let farm: Vec<Unit> = f.units.iter().filter(|u| u.farmable && f.skill_target(u) && u.pos.within(o, 110_000)).cloned().collect();
            let big = |u: &Unit| u.max_hp >= d::AI_FARM_BIG_HP;
            if let Some(m) = farm.iter().min_by_key(|u| (u.pos.d2(o), u.id)).cloned() {
                let around = farm.iter().filter(|v| v.pos.within(m.pos, d::CYCLONE_RADIUS)).count();
                let dist = m.pos.dist(o) as i64;
                if self.ready(1, f) && dist <= d::CYCLONE_RANGE && (around >= 2 || big(&m)) {
                    self.reason = "Farm: Shadow Cyclone".into();
                    return Some(Order::S2 { aim: m.pos });
                }
                if self.ready(2, f) {
                    if let Some(b) = farm.iter().filter(|u| big(u) && u.pos.within(o, d::SLAM_GRAB + u.radius)).min_by_key(|u| (u.pos.d2(o), u.id)) {
                        self.reason = "Farm: Slam".into();
                        return Some(Order::S3 { aim: b.pos });
                    }
                }
                if self.proj.is_none() && self.ready(0, f) && big(&m) && self.charges >= d::CHARGES_MAX && dist >= 45_000 && dist <= d::PROJECTION_RANGE {
                    self.reason = "Farm: project onto the camp".into();
                    return Some(Order::S1 { aim: m.pos });
                }
                if self.ult_ready(f) && m.max_hp >= 2 * d::AI_FARM_BIG_HP && m.pos.within(f.pos, d::ULT_START_RANGE) {
                    self.reason = "Farm: Shadow Flurry on a big camp".into();
                    return Some(Order::Ult);
                }
                if self.proj.is_some() {
                    return Some(Order::Move { aim: m.pos });
                }
            }
        }
        self.reason.clear();
        None
    }
}
