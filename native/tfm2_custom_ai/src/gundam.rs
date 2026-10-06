//! Aegis Zero: the data champion owns targeting, cooldowns and immediate ally
//! shield. This passive supplies terrain push, distance-based basic attacks and
//! the ally-bound arrival. Wing buffs mirror this state machine exactly.
use mod_api_stable::{AttackTypeV1, BuffV1, CcV1, InputTargetV1, StablePassive, StableSim};
use crate::{champions, d2, sq, walls, Champ};

const ID: &str = "tfm2_gundam_aegis_zero";
const MELEE: i64 = 26_000;
const PROTECT_R: i64 = 60_000;
const LAND_R: i64 = 45_000;
const AURA_R: i64 = 52_000;
const CHARGE_SPEED: i64 = 3_500;
const CHARGE_TICKS: usize = 21;
const DEPLOY_TICKS: usize = 66;
const FLIGHT_TICKS: usize = 90;
const ZERO_TICKS: usize = 360;
const RETRACT_TICKS: usize = 45;

#[derive(Clone)]
enum Phase {
    Charge { x: i64, y: i64, dx: f64, dy: f64, start: usize, pushed: Option<usize> },
    Deploy { target: usize, start: usize, last: (i64, i64) },
    Flight { target: usize, start: usize, from: (i64, i64), last: (i64, i64), duration: usize },
    Empowered,
    Retract { start: usize },
}

#[derive(Clone, Default)]
pub struct Gundam {
    me: Option<usize>,
    phase: Option<Phase>,
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

fn living<'a>(all: &'a [Champ], id: usize) -> Option<&'a Champ> {
    all.iter().find(|c| c.id == id)
}

fn aim_target<'a>(me: &Champ, all: &'a [Champ], marker: &str, ally: bool) -> Option<&'a Champ> {
    all.iter()
        .filter(|c| c.id != me.id && (c.team == me.team) == ally && c.has(marker))
        .min_by_key(|c| d2(me.x, me.y, c.x, c.y))
}

fn put(sim: &mut StableSim<'_>, id: usize, x: i64, y: i64) {
    let _ = sim.entity_set_pos(id, x.max(0) as u64, y.max(0) as u64);
}

fn remove(sim: &mut StableSim<'_>, id: usize, names: &[&str]) {
    for n in names { sim.entity_remove_buff(id, n); }
}

fn open_wings(sim: &mut StableSim<'_>, id: usize, duration: usize) {
    sim.entity_remove_buff(id, "gdm_wing_retract");
    sim.entity_remove_buff(id, "gdm_wing_open");
    sim.add_buff(id, &BuffV1::timed("gdm_wing_open", duration));
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
    remove(sim, me, &["gdm_wing_open", "gdm_flight", "gdm_fade", "gdm_zero"]);
    clear_ally_aura(sim, me, team);
    sim.add_buff(me, &BuffV1::timed("gdm_wing_retract", RETRACT_TICKS));
    fx(sim, me, "retract", InputTargetV1::target(me));
    Phase::Retract { start: tick }
}

fn landing_spot(target: &Champ, all: &[Champ]) -> (i64, i64) {
    let nearest = all.iter().filter(|e| e.team != target.team)
        .min_by_key(|e| d2(e.x, e.y, target.x, target.y));
    let Some(enemy) = nearest else { return (target.x + 14_000, target.y) };
    let vx = (enemy.x - target.x) as f64;
    let vy = (enemy.y - target.y) as f64;
    let len = vx.hypot(vy).max(1.0);
    let wanted = (target.x + (vx / len * 14_000.0) as i64,
                  target.y + (vy / len * 14_000.0) as i64);
    if walls::wall_at(wanted.0, wanted.1) { (target.x, target.y) } else { wanted }
}

fn impact(sim: &mut StableSim<'_>, me: &Champ, x: i64, y: i64) {
    fx(sim, me.id, "landing", InputTargetV1::pos(x.max(0) as u64, y.max(0) as u64));
    let attack = sim.get_entity(me.id).map(|e| e.stat().attack).unwrap_or(68);
    let damage = 90 + attack * 70 / 100;
    for i in 0..sim.entity_count() {
        let Some(e) = sim.entity_at(i) else { continue };
        if !e.is_alive() || e.team() == me.team || e.is_tower() { continue; }
        let id = e.id();
        let champion = e.is_champion();
        let (ex, ey) = e.pos();
        if d2(ex as i64, ey as i64, x, y) > sq(LAND_R) { continue; }
        sim.deal_damage(me.id, id, damage, 0, AttackTypeV1::Skill);
        if champion { sim.apply_cc(id, &CcV1::of_kind(mod_api_stable::CcKindV1::Airborne, 60)); }
    }
}

