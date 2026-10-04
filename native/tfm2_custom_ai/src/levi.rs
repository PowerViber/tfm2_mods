//! Levi (round 77, Rian): a cable flyer in the spirit of Fanny's cables.
//!
//! S1 fires one cable (the data slot has a tiny cooldown and reaches everywhere, so the game's AI keeps pressing it;
//! each press asks Levi's brain whether to fire now and where; nothing is ever injected). A cable bites the first
//! wall (or tower) along its line. At most two cables are active: they steer him (70 % toward the newest, 30 % toward
//! the older). Every cable that bites during a flight is a Chain stack and adds speed when it connects, by the angle
//! between the two active cables (best at 90°, a 0.25 floor so corridors still work). Speed has no cap: tight corners
//! do the limiting, since he turns at most TURN radians a tick. Reaching a cable's wall above CRASH_SPEED, or hitting
//! any other wall, slams him into it: stunned, longer the faster he was. A flight lapses FLIGHT_T ticks after the last
//! cable, and he glides to a stop.
//!
//! S2 is gas: on a cable a boost with no cooldown; off a cable a short burst dash (10 gas, 3 s cooldown). The tank
//! refills instantly at home. The ult, Rampage, makes him immune to crowd control (no crash stun) for 8 s, and every
//! enemy he passes is cut (once per pass), harder the faster he is.
//!
//! Round 80 (the Fanny pass): flights are meant to be seamless at the top. Cables fire all the way round him; one
//! that bites behind him whips him round to it (a turnback, TURNBACK_KEEP of his speed). He flies toward the game's
//! move order (the input AI hands it over, note_dest) along the walking path around the walls (aim_point), takes off
//! when it's more than a cable's reach away and stops chaining within one. In the air an S1 press holds the button
//! (HOLD_T): from Tethered up he fires the next cable when his timing says (LOOKAHEAD, off by up to JITTER), never
//! toward nothing, and from Swinger up he reads each candidate's flight path for walls (READ ticks, CLEAR clearance).
//! When no cable can save a flight he may brake into a soft landing (BRAKE percent, rolled per cable). Grounded fires
//! at the press with no timing and never brakes, so he hits walls; from Skyrunner up he doesn't. editor/levilab.js
//! mirrors all of it (keep the two in step) and measures it per rank.
//!
//! Round 81 (between the walls): two cables pull him between them (NEWEST_W: the newest one's share, 0.5 at the top,
//! so he flies down the middle); from Swinger up he pairs a lone cable at once and fires the next as he passes his
//! newest anchor (PASS), scoring a new anchor by where the pair would steer him and liking walls either side of his
//! line; a cable that falls behind lets go (RELEASE); cables under SHORT aren't worth it and he only takes off toward
//! walls ahead; when nothing carries a flight on, a reader lets go LETGO ticks out and drops onto open ground. The
//! cables hum (4 phases, a light pulse running along) and a new one shoots out over 4 ticks.
//!
//! Round 82 (the pair, from Rian's drawing): from Swinger up he fires his cables two at a time, one each side of where
//! he's going (pick_pair: 20-80° off it, the pull between them toward it, near a right angle, both long, a clear path
//! between and past them), and flies the diagonal between, never at an anchor; the next pair as he passes this one
//! (PAIR_PASS, early or late by his timing). Only an anchor in front of him (AHEAD) counts as a wall to let go before.
//!
//! Round 84: Comet and Apex plan: they fly the best few pairs out (sim_pair) two / three pairs deep (plan) and take the
//! line that gets them there soonest, and re-plan every REPLAN ticks, switching when a line beats flying out the pair
//! they hold; from Stormcutter up, where no cable reaches, gas buys an air dash straight on (air_dash). The mantle
//! streams behind him (stream fx, sheet 'levi_cape') while he flies in his form or dashes, and hangs at rest (lv_skin).
//!
//! Who plays him matters, like Scribble: each athlete has a mastery rank from the games they have played on him
//! (Grounded 0+, Tethered 5+, Swinger 15+, Glider 30+, Skyrunner 60+, Stormcutter 100+, Comet 150+, and Apex: the ten
//! with the most points, 300+ each). Rank sets his starting cable speed (more speed = more reward and more risk), how
//! often a cable misaims, how fast he recovers from a miss or a crash, how well he picks walls, and how he uses it
//! all (escapes, chases, gas, routes, braking). Files (next to the DLL, mods/tfm2_custom_ai/): levi_memory.txt (G
//! lines), levi_pending.txt (g / r lines, merged at the next launch), levi_history.txt. No spell meta.

use crate::scribble::Memory;
use crate::{champions, d2, sq, timed, walls, Champ, MOD_ID};
use mod_api_stable::{AttackTypeV1, BuffV1, CcKindV1, CcV1, SimOriginV1, StablePassive, StableSim};
use std::collections::{HashMap, HashSet, VecDeque};
use std::io::Write;
use std::sync::{Arc, Mutex, OnceLock};

pub const RANK_NAMES: [&str; 8] = ["Grounded", "Tethered", "Swinger", "Glider", "Skyrunner", "Stormcutter", "Comet", "Apex"];
const APEX: usize = 7;

/// Starting cable speed per rank (units a tick; a champion walks about 1000). Apex goes from 4300 (#10) to 4800 (#1).
const BASE_SPEED: [f64; 8] = [2200.0, 2500.0, 2800.0, 3100.0, 3400.0, 3700.0, 4000.0, 4300.0];
const APEX_TOP_SPEED: f64 = 4800.0;
/// Chance (percent) a cable veers 10-40 degrees off the wall he meant.
const MISAIM: [u64; 8] = [25, 16, 10, 6, 3, 1, 0, 0];
/// Ticks before he can cable again after a miss or a crash.
const RECOVER: [usize; 8] = [60, 45, 36, 27, 18, 12, 9, 8];
/// He fires the next cable once the current one is this many ticks from biting its wall (0 = whenever the button
/// comes up: a Grounded player doesn't time it).
const LOOKAHEAD: [f64; 8] = [0.0, 6.0, 10.0, 14.0, 18.0, 24.0, 30.0, 30.0];
/// Round 80 (the Fanny pass): how far ahead (ticks) he reads a cable's flight path for walls before picking it.
const READ: [usize; 8] = [0, 0, 10, 16, 22, 30, 36, 40];
/// How many ticks his next-cable timing is off, either way (a late cable is a slam).
const JITTER: [f64; 8] = [0.0, 6.0, 4.0, 3.0, 2.0, 1.0, 0.0, 0.0];
/// How often (percent, rolled as each cable bites) he reads the landing and brakes into it softly when no cable can
/// save the flight.
const BRAKE: [u64; 8] = [0, 25, 50, 75, 100, 100, 100, 100];
/// Cables fire all the way round: one that bites behind him (past TURNBACK) whips him round to it at once, losing a
/// share of his speed, and lets go of the other cable.
const TURNBACK: f64 = 100.0 * std::f64::consts::PI / 180.0;
const TURNBACK_KEEP: f64 = 0.75;
/// The clearance he wants beside a cable's path when he reads it.
const CLEAR: f64 = 6_000.0;
/// The path checks look at every this many units of flight (fine enough not to step over a wall's corner at speed).
const PROBE: f64 = 2_500.0;
/// A press in the air holds the cable button this long; he fires when the moment is right, at most every MIN_GAP.
const HOLD_T: usize = 30;
const MIN_GAP: usize = 6;
/// A destination worth a flight (by the walking path), and close enough that he's there.
const FLY_FROM: f64 = 90_000.0;
const ARRIVED: f64 = 25_000.0;
/// How long the game's move order (seen by the input AI) stays his destination.
const DEST_FRESH: usize = 60;
/// Round 81 (between the walls): with two cables, the newest one's share of the pull; at 0.5 he flies down the middle
/// between his two anchors instead of into the newest one's wall.
const NEWEST_W: [f64; 8] = [0.8, 0.75, 0.65, 0.6, 0.55, 0.5, 0.5, 0.5];
/// From Swinger up he fires the next cable as he passes his newest anchor (it's this far off his heading), and pairs a
/// lone cable at once, so he swings past walls instead of reaching them.
const PASS: f64 = 70.0 * std::f64::consts::PI / 180.0;
/// A cable this far behind him lets go (it would only pull him back).
const RELEASE: f64 = 100.0 * std::f64::consts::PI / 180.0;
/// From Tethered up a cable under this length isn't worth it (and he never takes off on one).
const SHORT: f64 = 30_000.0;
/// He lets go this many ticks before reaching his anchor's wall when nothing can carry the flight on (if he reads it),
/// and drops onto open ground.
const LETGO: f64 = 8.0;
/// Round 82 (the pair, Rian's drawing): from Swinger up he fires his cables two at a time, one to each side of where
/// he's going (PAIR_MIN-PAIR_MAX degrees off it), and flies the diagonal between them, never at either anchor; as he
/// passes the pair (both anchors PAIR_PASS off his line) he fires the next pair ahead.
const PAIR_FROM: usize = 2;
const PAIR_MIN: f64 = 20.0;
const PAIR_MAX: f64 = 80.0;
const PAIR_PASS: f64 = 75.0 * std::f64::consts::PI / 180.0;
/// And a lone cable he fires in the air goes at least this far off his line, so he swings on it, never at it.
const SIDE: f64 = 45.0 * std::f64::consts::PI / 180.0;
/// Round 83: how fast he can swing round (radians a tick) by rank: the top ranks react at once to a new direction.
const TURN_R: [f64; 8] = [0.15, 0.18, 0.22, 0.27, 0.33, 0.4, 0.48, 0.55];
/// From Swinger up, a new direction this far from the one his pair was fired for gets a new pair at once.
const REAIM: f64 = 40.0 * std::f64::consts::PI / 180.0;
/// Round 84: Comet looks 2 pairs ahead and Apex 3, trying the best few pairs at each step (PLAN_K) by flying them out
/// (up to PLAN_T ticks each), and takes the first pair of the line that gets him there soonest; every REPLAN ticks he
/// looks again and switches when another line is clearly faster than flying out the pair he holds.
const PLAN_DEPTH: [usize; 8] = [0, 0, 0, 0, 0, 0, 2, 3];
const PLAN_K: [usize; 8] = [0, 0, 0, 0, 0, 0, 4, 7];
/// Past the first step the planner flies out only this many of the best-looking pairs.
const PLAN_K_DEEP: usize = 3;
const PLAN_T: usize = 45;
const REPLAN: usize = 6;
/// From Stormcutter up, where no cable will do (open ground) he spends gas on an air dash toward where he's going:
/// straight there at 1.5 x DASH_SPEED or more for DASH_T ticks, if the way is clear; at most every AIR_DASH_CD.
const AIR_DASH_FROM: usize = 5;
const AIR_DASH_CD: usize = 60;
/// Long cables are drawn as a chain of segments of at most this many pixels (the longest cable sprite).
const SEG_PX: f64 = 96.0;
/// An anchor counts as a wall coming at him (for letting go) only when it's within this of his line.
const AHEAD: f64 = 60.0 * std::f64::consts::PI / 180.0;

