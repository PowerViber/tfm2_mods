//! Native bridge for Contender and the gated Shadow Link carrier prototype.
use crate::contender_shadow_core::{Barrier, Core, Effect, Frame, LinkTarget, Point, ShadowObservation, TargetDefence, Unit, CELL, CELLS};
use crate::contender_shadow_data as d;
use mod_api_stable::{AttackTypeV1, BuffV1, CcKindV1, CcV1, InputTargetV1, StableEffectType, StablePassive, StableSim, StatV1};

pub(crate) const GO_PREFIX: &str = "ct_go:";
/// a cleave spot other AIs should avoid: `ctw:x:y:r:until` on him (read by `danger_zones`)
const ZONE_PREFIX: &str = "ctw:";
const HOLD_PREFIX: &str = "ct_hold:";
const SHADOW_ATTACK: &str = "ct_attack_shadow:";
const CARRIER: &str = "ct_link_carrier";
const PROBE: bool = cfg!(feature = "contender-shadow-link-probe");

#[derive(Clone, Copy)]
struct Carrier {
    cast: u64,
    id: usize,
}


fn named(kind: CcKindV1, ticks: u64, name: &str) -> CcV1 {
    let mut c = CcV1::of_kind(kind, ticks);
    c.set_name(name);
    c
}
fn replace_buff(sim: &mut StableSim, id: usize, b: &BuffV1) {
    sim.entity_remove_buff(id, b.name());
    sim.add_buff(id, b);
}
fn remove_prefix(sim: &mut StableSim, id: usize, prefix: &str) {
    let names: Vec<String> = sim
        .get_entity(id)
        .map(|e| (0..e.buff_count()).filter_map(|i| e.buff_at(i)).filter(|b| b.name().starts_with(prefix)).map(|b| b.name().to_owned()).collect())
        .unwrap_or_default();
    for n in names {
        sim.entity_remove_buff(id, &n);
    }
}
const HARD: [CcKindV1; 5] = [CcKindV1::Stun, CcKindV1::Airborne, CcKindV1::Taunt, CcKindV1::Fear, CcKindV1::Charm];
fn hard_cc(sim: &StableSim, id: usize) -> bool {
    sim.get_entity(id).is_some_and(|e| {
        (0..e.cc_count())
            .filter_map(|i| e.cc_at(i))
            .any(|c| !c.name().starts_with("ct_") && !crate::motion::is_self_motion(&c) && HARD.iter().any(|k| k.code() == c.kind))
    })
}
/// Did `attacker` cast one of its engine skills (Skill, Skill2, Ult) in the last 1.5 s? The passive's `on_damaged` hook
/// doesn't say whether a hit was a basic attack, so a hit right after a cast counts as a skill. Native kits whose
/// skills bypass the engine slots (placeholder cooltimes) read as basics here.
pub(crate) fn recently_cast(sim: &StableSim, attacker: usize) -> bool {
    const WINDOW: usize = 90;
    let Some(a) = sim.get_entity(attacker) else { return false };
    [mod_api_stable::SkillSlotV1::Skill, mod_api_stable::SkillSlotV1::Skill2, mod_api_stable::SkillSlotV1::Ult].into_iter().any(|slot| {
        a.cooldown(slot).is_some_and(|c| c.cooltime > 0 && c.cooltime < 5_000_000 && c.remaining > 0 && c.remaining + WINDOW >= c.cooltime)
    })
}

fn unhurt(name: &str, ticks: u64, cc_immune: bool) -> BuffV1 {
    let mut b = crate::timed(name, ticks as usize);
    b.damaged_reduce = 100;
    b.base_attack_damaged_reduce = 100;
    b.skill_damaged_reduce = 100;
    b.cc_immune = cc_immune;
    b
}

/// The move a steering buff asks for (read by the shared input AI).
pub fn steer_of(name: &str) -> Option<(i64, i64)> {
    let mut it = name.strip_prefix(GO_PREFIX)?.split(':').filter_map(|s| s.parse::<i64>().ok());
    Some((it.next()?, it.next()?))
}

