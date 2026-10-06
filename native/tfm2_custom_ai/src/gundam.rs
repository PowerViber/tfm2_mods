//! Aegis Zero: the data champion owns targeting, cooldowns and the ally's immediate shield. This passive supplies the
//! charge (terrain push), the melee basic attack and the ally-bound arrival.
//!
//! Round 88 (Rian: "the skill does nothing, just some animation"; redrawn after Destiny Gundam):
//! - One wing visual at a time: the Wings of Light are a single buff (`gdm_wings`, then `gdm_fade` for the last second
//!   of Zero Protection), opened by a one-shot `deploy` effect and closed by a one-shot `retract` effect. The old
//!   overlapping wing buffs, the flight effect spam and the CasterAnimation poses (which never showed) are gone; in
//!   flight he leaves pink afterimages.
//! - The basic attack is melee and only real basic attacks count (on_base_attack): a beam-saber cut, every third one a
//!   Palma Fiocina palm blast for 25% more.
//! - Crowd control stops the charge and the flight.
//! - While the ult runs his S1 / S2 presses would be wasted: the input AI turns them into basic attacks (press flags).
//! - gundam_log.txt (next to the DLL) gets a line for every cast the passive sees and every phase it runs, so the first
//!   live game shows whether the skills fire.
use mod_api_stable::{AttackTypeV1, BuffV1, CcKindV1, CcV1, InputTargetV1, StablePassive, StableSim};
use crate::{champions, d2, sq, walls, Champ, MOD_ID};
use std::collections::HashSet;
use std::io::Write;
use std::sync::Mutex;

const ID: &str = "tfm2_gundam_aegis_zero";
const PROTECT_R: i64 = 60_000;
const LAND_R: i64 = 45_000;
const AURA_R: i64 = 52_000;
const CHARGE_SPEED: i64 = 3_500;
const CHARGE_TICKS: usize = 21;
/// The wings open (the 6-frame deploy effect, ~22 ticks), then he takes off.
const DEPLOY_TICKS: usize = 26;
const FLIGHT_TICKS: usize = 80;
const ZERO_TICKS: usize = 360;
const FADE_TICKS: usize = 60;
/// The 6-frame retract effect (~25 ticks).
const RETRACT_TICKS: usize = 25;
/// A real basic attack lands within this many ticks of on_base_attack.
const BASE_WINDOW: usize = 30;
/// Every third basic attack is the palm blast.
const PALM_EVERY: usize = 3;
const WINGS: &str = "gdm_wings";
const FADE: &str = "gdm_fade";

#[derive(Clone, Debug, PartialEq)]
enum Phase {
    Charge { x: i64, y: i64, dx: f64, dy: f64, start: usize, pushed: Option<usize> },
    Deploy { target: usize, start: usize },
    Flight { target: usize, start: usize, from: (i64, i64), last: (i64, i64) },
    Empowered,
    Retract { start: usize },
}

#[derive(Clone, Default)]
pub struct Gundam {
    me: Option<usize>,
    phase: Option<Phase>,
    base_until: usize,
    hits: usize,
}

fn fx(sim: &mut StableSim<'_>, caster: usize, tag: &str, target: InputTargetV1) {
    let name = format!("{ID}_{tag}");
    let _ = sim.play_view_effect(&name, caster, &target, 0, 0, 0);
}

fn has(e: &mod_api_stable::StableEntity<'_, '_>, name: &str) -> bool {
    (0..e.buff_count()).filter_map(|i| e.buff_at(i)).any(|b| b.name() == name)
}

fn buff_ticks(e: &mod_api_stable::StableEntity<'_, '_>, name: &str) -> Option<usize> {
    (0..e.buff_count()).filter_map(|i| e.buff_at(i)).find(|b| b.name() == name).map(|b| b.duration_tick)
}

fn living(all: &[Champ], id: usize) -> Option<&Champ> {
    all.iter().find(|c| c.id == id)
}

fn aim_target<'a>(me: &Champ, all: &'a [Champ], marker: &str, ally: bool) -> Option<&'a Champ> {
    all.iter()
        .filter(|c| c.id != me.id && (c.team == me.team) == ally && c.has(marker))
        .min_by_key(|c| (d2(me.x, me.y, c.x, c.y), c.id))
}

