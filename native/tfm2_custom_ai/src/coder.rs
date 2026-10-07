//! Round 101: the Coder ("Root"), a mastery champion who fights by writing code (Rian: "hard to master, a significant
//! gap between each" rank; "make each function harder to write the lower the rank ... the design actually writes
//! what the function does"; plus CPU, RAM, storage, heat, languages and overclocking).
//!
//! He picks a function, types its real code (coder_code.rs, generated with the art from Claude outputs/coder/
//! coder_functions.py) character by character at his rank's speed; each character can be a typo (more on symbols,
//! more in C++ / Rust, more when hot or overclocked). Typos are SyntaxErrors (the compile fails, he retypes the line)
//! or logic bugs that compile and misbehave (wrong target, off-by-one, infinite loop, null reference, sign flip; C++
//! also segfaults and leaks). He reviews before compiling and catches each typo at his rank's NOTICE; Rust's compiler
//! catches most logic bugs too, at the price of a long compile. A compiled function joins his program (5 slots) and
//! runs on its own whenever its trigger holds, checked every CLOCK ticks. The rig: a 3.0 GHz CPU (load, throttled
//! when hot, 3.9 GHz overclocked), 16 GB of RAM (processes that last hold it; over the top is OOM), 8 storage slots
//! (a saved function reloads in 1 s, bugs and all) and heat (85 C throttles, 100 C blue-screens: stunned and every
//! unsaved function lost). Every rank can try every function; rank sets the speed, typo rate, review, judgement of
//! his own ability, clock and how well he rides the overclock.
//!
//! His three skills are never cast by the game (like Scribble's): S1 Overclock, S2 debug and the ult are his brain's.
//! Phase 1: 10 functions in Python, C++ and Rust; the AI-copilot ult, JavaScript, Assembly and tiers 4-5 come next.

use crate::coder_code::{FUNCS, LANGS};
use crate::mastery::{signature, Book, Record};
use crate::{champions, d2, sq, timed, walls, Champ};
use mod_api_stable::{AttackTypeV1, BuffV1, CcV1, StablePassive, StableSim};

pub static BOOK: Book = Book::new("coder");

pub const RANK_NAMES: [&str; 8] = ["Script Kiddie", "Intern", "Junior", "Developer", "Senior", "Staff", "Architect", "Root"];
pub const ROOT: usize = 7;
const P: &str = "tfm2_custom_coder_";

// ------------------------------------------------------------------ the rank tables (index 7 = Root #10; then #1)

/// Typing speed, characters a second x100 (about WPM x 12 / 100).
const CPS100: [usize; 8] = [300, 500, 700, 900, 1200, 1500, 1900, 2200];
const CPS100_TOP: usize = 3000;
/// Typos per 10000 plain characters (symbols count more, see coder_code.rs risk).
const TYPO: [usize; 8] = [420, 310, 220, 150, 95, 55, 28, 14];
const TYPO_TOP: usize = 0;
/// % chance to catch each typo when reviewing.
const NOTICE: [usize; 8] = [15, 30, 45, 60, 75, 87, 94, 97];
const NOTICE_TOP: usize = 100;
/// % of the truth he sees about his own ability (low: he thinks he's faster and cleaner than he is: overreach).
const AWARE: [usize; 8] = [20, 35, 50, 62, 75, 85, 93, 96];
const AWARE_TOP: usize = 100;
/// Ticks between program checks at 3.0 GHz.
const CLOCK: [usize; 8] = [60, 40, 30, 24, 20, 15, 12, 11];
const CLOCK_TOP: usize = 10;
/// % noise on what he reads of the fight (health), unless a scan is up.
const READ: [usize; 8] = [30, 22, 16, 11, 7, 4, 2, 1];
const READ_TOP: usize = 0;
/// % judgement: the right language, saving clean code, not running into an OOM.
const IQ: [usize; 8] = [10, 25, 40, 55, 70, 85, 93, 97];
const IQ_TOP: usize = 100;
/// Overclock: the heat (C) he turns it off at, and how late (ticks) he reacts.
const OC_OFF: [usize; 8] = [103, 101, 98, 95, 92, 90, 89, 89];
const OC_OFF_TOP: usize = 88;
const OC_LAG: [usize; 8] = [60, 45, 30, 20, 12, 8, 4, 2];
const OC_LAG_TOP: usize = 0;

/// A table's value at `rank`; the Root (Top 10) interpolates from #10 (the table's last entry) to #1 (`top`).
fn tv(table: &[usize; 8], top: usize, rank: usize, root: Option<usize>) -> usize {
    if rank < ROOT { return table[rank]; }
    let p = root.unwrap_or(10).clamp(1, 10);
    let (a, b) = (table[ROOT] as i64, top as i64);
    (a + (b - a) * (10 - p as i64) / 9) as usize
}

// ------------------------------------------------------------------ languages

#[derive(Clone, Copy)]
struct Lang {
    /// typo rate x100
    typo: usize,
    /// % of typos that are syntax errors (the rest compile into logic bugs)
    syntax: usize,
    /// compile ticks (Rust's grows at low rank: fighting the borrow checker)
    compile: usize,
    /// run power x100
    power: usize,
    /// heat per run (C x100)
    heat: i32,
    /// RAM x100 of the function's base
    ram: usize,
    /// % of logic bugs the compiler catches
    catch: usize,
    /// logic bug weights: wrong target, off-by-one, infinite loop, null ref, sign flip, segfault, leak
    bugs: [usize; 7],
}
const LANG: [Lang; 3] = [
    Lang { typo: 80, syntax: 50, compile: 0, power: 80, heat: 150, ram: 150, catch: 0, bugs: [20, 25, 10, 35, 10, 0, 0] },
    Lang { typo: 120, syntax: 60, compile: 60, power: 130, heat: 400, ram: 80, catch: 0, bugs: [15, 20, 10, 0, 10, 25, 20] },
    Lang { typo: 110, syntax: 70, compile: 150, power: 120, heat: 250, ram: 70, catch: 80, bugs: [25, 35, 20, 0, 20, 0, 0] },
];
pub const PY: usize = 0;
pub const CPP: usize = 1;
pub const RUST: usize = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bug {
    WrongTarget,
    OffByOne,
    InfiniteLoop,
    NullRef,
    SignFlip,
    Segfault,
    Leak,
}
const BUGS: [Bug; 7] = [Bug::WrongTarget, Bug::OffByOne, Bug::InfiniteLoop, Bug::NullRef, Bug::SignFlip, Bug::Segfault, Bug::Leak];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Typo {
    Syntax,
    Logic(Bug),
}

// ------------------------------------------------------------------ the functions

const NF: usize = 10;
const PING: usize = 0;
const SHIELD: usize = 1;
const HEAL: usize = 2;
const SCAN: usize = 3;
const SPRAY: usize = 4;
const BLINK: usize = 5;
const SLOW: usize = 6;
const CACHE: usize = 7;
const CHAIN: usize = 8;
const FIREWALL: usize = 9;

