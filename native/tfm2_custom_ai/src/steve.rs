//! Steve (tfm2_blockcraft): a blocky tank support.
//!
//!   S1  three tools, picked by the situation (see choose_tool):
//!       pearl  thrown, he teleports where it lands; a tethered enemy is dragged along to the landing spot
//!       TNT    thrown at the target; a red ring for the fuse, then it blows up (damage + knockback)
//!       apple  heal + shield on the hooked ally (rod), otherwise the ally lowest on HP (him included)
//!   S2  fishing rod (70000, charged up to 154000; a big slow hook, a fast reel): the hook flies through walls (terrain and boat walls) and
//!       catches the first thing it reaches; whatever it reels in comes through the walls too:
//!       an enemy  → pulled to him (tether; a pearl during the tether drags them further)
//!       an ally   → yanked to him (rescue), and his next apple goes to them
//!       a wall    → he's pulled to it (grapple)
//!       his TNT   → yanked and flung at the nearest enemy, blowing up as it lands
//!   Ult boat: he rides a boat along an ice trail; a dark stone wall two blocks high rises behind the boat and blocks
//!       every unit for WALL_LIFE (the match hook pushes back anyone who would cross; the input AI walks around it).
//!       The wall boxes in a fight or an objective (three sides, open toward his team; see plan_ult), or, when his
//!       side is losing, splits the teams. He jumps off midway (into the box, or to a teammate in trouble) and the
//!       empty boat finishes the wall. The cast waits for a good setup: the data's ult targets an enemy in CC, and
//!       while the ult is ready and plan_ult finds a setup, a 2-tick bind on the key enemy lets the AI press it.
//!
//! The data file only sets markers (stv_s1, stv_rod, stv_boat + target marks); everything else is here.

use super::*;

const STEVE_ID: &str = "tfm2_blockcraft_steve";

// ---- S1
const PEARL_RANGE: i64 = 70_000;
const PEARL_T: usize = 18;
const PEARL_ESCAPE: i64 = 60_000;
const TNT_FLIGHT: usize = 15;
const TNT_FUSE: usize = 30;                      // round 32: lit as it lands, 0.6 s fuse (it used to lie unlit until
                                                 // someone swung next to it, then 1 s)
const TNT_QUICK: usize = 6;                      // a swing right next to a lit TNT sets it off at once
const TNT_LIFE: usize = 900;
const TNT_TOUCH_R: i64 = 22_000;                 // a basic attack started this close lights it
const TNT_R: i64 = 32_000;                       // round 36: was 26000
const TNT_MINE_LIFE: usize = 300;                // landed with nobody in reach: a mine for up to 5 s
const TNT_TRIGGER_R: i64 = 20_000;               // an enemy stepping this close sets the mine off
const TNT_MINE_FUSE: usize = 12;
const TNT_DMG: (usize, usize) = (40, 6);         // base + % of Steve's max HP
const TNT_PUSH: (u64, u64) = (3_000, 12);        // knockback speed, ticks
const TNT_OFFSET: i64 = 8_000;                   // lands this far behind (or in front of) where the target will be (was 12000)
const TNT_SIDE_R: i64 = 40_000;                  // who's around him, to read the fight
const TNT_PEEL_PCT: usize = 50;
const APPLE_R: i64 = 60_000;
const APPLE_HEAL: (usize, usize) = (60, 8);      // base + % of Steve's max HP
const APPLE_SHIELD_PCT: usize = 10;
const APPLE_SHIELD_TICKS: usize = 240;
// ---- S2
const HOOK_SPEED: i64 = 4_200;                   // a big, slow hook
const HOOK_RANGE: i64 = 154_000;                 // fully charged
const HOOK_MIN: i64 = 70_000;                    // released at once
const CHARGE_MAX: usize = 60;                    // 1 s to full charge
const CHARGE_SLOW: i32 = -40;                    // he walks slowly while charging
const CHARGE_MARGIN: i64 = 8_000;                // charge a little past the target (it moves)
const HOOK_HIT_R: i64 = 16_000;
const HOOK_DMG: (usize, usize) = (20, 2);
const PULL: (usize, usize) = (4_200, 40);        // enemy tether: speed, max ticks (reeled in to ~10000 from him)
const RESCUE: (usize, usize) = (5_000, 35);      // ally yank
const GRAPPLE_SPEED: i64 = 3_500;
const TNT_FLING_T: usize = 14;
const LOW_PCT: usize = 35;
// ---- ult
const RIDE_SPEED: i64 = 2_500;                   // round 26: slower along the wall (was 6000)
const WALL_LIFE: usize = 600;                    // 10 s after it's built
const HOP_MAX: i64 = 60_000;                     // he hops to the path's start if it's this close, else boats there first
const ULT_REACH: i64 = 340_000;                  // what he walls in is at most this far from him
const ULT_RANGE: i64 = 480_000;                  // the data's ult range (the key enemy must be inside it)
const WALL_LEN: i64 = 300_000;                   // one straight wall, at most this long (terrain cuts it short;
                                                 // round 30: it no longer runs over terrain)
const BOAT_HIT_R: i64 = 16_000;                  // the boat rams enemies this close
const BOAT_KNOCK: (u64, u64) = (3_500, 12);      // knockback speed, ticks (42000: clear of the wall)
const BOAT_DMG: (usize, usize) = (30, 4);        // base + % of Steve's max HP
const MAP: i64 = 960_000;
const JUMP_T: usize = 14;                        // jumping off the boat
const JUMP_MAX: i64 = 60_000;
const ULT_BIND_EVERY: usize = 20;
const ICE_MS: i32 = 25;
pub const WALL_HALF: i64 = 8_000;                // half thickness of the wall (round 26: blocks twice the size)
pub const BLINK_PASS: i64 = 15_000;              // per-tick jumps this big are teleports / blinks: they cross it.
                                                 // Round 28: was 6000, which let fast dashes and big knockbacks through

#[derive(Clone, Copy, Debug)]
enum HookMode { Enemy, Ally(usize), Wall, Tnt(usize) }

#[derive(Clone, Debug)]
struct Hook { x: i64, y: i64, dx: f64, dy: f64, travelled: i64, range: i64, mode: HookMode }

/// Charging the rod: what it's aimed at (a unit to follow, or a point) and since when.
#[derive(Clone, Debug)]
struct Charge { mode: HookMode, aim_id: Option<usize>, pt: (i64, i64), t0: usize }

#[derive(Clone, Debug)]
/// A placed TNT. It lies unlit (det = usize::MAX) until Steve or a teammate hits it with a basic attack (one started
/// right next to it) or his rod yanks it (then it's thrown like a bomb and blows up as it lands); unlit for
/// TNT_LIFE it fizzles out.
struct Tnt { x: i64, y: i64, land: usize, det: usize, expire: usize, from: (i64, i64), checked: bool }

/// Where the boat goes: the wall's two ends, the way it blocks (axis: from his team's side to the far side), where
/// Steve jumps off and the enemy to hold so the AI casts it.
#[derive(Clone, Debug)]
struct UltPlan { pts: Vec<(i64, i64)>, axis: (f64, f64), jump_to: Option<(i64, i64)>, key: Option<usize>, score: i64,
    rally: Option<((i64, i64), Option<usize>)> }   // where his team should go to use the wall (a spot, or an enemy to follow)

#[derive(Clone, Debug)]
struct Ride { pts: Vec<(i64, i64)>, cum: Vec<f64>, t0: usize, rider: bool, jump_at: Option<f64>, jump_to: Option<(i64, i64)>, home: (f64, f64), hit: Vec<usize> }

#[derive(Clone, Default)]
pub struct Steve {
    tool: usize,
    tnts: Vec<Tnt>,
    pearl: Option<(i64, i64, i64, i64, usize)>,     // from x, y, to x, y, t0
    hook: Option<Hook>,
    charge: Option<Charge>,
    tether: Option<(usize, usize)>,                 // enemy, until
    grapple: Option<(i64, i64, usize)>,             // point, until
    ally_hooked: Option<(usize, usize)>,            // ally, tick
    ride: Option<Ride>,
    jump: Option<(i64, i64, i64, i64, usize)>,      // jumping off the boat: from x, y, to x, y, t0
    reels: Vec<(usize, usize, usize)>,              // reeled in through terrain by hand: unit, speed, until
    shown_tool: Option<usize>,
    atk_cd: Vec<(usize, usize)>,                    // teammates' attack cooldowns last tick (to see a swing start)
    my_swing: usize,                                // tick of his own last basic attack
    seen: Vec<(usize, i64, i64)>,                   // enemies' positions last tick (to lead throws)
}

fn v(m: &Champ, n: &str) -> String {
    let id = if m.name.starts_with("tfm2_") { m.name.as_str() } else { STEVE_ID };
    format!("{id}_{n}")
}

fn fx(sim: &mut StableSim<'_>, name: &str, caster: usize, x: i64, y: i64, t: u64) {
    sim.play_view_effect(name, caster, &InputTargetV1::pos(x.max(0) as u64, y.max(0) as u64), 0, 0, t);
}

fn norm(dx: f64, dy: f64) -> (f64, f64) {
    let l = dx.hypot(dy);
    if l < 1e-9 { (1.0, 0.0) } else { (dx / l, dy / l) }
}

fn pct(c: &Champ) -> usize {
    if c.max_hp == 0 { 100 } else { c.hp * 100 / c.max_hp }
}

/// A purely visual flight (noop effect) from a to b.
fn fly(sim: &mut StableSim<'_>, name: &str, m: &Champ, ax: i64, ay: i64, bx: i64, by: i64, ticks: usize) {
    let dist = ((bx - ax) as f64).hypot((by - ay) as f64);
    let spec = ProjectileSpawnV1 {
        caster_id: m.id, team: m.team, x: ax.max(0) as u64, y: ay.max(0) as u64, radius: 1_000,
        speed: ((dist / ticks.max(1) as f64) as u64).max(1), move_kind: ProjectileMoveKindV1::Linear.code(),
        target_x: bx.max(0) as u64, target_y: by.max(0) as u64, penetrate: true, casting_target: CastingTargetV1::None.code(),
        ..ProjectileSpawnV1::default()
    };
    sim.spawn_projectile(name, &format!("{MOD_ID}:noop"), &spec);
}

/// Where his team is: the average of his living teammates (else his own towers, else himself).
fn home(sim: &StableSim<'_>, m: &Champ, all: &[Champ]) -> (i64, i64) {
    let mates: Vec<&Champ> = all.iter().filter(|c| c.team == m.team && c.id != m.id).collect();
    if !mates.is_empty() {
        let n = mates.len() as i64;
        return (mates.iter().map(|c| c.x).sum::<i64>() / n, mates.iter().map(|c| c.y).sum::<i64>() / n);
    }
    team_towers(sim, m.team).unwrap_or((m.x, m.y))
}

fn team_towers(sim: &StableSim<'_>, team: usize) -> Option<(i64, i64)> {
    let pts: Vec<(i64, i64)> = (0..sim.tower_count()).filter_map(|i| sim.get_entity(sim.tower_id_at(i)))
        .filter(|t| t.is_alive() && t.team() == team).map(|t| { let (x, y) = t.pos(); (x as i64, y as i64) }).collect();
    if pts.is_empty() { return None; }
    let n = pts.len() as i64;
    Some((pts.iter().map(|p| p.0).sum::<i64>() / n, pts.iter().map(|p| p.1).sum::<i64>() / n))
}

// ------------------------------------------------------------------ walls (shared with the match hook and the AI)

/// An active boat wall: built from a toward b between t0 and tb; it stands until tend.
#[derive(Clone, Copy, Debug)]
pub struct Wall { pub ax: i64, pub ay: i64, pub bx: i64, pub by: i64, pub t0: usize, pub tb: usize, pub tend: usize }

impl Wall {
    /// The built end right now (the wall grows behind the boat).
    pub fn built_end(&self, tick: usize) -> (i64, i64) {
        if tick >= self.tb || self.tb <= self.t0 { return (self.bx, self.by); }
        let k = (tick - self.t0) as f64 / (self.tb - self.t0) as f64;
        (self.ax + ((self.bx - self.ax) as f64 * k) as i64, self.ay + ((self.by - self.ay) as f64 * k) as i64)
    }
}

/// Every wall that is up, from the "sbw:ax:ay:bx:by:t0:tb:tend" buffs of any Steve.
pub fn walls_up(all: &[Champ], tick: usize) -> Vec<Wall> {
    let mut out = Vec::new();
    for c in all {
        for b in &c.buffs {
            if let Some(rest) = b.name().strip_prefix("sbw:") {
                let v: Vec<i64> = rest.split(':').filter_map(|s| s.parse().ok()).collect();
                if v.len() >= 7 {
                    let w = Wall { ax: v[0], ay: v[1], bx: v[2], by: v[3], t0: v[4].max(0) as usize, tb: v[5].max(0) as usize, tend: v[6].max(0) as usize };
                    if tick >= w.t0 && tick < w.tend { out.push(w); }
                }
            }
        }
    }
    out
}

/// Does the segment p→q cross the built part of the wall?
/// (The built part, plus WALL_HALF past each end: the end blocks are drawn that far out, and they block too.)
pub fn crosses(w: &Wall, tick: usize, px: i64, py: i64, qx: i64, qy: i64) -> bool {
    let (ex, ey) = w.built_end(tick);
    let (ux, uy) = norm((ex - w.ax) as f64, (ey - w.ay) as f64);
    let h = WALL_HALF as f64;
    let a = ((w.ax as f64 - ux * h) as i64, (w.ay as f64 - uy * h) as i64);
    let b = ((ex as f64 + ux * h) as i64, (ey as f64 + uy * h) as i64);
    seg_cross((px, py), (qx, qy), a, b)
}

fn seg_cross(p: (i64, i64), q: (i64, i64), a: (i64, i64), b: (i64, i64)) -> bool {
    let o = |a: (i64, i64), b: (i64, i64), c: (i64, i64)| -> i128 {
        let v = (b.0 - a.0) as i128 * (c.1 - a.1) as i128 - (b.1 - a.1) as i128 * (c.0 - a.0) as i128;
        v.signum()
    };
    let (d1, d2, d3, d4) = (o(a, b, p), o(a, b, q), o(p, q, a), o(p, q, b));
    d1 != 0 && d2 != 0 && d1 != d2 && d3 != d4
}

