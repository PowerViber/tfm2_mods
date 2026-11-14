//! Kagami's bridge: reads the sim into a `kagami_core::Frame` once per tick and applies the core's effects. The engine
//! walks her and fires her shard (the data basic attack); native code casts everything else (the data slots are
//! never-cast shells, like Akane's and Kicker's).
//!
//! * The clone: a summoned unit (`spawn_unit`, a base body until Codex's art) at 35% of her HP with her Defence. It
//!   never walks or swings by itself (move speed 1, attack and skills blocked); the core glides it
//!   (`crate::motion::glide`) and deals its shards and casts. It carries `kg_clone:<owner>` so the frame finds it.
//! * Swaps: the core locks on to two ids for 0.5 s (nobody is rooted), then `Effect::Swap` trades the places they
//!   hold at that moment: the two hand-set positions here (a teleport by design, allowlisted in
//!   `motion_guard_tests.rs`), each through `place::safe`.
//! * The ult's ward is `kg_ward` and her own Mirror Swap ward is `kg_self` (`undying`, renewed a few ticks at a time
//!   while they last): a burst can't skip the save. The seal is the engine's banish (untargetable, unseen,
//!   input-locked) plus `kg_sealed`, so every Kagami's frame sees it.
//! * The ult's revive: full HP, `kg_revived` (Kindred Charm's stats, +30% Defence, `cc_immune`) and a shield.
//! * The AI's walk (`Effect::Steer`) is a short `kg_go:x:y` buff that the shared input AI (`steve::WallAi`) turns into
//!   a move order for her only, and only when the engine wanted a walk.
use crate::kagami_core::{CloneView, Core, Effect, Frame, Point, Unit};
use crate::kagami_data as d;
use mod_api_stable::{AttackTypeV1, BuffV1, CcKindV1, CcV1, StablePassive, StableSim, StatV1, UnitAttackV1};

pub(crate) const GO_PREFIX: &str = "kg_go:";
pub(crate) const CLONE_TAG: &str = "kg_clone:";
pub(crate) const WARD: &str = "kg_ward";
pub(crate) const SELF_WARD: &str = "kg_self";
pub(crate) const SEALED: &str = "kg_sealed";
pub(crate) const CHARM: &str = "kg_charm";
pub(crate) const REVIVED: &str = "kg_revived";
/// the clone's placeholder body (a base champion's sprite) until Codex draws hers
pub(crate) const CLONE_BODY: &str = "priest";
/// a clone has no time limit: a summon lasting a whole long match
const FOREVER: u64 = 60 * 60 * 90;
const HARD: [CcKindV1; 5] = [CcKindV1::Stun, CcKindV1::Airborne, CcKindV1::Taunt, CcKindV1::Fear, CcKindV1::Charm];

