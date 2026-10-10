//! Native helpers for the Skill Lab champions (the editor-made data mods: tfm2_custom, tfm2_jjk,
//! tfm2_jojo, tfm2_cyberpunk, tfm2_ultrakill, tfm2_starwars, tfm2_frieren, tfm2_blockcraft,
//! tfm2_valorant, tfm2_toon). Per-champion modules: batch2 (DIO, David, V1, Vader, Frieren),
//! steve, valorant (Omen), scribble, plus flash (every champion) and tactics (the Map tab plans).
//!
//! Input AI rule (since 0.2): never inject or hold casts. Overriding the game's own casts broke its
//! planner (game-ai plan_legacy panics, champions freezing), so the input AI only retargets MOVE
//! inputs (wall detours, smoke checks, team calls) and leaves casts and attacks to the game.
//!
//! Gojo's Unlimited Void rules run through a match hook that runs once per tick
//! after the game's own tick. The data side (Gojo's ult) marks an open domain by giving Gojo the
//! buff `gojo_void_active` for as long as the domain lasts. While that buff is up:
//!   * the domain centre is where Gojo stood when it opened (kept in a buff name, no hidden state);
//!   * the ONE allied champion closest to Gojo inside the domain is his guest: free to move, and
//!     guarded like Gojo (no damage from basic attacks / skills, immune to crowd control);
//!   * everyone else who was inside when it opened is trapped: stunned the whole time, and any
//!     crowd-control immunity they have (e.g. Minato's Kurama burst) is stripped first;
//!   * those caught are frozen on the spot (anything that moves them is undone) and sealed;
//!   * once conjured the edge is a wall for everyone outside: whoever reaches it stops right there
//!     (walking, dashing and teleporting in all fail).
//!
//! Everything is read from the match and written back through the stable API, deterministically.

use mod_api_stable::{
    declare_stable_mod, AttackTypeV1, BuffDurationV1, BuffV1, CastingTargetV1, CcKindV1, CcV1, InputTargetV1, LogLevel,
    GameModeKindV1, ProjectileMoveKindV1, ProjectileSpawnV1, StableEffectType, StableHost, StableJsonDoc, StableMapCustomizer, StatV1, UnitAttackV1,
    StableMatchHook, StableMod, StablePassive, StableSim, StablePlayerAi, StableAiContext, InputV1, InputKindV1,
};

const MOD_ID: &str = "tfm2_custom_ai";

mod batch2;
mod perf;
mod steve;
mod valorant;
mod tactics;
mod flash;
mod scribble;
mod levi;
mod gundam;
mod press;
mod isliid;
mod coder;
mod coder_code;
mod mastery;
mod unified_theory;
mod unified_theory_data;
mod unified_theory_math;

/// Domain radius in world units (Gojo preset `domainRadius`).
const DOMAIN_R: i64 = 76_000;   // was 60000; matches the domain art (80 px at 950 units/px)
/// The seal starts this wide around Gojo on the press (Gojo preset `growStart`).
const GROW_START: i64 = 8_000;
/// Edge colours for debug_draw_circle (0xRRGGBBAA).
const EDGE_WARN: u32 = 0xFF_5A_3C_FF;
const EDGE_WARN_SOFT: u32 = 0xFF_5A_3C_70;
const EDGE_WALL: u32 = 0xB9_F3_FF_FF;
const EDGE_WALL_SOFT: u32 = 0xC7_7D_FF_A0;
const EDGE_FULL_FAINT: u32 = 0xB9_F3_FF_50;
/// Trapped champions caught right on the edge are frozen this far inside it.
const WALL_INSET: i64 = 2_000;
/// Outsiders stop this far outside the edge.
const WALL_OUT: i64 = 1_500;
/// A frozen champion may drift this much before being put back (no jitter from rounding).
const FREEZE_SLACK: i64 = 400;
/// … and is only put back from up to this far away (a respawn at base is never pulled back in).
const FREEZE_CATCH: i64 = 120_000;
const ACTIVE: &str = "gojo_void_active";

/// Mod Power (round 53, Rian; per champion since round 55): a permanent buff in percent (attack, magic power, max HP,
/// armour, magic resist, move speed, attack speed) on the Skill Lab champions, matched by the end of their id.
/// Round 75 (Rian): a full mod line-up had the setups but not the damage, and lost to minion pushing, so every mod
/// champion now sits clearly above its base class instead of at its median. Being native, none of this is touched by
/// the game's own balance patches (those only change a career's copy of the data). Anyone missing = none.
const MOD_POWER: &[(&str, [i32; 7])] = &[
    //               atk  ap  hp  def  mr  ms  as
    ("_minato",     [ 25,  0, 10, 10, 10,  0, 10]),   // round 75 (was nothing)
    ("_gojo",       [  0, 35, 10, 10, 10,  0,  0]),   // was 0/15/5/5/5
    ("_dio",        [ 45,  0, 15, 10, 10,  0, 10]),   // was 25
    ("_david",      [ 40,  0, 10,  5,  5,  0, 10]),   // was 25
    ("_v1",         [ 45,  0, 10, 10, 10,  0, 10]),   // was 30/0/5/10/10
    ("_vader",      [ 35,  0, 20, 15, 15,  0,  0]),   // was 5
    ("_frieren",    [  0, 40, 15, 15, 10,  0, 10]),   // was 0/25/10/15/10 (round 71)
    ("_steve",      [ 30,  0, 15, 15, 15,  0,  0]),   // was nothing; his skills scale on max HP
    ("_omen",       [ 35,  0, 10, 10, 10,  0, 15]),   // was 10/0/10/10/10
    ("_scribble",   [  0, 30, 10,  5, 10,  0,  0]),   // was 0/10/5/0/5
    ("_levi",       [ 30,  0, 10, 10, 10,  0, 10]),   // round 77
    ("_emperor",    [ 25,  0, 10, 10, 10,  0, 10]),
    ("_aegis_zero", [ 20,  0, 15, 15, 15,  0,  0]),   // round 88: a tank / support, above the base tanks
    ("_unified_theory", [0, 0, 0, 0, 0, 0, 0]), // Explicit neutral scientific baseline
    ("_coder",      [  0, 40, 30, 15, 15,  5,  0]),   // round 107: buffed well above the base mages
];
const MOD_POWER_BUFF: &str = "mod_power";

fn mod_power_of(name: &str) -> Option<[i32; 7]> {
    if !name.starts_with("tfm2_") { return None; }
    MOD_POWER.iter().find(|(end, _)| name.ends_with(end)).map(|(_, v)| *v).filter(|v| v.iter().any(|x| *x != 0))
}

fn mod_power(sim: &mut StableSim<'_>, all: &[Champ]) {
    for c in all.iter().filter(|c| !c.has(MOD_POWER_BUFF)) {
        let Some([atk, ap, hp, def, mr, ms, aspd]) = mod_power_of(&c.name) else { continue };
        let mut b = BuffV1::named(MOD_POWER_BUFF);
        b.attack_mult = atk;
        b.magic_power_mult = ap;
        b.hp_mult = hp;
        b.defence_mult = def;
        b.magic_resistance_mult = mr;
        b.move_speed_mult = ms;
        b.attack_speed_mult = aspd;
        sim.add_buff(c.id, &b);
        // start at full health with the bigger pool
        if let Some(e) = sim.get_entity(c.id) { let (_, mx) = e.hp(); if c.hp == c.max_hp { sim.entity_set_hp(c.id, mx); } }
    }
}

#[derive(Clone)]
struct Champ {
    id: usize,
    team: usize,
    x: i64,
    y: i64,
    buffs: Vec<BuffV1>,
    stunned: bool,
    #[allow(dead_code)]
    pushed: bool,
    hp: usize,
    max_hp: usize,
    attack: usize,
    name: String,
}

impl Champ {
    fn buff(&self, name: &str) -> Option<&BuffV1> {
        self.buffs.iter().find(|b| b.name() == name)
    }
    fn has(&self, name: &str) -> bool {
        self.buff(name).is_some()
    }
}

fn d2(ax: i64, ay: i64, bx: i64, by: i64) -> i128 {
    let (dx, dy) = ((ax - bx) as i128, (ay - by) as i128);
    dx * dx + dy * dy
}
fn sq(v: i64) -> i128 {
    (v as i128) * (v as i128)
}

const FARM_BUFF: &str = "mod_farm";
const FARM_R: i64 = 110_000;

/// Round 75: farm mode also beside a wave. A mod champion with WAVE_MIN+ enemy minions (or camp monsters) within
/// WAVE_NEAR and no visible enemy champion within WAVE_CHAMP_R clears the wave with the plain versions of its skills,
/// as base champions do, instead of holding them for a laner who isn't in reach anyway.
const WAVE_MIN: usize = 3;
const WAVE_NEAR: i64 = 45_000;
const WAVE_CHAMP_R: i64 = 40_000;

/// Living non-champion, non-tower units: (id, team, x, y). Lane minions, camp monsters and summons.
pub(crate) fn units(sim: &StableSim<'_>) -> Vec<(usize, usize, i64, i64)> {
    let mut out = Vec::new();
    for i in 0..sim.entity_count() {
        let Some(e) = sim.entity_at(i) else { continue };
        if !e.is_alive() || e.is_champion() || e.is_tower() { continue; }
        let (x, y) = e.pos();
        out.push((e.id(), e.team(), x as i64, y as i64));
    }
    out
}

/// Round 75 (Rian): native skills hit minions too. Deals `ad` / `ap` (as Skill damage) to every enemy non-champion,
/// non-tower unit within `r` of (x, y): lane minions, camp monsters, enemy summons. A unit is hit at most once per
/// caster per 2 ticks, so a cast that hits three champions doesn't hit the wave three times.
pub(crate) fn wave_at(sim: &mut StableSim<'_>, caster: usize, x: i64, y: i64, r: i64, ad: usize, ap: usize) {
    let Some(team) = sim.get_entity(caster).map(|e| e.team()) else { return };
    let mark = format!("wsp{caster}");
    for (id, t, ux, uy) in units(sim) {
        if t == team || d2(ux, uy, x, y) > sq(r) { continue; }
        let marked = sim.get_entity(id).map_or(true, |e| (0..e.buff_count()).filter_map(|b| e.buff_at(b)).any(|b| b.name() == mark));
        if marked { continue; }
        sim.add_buff(id, &timed(&mark, 2));
        sim.deal_damage(caster, id, ad, ap, AttackTypeV1::Skill);
    }
}

/// `wave_at` around a unit that was just hit (a champion, usually; if it was a minion itself, it isn't hit twice).
pub(crate) fn wave_near(sim: &mut StableSim<'_>, caster: usize, target: usize, r: i64, ad: usize, ap: usize) {
    let Some((x, y)) = sim.get_entity(target).map(|e| e.pos()) else { return };
    sim.add_buff(target, &timed(&format!("wsp{caster}"), 2));
    wave_at(sim, caster, x as i64, y as i64, r, ad, ap);
}

fn farm_mode(sim: &mut StableSim<'_>, all: &[Champ]) {
    let mut wave: Option<Vec<(usize, usize, i64, i64)>> = None;
    for c in all.iter().filter(|c| c.name.starts_with("tfm2_")) {
        let foe_within = |r: i64| all.iter().any(|e| e.team != c.team && sim.is_visible(c.team, e.id) && d2(e.x, e.y, c.x, c.y) <= sq(r));
        let mut calm = !foe_within(FARM_R);
        if !calm && !foe_within(WAVE_CHAMP_R) {
            let w = wave.get_or_insert_with(|| units(sim));
            calm = w.iter().filter(|u| u.1 != c.team && d2(u.2, u.3, c.x, c.y) <= sq(WAVE_NEAR)).count() >= WAVE_MIN;
        }
        if calm {
            sim.entity_remove_buff(c.id, FARM_BUFF);
            sim.add_buff(c.id, &timed(FARM_BUFF, 14));
        } else if c.has(FARM_BUFF) {
            sim.entity_remove_buff(c.id, FARM_BUFF);
        }
    }
}

/// A key for "this simulation" (round 61): native memory that must last across ticks (Flash cooldowns, smoke reveals,
/// Steve's wall positions) is kept per key. It must differ between two simulations of the same match running side by
/// side (the precomputed "server" sim and the "live" one): keyed by the seed or the line-up they shared one memory,
/// which made the two sims diverge (log.log: "server/live simulation diverged") and wiped Flash's cooldown.
pub(crate) fn match_key(sim: &StableSim<'_>) -> u64 {
    sim.instance() as u64
}

fn champions(sim: &StableSim<'_>) -> Vec<Champ> {
    let mut out = Vec::new();
    for i in 0..sim.champion_count() {
        let id = sim.champion_id_at(i);
        let Some(e) = sim.get_entity(id) else { continue };
        if !e.is_alive() {
            continue;
        }
        let (x, y) = e.pos();
        let (hp, max_hp) = e.hp();
        let attack = e.stat().attack;
        let name = e.name().unwrap_or_default();
        let mut buffs = Vec::new();
        for b in 0..e.buff_count() {
            if let Some(buff) = e.buff_at(b) {
                buffs.push(buff);
            }
        }
        let (mut stunned, mut pushed) = (false, false);
        for c in 0..e.cc_count() {
            if let Some(cc) = e.cc_at(c) {
                if cc.kind == CcKindV1::Stun.code() {
                    stunned = true;
                }
                if cc.kind == CcKindV1::ForceMove.code() {
                    pushed = true;
                }
            }
        }
        out.push(Champ { id, team: e.team(), x: x as i64, y: y as i64, buffs, stunned, pushed, hp, max_hp, attack, name });
    }
    out
}

