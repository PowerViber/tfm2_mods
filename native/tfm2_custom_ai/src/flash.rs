//! Flash (rounds 58-59, Rian): every champion in the game, base and mod, has a summoner-style Flash on a 120 s cooldown,
//! kept for the important moments. A blink of FLASH_R in any direction, over walls (like a teleport: it also crosses
//! Steve's boat wall). The match hook decides for each champion every CHECK_EVERY ticks, first match wins:
//!   run      at 20% HP or less with a visible enemy on them and the fight lost around them (or at 10% or less):
//!            away from the enemies, toward home and teammates, never under an enemy tower;
//!   evade    a shot that matters about to land: an ult shot, Hollow Purple, Paranoia, or any skill shot when they're at
//!            30% HP or less. Sideways out of its line. Ordinary hits never;
//!   help     a teammate at 35% HP or less with an enemy on them, cut off by terrain, within one blink, and they're
//!            healthy (50%+): over the wall to the teammate;
//!   catch    healthy (40%+), a visible enemy at 15% HP or less and within two of their hits, running away and getting
//!            out of reach: in next to them, unless that lands in a crowd or under their tower;
//!   combo    their ult is ready, 3+ enemies bunched just out of reach, their team close enough to follow, they're at
//!            60%+: into the group so the ult lands.
//! Not while stunned, sealed in a domain, in stopped time or riding Steve's boat.
//! The cooldown lives in native state keyed by the match (match_key: round 59, the old key was not stable, so the
//! cooldown never held) and champion, so it survives death like a real one.

use super::*;
use std::collections::HashMap;
use std::sync::Mutex;

pub const FLASH_CD: usize = 7_200;
const FLASH_R: i64 = 40_000;
const CHECK_EVERY: usize = 4;
const EVADE_T: f64 = 18.0;
/// A projectile counts as a skill shot when its caster cast a skill or ult this recently.
const CAST_FRESH: usize = 90;
/// gojo_purple_strain's full length (presets.js purpleStrainTicks): a strain this fresh means Purple is in the air.
const PURPLE_STRAIN: usize = 1_260;
/// The blink effect, drawn from the Valorant folder's sheet (name registered on Omen's data).
const FX: &str = "tfm2_valorant_omen_flash";
/// Each team's nexus (the map document): where "home" is.
const HOME: [(i64, i64); 2] = [(96_000, 864_000), (864_000, 96_000)];
const TOWER_R: i64 = 85_000;

#[derive(Default)]
struct MatchState {
    last_tick: usize,
    ready: HashMap<usize, usize>,               // champion id → tick its flash is back
    seen: HashMap<usize, (i64, i64)>,           // champion positions at the last check
    cds: HashMap<usize, (usize, usize, usize)>, // champion id → skill, skill2, ult cooldowns at the last check
    cast: HashMap<usize, (usize, bool)>,        // champion id → tick of the last skill / ult cast, was it the ult
    shots: Vec<(usize, i64, i64)>,              // projectiles at the last check: caster, x, y
}

static STATE: Mutex<Option<HashMap<u64, MatchState>>> = Mutex::new(None);

fn pct(c: &Champ) -> usize {
    if c.max_hp == 0 { 100 } else { c.hp * 100 / c.max_hp }
}

fn dist(ax: i64, ay: i64, bx: i64, by: i64) -> f64 {
    ((ax - bx) as f64).hypot((ay - by) as f64)
}

fn blocked(c: &Champ) -> bool {
    c.stunned || c.buffs.iter().any(|b| {
        let n = b.name();
        n.starts_with("void_trapped") || n.starts_with("tsf") || n == "stv_riding"
    })
}

fn enemy_tower_near(sim: &StableSim<'_>, team: usize, p: (i64, i64)) -> bool {
    (0..sim.tower_count()).filter_map(|i| sim.get_entity(sim.tower_id_at(i)))
        .any(|t| { let (x, y) = t.pos(); t.is_alive() && t.team() != team && d2(x as i64, y as i64, p.0, p.1) <= sq(TOWER_R) })
}

fn free(p: (i64, i64)) -> bool {
    p.0 > 8_000 && p.1 > 8_000 && p.0 < 952_000 && p.1 < 952_000 && !walls::wall_at(p.0, p.1)
}

