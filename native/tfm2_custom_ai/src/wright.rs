//! Wright's bridge: reads the sim into a `wright_core::Frame` once per tick and applies the core's effects.
//! The engine walks him and fires his rivet gun (the data basic attack); native code launches his planes (never() data
//! shells).
//!
//! Fighters and the scout drone are hidden carrier units (Raze's pattern): real and targetable, so they can be shot
//! down; their bodies are drawn as effects. The bomber, the repair aircraft, rockets and the Sky Carrier are simulated in
//! the core and drawn as effects only. Every visible move goes through `crate::motion` (round 156).
use crate::wright_core::{Core, Effect, Frame, PlaneView, Point, Unit};
use crate::wright_data as d;
use mod_api_stable::{AttackTypeV1, CcKindV1, CcV1, StablePassive, StableSim, StatV1, UnitAttackV1};
use std::collections::BTreeMap;

pub(crate) const OWN_CC: &str = "wrt_";
pub(crate) const GUARD: &str = "wrt_guard";
const HARD: [CcKindV1; 6] = [CcKindV1::Stun, CcKindV1::Airborne, CcKindV1::Taunt, CcKindV1::Fear, CcKindV1::Charm, CcKindV1::BlockSkill];

fn named(kind: CcKindV1, ticks: u64, name: &str) -> CcV1 {
    let mut c = CcV1::of_kind(kind, ticks);
    c.set_name(name);
    c
}

#[derive(Clone, Default)]
pub struct Wright {
    pub core: Core,
    profile: Option<crate::athlete_ai_core::Profile>,
    terrain: Option<(u64, Vec<bool>)>,
    /// plane key -> its carrier unit
    carriers: BTreeMap<u32, usize>,
    attack_target: Option<usize>,
    /// damage he took since the last frame (what got past the carrier's guard)
    damage_taken: i64,
    in_skill: bool,
    owner: Option<usize>,
    fx_tokens: usize,
    fx_tick: u64,
}

/// A hidden, targetable carrier unit for one of his planes; its body is drawn as effects.
fn spawn_carrier(sim: &mut StableSim, owner: usize, team: usize, pos: Point, ticks: u64, hp_pct: i64) -> Option<usize> {
    let max = sim.get_entity(owner).map_or(1000, |e| e.hp().1);
    let stat = StatV1 { hp: (max as i64 * hp_pct / 100).max(1) as usize, defence: 10, magic_resistance: 10, move_speed: 1, ..StatV1::default() };
    let attack = UnitAttackV1 { attack: 0, attack_ratio: 0, range: 1, cooltime: 5_184_000, duration: 1, start_timing: 0, ..UnitAttackV1::default() };
    let host = sim.spawn_unit("alchemist", owner, team, pos.x.max(0) as u64, pos.y.max(0) as u64, ticks, &stat, &attack)?;
    // no Bind: it would hold the carrier against its own engine glides
    for k in [CcKindV1::BlockAttack, CcKindV1::BlockSkill] {
        sim.apply_cc(host, &named(k, ticks, "wrt_plane_hold"));
    }
    sim.apply_cc(host, &named(CcKindV1::Animation, ticks, "wrt_plane_hidden"));
    sim.add_buff(host, &crate::timed(&format!("wrt_plane:{owner}"), ticks as usize));
    Some(host)
}

impl Wright {
    fn terrain(&mut self, tick: u64) -> Vec<bool> {
        if let Some((t, g)) = &self.terrain {
            if tick < t + 600 {
                return g.clone();
            }
        }
        let n = crate::wright_core::CELLS;
        let c = crate::wright_core::CELL;
        let g: Vec<bool> = (0..n * n).map(|i| crate::walls::wall_at(i % n * c + c / 2, i / n * c + c / 2)).collect();
        self.terrain = Some((tick, g.clone()));
        g
    }