fn put(sim: &mut StableSim<'_>, id: usize, x: i64, y: i64) {
    let _ = sim.entity_set_pos(id, x.max(0) as u64, y.max(0) as u64);
}

fn remove(sim: &mut StableSim<'_>, id: usize, names: &[&str]) {
    for n in names { sim.entity_remove_buff(id, n); }
}

/// The wing visual for each phase: exactly one of none / the Wings of Light / their last-second flicker.
fn wing_buff(phase: Option<&Phase>, zero_left: Option<usize>, deploy_elapsed: usize) -> Option<&'static str> {
    match phase {
        Some(Phase::Deploy { .. }) if deploy_elapsed >= DEPLOY_TICKS - 4 => Some(WINGS),
        Some(Phase::Flight { .. }) => Some(WINGS),
        Some(Phase::Empowered) => Some(if zero_left.is_some_and(|t| t <= FADE_TICKS) { FADE } else { WINGS }),
        _ => None,
    }
}

fn show_wings(sim: &mut StableSim<'_>, me: usize, want: Option<&str>) {
    for name in [WINGS, FADE] {
        let on = sim.get_entity(me).is_some_and(|e| has(&e, name));
        if Some(name) == want {
            if !on { sim.add_buff(me, &BuffV1::named(name)); }
        } else if on {
            sim.entity_remove_buff(me, name);
        }
    }
}

fn clear_ally_aura(sim: &mut StableSim<'_>, me: usize, team: usize) {
    for i in 0..sim.champion_count() {
        let id = sim.champion_id_at(i);
        if id != me && sim.get_entity(id).is_some_and(|e| e.team() == team) {
            sim.entity_remove_buff(id, "gdm_zero_ally");
        }
    }
}

fn retract(sim: &mut StableSim<'_>, me: usize, team: usize, tick: usize) -> Phase {
    remove(sim, me, &[WINGS, FADE, "gdm_zero"]);
    clear_ally_aura(sim, me, team);
    fx(sim, me, "retract", InputTargetV1::target(me));
    Phase::Retract { start: tick }
}

/// Where he lands for an ally: 14000 from them toward the nearest enemy (onto the ally if that's a wall).
fn landing_spot(target: &Champ, all: &[Champ]) -> (i64, i64) {
    let nearest = all.iter().filter(|e| e.team != target.team)
        .min_by_key(|e| (d2(e.x, e.y, target.x, target.y), e.id));
    let Some(enemy) = nearest else { return (target.x + 14_000, target.y) };
    let vx = (enemy.x - target.x) as f64;
    let vy = (enemy.y - target.y) as f64;
    let len = vx.hypot(vy).max(1.0);
    let wanted = (target.x + (vx / len * 14_000.0) as i64,
                  target.y + (vy / len * 14_000.0) as i64);
    if walls::wall_at(wanted.0, wanted.1) { (target.x, target.y) } else { wanted }
}

fn impact(sim: &mut StableSim<'_>, me: &Champ, x: i64, y: i64) -> usize {
    fx(sim, me.id, "landing", InputTargetV1::pos(x.max(0) as u64, y.max(0) as u64));
    let attack = sim.get_entity(me.id).map(|e| e.stat().attack).unwrap_or(68);
    let damage = 90 + attack * 70 / 100;
    // collect first: damage can kill and reshuffle the entity list
    let mut hit: Vec<(usize, bool)> = Vec::new();
    for i in 0..sim.entity_count() {
        let Some(e) = sim.entity_at(i) else { continue };
        if !e.is_alive() || e.team() == me.team || e.is_tower() { continue; }
        let (ex, ey) = e.pos();
        if d2(ex as i64, ey as i64, x, y) <= sq(LAND_R) { hit.push((e.id(), e.is_champion())); }
    }
    for &(id, champion) in &hit {
        sim.deal_damage(me.id, id, damage, 0, AttackTypeV1::Skill);
        if champion { sim.apply_cc(id, &CcV1::of_kind(CcKindV1::Airborne, 60)); }
    }
    hit.iter().filter(|h| h.1).count()
}