fn ring(c: (i64, i64), r: i64, n: usize) -> Vec<(i64, i64)> {
    (0..n).map(|i| {
        let a = i as f64 * std::f64::consts::TAU / n as f64;
        (c.0 + (a.cos() * r as f64) as i64, c.1 + (a.sin() * r as f64) as i64)
    }).filter(|p| free(*p)).collect()
}

/// A landing spot `back` short of `target` on the line from `m`, within one blink, on free ground.
fn toward(m: &Champ, target: (i64, i64), back: i64) -> Option<(i64, i64)> {
    let d = dist(m.x, m.y, target.0, target.1).max(1.0);
    let (ux, uy) = ((target.0 - m.x) as f64 / d, (target.1 - m.y) as f64 / d);
    let go = (d - back as f64).clamp(0.0, FLASH_R as f64);
    let p = (m.x + (ux * go) as i64, m.y + (uy * go) as i64);
    free(p).then_some(p)
}

struct Ctx<'s> {
    seen: &'s HashMap<usize, (i64, i64)>,
    cast: &'s HashMap<usize, (usize, bool)>,
    cds: &'s HashMap<usize, (usize, usize, usize)>,
    shots: &'s [(usize, i64, i64, f64, f64)], // caster, x, y, vx, vy (per tick)
    tick: usize,
}

