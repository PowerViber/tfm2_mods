//! Contender 2.0's bridge (round 178): reads the sim into a `contender_shadow_core::Frame` once per tick and applies the
//! core's effects. The engine walks him and throws his Jab / Cross (the data basic attack); native code casts everything
//! else (the data slots are never-cast shells, like Kicker's, Kagami's and Basalt's).
//!
//! * His projected shadow (S1) is a hidden carrier unit with his max HP (Thalassa's and Raze's pattern): enemies can
//!   hit it, the core relays 70% of what it loses to him, and its body is drawn as effects. It moves by engine glides.
//! * His body during a projection: a named Bind, a named attack block and an unhurt buff, renewed as a short lease.
//! * The ult's vanish: `entity_set_invisible` plus an unhurt, CC-immune buff and a named Bind, renewed as a lease; he
//!   reappears by `entity_set_pos` (a teleport by design), as does the projection swap.
//! * Basic attacks: `on_attack` adds Shadow Rhythm's vulnerability and reports the hit to the core (stacks, echoes,
//!   the ult's cooldown, the charge recharge, the haymaker).
//! * The AI's walk (`Effect::Steer`) is a short `ct_go:x:y` buff the shared input AI turns into a move order.
//!
//! The old kit (Combination, Slip / Guard, Main Event) is retired: `contender_core.rs` and `contender_arena.rs` stay
//! for the realm code that shares their types, and no ring is ever opened now.
use crate::contender_shadow_core::{Core, Effect, Frame, Point, ShadowView, Unit, CELL, CELLS};
use crate::contender_shadow_data as d;
use mod_api_stable::{AttackTypeV1, BuffV1, CcKindV1, CcV1, InputTargetV1, StableEffectType, StablePassive, StableSim, StatV1, UnitAttackV1};

