//! Emperor Isliid: seven individually tracked swords and position-cast engravings.
//! All mutable combat state lives in the passive instance so replay/planning clones
//! cannot change the live match. Native effects only pass the cast point as a buff.

use crate::scribble::Memory;
use crate::{sq, timed};
use mod_api_stable::{AttackTypeV1, BuffV1, CcV1, InputTargetV1,
    SimOriginV1, StableEffectType, StablePassive, StableSim, StatV1};
use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, OnceLock};

const SWORDS: [&str; 7] = ["skylight", "terra", "darkbringer", "gale", "blood", "rift", "emperor"];
const NAMES: [&str; 8] = ["Bearer", "Squire", "Engraver", "Tactician", "Swordmaster", "Regent", "Sovereign", "Imperial"];
const SNAP: i64 = 12_000;
const MELEE: i64 = 23_000;
const ANCHOR_LIFE: usize = 1800;
/// Round 93 (Rian: "make the engraving disappear faster like the lines, that might be what's lagging"): an unfired
/// stroke lives 8 s (was 20 s; a stroke of a live plan lives at least until the plan's deadline), and the strokes of a
/// formation that fires flare, then fade out within FIRED_LIFE.
const MARK_LIFE: usize = 480;
const FIRED_LIFE: usize = 90;
/// Round 93: how long a sword's strokes still count against the power of his next engravings (power is divided by the
/// swords with strokes on the ground). Kept at the round 92 stroke life, so shorter-lived strokes don't make him stronger.
const COMMIT_LIFE: usize = 1200;
const RETURN_MULT: i64 = 3;
const RETURN_DIV: i64 = 2;
const SPEED: [i64; 7] = [8_000, 5_500, 7_000, 12_000, 7_500, 9_000, 6_500];
const THINK_TICKS: [usize; 8] = [90, 75, 60, 48, 38, 30, 22, 15];
const LOOK_AHEAD: [i64; 8] = [0, 30, 60, 90, 120, 180, 240, 300];
const PATTERN_BUDGET: [usize; 8] = [3, 5, 8, 12, 16, 21, 26, 30];
// Round 88 (sword control): escorts for threatened allies (how many per ally, how often the brain looks again), how
// soon an idle grounded sword comes home, how often an escort strikes from its host. Higher mastery reassesses and
// reallocates sooner; no sword is rank-locked.
const ESCORTS: [usize; 8] = [1, 1, 1, 1, 2, 2, 2, 3];
/// Round 91: an escort's lease grows with mastery; it's renewed only when he notices the ally again (NOTICE).
const REASSESS: [usize; 8] = [45, 55, 65, 80, 95, 110, 130, 150];
const IDLE_RETURN: [usize; 8] = [240, 210, 180, 150, 120, 100, 80, 60];
const STRIKE_GAP: [usize; 8] = [90, 84, 78, 72, 66, 60, 54, 48];
/// An ally with an enemy champion this close (now or forecast) and missing 25% HP, or two enemies close, is threatened.
const THREAT_R: i64 = 105_000;
/// A cast is accepted only this soon after the brain wanted it (the 5-tick start timing fits inside).
const WANT_WINDOW: usize = 12;
/// A sword touched by an accepted command is left alone by the brain this long.
const LOCK_TICKS: usize = 60;
/// Round 89: a formation's deadline past the slowest sword's travel (it still has to land, arm and draw its stroke).
const PLAN_SLACK: usize = 150;

/// Round 88: engraving grades, the same in the editor's Engraving lab (isliidlab.js GRADES): unrounded accuracy, the
/// lowest accuracy of each grade and its multiplier in percent. Under 60 the engraving fails (nothing applies).
const GRADES: [(&str, f64, usize); 5] = [("Imperial", 99.0, 120), ("Perfect", 95.0, 110), ("Refined", 85.0, 100),
    ("Stable", 70.0, 85), ("Crude", 60.0, 70)];
/// A solo stroke's quality by mastery (graded like a formation). Round 91: much wider (Rian: only the very top is
/// perfect); Imperial goes from IMPERIAL_SOLO at #10 to 100 at #1.
const SOLO_QUALITY: [f64; 8] = [45.0, 53.0, 60.0, 67.0, 74.0, 81.0, 87.0, 92.0];
/// How far off a planned stroke's endpoints land, by mastery (both ends, +e / -e). Round 91: much wider; Imperial uses
/// IMPERIAL_WOBBLE by level (#1 = 0, the only perfect hand).
const WOBBLE: [i64; 8] = [13_800, 12_000, 10_500, 8_250, 6_450, 4_500, 3_000, 1_600];
/// Imperial #1..#10: the aim error per level step (#1 none, #10 9 steps).
const IMPERIAL_WOBBLE_STEP: i64 = 178;
/// Round 91: the chance (%) that, at each look, he notices a threatened ally at all (Imperial #10 .. #1 from 32 to 99).
/// Tuned in the Engraving lab's skirmish to ally cover ~10 / 20 / 30 / 42 / 55 / 67 / 78 / 86 (#10) .. 99 (#1) %; higher
/// ranks also look more often (THINK_TICKS), hold escorts longer (REASSESS) and send more (ESCORTS).
const NOTICE: [u64; 8] = [6, 15, 15, 25, 27, 32, 32, 32];

/// A formation's accuracy: 100 minus the summed endpoint error over the legs, relative to radius x legs (unrounded).
fn formation_accuracy(error: i64, radius: i64, legs: usize) -> f64 {
    (100.0 - error as f64 * 100.0 / (radius.max(1) as f64 * legs.max(1) as f64)).clamp(0.0, 100.0)
}

/// The grade of an accuracy and its multiplier (percent), or None: too far off to take effect.
fn grade(accuracy: f64) -> Option<(&'static str, usize)> {
    GRADES.iter().find(|g| accuracy >= g.1).map(|g| (g.0, g.2))
}

/// Round 91: his aim error by mastery: the rank's, or at Imperial by level (#1 = 0).
fn wobble(rank: usize, imperial: Option<usize>) -> i64 {
    if rank >= 7 { (imperial.unwrap_or(10).clamp(1, 10) as i64 - 1) * IMPERIAL_WOBBLE_STEP } else { WOBBLE[rank] }
}

/// A solo stroke's quality by mastery: the rank's, or at Imperial from SOLO_QUALITY[7] at #10 up to 100 at #1.
fn solo_quality(rank: usize, imperial: Option<usize>) -> f64 {
    if rank >= 7 { 100.0 - (100.0 - SOLO_QUALITY[7]) * (imperial.unwrap_or(10).clamp(1, 10) - 1) as f64 / 9.0 } else { SOLO_QUALITY[rank] }
}

/// The notice chance (%) by mastery: the rank's, or at Imperial from NOTICE[7] at #10 up to 99 at #1.
fn notice_pct(rank: usize, imperial: Option<usize>) -> u64 {
    if rank >= 7 { 99 - (99 - NOTICE[7]) * (imperial.unwrap_or(10).clamp(1, 10) as u64 - 1) / 9 } else { NOTICE[rank] }
}

/// Whether he notices threatened ally `ally` on this look (a hash of seed, tick and ally: the precomputed and the live
/// simulation agree; the lab reproduces it exactly).
fn notices(seed: u64, tick: usize, ally: usize, pct: u64) -> bool {
    let h = (seed ^ (tick as u64).wrapping_mul(0x9e37_79b9) ^ ((ally as u64) << 32)).wrapping_mul(0x2545_f491_4f6c_dd1d);
    (h >> 33) % 100 < pct
}

/// The aim error of planned stroke j of sword i (native and lab share it exactly); `w` from `wobble`.
fn plan_wobble(seed: u64, tick: usize, i: usize, j: usize, w: i64) -> i64 {
    if w == 0 { return 0; }
    let salt = ((seed ^ tick as u64 ^ ((i as u64) << 24) ^ j as u64).wrapping_mul(0x9e37_79b9)) as i64;
    salt.rem_euclid(w * 2 + 1) - w
}

/// Round 95 (Rian: "the sword animation for moving isn't there"): natively spawned projectile art never rendered (as
/// with Vader's saber), so a flying sword is a point effect at its live position. Round 96 (Rian: "now it lags because
/// of the flying swords ... turn it into something that lessens the use, 8 sprites"): it flies as a small Spirit
/// comet (Imperial a solar system) in an 8-frame loop, at 8 headings: below FAST_FLY a 2-frame pair every
/// SLOW_FLY_STEP ticks (most of the launch ramp), at or above it one frame every FAST_FLY_STEP (round 99: a pair's first frame).
const FAST_FLY: i64 = 2_500;
const SLOW_FLY_STEP: usize = 6;
const FAST_FLY_STEP: usize = 3;

/// Round 96: how often a flying sword at `speed` is redrawn.
fn fly_cadence(speed: i64) -> usize { if speed < FAST_FLY { SLOW_FLY_STEP } else { FAST_FLY_STEP } }
/// Round 90: grounded swords and aura fields change frame every 6 ticks.
const FRAME_STEP: usize = 6;
/// Round 91: grounded swords are emitted as 2-frame pairs of their 12-frame loop (_pair0.._pair5), every 12 ticks.
const PAIR_STEP: usize = 12;
/// Round 92 (Rian: "laggy when Imperial uses an engraving, a sudden burst of many swords"): a formation's swords leave
/// one after another, this many ticks apart (a rapid volley instead of every launch, flight and aura on one tick).
const LAUNCH_STAGGER: usize = 3;
/// Round 92: at most this many hit markers per formation (the champions nearest its centre).
const HITMARK_CAP: usize = 6;
/// The tick the k-th sword of a formation planned on `tick` leaves.
fn launch_tick(tick: usize, k: usize) -> usize { tick + k * LAUNCH_STAGGER }
const FX_LAUNCH: u8 = 1;
const FX_RECALL: u8 = 2;
const FX_IMPACT: u8 = 4;
/// Round 97: a sword just came home into Imperial's black hole (a wormhole swallows it there).
const FX_INTO_HOLE: u8 = 8;
/// Round 97 (Rian: "at Imperial he has a black hole on top of his head where he sends out the swords ... and a wormhole
/// when the swords travel"): where the black hole sits relative to him (the art's HOLE_C in a 72 x 128 buff frame at
/// 950 units a pixel), and over how much flight a sword leaving it (or coming home) blends from the hole to its path.
const HEAD_LIFT: (i64, i64) = (-5_700, -35_000);
const HOLE_BLEND: i64 = 60_000;

/// Round 97: Imperial's black hole buff with `inside` swords in it (#1's is prismatic).
fn hole_buff(inside: usize, top: bool) -> String { format!("il_blackhole{}_n{}", if top { "1" } else { "" }, inside.min(7)) }

/// Round 97: the wormhole a sword leaves (`way` "out") or arrives (`way` "in") through; #1's are prismatic.
fn wormhole(way: &str, i: usize, top: bool) -> String { format!("wormhole_{way}_{}", if top { "p" } else { SWORDS[i] }) }

/// Round 97: how much of HEAD_LIFT still applies `d` units from the hole (all of it at the hole, none from HOLE_BLEND on).
fn hole_lift(d: i64) -> (i64, i64) {
    let k = (HOLE_BLEND - d.clamp(0, HOLE_BLEND)) as f64 / HOLE_BLEND as f64;
    ((HEAD_LIFT.0 as f64 * k) as i64, (HEAD_LIFT.1 as f64 * k) as i64)
}

/// The one visual a sword has in each state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Visual { Orbit, Flying(bool), Grounded(bool) }

fn visual_for(mode: SwordMode, idle_path: bool) -> Visual {
    match mode {
        SwordMode::Orbit => Visual::Orbit,
        SwordMode::Draw => Visual::Flying(true),
        SwordMode::Throw | SwordMode::Strike | SwordMode::Return => Visual::Flying(false),
        SwordMode::Stage if !idle_path => Visual::Flying(false),
        SwordMode::Ready => Visual::Grounded(true),
        SwordMode::Stage | SwordMode::Planted => Visual::Grounded(false),
    }
}

/// Round 96: a flying sword's heading in 8 steps over the full turn (0 right, 2 down, 4 left, 6 up with y down), the
/// `_comet_a<h>` art it shows; `trail_angle` is the 16-step half-turn version for unoriented strokes.
fn fly_angle(from: (i64, i64), to: (i64, i64)) -> usize {
    let angle = ((to.1 - from.1) as f64).atan2((to.0 - from.0) as f64).rem_euclid(std::f64::consts::TAU);
    ((angle * 8.0 / std::f64::consts::TAU).round() as usize) % 8
}

/// Round 96: the flight art's tag at `tick` for sword `i` at art rank `art` (8 = Imperial #1), heading `h`, redrawn
/// every `step` ticks: a 2-frame pair of the 8-frame loop when slow, one frame when fast.
/// Round 99: always a 2-frame pair, so the game holds half the comet views (the single-frame aliases are gone). A fast
/// flight's comet lives FAST_FLY_STEP ticks, so it shows each pair's first frame: a 4-frame loop instead of 8.
fn comet_tag(i: usize, art: usize, h: usize, step: usize, tick: usize) -> String {
    format!("{}_rank{art}_comet_a{h}_pair{}", SWORDS[i], (tick / step.max(1)) % 4)
}

/// Round 89: engraving art comes in four mastery tiers, the swords' own (Bearer-Squire, Engraver-Tactician,
/// Swordmaster-Regent, Sovereign-Imperial): scars, leg flares, the activation burst, hit markers and the shatter.
fn tier(rank: usize) -> usize { [0, 0, 1, 1, 2, 2, 3, 3][rank.min(7)] }
/// How long a formation's legs flare after it fires.
const FLARE_TICKS: usize = 36;
/// Round 98 (Rian: "laggy when the swords make an engraving spot ... usage spikes"): a fired formation's legs light one
/// after another, RIPPLE ticks apart, so their flares run round the shape instead of all landing on one tick.
const RIPPLE: usize = 6;

/// Round 98: Imperial's engraving strokes are constellation lines, their own art tier (bursts, hit markers and shatters
/// keep tier 3).
fn stroke_tier(rank: usize) -> usize { if rank >= 7 { 4 } else { tier(rank) } }

/// Round 98: the sword that dominates a formation (the most drawn length, ties to the lower sword): at Imperial its body
/// is what falls on the engraving. `strokes` = (sword, from, to).
fn dominant_sword(strokes: &[(usize, (i64, i64), (i64, i64))]) -> usize {
    let mut length = [0_i64; 7];
    for &(sword, from, to) in strokes { length[sword.min(6)] += dist(from, to); }
    (0..7).rev().max_by_key(|&i| length[i]).unwrap_or(0)
}

/// Round 98: Imperial's formation fire: the dominant sword's body falling onto the shape (#1's prismatic).
fn fall_name(sword: usize, radius: i64, top: bool) -> String {
    format!("fall_{}_r{}{}", SWORDS[sword], u8::from(radius > 45_000), if top { "_p" } else { "" })
}
/// Round 90 (lag): one scar sprite per this much stroke (the art covers it), how often fresh and cooled strokes are
/// re-emitted (their art loops over exactly that long), when a stroke cools, and the sprite count above which cooled
/// strokes keep every second piece.
const SCAR_STEP: i64 = 30_000;
const SCAR_HOT_EVERY: usize = 12;
const SCAR_COOL_EVERY: usize = 120;   // round 93: was 60 (the cooled groove is one still frame)
const SCAR_HOT: usize = 180;   // round 93: was 300
const SCAR_BUDGET: usize = 160;
/// Frames in a grounded sword's loop (round 89: 12, was 8), read as single-frame aliases.
const PLANTED_FRAMES: usize = 12;
/// Frames of a logo's completion pop (its `_f<k>` aliases), one per 6 ticks.
const POP_FRAMES: usize = 6;
/// The family each sword's solo stroke fires as (Skylight reveal, Terra slow, Darkbringer shred, Gale speed, Blood
/// damage, Rift pull, Emperor attack).
const SOLO_FAMILY: [usize; 7] = [9, 1, 5, 4, 0, 2, 8];

/// The activation burst: its family motif, its tier and its size (0 up to a 40000 radius, 1 above).
fn fire_name(family: usize, tier: usize, radius: i64) -> String {
    format!("fire_{}_t{tier}_r{}", FAMILIES[family.min(12)], usize::from(radius > 40_000))
}

/// Effect families of PATTERNS[..].effect, as the logos are named (tools/generate_isliid_eight_frame_art.py FAMILIES).
const FAMILIES: [&str; 13] = ["damage", "bind", "pull", "push", "speed", "shred", "weaken", "guard", "attack", "burst",
    "cooldown", "heal", "domain"];

/// The logo for a plan's flag: `pattern_<i>` -> its effect family, `solo_<sword>` -> that sword's own logo.
fn logo_name(tag: &str, phase: FlagPhase) -> Option<String> {
    let ph = match phase { FlagPhase::Planned => "planned", FlagPhase::Drawing => "drawing",
        FlagPhase::Complete => "complete", FlagPhase::Cancelled => "cancelled" };
    let kind = if let Some(i) = tag.strip_prefix("pattern_") {
        FAMILIES.get(PATTERNS.get(i.parse::<usize>().ok()?)?.effect)?.to_string()
    } else {
        format!("solo{}", tag.strip_prefix("solo_")?.parse::<usize>().ok().filter(|&k| k < 7)?)
    };
    Some(format!("logo_{kind}_{ph}"))
}

fn arsenal_buff(i: usize, rank: usize, selected: bool) -> String {
    format!("il_ar_{}_rank{rank}{}", SWORDS[i], if selected { "_sel" } else { "" })
}

/// An ally as the brain sees it: id, position now, forecast position, missing HP %.
type AllyFuture = (usize, (i64, i64), (i64, i64), usize);

/// How useful sword `i` is as an escort for an ally missing `missing`% HP with `foes` enemy champions close.
fn escort_score(i: usize, missing: usize, foes: usize) -> i64 {
    let m = missing as i64;
    let f = foes as i64;
    match i {
        0 => 10 + if m < 25 && f >= 1 { 15 } else { 0 },        // Skylight: vision, range for a healthy carry
        1 => 20 + m / 2 + if f >= 2 { 10 } else { 0 },           // Terra: peel (damage reduction, slows)
        2 => 15 + if m < 30 { 15 } else { 0 },                    // Darkbringer: a carry's focus (attack, shred)
        3 => 15 + if m >= 60 { 25 } else { 0 },                   // Gale: escape (move speed)
        4 => 15 + m / 3,                                          // Blood: brawl sustain (lifesteal)
        5 => 12 + 8 * f,                                          // Rift: engage (radius, slows, pulls)
        _ => if m < 40 { 18 } else { 8 },                         // Emperor: tempo (cooldowns)
    }
}

/// Only take an objective when a living teammate is near it.
fn objective_ok(p: (i64, i64), allies: &[(usize, (i64, i64))]) -> bool {
    allies.iter().any(|(_, a)| near(*a, p, 100_000))
}


fn aura_source(mode: SwordMode, sword_pos: (i64,i64), holder_pos: Option<(i64,i64)>) -> Option<(i64,i64)> {
    if mode==SwordMode::Orbit {holder_pos} else {Some(sword_pos)}
}

// A scar is an unoriented line: sixteen angles cover a half turn at 11.25 degrees.
fn trail_angle(from: (i64, i64), to: (i64, i64)) -> usize {
    let angle = ((to.1 - from.1) as f64).atan2((to.0 - from.0) as f64)
        .rem_euclid(std::f64::consts::PI);
    ((angle * 16.0 / std::f64::consts::PI).round() as usize) % 16
}

/// Round 90: the fewest swords a formation should use by mastery, so a lone target gets a real shape (a triangle, a
/// square, a star) instead of the two-sword straight line every time; lines stay for Bearer or when only 2 are free.
const SHAPE_MIN: [usize; 8] = [2, 3, 3, 3, 4, 4, 5, 5];
/// How long he waits for swords flying home before settling for a line or a solo stroke (no ally in danger).
const GATHER_WAIT: [usize; 8] = [0, 60, 55, 50, 45, 40, 35, 30];
/// Swords escorts leave free for engraving while an enemy champion is in reach (unless an ally is nearly dead).
const FORMATION_RESERVE: usize = 3;
/// Round 91 (Rian: "engraving random places with no one there"): he leads an enemy champion at most LEAD_CAP[rank] and
/// engraves a camp only when an enemy champion contests it.
const LEAD_CAP: [i64; 8] = [0, 8_000, 12_000, 16_000, 20_000, 24_000, 28_000, 32_000];
/// Round 97 (Rian: "the swords go a bit slow, the prediction is late ... give the flying swords a 1-time redirect"):
/// when a formation's first sword is REDIRECT_ETA ticks from its leg start, the whole shape shifts once onto the
/// target's fresh forecast, by at most REDIRECT_MAX[rank] (Imperial #1: REDIRECT_MAX_TOP); under REDIRECT_MIN it stays.
const REDIRECT_MAX: [i64; 8] = [20_000, 30_000, 40_000, 50_000, 60_000, 70_000, 80_000, 90_000];
const REDIRECT_MAX_TOP: i64 = 100_000;
const REDIRECT_MIN: i64 = 6_000;
const REDIRECT_ETA: usize = 20;

/// Round 97: how far rank `rank` (Imperial number `imperial`) may shift a formation in its one redirect.
fn redirect_max(rank: usize, imperial: Option<usize>) -> i64 {
    if rank >= 7 && imperial == Some(1) { REDIRECT_MAX_TOP } else { REDIRECT_MAX[rank.min(7)] }
}

/// Round 97: the shift that moves a formation centred at `center` onto `aim`, capped at `cap`; None when it's too small
/// to matter.
fn redirect_delta(center: (i64, i64), aim: (i64, i64), cap: i64) -> Option<(i64, i64)> {
    let (dx, dy) = (aim.0 - center.0, aim.1 - center.1);
    let len = ((dx * dx + dy * dy) as f64).sqrt();
    if len < REDIRECT_MIN as f64 { return None; }
    let k = (cap as f64 / len).min(1.0);
    Some(((dx as f64 * k) as i64, (dy as f64 * k) as i64))
}
const CONTEST_R: i64 = 70_000;
/// Round 92 (Rian: "help an ally or do anything with his swords anywhere, anytime ... nerf the damage when it's not at
/// Isliid"): no reach limit on plans or escorts, at most PER_ALLY swords on one teammate, and sword damage falls off
/// with distance from Isliid himself: full within FULL_R, down to FAR_PCT% at FAR_R and beyond. Utility is unchanged.
const PER_ALLY: usize = 2;
const FULL_R: i64 = 60_000;
const FAR_R: i64 = 200_000;
const FAR_PCT: usize = 25;
/// Round 93 (Rian: "longer cooldown on the swords' basic attacks the further Isliid is"): an escort's strike gap grows
/// from 1x within FULL_R to STRIKE_FAR_PCT% at FAR_R and beyond.
const STRIKE_FAR_PCT: usize = 250;
/// Round 94 (Rian: "still teleporting ... make the swords' base speed all slow, but the more they're in the air, the
/// faster"): every sword leaves at LAUNCH_SPEED and speeds up linearly to TOP_PCT% of its SPEED after RAMP_TICKS in the
/// air (returns 1.5x that). Basic-attack throws keep their full SPEED so the sword matches the instant hit. This
/// replaces round 93's slowdown with distance from Isliid.
const LAUNCH_SPEED: i64 = 1_000;
const RAMP_TICKS: usize = 150;
const TOP_PCT: i64 = 60;

/// Linear from `near` % within FULL_R to `far` % at FAR_R and beyond.
fn falloff(distance: i64, near: usize, far: usize) -> usize {
    if distance <= FULL_R { return near; }
    if distance >= FAR_R { return far; }
    let t = distance - FULL_R;
    let span = FAR_R - FULL_R;
    (near as i64 + (far as i64 - near as i64) * t / span) as usize
}

/// Round 94: sword `i`'s speed in `mode` after `air` ticks in flight.
fn sword_speed(i: usize, mode: SwordMode, air: usize) -> i64 {
    if mode == SwordMode::Throw { return SPEED[i]; }
    let mut top = SPEED[i] * TOP_PCT / 100;
    if mode == SwordMode::Return { top = top * RETURN_MULT / RETURN_DIV; }
    let top = top.max(LAUNCH_SPEED);
    LAUNCH_SPEED + (top - LAUNCH_SPEED) * air.min(RAMP_TICKS) as i64 / RAMP_TICKS as i64
}