/// A cable he could fire: its angle and where it bites.
type Shot = (f64, (i64, i64));
/// A flight state the planner flies forward: position, heading, speed, whether he's on cables yet.
#[derive(Clone, Copy)]
struct Fly { x: f64, y: f64, h: f64, sp: f64, in_air: bool }

/// His walking distance to `dest`'s field from (x, y), smooth inside a cell.
fn path_len(f: &[f32], x: f64, y: f64) -> f64 {
    let n = walls::N;
    let (cx, cy) = ((x as i64).div_euclid(walls::CELL), (y as i64).div_euclid(walls::CELL));
    let mut best = f64::INFINITY;
    for dy in -1..=1 {
        for dx in -1..=1 {
            let (nx, ny) = (cx + dx, cy + dy);
            if nx < 0 || ny < 0 || nx >= n || ny >= n { continue; }
            let v = f[(ny * n + nx) as usize];
            if !v.is_finite() { continue; }
            let c = walls::CELL as f64;
            best = best.min(v as f64 * c + ((nx as f64 + 0.5) * c - x).hypot((ny as f64 + 0.5) * c - y));
        }
    }
    best
}

/// A cable seen from where he is: its angle, where it bites, how long it is.
type Seen = (f64, (i64, i64), f64);

const CABLE_RANGE: i64 = 200_000;
const CABLE_MIN: i64 = 12_000;
const FLIGHT_T: usize = 90;
const ANGLE_FLOOR: f64 = 0.25;
const GAIN: (f64, f64) = (250.0, 550.0);
/// Not a balance cap: past this a tick's move would skip whole wall cells.
const SPEED_CEIL: f64 = 30_000.0;
const STEP: f64 = 6_000.0;
const CRASH_SPEED: f64 = 3_500.0;
const LAND_R: i64 = 9_000;
const FAST: f64 = 4_500.0;

const GAS_MAX: i32 = 100;
const BOOST_COST: i32 = 8;
const BOOST_ADD: f64 = 600.0;
const DASH_COST: i32 = 10;
const DASH_CD: usize = 180;
const DASH_T: usize = 10;
const DASH_SPEED: f64 = 4_000.0;
const HOME_R: i64 = 45_000;

const RAMPAGE_T: usize = 480;
const CUT_R: i64 = 12_000;
const CUT_LOCK: usize = 60;
const CUT_DMG: (usize, usize) = (15, 25);
const RAMPAGE_R: i64 = 20_000;
const RAMPAGE_LOCK: usize = 20;
const RAMPAGE_DMG: (usize, usize) = (30, 45);
/// Damage scales with speed / 2600, at most this much.
const SPEED_DMG_CAP: f64 = 4.0;

fn deg(a: f64) -> f64 {
    a.to_radians()
}

fn wrap(a: f64) -> f64 {
    let mut a = a % (2.0 * std::f64::consts::PI);
    if a > std::f64::consts::PI { a -= 2.0 * std::f64::consts::PI; }
    if a < -std::f64::consts::PI { a += 2.0 * std::f64::consts::PI; }
    a
}

fn ang_to(ax: f64, ay: f64, bx: f64, by: f64) -> f64 {
    (by - ay).atan2(bx - ax)
}

/// How far the first wall is along heading h, within `reach`.
fn wall_ahead(x: f64, y: f64, h: f64, reach: f64) -> Option<f64> {
    let mut d = PROBE;
    while d <= reach {
        if walls::wall_at((x + h.cos() * d) as i64, (y + h.sin() * d) as i64) { return Some(d); }
        d += PROBE;
    }
    None
}

/// Where his cables steer him: toward a lone anchor, or between two (the newest weighted by rank).
fn pull(x: f64, y: f64, cables: &[(f64, f64)], r: usize) -> f64 {
    let b = cables[cables.len() - 1];
    let tb = ang_to(x, y, b.0, b.1);
    if cables.len() < 2 { return tb; }
    let ta = ang_to(x, y, cables[0].0, cables[0].1);
    let w = NEWEST_W[r.min(APEX)];
    (w * tb.sin() + (1.0 - w) * ta.sin()).atan2(w * tb.cos() + (1.0 - w) * ta.cos())
}

/// The angle quality of a cable pair: 1 at 90°, 0 at 0° and 180°, never under the floor.
pub fn angle_quality(a: f64, b: f64) -> f64 {
    let pair = wrap(b - a).abs().to_degrees();
    (1.0 - (pair - 90.0).abs() / 90.0).max(ANGLE_FLOOR)
}

pub fn base_speed(rank: usize, apex: Option<usize>) -> f64 {
    if rank >= APEX {
        let p = apex.unwrap_or(10).clamp(1, 10) as f64;
        return APEX_TOP_SPEED - (p - 1.0) * (APEX_TOP_SPEED - BASE_SPEED[APEX]) / 9.0;
    }
    BASE_SPEED[rank.min(APEX)]
}

/// Where a cable shot from (x, y) at angle `a` bites: the last free point before the first wall (or a tower close to
/// its line), between CABLE_MIN and CABLE_RANGE away.
fn raycast(x: f64, y: f64, a: f64, towers: &[(i64, i64)]) -> Option<(i64, i64)> {
    let (c, s) = (a.cos(), a.sin());
    let mut hit: Option<(i64, i64, f64)> = None;
    let mut t = 3_000.0;
    let mut last = (x as i64, y as i64);
    while t <= CABLE_RANGE as f64 {
        let (px, py) = ((x + c * t) as i64, (y + s * t) as i64);
        if walls::wall_at(px, py) {
            if t >= CABLE_MIN as f64 { hit = Some((last.0, last.1, t)); }
            break;
        }
        last = (px, py);
        t += 3_000.0;
    }
    // a tower near the line, before the wall, can be bitten too
    let limit = hit.map_or(CABLE_RANGE as f64, |h| h.2);
    let mut best: Option<(i64, i64, f64)> = None;
    for &(tx, ty) in towers {
        let (dx, dy) = (tx as f64 - x, ty as f64 - y);
        let along = dx * c + dy * s;
        let off = (dx * s - dy * c).abs();
        if along >= CABLE_MIN as f64 && along < limit && off <= 7_000.0 && best.map_or(true, |b| along < b.2) {
            best = Some((tx, ty, along));
        }
    }
    best.or(hit).map(|h| (h.0, h.1))
}

const NB: [(i64, i64); 8] = [(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (1, -1), (-1, 1), (-1, -1)];

fn cell_wall(cx: i64, cy: i64) -> bool {
    walls::wall_at(cx * walls::CELL + walls::CELL / 2, cy * walls::CELL + walls::CELL / 2)
}

/// The walking distance (in cells) from every cell to `dest`'s, around the walls (8 ways, no corner cutting).
fn field(dest: (i64, i64)) -> Vec<f32> {
    let n = walls::N;
    let mut d = vec![f32::INFINITY; (n * n) as usize];
    let (sx, sy) = ((dest.0 / walls::CELL).clamp(0, n - 1), (dest.1 / walls::CELL).clamp(0, n - 1));
    d[(sy * n + sx) as usize] = 0.0;
    let mut q = VecDeque::from([(sx, sy)]);
    while let Some((x, y)) = q.pop_front() {
        let here = d[(y * n + x) as usize];
        for (dx, dy) in NB {
            let (nx, ny) = (x + dx, y + dy);
            if nx < 0 || ny < 0 || nx >= n || ny >= n || cell_wall(nx, ny) { continue; }
            if dx != 0 && dy != 0 && (cell_wall(nx, y) || cell_wall(x, ny)) { continue; }
            let nd = here + if dx != 0 && dy != 0 { 1.414 } else { 1.0 };
            if nd < d[(ny * n + nx) as usize] {
                d[(ny * n + nx) as usize] = nd;
                q.push_back((nx, ny));
            }
        }
    }
    d
}

/// Where he heads for on the way to `dest`: the farthest point along the walking path (downhill in the field) that he
/// can see in a straight line; and the path's length.
fn aim_point(f: &[f32], x: f64, y: f64, dest: (i64, i64)) -> (f64, f64, f64) {
    let n = walls::N;
    let straight = (dest.0 as f64 - x).hypot(dest.1 as f64 - y);
    if walls::clip(x as i64, y as i64, dest.0, dest.1) == dest { return (dest.0 as f64, dest.1 as f64, straight); }
    let (mut cx, mut cy) = (((x as i64) / walls::CELL).clamp(0, n - 1), ((y as i64) / walls::CELL).clamp(0, n - 1));
    let at = |cx: i64, cy: i64| f[(cy * n + cx) as usize];
    let path = if at(cx, cy).is_finite() { at(cx, cy) as f64 * walls::CELL as f64 } else { straight };
    let mut best: Option<(f64, f64)> = None;
    for _ in 0..24 {
        let mut nb: Option<(f32, i64, i64)> = None;
        for (dx, dy) in NB {
            let (nx, ny) = (cx + dx, cy + dy);
            if nx < 0 || ny < 0 || nx >= n || ny >= n || cell_wall(nx, ny) { continue; }
            if dx != 0 && dy != 0 && (cell_wall(nx, cy) || cell_wall(cx, ny)) { continue; }
            if at(nx, ny) < at(cx, cy) && nb.is_none_or(|b| at(nx, ny) < b.0) { nb = Some((at(nx, ny), nx, ny)); }
        }
        let Some((_, nx, ny)) = nb else { break };
        (cx, cy) = (nx, ny);
        let (px, py) = (cx * walls::CELL + walls::CELL / 2, cy * walls::CELL + walls::CELL / 2);
        if walls::clip(x as i64, y as i64, px, py) != (px, py) {
            if best.is_some() { break; }
            continue;
        }
        best = Some((px as f64, py as f64));
        if at(cx, cy) == 0.0 { break; }
    }
    let (ax, ay) = best.unwrap_or((dest.0 as f64, dest.1 as f64));
    (ax, ay, path)
}

// ------------------------------------------------------------------ mastery (per athlete, no spell meta)

fn mod_dir() -> Option<std::path::PathBuf> {
    std::env::current_exe().ok().and_then(|e| e.parent().map(|d| d.join("mods").join(MOD_ID)))
}

static MEMORY: OnceLock<Memory> = OnceLock::new();

fn memory() -> &'static Memory {
    MEMORY.get_or_init(|| {
        let Some(dir) = mod_dir() else { return Memory::default() };
        let mem_p = dir.join("levi_memory.txt");
        let pend_p = dir.join("levi_pending.txt");
        let mut m = Memory::parse(&std::fs::read_to_string(&mem_p).unwrap_or_default());
        if let Ok(p) = std::fs::read_to_string(&pend_p) {
            if !p.trim().is_empty() {
                m.merge(&p);
                m.meta.clear();
                if std::fs::write(&mem_p, m.render()).is_ok() {
                    let _ = std::fs::OpenOptions::new().create(true).append(true).open(dir.join("levi_history.txt"))
                        .and_then(|mut f| f.write_all(p.as_bytes()));
                    let _ = std::fs::write(&pend_p, "");
                }
            }
        }
        m
    })
}

