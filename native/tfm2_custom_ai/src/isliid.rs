//! Emperor Isliid: seven individually tracked swords and position-cast engravings.
//! All mutable combat state lives in the passive instance so replay/planning clones
//! cannot change the live match. Native effects only pass the cast point as a buff.

use crate::scribble::Memory;
use crate::{sq, timed, MOD_ID};
use mod_api_stable::{AttackTypeV1, BuffV1, CastingTargetV1, CcV1, InputTargetV1,
    ProjectileMoveKindV1, ProjectileSpawnV1, SimOriginV1, StableEffectType, StablePassive, StableSim, StatV1};
use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, OnceLock};

const SWORDS: [&str; 7] = ["skylight", "terra", "darkbringer", "gale", "blood", "rift", "emperor"];
const NAMES: [&str; 8] = ["Bearer", "Squire", "Engraver", "Tactician", "Swordmaster", "Regent", "Sovereign", "Imperial"];
const SNAP: i64 = 12_000;
const MELEE: i64 = 23_000;
const ANCHOR_LIFE: usize = 1800;
const MARK_LIFE: usize = 1800;
const RETURN_MULT: i64 = 3;
const RETURN_DIV: i64 = 2;
const SPEED: [i64; 7] = [8_000, 5_500, 7_000, 12_000, 7_500, 9_000, 6_500];
const THINK_TICKS: [usize; 8] = [90, 75, 60, 48, 38, 30, 22, 15];
const LOOK_AHEAD: [i64; 8] = [0, 30, 60, 90, 120, 180, 240, 300];
const PATTERN_BUDGET: [usize; 8] = [3, 5, 8, 12, 16, 21, 26, 30];
// Round 88 (sword control): escorts for threatened allies (how many per ally, how often the brain looks again), how
// soon an idle grounded sword comes home, how often an escort strikes from its host. Higher mastery reassesses and
// reallocates sooner; no sword is rank-locked.
const ESCORTS: [usize; 8] = [1, 1, 2, 2, 2, 3, 3, 4];
const REASSESS: [usize; 8] = [150, 130, 110, 90, 75, 60, 45, 30];
const IDLE_RETURN: [usize; 8] = [240, 210, 180, 150, 120, 100, 80, 60];
const STRIKE_GAP: [usize; 8] = [90, 84, 78, 72, 66, 60, 54, 48];
/// An ally with an enemy champion this close (now or forecast) and missing 25% HP, or two enemies close, is threatened.
const THREAT_R: i64 = 105_000;
/// A cast is accepted only this soon after the brain wanted it (the 5-tick start timing fits inside).
const WANT_WINDOW: usize = 12;
/// A sword touched by an accepted command is left alone by the brain this long.
const LOCK_TICKS: usize = 60;

const SEG: usize = 3;
const FX_LAUNCH: u8 = 1;
const FX_RECALL: u8 = 2;
const FX_IMPACT: u8 = 4;

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