/// The way around a wall from (x, y) to (tx, ty): a point just past its nearer useful end.
pub fn detour(w: &Wall, tick: usize, x: i64, y: i64, tx: i64, ty: i64) -> (i64, i64) {
    let (ex, ey) = w.built_end(tick);
    let (ux, uy) = norm((ex - w.ax) as f64, (ey - w.ay) as f64);
    let ends = [((w.ax as f64 - ux * 16_000.0) as i64, (w.ay as f64 - uy * 16_000.0) as i64),
                ((ex as f64 + ux * 16_000.0) as i64, (ey as f64 + uy * 16_000.0) as i64)];
    let cost = |p: &(i64, i64)| ((p.0 - x) as f64).hypot((p.1 - y) as f64) + ((tx - p.0) as f64).hypot((ty - p.1) as f64);
    let best = if cost(&ends[0]) <= cost(&ends[1]) { ends[0] } else { ends[1] };
    (best.0.max(0), best.1.max(0))
}

/// Signed distance from the wall's line (+ one side, − the other) and the position along it.
fn side(w: &Wall, x: i64, y: i64) -> (f64, f64) {
    let (ux, uy) = norm((w.bx - w.ax) as f64, (w.by - w.ay) as f64);
    let (rx, ry) = ((x - w.ax) as f64, (y - w.ay) as f64);
    (rx * -uy + ry * ux, rx * ux + ry * uy)
}

use std::collections::HashMap;
use std::sync::Mutex;
/// Last tick's positions, per simulation (crate::match_key: the game runs several simulations in parallel).
static LAST: Mutex<Option<HashMap<u64, (usize, HashMap<usize, (i64, i64)>)>>> = Mutex::new(None);

/// Match hook: nobody walks, dashes, teleports or gets knocked through a standing boat wall. Anyone who would end up
/// on the other side (or inside it) is put back where they were last tick; someone the wall rises under is pushed
/// out to the side they were on. Riders on the boat itself and anyone on his fishing line are left alone.
pub fn enforce_walls(sim: &mut StableSim<'_>, all: &[Champ], tick: usize) {
    let walls = walls_up(all, tick);
    let key = crate::match_key(sim);
    let mut guard = match LAST.lock() { Ok(g) => g, Err(_) => return };
    let maps = guard.get_or_insert_with(HashMap::new);
    if walls.is_empty() {
        // nothing standing: forget this match's positions (they're only needed while a wall is up)
        maps.remove(&key);
        return;
    }
    let entry = maps.entry(key).or_insert_with(|| (tick, HashMap::new()));
    if tick < entry.0 || tick > entry.0 + 5 {
        *entry = (tick, HashMap::new());
    }
    let last = &mut entry.1;
    let mut now: Vec<(usize, i64, i64, bool)> = Vec::new();
    for i in 0..sim.entity_count() {
        if let Some(e) = sim.entity_at(i) {
            if !e.is_alive() { continue; }
            let (x, y) = e.pos();
            let riding = all.iter().any(|c| c.id == e.id() && (c.has("stv_riding") || c.has("stv_reeled")));
            now.push((e.id(), x as i64, y as i64, riding));
        }
    }
    {
        for &(id, x, y, riding) in &now {
            if riding { continue; }
            let Some(&(px, py)) = last.get(&id) else { continue };
            // a teleport or blink (a jump of BLINK_PASS+ in one tick) goes over the wall, like a blink over terrain;
            // walking, dashes, knockbacks and pulls (all well under it per tick) don't
            if d2(px, py, x, y) >= sq(BLINK_PASS) { continue; }
            let mut fixed: Option<(i64, i64)> = None;
            for w in &walls {
                let (ex, ey) = w.built_end(tick);
                let len = ((ex - w.ax) as f64).hypot((ey - w.ay) as f64);
                let (sd, along) = side(w, x, y);
                let (psd, _) = side(w, px, py);
                // the whole drawn wall blocks: the end blocks reach WALL_HALF past each end (round 28: was 3000)
                let within = along >= -(WALL_HALF as f64) && along <= len + WALL_HALF as f64;
                if !within { continue; }
                let inside = sd.abs() < WALL_HALF as f64;
                let flipped = sd.signum() != psd.signum() && psd.abs() >= WALL_HALF as f64 * 0.5;
                if crosses(w, tick, px, py, x, y) || flipped || inside {
                    if psd.abs() >= WALL_HALF as f64 {
                        fixed = Some((px, py));
                    } else {
                        // it rose under them: out to the side they were on (or are on)
                        let s = if psd != 0.0 { psd.signum() } else if sd != 0.0 { sd.signum() } else { 1.0 };
                        let (ux, uy) = norm((w.bx - w.ax) as f64, (w.by - w.ay) as f64);
                        let push = WALL_HALF as f64 + 3_000.0 - sd * s;
                        fixed = Some(((x as f64 + -uy * s * push) as i64, (y as f64 + ux * s * push) as i64));
                    }
                    break;
                }
            }
            if let Some((fx_, fy_)) = fixed {
                sim.entity_set_pos(id, fx_.max(0) as u64, fy_.max(0) as u64);
                if all.iter().any(|c| c.id == id) {
                    sim.entity_remove_buff(id, "swb_blk");
                    sim.add_buff(id, &timed("swb_blk", 30));
                }
            }
        }
    }
    // remember where everyone is (after any fix)
    last.clear();
    for i in 0..sim.entity_count() {
        if let Some(e) = sim.entity_at(i) {
            if e.is_alive() {
                let (x, y) = e.pos();
                last.insert(e.id(), (x as i64, y as i64));
            }
        }
    }
    entry.0 = tick;
}

// ------------------------------------------------------------------ fog of war: a boat wall blocks sight

const SIGHT_R: i64 = 110_000;

/// Match hook: a standing boat wall blocks sight like terrain. A champion whose every enemy nearby (champions,
/// towers and minions of the other team within SIGHT_R) is on the other side of a wall is hidden from that team
/// (the game's invisible flag, refreshed every 2 ticks while it holds). With nobody of theirs nearby, the game's own
/// fog decides. Steve riding the boat stays in sight.
pub fn wall_fog(sim: &mut StableSim<'_>, all: &[Champ], tick: usize) {
    if tick % 2 != 0 { return; }
    let walls = walls_up(all, tick);
    if walls.is_empty() { return; }
    // while one of Omen's smokes is up, valorant::fog handles the walls too
    if !crate::valorant::smokes_up(all, tick).is_empty() { return; }
    let teams: Vec<usize> = all.iter().map(|c| c.team).collect();
    let mut eyes: Vec<(usize, i64, i64)> = Vec::new();
    for i in 0..sim.entity_count() {
        if let Some(e) = sim.entity_at(i) {
            if e.is_alive() && teams.contains(&e.team()) {
                let (x, y) = e.pos();
                eyes.push((e.team(), x as i64, y as i64));
            }
        }
    }
    for c in all.iter().filter(|c| !c.has("stv_riding")) {
        let mut seen_by_any = false;
        let mut watchers = 0;
        for &(team, x, y) in eyes.iter().filter(|w| w.0 != c.team && d2(w.1, w.2, c.x, c.y) <= sq(SIGHT_R)) {
            let _ = team;
            watchers += 1;
            if !walls.iter().any(|w| crosses(w, tick, x, y, c.x, c.y)) { seen_by_any = true; break; }
        }
        if watchers > 0 && !seen_by_any {
            sim.entity_set_invisible(c.id, 3);
        }
    }
}

// ------------------------------------------------------------------ map awareness: something worth crossing for

/// Map awareness for champions who can get over a boat wall (teleports, dashes): the most urgent thing on the
/// other side of a standing wall, within `reach` of `me`:
///   1. a teammate in trouble (at 55% HP or less with an enemy champion on them, or outnumbered where they stand);
///   2. an allied tower with enemy champions at it;
///   3. a big neutral objective (a camp monster with 2500+ max HP) that enemy champions are taking.
/// Returns where it is, or None (also when no wall stands).
pub fn wall_need(sim: &StableSim<'_>, all: &[Champ], me: &Champ, tick: usize, reach: i64) -> Option<(i64, i64)> {
    let walls = walls_up(all, tick);
    if walls.is_empty() { return None; }
    let behind = |x: i64, y: i64| walls.iter().any(|w| crosses(w, tick, me.x, me.y, x, y));
    let in_reach = |x: i64, y: i64| d2(x, y, me.x, me.y) <= sq(reach);
    // fog of war: only enemies his team can see
    let count = |x: i64, y: i64, r: i64, ally: bool| all.iter().filter(|c| (c.team == me.team) == ally && (ally || sim.is_visible(me.team, c.id))
        && d2(c.x, c.y, x, y) <= sq(r)).count();
    // 1. teammates in trouble
    let mut best: Option<(usize, (i64, i64))> = None;
    for a in all.iter().filter(|c| c.team == me.team && c.id != me.id && in_reach(c.x, c.y) && behind(c.x, c.y)) {
        let foes = count(a.x, a.y, 35_000, false);
        let friends = count(a.x, a.y, 35_000, true);
        let low = pct(a) <= 55 && count(a.x, a.y, 30_000, false) > 0;
        if low || (foes > 0 && foes > friends) {
            let urgency = pct(a);
            if best.map_or(true, |(u, _)| urgency < u) { best = Some((urgency, (a.x, a.y))); }
        }
    }
    if let Some((_, p)) = best { return Some(p); }
    // 2. allied towers under attack, 3. big neutral objectives being taken
    let teams: Vec<usize> = all.iter().map(|c| c.team).collect();
    for i in 0..sim.entity_count() {
        let Some(e) = sim.entity_at(i) else { continue };
        // (round 40: no is_minion filter: neutral camp monsters may report as minions; lane minions belong to a
        // champion team and are excluded below anyway)
        if !e.is_alive() || e.is_champion() || (e.is_minion() && teams.contains(&e.team())) { continue; }
        let (x, y) = { let (x, y) = e.pos(); (x as i64, y as i64) };
        if !in_reach(x, y) || !behind(x, y) { continue; }
        let tower_hit = e.is_tower() && e.team() == me.team && count(x, y, 30_000, false) > 0;
        let objective = !e.is_tower() && !teams.contains(&e.team()) && e.hp().1 >= 2_500
            && count(x, y, 30_000, false) > count(x, y, 30_000, true);
        if tower_hit || objective { return Some((x, y)); }
    }
    None
}

// ------------------------------------------------------------------ the ult plan: what the boat walls in

fn cum_len(pts: &[(i64, i64)]) -> Vec<f64> {
    let mut out = vec![0.0];
    for w in pts.windows(2) {
        let l = ((w[1].0 - w[0].0) as f64).hypot((w[1].1 - w[0].1) as f64);
        out.push(out.last().copied().unwrap_or(0.0) + l);
    }
    out
}

/// The point `d` along the path, and the index of the segment it's on.
fn path_at(pts: &[(i64, i64)], cum: &[f64], d: f64) -> ((i64, i64), usize) {
    if pts.len() < 2 { return (pts.first().copied().unwrap_or((0, 0)), 0); }
    for i in 0..pts.len() - 1 {
        if d <= cum[i + 1] || i == pts.len() - 2 {
            let l = (cum[i + 1] - cum[i]).max(1.0);
            let k = ((d - cum[i]) / l).clamp(0.0, 1.0);
            let (a, b) = (pts[i], pts[i + 1]);
            return ((a.0 + ((b.0 - a.0) as f64 * k) as i64, a.1 + ((b.1 - a.1) as f64 * k) as i64), i);
        }
    }
    (*pts.last().unwrap(), pts.len() - 2)
}

fn centroid(pts: &[(i64, i64)]) -> (i64, i64) {
    let n = pts.len().max(1) as i64;
    (pts.iter().map(|p| p.0).sum::<i64>() / n, pts.iter().map(|p| p.1).sum::<i64>() / n)
}

/// Is this enemy caught by real crowd control (a stun, root, knock-up, taunt, fear or charm of CAUGHT_MIN+ ticks)?
/// Steve's own 2-tick hold for the AI doesn't count.
const CAUGHT_MIN: u64 = 8;
fn caught(sim: &StableSim<'_>, id: usize) -> bool {
    let Some(e) = sim.get_entity(id) else { return false };
    (0..e.cc_count()).filter_map(|i| e.cc_at(i)).any(|cc| cc.tick >= CAUGHT_MIN && [CcKindV1::Airborne, CcKindV1::Stun, CcKindV1::Bind,
        CcKindV1::Taunt, CcKindV1::Fear, CcKindV1::Charm].iter().any(|k| k.code() == cc.kind))
}

/// The middle of a team's towers; with none left, mirrored from the other side's (the map is 960000 square).
fn base_of(sim: &StableSim<'_>, team: usize, other: usize, fallback: (i64, i64)) -> (i64, i64) {
    team_towers(sim, team).or_else(|| team_towers(sim, other).map(|(x, y)| (MAP - x, MAP - y))).unwrap_or(fallback)
}

/// A straight wall through c, square to `axis` (the way it blocks), always at its full length (WALL_LEN) when there's
/// room (round 41): where terrain or the map edge cuts one side short, the wall slides along its line into the other
/// side, staying as centred on c as it can. Only a passage narrower than WALL_LEN gets a shorter wall, wall to wall.
/// The two ends (the one nearer Steve first) and how many of them rest on terrain (2 = the passage is sealed).
fn lay_line(m: &Champ, anchor: (i64, i64), c: (i64, i64), axis: (f64, f64), min_len: f64) -> Option<((i64, i64), (i64, i64), usize)> {
    let c = walls::pull_back(anchor.0, anchor.1, c.0.clamp(0, MAP), c.1.clamp(0, MAP));
    if walls::wall_at(c.0, c.1) { return None; }
    let (px, py) = (-axis.1, axis.0);
    let full = WALL_LEN as f64;
    // how far the line runs free each way (up to a whole wall length)
    let free = |s: f64| {
        let t = ((c.0 as f64 + px * full * s) as i64, (c.1 as f64 + py * full * s) as i64);
        let q = walls::clip(c.0, c.1, t.0, t.1);
        ((q.0 - c.0) as f64).hypot((q.1 - c.1) as f64)
    };
    let (fa, fb) = (free(1.0), free(-1.0));
    let total = full.min(fa + fb);
    let ka = fa.min((full / 2.0).max(total - fb));
    let kb = (total - ka).min(fb);
    let a = ((c.0 as f64 + px * ka) as i64, (c.1 as f64 + py * ka) as i64);
    let b = ((c.0 as f64 - px * kb) as i64, (c.1 as f64 - py * kb) as i64);
    let sealed = (ka >= fa - 3_000.0 && fa < full - 3_000.0) as usize + (kb >= fb - 3_000.0 && fb < full - 3_000.0) as usize;
    let len = ka + kb;
    if len < min_len && !(sealed == 2 && len >= 25_000.0) { return None; }
    Some(if d2(b.0, b.1, m.x, m.y) < d2(a.0, a.1, m.x, m.y) { (b, a, sealed) } else { (a, b, sealed) })
}

