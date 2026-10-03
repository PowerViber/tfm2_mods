//! Scribble (round 62): a toon mage with 35 spells.
//!
//! He weaves up to six element dots (1 Pencil, 2 Eraser, 3 Paint, 4 Gadget, 5 Page) and his ultimate, Invoke, fires
//! the spell whose recipe is exactly that dot sequence (order matters). Cooldowns are per dot count: casting a 5-dot
//! spell puts all 5-dot spells on that spell's cooldown, the other tiers stay free. The data
//! side's S1 / S2 / ult never cast: everything here is native, on his passive (no inputs are ever injected).
//!
//! Who plays him matters more than his stats: every athlete has a mastery rank from the games they have played on
//! him (Novice 0-4 games ... Archmage 150+), and the ten with the most (300+ each) are the Top 10, ranked #1-#10.
//! Every rank can try every spell (round 72); rank sets how fast they weave (2.5 dots a second for a Novice up to 15
//! for the Top 10 #1), which recipe length they build reliably and how badly they fumble past it, whether they notice
//! a wrong dot, how long they hold a ready spell for the right moment, and whether they chain a slow spell after a
//! stun. Grandmaster, Archmage and the Top 10 also wear their own skin.
//!
//! Learning is global: every cast is scored (what it really did / what it promised) per spell and situation, and the
//! scores of all games feed one shared meta that every Scribble player uses next time. Memory only changes when the
//! game loads (pending lines written during play are merged then), so the two simulations of one match (the server's
//! precomputed one and the live one) always see the same memory and never diverge.
//!
//! Files (next to the DLL, mods/tfm2_custom_ai/):
//!   scribble_memory.txt   `W <games>` / `G <athlete> <games>` / `M <spell> <bucket> <sum> <count>`
//!   scribble_pending.txt  `g <sig> <athlete>` / `c <sig> <athlete> <idx> <spell> <bucket> <ratio>` (merged at load)
//!   scribble_log.txt      one line per game: who played him, rank, casts, misfires (for reading, never read back)

use crate::{d2, sq, timed, walls, MOD_ID};
use mod_api_stable::{
    AttackTypeV1, BuffV1, CastingTargetV1, CcKindV1, CcV1, InputTargetV1, ProjectileMoveKindV1, ProjectileSpawnV1,
    SimOriginV1, StablePassive, StableSim, StatV1, UnitAttackV1,
};
use std::collections::{HashMap, HashSet, VecDeque};
use std::io::Write;
use std::sync::{Arc, Mutex, OnceLock};

const MAP: i64 = 960_000;
const ID: &str = "tfm2_toon_scribble";

// ------------------------------------------------------------------ the spell book

#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub struct Spell {
    pub name: &'static str,
    pub tag: &'static str,
    pub recipe: &'static [u8],
    pub cd: usize,
}

const fn sp(name: &'static str, tag: &'static str, recipe: &'static [u8], cd: usize) -> Spell {
    Spell { name, tag, recipe, cd }
}

pub const N: usize = 35;
pub const SPELLS: [Spell; N] = [
    // tier 1
    sp("Pencil Poke", "poke", &[1], 120),
    sp("Smudge", "smudge", &[2], 180),
    sp("Paint Splat", "splat", &[3], 180),
    sp("Squeaky Horn", "honk", &[4], 240),
    sp("Paper Cut", "cut", &[5], 120),
    // tier 2
    sp("Draw a Door", "door", &[1, 1], 360),
    sp("Cutout", "cutout", &[1, 3], 420),
    sp("Erase Myself", "erase_self", &[2, 1], 480),
    sp("Bucket", "bucket", &[3, 3], 360),
    sp("Banana Peel", "peel", &[3, 2], 360),
    sp("Rubber Chicken", "chicken", &[4, 3], 300),
    sp("Wind-up Key", "key", &[4, 4], 480),
    sp("?! Bubble", "bubble", &[5, 4], 480),
    // tier 3
    sp("Say Cheese", "phone", &[4, 1, 5], 600),
    sp("Mallet", "mallet", &[4, 4, 1], 540),
    sp("Anvil", "anvil", &[4, 2, 3], 720),
    sp("Rubber Arm", "glove", &[1, 5, 2], 600),
    sp("Erase Legs", "erase_legs", &[2, 5, 1], 660),
    sp("Pie", "pie", &[4, 3, 3], 600),
    sp("Present", "present", &[4, 1, 1], 720),
    sp("Slippery Floor", "floor", &[3, 1, 3], 840),
    // tier 4
    sp("Hole Network", "hole", &[1, 4, 2, 5], 1080),
    sp("Group Photo", "panorama", &[4, 1, 5, 5], 960),
    sp("Brawl", "brawl", &[3, 4, 3, 4], 1080),
    sp("Redraw", "redraw", &[2, 2, 5, 1], 960),
    sp("Stamp", "stamp", &[5, 3, 4, 1], 1200),
    sp("Drawn Wall", "wall_draw", &[1, 2, 1, 2], 1200),
    // tier 5
    sp("Rewind", "rewind", &[5, 5, 3, 1, 4], 2100),
    sp("Piano", "piano", &[4, 2, 4, 1, 3], 1800),
    sp("Ink Flood", "ink_wave", &[3, 5, 2, 2, 1], 1800),
    sp("Chase Scene", "chase", &[1, 3, 5, 4, 2], 1680),
    sp("Laugh Track", "laugh", &[2, 4, 1, 3, 5], 2100),
    // tier 6
    sp("Page Flip", "page", &[5, 5, 1, 2, 3, 4], 5400),
    sp("PAUSE", "pause", &[4, 4, 4, 1, 5, 2], 4800),
    sp("Draw a Friend", "sketch_in", &[3, 1, 2, 4, 5, 3], 5400),
];

pub fn recipe_of(dots: &[u8]) -> Option<usize> {
    SPELLS.iter().position(|s| s.recipe == dots)
}

pub fn tier(i: usize) -> usize {
    SPELLS[i].recipe.len()
}

// ------------------------------------------------------------------ mastery

/// Round 72 (Rian): eight ranks. 0-6 come from the athlete's mastery points; 7 is the Top 10: the ten athletes with
/// the most points on him, among those with at least TOP_POINTS. Every athlete in this game is a pro, so even a
/// Novice weaves at 2.5 dots a second; the Top 10 click like the best Invoker players (11 CPS at #10 up to 15 at #1).
pub const RANKS: usize = 8;
pub const TOP: usize = 7;
pub const RANK_NAMES: [&str; RANKS] = ["Novice", "Apprentice", "Adept", "Expert", "Master", "Grandmaster", "Archmage", "Top 10"];
pub const RANK_GAMES: [usize; 7] = [0, 5, 15, 30, 60, 100, 150];
/// Mastery points an athlete needs before they can be in the Top 10.
pub const TOP_POINTS: f64 = 300.0;
pub const TOP_SIZE: usize = 10;
/// Round 72: every rank can try every spell. This is the longest recipe an athlete builds reliably; each dot past it
/// is an overreach with its own (much higher) chance of coming out wrong.
pub const COMFORT_TIER: [usize; RANKS] = [2, 3, 3, 4, 5, 5, 6, 6];
/// Weave speed, dots (clicks) per second x100. The Top 10 go from 1100 (#10) to 1500 (#1), see `cps100`.
const CPS100: [u64; RANKS] = [250, 325, 400, 500, 650, 800, 950, 1100];
const TOP_CPS100: (u64, u64) = (1100, 1500);
/// Chance (percent) that a dot within the athlete's comfort comes out as the wrong element.
const MISFIRE: [u64; RANKS] = [16, 11, 8, 5, 3, 1, 0, 0];
/// Chance (percent) that the first dot past the comfort tier comes out wrong; each further dot adds OVERREACH_STEP
/// (a Novice going for a 6-dot spell: dot 3 80%, dot 4 86%, dot 5 92%, dot 6 98%).
const OVERREACH: [u64; RANKS] = [80, 65, 50, 40, 30, 20, 0, 0];
const OVERREACH_STEP: u64 = 6;
const OVERREACH_CAP: u64 = 98;
/// How much an athlete's spell choice accounts for the chance they will fumble the recipe (value x p^AWARE): rookies
/// still go for big spells they can't build, the best weigh it fully.
const AWARE: [f64; RANKS] = [0.35, 0.5, 0.65, 0.8, 0.9, 1.0, 1.0, 1.0];
/// Chance (percent) that a wrong dot is noticed (and the dots flicked away before casting).
const NOTICE: [u64; RANKS] = [25, 45, 65, 85, 95, 100, 100, 100];
/// Invoke cast time, ticks.
const INVOKE_T: [usize; RANKS] = [30, 27, 24, 21, 18, 15, 12, 10];
/// How wrong their read of a situation can be (multiplicative noise on a spell's value).
const NOISE: [f64; RANKS] = [0.6, 0.45, 0.35, 0.25, 0.15, 0.08, 0.03, 0.01];
/// How long a woven spell is held waiting for its moment (0 = they fire at whatever is around).
const HOLD_T: [usize; RANKS] = [0, 60, 150, 240, 300, 360, 420, 480];
/// How much better another spell must look before he drops the one he is weaving or holding (Expert+ compare real
/// values; lower ranks compare through their misreads and hold on longer).
const SWITCH_K: [f64; RANKS] = [2.5, 2.0, 1.5, 1.3, 1.15, 1.1, 1.05, 1.03];
/// Round 70 (Rian): how long until he notices the spell he is weaving toward has its tier on cooldown (another spell
/// with as many dots was just cast) and goes for the next best one instead, ticks. Expert+ see it at once.
const CD_NOTICE: [usize; RANKS] = [90, 60, 30, 0, 0, 0, 0, 0];
/// Skins (round 72): Grandmaster, Archmage and Top 10 wear their own look, drawn as two buff visuals (a back layer
/// behind him and a front layer over him), since a champion's sheet can't be swapped during a match.
pub fn skin_of(rank: usize) -> Option<usize> {
    match rank { 5 => Some(0), 6 => Some(1), TOP => Some(2), _ => None }
}
/// The badge buff over his head: scr_rank0-6, or scr_top1-10 in the Top 10.
pub fn badge_buff(rank: usize, top_pos: Option<usize>) -> String {
    match (rank, top_pos) {
        (TOP, Some(p)) => format!("scr_top{}", p.clamp(1, TOP_SIZE)),
        (TOP, None) => format!("scr_top{TOP_SIZE}"),
        (r, _) => format!("scr_rank{}", r.min(TOP - 1)),
    }
}
const TICKS_PER_S: u64 = 60;
const WEAVE_BASE: usize = 30;
const THINK_EVERY: usize = 6;
const INVOKE_GAP: usize = 20;

pub fn rank_of(games: usize) -> usize {
    RANK_GAMES.iter().rposition(|&g| games >= g).unwrap_or(0)
}

/// Dots per second x100 for a rank (and Top 10 position, 1 = best).
pub fn cps100(rank: usize, top_pos: Option<usize>) -> u64 {
    let r = rank.min(RANKS - 1);
    if r == TOP {
        let p = top_pos.unwrap_or(TOP_SIZE).clamp(1, TOP_SIZE) as u64;
        let (lo, hi) = TOP_CPS100;
        return hi - (p - 1) * (hi - lo) / (TOP_SIZE as u64 - 1);
    }
    CPS100[r]
}

/// Time between two dots, in thousandths of a tick.
pub fn weave_milli(rank: usize, top_pos: Option<usize>) -> u64 {
    TICKS_PER_S * 1000 * 100 / cps100(rank, top_pos)
}

/// Chance (percent) that the dot at this position (1-based) of a recipe comes out wrong.
pub fn slip_pct(rank: usize, pos: usize) -> u64 {
    let r = rank.min(RANKS - 1);
    let comfort = COMFORT_TIER[r];
    if pos <= comfort { return MISFIRE[r]; }
    (OVERREACH[r] + OVERREACH_STEP * (pos - comfort - 1) as u64).min(OVERREACH_CAP)
}

/// Chance that the dots from position `from` (0-based) to the end of a recipe of `len` dots all come out right.
pub fn build_chance(rank: usize, from: usize, len: usize) -> f64 {
    (from + 1..=len).map(|pos| 1.0 - slip_pct(rank, pos) as f64 / 100.0).product()
}

// ------------------------------------------------------------------ memory (global meta + athletes' games)

/// What an athlete has on him: mastery points (an official match counts 1, a scrim / exhibition 0.5, a win x1.5),
/// games and wins.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Played {
    pub points: f64,
    pub games: usize,
    pub wins: usize,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Memory {
    pub world: usize,
    pub games: HashMap<usize, Played>,
    /// (spell, bucket) -> (weighted sum of delivered / promised, weight)
    pub meta: HashMap<(usize, usize), (f64, f64)>,
}

pub const BUCKETS: usize = 9;
const META_K: f64 = 12.0;
const META_CAP: f64 = 4000.0;
/// Mastery and meta weight of a match: official (it has a match id: league, cups, ranked) or not (scrims, exhibitions).
pub const OFFICIAL_W: f64 = 1.0;
pub const SCRIM_W: f64 = 0.5;
/// A won game counts this much more for the athlete, and its casts weigh this much more in the meta.
pub const WIN_MASTERY: f64 = 1.5;
pub const WIN_META: f64 = 1.25;

/// The game a pending line belongs to: the last part of its signature (the line-up and positions hash), so older lines
/// that also carried a set index ("588.0.fa62…") and newer ones ("588.fa62…") of the same game agree.
pub fn game_key(sig: &str) -> &str {
    sig.rsplit('.').next().unwrap_or(sig)
}
/// A match with an id (league, cup, ranked) rather than a scrim or exhibition ("x." signatures).
pub fn official(sig: &str) -> bool {
    !sig.starts_with("x.")
}

/// Who won, from the last structure record of a game: (my towers, their towers, my nexus %, their nexus %, score
/// difference). Some(true) = won, Some(false) = lost, None = can't tell.
pub fn result_of(r: (i64, i64, i64, i64, i64)) -> Option<bool> {
    let (mt, et, mn, en, sd) = r;
    if mn != en { return Some(en < mn); }
    if mt != et { return Some(et < mt); }
    if sd != 0 { return Some(sd > 0); }
    None
}