/// Where a `life`-tick flight segment from `pos` toward `goal` at `speed` ends (never past the goal).
fn segment_end(pos: (i64, i64), goal: (i64, i64), speed: i64, life: usize) -> (i64, i64) {
    let d = (sqdist(pos, goal) as f64).sqrt();
    let reach = (speed * life as i64) as f64;
    if d <= reach || d < 1.0 { return goal; }
    (pos.0 + ((goal.0 - pos.0) as f64 * reach / d) as i64, pos.1 + ((goal.1 - pos.1) as f64 * reach / d) as i64)
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

fn mod_dir() -> Option<std::path::PathBuf> {
    std::env::current_exe().ok().and_then(|e| e.parent().map(|p| p.join("mods").join(MOD_ID)))
}

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

fn record_game(seed: u64, player: usize, athlete: usize, match_id: u64) {
    let key = format!("{seed:x}.{player}.{athlete}");
    let Ok(mut session) = memory().lock() else { return };
    if session.recorded.len() > 200_000 { session.recorded.clear(); }
    if !session.recorded.insert(key.clone()) { return; }
    let prefix = if match_id == SimOriginV1::NONE { "x".to_string() } else { match_id.to_string() };
    let line = format!("g {prefix}.{seed:x}-{player} {athlete}\n");
    session.memory.merge(&line);
    let Some(dir) = mod_dir() else { return };
    use std::io::Write;
    let _ = std::fs::OpenOptions::new().create(true).append(true)
        .open(dir.join("isliid_pending.txt"))
        .and_then(|mut f| f.write_all(line.as_bytes()));
}

fn record_result(seed: u64, player: usize, athlete: usize, match_id: u64,
                 tick: usize, result: (i64, i64, i64, i64, i64)) {
    let Some(dir) = mod_dir() else { return };
    let prefix = if match_id == SimOriginV1::NONE { "x".to_string() } else { match_id.to_string() };
    let sig = format!("{prefix}.{seed:x}-{player}");
    let line = format!("r {sig} {athlete} {tick} {} {} {} {} {}\n",
        result.0, result.1, result.2, result.3, result.4);
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

#[derive(Clone, Copy, PartialEq, Eq)]
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
    activation_host: Option<usize>,
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
}

impl Default for SwordMotion {
    fn default() -> Self {
        Self { pos: (0, 0), goal: (0, 0), leg_from: (0, 0), mode: SwordMode::Orbit,
            holder: None, last_ally: None, target: None, path: Vec::new(), pending_draw: Vec::new(),
            activate_on_arrival: false, ready_at: 0, activation_host: None,
            auto_owned: false, plan_id: None,
            waypoint: 0, planned: 0, travelled: 0, mark_start_id: 0, attack_at: 0,
            return_hits: HashSet::new(), vis_until: 0, fx_due: 0, idle_since: 0, escort_until: 0, locked_until: 0 }
    }
}

#[derive(Clone)]
struct EngravingMark {
    sword: usize,
    from: (i64, i64),
    to: (i64, i64),
    until: usize,
    id: u64,
    host: Option<usize>,
}

#[derive(Clone)]
struct FormationPlan {
    id: u64,
    host: Option<usize>,
    pattern: usize,
    center: (i64,i64),
    radius: i64,
    legs: Vec<((i64,i64),(i64,i64))>,
    until: usize,
    completed: bool,
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
    last_mark: usize,
    marks: HashSet<String>,
    dark_stacks: HashMap<usize, (usize, usize)>,
    last_result: Option<(i64, i64, i64, i64, i64)>,
    result_at: usize,
    want_at: [usize; 2],
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
}

impl Default for Isliid {
    fn default() -> Self {
        Self { anchors: [None; 7], selected: 0, grabbed: None, rank: None, imperial: None,
            shown: None, arsenal_shown: [None; 7], holder_shown: [None; 7], selected_shown: None, last_mark: 0,
            marks: HashSet::new(), dark_stacks: HashMap::new(),
            last_result: None, result_at: 0, want_at: [0; 2], gathering: false,
            prepared_at: 0, next_plan_at: 0, base_hit_ready: false, base_hit_until: 0,
            base_ready_at: 0, native_hit: false,
            ally_observed: HashMap::new(), aura_active: HashSet::new(), aura_visual_at: HashMap::new(),
            aura_base_shown: HashMap::new(),
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
    if distance <= 100_000 { 100 }
    else if distance >= 300_000 { 35 }
    else { 100 - ((distance-100_000) as usize * 65 / 200_000) }
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

    /// Round 88: every sword has exactly one visual. Flying: a short cosmetic projectile segment re-aimed every 3
    /// ticks at its (possibly moving) goal, its frame alias chosen by the tick so the smear animates without restarting
    /// (projectiles can't be removed, so a new segment never starts before the last one ends); grounded: a frame alias
    /// of the planted / ready loop every 3 ticks; orbiting: the arsenal buff on its holder (show(), once the last
    /// segment is over). Launch, recall-snap and plant-impact effects play once on the change.
    fn update_visuals(&mut self, sim: &mut StableSim<'_>, entity: usize) {
        let tick=sim.tick();
        let rank=self.rank();
        let Some(team)=sim.get_entity(entity).map(|e|e.team()) else {return};
        for i in 0..7 {
            let due=std::mem::take(&mut self.swords[i].fx_due);
            let pos=self.swords[i].pos;
            if due & FX_LAUNCH != 0 { Self::fx(sim,entity,&format!("{}_launch",SWORDS[i]),pos,0); }
            if due & FX_RECALL != 0 { Self::fx(sim,entity,&format!("{}_recall",SWORDS[i]),pos,0); }
            if due & FX_IMPACT != 0 { Self::fx(sim,entity,&format!("{}_impact",SWORDS[i]),pos,0); }
            if tick<self.swords[i].vis_until {continue}
            let life=SEG-(tick%SEG);
            match visual_for(self.swords[i].mode,self.swords[i].path.is_empty()) {
                Visual::Orbit => {}
                Visual::Grounded(ready) => {
                    let tag=format!("{}_rank{rank}_{}_frame{}",SWORDS[i],if ready {"ready"} else {"planted"},(tick/6)%8);
                    Self::fx(sim,entity,&tag,pos,life as u64);
                    self.swords[i].vis_until=tick+life;
                }
                Visual::Flying(drawing) => {
                    let s=&self.swords[i];
                    let speed=SPEED[i]*if s.mode==SwordMode::Return {RETURN_MULT} else {1}/if s.mode==SwordMode::Return {RETURN_DIV} else {1};
                    let end=segment_end(pos,s.goal,speed,life);
                    if near(pos,end,500) {continue}
                    let spec=ProjectileSpawnV1 {
                        caster_id:entity, team, x:pos.0.max(0) as u64, y:pos.1.max(0) as u64, radius:1_000,
                        speed:speed.max(1) as u64, move_kind:ProjectileMoveKindV1::Linear.code(),
                        target_x:end.0.max(0) as u64, target_y:end.1.max(0) as u64,
                        penetrate:true, casting_target:CastingTargetV1::None.code(),
                        ..ProjectileSpawnV1::default()
                    };
                    sim.spawn_projectile(&format!("tfm2_isliid_emperor_{}_rank{rank}_{}_f{}",SWORDS[i],
                        if drawing {"drawing"} else {"flight"},(tick/6)%4),&format!("{MOD_ID}:noop"),&spec);
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

    fn remote_damage(&self, sim: &StableSim<'_>, caster: usize, host: Option<usize>,
                     at: (i64,i64), damage: usize) -> usize {
        let source=host.and_then(|id|sim.get_entity(id)).filter(|e|e.is_alive())
            .or_else(||sim.get_entity(caster));
        let Some(source)=source else {return damage*35/100};
        let p=source.pos();
        let distance=(sqdist((p.0 as i64,p.1 as i64),at) as f64).sqrt() as i64;
        damage.saturating_mul(damage_pct(distance))/100
    }

    // Short visual frames are repainted from the live ledger. A staging flight
    // never enters this ledger, so it cannot leave an engraving by accident.
    fn render_marks(&self, sim: &mut StableSim<'_>, entity: usize) {
        if sim.tick() % 4 != 0 { return; }
        let paint = |sim: &mut StableSim<'_>, sword: usize, from: (i64,i64), to: (i64,i64), remaining: usize| {
            let count = (((sqdist(from,to) as f64).sqrt() / 15_000.0).ceil() as usize).clamp(1,80);
            let angle = trail_angle(from, to);
            for k in 0..=count {
                if remaining < 60 && (k * 17 + sword * 7) % 60 >= remaining { continue; }
                let p = (from.0 + (to.0 - from.0) * k as i64 / count as i64,
                         from.1 + (to.1 - from.1) * k as i64 / count as i64);
                Self::fx(sim, entity, &format!("scar_{sword}_a{angle}"), p, 4);
            }
        };
        for mark in &self.engravings {
            paint(sim, mark.sword, mark.from, mark.to, mark.until.saturating_sub(sim.tick()));
        }
        for (i,sword) in self.swords.iter().enumerate().filter(|(_,s)|s.mode==SwordMode::Draw) {
            paint(sim, i, sword.leg_from, sword.pos, MARK_LIFE);
        }
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
        let s = &mut self.swords[sword];
        s.pos = from; s.goal = path[0]; s.leg_from = from; s.mode = mode; s.path = path;
        s.fx_due |= if mode == SwordMode::Return { FX_RECALL } else { FX_LAUNCH };
        s.waypoint = 0; s.planned = planned; s.travelled = 0; s.target = target;
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
        if instant { self.engravings.retain(|m| m.until > tick); }
    }

    fn mark_segment(&mut self, sim: &mut StableSim<'_>, entity: usize, sword: usize,
                    from: (i64, i64), to: (i64, i64)) {
        if near(from, to, 1_000) { return; }
        self.next_mark += 1;
        self.engravings.push(EngravingMark { sword, from, to,
            until: sim.tick() + MARK_LIFE, id: self.next_mark,
            host: self.swords[sword].activation_host });
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
            // round 88: a 24 x 24 effect logo just above the plan (it used to be a 120 x 48 text banner on top of it)
            let Some(name)=logo_name(&flag.tag,flag.phase) else {continue};
            Self::fx(sim,entity,&name,(flag.center.0,flag.center.1-20_000),6);
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

    fn engraving_host(&self, sim: &StableSim<'_>, entity: usize, p: (i64, i64)) -> Option<usize> {
        let team = sim.get_entity(entity)?.team();
        (0..sim.entity_count()).filter_map(|n| sim.entity_at(n))
            .filter(|e| e.is_alive() && e.is_champion() && e.team() == team)
            .filter_map(|e| { let q=e.pos(); let d=sqdist(p,(q.0 as i64,q.1 as i64));
                (d<=sq(100_000)).then_some((e.id(),d)) })
            .min_by_key(|(_,d)|*d).map(|(id,_)|id).or(Some(entity))
    }

    fn arm(&mut self, sim: &StableSim<'_>, entity: usize, sword: usize) {
        let p=self.swords[sword].pos;
        self.swords[sword].mode=SwordMode::Ready;
        self.swords[sword].ready_at=sim.tick();
        self.swords[sword].activation_host=self.engraving_host(sim,entity,p);
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
                && s.path.is_empty() && near(self.position(sim,entity,j),p,SNAP)
        }).min_by_key(|&j|sqdist(self.position(sim,entity,j),p)) {
            self.selected=i;
            if self.swords[i].mode!=SwordMode::Ready {self.arm(sim,entity,i);}
            return;
        }
        let i=self.selected;
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
        let Some(i) = (if to_last {Some(self.selected)} else {self.nearest(p, 28_000).or_else(||
            (0..7).min_by_key(|&j| sqdist(self.position(sim, entity, j), p)))}) else { return };
        if self.swords[i].is_activated() { return; }
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
            self.engravings.iter().any(|m| m.sword == j && m.until > sim.tick())).count().max(1);
        let quality = [70, 76, 81, 86, 90, 94, 97, 100][self.rank()];
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
        for (id, ally, p) in targets {
            match (sword, ally) {
                (0, false) => { sim.entity_set_invisible(id, 0); sim.add_buff(id, &timed("il_sky_mark", 90)); }
                (1, false) => { let mut b=timed("il_terra_mark", 100); b.move_speed_mult=-(strength as i32 / 3).max(8); sim.add_buff(id,&b); }
                (2, false) => { let mut b=timed("il_dark_mark", 150); b.defence_mult=-(strength as i32 / 3).max(8); sim.add_buff(id,&b); }
                (3, true) => { let mut b=timed("il_gale_mark", 120); b.move_speed_mult=(strength as i32 / 3).max(8); sim.add_buff(id,&b); }
                (4, false) => { let damage=self.remote_damage(sim,entity,self.swords[sword].activation_host,
                    p,20 + attack * strength / 100); self.native_damage(sim,entity,id,damage);
                    sim.heal(entity,entity,damage/5); }
                (5, false) => { let q=((p.0*4+center.0)/5,(p.1*4+center.1)/5); sim.entity_set_pos(id,q.0.max(0) as u64,q.1.max(0) as u64); }
                (6, true) => { let mut b=timed("il_emperor_mark", 120); b.attack_mult=(strength as i32 / 5).max(4); sim.add_buff(id,&b); }
                _ => {}
            }
        }
        Self::fx(sim, entity, if sword == 6 { "seal" } else { "scar" }, center, 14);
    }

    fn remote_hit(&mut self, sim: &mut StableSim<'_>, entity: usize, sword: usize, target: usize) {
        let (Some(me),Some(victim))=(sim.get_entity(entity),sim.get_entity(target)) else { return };
        if !victim.is_alive() || me.team()==victim.team() || !sim.is_visible(me.team(),target) { return; }
        let Some(holder)=self.swords[sword].holder.and_then(|id|sim.get_entity(id))
            .filter(|e|e.is_alive() && e.team()==me.team()) else {return};
        let h=holder.pos(); let v=victim.pos();
        if !near((h.0 as i64,h.1 as i64),(v.0 as i64,v.1 as i64),100_000) {return}
        let damage=25 + me.stat().attack * if sword==6 { 35 } else { 60 } / 100;
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
            let base=SPEED[i] * if mode==SwordMode::Return { RETURN_MULT } else { 1 }
                / if mode==SwordMode::Return { RETURN_DIV } else { 1 };
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
                self.swords[i].mode=SwordMode::Orbit;
                self.swords[i].path.clear();
                self.swords[i].auto_owned=false;
                self.swords[i].plan_id=None;
                self.anchors[i]=None;
            } else {
                self.swords[i].path.clear();
                if mode==SwordMode::Throw { self.anchors[i]=Some(Anchor{x:next.0,y:next.1,until:tick+ANCHOR_LIFE}); }
                if self.swords[i].holder.is_none() { self.swords[i].fx_due |= FX_IMPACT; }
                if mode==SwordMode::Stage && self.swords[i].activate_on_arrival {
                    self.arm(sim,entity,i);
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
            let future=forecast(p,self.ally_observed.get(&id).copied(),tick,LOOK_AHEAD[self.rank()]);
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
        let active_plans:HashSet<u64>=self.formations.iter().map(|p|p.id).collect();
        for i in 0..7 {
            let s=&self.swords[i];
            if !s.auto_owned || s.is_activated() {continue}
            let orphan=s.plan_id.is_none_or(|id|!active_plans.contains(&id));
            let unsupported=objective.is_none() && foes.iter().all(|(_,p,_)|!near(*p,s.pos,120_000));
            if !orphan && !unsupported {continue}
            if s.mode==SwordMode::Stage && !s.path.is_empty() && !unsupported {continue}
            if !matches!(s.mode,SwordMode::Stage|SwordMode::Planted) {continue}
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
            !self.swords.iter().any(SwordMotion::is_committed) &&
            !self.gathering && tick>=self.prepared_at+300 {
            self.gathering=true;
            for i in 0..7 {
                if self.swords[i].mode==SwordMode::Orbit && self.swords[i].holder.is_none() {continue}
                if self.swords[i].is_activated() { continue; }
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
            let future=forecast(p,self.observed.get(&id).copied(),tick,LOOK_AHEAD[self.rank()]);
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

        if let Some(center)=prediction.map(|(_,p)|p).or(objective) {
            if tick<self.next_plan_at {return}
            // Higher mastery tests more possible placements and uses more of
            // the Arsenal in a teamfight; no sword count is rank-locked.
            let density=foes.iter().filter(|(_,p,_)| near(*p,center,130_000)).count();
            let desired=if density>=4 {7} else if density>=3 {5} else if density>=2 {3} else {2};
            if self.formations.iter().any(|p|!p.completed && p.until>tick && near(p.center,center,100_000)) {return}
            let available:Vec<usize>=(0..7).filter(|&i| {
                let s=&self.swords[i];
                if tick<s.locked_until {return false}
                (s.mode==SwordMode::Orbit && s.holder.is_none()) ||
                (matches!(s.mode,SwordMode::Stage|SwordMode::Planted) && s.path.is_empty())
            }).collect();
            if available.is_empty() {return}
            let radius=if density>=3 {55_000} else {35_000};
            let mut best:Option<(usize,i64,Vec<((i64,i64),(i64,i64))>)>=None;
            for idx in candidate_patterns(self.rank()) {
                let spec=PATTERNS[idx];
                let legs=pattern_legs(spec.style,spec.swords,center,radius);
                let reused=legs.iter().filter(|&&(a,b)|self.engravings.iter().any(|m|
                    m.until>tick && ((near(a,m.from,12_000)&&near(b,m.to,12_000)) ||
                    (near(a,m.to,12_000)&&near(b,m.from,12_000))))).count();
                if reused==spec.swords {continue}
                if spec.swords.saturating_sub(reused)>available.len() {continue}
                let role=match spec.effect {7|10|11|12 if !allies.is_empty()=>20, 9 if density>=2=>25,
                    2|3|6 if density>=2=>12, _=>0};
                let score=100-(spec.swords as i64-desired as i64).abs()*15+role+reused as i64*20
                    -(spec.swords as i64*4);
                if best.as_ref().is_none_or(|(_,s,_)|score>*s) {best=Some((idx,score,legs));}
            }
            if let Some((idx,_,legs))=best {
                self.next_plan_id+=1;
                let plan_id=self.next_plan_id;
                self.formations.push(FormationPlan{id:plan_id,host:self.engraving_host(sim,entity,center),
                    pattern:idx,center,radius,
                    legs:legs.clone(),until:tick+120,completed:false});
                let mut free=available;
                let mut deadline=tick+120;
                let wobble=[9_000,7_000,5_000,3_500,2_000,1_000,500,0][self.rank()];
                for (j,(a,b)) in legs.into_iter().enumerate() {
                    if self.engravings.iter().any(|m|m.until>tick &&
                        ((near(a,m.from,12_000)&&near(b,m.to,12_000)) ||
                         (near(a,m.to,12_000)&&near(b,m.from,12_000)))) {continue}
                    if free.is_empty() {break}
                    let nearest=free.iter().enumerate().min_by_key(|(_,i)|sqdist(self.position(sim,entity,**i),a)).map(|(k,_)|k).unwrap_or(0);
                    let i=free.remove(nearest);
                    let salt=((sim.seed() ^ tick as u64 ^ ((i as u64)<<24) ^ j as u64).wrapping_mul(0x9e37_79b9)) as i64;
                    let error=if wobble==0 {0} else {salt.rem_euclid(wobble*2+1)-wobble};
                    let start=((a.0+error).clamp(0,1_000_000),(a.1-error).clamp(0,1_000_000));
                    let end=((b.0+error).clamp(0,1_000_000),(b.1-error).clamp(0,1_000_000));
                    let travel=((sqdist(self.position(sim,entity,i),start) as f64).sqrt()+
                        (sqdist(start,end) as f64).sqrt()) as usize / SPEED[i] as usize;
                    deadline=deadline.max(tick+travel+90);
                    self.send(sim,entity,i,SwordMode::Stage,vec![start],None);
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
                ready.push((plan.clone(),swords.len(),error,formation_key(plan.pattern,&used.into_iter().collect::<Vec<_>>() )));
            }
        }
        for (plan,distinct,error,key) in ready {
            if self.activated.insert(key) {
                if let Some(f)=self.flags.iter_mut().find(|f|f.id==plan.id) {
                    f.phase=FlagPhase::Complete;f.until=sim.tick()+120;
                }
                self.apply_formation(sim,entity,&plan,distinct,error);
            }
        }
        // Player-made paths count too. Match the newest connected group at any
        // rotation, so a focused S1 drawing can finish a catalogue formation.
        let mut inferred:Option<(FormationPlan,usize,i64,u64,i64)>=None;
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
            let host=recent.iter().find_map(|m|m.host).or(self.engraving_host(sim,entity,center));
            let plan=FormationPlan{id:0,host,pattern:spec_index,center,radius,
                legs:pattern_legs(spec.style,spec.swords,center,radius),
                until:sim.tick()+MARK_LIFE,completed:true};
            if inferred.as_ref().is_none_or(|(_,_,_,_,best)|score>*best) {
                inferred=Some((plan,distinct.len(),error,key,score));
            }
        }
        if let Some((plan,distinct,error,key,_))=inferred {
            if self.activated.insert(key) {
                self.next_plan_id+=1;
                self.flags.push(EngravingFlag{id:self.next_plan_id,center:plan.center,
                    tag:format!("pattern_{}",plan.pattern),phase:FlagPhase::Complete,until:sim.tick()+120});
                self.apply_formation(sim,entity,&plan,distinct,error);
            }
        }
        self.formations.retain(|p|p.until>sim.tick() && !p.completed);
        if self.activated.len()>10_000 {self.activated.clear();}
    }

    fn apply_formation(&mut self, sim: &mut StableSim<'_>, entity: usize,
                       plan: &FormationPlan, distinct: usize, error: i64) {
        let Some(me)=sim.get_entity(entity) else {return};
        let team=me.team(); let attack=me.stat().attack;
        let spec=PATTERNS[plan.pattern];
        let _pattern_name=spec.name;
        let committed=(0..7).filter(|&j|self.swords[j].mode==SwordMode::Draw ||
            self.engravings.iter().any(|m|m.sword==j && m.until>sim.tick())).count().max(1);
        let accuracy=(100-error*100/(plan.radius.max(1)*spec.swords as i64)).clamp(50,100) as usize;
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
        for (id,ally,p) in affected {
            match (spec.effect,ally) {
                (0|9,false) => {let focus=if spec.effect==9 {(radius as f64/(sqdist(p,plan.center) as f64).sqrt().max(10_000.0)).clamp(0.5,2.0)} else {1.0};
                    let raw=((35+attack*power/100) as f64*focus) as usize;
                    let damage=self.remote_damage(sim,entity,plan.host,p,raw);
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
        Self::fx(sim,entity,if spec.effect==9 {"seal"} else {"scar"},plan.center,20);
    }

    /// A sword lying on the ground with nothing to do (no pending draw, not armed, not drawing, no live plan).
    fn idle_grounded(&self, i: usize) -> bool {
        let s=&self.swords[i];
        matches!(s.mode,SwordMode::Planted|SwordMode::Stage) && s.path.is_empty() && !s.is_committed()
            && s.plan_id.is_none_or(|id|!self.formations.iter().any(|p|p.id==id))
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
                      ally_future: &[(usize,(i64,i64),(i64,i64),usize)], foes: &[(usize,(i64,i64),usize)], tick: usize) {
        let rank=self.rank();
        let my_pos=sim.get_entity(entity).map(|e|{let p=e.pos();(p.0 as i64,p.1 as i64)}).unwrap_or((0,0));
        let threat=|p:(i64,i64),f:(i64,i64)| foes.iter().filter(|(_,q,_)|near(*q,p,THREAT_R)||near(*q,f,THREAT_R)).count();
        let mut threatened:Vec<(usize,(i64,i64),usize,usize)>=ally_future.iter()
            .map(|&(id,p,f,missing)|(id,p,missing,threat(p,f)))
            .filter(|&(_,_,missing,n)| n>=1 && (missing>=25 || n>=2)).collect();
        threatened.sort_by_key(|&(id,_,missing,_)|(std::cmp::Reverse(missing),id));
        // leases
        for i in 0..7 {
            let Some(h)=self.swords[i].holder.filter(|&h|h!=entity) else {continue};
            if !matches!(self.swords[i].mode,SwordMode::Orbit|SwordMode::Stage) || tick<self.swords[i].escort_until {continue}
            if threatened.iter().any(|t|t.0==h) { self.swords[i].escort_until=tick+REASSESS[rank]; continue; }
            let close=ally_future.iter().find(|a|a.0==h).is_some_and(|a|near(a.1,my_pos,40_000));
            if close || tick<self.swords[i].locked_until {continue}
            self.send(sim,entity,i,SwordMode::Return,vec![my_pos],None);
            self.swords[i].holder=None;
        }
        for &(ally,p,missing,n) in &threatened {
            let have=(0..7).filter(|&i|self.swords[i].holder==Some(ally) &&
                matches!(self.swords[i].mode,SwordMode::Orbit|SwordMode::Stage|SwordMode::Strike|SwordMode::Return)).count();
            for _ in have..ESCORTS[rank] {
                let mut free=self.free_swords(entity,tick);
                let at_home=free.iter().filter(|&&i|self.swords[i].mode==SwordMode::Orbit).count();
                let give_last=rank>=5 && missing>=70;
                if at_home<=1 && !give_last {
                    free.retain(|&i|self.swords[i].mode!=SwordMode::Orbit);
                }
                let best=free.into_iter().map(|i|{
                    let from=self.position(sim,entity,i);
                    let eta=((sqdist(from,p) as f64).sqrt()/SPEED[i] as f64) as i64;
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
        for i in 0..7 {
            let s=&self.swords[i];
            if s.mode!=SwordMode::Orbit || s.holder.is_none() || sim.tick()<s.attack_at {continue}
            let Some(holder)=s.holder.and_then(|id|sim.get_entity(id)).filter(|e|e.is_alive()) else {continue};
            let hp=holder.pos(); let hp=(hp.0 as i64,hp.1 as i64);
            let target=(0..sim.entity_count()).filter_map(|n|sim.entity_at(n))
                .filter(|e|e.is_alive() && e.team()!=team && !e.is_tower() && sim.is_visible(team,e.id()))
                .filter(|e|{let p=e.pos();near(hp,(p.0 as i64,p.1 as i64),100_000)})
                .min_by_key(|e|{let p=e.pos();sqdist(hp,(p.0 as i64,p.1 as i64))})
                .map(|e|{let p=e.pos();(e.id(),(p.0 as i64,p.1 as i64))});
            if let Some((target,p))=target {
                self.send(sim,entity,i,SwordMode::Strike,vec![p],Some(target));
                self.swords[i].attack_at=sim.tick()+STRIKE_GAP[self.rank()];
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
            if self.swords[i].mode!=SwordMode::Orbit && sim.tick().is_multiple_of(3) {
                let tag=format!("aura_field_{i}_rank{rank}_frame{}",(sim.tick()/6)%8);
                Self::fx(sim,entity,&tag,p,4);
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
                let name=format!("il_aura_{i}_{side}");
                next.insert((id,i));
                if i==0 && !ally {sim.entity_set_invisible(id,0);}
                if self.aura_visual_at.get(&(id,i))==Some(&(ally,rank,count)) {continue}
                if let Some((old_ally,old_rank,_))=self.aura_visual_at.get(&(id,i)) {
                    sim.entity_remove_buff(id,&format!("il_aura_{i}_{}",if *old_ally {"ally"} else {"enemy"}));
                    sim.entity_remove_buff(id,&format!("il_aura_visual_{i}_rank{old_rank}_{}",
                        if *old_ally {"ally"} else {"enemy"}));
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
                sim.add_buff(id,&BuffV1::named(&format!("il_aura_visual_{i}_rank{rank}_{side}")));
                self.aura_visual_at.insert((id,i),(ally,rank,count));
            }
        }
        for &(id,i) in self.aura_active.difference(&next) {
            sim.entity_remove_buff(id,&format!("il_aura_{i}_ally"));
            sim.entity_remove_buff(id,&format!("il_aura_{i}_enemy"));
            if let Some((ally,old_rank,_))=self.aura_visual_at.remove(&(id,i)) {
                sim.entity_remove_buff(id,&format!("il_aura_visual_{i}_rank{old_rank}_{}",
                    if ally {"ally"} else {"enemy"}));
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
        let badge = if self.rank() == 7 { format!("il_imperial{}", self.imperial.unwrap_or(10)) }
            else { format!("il_rank{}", self.rank()) };
        if self.shown.as_deref() != Some(&badge) {
            if let Some(old) = &self.shown { sim.entity_remove_buff(entity, old); }
            sim.add_buff(entity, &BuffV1::named(&badge));
            self.shown = Some(badge);
        }
        let rank = self.rank();
        for i in 0..7 {
            let home = self.swords[i].mode == SwordMode::Orbit && self.swords[i].holder.is_none_or(|h| h == entity)
                && sim.tick() >= self.swords[i].vis_until;
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
            tick: usize, horizon: i64) -> (i64,i64) {
    let Some((prior,seen))=observed else { return current };
    let dt=tick.saturating_sub(seen).max(1) as i64;
    let vx=((current.0-prior.0)/dt).clamp(-2_000,2_000);
    let vy=((current.1-prior.1)/dt).clamp(-2_000,2_000);
    ((current.0+vx*horizon).clamp(0,1_000_000),
     (current.1+vy*horizon).clamp(0,1_000_000))
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
        self.engravings.clear(); self.formations.clear(); self.empowerment = 0;
        self.gathering = false; self.base_hit_ready = false;
        for (id,i) in self.aura_active.drain() {
            sim.entity_remove_buff(id,&format!("il_aura_{i}_ally"));
            sim.entity_remove_buff(id,&format!("il_aura_{i}_enemy"));
        }
        for ((id,i),(ally,rank,_)) in self.aura_visual_at.drain() {
            sim.entity_remove_buff(id,&format!("il_aura_visual_{i}_rank{rank}_{}",
                if ally {"ally"} else {"enemy"}));
        }
        for (id,(ally,rank)) in self.aura_base_shown.drain() {
            sim.entity_remove_buff(id,&format!("il_aura_base_rank{rank}_{}",
                if ally {"ally"} else {"enemy"}));
        }
        self.shown = None; self.arsenal_shown = [None; 7]; self.selected_shown = None;
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
            crate::fx_unit(sim, &format!("tfm2_isliid_emperor_{}_hit", SWORDS[sword]), entity, target, 0);
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
            let _ = NAMES[rank.min(7)];
            // Mastery changes forecast and decisions, not sword access or speed.
        }
        if tick >= 1800 {
            if let Some(a) = athlete_of(sim.seed(), player) {
                let id = sim.sim_origin().unwrap_or_default().match_id;
                record_game(sim.seed(), player, a, id);
                if tick % 30 == 0 { self.track_result(sim, player, entity); }
            }
        }
        for a in &mut self.anchors { if a.is_some_and(|a| a.until <= tick) { *a = None; } }
        let commands: Vec<String> = sim.get_entity(entity).map_or(Vec::new(), |e|
            (0..e.buff_count()).filter_map(|i| e.buff_at(i)).map(|b| b.name().to_string())
                .filter(|n| n.starts_with("il_draw_") || n.starts_with("il_recall_") || n == "il_manifest").collect());
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
        self.update_visuals(sim,entity);
        self.update_auras(sim,entity);
        self.render_marks(sim,entity);
        self.render_flags(sim,entity);
        self.show(sim, entity);
        self.note_wants(sim, player, entity);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn flight_segments_never_overshoot() {
        assert_eq!(segment_end((0, 0), (10_000, 0), 8_000, 3), (10_000, 0));
        assert_eq!(segment_end((0, 0), (100_000, 0), 8_000, 3), (24_000, 0));
        assert_eq!(segment_end((5, 5), (5, 5), 8_000, 3), (5, 5));
        let e = segment_end((0, 0), (30_000, 40_000), 5_000, 2);
        assert!((sqdist((0, 0), e) as f64).sqrt() <= 10_001.0);
    }

    #[test]
    fn every_visual_name_exists_in_the_data() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../mods/tfm2_custom/champion/tfm2_isliid_emperor.data_champion");
        let Ok(text) = std::fs::read_to_string(&root) else { return };   // the data isn't next to the source
        let has = |name: &str| text.contains(&format!("\"name\": \"{name}\""));
        let p = "tfm2_isliid_emperor_";
        for r in 0..8 {
            for (i, s) in SWORDS.iter().enumerate() {
                for k in 0..4 {
                    for st in ["flight", "drawing"] { assert!(has(&format!("{p}{s}_rank{r}_{st}_f{k}")), "{s} {r} {st} {k}"); }
                }
                for k in 0..8 {
                    for st in ["planted", "ready"] { assert!(has(&format!("{p}{s}_rank{r}_{st}_frame{k}"))); }
                    assert!(has(&format!("{p}aura_field_{i}_rank{r}_frame{k}")));
                }
                assert!(has(&arsenal_buff(i, r, false)) && has(&arsenal_buff(i, r, true)));
            }
        }
        for s in SWORDS {
            for fx in ["launch", "recall", "impact", "hit"] { assert!(has(&format!("{p}{s}_{fx}")), "{s}_{fx}"); }
        }
        for i in 0..7 { for a in 0..16 { assert!(has(&format!("{p}scar_{i}_a{a}"))); } }
        for r in 0..7 { assert!(has(&format!("il_rank{r}"))); }
        for ph in [FlagPhase::Planned, FlagPhase::Drawing, FlagPhase::Complete, FlagPhase::Cancelled] {
            for i in 0..PATTERNS.len() { assert!(has(&format!("{p}{}", logo_name(&format!("pattern_{i}"), ph).unwrap()))); }
            for k in 0..7 { assert!(has(&format!("{p}{}", logo_name(&format!("solo_{k}"), ph).unwrap()))); }
        }
        for n in 1..=10 { assert!(has(&format!("il_imperial{n}"))); }
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
    fn idle_swords_come_home_at_every_rank() {
        for r in 0..8 { assert!(IDLE_RETURN[r] <= 240 && IDLE_RETURN[r] > 0); if r>0 { assert!(IDLE_RETURN[r] <= IDLE_RETURN[r-1]); } }
        for r in 1..8 { assert!(REASSESS[r] <= REASSESS[r-1] && ESCORTS[r] >= ESCORTS[r-1] && STRIKE_GAP[r] <= STRIKE_GAP[r-1]); }
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
    fn remote_engraving_falloff_has_near_and_far_caps() {
        assert_eq!(damage_pct(0), 100);
        assert_eq!(damage_pct(100_000), 100);
        assert_eq!(damage_pct(200_000), 68);
        assert_eq!(damage_pct(300_000), 35);
        assert_eq!(damage_pct(900_000), 35);
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
        isliid.formations.push(FormationPlan { id:1, host:Some(0), pattern:0, center:(400_000,400_000),
            radius:35_000, legs:legs.clone(), until:500, completed:false });
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
                EngravingMark{sword,from,to,until:1800,id:sword as u64+1,host:Some(0)}).collect();
            let refs:Vec<_>=marks.iter().collect();
            assert!(match_live_drawing(p,&refs).is_some(),"{}",p.name);
        }
    }
}
