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


use crate::mastery::{signature, Book, Record};
use crate::{champions, d2, sq, timed, walls, Champ};
use mod_api_stable::{AttackTypeV1, BuffDurationV1, BuffV1, CcKindV1, CcV1, StablePassive, StableSim};
use std::collections::{HashMap, VecDeque};

pub static BOOK: Book = Book::new("coder");

pub const RANK_NAMES: [&str; 8] = ["Script Kiddie", "Intern", "Junior", "Developer", "Senior", "Staff", "Architect", "Root"];
pub const ROOT: usize = 7;
const P: &str = "tfm2_custom_coder_";

// ------------------------------------------------------------------ the rank tables (index 7 = Root #10; then #1)

/// Typing speed, characters a second x100 (about WPM x 12 / 100).
const CPS100: [usize; 8] = [450, 750, 1050, 1350, 1800, 2250, 2850, 3300];  // round 107: x1.5
const CPS100_TOP: usize = 4500;
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
// round 107: 13 languages (coder_code.rs LANGS order). Each has its own feel.
const LANG: [Lang; 13] = [
    Lang { typo: 80, syntax: 50, compile: 0, power: 80, heat: 150, ram: 150, catch: 0, bugs: [20, 25, 10, 35, 10, 0, 0] },     // py
    Lang { typo: 90, syntax: 40, compile: 0, power: 90, heat: 150, ram: 120, catch: 0, bugs: [15, 15, 10, 30, 30, 0, 0] },     // js
    Lang { typo: 95, syntax: 45, compile: 40, power: 95, heat: 160, ram: 120, catch: 50, bugs: [15, 20, 10, 20, 20, 0, 0] },   // ts
    Lang { typo: 120, syntax: 60, compile: 60, power: 130, heat: 400, ram: 80, catch: 0, bugs: [15, 20, 10, 0, 10, 25, 20] },  // cpp
    Lang { typo: 110, syntax: 70, compile: 150, power: 120, heat: 250, ram: 70, catch: 80, bugs: [25, 35, 20, 0, 20, 0, 0] },  // rust
    Lang { typo: 85, syntax: 45, compile: 30, power: 105, heat: 200, ram: 90, catch: 30, bugs: [20, 20, 15, 10, 10, 10, 0] },  // go
    Lang { typo: 100, syntax: 55, compile: 120, power: 110, heat: 250, ram: 160, catch: 40, bugs: [15, 20, 10, 30, 10, 0, 10] },// java
    Lang { typo: 100, syntax: 55, compile: 80, power: 110, heat: 250, ram: 120, catch: 40, bugs: [18, 22, 12, 15, 12, 8, 5] }, // cs
    Lang { typo: 70, syntax: 40, compile: 0, power: 75, heat: 120, ram: 60, catch: 0, bugs: [20, 25, 10, 30, 15, 0, 0] },      // lua
    Lang { typo: 150, syntax: 60, compile: 260, power: 120, heat: 250, ram: 90, catch: 95, bugs: [20, 20, 20, 10, 20, 0, 0] }, // hs (almost no logic bugs)
    Lang { typo: 95, syntax: 45, compile: 0, power: 95, heat: 180, ram: 70, catch: 0, bugs: [35, 20, 15, 10, 15, 0, 0] },      // sh (fragile: wrong targets)
    Lang { typo: 90, syntax: 50, compile: 0, power: 95, heat: 150, ram: 100, catch: 30, bugs: [20, 25, 10, 15, 10, 0, 0] },    // sql
    Lang { typo: 160, syntax: 30, compile: 0, power: 160, heat: 600, ram: 40, catch: 0, bugs: [15, 15, 30, 0, 10, 30, 0] },    // asm
];
#[allow(dead_code)] pub const PY: usize = 0;
#[allow(dead_code)] pub const JS: usize = 1;
#[allow(dead_code)] pub const TS: usize = 2;
pub const CPP: usize = 3;
pub const RUST: usize = 4;
pub const GO: usize = 5;
#[allow(dead_code)] pub const JAVA: usize = 6;
#[allow(dead_code)]
pub const CS: usize = 7;
#[allow(dead_code)]
pub const LUA: usize = 8;
#[allow(dead_code)] pub const HS: usize = 9;
#[allow(dead_code)] pub const SH: usize = 10;
#[allow(dead_code)] pub const SQL: usize = 11;
pub const ASM: usize = 12;
/// Round 107: Go daemons cost less CPU (goroutines are cheap); Java daemons cost more RAM.
fn lang_cpu(lang: usize, cpu: i32) -> i32 {
    match lang { GO => cpu * 70 / 100, _ => cpu }
}

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

use crate::coder_code::*;   // round 108: IDEAL (as gen_ideal), KIND, NF and every F_<NAME>
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
    // round 107: the 76 new functions (coder_code.rs order 24..99)
    (300, 1500, 2000, 25, 360),     // cloud_deploy
    (600, 2000, 0, 30, 0),          // docker
    (900, 2500, 3000, 25, 240),     // kubernetes
    (600, 1000, 1000, 15, 480),     // cron
    (600, 1500, 1500, 25, 180),     // load_balancer
    (480, 1500, 0, 25, 0),          // cdn
    (360, 1800, 0, 30, 0),          // serverless
    (900, 1000, 1000, 20, 480),     // ci_cd
    (480, 2000, 0, 30, 0),          // canary_deploy
    (600, 2500, 0, 35, 0),          // chaos_monkey
    (600, 2000, 0, 30, 0),          // terraform
    (900, 1000, 2000, 20, 999),     // autoscale
    (600, 1000, 0, 25, 0),          // git_revert
    (480, 1500, 0, 30, 0),          // git_blame
    (480, 2000, 0, 30, 0),          // git_push_force
    (480, 1000, 0, 25, 0),          // git_stash
    (600, 1000, 0, 25, 0),          // cherry_pick
    (900, 500, 0, 25, 0),           // rebase
    (900, 2500, 0, 40, 0),          // merge_conflict
    (600, 1500, 0, 30, 0),          // hotfix
    (480, 2000, 0, 35, 0),          // sql_injection
    (600, 2000, 0, 35, 0),          // ransomware
    (600, 1000, 1000, 15, 240),     // keylogger
    (480, 1500, 0, 30, 0),          // dns_spoof
    (480, 1500, 0, 25, 0),          // honeypot
    (900, 2500, 4000, 40, 360),     // botnet
    (600, 2500, 0, 50, 0),          // buffer_overflow
    (600, 1200, 0, 20, 0),          // vpn
    (900, 3000, 0, 40, 0),          // fork_bomb
    (600, 1800, 0, 30, 0),          // phishing
    (600, 2500, 0, 50, 0),          // zero_day
    (480, 1200, 0, 20, 0),          // port_scan
    (480, 1500, 0, 30, 0),          // mitm
    (480, 2200, 0, 35, 0),          // brute_force
    (480, 3000, 0, 55, 0),          // cuda_kernel
    (900, 1000, 2000, 25, 480),     // tensor_core
    (600, 1000, 1500, 20, 999),     // train_model
    (600, 2000, 0, 25, 0),          // ray_tracing
    (480, 2000, 0, 40, 0),          // quantum
    (900, 1000, 2000, 20, 480),     // llm_agent
    (600, 1800, 0, 30, 0),          // deepfake
    (480, 1500, 1000, 25, 180),     // diffusion
    (480, 2500, 0, 45, 0),          // overfit
    (900, 1000, 1500, 15, 480),     // neural_net
    (480, 1000, 0, 20, 0),          // sql_query
    (900, 1000, 1500, 15, 480),     // index_scan
    (600, 2500, 0, 40, 0),          // sharding
    (600, 1500, 2000, 25, 180),     // replication
    (900, 500, 1500, 20, 999),      // backup
    (600, 1500, 0, 30, 0),          // migrate
    (900, 2000, 0, 35, 0),          // transaction
    (900, 2500, 0, 40, 0),          // deadlock
    (600, 2000, 0, 30, 0),          // vacuum
    (900, 3000, 0, 45, 0),          // map_reduce
    (900, 500, 1500, 20, 999),      // blockchain
    (900, 500, 1000, 15, 480),      // bloom_filter
    (480, 1500, 0, 25, 0),          // traceroute
    (480, 2200, 0, 35, 0),          // tcp_handshake
    (300, 2000, 0, 30, 0),          // udp_flood
    (600, 1500, 1500, 25, 360),     // websocket
    (480, 1500, 0, 25, 0),          // rate_limiter
    (600, 1000, 0, 25, 0),          // oauth
    (900, 1000, 1500, 20, 480),     // webhook
    (480, 1800, 0, 30, 0),          // captcha
    (480, 1500, 0, 25, 0),          // cors
    (600, 1000, 1500, 20, 240),     // api_gateway
    (600, 1000, 0, 25, 0),          // sudo
    (480, 1500, 0, 25, 0),          // chmod
    (480, 1500, 0, 25, 0),          // nice
    (600, 2000, 0, 30, 0),          // kill_all
    (600, 1500, 0, 25, 0),          // dijkstra
    (600, 2000, 0, 45, 0),          // binary_search
    (480, 2000, 0, 30, 0),          // quicksort
    (900, 1000, 1500, 20, 360),     // dynamic_prog
    (600, 1800, 0, 30, 0),          // regex
    (900, 1000, 1500, 20, 240),     // mutex
];
/// Round 108: each new function's trigger, range, class, and two numbers (index f - 24). The class is a coarse kind
/// the brain values it by (and the Code lab's arena models it by); what it actually does is execute_new()'s arm for it.
/// trigger: 0 nearest enemy in r; 1 >= 2 enemies in r; 2 most-hurt ally (< 55%); 3 a healthy ally under pressure;
///          4 himself low (< 40%); 5 a fight within 70000; 6 a diver on him; 7 an ally near and a fight in 120000;
///          8 passive (never runs: it works while it's in his program); 9 an endangered ally (< 40%, an enemy on them);
///          10 an ally just hit (its attacker); 11 an ally hit by two or more; 12 a last script to re-run; 13 a last
///          effect to repeat; 14 a shield to replicate; 15 himself under 15%; 16 he lost 20% in 3 s; 17 an enemy under
///          a %; 18 the last enemy he hit; 19 a bug in his program; 20 a daemon cooling down; 21 a cloud bill he can pay;
///          22 nothing stashed; 23 an ally with a buff to copy; 24 no combo armed; 25 a buffed / shielded enemy.
/// class  : 0 hit, 1 area, 2 heal, 3 shield, 4 crowd control, 5 ally buff, 6 passive / meta, 7 utility.
pub const NB: [(u8, i64, u8, i32, i32); 76] = [
    (21, 70_000, 0, 35, 50),   // cloud_deploy: a cloud node pings (no local CPU; 0.5 BTC a run)
    (9, 0, 7, 90, 0),          // docker: an endangered ally is containerised (untargetable 1.5 s)
    (8, 0, 6, 0, 0),           // kubernetes: every daemon run gets a 50% replica
    (12, 0, 6, 0, 0),          // cron: re-runs his last script (every 8 s)
    (3, 0, 5, 30, 180),        // load_balancer: the ally under pressure takes -30% for 3 s
    (7, 0, 5, 25, 180),        // cdn: allies near him +25% move speed for 3 s
    (1, 80_000, 1, 25, 30),    // serverless: a hit on every enemy in range (no local CPU; 0.3 BTC each)
    (8, 0, 6, 0, 0),           // ci_cd: half the logic bugs of every later compile are caught
    (0, 80_000, 6, 50, 0),     // canary_deploy: his strongest daemon test-run at 50% on one enemy
    (1, 80_000, 6, 60, 0),     // chaos_monkey: three random effects from his program, random targets, 60%
    (6, 50_000, 7, 180, 0),    // terraform: a server-block wall between him and the diver
    (8, 0, 6, 2, 0),           // autoscale: +2 program slots while enemies outnumber his team near him
    (16, 0, 2, 0, 0),          // git_revert: his health back to 3 s ago
    (0, 90_000, 4, 20, 300),   // git_blame: the most dangerous enemy near: marked +20% damage taken for 5 s
    (1, 50_000, 4, 3_000, 14), // git_push_force: knocks back every enemy near him
    (22, 0, 3, 160, 0),        // git_stash: a shield stored, popped by itself when he drops under 30%
    (23, 0, 5, 0, 0),          // cherry_pick: copies an ally's best buff onto himself
    (20, 0, 6, 0, 0),          // rebase: resets his daemons' cooldowns
    (1, 60_000, 4, 60, 0),     // merge_conflict: two enemies collide: both stunned 1 s
    (19, 0, 6, 0, 0),          // hotfix: patches his program's worst bug, then runs that function at once
    (25, 60_000, 4, 120, 0),   // sql_injection: DROP TABLE: an enemy's buffs and shields are gone
    (0, 60_000, 4, 120, 0),    // ransomware: an enemy's skills locked 2 s
    (0, 70_000, 7, 240, 0),    // keylogger: one enemy's health read exactly and marked for 4 s
    (0, 60_000, 4, 90, 0),     // dns_spoof: an enemy taunted onto his tankiest ally for 1.5 s
    (3, 0, 5, 30, 180),        // honeypot: an ally reflects 30% of damage for 3 s
    (5, 0, 0, 3, 0),           // botnet: 3-5 mini drones (by rank)
    (0, 55_000, 0, 150, 70),   // buffer_overflow: a huge hit; 40% he segfaults himself
    (4, 0, 7, 120, 0),         // vpn: invisible 2 s
    (1, 80_000, 1, 12, 8),     // fork_bomb: 12 hits over every enemy; his CPU maxed
    (0, 70_000, 4, 60, 0),     // phishing: an enemy pulled toward his team, charmed 1 s
    (0, 70_000, 0, 25, 0),     // zero_day: true damage, 25% of max health
    (1, 80_000, 4, 80_000, 0), // port_scan: every enemy near read exactly; the weakest marked
    (0, 70_000, 2, 60, 180),   // mitm: steals an enemy's healing for 3 s (he heals instead)
    (0, 60_000, 0, 10, 5),     // brute_force: 10 hits, each harder
    (0, 80_000, 0, 120, 80),   // cuda_kernel (GPU): a massive beam on one target
    (8, 0, 6, 0, 0),           // tensor_core: AI pools refill x2; lite models write like flagships
    (8, 0, 6, 15, 0),          // train_model: +2% power a run this fight, up to +30%
    (1, 120_000, 4, 180, 0),   // ray_tracing (GPU): every enemy near read exactly and marked 3 s
    (0, 70_000, 0, 40, 0),     // quantum (GPU): 50/50: a triple hit or nothing
    (13, 0, 6, 20, 0),         // llm_agent: a mini AI writes one script for him (every 12 s)
    (0, 70_000, 4, 120, 0),    // deepfake (GPU): an enemy fears its own shadow 2 s (attacks nothing)
    (2, 0, 2, 25, 5),          // diffusion (GPU): heal over time on the most-hurt ally, "denoising"
    (18, 80_000, 0, 140, 60),  // overfit (GPU): huge damage to the last enemy he hit
    (8, 0, 6, 80, 0),          // neural_net: his typos -20% while it's in his program
    (1, 120_000, 4, 40, 0),    // sql_query: SELECT: every enemy under 40% read exactly and marked
    (8, 0, 6, 0, 0),           // index_scan: his program checks its triggers twice as often
    (1, 90_000, 1, 180, 0),    // sharding: one big hit split over up to 3 enemies
    (14, 0, 3, 0, 0),          // replication: his last shield copied onto a second ally
    (15, 0, 2, 30, 0),         // backup: 30% of his health restored when he'd drop under 15% (once a minute)
    (9, 0, 7, 0, 0),           // migrate: an endangered ally teleported to his side
    (24, 0, 6, 3, 0),          // transaction: his next 3 scripts run as one combo (+30% each)
    (1, 80_000, 4, 90, 0),     // deadlock: two enemies frozen 1.5 s
    (1, 80_000, 4, 2_400, 20), // vacuum: nearby enemies pulled together onto him
    (1, 120_000, 1, 90, 0),    // map_reduce (GPU): every visible enemy in 120000 hit, split
    (8, 0, 6, 0, 0),           // blockchain: kills and assists mint double Bitcoin and a shield
    (8, 0, 6, 15, 0),          // bloom_filter: 15% less damage taken while it's in his program
    (6, 70_000, 4, 40, 120),   // traceroute: the diver slowed 40% for 2 s
    (0, 55_000, 0, 30, 25),    // tcp_handshake: SYN, SYN-ACK, ACK: 3 hits, the 3rd stuns
    (1, 60_000, 1, 8, 10),     // udp_flood: a fast spray with no aim
    (0, 70_000, 0, 20, 240),   // websocket: a tether pinging one enemy every 0.5 s for 4 s
    (0, 55_000, 4, 40, 180),   // rate_limiter: an enemy's attack speed -40% for 3 s
    (23, 0, 5, 180, 0),        // oauth: borrows an ally's buff for 3 s
    (10, 0, 0, 20, 20),        // webhook: when an ally is hit, pings the attacker
    (1, 70_000, 4, 60, 0),     // captcha: the enemy with the lowest attack stunned 1 s
    (0, 60_000, 4, 180, 0),    // cors: an enemy can't be healed for 3 s
    (7, 0, 5, 10, 180),        // api_gateway: allies near him +10% attack for 3 s
    (20, 0, 6, 150, 0),        // sudo: one daemon's cooldown overridden; his next run +50%
    (25, 60_000, 4, 180, 0),   // chmod: an enemy's buffs stripped, and kept off for 3 s
    (0, 60_000, 4, 30, 180),   // nice: an enemy slowed 30% for 3 s
    (5, 80_000, 1, 20, 20),    // kill_all: every enemy process near him killed: shields gone and a hit
    (9, 0, 7, 0, 0),           // dijkstra: dashes him along the shortest path to an endangered ally
    (17, 70_000, 0, 20, 0),    // binary_search: an enemy under 20% halved
    (1, 60_000, 4, 3_000, 0),  // quicksort: the weakest enemy pulled to the front (onto him)
    (13, 0, 6, 50, 0),         // dynamic_prog: his last effect again at 50% (every 6 s)
    (0, 70_000, 4, 90, 180),   // regex: a trap where the enemy stands: the first in is rooted 1.5 s
    (11, 0, 5, 40, 120),       // mutex: an ally hit by two or more takes -40% for 2 s
];
/// Round 108: the functions that need a GPU (the GPU / AI ones that hit, and map_reduce): at 25% of their power
/// without one, and they heat the rig (GPU_HEAT).
const GPU_FX: [usize; 7] = [F_CUDA_KERNEL, F_RAY_TRACING, F_QUANTUM, F_DEEPFAKE, F_DIFFUSION, F_OVERFIT, F_MAP_REDUCE];
/// Round 108: functions that run other functions (never repeated by cron / dynamic_prog / chaos_monkey / canary).
const META: [usize; 7] = [F_KUBERNETES, F_CRON, F_CANARY_DEPLOY, F_CHAOS_MONKEY, F_LLM_AGENT, F_HOTFIX, F_DYNAMIC_PROG];
fn passive(f: usize) -> bool {
    f >= 24 && NB[f - 24].0 == 8
}
fn class(f: usize) -> u8 {
    if f >= 24 { NB[f - 24].2 } else { 0 }
}
fn needs_gpu(f: usize) -> bool {
    GPU_FX.contains(&f)
}
/// Round 108: functions that run in the cloud (billed in Bitcoin instead of loading his CPU).
fn cloud_cpu(f: usize) -> bool {
    matches!(f, F_CLOUD_DEPLOY | F_SERVERLESS)
}
/// Round 107 (buff all): every function's cooldown is 0.8x, and its damage / heal / shield 1.3x.
fn cooldown_ticks(f: usize) -> usize {
    SPEC[f].0 * 80 / 100
}
const POWER_BUFF: usize = 130;