struct Session {
    lines: Vec<String>,
    pins: HashMap<u64, Arc<Memory>>,
    order: VecDeque<u64>,
    written: HashSet<String>,
}
static SESSION: Mutex<Option<Session>> = Mutex::new(None);

fn with_session<R>(f: impl FnOnce(&mut Session) -> R) -> Option<R> {
    let mut g = SESSION.lock().ok()?;
    let s = g.get_or_insert_with(|| Session { lines: Vec::new(), pins: HashMap::new(), order: VecDeque::new(), written: HashSet::new() });
    Some(f(s))
}

/// The memory this match plays with: the launch memory plus the games of this launch, pinned by the match's seed so
/// the server's precomputed simulation and the live one agree.
fn pinned(seed: u64) -> Arc<Memory> {
    let base = memory();
    with_session(|s| {
        if let Some(m) = s.pins.get(&seed) { return m.clone(); }
        let mut m = base.clone();
        if !s.lines.is_empty() { m.merge(&s.lines.join("\n")); }
        let m = Arc::new(m);
        s.pins.insert(seed, m.clone());
        s.order.push_back(seed);
        while s.order.len() > 256 {
            if let Some(o) = s.order.pop_front() { s.pins.remove(&o); }
        }
        m
    })
    .unwrap_or_else(|| Arc::new(base.clone()))
}

fn emit(lines: Vec<String>) {
    let fresh: Vec<String> = with_session(|s| {
        if s.written.len() > 200_000 { s.written.clear(); }
        let fresh: Vec<String> = lines.into_iter().filter(|l| s.written.insert(l.clone())).collect();
        s.lines.extend(fresh.iter().cloned());
        fresh
    })
    .unwrap_or_default();
    if fresh.is_empty() { return; }
    let Some(dir) = mod_dir() else { return };
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(dir.join("levi_pending.txt")) {
        let _ = f.write_all((fresh.join("\n") + "\n").as_bytes());
    }
}

static ATHLETES: Mutex<Option<HashMap<(u64, usize), usize>>> = Mutex::new(None);

/// Called by the input AI, the one place the game says which athlete plays a champion.
pub fn note_athlete(seed: u64, player: usize, athlete: usize) {
    if let Ok(mut g) = ATHLETES.lock() {
        let m = g.get_or_insert_with(HashMap::new);
        if m.len() > 4096 { m.clear(); }
        m.insert((seed, player), athlete);
    }
}

/// (seed, player) -> the last move order (x, y, tick)
type Dests = HashMap<(u64, usize), (i64, i64, usize)>;
static DESTS: Mutex<Option<Dests>> = Mutex::new(None);

/// Called by the input AI with the game's move orders: where the AI is walking him (his flights head there).
pub fn note_dest(seed: u64, player: usize, x: i64, y: i64, tick: usize) {
    if let Ok(mut g) = DESTS.lock() {
        let m = g.get_or_insert_with(HashMap::new);
        if m.len() > 4096 { m.clear(); }
        m.insert((seed, player), (x, y, tick));
    }
}

fn dest_of(seed: u64, player: usize) -> Option<(i64, i64, usize)> {
    DESTS.lock().ok().and_then(|g| g.as_ref().and_then(|m| m.get(&(seed, player)).copied()))
}

fn athlete_of(seed: u64, player: usize) -> Option<usize> {
    ATHLETES.lock().ok().and_then(|g| g.as_ref().and_then(|m| m.get(&(seed, player)).copied()))
}

fn nexus_pos(team: usize) -> (i64, i64) {
    if team == 0 { (96_000, 864_000) } else { (864_000, 96_000) }
}

// ------------------------------------------------------------------ the passive

#[derive(Clone, Default)]
pub struct Levi {
    started: bool,
    rng: u64,
    athlete: Option<usize>,
    rank: Option<usize>,
    apex: Option<usize>,
    sig: Option<String>,
    team: Option<usize>,
    last_result: Option<(i64, i64, i64, i64, i64)>,
    result_at: usize,
    home: Option<(i64, i64)>,
    was_alive: bool,
    // flight
    flying: bool,
    pos: (f64, f64),
    heading: f64,
    speed: f64,
    cables: Vec<(i64, i64)>,
    chain: usize,
    flight_until: usize,
    gliding: bool,
    recover_until: usize,
    last_cable: usize,
    goal: Option<f64>,
    // gas
    gas: i32,
    dash_cd: usize,
    dash: Option<(f64, usize)>,
    // ult
    rampage_until: usize,
    cut: HashMap<usize, usize>,
    // where he has been (movement direction, afterimages)
    track: VecDeque<(usize, i64, i64)>,
    shown: (Option<usize>, Option<i32>, Option<(usize, Option<usize>)>),
    // the form he is shown in (2 Stormcutter, 3 Comet, 4 Apex, 0 none) and where the last trail copy went
    form: usize,
    last_trail: Option<(f64, f64)>,
    me: Option<usize>,
    // round 80: the held cable button, his timing error, whether he reads this cable's landing; where the game is
    // walking him and the walking distances to it
    held_until: usize,
    late: f64,
    reads: bool,
    drop: bool,
    pair_goal: Option<f64>,
    skin: usize,
    replan_at: usize,
    air_dash: usize,
    air_cd: usize,
    dest: Option<(i64, i64)>,
    field: Option<((i64, i64), Vec<f32>)>,
}

impl Levi {
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
    fn fx(&self, m: &Champ, tag: &str) -> String {
        format!("{}_{tag}", m.name)
    }

    fn latch_rank(&mut self, sim: &StableSim<'_>, player: usize) {
        let athlete = athlete_of(sim.seed(), player);
        let mem = pinned(sim.seed());
        let (rank, apex) = mem.rank_for(athlete);
        self.athlete = athlete;
        self.rank = Some(rank);
        self.apex = apex;
    }

    fn make_sig(&mut self, sim: &StableSim<'_>, all: &[Champ]) {
        let origin = sim.sim_origin().unwrap_or_default();
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        let mut eat = |v: u64| {
            for b in v.to_le_bytes() {
                h ^= b as u64;
                h = h.wrapping_mul(0x100_0000_01b3);
            }
        };
        eat(sim.seed());
        let mut v: Vec<&Champ> = all.iter().collect();
        v.sort_by_key(|c| c.id);
        for c in v {
            eat(c.id as u64);
            eat(c.x as u64);
            eat(c.y as u64);
            for b in c.name.bytes() { eat(b as u64); }
        }
        let mid = if origin.match_id == SimOriginV1::NONE { "x".to_string() } else { format!("{}", origin.match_id) };
        let sig = format!("{mid}.{h:x}");
        if let Some(a) = self.athlete { emit(vec![format!("g {sig} {a}")]); }
        // one line per game for reading (who played him, at what rank, how fast he starts)
        let r = self.rank();
        let label = match (r, self.apex) { (APEX, Some(p)) => format!("Apex #{p}"), _ => RANK_NAMES[r.min(APEX)].to_string() };
        let line = format!("game {sig} athlete {} rank {r} ({label}), cable speed {:.0}\n",
            self.athlete.map_or("?".to_string(), |a| a.to_string()), base_speed(r, self.apex));
        let first = with_session(|s| s.written.insert(format!("log {sig}"))).unwrap_or(false);
        if first {
            if let Some(dir) = mod_dir() {
                if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(dir.join("levi_log.txt")) { let _ = f.write_all(line.as_bytes()); }
            }
        }
        self.sig = Some(sig);
    }