// ------------------------------------------------------------------ the rally: Steve calls his team in

/// Teammates this far from the rally spot are called in (their moves only; casts and attacks are never touched).
const RALLY_R: i64 = 220_000;
/// The enemies that count at the spot.
const RALLY_FIGHT_R: i64 = 70_000;

/// Is going in a winning situation? Everyone on his team who can come (within RALLY_R, above 30% HP) at least matches
/// the enemies his team can see at the spot that no standing wall cuts off from it, and there are two or more of them.
fn rally_good(sim: &StableSim<'_>, team: usize, all: &[Champ], p: (i64, i64), tick: usize) -> bool {
    let walls_ = walls_up(all, tick);
    let ours = all.iter().filter(|c| c.team == team && pct(c) > 30 && d2(c.x, c.y, p.0, p.1) <= sq(RALLY_R)).count();
    let theirs = all.iter().filter(|c| c.team != team && sim.is_visible(team, c.id) && d2(c.x, c.y, p.0, p.1) <= sq(RALLY_FIGHT_R)
        && !walls_.iter().any(|w| crosses(w, tick, c.x, c.y, p.0, p.1))).count();
    ours >= 2 && ours >= theirs
}

/// The rally Steve has called (from the "stv_rally:<x>:<y>:<enemy id or -1>:<until>" buff on any Steve of `team`):
/// where to go right now (the enemy to follow if it's still alive and seen, else the spot) and until when.
fn rally_of(sim: &StableSim<'_>, all: &[Champ], team: usize, tick: usize) -> Option<(i64, i64)> {
    for c in all.iter().filter(|c| c.team == team) {
        for b in &c.buffs {
            let n = b.name();
            let Some(r) = n.strip_prefix("stv_rally:") else { continue };
            let v: Vec<i64> = r.split(':').filter_map(|t| t.parse().ok()).collect();
            if v.len() < 4 || tick as i64 >= v[3] { continue; }
            let follow = (v[2] >= 0).then(|| all.iter().find(|e| e.id == v[2] as usize && e.team != team && sim.is_visible(team, e.id))).flatten();
            return Some(follow.map_or((v[0], v[1]), |e| (e.x, e.y)));
        }
    }
    None
}

/// The best straight wall right now (None: nothing worth it, unless `relaxed`, used when the cast already happened).
/// Fog of war: only enemies his team can see count; the ones it can't see are assumed to come from their base.
/// A wall that reaches terrain at both ends (seals a passage) is worth more. Candidates:
///   Cut: an enemy group his side can take (as many of his team around it, a dive on his tower, or a lone enemy at
///     half HP with two of his team on it): the wall goes behind it, across its way home (which is also the way
///     their unseen teammates would come). He jumps off behind the group, between it and the wall.
///   Split: his team is fighting part of theirs and the rest is on its way: the wall goes between the fight and
///     the reinforcements, when his side wins the fight once it's cut off. He jumps off on the fight's side.
///   Lock: his team is on a big neutral monster: the wall goes between it and where the enemies would come from.
///   Trap: the enemies are on one and his team is close enough to punish: the wall cuts their way home.
///   Peel: his side is outnumbered or a teammate is about to die: the wall goes between the teams; he jumps off
///     on his team's side.
fn plan_ult(sim: &StableSim<'_>, m: &Champ, all: &[Champ], relaxed: bool) -> Option<UltPlan> {
    let enemies: Vec<&Champ> = all.iter().filter(|c| c.team != m.team && sim.is_visible(m.team, c.id)).collect();
    let unseen = all.iter().filter(|c| c.team != m.team).count() - enemies.len();
    let allies: Vec<&Champ> = all.iter().filter(|c| c.team == m.team).collect();
    let count = |v: &[&Champ], x: i64, y: i64, r: i64| v.iter().filter(|c| d2(c.x, c.y, x, y) <= sq(r)).count();
    let towers: Vec<(i64, i64)> = (0..sim.tower_count()).filter_map(|i| sim.get_entity(sim.tower_id_at(i)))
        .filter(|t| t.is_alive() && t.team() == m.team).map(|t| { let (x, y) = t.pos(); (x as i64, y as i64) }).collect();
    let their_team = all.iter().find(|c| c.team != m.team).map_or(1 - m.team.min(1), |c| c.team);
    let their_base = base_of(sim, their_team, m.team, (MAP - m.x, MAP - m.y));
    let key_near = |c: (i64, i64)| enemies.iter().filter(|e| d2(e.x, e.y, m.x, m.y) <= sq(ULT_RANGE - 5_000))
        .min_by_key(|e| (d2(e.x, e.y, c.0, c.1), e.id)).map(|e| e.id);
    let at = |c: (i64, i64), d: (f64, f64), k: f64| ((c.0 as f64 + d.0 * k) as i64, (c.1 as f64 + d.1 * k) as i64);
    let min_len = if relaxed { 60_000.0 } else { 90_000.0 };   // round 38: was 120000 (more walls in tight spots)
    // (round 35: the round-30 hold for objectives is gone, it cost too many walls; objective plans still score
    // highest, so they win whenever one is on offer)
    let teams: Vec<usize> = all.iter().map(|c| c.team).collect();
    let mut best: Option<UltPlan> = None;
    // a plan needs an enemy his team can see to hold (the AI only presses the ult at an enemy in CC), unless the cast
    // already happened
    let mut consider = |p: UltPlan| if (p.key.is_some() || relaxed) && best.as_ref().map_or(true, |b| p.score > b.score) { best = Some(p); };

    // The editor's Map tab (round 45): his wall lines and the team's "block this choke" lines are built as drawn;
    // "control this area" walls the area off from the enemy side. Weighed against his own reads by priority
    // (5 beats everything but a sure pick), and only with his team there.
    {
        use crate::tactics::{marks_for, team_there, Kind};
        let mut lines = marks_for(sim, all, m, Kind::Wall);
        lines.extend(marks_for(sim, all, m, Kind::Block));
        for k in lines.into_iter().filter(|k| team_there(all, k, m.team)) {
            let mid = ((k.a.0 + k.b.0) / 2, (k.a.1 + k.b.1) / 2);
            if d2(mid.0, mid.1, m.x, m.y) > sq(ULT_REACH) { continue; }
            let (s, t) = if d2(k.a.0, k.a.1, m.x, m.y) <= d2(k.b.0, k.b.1, m.x, m.y) { (k.a, k.b) } else { (k.b, k.a) };
            // as drawn, but never over terrain (round 30's rule): the free stretch of the line around its middle, so a
            // line drawn across a choke rests on the terrain at both ends
            if walls::wall_at(mid.0, mid.1) { continue; }
            let (s, t) = (walls::clip(mid.0, mid.1, s.0, s.1), walls::clip(mid.0, mid.1, t.0, t.1));
            if d2(s.0, s.1, t.0, t.1) < sq(40_000) { continue; }
            let along = norm((t.0 - s.0) as f64, (t.1 - s.1) as f64);
            let mut ax = (-along.1, along.0);
            if (ax.0 * (their_base.0 - mid.0) as f64 + ax.1 * (their_base.1 - mid.1) as f64) < 0.0 { ax = (-ax.0, -ax.1); }
            let jump = walls::pull_back(mid.0, mid.1, at(mid, ax, -9_000.0).0, at(mid, ax, -9_000.0).1);
            // macro with Omen (round 51): a choke his team has already smoked counts for less (the vision is taken
            // care of; a wall still stops them walking through, so it's not dropped)
            let smoked = crate::valorant::smokes_up(all, sim.tick()).iter().any(|sm| sm.team == m.team && sm.end > sim.tick() + 120
                && d2(sm.x, sm.y, mid.0, mid.1) <= sq(crate::valorant::SMOKE_R));
            consider(UltPlan { pts: vec![s, t], axis: ax, jump_to: Some(jump), key: key_near(mid), score: 50 + k.prio * 12 - if smoked { 15 } else { 0 }, rally: Some((mid, None)) });
        }
        for k in marks_for(sim, all, m, Kind::Control).into_iter().filter(|k| team_there(all, k, m.team)) {
            if d2(k.a.0, k.a.1, m.x, m.y) > sq(ULT_REACH) { continue; }
            let foes: Vec<(i64, i64)> = enemies.iter().filter(|e| d2(e.x, e.y, k.a.0, k.a.1) <= sq(k.r + 200_000)).map(|e| (e.x, e.y)).collect();
            let from = if foes.is_empty() { their_base } else { centroid(&foes) };
            let ax = norm((from.0 - k.a.0) as f64, (from.1 - k.a.1) as f64);
            let Some((s, t, sealed)) = lay_line(m, k.a, at(k.a, ax, (k.r + 20_000) as f64), ax, min_len) else { continue };
            let jump = walls::pull_back(k.a.0, k.a.1, at(k.a, ax, k.r as f64 * 0.5).0, at(k.a, ax, k.r as f64 * 0.5).1);
            consider(UltPlan { pts: vec![s, t], axis: ax, jump_to: Some(jump), key: key_near(k.a), score: 45 + k.prio * 12 + sealed as i64 * 5, rally: Some((k.a, None)) });
        }
    }
    // Cut
    for e in enemies.iter().filter(|e| d2(e.x, e.y, m.x, m.y) <= sq(ULT_REACH)) {
        let grp: Vec<&&Champ> = enemies.iter().filter(|o| d2(o.x, o.y, e.x, e.y) <= sq(30_000)).collect();
        let c = centroid(&grp.iter().map(|g| (g.x, g.y)).collect::<Vec<_>>());
        let n = grp.len();
        let spread = grp.iter().map(|g| (d2(g.x, g.y, c.0, c.1) as f64).sqrt()).fold(0.0, f64::max);
        let mates = count(&allies, c.0, c.1, 90_000);
        let low = grp.iter().filter(|g| pct(g) <= 50).count();
        let dive = towers.iter().any(|t| d2(t.0, t.1, c.0, c.1) <= sq(35_000));
        let favourable = mates >= n || (dive && mates + 1 >= n);
        // round 38: any fight his side can take is worth a wall, even a 1v1 or a 2v2 in a lane (a gank: cut the laner's
        // way home)
        if !favourable && !relaxed { continue; }
        let h = norm((their_base.0 - c.0) as f64, (their_base.1 - c.1) as f64);
        let Some((s, t, sealed)) = lay_line(m, c, at(c, h, spread + 24_000.0), h, min_len) else { continue };
        let jump = walls::pull_back(c.0, c.1, at(c, h, spread + 9_000.0).0, at(c, h, spread + 9_000.0).1);
        let score = 20 + n as i64 * 10 + mates as i64 * 3 + low as i64 * 8 + if dive { 15 } else { 0 } + sealed as i64 * 10
            + unseen as i64 * 3 - if favourable { 0 } else { 40 };
        consider(UltPlan { pts: vec![s, t], axis: h, jump_to: Some(jump), key: key_near(c), score, rally: Some((c, None)) });
    }
    // Pick (round 30): a teammate has just caught an enemy (crowd control) and is on it: the wall goes between the
    // caught enemy and whoever would come to help (the visible enemies, else their base), so the pick can't be saved
    // and the enemy can't walk away once it's free. Always worth it, objectives or not, when his side is there to
    // finish it. He jumps off next to it, on the wall side.
    for e in enemies.iter().filter(|e| d2(e.x, e.y, m.x, m.y) <= sq(ULT_REACH) && caught(sim, e.id)) {
        let on_it = allies.iter().filter(|a| a.id != m.id && d2(a.x, a.y, e.x, e.y) <= sq(60_000)).count();
        if on_it == 0 { continue; }
        let foes = count(&enemies, e.x, e.y, 50_000);
        if count(&allies, e.x, e.y, 70_000) < foes { continue; }
        let help: Vec<(i64, i64)> = enemies.iter().filter(|o| o.id != e.id && d2(o.x, o.y, e.x, e.y) <= sq(200_000)).map(|o| (o.x, o.y)).collect();
        let from = if help.is_empty() { their_base } else { centroid(&help) };
        let ax = norm((from.0 - e.x) as f64, (from.1 - e.y) as f64);
        let c = (e.x, e.y);
        let Some((s, t, sealed)) = lay_line(m, c, at(c, ax, 25_000.0), ax, min_len) else { continue };
        let jump = walls::pull_back(c.0, c.1, at(c, ax, 10_000.0).0, at(c, ax, 10_000.0).1);
        let score = 50 + on_it as i64 * 10 + if pct(e) <= 50 { 15 } else { 0 } + help.len() as i64 * 5 + sealed as i64 * 10;
        consider(UltPlan { pts: vec![s, t], axis: ax, jump_to: Some(jump), key: Some(e.id), score, rally: Some((c, Some(e.id))) });
    }
    // Split
    let fight: Vec<&&Champ> = enemies.iter().filter(|e| d2(e.x, e.y, m.x, m.y) <= sq(ULT_REACH)
        && allies.iter().any(|a| d2(a.x, a.y, e.x, e.y) <= sq(50_000))).collect();
    if !fight.is_empty() {
        let fc = centroid(&fight.iter().map(|g| (g.x, g.y)).collect::<Vec<_>>());
        let coming: Vec<(i64, i64)> = enemies.iter().filter(|e| !fight.iter().any(|f| f.id == e.id) && d2(e.x, e.y, fc.0, fc.1) <= sq(300_000))
            .map(|e| (e.x, e.y)).collect();
        let mates = count(&allies, fc.0, fc.1, 90_000);
        if !coming.is_empty() && mates >= fight.len() {
            let rc = centroid(&coming);
            let dist = (d2(rc.0, rc.1, fc.0, fc.1) as f64).sqrt();
            if dist >= 35_000.0 {   // round 38: was 55000 (isolate a laner from its support, too)
                let ax = norm((rc.0 - fc.0) as f64, (rc.1 - fc.1) as f64);
                let wc = at(fc, ax, (dist * 0.5).clamp(35_000.0, dist - 20_000.0));
                if let Some((s, t, sealed)) = lay_line(m, fc, wc, ax, min_len) {
                    let jump = walls::pull_back(fc.0, fc.1, at(wc, ax, -20_000.0).0, at(wc, ax, -20_000.0).1);
                    let score = 30 + coming.len() as i64 * 12 + sealed as i64 * 10 + if mates < fight.len() + coming.len() { 10 } else { 0 };
                    consider(UltPlan { pts: vec![s, t], axis: ax, jump_to: Some(jump), key: key_near(fc), score, rally: Some((fc, None)) });
                }
            }
        }
    }
    // Lock / Trap (big neutral monsters: their spots are known through the fog)
    for i in 0..sim.entity_count() {
        let Some(t) = sim.entity_at(i) else { continue };
        if !t.is_alive() || t.is_champion() || t.is_tower() || teams.contains(&t.team()) || t.hp().1 < 2_500 { continue; }
        let c = { let (x, y) = t.pos(); (x as i64, y as i64) };
        if d2(c.0, c.1, m.x, m.y) > sq(ULT_REACH) { continue; }
        // (round 40: 70000, was 40000: ranged champions hit it from further out)
        let (ours, theirs) = (count(&allies, c.0, c.1, 70_000), count(&enemies, c.0, c.1, 70_000));
        if ours >= 1 && theirs == 0 {
            let threats: Vec<(i64, i64)> = enemies.iter().filter(|e| d2(e.x, e.y, c.0, c.1) <= sq(250_000)).map(|e| (e.x, e.y)).collect();
            if threats.is_empty() && unseen < 2 { continue; }
            let from = if threats.is_empty() { their_base } else { centroid(&threats) };
            let ax = norm((from.0 - c.0) as f64, (from.1 - c.1) as f64);
            if let Some((s, t, sealed)) = lay_line(m, c, at(c, ax, 35_000.0), ax, min_len) {
                let score = 60 + ours as i64 * 5 + threats.len() as i64 * 8 + sealed as i64 * 10 + if threats.is_empty() { unseen as i64 * 3 } else { 0 };
                consider(UltPlan { pts: vec![s, t], axis: ax, jump_to: Some(c), key: key_near(c), score, rally: Some((c, None)) });
            }
        } else if theirs >= 1 && count(&allies, c.0, c.1, 120_000) >= theirs {
            let h = norm((their_base.0 - c.0) as f64, (their_base.1 - c.1) as f64);
            if let Some((s, t, sealed)) = lay_line(m, c, at(c, h, 35_000.0), h, min_len) {
                let jump = walls::pull_back(c.0, c.1, at(c, h, 20_000.0).0, at(c, h, 20_000.0).1);
                let score = 65 + theirs as i64 * 10 + sealed as i64 * 10;
                consider(UltPlan { pts: vec![s, t], axis: h, jump_to: Some(jump), key: key_near(c), score, rally: Some((c, None)) });
            }
        }
    }
    // Screen (round 38): his team moving together (2+ within 60000 of him) with enemies his team can see off to one
    // side but not on them yet: a wall between, so they can pass, rotate or take a fight on their own terms
    let mates_here: Vec<(i64, i64)> = allies.iter().filter(|a| d2(a.x, a.y, m.x, m.y) <= sq(60_000)).map(|a| (a.x, a.y)).collect();
    if mates_here.len() >= 2 {
        let ac = centroid(&mates_here);
        let near_foes = enemies.iter().filter(|e| d2(e.x, e.y, ac.0, ac.1) <= sq(35_000)).count();
        let off: Vec<(i64, i64)> = enemies.iter().filter(|e| d2(e.x, e.y, ac.0, ac.1) <= sq(160_000)).map(|e| (e.x, e.y)).collect();
        if near_foes == 0 && !off.is_empty() {
            let ec = centroid(&off);
            let ax = norm((ec.0 - ac.0) as f64, (ec.1 - ac.1) as f64);
            let gap = ((d2(ec.0, ec.1, ac.0, ac.1) as f64).sqrt() * 0.5).min(35_000.0);
            let mid = at(ac, ax, gap);
            if let Some((s, t, sealed)) = lay_line(m, ac, mid, ax, min_len) {
                let jump = walls::pull_back(ac.0, ac.1, at(mid, ax, -20_000.0).0, at(mid, ax, -20_000.0).1);
                let score = 12 + mates_here.len() as i64 * 3 + off.len() as i64 * 2 + sealed as i64 * 6;
                consider(UltPlan { pts: vec![s, t], axis: ax, jump_to: Some(jump), key: key_near(ec), score, rally: None });
            }
        }
    }
    // Escape (round 40): he's being chased (at half HP or less, or outnumbered) or a teammate at 40% or less is, with
    // enemies on them: the wall goes between the chasers and the ones running, close to the chasers, and he jumps off
    // on the safe side, toward home. Worth more than any fight while it's needed.
    {
        let me_hunted = count(&enemies, m.x, m.y, 50_000) > 0 && (pct(m) <= 50 || count(&enemies, m.x, m.y, 50_000) > count(&allies, m.x, m.y, 50_000));
        let friend: Option<&&Champ> = allies.iter().filter(|a| a.id != m.id && pct(a) <= 40 && d2(a.x, a.y, m.x, m.y) <= sq(120_000)
            && count(&enemies, a.x, a.y, 45_000) > 0).min_by_key(|a| (pct(a), a.id));
        if me_hunted || friend.is_some() {
            let mut runners: Vec<(i64, i64)> = Vec::new();
            if me_hunted { runners.push((m.x, m.y)); }
            if let Some(f) = friend { runners.push((f.x, f.y)); }
            let rc = centroid(&runners);
            let chasers: Vec<(i64, i64)> = enemies.iter().filter(|e| d2(e.x, e.y, rc.0, rc.1) <= sq(70_000)).map(|e| (e.x, e.y)).collect();
            if !chasers.is_empty() {
                let ec = centroid(&chasers);
                let ax = norm((ec.0 - rc.0) as f64, (ec.1 - rc.1) as f64);
                let gap = ((d2(ec.0, ec.1, rc.0, rc.1) as f64).sqrt() * 0.6).clamp(8_000.0, 30_000.0);
                let mid = at(rc, ax, gap);
                if let Some((s, t, sealed)) = lay_line(m, rc, mid, ax, min_len.min(60_000.0)) {
                    let (hx, hy) = team_towers(sim, m.team).unwrap_or((rc.0 - (ax.0 * 50_000.0) as i64, rc.1 - (ax.1 * 50_000.0) as i64));
                    let home_dir = norm((hx - rc.0) as f64, (hy - rc.1) as f64);
                    let jump = walls::pull_back(rc.0, rc.1, at(rc, home_dir, 25_000.0).0, at(rc, home_dir, 25_000.0).1);
                    let score = 80 + if friend.is_some() { 10 } else { 0 } + chasers.len() as i64 * 4 + sealed as i64 * 8;
                    consider(UltPlan { pts: vec![s, t], axis: ax, jump_to: Some(jump), key: key_near(ec), score, rally: None });
                }
            }
        }
    }
    if best.is_some() { return best; }
    // Peel
    let foes = count(&enemies, m.x, m.y, 45_000);
    let ally_dying = allies.iter().any(|a| a.id != m.id && d2(a.x, a.y, m.x, m.y) <= sq(60_000) && pct(a) <= 40 && count(&enemies, a.x, a.y, 25_000) > 0);
    if foes > count(&allies, m.x, m.y, 45_000) || ally_dying || relaxed {
        let ec: Vec<(i64, i64)> = enemies.iter().filter(|e| d2(e.x, e.y, m.x, m.y) <= sq(60_000)).map(|e| (e.x, e.y)).collect();
        if ec.is_empty() { return None; }
        let ac: Vec<(i64, i64)> = allies.iter().filter(|a| d2(a.x, a.y, m.x, m.y) <= sq(60_000)).map(|a| (a.x, a.y)).collect();
        let (ec, ac) = (centroid(&ec), centroid(&ac));
        let ax = norm((ec.0 - ac.0) as f64, (ec.1 - ac.1) as f64);
        let gap = ((d2(ec.0, ec.1, ac.0, ac.1) as f64).sqrt() * 0.5).min(20_000.0);
        let mid = at(ac, ax, gap);
        let (s, t, _) = lay_line(m, ac, mid, ax, min_len.min(60_000.0))?;
        let jump = walls::pull_back(ac.0, ac.1, at(mid, ax, -25_000.0).0, at(mid, ax, -25_000.0).1);
        return Some(UltPlan { pts: vec![s, t], axis: ax, jump_to: Some(jump), key: key_near(ec), score: 1, rally: None });
    }
    None
}