/// A view effect at a point / on a unit (named `<champion id>_<tag>` in the data's view_effects).
/// Round 91: the native version, written in the game log and the champions' logs (gundam_log.txt / isliid_log.txt) so a
/// game shows which build ran.
pub(crate) const VERSION: &str = "0.10.30";

static LOGGED: std::sync::Mutex<Option<std::collections::HashSet<String>>> = std::sync::Mutex::new(None);

/// mods/tfm2_custom_ai next to the game exe. Round 93: looked up once (the exe doesn't move while the game runs; every
/// log line and mastery write used to ask the OS again).
pub(crate) fn mod_dir() -> Option<std::path::PathBuf> {
    static DIR: std::sync::OnceLock<Option<std::path::PathBuf>> = std::sync::OnceLock::new();
    DIR.get_or_init(|| std::env::current_exe().ok().and_then(|e| e.parent().map(|d| d.join("mods").join(MOD_ID)))).clone()
}

/// One line in `file` next to the DLL (mods/tfm2_custom_ai); `key` keeps the same event from being written twice (the
/// game runs two simulations of each match).
pub(crate) fn mod_log(sim: &StableSim<'_>, file: &str, key: &str, line: &str) {
    use std::io::Write;
    let k = format!("{file}.{:x}.{key}", sim.seed());
    if let Ok(mut g) = LOGGED.lock() {
        let s = g.get_or_insert_with(std::collections::HashSet::new);
        if s.len() > 20_000 { s.clear(); }
        if !s.insert(k) { return; }
    }
    let Some(dir) = mod_dir() else { return };
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(dir.join(file)) {
        let _ = writeln!(f, "game {:x} tick {}: {line}", sim.seed(), sim.tick());
    }
}

pub(crate) fn fx_point(sim: &mut StableSim<'_>, name: &str, caster: usize, x: i64, y: i64, time: u64) -> bool {
    perf::note_fx(caster, sim.tick());
    sim.play_view_effect(name, caster, &InputTargetV1::pos(x.max(0) as u64, y.max(0) as u64), 0, 0, time)
}
pub(crate) fn fx_unit(sim: &mut StableSim<'_>, name: &str, caster: usize, target: usize, time: u64) -> bool {
    perf::note_fx(caster, sim.tick());
    sim.play_view_effect(name, caster, &InputTargetV1::target(target), 0, 0, time)
}

/// Round 88: Isliid's flying swords are cosmetic projectiles (his damage is native): no reflex (DIO's guard, V1's
/// parry, Minato's dodge, Flash) should treat them as incoming shots.
pub(crate) fn cosmetic_shot<'a>(mut champs: impl Iterator<Item = &'a Champ>, caster: usize) -> bool {
    champs.any(|c| c.id == caster && (c.name.ends_with("_emperor") || c.name.ends_with("_aegis_zero")))
}

fn timed(name: &str, ticks: usize) -> BuffV1 {
    BuffV1::timed(name, ticks.max(1))
}

/// Take crowd-control immunity away without touching the rest of the buff: every buff with
/// `cc_immune` is removed and put straight back with the same stats and remaining time.
fn strip_cc_immunity(sim: &mut StableSim<'_>, c: &Champ) {
    let mut names: Vec<String> = Vec::new();
    for b in &c.buffs {
        if b.cc_immune && !names.iter().any(|n| n == b.name()) {
            names.push(b.name().to_string());
        }
    }
    for name in names {
        let copies: Vec<BuffV1> = c.buffs.iter().filter(|b| b.name() == name).copied().collect();
        sim.entity_remove_buff(c.id, &name);
        for mut b in copies {
            b.cc_immune = false;
            if b.duration_kind == BuffDurationV1::Time.code() && b.duration_tick == 0 {
                continue;
            }
            sim.add_buff(c.id, &b);
        }
    }
}

/// Pin a trapped champion's total move speed bonus at exactly −100% (standing still), whatever speed
/// buffs they have (e.g. Kurama Mode +30% would otherwise leave −100 + 30 = −70%, a slow walk).
fn anchor(sim: &mut StableSim<'_>, c: &Champ, gid: usize) {
    let name = format!("void_anchor{}", gid);
    let others: i64 = c.buffs.iter().filter(|b| b.name() != name).map(|b| b.move_speed_mult as i64).sum();
    let want = (-100 - others).min(0).max(-1000) as i32;
    let have = c.buff(&name).map(|b| b.move_speed_mult);
    if have == Some(want) {
        return;
    }
    if have.is_some() {
        sim.entity_remove_buff(c.id, &name);
    }
    if want < 0 {
        let mut b = timed(&name, 30);
        b.move_speed_mult = want;
        sim.add_buff(c.id, &b);
    }
}

/// "void_p<gojo id>:<x>:<y>" on a trapped champion: where they are frozen.
fn frozen_at(c: &Champ, gid: usize) -> Option<(i64, i64)> {
    let prefix = format!("void_p{}:", gid);
    for b in &c.buffs {
        if let Some(rest) = b.name().strip_prefix(prefix.as_str()) {
            let mut it = rest.split(':');
            let x = it.next()?.parse().ok()?;
            let y = it.next()?.parse().ok()?;
            return Some((x, y));
        }
    }
    None
}

/// The same point, moved in to WALL_INSET inside the edge if it is further out.
fn inside_edge(x: i64, y: i64, cx: i64, cy: i64) -> (i64, i64) {
    let dist2 = d2(x, y, cx, cy);
    if dist2 <= sq(DOMAIN_R - WALL_INSET) {
        return (x, y);
    }
    let len = (dist2 as f64).sqrt().max(1.0);
    let k = (DOMAIN_R - WALL_INSET) as f64 / len;
    ((cx as f64 + (x - cx) as f64 * k) as i64, (cy as f64 + (y - cy) as f64 * k) as i64)
}

/// "void_c<gojo id>:<x>:<y>": the domain centre, stored on Gojo for the domain's lifetime.
fn centre_of(g: &Champ) -> Option<(i64, i64)> {
    let prefix = format!("void_c{}:", g.id);
    for b in &g.buffs {
        if let Some(rest) = b.name().strip_prefix(prefix.as_str()) {
            let mut it = rest.split(':');
            let x = it.next()?.parse().ok()?;
            let y = it.next()?.parse().ok()?;
            return Some((x, y));
        }
    }
    None
}

#[derive(Clone)]
struct VoidField;

impl VoidField {
    fn run_domain(&self, sim: &mut StableSim<'_>, g: &Champ, all: &[Champ]) {
        let tick = sim.tick();
        let rem = g.buff(ACTIVE).map(|b| b.duration_tick).unwrap_or(0);
        if rem == 0 {
            return;
        }
        let guest_name = format!("void_guest{}", g.id);
        let trapped_name = format!("void_trapped{}", g.id);
        let guard_name = format!("void_guard{}", g.id);

        // Warning phase (gojo_void_warn, the first second after the press): only the centre is fixed, nobody is
        // caught yet. The data side overlaps warn and conj by a couple of ticks, so there's never a gap between them.
        let conj = conjuring(g);
        let (cx, cy) = match centre_of(g) {
            Some(c) => c,
            None => {
                sim.add_buff(g.id, &timed(&format!("void_c{}:{}:{}", g.id, g.x, g.y), rem));
                (g.x, g.y)
            }
        };
        let (ux, uy) = (cx.max(0) as u64, cy.max(0) as u64);
        if g.has("gojo_void_warn") && conj.is_none() {
            // warning: the exact danger zone, in red
            sim.debug_draw_circle(ux, uy, DOMAIN_R as u64, EDGE_WARN);
            sim.debug_draw_circle(ux, uy, (DOMAIN_R - 1_200) as u64, EDGE_WARN_SOFT);
            return;
        }

        // The seal's edge grows with the ring while the domain is being conjured: Gojo carries
        // "gojo_void_conj<cast ticks>" for that time (the data side grows its seal the same way).
        let r = match conj {
            Some((cast, left)) if cast > 0 => {
                let done = cast.saturating_sub(left).min(cast) as i64;
                GROW_START + (DOMAIN_R - GROW_START) * done / cast as i64
            }
            _ => DOMAIN_R,
        };
        let conjured = r >= DOMAIN_R;
        // The real edge, drawn in world coordinates (exact, unlike the pixel-art picture): while spreading, the
        // seal's current edge plus a faint outline of the full domain; once conjured, the wall.
        if conjured {
            sim.debug_draw_circle(ux, uy, DOMAIN_R as u64, EDGE_WALL);
            sim.debug_draw_circle(ux, uy, (DOMAIN_R - 1_200) as u64, EDGE_WALL_SOFT);
        } else {
            sim.debug_draw_circle(ux, uy, r.max(0) as u64, EDGE_WALL);
            sim.debug_draw_circle(ux, uy, DOMAIN_R as u64, EDGE_FULL_FAINT);
        }
        // The tick the domain finishes conjuring the edge reaches the full radius: everyone inside is caught then,
        // and only AFTER that does "not trapped but inside" mean an intruder.
        let sealed_name = format!("void_sealed{}", g.id);
        let first_conjured = conjured && !g.has(&sealed_name);
        if first_conjured {
            sim.add_buff(g.id, &timed(&sealed_name, rem));
        }

        // Once the warning is over: pick the guest (closest teammate in the full domain), once.
        let mut newly: Vec<(usize, bool)> = Vec::new();   // (id, is_guest) marked this tick
        let picked_name = format!("void_gpick{}", g.id);
        if !g.has(&picked_name) {
            sim.add_buff(g.id, &timed(&picked_name, rem));
            let guest = all
                .iter()
                .filter(|c| c.id != g.id && c.team == g.team && d2(c.x, c.y, cx, cy) <= sq(DOMAIN_R))
                .min_by_key(|c| (d2(c.x, c.y, g.x, g.y), c.id));
            if let Some(c) = guest {
                sim.add_buff(c.id, &timed(&guest_name, rem));
                let mut guard = timed(&guard_name, rem);
                guard.base_attack_damaged_reduce = 100;
                guard.skill_damaged_reduce = 100;
                guard.cc_immune = true;
                sim.add_buff(c.id, &guard);
                newly.push((c.id, true));
            }
        }

        for c in all {
            if c.id == g.id {
                continue;
            }
            let fresh_guest = newly.iter().any(|&(id, gst)| id == c.id && gst);
            if fresh_guest || c.has(&guest_name) {
                // share Gojo's domain buff ("void": cooldown speed, damage, speed) once it's up
                let ally_name = format!("void_ally{}", g.id);
                if let Some(v) = g.buff("void") {
                    if !c.has(&ally_name) {
                        let mut copy = *v;
                        copy.set_name(&ally_name);
                        sim.add_buff(c.id, &copy);
                    }
                }
                continue;
            }
            let dist2 = d2(c.x, c.y, cx, cy);
            let mut trapped = c.has(&trapped_name);
            if !trapped && ((!conjured && dist2 <= sq(r)) || (first_conjured && dist2 <= sq(DOMAIN_R))) {
                // the growing edge just reached them: sealed until the domain closes
                sim.add_buff(c.id, &timed(&trapped_name, rem));
                trapped = true;
            }
            if trapped {
                // Frozen where the ring caught them (moved inside the edge if they were caught right on it).
                // Anything that moves them (walking, knockbacks, pulls, dashes) is undone next tick. Nothing here
                // ever clears crowd control: 0.2.5 cleared it at the wall, which wiped the seal near the edge.
                let spot = frozen_at(c, g.id);
                let (fx, fy) = match spot {
                    Some(p) => p,
                    None => {
                        let p = inside_edge(c.x, c.y, cx, cy);
                        sim.add_buff(c.id, &timed(&format!("void_p{}:{}:{}", g.id, p.0, p.1), rem));
                        p
                    }
                };
                let moved = d2(c.x, c.y, fx, fy);
                if moved > sq(FREEZE_SLACK) && moved <= sq(FREEZE_CATCH) {
                    sim.entity_set_pos(c.id, fx.max(0) as u64, fy.max(0) as u64);
                }
                if moved <= sq(FREEZE_CATCH) {
                    anchor(sim, c, g.id);
                    if spot.is_none() || !c.stunned || tick % 10 == 0 {
                        // the full seal: no moving, no attacks, no skills
                        strip_cc_immunity(sim, c);
                        for kind in [CcKindV1::Stun, CcKindV1::Bind, CcKindV1::BlockAttack, CcKindV1::BlockSkill, CcKindV1::BlockMoveSkill] {
                            sim.apply_cc(c.id, &CcV1::of_kind(kind, 12));
                        }
                    }
                }
            } else if conjured && dist2 < sq(DOMAIN_R + WALL_OUT) {
                // Outside the domain the edge is a wall: whoever reaches it stops right there, just outside.
                let (mut dx, dy) = (c.x - cx, c.y - cy);
                if dx == 0 && dy == 0 {
                    dx = 1;
                }
                let len = (dx as f64).hypot(dy as f64).max(1.0);
                let k = (DOMAIN_R + WALL_OUT) as f64 / len;
                let nx = (cx as f64 + dx as f64 * k).max(0.0);
                let ny = (cy as f64 + dy as f64 * k).max(0.0);
                if dist2 < sq(DOMAIN_R + WALL_OUT - FREEZE_SLACK) {
                    strip_cc_immunity(sim, c);
                    sim.entity_set_pos(c.id, nx as u64, ny as u64);
                }
            }
        }
    }
}

