//! Round 101-102: the Coder, a mastery champion who fights by writing code (Rian: "hard to master, a significant gap
//! between each" rank; "make each function harder to write the lower the rank ... the design actually writes what the
//! function does"; CPU, RAM, storage, heat, languages and overclocking; an AI copilot; "buys a new RAM or storage").
//!
//! He picks a function, types its real code (coder_code.rs, generated with the art from Claude outputs/coder/
//! coder_functions.py) character by character at his rank's speed; each character can be a typo (more on symbols,
//! by language, when hot, overclocked or mining). Typos are SyntaxErrors (the compile fails, he retypes the line) or
//! logic bugs that compile and misbehave (wrong target, off-by-one, infinite loop, null reference, sign flip,
//! segfault, leak). He reviews before compiling (NOTICE); Rust's compiler catches most logic bugs, slowly. A compiled
//! function joins his program (5 slots) and runs itself whenever its trigger holds, checked every CLOCK ticks.
//!
//! The rig: CPU (load; 3.0 GHz, 3.9 overclocked, throttled above 85 C), RAM (lasting effects hold it; over the top is
//! OOM), storage (saved functions reload with their bugs), heat (100 C blue-screens: stunned, unsaved functions lost).
//! Round 102: he mines Bitcoin (a passive trickle, more with the miner on: it loads and heats his CPU), earns it from
//! functions that land, kills and assists, and buys parts with it: RAM, storage, an SSD, a cooler, a CPU. Away from
//! his base an install stuns him for 2 s. The ult (his brain's): an AI writes for him for 12 s: Claude Max 20x
//! (fewest syntax errors, smallest usage pool), ChatGPT Pro (fewest logic bugs, thinks first), Gemini AI Ultra (three
//! functions per prompt, reads the fight perfectly, more hallucinated syntax); a drained pool gives the lite model
//! (Haiku / mini / Flash). His prompts and his review stay his own.
//!
//! Every rank can try every function; rank sets the speed, typo rate, review, self-judgement, clock, overclock
//! timing, language and part choices, prompting and AI juggling. His three skills are never cast by the game.

use crate::coder_code::{FUNCS, LANGS};
use crate::mastery::{signature, Book, Record};
use crate::{champions, d2, sq, timed, walls, Champ};
use mod_api_stable::{AttackTypeV1, BuffV1, CcV1, StablePassive, StableSim};
use std::collections::{HashMap, VecDeque};

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
/// % judgement: the right language, saving clean code, RAM, mining, parts, which AI and when to switch.
const IQ: [usize; 8] = [10, 25, 40, 55, 70, 85, 93, 97];
const IQ_TOP: usize = 100;
/// Overclock: the heat (C) he turns it off at, and how late (ticks) he reacts.
const OC_OFF: [usize; 8] = [103, 101, 98, 95, 92, 90, 89, 89];
const OC_OFF_TOP: usize = 88;
const OC_LAG: [usize; 8] = [60, 45, 30, 20, 12, 8, 4, 2];
const OC_LAG_TOP: usize = 0;
/// Round 102: how much his prompts multiply an AI's error rates (x100): a vague prompt gets worse code.
const PROMPT: [usize; 8] = [200, 180, 160, 140, 120, 100, 80, 70];
const PROMPT_TOP: usize = 50;

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
const LANG: [Lang; 5] = [
    Lang { typo: 80, syntax: 50, compile: 0, power: 80, heat: 150, ram: 150, catch: 0, bugs: [20, 25, 10, 35, 10, 0, 0] },
    Lang { typo: 120, syntax: 60, compile: 60, power: 130, heat: 400, ram: 80, catch: 0, bugs: [15, 20, 10, 0, 10, 25, 20] },
    Lang { typo: 110, syntax: 70, compile: 150, power: 120, heat: 250, ram: 70, catch: 80, bugs: [25, 35, 20, 0, 20, 0, 0] },
    // round 102: JavaScript, quick and loose (NaN: null references and sign flips); Assembly, the hardest hitting and
    // the easiest to break (loops that never end, crashes)
    Lang { typo: 90, syntax: 40, compile: 0, power: 90, heat: 150, ram: 120, catch: 0, bugs: [15, 15, 10, 30, 30, 0, 0] },
    Lang { typo: 160, syntax: 30, compile: 0, power: 160, heat: 600, ram: 40, catch: 0, bugs: [15, 15, 30, 0, 10, 30, 0] },
];
pub const PY: usize = 0;
pub const CPP: usize = 1;
pub const RUST: usize = 2;
pub const JS: usize = 3;
pub const ASM: usize = 4;

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

const NF: usize = 24;
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
const DDOS: usize = 10;
const CLEANSE: usize = 11;
const BOOST: usize = 12;
const FORK: usize = 13;
const SWAP: usize = 14;
const SORT: usize = 15;
const ENCRYPT: usize = 16;
const DDOS_ALL: usize = 17;
const KILL9: usize = 18;
const ROLLBACK: usize = 19;
const RECURSE: usize = 20;
const INJECT: usize = 21;
const GC: usize = 22;
const DEPLOY: usize = 23;

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
    (300, 2000, 0, 35, 0),          // ddos
    (480, 1500, 0, 25, 0),          // cleanse
    (900, 3000, 3000, 35, 180),     // boost
    (900, 2500, 4000, 45, 360),     // fork (per drone)
    (900, 2000, 0, 30, 0),          // swap
    (1200, 3000, 0, 40, 0),         // sort
    (600, 1500, 1500, 30, 180),     // encrypt
    (1200, 3500, 0, 45, 0),         // ddos_all
    (600, 2500, 0, 50, 0),          // kill9
    (1200, 3000, 0, 45, 0),         // rollback
    (600, 2000, 0, 40, 0),          // recurse
    (900, 2500, 0, 45, 0),          // inject
    (900, 2500, 0, 40, 0),          // gc
    (1800, 0, 2000, 40, 480),       // deploy
];
/// The language a perfect judge writes each function in: short utility in Python or JavaScript, hot damage loops in
/// C++, anything that must not misfire (swaps, executions, rollbacks, the drones' threads) in Rust. From Architect up
/// the hot loops go to Assembly (see ideal(); round 103: not ping(), whose 45-tick cooldown can't carry Assembly's heat).
const IDEAL: [usize; NF] = [PY, PY, PY, JS, PY, PY, PY, JS, CPP, RUST, CPP, PY, PY, RUST, RUST, PY, RUST, CPP, RUST, RUST, CPP, RUST, CPP, RUST];
const SLOTS: usize = 5;
const HOME_R: i64 = 40_000;
/// Round 102: the terminal line's centre, 50 px over his (950 world units a px). Its sprite is just the panel, placed
/// there as a point effect and re-placed every TERM_EVERY ticks so it follows him (a follow effect is centred on him,
/// so its sprite had to carry 100 px of empty space to sit over his head: ten times the texture).
const TERM_DY: i64 = 50 * 950;
const TERM_EVERY: usize = 6;

/// Round 103: from Senior up a rig floats around him (sheet coder_rank: cd_rig<k> behind him, cd_rigf<k> in front):
/// Senior 1, Staff 2, Architect 3, Root 4, Zero-Day (#1) 5.
pub fn rig_tier(rank: usize, root: Option<usize>) -> usize {
    match rank {
        4 => 1,
        5 => 2,
        6 => 3,
        r if r >= ROOT => if root == Some(1) { 5 } else { 4 },
        _ => 0,
    }
}
/// Round 103: the effects that grow from Architect up (<tag>_hi: twice the size, a gold trim).
const HI_FX: [&str; 8] = ["fx_ping", "fx_shield", "fx_heal", "fx_chain", "fx_ddos", "fx_kill9", "fx_inject", "fx_rollback"];
const HI_RANK: usize = 6;
thread_local! {
    /// whether the Coder being updated draws the top-rank effects (set at the top of each update)
    static HI: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}
fn fx_name(tag: &str) -> String {
    if HI.with(|h| h.get()) && HI_FX.contains(&tag) { format!("{P}{tag}_hi") } else { format!("{P}{tag}") }
}

/// Round 105: whether a status line of `prio` may take the slot now (a live line of higher priority keeps it).
fn say_accepts(cur: &Option<(String, usize, u8)>, tick: usize, prio: u8) -> bool {
    !matches!(cur, Some((_, until, p)) if tick < *until && *p > prio)
}

fn ideal(f: usize, rank: usize) -> usize {
    if rank >= 6 && matches!(f, CHAIN | DDOS | RECURSE) { ASM } else { IDEAL[f] }
}

fn lines(f: usize, lang: usize) -> &'static [(usize, usize)] {
    FUNCS[f].2[lang]
}
pub fn chars(f: usize, lang: usize) -> usize {
    lines(f, lang).iter().map(|l| l.0).sum()
}
fn tier(f: usize) -> usize {
    FUNCS[f].1
}

// ------------------------------------------------------------------ hardware (round 102: bought with Bitcoin)

pub const RAM: usize = 0;
pub const DISK: usize = 1;
pub const SSD: usize = 2;
pub const COOL: usize = 3;
pub const CPU: usize = 4;
pub const PARTS: [&str; 5] = ["ram", "disk", "ssd", "cool", "cpu"];
/// Price (Bitcoin) of each part's tier 1 and 2.
pub const PRICE: [[usize; 2]; 5] = [[60, 140], [40, 100], [50, 120], [50, 130], [80, 180]];
const RAM_MB: [usize; 3] = [16_000, 32_000, 64_000];
const STORAGE: [usize; 3] = [8, 16, 32];
const RELOAD: [usize; 3] = [60, 30, 12];
const COMPILE_PCT: [usize; 3] = [100, 85, 70];
/// Cooling, C x100 a tick.
const COOLING: [i32; 3] = [6, 9, 13];
/// Base clock, GHz x100 (overclock adds 90).
const GHZ: [usize; 3] = [300, 360, 420];

/// What his problems this game call for (the part he'd buy next with perfect judgement): counters of OOMs, storage
/// overwrites, reloads, throttled seconds and blue screens; the CPU when nothing is wrong. None when everything that
/// would help is maxed.
pub fn needed(tiers: &[usize; 5], ooms: usize, overwrites: usize, reloads: usize, hot_secs: usize, bsods: usize) -> Option<usize> {
    let score = [ooms * 3, overwrites * 2, reloads, bsods * 4 + hot_secs / 10, 1];
    (0..5).filter(|&p| tiers[p] < 2).max_by_key(|&p| (score[p], p == CPU))
}

// ------------------------------------------------------------------ AI copilots (round 102)

pub const CLAUDE: usize = 0;
pub const GPT: usize = 1;
pub const GEMINI: usize = 2;
pub const PROVIDERS: [&str; 3] = ["claude", "gpt", "gemini"];

