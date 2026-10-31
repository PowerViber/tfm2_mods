//! Native playtest adapter. Uses real entities; unsupported host isolation is never emulated by damage repair
//! (round 146 removed the outside-hit healing that 0.10.81 had added).
use crate::contender_core::*;
use mod_api_stable::{
    AttackTypeV1, BuffV1, CcKindV1, CcV1, InputTargetV1, StableEffectType, StablePassive,
    StableSim, StatV1,
};
#[derive(Clone, Default)]
pub struct Contender {
    pub core: Core,
    profile: Option<crate::athlete_ai_core::Profile>,
    serial: u64,
    receipts: Vec<Receipt>,
    last_observed: (usize, usize),
    was_arena: bool,
    history: std::collections::BTreeMap<usize, (usize, Point)>,
    reserved_target: Option<usize>,
    stamina_marker: Option<i64>,
    /// Round 146: the opponent of a duel that just opened, reported to the core once in the next frame
    /// (`arena_started`; it was never set, so the core never knew a duel had begun).
    started: Option<usize>,
}
pub struct BasicImpact;
impl StableEffectType for BasicImpact {
    fn apply(&self, sim: &mut StableSim, _: u64, caster: usize, input: InputTargetV1) {
        if input.kind == mod_api_stable::InputTargetKindV1::Target.code() {
            let tag = format!("ct_hit:{}:{}", sim.tick(), input.target_id);
            if !has(sim, caster, &tag) {
                sim.add_buff(caster, &crate::timed(&tag, 2));
            }
        }
    }
    fn expected_damage(&self, s: &StatV1) -> (usize, usize) {
        (s.attack, 0)
    }
}
pub struct UltimateLearned;
impl StablePassive for UltimateLearned {
    fn clone_box(&self) -> Box<dyn StablePassive> {
        Box::new(Self)
    }
    fn on_update(&mut self, sim: &mut StableSim, _: u64, _: usize, id: usize) {
        if !has(sim, id, "ct_ultimate") {
            sim.add_buff(id, &BuffV1::named("ct_ultimate"));
        }
    }
}
fn has(sim: &StableSim, id: usize, tag: &str) -> bool {
    sim.get_entity(id).is_some_and(|e| {
        (0..e.buff_count())
            .filter_map(|i| e.buff_at(i))
            .any(|b| b.name() == tag)
    })
}
fn clear(sim: &mut StableSim, id: usize, prefix: &str) {
    let names = sim
        .get_entity(id)
        .map(|e| {
            (0..e.buff_count())
                .filter_map(|i| e.buff_at(i))
                .filter(|b| b.name().starts_with(prefix))
                .map(|b| b.name().to_owned())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    for n in names {
        sim.entity_remove_buff(id, &n);
    }
}
fn control(sim: &StableSim, id: usize, kinds: &[CcKindV1]) -> bool {
    sim.get_entity(id).is_some_and(|e| {
        (0..e.cc_count())
            .filter_map(|i| e.cc_at(i))
            .any(|c| kinds.iter().any(|k| k.code() == c.kind))
    })
}
impl Contender {
    fn frame(&mut self, sim: &StableSim, owner: usize) -> Frame {
        let team = sim.get_entity(owner).map_or(usize::MAX, |e| e.team());
        let profile = crate::athlete_ai::pinned_owner(sim, owner, &mut self.profile);
        let farming = crate::farming::Context::new(sim, owner);
        let realms = crate::realm::realms(sim);
        let membership = |id| crate::realm::realm_of(&realms, id);
        let owner_ring = membership(owner);
        let duel_unavailable = (0..sim.champion_count())
            .map(|i| sim.champion_id_at(i))
            .filter(|&id| {
                membership(id).is_some()
                    || sim.get_entity(id).is_some_and(|e| {
                        (0..e.buff_count())
                            .filter_map(|i| e.buff_at(i))
                            .any(|b| b.name().starts_with("ct_reserve:"))
                    })
            })
            .collect();
        let mut new_history = std::collections::BTreeMap::new();
        let units = (0..sim.entity_count())
            .filter_map(|i| sim.entity_at(i))
            .filter(|e| (e.is_alive() || e.id() == owner) && owner_ring == membership(e.id()))
            .map(|e| {
                let s = e.stat();
                let p = e.pos();
                let e_team = e.team();
                let visible = e_team == team || sim.is_visible(team, e.id());
                // Round 145: each unit's buffs once, and its farm check from what is already read
                let buffs: Vec<_> = (0..e.buff_count()).filter_map(|i| e.buff_at(i)).collect();
                let targetable = e.is_targetable();
                let pos = if visible {
                    Point {
                        x: p.0 as i64,
                        y: p.1 as i64,
                    }
                } else {
                    Point::default()
                };
                let hp = e.hp();
                let velocity = if visible {
                    self.history
                        .get(&e.id())
                        .filter(|(t, _)| sim.tick() > *t && sim.tick() - *t < 120)
                        .map(|(t, p)| Point {
                            x: (pos.x - p.x) / (sim.tick() - *t) as i64,
                            y: (pos.y - p.y) / (sim.tick() - *t) as i64,
                        })
                        .unwrap_or_default()
                } else {
                    Point::default()
                };
                if visible {
                    new_history.insert(e.id(), (sim.tick(), pos));
                }
                Unit {
                    id: e.id(),
                    team: e_team,
                    pos,
                    hp: hp.0 as i64,
                    max_hp: hp.1.max(1) as i64,
                    attack: s.attack as i64,
                    radius: e.radius().max(1000) as i64,
                    champion: e.is_champion(),
                    visible,
                    targetable,
                    cc_immune: buffs.iter().any(|b| b.cc_immune),
                    // same as farming.farmable(id): calm, and eligible (a living, targetable enemy its team sees)
                    farmable: farming.calm()
                        && e_team != team
                        && e.is_alive()
                        && targetable
                        && visible
                        && {
                            let kind = e.kind();
                            farming.eligible_known(e.id(), e_team, kind, p, kind.is_none() && e.is_minion(), &buffs)
                        },
                    velocity,
                }
            })
            .collect();
        self.history = new_history;
        let mut basics = vec![];
        if let Some(e) = sim.get_entity(owner) {
            for b in (0..e.buff_count()).filter_map(|i| e.buff_at(i)) {
                if let Some(raw) = b.name().strip_prefix("ct_hit:") {
                    let v: Vec<_> = raw
                        .split(':')
                        .filter_map(|s| s.parse::<u64>().ok())
                        .collect();
                    if v.len() == 2 {
                        basics.push(Basic {
                            serial: v[0].saturating_add(1),
                            target: v[1] as usize,
                        });
                    }
                }
            }
        }
        let ring = crate::contender_arena::member(sim, owner);
        let ended = self.was_arena && ring.is_none();
        self.was_arena = ring.is_some();
        let mut values = [50; 13];
        values[..9].copy_from_slice(&profile.values);
        values[9..].copy_from_slice(&profile.sense.0);
        let bounds = ring.map_or_else(map, |r| r.bounds());
        // the ring stands on the map (0.10.81), so its walls still block
        let blocks = crate::walls::boxing_blocks();
        Frame {
            version: 1,
            tick: sim.tick() as u64,
            owner,
            units,
            blocks,
            bounds,
            profile: values,
            manual: false,
            action: None,
            skill1: true,
            skill2: sim.get_entity(owner).is_some_and(|e| e.level() >= 3),
            ultimate: has(sim, owner, "ct_ultimate"),
            interrupted: control(
                sim,
                owner,
                &[
                    CcKindV1::Stun,
                    CcKindV1::Airborne,
                    CcKindV1::BlockSkill,
                    CcKindV1::Taunt,
                    CcKindV1::Fear,
                    CcKindV1::Charm,
                ],
            ),
            rooted: control(sim, owner, &[CcKindV1::Bind, CcKindV1::BlockMoveSkill]),
            receipts: std::mem::take(&mut self.receipts),
            basics,
            arena_started: self.started.take().filter(|_| ring.is_some()),
            arena_ended: ended,
            basics_validated_by_host: true,
            dealt_at: None,
            duel_unavailable,
        }
    }
    fn effects(&mut self, sim: &mut StableSim, owner: usize, effects: Vec<Effect>) {
        for effect in effects {
            match effect {
                Effect::Damage {
                    target,
                    amount,
                    basic,
                    ..
                } => {
                    let realms = crate::realm::realms(sim);
                    if !crate::realm::can_interact(&realms, owner, target) {
                        continue;
                    }
                    crate::fx_unit(sim, "ct_impact", owner, target, 6);
                    let before = sim
                        .get_entity(target)
                        .map(|e| (e.hp().0, e.shield(), e.is_champion()));
                    sim.deal_damage(
                        owner,
                        target,
                        amount.max(0) as usize,
                        0,
                        if basic {
                            AttackTypeV1::BaseAttack
                        } else {
                            AttackTypeV1::Skill
                        },
                    );
                    if let (Some((hp, shield, champion)), Some(after)) =
                        (before, sim.get_entity(target))
                    {
                        self.core.dealt(
                            sim.tick() as u64,
                            champion,
                            after.hp().0 < hp || after.shield() < shield,
                        );
                    }
                }
                Effect::Airborne { target, ticks } => {
                    sim.apply_cc(target, &CcV1::of_kind(CcKindV1::Airborne, ticks))
                }
                Effect::AttackSlow {
                    target,
                    percent,
                    ticks,
                } => {
                    let tag = format!("ct_body:{owner}");
                    sim.entity_remove_buff(target, &tag);
                    let mut b = crate::timed(&tag, ticks as usize);
                    b.attack_speed_mult = -(percent as i32);
                    sim.add_buff(target, &b);
                }
                Effect::Shield { amount, ticks } => {
                    sim.entity_add_shield(owner, amount.max(0) as usize, ticks as usize);
                }
                Effect::Defence {
                    percent,
                    move_penalty,
                    ticks,
                } => {
                    let mut b = crate::timed("ct_guard", ticks as usize);
                    b.damaged_reduce = percent as usize;
                    b.move_speed_mult = -(move_penalty as i32);
                    sim.add_buff(owner, &b);
                    crate::fx_unit(sim, "ct_guard_arc", owner, owner, 6);
                }
                Effect::Dodge { ticks } => {
                    let mut b = crate::timed("ct_slip", ticks as usize);
                    b.damaged_reduce = 100;
                    b.cc_immune = true;
                    sim.add_buff(owner, &b);
                    sim.entity_banish(owner, owner, ticks as usize, "", "");
                }
                Effect::Move { pos } => {
                    sim.entity_set_pos(owner, pos.x.max(0) as u64, pos.y.max(0) as u64);
                }
                Effect::Warn {
                    from,
                    to,
                    radius,
                    until,
                    label,
                } => {
                    if label == "Main Event" {
                        if let Some(target) = self.core.cast.as_ref().and_then(|c| c.target) {
                            if !crate::contender_arena::reserve(
                                sim,
                                owner,
                                target,
                                until.saturating_sub(sim.tick() as u64) as usize + 1,
                            ) {
                                self.core.cast = None;
                                self.core.rejection = "Participant already reserved".into();
                                continue;
                            } else {
                                self.reserved_target = Some(target)
                            }
                        }
                    }
                    let steps = 3;
                    for n in 0..=steps {
                        let x = from.x + (to.x - from.x) * n / steps;
                        let y = from.y + (to.y - from.y) * n / steps;
                        let tag = format!("ctw:{x}:{y}:{radius}:{until}");
                        sim.add_buff(
                            owner,
                            &crate::timed(
                                &tag,
                                until.saturating_sub(sim.tick() as u64) as usize + 1,
                            ),
                        );
                    }
                }
                Effect::ClearWarnings => {
                    clear(sim, owner, "ctw:");
                    if let Some(t) = self.reserved_target.take() {
                        crate::contender_arena::release_reservation(sim, owner, t)
                    }
                }
                Effect::Transfer {
                    target, duration, ..
                } => {
                    if crate::contender_arena::enter(sim, owner, target, duration as usize) {
                        // the core learns the duel began from the next frame's arena_started
                        self.started = Some(target);
                        self.was_arena = true;
                    } else {
                        self.core.rejection =
                            "Host rejected the ring transfer; exact return attempted".into();
                    }
                }
                Effect::Pose { name } => {
                    let duration = self
                        .core
                        .cast
                        .as_ref()
                        .map_or(15, |c| c.until.saturating_sub(sim.tick() as u64).max(1));
                    let mut c = CcV1::of_kind(CcKindV1::Animation, duration);
                    c.set_name(if name == "ultimate" {
                        "mainEvent"
                    } else {
                        &name
                    });
                    sim.apply_cc(owner, &c);
                }
            }
        }
    }
}
impl StablePassive for Contender {
    fn clone_box(&self) -> Box<dyn StablePassive> {
        Box::new(self.clone())
    }
    fn on_spawn(&mut self, sim: &mut StableSim, _: usize, id: usize) {
        self.core.respawn(sim.tick() as u64);
        self.stamina_marker = None;
        if let Some(e) = sim.get_entity(id) {
            self.last_observed = (e.hp().0, e.shield());
        }
    }
    // Contender's own hits, gated before they land: inside Main Event he only hurts his realm, and outside it he
    // never hurts anyone in a realm. (This is his own damage; nobody else's hits are changed or healed back.)
    fn on_attack(&mut self, sim: &mut StableSim, _: usize, id: usize, target: usize, damage: &mut usize) {
        let realms = crate::realm::realms(sim);
        if !realms.is_empty() && !crate::realm::can_interact(&realms, id, target) {
            *damage = 0;
        }
    }
    fn on_damaged(
        &mut self,
        sim: &mut StableSim,
        _: usize,
        id: usize,
        attacker: usize,
        damage: usize,
    ) {
        let Some(e) = sim.get_entity(id) else { return };
        let now = (e.hp().0, e.shield());
        let champion = sim
            .get_entity(attacker)
            .is_some_and(|a| a.is_champion() && a.team() != e.team());
        self.serial += 1;
        self.receipts.push(Receipt {
            serial: self.serial,
            source: attacker,
            champion,
            positive_damage: damage > 0
                || now.0 < self.last_observed.0
                || now.1 < self.last_observed.1,
        });
        self.last_observed = now;
    }
    fn on_update(&mut self, sim: &mut StableSim, _: u64, _: usize, id: usize) {
        let f = self.frame(sim, id);
        let e = self.core.step(&f);
        self.effects(sim, id, e);
        let marker = (self.core.stamina / 10000).clamp(0, 10);
        if self.stamina_marker != Some(marker) {
            clear(sim, id, "ct_stamina");
            sim.add_buff(id, &BuffV1::named(&format!("ct_stamina{marker}")));
            self.stamina_marker = Some(marker);
        }
        clear(sim, id, "ct_hit:");
        if self.core.busy() {
            let remaining = self
                .core
                .cast
                .as_ref()
                .unwrap()
                .until
                .saturating_sub(sim.tick() as u64)
                .min(2);
            sim.apply_cc(id, &CcV1::of_kind(CcKindV1::BlockAttack, remaining));
            if self.core.cast.as_ref().is_some_and(|c| {
                matches!(
                    c.technique,
                    Technique::Combination | Technique::BodyShot | Technique::Uppercut
                )
            }) {
                sim.apply_cc(id, &CcV1::of_kind(CcKindV1::Bind, remaining));
                if sim.tick() % 2 == 0 {
                    let cast = self.core.cast.as_ref().unwrap();
                    if let Some(e) = sim.get_entity(id) {
                        let p = e.pos();
                        let start = Point {
                            x: p.0 as i64,
                            y: p.1 as i64,
                        };
                        let end = start.toward(
                            cast.aim,
                            if cast.technique == Technique::Combination {
                                23000
                            } else {
                                20000
                            },
                        );
                        let angle = ((end.y - start.y) as f64).atan2((end.x - start.x) as f64);
                        let direction =
                            ((angle * 4. / std::f64::consts::PI).round() as i64).rem_euclid(8);
                        crate::fx_point(
                            sim,
                            &format!("ct_warning{direction}"),
                            id,
                            (start.x + end.x) / 2,
                            (start.y + end.y) / 2,
                            remaining,
                        );
                    }
                }
            }
        }
        if let Some(u) = sim.get_entity(id) {
            self.last_observed = (u.hp().0, u.shield());
        }
    }
}
pub fn danger_zones(
    all: &[crate::Champ],
    me: &crate::Champ,
    tick: usize,
) -> Vec<(i64, i64, i64, usize, usize)> {
    let mut zones = vec![];
    for c in all.iter().filter(|c| c.team != me.team) {
        for b in &c.buffs {
            if let Some(raw) = b.name().strip_prefix("ctw:") {
                let v: Vec<_> = raw
                    .split(':')
                    .filter_map(|s| s.parse::<i64>().ok())
                    .collect();
                if v.len() == 4 && tick < v[3] as usize {
                    zones.push((v[0], v[1], v[2], v[3] as usize, 0));
                }
            }
        }
    }
    zones
}