/// Adjust only an undecided order or a walk. The engine owns attacks and casts already chosen.
pub(crate) fn shadow_input(sim: &StableSim<'_>, owner: usize, base: Option<mod_api_stable::InputV1>) -> Option<mod_api_stable::InputV1> {
    use mod_api_stable::{InputKindV1, InputV1};
    if base.is_some_and(|input| input.kind != InputKindV1::Move.code()) {
        return None;
    }
    let me = sim.get_entity(owner)?;
    let (x, y) = me.pos();
    for buff in (0..me.buff_count()).filter_map(|i| me.buff_at(i)) {
        let name = buff.name();
        if let Some(raw) = name.strip_prefix(SHADOW_ATTACK) {
            let id: usize = raw.parse().ok()?;
            let target = sim.get_entity(id)?;
            let (tx, ty) = target.pos();
            if target.team() != me.team() && target.is_alive() && target.is_targetable()
                && sim.is_visible(me.team(), id)
                && Point::new(x as i64, y as i64).within(Point::new(tx as i64, ty as i64), d::BASIC_RANGE)
                && (0..target.buff_count()).filter_map(|i| target.buff_at(i)).any(|b| b.name() == CARRIER) {
                return Some(InputV1::action(InputKindV1::Attack, InputTargetV1::target(id)));
            }
        }
    }
    let movement = base?;
    for buff in (0..me.buff_count()).filter_map(|i| me.buff_at(i)) {
        if let Some(raw) = buff.name().strip_prefix(HOLD_PREFIX) {
            let mut coords = raw.split(':').filter_map(|part| part.parse::<i64>().ok());
            let (tx, ty, cx, cy) = (coords.next()?, coords.next()?, coords.next()?, coords.next()?);
            let dx = movement.x as i64 - x as i64;
            let dy = movement.y as i64 - y as i64;
            let forward = i128::from(dx) * i128::from(tx - x as i64) + i128::from(dy) * i128::from(ty - y as i64);
            if forward > 0 {
                return Some(InputV1::move_to(cx.max(0) as u64, cy.max(0) as u64));
            }
        }
    }
    None
}

/// Danger zones for other AIs: the cleave circles his shadows are spinning in.
pub fn danger_zones(all: &[crate::Champ], me: &crate::Champ, tick: usize) -> Vec<(i64, i64, i64, usize, usize)> {
    let mut zones = vec![];
    for c in all.iter().filter(|c| c.team != me.team) {
        for b in &c.buffs {
            if let Some(raw) = b.name().strip_prefix(ZONE_PREFIX) {
                let v: Vec<i64> = raw.split(':').filter_map(|s| s.parse::<i64>().ok()).collect();
                if v.len() == 4 && tick < v[3] as usize {
                    zones.push((v[0], v[1], v[2], v[3] as usize, 0));
                }
            }
        }
    }
    zones
}

/// Old careers keep the round-176 data that points its basic at `tfm2_custom_ai:contender_basic`: it still lands as a
/// plain basic attack.
pub struct BasicImpact;
impl StableEffectType for BasicImpact {
    fn apply(&self, sim: &mut StableSim, _: u64, caster: usize, input: InputTargetV1) {
        if input.kind == mod_api_stable::InputTargetKindV1::Target.code() {
            let ad = sim.get_entity(caster).map_or(0, |e| e.stat().attack);
            sim.deal_damage(caster, input.target_id, ad, 0, AttackTypeV1::BaseAttack);
        }
    }
    fn expected_damage(&self, s: &StatV1) -> (usize, usize) {
        (s.attack, 0)
    }
}
/// Old careers' `contender_ultimate` passive: the shared ult-learned marker.
pub struct UltimateLearned;
impl StablePassive for UltimateLearned {
    fn clone_box(&self) -> Box<dyn StablePassive> {
        Box::new(Self)
    }
    fn on_update(&mut self, sim: &mut StableSim, _: u64, _: usize, id: usize) {
        let has = sim.get_entity(id).is_some_and(|e| (0..e.buff_count()).filter_map(|i| e.buff_at(i)).any(|b| b.name() == crate::ULT_LEARNED));
        if !has && sim.get_entity(id).is_some_and(|e| e.is_alive()) {
            sim.add_buff(id, &BuffV1::named(crate::ULT_LEARNED));
        }
    }
}