/// (cooldown ticks, CPU load % x100, RAM MB while it lasts, base value, how long it lasts)
const SPEC: [(usize, i32, usize, usize, usize); NF] = [
    (45, 600, 0, 40, 0),            // ping
    (360, 1500, 2000, 30, 180),     // shield
    (300, 1500, 0, 25, 0),          // heal
    (600, 1000, 1000, 15, 180),     // scan
    (180, 1800, 0, 30, 0),          // spray
    (600, 1200, 0, 20, 0),          // blink
    (300, 1000, 1500, 25, 120),     // slow
    (480, 0, 0, 10, 0),             // cache
    (300, 2200, 0, 45, 0),          // chain
    (600, 2500, 4000, 40, 240),     // firewall
];
/// The language a perfect judge writes each function in: short utility in Python, the hot damage loop in C++, the
/// long-lived wall in Rust.
const IDEAL: [usize; NF] = [PY, PY, PY, PY, PY, PY, PY, PY, CPP, RUST];
const SLOTS: usize = 5;
const STORAGE: usize = 8;
const RAM_MB: usize = 16_000;

fn lines(f: usize, lang: usize) -> &'static [(usize, usize)] {
    FUNCS[f].2[lang]
}
pub fn chars(f: usize, lang: usize) -> usize {
    lines(f, lang).iter().map(|l| l.0).sum()
}

// ------------------------------------------------------------------ randomness (both simulations roll the same)

#[derive(Clone, Default)]
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        let mut x = self.0.max(1);
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
    /// true with `per` in `of` odds
    fn chance(&mut self, per: usize, of: usize) -> bool {
        (self.next() % of as u64) < per as u64
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n.max(1) as u64) as usize
    }
}

// ------------------------------------------------------------------ typing: one function, line by line

/// What his rank and rig make of typing right now.
#[derive(Clone, Copy)]
struct Knobs {
    cps100: usize,
    typo: usize,
    notice: usize,
    rank: usize,
}

#[derive(Clone, Debug, PartialEq)]
enum Phase {
    Type { line: usize, done: usize },
    Review { left: usize },
    Fix { queue: Vec<usize>, done: usize },
    Compile { left: usize },
    Load { left: usize },
}

#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    None,
    /// a SyntaxError (the line to retype); or a Rust compile error caught a logic bug
    CompileFailed(usize),
    /// compiled: the logic bugs that made it through
    Compiled(Vec<Bug>),
}

#[derive(Clone, Debug)]
struct Typing {
    f: usize,
    lang: usize,
    phase: Phase,
    typos: Vec<(usize, Typo)>,
    /// fixed on review or by the compiler, for the stats
    caught: usize,
    started: usize,
}

impl Typing {
    fn new(f: usize, lang: usize, tick: usize) -> Typing {
        Typing { f, lang, phase: Phase::Type { line: 0, done: 0 }, typos: Vec::new(), caught: 0, started: tick }
    }
    fn reload(f: usize, lang: usize, bugs: &[Bug], tick: usize) -> Typing {
        Typing { f, lang, phase: Phase::Load { left: 60 }, typos: bugs.iter().map(|&b| (0, Typo::Logic(b))).collect(), caught: 0, started: tick }
    }

    /// Where the terminal is: (line, characters typed of it) while typing or fixing.
    fn cursor(&self) -> Option<(usize, usize)> {
        match &self.phase {
            Phase::Type { line, done } => Some((*line, done / 100)),
            Phase::Fix { queue, done } => queue.first().map(|&l| (l, done / 100)),
            _ => None,
        }
    }

    /// Type `n` characters x100 of the current line, rolling a typo at each character crossed. True when it's done.
    fn type_line(&mut self, line: usize, done: &mut usize, k: &Knobs, rng: &mut Rng) -> bool {
        let (len, risk) = lines(self.f, self.lang)[line];
        let before = *done / 100;
        *done += k.cps100 * 100 / 6000;   // chars x100 a tick (60 ticks a second)
        let after = (*done / 100).min(len);
        // per character: typo rate x the line's syntax risk (risk is x10 per character; plain text averages 10)
        let per_char = k.typo * LANG[self.lang].typo / 100 * risk / (10 * len.max(1));
        for _ in before..after {
            if rng.chance(per_char, 10_000) {
                let t = if rng.chance(LANG[self.lang].syntax, 100) { Typo::Syntax } else {
                    let w = LANG[self.lang].bugs;
                    let mut r = rng.below(w.iter().sum());
                    let mut pick = Bug::OffByOne;
                    for (i, &x) in w.iter().enumerate() {
                        if r < x { pick = BUGS[i]; break; }
                        r -= x;
                    }
                    Typo::Logic(pick)
                };
                self.typos.push((line, t));
            }
        }
        after >= len
    }

    /// One tick of typing / reviewing / fixing / compiling.
    fn tick(&mut self, k: &Knobs, rng: &mut Rng) -> Event {
        let n_lines = lines(self.f, self.lang).len();
        let mut phase = std::mem::replace(&mut self.phase, Phase::Review { left: 0 });
        let mut out = Event::None;
        phase = match phase {
            Phase::Type { line, mut done } => {
                if self.type_line(line, &mut done, k, rng) {
                    if line + 1 < n_lines { Phase::Type { line: line + 1, done: 0 } } else {
                        // reading it over goes three times as fast as typing it
                        Phase::Review { left: chars(self.f, self.lang) * 6000 / (k.cps100 * 3).max(1) }
                    }
                } else { Phase::Type { line, done } }
            }
            Phase::Review { left } if left > 0 => Phase::Review { left: left - 1 },
            Phase::Review { .. } => {
                // each typo caught with NOTICE; the caught ones are retyped
                let mut queue: Vec<usize> = Vec::new();
                let mut kept = Vec::new();
                for (l, t) in std::mem::take(&mut self.typos) {
                    if rng.chance(k.notice, 100) {
                        self.caught += 1;
                        if !queue.contains(&l) { queue.push(l); }
                    } else { kept.push((l, t)); }
                }
                // retyping a line rewrites all of it (its other typos go too)
                kept.retain(|(l, _)| !queue.contains(l));
                self.typos = kept;
                if queue.is_empty() { Phase::Compile { left: compile_ticks(self.lang, k.rank) } } else { Phase::Fix { queue, done: 0 } }
            }
            Phase::Fix { mut queue, mut done } => {
                let l = queue[0];
                if self.type_line(l, &mut done, k, rng) {
                    queue.remove(0);
                    if queue.is_empty() { Phase::Compile { left: compile_ticks(self.lang, k.rank) } } else { Phase::Fix { queue, done: 0 } }
                } else { Phase::Fix { queue, done } }
            }
            Phase::Compile { left } if left > 0 => Phase::Compile { left: left - 1 },
            Phase::Compile { .. } => {
                // a SyntaxError fails the build; Rust's compiler also catches most logic bugs
                let catch = LANG[self.lang].catch;
                let mut failed: Option<usize> = self.typos.iter().find(|t| t.1 == Typo::Syntax).map(|t| t.0);
                if failed.is_none() && catch > 0 {
                    for (l, t) in &self.typos {
                        if matches!(t, Typo::Logic(_)) && rng.chance(catch, 100) { failed = Some(*l); break; }
                    }
                }
                match failed {
                    Some(l) => {
                        self.caught += self.typos.iter().filter(|t| t.0 == l).count();
                        self.typos.retain(|t| t.0 != l);
                        out = Event::CompileFailed(l);
                        Phase::Fix { queue: vec![l], done: 0 }
                    }
                    None => {
                        out = Event::Compiled(self.typos.iter().filter_map(|t| if let Typo::Logic(b) = t.1 { Some(b) } else { None }).collect());
                        Phase::Compile { left: 0 }
                    }
                }
            }
            Phase::Load { left } if left > 0 => Phase::Load { left: left - 1 },
            Phase::Load { .. } => {
                out = Event::Compiled(self.typos.iter().filter_map(|t| if let Typo::Logic(b) = t.1 { Some(b) } else { None }).collect());
                Phase::Load { left: 0 }
            }
        };
        self.phase = phase;
        out
    }
}