// ------------------------------------------------------------------ diagnostics

static LOGGED: Mutex<Option<HashSet<String>>> = Mutex::new(None);

/// One line in gundam_log.txt (next to the DLL); `key` keeps the same event from being written twice (the game runs
/// two simulations of each match).
fn log(sim: &StableSim<'_>, key: &str, line: &str) {
    let k = format!("{:x}.{key}", sim.seed());
    if let Ok(mut g) = LOGGED.lock() {
        let s = g.get_or_insert_with(HashSet::new);
        if s.len() > 20_000 { s.clear(); }
        if !s.insert(k) { return; }
    }
    let Some(dir) = std::env::current_exe().ok().and_then(|e| e.parent().map(|d| d.join("mods").join(MOD_ID))) else { return };
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(dir.join("gundam_log.txt")) {
        let _ = writeln!(f, "game {:x} tick {}: {line}", sim.seed(), sim.tick());
    }
}

impl Gundam {
    fn clean(&mut self, sim: &mut StableSim<'_>) {
        if let Some(me) = self.me {
            let team = sim.get_entity(me).map(|e| e.team()).unwrap_or(usize::MAX);
            remove(sim, me, &["gdm_armor", "gdm_protect", "gdm_charge", "gdm_challenge", "gdm_arrival_cast",
                WINGS, FADE, "gdm_zero"]);
            clear_ally_aura(sim, me, team);
        }
        self.phase = None;
    }

    fn passive(&self, sim: &mut StableSim<'_>, me: &Champ, all: &[Champ]) {
        let Some(e) = sim.get_entity(me.id) else { return };
        if !has(&e, "gdm_armor") {
            let mut b = BuffV1::named("gdm_armor");
            b.defence = 8;
            b.magic_resistance = 8;
            sim.add_buff(me.id, &b);
        }
        let endangered = all.iter().any(|a| a.id != me.id && a.team == me.team &&
            a.hp * 100 <= a.max_hp * 40 && d2(me.x, me.y, a.x, a.y) <= sq(PROTECT_R));
        let active = sim.get_entity(me.id).is_some_and(|e| has(&e, "gdm_protect"));
        if endangered && !active {
            let mut b = BuffV1::named("gdm_protect");
            b.defence = 10;
            b.magic_resistance = 10;
            sim.add_buff(me.id, &b);
        } else if !endangered && active {
            sim.entity_remove_buff(me.id, "gdm_protect");
        }
    }

    fn challenge(&self, sim: &mut StableSim<'_>, me: &Champ, all: &[Champ]) {
        if !sim.get_entity(me.id).is_some_and(|e| has(&e, "gdm_challenge")) { return; }
        sim.entity_remove_buff(me.id, "gdm_challenge");
        let count = all.iter().filter(|c| c.team != me.team && c.has("gdm_challenged") &&
            d2(c.x, c.y, me.x, me.y) <= sq(37_000)).count();
        if count > 0 { let _ = sim.entity_add_shield(me.id, 120 + 65 * count, 240); }
        log(sim, &format!("s2.{}", sim.tick() / 30), &format!("Arondight: Challenge, {count} champion(s) taunted"));
        if sim.get_entity(me.id).is_some_and(|e| has(&e, "gdm_zero")) {
            fx(sim, me.id, "flare", InputTargetV1::target(me.id));
        }
    }

    fn aura(&self, sim: &mut StableSim<'_>, me: &Champ, all: &[Champ]) {
        for a in all.iter().filter(|a| a.team == me.team && a.id != me.id &&
            d2(a.x, a.y, me.x, me.y) <= sq(AURA_R)) {
            let refresh = sim.get_entity(a.id).and_then(|e| buff_ticks(&e, "gdm_zero_ally"))
                .is_none_or(|t| t < 5);
            if refresh {
                sim.entity_remove_buff(a.id, "gdm_zero_ally");
                let mut b = BuffV1::timed("gdm_zero_ally", 20);
                b.defence = 8;
                b.magic_resistance = 8;
                sim.add_buff(a.id, &b);
            }
        }
    }