#[derive(Clone, Default)]
pub struct Contender {
    pub core: Core,
    profile: Option<crate::athlete_ai_core::Profile>,
    terrain: Option<(u64, Vec<bool>)>,
    shadow_drawn: u64,
    carrier: Option<Carrier>,
    landing_receipt: Option<ShadowObservation>,
    hold_hint: Option<String>,
    shadow_hint: Option<String>,
    shadow_art: bool,
    shadow_directions: bool,
    merge_art: bool,
    merge_presenter: crate::contender_merge::Presenter,
    rhythm_art: bool,
    rhythm_presenter: crate::contender_rhythm::Presenter,
    cyclone_drawn: Option<(Point, u64)>,
    /// enemies his basic attacks landed on since the last frame
    basic_hits: Vec<usize>,
    in_skill: bool,
    fx_tokens: usize,
    fx_tick: u64,
}

impl Contender {
    fn grid(&mut self, tick: u64) -> Vec<bool> {
        if let Some((t, g)) = &self.terrain {
            if tick < t + 600 {
                return g.clone();
            }
        }
        let g: Vec<bool> = (0..CELLS * CELLS).map(|i| crate::walls::wall_at(i % CELLS * CELL + CELL / 2, i / CELLS * CELL + CELL / 2)).collect();
        self.terrain = Some((tick, g.clone()));
        g
    }