fn compile_ticks(lang: usize, rank: usize) -> usize {
    let c = LANG[lang].compile;
    if lang == RUST { c * (100 + 15 * (ROOT - rank.min(ROOT))) / 100 } else { c }
}

// ------------------------------------------------------------------ the program and the rig

#[derive(Clone, Debug)]
struct Compiled {
    f: usize,
    lang: usize,
    bugs: Vec<Bug>,
    saved: bool,
}

#[derive(Clone, Debug)]
struct Proc {
    until: usize,
    mb: usize,
    f: usize,
    target: usize,
}

#[derive(Clone, Debug)]
struct Wall {
    a: (i64, i64),
    b: (i64, i64),
    until: usize,
    next: usize,
    dmg: usize,
    flip: bool,
}

#[derive(Clone, Default, Debug)]
pub struct Stats {
    pub shipped: usize,
    pub clean: usize,
    pub bugs: usize,
    pub caught: usize,
    pub syntax_errors: usize,
    pub bsods: usize,
    pub ooms: usize,
    pub runs: usize,
    pub frozen: usize,
    /// ticks spent writing what shipped
    pub write_ticks: usize,
}

#[derive(Clone, Default)]
pub struct Coder {
    started: bool,
    rng: Rng,
    me: Option<usize>,
    team: Option<usize>,
    athlete: Option<usize>,
    rank: Option<usize>,
    root: Option<usize>,
    sig: Option<String>,
    rec: Record,
    alive: bool,
    // writing
    typing: Option<Typing>,
    program: Vec<Compiled>,
    storage: Vec<Compiled>,
    cooldown: [usize; NF],
    next_check: usize,
    next_choice: usize,
    next_debug: usize,
    debug_until: usize,
    last_hp: usize,
    // the rig: heat and load x100, overclock and when he'll notice to turn it off
    heat: i32,
    load: i32,
    oc: bool,
    oc_off_at: Option<usize>,
    frozen_until: usize,
    procs: Vec<Proc>,
    leak_mb: usize,
    walls: Vec<Wall>,
    scan_until: usize,
    // what's shown: the terminal line, the HUD buckets, the badge
    term: Option<(usize, usize, usize, usize)>,
    hud: (Option<usize>, Option<usize>, Option<usize>, Option<bool>, Option<(usize, Option<usize>)>),
    pub stats: Stats,
}

impl Coder {
    fn rank(&self) -> usize {
        self.rank.unwrap_or(0)
    }
    fn t(&self, table: &[usize; 8], top: usize) -> usize {
        tv(table, top, self.rank(), self.root)
    }
    /// CPU clock (GHz x100): 3.0, 3.9 overclocked, throttled above 85 C.
    fn ghz(&self) -> usize {
        let base = if self.oc { 390 } else { 300 };
        let h = self.heat / 100;
        if h <= 85 { base } else { (base as i64 * (100 - ((h - 85) as i64 * 100 / 30).min(70)) / 100) as usize }
    }
    fn knobs(&self) -> Knobs {
        let mut cps = self.t(&CPS100, CPS100_TOP) * self.ghz() / 300;
        if self.oc { cps = cps * 120 / 100; }
        let mut typo = self.t(&TYPO, TYPO_TOP);
        if self.oc { typo = typo * 115 / 100; }
        if self.heat >= 8500 { typo = typo * 125 / 100; }
        Knobs { cps100: cps, typo, notice: self.t(&NOTICE, NOTICE_TOP), rank: self.rank() }
    }
    fn ram_used(&self) -> usize {
        self.procs.iter().map(|p| p.mb).sum::<usize>() + self.leak_mb
    }
    fn fx(sim: &mut StableSim<'_>, me: usize, tag: &str, target: usize, life: u64) {
        crate::fx_unit(sim, &format!("{P}{tag}"), me, target, life);
    }
    fn fx_at(sim: &mut StableSim<'_>, me: usize, tag: &str, p: (i64, i64), life: u64) {
        crate::fx_point(sim, &format!("{P}{tag}"), me, p.0, p.1, life);
    }

    /// Frozen (an infinite loop, a segfault, a blue screen, an OOM stutter, the cache's idle): stunned for real.
    fn freeze(&mut self, sim: &mut StableSim<'_>, me: usize, ticks: usize, overlay: &str) {
        let tick = sim.tick();
        self.frozen_until = self.frozen_until.max(tick + ticks);
        self.stats.frozen += ticks;
        sim.apply_cc(me, &CcV1::stun(ticks as u64));
        Self::fx(sim, me, overlay, me, ticks.max(30) as u64);
    }

    // ---------------------------------------------------------------- the brain: what to write next

    /// How much a function is worth now (x10), by what's around him.
    fn value(&self, f: usize, all: &[Champ], m: &Champ) -> usize {
        let foes = all.iter().filter(|c| c.team != m.team && d2(c.x, c.y, m.x, m.y) <= sq(90_000)).count();
        let hurt = all.iter().filter(|c| c.team == m.team && c.hp * 100 < c.max_hp * 70).count();
        let base = SPEC[f].3;
        base + match f {
            SHIELD | HEAL => 15 * hurt.min(2),
            CHAIN | SPRAY | FIREWALL => 8 * foes.min(3),
            BLINK => if m.hp * 2 < m.max_hp { 20 } else { 0 },
            SCAN => if self.t(&READ, READ_TOP) > 5 { 15 } else { 0 },
            CACHE => if self.load > 5000 { 15 } else { 0 },
            _ => 0,
        }
    }

    /// Expected (seconds to finish, % chance it ships clean) of writing f in lang, as he *believes* it: the truth
    /// pulled toward "fast and clean" by how little he knows his limits (AWARE).
    fn believed(&self, f: usize, lang: usize) -> (usize, usize) {
        let k = self.knobs();
        let n = chars(f, lang);
        let risk: usize = lines(f, lang).iter().map(|l| l.1).sum();
        let secs = n * 100 / k.cps100.max(1) + compile_ticks(lang, k.rank) / 60;
        // expected typos x100, and how many slip through review
        let typos100 = k.typo * LANG[lang].typo / 100 * risk / 10 / 100;
        let slip100 = typos100 * (100 - k.notice) / 100 * (100 - LANG[lang].catch * (100 - LANG[lang].syntax) / 100) / 100;
        let clean = 100usize.saturating_sub(slip100.min(100));
        let aware = self.t(&AWARE, AWARE_TOP);
        let b_secs = secs * (40 + 60 * aware / 100) / 100;
        let b_clean = (clean * aware + 100 * (100 - aware)) / 100;
        (b_secs, b_clean)
    }