impl Memory {
    pub fn parse(text: &str) -> Memory {
        let mut m = Memory::default();
        for line in text.lines() {
            let f: Vec<&str> = line.split_whitespace().collect();
            match f.as_slice() {
                ["W", n] => m.world = n.parse().unwrap_or(0),
                ["G", a, n] => {
                    if let (Ok(a), Ok(n)) = (a.parse(), n.parse::<f64>()) {
                        m.games.insert(a, Played { points: n, games: n as usize, wins: 0 });
                    }
                }
                ["G", a, p, g, w] => {
                    if let (Ok(a), Ok(p), Ok(g), Ok(w)) = (a.parse(), p.parse::<f64>(), g.parse(), w.parse()) {
                        m.games.insert(a, Played { points: p, games: g, wins: w });
                    }
                }
                ["M", s, b, sum, c] => {
                    if let (Ok(s), Ok(b), Ok(sum), Ok(c)) = (s.parse::<usize>(), b.parse::<usize>(), sum.parse::<f64>(), c.parse::<f64>()) {
                        if s < N && b < BUCKETS { m.meta.insert((s, b), (sum, c)); }
                    }
                }
                _ => {}
            }
        }
        m
    }

    pub fn render(&self) -> String {
        let mut out = format!("W {}\n", self.world);
        let mut g: Vec<_> = self.games.iter().collect();
        g.sort_by_key(|(a, _)| **a);
        for (a, p) in g {
            out += &format!("G {a} {:.2} {} {}\n", p.points, p.games, p.wins);
        }
        let mut mm: Vec<_> = self.meta.iter().collect();
        mm.sort_by_key(|(k, _)| **k);
        for ((s, b), (sum, c)) in mm {
            out += &format!("M {s} {b} {sum:.3} {c:.2}\n");
        }
        out
    }

    /// Fold the lines written during play into the memory, each game / cast once (both simulations of a match and
    /// any replay write the same lines). Official matches count fully, scrims half; a win boosts the athlete's mastery
    /// and the weight of that game's casts.
    pub fn merge(&mut self, pending: &str) {
        // results first: the last structure record of each game decides it
        let mut last: HashMap<(String, usize), (usize, (i64, i64, i64, i64, i64))> = HashMap::new();
        for line in pending.lines() {
            let f: Vec<&str> = line.split_whitespace().collect();
            if let ["r", sig, a, t, mt, et, mn, en, sd] = f.as_slice() {
                let v: Vec<i64> = [mt, et, mn, en, sd].iter().filter_map(|x| x.parse().ok()).collect();
                let (Ok(a), Ok(t)) = (a.parse::<usize>(), t.parse::<usize>()) else { continue };
                if v.len() != 5 { continue; }
                let e = last.entry((game_key(sig).to_string(), a)).or_insert((0, (0, 0, 0, 0, 0)));
                if t >= e.0 { *e = (t, (v[0], v[1], v[2], v[3], v[4])); }
            }
        }
        let won = |sig: &str, a: usize| last.get(&(game_key(sig).to_string(), a)).and_then(|r| result_of(r.1)).unwrap_or(false);
        let mut seen_g: HashSet<(String, usize)> = HashSet::new();
        let mut seen_w: HashSet<String> = HashSet::new();
        let mut seen_c: HashSet<(String, usize, usize)> = HashSet::new();
        for line in pending.lines() {
            let f: Vec<&str> = line.split_whitespace().collect();
            match f.as_slice() {
                ["g", sig, a] => {
                    let Ok(a) = a.parse::<usize>() else { continue };
                    let key = game_key(sig).to_string();
                    if seen_g.insert((key.clone(), a)) {
                        let w = won(sig, a);
                        let p = self.games.entry(a).or_default();
                        p.points += if official(sig) { OFFICIAL_W } else { SCRIM_W } * if w { WIN_MASTERY } else { 1.0 };
                        p.games += 1;
                        if w { p.wins += 1; }
                    }
                    if seen_w.insert(key) {
                        self.world += 1;
                    }
                }
                ["c", sig, a, i, s, b, r] => {
                    let (Ok(a), Ok(i), Ok(s), Ok(b), Ok(r)) = (a.parse::<usize>(), i.parse::<usize>(), s.parse::<usize>(), b.parse::<usize>(), r.parse::<f64>()) else { continue };
                    if s >= N || b >= BUCKETS || !r.is_finite() { continue; }
                    if seen_c.insert((game_key(sig).to_string(), a, i)) {
                        let w = if official(sig) { OFFICIAL_W } else { SCRIM_W } * if won(sig, a) { WIN_META } else { 1.0 };
                        let e = self.meta.entry((s, b)).or_insert((0.0, 0.0));
                        e.0 += r.clamp(0.0, 3.0) * w;
                        e.1 += w;
                        if e.1 > META_CAP {
                            // old games fade: the meta keeps moving
                            e.0 *= 0.5;
                            e.1 *= 0.5;
                        }
                    }
                }
                _ => {}
            }
        }
    }

    /// How much this spell really delivers in this situation compared to what it looks like it will (1 = as
    /// promised). Few casts = close to 1.
    pub fn factor(&self, spell: usize, bucket: usize) -> f64 {
        let (sum, c) = self.meta.get(&(spell, bucket)).copied().unwrap_or((0.0, 0.0));
        ((META_K + sum) / (META_K + c)).clamp(0.35, 1.8)
    }

    /// Mastery points (what the rank is read from).
    pub fn games_of(&self, athlete: usize) -> f64 {
        self.games.get(&athlete).map_or(0.0, |p| p.points)
    }

    /// Round 72: the Top 10, best first. Athletes with at least TOP_POINTS mastery points, ordered by points, then
    /// games, then wins (athlete id breaks a full tie, so both simulations of a match always agree).
    pub fn top_ten(&self) -> Vec<usize> {
        let mut v: Vec<(usize, &Played)> = self.games.iter().filter(|(_, p)| p.points >= TOP_POINTS).map(|(a, p)| (*a, p)).collect();
        v.sort_by(|a, b| {
            b.1.points.partial_cmp(&a.1.points).unwrap_or(std::cmp::Ordering::Equal)
                .then(b.1.games.cmp(&a.1.games))
                .then(b.1.wins.cmp(&a.1.wins))
                .then(a.0.cmp(&b.0))
        });
        v.into_iter().take(TOP_SIZE).map(|(a, _)| a).collect()
    }

    /// An athlete's rank (0-7) and, in the Top 10, their position (1 = best).
    pub fn rank_for(&self, athlete: Option<usize>) -> (usize, Option<usize>) {
        let Some(a) = athlete else { return (0, None) };
        if let Some(i) = self.top_ten().iter().position(|&x| x == a) {
            return (TOP, Some(i + 1));
        }
        (rank_of(self.games_of(a).max(0.0) as usize), None)
    }
}

fn mod_dir() -> Option<std::path::PathBuf> {
    std::env::current_exe().ok().and_then(|e| e.parent().map(|d| d.join("mods").join(MOD_ID)))
}

static MEMORY: OnceLock<Memory> = OnceLock::new();

/// The memory of this game launch: the saved memory plus everything played up to the last launch (merged now). Read
/// once. Matches played during this launch are added on top per match (see `pinned`).
pub fn memory() -> &'static Memory {
    MEMORY.get_or_init(|| {
        let Some(dir) = mod_dir() else { return Memory::default() };
        let mem_p = dir.join("scribble_memory.txt");
        let pend_p = dir.join("scribble_pending.txt");
        let mut m = Memory::parse(&std::fs::read_to_string(&mem_p).unwrap_or_default());
        if let Ok(p) = std::fs::read_to_string(&pend_p) {
            if !p.trim().is_empty() {
                m.merge(&p);
                if std::fs::write(&mem_p, m.render()).is_ok() {
                    // the merged lines are kept for reading (the editor's per-game view), never merged again
                    let _ = std::fs::OpenOptions::new().create(true).append(true).open(dir.join("scribble_history.txt"))
                        .and_then(|mut f| f.write_all(p.as_bytes()));
                    let _ = std::fs::write(&pend_p, "");
                }
            }
        }
        m
    })
}

/// Write lines to one of the files; lines for the pending file also go into this launch's own learning (so the next
/// matches of the same launch already use them).
fn append(file: &str, lines: &[String]) {
    if lines.is_empty() { return; }
    if file == "scribble_pending.txt" {
        with_session(|s| s.lines.extend(lines.iter().cloned()));
    }
    let Some(dir) = mod_dir() else { return };
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(dir.join(file)) {
        let mut s = String::new();
        for l in lines {
            s += l;
            s.push('\n');
        }
        let _ = f.write_all(s.as_bytes());
    }
}

/// Who plays which Scribble: (sim seed, player id) -> athlete id, filled in by the input AI (the only place the game
/// says which athlete a player is). Both simulations of a match share the seed and the athletes.
static ATHLETES: Mutex<Option<HashMap<(u64, usize), usize>>> = Mutex::new(None);

pub fn note_athlete(seed: u64, player: usize, athlete: usize) {
    if let Ok(mut g) = ATHLETES.lock() {
        let m = g.get_or_insert_with(HashMap::new);
        if m.len() > 4096 { m.clear(); }
        m.insert((seed, player), athlete);
    }
}

fn athlete_of(seed: u64, player: usize) -> Option<usize> {
    ATHLETES.lock().ok().and_then(|g| g.as_ref().and_then(|m| m.get(&(seed, player)).copied()))
}

/// This launch's learning: every pending line written so far, and the memory each match was given. A match takes a
/// snapshot (the launch memory + this launch's lines) the first time one of its simulations asks, keyed by the
/// match's seed, which the server's precomputed simulation and the live one share: both always see the same memory
/// even when other games finish in between.
struct Session {
    lines: Vec<String>,
    pins: HashMap<u64, Arc<Memory>>,
    pin_order: VecDeque<u64>,
    written: HashSet<String>,
    logged: HashSet<(String, usize, u8)>,
}
static SESSION: Mutex<Option<Session>> = Mutex::new(None);

fn with_session<R>(f: impl FnOnce(&mut Session) -> R) -> Option<R> {
    let mut g = SESSION.lock().ok()?;
    let s = g.get_or_insert_with(|| Session {
        lines: Vec::new(),
        pins: HashMap::new(),
        pin_order: VecDeque::new(),
        written: HashSet::new(),
        logged: HashSet::new(),
    });
    Some(f(s))
}

/// The memory this match plays with (see Session).
fn pinned(seed: u64) -> Arc<Memory> {
    let base = memory();
    with_session(|s| {
        if let Some(m) = s.pins.get(&seed) { return m.clone(); }
        let mut m = base.clone();
        if !s.lines.is_empty() { m.merge(&s.lines.join("\n")); }
        let m = Arc::new(m);
        s.pins.insert(seed, m.clone());
        s.pin_order.push_back(seed);
        while s.pin_order.len() > 256 {
            if let Some(old) = s.pin_order.pop_front() { s.pins.remove(&old); }
        }
        m
    })
    .unwrap_or_else(|| Arc::new(base.clone()))
}

/// Pending lines, each written once per launch (both simulations of a match produce the same ones).
fn emit(lines: Vec<String>) {
    let fresh: Vec<String> = with_session(|s| {
        if s.written.len() > 400_000 { s.written.clear(); }
        lines.into_iter().filter(|l| s.written.insert(l.clone())).collect()
    })
    .unwrap_or_default();
    append("scribble_pending.txt", &fresh);
}

// ------------------------------------------------------------------ world snapshot

#[derive(Clone, Debug)]
struct U {
    id: usize,
    team: usize,
    x: i64,
    y: i64,
    hp: usize,
    max_hp: usize,
    attack: usize,
    held: usize, // ticks of stun / bind / airborne left
    shield: usize,
    name: String,
    buffs: Vec<String>,
}

impl U {
    fn pct(&self) -> usize {
        if self.max_hp == 0 { 100 } else { self.hp * 100 / self.max_hp }
    }
    fn has(&self, n: &str) -> bool {
        self.buffs.iter().any(|b| b == n)
    }
}

fn snapshot(sim: &StableSim<'_>) -> Vec<U> {
    let mut out = Vec::new();
    for i in 0..sim.champion_count() {
        let id = sim.champion_id_at(i);
        let Some(e) = sim.get_entity(id) else { continue };
        if !e.is_alive() { continue; }
        let (x, y) = e.pos();
        let (hp, max_hp) = e.hp();
        let mut held = 0usize;
        for c in 0..e.cc_count() {
            if let Some(cc) = e.cc_at(c) {
                if cc.kind == CcKindV1::Stun.code() || cc.kind == CcKindV1::Bind.code() || cc.kind == CcKindV1::Airborne.code() {
                    held = held.max(cc.tick as usize);
                }
            }
        }
        let buffs = (0..e.buff_count()).filter_map(|b| e.buff_at(b)).map(|b| b.name().to_string()).collect();
        out.push(U {
            id, team: e.team(), x: x as i64, y: y as i64, hp, max_hp, attack: e.stat().attack, held, shield: e.shield(),
            name: e.name().unwrap_or_default(), buffs,
        });
    }
    out
}

fn dist(ax: i64, ay: i64, bx: i64, by: i64) -> f64 {
    ((ax - bx) as f64).hypot((ay - by) as f64)
}

fn norm(dx: f64, dy: f64) -> (f64, f64) {
    let l = dx.hypot(dy);
    if l < 1e-9 { (1.0, 0.0) } else { (dx / l, dy / l) }
}

fn clampm(v: i64) -> i64 {
    v.clamp(8_000, MAP - 8_000)
}

fn nexus_pos(team: usize) -> (i64, i64) {
    if team == 0 { (96_000, 864_000) } else { (864_000, 96_000) }
}

// ------------------------------------------------------------------ visuals

fn vn(tag: &str) -> String {
    format!("{ID}_{tag}")
}

fn fx(sim: &mut StableSim<'_>, tag: &str, caster: usize, x: i64, y: i64, t: u64) {
    sim.play_view_effect(&vn(tag), caster, &InputTargetV1::pos(x.max(0) as u64, y.max(0) as u64), 0, 0, t);
}

/// Big effects both teams must see: played with a tower as the caster (everyone sees structures).
fn fx_all(sim: &mut StableSim<'_>, tag: &str, caster: usize, x: i64, y: i64, t: u64) {
    let at = InputTargetV1::pos(x.max(0) as u64, y.max(0) as u64);
    let tower = (0..sim.tower_count()).map(|i| sim.tower_id_at(i)).find(|&id| sim.get_entity(id).map_or(false, |e| e.is_alive()));
    if let Some(tw) = tower {
        if sim.play_view_effect(&vn(tag), tw, &at, 0, 0, t) { return; }
    }
    sim.play_view_effect(&vn(tag), caster, &at, 0, 0, t);
}

fn fx_on(sim: &mut StableSim<'_>, tag: &str, caster: usize, target: usize, t: u64) {
    sim.play_view_effect(&vn(tag), caster, &InputTargetV1::target(target), 0, 0, t);
}