    pub(crate) fn frame(&mut self, sim: &StableSim, owner: usize) -> Frame {
        let tick = sim.tick() as u64;
        let me = sim.get_entity(owner);
        let team = me.as_ref().map_or(usize::MAX, |e| e.team());
        let (pos, hp, max_hp, attack, alive) = me.as_ref().map_or((Point::default(), 0, 1, 0, false), |e| {
            let p = e.pos();
            let h = e.hp();
            (Point::new(p.0 as i64, p.1 as i64), h.0 as i64, h.1.max(1) as i64, e.stat().attack as i64, e.is_alive())
        });
        let profile = crate::athlete_ai::pinned_owner(sim, owner, &mut self.profile);
        let mut values = [50; 13];
        values[..9].copy_from_slice(&profile.values);
        values[9..].copy_from_slice(&profile.sense.0);
        let snapshot = crate::champions_shared(sim);
        let realms = crate::realm::realms(sim);
        let farming = crate::farming::Context::new(sim, owner);
        let mut units = vec![];
        let mut link_excluded = vec![];
        let mut separated = vec![];
        let mut defences = vec![];
        let owner_can_spawn = PROBE && team < 2 && crate::realm::realm_of(&realms, owner).is_none()
            && snapshot.iter().find(|champ| champ.id == owner).and_then(crate::grimhald_cage::member_of).is_none();
        let barriers = crate::steve::walls_up(&snapshot, tick as usize).into_iter().map(|wall| {
            let (bx, by) = wall.built_end(tick as usize);
            Barrier {
                from: Point::new(wall.ax, wall.ay), to: Point::new(bx, by), half_width: crate::steve::WALL_HALF,
            }
        }).collect();
        let shadow_observation = self.landing_receipt.take().or_else(|| {
            let carrier = self.carrier?;
            let entity = sim.get_entity(carrier.id)?;
            if entity.is_alive() && !(0..entity.buff_count()).filter_map(|i| entity.buff_at(i)).any(|b| b.name() == CARRIER) {
                return None;
            }
            let hp = entity.hp().0 as i64;
            if !entity.is_alive() && hp != 0 {
                return None;
            }
            Some(ShadowObservation::Health { cast: carrier.cast, hp })
        });
        for e in (0..sim.entity_count()).filter_map(|i| sim.entity_at(i)) {
            if e.id() == owner || !e.is_alive() || (PROBE && self.carrier.is_some_and(|carrier| carrier.id == e.id()))
                || (PROBE && e.kind() == Some(mod_api_stable::EntityKindV1::Summon)
                    && (0..e.buff_count()).filter_map(|i| e.buff_at(i)).any(|b| b.name() == CARRIER)) {
                continue;
            }
            let e_team = e.team();
            let visible = e_team == team || sim.is_visible(team, e.id());
            let p = e.pos();
            let h = e.hp();
            let reach = crate::realm::can_interact(&realms, owner, e.id()) && crate::grimhald_cage::can_interact(&snapshot, owner, e.id());
            let kind = e.kind();
            let champion = e.is_champion();
            let targetable = e.is_targetable() && reach;
            if !reach { separated.push(e.id()); }
            if visible && champion && e_team != team {
                defences.push(TargetDefence { id: e.id(), armour: e.stat().defence as i64, shield: e.shield() as i64 });
            }
            if owner_can_spawn {
                use mod_api_stable::EntityKindV1 as K;
                let eligible = champion || matches!(kind, Some(K::Minion | K::Jungle)) || (kind.is_none() && e.is_minion());
                let immune = (0..e.buff_count()).filter_map(|i| e.buff_at(i)).any(|b| b.cc_immune);
                if !eligible || immune { link_excluded.push(e.id()); }
            }
            let linked = matches!(&self.core.shadow_link, Some(crate::contender_shadow_core::ShadowLink::Active { linked, .. }) if linked.contains(&e.id()));
            let farmable = farming.calm()
                && e_team != team
                && targetable
                && visible
                && !champion
                && farming.eligible_known(e.id(), e_team, kind, p, kind.is_none() && e.is_minion(), &crate::entity_buffs(&snapshot, &e));
            units.push(Unit {
                id: e.id(),
                team: e_team,
                pos: if visible || linked { Point::new(p.0 as i64, p.1 as i64) } else { Point::default() },
                hp: h.0 as i64,
                max_hp: h.1.max(1) as i64,
                radius: e.radius().max(1000) as i64,
                champion,
                tower: e.is_tower() || kind == Some(mod_api_stable::EntityKindV1::Nexus),
                visible,
                targetable,
                farmable,
            });
        }
        const HOME: [(i64, i64); 2] = [(96_000, 864_000), (864_000, 96_000)];
        let home = HOME.get(team).copied().unwrap_or(HOME[0]);
        Frame {
            tick,
            owner,
            team,
            alive,
            pos,
            hp,
            max_hp,
            attack,
            stunned: hard_cc(sim, owner),
            ult_unlocked: sim.get_entity(owner).is_some_and(|e| (0..e.buff_count()).filter_map(|i| e.buff_at(i)).any(|b| b.name() == crate::ULT_LEARNED)),
            level: sim.get_entity(owner).map_or(0, |e| e.level() as i64),
            home: Point::new(home.0, home.1),
            units,
            terrain: self.grid(tick),
            profile: values,
            manual: false,
            order: None,
            basic_hits: std::mem::take(&mut self.basic_hits),
            incoming: vec![],
            shadow_link_enabled: owner_can_spawn,
            shadow_observation,
            link_excluded,
            separated,
            barriers,
            defences,
        }
    }

    fn clear_carrier(&mut self, sim: &mut StableSim<'_>) {
        if let Some(carrier) = self.carrier.take() {
            if sim.get_entity(carrier.id).is_some_and(|e| e.is_alive()
                && (0..e.buff_count()).filter_map(|i| e.buff_at(i)).any(|b| b.name() == CARRIER)) {
                sim.entity_set_hp(carrier.id, 0);
            }
        }
        self.landing_receipt = None;
    }