/// A model's typing speed (cps x100), syntax and logic errors per line (% x100), think ticks before writing, and
/// functions per prompt.
#[derive(Clone, Copy)]
struct Model {
    cps100: usize,
    syntax: usize,
    logic: usize,
    think: usize,
    per_prompt: usize,
}
/// [provider][0 flagship, 1 lite]
const MODELS: [[Model; 2]; 3] = [
    [Model { cps100: 4000, syntax: 100, logic: 300, think: 0, per_prompt: 1 },     // Claude Max 20x
     Model { cps100: 7000, syntax: 300, logic: 600, think: 0, per_prompt: 1 }],    // Claude Haiku
    [Model { cps100: 4500, syntax: 300, logic: 100, think: 150, per_prompt: 1 },   // ChatGPT Pro
     Model { cps100: 7000, syntax: 400, logic: 900, think: 0, per_prompt: 1 }],    // GPT mini
    [Model { cps100: 5000, syntax: 400, logic: 300, think: 0, per_prompt: 3 },     // Gemini AI Ultra
     Model { cps100: 7000, syntax: 800, logic: 600, think: 0, per_prompt: 1 }],    // Gemini Flash
];
/// Usage pools (x100) and refill a second (x100); a prompt costs half the function's characters.
const POOL: [i32; 3] = [10_000, 16_000, 24_000];
const REFILL: [i32; 3] = [50, 90, 130];
const AI_TICKS: usize = 720;
/// Round 103: after a blue screen he doesn't overclock again for this long (he'd crash every few seconds).
const BSOD_SHY: usize = 600;
/// Round 103: with judgement he holds back a run that would take his CPU past this (C x100).
const HOT_SKIP: i32 = 9500;
const AI_COOLDOWN: usize = 2400;