fn fly(sim: &mut StableSim<'_>, tag: &str, me: &U, tx: i64, ty: i64, speed: u64) {
    let spec = ProjectileSpawnV1 {
        caster_id: me.id, team: me.team, x: me.x.max(0) as u64, y: (me.y - 3_000).max(0) as u64, radius: 1_000, speed,
        move_kind: ProjectileMoveKindV1::Linear.code(), target_x: tx.max(0) as u64, target_y: ty.max(0) as u64, penetrate: true,
        casting_target: CastingTargetV1::None.code(), ..ProjectileSpawnV1::default()
    };
    sim.spawn_projectile(&vn(tag), &format!("{MOD_ID}:noop"), &spec);
}

// ------------------------------------------------------------------ plans

#[derive(Clone, Debug, Default)]
struct Plan {
    target: Option<usize>,
    x: i64,
    y: i64,
    value: f64,
    bucket: usize,
    /// For slow spells: does it need the target to stay put for this many ticks?
    delay: usize,
}

struct Ctx<'a> {
    me: &'a U,
    ap: usize,
    allies: Vec<&'a U>, // with me
    foes: Vec<&'a U>,   // the ones he can see
    tick: usize,
}

impl Ctx<'_> {
    fn dmg(&self, base: usize, ratio: usize) -> usize {
        base + ratio * self.ap / 100
    }
    fn foes_in(&self, x: i64, y: i64, r: i64) -> Vec<&U> {
        self.foes.iter().copied().filter(|e| d2(e.x, e.y, x, y) <= sq(r)).collect()
    }
    fn allies_in(&self, x: i64, y: i64, r: i64) -> Vec<&U> {
        self.allies.iter().copied().filter(|a| d2(a.x, a.y, x, y) <= sq(r)).collect()
    }
    fn me_low(&self) -> bool {
        self.me.pct() < 35
    }
    fn threatened(&self, u: &U, r: i64) -> usize {
        self.foes.iter().filter(|e| d2(e.x, e.y, u.x, u.y) <= sq(r)).count()
    }
    fn bucket(&self, x: i64, y: i64) -> usize {
        let clump = match self.foes_in(x, y, 25_000).len() { 0 | 1 => 0, 2 => 1, _ => 2 };
        let state = if self.me_low() {
            1
        } else if self.allies.iter().any(|a| a.id != self.me.id && a.pct() < 35 && d2(a.x, a.y, self.me.x, self.me.y) <= sq(60_000)) {
            2
        } else {
            0
        };
        clump * 3 + state
    }
    /// A damage value: the damage, plus a kill's worth when it would finish them.
    fn dmg_value(&self, t: &U, d: usize) -> f64 {
        d.min(t.hp) as f64 + if d >= t.hp { 300.0 } else { 0.0 }
    }
    /// Best single enemy in range for a damage amount (the lowest effective HP first).
    fn best_single(&self, r: i64, d: usize) -> Option<(&U, f64)> {
        self.foes_in(self.me.x, self.me.y, r).into_iter()
            .map(|t| (t, self.dmg_value(t, d)))
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap().then(b.0.id.cmp(&a.0.id)))
    }
    fn in_cone(&self, tx: i64, ty: i64, e: &U, r: i64, half_deg: f64) -> bool {
        if d2(e.x, e.y, self.me.x, self.me.y) > sq(r) { return false; }
        let a = norm((tx - self.me.x) as f64, (ty - self.me.y) as f64);
        let b = norm((e.x - self.me.x) as f64, (e.y - self.me.y) as f64);
        (a.0 * b.0 + a.1 * b.1) >= half_deg.to_radians().cos() || d2(e.x, e.y, self.me.x, self.me.y) <= sq(6_000)
    }
    fn plan_at(&self, t: Option<&U>, x: i64, y: i64, value: f64) -> Option<Plan> {
        (value > 0.0).then(|| Plan { target: t.map(|t| t.id), x, y, value, bucket: self.bucket(x, y), delay: 0 })
    }
    /// The enemy most worth piling a slow area spell on: the most enemies within r of them, then the lowest HP.
    fn best_cluster(&self, range: i64, r: i64) -> Option<(&U, usize)> {
        self.foes_in(self.me.x, self.me.y, range).into_iter()
            .map(|t| (t, self.foes_in(t.x, t.y, r).len()))
            .max_by_key(|(t, n)| (*n, usize::MAX - t.hp, t.id))
    }
    /// Where a step away from danger goes: toward home, away from the nearest enemies.
    fn escape_dir(&self) -> (f64, f64) {
        let (hx, hy) = nexus_pos(self.me.team);
        let mut v = norm((hx - self.me.x) as f64, (hy - self.me.y) as f64);
        for e in self.foes_in(self.me.x, self.me.y, 60_000) {
            let a = norm((self.me.x - e.x) as f64, (self.me.y - e.y) as f64);
            v = (v.0 + a.0, v.1 + a.1);
        }
        norm(v.0, v.1)
    }
}

/// What casting `s` right now would do, in value points (100 = 100 damage = 1 s of hard crowd control), and where.
/// Naive: it assumes the spell lands. None = nothing to do with it.
fn eval(s: usize, c: &Ctx, hist: &VecDeque<(usize, Vec<(usize, i64, i64, usize)>)>, dead_ally: bool) -> Option<Plan> {
    let me = c.me;
    let (mx, my) = (me.x, me.y);
    let fighting = c.threatened(me, 70_000) > 0;
    match s {
        0 => {
            let d = c.dmg(30, 50);
            let (t, v) = c.best_single(60_000, d)?;
            c.plan_at(Some(t), t.x, t.y, v)
        }
        1 => {
            let d = c.dmg(20, 30);
            let t = c.foes_in(mx, my, 60_000).into_iter().max_by_key(|t| (t.shield, usize::MAX - t.hp, t.id))?;
            c.plan_at(Some(t), t.x, t.y, c.dmg_value(t, d) + t.shield as f64)
        }
        2 => {
            let d = c.dmg(25, 40);
            let (t, _) = c.best_cluster(60_000, 15_000)?;
            let v: f64 = c.foes_in(t.x, t.y, 15_000).iter().map(|e| c.dmg_value(e, d) + 25.0).sum();
            c.plan_at(Some(t), t.x, t.y, v)
        }
        3 => {
            let d = c.dmg(15, 30);
            let hit = c.foes_in(mx, my, 30_000);
            let v: f64 = hit.iter().map(|e| c.dmg_value(e, d) + 20.0).sum::<f64>() + if c.me_low() { 60.0 * hit.len() as f64 } else { 0.0 };
            c.plan_at(None, mx, my, v)
        }
        4 => {
            let d = c.dmg(40, 60);
            let (t, v) = c.best_single(70_000, d)?;
            c.plan_at(Some(t), t.x, t.y, v * 0.9)
        }
        5 => {
            // Door: out when low and chased, or after a fleeing kill
            if c.me_low() && c.threatened(me, 45_000) > 0 {
                return c.plan_at(None, mx, my, 260.0);
            }
            let t = c.foes.iter().copied().filter(|t| t.pct() < 25 && d2(t.x, t.y, mx, my) > sq(60_000) && d2(t.x, t.y, mx, my) <= sq(90_000))
                .min_by_key(|t| (t.hp, t.id))?;
            c.plan_at(Some(t), t.x, t.y, 140.0)
        }
        6 => {
            let sh = c.dmg(80, 60) as f64;
            let t = c.allies.iter().copied().filter(|a| d2(a.x, a.y, mx, my) <= sq(50_000) && c.threatened(a, 40_000) > 0)
                .min_by_key(|a| (a.pct(), a.id))?;
            let v = sh * 0.7 * if t.pct() < 50 { 1.4 } else { 1.0 };
            c.plan_at(Some(t), t.x, t.y, v)
        }
        7 => (c.me_low() && c.threatened(me, 50_000) > 0).then(|| c.plan_at(None, mx, my, 300.0)).flatten(),
        8 => {
            let h = c.dmg(70, 50);
            let t = c.allies.iter().copied().filter(|a| d2(a.x, a.y, mx, my) <= sq(50_000) && a.max_hp > a.hp)
                .max_by_key(|a| ((a.max_hp - a.hp).min(h), a.id))?;
            // round 64: Bucket was 40% of all casts (healing chip damage between fights): out of a fight a heal is
            // worth little unless someone is really hurt
            let in_fight = c.threatened(t, 70_000) > 0;
            let k = if t.pct() < 30 { 1.5 } else if in_fight { 1.0 } else if t.pct() < 60 { 0.4 } else { 0.0 };
            let v = (t.max_hp - t.hp).min(h) as f64 * k;
            (v >= 60.0).then(|| c.plan_at(Some(t), t.x, t.y, v)).flatten()
        }
        9 => {
            // a peel where they are coming from: on the nearest chaser, or in a choke
            let t = c.foes_in(mx, my, 50_000).into_iter().min_by_key(|t| (d2(t.x, t.y, mx, my), t.id))?;
            let v = 70.0 + if c.me_low() { 120.0 } else { 0.0 };
            // round 64: dropped half-way between them it was stepped on 5% of the time; now right at their feet
            let d = norm((mx - t.x) as f64, (my - t.y) as f64);
            let (x, y) = (t.x + (d.0 * 4_000.0) as i64, t.y + (d.1 * 4_000.0) as i64);
            c.plan_at(Some(t), x, y, v).map(|mut p| { p.delay = 1; p })
        }
        10 => {
            let d = c.dmg(40, 50);
            let t = c.foes_in(mx, my, 40_000).into_iter().min_by_key(|t| (d2(t.x, t.y, mx, my), t.id))?;
            let v = c.dmg_value(t, d) + 40.0 + if c.me_low() { 150.0 } else { 0.0 };
            c.plan_at(Some(t), t.x, t.y, v)
        }
        11 => {
            if !fighting { return None; }
            c.plan_at(None, mx, my, 110.0 + if c.me_low() { 80.0 } else { 0.0 })
        }
        12 => {
            let t = c.foes_in(mx, my, 60_000).into_iter().max_by_key(|t| (t.attack, t.id))?;
            c.plan_at(Some(t), t.x, t.y, 80.0 + t.attack as f64 * 0.8)
        }
        13 => {
            let t = c.foes_in(mx, my, 45_000).into_iter().max_by_key(|t| {
                (c.foes.iter().filter(|e| c.in_cone(t.x, t.y, e, 45_000, 35.0)).count(), usize::MAX - t.hp, t.id)
            })?;
            let n = c.foes.iter().filter(|e| c.in_cone(t.x, t.y, e, 45_000, 35.0)).count();
            c.plan_at(Some(t), t.x, t.y, 120.0 * n as f64)
        }
        14 => {
            let d = c.dmg(60, 80);
            let (t, v) = c.best_single(28_000, d)?;
            c.plan_at(Some(t), t.x, t.y, v + 100.0)
        }
        15 => {
            let d = c.dmg(90, 100);
            let (t, _) = c.best_cluster(60_000, 12_000)?;
            let v: f64 = c.foes_in(t.x, t.y, 12_000).iter().map(|e| c.dmg_value(e, d) + 100.0).sum();
            c.plan_at(Some(t), t.x, t.y, v).map(|mut p| { p.delay = 48; p })
        }
        16 => {
            if c.me_low() { return None; }
            let t = c.foes.iter().copied().filter(|t| d2(t.x, t.y, mx, my) <= sq(90_000) && d2(t.x, t.y, mx, my) > sq(30_000))
                .min_by_key(|t| (t.pct(), t.id))?;
            let backed = c.allies_in(mx, my, 45_000).len();
            let v = 60.0 + 60.0 * backed as f64 + if t.pct() < 35 { 200.0 } else { 0.0 } - 80.0 * c.threatened(me, 30_000) as f64;
            c.plan_at(Some(t), t.x, t.y, v)
        }
        17 => {
            let t = c.foes_in(mx, my, 60_000).into_iter().max_by_key(|t| (t.attack + if t.pct() < 40 { 400 } else { 0 }, t.id))?;
            c.plan_at(Some(t), t.x, t.y, 150.0 + if t.pct() < 40 { 80.0 } else { 0.0 })
        }
        18 => {
            let t = c.foes_in(mx, my, 50_000).into_iter().max_by_key(|t| (t.attack, t.id))?;
            c.plan_at(Some(t), t.x, t.y, 150.0 + t.attack as f64 * 0.4)
        }
        19 => {
            let d = c.dmg(120, 110);
            let (t, _) = c.best_cluster(55_000, 20_000)?;
            let v: f64 = c.foes_in(t.x, t.y, 20_000).iter().map(|e| c.dmg_value(e, d)).sum::<f64>() * 0.85;
            c.plan_at(Some(t), t.x, t.y, v)
        }
        20 => {
            if !fighting { return None; }
            let (cx, cy) = (c.foes.iter().map(|e| e.x).sum::<i64>(), c.foes.iter().map(|e| e.y).sum::<i64>());
            let n = c.foes.len().max(1) as i64;
            let (fx_, fy_) = ((cx / n + mx) / 2, (cy / n + my) / 2);
            let e_in = c.foes_in(fx_, fy_, 40_000).len();
            let a_in = c.allies_in(fx_, fy_, 40_000).len();
            c.plan_at(None, fx_, fy_, 60.0 * e_in as f64 + 30.0 * a_in as f64)
        }
        21 => {
            if c.me_low() && c.threatened(me, 50_000) > 0 {
                let d = c.escape_dir();
                let (x, y) = walls::pull_back(mx, my, clampm(mx + (d.0 * 120_000.0) as i64), clampm(my + (d.1 * 120_000.0) as i64));
                let carried = c.allies_in(mx, my, 40_000).len() - 1;
                return c.plan_at(None, x, y, 300.0 + 60.0 * carried as f64);
            }
            if c.threatened(me, 80_000) > 0 { return None; }
            // a fight far away: join it (with whoever stands by)
            let a = c.allies.iter().copied()
                .filter(|a| a.id != me.id && c.threatened(a, 40_000) > 0 && d2(a.x, a.y, mx, my) > sq(90_000) && d2(a.x, a.y, mx, my) <= sq(200_000))
                .min_by_key(|a| (a.pct(), a.id))?;
            let d = norm((a.x - mx) as f64, (a.y - my) as f64);
            let reach = dist(mx, my, a.x, a.y).min(120_000.0) - 15_000.0;
            let (x, y) = walls::pull_back(mx, my, clampm(mx + (d.0 * reach) as i64), clampm(my + (d.1 * reach) as i64));
            let carried = c.allies_in(mx, my, 40_000).len() - 1;
            c.plan_at(Some(a), x, y, 180.0 + 60.0 * carried as f64)
        }
        22 => {
            let t = c.foes_in(mx, my, 70_000).into_iter().max_by_key(|t| {
                (c.foes.iter().filter(|e| c.in_cone(t.x, t.y, e, 70_000, 60.0)).count(), t.id)
            })?;
            let n = c.foes.iter().filter(|e| c.in_cone(t.x, t.y, e, 70_000, 60.0)).count();
            (n >= 2).then(|| c.plan_at(Some(t), t.x, t.y, 150.0 * n as f64)).flatten()
        }
        23 => {
            let d = c.dmg(30, 25) * 4;
            let t = c.foes_in(mx, my, 30_000).into_iter().max_by_key(|t| (c.foes_in(t.x, t.y, 15_000).len(), usize::MAX - t.hp, t.id))?;
            let v: f64 = c.foes_in(t.x, t.y, 15_000).iter().map(|e| c.dmg_value(e, d) + 200.0).sum();
            c.plan_at(Some(t), t.x, t.y, v)
        }
        24 => {
            // back to where he was 3 s ago, if he was safer and healthier there
            let then = hist.iter().find(|h| h.0 + 180 <= c.tick && h.0 + 200 >= c.tick)?;
            let &(_, x, y, hp) = then.1.iter().find(|p| p.0 == me.id)?;
            if !(c.me_low() && c.threatened(me, 50_000) > 0) { return None; }
            let safe = c.foes.iter().filter(|e| d2(e.x, e.y, x, y) <= sq(40_000)).count() < c.threatened(me, 40_000);
            (safe || hp > me.hp).then(|| c.plan_at(None, x, y, 240.0)).flatten()
        }
        25 => {
            let hit = c.foes_in(mx, my, 35_000);
            if hit.is_empty() { return None; }
            let sh = c.dmg(120, 80) as f64 * 0.7;
            let al = c.allies_in(mx, my, 35_000).len() as f64;
            c.plan_at(None, mx, my, 120.0 * hit.len() as f64 + sh * al)
        }
        26 => {
            // a wall between him and his chasers, or behind a low enemy running away
            if c.me_low() && c.threatened(me, 50_000) > 0 {
                return c.plan_at(None, mx, my, 280.0);
            }
            let t = c.foes.iter().copied().filter(|t| t.pct() < 30 && d2(t.x, t.y, mx, my) <= sq(70_000)).min_by_key(|t| (t.hp, t.id))?;
            c.plan_at(Some(t), t.x, t.y, 200.0)
        }
        27 => {
            let then = hist.iter().find(|h| h.0 + 240 <= c.tick && h.0 + 270 >= c.tick)?;
            let mut v = 0.0;
            for a in c.allies_in(mx, my, 70_000) {
                if let Some(&(_, _, _, hp)) = then.1.iter().find(|p| p.0 == a.id) {
                    if hp > a.hp { v += (hp - a.hp) as f64 + if a.pct() < 25 { 200.0 } else { 0.0 }; }
                }
            }
            (v >= 250.0).then(|| c.plan_at(None, mx, my, v)).flatten()
        }
        28 => {
            let d = c.dmg(200, 150);
            let (t, _) = c.best_cluster(70_000, 30_000)?;
            let v: f64 = c.foes_in(t.x, t.y, 30_000).iter().map(|e| c.dmg_value(e, d) + 150.0).sum();
            c.plan_at(Some(t), t.x, t.y, v).map(|mut p| { p.delay = 60; p })
        }
        29 => {
            let d = c.dmg(60, 50);
            let t = c.foes_in(mx, my, 120_000).into_iter().max_by_key(|t| {
                (c.foes.iter().filter(|e| line_hit(mx, my, t.x, t.y, e, 160_000, 20_000)).count(), t.id)
            })?;
            let v: f64 = c.foes.iter().filter(|e| line_hit(mx, my, t.x, t.y, e, 160_000, 20_000)).map(|e| c.dmg_value(e, d) + 160.0).sum();
            c.plan_at(Some(t), t.x, t.y, v)
        }
        30 => {
            if !fighting { return None; }
            let n = c.foes_in(mx, my, 60_000).len() as f64;
            c.plan_at(None, mx, my, 100.0 + 70.0 * n + if c.me_low() { 150.0 } else { 0.0 })
        }
        31 => {
            let e = c.foes_in(mx, my, 90_000).len();
            if e == 0 { return None; }
            let h = c.dmg(60, 40);
            let heal: f64 = c.allies_in(mx, my, 90_000).iter().map(|a| (a.max_hp - a.hp).min(h) as f64).sum();
            c.plan_at(None, mx, my, 180.0 * e as f64 + heal)
        }
        32 => {
            // the page turns: top and bottom swap, mid stays. Worth it when an ally off mid is about to die to enemies
            // there (they land on the other side), or when enemies off mid are diving a lane his team is winning.
            let mut v = 0.0;
            for a in c.allies.iter() {
                if off_mid(a.x, a.y) && a.pct() < 35 && c.threatened(a, 40_000) > 0 { v += 220.0; }
            }
            let fights_mid = c.foes.iter().filter(|e| !off_mid(e.x, e.y)).count();
            if fights_mid >= 2 && c.allies.iter().filter(|a| !off_mid(a.x, a.y)).count() + 1 >= fights_mid { v += 60.0 * fights_mid as f64; }
            (v >= 220.0).then(|| c.plan_at(None, mx, my, v)).flatten()
        }
        33 => {
            // everyone on the other team stops: best in a teamfight
            let n = c.foes_in(mx, my, 70_000).len();
            (n >= 2).then(|| c.plan_at(None, mx, my, 200.0 * n as f64)).flatten()
        }
        34 => (dead_ally && fighting).then(|| c.plan_at(None, mx, my, 260.0)).flatten(),
        _ => None,
    }
}