    fn land_shadow(&mut self, sim: &mut StableSim<'_>, owner: usize, cast: u64, at: Point, candidates: Vec<LinkTarget>) {
        if !PROBE { return; }
        let Some(team) = sim.get_entity(owner).map(|e| e.team()).filter(|team| *team < 2) else { return };
        if self.carrier.is_some() { return; }
        let mut confirmed = Vec::new();
        for target in candidates {
            let eligible = sim.get_entity(target.id).is_some_and(|e| e.is_alive() && e.is_targetable()
                && !(0..e.buff_count()).filter_map(|i| e.buff_at(i)).any(|b| b.cc_immune));
            if !eligible { continue; }
            let name = format!("ct_link_slow:{owner}");
            let mut slow = crate::timed(&name, d::SHADOW_LINK_SLOW_TICKS as usize);
            slow.move_speed_mult = -d::SHADOW_LINK_SLOW_PCT as i32;
            replace_buff(sim, target.id, &slow);
            if sim.get_entity(target.id).is_some_and(|e| (0..e.buff_count())
                .filter_map(|i| e.buff_at(i)).any(|b| b.name() == name)) {
                confirmed.push(target);
            }
        }
        let Some(hp) = confirmed.iter().map(|target| target.max_hp).max() else { return };
        // Keep the carrier through the terminal observation at logical expiry.
        let life = d::SHADOW_LINK_TICKS as u64 + 2;
        let stat = StatV1 { hp: hp as usize, move_speed: 1, ..StatV1::default() };
        let attack = mod_api_stable::UnitAttackV1 { range: 1, cooltime: 5_184_000, duration: 1, ..Default::default() };
        let Some(id) = sim.spawn_unit("alchemist", owner, 1 - team, at.x as u64, at.y as u64, life, &stat, &attack) else { return };
        for kind in [CcKindV1::Bind, CcKindV1::BlockAttack, CcKindV1::BlockSkill] {
            sim.apply_cc(id, &named(kind, life, "ct_link_idle"));
        }
        let mut marker = crate::timed(CARRIER, life as usize);
        marker.cc_immune = true;
        sim.add_buff(id, &marker);
        self.carrier = Some(Carrier { id, cast });
        self.landing_receipt = Some(ShadowObservation::Landed { cast, targets: confirmed.into_iter().map(|target| target.id).collect() });
    }

    fn sync_hint(sim: &mut StableSim<'_>, owner: usize, slot: &mut Option<String>, hint: Option<String>) {
        if *slot != hint {
            if let Some(old) = slot.take() { sim.entity_remove_buff(owner, &old); }
            if let Some(next) = &hint { sim.add_buff(owner, &BuffV1::named(next)); }
            *slot = hint;
        }
    }

    fn fx_ok(&mut self, tick: u64) -> bool {
        if tick != self.fx_tick {
            let gained = (tick.saturating_sub(self.fx_tick) / 2) as usize;
            self.fx_tokens = (self.fx_tokens + gained.max(1)).min(d::FX_TOKENS);
            self.fx_tick = tick;
        }
        if self.fx_tokens > 0 {
            self.fx_tokens -= 1;
            true
        } else {
            false
        }
    }