    /// The structure record of the game (towers, nexus, score) for the win check, like Scribble's.
    fn track_result(&mut self, sim: &StableSim<'_>, tick: usize) {
        let (Some(sig), Some(team), Some(a)) = (self.sig.clone(), self.team, self.athlete) else { return };
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
            emit(vec![format!("r {sig} {a} {tick} {} {} {} {} {}", rec.0, rec.1, rec.2, rec.3, rec.4)]);
        }
    }

    /// Where he wants to go now (an angle), whether it's worth a flight from the ground, and how far it is by the
    /// walking path (infinite when he's chasing or running: no stopping short).
    fn want(&self, m: &Champ, all: &[Champ], sim: &StableSim<'_>) -> Option<(f64, bool, f64)> {
        let r = self.rank();
        let (x, y) = if self.flying { self.pos } else { (m.x as f64, m.y as f64) };
        let foes: Vec<&Champ> = all.iter().filter(|c| c.team != m.team && sim.is_visible(m.team, c.id)).collect();
        let near = |rr: i64| foes.iter().filter(|f| d2(f.x, f.y, m.x, m.y) <= sq(rr)).min_by_key(|f| d2(f.x, f.y, m.x, m.y)).copied();
        let low = m.hp * 100 <= m.max_hp * 35;
        // escape on a cable (Tethered and up)
        if r >= 1 && low {
            if let Some(f) = near(60_000) {
                let away = ang_to(f.x as f64, f.y as f64, x, y);
                let home = self.home.map_or(away, |h| ang_to(x, y, h.0 as f64, h.1 as f64));
                let a = (away.sin() + home.sin()).atan2(away.cos() + home.cos());
                return Some((a, true, f64::INFINITY));
            }
        }
        // chase on a cable (Swinger and up): a weak enemy in reach, or the nearest one when his team is fighting
        if r >= 2 {
            let mates = all.iter().filter(|c| c.team == m.team && c.id != m.id && d2(c.x, c.y, m.x, m.y) <= sq(80_000)).count();
            let target = foes.iter().filter(|f| d2(f.x, f.y, m.x, m.y) <= sq(150_000) && f.hp * 100 <= f.max_hp * 45)
                .min_by_key(|f| f.hp).copied()
                .or_else(|| if mates > 0 { near(90_000) } else { None });
            if let Some(t) = target {
                return Some((ang_to(x, y, t.x as f64, t.y as f64), true, f64::INFINITY));
            }
        }
        // round 80: where the game is walking him (its move order, seen by the input AI), along the walking path
        if let (Some(dest), Some((_, f))) = (self.dest, self.field.as_ref()) {
            let d = (dest.0 as f64 - x).hypot(dest.1 as f64 - y);
            if d < ARRIVED { return None; }
            let (ax, ay, path) = aim_point(f, x, y, dest);
            return Some((ang_to(x, y, ax, ay), path >= FLY_FROM, path));
        }
        // otherwise the way he has been walking: a steady walk is worth a flight
        let now = self.track.back()?;
        let then = self.track.iter().find(|p| p.0 + 30 >= now.0)?;
        let (dx, dy) = ((now.1 - then.1) as f64, (now.2 - then.2) as f64);
        let walked = dx.hypot(dy);
        if walked < 4_000.0 { return None; }
        Some((dy.atan2(dx), walked >= 20_000.0, f64::INFINITY))
    }

    /// Pick the wall to cable to, by rank.
    fn pick_anchor(&mut self, want: f64, towers: &[(i64, i64)]) -> Option<(f64, (i64, i64))> {
        let r = self.rank();
        let (x, y) = self.pos;
        let older = self.cables.last().map(|c| ang_to(x, y, c.0 as f64, c.1 as f64));
        let mut best: Option<(f64, f64, (i64, i64))> = None;
        for k in -9i32..=9 {
            let a = want + deg(10.0 * k as f64);
            let Some(p) = raycast(x, y, a, towers) else { continue };
            let dist = ((p.0 as f64 - x).hypot(p.1 as f64 - y)).max(1.0);
            let kept = if self.flying && !self.gliding { self.cables.last().map(|c| (c.0 as f64, c.1 as f64)) } else { None };
            let mut score = -(k.abs() as f64) * if kept.is_some() && r >= 2 { 4.0 } else { 10.0 };
            if let (Some(kc), true) = (kept, r >= 2) {
                // the pair he'd hold: where it steers him, and whether the two walls are either side of his line
                let pd = pull(x, y, &[kc, (p.0 as f64, p.1 as f64)], r);
                score -= wrap(pd - want).abs().to_degrees() * 0.6;
                let (sk, sq) = (wrap(ang_to(x, y, kc.0, kc.1) - self.heading).signum(), wrap(a - self.heading).signum());
                if sk != sq { score += 20.0; }
            }
            if r >= 1 && self.flying && dist < self.speed * 8.0 { score -= 40.0; }        // too close at this speed
            if r >= 2 {
                if let Some(o) = older { score += angle_quality(o, a) * if r >= 3 { 50.0 } else { 30.0 }; }
            }
            if r >= 4 {
                // two cables ahead: is there a wall to carry on from that one, toward where he is going?
                let (mx, my) = (x + (p.0 as f64 - x) * 0.7, y + (p.1 as f64 - y) * 0.7);
                if [-0.6f64, 0.0, 0.6].iter().any(|o| raycast(mx, my, want + o, towers).is_some()) { score += 15.0; }
            }
            if r >= 5 && self.flying && dist < self.speed * 10.0 { score -= 80.0; }        // reads a crash coming
            if r >= 3 && self.flying { score -= wrap(a - self.heading).abs().to_degrees() * 0.3; }   // keeps his line
            if self.flying && wrap(a - self.heading).abs() > TURNBACK {
                // a turnback costs speed: only when where he's going is behind him
                if wrap(want - self.heading).abs() < std::f64::consts::FRAC_PI_2 { continue; }
                score -= 40.0;
            }
            if r >= 1 && dist < SHORT { score -= 60.0; }                                               // too short to be worth it
            if r >= 1 && !self.flying && (k.abs() > 6 || dist < SHORT) { continue; }                   // takes off only toward walls ahead
            if r >= PAIR_FROM && self.flying && wrap(a - self.heading).abs() < SIDE { continue; }      // a lone cable: to swing on, never at
            if READ[r] > 0 && self.flying && self.path_hits(p, READ[r]) { score -= 300.0; }          // reads the path for walls
            if best.map_or(true, |b| score > b.0) { best = Some((score, a, p)); }
        }
        // every path ends in a wall: hold the cable
        if READ[r] > 0 && self.flying && best.is_some_and(|b| b.0 < -150.0) { return None; }
        best.map(|b| (b.1, b.2))
    }

    /// Fly the steering of the cable pair (the newest, p) for up to `ticks` and see whether a wall other than p's
    /// comes first; a path that grazes a wall (within CLEAR) counts. Arriving at p is fine: that's the next cable's job.
    fn path_hits(&self, p: (i64, i64), ticks: usize) -> bool {
        let (mut x, mut y) = self.pos;
        let mut h = self.heading;
        let (px, py) = (p.0 as f64, p.1 as f64);
        let mut old = self.cables.last().map(|c| (c.0 as f64, c.1 as f64));
        let mut sp = self.speed + old.map_or(0.0, |o| GAIN.0 + GAIN.1 * angle_quality(ang_to(x, y, o.0, o.1), ang_to(x, y, px, py)));
        if wrap(ang_to(x, y, px, py) - h).abs() > TURNBACK {
            h = ang_to(x, y, px, py);
            sp *= TURNBACK_KEEP;
            old = None;
        }
        sp = sp.min(SPEED_CEIL);
        let near = |x: f64, y: f64| (x - px).powi(2) + (y - py).powi(2);
        let r = self.rank();
        for _ in 0..ticks {
            let target = match old { Some(o) => pull(x, y, &[o, (px, py)], r), None => ang_to(x, y, px, py) };
            h = wrap(h + wrap(target - h).clamp(-TURN_R[r], TURN_R[r]));
            let n = (sp / PROBE).ceil().max(1.0) as usize;
            let per = sp / n as f64;
            let (nx, ny) = (-h.sin() * CLEAR, h.cos() * CLEAR);
            for _ in 0..n {
                x += h.cos() * per;
                y += h.sin() * per;
                let dp = near(x, y);
                if dp <= (LAND_R as f64).powi(2) { return false; }
                if walls::wall_at(x as i64, y as i64) { return true; }
                if dp > (3.0 * LAND_R as f64).powi(2)
                    && (walls::wall_at((x + nx) as i64, (y + ny) as i64) || walls::wall_at((x - nx) as i64, (y - ny) as i64)) {
                    return true;
                }
            }
        }
        false
    }

    /// One S1 press. On the ground: take off if it's worth it. In the air: Grounded fires the moment the button is
    /// up; everyone else holds it for the right moment (maybe_fire).
    fn on_cable_press(&mut self, sim: &mut StableSim<'_>, m: &Champ, all: &[Champ], tick: usize) {
        if tick < self.recover_until || self.dash.is_some() { return; }
        let r = self.rank();
        if self.flying {
            if r == 0 {
                if tick >= self.last_cable + 18 {
                    if let Some(w) = self.want(m, all, sim) { self.goal = Some(w.0); }
                    self.fire(sim, m, tick);
                }
                return;
            }
            if self.held_until <= tick { self.late = (self.unit() * 2.0 - 1.0) * JITTER[r]; }
            self.held_until = tick + HOLD_T;
            return;
        }
        let Some((a, go, _)) = self.want(m, all, sim) else { return };
        if !go { return; }
        self.goal = Some(a);
        self.pos = (m.x as f64, m.y as f64);
        let towers = Self::towers(sim);
        match self.pick_pair(a, &towers) {
            Some(pair) => self.fire_pair(sim, m, tick, pair, &towers),
            None => { self.fire(sim, m, tick); }
        }
    }

    /// The held button, every tick in the air: fire when the current cable is about to bite (his timing, off by
    /// `late`) or at once when he has lost his cables; never into a wall's path or toward nothing; and not within a
    /// cable's reach of where he's going (he lands and walks the rest).
    fn maybe_fire(&mut self, sim: &mut StableSim<'_>, m: &Champ, all: &[Champ], tick: usize) {
        let r = self.rank();
        if !self.flying || r == 0 || tick >= self.held_until || tick < self.recover_until || tick < self.last_cable + MIN_GAP { return; }
        let newest = self.cables.last().copied();
        let tta = newest.map_or(0.0, |c| self.tta_of(c));
        // within a cable's reach of where he's going: stop chaining and come down (he walks the rest)
        let w = self.want(m, all, sim);
        let Some((a, _, _)) = w.filter(|w| w.2 >= (2.0 * ARRIVED).max(self.speed * 8.0)) else {
            if self.reads && newest.is_some() && tta < LETGO { self.let_go(); }
            return;
        };
        self.goal = Some(a);
        if r >= PAIR_FROM {
            // his pair: the next one when he's passing it (his timing makes that early or late), when he's about to
            // reach an anchor, or at once when he's lost his cables; one cable only if no pair will do
            let pass = PAIR_PASS + deg(3.0) * self.late;
            let (x, y, h) = (self.pos.0, self.pos.1, self.heading);
            let passing = self.cables.len() < 2 || self.cables.iter().all(|q| wrap(ang_to(x, y, q.0 as f64, q.1 as f64) - h).abs() > pass);
            let close = newest.is_some() && tta <= (LOOKAHEAD[r] * 0.5 - self.late).max(3.0);
            let turned = self.pair_goal.is_some_and(|g| wrap(a - g).abs() > REAIM);
            // the planners look again every REPLAN ticks: a line clearly faster than flying out the pair he holds
            if PLAN_DEPTH[r] > 0 && self.cables.len() == 2 && !self.gliding && tick >= self.replan_at {
                self.replan_at = tick + REPLAN;
                if let (Some(dest), Some((_, f))) = (self.dest, self.field.as_ref()) {
                    let towers = Self::towers(sim);
                    let st0 = Fly { x: self.pos.0, y: self.pos.1, h: self.heading, sp: self.speed, in_air: true };
                    let (c0, c1) = (self.cables[0], self.cables[1]);
                    let (e, t, hit) = self.sim_pair(st0, (c0.0 as f64, c0.1 as f64), (c1.0 as f64, c1.1 as f64), PLAN_T, true);
                    let keep = if hit { f64::INFINITY } else {
                        let left = path_len(f, e.x, e.y);
                        let sub = if left > 2.0 * ARRIVED { self.plan(e, PLAN_DEPTH[r] - 1, f, dest, &towers) } else { None };
                        t.max(1) as f64 + sub.map_or(left / e.sp.max(3_000.0), |s_| s_.0)
                    };
                    if let Some((cost, pair)) = self.plan(st0, PLAN_DEPTH[r], f, dest, &towers) {
                        if cost < keep * 0.9 {
                            self.fire_pair(sim, m, tick, pair, &towers);
                            self.late = 0.0;
                            return;
                        }
                    }
                }
            }
            if !self.gliding && newest.is_some() && !passing && !close && !turned { return; }
            let towers = Self::towers(sim);
            if let Some(pair) = self.pick_pair(a, &towers) {
                self.fire_pair(sim, m, tick, pair, &towers);
                self.late = (self.unit() * 2.0 - 1.0) * JITTER[r];
                return;
            }
        }
        // from Swinger up: pair a lone cable at once, and fire the next as he passes the newest anchor
        let swing = r >= 2 && newest.is_some_and(|c| {
            self.cables.len() < 2 || wrap(ang_to(self.pos.0, self.pos.1, c.0 as f64, c.1 as f64) - self.heading).abs() > PASS
        });
        if !self.gliding && newest.is_some() && !swing && tta > LOOKAHEAD[r] - self.late { return; }
        if self.fire(sim, m, tick) { self.late = (self.unit() * 2.0 - 1.0) * JITTER[r]; }
        else if self.air_dash(sim, m, tick) { /* open ground: gas carries him on */ }
        else if self.reads && newest.is_some() && tta < LETGO { self.let_go(); }   // nothing to carry on to: let go before the wall
    }

    /// He lets go of his cables and drops out of the flight onto open ground, short of the wall.
    fn let_go(&mut self) {
        self.cables.clear();
        self.gliding = true;
        self.drop = true;
    }

    /// Ticks until he reaches anchor q, if it's in front of him (infinite when he's passing beside or behind it).
    fn tta_of(&self, q: (i64, i64)) -> f64 {
        let (x, y) = self.pos;
        if wrap(ang_to(x, y, q.0 as f64, q.1 as f64) - self.heading).abs() > AHEAD { return f64::INFINITY; }
        (q.0 as f64 - x).hypot(q.1 as f64 - y) / self.speed.max(1.0)
    }

    /// Fly between the pair (a, b) for up to `ticks` and see whether a wall comes first (with clearance, or too near
    /// an anchor: flying at one is what the pair is meant to avoid); past the pair, the stretch he carries straight
    /// on until the next one bites must be clear too.
    fn pair_hits(&self, a: (f64, f64), b: (f64, f64), ticks: usize) -> bool {
        let r = self.rank();
        let (mut x, mut y) = self.pos;
        let mut h = if self.flying && !self.gliding { self.heading } else { pull(x, y, &[a, b], r) };
        let base = if self.flying { self.speed } else { base_speed(r, self.apex) };
        let sp = (base + GAIN.0 + GAIN.1 * angle_quality(ang_to(x, y, a.0, a.1), ang_to(x, y, b.0, b.1))).min(SPEED_CEIL);
        let far = |px: f64, py: f64, q: (f64, f64)| (px - q.0).powi(2) + (py - q.1).powi(2) > (3.0 * LAND_R as f64).powi(2);
        for _ in 0..ticks {
            if [a, b].iter().all(|q| wrap(ang_to(x, y, q.0, q.1) - h).abs() > PAIR_PASS) {
                let n = (sp * LETGO / 3_000.0).ceil() as usize;
                let (nx, ny) = (-h.sin() * CLEAR, h.cos() * CLEAR);
                return (1..=n).any(|i| {
                    let (px, py) = (x + h.cos() * 3_000.0 * i as f64, y + h.sin() * 3_000.0 * i as f64);
                    walls::wall_at(px as i64, py as i64) || walls::wall_at((px + nx) as i64, (py + ny) as i64)
                        || walls::wall_at((px - nx) as i64, (py - ny) as i64)
                });
            }
            let dp = wrap(pull(x, y, &[a, b], r) - h);
            if dp.abs() <= std::f64::consts::FRAC_PI_2 { h = wrap(h + dp.clamp(-TURN_R[r], TURN_R[r])); }
            let n = (sp / PROBE).ceil().max(1.0) as usize;
            let per = sp / n as f64;
            let (nx, ny) = (-h.sin() * CLEAR, h.cos() * CLEAR);
            for _ in 0..n {
                x += h.cos() * per;
                y += h.sin() * per;
                if !far(x, y, a) || !far(x, y, b) { return true; }
                if walls::wall_at(x as i64, y as i64) || walls::wall_at((x + nx) as i64, (y + ny) as i64)
                    || walls::wall_at((x - nx) as i64, (y - ny) as i64) {
                    return true;
                }
            }
        }
        false
    }

    /// The best two walls either side of where he's going: the pull between them toward it, near a right angle (the
    /// most speed), long cables, a clear path between.
    /// Every pair he could fire from (x, y) flying along h, best-looking first.
    fn pair_candidates(&self, st: Fly, want: f64, towers: &[(i64, i64)]) -> Vec<(f64, Shot, Shot)> {
        let r = self.rank();
        let (x, y) = (st.x, st.y);
        // per side: (angle, where it bites, length)
        let mut side: [Vec<Seen>; 2] = Default::default();
        let mut dg = PAIR_MIN;
        while dg <= PAIR_MAX {
            for (i, sg) in [(0usize, 1.0f64), (1, -1.0)] {
                let a = want + sg * deg(dg);
                if let Some(p) = raycast(x, y, a, towers) {
                    let dist = (p.0 as f64 - x).hypot(p.1 as f64 - y);
                    // in the air both anchors must still be ahead of him (a pair he's already passing carries him nowhere)
                    let behind = st.in_air && wrap(a - st.h).abs() > PAIR_PASS - deg(15.0);
                    if dist >= SHORT && !behind { side[i].push((a, p, dist)); }
                }
            }
            dg += 10.0;
        }
        let mut out = Vec::new();
        for &(aa, pa, da) in &side[0] {
            for &(ab, pb, db) in &side[1] {
                let pd = pull(x, y, &[(pa.0 as f64, pa.1 as f64), (pb.0 as f64, pb.1 as f64)], r);
                let score = -wrap(pd - want).abs().to_degrees() + angle_quality(aa, ab) * 40.0 + da.min(db) / CABLE_RANGE as f64 * 15.0;
                if score >= -40.0 { out.push((score, (aa, pa), (ab, pb))); }
            }
        }
        out.sort_by(|a, b| b.0.total_cmp(&a.0));
        out
    }

    /// The best two walls either side of where he's going: the planners fly the options out; the rest take the
    /// best-looking pair with a clear path.
    fn pick_pair(&self, want: f64, towers: &[(i64, i64)]) -> Option<(Shot, Shot)> {
        let r = self.rank();
        if r < PAIR_FROM { return None; }
        let st = self.fly_state();
        if PLAN_DEPTH[r] > 0 {
            if let (Some(dest), Some((_, f))) = (self.dest, self.field.as_ref()) {
                if let Some((_, pair)) = self.plan(st, PLAN_DEPTH[r], f, dest, towers) { return Some(pair); }
            }
        }
        self.pair_candidates(st, want, towers).into_iter()
            .find(|c| READ[r] == 0 || !self.pair_hits((c.1 .1 .0 as f64, c.1 .1 .1 as f64), (c.2 .1 .0 as f64, c.2 .1 .1 as f64), READ[r]))
            .map(|c| (c.1, c.2))
    }

    fn fly_state(&self) -> Fly {
        let r = self.rank();
        Fly { x: self.pos.0, y: self.pos.1, h: self.heading, sp: if self.flying { self.speed } else { base_speed(r, self.apex) },
              in_air: self.flying && !self.gliding }
    }

    /// Fly between a and b from state st until he's passed them (or `ticks`): the state then, the ticks, and whether a
    /// wall came first. A new pair adds its speed as it bites; the pair he already holds (`held`) adds nothing.
    fn sim_pair(&self, st: Fly, a: (f64, f64), b: (f64, f64), ticks: usize, held: bool) -> (Fly, usize, bool) {
        let r = self.rank();
        let (mut x, mut y) = (st.x, st.y);
        let mut h = if st.in_air { st.h } else { pull(x, y, &[a, b], r) };
        let sp = if held { st.sp } else {
            (st.sp + 2.0 * GAIN.0 + GAIN.1 * (0.6 + angle_quality(ang_to(x, y, a.0, a.1), ang_to(x, y, b.0, b.1)))).min(SPEED_CEIL)
        };
        let far = |px: f64, py: f64, q: (f64, f64)| (px - q.0).powi(2) + (py - q.1).powi(2) > (3.0 * LAND_R as f64).powi(2);
        for k in 0..ticks {
            if [a, b].iter().all(|q| wrap(ang_to(x, y, q.0, q.1) - h).abs() > PAIR_PASS) {
                return (Fly { x, y, h, sp, in_air: true }, k, false);
            }
            let dp = wrap(pull(x, y, &[a, b], r) - h);
            if dp.abs() <= std::f64::consts::FRAC_PI_2 { h = wrap(h + dp.clamp(-TURN_R[r], TURN_R[r])); }
            let n = (sp / PROBE).ceil().max(1.0) as usize;
            let per = sp / n as f64;
            let (nx, ny) = (-h.sin() * CLEAR, h.cos() * CLEAR);
            for _ in 0..n {
                x += h.cos() * per;
                y += h.sin() * per;
                if !far(x, y, a) || !far(x, y, b) || walls::wall_at(x as i64, y as i64)
                    || walls::wall_at((x + nx) as i64, (y + ny) as i64) || walls::wall_at((x - nx) as i64, (y - ny) as i64) {
                    return (Fly { x, y, h, sp, in_air: true }, k, true);
                }
            }
        }
        (Fly { x, y, h, sp, in_air: true }, ticks, false)
    }

    /// The line of pairs (depth deep) that gets him to `dest` soonest: the ticks it takes plus what's left at the speed
    /// it leaves him with. Returns that cost and the line's first pair.
    fn plan(&self, st: Fly, depth: usize, f: &[f32], dest: (i64, i64), towers: &[(i64, i64)]) -> Option<(f64, (Shot, Shot))> {
        let r = self.rank();
        let (ax, ay, _) = aim_point(f, st.x, st.y, dest);
        let want = ang_to(st.x, st.y, ax, ay);
        let mut best: Option<(f64, (Shot, Shot))> = None;
        let k = if depth == PLAN_DEPTH[r] { PLAN_K[r] } else { PLAN_K_DEEP };
        for (_, sa, sb) in self.pair_candidates(st, want, towers).into_iter().take(k) {
            let (e, t, hit) = self.sim_pair(st, (sa.1 .0 as f64, sa.1 .1 as f64), (sb.1 .0 as f64, sb.1 .1 as f64), PLAN_T, false);
            if hit { continue; }
            let left = path_len(f, e.x, e.y);
            let mut cost = t.max(1) as f64 + left / e.sp.max(3_000.0);
            if depth > 1 && left > 2.0 * ARRIVED {
                cost = match self.plan(e, depth - 1, f, dest, towers) {
                    Some((sub, _)) => t.max(1) as f64 + sub,
                    None => cost + 20.0,   // a dead end: he'd have to come down
                };
            }
            if best.is_none_or(|b| cost < b.0) { best = Some((cost, (sa, sb))); }
        }
        best
    }

    /// Where no cable will do, gas straight toward where he's going, if the way is clear (Stormcutter and up).
    fn air_dash(&mut self, sim: &mut StableSim<'_>, m: &Champ, tick: usize) -> bool {
        let Some(goal) = self.goal else { return false };
        if self.rank() < AIR_DASH_FROM || self.gas < DASH_COST || tick < self.air_cd { return false; }
        let sp = self.speed.max(DASH_SPEED * 1.5);
        if wall_ahead(self.pos.0, self.pos.1, goal, sp * (DASH_T + 4) as f64).is_some() { return false; }
        self.gas -= DASH_COST;
        self.air_cd = tick + AIR_DASH_CD;
        self.cables.clear();
        self.gliding = false;
        self.drop = false;
        self.heading = goal;
        self.speed = sp;
        self.air_dash = tick + DASH_T;
        self.flight_until = tick + DASH_T + 20;
        self.last_cable = tick;
        crate::fx_point(sim, &self.fx(m, "dash_gas"), m.id, self.pos.0 as i64, self.pos.1 as i64, 12);
        true
    }

    /// Both cables of a pair at once; a misaim on either is a miss (the chain breaks).
    fn fire_pair(&mut self, sim: &mut StableSim<'_>, m: &Champ, tick: usize, pair: (Shot, Shot), towers: &[(i64, i64)]) {
        let r = self.rank();
        let mut hits = Vec::with_capacity(2);
        for (a, p) in [pair.0, pair.1] {
            if !self.roll(MISAIM[r]) { hits.push(p); continue; }
            let off = deg(10.0 + 30.0 * self.unit()) * if self.roll(50) { 1.0 } else { -1.0 };
            match raycast(self.pos.0, self.pos.1, a + off, towers) {
                Some(q) if (q.0 as f64 - self.pos.0).hypot(q.1 as f64 - self.pos.1) >= CABLE_MIN as f64 => hits.push(q),
                _ => { self.whiff(sim, m, a + off, tick); return; }
            }
        }
        let launch = !self.flying || self.gliding;
        if !self.flying {
            self.flying = true;
            self.speed = base_speed(r, self.apex);
            self.chain = 0;
        }
        if self.gliding { self.cables.clear(); }
        let (x, y) = self.pos;
        for &p in &hits {
            if let Some(c) = self.cables.last() {
                self.speed += GAIN.0 + GAIN.1 * angle_quality(ang_to(x, y, c.0 as f64, c.1 as f64), ang_to(x, y, p.0 as f64, p.1 as f64));
            }
            self.cables.push(p);
            self.chain += 1;
            crate::fx_point(sim, &self.fx(m, "hook"), m.id, p.0, p.1, 9);
        }
        while self.cables.len() > 2 { self.cables.remove(0); }
        if launch {
            let held: Vec<(f64, f64)> = self.cables.iter().map(|c| (c.0 as f64, c.1 as f64)).collect();
            self.heading = pull(x, y, &held, r);
        }
        self.reads = self.roll(BRAKE[r]);
        self.drop = false;
        self.pair_goal = self.goal;
        self.speed = self.speed.min(SPEED_CEIL);
        self.gliding = false;
        self.flight_until = tick + FLIGHT_T;
        self.last_cable = tick;
        if self.speed >= FAST {
            match r {
                5 => for &p in &hits { crate::fx_point(sim, &self.fx(m, "storm_hook"), m.id, p.0, p.1, 12); },
                6 => { crate::fx_point(sim, &self.fx(m, "starburst"), m.id, x as i64, y as i64, 12); }
                APEX => { crate::fx_point(sim, &self.fx(m, "apex_ring"), m.id, x as i64, y as i64, 15); }
                _ => {}
            }
        }
    }

    fn towers(sim: &StableSim<'_>) -> Vec<(i64, i64)> {
        (0..sim.tower_count()).filter_map(|i| sim.get_entity(sim.tower_id_at(i)))
            .filter(|e| e.is_alive()).map(|e| { let (x, y) = e.pos(); (x as i64, y as i64) }).collect()
    }

    /// Fire a cable toward the goal (misaims by rank). Tethered and up don't fire when no wall would do.
    fn fire(&mut self, sim: &mut StableSim<'_>, m: &Champ, tick: usize) -> bool {
        let r = self.rank();
        let Some(want) = self.goal else { return false };
        let towers = Self::towers(sim);
        let picked = self.pick_anchor(want, &towers);
        if picked.is_none() && r >= 1 { return false; }
        let shot = match picked {
            Some((a, _)) if self.roll(MISAIM[r]) => {
                // a misaimed cable veers 10-40 degrees and bites whatever is there, or nothing
                let off = deg(10.0 + 30.0 * self.unit()) * if self.roll(50) { 1.0 } else { -1.0 };
                raycast(self.pos.0, self.pos.1, a + off, &towers).map(|q| (a + off, q)).ok_or(a + off)
            }
            Some(x) => Ok(x),
            None => Err(want),
        };
        match shot {
            Ok((_, p)) => self.connect(sim, m, p, tick),
            Err(a) => self.whiff(sim, m, a, tick),
        }
        true
    }

    fn connect(&mut self, sim: &mut StableSim<'_>, m: &Champ, p: (i64, i64), tick: usize) {
        let (x, y) = self.pos;
        let a_new = ang_to(x, y, p.0 as f64, p.1 as f64);
        if !self.flying {
            self.flying = true;
            self.heading = a_new;
            self.speed = base_speed(self.rank(), self.apex);
            self.chain = 1;
            self.cables = vec![p];
        } else {
            // the newest active cable becomes cable A; the gain comes from the A-B angle, once, as B bites
            if let Some(a) = self.cables.last() {
                let a_old = ang_to(x, y, a.0 as f64, a.1 as f64);
                self.speed += GAIN.0 + GAIN.1 * angle_quality(a_old, a_new);
            }
            self.cables.push(p);
            while self.cables.len() > 2 { self.cables.remove(0); }
            self.chain += 1;
            if wrap(a_new - self.heading).abs() > TURNBACK {
                // a turnback: the cable behind him whips him round to it
                self.heading = a_new;
                self.speed *= TURNBACK_KEEP;
                self.cables = vec![p];
                crate::fx_point(sim, &self.fx(m, "dash_gas"), m.id, x as i64, y as i64, 12);
            }
        }
        let r = self.rank();
        self.reads = self.roll(BRAKE[r]);
        self.speed = self.speed.min(SPEED_CEIL);
        self.gliding = false;
        self.flight_until = tick + FLIGHT_T;
        self.last_cable = tick;
        crate::fx_point(sim, &self.fx(m, "hook"), m.id, p.0, p.1, 9);
        if self.speed >= FAST {
            match self.rank() {
                5 => crate::fx_point(sim, &self.fx(m, "storm_hook"), m.id, p.0, p.1, 12),
                6 => crate::fx_point(sim, &self.fx(m, "starburst"), m.id, x as i64, y as i64, 12),
                APEX => crate::fx_point(sim, &self.fx(m, "apex_ring"), m.id, x as i64, y as i64, 15),
                _ => false,
            };
        }
    }

    fn whiff(&mut self, sim: &mut StableSim<'_>, m: &Champ, a: f64, tick: usize) {
        let (x, y) = self.pos;
        let (wx, wy) = ((x + a.cos() * CABLE_RANGE as f64 * 0.8) as i64, (y + a.sin() * CABLE_RANGE as f64 * 0.8) as i64);
        crate::fx_point(sim, &self.fx(m, "whiff"), m.id, wx.max(0), wy.max(0), 10);
        // a miss breaks the chain: the cables let go and he glides down
        if self.flying {
            self.gliding = true;
            self.cables.clear();
            self.chain = 0;
        }
        self.recover_until = tick + RECOVER[self.rank()];
    }

    fn end_flight(&mut self) {
        self.flying = false;
        self.gliding = false;
        self.cables.clear();
        self.chain = 0;
        self.speed = 0.0;
        self.held_until = 0;
        self.drop = false;
    }

    fn crash(&mut self, sim: &mut StableSim<'_>, m: &Champ, tick: usize) {
        let speed = self.speed;
        crate::fx_point(sim, &self.fx(m, "crash"), m.id, self.pos.0 as i64, self.pos.1 as i64, 16);
        if tick >= self.rampage_until {
            let t = (30.0 + (speed - CRASH_SPEED) / 60.0).clamp(30.0, 120.0) as u64;
            sim.apply_cc(m.id, &CcV1::of_kind(CcKindV1::Stun, t));
        }
        self.end_flight();
        self.recover_until = tick + RECOVER[self.rank()];
    }

    /// One tick of flight: steer, move in small steps (walls and the anchor checked all along), cut what he passes.
    fn fly(&mut self, sim: &mut StableSim<'_>, m: &Champ, all: &[Champ], tick: usize) {
        if !self.gliding && tick >= self.flight_until {
            self.gliding = true;
            self.cables.clear();
        }
        if tick < self.air_dash {
            // the air dash: straight on at full speed
        } else if self.gliding {
            self.speed *= if self.drop { 0.7 } else { 0.9 };
            if self.speed < 1_400.0 { self.end_flight(); return; }
        } else {
            // a cable that's fallen behind him lets go
            let (x, y, h) = (self.pos.0, self.pos.1, self.heading);
            self.cables.retain(|c| wrap(ang_to(x, y, c.0 as f64, c.1 as f64) - h).abs() <= RELEASE);
            if self.cables.is_empty() {
                self.gliding = true;
                self.speed *= 0.9;
            }
        }
        if let (false, Some(&b)) = (self.gliding, self.cables.last()) {
            let (x, y) = self.pos;
            let r = self.rank();
            let held: Vec<(f64, f64)> = self.cables.iter().map(|c| (c.0 as f64, c.1 as f64)).collect();
            let mut target = pull(x, y, &held, r);
            // from Swinger up a lone cable only swings him: half toward it, half toward where he's going
            if let (1, true, Some(g)) = (held.len(), r >= PAIR_FROM, self.goal) {
                target = (target.sin() + g.sin()).atan2(target.cos() + g.cos());
            }
            // cables that would yank him back round (he's passing them) don't steer him: he carries straight on
            let d = wrap(target - self.heading);
            if d.abs() <= std::f64::consts::FRAC_PI_2 { self.heading = wrap(self.heading + d.clamp(-TURN_R[r], TURN_R[r])); }
            // a slam coming and nothing to save it: if he reads it, he lets go (no press held) or brakes softly
            let dist = (b.0 as f64 - x).hypot(b.1 as f64 - y);
            let tta = dist / self.speed.max(1.0);
            if self.reads && tta < LETGO && tick >= self.held_until {
                self.let_go();
            } else if self.reads && tta < 4.0 && self.speed >= CRASH_SPEED {
                self.speed = (self.speed * 0.75).min(dist / 4.0).max(CRASH_SPEED - 100.0);
            }
        }
        // the safety read: a wall straight ahead within 4 ticks and he reads it: brake under slam speed
        if self.reads && self.speed >= CRASH_SPEED {
            if let Some(dw) = wall_ahead(self.pos.0, self.pos.1, self.heading, self.speed * 4.0) {
                self.speed = (self.speed * 0.6).min(dw / 3.0).max(CRASH_SPEED - 100.0);
            }
        }
        let (x0, y0) = self.pos;
        let n = (self.speed / STEP).ceil().max(1.0) as usize;
        let per = self.speed / n as f64;
        let (c, s) = (self.heading.cos(), self.heading.sin());
        let (mut x, mut y) = (x0, y0);
        let mut stop: Option<bool> = None;   // Some(true) = slammed, Some(false) = landed softly
        for _ in 0..n {
            let (nx, ny) = (x + c * per, y + s * per);
            if let Some(&b) = self.cables.last() {
                if d2(nx as i64, ny as i64, b.0, b.1) <= sq(LAND_R) {
                    (x, y) = (nx, ny);
                    stop = Some(self.speed >= CRASH_SPEED);
                    break;
                }
            }
            if walls::wall_at(nx as i64, ny as i64) {
                stop = Some(self.speed >= CRASH_SPEED);
                break;
            }
            (x, y) = (nx, ny);
        }
        let (fx, fy) = walls::pull_back(x0 as i64, y0 as i64, x as i64, y as i64);
        self.pos = (fx as f64, fy as f64);
        sim.entity_set_pos(m.id, fx.max(0) as u64, fy.max(0) as u64);
        self.cuts(sim, m, all, (x0, y0), self.pos, tick);
        match stop {
            Some(true) => self.crash(sim, m, tick),
            Some(false) => self.end_flight(),
            None => {}
        }
    }

    /// Cuts: a light one on enemies right in his path while flying fast; in Rampage everyone he passes, harder.
    fn cuts(&mut self, sim: &mut StableSim<'_>, m: &Champ, all: &[Champ], a: (f64, f64), b: (f64, f64), tick: usize) {
        let rampage = tick < self.rampage_until;
        if !rampage && self.speed < 3_000.0 { return; }
        let (r, lock, dmg) = if rampage { (RAMPAGE_R, RAMPAGE_LOCK, RAMPAGE_DMG) } else { (CUT_R, CUT_LOCK, CUT_DMG) };
        let k = (self.speed / 2_600.0).clamp(0.5, SPEED_DMG_CAP);
        let (dx, dy) = (b.0 - a.0, b.1 - a.1);
        let len2 = (dx * dx + dy * dy).max(1.0);
        for e in all.iter().filter(|e| e.team != m.team) {
            let t = (((e.x as f64 - a.0) * dx + (e.y as f64 - a.1) * dy) / len2).clamp(0.0, 1.0);
            let (px, py) = (a.0 + dx * t, a.1 + dy * t);
            if d2(e.x, e.y, px as i64, py as i64) > sq(r) { continue; }
            if self.cut.get(&e.id).map_or(false, |&until| tick < until) { continue; }
            self.cut.insert(e.id, tick + lock);
            let amount = ((dmg.0 + m.attack * dmg.1 / 100) as f64 * k) as usize;
            sim.deal_damage(m.id, e.id, amount, 0, AttackTypeV1::Skill);
            crate::wave_near(sim, m.id, e.id, 15_000, amount, 0);
            crate::fx_unit(sim, &self.fx(m, if rampage { "slice_big" } else { "slice" }), m.id, e.id, 10);
        }
    }

    /// One S2 press: gas.
    fn on_gas_press(&mut self, sim: &mut StableSim<'_>, m: &Champ, all: &[Champ], tick: usize) {
        let r = self.rank();
        let want = self.want(m, all, sim);
        if self.flying && !self.gliding {
            if self.gas < BOOST_COST { return; }
            let urge = want.map_or(false, |w| w.1);
            let use_it = match r {
                0 => self.roll(50),
                1 | 2 => urge,
                _ => urge || self.speed < base_speed(r, self.apex) * 1.5,
            };
            if !use_it { return; }
            self.gas -= BOOST_COST;
            self.speed = (self.speed + BOOST_ADD).min(SPEED_CEIL);
            crate::fx_point(sim, &self.fx(m, "dash_gas"), m.id, self.pos.0 as i64, self.pos.1 as i64, 12);
        } else if !self.flying && tick >= self.dash_cd && self.gas >= DASH_COST {
            let foe_near = all.iter().any(|c| c.team != m.team && d2(c.x, c.y, m.x, m.y) <= sq(80_000));
            let go = match (r, want) {
                (0, Some(_)) => foe_near && self.roll(30),
                (_, Some((_, true, _))) => true,
                _ => false,
            };
            if !go { return; }
            let Some((a, _, _)) = want else { return };
            self.gas -= DASH_COST;
            self.dash_cd = tick + DASH_CD;
            self.dash = Some((a, tick + DASH_T));
            self.pos = (m.x as f64, m.y as f64);
            crate::fx_point(sim, &self.fx(m, "dash_gas"), m.id, m.x, m.y, 12);
        }
    }

    fn dash_step(&mut self, sim: &mut StableSim<'_>, m: &Champ, tick: usize) {
        let Some((a, until)) = self.dash else { return };
        if tick >= until { self.dash = None; return; }
        let (x, y) = (m.x as f64, m.y as f64);
        let (tx, ty) = (x + a.cos() * DASH_SPEED, y + a.sin() * DASH_SPEED);
        let (cx, cy) = walls::clip(x as i64, y as i64, tx as i64, ty as i64);
        if (cx, cy) != (tx as i64, ty as i64) { self.dash = None; }
        sim.entity_set_pos(m.id, cx.max(0) as u64, cy.max(0) as u64);
        self.pos = (cx as f64, cy as f64);
    }

    fn start_rampage(&mut self, sim: &mut StableSim<'_>, m: &Champ, all: &[Champ], tick: usize) {
        self.rampage_until = tick + RAMPAGE_T;
        let mut b = timed("lv_rampage", RAMPAGE_T);
        b.cc_immune = true;
        sim.add_buff(m.id, &b);
        self.recover_until = 0;
        // from the ground: a free, sure cable toward the fight
        if !self.flying {
            let target = all.iter().filter(|c| c.team != m.team && sim.is_visible(m.team, c.id))
                .min_by_key(|c| d2(c.x, c.y, m.x, m.y)).map(|c| ang_to(m.x as f64, m.y as f64, c.x as f64, c.y as f64));
            if let Some(a) = target.or(self.goal) {
                self.goal = Some(a);
                self.pos = (m.x as f64, m.y as f64);
                let towers: Vec<(i64, i64)> = Vec::new();
                if let Some((_, p)) = self.pick_anchor(a, &towers) { self.connect(sim, m, p, tick); }
            }
        }
    }

    /// 0 slow, 1 fast, then the rank forms: 2 Stormcutter, 3 Comet, 4 Apex
    fn tier(&self) -> usize {
        if !self.flying || self.speed < CRASH_SPEED { 0 } else if self.speed < FAST { 1 } else {
            match self.rank() { 5 => 2, 6 => 3, APEX => 4, _ => 1 }
        }
    }

    fn set_form(&mut self, sim: &mut StableSim<'_>, m: &Champ, form: usize, streaming: bool) {
        // round 84: from Stormcutter up he wears his mantle hanging (lv_skin) whenever it isn't streaming behind him
        let skin = match self.rank() { 5 => 2, 6 => 3, APEX => 4, _ => 0 };
        let want_skin = if form >= 2 || streaming || skin == 0 { 0 } else { skin };
        if self.skin != want_skin || (want_skin > 0 && !m.has(&format!("lv_skin{want_skin}"))) {
            for k in 2..=4 { sim.entity_remove_buff(m.id, &format!("lv_skin{k}")); }
            if want_skin > 0 { sim.add_buff(m.id, &BuffV1::named(&format!("lv_skin{want_skin}"))); }
            self.skin = want_skin;
        }
        if self.form == form && (form < 2 || m.has(&format!("lv_form{form}"))) { return; }
        for k in 2..=4 { sim.entity_remove_buff(m.id, &format!("lv_form{k}")); }
        if form >= 2 {
            sim.add_buff(m.id, &BuffV1::named(&format!("lv_form{form}")));
            // the moment he reaches the form: one burst that rides on him
            if self.form < 2 { crate::fx_unit(sim, &self.fx(m, &format!("ignite{form}")), m.id, m.id, 30); }
        }
        self.form = form;
    }

    fn visuals(&mut self, sim: &mut StableSim<'_>, m: &Champ, tick: usize) {
        let tier = self.tier();
        // round 84: from Stormcutter up his mantle streams behind him, dragged by the wind, while he flies in his form or
        // dashes (on the ground or in the air)
        let rank_tier = match self.rank() { 5 => 2, 6 => 3, APEX => 4, _ => 0 };
        let dashing = tick < self.air_dash || self.dash.is_some();
        let stream = if tier >= 2 { tier } else if dashing { rank_tier } else { 0 };
        self.set_form(sim, m, if tier >= 2 { tier } else { 0 }, stream > 0);
        if stream > 0 && tick % 2 == 0 {
            let h = match self.dash { Some((a, _)) if !self.flying => a, _ => self.heading };
            let d = ((h.to_degrees().rem_euclid(360.0) / 22.5).round() as usize) % 16;
            crate::fx_unit(sim, &self.fx(m, &format!("stream{stream}_{d}_{}", (tick / 2) % 4)), m.id, m.id, 2);
        }
        if !self.flying { self.last_trail = None; }
        if self.flying && tick % 2 == 0 {
            let (x, y) = self.pos;
            // round 81: the cables hum (a phase every 2 ticks: light runs along them), and the newest one shoots
            // out over its first 4 ticks
            let ph = (tick / 2) % 4;
            let n = self.cables.len();
            let mut wires = Vec::with_capacity(n);
            for (i, &(ax, ay)) in self.cables.iter().enumerate() {
                let k = if i + 1 == n && tick < self.last_cable + 4 { (tick + 1 - self.last_cable) as f64 / 4.0 } else { 1.0 };
                let (ex, ey) = (x + (ax as f64 - x) * k, y + (ay as f64 - y) * k);
                let a = ang_to(x, y, ax as f64, ay as f64).to_degrees().rem_euclid(180.0);
                let d = ((a / 11.25).round() as usize) % 16;
                // a long cable is a chain of segments (the sprites go up to SEG_PX)
                let len_px = (ex - x).hypot(ey - y) / 950.0;
                let segs = (len_px / SEG_PX).ceil().max(1.0) as usize;
                let b = ((len_px / segs as f64 / 16.0).round() as usize).clamp(1, 6);
                for s_ in 0..segs {
                    let t = (s_ as f64 + 0.5) / segs as f64;
                    let (mx, my) = ((x + (ex - x) * t) as i64, (y + (ey - y) * t) as i64);
                    wires.push((format!("cable_{d}_{b}_{ph}"), mx.max(0), my.max(0)));
                }
            }
            for (tag, mx, my) in wires {
                crate::fx_point(sim, &self.fx(m, &tag), m.id, mx, my, 2);
            }
            let d = ((self.heading.to_degrees().rem_euclid(360.0) / 22.5).round() as usize) % 16;
            let trail = self.fx(m, &format!("trail{tier}_{d}"));
            // at high speed the copies would spread apart: fill the gap since the last one (at most 3 extra)
            if let Some((lx, ly)) = self.last_trail {
                let gap_px = (x - lx).hypot(y - ly) / 950.0;
                let extra = ((gap_px / 18.0).ceil() as usize).saturating_sub(1).min(3);
                for i in 1..=extra {
                    let t = i as f64 / (extra + 1) as f64;
                    crate::fx_point(sim, &trail, m.id, (lx + (x - lx) * t) as i64, (ly + (y - ly) * t) as i64, 2);
                }
            }
            crate::fx_point(sim, &trail, m.id, x as i64, y as i64, 2);
            self.last_trail = Some((x, y));
            // at a full chain, an afterimage where he was a moment ago, tinted by his form
            if self.chain >= 8 && tick % 4 == 0 {
                if let Some(p) = self.track.iter().rev().nth(6).copied() {
                    let side = if self.heading.cos() < 0.0 { "l" } else { "r" };
                    let tag = if tier >= 2 { format!("after{tier}_{side}") } else { format!("after_{side}") };
                    crate::fx_point(sim, &self.fx(m, &tag), m.id, p.1, p.2, 9);
                }
            }
        }
        // the HUD: chain pips, gas, the rank badge
        let pips = if self.flying { self.chain.min(8) } else { 0 };
        let gas = (self.gas.clamp(0, GAS_MAX) + 5) / 10;
        let rank = (self.rank(), self.apex);
        let badge = if rank.0 >= APEX { format!("lv_apex{}", rank.1.unwrap_or(10)) } else { format!("lv_rank{}", rank.0) };
        let have = |n: &str| m.has(n);
        if self.shown.0 != Some(pips) || (pips > 0 && !have(&format!("lv_pips{pips}"))) {
            for k in 1..=8 { sim.entity_remove_buff(m.id, &format!("lv_pips{k}")); }
            if pips > 0 { sim.add_buff(m.id, &BuffV1::named(&format!("lv_pips{pips}"))); }
            self.shown.0 = Some(pips);
        }
        if self.shown.1 != Some(gas) || !have(&format!("lv_gas{gas}")) {
            for k in 0..=10 { sim.entity_remove_buff(m.id, &format!("lv_gas{k}")); }
            sim.add_buff(m.id, &BuffV1::named(&format!("lv_gas{gas}")));
            self.shown.1 = Some(gas);
        }
        if self.rank.is_some() && (self.shown.2 != Some(rank) || !have(&badge)) {
            for k in 0..APEX { sim.entity_remove_buff(m.id, &format!("lv_rank{k}")); }
            for p in 1..=10 { sim.entity_remove_buff(m.id, &format!("lv_apex{p}")); }
            sim.add_buff(m.id, &BuffV1::named(&badge));
            self.shown.2 = Some(rank);
        }
    }
}