/// Round 94: ticks to fly `distance` in `mode`, leaving with `air` ticks already in flight (the same per-tick steps
/// update_swords takes; capped at 600).
fn flight_ticks(i: usize, mode: SwordMode, distance: i64, air: usize) -> usize {
    let (mut left, mut t) = (distance, 0);
    while left > 0 && t < 600 { left -= sword_speed(i, mode, air + t); t += 1; }
    t
}

/// Round 94: how long a sword has been in the air (a volley sword counts from when it leaves).
fn airtime(s: &SwordMotion, tick: usize) -> usize { tick.saturating_sub(s.air_since.max(s.wait_until)) }

/// Round 93: an escort's strike gap at `distance` between Isliid and the ally it guards.
fn strike_gap(rank: usize, distance: i64) -> usize {
    STRIKE_GAP[rank.min(7)] * falloff(distance, 100, STRIKE_FAR_PCT) / 100
}

fn dist(a: (i64, i64), b: (i64, i64)) -> i64 { (sqdist(a, b) as f64).sqrt() as i64 }

/// How many swords he wants in a formation: the crowd at the target, but at least his mastery's shape size.
fn desired_swords(rank: usize, density: usize) -> usize {
    let crowd = if density >= 4 { 7 } else if density >= 3 { 5 } else if density >= 2 { 3 } else { 2 };
    crowd.max(SHAPE_MIN[rank.min(7)])
}

/// A candidate formation's score: close to the wanted size, its role, reused live strokes, and (round 90) a mastery
/// bonus per sword so masters draw bigger engravings.
fn pattern_score(rank: usize, spec: Pattern, desired: usize, density: usize, allies: bool, reused: usize) -> i64 {
    let role = match spec.effect { 7|10|11|12 if allies => 20, 9 if density >= 2 => 25, 2|3|6 if density >= 2 => 12, _ => 0 };
    100 - (spec.swords as i64 - desired as i64).abs() * 15 + role + reused as i64 * 20
        - spec.swords as i64 * 4 + spec.swords as i64 * rank.min(7) as i64 * 3
}

/// The best catalogue formation he can draw with `free` swords (reused strokes needing none), by `pattern_score`.
fn choose_pattern(rank: usize, density: usize, free: usize, allies: bool, reused: impl Fn(usize) -> usize) -> Option<usize> {
    let desired = desired_swords(rank, density);
    candidate_patterns(rank).filter_map(|idx| {
        let spec = PATTERNS[idx];
        let r = reused(idx);
        (r < spec.swords && spec.swords - r <= free).then(|| (pattern_score(rank, spec, desired, density, allies, r), idx))
    }).max_by_key(|&(score, idx)| (score, std::cmp::Reverse(idx))).map(|(_, idx)| idx)
}

fn candidate_patterns(rank: usize) -> impl Iterator<Item = usize> {
    0..PATTERN_BUDGET[rank.min(7)]
}

#[derive(Clone, Copy)]
struct Pattern { name: &'static str, swords: usize, style: usize, effect: usize }

// Effect families: damage, bind, pull, push, speed, shred, weaken, guard,
// attack, burst, cooldown, heal, mixed. Geometry is generated by `pattern_legs`.
const PATTERNS: [Pattern; 30] = [
    Pattern{name:"Severing Line",swords:2,style:0,effect:0},
    Pattern{name:"Tripwire",swords:2,style:0,effect:1},
    Pattern{name:"Funnel",swords:3,style:1,effect:2},
    Pattern{name:"Expulsion",swords:3,style:2,effect:3},
    Pattern{name:"Corner Guard",swords:3,style:3,effect:1},
    Pattern{name:"Piercing Road",swords:3,style:4,effect:8},
    Pattern{name:"Execution Seal",swords:3,style:5,effect:5},
    Pattern{name:"Suppression Seal",swords:3,style:6,effect:6},
    Pattern{name:"Ambush Seal",swords:3,style:7,effect:4},
    Pattern{name:"Imperial Fortress",swords:4,style:8,effect:7},
    Pattern{name:"Marching Ground",swords:4,style:9,effect:4},
    Pattern{name:"Judgment Field",swords:4,style:10,effect:9},
    Pattern{name:"Siege Ground",swords:4,style:11,effect:8},
    Pattern{name:"Pursuit Seal",swords:4,style:12,effect:6},
    Pattern{name:"Rupture",swords:4,style:13,effect:9},
    Pattern{name:"Divine Intersection",swords:4,style:14,effect:0},
    Pattern{name:"Execution Point",swords:4,style:15,effect:9},
    Pattern{name:"Roadblock",swords:4,style:16,effect:3},
    Pattern{name:"Emperor's Blessing",swords:5,style:17,effect:10},
    Pattern{name:"Celestial Judgment",swords:5,style:18,effect:9},
    Pattern{name:"Sanctuary",swords:5,style:19,effect:11},
    Pattern{name:"Imperial Charge",swords:5,style:20,effect:4},
    Pattern{name:"Time of Judgment",swords:5,style:21,effect:6},
    Pattern{name:"Absolute Territory",swords:6,style:22,effect:12},
    Pattern{name:"Grand Execution",swords:6,style:23,effect:9},
    Pattern{name:"Imperial Maelstrom",swords:6,style:24,effect:2},
    Pattern{name:"Imperial Prison",swords:6,style:25,effect:1},
    Pattern{name:"Emperor's Domain",swords:7,style:26,effect:12},
    Pattern{name:"Heavenfall",swords:7,style:27,effect:9},
    Pattern{name:"King's Authority",swords:7,style:28,effect:12},
];

fn pattern_legs(style: usize, count: usize, center: (i64,i64), radius: i64) -> Vec<((i64,i64),(i64,i64))> {
    let point = |j:usize, n:usize, offset:f64, sx:f64, sy:f64| -> (i64,i64) {
        let a=std::f64::consts::TAU*j as f64/n as f64+offset;
        (center.0+(a.cos()*radius as f64*sx) as i64,center.1+(a.sin()*radius as f64*sy) as i64)
    };
    let mut nodes:Vec<(i64,i64)>=(0..count).map(|j|point(j,count,-std::f64::consts::FRAC_PI_2,1.0,1.0)).collect();
    match style {
        0 => nodes=vec![(center.0-radius,center.1),(center.0+radius,center.1)],
        1 => nodes=vec![(center.0-radius,center.1-radius/2),(center.0,center.1+radius),(center.0+radius,center.1-radius/2)],
        2 => nodes=vec![(center.0-radius,center.1+radius/2),(center.0,center.1-radius),(center.0+radius,center.1+radius/2)],
        3 => nodes=vec![(center.0-radius,center.1-radius),(center.0-radius,center.1+radius),(center.0+radius,center.1+radius)],
        4 => nodes=vec![(center.0-radius,center.1),(center.0,center.1),(center.0+radius,center.1)],
        6 => nodes=vec![(center.0,center.1+radius),(center.0-radius,center.1-radius),(center.0+radius,center.1-radius)],
        7 => nodes=vec![(center.0-radius,center.1-radius),(center.0-radius,center.1+radius),(center.0+radius,center.1+radius)],
        8 => nodes=vec![(center.0-radius,center.1-radius),(center.0+radius,center.1-radius),(center.0+radius,center.1+radius),(center.0-radius,center.1+radius)],
        9 => nodes=vec![(center.0-radius*3/2,center.1-radius/2),(center.0+radius*3/2,center.1-radius/2),(center.0+radius*3/2,center.1+radius/2),(center.0-radius*3/2,center.1+radius/2)],
        10 => nodes=vec![(center.0,center.1-radius),(center.0+radius,center.1),(center.0,center.1+radius),(center.0-radius,center.1)],
        11 => nodes=vec![(center.0-radius/2,center.1-radius),(center.0+radius/2,center.1-radius),(center.0+radius,center.1+radius),(center.0-radius,center.1+radius)],
        12 => nodes=vec![(center.0,center.1-radius),(center.0+radius/2,center.1),(center.0,center.1+radius),(center.0-radius,center.1)],
        13 => nodes=vec![(center.0-radius,center.1-radius),(center.0+radius,center.1+radius),(center.0+radius,center.1-radius),(center.0-radius,center.1+radius)],
        21 => nodes=vec![(center.0-radius,center.1-radius),(center.0+radius,center.1+radius),(center.0+radius,center.1-radius),(center.0-radius,center.1+radius),center],
        14 | 16 => nodes=vec![(center.0-radius,center.1),(center.0,center.1-radius),(center.0+radius,center.1),(center.0,center.1+radius)],
        15 => nodes=vec![(center.0-radius,center.1-radius),(center.0+radius,center.1+radius),(center.0+radius,center.1-radius),(center.0-radius,center.1+radius)],
        19 => nodes=vec![(center.0-radius,center.1+radius),(center.0-radius,center.1),(center.0,center.1-radius),(center.0+radius,center.1),(center.0+radius,center.1+radius)],
        20 => nodes=vec![(center.0-radius,center.1-radius/2),(center.0,center.1-radius/2),(center.0,center.1-radius),(center.0+radius,center.1),(center.0,center.1+radius)],
        24 => nodes=(0..count).map(|j|{let r=radius*(j as i64+1)/count as i64;point(j,count,-std::f64::consts::FRAC_PI_2,r as f64/radius as f64,r as f64/radius as f64)}).collect(),
        28 => nodes=vec![(center.0-radius,center.1),(center.0-radius*2/3,center.1-radius),(center.0-radius/3,center.1),(center.0,center.1-radius),(center.0+radius/3,center.1),(center.0+radius*2/3,center.1-radius),(center.0+radius,center.1)],
        _ => {}
    }
    let middle = |a:(i64,i64),b:(i64,i64)| ((a.0+b.0)/2,(a.1+b.1)/2);
    if style==0 {
        let m=middle(nodes[0],nodes[1]);
        return vec![(nodes[0],m),(m,nodes[1])];
    }
    if matches!(style,1|2|3|4) {
        let m=middle(nodes[0],nodes[1]);
        return vec![(nodes[0],m),(m,nodes[1]),(nodes[1],nodes[2])];
    }
    if style==14 || style==16 { return nodes.into_iter().map(|p|(p,center)).collect(); }
    if style==15 {
        let a=middle(nodes[0],nodes[1]); let b=middle(nodes[2],nodes[3]);
        return vec![(nodes[0],a),(a,nodes[1]),(nodes[2],b),(b,nodes[3])];
    }
    (0..count).map(|j|{
        let next=match style { 0 => 1-j,
            1|2 => if j+1<count {j+1} else {1},
            3|4|16|24|28 => (j+1)%count,
            15 => if j%2==0 {j+1} else {j-1},
            18|23|27 => (j+2)%count,
            _ => (j+1)%count };
        (nodes[j],nodes[next])
    }).collect()
}

static ATHLETES: Mutex<Option<HashMap<(u64, usize), usize>>> = Mutex::new(None);
static MEMORY: OnceLock<Mutex<MasterySession>> = OnceLock::new();

struct MasterySession {
    memory: Memory,
    pinned: HashMap<u64, Memory>,
    recorded: HashSet<String>,
}

fn mod_dir() -> Option<std::path::PathBuf> { crate::mod_dir() }

fn memory() -> &'static Mutex<MasterySession> {
    MEMORY.get_or_init(|| {
        let mut m = Memory::default();
        if let Some(dir) = mod_dir() {
            let main = dir.join("isliid_memory.txt");
            let pending = dir.join("isliid_pending.txt");
            m = Memory::parse(&std::fs::read_to_string(&main).unwrap_or_default());
            if let Ok(lines) = std::fs::read_to_string(&pending) {
                if !lines.trim().is_empty() {
                    m.merge(&lines);
                    m.meta.clear();
                    if std::fs::write(&main, m.render()).is_ok() {
                        let _ = std::fs::write(&pending, "");
                    }
                }
            }
        }
        Mutex::new(MasterySession { memory: m, pinned: HashMap::new(), recorded: HashSet::new() })
    })
}

fn rank_for(seed: u64, athlete: Option<usize>) -> (usize, Option<usize>) {
    let Ok(mut session) = memory().lock() else { return (0, None) };
    if !session.pinned.contains_key(&seed) {
        if session.pinned.len() > 256 { session.pinned.clear(); }
        let snapshot = session.memory.clone();
        session.pinned.insert(seed, snapshot);
    }
    session.pinned.get(&seed).unwrap().rank_for(athlete)
}

pub fn note_athlete(seed: u64, player: usize, athlete: usize) {
    if let Ok(mut g) = ATHLETES.lock() {
        let m = g.get_or_insert_with(HashMap::new);
        if m.len() > 4096 { m.clear(); }
        m.insert((seed, player), athlete);
    }
}

fn athlete_of(seed: u64, player: usize) -> Option<usize> {
    ATHLETES.lock().ok()?.as_ref()?.get(&(seed, player)).copied()
}

/// True once this game is in the session memory (round 93: the caller then stops calling it every tick).
fn record_game(seed: u64, player: usize, athlete: usize, match_id: u64) -> bool {
    let key = format!("{seed:x}.{player}.{athlete}");
    let Ok(mut session) = memory().lock() else { return false };
    if session.recorded.len() > 200_000 { session.recorded.clear(); }
    if !session.recorded.insert(key.clone()) { return true; }
    let prefix = if match_id == SimOriginV1::NONE { "x".to_string() } else { match_id.to_string() };
    let line = format!("g {prefix}.{seed:x}-{player} {athlete}\n");
    session.memory.merge(&line);
    let Some(dir) = mod_dir() else { return true };
    use std::io::Write;
    let _ = std::fs::OpenOptions::new().create(true).append(true)
        .open(dir.join("isliid_pending.txt"))
        .and_then(|mut f| f.write_all(line.as_bytes()));
    true
}

fn record_result(seed: u64, player: usize, athlete: usize, match_id: u64,
                 tick: usize, result: (i64, i64, i64, i64, i64)) {
    let Some(dir) = mod_dir() else { return };
    let prefix = if match_id == SimOriginV1::NONE { "x".to_string() } else { match_id.to_string() };
    let sig = format!("{prefix}.{seed:x}-{player}");
    let line = format!("r {sig} {athlete} {tick} {} {} {} {} {}\n",
        result.0, result.1, result.2, result.3, result.4);
    // round 93: both simulations of a match write the same line; the second copy is skipped (every reader keeps the
    // latest tick of a game, so the points are the same)
    static LAST: Mutex<Option<HashMap<String, String>>> = Mutex::new(None);
    if let Ok(mut last) = LAST.lock() {
        let last = last.get_or_insert_with(HashMap::new);
        if last.len() > 4096 { last.clear(); }
        if last.get(&sig) == Some(&line) { return; }
        last.insert(sig, line.clone());
    }
    use std::io::Write;
    let _ = std::fs::OpenOptions::new().create(true).append(true)
        .open(dir.join("isliid_pending.txt"))
        .and_then(|mut f| f.write_all(line.as_bytes()));
}

fn point(sim: &StableSim<'_>, input: InputTargetV1) -> Option<(i64, i64)> {
    match input.kind {
        1 => sim.get_entity(input.target_id).map(|e| { let p = e.pos(); (p.0 as i64, p.1 as i64) }),
        3 => Some((input.x as i64, input.y as i64)),
        _ => None,
    }
}

fn signal(sim: &mut StableSim<'_>, caster: usize, input: InputTargetV1, command: &str) {
    if let Some((x, y)) = point(sim, input) {
        sim.add_buff(caster, &timed(&format!("il_{command}_{x}_{y}_{}", sim.tick()), 5));
    }
}

pub struct Guidance;
impl StableEffectType for Guidance {
    fn apply(&self, sim: &mut StableSim<'_>, _rng: u64, caster: usize, input: InputTargetV1) {
        signal(sim, caster, input, "draw");
    }
    fn expected_damage(&self, stat: &StatV1) -> (usize, usize) { (stat.attack / 2, 0) }
}

pub struct Recall;
impl StableEffectType for Recall {
    fn apply(&self, sim: &mut StableSim<'_>, _rng: u64, caster: usize, input: InputTargetV1) {
        signal(sim, caster, input, "recall");
    }
    fn expected_damage(&self, stat: &StatV1) -> (usize, usize) { (stat.attack / 3, 0) }
}

pub struct Manifest;
impl StableEffectType for Manifest {
    fn apply(&self, sim: &mut StableSim<'_>, _rng: u64, caster: usize, _input: InputTargetV1) {
        sim.add_buff(caster, &timed("il_manifest", 5));
    }
    fn expected_damage(&self, stat: &StatV1) -> (usize, usize) { (stat.attack * 2, 0) }
}

pub struct Scar;
impl StableEffectType for Scar {
    fn apply(&self, sim: &mut StableSim<'_>, _rng: u64, caster: usize, input: InputTargetV1) {
        if let Some(p) = point(sim, input) { Isliid::fx(sim, caster, "scar", p, 10); }
    }
}

#[derive(Clone, Copy)]
struct Anchor { x: i64, y: i64, until: usize }

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum SwordMode { Orbit, Stage, Planted, Ready, Throw, Draw, Strike, Return }

#[derive(Clone)]
struct SwordMotion {
    pos: (i64, i64),
    goal: (i64, i64),
    leg_from: (i64, i64),
    mode: SwordMode,
    holder: Option<usize>,
    last_ally: Option<usize>,
    target: Option<usize>,
    path: Vec<(i64, i64)>,
    pending_draw: Vec<(i64, i64)>,
    activate_on_arrival: bool,
    ready_at: usize,
    auto_owned: bool,
    plan_id: Option<u64>,
    waypoint: usize,
    planned: i64,
    travelled: i64,
    mark_start_id: u64,
    attack_at: usize,
    return_hits: HashSet<usize>,
    // round 88: one visual at a time: the tick its current visual (a flight segment or a grounded frame) ends, and
    // the one-shot effects due (1 launch, 2 recall snap, 4 plant impact)
    vis_until: usize,
    fx_due: u8,
    // round 88: when it went idle on the ground (0 = not idle), its escort lease, the brain-free lock
    idle_since: usize,
    escort_until: usize,
    locked_until: usize,
    // round 92: a formation sword waits here (no move, no visual) until its turn in the volley
    wait_until: usize,
    // round 94: the tick it took off (its speed grows with the time since)
    air_since: usize,
    // round 97: it left Imperial's black hole (drawn coming out of it)
    from_hole: bool,
}

impl Default for SwordMotion {
    fn default() -> Self {
        Self { pos: (0, 0), goal: (0, 0), leg_from: (0, 0), mode: SwordMode::Orbit,
            holder: None, last_ally: None, target: None, path: Vec::new(), pending_draw: Vec::new(),
            activate_on_arrival: false, ready_at: 0,
            auto_owned: false, plan_id: None,
            waypoint: 0, planned: 0, travelled: 0, mark_start_id: 0, attack_at: 0,
            return_hits: HashSet::new(), vis_until: 0, fx_due: 0, idle_since: 0, escort_until: 0, locked_until: 0, wait_until: 0, air_since: 0, from_hole: false }
    }
}

#[derive(Clone)]
struct EngravingMark {
    // round 89: the tick its formation (or solo stroke) fired; the leg flares for FLARE_TICKS after
    lit: usize,
    // round 93: the tick it was drawn (its age; `until` is cut short when its formation fires)
    born: usize,
    sword: usize,
    from: (i64, i64),
    to: (i64, i64),
    until: usize,
    id: u64,
}

#[derive(Clone)]
struct FormationPlan {
    id: u64,
    pattern: usize,
    center: (i64,i64),
    radius: i64,
    legs: Vec<((i64,i64),(i64,i64))>,
    until: usize,
    completed: bool,
    // round 97: the enemy champion it was aimed at (None for a camp or a shape read from live marks), and whether its
    // one mid-flight redirect is spent
    target: Option<usize>,
    redirected: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum FlagPhase { Planned, Drawing, Complete, Cancelled }

#[derive(Clone)]
struct EngravingFlag {
    id: u64,
    center: (i64, i64),
    tag: String,
    phase: FlagPhase,
    until: usize,
}

#[derive(Clone)]
pub struct Isliid {
    anchors: [Option<Anchor>; 7],
    swords: [SwordMotion; 7],
    engravings: Vec<EngravingMark>,
    next_mark: u64,
    activated: HashSet<u64>,
    formations: Vec<FormationPlan>,
    flags: Vec<EngravingFlag>,
    next_plan_id: u64,
    empowerment: usize,
    last_think: usize,
    observed: HashMap<usize, ((i64, i64), usize)>,
    selected: usize,
    grabbed: Option<usize>,
    rank: Option<usize>,
    imperial: Option<usize>,
    shown: Option<String>,
    arsenal_shown: [Option<(usize, bool)>; 7],
    holder_shown: [Option<(usize,usize)>; 7],
    selected_shown: Option<usize>,
    // round 97: the black hole buff shown above Imperial's head
    hole_shown: Option<String>,
    last_mark: usize,
    marks: HashSet<String>,
    dark_stacks: HashMap<usize, (usize, usize)>,
    last_result: Option<(i64, i64, i64, i64, i64)>,
    result_at: usize,
    want_at: [usize; 2],
    // round 90: when he first found too few swords for a real shape (0 = not waiting)
    short_since: usize,
    gathering: bool,
    prepared_at: usize,
    next_plan_at: usize,
    base_hit_ready: bool,
    base_hit_until: usize,
    base_ready_at: usize,
    native_hit: bool,
    ally_observed: HashMap<usize, ((i64, i64), usize)>,
    aura_active: HashSet<(usize, usize)>,
    aura_visual_at: HashMap<(usize, usize), (bool, usize, usize)>,
    aura_base_shown: HashMap<usize, (bool, usize)>,
    // round 91: the one sword aura shown on each champion (sword, ally, rank); the stat buffs stay one per sword
    aura_sword_shown: HashMap<usize, (usize, bool, usize)>,
    // round 93: this game is recorded in the mastery memory (record_game ran every tick after 30 s)
    recorded: bool,
    badge_for: Option<(Option<usize>, Option<usize>)>,
    // round 93: per sword, until when its strokes count as committed (see COMMIT_LIFE)
    commit_until: [usize; 7],
}

impl Default for Isliid {
    fn default() -> Self {
        Self { anchors: [None; 7], selected: 0, grabbed: None, rank: None, imperial: None,
            shown: None, arsenal_shown: [None; 7], holder_shown: [None; 7], selected_shown: None, hole_shown: None, last_mark: 0,
            marks: HashSet::new(), dark_stacks: HashMap::new(),
            last_result: None, result_at: 0, want_at: [0; 2], short_since: 0, gathering: false,
            prepared_at: 0, next_plan_at: 0, base_hit_ready: false, base_hit_until: 0,
            base_ready_at: 0, native_hit: false,
            ally_observed: HashMap::new(), aura_active: HashSet::new(), aura_visual_at: HashMap::new(),
            aura_base_shown: HashMap::new(), aura_sword_shown: HashMap::new(), recorded: false, badge_for: None, commit_until: [0; 7],
            swords: std::array::from_fn(|_| SwordMotion::default()), engravings: Vec::new(),
            next_mark: 0, activated: HashSet::new(), formations: Vec::new(),
            flags: Vec::new(), next_plan_id: 0,
            empowerment: 0, last_think: 0,
            observed: HashMap::new() }
    }
}

fn near(a: (i64, i64), b: (i64, i64), radius: i64) -> bool {
    sqdist(a, b) <= sq(radius)
}

fn damage_pct(distance: i64) -> usize {
    if distance <= FULL_R { 100 }
    else if distance >= FAR_R { FAR_PCT }
    else { 100 - ((distance-FULL_R) as usize * (100-FAR_PCT) / (FAR_R-FULL_R) as usize) }
}

fn aura_part(amount: usize, count: usize) -> usize {
    amount.div_ceil(count.max(1))
}

fn on_segment(p: (i64, i64), a: (i64, i64), b: (i64, i64), width: i64) -> bool {
    let (dx, dy) = ((b.0 - a.0) as f64, (b.1 - a.1) as f64);
    let len = dx * dx + dy * dy;
    if len < 1.0 { return near(p, a, width); }
    let t = (((p.0 - a.0) as f64 * dx + (p.1 - a.1) as f64 * dy) / len).clamp(0.0, 1.0);
    let q = (a.0 as f64 + t * dx, a.1 as f64 + t * dy);
    ((p.0 as f64 - q.0).powi(2) + (p.1 as f64 - q.1).powi(2)) <= (width * width) as f64
}

impl Isliid {
    fn rank(&self) -> usize { self.rank.unwrap_or(0) }