/// A model writing f: its knobs (the per-character error rate from its per-line rates, the syntax share), scaled by
/// his prompting. `read100` is his own reading speed (he still reviews it himself).
fn ai_knobs(p: usize, lite: bool, f: usize, lang: usize, prompt: usize, notice: usize, rank: usize, read100: usize) -> Knobs {
    let m = MODELS[p][lite as usize];
    let mut syntax = m.syntax;
    if p == CLAUDE && lite && tier(f) >= 4 { syntax *= 3; }
    if p == GEMINI && !lite && chars(f, lang) > 80 { syntax *= 2; }
    let n = lines(f, lang).len().max(1);
    let avg = (chars(f, lang) / n).max(1);
    // per line (% x100) -> per 10000 characters
    let per_char = (syntax + m.logic) * prompt / 100 / avg;
    Knobs { cps100: m.cps100, typo: per_char, notice, rank, syntax: Some(syntax * 100 / (syntax + m.logic).max(1)), lang_mult: false,
            read100, compile_pct: 100 }
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

/// What his rank and rig (or the AI writing for him) make of writing right now.
#[derive(Clone, Copy)]
struct Knobs {
    cps100: usize,
    /// typos per 10000 plain characters
    typo: usize,
    notice: usize,
    rank: usize,
    /// the share of typos that are syntax errors, when not the language's (an AI's own profile)
    syntax: Option<usize>,
    /// the language scales the typo rate (his own typing; an AI's rate is already its own)
    lang_mult: bool,
    /// his reading speed for the review
    read100: usize,
    /// compile time % (the SSD)
    compile_pct: usize,
}

#[derive(Clone, Debug, PartialEq)]
enum Phase {
    Think { left: usize },
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
    /// written by an AI: (provider, lite)
    ai: Option<(usize, bool)>,
}

impl Typing {
    fn new(f: usize, lang: usize, tick: usize) -> Typing {
        Typing { f, lang, phase: Phase::Type { line: 0, done: 0 }, typos: Vec::new(), caught: 0, started: tick, ai: None }
    }
    fn by_ai(f: usize, lang: usize, tick: usize, ai: (usize, bool), think: usize) -> Typing {
        let phase = if think > 0 { Phase::Think { left: think } } else { Phase::Type { line: 0, done: 0 } };
        Typing { f, lang, phase, typos: Vec::new(), caught: 0, started: tick, ai: Some(ai) }
    }
    fn reload(f: usize, lang: usize, bugs: &[Bug], tick: usize, ticks: usize) -> Typing {
        Typing { f, lang, phase: Phase::Load { left: ticks }, typos: bugs.iter().map(|&b| (0, Typo::Logic(b))).collect(), caught: 0, started: tick, ai: None }
    }

    /// Where the terminal is: (line, characters typed of it) while typing or fixing.
    fn cursor(&self) -> Option<(usize, usize)> {
        match &self.phase {
            Phase::Type { line, done } => Some((*line, done / 100)),
            Phase::Fix { queue, done } => queue.first().map(|&l| (l, done / 100)),
            _ => None,
        }
    }

    /// Type a tick's worth of the current line, rolling a typo at each character crossed. True when it's done.
    fn type_line(&mut self, line: usize, done: &mut usize, k: &Knobs, rng: &mut Rng) -> bool {
        let (len, risk) = lines(self.f, self.lang)[line];
        let before = *done / 100;
        *done += k.cps100 * 100 / 6000;   // chars x100 a tick (60 ticks a second)
        let after = (*done / 100).min(len);
        // per character: typo rate x the line's syntax risk (risk is x10 per character; plain text averages 10)
        let lang_k = if k.lang_mult { LANG[self.lang].typo } else { 100 };
        let per_char = k.typo * lang_k / 100 * risk / (10 * len.max(1));
        let syntax = k.syntax.unwrap_or(LANG[self.lang].syntax);
        for _ in before..after {
            if rng.chance(per_char, 10_000) {
                let t = if rng.chance(syntax, 100) { Typo::Syntax } else {
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

    /// One tick of thinking / typing / reviewing / fixing / compiling.
    fn tick(&mut self, k: &Knobs, rng: &mut Rng) -> Event {
        let n_lines = lines(self.f, self.lang).len();
        let mut phase = std::mem::replace(&mut self.phase, Phase::Review { left: 0 });
        let mut out = Event::None;
        let compile = compile_ticks(self.lang, k.rank) * k.compile_pct / 100;
        phase = match phase {
            Phase::Think { left } if left > 0 => Phase::Think { left: left - 1 },
            Phase::Think { .. } => Phase::Type { line: 0, done: 0 },
            Phase::Type { line, mut done } => {
                if self.type_line(line, &mut done, k, rng) {
                    if line + 1 < n_lines { Phase::Type { line: line + 1, done: 0 } } else {
                        // reading it over goes three times as fast as he types
                        Phase::Review { left: chars(self.f, self.lang) * 6000 / (k.read100 * 3).max(1) }
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
                if queue.is_empty() { Phase::Compile { left: compile } } else { Phase::Fix { queue, done: 0 } }
            }
            Phase::Fix { mut queue, mut done } => {
                let l = queue[0];
                if self.type_line(l, &mut done, k, rng) {
                    queue.remove(0);
                    if queue.is_empty() { Phase::Compile { left: compile } } else { Phase::Fix { queue, done: 0 } }
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
                        out = Event::Compiled(self.logic_bugs());
                        Phase::Compile { left: 0 }
                    }
                }
            }
            Phase::Load { left } if left > 0 => Phase::Load { left: left - 1 },
            Phase::Load { .. } => {
                out = Event::Compiled(self.logic_bugs());
                Phase::Load { left: 0 }
            }
        };
        self.phase = phase;
        out
    }

    fn logic_bugs(&self) -> Vec<Bug> {
        self.typos.iter().filter_map(|t| if let Typo::Logic(b) = t.1 { Some(b) } else { None }).collect()
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

/// A hit landing later (ddos packets, recursion's pings, the drones'); `nearest` re-aims at the nearest enemy.
#[derive(Clone, Debug)]
struct Hit {
    at: usize,
    target: usize,
    dmg: usize,
    flip: bool,
    nearest: bool,
    fx: &'static str,
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
    // round 102
    pub overwrites: usize,
    pub reloads: usize,
    pub hot_ticks: usize,
    pub btc_earned: usize,
    pub bought: Vec<String>,
    pub prompts: [usize; 3],
    pub lite_prompts: usize,
}

#[derive(Clone, Default)]
struct Copilot {
    /// usage left (x100) per provider
    pools: [i32; 3],
    provider: usize,
    lite: bool,
    until: usize,
    next: usize,
    /// a switch (or ChatGPT's thinking) holds the next prompt until this tick
    hold: usize,
    /// Gemini's extra functions from the same prompt
    queue: Vec<(usize, usize)>,
    shown: Option<(usize, bool)>,
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
    home: Option<(i64, i64)>,
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
    // the rig: heat and load x100, overclock and when he'll notice to turn it off; the parts' tiers
    heat: i32,
    load: i32,
    oc: bool,
    oc_off_at: Option<usize>,
    frozen_until: usize,
    procs: Vec<Proc>,
    leak_mb: usize,
    walls: Vec<Wall>,
    hits: Vec<Hit>,
    drones: Vec<(usize, usize)>,
    scan_until: usize,
    deploy_until: usize,
    pub tiers: [usize; 5],
    // Bitcoin x100, the miner, the counters he earns from
    btc: usize,
    mining: bool,
    kills: (usize, usize),
    ai: Copilot,
    /// where every champion was (tick, x, y, hp), every 6 ticks for 3 s: rollback
    history: HashMap<usize, VecDeque<(usize, i64, i64, usize)>>,
    // what's shown: the terminal line, the HUD buckets, the badge
    term: Option<(usize, usize, usize, usize)>,
    term_next: usize,
    hud: (Option<usize>, Option<usize>, Option<usize>, Option<bool>, Option<(usize, Option<usize>)>),
    shown_btc: Option<usize>,
    shown_drones: usize,
    shown_rig: Option<usize>,
    bsod_until: usize,
    /// round 105: the one status line over his terminal (tag, until, priority) and when it's next re-placed
    say: Option<(String, usize, u8)>,
    say_next: usize,
    /// the tick being updated (for the brain's judgement of what could run now)
    now: usize,
    pub stats: Stats,
}

impl Coder {
    fn rank(&self) -> usize {
        self.rank.unwrap_or(0)
    }
    fn t(&self, table: &[usize; 8], top: usize) -> usize {
        tv(table, top, self.rank(), self.root)
    }
    fn ram_cap(&self) -> usize {
        RAM_MB[self.tiers[RAM]]
    }
    fn storage_cap(&self) -> usize {
        STORAGE[self.tiers[DISK]]
    }
    fn ai_on(&self, tick: usize) -> bool {
        tick < self.ai.until
    }
    /// CPU clock (GHz x100): the CPU's base, +0.9 overclocked, throttled above 85 C.
    fn ghz(&self) -> usize {
        let base = GHZ[self.tiers[CPU]] + if self.oc { 90 } else { 0 };
        let h = self.heat / 100;
        if h <= 85 { base } else { (base as i64 * (100 - ((h - 85) as i64 * 100 / 30).min(70)) / 100) as usize }
    }
    /// His own typing right now.
    fn knobs(&self) -> Knobs {
        let mut cps = self.t(&CPS100, CPS100_TOP) * self.ghz() / 300;
        if self.oc { cps = cps * 120 / 100; }
        if self.mining { cps = cps * 80 / 100; }
        let mut typo = self.t(&TYPO, TYPO_TOP);
        if self.oc { typo = typo * 115 / 100; }
        if self.heat >= 8500 { typo = typo * 125 / 100; }
        Knobs { cps100: cps, typo, notice: self.t(&NOTICE, NOTICE_TOP), rank: self.rank(), syntax: None, lang_mult: true,
                read100: cps, compile_pct: COMPILE_PCT[self.tiers[SSD]] }
    }
    /// Who's writing this function: him, or the AI while it's on.
    fn knobs_for(&self, t: &Typing, tick: usize) -> Knobs {
        let mine = self.knobs();
        match t.ai {
            Some((p, lite)) if self.ai_on(tick) => {
                let mut k = ai_knobs(p, lite, t.f, t.lang, self.t(&PROMPT, PROMPT_TOP), mine.notice, mine.rank, mine.cps100);
                k.compile_pct = mine.compile_pct;
                k
            }
            _ => mine,
        }
    }
    fn ram_used(&self) -> usize {
        self.procs.iter().map(|p| p.mb).sum::<usize>() + self.leak_mb
    }
    fn fx(sim: &mut StableSim<'_>, me: usize, tag: &str, target: usize, life: u64) {
        crate::fx_unit(sim, &fx_name(tag), me, target, life);
    }
    fn fx_at(sim: &mut StableSim<'_>, me: usize, tag: &str, p: (i64, i64), life: u64) {
        crate::fx_point(sim, &fx_name(tag), me, p.0, p.1, life);
    }
    /// Round 105: show a status line (`ov_*`) for `life` ticks. The game plays an effect's animation to its end whatever
    /// life it's given, so the line's frames are TERM_EVERY long and step_say re-places it every TERM_EVERY ticks
    /// until it's due to go (a 6 s frame left every line on screen 6 s). One line at a time: a freeze's line
    /// (priority 2) holds the slot until it ends; any other line replaces the one shown.
    fn say(&mut self, sim: &mut StableSim<'_>, me: usize, tag: &str, life: usize, prio: u8) {
        let tick = sim.tick();
        if !say_accepts(&self.say, tick, prio) { return; }
        self.say = Some((tag.to_string(), tick + life.max(1), prio));
        self.say_next = tick;
        self.step_say(sim, me, tick);
    }
    fn step_say(&mut self, sim: &mut StableSim<'_>, me: usize, tick: usize) {
        let Some((tag, until, _)) = self.say.clone() else { return };
        if tick >= until { self.say = None; return; }
        if tick < self.say_next { return; }
        self.say_next = tick + TERM_EVERY;
        Self::fx(sim, me, &tag, me, TERM_EVERY as u64 + 1);
    }
    fn earn(&mut self, btc100: usize) {
        self.btc += btc100;
        self.stats.btc_earned += btc100;
    }

    /// Frozen (an infinite loop, a segfault, a blue screen, an OOM stutter, the cache's idle, an install): stunned.
    fn freeze(&mut self, sim: &mut StableSim<'_>, me: usize, ticks: usize, overlay: &str) {
        let tick = sim.tick();
        self.frozen_until = self.frozen_until.max(tick + ticks);
        self.stats.frozen += ticks;
        sim.apply_cc(me, &CcV1::stun(ticks as u64));
        if overlay.starts_with("ov_") { self.say(sim, me, overlay, ticks.max(30), 2); } else { Self::fx(sim, me, overlay, me, ticks.max(30) as u64); }
    }

    // ---------------------------------------------------------------- the brain: what to write next

    /// How much a function is worth now (x10), by what's around him.
    fn value(&self, f: usize, all: &[Champ], m: &Champ) -> usize {
        let foes = all.iter().filter(|c| c.team != m.team && d2(c.x, c.y, m.x, m.y) <= sq(90_000)).count();
        let hurt = all.iter().filter(|c| c.team == m.team && c.hp * 100 < c.max_hp * 70).count();
        let mates = all.iter().filter(|c| c.team == m.team && c.id != m.id && d2(c.x, c.y, m.x, m.y) <= sq(60_000)).count();
        // round 103: foresight: allies still healthy but with an enemy on them count for half, as far as he sees it coming
        let pressed = all.iter().filter(|c| c.team == m.team && c.hp * 100 >= c.max_hp * 70
            && all.iter().any(|e| e.team != m.team && d2(e.x, e.y, c.x, c.y) <= sq(30_000))).count();
        let base = SPEC[f].3;
        base + match f {
            SHIELD | HEAL | ENCRYPT | CLEANSE | ROLLBACK | SWAP => 12 * hurt.min(2) + 6 * pressed.min(2) * self.t(&IQ, IQ_TOP) / 100,
            CHAIN | SPRAY | FIREWALL | DDOS_ALL | SORT | GC => 8 * foes.min(3),
            BOOST => 6 * mates.min(3),
            BLINK => if m.hp * 2 < m.max_hp { 20 } else { 0 },
            SCAN => if self.t(&READ, READ_TOP) > 5 { 15 } else { 0 },
            CACHE => if self.load > 5000 { 15 } else { 0 },
            DEPLOY => 5 * self.program.len(),
            _ => 0,
        }
    }

    /// Expected (seconds to finish, % chance it ships clean) of writing f in lang, as he *believes* it: the truth
    /// pulled toward "fast and clean" by how little he knows his limits (`aware`).
    fn believed(&self, f: usize, lang: usize, aware: usize) -> (usize, usize) {
        let k = self.knobs();
        let n = chars(f, lang);
        let risk: usize = lines(f, lang).iter().map(|l| l.1).sum();
        let secs = n * 100 / k.cps100.max(1) + compile_ticks(lang, k.rank) / 60;
        // expected typos x100, and how many slip through review
        let typos100 = k.typo * LANG[lang].typo / 100 * risk / 10 / 100;
        let slip100 = typos100 * (100 - k.notice) / 100 * (100 - LANG[lang].catch * (100 - LANG[lang].syntax) / 100) / 100;
        let clean = 100usize.saturating_sub(slip100.min(100));
        let b_secs = secs * (40 + 60 * aware / 100) / 100;
        let b_clean = (clean * aware + 100 * (100 - aware)) / 100;
        (b_secs, b_clean)
    }

    /// The functions worth writing next, best first: (score, f, lang). `sharp`: an AI that reasons it through
    /// (ChatGPT) judges with the truth and picks the right language.
    fn candidates(&mut self, all: &[Champ], m: &Champ, sharp: bool) -> Vec<(usize, usize, usize)> {
        let iq = if sharp { 100 } else { self.t(&IQ, IQ_TOP) };
        let aware = if sharp { 100 } else { self.t(&AWARE, AWARE_TOP) };
        let weakest = self.program.iter().map(|c| self.value(c.f, all, m)).min().unwrap_or(0);
        let mut out = Vec::new();
        for f in 0..NF {
            if self.program.iter().any(|c| c.f == f) { continue; }
            if self.typing.as_ref().is_some_and(|t| t.f == f) { continue; }
            let lang = if self.rng.chance(iq, 100) { ideal(f, self.rank()) } else { self.rng.below(LANGS.len()) };
            let v = self.value(f, all, m);
            if self.program.len() >= SLOTS && v * 10 < weakest * 13 { continue; }
            // round 103: judgement also asks whether it could run now (the lab showed the top ranks writing sort() with
            // two enemies about, while the low ranks' ping() ran all fight)
            // (a defensive function is insurance: it counts as ready while an ally has an enemy on them)
            let insurance = matches!(f, SHIELD | HEAL | ENCRYPT | CLEANSE | ROLLBACK | SWAP) && all.iter().any(|c| c.team == m.team
                && all.iter().any(|e| e.team != m.team && d2(e.x, e.y, c.x, c.y) <= sq(30_000)));
            let ready = insurance || self.trigger(f, all, m, self.now, false, false).is_some();
            let v = if ready { v } else { v * (100 - iq / 2) / 100 };
            let (secs, clean) = self.believed(f, lang, aware);
            out.push((v * clean * 100 / (100 + secs * 8), f, lang));
        }
        out.sort_by_key(|c| std::cmp::Reverse(c.0));
        out
    }

    fn choose(&mut self, sim: &mut StableSim<'_>, all: &[Champ], m: &Champ, tick: usize) {
        if self.typing.is_some() || tick < self.next_choice || tick < self.debug_until { return; }
        self.next_choice = tick + 30;
        // round 102: what's left of the AI's last prompt (Gemini writes several at once)
        if self.ai_on(tick) {
            if let Some((f, lang)) = self.ai.queue.first().copied() {
                self.ai.queue.remove(0);
                if !self.program.iter().any(|c| c.f == f) {
                    self.typing = Some(Typing::by_ai(f, lang, tick, (self.ai.provider, self.ai.lite), 0));
                    return;
                }
            }
            if tick < self.ai.hold { return; }
            self.prompt_ai(sim, all, m, tick);
            return;
        }
        let picks = self.candidates(all, m, false);
        let Some(&(_, f, lang)) = picks.first() else { return };
        // a saved copy reloads (with whatever bugs it was saved with)
        if let Some(s) = self.storage.iter().find(|s| s.f == f) {
            self.stats.reloads += 1;
            self.typing = Some(Typing::reload(f, s.lang, &s.bugs.clone(), tick, RELOAD[self.tiers[SSD]]));
        } else {
            self.typing = Some(Typing::new(f, lang, tick));
        }
    }

    // ---------------------------------------------------------------- the AI copilot (the ult)

    fn flagship_ok(&self, p: usize) -> bool {
        self.ai.pools[p] * 100 >= POOL[p] * 60
    }

    /// Activate the AI: when a fight is on or near and it's off cooldown. Which one: with judgement, Gemini to fill an
    /// empty program, Claude when long risky code is next, ChatGPT otherwise; whichever has its flagship up. Without
    /// it, any.
    fn activate_ai(&mut self, sim: &mut StableSim<'_>, all: &[Champ], m: &Champ, tick: usize) {
        if self.ai_on(tick) || tick < self.ai.next || self.rank.is_none() || !tick.is_multiple_of(30) { return; }
        let near = all.iter().any(|c| c.team != m.team && d2(c.x, c.y, m.x, m.y) <= sq(120_000));
        if !near { return; }
        let smart = self.rng.chance(self.t(&IQ, IQ_TOP), 100);
        let p = if smart {
            let next_tier = self.candidates(all, m, false).first().map_or(1, |c| tier(c.1));
            let want = if self.program.len() <= 2 { GEMINI } else if next_tier >= 4 { CLAUDE } else { GPT };
            if self.flagship_ok(want) { want } else { (0..3).find(|&q| self.flagship_ok(q)).unwrap_or(want) }
        } else { self.rng.below(3) };
        // round 103: with judgement he hands the AI what he's writing (drops his draft and prompts); without it he keeps
        // typing and the AI waits for him (the lab showed the slow ranks burning their whole ult on their own typing)
        if smart && self.typing.as_ref().is_some_and(|t| t.ai.is_none() && !matches!(t.phase, Phase::Load { .. })) {
            self.typing = None;
            self.term = None;
        }
        self.ai.provider = p;
        self.ai.until = tick + AI_TICKS;
        self.ai.next = tick + AI_COOLDOWN;
        self.ai.hold = 0;
        self.ai.queue.clear();
        self.next_choice = tick;
        let _ = sim;
    }

    /// A prompt: the provider's flagship while its pool is over 60%, its lite model below that, nothing when the
    /// prompt doesn't fit (rate limited). With judgement he switches to a provider whose flagship is up (1 s).
    fn prompt_ai(&mut self, sim: &mut StableSim<'_>, all: &[Champ], m: &Champ, tick: usize) {
        let me = m.id;
        let p = self.ai.provider;
        if !self.flagship_ok(p) && self.rng.chance(self.t(&IQ, IQ_TOP), 100) {
            if let Some(q) = (0..3).filter(|&q| q != p && self.flagship_ok(q)).max_by_key(|&q| self.ai.pools[q] * 100 / POOL[q]) {
                self.ai.provider = q;
                self.ai.hold = tick + 60;
                self.say(sim, me, &format!("ov_switch_{}", PROVIDERS[q]), 60, 1);
                return;
            }
        }
        let lite = !self.flagship_ok(p);
        let sharp = p == GPT && !lite;
        let picks = self.candidates(all, m, sharp);
        let n = MODELS[p][lite as usize].per_prompt;
        let mut chosen: Vec<(usize, usize)> = Vec::new();
        let mut cost = 0;
        for &(_, f, lang) in picks.iter().take(n) {
            cost += chars(f, lang) as i32 * 50;
            chosen.push((f, lang));
        }
        let Some(&(f, lang)) = chosen.first() else { return };
        if self.ai.pools[p] < cost {
            // rate limited: this provider is out until it refills
            self.say(sim, me, "ov_ratelimit", 60, 1);
            self.ai.hold = tick + 120;
            return;
        }
        self.ai.pools[p] -= cost;
        self.ai.lite = lite;
        self.stats.prompts[p] += 1;
        if lite { self.stats.lite_prompts += 1; }
        let think = MODELS[p][lite as usize].think;
        let overlay = match (p, lite) {
            (GPT, false) => "ov_reasoning",
            (GEMINI, false) if chosen.len() > 1 => "ov_diff",
            _ => "ov_thinking",
        };
        self.say(sim, me, overlay, think.max(45), 1);
        self.typing = Some(Typing::by_ai(f, lang, tick, (p, lite), think));
        self.ai.queue = chosen.into_iter().skip(1).collect();
    }

    fn step_ai(&mut self, sim: &mut StableSim<'_>, me: usize, tick: usize) {
        if tick.is_multiple_of(60) {
            for p in 0..3 { self.ai.pools[p] = (self.ai.pools[p] + REFILL[p]).min(POOL[p]); }
        }
        // the icon over the terminal: who's writing, flagship or lite
        let want = self.ai_on(tick).then_some((self.ai.provider, self.ai.lite));
        if want != self.ai.shown {
            if let Some((p, l)) = self.ai.shown { sim.entity_remove_buff(me, &format!("cd_ai_{}{}", PROVIDERS[p], if l { "_lite" } else { "" })); }
            if let Some((p, l)) = want { sim.add_buff(me, &BuffV1::named(&format!("cd_ai_{}{}", PROVIDERS[p], if l { "_lite" } else { "" }))); }
            self.ai.shown = want;
        }
        if !self.ai_on(tick) { self.ai.queue.clear(); }
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
            if self.storage.len() >= self.storage_cap() {
                self.storage.remove(0);
                self.stats.overwrites += 1;
            }
            self.storage.push(c.clone());
            self.say(sim, me, "ov_saved", 40, 1);
        } else {
            self.say(sim, me, "ov_compiled", 40, 1);
        }
        // by value: the program is checked top down
        self.program.push(c);
        let vals: Vec<usize> = self.program.iter().map(|c| self.value(c.f, all, m)).collect();
        let mut idx: Vec<usize> = (0..self.program.len()).collect();
        idx.sort_by_key(|&i| std::cmp::Reverse(vals[i]));
        self.program = idx.into_iter().map(|i| self.program[i].clone()).collect();
    }

    fn step_typing(&mut self, sim: &mut StableSim<'_>, me: usize, all: &[Champ], m: &Champ, tick: usize) {
        let Some(mut t) = self.typing.take() else { return };
        let k = self.knobs_for(&t, tick);
        let reloaded = matches!(t.phase, Phase::Load { .. });
        let ev = t.tick(&k, &mut self.rng);
        // typing warms the CPU (an AI's tokens don't)
        if t.cursor().is_some() && t.ai.is_none() { self.heat += if self.oc { 4 } else { 2 }; }
        match ev {
            Event::None => {
                self.show_line(sim, me, &t);
                self.typing = Some(t);
            }
            Event::CompileFailed(_) => {
                self.stats.syntax_errors += 1;
                self.say(sim, me, if t.lang == RUST { "ov_borrow" } else { "ov_syntax" }, 45, 1);
                self.typing = Some(t);
            }
            Event::Compiled(bugs) => {
                self.stats.caught += t.caught;
                self.stats.write_ticks += tick.saturating_sub(t.started);
                self.term = None;
                self.next_choice = tick;
                self.ship(sim, me, all, m, t.f, t.lang, bugs, reloaded);
            }
        }
    }

    /// The terminal over his head: the line being typed, in 4 reveal steps, re-placed over him every TERM_EVERY ticks.
    fn show_line(&mut self, sim: &mut StableSim<'_>, me: usize, t: &Typing) {
        let Some((line, typed)) = t.cursor() else {
            if let Phase::Compile { left } = t.phase {
                if self.term != Some((t.f, t.lang, 99, 0)) {
                    self.term = Some((t.f, t.lang, 99, 0));
                    self.say(sim, me, if t.lang == RUST { "ov_rustc" } else { "ov_compile" }, left.clamp(10, 300), 1);
                }
            }
            if let Phase::Load { left } = t.phase {
                if self.term != Some((t.f, t.lang, 98, 0)) {
                    self.term = Some((t.f, t.lang, 98, 0));
                    self.say(sim, me, "ov_load", left.max(10), 1);
                }
            }
            return;
        };
        let len = lines(t.f, t.lang)[line].0.max(1);
        let step = (typed * 4 / len + 1).min(4);
        let key = (t.f, t.lang, line, step);
        let tick = sim.tick();
        if self.term == Some(key) && tick < self.term_next { return; }
        self.term = Some(key);
        self.term_next = tick + TERM_EVERY;
        let Some(e) = sim.get_entity(me) else { return };
        let (x, y) = e.pos();
        let tag = format!("ln_{}_{}_{line}_{step}", LANGS[t.lang], FUNCS[t.f].0);
        Self::fx_at(sim, me, &tag, (x as i64, y as i64 - TERM_DY), TERM_EVERY as u64 + 1);
    }

    // ---------------------------------------------------------------- the rig, Bitcoin and the shop

    fn step_rig(&mut self, sim: &mut StableSim<'_>, me: usize, tick: usize) {
        // cooling, the overclock's and the miner's heat, the CPU's load draining (the miner keeps some)
        if self.heat > 4000 { self.heat -= COOLING[self.tiers[COOL]]; }
        if self.oc { self.heat += 14; }
        if self.mining { self.heat += 4; }
        self.load = (self.load - 50 + if self.mining { 20 } else { 0 }).max(0);
        if self.heat >= 8500 { self.stats.hot_ticks += 1; }
        if tick.is_multiple_of(60) {
            // a leak grows while the leaking function is in his program
            let leaks = self.program.iter().filter(|c| c.bugs.contains(&Bug::Leak)).count();
            self.leak_mb += 400 * leaks;
            // Bitcoin: a trickle always, more with the miner on
            self.earn(50 + if self.mining { 200 } else { 0 });
        }
        self.procs.retain(|p| p.until > tick);
        if self.ram_used() > self.ram_cap() { self.oom(sim, me); }
        if self.heat >= 10_000 { self.bsod(sim, me); }
    }

    fn bsod(&mut self, sim: &mut StableSim<'_>, me: usize) {
        self.stats.bsods += 1;
        self.heat = 7000;
        self.oc = false;
        self.oc_off_at = None;
        self.mining = false;
        self.typing = None;
        self.term = None;
        self.program.retain(|c| c.saved);
        self.procs.clear();
        self.leak_mb = 0;
        self.freeze(sim, me, 150, "ov_bsod");
        self.bsod_until = sim.tick() + 150;
        sim.entity_remove_buff(me, "cd_oc");
        sim.entity_remove_buff(me, "cd_mine");
    }

    /// Out of memory: the newest process is killed and he stutters.
    fn oom(&mut self, sim: &mut StableSim<'_>, me: usize) {
        self.stats.ooms += 1;
        if let Some(p) = self.procs.pop() {
            match p.f {
                SHIELD => { sim.entity_remove_buff(p.target, "cd_shield"); }
                SLOW => { sim.entity_remove_buff(p.target, "cd_lag"); }
                ENCRYPT => { sim.entity_remove_buff(p.target, "cd_encrypt"); }
                FIREWALL => { self.walls.pop(); }
                FORK => { self.drones.pop(); }
                SCAN => { self.scan_until = 0; }
                DEPLOY => { self.deploy_until = 0; }
                _ => {}
            }
        }
        if self.ram_used() > self.ram_cap() { self.leak_mb = 0; }   // the leaking process goes too
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
        } else if fighting && self.heat < off - 1500 && tick >= self.bsod_until + BSOD_SHY {
            self.oc = true;
            sim.add_buff(me, &BuffV1::named("cd_oc"));
        }
    }

    /// The miner: with judgement only when nothing's around and the rig is cool; without, always.
    fn step_mining(&mut self, sim: &mut StableSim<'_>, me: usize, fighting: bool, tick: usize) {
        if !tick.is_multiple_of(60) { return; }
        let want = if self.rng.chance(self.t(&IQ, IQ_TOP), 100) { !fighting && self.heat < 7000 && !self.oc } else { true };
        if want != self.mining {
            self.mining = want;
            if want { sim.add_buff(me, &BuffV1::named("cd_mine")); } else { sim.entity_remove_buff(me, "cd_mine"); }
        }
    }

    /// Kills and assists pay (the player's own counters).
    fn step_bounties(&mut self, sim: &StableSim<'_>, player: usize) {
        let Some(p) = sim.get_player(player) else { return };
        let (k, a) = (p.kills(), p.assists());
        let (k0, a0) = self.kills;
        if k > k0 { self.earn((k - k0) * 2500); }
        if a > a0 { self.earn((a - a0) * 1000); }
        self.kills = (k, a);
    }

    /// The shop: what he buys and when. With judgement, the part his problems call for, when he can afford it, at
    /// base or out of a fight; without, any part he can afford, anywhere (an install away from base stuns him 2 s).
    fn step_shop(&mut self, sim: &mut StableSim<'_>, m: &Champ, fighting: bool, tick: usize) {
        if !tick.is_multiple_of(60) { return; }
        let btc = self.btc / 100;
        let at_home = self.home.is_some_and(|h| d2(h.0, h.1, m.x, m.y) <= sq(HOME_R));
        let s = &self.stats;
        let need = needed(&self.tiers, s.ooms, s.overwrites, s.reloads, s.hot_ticks / 60, s.bsods);
        let pick = if self.rng.chance(self.t(&IQ, IQ_TOP), 100) {
            need.filter(|&p| btc >= PRICE[p][self.tiers[p]] && (at_home || !fighting))
        } else {
            let affordable: Vec<usize> = (0..5).filter(|&p| self.tiers[p] < 2 && btc >= PRICE[p][self.tiers[p]]).collect();
            (!affordable.is_empty()).then(|| affordable[self.rng.below(affordable.len())])
        };
        let Some(p) = pick else { return };
        let tier = self.tiers[p];
        self.btc -= PRICE[p][tier] * 100;
        self.tiers[p] = tier + 1;
        self.stats.bought.push(format!("{}{}", PARTS[p], tier + 1));
        self.say(sim, m.id, &format!("ov_buy_{}{}", PARTS[p], tier + 1), 90, 1);
        if !at_home { self.freeze(sim, m.id, 120, "ov_install"); }
    }

    // ---------------------------------------------------------------- the runtime: his program runs itself

    /// Health % of `c` as he reads it (noisy unless a scan is up, or Gemini's flagship is reading the fight).
    fn read_hp(&self, c: &Champ, tick: usize) -> i64 {
        let real = (c.hp * 100 / c.max_hp.max(1)) as i64;
        if tick < self.scan_until || (self.ai_on(tick) && self.ai.provider == GEMINI && !self.ai.lite) { return real; }
        let n = self.t(&READ, READ_TOP) as i64;
        if n == 0 { return real; }
        let h = (c.id as u64 ^ (tick / 60) as u64 ^ self.rng.0.rotate_left(7)).wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 40;
        real + (h % (2 * n as u64 + 1)) as i64 - n
    }

    fn run_program(&mut self, sim: &mut StableSim<'_>, all: &[Champ], m: &Champ, tick: usize) {
        if tick < self.next_check { return; }
        let clock = self.t(&CLOCK, CLOCK_TOP) * 300 / self.ghz().max(1);
        self.next_check = tick + clock.max(4);
        let deployed = tick < self.deploy_until;
        for i in 0..self.program.len() {
            let c = self.program[i].clone();
            if tick < self.cooldown[c.f] { continue; }
            let off = c.bugs.contains(&Bug::OffByOne);
            let Some(target) = self.trigger(c.f, all, m, tick, off, c.bugs.contains(&Bug::WrongTarget)) else { continue };
            let cost = SPEC[c.f].1 / if deployed { 2 } else { 1 };
            if self.load + cost > 10_000 { continue; }   // queued: the CPU is busy
            // RAM: a good engineer doesn't start what won't fit
            let mb = SPEC[c.f].2 * LANG[c.lang].ram / 100;
            if mb > 0 && self.ram_used() + mb > self.ram_cap() && self.rng.chance(self.t(&IQ, IQ_TOP), 100) { continue; }
            // round 103: nor what would blue-screen him (the lab showed Assembly ping() cooking the top ranks' rigs)
            let heat = LANG[c.lang].heat * if self.oc { 2 } else { 1 };
            if self.heat + heat >= HOT_SKIP && self.rng.chance(self.t(&IQ, IQ_TOP), 100) { continue; }
            self.load += cost;
            self.heat += heat;
            self.cooldown[c.f] = tick + SPEC[c.f].0;
            self.stats.runs += 1;
            if self.execute(sim, all, m, &c, target, tick, mb) { self.earn(100); }
            return;   // one function a check
        }
    }

    /// Whether f's trigger holds, and on whom (a unit id, or his own for himself / an area).
    fn trigger(&self, f: usize, all: &[Champ], m: &Champ, tick: usize, off: bool, wrong: bool) -> Option<usize> {
        let k = if off { 80 } else { 100 };   // an off-by-one shrinks every range and shifts every threshold
        let shift = if off { 20 } else { 0 };
        let foes: Vec<&Champ> = all.iter().filter(|c| c.team != m.team).collect();
        let near = |r: i64| -> Vec<&Champ> {
            let mut v: Vec<&Champ> = foes.iter().copied().filter(|c| d2(c.x, c.y, m.x, m.y) <= sq(r * k / 100)).collect();
            v.sort_by_key(|c| d2(c.x, c.y, m.x, m.y));
            if wrong { v.reverse(); }
            v
        };
        let mates = || all.iter().filter(|c| c.team == m.team && d2(c.x, c.y, m.x, m.y) <= sq(60_000 * k / 100));
        let pressed = |c: &Champ| foes.iter().any(|e| d2(e.x, e.y, c.x, c.y) <= sq(60_000));
        match f {
            PING => near(70_000).first().map(|c| c.id),
            SHIELD => mates().filter(|c| self.read_hp(c, tick) < 70 + shift && pressed(c)).min_by_key(|c| self.read_hp(c, tick)).map(|c| c.id),
            HEAL => mates().filter(|c| self.read_hp(c, tick) < 55 + shift).min_by_key(|c| self.read_hp(c, tick)).map(|c| c.id),
            SCAN => (tick >= self.scan_until && !near(90_000).is_empty()).then_some(m.id),
            SPRAY => (near(25_000).len() >= 2).then_some(m.id),
            BLINK => (self.read_hp(m, tick) < 40 + shift).then(|| near(30_000).first().map(|c| c.id)).flatten(),
            SLOW | FIREWALL | INJECT => near(if f == SLOW { 45_000 } else { 55_000 }).first().map(|c| c.id),
            CACHE => (self.load > 6000 && near(60_000).is_empty()).then_some(m.id),
            CHAIN => { let v = near(50_000); (v.len() >= 2).then(|| v[0].id) }
            DDOS | RECURSE => near(70_000).first().map(|c| c.id),
            CLEANSE => mates().find(|c| c.stunned).map(|c| c.id),
            BOOST => (mates().count() >= 2 && !near(70_000).is_empty()).then_some(m.id),
            FORK => (!near(80_000).is_empty()).then_some(m.id),
            SWAP => mates().filter(|c| c.id != m.id && self.read_hp(c, tick) < 40 + shift)
                .find(|c| foes.iter().any(|e| d2(e.x, e.y, c.x, c.y) <= sq(20_000))).map(|c| c.id),
            SORT => (near(60_000).len() >= 3).then_some(m.id),
            ENCRYPT => mates().filter(|c| self.read_hp(c, tick) < 50 + shift && pressed(c)).min_by_key(|c| self.read_hp(c, tick)).map(|c| c.id),
            DDOS_ALL => (near(80_000).len() >= 2).then_some(m.id),
            KILL9 => near(70_000).into_iter().find(|c| self.read_hp(c, tick) < 15 + shift).map(|c| c.id),
            ROLLBACK => {
                // an ally who just lost a third of their health, or an enemy who just dived in
                let burst = mates().find(|c| self.history.get(&c.id).and_then(|h| h.front()).is_some_and(|s| s.3 > c.hp + c.max_hp / 3));
                let diver = near(70_000).into_iter().find(|c| self.history.get(&c.id).and_then(|h| h.front()).is_some_and(|s| d2(s.1, s.2, c.x, c.y) > sq(40_000)));
                burst.or(diver).map(|c| c.id)
            }
            GC => near(70_000).into_iter().any(|c| self.read_hp(c, tick) < 30 + shift).then_some(m.id),
            DEPLOY => (self.program.len() >= 4 && !near(80_000).is_empty()).then_some(m.id),
            _ => None,
        }
    }

    /// Runs a compiled function. True when it did something (it pays a little Bitcoin).
    fn execute(&mut self, sim: &mut StableSim<'_>, all: &[Champ], m: &Champ, c: &Compiled, target: usize, tick: usize, mb: usize) -> bool {
        let me = m.id;
        // the bugs that strike at run time
        if c.bugs.contains(&Bug::InfiniteLoop) && self.rng.chance(60, 100) {
            self.freeze(sim, me, if c.f == RECURSE { 150 } else { 90 }, "ov_loop");
            return false;
        }
        if c.bugs.contains(&Bug::Segfault) && self.rng.chance(50, 100) { self.freeze(sim, me, 60, "ov_segv"); return false; }
        if c.bugs.contains(&Bug::NullRef) && self.rng.chance(50, 100) { self.say(sim, me, "ov_null", 40, 1); return false; }
        // round 105: the terminal prints the run, so the program is seen executing what he wrote
        self.say(sim, me, &format!("ov_run_{}", FUNCS[c.f].0), 30, 1);
        let flip = c.bugs.contains(&Bug::SignFlip);
        let wrong = c.bugs.contains(&Bug::WrongTarget);
        let ap = sim.get_entity(me).map_or(40, |e| e.stat().magic_power);
        let deploy = if tick < self.deploy_until { 150 } else { 100 };
        let power = LANG[c.lang].power * self.ghz() / 300 * deploy / 100;   // x100
        let amt = |base: usize, ratio: usize| (base + ap * ratio / 100) * power / 100;
        let hit = |sim: &mut StableSim<'_>, t: usize, n: usize| {
            if flip { sim.heal(me, t, n); } else { sim.deal_damage(me, t, 0, n, AttackTypeV1::Skill); }
        };
        let pos = |id: usize| all.iter().find(|x| x.id == id).map(|x| (x.x, x.y));
        let foes = |r: i64| -> Vec<&Champ> { all.iter().filter(|x| x.team != m.team && d2(x.x, x.y, m.x, m.y) <= sq(r)).collect() };
        if mb > 0 { self.procs.push(Proc { until: tick + SPEC[c.f].4, mb, f: c.f, target }); }
        Self::fx(sim, me, "fx_send", me, 18);
        match c.f {
            PING => { hit(sim, target, amt(35, 50)); Self::fx(sim, me, if flip { "fx_heal" } else { "fx_ping" }, target, 24); }
            SHIELD | HEAL | ENCRYPT => {
                // a wrong target helps the nearest enemy instead
                let t = if wrong { all.iter().filter(|x| x.team != m.team).min_by_key(|x| d2(x.x, x.y, m.x, m.y)).map_or(target, |x| x.id) } else { target };
                if c.f == SHIELD {
                    sim.entity_add_shield(t, amt(120, 50), 180);
                    sim.add_buff(t, &timed("cd_shield", 180));
                    Self::fx(sim, me, "fx_shield", t, 24);
                } else if c.f == ENCRYPT {
                    let mut b = timed("cd_encrypt", 180);
                    b.damaged_reduce = if flip { 0 } else { 50 };
                    b.damaged_amplify = if flip { 30 } else { 0 };
                    sim.add_buff(t, &b);
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
                for e in foes(25_000) { hit(sim, e.id, amt(25, 30)); }
                Self::fx_at(sim, me, "fx_spray", (m.x, m.y), 30);
            }
            BLINK => {
                let Some((ex, ey)) = pos(target) else { return false };
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
                let Some((ex, ey)) = pos(target) else { return false };
                // across the way between him and the enemy, 60% of the way there
                let (cx, cy) = (m.x + (ex - m.x) * 6 / 10, m.y + (ey - m.y) * 6 / 10);
                let (dx, dy) = ((ex - m.x) as f64, (ey - m.y) as f64);
                let l = dx.hypot(dy).max(1.0);
                let (nx, ny) = (-dy / l * 20_000.0, dx / l * 20_000.0);
                let a = (cx - nx as i64, cy - ny as i64);
                let b = (cx + nx as i64, cy + ny as i64);
                self.walls.push(Wall { a, b, until: tick + 240, next: tick, dmg: amt(15, 15), flip });
            }
            DDOS | DDOS_ALL => {
                let targets: Vec<usize> = if c.f == DDOS { vec![target] } else { foes(80_000).iter().map(|x| x.id).collect() };
                let (n, gap, dmg, slow) = if c.f == DDOS { (20, 3, amt(4, 6), -30) } else { (8, 3, amt(3, 4), -20) };
                for &t in &targets {
                    for k in 0..n { self.hits.push(Hit { at: tick + k * gap, target: t, dmg, flip, nearest: false, fx: if k % 4 == 0 { "fx_ddos" } else { "" } }); }
                    let mut b = timed("cd_ddos", 150);
                    b.attack_speed_mult = if flip { -slow } else { slow };
                    sim.add_buff(t, &b);
                }
            }
            CLEANSE => {
                sim.entity_clear_cc(target);
                sim.entity_add_shield(target, amt(40, 20), 120);
                Self::fx(sim, me, "fx_cleanse", target, 30);
            }
            BOOST => {
                for a in all.iter().filter(|x| (x.team == m.team) != wrong && d2(x.x, x.y, m.x, m.y) <= sq(60_000)) {
                    let mut b = timed("cd_boost", 180);
                    b.attack_speed_mult = if flip { -30 } else { 30 };
                    sim.add_buff(a.id, &b);
                }
            }
            FORK => {
                // one drone a fork; a second fits in what's left of his RAM
                let n = if self.ram_used() + 2 * mb <= self.ram_cap() { 2 } else { 1 };
                for k in 0..n {
                    self.drones.push((tick + SPEC[FORK].4, tick + 15 * k));
                    if k == 1 { self.procs.push(Proc { until: tick + SPEC[FORK].4, mb, f: FORK, target }); }
                }
            }
            SWAP => {
                // the ally in trouble trades places with the enemy on them
                let Some(a) = pos(target) else { return false };
                let diver = all.iter().filter(|x| x.team != m.team).min_by_key(|x| d2(x.x, x.y, a.0, a.1)).map(|x| (x.id, x.x, x.y));
                let Some((d, dx, dy)) = diver else { return false };
                let (first, second) = if wrong { (me, target) } else { (target, d) };
                let (p1, p2) = if wrong { ((m.x, m.y), a) } else { (a, (dx, dy)) };
                sim.entity_set_pos(first, p2.0.max(0) as u64, p2.1.max(0) as u64);
                sim.entity_set_pos(second, p1.0.max(0) as u64, p1.1.max(0) as u64);
                Self::fx_at(sim, me, "fx_swap", p1, 30);
                Self::fx_at(sim, me, "fx_swap", p2, 30);
            }
            SORT => {
                // up to five enemies in a row at their centre, weakest first, across his line to them
                let mut v: Vec<&Champ> = foes(60_000);
                v.truncate(5);
                if v.is_empty() { return false; }
                v.sort_by_key(|x| (x.hp * 100 / x.max_hp.max(1), x.id));
                if flip { v.reverse(); }
                let (cx, cy) = (v.iter().map(|x| x.x).sum::<i64>() / v.len() as i64, v.iter().map(|x| x.y).sum::<i64>() / v.len() as i64);
                let (dx, dy) = ((cx - m.x) as f64, (cy - m.y) as f64);
                let l = dx.hypot(dy).max(1.0);
                let (nx, ny) = (-dy / l, dx / l);
                for (i, e) in v.iter().enumerate() {
                    let off = (i as f64 - (v.len() - 1) as f64 / 2.0) * 9_000.0;
                    let to = walls::clip(cx, cy, cx + (nx * off) as i64, cy + (ny * off) as i64);
                    sim.entity_set_pos(e.id, to.0.max(0) as u64, to.1.max(0) as u64);
                    Self::fx(sim, me, "fx_sort", e.id, 30);
                }
                let mut b = timed("cd_marked", 180);
                b.damaged_amplify = 20;
                sim.add_buff(v[0].id, &b);
            }
            KILL9 => {
                let Some(e) = all.iter().find(|x| x.id == target) else { return false };
                let executes = e.hp * 100 < e.max_hp * 15 && !flip;
                if executes { sim.deal_damage(me, target, 0, e.hp + e.max_hp, AttackTypeV1::Skill); } else { hit(sim, target, amt(60, 40)); }
                Self::fx(sim, me, "fx_kill9", target, 30);
            }
            ROLLBACK => {
                let Some(s) = self.history.get(&target).and_then(|h| h.front()).copied() else { return false };
                let Some(e) = all.iter().find(|x| x.id == target) else { return false };
                Self::fx_at(sim, me, "fx_rollback", (e.x, e.y), 30);
                if e.team == m.team && !flip {
                    if s.3 > e.hp { sim.heal(me, target, s.3 - e.hp); }
                } else {
                    sim.entity_set_pos(target, s.1.max(0) as u64, s.2.max(0) as u64);
                    Self::fx_at(sim, me, "fx_rollback", (s.1, s.2), 30);
                }
            }
            RECURSE => {
                for k in 0..8 { self.hits.push(Hit { at: tick + 6 * k, target, dmg: amt(12, 15), flip, nearest: true, fx: "fx_ping" }); }
            }
            INJECT => {
                sim.apply_cc(target, &CcV1::stun(if flip { 1 } else { 60 }));
                hit(sim, target, amt(40, 40));
                Self::fx(sim, me, "fx_inject", target, 40);
            }
            GC => {
                for e in foes(70_000).into_iter().filter(|x| x.hp * 100 < x.max_hp * 30) {
                    if !flip { sim.entity_clear_shield(e.id); }
                    hit(sim, e.id, amt(50, 40));
                    Self::fx(sim, me, "fx_gc", e.id, 30);
                }
            }
            DEPLOY => {
                self.deploy_until = tick + SPEC[DEPLOY].4;
                self.load = 0;
                sim.add_buff(me, &timed("cd_deploy", SPEC[DEPLOY].4));
            }
            _ => {}
        }
        true
    }

    /// Hits landing later, the drones, the walls.
    fn step_effects(&mut self, sim: &mut StableSim<'_>, all: &[Champ], m: &Champ, tick: usize) {
        let me = m.id;
        let foes: Vec<&Champ> = all.iter().filter(|x| x.team != m.team).collect();
        let nearest = |r: i64| foes.iter().filter(|x| d2(x.x, x.y, m.x, m.y) <= sq(r)).min_by_key(|x| d2(x.x, x.y, m.x, m.y)).map(|x| x.id);
        let due: Vec<Hit> = self.hits.iter().filter(|h| h.at <= tick).cloned().collect();
        self.hits.retain(|h| h.at > tick);
        for h in due {
            let t = if h.nearest { nearest(70_000) } else { foes.iter().any(|x| x.id == h.target).then_some(h.target) };
            let Some(t) = t else { continue };
            if h.flip { sim.heal(me, t, h.dmg); } else { sim.deal_damage(me, t, 0, h.dmg, AttackTypeV1::Skill); }
            if !h.fx.is_empty() { Self::fx(sim, me, h.fx, t, 20); }
        }
        // the drones ping the nearest enemy every half second
        self.drones.retain(|d| d.0 > tick);
        let ap = sim.get_entity(me).map_or(40, |e| e.stat().magic_power);
        for i in 0..self.drones.len() {
            if tick < self.drones[i].1 { continue; }
            self.drones[i].1 = tick + 30;
            if let Some(t) = nearest(70_000) {
                sim.deal_damage(me, t, 0, (15 + ap / 5) * self.ghz() / 300, AttackTypeV1::Skill);
                Self::fx(sim, me, "fx_ping", t, 20);
            }
        }
        let n = self.drones.len().min(2);
        if n != self.shown_drones || (n > 0 && !m.has(&format!("cd_drone{n}"))) {
            for k in 1..=2 { sim.entity_remove_buff(me, &format!("cd_drone{k}")); }
            if n > 0 { sim.add_buff(me, &BuffV1::named(&format!("cd_drone{n}"))); }
            self.shown_drones = n;
        }
        self.step_walls(sim, all, m, tick);
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

    /// Where every champion was over the last 3 s (rollback reads the oldest).
    fn step_history(&mut self, all: &[Champ], tick: usize) {
        if !tick.is_multiple_of(6) { return; }
        for c in all {
            let h = self.history.entry(c.id).or_default();
            h.push_back((tick, c.x, c.y, c.hp));
            while h.len() > 31 { h.pop_front(); }
        }
    }

    // ---------------------------------------------------------------- the HUD and the badge

    fn show(&mut self, sim: &mut StableSim<'_>, m: &Champ) {
        let heat = ((self.heat.max(0) / 100) as usize / 10).min(10);
        let ram = (self.ram_used() * 8 / self.ram_cap()).min(8);
        let disk = (self.storage.len() * 8).div_ceil(self.storage_cap()).min(8);
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
        // Bitcoin: three digits by his crest
        let btc = (self.btc / 100).min(999);
        if self.shown_btc != Some(btc) || !m.has(&format!("cd_btc_o{}", btc % 10)) {
            let old = self.shown_btc;
            for (place, d) in [("h", btc / 100), ("t", btc / 10 % 10), ("o", btc % 10)] {
                if let Some(o) = old {
                    let od = match place { "h" => o / 100, "t" => o / 10 % 10, _ => o % 10 };
                    sim.entity_remove_buff(m.id, &format!("cd_btc_{place}{od}"));
                }
                sim.add_buff(m.id, &BuffV1::named(&format!("cd_btc_{place}{d}")));
            }
            self.shown_btc = Some(btc);
        }
        let badge = (self.rank(), self.root);
        let name = if badge.0 >= ROOT { format!("cd_root{}", badge.1.unwrap_or(10)) } else { format!("cd_rank{}", badge.0) };
        if self.rank.is_some() && (self.hud.4 != Some(badge) || !m.has(&name)) {
            for k in 0..ROOT { sim.entity_remove_buff(m.id, &format!("cd_rank{k}")); }
            for p in 1..=10 { sim.entity_remove_buff(m.id, &format!("cd_root{p}")); }
            sim.add_buff(m.id, &BuffV1::named(&name));
            self.hud.4 = Some(badge);
        }
        // round 103: his rig (hidden while he's blue-screened)
        let rig = if sim.tick() < self.bsod_until { 0 } else { rig_tier(self.rank(), self.root) };
        if self.rank.is_some() && (self.shown_rig != Some(rig) || (rig > 0 && !m.has(&format!("cd_rig{rig}")))) {
            for k in 1..=5 {
                sim.entity_remove_buff(m.id, &format!("cd_rig{k}"));
                sim.entity_remove_buff(m.id, &format!("cd_rigf{k}"));
            }
            if rig > 0 {
                sim.add_buff(m.id, &BuffV1::named(&format!("cd_rig{rig}")));
                sim.add_buff(m.id, &BuffV1::named(&format!("cd_rigf{rig}")));
            }
            self.shown_rig = Some(rig);
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

impl StablePassive for Coder {
    fn clone_box(&self) -> Box<dyn StablePassive> {
        Box::new(self.clone())
    }
    fn on_dead(&mut self, sim: &mut StableSim<'_>, _player: usize) {
        // dying loses what's on the screen, not what's in the program, the shop or the wallet
        self.typing = None;
        self.term = None;
        self.procs.clear();
        self.walls.clear();
        self.hits.clear();
        self.drones.clear();
        self.heat = 4000;
        self.oc = false;
        self.oc_off_at = None;
        self.mining = false;
        self.alive = false;
        self.hud = (None, None, None, None, None);
        self.shown_btc = None;
        self.shown_drones = 0;
        self.shown_rig = None;
        self.say = None;
        self.ai.shown = None;
        if let Some(me) = self.me {
            for b in ["cd_oc", "cd_mine", "cd_drone1", "cd_drone2"] { sim.entity_remove_buff(me, b); }
            for k in 1..=5 {
                sim.entity_remove_buff(me, &format!("cd_rig{k}"));
                sim.entity_remove_buff(me, &format!("cd_rigf{k}"));
            }
        }
    }
    fn on_update(&mut self, sim: &mut StableSim<'_>, _seed: u64, player: usize, entity: usize) {
        let tick = sim.tick();
        self.me = Some(entity);
        self.now = tick;
        HI.with(|h| h.set(self.rank.is_some_and(|r| r >= HI_RANK)));
        if !self.started {
            self.started = true;
            self.rng = Rng(sim.seed() ^ (entity as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ 0xC0DE);
            self.heat = 4000;
            self.next_debug = 600;
            self.ai.pools = POOL;
            self.ai.next = 1200;
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
                    {} runs, frozen {:.1} s, {} blue screens, {} out of memory; BTC {} earned, {} held, bought [{}]; AI prompts \
                    claude {} gpt {} gemini {} ({} lite); program: {}", tick / 3600, s.shipped, s.clean, s.bugs,
                    s.caught, s.syntax_errors, s.runs, s.frozen as f64 / 60.0, s.bsods, s.ooms, s.btc_earned / 100, self.btc / 100,
                    s.bought.join(" "), s.prompts[0], s.prompts[1], s.prompts[2], s.lite_prompts,
                    self.program.iter().map(|c| format!("{}.{}{}", FUNCS[c.f].0, LANGS[c.lang], if c.bugs.is_empty() { "" } else { "*" })).collect::<Vec<_>>().join(" ")));
            }
        }
        if !self.alive {
            // (re)spawned: that's his base (installs there are instant)
            self.alive = true;
            self.last_hp = m.hp;
            self.home = Some((m.x, m.y));
        }
        self.step_history(&all_raw, tick);
        self.step_rig(sim, entity, tick);
        self.step_ai(sim, entity, tick);
        self.step_bounties(sim, player);
        self.step_effects(sim, &all, &m, tick);
        self.step_say(sim, entity, tick);
        if tick.is_multiple_of(6) { self.show(sim, &m); }
        if tick < self.frozen_until { return; }
        let fighting = all.iter().any(|c| c.team != m.team && d2(c.x, c.y, m.x, m.y) <= sq(70_000));
        self.step_overclock(sim, entity, fighting, tick);
        self.step_mining(sim, entity, fighting, tick);
        self.step_shop(sim, &m, fighting, tick);
        if tick < self.frozen_until { return; }
        // being hit can break his concentration: the line he's on starts over (not the AI's)
        if m.hp + m.max_hp / 30 < self.last_hp && self.rng.chance(25, 100) {
            if let Some(t) = self.typing.as_mut().filter(|t| t.ai.is_none()) {
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
            // round 105: it only says something when it fixed something (it showed "debugging..." 6 s every 10 s)
            if found > 0 { self.say(sim, entity, &format!("ov_debug{}", found.min(3)), 60, 1); }
        }
        self.activate_ai(sim, &all, &m, tick);
        self.choose(sim, &all, &m, tick);
        self.step_typing(sim, entity, &all, &m, tick);
        self.run_program(sim, &all, &m, tick);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn knobs(rank: usize, root: Option<usize>) -> Knobs {
        let cps100 = tv(&CPS100, CPS100_TOP, rank, root);
        Knobs { cps100, typo: tv(&TYPO, TYPO_TOP, rank, root), notice: tv(&NOTICE, NOTICE_TOP, rank, root), rank, syntax: None,
                lang_mult: true, read100: cps100, compile_pct: 100 }
    }

    /// Writes f in lang start to finish with these knobs: (ticks, logic bugs shipped, syntax errors hit).
    fn run(k: &Knobs, f: usize, lang: usize, seed: u64, think: usize) -> (usize, usize, usize) {
        let mut rng = Rng(seed);
        let mut t = if think > 0 { Typing::by_ai(f, lang, 0, (0, false), think) } else { Typing::new(f, lang, 0) };
        let mut fails = 0;
        for tick in 1..400_000 {
            match t.tick(k, &mut rng) {
                Event::Compiled(b) => return (tick, b.len(), fails),
                Event::CompileFailed(_) => fails += 1,
                Event::None => {}
            }
        }
        panic!("never compiled");
    }
    fn write(rank: usize, root: Option<usize>, f: usize, lang: usize, seed: u64) -> (usize, usize, usize) {
        run(&knobs(rank, root), f, lang, seed, 0)
    }

    fn mean_k(k: &Knobs, f: usize, lang: usize, think: usize, n: usize) -> (f64, f64, f64) {
        let mut s = (0.0, 0.0, 0.0);
        for i in 0..n {
            let (t, b, e) = run(k, f, lang, 0x1234 + i as u64 * 7919, think);
            s.0 += t as f64; s.1 += b as f64; s.2 += e as f64;
        }
        (s.0 / n as f64 / 60.0, s.1 / n as f64, s.2 / n as f64)
    }
    fn mean(rank: usize, root: Option<usize>, f: usize, lang: usize) -> (f64, f64, f64) {
        mean_k(&knobs(rank, root), f, lang, 0, 2000)
    }

    #[test]
    fn the_code_table_matches_the_functions() {
        assert_eq!(FUNCS.len(), NF);
        let names = ["ping", "shield", "heal", "scan", "spray", "blink", "slow", "cache", "chain", "firewall", "ddos", "cleanse",
            "boost", "fork", "swap", "sort", "encrypt", "ddos_all", "kill9", "rollback", "recurse", "inject", "gc", "deploy"];
        for (i, name) in names.iter().enumerate() { assert_eq!(FUNCS[i].0, *name); }
        assert_eq!(LANGS, ["py", "cpp", "rust", "js", "asm"]);
        // over all of them, Python is the shortest to write and Assembly the longest
        let total = |l: usize| (0..NF).map(|f| chars(f, l)).sum::<usize>();
        assert!(total(PY) < total(CPP) && total(PY) < total(RUST) && total(ASM) > total(CPP), "{:?}", (0..5).map(total).collect::<Vec<_>>());
        // the higher tiers are longer
        let avg = |t: usize| { let v: Vec<usize> = (0..NF).filter(|&f| tier(f) == t).map(|f| chars(f, PY)).collect(); v.iter().sum::<usize>() / v.len() };
        assert!(avg(5) > avg(1));
    }

    #[test]
    fn root_interpolates_to_number_one() {
        assert_eq!(tv(&CPS100, CPS100_TOP, 3, None), 900);
        assert_eq!(tv(&CPS100, CPS100_TOP, ROOT, Some(10)), 2200);
        assert_eq!(tv(&CPS100, CPS100_TOP, ROOT, Some(1)), 3000);
        assert_eq!(tv(&TYPO, TYPO_TOP, ROOT, Some(1)), 0);
        assert_eq!(tv(&NOTICE, NOTICE_TOP, ROOT, Some(1)), 100);
        assert_eq!(tv(&PROMPT, PROMPT_TOP, ROOT, Some(1)), 50);
    }

    #[test]
    fn writing_is_deterministic_per_seed() {
        assert_eq!(write(2, None, FIREWALL, RUST, 99), write(2, None, FIREWALL, RUST, 99));
        assert_eq!(write(2, None, ROLLBACK, ASM, 7), write(2, None, ROLLBACK, ASM, 7));
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
        // a tier-5 function in Assembly is a different story at every step
        let (kid_rb, kid_rb_bugs, _) = mean(0, None, ROLLBACK, ASM);
        let (arch_rb, arch_rb_bugs, _) = mean(6, None, ROLLBACK, ASM);
        eprintln!("rollback in Assembly: Script Kiddie {kid_rb:.0} s {kid_rb_bugs:.2} bugs, Architect {arch_rb:.0} s {arch_rb_bugs:.2} bugs");
        assert!(kid_rb > 4.0 * arch_rb && kid_rb_bugs > 10.0 * arch_rb_bugs.max(0.01));
    }

    #[test]
    fn rust_catches_bugs_at_a_price() {
        let (py_s, _, _) = mean(2, None, FIREWALL, PY);
        let (cpp_s, cpp_b, _) = mean(2, None, FIREWALL, CPP);
        let (rs_s, rs_b, _) = mean(2, None, FIREWALL, RUST);
        eprintln!("Junior firewall: py {py_s:.1} s, C++ {cpp_s:.1} s {cpp_b:.2} bugs, Rust {rs_s:.1} s {rs_b:.2} bugs");
        assert!(rs_b < cpp_b * 0.6, "{rs_b} vs {cpp_b}");
        assert!(rs_s > cpp_s && cpp_s > py_s);
    }

    #[test]
    fn assembly_hits_hardest_and_breaks_most() {
        assert!((0..5).all(|l| l == ASM || LANG[ASM].power > LANG[l].power));
        let (_, asm_b, _) = mean(2, None, CHAIN, ASM);
        let (_, py_b, _) = mean(2, None, CHAIN, PY);
        let (js_s, js_b, _) = mean(2, None, CHAIN, JS);
        let (cpp_s, _, _) = mean(2, None, CHAIN, CPP);
        eprintln!("Junior chain: asm {asm_b:.2} bugs, py {py_b:.2}, js {js_b:.2} ({js_s:.1} s vs C++ {cpp_s:.1} s)");
        assert!(asm_b > 2.0 * py_b && js_s < cpp_s);
    }

    #[test]
    fn a_saved_function_reloads_with_its_bugs_faster_off_an_ssd() {
        let k = knobs(3, None);
        let time = |ticks: usize| {
            let mut rng = Rng(5);
            let mut t = Typing::reload(SLOW, PY, &[Bug::OffByOne], 0, ticks);
            for i in 1..500 { if let Event::Compiled(b) = t.tick(&k, &mut rng) { assert_eq!(b, vec![Bug::OffByOne]); return i; } }
            panic!()
        };
        assert!(time(RELOAD[2]) < time(RELOAD[1]) && time(RELOAD[1]) < time(RELOAD[0]));
    }

    #[test]
    fn heat_throttles_the_clock_and_parts_change_the_rig() {
        let mut c = Coder { rank: Some(3), ..Coder::default() };
        c.heat = 6000;
        assert_eq!(c.ghz(), 300);
        c.oc = true;
        assert_eq!(c.ghz(), 390);
        c.heat = 9700;
        assert!(c.ghz() < 300, "{}", c.ghz());
        assert!(tv(&OC_OFF, OC_OFF_TOP, 0, None) >= 100 && tv(&OC_OFF, OC_OFF_TOP, ROOT, Some(1)) < 90);
        // the parts
        let mut d = Coder { rank: Some(3), heat: 6000, ..Coder::default() };
        assert_eq!((d.ram_cap(), d.storage_cap(), d.ghz()), (16_000, 8, 300));
        d.tiers = [2, 1, 2, 2, 2];
        assert_eq!((d.ram_cap(), d.storage_cap(), d.ghz()), (64_000, 16, 420));
        assert!(d.knobs().cps100 > Coder { rank: Some(3), heat: 6000, ..Coder::default() }.knobs().cps100, "a faster CPU types faster");
        assert_eq!(d.knobs().compile_pct, 70);
    }

    #[test]
    fn the_shop_buys_what_the_problems_call_for() {
        let stock = [0usize; 5];
        assert_eq!(needed(&stock, 3, 0, 0, 0, 0), Some(RAM));
        assert_eq!(needed(&stock, 0, 2, 0, 0, 0), Some(DISK));
        assert_eq!(needed(&stock, 0, 0, 5, 0, 0), Some(SSD));
        assert_eq!(needed(&stock, 0, 0, 0, 30, 1), Some(COOL));
        assert_eq!(needed(&stock, 0, 0, 0, 0, 0), Some(CPU), "nothing wrong: a faster CPU");
        assert_eq!(needed(&[2, 0, 0, 0, 0], 9, 0, 0, 0, 0), Some(CPU), "maxed RAM: the next best");
        assert_eq!(needed(&[2; 5], 9, 9, 9, 9, 9), None);
        assert!(PRICE.iter().all(|p| p[1] > p[0]));
    }

    #[test]
    fn mining_slows_his_typing() {
        let a = Coder { rank: Some(4), heat: 5000, ..Coder::default() };
        let b = Coder { rank: Some(4), heat: 5000, mining: true, ..Coder::default() };
        assert!(b.knobs().cps100 < a.knobs().cps100);
    }

    /// Each AI's profile: Claude the fewest syntax errors, ChatGPT the fewest logic bugs, Gemini the fastest at
    /// filling a program; the lites worse; and a Script Kiddie with any AI still ships more bugs than an Architect.
    #[test]
    fn the_copilots_have_their_profiles() {
        let n = 3000;
        let stats = |p: usize, lite: bool, rank: usize, f: usize, lang: usize| {
            let k = ai_knobs(p, lite, f, lang, tv(&PROMPT, PROMPT_TOP, rank, None), 0, rank, 900);
            mean_k(&k, f, lang, MODELS[p][lite as usize].think, n)
        };
        let (cl_s, cl_b, cl_e) = stats(CLAUDE, false, 3, ROLLBACK, RUST);
        let (gp_s, gp_b, gp_e) = stats(GPT, false, 3, ROLLBACK, RUST);
        let (ge_s, ge_b, ge_e) = stats(GEMINI, false, 3, ROLLBACK, RUST);
        eprintln!("rollback (Rust), no review: Claude {cl_s:.1} s {cl_b:.2} bugs {cl_e:.2} syntax; ChatGPT {gp_s:.1} s {gp_b:.2} {gp_e:.2}; Gemini {ge_s:.1} s {ge_b:.2} {ge_e:.2}");
        assert!(cl_e < gp_e && cl_e < ge_e, "Claude: the fewest syntax errors");
        // logic bugs before Rust's compiler: compare in Python, where nothing catches them
        let (_, cl_pb, _) = stats(CLAUDE, false, 3, ROLLBACK, PY);
        let (_, gp_pb, _) = stats(GPT, false, 3, ROLLBACK, PY);
        assert!(gp_pb < cl_pb, "ChatGPT: the fewest logic bugs ({gp_pb:.3} vs {cl_pb:.3})");
        assert!(gp_s > cl_s, "ChatGPT thinks first");
        assert_eq!(MODELS[GEMINI][0].per_prompt, 3);
        for p in 0..3 {
            let (_, fb, fe) = stats(p, false, 3, SORT, PY);
            let (_, lb, le) = stats(p, true, 3, SORT, PY);
            assert!(lb + le > fb + fe, "{}: the lite model is worse", PROVIDERS[p]);
        }
        // prompting is a skill: the same model, a Script Kiddie's prompt vs a Root's
        let (_, kid_b, kid_e) = stats(CLAUDE, false, 0, SORT, PY);
        let (_, root_b, root_e) = stats(CLAUDE, false, ROOT, SORT, PY);
        assert!(kid_b + kid_e > root_b + root_e);
        // and his review matters: a Script Kiddie with Gemini Flash vs an Architect by hand
        let kid = { let mut k = ai_knobs(GEMINI, true, CHAIN, CPP, tv(&PROMPT, PROMPT_TOP, 0, None), tv(&NOTICE, NOTICE_TOP, 0, None), 0, 300); k.compile_pct = 100; mean_k(&k, CHAIN, CPP, 0, n) };
        let arch = mean(6, None, CHAIN, CPP);
        assert!(kid.1 > arch.1, "vibe-coded {:.2} bugs vs an Architect's {:.2}", kid.1, arch.1);
    }

    #[test]
    fn believed_cost_is_rosier_at_low_rank() {
        let kid = Coder { rank: Some(0), heat: 4000, ..Coder::default() };
        let (secs, clean) = kid.believed(FIREWALL, CPP, tv(&AWARE, AWARE_TOP, 0, None));
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
        for f in 0..NF { assert!(fx(&format!("ov_run_{}", FUNCS[f].0)), "ov_run_{}", FUNCS[f].0); }
        for o in ["compiled", "saved", "syntax", "borrow", "rustc", "compile", "load", "debug1", "debug2", "debug3", "oom", "segv", "null", "loop", "bsod",
                  "thinking", "reasoning", "ratelimit", "diff", "install"] {
            assert!(fx(&format!("ov_{o}")), "ov_{o}");
        }
        for p in PROVIDERS { assert!(fx(&format!("ov_switch_{p}")), "ov_switch_{p}"); }
        for p in PARTS { for t in 1..=2 { assert!(fx(&format!("ov_buy_{p}{t}")), "ov_buy_{p}{t}"); } }
        for e in ["send", "ping", "heal", "shield", "scan", "spray", "blink_out", "blink_in", "cache", "chain", "ddos", "cleanse", "swap",
                  "sort", "kill9", "rollback", "inject", "gc"] { assert!(fx(&format!("fx_{e}")), "fx_{e}"); }
        for a in 0..8 { assert!(fx(&format!("fx_wall_{a}"))); }
        let buff = |n: String| names.contains(n.as_str());
        for b in ["cd_shield", "cd_lag", "cd_oc", "cd_mine", "cd_ddos", "cd_boost", "cd_drone1", "cd_drone2", "cd_marked", "cd_encrypt", "cd_deploy"] {
            assert!(buff(b.into()), "{b}");
        }
        for p in PROVIDERS { assert!(buff(format!("cd_ai_{p}")) && buff(format!("cd_ai_{p}_lite")), "{p}"); }
        for n in 0..=10 { assert!(buff(format!("cd_heat{n}"))); }
        for n in 0..=8 { assert!(buff(format!("cd_ram{n}")) && buff(format!("cd_disk{n}"))); }
        for d in 0..=9 { for place in ["h", "t", "o"] { assert!(buff(format!("cd_btc_{place}{d}"))); } }
        for r in 0..ROOT { assert!(buff(format!("cd_rank{r}"))); }
        for k in 1..=5 { assert!(buff(format!("cd_rig{k}")) && buff(format!("cd_rigf{k}")), "rig {k}"); }
        for t in HI_FX { assert!(fx(&format!("{t}_hi")), "{t}_hi"); }
        for p in 1..=10 { assert!(buff(format!("cd_root{p}"))); }
        assert!(text.contains("\"passive_ref\": \"tfm2_custom_ai:coder\""));
    }

    /// Round 103: the runs the Code lab must reproduce run for run (editor/coderlab.js vectors(); tools/verify_coder.py
    /// compares them). `CODER_VECTORS=write cargo test lab_vectors` rewrites the file.
    #[test]
    fn lab_vectors() {
        let mut out = Vec::new();
        let ranks = [(0, None), (1, None), (2, None), (3, None), (4, None), (5, None), (6, None), (7, None), (7, Some(1))];
        let seeds = [1u64, 99, 0x1234];
        for (r, p) in ranks {
            for f in [PING, CHAIN, FIREWALL, ROLLBACK] {
                for l in 0..LANGS.len() {
                    for s in seeds {
                        let (a, b, c) = write(r, p, f, l, s);
                        out.push(format!("write {r} {} {f} {l} {s} {a} {b} {c}", p.map_or("-".to_string(), |p: usize| p.to_string())));
                    }
                }
            }
        }
        for p in 0..3 {
            for lite in [false, true] {
                for s in seeds {
                    let k = ai_knobs(p, lite, CHAIN, CPP, PROMPT[3], NOTICE[3], 3, CPS100[3]);
                    let (a, b, c) = run(&k, CHAIN, CPP, s, MODELS[p][lite as usize].think);
                    out.push(format!("ai {p} {} {CHAIN} {CPP} {s} {a} {b} {c}", lite as u8));
                }
            }
        }
        let text = out.join("\n") + "\n";
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/src/coder_vectors.txt");
        if std::env::var("CODER_VECTORS").as_deref() == Ok("write") { std::fs::write(path, &text).unwrap(); }
        assert_eq!(std::fs::read_to_string(path).unwrap_or_default(), text, "coder_vectors.txt is stale: CODER_VECTORS=write cargo test lab_vectors");
    }

    /// Round 105: one status line at a time: a freeze's line holds the slot until it ends, others replace each other.
    #[test]
    fn one_status_line_at_a_time() {
        assert!(say_accepts(&None, 100, 1));
        let freeze = Some(("ov_bsod".to_string(), 250, 2));
        assert!(!say_accepts(&freeze, 100, 1), "a blue screen isn't covered by a run line");
        assert!(say_accepts(&freeze, 100, 2));
        assert!(say_accepts(&freeze, 250, 1), "it frees the slot when it ends");
        let run = Some(("ov_run_ping".to_string(), 130, 1));
        assert!(say_accepts(&run, 110, 1), "the newest line replaces the one shown");
    }

    #[test]
    fn the_rig_follows_the_rank() {
        let tiers: Vec<usize> = (0..ROOT).map(|r| rig_tier(r, None)).collect();
        assert_eq!(tiers, [0, 0, 0, 0, 1, 2, 3]);
        assert_eq!(rig_tier(ROOT, Some(10)), 4);
        assert_eq!(rig_tier(ROOT, Some(2)), 4);
        assert_eq!(rig_tier(ROOT, Some(1)), 5);
        HI.with(|h| h.set(false));
        assert_eq!(fx_name("fx_ping"), format!("{P}fx_ping"));
        HI.with(|h| h.set(true));
        assert_eq!(fx_name("fx_ping"), format!("{P}fx_ping_hi"));
        assert_eq!(fx_name("fx_scan"), format!("{P}fx_scan"));
        HI.with(|h| h.set(false));
    }

    #[test]
    fn wall_distance() {
        assert_eq!(seg_d2((0, 10), (-10, 0), (10, 0)), 100);
        assert_eq!(seg_d2((20, 0), (-10, 0), (10, 0)), 100);
    }
}