    pub(crate) fn apply(&mut self, sim: &mut StableSim, owner: usize, effects: Vec<Effect>) {
        let tick = sim.tick() as u64;
        let all = crate::champions_shared(sim);
        let realms = crate::realm::realms(sim);
        let reach = |a: usize, b: usize| crate::realm::can_interact(&realms, a, b) && crate::grimhald_cage::can_interact(&all, a, b);
        let not_tower = |sim: &StableSim, id: usize| sim.get_entity(id).is_some_and(|e| !e.is_tower() && e.kind() != Some(mod_api_stable::EntityKindV1::Nexus));
        let mut hold = None;
        let mut prefer_shadow = false;
        for e in effects {
            match e {
                Effect::Damage { target, amount } | Effect::ShadowPayout { target, amount } => {
                    if reach(owner, target) && not_tower(sim, target) {
                        self.in_skill = true;
                        sim.deal_damage(owner, target, amount.max(0) as usize, 0, AttackTypeV1::Skill);
                        self.in_skill = false;
                    }
                }
                Effect::Stun { target, ticks } => {
                    if reach(owner, target) && not_tower(sim, target) {
                        sim.apply_cc(target, &named(CcKindV1::Stun, ticks, "ct_stun"));
                    }
                }
                Effect::KnockUp { target, ticks } => {
                    if reach(owner, target) && not_tower(sim, target) {
                        sim.apply_cc(target, &named(CcKindV1::Airborne, ticks, "ct_knockup"));
                    }
                }
                Effect::Slow { target, pct, ticks } => {
                    if reach(owner, target) && not_tower(sim, target) {
                        let mut b = crate::timed("ct_slow", ticks as usize);
                        b.move_speed_mult = -(pct.clamp(0, 90) as i32);
                        replace_buff(sim, target, &b);
                    }
                }
                Effect::Knock { target, to, ticks } => {
                    if reach(owner, target) && not_tower(sim, target) {
                        crate::motion::glide(sim, target, (to.x, to.y), ticks, crate::motion::Kind::Knock);
                    }
                }
                Effect::Glide { to, ticks } => {
                    crate::motion::glide(sim, owner, (to.x, to.y), ticks, crate::motion::Kind::SelfMove);
                }
                Effect::Teleport { to } => {
                    // the projection swap and the ult's reappearance (teleports by design), onto open ground
                    sim.entity_set_pos(owner, to.x.max(0) as u64, to.y.max(0) as u64);
                    crate::steve::note_placed(sim, owner, to.x, to.y);
                    replace_buff(sim, owner, &crate::timed(crate::place::MOVED, 2));
                }
                Effect::Vanish { ticks } => {
                    // contender-2.7: banished for the whole flurry (untargetable, unseen, input-locked; renewed every
                    // flurry tick), so enemies can't target or hit him; the unhurt, CC-immune lease stays as a backstop
                    sim.entity_banish(owner, owner, ticks as usize, "", "");
                    sim.entity_set_invisible(owner, ticks as usize);
                    sim.apply_cc(owner, &named(CcKindV1::Bind, ticks, "ct_vanish"));
                    sim.apply_cc(owner, &named(CcKindV1::BlockAttack, ticks, "ct_vanish_attack"));
                    replace_buff(sim, owner, &unhurt("ct_vanish", ticks, true));
                }
                Effect::Invulnerable { ticks } => {
                    replace_buff(sim, owner, &unhurt("ct_inv", ticks, false));
                }
                Effect::Merge { ticks, move_pct } => {
                    let mut b = crate::timed("ct_merge", ticks as usize);
                    b.move_speed_mult = move_pct as i32;
                    replace_buff(sim, owner, &b);
                    self.shadow_drawn = 0;
                }
                Effect::Rhythm { attack_speed, attack_permille, move_permille, ticks } => {
                    let mut b = crate::timed("ct_rhythm", ticks as usize);
                    b.attack_speed_mult = attack_speed as i32;
                    b.attack_mult = (attack_permille / 10) as i32;
                    b.move_speed_mult = (move_permille / 10) as i32;
                    replace_buff(sim, owner, &b);
                }
                Effect::Charges { count } => {
                    // the pips show up to five (Merge's extra dodges ride on top)
                    remove_prefix(sim, owner, "ct_charges:");
                    sim.add_buff(owner, &BuffV1::named(&format!("ct_charges:{}", count.clamp(0, d::CHARGES_MAX))));
                }
                Effect::Steer { to } => {
                    remove_prefix(sim, owner, GO_PREFIX);
                    sim.add_buff(owner, &crate::timed(&format!("{GO_PREFIX}{}:{}", to.x, to.y), 6));
                }
                Effect::Fx { name, at, ticks, heading } => {
                    let name = if name == "ct_break" && !self.rhythm_art { "ct_haymaker".into() } else { name };
                    if name == "ct_shadow" {
                        // a cleave spot: other AIs read it as danger
                        remove_prefix(sim, owner, ZONE_PREFIX);
                        let until = tick + ticks;
                        sim.add_buff(owner, &crate::timed(&format!("{ZONE_PREFIX}{}:{}:{}:{until}", at.x, at.y, d::CYCLONE_RADIUS), ticks as usize));
                    }
                    if !(self.shadow_art && name == "ct_shadow") && self.fx_ok(tick) {
                        let visual = if self.shadow_art && self.shadow_directions && name == "ct_flurry" && heading.is_some() {
                            format!("ct_flurry_d{}_t{ticks}", heading.unwrap_or(0) % 8)
                        } else if self.shadow_art {
                            let left = sim.get_entity(owner).is_some_and(|e| (e.pos().0 as i64) > at.x);
                            format!("{name}_t{ticks}{}", if left { "_l" } else { "" })
                        } else { name };
                        crate::fx_point(sim, &visual, owner, at.x, at.y, ticks);
                    }
                }
                Effect::ShadowLand { cast, at, candidates } => {
                    let valid = candidates.into_iter().filter(|target| reach(owner, target.id)).collect();
                    self.land_shadow(sim, owner, cast, at, valid);
                }
                Effect::ShadowStrike { cast, amount } => {
                    if let Some(carrier) = self.carrier.filter(|carrier| carrier.cast == cast) {
                        if reach(owner, carrier.id) {
                            self.in_skill = true;
                            sim.deal_damage(owner, carrier.id, amount.max(0) as usize, 0, AttackTypeV1::Skill);
                            self.in_skill = false;
                        }
                    }
                }
                Effect::ShadowEnd { cast, .. } => {
                    if self.carrier.is_some_and(|carrier| carrier.cast == cast) { self.clear_carrier(sim); }
                }
                Effect::PreferShadow => prefer_shadow = true,
                Effect::HoldForward { threat, cover } => {
                    hold = Some(format!("{HOLD_PREFIX}{}:{}:{}:{}", threat.x, threat.y, cover.x, cover.y));
                }
                Effect::Log { .. } => {}
            }
        }
        // S2's clone exists only while the normal cleave is active. Short adjoining
        // leases avoid replaying a long clone animation over the recast's real body.
        if self.shadow_art {
            if let Some(c) = self.core.cleaves.iter().find(|c| !c.recast) {
                let at = c.at;
                if self.cyclone_drawn.is_none_or(|(p, due)| p != at || tick >= due) {
                    let left = sim.get_entity(owner).is_some_and(|e| (e.pos().0 as i64) > at.x);
                    let name = format!("ct_body_punch{}{}", (tick / 10) % 2, if left { "_l" } else { "" });
                    crate::fx_point(sim, &name, owner, at.x, at.y, 10);
                    self.cyclone_drawn = Some((at, tick + 10));
                }
            } else {
                self.cyclone_drawn = None;
            }
        }
        // Legacy definitions keep their separate Merge overlay.
        if !self.merge_art && self.core.merged(tick) && tick >= self.shadow_drawn + 10 {
            let name = if self.shadow_art { "ct_body_idle0" } else { "ct_shadow" };
            crate::fx_unit(sim, name, owner, owner, 10);
            self.shadow_drawn = tick;
        }
        if hold.is_some() && !self.core.reason.starts_with("Retreat") && !self.core.reason.starts_with("Peel") {
            remove_prefix(sim, owner, GO_PREFIX);
        }
        let attack = if prefer_shadow { self.carrier.map(|carrier| format!("{SHADOW_ATTACK}{}", carrier.id)) } else { None };
        Self::sync_hint(sim, owner, &mut self.hold_hint, hold);
        Self::sync_hint(sim, owner, &mut self.shadow_hint, attack);
    }
}

