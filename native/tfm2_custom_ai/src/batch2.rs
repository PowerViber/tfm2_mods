//! Rules for the second batch of Skill Lab champions: DIO (jojo), David Martinez (cyberpunk), V1 (ultrakill),
//! Darth Vader (starwars) and Frieren (frieren).
//!
//! The data files trigger everything with short marker buffs (e.g. `dio_knives3`, `v1_coin`, `dv_grav`), like
//! Flying Raijin. DIO and Frieren run from the match hook (`run_dio`, `run_frieren`); V1, Vader and David are
//! native passives (`tfm2_custom_ai:v1` / `:vader` / `:david`) because they react to damage events and keep bars
//! and stacks in the passive itself.

use super::*;

// ------------------------------------------------------------------ shared helpers

fn parse_vals(rest: &str) -> Vec<i64> {
    rest.split(':').filter_map(|v| v.parse::<i64>().ok()).collect()
}

/// The first buff whose name starts with `prefix`: (full name, numbers after the prefix).
fn state(c: &Champ, prefix: &str) -> Option<(String, Vec<i64>)> {
    c.buffs.iter().find_map(|b| b.name().strip_prefix(prefix).map(|r| (b.name().to_string(), parse_vals(r))))
}

fn states(c: &Champ, prefix: &str) -> Vec<(String, Vec<i64>)> {
    c.buffs.iter().filter_map(|b| b.name().strip_prefix(prefix).map(|r| (b.name().to_string(), parse_vals(r)))).collect()
}

/// Visual binding names are `<champion id>_<name>`; the entity name is the champion id for data champions.
fn vname(c_name: &str, fallback_id: &str, n: &str) -> String {
    let id = if c_name.starts_with("tfm2_") { c_name } else { fallback_id };
    format!("{id}_{n}")
}

fn fx_at(sim: &mut StableSim<'_>, name: &str, caster: usize, x: i64, y: i64, time: u64) {
    sim.play_view_effect(name, caster, &InputTargetV1::pos(x.max(0) as u64, y.max(0) as u64), 0, 0, time);
}

fn fx_on(sim: &mut StableSim<'_>, name: &str, caster: usize, target: usize, time: u64) {
    sim.play_view_effect(name, caster, &InputTargetV1::target(target), 0, 0, time);
}

fn dir(ax: i64, ay: i64, bx: i64, by: i64) -> (f64, f64) {
    let (dx, dy) = ((bx - ax) as f64, (by - ay) as f64);
    let l = dx.hypot(dy);
    if l < 1e-9 { (1.0, 0.0) } else { (dx / l, dy / l) }
}

fn rot(d: (f64, f64), a: f64) -> (f64, f64) {
    (d.0 * a.cos() - d.1 * a.sin(), d.0 * a.sin() + d.1 * a.cos())
}

/// A straight projectile from (x,y) to (tx,ty) applying a registered native effect on hit (or `noop` = visual only).
fn shoot_linear(sim: &mut StableSim<'_>, name: &str, effect: &str, caster: usize, team: usize, x: i64, y: i64, tx: i64, ty: i64,
    speed: i64, radius: i64, penetrate: bool) {
    let spec = ProjectileSpawnV1 {
        caster_id: caster, team, x: x.max(0) as u64, y: y.max(0) as u64, radius: radius.max(1) as u64, speed: speed.max(1) as u64,
        move_kind: ProjectileMoveKindV1::Linear.code(), target_x: tx.max(0) as u64, target_y: ty.max(0) as u64, penetrate,
        attack_type: AttackTypeV1::Skill.code(),
        casting_target: if effect.ends_with(":noop") { CastingTargetV1::None.code() } else { CastingTargetV1::Enemy.code() },
        ..ProjectileSpawnV1::default()
    };
    sim.spawn_projectile(name, &format!("{MOD_ID}:{effect}"), &spec);
}

/// A homing projectile from (x,y) onto `target`.
fn shoot_homing(sim: &mut StableSim<'_>, name: &str, effect: &str, caster: usize, team: usize, x: i64, y: i64, target: usize, speed: i64) {
    let spec = ProjectileSpawnV1 {
        caster_id: caster, team, x: x.max(0) as u64, y: y.max(0) as u64, radius: 6_000, speed: speed.max(1) as u64,
        move_kind: ProjectileMoveKindV1::Target.code(), target_id: target, attack_type: AttackTypeV1::Skill.code(),
        casting_target: if effect.ends_with(":noop") { CastingTargetV1::None.code() } else { CastingTargetV1::Enemy.code() },
        ..ProjectileSpawnV1::default()
    };
    sim.spawn_projectile(name, &format!("{MOD_ID}:{effect}"), &spec);
}

/// Every hard lock at once (the Taoist talismans combined), `ticks` long.
fn seal(sim: &mut StableSim<'_>, id: usize, ticks: u64) {
    for k in [CcKindV1::Stun, CcKindV1::Bind, CcKindV1::BlockAttack, CcKindV1::BlockSkill, CcKindV1::BlockMoveSkill] {
        sim.apply_cc(id, &CcV1::of_kind(k, ticks));
    }
}

/// Hold a champion still under `key` (time stop, gravity): the spot is kept in a buff "<key>:x:y" that the caller
/// refreshes every tick it wants the freeze; when the caller stops, the buff runs out and they thaw.
fn freeze(sim: &mut StableSim<'_>, c: &Champ, key: &str, tick: usize) {
    let prefix = format!("{key}:");
    match state(c, &prefix) {
        None => {
            strip_cc_immunity(sim, c);
            sim.add_buff(c.id, &timed(&format!("{key}:{}:{}", c.x, c.y), 20));
            seal(sim, c.id, 12);
        }
        Some((name, v)) if v.len() >= 2 => {
            let (x, y) = (v[0], v[1]);
            if tick % 10 == 0 {
                strip_cc_immunity(sim, c);
                sim.entity_remove_buff(c.id, &name);
                sim.add_buff(c.id, &timed(&name, 20));
                seal(sim, c.id, 12);
            }
            if d2(c.x, c.y, x, y) > sq(400) && d2(c.x, c.y, x, y) < sq(120_000) {
                sim.entity_set_pos(c.id, x.max(0) as u64, y.max(0) as u64);
            }
        }
        _ => {}
    }
}

fn attack_of(sim: &StableSim<'_>, id: usize) -> usize {
    sim.get_entity(id).map_or(0, |e| e.stat().attack)
}

fn magic_of(sim: &StableSim<'_>, id: usize) -> usize {
    sim.get_entity(id).map_or(0, |e| e.stat().magic_power)
}

/// Cheap deterministic dice (0..100) from the tick and two ids.
fn roll(tick: usize, a: usize, b: usize) -> usize {
    let mut h = (tick as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ (a as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F) ^ (b as u64).wrapping_mul(0x1656_67B1_9E37_79F9);
    h ^= h >> 29;
    h = h.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    h ^= h >> 32;
    (h % 100) as usize
}

fn nearest<'a>(from: &Champ, pool: &[&'a Champ], max: i64) -> Option<&'a Champ> {
    pool.iter().filter(|c| d2(c.x, c.y, from.x, from.y) <= sq(max)).min_by_key(|c| (d2(c.x, c.y, from.x, from.y), c.id)).copied()
}

/// Danger zones other champions' rules avoid (Minato's teleports and dodges), from enemies of `team`:
/// (x, y, radius, the tick it strikes or 0 if it simply lasts, locked target id + 1 or 0).
/// DIO's red mark (strikes after MARK_DELAY), DIO's time-stop field (lasts), David's gravity before its smash.
pub fn danger_zones(all: &[Champ], team: usize) -> Vec<(i64, i64, i64, usize, usize)> {
    let mut z = Vec::new();
    for c in all.iter().filter(|c| c.team != team) {
        if let Some((_, v)) = state(c, "dmk:") {
            if v.len() >= 3 { z.push((v[0], v[1], MARK_R, v[2].max(0) as usize + MARK_DELAY, 0)); }
        }
        if c.has("dio_timestop") {
            if let Some((_, v)) = state(c, "dts:") {
                if v.len() >= 4 { z.push((v[2], v[3], TS_R, 0, 0)); }
            }
        }
        if let Some((_, v)) = state(c, "dvz:") {
            if v.len() >= 4 { z.push((v[0], v[1], if v[3] > 0 { GRAV_R } else { GRAV_R_AREA }, v[2].max(0) as usize, v[3].max(0) as usize)); }
        }
    }
    z
}

// ------------------------------------------------------------------ native effects (damage on hit)

/// DIO's knife: 30 + 60% of DIO's attack.
#[derive(Clone, Debug)]
pub struct DioKnife;
impl StableEffectType for DioKnife {
    fn apply(&self, sim: &mut StableSim<'_>, _seed: u64, caster: usize, input: InputTargetV1) {
        if input.kind == mod_api_stable::InputTargetKindV1::Target.code() {
            let ad = 30 + attack_of(sim, caster) * 60 / 100;
            sim.deal_damage(caster, input.target_id, ad, 0, AttackTypeV1::Skill);
        }
    }
}

fn is_tower(sim: &StableSim<'_>, id: usize) -> bool {
    (0..sim.tower_count()).any(|i| sim.tower_id_at(i) == id)
}

/// One hit per beam per target. A native piercing projectile can apply its effect again on later ticks while it still
/// overlaps the target: Fern's huge slow beam (radius 18000, speed 7000) overlaps for ~5 ticks, so it could hit ~5
/// times (Rian, Oct 1: Frieren mid out-damaged everyone by 2-5x). A short marker on the target blocks the repeats.
fn first_hit(sim: &mut StableSim<'_>, caster: usize, target: usize, key: &str, ticks: usize) -> bool {
    let name = format!("{key}{caster}");
    let hit = sim.get_entity(target).map_or(false, |e| (0..e.buff_count()).filter_map(|i| e.buff_at(i)).any(|b| b.name() == name));
    if hit {
        return false;
    }
    sim.add_buff(target, &timed(&name, ticks));
    true
}

/// Fern's Zoltraak: 40 + 55% of Frieren's magic power (round 25; was 50 + 70%, before that 55 + 80%). Towers take no damage.
#[derive(Clone, Debug)]
pub struct Zoltraak;
impl StableEffectType for Zoltraak {
    fn apply(&self, sim: &mut StableSim<'_>, _seed: u64, caster: usize, input: InputTargetV1) {
        if input.kind == mod_api_stable::InputTargetKindV1::Target.code() && !is_tower(sim, input.target_id)
            && first_hit(sim, caster, input.target_id, "zlt", 45) {
            let ap = 50 + magic_of(sim, caster) * 70 / 100;   // round 71: was 40 + 55%
            sim.deal_damage(caster, input.target_id, 0, ap, AttackTypeV1::Skill);
        }
    }
}

/// Fern's homing Zoltraak in Limiter release (every 4 s): 24 + 35% of Frieren's magic power (round 25; was 30 + 45%). No towers.
#[derive(Clone, Debug)]
pub struct ZoltraakParty;
impl StableEffectType for ZoltraakParty {
    fn apply(&self, sim: &mut StableSim<'_>, _seed: u64, caster: usize, input: InputTargetV1) {
        if input.kind == mod_api_stable::InputTargetKindV1::Target.code() && !is_tower(sim, input.target_id)
            && first_hit(sim, caster, input.target_id, "zlp", 30) {
            let ap = 30 + magic_of(sim, caster) * 45 / 100;   // round 71: was 24 + 35%
            sim.deal_damage(caster, input.target_id, 0, ap, AttackTypeV1::Skill);
        }
    }
}

// ------------------------------------------------------------------ DIO (Stand Out / Stand In)
//
// Two modes, switched by these rules (at most every MODE_CD; also inside the time stop):
//   Stand Out ("dio_out": +range, no lifesteal): his basic attack is the Stand's punch thrown at range. S1 arms the
//     Stand's guard ("dio_guard"): it blocks the next shot from a champion (he takes 75% of it, no crowd control) or
//     counters the next melee hit (the Stand appears in front: stun + barrage). S2: the Stand winds up for 1 s, then
//     lunges (through walls); on a hit it grabs the enemy by the neck, punches and slows it, then returns.
//   Stand In ("dio_in": lifesteal): melee basic attack, S1 knife fan, S2 dash strike (the Stand bursts out in its
//     'the world' pose, DIO vanishes and reappears behind the target, strikes and stuns).
//   Ult (both): time stops for 2 s in a black-and-white field the size of Unlimited Void; the Stand stands at its
//     centre. Everyone else inside is frozen; his damage is lower meanwhile (data buff), and he can switch modes.
// The Stand floats behind and beside him in Stand Out (drawn here, pose by pose); in Stand In it only appears for
// the dash strike.

const DIO_ID: &str = "tfm2_jojo_dio";
const KNIFE_RANGE: i64 = 160_000;
const KNIFE_SPEED: i64 = 6_500;
/// The dash strike's "!" mark: published as "dmk:<x>:<y>:<t0>" for danger_zones while it's pending.
const MARK_R: i64 = 24_200;
const MARK_DELAY: usize = 36;
/// Time stop: a flat field the size of Unlimited Void, fixed where he cast it.
const TS_R: i64 = 76_000;
const TS_TICKS: usize = 120;          // 2 s
const TS_KNIVES: usize = 4;
const TS_FAN: f64 = 0.16;             // radians between hanging knives
const TS_TAUNT_EVERY: usize = 45;
const TS_TAUNT_TICKS: u64 = 30;
/// Round 26: one teammate is spared (the one closest to DIO inside the field); DIO and that teammate get the time-stop
/// buff for its whole length: +100% cooldown speed, +20% move and attack speed. The guest's damage is cut like DIO's
/// (the data's dio_ts_cut, 40%), since frozen enemies are free hits.
const TS_CDR: i32 = 100;
const TS_MS: i32 = 20;
const TS_AS: i32 = 20;
const TS_GUEST_CUT: i32 = 40;
// modes
const MODE_CD: usize = 300;           // at most one switch every 5 s
const MELEE_IN: i64 = 28_000;         // target this close: Stand In (melee, lifesteal)
const RANGED_OUT: i64 = 38_000;       // target this far: Stand Out (the Stand's reach; round 26: was 45000)
const OUT_RANGE_BONUS: usize = 18_000; // Stand Out basic attack range (23000 + this; round 26: was 32000)
const TOWER_HOLD: usize = 90;          // Stand In held 1.5 s after each hit on a tower
const OUT_ATTACK_SPEED: i32 = 45;       // Stand Out: +45% attack speed (each punch weaker)
const IN_VAMP: i32 = 12;
// Stand Out: guard and counter
const GUARD_INCOMING_R: i64 = 16_000;
const GUARD_BLOCK_TICKS: usize = 8;
const GUARD_BLOCK_REDUCE: usize = 25; // he takes 75%
const COUNTER_MELEE_R: i64 = 30_000;
const COUNTER_STUN: u64 = 45;
const COUNTER_TICKS: usize = 36;
const COUNTER_HITS: usize = 6;
const COUNTER_HIT: (usize, usize) = (6, 12);   // per hit: base + % AD
// Stand Out: lunge and grab
const LUNGE_WINDUP: usize = 60;
const LUNGE_SPEED: i64 = 4_500;      // ~18 ticks over its range so it can be seen (was 9000: gone in 9 ticks)
const LUNGE_RANGE: i64 = 80_000;
const LUNGE_HIT_R: i64 = 14_000;
const GRAB_TICKS: usize = 42;          // 0.7 s
const GRAB_HITS: usize = 5;
const GRAB_HIT: (usize, usize) = (10, 14);
const GRAB_SLOW: i32 = -40;
const RETURN_TICKS: usize = 14;
// Stand In: dash strike
const DASH_BEHIND: i64 = 9_000;
const DASH_HIT: (usize, usize) = (60, 100);
const DASH_STUN: u64 = 36;
// where the Stand floats in Stand Out
const STAND_BACK: i64 = 9_000;
// Stand Out basic attack: the Stand rushes to the target (PUNCH_TRAVEL), punches (PUNCH_SWING), lingers by it
const PUNCH_TRAVEL: usize = 10;
const PUNCH_SWING: usize = 12;
const PUNCH_LINGER: usize = 24;
const STAND_UP: i64 = 3_000;

pub fn is_dio(c: &Champ) -> bool {
    // his own buffs only: enemies he marks carry `dio_target`, which must not count
    c.buffs.iter().any(|b| {
        let n = b.name();
        (n.starts_with("dio_") && n != "dio_target") || n.starts_with("dkn") || n.starts_with("dts")
    })
}

/// A fan of knives hanging in stopped time just in front of DIO, pointing at `t`; they fly straight on when time
/// moves again. State per knife: "dkn<slot>:<dx>:<dy>:<x>:<y>" (direction ×1000).
fn hang_fan(sim: &mut StableSim<'_>, d: &Champ, t: &Champ, first_slot: usize) {
    let base = dir(d.x, d.y, t.x, t.y);
    let hang = vname(&d.name, DIO_ID, "knife_hang");
    for k in 0..TS_KNIVES {
        let a = (k as f64 - (TS_KNIVES as f64 - 1.0) / 2.0) * TS_FAN;
        let (ex, ey) = rot(base, a);
        let (x, y) = (d.x + (ex * 10_000.0) as i64, d.y + (ey * 10_000.0) as i64);
        sim.add_buff(d.id, &timed(&format!("dkn{}:{}:{}:{x}:{y}", first_slot + k, (ex * 1000.0) as i64, (ey * 1000.0) as i64), 600));
        fx_at(sim, &hang, d.id, x, y, 30);
    }
}

