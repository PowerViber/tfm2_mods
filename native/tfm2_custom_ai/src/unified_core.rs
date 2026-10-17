//! The Unified Theory: eighteen equation skills on five independently recharging charges.
//! Every skill is either cast or reserved; a reserved skill becomes a trait of the next cast,
//! so one cast carries up to four traits (a five-step combo). Pure and deterministic: integer
//! arithmetic, BTreeMaps and a private xorshift. The native passive and Equation Lab (WASM)
//! run this same code.
use crate::unified_eq_data::*;
use crate::unified_theory_math::{unit, Window};
use std::collections::BTreeMap;

pub const N: usize = 18;
pub const APPLE: usize = 0;
pub const PRISM: usize = 1;
pub const ORBIT: usize = 2;
pub const CRADLE: usize = 3;
pub const THIRD_LAW: usize = 4;
pub const CALCULUS: usize = 5;
pub const RADIUM: usize = 6;
pub const ALPHA: usize = 7;
pub const BETA: usize = 8;
pub const GAMMA: usize = 9;
pub const HALF_LIFE: usize = 10;
pub const CHAIN: usize = 11;
pub const TIME_FIELD: usize = 12;
pub const EMC2: usize = 13;
pub const RING: usize = 14;
pub const TWIN: usize = 15;
pub const WELL: usize = 16;
pub const PHOTO: usize = 17;
pub const RANKS: [&str; 8] = [
    "Student",
    "Lab Assistant",
    "Researcher",
    "Scientist",
    "Professor",
    "Fellow",
    "Laureate",
    "Unified Mind",
];

// What a rank changes. Never raw power: only how well the eighteen equations are used.
/// Longest combo the athlete plans (steps, including the cast).
const PLAN_STEPS: [usize; 8] = [1, 2, 2, 3, 4, 4, 5, 5];
/// Ticks between two steps of a planned combo. A step within INSIGHT_TICKS is an Insight.
const STEP_TICKS: [usize; 8] = [34, 30, 26, 22, 18, 14, 10, 8];
/// Fraction (per 100) of the target's motion the aim leads by.
const LEAD: [i64; 8] = [0, 0, 30, 50, 70, 85, 100, 100];
/// Noise (per 100) on every option's estimated value.
const NOISE: [i64; 8] = [45, 36, 28, 20, 13, 8, 4, 0];
/// Chance (per 100) to take a random legal option instead of the best one.
const MISTAKE: [u64; 8] = [30, 22, 15, 10, 6, 3, 1, 0];
/// Signatures the athlete knows: none, three-step ones, then all.
const KNOWS: [usize; 8] = [0, 0, 0, 0, 3, 3, 5, 5];
/// Final skills whose reserve combinations are searched in depth.
const FINALS: usize = 6;
/// Options the reserve search keeps at each depth.
const BEAM: [usize; 8] = [1, 1, 1, 2, 2, 3, 3, 4];
/// Charges kept back for a reaction (unless something urgent happens).
const KEEP: [usize; 8] = [0; 8];
/// Share of a reserved skill's own power it lends to the cast.
pub const FUSE_PCT: i64 = 80;
/// How well the athlete reads each equation's conditions (per 100). The rest of its estimate is naive:
/// a Student fires Photoelectric without marks, drops Apple point-blank and throws Orbit from afar.
const UNDERSTANDING: [i64; 8] = [0, 15, 35, 55, 70, 85, 95, 100];
/// Before the first combo of a fight the top ranks hold charges for a bigger play (value per charge below
/// this share of the best single is not worth spending on, unless urgent).
const HOLD: [i64; 8] = [0, 0, 0, 0, 60, 70, 80, 85];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fighter {
    pub id: usize,
    pub team: usize,
    pub x: i64,
    pub y: i64,
    pub hp: i64,
    pub max_hp: i64,
    pub ad: i64,
    pub ap: i64,
    pub champion: bool,
    pub visible: bool,
    pub targetable: bool,
    pub cc_immune: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Aim {
    Unit(usize),
    Point(i64, i64),
}

/// A manual step (Equation Lab, tests). The game drives Unified with `automatic`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Reserve(usize, Aim),
    Cast(usize, Aim),
    Ult(Aim),
}

pub struct Input<'a> {
    pub tick: usize,
    pub me: usize,
    pub fighters: &'a [Fighter],
    pub automatic: bool,
    pub action: Option<Action>,
    pub ult_unlocked: bool,
    pub stunned: bool,
    pub wall: &'a dyn Fn(i64, i64) -> bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Cmd {
    Damage { target: usize, amount: i64, kind: Kind },
    Heal { target: usize, amount: i64 },
    Shield { target: usize, amount: i64, ticks: usize },
    Root { target: usize, ticks: usize },
    /// Move speed change in percent (negative slows) for `ticks`.
    Speed { target: usize, pct: i64, ticks: usize },
    SetPos { target: usize, x: i64, y: i64 },
    /// A view effect `<champion>_<tag>` at a point, or following a unit.
    Fx { tag: String, x: i64, y: i64, unit: Option<usize>, ticks: usize },
    Log(String),
}