impl Gundam {
    fn clean(&mut self, sim: &mut StableSim<'_>) {
        if let Some(me) = self.me {
            let team = sim.get_entity(me).map(|e| e.team()).unwrap_or(usize::MAX);
            remove(sim, me, &["gdm_armor", "gdm_protect", "gdm_charge", "gdm_challenge",
                "gdm_arrival_cast", "gdm_wing_open", "gdm_wing_retract", "gdm_flight",
                "gdm_zero", "gdm_fade"]);
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
}

impl StablePassive for Gundam {
    fn clone_box(&self) -> Box<dyn StablePassive> { Box::new(self.clone()) }

    fn on_spawn(&mut self, sim: &mut StableSim<'_>, _player: usize, entity: usize) {
        self.me = Some(entity);
        self.clean(sim);
    }

    fn on_dead(&mut self, sim: &mut StableSim<'_>, _player: usize) { self.clean(sim); }

    fn on_attack(&mut self, sim: &mut StableSim<'_>, _player: usize, entity: usize,
                 target: usize, damage: &mut usize) {
        let near = sim.distance_sq(entity, target) <= sq(MELEE) as u64;
        if near {
            fx(sim, entity, "saber", InputTargetV1::target(target));
        } else {
            *damage = *damage * 72 / 100;
            fx(sim, entity, "vulcan", InputTargetV1::target(target));
        }
    }

    fn on_update(&mut self, sim: &mut StableSim<'_>, _seed: u64, _player: usize, entity: usize) {
        self.me = Some(entity);
        if sim.is_end() { self.clean(sim); return; }
        let all = champions(sim);
        let Some(me) = living(&all, entity).cloned() else { self.clean(sim); return };
        let tick = sim.tick();
        self.passive(sim, &me, &all);
        self.challenge(sim, &me, &all);

        let state = self.phase.take();
        self.phase = match state {
            Some(Phase::Charge { x, y, dx, dy, start, mut pushed }) => {
                let nx = x + (dx * CHARGE_SPEED as f64) as i64;
                let ny = y + (dy * CHARGE_SPEED as f64) as i64;
                let wall = walls::wall_at(nx + (dx * 13_000.0) as i64,
                                          ny + (dy * 13_000.0) as i64);
                if pushed.is_none() {
                    pushed = all.iter().filter(|c| c.team != me.team &&
                        d2(c.x, c.y, nx, ny) <= sq(15_000))
                        .min_by_key(|c| d2(c.x, c.y, nx, ny)).map(|c| c.id);
                    if let Some(id) = pushed { fx(sim, me.id, "charge_hit", InputTargetV1::target(id)); }
                }
                let done = tick.saturating_sub(start) >= CHARGE_TICKS || wall;
                if wall {
                    if let Some(id) = pushed {
                        sim.apply_cc(id, &CcV1::stun(60));
                        fx(sim, me.id, "wall_hit", InputTargetV1::target(id));
                    }
                } else {
                    put(sim, me.id, nx, ny);
                    if let Some(id) = pushed {
                        if living(&all, id).is_some() {
                            put(sim, id, nx + (dx * 12_000.0) as i64,
                                          ny + (dy * 12_000.0) as i64);
                            sim.apply_cc(id, &CcV1::stun(2));
                        } else { pushed = None; }
                    }
                }
                if done {
                    sim.entity_remove_buff(me.id, "gdm_charge");
                    if let Some(id) = pushed { sim.deal_damage(me.id, id, 55 + me.attack / 2, 0, AttackTypeV1::Skill); }
                    None
                } else { Some(Phase::Charge { x: nx, y: ny, dx, dy, start, pushed }) }
            }
            Some(Phase::Deploy { target, start, mut last }) => {
                if let Some(a) = living(&all, target) { last = (a.x, a.y); }
                if living(&all, target).is_none() {
                    Some(retract(sim, me.id, me.team, tick))
                } else if tick.saturating_sub(start) >= DEPLOY_TICKS {
                    open_wings(sim, me.id, 540);
                    sim.add_buff(me.id, &BuffV1::timed("gdm_flight", 130));
                    fx(sim, me.id, "takeoff", InputTargetV1::target(me.id));
                    let duration = FLIGHT_TICKS;
                    Some(Phase::Flight { target, start: tick, from: (me.x, me.y), last, duration })
                } else { Some(Phase::Deploy { target, start, last }) }
            }
            Some(Phase::Flight { target, start, from, mut last, duration }) => {
                if let Some(a) = living(&all, target) { last = landing_spot(a, &all); }
                let elapsed = tick.saturating_sub(start);
                if elapsed >= duration {
                    let pos = if walls::wall_at(last.0, last.1) { from } else { last };
                    put(sim, me.id, pos.0, pos.1);
                    sim.entity_remove_buff(me.id, "gdm_flight");
                    impact(sim, &me, pos.0, pos.1);
                    open_wings(sim, me.id, ZERO_TICKS + RETRACT_TICKS + 10);
                    let mut b = BuffV1::timed("gdm_zero", ZERO_TICKS);
                    b.defence = 20;
                    b.magic_resistance = 20;
                    sim.add_buff(me.id, &b);
                    Some(Phase::Empowered)
                } else {
                    let f = elapsed as f64 / duration as f64;
                    let x = from.0 + ((last.0 - from.0) as f64 * f) as i64;
                    let y = from.1 + ((last.1 - from.1) as f64 * f) as i64;
                    put(sim, me.id, x, y);
                    if elapsed % 8 == 0 { fx(sim, me.id, "flight", InputTargetV1::target(me.id)); }
                    Some(Phase::Flight { target, start, from, last, duration })
                }
            }
            Some(Phase::Empowered) => {
                if let Some(remaining) = sim.get_entity(me.id).and_then(|e| buff_ticks(&e, "gdm_zero")) {
                    self.aura(sim, &me, &all);
                    if remaining <= 60 && !sim.get_entity(me.id).is_some_and(|e| has(&e, "gdm_fade")) {
                        sim.add_buff(me.id, &BuffV1::timed("gdm_fade", remaining));
                    }
                    Some(Phase::Empowered)
                } else { Some(retract(sim, me.id, me.team, tick)) }
            }
            Some(Phase::Retract { start }) => {
                if tick.saturating_sub(start) >= RETRACT_TICKS {
                    sim.entity_remove_buff(me.id, "gdm_wing_retract");
                    None
                } else { Some(Phase::Retract { start }) }
            }
            None => None,
        };

        if self.phase.is_none() {
            if sim.get_entity(me.id).is_some_and(|e| has(&e, "gdm_arrival_cast")) {
                sim.entity_remove_buff(me.id, "gdm_arrival_cast");
                if let Some(a) = aim_target(&me, &all, "gdm_arrival_target", true) {
                    sim.entity_remove_buff(a.id, "gdm_arrival_target");
                    self.phase = Some(Phase::Deploy { target: a.id, start: tick, last: (a.x, a.y) });
                } else if sim.get_entity(me.id).is_some_and(|e| has(&e, "gdm_arrival_target")) {
                    // Some game builds include self in AllyChampion. Redirect a
                    // self-selected cast to a real ally, preserving its shield.
                    sim.entity_remove_buff(me.id, "gdm_arrival_target");
                    if let Some(a) = all.iter().filter(|a| a.id != me.id && a.team == me.team)
                        .min_by_key(|a| d2(me.x, me.y, a.x, a.y)) {
                        let _ = sim.entity_add_shield(a.id, 320, 180);
                        self.phase = Some(Phase::Deploy { target: a.id, start: tick, last: (a.x, a.y) });
                    }
                }
            } else if sim.get_entity(me.id).is_some_and(|e| has(&e, "gdm_charge")) {
                if let Some(enemy) = aim_target(&me, &all, "gdm_charge_target", false) {
                    sim.entity_remove_buff(enemy.id, "gdm_charge_target");
                    let dx = (enemy.x - me.x) as f64;
                    let dy = (enemy.y - me.y) as f64;
                    let len = dx.hypot(dy).max(1.0);
                    self.phase = Some(Phase::Charge { x: me.x, y: me.y, dx: dx / len, dy: dy / len,
                        start: tick, pushed: None });
                } else { sim.entity_remove_buff(me.id, "gdm_charge"); }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn landing_blocks_for_ally() {
        let ally = Champ { id: 1, team: 0, x: 100_000, y: 100_000, buffs: vec![], stunned: false,
            pushed: false, hp: 100, max_hp: 200, attack: 10, name: "ally".into() };
        let enemy = Champ { id: 2, team: 1, x: 130_000, y: 100_000, buffs: vec![], stunned: false,
            pushed: false, hp: 100, max_hp: 200, attack: 10, name: "enemy".into() };
        let p = landing_spot(&ally, &[ally.clone(), enemy]);
        assert_eq!(p, (114_000, 100_000));
    }
}