    pub(crate) fn frame(&mut self, sim: &StableSim, owner: usize) -> Frame {
        let tick = sim.tick() as u64;
        let me = sim.get_entity(owner);
        let team = me.as_ref().map_or(usize::MAX, |e| e.team());
        let (pos, hp, max_hp, ap, alive) = me.as_ref().map_or((Point::default(), 0, 1, 0, false), |e| {
            let p = e.pos();
            let h = e.hp();
            (Point::new(p.0 as i64, p.1 as i64), h.0 as i64, h.1.max(1) as i64, e.stat().magic_power as i64, e.is_alive())
        });
        let stunned = me.as_ref().is_some_and(|e| {
            (0..e.cc_count()).filter_map(|i| e.cc_at(i)).any(|c| !c.name().starts_with(OWN_CC) && !crate::motion::is_self_motion(&c) && HARD.iter().any(|k| k.code() == c.kind))
        });
        let profile = crate::athlete_ai::pinned_owner(sim, owner, &mut self.profile);
        let mut values = [50; 13];
        values[..9].copy_from_slice(&profile.values);
        values[9..].copy_from_slice(&profile.sense.0);
        let farming = crate::farming::Context::new(sim, owner);
        let realms = crate::realm::realms(sim);
        let my_realm = crate::realm::realm_of(&realms, owner);
        let mine: Vec<usize> = self.carriers.values().copied().collect();
        let mut units = vec![];
        for e in (0..sim.entity_count()).filter_map(|i| sim.entity_at(i)) {
            if e.id() == owner || mine.contains(&e.id()) || !e.is_alive() || crate::realm::realm_of(&realms, e.id()) != my_realm {
                continue;
            }
            let e_team = e.team();
            let visible = e_team == team || sim.is_visible(team, e.id());
            let p = e.pos();
            let h = e.hp();
            let buffs: Vec<_> = (0..e.buff_count()).filter_map(|i| e.buff_at(i)).collect();
            let targetable = e.is_targetable();
            let kind = e.kind();
            units.push(Unit {
                id: e.id(),
                team: e_team,
                pos: if visible { Point::new(p.0 as i64, p.1 as i64) } else { Point::default() },
                hp: h.0 as i64,
                max_hp: h.1.max(1) as i64,
                radius: e.radius().max(1000) as i64,
                champion: e.is_champion(),
                tower: e.is_tower(),
                visible,
                targetable,
                farmable: farming.calm()
                    && e_team != team
                    && targetable
                    && visible
                    && !e.is_champion()
                    && farming.eligible_known(e.id(), e_team, kind, p, kind.is_none() && e.is_minion(), &buffs),
            });
        }
        // his planes' carriers: alive ones report back; dead ones are dropped (the core sees them shot down)
        let mut planes = vec![];
        self.carriers.retain(|key, id| match sim.get_entity(*id).filter(|e| e.is_alive()) {
            Some(e) => {
                let p = e.pos();
                let h = e.hp();
                planes.push(PlaneView { key: *key, pos: Point::new(p.0 as i64, p.1 as i64), hp: h.0 as i64, max_hp: h.1.max(1) as i64 });
                true
            }
            None => false,
        });
        let ult_unlocked = me.as_ref().is_some_and(|e| (0..e.buff_count()).filter_map(|i| e.buff_at(i)).any(|b| b.name() == crate::ULT_LEARNED));
        Frame {
            tick,
            owner,
            team,
            alive,
            pos,
            hp,
            max_hp,
            ap,
            stunned,
            ult_unlocked,
            units,
            terrain: self.terrain(tick),
            profile: values,
            planes,
            attack_target: self.attack_target,
            damage_taken: std::mem::take(&mut self.damage_taken),
            manual: false,
            order: None,
        }
    }

    fn fx_ok(&mut self, tick: u64, ambient: bool) -> bool {
        if tick != self.fx_tick {
            let gained = (tick.saturating_sub(self.fx_tick) / 2) as usize;
            self.fx_tokens = (self.fx_tokens + gained.max(1)).min(d::FX_TOKENS);
            self.fx_tick = tick;
        }
        let floor = if ambient { 4 } else { 0 };
        if self.fx_tokens > floor {
            self.fx_tokens -= 1;
            true
        } else {
            false
        }
    }