/// The way around every standing wall from (x, y) toward (tx, ty): the end of a wall that can be reached in a
/// straight line without crossing any wall, cheapest overall (a box is left through its open side).
pub fn detour_all(walls_: &[Wall], tick: usize, x: i64, y: i64, tx: i64, ty: i64) -> Option<(i64, i64)> {
    let mut best: Option<(f64, (i64, i64))> = None;
    for w in walls_ {
        let (ex, ey) = w.built_end(tick);
        let (ux, uy) = norm((ex - w.ax) as f64, (ey - w.ay) as f64);
        for &(bx, by, sgn) in &[(w.ax, w.ay, -1.0), (ex, ey, 1.0)] {
            for &k in &[16_000.0, 26_000.0] {
                let p = ((bx as f64 + ux * sgn * k) as i64, (by as f64 + uy * sgn * k) as i64);
                if p.0 < 0 || p.1 < 0 || walls::wall_at(p.0, p.1) { continue; }
                if walls_.iter().any(|v| crosses(v, tick, x, y, p.0, p.1)) { continue; }
                let on = walls_.iter().any(|v| { let (sd, along) = side(v, p.0, p.1); let (vx, vy) = v.built_end(tick);
                    sd.abs() < WALL_HALF as f64 + 1_000.0 && along >= -2_000.0 && along <= ((vx - v.ax) as f64).hypot((vy - v.ay) as f64) + 2_000.0 });
                if on { continue; }
                let blocked_after = walls_.iter().any(|v| crosses(v, tick, p.0, p.1, tx, ty));
                let cost = ((p.0 - x) as f64).hypot((p.1 - y) as f64) + ((tx - p.0) as f64).hypot((ty - p.1) as f64)
                    + if blocked_after { 60_000.0 } else { 0.0 };
                if best.map_or(true, |(c, _)| cost < c) { best = Some((cost, p)); }
            }
        }
    }
    best.map(|(_, p)| p)
}

// ------------------------------------------------------------------ the input AI: walk around the wall

/// When a boat wall is up, a MOVE order whose straight path would cross it becomes a move to the way around it (past
/// the nearer useful end). For Steve it also keeps him with his team: a move that would take him alone among two or
/// more enemies becomes a move to a teammate who's fighting. Only moves are ever touched: casts and attacks are never injected, replaced or held
/// (that broke the game's own planner once, notes Sep 30: plan_legacy panics and frozen players). With no wall up it
/// only ever touches Steve's own moves.
#[derive(Clone, Default)]
pub struct WallAi;

impl StablePlayerAi for WallAi {
    fn clone_box(&self) -> Box<dyn StablePlayerAi> {
        Box::new(self.clone())
    }
    fn id(&self) -> String {
        format!("{MOD_ID}:wall_ai")
    }
    fn think(&mut self, ctx: &mut StableAiContext<'_>, base: Option<InputV1>) -> Option<InputV1> {
        // Scribble's mastery is per athlete: this is the one place the game says who plays him (reads only)
        if ctx.champion_name().map_or(false, |n| n.ends_with("scribble")) {
            let (pid, aid) = (ctx.player_id(), ctx.athlete_id());
            if let Some(sim) = ctx.sim() { crate::scribble::note_athlete(sim.seed(), pid, aid); }
        }
        let input = base?;
        if input.kind != InputKindV1::Move.code() {
            return None;
        }
        let pid = ctx.player_id();
        let lane = ctx.lane();
        let out = {
            let sim = ctx.sim()?;
            let tick = sim.tick();
            let all = champions(&sim);
            let walls = walls_up(&all, tick);
            let me_e = sim.get_player(pid)?.champion()?;
            let me = all.iter().find(|c| c.id == me_e.id())?;
            let (mx, my) = (me.x, me.y);
            let mut dest = (input.x as i64, input.y as i64);
            let mut changed = false;
            // Steve (a support): never walk alone into the enemies; go stand with a teammate who's fighting instead
            if me.name.ends_with("steve") {
                let foes = |x: i64, y: i64, r: i64| all.iter().filter(|c| c.team != me.team && sim.is_visible(me.team, c.id) && d2(c.x, c.y, x, y) <= sq(r)).count();
                let mates: Vec<&Champ> = all.iter().filter(|c| c.team == me.team && c.id != me.id).collect();
                let backed = mates.iter().filter(|a| d2(a.x, a.y, dest.0, dest.1) <= sq(40_000)).count();
                if foes(dest.0, dest.1, 35_000) >= 2 && backed == 0 {
                    let pick = mates.iter().filter(|a| foes(a.x, a.y, 40_000) > 0).min_by_key(|a| (d2(a.x, a.y, mx, my), a.id))
                        .or_else(|| mates.iter().min_by_key(|a| (d2(a.x, a.y, mx, my), a.id)));
                    if let Some(a) = pick {
                        let (hx, hy) = team_towers(&sim, me.team).unwrap_or((a.x, a.y));
                        let (dx, dy) = norm((hx - a.x) as f64, (hy - a.y) as f64);
                        dest = walls::pull_back(a.x, a.y, a.x + (dx * 6_000.0) as i64, a.y + (dy * 6_000.0) as i64);
                        changed = true;
                    }
                }
            }
            // Steve's call (round 35): a teammate on a move order goes where his wall is meant to be used, while it's a
            // winning situation (Steve keeps the call up only while it is). Not when low.
            // (Omen calls one too, round 55, after a Paranoia that blinds 2+; he keeps his own range himself)
            if !me.name.ends_with("steve") && !me.name.ends_with("_omen") && pct(me) > 30 {
                if let Some(t) = rally_of(&sim, &all, me.team, tick) {
                    let dd = d2(t.0, t.1, mx, my);
                    if dd <= sq(RALLY_R) && dd > sq(20_000) && d2(dest.0, dest.1, t.0, t.1) > sq(25_000) {
                        dest = walls::pull_back(mx, my, t.0, t.1);
                        changed = true;
                    }
                }
            }
            // the team's plan from the Map tab: avoid areas, gather points / controlled areas
            if let Some(g) = crate::tactics::move_goal(&sim, &all, me, lane, dest) {
                dest = g;
                changed = true;
            }
            // Omen's smokes: Omen lurking in his own holds still; anyone else checks an enemy smoke in the way first
            if let Some(h) = crate::valorant::omen_hold(&all, me, tick) {
                dest = h;
                changed = true;
            } else if let Some(c) = crate::valorant::clear_smoke(&sim, &all, me, dest, tick) {
                dest = c;
                changed = true;
            }
            if let Some(w) = walls.iter().find(|w| crosses(w, tick, mx, my, dest.0, dest.1)) {
                let (wx, wy) = detour_all(&walls, tick, mx, my, dest.0, dest.1).unwrap_or_else(|| detour(w, tick, mx, my, dest.0, dest.1));
                if !walls::wall_at(wx, wy) {
                    dest = (wx, wy);
                    changed = true;
                }
            }
            if !changed || walls::wall_at(dest.0, dest.1) { return None; }
            InputV1::move_to(dest.0.max(0) as u64, dest.1.max(0) as u64)
        };
        if ctx.is_valid_input(&out) { Some(out) } else { None }
    }
}