    fn track_result(&mut self, sim: &StableSim<'_>, player: usize, entity: usize) {
        let (Some(athlete), Some(me)) = (athlete_of(sim.seed(), player), sim.get_entity(entity)) else { return };
        let team = me.team().min(1);
        let mut towers = [0_i64; 2];
        let mut nexus = [100_i64; 2];
        let mut best = [i128::MAX; 2];
        for index in 0..sim.tower_count() {
            let Some(tower) = sim.get_entity(sim.tower_id_at(index)) else { continue };
            let side = tower.team().min(1);
            if tower.is_alive() { towers[side] += 1; }
            let pos = tower.pos();
            let home = if side == 0 { (96_000, 864_000) } else { (864_000, 96_000) };
            let distance = sqdist((pos.0 as i64, pos.1 as i64), home);
            if distance <= sq(70_000) && distance < best[side] {
                best[side] = distance;
                let (hp, maximum) = tower.hp();
                nexus[side] = if !tower.is_alive() || maximum == 0 { 0 } else { (hp * 100 / maximum) as i64 };
            }
        }
        let result = (towers[team], towers[1 - team], nexus[team], nexus[1 - team], sim.score_diff(team) as i64);
        if Some(result) != self.last_result || sim.tick() >= self.result_at + 600 {
            self.last_result = Some(result);
            self.result_at = sim.tick();
            record_result(sim.seed(), player, athlete, sim.sim_origin().unwrap_or_default().match_id,
                          sim.tick(), result);
        }
    }

    fn fx(sim: &mut StableSim<'_>, entity: usize, tag: &str, p: (i64, i64), time: u64) {
        crate::fx_point(sim, &format!("tfm2_isliid_emperor_{tag}"), entity, p.0, p.1, time);
    }

    /// Round 88: every sword has exactly one visual. Flying (round 96): a Spirit comet at its heading, a point effect at
    /// its live position every fly_cadence(speed) ticks; grounded: a 2-frame pair of the planted / ready loop every
    /// PAIR_STEP ticks; orbiting: the arsenal buff on its holder (show()). Launch, recall-snap and plant-impact effects
    /// play once on the change.
    fn update_visuals(&mut self, sim: &mut StableSim<'_>, entity: usize) {
        let tick=sim.tick();
        let rank=self.rank();
        let art=if rank>=7 && self.imperial==Some(1) {8} else {rank};   // round 96: Imperial #1's prismatic comet
        let Some(me)=sim.get_entity(entity).map(|e|{let p=e.pos();(p.0 as i64,p.1 as i64)}) else {return};
        let (imperial,top)=(rank>=7,art==8);
        let hole=(me.0+HEAD_LIFT.0,me.1+HEAD_LIFT.1);
        for i in 0..7 {
            if tick<self.swords[i].wait_until {continue}   // round 92: its launch (and launch effect) is still to come
            let due=std::mem::take(&mut self.swords[i].fx_due);
            let pos=self.swords[i].pos;
            // round 97: at Imperial the swords leave the black hole and land (or come home) through wormholes
            if due & FX_LAUNCH != 0 {
                if self.swords[i].from_hole { Self::fx(sim,entity,&wormhole("out",i,top),hole,0); }
                else { Self::fx(sim,entity,&format!("{}_launch",SWORDS[i]),pos,0); }
            }
            if due & FX_RECALL != 0 { Self::fx(sim,entity,&format!("{}_recall",SWORDS[i]),pos,0); }
            if due & FX_IMPACT != 0 {
                if imperial { Self::fx(sim,entity,&wormhole("in",i,top),pos,0); }
                else { Self::fx(sim,entity,&format!("{}_impact",SWORDS[i]),pos,0); }
            }
            if due & FX_INTO_HOLE != 0 { Self::fx(sim,entity,&wormhole("in",i,top),hole,0); }
            if tick<self.swords[i].vis_until {continue}
            let s=&self.swords[i];
            let step=match visual_for(s.mode,s.path.is_empty()) {
                Visual::Flying(_) => fly_cadence(sword_speed(i,s.mode,airtime(s,tick))),
                Visual::Grounded(_) => PAIR_STEP,   // round 91: two frames per emission
                Visual::Orbit => FRAME_STEP,
            };
            let life=step-(tick%step);
            match visual_for(self.swords[i].mode,self.swords[i].path.is_empty()) {
                Visual::Orbit => {}
                Visual::Grounded(ready) => {
                    let tag=format!("{}_rank{rank}_{}_pair{}",SWORDS[i],if ready {"ready"} else {"planted"},(tick/PAIR_STEP)%(PLANTED_FRAMES/2));
                    Self::fx(sim,entity,&tag,pos,life as u64);
                    self.swords[i].vis_until=tick+life;
                }
                Visual::Flying(_) => {
                    // round 96: the comet where the sword is, heading where it's going (along its leg once on the goal)
                    let s=&self.swords[i];
                    let heading=if near(pos,s.goal,1) {fly_angle(s.leg_from,s.goal)} else {fly_angle(pos,s.goal)};
                    // round 97: drawn coming out of the black hole (or going back into it), blending to its true path
                    let lift=if !imperial {(0,0)}
                        else if s.mode==SwordMode::Return && s.holder.is_none_or(|h|h==entity) {hole_lift(dist(pos,s.goal))}
                        else if s.from_hole && s.mode!=SwordMode::Return {hole_lift(s.travelled)} else {(0,0)};
                    Self::fx(sim,entity,&comet_tag(i,art,heading,step,tick),(pos.0+lift.0,pos.1+lift.1),life as u64);
                    self.swords[i].vis_until=tick+life;
                }
            }
        }
    }

    fn native_damage(&mut self, sim: &mut StableSim<'_>, caster: usize, target: usize, damage: usize) {
        self.native_hit = true;
        sim.deal_damage(caster, target, damage, 0, AttackTypeV1::Skill);
        self.native_hit = false;
    }

    /// Round 92: measured from Isliid himself (it was the nearest ally host), so far help hits softly.
    fn remote_damage(&self, sim: &StableSim<'_>, caster: usize, at: (i64,i64), damage: usize) -> usize {
        let Some(source)=sim.get_entity(caster) else {return damage*FAR_PCT/100};
        let p=source.pos();
        let distance=(sqdist((p.0 as i64,p.1 as i64),at) as f64).sqrt() as i64;
        damage.saturating_mul(damage_pct(distance))/100
    }

    // Short visual frames are repainted from the live ledger. A staging flight
    // never enters this ledger, so it cannot leave an engraving by accident.
    fn render_marks(&self, sim: &mut StableSim<'_>, entity: usize) {
        for (name, p, life) in self.mark_sprites(sim.tick()) { Self::fx(sim, entity, &name, p, life); }
    }

    /// Round 90: the scar sprites due this tick, as (name, point, life). Each sprite covers SCAR_STEP of its stroke
    /// (centred on its piece) and lives as long as its shimmer loop, so a stroke is re-emitted every SCAR_HOT_EVERY
    /// ticks while fresh (it was every 4 ticks with twice the sprites), then every SCAR_COOL_EVERY ticks as a cooled,
    /// static groove. New, newly lit and newly cooled marks are emitted at once with the life left to the cadence, so
    /// nothing appears late; a stroke being drawn is repainted every 12 ticks (round 92).
    fn mark_sprites(&self, tick: usize) -> Vec<(String, (i64, i64), u64)> {
        let mut out = Vec::new();
        if !tick.is_multiple_of(6) { return out; }
        let t = stroke_tier(self.rank());
        let mut plans: Vec<(usize, (i64,i64), (i64,i64), String, u64, usize, bool)> = Vec::new();
        // round 98: each stroke re-emits on its own phase (from the pass it was drawn on, or lit on), not on a cadence
        // shared by every stroke, so strokes drawn or lit apart stay apart (a fired formation's legs all re-emitting on
        // one tick was the spike)
        let pass = |t0: usize| t0.div_ceil(6) * 6;
        for m in &self.engravings {
            let age = tick.saturating_sub(m.born);
            let remaining = m.until.saturating_sub(tick);
            let lit = m.lit > 0 && tick >= m.lit;   // a leg lit later in the ripple doesn't flare before its turn
            let lit_pass = pass(m.lit);
            let (kind, life) = if lit && tick < lit_pass + FLARE_TICKS {
                if !(tick - lit_pass).is_multiple_of(SCAR_HOT_EVERY) { continue; }
                ("flare", SCAR_HOT_EVERY as u64)
            } else {
                let every = if age < SCAR_HOT { SCAR_HOT_EVERY } else { SCAR_COOL_EVERY };
                // a lit stroke stays on the phase it was lit on (its formation's ripple stays spread until it fades);
                // the others are split into two phases by sword, so strokes drawn together re-emit half at a time
                let base = if lit { lit_pass } else { pass(m.born) + 6 * (m.sword % 2) };
                let kind = if age < SCAR_HOT { "scar" } else { "scar_dim" };
                if tick < base {   // its very first pass, bridging to its phase
                    (kind, (base - tick) as u64)
                } else {
                    let since = tick - base;
                    // the first pass after it cools bridges to the cool cadence
                    let resume = age >= SCAR_HOT && age < SCAR_HOT + 6;
                    if !since.is_multiple_of(every) && !resume { continue; }
                    (kind, (every - since % every) as u64)
                }
            };
            let cool = kind == "scar_dim";
            // round 93: never outlives its stroke (a fired formation's strokes are gone 90 ticks after it fires)
            let life = life.min(remaining as u64).max(1);
            plans.push((m.sword, m.from, m.to, format!("{kind}_{}_t{t}_a{}", m.sword, trail_angle(m.from, m.to)),
                life, remaining, cool));
        }
        // round 92: a stroke being drawn is repainted every SCAR_HOT_EVERY ticks (was 6); the flying sword leads it
        if tick.is_multiple_of(SCAR_HOT_EVERY) {
            for (i, s) in self.swords.iter().enumerate().filter(|(_, s)| s.mode == SwordMode::Draw) {
                plans.push((i, s.leg_from, s.pos, format!("scar_{i}_t{t}_a{}", trail_angle(s.leg_from, s.pos)), SCAR_HOT_EVERY as u64, MARK_LIFE, false));
            }
        }
        let pieces = |from: (i64,i64), to: (i64,i64)| (((sqdist(from, to) as f64).sqrt() / SCAR_STEP as f64).ceil() as usize).clamp(1, 40);
        let total: usize = plans.iter().map(|p| pieces(p.1, p.2)).sum();
        for (sword, from, to, name, life, remaining, cool) in plans {
            let count = pieces(from, to);
            for k in 0..count {
                // over budget: the cooled grooves keep every second piece
                if cool && total > SCAR_BUDGET && k % 2 == 1 { continue; }
                if remaining < 60 && (k * 17 + sword * 7) % 60 >= remaining { continue; }
                let num = 2 * k as i64 + 1;
                let den = 2 * count as i64;
                let p = (from.0 + (to.0 - from.0) * num / den, from.1 + (to.1 - from.1) * num / den);
                out.push((name.clone(), p, life));
            }
        }
        out
    }

    fn position(&self, sim: &StableSim<'_>, entity: usize, sword: usize) -> (i64, i64) {
        let s = &self.swords[sword];
        if s.mode == SwordMode::Orbit {
            if let Some(e) = sim.get_entity(s.holder.unwrap_or(entity)) {
                let p = e.pos(); return (p.0 as i64, p.1 as i64);
            }
        }
        if let Some(a) = self.anchors[sword] { return (a.x, a.y); }
        s.pos
    }

    fn send(&mut self, sim: &StableSim<'_>, entity: usize, sword: usize,
            mode: SwordMode, path: Vec<(i64, i64)>, target: Option<usize>) {
        if path.is_empty() { return; }
        let from = self.position(sim, entity, sword);
        let mut last = from;
        let mut planned = 0;
        for &p in &path { planned += (sqdist(last, p) as f64).sqrt() as i64; last = p; }
        let imperial = self.rank() >= 7;
        let s = &mut self.swords[sword];
        // round 97: at Imperial a sword leaving his ring comes out of the black hole above his head
        if mode != SwordMode::Return {
            s.from_hole = imperial && s.mode == SwordMode::Orbit && s.holder.is_none_or(|h| h == entity);
        }
        // round 94: a new flight starts slow; re-aiming one already in the air keeps its speed
        if !matches!(visual_for(s.mode, s.path.is_empty()), Visual::Flying(_)) { s.air_since = sim.tick(); }
        s.pos = from; s.goal = path[0]; s.leg_from = from; s.mode = mode; s.path = path;
        s.fx_due |= if mode == SwordMode::Return { FX_RECALL } else { FX_LAUNCH };
        s.waypoint = 0; s.planned = planned; s.travelled = 0; s.target = target; s.wait_until = 0;
        if mode == SwordMode::Draw { s.mark_start_id = self.next_mark + 1; }
        if mode == SwordMode::Return { s.return_hits.clear(); }
        if mode != SwordMode::Stage { s.pending_draw.clear(); }
        if mode != SwordMode::Stage { s.activate_on_arrival = false; }
        if matches!(mode, SwordMode::Draw | SwordMode::Throw | SwordMode::Stage) { s.holder = None; }
        self.anchors[sword] = None;
    }

    fn cancel_marks(&mut self, sword: usize, tick: usize, instant: bool) {
        if let Some(id)=self.swords[sword].plan_id {
            if let Some(flag)=self.flags.iter_mut().find(|f|f.id==id) {
                flag.phase=FlagPhase::Cancelled;
                flag.until=tick+if instant {30} else {60};
            }
        }
        for m in &mut self.engravings {
            if m.sword == sword { m.until = m.until.min(tick + if instant { 0 } else { 60 }); }
        }
        self.commit_until[sword] = self.commit_until[sword].min(tick + if instant { 0 } else { 60 });
        if instant { self.engravings.retain(|m| m.until > tick); }
    }

    fn mark_segment(&mut self, sim: &mut StableSim<'_>, entity: usize, sword: usize,
                    from: (i64, i64), to: (i64, i64)) {
        if near(from, to, 1_000) { return; }
        self.next_mark += 1;
        let tick = sim.tick();
        // round 93: a leg of a live formation stays at least until the plan's deadline, so the other legs (far swords
        // fly slower now) can still finish it
        let plan_until = self.swords[sword].plan_id
            .and_then(|id| self.formations.iter().find(|p| p.id == id && !p.completed)).map_or(0, |p| p.until + 30);
        self.engravings.push(EngravingMark { lit: 0, born: tick, sword, from, to,
            until: (tick + MARK_LIFE).max(plan_until), id: self.next_mark });
        self.commit_until[sword] = self.commit_until[sword].max(tick + COMMIT_LIFE);
        if self.engravings.len() > 512 { self.engravings.drain(..self.engravings.len() - 512); }
        let _ = (sim, entity);
    }

    fn solo_flag(&mut self, sword: usize, from: (i64,i64), to: (i64,i64), tick: usize) {
        self.next_plan_id+=1;
        let id=self.next_plan_id;
        self.swords[sword].plan_id=Some(id);
        self.flags.push(EngravingFlag{id,center:((from.0+to.0)/2,(from.1+to.1)/2),
            tag:format!("solo_{sword}"),phase:FlagPhase::Planned,until:tick+ANCHOR_LIFE});
    }

    fn render_flags(&mut self, sim: &mut StableSim<'_>, entity: usize) {
        let tick=sim.tick();
        self.flags.retain(|f|f.until>tick);
        if !tick.is_multiple_of(6) {return}
        for flag in &self.flags {
            // round 93: a still logo (planned, drawing, cancelled) every 12 ticks; only the completed pop steps every 6
            let still=flag.phase!=FlagPhase::Complete;
            if still && !tick.is_multiple_of(12) {continue}
            // round 88: a 24 x 24 effect logo just above the plan (it used to be a 120 x 48 text banner on top of it)
            let Some(mut name)=logo_name(&flag.tag,flag.phase) else {continue};
            if flag.phase==FlagPhase::Complete {
                // round 89: the completed logo pops (it's set to last 120 ticks when it completes)
                let since=(tick+120).saturating_sub(flag.until);
                name=format!("{name}_f{}",(since/6).min(POP_FRAMES-1));
            }
            Self::fx(sim,entity,&name,(flag.center.0,flag.center.1-20_000),if still {12} else {6});
        }
    }

    fn nearest(&self, p: (i64, i64), radius: i64) -> Option<usize> {
        self.anchors.iter().enumerate().filter_map(|(i, a)| a.map(|v| (i, sqdist(p, (v.x, v.y)))))
            .filter(|(_, d)| *d <= sq(radius)).min_by_key(|(_, d)| *d).map(|(i, _)| i)
    }

    fn available(&self, from: usize) -> Option<usize> {
        (0..7).map(|d| (from + d) % 7)
            .find(|&i| self.swords[i].mode == SwordMode::Orbit && self.swords[i].holder.is_none())
    }

    fn attack_sword(&self, health: (usize, usize), target_defence: usize, distance: i64) -> Option<usize> {
        if self.swords[self.selected].mode == SwordMode::Orbit && self.swords[self.selected].holder.is_none() { return Some(self.selected); }
        let usable = |i: usize| self.swords[i].mode == SwordMode::Orbit && self.swords[i].holder.is_none();
        if health.1 > 0 && health.0 * 2 < health.1 && usable(4) { return Some(4); }
        if distance > 50_000 && usable(0) { return Some(0); }
        if target_defence > 70 && usable(2) { return Some(2); }
        if distance <= MELEE && usable(3) { return Some(3); }
        [1, 5, 2, 4, 0, 3, 6].into_iter().find(|&i| usable(i))
    }

    fn arm(&mut self, sim: &StableSim<'_>, sword: usize) {
        let p=self.swords[sword].pos;
        self.swords[sword].mode=SwordMode::Ready;
        self.swords[sword].ready_at=sim.tick();
        self.swords[sword].activate_on_arrival=false;
        self.anchors[sword]=Some(Anchor{x:p.0,y:p.1,until:sim.tick()+ANCHOR_LIFE});
    }

    fn draw(&mut self, sim: &mut StableSim<'_>, entity: usize, p: (i64, i64)) {
        let Some(me) = sim.get_entity(entity) else { return };
        let origin = { let q = me.pos(); (q.0 as i64, q.1 as i64) };
        if near(p, origin, 25_000) {
            // Seven ground directions directly select seven sword identities.
            // This also works for a planted sword, which can then be recalled.
            let delta = (p.0-origin.0, p.1-origin.1);
            let sectors = [(-27,-17),(0,-33),(33,-19),(-40,17),(45,17),(-23,46),(27,48)];
            if !near(p, origin, 4_000) {
                self.selected = sectors.iter().enumerate().max_by(|(_,a),(_,b)| {
                    let score = |v: &(i64,i64)| {
                        let dot = (delta.0*v.0 + delta.1*v.1) as f64;
                        dot / (v.0 as f64).hypot(v.1 as f64)
                    };
                    score(a).partial_cmp(&score(b)).unwrap_or(std::cmp::Ordering::Equal)
                }).map_or(self.selected, |(i,_)| i);
            }
            self.grabbed = None;
            return;
        }
        if let Some(i)=(0..7).filter(|&j| {
            let s=&self.swords[j];
            matches!(s.mode,SwordMode::Planted|SwordMode::Stage|SwordMode::Ready)
                && s.path.is_empty() && !self.reserved(j) && near(self.position(sim,entity,j),p,SNAP)
        }).min_by_key(|&j|sqdist(self.position(sim,entity,j),p)) {
            self.selected=i;
            if self.swords[i].mode!=SwordMode::Ready {self.arm(sim,i);}
            return;
        }
        // round 89: a press never takes a sword working on a formation (it used to redraw a Ready plan sword to the
        // press point, so the formation lost a leg); another free sword goes instead, or nothing does
        let Some(i)=self.press_sword(entity,sim.tick(),p,|j|self.position(sim,entity,j)) else {return};
        self.selected=i;
        match self.swords[i].mode {
            SwordMode::Ready => {
                let from=self.swords[i].pos;
                self.solo_flag(i,from,p,sim.tick());
                self.send(sim,entity,i,SwordMode::Draw,vec![p],None);
                self.swords[i].auto_owned=false;
            }
            SwordMode::Orbit|SwordMode::Planted|SwordMode::Stage if self.swords[i].path.is_empty() => {
                self.send(sim,entity,i,SwordMode::Stage,vec![p],None);
                self.swords[i].activate_on_arrival=true;
                self.swords[i].auto_owned=false;
            }
            _ => {}
        }
    }

    fn recall(&mut self, sim: &mut StableSim<'_>, entity: usize, p: (i64, i64)) {
        let Some(me) = sim.get_entity(entity) else { return };
        let home = { let q = me.pos(); (q.0 as i64, q.1 as i64) };
        let to_last = near(p,home,25_000);
        // round 89: only a sword with nothing left to do comes back (the press point is usually an enemy, and the
        // nearest sword to an enemy is usually one about to draw a formation's stroke)
        let Some(i) = self.recall_pick(to_last, p, |j| self.position(sim, entity, j)) else { return };
        self.swords[i].locked_until = sim.tick() + LOCK_TICKS;
        let last=self.swords[i].last_ally.filter(|&id|sim.get_entity(id).is_some_and(|e|e.is_alive()));
        let destination=if to_last {last} else {None};
        let to=destination.and_then(|id|sim.get_entity(id)).map(|e|{let q=e.pos();(q.0 as i64,q.1 as i64)}).unwrap_or(home);
        let replaced=self.swords[i].holder;
        let instant = self.swords[i].mode == SwordMode::Return;
        self.cancel_marks(i, sim.tick(), instant);
        self.send(sim, entity, i, SwordMode::Return, vec![to], None);
        self.swords[i].holder = destination;
        if let Some(ally)=replaced.filter(|&id|Some(id)!=destination) {
            if let Some(q)=sim.get_entity(ally).filter(|e|e.is_alive()).map(|e|e.pos()) {
                let team=me.team();
                let threatened=(0..sim.entity_count()).filter_map(|n|sim.entity_at(n))
                    .any(|e|e.is_alive() && e.team()!=team && e.is_champion() &&
                        sim.is_visible(team,e.id()) && {let v=e.pos();near((q.0 as i64,q.1 as i64),(v.0 as i64,v.1 as i64),110_000)});
                if threatened {
                    if let Some(j)=self.available((i+1)%7) {
                        self.send(sim,entity,j,SwordMode::Stage,vec![(q.0 as i64,q.1 as i64)],None);
                        self.swords[j].holder=Some(ally); self.swords[j].last_ally=Some(ally);
                    }
                }
            }
        }
        self.selected = i;
        self.grabbed = None;
    }