fn release_knives(sim: &mut StableSim<'_>, d: &Champ) {
    let knife = vname(&d.name, DIO_ID, "knife");
    for (name, v) in states(d, "dkn") {
        if v.len() >= 5 {
            let (dx, dy) = (v[1] as f64 / 1000.0, v[2] as f64 / 1000.0);
            let (x, y) = (v[3], v[4]);
            shoot_linear(sim, &knife, "dio_knife", d.id, d.team, x, y, x + (dx * KNIFE_RANGE as f64) as i64, y + (dy * KNIFE_RANGE as f64) as i64,
                KNIFE_SPEED, 7_000, false);
        }
        sim.entity_remove_buff(d.id, &name);
    }
}

/// Where ZA WARUDO is worth casting (round 33: no more lone time stops): the best spot DIO can blink to (at most
/// TS_BLINK, his Stand Out basic attack range; round 34: was 170000), toward a group of enemies his team can see,
/// stopping 15000 short of them, and the enemy to hold so the AI presses it.
/// Worth it when the field catches 2+ enemies, or one with a teammate near enough to join (TS_MATE) or at 40% HP or
/// less. Never a lone dive under an enemy tower (fewer than 3 caught).
const TS_BLINK: i64 = 23_000 + OUT_RANGE_BONUS as i64;   // 41000: his Stand Out basic attack range
const TS_REACH: i64 = TS_BLINK + TS_R;                   // groups farther than this can't be caught
const TS_MATE: i64 = 110_000;
/// `relaxed` (the cast already happened, e.g. someone else's CC made the AI press it): any spot that catches an enemy.
fn ts_plan(sim: &StableSim<'_>, d: &Champ, all: &[Champ], relaxed: bool) -> Option<((i64, i64), usize)> {
    let enemies: Vec<&Champ> = all.iter().filter(|c| c.team != d.team && sim.is_visible(d.team, c.id)).collect();
    let towers: Vec<(i64, i64)> = (0..sim.tower_count()).filter_map(|i| sim.get_entity(sim.tower_id_at(i)))
        .filter(|t| t.is_alive() && t.team() != d.team).map(|t| { let (x, y) = t.pos(); (x as i64, y as i64) }).collect();
    let mut best: Option<(i64, (i64, i64), usize)> = None;
    for e in enemies.iter().filter(|e| d2(e.x, e.y, d.x, d.y) <= sq(TS_REACH)) {
        let grp: Vec<&&Champ> = enemies.iter().filter(|o| d2(o.x, o.y, e.x, e.y) <= sq(TS_R * 4 / 5)).collect();
        let n = grp.len() as i64;
        let c = (grp.iter().map(|g| g.x).sum::<i64>() / n.max(1), grp.iter().map(|g| g.y).sum::<i64>() / n.max(1));
        // land just in front of them (15000 short, on his side), unless he's already that close
        let dist = (d2(c.0, c.1, d.x, d.y) as f64).sqrt();
        let spot = if dist <= 25_000.0 { (d.x, d.y) } else {
            let (ux, uy) = dir(d.x, d.y, c.0, c.1);
            let hop = (dist - 15_000.0).clamp(0.0, TS_BLINK as f64);
            crate::walls::pull_back(d.x, d.y, d.x + (ux * hop) as i64, d.y + (uy * hop) as i64)
        };
        let caught = enemies.iter().filter(|o| d2(o.x, o.y, spot.0, spot.1) <= sq(TS_R - 6_000)).count() as i64;
        let mates = all.iter().filter(|a| a.team == d.team && a.id != d.id && d2(a.x, a.y, spot.0, spot.1) <= sq(TS_MATE)).count() as i64;
        let low = enemies.iter().any(|o| d2(o.x, o.y, spot.0, spot.1) <= sq(TS_R - 6_000) && o.max_hp > 0 && o.hp * 100 / o.max_hp <= 40);
        let ok = caught >= 2 || (caught >= 1 && (mates >= 1 || low)) || (relaxed && caught >= 1);
        let tower_dive = !relaxed && towers.iter().any(|t| d2(t.0, t.1, spot.0, spot.1) <= sq(60_000)) && caught < 3;
        if !ok || tower_dive { continue; }
        // an enemy already stunned in the field: the time stop chains onto it
        let held = enemies.iter().any(|o| o.stunned && d2(o.x, o.y, spot.0, spot.1) <= sq(TS_R - 6_000));
        let score = caught * 10 + mates * 5 + if low { 8 } else { 0 } + if held { 10 } else { 0 } - (dist / 20_000.0) as i64;
        if best.map_or(true, |b| score > b.0) { best = Some((score, spot, e.id)); }
    }
    best.map(|(_, spot, key)| (spot, key))
}