    fn choose(&mut self, all: &[Champ], m: &Champ, tick: usize) {
        if self.typing.is_some() || tick < self.next_choice || tick < self.debug_until { return; }
        self.next_choice = tick + 30;
        let iq = self.t(&IQ, IQ_TOP);
        let weakest = self.program.iter().map(|c| self.value(c.f, all, m)).min().unwrap_or(0);
        let mut best: Option<(usize, usize, usize)> = None;   // (score, f, lang)
        for f in 0..NF {
            if self.program.iter().any(|c| c.f == f) { continue; }
            let lang = if self.rng.chance(iq, 100) { IDEAL[f] } else { self.rng.below(LANGS.len()) };
            let v = self.value(f, all, m);
            if self.program.len() >= SLOTS && v * 10 < weakest * 13 { continue; }
            let (secs, clean) = self.believed(f, lang);
            let score = v * clean * 100 / (100 + secs * 8);
            if best.is_none_or(|b| score > b.0) { best = Some((score, f, lang)); }
        }
        let Some((_, f, lang)) = best else { return };
        // a saved copy reloads in a second (with whatever bugs it was saved with)
        if let Some(s) = self.storage.iter().find(|s| s.f == f) {
            self.typing = Some(Typing::reload(f, s.lang, &s.bugs.clone(), tick));
        } else {
            self.typing = Some(Typing::new(f, lang, tick));
        }
    }

    /// The compiled function joins the program (replacing the weakest when full; an unsaved one is lost), and with
    /// good habits he saves it.
    fn ship(&mut self, sim: &mut StableSim<'_>, me: usize, all: &[Champ], m: &Champ, f: usize, lang: usize, bugs: Vec<Bug>, reloaded: bool) {
        self.stats.shipped += 1;
        if bugs.is_empty() { self.stats.clean += 1; }
        self.stats.bugs += bugs.len();
        if self.program.len() >= SLOTS {
            if let Some(i) = (0..self.program.len()).min_by_key(|&i| self.value(self.program[i].f, all, m)) { self.program.remove(i); }
        }
        let iq = self.t(&IQ, IQ_TOP);
        let save = reloaded || self.rng.chance(iq, 100);
        let c = Compiled { f, lang, bugs, saved: save };
        if save && !reloaded {
            self.storage.retain(|s| s.f != f);
            if self.storage.len() >= STORAGE { self.storage.remove(0); }
            self.storage.push(c.clone());
            Self::fx(sim, me, "ov_saved", me, 40);
        } else {
            Self::fx(sim, me, "ov_compiled", me, 40);
        }
        // by value: the program is checked top down
        self.program.push(c);
        let vals: Vec<usize> = self.program.iter().map(|c| self.value(c.f, all, m)).collect();
        let mut idx: Vec<usize> = (0..self.program.len()).collect();
        idx.sort_by_key(|&i| std::cmp::Reverse(vals[i]));
        self.program = idx.into_iter().map(|i| self.program[i].clone()).collect();
    }

    fn step_typing(&mut self, sim: &mut StableSim<'_>, me: usize, all: &[Champ], m: &Champ, tick: usize) {
        let k = self.knobs();
        let Some(mut t) = self.typing.take() else { return };
        let reloaded = matches!(t.phase, Phase::Load { .. });
        let ev = t.tick(&k, &mut self.rng);
        // typing warms the CPU
        if t.cursor().is_some() { self.heat += if self.oc { 4 } else { 2 }; }
        match ev {
            Event::None => {
                self.show_line(sim, me, &t, &k);
                self.typing = Some(t);
            }
            Event::CompileFailed(_) => {
                self.stats.syntax_errors += 1;
                Self::fx(sim, me, if t.lang == RUST { "ov_borrow" } else { "ov_syntax" }, me, 45);
                self.typing = Some(t);
            }
            Event::Compiled(bugs) => {
                self.stats.caught += t.caught;
                self.stats.write_ticks += tick.saturating_sub(t.started);
                self.term = None;
                self.ship(sim, me, all, m, t.f, t.lang, bugs, reloaded);
            }
        }
    }

    /// The terminal over his head: the line he's typing, in 4 reveal steps (re-emitted only when the step changes).
    fn show_line(&mut self, sim: &mut StableSim<'_>, me: usize, t: &Typing, k: &Knobs) {
        let Some((line, typed)) = t.cursor() else {
            if let Phase::Compile { left } = t.phase {
                if self.term != Some((t.f, t.lang, 99, 0)) {
                    self.term = Some((t.f, t.lang, 99, 0));
                    Self::fx(sim, me, if t.lang == RUST { "ov_rustc" } else { "ov_compile" }, me, left.clamp(10, 300) as u64);
                }
            }
            if let Phase::Load { left } = t.phase {
                if self.term != Some((t.f, t.lang, 98, 0)) {
                    self.term = Some((t.f, t.lang, 98, 0));
                    Self::fx(sim, me, "ov_load", me, left.max(10) as u64);
                }
            }
            return;
        };
        let len = lines(t.f, t.lang)[line].0.max(1);
        let step = (typed * 4 / len + 1).min(4);
        let key = (t.f, t.lang, line, step);
        if self.term == Some(key) { return; }
        self.term = Some(key);
        // lasts until the next step should come (a little over, so the line never blinks out)
        let per_step = len.div_ceil(4) * 6000 / k.cps100.max(1);
        let tag = format!("ln_{}_{}_{line}_{step}", LANGS[t.lang], FUNCS[t.f].0);
        Self::fx(sim, me, &tag, me, (per_step + 6).clamp(6, 240) as u64);
    }

    // ---------------------------------------------------------------- the rig

    fn step_rig(&mut self, sim: &mut StableSim<'_>, me: usize, tick: usize) {
        // cooling, the overclock's heat, the CPU's load draining
        if self.heat > 4000 { self.heat -= 6; }
        if self.oc { self.heat += 14; }
        self.load = (self.load - 50).max(0);
        // a leak grows while the leaking function is in his program
        if tick.is_multiple_of(60) {
            let leaks = self.program.iter().filter(|c| c.bugs.contains(&Bug::Leak)).count();
            self.leak_mb += 400 * leaks;
        }
        self.procs.retain(|p| p.until > tick);
        if self.ram_used() > RAM_MB { self.oom(sim, me); }
        if self.heat >= 10_000 { self.bsod(sim, me); }
    }

    fn bsod(&mut self, sim: &mut StableSim<'_>, me: usize) {
        self.stats.bsods += 1;
        self.heat = 7000;
        self.oc = false;
        self.oc_off_at = None;
        self.typing = None;
        self.term = None;
        self.program.retain(|c| c.saved);
        self.procs.clear();
        self.leak_mb = 0;
        self.freeze(sim, me, 150, "ov_bsod");
        sim.entity_remove_buff(me, "cd_oc");
    }

    /// Out of memory: the newest process is killed and he stutters.
    fn oom(&mut self, sim: &mut StableSim<'_>, me: usize) {
        self.stats.ooms += 1;
        if let Some(p) = self.procs.pop() {
            match p.f {
                SHIELD => { sim.entity_remove_buff(p.target, "cd_shield"); }
                SLOW => { sim.entity_remove_buff(p.target, "cd_lag"); }
                FIREWALL => { self.walls.pop(); }
                SCAN => { self.scan_until = 0; }
                _ => {}
            }
        }
        if self.ram_used() > RAM_MB { self.leak_mb = 0; }   // the leaking process goes too
        self.freeze(sim, me, 30, "ov_oom");
    }