    fn solo(&mut self, sim: &mut StableSim<'_>, entity: usize, sword: usize, center: (i64, i64), length: i64) {
        let Some(me) = sim.get_entity(entity) else { return };
        let team = me.team();
        let attack = me.stat().attack;
        let committed = (0..7).filter(|&j| self.swords[j].mode == SwordMode::Draw ||
            self.commit_until[j] > sim.tick()).count().max(1);
        let quality = grade(solo_quality(self.rank(), self.imperial)).map_or(70, |g| g.1);
        let concentration = (130_000_i64 * 100 / length.max(65_000)).clamp(55, 125) as usize;
        let amp = if self.empowerment > 0 { self.empowerment -= 1; 150 } else { 100 };
        let strength = (quality * concentration * amp / 10_000 / committed).max(1);
        let radius = (12_000 + length / 5).clamp(14_000, 80_000);
        let start_id=self.swords[sword].mark_start_id;
        let targets: Vec<(usize, bool, (i64, i64))> = (0..sim.entity_count())
            .filter_map(|n| sim.entity_at(n))
            .filter(|e| e.is_alive() && e.is_champion())
            .filter_map(|e| { let p=e.pos(); let p=(p.0 as i64,p.1 as i64);
                self.engravings.iter().any(|m|m.sword==sword && m.id>=start_id &&
                    on_segment(p,m.from,m.to,radius/3)).then_some((e.id(),e.team()==team,p)) }).collect();
        let hit: Vec<(usize, bool, (i64, i64))> = targets.iter().copied()
            .filter(|&(_, ally, _)| matches!((sword, ally), (0|1|2|4|5, false) | (3|6, true))).collect();
        for (id, ally, p) in targets {
            match (sword, ally) {
                (0, false) => { sim.entity_set_invisible(id, 0); sim.add_buff(id, &timed("il_sky_mark", 90)); }
                (1, false) => { let mut b=timed("il_terra_mark", 100); b.move_speed_mult=-(strength as i32 / 3).max(8); sim.add_buff(id,&b); }
                (2, false) => { let mut b=timed("il_dark_mark", 150); b.defence_mult=-(strength as i32 / 3).max(8); sim.add_buff(id,&b); }
                (3, true) => { let mut b=timed("il_gale_mark", 120); b.move_speed_mult=(strength as i32 / 3).max(8); sim.add_buff(id,&b); }
                (4, false) => { let damage=self.remote_damage(sim,entity,
                    p,20 + attack * strength / 100); self.native_damage(sim,entity,id,damage);
                    sim.heal(entity,entity,damage/5); }
                (5, false) => { let q=((p.0*4+center.0)/5,(p.1*4+center.1)/5); sim.entity_set_pos(id,q.0.max(0) as u64,q.1.max(0) as u64); }
                (6, true) => { let mut b=timed("il_emperor_mark", 120); b.attack_mult=(strength as i32 / 5).max(4); sim.add_buff(id,&b); }
                _ => {}
            }
        }
        // round 89: the stroke visibly fires: its marks flare, a burst in its family's motif, a marker on everyone hit.
        // Round 98: not a leg of a live formation (the formation's one fire covers it; seven at once were the spike)
        if self.swords[sword].plan_id.is_some_and(|id| self.formations.iter().any(|p| p.id == id && !p.completed)) { return; }
        let (tick, t) = (sim.tick(), tier(self.rank()));
        for m in self.engravings.iter_mut().filter(|m| m.sword == sword && m.id >= start_id) { m.lit = tick; }
        Self::fx(sim, entity, &fire_name(SOLO_FAMILY[sword], t, radius), center, 0);
        for (id, _, _) in hit { crate::fx_unit(sim, &format!("tfm2_isliid_emperor_hitmark_{}_t{t}", FAMILIES[SOLO_FAMILY[sword]]), entity, id, 0); }
    }

    fn remote_hit(&mut self, sim: &mut StableSim<'_>, entity: usize, sword: usize, target: usize) {
        let (Some(me),Some(victim))=(sim.get_entity(entity),sim.get_entity(target)) else { return };
        if !victim.is_alive() || me.team()==victim.team() || !sim.is_visible(me.team(),target) { return; }
        let Some(holder)=self.swords[sword].holder.and_then(|id|sim.get_entity(id))
            .filter(|e|e.is_alive() && e.team()==me.team()) else {return};
        let h=holder.pos(); let v=victim.pos();
        if !near((h.0 as i64,h.1 as i64),(v.0 as i64,v.1 as i64),100_000) {return}
        let raw=25 + me.stat().attack * if sword==6 { 35 } else { 60 } / 100;
        let damage=self.remote_damage(sim,entity,(v.0 as i64,v.1 as i64),raw);
        self.native_damage(sim,entity,target,damage);
        match sword {
            0 => { sim.entity_set_invisible(target,0); },
            1 => { let mut b=timed("il_terra_remote",45); b.move_speed_mult=-20; sim.add_buff(target,&b); },
            2 => { let mut b=timed("il_dark_remote",120); b.defence_mult=-15; sim.add_buff(target,&b); },
            4 => { sim.heal(entity,entity,damage/5); },
            5 => { if let Some(e)=sim.get_entity(target) { let p=e.pos(); let q=self.swords[sword].pos;
                sim.entity_set_pos(target,((p.0 as i64*4+q.0)/5).max(0) as u64,((p.1 as i64*4+q.1)/5).max(0) as u64); } },
            _ => {}
        }
    }

    fn update_swords(&mut self, sim: &mut StableSim<'_>, entity: usize) {
        let tick=sim.tick();
        for i in 0..7 {
            if tick<self.swords[i].wait_until {continue}
            let mode=self.swords[i].mode;
            if mode==SwordMode::Orbit {
                let holder=self.swords[i].holder.unwrap_or(entity);
                if let Some(e)=sim.get_entity(holder).filter(|e| e.is_alive()) {
                    let p=e.pos(); self.swords[i].pos=(p.0 as i64,p.1 as i64);
                } else {
                    self.swords[i].holder=None;
                    self.reassign_or_return(sim,entity,i);
                }
                continue;
            }
            if mode==SwordMode::Ready && self.swords[i].path.is_empty() {
                if self.anchors[i].is_none() {
                    self.swords[i].mode=SwordMode::Planted;
                    self.swords[i].pending_draw.clear();
                    continue;
                }
                if tick>self.swords[i].ready_at && !self.swords[i].pending_draw.is_empty() {
                    let draw=std::mem::take(&mut self.swords[i].pending_draw);
                    if let Some(id)=self.swords[i].plan_id {
                        if let Some(f)=self.flags.iter_mut().find(|f|f.id==id) {f.phase=FlagPhase::Drawing;}
                    }
                    self.send(sim,entity,i,SwordMode::Draw,draw,None);
                    continue;
                }
            }
            if matches!(mode,SwordMode::Stage|SwordMode::Planted|SwordMode::Ready) && self.swords[i].path.is_empty() {
                continue;
            }
            if mode==SwordMode::Return {
                let holder=self.swords[i].holder.unwrap_or(entity);
                if let Some(e)=sim.get_entity(holder).filter(|e| e.is_alive()) {
                    let p=e.pos(); self.swords[i].goal=(p.0 as i64,p.1 as i64);
                } else {
                    // its host died on the way back: home now (no lag of a tick aimed at a corpse)
                    self.swords[i].holder=None;
                    if let Some(me)=sim.get_entity(entity) { let p=me.pos(); self.swords[i].goal=(p.0 as i64,p.1 as i64); }
                }
            }
            if mode==SwordMode::Stage {
                if let Some(holder)=self.swords[i].holder {
                    if let Some(e)=sim.get_entity(holder).filter(|e|e.is_alive()) {
                        let p=e.pos(); self.swords[i].goal=(p.0 as i64,p.1 as i64);
                    } else {
                        self.swords[i].holder=None;
                        self.reassign_or_return(sim,entity,i);
                        continue;
                    }
                }
            }
            let from=self.swords[i].pos;
            let goal=self.swords[i].goal;
            let d=(sqdist(from,goal) as f64).sqrt();
            // round 94: launches slow, faster the longer it's in the air
            let base=sword_speed(i,mode,airtime(&self.swords[i],tick));
            let arrived=d <= base as f64;
            let next=if arrived { goal } else { (from.0+((goal.0-from.0) as f64*base as f64/d) as i64,
                from.1+((goal.1-from.1) as f64*base as f64/d) as i64) };
            self.swords[i].pos=next;
            self.swords[i].travelled += (sqdist(from,next) as f64).sqrt() as i64;
            if !arrived { continue; }
            if mode==SwordMode::Draw {
                let start=self.swords[i].leg_from;
                self.mark_segment(sim,entity,i,start,next);
                self.swords[i].leg_from=next;
                if self.swords[i].waypoint+1 < self.swords[i].path.len() {
                    self.swords[i].waypoint+=1;
                    self.swords[i].goal=self.swords[i].path[self.swords[i].waypoint];
                    continue;
                }
                let length=self.swords[i].planned;
                self.swords[i].mode=SwordMode::Planted;
                self.swords[i].path.clear();
                if let Some(id)=self.swords[i].plan_id {
                    if let Some(f)=self.flags.iter_mut().find(|f|f.id==id && f.tag.starts_with("solo_")) {
                        f.phase=FlagPhase::Complete;f.until=tick+120;
                    }
                }
                self.anchors[i]=Some(Anchor{x:next.0,y:next.1,until:tick+ANCHOR_LIFE});
                self.solo(sim,entity,i,next,length);
                self.evaluate_formations(sim,entity);
            } else if mode==SwordMode::Strike {
                if let Some(target)=self.swords[i].target { self.remote_hit(sim,entity,i,target); }
                self.swords[i].target=None;
                self.swords[i].mode=SwordMode::Return;
                self.swords[i].path.clear();
                self.swords[i].return_hits.clear();
            } else if mode==SwordMode::Return {
                if self.rank()>=7 && self.swords[i].holder.is_none_or(|h|h==entity) { self.swords[i].fx_due|=FX_INTO_HOLE; }
                self.swords[i].mode=SwordMode::Orbit;
                self.swords[i].path.clear();
                self.swords[i].auto_owned=false;
                self.swords[i].plan_id=None;
                self.anchors[i]=None;
            } else {
                self.swords[i].path.clear();
                if mode==SwordMode::Throw { self.anchors[i]=Some(Anchor{x:next.0,y:next.1,until:tick+ANCHOR_LIFE}); }
                // round 98: a formation sword reaching its leg start lands quietly (seven landings at once added to the spike)
                if self.swords[i].holder.is_none() && !self.swords[i].activate_on_arrival { self.swords[i].fx_due |= FX_IMPACT; }
                if mode==SwordMode::Stage && self.swords[i].activate_on_arrival {
                    self.arm(sim,i);
                    continue;
                }
                if self.swords[i].holder.is_some() { self.swords[i].mode=SwordMode::Orbit; }
                else { self.swords[i].mode=if mode==SwordMode::Throw {SwordMode::Planted} else {SwordMode::Stage}; }
            }
        }
    }

    fn prune_formations(&mut self, tick: usize) {
        let marks=&self.engravings;
        let swords=&self.swords;
        let mut cancelled=Vec::new();
        self.formations.retain(|plan| {
            if plan.completed {return false}
            let valid=plan.until>tick && plan.legs.iter().all(|&(a,b)| {
                let matches=|from,to| (near(a,from,18_000) && near(b,to,18_000)) ||
                    (near(a,to,18_000) && near(b,from,18_000));
                marks.iter().any(|m|m.until>tick && matches(m.from,m.to)) ||
                    swords.iter().any(|s| {
                        if !s.is_committed() { return false; }
                        if s.mode==SwordMode::Draw { matches(s.leg_from,s.goal) }
                        else { s.pending_draw.first().is_some_and(|&to|matches(s.goal,to)) }
                    })
            });
            if !valid {cancelled.push(plan.id)}
            valid
        });
        for id in cancelled {
            if let Some(f)=self.flags.iter_mut().find(|f|f.id==id) {
                f.phase=FlagPhase::Cancelled;f.until=tick+60;
            }
        }
    }

    fn think(&mut self, sim: &mut StableSim<'_>, entity: usize) {
        let tick=sim.tick();
        if tick < self.last_think + THINK_TICKS[self.rank()] { return; }
        self.last_think=tick;
        let Some(me)=sim.get_entity(entity) else { return };
        let team=me.team();
        let my_pos=me.pos();
        let my_pos=(my_pos.0 as i64,my_pos.1 as i64);
        let allies: Vec<(usize,(i64,i64))>=(0..sim.entity_count()).filter_map(|n| sim.entity_at(n))
            .filter(|e| e.is_alive() && e.is_champion() && e.team()==team && e.id()!=entity)
            .map(|e| { let p=e.pos(); (e.id(),(p.0 as i64,p.1 as i64)) }).collect();
        let mut foes: Vec<(usize,(i64,i64),usize)>=(0..sim.entity_count()).filter_map(|n| sim.entity_at(n))
            .filter(|e| e.is_alive() && e.is_champion() && e.team()!=team && sim.is_visible(team,e.id()))
            .map(|e| { let p=e.pos(); (e.id(),(p.0 as i64,p.1 as i64),e.hp().0) }).collect();
        foes.sort_by_key(|(_,p,hp)| {
            let nearest=allies.iter().map(|(_,a)| sqdist(*p,*a)).chain(std::iter::once(sqdist(*p,my_pos))).min().unwrap_or(0);
            nearest/100_000 + *hp as i128 * 100
        });
        let ally_future:Vec<(usize,(i64,i64),(i64,i64),usize)> = allies.iter().map(|&(id,p)| {
            let future=forecast(p,self.ally_observed.get(&id).copied(),tick,LOOK_AHEAD[self.rank()],LEAD_CAP[self.rank()]);
            let health=sim.get_entity(id).map(|e|e.hp()).unwrap_or((1,1));
            let missing=if health.1==0 {0} else {100-health.0.saturating_mul(100)/health.1};
            (id,p,future,missing)
        }).collect();
        for &(id,p) in &allies { self.ally_observed.insert(id,(p,tick)); }
        self.ally_observed.retain(|_,(_,seen)|tick.saturating_sub(*seen)<600);

        let objective=(0..sim.entity_count()).filter_map(|n|sim.entity_at(n))
            .filter(|e|e.is_alive() && !e.is_champion() && !e.is_tower() &&
                !e.is_minion() && e.hp().1>=2_000 && sim.is_visible(team,e.id()))
            .filter(|e|{let p=e.pos();let p=(p.0 as i64,p.1 as i64);
                (near(my_pos,p,220_000) || ally_future.iter().any(|(_,_,future,_)|near(*future,p,220_000)))
                    && objective_ok(p,&allies)})
            .min_by_key(|e|{let p=e.pos();sqdist(my_pos,(p.0 as i64,p.1 as i64))})
            .map(|e|{let p=e.pos();(p.0 as i64,p.1 as i64)});

        self.prune_formations(tick);
        // round 88: urgent ally support first, ahead of the plan gap and the gathering
        self.assign_escorts(sim,entity,&ally_future,&foes,tick);
        for i in 0..7 {
            if !self.abandoned(i) {continue}
            let s=&self.swords[i];
            let nearby=allies.iter().filter(|(_,p)|near(*p,s.pos,100_000))
                .min_by_key(|(_,p)|sqdist(*p,s.pos)).map(|(id,_)|*id);
            let destination=nearby.and_then(|id|sim.get_entity(id))
                .map(|e|{let p=e.pos();(p.0 as i64,p.1 as i64)}).unwrap_or(my_pos);
            self.send(sim,entity,i,SwordMode::Return,vec![destination],None);
            self.swords[i].holder=nearby;
            self.swords[i].pending_draw.clear();
        }

        let urgent=foes.iter().any(|(_,p,_)| near(*p,my_pos,105_000) ||
            allies.iter().any(|(_,ally)|near(*p,*ally,105_000)));
        let remote=(0..7).filter(|&i|self.swords[i].mode!=SwordMode::Orbit ||
            self.swords[i].holder.is_some()).count();
        // When time permits, bring every blade back first. This is a choice,
        // not a mastery lock: all seven can be moved by every rank.
        if self.gathering && (urgent || objective.is_some()) { self.gathering=false; }
        if self.rank()>=3 && !urgent && objective.is_none() && remote>=2 &&
            !self.swords.iter().any(SwordMotion::is_committed) && !(0..7).any(|i|self.in_live_plan(i)) &&
            !self.gathering && tick>=self.prepared_at+300 {
            self.gathering=true;
            for i in 0..7 {
                if self.swords[i].mode==SwordMode::Orbit && self.swords[i].holder.is_none() {continue}
                if self.swords[i].is_activated() || self.reserved(i) { continue; }
                self.send(sim,entity,i,SwordMode::Return,vec![my_pos],None);
                self.swords[i].holder=None;
            }
            return;
        }
        if self.gathering {
            if remote>0 {return}
            self.gathering=false;
            self.prepared_at=tick+60;
        }
        if tick<self.prepared_at && !urgent && objective.is_none() {return}
        // Only observed positions enter the forecast. Invisible enemies never
        // become candidates, even if the simulation still exposes their entities.
        let candidates=[1,1,2,2,3,4,5,7][self.rank()];
        let prediction=foes.iter().take(candidates).map(|&(id,p,hp)| {
            let future=forecast(p,self.observed.get(&id).copied(),tick,LOOK_AHEAD[self.rank()],LEAD_CAP[self.rank()]);
            let personal=if near(future,my_pos,110_000) {55} else {0};
            let support=ally_future.iter().filter(|(_,_,ap,_)|near(future,*ap,115_000))
                .map(|(_,_,_,missing)|30+(*missing as i64/2)).max().unwrap_or(0);
            let contest=if objective.is_some_and(|o|near(o,future,110_000)) {28} else {0};
            let flank=if self.rank()>=5 && !near(p,future,20_000) {15} else {0};
            let ready=sim.get_entity(id).and_then(|e|(0..sim.player_count()).filter_map(|n|sim.player_at(n))
                .find(|owner|owner.champion().is_some_and(|c|c.id()==e.id()))
                .and_then(|owner|owner.cooldowns()))
                .map_or(0,|(_,skill,_,ult)|if ult<LOOK_AHEAD[self.rank()] as usize || skill<30 {15} else {0});
            let score=personal.max(support)+contest+flank+ready-(hp as i64/120).min(20);
            (id,future,score)
        }).max_by_key(|(_,_,score)|*score).map(|(id,p,_)|(id,p));
        for (id,p,_) in &foes { self.observed.insert(*id,(*p,tick)); }
        if self.observed.len()>64 { self.observed.retain(|_,(_,t)| tick.saturating_sub(*t)<600); }

        // round 91: a camp only when an enemy champion contests it (it used to engrave empty camps)
        let target=prediction.map(|(id,_)|id);
        if let Some(center)=prediction.map(|(_,p)|p).or(objective.filter(|o|contested(*o,&foes))) {
            if tick<self.next_plan_at {return}
            // Higher mastery tests more possible placements and uses more of
            // the Arsenal in a teamfight; no sword count is rank-locked.
            let density=foes.iter().filter(|(_,p,_)| near(*p,center,130_000)).count();
            if self.formations.iter().any(|p|!p.completed && p.until>tick && near(p.center,center,100_000)) {return}
            // round 89: finish setting up one formation before planning the next (its swords still on their way)
            if (0..7).any(|i|self.in_live_plan(i) && !self.swords[i].pending_draw.is_empty()) {return}
            let available:Vec<usize>=(0..7).filter(|&i| {
                let s=&self.swords[i];
                if tick<s.locked_until || self.reserved(i) {return false}
                (s.mode==SwordMode::Orbit && s.holder.is_none()) ||
                (matches!(s.mode,SwordMode::Stage|SwordMode::Planted) && s.path.is_empty())
            }).collect();
            if available.is_empty() {return}
            // round 90: short of swords for a real shape with some on their way home and no ally in danger: wait a
            // moment for them rather than drawing the lone straight line
            let danger=ally_future.iter().any(|&(_,p,f,missing)|missing>=50 &&
                foes.iter().any(|(_,q,_)|near(*q,p,THREAT_R)||near(*q,f,THREAT_R)));
            let coming=(0..7).any(|i|self.swords[i].mode==SwordMode::Return && self.swords[i].holder.is_none());
            if available.len()<SHAPE_MIN[self.rank()].min(3) && coming && !danger {
                if self.short_since==0 { self.short_since=tick; }
                if tick<self.short_since+GATHER_WAIT[self.rank()] {return}
            }
            self.short_since=0;
            let radius=if density>=3 {55_000} else {35_000};
            let reused_legs=|idx:usize| {
                let spec=PATTERNS[idx];
                pattern_legs(spec.style,spec.swords,center,radius).iter().filter(|&&(a,b)|self.engravings.iter().any(|m|
                    m.until>tick && ((near(a,m.from,12_000)&&near(b,m.to,12_000)) ||
                    (near(a,m.to,12_000)&&near(b,m.from,12_000))))).count()
            };
            let best=choose_pattern(self.rank(),density,available.len(),!allies.is_empty(),reused_legs)
                .map(|idx|{let spec=PATTERNS[idx];(idx,0,pattern_legs(spec.style,spec.swords,center,radius))});
            if let Some((idx,_,legs))=best {
                self.next_plan_id+=1;
                let plan_id=self.next_plan_id;
                // round 91: proof in the log that every engraving sits on an enemy near him
                let enemy=foes.iter().map(|(_,p,_)|(sqdist(*p,center) as f64).sqrt() as i64).min().unwrap_or(-1);
                crate::mod_log(sim,"isliid_log.txt",&format!("plan.{tick}"),&format!("{} ({} swords) at {} from him, nearest enemy {} away",
                    PATTERNS[idx].name,PATTERNS[idx].swords,(sqdist(center,my_pos) as f64).sqrt() as i64,enemy));
                self.formations.push(FormationPlan{id:plan_id,
                    pattern:idx,center,radius,
                    legs:legs.clone(),until:tick+120,completed:false,target,redirected:false});
                let mut free=available;
                let mut deadline=tick+120;
                let mut launched=0;
                for (j,(a,b)) in legs.into_iter().enumerate() {
                    if self.engravings.iter().any(|m|m.until>tick &&
                        ((near(a,m.from,12_000)&&near(b,m.to,12_000)) ||
                         (near(a,m.to,12_000)&&near(b,m.from,12_000)))) {continue}
                    if free.is_empty() {break}
                    let nearest=free.iter().enumerate().min_by_key(|(_,i)|sqdist(self.position(sim,entity,**i),a)).map(|(k,_)|k).unwrap_or(0);
                    let i=free.remove(nearest);
                    let error=plan_wobble(sim.seed(),tick,i,j,wobble(self.rank(),self.imperial));
                    let start=((a.0+error).clamp(0,1_000_000),(a.1-error).clamp(0,1_000_000));
                    let end=((b.0+error).clamp(0,1_000_000),(b.1-error).clamp(0,1_000_000));
                    // round 94: both legs start from the ground, slow, and speed up
                    let from=self.position(sim,entity,i);
                    let travel=flight_ticks(i,SwordMode::Stage,dist(from,start),0)+flight_ticks(i,SwordMode::Draw,dist(start,end),0);
                    let leave=launch_tick(tick,launched);
                    launched+=1;
                    deadline=deadline.max(leave+travel+PLAN_SLACK);
                    self.send(sim,entity,i,SwordMode::Stage,vec![start],None);
                    self.swords[i].wait_until=leave;
                    self.swords[i].pending_draw=vec![end];
                    self.swords[i].activate_on_arrival=true;
                    self.swords[i].auto_owned=true;
                    self.swords[i].plan_id=Some(plan_id);
                }
                if let Some(plan)=self.formations.last_mut() { plan.until=deadline; }
                self.flags.push(EngravingFlag{id:plan_id,center,tag:format!("pattern_{idx}"),
                    phase:FlagPhase::Planned,until:deadline});
                self.next_plan_at=tick+180;
            } else if let Some(&i)=available.first() {
                // A lone free sword can still finish its own engraving when
                // there are not enough swords for a connected formation.
                let start=((center.0-30_000).clamp(0,1_000_000),center.1);
                let end=((center.0+30_000).clamp(0,1_000_000),center.1);
                self.send(sim,entity,i,SwordMode::Stage,vec![start],None);
                self.swords[i].pending_draw=vec![end];
                self.swords[i].activate_on_arrival=true;
                self.swords[i].auto_owned=true;
                self.solo_flag(i,start,end,tick);
                self.next_plan_at=tick+180;
            }
        }
    }