// ------------------------------------------------------------------ the passive

impl Steve {
    fn tnt_boom(sim: &mut StableSim<'_>, m: &Champ, all: &[Champ], x: i64, y: i64) {
        fx(sim, &v(m, "boom"), m.id, x, y, 24);
        let dmg = TNT_DMG.0 + m.max_hp * TNT_DMG.1 / 100;
        for e in all.iter().filter(|c| c.team != m.team && d2(c.x, c.y, x, y) <= sq(TNT_R)) {
            sim.deal_damage(m.id, e.id, dmg, 0, AttackTypeV1::Skill);
            let (dx, dy) = norm((e.x - x) as f64, (e.y - y) as f64);
            let mut push = CcV1::of_kind(CcKindV1::ForceMove, TNT_PUSH.1);
            push.dx = (dx * 1000.0) as i64;
            push.dy = (dy * 1000.0) as i64;
            push.speed = TNT_PUSH.0;
            sim.apply_cc(e.id, &push);
        }
    }

    /// Where the TNT should land around `t`. The blast knocks enemies away from it, so:
    ///   behind the target (seen from Steve) → it's blown toward Steve and his team: when they can take the fight
    ///     (he's healthy and his side isn't outnumbered around him);
    ///   in front of it (between them) → it's blown away: peel, when he's low or his side is outnumbered.
    /// Throw a TNT (round 36: he plays it with the rod):
    ///   an enemy on his fishing line → it lands where the enemy is being reeled to (just on their side of him), lit, so
    ///     the pull drags them into the blast;
    ///   otherwise → where the target will be when it lands (led by its movement), behind or in front of it (tnt_spot).
    fn throw_tnt(&mut self, sim: &mut StableSim<'_>, m: &Champ, all: &[Champ], t: &Champ, tick: usize) {
        let tethered = self.tether.filter(|&(_, until)| tick < until).and_then(|(id, _)| all.iter().find(|c| c.id == id));
        let (tx, ty) = if let Some(e) = tethered {
            let (dx, dy) = norm((e.x - m.x) as f64, (e.y - m.y) as f64);
            walls::pull_back(m.x, m.y, m.x + (dx * 14_000.0) as i64, m.y + (dy * 14_000.0) as i64)
        } else {
            let (px, py) = self.seen.iter().find(|s| s.0 == t.id).map_or((t.x, t.y), |s| (s.1, s.2));
            let lead = (TNT_FLIGHT + TNT_FUSE / 2) as i64;
            let ahead = Champ { x: (t.x + (t.x - px) * lead).clamp(0, MAP), y: (t.y + (t.y - py) * lead).clamp(0, MAP), ..t.clone() };
            Self::tnt_spot(m, all, &ahead)
        };
        fly(sim, &v(m, "tnt_fly"), m, m.x, m.y, tx, ty, TNT_FLIGHT);
        self.tnts.push(Tnt { x: tx, y: ty, land: tick + TNT_FLIGHT, det: usize::MAX, expire: tick + TNT_FLIGHT + TNT_LIFE, from: (m.x, m.y), checked: false });
    }

    fn tnt_spot(m: &Champ, all: &[Champ], t: &Champ) -> (i64, i64) {
        let near = |team_same: bool| all.iter().filter(|c| (c.team == m.team) == team_same && d2(c.x, c.y, m.x, m.y) <= sq(TNT_SIDE_R)).count();
        let peel = pct(m) <= TNT_PEEL_PCT || near(false) > near(true);
        let (dx, dy) = norm((t.x - m.x) as f64, (t.y - m.y) as f64);
        let off = if peel { -TNT_OFFSET } else { TNT_OFFSET } as f64;
        walls::pull_back(t.x, t.y, (t.x + (dx * off) as i64).max(0), (t.y + (dy * off) as i64).max(0))
    }

    /// Where the pearl should land, or None when there's no good reason to pearl. He never pearls alone into the
    /// enemies; supporting a teammate comes before engaging:
    ///   an enemy on his line → toward his team (dragging them along);
    ///   a teammate / tower / objective in trouble behind a wall → over it;
    ///   he's low with enemies on him → away from them, toward his team;
    ///   a teammate in a fight farther than 35000 → beside them, on the side away from the enemies (lowest HP first);
    ///   the target → only if a teammate is there (within 40000 of where he lands) and the enemies there don't
    ///     outnumber his side.
    /// Fog of war: only enemies his team can see count.
    fn pearl_spot(&self, sim: &StableSim<'_>, m: &Champ, all: &[Champ], tick: usize) -> Option<(i64, i64)> {
        let enemies: Vec<&Champ> = all.iter().filter(|c| c.team != m.team && sim.is_visible(m.team, c.id)).collect();
        let mates: Vec<&Champ> = all.iter().filter(|c| c.team == m.team && c.id != m.id).collect();
        let foes_at = |x: i64, y: i64, r: i64| enemies.iter().filter(|e| d2(e.x, e.y, x, y) <= sq(r)).count();
        let mates_at = |x: i64, y: i64, r: i64| mates.iter().filter(|a| d2(a.x, a.y, x, y) <= sq(r)).count();
        let toward = |x: i64, y: i64, k: f64| -> (i64, i64) {
            let (dx, dy) = norm((x - m.x) as f64, (y - m.y) as f64);
            let dist = ((d2(x, y, m.x, m.y) as f64).sqrt() - k).clamp(0.0, PEARL_RANGE as f64);
            (m.x + (dx * dist) as i64, m.y + (dy * dist) as i64)
        };
        if self.tether.map_or(false, |(_, until)| tick < until) {
            let (hx, hy) = home(sim, m, all);
            let (dx, dy) = norm((hx - m.x) as f64, (hy - m.y) as f64);
            return Some((m.x + (dx * PEARL_ESCAPE as f64) as i64, m.y + (dy * PEARL_ESCAPE as f64) as i64));
        }
        if pct(m) > LOW_PCT {
            if let Some((nx, ny)) = wall_need(sim, all, m, tick, PEARL_RANGE + 20_000) { return Some(toward(nx, ny, 9_000.0)); }
        }
        if pct(m) <= LOW_PCT && foes_at(m.x, m.y, 30_000) > 0 {
            let near = enemies.iter().min_by_key(|e| d2(e.x, e.y, m.x, m.y));
            let away = near.map_or((1.0, 0.0), |e| norm((m.x - e.x) as f64, (m.y - e.y) as f64));
            let (hx, hy) = home(sim, m, all);
            let h = norm((hx - m.x) as f64, (hy - m.y) as f64);
            let (dx, dy) = norm(away.0 + h.0, away.1 + h.1);
            return Some((m.x + (dx * PEARL_ESCAPE as f64) as i64, m.y + (dy * PEARL_ESCAPE as f64) as i64));
        }
        // support a teammate in a fight
        let support = mates.iter().filter(|a| d2(a.x, a.y, m.x, m.y) > sq(35_000) && d2(a.x, a.y, m.x, m.y) <= sq(PEARL_RANGE + 25_000)
            && foes_at(a.x, a.y, 35_000) > 0).min_by_key(|a| (pct(a), a.id));
        if let Some(a) = support {
            let e = enemies.iter().min_by_key(|e| d2(e.x, e.y, a.x, a.y)).map_or((a.x, a.y), |e| (e.x, e.y));
            let (dx, dy) = norm((a.x - e.0) as f64, (a.y - e.1) as f64);
            let spot = (a.x + (dx * 8_000.0) as i64, a.y + (dy * 8_000.0) as i64);
            let (lx, ly) = toward(spot.0, spot.1, 0.0);
            if mates_at(lx, ly, 40_000) > 0 { return Some((lx, ly)); }
        }
        // engage, never alone
        let target = enemies.iter().filter(|e| e.has("stv_target")).min_by_key(|e| (d2(e.x, e.y, m.x, m.y), e.id)).copied()
            .or_else(|| enemies.iter().min_by_key(|e| (d2(e.x, e.y, m.x, m.y), e.id)).copied())?;
        let (lx, ly) = toward(target.x, target.y, 9_000.0);
        let backed = mates_at(lx, ly, 40_000);
        (backed > 0 && foes_at(lx, ly, 35_000) <= backed + 1).then_some((lx, ly))
    }

    /// Which tool fits right now (0 pearl, 1 TNT, 2 golden apple), checked in this order:
    ///   an enemy on his line, or a wall cutting him off from something urgent → pearl;
    ///   a teammate in trouble (≤ 45% HP with an enemy on them, or the ally he just hooked) → apple;
    ///   himself low (≤ 35%) with enemies on him → pearl away if he's outnumbered, else apple himself;
    ///   a teammate fighting farther away → pearl to them (support first);
    ///   enemies grouped (2+ around the target) or the target close (≤ 60000) → TNT;
    ///   the target far → pearl in only if his team is there (pearl_spot), otherwise TNT.
    fn choose_tool(&self, sim: &StableSim<'_>, m: &Champ, all: &[Champ], tick: usize) -> usize {
        let enemies: Vec<&Champ> = all.iter().filter(|c| c.team != m.team && sim.is_visible(m.team, c.id)).collect();
        let foes = |x: i64, y: i64, r: i64| enemies.iter().filter(|c| d2(c.x, c.y, x, y) <= sq(r)).count();
        let friends = |x: i64, y: i64, r: i64| all.iter().filter(|c| c.team == m.team && d2(c.x, c.y, x, y) <= sq(r)).count();
        // an enemy on his line: TNT where they're being reeled to when his side can take them, else pearl them home
        if self.tether.map_or(false, |(_, until)| tick < until) {
            return if foes(m.x, m.y, 45_000) <= friends(m.x, m.y, 45_000) { 1 } else { 0 };
        }
        if pct(m) > LOW_PCT && wall_need(sim, all, m, tick, PEARL_RANGE + 20_000).is_some() { return 0; }
        let hooked = self.ally_hooked.map_or(false, |(id, t)| tick < t + 120 && all.iter().any(|c| c.id == id));
        let ally_trouble = all.iter().any(|c| c.team == m.team && c.id != m.id && d2(c.x, c.y, m.x, m.y) <= sq(APPLE_R)
            && pct(c) <= 45 && foes(c.x, c.y, 30_000) > 0);
        if hooked || ally_trouble { return 2; }
        if pct(m) <= LOW_PCT && foes(m.x, m.y, 30_000) > 0 {
            return if foes(m.x, m.y, 35_000) > friends(m.x, m.y, 35_000) { 0 } else { 2 };
        }
        let supporting = all.iter().any(|a| a.team == m.team && a.id != m.id && d2(a.x, a.y, m.x, m.y) > sq(35_000)
            && d2(a.x, a.y, m.x, m.y) <= sq(PEARL_RANGE + 25_000) && foes(a.x, a.y, 35_000) > 0);
        if supporting && self.pearl_spot(sim, m, all, tick).is_some() { return 0; }
        let target = enemies.iter().filter(|e| e.has("stv_target")).min_by_key(|e| (d2(e.x, e.y, m.x, m.y), e.id)).copied()
            .or_else(|| enemies.iter().min_by_key(|e| (d2(e.x, e.y, m.x, m.y), e.id)).copied());
        match target {
            Some(t) if foes(t.x, t.y, 30_000) >= 2 || d2(t.x, t.y, m.x, m.y) <= sq(60_000) => 1,
            Some(_) if self.pearl_spot(sim, m, all, tick).is_some() => 0,
            _ => 1,
        }
    }

    fn use_tool(&mut self, sim: &mut StableSim<'_>, m: &Champ, all: &[Champ], tick: usize) {
        let enemies: Vec<&Champ> = all.iter().filter(|c| c.team != m.team).collect();
        let target = enemies.iter().filter(|e| e.has("stv_target")).min_by_key(|e| (d2(e.x, e.y, m.x, m.y), e.id)).copied()
            .or_else(|| enemies.iter().min_by_key(|e| (d2(e.x, e.y, m.x, m.y), e.id)).copied());
        self.tool = self.choose_tool(sim, m, all, tick);
        match self.tool {
            0 if self.pearl_spot(sim, m, all, tick).is_some() => {
                let (tx, ty) = self.pearl_spot(sim, m, all, tick).unwrap();
                let (tx, ty) = walls::clip(m.x, m.y, tx.max(0), ty.max(0));
                fly(sim, &v(m, "pearl"), m, m.x, m.y, tx, ty, PEARL_T);
                self.pearl = Some((m.x, m.y, tx, ty, tick));
            }
            0 => {
                // nowhere good to pearl after all: TNT instead
                self.tool = 1;
                let Some(t) = target else { return };
                self.throw_tnt(sim, m, all, t, tick);
            }
            1 => {
                let Some(t) = target else { return };
                self.throw_tnt(sim, m, all, t, tick);
            }
            _ => {
                // the apple: the ally he just hooked, otherwise the one lowest on HP (him included)
                let hooked = self.ally_hooked.filter(|&(_, t)| tick < t + 120).and_then(|(id, _)| all.iter().find(|c| c.id == id && c.team == m.team));
                let ally = hooked.or_else(|| all.iter().filter(|c| c.team == m.team && d2(c.x, c.y, m.x, m.y) <= sq(APPLE_R)).min_by_key(|c| (pct(c), c.id)));
                let Some(a) = ally else { return };
                if a.id != m.id {
                    fly(sim, &v(m, "apple"), m, m.x, m.y, a.x, a.y, 12);
                }
                sim.heal(m.id, a.id, APPLE_HEAL.0 + m.max_hp * APPLE_HEAL.1 / 100);
                sim.entity_add_shield(a.id, m.max_hp * APPLE_SHIELD_PCT / 100, APPLE_SHIELD_TICKS);
                sim.play_view_effect(&v(m, "apple_glow"), m.id, &InputTargetV1::target(a.id), 0, 0, 40);
                self.ally_hooked = None;
            }
        }
    }