/// ("gojo_void_conj<cast>", remaining ticks) while the domain is still being conjured.
fn conjuring(g: &Champ) -> Option<(usize, usize)> {
    for b in &g.buffs {
        if let Some(n) = b.name().strip_prefix("gojo_void_conj") {
            if let Ok(cast) = n.parse::<usize>() {
                return Some((cast, b.duration_tick));
            }
        }
    }
    None
}

impl StableMatchHook for VoidField {
    fn on_match_tick(&self, sim: &mut StableSim<'_>, _rng_seed: u64) {
        let all = champions(sim);
        mod_power(sim, &all);
        for g in all.iter().filter(|c| c.has(ACTIVE)) {
            self.run_domain(sim, g, &all);
        }
        // Minato's Flying Raijin: any champion carrying its buffs
        let tick = sim.tick();
        for m in all.iter().filter(|c| c.buffs.iter().any(|b| b.name().starts_with("raijin_") || b.name().starts_with("rk") || b.name().starts_with("tp_"))) {
            raijin::run(sim, m, &all, tick);
        }
        // DIO (knives, The World, time stop) and Frieren (Fern's Zoltraak)
        for d in all.iter().filter(|c| batch2::is_dio(c)) {
            batch2::run_dio(sim, d, &all, tick);
        }
        for f in all.iter().filter(|c| batch2::is_frieren(c)) {
            batch2::run_frieren(sim, f, &all, tick);
        }
        // Steve's boat walls: nobody gets through while they stand
        steve::enforce_walls(sim, &all, tick);
        steve::wall_fog(sim, &all, tick);
        // Omen's smokes: nobody sees in, through or out
        valorant::fog(sim, &all, tick);
        // the Map tab's flank spots: who has been through them
        if tick % 10 == 0 { tactics::tick(sim, &all); }
        // Flash for everyone (round 58): 120 s summoner blink, escapes and kills
        flash::run(sim, &all, tick);
        // Farming (round 60): a mod champion with no enemy champion it can see near gets "mod_farm", which switches its
        // damage skills to plain versions that hit monsters and minions (presets.js withFarm)
        if tick % 10 == 0 { farm_mode(sim, &all); }
    }
}


// ------------------------------------------------------------------ Minato: Flying Raijin

/// Flying Raijin, driven from Minato's data file through buffs:
///   * his 1st cast adds `raijin_throw` (and `raijin_target` on the enemy he aimed at);
///   * his 2nd / 3rd casts add `tp_now` (the 3rd also `tp_last`).
/// This module throws the kunai and does the teleports. State lives in buff names on Minato:
///   `rkm:<x>:<y>:<travelled>:<target>:<dx>:<dy>`  the middle kunai while it flies (homing, with a lifespan)
///   `rkl<i>:<x>:<y>`                              a kunai on the ground he can teleport to (i = 0 middle, 1/2 sides)
///   `rka<i>`                                      side kunai still in the air (can't teleport to it yet)
/// The map's wall grid (30 x 30 cells of 32000 units), read from the map document at match creation by `WallReader`.
/// Used so Minato's normal kunai stop at walls (and nobody is teleported into one).
pub(crate) mod walls {
    use std::sync::RwLock;
    pub const CELL: i64 = 32_000;
    pub const N: i64 = 30;
    /// Tests that set the (global) grid hold this so they don't race.
    #[cfg(test)]
    pub static TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    static GRID: RwLock<Option<Vec<bool>>> = RwLock::new(None);   // index cy * N + cx
    static BUSH: RwLock<Option<Vec<bool>>> = RwLock::new(None);   // the bush grid, same layout

    /// Bumped whenever the grid changes, so each thread's cached copy (wall_at's fast path) knows to refresh.
    static VERSION: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    thread_local! {
        static CACHE: std::cell::RefCell<(u64, Vec<bool>)> = const { std::cell::RefCell::new((0, Vec::new())) };
    }

    pub fn set(cells: Vec<bool>) {
        if let Ok(mut g) = GRID.write() {
            *g = Some(cells);
        }
        VERSION.fetch_add(1, std::sync::atomic::Ordering::Release);
    }
    pub fn set_bushes(cells: Vec<bool>) {
        if let Ok(mut g) = BUSH.write() {
            *g = Some(cells);
        }
    }
    pub fn bush_at(x: i64, y: i64) -> bool {
        if x < 0 || y < 0 || x / CELL >= N || y / CELL >= N { return false; }
        BUSH.read().ok().and_then(|g| g.as_ref().map(|v| v.get(((y / CELL) * N + x / CELL) as usize).copied().unwrap_or(false))).unwrap_or(false)
    }
    /// Centres of the bush cells within `r` of (x, y), nearest first.
    pub fn bushes_near(x: i64, y: i64, r: i64) -> Vec<(i64, i64)> {
        let mut out = Vec::new();
        for cy in 0..N {
            for cx in 0..N {
                let (bx, by) = (cx * CELL + CELL / 2, cy * CELL + CELL / 2);
                let d = (bx - x) as i128 * (bx - x) as i128 + (by - y) as i128 * (by - y) as i128;
                if d <= (r as i128) * (r as i128) && bush_at(bx, by) && !wall_at(bx, by) { out.push((d, (bx, by))); }
            }
        }
        out.sort();
        out.into_iter().map(|p| p.1).collect()
    }
    pub fn wall_at(x: i64, y: i64) -> bool {
        if x < 0 || y < 0 {
            return true;
        }
        let (cx, cy) = (x / CELL, y / CELL);
        if cx >= N || cy >= N {
            return true;
        }
        // round 84: a per-thread copy of the grid, refreshed only when it changes (the planners look up walls a lot)
        let v = VERSION.load(std::sync::atomic::Ordering::Acquire);
        CACHE.with(|c| {
            let mut c = c.borrow_mut();
            if c.0 != v {
                c.1 = GRID.read().ok().and_then(|g| g.clone()).unwrap_or_default();
                c.0 = v;
            }
            c.1.get((cy * N + cx) as usize).copied().unwrap_or(false)
        })
    }
    /// Walking from (x0, y0) toward (x1, y1): the last free point before the first wall.
    pub fn clip(x0: i64, y0: i64, x1: i64, y1: i64) -> (i64, i64) {
        let len = (((x1 - x0) as f64).hypot((y1 - y0) as f64)) as i64;
        let steps = (len / 3_000).max(1);
        let (mut lx, mut ly) = (x0, y0);
        for i in 1..=steps {
            let (x, y) = (x0 + (x1 - x0) * i / steps, y0 + (y1 - y0) * i / steps);
            if wall_at(x, y) {
                return (lx, ly);
            }
            (lx, ly) = (x, y);
        }
        (x1, y1)
    }
    /// A landing spot that may be inside a wall: walk back toward (x0, y0) to the nearest free point.
    pub fn pull_back(x0: i64, y0: i64, x1: i64, y1: i64) -> (i64, i64) {
        let len = (((x1 - x0) as f64).hypot((y1 - y0) as f64)) as i64;
        let steps = (len / 3_000).max(1);
        for i in 0..=steps {
            let (x, y) = (x1 + (x0 - x1) * i / steps, y1 + (y0 - y1) * i / steps);
            if !wall_at(x, y) {
                return (x, y);
            }
        }
        (x0, y0)
    }
}

/// Reads the wall grid out of the map document. The grid's row order isn't documented, so it picks the reading
/// (rows = y or rows = x) that puts the fewest towers inside walls.
struct WallReader;
static MAP_DUMPED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

fn json_rows(js: &str) -> Vec<Vec<i64>> {
    let mut rows: Vec<Vec<i64>> = Vec::new();
    let (mut depth, mut num) = (0i32, String::new());
    let mut flat: Vec<i64> = Vec::new();
    for ch in js.chars() {
        match ch {
            '[' => {
                depth += 1;
                if depth == 2 { rows.push(Vec::new()); }
            }
            ']' | ',' => {
                if !num.is_empty() {
                    let v = num.parse::<f64>().map(|f| f as i64).unwrap_or(0);
                    if depth >= 2 { if let Some(r) = rows.last_mut() { r.push(v); } } else { flat.push(v); }
                    num.clear();
                }
                if ch == ']' { depth -= 1; }
            }
            c if c.is_ascii_digit() || c == '-' || c == '.' => num.push(c),
            c if c == 't' || c == 'f' => {
                // booleans: true / false
                if depth >= 2 { if let Some(r) = rows.last_mut() { r.push(if c == 't' { 1 } else { 0 }); } } else { flat.push(if c == 't' { 1 } else { 0 }); }
            }
            _ => {}
        }
    }
    if rows.is_empty() && !flat.is_empty() {
        rows = flat.chunks(walls::N as usize).map(|c| c.to_vec()).collect();
    }
    rows
}

impl StableMapCustomizer for WallReader {
    fn customize(&self, mode: Option<GameModeKindV1>, doc: &mut StableJsonDoc<'_>) {
        // the map editor (Skill Lab): the 5v5 map document, written once per game run next to this mod's DLL
        if matches!(mode, None | Some(GameModeKindV1::Moba)) && !MAP_DUMPED.swap(true, std::sync::atomic::Ordering::Relaxed) {
            if let (Some(all), Ok(exe)) = (doc.get_json(""), std::env::current_exe()) {
                if let Some(dir) = exe.parent() {
                    let _ = std::fs::write(dir.join("mods").join(MOD_ID).join("map_dump.json"), all);
                }
            }
        }
        // Rian's map plans (the editor's Map tab), fresh for every match
        tactics::load();
        let Some(js) = doc.get_json("walls") else { return };
        // json_rows also picks up the letters of "true"/"false" one by one; keep only full-size rows
        let rows: Vec<Vec<i64>> = json_rows(&js).into_iter().filter(|r| !r.is_empty()).collect();
        let n = walls::N as usize;
        if rows.len() < n || rows.iter().take(n).any(|r| r.len() < n) {
            return;
        }
        // tower positions, to decide the orientation
        let towers: Vec<(i64, i64)> = doc.get_json("towers").map(|t| {
            let mut out = Vec::new();
            let mut rest = t.as_str();
            while let Some(i) = rest.find("\"pos\"") {
                rest = &rest[i + 5..];
                let nums: Vec<i64> = rest.split(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-')).filter(|s| !s.is_empty())
                    .take(2).filter_map(|s| s.parse::<f64>().ok().map(|f| f as i64)).collect();
                if nums.len() == 2 { out.push((nums[0], nums[1])); }
            }
            out
        }).unwrap_or_default();
        let cell = |x: i64, y: i64| ((x / walls::CELL).clamp(0, walls::N - 1) as usize, (y / walls::CELL).clamp(0, walls::N - 1) as usize);
        let bad_rows_y = towers.iter().filter(|&&(x, y)| { let (cx, cy) = cell(x, y); rows[cy][cx] != 0 }).count();
        let bad_rows_x = towers.iter().filter(|&&(x, y)| { let (cx, cy) = cell(x, y); rows[cx][cy] != 0 }).count();
        let mut grid = vec![false; n * n];
        for cy in 0..n {
            for cx in 0..n {
                grid[cy * n + cx] = if bad_rows_y <= bad_rows_x { rows[cy][cx] != 0 } else { rows[cx][cy] != 0 };
            }
        }
        walls::set(grid);
        // the bushes, read the same way (Omen hides his ult in them)
        if let Some(bj) = doc.get_json("bushes") {
            let brows: Vec<Vec<i64>> = json_rows(&bj).into_iter().filter(|r| !r.is_empty()).collect();
            if brows.len() >= n && brows.iter().take(n).all(|r| r.len() >= n) {
                let mut b = vec![false; n * n];
                for cy in 0..n {
                    for cx in 0..n {
                        b[cy * n + cx] = if bad_rows_y <= bad_rows_x { brows[cy][cx] != 0 } else { brows[cx][cy] != 0 };
                    }
                }
                walls::set_bushes(b);
            }
        }
    }
}

mod raijin {
    use super::*;

