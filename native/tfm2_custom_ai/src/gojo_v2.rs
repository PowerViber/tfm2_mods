//! Native Gojo adapter. A data-native basic impact emits an owner-local inbox event;
//! only this passive turns it into one attributed physical+magic BaseAttack.
use crate::gojo_v2_core::*;
use mod_api_stable::{
    AttackTypeV1, CcKindV1, CcV1, InputTargetV1, StableEffectType, StablePassive, StableSim, StatV1,
};
#[derive(Clone, Default)]
pub struct Gojo {
    pub core: Core,
    last_tick: Option<usize>,
    owner: usize,
    last_champion_hit: bool,
    hud: [Option<usize>; 2],
    warnings: Vec<String>,
    published_shield: bool,
    status_expiry: [usize; 2],
    visual_origin: [usize; 3],
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
        let name = format!("gj2_hit:{}:{}", sim.tick(), input.target_id);
        if !(0..e.buff_count())
            .filter_map(|i| e.buff_at(i))
            .any(|b| b.name() == name)
        {
            sim.add_buff(caster, &crate::timed(&name, 3));
        }
    }
    fn expected_damage(&self, s: &StatV1) -> (usize, usize) {
        (s.attack, s.magic_power * 35 / 100)
    }
}
fn has(sim: &StableSim, id: usize, name: &str) -> bool {
    sim.get_entity(id).is_some_and(|e| {
        (0..e.buff_count())
            .filter_map(|i| e.buff_at(i))
            .any(|b| b.name() == name)
    })
}
fn cleared(sim: &mut StableSim, id: usize, prefix: &str) {
    let names = sim
        .get_entity(id)
        .map(|e| {
            (0..e.buff_count())
                .filter_map(|i| e.buff_at(i))
                .map(|b| b.name().to_string())
                .filter(|n| n.starts_with(prefix))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    for n in names {
        sim.entity_remove_buff(id, &n);
    }
}
fn cc(sim: &StableSim, id: usize, k: CcKindV1) -> bool {
    sim.get_entity(id).is_some_and(|e| {
        (0..e.cc_count())
            .filter_map(|i| e.cc_at(i))
            .any(|c| c.kind == k.code())
    })
}
impl Gojo {
    fn snapshot(&self, sim: &StableSim, owner: usize) -> Vec<Fighter> {
        let team = sim.get_entity(owner).map_or(usize::MAX, |e| e.team());
        (0..sim.entity_count())
            .filter_map(|i| sim.entity_at(i))
            .filter(|e| e.is_alive() || e.id() == owner)
            .map(|e| {
                let p = e.pos();
                let hp = e.hp();
                let s = e.stat();
                let (immune, cc_immune) = (0..e.buff_count()).filter_map(|i| e.buff_at(i)).fold(
                    (false, false),
                    |(damage, cc), b| {
                        (
                            damage
                                || b.damaged_reduce >= 100
                                || (b.base_attack_damaged_reduce >= 100
                                    && b.skill_damaged_reduce >= 100),
                            cc || b.cc_immune,
                        )
                    },
                );
                Fighter {
                    id: e.id(),
                    team: e.team(),
                    x: p.0 as i64,
                    y: p.1 as i64,
                    hp: hp.0 as i64,
                    max_hp: hp.1 as i64,
                    ad: s.attack as i64,
                    ap: s.magic_power as i64,
                    level: e.level() as i64,
                    champion: e.is_champion(),
                    visible: e.team() == team || sim.is_visible(team, e.id()),
                    targetable: e.is_targetable(),
                    immune,
                    cc_immune,
                    immovable: matches!(
                        e.kind(),
                        Some(
                            mod_api_stable::EntityKindV1::Epic
                                | mod_api_stable::EntityKindV1::Serpen
                        )
                    ) || e.is_tower(),
                    cleansed: self
                        .core
                        .spirals
                        .iter()
                        .any(|m| m.target == e.id() && sim.tick() > m.begin)
                        && !cc(sim, e.id(), CcKindV1::Bind)
                        && !cc(sim, e.id(), CcKindV1::BlockMoveSkill),
                    farmable: crate::farming::eligible(sim, owner, e.id()),
                    tower: e.is_tower()
                        || matches!(e.kind(), Some(mod_api_stable::EntityKindV1::Nexus)),
                }
            })
            .collect()
    }
    fn finish_domain(&mut self, sim: &mut StableSim) {
        let prefix = format!("gj_guard{}", self.owner);
        let ids: Vec<usize> = (0..sim.entity_count())
            .filter_map(|i| sim.entity_at(i))
            .map(|e| e.id())
            .collect();
        for id in ids {
            sim.entity_remove_buff(id, &prefix);
        }
        cleared(sim, self.owner, "gj2_domain:");
    }
    fn execute(&mut self, sim: &mut StableSim, commands: Vec<Cmd>) {
        let owner = self.owner;
        let tick = sim.tick();
        for command in commands {
            match command {
                Cmd::Damage {
                    target,
                    ad,
                    ap,
                    basic,
                    ..
                } => {
                    sim.deal_damage(
                        owner,
                        target,
                        ad.max(0) as usize,
                        ap.max(0) as usize,
                        if basic {
                            AttackTypeV1::BaseAttack
                        } else {
                            AttackTypeV1::Skill
                        },
                    );
                }
                Cmd::SelfCost => {
                    if let Some(hp) = sim.get_entity(owner).map(|e| e.hp().0) {
                        if hp > 0 {
                            sim.entity_set_hp(owner, (hp - hp / 2).max(1));
                        }
                    }
                }
                Cmd::Move { target, x, y } => {
                    sim.entity_set_pos(
                        target,
                        x.clamp(0, 959999) as u64,
                        y.clamp(0, 959999) as u64,
                    );
                }
                Cmd::Heal(n) => sim.heal(owner, owner, n.max(0) as usize),
                Cmd::Infinity { active, shield } => self.infinity(sim, active, shield),
                Cmd::Seal { target, ticks } => {
                    if let Some(c) = crate::champions(sim).iter().find(|c| c.id == target) {
                        crate::strip_cc_immunity(sim, c);
                    }
                    for k in [
                        CcKindV1::Stun,
                        CcKindV1::Bind,
                        CcKindV1::BlockAttack,
                        CcKindV1::BlockSkill,
                        CcKindV1::BlockMoveSkill,
                    ] {
                        sim.apply_cc(target, &CcV1::of_kind(k, ticks as u64));
                    }
                }
                Cmd::Guard {
                    target,
                    ticks,
                    slow: _,
                } => {
                    let name = format!("gj_guard{owner}");
                    sim.entity_remove_buff(target, &name);
                    let mut b = crate::timed(&name, ticks);
                    b.base_attack_damaged_reduce = 100;
                    b.skill_damaged_reduce = 100;
                    b.cc_immune = true;
                    sim.add_buff(target, &b);
                }
                Cmd::Fx { kind, x, y, ticks } => {
                    // Current Domain/guest layers are renewed briefly below.
                    if kind == 10 || kind == 11 || kind == 15 {
                        continue;
                    }
                    let flight = match kind {
                        12 => Some((1, "red", 4)),
                        13 => Some((0, "blue", 4)),
                        7 if !self
                            .core
                            .pending
                            .as_ref()
                            .is_some_and(|c| matches!(c.action, Action::Purple(_))) =>
                        {
                            Some((2, "purple", 6))
                        }
                        _ => None,
                    };
                    if let Some((clock, name, count)) = flight {
                        let frame =
                            (tick.saturating_sub(self.visual_origin[clock]) * 5 / 18) % count;
                        crate::fx_point(
                            sim,
                            &format!("tfm2_custom_gojo_flight_{name}{frame}"),
                            owner,
                            x,
                            y,
                            ticks as u64,
                        );
                        if kind != 7 || tick >= self.core.amp_until {
                            continue;
                        }
                    }
                    let name = if kind == 14 {
                        format!(
                            "tfm2_custom_gojo_v2_orb{}",
                            self.core.orb.as_ref().map_or(1, |o| o
                                .end
                                .saturating_sub(tick)
                                .div_ceil(60)
                                .clamp(1, 6))
                        )
                    } else {
                        format!("tfm2_custom_gojo_new_{kind}")
                    };
                    crate::fx_point(sim, &name, owner, x, y, ticks as u64);
                }
                Cmd::Warning { x, y, r, ticks } => {
                    // Domain geometry already publishes its own owner/guest-aware warning.
                    if self.core.domain.as_ref().is_some_and(|d|d.start==tick && d.centre==(x,y) && d.radius==r && d.expand==ticks) {continue;}
                    let n = format!("gj2w:{x}:{y}:{r}:{}", tick + ticks);
                    sim.add_buff(owner, &crate::timed(&n, ticks + 1));
                    self.warnings.push(n);
                    crate::steve::note_map_buff(sim.seed());
                }
                Cmd::Lock(n) => {
                    if let Some(c) = &self.core.pending {
                        let k = c.action.index();
                        if k <= 4 {
                            self.visual_origin[if k < 2 {
                                0
                            } else if k < 4 {
                                1
                            } else {
                                2
                            }] = c.end;
                        }
                    }
                    let mut pose = CcV1::of_kind(CcKindV1::Animation, n as u64);
                    pose.set_name(
                        if self.core.domain.as_ref().is_some_and(|d| d.start == tick) {
                            "ult"
                        } else if self
                            .core
                            .pending
                            .as_ref()
                            .is_some_and(|c| matches!(c.action, Action::Red(_) | Action::MaxRed(_)))
                        {
                            "skill2"
                        } else {
                            "skill1"
                        },
                    );
                    sim.apply_cc(owner, &pose);
                    sim.apply_cc(owner, &CcV1::of_kind(CcKindV1::BlockAttack, n as u64));
                    // Our casting lock is represented by core state, never an enemy stun.
                    sim.apply_cc(owner, &CcV1::of_kind(CcKindV1::Bind, n as u64));
                }
                Cmd::Hold { target, ticks } => {
                    sim.apply_cc(target, &CcV1::of_kind(CcKindV1::Bind, ticks as u64));
                    sim.apply_cc(
                        target,
                        &CcV1::of_kind(CcKindV1::BlockMoveSkill, ticks as u64),
                    );
                }
                Cmd::DomainEnd => self.finish_domain(sim),
            }
        }
    }
    // SDK shields have no individual removal handle. Approved policy: retain the
    // existing layer while reductions/rebuilding pause. Never clear other shields.
    fn infinity(&mut self, sim: &mut StableSim, active: bool, shield: i64) {
        sim.entity_remove_buff(self.owner, "gojo_infinity");
        if shield <= 0 {
            self.published_shield = false;
        }
        if active {
            if !self.published_shield && shield > 0 {
                self.published_shield = sim.entity_add_shield(self.owner, shield as usize, 5184000);
            }
            if self.published_shield {
                let mut b = crate::timed("gojo_infinity", 5184000);
                b.skill_damaged_reduce = crate::gojo_v2_data::INFINITY_SKILL_REDUCE as usize;
                b.base_attack_damaged_reduce = crate::gojo_v2_data::INFINITY_BASIC_REDUCE as usize;
                sim.add_buff(self.owner, &b);
            }
        }
    }
    fn presentation(&mut self, sim: &mut StableSim) {
        let tick = sim.tick();
        let owner = self.owner;
        for (i, cell) in [
            (self.core.ce / 5000).clamp(0, 20) as usize,
            (self.core.strain / 12500).clamp(0, 20) as usize,
        ]
        .into_iter()
        .enumerate()
        {
            if self.hud[i] != Some(cell) {
                if let Some(n) = self.hud[i] {
                    sim.entity_remove_buff(
                        owner,
                        &format!("gj_{}{n}", if i == 0 { "ce" } else { "strain" }),
                    );
                }
                sim.add_buff(
                    owner,
                    &crate::timed(
                        &format!("gj_{}{cell}", if i == 0 { "ce" } else { "strain" }),
                        5184000,
                    ),
                );
                self.hud[i] = Some(cell);
            }
        }
        for (i, (name, end)) in [
            ("gj_amp", self.core.amp_until),
            ("gj_burnout", self.core.burnout),
        ]
        .into_iter()
        .enumerate()
        {
            if tick < end {
                if self.status_expiry[i] != end || !has(sim, owner, name) {
                    sim.entity_remove_buff(owner, name);
                    sim.add_buff(owner, &crate::timed(name, end - tick));
                    self.status_expiry[i] = end;
                }
            } else {
                sim.entity_remove_buff(owner, name);
                self.status_expiry[i] = 0;
            }
        }
        for name in ["gj_rework", "gj_v2"] {
            if !has(sim, owner, name) {
                sim.add_buff(owner, &crate::timed(name, 5184000));
            }
        }
        if self.core.eyes {
            if !has(sim, owner, "gj_six_eyes") {
                sim.add_buff(owner, &crate::timed("gj_six_eyes", 5184000));
            }
        } else {
            sim.entity_remove_buff(owner, "gj_six_eyes");
        }
        for (name, active) in [
            ("gj_eye_exhausted", !self.core.eyes),
            ("gj_inf_paused", !self.core.infinity && self.core.shield > 0),
            ("gj_inf_broken", self.core.shield <= 0),
        ] {
            if active {
                if !has(sim, owner, name) {
                    sim.add_buff(owner, &crate::timed(name, 5184000));
                }
            } else {
                sim.entity_remove_buff(owner, name);
            }
        }
        if let Some(d) = &self.core.domain {
            crate::steve::note_map_buff(sim.seed());
            cleared(sim, owner, "gj2_domain:");
            let name = format!(
                "gj2_domain:{}:{}:{}:{}:{}:{}",
                d.centre.0,
                d.centre.1,
                d.start + d.expand,
                d.end,
                d.guest.map_or(0, |n| n + 1),
                d.radius
            );
            sim.add_buff(owner, &crate::timed(&name, d.end.saturating_sub(tick) + 1));
            let r = 8000
                + (d.radius - 8000) * (tick - d.start).min(d.expand) as i64
                    / d.expand.max(1) as i64;
            sim.debug_draw_circle(d.centre.0 as u64, d.centre.1 as u64, r as u64, 0xB9F3FFFF);
            if tick % 6 == 0 {
                let elapsed = tick - d.start;
                let mode = if d.slow { "full" } else { "quick" };
                if elapsed < d.expand {
                    let frame = crate::gojo_v2_art::frame(mode, "cast", elapsed, d.expand);
                    crate::fx_point(
                        sim,
                        &format!("tfm2_custom_gojo_v2_void_{mode}_cast{frame}"),
                        owner,
                        d.centre.0,
                        d.centre.1,
                        8,
                    );
                } else {
                    for part in ["field", "ring"] {
                        let frame = crate::gojo_v2_art::frame(mode, part, elapsed, d.expand);
                        crate::fx_point(
                            sim,
                            &format!("tfm2_custom_gojo_v2_void_{mode}_{part}{frame}"),
                            owner,
                            d.centre.0,
                            d.centre.1,
                            8,
                        );
                    }
                }
                if let Some(id) = d.guest {
                    if let Some(e) = sim.get_entity(id).filter(|e| e.is_alive()) {
                        let p = e.pos();
                        crate::fx_point(
                            sim,
                            "tfm2_custom_gojo_new_11",
                            owner,
                            p.0 as i64,
                            p.1 as i64,
                            8,
                        );
                    }
                }
            }
        }
        if tick % 6 == 0 {
            if let Some(f) = &self.core.fusion {
                crate::fx_point(
                    sim,
                    "tfm2_custom_gojo_new_15",
                    owner,
                    f.point.0,
                    f.point.1,
                    8,
                );
            }
        }
        self.warnings.retain(|n| has(sim, owner, n));
        if tick % 6 == 0 {
            for name in &self.warnings {
                let v: Vec<i64> = name
                    .split(':')
                    .skip(1)
                    .filter_map(|v| v.parse().ok())
                    .collect();
                if v.len() == 4 && tick < v[3] as usize {
                    let radius = [3, 20, 26, 32, 36, 64, 76, 96]
                        .into_iter()
                        .min_by_key(|n| (n * 1000 - v[2]).abs())
                        .unwrap();
                    crate::fx_point(
                        sim,
                        &format!("tfm2_custom_gojo_ring_warning{radius}"),
                        owner,
                        v[0],
                        v[1],
                        8,
                    );
                }
            }
        }
    }
}
impl StablePassive for Gojo {
    fn clone_box(&self) -> Box<dyn StablePassive> {
        Box::new(self.clone())
    }
    fn on_spawn(&mut self, sim: &mut StableSim, _: usize, id: usize) {
        self.owner = id;
        sim.add_buff(id, &crate::timed("gj_rework", 5184000));
        sim.add_buff(id, &crate::timed("gj_six_eyes", 5184000));
    }
    fn on_dead(&mut self, sim: &mut StableSim, _: usize) {
        let out = self.core.died(sim.tick());
        self.execute(sim, out);
        self.finish_domain(sim);
        let ids: Vec<_> = (0..sim.entity_count())
            .filter_map(|i| sim.entity_at(i))
            .map(|e| e.id())
            .collect();
        for id in ids {
            sim.entity_remove_buff(id, &format!("gj_power{}", self.owner));
        }
        self.hud = [None; 2];
        self.status_expiry = [0; 2];
        cleared(sim, self.owner, "gj2_hit:");
        cleared(sim, self.owner, "gj2w:");
        for prefix in [
            "gj_ce",
            "gj_strain",
            "gj_amp",
            "gj_burnout",
            "gj_six_eyes",
            "gj_eye_exhausted",
            "gj_v2",
            "gj_inf_",
        ] {
            cleared(sim, self.owner, prefix);
        }
    }
    fn on_damaged(&mut self, sim: &mut StableSim, _: usize, _: usize, attacker: usize, _: usize) {
        if sim.get_entity(attacker).is_some_and(|e| e.is_champion()) {
            self.last_champion_hit = true;
        }
    }
    fn on_update(&mut self, sim: &mut StableSim, _: u64, _: usize, owner: usize) {
        let t = sim.tick();
        self.owner = owner;
        if self.last_tick == Some(t) {
            return;
        }
        self.last_tick = Some(t);
        let all = self.snapshot(sim, owner);
        let Some(me) = all.iter().find(|u| u.id == owner) else {
            return;
        };
        let names = sim
            .get_entity(owner)
            .map(|e| {
                (0..e.buff_count())
                    .filter_map(|i| e.buff_at(i))
                    .map(|b| b.name().to_string())
                    .filter(|n| n.starts_with("gj2_hit:"))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let mut basics = vec![];
        for n in names {
            let p = n
                .split(':')
                .skip(1)
                .filter_map(|n| n.parse::<usize>().ok())
                .collect::<Vec<_>>();
            if p.len() == 2 && p[0] <= t && t - p[0] <= 2 {
                basics.push((((p[0] as u64 + 1) << 32) | (p[1] as u64 + 1), p[1]));
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
                .is_some_and(|c| c.unlocked)
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
        let bound = cc(sim, owner, CcKindV1::Bind) || cc(sim, owner, CcKindV1::BlockMoveSkill);
        let champs = crate::champions(sim);
        let dynamic = crate::steve::walls_up(&champs, t);
        let smokes = crate::valorant::smokes_up(&champs, t);
        let mut domains = domain_zones(&champs, owner, t);
        domains.extend(crate::gojo::domain_zones(&champs, owner, t));
        let barrier = |x: i64, y: i64| {
            domains
                .iter()
                .any(|v| v.2 != 96000 && (v.3 == 0 || t >= v.3) && within((v.0, v.1), (x, y), v.2))
        };
        let wall = |x: i64, y: i64| {
            crate::walls::wall_at(x, y)
                || dynamic.iter().any(|w| {
                    crate::steve::crosses(w, t, x - 1000, y, x + 1000, y)
                        || crate::steve::crosses(w, t, x, y - 1000, x, y + 1000)
                })
        };
        // The SDK exposes visibility for entities, not arbitrary points. A short,
        // unobstructed ray from Gojo is a conservative visible destination.
        let sight = |x: i64, y: i64| {
            within(me.pos(), (x, y), 32000)
                && (0..=16).all(|n| {
                    let p = (me.x + (x - me.x) * n / 16, me.y + (y - me.y) * n / 16);
                    !wall(p.0, p.1)
                        && !smokes
                            .iter()
                            .any(|s| within((s.x, s.y), p, crate::valorant::SMOKE_R))
                })
        };
        let buffs = sim
            .get_entity(owner)
            .map(|e| {
                (0..e.buff_count())
                    .filter_map(|i| e.buff_at(i))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let haste = buffs
            .iter()
            .filter(|b| !b.name().starts_with("gj_power"))
            .map(|b| b.skill_cooldown_mult as i64)
            .sum();
        let ult_haste = buffs.iter().map(|b| b.ult_cooldown_mult as i64).sum();
        let input = Input {
            tick: t,
            me,
            all: &all,
            unlocked,
            controlled,
            bound,
            automatic: true,
            haste,
            ult_haste,
            action: None,
            basics: &basics,
            champion_damage: self.last_champion_hit,
            shield: if self.published_shield {
                sim.get_entity(owner).map(|e| e.shield() as i64)
            } else {
                None
            },
            wall: &wall,
            sight: &sight,
            barrier: &barrier,
        };
        self.last_champion_hit = false;
        let preparing=self.core.pending.is_some();
        let out = self.core.step(&input);
        if preparing && self.core.pending.is_none() && matches!(
            self.core.reason,
            "Interrupted" | "Preparation lost target" | "Spatial step invalidated"
        ) {
            let mut keep=vec![];
            for name in std::mem::take(&mut self.warnings) {
                if self.warning_survives_cancel(&name,t) {keep.push(name);} else {sim.entity_remove_buff(owner,&name);}
            }
            self.warnings=keep;
        }
        self.execute(sim, out);
        if me.hp <= 0 {
            return;
        }
        self.presentation(sim);
    }
}

impl Gojo {
    fn warning_survives_cancel(&self,name:&str,tick:usize)->bool {
        let v=name.split(':').skip(1).filter_map(|n|n.parse::<i64>().ok()).collect::<Vec<_>>();
        v.len()==4 && (v[3]<=tick as i64+6 || self.core.fusion.as_ref().is_some_and(|f|v==[f.point.0,f.point.1,96000,f.at as i64]))
    }
}

/// Owner-aware Domain geometry. The chosen guest and the caster are exempt;
/// all other allies face the same boundary and danger as enemies.
fn domain_zones(
    all: &[crate::Champ],
    owner: usize,
    tick: usize,
) -> Vec<(i64, i64, i64, usize, usize)> {
    let mut zones = vec![];
    for c in all.iter().filter(|c| c.id != owner) {
        for b in &c.buffs {
            if let Some(raw) = b.name().strip_prefix("gj2_domain:") {
                let v: Vec<i64> = raw.split(':').filter_map(|v| v.parse().ok()).collect();
                if v.len() == 6 && tick < v[3] as usize && v[4] != owner as i64 + 1 {
                    zones.push((v[0], v[1], v[5], v[2].max(0) as usize, 0));
                }
            }
        }
    }
    zones
}
pub fn danger_zones(
    all: &[crate::Champ],
    me: &crate::Champ,
    tick: usize,
) -> Vec<(i64, i64, i64, usize, usize)> {
    let mut zones = domain_zones(all, me.id, tick);
    for c in all.iter().filter(|c| c.team != me.team) {
        for b in &c.buffs {
            if let Some(raw) = b.name().strip_prefix("gj2w:") {
                let v: Vec<i64> = raw.split(':').filter_map(|v| v.parse().ok()).collect();
                if v.len() == 4 && tick < v[3] as usize {
                    zones.push((v[0], v[1], v[2], v[3] as usize, 0));
                }
            }
        }
    }
    zones
}
/// Movement-only warning avoidance; never injects attacks or skill inputs.
pub fn avoid(
    all: &[crate::Champ],
    me: &crate::Champ,
    dest: (i64, i64),
    tick: usize,
) -> Option<(i64, i64)> {
    let zones = danger_zones(all, me, tick);
    let bad = |p: (i64, i64)| zones.iter().any(|v| within((v.0, v.1), p, v.2 + 2500));
    if !bad((me.x, me.y)) && !bad(dest) {
        return None;
    }
    let mut candidates = vec![];
    for v in &zones {
        let d = crate::unified_theory_math::unit(me.x - v.0, me.y - v.1);
        let d = if d == (0, 0) { (10000, 0) } else { d };
        for (x, y) in [d, (-d.1, d.0), (d.1, -d.0)] {
            candidates.push((
                v.0 + x * (v.2 + 6000) / 10000,
                v.1 + y * (v.2 + 6000) / 10000,
            ));
        }
    }
    candidates
        .into_iter()
        .filter(|p| {
            p.0 >= 0
                && p.1 >= 0
                && p.0 < 960000
                && p.1 < 960000
                && !bad(*p)
                && !crate::walls::wall_at(p.0, p.1)
        })
        .min_by_key(|p| (distance(*p, dest), p.0, p.1))
}

#[cfg(test)]
#[path = "gojo_v2_bridge_tests.rs"]
mod bridge_tests;