/// What the next fight looks like, for the opener he prepares out of sight (round 68).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Intent { Default, Gank, Teamfight, Escape }

fn intent_of(c: &Ctx) -> Intent {
    let me = c.me;
    if me.pct() < 40 { return Intent::Escape; }
    // a teammate in a small fight within reach: a gank
    let gank = c.allies.iter().any(|a| {
        a.id != me.id && d2(a.x, a.y, me.x, me.y) <= sq(250_000)
            && (1..=2).contains(&c.foes_in(a.x, a.y, 50_000).len()) && c.foes_in(a.x, a.y, 100_000).len() <= 2
    });
    // three or more of them together somewhere near, or his team grouped up: a teamfight
    let clump = c.foes.iter().any(|e| d2(e.x, e.y, me.x, me.y) <= sq(250_000) && c.foes_in(e.x, e.y, 60_000).len() >= 3);
    let grouped = c.allies_in(me.x, me.y, 60_000).len() >= 4;
    if clump || (grouped && !gank) { Intent::Teamfight } else if gank { Intent::Gank } else { Intent::Default }
}

/// The openers for each kind of fight, best first (the first one he knows and has ready is prepared).
fn prep_list(i: Intent) -> &'static [usize] {
    match i {
        // single-target catches: root, stun, pull to the team, photo, pie, bubble, chicken, poke
        Intent::Gank => &[17, 14, 16, 13, 18, 12, 10, 0],
        // area: PAUSE, Piano, Group Photo, Laugh Track, Ink Flood, Stamp, Say Cheese, Paint Splat
        Intent::Teamfight => &[33, 28, 22, 31, 29, 25, 13, 2],
        // ways out: Hole Network, Redraw, Door, Erase Myself, Bucket
        Intent::Escape => &[21, 24, 5, 7, 8],
        Intent::Default => &[33, 28, 22, 13, 17, 14, 12, 0],
    }
}

fn line_hit(ax: i64, ay: i64, tx: i64, ty: i64, e: &U, len: i64, half_w: i64) -> bool {
    let (dx, dy) = norm((tx - ax) as f64, (ty - ay) as f64);
    let (ex, ey) = ((e.x - ax) as f64, (e.y - ay) as f64);
    let along = ex * dx + ey * dy;
    let side = (ex * dy - ey * dx).abs();
    along >= -4_000.0 && along <= len as f64 && side <= half_w as f64
}

/// Off the mid lane (the x + y = 960000 diagonal), where Page Flip swaps top and bottom.
pub fn off_mid(x: i64, y: i64) -> bool {
    (x + y - MAP).abs() > 110_000
}

/// Top <-> bottom: the mirror over the mid lane.
pub fn flip(x: i64, y: i64) -> (i64, i64) {
    (MAP - y, MAP - x)
}

// ------------------------------------------------------------------ delayed effects