    const HOP: u64 = 5;                    // a new visible flight segment every 5 ticks
    const MID_SPEED: i64 = 5_000;          // middle kunai: straight now (was homing at 7000)
    const MID_RANGE: i64 = 170_000;        // its range (was 150000)
    const SIDE_SPEED: i64 = 6_500;         // was 9000
    const SIDE_RANGE: i64 = 145_000;       // was 120000
    const SIDE_ANGLE: f64 = 0.55;          // ~32 degrees either side (was 0.38)
    const HIT_RADIUS: i64 = 11_000;
    const SLOTS: usize = 9;                // set 0 = his throw (slots 0-2), set 1 = Kurama Mode's 5 (3-7), 8 = backpack kunai
    // Backpack: he picks up kunai lying on the ground (ones he hasn't teleported to) by walking over them, up to 2.
    // When nothing of his is on the ground he throws one from the pack: a single straight kunai with 70% of the range,
    // half the damage and half the lifespan, that he can teleport to (it opens a Flying Raijin teleport stage).
    const PACK_MAX: i64 = 2;
    const PACK_PICK_R: i64 = 12_000;
    const PACK_SLOT: usize = 8;
    const PACK_RANGE_PCT: i64 = 70;
    const PACK_LIFE: usize = 360;
    const PACK_CD: usize = 120;
    const PACK_HIT: (usize, usize) = (30, 45);   // round 75: was 20 + 30%
    // Kunai basic attack (data): every hit marks the enemy ("minato_mark", 4 s). A teleport cast goes straight behind a
    // marked enemy (using the mark up) before it looks at kunai. A charged Rasengan rides the kunai: when it hits a
    // champion ("minato_ras_go" on them) he flashes behind them and slams it.
    const MARK_TP_R: i64 = 160_000;
    const BEHIND: f64 = 9_000.0;
    // hit-and-run: RET_DELAY ticks after any attacking teleport he flashes back to the spot he left (a seal stays there),
    // if it's still safe
    const RET_DELAY: usize = 24;
    const CHASE_MIN: i64 = 35_000;         // an enemy further than this (past his melee reach) is one he chases
    const CHASE_LOW_PCT: usize = 40;       // … if it runs from him, or it's this low
    // teleports: an engage teleport needs a kunai near an enemy (else the cast throws a fresh fan instead); an escape
    // needs a kunai clearly further from the enemies than he is now (else he doesn't teleport at all)
    const ENGAGE_REACH: i64 = 45_000;
    const ESCAPE_GAIN: i64 = 25_000;
    const ESCAPE_MIN: i64 = 55_000;
    // Kurama Mode's free throw: ONE homing kunai with a long range (through walls). Where it hits an enemy, or at
    // the end of its range, it splits into 5 kunai that fly on through walls: 3 ahead, 2 back the way it came.
    const K_HOMING_SPEED: i64 = 9_000;
    const K_HOMING_RANGE: i64 = 320_000;
    const K_TURN: f64 = 0.17;              // ~10 degrees per tick
    const K_HIT: (usize, usize) = (75, 100);   // round 75: was 60 + 80%
    const K_FRONT_SPREAD: f64 = 0.45;
    const K_FRONT_DIST: i64 = 120_000;
    const K_BACK_SPREAD: f64 = 0.4;
    const K_BACK_DIST: i64 = 90_000;
    const K_SPEED: i64 = 10_000;
    // Flying Raijin dodge: a dangerous enemy shot about to hit him → he teleports out (to a kunai if one lies on the
    // ground, using it up; otherwise a short sidestep on its own cooldown), leaving a 0.5 s afterimage that the shot
    // hits instead (he can't be hit for those 0.5 s). Every dodge adds a stack (max 5, 7 s, refreshed).
    const DODGE_R: i64 = 15_000;           // Kurama Mode: +50% (KCM_DODGE_PCT)
    const DODGE_CD: usize = 90;            // at most one dodge every 1.5 s (Kurama Mode: 1 s)
    const KCM_DODGE_PCT: i64 = 150;
    const ZONE_MARGIN: i64 = 4_000;        // keep this far outside a danger zone
    const ZONE_REACT: usize = 14;          // dodge out of a zone this many ticks before it strikes
    const BLINK_CD: usize = 180;           // the no-kunai sidestep: every 3 s
    const BLINK_DIST: i64 = 20_000;
    const KUNAI_DODGE_MIN: i64 = 12_000;   // a kunai right under him doesn't get him out of the way
    const KUNAI_DODGE_MAX: i64 = 160_000;
    const AFTER_TICKS: usize = 30;
    const FLOW_TICKS: usize = 420;
    const FLOW_MAX: i64 = 5;
    const KUNAI_LIFE: usize = 720;         // 12 s on the ground
    const FEAR_RADIUS: i64 = 18_000;
    const FEAR_HOMING: u64 = 30;           // teleporting to a kunai that hit an enemy: 0.5 s fear, every time
    const FEAR_FIRST: u64 = 6;             // a side kunai: 0.1 s fear, on the first teleport only
    const LOW_HP_PCT: usize = 35;          // at or below: teleport to the safest kunai
    const SLASH_R: i64 = 20_000;           // teleport slash around where he lands
    // with 3+ dodge stacks the slash is empowered and spends 2 stacks; otherwise it's a weak slash
    const SLASH_STRONG: (usize, usize) = (90, 140);   // round 75: was 75 + 115%
    const SLASH_WEAK: (usize, usize) = (45, 70);   // round 75: was 35 + 55%
    const SLASH_STACKS: i64 = 3;
    const SLASH_COST: i64 = 2;
    // a charged Rasengan goes off on arrival (no basic attack needed) if he still has a stack; it spends 1
    const RAS_ARRIVE_R: i64 = 25_000;
    // 50 + 80% AD, +20% AD for every dodge stack he holds when it goes off (5 stacks: 50 + 180%, the old full hit)
    const RAS_DMG: (usize, usize) = (65, 100);   // round 75: was 50 + 80%
    const RAS_PER_STACK: usize = 20;
    const RAS_STUN: u64 = 45;
    // melee dodge: an enemy right next to him starting an attack → he flashes out, spending a dodge stack
    const MELEE_DODGE_R: i64 = 32_000;
    const MELEE_DODGE_CD: usize = 180;
    const MELEE_STEP: i64 = 30_000;
    const STAGE2_WINDOW: usize = 480;
    const TOWER_REACH: i64 = 75_000 + 10_000; // tower attack range (game_setting) + a champion's radius
    const TOWER_HIT_MARGIN: usize = 120;       // a tower dive is allowed only if HP > 1.2 x one tower shot

    /// Enemy towers still standing: (x, y, estimated damage of one shot on Minato).
    fn enemy_towers(sim: &StableSim<'_>, m: &Champ) -> Vec<(i64, i64, usize)> {
        let armor = sim.get_entity(m.id).map_or(0, |e| e.stat().defence);
        let mut out = Vec::new();
        for i in 0..sim.tower_count() {
            let id = sim.tower_id_at(i);
            let Some(t) = sim.get_entity(id) else { continue };
            if !t.is_alive() || t.team() == m.team {
                continue;
            }
            let (x, y) = t.pos();
            // physical shot, estimated with the usual armor curve: dmg x 100 / (100 + armor)
            let hit = t.stat().attack * 100 / (100 + armor);
            out.push((x as i64, y as i64, hit));
        }
        out
    }      // matches the data's stage2Window: time to use the first teleport            // an attacking teleport waits up to 6 s for the Rasengan, then lapses

    /// Remaining ticks on the champion's skill2 (Rasengan) cooldown.

    fn parse(rest: &str) -> Vec<i64> {
        rest.split(':').filter_map(|v| v.parse::<i64>().ok()).collect()
    }
    fn find<'a>(m: &'a Champ, prefix: &str) -> Option<(&'a BuffV1, Vec<i64>)> {
        m.buffs.iter().find_map(|b| b.name().strip_prefix(prefix).map(|rest| (b, parse(rest))))
    }
    fn set_slots(set: usize) -> std::ops::Range<usize> {
        if set == 0 { 0..3 } else { 3..SLOTS }
    }
    fn names(m: &Champ) -> (String, String, String, String) {
        // visual bindings are named after the champion id; fall back to the Skill Lab Minato's id
        let id = if m.name.starts_with("tfm2_") && m.name.ends_with("minato") { m.name.clone() } else { "tfm2_custom_minato".to_string() };
        (format!("{id}_kunai"), format!("{id}_kunai_seal"), format!("{id}_kunai_planted"), format!("{id}_flash"))
    }
    fn fx(sim: &mut StableSim<'_>, name: &str, caster: usize, x: i64, y: i64, time: u64) {
        crate::perf::note_fx(caster, sim.tick());
        sim.play_view_effect(name, caster, &InputTargetV1::pos(x.max(0) as u64, y.max(0) as u64), 0, 0, time);
    }
    /// A purely visual kunai flight from (x,y) to (tx,ty): it hits nothing, the logic is all here.
    fn fly(sim: &mut StableSim<'_>, name: &str, m: &Champ, x: i64, y: i64, tx: i64, ty: i64, speed: i64) {
        let spec = ProjectileSpawnV1 {
            caster_id: m.id,
            team: m.team,
            x: x.max(0) as u64,
            y: y.max(0) as u64,
            radius: 1_000,
            speed: speed.max(1) as u64,
            move_kind: ProjectileMoveKindV1::Linear.code(),
            target_x: tx.max(0) as u64,
            target_y: ty.max(0) as u64,
            penetrate: true,
            casting_target: CastingTargetV1::None.code(),
            ..ProjectileSpawnV1::default()
        };
        sim.spawn_projectile(name, &format!("{MOD_ID}:noop"), &spec);
    }
    fn norm(dx: f64, dy: f64) -> (f64, f64) {
        let l = dx.hypot(dy);
        if l < 1e-9 { (1.0, 0.0) } else { (dx / l, dy / l) }
    }
    fn rotate(d: (f64, f64), a: f64) -> (f64, f64) {
        (d.0 * a.cos() - d.1 * a.sin(), d.0 * a.sin() + d.1 * a.cos())
    }
    /// Remove one set's kunai (its middle kunai in flight and its ground slots).
    fn clear_set(sim: &mut StableSim<'_>, m: &Champ, set: usize) {
        let mine = |n: &str| -> bool {
            if n.starts_with(&format!("rkm{set}:")) {
                return true;
            }
            set_slots(set).any(|slot| n.starts_with(&format!("rkl{slot}:")) || n == format!("rka{slot}"))
        };
        let old: Vec<String> = m.buffs.iter().map(|b| b.name().to_string()).filter(|n| mine(n)).collect();
        for n in old {
            sim.entity_remove_buff(m.id, &n);
        }
    }

    /// Throw his fan of three kunai at `t`, all straight: the middle one hits the first enemy champion in its path
    /// (and drops there), the sides fly their full range.
    fn throw_fan(sim: &mut StableSim<'_>, m: &Champ, set: usize, t: &Champ, kunai_name: &str, speed_mult: f64, range_mult: f64) {
        let dir = norm((t.x - m.x) as f64, (t.y - m.y) as f64);
        // middle: straight, it flies (and hits) in run()
        let (dx, dy) = ((dir.0 * 1000.0) as i64, (dir.1 * 1000.0) as i64);
        sim.add_buff(m.id, &timed(&format!("rkm{set}:{}:{}:0:{}:{}", m.x, m.y, dx, dy), 300));
        // sides: straight, full range
        let side_speed = (SIDE_SPEED as f64 * speed_mult) as i64;
        let side_range = SIDE_RANGE as f64 * range_mult;
        for (k, a) in [(1usize, -SIDE_ANGLE), (2usize, SIDE_ANGLE)] {
            let slot = set * 3 + k;
            let d = rotate(dir, a);
            let (ex, ey) = (m.x + (d.0 * side_range) as i64, m.y + (d.1 * side_range) as i64);
            let (ex, ey) = walls::clip(m.x, m.y, ex.max(0), ey.max(0));   // normal kunai stop at walls
            fly(sim, kunai_name, m, m.x, m.y, ex, ey, side_speed);
            sim.add_buff(m.id, &timed(&format!("rkl{slot}:{ex}:{ey}:0"), KUNAI_LIFE));
            sim.add_buff(m.id, &timed(&format!("rka{slot}"), (side_range / side_speed as f64).ceil() as usize + 1));
        }
    }

    /// Kurama Mode's free throw: one homing kunai at `t` ("rkm1:<x>:<y>:<travelled>:<target>:<dx>:<dy>").
    fn throw_kurama(sim: &mut StableSim<'_>, m: &Champ, t: &Champ) {
        let dir = norm((t.x - m.x) as f64, (t.y - m.y) as f64);
        let (dx, dy) = ((dir.0 * 1000.0) as i64, (dir.1 * 1000.0) as i64);
        sim.add_buff(m.id, &timed(&format!("rkm1:{}:{}:0:{}:{dx}:{dy}", m.x, m.y, t.id), 300));
    }
    /// The Kurama kunai splits at (x, y) heading `dir`: 3 kunai on ahead, 2 back, all through walls (a spot inside a
    /// wall is pulled back out). Slots 3-7.
    fn split_kurama(sim: &mut StableSim<'_>, m: &Champ, x: i64, y: i64, dir: (f64, f64), kunai_name: &str) {
        let back = (-dir.0, -dir.1);
        let spots = [
            (rotate(dir, -K_FRONT_SPREAD), K_FRONT_DIST), (dir, K_FRONT_DIST), (rotate(dir, K_FRONT_SPREAD), K_FRONT_DIST),
            (rotate(back, -K_BACK_SPREAD), K_BACK_DIST), (rotate(back, K_BACK_SPREAD), K_BACK_DIST),
        ];
        for (k, (d, r)) in spots.iter().enumerate() {
            let slot = 3 + k;
            let (ex, ey) = ((x + (d.0 * *r as f64) as i64).max(0), (y + (d.1 * *r as f64) as i64).max(0));
            let (ex, ey) = walls::pull_back(x, y, ex, ey);
            let dist = ((ex - x) as f64).hypot((ey - y) as f64);
            fly(sim, kunai_name, m, x, y, ex, ey, K_SPEED);
            sim.add_buff(m.id, &timed(&format!("rkl{slot}:{ex}:{ey}:0"), KUNAI_LIFE));
            sim.add_buff(m.id, &timed(&format!("rka{slot}"), (dist / K_SPEED as f64).ceil() as usize + 1));
        }
    }