impl StablePassive for Levi {
    fn clone_box(&self) -> Box<dyn StablePassive> {
        Box::new(self.clone())
    }
    fn on_dead(&mut self, sim: &mut StableSim<'_>, _player: usize) {
        if let Some(me) = self.me {
            for k in 2..=4 {
                for n in ["lv_form", "lv_skin"] { sim.entity_remove_buff(me, &format!("{n}{k}")); }
            }
        }
        self.skin = 0;
        self.form = 0;
        self.last_trail = None;
        self.end_flight();
        self.dash = None;
        self.was_alive = false;
        self.shown = (None, None, None);
    }
    fn on_update(&mut self, sim: &mut StableSim<'_>, _seed: u64, player: usize, entity: usize) {
        let tick = sim.tick();
        self.me = Some(entity);
        if !self.started {
            self.started = true;
            self.rng = sim.seed() ^ (entity as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ 0x1E71;
            self.gas = GAS_MAX;
            let _ = memory();
        }
        if self.rank.is_none() && tick >= 60 { self.latch_rank(sim, player); }
        let all = champions(sim);
        let Some(m) = all.iter().find(|c| c.id == entity).cloned() else { return };
        if self.team.is_none() { self.team = Some(m.team); }
        if self.sig.is_none() && tick >= 1800 && self.rank.is_some() { self.make_sig(sim, &all); }
        if self.sig.is_some() && tick % 30 == 0 { self.track_result(sim, tick); }
        if !self.was_alive {
            // (re)spawned: that spot is home, and the tank is full
            self.was_alive = true;
            self.home = Some((m.x, m.y));
            self.gas = GAS_MAX;
            self.end_flight();
            self.track.clear();
        }
        if self.home.map_or(false, |h| d2(m.x, m.y, h.0, h.1) <= sq(HOME_R)) { self.gas = GAS_MAX; }
        // where the game is walking him (fresh move orders only), and the walking distances to it
        self.dest = dest_of(sim.seed(), player).filter(|d| tick <= d.2 + DEST_FRESH && !walls::wall_at(d.0, d.1)).map(|d| (d.0, d.1));
        if let Some(d) = self.dest {
            let cell = (d.0 / walls::CELL, d.1 / walls::CELL);
            if self.field.as_ref().map_or(true, |f| f.0 != cell) { self.field = Some((cell, field(d))); }
        }
        self.track.push_back((tick, if self.flying { self.pos.0 as i64 } else { m.x }, if self.flying { self.pos.1 as i64 } else { m.y }));
        while self.track.len() > 40 { self.track.pop_front(); }

        // ---- the data slots' markers: one press each
        if m.has("lv_ult") && !m.has("lv_ultseen") {
            sim.add_buff(entity, &timed("lv_ultseen", 10));
            self.start_rampage(sim, &m, &all, tick);
        }
        if m.has("lv_s1") && !m.has("lv_s1seen") {
            sim.add_buff(entity, &timed("lv_s1seen", 8));
            if !m.stunned { self.on_cable_press(sim, &m, &all, tick); }
        }
        if m.has("lv_s2") && !m.has("lv_s2seen") {
            sim.add_buff(entity, &timed("lv_s2seen", 8));
            if !m.stunned { self.on_gas_press(sim, &m, &all, tick); }
        }
        if m.stunned && tick >= self.rampage_until && self.flying { self.end_flight(); }
        if !m.stunned { self.maybe_fire(sim, &m, &all, tick); }

        if self.flying {
            self.fly(sim, &m, &all, tick);
        } else if self.dash.is_some() {
            self.dash_step(sim, &m, tick);
        }
        self.visuals(sim, &m, tick);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn angle_quality_peaks_at_90() {
        assert!((angle_quality(0.0, deg(90.0)) - 1.0).abs() < 1e-9);
        assert!((angle_quality(0.0, deg(45.0)) - 0.5).abs() < 1e-9);
        assert!((angle_quality(0.0, deg(135.0)) - 0.5).abs() < 1e-9);
        assert!((angle_quality(0.0, deg(0.0)) - ANGLE_FLOOR).abs() < 1e-9);
        assert!((angle_quality(0.0, deg(180.0)) - ANGLE_FLOOR).abs() < 1e-9);
        // the minor angle, whichever way round
        assert!((angle_quality(deg(350.0), deg(80.0)) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn flights_follow_the_walking_path_and_read_walls() {
        let _grid = walls::TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        // a wall across the map at row 15, open only at x 26-29
        let mut cells = vec![false; (walls::N * walls::N) as usize];
        for x in 0..26 { cells[(15 * walls::N + x) as usize] = true; }
        walls::set(cells);
        let c = |v: f64| v * walls::CELL as f64;
        let dest = (c(5.5) as i64, c(10.5) as i64);
        let f = field(dest);
        let (ax, ay, path) = aim_point(&f, c(5.5), c(20.5), dest);
        // the straight line is walled off: he aims along the path (east, toward the gap), and it's the long way
        assert!(ax > c(8.0) && ay > c(15.0), "aims along the path: {ax} {ay}");
        assert!(path > c(30.0), "path length {path}");
        // a visible destination is aimed at directly
        let (bx, by, _) = aim_point(&f, c(5.5), c(12.5), dest);
        assert_eq!((bx as i64, by as i64), dest);
        // path reading: flying north at 5000, a cable to the near side of the wall is clear, one past it isn't
        let mut l = Levi::default();
        l.flying = true;
        l.pos = (c(5.5), c(20.5));
        l.heading = -std::f64::consts::FRAC_PI_2;
        l.speed = 5_000.0;
        assert!(!l.path_hits((c(5.5) as i64, c(16.1) as i64), 40));
        assert!(l.path_hits((c(5.5) as i64, c(12.0) as i64), 40));
        // grazing: a cable along the wall's face, 2000 off it, counts as a wall
        l.pos = (c(2.5), c(16.0) + 2_000.0);
        l.heading = 0.0;
        assert!(l.path_hits((c(20.0) as i64, (c(16.0) + 2_000.0) as i64), 40));
        walls::set(vec![false; (walls::N * walls::N) as usize]);
    }

    #[test]
    fn the_pair_is_one_wall_each_side_and_he_flies_between() {
        let _grid = walls::TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        // a corridor running east: walls along rows 12 and 16, open between
        let mut cells = vec![false; (walls::N * walls::N) as usize];
        for x in 0..30 {
            cells[(12 * walls::N + x) as usize] = true;
            cells[(16 * walls::N + x) as usize] = true;
        }
        walls::set(cells);
        let c = |v: f64| v * walls::CELL as f64;
        let mut l = Levi::default();
        l.rank = Some(APEX);
        l.apex = Some(1);
        l.pos = (c(5.0), c(14.5));
        let ((_, a), (_, b)) = l.pick_pair(0.0, &[]).expect("a pair in the corridor");
        // one anchor on each wall, both ahead of him
        let (top, bottom) = if a.1 < b.1 { (a, b) } else { (b, a) };
        assert!((top.1 as f64) < c(13.5) && (bottom.1 as f64) > c(15.5), "one each side: {top:?} {bottom:?}");
        assert!(top.0 as f64 > c(5.0) && bottom.0 as f64 > c(5.0), "both ahead");
        // the pull between them points down the corridor, not at a wall
        let pd = pull(l.pos.0, l.pos.1, &[(a.0 as f64, a.1 as f64), (b.0 as f64, b.1 as f64)], APEX);
        assert!(pd.abs().to_degrees() < 15.0, "flies between: {}", pd.to_degrees());
        // Tethered doesn't fire pairs
        l.rank = Some(1);
        assert!(l.pick_pair(0.0, &[]).is_none());
        walls::set(vec![false; (walls::N * walls::N) as usize]);
    }

    #[test]
    fn apex_plans_a_clear_line_down_the_corridor() {
        let _grid = walls::TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut cells = vec![false; (walls::N * walls::N) as usize];
        for x in 0..30 {
            cells[(12 * walls::N + x) as usize] = true;
            cells[(16 * walls::N + x) as usize] = true;
        }
        walls::set(cells);
        let c = |v: f64| v * walls::CELL as f64;
        let dest = (c(27.5) as i64, c(14.5) as i64);
        let f = field(dest);
        let mut l = Levi::default();
        l.rank = Some(APEX);
        l.apex = Some(1);
        l.pos = (c(3.0), c(14.5));
        let st = l.fly_state();
        let (cost, (a, b)) = l.plan(st, PLAN_DEPTH[APEX], &f, dest, &[]).expect("a line down the corridor");
        // the line gets him there far faster than walking (24 cells at 1050 a tick is ~730 ticks)
        assert!(cost < 300.0, "cost {cost}");
        let (e, _, hit) = l.sim_pair(st, (a.1 .0 as f64, a.1 .1 as f64), (b.1 .0 as f64, b.1 .1 as f64), PLAN_T, false);
        assert!(!hit, "the first pair is clear");
        assert!(e.x > l.pos.0, "and carries him on toward the end");
        walls::set(vec![false; (walls::N * walls::N) as usize]);
    }

    #[test]
    fn rank_speeds() {
        assert_eq!(base_speed(0, None), 2200.0);
        assert_eq!(base_speed(APEX, Some(1)), 4800.0);
        assert_eq!(base_speed(APEX, Some(10)), 4300.0);
        assert!((0..APEX).all(|r| base_speed(r, None) < base_speed(r + 1, Some(10))));
        assert!(MISAIM.windows(2).all(|w| w[0] >= w[1]));
        assert!(RECOVER.windows(2).all(|w| w[0] >= w[1]));
    }

    #[test]
    fn chain_gain_needs_a_new_cable() {
        // a cable pair at 90 degrees adds the full gain once, when the second cable bites
        let mut l = Levi::default();
        l.flying = true;
        l.pos = (500_000.0, 500_000.0);
        l.speed = 2_200.0;
        l.cables = vec![(590_000, 500_000)];
        l.chain = 1;
        let before = l.speed;
        let (x, y) = l.pos;
        let a_old = ang_to(x, y, 590_000.0, 500_000.0);
        let a_new = ang_to(x, y, 500_000.0, 590_000.0);
        l.speed += GAIN.0 + GAIN.1 * angle_quality(a_old, a_new);
        assert!((l.speed - before - 800.0).abs() < 1e-6);
    }
}