#[derive(Clone, Debug)]
enum Later {
    Drop { at: usize, x: i64, y: i64, r: i64, dmg: usize, stun: usize, tag: &'static str, rec: usize },
    Bleed { at: usize, t: usize, dmg: usize, left: usize, rec: usize },
    Boom { at: usize, t: usize, dmg: usize, rec: usize },
    Follow { at: usize, x: i64, y: i64, who: Vec<usize> },
    Brawl { at: usize, ts: Vec<usize>, dmg: usize, left: usize, rec: usize },
    Floor { until: usize, x: i64, y: i64, next: usize },
    Peel { until: usize, x: i64, y: i64, next: usize, rec: usize },
    Chase { until: usize, hit: Vec<usize>, dmg: usize, rec: usize },
    Wall { until: usize, ax: i64, ay: i64, bx: i64, by: i64, next: usize, drawn: bool },
    Shadow { until: usize, x: i64, y: i64, tag: &'static str, next: usize },
}

#[derive(Clone, Debug)]
struct Rec {
    idx: usize,
    spell: usize,
    bucket: usize,
    est: f64,
    got: f64,
    targets: Vec<usize>,
    close: usize,
}

// ------------------------------------------------------------------ the passive

#[derive(Clone, Default)]
pub struct Scribble {
    started: bool,
    rng: u64,
    athlete: Option<usize>,
    rank: Option<usize>,
    top_pos: Option<usize>,
    dots: Vec<u8>,
    shown: Vec<u8>,
    rank_shown: Option<(usize, Option<usize>)>,
    goal: Option<usize>,
    goal_score: f64,
    next_weave_m: u64,
    flick_at: Option<usize>,
    held_since: Option<usize>,
    invoking: Option<(Option<usize>, usize, f64)>, // spell (None = no recipe), resolves at, value promised
    lock_until: usize,
    ready: Vec<usize>,
    later: Vec<Later>,
    hist: VecDeque<(usize, Vec<(usize, i64, i64, usize)>)>,
    recs: Vec<Rec>,
    done: Vec<(usize, usize, usize, f64)>, // idx, spell, bucket, ratio
    casts: usize,
    misfires: usize,
    fizzles: usize,
    cc_mark: Option<(usize, usize)>, // an enemy he just locked down, until
    sig: Option<String>,
    was_alive: bool,
    entity: usize,
    by_spell: Vec<usize>,
    mem: Option<Arc<Memory>>,
    prep: Option<Intent>,
    cd_wait: Option<usize>,
    team: Option<usize>,
    origin_kind: u32,
    last_result: Option<(i64, i64, i64, i64, i64)>,
    result_at: usize,
    summary_tick: usize,
}

impl Scribble {
    fn roll(&mut self, pct: u64) -> bool {
        self.next() % 100 < pct
    }
    fn next(&mut self) -> u64 {
        let mut x = self.rng.max(1);
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.rng = x;
        x
    }
    fn unit(&mut self) -> f64 {
        (self.next() % 10_000) as f64 / 10_000.0
    }
    fn rank(&self) -> usize {
        self.rank.unwrap_or(0)
    }

    fn show_dots(&mut self, sim: &mut StableSim<'_>, me: usize) {
        if self.shown == self.dots { return; }
        for (k, e) in self.shown.iter().enumerate() {
            sim.entity_remove_buff(me, &format!("scr_d{k}_{e}"));
        }
        for (k, e) in self.dots.iter().enumerate() {
            sim.add_buff(me, &BuffV1::named(&format!("scr_d{k}_{e}")));
        }
        self.shown = self.dots.clone();
    }

    /// The rank badge (scr_rank0-6, or scr_top1-10 with the Top 10 position) and, from Grandmaster up, the skin's
    /// two layers (scr_skin<k>_b behind him, scr_skin<k>_f over him).
    fn show_rank(&mut self, sim: &mut StableSim<'_>, me: &U) {
        let Some(r) = self.rank else { return };
        let badge = badge_buff(r, self.top_pos);
        let skin = skin_of(r);
        let present = me.has(&badge) && skin.map_or(true, |k| me.has(&format!("scr_skin{k}_b")) && me.has(&format!("scr_skin{k}_f")));
        if self.rank_shown == Some((r, self.top_pos)) && present { return; }
        for k in 0..TOP {
            sim.entity_remove_buff(me.id, &format!("scr_rank{k}"));
        }
        for p in 1..=TOP_SIZE {
            sim.entity_remove_buff(me.id, &format!("scr_top{p}"));
        }
        for k in 0..3 {
            sim.entity_remove_buff(me.id, &format!("scr_skin{k}_b"));
            sim.entity_remove_buff(me.id, &format!("scr_skin{k}_f"));
        }
        sim.add_buff(me.id, &BuffV1::named(&badge));
        if let Some(k) = skin {
            sim.add_buff(me.id, &BuffV1::named(&format!("scr_skin{k}_b")));
            sim.add_buff(me.id, &BuffV1::named(&format!("scr_skin{k}_f")));
        }
        self.rank_shown = Some((r, self.top_pos));
    }

    fn rank_label(&self) -> String {
        match (self.rank(), self.top_pos) {
            (TOP, Some(p)) => format!("Top 10 #{p}"),
            (r, _) => RANK_NAMES[r.min(RANKS - 1)].to_string(),
        }
    }

    /// Thousandths of a tick between two dots for this athlete.
    fn weave_m(&self) -> u64 {
        weave_milli(self.rank(), self.top_pos)
    }

    /// Ticks to weave `n` more dots and invoke.
    fn build_ticks(&self, n: usize) -> usize {
        (n as u64 * self.weave_m()).div_ceil(1000) as usize + INVOKE_T[self.rank()]
    }

    fn clear_dots(&mut self) {
        self.dots.clear();
        self.flick_at = None;
        self.held_since = None;
    }

    /// Rank and memory: read once per match (at 1 s, when the input AI has said who plays him), from the snapshot
    /// pinned to this match, so the server's and the live simulation agree even if a game finished in between.
    fn latch_rank(&mut self, sim: &StableSim<'_>, player: usize) {
        if self.rank.is_some() { return; }
        let athlete = athlete_of(sim.seed(), player);
        let mem = pinned(sim.seed());
        let (rank, top_pos) = mem.rank_for(athlete);
        self.athlete = athlete;
        self.rank = Some(rank);
        self.top_pos = top_pos;
        self.mem = Some(mem);
    }

    /// The game's signature (the same in both simulations and in a replay): the match id ("x" for a scrim /
    /// exhibition) plus a hash of the seed, the line-up and where everyone stands 30 s in.
    fn make_sig(&mut self, sim: &StableSim<'_>, all: &[U]) {
        let origin = sim.sim_origin().unwrap_or_default();
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        let mut eat = |v: u64| {
            for b in v.to_le_bytes() {
                h ^= b as u64;
                h = h.wrapping_mul(0x100_0000_01b3);
            }
        };
        eat(sim.seed());
        let mut v: Vec<&U> = all.iter().collect();
        v.sort_by_key(|u| u.id);
        for u in v {
            eat(u.id as u64);
            eat(u.x as u64);
            eat(u.y as u64);
            for b in u.name.bytes() { eat(b as u64); }
        }
        let m = if origin.match_id == SimOriginV1::NONE { "x".to_string() } else { format!("{}", origin.match_id) };
        let sig = format!("{m}.{h:x}");
        self.origin_kind = origin.kind;
        if let Some(a) = self.athlete {
            emit(vec![format!("g {sig} {a}")]);
        }
        self.sig = Some(sig);
    }

    fn key_athlete(&self) -> usize {
        // the meta learns even when the athlete is unknown (then only their games aren't counted)
        self.athlete.unwrap_or(1_000_000 + self.entity)
    }

    fn flush(&mut self) {
        let Some(sig) = self.sig.clone() else { return };
        if self.done.is_empty() { return; }
        let a = self.key_athlete();
        let lines = std::mem::take(&mut self.done).into_iter().map(|(i, sp_, b, r)| format!("c {sig} {a} {i} {sp_} {b} {r:.3}")).collect();
        emit(lines);
    }

    /// Where the game stands, for the win boost: towers alive and nexus health on both sides, and the score. Written
    /// when it changes (and every 10 s); the last record of a game decides who won.
    fn track_result(&mut self, sim: &StableSim<'_>, tick: usize) {
        let (Some(sig), Some(team)) = (self.sig.clone(), self.team) else { return };
        let mut towers = [0i64; 2];
        let mut nexus = [100i64; 2];
        let mut best = [i128::MAX; 2];
        for i in 0..sim.tower_count() {
            let id = sim.tower_id_at(i);
            let Some(e) = sim.get_entity(id) else { continue };
            let t = e.team().min(1);
            let (x, y) = e.pos();
            let alive = e.is_alive();
            if alive { towers[t] += 1; }
            let (nx, ny) = nexus_pos(t);
            let d = d2(x as i64, y as i64, nx, ny);
            if d <= sq(70_000) && d < best[t] {
                best[t] = d;
                let (hp, mx) = e.hp();
                nexus[t] = if !alive || mx == 0 { 0 } else { (hp * 100 / mx) as i64 };
            }
        }
        let me = team.min(1);
        let rec = (towers[me], towers[1 - me], nexus[me], nexus[1 - me], sim.score_diff(team) as i64);
        if Some(rec) != self.last_result || tick >= self.result_at + 600 {
            self.last_result = Some(rec);
            self.result_at = tick;
            let a = self.key_athlete();
            emit(vec![format!("r {sig} {a} {tick} {} {} {} {} {}", rec.0, rec.1, rec.2, rec.3, rec.4)]);
        }
    }

    /// A per-game summary for the editor's memory page (the latest one of a game counts).
    fn summary(&mut self, tick: usize) {
        let Some(sig) = self.sig.clone() else { return };
        let a = self.key_athlete();
        let top: Vec<String> = self.by_spell.iter().enumerate().filter(|x| *x.1 > 0).map(|(i, n)| format!("{i}:{n}")).collect();
        emit(vec![format!("s {sig} {a} {tick} {} {} {} {} {} {} {}", self.rank(), self.casts, self.misfires, self.fizzles, self.origin_kind,
            if official(&sig) { 1 } else { 0 }, if top.is_empty() { "-".to_string() } else { top.join(",") })]);
    }

    fn log_summary(&mut self, kind: u8) {
        let (Some(sig), a) = (self.sig.clone(), self.athlete.unwrap_or(usize::MAX)) else { return };
        let first = with_session(|s| s.logged.insert((sig.clone(), a, kind))).unwrap_or(false);
        if !first { return; }
        let r = self.rank();
        let world = self.mem.as_ref().map_or(memory().world, |m| m.world);
        let points = self.mem.as_ref().map_or(0.0, |m| m.games_of(a));
        let line = format!(
            "start game {sig} athlete {a} rank {} ({}, {:.1} mastery points, {:.1} CPS) | {} | world games {}",
            r, self.rank_label(), points, cps100(r, self.top_pos) as f64 / 100.0, if official(&sig) { "official" } else { "scrim/exhibition" }, world
        );
        append("scribble_log.txt", &[line]);
    }

    fn record(&mut self, spell: usize, plan: &Plan, targets: Vec<usize>, tick: usize) -> usize {
        self.casts += 1;
        if let Some(n) = self.by_spell.get_mut(spell) { *n += 1; }
        let idx = self.casts;
        self.recs.push(Rec { idx, spell, bucket: plan.bucket, est: plan.value.max(1.0), got: 0.0, targets, close: tick + 240 });
        idx
    }

    fn credit(&mut self, rec: usize, v: f64, t: Option<usize>) {
        if let Some(r) = self.recs.iter_mut().find(|r| r.idx == rec) {
            r.got += v;
            if let Some(t) = t {
                if !r.targets.contains(&t) { r.targets.push(t); }
            }
        }
    }

    fn hit(&mut self, sim: &mut StableSim<'_>, me: usize, t: usize, amount: usize, rec: usize) {
        if amount == 0 { return; }
        let hp = sim.get_entity(t).map_or(0, |e| e.hp().0);
        sim.deal_damage(me, t, 0, amount, AttackTypeV1::Skill);
        self.credit(rec, amount.min(hp) as f64, Some(t));
    }

    fn cc(&mut self, sim: &mut StableSim<'_>, t: usize, kind: CcKindV1, ticks: usize, rec: usize, tick: usize) {
        sim.apply_cc(t, &CcV1::of_kind(kind, ticks as u64));
        let w = match kind { CcKindV1::Stun | CcKindV1::Bind | CcKindV1::Airborne => 1.0, _ => 0.8 };
        self.credit(rec, ticks as f64 / 60.0 * 100.0 * w, Some(t));
        if matches!(kind, CcKindV1::Stun | CcKindV1::Bind | CcKindV1::Airborne) && ticks >= 40 {
            self.cc_mark = Some((t, tick + ticks));
        }
    }

    fn slow(&mut self, sim: &mut StableSim<'_>, t: usize, pct: i32, ticks: usize, rec: usize) {
        sim.entity_remove_buff(t, "scr_slow");
        let mut b = timed("scr_slow", ticks);
        b.move_speed_mult = -pct;
        sim.add_buff(t, &b);
        self.credit(rec, pct as f64 * ticks as f64 / 60.0, Some(t));
    }

    /// Pick (or keep) what to weave toward.
    /// A spell's worth right now for this athlete, before their misreads (None = no use for it now).
    fn score(&self, s: usize, c: &Ctx, mem: &Memory, dead_ally: bool) -> Option<f64> {
        let r = self.rank();
        let wi = self.weave_m() as f64 / 1000.0;
        let w = 0.4 + 0.1 * r.min(6) as f64;
        let p = eval(s, c, &self.hist, dead_ally)?;
        let mut v = p.value * mem.factor(s, p.bucket).powf(w);
        // experienced players know a slow spell only lands on someone who can't move
        if r >= 3 && p.delay > 1 {
            let held = p.target.and_then(|t| c.foes.iter().find(|e| e.id == t)).map_or(0, |e| e.held);
            v *= if held + 6 >= p.delay { 1.0 } else { 0.35 };
        }
        // chaining: right after a lockdown they reach for the big, slow hits on that target
        if r >= 3 {
            if let Some((t, until)) = self.cc_mark {
                if until > c.tick && p.target == Some(t) && (p.delay > 1 || matches!(s, 14 | 19 | 23)) { v *= 1.0 + 0.12 * r as f64; }
            }
        }
        // the closer the dots already are, the cheaper it is
        let have = common_prefix(&self.dots, SPELLS[s].recipe);
        v *= 1.0 + 0.15 * have as f64;
        // bigger spells take longer to weave: a little patience tax for the slow weavers
        v /= 1.0 + 0.05 * (tier(s) as f64) * (wi / WEAVE_BASE as f64);
        // round 72: any rank may go for any spell, but a recipe past their comfort is likely to come out wrong;
        // how much they account for that grows with rank (rookies still try)
        v *= build_chance(r, have, tier(s)).powf(AWARE[r]);
        Some(v)
    }

    /// The spell's tier is on cooldown for longer than it would take to weave it.
    fn tier_locked(&self, s: usize, c: &Ctx) -> bool {
        let need = tier(s).saturating_sub(common_prefix(&self.dots, SPELLS[s].recipe));
        self.ready[s] > c.tick + self.build_ticks(need)
    }

    /// Round 72: every rank can go for every spell (their slips decide whether it comes out); only the cooldown
    /// matters here.
    fn usable(&self, s: usize, c: &Ctx) -> bool {
        !self.tier_locked(s, c)
    }

    /// What they would prepare as an opener: only recipes they can build reliably.
    fn preparable(&self, s: usize, c: &Ctx) -> bool {
        tier(s) <= COMFORT_TIER[self.rank()] && self.usable(s, c)
    }

    /// Pick (or keep) what to weave toward.
    fn choose(&mut self, c: &Ctx, dead_ally: bool) {
        let r = self.rank();
        if self.goal.map_or(true, |g| !self.tier_locked(g, c)) { self.cd_wait = None; }
        let mem = self.mem.clone().unwrap_or_else(|| Arc::new(memory().clone()));
        let mut best: Option<(usize, f64, f64)> = None; // spell, value as he reads it, value
        for s in 0..N {
            if !self.usable(s, c) { continue; }
            let Some(v) = self.score(s, c, &mem, dead_ally) else { continue };
            let noise = NOISE[r];
            let seen = v * (1.0 - noise + 2.0 * noise * self.unit());
            if seen > best.map_or(30.0, |b| b.1) { best = Some((s, seen, v)); }
        }
        match (self.goal, best) {
            (_, None) => {
                // nothing to do with any spell right now: experienced players prepare the opener the situation
                // calls for (round 68: a gank, a teamfight, an escape) and hold it; Expert+ re-pick when it changes
                let quiet = c.foes.iter().all(|e| d2(e.x, e.y, c.me.x, c.me.y) > sq(90_000));
                if r >= 2 && quiet {
                    let intent = intent_of(c);
                    // round 70: an opener whose tier is on cooldown is skipped for the next one in the list
                    let pick = prep_list(intent).iter().copied().find(|&s| self.preparable(s, c));
                    let locked = self.goal.map_or(false, |g| self.tier_locked(g, c));
                    let repick = self.goal.is_none() || locked || (r >= 3 && self.prep != Some(intent) && pick != self.goal);
                    if let (Some(s), true) = (pick, repick) {
                        self.set_goal(s, 200.0);
                        self.prep = Some(intent);
                    }
                }
            }
            (Some(g), Some((s, seen, _))) if g != s && self.tier_locked(g, c) => {
                // round 70: the spell he wants has its tier on cooldown (a spell with as many dots was just cast):
                // once he notices, he goes for the next best one he can cast, keeping the dots that start it (4-4 for
                // Mallet becomes Wind-up Key, say)
                let since = *self.cd_wait.get_or_insert(c.tick);
                if c.tick >= since + CD_NOTICE[r] {
                    self.cd_wait = None;
                    self.set_goal(s, seen);
                    self.prep = None;
                }
            }
            (Some(g), Some((s, seen, v))) if g != s => {
                // round 68 (Rian): is the spell he is holding still the right one? Beginners stick with what they
                // started (they only see it through their misreads); from Expert on he reads the held spell's real
                // worth now and drops it at once when it has none, or switches when another is clearly better. The
                // dots that start the new spell are kept, the rest are flicked away.
                let cur = if self.usable(g, c) { self.score(g, c, &mem, dead_ally).unwrap_or(0.0) } else { 0.0 };
                let switch = if r >= 3 {
                    cur <= 0.0 || v > cur * SWITCH_K[r]
                } else {
                    seen > self.goal_score * SWITCH_K[r] || self.ready[g] > c.tick + 600
                };
                if switch {
                    self.set_goal(s, seen);
                    self.prep = None;
                }
            }
            (Some(_), Some((_, seen, _))) => self.goal_score = seen,
            (None, Some((s, seen, _))) => {
                self.set_goal(s, seen);
                self.prep = None;
            }
        }
    }

    /// A new goal: dots that don't start it are flicked away (the ones that do are kept). An unnoticed wrong dot
    /// under the same goal stays (that is how a misfire gets cast).
    fn set_goal(&mut self, s: usize, v: f64) {
        if self.goal != Some(s) && !SPELLS[s].recipe.starts_with(&self.dots) {
            self.dots.clear();
            self.flick_at = None;
        }
        self.goal = Some(s);
        self.goal_score = v;
        self.held_since = None;
    }

    fn start_invoke(&mut self, sim: &mut StableSim<'_>, me: &U, spell: Option<usize>, value: f64, tick: usize) {
        let t = INVOKE_T[self.rank()];
        let mut a = CcV1::of_kind(CcKindV1::Animation, t as u64);
        a.set_name("ult");
        sim.apply_cc(me.id, &a);
        fx_on(sim, "invoke", me.id, me.id, t as u64);
        self.invoking = Some((spell, tick + t, value));
        self.held_since = None;
    }

    fn resolve(&mut self, sim: &mut StableSim<'_>, c: &Ctx, spell: Option<usize>, promised: f64, dead_ally: bool) {
        let me = c.me;
        let tick = c.tick;
        let sp_ = match spell {
            Some(s) if self.ready[s] <= tick => s,
            _ => {
                // no such recipe (or it is still on cooldown): a puff of smoke
                self.fizzles += 1;
                fx_on(sim, "fizzle", me.id, me.id, 30);
                return;
            }
        };
        let plan = eval(sp_, c, &self.hist, dead_ally);
        // round 64 (Rian): the cooldown is per dot count, not per spell. A 5-dot spell puts every 5-dot spell on its
        // cooldown; the 1-4 and 6-dot spells stay free. (Before, each of the 35 had its own, so an Archmage could chain
        // all three 6-dot spells.)
        let until = tick + SPELLS[sp_].cd;
        for s in (0..N).filter(|&s| tier(s) == tier(sp_)) { self.ready[s] = self.ready[s].max(until); }
        let Some(p) = plan else {
            // the moment passed (or a misfired spell with nothing to do): wasted, and scored as such
            fx_on(sim, "fizzle", me.id, me.id, 30);
            let bucket = c.bucket(me.x, me.y);
            self.casts += 1;
            let idx = self.casts;
            self.done.push((idx, sp_, bucket, 0.0));
            let _ = promised;
            return;
        };
        let tgt = p.target.and_then(|t| c.foes.iter().chain(c.allies.iter()).find(|u| u.id == t).copied());
        let rec = self.record(sp_, &p, p.target.into_iter().collect(), tick);
        let (mx, my) = (me.x, me.y);
        match sp_ {
            0 => {
                let t = tgt.unwrap();
                fx(sim, "poke", me.id, t.x, t.y, 18);
                self.hit(sim, me.id, t.id, c.dmg(30, 50), rec);
            }
            1 => {
                let t = tgt.unwrap();
                fx_on(sim, "smudge", me.id, t.id, 22);
                if t.shield > 0 {
                    sim.entity_clear_shield(t.id);
                    self.credit(rec, t.shield as f64, None);
                }
                self.hit(sim, me.id, t.id, c.dmg(20, 30), rec);
            }
            2 => {
                let t = tgt.unwrap();
                fx(sim, "splat", me.id, t.x, t.y, 22);
                for e in c.foes_in(t.x, t.y, 15_000) {
                    self.hit(sim, me.id, e.id, c.dmg(25, 40), rec);
                    self.slow(sim, e.id, 20, 60, rec);
                }
            }
            3 => {
                fx_on(sim, "honk", me.id, me.id, 22);
                for e in c.foes_in(mx, my, 30_000) {
                    self.hit(sim, me.id, e.id, c.dmg(15, 30), rec);
                    self.cc(sim, e.id, CcKindV1::Stun, 12, rec, tick);
                }
            }
            4 => {
                let t = tgt.unwrap();
                fly(sim, "page_fly", me, t.x, t.y, 9_000);
                let travel = (dist(mx, my, t.x, t.y) / 9_000.0) as usize + 1;
                self.later.push(Later::Bleed { at: tick + travel, t: t.id, dmg: c.dmg(40, 60) / 4, left: 4, rec });
            }
            5 => {
                let (dx, dy) = match tgt {
                    Some(t) => norm((t.x - mx) as f64, (t.y - my) as f64),
                    None => c.escape_dir(),
                };
                let (x, y) = walls::pull_back(mx, my, clampm(mx + (dx * 35_000.0) as i64), clampm(my + (dy * 35_000.0) as i64));
                fx(sim, "door", me.id, mx, my, 30);
                sim.entity_set_pos(me.id, x as u64, y as u64);
                fx(sim, "door", me.id, x, y, 30);
                self.credit(rec, p.value, None);
            }
            6 => {
                let t = tgt.unwrap();
                fx_on(sim, "cutout", me.id, t.id, 20);
                let sh = c.dmg(80, 60);
                sim.entity_add_shield(t.id, sh, 180);
                self.credit(rec, sh as f64 * 0.7, None);
            }
            7 => {
                fx(sim, "erase_self", me.id, mx, my, 26);
                sim.entity_set_invisible(me.id, 90);
                self.credit(rec, p.value, None);
            }
            8 => {
                let t = tgt.unwrap();
                fx_on(sim, "bucket", me.id, t.id, 30);
                let h = c.dmg(70, 50).min(t.max_hp - t.hp);
                sim.heal(me.id, t.id, h);
                self.credit(rec, h as f64, None);
            }
            9 => {
                fx(sim, "peel", me.id, p.x, p.y, 36);
                self.later.push(Later::Peel { until: tick + 300, x: p.x, y: p.y, next: tick + 36, rec });
            }
            10 => {
                let t = tgt.unwrap();
                fx(sim, "chicken", me.id, t.x, t.y, 18);
                self.hit(sim, me.id, t.id, c.dmg(40, 50), rec);
                if sim.entity_knockback(me.id, t.id, 3_000, 12) { self.credit(rec, 40.0, None); }
            }
            11 => {
                fx_on(sim, "key", me.id, me.id, 180);
                sim.entity_remove_buff(me.id, "scr_key");
                let mut b = timed("scr_key", 180);
                b.move_speed_mult = 40;
                b.attack_speed_mult = 40;
                sim.add_buff(me.id, &b);
                self.credit(rec, p.value, None);
            }
            12 => {
                let t = tgt.unwrap();
                fx_on(sim, "bubble", me.id, t.id, 48);
                self.cc(sim, t.id, CcKindV1::BlockAttack, 48, rec, tick);
                self.credit(rec, t.attack as f64 * 0.8, None);
            }
            13 => {
                let t = tgt.unwrap();
                fx_on(sim, "phone", me.id, me.id, 26);
                for e in c.foes.iter().filter(|e| c.in_cone(t.x, t.y, e, 45_000, 35.0)).map(|e| e.id).collect::<Vec<_>>() {
                    fx_on(sim, "photo", me.id, e, 72);
                    self.cc(sim, e, CcKindV1::Stun, 72, rec, tick);
                }
            }
            14 => {
                let t = tgt.unwrap();
                fx(sim, "mallet", me.id, t.x, t.y, 18);
                self.hit(sim, me.id, t.id, c.dmg(60, 80), rec);
                self.cc(sim, t.id, CcKindV1::Stun, 60, rec, tick);
            }
            15 => {
                let t = tgt.unwrap();
                fx_all(sim, "anvil_shadow", me.id, t.x, t.y, 36);
                self.later.push(Later::Shadow { until: tick + 48, x: t.x, y: t.y, tag: "anvil_shadow", next: tick + 36 });
                self.later.push(Later::Drop { at: tick + 48, x: t.x, y: t.y, r: 12_000, dmg: c.dmg(90, 100), stun: 60, tag: "anvil", rec });
            }
            16 => {
                let t = tgt.unwrap();
                fly(sim, "glove", me, t.x, t.y, 12_000);
                if sim.entity_grab(me.id, t.id, 4_000, 0) { self.credit(rec, 120.0, Some(t.id)); }
            }
            17 => {
                let t = tgt.unwrap();
                fx_on(sim, "erase_legs", me.id, t.id, 90);
                self.cc(sim, t.id, CcKindV1::Bind, 90, rec, tick);
            }
            18 => {
                let t = tgt.unwrap();
                fx_on(sim, "pie", me.id, t.id, 60);
                self.cc(sim, t.id, CcKindV1::BlockSkill, 120, rec, tick);
                self.cc(sim, t.id, CcKindV1::BlockAttack, 60, rec, tick);
            }
            19 => {
                let t = tgt.unwrap();
                fx_on(sim, "present", me.id, t.id, 120);
                self.later.push(Later::Boom { at: tick + 120, t: t.id, dmg: c.dmg(120, 110), rec });
            }
            20 => {
                fx_all(sim, "floor", me.id, p.x, p.y, 30);
                self.later.push(Later::Floor { until: tick + 240, x: p.x, y: p.y, next: tick });
                self.credit(rec, p.value * 0.8, None);
            }
            21 => {
                fx(sim, "hole", me.id, mx, my, 40);
                let who: Vec<usize> = c.allies_in(mx, my, 40_000).iter().filter(|a| a.id != me.id).map(|a| a.id).collect();
                sim.entity_set_pos(me.id, p.x as u64, p.y as u64);
                fx(sim, "hole", me.id, p.x, p.y, 70);
                fx_on(sim, "hole_pop", me.id, me.id, 16);
                self.later.push(Later::Follow { at: tick + 60, x: p.x, y: p.y, who });
                self.credit(rec, p.value, None);
            }
            22 => {
                let t = tgt.unwrap();
                fx_all(sim, "panorama", me.id, (mx + t.x) / 2, (my + t.y) / 2, 26);
                for e in c.foes.iter().filter(|e| c.in_cone(t.x, t.y, e, 70_000, 60.0)).map(|e| e.id).collect::<Vec<_>>() {
                    fx_on(sim, "photo", me.id, e, 90);
                    self.cc(sim, e, CcKindV1::Stun, 90, rec, tick);
                }
            }
            23 => {
                let t = tgt.unwrap();
                let ts: Vec<usize> = c.foes_in(t.x, t.y, 15_000).iter().map(|e| e.id).collect();
                for &e in &ts {
                    fx_on(sim, "brawl", me.id, e, 120);
                    self.cc(sim, e, CcKindV1::Bind, 120, rec, tick);
                }
                self.later.push(Later::Brawl { at: tick + 24, ts, dmg: c.dmg(30, 25), left: 4, rec });
            }
            24 => {
                fx(sim, "redraw", me.id, mx, my, 22);
                sim.entity_clear_cc(me.id);
                sim.entity_set_pos(me.id, p.x as u64, p.y as u64);
                fx_on(sim, "redraw", me.id, me.id, 22);
                self.credit(rec, p.value, None);
            }
            25 => {
                fx_all(sim, "stamp", me.id, mx, my, 26);
                for e in c.foes_in(mx, my, 35_000).iter().map(|e| e.id).collect::<Vec<_>>() {
                    self.cc(sim, e, CcKindV1::Stun, 72, rec, tick);
                }
                let sh = c.dmg(120, 80);
                for a in c.allies_in(mx, my, 35_000) {
                    sim.entity_add_shield(a.id, sh, 240);
                    self.credit(rec, sh as f64 * 0.7, None);
                }
            }
            26 => {
                // the wall: across his escape path (between him and the chasers), or behind a fleeing enemy
                let (cx, cy, d) = match tgt {
                    Some(t) if t.team != me.team => {
                        let d = norm((t.x - mx) as f64, (t.y - my) as f64);
                        (t.x + (d.0 * 12_000.0) as i64, t.y + (d.1 * 12_000.0) as i64, d)
                    }
                    _ => {
                        let e = c.escape_dir();
                        (mx - (e.0 * 14_000.0) as i64, my - (e.1 * 14_000.0) as i64, (-e.0, -e.1))
                    }
                };
                let (px, py) = (-d.1, d.0);
                let half = 45_000.0;
                let (ax, ay) = (clampm(cx + (px * half) as i64), clampm(cy + (py * half) as i64));
                let (bx, by) = (clampm(cx - (px * half) as i64), clampm(cy - (py * half) as i64));
                let (t0, tb, tend) = (tick, tick + 12, tick + 300);
                sim.add_buff(me.id, &timed(&format!("sbw:{ax}:{ay}:{bx}:{by}:{t0}:{tb}:{tend}"), tend - tick + 5));
                self.later.push(Later::Wall { until: tend, ax, ay, bx, by, next: tick, drawn: false });
                self.credit(rec, p.value, None);
            }
            27 => {
                fx_all(sim, "rewind", me.id, mx, my, 26);
                let then = self.hist.iter().find(|h| h.0 + 240 <= tick && h.0 + 270 >= tick).cloned();
                if let Some((_, past)) = then {
                    for a in c.allies_in(mx, my, 70_000) {
                        let Some(&(_, x, y, hp)) = past.iter().find(|p| p.0 == a.id) else { continue };
                        fx_on(sim, "rewind_swirl", me.id, a.id, 22);
                        if !walls::wall_at(x, y) { sim.entity_set_pos(a.id, x as u64, y as u64); }
                        if hp > a.hp {
                            sim.heal(me.id, a.id, hp - a.hp);
                            self.credit(rec, (hp - a.hp) as f64, None);
                        }
                    }
                }
            }
            28 => {
                let t = tgt.unwrap();
                fx_all(sim, "piano_shadow", me.id, t.x, t.y, 42);
                self.later.push(Later::Shadow { until: tick + 60, x: t.x, y: t.y, tag: "piano_shadow", next: tick + 42 });
                self.later.push(Later::Drop { at: tick + 60, x: t.x, y: t.y, r: 30_000, dmg: c.dmg(200, 150), stun: 90, tag: "piano", rec });
            }
            29 => {
                let t = tgt.unwrap();
                let d = norm((t.x - mx) as f64, (t.y - my) as f64);
                for k in 0..8 {
                    let s = 10_000.0 + k as f64 * 20_000.0;
                    fx_all(sim, "ink_wave", me.id, mx + (d.0 * s) as i64, my + (d.1 * s) as i64, 26);
                }
                for e in c.foes.iter().filter(|e| line_hit(mx, my, t.x, t.y, e, 160_000, 20_000)).map(|e| e.id).collect::<Vec<_>>() {
                    self.hit(sim, me.id, e, c.dmg(60, 50), rec);
                    self.cc(sim, e, CcKindV1::BlockAttack, 120, rec, tick);
                    self.slow(sim, e, 40, 120, rec);
                }
            }
            30 => {
                fx_on(sim, "chase", me.id, me.id, 240);
                sim.entity_remove_buff(me.id, "scr_chase");
                let mut b = timed("scr_chase", 240);
                b.move_speed_mult = 100;
                sim.add_buff(me.id, &b);
                self.later.push(Later::Chase { until: tick + 240, hit: Vec::new(), dmg: c.dmg(40, 40), rec });
                self.credit(rec, 100.0, None);
            }
            31 => {
                fx_all(sim, "laugh", me.id, mx, my - 30_000, 30);
                for e in c.foes_in(mx, my, 90_000).iter().map(|e| e.id).collect::<Vec<_>>() {
                    fx_on(sim, "haha", me.id, e, 90);
                    self.cc(sim, e, CcKindV1::BlockAttack, 90, rec, tick);
                    self.cc(sim, e, CcKindV1::BlockSkill, 90, rec, tick);
                }
                let h = c.dmg(60, 40);
                for a in c.allies_in(mx, my, 90_000) {
                    let hh = h.min(a.max_hp - a.hp);
                    sim.heal(me.id, a.id, hh);
                    self.credit(rec, hh as f64, None);
                }
            }
            32 => {
                fx_all(sim, "page", me.id, MAP / 2, MAP / 2, 30);
                let everyone = snapshot(sim);
                for u in everyone.iter().filter(|u| off_mid(u.x, u.y)) {
                    let (nx, ny) = flip(u.x, u.y);
                    let spot = free_near(nx, ny);
                    if let Some((x, y)) = spot {
                        fx_on(sim, "page_swish", me.id, u.id, 16);
                        sim.entity_set_pos(u.id, x as u64, y as u64);
                    }
                }
                self.credit(rec, p.value, None);
            }
            33 => {
                fx_all(sim, "pause", me.id, mx, my - 40_000, 120);
                let foes: Vec<usize> = snapshot(sim).iter().filter(|u| u.team != me.team).map(|u| u.id).collect();
                for e in foes {
                    fx_on(sim, "pause_icon", me.id, e, 120);
                    self.cc(sim, e, CcKindV1::Stun, 120, rec, tick);
                }
            }
            34 => {
                self.draw_friend(sim, me, rec);
            }
            _ => {}
        }
    }

    fn draw_friend(&mut self, sim: &mut StableSim<'_>, me: &U, rec: usize) {
        // the strongest fallen teammate, sketched back in at 60 % for 10 s
        let mut best: Option<(usize, String, StatV1, usize)> = None;
        for i in 0..sim.champion_count() {
            let id = sim.champion_id_at(i);
            let Some(e) = sim.get_entity(id) else { continue };
            if e.is_alive() || e.team() != me.team || id == me.id { continue; }
            let st = e.stat();
            let mx = e.hp().1;
            if best.as_ref().map_or(true, |b| st.attack + st.magic_power > b.2.attack + b.2.magic_power) {
                best = Some((id, e.name().unwrap_or_default(), st, mx));
            }
        }
        let Some((id, name, st, max_hp)) = best else {
            fx_on(sim, "fizzle", me.id, me.id, 30);
            return;
        };
        let sprite = crate::valorant::BASE_SPRITES.iter().find(|n| **n == name).copied().unwrap_or(crate::valorant::SHADOW_SPRITE);
        let stat = StatV1 {
            attack: st.attack * 6 / 10, magic_power: st.magic_power * 6 / 10, hp: (max_hp.max(st.hp) * 6 / 10).max(300),
            defence: st.defence * 6 / 10, magic_resistance: st.magic_resistance * 6 / 10, move_speed: st.move_speed.max(900), ..StatV1::default()
        };
        let atk = UnitAttackV1 {
            attack_ratio: 100, attack: 0, range: 30_000, cooltime: 80, duration: 20, start_timing: 12, cancelable: true,
            attack_type: AttackTypeV1::BaseAttack.code(),
        };
        let (x, y) = walls::pull_back(me.x, me.y, clampm(me.x + 8_000), clampm(me.y - 8_000));
        if let Some(uid) = sim.spawn_unit(sprite, me.id, me.team, x as u64, y as u64, 600, &stat, &atk) {
            fx_on(sim, "sketch_in", me.id, uid, 34);
            sim.add_buff(uid, &timed("scr_sketch", 600));
            self.credit(rec, 260.0, None);
        }
        let _ = id;
    }

    fn run_later(&mut self, sim: &mut StableSim<'_>, me: &U, all: &[U], tick: usize) {
        let later = std::mem::take(&mut self.later);
        let mut keep = Vec::new();
        let foes_in = |x: i64, y: i64, r: i64| -> Vec<usize> {
            all.iter().filter(|u| u.team != me.team && d2(u.x, u.y, x, y) <= sq(r)).map(|u| u.id).collect()
        };
        for l in later {
            match l {
                Later::Drop { at, x, y, r, dmg, stun, tag, rec } => {
                    if tick < at { keep.push(Later::Drop { at, x, y, r, dmg, stun, tag, rec }); continue; }
                    fx_all(sim, tag, me.id, x, y, 24);
                    for e in foes_in(x, y, r) {
                        self.hit(sim, me.id, e, dmg, rec);
                        self.cc(sim, e, CcKindV1::Stun, stun, rec, tick);
                        fx_on(sim, "bonk", me.id, e, 16);
                    }
                }
                Later::Bleed { at, t, dmg, left, rec } => {
                    if tick < at { keep.push(Later::Bleed { at, t, dmg, left, rec }); continue; }
                    if all.iter().any(|u| u.id == t) {
                        fx_on(sim, "cut", me.id, t, 18);
                        self.hit(sim, me.id, t, dmg, rec);
                        if left > 1 { keep.push(Later::Bleed { at: tick + 30, t, dmg, left: left - 1, rec }); }
                    }
                }
                Later::Boom { at, t, dmg, rec } => {
                    if tick < at { keep.push(Later::Boom { at, t, dmg, rec }); continue; }
                    if let Some(u) = all.iter().find(|u| u.id == t) {
                        fx_all(sim, "boom", me.id, u.x, u.y, 26);
                        for e in foes_in(u.x, u.y, 20_000) {
                            self.hit(sim, me.id, e, dmg, rec);
                        }
                    }
                }
                Later::Follow { at, x, y, who } => {
                    if tick < at { keep.push(Later::Follow { at, x, y, who }); continue; }
                    for (k, id) in who.iter().enumerate() {
                        if !all.iter().any(|u| u.id == *id) { continue; }
                        let a = k as f64 * 2.1;
                        let (tx, ty) = walls::pull_back(x, y, clampm(x + (a.cos() * 9_000.0) as i64), clampm(y + (a.sin() * 9_000.0) as i64));
                        sim.entity_set_pos(*id, tx as u64, ty as u64);
                        fx_on(sim, "hole_pop", me.id, *id, 16);
                    }
                }
                Later::Brawl { at, ts, dmg, left, rec } => {
                    if tick < at { keep.push(Later::Brawl { at, ts, dmg, left, rec }); continue; }
                    for &e in &ts {
                        if all.iter().any(|u| u.id == e) {
                            self.hit(sim, me.id, e, dmg, rec);
                            fx_on(sim, "bonk", me.id, e, 16);
                        }
                    }
                    if left > 1 { keep.push(Later::Brawl { at: tick + 28, ts, dmg, left: left - 1, rec }); }
                }
                Later::Floor { until, x, y, next } => {
                    if tick >= until { continue; }
                    let mut next = next;
                    if tick >= next {
                        fx_all(sim, "floor", me.id, x, y, 30);
                        next = tick + 30;
                    }
                    if tick % 10 == 0 {
                        for u in all.iter().filter(|u| d2(u.x, u.y, x, y) <= sq(40_000)) {
                            let n = if u.team == me.team { "scr_floor_up" } else { "scr_floor_dn" };
                            sim.entity_remove_buff(u.id, n);
                            let mut b = timed(n, 12);
                            b.move_speed_mult = if u.team == me.team { 25 } else { -30 };
                            sim.add_buff(u.id, &b);
                        }
                    }
                    keep.push(Later::Floor { until, x, y, next });
                }
                Later::Peel { until, x, y, next, rec } => {
                    if tick >= until { continue; }
                    let hit = all.iter().filter(|u| u.team != me.team && d2(u.x, u.y, x, y) <= sq(10_000)).min_by_key(|u| u.id).map(|u| u.id);
                    if let Some(e) = hit {
                        fx_on(sim, "slip", me.id, e, 22);
                        self.cc(sim, e, CcKindV1::Airborne, 30, rec, tick);
                        sim.entity_knockback(me.id, e, 2_000, 14);
                        continue;
                    }
                    let mut next = next;
                    if tick >= next {
                        fx(sim, "peel", me.id, x, y, 36);
                        next = tick + 36;
                    }
                    keep.push(Later::Peel { until, x, y, next, rec });
                }
                Later::Chase { until, mut hit, dmg, rec } => {
                    if tick >= until { continue; }
                    for e in all.iter().filter(|u| u.team != me.team && !hit.contains(&u.id) && d2(u.x, u.y, me.x, me.y) <= sq(14_000)).map(|u| u.id).collect::<Vec<_>>() {
                        hit.push(e);
                        fx_on(sim, "bonk", me.id, e, 16);
                        self.hit(sim, me.id, e, dmg, rec);
                        self.cc(sim, e, CcKindV1::Stun, 18, rec, tick);
                    }
                    keep.push(Later::Chase { until, hit, dmg, rec });
                }
                Later::Wall { until, ax, ay, bx, by, next, drawn } => {
                    if tick >= until { continue; }
                    let (mut next, mut drawn) = (next, drawn);
                    if tick >= next {
                        // drawn in with the pencil first, then the finished wall, refreshed until it is rubbed out
                        let n = (dist(ax, ay, bx, by) / 20_000.0).ceil().max(1.0) as i64;
                        for k in 0..=n {
                            let (x, y) = (ax + (bx - ax) * k / n, ay + (by - ay) * k / n);
                            fx_all(sim, if drawn { "wall_seg" } else { "wall_draw" }, me.id, x, y, if drawn { 48 } else { 14 });
                        }
                        next = tick + if drawn { 48 } else { 12 };
                        drawn = true;
                    }
                    keep.push(Later::Wall { until, ax, ay, bx, by, next, drawn });
                }
                Later::Shadow { until, x, y, tag, next } => {
                    if tick >= until { continue; }
                    let mut next = next;
                    if tick >= next {
                        fx_all(sim, tag, me.id, x, y, (until - tick) as u64);
                        next = until;
                    }
                    keep.push(Later::Shadow { until, x, y, tag, next });
                }
            }
        }
        keep.extend(std::mem::take(&mut self.later));
        self.later = keep;
    }

    fn close_records(&mut self, sim: &StableSim<'_>, tick: usize) {
        let recs = std::mem::take(&mut self.recs);
        for mut r in recs {
            if tick < r.close {
                self.recs.push(r);
                continue;
            }
            for t in &r.targets {
                if sim.get_entity(*t).map_or(false, |e| e.is_champion() && !e.is_alive()) { r.got += 300.0; }
            }
            self.done.push((r.idx, r.spell, r.bucket, (r.got / r.est).clamp(0.0, 3.0)));
        }
    }
}

fn common_prefix(a: &[u8], b: &[u8]) -> usize {
    a.iter().zip(b).take_while(|(x, y)| x == y).count()
}

fn free_near(x: i64, y: i64) -> Option<(i64, i64)> {
    let (x, y) = (clampm(x), clampm(y));
    if !walls::wall_at(x, y) { return Some((x, y)); }
    for r in [8_000i64, 16_000, 24_000, 32_000, 40_000] {
        for k in 0..8 {
            let a = k as f64 * std::f64::consts::PI / 4.0;
            let (px, py) = (clampm(x + (a.cos() * r as f64) as i64), clampm(y + (a.sin() * r as f64) as i64));
            if !walls::wall_at(px, py) { return Some((px, py)); }
        }
    }
    None
}

impl StablePassive for Scribble {
    fn clone_box(&self) -> Box<dyn StablePassive> {
        Box::new(self.clone())
    }
    fn on_spawn(&mut self, _sim: &mut StableSim<'_>, _player: usize, _entity: usize) {
        // a fresh body: no dots, no HUD (it is put back on the next update)
        self.clear_dots();
        self.shown.clear();
        self.rank_shown = None;
        self.invoking = None;
        self.goal = None;
    }
    fn on_update(&mut self, sim: &mut StableSim<'_>, _seed: u64, player: usize, entity: usize) {
        let tick = sim.tick();
        if !self.started {
            self.started = true;
            self.rng = sim.seed() ^ (entity as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ 0x5C21_BB1E;
            self.ready = vec![0; N];
            self.by_spell = vec![0; N];
            self.entity = entity;
            let _ = memory();
        }
        if self.rank.is_none() && tick >= 60 {
            self.latch_rank(sim, player);
        }
        if self.sig.is_some() && tick % 30 == 0 { self.track_result(sim, tick); }
        if self.sig.is_some() && tick >= self.summary_tick + 1800 { self.summary_tick = tick; self.flush(); self.summary(tick); }
        let all = snapshot(sim);
        let Some(me) = all.iter().find(|u| u.id == entity).cloned() else {
            if self.was_alive {
                self.was_alive = false;
                self.clear_dots();
                self.shown.clear();
                self.rank_shown = None;
                self.invoking = None;
            }
            return;
        };
        if self.team.is_none() { self.team = Some(me.team); }
        if !self.was_alive {
            self.was_alive = true;
            self.shown.clear();
            self.rank_shown = None;
        }
        // history for Redraw / Rewind (positions and HP of his team, the last 4.5 s)
        if tick % 6 == 0 {
            self.hist.push_back((tick, all.iter().filter(|u| u.team == me.team).map(|u| (u.id, u.x, u.y, u.hp)).collect()));
            while self.hist.front().map_or(false, |h| h.0 + 280 < tick) { self.hist.pop_front(); }
        }
        if self.sig.is_none() && tick >= 1800 && self.rank.is_some() {
            self.make_sig(sim, &all);
            self.log_summary(0);
        }
        if tick % 300 == 0 { self.flush(); }
        if tick % 30 == 0 { self.show_rank(sim, &me); }

        self.run_later(sim, &me, &all, tick);
        self.close_records(sim, tick);

        let foes: Vec<&U> = all.iter().filter(|u| u.team != me.team && sim.is_visible(me.team, u.id)).collect();
        let allies: Vec<&U> = all.iter().filter(|u| u.team == me.team).collect();
        let ap = sim.get_entity(entity).map_or(0, |e| e.stat().magic_power);
        let c = Ctx { me: &me, ap, allies, foes, tick };
        let dead_ally = (0..sim.champion_count()).map(|i| sim.champion_id_at(i))
            .any(|id| sim.get_entity(id).map_or(false, |e| !e.is_alive() && e.team() == me.team));

        // ---- an invoke in progress
        if let Some((spell, at, promised)) = self.invoking {
            if tick >= at {
                self.invoking = None;
                self.lock_until = tick + INVOKE_GAP;
                self.resolve(sim, &c, spell, promised, dead_ally);
                self.clear_dots();
                self.goal = None;
                self.show_dots(sim, entity);
            }
            return;
        }
        if self.rank.is_none() { return; }
        // the brain thinks every THINK_EVERY ticks; the hands weave every tick (round 72: the Top 10 click up to 15
        // dots a second, faster than the brain's beat)
        let think = tick % THINK_EVERY == entity % THINK_EVERY;
        if me.held > 0 || me.has("omn_blinded") { return; }
        let r = self.rank();

        // a noticed wrong dot: flick them away
        if let Some(at) = self.flick_at {
            if tick >= at {
                self.clear_dots();
                fx_on(sim, "weave", entity, entity, 9);
                self.show_dots(sim, entity);
            }
            return;
        }

        if think { self.choose(&c, dead_ally); }
        let Some(g) = self.goal else {
            if think && !self.dots.is_empty() && self.held_since.map_or(false, |h| tick > h + 600) {
                self.clear_dots();
                self.show_dots(sim, entity);
            }
            return;
        };
        let rec = SPELLS[g].recipe;

        // ---- weave the next dot
        let mut completed = false;
        if self.dots.len() < rec.len() {
            let now_m = tick as u64 * 1000;
            if self.dots.len() >= 6 || now_m < self.next_weave_m { return; }
            let want = rec[self.dots.len()];
            let mut el = want;
            // round 72: a dot past the athlete's comfort tier is an overreach, far likelier to come out wrong
            if self.roll(slip_pct(r, self.dots.len() + 1)) {
                el = (want - 1 + 1 + (self.next() % 4) as u8) % 5 + 1;
                self.misfires += 1;
                if self.roll(NOTICE[r]) { self.flick_at = Some(tick + 8); }
            }
            self.dots.push(el);
            fx_on(sim, "weave", entity, entity, 9);
            // keep the fraction of a tick between dots (a steady 4.3 ticks a dot), but never bank time while idle
            let base = self.next_weave_m.max(now_m.saturating_sub(999));
            self.next_weave_m = base + self.weave_m();
            self.show_dots(sim, entity);
            if self.dots.len() < rec.len() || self.flick_at.is_some() { return; }
            completed = true;
        }
        if !think && !completed { return; }

        // ---- the dots are complete: invoke (or hold)
        if self.dots.len() >= rec.len() && tick >= self.lock_until {
            let matched = recipe_of(&self.dots);
            if matched != Some(g) {
                // an unnoticed slip: they think it is the goal and invoke whatever it really is
                self.start_invoke(sim, &me, matched, self.goal_score, tick);
                return;
            }
            if self.ready[g] > tick { return; }
            match eval(g, &c, &self.hist, dead_ally) {
                Some(p) if p.value >= 40.0 => {
                    // experienced players wait for a slow spell's target to be locked down (up to their patience)
                    let held = p.target.and_then(|t| c.foes.iter().find(|e| e.id == t)).map_or(0, |e| e.held);
                    let wait = r >= 3 && p.delay > 1 && held + 6 < p.delay
                        && self.held_since.map_or(true, |h| tick < h + HOLD_T[r]);
                    if wait {
                        self.held_since.get_or_insert(tick);
                        return;
                    }
                    self.start_invoke(sim, &me, Some(g), p.value, tick);
                }
                _ => {
                    let since = *self.held_since.get_or_insert(tick);
                    let near = c.foes.iter().any(|e| d2(e.x, e.y, me.x, me.y) <= sq(70_000));
                    if HOLD_T[r] == 0 && near {
                        // beginners just fire it at whatever is around
                        self.start_invoke(sim, &me, Some(g), 40.0, tick);
                    } else if near && tick > since + HOLD_T[r] {
                        // the moment never came: pick something else (the dots stay if they start it)
                        self.goal = None;
                        self.held_since = None;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recipes_are_unique_and_tiered() {
        for (i, a) in SPELLS.iter().enumerate() {
            assert!(!a.recipe.is_empty() && a.recipe.len() <= 6);
            assert!(a.recipe.iter().all(|e| (1..=5).contains(e)), "{}", a.name);
            for b in SPELLS.iter().skip(i + 1) {
                assert_ne!(a.recipe, b.recipe, "{} / {}", a.name, b.name);
            }
        }
        let per: Vec<usize> = (1..=6).map(|t| SPELLS.iter().filter(|s| s.recipe.len() == t).count()).collect();
        assert_eq!(per, vec![5, 8, 8, 6, 5, 3]);
        assert_eq!(recipe_of(&[4, 4, 4, 1, 5, 2]), Some(33));
        assert_eq!(recipe_of(&[5, 5]), None);
    }

    #[test]
    fn ranks() {
        assert_eq!(rank_of(0), 0);
        assert_eq!(rank_of(4), 0);
        assert_eq!(rank_of(5), 1);
        assert_eq!(rank_of(29), 2);
        assert_eq!(rank_of(30), 3);
        assert_eq!(rank_of(99), 4);
        assert_eq!(rank_of(149), 5);
        assert_eq!(rank_of(150), 6);
        assert_eq!(rank_of(10_000), 6);
        assert_eq!(COMFORT_TIER[6], 6);
        // 2.5 dots a second for a Novice (24 ticks a dot), 11 for the Top 10 #10, 15 for #1 (4 ticks a dot)
        assert_eq!(weave_milli(0, None), 24_000);
        assert_eq!(cps100(TOP, Some(10)), 1100);
        assert_eq!(cps100(TOP, Some(1)), 1500);
        assert_eq!(weave_milli(TOP, Some(1)), 4_000);
        assert!((1..=TOP_SIZE).all(|p| (1100..=1500).contains(&cps100(TOP, Some(p)))));
        assert!((0..RANKS - 1).all(|r| cps100(r, None) < cps100(r + 1, Some(TOP_SIZE))));
        assert_eq!(badge_buff(3, None), "scr_rank3");
        assert_eq!(badge_buff(TOP, Some(1)), "scr_top1");
        assert_eq!(skin_of(4), None);
        assert_eq!(skin_of(5), Some(0));
        assert_eq!(skin_of(TOP), Some(2));
    }

    #[test]
    fn overreach_slips() {
        // a Novice going for a 6-dot spell: dot 3 80%, dot 4 86%, dot 5 92%, dot 6 98%
        assert_eq!((1..=6).map(|p| slip_pct(0, p)).collect::<Vec<_>>(), vec![16, 16, 80, 86, 92, 98]);
        // within the comfort tier only the rank's normal misfire chance applies
        assert_eq!(slip_pct(3, 4), 5);
        assert_eq!(slip_pct(3, 5), 40);
        assert_eq!(slip_pct(6, 6), 0);
        assert_eq!(slip_pct(TOP, 6), 0);
        // so a Novice almost never lands a 6-dot spell, an Archmage always does
        assert!(build_chance(0, 0, 6) < 0.001);
        assert!(build_chance(0, 0, 2) > 0.7);
        assert_eq!(build_chance(6, 0, 6), 1.0);
        // dots already woven count as done
        assert!(build_chance(0, 4, 6) > build_chance(0, 0, 6));
    }

    #[test]
    fn top_ten_needs_300_points_and_is_ordered() {
        let mut m = Memory::default();
        for a in 0..14usize {
            m.games.insert(a, Played { points: 290.0 + a as f64 * 5.0, games: 300 + a, wins: 0 });
        }
        // 290 and 295 are short of 300; the other 12 qualify and the best 10 are kept, best first
        let top = m.top_ten();
        assert_eq!(top.len(), 10);
        assert_eq!(top[0], 13);
        assert_eq!(top[9], 4);
        assert_eq!(m.rank_for(Some(13)), (TOP, Some(1)));
        assert_eq!(m.rank_for(Some(4)), (TOP, Some(10)));
        // qualified but outside the ten: back to their points rank (Archmage)
        assert_eq!(m.rank_for(Some(3)), (6, None));
        assert_eq!(m.rank_for(Some(0)), (6, None));
        assert_eq!(m.rank_for(None), (0, None));
        // a full tie is broken by games, then by the athlete id, the same way every time
        let mut t = Memory::default();
        t.games.insert(8, Played { points: 400.0, games: 300, wins: 1 });
        t.games.insert(2, Played { points: 400.0, games: 300, wins: 1 });
        t.games.insert(5, Played { points: 400.0, games: 310, wins: 0 });
        assert_eq!(t.top_ten(), vec![5, 2, 8]);
    }

    #[test]
    fn memory_round_trip_and_merge() {
        let mut m = Memory::default();
        // official game A (athlete 7 wins it), scrim B, old-format sig with a set index for A again (same game)
        m.merge("g 5.aa 7\ng 5.0.aa 7\ng x.bb 7\ng 5.aa 9\n\
                 r 5.aa 7 100 9 9 100 100 0\nr 5.aa 7 9000 9 4 100 0 3\nr 5.aa 9 9000 4 9 0 100 -3\n\
                 c 5.aa 7 1 14 3 1.5\nc 5.0.aa 7 1 14 3 1.5\nc 5.aa 7 2 14 3 0.5\nc x.bb 7 1 14 3 9\nbad line\n");
        let p7 = m.games[&7];
        assert_eq!(p7.games, 2);
        assert_eq!(p7.wins, 1);
        assert!((p7.points - (1.5 + 0.5)).abs() < 1e-9);   // official win 1.5 + scrim 0.5
        assert!((m.games_of(9) - 1.0).abs() < 1e-9);         // official loss
        assert_eq!(m.world, 2);
        let (sum, w) = m.meta[&(14, 3)];
        assert!((w - (1.25 + 1.25 + 0.5)).abs() < 1e-9);
        assert!((sum - (1.5 * 1.25 + 0.5 * 1.25 + 3.0 * 0.5)).abs() < 1e-9);
        let back = Memory::parse(&m.render());
        assert_eq!(back.games[&7].games, 2);
        assert!((back.games_of(7) - 2.0).abs() < 1e-9);
        assert!((back.meta[&(14, 3)].1 - 3.0).abs() < 1e-6);
        // the old two-number G lines still read
        assert!((Memory::parse("G 4 33\n").games_of(4) - 33.0).abs() < 1e-9);
        assert!(m.factor(14, 3) > 1.0);
        assert_eq!(m.factor(0, 0), 1.0);
        assert_eq!(result_of((9, 4, 100, 0, 3)), Some(true));
        assert_eq!(result_of((4, 9, 0, 100, -3)), Some(false));
        assert_eq!(result_of((5, 5, 100, 100, 0)), None);
    }

    fn unit(id: usize, team: usize, x: i64, y: i64, pct: usize) -> U {
        U { id, team, x, y, hp: pct * 10, max_hp: 1000, attack: 100, held: 0, shield: 0, name: String::new(), buffs: Vec::new() }
    }

    #[test]
    fn opener_follows_the_situation() {
        let me = unit(1, 0, 400_000, 500_000, 100);
        let mate = unit(2, 0, 400_000, 300_000, 80);   // top lane, 200000 away
        let foe = unit(3, 1, 420_000, 300_000, 70);    // fighting the teammate
        let (allies, foes) = (vec![&me, &mate], vec![&foe]);
        let c = Ctx { me: &me, ap: 100, allies, foes, tick: 0 };
        assert_eq!(intent_of(&c), Intent::Gank);
        let f2 = unit(4, 1, 430_000, 310_000, 70);
        let f3 = unit(5, 1, 440_000, 290_000, 70);
        let c = Ctx { me: &me, ap: 100, allies: vec![&me, &mate], foes: vec![&foe, &f2, &f3], tick: 0 };
        assert_eq!(intent_of(&c), Intent::Teamfight);
        let low = unit(1, 0, 400_000, 500_000, 30);
        let c = Ctx { me: &low, ap: 100, allies: vec![&low], foes: vec![], tick: 0 };
        assert_eq!(intent_of(&c), Intent::Escape);
        assert_eq!(prep_list(Intent::Gank)[0], 17);
        assert!(SWITCH_K.windows(2).all(|w| w[0] >= w[1]));
    }

    #[test]
    fn locked_tier_goes_to_the_next_spell() {
        let me = unit(1, 0, 400_000, 500_000, 100);
        let foe = unit(3, 1, 420_000, 500_000, 90);
        let c = Ctx { me: &me, ap: 100, allies: vec![&me], foes: vec![&foe], tick: 1000 };
        let mut sc = Scribble::default();
        sc.rank = Some(6);
        sc.ready = vec![0; N];
        sc.rng = 7;
        for s in (0..N).filter(|&s| tier(s) == 3) { sc.ready[s] = 1000 + 600; }   // 3-dot spells on cooldown for 10 s
        sc.goal = Some(14);            // Mallet 4-4-1
        sc.dots = vec![4, 4];
        sc.goal_score = 500.0;
        sc.choose(&c, false);
        let g = sc.goal.unwrap();
        assert_ne!(tier(g), 3, "still going for a locked 3-dot spell: {}", SPELLS[g].name);
        // the woven dots stay when the new spell starts with them
        assert!(SPELLS[g].recipe.starts_with(&sc.dots));
    }

    #[test]
    fn rookies_value_big_spells_by_their_odds() {
        // a teamfight: three enemies bunched in front of him
        let me = unit(1, 0, 400_000, 500_000, 100);
        let (f1, f2, f3) = (unit(3, 1, 430_000, 500_000, 80), unit(4, 1, 435_000, 505_000, 80), unit(5, 1, 432_000, 495_000, 80));
        let c = Ctx { me: &me, ap: 200, allies: vec![&me], foes: vec![&f1, &f2, &f3], tick: 1000 };
        let mem = Memory::default();
        let at = |rank: usize, s: usize| {
            let mut sc = Scribble::default();
            sc.rank = Some(rank);
            sc.top_pos = if rank == TOP { Some(1) } else { None };
            sc.ready = vec![0; N];
            sc.score(s, &c, &mem, false)
        };
        // every 4-, 5- and 6-dot spell that has a use here is worth far less to a Novice than to an Archmage
        let mut checked = 0;
        for s in (0..N).filter(|&s| tier(s) >= 4) {
            if let (Some(n), Some(a)) = (at(0, s), at(6, s)) {
                assert!(n < a * 0.5, "{}: novice {n:.0} vs archmage {a:.0}", SPELLS[s].name);
                checked += 1;
            }
        }
        assert!(checked > 0);
        // and it is still on the table: a rookie can go for it (no rank rules a spell out)
        let mut sc = Scribble::default();
        sc.rank = Some(0);
        sc.ready = vec![0; N];
        assert!((0..N).all(|s| sc.usable(s, &c)));
    }

    #[test]
    fn page_flip_keeps_mid_and_bases() {
        assert!(!off_mid(480_000, 480_000));
        assert!(!off_mid(96_000, 864_000));
        assert!(off_mid(100_000, 300_000));
        assert_eq!(flip(100_000, 300_000), (660_000, 860_000));
        assert_eq!(flip(96_000, 864_000), (96_000, 864_000));
    }
}

#[cfg(test)]
mod real_data {
    use super::*;
    /// SCRIBBLE_PENDING=<file> cargo test --release real_pending -- --ignored --nocapture
    #[test]
    #[ignore]
    fn real_pending() {
        let Ok(p) = std::env::var("SCRIBBLE_PENDING") else { return };
        let text = std::fs::read_to_string(p).unwrap();
        let mut m = Memory::default();
        m.merge(&text);
        let mut g: Vec<_> = m.games.iter().collect();
        g.sort_by(|a, b| b.1.points.partial_cmp(&a.1.points).unwrap());
        println!("world {} athletes {}", m.world, m.games.len());
        for (a, p) in g.iter().take(6) { println!("athlete {a}: {:.1} points, {} games, {} wins, rank {}", p.points, p.games, p.wins, RANK_NAMES[rank_of(p.points as usize)]); }
        let mut rows: Vec<(usize, f64, f64)> = (0..N).map(|s| {
            let (sum, w) = (0..BUCKETS).fold((0.0, 0.0), |acc, b| { let x = m.meta.get(&(s, b)).copied().unwrap_or((0.0, 0.0)); (acc.0 + x.0, acc.1 + x.1) });
            (s, w, if w > 0.0 { sum / w } else { 0.0 })
        }).filter(|r| r.1 > 0.0).collect();
        rows.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        for (s, w, avg) in rows { println!("{:16} weight {:7.1} delivered {:.2} factor(b0) {:.2}", SPELLS[s].name, w, avg, m.factor(s, 0)); }
    }
}