    /// Advance the running phase one tick.
    fn step(&mut self, sim: &mut StableSim<'_>, me: &Champ, all: &[Champ], tick: usize) {
        let held = me.stunned;
        let state = self.phase.take();
        self.phase = match state {
            Some(Phase::Charge { x, y, dx, dy, start, mut pushed }) => {
                let nx = x + (dx * CHARGE_SPEED as f64) as i64;
                let ny = y + (dy * CHARGE_SPEED as f64) as i64;
                let wall = walls::wall_at(nx + (dx * 13_000.0) as i64, ny + (dy * 13_000.0) as i64);
                if pushed.is_none() {
                    pushed = all.iter().filter(|c| c.team != me.team && d2(c.x, c.y, nx, ny) <= sq(15_000))
                        .min_by_key(|c| (d2(c.x, c.y, nx, ny), c.id)).map(|c| c.id);
                    if let Some(id) = pushed { fx(sim, me.id, "charge_hit", InputTargetV1::target(id)); }
                }
                let done = held || tick.saturating_sub(start) >= CHARGE_TICKS || wall;
                if wall {
                    if let Some(id) = pushed {
                        sim.apply_cc(id, &CcV1::stun(60));
                        fx(sim, me.id, "wall_hit", InputTargetV1::target(id));
                    }
                } else if !held {
                    put(sim, me.id, nx, ny);
                    if tick.is_multiple_of(3) {
                        fx(sim, me.id, if dx >= 0.0 { "after_r" } else { "after_l" }, InputTargetV1::pos(x.max(0) as u64, y.max(0) as u64));
                    }
                    if let Some(id) = pushed {
                        if living(all, id).is_some() {
                            put(sim, id, nx + (dx * 12_000.0) as i64, ny + (dy * 12_000.0) as i64);
                            sim.apply_cc(id, &CcV1::stun(2));
                        } else { pushed = None; }
                    }
                }
                if done {
                    sim.entity_remove_buff(me.id, "gdm_charge");
                    if let Some(id) = pushed {
                        sim.deal_damage(me.id, id, 55 + me.attack / 2, 0, AttackTypeV1::Skill);
                        fx(sim, me.id, "palm", InputTargetV1::target(id));
                    }
                    log(sim, &format!("s1end.{start}"), &format!("Palma Charge ended ({}), carried {:?}",
                        if held { "crowd control" } else if wall { "wall" } else { "distance" }, pushed));
                    None
                } else { Some(Phase::Charge { x: nx, y: ny, dx, dy, start, pushed }) }
            }
            Some(Phase::Deploy { target, start }) => {
                if living(all, target).is_none() {
                    log(sim, &format!("ult.dead.{start}"), "Wings of Light: the ally died before take-off");
                    Some(retract(sim, me.id, me.team, tick))
                } else if held {
                    log(sim, &format!("ult.cc.{start}"), "Wings of Light: stopped by crowd control before take-off");
                    Some(retract(sim, me.id, me.team, tick))
                } else if tick.saturating_sub(start) >= DEPLOY_TICKS {
                    log(sim, &format!("ult.fly.{start}"), &format!("Wings of Light: take-off toward ally {target}"));
                    Some(Phase::Flight { target, start: tick, from: (me.x, me.y), last: (me.x, me.y) })
                } else { Some(Phase::Deploy { target, start }) }
            }
            Some(Phase::Flight { target, start, from, mut last }) => {
                if let Some(a) = living(all, target) { last = landing_spot(a, all); }
                let elapsed = tick.saturating_sub(start);
                if held {
                    log(sim, &format!("ult.cc.{start}"), "Wings of Light: flight stopped by crowd control");
                    Some(retract(sim, me.id, me.team, tick))
                } else if elapsed >= FLIGHT_TICKS {
                    let pos = if walls::wall_at(last.0, last.1) { from } else { last };
                    put(sim, me.id, pos.0, pos.1);
                    let n = impact(sim, me, pos.0, pos.1);
                    let mut b = BuffV1::timed("gdm_zero", ZERO_TICKS);
                    b.defence = 20;
                    b.magic_resistance = 20;
                    sim.add_buff(me.id, &b);
                    log(sim, &format!("ult.land.{start}"), &format!("Wings of Light: landed, {n} champion(s) knocked up"));
                    Some(Phase::Empowered)
                } else {
                    // ease in and out across the map
                    let f = elapsed as f64 / FLIGHT_TICKS as f64;
                    let e = f * f * (3.0 - 2.0 * f);
                    let x = from.0 + ((last.0 - from.0) as f64 * e) as i64;
                    let y = from.1 + ((last.1 - from.1) as f64 * e) as i64;
                    if tick.is_multiple_of(3) {
                        fx(sim, me.id, if last.0 >= from.0 { "after_r" } else { "after_l" }, InputTargetV1::pos(me.x.max(0) as u64, me.y.max(0) as u64));
                    }
                    put(sim, me.id, x, y);
                    Some(Phase::Flight { target, start, from, last })
                }
            }
            Some(Phase::Empowered) => {
                if sim.get_entity(me.id).and_then(|e| buff_ticks(&e, "gdm_zero")).is_some() {
                    self.aura(sim, me, all);
                    Some(Phase::Empowered)
                } else { Some(retract(sim, me.id, me.team, tick)) }
            }
            Some(Phase::Retract { start }) => {
                if tick.saturating_sub(start) >= RETRACT_TICKS { None } else { Some(Phase::Retract { start }) }
            }
            None => None,
        };
    }