/// The traits a cast carries (each reserved skill once or twice) and its multiplier.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Mods {
    pub traits: [u8; N],
    /// Percent applied to every hit of the cast.
    pub mult: i64,
    pub sig: Option<usize>,
    /// Secondary bodies (lasers, drops, puddles, jumps) carry no traits, only the multiplier.
    pub secondary: bool,
}
impl Mods {
    fn plain(mult: i64) -> Mods {
        Mods { traits: [0; N], mult, sig: None, secondary: true }
    }
    pub fn has(&self, skill: usize) -> i64 {
        i64::from(self.traits[skill])
    }
    fn fx(&self, fx: Fx) -> Option<i64> {
        let s = &SIGNATURES[self.sig?];
        s.fx.iter().find(|(f, _)| *f == fx).map(|(_, v)| *v)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Body {
    Ray {
        x: i64,
        y: i64,
        dx: i64,
        dy: i64,
        remaining: i64,
        speed: i64,
        width: i64,
        pierce: usize,
        hits: Vec<usize>,
        walls: bool,
        turn: i64,
        bent: bool,
        stop: bool,
        split: bool,
        /// Where the first impact happened (a Split laser leaves from there).
        first: Option<(i64, i64)>,
        attenuation: i64,
    },
    Lob { x: i64, y: i64, land: usize, radius: i64, root: usize, frac: i64 },
    Planet { x: i64, y: i64, cx: i64, cy: i64, dx: i64, dy: i64, travel: i64, speed: i64, radius: i64, angle: usize, left: usize, power: i64, last: BTreeMap<usize, usize> },
    Zone { x: i64, y: i64, radius: i64, start: usize, until: usize, period: usize, next: usize, enriched: usize, reopen: bool, frac: i64 },
    Dot { target: usize, a: i64, n: usize, next: usize },
    /// Third Law: a shield, then the equal and opposite reaction.
    Reaction { until: usize, taken: i64 },
    /// E = mc² through a Time Field: the energy is released later.
    Delayed { at: usize, target: usize, amount: i64 },
    /// A knockback or pull applied over a few ticks (walls stop it).
    Shove { target: usize, dx: i64, dy: i64, left: usize },
    /// Rewind / repeat: the whole cast again from the same origin.
    Echo { at: usize, aim: Aim, ox: i64, oy: i64, pct: i64 },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Live {
    pub uid: u32,
    pub skill: usize,
    pub combo: u32,
    pub mods: Mods,
    pub body: Body,
    pub done: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Step {
    pub skill: usize,
    pub at: usize,
    pub insight: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Marks {
    /// Mark kind -> expiry tick. Kinds: 0 root, 1 slow, 2 decay, 3 calculus, 4 irradiated, 5 pulled, 6 knocked, 7 revealed.
    pub until: [usize; 8],
}
impl Marks {
    pub fn count(&self, tick: usize) -> usize {
        self.until.iter().filter(|u| **u > tick).count()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Stats {
    pub casts: usize,
    pub steps: usize,
    pub combo_steps: usize,
    pub combos: usize,
    pub signatures: usize,
    pub unifications: usize,
    pub insights: usize,
    pub fizzles: usize,
    pub damage: i64,
    pub healing: i64,
    pub ults: usize,
    pub uses: [usize; N],
    /// Sum of each cast's equation efficiency (per 100) and the number of casts measured.
    pub efficiency: i64,
    pub measured: usize,
    pub sig_uses: Vec<usize>,
    pub longest: usize,
    /// Casts that used under 35% of their equation (Photoelectric without marks, a point-blank Apple...).
    pub misplays: usize,
    pub searches: usize,
}

/// What the planner measured about one (skill, target), so each candidate combo costs only arithmetic.
#[derive(Clone, Debug)]
struct Facts {
    skill: usize,
    naive: bool,
    d_real: i64,
    d: i64,
    blocked: bool,
    raw: i64,
    n20: i64,
    n25: i64,
    n30: i64,
    n_radius: i64,
    marks_now: i64,
    moving: bool,
    speed: i64,
    in_radium: bool,
    loss: i64,
    my_loss: i64,
    threat: i64,
    rays: bool,
    ally_loss: i64,
    left: i64,
    lent: [i64; N],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Planned {
    pub skill: usize,
    pub aim: Aim,
    pub cast: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Combo {
    pub steps: Vec<usize>,
    pub aim: Aim,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Unify {
    order: usize,
    combo: u32,
    settle: usize,
    target: Option<usize>,
}

#[derive(Clone, Debug)]
pub struct Core {
    pub rank: usize,
    pub rng: u64,
    pub charges: [usize; CHARGES],
    pub cooldown: [usize; N],
    pub reserve: Vec<Step>,
    pub last_step: usize,
    pub live: Vec<Live>,
    pub marks: BTreeMap<usize, Marks>,
    /// Per fighter: recent (tick, hp, x, y) samples every 6 ticks.
    pub history: BTreeMap<usize, Vec<(usize, i64, i64, i64)>>,
    pub limits: BTreeMap<usize, Window>,
    pub controls: BTreeMap<usize, Window>,
    pub twin_lock: BTreeMap<usize, usize>,
    pub unify_at: usize,
    unify: Option<Unify>,
    pub combo_damage: BTreeMap<u32, i64>,
    /// Power lent by the reserved skills, added to the combo's first hit.
    pub fused: BTreeMap<u32, i64>,
    pub ult_at: usize,
    pub ult_warn: Option<(usize, Aim)>,
    pub last_combo: Option<Combo>,
    pub plan: Vec<Planned>,
    pub next_step: usize,
    /// After a search that found nothing to play, the next one waits (unless the charges change).
    next_think: (usize, usize),
    pub stats: Stats,
    pub persona: usize,
    pub last_cast: usize,
    pub reason: &'static str,
    pub last_hp: i64,
    next_uid: u32,
    next_combo: u32,
    hud: (usize, usize, usize, usize),
    hud_until: usize,
    queue_drawn: Vec<usize>,
    last_tick: Option<usize>,
}

fn d2(ax: i64, ay: i64, bx: i64, by: i64) -> i128 {
    let (dx, dy) = ((ax - bx) as i128, (ay - by) as i128);
    dx * dx + dy * dy
}
fn sq(v: i64) -> i128 {
    (v as i128) * (v as i128)
}
fn isqrt(v: i128) -> i64 {
    if v <= 0 {
        return 0;
    }
    let mut x = v;
    let mut y = (x + 1) / 2;
    while y < x {
        x = y;
        y = (x + v / x) / 2;
    }
    x as i64
}
fn dist(ax: i64, ay: i64, bx: i64, by: i64) -> i64 {
    isqrt(d2(ax, ay, bx, by))
}
/// Distance from (px, py) to the segment a-b, squared.
fn seg_d2(ax: i64, ay: i64, bx: i64, by: i64, px: i64, py: i64) -> i128 {
    let (vx, vy) = ((bx - ax) as i128, (by - ay) as i128);
    let (wx, wy) = ((px - ax) as i128, (py - ay) as i128);
    let len = vx * vx + vy * vy;
    if len == 0 {
        return wx * wx + wy * wy;
    }
    let t = ((wx * vx + wy * vy) * 1000 / len).clamp(0, 1000);
    let (cx, cy) = (vx * t / 1000 - wx, vy * t / 1000 - wy);
    cx * cx + cy * cy
}
/// Eight headings, 0 = east, counter-clockwise in screen terms (y grows down).
pub fn heading(dx: i64, dy: i64) -> usize {
    if dx == 0 && dy == 0 {
        return 0;
    }
    let (ax, ay) = (dx.abs(), dy.abs());
    let diag = ax * 5 < ay * 12 && ay * 5 < ax * 12;
    match (dx >= 0, dy >= 0, diag, ax >= ay) {
        (true, _, false, true) => 0,
        (true, false, true, _) => 1,
        (_, false, false, false) => 2,
        (false, false, true, _) => 3,
        (false, _, false, true) => 4,
        (false, true, true, _) => 5,
        (_, true, false, false) => 6,
        (true, true, true, _) => 7,
    }
}

impl Core {
    pub fn new(seed: u64, owner: usize, rank: usize) -> Core {
        Core {
            rank: rank.min(7),
            rng: (seed ^ (owner as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15) ^ 0x5543_4f52_45).max(1),
            charges: [0; CHARGES],
            cooldown: [0; N],
            reserve: vec![],
            last_step: 0,
            live: vec![],
            marks: BTreeMap::new(),
            history: BTreeMap::new(),
            limits: BTreeMap::new(),
            controls: BTreeMap::new(),
            twin_lock: BTreeMap::new(),
            unify_at: 0,
            unify: None,
            combo_damage: BTreeMap::new(),
            fused: BTreeMap::new(),
            ult_at: 0,
            ult_warn: None,
            last_combo: None,
            plan: vec![],
            next_step: 0,
            next_think: (0, usize::MAX),
            stats: Stats::default(),
            persona: 0,
            last_cast: 0,
            reason: "",
            last_hp: 0,
            next_uid: 0,
            next_combo: 0,
            hud: (usize::MAX, 0, 0, 0),
            hud_until: 0,
            queue_drawn: vec![],
            last_tick: None,
        }
    }
    fn random(&mut self, n: u64) -> u64 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 7;
        self.rng ^= self.rng << 17;
        if n == 0 {
            0
        } else {
            self.rng % n
        }
    }
    pub fn ready_charges(&self, tick: usize) -> usize {
        self.charges.iter().filter(|t| **t <= tick).count()
    }
    pub fn usable(&self, skill: usize, tick: usize) -> bool {
        self.cooldown[skill] <= tick
    }
    fn spend_charge(&mut self, tick: usize) -> bool {
        let Some(slot) = (0..CHARGES).filter(|i| self.charges[*i] <= tick).min_by_key(|i| (self.charges[*i], *i)) else {
            return false;
        };
        self.charges[slot] = tick + CHARGE_TICKS;
        true
    }
    /// Clears everything owned by a dead Unified (charges and cooldowns keep running).
    pub fn on_dead(&mut self) {
        self.reserve.clear();
        self.live.clear();
        self.plan.clear();
        self.unify = None;
        self.ult_warn = None;
        self.marks.clear();
        self.history.clear();
        self.combo_damage.clear();
        self.fused.clear();
        self.hud = (usize::MAX, 0, 0, 0);
        self.queue_drawn.clear();
    }

    // ---------------------------------------------------------------- the world model

    fn me<'a>(&self, inp: &'a Input) -> Option<&'a Fighter> {
        inp.fighters.iter().find(|f| f.id == inp.me)
    }
    fn get<'a>(inp: &'a Input, id: usize) -> Option<&'a Fighter> {
        inp.fighters.iter().find(|f| f.id == id && f.hp > 0)
    }
    fn enemies<'a>(inp: &'a Input, team: usize) -> impl Iterator<Item = &'a Fighter> {
        inp.fighters.iter().filter(move |f| f.team != team && f.hp > 0 && f.visible && f.targetable)
    }
    /// Average motion per tick over the last ~half second.
    pub fn velocity(&self, id: usize) -> (i64, i64) {
        let Some(h) = self.history.get(&id) else { return (0, 0) };
        if h.len() < 2 {
            return (0, 0);
        }
        let (a, b) = (h[h.len().saturating_sub(6).min(h.len() - 2)], h[h.len() - 1]);
        let dt = (b.0 - a.0).max(1) as i64;
        ((b.2 - a.2) / dt, (b.3 - a.3) / dt)
    }
    pub fn speed(&self, id: usize) -> i64 {
        let (vx, vy) = self.velocity(id);
        isqrt(sq(vx) + sq(vy))
    }
    /// HP the fighter lost over the last `ticks` (0 if it gained).
    pub fn recent_loss(&self, id: usize, now_hp: i64, ticks: usize, tick: usize) -> i64 {
        let Some(h) = self.history.get(&id) else { return 0 };
        let past = h.iter().find(|s| tick.saturating_sub(s.0) <= ticks).map_or(now_hp, |s| s.1);
        (past - now_hp).max(0)
    }
    pub fn marks_on(&self, id: usize, tick: usize) -> usize {
        self.marks.get(&id).map_or(0, |m| m.count(tick))
    }
    fn mark(&mut self, id: usize, kind: usize, until: usize) {
        let m = self.marks.entry(id).or_default();
        m.until[kind] = m.until[kind].max(until);
    }
    fn observe(&mut self, inp: &Input) {
        if inp.tick % 6 != 0 {
            return;
        }
        for f in inp.fighters.iter().filter(|f| f.hp > 0) {
            let h = self.history.entry(f.id).or_default();
            h.push((inp.tick, f.hp, f.x, f.y));
            let keep = HISTORY_TICKS + 60;
            while h.first().is_some_and(|s| inp.tick.saturating_sub(s.0) > keep) {
                h.remove(0);
            }
        }
        self.history.retain(|id, _| inp.fighters.iter().any(|f| f.id == *id && f.hp > 0));
        let tick = inp.tick;
        self.marks.retain(|_, m| m.count(tick) > 0);
        self.twin_lock.retain(|_, t| *t > tick);
    }

    // ---------------------------------------------------------------- damage and control

    /// Raw damage of `frac` percent of a skill's hit, before limits.
    pub fn raw(&self, skill: usize, me: &Fighter, target: Option<&Fighter>, frac: i64, mult: i64) -> i64 {
        let e = &EQS[skill];
        let mut n = e.base + me.ad * e.ad / 100 + me.ap * e.ap / 100;
        if let Some(t) = target {
            n += t.max_hp * e.hp / 100;
        }
        n * frac / 100 * mult / 100
    }
    fn damage(&mut self, inp: &Input, out: &mut Vec<Cmd>, target: &Fighter, amount: i64, kind: Kind, combo: u32) -> i64 {
        if amount <= 0 || target.hp <= 0 {
            return 0;
        }
        let cap = if target.champion { target.max_hp * LIMIT_PCT / 100 } else { i64::MAX / 4 };
        let n = self.limits.entry(target.id).or_default().allow(inp.tick, LIMIT_TICKS, cap.max(1) as usize, amount as usize) as i64;
        if n == 0 {
            return 0;
        }
        out.push(Cmd::Damage { target: target.id, amount: n, kind });
        self.stats.damage += n;
        *self.combo_damage.entry(combo).or_default() += n;
        n
    }
    fn control(&mut self, inp: &Input, target: &Fighter, ticks: usize) -> usize {
        if target.cc_immune {
            return 0;
        }
        self.controls.entry(target.id).or_default().allow(inp.tick, CONTROL_WINDOW_TICKS, CONTROL_LIMIT_TICKS, ticks)
    }
    fn root(&mut self, inp: &Input, out: &mut Vec<Cmd>, target: &Fighter, ticks: usize) {
        let n = self.control(inp, target, ticks);
        if n > 0 {
            out.push(Cmd::Root { target: target.id, ticks: n });
            self.mark(target.id, 0, inp.tick + MARK_TICKS);
        }
    }
    fn slow(&mut self, inp: &Input, out: &mut Vec<Cmd>, target: &Fighter, pct: i64, ticks: usize) {
        out.push(Cmd::Speed { target: target.id, pct: -pct.clamp(0, 60), ticks });
        self.mark(target.id, 1, inp.tick + MARK_TICKS);
    }
    /// A shove `amount` units along (dx, dy) (per 10000) over 6 ticks.
    fn shove(&mut self, inp: &Input, target: &Fighter, dx: i64, dy: i64, amount: i64, combo: u32, kind: usize) {
        if target.cc_immune || amount <= 0 {
            return;
        }
        if self.control(inp, target, 6) == 0 {
            return;
        }
        self.mark(target.id, kind, inp.tick + MARK_TICKS);
        let uid = self.uid();
        self.live.push(Live {
            uid,
            skill: WELL,
            combo,
            mods: Mods::plain(100),
            body: Body::Shove { target: target.id, dx: dx * amount / 10000 / 6, dy: dy * amount / 10000 / 6, left: 6 },
            done: false,
        });
    }
    fn uid(&mut self) -> u32 {
        self.next_uid += 1;
        self.next_uid
    }
    fn fx(out: &mut Vec<Cmd>, tag: String, x: i64, y: i64, unit: Option<usize>, ticks: usize) {
        out.push(Cmd::Fx { tag, x, y, unit, ticks });
    }

    // ---------------------------------------------------------------- the hit pipeline

    /// One hit of a cast (or of one of its secondary bodies) on `target`.
    #[allow(clippy::too_many_arguments)]
    fn strike(&mut self, inp: &Input, out: &mut Vec<Cmd>, skill: usize, mods: &Mods, combo: u32, target: &Fighter, frac: i64, from: (i64, i64), center: (i64, i64)) -> i64 {
        let Some(me) = self.me(inp).cloned() else { return 0 };
        let tick = inp.tick;
        let e = &EQS[skill];
        let mut amount = self.raw(skill, &me, Some(target), frac, mods.mult);
        let marks = self.marks_on(target.id, tick);
        if skill == PHOTO {
            amount = if marks >= 2 { amount * (100 + 10 * (marks as i64 - 2)) / 100 } else { amount * 15 / 100 };
        }
        // Traits that change this hit's size.
        if mods.has(ALPHA) > 0 {
            amount = amount * (100 + 40 * mods.has(ALPHA)) / 100;
        }
        if mods.has(PHOTO) > 0 && marks >= 2 {
            amount = amount * (100 + 80 * mods.has(PHOTO)) / 100;
        }
        if mods.has(CALCULUS) > 0 || mods.fx(Fx::Derivative).is_some() {
            let pct = 20 * mods.has(CALCULUS) + mods.fx(Fx::Derivative).unwrap_or(0);
            amount += self.recent_loss(target.id, target.hp, 120, tick) * pct / 100;
        }
        if let Some(cap) = mods.fx(Fx::Gamma) {
            let v = self.speed(target.id).min(1200);
            amount = amount * (100 + cap * v / 1200) / 100;
        }
        // Calculus's own mark pays out on the next hit.
        if self.marks.get(&target.id).is_some_and(|m| m.until[3] > tick) && skill != CALCULUS {
            let bonus = (self.recent_loss(target.id, target.hp, 120, tick) * 35 / 100).min(target.max_hp * 8 / 100 + 60);
            amount += bonus;
            if let Some(m) = self.marks.get_mut(&target.id) {
                m.until[3] = 0;
            }
        }
        if !mods.secondary {
            if let Some(f) = self.fused.remove(&combo) {
                amount += f * mods.mult / 100;
            }
        }
        let dealt = self.damage(inp, out, target, amount, e.kind, combo);
        if mods.has(EMC2) > 0 {
            let extra = target.max_hp * 2 * mods.has(EMC2) / 100 * mods.mult / 100;
            self.damage(inp, out, target, extra, Kind::True, combo);
        }
        if let Some(pct) = mods.fx(Fx::Burst) {
            if marks >= 2 && target.hp < target.max_hp {
                self.damage(inp, out, target, target.max_hp * pct / 100, Kind::True, combo);
            }
        }
        if mods.fx(Fx::Reveal).is_some() {
            self.mark(target.id, 7, tick + mods.fx(Fx::Reveal).unwrap_or(240) as usize);
        }
        // Skill riders.
        match skill {
            APPLE if frac >= 100 => self.root(inp, out, target, 30),
            CALCULUS => self.mark(target.id, 3, tick + MARK_TICKS),
            HALF_LIFE if frac >= 100 => self.dot(target.id, self.raw(HALF_LIFE, &me, None, 100, mods.mult), combo, tick),
            RADIUM | WELL => self.mark(target.id, if skill == RADIUM { 4 } else { 5 }, tick + 30),
            _ => {}
        }
        if mods.secondary {
            return dealt;
        }
        // Traits that leave something behind.
        if mods.has(APPLE) > 0 {
            for k in 0..mods.has(APPLE) {
                self.spawn_lob(inp, APPLE, combo, mods.mult, target.x + 2000 * k, target.y, 18 + 6 * k as usize, 30, 18);
            }
        }
        if mods.has(ORBIT) > 0 {
            self.spawn_planet(inp, ORBIT, combo, Mods::plain(mods.mult * 25 / 100 * mods.has(ORBIT)), (target.x, target.y), (target.x, target.y), 0, 7000, 2);
        }
        if mods.has(CRADLE) > 0 {
            let (dx, dy) = unit(target.x - from.0, target.y - from.1);
            if let Some(last) = Self::enemies(inp, me.team)
                .filter(|f| f.id != target.id && seg_d2(target.x, target.y, target.x + dx * 3, target.y + dy * 3, f.x, f.y) <= sq(6000))
                .max_by_key(|f| (d2(target.x, target.y, f.x, f.y), f.id))
                .cloned()
            {
                let plain = Mods::plain(mods.mult * 70 / 100);
                self.strike(inp, out, skill, &plain, combo, &last, frac, from, center);
                Self::fx(out, format!("im{CRADLE}"), last.x, last.y, None, 24);
            }
        }
        if mods.has(THIRD_LAW) > 0 {
            let (dx, dy) = unit(target.x - center.0, target.y - center.1);
            let (dx, dy) = if dx == 0 && dy == 0 { unit(target.x - me.x, target.y - me.y) } else { (dx, dy) };
            self.shove(inp, target, dx, dy, 6000 * mods.has(THIRD_LAW), combo, 6);
            out.push(Cmd::Shield { target: me.id, amount: dealt * 8 * mods.has(THIRD_LAW) / 100, ticks: 120 });
        }
        if mods.has(RADIUM) > 0 {
            self.spawn_zone(inp, RADIUM, combo, Mods::plain(mods.mult), target.x, target.y, 7000, 90, 15, 40 * mods.has(RADIUM));
        }
        if mods.has(HALF_LIFE) > 0 {
            self.dot(target.id, dealt * 15 * mods.has(HALF_LIFE) / 100, combo, tick);
        }
        if mods.has(TIME_FIELD) > 0 {
            let gamma = 30 * self.speed(target.id).min(1200) / 1200;
            let cap = mods.fx(Fx::Slowcap).unwrap_or(45);
            self.slow(inp, out, target, ((15 + gamma) * mods.has(TIME_FIELD)).min(cap), 90);
        }
        if mods.has(WELL) > 0 {
            let (dx, dy) = unit(center.0 - target.x, center.1 - target.y);
            if dx != 0 || dy != 0 {
                let d = dist(center.0, center.1, target.x, target.y).min(8000 * mods.has(WELL));
                self.shove(inp, target, dx, dy, d, combo, 5);
            }
        }
        if mods.fx(Fx::Lineknock).is_some() {
            let (dx, dy) = unit(target.x - from.0, target.y - from.1);
            self.shove(inp, target, dx, dy, mods.fx(Fx::Lineknock).unwrap_or(0), combo, 6);
        }
        if mods.has(CHAIN) > 0 || (mods.fx(Fx::Markjump).is_some() && marks >= 1) {
            let jumps = mods.has(CHAIN) as usize + usize::from(mods.fx(Fx::Markjump).is_some() && marks >= 1);
            let k = if mods.fx(Fx::Critical).is_some() || self.in_zone(RADIUM, target.x, target.y) { 100 } else { 70 };
            self.chain_from(inp, out, skill, combo, target, jumps, k, frac * k / 100, mods.mult);
        }
        dealt
    }
    fn dot(&mut self, target: usize, a: i64, combo: u32, tick: usize) {
        if a <= 0 {
            return;
        }
        self.mark(target, 2, tick + 300);
        let uid = self.uid();
        self.live.push(Live { uid, skill: HALF_LIFE, combo, mods: Mods::plain(100), body: Body::Dot { target, a, n: 0, next: tick + 1 }, done: false });
    }
    #[allow(clippy::too_many_arguments)]
    fn chain_from(&mut self, inp: &Input, out: &mut Vec<Cmd>, skill: usize, combo: u32, first: &Fighter, jumps: usize, k: i64, frac: i64, mult: i64) {
        let Some(me) = self.me(inp).cloned() else { return };
        let mut hit = vec![first.id];
        let mut at = first.clone();
        let mut f = frac;
        for _ in 0..jumps {
            let Some(next) = Self::enemies(inp, me.team)
                .filter(|e| !hit.contains(&e.id) && d2(at.x, at.y, e.x, e.y) <= sq(25000))
                .min_by_key(|e| (d2(at.x, at.y, e.x, e.y), e.id))
                .cloned()
            else {
                break;
            };
            Self::fx(out, format!("jump_a{}", heading(next.x - at.x, next.y - at.y)), (at.x + next.x) / 2, (at.y + next.y) / 2, None, 8);
            self.strike(inp, out, skill, &Mods::plain(mult), combo, &next, f, (at.x, at.y), (at.x, at.y));
            hit.push(next.id);
            at = next;
            f = f * k / 100;
        }
    }
    fn in_zone(&self, skill: usize, x: i64, y: i64) -> bool {
        self.live.iter().any(|l| l.skill == skill && !l.done && matches!(l.body, Body::Zone { x: zx, y: zy, radius, .. } if d2(zx, zy, x, y) <= sq(radius)))
    }

    // ---------------------------------------------------------------- spawning bodies

    #[allow(clippy::too_many_arguments)]
    fn spawn_lob(&mut self, inp: &Input, skill: usize, combo: u32, mult: i64, x: i64, y: i64, flight: usize, frac: i64, root: usize) {
        let uid = self.uid();
        self.live.push(Live {
            uid,
            skill,
            combo,
            mods: Mods::plain(mult),
            body: Body::Lob { x, y, land: inp.tick + flight, radius: EQS[APPLE].radius * 2 / 3, root, frac },
            done: false,
        });
    }
    #[allow(clippy::too_many_arguments)]
    fn spawn_planet(&mut self, inp: &Input, skill: usize, combo: u32, mods: Mods, from: (i64, i64), to: (i64, i64), speed: i64, radius: i64, laps: usize) {
        let (dx, dy) = unit(to.0 - from.0, to.1 - from.1);
        let uid = self.uid();
        let travel = dist(from.0, from.1, to.0, to.1);
        let power = mods.mult;
        let _ = inp;
        self.live.push(Live {
            uid,
            skill,
            combo,
            mods,
            body: Body::Planet { x: from.0, y: from.1, cx: to.0, cy: to.1, dx, dy, travel, speed: speed.max(1), radius, angle: 0, left: 48 * laps, power, last: BTreeMap::new() },
            done: false,
        });
    }
    #[allow(clippy::too_many_arguments)]
    fn spawn_zone(&mut self, inp: &Input, skill: usize, combo: u32, mods: Mods, x: i64, y: i64, radius: i64, life: usize, period: usize, frac: i64) {
        let uid = self.uid();
        self.live.push(Live {
            uid,
            skill,
            combo,
            mods,
            body: Body::Zone { x, y, radius, start: inp.tick, until: inp.tick + life, period: period.max(1), next: inp.tick, enriched: inp.tick, reopen: false, frac },
            done: false,
        });
    }
    #[allow(clippy::too_many_arguments)]
    fn spawn_ray(&mut self, skill: usize, combo: u32, mods: Mods, from: (i64, i64), to: (i64, i64), range: i64) {
        let e = &EQS[skill];
        let (dx, dy) = unit(to.0 - from.0, to.1 - from.1);
        let (dx, dy) = if dx == 0 && dy == 0 { (10000, 0) } else { (dx, dy) };
        let pen = mods.has(GAMMA) > 0 || mods.fx(Fx::Wallless).is_some();
        let uid = self.uid();
        let split = mods.has(PRISM) > 0 && !mods.secondary;
        self.live.push(Live {
            uid,
            skill,
            combo,
            body: Body::Ray {
                x: from.0,
                y: from.1,
                dx,
                dy,
                remaining: range,
                speed: e.speed.max(3000),
                width: e.width.max(4000),
                pierce: if pen { 9 } else { e.pierce },
                hits: vec![],
                walls: e.walls || pen,
                turn: if mods.has(BETA) > 0 { 300 * mods.has(BETA) } else { e.turn },
                bent: false,
                stop: e.stop && !pen,
                split,
                first: None,
                attenuation: 100,
            },
            mods,
            done: false,
        });
    }

    /// Resolve a cast: the skill `skill` at `aim` carrying `mods`.
    #[allow(clippy::too_many_arguments)]
    fn launch(&mut self, inp: &Input, out: &mut Vec<Cmd>, skill: usize, aim: Aim, mods: Mods, combo: u32, origin: (i64, i64)) {
        let Some(me) = self.me(inp).cloned() else { return };
        let e = &EQS[skill];
        let target = match aim {
            Aim::Unit(id) => Self::get(inp, id).cloned(),
            Aim::Point(..) => None,
        };
        let (tx, ty) = match aim {
            Aim::Unit(_) => target.as_ref().map_or((me.x, me.y), |t| (t.x, t.y)),
            Aim::Point(x, y) => (x, y),
        };
        let mut range = e.range;
        let mut radius = e.radius;
        if mods.has(ALPHA) > 0 {
            range = range * 70 / 100;
            radius = radius * 70 / 100;
        }
        if mods.has(GAMMA) > 0 && e.shape == Shape::Zone {
            radius = radius * 120 / 100;
        }
        match e.shape {
            Shape::Line | Shape::Bolt => {
                let mut to = (tx, ty);
                // E = mc² meets an own Time Field on its way: the energy waits, then releases +25%.
                self.spawn_ray(skill, combo, mods.clone(), origin, to, range);
                if mods.fx(Fx::Spectrum).is_some() {
                    for (k, s) in [ALPHA, BETA, GAMMA].into_iter().enumerate() {
                        let off = (k as i64 - 1) * 3000;
                        to = (tx + off, ty - off);
                        self.spawn_ray(s, combo, Mods::plain(mods.mult * mods.fx(Fx::Spectrum).unwrap_or(50) / 100), origin, to, EQS[s].range);
                    }
                }
            }
            Shape::Lob => {
                let h = dist(origin.0, origin.1, tx, ty).min(range);
                let frac = 100 + 60 * h / range.max(1);
                let uid = self.uid();
                self.live.push(Live {
                    uid,
                    skill,
                    combo,
                    mods: mods.clone(),
                    body: Body::Lob { x: tx, y: ty, land: inp.tick + e.flight, radius, root: 30, frac },
                    done: false,
                });
                Self::fx(out, "drop".into(), tx, ty, None, e.flight);
            }
            Shape::Planet => {
                let to = if dist(origin.0, origin.1, tx, ty) > range {
                    let (dx, dy) = unit(tx - origin.0, ty - origin.1);
                    (origin.0 + dx * range / 10000, origin.1 + dy * range / 10000)
                } else {
                    (tx, ty)
                };
                let r = dist(origin.0, origin.1, tx, ty).max(1);
                let power = (100 * 625_000_000 / (r as i128 * r as i128).max(1)).clamp(50, 160) as i64;
                let mut m = mods.clone();
                m.mult = m.mult * power / 100;
                let laps = 1 + mods.fx(Fx::Repeat).unwrap_or(0) as usize;
                self.spawn_planet(inp, skill, combo, m, origin, to, e.speed, radius, laps);
            }
            Shape::Cradle => {
                let (dx, dy) = unit(tx - origin.0, ty - origin.1);
                let (dx, dy) = if dx == 0 && dy == 0 { (10000, 0) } else { (dx, dy) };
                let end = (origin.0 + dx * range / 10000, origin.1 + dy * range / 10000);
                let walls = mods.fx(Fx::Wallless).is_some() || mods.has(GAMMA) > 0;
                let mut line: Vec<Fighter> = Self::enemies(inp, me.team)
                    .filter(|f| seg_d2(origin.0, origin.1, end.0, end.1, f.x, f.y) <= sq(e.width))
                    .filter(|f| walls || !self.blocked(inp, origin, (f.x, f.y)))
                    .cloned()
                    .collect();
                line.sort_by_key(|f| (d2(origin.0, origin.1, f.x, f.y), f.id));
                Self::fx(out, format!("pk{CRADLE}_a{}", heading(dx, dy)), origin.0 + dx * 2, origin.1 + dy * 2, None, 12);
                if let Some(first) = line.first().cloned() {
                    self.strike(inp, out, skill, &mods, combo, &first, 100, origin, origin);
                    Self::fx(out, format!("im{CRADLE}"), first.x, first.y, None, 24);
                    if line.len() > 1 {
                        let last = line.last().cloned().unwrap_or(first.clone());
                        let frac = 100 + 15 * (line.len() as i64 - 2);
                        let mut plain = mods.clone();
                        plain.secondary = true;
                        self.strike(inp, out, skill, &plain, combo, &last, frac, origin, origin);
                        self.shove(inp, &last, dx, dy, 9000, combo, 6);
                        Self::fx(out, format!("im{CRADLE}"), last.x, last.y, None, 24);
                    }
                }
            }
            Shape::Caster => {
                let shield = 40 + me.ap * 30 / 100;
                out.push(Cmd::Shield { target: me.id, amount: shield * mods.mult / 100, ticks: e.life });
                let uid = self.uid();
                self.live.push(Live { uid, skill, combo, mods: mods.clone(), body: Body::Reaction { until: inp.tick + e.life, taken: 0 }, done: false });
                Self::fx(out, "shield".into(), me.x, me.y, Some(me.id), e.life);
            }
            Shape::Ally => {
                let ally = target.filter(|t| t.team == me.team).unwrap_or(me.clone());
                let past = self
                    .history
                    .get(&ally.id)
                    .and_then(|h| h.iter().find(|s| inp.tick.saturating_sub(s.0) <= HISTORY_TICKS).copied());
                if let Some((_, hp, x, y)) = past {
                    if self.twin_lock.get(&ally.id).is_none_or(|t| *t <= inp.tick) {
                        self.twin_lock.insert(ally.id, inp.tick + TWIN_LOCK_TICKS);
                        let heal = (hp - ally.hp).clamp(0, ally.max_hp * 20 / 100);
                        if heal > 0 {
                            out.push(Cmd::Heal { target: ally.id, amount: heal });
                            self.stats.healing += heal;
                        }
                        let travel = dist(ally.x, ally.y, x, y);
                        let (nx, ny) = if travel > 40000 {
                            let (dx, dy) = unit(x - ally.x, y - ally.y);
                            (ally.x + dx * 4, ally.y + dy * 4)
                        } else {
                            (x, y)
                        };
                        if !ally.cc_immune && !(inp.wall)(nx, ny) {
                            out.push(Cmd::SetPos { target: ally.id, x: nx, y: ny });
                        }
                        Self::fx(out, "twin".into(), ally.x, ally.y, None, 30);
                        Self::fx(out, "twin".into(), nx, ny, None, 30);
                        let pulse: Vec<Fighter> = Self::enemies(inp, me.team).filter(|f| d2(f.x, f.y, nx, ny) <= sq(radius)).cloned().collect();
                        for f in pulse {
                            self.strike(inp, out, skill, &mods, combo, &f, 100, (nx, ny), (nx, ny));
                        }
                    }
                }
            }
            Shape::Zone => {
                let (zx, zy) = if dist(origin.0, origin.1, tx, ty) > range {
                    let (dx, dy) = unit(tx - origin.0, ty - origin.1);
                    (origin.0 + dx * range / 10000, origin.1 + dy * range / 10000)
                } else {
                    (tx, ty)
                };
                // Enrich: Radium recast on a live sample resets N₀ there instead of stacking.
                if skill == RADIUM {
                    if let Some(l) = self.live.iter_mut().find(|l| l.skill == RADIUM && !l.mods.secondary && !l.done && matches!(l.body, Body::Zone { x, y, radius, .. } if d2(x, y, zx, zy) <= sq(radius))) {
                        if let Body::Zone { enriched, until, .. } = &mut l.body {
                            *enriched = inp.tick;
                            *until = inp.tick + e.life;
                        }
                        Self::fx(out, format!("zn{RADIUM}"), zx, zy, None, 48);
                        return;
                    }
                }
                let mut life = e.life;
                if let Some(p) = mods.fx(Fx::Extend) {
                    life = life * (100 + p as usize) / 100;
                }
                let period = if e.period == 0 { life } else { e.period };
                self.spawn_zone(inp, skill, combo, mods.clone(), zx, zy, radius, life, period, 100);
                if skill == RING {
                    let inside: Vec<Fighter> = Self::enemies(inp, me.team).filter(|f| d2(f.x, f.y, zx, zy) <= sq(radius)).cloned().collect();
                    for f in inside {
                        self.strike(inp, out, skill, &mods, combo, &f, 100, (zx, zy), (zx, zy));
                    }
                }
                if let Some(n) = mods.fx(Fx::Rain) {
                    for k in 0..n {
                        let a = (k * 10000 / n.max(1)) as i64;
                        let (ox, oy) = ((a - 5000) * radius / 10000, ((k * 7919) % 10000 - 5000) * radius / 10000);
                        self.spawn_lob(inp, APPLE, combo, mods.mult, zx + ox, zy + oy, 24 + 8 * k as usize, 30, 18);
                    }
                }
            }
        }
        if e.shape != Shape::Zone {
            if let Some(n) = mods.fx(Fx::Rain) {
                for k in 0..n {
                    let off = (k - n / 2) * 5000;
                    self.spawn_lob(inp, APPLE, combo, mods.mult, tx + off, ty - off / 2, 24 + 8 * k as usize, 30, 18);
                }
            }
        }
        // Rewind and repeat: the cast happens again from the same origin.
        let echoes = mods.has(TWIN) + mods.fx(Fx::Repeat).filter(|_| e.shape != Shape::Planet).unwrap_or(0);
        if !mods.secondary {
            for k in 0..echoes {
                let pct = if mods.has(TWIN) > 0 { 60 } else { 100 };
                let uid = self.uid();
                let mut m = mods.clone();
                m.traits[TWIN] = 0;
                m.sig = m.sig.filter(|s| SIGNATURES[*s].fx.iter().all(|(f, _)| *f != Fx::Repeat));
                m.mult = m.mult * pct / 100;
                self.live.push(Live { uid, skill, combo, mods: m, body: Body::Echo { at: inp.tick + 60 * (k as usize + 1), aim, ox: origin.0, oy: origin.1, pct }, done: false });
            }
        }
    }
    fn blocked(&self, inp: &Input, a: (i64, i64), b: (i64, i64)) -> bool {
        let n = (dist(a.0, a.1, b.0, b.1) / 4000).clamp(1, 32);
        (1..n).any(|i| (inp.wall)(a.0 + (b.0 - a.0) * i / n, a.1 + (b.1 - a.1) * i / n))
    }

    // ---------------------------------------------------------------- steps: reserve, cast, ultimate

    /// Can `skill` be reserved or cast now (charge, cooldown, not stunned)?
    pub fn can_step(&self, inp: &Input, skill: usize) -> bool {
        !inp.stunned && skill < N && self.usable(skill, inp.tick) && self.ready_charges(inp.tick) > 0 && self.reserve.len() <= MAX_RESERVE && self.ult_warn.is_none()
    }
    pub fn reserve(&mut self, inp: &Input, out: &mut Vec<Cmd>, skill: usize) -> bool {
        if !self.can_step(inp, skill) || self.reserve.len() >= MAX_RESERVE || !self.spend_charge(inp.tick) {
            return false;
        }
        let insight = !self.reserve.is_empty() && inp.tick <= self.last_step + INSIGHT_TICKS;
        self.reserve.push(Step { skill, at: inp.tick, insight });
        self.last_step = inp.tick;
        self.persona = EQS[skill].science;
        self.stats.steps += 1;
        if insight {
            self.stats.insights += 1;
            if let Some(me) = self.me(inp) {
                Self::fx(out, "insight".into(), me.x, me.y, Some(me.id), 20);
            }
        }
        out.push(Cmd::Log(format!("tick {} reserve {} ({}) queue {:?}", inp.tick, EQS[skill].name, EQS[skill].trait_name, self.reserve.iter().map(|s| s.skill).collect::<Vec<_>>())));
        true
    }
    /// The signature matching the reserved steps plus `last` (ULT_STEP for the ultimate).
    pub fn signature(steps: &[usize], last: u8) -> Option<usize> {
        SIGNATURES.iter().position(|s| {
            s.steps.len() == steps.len() + 1
                && s.steps[..steps.len()].iter().zip(steps).all(|(a, b)| usize::from(*a) == *b)
                && s.steps[steps.len()] == last
        })
    }
    /// Combine the reserved traits with the cast (or the ultimate) into one Mods.
    pub fn compose(&self, reserved: &[Step], last: u8, last_insight: bool) -> Mods {
        let mut traits = [0u8; N];
        for s in reserved {
            traits[s.skill] = (traits[s.skill] + 1).min(2);
        }
        let steps = reserved.len() as i64 + 1;
        let insights = reserved.iter().filter(|s| s.insight).count() as i64 + i64::from(last_insight);
        let sig = SIGNATURES.iter().position(|g| {
            g.steps.len() == reserved.len() + 1 && g.steps[reserved.len()] == last && g.steps.iter().zip(reserved).all(|(a, b)| usize::from(*a) == b.skill)
        });
        let mult = 100 + LINK_PCT * (steps - 1) + INSIGHT_PCT * insights + sig.map_or(0, |s| SIGNATURES[s].bonus);
        Mods { traits, mult, sig, secondary: false }
    }
    pub fn cast(&mut self, inp: &Input, out: &mut Vec<Cmd>, skill: usize, aim: Aim) -> bool {
        if !self.can_step(inp, skill) || !self.spend_charge(inp.tick) {
            return false;
        }
        let Some(me) = self.me(inp).cloned() else { return false };
        let insight = !self.reserve.is_empty() && inp.tick <= self.last_step + INSIGHT_TICKS;
        let mods = self.compose(&self.reserve, skill as u8, insight);
        self.resolve(inp, out, skill, aim, mods, insight, &me, 100);
        true
    }
    /// Common to a cast and the ultimate: cooldowns, Unification, statistics, labels, then the bodies.
    #[allow(clippy::too_many_arguments)]
    fn resolve(&mut self, inp: &Input, out: &mut Vec<Cmd>, skill: usize, aim: Aim, mut mods: Mods, insight: bool, me: &Fighter, extra: i64) {
        let tick = inp.tick;
        let mut steps: Vec<usize> = self.reserve.iter().map(|s| s.skill).collect();
        steps.push(skill);
        for s in &steps {
            self.cooldown[*s] = self.cooldown[*s].max(tick + EQS[*s].cooldown);
            self.stats.uses[*s] += 1;
        }
        mods.mult = mods.mult * extra / 100;
        self.next_combo += 1;
        let combo = self.next_combo;
        // Every reserved skill lends FUSE_PCT of its own power to the cast's first hit.
        let lent: i64 = steps[..steps.len() - 1].iter().map(|k| self.raw(*k, me, None, FUSE_PCT, 100)).sum();
        if lent > 0 {
            self.fused.insert(combo, lent);
        }
        self.stats.casts += 1;
        self.stats.steps += 1;
        self.stats.longest = self.stats.longest.max(steps.len());
        if steps.len() > 1 {
            self.stats.combos += 1;
            self.stats.combo_steps += steps.len();
        }
        if insight {
            self.stats.insights += 1;
            Self::fx(out, "insight".into(), me.x, me.y, Some(me.id), 20);
        }
        self.persona = EQS[skill].science;
        self.last_cast = tick;
        // Labels: the equation of the cast, the signature's name, then the Unification.
        Self::fx(out, format!("eq{skill}"), me.x, me.y, Some(me.id), 60);
        if let Some(s) = mods.sig {
            self.stats.signatures += 1;
            self.stats.sig_uses.push(s);
            Self::fx(out, format!("sig{s}"), me.x, me.y, Some(me.id), 90);
        }
        let mut order = vec![];
        for s in &steps {
            let sc = EQS[*s].science;
            if !order.contains(&sc) {
                order.push(sc);
            }
        }
        if order.len() == 3 && tick >= self.unify_at {
            let k = UNIFICATION.iter().position(|(o, _)| o.as_slice() == order.as_slice()).unwrap_or(0);
            self.unify_at = tick + UNIFY_COOLDOWN_TICKS;
            self.stats.unifications += 1;
            Self::fx(out, format!("uni{k}"), me.x, me.y, Some(me.id), 90);
            let target = match aim {
                Aim::Unit(id) => Some(id),
                Aim::Point(x, y) => Self::enemies(inp, me.team).min_by_key(|f| (d2(f.x, f.y, x, y), f.id)).map(|f| f.id),
            };
            self.unify_now(inp, out, k, target, me);
            self.unify = Some(Unify { order: k, combo, settle: tick + 75, target });
        }
        self.efficiency(inp, skill, aim, &mods, me);
        out.push(Cmd::Log(format!(
            "tick {tick} cast {} steps {:?} mult {}% sig {:?} insight {insight}",
            EQS[skill].name,
            steps.iter().map(|s| EQS[*s].key).collect::<Vec<_>>(),
            mods.mult,
            mods.sig.map(|s| SIGNATURES[s].name)
        )));
        self.last_combo = Some(Combo { steps: steps.clone(), aim });
        self.reserve.clear();
        self.last_step = tick;
        Self::fx(out, format!("im{skill}"), me.x, me.y, Some(me.id), 18);
        self.launch(inp, out, skill, aim, mods, combo, (me.x, me.y));
    }
    /// The parts of a Unification that happen at once.
    fn unify_now(&mut self, inp: &Input, out: &mut Vec<Cmd>, k: usize, target: Option<usize>, me: &Fighter) {
        match k {
            1 => {
                if let Some(t) = target.and_then(|id| Self::get(inp, id)).cloned() {
                    let around: Vec<Fighter> = Self::enemies(inp, me.team).filter(|f| d2(f.x, f.y, t.x, t.y) <= sq(18000)).cloned().collect();
                    for f in around {
                        let (dx, dy) = unit(t.x - f.x, t.y - f.y);
                        let d = dist(t.x, t.y, f.x, f.y) / 2;
                        self.shove(inp, &f, dx, dy, d, 0, 5);
                        self.root(inp, out, &f, 24);
                    }
                }
            }
            3 => out.push(Cmd::Shield { target: me.id, amount: me.max_hp * 12 / 100, ticks: 180 }),
            5 => {
                out.push(Cmd::Speed { target: me.id, pct: 30, ticks: 120 });
                if let Some(slot) = (0..CHARGES).filter(|i| self.charges[*i] > inp.tick).min_by_key(|i| (self.charges[*i], *i)) {
                    self.charges[slot] = inp.tick;
                }
            }
            _ => {}
        }
    }
    /// The parts that depend on the combo's damage, once it has landed.
    fn unify_settle(&mut self, inp: &Input, out: &mut Vec<Cmd>) {
        let Some(u) = self.unify.clone() else { return };
        if inp.tick < u.settle {
            return;
        }
        self.unify = None;
        let Some(me) = self.me(inp).cloned() else { return };
        let dealt = self.combo_damage.get(&u.combo).copied().unwrap_or(0);
        let target = u.target.and_then(|id| Self::get(inp, id)).cloned();
        match u.order {
            0 => {
                if let Some(t) = target {
                    self.damage(inp, out, &t, dealt * 40 / 100 * 130 / 100, Kind::Magic, u.combo);
                    Self::fx(out, "im13".into(), t.x, t.y, None, 24);
                }
            }
            2 => {
                let heal = dealt * 30 / 100;
                if heal > 0 {
                    out.push(Cmd::Heal { target: me.id, amount: heal });
                    self.stats.healing += heal;
                }
            }
            4 => {
                if let Some(t) = target {
                    self.damage(inp, out, &t, (t.max_hp - t.hp) * 8 / 100, Kind::True, u.combo);
                }
            }
            _ => {}
        }
        self.combo_damage.retain(|c, _| *c + 8 > u.combo);
        self.fused.retain(|c, _| *c + 8 > u.combo);
    }
    pub fn ult(&mut self, inp: &Input, out: &mut Vec<Cmd>, aim: Aim) -> bool {
        if inp.stunned || !inp.ult_unlocked || inp.tick < self.ult_at || self.ult_warn.is_some() {
            return false;
        }
        let (x, y) = match aim {
            Aim::Unit(id) => match Self::get(inp, id) {
                Some(t) => (t.x, t.y),
                None => return false,
            },
            Aim::Point(x, y) => (x, y),
        };
        self.ult_at = inp.tick + ULT_COOLDOWN_TICKS;
        self.ult_warn = Some((inp.tick + ULT_WARN_TICKS, aim));
        Self::fx(out, "ultwarn".into(), x, y, None, ULT_WARN_TICKS);
        true
    }
    fn ult_fire(&mut self, inp: &Input, out: &mut Vec<Cmd>) {
        let Some((at, aim)) = self.ult_warn else { return };
        if inp.tick < at {
            return;
        }
        self.ult_warn = None;
        let Some(me) = self.me(inp).cloned() else { return };
        if inp.stunned {
            self.reason = "Theory of Everything interrupted";
            Self::fx(out, "fizzle".into(), me.x, me.y, Some(me.id), 15);
            return;
        }
        // The last combo again, without charges, with whatever is reserved now added.
        let last = self.last_combo.clone().unwrap_or(Combo { steps: vec![APPLE, RADIUM, EMC2], aim });
        let skill = *last.steps.last().unwrap_or(&EMC2);
        let mut reserved: Vec<Step> = last.steps[..last.steps.len() - 1].iter().map(|s| Step { skill: *s, at: inp.tick, insight: false }).collect();
        let own = self.reserve.clone();
        let sig_with_ult = Self::signature(&own.iter().map(|s| s.skill).collect::<Vec<_>>(), ULT_STEP);
        reserved.extend(own.iter().copied());
        let mut mods = self.compose(&reserved[reserved.len().saturating_sub(MAX_RESERVE)..], skill as u8, false);
        if sig_with_ult.is_some() {
            mods.sig = sig_with_ult;
            mods.mult += SIGNATURES[sig_with_ult.unwrap_or(0)].bonus;
        }
        self.stats.ults += 1;
        Self::fx(out, "ult".into(), me.x, me.y, Some(me.id), 48);
        self.resolve(inp, out, skill, aim, mods, false, &me, ULT_PCT);
    }

    // ---------------------------------------------------------------- the tick

    pub fn step(&mut self, inp: &Input) -> Vec<Cmd> {
        let mut out = vec![];
        if self.last_tick == Some(inp.tick) {
            return out;
        }
        self.last_tick = Some(inp.tick);
        let Some(me) = self.me(inp).cloned() else { return out };
        if me.hp <= 0 {
            return out;
        }
        self.observe(inp);
        // Reactions (Third Law) measure damage taken from Unified's own HP.
        let lost = (self.last_hp - me.hp).max(0);
        self.last_hp = me.hp;
        if lost > 0 {
            for l in &mut self.live {
                if let Body::Reaction { taken, .. } = &mut l.body {
                    *taken += lost;
                }
            }
        }
        // A reserve not cast in time is wasted: the charges stay spent.
        if !self.reserve.is_empty() && inp.tick > self.last_step + RESERVE_TICKS {
            self.reserve.clear();
            self.plan.clear();
            self.stats.fizzles += 1;
            self.reason = "Reserve expired";
            Self::fx(&mut out, "fizzle".into(), me.x, me.y, Some(me.id), 15);
        }
        self.ult_fire(inp, &mut out);
        if let Some(a) = inp.action {
            match a {
                Action::Reserve(s, _) => {
                    self.reserve(inp, &mut out, s);
                }
                Action::Cast(s, aim) => {
                    self.cast(inp, &mut out, s, aim);
                }
                Action::Ult(aim) => {
                    self.ult(inp, &mut out, aim);
                }
            }
        }
        if inp.automatic && !inp.stunned {
            self.think(inp, &mut out);
        }
        self.step_bodies(inp, &mut out);
        self.unify_settle(inp, &mut out);
        self.hud(inp, &mut out, &me);
        out
    }

    fn step_bodies(&mut self, inp: &Input, out: &mut Vec<Cmd>) {
        let tick = inp.tick;
        let Some(me) = self.me(inp).cloned() else { return };
        let count = self.live.len();
        for i in 0..count {
            if self.live[i].done {
                continue;
            }
            let mut l = self.live[i].clone();
            match &mut l.body {
                Body::Ray { x, y, dx, dy, remaining, speed, width, pierce, hits, walls, turn, bent, stop, split, first, attenuation } => {
                    // Curving rays (Beta, Curve) steer toward the nearest enemy ahead.
                    if *turn > 0 {
                        if let Some(t) = Self::enemies(inp, me.team).filter(|f| !hits.contains(&f.id)).min_by_key(|f| (d2(*x, *y, f.x, f.y), f.id)) {
                            let (tx, ty) = unit(t.x - *x, t.y - *y);
                            let (nx, ny) = unit(*dx * 10000 / (*turn).max(1) + tx, *dy * 10000 / (*turn).max(1) + ty);
                            if nx != 0 || ny != 0 {
                                (*dx, *dy) = (nx, ny);
                            }
                        }
                    }
                    // A ray crossing an Einstein Ring (or carrying Lens) bends once toward the nearest enemy.
                    let lens = l.mods.has(RING) > 0 || self.in_zone(RING, *x, *y);
                    if lens && !*bent {
                        if let Some(t) = Self::enemies(inp, me.team).filter(|f| !hits.contains(&f.id) && d2(*x, *y, f.x, f.y) <= sq(*remaining)).min_by_key(|f| (d2(*x, *y, f.x, f.y), f.id)) {
                            let (nx, ny) = unit(t.x - *x, t.y - *y);
                            if nx != 0 || ny != 0 {
                                (*dx, *dy) = (nx, ny);
                                *bent = true;
                                l.mods.mult = l.mods.mult * 115 / 100;
                            }
                        }
                    }
                    let step = (*speed).min(*remaining);
                    let (nx, ny) = (*x + *dx * step / 10000, *y + *dy * step / 10000);
                    if !*walls && self.blocked(inp, (*x, *y), (nx, ny)) {
                        *remaining = 0;
                    } else {
                        // E = mc² entering a Time Field: delayed release.
                        let delayed = l.skill == EMC2 && self.in_zone(TIME_FIELD, nx, ny);
                        let mut struck: Vec<Fighter> = Self::enemies(inp, me.team)
                            .filter(|f| !hits.contains(&f.id) && seg_d2(*x, *y, nx, ny, f.x, f.y) <= sq(*width))
                            .cloned()
                            .collect();
                        struck.sort_by_key(|f| (d2(*x, *y, f.x, f.y), f.id));
                        for f in struck {
                            hits.push(f.id);
                            let frac = *attenuation;
                            if delayed {
                                let amount = self.raw(EMC2, &me, Some(&f), 125, l.mods.mult);
                                let uid = self.uid();
                                self.live.push(Live { uid, skill: EMC2, combo: l.combo, mods: Mods::plain(100), body: Body::Delayed { at: tick + 60, target: f.id, amount }, done: false });
                                Self::fx(out, "dot".into(), f.x, f.y, Some(f.id), 60);
                            } else {
                                self.strike(inp, out, l.skill, &l.mods, l.combo, &f, frac, (me.x, me.y), (f.x, f.y));
                            }
                            Self::fx(out, format!("im{}", l.skill), f.x, f.y, None, 24);
                            if l.skill == GAMMA {
                                *attenuation = *attenuation * 85 / 100;
                            }
                            if first.is_none() {
                                *first = Some((f.x, f.y));
                                if *split {
                                    let (mods, combo) = (l.mods.clone(), l.combo);
                                    self.split(out, &mods, combo, (f.x, f.y), (*dx, *dy), &me);
                                }
                            }
                            // A ray hits at most 1 + pierce bodies; a stopping ray ends at its first.
                            if (*stop && *pierce == 0) || hits.len() > *pierce {
                                *remaining = 0;
                                break;
                            }
                        }
                        (*x, *y) = (nx, ny);
                        *remaining -= step;
                        if tick % 4 == 0 && *remaining > 0 {
                            Self::fx(out, format!("pk{}_a{}", l.skill, heading(*dx, *dy)), *x, *y, None, 4);
                        }
                    }
                    if *remaining <= 0 {
                        l.done = true;
                    }
                }
                Body::Lob { x, y, land, radius, root, frac } => {
                    if tick >= *land {
                        let struck: Vec<Fighter> = Self::enemies(inp, me.team).filter(|f| d2(f.x, f.y, *x, *y) <= sq(*radius)).cloned().collect();
                        for f in struck {
                            self.strike(inp, out, APPLE, &l.mods, l.combo, &f, *frac, (me.x, me.y), (*x, *y));
                            if l.mods.secondary {
                                self.root(inp, out, &f, *root);
                            }
                        }
                        Self::fx(out, format!("im{APPLE}"), *x, *y, None, 24);
                        l.done = true;
                    }
                }
                Body::Planet { x, y, cx, cy, dx, dy, travel, speed, radius, angle, left, power, last } => {
                    if *travel > 0 {
                        let s = (*speed).min(*travel);
                        *x += *dx * s / 10000;
                        *y += *dy * s / 10000;
                        *travel -= s;
                    } else {
                        // One lap = 48 ticks around the arrival point.
                        *angle = (*angle + 1) % 48;
                        let a = *angle as i64;
                        let (ux, uy) = circle(a);
                        *x = *cx + ux * *radius / 10000;
                        *y = *cy + uy * *radius / 10000;
                        *left = left.saturating_sub(1);
                    }
                    let near: Vec<Fighter> = Self::enemies(inp, me.team).filter(|f| d2(f.x, f.y, *x, *y) <= sq(7000)).cloned().collect();
                    for f in near {
                        if last.get(&f.id).is_some_and(|t| tick < t + 15) {
                            continue;
                        }
                        last.insert(f.id, tick);
                        let _ = power;
                        self.strike(inp, out, l.skill, &l.mods, l.combo, &f, 100, (*x, *y), (*cx, *cy));
                        if !l.mods.secondary {
                            let (px, py) = unit(*cx - f.x, *cy - f.y);
                            self.shove(inp, &f, px, py, dist(f.x, f.y, *cx, *cy).min(4000), l.combo, 5);
                        }
                    }
                    if tick % 4 == 0 {
                        Self::fx(out, format!("pk{ORBIT}_a{}", heading(*dx, *dy)), *x, *y, None, 4);
                    }
                    if *left == 0 {
                        l.done = true;
                    }
                }
                Body::Zone { x, y, radius, start, until, period, next, enriched, reopen, frac } => {
                    if tick >= *until {
                        if l.mods.has(TWIN) > 0 && !*reopen {
                            *reopen = true;
                            let life = *until - *start;
                            *start = tick + 60;
                            *until = tick + 60 + life;
                            *next = tick + 60;
                            l.mods.mult = l.mods.mult * 60 / 100;
                        } else {
                            l.done = true;
                        }
                    }
                    if !l.done && tick >= *start {
                        // Curve: the zone drifts toward the nearest enemy.
                        if l.mods.has(BETA) > 0 {
                            if let Some(t) = Self::enemies(inp, me.team).min_by_key(|f| (d2(*x, *y, f.x, f.y), f.id)) {
                                let (ux, uy) = unit(t.x - *x, t.y - *y);
                                *x += ux * 150 / 10000;
                                *y += uy * 150 / 10000;
                            }
                        }
                        let inside: Vec<Fighter> = Self::enemies(inp, me.team).filter(|f| d2(f.x, f.y, *x, *y) <= sq(*radius)).cloned().collect();
                        // Continuous zone effects: Time Field slows, Well pulls, allies hasted.
                        if l.skill == TIME_FIELD && tick.is_multiple_of(6) {
                            for f in &inside {
                                let gamma = 30 * self.speed(f.id).min(1200) / 1200;
                                let cap = l.mods.fx(Fx::Slowcap).unwrap_or(45);
                                let pct = (15 + gamma + 15 * l.mods.has(TIME_FIELD)).min(cap);
                                self.slow(inp, out, f, pct, 12);
                            }
                            for a in inp.fighters.iter().filter(|a| a.team == me.team && a.hp > 0 && d2(a.x, a.y, *x, *y) <= sq(*radius)) {
                                out.push(Cmd::Speed { target: a.id, pct: 15, ticks: 12 });
                            }
                        }
                        if (l.skill == WELL || l.mods.has(WELL) > 0) && tick.is_multiple_of(3) {
                            for f in &inside {
                                let r = dist(f.x, f.y, *x, *y).max(4000);
                                let step = (900i128 * 36_000_000 / (r as i128 * r as i128)).clamp(150, 900) as i64 * 3;
                                let (ux, uy) = unit(*x - f.x, *y - f.y);
                                if !f.cc_immune && (ux != 0 || uy != 0) && self.control(inp, f, 1) > 0 {
                                    let (nx, ny) = (f.x + ux * step.min(r) / 10000, f.y + uy * step.min(r) / 10000);
                                    if !(inp.wall)(nx, ny) {
                                        out.push(Cmd::SetPos { target: f.id, x: nx, y: ny });
                                    }
                                    self.mark(f.id, 5, tick + 30);
                                }
                            }
                        }
                        if l.skill == RADIUM || l.skill == RING || l.mods.secondary {
                            for f in &inside {
                                self.mark(f.id, 4, tick + 30);
                            }
                        }
                        if tick >= *next {
                            *next = tick + *period;
                            // N = N₀·e^(−λt): Radium (and its puddles) halve every half-life; Isotope doubles it.
                            let decay = if l.skill == RADIUM || l.mods.secondary {
                                let half_life = if l.mods.fx(Fx::Extend).is_some() { 240 } else { 120 };
                                100i64 >> (tick.saturating_sub(*enriched) / half_life).min(6)
                            } else {
                                100
                            };
                            let hits = l.skill != TIME_FIELD && l.skill != RING && (EQS[l.skill].base > 0 || l.mods.secondary);
                            for f in &inside {
                                if hits {
                                    self.strike(inp, out, l.skill, &l.mods, l.combo, f, *frac * decay / 100, (*x, *y), (*x, *y));
                                }
                            }
                            // Lasing (Split): a small laser at an enemy inside every 0.5 s.
                            if l.mods.has(PRISM) > 0 && (tick - *start) % 30 < *period {
                                if let Some(t) = inside.first() {
                                    for _ in 0..l.mods.has(PRISM) {
                                        self.spawn_ray(PRISM, l.combo, Mods::plain(l.mods.mult * 20 / 100), (*x, *y - 3000), (t.x, t.y), *radius + 10000);
                                    }
                                }
                            }
                            // Apples rain (Drop) inside a zone every 0.5 s.
                            if l.mods.has(APPLE) > 0 && (tick - *start) % 30 < *period {
                                let a = ((tick - *start) / 30) as i64;
                                let (ux, uy) = circle(a * 13);
                                self.spawn_lob(inp, APPLE, l.combo, l.mods.mult, *x + ux * *radius / 20000, *y + uy * *radius / 20000, 18, 30, 18);
                            }
                            // Curie's Notebook / Radioactive Series: rays from the zone.
                            if let Some(every) = l.mods.fx(Fx::Pulse) {
                                if (tick - *start) as i64 % every.max(1) < *period as i64 {
                                    if let Some(t) = Self::enemies(inp, me.team).filter(|f| d2(f.x, f.y, *x, *y) <= sq(*radius + 30000)).min_by_key(|f| (d2(f.x, f.y, *x, *y), f.id)) {
                                        let s = [ALPHA, BETA, GAMMA][((tick - *start) / 30) % 3];
                                        self.spawn_ray(s, l.combo, Mods::plain(l.mods.mult * 50 / 100), (*x, *y), (t.x, t.y), EQS[s].range);
                                    }
                                }
                            }
                            if l.mods.has(CHAIN) > 0 {
                                if let Some(t) = inside.first() {
                                    let k = if l.mods.fx(Fx::Critical).is_some() || l.skill == RADIUM { 100 } else { 70 };
                                    self.chain_from(inp, out, l.skill, l.combo, t, 1, k, *frac * decay / 100 * k / 100, l.mods.mult);
                                }
                            }
                            // Decay / Irradiate on a zone: once a second, a halving dose on everyone inside.
                            let doses = l.mods.has(HALF_LIFE) + if l.mods.secondary { 0 } else { l.mods.has(RADIUM) };
                            if doses > 0 && (tick - *start) % 60 < *period {
                                for f in &inside {
                                    let a = self.raw(l.skill, &me, None, 10 * doses, l.mods.mult).max(4);
                                    self.dot(f.id, a, l.combo, tick);
                                }
                            }
                        }
                        if tick.is_multiple_of(48) || tick == *start {
                            Self::fx(out, if l.mods.secondary && l.skill == RADIUM { "puddle".into() } else { format!("zn{}", l.skill) }, *x, *y, None, 48);
                        }
                    }
                    // Detonate: every damage over time from this combo released now.
                    if l.mods.fx(Fx::Detonate).is_some() && tick == *start + 6 {
                        self.detonate(inp, out, l.combo);
                    }
                }
                Body::Dot { target, a, n, next } => {
                    if tick >= *next {
                        if let Some(f) = Self::get(inp, *target).cloned() {
                            let amount = *a >> *n;
                            self.damage(inp, out, &f, amount.max(1), Kind::Magic, l.combo);
                            if *n == 0 {
                                Self::fx(out, "dot".into(), f.x, f.y, Some(f.id), 300);
                            }
                            *n += 1;
                            *next = tick + 60;
                            if *n >= 5 {
                                l.done = true;
                            }
                        } else {
                            l.done = true;
                        }
                    }
                }
                Body::Reaction { until, taken } => {
                    if tick >= *until {
                        let shove: Vec<Fighter> = Self::enemies(inp, me.team).filter(|f| d2(f.x, f.y, me.x, me.y) <= sq(EQS[THIRD_LAW].radius)).cloned().collect();
                        for f in shove {
                            let (dx, dy) = unit(f.x - me.x, f.y - me.y);
                            let extra = *taken * 30 / 100;
                            let amount = self.raw(THIRD_LAW, &me, None, 100, l.mods.mult) + extra;
                            self.damage(inp, out, &f, amount, Kind::Phys, l.combo);
                            let mut m = l.mods.clone();
                            m.traits[THIRD_LAW] = 0;
                            self.strike(inp, out, THIRD_LAW, &m, l.combo, &f, 0, (me.x, me.y), (me.x, me.y));
                            self.shove(inp, &f, dx, dy, 8000, l.combo, 6);
                        }
                        Self::fx(out, format!("im{THIRD_LAW}"), me.x, me.y, None, 24);
                        l.done = true;
                    }
                }
                Body::Delayed { at, target, amount } => {
                    if tick >= *at {
                        if let Some(f) = Self::get(inp, *target).cloned() {
                            self.damage(inp, out, &f, *amount, Kind::True, l.combo);
                            Self::fx(out, format!("im{EMC2}"), f.x, f.y, None, 24);
                        }
                        l.done = true;
                    }
                }
                Body::Shove { target, dx, dy, left } => {
                    if let Some(f) = Self::get(inp, *target) {
                        let (nx, ny) = (f.x + *dx, f.y + *dy);
                        if !(inp.wall)(nx, ny) {
                            out.push(Cmd::SetPos { target: f.id, x: nx, y: ny });
                        } else {
                            *left = 1;
                        }
                    }
                    *left = left.saturating_sub(1);
                    if *left == 0 {
                        l.done = true;
                    }
                }
                Body::Echo { at, aim, ox, oy, .. } => {
                    if tick >= *at {
                        let (a, o) = (*aim, (*ox, *oy));
                        Self::fx(out, "twin".into(), o.0, o.1, None, 20);
                        self.launch(inp, out, l.skill, a, l.mods.clone(), l.combo, o);
                        l.done = true;
                    }
                }
            }
            self.live[i] = l;
        }
        self.live.retain(|l| !l.done);
        // Bounded: a long fight cannot accumulate unbounded bodies.
        if self.live.len() > 96 {
            let extra = self.live.len() - 96;
            self.live.drain(0..extra);
        }
    }
    fn split(&mut self, out: &mut Vec<Cmd>, mods: &Mods, combo: u32, at: (i64, i64), dir: (i64, i64), me: &Fighter) {
        // Refraction: closer impacts spread wider (25° near, 10° far).
        let d = dist(me.x, me.y, at.0, at.1);
        let spread = (2500 - d * 1500 / 80000).clamp(1000, 2500);
        let focus = mods.fx(Fx::Focus).is_some();
        let n = mods.has(PRISM).max(1);
        if focus {
            self.spawn_ray(PRISM, combo, Mods::plain(mods.mult * 2 * 20 * 3 * n / 100), at, (at.0 + dir.0 * 4, at.1 + dir.1 * 4), 40000);
            Self::fx(out, format!("pk{PRISM}_a{}", heading(dir.0, dir.1)), at.0, at.1, None, 8);
            return;
        }
        for k in -1..=1i64 {
            for _ in 0..n {
                let (rx, ry) = (dir.0 - dir.1 * k * spread / 10000, dir.1 + dir.0 * k * spread / 10000);
                self.spawn_ray(PRISM, combo, Mods::plain(mods.mult * 20 / 100), at, (at.0 + rx * 4, at.1 + ry * 4), 40000);
            }
        }
        Self::fx(out, format!("split_a{}", heading(dir.0, dir.1)), at.0, at.1, None, 12);
    }
    fn detonate(&mut self, inp: &Input, out: &mut Vec<Cmd>, _combo: u32) {
        let mut burst: BTreeMap<usize, i64> = BTreeMap::new();
        for l in &mut self.live {
            if let Body::Dot { target, a, n, .. } = &l.body {
                let rest: i64 = (*n..5).map(|k| *a >> k).sum();
                *burst.entry(*target).or_default() += rest;
                l.done = true;
            }
        }
        for (id, amount) in burst {
            if let Some(f) = Self::get(inp, id).cloned() {
                self.damage(inp, out, &f, amount, Kind::Magic, 0);
                Self::fx(out, format!("im{HALF_LIFE}"), f.x, f.y, None, 24);
            }
        }
    }

    // ---------------------------------------------------------------- HUD

    fn hud(&mut self, inp: &Input, out: &mut Vec<Cmd>, me: &Fighter) {
        let ready = self.ready_charges(inp.tick);
        let reserved = self.reserve.len();
        // Progress (0..3) of the next charge to return.
        let next = self.charges.iter().filter(|t| **t > inp.tick).min().copied();
        let p = next.map_or(3, |t| 3 - ((t - inp.tick) * 4 / (CHARGE_TICKS + 1)).min(3));
        let key = (ready, reserved, p, self.persona);
        if key != self.hud || inp.tick >= self.hud_until {
            self.hud = key;
            self.hud_until = inp.tick + 8;
            Self::fx(out, format!("ch_{}_{}_{}", ready.min(5), reserved.min(4), p), me.x, me.y, None, 8);
        }
        let queue: Vec<usize> = self.reserve.iter().map(|s| s.skill).collect();
        if queue != self.queue_drawn || (!queue.is_empty() && inp.tick % 30 == 0) {
            for (slot, s) in queue.iter().enumerate() {
                Self::fx(out, format!("rv{slot}_{s}"), me.x, me.y, Some(me.id), 30);
            }
            self.queue_drawn = queue;
        }
    }

    // ---------------------------------------------------------------- equation efficiency

    /// How well the cast uses its own equation (per 100), at the moment it was cast.
    pub fn efficiency_of(&self, inp: &Input, skill: usize, aim: Aim, mods: &Mods, me: &Fighter) -> i64 {
        let target = match aim {
            Aim::Unit(id) => Self::get(inp, id),
            Aim::Point(x, y) => Self::enemies(inp, me.team).min_by_key(|f| (d2(f.x, f.y, x, y), f.id)),
        };
        let (tx, ty) = match aim {
            Aim::Unit(_) => target.map_or((me.x, me.y), |t| (t.x, t.y)),
            Aim::Point(x, y) => (x, y),
        };
        let d = dist(me.x, me.y, tx, ty);
        let e = &EQS[skill];
        let near = |r: i64| Self::enemies(inp, me.team).filter(|f| d2(f.x, f.y, tx, ty) <= sq(r)).count() as i64;
        let marks = target.map_or(0, |t| self.marks_on(t.id, inp.tick)) as i64 + mods.has(APPLE) + mods.has(TIME_FIELD) + mods.has(HALF_LIFE).min(1);
        match skill {
            APPLE => 30 + 70 * d.min(e.range) / e.range.max(1),
            PRISM => if near(25000) >= 2 { 100 } else { 50 },
            ORBIT => ((100 * 625_000_000 / sq(d.max(1)).max(1)).clamp(50, 160) * 100 / 160) as i64,
            CRADLE => if near(30000) >= 2 { 100 } else { 40 },
            THIRD_LAW => (100 * self.recent_loss(me.id, me.hp, 120, inp.tick) / (me.max_hp / 10).max(1)).clamp(10, 100),
            CALCULUS => target.map_or(10, |t| (100 * self.recent_loss(t.id, t.hp, 120, inp.tick) / (t.max_hp / 10).max(1)).clamp(10, 100)),
            RADIUM | WELL => (40 + 30 * near(e.radius)).min(100),
            ALPHA => if d <= e.range { 100 } else { 20 },
            BETA => if target.is_some_and(|t| self.speed(t.id) > 300) { 100 } else { 70 },
            GAMMA => (40 + 30 * near(30000)).min(100),
            HALF_LIFE => 90,
            CHAIN => if self.in_zone(RADIUM, tx, ty) || mods.fx(Fx::Critical).is_some() { 100 } else { (40 + 20 * near(25000)).min(80) },
            TIME_FIELD => target.map_or(30, |t| (30 + 70 * self.speed(t.id).min(1200) / 1200).min(100)),
            EMC2 => {
                let heaviest = Self::enemies(inp, me.team).map(|f| f.max_hp).max().unwrap_or(1).max(1);
                target.map_or(30, |t| 100 * t.max_hp / heaviest)
            }
            RING => 60 + 40 * i64::from(self.live.iter().any(|l| matches!(l.body, Body::Ray { .. }))),
            TWIN => 60,
            PHOTO => if marks >= 2 { 100 } else { 20 },
            _ => 50,
        }
    }
    fn efficiency(&mut self, inp: &Input, skill: usize, aim: Aim, mods: &Mods, me: &Fighter) {
        let e = self.efficiency_of(inp, skill, aim, mods, me);
        self.stats.efficiency += e;
        self.stats.measured += 1;
        if e < 35 {
            self.stats.misplays += 1;
            if std::env::var("UNIFIED_MISPLAY").is_ok() {
                eprintln!("rank {} misplay {} eff {e} reserve {:?}", self.rank, EQS[skill].name, self.reserve.iter().map(|s| EQS[s.skill].key).collect::<Vec<_>>());
            }
        }
    }

    // ---------------------------------------------------------------- the planner

    /// Where to aim `skill` at `t`: lead by its observed motion according to rank.
    pub fn lead(&self, skill: usize, me: &Fighter, t: &Fighter) -> (i64, i64) {
        let e = &EQS[skill];
        let d = dist(me.x, me.y, t.x, t.y);
        let ticks = match e.shape {
            Shape::Lob => e.flight as i64,
            Shape::Line | Shape::Bolt | Shape::Planet => d / e.speed.max(3000),
            Shape::Zone => 20,
            _ => 0,
        };
        let (vx, vy) = self.velocity(t.id);
        let k = LEAD[self.rank];
        (t.x + vx * ticks * k / 100, t.y + vy * ticks * k / 100)
    }
    /// Estimated value of casting `skill` at `t` with `mods` (damage-equivalent points).
    pub fn value(&self, inp: &Input, me: &Fighter, skill: usize, t: &Fighter, mods: &Mods) -> i64 {
        let (real, naive) = (self.facts(inp, me, skill, t, false), self.facts(inp, me, skill, t, true));
        self.judge(&real, &naive, me, t, mods, inp.tick)
    }
    /// What the athlete believes: the real estimate blended with the naive one by rank.
    fn judge(&self, real: &Facts, naive: &Facts, me: &Fighter, t: &Fighter, mods: &Mods, tick: usize) -> i64 {
        let u = UNDERSTANDING[self.rank];
        let r = self.eval(real, me, t, mods, tick);
        if u >= 100 || r == i64::MIN / 4 {
            return r;
        }
        (r * u + self.eval(naive, me, t, mods, tick) * (100 - u)) / 100
    }
    /// Everything about (skill, target) the estimate needs, measured once. `naive` ignores every
    /// condition of the equations (distance, marks, motion, grouping): what a beginner believes.
    fn facts(&self, inp: &Input, me: &Fighter, skill: usize, t: &Fighter, naive: bool) -> Facts {
        let e = &EQS[skill];
        let tick = inp.tick;
        let near = |r: i64| if naive { 1 } else { Self::enemies(inp, me.team).filter(|f| f.id != t.id && d2(f.x, f.y, t.x, t.y) <= sq(r)).count() as i64 };
        let ally_loss = if e.shape == Shape::Ally {
            inp.fighters
                .iter()
                .filter(|a| a.team == me.team && a.hp > 0 && a.champion && d2(a.x, a.y, me.x, me.y) <= sq(e.range) && self.twin_lock.get(&a.id).is_none_or(|u| *u <= tick))
                .map(|a| self.recent_loss(a.id, a.hp, HISTORY_TICKS, tick).min(a.max_hp * 20 / 100))
                .max()
                .unwrap_or(0)
        } else {
            0
        };
        let used: usize = self.limits.get(&t.id).map_or(0, |w| w.0.iter().filter(|(at, _)| tick.saturating_sub(*at) < LIMIT_TICKS).map(|(_, n)| *n).sum());
        let mut lent = [0i64; N];
        for (k, l) in lent.iter_mut().enumerate() {
            *l = self.raw(k, me, Some(t), FUSE_PCT, 100);
        }
        Facts {
            skill,
            naive,
            d_real: dist(me.x, me.y, t.x, t.y),
            d: if naive { e.range * 7 / 10 } else { dist(me.x, me.y, t.x, t.y) },
            blocked: self.blocked(inp, (me.x, me.y), (t.x, t.y)),
            raw: self.raw(skill, me, Some(t), 100, 100),
            n20: near(20000),
            n25: near(25000),
            n30: near(30000),
            n_radius: near(e.radius),
            marks_now: if naive { 2 } else { self.marks_on(t.id, tick) as i64 },
            moving: naive || self.speed(t.id) > 300,
            speed: self.speed(t.id).min(1200),
            in_radium: !naive && self.in_zone(RADIUM, t.x, t.y),
            loss: self.recent_loss(t.id, t.hp, 120, tick),
            my_loss: self.recent_loss(me.id, me.hp, 120, tick),
            threat: Self::enemies(inp, me.team).filter(|f| d2(f.x, f.y, me.x, me.y) <= sq(25000)).count() as i64,
            rays: self.live.iter().any(|l| matches!(l.body, Body::Ray { .. })),
            ally_loss,
            left: (t.max_hp * LIMIT_PCT / 100 - used as i64).max(0),
            lent,
        }
    }
    fn eval(&self, f: &Facts, me: &Fighter, t: &Fighter, mods: &Mods, tick: usize) -> i64 {
        let skill = f.skill;
        let e = &EQS[skill];
        let reach = if mods.has(ALPHA) > 0 { e.range * 70 / 100 } else { e.range };
        let ally = e.shape == Shape::Ally;
        if !ally && e.shape != Shape::Caster && f.d_real > reach + e.radius {
            return i64::MIN / 4;
        }
        if !ally && e.shape != Shape::Caster && e.shape != Shape::Lob && !e.walls && mods.has(GAMMA) == 0 && f.blocked {
            return i64::MIN / 4;
        }
        let (raw, d) = (f.raw, f.d);
        let added = i64::from(mods.has(APPLE) > 0 || skill == APPLE) + i64::from(mods.has(TIME_FIELD) > 0 || skill == TIME_FIELD) + i64::from(mods.has(HALF_LIFE) > 0) + i64::from(mods.has(CALCULUS) > 0);
        let marks = f.marks_now + added;
        let mut v = match skill {
            APPLE => raw * (100 + 60 * d.min(e.range) / e.range) / 100 + 25,
            PRISM => raw + raw * 120 / 100 * f.n25.min(2) / 2,
            ORBIT => raw * 3 * (100 * 625_000_000 / sq(d.max(1)).max(1)).clamp(50, 160) as i64 / 100 + 15,
            CRADLE => raw + if f.n30 > 0 { raw + 20 } else { 0 },
            THIRD_LAW => {
                if me.hp * 100 < me.max_hp * 60 && f.threat > 0 { 60 + me.ap * 30 / 100 + raw * f.threat + f.my_loss * 30 / 100 } else { raw * f.threat / 2 }
            }
            CALCULUS => raw + (f.loss * 35 / 100).min(t.max_hp * 8 / 100 + 60) * 70 / 100,
            RADIUM => raw * (6 + 5 * f.n_radius) * (if !f.moving { 2 } else { 1 }) / 2,
            BETA => raw * if f.moving { 110 } else { 100 } / 100,
            GAMMA => raw * (100 + 85 * f.n20.min(3)) / 100,
            HALF_LIFE => raw * 2,
            CHAIN => {
                let k = if f.in_radium || mods.fx(Fx::Critical).is_some() { 100 } else { 70 };
                raw + raw * k / 100 * f.n25.min(4)
            }
            TIME_FIELD => 20 + if f.naive { 25 } else { 30 * f.speed / 1200 } + 10 * f.n_radius,
            RING => raw * (1 + f.n_radius) + 20 * i64::from(f.rays),
            TWIN => 0,
            WELL => raw * 4 + 25 * f.n_radius,
            PHOTO => if marks >= 2 { raw * (100 + 10 * (marks - 2)) / 100 } else { raw * 15 / 100 },
            _ => raw,
        };
        if ally {
            // Twin Paradox: the ally (or Unified) that lost the most in the last 3 s.
            v = if f.ally_loss * 100 >= me.max_hp * 12 { f.ally_loss * 130 / 100 } else { 0 };
        }
        // Each trait's estimated contribution, plus the power each reserved skill lends.
        let r = raw.max(1);
        v += (0..N).map(|k| mods.has(k) * f.lent[k]).sum::<i64>();
        v += mods.has(APPLE) * (r * 30 / 100 + 15);
        v += mods.has(PRISM) * r * 60 / 100 * i64::from(f.n20 > 0 || e.shape == Shape::Zone);
        v += mods.has(ORBIT) * r * 50 / 100;
        v += mods.has(CRADLE) * r * 70 / 100 * i64::from(f.n30 > 0);
        v += mods.has(THIRD_LAW) * 12;
        v += mods.has(CALCULUS) * f.loss * 20 / 100;
        v += mods.has(RADIUM) * r * 40 / 100;
        v += if mods.has(ALPHA) > 0 { v * 40 * mods.has(ALPHA) / 100 } else { 0 };
        v += mods.has(BETA) * r * 15 / 100 * i64::from(f.moving);
        v += mods.has(GAMMA) * (r * 50 / 100 * f.n20.min(2) + 20 * i64::from(f.blocked));
        v += mods.has(HALF_LIFE) * r * 30 / 100;
        v += mods.has(CHAIN) * r * 70 / 100 * f.n25.min(2);
        v += mods.has(TIME_FIELD) * 18;
        v += mods.has(EMC2) * t.max_hp * 2 / 100;
        v += mods.has(RING) * r * 15 / 100;
        v += mods.has(TWIN) * v * 60 / 100;
        v += mods.has(WELL) * 14;
        if mods.has(PHOTO) > 0 && marks >= 2 {
            v += v * 80 / 100;
        }
        if let Some(s) = mods.sig {
            v += v * SIGNATURES[s].bonus / 100 + 40 + 15 * SIGNATURES[s].fx.len() as i64;
        }
        // Three scientists in one cast: a Unification (when it is ready).
        let mut sciences = [false; 3];
        sciences[e.science] = true;
        for k in 0..N {
            if mods.has(k) > 0 {
                sciences[EQS[k].science] = true;
            }
        }
        if sciences.iter().all(|x| *x) && tick >= self.unify_at {
            v += v * 30 / 100 + 40;
        }
        let mut v = v * mods.mult / 100;
        // Damage past the per-target limit is wasted (a little spills onto others).
        if !f.naive && t.champion && v > f.left {
            v = f.left + (v - f.left) * 30 / 100;
        }
        if !t.champion {
            v /= 4;
        }
        // Low HP targets are worth finishing.
        if t.champion && t.hp * 4 < t.max_hp {
            v = v * 130 / 100;
        }
        v
    }
    fn noisy(&mut self, v: i64) -> i64 {
        let n = NOISE[self.rank];
        if n == 0 || v <= 0 {
            return v;
        }
        let r = self.random(2 * n as u64 + 1) as i64 - n;
        v * (100 + r) / 100
    }
    /// Pick the next combo when idle; then walk it one step at a time.
    fn think(&mut self, inp: &Input, out: &mut Vec<Cmd>) {
        let tick = inp.tick;
        if !self.plan.is_empty() {
            if tick < self.next_step {
                return;
            }
            let p = self.plan.remove(0);
            let ok = if p.cast { self.cast(inp, out, p.skill, p.aim) } else { self.reserve(inp, out, p.skill) };
            if !ok {
                self.plan.clear();
                return;
            }
            let jitter = self.random(7) as usize;
            self.next_step = tick + STEP_TICKS[self.rank] + jitter;
            return;
        }
        if !tick.is_multiple_of(6) || self.ult_warn.is_some() {
            return;
        }
        let Some(me) = self.me(inp).cloned() else { return };
        // The ultimate: replay the last combo once one exists and a champion is in reach.
        if inp.ult_unlocked && tick >= self.ult_at && self.last_combo.is_some() {
            if let Some(t) = Self::enemies(inp, me.team).filter(|f| f.champion && d2(f.x, f.y, me.x, me.y) <= sq(70000)).min_by_key(|f| (f.hp, f.id)).cloned() {
                if self.rank >= 2 || self.random(4) == 0 {
                    let (x, y) = self.lead(EMC2, &me, &t);
                    let aim = if EQS[*self.last_combo.as_ref().map_or(&EMC2, |c| c.steps.last().unwrap_or(&EMC2))].shape == Shape::Zone { Aim::Point(x, y) } else { Aim::Unit(t.id) };
                    if self.ult(inp, out, aim) {
                        self.reason = "Theory of Everything";
                        return;
                    }
                }
            }
        }
        let charges = self.ready_charges(tick);
        if charges == 0 || !self.reserve.is_empty() {
            return;
        }
        if tick < self.next_think.0 && charges == self.next_think.1 {
            return;
        }
        self.next_think = (tick + 18, charges);
        self.stats.searches += 1;
        let targets: Vec<Fighter> = Self::enemies(inp, me.team).filter(|f| d2(f.x, f.y, me.x, me.y) <= sq(130000)).cloned().collect();
        if targets.is_empty() && self.recent_loss(me.id, me.hp, HISTORY_TICKS, tick) == 0 {
            return;
        }
        let urgent = me.hp * 100 < me.max_hp * 40;
        let keep = if urgent { 0 } else { KEEP[self.rank] };
        // Plans are built for a full bar; the ones that fit today's charges are playable now.
        let budget = CHARGES.saturating_sub(keep).min(PLAN_STEPS[self.rank]);
        let playable = charges.saturating_sub(keep).min(budget);
        if playable == 0 {
            return;
        }
        let usable: Vec<usize> = (0..N).filter(|s| self.usable(*s, tick)).collect();
        // Every option: (score, steps, final skill, target).
        let mut options: Vec<(i64, Vec<usize>, usize, Fighter)> = vec![];
        let ally_target = inp
            .fighters
            .iter()
            .filter(|a| a.team == me.team && a.hp > 0 && a.champion && d2(a.x, a.y, me.x, me.y) <= sq(EQS[TWIN].range))
            .max_by_key(|a| (self.recent_loss(a.id, a.hp, HISTORY_TICKS, tick), a.id))
            .cloned();
        // Facts are measured once per (skill, target) for the whole search; a naive copy only
        // matters below full understanding.
        let mut facts: BTreeMap<(usize, usize), (Facts, Facts)> = BTreeMap::new();
        let full = UNDERSTANDING[self.rank] >= 100;
        let mut measure = |core: &Core, s: usize, t: &Fighter| -> (Facts, Facts) {
            facts
                .entry((s, t.id))
                .or_insert_with(|| {
                    let real = core.facts(inp, &me, s, t, false);
                    let naive = if full { real.clone() } else { core.facts(inp, &me, s, t, true) };
                    (real, naive)
                })
                .clone()
        };
        let pool_for = |s: usize| -> Vec<Fighter> {
            match EQS[s].shape {
                Shape::Ally => ally_target.iter().cloned().collect(),
                Shape::Caster => targets.iter().min_by_key(|f| (d2(f.x, f.y, me.x, me.y), f.id)).into_iter().cloned().collect(),
                _ => targets.clone(),
            }
        };
        // Every usable skill alone, at its best target.
        let mut finals: Vec<(i64, usize, Fighter)> = vec![];
        for &s in &usable {
            let plain = self.compose(&[], s as u8, false);
            let best = pool_for(s)
                .iter()
                .map(|t| {
                    let (fr, fnv) = measure(self, s, t);
                    (self.judge(&fr, &fnv, &me, t, &plain, tick), t.clone())
                })
                .max_by_key(|(v, t)| (*v, std::cmp::Reverse(t.id)));
            if let Some((base, t)) = best.filter(|(v, _)| *v > 0) {
                options.push((base, vec![s], s, t.clone()));
                finals.push((base, s, t));
            }
        }
        // Reserved traits in front of the most promising finals: a beam over 1..budget-1 traits.
        finals.sort_by_key(|(v, s, _)| (std::cmp::Reverse(*v), *s));
        finals.truncate(FINALS);
        let insight = STEP_TICKS[self.rank] <= INSIGHT_TICKS;
        for (base, s, t) in finals {
            let (fr, fnv) = measure(self, s, &t);
            // Beam entries keep their reserved steps in a fixed array: no allocation per candidate.
            let mut beam: Vec<(i64, [usize; MAX_RESERVE], usize)> = vec![(base, [0; MAX_RESERVE], 0)];
            let blank = Step { skill: 0, at: tick, insight };
            for _depth in 1..budget {
                let mut next: Vec<(i64, [usize; MAX_RESERVE], usize)> = Vec::with_capacity(beam.len() * usable.len());
                for (_, pre, n) in &beam {
                    let mut reserved = [blank; MAX_RESERVE];
                    for (i, k) in pre[..*n].iter().enumerate() {
                        reserved[i].skill = *k;
                    }
                    for &a in &usable {
                        reserved[*n].skill = a;
                        let mut mods = self.compose(&reserved[..*n + 1], s as u8, insight);
                        // Signatures only count when the athlete knows them; otherwise the bonus is luck.
                        if mods.sig.is_some_and(|g| SIGNATURES[g].steps.len() > KNOWS[self.rank]) {
                            mods.mult -= mods.sig.map_or(0, |g| SIGNATURES[g].bonus);
                            mods.sig = None;
                        }
                        let mut steps = *pre;
                        steps[*n] = a;
                        next.push((self.judge(&fr, &fnv, &me, &t, &mods, tick), steps, *n + 1));
                    }
                }
                next.sort_by(|(v, a, n), (w, b, m)| w.cmp(v).then_with(|| a[..*n].cmp(&b[..*m])));
                // Orders matter in play, but not to the estimate: keep one order per set of steps.
                let mut seen: Vec<[usize; MAX_RESERVE]> = vec![];
                next.retain(|(_, steps, n)| {
                    let mut k = [usize::MAX; MAX_RESERVE];
                    k[..*n].copy_from_slice(&steps[..*n]);
                    k[..*n].sort();
                    if seen.contains(&k) {
                        false
                    } else {
                        seen.push(k);
                        true
                    }
                });
                next.truncate(BEAM[self.rank]);
                for (v, steps, n) in &next {
                    let mut all = steps[..*n].to_vec();
                    all.push(s);
                    options.push((*v, all, s, t.clone()));
                }
                beam = next;
            }
        }
        let beam_options = options.len();
        // Known signatures whose skills are all ready, even if the beam missed them.
        for (g, sig) in SIGNATURES.iter().enumerate() {
            let len = sig.steps.len();
            if len > KNOWS[self.rank] || len > budget || sig.steps.contains(&ULT_STEP) {
                continue;
            }
            if !sig.steps.iter().all(|k| self.usable(usize::from(*k), tick)) {
                continue;
            }
            let s = usize::from(sig.steps[len - 1]);
            let reserved: Vec<Step> = sig.steps[..len - 1].iter().map(|k| Step { skill: usize::from(*k), at: tick, insight }).collect();
            let mods = self.compose(&reserved, s as u8, insight);
            let best = pool_for(s)
                .iter()
                .map(|t| {
                    let (fr, fnv) = measure(self, s, t);
                    (self.judge(&fr, &fnv, &me, t, &mods, tick), t.clone())
                })
                .max_by_key(|(v, t)| (*v, std::cmp::Reverse(t.id)));
            if let Some((v, t)) = best {
                let _ = g;
                options.push((v, sig.steps.iter().map(|k| usize::from(*k)).collect(), s, t));
            }
        }
        if options.is_empty() {
            return;
        }
        if std::env::var("UNIFIED_DEBUG").is_ok() && tick % 120 == 0 {
            let best = options.iter().map(|o| o.0 * 100 / (100 + 85 * (o.1.len() as i64 - 1))).max().unwrap_or(0);
            let sigs: Vec<(i64, Vec<usize>)> = options[beam_options..].iter().map(|o| (o.0 * 100 / (100 + 85 * (o.1.len() as i64 - 1)), o.1.clone())).collect();
            let mut top: Vec<(i64, Vec<usize>)> = options.iter().map(|o| (o.0 * 100 / (100 + 85 * (o.1.len() as i64 - 1)), o.1.clone())).collect();
            top.sort_by_key(|x| std::cmp::Reverse(x.0));
            top.truncate(5);
            let _ = sigs;
            eprintln!("tick {tick} rank {} charges {charges} best/charge {best} top {top:?}", self.rank);
        }
        // Charges are the currency: score each option per charge (an extra step costs 0.85 of one).
        let mut scored: Vec<(i64, usize)> = vec![];
        for (i, o) in options.iter().enumerate() {
            let v = self.noisy(o.0);
            scored.push((v * 100 / (100 + 85 * (o.1.len() as i64 - 1)), i));
        }
        scored.sort_by_key(|(s, i)| (std::cmp::Reverse(*s), *i));
        let best_full = scored.first().map_or(0, |x| x.0);
        scored.retain(|(_, i)| options[*i].1.len() <= playable);
        if scored.is_empty() {
            self.reason = "Banking charges for a combo";
            return;
        }
        let mut pick = scored[0].1;
        if self.random(100) < MISTAKE[self.rank] {
            pick = scored[self.random(scored.len() as u64) as usize].1;
        }
        let (score, steps, s, t) = options[pick].clone();
        if score <= 0 {
            return;
        }
        // The top ranks bank charges for a bigger combo instead of trickling them out on weak casts.
        let per_charge = scored.iter().find(|(_, i)| *i == pick).map_or(0, |x| x.0);
        if !urgent && charges < CHARGES && t.hp * 3 > t.max_hp && per_charge * 100 < best_full * HOLD[self.rank] {
            self.reason = "Banking charges for a combo";
            return;
        }
        // Patience: a strong athlete holds a weak, inefficient cast for a better moment.
        let e = self.efficiency_of(inp, s, Aim::Unit(t.id), &self.compose(&[], s as u8, false), &me);
        if self.rank >= 4 && steps.len() == 1 && e < 35 && charges < CHARGES && !urgent {
            self.reason = "Waiting for a better moment";
            return;
        }
        let aim = match EQS[s].shape {
            Shape::Zone | Shape::Lob | Shape::Planet => {
                let (x, y) = self.lead(s, &me, &t);
                Aim::Point(x, y)
            }
            _ => Aim::Unit(t.id),
        };
        self.reason = if steps.len() >= 3 { "Building a combo" } else if steps.len() == 2 { "Pairing two equations" } else { "Single equation" };
        self.plan = steps
            .iter()
            .enumerate()
            .map(|(i, k)| Planned { skill: *k, aim, cast: i + 1 == steps.len() })
            .collect();
        self.next_step = tick;
        self.think(inp, out);
    }
}

/// Unit circle in 48 steps, scaled by 10000.
fn circle(a: i64) -> (i64, i64) {
    const C: [(i64, i64); 12] = [
        (10000, 0),
        (9914, 1305),
        (9659, 2588),
        (9239, 3827),
        (8660, 5000),
        (7934, 6088),
        (7071, 7071),
        (6088, 7934),
        (5000, 8660),
        (3827, 9239),
        (2588, 9659),
        (1305, 9914),
    ];
    let a = a.rem_euclid(48) as usize;
    let (q, k) = (a / 12, a % 12);
    let (x, y) = C[k];
    match q {
        0 => (x, y),
        1 => (-y, x),
        2 => (-x, -y),
        _ => (y, -x),
    }
}