pub(crate) const GO_PREFIX: &str = "ct_go:";
const SHADOW_TAG: &str = "ct_shadow_unit";
/// a cleave spot other AIs should avoid: `ctw:x:y:r:until` on him (read by `danger_zones`)
const ZONE_PREFIX: &str = "ctw:";

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
    /// the projected shadow's carrier unit
    pub shadow: Option<usize>,
    shadow_drawn: u64,
    shadow_art: bool,
    shadow_last: Option<Point>,
    shadow_punch_until: u64,
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
        for e in (0..sim.entity_count()).filter_map(|i| sim.entity_at(i)) {
            if e.id() == owner || !e.is_alive() || Some(e.id()) == self.shadow {
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
            let farmable = farming.calm()
                && e_team != team
                && targetable
                && visible
                && !champion
                && farming.eligible_known(e.id(), e_team, kind, p, kind.is_none() && e.is_minion(), &crate::entity_buffs(&snapshot, &e));
            units.push(Unit {
                id: e.id(),
                team: e_team,
                pos: if visible { Point::new(p.0 as i64, p.1 as i64) } else { Point::default() },
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
        let shadow = self.shadow.and_then(|id| sim.get_entity(id).filter(|e| e.is_alive())).map(|e| {
            let p = e.pos();
            let h = e.hp();
            ShadowView { id: e.id(), pos: Point::new(p.0 as i64, p.1 as i64), hp: h.0 as i64, max_hp: h.1.max(1) as i64 }
        });
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
            home: Point::new(home.0, home.1),
            units,
            terrain: self.grid(tick),
            profile: values,
            manual: false,
            order: None,
            basic_hits: std::mem::take(&mut self.basic_hits),
            shadow,
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

    /// The projected shadow's hidden carrier: his max HP and defences, no attack of its own (the core punches for it).
    fn spawn_shadow(&mut self, sim: &mut StableSim, owner: usize, at: Point, hp: i64, ticks: u64) {
        let Some(me) = sim.get_entity(owner) else { return };
        let (team, stat_me) = (me.team(), me.stat());
        let stat = StatV1 {
            hp: hp.max(1) as usize,
            defence: stat_me.defence,
            magic_resistance: stat_me.magic_resistance,
            move_speed: 1,
            ..StatV1::default()
        };
        let attack = UnitAttackV1 { attack: 0, attack_ratio: 0, range: 1, cooltime: 5_184_000, duration: 1, start_timing: 0, ..UnitAttackV1::default() };
        let Some(id) = sim.spawn_unit("alchemist", owner, team, at.x.max(0) as u64, at.y.max(0) as u64, ticks, &stat, &attack) else { return };
        for k in [CcKindV1::BlockAttack, CcKindV1::BlockSkill] {
            sim.apply_cc(id, &named(k, ticks, "ct_shadow_hold"));
        }
        sim.apply_cc(id, &named(CcKindV1::Animation, ticks, "ct_shadow_hidden"));
        sim.add_buff(id, &crate::timed(&format!("{SHADOW_TAG}:{owner}"), ticks as usize));
        self.shadow = Some(id);
        self.shadow_drawn = (sim.tick() as u64).saturating_sub(10);
        self.shadow_last = None;
        if self.shadow_art {
            crate::fx_unit(sim, "ct_body_idle0", owner, id, 10);
            self.shadow_drawn = sim.tick() as u64;
        }
    }

    pub(crate) fn apply(&mut self, sim: &mut StableSim, owner: usize, effects: Vec<Effect>) {
        let tick = sim.tick() as u64;
        let all = crate::champions_shared(sim);
        let realms = crate::realm::realms(sim);
        let reach = |a: usize, b: usize| crate::realm::can_interact(&realms, a, b) && crate::grimhald_cage::can_interact(&all, a, b);
        let not_tower = |sim: &StableSim, id: usize| sim.get_entity(id).is_some_and(|e| !e.is_tower() && e.kind() != Some(mod_api_stable::EntityKindV1::Nexus));
        for e in effects {
            match e {
                Effect::Damage { target, amount } => {
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
                Effect::Hold { ticks } => {
                    sim.apply_cc(owner, &named(CcKindV1::Bind, ticks, "ct_hold"));
                    sim.apply_cc(owner, &named(CcKindV1::BlockAttack, ticks, "ct_hold_attack"));
                    replace_buff(sim, owner, &unhurt("ct_hold", ticks, false));
                }
                Effect::Vanish { ticks } => {
                    sim.entity_set_invisible(owner, ticks as usize);
                    sim.apply_cc(owner, &named(CcKindV1::Bind, ticks, "ct_vanish"));
                    sim.apply_cc(owner, &named(CcKindV1::BlockAttack, ticks, "ct_vanish_attack"));
                    replace_buff(sim, owner, &unhurt("ct_vanish", ticks, true));
                }
                Effect::Invulnerable { ticks } => {
                    replace_buff(sim, owner, &unhurt("ct_inv", ticks, false));
                }
                Effect::SpawnShadow { at, hp, ticks } => {
                    if let Some(old) = self.shadow.take() {
                        sim.entity_set_hp(old, 0);
                    }
                    self.spawn_shadow(sim, owner, at, hp, ticks);
                }
                Effect::MoveShadow { to, ticks } => {
                    if let Some(id) = self.shadow {
                        crate::motion::glide(sim, id, (to.x, to.y), ticks, crate::motion::Kind::SelfMove);
                    }
                }
                Effect::DespawnShadow => {
                    if let Some(id) = self.shadow.take() {
                        sim.entity_set_hp(id, 0);
                    }
                }
                Effect::SelfDamage { amount } => {
                    if amount > 0 {
                        sim.deal_damage_raw(owner, owner, amount as usize, 0, AttackTypeV1::Skill);
                    }
                }
                Effect::Rhythm { attack_speed, attack_permille, move_permille, ticks } => {
                    let mut b = crate::timed("ct_rhythm", ticks as usize);
                    b.attack_speed_mult = attack_speed as i32;
                    b.attack_mult = (attack_permille / 10) as i32;
                    b.move_speed_mult = (move_permille / 10) as i32;
                    replace_buff(sim, owner, &b);
                }
                Effect::Charges { count } => {
                    remove_prefix(sim, owner, "ct_charges:");
                    sim.add_buff(owner, &BuffV1::named(&format!("ct_charges:{count}")));
                }
                Effect::Steer { to } => {
                    remove_prefix(sim, owner, GO_PREFIX);
                    sim.add_buff(owner, &crate::timed(&format!("{GO_PREFIX}{}:{}", to.x, to.y), 6));
                }
                Effect::Fx { name, at, ticks } => {
                    if name == "ct_cross" && ticks == 10 && self.shadow.is_some() {
                        self.shadow_punch_until = tick + ticks;
                    }
                    if name == "ct_shadow" {
                        // a cleave spot: other AIs read it as danger
                        remove_prefix(sim, owner, ZONE_PREFIX);
                        let until = tick + ticks;
                        sim.add_buff(owner, &crate::timed(&format!("{ZONE_PREFIX}{}:{}:{}:{until}", at.x, at.y, d::CYCLONE_RADIUS), ticks as usize));
                    }
                    if !(self.shadow_art && name == "ct_shadow") && self.fx_ok(tick) {
                        let visual = if self.shadow_art {
                            let left = sim.get_entity(owner).is_some_and(|e| (e.pos().0 as i64) > at.x);
                            format!("{name}_t{ticks}{}", if left { "_l" } else { "" })
                        } else { name };
                        crate::fx_point(sim, &visual, owner, at.x, at.y, ticks);
                    }
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
        // the projected shadow's body: drawn where its carrier stands
        if let Some(id) = self.shadow {
            if tick >= self.shadow_drawn + 10 {
                if let Some((x, y)) = sim.get_entity(id).filter(|e| e.is_alive()).map(|e| e.pos()) {
                    let at = Point::new(x as i64, y as i64);
                    if self.shadow_art {
                        let previous = self.shadow_last.unwrap_or(at);
                        let moving = at.dist(previous) > 1000.0;
                        let left = if moving { at.x < previous.x } else {
                            sim.get_entity(owner).is_some_and(|e| (e.pos().0 as i64) > at.x)
                        };
                        let punching = tick < self.shadow_punch_until;
                        let name = format!("ct_body_{}{}{}", if punching { "punch" } else if moving { "run" } else { "idle" },
                            if punching { (tick / 10) % 2 } else if moving { (tick / 8) % 6 } else { 0 }, if left { "_l" } else { "" });
                        crate::fx_unit(sim, &name, owner, id, 10);
                    } else {
                        crate::fx_point(sim, "ct_shadow", owner, x as i64, y as i64, 12);
                    }
                    self.shadow_last = Some(at);
                    self.shadow_drawn = tick;
                }
            }
        }
    }
}

impl StablePassive for Contender {
    fn configure(&mut self, params: &str) {
        self.shadow_art = serde_json::from_str::<serde_json::Value>(params).ok()
            .and_then(|v| v.get("shadow_art_v1").and_then(|n| n.as_i64())) == Some(1);
    }
    fn clone_box(&self) -> Box<dyn StablePassive> {
        Box::new(self.clone())
    }
    fn on_spawn(&mut self, sim: &mut StableSim, _: usize, id: usize) {
        self.core.respawn();
        if let Some(s) = self.shadow.take() {
            sim.entity_set_hp(s, 0);
        }
        remove_prefix(sim, id, GO_PREFIX);
        remove_prefix(sim, id, ZONE_PREFIX);
        self.basic_hits.clear();
        self.in_skill = false;
        self.shadow_last = None;
        self.cyclone_drawn = None;
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
        if sim.get_entity(target).is_some_and(|e| !e.is_tower()) && self.basic_hits.len() < 16 {
            self.basic_hits.push(target);
        }
    }
    fn on_update(&mut self, sim: &mut StableSim, _: u64, _: usize, id: usize) {
        let f = self.frame(sim, id);
        let effects = self.core.step(&f);
        self.apply(sim, id, effects);
        if !f.alive {
            if let Some(s) = self.shadow.take() {
                sim.entity_set_hp(s, 0);
            }
        }
    }
}