impl StablePassive for Contender {
    fn configure(&mut self, params: &str) {
        self.shadow_art = serde_json::from_str::<serde_json::Value>(params).ok()
            .and_then(|v| v.get("shadow_art_v1").and_then(|n| n.as_i64())) == Some(1);
        self.shadow_directions = serde_json::from_str::<serde_json::Value>(params).ok()
            .and_then(|v| v.get("shadow_directions_v1").and_then(|n| n.as_i64())) == Some(1);
        self.merge_art = serde_json::from_str::<serde_json::Value>(params).ok()
            .and_then(|v| v.get("shadow_merge_art_v1").and_then(|n| n.as_i64())) == Some(1);
        self.rhythm_art = serde_json::from_str::<serde_json::Value>(params).ok()
            .and_then(|v| v.get("shadow_rhythm_art_v1").and_then(|n| n.as_i64())) == Some(1);
    }
    fn clone_box(&self) -> Box<dyn StablePassive> {
        Box::new(self.clone())
    }
    fn on_spawn(&mut self, sim: &mut StableSim, _: usize, id: usize) {
        self.merge_presenter.clear(sim);
        self.merge_presenter = crate::contender_merge::Presenter::default();
        self.rhythm_presenter.clear(sim);
        self.rhythm_presenter = crate::contender_rhythm::Presenter::default();
        self.clear_carrier(sim);
        Self::sync_hint(sim, id, &mut self.hold_hint, None);
        Self::sync_hint(sim, id, &mut self.shadow_hint, None);
        self.core.respawn();
        sim.entity_remove_buff(id, "ct_merge");
        remove_prefix(sim, id, GO_PREFIX);
        remove_prefix(sim, id, ZONE_PREFIX);
        remove_prefix(sim, id, "ct_bank:");
        self.basic_hits.clear();
        self.in_skill = false;
        self.cyclone_drawn = None;
    }
    /// A Shadow dodge: an enemy champion's hit, with a dodge ready, is healed straight back (Kratos's pattern; the hook
    /// reports damage after it lands) and the core slips him aside on its next step.
    fn on_damaged(&mut self, sim: &mut StableSim, _: usize, id: usize, attacker: usize, damage: usize) {
        if damage > 0 && self.merge_art { self.merge_presenter.hit(sim.tick() as u64); }
        if damage == 0 {
            return;
        }
        let team = sim.get_entity(id).map(|e| e.team());
        let Some(from) = sim.get_entity(attacker).filter(|a| a.is_champion() && Some(a.team()) != team).map(|a| a.pos()) else { return };
        // contender-2.7: only an enemy champion's basic attack can be dodged; skills and ults always land
        if crate::contender_shadow_data::CHARGES_BASICS_ONLY != 0 && recently_cast(sim, attacker) {
            return;
        }
        if self.core.try_dodge(sim.tick() as u64, Point::new(from.0 as i64, from.1 as i64)) {
            sim.heal(id, id, damage);
        }
    }
    /// His Jab / Cross: never across a cage or a ring; Shadow Rhythm's vulnerability; the hit goes to the core.
    fn on_attack(&mut self, sim: &mut StableSim, _: usize, id: usize, target: usize, damage: &mut usize) {
        if self.in_skill || (PROBE && sim.get_entity(target).is_some_and(|e| (0..e.buff_count())
            .filter_map(|i| e.buff_at(i)).any(|b| b.name() == CARRIER))) {
            return;
        }
        let all = crate::champions_shared(sim);
        let realms = crate::realm::realms(sim);
        if !crate::grimhald_cage::can_interact(&all, id, target) || !crate::realm::can_interact(&realms, id, target) {
            *damage = 0;
            return;
        }
        let vuln = self.core.vuln_permille(target, sim.tick() as u64);
        if vuln > 0 {
            *damage = (*damage as i64 * (1_000 + vuln) / 1_000).max(0) as usize;
        }
        if sim.get_entity(target).is_some_and(|e| !e.is_tower()) && self.basic_hits.len() < 16 {
            self.basic_hits.push(target);
        }
    }
    fn on_base_attack(&mut self, sim: &mut StableSim, _: u64, _: usize, _: usize) {
        if self.merge_art { self.merge_presenter.basic(sim.tick() as u64); }
    }
    fn on_dead(&mut self, sim: &mut StableSim, id: usize) {
        self.clear_carrier(sim);
        Self::sync_hint(sim, id, &mut self.hold_hint, None);
        Self::sync_hint(sim, id, &mut self.shadow_hint, None);
        self.merge_presenter.clear(sim);
        self.rhythm_presenter.clear(sim);
        remove_prefix(sim, id, "ct_bank:");
    }
    fn on_update(&mut self, sim: &mut StableSim, _: u64, player: usize, id: usize) {
        let f = self.frame(sim, id);
        let effects = self.core.step(&f);
        if self.merge_art {
            let cd = sim.get_player(player).and_then(|p| p.cooldowns()).map(|c| c.0);
            self.merge_presenter.sync(sim, id, &f, &self.core, &effects, cd);
        }
        if self.rhythm_art { self.rhythm_presenter.sync(sim, id, player, &f, &self.core, &effects); }
        self.apply(sim, id, effects);
    }
}