    /// Round 97: each formation aimed at an enemy shifts once, all of its swords together, when the first of them is
    /// about to reach its leg start: onto the target's fresh forecast (the rank's own LOOK_AHEAD / LEAD_CAP), by at most
    /// redirect_max(). Too late once any of its swords has armed or started drawing.
    fn redirect_formations(&mut self, sim: &mut StableSim<'_>, entity: usize) {
        let tick=sim.tick();
        let Some(team)=sim.get_entity(entity).map(|e|e.team()) else {return};
        let rank=self.rank();
        for k in 0..self.formations.len() {
            let plan=&self.formations[k];
            if plan.redirected || plan.completed || plan.until<=tick {continue}
            let id=plan.id;
            let mine:Vec<usize>=(0..7).filter(|&i|self.swords[i].plan_id==Some(id)).collect();
            if mine.is_empty() {continue}
            let staging=|s:&SwordMotion| s.mode==SwordMode::Stage && !s.path.is_empty() && !s.pending_draw.is_empty();
            if mine.iter().any(|&i|!staging(&self.swords[i])) { self.formations[k].redirected=true; continue }
            let eta=mine.iter().map(|&i|{let s=&self.swords[i];
                s.wait_until.saturating_sub(tick)+flight_ticks(i,s.mode,dist(s.pos,s.goal),airtime(s,tick))}).min().unwrap_or(0);
            if eta>REDIRECT_ETA {continue}
            self.formations[k].redirected=true;
            let Some(target)=self.formations[k].target else {continue};
            let Some(p)=sim.get_entity(target).filter(|e|e.is_alive() && sim.is_visible(team,target))
                .map(|e|{let q=e.pos();(q.0 as i64,q.1 as i64)}) else {continue};
            let horizon=LOOK_AHEAD[rank].min(eta as i64+30);
            let aim=forecast(p,self.observed.get(&target).copied(),tick,horizon,LEAD_CAP[rank]);
            let center=self.formations[k].center;
            let Some(delta)=redirect_delta(center,aim,redirect_max(rank,self.imperial)) else {continue};
            self.shift_plan(k,delta);
            crate::mod_log(sim,"isliid_log.txt",&format!("redirect.{tick}"),&format!("plan {id} moved {} onto its target",
                (((delta.0*delta.0+delta.1*delta.1) as f64).sqrt()) as i64));
        }
    }

    /// Round 97: move formation `k` by `delta`: its centre, legs and flag, and every one of its swords' flight and
    /// pending stroke (their aim error rides along). Its deadline grows by the extra flight at launch speed.
    fn shift_plan(&mut self, k: usize, delta: (i64, i64)) {
        let mv=|p:(i64,i64)|((p.0+delta.0).clamp(0,1_000_000),(p.1+delta.1).clamp(0,1_000_000));
        let extra=(((delta.0*delta.0+delta.1*delta.1) as f64).sqrt()/LAUNCH_SPEED as f64).ceil() as usize;
        let plan=&mut self.formations[k];
        plan.center=mv(plan.center);
        for leg in &mut plan.legs { *leg=(mv(leg.0),mv(leg.1)); }
        plan.until+=extra;
        let id=plan.id;
        for f in self.flags.iter_mut().filter(|f|f.id==id) { f.center=mv(f.center); f.until+=extra; }
        for s in self.swords.iter_mut().filter(|s|s.plan_id==Some(id)) {
            for p in &mut s.path { *p=mv(*p); }
            for p in &mut s.pending_draw { *p=mv(*p); }
            s.goal=mv(s.goal);
        }
    }

    fn evaluate_formations(&mut self, sim: &mut StableSim<'_>, entity: usize) {
        let mut ready=Vec::new();
        for plan in &mut self.formations {
            if plan.completed || plan.until<=sim.tick() {continue}
            let mut used=HashSet::new();
            let mut swords=HashSet::new();
            let mut error=0_i64;
            for &(a,b) in &plan.legs {
                let found=self.engravings.iter().filter(|m|m.until>sim.tick() && !used.contains(&m.id))
                    .filter_map(|m|{
                        let direct=((sqdist(a,m.from) as f64).sqrt()+(sqdist(b,m.to) as f64).sqrt()) as i64;
                        let reverse=((sqdist(a,m.to) as f64).sqrt()+(sqdist(b,m.from) as f64).sqrt()) as i64;
                        let d=direct.min(reverse);
                        (d<=32_000).then_some((m.id,m.sword,d))
                    }).min_by_key(|v|v.2);
                if let Some((id,sword,d))=found {used.insert(id);swords.insert(sword);error+=d;}
                else {break}
            }
            if used.len()==plan.legs.len() {
                plan.completed=true;
                let ids:Vec<u64>=used.into_iter().collect();
                ready.push((plan.clone(),swords.len(),error,formation_key(plan.pattern,&ids),ids));
            }
        }
        for (plan,distinct,error,key,ids) in ready {
            if self.activated.insert(key) {
                let took=self.apply_formation(sim,entity,&plan,distinct,error,&ids);
                if took { self.light(&ids,sim.tick()); }
                if let Some(f)=self.flags.iter_mut().find(|f|f.id==plan.id) {
                    f.phase=if took {FlagPhase::Complete} else {FlagPhase::Cancelled};f.until=sim.tick()+120;
                }
            }
        }
        // Player-made paths count too. Match the newest connected group at any
        // rotation, so a focused S1 drawing can finish a catalogue formation.
        let mut inferred:Option<(FormationPlan,usize,i64,u64,i64,Vec<u64>)>=None;
        for spec_index in 0..PATTERNS.len() {
            let spec=PATTERNS[spec_index];
            if self.engravings.len()<spec.swords {continue}
            let recent:Vec<_>=self.engravings.iter().rev().take(spec.swords).collect();
            if !recent.iter().any(|m|m.id==self.next_mark) {continue}
            let distinct:HashSet<_>=recent.iter().map(|m|m.sword).collect();
            if distinct.len()!=spec.swords {continue}
            let Some((center,radius,error))=match_live_drawing(spec,&recent) else {continue};
            let ids:Vec<_>=recent.iter().map(|m|m.id).collect();
            let key=formation_key(spec_index,&ids);
            if self.activated.contains(&key) {continue}
            let terra=distinct.contains(&1);
            let preference=if spec.effect==1 && terra {3} else if spec.effect==0 && !terra {2} else {0};
            let score=spec.swords as i64*10_000-error/10_000+preference;
            let plan=FormationPlan{id:0,pattern:spec_index,center,radius,
                legs:pattern_legs(spec.style,spec.swords,center,radius),
                until:sim.tick()+MARK_LIFE,completed:true,target:None,redirected:true};
            if inferred.as_ref().is_none_or(|(_,_,_,_,best,_)|score>*best) {
                inferred=Some((plan,distinct.len(),error,key,score,ids));
            }
        }
        if let Some((plan,distinct,error,key,_,ids))=inferred {
            if self.activated.insert(key) {
                self.next_plan_id+=1;
                let took=self.apply_formation(sim,entity,&plan,distinct,error,&ids);
                if took { self.light(&ids,sim.tick()); }
                self.flags.push(EngravingFlag{id:self.next_plan_id,center:plan.center,
                    tag:format!("pattern_{}",plan.pattern),phase:if took {FlagPhase::Complete} else {FlagPhase::Cancelled},until:sim.tick()+120});
            }
        }
        self.formations.retain(|p|p.until>sim.tick() && !p.completed);
        if self.activated.len()>10_000 {self.activated.clear();}
    }

    /// Round 89: a fired formation's legs flare; round 93: then they fade out within FIRED_LIFE (they stayed 20 s).
    /// Round 98: they light in a ripple, RIPPLE ticks apart in sword order; at Imperial they don't flare at all (the
    /// falling body is the fire), they only fade.
    fn light(&mut self, ids: &[u64], tick: usize) {
        let flare = self.rank() < 7;
        let mut order: Vec<usize> = self.engravings.iter().filter(|m| ids.contains(&m.id)).map(|m| m.sword).collect();
        order.sort_unstable(); order.dedup();
        for m in self.engravings.iter_mut().filter(|m| ids.contains(&m.id)) {
            let k = order.iter().position(|&s| s == m.sword).unwrap_or(0);
            if flare { m.lit = tick + k * RIPPLE; }
            m.until = m.until.min(tick + FIRED_LIFE + k * RIPPLE);
        }
    }

    /// Apply a finished formation; false when its grade fails (under 60% accuracy: nothing happens).
    fn apply_formation(&mut self, sim: &mut StableSim<'_>, entity: usize,
                       plan: &FormationPlan, distinct: usize, error: i64, ids: &[u64]) -> bool {
        let Some(me)=sim.get_entity(entity) else {return false};
        let team=me.team(); let attack=me.stat().attack;
        let spec=PATTERNS[plan.pattern];
        let _pattern_name=spec.name;
        let committed=(0..7).filter(|&j|self.swords[j].mode==SwordMode::Draw ||
            self.commit_until[j]>sim.tick()).count().max(1);
        let t=tier(self.rank());
        let Some((_,accuracy))=grade(formation_accuracy(error,plan.radius,spec.swords)) else {
            // round 89: too far off to take: it visibly cracks apart (it used to just show the cancelled logo)
            Self::fx(sim,entity,&format!("shatter_t{t}"),plan.center,0);
            return false
        };
        let synergy=100+25*distinct.saturating_sub(1);
        let participation=(distinct*100/spec.swords).max(25);
        let amp=if self.empowerment>0 {self.empowerment-=1;150} else {100};
        let concentration=(100_000*100/(plan.radius*2).max(50_000)).clamp(55,125) as usize;
        let power=(accuracy*synergy*participation*amp*concentration/
            (100*100*100*100*committed)).max(1);
        let radius=(plan.radius+15_000).clamp(25_000,170_000);
        let affected:Vec<(usize,bool,(i64,i64))>=(0..sim.entity_count()).filter_map(|n|sim.entity_at(n))
            .filter(|e|e.is_alive() && e.is_champion())
            .filter_map(|e|{let p=e.pos();let p=(p.0 as i64,p.1 as i64);
                near(p,plan.center,radius).then_some((e.id(),e.team()==team,p))}).collect();
        let family=FAMILIES[spec.effect.min(12)];
        let mut marked:Vec<(i128,usize)>=affected.iter().filter(|&&(_,ally,_)|match spec.effect { 4|7|8|10|11 => ally, 12 => true, _ => !ally })
            .map(|&(id,_,p)|(sqdist(p,plan.center),id)).collect();
        marked.sort_unstable();
        for (_,id) in marked.into_iter().take(HITMARK_CAP) {
            crate::fx_unit(sim,&format!("tfm2_isliid_emperor_hitmark_{family}_t{t}"),entity,id,0);
        }
        for (id,ally,p) in affected {
            match (spec.effect,ally) {
                (0|9,false) => {let focus=if spec.effect==9 {(radius as f64/(sqdist(p,plan.center) as f64).sqrt().max(10_000.0)).clamp(0.5,2.0)} else {1.0};
                    let raw=((35+attack*power/100) as f64*focus) as usize;
                    let damage=self.remote_damage(sim,entity,p,raw);
                    self.native_damage(sim,entity,id,damage);},
                (1,false) => {sim.apply_cc(id,&CcV1::stun((5+power/10).min(25) as u64));},
                (2,false) => {let q=((p.0*4+plan.center.0)/5,(p.1*4+plan.center.1)/5);
                    sim.entity_set_pos(id,q.0.max(0) as u64,q.1.max(0) as u64);},
                (3,false) => {let q=(p.0+(p.0-plan.center.0)/5,p.1+(p.1-plan.center.1)/5);
                    sim.entity_set_pos(id,q.0.max(0) as u64,q.1.max(0) as u64);},
                (4,true) => {let mut b=timed("il_formation_speed",150);b.move_speed_mult=(power as i32/3).max(8);sim.add_buff(id,&b);},
                (5,false) => {let mut b=timed("il_formation_shred",180);b.defence_mult=-(power as i32/2).max(8);sim.add_buff(id,&b);},
                (6,false) => {let mut b=timed("il_formation_suppress",150);b.attack_mult=-(power as i32/3).max(7);sim.add_buff(id,&b);},
                (7,true) => {let mut b=timed("il_formation_guard",150);b.damaged_reduce=(power/3).max(6).min(30);sim.add_buff(id,&b);},
                (8,true) => {let mut b=timed("il_formation_attack",150);b.attack_mult=(power as i32/3).max(7);sim.add_buff(id,&b);},
                (10,true) => {let mut b=timed("il_formation_cooldown",150);b.skill_cooldown_mult=(power as i32/3).max(8);sim.add_buff(id,&b);},
                (11,true) => {sim.heal(entity,id,20+attack*power/120);},
                (12,true) => {let mut b=timed("il_formation_domain",180);b.attack_mult=(power as i32/4).max(5);
                    b.damaged_reduce=(power/4).max(5).min(25);b.skill_cooldown_mult=(power as i32/5).max(4);sim.add_buff(id,&b);},
                (12,false) => {let mut b=timed("il_formation_domain_enemy",180);b.defence_mult=-(power as i32/4).max(5);sim.add_buff(id,&b);},
                _ => {}
            }
        }
        if self.rank()>=7 {
            // round 98: at Imperial the dominant sword's body falls onto the engraving (one effect)
            let strokes:Vec<_>=self.engravings.iter().filter(|m|ids.contains(&m.id)).map(|m|(m.sword,m.from,m.to)).collect();
            Self::fx(sim,entity,&fall_name(dominant_sword(&strokes),plan.radius,self.imperial==Some(1)),plan.center,0);
        } else {
            Self::fx(sim,entity,&fire_name(spec.effect,t,plan.radius),plan.center,0);
        }
        true
    }

    /// Round 89: sword `i` belongs to a formation that hasn't finished or expired yet.
    fn in_live_plan(&self, i: usize) -> bool {
        self.swords[i].plan_id.is_some_and(|id|self.formations.iter().any(|p|p.id==id && !p.completed))
    }

    /// Round 89: a sword the brain or a press must not take: in a live formation, on its way to draw a stroke, drawing,
    /// or armed by the brain. A Ready sword armed by a manual S1 press stays usable (the next press draws it).
    fn reserved(&self, i: usize) -> bool {
        let s=&self.swords[i];
        self.in_live_plan(i) || !s.pending_draw.is_empty() || s.mode==SwordMode::Draw || (s.auto_owned && s.is_committed())
    }

    /// A sword lying on the ground with nothing to do (no pending draw, not armed, not drawing, no live plan).
    fn idle_grounded(&self, i: usize) -> bool {
        let s=&self.swords[i];
        matches!(s.mode,SwordMode::Planted|SwordMode::Stage) && s.path.is_empty() && !s.is_committed()
            && !self.in_live_plan(i)
    }

    /// Round 89: the sword an S1 press moves: the selected one unless it's reserved, else the free sword nearest the
    /// press point (orbiting ones first: they're in hand), else none.
    fn press_sword(&self, entity: usize, tick: usize, p: (i64,i64), pos: impl Fn(usize)->(i64,i64)) -> Option<usize> {
        let s=&self.swords[self.selected];
        let usable=matches!(s.mode,SwordMode::Ready|SwordMode::Orbit|SwordMode::Planted|SwordMode::Stage)
            && s.path.is_empty() && s.holder.is_none_or(|h|h==entity);
        if usable && !self.reserved(self.selected) { return Some(self.selected); }
        self.free_swords(entity,tick).into_iter()
            .min_by_key(|&j|(self.swords[j].mode!=SwordMode::Orbit,sqdist(pos(j),p),j))
    }

    /// Round 89: the sword an S2 press calls back: the selected one for a press at his feet, else the idle grounded
    /// sword nearest the press point. Never a reserved or armed one.
    fn recall_pick(&self, to_last: bool, p: (i64,i64), pos: impl Fn(usize)->(i64,i64)) -> Option<usize> {
        let ok=|j:usize| !self.reserved(j) && !self.swords[j].is_activated();
        if to_last { return ok(self.selected).then_some(self.selected); }
        (0..7).filter(|&j|ok(j) && self.idle_grounded(j)).min_by_key(|&j|(sqdist(pos(j),p),j))
            .or_else(|| self.nearest(p, 28_000).filter(|&j|ok(j)))
    }

    /// Round 89: an auto-owned sword the brain is done with: its formation finished or expired (or its solo stroke is
    /// drawn) and it lies idle on the ground. It used to be recalled whenever no visible enemy stood within 120000 of
    /// the sword's current position, so a sword launched at a far target, or one whose target stepped into a bush,
    /// came back mid-flight before it could draw, and the engraving was cancelled.
    fn abandoned(&self, i: usize) -> bool {
        let s=&self.swords[i];
        s.auto_owned && !self.reserved(i) && !s.is_activated()
            && matches!(s.mode,SwordMode::Stage|SwordMode::Planted) && s.path.is_empty()
    }

    /// Swords the brain may hand out: orbiting Isliid himself, or idle on the ground; never one under a command lock.
    fn free_swords(&self, entity: usize, tick: usize) -> Vec<usize> {
        (0..7).filter(|&i| {
            let s=&self.swords[i];
            tick>=s.locked_until && ((s.mode==SwordMode::Orbit && s.holder.is_none_or(|h|h==entity)) || self.idle_grounded(i))
        }).collect()
    }

    /// Round 88: threatened allies get escorts (the best sword for them by escort_score, the nearest on a tie), up to
    /// ESCORTS[rank] each, on a lease of REASSESS[rank] ticks. One sword always stays with Isliid for his basic
    /// attacks (at Regent and up he'll give the last one to an ally under 30% HP). An expired lease on an ally who's
    /// no longer threatened sends the sword home (unless the ally is right beside him).
    fn assign_escorts(&mut self, sim: &StableSim<'_>, entity: usize,
                      ally_future: &[AllyFuture], foes: &[(usize,(i64,i64),usize)], tick: usize) {
        let rank=self.rank();
        let my_pos=sim.get_entity(entity).map(|e|{let p=e.pos();(p.0 as i64,p.1 as i64)}).unwrap_or((0,0));
        let threat=|p:(i64,i64),f:(i64,i64)| foes.iter().filter(|(_,q,_)|near(*q,p,THREAT_R)||near(*q,f,THREAT_R)).count();
        // round 92: allies anywhere, but only those he notices on this look (mastery decides how often)
        let pct=notice_pct(rank,self.imperial);
        let mut threatened:Vec<(usize,(i64,i64),usize,usize)>=ally_future.iter()
            .map(|&(id,p,f,missing)|(id,p,missing,threat(p,f)))
            .filter(|&(_,_,missing,n)| n>=1 && (missing>=25 || n>=2))
            .filter(|&(id,_,_,_)| notices(sim.seed(),tick,id,pct)).collect();
        threatened.sort_by_key(|&(id,_,missing,_)|(std::cmp::Reverse(missing),id));
        let engaged=foes.iter().any(|(_,q,_)|near(*q,my_pos,200_000));
        // leases
        for i in 0..7 {
            let Some(h)=self.swords[i].holder.filter(|&h|h!=entity) else {continue};
            if !matches!(self.swords[i].mode,SwordMode::Orbit|SwordMode::Stage) {continue}
            if tick<self.swords[i].escort_until {continue}
            if threatened.iter().any(|t|t.0==h) { self.swords[i].escort_until=tick+REASSESS[rank]; continue; }
            let close=ally_future.iter().find(|a|a.0==h).is_some_and(|a|near(a.1,my_pos,40_000));
            if close || tick<self.swords[i].locked_until {continue}
            self.send(sim,entity,i,SwordMode::Return,vec![my_pos],None);
            self.swords[i].holder=None;
        }
        for &(ally,p,missing,n) in &threatened {
            let have=(0..7).filter(|&i|self.swords[i].holder==Some(ally) &&
                matches!(self.swords[i].mode,SwordMode::Orbit|SwordMode::Stage|SwordMode::Strike|SwordMode::Return)).count();
            let cap=ESCORTS[rank].min(PER_ALLY);
            for _ in have..cap {
                let mut free=self.free_swords(entity,tick);
                // round 90: keep enough swords free to engrave while an enemy is in reach (all hands for a dying ally)
                if engaged && missing<70 && free.len()<=FORMATION_RESERVE {break}
                let at_home=free.iter().filter(|&&i|self.swords[i].mode==SwordMode::Orbit).count();
                let give_last=rank>=5 && missing>=70;
                if at_home<=1 && !give_last {
                    free.retain(|&i|self.swords[i].mode!=SwordMode::Orbit);
                }
                let best=free.into_iter().map(|i|{
                    let from=self.position(sim,entity,i);
                    let eta=flight_ticks(i,SwordMode::Stage,dist(from,p),0) as i64;
                    (escort_score(i,missing,n)-eta/4,i)
                }).max_by_key(|&(score,i)|(score,std::cmp::Reverse(i)));
                let Some((_,i))=best else {break};
                self.send(sim,entity,i,SwordMode::Stage,vec![p],None);
                self.swords[i].holder=Some(ally);
                self.swords[i].last_ally=Some(ally);
                self.swords[i].auto_owned=false;
                self.swords[i].plan_id=None;
                self.swords[i].escort_until=tick+REASSESS[rank];
            }
        }
    }

    /// Its host died (or it's been idle too long): the nearest ally in a fight gets it, otherwise it comes home.
    fn reassign_or_return(&mut self, sim: &StableSim<'_>, entity: usize, i: usize) {
        let Some(me)=sim.get_entity(entity) else {return};
        let team=me.team();
        let home={let q=me.pos();(q.0 as i64,q.1 as i64)};
        let from=self.swords[i].pos;
        let foes:Vec<(i64,i64)>=(0..sim.entity_count()).filter_map(|n|sim.entity_at(n))
            .filter(|e|e.is_alive() && e.is_champion() && e.team()!=team && sim.is_visible(team,e.id()))
            .map(|e|{let p=e.pos();(p.0 as i64,p.1 as i64)}).collect();
        let ally=(0..sim.entity_count()).filter_map(|n|sim.entity_at(n))
            .filter(|e|e.is_alive() && e.is_champion() && e.team()==team && e.id()!=entity)
            .map(|e|{let p=e.pos();(e.id(),(p.0 as i64,p.1 as i64))})
            .filter(|&(_,p)|near(p,from,100_000) && foes.iter().any(|f|near(*f,p,THREAT_R)))
            .filter(|&(id,_)|notices(sim.seed(),sim.tick(),id,notice_pct(self.rank(),self.imperial)))
            .min_by_key(|&(id,p)|(sqdist(p,from),id));
        if let Some((id,p))=ally {
            self.send(sim,entity,i,SwordMode::Stage,vec![p],None);
            self.swords[i].holder=Some(id);
            self.swords[i].last_ally=Some(id);
            self.swords[i].escort_until=sim.tick()+REASSESS[self.rank()];
        } else {
            self.send(sim,entity,i,SwordMode::Return,vec![home],None);
            self.swords[i].holder=None;
        }
        self.swords[i].auto_owned=false;
        self.swords[i].plan_id=None;
    }

    /// Round 88: every tick, at every rank: a sword idle on the ground for IDLE_RETURN[rank] ticks is reclaimed (thrown
    /// swords used to strand at the low ranks, and with no sword in hand his basic attacks did nothing).
    fn idle_reclaim(&mut self, sim: &StableSim<'_>, entity: usize) {
        let tick=sim.tick();
        for i in 0..7 {
            if !self.idle_grounded(i) || tick<self.swords[i].locked_until { self.swords[i].idle_since=0; continue; }
            if self.swords[i].idle_since==0 { self.swords[i].idle_since=tick.max(1); continue; }
            if tick>=self.swords[i].idle_since+IDLE_RETURN[self.rank()] {
                self.swords[i].idle_since=0;
                self.reassign_or_return(sim,entity,i);
            }
        }
    }

    /// Round 88: which presses the brain would use now (S1: it's free to plan, has 2+ swords to spare and an enemy
    /// champion in reach; S2: a sword lies idle to call back), for the input AI and for the command filter.
    fn note_wants(&mut self, sim: &StableSim<'_>, player: usize, entity: usize) {
        let tick=sim.tick();
        let Some(me)=sim.get_entity(entity) else {return};
        let team=me.team();
        let at={let p=me.pos();(p.0 as i64,p.1 as i64)};
        let enemy=(0..sim.entity_count()).filter_map(|n|sim.entity_at(n))
            .any(|e|e.is_alive() && e.is_champion() && e.team()!=team && sim.is_visible(team,e.id())
                && {let p=e.pos();near(at,(p.0 as i64,p.1 as i64),200_000)});
        let s1=tick>=self.next_plan_at && enemy && self.free_swords(entity,tick).len()>=2;
        let s2=(0..7).any(|i|self.idle_grounded(i));
        if s1 { self.want_at[0]=tick; }
        if s2 { self.want_at[1]=tick; }
        crate::press::note(sim.seed(),player,tick,if s1 {crate::press::S1} else {0}|if s2 {crate::press::S2} else {0});
    }