fn named(kind: CcKindV1, ticks: u64, name: &str) -> CcV1 {
    let mut c = CcV1::of_kind(kind, ticks);
    c.set_name(name);
    c
}
fn hard_cc(sim: &StableSim, id: usize) -> bool {
    sim.get_entity(id).is_some_and(|e| {
        (0..e.cc_count()).filter_map(|i| e.cc_at(i)).any(|c| !crate::motion::is_self_motion(&c) && HARD.iter().any(|k| k.code() == c.kind))
    })
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
fn has_buff(sim: &StableSim, id: usize, name: &str) -> bool {
    sim.get_entity(id).is_some_and(|e| (0..e.buff_count()).filter_map(|i| e.buff_at(i)).any(|b| b.name() == name))
}

/// The move a steering buff asks for (read by the shared input AI).
pub fn steer_of(name: &str) -> Option<(i64, i64)> {
    let mut it = name.strip_prefix(GO_PREFIX)?.split(':').filter_map(|s| s.parse::<i64>().ok());
    Some((it.next()?, it.next()?))
}

/// Kindred Charm's stats (also part of the ult's revive).
pub(crate) fn charm_stats(b: &mut BuffV1) {
    b.attack_speed_mult += d::CHARM_ATTACK_SPEED_PCT as i32;
    b.move_speed_mult += d::CHARM_MOVE_SPEED_PCT as i32;
    b.attack_mult += d::CHARM_DAMAGE_PCT as i32;
    b.magic_power_mult += d::CHARM_DAMAGE_PCT as i32;
}
pub(crate) fn charm_buff(ticks: u64) -> BuffV1 {
    let mut b = crate::timed(CHARM, ticks as usize);
    charm_stats(&mut b);
    b
}
pub(crate) fn revived_buff() -> BuffV1 {
    let mut b = crate::timed(REVIVED, d::ULT_REVIVE_TICKS.max(1) as usize);
    charm_stats(&mut b);
    b.defence_mult = d::ULT_DEFENCE_PCT as i32;
    b.cc_immune = true;
    b
}
fn ward_buff(name: &str, ticks: u64) -> BuffV1 {
    let mut b = crate::timed(name, ticks as usize);
    b.undying = true;
    b
}

#[derive(Clone, Default)]
pub struct Kagami {
    pub core: Core,
    profile: Option<crate::athlete_ai_core::Profile>,
    in_skill: bool,
    fx_tokens: usize,
    fx_tick: u64,
}

impl Kagami {
    /// Her live clone: the summon tagged with her id (the known one first; a full search only when it's gone).
    pub(crate) fn find_clone(sim: &StableSim, owner: usize, known: Option<usize>) -> Option<CloneView> {
        let tag = format!("{CLONE_TAG}{owner}");
        let known = known.and_then(|id| sim.get_entity(id)).filter(|e| e.is_alive() && has_buff(sim, e.id(), &tag));
        known.or_else(|| (0..sim.entity_count()).filter_map(|i| sim.entity_at(i)).find(|e| e.is_alive() && !e.is_champion() && has_buff(sim, e.id(), &tag))).map(|e| {
            let p = e.pos();
            let h = e.hp();
            CloneView { id: e.id(), pos: Point::new(p.0 as i64, p.1 as i64), hp: h.0 as i64, max_hp: h.1.max(1) as i64 }
        })
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
        // the full search only after a death, every 30 ticks (it is far dearer than the rest of the frame)
        let known = self.core.clone;
        let clone = if known.is_some() || tick % 30 == 0 || tick >= self.core.clone_back_at { Self::find_clone(sim, owner, known) } else { None };
        let snapshot = crate::champions_shared(sim);
        let realms = crate::realm::realms(sim);
        let mut units = vec![];
        for e in (0..sim.entity_count()).filter_map(|i| sim.entity_at(i)) {
            if e.id() == owner || !e.is_alive() || clone.is_some_and(|c| c.id == e.id()) {
                continue;
            }
            let e_team = e.team();
            let visible = e_team == team || sim.is_visible(team, e.id());
            let p = e.pos();
            let h = e.hp();
            let buffs = crate::entity_buffs(&snapshot, &e);
            let reach = crate::realm::can_interact(&realms, owner, e.id()) && crate::grimhald_cage::can_interact(&snapshot, owner, e.id());
            let kind = e.kind();
            units.push(Unit {
                id: e.id(),
                team: e_team,
                pos: if visible { Point::new(p.0 as i64, p.1 as i64) } else { Point::default() },
                hp: h.0 as i64,
                max_hp: h.1.max(1) as i64,
                attack: e.stat().attack as i64,
                champion: e.is_champion(),
                tower: e.is_tower() || kind == Some(mod_api_stable::EntityKindV1::Nexus),
                visible,
                targetable: e.is_targetable() && reach,
                sealed: buffs.iter().any(|b| b.name() == SEALED),
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
            ult_unlocked: has_buff(sim, owner, crate::ULT_LEARNED),
            home: Point::new(home.0, home.1),
            units,
            clone,
            profile: values,
            manual: false,
            order: None,
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

    fn spawn_clone(sim: &mut StableSim, owner: usize, at: Point, hp: i64) -> Option<usize> {
        let me = sim.get_entity(owner)?;
        let (team, st) = (me.team(), me.stat());
        let stat = StatV1 {
            hp: hp.max(1) as usize,
            defence: st.defence,
            magic_resistance: st.magic_resistance,
            move_speed: 1,
            ..StatV1::default()
        };
        let attack = UnitAttackV1 { attack: 0, attack_ratio: 0, range: 1, cooltime: 5_184_000, duration: 1, start_timing: 0, ..UnitAttackV1::default() };
        let id = sim.spawn_unit(CLONE_BODY, owner, team, at.x.max(0) as u64, at.y.max(0) as u64, FOREVER, &stat, &attack)?;
        // no Bind: it would hold the clone against its own glides
        for k in [CcKindV1::BlockAttack, CcKindV1::BlockSkill] {
            sim.apply_cc(id, &named(k, FOREVER, "kg_clone_hold"));
        }
        sim.add_buff(id, &crate::timed(&format!("{CLONE_TAG}{owner}"), FOREVER as usize));
        Some(id)
    }

    pub(crate) fn apply(&mut self, sim: &mut StableSim, owner: usize, effects: Vec<Effect>) {
        let tick = sim.tick();
        let all = crate::champions_shared(sim);
        let realms = crate::realm::realms(sim);
        let reach = |a: usize, b: usize| crate::realm::can_interact(&realms, a, b) && crate::grimhald_cage::can_interact(&all, a, b);
        let not_tower = |sim: &StableSim, id: usize| sim.get_entity(id).is_some_and(|e| !e.is_tower() && e.kind() != Some(mod_api_stable::EntityKindV1::Nexus));
        let pos_of = |sim: &StableSim, id: usize| sim.get_entity(id).filter(|e| e.is_alive()).map(|e| e.pos()).map(|p| (p.0 as i64, p.1 as i64));
        for e in effects {
            match e {
                Effect::SpawnClone { at, hp } => {
                    match Self::spawn_clone(sim, owner, at, hp) {
                        // the next frame looks it up directly
                        Some(id) => self.core.clone = Some(id),
                        None => crate::mod_log(sim, "kagami_log.txt", &format!("clone{tick}"), "reflection: the host refused the clone unit"),
                    }
                }
                Effect::CloneMove { to, ticks } => {
                    if let Some(c) = self.core.clone {
                        crate::motion::glide(sim, c, (to.x, to.y), ticks, crate::motion::Kind::SelfMove);
                    }
                }
                Effect::CloneShot { target, amount, from } => {
                    if !reach(owner, target) || !not_tower(sim, target) {
                        continue;
                    }
                    self.in_skill = true;
                    sim.deal_damage(owner, target, amount.max(0) as usize, 0, AttackTypeV1::Skill);
                    self.in_skill = false;
                    if let Some(p) = pos_of(sim, target) {
                        if self.fx_ok(tick as u64) {
                            crate::fx_point(sim, "kg_shard_hit", owner, p.0, p.1, 12);
                        }
                    }
                    let _ = from;
                }
                Effect::Heal { target, amount, from } => {
                    sim.heal(owner, target, amount.max(0) as usize);
                    if let Some(p) = pos_of(sim, target) {
                        if self.fx_ok(tick as u64) {
                            crate::fx_point(sim, "kg_heal", owner, p.0, p.1, 30);
                        }
                    }
                    let _ = from;
                }
                Effect::Charm { target, ticks } => {
                    replace_buff(sim, target, &charm_buff(ticks));
                }
                Effect::SwapLock { a, b, ticks } => {
                    if let (Some(pa), Some(pb)) = (pos_of(sim, a), pos_of(sim, b)) {
                        if self.fx_ok(tick as u64) {
                            crate::fx_point(sim, "kg_thread", owner, (pa.0 + pb.0) / 2, (pa.1 + pb.1) / 2, ticks.max(1));
                        }
                    }
                }
                Effect::Swap { a, b } => {
                    // a deliberate swap through the mirror (a teleport by design): each lands where the other stands now
                    let (Some(pa), Some(pb)) = (pos_of(sim, a), pos_of(sim, b)) else { continue };
                    let to_a = crate::place::safe(&all, tick, pa, pb);
                    sim.entity_set_pos(a, to_a.0.max(0) as u64, to_a.1.max(0) as u64);
                    let to_b = crate::place::safe(&all, tick, pb, pa);
                    sim.entity_set_pos(b, to_b.0.max(0) as u64, to_b.1.max(0) as u64);
                    for (id, p) in [(a, to_a), (b, to_b)] {
                        crate::steve::note_placed(sim, id, p.0, p.1);
                        sim.add_buff(id, &crate::timed(crate::place::MOVED, 2));
                    }
                }
                Effect::Stun { target, ticks } => {
                    if reach(owner, target) && not_tower(sim, target) {
                        sim.apply_cc(target, &named(CcKindV1::Stun, ticks, "kg_stun"));
                    }
                }
                Effect::Knockback { target } => {
                    if reach(owner, target) && not_tower(sim, target) {
                        sim.entity_knockback(owner, target, d::ANGEL_KNOCK_SPEED.max(0) as usize, d::ANGEL_KNOCK_TICKS.max(1) as usize);
                    }
                }
                Effect::Shield { target, amount, ticks } => {
                    sim.entity_add_shield(target, amount.max(0) as usize, ticks as usize);
                }
                Effect::Ward { target, ticks } => replace_buff(sim, target, &ward_buff(WARD, ticks)),
                Effect::SelfWard { ticks } => replace_buff(sim, owner, &ward_buff(SELF_WARD, ticks)),
                Effect::Seal { target, ticks } => {
                    sim.entity_banish(owner, target, ticks as usize, "", "");
                    replace_buff(sim, target, &crate::timed(SEALED, ticks as usize));
                }
                Effect::Revive { target, hp, blessed } => {
                    sim.entity_remove_buff(target, SEALED);
                    let now = sim.get_entity(target).map_or(0, |e| e.hp().0 as i64);
                    if hp > now {
                        sim.entity_set_hp(target, hp as usize);
                    }
                    if blessed {
                        replace_buff(sim, target, &revived_buff());
                        let max = sim.get_entity(target).map_or(0, |e| e.hp().1 as i64);
                        sim.entity_add_shield(target, (max * d::ULT_SHIELD_PCT_MAX_HP / 100).max(0) as usize, d::ULT_REVIVE_TICKS.max(1) as usize);
                    }
                }
                Effect::Steer { to } => {
                    remove_prefix(sim, owner, GO_PREFIX);
                    sim.add_buff(owner, &crate::timed(&format!("{GO_PREFIX}{}:{}", to.x, to.y), 6));
                }
                Effect::Fx { name, at, ticks } => {
                    if self.fx_ok(tick as u64) {
                        crate::fx_point(sim, &name, owner, at.x, at.y, ticks);
                    }
                }
                Effect::Log { .. } => {}
            }
        }
    }
}

impl StablePassive for Kagami {
    fn configure(&mut self, _: &str) {}
    fn clone_box(&self) -> Box<dyn StablePassive> {
        Box::new(self.clone())
    }
    fn on_spawn(&mut self, sim: &mut StableSim, _: usize, id: usize) {
        self.core.respawn(sim.tick() as u64);
        for b in [SELF_WARD, SEALED] {
            sim.entity_remove_buff(id, b);
        }
        remove_prefix(sim, id, GO_PREFIX);
        self.in_skill = false;
    }
    /// Her shards never cross a cage or a Main Event ring.
    fn on_attack(&mut self, sim: &mut StableSim, _: usize, id: usize, target: usize, damage: &mut usize) {
        let all = crate::champions_shared(sim);
        let realms = crate::realm::realms(sim);
        if !crate::grimhald_cage::can_interact(&all, id, target) || !crate::realm::can_interact(&realms, id, target) {
            *damage = 0;
        }
    }
    fn on_update(&mut self, sim: &mut StableSim, _: u64, _: usize, id: usize) {
        let f = self.frame(sim, id);
        let effects = self.core.step(&f);
        self.apply(sim, id, effects);
    }
}