    /// Dodge stacks: "hrs:<n>" counts them (its duration is the stacks' time left), "minato_flow<n>" carries the
    /// stats (and the lightning bolts over his head).
    fn flow_count(m: &Champ) -> (i64, usize) {
        m.buffs.iter().find_map(|b| b.name().strip_prefix("hrs:").and_then(|r| r.parse::<i64>().ok()).map(|n| (n, b.duration_tick)))
            .unwrap_or((0, 0))
    }
    fn set_flow(sim: &mut StableSim<'_>, m: &Champ, n: i64, ticks: usize) {
        if let Some((b, _)) = find(m, "hrs:") {
            let name = b.name().to_string();
            sim.entity_remove_buff(m.id, &name);
        }
        for k in 1..=FLOW_MAX {
            sim.entity_remove_buff(m.id, &format!("minato_flow{k}"));
        }
        let n = n.clamp(0, FLOW_MAX);
        if n == 0 || ticks == 0 {
            return;
        }
        sim.add_buff(m.id, &timed(&format!("hrs:{n}"), ticks));
        let mut b = timed(&format!("minato_flow{n}"), ticks);
        b.attack_mult = 8 * n as i32;
        b.attack_speed_mult = 6 * n as i32;
        b.move_speed_mult = 5 * n as i32;
        if n >= FLOW_MAX {
            b.vamp = 10;
        }
        sim.add_buff(m.id, &b);
    }
    /// One more dodge stack (refreshes the 7 s).
    fn add_flow(sim: &mut StableSim<'_>, m: &Champ) {
        let (n, _) = flow_count(m);
        set_flow(sim, m, n + 1, FLOW_TICKS);
    }


    /// The spot just behind `t` as seen from `m` (pulled out of walls).
    fn behind(m: &Champ, t: &Champ) -> (i64, i64) {
        let d = norm((t.x - m.x) as f64, (t.y - m.y) as f64);
        let (bx, by) = ((t.x + (d.0 * BEHIND) as i64).max(0), (t.y + (d.1 * BEHIND) as i64).max(0));
        walls::pull_back(t.x, t.y, bx, by)
    }

    /// Throw one kunai from the backpack at `t` (it flies in run(): "rkb:...").
    fn pack_throw(sim: &mut StableSim<'_>, m: &Champ, t: &Champ) {
        let d = norm((t.x - m.x) as f64, (t.y - m.y) as f64);
        sim.add_buff(m.id, &timed(&format!("rkb:{}:{}:0:{}:{}", m.x, m.y, (d.0 * 1000.0) as i64, (d.1 * 1000.0) as i64), 300));
        sim.add_buff(m.id, &timed("rbp_cd", PACK_CD));
    }
    /// The backpack count ("rbp:<n>") and its icon (minato_pack<n>).
    fn set_pack(sim: &mut StableSim<'_>, m: &Champ, n: i64) {
        for k in 0..=PACK_MAX {
            sim.entity_remove_buff(m.id, &format!("rbp:{k}"));
            sim.entity_remove_buff(m.id, &format!("minato_pack{k}"));
        }
        if n > 0 {
            sim.add_buff(m.id, &BuffV1::named(&format!("rbp:{n}")));
            sim.add_buff(m.id, &BuffV1::named(&format!("minato_pack{n}")));
        }
    }

    /// He has just arrived at (x, y) by Flying Raijin: primes the Rasengan stun, slashes the enemies around him
    /// (empowered with 3+ dodge stacks), fires a charged Rasengan if he has a stack to spare, and fears them
    /// (0.5 s at a kunai that hit an enemy; 0.1 s if `fear_first`).
    fn arrive(sim: &mut StableSim<'_>, m: &Champ, enemies: &[&Champ], flash_name: &str, x: i64, y: i64, struck: bool, fear_first: bool) {
        let last = !fear_first;
        // hit-and-run: m is still where he left from; a seal stays there and he flashes back to it after the strike
        for b in m.buffs.iter().filter(|b| b.name().starts_with("rret:")) {
            let n = b.name().to_string();
            sim.entity_remove_buff(m.id, &n);
        }
        // (not after an escape: he's low and got away, there's nothing to go back to)
        let low = m.max_hp > 0 && m.hp * 100 <= m.max_hp * LOW_HP_PCT;
        if !low {
            sim.add_buff(m.id, &timed(&format!("rret:{}:{}:{}", m.x, m.y, sim.tick() + RET_DELAY), RET_DELAY + 30));
            fx(sim, &flash_name.replace("_flash", "_kunai_planted"), m.id, m.x, m.y, RET_DELAY as u64);
        }
        sim.add_buff(m.id, &timed("hiraishin", 120));   // primes the Rasengan stun
        // teleport slash around where he lands (V1 can parry it): empowered with 3+ dodge stacks (spends 2),
        // weak otherwise
        let mut parried: Vec<usize> = Vec::new();
        let (mut stacks, left) = flow_count(m);
        let strong = stacks >= SLASH_STACKS;
        let (sb, sr) = if strong { SLASH_STRONG } else { SLASH_WEAK };
        let slash = sb + m.attack * sr / 100;
        let slash_fx = flash_name.replace("_flash", if strong { "_slash_big" } else { "_flash" });
        let mut hit_any = false;
        for e in enemies.iter().filter(|e| d2(e.x, e.y, x, y) <= sq(SLASH_R)) {
            if crate::batch2::try_parry(sim, e, m.id, crate::batch2::hit_estimate(sim, m, 150)) {
                parried.push(e.id);
                continue;
            }
            sim.deal_damage(m.id, e.id, slash, 0, AttackTypeV1::Skill);
            crate::wave_near(sim, m.id, e.id, SLASH_R, slash, 0);
            fx(sim, &slash_fx, m.id, e.x, e.y, 12);
            hit_any = true;
        }
        if strong && hit_any {
            stacks -= SLASH_COST;
        }
        // Rasengan on arrival: a charged Rasengan goes off at once on the nearest enemy if he has a stack left
        // (spends 1); without a stack it waits for his basic attack as usual
        if stacks >= 1 && m.has("minato_rasengan") {
            let near = enemies.iter().filter(|e| d2(e.x, e.y, x, y) <= sq(RAS_ARRIVE_R) && !parried.contains(&e.id))
                .min_by_key(|e| (d2(e.x, e.y, x, y), e.id)).copied();
            if let Some(t) = near {
                sim.entity_remove_buff(m.id, "minato_rasengan");
                if !crate::batch2::try_parry(sim, t, m.id, crate::batch2::hit_estimate(sim, m, 200)) {
                    let ratio = RAS_DMG.1 + RAS_PER_STACK * stacks.clamp(0, FLOW_MAX) as usize;
                    sim.deal_damage(m.id, t.id, RAS_DMG.0 + m.attack * ratio / 100, 0, AttackTypeV1::Skill);
                    crate::wave_near(sim, m.id, t.id, 20_000, RAS_DMG.0 + m.attack * ratio / 100, 0);
                    sim.apply_cc(t.id, &CcV1::of_kind(CcKindV1::Stun, RAS_STUN));
                    fx(sim, &flash_name.replace("_flash", "_rasengan_hit"), m.id, t.x, t.y, 22);
                }
                stacks -= 1;
            }
        }
        if stacks != flow_count(m).0 {
            set_flow(sim, m, stacks, left);
        }
        let fear_ticks = if struck { FEAR_HOMING } else if !last { FEAR_FIRST } else { 0 };
        if fear_ticks > 0 {
            for e in enemies.iter().filter(|e| d2(e.x, e.y, x, y) <= sq(FEAR_RADIUS) && !parried.contains(&e.id)) {
                let d = norm((e.x - x) as f64, (e.y - y) as f64);
                let mut fear = CcV1::of_kind(CcKindV1::Fear, fear_ticks);
                fear.dx = (d.0 * 1000.0) as i64;
                fear.dy = (d.1 * 1000.0) as i64;
                sim.apply_cc(e.id, &fear);
            }
        }
    }