    fn step_overclock(&mut self, sim: &mut StableSim<'_>, me: usize, fighting: bool, tick: usize) {
        if !tick.is_multiple_of(6) { return; }
        let off = self.t(&OC_OFF, OC_OFF_TOP) as i32 * 100;
        if self.oc {
            if self.heat >= off && self.oc_off_at.is_none() { self.oc_off_at = Some(tick + self.t(&OC_LAG, OC_LAG_TOP)); }
            if self.oc_off_at.is_some_and(|t| tick >= t) || !fighting {
                self.oc = false;
                self.oc_off_at = None;
                sim.entity_remove_buff(me, "cd_oc");
            }
        } else if fighting && self.heat < off - 1500 {
            self.oc = true;
            sim.add_buff(me, &BuffV1::named("cd_oc"));
        }
    }

    // ---------------------------------------------------------------- the runtime: his program runs itself

    /// Health % of `c` as he reads it (noisy unless a scan is up). Deterministic per unit and second.
    fn read_hp(&self, c: &Champ, tick: usize) -> i64 {
        let real = (c.hp * 100 / c.max_hp.max(1)) as i64;
        if tick < self.scan_until { return real; }
        let n = self.t(&READ, READ_TOP) as i64;
        if n == 0 { return real; }
        let h = (c.id as u64 ^ (tick / 60) as u64 ^ self.rng.0.rotate_left(7)).wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 40;
        real + (h % (2 * n as u64 + 1)) as i64 - n
    }

    fn run_program(&mut self, sim: &mut StableSim<'_>, all: &[Champ], m: &Champ, tick: usize) {
        if tick < self.next_check { return; }
        let clock = self.t(&CLOCK, CLOCK_TOP) * 300 / self.ghz().max(1);
        self.next_check = tick + clock.max(4);
        for i in 0..self.program.len() {
            let c = self.program[i].clone();
            if tick < self.cooldown[c.f] { continue; }
            let off = c.bugs.contains(&Bug::OffByOne);
            let Some(target) = self.trigger(c.f, all, m, tick, off, c.bugs.contains(&Bug::WrongTarget)) else { continue };
            let cost = SPEC[c.f].1;
            if self.load + cost > 10_000 { continue; }   // queued: the CPU is busy
            // RAM: a good engineer doesn't start what won't fit
            let mb = SPEC[c.f].2 * LANG[c.lang].ram / 100;
            if mb > 0 && self.ram_used() + mb > RAM_MB && self.rng.chance(self.t(&IQ, IQ_TOP), 100) { continue; }
            self.load += cost;
            self.heat += LANG[c.lang].heat * if self.oc { 2 } else { 1 };
            self.cooldown[c.f] = tick + SPEC[c.f].0;
            self.stats.runs += 1;
            self.execute(sim, all, m, &c, target, tick, mb);
            return;   // one function a check
        }
    }

    /// Whether f's trigger holds, and on whom (a unit id, or 0 for himself / an area).
    fn trigger(&self, f: usize, all: &[Champ], m: &Champ, tick: usize, off: bool, wrong: bool) -> Option<usize> {
        let k = if off { 80 } else { 100 };   // an off-by-one shrinks every range and shifts every threshold
        let shift = if off { 20 } else { 0 };
        let foes: Vec<&Champ> = all.iter().filter(|c| c.team != m.team && sim_visible(c)).collect();
        let near = |r: i64| -> Vec<&Champ> {
            let mut v: Vec<&Champ> = foes.iter().copied().filter(|c| d2(c.x, c.y, m.x, m.y) <= sq(r * k / 100)).collect();
            v.sort_by_key(|c| d2(c.x, c.y, m.x, m.y));
            if wrong { v.reverse(); }
            v
        };
        let mates = || all.iter().filter(|c| c.team == m.team && d2(c.x, c.y, m.x, m.y) <= sq(60_000 * k / 100));
        match f {
            PING => near(70_000).first().map(|c| c.id),
            SHIELD => mates().filter(|c| self.read_hp(c, tick) < 70 + shift && foes.iter().any(|e| d2(e.x, e.y, c.x, c.y) <= sq(60_000)))
                .min_by_key(|c| self.read_hp(c, tick)).map(|c| c.id),
            HEAL => mates().filter(|c| self.read_hp(c, tick) < 55 + shift).min_by_key(|c| self.read_hp(c, tick)).map(|c| c.id),
            SCAN => (tick >= self.scan_until && !near(90_000).is_empty()).then_some(m.id),
            SPRAY => (near(25_000).len() >= 2).then_some(m.id),
            BLINK => (self.read_hp(m, tick) < 40 + shift).then(|| near(30_000).first().map(|c| c.id)).flatten(),
            SLOW => near(45_000).first().map(|c| c.id),
            CACHE => (self.load > 6000 && near(60_000).is_empty()).then_some(m.id),
            CHAIN => { let v = near(50_000); (v.len() >= 2).then(|| v[0].id) }
            FIREWALL => near(50_000).first().map(|c| c.id),
            _ => None,
        }
    }

