//! Thin host adapter. Core IDs are owner-local and never assume host allocation order.
use crate::steve_craft_core::*;
use crate::steve_craft_geometry::*;
use mod_api_stable::{
    AttackTypeV1, CcKindV1, CcV1, InputTargetV1, StableEffectType, StablePassive, StableSim,
    StatV1, UnitAttackV1,
};
use std::collections::BTreeMap;
#[cfg(test)]
#[path = "steve_craft_bridge_tests.rs"]
mod tests;
#[derive(Clone, Default)]
pub struct Steve {
    pub core: Core,
    owner: usize,
    units: BTreeMap<u32, usize>,
    published: Vec<String>,
    hud: Option<(Form, usize, usize)>,
    seen: BTreeMap<usize, (i64, i64, usize)>,
    facing: crate::facing::Facing,
    attack_cds: BTreeMap<usize, usize>,
}
pub struct BasicImpact;
impl StableEffectType for BasicImpact {
    fn apply(&self, sim: &mut StableSim, _: u64, caster: usize, input: InputTargetV1) {
        if input.kind != mod_api_stable::InputTargetKindV1::Target.code() {
            return;
        }
        let Some(e) = sim.get_entity(caster) else {
            return;
        };
        let ad = e.stat().attack;
        let Some(u) = sim.get_entity(input.target_id) else {
            return;
        };
        if !u.is_alive() || !u.is_targetable() || u.team() == e.team() {
            return;
        }
        let name = format!("sc_hit:{}:{}", sim.tick(), input.target_id);
        if has(sim, caster, &name) {
            return;
        }
        let before = u.hp().0.saturating_add(u.shield());
        sim.deal_damage(caster, input.target_id, ad, 0, AttackTypeV1::BaseAttack);
        let after = sim
            .get_entity(input.target_id)
            .map_or(0, |e| e.hp().0.saturating_add(e.shield()));
        // The primary attack owns item procs. Bonuses are separate skill damage.
        if before > after {
            sim.add_buff(caster, &crate::timed(&name, 3));
        }
    }
    fn expected_damage(&self, s: &StatV1) -> (usize, usize) {
        (s.attack, 0)
    }
}
fn has(sim: &StableSim, id: usize, name: &str) -> bool {
    sim.get_entity(id).is_some_and(|e| {
        (0..e.buff_count())
            .filter_map(|i| e.buff_at(i))
            .any(|b| b.name() == name)
    })
}
fn cc(sim: &StableSim, id: usize, k: CcKindV1) -> bool {
    sim.get_entity(id).is_some_and(|e| {
        (0..e.cc_count())
            .filter_map(|i| e.cc_at(i))
            .any(|c| c.kind == k.code())
    })
}
impl Steve {
    fn snapshot(&self, sim: &StableSim, owner: usize) -> Vec<Unit> {
        let team = sim.get_entity(owner).map_or(0, |e| e.team());
        (0..sim.entity_count())
            .filter_map(|n| sim.entity_at(n))
            .filter(|e| e.is_alive() || e.id() == owner)
            .map(|e| {
                let (x, y) = e.pos();
                let (hp, max) = e.hp();
                let (immune, cc_immune) = (0..e.buff_count()).filter_map(|n| e.buff_at(n)).fold(
                    (false, false),
                    |(a, b), v| {
                        (
                            a || v.damaged_reduce >= 100
                                || (v.base_attack_damaged_reduce >= 100
                                    && v.skill_damaged_reduce >= 100),
                            b || v.cc_immune,
                        )
                    },
                );
                let vx = self
                    .seen
                    .get(&e.id())
                    .filter(|v| sim.tick() > v.2)
                    .map_or(0, |v| (x as i64 - v.0) / (sim.tick() - v.2) as i64);
                let vy = self
                    .seen
                    .get(&e.id())
                    .filter(|v| sim.tick() > v.2)
                    .map_or(0, |v| (y as i64 - v.1) / (sim.tick() - v.2) as i64);
                Unit {
                    id: e.id(),
                    team: e.team(),
                    x: x as i64,
                    y: y as i64,
                    hp: hp as i64,
                    max_hp: max as i64,
                    ad: e.stat().attack as i64,
                    shield: e.shield() as i64,
                    visible: e.team() == team || sim.is_visible(team, e.id()),
                    targetable: e.is_targetable(),
                    champion: e.is_champion(),
                    farmable: crate::farming::eligible(sim, owner, e.id()),
                    immune,
                    cc_immune,
                    tower: e.is_tower()
                        || matches!(e.kind(), Some(mod_api_stable::EntityKindV1::Nexus)),
                    gadget: (0..e.buff_count())
                        .filter_map(|n| e.buff_at(n))
                        .any(|b| b.name().starts_with("sc_owner:")),
                    immovable: e.is_tower()
                        || matches!(
                            e.kind(),
                            Some(
                                mod_api_stable::EntityKindV1::Epic
                                    | mod_api_stable::EntityKindV1::Serpen
                                    | mod_api_stable::EntityKindV1::Nexus
                            )
                        ),
                    vx,
                    vy,
                    swing: self.attack_cds.get(&e.id()).is_some_and(|prev| {
                        e.cooldown(mod_api_stable::SkillSlotV1::Attack)
                            .is_some_and(|c| c.remaining > *prev + 10)
                    }),
                }
            })
            .collect()
    }
    fn remove(&mut self, sim: &mut StableSim, id: u32) {
        if let Some(entity) = self.units.remove(&id) {
            if has(sim, entity, &format!("sc_owner:{}:{id}", self.owner)) {
                sim.entity_set_hp(entity, 0);
            }
        }
    }
    fn spawn(&mut self, sim: &mut StableSim, id: u32, p: Point, hp: i64) -> bool {
        let Some(team) = sim.get_entity(self.owner).map(|e| e.team()) else {
            return false;
        };
        let stat = StatV1 {
            hp: hp.max(1) as usize,
            defence: 15,
            magic_resistance: 15,
            move_speed: 1,
            ..Default::default()
        };
        let atk = UnitAttackV1 {
            attack: 0,
            attack_ratio: 0,
            range: 1,
            cooltime: 100000,
            duration: 10,
            start_timing: 5,
            cancelable: true,
            attack_type: AttackTypeV1::BaseAttack.code(),
            ..Default::default()
        };
        let Some(entity) = sim.spawn_unit(
            "alchemist",
            self.owner,
            team,
            p.0 as u64,
            p.1 as u64,
            5184000,
            &stat,
            &atk,
        ) else {
            return false;
        };
        sim.entity_set_hp(entity, hp as usize);
        for k in [CcKindV1::Bind, CcKindV1::BlockAttack, CcKindV1::BlockSkill] {
            sim.apply_cc(entity, &CcV1::of_kind(k, 5184000));
        }
        sim.add_buff(
            entity,
            &crate::timed(&format!("sc_owner:{}:{id}", self.owner), 5184000),
        );
        self.units.insert(id, entity);
        true
    }
    fn warning(&self, sim: &mut StableSim, p: Point, r: i64, end: usize, hazard: bool) {
        let t = sim.tick();
        if end <= t {
            return;
        }
        let name = format!("scw:{}:{}:{r}:{end}:{}", p.0, p.1, hazard as u8);
        sim.add_buff(self.owner, &crate::timed(&name, end - t + 1));
        crate::steve::note_map_buff(sim.seed());
        crate::fx_point(sim, &format!("tfm2_blockcraft_craft_warn{}", r/1000), self.owner, p.0, p.1, (end-t) as u64);
    }
    fn execute(&mut self, sim: &mut StableSim, commands: Vec<Cmd>) {
        let tick = sim.tick();
        for command in commands {
            match command {
                Cmd::Damage {
                    event,
                    target,
                    ad,
                    ap,
                    ..
                } => {
                    let before = sim
                        .get_entity(target)
                        .map_or(0, |e| e.hp().0.saturating_add(e.shield()));
                    sim.deal_damage(
                        self.owner,
                        target,
                        ad.max(0) as usize,
                        ap.max(0) as usize,
                        AttackTypeV1::Skill,
                    );
                    let after = sim
                        .get_entity(target)
                        .map_or(0, |e| e.hp().0.saturating_add(e.shield()));
                    self.core
                        .receipt(event, before.saturating_sub(after) as i64, tick);
                }
                Cmd::Heal(id, n) => {
                    sim.heal(self.owner, id, n.max(0) as usize);
                }
                Cmd::Shield(id, n, t) => {
                    sim.entity_add_shield(id, n.max(0) as usize, t);
                }
                Cmd::Move(id, p) => {
                    if id != self.owner {
                        sim.add_buff(id, &crate::timed("stv_reeled", 3));
                    }
                    sim.entity_set_pos(
                        id,
                        p.0.clamp(0, 959999) as u64,
                        p.1.clamp(0, 959999) as u64,
                    );
                }
                Cmd::Push(id, p, t) => {
                    if let Some(e) = sim.get_entity(id) {
                        let a = e.pos();
                        let a = (a.0 as i64, a.1 as i64);
                        let d = length(a, p).max(1);
                        let mut c = CcV1::of_kind(CcKindV1::ForceMove, t as u64);
                        c.dx = (p.0 - a.0) * 1000 / d;
                        c.dy = (p.1 - a.1) * 1000 / d;
                        c.speed = (d / t.max(1) as i64) as u64;
                        sim.apply_cc(id, &c);
                    }
                }
                Cmd::Slow(id, pct, t) => {
                    let mut b = crate::timed(&format!("sc_slow{}", self.owner), t);
                    b.move_speed_mult = -(pct as i32);
                    sim.entity_remove_buff(id, b.name());
                    sim.add_buff(id, &b);
                }
                Cmd::Speed(id, pct, t) => {
                    let mut b = crate::timed(&format!("sc_speed{}", self.owner), t);
                    b.move_speed_mult = pct as i32;
                    sim.entity_remove_buff(id, b.name());
                    sim.add_buff(id, &b);
                }
                Cmd::Spawn {
                    object, point, hp, ..
                } => {
                    let ok = self.spawn(sim, object, point, hp);
                    self.core.allocated(object, ok);
                }
                Cmd::Remove(id) => self.remove(sim, id),
                Cmd::Warning {
                    point,
                    radius,
                    until,
                    owner_hazard,
                } => {
                    self.warning(sim, point, radius, until, owner_hazard);
                }
                Cmd::PathWarning { path, until } => {
                    for pair in path.windows(2) {
                        let len = length(pair[0], pair[1]);
                        let n = (len / 16000).max(1);
                        for k in 0..=n {
                            self.warning(
                                sim,
                                toward(pair[0], pair[1], len * k / n),
                                10000,
                                until,
                                false,
                            );
                        }
                    }
                }
                Cmd::Lock { ticks, pose } => {
                    let mut c = CcV1::of_kind(CcKindV1::Animation, ticks as u64);
                    c.set_name(if pose == 2 {
                        "ult"
                    } else if pose == 1 {
                        "skill2"
                    } else {
                        "skill1"
                    });
                    sim.apply_cc(self.owner, &c);
                    for k in [CcKindV1::Bind, CcKindV1::BlockAttack] {
                        sim.apply_cc(self.owner, &CcV1::of_kind(k, ticks as u64));
                    }
                }
                Cmd::Immune(n) => {
                    let mut b = crate::timed("stv_riding", n);
                    b.cc_immune = true;
                    sim.entity_remove_buff(self.owner, "stv_riding");
                    sim.add_buff(self.owner, &b);
                }
                Cmd::Fx { kind, point } => {
                    crate::fx_point(
                        sim,
                        &format!("tfm2_blockcraft_craft_fx{kind}"),
                        self.owner,
                        point.0,
                        point.1,
                        18,
                    );
                }
            }
        }
    }
    fn publish(&mut self, sim: &mut StableSim) {
        let t = sim.tick();
        let mut names = vec![];
        for w in &self.core.walls {
            if t >= w.end
                || w.object
                    .is_some_and(|id| !self.core.objects.iter().any(|o| o.id == id && o.confirmed))
            {
                continue;
            }
            for (n, p) in w.path.windows(2).enumerate() {
                names.push((
                    format!(
                        "sbw:{}:{}:{}:{}:{}:{}:{}",
                        p[0].0, p[0].1, p[1].0, p[1].1, w.begins[n], w.finishes[n], w.end
                    ),
                    w.end,
                ));
            }
        }
        for n in &self.published {
            if !names.iter().any(|(v, _)| n == v) {
                sim.entity_remove_buff(self.owner, n);
            }
        }
        for (n, end) in &names {
            if !self.published.contains(n) {
                sim.add_buff(self.owner, &crate::timed(n, end - t + 1));
                crate::steve::note_map_buff(sim.seed());
            }
        }
        self.published = names.into_iter().map(|v| v.0).collect();
        let ready = self.core.available(t);
        let reserved = self
            .core
            .charges
            .iter()
            .filter(|v| matches!(v, Charge::Reserved))
            .count();
        let hud = (self.core.form, ready, reserved);
        if self.hud != Some(hud) {
            if let Some((f, a, b)) = self.hud {
                sim.entity_remove_buff(
                    self.owner,
                    &format!(
                        "sc_hud_{}_{}_{}",
                        if f == Form::Steve { 0 } else { 1 },
                        a,
                        b
                    ),
                );
            }
            sim.add_buff(
                self.owner,
                &crate::timed(
                    &format!(
                        "sc_hud_{}_{}_{}",
                        if hud.0 == Form::Steve { 0 } else { 1 },
                        ready,
                        reserved
                    ),
                    5184000,
                ),
            );
            self.hud = Some(hud);
        }
        if self.core.ride.is_none() && !self.core.reels.is_empty() {
            for r in &self.core.reels {
                sim.add_buff(r.target, &crate::timed("stv_reeled", 3));
            }
        }
        // Compact, bounded presentation windows; one shared clock, no gameplay randomness.
        if t % 8 != 0 {
            return;
        }
        let mut budget = 64usize;
        let emit = |sim: &mut StableSim, tag: &str, p: Point, budget: &mut usize| {
            if *budget > 0 {
                crate::fx_point(
                    sim,
                    &format!("tfm2_blockcraft_craft_{tag}"),
                    self.owner,
                    p.0,
                    p.1,
                    10,
                );
                *budget -= 1;
            }
        };
        if self.core.form == Form::Herobrine {
            let p = sim
                .get_entity(self.owner)
                .map(|e| e.pos())
                .unwrap_or_default();
            emit(
                sim,
                if self.facing.left() { "eyes_l" } else { "eyes" },
                (p.0 as i64, p.1 as i64),
                &mut budget,
            );
        }
        for o in self.core.objects.iter().filter(|o| o.confirmed && o.hp > 0) {
            emit(sim, &format!("object{}", o.recipe), o.point, &mut budget);
        }
        for s in &self.core.shots {
            emit(sim, &format!("tool{}", s.recipe), s.point, &mut budget);
        }
        if let Some(p) = &self.core.pending {
            let pos = sim
                .get_entity(self.owner)
                .map(|e| e.pos())
                .unwrap_or_default();
            let left = match &p.action {
                Action::Recipe { point, .. } | Action::Deploy(point) => point.0 < pos.0 as i64,
                _ => false,
            };
            emit(
                sim,
                if left { "cast_l" } else { "cast_r" },
                (pos.0 as i64, pos.1 as i64),
                &mut budget,
            );
        }
        for (a, b) in self.core.segments(t) {
            let len = length(a, b);
            let n = (len / 14000).max(1);
            for k in 0..=n {
                emit(sim, "wall", toward(a, b, len * k / n), &mut budget);
            }
        }
        for &(a,b,_) in &self.core.trails{let len=length(a,b);let n=(len/12000).max(1);for k in 0..=n{if budget>0{let p=toward(a,b,len*k/n);crate::fx_point(sim,"tfm2_blockcraft_steve_ice",self.owner,p.0,p.1,10);budget-=1;}}}
        if let Some(r) = &self.core.ride {
            let (p, _) = at(&r.path, t.saturating_sub(r.start) as i64 * 2500);
            emit(sim, "boat", p, &mut budget);
        }
    }
}
impl StablePassive for Steve {
    fn clone_box(&self) -> Box<dyn StablePassive> {
        Box::new(self.clone())
    }
    fn on_spawn(&mut self, sim: &mut StableSim, _: usize, id: usize) {
        self.owner = id;
        sim.add_buff(id, &crate::timed("sc_rework", 5184000));
    }
    fn on_dead(&mut self, sim: &mut StableSim, _: usize) {
        let commands = self.core.died(sim.tick());
        self.execute(sim, commands);
        for n in self.published.drain(..) {
            sim.entity_remove_buff(self.owner, &n);
        }
        let names = sim
            .get_entity(self.owner)
            .map(|e| {
                (0..e.buff_count())
                    .filter_map(|n| e.buff_at(n))
                    .map(|b| b.name().to_string())
                    .filter(|n| n.starts_with("sc_") || n.starts_with("scw:") || n == "stv_riding")
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        for n in names {
            sim.entity_remove_buff(self.owner, &n);
        }
        self.hud = None;
        self.seen.clear();
    }
    fn on_update(&mut self, sim: &mut StableSim, _: u64, _: usize, owner: usize) {
        let t = sim.tick();
        if self.core.last_tick == Some(t) {
            return;
        }
        self.owner = owner;
        let all = self.snapshot(sim, owner);
        let Some(me) = all.iter().find(|e| e.id == owner) else {
            return;
        };
        let mut basics = vec![];
        let names = sim
            .get_entity(owner)
            .map(|e| {
                (0..e.buff_count())
                    .filter_map(|i| e.buff_at(i))
                    .map(|b| b.name().to_string())
                    .filter(|n| n.starts_with("sc_hit:"))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        for n in names {
            let v: Vec<usize> = n
                .split(':')
                .skip(1)
                .filter_map(|s| s.parse().ok())
                .collect();
            if v.len() == 2 && v[0] <= t && t - v[0] <= 2 {
                basics.push(((((v[0] + 1) as u64) << 32) | (v[1] + 1) as u64, v[1]));
            }
            sim.entity_remove_buff(owner, &n);
        }
        basics.sort_unstable();
        let unlocked = [
            mod_api_stable::SkillSlotV1::Skill,
            mod_api_stable::SkillSlotV1::Skill2,
            mod_api_stable::SkillSlotV1::Ult,
        ]
        .map(|s| {
            sim.get_entity(owner)
                .and_then(|e| e.cooldown(s))
                .is_some_and(|v| v.unlocked)
        });
        let controlled = [
            CcKindV1::Stun,
            CcKindV1::Airborne,
            CcKindV1::BlockSkill,
            CcKindV1::Fear,
            CcKindV1::Charm,
            CcKindV1::Taunt,
            CcKindV1::ForceMove,
        ]
        .into_iter()
        .any(|k| cc(sim, owner, k));
        let bound = cc(sim, owner, CcKindV1::BlockMoveSkill)
            || (self.core.pending.is_none() && cc(sim, owner, CcKindV1::Bind));
        let objects: Vec<_> = self
            .units
            .iter()
            .map(|(&id, &host)| {
                (
                    id,
                    if has(sim, host, &format!("sc_owner:{owner}:{id}")) {
                        sim.get_entity(host).map_or(0, |e| e.hp().0 as i64)
                    } else {
                        0
                    },
                )
            })
            .collect();
        let allies: Vec<_> = all
            .iter()
            .filter(|e| e.id != owner && e.team == me.team && e.champion && e.hp > 0)
            .collect();
        let home = if allies.is_empty() {
            if me.team == 0 {
                (96000, 864000)
            } else {
                (864000, 96000)
            }
        } else {
            (
                allies.iter().map(|e| e.x).sum::<i64>() / allies.len() as i64,
                allies.iter().map(|e| e.y).sum::<i64>() / allies.len() as i64,
            )
        };
        // V1's successful parry destroys this owner's whole ultimate, including empty boat.
        if let Some(r) = &self.core.ride {
            let (p, _) = at(&r.path, t.saturating_sub(r.start) as i64 * 2500);
            let champs = crate::champions(sim);
            if champs
                .iter()
                .filter(|e| {
                    e.team != me.team && !r.hit.contains(&e.id) && within((e.x, e.y), p, 16000)
                })
                .any(|e| {
                    crate::batch2::try_parry(sim, e, owner, (45 + me.max_hp * 5 / 100) as usize)
                })
            {
                self.core.parried();
            }
        }
        let buffs = sim
            .get_entity(owner)
            .map(|e| {
                (0..e.buff_count())
                    .filter_map(|n| e.buff_at(n))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let haste = buffs.iter().map(|b| b.skill_cooldown_mult as i64).sum();
        let ult_haste = buffs.iter().map(|b| b.ult_cooldown_mult as i64).sum();
        let world=crate::champions(sim);let barriers:Vec<_>=crate::steve::walls_up(&world,t).iter().map(|w|((w.ax,w.ay),w.built_end(t))).collect();
        let input = Input {
            tick: t,
            me,
            units: &all,
            unlocked,
            controlled,
            bound,
            automatic: true,
            action: None,
            basics: &basics,
            objects: &objects,
            wall: &crate::walls::wall_at,
            barriers: &barriers,
            home,
            haste,
            ult_haste,
        };
        let commands = self.core.step(&input);
        self.execute(sim, commands);
        self.publish(sim);
        self.facing.update(
            t,
            me.pos(),
            all.iter()
                .filter(|u| {
                    u.team != me.team && u.visible && u.hp > 0 && within(u.pos(), me.pos(), 90000)
                })
                .min_by_key(|u| (square(me.pos(), u.pos()), u.id))
                .map(|u| u.pos()),
        );
        self.attack_cds = all
            .iter()
            .filter_map(|u| {
                sim.get_entity(u.id)
                    .and_then(|e| e.cooldown(mod_api_stable::SkillSlotV1::Attack))
                    .map(|cd| (u.id, cd.remaining))
            })
            .collect();
        self.seen = all
            .iter()
            .filter(|e| e.visible)
            .map(|e| (e.id, (e.x, e.y, t)))
            .collect();
    }
}
pub fn danger_zones(
    all: &[crate::Champ],
    me: &crate::Champ,
    t: usize,
) -> Vec<(i64, i64, i64, usize, usize)> {
    let mut zones = vec![];
    for c in all {
        for b in &c.buffs {
            if let Some(raw) = b.name().strip_prefix("scw:") {
                let v: Vec<i64> = raw.split(':').filter_map(|n| n.parse().ok()).collect();
                if v.len() == 5
                    && t < v[3] as usize
                    && (c.team != me.team || (c.id == me.id && v[4] != 0))
                {
                    zones.push((v[0], v[1], v[2], v[3] as usize, 0));
                }
            }
        }
    }
    zones
}
pub fn avoid(all: &[crate::Champ], me: &crate::Champ, dest: Point, t: usize) -> Option<Point> {
    let zones = danger_zones(all, me, t);
    let bad = |p| zones.iter().any(|v| within(p, (v.0, v.1), v.2 + 2500));
    if !bad((me.x, me.y)) && !bad(dest) {
        return None;
    }
    let mut candidates = vec![];
    for v in &zones {
        let d = crate::unified_theory_math::unit(me.x - v.0, me.y - v.1);
        let d = if d == (0, 0) { (10000, 0) } else { d };
        for q in [d, (-d.1, d.0), (d.1, -d.0)] {
            candidates.push((
                v.0 + q.0 * (v.2 + 6000) / 10000,
                v.1 + q.1 * (v.2 + 6000) / 10000,
            ));
        }
    }
    candidates
        .into_iter()
        .filter(|&p| legal(p) && !crate::walls::wall_at(p.0, p.1) && !bad(p))
        .min_by_key(|&p| square((me.x, me.y), p) + square(dest, p) / 4)
}