    pub fn run(sim: &mut StableSim<'_>, m: &Champ, all: &[Champ], tick: usize) {
        let (kunai_name, seal_name, planted_name, flash_name) = names(m);
        let kcm = m.has("minato_kcm");
        let (range_mult, speed_mult) = if kcm { (2.0, 1.5) } else { (1.0, 1.0) };
        let enemies: Vec<&Champ> = all.iter().filter(|c| c.team != m.team).collect();
        // enemy towers: a spot inside one is a dive. tower_hit = damage he'd take there (0 = safe),
        // tower_ok = safe, or he survives one shot with margin (the last-resort rule).
        let towers = enemy_towers(sim, m);
        let tower_hit = |x: i64, y: i64| -> usize {
            towers.iter().filter(|&&(tx, ty, _)| d2(x, y, tx, ty) <= sq(TOWER_REACH)).map(|&(_, _, h)| h).sum()
        };
        let tower_ok = |x: i64, y: i64| -> bool {
            let h = tower_hit(x, y);
            h == 0 || m.hp * 100 > h * TOWER_HIT_MARGIN
        };
        // danger zones: any open Unlimited Void (it seals whoever's inside), DIO's mark and time-stop field, David's
        // gravity before it smashes down. He never teleports into one, and dodges out of one about to strike.
        let mut zones = crate::batch2::danger_zones(all, m.team);
        for g in all.iter().filter(|c| c.has("gojo_void_active")) {
            if g.id == m.id { continue; }
            let (gx, gy) = centre_of(g).unwrap_or((g.x, g.y));
            zones.push((gx, gy, DOMAIN_R, 0, 0));
        }
        let safe = |x: i64, y: i64| zones.iter().all(|&(zx, zy, r, _, _)| d2(x, y, zx, zy) > sq(r + ZONE_MARGIN));
        let hp_pct = if m.max_hp == 0 { 100 } else { m.hp * 100 / m.max_hp };
        let escaping = hp_pct <= LOW_HP_PCT;
        let crowd = |x: i64, y: i64| enemies.iter().filter(|e| d2(e.x, e.y, x, y) <= sq(25_000)).count();
        let nearest_enemy = |x: i64, y: i64| enemies.iter().map(|e| d2(e.x, e.y, x, y)).min().unwrap_or(i128::MAX);

        // ---- throws. Two independent sets of three kunai: set 0 = his Flying Raijin throw, set 1 = the free throw
        //      when Kurama Mode activates. A new throw only replaces kunai of its own set, so up to 6 can lie
        //      on the ground; kunai he teleports to disappear.
        for (set, marker, seen) in [(0usize, "raijin_throw", "raijin_seen"), (1usize, "raijin_kthrow", "raijin_kseen")] {
            if !m.has(marker) || m.has(seen) {
                continue;
            }
            sim.add_buff(m.id, &timed(seen, 12));
            clear_set(sim, m, set);
            if set == 0 {
                sim.entity_remove_buff(m.id, "tp_wait");        // a new throw cancels a held teleport
                sim.entity_remove_buff(m.id, "tp_wait_last");
            }
            let target = enemies.iter().filter(|e| set == 0 && e.has("raijin_target")).min_by_key(|e| (d2(e.x, e.y, m.x, m.y), e.id))
                .or_else(|| enemies.iter().filter(|e| d2(e.x, e.y, m.x, m.y) <= sq(200_000)).min_by_key(|e| (d2(e.x, e.y, m.x, m.y), e.id)));
            let Some(t) = target else { continue };
            if set == 0 {
                throw_fan(sim, m, set, t, &kunai_name, speed_mult, range_mult);
            } else {
                throw_kurama(sim, m, t);
            }
        }

        // ---- middle kunai in flight (straight): hits the first enemy champion in its path, drops at its range
        if let Some((b, v)) = find(m, "rkm0:") {
            if v.len() >= 5 {
                let old_name = b.name().to_string();
                let (mut x, mut y, mut travelled) = (v[0], v[1], v[2]);
                let dir = norm(v[3] as f64, v[4] as f64);
                let speed = (MID_SPEED as f64 * speed_mult) as i64;
                let range = (MID_RANGE as f64 * range_mult) as i64;
                if tick as u64 % HOP == 0 || travelled == 0 {
                    let seg = (speed as u64 * HOP) as f64;
                    fly(sim, &kunai_name, m, x, y, x + (dir.0 * seg) as i64, y + (dir.1 * seg) as i64, speed);
                }
                let (nx, ny) = ((x + (dir.0 * speed as f64) as i64).max(0), (y + (dir.1 * speed as f64) as i64).max(0));
                let walled = walls::wall_at(nx, ny);   // a normal kunai stops at a wall and drops there
                if !walled {
                    x = nx;
                    y = ny;
                }
                travelled += speed;
                sim.entity_remove_buff(m.id, &old_name);
                let hit = enemies.iter().filter(|e| d2(e.x, e.y, x, y) <= sq(HIT_RADIUS)).min_by_key(|e| (d2(e.x, e.y, x, y), e.id)).copied();
                if hit.is_some() || travelled >= range || walled {
                    let mut struck = 0;
                    if let Some(t) = hit {
                        // 40 + 60% AD physical, through the normal damage pipeline
                        let ad = 50 + m.attack * 75 / 100;   // round 75: was 40 + 60%
                        sim.deal_damage(m.id, t.id, ad, 0, AttackTypeV1::Skill);
                        crate::wave_near(sim, m.id, t.id, 15_000, ad, 0);
                        x = t.x;
                        y = t.y;
                        struck = 1;
                    }
                    fx(sim, &seal_name, m.id, x, y, 20);
                    sim.add_buff(m.id, &timed(&format!("rkl0:{x}:{y}:{struck}"), KUNAI_LIFE));
                } else {
                    sim.add_buff(m.id, &timed(&format!("rkm0:{x}:{y}:{travelled}:{}:{}", v[3], v[4]), 300));
                }
            }
        }

        // ---- the Kurama kunai in flight: homes in on its target through walls; splits on a hit or at its range
        if let Some((b, v)) = find(m, "rkm1:") {
            if v.len() >= 6 {
                let old_name = b.name().to_string();
                let (mut x, mut y, mut travelled, tid) = (v[0], v[1], v[2], v[3] as usize);
                let mut dir = norm(v[4] as f64, v[5] as f64);
                let tgt = enemies.iter().find(|c| c.id == tid).copied();
                if let Some(t) = tgt {
                    let want = norm((t.x - x) as f64, (t.y - y) as f64);
                    let mut diff = want.1.atan2(want.0) - dir.1.atan2(dir.0);
                    while diff > std::f64::consts::PI { diff -= 2.0 * std::f64::consts::PI; }
                    while diff < -std::f64::consts::PI { diff += 2.0 * std::f64::consts::PI; }
                    dir = rotate(dir, diff.clamp(-K_TURN, K_TURN));
                }
                if tick as u64 % HOP == 0 || travelled == 0 {
                    let seg = (K_HOMING_SPEED as u64 * HOP) as f64;
                    fly(sim, &kunai_name, m, x, y, x + (dir.0 * seg) as i64, y + (dir.1 * seg) as i64, K_HOMING_SPEED);
                }
                x = (x + (dir.0 * K_HOMING_SPEED as f64) as i64).max(0);
                y = (y + (dir.1 * K_HOMING_SPEED as f64) as i64).max(0);
                travelled += K_HOMING_SPEED;
                sim.entity_remove_buff(m.id, &old_name);
                let hit = enemies.iter().filter(|e| d2(e.x, e.y, x, y) <= sq(HIT_RADIUS)).min_by_key(|e| (d2(e.x, e.y, x, y), e.id)).copied();
                if hit.is_some() || travelled >= K_HOMING_RANGE {
                    if let Some(t) = hit {
                        sim.deal_damage(m.id, t.id, K_HIT.0 + m.attack * K_HIT.1 / 100, 0, AttackTypeV1::Skill);
                        crate::wave_near(sim, m.id, t.id, 15_000, K_HIT.0 + m.attack * K_HIT.1 / 100, 0);
                        x = t.x;
                        y = t.y;
                    }
                    fx(sim, &seal_name, m.id, x, y, 20);
                    fx(sim, &flash_name, m.id, x, y, 12);
                    split_kurama(sim, m, x, y, dir, &kunai_name);
                } else {
                    let (dx, dy) = ((dir.0 * 1000.0) as i64, (dir.1 * 1000.0) as i64);
                    sim.add_buff(m.id, &timed(&format!("rkm1:{x}:{y}:{travelled}:{tid}:{dx}:{dy}"), 300));
                }
            }
        }

        // ---- the backpack kunai in flight (straight, stops at walls): "rkb:<x>:<y>:<travelled>:<dx>:<dy>"
        if let Some((b, v)) = find(m, "rkb:") {
            if v.len() >= 5 {
                let old_name = b.name().to_string();
                let (mut x, mut y, mut travelled) = (v[0], v[1], v[2]);
                let dir = norm(v[3] as f64, v[4] as f64);
                let speed = MID_SPEED;
                let range = MID_RANGE * PACK_RANGE_PCT / 100;
                if tick as u64 % HOP == 0 || travelled == 0 {
                    let seg = (speed as u64 * HOP) as f64;
                    fly(sim, &kunai_name, m, x, y, x + (dir.0 * seg) as i64, y + (dir.1 * seg) as i64, speed);
                }
                let (nx, ny) = ((x + (dir.0 * speed as f64) as i64).max(0), (y + (dir.1 * speed as f64) as i64).max(0));
                let walled = walls::wall_at(nx, ny);
                if !walled { x = nx; y = ny; }
                travelled += speed;
                sim.entity_remove_buff(m.id, &old_name);
                let hit = enemies.iter().filter(|e| d2(e.x, e.y, x, y) <= sq(HIT_RADIUS)).min_by_key(|e| (d2(e.x, e.y, x, y), e.id)).copied();
                if hit.is_some() || travelled >= range || walled {
                    let mut struck = 0;
                    if let Some(t) = hit {
                        sim.deal_damage(m.id, t.id, PACK_HIT.0 + m.attack * PACK_HIT.1 / 100, 0, AttackTypeV1::Skill);
                        crate::wave_near(sim, m.id, t.id, 12_000, PACK_HIT.0 + m.attack * PACK_HIT.1 / 100, 0);
                        x = t.x; y = t.y; struck = 1;
                    }
                    fx(sim, &seal_name, m.id, x, y, 20);
                    // the chase: a pack kunai that lands on (or next to) an enemy pulls him straight there, no cast
                    // needed — unless that spot is a crowd, a deadly tower or a danger zone, or he's fleeing
                    let chase_ok = !escaping && !m.stunned && nearest_enemy(x, y) <= sq(ENGAGE_REACH) && crowd(x, y) < 3
                        && tower_ok(x, y) && safe(x, y);
                    if chase_ok {
                        fx(sim, &flash_name, m.id, m.x, m.y, 15);
                        sim.entity_set_pos(m.id, x.max(0) as u64, y.max(0) as u64);
                        fx(sim, &flash_name, m.id, x, y, 15);
                        arrive(sim, m, &enemies, &flash_name, x, y, struck == 1, false);
                    } else {
                        sim.add_buff(m.id, &timed(&format!("rkl{PACK_SLOT}:{x}:{y}:{struck}"), PACK_LIFE));
                        // his next Flying Raijin cast can teleport to it
                        if !m.has("raijin_2") && !m.has("raijin_3") {
                            sim.add_buff(m.id, &timed("raijin_3", STAGE2_WINDOW));
                        }
                    }
                } else {
                    sim.add_buff(m.id, &timed(&format!("rkb:{x}:{y}:{travelled}:{}:{}", v[3], v[4]), 300));
                }
            }
        }

        // ---- kunai on the ground: the seal pulses where they lie
        let landed: Vec<(usize, i64, i64)> = (0..SLOTS)
            .filter(|i| !m.has(&format!("rka{i}")))
            .filter_map(|i| find(m, &format!("rkl{i}:")).and_then(|(_, v)| (v.len() >= 2).then(|| (i, v[0], v[1]))))
            .collect();
        if tick % 30 == 0 {
            for &(_, x, y) in &landed {
                fx(sim, &planted_name, m.id, x, y, 30);
            }
        }

        // ---- backpack (max 2). Pick-up: walking over one of his kunai, only when it's no use where it lies (no
        //      enemy near it), the spot is safe and the pack has room. Throw: chasing an enemy (one running from him
        //      or low, out of his reach) with no kunai lying near any enemy → a pack kunai first; or simply when
        //      nothing of his is on the ground. It flies 70% as far, and if it lands on an enemy he flashes there.
        let mut pack = find(m, "rbp:").and_then(|(_, v)| v.first().copied()).unwrap_or(0).clamp(0, PACK_MAX);
        let pack_before = pack;
        let requested_tp = m.has("tp_now") || m.has("tp_wait");
        let mut picked: Vec<usize> = Vec::new();
        if pack < PACK_MAX && !m.stunned && !requested_tp {
            for &(i, x, y) in &landed {
                if pack >= PACK_MAX { break; }
                let useless_here = nearest_enemy(x, y) > sq(ENGAGE_REACH);
                if d2(x, y, m.x, m.y) <= sq(PACK_PICK_R) && useless_here && safe(x, y) && tower_hit(x, y) == 0 {
                    if let Some((b, _)) = find(m, &format!("rkl{i}:")) {
                        let n = b.name().to_string();
                        sim.entity_remove_buff(m.id, &n);
                    }
                    fx(sim, &flash_name, m.id, x, y, 8);
                    picked.push(i);
                    pack += 1;
                }
            }
        }
        let landed: Vec<(usize, i64, i64)> = landed.into_iter().filter(|(i, _, _)| !picked.contains(i)).collect();
        let pack_in_flight = find(m, "rkb:").is_some();
        let set_in_flight = (0..SLOTS).any(|i| m.has(&format!("rka{i}"))) || find(m, "rkm0:").is_some() || find(m, "rkm1:").is_some();
        let kunai_on_enemy = landed.iter().any(|&(_, x, y)| nearest_enemy(x, y) <= sq(ENGAGE_REACH));
        let pack_reach = MID_RANGE * PACK_RANGE_PCT / 100;
        // the chase: the enemy he's after this second (nearest one past his melee reach), and whether it's getting away
        let quarry = enemies.iter().filter(|e| d2(e.x, e.y, m.x, m.y) > sq(CHASE_MIN) && d2(e.x, e.y, m.x, m.y) <= sq(pack_reach))
            .min_by_key(|e| (d2(e.x, e.y, m.x, m.y), e.id)).copied();
        let mut fleeing = false;
        if let Some(q) = quarry {
            let dist = (d2(q.x, q.y, m.x, m.y) as f64).sqrt() as i64;
            if let Some((b, v)) = find(m, "rch:") {
                if v.len() >= 2 && v[0] == q.id as i64 && dist > v[1] + 1_000 { fleeing = true; }
                if tick % 10 == 0 { let n = b.name().to_string(); sim.entity_remove_buff(m.id, &n); }
            }
            if tick % 10 == 0 || find(m, "rch:").is_none() {
                sim.add_buff(m.id, &timed(&format!("rch:{}:{dist}", q.id), 30));
            }
        }
        let can_pack = pack > 0 && !pack_in_flight && !set_in_flight && !m.has("rbp_cd") && !m.stunned && !escaping
            && !m.has("raijin_throw") && !m.has("raijin_seen");
        let chasing = quarry.map_or(false, |q| fleeing || (q.max_hp > 0 && q.hp * 100 / q.max_hp <= CHASE_LOW_PCT));
        let pack_target = if !can_pack {
            None
        } else if chasing && !kunai_on_enemy {
            quarry.filter(|q| crowd(q.x, q.y) < 3 && tower_ok(q.x, q.y) && safe(q.x, q.y))
        } else if landed.is_empty() {
            enemies.iter().filter(|e| d2(e.x, e.y, m.x, m.y) <= sq(pack_reach)).min_by_key(|e| (d2(e.x, e.y, m.x, m.y), e.id)).copied()
        } else {
            None
        };
        if let Some(t) = pack_target {
            pack_throw(sim, m, t);
            pack -= 1;
        }
        if pack != pack_before || (pack > 0 && !m.has(&format!("minato_pack{pack}"))) {
            set_pack(sim, m, pack);
        }

        // ---- map awareness: a boat wall cuts him off from a teammate in trouble, a tower or an objective under attack →
        //      Flying Raijin over it (a kunai lying on that side; else a backpack kunai thrown over at the nearest enemy
        //      there, which pulls him across when it lands). At most every 4 s.
        if !escaping && !m.stunned && !m.has("rwx_cd") {
            if let Some((nx, ny)) = crate::steve::wall_need(sim, all, m, tick, MARK_TP_R) {
                let spot = landed.iter().filter(|&&(_, x, y)| d2(x, y, nx, ny) <= sq(ENGAGE_REACH)
                    && crate::steve::walls_up(all, tick).iter().any(|w| crate::steve::crosses(w, tick, m.x, m.y, x, y))
                    && crowd(x, y) < 3 && tower_ok(x, y) && safe(x, y))
                    .min_by_key(|&&(i, x, y)| (d2(x, y, nx, ny), i)).copied();
                if let Some((i, x, y)) = spot {
                    sim.add_buff(m.id, &timed("rwx_cd", 240));
                    if let Some((b, _)) = find(m, &format!("rkl{i}:")) { let n = b.name().to_string(); sim.entity_remove_buff(m.id, &n); }
                    fx(sim, &flash_name, m.id, m.x, m.y, 15);
                    sim.entity_set_pos(m.id, x.max(0) as u64, y.max(0) as u64);
                    fx(sim, &flash_name, m.id, x, y, 15);
                    arrive(sim, m, &enemies, &flash_name, x, y, false, true);
                    return;
                }
                if pack > 0 && !pack_in_flight && !m.has("rbp_cd") {
                    if let Some(t) = enemies.iter().filter(|e| d2(e.x, e.y, nx, ny) <= sq(45_000) && d2(e.x, e.y, m.x, m.y) <= sq(pack_reach))
                        .min_by_key(|e| (d2(e.x, e.y, nx, ny), e.id)).copied() {
                        sim.add_buff(m.id, &timed("rwx_cd", 240));
                        pack_throw(sim, m, t);
                        set_pack(sim, m, pack - 1);
                    }
                }
            }
        }

        // ---- Hiraishin reflex: caught in an enemy Unlimited Void that is still warning or spreading, with a kunai
        //      lying outside it → he flashes out to that kunai at once (no cast needed), once per domain.
        for g in all.iter().filter(|c| c.team != m.team && c.has("gojo_void_active")) {
            let evade_name = format!("raijin_evade{}", g.id);
            if m.has(&format!("void_trapped{}", g.id)) || m.has(&evade_name) || m.stunned {
                continue;
            }
            if !(g.has("gojo_void_warn") || conjuring(g).is_some()) {
                continue;
            }
            let (gx, gy) = centre_of(g).unwrap_or((g.x, g.y));
            if d2(m.x, m.y, gx, gy) > sq(DOMAIN_R) {
                continue;
            }
            let out = landed.iter().filter(|&&(_, x, y)| d2(x, y, gx, gy) > sq(DOMAIN_R + 5_000) && tower_ok(x, y))
                .max_by_key(|&&(i, x, y)| (tower_hit(x, y) == 0, d2(x, y, gx, gy), std::cmp::Reverse(i))).copied();
            if let Some((i, x, y)) = out {
                fx(sim, &flash_name, m.id, m.x, m.y, 15);
                sim.entity_set_pos(m.id, x.max(0) as u64, y.max(0) as u64);
                fx(sim, &flash_name, m.id, x, y, 15);
                if let Some((b, _)) = find(m, &format!("rkl{i}:")) {
                    let n = b.name().to_string();
                    sim.entity_remove_buff(m.id, &n);
                }
                sim.add_buff(m.id, &timed(&evade_name, 600));
                return;
            }
        }

        // ---- Flying Raijin dodge (see DODGE_R): kunai on the ground are his dodge charges
        let trapped = m.buffs.iter().any(|b| b.name().starts_with("void_trapped"));
        let dodge_r = if kcm { DODGE_R * KCM_DODGE_PCT / 100 } else { DODGE_R };
        let after_ticks = if kcm { AFTER_TICKS * KCM_DODGE_PCT as usize / 100 } else { AFTER_TICKS };
        let dodge_cd = if kcm { DODGE_CD * 100 / KCM_DODGE_PCT as usize } else { DODGE_CD };
        if !m.stunned && !trapped && !m.has("hr_dodge_cd") && !m.has("hr_after") {
            let incoming = (0..sim.projectile_count()).filter_map(|i| sim.projectile_at(i)).find(|p| {
                if p.is_end || p.team == m.team || cosmetic_shot(enemies.iter().copied(), p.caster_id) { return false; }
                let (px, py) = (p.x as i64, p.y as i64);
                if d2(px, py, m.x, m.y) > sq(dodge_r) { return false; }
                // only shots from enemy champions (not minions or towers), on their way to him
                match enemies.iter().find(|c| c.id == p.caster_id) {
                    Some(c) => d2(px, py, m.x, m.y) < d2(px, py, c.x, c.y),
                    None => false,
                }
            });
            // a danger zone he's standing in, about to strike (not a gravity locked onto him: that one follows him)
            let zone_threat = zones.iter().find(|&&(zx, zy, r, at, lock)| at > 0 && tick + ZONE_REACT >= at && tick < at
                && lock != m.id + 1 && d2(m.x, m.y, zx, zy) <= sq(r + 2_000)).map(|&(zx, zy, r, _, _)| (zx, zy, r));
            let threat = incoming.map(|p| (p.x as i64, p.y as i64, None)).or(zone_threat.map(|(zx, zy, r)| (zx, zy, Some(r))));
            if let Some((px, py, zone_r)) = threat {
                // a kunai: the nearest one that actually gets him out of the way (not into a crowd or a deadly tower)
                let spot = landed.iter()
                    .filter(|&&(_, x, y)| d2(x, y, m.x, m.y) >= sq(KUNAI_DODGE_MIN) && d2(x, y, m.x, m.y) <= sq(KUNAI_DODGE_MAX)
                        && d2(x, y, px, py) >= sq(KUNAI_DODGE_MIN) && crowd(x, y) < 3 && tower_ok(x, y) && safe(x, y))
                    .min_by_key(|&&(i, x, y)| (d2(x, y, m.x, m.y), i)).copied();
                let dest = match spot {
                    Some((i, x, y)) => {
                        if let Some((b, _)) = find(m, &format!("rkl{i}:")) {
                            let n = b.name().to_string();
                            sim.entity_remove_buff(m.id, &n);
                        }
                        Some((x, y))
                    }
                    None if !m.has("hr_blink_cd") => {
                        // no kunai: a short sidestep across the shot's path, to the side with fewer enemies
                        let d = norm((m.x - px) as f64, (m.y - py) as f64);
                        // a shot: step across its path; a zone: step straight out of it
                        let sides: Vec<(i64, i64)> = match zone_r {
                            None => [(-d.1, d.0), (d.1, -d.0)].iter().map(|s| ((m.x + (s.0 * BLINK_DIST as f64) as i64).max(0), (m.y + (s.1 * BLINK_DIST as f64) as i64).max(0))).collect(),
                            Some(r) => [0.0f64, 0.6, -0.6].iter().map(|&a| { let s = rotate(d, a); ((px + (s.0 * (r + ZONE_MARGIN + 2_000) as f64) as i64).max(0), (py + (s.1 * (r + ZONE_MARGIN + 2_000) as f64) as i64).max(0)) }).collect(),
                        };
                        let best = sides.iter().copied().filter(|&(x, y)| tower_ok(x, y) && !walls::wall_at(x, y) && safe(x, y))
                            .min_by_key(|&(x, y)| (crowd(x, y), std::cmp::Reverse(enemies.iter().map(|e| d2(e.x, e.y, x, y)).min().unwrap_or(0))));
                        if best.is_some() {
                            sim.add_buff(m.id, &timed("hr_blink_cd", BLINK_CD));
                        }
                        best
                    }
                    None => None,
                };
                if let Some((x, y)) = dest {
                    let side = if x >= m.x { "after_r" } else { "after_l" };
                    let after = flash_name.replace("_flash", &format!("_{side}"));
                    fx(sim, &after, m.id, m.x, m.y, after_ticks as u64);
                    fx(sim, &flash_name, m.id, x, y, 12);
                    sim.entity_set_pos(m.id, x as u64, y as u64);
                    let mut im = timed("hr_after", after_ticks);
                    im.damaged_reduce = 100;
                    im.base_attack_damaged_reduce = 100;
                    im.skill_damaged_reduce = 100;
                    sim.add_buff(m.id, &im);
                    sim.add_buff(m.id, &timed("hr_dodge_cd", dodge_cd));
                    add_flow(sim, m);
                    return;
                }
            }
        }

        // ---- hit-and-run: flash back to the spot he struck from, if it's still safe
        if let Some((b, v)) = find(m, "rret:") {
            if v.len() >= 3 && tick as i64 >= v[2] {
                let n = b.name().to_string();
                sim.entity_remove_buff(m.id, &n);
                let (ox, oy) = (v[0], v[1]);
                if !m.stunned && !trapped && safe(ox, oy) && tower_ok(ox, oy) && crowd(ox, oy) < 2 && !walls::wall_at(ox, oy) {
                    let side = if ox >= m.x { "after_r" } else { "after_l" };
                    fx(sim, &flash_name.replace("_flash", &format!("_{side}")), m.id, m.x, m.y, 20);
                    fx(sim, &flash_name, m.id, ox, oy, 12);
                    sim.entity_set_pos(m.id, ox.max(0) as u64, oy.max(0) as u64);
                    return;
                }
            }
        }

        // ---- the charged Rasengan rides his kunai: it hit a champion → flash behind them and slam it
        if m.has("minato_rasengan") && !m.stunned && !trapped {
            if let Some(t) = enemies.iter().find(|e| e.has("minato_ras_go")).copied() {
                sim.entity_remove_buff(t.id, "minato_ras_go");
                let (bx, by) = behind(m, t);
                if safe(bx, by) && tower_ok(bx, by) && crowd(bx, by) < 3 {
                    for b in m.buffs.iter().filter(|b| b.name().starts_with("rret:")) { let n = b.name().to_string(); sim.entity_remove_buff(m.id, &n); }
                    sim.add_buff(m.id, &timed(&format!("rret:{}:{}:{}", m.x, m.y, tick + RET_DELAY), RET_DELAY + 30));
                    fx(sim, &planted_name, m.id, m.x, m.y, RET_DELAY as u64);
                    fx(sim, &flash_name, m.id, m.x, m.y, 15);
                    sim.entity_set_pos(m.id, bx as u64, by as u64);
                    fx(sim, &flash_name, m.id, bx, by, 15);
                    sim.entity_remove_buff(m.id, "minato_rasengan");
                    let (stacks, _) = flow_count(m);
                    if !crate::batch2::try_parry(sim, t, m.id, crate::batch2::hit_estimate(sim, m, 200)) {
                        let ratio = RAS_DMG.1 + RAS_PER_STACK * stacks.clamp(0, FLOW_MAX) as usize;
                        sim.deal_damage(m.id, t.id, RAS_DMG.0 + m.attack * ratio / 100, 0, AttackTypeV1::Skill);
                        crate::wave_near(sim, m.id, t.id, 20_000, RAS_DMG.0 + m.attack * ratio / 100, 0);
                        sim.apply_cc(t.id, &CcV1::of_kind(CcKindV1::Stun, RAS_STUN));
                        fx(sim, &flash_name.replace("_flash", "_rasengan_hit"), m.id, t.x, t.y, 22);
                    }
                    return;
                }
            }
        }
        for e in enemies.iter().filter(|e| e.has("minato_ras_go")) {
            sim.entity_remove_buff(e.id, "minato_ras_go");
        }

        // ---- a teleport cast goes behind a marked enemy first (his kunai marked them), using the mark up
        if (m.has("tp_now") || m.has("tp_wait")) && !escaping && !m.stunned && !trapped {
            let pick = enemies.iter().filter(|e| e.has("minato_mark") && d2(e.x, e.y, m.x, m.y) <= sq(MARK_TP_R))
                .map(|e| (*e, behind(m, e)))
                .filter(|&(_, (bx, by))| safe(bx, by) && tower_ok(bx, by) && crowd(bx, by) < 3)
                .min_by_key(|(e, _)| (if e.max_hp == 0 { 100 } else { e.hp * 100 / e.max_hp }, e.id));
            if let Some((t, (bx, by))) = pick {
                sim.entity_remove_buff(t.id, "minato_mark");
                let last = m.has("tp_last") || m.has("tp_wait_last");
                fx(sim, &flash_name, m.id, m.x, m.y, 15);
                sim.entity_set_pos(m.id, bx as u64, by as u64);
                fx(sim, &flash_name, m.id, bx, by, 15);
                arrive(sim, m, &enemies, &flash_name, bx, by, false, !last);
                for n in ["tp_now", "tp_last", "tp_wait", "tp_wait_last"] {
                    sim.entity_remove_buff(m.id, n);
                }
                return;
            }
        }

        // ---- melee dodge: an enemy champion right next to him starts a basic attack (its attack cooldown just
        //      restarted) → with a dodge stack to spend he flashes out before it lands (to a kunai nearby, else a
        //      step straight away from it), leaving the afterimage. Costs 1 stack; at most every 3 s.
        let mut swingers: Vec<usize> = Vec::new();
        for i in 0..sim.player_count() {
            let Some(p) = sim.player_at(i) else { continue };
            let Some(c) = p.champion() else { continue };
            let cid = c.id();
            let Some(e) = enemies.iter().find(|e| e.id == cid) else { continue };
            let cd = p.cooldowns().map_or(0, |c| c.0);
            let key = format!("rac{cid}:");
            let prev = find(m, &key).and_then(|(_, v)| v.first().copied()).unwrap_or(cd as i64);
            if let Some((b, _)) = find(m, &key) { let n = b.name().to_string(); sim.entity_remove_buff(m.id, &n); }
            sim.add_buff(m.id, &timed(&format!("rac{cid}:{cd}"), 3));
            if (cd as i64) > prev + 10 && d2(e.x, e.y, m.x, m.y) <= sq(MELEE_DODGE_R) {
                // aimed at him: he's the closest enemy of that champion
                let closest = all.iter().filter(|c| c.team != e.team).min_by_key(|c| (d2(c.x, c.y, e.x, e.y), c.id)).map(|c| c.id);
                if closest == Some(m.id) { swingers.push(cid); }
            }
        }
        let (stacks_now, stacks_left) = flow_count(m);
        if let Some(&sw) = swingers.first() {
            if stacks_now >= 1 && !m.stunned && !trapped && !m.has("hr_dodge_cd") && !m.has("hr_melee_cd") && !m.has("hr_after") {
                let e = enemies.iter().find(|e| e.id == sw).copied().unwrap();
                let away = norm((m.x - e.x) as f64, (m.y - e.y) as f64);
                let spot = landed.iter().filter(|&&(_, x, y)| d2(x, y, e.x, e.y) >= sq(MELEE_STEP) && d2(x, y, m.x, m.y) <= sq(KUNAI_DODGE_MAX)
                        && crowd(x, y) < 2 && tower_ok(x, y) && safe(x, y))
                    .min_by_key(|&&(i, x, y)| (d2(x, y, m.x, m.y), i)).copied();
                let dest = match spot {
                    Some((i, x, y)) => {
                        if let Some((b, _)) = find(m, &format!("rkl{i}:")) { let n = b.name().to_string(); sim.entity_remove_buff(m.id, &n); }
                        Some((x, y))
                    }
                    None => [0.0f64, 0.7, -0.7].iter().map(|&a| { let d = rotate(away, a); ((m.x + (d.0 * MELEE_STEP as f64) as i64).max(0), (m.y + (d.1 * MELEE_STEP as f64) as i64).max(0)) })
                        .find(|&(x, y)| !walls::wall_at(x, y) && tower_ok(x, y) && safe(x, y)),
                };
                if let Some((x, y)) = dest {
                    let side = if x >= m.x { "after_r" } else { "after_l" };
                    fx(sim, &flash_name.replace("_flash", &format!("_{side}")), m.id, m.x, m.y, after_ticks as u64);
                    fx(sim, &flash_name, m.id, x, y, 12);
                    sim.entity_set_pos(m.id, x as u64, y as u64);
                    let mut im = timed("hr_after", after_ticks);
                    im.damaged_reduce = 100;
                    im.base_attack_damaged_reduce = 100;
                    im.skill_damaged_reduce = 100;
                    sim.add_buff(m.id, &im);
                    sim.add_buff(m.id, &timed("hr_dodge_cd", dodge_cd));
                    sim.add_buff(m.id, &timed("hr_melee_cd", MELEE_DODGE_CD));
                    set_flow(sim, m, stacks_now - 1, stacks_left);
                    return;
                }
            }
        }

        // ---- a teleport cast with nowhere to go becomes a fresh throw instead of a wasted cast.
        //      (His data turns the 2nd and 3rd casts after a throw into teleport requests; if every kunai was
        //      used up or expired, or every spot is surrounded, that cast used to do nothing.)
        let in_flight = (0..SLOTS).any(|i| m.has(&format!("rka{i}"))) || find(m, "rkm0:").is_some() || find(m, "rkm1:").is_some();
        let just_threw = m.has("raijin_throw") || m.has("raijin_seen");
        let rethrow = |sim: &mut StableSim<'_>| {
            for n in ["tp_now", "tp_last", "tp_wait", "tp_wait_last", "raijin_3", "raijin_2"] {
                sim.entity_remove_buff(m.id, n);
            }
            // back to stage 2: the next two casts teleport to this new fan
            sim.add_buff(m.id, &timed("raijin_2", STAGE2_WINDOW));
            sim.add_buff(m.id, &timed("raijin_seen", 12));
            clear_set(sim, m, 0);
            let t = enemies.iter().min_by_key(|e| (d2(e.x, e.y, m.x, m.y), e.id));
            if let Some(t) = t {
                fx(sim, &flash_name, m.id, m.x, m.y, 10);
                throw_fan(sim, m, 0, t, &kunai_name, speed_mult, range_mult);
            }
        };
        if m.has("tp_now") && landed.is_empty() && !in_flight && !just_threw {
            rethrow(sim);
            return;
        }
        // Stage buffs left over with no kunai anywhere (all used / expired): drop them so his next cast is a
        // normal throw from the data side.
        if landed.is_empty() && !in_flight && !just_threw && !m.has("tp_now") {
            for n in ["raijin_2", "raijin_3", "tp_wait", "tp_wait_last"] {
                if m.has(n) {
                    sim.entity_remove_buff(m.id, n);
                }
            }
        }