    fn execute(&mut self, sim: &mut StableSim<'_>, all: &[Champ], m: &Champ, c: &Compiled, target: usize, tick: usize, mb: usize) {
        let me = m.id;
        // the bugs that strike at run time
        if c.bugs.contains(&Bug::InfiniteLoop) && self.rng.chance(60, 100) { self.freeze(sim, me, 90, "ov_loop"); return; }
        if c.bugs.contains(&Bug::Segfault) && self.rng.chance(50, 100) { self.freeze(sim, me, 60, "ov_segv"); return; }
        if c.bugs.contains(&Bug::NullRef) && self.rng.chance(50, 100) { Self::fx(sim, me, "ov_null", me, 40); return; }
        let flip = c.bugs.contains(&Bug::SignFlip);
        let wrong = c.bugs.contains(&Bug::WrongTarget);
        let ap = sim.get_entity(me).map_or(40, |e| e.stat().magic_power);
        let power = LANG[c.lang].power * self.ghz() / 300;   // x100
        let amt = |base: usize, ratio: usize| (base + ap * ratio / 100) * power / 100;
        let hit = |sim: &mut StableSim<'_>, t: usize, n: usize| {
            if flip { sim.heal(me, t, n); } else { sim.deal_damage(me, t, 0, n, AttackTypeV1::Skill); }
        };
        let pos = |id: usize| all.iter().find(|x| x.id == id).map(|x| (x.x, x.y));
        if mb > 0 { self.procs.push(Proc { until: tick + SPEC[c.f].4, mb, f: c.f, target }); }
        Self::fx(sim, me, "fx_send", me, 18);
        match c.f {
            PING => { hit(sim, target, amt(35, 50)); Self::fx(sim, me, if flip { "fx_heal" } else { "fx_ping" }, target, 24); }
            SHIELD | HEAL => {
                // a wrong target shields / heals the nearest enemy instead
                let t = if wrong { all.iter().filter(|x| x.team != m.team).min_by_key(|x| d2(x.x, x.y, m.x, m.y)).map_or(target, |x| x.id) } else { target };
                if c.f == SHIELD {
                    sim.entity_add_shield(t, amt(120, 50), 180);
                    sim.add_buff(t, &timed("cd_shield", 180));
                    Self::fx(sim, me, "fx_shield", t, 24);
                } else if flip {
                    if let Some(e) = sim.get_entity(t) { let (hp, _) = e.hp(); sim.entity_set_hp(t, hp.saturating_sub(amt(40, 20)).max(1)); }
                    Self::fx(sim, me, "fx_ping", t, 24);
                } else {
                    sim.heal(me, t, amt(80, 40));
                    Self::fx(sim, me, "fx_heal", t, 30);
                }
            }
            SCAN => { self.scan_until = tick + 180; Self::fx_at(sim, me, "fx_scan", (m.x, m.y), 36); }
            SPRAY => {
                for e in all.iter().filter(|x| x.team != m.team && d2(x.x, x.y, m.x, m.y) <= sq(25_000)) { hit(sim, e.id, amt(25, 30)); }
                Self::fx_at(sim, me, "fx_spray", (m.x, m.y), 30);
            }
            BLINK => {
                let Some((ex, ey)) = pos(target) else { return };
                let (dx, dy) = ((m.x - ex) as f64, (m.y - ey) as f64);
                let l = dx.hypot(dy).max(1.0);
                let s = if wrong { -30_000.0 } else { 30_000.0 };   // a wrong sign blinks him into them
                let to = walls::clip(m.x, m.y, m.x + (dx / l * s) as i64, m.y + (dy / l * s) as i64);
                Self::fx_at(sim, me, "fx_blink_out", (m.x, m.y), 24);
                sim.entity_set_pos(me, to.0.max(0) as u64, to.1.max(0) as u64);
                Self::fx_at(sim, me, "fx_blink_in", to, 24);
            }
            SLOW => {
                let mut b = timed("cd_lag", 120);
                b.move_speed_mult = if flip { 25 } else { -40 };
                sim.add_buff(target, &b);
            }
            CACHE => { self.load = (self.load - 4000).max(0); self.freeze(sim, me, 30, "fx_cache"); }
            CHAIN => {
                let mut cur = target;
                let mut hits: Vec<usize> = Vec::new();
                for _ in 0..4 {
                    hit(sim, cur, amt(30, 35));
                    Self::fx(sim, me, "fx_chain", cur, 20);
                    hits.push(cur);
                    let Some(p) = pos(cur) else { break };
                    let next = all.iter().filter(|x| x.team != m.team && !hits.contains(&x.id) && d2(x.x, x.y, p.0, p.1) <= sq(35_000))
                        .min_by_key(|x| d2(x.x, x.y, p.0, p.1)).map(|x| x.id);
                    match next { Some(n) => cur = n, None => break }
                }
            }
            FIREWALL => {
                let Some((ex, ey)) = pos(target) else { return };
                // across the way between him and the enemy, 60% of the way there
                let (cx, cy) = (m.x + (ex - m.x) * 6 / 10, m.y + (ey - m.y) * 6 / 10);
                let (dx, dy) = ((ex - m.x) as f64, (ey - m.y) as f64);
                let l = dx.hypot(dy).max(1.0);
                let (nx, ny) = (-dy / l * 20_000.0, dx / l * 20_000.0);
                let a = (cx - nx as i64, cy - ny as i64);
                let b = (cx + nx as i64, cy + ny as i64);
                self.walls.push(Wall { a, b, until: tick + 240, next: tick, dmg: amt(15, 15), flip });
            }
            _ => {}
        }
    }

    fn step_walls(&mut self, sim: &mut StableSim<'_>, all: &[Champ], m: &Champ, tick: usize) {
        self.walls.retain(|w| w.until > tick);
        for i in 0..self.walls.len() {
            if tick < self.walls[i].next { continue; }
            self.walls[i].next = tick + 20;
            let w = self.walls[i].clone();
            for e in all.iter().filter(|x| x.team != m.team && seg_d2((x.x, x.y), w.a, w.b) <= sq(6_000)) {
                if w.flip { sim.heal(m.id, e.id, w.dmg); } else { sim.deal_damage(m.id, e.id, 0, w.dmg, AttackTypeV1::Skill); }
            }
            // drawn as two pieces along the wall, at its angle (8 buckets over 180 degrees)
            let ang = ((w.b.1 - w.a.1) as f64).atan2((w.b.0 - w.a.0) as f64).to_degrees().rem_euclid(180.0);
            let a8 = ((ang / 22.5).round() as usize) % 8;
            for q in [1i64, 3] {
                let p = (w.a.0 + (w.b.0 - w.a.0) * q / 4, w.a.1 + (w.b.1 - w.a.1) * q / 4);
                Self::fx_at(sim, m.id, &format!("fx_wall_{a8}"), p, 20);
            }
        }
    }

    // ---------------------------------------------------------------- the HUD and the badge

    fn show(&mut self, sim: &mut StableSim<'_>, m: &Champ) {
        let heat = ((self.heat.max(0) / 100) as usize / 10).min(10);
        let ram = (self.ram_used() * 8 / RAM_MB).min(8);
        let disk = self.storage.len().min(8);
        let swap = |sim: &mut StableSim<'_>, old: Option<usize>, new: usize, name: &str, has: bool| {
            if old != Some(new) || !has {
                if let Some(o) = old { sim.entity_remove_buff(m.id, &format!("{name}{o}")); }
                sim.add_buff(m.id, &BuffV1::named(&format!("{name}{new}")));
            }
        };
        swap(sim, self.hud.0, heat, "cd_heat", m.has(&format!("cd_heat{heat}")));
        self.hud.0 = Some(heat);
        swap(sim, self.hud.1, ram, "cd_ram", m.has(&format!("cd_ram{ram}")));
        self.hud.1 = Some(ram);
        swap(sim, self.hud.2, disk, "cd_disk", m.has(&format!("cd_disk{disk}")));
        self.hud.2 = Some(disk);
        let badge = (self.rank(), self.root);
        let name = if badge.0 >= ROOT { format!("cd_root{}", badge.1.unwrap_or(10)) } else { format!("cd_rank{}", badge.0) };
        if self.rank.is_some() && (self.hud.4 != Some(badge) || !m.has(&name)) {
            for k in 0..ROOT { sim.entity_remove_buff(m.id, &format!("cd_rank{k}")); }
            for p in 1..=10 { sim.entity_remove_buff(m.id, &format!("cd_root{p}")); }
            sim.add_buff(m.id, &BuffV1::named(&name));
            self.hud.4 = Some(badge);
        }
    }
}

/// Square distance from p to the segment a-b.
fn seg_d2(p: (i64, i64), a: (i64, i64), b: (i64, i64)) -> i128 {
    let (ax, ay, bx, by, px, py) = (a.0 as f64, a.1 as f64, b.0 as f64, b.1 as f64, p.0 as f64, p.1 as f64);
    let (dx, dy) = (bx - ax, by - ay);
    let l2 = dx * dx + dy * dy;
    let t = if l2 == 0.0 { 0.0 } else { (((px - ax) * dx + (py - ay) * dy) / l2).clamp(0.0, 1.0) };
    let (qx, qy) = (ax + dx * t, ay + dy * t);
    ((px - qx).powi(2) + (py - qy).powi(2)) as i128
}