/// Match-hook part: knives (Stand In S1) and the time stop.
pub fn run_dio(sim: &mut StableSim<'_>, d: &Champ, all: &[Champ], tick: usize) {
    let enemies: Vec<&Champ> = all.iter().filter(|c| c.team != d.team).collect();
    let target = |mark: &str, max: i64| -> Option<&Champ> {
        enemies.iter().filter(|e| e.has(mark)).min_by_key(|e| (d2(e.x, e.y, d.x, d.y), e.id)).copied()
            .or_else(|| nearest(d, &enemies, max))
    };
    let stopped = d.has("dio_timestop");
    let freeze_key = format!("tsf{}", d.id);
    let frozen_prefix = format!("{freeze_key}:");
    let mut frozen: Vec<&Champ> = enemies.iter().filter(|e| e.buffs.iter().any(|b| b.name().starts_with(frozen_prefix.as_str()))).copied().collect();
    frozen.sort_by_key(|e| (d2(e.x, e.y, d.x, d.y), e.id));

    // ---- knife fan: 3 straight knives; in stopped time 4 that hang in the air until time moves again
    if (d.has("dio_knives3") || d.has("dio_knives4")) && !d.has("dio_kseen") && d.has("dio_out") && !d.has("dio_sw_in") && !d.has("dio_in") {
        // the Stand is out: no knives (the cast arrived as the mode changed) — the Stand stands guard instead
        sim.add_buff(d.id, &timed("dio_kseen", 8));
        if !d.has("dio_guard") { sim.add_buff(d.id, &BuffV1::named("dio_guard")); }
    } else if (d.has("dio_knives3") || d.has("dio_knives4")) && !d.has("dio_kseen") {
        sim.add_buff(d.id, &timed("dio_kseen", 8));
        if let Some(t) = frozen.first().copied().or_else(|| target("dio_target", KNIFE_RANGE + 10_000)) {
            if stopped {
                hang_fan(sim, d, t, (tick / 7 % 2) * TS_KNIVES);
            } else {
                let base = dir(d.x, d.y, t.x, t.y);
                let knife = vname(&d.name, DIO_ID, "knife");
                for a in [-0.2f64, 0.0, 0.2] {
                    let (ex, ey) = rot(base, a);
                    shoot_linear(sim, &knife, "dio_knife", d.id, d.team, d.x, d.y, d.x + (ex * KNIFE_RANGE as f64) as i64, d.y + (ey * KNIFE_RANGE as f64) as i64,
                        KNIFE_SPEED, 7_000, false);
                }
            }
        }
    }
    if !stopped && d.buffs.iter().any(|b| b.name().starts_with("dkn")) {
        release_knives(sim, d);
    } else if tick % 30 == 0 {
        let hang = vname(&d.name, DIO_ID, "knife_hang");
        for (_, v) in states(d, "dkn") {
            if v.len() >= 5 {
                fx_at(sim, &hang, d.id, v[3], v[4], 30);
            }
        }
    }

    // ---- ZA WARUDO: everyone else inside the field is frozen; the Stand stands at its centre
    if stopped {
        let (fx0, fy0) = match state(d, "dts:") {
            Some((_, v)) if v.len() >= 4 => (v[2], v[3]),
            _ => {
                // round 33: he blinks in first, right in front of the enemies he's stopping time on, so the field
                // catches them (and he isn't left alone in it)
                let spot = ts_plan(sim, d, all, false).or_else(|| ts_plan(sim, d, all, true)).map_or((d.x, d.y), |p| p.0);
                if d2(spot.0, spot.1, d.x, d.y) > sq(12_000) {
                    fx_at(sim, &vname(&d.name, DIO_ID, "vanish"), d.id, d.x, d.y, 18);
                    sim.entity_set_pos(d.id, spot.0.max(0) as u64, spot.1.max(0) as u64);
                    fx_at(sim, &vname(&d.name, DIO_ID, "appear"), d.id, spot.0, spot.1, 18);
                }
                let (cx, cy) = spot;
                sim.add_buff(d.id, &timed(&format!("dts:{tick}:0:{cx}:{cy}"), 400));
                fx_at(sim, &vname(&d.name, DIO_ID, "timestop_field"), d.id, cx, cy, TS_TICKS as u64);
                fx_at(sim, &vname(&d.name, DIO_ID, "stand_world_ult"), d.id, cx, cy - STAND_UP, TS_TICKS as u64);
                (cx, cy)
            }
        };
        // the ONE teammate closest to DIO inside the field moves in stopped time with him (like Gojo's guest); both get
        // the time-stop buff (cooldown speed, speed)
        let guest_key = format!("dtg{}:", d.id);
        let guest = match state(d, &guest_key) {
            Some((_, v)) => v.first().filter(|&&g| g >= 0).map(|&g| g as usize),
            None => {
                let g = all.iter().filter(|c| c.id != d.id && c.team == d.team && d2(c.x, c.y, fx0, fy0) <= sq(TS_R))
                    .min_by_key(|c| (d2(c.x, c.y, fx0, fy0), c.id)).map(|c| c.id);
                sim.add_buff(d.id, &timed(&format!("{guest_key}{}", g.map_or(-1, |g| g as i64)), TS_TICKS + 10));
                let world = |name: &str| {
                    let mut b = timed(name, TS_TICKS);
                    b.skill_cooldown_mult = TS_CDR;
                    b.move_speed_mult = TS_MS;
                    b.attack_speed_mult = TS_AS;
                    b
                };
                sim.add_buff(d.id, &world("dio_world"));
                if let Some(g) = g {
                    let mut b = world("dio_world_guest");
                    b.attack_mult = -TS_GUEST_CUT;
                    b.cc_immune = true;
                    sim.add_buff(g, &b);
                    fx_on(sim, &vname(&d.name, DIO_ID, "stand_swap"), d.id, g, 20);
                }
                g
            }
        };
        for c in all.iter().filter(|c| c.id != d.id && Some(c.id) != guest
            && (d2(c.x, c.y, fx0, fy0) <= sq(TS_R) || c.buffs.iter().any(|b| b.name().starts_with(frozen_prefix.as_str())))) {
            freeze(sim, c, &freeze_key, tick);
        }
        let t0 = state(d, "dts:").and_then(|(_, v)| v.first().copied()).map_or(tick, |t| t.max(0) as usize);
        if tick.saturating_sub(t0) % TS_TAUNT_EVERY == 5 {
            if let Some(t) = frozen.iter().find(|e| e.hp > 0).copied() {
                let mut taunt = CcV1::of_kind(CcKindV1::Taunt, TS_TAUNT_TICKS);
                taunt.target = t.id;
                sim.apply_cc(d.id, &taunt);
            }
        }
    } else if d.buffs.iter().any(|b| b.name().starts_with("dts")) {
        if let Some((name, _)) = state(d, "dts:") {
            sim.entity_remove_buff(d.id, &name);
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum StandAct {
    Punch { t0: usize, tid: usize, sx: i64, sy: i64 },
    Block { t0: usize },
    Counter { t0: usize, foe: usize },
    Windup { t0: usize, tid: usize },
    Lunge { t0: usize, x: i64, y: i64, dx: i64, dy: i64, travelled: i64 },
    Grab { t0: usize, foe: usize },
    Return { t0: usize, x: i64, y: i64 },
    Pop { t0: usize },
}

#[derive(Clone, Default)]
pub struct Dio {
    out: bool,
    mode_ready: usize,
    face: i64,                  // 1 right, -1 left
    last: Option<(i64, i64)>,
    act: Option<StandAct>,
    alert_ready: usize,
    spos: Option<(i64, i64)>,   // where the Stand was last drawn
    tower_until: usize,         // hitting a tower: hold Stand In until this tick
    out_vamp: i32,              // his items' lifesteal, cancelled in Stand Out
    rdy: [usize; 4],            // per-mode cooldowns: tick each is ready again (S1 In, S1 Out, S2 In, S2 Out)
    dash_pending: Option<(usize, usize)>,   // Stand In S2: (target, tick the red "!" went up); strikes DASH_DELAY later
}

/// Per-mode cooldowns (Rian, Oct 1: "2 different cd for each mode"). The game has one cooldown per slot, so the data
/// asks the native rules which version is ready (buffs dio_r1in / dio_r1out / dio_r2in / dio_r2out, kept here) and
/// reports a use with dio_c1in / dio_c1out / dio_c2in / dio_c2out. A cast whose own mode's version is on cooldown
/// switches stance (dio_sw_in / dio_sw_out) and uses the other mode's version: S2 dash, S1 knives, S2 again (change +
/// lunge), S1 again (guard). The slot's real cooldown is cut to ~1 s when the other mode's version is still ready.
const DIO_CD: [usize; 4] = [600, 600, 660, 660];
const DIO_RDY: [&str; 4] = ["dio_r1in", "dio_r1out", "dio_r2in", "dio_r2out"];
const DIO_USE: [&str; 4] = ["dio_c1in", "dio_c1out", "dio_c2in", "dio_c2out"];
/// The dash strike's red "!" over the target before DIO vanishes and strikes (also a danger zone Minato avoids).
const DASH_DELAY: usize = MARK_DELAY;

/// Lifesteal from a champion's items (the base items that carry vamp).
fn item_vamp(sim: &StableSim<'_>, entity: usize) -> i32 {
    let mut v = 0;
    for i in 0..sim.player_count() {
        let Some(p) = sim.player_at(i) else { continue };
        if p.champion().map(|c| c.id()) != Some(entity) { continue; }
        for k in p.item_keys() {
            v += match k.as_str() {
                s if s.contains("ruinous_blade") => 5,
                s if s.contains("conquerors_greatsword") => 10,
                s if s.contains("warlords_final_judgement") => 15,
                _ => 0,
            };
        }
    }
    v
}

impl Dio {
    fn set_mode(&mut self, sim: &mut StableSim<'_>, m: &Champ, out: bool, tick: usize) {
        self.out = out;
        self.mode_ready = tick + MODE_CD;
        sim.entity_remove_buff(m.id, "dio_out");
        sim.entity_remove_buff(m.id, "dio_in");
        if out {
            let mut b = BuffV1::named("dio_out");
            b.range = OUT_RANGE_BONUS;
            b.attack_speed_mult = OUT_ATTACK_SPEED;   // the Stand's rapid punches
            b.vamp = -item_vamp(sim, m.id);           // cancels lifesteal from his items too: Stand Out has none
            sim.add_buff(m.id, &b);
        } else {
            let mut b = BuffV1::named("dio_in");
            b.vamp = IN_VAMP;
            sim.add_buff(m.id, &b);
            sim.entity_remove_buff(m.id, "dio_guard");   // the guard is a Stand Out thing
        }
        let (sx, sy) = self.stand_home(m);
        fx_at(sim, &vname(&m.name, DIO_ID, "stand_swap"), m.id, sx, sy, 20);
    }
    /// Re-apply the mode buff when his items' lifesteal changed (he bought or sold one), so Stand Out stays at 0.
    fn refresh_out_vamp(&mut self, sim: &mut StableSim<'_>, m: &Champ) {
        let v = item_vamp(sim, m.id);
        if v != self.out_vamp {
            self.out_vamp = v;
            if self.out {
                sim.entity_remove_buff(m.id, "dio_out");
                let mut b = BuffV1::named("dio_out");
                b.range = OUT_RANGE_BONUS;
                b.attack_speed_mult = OUT_ATTACK_SPEED;
                b.vamp = -v;
                sim.add_buff(m.id, &b);
            }
        }
    }
    fn stand_home(&self, m: &Champ) -> (i64, i64) {
        (m.x - self.face * STAND_BACK, m.y - STAND_UP)
    }
    fn draw(&self, sim: &mut StableSim<'_>, m: &Champ, pose: &str, frame: usize, x: i64, y: i64, face: i64) {
        // a pose every 2 ticks lasting exactly 2 ticks (a 1-tick effect plays its whole animation instead, which
        // left a trail of copies behind the Stand); the position itself is updated every tick
        if sim.tick() % 2 != 0 {
            return;
        }
        let side = if face >= 0 { 'r' } else { 'l' };
        fx_at(sim, &vname(&m.name, DIO_ID, &format!("stand_{pose}{side}{frame}")), m.id, x, y, 2);
    }
}

impl StablePassive for Dio {
    fn clone_box(&self) -> Box<dyn StablePassive> {
        Box::new(self.clone())
    }
    fn on_spawn(&mut self, _sim: &mut StableSim<'_>, _player: usize, _entity: usize) {
        *self = Dio { face: 1, ..Dio::default() };
    }
    fn on_dead(&mut self, _sim: &mut StableSim<'_>, _player: usize) {
        self.act = None;
        self.dash_pending = None;
        self.last = None;
        self.mode_ready = 0;
        self.spos = None;
    }
    fn on_attack(&mut self, sim: &mut StableSim<'_>, _player: usize, entity: usize, target: usize, _damage: &mut usize) {
        // towers: the Stand doesn't siege from range. A basic attack at a tower switches him to Stand In at once (melee,
        // like any melee champion against a tower), and he stays in Stand In while he keeps hitting towers.
        if is_tower(sim, target) {
            let tick = sim.tick();
            self.tower_until = tick + TOWER_HOLD;
            if self.out {
                let all = champions(sim);
                if let Some(m) = all.iter().find(|c| c.id == entity).cloned() {
                    self.set_mode(sim, &m, false, tick);
                }
            }
            return;
        }
        // Stand Out: the Stand rushes to the target and punches it there (the data's hit is an invisible, slower
        // projectile, so the damage lands about when the Stand's fist does)
        let free = matches!(self.act, None | Some(StandAct::Punch { .. }) | Some(StandAct::Return { .. }));
        if self.out && free {
            let (sx, sy) = self.spos.unwrap_or_else(|| {
                sim.get_entity(entity).map_or((0, 0), |e| { let (x, y) = e.pos(); (x as i64 - self.face * STAND_BACK, y as i64 - STAND_UP) })
            });
            self.act = Some(StandAct::Punch { t0: sim.tick(), tid: target, sx, sy });
        }
    }
    fn on_damaged(&mut self, sim: &mut StableSim<'_>, _player: usize, entity: usize, attacker: usize, _damage: usize) {
        if !self.out {
            return;
        }
        let tick = sim.tick();
        let all = champions(sim);
        let Some(m) = all.iter().find(|c| c.id == entity).cloned() else { return };
        if !m.has("dio_guard") || m.stunned || m.has("dio_timestop") {
            return;
        }
        // a melee hit from an enemy champion: the Stand counters
        if let Some(foe) = all.iter().find(|c| c.id == attacker && c.team != m.team && d2(c.x, c.y, m.x, m.y) <= sq(COUNTER_MELEE_R)) {
            sim.entity_remove_buff(entity, "dio_guard");
            if crate::batch2::try_parry(sim, foe, entity, hit_estimate(sim, &m, 100)) {
                return;   // V1 parried the counter
            }
            sim.apply_cc(foe.id, &CcV1::of_kind(CcKindV1::Stun, COUNTER_STUN));
            let (bx, by) = ((m.x + foe.x) / 2, (m.y + foe.y) / 2);
            fx_at(sim, &vname(&m.name, DIO_ID, "stand_block"), entity, bx, by - STAND_UP, 16);
            if !matches!(self.act, Some(StandAct::Windup { .. }) | Some(StandAct::Lunge { .. }) | Some(StandAct::Grab { .. })) {
                self.act = Some(StandAct::Counter { t0: tick, foe: foe.id });
            } else {
                // busy lunging: the counter's stun still lands, the barrage is a single hit
                sim.deal_damage(entity, foe.id, attack_of(sim, entity) * COUNTER_HIT.1 * COUNTER_HITS as usize / 100 / 2, 0, AttackTypeV1::Skill);
            }
        }
    }
    fn on_update(&mut self, sim: &mut StableSim<'_>, _seed: u64, _player: usize, entity: usize) {
        let tick = sim.tick();
        let all = champions(sim);
        let Some(m) = all.iter().find(|c| c.id == entity).cloned() else { return };
        let enemies: Vec<&Champ> = all.iter().filter(|c| c.team != m.team).collect();
        let v = |n: &str| vname(&m.name, DIO_ID, n);
        let ad = |sim: &StableSim<'_>, base: usize, pct: usize| base + attack_of(sim, entity) * pct / 100;

        // facing: toward the nearest enemy close by, otherwise the way he moves
        let near = nearest(&m, &enemies, 90_000);
        if let Some(t) = near {
            if t.x != m.x { self.face = if t.x > m.x { 1 } else { -1 }; }
        } else if let Some((lx, _)) = self.last {
            if (m.x - lx).abs() > 200 { self.face = if m.x > lx { 1 } else { -1 }; }
        }
        self.last = Some((m.x, m.y));
        if self.face == 0 { self.face = 1; }

        if tick % 30 == 0 {
            self.refresh_out_vamp(sim, &m);
        }
        // ---- ZA WARUDO waits for a setup (round 33): dio_ts_ok tells the data it's worth it; while it is (and the ult
        //      is ready) the key enemy is held 2 ticks every 20 so the AI (ult aimed at an enemy in CC) presses it
        if tick % 20 == 0 && !m.has("dio_timestop") {
            let ready = (0..sim.player_count()).filter_map(|i| sim.player_at(i))
                .find(|p| p.champion().map_or(false, |c| c.id() == entity)).and_then(|p| p.cooldowns()).map_or(false, |c| c.3 == 0);
            sim.entity_remove_buff(entity, "dio_ts_ok");
            if let (true, false, Some((_, key))) = (ready, m.stunned, ts_plan(sim, &m, &all, false)) {
                sim.add_buff(entity, &timed("dio_ts_ok", 30));
                sim.apply_cc(key, &CcV1::of_kind(CcKindV1::Bind, 2));
            }
        }
        // ---- per-mode cooldowns: a cast that switches stance, uses reported by the data, readiness buffs
        for (n, out) in [("dio_sw_out", true), ("dio_sw_in", false)] {
            if m.has(n) {
                sim.entity_remove_buff(entity, n);
                if out != self.out {
                    self.set_mode(sim, &m, out, tick);
                }
            }
        }
        for i in 0..4 {
            if m.has(DIO_USE[i]) {
                sim.entity_remove_buff(entity, DIO_USE[i]);
                if tick >= self.rdy[i] {
                    self.rdy[i] = tick + DIO_CD[i];
                }
            }
            let ready = tick >= self.rdy[i];
            if ready != m.has(DIO_RDY[i]) {
                if ready { sim.add_buff(entity, &BuffV1::named(DIO_RDY[i])); } else { sim.entity_remove_buff(entity, DIO_RDY[i]); }
            }
        }
        // ---- mode buffs always in place; switching by the rules above
        if !m.has("dio_out") && !m.has("dio_in") {
            let out = self.out;
            self.set_mode(sim, &m, out, tick);
            self.mode_ready = tick + 30;
        } else if tick >= self.mode_ready && !m.stunned && self.dash_pending.is_none() && matches!(self.act, None | Some(StandAct::Punch { .. }))
            && !m.buffs.iter().any(|b| b.name().starts_with("dio_knives") || b.name() == "dio_lunge" || b.name() == "dio_dash") {
            let t = nearest(&m, &enemies, 160_000);
            let want = match t {
                Some(t) => {
                    let dd = d2(t.x, t.y, m.x, m.y);
                    if dd <= sq(MELEE_IN) { Some(false) } else if dd >= sq(RANGED_OUT) { Some(true) } else { None }
                }
                None => None,
            };
            // hitting a tower: stay in Stand In
            let want = if tick < self.tower_until { Some(false) } else { want };
            if let Some(out) = want {
                if out != self.out {
                    self.set_mode(sim, &m, out, tick);
                }
            }
        }

        // ---- Stand Out S1: the guard blocks a dangerous shot about to land
        if self.out && m.has("dio_guard") && !m.stunned && !m.has("dio_block") && !m.has("dio_timestop") {
            let incoming = (0..sim.projectile_count()).filter_map(|i| sim.projectile_at(i)).find(|p| {
                if p.is_end || p.team == m.team { return false; }
                let (px, py) = (p.x as i64, p.y as i64);
                if d2(px, py, m.x, m.y) > sq(GUARD_INCOMING_R) { return false; }
                match all.iter().find(|c| c.id == p.caster_id) {
                    Some(c) => d2(px, py, m.x, m.y) < d2(px, py, c.x, c.y),
                    None => false,   // champions' shots only
                }
            });
            if incoming.is_some() {
                sim.entity_remove_buff(entity, "dio_guard");
                let mut b = timed("dio_block", GUARD_BLOCK_TICKS);
                b.damaged_reduce = GUARD_BLOCK_REDUCE;
                b.cc_immune = true;
                sim.add_buff(entity, &b);
                fx_at(sim, &v("stand_block"), entity, m.x + self.face * 7_000, m.y - STAND_UP, 16);
                // the block never cuts a lunge short (round 26: blocked shots kept wiping the wind-up, so the lunge was
                // never seen)
                if !matches!(self.act, Some(StandAct::Windup { .. }) | Some(StandAct::Lunge { .. }) | Some(StandAct::Grab { .. })) {
                    self.act = Some(StandAct::Block { t0: tick });
                }
            }
        }

        // ---- Stand Out S2: lunge (wind-up first)
        if m.has("dio_lunge") && !m.has("dio_lseen") {
            sim.add_buff(entity, &timed("dio_lseen", 10));
            let t = enemies.iter().filter(|e| e.has("dio_target")).min_by_key(|e| (d2(e.x, e.y, m.x, m.y), e.id)).copied()
                .or_else(|| nearest(&m, &enemies, LUNGE_RANGE));
            if let Some(t) = t {
                self.act = Some(StandAct::Windup { t0: tick, tid: t.id });
                // golden streaks gathering at the Stand's fist for the whole wind-up
                fx_at(sim, &v("stand_charge"), entity, m.x + self.face * 13_000, m.y - STAND_UP - 1_500, LUNGE_WINDUP as u64);
            }
        }
        // ---- Stand In S2: dash strike. A red "!" goes up over the target first (DASH_DELAY); then the Stand bursts out,
        //      DIO vanishes and reappears behind the target and strikes.
        if m.has("dio_dash") && !m.has("dio_dseen") {
            sim.add_buff(entity, &timed("dio_dseen", 10));
            // map awareness: a boat wall cuts him off from a teammate in trouble (or a tower / objective under attack)
            // → the dash (a teleport, so it crosses the wall) goes for the enemy there
            let over = crate::steve::wall_need(sim, &all, &m, tick, 90_000).and_then(|(nx, ny)|
                enemies.iter().filter(|e| d2(e.x, e.y, nx, ny) <= sq(40_000) && d2(e.x, e.y, m.x, m.y) <= sq(100_000))
                    .min_by_key(|e| (d2(e.x, e.y, nx, ny), e.id)).copied());
            let t = over.or_else(|| enemies.iter().filter(|e| e.has("dio_target")).min_by_key(|e| (d2(e.x, e.y, m.x, m.y), e.id)).copied())
                .or_else(|| nearest(&m, &enemies, 90_000));
            if let Some(t) = t {
                fx_on(sim, &v("mark"), entity, t.id, DASH_DELAY as u64);
                self.dash_pending = Some((t.id, tick));
            }
        }
        if let Some((tid, t0)) = self.dash_pending {
            if let Some((n, _)) = state(&m, "dmk:") { sim.entity_remove_buff(entity, &n); }
            match all.iter().find(|c| c.id == tid && c.team != m.team) {
                None => self.dash_pending = None,
                Some(t) if tick < t0 + DASH_DELAY => {
                    sim.add_buff(entity, &timed(&format!("dmk:{}:{}:{t0}", t.x, t.y), 3));
                    let el = tick - t0;
                    // the "!" animation is shorter than the wait: shown again so it stays up until the strike
                    if el > 0 && el % 24 == 0 { fx_on(sim, &v("mark"), entity, t.id, (t0 + DASH_DELAY - tick) as u64); }
                    // the gold strip on the ground from DIO to the target, filling up until he strikes
                    if el % 6 == 0 {
                        let (ax, ay) = dir(m.x, m.y, t.x, t.y);
                        let k = (((ay.atan2(ax) / (std::f64::consts::PI / 4.0)).round() as i64).rem_euclid(8)) as usize;
                        let a = k as f64 * std::f64::consts::PI / 4.0;
                        let stage = (el * 6 / DASH_DELAY).min(5);
                        let half = LUNGE_RANGE as f64 / 2.0;
                        fx_at(sim, &v(&format!("lunge_area{k}_{stage}")), entity, m.x + (a.cos() * half) as i64, m.y + (a.sin() * half) as i64, 6);
                    }
                }
                Some(t) => {
                    self.dash_pending = None;
                    if !m.stunned {
                        fx_at(sim, &v("stand_pop"), entity, m.x - self.face * 6_000, m.y - STAND_UP, 30);
                        fx_at(sim, &v("vanish"), entity, m.x, m.y, 18);
                        let (bx, by) = dir(m.x, m.y, t.x, t.y);
                        let (px, py) = crate::walls::pull_back(t.x, t.y, t.x + (bx * DASH_BEHIND as f64) as i64, t.y + (by * DASH_BEHIND as f64) as i64);
                        sim.entity_set_pos(entity, px.max(0) as u64, py.max(0) as u64);
                        fx_at(sim, &v("appear"), entity, px, py, 18);
                        if !try_parry(sim, t, entity, hit_estimate(sim, &m, 150)) {
                            sim.deal_damage(entity, t.id, ad(sim, DASH_HIT.0, DASH_HIT.1), 0, AttackTypeV1::Skill);
                            sim.apply_cc(t.id, &CcV1::of_kind(CcKindV1::Stun, DASH_STUN));
                            fx_on(sim, &v("strike"), entity, t.id, 12);
                        }
                        if !matches!(self.act, Some(StandAct::Windup { .. }) | Some(StandAct::Lunge { .. }) | Some(StandAct::Grab { .. })) {
                            self.act = Some(StandAct::Pop { t0: tick });
                        }
                    }
                }
            }
        }

        // ---- the Stand's actions
        let mut next = self.act;
        match self.act {
            Some(StandAct::Punch { t0, tid, .. }) => {
                // after the punch it lingers by the target (locked on) a moment, then drifts home
                if tick >= t0 + PUNCH_TRAVEL + PUNCH_SWING + PUNCH_LINGER {
                    let (x, y) = self.spos.unwrap_or(self.stand_home(&m));
                    next = Some(StandAct::Return { t0: tick, x, y });
                } else if sim.get_entity(tid).map_or(true, |e| !e.is_alive()) && tick >= t0 + PUNCH_TRAVEL + PUNCH_SWING {
                    let (x, y) = self.spos.unwrap_or(self.stand_home(&m));
                    next = Some(StandAct::Return { t0: tick, x, y });
                }
            }
            Some(StandAct::Block { t0 }) => { if tick >= t0 + 16 { next = None; } }
            Some(StandAct::Pop { t0 }) => { if tick >= t0 + 30 { next = None; } }
            Some(StandAct::Counter { t0, foe }) => {
                let el = tick.saturating_sub(t0);
                if el >= COUNTER_TICKS || !all.iter().any(|c| c.id == foe) {
                    next = None;
                } else if el % (COUNTER_TICKS / COUNTER_HITS) == 0 {
                    sim.deal_damage(entity, foe, ad(sim, COUNTER_HIT.0, COUNTER_HIT.1), 0, AttackTypeV1::Skill);
                    fx_on(sim, &v("muda"), entity, foe, 8);
                }
            }
            Some(StandAct::Windup { t0, tid }) => {
                if tick >= t0 + LUNGE_WINDUP {
                    let (tx, ty) = all.iter().find(|c| c.id == tid).map_or((m.x + self.face * 1000, m.y), |c| (c.x, c.y));
                    let (sx, sy) = (m.x + self.face * 6_000, m.y - STAND_UP);
                    let (dx, dy) = dir(sx, sy, tx, ty - STAND_UP);
                    next = Some(StandAct::Lunge { t0: tick, x: sx, y: sy, dx: (dx * 1000.0) as i64, dy: (dy * 1000.0) as i64, travelled: 0 });
                }
            }
            Some(StandAct::Lunge { t0, x, y, dx, dy, travelled }) => {
                // through walls: no wall checks here
                let (nx, ny) = (x + dx * LUNGE_SPEED / 1000, y + dy * LUNGE_SPEED / 1000);
                let hit = enemies.iter().filter(|e| d2(e.x, e.y - STAND_UP, nx, ny) <= sq(LUNGE_HIT_R)).min_by_key(|e| (d2(e.x, e.y, nx, ny), e.id)).copied();
                if let Some(e) = hit {
                    if try_parry(sim, e, entity, hit_estimate(sim, &m, 120)) {
                        next = Some(StandAct::Return { t0: tick, x: nx, y: ny });
                    } else {
                        let mut slow = timed("dio_grab_slow", GRAB_TICKS);
                        slow.move_speed_mult = GRAB_SLOW;
                        sim.entity_remove_buff(e.id, "dio_grab_slow");
                        sim.add_buff(e.id, &slow);
                        next = Some(StandAct::Grab { t0: tick, foe: e.id });
                    }
                } else if travelled + LUNGE_SPEED >= LUNGE_RANGE {
                    next = Some(StandAct::Return { t0: tick, x: nx, y: ny });
                } else {
                    next = Some(StandAct::Lunge { t0, x: nx, y: ny, dx, dy, travelled: travelled + LUNGE_SPEED });
                }
            }
            Some(StandAct::Grab { t0, foe }) => {
                let el = tick.saturating_sub(t0);
                let fpos = all.iter().find(|c| c.id == foe).map(|c| (c.x, c.y));
                match fpos {
                    Some((fx, fy)) if el < GRAB_TICKS => {
                        if el % (GRAB_TICKS / GRAB_HITS) == 0 {
                            sim.deal_damage(entity, foe, ad(sim, GRAB_HIT.0, GRAB_HIT.1), 0, AttackTypeV1::Skill);
                            fx_on(sim, &v("muda"), entity, foe, 8);
                        }
                        let _ = (fx, fy);
                    }
                    Some((fx, fy)) => next = Some(StandAct::Return { t0: tick, x: fx - self.face * 6_000, y: fy - STAND_UP }),
                    None => next = Some(StandAct::Return { t0: tick, x: m.x, y: m.y - STAND_UP }),
                }
            }
            Some(StandAct::Return { t0, .. }) => { if tick >= t0 + RETURN_TICKS { next = None; } }
            None => {}
        }
        self.act = next;

        // ---- draw the Stand (one pose every tick, each lasting exactly one tick, so it moves smoothly and never
        //      doubles up). Not during the time stop (it's at the field's centre then).
        if m.has("dio_timestop") {
            return;
        }
        let f = self.face;
        match self.act {
            Some(StandAct::Punch { t0, tid, sx, sy }) => {
                // rush to the target's near side, punch there, hold position by it
                let el = tick.saturating_sub(t0);
                if let Some((tx, ty)) = sim.get_entity(tid).filter(|e| e.is_alive()).map(|e| { let (x, y) = e.pos(); (x as i64, y as i64) }) {
                    let ff = if tx >= m.x { 1 } else { -1 };
                    let (gx, gy) = (tx - ff * 8_000, ty - STAND_UP);
                    if el < PUNCH_TRAVEL {
                        // ease out: fast start, soft arrival
                        let k = el as f64 / PUNCH_TRAVEL as f64;
                        let e = 1.0 - (1.0 - k) * (1.0 - k);
                        let (px, py) = (sx + ((gx - sx) as f64 * e) as i64, sy + ((gy - sy) as f64 * e) as i64);
                        self.draw(sim, &m, "lunge", 0, px, py, ff);
                        self.spos = Some((px, py));
                    } else {
                        let k = ((el - PUNCH_TRAVEL) / 3).min(3);
                        self.draw(sim, &m, "punch", k, gx, gy, ff);
                        self.spos = Some((gx, gy));
                    }
                }
            }
            Some(StandAct::Block { t0 }) => {
                let k = (tick.saturating_sub(t0) / 4).min(3);
                self.draw(sim, &m, "punch", k, m.x + f * 7_000, m.y - STAND_UP, f);
            }
            Some(StandAct::Counter { foe, .. }) => {
                let (fx, fy) = all.iter().find(|c| c.id == foe).map_or((m.x + f * 12_000, m.y), |c| (c.x, c.y));
                let ff = if fx >= m.x { 1 } else { -1 };
                self.draw(sim, &m, "bar", (tick / 2) % 4, (m.x * 2 + fx) / 3, (m.y * 2 + fy) / 3 - STAND_UP, ff);
            }
            Some(StandAct::Windup { t0, tid }) => {
                // the lunge's area on the ground, from under the Stand toward the target, filling up as it charges
                // (stage 0..5, redrawn every 6 ticks so it tracks the target)
                let el = tick.saturating_sub(t0);
                if el % 6 == 0 {
                    let (sx, sy) = (m.x + f * 6_000, m.y);
                    let (tx, ty) = all.iter().find(|c| c.id == tid).map_or((sx + f * 1_000, sy), |c| (c.x, c.y));
                    let (ax, ay) = dir(sx, sy, tx, ty);
                    let k = (((ay.atan2(ax) / (std::f64::consts::PI / 4.0)).round() as i64).rem_euclid(8)) as usize;
                    let a = k as f64 * std::f64::consts::PI / 4.0;
                    let stage = (el * 6 / LUNGE_WINDUP).min(5);
                    let half = LUNGE_RANGE as f64 / 2.0;
                    fx_at(sim, &v(&format!("lunge_area{k}_{stage}")), entity, sx + (a.cos() * half) as i64, sy + (a.sin() * half) as i64, 6);
                }
                // the Stand draws its arm back and the flare at its fist grows ('world' pose 0..3), trembling at the end
                let k = (el * 4 / LUNGE_WINDUP).min(3);
                let shake = if k == 3 && (tick / 2) % 2 == 0 { 400 } else { 0 };
                self.draw(sim, &m, "world", k, m.x + f * 6_000 + shake, m.y - STAND_UP, f);
            }
            Some(StandAct::Lunge { x, y, dx, .. }) => {
                let ff = if dx >= 0 { 1 } else { -1 };
                // a fading golden afterimage left behind every 2 ticks, so the dash reads as a streak
                if tick % 2 == 0 {
                    fx_at(sim, &v(if ff >= 0 { "stand_ghostr" } else { "stand_ghostl" }), entity, x, y, 9);
                }
                self.draw(sim, &m, "lunge", 0, x, y, ff);
            }
            Some(StandAct::Grab { foe, .. }) => {
                if let Some(c) = all.iter().find(|c| c.id == foe) {
                    let ff = if c.x >= m.x { 1 } else { -1 };
                    let pose = if (tick / 6) % 2 == 0 { "grab" } else { "bar" };
                    let k = if pose == "grab" { 0 } else { (tick / 2) % 4 };
                    self.draw(sim, &m, pose, k, c.x - ff * 7_000, c.y - STAND_UP, ff);
                }
            }
            Some(StandAct::Return { t0, x, y }) => {
                let k = (tick.saturating_sub(t0) as i64).min(RETURN_TICKS as i64);
                let (hx, hy) = self.stand_home(&m);
                let (px, py) = (x + (hx - x) * k / RETURN_TICKS as i64, y + (hy - y) * k / RETURN_TICKS as i64);
                if self.out { self.draw(sim, &m, "idle", 0, px, py, f); }
                self.spos = Some((px, py));
            }
            Some(StandAct::Pop { .. }) => {}
            None => {
                if self.out {
                    // float after him: close a quarter of the gap every tick (no snapping), with a gentle bob
                    let (hx, hy) = self.stand_home(&m);
                    let (cx, cy) = self.spos.unwrap_or((hx, hy));
                    let (nx, ny) = (cx + (hx - cx) / 4, cy + (hy - cy) / 4);
                    let bob = ((tick as f64) * 0.08).sin() * 450.0;
                    self.draw(sim, &m, "idle", (tick / 8) % 4, nx, ny + bob as i64, f);
                    self.spos = Some((nx, ny));
                }
            }
        }
        let _ = self.alert_ready;
    }
}

// ------------------------------------------------------------------ Frieren

const FRIEREN_ID: &str = "tfm2_frieren_frieren";
const FERN_DELAY: usize = 90;
/// Fern picks her Zoltraak by distance: close → slow and huge; far → long and thin.
const FERN_CLOSE: i64 = 70_000;
const ZOLT_BIG: (i64, i64, i64) = (7_000, 18_000, 150_000);    // speed, radius, range
const ZOLT_LONG: (i64, i64, i64) = (11_000, 6_000, 260_000);
/// Stark's jump: lands on the spot the target stood on when he jumped (no homing), except in Limiter release.
const STARK_T: usize = 50;
const STARK_HEIGHT: f64 = 36_000.0;
const STARK_SMALL: i64 = 20_000;       // knock-up circle (was 12000)
const STARK_BIG: i64 = 40_000;         // slowing shockwave (was 50000)
const STARK_AIR: u64 = 50;
const STARK_SLOW: i32 = -35;           // round 71 (was -25; -40 before round 25)
const STARK_SLOW_TICKS: usize = 105;   // round 71 (was 75; 120 before round 25)
/// Limiter release: Fern and Stark stay for the whole ult, and both home in.
const PARTY_FERN_EVERY: usize = 180;   // Fern shoots every 3 s in Limiter release (round 71; 4 s since round 25)
/// Stark in Limiter release: one jump onto each enemy that comes within reach (never the same one twice),
/// a jump at most every STARK_JUMP_CD; otherwise he walks to his target and basic-attacks it, slowly.
const STARK_REACH: i64 = 90_000;
const STARK_JUMP_CD: usize = 120;
const STARK_MELEE: i64 = 22_000;
const STARK_BA_CD: usize = 90;          // 1.5 s between swings (a slow attacker)
const STARK_SWING: usize = 20;          // the swing animation (5 poses × 4 ticks)
const STARK_HIT_AT: usize = 10;         // the axe connects 10 ticks into the swing
const STARK_WALK: i64 = 900;            // per tick (a bit slower than a champion)
const STARK_BA_BASE: usize = 30;        // round 71: 30 + 35% AP (was 20 + 25%)
const STARK_BA_AP: usize = 35;
/// Stark as a real companion (like the Druid's bear): a summoned unit for STARK_UNIT_TICKS from the start of Limiter
/// release, targetable, moved and attacking by the game's own summon AI. Mod-summoned units are drawn with the ghoul
/// sprite, which the Frieren mod overrides with Stark's body. He leaps (an instant hop with a landing hit) once onto
/// every new enemy within reach.
const STARK_UNIT_TICKS: usize = 720;    // 12 s
const STARK_UNIT_RANGE: usize = 22_000;
const STARK_UNIT_CD: usize = 90;
/// Stark goes down after this many hits, whatever his HP.
const STARK_MAX_HITS: i64 = 4;   // round 71: Stark survives 4 hits (was 3)
const STARK_HIT_MIN: usize = 10;        // smaller HP drops (damage over time) don't count as a hit

pub fn is_frieren(c: &Champ) -> bool {
    c.has("frieren_fern") || c.has("frieren_stark") || c.has("frieren_limit")
        || c.buffs.iter().any(|b| { let n = b.name(); n.starts_with("ffn:") || n.starts_with("fsk:") || n.starts_with("flp:") || n.starts_with("fsu:") || n.starts_with("fsh:") })
}

/// Start a Stark jump from (sx, sy) at `t`: fixed landing spot, or homing on `t` (Limiter release).
fn stark_jump(sim: &mut StableSim<'_>, f: &Champ, sx: i64, sy: i64, t: &Champ, homing: bool, tick: usize) {
    sim.add_buff(f.id, &timed(&format!("fsk:{sx}:{sy}:{}:{tick}:{}:{}:{}", t.id, t.x, t.y, homing as i64), STARK_T + 10));
    let mark = vname(&f.name, FRIEREN_ID, "stark_mark");
    if homing {
        fx_on(sim, &mark, f.id, t.id, STARK_T as u64);
    } else {
        fx_at(sim, &mark, f.id, t.x, t.y, STARK_T as u64);
    }
}

fn stark_land(sim: &mut StableSim<'_>, f: &Champ, enemies: &[&Champ], x: i64, y: i64, homing: bool) {
    // ability 2: 55 + 80% AP; the free jumps in Limiter release: 35 + 50% AP (round 71, back from 45 + 65% / 25 + 40%)
    let ap = if homing { 35 + f_magic(sim, f.id) * 50 / 100 } else { 55 + f_magic(sim, f.id) * 80 / 100 };
    for e in enemies.iter().filter(|e| d2(e.x, e.y, x, y) <= sq(STARK_BIG)) {
        if d2(e.x, e.y, x, y) <= sq(STARK_SMALL) {
            sim.deal_damage(f.id, e.id, 0, ap, AttackTypeV1::Skill);
            sim.apply_cc(e.id, &CcV1::of_kind(CcKindV1::Airborne, STARK_AIR));
        }
        sim.entity_remove_buff(e.id, "stark_slow");
        let mut slow = timed("stark_slow", STARK_SLOW_TICKS);
        slow.move_speed_mult = STARK_SLOW;
        sim.add_buff(e.id, &slow);
    }
    fx_at(sim, &vname(&f.name, FRIEREN_ID, "stark_land"), f.id, x, y, 24);
}

fn f_magic(sim: &StableSim<'_>, id: usize) -> usize {
    magic_of(sim, id)
}

pub fn run_frieren(sim: &mut StableSim<'_>, f: &Champ, all: &[Champ], tick: usize) {
    let enemies: Vec<&Champ> = all.iter().filter(|c| c.team != f.team).collect();
    let v = |n: &str| vname(&f.name, FRIEREN_ID, n);

    // ---- while Limiter release's Fern or companion Stark is out, she can't cast her own Fern / Stark on top of them
    //      (they stacked: two Ferns and two Starks at once). Her basic attack still works.
    let party_out = f.has("frieren_limit") || state(f, "fsu:").is_some();
    if party_out && tick % 10 == 0 {
        sim.apply_cc(f.id, &CcV1::of_kind(CcKindV1::BlockSkill, 12));
    }
    if !party_out && f.buffs.iter().any(|b| b.name().starts_with("fsh:")) {
        if let Some((n, _)) = state(f, "fsh:") { sim.entity_remove_buff(f.id, &n); }
    }

    // ---- Fern (ability 1): one continuous animation (appear, wait, cast), so only one Fern is ever on screen
    if f.has("frieren_fern") && !f.has("ffn_seen") && !party_out {
        sim.add_buff(f.id, &timed("ffn_seen", 12));
        let t = enemies.iter().filter(|e| e.has("fern_target")).min_by_key(|e| (d2(e.x, e.y, f.x, f.y), e.id)).copied()
            .or_else(|| nearest(f, &enemies, ZOLT_LONG.2));
        if let Some(t) = t {
            let (dx, dy) = dir(f.x, f.y, t.x, t.y);
            let (px, py) = (-dy, dx);   // Fern stands beside Frieren
            let (x, y) = (f.x + (px * 9_000.0) as i64, f.y + (py * 9_000.0) as i64);
            sim.add_buff(f.id, &timed(&format!("ffn:{x}:{y}:{}:{tick}:{}:{}", t.id, (dx * 1000.0) as i64, (dy * 1000.0) as i64), (FERN_DELAY + 30) as usize));
            fx_at(sim, &v("fern_circle"), f.id, x, y, FERN_DELAY as u64);
            fx_at(sim, &v("fern_s1"), f.id, x, y, (FERN_DELAY + 12) as u64);
        }
    }
    if let Some((name, s)) = state(f, "ffn:") {
        if s.len() >= 6 {
            let (x, y, tid, t0) = (s[0], s[1], s[2] as usize, s[3] as usize);
            if tick.saturating_sub(t0) >= FERN_DELAY {
                sim.entity_remove_buff(f.id, &name);
                // aim at where the target is now (or the original direction if it's gone)
                let (dx, dy) = match all.iter().find(|c| c.id == tid) {
                    Some(t) => dir(x, y, t.x, t.y),
                    None => (s[4] as f64 / 1000.0, s[5] as f64 / 1000.0),
                };
                // close target → a slow, huge beam; far target → a long, thin one
                let near = all.iter().find(|c| c.id == tid).map_or(false, |t| d2(t.x, t.y, x, y) <= sq(FERN_CLOSE));
                let (vis, (sp, rad, rng)) = if near { ("zoltraak_big", ZOLT_BIG) } else { ("zoltraak_fern", ZOLT_LONG) };
                shoot_linear(sim, &v(vis), "zoltraak", f.id, f.team, x, y, x + (dx * rng as f64) as i64, y + (dy * rng as f64) as i64, sp, rad, true);
                fx_at(sim, &v("fern_fade"), f.id, x, y, 20);
            }
        }
    }

    // ---- Stark (ability 2): jumps onto the spot the target stands on now; a big red circle marks it
    if f.has("frieren_stark") && !f.has("fsk_seen") && state(f, "fsk:").is_none() && !party_out {
        sim.add_buff(f.id, &timed("fsk_seen", 12));
        let t = enemies.iter().filter(|e| e.has("stark_target")).min_by_key(|e| (d2(e.x, e.y, f.x, f.y), e.id)).copied()
            .or_else(|| nearest(f, &enemies, 90_000));
        if let Some(t) = t {
            stark_jump(sim, f, f.x, f.y, t, false, tick);
        }
    }

    // ---- Stark the companion (a summoned unit, see STARK_UNIT_TICKS): "fsu:<unit id>" lives exactly as long as he does.
    //      Only plain, documented calls touch the unit (position read, entity_set_pos for his leap), and only after
    //      checking it's still him (alive, Frieren's team, not a champion).
    let unit = state(f, "fsu:").and_then(|(_, v)| v.first().copied()).map(|u| u as usize).and_then(|u| {
        sim.get_entity(u).filter(|e| e.is_alive() && e.team() == f.team && !e.is_champion()).map(|e| { let (x, y) = e.pos(); (u, x as i64, y as i64) })
    });
    if let Some((u, ux, uy)) = unit {
        // he takes three hits, then he's gone: "fsh:<hp last tick>:<hits>"
        let hp_now = sim.get_entity(u).map_or(0, |e| e.hp().0);
        let (last_hp, mut hits) = state(f, "fsh:").filter(|(_, v)| v.len() >= 2).map_or((hp_now as i64, 0), |(_, v)| (v[0], v[1]));
        if (hp_now as i64) + (STARK_HIT_MIN as i64) <= last_hp {
            hits += 1;
        }
        if let Some((n, _)) = state(f, "fsh:") { sim.entity_remove_buff(f.id, &n); }
        if hits >= STARK_MAX_HITS && hp_now > 0 {
            // knocked out: the rest of his HP, credited to the nearest enemy champion
            let by = nearest(f, &enemies, 400_000).map_or(u, |e| e.id);
            fx_at(sim, &v("stark_up"), f.id, ux, uy, 10);
            sim.deal_damage_raw(by, u, hp_now + 1, 0, AttackTypeV1::Skill);
        } else {
            sim.add_buff(f.id, &timed(&format!("fsh:{hp_now}:{hits}"), STARK_UNIT_TICKS));
        }
        if tick % 2 == 0 {
            let last_jump = state(f, "fsjt:").and_then(|(_, v)| v.first().copied()).map_or(0, |t| t as usize);
            let fresh = enemies.iter().filter(|e| d2(e.x, e.y, ux, uy) <= sq(STARK_REACH) && !f.has(&format!("fsj{}", e.id)))
                .min_by_key(|e| (d2(e.x, e.y, ux, uy), e.id)).copied();
            if let (true, Some(t)) = (tick >= last_jump + STARK_JUMP_CD, fresh) {
                // a leap: a dust burst where he was, then he's on top of the target with the landing hit
                let (dx, dy) = dir(t.x, t.y, ux, uy);
                let (lx, ly) = crate::walls::pull_back(t.x, t.y, t.x + (dx * 8_000.0) as i64, t.y + (dy * 8_000.0) as i64);
                fx_at(sim, &v("stark_up"), f.id, ux, uy, 10);
                sim.entity_set_pos(u, lx.max(0) as u64, ly.max(0) as u64);
                stark_land(sim, f, &enemies, t.x, t.y, true);
                sim.add_buff(f.id, &timed(&format!("fsj{}", t.id), STARK_UNIT_TICKS));
                if let Some((n, _)) = state(f, "fsjt:") { sim.entity_remove_buff(f.id, &n); }
                sim.add_buff(f.id, &timed(&format!("fsjt:{tick}"), STARK_UNIT_TICKS));
            }
        }
    }

    // ---- Limiter release. Fern: a buff visual on Frieren (`frieren_fern_side`, from the data) standing at her side;
    //      she fires a huge homing Zoltraak every PARTY_FERN_EVERY. Stark: summoned as a unit when it starts (above).
    //      If the game refuses the unit, he's drawn and run here instead (not targetable): "flp:<t0>:<x>:<y>:<face>"
    //      = his spot, "fsba:<tick>" = his last swing.
    if f.has("frieren_limit") {
        let (t0, mut sx, mut sy, mut face) = match state(f, "flp:") {
            Some((_, s)) if s.len() >= 3 => (s[0] as usize, s[1], s[2], s.get(3).copied().unwrap_or(1)),
            _ => {
                let (x, y) = ((f.x - 9_000).max(0), f.y);
                sim.add_buff(f.id, &timed(&format!("flp:{tick}:{x}:{y}:1"), 1_200));
                // the companion: 12 s, from where Frieren stands
                if unit.is_none() {
                    let stat = StatV1 {
                        attack: STARK_BA_BASE + magic_of(sim, f.id) * STARK_BA_AP / 100,
                        hp: 300 + f.max_hp / 4, defence: 20, magic_resistance: 15, move_speed: 1000,
                        ..StatV1::default()
                    };
                    let atk = UnitAttackV1 { attack_ratio: 100, attack: 0, range: STARK_UNIT_RANGE, cooltime: STARK_UNIT_CD, duration: 24,
                        start_timing: 10, cancelable: true, attack_type: AttackTypeV1::BaseAttack.code() };
                    // The match view draws a spawned unit with the base sprite named after it
                    // (asset/base/aseprite_resources/champions/<name>#anim) and panics if there is none — that was the
                    // crash. "ghoul" exists, and the Frieren mod overrides the ghoul sprite with Stark's body.
                    if let Some(uid) = sim.spawn_unit("ghoul", f.id, f.team, x as u64, y as u64, STARK_UNIT_TICKS as u64, &stat, &atk) {
                        sim.add_buff(f.id, &timed(&format!("fsu:{uid}"), STARK_UNIT_TICKS));
                        fx_at(sim, &v("stark_land"), f.id, x, y, 24);
                    }
                }
                (tick, x, y, 1)
            }
        };
        let el = tick.saturating_sub(t0);
        // Fern shoots from where her visual stands (14 px = 13300 units to Frieren's right)
        if el % PARTY_FERN_EVERY == 30 {
            if let Some(t) = nearest(f, &enemies, 200_000) {
                shoot_homing(sim, &v("zoltraak_big"), "zoltraak_party", f.id, f.team, f.x + 13_300, f.y, t.id, 9_000);
            }
        }
        let has_unit = state(f, "fsu:").is_some();
        if !has_unit && state(f, "fsk:").is_none() && tick % 2 == 0 {
            let last_jump = state(f, "fsjt:").and_then(|(_, s)| s.first().copied()).map_or(0, |t| t as usize);
            let fresh = enemies.iter().filter(|e| d2(e.x, e.y, sx, sy) <= sq(STARK_REACH) && !f.has(&format!("fsj{}", e.id)))
                .min_by_key(|e| (d2(e.x, e.y, sx, sy), e.id)).copied();
            let last_ba = state(f, "fsba:").and_then(|(_, s)| s.first().copied()).map_or(0, |t| t.max(0) as usize);
            let swinging = last_ba > 0 && tick < last_ba + STARK_SWING;
            if let (true, false, Some(t)) = (tick >= last_jump + STARK_JUMP_CD, swinging, fresh) {
                stark_jump(sim, f, sx, sy, t, true, tick);
                sim.add_buff(f.id, &timed(&format!("fsj{}", t.id), 1_200));
                if let Some((n, _)) = state(f, "fsjt:") { sim.entity_remove_buff(f.id, &n); }
                sim.add_buff(f.id, &timed(&format!("fsjt:{tick}"), 1_200));
            } else {
                // on foot: walk to the nearest enemy, swing when in reach
                let target = enemies.iter().filter(|e| d2(e.x, e.y, sx, sy) <= sq(200_000))
                    .min_by_key(|e| (d2(e.x, e.y, sx, sy), e.id)).copied();
                let mut moving = false;
                if let Some(t) = target {
                    if t.x != sx { face = if t.x > sx { 1 } else { 0 }; }
                    let dist = (d2(t.x, t.y, sx, sy) as f64).sqrt() as i64;
                    if swinging {
                        if tick == last_ba + STARK_HIT_AT && dist <= STARK_MELEE + 6_000 {
                            let dmg = STARK_BA_BASE + f_magic(sim, f.id) * STARK_BA_AP / 100;
                            sim.deal_damage(f.id, t.id, dmg, 0, AttackTypeV1::Skill);
                        }
                    } else if dist > STARK_MELEE {
                        let step = (STARK_WALK * 2).min(dist - STARK_MELEE + 2_000);
                        let (dx, dy) = dir(sx, sy, t.x, t.y);
                        sx += (dx * step as f64) as i64;
                        sy += (dy * step as f64) as i64;
                        moving = true;
                    } else if tick >= last_ba + STARK_BA_CD {
                        if let Some((n, _)) = state(f, "fsba:") { sim.entity_remove_buff(f.id, &n); }
                        sim.add_buff(f.id, &timed(&format!("fsba:{tick}"), 1_200));
                    }
                }
                let side = if face == 1 { 'r' } else { 'l' };
                let fresh_swing = state(f, "fsba:").and_then(|(_, s)| s.first().copied()).map_or(0, |t| t.max(0) as usize);
                let pose = if fresh_swing > 0 && tick < fresh_swing + STARK_SWING {
                    format!("stark_a{side}{}", ((tick - fresh_swing) / 4).min(4))
                } else if moving {
                    format!("stark_r{side}{}", (tick / 4) % 8)
                } else {
                    format!("stark_i{side}{}", (tick / 8) % 4)
                };
                fx_at(sim, &v(&pose), f.id, sx, sy, 2);
                if let Some((n, _)) = state(f, "flp:") { sim.entity_remove_buff(f.id, &n); }
                sim.add_buff(f.id, &timed(&format!("flp:{t0}:{sx}:{sy}:{face}"), 1_200));
            }
        }
    } else if f.buffs.iter().any(|b| { let n = b.name(); n.starts_with("flp:") || n.starts_with("fsba:") || (unit.is_none() && n.starts_with("fsj")) }) {
        let keep_jumps = unit.is_some();
        let old: Vec<String> = f.buffs.iter().map(|b| b.name().to_string())
            .filter(|n| n.starts_with("flp:") || n.starts_with("fsba:") || (!keep_jumps && n.starts_with("fsj"))).collect();
        for n in old {
            sim.entity_remove_buff(f.id, &n);
        }
    }

    // ---- a Stark jump in flight: "fsk:<sx>:<sy>:<tid>:<t0>:<tx>:<ty>:<homing>"
    if let Some((name, s)) = state(f, "fsk:") {
        if s.len() >= 7 {
            let (sx, sy, tid, t0, homing) = (s[0], s[1], s[2] as usize, s[3] as usize, s[6] != 0);
            let (tx, ty) = if homing { all.iter().find(|c| c.id == tid).map_or((s[4], s[5]), |c| (c.x, c.y)) } else { (s[4], s[5]) };
            let el = tick.saturating_sub(t0);
            if el >= STARK_T {
                sim.entity_remove_buff(f.id, &name);
                stark_land(sim, f, &enemies, tx, ty, homing);
                if let Some((pn, p)) = state(f, "flp:") {
                    sim.entity_remove_buff(f.id, &pn);
                    sim.add_buff(f.id, &timed(&format!("flp:{}:{tx}:{ty}:{}", p.first().copied().unwrap_or(tick as i64), p.get(3).copied().unwrap_or(1)), 1_200));
                }
            } else if el % 2 == 0 {
                // one pose at a time: each lasts exactly until the next is drawn
                let k = el as f64 / STARK_T as f64;
                let h = (std::f64::consts::PI * k).sin() * STARK_HEIGHT;
                let x = sx + ((tx - sx) as f64 * k) as i64;
                let y = sy + ((ty - sy) as f64 * k) as i64 - h as i64;
                fx_at(sim, &v(if k < 0.5 { "stark_up" } else { "stark_down" }), f.id, x, y, 2);
            }
        }
    }
}

// ------------------------------------------------------------------ V1 (passive)

const V1_ID: &str = "tfm2_ultrakill_v1";
/// Coin: tossed up just in front of him and stays in the air COIN_AIR ticks. His next basic attack while it's up
/// shoots the coin instead: that shot plus the skill's damage ricochets into every enemy near the coin, shared out.
/// Once it has dropped, it's gone.
const COIN_AIR: usize = 50;
const COIN_AHEAD: i64 = 14_000;
const COIN_HEIGHT: f64 = 26_000.0;
const COIN_REACH: i64 = 60_000;
const COIN_FAR: i64 = 40_000;           // the coin lands at most this far out, toward the enemy
const COIN_FLY: usize = 18;             // ticks to fly out there
/// Parry: casting S2 arms it ("v1_guard", until used; S2 cooldown 10 s). While armed, the reflex opens a 0.1 s
/// window right before a dangerous shot lands, or the moment an enemy teleports next to him (and DIO's / Minato's
/// teleport strikes check it before they hit).
const PARRY_TICKS: usize = 6;
const PARRY_CD: usize = 600;            // at most one parry every 10 s (S2's cooldown)
const INCOMING_R: i64 = 16_000;
const PARRIED_TICKS: usize = 180;       // after a parry: 3 s of buffs (scaled by the blocked hit) and he hunts
const TP_JUMP: i64 = 35_000;            // an enemy that moved this far in one tick teleported
/// After a parry his next basic attack is a shotgun blast: 6 pellets in a cone, each a share of the shot that falls off
/// with distance (point blank hurts most).
const SHOT_PELLETS: usize = 6;
const SHOT_SPREAD: f64 = 0.35;          // radians either side
const SHOT_RANGE: i64 = 45_000;
const SHOT_PELLET_PCT: usize = 22;      // of the basic attack, per pellet, at point blank (round 25: was 28)
const PUMP2_PCT: usize = 70;            // the second pump hits 70% as hard (round 25: was 100)
const SHOT_MIN_PCT: usize = 25;         // falloff floor at the end of its range
const PELLET_R: i64 = 6_000;
/// While his parry is armed he plays forward: a short nudge toward the nearest enemy champion every second.
const ARMED_NUDGE_EVERY: usize = 60;
const ARMED_NUDGE_TICKS: u64 = 24;
const ARMED_NUDGE_R: i64 = 70_000;
const TP_NEAR: i64 = 35_000;            // … and landed this close to him
const HUNT_TICKS: u64 = 120;            // after a parry he goes for the attacker (Taunt) for 2 s
const PARRY_SFX: &str = "spellbreaker_attack";   // a short metallic ring
const POOL_R: i64 = 12_000;
const POOL_LIFE: usize = 600;
const POOL_MAX: usize = 3;
const POOL_CD: usize = 720;             // a pool at most every 12 s
const BLOOD_HEAL: usize = 70;
const BLOOD_EVERY: usize = 480;
/// Double pump: the shotgun fires twice, the second blast PUMP_DELAY ticks after the first.
const PUMP_DELAY: usize = 12;
/// The coin is shot by itself at its peak (if his basic attack didn't get to it first) as long as an enemy is in
/// reach of it; with the shotgun loaded the whole load goes into the coin ("split shot": every enemy near it takes
/// the full ricochet instead of a share).
const COIN_AUTO_AT: usize = 20;
/// Pressing forward (after a parry, or with a parry armed) only when it's worth it: the enemy within HUNT_MAX, not
/// under its tower (unless it's nearly dead and he's healthy), not into a crowd, not while he's low.
const HUNT_MAX: i64 = 60_000;
const HUNT_TOWER_R: i64 = 85_000;
const HUNT_LOW_PCT: usize = 35;
const PARRIED_NUDGE_EVERY: usize = 30;

#[derive(Clone, Default)]
pub struct V1 {
    pools: Vec<(i64, i64, usize)>,
    pool_ready: usize,
    heal_ready: usize,
    coin: Option<(i64, i64, usize)>,   // ground spot under the coin, tossed at tick
    coin_from: (i64, i64),             // where V1 tossed it from
    alert_ready: usize,
    parried_ready: usize,
    last_pos: Vec<(usize, i64, i64)>,
    hunt: Option<(usize, usize)>,       // (attacker, tick the parry window ends)
    pump: Option<(usize, usize, usize)>, // second shotgun blast: (target, at tick, base damage)
}

/// Railgun (his ult) is up: no parrying meanwhile.
fn v1_railing(c: &Champ) -> bool {
    c.has("v1_rail3") || c.has("v1_rail2") || c.has("v1_rail1")
}

/// Is going after `t` worth it for `m`, or would it get him killed?
fn worth_chase(sim: &StableSim<'_>, m: &Champ, t: &Champ, all: &[Champ]) -> bool {
    let pct = |c: &Champ| if c.max_hp == 0 { 100 } else { c.hp * 100 / c.max_hp };
    if pct(m) <= HUNT_LOW_PCT || d2(m.x, m.y, t.x, t.y) > sq(HUNT_MAX) {
        return false;
    }
    let under_tower = (0..sim.tower_count()).filter_map(|i| sim.get_entity(sim.tower_id_at(i)))
        .filter(|tw| tw.is_alive() && tw.team() != m.team)
        .any(|tw| { let (x, y) = tw.pos(); d2(t.x, t.y, x as i64, y as i64) <= sq(HUNT_TOWER_R) });
    if under_tower && !(pct(t) <= 25 && pct(m) >= 60) {
        return false;
    }
    let foes = all.iter().filter(|c| c.team != m.team && d2(c.x, c.y, t.x, t.y) <= sq(30_000)).count();
    let friends = all.iter().filter(|c| c.team == m.team && c.id != m.id && d2(c.x, c.y, m.x, m.y) <= sq(40_000)).count();
    !(foes >= 3 && friends < 2)
}

/// How hard a hit from `attacker` is likely to be (their attack + magic power, more for big ults / teleport strikes).
pub fn hit_estimate(sim: &StableSim<'_>, attacker: &Champ, mult_pct: usize) -> usize {
    let mut e = attacker.attack + magic_of(sim, attacker.id);
    if attacker.has("gojo_purple_strain") || attacker.has("frieren_limit") || attacker.buffs.iter().any(|b| b.name().starts_with("dts:")) {
        e = e * 5 / 2;
    }
    e * mult_pct / 100
}

/// V1 parries if his parry is armed: opens the 0.1 s window (no damage, no CC) and tells his passive who it was.
/// Called by his own reflex and by teleport strikes (DIO, Minato) right before they hit. true = parried.
pub fn try_parry(sim: &mut StableSim<'_>, target: &Champ, attacker: usize, estimate: usize) -> bool {
    if !target.has("v1_guard") || target.has("v1_parry_cd") || target.stunned || v1_railing(target) {
        return false;
    }
    sim.entity_remove_buff(target.id, "v1_guard");
    sim.add_buff(target.id, &timed("v1_parry_cd", PARRY_CD));
    let mut b = timed("v1_parry", PARRY_TICKS);
    b.damaged_reduce = 100;
    b.base_attack_damaged_reduce = 100;
    b.skill_damaged_reduce = 100;
    b.cc_immune = true;
    sim.add_buff(target.id, &b);
    sim.add_buff(target.id, &timed(&format!("v1_pv:{attacker}:{estimate}"), 12));
    true
}


impl V1 {
    /// One shotgun blast from V1 toward (tx, ty): SHOT_PELLETS pellets in a cone, each hitting the first enemy
    /// champion on its line, with falloff.
    fn blast(&mut self, sim: &mut StableSim<'_>, m: &Champ, all: &[Champ], tx: i64, ty: i64, base: usize, tick: usize) {
        let v = |n: &str| vname(&m.name, V1_ID, n);
        let aim = dir(m.x, m.y, tx, ty);
        let side = if aim.0 >= 0.0 { "r" } else { "l" };
        fx_at(sim, &v(&format!("muzzle_{side}")), m.id, m.x + (aim.0 * 9_000.0) as i64, m.y + (aim.1 * 9_000.0) as i64 - 2_000, 10);
        let mut first_hit: Option<(i64, i64)> = None;
        for k in 0..SHOT_PELLETS {
            let a = -SHOT_SPREAD + 2.0 * SHOT_SPREAD * k as f64 / (SHOT_PELLETS - 1) as f64;
            let d = rot(aim, a);
            let hit = all.iter().filter(|e| e.team != m.team).filter_map(|e| {
                let (ex, ey) = ((e.x - m.x) as f64, (e.y - m.y) as f64);
                let along = ex * d.0 + ey * d.1;
                let off = (ex * d.1 - ey * d.0).abs();
                (along > 0.0 && along <= SHOT_RANGE as f64 && off <= PELLET_R as f64).then_some((e, along as i64))
            }).min_by_key(|&(e, along)| (along, e.id));
            let end = hit.map_or(SHOT_RANGE, |(_, along)| along);
            shoot_linear(sim, &v("pellet"), "noop", m.id, m.team, m.x, m.y, m.x + (d.0 * end as f64) as i64, m.y + (d.1 * end as f64) as i64, 14_000, 1_000, true);
            if let Some((e, along)) = hit {
                let pct = SHOT_PELLET_PCT * (100 - (100 - SHOT_MIN_PCT) * along.min(SHOT_RANGE) as usize / SHOT_RANGE as usize) / 100;
                sim.deal_damage(m.id, e.id, (base * pct / 100).max(1), 0, AttackTypeV1::Skill);
                first_hit.get_or_insert((e.x, e.y));
            }
        }
        if let Some((x, y)) = first_hit {
            self.pool(x, y, tick);
        }
    }
    /// Shoot the coin (if it's up): the shot ricochets into every enemy champion within COIN_REACH of it. Normally
    /// the shot (`base` + 70 + 100% AD) is shared out between them; with the shotgun it's a split shot and each takes
    /// all of it. true = there was a coin to shoot.
    fn shoot_coin(&mut self, sim: &mut StableSim<'_>, m: &Champ, all: &[Champ], base: usize, shotgun: bool, tick: usize) -> bool {
        let Some((cx, cy, t0)) = self.coin else { return false };
        if tick >= t0 + COIN_AIR {
            return false;
        }
        self.coin = None;
        let v = |n: &str| vname(&m.name, V1_ID, n);
        let k = (tick - t0) as f64 / COIN_AIR as f64;
        let h = (std::f64::consts::PI * k).sin() * COIN_HEIGHT;
        let (hx, hy) = (cx, cy - h as i64);
        if shotgun {
            let aim = dir(m.x, m.y, hx, hy);
            let side = if aim.0 >= 0.0 { "r" } else { "l" };
            fx_at(sim, &v(&format!("muzzle_{side}")), m.id, m.x + (aim.0 * 9_000.0) as i64, m.y + (aim.1 * 9_000.0) as i64 - 2_000, 10);
            for a in [-0.08f64, 0.0, 0.08] {
                let d = rot(aim, a);
                let l = ((d2(m.x, m.y, hx, hy) as f64).sqrt()).max(1.0);
                shoot_linear(sim, &v("pellet"), "noop", m.id, m.team, m.x, m.y, m.x + (d.0 * l) as i64, m.y + (d.1 * l) as i64, 14_000, 1_000, true);
            }
        } else {
            shoot_linear(sim, &v("coin_ray"), "noop", m.id, m.team, m.x, m.y, hx, hy, 16_000, 1_000, true);
        }
        let hits: Vec<&Champ> = all.iter().filter(|e| e.team != m.team && d2(e.x, e.y, cx, cy) <= sq(COIN_REACH)).collect();
        if !hits.is_empty() {
            // round 19 (V1 did 27k in mid): 40 + 60% AD on top of the shot (was 70 + 100%); the split shot gives each
            // enemy 60% of it (was 100%)
            // round 25: 30 + 45% AD (was 40 + 60%), split shot 50% each (was 60%)
            let total = base + 30 + m.attack * 45 / 100;
            let each = if shotgun { total * 50 / 100 } else { (total / hits.len()).max(1) };
            for e in &hits {
                shoot_homing(sim, &v("coin_ray"), "noop", m.id, m.team, hx, hy, e.id, 16_000);
                sim.deal_damage(m.id, e.id, each, 0, AttackTypeV1::Skill);
            }
            let first = hits[0];
            self.pool(first.x, first.y, tick);
        }
        true
    }
    fn pool(&mut self, x: i64, y: i64, tick: usize) {
        if tick < self.pool_ready {
            return;
        }
        self.pool_ready = tick + POOL_CD;
        self.pools.retain(|p| p.2 > tick);
        if self.pools.len() >= POOL_MAX {
            self.pools.remove(0);
        }
        self.pools.push((x, y, tick + POOL_LIFE));
    }
    /// A parry went through: ring, "!!", and 3 s of buffs that grow with the hit he blocked (no counter-shot);
    /// once the window closes he goes for the attacker.
    fn parried(&mut self, sim: &mut StableSim<'_>, m: &Champ, attacker: usize, estimate: usize, tick: usize) {
        if tick < self.parried_ready {
            return;
        }
        self.parried_ready = tick + 20;
        sim.play_sfx(PARRY_SFX, m.id, &InputTargetV1::target(m.id));
        fx_on(sim, &vname(&m.name, V1_ID, "alert_parry"), m.id, m.id, 30);
        fx_on(sim, &vname(&m.name, V1_ID, "parry"), m.id, m.id, 12);
        // 15% for a light hit … 70% for something like Purple
        let pct = (10 + estimate * 60 / m.max_hp.max(1)).min(25) as i32;   // round 25: 10..25 (was 10..40, before that 15..70)
        sim.entity_remove_buff(m.id, "v1_parried");
        let mut b = timed("v1_parried", PARRIED_TICKS);
        b.attack_mult = pct;
        b.attack_speed_mult = pct / 2;
        b.move_speed_mult = 20;
        sim.add_buff(m.id, &b);
        sim.entity_remove_buff(m.id, "v1_shotgun");
        sim.add_buff(m.id, &timed("v1_shotgun", PARRIED_TICKS));
        self.hunt = Some((attacker, tick + PARRY_TICKS + 1));
    }
}

impl StablePassive for V1 {
    fn clone_box(&self) -> Box<dyn StablePassive> {
        Box::new(self.clone())
    }
    fn on_dead(&mut self, _sim: &mut StableSim<'_>, _player: usize) {
        self.coin = None;
        self.pump = None;
    }
    fn on_attack(&mut self, sim: &mut StableSim<'_>, _player: usize, entity: usize, target: usize, damage: &mut usize) {
        let tick = sim.tick();
        let all = champions(sim);
        let Some(m) = all.iter().find(|c| c.id == entity).cloned() else { return };
        // a coin in the air: this basic attack shoots the coin instead (with the shotgun loaded: a split shot)
        let shotgun = m.has("v1_shotgun");
        if self.shoot_coin(sim, &m, &all, *damage, shotgun, tick) {
            if shotgun {
                sim.entity_remove_buff(entity, "v1_shotgun");
            }
            *damage = 0;
            return;
        }
        // after a parry: this shot is a shotgun blast, and the double pump fires a second one right after
        if shotgun {
            sim.entity_remove_buff(entity, "v1_shotgun");
            let base = *damage;
            *damage = 0;
            let (tx, ty) = sim.get_entity(target).map(|t| { let (x, y) = t.pos(); (x as i64, y as i64) }).unwrap_or((m.x + 1, m.y));
            self.blast(sim, &m, &all, tx, ty, base, tick);
            self.pump = Some((target, tick + PUMP_DELAY, base));
            return;
        }
        if let Some(t) = sim.get_entity(target) {
            if t.is_champion() && *damage > 0 {
                let (x, y) = t.pos();
                self.pool(x as i64, y as i64, tick);
            }
        }
    }
    fn on_update(&mut self, sim: &mut StableSim<'_>, _seed: u64, _player: usize, entity: usize) {
        let tick = sim.tick();
        let all = champions(sim);
        let Some(m) = all.iter().find(|c| c.id == entity).cloned() else { return };
        let enemies: Vec<&Champ> = all.iter().filter(|c| c.team != m.team).collect();
        let v = |n: &str| vname(&m.name, V1_ID, n);

        // coin toss: ONE coin, up in the air just in front of him
        if m.has("v1_coin") && !m.has("v1_coin_seen") {
            sim.add_buff(entity, &timed("v1_coin_seen", 12));
            let t = enemies.iter().filter(|e| e.has("v1_coin_target")).min_by_key(|e| (d2(e.x, e.y, m.x, m.y), e.id)).copied()
                .or_else(|| nearest(&m, &enemies, 100_000));
            let (dx, dy) = t.map_or((1.0, 0.0), |t| dir(m.x, m.y, t.x, t.y));
            // toward the enemy: 60% of the way there (at least COIN_AHEAD, at most COIN_FAR), so the ricochet is
            // centred on them, not on V1
            let dist = t.map_or(COIN_AHEAD as f64, |t| (d2(t.x, t.y, m.x, m.y) as f64).sqrt());
            let reach = (dist * 0.6).clamp(COIN_AHEAD as f64, COIN_FAR as f64);
            self.coin = Some((m.x + (dx * reach) as i64, m.y + (dy * reach) as i64, tick));
            self.coin_from = (m.x, m.y);
        }
        // the second pump of the shotgun
        if let Some((tid, at, base)) = self.pump {
            if tick >= at {
                self.pump = None;
                let t = sim.get_entity(tid).filter(|t| t.is_alive()).map(|t| { let (x, y) = t.pos(); (x as i64, y as i64) });
                if let (Some((tx, ty)), false) = (t, m.stunned) {
                    if d2(tx, ty, m.x, m.y) <= sq(SHOT_RANGE + 15_000) {
                        self.blast(sim, &m, &all, tx, ty, base * PUMP2_PCT / 100, tick);
                    }
                }
            }
        }
        // the coin at its peak: if no basic attack has shot it yet, he shoots it now (a coin is never wasted while an
        // enemy is near it); the shot counts as a basic attack's worth
        if let Some((cx, cy, t0)) = self.coin {
            let el = tick.saturating_sub(t0);
            if el >= COIN_AUTO_AT && el < COIN_AIR && !m.stunned
                && enemies.iter().any(|e| d2(e.x, e.y, cx, cy) <= sq(COIN_REACH)) {
                let shotgun = m.has("v1_shotgun");
                if self.shoot_coin(sim, &m, &all, m.attack * 60 / 100, shotgun, tick) && shotgun {
                    sim.entity_remove_buff(entity, "v1_shotgun");
                }
            }
        }
        if let Some((cx, cy, t0)) = self.coin {
            let el = tick.saturating_sub(t0);
            if el >= COIN_AIR {
                self.coin = None;   // it dropped: gone
            } else if el % 2 == 0 {
                let k = el as f64 / COIN_AIR as f64;
                let h = (std::f64::consts::PI * k).sin() * COIN_HEIGHT;
                // it flies out to its spot over COIN_FLY ticks while it rises
                let f = (el as f64 / COIN_FLY as f64).min(1.0);
                let (fx0, fy0) = self.coin_from;
                let (px, py) = (fx0 + ((cx - fx0) as f64 * f) as i64, fy0 + ((cy - fy0) as f64 * f) as i64);
                fx_at(sim, &v("coin"), entity, px, py - h as i64, 2);   // one coin: each frame lasts until the next
            }
        }

        // a parry that went through (his reflex below, or a teleport strike that checked try_parry)
        if let Some((name, pv)) = state(&m, "v1_pv:") {
            sim.entity_remove_buff(entity, &name);
            if pv.len() >= 2 {
                self.parried(sim, &m, pv[0].max(0) as usize, pv[1].max(0) as usize, tick);
            }
        }
        // after the window: hunt the attacker (Taunt = he chases and attacks it) while the buffs last
        if let Some((att, at)) = self.hunt {
            if tick >= at {
                self.hunt = None;
                if all.iter().find(|c| c.id == att && c.team != m.team).map_or(false, |a| worth_chase(sim, &m, a, &all)) {
                    let mut taunt = CcV1::of_kind(CcKindV1::Taunt, HUNT_TICKS);
                    taunt.target = att;
                    sim.apply_cc(entity, &taunt);
                }
            }
        }
        if m.has("v1_parried") && tick % 12 == 0 {
            fx_on(sim, &v("parried_on"), entity, entity, 12);
        }

        // parry reflex (only while armed by S2): a dangerous enemy shot about to land, or an enemy that just
        // teleported next to him → the 0.1 s window right then. Not armed: a red "!" for a dangerous shot.
        let armed = m.has("v1_guard") && !m.has("v1_parry_cd") && !m.stunned && !v1_railing(&m);
        let hp_low = m.max_hp > 0 && m.hp * 100 <= m.max_hp * 40;
        let incoming = (0..sim.projectile_count()).filter_map(|i| sim.projectile_at(i)).find(|p| {
            if p.is_end || p.team == m.team { return false; }
            let (px, py) = (p.x as i64, p.y as i64);
            // Omen's Paranoia is a big slow shadow (radius 26000): spotted that much earlier, so the parry window is
            // open when it first touches him (round 52: V1 can parry the blind)
            let paranoia = all.iter().any(|c| c.id == p.caster_id && c.has("omn_para_fly"));
            let reach = if paranoia { INCOMING_R + 26_000 + 8_000 } else { INCOMING_R };
            if d2(px, py, m.x, m.y) > sq(reach) { return false; }
            match all.iter().find(|c| c.id == p.caster_id) {
                Some(c) => d2(px, py, m.x, m.y) < d2(px, py, c.x, c.y),   // on its way to him
                None => false,                                             // minions / towers: never parried
            }
        });
        let mut threat: Option<(usize, usize)> = None;   // (attacker, estimate)
        if let Some(p) = incoming {
            let caster = all.iter().find(|c| c.id == p.caster_id);
            let est = caster.map_or(m.attack, |c| hit_estimate(sim, c, 100));
            // (Paranoia does no damage but blinds: always worth a parry)
            let risky = hp_low || est * 10 >= m.attack * 9 || caster.map_or(true, |c| c.has("gojo_purple_strain") || c.has("frieren_limit") || c.has("omn_para_fly"));
            if risky {
                threat = Some((p.caster_id, est));
            }
        }
        // teleports: an enemy champion that jumped TP_JUMP+ since last tick and landed within TP_NEAR of him
        for e in &enemies {
            if let Some(&(_, px, py)) = self.last_pos.iter().find(|p| p.0 == e.id) {
                if d2(px, py, e.x, e.y) >= sq(TP_JUMP) && d2(e.x, e.y, m.x, m.y) <= sq(TP_NEAR) {
                    threat = Some((e.id, hit_estimate(sim, e, 150)));
                }
            }
        }
        self.last_pos = enemies.iter().map(|e| (e.id, e.x, e.y)).collect();
        if let Some((att, est)) = threat {
            if !m.has("v1_parry") {
                if armed {
                    try_parry(sim, &m, att, est);
                } else if tick >= self.alert_ready {
                    self.alert_ready = tick + 30;
                    fx_on(sim, &v("alert_red"), entity, entity, 30);
                }
            }
        }

        // the shotgun in his hand while it's loaded (after a parry), pointing at the nearest enemy
        if (m.has("v1_shotgun") || self.pump.is_some()) && tick % 2 == 0 {
            let right = nearest(&m, &enemies, 120_000).map_or(true, |t| t.x >= m.x);
            let (side, ox) = if right { ("r", 7_000) } else { ("l", -7_000) };
            fx_at(sim, &v(&format!("shotgun_{side}")), entity, m.x + ox, m.y - 2_000, 2);
        }

        // parry armed (and not low): play forward, a short nudge toward the nearest enemy champion every second
        // after a parry (powered up): keep pressing, every half second, but only toward a fight worth taking
        if m.has("v1_parried") && self.hunt.is_none() && tick % PARRIED_NUDGE_EVERY == 0 {
            if let Some(t) = enemies.iter().filter(|e| worth_chase(sim, &m, e, &all)).min_by_key(|e| (d2(e.x, e.y, m.x, m.y), e.id)) {
                let mut taunt = CcV1::of_kind(CcKindV1::Taunt, ARMED_NUDGE_TICKS);
                taunt.target = t.id;
                sim.apply_cc(entity, &taunt);
            }
        }
        if armed && !hp_low && m.hp * 100 > m.max_hp.max(1) * 50 && self.hunt.is_none() && !m.has("v1_parried") && tick % ARMED_NUDGE_EVERY == 0 {
            if let Some(t) = nearest(&m, &enemies, ARMED_NUDGE_R).filter(|t| worth_chase(sim, &m, t, &all)) {
                let mut taunt = CcV1::of_kind(CcKindV1::Taunt, ARMED_NUDGE_TICKS);
                taunt.target = t.id;
                sim.apply_cc(entity, &taunt);
            }
        }

        // blood pools: shown where they lie; standing on one heals a fixed amount, at most every 8 s
        self.pools.retain(|p| p.2 > tick);
        if tick % 30 == 0 {
            for &(x, y, _) in &self.pools {
                fx_at(sim, &v("blood"), entity, x, y, 30);
            }
        }
        if tick >= self.heal_ready && m.hp < m.max_hp {
            if let Some(i) = self.pools.iter().position(|&(x, y, _)| d2(x, y, m.x, m.y) <= sq(POOL_R)) {
                self.pools.remove(i);
                self.heal_ready = tick + BLOOD_EVERY;
                sim.heal(entity, entity, BLOOD_HEAL);
                fx_on(sim, &v("heal"), entity, entity, 30);
            }
        }
    }
}

// ------------------------------------------------------------------ Darth Vader (passive)

/// The red force-choke wedge from Vader toward (ax, ay), nearest of CHOKE_DIRS drawings.
fn draw_wedge(sim: &mut StableSim<'_>, m: &Champ, ax: f64, ay: f64) {
    let step = 2.0 * std::f64::consts::PI / CHOKE_DIRS as f64;
    let k = ((ay.atan2(ax) / step).round() as i64).rem_euclid(CHOKE_DIRS as i64) as usize;
    let a = k as f64 * step;
    let name = vname(&m.name, "tfm2_starwars_vader", &format!("choke_v{k}"));
    fx_at(sim, &name, m.id, m.x + (a.cos() * CHOKE_LEN as f64 * 0.5) as i64, m.y + (a.sin() * CHOKE_LEN as f64 * 0.5) as i64, 24);
}

const CHOKE_LEN: i64 = 70_000;       // round 37: was 55000
const CHOKE_HALF_ANGLE: f64 = 0.8;   // ~46 degrees either side (round 37: was 0.6)
const CHOKE_DIRS: usize = 16;        // the wedge points right at the target (16 drawings; was 8)
const CHOKE_TARGET_SLACK: i64 = 12_000;   // the aimed target is caught up to this far past the wedge's tip
/// Force choke is a grab and a throw: everyone caught hangs choking for CHOKE_HOLD while Vader stands still in the
/// choke pose (he's rooted, so it stays on screen); then two saber slashes, the second knocking them back.
const CHOKE_HOLD: usize = 60;
const SLASH1_AT: usize = 60;
const SLASH2_AT: usize = 78;
const CHOKE_LIFT: (usize, usize) = (20, 40);
const SLASH1: (usize, usize) = (20, 40);
const SLASH2: (usize, usize) = (30, 60);
const SLASH_REACH: i64 = 70_000;
const THROW: (u64, u64) = (3_200, 18);   // knockback speed, ticks
const MAX_STACKS: usize = 10;
const BA_LINE_MID: f64 = 16_000.0;   // the basic attack's hit box: 32000 x 12000 ahead of him, centred 16000 out
const BA_LINE_TICKS: u64 = 10;

#[derive(Clone, Default)]
pub struct Vader {
    choke_seq: Option<(usize, Vec<usize>)>,   // (start tick, caught ids)
    stacks: usize,
    last_hit: (usize, usize),   // (tick, attacker): one stack per attacker per tick
    saber: Option<Saber>,
}

/// The thrown lightsaber: slow, homing on its target out to SABER_LIFE of flight, then back to Vader.
#[derive(Clone)]
struct Saber {
    x: f64,
    y: f64,
    dir: (f64, f64),
    travelled: i64,
    #[allow(dead_code)]
    target: usize,
    back: bool,
    hit_out: Vec<usize>,
    hit_back: Vec<usize>,
}

const SABER_SPEED: f64 = 2_600.0;      // slower again (was 3500, before that 5000)
const SABER_BACK_SPEED: f64 = 5_000.0;
const SABER_LIFE: i64 = 112_000;       // -20% (was 140000); straight, no homing
const SABER_HIT_R: i64 = 10_000;
const SABER_CATCH_R: i64 = 9_000;

impl StablePassive for Vader {
    fn clone_box(&self) -> Box<dyn StablePassive> {
        Box::new(self.clone())
    }
    fn on_damaged(&mut self, sim: &mut StableSim<'_>, _player: usize, entity: usize, attacker: usize, damage: usize) {
        let tick = sim.tick();
        if damage == 0 || attacker == entity || self.last_hit == (tick, attacker) {
            return;
        }
        self.last_hit = (tick, attacker);
        self.stacks = (self.stacks + 1).min(MAX_STACKS);
    }
    fn on_update(&mut self, sim: &mut StableSim<'_>, _seed: u64, _player: usize, entity: usize) {
        let tick = sim.tick();
        let all = champions(sim);
        let Some(m) = all.iter().find(|c| c.id == entity).cloned() else { return };
        let enemies: Vec<&Champ> = all.iter().filter(|c| c.team != m.team).collect();

        // ---- saber throw
        let sv = |n: &str| vname(&m.name, "tfm2_starwars_vader", n);
        if m.has("vader_throw") && self.saber.is_none() {
            sim.entity_remove_buff(entity, "vader_throw");
            let t = enemies.iter().filter(|e| e.has("vader_saber_target")).min_by_key(|e| (d2(e.x, e.y, m.x, m.y), e.id)).copied()
                .or_else(|| nearest(&m, &enemies, 120_000));
            if let Some(t) = t {
                self.saber = Some(Saber { x: m.x as f64, y: m.y as f64, dir: dir(m.x, m.y, t.x, t.y), travelled: 0, target: t.id, back: false, hit_out: Vec::new(), hit_back: Vec::new() });
            }
        }
        if let Some(mut sb) = self.saber.take() {
            // straight out; on the way back it flies at Vader
            if sb.back {
                sb.dir = dir(sb.x as i64, sb.y as i64, m.x, m.y);
            }
            let speed = if sb.back { SABER_BACK_SPEED } else { SABER_SPEED };
            if tick % 2 == 0 {
                fx_at(sim, &sv("saber_spin"), entity, sb.x as i64, sb.y as i64, 2);   // one spinning blade, redrawn as it moves
            }
            sb.x += sb.dir.0 * speed;
            sb.y += sb.dir.1 * speed;
            sb.travelled += speed as i64;
            let (sx, sy) = (sb.x as i64, sb.y as i64);
            for e in enemies.iter().filter(|e| d2(e.x, e.y, sx, sy) <= sq(SABER_HIT_R)) {
                let list = if sb.back { &mut sb.hit_back } else { &mut sb.hit_out };
                if list.contains(&e.id) {
                    continue;
                }
                list.push(e.id);
                let ad = if sb.back { 25 + m.attack * 45 / 100 } else { 50 + m.attack * 90 / 100 };
                sim.deal_damage(entity, e.id, ad, 0, AttackTypeV1::Skill);
                fx_on(sim, &sv("saber_hit"), entity, e.id, 12);
            }
            if !sb.back && sb.travelled >= SABER_LIFE {
                sb.back = true;
            }
            let caught = sb.back && d2(m.x, m.y, sx, sy) <= sq(SABER_CATCH_R);
            if !caught && sb.travelled < SABER_LIFE * 3 {
                // no basic attacks while the saber is away
                if tick % 10 == 0 {
                    sim.apply_cc(entity, &CcV1::of_kind(CcKindV1::BlockAttack, 12));
                }
                self.saber = Some(sb);
            }
        }

        if m.has("vader_rage_used") {
            sim.entity_remove_buff(entity, "vader_rage_used");
            sim.entity_remove_buff(entity, "vader_full");
            self.stacks = 0;
        }
        // stacks shown as buff copies (one per stack)
        let shown = m.buffs.iter().filter(|b| b.name() == "vader_stack").count();
        if shown != self.stacks {
            sim.entity_remove_buff(entity, "vader_stack");
            for _ in 0..self.stacks {
                sim.add_buff(entity, &BuffV1::named("vader_stack"));
            }
        }
        if self.stacks >= MAX_STACKS {
            if !m.has("vader_full") {
                sim.add_buff(entity, &BuffV1::named("vader_full"));
            }
            // his ult can only be pressed at an enemy in crowd control: hold the nearest one for 2 ticks, twice a second
            if tick % 30 == 0 {
                if let Some(e) = nearest(&m, &enemies, 60_000) {
                    sim.apply_cc(e.id, &CcV1::of_kind(CcKindV1::Bind, 2));
                }
            }
        } else if m.has("vader_full") {
            sim.entity_remove_buff(entity, "vader_full");
        }

        // basic attack: a red line on the ground over the swing's hit box while he winds up (the data deals the hit
        // 10 ticks after this marker)
        if m.has("vader_ba") && !m.has("vader_ba_seen") {
            sim.add_buff(entity, &timed("vader_ba_seen", 6));
            let t = enemies.iter().filter(|e| e.has("vader_ba_target")).min_by_key(|e| (d2(e.x, e.y, m.x, m.y), e.id)).copied()
                .or_else(|| nearest(&m, &enemies, 40_000));
            if let Some(t) = t {
                let (ax, ay) = dir(m.x, m.y, t.x, t.y);
                let k = (((ay.atan2(ax) / (std::f64::consts::PI / 4.0)).round() as i64).rem_euclid(8)) as usize;
                // snap to the drawn direction so the line sits exactly on its art
                let a = k as f64 * std::f64::consts::PI / 4.0;
                fx_at(sim, &sv(&format!("ba_line{k}")), entity, m.x + (a.cos() * BA_LINE_MID) as i64, m.y + (a.sin() * BA_LINE_MID) as i64, BA_LINE_TICKS);
            }
        }

        // Force choke: a V-shaped wedge in front of him (drawn in red); everyone inside is lifted, the knock-up shared
        if m.has("vader_choking") && !m.has("vader_choke_seen") {
            sim.add_buff(entity, &timed("vader_choke_seen", 10));
            let aim = enemies.iter().filter(|e| e.has("vader_choke_aim")).min_by_key(|e| (d2(e.x, e.y, m.x, m.y), e.id)).copied()
                .or_else(|| nearest(&m, &enemies, CHOKE_LEN));
            if let Some(t) = aim {
                // aimed exactly at the target (not snapped to 8 ways); the target itself can't slip out of the edge
                let (ax, ay) = dir(m.x, m.y, t.x, t.y);
                draw_wedge(sim, &m, ax, ay);
                let caught: Vec<&&Champ> = enemies.iter().filter(|e| {
                    let dd = d2(e.x, e.y, m.x, m.y);
                    if e.id == t.id && dd <= sq(CHOKE_LEN + CHOKE_TARGET_SLACK) { return true; }
                    if dd > sq(CHOKE_LEN) { return false; }
                    if dd < sq(8_000) { return true; }
                    let (ex, ey) = dir(m.x, m.y, e.x, e.y);
                    (ex * ax + ey * ay) >= CHOKE_HALF_ANGLE.cos()
                }).collect();
                if !caught.is_empty() {
                    let choke = sv("choke");
                    for e in &caught {
                        // they hang until the second slash throws them
                        fx_on(sim, &choke, entity, e.id, CHOKE_HOLD as u64);
                        sim.deal_damage(entity, e.id, CHOKE_LIFT.0 + m.attack * CHOKE_LIFT.1 / 100, 0, AttackTypeV1::Skill);
                        sim.apply_cc(e.id, &CcV1::of_kind(CcKindV1::Airborne, SLASH2_AT as u64));
                    }
                    // Vader holds the choke through his skill's own action (the 1.6 s 'choke_seq' animation: choke
                    // pose, then two saber swings), so he can't walk off; no self-CC (CC states hid the pose)
                    self.choke_seq = Some((tick, caught.iter().map(|e| e.id).collect()));
                }
            }
        }
        // the two slashes after the choke; the second one throws them back
        if let Some((t0, ids)) = self.choke_seq.clone() {
            let el = tick.saturating_sub(t0);
            // the wedge stays up through the hold, following the one he's choking
            if el > 0 && el < CHOKE_HOLD && el % 20 == 0 {
                if let Some(c) = all.iter().find(|c| ids.first() == Some(&c.id)) {
                    let (ax, ay) = dir(m.x, m.y, c.x, c.y);
                    draw_wedge(sim, &m, ax, ay);
                }
            }
            let targets: Vec<&Champ> = all.iter().filter(|c| ids.contains(&c.id) && d2(c.x, c.y, m.x, m.y) <= sq(SLASH_REACH)).collect();
            if el == SLASH1_AT || el == SLASH2_AT {
                fx_on(sim, &sv("swing"), entity, entity, 12);
                let (b, r) = if el == SLASH1_AT { SLASH1 } else { SLASH2 };
                for e in &targets {
                    sim.deal_damage(entity, e.id, b + m.attack * r / 100, 0, AttackTypeV1::Skill);
                    fx_on(sim, &sv("saber_hit"), entity, e.id, 10);
                    if el == SLASH2_AT {
                        let (dx, dy) = dir(m.x, m.y, e.x, e.y);
                        let mut push = CcV1::of_kind(CcKindV1::ForceMove, THROW.1);
                        push.dx = (dx * 1000.0) as i64;
                        push.dy = (dy * 1000.0) as i64;
                        push.speed = THROW.0;
                        sim.apply_cc(e.id, &push);
                    }
                }
            }
            if el >= SLASH2_AT {
                self.choke_seq = None;
            }
        }
    }
}

// ------------------------------------------------------------------ David Martinez (passive)

const DAVID_ID: &str = "tfm2_cyberpunk_david";
const FULL: i32 = 10_000;            // bars are kept ×100 (0..10000 = 0..100)
const FUEL_DRAIN: i32 = 2_200 / 60;  // 22/s while Sandevistan is on
const FUEL_REGEN: i32 = 800 / 60;    // 8/s while off
const FUEL_BACK: i32 = 1_500;        // after running dry, usable again at 15 (was 30)
const CP_SANDE: i32 = 1_000 / 60;    // +10/s while on
const CP_DRAIN: i32 = 400 / 60;      // −4/s …
const CP_IDLE: usize = 150;          // … after 2.5 s without Sandevistan
const CP_S2: i32 = 1_000;
/// David vs Gojo: Sandevistan is too fast for Infinity. While it's on, his basic attacks ignore Infinity's 30% cut and
/// the INF_SHATTER_HITS-th hit inside INF_SHATTER_WINDOW shatters it (down for its usual 15 s). The gravity smash
/// shatters it too (not inside Gojo's own open domain, where nothing from outside reaches him).
const INF_SHATTER_HITS: u32 = 3;
const INF_SHATTER_WINDOW: usize = 180;
const MINI_GRAV_R: i64 = 25_000;     // the dash slam's little gravity
const MINI_GRAV_PULL: (usize, usize) = (2_400, 10);
/// Sandevistan versions: v2 from 80 cyberpsychosis, v3 in full cyberpsychosis. Shown as red / blue fire on his bar.
/// Round 69 (Rian: "he's been off"): they used to trade attack for speed (v2 -15%, v3 -25% attack), so the meter only
/// made him faster, never stronger. Now both add attack, attack speed and lifesteal, and v3 also cuts damage taken
/// (full cyberpsychosis drains 7% max HP a second).
const SV2: (i32, i32, i32) = (25, 35, 10);   // move speed, attack speed, attack (%)
const SV3: (i32, i32, i32) = (50, 60, 20);
const SV_VAMP: [i32; 2] = [8, 15];           // lifesteal (v2, v3)
const SV3_REDUCE: usize = 15;                // v3: damage taken -15%
const SV2_AT: i32 = 8_000;
const CP_RUSH: i32 = 500;          // the Sandevistan rush of the 70+ grab
const CP_ULT_AREA: i32 = 2_500;      // gravity on an area
const CP_ULT_LOCK: i32 = 4_000;      // gravity locked on one target
const SANDE_MS: i32 = 40;            // Sandevistan speed (was 60 / 80)
const SANDE_AS: i32 = 50;
const SANDE_SLOW_R: i64 = 30_000;
const GRAV_R: i64 = 30_000;
const GRAV_DELAY: usize = 60;
const GRAV_TICKS: usize = 150;         // 2.5 s (was 3 s)
/// The area cast (no lock-on; round 36): bigger, faster to conjure, shorter pin.
const GRAV_R_AREA: i64 = 40_000;
const GRAV_DELAY_AREA: usize = 36;     // 0.6 s
const GRAV_TICKS_AREA: usize = 90;     // 1.5 s
const ESCAPE_FUEL: i32 = 2_000;
const ESCAPE_CP_MAX: i32 = 6_000;     // "safe" cyberpsychosis for an escape
const ESCAPE_HP_PCT: usize = 45;
const TRAVEL_FUEL: i32 = 8_000;
const TRAVEL_CP_MAX: i32 = 2_500;
const TRAVEL_CLEAR_R: i64 = 150_000;
const PSYCHO_DRAIN_PER_HALF_S: usize = 35;   // ‰ of max HP every 30 ticks = 7% per second

/// Break Gojo's Infinity: drop its shield (the buff lasts WithShield, and his data marks it as broken on his next action).
fn shatter_infinity(sim: &mut StableSim<'_>, gojo: usize, caster: usize, fx: &str) {
    sim.entity_clear_shield(gojo);
    sim.entity_remove_buff(gojo, "gojo_infinity");
    sim.play_view_effect(fx, caster, &InputTargetV1::target(gojo), 0, 0, 18);
}

#[derive(Clone)]
pub struct David {
    cp: i32,
    fuel: i32,
    dry: bool,
    idle: usize,
    psycho: bool,
    tier: i32,
    sv: i32,
    bar: i32,
    fuel_bar: i32,
    grav: Option<(i64, i64, usize)>,
    grav_caught: Vec<usize>,
    grav_lock: Option<usize>,
    inf_hits: (usize, u32, usize),   // (Gojo id, Sandevistan hits on his Infinity, last hit tick)
    /// recent positions (tick, x, y) for the Sandevistan afterimages
    trail: Vec<(usize, i64, i64)>,
}

impl Default for David {
    fn default() -> Self {
        David { cp: 0, fuel: FULL, dry: false, idle: 0, psycho: false, tier: -1, sv: 1, bar: -1, fuel_bar: -1, grav: None, grav_caught: Vec::new(), grav_lock: None, inf_hits: (usize::MAX, 0, 0), trail: Vec::new() }
    }
}

impl David {
    fn reset(&mut self) {
        *self = David::default();
    }
}

impl StablePassive for David {
    fn clone_box(&self) -> Box<dyn StablePassive> {
        Box::new(self.clone())
    }
    fn on_spawn(&mut self, _sim: &mut StableSim<'_>, _player: usize, _entity: usize) {
        self.reset();
    }
    fn on_dead(&mut self, _sim: &mut StableSim<'_>, _player: usize) {
        self.reset();
    }
    fn on_attack(&mut self, sim: &mut StableSim<'_>, _player: usize, entity: usize, target: usize, damage: &mut usize) {
        let tick = sim.tick();
        let all = champions(sim);
        let Some(m) = all.iter().find(|c| c.id == entity) else { return };
        let allies: Vec<&Champ> = all.iter().filter(|c| c.team == m.team && c.id != entity).collect();
        // Sandevistan vs Infinity (see INF_SHATTER_HITS)
        let infinity = all.iter().find(|c| c.id == target && c.team != m.team && c.has("gojo_infinity") && !c.has("gojo_void_active")).map(|c| c.id);
        if let (Some(g), true) = (infinity, m.has("dv_sande")) {
            *damage = *damage * 10 / 7;   // undoes Infinity's 30% basic-attack cut
            let (id, n, last) = self.inf_hits;
            let n = if id == g && tick <= last + INF_SHATTER_WINDOW { n + 1 } else { 1 };
            self.inf_hits = (g, n, tick);
            if n >= INF_SHATTER_HITS {
                self.inf_hits = (usize::MAX, 0, 0);
                shatter_infinity(sim, g, entity, &vname(&m.name, DAVID_ID, "slam"));
            }
        }
        if self.psycho {
            // lost it: lower damage, and it spills onto a teammate nearby
            *damage = *damage * 80 / 100;
            if let Some(a) = nearest(m, &allies, 40_000) {
                if roll(tick, entity, target) < 40 {
                    sim.deal_damage_raw(entity, a.id, *damage / 2, 0, AttackTypeV1::Skill);
                }
            }
        } else if self.cp >= 7_000 && roll(tick, entity, target) < 25 {
            // hallucination: the shot goes into a teammate (or himself) instead
            let hurt = *damage;
            *damage = 0;
            match nearest(m, &allies, 40_000) {
                Some(a) if roll(tick + 7, target, entity) < 60 => sim.deal_damage_raw(entity, a.id, hurt, 0, AttackTypeV1::Skill),
                _ => sim.deal_damage_raw(entity, entity, hurt / 2, 0, AttackTypeV1::Skill),
            }
        }
    }
    fn on_update(&mut self, sim: &mut StableSim<'_>, _seed: u64, _player: usize, entity: usize) {
        let tick = sim.tick();
        let all = champions(sim);
        let Some(m) = all.iter().find(|c| c.id == entity).cloned() else { return };
        let enemies: Vec<&Champ> = all.iter().filter(|c| c.team != m.team).collect();
        let v = |n: &str| vname(&m.name, DAVID_ID, n);

        // markers from his data file
        if m.has("dv_s2") {
            sim.entity_remove_buff(entity, "dv_s2");
            self.cp += CP_S2;
        }
        // 70+: the Sandevistan rush grab costs the ability's 10 plus 5 for the rush
        if m.has("dv_s2r") {
            sim.entity_remove_buff(entity, "dv_s2r");
            self.cp += CP_S2 + CP_RUSH;
        }
        // 35-69 cyber dash slam: a little gravity well where he lands pulls the enemies around him in
        if m.has("dv_minigrav") && !m.has("dv_minigrav_seen") {
            sim.add_buff(entity, &timed("dv_minigrav_seen", 8));
            fx_at(sim, &v("grav_mini"), entity, m.x, m.y, 24);
            for e in enemies.iter().filter(|e| d2(e.x, e.y, m.x, m.y) <= sq(MINI_GRAV_R) && d2(e.x, e.y, m.x, m.y) > sq(4_000)) {
                sim.entity_pull(entity, e.id, MINI_GRAV_PULL.0, MINI_GRAV_PULL.1);
            }
        }
        if m.has("dv_selfboom") {
            sim.entity_remove_buff(entity, "dv_selfboom");
            sim.deal_damage_raw(entity, entity, 40 + m.attack * 40 / 100, 0, AttackTypeV1::Skill);
        }
        if m.has("dv_grav") && !m.has("dv_grav_seen") {
            sim.add_buff(entity, &timed("dv_grav_seen", 10));
            let t = enemies.iter().filter(|e| e.has("dv_grav_target")).min_by_key(|e| (d2(e.x, e.y, m.x, m.y), e.id)).copied()
                .or_else(|| nearest(&m, &enemies, 100_000));
            if let Some(t) = t {
                // two ways to cast it. Other enemies near the target → an area on that spot (cheaper, +25).
                // A lone target that's worth it (low HP, or hits hard) → locked on: the crosshair follows them and the
                // smash lands wherever they are after the second (can't be walked out of; +40).
                let others = enemies.iter().filter(|e| e.id != t.id && d2(e.x, e.y, t.x, t.y) <= sq(GRAV_R)).count();
                let low = t.max_hp > 0 && t.hp * 100 <= t.max_hp * 55;
                let threat = t.attack + magic_of(sim, t.id) >= m.attack;
                if others == 0 && (low || threat) {
                    self.cp += CP_ULT_LOCK;
                    self.grav_lock = Some(t.id);
                    self.grav = Some((t.x, t.y, tick));
                    fx_on(sim, &v("crosshair_lock"), entity, t.id, GRAV_DELAY as u64);
                } else {
                    self.cp += CP_ULT_AREA;
                    self.grav_lock = None;
                    self.grav = Some((t.x, t.y, tick));
                    fx_at(sim, &v("crosshair_area"), entity, t.x, t.y, GRAV_DELAY_AREA as u64);
                }
            }
        }

        // Sandevistan fuel + cyberpsychosis
        let mut sande = m.has("dv_sande");
        if self.psycho && !sande {
            let mut b = timed("dv_sande", 120);
            b.move_speed_mult = SANDE_MS;
            b.attack_speed_mult = SANDE_AS;
            sim.add_buff(entity, &b);
            sande = true;
        }
        // running away: low HP, an enemy on his heels, fuel in the tank and cyberpsychosis still at a safe level →
        // he triggers Sandevistan himself to get away
        if !sande && !self.dry && !self.psycho && self.fuel >= ESCAPE_FUEL && self.cp < ESCAPE_CP_MAX
            && m.max_hp > 0 && m.hp * 100 <= m.max_hp * ESCAPE_HP_PCT {
            if let (Some(e), Some(&(_, px, py))) = (nearest(&m, &enemies, 50_000), self.trail.iter().find(|&&(t, _, _)| t + 6 >= tick)) {
                if d2(e.x, e.y, m.x, m.y) > d2(e.x, e.y, px, py) {   // moving away from them
                    let mut b = timed("dv_sande", 60);
                    b.move_speed_mult = SANDE_MS;
                    b.attack_speed_mult = SANDE_AS;
                    sim.add_buff(entity, &b);
                    sande = true;
                }
            }
        }
        // round 59: getting around (the jungle, a rotation) with no enemy champion near, on the move, fuel nearly full
        // and the meter low → a 1 s burst of Sandevistan to get there faster
        if !sande && !self.dry && !self.psycho && self.fuel >= TRAVEL_FUEL && self.cp < TRAVEL_CP_MAX
            && nearest(&m, &enemies, TRAVEL_CLEAR_R).is_none() {
            if let Some(&(_, px, py)) = self.trail.iter().find(|&&(t, _, _)| t + 6 <= tick) {
                if d2(px, py, m.x, m.y) >= sq(3_000) {
                    let mut b = timed("dv_sande", 60);
                    b.move_speed_mult = SANDE_MS;
                    b.attack_speed_mult = SANDE_AS;
                    sim.add_buff(entity, &b);
                    sande = true;
                }
            }
        }
        if sande {
            self.idle = 0;
            if !self.psycho {
                self.fuel -= FUEL_DRAIN;
                self.cp += CP_SANDE;
                if self.fuel <= 0 {
                    self.fuel = 0;
                    self.dry = true;
                    sim.entity_remove_buff(entity, "dv_sande");
                }
            }
            if tick % 10 == 0 {
                for e in enemies.iter().filter(|e| d2(e.x, e.y, m.x, m.y) <= sq(SANDE_SLOW_R)) {
                    sim.entity_remove_buff(e.id, "dv_timeslow");
                    let mut s = timed("dv_timeslow", 15);
                    s.move_speed_mult = -35;
                    s.attack_speed_mult = -25;
                    sim.add_buff(e.id, &s);
                }
            }
        } else {
            self.fuel = (self.fuel + FUEL_REGEN).min(FULL);
            self.idle += 1;
            if self.idle > CP_IDLE && !self.psycho {
                self.cp -= CP_DRAIN;
            }
        }
        if self.dry && self.fuel >= FUEL_BACK {
            self.dry = false;
        }
        self.cp = self.cp.clamp(0, FULL);
        if self.cp >= FULL {
            self.psycho = true;
        }
        // Sandevistan version (only while it's on): v2 from 80, v3 in full cyberpsychosis
        let ver = if !sande { 1 } else if self.psycho { 3 } else if self.cp >= SV2_AT { 2 } else { 1 };
        if ver != self.sv || (ver >= 2 && !m.has(&format!("dv_sv{ver}"))) {
            self.sv = ver;
            sim.entity_remove_buff(entity, "dv_sv2");
            sim.entity_remove_buff(entity, "dv_sv3");
            if ver >= 2 {
                let (ms, asp, atk) = if ver == 3 { SV3 } else { SV2 };
                let mut b = BuffV1::named(&format!("dv_sv{ver}"));
                b.move_speed_mult = ms;
                b.attack_speed_mult = asp;
                b.attack_mult = atk;
                b.vamp = SV_VAMP[(ver - 2) as usize];
                if ver == 3 { b.damaged_reduce = SV3_REDUCE; }
                sim.add_buff(entity, &b);
            }
        }
        let want_nofuel = self.dry && !self.psycho;
        if want_nofuel != m.has("dv_nofuel") {
            if want_nofuel { sim.add_buff(entity, &BuffV1::named("dv_nofuel")); } else { sim.entity_remove_buff(entity, "dv_nofuel"); }
        }

        // ability 2 tier + caution (slower cooldowns the higher the bar)
        // ability 2: pistol barrage 0-34, cyber dash slam 35-69, Sandevistan rush grab 70+ (and in full cyberpsychosis)
        let tier = if self.psycho || self.cp >= 7_000 { 2 } else if self.cp >= 3_500 { 1 } else { 0 };
        let key = tier * 10 + self.psycho as i32;
        if key != self.tier {
            self.tier = key;
            for (t, n) in [(0, "dv_t0"), (1, "dv_t1"), (2, "dv_t2")] {
                if t == tier { sim.add_buff(entity, &BuffV1::named(n)); } else { sim.entity_remove_buff(entity, n); }
            }
            sim.entity_remove_buff(entity, "dv_t3");
            sim.entity_remove_buff(entity, "dv_caution");
            if tier >= 2 {
                let mut c = BuffV1::named("dv_caution");
                c.skill_cooldown_mult = if self.psycho { -50 } else { -25 };
                sim.add_buff(entity, &c);
            }
        }
        // the cyberpsychosis bar above his head: one of dv_bar0..dv_bar10 (the icon beside it is the dv_t* buff)
        let bar = (self.cp + 500) / 1_000;
        if bar != self.bar || !m.has(&format!("dv_bar{bar}")) {
            if self.bar >= 0 {
                sim.entity_remove_buff(entity, &format!("dv_bar{}", self.bar));
            }
            self.bar = bar;
            sim.add_buff(entity, &BuffV1::named(&format!("dv_bar{bar}")));
        }
        // the fuel bar under it: dv_fuel0..10 while usable, dv_fueldry (flashing) while recharging after running dry
        let fb = if self.dry && !self.psycho { 11 } else { (self.fuel + 500) / 1_000 };
        let fname = |b: i32| if b == 11 { "dv_fueldry".to_string() } else { format!("dv_fuel{b}") };
        if fb != self.fuel_bar || !m.has(&fname(fb)) {
            if self.fuel_bar >= 0 {
                sim.entity_remove_buff(entity, &fname(self.fuel_bar));
            }
            self.fuel_bar = fb;
            sim.add_buff(entity, &BuffV1::named(&fname(fb)));
        }
        if !m.has(&format!("dv_t{tier}")) {
            sim.add_buff(entity, &BuffV1::named(&format!("dv_t{tier}")));
        }
        // Sandevistan afterimages: a tinted copy of him where he was 6 ticks ago, every 4 ticks, colours cycling
        self.trail.push((tick, m.x, m.y));
        self.trail.retain(|&(t, _, _)| t + 12 >= tick);
        if (sande || m.has("dv_rush")) && tick % 4 == 0 {
            if let Some(&(_, px, py)) = self.trail.iter().find(|&&(t, _, _)| t + 6 >= tick) {
                if d2(px, py, m.x, m.y) > sq(2_500) {
                    let side = if m.x < px { "l" } else { "r" };
                    let col = ["g", "m", "c"][(tick / 4) % 3];
                    fx_at(sim, &v(&format!("after_{col}_{side}")), entity, px, py, 9);
                }
            }
        }
        // psycho: visual, and his HP drains 7% of max HP per second until he dies
        if self.psycho {
            if !m.has("dv_psycho") {
                sim.add_buff(entity, &BuffV1::named("dv_psycho"));
            }
            if tick % 30 == 0 {
                sim.deal_damage_raw(entity, entity, (m.max_hp * PSYCHO_DRAIN_PER_HALF_S / 1000).max(1), 0, AttackTypeV1::Skill);
            }
        }

        // Gravity projection: after the crosshair second ONE smash-down hits the area: whoever is in it right then
        // (both teams) is pinned for 3 s; the field doesn't linger, so anyone walking in later is fine
        if let Some((mut x, mut y, t0)) = self.grav {
            let el = tick.saturating_sub(t0);
            let (grav_r, grav_delay, grav_ticks) = if self.grav_lock.is_some() { (GRAV_R, GRAV_DELAY, GRAV_TICKS) }
                else { (GRAV_R_AREA, GRAV_DELAY_AREA, GRAV_TICKS_AREA) };
            if let Some(lid) = self.grav_lock {
                if el <= grav_delay {
                    if let Some(c) = all.iter().find(|c| c.id == lid) { x = c.x; y = c.y; self.grav = Some((x, y, t0)); }
                }
            }
            // let others see the coming smash: "dvz:<x>:<y>:<smash tick>:<locked id + 1>" (see danger_zones)
            if let Some((n, _)) = state(&m, "dvz:") { sim.entity_remove_buff(entity, &n); }
            if el < grav_delay {
                sim.add_buff(entity, &timed(&format!("dvz:{x}:{y}:{}:{}", t0 + grav_delay, self.grav_lock.map_or(0, |l| l + 1)), 3));
            }
            if el == grav_delay {
                fx_at(sim, &v(if self.grav_lock.is_some() { "gravity" } else { "gravity_area" }), entity, x, y, 30);
                self.grav_caught = all.iter().filter(|c| c.id != entity && d2(c.x, c.y, x, y) <= sq(grav_r)).map(|c| c.id).collect();
                // gravity x100 crushes Infinity
                for c in all.iter().filter(|c| self.grav_caught.contains(&c.id) && c.team != m.team && c.has("gojo_infinity") && !c.has("gojo_void_active")) {
                    shatter_infinity(sim, c.id, entity, &v("slam"));
                }
            }
            if el >= grav_delay && el < grav_delay + grav_ticks {
                let key = format!("dvg{entity}");
                for c in all.iter().filter(|c| self.grav_caught.contains(&c.id)) {
                    freeze(sim, c, &key, tick);
                }
            } else if el >= grav_delay + grav_ticks {
                self.grav = None;
                self.grav_caught.clear();
            }
        }

        // lost it: go for the nearest enemy champion instead of anything else (no backing off, no recall)
        if self.psycho && tick % 30 == 0 {
            if let Some(t) = nearest(&m, &enemies, 200_000) {
                let mut taunt = CcV1::of_kind(CcKindV1::Taunt, 40);
                taunt.target = t.id;
                sim.apply_cc(entity, &taunt);
            }
        }
    }
}