    pub(crate) fn apply(&mut self, sim: &mut StableSim, owner: usize, effects: Vec<Effect>) {
        let tick = sim.tick();
        let all = crate::champions_shared(sim);
        let realms = crate::realm::realms(sim);
        let reach = |a: usize, b: usize| crate::realm::can_interact(&realms, a, b) && crate::grimhald_cage::can_interact(&all, a, b);
        let team = sim.get_entity(owner).map_or(usize::MAX, |e| e.team());
        for e in effects {
            match e {
                Effect::Damage { target, amount, direct } => {
                    if !reach(owner, target) {
                        continue;
                    }
                    // a rocket or a kamikaze is a single strike: V1 may parry it
                    if direct {
                        if let Some(c) = all.iter().find(|c| c.id == target).cloned() {
                            if crate::batch2::try_parry(sim, &c, owner, amount.max(0) as usize) {
                                crate::mod_log(sim, "wright_log.txt", &format!("parry{tick}"), &format!("strike parried by {target}"));
                                continue;
                            }
                        }
                    }
                    self.in_skill = true;
                    sim.deal_damage(owner, target, 0, amount.max(0) as usize, AttackTypeV1::Skill);
                    self.in_skill = false;
                }
                Effect::Slow { target, percent, ticks } => {
                    if reach(owner, target) {
                        sim.entity_remove_buff(target, "wrt_slow");
                        let mut b = crate::timed("wrt_slow", ticks as usize);
                        b.move_speed_mult = -(percent as i32);
                        sim.add_buff(target, &b);
                    }
                }
                Effect::Stun { target, ticks } => {
                    if reach(owner, target) {
                        sim.apply_cc(target, &named(CcKindV1::Stun, ticks, "wrt_stun"));
                    }
                }
                Effect::Guard { on, pct } => {
                    // the Sky Carrier over him takes pct% of the damage he takes (its pool is kept by the core)
                    sim.entity_remove_buff(owner, GUARD);
                    if on {
                        let mut b = crate::timed(GUARD, d::CARRIER_TICKS as usize + 120);
                        b.damaged_reduce = pct.clamp(0, 100) as usize;
                        sim.add_buff(owner, &b);
                    }
                }
                Effect::Knock { target, to, ticks } => {
                    if reach(owner, target) {
                        let from = sim.get_entity(target).map(|e| e.pos()).map_or((to.x, to.y), |p| (p.0 as i64, p.1 as i64));
                        let to = crate::place::safe(&all, tick, from, (to.x, to.y));
                        crate::motion::glide(sim, target, to, ticks, crate::motion::Kind::Knock);
                    }
                }
                Effect::Heal { target, amount } => sim.heal(owner, target, amount.max(0) as usize),
                Effect::Shield { target, amount, ticks } => {
                    sim.entity_add_shield(target, amount.max(0) as usize, ticks as usize);
                    sim.entity_remove_buff(target, "wrt_shielded");
                    sim.add_buff(target, &crate::timed("wrt_shielded", ticks as usize));
                }
                Effect::Burn { target, ticks } => {
                    if reach(owner, target) {
                        sim.entity_remove_buff(target, "wrt_burn");
                        sim.add_buff(target, &crate::timed("wrt_burn", ticks as usize));
                    }
                }
                Effect::Taunt { target, key, ticks } => {
                    if let Some(&carrier) = self.carriers.get(&key) {
                        if reach(owner, target) {
                            let mut taunt = named(CcKindV1::Taunt, ticks, "wrt_taunt");
                            taunt.target = carrier;
                            sim.apply_cc(target, &taunt);
                        }
                    }
                }
                Effect::SpawnPlane { key, kind, pos, ticks, hp_pct } => match spawn_carrier(sim, owner, team, pos, ticks, hp_pct) {
                    Some(id) => {
                        self.carriers.insert(key, id);
                    }
                    None => {
                        // A known rejected spawn is not an unreported receipt.
                        // Remove it immediately instead of letting the core's
                        // one-tick receipt grace produce an invisible attack.
                        self.core.planes.retain(|p|p.key!=key);
                        crate::mod_log(sim,"wright_log.txt",&format!("plane{tick}"),&format!("{}: the host refused the carrier",kind.tag()));
                    }
                },
                Effect::PlaneMove { key, to, ticks } => {
                    if let Some(&c) = self.carriers.get(&key) {
                        let from = sim.get_entity(c).map(|e| e.pos()).map_or((to.x, to.y), |p| (p.0 as i64, p.1 as i64));
                        let to = crate::place::safe(&all, tick, from, (to.x, to.y));
                        crate::motion::glide(sim, c, to, ticks, crate::motion::Kind::Knock);
                    }
                }
                Effect::RemovePlane { key } => {
                    if let Some(c) = self.carriers.remove(&key) {
                        sim.entity_set_hp(c, 0);
                    }
                }
                Effect::RepairPlane { key, pct } => {
                    if let Some(&c) = self.carriers.get(&key) {
                        let max = sim.get_entity(c).map_or(0, |e| e.hp().1);
                        sim.heal(owner, c, max * pct.max(0) as usize / 100);
                    }
                }
                Effect::Stock { count } => {
                    // the workshop counter, for the HUD and logs
                    for n in 0..=d::WORKSHOP_MAX_STOCK {
                        sim.entity_remove_buff(owner, &format!("wrt_airframes:{n}"));
                    }
                    sim.add_buff(owner, &crate::timed(&format!("wrt_airframes:{count}"), 5_184_000));
                }
                Effect::Fx { name, at, ticks, ambient } => {
                    if !ambient || self.fx_ok(tick as u64, true) {
                        let tag=crate::wright_art::effect_name(&name,ticks);
                        // The carrier's shadow follows him while it falls.
                        let target=if name=="wrt_carrier_shadow" {Some(owner)}else{None};
                        if let Some(id)=target {crate::fx_unit(sim,&tag,owner,id,ticks);}
                        else {crate::fx_point(sim,&tag,owner,at.x,at.y,ticks);}
                    }
                }
                Effect::Pose { name, ticks } => sim.apply_cc(owner, &named(CcKindV1::Animation, ticks, name)),
                Effect::Log(line) => crate::mod_log(sim, "wright_log.txt", &format!("{tick}:{line}"), &line),
            }
        }
    }