    /// What the rod would do toward (x, y) (round 36: he knows before he casts): None = the line is clear, so it reaches
    /// what's there and pulls it; Some(p) = it bites a wall first (terrain or a boat wall) at p, so it's a dash to p.
    fn line_block(m: &Champ, all: &[Champ], tick: usize, x: i64, y: i64) -> Option<(i64, i64)> {
        let walls_ = walls_up(all, tick);
        let d = (d2(x, y, m.x, m.y) as f64).sqrt().min(HOOK_RANGE as f64);
        let (dx, dy) = norm((x - m.x) as f64, (y - m.y) as f64);
        let mut k = 4_000.0;
        while k < d - 4_000.0 {
            let (px, py) = (m.x + (dx * k) as i64, m.y + (dy * k) as i64);
            let boat = walls_.iter().any(|w| { let (sd, along) = side(w, px, py); let (ex, ey) = w.built_end(tick);
                sd.abs() < WALL_HALF as f64 && along >= -(WALL_HALF as f64) && along <= ((ex - w.ax) as f64).hypot((ey - w.ay) as f64) + WALL_HALF as f64 });
            if walls::wall_at(px, py) || boat { return Some((px, py)); }
            k += 3_000.0;
        }
        None
    }

    /// Is a dash to the wall at p a good move? Toward a teammate or his team's side, or into a fight his side can
    /// take (as many of his team around p as enemies his team can see there); never a lone dive when he's low.
    fn dash_ok(sim: &StableSim<'_>, m: &Champ, all: &[Champ], p: (i64, i64)) -> bool {
        if d2(p.0, p.1, m.x, m.y) < sq(20_000) { return false; }   // too short to be worth the rod
        let foes = all.iter().filter(|c| c.team != m.team && sim.is_visible(m.team, c.id) && d2(c.x, c.y, p.0, p.1) <= sq(40_000)).count();
        let mates = all.iter().filter(|c| c.team == m.team && c.id != m.id && d2(c.x, c.y, p.0, p.1) <= sq(50_000)).count();
        if pct(m) <= LOW_PCT { return foes == 0; }
        foes <= mates + 1
    }

    fn cast_rod(&mut self, sim: &mut StableSim<'_>, m: &Champ, all: &[Champ], tick: usize) {
        let enemies: Vec<&Champ> = all.iter().filter(|c| c.team != m.team).collect();
        let near_enemy = |x: i64, y: i64, r: i64| enemies.iter().any(|e| d2(e.x, e.y, x, y) <= sq(r));
        let clear = |x: i64, y: i64| Self::line_block(m, all, tick, x, y).is_none();
        // 1) an ally in trouble (supporting comes first): yanked if the line is clear; if a wall is in the way, he
        //    dashes to that wall instead when it brings him closer to them
        let allies_in_trouble: Vec<&Champ> = all.iter().filter(|c| c.team == m.team && c.id != m.id && pct(c) <= 40 && d2(c.x, c.y, m.x, m.y) <= sq(HOOK_RANGE)
            && d2(c.x, c.y, m.x, m.y) > sq(15_000) && near_enemy(c.x, c.y, 25_000)).collect();
        let ally = allies_in_trouble.iter().filter(|c| clear(c.x, c.y)).min_by_key(|c| (pct(c), c.id)).copied();
        let ally_dash = allies_in_trouble.iter().filter_map(|c| Self::line_block(m, all, tick, c.x, c.y).map(|p| (c, p)))
            .find(|(c, p)| pct(m) > LOW_PCT && d2(p.0, p.1, c.x, c.y) < d2(m.x, m.y, c.x, c.y) && Self::dash_ok(sim, m, all, *p)).map(|(_, p)| p);
        // 2) his TNT on the ground that would miss while an enemy is close enough to bomb: yank it and fling it (only
        //    with a clear line to it)
        let tnt = self.tnts.iter().enumerate().filter(|(_, t)| t.land != usize::MAX && tick >= t.land && d2(t.x, t.y, m.x, m.y) <= sq(HOOK_RANGE)
            && !near_enemy(t.x, t.y, TNT_R) && near_enemy(t.x, t.y, 70_000) && clear(t.x, t.y)).map(|(i, _)| i).next();
        // 3) enemies: only ones with a clear line can be pulled; among those, one whose pull drags them over his TNT,
        //    then the one marked by the data, then the nearest
        let over_tnt = |e: &Champ| self.tnts.iter().any(|t| t.land != usize::MAX && tick >= t.land && {
            let (ax, ay, bx, by) = (e.x as f64, e.y as f64, m.x as f64, m.y as f64);
            let (vx, vy) = (bx - ax, by - ay);
            let k = (((t.x as f64 - ax) * vx + (t.y as f64 - ay) * vy) / (vx * vx + vy * vy).max(1.0)).clamp(0.0, 1.0);
            ((ax + vx * k - t.x as f64).powi(2) + (ay + vy * k - t.y as f64).powi(2)).sqrt() <= 14_000.0
        });
        let pullable: Vec<&Champ> = enemies.iter().filter(|e| d2(e.x, e.y, m.x, m.y) <= sq(HOOK_RANGE) && clear(e.x, e.y)).copied().collect();
        let target = pullable.iter().filter(|e| over_tnt(e)).min_by_key(|e| (d2(e.x, e.y, m.x, m.y), e.id)).copied()
            .or_else(|| pullable.iter().filter(|e| e.has("stv_rod_target")).min_by_key(|e| (d2(e.x, e.y, m.x, m.y), e.id)).copied())
            .or_else(|| pullable.iter().min_by_key(|e| (d2(e.x, e.y, m.x, m.y), e.id)).copied());
        // the data's target behind a wall: the rod would bite the wall, so it's a dash: only when that's a good move
        let engage_dash = enemies.iter().filter(|e| e.has("stv_rod_target")).min_by_key(|e| (d2(e.x, e.y, m.x, m.y), e.id))
            .and_then(|e| Self::line_block(m, all, tick, e.x, e.y)).filter(|p| Self::dash_ok(sim, m, all, *p));
        let (mode, aim) = if let Some(a) = ally {
            (HookMode::Ally(a.id), (a.x, a.y))
        } else if let Some(p) = ally_dash {
            (HookMode::Wall, p)
        } else if let Some(i) = tnt {
            (HookMode::Tnt(i), (self.tnts[i].x, self.tnts[i].y))
        } else if pct(m) <= LOW_PCT && near_enemy(m.x, m.y, 30_000) {
            // low: dash away to a wall (terrain or a boat wall) in reach away from the enemies
            let near = enemies.iter().min_by_key(|e| d2(e.x, e.y, m.x, m.y));
            let (dx, dy) = near.map_or((1.0, 0.0), |e| norm((m.x - e.x) as f64, (m.y - e.y) as f64));
            let far = (m.x + (dx * HOOK_RANGE as f64) as i64, m.y + (dy * HOOK_RANGE as f64) as i64);
            match Self::line_block(m, all, tick, far.0, far.1).filter(|p| d2(p.0, p.1, m.x, m.y) >= sq(25_000)) {
                Some(p) => (HookMode::Wall, p),
                None => match target { Some(t) => (HookMode::Enemy, (t.x, t.y)), None => return },
            }
        } else if let Some(t) = target {
            (HookMode::Enemy, (t.x, t.y))
        } else if let Some(p) = engage_dash {
            (HookMode::Wall, p)
        } else {
            return;   // nothing to pull and no dash worth taking: he keeps the line in
        };
        let aim_id = match mode { HookMode::Ally(id) => Some(id), HookMode::Enemy => target.map(|t| t.id), _ => None };
        self.charge = Some(Charge { mode, aim_id, pt: aim, t0: tick });
    }

    /// Reel a unit in on the line, through any wall: the game's own pull when the way is clear of terrain, otherwise
    /// moved by hand (and set down on free ground). Either way it's marked stv_reeled, which lets it through boat
    /// walls. Returns how long the pull lasts.
    fn reel(&mut self, sim: &mut StableSim<'_>, m: &Champ, all: &[Champ], id: usize, (speed, max_t): (usize, usize)) -> usize {
        let Some(u) = all.iter().find(|c| c.id == id) else { return 0 };
        let dist = (d2(u.x, u.y, m.x, m.y) as f64).sqrt() as usize;
        let ticks = (dist.saturating_sub(10_000) / speed).clamp(6, max_t);
        let q = walls::clip(u.x, u.y, m.x, m.y);
        if d2(q.0, q.1, m.x, m.y) <= sq(3_000) {
            sim.entity_pull(m.id, id, speed, ticks);
        } else {
            self.reels.retain(|r| r.0 != id);
            self.reels.push((id, speed, sim.tick() + ticks));
        }
        sim.entity_remove_buff(id, "stv_reeled");
        sim.add_buff(id, &timed("stv_reeled", ticks + 4));
        ticks
    }

    /// The fishing line: dots from Steve to (x, y), redrawn every 2 ticks.
    fn draw_line(sim: &mut StableSim<'_>, m: &Champ, x: i64, y: i64) {
        let d = ((x - m.x) as f64).hypot((y - m.y) as f64);
        let n = ((d / 7_000.0) as i64).clamp(1, 32);
        for k in 1..n {
            let (lx, ly) = (m.x + (x - m.x) * k / n, m.y - 3_000 + (y - m.y + 3_000) * k / n);
            fx(sim, &v(m, "line_dot"), m.id, lx, ly, 2);
        }
    }
}