    fn ally_attacks(&mut self, sim: &mut StableSim<'_>, entity: usize) {
        let Some(me)=sim.get_entity(entity) else {return};
        let team=me.team();
        let mine={let p=me.pos();(p.0 as i64,p.1 as i64)};
        // round 93: each holder's nearest target is found once per call, not once per sword (send() doesn't touch the
        // sim, so the answer is the same)
        let mut targets:Vec<(usize,(i64,i64),Option<(usize,(i64,i64))>)>=Vec::new();
        for i in 0..7 {
            let s=&self.swords[i];
            if s.mode!=SwordMode::Orbit || s.holder.is_none() || sim.tick()<s.attack_at {continue}
            let Some(holder)=s.holder.and_then(|id|sim.get_entity(id)).filter(|e|e.is_alive()) else {continue};
            let hp=holder.pos(); let hp=(hp.0 as i64,hp.1 as i64);
            let target=if let Some(&(_,_,t))=targets.iter().find(|(h,_,_)|*h==holder.id()) { t } else {
                let t=(0..sim.entity_count()).filter_map(|n|sim.entity_at(n))
                    .filter(|e|e.is_alive() && e.team()!=team && !e.is_tower() && sim.is_visible(team,e.id()))
                    .filter(|e|{let p=e.pos();near(hp,(p.0 as i64,p.1 as i64),100_000)})
                    .min_by_key(|e|{let p=e.pos();sqdist(hp,(p.0 as i64,p.1 as i64))})
                    .map(|e|{let p=e.pos();(e.id(),(p.0 as i64,p.1 as i64))});
                targets.push((holder.id(),hp,t));
                t
            };
            if let Some((target,p))=target {
                self.send(sim,entity,i,SwordMode::Strike,vec![p],Some(target));
                // round 93: the further the ally from Isliid, the longer the gap
                self.swords[i].attack_at=sim.tick()+strike_gap(self.rank(),dist(mine,hp));
            }
        }
    }

    fn update_auras(&mut self, sim: &mut StableSim<'_>, entity: usize) {
        let Some(me)=sim.get_entity(entity) else {return};
        let team=me.team();
        let rank=self.rank();
        let sources:Vec<(usize,(i64,i64))>=(0..7).filter_map(|i|{
            let s=&self.swords[i];
            let holder_pos=if s.mode==SwordMode::Orbit {
                let holder=sim.get_entity(s.holder.unwrap_or(entity))?;
                if !holder.is_alive() || holder.team()!=team {return None}
                let p=holder.pos();Some((p.0 as i64,p.1 as i64))
            } else {None};
            let center=aura_source(s.mode,s.pos,holder_pos)?;
            Some((i,center))
        }).collect();
        // A detached sword is its own moving field source. Single-frame aliases
        // retain the shared eight-frame phase while the effect follows its point.
        for &(i,p) in &sources {
            // round 92: only a sword on the ground shows its field (a flying one is too brief to read, and a volley of
            // them was a burst of sprites); the aura itself still works in flight
            let grounded=matches!(visual_for(self.swords[i].mode,self.swords[i].path.is_empty()),Visual::Grounded(_));
            // round 93: as 2-frame pairs every PAIR_STEP ticks (single frames every 6 before), like the grounded swords
            if grounded && sim.tick().is_multiple_of(PAIR_STEP) {
                let tag=format!("aura_field_{i}_rank{rank}_pair{}",(sim.tick()/PAIR_STEP)%4);
                Self::fx(sim,entity,&tag,p,PAIR_STEP as u64);
            }
        }
        let recipients:Vec<(usize,bool,(i64,i64))>=(0..sim.entity_count())
            .filter_map(|n|sim.entity_at(n))
            .filter(|e|e.is_alive() && e.is_champion())
            .map(|e|{let p=e.pos();(e.id(),e.team()==team,(p.0 as i64,p.1 as i64))})
            .collect();
        let mut next=HashSet::new();
        let mut bases=HashMap::new();
        for (id,ally,p) in recipients {
            let affecting:Vec<usize>=sources.iter().filter(|(_,center)|near(*center,p,40_000))
                .map(|(i,_)|*i).collect();
            let count=affecting.len();
            if count==0 {continue}
            let first=affecting[0];
            bases.insert(id,(ally,rank));
            let side=if ally {"ally"} else {"enemy"};
            if self.aura_base_shown.get(&id)!=Some(&(ally,rank)) {
                if let Some((old_side,old_rank))=self.aura_base_shown.get(&id) {
                    sim.entity_remove_buff(id,&format!("il_aura_base_rank{old_rank}_{}",
                        if *old_side {"ally"} else {"enemy"}));
                }
                sim.add_buff(id,&BuffV1::named(&format!("il_aura_base_rank{rank}_{side}")));
            }
            let part=|amount:usize|aura_part(amount,count);
            for i in affecting {
                next.insert((id,i));
                if i==0 && !ally {sim.entity_set_invisible(id,0);}
                if self.aura_visual_at.get(&(id,i))==Some(&(ally,rank,count)) {continue}
                let name=format!("il_aura_{i}_{side}");
                if let Some((old_ally,_,_))=self.aura_visual_at.get(&(id,i)) {
                    sim.entity_remove_buff(id,&format!("il_aura_{i}_{}",if *old_ally {"ally"} else {"enemy"}));
                }
                let mut buff=BuffV1::named(&name);
                match (i,ally) {
                    (0,true)=>buff.range=part(10_000),
                    (0,false)=>{},
                    (1,true)=>buff.damaged_reduce=part(5),
                    (1,false)=>buff.move_speed_mult=-(part(10) as i32),
                    (2,true)=>buff.attack_mult=part(6) as i32,
                    (2,false)=>buff.defence_mult=-(part(8) as i32),
                    (3,true)=>buff.move_speed_mult=part(10) as i32,
                    (3,false)=>buff.attack_speed_mult=-(part(8) as i32),
                    (4,true)=>buff.vamp=part(6) as i32,
                    (4,false)=>buff.heal_reduce=part(10),
                    (5,true)=>buff.radius_mult=part(6) as i32,
                    (5,false)=>buff.move_speed_mult=-(part(6) as i32),
                    (6,true)=>buff.skill_cooldown_mult=part(6) as i32,
                    (6,false)=>buff.attack_mult=-(part(5) as i32),
                    _=>{}
                }
                sim.add_buff(id,&buff);
                self.aura_visual_at.insert((id,i),(ally,rank,count));
            }
            // round 91: one aura visual per champion (the first sword on them), not one per sword (up to 7 animated
            // buffs each, re-added whenever the sword count changed)
            let shown=(first,ally,rank);
            if self.aura_sword_shown.get(&id)!=Some(&shown) {
                if let Some((old,old_ally,old_rank))=self.aura_sword_shown.get(&id) {
                    sim.entity_remove_buff(id,&format!("il_aura_visual_{old}_rank{old_rank}_{}",if *old_ally {"ally"} else {"enemy"}));
                }
                sim.add_buff(id,&BuffV1::named(&format!("il_aura_visual_{first}_rank{rank}_{side}")));
                self.aura_sword_shown.insert(id,shown);
            }
        }
        for &(id,i) in self.aura_active.difference(&next) {
            sim.entity_remove_buff(id,&format!("il_aura_{i}_ally"));
            sim.entity_remove_buff(id,&format!("il_aura_{i}_enemy"));
            self.aura_visual_at.remove(&(id,i));
        }
        let gone:Vec<usize>=self.aura_sword_shown.keys().copied().filter(|id|!bases.contains_key(id)).collect();
        for id in gone {
            if let Some((sword,ally,old_rank))=self.aura_sword_shown.remove(&id) {
                sim.entity_remove_buff(id,&format!("il_aura_visual_{sword}_rank{old_rank}_{}",if ally {"ally"} else {"enemy"}));
            }
        }
        for (&id,&(ally,old_rank)) in &self.aura_base_shown {
            if !bases.contains_key(&id) {
                sim.entity_remove_buff(id,&format!("il_aura_base_rank{old_rank}_{}",
                    if ally {"ally"} else {"enemy"}));
            }
        }
        self.aura_active=next;
        self.aura_base_shown=bases;
    }

    fn show(&mut self, sim: &mut StableSim<'_>, entity: usize) {
        // round 93: the badge name is only rebuilt when the rank changes (it was formatted every tick)
        let key = (self.rank, self.imperial);
        if self.badge_for != Some(key) || self.shown.is_none() {
            self.badge_for = Some(key);
            let badge = if self.rank() == 7 { format!("il_imperial{}", self.imperial.unwrap_or(10)) }
                else { format!("il_rank{}", self.rank()) };
            if self.shown.as_deref() != Some(&badge) {
                if let Some(old) = &self.shown { sim.entity_remove_buff(entity, old); }
                sim.add_buff(entity, &BuffV1::named(&badge));
                self.shown = Some(badge);
            }
        }
        let rank = self.rank();
        // round 97: at Imperial his swords live inside the black hole above his head (one buff for however many are in)
        let inside = (0..7).filter(|&i| on_ring(&self.swords[i], entity, sim.tick())).count();
        let hole = (rank >= 7).then(|| hole_buff(inside, self.imperial == Some(1)));
        if hole != self.hole_shown {
            if let Some(old) = &self.hole_shown { sim.entity_remove_buff(entity, old); }
            if let Some(name) = &hole { sim.add_buff(entity, &BuffV1::named(name)); }
            self.hole_shown = hole;
        }
        for i in 0..7 {
            let home = on_ring(&self.swords[i], entity, sim.tick()) && rank < 7;
            let want = home.then_some((rank, i == self.selected));
            if self.arsenal_shown[i] != want {
                if let Some((old, sel)) = self.arsenal_shown[i] { sim.entity_remove_buff(entity, &arsenal_buff(i, old, sel)); }
                if let Some((t, sel)) = want { sim.add_buff(entity, &BuffV1::named(&arsenal_buff(i, t, sel))); }
                self.arsenal_shown[i] = want;
            }
            let holder = if self.swords[i].mode == SwordMode::Orbit && sim.tick() >= self.swords[i].vis_until {
                self.swords[i].holder.filter(|&h| h != entity).map(|id|(id,rank))
            } else { None };
            if holder != self.holder_shown[i] {
                if let Some((old,old_rank))=self.holder_shown[i] {
                    sim.entity_remove_buff(old,&format!("il_ar_{}_rank{old_rank}",SWORDS[i]));
                }
                if let Some((ally,tier))=holder {
                    sim.add_buff(ally,&BuffV1::named(&format!("il_ar_{}_rank{tier}",SWORDS[i])));
                }
                self.holder_shown[i]=holder;
            }
        }
        let selected = (self.swords[self.selected].mode == SwordMode::Orbit && self.swords[self.selected].holder.is_none()).then_some(self.selected);
        if self.selected_shown != selected {
            if let Some(old) = self.selected_shown {
                sim.entity_remove_buff(entity, &format!("il_select_{}", SWORDS[old]));
            }
            if let Some(i)=selected {
                let mut buff = BuffV1::named(&format!("il_select_{}", SWORDS[i]));
                if i == 0 { buff.range = 25_000; }
                if i == 1 { buff.attack_speed_mult = -20; }
                if i == 3 { buff.attack_speed_mult = 20; }
                sim.add_buff(entity, &buff);
            }
            self.selected_shown = selected;
        }
    }
}

/// Whether sword `s` shows on Isliid's arsenal ring: orbiting him with its last flight segment over, or (round 93) a
/// formation sword still waiting its turn in the volley (it used to vanish from his ring, then appear far away).
fn on_ring(s: &SwordMotion, entity: usize, tick: usize) -> bool {
    (s.mode == SwordMode::Orbit && s.holder.is_none_or(|h| h == entity) && tick >= s.vis_until) || tick < s.wait_until
}

fn sqdist(a: (i64, i64), b: (i64, i64)) -> i128 {
    let dx = (a.0 - b.0) as i128; let dy = (a.1 - b.1) as i128; dx * dx + dy * dy
}

impl SwordMotion {
    fn is_activated(&self) -> bool {
        matches!(self.mode, SwordMode::Ready | SwordMode::Draw)
    }
    fn is_committed(&self) -> bool {
        self.is_activated() || (self.mode == SwordMode::Stage && !self.pending_draw.is_empty())
    }
}

fn formation_key(pattern: usize, ids: &[u64]) -> u64 {
    let mut sorted=ids.to_vec(); sorted.sort_unstable();
    sorted.into_iter().fold(0x9e37_79b9_u64 ^ pattern as u64,
        |key,id|key.rotate_left(9) ^ id.wrapping_mul(0xbf58_476d_1ce4_e5b9))
}

// Fit a catalogue drawing to recent live marks. Rotation and uniform scale
// are free, while endpoint error determines its engraving quality.
fn match_live_drawing(spec: Pattern, marks: &[&EngravingMark]) -> Option<((i64,i64),i64,i64)> {
    if marks.len()!=spec.swords {return None}
    let distinct:HashSet<_>=marks.iter().map(|m|m.sword).collect();
    if distinct.len()<2 {return None}
    let x0=marks.iter().flat_map(|m|[m.from.0,m.to.0]).min()?;
    let x1=marks.iter().flat_map(|m|[m.from.0,m.to.0]).max()?;
    let y0=marks.iter().flat_map(|m|[m.from.1,m.to.1]).min()?;
    let y1=marks.iter().flat_map(|m|[m.from.1,m.to.1]).max()?;
    let center=((x0+x1)/2,(y0+y1)/2);
    let span=(x1-x0).max(y1-y0);
    if span<18_000 {return None}
    let template=pattern_legs(spec.style,spec.swords,(0,0),100_000);
    let mut best:Option<(i64,i64)>=None;
    for turn in 0..16 {
        let angle=std::f64::consts::TAU*turn as f64/16.0;
        let (sn,cs)=angle.sin_cos();
        let rotate=|p:(i64,i64)|->(i64,i64) {
            ((p.0 as f64*cs-p.1 as f64*sn) as i64,(p.0 as f64*sn+p.1 as f64*cs) as i64)
        };
        let rotated:Vec<_>=template.iter().map(|&(a,b)|(rotate(a),rotate(b))).collect();
        let tx0=rotated.iter().flat_map(|(a,b)|[a.0,b.0]).min()?;
        let tx1=rotated.iter().flat_map(|(a,b)|[a.0,b.0]).max()?;
        let ty0=rotated.iter().flat_map(|(a,b)|[a.1,b.1]).min()?;
        let ty1=rotated.iter().flat_map(|(a,b)|[a.1,b.1]).max()?;
        let tspan=(tx1-tx0).max(ty1-ty0).max(1);
        let radius=span*100_000/tspan;
        let template_center=((tx0+tx1)/2,(ty0+ty1)/2);
        let at=|p:(i64,i64)|->(i64,i64) {
            (center.0+(p.0-template_center.0)*span/tspan,
             center.1+(p.1-template_center.1)*span/tspan)
        };
        let tolerance=(radius/3).clamp(12_000,34_000);
        let mut used=HashSet::new(); let mut error=0_i64;
        for &(a,b) in &rotated {
            let (a,b)=(at(a),at(b));
            let found=marks.iter().enumerate().filter(|(j,_)|!used.contains(j))
                .map(|(j,m)|{
                    let d=((sqdist(a,m.from) as f64).sqrt()+(sqdist(b,m.to) as f64).sqrt()) as i64;
                    let r=((sqdist(a,m.to) as f64).sqrt()+(sqdist(b,m.from) as f64).sqrt()) as i64;
                    (j,d.min(r))
                }).min_by_key(|(_,distance)|*distance);
            let Some((index,distance))=found else {error=i64::MAX;break};
            if distance>tolerance*2 {error=i64::MAX;break}
            used.insert(index);error+=distance;
        }
        if error<i64::MAX && best.is_none_or(|(_,old)|error<old) {best=Some((radius,error));}
    }
    best.map(|(radius,error)|(center,radius,error))
}

fn forecast(current: (i64,i64), observed: Option<((i64,i64),usize)>,
            tick: usize, horizon: i64, cap: i64) -> (i64,i64) {
    let Some((prior,seen))=observed else { return current };
    let dt=tick.saturating_sub(seen).max(20) as i64;
    let vx=((current.0-prior.0)/dt).clamp(-2_000,2_000);
    let vy=((current.1-prior.1)/dt).clamp(-2_000,2_000);
    // round 91: never lead further than the cap (it used to reach up to 600000 ahead)
    let (mut lx, mut ly)=(vx*horizon, vy*horizon);
    let len=((lx*lx+ly*ly) as f64).sqrt();
    if len>cap as f64 { let k=cap as f64/len.max(1.0); lx=(lx as f64*k) as i64; ly=(ly as f64*k) as i64; }
    ((current.0+lx).clamp(0,1_000_000), (current.1+ly).clamp(0,1_000_000))
}


/// Round 91: a camp he may engrave: an enemy champion contests it.
fn contested(objective: (i64,i64), foes: &[(usize,(i64,i64),usize)]) -> bool {
    foes.iter().any(|(_,p,_)| near(*p, objective, CONTEST_R))
}

impl StablePassive for Isliid {
    fn clone_box(&self) -> Box<dyn StablePassive> { Box::new(self.clone()) }

    fn on_spawn(&mut self, _sim: &mut StableSim<'_>, _player: usize, _entity: usize) {}

    fn on_base_attack(&mut self, sim: &mut StableSim<'_>, _seed: u64, _player: usize, _entity: usize) {
        self.base_hit_ready = true;
        self.base_hit_until = sim.tick() + 60;
    }

    fn on_dead(&mut self, sim: &mut StableSim<'_>, _player: usize) {
        for (i, holder) in self.holder_shown.iter_mut().enumerate() {
            if let Some((id,rank)) = holder.take() {
                sim.entity_remove_buff(id, &format!("il_ar_{}_rank{rank}",SWORDS[i]));
            }
        }
        self.swords = std::array::from_fn(|_| SwordMotion::default());
        self.anchors = [None; 7];
        self.engravings.clear(); self.formations.clear(); self.empowerment = 0; self.commit_until = [0; 7];
        self.gathering = false; self.base_hit_ready = false;
        for (id,i) in self.aura_active.drain() {
            sim.entity_remove_buff(id,&format!("il_aura_{i}_ally"));
            sim.entity_remove_buff(id,&format!("il_aura_{i}_enemy"));
        }
        for ((id,i),(ally,rank,_)) in self.aura_visual_at.drain() {
            sim.entity_remove_buff(id,&format!("il_aura_visual_{i}_rank{rank}_{}",
                if ally {"ally"} else {"enemy"}));
        }
        for (id,(sword,ally,rank)) in self.aura_sword_shown.drain() {
            sim.entity_remove_buff(id,&format!("il_aura_visual_{sword}_rank{rank}_{}",if ally {"ally"} else {"enemy"}));
        }
        for (id,(ally,rank)) in self.aura_base_shown.drain() {
            sim.entity_remove_buff(id,&format!("il_aura_base_rank{rank}_{}",
                if ally {"ally"} else {"enemy"}));
        }
        self.shown = None; self.arsenal_shown = [None; 7]; self.selected_shown = None; self.hole_shown = None;
    }

    fn on_attack(&mut self, sim: &mut StableSim<'_>, _player: usize, entity: usize, target: usize, damage: &mut usize) {
        if self.native_hit || !self.base_hit_ready || sim.tick() > self.base_hit_until { return; }
        self.base_hit_ready = false;
        if sim.tick() < self.base_ready_at { *damage=0; return; }
        let (Some(me), Some(victim)) = (sim.get_entity(entity), sim.get_entity(target)) else { return };
        let from = { let q=me.pos(); (q.0 as i64,q.1 as i64) };
        let to = { let q=victim.pos(); (q.0 as i64,q.1 as i64) };
        let distance = ((sqdist(from, to) as f64).sqrt()) as i64;
        let Some(sword) = self.attack_sword(me.hp(), victim.stat().defence, distance) else {
            // no sword in hand: this hit is lost, but the nearest idle blade comes home at once
            *damage = 0;
            if let Some(i)=(0..7).filter(|&i|self.idle_grounded(i)).min_by_key(|&i|(sqdist(self.swords[i].pos,from),i)) {
                self.send(sim,entity,i,SwordMode::Return,vec![from],None);
                self.swords[i].holder=None;
            }
            return
        };
        self.base_ready_at=sim.tick()+if sword==3 {60} else {72};
        self.selected = sword;
        if !near(from, to, MELEE) {
            self.send(sim,entity,sword,SwordMode::Throw,vec![to],Some(target));
        } else {
            // round 98: at Imperial the melee hit is a star-burst, not a slash
            let cosmic = if self.rank() >= 7 { "_cosmic" } else { "" };
            crate::fx_unit(sim, &format!("tfm2_isliid_emperor_{}_hit{cosmic}", SWORDS[sword]), entity, target, 0);
        }
        match sword {
            0 => { // Skylight: longest reach and reveals its victim.
                sim.entity_set_invisible(target, 0);
                sim.add_buff(target, &timed("il_revealed", 90));
            }
            1 => { // Terra: heavy stagger.
                let mut slow = timed("il_terra", 45); slow.move_speed_mult = -28; sim.add_buff(target, &slow);
                sim.apply_cc(target, &CcV1::stun(6));
                *damage = damage.saturating_mul(115) / 100;
            }
            2 => { // Darkbringer: stacking physical defence break.
                let stack = self.dark_stacks.entry(target).or_insert((0, 0));
                if stack.1 <= sim.tick() { stack.0 = 0; }
                stack.0 = (stack.0 + 1).min(4);
                stack.1 = sim.tick() + 180;
                for j in 1..=4 { sim.entity_remove_buff(target, &format!("il_dark{j}")); }
                let mut b = timed(&format!("il_dark{}", stack.0), 180); b.defence_mult = -8 * stack.0 as i32;
                sim.add_buff(target, &b);
            }
            3 => { // Gale: a short forward drift at melee range.
                if near(from, to, MELEE) {
                    let p = ((from.0 * 2 + to.0) / 3, (from.1 * 2 + to.1) / 3);
                    sim.entity_set_pos(entity, p.0.max(0) as u64, p.1.max(0) as u64);
                }
            }
            4 => { // Blood: trade health for a stronger hit, then leech.
                let (hp, max_hp) = sim.get_entity(entity).map_or((0,0), |e| e.hp());
                let cost = (max_hp / 100).max(1).min(hp.saturating_sub(1));
                sim.entity_set_hp(entity, hp.saturating_sub(cost));
                *damage = damage.saturating_mul(110) / 100;
                sim.heal(entity, entity, *damage / 5);
            }
            5 => { // Rift: displace the target slightly toward the blade's impact.
                let p = ((to.0 * 4 + from.0) / 5, (to.1 * 4 + from.1) / 5);
                sim.entity_set_pos(target, p.0.max(0) as u64, p.1.max(0) as u64);
            }
            _ => { *damage = damage.saturating_mul(75) / 100; }
        }
    }

    fn on_update(&mut self, sim: &mut StableSim<'_>, _seed: u64, player: usize, entity: usize) {
        let tick = sim.tick();
        if self.rank.is_none() && tick >= 60 {
            let (rank, imperial) = rank_for(sim.seed(), athlete_of(sim.seed(), player));
            self.rank = Some(rank);
            self.imperial = imperial;
            crate::mod_log(sim, "isliid_log.txt", &format!("spawn.{player}"), &format!("Isliid native {} running, rank {} {}{} (player {player})",
                crate::VERSION, rank, NAMES[rank.min(7)], imperial.map_or(String::new(), |n| format!(" #{n}"))));
            let _ = NAMES[rank.min(7)];
            // Mastery changes forecast and decisions, not sword access or speed.
        }
        if tick >= 1800 {
            if let Some(a) = athlete_of(sim.seed(), player) {
                if !self.recorded {
                    let id = sim.sim_origin().unwrap_or_default().match_id;
                    self.recorded = record_game(sim.seed(), player, a, id);
                }
                if tick % 30 == 0 { self.track_result(sim, player, entity); }
            }
        }
        for a in &mut self.anchors { if a.is_some_and(|a| a.until <= tick) { *a = None; } }
        let commands: Vec<String> = sim.get_entity(entity).map_or(Vec::new(), |e|
            (0..e.buff_count()).filter_map(|i| e.buff_at(i))
                .filter(|b| { let n = b.name(); n.starts_with("il_draw_") || n.starts_with("il_recall_") || n == "il_manifest" })
                .map(|b| b.name().to_string()).collect());
        for cmd in commands {
            if !self.marks.insert(cmd.clone()) { continue; }
            // round 88: the game's AI presses S1 / S2 whenever they're up; only a press his brain wanted counts (the
            // input AI turns the others into basic attacks when it can), so it no longer stalls his own plans
            let wanted = |k: usize| self.want_at[k] > 0 && tick <= self.want_at[k] + WANT_WINDOW;
            let accept = if cmd.starts_with("il_draw_") { wanted(0) } else if cmd.starts_with("il_recall_") { wanted(1) } else { true };
            if !accept { sim.entity_remove_buff(entity, &cmd); continue; }
            if cmd == "il_manifest" { self.empowerment=3; }
            else {
                let p: Vec<&str> = cmd.split('_').collect();
                if p.len() == 5 {
                    if let (Ok(x), Ok(y)) = (p[2].parse::<i64>(), p[3].parse::<i64>()) {
                        if p[1] == "draw" { self.draw(sim, entity, (x, y)); let i=self.selected; self.swords[i].locked_until=tick+LOCK_TICKS; }
                        if p[1] == "recall" { self.recall(sim, entity, (x, y)); }
                    }
                }
            }
            sim.entity_remove_buff(entity, &cmd);
        }
        if tick > self.last_mark + 10 { self.marks.clear(); self.last_mark = tick; }
        self.engravings.retain(|m|m.until>tick);
        self.think(sim,entity);
        self.idle_reclaim(sim,entity);
        self.ally_attacks(sim,entity);
        self.update_swords(sim,entity);
        self.redirect_formations(sim,entity);
        self.update_visuals(sim,entity);
        // round 91: auras every 3 ticks (their buffs are permanent until changed; the field art changes every 6)
        if tick.is_multiple_of(3) { self.update_auras(sim,entity); }
        self.render_marks(sim,entity);
        self.render_flags(sim,entity);
        self.show(sim, entity);
        self.note_wants(sim, player, entity);
    }
}

#[cfg(test)]
const WOBBLE_VECTOR: i64 = -9516;
/// Which of ticks 600..699 (bit t-600) he notices ally 3 at 50% (seed 70217): the lab checks the same mask.
#[cfg(test)]
const NOTICE_VECTOR: u128 = 820_915_055_570_322_631_965_375_425_196;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn grades_use_unrounded_accuracy() {
        assert_eq!(grade(99.0), Some(("Imperial", 120)));
        assert_eq!(grade(98.999), Some(("Perfect", 110)));
        assert_eq!(grade(95.0), Some(("Perfect", 110)));
        assert_eq!(grade(94.99), Some(("Refined", 100)));
        assert_eq!(grade(85.0), Some(("Refined", 100)));
        assert_eq!(grade(70.0), Some(("Stable", 85)));
        assert_eq!(grade(69.9), Some(("Crude", 70)));
        assert_eq!(grade(60.0), Some(("Crude", 70)));
        assert_eq!(grade(59.999), None);
        assert!((formation_accuracy(3_500, 35_000, 3) - (100.0 - 350_000.0 / 105_000.0)).abs() < 1e-9);
        assert_eq!(formation_accuracy(0, 35_000, 3), 100.0);
        assert_eq!(formation_accuracy(10_000_000, 35_000, 3), 0.0);
        // the lab checks the same vectors (editor/isliidlab.js selfTest)
        assert!((formation_accuracy(12_345, 55_000, 5) - 95.510_909_090_909_09).abs() < 1e-9);
    }

