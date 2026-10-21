//! Thin native adapter; all mana, timing, targeting and party rules live in frieren_core.
use crate::frieren_core::{Cmd, Core, Input, Stark, Unit};
use mod_api_stable::{CcKindV1, StablePassive, StableSim, StatV1, UnitAttackV1};
use std::collections::BTreeMap;
pub struct BasicDamage;
impl mod_api_stable::StableEffectType for BasicDamage{
 fn apply(&self,sim:&mut StableSim<'_>,_:u64,owner:usize,input:mod_api_stable::InputTargetV1){
  if input.kind!=mod_api_stable::InputTargetKindV1::Target.code(){return;}
  let Some(me)=sim.get_entity(owner)else{return};let Some(t)=sim.get_entity(input.target_id)else{return};
  if t.team()==me.team()||!t.is_alive()||!t.is_targetable()||matches!(t.kind(),Some(mod_api_stable::EntityKindV1::Tower|mod_api_stable::EntityKindV1::Nexus)){return;}
  let raw=(20+me.stat().magic_power*45/100)*crate::balance_data::FRIEREN_DAMAGE_PERCENT as usize/100;
  sim.deal_damage(owner,input.target_id,0,raw,mod_api_stable::AttackTypeV1::BaseAttack);
 }
 fn expected_damage(&self,s:&StatV1)->(usize,usize){(0,(20+s.magic_power*45/100)*crate::balance_data::FRIEREN_DAMAGE_PERCENT as usize/100)}
}
#[derive(Clone, Default)]
pub struct Frieren {
    pub core: Core,
    profile: crate::adapter_perf::Profile,
    stark: Option<usize>,
    observations: BTreeMap<usize, (usize, i64, i64)>,
    last_tick: Option<usize>,
    mana_cell: Option<usize>,
    owner: usize,
    leap_zone: Option<String>,
    unit_buffer: Vec<Unit>,
}
fn has(sim: &StableSim, id: usize, name: &str) -> bool {
    sim.get_entity(id).is_some_and(|e| {
        (0..e.buff_count())
            .filter_map(|i| e.buff_at(i))
            .any(|b| b.name() == name)
    })
}
pub struct UltimateLearned;
impl StablePassive for UltimateLearned {
    fn clone_box(&self) -> Box<dyn StablePassive> {
        Box::new(Self)
    }
    fn on_update(&mut self, sim: &mut StableSim, _: u64, _: usize, id: usize) {
        if !has(sim, id, "fr_ult_learned") {
            sim.add_buff(id, &crate::timed("fr_ult_learned", 5_184_000));
        }
    }
}
impl Frieren {
    fn clear_leap_zone(&mut self, sim: &mut StableSim) {
        if let Some(name) = self.leap_zone.take() {
            sim.entity_remove_buff(self.owner, &name);
        }
    }
    fn remove_stark(&mut self, sim: &mut StableSim) {
        self.clear_leap_zone(sim);
        if let Some(id) = self.stark.take() {
            if sim.get_entity(id).is_some_and(|e| e.is_alive()) {
                sim.entity_set_hp(id, 0);
            }
        }
    }
}
impl StablePassive for Frieren {
    fn clone_box(&self) -> Box<dyn StablePassive> {
        Box::new(self.clone())
    }
    fn on_spawn(&mut self, sim: &mut StableSim, _: usize, entity: usize) {
        self.owner = entity;
        if !has(sim, entity, "fr_rework") {
            sim.add_buff(entity, &crate::timed("fr_rework", 5_184_000));
        }
    }
    fn on_dead(&mut self, sim: &mut StableSim, _: usize) {
        self.remove_stark(sim);
        self.core.died();
        self.observations.clear();
        self.mana_cell = None;
    }
    fn on_update(&mut self, sim: &mut StableSim, _: u64, player: usize, owner: usize) {
        self.owner = owner;
        let tick = sim.tick();
        if self.last_tick == Some(tick) {
            return;
        }
        self.last_tick = Some(tick);
        let Some(me) = sim.get_entity(owner) else {
            self.remove_stark(sim);
            self.core.died();
            return;
        };
        self.profile.bind(sim, owner);
        let snapshot_at = self.profile.start();
        let team = me.team();
        let farming_context = crate::farming::Context::new(sim, owner);
        let mut units = std::mem::take(&mut self.unit_buffer);
        units.clear();
        units.extend(
            (0..sim.entity_count())
                .filter_map(|i| sim.entity_at(i))
                .filter(|e| e.is_alive())
                .map(|e| {
                    let (x, y) = e.pos();
                    let (hp, max_hp) = e.hp();
                    let stat = e.stat();
                    let visible = e.team() == team || sim.is_visible(team, e.id());
                    let prev = self
                        .observations
                        .get(&e.id())
                        .filter(|(t, _, _)| *t + 1 == tick);
                    Unit {
                        id: e.id(),
                        team: e.team(),
                        x: x as i64,
                        y: y as i64,
                        hp: hp as i64,
                        max_hp: max_hp as i64,
                        ap: stat.magic_power as i64,
                        visible,
                        targetable: e.is_targetable()
                            && !e.is_tower()
                            && !matches!(e.kind(), Some(mod_api_stable::EntityKindV1::Nexus)),
                        champion: e.is_champion(),
                        farmable: farming_context.eligible(e.id()),
                        controlled: (0..e.cc_count()).filter_map(|i| e.cc_at(i)).any(|c| {
                            [
                                CcKindV1::Stun,
                                CcKindV1::Airborne,
                                CcKindV1::BlockSkill,
                                CcKindV1::Fear,
                                CcKindV1::Charm,
                                CcKindV1::Taunt,
                            ]
                            .iter()
                            .any(|k| k.code() == c.kind)
                        }),
                        vx: prev.map_or(0, |(_, x0, _)| x as i64 - x0),
                        vy: prev.map_or(0, |(_, _, y0)| y as i64 - y0),
                    }
                }),
        );
        let Some(me) = units.iter().find(|u| u.id == owner).cloned() else {
            self.remove_stark(sim);
            self.core.died();
            return;
        };
        // The shared farming safety window is a decision gate, not a damage-immunity rule.
        let heroes = crate::champions(sim);
        let safe = heroes.iter().find(|u| u.id == owner).is_some_and(|u| {
            crate::farming::window(sim,u,&heroes)
        });
        if !safe {
            for u in &mut units {
                u.farmable = false;
            }
        }
        self.observations = units
            .iter()
            .filter(|u| u.visible)
            .map(|u| (u.id, (tick, u.x, u.y)))
            .collect();
        let stark = self
            .stark
            .and_then(|id| sim.get_entity(id))
            .filter(|e| e.is_alive())
            .map(|e| {
                let (x, y) = e.pos();
                Stark {
                    id: e.id(),
                    hp: e.hp().0 as i64,
                    interrupted: (0..e.cc_count()).filter_map(|i| e.cc_at(i)).any(|c| {
                        [
                            CcKindV1::Stun,
                            CcKindV1::Airborne,
                            CcKindV1::Fear,
                            CcKindV1::Charm,
                            CcKindV1::Taunt,
                            CcKindV1::ForceMove,
                        ]
                        .iter()
                        .any(|k| k.code() == c.kind)
                    }),
                    ..Stark::at(x as i64, y as i64)
                }
            });
        let unlocked = has(sim, owner, "fr_ult_learned")
            || sim
                .get_entity(owner)
                .and_then(|e| e.cooldown(mod_api_stable::SkillSlotV1::Ult))
                .is_some_and(|c| c.unlocked);
        let input = Input {
            tick,
            owner: &me,
            units: &units,
            automatic: true,
            action: None,
            ult_unlocked: unlocked,
            spell_unlocked: sim
                .get_entity(owner)
                .and_then(|e| e.cooldown(mod_api_stable::SkillSlotV1::Skill))
                .is_some_and(|c| c.unlocked),
            barrier_unlocked: sim
                .get_entity(owner)
                .and_then(|e| e.cooldown(mod_api_stable::SkillSlotV1::Skill2))
                .is_some_and(|c| c.unlocked),
            stark,
            wall: &|x, y| crate::walls::wall_at(x, y),
        };
        self.profile.finish("snapshot", snapshot_at);
        let core_at = self.profile.start();
        let commands = self.core.step(&input);
        self.profile.finish("core", core_at);
        let host_at = self.profile.start();
        if !has(sim, owner, "fr_rework") {
            sim.add_buff(owner, &crate::timed("fr_rework", 5_184_000));
        }
        for command in commands {
            match command {
                Cmd::StarkWarning { x, y, ticks } => {
                    self.clear_leap_zone(sim);
                    let name = format!("frj:{x}:{y}:40000:{}", tick + ticks);
                    sim.add_buff(owner, &crate::timed(&name, ticks + 1));
                    self.leap_zone = Some(name);
                }
                Cmd::StarkCancel => self.clear_leap_zone(sim),
                Cmd::StarkPose {
                    x,
                    y,
                    height,
                    descending,
                } => {
                    crate::fx_point(
                        sim,
                        if descending {
                            "tfm2_frieren_frieren_stark_down"
                        } else {
                            "tfm2_frieren_frieren_stark_up"
                        },
                        owner,
                        x,
                        y - height,
                        2,
                    );
                    if let Some(p) = &self.core.leap {
                        crate::fx_point(
                            sim,
                            "tfm2_frieren_frieren_stark_mark",
                            owner,
                            p.to.0,
                            p.to.1,
                            2,
                        );
                    }
                }
                Cmd::StarkLanding { x, y } => {
                    crate::fx_point(sim, "tfm2_frieren_frieren_stark_land", owner, x, y, 24);
                }
                Cmd::Control {
                    target,
                    airborne,
                    slow,
                    ticks,
                } => {
                    if airborne > 0 {
                        sim.apply_cc(
                            target,
                            &mod_api_stable::CcV1::of_kind(CcKindV1::Airborne, airborne as u64),
                        );
                    }
                    sim.entity_remove_buff(target, "stark_slow");
                    let mut b = crate::timed("stark_slow", ticks);
                    b.move_speed_mult = slow as i32;
                    sim.add_buff(target, &b);
                }
                Cmd::Damage { target, amount } => sim.deal_damage(
                    owner,
                    target,
                    0,
                    amount.max(0) as usize,
                    mod_api_stable::AttackTypeV1::Skill,
                ),
                Cmd::Shield {
                    target,
                    amount,
                    ticks,
                } => {
                    sim.entity_add_shield(target, amount.max(0) as usize, ticks);
                }
                Cmd::Empower { ticks } => {
                    let mut b = crate::timed("fr_revealed", ticks);
                    b.magic_power_mult = 25;
                    b.attack_speed_mult = 15;
                    b.range = 15_000;
                    sim.add_buff(owner, &b);
                }
                Cmd::SpawnStark { hp, attack } => {
                    self.remove_stark(sim);
                    let stat = StatV1 {
                        hp: hp as usize,
                        attack: attack as usize,
                        defence: 20,
                        magic_resistance: 15,
                        move_speed: 1000,
                        ..StatV1::default()
                    };
                    let atk = UnitAttackV1 {
                        attack_ratio: 0,
                        attack: 0,
                        range: 1,
                        cooltime: 5_184_000,
                        duration: 1,
                        start_timing: 0,
                        ..UnitAttackV1::default()
                    };
                    self.stark = sim.spawn_unit(
                        "ghoul",
                        owner,
                        team,
                        me.x.max(0) as u64,
                        me.y.max(0) as u64,
                        720,
                        &stat,
                        &atk,
                    );
                    if let Some(id) = self.stark {
                        for kind in [CcKindV1::BlockAttack, CcKindV1::BlockSkill, CcKindV1::Bind] {
                            sim.apply_cc(id, &mod_api_stable::CcV1::of_kind(kind, 720));
                        }
                        sim.add_buff(id, &crate::timed("fr_stark_owned", 720));
                    }
                }
                Cmd::RemoveStark => self.remove_stark(sim),
                Cmd::MoveStark { x, y } => {
                    if let Some(id) = self.stark {
                        if sim.get_entity(id).is_some_and(|e| e.is_alive()) {
                            sim.entity_set_pos(
                                id,
                                x.clamp(0, 960_000) as u64,
                                y.clamp(0, 960_000) as u64,
                            );
                        }
                    }
                }
                Cmd::Warning {
                    target,
                    charged,
                    ticks,
                } => {
                    sim.apply_cc(
                        owner,
                        &mod_api_stable::CcV1::of_kind(CcKindV1::BlockAttack, ticks as u64),
                    );
                    crate::fx_unit(
                        sim,
                        if charged {
                            "tfm2_frieren_frieren_fern_circle"
                        } else {
                            "tfm2_frieren_frieren_fern_s1"
                        },
                        owner,
                        owner,
                        ticks as u64,
                    );
                    // The marked enemy sees the shot coming for the whole preparation.
                    crate::fx_unit(
                        sim,
                        "tfm2_frieren_frieren_stark_mark",
                        owner,
                        target,
                        ticks as u64,
                    );
                }
                Cmd::Shot { .. } => {}
                Cmd::BarrierArt { target } => {
                    crate::fx_unit(sim, "tfm2_frieren_frieren_limit_on", owner, target, 30);
                }
                Cmd::PartyArt => {
                    crate::fx_unit(sim, "tfm2_frieren_frieren_limit_on", owner, owner, 30);
                }
            }
        }
        self.profile.finish("host_commands", host_at);
        let visual_at = self.profile.start();
        if tick.is_multiple_of(3) {
            for ray in self.core.rays.iter().take(4) {
                let angle = ((ray.dy as f64).atan2(ray.dx as f64) * 4.0 / std::f64::consts::PI)
                    .round() as i32;
                crate::fx_point(
                    sim,
                    &format!("tfm2_frieren_ray{}", angle.rem_euclid(8)),
                    owner,
                    ray.x,
                    ray.y,
                    3,
                );
            }
        }
        let cell = (self.core.mana.max(0) as usize / 300).min(20);
        if self.mana_cell != Some(cell) {
            if let Some(old) = self.mana_cell {
                sim.entity_remove_buff(owner, &format!("fr_mana{old}"));
            }
            sim.add_buff(owner, &crate::timed(&format!("fr_mana{cell}"), 5_184_000));
            self.mana_cell = Some(cell);
        }
        self.profile.finish("visual_updates", visual_at);
        self.profile.flush(sim, owner, "Frieren");
        units.clear();
        self.unit_buffer = units;
        let _ = player;
    }
}