        // ---- a teleport request: pick the kunai (or none)
        let requested = m.has("tp_now") || m.has("tp_wait");
        if requested && !landed.is_empty() {
            let last = m.has("tp_last") || m.has("tp_wait_last");
            let pick = if escaping {
                // escape: the kunai furthest from every enemy, and only if it's clearly safer than where he stands
                // (kunai thrown at the enemies to slow them are no escape: teleporting there was suicide)
                let here = nearest_enemy(m.x, m.y);
                let far_enough = |x: i64, y: i64| { let n = nearest_enemy(x, y); n >= sq(ESCAPE_MIN) && n >= here + (ESCAPE_GAIN as i128) * (ESCAPE_GAIN as i128) };
                landed.iter().filter(|&&(_, x, y)| tower_ok(x, y) && safe(x, y) && far_enough(x, y))
                    .max_by_key(|&&(i, x, y)| (tower_hit(x, y) == 0, nearest_enemy(x, y), std::cmp::Reverse(i))).copied()
            } else {
                // engage: the kunai closest to the weakest enemy, unless it drops him into 3+ enemies
                let weakest = enemies.iter().min_by_key(|e| (if e.max_hp == 0 { 100 } else { e.hp * 100 / e.max_hp }, e.id));
                let (wx, wy) = weakest.map_or((m.x, m.y), |w| (w.x, w.y));
                // a spot outside enemy towers first; a tower spot only as the last resort, and only if he
                // survives one tower shot
                // only kunai that actually put him on an enemy; the old ones lying far away don't count
                landed.iter().filter(|&&(_, x, y)| crowd(x, y) < 3 && tower_ok(x, y) && safe(x, y) && nearest_enemy(x, y) <= sq(ENGAGE_REACH))
                    .min_by_key(|&&(i, x, y)| (tower_hit(x, y) > 0, d2(x, y, wx, wy), i)).copied()
            };
            if let Some((i, x, y)) = pick {
                fx(sim, &flash_name, m.id, m.x, m.y, 15);
                sim.entity_set_pos(m.id, x.max(0) as u64, y.max(0) as u64);
                fx(sim, &flash_name, m.id, x, y, 15);
                let mut struck = false;
                if let Some((b, v)) = find(m, &format!("rkl{i}:")) {
                    struck = v.get(2).copied().unwrap_or(0) == 1;
                    let n = b.name().to_string();
                    sim.entity_remove_buff(m.id, &n);
                }
                arrive(sim, m, &enemies, &flash_name, x, y, struck, !last);
            } else if escaping {
                // no kunai is a real escape: stay put (don't teleport, don't throw)
            } else if kunai_on_enemy {
                // a kunai still lies by an enemy, it just isn't safe to land on right now (a crowd, a tower, a
                // danger zone): keep the kunai, no rethrow
            } else if let Some(t) = (pack > 0 && !pack_in_flight).then(|| quarry.or_else(|| enemies.iter()
                .filter(|e| d2(e.x, e.y, m.x, m.y) <= sq(pack_reach)).min_by_key(|e| (d2(e.x, e.y, m.x, m.y), e.id)).copied())).flatten() {
                // nothing on the ground is near an enemy, but the backpack has one: throw that (the old kunai stay)
                pack_throw(sim, m, t);
                set_pack(sim, m, pack - 1);
            } else {
                // nothing on the ground is near an enemy and the backpack is empty: a fresh fan replaces the old one
                rethrow(sim);
                return;
            }
            // answered
            for n in ["tp_now", "tp_last", "tp_wait", "tp_wait_last"] {
                sim.entity_remove_buff(m.id, n);
            }
        }
    }
}