    #[test]
    fn plan_wobble_is_shared_with_the_lab() {
        // the lab recomputes these with BigInt (editor/isliidlab.js planWobble)
        assert_eq!(plan_wobble(0x1234_5678_9abc_def0, 777, 3, 2, wobble(0, None)), plan_wobble(0x1234_5678_9abc_def0, 777, 3, 2, wobble(0, None)));
        assert_eq!(plan_wobble(1, 1, 0, 0, wobble(7, Some(1))), 0);
        for r in 0..7 { let w = plan_wobble(99, 600, 4, 1, wobble(r, None)); assert!(w.abs() <= WOBBLE[r]); }
        assert_eq!(plan_wobble(70_217, 600, 2, 1, wobble(0, None)), WOBBLE_VECTOR);
        assert_eq!((0..100).filter(|&t| notices(70_217, 600 + t, 3, 50)).map(|t| 1u128 << t).sum::<u128>(), NOTICE_VECTOR);
    }

    #[test]
    fn every_sword_state_has_exactly_one_visual() {
        use SwordMode::*;
        for mode in [Orbit, Stage, Planted, Ready, Throw, Draw, Strike, Return] {
            for idle in [true, false] {
                let v = visual_for(mode, idle);
                // flying states are projectiles, grounded ones effects, orbiting ones the arsenal buff: never two
                match mode {
                    Orbit => assert_eq!(v, Visual::Orbit),
                    Draw => assert_eq!(v, Visual::Flying(true)),
                    Throw | Strike | Return => assert_eq!(v, Visual::Flying(false)),
                    Stage => assert_eq!(v, if idle { Visual::Grounded(false) } else { Visual::Flying(false) }),
                    Ready => assert_eq!(v, Visual::Grounded(true)),
                    Planted => assert_eq!(v, Visual::Grounded(false)),
                }
            }
        }
    }

    #[test]
    fn fly_angle_covers_the_full_turn() {
        let o = (500_000, 500_000);
        assert_eq!(fly_angle(o, (600_000, 500_000)), 0);    // right
        assert_eq!(fly_angle(o, (500_000, 600_000)), 2);    // down (y grows downward, like the scars)
        assert_eq!(fly_angle(o, (400_000, 500_000)), 4);    // left
        assert_eq!(fly_angle(o, (500_000, 400_000)), 6);    // up
        assert_eq!(fly_angle(o, (600_000, 600_000)), 1);
        for k in 0..8 {
            let a = k as f64 * std::f64::consts::TAU / 8.0;
            let to = (o.0 + (a.cos() * 100_000.0).round() as i64, o.1 + (a.sin() * 100_000.0).round() as i64);
            assert_eq!(fly_angle(o, to), k);
            // a stroke drawn along the flight has the same line angle (trail_angle: half a turn in 16 steps)
            assert_eq!(trail_angle(o, to), (4 * k) % 16, "k {k}");
        }
    }

    /// Round 96: (sprites emitted, ticks, longest gap) for sword `i` flying `distance` from rest, with the
    /// update_visuals cadence (an emission whenever the last one's life is over, aligned to the cadence).
    fn flight_emissions(i: usize, distance: i64) -> (usize, usize, usize) {
        let ticks = flight_ticks(i, SwordMode::Stage, distance, 0);
        let (mut count, mut next, mut last, mut gap) = (0, 0, 0, 0);
        for t in 0..ticks {
            if t < next { continue; }
            let step = fly_cadence(sword_speed(i, SwordMode::Stage, t));
            if count > 0 { gap = gap.max(t - last); }
            count += 1; last = t; next = t + step - t % step;
        }
        (count, ticks, gap.max(ticks - last))
    }

    #[test]
    fn flying_swords_cost_less() {
        // round 95 redrew every flying sword every 3 ticks; the comets are redrawn every 6 while slow
        for i in 0..7 {
            let (count, ticks, gap) = flight_emissions(i, 60_000);
            let old = ticks.div_ceil(3);
            assert!(count * 100 <= old * 60, "sword {i}: {count} vs {old}");
            assert!(gap <= SLOW_FLY_STEP, "sword {i}: a {gap}-tick gap");
        }
        assert_eq!(fly_cadence(LAUNCH_SPEED), SLOW_FLY_STEP);
        assert_eq!(fly_cadence(SPEED[3]), FAST_FLY_STEP);   // a fast basic-attack throw stays smooth
        assert!(comet_tag(0, 8, 3, SLOW_FLY_STEP, 18).ends_with("rank8_comet_a3_pair3"));
        assert!(comet_tag(6, 7, 0, FAST_FLY_STEP, 9).ends_with("rank7_comet_a0_pair3"));
    }

    #[test]
    fn every_visual_name_exists_in_the_data() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../mods/tfm2_custom/champion/tfm2_isliid_emperor.data_champion");
        let Ok(text) = std::fs::read_to_string(&root) else { return };   // the data isn't next to the source
        let names: HashSet<&str> = text.split("\"name\": \"").skip(1).filter_map(|t| t.split('"').next()).collect();
        assert!(names.len() > 4_500, "{} view names parsed", names.len());
        let has = |name: &str| names.contains(name);
        let p = "tfm2_isliid_emperor_";
        for r in 0..8 {
            for (i, s) in SWORDS.iter().enumerate() {
                for art in [r, 8] { for h in 0..8 {
                    // round 99: pairs only, the single-frame aliases are gone (half the data)
                    assert!(!has(&format!("{p}{s}_rank{art}_comet_a{h}_frame0")), "{s} {art} comet {h} still has frames");
                    for k in 0..4 { assert!(has(&format!("{p}{s}_rank{art}_comet_a{h}_pair{k}")), "{s} {art} comet {h} p{k}"); }
                } }
                for k in 0..PLANTED_FRAMES / 2 {
                    for st in ["planted", "ready"] { assert!(has(&format!("{p}{s}_rank{r}_{st}_pair{k}")), "{s} {r} {st} pair {k}"); }
                }
                for k in 0..4 { assert!(has(&format!("{p}aura_field_{i}_rank{r}_pair{k}"))); }
                assert!(!has(&format!("{p}aura_field_{i}_rank{r}_frame0")));
                assert!(has(&arsenal_buff(i, r, false)) && has(&arsenal_buff(i, r, true)));
            }
        }
        for s in SWORDS {
            for fx in ["launch", "recall", "impact", "hit"] { assert!(has(&format!("{p}{s}_{fx}")), "{s}_{fx}"); }
        }
        for i in 0..7 { for top in [false, true] { for way in ["out", "in"] {
            assert!(has(&format!("{p}{}", wormhole(way, i, top))), "{}", wormhole(way, i, top));
        } } }
        for n in 0..8 { for top in [false, true] { assert!(has(&hole_buff(n, top)), "{}", hole_buff(n, top)); } }
        // round 98: Imperial's falls, constellation strokes and star-burst hits
        for i in 0..7 {
            for r in [35_000, 55_000] { for top in [false, true] { assert!(has(&format!("{p}{}", fall_name(i, r, top))), "{}", fall_name(i, r, top)); } }
            for a in 0..16 { for kind in ["scar", "flare", "scar_dim"] { assert!(has(&format!("{p}{kind}_{i}_t4_a{a}")), "{kind} {i} t4 a{a}"); } }
            assert!(has(&format!("{p}{}_hit_cosmic", SWORDS[i])));
        }
        for t in 0..4 {
            for i in 0..7 { for a in 0..16 { for kind in ["scar", "flare", "scar_dim"] {
                assert!(has(&format!("{p}{kind}_{i}_t{t}_a{a}")), "{kind} {i} t{t} a{a}");
            } } }
            for f in 0..FAMILIES.len() {
                for radius in [35_000, 55_000] { assert!(has(&format!("{p}{}", fire_name(f, t, radius))), "fire {f} t{t}"); }
                assert!(has(&format!("{p}hitmark_{}_t{t}", FAMILIES[f])));
            }
            assert!(has(&format!("{p}shatter_t{t}")));
        }
        assert!(has(&format!("{p}crown_flash")));
        for r in 0..7 { assert!(has(&format!("il_rank{r}"))); }
        for ph in [FlagPhase::Planned, FlagPhase::Drawing, FlagPhase::Complete, FlagPhase::Cancelled] {
            for i in 0..PATTERNS.len() { assert!(has(&format!("{p}{}", logo_name(&format!("pattern_{i}"), ph).unwrap()))); }
            for k in 0..7 { assert!(has(&format!("{p}{}", logo_name(&format!("solo_{k}"), ph).unwrap()))); }
            if ph == FlagPhase::Complete {
                for i in 0..PATTERNS.len() { for f in 0..POP_FRAMES {
                    assert!(has(&format!("{p}{}_f{f}", logo_name(&format!("pattern_{i}"), ph).unwrap())));
                } }
                for k in 0..7 { for f in 0..POP_FRAMES {
                    assert!(has(&format!("{p}{}_f{f}", logo_name(&format!("solo_{k}"), ph).unwrap())));
                } }
            }
        }
        for n in 1..=10 { assert!(has(&format!("il_imperial{n}"))); }
    }

    /// Round 93: a busy fight over 10 s: a 5-leg formation (legs of 70000) fires every 2 s, each leg drawn half a second
    /// before; 3 unfired strokes lie about; 7 swords on the ground, 3 flights to a point, 2 chasing flights, 3 logos.
    /// Everything is counted (strokes, grounded swords, their fields, flights, logos) against the round 92 cadence and
    /// stroke life, and every live stroke still shows.
    #[test]
    fn busy_fight_effect_budget() {
        let (start, span) = (10_000usize, 600usize);
        let mut isliid = Isliid { rank: Some(6), ..Isliid::default() };
        let mut id = 0;
        // (born, fired, from, to) of every stroke the fight leaves, oldest first (the last 20 s, as round 92 kept them)
        let mut strokes = Vec::new();
        for f in 0..15usize {
            let fired = start + span - f * 120;
            for leg in 0..5usize {
                let (x, y) = (200_000 + f as i64 * 40_000, 200_000 + leg as i64 * 20_000);
                strokes.push((fired - 30, Some(fired), (x, y), (x + 70_000, y + 10_000), leg));
            }
        }
        for k in 0..3usize { let y = 700_000 + k as i64 * 30_000; strokes.push((start + 100 * k, None, (100_000, y), (170_000, y), k)); }
        let mut old = 0usize;
        for &(born, fired, from, to, sword) in &strokes {
            id += 1;
            let until = match fired { Some(t) => (born + MARK_LIFE).min(t + FIRED_LIFE), None => born + MARK_LIFE };
            isliid.engravings.push(EngravingMark { lit: fired.unwrap_or(0), born, sword, from, to, until, id });
            // round 92: lived 1200 ticks; hot 300 ticks, repainted every 12; cooled every 60; drawn at once (6-tick pass)
            let pieces = ((sqdist(from, to) as f64).sqrt() / SCAR_STEP as f64).ceil() as usize;
            for tick in (start..start + span).filter(|t| t % 6 == 0 && *t >= born && *t < born + 1200) {
                let age = tick - born;
                if (age < 300 && (tick % 12 == 0 || age < 6)) || (age >= 300 && (tick % 60 == 0 || age < 306)) { old += pieces; }
            }
        }
        let all = isliid.engravings.clone();
        let mut new: usize = (start..start + span).map(|t| {
            isliid.engravings = all.iter().filter(|m| m.born <= t && m.until > t).cloned().collect();
            isliid.mark_sprites(t).len()
        }).sum();
        // grounded swords (pairs every 12) and their fields (frames every 6 then, pairs every 12 now)
        old += 7 * span / PAIR_STEP + 7 * span / 6;
        new += 7 * span / PAIR_STEP + 7 * span / PAIR_STEP;
        // flights: to a point every 6 then, chasing every 3 (round 92 projectiles, which never rendered); now five
        // comets on 60000 trips, at the round 96 cadence
        old += 3 * span / 6 + 2 * span / 3;
        let (sprites, ticks) = (0..7).map(|i| flight_emissions(i, 60_000)).fold((0, 0), |a, (c, t, _)| (a.0 + c, a.1 + t));
        new += 5 * span * sprites / ticks;
        // logos: every 6 then, every 12 now (still ones)
        old += 3 * span / 6;
        new += 3 * span / 12;
        let (new_s, old_s) = (new * 60 / span, old * 60 / span);
        eprintln!("effect spawns a second in a busy fight: round 92 {old_s}, round 98 {new_s}");
        assert!(new * 100 <= old * 65, "at least a 35% cut: round 92 {old_s}/s vs round 98 {new_s}/s");
        // every live stroke still shows within one cool cadence
        let at = start + span;
        isliid.engravings = all.iter().filter(|m| m.born <= at && m.until > at).cloned().collect();
        for m in &isliid.engravings {
            let shown = (at..at + SCAR_COOL_EVERY).any(|t| isliid.mark_sprites(t).iter()
                .any(|(name, _, _)| name.contains(&format!("_{}_t", m.sword))));
            assert!(shown);
        }
    }

    /// Round 93: a fired formation's strokes flare, then are gone within FIRED_LIFE; an unfired one lasts MARK_LIFE;
    /// no sprite outlives its stroke.
    #[test]
    fn fired_formations_fade_fast() {
        let mut isliid = Isliid { rank: Some(7), ..Isliid::default() };
        for k in 0..3u64 {
            isliid.engravings.push(EngravingMark { lit: 0, born: 1_000, sword: k as usize, from: (0, k as i64 * 50_000),
                to: (70_000, k as i64 * 50_000), until: 1_000 + MARK_LIFE, id: k + 1 });
        }
        isliid.light(&[1, 2], 1_030);
        assert_eq!(isliid.engravings[0].until, 1_030 + FIRED_LIFE);
        assert_eq!(isliid.engravings[2].until, 1_000 + MARK_LIFE, "an unfired stroke keeps its life");
        assert!(FIRED_LIFE <= 90 && MARK_LIFE <= 480);
        for t in (1_030..1_000 + MARK_LIFE).step_by(6) {
            isliid.engravings.retain(|m| m.until > t);
            for (name, _, life) in isliid.mark_sprites(t) {
                let sword: usize = name.split('_').rev().nth(2).unwrap().parse().unwrap();
                let m = isliid.engravings.iter().find(|m| m.sword == sword).unwrap();
                assert!(t + life as usize <= m.until, "{name} at {t} lives past its stroke");
            }
        }
        isliid.engravings.retain(|m| m.until > 1_030 + FIRED_LIFE);
        assert_eq!(isliid.engravings.len(), 1);
    }

    /// Round 93: a fired stroke vanishes fast but still divides his next engravings' power for the round 92 stroke life
    /// (so the shorter strokes don't make him stronger); a recall still frees the sword as before.
    #[test]
    fn fired_strokes_still_count_against_power() {
        let mut i = Isliid::default();
        i.commit_until[2] = 1_000 + COMMIT_LIFE;
        i.engravings.push(EngravingMark { lit: 0, born: 1_000, sword: 2, from: (0, 0), to: (70_000, 0), until: 1_000 + MARK_LIFE, id: 1 });
        i.light(&[1], 1_030);
        assert_eq!(i.engravings[0].until, 1_030 + FIRED_LIFE);
        assert_eq!(i.commit_until[2], 1_000 + COMMIT_LIFE);
        assert_eq!(COMMIT_LIFE, 1_200, "round 92's stroke life");
        i.cancel_marks(2, 1_100, false);
        assert_eq!(i.commit_until[2], 1_160);
        i.cancel_marks(2, 1_110, true);
        assert_eq!(i.commit_until[2], 1_110);
    }

    /// Round 93: a leg of a live formation outlives the plan's deadline (far swords fly slower, so the last leg can come
    /// long after the first).
    #[test]
    fn plan_legs_outlive_the_deadline() {
        let mut isliid = plan_in_progress();
        isliid.formations[0].until = 2_000;
        let mut i = isliid.clone();
        i.swords[1].plan_id = Some(7);
        // mark_segment needs a sim for the tick only; reproduce its rule
        let plan_until = i.swords[1].plan_id.and_then(|id| i.formations.iter().find(|p| p.id == id && !p.completed)).map_or(0, |p| p.until + 30);
        assert!((100 + MARK_LIFE).max(plan_until) > 2_000);
        let src = include_str!("isliid.rs");
        assert!(src.contains("until: (tick + MARK_LIFE).max(plan_until)"), "mark_segment keeps plan legs alive");
    }

    #[test]
    fn fresh_strokes_show_at_once_and_cool_later() {
        let mut isliid = Isliid::default();
        isliid.engravings.push(EngravingMark { lit: 0, born: 1_007, sword: 2, from: (0, 0), to: (60_000, 0), until: 1_007 + MARK_LIFE, id: 1 });
        // drawn at tick 1007: the next 6-tick pass (1008) shows it, with life up to the hot cadence boundary
        let first = isliid.mark_sprites(1_008);
        assert_eq!(first.len(), 2, "two pieces of 30000");
        assert!(first.iter().all(|(n, _, life)| n.starts_with("scar_2_") && *life == (SCAR_HOT_EVERY - 1_008 % SCAR_HOT_EVERY) as u64));
        assert_eq!(first[0].1, (15_000, 0));
        // ten seconds on: a cooled groove
        let later = (1_007 + SCAR_HOT..1_007 + SCAR_HOT + 12).flat_map(|t| isliid.mark_sprites(t)).collect::<Vec<_>>();
        assert!(!later.is_empty() && later.iter().all(|(n, _, _)| n.starts_with("scar_dim_2_")), "{later:?}");
    }

    #[test]
    fn single_target_draws_a_shape() {
        // round 90: against one enemy with swords to spare, no rank above Bearer settles for the straight line
        for rank in 0..8 {
            let idx = choose_pattern(rank, 1, 7, true, |_| 0).unwrap();
            eprintln!("rank {rank}: {} ({} swords)", PATTERNS[idx].name, PATTERNS[idx].swords);
            if rank >= 1 { assert!(PATTERNS[idx].swords >= 3, "rank {rank} drew {}", PATTERNS[idx].name); }
            if rank >= 4 { assert!(PATTERNS[idx].swords >= 4); }
        }
        // a crowd still asks for more
        assert!(PATTERNS[choose_pattern(6, 4, 7, true, |_| 0).unwrap()].swords >= 6);
    }

    #[test]
    fn bearer_may_still_draw_lines() {
        assert_eq!(PATTERNS[choose_pattern(0, 1, 2, false, |_| 0).unwrap()].swords, 2);
        // two free swords: a line even for a master (the alternative is nothing)
        assert_eq!(PATTERNS[choose_pattern(7, 1, 2, false, |_| 0).unwrap()].swords, 2);
        for r in 1..8 { assert!(SHAPE_MIN[r] >= SHAPE_MIN[r - 1] && GATHER_WAIT[r] <= GATHER_WAIT[r.max(2) - 1]); }
        assert!(FORMATION_RESERVE >= 3);
    }

    /// A live formation with sword 0 flying to its start (as think() launches it) and sword 1 planted after its leg.
    fn plan_in_progress() -> Isliid {
        let mut isliid = Isliid::default();
        isliid.formations.push(FormationPlan { id: 7, pattern: 0, center: (500_000, 500_000), radius: 35_000,
            legs: vec![((470_000, 500_000), (530_000, 500_000))], until: 10_000, completed: false, target: None, redirected: false });
        for (i, mode) in [(0, SwordMode::Stage), (1, SwordMode::Planted)] {
            let s = &mut isliid.swords[i];
            s.mode = mode; s.auto_owned = true; s.plan_id = Some(7); s.pos = (300_000, 300_000);
        }
        isliid.swords[0].path = vec![(470_000, 500_000)];
        isliid.swords[0].pending_draw = vec![(530_000, 500_000)];
        isliid
    }

