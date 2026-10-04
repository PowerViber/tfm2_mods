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

const CABLE_RANGE: i64 = 90_000;
const CABLE_MIN: i64 = 12_000;
const FLIGHT_T: usize = 90;
const TURN: f64 = 0.15;
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

    /// Where he wants to go now (an angle), and whether it's worth a flight from the ground.
    fn want(&self, m: &Champ, all: &[Champ], sim: &StableSim<'_>) -> Option<(f64, bool)> {
        let r = self.rank();
        let (x, y) = (m.x as f64, m.y as f64);
        let foes: Vec<&Champ> = all.iter().filter(|c| c.team != m.team && sim.is_visible(m.team, c.id)).collect();
        let near = |rr: i64| foes.iter().filter(|f| d2(f.x, f.y, m.x, m.y) <= sq(rr)).min_by_key(|f| d2(f.x, f.y, m.x, m.y)).copied();
        let low = m.hp * 100 <= m.max_hp * 35;
        // escape on a cable (Tethered and up)
        if r >= 1 && low {
            if let Some(f) = near(60_000) {
                let away = ang_to(f.x as f64, f.y as f64, x, y);
                let home = self.home.map_or(away, |h| ang_to(x, y, h.0 as f64, h.1 as f64));
                let a = (away.sin() + home.sin()).atan2(away.cos() + home.cos());
                return Some((a, true));
            }
        }
        // chase on a cable (Swinger and up): a weak enemy in reach, or the nearest one when his team is fighting
        if r >= 2 {
            let mates = all.iter().filter(|c| c.team == m.team && c.id != m.id && d2(c.x, c.y, m.x, m.y) <= sq(80_000)).count();
            let target = foes.iter().filter(|f| d2(f.x, f.y, m.x, m.y) <= sq(150_000) && f.hp * 100 <= f.max_hp * 45)
                .min_by_key(|f| f.hp).copied()
                .or_else(|| if mates > 0 { near(90_000) } else { None });
            if let Some(t) = target {
                return Some((ang_to(x, y, t.x as f64, t.y as f64), true));
            }
        }
        // otherwise where the game walks him: a steady walk is worth a flight
        let now = self.track.back()?;
        let then = self.track.iter().find(|p| p.0 + 30 >= now.0)?;
        let (dx, dy) = ((now.1 - then.1) as f64, (now.2 - then.2) as f64);
        let walked = dx.hypot(dy);
        if walked < 4_000.0 { return None; }
        Some((dy.atan2(dx), walked >= 20_000.0))
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
            let mut score = -(k.abs() as f64) * 10.0;
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
            if best.map_or(true, |b| score > b.0) { best = Some((score, a, p)); }
        }
        best.map(|b| (b.1, b.2))
    }

    /// One S1 press: fire a cable or not.
    fn on_cable_press(&mut self, sim: &mut StableSim<'_>, m: &Champ, all: &[Champ], tick: usize) {
        if tick < self.recover_until || self.dash.is_some() { return; }
        let r = self.rank();
        let fresh = self.want(m, all, sim);
        if self.flying {
            let tta = self.cables.last().map_or(0.0, |c| {
                ((c.0 as f64 - self.pos.0).hypot(c.1 as f64 - self.pos.1)) / self.speed.max(1.0)
            });
            let fire = if r == 0 { tick >= self.last_cable + 18 } else { self.gliding || tta <= LOOKAHEAD[r] || tick >= self.last_cable + 40 };
            if !fire { return; }
            if let Some((a, _)) = fresh { if r >= 2 { self.goal = Some(a); } }
        } else {
            let Some((a, go)) = fresh else { return };
            if !go { return; }
            self.goal = Some(a);
            self.pos = (m.x as f64, m.y as f64);
        }
        let Some(want) = self.goal else { return };
        let towers: Vec<(i64, i64)> = (0..sim.tower_count()).filter_map(|i| sim.get_entity(sim.tower_id_at(i)))
            .filter(|e| e.is_alive()).map(|e| { let (x, y) = e.pos(); (x as i64, y as i64) }).collect();
        let picked = self.pick_anchor(want, &towers);
        let shot = match picked {
            Some((a, p)) if self.roll(MISAIM[r]) => {
                // a misaimed cable veers 10-40 degrees and bites whatever is there, or nothing
                let off = deg(10.0 + 30.0 * self.unit()) * if self.roll(50) { 1.0 } else { -1.0 };
                let _ = p;
                raycast(self.pos.0, self.pos.1, a + off, &towers).map(|q| (a + off, q)).ok_or(a + off)
            }
            Some(x) => Ok(x),
            None => Err(want),
        };
        match shot {
            Ok((_, p)) => self.connect(sim, m, p, tick),
            Err(a) => self.whiff(sim, m, a, tick),
        }
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
        }
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
        let r = self.rank();
        if !self.gliding && tick >= self.flight_until {
            self.gliding = true;
            self.cables.clear();
        }
        if self.gliding {
            self.speed *= 0.9;
            if self.speed < 1_400.0 { self.end_flight(); return; }
        } else if let Some(&b) = self.cables.last() {
            let (x, y) = self.pos;
            let tb = ang_to(x, y, b.0 as f64, b.1 as f64);
            let target = if self.cables.len() == 2 {
                let ta = ang_to(x, y, self.cables[0].0 as f64, self.cables[0].1 as f64);
                (0.7 * tb.sin() + 0.3 * ta.sin()).atan2(0.7 * tb.cos() + 0.3 * ta.cos())
            } else { tb };
            let d = wrap(target - self.heading);
            self.heading = wrap(self.heading + d.clamp(-TURN, TURN));
            // Stormcutter and up brake when a slam is coming and no cable is ready
            let tta = ((b.0 as f64 - x).hypot(b.1 as f64 - y)) / self.speed.max(1.0);
            if r >= 5 && tta < 6.0 && self.speed >= CRASH_SPEED && tick < self.recover_until.max(self.last_cable + 12) {
                self.speed = (self.speed * 0.85).max(CRASH_SPEED - 100.0);
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
                (_, Some((_, true))) => true,
                _ => false,
            };
            if !go { return; }
            let Some((a, _)) = want else { return };
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

    fn set_form(&mut self, sim: &mut StableSim<'_>, m: &Champ, form: usize) {
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
        self.set_form(sim, m, if tier >= 2 { tier } else { 0 });
        if !self.flying { self.last_trail = None; }
        if self.flying && tick % 2 == 0 {
            let (x, y) = self.pos;
            for &(ax, ay) in &self.cables {
                let (mx, my) = ((x as i64 + ax) / 2, (y as i64 + ay) / 2);
                let a = ang_to(x, y, ax as f64, ay as f64).to_degrees().rem_euclid(180.0);
                let d = ((a / 11.25).round() as usize) % 16;
                let len_px = ((ax as f64 - x).hypot(ay as f64 - y)) / 950.0;
                let b = ((len_px / 16.0).round() as usize).clamp(1, 6);
                crate::fx_point(sim, &self.fx(m, &format!("cable_{d}_{b}")), m.id, mx.max(0), my.max(0), 2);
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
        if let (true, Some(me)) = (self.form >= 2, self.me) {
            for k in 2..=4 { sim.entity_remove_buff(me, &format!("lv_form{k}")); }
        }
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