impl StablePassive for Steve {
    fn clone_box(&self) -> Box<dyn StablePassive> {
        Box::new(self.clone())
    }
    fn on_spawn(&mut self, _sim: &mut StableSim<'_>, _player: usize, _entity: usize) {
        *self = Steve::default();
    }
    fn on_attack(&mut self, sim: &mut StableSim<'_>, _player: usize, _entity: usize, _target: usize, _damage: &mut usize) {
        self.my_swing = sim.tick();
    }
    fn on_dead(&mut self, _sim: &mut StableSim<'_>, _player: usize) {
        let tool = self.tool;
        *self = Steve::default();
        self.tool = tool;
    }
    fn on_update(&mut self, sim: &mut StableSim<'_>, _seed: u64, _player: usize, entity: usize) {
        let tick = sim.tick();
        let all = champions(sim);
        let Some(m) = all.iter().find(|c| c.id == entity).cloned() else { return };
        let enemies: Vec<&Champ> = all.iter().filter(|c| c.team != m.team).collect();

        // the tool he'll use next, as an icon over his head
        if tick % 10 == 0 {
            self.tool = self.choose_tool(sim, &m, &all, tick);
        }
        if self.shown_tool != Some(self.tool) || !m.has(&format!("stv_tool{}", self.tool)) {
            for k in 0..3 { sim.entity_remove_buff(entity, &format!("stv_tool{k}")); }
            sim.add_buff(entity, &BuffV1::named(&format!("stv_tool{}", self.tool)));
            self.shown_tool = Some(self.tool);
        }

        // ---- markers from the data file
        if m.has("stv_s1") && !m.has("stv_s1seen") {
            sim.add_buff(entity, &timed("stv_s1seen", 10));
            self.use_tool(sim, &m, &all, tick);
        }
        if m.has("stv_rod") && !m.has("stv_rodseen") && self.hook.is_none() && self.charge.is_none() {
            sim.add_buff(entity, &timed("stv_rodseen", 10));
            self.cast_rod(sim, &m, &all, tick);
        }
        // ---- the ult waits for a setup: while it's ready and there's something worth walling in, the key enemy is held
        //      for 2 ticks every ULT_BIND_EVERY so the AI (whose ult targets an enemy in CC) presses it
        //      When a teammate catches an enemy (the moment the AI presses the ult by itself), it's checked every 2 ticks
        //      so a pick is never missed.
        let any_caught = tick % 2 == 0 && enemies.iter().any(|e| caught(sim, e.id));
        if (tick % ULT_BIND_EVERY == 0 || any_caught) && self.ride.is_none() && self.jump.is_none() {
            let ready = (0..sim.player_count()).filter_map(|i| sim.player_at(i))
                .find(|p| p.champion().map_or(false, |c| c.id() == entity)).and_then(|p| p.cooldowns()).map_or(false, |c| c.3 == 0);
            // stv_ult_ok tells the data a setup exists; without it a cast (any other CC on an enemy makes the AI press
            // it) does nothing and the ult comes back in ~2 s instead of being wasted
            let plan = if ready && !m.stunned { plan_ult(sim, &m, &all, false) } else { None };
            sim.entity_remove_buff(entity, "stv_ult_ok");
            if let Some(p) = plan {
                sim.add_buff(entity, &timed("stv_ult_ok", ULT_BIND_EVERY + 10));
                if let (Some(k), true) = (p.key, tick % ULT_BIND_EVERY == 0 && !any_caught) {
                    sim.apply_cc(k, &CcV1::of_kind(CcKindV1::Bind, 2));
                }
            }
        }
        if m.has("stv_boat") && !m.has("stv_boatseen") && self.ride.is_none() {
            sim.add_buff(entity, &timed("stv_boatseen", 10));
            if let Some(plan) = plan_ult(sim, &m, &all, false).or_else(|| plan_ult(sim, &m, &all, true)) {
                let mut pts = plan.pts.clone();
                let s0 = pts[0];
                // to the start: a hop when it's close (or the way there is blocked), else the boat rides there first
                let lead = d2(s0.0, s0.1, m.x, m.y) > sq(HOP_MAX) && {
                    let q = walls::clip(m.x, m.y, s0.0, s0.1);
                    d2(q.0, q.1, s0.0, s0.1) <= sq(3_000)
                };
                if lead {
                    pts.insert(0, (m.x, m.y));
                } else if d2(s0.0, s0.1, m.x, m.y) > sq(2_000) {
                    fx(sim, &v(&m, "splash"), entity, m.x, m.y, 16);
                    sim.entity_set_pos(entity, s0.0.max(0) as u64, s0.1.max(0) as u64);
                }
                let cum = cum_len(&pts);
                let total = *cum.last().unwrap();
                let ticks_at = |d: f64| tick + (d / RIDE_SPEED as f64).ceil() as usize;
                let tend = ticks_at(total) + WALL_LIFE;
                let first = if lead { 1 } else { 0 };
                for i in first..pts.len() - 1 {
                    let (sa, mut sb) = (pts[i], pts[i + 1]);
                    let mut l = cum[i + 1] - cum[i];
                    if i == pts.len() - 2 && l > 16_000.0 {
                        // the last wall stops 8000 short of where the boat stops, so a rider ends up past its end
                        let (ux, uy) = norm((sb.0 - sa.0) as f64, (sb.1 - sa.1) as f64);
                        sb = (sb.0 - (ux * 8_000.0) as i64, sb.1 - (uy * 8_000.0) as i64);
                        l -= 8_000.0;
                    }
                    let (t0, tb) = (ticks_at(cum[i]), ticks_at(cum[i] + l));
                    sim.add_buff(entity, &timed(&format!("sbw:{}:{}:{}:{}:{t0}:{tb}:{tend}", sa.0, sa.1, sb.0, sb.1), tend - tick + 5));
                }
                // where he jumps off: the point of the wall path nearest the jump spot
                let jump_at = plan.jump_to.map(|j| {
                    let mut best = (f64::MAX, total);
                    let mut d = cum[first];
                    while d <= total {
                        let (p, _) = path_at(&pts, &cum, d);
                        let dd = d2(p.0, p.1, j.0, j.1) as f64;
                        if dd < best.0 { best = (dd, d); }
                        d += 2_000.0;
                    }
                    best.1
                });
                let mut b = timed("stv_riding", (total / RIDE_SPEED as f64) as usize + JUMP_T + 4);
                b.cc_immune = true;
                sim.entity_remove_buff(entity, "stv_riding");
                sim.add_buff(entity, &b);
                self.ride = Some(Ride { pts, cum, t0: tick, rider: true, jump_at, jump_to: plan.jump_to, home: (-plan.axis.0, -plan.axis.1), hit: Vec::new() });
                // the call: his team comes to use the wall, if going in is a winning situation
                if let Some((pt, follow)) = plan.rally {
                    if rally_good(sim, m.team, &all, pt, tick) {
                        let until = tick + (total / RIDE_SPEED as f64) as usize + WALL_LIFE;
                        for b in m.buffs.iter().filter(|b| b.name().starts_with("stv_rally:")) { sim.entity_remove_buff(entity, &b.name()); }
                        sim.add_buff(entity, &timed(&format!("stv_rally:{}:{}:{}:{until}", pt.0, pt.1, follow.map_or(-1, |f| f as i64)), until - tick + 2));
                        fx(sim, &v(&m, "warp"), entity, pt.0, pt.1, 14);
                    }
                }
            }
        }

        // ---- the call: re-checked every 30 ticks (dropped once going in isn't winning any more), and shown with a
        //      ping where his team is meant to go
        if tick % 30 == 0 {
            if let Some(b) = m.buffs.iter().find(|b| b.name().starts_with("stv_rally:")) {
                match rally_of(sim, &all, m.team, tick) {
                    Some(pt) if rally_good(sim, m.team, &all, pt, tick) => fx(sim, &v(&m, "warp"), entity, pt.0, pt.1, 14),
                    _ => { sim.entity_remove_buff(entity, &b.name()); }
                }
            }
        }

        // ---- the boat ride: Steve rides until he jumps off; the empty boat finishes the wall
        if let Some(mut r) = self.ride.take() {
            let total = *r.cum.last().unwrap_or(&0.0);
            let d = tick.saturating_sub(r.t0) as f64 * RIDE_SPEED as f64;
            let ((x, y), seg) = path_at(&r.pts, &r.cum, d);
            let dir_r = r.pts.get(seg + 1).map_or(true, |b| b.0 >= r.pts[seg].0);
            if tick % 2 == 0 {
                fx(sim, &v(&m, if dir_r { "boat_r" } else { "boat_l" }), entity, x, y, 2);
            }
            // the boat rams the enemies in its way (once each): damage and a knock out to the side they're on (or,
            // right on the line, to the far side from his team), clear of the wall rising behind it. V1 parrying the
            // boat breaks the whole wall (round 28).
            let mut broken = false;
            {
                let (sx, sy) = r.pts[seg];
                let (ex, ey) = r.pts.get(seg + 1).copied().unwrap_or((sx + 1, sy));
                let (ux, uy) = norm((ex - sx) as f64, (ey - sy) as f64);
                let (nx, ny) = (-uy, ux);
                let rammed: Vec<&&Champ> = enemies.iter().filter(|e| !r.hit.contains(&e.id) && d2(e.x, e.y, x, y) <= sq(BOAT_HIT_R)).collect();
                for e in rammed {
                    let sd = (e.x - x) as f64 * nx + (e.y - y) as f64 * ny;
                    let s = if sd.abs() >= 3_000.0 { sd.signum() } else { -(r.home.0 * nx + r.home.1 * ny).signum() };
                    let s = if s == 0.0 { 1.0 } else { s };
                    if crate::batch2::try_parry(sim, e, entity, BOAT_DMG.0 + m.max_hp * BOAT_DMG.1 / 100) { r.hit.push(e.id); broken = true; break; }
                    sim.deal_damage(entity, e.id, BOAT_DMG.0 + m.max_hp * BOAT_DMG.1 / 100, 0, AttackTypeV1::Skill);
                    let mut push = CcV1::of_kind(CcKindV1::ForceMove, BOAT_KNOCK.1);
                    push.dx = (nx * s * 1000.0) as i64;
                    push.dy = (ny * s * 1000.0) as i64;
                    push.speed = BOAT_KNOCK.0;
                    sim.apply_cc(e.id, &push);
                    fx(sim, &v(&m, "splash"), entity, e.x, e.y, 16);
                    r.hit.push(e.id);
                }
            }
            if r.rider {
                sim.entity_set_pos(entity, x.max(0) as u64, y.max(0) as u64);
                // jump off midway: a teammate about to die near the boat comes first, then the planned spot; when he's
                // low he jumps at once to his team's side of the wall
                let rescue = all.iter().filter(|a| a.team == m.team && a.id != m.id && pct(a) <= 35 && d2(a.x, a.y, x, y) <= sq(JUMP_MAX)
                    && enemies.iter().any(|e| d2(e.x, e.y, a.x, a.y) <= sq(25_000))).min_by_key(|a| (pct(a), a.id))
                    .map(|a| {
                        let e = enemies.iter().min_by_key(|e| d2(e.x, e.y, a.x, a.y)).map_or((a.x, a.y), |e| (e.x, e.y));
                        let (dx, dy) = norm((a.x - e.0) as f64, (a.y - e.1) as f64);
                        (a.x + (dx * 6_000.0) as i64, a.y + (dy * 6_000.0) as i64)
                    });
                let planned = r.jump_at.filter(|&ja| d >= ja).and(r.jump_to);
                let to = if pct(&m) > LOW_PCT { rescue.or(planned) } else {
                    Some(((x as f64 + r.home.0 * 25_000.0) as i64, (y as f64 + r.home.1 * 25_000.0) as i64))
                };
                if let Some(to) = to {
                    let (dx, dy) = norm((to.0 - x) as f64, (to.1 - y) as f64);
                    let dist = ((d2(to.0, to.1, x, y) as f64).sqrt()).min(JUMP_MAX as f64);
                    let to = walls::pull_back(x, y, (x as f64 + dx * dist) as i64, (y as f64 + dy * dist) as i64);
                    fx(sim, &v(&m, "splash"), entity, x, y, 16);
                    self.jump = Some((x, y, to.0, to.1, tick));
                    r.rider = false;
                } else if d >= total {
                    // the boat may stop on terrain: step back along the wall onto free ground
                    if walls::wall_at(x, y) {
                        let (fx_, fy_) = walls::pull_back(r.pts[0].0, r.pts[0].1, x, y);
                        sim.entity_set_pos(entity, fx_.max(0) as u64, fy_.max(0) as u64);
                    }
                    sim.entity_remove_buff(entity, "stv_riding");
                }
            }
            if broken {
                // the wall shatters: every block built so far crumbles, the boat sinks, Steve lands on free ground
                for b in m.buffs.iter().filter(|b| b.name().starts_with("sbw:")) {
                    let v_: Vec<i64> = b.name()["sbw:".len()..].split(':').filter_map(|t| t.parse().ok()).collect();
                    if v_.len() >= 7 {
                        let w = Wall { ax: v_[0], ay: v_[1], bx: v_[2], by: v_[3], t0: v_[4].max(0) as usize, tb: v_[5].max(0) as usize, tend: v_[6].max(0) as usize };
                        if tick >= w.t0 {
                            let (ex, ey) = w.built_end(tick);
                            let n = ((((ex - w.ax) as f64).hypot((ey - w.ay) as f64)) / 15_200.0) as i64;
                            for k in 0..=n {
                                let f = if n == 0 { 0.0 } else { k as f64 / n as f64 };
                                fx(sim, &v(&m, "wall_rise"), entity, w.ax + ((ex - w.ax) as f64 * f) as i64, w.ay + ((ey - w.ay) as f64 * f) as i64, 12);
                            }
                        }
                    }
                    sim.entity_remove_buff(entity, &b.name());
                }
                fx(sim, &v(&m, "splash"), entity, x, y, 16);
                if r.rider {
                    let (fx_, fy_) = if walls::wall_at(x, y) { walls::pull_back(r.pts[0].0, r.pts[0].1, x, y) } else { (x, y) };
                    sim.entity_set_pos(entity, fx_.max(0) as u64, fy_.max(0) as u64);
                    sim.entity_remove_buff(entity, "stv_riding");
                }
            } else if d < total { self.ride = Some(r); }
        }
        // ---- jumping off the boat (still exempt from the walls and from CC until he lands)
        if let Some((fx_, fy_, tx, ty, t0)) = self.jump {
            let k = (tick.saturating_sub(t0) as f64 / JUMP_T as f64).min(1.0);
            let (x, y) = (fx_ + ((tx - fx_) as f64 * k) as i64, fy_ + ((ty - fy_) as f64 * k) as i64);
            sim.entity_set_pos(entity, x.max(0) as u64, y.max(0) as u64);
            if k >= 1.0 {
                self.jump = None;
                fx(sim, &v(&m, "splash"), entity, tx, ty, 16);
                sim.entity_remove_buff(entity, "stv_riding");
            }
        }
        // the walls and their ice: blocks appear as the boat passes, and stay until the walls fall
        let segs: Vec<Wall> = m.buffs.iter().filter_map(|b| b.name().strip_prefix("sbw:").map(|r| r.split(':').filter_map(|s| s.parse::<i64>().ok()).collect::<Vec<_>>()))
            .filter(|v_| v_.len() >= 7)
            .map(|v_| Wall { ax: v_[0], ay: v_[1], bx: v_[2], by: v_[3], t0: v_[4].max(0) as usize, tb: v_[5].max(0) as usize, tend: v_[6].max(0) as usize })
            .collect();
        // every block of the built part is (re)placed every 30 ticks (each lasts 0.5 s), so the wall stays on the map
        // until it falls; new blocks rise as the boat passes; when it falls every block crumbles
        const STEP: f64 = 15_200.0;  // one block (16 px; round 26: was 7600)
        for w in segs.iter().filter(|w| tick >= w.t0 && tick <= w.tend) {
            let (ex, ey) = w.built_end(tick);
            let len = ((ex - w.ax) as f64).hypot((ey - w.ay) as f64);
            let total = ((w.bx - w.ax) as f64).hypot((w.by - w.ay) as f64).max(1.0);
            let block = |b: i64| { let f = (b as f64 * STEP) / total; (w.ax + ((w.bx - w.ax) as f64 * f) as i64, w.ay + ((w.by - w.ay) as f64 * f) as i64) };
            let built_blocks = (len / STEP) as i64;
            if tick == w.tend {
                for b in 0..=built_blocks { let (bx, by) = block(b); fx(sim, &v(&m, "wall_rise"), entity, bx, by, 12); }
                continue;
            }
            let prev_blocks = if tick > w.t0 { ((total * ((tick - 1 - w.t0) as f64 / (w.tb - w.t0).max(1) as f64).min(1.0)) / STEP) as i64 } else { -1 };
            let refresh = tick % 30 == 0 && w.tend - tick >= 12;
            for b in 0..=built_blocks {
                let new = b > prev_blocks && tick <= w.tb;
                if !(new || refresh) { continue; }
                let (bx, by) = block(b);
                fx(sim, &v(&m, "wall_block"), entity, bx, by, 30);
                fx(sim, &v(&m, "ice"), entity, bx, by, 30);
                if new { fx(sim, &v(&m, "wall_rise"), entity, bx, by, 12); }
            }
        }
        // allies on the ice trail are faster
        if tick % 10 == 0 && !segs.is_empty() {
            for a in all.iter().filter(|c| c.team == m.team) {
                let on_ice = segs.iter().filter(|w| tick >= w.t0 && tick < w.tend).any(|w| {
                    let (ex, ey) = w.built_end(tick);
                    let (sd, along) = side(w, a.x, a.y);
                    sd.abs() <= 16_000.0 && along >= 0.0 && along <= ((ex - w.ax) as f64).hypot((ey - w.ay) as f64)
                });
                if on_ice {
                    let mut b = timed("stv_ice", 15);
                    b.move_speed_mult = ICE_MS;
                    sim.entity_remove_buff(a.id, "stv_ice");
                    sim.add_buff(a.id, &b);
                }
            }
        }

        // ---- the pearl in flight
        if let Some((_, _, tx, ty, t0)) = self.pearl {
            if tick >= t0 + PEARL_T {
                self.pearl = None;
                if !m.stunned {
                    fx(sim, &v(&m, "warp"), entity, m.x, m.y, 14);
                    let (lx, ly) = walls::pull_back(m.x, m.y, tx, ty);
                    sim.entity_set_pos(entity, lx.max(0) as u64, ly.max(0) as u64);
                    fx(sim, &v(&m, "warp"), entity, lx, ly, 14);
                    // a hooked enemy comes along: the pull starts again toward where he landed
                    if let Some((tid, until)) = self.tether {
                        if tick < until + 10 && all.iter().any(|c| c.id == tid) {
                            let landed = Champ { x: lx, y: ly, ..m.clone() };
                            let t = self.reel(sim, &landed, &all, tid, PULL);
                            self.tether = Some((tid, tick + t));
                        }
                    }
                }
            }
        }

        // ---- TNT: lies unlit until a teammate (or Steve) swings right next to it, then the fuse ring, then boom
        let mut swings: Vec<(i64, i64)> = Vec::new();
        if self.my_swing + 1 >= tick && self.my_swing > 0 { swings.push((m.x, m.y)); }
        if !self.tnts.is_empty() {
            let mut cds: Vec<(usize, usize)> = Vec::new();
            for i in 0..sim.player_count() {
                let Some(p) = sim.player_at(i) else { continue };
                let Some(c) = p.champion() else { continue };
                let cid = c.id();
                if cid == entity { continue; }
                let Some(a) = all.iter().find(|u| u.id == cid && u.team == m.team) else { continue };
                let cd = p.cooldowns().map_or(0, |c| c.0);
                if let Some(&(_, prev)) = self.atk_cd.iter().find(|(id, _)| *id == cid) {
                    if cd > prev + 10 { swings.push((a.x, a.y)); }
                }
                cds.push((cid, cd));
            }
            self.atk_cd = cds;
        }
        let mut booms: Vec<(i64, i64)> = Vec::new();
        for t in self.tnts.iter_mut() {
            let landed = t.land != usize::MAX && tick >= t.land;
            if landed && t.det == usize::MAX {
                let near = |r: i64| enemies.iter().any(|e| d2(e.x, e.y, t.x, t.y) <= sq(r));
                if !t.checked {
                    // as it lands: lit at once if anyone is in its reach, otherwise it lies as a mine
                    t.checked = true;
                    if near(TNT_R) { t.det = tick + TNT_FUSE; t.expire = usize::MAX; } else { t.expire = tick + TNT_MINE_LIFE; }
                } else if near(TNT_TRIGGER_R) || swings.iter().any(|&(x, y)| d2(x, y, t.x, t.y) <= sq(TNT_TOUCH_R)) {
                    // the mine: an enemy steps close (or a swing right by it) sets it off
                    t.det = tick + TNT_MINE_FUSE;
                    t.expire = usize::MAX;
                    fx(sim, &v(&m, "tnt_ring"), entity, t.x, t.y, TNT_MINE_FUSE as u64);
                } else if (tick - t.land) % 12 == 0 {
                    fx(sim, &v(&m, "tnt_idle"), entity, t.x, t.y, 12);
                }
            }
            if landed && t.det != usize::MAX && tick < t.det {
                // a swing right next to it sets it off at once
                if swings.iter().any(|&(x, y)| d2(x, y, t.x, t.y) <= sq(TNT_TOUCH_R)) {
                    t.det = t.det.min(tick + TNT_QUICK);
                }
                // the block and its red ring, shown again every 12 ticks (their animations are short) until it blows
                if (tick - t.land) % 12 == 0 {
                    fx(sim, &v(&m, "tnt"), entity, t.x, t.y, 12);
                    fx(sim, &v(&m, "tnt_ring"), entity, t.x, t.y, 12);
                }
            }
            if tick >= t.det { booms.push((t.x, t.y)); }
            else if t.det == usize::MAX && tick >= t.expire {
                fx(sim, &v(&m, "wall_rise"), entity, t.x, t.y, 12);
            }
        }
        self.tnts.retain(|t| tick < t.det && tick < t.expire);
        for (x, y) in booms { Self::tnt_boom(sim, &m, &all, x, y); }

        // ---- charging the rod: slowed, a bar over his head; he lets go as soon as the charge reaches what he's aiming
        //      at (a little past it), or at full charge
        if let Some(c) = self.charge.clone() {
            let el = tick.saturating_sub(c.t0);
            let pt = c.aim_id.and_then(|id| all.iter().find(|u| u.id == id)).map_or(c.pt, |u| (u.x, u.y));
            let pt = match c.mode { HookMode::Tnt(i) => self.tnts.get(i).map_or(pt, |t| (t.x, t.y)), _ => pt };
            let need = ((d2(pt.0, pt.1, m.x, m.y) as f64).sqrt() as i64 + CHARGE_MARGIN).clamp(HOOK_MIN, HOOK_RANGE);
            let reach = HOOK_MIN + (HOOK_RANGE - HOOK_MIN) * (el.min(CHARGE_MAX) as i64) / CHARGE_MAX as i64;
            let stage = ((reach - HOOK_MIN) * 5 / (HOOK_RANGE - HOOK_MIN)) as usize;
            for k in 0..6 { if k != stage { sim.entity_remove_buff(entity, &format!("stv_ch{k}")); } }
            if m.stunned {
                self.charge = None;
            } else if reach >= need || el >= CHARGE_MAX {
                self.charge = None;
                let (dx, dy) = norm((pt.0 - m.x) as f64, (pt.1 - m.y) as f64);
                self.hook = Some(Hook { x: m.x, y: m.y, dx, dy, travelled: 0, range: reach, mode: c.mode });
            } else {
                if !m.has(&format!("stv_ch{stage}")) { sim.add_buff(entity, &BuffV1::named(&format!("stv_ch{stage}"))); }
                if !m.has("stv_charging") {
                    let mut b = timed("stv_charging", CHARGE_MAX + 5);
                    b.move_speed_mult = CHARGE_SLOW;
                    sim.add_buff(entity, &b);
                }
            }
            if self.charge.is_none() {
                for k in 0..6 { sim.entity_remove_buff(entity, &format!("stv_ch{k}")); }
                sim.entity_remove_buff(entity, "stv_charging");
            }
        }

        // ---- the hook in flight
        if let Some(mut h) = self.hook.take() {
            h.x += (h.dx * HOOK_SPEED as f64) as i64;
            h.y += (h.dy * HOOK_SPEED as f64) as i64;
            h.travelled += HOOK_SPEED;
            if tick % 2 == 0 {
                fx(sim, &v(&m, "hook"), entity, h.x, h.y - 3_000, 2);
                Self::draw_line(sim, &m, h.x, h.y);
            }
            let mut done = h.travelled >= h.range;
            // round 32: the hook doesn't go through walls. Whatever it was cast at, if it reaches terrain or a boat wall
            // first it bites there and Steve dashes to it (a grapple)
            let in_boat_wall = walls_up(&all, tick).iter().any(|w| { let (sd, along) = side(w, h.x, h.y); let (ex, ey) = w.built_end(tick);
                sd.abs() < WALL_HALF as f64 && along >= -(WALL_HALF as f64) && along <= ((ex - w.ax) as f64).hypot((ey - w.ay) as f64) + WALL_HALF as f64 });
            if !done && (walls::wall_at(h.x, h.y) || in_boat_wall) {
                let back = if in_boat_wall { WALL_HALF as f64 + 4_000.0 } else { 6_000.0 };
                let (gx, gy) = (h.x - (h.dx * back) as i64, h.y - (h.dy * back) as i64);
                let (gx, gy) = walls::pull_back(m.x, m.y, gx, gy);
                self.grapple = Some((gx, gy, tick + 45));
                fx(sim, &v(&m, "hook"), entity, h.x, h.y - 3_000, 10);
                done = true;
            } else { match h.mode {
                HookMode::Tnt(i) => {
                    if let Some(t) = self.tnts.get(i).cloned() {
                        if d2(t.x, t.y, h.x, h.y) <= sq(HOOK_HIT_R) {
                            // yank it and fling it at the nearest enemy; it blows up as it lands
                            let to = enemies.iter().min_by_key(|e| d2(e.x, e.y, m.x, m.y)).map_or((t.x, t.y), |e| Self::tnt_spot(&m, &all, e));
                            fly(sim, &v(&m, "tnt_fly"), &m, t.x, t.y, to.0, to.1, TNT_FLING_T);
                            self.tnts[i] = Tnt { x: to.0, y: to.1, land: usize::MAX, det: tick + TNT_FLING_T + 2, expire: usize::MAX, from: t.from, checked: true };
                            done = true;
                        }
                    } else { done = true; }
                }
                HookMode::Ally(id) => {
                    if let Some(a) = all.iter().find(|c| c.id == id) {
                        let (dx, dy) = norm((a.x - h.x) as f64, (a.y - h.y) as f64);
                        h.dx = dx; h.dy = dy;
                        if d2(a.x, a.y, h.x, h.y) <= sq(HOOK_HIT_R) {
                            self.reel(sim, &m, &all, id, RESCUE);
                            self.ally_hooked = Some((id, tick));
                            done = true;
                        }
                    } else { done = true; }
                }
                HookMode::Wall => {}   // (the wall bite is handled above, for every mode)
                HookMode::Enemy => {
                    if let Some(e) = enemies.iter().filter(|e| d2(e.x, e.y, h.x, h.y) <= sq(HOOK_HIT_R)).min_by_key(|e| (d2(e.x, e.y, h.x, h.y), e.id)) {
                        if !crate::batch2::try_parry(sim, e, entity, m.max_hp * HOOK_DMG.1 / 100 + HOOK_DMG.0) {
                            sim.deal_damage(entity, e.id, HOOK_DMG.0 + m.max_hp * HOOK_DMG.1 / 100, 0, AttackTypeV1::Skill);
                            let eid = e.id;
                            let t = self.reel(sim, &m, &all, eid, PULL);
                            self.tether = Some((eid, tick + t));
                            sim.play_view_effect(&v(&m, "hooked"), entity, &InputTargetV1::target(eid), 0, 0, t as u64);
                        }
                        done = true;
                    }
                }
            } }
            if !done { self.hook = Some(h); }
        }
        // the line stays on a hooked enemy while it's being pulled
        if let Some((tid, until)) = self.tether {
            if tick >= until { self.tether = None; }
            else if tick % 2 == 0 {
                if let Some(e) = enemies.iter().find(|e| e.id == tid) { Self::draw_line(sim, &m, e.x, e.y); }
            }
        }
        self.seen = enemies.iter().map(|e| (e.id, e.x, e.y)).collect();
        // ---- reeling in through terrain by hand
        let mut reels = std::mem::take(&mut self.reels);
        reels.retain(|&(id, speed, until)| {
            let Some(u) = all.iter().find(|c| c.id == id) else { return false };
            let d = (d2(u.x, u.y, m.x, m.y) as f64).sqrt();
            let finish = tick >= until || d <= 10_000.0;
            let (nx, ny) = if finish { (u.x, u.y) } else {
                let (dx, dy) = norm((m.x - u.x) as f64, (m.y - u.y) as f64);
                let step = (speed as f64).min(d - 10_000.0);
                (u.x + (dx * step) as i64, u.y + (dy * step) as i64)
            };
            // set down on free ground at the end (walk back toward Steve out of any wall)
            let (nx, ny) = if finish && walls::wall_at(nx, ny) { walls::pull_back(m.x, m.y, nx, ny) } else { (nx, ny) };
            sim.entity_set_pos(id, nx.max(0) as u64, ny.max(0) as u64);
            !finish
        });
        self.reels = reels;
        // ---- grapple: reel himself to the wall
        if let Some((gx, gy, until)) = self.grapple {
            let d = ((gx - m.x) as f64).hypot((gy - m.y) as f64);
            if tick >= until || d <= GRAPPLE_SPEED as f64 || m.stunned {
                self.grapple = None;
            } else {
                let (dx, dy) = norm((gx - m.x) as f64, (gy - m.y) as f64);
                sim.entity_set_pos(entity, (m.x + (dx * GRAPPLE_SPEED as f64) as i64).max(0) as u64, (m.y + (dy * GRAPPLE_SPEED as f64) as i64).max(0) as u64);
                if tick % 2 == 0 { Self::draw_line(sim, &m, gx, gy); }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn champ(id: usize, team: usize, x: i64, y: i64) -> Champ {
        Champ { id, team, x, y, buffs: vec![], stunned: false, pushed: false, hp: 100, max_hp: 100, attack: 10, name: String::new() }
    }
    #[test]
    fn straight_wall_and_detour() {
        let m = champ(1, 0, 300_000, 300_000);
        let c = (460_000, 480_000);
        let axis = (1.0, 0.0);   // blocks east-west traffic: the wall runs north-south
        let (s, e, sealed) = lay_line(&m, c, c, axis, 120_000.0).unwrap();
        println!("{s:?} {e:?} sealed {sealed}");
        assert_eq!(s.0, c.0); assert_eq!(e.0, c.0);
        assert!(((s.1 - e.1).abs() - WALL_LEN).abs() <= 1);
        let walls_ = vec![Wall { ax: s.0, ay: s.1, bx: e.0, by: e.1, t0: 0, tb: 0, tend: 1000 }];
        assert!(crosses(&walls_[0], 10, 400_000, 480_000, 520_000, 480_000));
        let d = detour_all(&walls_, 10, 400_000, 480_000, 520_000, 480_000).unwrap();
        println!("detour {d:?}");
        assert!(!crosses(&walls_[0], 10, 400_000, 480_000, d.0, d.1));
        // a straight path from the detour point onward is clear
        assert!(!crosses(&walls_[0], 10, d.0, d.1, 520_000, 480_000));
        let cum = cum_len(&[s, e]);
        let (mid, _) = path_at(&[s, e], &cum, cum[1] / 2.0);
        assert!((mid.1 - c.1).abs() <= 1);
        // terrain cuts the south side short: the wall slides north and keeps its full length
        let mut cells = vec![false; (walls::N * walls::N) as usize];
        cells[(16 * walls::N + 14) as usize] = true;   // cell x 14, y 16 (y 512000..544000)
        walls::set(cells);
        let (s2, e2, sealed2) = lay_line(&m, c, c, axis, 120_000.0).unwrap();
        println!("slid: {s2:?} {e2:?} sealed {sealed2}");
        let len2 = ((s2.1 - e2.1).abs()) as i64;
        assert!((len2 - WALL_LEN).abs() <= 3_000, "full length kept: {len2}");
        assert!(s2.1.max(e2.1) < 512_000 && s2.1.max(e2.1) > 505_000, "ends at the terrain");
        assert_eq!(sealed2, 1);
        walls::set(vec![false; (walls::N * walls::N) as usize]);
    }
}