    /// Start a phase from the data's cast markers (only when none is running).
    fn start(&mut self, sim: &mut StableSim<'_>, me: &Champ, all: &[Champ], tick: usize) {
        if sim.get_entity(me.id).is_some_and(|e| has(&e, "gdm_arrival_cast")) {
            sim.entity_remove_buff(me.id, "gdm_arrival_cast");
            let target = if let Some(a) = aim_target(me, all, "gdm_arrival_target", true) {
                sim.entity_remove_buff(a.id, "gdm_arrival_target");
                Some(a.id)
            } else if sim.get_entity(me.id).is_some_and(|e| has(&e, "gdm_arrival_target")) {
                // some game builds include himself in AllyChampion: redirect to the nearest real ally, with its shield
                sim.entity_remove_buff(me.id, "gdm_arrival_target");
                let a = all.iter().filter(|a| a.id != me.id && a.team == me.team).min_by_key(|a| (d2(me.x, me.y, a.x, a.y), a.id));
                if let Some(a) = a { let _ = sim.entity_add_shield(a.id, 320, 180); }
                a.map(|a| a.id)
            } else { None };
            match target {
                Some(t) => {
                    fx(sim, me.id, "deploy", InputTargetV1::target(me.id));
                    log(sim, &format!("ult.cast.{tick}"), &format!("Wings of Light cast for ally {t}"));
                    self.phase = Some(Phase::Deploy { target: t, start: tick });
                }
                None => log(sim, &format!("ult.none.{tick}"), "Wings of Light cast, but no ally found (no flight)"),
            }
        } else if sim.get_entity(me.id).is_some_and(|e| has(&e, "gdm_charge")) {
            if let Some(enemy) = aim_target(me, all, "gdm_charge_target", false) {
                sim.entity_remove_buff(enemy.id, "gdm_charge_target");
                let dx = (enemy.x - me.x) as f64;
                let dy = (enemy.y - me.y) as f64;
                let len = dx.hypot(dy).max(1.0);
                log(sim, &format!("s1.{tick}"), &format!("Palma Charge at enemy {}", enemy.id));
                self.phase = Some(Phase::Charge { x: me.x, y: me.y, dx: dx / len, dy: dy / len, start: tick, pushed: None });
            } else {
                sim.entity_remove_buff(me.id, "gdm_charge");
                log(sim, &format!("s1.none.{tick}"), "Palma Charge cast, but the target marker wasn't found");
            }
        }
    }
}

impl StablePassive for Gundam {
    fn clone_box(&self) -> Box<dyn StablePassive> { Box::new(self.clone()) }