    #[test]
    fn far_target_plan_keeps_its_swords() {
        // round 89: a plan sword 200000+ from every enemy (or with its target out of sight) is not recalled while its
        // formation is live, in flight or already planted
        let mut i = plan_in_progress();
        assert!(!i.abandoned(0) && !i.abandoned(1));
        assert!(i.reserved(0) && i.reserved(1));
        assert!(!i.free_swords(99, 0).contains(&0) && !i.free_swords(99, 0).contains(&1));
        // the formation completes (or expires): the planted sword is done and comes home
        i.formations.clear();
        assert!(i.abandoned(1));
        assert!(!i.abandoned(0), "still flying to draw its stroke");
    }

    #[test]
    fn recall_never_takes_a_plan_sword() {
        let mut i = plan_in_progress();
        let at = |j: usize| if j == 0 { (470_000, 500_000) } else { (900_000, 900_000) };
        // the press lands on the plan sword: nothing idle to take, nothing comes back
        assert_eq!(i.recall_pick(false, (470_000, 500_000), at), None);
        i.selected = 0;
        assert_eq!(i.recall_pick(true, (0, 0), at), None);
        // an idle thrown sword far away is the one that comes back
        i.swords[4].mode = SwordMode::Planted;
        assert_eq!(i.recall_pick(false, (470_000, 500_000), at), Some(4));
    }

    #[test]
    fn draw_press_does_not_hijack_a_ready_plan_sword() {
        let mut i = plan_in_progress();
        // sword 0 lands and is armed with its stroke still to draw
        i.swords[0].mode = SwordMode::Ready; i.swords[0].path.clear();
        i.selected = 0;
        let at = |_| (0, 0);
        let pick = i.press_sword(99, 0, (480_000, 500_000), at).unwrap();
        assert!(pick != 0 && pick != 1);
        assert_eq!(i.swords[pick].mode, SwordMode::Orbit, "a sword in hand goes instead");
        // a Ready sword armed by a manual press is still drawn by the next press
        let mut manual = Isliid::default();
        manual.swords[3].mode = SwordMode::Ready; manual.selected = 3;
        assert_eq!(manual.press_sword(99, 0, (1, 1), at), Some(3));
        // every sword busy: the press does nothing
        for s in 2..7 { i.swords[s].holder = Some(42); }
        assert_eq!(i.press_sword(99, 0, (1, 1), at), None);
    }

    #[test]
    fn escort_roles_fit_the_ally() {
        let best=|missing,foes| (0..7).max_by_key(|&i|(escort_score(i,missing,foes),std::cmp::Reverse(i))).unwrap();
        assert!(matches!(best(70,1),1|3), "a low ally gets peel or escape");
        assert_eq!(best(0,1),2, "a healthy carry in a fight gets Darkbringer");
        assert_eq!(best(30,3),1, "a crowd on a hurt ally: Terra");
        assert!(escort_score(5,10,4) > escort_score(5,10,1), "Rift likes crowds");
        assert!(escort_score(3,80,1) > escort_score(3,20,1), "Gale is for escapes");
    }

    #[test]
    fn objectives_need_a_teammate() {
        assert!(!objective_ok((500_000,500_000), &[]));
        assert!(!objective_ok((500_000,500_000), &[(1,(700_000,500_000))]));
        assert!(objective_ok((500_000,500_000), &[(1,(560_000,500_000))]));
    }

    #[test]
    fn far_engravings_deal_less() {
        // round 92: he reaches anywhere, but the damage falls off with distance from him
        assert_eq!(damage_pct(0), 100);
        assert_eq!(damage_pct(60_000), 100);
        assert!((78..=80).contains(&damage_pct(100_000)), "{}", damage_pct(100_000));
        assert!((51..=53).contains(&damage_pct(150_000)), "{}", damage_pct(150_000));
        assert_eq!(damage_pct(200_000), 25);
        assert_eq!(damage_pct(900_000), 25);
        for d in (0..300_000).step_by(5_000) { assert!(damage_pct(d) >= damage_pct(d + 5_000)); }
    }

    #[test]
    fn lead_is_capped() {
        // a fast enemy seen 30 ticks ago, forecast 300 ticks ahead: never more than the rank's cap from where he is
        let now = (500_000, 500_000);
        for r in 0..8 {
            let f = forecast(now, Some(((440_000, 500_000), 970)), 1_000, LOOK_AHEAD[r], LEAD_CAP[r]);
            assert!(sqdist(f, now) <= sq(LEAD_CAP[r] + 1), "rank {r}: {f:?}");
        }
    }

    #[test]
    fn no_engraving_on_an_empty_camp() {
        let camp = (600_000, 600_000);
        assert!(!contested(camp, &[]));
        assert!(!contested(camp, &[(5, (800_000, 600_000), 500)]));
        assert!(contested(camp, &[(5, (650_000, 600_000), 500)]));
    }

    #[test]
    fn big_formation_burst_budget() {
        // round 92: a 7-sword Imperial formation used to launch, fly and show its aura field for every sword on one
        // tick (3 spawns each); now the swords leave LAUNCH_STAGGER apart and fly without a field (2 spawns each)
        let old_peak = 7 * 3;
        let mut per_tick: HashMap<usize, usize> = HashMap::new();
        for k in 0..7 { *per_tick.entry(launch_tick(1_000, k)).or_insert(0) += 2; }
        let new_peak = *per_tick.values().max().unwrap();
        assert!(new_peak * 2 <= old_peak, "peak {new_peak} vs {old_peak}");
        assert!(launch_tick(1_000, 6) <= 1_000 + 20, "the volley still reads as one formation");
        // seven strokes being drawn at once: repainted half as often as the old 6-tick cadence
        let mut isliid = Isliid { rank: Some(7), ..Isliid::default() };
        for i in 0..7 {
            let s = &mut isliid.swords[i];
            s.mode = SwordMode::Draw; s.leg_from = (400_000, 400_000 + i as i64 * 10_000); s.pos = (460_000, 400_000 + i as i64 * 10_000);
        }
        let new_total: usize = (1_000..1_024).map(|t| isliid.mark_sprites(t).len()).sum();
        let old_total = 7 * 2 * (24 / 6);   // two pieces per 60000 stroke, every 6 ticks
        assert!(new_total > 0 && new_total * 2 <= old_total, "{new_total} vs {old_total}");
        assert!((1_000..1_024).map(|t| isliid.mark_sprites(t).len()).max().unwrap() <= 7 * 2);
    }

    /// Round 98: per-tick effects as a 7-leg formation (70000 legs, drawn together) fires at tick 1000 at `rank`:
    /// its fire (1) and hit markers (6) on the fire tick, plus every stroke sprite after.
    fn fire_profile(rank: usize) -> Vec<usize> {
        let mut isliid = Isliid { rank: Some(rank), ..Isliid::default() };
        for k in 0..7usize {
            let (x, y) = (300_000 + k as i64 * 50_000, 500_000);
            isliid.engravings.push(EngravingMark { lit: 0, born: 990 + k, sword: k, from: (x, y), to: (x + 70_000, y), until: 990 + MARK_LIFE, id: k as u64 + 1 });
        }
        let ids: Vec<u64> = (1..=7).collect();
        isliid.light(&ids, 1_000);
        (1_000..1_150).map(|t| { isliid.engravings.retain(|m| m.until > t); isliid.mark_sprites(t).len() + if t == 1_000 { 7 } else { 0 } }).collect()
    }

    #[test]
    fn formation_fire_has_no_spike() {
        // before round 98 the same moment emitted every leg's flare at once (21 pieces), a burst per leg (7), the
        // formation burst and up to 6 hit markers: 30+ on one tick
        for rank in [3, 6, 7] {
            let profile = fire_profile(rank);
            let peak = *profile.iter().max().unwrap();
            eprintln!("rank {rank}: peak {peak} effects on one tick, {} over 2.5 s", profile.iter().sum::<usize>());
            assert!(peak <= 12, "rank {rank}: peak {peak}");
        }
        // the legs still fade after firing
        assert!(fire_profile(5)[FIRED_LIFE + 6 * RIPPLE + 6..].iter().all(|&n| n == 0));
    }

    #[test]
    fn imperial_fall_follows_the_dominant_sword() {
        let s = |sword, len| (sword, (0, 0), (len, 0));
        assert_eq!(dominant_sword(&[s(2, 60_000), s(5, 40_000), s(0, 50_000)]), 2, "Darkbringer drew the most");
        assert_eq!(dominant_sword(&[s(4, 30_000), s(1, 30_000)]), 1, "a tie goes to the lower sword");
        assert_eq!(dominant_sword(&[s(6, 20_000), s(6, 20_000), s(3, 35_000)]), 6, "two strokes add up");
        assert_eq!(fall_name(2, 35_000, false), "fall_darkbringer_r0");
        assert_eq!(fall_name(6, 55_000, true), "fall_emperor_r1_p");
        assert_eq!(stroke_tier(7), 4);
        assert_eq!(stroke_tier(6), tier(6));
    }

    #[test]
    fn swords_come_out_of_the_black_hole() {
        assert_eq!(hole_buff(3, false), "il_blackhole_n3");
        assert_eq!(hole_buff(9, true), "il_blackhole1_n7");
        assert_eq!(wormhole("out", 2, false), "wormhole_out_darkbringer");
        assert_eq!(wormhole("in", 2, true), "wormhole_in_p");
        // drawn at the hole on leaving, blending to its path over HOLE_BLEND
        assert_eq!(hole_lift(0), HEAD_LIFT);
        assert_eq!(hole_lift(HOLE_BLEND), (0, 0));
        assert_eq!(hole_lift(900_000), (0, 0));
        let half = hole_lift(HOLE_BLEND / 2);
        assert!(half.1 < 0 && half.1 > HEAD_LIFT.1, "{half:?}");
        assert!(HEAD_LIFT.1 < 0, "the hole is above his head (y grows downward)");
    }

    #[test]
    fn redirect_clamps_by_rank() {
        for r in 1..8 { assert!(redirect_max(r, None) > redirect_max(r - 1, None)); }
        assert!(redirect_max(7, Some(1)) > redirect_max(7, Some(2)));
        assert_eq!(redirect_delta((0, 0), (3_000, 4_000), 50_000), None, "a small shift isn't worth it");
        assert_eq!(redirect_delta((0, 0), (30_000, 40_000), 100_000), Some((30_000, 40_000)));
        assert_eq!(redirect_delta((0, 0), (30_000, 40_000), 20_000), Some((12_000, 16_000)), "capped along the line");
    }

    #[test]
    fn redirect_moves_the_whole_shape_once() {
        let mut isliid = Isliid::default();
        let legs = pattern_legs(1, 3, (400_000, 400_000), 35_000);
        isliid.formations.push(FormationPlan { id: 3, pattern: 1, center: (400_000, 400_000), radius: 35_000,
            legs: legs.clone(), until: 500, completed: false, target: Some(9), redirected: false });
        isliid.flags.push(EngravingFlag { id: 3, center: (400_000, 400_000), tag: "pattern_1".into(), phase: FlagPhase::Planned, until: 500 });
        for (i, &(a, b)) in legs.iter().enumerate() {
            let s = &mut isliid.swords[i];
            s.mode = SwordMode::Stage; s.path = vec![a]; s.goal = a; s.pending_draw = vec![b]; s.plan_id = Some(3);
        }
        let d = (25_000, -10_000);
        isliid.shift_plan(0, d);
        let plan = &isliid.formations[0];
        assert_eq!(plan.center, (425_000, 390_000));
        for (j, &(a, b)) in legs.iter().enumerate() {
            let moved = ((a.0 + d.0, a.1 + d.1), (b.0 + d.0, b.1 + d.1));
            assert_eq!(plan.legs[j], moved);
            let s = &isliid.swords[j];
            assert_eq!((s.goal, s.path[0], s.pending_draw[0]), (moved.0, moved.0, moved.1));
        }
        assert_eq!(isliid.flags[0].center, (425_000, 390_000));
        assert!(plan.until > 500 && isliid.flags[0].until == plan.until);
        assert!(isliid.swords[3].goal == (0, 0), "swords of other plans stay put");
    }

    #[test]
    fn swords_start_slow_and_speed_up() {
        // round 94: every sword leaves at LAUNCH_SPEED and reaches TOP_PCT% of its SPEED after RAMP_TICKS in the air
        for i in 0..7 {
            for m in [SwordMode::Stage, SwordMode::Draw, SwordMode::Strike, SwordMode::Return] {
                assert_eq!(sword_speed(i, m, 0), LAUNCH_SPEED, "sword {i} {m:?}");
                for t in 0..300 { assert!(sword_speed(i, m, t + 1) >= sword_speed(i, m, t)); }
            }
            assert_eq!(sword_speed(i, SwordMode::Stage, RAMP_TICKS), SPEED[i] * TOP_PCT / 100);
            assert_eq!(sword_speed(i, SwordMode::Stage, 10_000), SPEED[i] * TOP_PCT / 100);
            assert!(sword_speed(i, SwordMode::Return, 60) > sword_speed(i, SwordMode::Stage, 60));
            // basic-attack throws stay fast, to match the instant hit
            assert_eq!(sword_speed(i, SwordMode::Throw, 0), SPEED[i]);
        }
    }

    #[test]
    fn flight_ticks_match_stepping() {
        for i in 0..7 {
            for d in [1_000, 60_000, 150_000, 300_000] {
                let (mut pos, mut t) = (0_i64, 0);
                while pos < d { pos += sword_speed(i, SwordMode::Stage, t); t += 1; }
                assert_eq!(flight_ticks(i, SwordMode::Stage, d, 0), t, "sword {i} {d}");
            }
        }
        assert_eq!(flight_ticks(0, SwordMode::Stage, 0, 0), 0);
    }

    #[test]
    fn short_flights_are_visible() {
        // a 60000 trip took about 7 ticks (0.1 s) before round 94; now it's well over half a second for every sword
        for i in 0..7 { assert!(flight_ticks(i, SwordMode::Stage, 60_000, 0) >= 30, "sword {i}"); }
        // a cross-map flight still arrives within about 2.5 s
        for i in 0..7 { assert!(flight_ticks(i, SwordMode::Stage, 300_000, 0) <= 160, "sword {i}"); }
    }

    #[test]
    fn far_strikes_cool_down_longer() {
        for r in 0..8 {
            assert_eq!(strike_gap(r, 30_000), STRIKE_GAP[r]);
            assert_eq!(strike_gap(r, FAR_R), STRIKE_GAP[r] * STRIKE_FAR_PCT / 100);
            assert_eq!(strike_gap(r, 1_000_000), STRIKE_GAP[r] * STRIKE_FAR_PCT / 100);
            assert!(strike_gap(r, 130_000) > STRIKE_GAP[r] && strike_gap(r, 130_000) < strike_gap(r, FAR_R));
        }
    }

    #[test]
    fn waiting_volley_swords_stay_on_the_ring() {
        // round 93: a formation sword waiting its turn shows on his ring until it leaves, then flies
        let mut s = SwordMotion { mode: SwordMode::Stage, path: vec![(500_000, 500_000)], wait_until: 1_009, ..SwordMotion::default() };
        assert!(on_ring(&s, 3, 1_000) && on_ring(&s, 3, 1_008));
        assert!(!on_ring(&s, 3, 1_009));
        s.mode = SwordMode::Orbit; s.wait_until = 0;
        assert!(on_ring(&s, 3, 1_009));
        s.holder = Some(5);
        assert!(!on_ring(&s, 3, 1_009), "an escort shows on the ally");
    }

    #[test]
    fn escorts_go_anywhere_two_per_teammate() {
        // round 92: no escort range any more, and never more than two swords on one teammate (Imperial #1 too)
        assert_eq!(PER_ALLY, 2);
        for r in 0..8 { assert!(ESCORTS[r].min(PER_ALLY) <= 2 && ESCORTS[r].min(PER_ALLY) >= 1); }
        let src = include_str!("isliid.rs");
        let gate = ["ESCORT", "_R"].concat();
        assert!(!src.contains(&format!("const {gate}")), "no escort reach constant");
    }

    #[test]
    fn only_imperial_one_is_perfect() {
        assert_eq!(wobble(7, Some(1)), 0);
        for r in 0..7 { assert!(wobble(r, None) > 0); }
        for n in 2..=10 { assert!(wobble(7, Some(n)) > 0 && wobble(7, Some(n)) > wobble(7, Some(n - 1))); }
        assert_eq!(solo_quality(7, Some(1)), 100.0);
        assert!(solo_quality(7, Some(2)) < 100.0 && solo_quality(6, None) < solo_quality(7, Some(10)));
    }

    #[test]
    fn accuracy_and_cover_widen_with_rank() {
        for r in 1..7 {
            assert!(WOBBLE[r] < WOBBLE[r - 1] && SOLO_QUALITY[r] > SOLO_QUALITY[r - 1] && NOTICE[r] >= NOTICE[r - 1]);
        }
        assert!(WOBBLE[0] >= 3 * WOBBLE[5], "a much wider gap than round 90 (9000 vs 1000)");
        assert_eq!(notice_pct(7, Some(1)), 99);
        assert_eq!(notice_pct(7, Some(10)), NOTICE[7]);
        // the notice hash is a fair coin at 50%
        let n = (0..10_000).filter(|&t| notices(42, t, 1, 50)).count();
        assert!((4_500..5_500).contains(&n), "{n}");
    }

    #[test]
    fn idle_swords_come_home_at_every_rank() {
        for r in 0..8 { assert!(IDLE_RETURN[r] <= 240 && IDLE_RETURN[r] > 0); if r>0 { assert!(IDLE_RETURN[r] <= IDLE_RETURN[r-1]); } }
        // round 91: leases grow with mastery (a low rank lets go sooner)
        for r in 1..8 { assert!(REASSESS[r] >= REASSESS[r-1] && ESCORTS[r] >= ESCORTS[r-1] && STRIKE_GAP[r] <= STRIKE_GAP[r-1]); }
        let mut i = Isliid::default();
        i.swords[2].mode = SwordMode::Planted;        // a thrown sword, landed
        assert!(i.idle_grounded(2));
        i.swords[2].pending_draw = vec![(1,1)];       // ... unless it still has a stroke to draw
        i.swords[2].mode = SwordMode::Stage;
        assert!(!i.idle_grounded(2));
        i.swords[2].pending_draw.clear();
        i.swords[2].mode = SwordMode::Ready;          // or is armed
        assert!(!i.idle_grounded(2));
    }

    #[test]
    fn locked_swords_are_not_handed_out() {
        let mut i = Isliid::default();
        assert_eq!(i.free_swords(99, 10).len(), 7);
        i.swords[4].locked_until = 70;
        assert!(!i.free_swords(99, 10).contains(&4));
        assert!(i.free_swords(99, 70).contains(&4));
        i.swords[5].holder = Some(42);                // escorting an ally: not free
        assert!(!i.free_swords(99, 70).contains(&5));
        i.swords[5].holder = Some(99);                // orbiting Isliid himself: free
        assert!(i.free_swords(99, 70).contains(&5));
    }

    #[test]
    fn seven_swords_remain_distinct() {
        let mut i = Isliid::default();
        for s in 0..7 {
            assert_eq!(i.available(s), Some(s));
            i.anchors[s] = Some(Anchor { x: s as i64 * 20_000, y: 0, until: 100 });
            i.swords[s].mode = SwordMode::Stage;
        }
        assert_eq!(i.available(0), None);
        assert_eq!(i.nearest((60_000, 0), 2_000), Some(3));
        i.anchors[3] = None;
        i.swords[3].mode = SwordMode::Orbit;
        assert_eq!(i.available(3), Some(3));
    }

    #[test]
    fn planted_selection_uses_context_not_fixed_rotation() {
        let mut i = Isliid::default();
        i.anchors[0] = Some(Anchor { x: 0, y: 0, until: 100 });
        i.swords[0].mode = SwordMode::Stage;
        assert_eq!(i.attack_sword((30,100), 10, 30_000), Some(4));
        assert_eq!(i.attack_sword((100,100), 100, 30_000), Some(2));
        assert_eq!(i.attack_sword((100,100), 10, 10_000), Some(3));
    }

    #[test]
    fn complete_catalogue_has_drawable_legs() {
        assert_eq!(PATTERNS.len(), 30);
        for p in PATTERNS {
            assert!((2..=7).contains(&p.swords), "{}", p.name);
            let legs=pattern_legs(p.style,p.swords,(400_000,400_000),40_000);
            assert_eq!(legs.len(),p.swords,"{}",p.name);
            assert!(legs.iter().all(|&(a,b)|sqdist(a,b)>=sq(10_000)),"{}",p.name);
        }
    }

    #[test]
    fn trails_follow_cardinal_and_diagonal_paths() {
        assert_eq!(trail_angle((0,0),(20_000,0)),0);
        assert_eq!(trail_angle((0,0),(0,20_000)),8);
        assert_eq!(trail_angle((0,0),(20_000,20_000)),4);
        assert_eq!(trail_angle((0,0),(-20_000,20_000)),12);
        assert_eq!(trail_angle((20_000,20_000),(0,0)),4);
    }

    #[test]
    fn aura_strength_divides_across_overlapping_swords() {
        assert_eq!(aura_part(10_000, 1), 10_000);
        assert_eq!(aura_part(10_000, 7), 1_429);
        assert_eq!(aura_part(5, 7), 1);
    }

    #[test]
    fn every_sword_state_has_a_moving_aura_source() {
        let host=(100_000,100_000);
        let blade=(350_000,220_000);
        assert_eq!(aura_source(SwordMode::Orbit,blade,Some(host)),Some(host));
        assert_eq!(aura_source(SwordMode::Orbit,blade,None),None);
        for mode in [SwordMode::Stage,SwordMode::Planted,SwordMode::Ready,
                     SwordMode::Throw,SwordMode::Draw,SwordMode::Strike,SwordMode::Return] {
            assert_eq!(aura_source(mode,blade,None),Some(blade));
            assert!(near(aura_source(mode,blade,None).unwrap(),(389_999,220_000),40_000));
            assert!(!near(aura_source(mode,blade,None).unwrap(),(390_001,220_000),40_000));
        }
    }

    #[test]
    fn activated_strokes_lock_recall_until_planted() {
        let mut sword=SwordMotion::default();
        sword.mode=SwordMode::Stage;
        assert!(!sword.is_activated());
        sword.pending_draw.push((40_000,40_000));
        assert!(!sword.is_activated());
        assert!(sword.is_committed());
        sword.mode=SwordMode::Ready;
        assert!(sword.is_activated());
        sword.mode=SwordMode::Draw;
        sword.pending_draw.clear();
        assert!(sword.is_activated());
        sword.mode=SwordMode::Planted;
        assert!(!sword.is_activated());
    }

    #[test]
    fn every_mastery_rank_considers_simple_engravings() {
        for rank in 0..8 {
            let candidates:Vec<_>=candidate_patterns(rank).collect();
            assert!(candidates.contains(&0) && candidates.contains(&1));
            assert!(candidates.iter().all(|&index|index<PATTERNS.len()));
        }
    }

    #[test]
    fn abandoned_plan_does_not_block_new_drawing() {
        let mut isliid=Isliid::default();
        let legs=pattern_legs(0,2,(400_000,400_000),35_000);
        isliid.formations.push(FormationPlan { id:1, pattern:0, center:(400_000,400_000),
            radius:35_000, legs:legs.clone(), until:500, completed:false, target:None, redirected:false });
        for (i,(from,to)) in legs.into_iter().enumerate() {
            isliid.swords[i].mode=SwordMode::Stage;
            isliid.swords[i].goal=from;
            isliid.swords[i].pending_draw=vec![to];
        }
        isliid.prune_formations(5);
        assert_eq!(isliid.formations.len(),1);
        isliid.swords[0].mode=SwordMode::Return;
        isliid.swords[0].pending_draw.clear();
        isliid.prune_formations(6);
        assert!(isliid.formations.is_empty());
    }

    #[test]
    fn live_marks_complete_every_catalogue_shape() {
        for p in PATTERNS {
            let legs=pattern_legs(p.style,p.swords,(400_000,400_000),65_000);
            let marks:Vec<_>=legs.into_iter().enumerate().map(|(sword,(from,to))|
                EngravingMark{lit:0,born:0,sword,from,to,until:1800,id:sword as u64+1}).collect();
            let refs:Vec<_>=marks.iter().collect();
            assert!(match_live_drawing(p,&refs).is_some(),"{}",p.name);
        }
    }
}