/// Where this champion should flash right now (and why, for the log), or None.
fn plan(sim: &StableSim<'_>, all: &[Champ], m: &Champ, cx: &Ctx<'_>) -> Option<(i64, i64)> {
    let foes: Vec<&Champ> = all.iter().filter(|c| c.team != m.team && sim.is_visible(m.team, c.id)).collect();
    if foes.is_empty() { return None; }
    let foes_at = |p: (i64, i64), r: i64| foes.iter().filter(|c| d2(c.x, c.y, p.0, p.1) <= sq(r)).count();
    let mates_at = |p: (i64, i64), r: i64| all.iter().filter(|c| c.team == m.team && d2(c.x, c.y, p.0, p.1) <= sq(r)).count();
    let home = HOME[m.team.min(1)];

    // run
    let on_me = foes.iter().any(|e| d2(e.x, e.y, m.x, m.y) <= sq(30_000));
    if on_me && (pct(m) <= 10 || (pct(m) <= 20 && foes_at((m.x, m.y), 45_000) >= mates_at((m.x, m.y), 45_000))) {
        let close: Vec<&&Champ> = foes.iter().filter(|e| d2(e.x, e.y, m.x, m.y) <= sq(60_000)).collect();
        let n = close.len().max(1) as i64;
        let (cx0, cy0) = (close.iter().map(|e| e.x).sum::<i64>() / n, close.iter().map(|e| e.y).sum::<i64>() / n);
        let now = dist(m.x, m.y, cx0, cy0);
        let best = ring((m.x, m.y), FLASH_R, 16).into_iter()
            .filter(|p| dist(p.0, p.1, cx0, cy0) > now + 20_000.0 && !enemy_tower_near(sim, m.team, *p))
            .max_by_key(|p| {
                let away = dist(p.0, p.1, cx0, cy0) as i64 / 1_000;
                let homeward = (dist(m.x, m.y, home.0, home.1) - dist(p.0, p.1, home.0, home.1)) as i64 / 2_000;
                (away + homeward + mates_at(*p, 45_000) as i64 * 10 - foes_at(*p, 40_000) as i64 * 30, -p.0, -p.1)
            });
        if best.is_some() { return best; }
    }

    // evade a skill shot about to land
    for &(caster, x, y, vx, vy) in cx.shots {
        let Some(c) = all.iter().find(|c| c.id == caster) else { continue };
        if c.team == m.team { continue; }
        // only the shots that matter: an ult, Hollow Purple, Paranoia, or any skill shot when low
        let purple = c.buffs.iter().any(|b| b.name() == "gojo_purple_strain" && b.duration_tick + 240 >= PURPLE_STRAIN);
        let paranoia = c.has("omn_para_fly");
        let fresh = cx.cast.get(&caster).filter(|(t, _)| cx.tick <= t + CAST_FRESH);
        let ult = fresh.map_or(false, |f| f.1);
        if !(purple || paranoia || ult || (fresh.is_some() && pct(m) <= 30)) { continue; }
        let sp = vx.hypot(vy);
        if sp < 200.0 { continue; }
        let (rx, ry) = ((m.x - x) as f64, (m.y - y) as f64);
        let along = (rx * vx + ry * vy) / sp;          // how far ahead of the shot he stands
        let off = ((rx * vy - ry * vx) / sp).abs();     // how far off its line
        if along <= 0.0 || along / sp > EVADE_T || off > 20_000.0 { continue; }
        // sideways, the side he's already on, away from the line
        let (nx, ny) = (-vy / sp, vx / sp);
        let side = if rx * nx + ry * ny >= 0.0 { 1.0 } else { -1.0 };
        for k in [1.0f64, 0.7] {
            let p = (m.x + (nx * side * FLASH_R as f64 * k) as i64, m.y + (ny * side * FLASH_R as f64 * k) as i64);
            if free(p) && !enemy_tower_near(sim, m.team, p) && foes_at(p, 30_000) <= foes_at((m.x, m.y), 30_000) { return Some(p); }
        }
    }

    // help a teammate cut off by a wall
    if pct(m) >= 50 {
        for a in all.iter().filter(|a| a.team == m.team && a.id != m.id && pct(a) <= 35 && foes_at((a.x, a.y), 30_000) > 0) {
            let d = dist(m.x, m.y, a.x, a.y);
            if !(25_000.0..=(FLASH_R + 25_000) as f64).contains(&d) { continue; }
            if walls::clip(m.x, m.y, a.x, a.y) == (a.x, a.y) { continue; } // nothing in the way: just walk
            if let Some(p) = toward(m, (a.x, a.y), 10_000) {
                if dist(p.0, p.1, a.x, a.y) <= 25_000.0 && !enemy_tower_near(sim, m.team, p) { return Some(p); }
            }
        }
    }

    // catch a low enemy running away
    if pct(m) >= 40 {
        if let Some(&(mx0, my0)) = cx.seen.get(&m.id) {
            for e in foes.iter().filter(|e| pct(e) <= 15 && e.hp <= m.attack * 2) {
                let d = d2(e.x, e.y, m.x, m.y);
                if d < sq(30_000) || d > sq(FLASH_R + 40_000) { continue; }
                let Some(&(px, py)) = cx.seen.get(&e.id) else { continue };
                if d <= d2(px, py, mx0, my0) { continue; }
                let Some(p) = toward(m, (e.x, e.y), 15_000) else { continue };
                let crowd = foes.iter().filter(|c| c.id != e.id && d2(c.x, c.y, p.0, p.1) <= sq(45_000)).count();
                if dist(p.0, p.1, e.x, e.y) <= 28_000.0 && crowd <= 1 && (!enemy_tower_near(sim, m.team, p) || pct(e) <= 8) { return Some(p); }
            }
        }
    }

    // combo: the ult is ready and a group is bunched just out of reach
    if pct(m) >= 60 && cx.cds.get(&m.id).map_or(false, |c| c.2 == 0) {
        for e in &foes {
            let group: Vec<&&Champ> = foes.iter().filter(|o| d2(o.x, o.y, e.x, e.y) <= sq(25_000)).collect();
            if group.len() < 3 { continue; }
            let n = group.len() as i64;
            let g = (group.iter().map(|o| o.x).sum::<i64>() / n, group.iter().map(|o| o.y).sum::<i64>() / n);
            let d = dist(m.x, m.y, g.0, g.1);
            if !(30_000.0..=75_000.0).contains(&d) { continue; }
            if mates_at(g, 90_000) < group.len().saturating_sub(1) { continue; }
            if let Some(p) = toward(m, g, 12_000) {
                if !enemy_tower_near(sim, m.team, p) { return Some(p); }
            }
        }
    }

    None
}

fn show(sim: &mut StableSim<'_>, all: &[Champ], caster: usize, p: (i64, i64)) {
    let at = InputTargetV1::pos(p.0.max(0) as u64, p.1.max(0) as u64);
    if !sim.play_view_effect(FX, caster, &at, 0, 0, 20) {
        // in case effects must come from a champion whose data registers them
        if let Some(o) = all.iter().find(|c| c.name.ends_with("_omen")) { sim.play_view_effect(FX, o.id, &at, 0, 0, 20); }
    }
}