    fn on_spawn(&mut self, sim: &mut StableSim<'_>, _player: usize, entity: usize) {
        self.me = Some(entity);
        self.clean(sim);
    }

    fn on_dead(&mut self, sim: &mut StableSim<'_>, _player: usize) { self.clean(sim); }

    fn on_base_attack(&mut self, sim: &mut StableSim<'_>, _seed: u64, _player: usize, _entity: usize) {
        self.base_until = sim.tick() + BASE_WINDOW;
    }

    fn on_attack(&mut self, sim: &mut StableSim<'_>, _player: usize, entity: usize,
                 target: usize, damage: &mut usize) {
        // only his real basic attacks (skills and the landing also land hits)
        if sim.tick() > self.base_until { return; }
        self.base_until = 0;
        self.hits += 1;
        if self.hits.is_multiple_of(PALM_EVERY) {
            *damage = *damage * 125 / 100;
            fx(sim, entity, "palm", InputTargetV1::target(target));
        } else {
            fx(sim, entity, "saber", InputTargetV1::target(target));
        }
    }

    fn on_update(&mut self, sim: &mut StableSim<'_>, _seed: u64, player: usize, entity: usize) {
        self.me = Some(entity);
        if sim.is_end() { self.clean(sim); return; }
        let all = champions(sim);
        let Some(me) = living(&all, entity).cloned() else { self.clean(sim); return };
        let tick = sim.tick();
        if tick < 3 { log(sim, "spawn", &format!("Aegis Zero's native passive is running (player {player})")); }
        self.passive(sim, &me, &all);
        self.challenge(sim, &me, &all);
        self.step(sim, &me, &all, tick);
        if self.phase.is_none() { self.start(sim, &me, &all, tick); }
        let zero_left = sim.get_entity(me.id).and_then(|e| buff_ticks(&e, "gdm_zero"));
        let deploy_elapsed = match self.phase { Some(Phase::Deploy { start, .. }) => tick.saturating_sub(start), _ => 0 };
        show_wings(sim, me.id, wing_buff(self.phase.as_ref(), zero_left, deploy_elapsed));
        // the presses he'd use: none while the ult (or a charge) is running
        let free = self.phase.is_none() || matches!(self.phase, Some(Phase::Empowered));
        let flags = if free { crate::press::S1 | crate::press::S2 } else { 0 };
        crate::press::note(sim.seed(), player, tick, flags);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn champ(id: usize, team: usize, x: i64) -> Champ {
        Champ { id, team, x, y: 100_000, buffs: vec![], stunned: false, pushed: false, hp: 100, max_hp: 200, attack: 10, name: "c".into() }
    }
    #[test]
    fn landing_blocks_for_ally() {
        let ally = champ(1, 0, 100_000);
        let enemy = champ(2, 1, 130_000);
        assert_eq!(landing_spot(&ally, &[ally.clone(), enemy]), (114_000, 100_000));
    }
    #[test]
    fn exactly_one_wing_visual_per_phase() {
        assert_eq!(wing_buff(None, None, 0), None);
        assert_eq!(wing_buff(Some(&Phase::Deploy { target: 1, start: 0 }), None, 2), None);   // the deploy effect plays
        assert_eq!(wing_buff(Some(&Phase::Deploy { target: 1, start: 0 }), None, DEPLOY_TICKS), Some(WINGS));
        let flight = Phase::Flight { target: 1, start: 0, from: (0, 0), last: (0, 0) };
        assert_eq!(wing_buff(Some(&flight), None, 0), Some(WINGS));
        assert_eq!(wing_buff(Some(&Phase::Empowered), Some(200), 0), Some(WINGS));
        assert_eq!(wing_buff(Some(&Phase::Empowered), Some(FADE_TICKS), 0), Some(FADE));
        assert_eq!(wing_buff(Some(&Phase::Retract { start: 0 }), None, 0), None);   // the retract effect plays
        assert_eq!(wing_buff(Some(&Phase::Charge { x: 0, y: 0, dx: 1.0, dy: 0.0, start: 0, pushed: None }), None, 0), None);
    }
}