/// Native effect used only as the "on hit" of purely visual projectiles.
#[derive(Clone, Debug)]
struct Noop;
impl StableEffectType for Noop {
    fn apply(&self, _sim: &mut StableSim<'_>, _seed: u64, _caster: usize, _input: InputTargetV1) {}
}

// ------------------------------------------------------------------ "ult learned" marker

/// Attached by a data champion as `"passive_ult": { "passive_ref": "tfm2_custom_ai:ult_learned", "params": {} }`.
/// The game switches a passive_ult on only once the ultimate is learned, so while it runs the owner keeps a
/// permanent `gojo_ult_learned` buff (re-added after respawns). Gojo's Hollow Purple checks for it.
#[derive(Clone)]
struct UltLearned;

const ULT_LEARNED: &str = "gojo_ult_learned";

fn mark_ult_learned(sim: &mut StableSim<'_>, entity: usize) {
    let has = match sim.get_entity(entity) {
        Some(e) if e.is_alive() => (0..e.buff_count()).any(|i| e.buff_at(i).map_or(false, |b| b.name() == ULT_LEARNED)),
        _ => return,
    };
    if !has {
        sim.add_buff(entity, &BuffV1::named(ULT_LEARNED));
    }
}

impl StablePassive for UltLearned {
    fn clone_box(&self) -> Box<dyn StablePassive> {
        Box::new(self.clone())
    }
    fn on_spawn(&mut self, sim: &mut StableSim<'_>, _player: usize, entity: usize) {
        mark_ult_learned(sim, entity);
    }
    fn on_update(&mut self, sim: &mut StableSim<'_>, _rng_seed: u64, _player: usize, entity: usize) {
        mark_ult_learned(sim, entity);
    }
}

// ------------------------------------------------------------------ entry

fn init(host: &StableHost) -> StableMod {
    let version = host.game_version();
    host.log(
        LogLevel::Warn,
        &format!(
            "{MOD_ID} {VERSION} loaded (game {}.{}.{}): Unlimited Void, Flying Raijin, DIO, David, V1, Vader, Frieren, Steve, Omen, Scribble, Levi, Aegis Zero, Emperor Isliid, the Coder, The Unified Theory + map plans (tactics.txt) + Mod Power; input AI: wall detours, smoke checks, Levi, Aegis and Isliid press swaps",
            version.major, version.minor, version.patch
        ),
    );
    let mut decl = StableMod::new(MOD_ID);
    decl.set_match_hook(perf::TimedHook(VoidField));
    decl.set_map_customizer(WallReader);
    decl.add_native_passive(format!("{MOD_ID}:ult_learned"), perf::Timed { name: "ult_learned", inner: UltLearned });
    decl.add_native_effect(format!("{MOD_ID}:noop"), Noop);
    decl.add_native_effect(format!("{MOD_ID}:dio_knife"), batch2::DioKnife);
    decl.add_native_effect(format!("{MOD_ID}:zoltraak"), batch2::Zoltraak);
    decl.add_native_effect(format!("{MOD_ID}:zoltraak_party"), batch2::ZoltraakParty);
    decl.add_native_passive(format!("{MOD_ID}:v1"), perf::Timed { name: "v1", inner: batch2::V1::default() });
    decl.add_native_passive(format!("{MOD_ID}:vader"), perf::Timed { name: "vader", inner: batch2::Vader::default() });
    decl.add_native_passive(format!("{MOD_ID}:david"), perf::Timed { name: "david", inner: batch2::David::default() });
    decl.add_native_passive(format!("{MOD_ID}:dio"), perf::Timed { name: "dio", inner: batch2::Dio::default() });
    decl.add_native_passive(format!("{MOD_ID}:steve"), perf::Timed { name: "steve", inner: steve::Steve::default() });
    decl.add_native_passive(format!("{MOD_ID}:omen"), perf::Timed { name: "omen", inner: valorant::Omen::default() });
    decl.add_native_effect(format!("{MOD_ID}:omen_blind"), valorant::OmenBlind);
    decl.add_native_passive(format!("{MOD_ID}:scribble"), perf::Timed { name: "scribble", inner: scribble::Scribble::default() });
    decl.add_native_passive(format!("{MOD_ID}:levi"), perf::Timed { name: "levi", inner: levi::Levi::default() });
    decl.add_native_passive(format!("{MOD_ID}:isliid"), perf::Timed { name: "isliid", inner: isliid::Isliid::default() });
    decl.add_native_effect(format!("{MOD_ID}:isliid_guidance"), isliid::Guidance);
    decl.add_native_effect(format!("{MOD_ID}:isliid_recall"), isliid::Recall);
    decl.add_native_effect(format!("{MOD_ID}:isliid_manifest"), isliid::Manifest);
    decl.add_native_effect(format!("{MOD_ID}:isliid_scar"), isliid::Scar);
    decl.add_native_passive(format!("{MOD_ID}:gundam"), perf::Timed { name: "gundam", inner: gundam::Gundam::default() });
    decl.add_native_passive(format!("{MOD_ID}:unified_theory"), perf::Timed { name: "unified_theory", inner: unified_theory::UnifiedTheory::default() });
    decl.add_native_passive(format!("{MOD_ID}:coder"), perf::Timed { name: "coder", inner: coder::Coder::default() });
    // moves only, and only while a boat wall stands (see steve::WallAi)
    decl.add_player_input_ai(perf::TimedAi(steve::WallAi));
    decl
}

declare_stable_mod!(init);

#[cfg(test)]
mod mod_power_tests {
    use super::*;
    #[test]
    fn per_champion() {
        // round 75: every mod champion has some, nobody else does
        for id in ["tfm2_custom_minato", "tfm2_custom_gojo", "tfm2_jojo_dio", "tfm2_cyberpunk_david", "tfm2_ultrakill_v1",
                   "tfm2_starwars_vader", "tfm2_frieren_frieren", "tfm2_blockcraft_steve", "tfm2_valorant_omen", "tfm2_toon_scribble"] {
            assert!(mod_power_of(id).is_some(), "{id}");
        }
        assert_eq!(mod_power_of("tfm2_jojo_dio").unwrap()[0], 45);
        assert_eq!(mod_power_of("tfm2_ultrakill_v1").unwrap()[6], 10);
        assert_eq!(mod_power_of("ninja"), None);
        assert_eq!(mod_power_of("tfm2_frieren_frieren").unwrap()[1], 40);
    }
}

#[cfg(test)]
mod version_tests {
    use super::VERSION;

    fn version_of(text: &str) -> &str {
        let rest = &text[text.find("\"version\"").expect("a version") + 9..];
        let start = rest.find('"').unwrap() + 1;
        &rest[start..start + rest[start..].find('"').unwrap()]
    }

    /// Round 108: the game disabled every champion when these disagreed. The manager's Build copies
    /// native/tfm2_custom_ai/mod.mod_info over the shipped one, so both must carry VERSION, and the champions' mod must
    /// require exactly it.
    #[test]
    fn every_version_agrees() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        for p in [root.join("mod.mod_info"), root.join("../../mods/tfm2_custom_ai/mod.mod_info")] {
            let text = std::fs::read_to_string(&p).unwrap();
            assert_eq!(version_of(&text), VERSION, "{}", p.display());
        }
        let champs = std::fs::read_to_string(root.join("../../mods/tfm2_custom/mod.mod_info")).unwrap();
        let dep = &champs[champs.find("\"tfm2_custom_ai\"").expect("the native dependency")..];
        assert_eq!(version_of(dep), format!(">={VERSION}"), "mods/tfm2_custom requires another native version");
    }
}