pub fn run(sim: &mut StableSim<'_>, all: &[Champ], tick: usize) {
    if tick % CHECK_EVERY != 0 { return; }
    let key = match_key(sim);
    // cooldowns and projectiles now
    let mut cds_now: HashMap<usize, (usize, usize, usize)> = HashMap::new();
    for i in 0..sim.player_count() {
        let Some(p) = sim.player_at(i) else { continue };
        let Some(c) = p.champion() else { continue };
        if let Some((_, s1, s2, u)) = p.cooldowns() { cds_now.insert(c.id(), (s1, s2, u)); }
    }
    let shots_now: Vec<(usize, i64, i64)> = (0..sim.projectile_count()).filter_map(|i| sim.projectile_at(i))
        .filter(|p| !p.is_end && all.iter().any(|c| c.id == p.caster_id)).map(|p| (p.caster_id, p.x as i64, p.y as i64)).collect();
    let mut moves: Vec<(usize, (i64, i64), (i64, i64))> = Vec::new();
    {
        let mut guard = match STATE.lock() { Ok(g) => g, Err(_) => return };
        let maps = guard.get_or_insert_with(HashMap::new);
        if maps.len() > 32 { maps.clear(); }
        let st = maps.entry(key).or_insert_with(|| MatchState { last_tick: tick, ..MatchState::default() });
        if tick < st.last_tick { *st = MatchState { last_tick: tick, ..MatchState::default() }; }
        st.last_tick = tick;
        // a cooldown that jumped up = a cast just now
        for (&id, &(s1, s2, u)) in &cds_now {
            if let Some(&(a, b, c)) = st.cds.get(&id) {
                if u > c + CHECK_EVERY { st.cast.insert(id, (tick, true)); }
                else if s1 > a + CHECK_EVERY || s2 > b + CHECK_EVERY { st.cast.insert(id, (tick, false)); }
            }
        }
        // shot velocities: each projectile matched to the nearest one of the same caster last check
        let shots: Vec<(usize, i64, i64, f64, f64)> = shots_now.iter().filter_map(|&(c, x, y)| {
            st.shots.iter().filter(|s| s.0 == c).min_by_key(|s| d2(s.1, s.2, x, y))
                .filter(|s| d2(s.1, s.2, x, y) <= sq(40_000) && d2(s.1, s.2, x, y) > 0)
                .map(|s| (c, x, y, (x - s.1) as f64 / CHECK_EVERY as f64, (y - s.2) as f64 / CHECK_EVERY as f64))
        }).collect();
        {
            let cx = Ctx { seen: &st.seen, cast: &st.cast, cds: &cds_now, shots: &shots, tick };
            for m in all {
                if st.ready.get(&m.id).map_or(false, |&r| tick < r) || m.has("flash_cd") || blocked(m) { continue; }
                if let Some(p) = plan(sim, all, m, &cx) { moves.push((m.id, (m.x, m.y), p)); }
            }
        }
        for (id, _, _) in &moves { st.ready.insert(*id, tick + FLASH_CD); }
        st.seen = all.iter().map(|c| (c.id, (c.x, c.y))).collect();
        st.cds = cds_now;
        st.shots = shots_now;
    }
    for (id, from, to) in moves {
        // also on the champion itself (a second guard on the cooldown, shown nowhere)
        sim.add_buff(id, &timed("flash_cd", FLASH_CD));
        show(sim, all, id, from);
        sim.entity_set_pos(id, to.0 as u64, to.1 as u64);
        show(sim, all, id, to);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ring_stays_on_map() {
        assert!(ring((10_000, 10_000), FLASH_R, 16).iter().all(|p| p.0 > 8_000 && p.1 > 8_000));
        assert_eq!(FLASH_CD, 120 * 60);
    }
    #[test]
    fn toward_stops_short() {
        let m = Champ { id: 1, team: 0, x: 100_000, y: 100_000, buffs: Vec::new(), stunned: false, pushed: false, hp: 1, max_hp: 1, attack: 1, name: String::new() };
        let p = toward(&m, (150_000, 100_000), 15_000).unwrap();
        assert_eq!(p, (135_000, 100_000));
        let far = toward(&m, (300_000, 100_000), 15_000).unwrap();
        assert_eq!(far, (100_000 + FLASH_R, 100_000));
    }
}