/// The language a perfect judge writes each function in (generated from coder_functions.py). From Architect up the hot
/// loops go to Assembly (ideal()); round 103: not ping().
fn gen_ideal(f: usize) -> usize {
    IDEAL[f]
}
/// Round 107: daemon program slots by rank (Script Kiddie .. Architect, then Root, Zero-Day).
fn slots(rank: usize, root: Option<usize>) -> usize {
    const S: [usize; 8] = [4, 5, 5, 6, 7, 8, 9, 10];
    if rank < ROOT { S[rank] } else if root == Some(1) { 12 } else { 11 }
}
const HOME_R: i64 = 40_000;
/// Round 106: the IDE window over his head (Claude outputs/coder/coder_code.py): the window, up to three code lines and
/// the status line are point effects placed over him (px from his centre, 950 world units a px) and re-placed every
/// TERM_EVERY ticks so they follow him (their frames are 0.11 s: the game plays an effect's animation to its end).
const PX: i64 = 950;
const WIN_DY: i64 = 66 * PX;
const ROW_DY: [i64; 2] = [72 * PX, 60 * PX];  // round 107: two rows (lighter)
const ROW_DX: i64 = -5 * PX;
const STATUS_DY: i64 = 47 * PX;
/// Reveal steps of a code line (1/3, 2/3, all of it).
const STEPS: usize = 3;
const TERM_EVERY: usize = 8;  // round 107: fewer effect re-places (lighter)

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

/// Round 106: each rank writes in its own editor theme (coder_code.py THEMES): Script Kiddie's green terminal, an
/// amber CRT (Intern, Junior), a dark editor (Developer, Senior), charcoal and gold (Staff), a blueprint (Architect),
/// red on black (Root), black and gold (Zero-Day).
pub fn theme_of(rank: usize, root: Option<usize>) -> usize {
    match rank {
        0 => 0,
        1 | 2 => 1,
        3 | 4 => 2,
        5 => 3,
        6 => 4,
        _ => if root == Some(1) { 6 } else { 5 },
    }
}
#[allow(dead_code)]   // the tests and tools/verify_coder.py read it
pub const THEMES: usize = 7;
/// Round 106: his outfit and keyboard (coder_ranks.py fit<k> / kb<k>, buffs cd_fit<k> / cd_kb<k>): one per rank,
/// Root and Zero-Day their own.
pub fn fit_of(rank: usize, root: Option<usize>) -> usize {
    if rank < ROOT { rank } else if root == Some(1) { 8 } else { 7 }
}

/// The code lines the window shows now: (row, line, step). While he types, the line he's on is at the bottom with
/// the two above it; while he fixes a line it sits in the middle; reviewing, compiling or loading shows the last ones.
fn term_rows(t: &Typing) -> Vec<(usize, usize, usize)> {
    let ls = lines(t.f, t.lang);
    let n = ls.len();
    if n == 0 { return Vec::new(); }
    let (cur, typed, last) = match &t.phase {
        Phase::Think { .. } => return Vec::new(),
        Phase::Type { line, done } => (*line, Some(*done / 100), *line),
        Phase::Fix { queue, done } => {
            let l = queue.first().copied().unwrap_or(n - 1);
            let first = l.saturating_sub(1).min(n.saturating_sub(3));
            (l, Some(*done / 100), (first + 2).min(n - 1))
        }
        _ => (n - 1, None, n - 1),
    };
    let first = last.saturating_sub(1);
    (first..=last).enumerate().map(|(row, l)| {
        let step = match typed {
            Some(c) if l == cur => (c * STEPS / ls[l].0.max(1) + 1).min(STEPS),
            _ => STEPS,
        };
        (row, l, step)
    }).collect()
}

/// Round 107: a function is written in a subset of the 13 languages (empty slice elsewhere).
fn avail(f: usize, lang: usize) -> bool {
    !FUNCS[f].2[lang].is_empty()
}
fn ideal(f: usize, rank: usize) -> usize {
    if rank >= 6 && avail(f, ASM) && matches!(f, CHAIN | DDOS | RECURSE) { ASM }
    else { gen_ideal(f) }
}
/// A language he actually writes f in: a random one of the few it's written in.
fn any_lang(f: usize, rng: &mut Rng) -> usize {
    let langs: Vec<usize> = (0..LANG.len()).filter(|&l| avail(f, l)).collect();
    if langs.is_empty() { gen_ideal(f) } else { langs[rng.below(langs.len())] }
}