    fn clear(&mut self, sim: &mut StableSim) {
        for (_, c) in std::mem::take(&mut self.carriers) {
            sim.entity_set_hp(c, 0);
        }
    }
}

impl StablePassive for Wright {
    fn clone_box(&self) -> Box<dyn StablePassive> {
        Box::new(self.clone())
    }
    fn on_spawn(&mut self, sim: &mut StableSim, _: usize, id: usize) {
        self.owner = Some(id);
        sim.entity_remove_buff(id, GUARD);
        self.damage_taken = 0;
        self.core.respawn(sim.tick() as u64);
        self.clear(sim);
        self.attack_target = None;
    }
    /// His own hits never cross a cage or a Main Event ring; his last target is the one his fighters join.
    fn on_attack(&mut self, sim: &mut StableSim, _: usize, id: usize, target: usize, damage: &mut usize) {
        let all = crate::champions_shared(sim);
        if !crate::grimhald_cage::can_interact(&all, id, target) {
            *damage = 0;
            return;
        }
        if !self.in_skill {
            self.attack_target = Some(target);
        }
    }
    /// What got past the carrier's guard drains its pool (the core scales it back up).
    fn on_damaged(&mut self, _: &mut StableSim, _: usize, entity: usize, _: usize, damage: usize) {
        if Some(entity) == self.owner {
            self.damage_taken += damage as i64;
        }
    }
    fn on_update(&mut self, sim: &mut StableSim, _: u64, _: usize, id: usize) {
        self.owner = Some(id);
        let f = self.frame(sim, id);
        if !f.alive {
            self.clear(sim);
            self.core.respawn(f.tick);
            self.attack_target=None;
            for count in 0..=d::WORKSHOP_MAX_STOCK {sim.entity_remove_buff(id,&format!("wrt_airframes:{count}"));}
            return;
        }
        let effects = self.core.step(&f);
        self.apply(sim, id, effects);
    }
}