/// Every champion in the list is alive; visibility is checked where the list is made (see on_update).
fn sim_visible(_c: &Champ) -> bool {
    true
}

impl StablePassive for Coder {
    fn clone_box(&self) -> Box<dyn StablePassive> {
        Box::new(self.clone())
    }
    fn on_dead(&mut self, sim: &mut StableSim<'_>, _player: usize) {
        // dying loses what's on the screen, not what's in the program
        self.typing = None;
        self.term = None;
        self.procs.clear();
        self.walls.clear();
        self.heat = 4000;
        self.oc = false;
        self.oc_off_at = None;
        self.alive = false;
        self.hud = (None, None, None, None, None);
        if let Some(me) = self.me { sim.entity_remove_buff(me, "cd_oc"); }
    }
    fn on_update(&mut self, sim: &mut StableSim<'_>, _seed: u64, player: usize, entity: usize) {
        let tick = sim.tick();
        self.me = Some(entity);
        if !self.started {
            self.started = true;
            self.rng = Rng(sim.seed() ^ (entity as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ 0xC0DE);
            self.heat = 4000;
            self.next_debug = 600;
            let _ = BOOK.memory();
        }
        if self.rank.is_none() && tick >= 60 {
            self.athlete = BOOK.athlete_of(sim.seed(), player);
            let (r, p) = BOOK.pinned(sim.seed()).rank_for(self.athlete);
            self.rank = Some(r);
            self.root = p;
        }
        let all_raw = champions(sim);
        let Some(m) = all_raw.iter().find(|c| c.id == entity).cloned() else { return };
        if self.team.is_none() { self.team = Some(m.team); }
        // he only acts on enemies his team can see
        let all: Vec<Champ> = all_raw.iter().filter(|c| c.team == m.team || sim.is_visible(m.team, c.id)).cloned().collect();
        if self.sig.is_none() && tick >= 1800 && self.rank.is_some() {
            let sig = signature(&BOOK, sim, &all_raw, self.athlete);
            if BOOK.first(format!("log {sig}")) {
                let r = self.rank();
                let label = if r >= ROOT { format!("Root #{}", self.root.unwrap_or(10)) } else { RANK_NAMES[r].to_string() };
                BOOK.log(&format!("game {sig} athlete {} rank {r} ({label}), {} chars/s, typo {}/10000, notice {}%",
                    self.athlete.map_or("?".into(), |a| a.to_string()), self.knobs().cps100 / 100, self.knobs().typo, self.knobs().notice));
            }
            self.sig = Some(sig);
        }
        if let (Some(sig), Some(team), Some(a)) = (self.sig.clone(), self.team, self.athlete) {
            if tick.is_multiple_of(30) { self.rec.track(&BOOK, sim, &sig, team, a, tick); }
            // every minute of a game: what he wrote and how it went (once, though both simulations run it)
            if tick.is_multiple_of(3600) && BOOK.first(format!("sum {sig} {tick}")) {
                let s = &self.stats;
                BOOK.log(&format!("game {sig} minute {}: shipped {} ({} clean, {} bugs shipped, {} typos caught, {} syntax errors), \
                    {} runs, frozen {:.1} s, {} blue screens, {} out of memory; program: {}", tick / 3600, s.shipped, s.clean, s.bugs,
                    s.caught, s.syntax_errors, s.runs, s.frozen as f64 / 60.0, s.bsods, s.ooms,
                    self.program.iter().map(|c| format!("{}.{}{}", FUNCS[c.f].0, LANGS[c.lang], if c.bugs.is_empty() { "" } else { "*" })).collect::<Vec<_>>().join(" ")));
            }
        }
        if !self.alive { self.alive = true; self.last_hp = m.hp; }
        self.step_rig(sim, entity, tick);
        self.step_walls(sim, &all, &m, tick);
        if tick.is_multiple_of(6) { self.show(sim, &m); }
        if tick < self.frozen_until { return; }
        let fighting = all.iter().any(|c| c.team != m.team && d2(c.x, c.y, m.x, m.y) <= sq(70_000));
        self.step_overclock(sim, entity, fighting, tick);
        // being hit can break his concentration: the line he's on starts over
        if m.hp + m.max_hp / 30 < self.last_hp && self.typing.is_some() && self.rng.chance(25, 100) {
            if let Some(t) = self.typing.as_mut() {
                if let Phase::Type { done, .. } = &mut t.phase { *done = 0; }
            }
        }
        self.last_hp = m.hp;
        // S2 debug: every 10 s when he isn't typing, he reads his program over and fixes what he finds
        if tick >= self.next_debug && self.typing.is_none() && !self.program.is_empty() {
            self.next_debug = tick + 600;
            self.debug_until = tick + 60;
            let notice = (self.t(&NOTICE, NOTICE_TOP) + 10).min(100);
            let mut found = 0;
            for i in 0..self.program.len() {
                let bugs = std::mem::take(&mut self.program[i].bugs);
                let mut kept = Vec::new();
                for b in bugs { if self.rng.chance(notice, 100) { found += 1; } else { kept.push(b); } }
                self.program[i].bugs = kept;
            }
            self.stats.caught += found;
            Self::fx(sim, entity, "ov_debug", entity, 60);
        }
        self.choose(&all, &m, tick);
        self.step_typing(sim, entity, &all, &m, tick);
        self.run_program(sim, &all, &m, tick);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn knobs(rank: usize, root: Option<usize>) -> Knobs {
        Knobs { cps100: tv(&CPS100, CPS100_TOP, rank, root), typo: tv(&TYPO, TYPO_TOP, rank, root), notice: tv(&NOTICE, NOTICE_TOP, rank, root), rank }
    }

    /// Writes f in lang start to finish: (ticks, logic bugs shipped, syntax errors hit).
    fn write(rank: usize, root: Option<usize>, f: usize, lang: usize, seed: u64) -> (usize, usize, usize) {
        let k = knobs(rank, root);
        let mut rng = Rng(seed);
        let mut t = Typing::new(f, lang, 0);
        let mut fails = 0;
        for tick in 1..200_000 {
            match t.tick(&k, &mut rng) {
                Event::Compiled(b) => return (tick, b.len(), fails),
                Event::CompileFailed(_) => fails += 1,
                Event::None => {}
            }
        }
        panic!("never compiled");
    }

    fn mean(rank: usize, root: Option<usize>, f: usize, lang: usize) -> (f64, f64, f64) {
        let n = 2000;
        let mut s = (0.0, 0.0, 0.0);
        for i in 0..n {
            let (t, b, e) = write(rank, root, f, lang, 0x1234 + i as u64 * 7919);
            s.0 += t as f64; s.1 += b as f64; s.2 += e as f64;
        }
        (s.0 / n as f64 / 60.0, s.1 / n as f64, s.2 / n as f64)
    }

    #[test]
    fn the_code_table_matches_the_functions() {
        assert_eq!(FUNCS.len(), NF);
        for (i, name) in ["ping", "shield", "heal", "scan", "spray", "blink", "slow", "cache", "chain", "firewall"].iter().enumerate() {
            assert_eq!(FUNCS[i].0, *name);
        }
        assert_eq!(LANGS, ["py", "cpp", "rust"]);
        // the same function is longer in C++ and Rust than in Python
        for f in 0..NF { assert!(chars(f, PY) < chars(f, CPP) && chars(f, PY) < chars(f, RUST), "{}", FUNCS[f].0); }
    }

    #[test]
    fn root_interpolates_to_number_one() {
        assert_eq!(tv(&CPS100, CPS100_TOP, 3, None), 900);
        assert_eq!(tv(&CPS100, CPS100_TOP, ROOT, Some(10)), 2200);
        assert_eq!(tv(&CPS100, CPS100_TOP, ROOT, Some(1)), 3000);
        assert_eq!(tv(&TYPO, TYPO_TOP, ROOT, Some(1)), 0);
        assert_eq!(tv(&NOTICE, NOTICE_TOP, ROOT, Some(1)), 100);
    }

    #[test]
    fn writing_is_deterministic_per_seed() {
        assert_eq!(write(2, None, FIREWALL, RUST, 99), write(2, None, FIREWALL, RUST, 99));
    }

    /// The gap: every rank step writes the same hard function faster and ships fewer bugs; only #1 is perfect.
    #[test]
    fn the_gap_grows_with_every_rank() {
        let mut last = (f64::MAX, f64::MAX);
        for rank in 0..ROOT {
            let (secs, bugs, errs) = mean(rank, None, CHAIN, CPP);
            eprintln!("{:13} chain in C++: {secs:5.1} s, {bugs:.2} bugs shipped, {errs:.2} syntax errors", RANK_NAMES[rank]);
            assert!(secs < last.0 && bugs <= last.1, "{}: {secs} s {bugs} bugs", RANK_NAMES[rank]);
            last = (secs, bugs);
        }
        let (secs1, bugs1, _) = mean(ROOT, Some(1), CHAIN, CPP);
        eprintln!("Root #1       chain in C++: {secs1:5.1} s, {bugs1:.2} bugs");
        assert_eq!(bugs1, 0.0, "Zero-Day ships perfect code");
        let (kid, _, _) = mean(0, None, CHAIN, CPP);
        assert!(kid > secs1 * 8.0, "a Script Kiddie takes far longer ({kid:.1} s vs {secs1:.1} s)");
        // the easy stuff is fine for everyone
        let (ping_secs, ping_bugs, _) = mean(0, None, PING, PY);
        assert!(ping_secs < 22.0 && ping_bugs < 0.7, "{ping_secs} {ping_bugs}");
    }

    #[test]
    fn rust_catches_bugs_at_a_price() {
        // the same rank writes firewall: Rust ships fewer logic bugs than C++, but compiles far longer
        let (py_s, _, _) = mean(2, None, FIREWALL, PY);
        let (cpp_s, cpp_b, _) = mean(2, None, FIREWALL, CPP);
        let (rs_s, rs_b, _) = mean(2, None, FIREWALL, RUST);
        eprintln!("Junior firewall: py {py_s:.1} s, C++ {cpp_s:.1} s {cpp_b:.2} bugs, Rust {rs_s:.1} s {rs_b:.2} bugs");
        assert!(rs_b < cpp_b * 0.6, "{rs_b} vs {cpp_b}");
        assert!(rs_s > cpp_s && cpp_s > py_s);
    }

    #[test]
    fn a_saved_function_reloads_with_its_bugs() {
        let k = knobs(3, None);
        let mut rng = Rng(5);
        let mut t = Typing::reload(SLOW, PY, &[Bug::OffByOne], 0);
        let mut ev = Event::None;
        for _ in 0..100 { ev = t.tick(&k, &mut rng); if ev != Event::None { break; } }
        assert_eq!(ev, Event::Compiled(vec![Bug::OffByOne]));
    }

    #[test]
    fn heat_throttles_the_clock() {
        let mut c = Coder { rank: Some(3), ..Coder::default() };
        c.heat = 6000;
        assert_eq!(c.ghz(), 300);
        c.oc = true;
        assert_eq!(c.ghz(), 390);
        c.heat = 9700;
        assert!(c.ghz() < 300, "{}", c.ghz());
        // low ranks keep the overclock on into a blue screen; the best let go well before
        assert!(tv(&OC_OFF, OC_OFF_TOP, 0, None) >= 100 && tv(&OC_OFF, OC_OFF_TOP, ROOT, Some(1)) < 90);
    }

    #[test]
    fn believed_cost_is_rosier_at_low_rank() {
        let kid = Coder { rank: Some(0), ..Coder::default() };
        let (secs, clean) = kid.believed(FIREWALL, CPP);
        let truth = mean(0, None, FIREWALL, CPP);
        assert!((secs as f64) < truth.0 * 0.7, "he thinks {secs} s, it takes {:.0} s", truth.0);
        assert!(clean > 60, "he thinks it's {clean}% clean");
    }

    /// Every effect and buff the native code plays exists in the data (the art and data scripts write them).
    #[test]
    fn every_view_name_exists() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../mods/tfm2_custom/champion/tfm2_custom_coder.data_champion");
        let Ok(text) = std::fs::read_to_string(&path) else { return };
        let names: std::collections::HashSet<&str> = text.split("\"name\": \"").skip(1).filter_map(|t| t.split('"').next()).collect();
        let fx = |tag: &str| names.contains(format!("{P}{tag}").as_str());
        for f in 0..NF {
            for (li, lang) in LANGS.iter().enumerate() {
                for line in 0..lines(f, li).len() {
                    for step in 1..=4 { assert!(fx(&format!("ln_{lang}_{}_{line}_{step}", FUNCS[f].0)), "{} {lang} {line} {step}", FUNCS[f].0); }
                }
                assert!(!fx(&format!("ln_{lang}_{}_{}_1", FUNCS[f].0, lines(f, li).len())), "the art has more lines than the table");
            }
        }
        for o in ["compiled", "saved", "syntax", "borrow", "rustc", "compile", "load", "debug", "oom", "segv", "null", "loop", "bsod"] {
            assert!(fx(&format!("ov_{o}")), "ov_{o}");
        }
        for e in ["send", "ping", "heal", "shield", "scan", "spray", "blink_out", "blink_in", "cache", "chain"] { assert!(fx(&format!("fx_{e}")), "fx_{e}"); }
        for a in 0..8 { assert!(fx(&format!("fx_wall_{a}"))); }
        let buff = |n: String| names.contains(n.as_str());
        for b in ["cd_shield", "cd_lag", "cd_oc"] { assert!(buff(b.into()), "{b}"); }
        for n in 0..=10 { assert!(buff(format!("cd_heat{n}"))); }
        for n in 0..=8 { assert!(buff(format!("cd_ram{n}")) && buff(format!("cd_disk{n}"))); }
        for r in 0..ROOT { assert!(buff(format!("cd_rank{r}"))); }
        for p in 1..=10 { assert!(buff(format!("cd_root{p}"))); }
        assert!(text.contains("\"passive_ref\": \"tfm2_custom_ai:coder\""));
    }

    #[test]
    fn wall_distance() {
        assert_eq!(seg_d2((0, 10), (-10, 0), (10, 0)), 100);
        assert_eq!(seg_d2((20, 0), (-10, 0), (10, 0)), 100);
    }
}