fn lines(f: usize, lang: usize) -> &'static [(usize, usize)] {
    FUNCS[f].2[lang]
}
fn is_script(f: usize) -> bool {
    KIND[f]
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
pub const GPU: usize = 5;
pub const NPARTS: usize = 6;
pub const PARTS: [&str; NPARTS] = ["ram", "disk", "ssd", "cool", "cpu", "gpu"];
/// Round 108: every part has tiers 0..4: stock, two consumer upgrades, a workstation part (tier 3) and a data-center
/// part (tier 4). Price (Bitcoin) of tiers 1..4.
pub const MAX_TIER: usize = 4;
pub const PRICE: [[usize; MAX_TIER]; NPARTS] = [
    [60, 160, 450, 1100],    // RAM: 64 GB, 128 GB, 512 GB ECC, 2 TB ECC
    [40, 110, 300, 800],     // storage: 32, 64, 128 RAID, 256 SAN
    [50, 130, 350, 900],     // SSD, NVMe, NVMe RAID0, RAM disk
    [50, 150, 400, 1000],    // air, liquid, custom loop, immersion
    [80, 220, 600, 1500],    // 3.6 GHz, 4.2 GHz, Threadripper, dual EPYC
    [90, 260, 700, 1800],    // RTX 4070, RTX 4090, RTX 6000 Ada, H100 rack
];
/// RAM in MB (tier 3+ is ECC: no leaks); storage in save slots; reload ticks (tier 4 is a RAM disk: instant, but a
/// blue screen or an outage wipes it); compile time %.
const RAM_MB: [usize; 5] = [32_000, 64_000, 128_000, 512_000, 2_000_000];
const STORAGE: [usize; 5] = [16, 32, 64, 128, 256];
const RELOAD: [usize; 5] = [48, 24, 6, 3, 0];
const COMPILE_PCT: [usize; 5] = [100, 80, 60, 50, 40];
/// Cooling, C x100 a tick: stock, air, liquid, custom loop, immersion (whose pump can fail).
const COOLING: [i32; 5] = [8, 12, 17, 24, 34];
/// Base clock, GHz x100 (overclock adds 90): 3.0, 3.6, 4.2, Threadripper 5.0, dual EPYC 5.6.
const GHZ: [usize; 5] = [300, 360, 420, 500, 560];
/// % of a run's CPU load the CPU feels (the many-core parts shrug it off).
const CPU_LOAD: [i32; 5] = [100, 100, 100, 50, 35];
/// GPU: none / RTX 4070 / RTX 4090 / RTX 6000 Ada / H100 rack. A GPU function runs at this % of power by tier.
const GPU_POWER: [usize; 5] = [25, 70, 100, 120, 150];
/// Heat (C x100) a GPU function adds: on the CPU without a GPU, then by card.
const GPU_HEAT: [i32; 5] = [300, 120, 150, 180, 250];
/// Watts: RAM, storage, SSD, cooling (pumps) always; the CPU idles at 30% of its draw and the GPU at 15%.
const PART_W: [[i32; 5]; NPARTS] = [
    [5, 10, 20, 60, 200],
    [5, 10, 15, 40, 80],
    [5, 5, 8, 15, 30],
    [0, 5, 20, 60, 150],
    [65, 90, 125, 280, 560],
    [0, 200, 450, 300, 700],
];
/// The circuit he's on trips above this (a 3 s outage: the rig reboots and unsaved daemons are lost).
const BREAKER_W: i32 = 1800;
/// Data-center billing: BTC x100 a second for each tier-4 part he owns; when he can't pay, the cloud rate-limits him
/// to tier-2 performance until he has BILL_RESUME again.
const UPKEEP: usize = 25;
const BILL_RESUME: usize = 2_000;
/// % a second that an immersion tank's pump fails (cooling 0 for PUMP_TICKS).
const PUMP_FAIL: usize = 1;
const PUMP_TICKS: usize = 300;
const OUTAGE_TICKS: usize = 180;
/// BTC x100 a data-center owner starts with (about 3 minutes of its bill, less what he earns).
const START_CREDIT: usize = 25_000;

/// Round 108: at most this many function effects a second per Coder (the window and status line don't count); past
/// it the effect is skipped, never the damage, heal or shield.
const FX_BUDGET: usize = 12;
const FX_REFILL: usize = 5;

/// Round 108: what rank r starts the game owning (the pros already have their rigs): tiers per part.
/// Script Kiddie..Junior stock; Developer tier 1; Senior 1-2; Staff 2 and an RTX 4090; Architect a tier-3 workstation;
/// Root a tier-4 data center; Zero-Day the same, maxed.
fn start_tiers(rank: usize, _root: Option<usize>) -> [usize; NPARTS] {
    match rank {
        0..=2 => [0; NPARTS],
        3 => [1; NPARTS],
        4 => [2, 1, 2, 1, 1, 1],
        5 => [2; NPARTS],
        6 => [3; NPARTS],
        _ => [4; NPARTS],
    }
}

/// What his problems this game call for (the part he'd buy next with perfect judgement): counters of OOMs, storage
/// overwrites, reloads, throttled seconds and blue screens; the CPU/GPU when nothing is wrong. None when maxed.
pub fn needed(tiers: &[usize; NPARTS], ooms: usize, overwrites: usize, reloads: usize, hot_secs: usize, bsods: usize) -> Option<usize> {
    let score = [ooms * 3, overwrites * 2, reloads, bsods * 4 + hot_secs / 10, 1, 1];
    (0..NPARTS).filter(|&p| tiers[p] < MAX_TIER).max_by_key(|&p| (score[p], p == CPU))
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
const POOL: [i32; 3] = [15_000, 24_000, 36_000];  // round 107: x1.5
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
    // round 108: the data-center risks
    pub outages: usize,
    pub pump_fails: usize,
    pub upkeep_paid: usize,
    pub lapsed_secs: usize,
    pub fx_skipped: usize,
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
    cooldown: Vec<usize>,   // per function (len NF); filled on first update
    last_run: Vec<usize>,   // round 107: when each function last ran (round-robin, variety)
    last_write: Vec<(usize, usize)>,  // round 107: (function, tick) he wrote recently (no-spam)
    pending_script: Option<(usize, usize, Vec<Bug>, usize)>,  // round 107: a compiled script waiting for its trigger (f, lang, bugs, until)
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
    pub tiers: [usize; NPARTS],
    /// round 108: GPU load x100 (decays), the immersion pump's failure, data-center billing lapsed (rate-limited)
    gpu_load: i32,
    pump_until: usize,
    lapsed: bool,
    /// round 108: the new functions' state: a replica / test run's power % (0 = a full run), train_model's runs,
    /// sudo's next-run boost, a transaction's combo left, git_stash's shield, what cron / dynamic_prog / overfit /
    /// replication repeat, a websocket's tether (target, until, next ping), the keylogger and chmod (target, until),
    /// regex traps (x, y, until), diffusion's heals (at, target, amount) and delayed stuns (at, target, ticks)
    scale: usize,
    trained: usize,
    boost_next: usize,
    combo: usize,
    stash: usize,
    last_script: Option<Compiled>,
    last_effect: Option<(Compiled, usize)>,
    last_shield: Option<(usize, usize, usize)>,
    last_hit: Option<usize>,
    tether: Option<(usize, usize, usize)>,
    keylog: Option<(usize, usize)>,
    chmod: Option<(usize, usize)>,
    traps: Vec<(i64, i64, usize)>,
    hots: Vec<(usize, usize, usize)>,
    ccs: Vec<(usize, usize, u64)>,
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
    /// round 108: the power and GPU meters shown
    hud_pow: Option<usize>,
    hud_gpu: Option<usize>,
    shown_btc: Option<usize>,
    shown_drones: usize,
    drone_pings: usize,
    /// round 108: the cosmetic effect budget (tokens x1: one comes back every FX_REFILL ticks, up to FX_BUDGET)
    fx_tokens: usize,
    fx_refill_at: usize,
    shown_rig: Option<usize>,
    shown_fit: Option<usize>,
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
    /// Round 108: the tier a part performs at (a lapsed data-center bill rate-limits tier 3-4 parts to tier 2).
    fn part(&self, p: usize) -> usize {
        if self.lapsed { self.tiers[p].min(2) } else { self.tiers[p] }
    }
    fn ram_cap(&self) -> usize {
        RAM_MB[self.part(RAM)]
    }
    fn storage_cap(&self) -> usize {
        STORAGE[self.part(DISK)]
    }
    /// Round 108: what the rig draws (W), plus `cpu` more load (x100) and the GPU busy if `gpu`.
    fn watts(&self, cpu: i32, gpu: bool) -> i32 {
        let t = |p: usize| self.tiers[p];
        let base: i32 = [RAM, DISK, SSD, COOL].iter().map(|&p| PART_W[p][t(p)]).sum();
        let load = (self.load + cpu).clamp(0, 10_000);
        let mut cpu_w = PART_W[CPU][t(CPU)] * (30 + 70 * load / 10_000) / 100;
        if self.oc { cpu_w = cpu_w * 125 / 100; }
        let busy = gpu || self.gpu_load > 3000;
        let gpu_w = PART_W[GPU][t(GPU)] * if busy { 100 } else { 15 } / 100;
        let mine_w = if self.mining { PART_W[GPU][t(GPU)] * 60 / 100 + 50 } else { 0 };
        base + cpu_w + gpu_w + mine_w
    }
    fn ai_on(&self, tick: usize) -> bool {
        tick < self.ai.until
    }
    /// CPU clock (GHz x100): the CPU's base, +0.9 overclocked, throttled above 85 C.
    fn ghz(&self) -> usize {
        let base = GHZ[self.part(CPU)] + if self.oc { 90 } else { 0 };
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
        if self.has_fn(F_NEURAL_NET) { typo = typo * (100 - NB[F_NEURAL_NET - 24].3 as usize / 4) / 100; }   // round 108: -20%
        Knobs { cps100: cps, typo, notice: self.t(&NOTICE, NOTICE_TOP), rank: self.rank(), syntax: None, lang_mult: true,
                read100: cps, compile_pct: COMPILE_PCT[self.part(SSD)] }
    }
    /// Who's writing this function: him, or the AI while it's on.
    fn knobs_for(&self, t: &Typing, tick: usize) -> Knobs {
        let mine = self.knobs();
        match t.ai {
            Some((p, lite)) if self.ai_on(tick) => {
                // round 108: with tensor_core the lite models write like the flagships
                let lite = lite && !self.has_fn(F_TENSOR_CORE);
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
    /// Round 108: spend one of the effect budget's tokens (refilled one every FX_REFILL ticks, up to FX_BUDGET).
    fn fx_ok(&mut self, tick: usize) -> bool {
        if tick >= self.fx_refill_at {
            let back = if self.fx_refill_at == 0 { FX_BUDGET } else { 1 + (tick - self.fx_refill_at) / FX_REFILL };
            self.fx_tokens = (self.fx_tokens + back).min(FX_BUDGET);
            self.fx_refill_at = tick + FX_REFILL;
        }
        if self.fx_tokens == 0 { self.stats.fx_skipped += 1; return false; }
        self.fx_tokens -= 1;
        true
    }
    /// A function's effect on a unit, within the budget.
    fn vfx(&mut self, sim: &mut StableSim<'_>, me: usize, tag: &str, target: usize, life: u64) {
        if self.fx_ok(sim.tick()) { Self::fx(sim, me, tag, target, life); }
    }
    /// A function's effect at a point, within the budget.
    fn vfx_at(&mut self, sim: &mut StableSim<'_>, me: usize, tag: &str, p: (i64, i64), life: u64) {
        if self.fx_ok(sim.tick()) { Self::fx_at(sim, me, tag, p, life); }
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
        let Some(e) = sim.get_entity(me) else { return };
        let (x, y) = e.pos();
        let (x, y) = (x as i64, y as i64);
        // round 106: in his rank's theme, on the window's status bar (the blue screen sits on him)
        if tag == "ov_bsod" {
            Self::fx_at(sim, me, &tag, (x, y), TERM_EVERY as u64 + 1);
        } else {
            let name = format!("{tag}_t{}", theme_of(self.rank(), self.root));
            Self::fx_at(sim, me, &name, (x, y - STATUS_DY), TERM_EVERY as u64 + 1);
        }
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
            // round 108: the new ones by class: heals and shields like the defensive functions, areas by the crowd,
            // ally buffs by the allies near, passive daemons a little more the fuller his program
            _ => match class(f) {
                2 | 3 => 12 * hurt.min(2) + 6 * pressed.min(2) * self.t(&IQ, IQ_TOP) / 100,
                1 => 8 * foes.min(3),
                5 => 6 * mates.min(3),
                6 if passive(f) => 2 * self.program.len(),
                _ => 0,
            },
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
        let slots_now = self.slots_now(all, m);
        // round 108: no-spam, per the plan: x0.55 for each time he wrote it in the last 45 s (a Script Kiddie gets
        // stuck on what he knows: x0.90), +15% for what he hasn't written lately
        let repeat = 90 - 5 * self.rank().min(ROOT);
        let fighting = all.iter().any(|c| c.team != m.team && d2(c.x, c.y, m.x, m.y) <= sq(70_000));
        let mut out = Vec::new();
        for f in 0..NF {
            if self.program.iter().any(|c| c.f == f) { continue; }
            if self.typing.as_ref().is_some_and(|t| t.f == f) { continue; }
            // round 108: a script still on cooldown (or already waiting to fire) isn't worth writing yet
            if is_script(f) && (self.now < self.cooldown[f] || self.pending_script.as_ref().is_some_and(|p| p.0 == f)) { continue; }
            let lang = if self.rng.chance(iq, 100) { ideal(f, self.rank()) } else { any_lang(f, &mut self.rng) };
            let v = self.value(f, all, m);
            if !is_script(f) && self.program.len() >= slots_now && v * 10 < weakest * 13 { continue; }
            // round 103: judgement also asks whether it could run now (the lab showed the top ranks writing sort() with
            // two enemies about, while the low ranks' ping() ran all fight)
            // (a defensive function is insurance: it counts as ready while an ally has an enemy on them)
            let insurance = matches!(f, SHIELD | HEAL | ENCRYPT | CLEANSE | ROLLBACK | SWAP) && all.iter().any(|c| c.team == m.team
                && all.iter().any(|e| e.team != m.team && d2(e.x, e.y, c.x, c.y) <= sq(30_000)));
            let ready = insurance || (passive(f) && fighting) || self.trigger(f, all, m, self.now, false, false).is_some();
            let v = if ready { v } else { v * (100 - iq / 2) / 100 };
            let (secs, clean) = self.believed(f, lang, aware);
            // round 107: no-spam. Recently written -> lower score; not written this fight -> a nudge up.
            let recent = self.last_write.iter().filter(|&&(g, _)| g == f).count();
            let var = if recent == 0 { 115 } else { (0..recent.min(4)).fold(100, |v, _| v * repeat / 100) };
            out.push((v * clean * var / 100 * 100 / (100 + secs * 8), f, lang));
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
            // round 108: a saved script is a few lines: it loads at once
            let ticks = if is_script(f) { 0 } else { RELOAD[self.part(SSD)] };
            self.typing = Some(Typing::reload(f, s.lang, &s.bugs.clone(), tick, ticks));
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
            let k = if self.has_fn(F_TENSOR_CORE) { 2 } else { 1 };   // round 108: tensor_core refills twice as fast
            for p in 0..3 { self.ai.pools[p] = (self.ai.pools[p] + REFILL[p] * k).min(POOL[p]); }
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
        // round 108: ci_cd catches half the logic bugs of every compile
        let bugs: Vec<Bug> = if self.has_fn(F_CI_CD) { bugs.into_iter().filter(|_| self.rng.chance(50, 100)).collect() } else { bugs };
        self.stats.shipped += 1;
        if bugs.is_empty() { self.stats.clean += 1; }
        self.stats.bugs += bugs.len();
        // round 107: remember what he wrote, so he doesn't keep writing the same thing (variety)
        let now = sim.tick();
        self.last_write.push((f, now));
        self.last_write.retain(|&(_, t)| t + 2700 > now);
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
        // round 107: a script runs at once (and takes no program slot); a daemon joins the program.
        if is_script(f) {
            let tick = sim.tick();
            if !self.run_one(sim, all, m, &c, tick) {
                self.pending_script = Some((f, lang, c.bugs.clone(), tick + 120));
            }
            return;
        }
        if self.program.len() >= self.slots_now(all, m) {
            if let Some(i) = (0..self.program.len()).min_by_key(|&i| self.value(self.program[i].f, all, m)) { self.program.remove(i); }
        }
        self.program.push(c);
        let vals: Vec<usize> = self.program.iter().map(|c| self.value(c.f, all, m)).collect();
        let mut idx: Vec<usize> = (0..self.program.len()).collect();
        idx.sort_by_key(|&i| std::cmp::Reverse(vals[i]));
        self.program = idx.into_iter().map(|i| self.program[i].clone()).collect();
    }

    /// Round 107: try to run a compiled function now (checking its trigger, CPU, RAM and heat as run_program does).
    /// True when it fired. Used by scripts (on compile) and the pending-script queue.
    /// Round 108: the CPU load (x100) a run costs: its base by language (Go is cheaper), halved by deploy, shrunk by
    /// a many-core CPU; the cloud functions cost his own CPU nothing.
    fn run_cost(&self, c: &Compiled, tick: usize) -> i32 {
        if cloud_cpu(c.f) { return 0; }
        let cost = lang_cpu(c.lang, SPEC[c.f].1) / if tick < self.deploy_until { 2 } else { 1 };
        cost * CPU_LOAD[self.part(CPU)] / 100
    }

    fn run_one(&mut self, sim: &mut StableSim<'_>, all: &[Champ], m: &Champ, c: &Compiled, tick: usize) -> bool {
        if tick < self.cooldown[c.f] { return false; }
        let off = c.bugs.contains(&Bug::OffByOne);
        let Some(target) = self.trigger(c.f, all, m, tick, off, c.bugs.contains(&Bug::WrongTarget)) else { return false };
        let cost = self.run_cost(c, tick);
        if self.load + cost > 10_000 { return false; }
        // round 108: with judgement he doesn't start what would trip the breaker
        let gpu = needs_gpu(c.f) && self.tiers[GPU] > 0;
        if self.watts(cost, gpu) > BREAKER_W && self.rng.chance(self.t(&IQ, IQ_TOP), 100) { return false; }
        let mb = SPEC[c.f].2 * LANG[c.lang].ram / 100;
        self.load += cost;
        let gpu_heat = if needs_gpu(c.f) { GPU_HEAT[self.part(GPU)] } else { 0 };
        if gpu { self.gpu_load = (self.gpu_load + 4000).min(10_000); }
        self.heat += LANG[c.lang].heat * if self.oc { 2 } else { 1 } + gpu_heat;
        self.cooldown[c.f] = tick + cooldown_ticks(c.f);
        self.stats.runs += 1;
        *self.last_run.get_mut(c.f).unwrap() = tick;
        if self.execute(sim, all, m, c, target, tick, mb) {
            self.earn(100);
            // round 108: what the meta functions build on
            if all.iter().any(|x| x.id == target && x.team != m.team) { self.last_hit = Some(target); }
            if !META.contains(&c.f) {
                if is_script(c.f) { self.last_script = Some(c.clone()); }
                self.last_effect = Some((c.clone(), target));
            }
            if self.has_fn(F_TRAIN_MODEL) { self.trained += 1; }
            // kubernetes: every daemon run gets a 50% replica
            if !is_script(c.f) && c.f != F_KUBERNETES && self.has_fn(F_KUBERNETES) {
                let keep = self.scale;
                self.scale = 50;
                self.execute(sim, all, m, c, target, tick, 0);
                self.scale = keep;
            }
        }
        true
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
                self.show_term(sim, me, &t);
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

    /// Round 106: the IDE window over his head, in his rank's theme: the window and up to three code lines (the one
    /// he's on and the two above it), re-placed over him every TERM_EVERY ticks. Compiling and loading say so on its
    /// status bar.
    fn show_term(&mut self, sim: &mut StableSim<'_>, me: usize, t: &Typing) {
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
        let tick = sim.tick();
        if tick < self.term_next { return; }
        self.term_next = tick + TERM_EVERY;
        let Some(e) = sim.get_entity(me) else { return };
        let (x, y) = e.pos();
        let (x, y) = (x as i64, y as i64);
        let life = TERM_EVERY as u64 + 1;
        let theme = theme_of(self.rank(), self.root);
        Self::fx_at(sim, me, &format!("tw_t{theme}_{}", LANGS[t.lang]), (x, y - WIN_DY), life);
        for (row, line, step) in term_rows(t) {
            let tag = format!("ln_{}_{}_{line}_{step}", LANGS[t.lang], FUNCS[t.f].0);
            Self::fx_at(sim, me, &tag, (x + ROW_DX, y - ROW_DY[row]), life);
        }
    }

    // ---------------------------------------------------------------- the rig, Bitcoin and the shop

    fn step_rig(&mut self, sim: &mut StableSim<'_>, me: usize, tick: usize) {
        // cooling (nothing while the immersion pump is down), the overclock's and the miner's heat, the CPU's load
        // draining (the miner keeps some), the GPU's load fading
        if self.heat > 4000 && tick >= self.pump_until { self.heat -= COOLING[self.part(COOL)]; }
        self.gpu_load = (self.gpu_load - 60).max(0);
        if self.oc { self.heat += 14; }
        if self.mining { self.heat += 4; }
        self.load = (self.load - 50 + if self.mining { 20 } else { 0 }).max(0);
        if self.heat >= 8500 { self.stats.hot_ticks += 1; }
        if tick.is_multiple_of(60) {
            // a leak grows while the leaking function is in his program
            let leaks = self.program.iter().filter(|c| c.bugs.contains(&Bug::Leak)).count();
            if self.part(RAM) < 3 { self.leak_mb += 400 * leaks; }   // round 108: ECC RAM doesn't leak
            // Bitcoin: a trickle always, more with the miner on
            self.earn(75 + if self.mining { 300 } else { 0 });
            self.step_datacenter(sim, me, tick);
        }
        // round 108: drawing more than the circuit holds trips the breaker
        if self.watts(0, false) > BREAKER_W { self.outage(sim, me); }
        self.procs.retain(|p| p.until > tick);
        if self.ram_used() > self.ram_cap() { self.oom(sim, me); }
        if self.heat >= 10_000 { self.bsod(sim, me); }
    }

    /// Round 108: the data center's running costs and risks, once a second: the bill for every tier-4 part (unpaid,
    /// the cloud rate-limits him to tier 2 until he has BILL_RESUME), and the immersion tank's pump failing.
    fn step_datacenter(&mut self, sim: &mut StableSim<'_>, me: usize, tick: usize) {
        if let Some(line) = self.pay_bill() { self.say(sim, me, line, 90, 1); }
        if self.pump_check(tick) { self.say(sim, me, "ov_pump", PUMP_TICKS, 1); }
    }
    /// One second's data-center bill. Some(status line) when the account lapses or comes back.
    fn pay_bill(&mut self) -> Option<&'static str> {
        let n4 = self.tiers.iter().filter(|&&t| t == MAX_TIER).count();
        if n4 == 0 { return None; }
        let bill = UPKEEP * n4;
        let mut line = None;
        if self.btc >= bill {
            self.btc -= bill;
            self.stats.upkeep_paid += bill;
            if self.lapsed && self.btc >= BILL_RESUME { self.lapsed = false; line = Some("ov_billok"); }
        } else {
            self.btc = 0;
            if !self.lapsed { self.lapsed = true; line = Some("ov_billing"); }
        }
        if self.lapsed { self.stats.lapsed_secs += 1; }
        line
    }
    /// Whether the immersion tank's pump fails this second.
    fn pump_check(&mut self, tick: usize) -> bool {
        if self.tiers[COOL] != MAX_TIER || tick < self.pump_until || !self.rng.chance(PUMP_FAIL, 100) { return false; }
        self.pump_until = tick + PUMP_TICKS;
        self.stats.pump_fails += 1;
        true
    }

    /// Round 108: the breaker trips: the rig goes dark for 3 s, every unsaved daemon and what he was typing are lost,
    /// and a RAM disk forgets everything on it.
    fn outage(&mut self, sim: &mut StableSim<'_>, me: usize) {
        self.stats.outages += 1;
        self.oc = false;
        self.oc_off_at = None;
        self.mining = false;
        self.typing = None;
        self.term = None;
        self.pending_script = None;
        self.program.retain(|c| c.saved);
        self.procs.clear();
        self.load = 0;
        self.gpu_load = 0;
        if self.tiers[SSD] == MAX_TIER { self.storage.clear(); }
        self.freeze(sim, me, OUTAGE_TICKS, "ov_outage");
        sim.entity_remove_buff(me, "cd_oc");
        sim.entity_remove_buff(me, "cd_mine");
    }

    fn bsod(&mut self, sim: &mut StableSim<'_>, me: usize) {
        self.stats.bsods += 1;
        if self.tiers[SSD] == MAX_TIER { self.storage.clear(); }   // round 108: the RAM disk is wiped
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
        } else if fighting && self.heat < off - 1500 && tick >= self.bsod_until + BSOD_SHY
            && !(self.watts(0, false) + PART_W[CPU][self.tiers[CPU]] / 4 > BREAKER_W && self.rng.chance(self.t(&IQ, IQ_TOP), 100)) {
            self.oc = true;
            sim.add_buff(me, &BuffV1::named("cd_oc"));
        }
    }

    /// The miner: with judgement only when nothing's around and the rig is cool; without, always.
    fn step_mining(&mut self, sim: &mut StableSim<'_>, me: usize, fighting: bool, tick: usize) {
        if !tick.is_multiple_of(60) { return; }
        let want = if self.rng.chance(self.t(&IQ, IQ_TOP), 100) {
            !fighting && self.heat < 7000 && !self.oc && (self.mining || self.watts(0, false) + PART_W[GPU][self.tiers[GPU]] * 60 / 100 + 50 < BREAKER_W)
        } else { true };
        if want != self.mining {
            self.mining = want;
            if want { sim.add_buff(me, &BuffV1::named("cd_mine")); } else { sim.entity_remove_buff(me, "cd_mine"); }
        }
    }

    /// Kills and assists pay (the player's own counters).
    fn step_bounties(&mut self, sim: &mut StableSim<'_>, player: usize) {
        let Some(p) = sim.get_player(player) else { return };
        let (k, a) = (p.kills(), p.assists());
        let (k0, a0) = self.kills;
        // round 108: blockchain mints double, and a shield for each kill
        let mint = if self.has_fn(F_BLOCKCHAIN) { 2 } else { 1 };
        if k > k0 { self.earn((k - k0) * 2500 * mint); }
        if a > a0 { self.earn((a - a0) * 1000 * mint); }
        if k > k0 && mint == 2 { if let Some(me) = self.me { sim.entity_add_shield(me, 150, 240); sim.add_buff(me, &timed("cd_shield", 240)); } }
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
            let affordable: Vec<usize> = (0..NPARTS).filter(|&p| self.tiers[p] < MAX_TIER && btc >= PRICE[p][self.tiers[p]]).collect();
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
        if self.keylog.is_some_and(|k| k.0 == c.id && tick < k.1) { return real; }   // round 108: the keylogger
        let n = self.t(&READ, READ_TOP) as i64;
        if n == 0 { return real; }
        let h = (c.id as u64 ^ (tick / 60) as u64 ^ self.rng.0.rotate_left(7)).wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 40;
        real + (h % (2 * n as u64 + 1)) as i64 - n
    }

    fn run_program(&mut self, sim: &mut StableSim<'_>, all: &[Champ], m: &Champ, tick: usize) {
        // round 107: a compiled script waiting for its trigger (index_scan daemons let him check twice as often)
        if let Some((f, lang, bugs, until)) = self.pending_script.clone() {
            if tick >= until { self.pending_script = None; }
            else if self.run_one(sim, all, m, &Compiled { f, lang, bugs, saved: false }, tick) { self.pending_script = None; }
        }
        if tick < self.next_check { return; }
        let fast = self.program.iter().any(|c| c.f == F_INDEX_SCAN);
        let clock = self.t(&CLOCK, CLOCK_TOP) * 300 / self.ghz().max(1) / if fast { 2 } else { 1 };
        self.next_check = tick + clock.max(4);
        // round 107: among the daemons that could run now, run the one that ran least recently (variety, not always
        // ping); value only breaks ties among those last run within the same 2 s. One function a check.
        let mut best: Option<(usize, usize, usize)> = None;   // (last_run, -value, program index)
        for i in 0..self.program.len() {
            let c = &self.program[i];
            if tick < self.cooldown[c.f] { continue; }
            let off = c.bugs.contains(&Bug::OffByOne);
            if self.trigger(c.f, all, m, tick, off, c.bugs.contains(&Bug::WrongTarget)).is_none() { continue; }
            let cost = self.run_cost(c, tick);
            if self.load + cost > 10_000 { continue; }
            let mb = SPEC[c.f].2 * LANG[c.lang].ram / 100;
            if mb > 0 && self.ram_used() + mb > self.ram_cap() && self.rng.chance(self.t(&IQ, IQ_TOP), 100) { continue; }
            let heat = LANG[c.lang].heat * if self.oc { 2 } else { 1 };
            if self.heat + heat >= HOT_SKIP && self.rng.chance(self.t(&IQ, IQ_TOP), 100) { continue; }
            let v = self.value(c.f, all, m);
            let key = (self.last_run[c.f] / 120, usize::MAX - v, i);
            if best.is_none_or(|b| key < b) { best = Some(key); }
        }
        if let Some((_, _, i)) = best {
            let c = self.program[i].clone();
            self.run_one(sim, all, m, &c, tick);
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
            _ => self.trigger_new(f, all, m, tick, off, wrong),   // round 107: the 76 new functions
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
        self.say(sim, me, "ov_run", 30, 1);
        let flip = c.bugs.contains(&Bug::SignFlip);
        let wrong = c.bugs.contains(&Bug::WrongTarget);
        let ap = sim.get_entity(me).map_or(40, |e| e.stat().magic_power);
        let deploy = if tick < self.deploy_until { 150 } else { 100 };
        let power = LANG[c.lang].power * self.ghz() / 300 * deploy / 100 * POWER_BUFF / 100 * self.mult(c) / 100;   // x100 (round 107: buff all)
        let amt = |base: usize, ratio: usize| (base + ap * ratio / 100) * power / 100;
        let hit = |sim: &mut StableSim<'_>, t: usize, n: usize| {
            if flip { sim.heal(me, t, n); } else { sim.deal_damage(me, t, 0, n, AttackTypeV1::Skill); }
        };
        let pos = |id: usize| all.iter().find(|x| x.id == id).map(|x| (x.x, x.y));
        let foes = |r: i64| -> Vec<&Champ> { all.iter().filter(|x| x.team != m.team && d2(x.x, x.y, m.x, m.y) <= sq(r)).collect() };
        if mb > 0 { self.procs.push(Proc { until: tick + SPEC[c.f].4, mb, f: c.f, target }); }
        self.vfx(sim, me, "fx_send", me, 18);
        match c.f {
            PING => { hit(sim, target, amt(35, 50)); self.vfx(sim, me, if flip { "fx_heal" } else { "fx_ping" }, target, 24); }
            SHIELD | HEAL | ENCRYPT => {
                // a wrong target helps the nearest enemy instead
                let t = if wrong { all.iter().filter(|x| x.team != m.team).min_by_key(|x| d2(x.x, x.y, m.x, m.y)).map_or(target, |x| x.id) } else { target };
                if c.f == SHIELD {
                    sim.entity_add_shield(t, amt(120, 50), 180);
                    self.last_shield = Some((tick, t, amt(120, 50)));
                    sim.add_buff(t, &timed("cd_shield", 180));
                    self.vfx(sim, me, "fx_shield", t, 24);
                } else if c.f == ENCRYPT {
                    let mut b = timed("cd_encrypt", 180);
                    b.damaged_reduce = if flip { 0 } else { 50 };
                    b.damaged_amplify = if flip { 30 } else { 0 };
                    sim.add_buff(t, &b);
                } else if flip {
                    if let Some(e) = sim.get_entity(t) { let (hp, _) = e.hp(); sim.entity_set_hp(t, hp.saturating_sub(amt(40, 20)).max(1)); }
                    self.vfx(sim, me, "fx_ping", t, 24);
                } else {
                    sim.heal(me, t, amt(80, 40));
                    self.vfx(sim, me, "fx_heal", t, 30);
                }
            }
            SCAN => { self.scan_until = tick + 180; self.vfx_at(sim, me, "fx_scan", (m.x, m.y), 36); }
            SPRAY => {
                for e in foes(25_000) { hit(sim, e.id, amt(25, 30)); }
                self.vfx_at(sim, me, "fx_spray", (m.x, m.y), 30);
            }
            BLINK => {
                let Some((ex, ey)) = pos(target) else { return false };
                let (dx, dy) = ((m.x - ex) as f64, (m.y - ey) as f64);
                let l = dx.hypot(dy).max(1.0);
                let s = if wrong { -30_000.0 } else { 30_000.0 };   // a wrong sign blinks him into them
                let to = walls::clip(m.x, m.y, m.x + (dx / l * s) as i64, m.y + (dy / l * s) as i64);
                self.vfx_at(sim, me, "fx_blink_out", (m.x, m.y), 24);
                sim.entity_set_pos(me, to.0.max(0) as u64, to.1.max(0) as u64);
                self.vfx_at(sim, me, "fx_blink_in", to, 24);
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
                    self.vfx(sim, me, "fx_chain", cur, 20);
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
                self.vfx(sim, me, "fx_cleanse", target, 30);
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
                self.vfx_at(sim, me, "fx_swap", p1, 30);
                self.vfx_at(sim, me, "fx_swap", p2, 30);
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
                    self.vfx(sim, me, "fx_sort", e.id, 30);
                }
                let mut b = timed("cd_marked", 180);
                b.damaged_amplify = 20;
                sim.add_buff(v[0].id, &b);
            }
            KILL9 => {
                let Some(e) = all.iter().find(|x| x.id == target) else { return false };
                let executes = e.hp * 100 < e.max_hp * 15 && !flip;
                if executes { sim.deal_damage(me, target, 0, e.hp + e.max_hp, AttackTypeV1::Skill); } else { hit(sim, target, amt(60, 40)); }
                self.vfx(sim, me, "fx_kill9", target, 30);
            }
            ROLLBACK => {
                let Some(s) = self.history.get(&target).and_then(|h| h.front()).copied() else { return false };
                let Some(e) = all.iter().find(|x| x.id == target) else { return false };
                self.vfx_at(sim, me, "fx_rollback", (e.x, e.y), 30);
                if e.team == m.team && !flip {
                    if s.3 > e.hp { sim.heal(me, target, s.3 - e.hp); }
                } else {
                    sim.entity_set_pos(target, s.1.max(0) as u64, s.2.max(0) as u64);
                    self.vfx_at(sim, me, "fx_rollback", (s.1, s.2), 30);
                }
            }
            RECURSE => {
                for k in 0..8 { self.hits.push(Hit { at: tick + 6 * k, target, dmg: amt(12, 15), flip, nearest: true, fx: if k % 4 == 0 { "fx_ping" } else { "" } }); }
            }
            INJECT => {
                sim.apply_cc(target, &CcV1::stun(if flip { 1 } else { 60 }));
                hit(sim, target, amt(40, 40));
                self.vfx(sim, me, "fx_inject", target, 40);
            }
            GC => {
                for e in foes(70_000).into_iter().filter(|x| x.hp * 100 < x.max_hp * 30) {
                    if !flip { sim.entity_clear_shield(e.id); }
                    hit(sim, e.id, amt(50, 40));
                    self.vfx(sim, me, "fx_gc", e.id, 30);
                }
            }
            DEPLOY => {
                self.deploy_until = tick + SPEC[DEPLOY].4;
                self.load = 0;
                sim.add_buff(me, &timed("cd_deploy", SPEC[DEPLOY].4));
            }
            _ => return self.execute_new(sim, all, m, c, target, tick),
        }
        true
    }

    /// Round 108: whether f is in his program.
    fn has_fn(&self, f: usize) -> bool {
        self.program.iter().any(|c| c.f == f)
    }
    /// Round 108: program slots now: his rank's, +2 with autoscale while enemies outnumber his team near him.
    fn slots_now(&self, all: &[Champ], m: &Champ) -> usize {
        let base = slots(self.rank(), self.root);
        if !self.has_fn(F_AUTOSCALE) { return base; }
        let near = |team: bool| all.iter().filter(|c| (c.team == m.team) == team && d2(c.x, c.y, m.x, m.y) <= sq(90_000)).count();
        base + if near(false) > near(true) { NB[F_AUTOSCALE - 24].3 as usize } else { 0 }
    }
    /// Round 108: the power multiplier (x100) of the run starting now: a replica / test run's scale, train_model's
    /// growth, sudo's next-run boost and a transaction's combo (the last two are spent by full runs only).
    fn mult(&mut self, c: &Compiled) -> usize {
        let sc = if self.scale == 0 { 100 } else { self.scale };
        let mut m = sc * (100 + 2 * self.trained.min(NB[F_TRAIN_MODEL - 24].3 as usize)) / 100;
        if self.scale == 0 {
            if self.boost_next > 0 { m = m * self.boost_next / 100; self.boost_next = 0; }
            if is_script(c.f) && self.combo > 0 { m = m * 130 / 100; self.combo -= 1; }
        }
        m
    }
    /// Round 108: another function's effect run by a meta function (a replica, cron, a test run): free, at `scale`%.
    fn rerun(&mut self, sim: &mut StableSim<'_>, all: &[Champ], m: &Champ, c: &Compiled, tick: usize, scale: usize, wrong: bool) -> bool {
        if META.contains(&c.f) || passive(c.f) { return false; }
        let Some(target) = self.trigger(c.f, all, m, tick, false, wrong) else { return false };
        let keep = self.scale;
        self.scale = scale;
        let did = self.execute(sim, all, m, c, target, tick, 0);
        self.scale = keep;
        did
    }
    /// Round 108: an ally's best buff (the one that adds the most) to copy, if any.
    fn best_buff(all: &[Champ], m: &Champ, r: i64) -> Option<(usize, BuffV1)> {
        let worth = |b: &BuffV1| b.attack_mult.max(0) + b.attack_speed_mult.max(0) + b.move_speed_mult.max(0)
            + b.magic_power_mult.max(0) + b.damaged_reduce as i32 + b.defence_mult.max(0);
        all.iter().filter(|c| c.team == m.team && c.id != m.id && d2(c.x, c.y, m.x, m.y) <= sq(r))
            .flat_map(|c| c.buffs.iter().map(move |b| (c.id, b)))
            .filter(|(_, b)| worth(b) > 0 && b.duration_kind == BuffDurationV1::Time.code())
            .max_by_key(|(_, b)| worth(b)).map(|(id, b)| (id, b.clone()))
    }
    /// Round 108: strip an enemy's helpful buffs (names of the ones that add anything) and shield.
    fn strip(sim: &mut StableSim<'_>, e: &Champ) {
        sim.entity_clear_shield(e.id);
        for b in &e.buffs {
            let good = b.attack_mult > 0 || b.attack_speed_mult > 0 || b.move_speed_mult > 0 || b.magic_power_mult > 0
                || b.damaged_reduce > 0 || b.defence_mult > 0 || b.cc_immune || b.undying;
            if good && b.duration_kind == BuffDurationV1::Time.code() && !b.name().is_empty() { sim.entity_remove_buff(e.id, b.name()); }
        }
    }

    /// Round 108: whether a new function's trigger holds, and on whom (see NB).
    fn trigger_new(&self, f: usize, all: &[Champ], m: &Champ, tick: usize, off: bool, wrong: bool) -> Option<usize> {
        let (trig, r, _class, a, _b) = NB[f - 24];
        let k = if off { 80 } else { 100 };
        let shift = if off { 20 } else { 0 };
        let r = r * k / 100;
        let foes: Vec<&Champ> = all.iter().filter(|c| c.team != m.team).collect();
        let near = |rr: i64| -> Option<&Champ> {
            let mut v: Vec<&Champ> = foes.iter().copied().filter(|c| d2(c.x, c.y, m.x, m.y) <= sq(rr)).collect();
            v.sort_by_key(|c| d2(c.x, c.y, m.x, m.y));
            if wrong { v.last().copied() } else { v.first().copied() }
        };
        let count = |rr: i64| foes.iter().filter(|c| d2(c.x, c.y, m.x, m.y) <= sq(rr)).count();
        let mates = || all.iter().filter(|c| c.team == m.team && c.id != m.id && d2(c.x, c.y, m.x, m.y) <= sq(60_000 * k / 100));
        let on = |c: &Champ, rr: i64| foes.iter().filter(|e| d2(e.x, e.y, c.x, c.y) <= sq(rr)).count();
        let fighting = count(70_000) >= 1;
        let ago = |id: usize, ticks: usize| self.history.get(&id).and_then(|h| h.iter().find(|s| s.0 + ticks >= tick).copied());
        match trig {
            0 => near(r).map(|c| c.id),
            1 => (count(r) >= 2).then_some(m.id),
            2 => mates().filter(|c| self.read_hp(c, tick) < 55 + shift).min_by_key(|c| self.read_hp(c, tick)).map(|c| c.id),
            3 => mates().filter(|c| self.read_hp(c, tick) >= 55 && on(c, 30_000) >= 1).min_by_key(|c| self.read_hp(c, tick)).map(|c| c.id),
            4 => (self.read_hp(m, tick) < 40 + shift && fighting).then_some(m.id),
            5 => fighting.then_some(m.id),
            6 => foes.iter().find(|c| d2(c.x, c.y, m.x, m.y) <= sq(r)
                && self.history.get(&c.id).and_then(|h| h.front()).is_some_and(|s| d2(s.1, s.2, c.x, c.y) > sq(40_000))).map(|c| c.id),
            7 => (mates().count() >= 1 && count(120_000) >= 1).then_some(m.id),
            9 => mates().filter(|c| self.read_hp(c, tick) < 40 + shift && on(c, 30_000) >= 1).min_by_key(|c| self.read_hp(c, tick)).map(|c| c.id),
            10 => mates().chain(std::iter::once(m)).find(|c| ago(c.id, 60).is_some_and(|s| s.3 > c.hp + c.max_hp * 8 / 100))
                .and_then(|c| foes.iter().filter(|e| d2(e.x, e.y, c.x, c.y) <= sq(40_000)).min_by_key(|e| d2(e.x, e.y, c.x, c.y)).map(|e| e.id)),
            11 => mates().chain(std::iter::once(m)).find(|c| on(c, 30_000) >= 2).map(|c| c.id),
            12 => (fighting && self.last_script.is_some()).then_some(m.id),
            13 => (fighting && if f == F_LLM_AGENT { true } else { self.last_effect.is_some() }).then_some(m.id),
            14 => self.last_shield.filter(|s| tick < s.0 + 180)
                .and_then(|s| mates().chain(std::iter::once(m)).filter(|c| c.id != s.1).min_by_key(|c| self.read_hp(c, tick)).map(|c| c.id)),
            15 => (self.read_hp(m, tick) < 15 + shift).then_some(m.id),
            16 => ago(m.id, 180).filter(|s| s.3 > m.hp + m.max_hp / 5).map(|_| m.id),
            17 => near(r).filter(|c| self.read_hp(c, tick) < a as i64 + shift).map(|c| c.id),
            18 => self.last_hit.filter(|&t| foes.iter().any(|e| e.id == t && d2(e.x, e.y, m.x, m.y) <= sq(r))),
            19 => (fighting && self.program.iter().any(|c| !c.bugs.is_empty())).then_some(m.id),
            20 => (fighting && self.program.iter().any(|c| !passive(c.f) && tick < self.cooldown[c.f] && !META.contains(&c.f))).then_some(m.id),
            21 => near(r).filter(|_| self.btc >= 50).map(|c| c.id),
            22 => (fighting && self.stash == 0 && self.read_hp(m, tick) > 30).then_some(m.id),
            23 => Self::best_buff(all, m, 60_000).map(|(id, _)| id),
            24 => (fighting && self.combo == 0).then_some(m.id),
            25 => {
                let buffed = |c: &&Champ| c.buffs.iter().any(|b| b.attack_mult > 0 || b.attack_speed_mult > 0 || b.damaged_reduce > 0 || b.cc_immune);
                foes.iter().copied().filter(|c| d2(c.x, c.y, m.x, m.y) <= sq(r)).find(buffed).map(|c| c.id).or_else(|| near(r).map(|c| c.id))
            }
            _ => None,   // 8: passive
        }
    }

    /// Round 108: runs a new function (each its own arm). True when it did something (it pays a little Bitcoin).
    fn execute_new(&mut self, sim: &mut StableSim<'_>, all: &[Champ], m: &Champ, c: &Compiled, target: usize, tick: usize) -> bool {
        let (_t, r, _class, a, b) = NB[c.f - 24];
        let me = m.id;
        let flip = c.bugs.contains(&Bug::SignFlip);
        let ap = sim.get_entity(me).map_or(40, |e| e.stat().magic_power) as usize;
        let mut power = LANG[c.lang].power * self.ghz() / 300 * POWER_BUFF / 100 * self.mult(c) / 100;
        if needs_gpu(c.f) { power = power * GPU_POWER[self.part(GPU)] / 100; }
        let amt = |base: usize, ratio: usize| (base + ap * ratio / 100) * power / 100;
        let (a, b) = (a.max(0) as usize, b.max(0) as usize);
        let foes: Vec<&Champ> = all.iter().filter(|x| x.team != m.team && d2(x.x, x.y, m.x, m.y) <= sq(r.max(60_000))).collect();
        let foe_ids: Vec<usize> = foes.iter().map(|x| x.id).collect();
        let hit = |sim: &mut StableSim<'_>, t: usize, n: usize| {
            if flip { sim.heal(me, t, n); } else { sim.deal_damage(me, t, 0, n, AttackTypeV1::Skill); }
        };
        let unit = |t: usize| all.iter().find(|x| x.id == t);
        let is_foe = |t: usize| unit(t).is_some_and(|x| x.team != m.team);
        let is_ally = |t: usize| unit(t).is_some_and(|x| x.team == m.team);
        let toward = |p: &Champ, q: (i64, i64), d: i64| {
            let (dx, dy) = ((q.0 - p.x) as f64, (q.1 - p.y) as f64);
            let l = dx.hypot(dy).max(1.0);
            ((p.x + (dx / l * d as f64) as i64).max(0) as u64, (p.y + (dy / l * d as f64) as i64).max(0) as u64)
        };
        let mark = |sim: &mut StableSim<'_>, t: usize, ticks: usize, amp: usize| {
            let mut bf = timed("cd_marked", ticks);
            bf.damaged_amplify = amp;
            sim.add_buff(t, &bf);
        };
        match c.f {
            // ---- cloud / devops
            F_CLOUD_DEPLOY => {
                self.btc = self.btc.saturating_sub(50);
                hit(sim, target, amt(a, b));
                sim.add_buff(me, &timed("cd_cloud", 120));
                self.vfx(sim, me, "fx_cloud", target, 24);
            }
            F_DOCKER => {
                if !is_ally(target) { return false; }
                sim.entity_banish(me, target, a, "", "");
                self.vfx(sim, me, "fx_docker", target, a as u64);
            }
            F_CRON => {
                let Some(s) = self.last_script.clone() else { return false };
                if !self.rerun(sim, all, m, &s, tick, 100, false) { return false; }
                self.vfx(sim, me, "fx_git", me, 24);
            }
            F_LOAD_BALANCER | F_HONEYPOT | F_MUTEX => {
                if !is_ally(target) { return false; }
                let mut bf = timed(if c.f == F_HONEYPOT { "cd_honeypot" } else { "cd_lb" }, b);
                if c.f == F_HONEYPOT { bf.damage_reflect = if flip { 0 } else { a }; } else if flip { bf.damaged_amplify = a; } else { bf.damaged_reduce = a; }
                sim.add_buff(target, &bf);
                self.vfx(sim, me, "fx_shield", target, 20);
            }
            F_CDN | F_API_GATEWAY => {
                for ally in all.iter().filter(|x| x.team == m.team && d2(x.x, x.y, m.x, m.y) <= sq(60_000)) {
                    let mut bf = timed(if c.f == F_CDN { "cd_cdn" } else { "cd_gate" }, b);
                    let v = if flip { -(a as i32) } else { a as i32 };
                    if c.f == F_CDN { bf.move_speed_mult = v; } else { bf.attack_mult = v; }
                    sim.add_buff(ally.id, &bf);
                }
            }
            F_SERVERLESS => {
                if foe_ids.is_empty() { return false; }
                for t in &foe_ids { hit(sim, *t, amt(a, b)); }
                self.btc = self.btc.saturating_sub(30 * foe_ids.len());
                self.vfx_at(sim, me, "fx_cloud", (m.x, m.y), 30);
            }
            F_CANARY_DEPLOY => {
                let best = self.program.iter().filter(|p| !passive(p.f) && !META.contains(&p.f)).max_by_key(|p| SPEC[p.f].3).cloned();
                let Some(p) = best else { return false };
                if !self.rerun(sim, all, m, &p, tick, a, false) { return false; }
            }
            F_CHAOS_MONKEY => {
                let pool: Vec<Compiled> = self.program.iter().filter(|p| !passive(p.f) && !META.contains(&p.f)).cloned().collect();
                if pool.is_empty() { return false; }
                let mut did = false;
                for _ in 0..3 {
                    let p = pool[self.rng.below(pool.len())].clone();
                    let wrong = self.rng.chance(50, 100);
                    did |= self.rerun(sim, all, m, &p, tick, a, wrong);
                }
                self.vfx_at(sim, me, "fx_glitch", (m.x, m.y), 24);
                if !did { return false; }
            }
            F_TERRAFORM => {
                let Some(d) = unit(target) else { return false };
                let (mx, my) = ((m.x + d.x) / 2, (m.y + d.y) / 2);
                let (dx, dy) = ((d.x - m.x) as f64, (d.y - m.y) as f64);
                let l = dx.hypot(dy).max(1.0);
                let (px, py) = ((-dy / l * 12_000.0) as i64, (dx / l * 12_000.0) as i64);
                self.walls.push(Wall { a: (mx - px, my - py), b: (mx + px, my + py), until: tick + a, next: tick, dmg: amt(15, 10), flip });
                sim.entity_knockback(me, target, 2_000, 10);
            }
            // ---- git
            F_GIT_REVERT => {
                let Some(s) = self.history.get(&me).and_then(|h| h.front()).copied() else { return false };
                if s.3 <= m.hp { return false; }
                sim.entity_set_hp(me, s.3.min(m.max_hp));
                self.vfx(sim, me, "fx_rollback", me, 30);
            }
            F_GIT_BLAME => {
                let Some(t) = foes.iter().max_by_key(|x| x.attack).map(|x| x.id) else { return false };
                mark(sim, t, b, if flip { 0 } else { a });
                self.vfx(sim, me, "fx_git", t, 30);
            }
            F_GIT_PUSH_FORCE => {
                if foe_ids.is_empty() { return false; }
                for t in foe_ids.iter().filter(|&&t| unit(t).is_some_and(|x| d2(x.x, x.y, m.x, m.y) <= sq(r))) {
                    if flip { sim.entity_pull(me, *t, a, b); } else { sim.entity_knockback(me, *t, a, b); }
                }
                self.vfx_at(sim, me, "fx_push", (m.x, m.y), 24);
            }
            F_GIT_STASH => {
                self.stash = amt(a, 60);
                sim.add_buff(me, &BuffV1::named("cd_stash"));
                self.vfx(sim, me, "fx_git", me, 24);
            }
            F_CHERRY_PICK | F_OAUTH => {
                let Some((_, mut bf)) = Self::best_buff(all, m, 60_000) else { return false };
                if c.f == F_OAUTH { bf.duration_tick = a; }
                bf.duration_tick = bf.duration_tick.clamp(30, 300);
                sim.add_buff(me, &bf);
                self.vfx(sim, me, "fx_git", me, 24);
            }
            F_REBASE => {
                for p in self.program.clone() { if !META.contains(&p.f) { self.cooldown[p.f] = tick; } }
                self.vfx(sim, me, "fx_git", me, 30);
            }
            F_MERGE_CONFLICT => {
                let two: Vec<&Champ> = foes.iter().copied().filter(|x| d2(x.x, x.y, m.x, m.y) <= sq(r)).take(2).collect();
                if two.len() < 2 { return false; }
                let mid = ((two[0].x + two[1].x) / 2, (two[0].y + two[1].y) / 2);
                for e in &two {
                    let (x, y) = toward(e, mid, d2(e.x, e.y, mid.0, mid.1).isqrt() as i64 - 3_000);
                    sim.entity_set_pos(e.id, x, y);
                    if !flip { sim.apply_cc(e.id, &CcV1::stun(a as u64)); }
                }
                self.vfx_at(sim, me, "fx_git", mid, 30);
            }
            F_HOTFIX => {
                let Some(i) = (0..self.program.len()).filter(|&i| !self.program[i].bugs.is_empty()).max_by_key(|&i| self.program[i].bugs.len()) else { return false };
                self.program[i].bugs.remove(0);
                self.stats.caught += 1;
                let p = self.program[i].clone();
                self.rerun(sim, all, m, &p, tick, 100, false);
                self.vfx(sim, me, "fx_cleanse", me, 30);
            }
            // ---- security
            F_SQL_INJECTION | F_CHMOD => {
                let Some(e) = unit(target).filter(|_| is_foe(target)) else { return false };
                if !flip { Self::strip(sim, e); }
                if c.f == F_CHMOD { self.chmod = Some((target, tick + a)); }
                sim.add_buff(target, &timed("cd_lock", if c.f == F_CHMOD { a } else { 60 }));
                self.vfx(sim, me, "fx_lock", target, 24);
            }
            F_RANSOMWARE => {
                sim.apply_cc(target, &CcV1::of_kind(CcKindV1::BlockSkill, if flip { 1 } else { a as u64 }));
                sim.add_buff(target, &timed("cd_lock", a));
                self.vfx(sim, me, "fx_lock", target, 24);
            }
            F_KEYLOGGER => {
                self.keylog = Some((target, tick + a));
                mark(sim, target, a, 0);
                self.vfx(sim, me, "fx_scan", target, 24);
            }
            F_DNS_SPOOF => {
                let tank = all.iter().filter(|x| x.team == m.team && x.id != me && d2(x.x, x.y, m.x, m.y) <= sq(90_000)).max_by_key(|x| x.hp);
                let mut cc = CcV1::of_kind(CcKindV1::Taunt, a as u64);
                cc.target = if flip { me } else { tank.map_or(me, |x| x.id) };
                sim.apply_cc(target, &cc);
                self.vfx(sim, me, "fx_glitch", target, 24);
            }
            F_BOTNET => {
                let n = a + self.rank().min(ROOT) * 2 / ROOT;   // 3 .. 5
                for k in 0..n { self.drones.push((tick + SPEC[FORK].4, tick + 12 * k)); }
            }
            F_BUFFER_OVERFLOW => {
                hit(sim, target, amt(a, b));
                self.vfx(sim, me, "fx_kill9", target, 30);
                if self.rng.chance(if matches!(c.lang, CPP | ASM) { 60 } else { 40 }, 100) { self.freeze(sim, me, 60, "ov_segv"); }
            }
            F_VPN => {
                sim.entity_set_invisible(me, a);
                self.vfx_at(sim, me, "fx_blink_out", (m.x, m.y), 24);
            }
            F_FORK_BOMB | F_UDP_FLOOD => {
                if foe_ids.is_empty() { return false; }
                let ids: Vec<usize> = if c.f == F_UDP_FLOOD { foe_ids.iter().copied().filter(|_| self.rng.chance(70, 100)).collect() } else { foe_ids.clone() };
                for k in 0..a {
                    let Some(&t) = ids.get(k % ids.len().max(1)) else { break };
                    self.hits.push(Hit { at: tick + k * 2, target: t, dmg: amt(b, 6), flip, nearest: false, fx: if k % 4 == 0 { "fx_ddos" } else { "" } });
                }
                if c.f == F_FORK_BOMB { self.load = 10_000; }
            }
            F_PHISHING => {
                let Some(e) = unit(target) else { return false };
                sim.entity_pull(me, target, 2_400, 25);
                let mut cc = CcV1::of_kind(CcKindV1::Charm, a as u64);
                (cc.dx, cc.dy) = (m.x - e.x, m.y - e.y);
                cc.speed = 900;
                if !flip { sim.apply_cc(target, &cc); }
                self.vfx(sim, me, "fx_glitch", target, 24);
            }
            F_ZERO_DAY => {
                let Some(e) = unit(target) else { return false };
                if flip { sim.heal(me, target, e.max_hp * a / 100); } else { sim.deal_damage(me, target, 0, e.max_hp * a / 100, AttackTypeV1::DotIgnoreShield); }
                self.vfx(sim, me, "fx_kill9", target, 30);
            }
            F_PORT_SCAN | F_RAY_TRACING | F_SQL_QUERY => {
                if foe_ids.is_empty() { return false; }
                self.scan_until = tick + if c.f == F_RAY_TRACING { a } else { 180 };
                let marked: Vec<usize> = match c.f {
                    F_SQL_QUERY => foes.iter().filter(|x| x.hp * 100 < x.max_hp * a).map(|x| x.id).collect(),
                    F_RAY_TRACING => foe_ids.clone(),
                    _ => foes.iter().min_by_key(|x| x.hp * 100 / x.max_hp.max(1)).map(|x| x.id).into_iter().collect(),
                };
                for t in marked { mark(sim, t, 180, if c.f == F_PORT_SCAN { 20 } else { 0 }); }
                self.vfx_at(sim, me, "fx_scan", (m.x, m.y), 36);
            }
            F_MITM | F_CORS => {
                let mut bf = timed("cd_noheal", b.max(a));
                bf.heal_reduce = if flip { 0 } else { 100 };
                sim.add_buff(target, &bf);
                if c.f == F_MITM { sim.heal(me, me, amt(a, 30)); }
                self.vfx(sim, me, "fx_lock", target, 24);
            }
            F_BRUTE_FORCE => {
                for k in 0..a { self.hits.push(Hit { at: tick + k * 3, target, dmg: amt(b + 3 * k, 10), flip, nearest: false, fx: if k % 4 == 0 { "fx_chain" } else { "" } }); }
            }
            // ---- GPU / AI
            F_CUDA_KERNEL => {
                hit(sim, target, amt(a, b));
                self.vfx(sim, me, "fx_beam", target, 24);
            }
            F_QUANTUM => {
                if self.rng.chance(50, 100) { hit(sim, target, amt(3 * a, 60)); self.vfx(sim, me, "fx_beam", target, 24); }
                else { self.vfx(sim, me, "fx_glitch", target, 24); }
            }
            F_LLM_AGENT => {
                // a mini AI writes him one script that's ready now (lite-model quality: one bug in five)
                let picks = self.candidates(all, m, true);
                let Some(&(_, f, lang)) = picks.iter().find(|p| is_script(p.1) && !META.contains(&p.1)) else { return false };
                let bugs = if self.rng.chance(20, 100) { vec![[Bug::WrongTarget, Bug::OffByOne][self.rng.below(2)]] } else { vec![] };
                self.say(sim, me, "ov_thinking", 30, 1);
                self.ship(sim, me, all, m, f, lang, bugs, false);
            }
            F_DEEPFAKE => {
                let Some(e) = unit(target) else { return false };
                let mut cc = CcV1::of_kind(CcKindV1::Fear, if flip { 1 } else { a as u64 });
                (cc.dx, cc.dy) = (e.x - m.x, e.y - m.y);
                cc.speed = 700;
                sim.apply_cc(target, &cc);
                self.vfx(sim, me, "fx_glitch", target, 30);
            }
            F_DIFFUSION => {
                if !is_ally(target) { return false; }
                for k in 0..b { self.hots.push((tick + 1 + k * 30, target, amt(a, 8))); }
                self.vfx(sim, me, "fx_heal", target, 30);
            }
            F_OVERFIT => {
                hit(sim, target, amt(a, b));
                self.vfx(sim, me, "fx_beam", target, 24);
            }
            // ---- data
            F_SHARDING | F_MAP_REDUCE => {
                let ids: Vec<usize> = if c.f == F_SHARDING { foe_ids.iter().copied().take(3).collect() } else { foe_ids.clone() };
                if ids.is_empty() { return false; }
                let each = amt(a, 60) / ids.len();
                for (i, t) in ids.iter().enumerate() { hit(sim, *t, each); if i < 3 { self.vfx(sim, me, "fx_chain", *t, 20); } }
            }
            F_REPLICATION => {
                let Some((_, _, n)) = self.last_shield else { return false };
                sim.entity_add_shield(target, n * 70 / 100, 180);
                sim.add_buff(target, &timed("cd_shield", 180));
                self.vfx(sim, me, "fx_shield", target, 24);
            }
            F_BACKUP => {
                sim.heal(me, me, m.max_hp * a / 100);
                self.cooldown[c.f] = tick + 3600;   // once a minute
                self.vfx(sim, me, "fx_rollback", me, 30);
            }
            F_MIGRATE => {
                if !is_ally(target) { return false; }
                sim.entity_set_pos(target, m.x.max(0) as u64, (m.y + 6_000).max(0) as u64);
                self.vfx(sim, me, "fx_blink_in", target, 24);
            }
            F_TRANSACTION => {
                self.combo = a;
                self.vfx(sim, me, "fx_git", me, 24);
            }
            F_DEADLOCK => {
                let two: Vec<usize> = foe_ids.iter().copied().take(2).collect();
                if two.len() < 2 { return false; }
                for t in two { sim.apply_cc(t, &CcV1::stun(if flip { 1 } else { a as u64 })); self.vfx(sim, me, "fx_lock", t, 24); }
            }
            F_VACUUM | F_QUICKSORT => {
                let ids: Vec<usize> = if c.f == F_QUICKSORT {
                    foes.iter().min_by_key(|x| x.hp * 100 / x.max_hp.max(1)).map(|x| x.id).into_iter().collect()
                } else { foe_ids.clone() };
                if ids.is_empty() { return false; }
                for t in ids { if flip { sim.entity_knockback(me, t, a, 12); } else { sim.entity_grab(me, t, a, 20); } }
                self.vfx_at(sim, me, "fx_push", (m.x, m.y), 24);
            }
            // ---- net
            F_TRACEROUTE | F_NICE | F_RATE_LIMITER => {
                let mut bf = timed(if c.f == F_RATE_LIMITER { "cd_ddos" } else { "cd_lag" }, b);
                let v = if flip { a as i32 } else { -(a as i32) };
                if c.f == F_RATE_LIMITER { bf.attack_speed_mult = v; } else { bf.move_speed_mult = v; }
                sim.add_buff(target, &bf);
                self.vfx(sim, me, "fx_inject", target, 20);
            }
            F_TCP_HANDSHAKE => {
                for k in 0..3 { self.hits.push(Hit { at: tick + k * 8, target, dmg: amt(a, b), flip, nearest: false, fx: "fx_chain" }); }
                if !flip { self.ccs.push((tick + 16, target, 45)); }
            }
            F_WEBSOCKET => {
                self.tether = Some((target, tick + b, tick));
                sim.add_buff(target, &timed("cd_tether", b));
            }
            F_WEBHOOK => {
                hit(sim, target, amt(a, b));
                self.vfx(sim, me, "fx_ping", target, 20);
            }
            F_CAPTCHA => {
                let Some(t) = foes.iter().filter(|x| d2(x.x, x.y, m.x, m.y) <= sq(r)).min_by_key(|x| x.attack).map(|x| x.id) else { return false };
                sim.apply_cc(t, &CcV1::stun(if flip { 1 } else { a as u64 }));
                self.vfx(sim, me, "fx_lock", t, 24);
            }
            // ---- OS / algorithms
            F_SUDO => {
                let Some(p) = self.program.iter().filter(|p| !passive(p.f) && !META.contains(&p.f) && tick < self.cooldown[p.f]).max_by_key(|p| SPEC[p.f].3).cloned() else { return false };
                self.cooldown[p.f] = tick;
                self.boost_next = a;
                self.vfx(sim, me, "fx_cleanse", me, 24);
            }
            F_KILL_ALL => {
                if foe_ids.is_empty() { return false; }
                for t in &foe_ids { if !flip { sim.entity_clear_shield(*t); } hit(sim, *t, amt(a, b)); }
                self.vfx_at(sim, me, "fx_spray", (m.x, m.y), 30);
            }
            F_DIJKSTRA => {
                let Some(ally) = unit(target).filter(|_| is_ally(target)) else { return false };
                let (x, y) = toward(m, (ally.x, ally.y), (d2(m.x, m.y, ally.x, ally.y).isqrt() as i64 - 8_000).max(0));
                sim.entity_set_pos(me, x, y);
                self.vfx_at(sim, me, "fx_blink_in", (x as i64, y as i64), 24);
            }
            F_BINARY_SEARCH => {
                let Some(e) = unit(target) else { return false };
                if flip { sim.heal(me, target, e.hp / 2); } else { sim.deal_damage(me, target, 0, e.hp / 2 + 1, AttackTypeV1::DotIgnoreShield); }
                self.vfx(sim, me, "fx_kill9", target, 30);
            }
            F_DYNAMIC_PROG => {
                let Some((p, _)) = self.last_effect.clone() else { return false };
                if !self.rerun(sim, all, m, &p, tick, a, false) { return false; }
                self.vfx(sim, me, "fx_git", me, 20);
            }
            F_REGEX => {
                let Some(e) = unit(target) else { return false };
                self.traps.push((e.x, e.y, tick + b));
                self.vfx_at(sim, me, "fx_trap", (e.x, e.y), b as u64);
            }
            _ => return false,   // passive functions never run
        }
        true
    }

    /// Round 108: the lasting effects of the new functions, every tick: git_stash's shield, a websocket's pings,
    /// chmod's lock, regex's traps, diffusion's healing, bloom_filter's guard, the keylogger.
    fn step_new(&mut self, sim: &mut StableSim<'_>, all: &[Champ], m: &Champ, tick: usize) {
        let me = m.id;
        if self.stash > 0 && m.hp * 100 < m.max_hp * 30 {
            sim.entity_add_shield(me, self.stash, 240);
            sim.add_buff(me, &timed("cd_shield", 240));
            sim.entity_remove_buff(me, "cd_stash");
            self.stash = 0;
            self.vfx(sim, me, "fx_shield", me, 24);
        }
        if let Some((t, until, next)) = self.tether {
            let near = all.iter().find(|x| x.id == t && x.team != m.team && d2(x.x, x.y, m.x, m.y) <= sq(90_000));
            if tick >= until || near.is_none() { self.tether = None; sim.entity_remove_buff(t, "cd_tether"); }
            else if tick >= next {
                let ap = sim.get_entity(me).map_or(40, |e| e.stat().magic_power);
                sim.deal_damage(me, t, 0, (NB[F_WEBSOCKET - 24].3 as usize + ap / 5) * self.ghz() / 300, AttackTypeV1::Skill);
                self.tether = Some((t, until, tick + 30));
            }
        }
        if let Some((t, until)) = self.chmod {
            if tick >= until { self.chmod = None; }
            else if tick.is_multiple_of(30) { if let Some(e) = all.iter().find(|x| x.id == t) { Self::strip(sim, e); } }
        }
        if !self.traps.is_empty() {
            self.traps.retain(|t| t.2 > tick);
            let mut sprung = Vec::new();
            for (i, &(x, y, _)) in self.traps.iter().enumerate() {
                if let Some(e) = all.iter().find(|e| e.team != m.team && d2(e.x, e.y, x, y) <= sq(8_000)) { sprung.push((i, e.id)); }
            }
            for &(i, e) in sprung.iter().rev() {
                sim.apply_cc(e, &CcV1::of_kind(CcKindV1::Bind, NB[F_REGEX - 24].3 as u64));
                self.traps.remove(i);
            }
        }
        if !self.ccs.is_empty() {
            let due: Vec<(usize, usize, u64)> = self.ccs.iter().filter(|h| h.0 <= tick).copied().collect();
            self.ccs.retain(|h| h.0 > tick);
            for (_, t, n) in due { sim.apply_cc(t, &CcV1::stun(n)); }
        }
        if !self.hots.is_empty() {
            let due: Vec<(usize, usize, usize)> = self.hots.iter().filter(|h| h.0 <= tick).copied().collect();
            self.hots.retain(|h| h.0 > tick);
            for (_, t, n) in due { sim.heal(me, t, n); }
        }
        if let Some((_, until)) = self.keylog { if tick >= until { self.keylog = None; } }
        if tick.is_multiple_of(60) && self.has_fn(F_BLOOM_FILTER) {
            let mut bf = timed("cd_bloom", 90);
            bf.damaged_reduce = NB[F_BLOOM_FILTER - 24].3 as usize;
            sim.add_buff(me, &bf);
        }
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
            if !h.fx.is_empty() { self.vfx(sim, me, h.fx, t, 20); }
        }
        // the drones ping the nearest enemy every half second
        self.drones.retain(|d| d.0 > tick);
        let ap = sim.get_entity(me).map_or(40, |e| e.stat().magic_power);
        for i in 0..self.drones.len() {
            if tick < self.drones[i].1 { continue; }
            self.drones[i].1 = tick + 30;
            if let Some(t) = nearest(70_000) {
                sim.deal_damage(me, t, 0, (15 + ap / 5) * self.ghz() / 300, AttackTypeV1::Skill);
                // round 108: one ping in four is drawn (lighter with a botnet up)
                self.drone_pings += 1;
                if self.drone_pings % 4 == 1 { self.vfx(sim, me, "fx_ping", t, 20); }
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
        // round 108: the power meter (draw against the breaker) and, with a GPU, its load
        let pow = (self.watts(0, false).max(0) as usize * 8 / BREAKER_W as usize).min(8);
        swap(sim, self.hud_pow, pow, "cd_pow", m.has(&format!("cd_pow{pow}")));
        self.hud_pow = Some(pow);
        if self.tiers[GPU] > 0 {
            let gpu = (self.gpu_load.max(0) as usize * 8 / 10_000).min(8);
            swap(sim, self.hud_gpu, gpu, "cd_gpu", m.has(&format!("cd_gpu{gpu}")));
            self.hud_gpu = Some(gpu);
        }
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
        // round 106: his rank's outfit and keyboard
        let fit = fit_of(self.rank(), self.root);
        if self.rank.is_some() && (self.shown_fit != Some(fit) || !m.has(&format!("cd_fit{fit}")) || !m.has(&format!("cd_kb{fit}"))) {
            for k in 0..9 {
                sim.entity_remove_buff(m.id, &format!("cd_fit{k}"));
                sim.entity_remove_buff(m.id, &format!("cd_kb{k}"));
            }
            sim.add_buff(m.id, &BuffV1::named(&format!("cd_fit{fit}")));
            sim.add_buff(m.id, &BuffV1::named(&format!("cd_kb{fit}")));
            self.shown_fit = Some(fit);
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
        self.hots.clear();
        self.ccs.clear();
        self.traps.clear();
        self.tether = None;
        self.chmod = None;
        self.pending_script = None;
        self.heat = 4000;
        self.oc = false;
        self.oc_off_at = None;
        self.mining = false;
        self.alive = false;
        self.hud = (None, None, None, None, None);
        self.hud_pow = None;
        self.hud_gpu = None;
        self.shown_btc = None;
        self.shown_drones = 0;
        self.shown_rig = None;
        self.shown_fit = None;
        self.say = None;
        self.ai.shown = None;
        if let Some(me) = self.me {
            for b in ["cd_oc", "cd_mine", "cd_drone1", "cd_drone2"] { sim.entity_remove_buff(me, b); }
            for k in 1..=5 {
                sim.entity_remove_buff(me, &format!("cd_rig{k}"));
                sim.entity_remove_buff(me, &format!("cd_rigf{k}"));
            }
            for k in 0..9 {
                sim.entity_remove_buff(me, &format!("cd_fit{k}"));
                sim.entity_remove_buff(me, &format!("cd_kb{k}"));
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
            self.cooldown = vec![0; NF];
            self.last_run = vec![0; NF];
            let _ = BOOK.memory();
        }
        if self.rank.is_none() && tick >= 60 {
            self.athlete = BOOK.athlete_of(sim.seed(), player);
            let (r, p) = BOOK.pinned(sim.seed()).rank_for(self.athlete);
            self.rank = Some(r);
            self.root = p;
            // round 107: the pros already own their rigs (high ranks start on the workstation / data-center tiers)
            self.tiers = start_tiers(r, p);
            // round 108: a data center comes with credit on the account (its parts bill every second)
            if self.tiers.contains(&MAX_TIER) { self.btc += START_CREDIT; }
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
        self.step_new(sim, &all, &m, tick);
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
        assert_eq!(LANGS.len(), 13);
        assert_eq!(&LANGS[..5], ["py", "js", "ts", "cpp", "rust"]);
        // Assembly is the longest to write of a function that has it
        assert!(chars(PING, ASM) > chars(PING, PY) && chars(CHAIN, ASM) > chars(CHAIN, CPP));
        // round 107: every function is written in at least one language, its ideal among them
        for f in 0..NF { assert!(avail(f, gen_ideal(f)), "{}", FUNCS[f].0); }
    }

    #[test]
    fn root_interpolates_to_number_one() {
        assert_eq!(tv(&CPS100, CPS100_TOP, 3, None), 1350);
        assert_eq!(tv(&CPS100, CPS100_TOP, ROOT, Some(10)), 3300);
        assert_eq!(tv(&CPS100, CPS100_TOP, ROOT, Some(1)), 4500);
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
        // the parts: a fresh rig is tier 0 everywhere
        let mut d = Coder { rank: Some(3), heat: 6000, ..Coder::default() };
        assert_eq!((d.ram_cap(), d.storage_cap(), d.ghz()), (32_000, 16, 300));
        assert_eq!(d.knobs().compile_pct, 100);
        // upgrading the parts raises the caps, the clock, and the compile speed
        d.tiers = [1, 1, 1, 1, 1, 1];
        assert_eq!((d.ram_cap(), d.storage_cap(), d.ghz()), (64_000, 32, 360));
        assert_eq!(d.knobs().compile_pct, 80);
        assert!(d.knobs().cps100 > Coder { rank: Some(3), heat: 6000, ..Coder::default() }.knobs().cps100, "a faster CPU types faster");
        d.tiers = [2, 2, 2, 2, 2, 2];
        assert_eq!((d.ram_cap(), d.storage_cap(), d.ghz()), (128_000, 64, 420));
        assert_eq!(d.knobs().compile_pct, 60);
        // round 108: the workstation and the data center
        d.tiers = [4; NPARTS];
        assert_eq!((d.ram_cap(), d.storage_cap(), d.ghz(), d.knobs().compile_pct), (2_000_000, 256, 560, 40));
        // a lapsed bill rate-limits them to tier 2
        d.lapsed = true;
        assert_eq!((d.ram_cap(), d.storage_cap(), d.ghz()), (128_000, 64, 420));
    }

    #[test]
    fn the_shop_buys_what_the_problems_call_for() {
        let stock = [0usize; 6];
        assert_eq!(needed(&stock, 3, 0, 0, 0, 0), Some(RAM));
        assert_eq!(needed(&stock, 0, 2, 0, 0, 0), Some(DISK));
        assert_eq!(needed(&stock, 0, 0, 5, 0, 0), Some(SSD));
        assert_eq!(needed(&stock, 0, 0, 0, 30, 1), Some(COOL));
        assert_eq!(needed(&stock, 0, 0, 0, 0, 0), Some(CPU), "nothing wrong: a faster CPU");
        assert_eq!(needed(&[MAX_TIER, 0, 0, 0, 0, 0], 9, 0, 0, 0, 0), Some(CPU), "maxed RAM: the next best");
        assert_eq!(needed(&[2; 6], 9, 0, 0, 0, 0), Some(RAM), "round 108: tiers 3 and 4 are buyable too");
        assert_eq!(needed(&[MAX_TIER; 6], 9, 9, 9, 9, 9), None);
        assert!(PRICE.iter().all(|p| p.windows(2).all(|w| w[1] > w[0])));
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
                    for step in 1..=STEPS { assert!(fx(&format!("ln_{lang}_{}_{line}_{step}", FUNCS[f].0)), "{} {lang} {line} {step}", FUNCS[f].0); }
                }
                assert!(!fx(&format!("ln_{lang}_{}_{}_1", FUNCS[f].0, lines(f, li).len())), "the art has more lines than the table");
            }
        }
        // round 106: the window and every status line, in every rank's theme
        assert!(fx("ov_bsod"));
        for th in 0..THEMES {
            let ov = |o: &str| fx(&format!("{o}_t{th}"));
            for lang in LANGS { assert!(fx(&format!("tw_t{th}_{lang}")), "tw_t{th}_{lang}"); }
            assert!(ov("ov_run"), "ov_run t{th}");
            for o in ["compiled", "saved", "syntax", "borrow", "rustc", "compile", "load", "debug1", "debug2", "debug3", "oom", "segv", "null", "loop",
                      "thinking", "reasoning", "ratelimit", "diff", "install"] {
                assert!(ov(&format!("ov_{o}")), "ov_{o} t{th}");
            }
            for p in PROVIDERS { assert!(ov(&format!("ov_switch_{p}")), "ov_switch_{p}"); }
            for p in PARTS { for t in 1..=MAX_TIER { assert!(ov(&format!("ov_buy_{p}{t}")), "ov_buy_{p}{t}"); } }
            for o in ["outage", "pump", "billing", "billok"] { assert!(ov(&format!("ov_{o}")), "ov_{o} t{th}"); }
        }
        for e in ["send", "ping", "heal", "shield", "scan", "spray", "blink_out", "blink_in", "cache", "chain", "ddos", "cleanse", "swap",
                  "sort", "kill9", "rollback", "inject", "gc"] { assert!(fx(&format!("fx_{e}")), "fx_{e}"); }
        for a in 0..8 { assert!(fx(&format!("fx_wall_{a}"))); }
        // round 108: the new functions' effects and marks, and the power / GPU meters
        for e in ["cloud", "docker", "push", "git", "lock", "beam", "glitch", "trap"] { assert!(fx(&format!("fx_{e}")), "fx_{e}"); }
        let buff = |n: String| names.contains(n.as_str());
        for b in ["cd_shield", "cd_lag", "cd_oc", "cd_mine", "cd_ddos", "cd_boost", "cd_drone1", "cd_drone2", "cd_marked", "cd_encrypt", "cd_deploy"] {
            assert!(buff(b.into()), "{b}");
        }
        for p in PROVIDERS { assert!(buff(format!("cd_ai_{p}")) && buff(format!("cd_ai_{p}_lite")), "{p}"); }
        for n in 0..=10 { assert!(buff(format!("cd_heat{n}"))); }
        for n in 0..=8 { assert!(buff(format!("cd_pow{n}")) && buff(format!("cd_gpu{n}"))); }
        for b in ["cd_cloud", "cd_lock", "cd_noheal", "cd_honeypot", "cd_tether", "cd_gate", "cd_lb", "cd_bloom", "cd_cdn", "cd_stash"] {
            assert!(buff(b.into()), "{b}");
        }
        for n in 0..=8 { assert!(buff(format!("cd_ram{n}")) && buff(format!("cd_disk{n}"))); }
        for d in 0..=9 { for place in ["h", "t", "o"] { assert!(buff(format!("cd_btc_{place}{d}"))); } }
        for r in 0..ROOT { assert!(buff(format!("cd_rank{r}"))); }
        for k in 1..=5 { assert!(buff(format!("cd_rig{k}")) && buff(format!("cd_rigf{k}")), "rig {k}"); }
        for k in 0..9 { assert!(buff(format!("cd_fit{k}")) && buff(format!("cd_kb{k}")), "fit / keyboard {k}"); }
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
                for l in (0..LANGS.len()).filter(|&l| avail(f, l)) {
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

    /// Round 106: the window shows the line he's on and the two above it, each at its reveal step.
    #[test]
    fn the_window_shows_three_lines() {
        let mut t = Typing::new(CHAIN, PY, 0);
        let n = lines(CHAIN, PY).len();
        assert!(n >= 3);
        t.phase = Phase::Type { line: 0, done: 0 };
        assert_eq!(term_rows(&t), vec![(0, 0, 1)]);
        t.phase = Phase::Type { line: 2, done: lines(CHAIN, PY)[2].0 * 100 };
        assert_eq!(term_rows(&t), vec![(0, 1, STEPS), (1, 2, STEPS)]);
        t.phase = Phase::Type { line: n - 1, done: 0 };
        assert_eq!(term_rows(&t).iter().map(|r| r.1).collect::<Vec<_>>(), vec![n - 2, n - 1]);
        t.phase = Phase::Compile { left: 5 };
        assert_eq!(term_rows(&t).len(), 2);
        t.phase = Phase::Think { left: 5 };
        assert!(term_rows(&t).is_empty());
        assert_eq!((0..ROOT).map(|r| theme_of(r, None)).collect::<Vec<_>>(), vec![0, 1, 1, 2, 2, 3, 4]);
        assert_eq!((theme_of(ROOT, Some(4)), theme_of(ROOT, Some(1))), (5, 6));
        assert_eq!((0..ROOT).map(|r| fit_of(r, None)).collect::<Vec<_>>(), vec![0, 1, 2, 3, 4, 5, 6]);
        assert_eq!((fit_of(ROOT, Some(10)), fit_of(ROOT, Some(1))), (7, 8));
    }

    /// Round 105: one status line at a time: a freeze's line holds the slot until it ends, others replace each other.
    #[test]
    fn one_status_line_at_a_time() {
        assert!(say_accepts(&None, 100, 1));
        let freeze = Some(("ov_bsod".to_string(), 250, 2));
        assert!(!say_accepts(&freeze, 100, 1), "a blue screen isn't covered by a run line");
        assert!(say_accepts(&freeze, 100, 2));
        assert!(say_accepts(&freeze, 250, 1), "it frees the slot when it ends");
        let run = Some(("ov_run".to_string(), 130, 1));
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

    fn unit(id: usize, team: usize, x: i64, hp: usize) -> Champ {
        Champ { id, team, x, y: 0, buffs: vec![], stunned: false, pushed: false, hp, max_hp: 1000, attack: 60 + id, name: String::new() }
    }
    fn fresh(rank: usize, root: Option<usize>) -> Coder {
        let mut c = Coder { rank: Some(rank), root, heat: 4000, ..Coder::default() };
        c.cooldown = vec![0; NF];
        c.last_run = vec![0; NF];
        c.tiers = start_tiers(rank, root);
        c
    }
    fn compiled(f: usize) -> Compiled {
        Compiled { f, lang: ideal(f, 0), bugs: vec![], saved: true }
    }

    /// Round 108: program slots by rank, +2 with autoscale while outnumbered.
    #[test]
    fn slots_grow_with_rank_and_autoscale() {
        assert_eq!((0..ROOT).map(|r| slots(r, None)).collect::<Vec<_>>(), vec![4, 5, 5, 6, 7, 8, 9]);
        assert_eq!((slots(ROOT, Some(10)), slots(ROOT, Some(1))), (11, 12));
        let mut c = fresh(3, None);
        let me = unit(0, 0, 0, 1000);
        let outnumbered = vec![me.clone(), unit(5, 1, 30_000, 1000), unit(6, 1, 40_000, 1000)];
        assert_eq!(c.slots_now(&outnumbered, &me), 6);
        c.program.push(compiled(F_AUTOSCALE));
        assert_eq!(c.slots_now(&outnumbered, &me), 8);
        assert_eq!(c.slots_now(&[me.clone(), unit(1, 0, 10_000, 1000), unit(5, 1, 30_000, 1000)], &me), 6, "not outnumbered");
    }

    /// Round 108: a script fires on compile (no slot), so with a full program he keeps writing scripts; one on
    /// cooldown isn't offered; and a passive daemon counts as ready in a fight.
    #[test]
    fn scripts_keep_him_writing() {
        let mut c = fresh(4, None);
        let me = unit(0, 0, 0, 1000);
        let all = vec![me.clone(), unit(1, 0, 20_000, 400), unit(5, 1, 30_000, 1000), unit(6, 1, 45_000, 1000)];
        c.program = (0..NF).filter(|&f| !is_script(f)).take(slots(4, None)).map(compiled).collect();
        let picks = c.candidates(&all, &me, true);
        assert!(picks.iter().any(|p| is_script(p.1)), "a full program still leaves him scripts to write");
        let s = picks.iter().find(|p| is_script(p.1)).unwrap().1;
        c.cooldown[s] = 10_000;
        assert!(!c.candidates(&all, &me, true).iter().any(|p| p.1 == s), "a script on cooldown isn't offered");
        assert!(passive(F_KUBERNETES) && c.trigger(F_KUBERNETES, &all, &me, 0, false, false).is_none(), "a passive daemon never runs");
    }

    /// Round 108 (Rian: "the AI spam the same codes"): what he wrote lately scores lower, more so at high rank.
    #[test]
    fn he_doesnt_spam_the_same_function() {
        let me = unit(0, 0, 0, 1000);
        let all = vec![me.clone(), unit(5, 1, 30_000, 1000)];
        // the same rng state for every call (his own judgement, not a perfect one)
        let score = |c: &mut Coder, f: usize| c.clone().candidates(&all, &me, false).iter().find(|p| p.1 == f).map_or(0, |p| p.0);
        for rank in [0, ROOT] {
            let mut c = fresh(rank, Some(10));
            let before = score(&mut c, PING);
            c.last_write = vec![(PING, 0)];
            let once = score(&mut c, PING);
            c.last_write = vec![(PING, 0), (PING, 1)];
            let twice = score(&mut c, PING);
            assert!(twice < once && once < before, "rank {rank}: {before} {once} {twice}");
            if rank == ROOT { assert!(once * 100 / before <= 50, "a Root rotates hard: {once} of {before}"); }
            else { assert!(once * 100 / before >= 70, "a Script Kiddie sticks to what he knows: {once} of {before}"); }
        }
    }

    /// Round 108: the rank loadouts, the breaker, the bill and the pump.
    #[test]
    fn the_data_center_has_its_risks() {
        assert_eq!(start_tiers(0, None), [0; NPARTS]);
        assert_eq!(start_tiers(3, None), [1; NPARTS]);
        assert_eq!(start_tiers(5, None)[GPU], 2, "Staff: an RTX 4090");
        assert_eq!(start_tiers(6, None), [3; NPARTS], "Architect: a workstation");
        assert_eq!(start_tiers(ROOT, Some(10)), [MAX_TIER; NPARTS], "Root: a data center");
        // a workstation never trips the breaker; a data center flat out does
        let mut w = fresh(6, None);
        w.load = 10_000; w.oc = true; w.mining = true; w.gpu_load = 10_000;
        assert!(w.watts(0, true) < BREAKER_W, "{} W", w.watts(0, true));
        let mut d = fresh(ROOT, Some(5));
        assert!(d.watts(0, false) < BREAKER_W, "idle {} W", d.watts(0, false));
        d.load = 9000; d.oc = true;
        assert!(d.watts(0, true) > BREAKER_W, "{} W", d.watts(0, true));
        // the bill: paid while he can, then the cloud rate-limits him to tier 2 until he has BILL_RESUME
        d.btc = UPKEEP * NPARTS * 2;
        assert_eq!(d.pay_bill(), None);
        assert_eq!(d.pay_bill(), None);
        assert_eq!(d.pay_bill(), Some("ov_billing"));
        d.oc = false;
        assert!(d.lapsed && d.ghz() == GHZ[2]);
        d.btc = BILL_RESUME + UPKEEP * NPARTS;
        assert_eq!(d.pay_bill(), Some("ov_billok"));
        assert!(!d.lapsed);
        assert_eq!(fresh(6, None).pay_bill(), None, "a workstation has no bill");
        // the immersion pump fails now and then (about 1% a second), and only immersion has one
        let mut fails = 0;
        for s in 0..6000 { if d.pump_check(s * 60) { fails += 1; d.pump_until = 0; } }
        assert!((20..=110).contains(&fails), "{fails} pump failures in 100 minutes");
        assert!(!(0..600).any(|s| fresh(6, None).pump_check(s * 60)));
    }

    /// Round 108: the new languages' feel.
    #[test]
    fn haskell_ships_clean_and_lua_is_quick() {
        let n = 400;
        let hs = mean(4, None, F_QUICKSORT, HS);
        let py = mean(4, None, F_QUICKSORT, PY);
        assert!(hs.1 < py.1 / 2.0, "Haskell bugs {:.2} vs Python {:.2}", hs.1, py.1);
        assert!(LANG[HS].compile > LANG[JAVA].compile && LANG[JAVA].compile > LANG[GO].compile);
        let per_char = |l: usize| { let k = knobs(4, None); mean_k(&k, F_REGEX, l, 0, n).0 / chars(F_REGEX, l) as f64 };
        assert!(per_char(LUA) < per_char(RUST), "Lua is the quicker to write");
        assert!(lang_cpu(GO, 1000) < lang_cpu(JAVA, 1000) && LANG[JAVA].ram > LANG[GO].ram);
    }

    /// Round 108: GPU functions need a GPU; the cloud ones cost his CPU nothing.
    #[test]
    fn gpu_and_cloud_functions() {
        assert!(GPU_FX.iter().all(|&f| needs_gpu(f)) && !needs_gpu(PING));
        assert!(GPU_POWER[0] == 25 && GPU_POWER.windows(2).all(|w| w[1] > w[0]));
        let c = fresh(0, None);
        assert_eq!(c.run_cost(&compiled(F_CLOUD_DEPLOY), 0), 0);
        let d = fresh(ROOT, Some(1));
        assert!(d.run_cost(&compiled(CHAIN), 0) * 2 < c.run_cost(&compiled(CHAIN), 0), "dual EPYC shrugs off the load");
        assert!(NB.len() == NF - 24 && META.iter().all(|&f| f >= 24));
    }

    /// Round 108: choosing what to write checks all 100 functions; it stays cheap.
    #[test]
    fn choosing_is_cheap() {
        let me = unit(0, 0, 0, 600);
        let all: Vec<Champ> = (0..10).map(|i| if i == 0 { me.clone() } else { unit(i, (i % 2) as usize, i as i64 * 9_000, 500) }).collect();
        let mut c = fresh(ROOT, Some(1));
        c.program = (0..NF).filter(|&f| !is_script(f)).take(10).map(compiled).collect();
        let t0 = std::time::Instant::now();
        for _ in 0..200 { std::hint::black_box(c.candidates(&all, &me, false)); }
        let per = t0.elapsed().as_micros() / 200;
        assert!(per < 2_000, "{per} us a choice (once every 30 ticks)");
    }

    #[test]
    fn wall_distance() {
        assert_eq!(seg_d2((0, 10), (-10, 0), (10, 0)), 100);
        assert_eq!(seg_d2((20, 0), (-10, 0), (10, 0)), 100);
    }
}
