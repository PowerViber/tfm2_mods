//! Native adapter for the deterministic Contender combat core.
use crate::contender_shadow_core::{Barrier, Core, Effect, Frame, HitSource, IncomingHit, Point, ShadowLink, TargetDefence, Unit, CELL, CELLS};
use crate::contender_shadow_data as d;
use mod_api_stable::{AttackTypeV1, BuffV1, CcKindV1, CcV1, InputTargetV1, StableEffectType, StablePassive, StableSim, StatV1};
use std::cell::Cell;

pub(crate) const GO_PREFIX: &str = "ct_go:";
const ZONE_PREFIX: &str = "ctw:";
const HOLD_PREFIX: &str = "ct_hold:";
const BASIC_RECEIPT: &str = "ct_basic_landed:";

// Set only during a synchronous host damage call. Never persisted or shared between simulated matches.
thread_local! {
    static COUNTER_PACKET: Cell<Option<(usize, usize, usize)>> = const { Cell::new(None) };
}

fn health(sim: &StableSim<'_>, id: usize) -> Option<(usize, usize)> {
    sim.get_entity(id).map(|entity| (entity.hp().0, entity.shield()))
}

fn landed(sim: &StableSim<'_>, id: usize, before: Option<(usize, usize)>) -> bool {
    before.zip(health(sim, id)).is_some_and(|((hp, shield), (after_hp, after_shield))|
        after_hp < hp || after_shield < shield)
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
            .any(|c| !matches!(c.name(), "ct_vanish" | "ct_vanish_attack") && !crate::motion::is_self_motion(&c) && HARD.iter().any(|k| k.code() == c.kind))
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

/// Warning circles other AIs can avoid using their normal movement rules.
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

/// Preserve the engine's BaseAttack hit and item identity, and sample the delivered impact.
pub struct BasicImpact;
impl StableEffectType for BasicImpact {
    fn apply(&self, sim: &mut StableSim, _: u64, caster: usize, input: InputTargetV1) {
        if input.kind != mod_api_stable::InputTargetKindV1::Target.code() {
            return;
        }
        let target = input.target_id;
        let attack = sim.get_entity(caster).map_or(0, |e| e.stat().attack);
        let before = health(sim, target);
        sim.deal_damage(caster, target, attack, 0, AttackTypeV1::BaseAttack);
        if landed(sim, target, before) {
            let tick = sim.tick();
            sim.add_buff(caster, &crate::timed(&format!("{BASIC_RECEIPT}{tick}:{target}"), 3));
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
    hold_hint: Option<String>,
    shadow_art: bool,
    shadow_directions: bool,
    merge_art: bool,
    merge_presenter: crate::contender_merge::Presenter,
    rhythm_art: bool,
    rhythm_presenter: crate::contender_rhythm::Presenter,
    link_drawn_at: u64,
    receipt_basic: bool,
    landed_hits: u16,
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
        let barriers = crate::steve::walls_up(&snapshot, tick as usize).into_iter().map(|wall| {
            let (bx, by) = wall.built_end(tick as usize);
            Barrier {
                from: Point::new(wall.ax, wall.ay), to: Point::new(bx, by), half_width: crate::steve::WALL_HALF,
            }
        }).collect();
        let link_airborne_name = format!("ct_link_air_{owner}");
        let mut link_airborne = Vec::new();
        let watch_link = self.core.ult_ready(&Frame { tick, level: 0, ult_unlocked: true, ..Frame::default() })
            || self.core.shadow_link.is_some();
        for e in (0..sim.entity_count()).filter_map(|i| sim.entity_at(i)) {
            if e.id() == owner || !e.is_alive() {
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
            if watch_link && e_team != team {
                use mod_api_stable::EntityKindV1 as K;
                let eligible = champion || matches!(kind, Some(K::Minion | K::Jungle)) || (kind.is_none() && e.is_minion());
                let immune = (0..e.buff_count()).filter_map(|i| e.buff_at(i)).any(|b| b.cc_immune);
                if !eligible || immune { link_excluded.push(e.id()); }
                if matches!(self.core.shadow_link, Some(ShadowLink::Airborne { .. }))
                    && (0..e.cc_count()).filter_map(|i| e.cc_at(i)).any(|cc| cc.name() == link_airborne_name) {
                    link_airborne.push(e.id());
                }
            }
            let counter_source = self.core.pending_dodge.is_some_and(|hit| hit.attacker == e.id());
            let farmable = farming.calm()
                && e_team != team
                && targetable
                && visible
                && !champion
                && farming.eligible_known(e.id(), e_team, kind, p, kind.is_none() && e.is_minion(), &crate::entity_buffs(&snapshot, &e));
            units.push(Unit {
                id: e.id(),
                team: e_team,
                pos: if visible || counter_source { Point::new(p.0 as i64, p.1 as i64) } else { Point::default() },
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
            landed_hits: std::mem::take(&mut self.landed_hits),
            link_excluded,
            link_airborne,
            separated,
            barriers,
            defences,
        }
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
        for e in effects {
            match e {
                Effect::Damage { target, amount, source } => {
                    if amount > 0 && reach(owner, target) && not_tower(sim, target) {
                        let before = source.credits_flurry().then(|| health(sim, target)).flatten();
                        let counter = source == HitSource::Counter;
                        let key = (sim.instance(), owner, target);
                        let previous = COUNTER_PACKET.with(|packet| packet.replace(counter.then_some(key)));
                        self.in_skill = true;
                        sim.deal_damage(owner, target, amount as usize, 0, AttackTypeV1::Skill);
                        self.in_skill = false;
                        COUNTER_PACKET.with(|packet| packet.set(previous));
                        if landed(sim, target, before) {
                            self.landed_hits = self.landed_hits.saturating_add(1);
                        }
                    }
                }
                Effect::LinkStun { target, ticks } => {
                    if reach(owner, target) && not_tower(sim, target) {
                        sim.apply_cc(target, &named(CcKindV1::Stun, ticks, &format!("ct_link_stun_{owner}")));
                    }
                }
                Effect::LinkAirborne { target, ticks } => {
                    if reach(owner, target) && not_tower(sim, target) {
                        sim.apply_cc(target, &named(CcKindV1::Airborne, ticks, &format!("ct_link_air_{owner}")));
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
                    if self.fx_ok(tick) {
                        let visual = if self.shadow_art && self.shadow_directions && name == "ct_flurry" && heading.is_some() {
                            format!("ct_flurry_d{}_t{ticks}", heading.unwrap_or(0) % 8)
                        } else if self.shadow_art {
                            let left = sim.get_entity(owner).is_some_and(|e| (e.pos().0 as i64) > at.x);
                            format!("{name}_t{ticks}{}", if left { "_l" } else { "" })
                        } else { name };
                        crate::fx_point(sim, &visual, owner, at.x, at.y, ticks);
                    }
                }
                Effect::LinkWarning { at, until } => {
                    self.link_drawn_at = 0;
                    remove_prefix(sim, owner, ZONE_PREFIX);
                    let radius = d::SHADOW_LINK_RADIUS;
                    sim.add_buff(owner, &crate::timed(&format!("{ZONE_PREFIX}{}:{}:{radius}:{until}", at.x, at.y),
                        until.saturating_sub(tick) as usize));
                }
                Effect::LinkEnd => remove_prefix(sim, owner, ZONE_PREFIX),
                Effect::HoldForward { threat, cover } => {
                    hold = Some(format!("{HOLD_PREFIX}{}:{}:{}:{}", threat.x, threat.y, cover.x, cover.y));
                }
                Effect::Log { .. } => {}
            }
        }
        if self.shadow_art && tick >= self.link_drawn_at {
            match self.core.shadow_link {
                Some(ShadowLink::Flight { cursor, from, at, .. }) => {
                    let facing = if at.x < from.x { "_l" } else { "" };
                    let name = format!("ct_body_run{}{facing}", (tick / 10) % 6);
                    crate::fx_point(sim, &name, owner, cursor.x, cursor.y, 4);
                    self.link_drawn_at = tick + 4;
                }
                Some(ShadowLink::Warning { at, until, .. }) => {
                    let lease = (until.saturating_sub(tick)).min(6);
                    if lease > 0 {
                        crate::fx_point(sim, "ct_link_warning", owner, at.x, at.y, lease);
                    }
                    self.link_drawn_at = tick + 6;
                }
                _ => {}
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
        Self::sync_hint(sim, owner, &mut self.hold_hint, hold);
    }
}

impl StablePassive for Contender {
    fn configure(&mut self, params: &str) {
        self.receipt_basic = serde_json::from_str::<serde_json::Value>(params).ok()
            .and_then(|v| v.get("shadow_receipts_v1").and_then(|n| n.as_i64())) == Some(1);
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
        Self::sync_hint(sim, id, &mut self.hold_hint, None);
        self.core.respawn();
        sim.entity_remove_buff(id, "ct_merge");
        remove_prefix(sim, id, GO_PREFIX);
        remove_prefix(sim, id, BASIC_RECEIPT);
        remove_prefix(sim, id, ZONE_PREFIX);
        self.basic_hits.clear();
        self.in_skill = false;
        self.link_drawn_at = 0;
        self.landed_hits = 0;
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
        let counter = COUNTER_PACKET.with(|packet| packet.get() == Some((sim.instance(), attacker, id)));
        let hit = IncomingHit { attacker, from: Point::new(from.0 as i64, from.1 as i64), counter };
        if self.core.try_dodge(sim.tick() as u64, hit) {
            sim.heal(id, id, damage);
        }
    }
    /// His Jab / Cross: never across a cage or a ring; Shadow Rhythm's vulnerability; the hit goes to the core.
    fn on_attack(&mut self, sim: &mut StableSim, _: usize, id: usize, target: usize, damage: &mut usize) {
        if self.in_skill {
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
        // Embedded definitions can still have a stock Attack effect and no delivery receipt.
        if !self.receipt_basic && *damage > 0 && self.basic_hits.len() < 16
            && sim.get_entity(target).is_some_and(|e| e.team() != sim.get_entity(id).map_or(usize::MAX, |owner| owner.team())) {
            self.basic_hits.push(target);
        }
    }
    fn on_base_attack(&mut self, sim: &mut StableSim, _: u64, _: usize, _: usize) {
        if self.merge_art { self.merge_presenter.basic(sim.tick() as u64); }
    }
    fn on_dead(&mut self, sim: &mut StableSim, id: usize) {
        Self::sync_hint(sim, id, &mut self.hold_hint, None);
        self.merge_presenter.clear(sim);
        self.rhythm_presenter.clear(sim);
        remove_prefix(sim, id, BASIC_RECEIPT);
        remove_prefix(sim, id, ZONE_PREFIX);
    }
    fn on_update(&mut self, sim: &mut StableSim, _: u64, player: usize, id: usize) {
        if let Some(entity) = sim.get_entity(id) {
            let receipts: Vec<_> = (0..entity.buff_count()).filter_map(|i| entity.buff_at(i))
                .filter_map(|b| b.name().strip_prefix(BASIC_RECEIPT).and_then(|raw| raw.split_once(':'))
                    .and_then(|(_, target)| target.parse::<usize>().ok().map(|target| (b.name().to_owned(), target))))
                .collect();
            for (name, target) in receipts {
                sim.entity_remove_buff(id, &name);
                if self.basic_hits.len() < 16 && self.receipt_basic {
                    self.basic_hits.push(target);
                }
            }
        }
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
