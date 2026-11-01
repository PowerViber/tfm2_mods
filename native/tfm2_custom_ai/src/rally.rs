//! Thin Rally adapter. Receipt-dependent rewards fail closed until host calibration is recorded.
use crate::rally_core::{self as core, *};
use mod_api_stable::{
    AttackTypeV1, BuffV1, CcKindV1, CcV1, InputTargetKindV1, InputTargetV1, SkillSlotV1,
    StableEffectType, StableEntityView, StablePassive, StableSim, StatV1,
};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Clone, Default)]
pub struct Rally {
    pub core: Core,
    profile: Option<crate::athlete_ai_core::Profile>,
    history: BTreeMap<usize, (u64, Point)>,
    concealed_since: BTreeMap<usize, u64>,
    hits: Vec<Hit>,
    last_diagnostic: Option<String>,
    hud: Vec<String>,
    facing: crate::facing::Facing,
    last_pos: Option<Point>,
    attack_until: u64,
}
fn has(sim: &StableSim, id: usize, name: &str) -> bool {
    sim.get_entity(id).is_some_and(|u| {
        (0..u.buff_count())
            .filter_map(|i| u.buff_at(i))
            .any(|b| b.name() == name)
    })
}
fn marker(sim: &mut StableSim, id: usize, name: &str, on: bool) {
    let present = has(sim, id, name);
    if on && !present {
        sim.add_buff(id, &BuffV1::named(name))
    } else if !on && present {
        sim.entity_remove_buff(id, name);
    }
}
pub struct BasicImpact;
impl StableEffectType for BasicImpact {
    fn apply(&self, sim: &mut StableSim, _: u64, caster: usize, input: InputTargetV1) {
        if input.kind != InputTargetKindV1::Target.code() || has(sim, caster, "rly_basic_lock") {
            return;
        }
        let realms = crate::realm::realms(sim);
        if !crate::realm::can_interact(&realms, caster, input.target_id) {
            return;
        }
        let name = format!("rly_hit:{}:{}", sim.tick() + 1, input.target_id);
        if !has(sim, caster, &name) {
            sim.add_buff(caster, &crate::timed(&name, 2));
        }
    }
    fn expected_damage(&self, s: &StatV1) -> (usize, usize) {
        (s.attack, 0)
    }
    fn may_adjust_range(&self) -> bool {
        true
    }
    fn range_adjust(&self, caster: &StableEntityView, _: &StableEntityView) -> u64 {
        if caster.has_buff("rly_ball_held") {
            (core::data::VOLLEY_RANGE - core::data::PALM_RANGE) as u64
        } else {
            0
        }
    }
}
fn cc(sim: &StableSim, id: usize, kinds: &[CcKindV1]) -> bool {
    sim.get_entity(id).is_some_and(|u| {
        (0..u.cc_count()).filter_map(|i| u.cc_at(i)).any(|c| {
            !["rly_motion", "rly_attack_lock"].contains(&c.name())
                && kinds.iter().any(|k| k.code() == c.kind)
        })
    })
}
impl Rally {
    fn frame(&mut self, sim: &StableSim, owner: usize) -> Frame {
        let tick = sim.tick() as u64;
        let team = sim.get_entity(owner).map_or(usize::MAX, |u| u.team());
        let profile = crate::athlete_ai::pinned_owner(sim, owner, &mut self.profile);
        let mut values = [50; 13];
        values[..9].copy_from_slice(&profile.values);
        values[9..].copy_from_slice(&profile.sense.0);
        let farming = crate::farming::Context::new(sim, owner);
        let realms = crate::realm::realms(sim);
        let mut history = BTreeMap::new();
        let mut units = vec![];
        let mut basics = vec![];
        let mut blocks = crate::walls::boxing_blocks()
            .into_iter()
            .map(|b| Rect {
                min: Point {
                    x: b.min.x,
                    y: b.min.y,
                },
                max: Point {
                    x: b.max.x,
                    y: b.max.y,
                },
            })
            .collect::<Vec<_>>();
        // champions_shared is the shared source of champion buff snapshots; other entities are read once.
        let champions = crate::champions_shared(sim);
        for i in 0..sim.entity_count() {
            let Some(u) = sim.entity_at(i) else { continue };
            if !u.is_alive() && u.id() != owner {
                continue;
            }
            if !crate::realm::can_interact(&realms, owner, u.id()) {
                continue;
            }
            if u.team() == team && !u.is_champion() {
                continue;
            }
            let id = u.id();
            let p = u.pos();
            let pos = Point {
                x: p.0 as i64,
                y: p.1 as i64,
            };
            let visible = u.team() == team || sim.is_visible(team, id);
            if u.is_champion() && u.team() != team {
                if visible {
                    self.concealed_since.remove(&id);
                } else {
                    self.concealed_since.entry(id).or_insert(tick);
                }
            }
            let stat = u.stat();
            let buffs = if let Some(c) = champions.iter().find(|c| c.id == id) {
                c.buffs.clone()
            } else {
                (0..u.buff_count()).filter_map(|i| u.buff_at(i)).collect()
            };
            let vel = if visible {
                self.history
                    .get(&id)
                    .filter(|(t, _)| tick > *t && tick - *t < 120)
                    .map(|(t, p)| Point {
                        x: (pos.x - p.x) / (tick - *t) as i64,
                        y: (pos.y - p.y) / (tick - *t) as i64,
                    })
                    .unwrap_or_default()
            } else {
                Point::default()
            };
            if visible {
                history.insert(id, (tick, pos));
            }
            let kind = u.kind();
            units.push(Unit {
                id,
                team: u.team(),
                pos,
                velocity: vel,
                hp: u.hp().0 as i64,
                max_hp: u.hp().1.max(1) as i64,
                attack: stat.attack as i64,
                radius: u.radius().max(1000) as i64,
                champion: u.is_champion(),
                damageable: u.is_champion()
                    || u.is_minion()
                    || matches!(
                        kind,
                        Some(
                            mod_api_stable::EntityKindV1::Summon
                                | mod_api_stable::EntityKindV1::Jungle
                                | mod_api_stable::EntityKindV1::Epic
                                | mod_api_stable::EntityKindV1::Serpen
                        )
                    ),
                visible,
                targetable: u.is_targetable(),
                farmable: u.team() != team
                    && visible
                    && u.is_targetable()
                    && farming.eligible_known(
                        id,
                        u.team(),
                        kind,
                        p,
                        kind.is_none() && u.is_minion(),
                        &buffs,
                    ),
                controlled: cc(
                    sim,
                    id,
                    &[CcKindV1::Stun, CcKindV1::Airborne, CcKindV1::BlockSkill],
                ),
                cc_immune: buffs.iter().any(|b| b.cc_immune),
                control_eligible: u.is_champion()
                    || u.is_minion()
                    || matches!(kind, Some(mod_api_stable::EntityKindV1::Summon)),
            });
            for b in &buffs {
                if id == owner {
                    if let Some(raw) = b.name().strip_prefix("rly_hit:") {
                        let v = raw
                            .split(':')
                            .filter_map(|x| x.parse::<u64>().ok())
                            .collect::<Vec<_>>();
                        if v.len() == 2 {
                            basics.push(Basic {
                                serial: v[0],
                                target: v[1] as usize,
                            })
                        }
                    }
                }
            }
        }
        self.history = history;
        self.concealed_since
            .retain(|id, _| units.iter().any(|u| u.id == *id && u.hp > 0));
        // Only time without allied sight is used. Never consult a concealed champion's
        // current position, HP, items or cooldowns to score uncertainty.
        let uncertainty = self
            .concealed_since
            .values()
            .map(|start| (20 + tick.saturating_sub(*start) / 60).min(60) as i64)
            .max()
            .unwrap_or(0);
        // Player-built wall segments are sampled at <=2k, not expanded into 32k terrain cells.
        for wall in crate::steve::walls_up(&champions, sim.tick()) {
            let a = Point {
                x: wall.ax,
                y: wall.ay,
            };
            let end = wall.built_end(sim.tick());
            let z = Point { x: end.0, y: end.1 };
            let n = (a.distance(z) / 2000 + 1).min(1000);
            for i in 0..=n {
                let p = Point {
                    x: a.x + (z.x - a.x) * i / n,
                    y: a.y + (z.y - a.y) * i / n,
                };
                blocks.push(Rect {
                    min: Point {
                        x: (p.x - 3500).max(0),
                        y: (p.y - 3500).max(0),
                    },
                    max: Point {
                        x: (p.x + 3500).min(core::data::MAP),
                        y: (p.y + 3500).min(core::data::MAP),
                    },
                })
            }
        }
        blocks.truncate(2048);
        let me = units.iter().find(|u| u.id == owner);
        let danger = me.map_or(0, |m| {
            units
                .iter()
                .filter(|u| {
                    u.champion
                        && u.visible
                        && u.team != team
                        && u.hp > 0
                        && u.pos.distance(m.pos) < u.radius + m.radius + 35000
                })
                .map(|u| u.attack / 3)
                .sum()
        });
        let skill2 = sim
            .get_entity(owner)
            .is_some_and(|e| e.cooldown(SkillSlotV1::Skill2).is_some_and(|c| c.unlocked));
        let ultimate = has(sim, owner, "ult_learned")
            || sim
                .get_entity(owner)
                .is_some_and(|e| e.cooldown(SkillSlotV1::Ult).is_some_and(|c| c.unlocked));
        Frame {
            tick,
            owner,
            seed: sim.seed(),
            units,
            blocks,
            smoke_disks: crate::valorant::smokes_up(&champions, sim.tick())
                .into_iter()
                .filter(|s| crate::realm::can_interact(&realms, owner, s.owner))
                .map(|s| Disk {
                    pos: Point { x: s.x, y: s.y },
                    radius: crate::valorant::SMOKE_R,
                })
                .collect(),
            profile: values,
            manual: false,
            move_step: sim
                .get_entity(owner)
                .map_or(1050, |e| e.stat().move_speed as i64)
                .clamp(1, 10000),
            skill2,
            skill1: sim
                .get_entity(owner)
                .is_some_and(|e| e.cooldown(SkillSlotV1::Skill).is_some_and(|c| c.unlocked)),
            ultimate,
            interrupted: cc(
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
            rooted: cc(sim, owner, &[CcKindV1::Bind, CcKindV1::BlockMoveSkill]),
            danger,
            uncertainty,
            hits: std::mem::take(&mut self.hits),
            basics,
            incoming_verified: false,
            support_verified: false,
            ..Frame::default()
        }
    }
    fn apply(&mut self, sim: &mut StableSim, owner: usize, effects: Vec<Effect>) {
        let mut aggregate = BTreeSet::new();
        let mut parried = BTreeSet::new();
        let mut hits = BTreeMap::<u64, (usize, bool, bool)>::new();
        for e in effects {
            match e {
                Effect::Damage {
                    serial,
                    target,
                    amount,
                    basic,
                    ..
                } => {
                    let realms = crate::realm::realms(sim);
                    if !crate::realm::can_interact(&realms, owner, target) {
                        continue;
                    }
                    let snapshot = crate::champions_shared(sim);
                    if let Some(t) = snapshot.iter().find(|u| u.id == target) {
                        if crate::batch2::try_parry(sim, t, owner, amount.max(0) as usize) {
                            parried.insert(target);
                            continue;
                        }
                    }
                    let before = sim
                        .get_entity(target)
                        .map(|u| (u.hp().0, u.shield(), u.is_champion()));
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
                        let delivered = after.hp().0 < hp || after.shield() < shield;
                        let row = hits.entry(serial).or_insert((target, false, false));
                        row.1 |= delivered;
                        row.2 |= delivered && champion;
                    }
                }
                Effect::Move { pos } => {
                    let from = sim.get_entity(owner).map_or((pos.x, pos.y), |u| {
                        let p = u.pos();
                        (p.0 as i64, p.1 as i64)
                    });
                    let p = crate::place::point(&[], sim.tick(), from, (pos.x, pos.y));
                    sim.entity_set_pos(owner, p.0.max(0) as u64, p.1.max(0) as u64);
                    let mut bind = CcV1::of_kind(CcKindV1::Bind, 1);
                    bind.set_name("rly_motion");
                    sim.apply_cc(owner, &bind);
                }
                Effect::Shield { amount, ticks } => {
                    sim.entity_add_shield(owner, amount.max(0) as usize, ticks as usize);
                }
                Effect::KnockUp { target, ticks } => {
                    let realms = crate::realm::realms(sim);
                    if !parried.contains(&target)
                        && crate::realm::can_interact(&realms, owner, target)
                        && sim.get_entity(target).is_some_and(|u| {
                            u.is_alive()
                                && !(0..u.buff_count())
                                    .filter_map(|i| u.buff_at(i))
                                    .any(|b| b.cc_immune)
                        })
                    {
                        let mut c = CcV1::of_kind(CcKindV1::Airborne, ticks);
                        c.set_name(&format!("rly_crater_hit:{owner}"));
                        sim.apply_cc(target, &c);
                    }
                }
                Effect::Slow {
                    target,
                    percent,
                    ticks,
                } => {
                    let realms = crate::realm::realms(sim);
                    if crate::realm::can_interact(&realms, owner, target)
                        && sim.get_entity(target).is_some_and(|u| {
                            u.is_alive()
                                && !(0..u.buff_count())
                                    .filter_map(|i| u.buff_at(i))
                                    .any(|b| b.cc_immune)
                        })
                    {
                        // One matching slow across Rally owners, renewed only while inside a live crater.
                        sim.entity_remove_buff(target, "rly_crater_slow");
                        let mut b = crate::timed("rly_crater_slow", ticks as usize);
                        b.move_speed_mult = -(percent as i32);
                        sim.add_buff(target, &b);
                    }
                }
                Effect::Buff { buff, remove } => {
                    let prefix = format!("rlym:{owner}:{}:", buff.channel);
                    clear_prefix(sim, buff.target, &prefix);
                    if !remove {
                        let name =
                            format!("{prefix}{}:{}:{}", buff.attack, buff.speed, buff.reduce);
                        let b = if buff.until == 0 {
                            BuffV1::named(&name)
                        } else {
                            crate::timed(
                                &name,
                                buff.until.saturating_sub(sim.tick() as u64).max(1) as usize,
                            )
                        };
                        sim.add_buff(buff.target, &b);
                    }
                    aggregate.insert(buff.target);
                }
                Effect::Warning {
                    serial,
                    pos,
                    radius,
                    until,
                } => {
                    // Freeze legitimate observers when the warning becomes public.
                    // Existing danger readers have no Sim, so encode the audience here.
                    let audience = crate::champions_shared(sim)
                        .iter()
                        .filter(|c| c.team < 63 && sim.is_visible(c.team, owner))
                        .fold(0u64, |mask, c| mask | (1u64 << c.team));
                    let name = format!(
                        "rlyw:{serial}:{}:{}:{radius}:{until}:{audience}",
                        pos.x, pos.y
                    );
                    sim.add_buff(
                        owner,
                        &crate::timed(
                            &name,
                            until.saturating_sub(sim.tick() as u64).max(1) as usize,
                        ),
                    );
                    crate::steve::note_map_buff(sim.seed());
                }
                Effect::ClearWarning { serial } => {
                    clear_prefix(sim, owner, &format!("rlyw:{serial}:"));
                    crate::steve::note_map_buff(sim.seed());
                }
                Effect::Visual { name, pos, height } => {
                    if ["spike_release", "black_release", "white_release", "contact"]
                        .contains(&name.as_str())
                    {
                        let label = height_label(height);
                        crate::fx_point(
                            sim,
                            &format!("rly_contact_height_{label}"),
                            owner,
                            pos.x,
                            pos.y
                                - (height - 2000).max(0) * core::data::HEIGHT_WORLD_PER_M / 1000
                                - 110000,
                            1,
                        );
                    }

                    let anchor =
                        if ["impact", "black_impact", "white_impact"].contains(&name.as_str()) {
                            core::data::IMPACT_ANCHOR_PIXELS * core::data::WORLD_UNITS_PER_PIXEL
                        } else {
                            0
                        };
                    let name = format!("rly_{name}");
                    crate::fx_point(
                        sim,
                        &name,
                        owner,
                        pos.x,
                        pos.y + anchor
                            - (height - 2000).max(0) * core::data::HEIGHT_WORLD_PER_M / 1000,
                        1,
                    );
                }
                Effect::RevealSector { .. } => {
                    // No supported team-sector reveal setter exists in this SDK.
                    // Never clear invisibility or feed concealed entities into the planner.
                    crate::mod_log(sim, "rally_log.txt", "rally-sector-vision",
                        "Serve sector vision unavailable: native SDK has no verified reveal operation");
                }
                Effect::Transform | Effect::WhiteTransform => {} // Strength belongs to the active role in the shared core.
                Effect::Reject { reason } => {
                    if self.last_diagnostic.as_ref() != Some(&reason) {
                        crate::mod_log(
                            sim,
                            "rally_log.txt",
                            "rally",
                            &format!("Rally #{owner}: {reason}"),
                        );
                        self.last_diagnostic = Some(reason);
                    }
                }
            }
        }
        for target in aggregate {
            aggregate_buffs(sim, target)
        }
        for (serial, (target, delivered, champion)) in hits {
            self.hits.push(Hit {
                serial,
                target,
                delivered,
                champion,
            });
        }
    }
    fn present(&mut self, sim: &mut StableSim, id: usize) {
        let alive = sim.get_entity(id).is_some_and(|u| u.is_alive());
        let all = crate::champions_shared(sim);
        if let Some(me) = all.iter().find(|u| u.id == id) {
            let enemy = all
                .iter()
                .filter(|u| u.team != me.team && u.hp > 0 && sim.is_visible(me.team, u.id))
                .min_by_key(|u| crate::d2(u.x, u.y, me.x, me.y))
                .map(|u| (u.x, u.y));
            self.facing.update(sim.tick(), (me.x, me.y), enemy);
        }
        let mut hud = vec![];
        if alive {
            hud.push(format!("rly_flow{}", self.core.flow / 10));
            hud.push(format!("rly_charges{}", self.core.charges));
            hud.push(format!(
                "rly_role{}",
                match self.core.role {
                    Role::Spiker => 0,
                    Role::Setter => 1,
                    Role::Receiver => 2,
                }
            ));
            if let Some(p) = sim.get_entity(id).map(|e| e.pos()) {
                let pos = Point {
                    x: p.0 as i64,
                    y: p.1 as i64,
                };
                let colour = display_colour(&self.core, sim.tick() as u64);
                let form = if self.core.awakened {
                    "unmasked"
                } else {
                    "masked"
                };
                let face = if self.facing.left() { "l" } else { "r" };
                if matches!(self.core.phase, Phase::Air { .. }) {
                    // Explicitly approved flight-only hiding. One tick, never idle/basic.
                    let mut hidden = CcV1::of_kind(CcKindV1::Animation, 1);
                    hidden.set_name("air_hidden");
                    sim.apply_cc(id, &hidden);
                } else if self.core.transformed || self.core.awakened {
                    let tag = if self
                        .core
                        .receive_start
                        .is_some_and(|s| (sim.tick() as u64) < s + 18)
                    {
                        "receive"
                    } else if (sim.tick() as u64) < self.attack_until {
                        "attack"
                    } else {
                        match self.core.phase {
                            Phase::Prepare { .. } => "set",
                            Phase::Approach { .. } => "run",
                            _ => {
                                if self.last_pos.is_some_and(|old| old != pos) {
                                    "run"
                                } else {
                                    "idle"
                                }
                            }
                        }
                    };
                    hud.push(format!("rly_body_{colour}_{form}_{tag}_{face}"));
                }
                self.last_pos = Some(pos);
            }
        }
        for old in self.hud.iter().filter(|x| !hud.contains(x)) {
            sim.entity_remove_buff(id, old);
        }
        for name in hud.iter().filter(|x| !self.hud.contains(x)) {
            sim.add_buff(id, &BuffV1::named(name));
        }
        self.hud = hud;
        if !alive {
            return;
        }
        // Bounded two-tick flight layers; contact/impact add one baked stroke burst, not one effect per bolt.
        // Two-tick phases end without an animation renewal tail after cleanup.
        if sim.tick() % 2 == 0 {
            let phase = (sim.tick() / 2) % 8;
            if matches!(self.core.phase, Phase::Air { .. }) {
                if let Some((x, y)) = sim.get_entity(id).map(|e| e.pos()) {
                    let colour = display_colour(&self.core, sim.tick() as u64);
                    let form = if self.core.awakened {
                        "unmasked"
                    } else {
                        "masked"
                    };
                    let face = if self.facing.left() { "l" } else { "r" };
                    let h = self.core.height(sim.tick() as u64);
                    crate::fx_point(
                        sim,
                        &format!("rly_air_{colour}_{form}_{face}_p{phase}"),
                        id,
                        x as i64,
                        y as i64 - (h - 2000) * core::data::HEIGHT_WORLD_PER_M / 1000,
                        1,
                    );
                    if matches!(
                        self.core.phase,
                        Phase::Air {
                            attempted: false,
                            ..
                        }
                    ) {
                        let label = height_label(h);
                        crate::fx_point(
                            sim,
                            &format!("rly_height_{label}"),
                            id,
                            x as i64,
                            y as i64 - (h - 2000) * core::data::HEIGHT_WORLD_PER_M / 1000 - 110000,
                            1,
                        );
                    }
                }
            }
            if let Some((p, _)) = self.core.helper {
                let colour = display_colour(&self.core, sim.tick() as u64);
                let form = if self.core.awakened {
                    "unmasked"
                } else {
                    "masked"
                };
                let face = if self.facing.left() { "l" } else { "r" };
                crate::fx_point(
                    sim,
                    &format!("rly_helper_{colour}_{form}_{face}_p{phase}"),
                    id,
                    p.x,
                    p.y,
                    1,
                );
            }

            let pos = match self.core.ball {
                Ball::Toss {
                    start,
                    origin,
                    to,
                    peak,
                } => Some(toss(start, origin, to, peak, sim.tick() as u64)),
                Ball::Flying {
                    origin,
                    pos,
                    to,
                    height,
                    ..
                } => Some((pos, flight_height(origin, pos, to, height))),
                Ball::Returning { pos } | Ball::Grounded { pos } => Some((pos, 2000)),
                _ => None,
            };
            if let Some((p, h)) = pos {
                let name = match self.core.ball {
                    Ball::Flying { white: true, .. } => "white",
                    Ball::Flying { black: true, .. } => "black",
                    Ball::Flying { thunder: true, .. } => "thunder",
                    _ => "ball",
                };
                crate::fx_point(
                    sim,
                    &format!("rly_{name}_p{phase}"),
                    id,
                    p.x,
                    p.y - (h - 2000).max(0) * core::data::HEIGHT_WORLD_PER_M / 1000,
                    1,
                );
                if let Ball::Flying {
                    to,
                    thunder: true,
                    black,
                    white,
                    ..
                } = self.core.ball
                {
                    let dx = to.x - p.x;
                    let dy = to.y - p.y + (h - 2000) * core::data::HEIGHT_WORLD_PER_M / 1000;
                    let angle = (dy as f64).atan2(dx as f64);
                    let heading =
                        ((angle / std::f64::consts::FRAC_PI_4).round() as i64).rem_euclid(8);
                    let behind = Point {
                        x: p.x - (angle.cos() * 80640.0) as i64,
                        y: p.y
                            - (h - 2000) * core::data::HEIGHT_WORLD_PER_M / 1000
                            - (angle.sin() * 80640.0) as i64,
                    };
                    crate::fx_point(
                        sim,
                        &format!(
                            "rly_trail_{}_a{heading}_p{phase}",
                            if white {
                                "white"
                            } else if black {
                                "black"
                            } else {
                                "thunder"
                            }
                        ),
                        id,
                        behind.x,
                        behind.y,
                        1,
                    );
                }
                if let Phase::Air { attempted, .. } = self.core.phase {
                    let tier = if attempted {
                        self.core.last_spike_tier
                    } else {
                        attack_tier(self.core.height(sim.tick() as u64))
                    };
                    if tier >= AttackTier::Thunder {
                        if let Some((x, y)) = sim.get_entity(id).map(|e| e.pos()) {
                            crate::fx_point(
                                sim,
                                &format!(
                                    "rly_body_{}_p{phase}",
                                    if tier == AttackTier::WhiteStorm {
                                        "white"
                                    } else if tier == AttackTier::BlackThunder {
                                        "black"
                                    } else {
                                        "thunder"
                                    }
                                ),
                                id,
                                x as i64,
                                y as i64
                                    - (self.core.height(sim.tick() as u64) - 2000)
                                        * core::data::HEIGHT_WORLD_PER_M
                                        / 1000,
                                1,
                            );
                        }
                    }
                }
            }
            for c in &self.core.craters {
                crate::fx_point(
                    sim,
                    &format!(
                        "rly_{}_p{phase}",
                        if c.radius > core::data::BLACK_RADIUS {
                            "crater_serve"
                        } else {
                            "crater"
                        }
                    ),
                    id,
                    c.pos.x,
                    c.pos.y,
                    1,
                );
            }
            if let Some(v) = &self.core.serve {
                let angle = ((v.aim.y - v.origin.y) as f64).atan2((v.aim.x - v.origin.x) as f64);
                for side in [-1.0, 1.0] {
                    let a = angle + side * std::f64::consts::PI / 6.0;
                    for n in 1..=core::data::SERVE_EDGE_MARKERS {
                        let d = core::data::SERVE_RANGE * n / core::data::SERVE_EDGE_MARKERS;
                        let x = v.origin.x + (a.cos() * d as f64) as i64;
                        let y = v.origin.y + (a.sin() * d as f64) as i64;
                        if (0..=core::data::MAP).contains(&x) && (0..=core::data::MAP).contains(&y)
                        {
                            crate::fx_point(sim, &format!("rly_serve_edge_p{phase}"), id, x, y, 1);
                        }
                    }
                }
            }
            if let Some(w) = sim.get_entity(id).and_then(|e| {
                (0..e.buff_count())
                    .filter_map(|i| e.buff_at(i))
                    .find(|b| b.name().starts_with("rlyw:"))
            }) {
                let v = w
                    .name()
                    .split(':')
                    .skip(1)
                    .filter_map(|v| v.parse::<i64>().ok())
                    .collect::<Vec<_>>();
                if v.len() == 6 {
                    crate::fx_point(
                        sim,
                        &format!(
                            "rly_{}_p{phase}",
                            if v[3] > core::data::BLACK_RADIUS {
                                "warning_serve"
                            } else if v[3] > core::data::SPIKE_RADIUS {
                                "warning_black"
                            } else {
                                "warning"
                            }
                        ),
                        id,
                        v[1],
                        v[2],
                        1,
                    );
                }
            }
        }
    }
}
fn display_colour(c: &Core, _t: u64) -> &'static str {
    if c.white_storm {
        "white"
    } else if c.transformed {
        "thunder"
    } else {
        "yellow"
    }
}
fn clear_prefix(sim: &mut StableSim, id: usize, prefix: &str) {
    let names = sim
        .get_entity(id)
        .map(|u| {
            (0..u.buff_count())
                .filter_map(|i| u.buff_at(i))
                .filter(|b| b.name().starts_with(prefix))
                .map(|b| b.name().to_string())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    for n in names {
        sim.entity_remove_buff(id, &n);
    }
}
fn aggregate_buffs(sim: &mut StableSim, target: usize) {
    let values = sim
        .get_entity(target)
        .map(|u| {
            (0..u.buff_count())
                .filter_map(|i| u.buff_at(i))
                .filter_map(|b| {
                    let mut p = b.name().strip_prefix("rlym:")?.split(':');
                    let _owner = p.next()?;
                    let channel = p.next()?.to_string();
                    let a = p.next()?.parse::<i32>().ok()?;
                    let m = p.next()?.parse::<i32>().ok()?;
                    let r = p.next()?.parse::<usize>().ok()?;
                    Some((channel, a, m, r))
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    for channel in ["aura", "support"] {
        sim.entity_remove_buff(target, &format!("rly_{channel}"));
        let a = values
            .iter()
            .filter(|v| v.0 == channel)
            .map(|v| v.1)
            .max()
            .unwrap_or(0);
        let m = values
            .iter()
            .filter(|v| v.0 == channel)
            .map(|v| v.2)
            .max()
            .unwrap_or(0);
        if a != 0 || m != 0 {
            let mut b = BuffV1::named(&format!("rly_{channel}"));
            b.attack_mult = a;
            b.magic_power_mult = a;
            b.move_speed_mult = m;
            sim.add_buff(target, &b);
        }
    }
    marker(
        sim,
        target,
        "rly_support_orb",
        values.iter().any(|v| v.0 == "support_ball"),
    );
    sim.entity_remove_buff(target, "rly_reduction");
    let r = values.iter().map(|v| v.3).max().unwrap_or(0);
    if r > 0 {
        let mut b = BuffV1::named("rly_reduction");
        b.damaged_reduce = r;
        sim.add_buff(target, &b);
    }
}
impl StablePassive for Rally {
    fn clone_box(&self) -> Box<dyn StablePassive> {
        Box::new(self.clone())
    }
    fn on_spawn(&mut self, sim: &mut StableSim, _: usize, id: usize) {
        self.core.alive = false;
        self.core.thunder_earned = false;
        self.core.transformed = false;
        self.core.white_storm = false;
        marker(sim, id, "rly_thunder_life", false);
        marker(sim, id, "rly_ball_held", true);
    }
    fn on_dead(&mut self, sim: &mut StableSim, id: usize) {
        let effects = self.core.died();
        self.apply(sim, id, effects);
        for name in [
            "rly_thunder_life",
            "rly_ball_held",
            "rly_flash_lock",
            "rly_basic_lock",
        ] {
            marker(sim, id, name, false);
        }
        clear_prefix(sim, id, "rlyw:");
        for name in self.hud.drain(..) {
            sim.entity_remove_buff(id, &name);
        }
        self.hits.clear();
        self.last_pos = None;
    }
    fn on_attack(
        &mut self,
        sim: &mut StableSim,
        _: usize,
        id: usize,
        target: usize,
        damage: &mut usize,
    ) {
        let realms = crate::realm::realms(sim);
        if !crate::realm::can_interact(&realms, id, target) {
            *damage = 0
        }
    }
    fn on_damaged(
        &mut self,
        sim: &mut StableSim,
        _: usize,
        id: usize,
        _attacker: usize,
        _damage: usize,
    ) {
        if self.last_diagnostic.is_none() {
            crate::mod_log(sim,"rally_log.txt","rally",&format!("Rally #{id}: incoming callback observed; caught power remains unverified (no raw-to-delivered assumption)"));
            self.last_diagnostic = Some("receipt-unverified".into());
        }
    }
    fn on_update(&mut self, sim: &mut StableSim, _: u64, _: usize, id: usize) {
        let f = self.frame(sim, id);
        if !f.basics.is_empty() {
            self.attack_until = sim.tick() as u64 + 18;
        }
        let effects = self.core.step(&f);
        self.apply(sim, id, effects);
        if !self.core.transformed {
            marker(sim, id, "rly_thunder_life", false);
        }
        marker(
            sim,
            id,
            "rly_ball_held",
            self.core.alive && self.core.ball == Ball::Held,
        );
        marker(
            sim,
            id,
            "rly_flash_lock",
            self.core.alive && self.core.flash_blocked(),
        );
        let lock = self.core.alive && (self.core.busy());
        marker(sim, id, "rly_basic_lock", lock);
        if lock {
            let mut c = CcV1::of_kind(CcKindV1::BlockAttack, 1);
            c.set_name("rly_attack_lock");
            sim.apply_cc(id, &c);
        }
        clear_prefix(sim, id, "rly_hit:");
        self.present(sim, id);
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
            if let Some(raw) = b.name().strip_prefix("rlyw:") {
                let v = raw
                    .split(':')
                    .filter_map(|x| x.parse::<i64>().ok())
                    .collect::<Vec<_>>();
                if v.len() == 6
                    && me.team < 63
                    && ((v[5] as u64) & (1u64 << me.team)) != 0
                    && tick < (v[4] as usize)
                {
                    zones.push((v[1], v[2], v[3], v[4] as usize, 0));
                }
            }
        }
    }
    zones
}
