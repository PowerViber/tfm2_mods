//! Kicker's bridge: reads the sim into a `kicker_core::Frame` once per tick and applies the core's effects.
//! The engine walks him and swings his kick (the data basic attack); native code casts everything else (the data
//! slots are never-cast shells, like Akane's).
//!
//! * Walls: `kk_roof` (a named `ignore_wall` buff, 15% less damage, +25% speed) is renewed every tick he stands on a
//!   wall; every leap carries a short `kk_pass` (`ignore_wall`) so the glide may start or end on terrain. The core
//!   drops him to open ground whenever the engine moved him off a wall or a CC knocked him off.
//! * Kick and Skyfall: `crate::motion::glide` (a Knock); the Skyfall's carried target holds `kk_carry` (`ignore_wall`)
//!   for the dive. A V1 parry of the strike cancels everything that strike set off (Strike .. StrikeEnd).
//! * Bruised: `kk_bruised:<stacks>` (less healing, more damage taken per stack, its own HP regen cancelled).
//! * Second Wind: `kk_second_wind` (`undying`): lethal damage leaves him at 1 HP; the core sends him back to the tag
//!   (the one hand-set position here, allowlisted in `motion_guard_tests.rs`) and sets his HP.
//! * Flash + kick: `crate::flash::kit_blink` (the same blink and cooldown as everyone's Flash).
//! * The AI's walk (`Effect::Steer`) is a short `kk_go:x:y` buff that the shared input AI (`steve::WallAi`) turns into
//!   a move order for him only, and only when the engine wanted a walk.
use crate::kicker_core::{self as core, Core, Effect, Frame, Point, Unit};
use crate::kicker_data as d;
use mod_api_stable::{AttackTypeV1, BuffV1, CcKindV1, CcV1, StablePassive, StableSim};

pub(crate) const OWN_CC: &str = "kk_";
pub(crate) const GO_PREFIX: &str = "kk_go:";
pub(crate) const ROOF: &str = "kk_roof";
const PASS: &str = "kk_pass";
const CARRY: &str = "kk_carry";
const INVULN: &str = "kk_invuln";
const WIND: &str = "kk_second_wind";
const BRUISED: &str = "kk_bruised:";
const HOME: [(i64, i64); 2] = [(96_000, 864_000), (864_000, 96_000)];
const HARD: [CcKindV1; 5] = [CcKindV1::Stun, CcKindV1::Airborne, CcKindV1::Taunt, CcKindV1::Fear, CcKindV1::Charm];

fn named(kind: CcKindV1, ticks: u64, name: &str) -> CcV1 {
    let mut c = CcV1::of_kind(kind, ticks);
    c.set_name(name);
    c
}
/// A hard CC on `id` (his own `kk_` locks and self-moves never count).
fn hard_cc(sim: &StableSim, id: usize, own: bool) -> bool {
    sim.get_entity(id).is_some_and(|e| {
        (0..e.cc_count()).filter_map(|i| e.cc_at(i)).any(|c| {
            !(own && c.name().starts_with(OWN_CC)) && !crate::motion::is_self_motion(&c) && HARD.iter().any(|k| k.code() == c.kind)
        })
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

/// The move a steering buff asks for (read by the shared input AI).
pub fn steer_of(name: &str) -> Option<(i64, i64)> {
    let mut it = name.strip_prefix(GO_PREFIX)?.split(':').filter_map(|s| s.parse::<i64>().ok());
    Some((it.next()?, it.next()?))
}

/// The roof buff: through walls while he stands on them, faster, and less damage taken.
pub(crate) fn roof_buff() -> BuffV1 {
    let mut b = crate::timed(ROOF, 3);
    b.ignore_wall = true;
    b.move_speed_mult = d::ROOF_SPEED_PCT as i32;
    b.damaged_reduce = d::ROOF_REDUCE_PCT.max(0) as usize;
    b
}

#[derive(Clone, Default)]
pub struct Kicker {
    pub core: Core,
    profile: Option<crate::athlete_ai_core::Profile>,
    terrain: Option<(u64, Vec<bool>)>,
    basic_hits: Vec<usize>,
    in_skill: bool,
    fx_tokens: usize,
    fx_tick: u64,
    art_v1: bool,
}

impl Kicker {
    fn grid(&mut self, tick: u64) -> Vec<bool> {
        if let Some((t, g)) = &self.terrain {
            if tick < t + 600 {
                return g.clone();
            }
        }
        let n = core::CELLS;
        let c = core::CELL;
        let g: Vec<bool> = (0..n * n).map(|i| crate::walls::wall_at(i % n * c + c / 2, i / n * c + c / 2)).collect();
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
        let has_buff = |id: usize, name: &str| sim.get_entity(id).is_some_and(|e| (0..e.buff_count()).filter_map(|i| e.buff_at(i)).any(|b| b.name() == name));
        let profile = crate::athlete_ai::pinned_owner(sim, owner, &mut self.profile);
        let mut values = [50; 13];
        values[..9].copy_from_slice(&profile.values);
        values[9..].copy_from_slice(&profile.sense.0);
        let farming = crate::farming::Context::new(sim, owner);
        let snapshot = crate::champions_shared(sim);
        let realms = crate::realm::realms(sim);
        let mut units = vec![];
        for e in (0..sim.entity_count()).filter_map(|i| sim.entity_at(i)) {
            if e.id() == owner || !e.is_alive() {
                continue;
            }
            let e_team = e.team();
            let visible = e_team == team || sim.is_visible(team, e.id());
            let p = e.pos();
            let h = e.hp();
            let buffs = crate::entity_buffs(&snapshot, &e);
            // a cage wall or another Main Event realm between them: nothing of his reaches it
            let reach = crate::realm::can_interact(&realms, owner, e.id()) && crate::grimhald_cage::can_interact(&snapshot, owner, e.id());
            let targetable = e.is_targetable() && reach;
            let kind = e.kind();
            let champion = e.is_champion();
            units.push(Unit {
                id: e.id(),
                team: e_team,
                pos: if visible { Point::new(p.0 as i64, p.1 as i64) } else { Point::default() },
                hp: h.0 as i64,
                max_hp: h.1.max(1) as i64,
                radius: e.radius().max(1000) as i64,
                attack: e.stat().attack as i64,
                champion,
                tower: e.is_tower(),
                visible,
                targetable,
                farmable: farming.calm()
                    && e_team != team
                    && targetable
                    && visible
                    && !champion
                    && farming.eligible_known(e.id(), e_team, kind, p, kind.is_none() && e.is_minion(), &buffs),
                stunned: champion && hard_cc(sim, e.id(), false),
                parry: champion && buffs.iter().any(|b| b.name() == "v1_guard") && !buffs.iter().any(|b| b.name() == "v1_parry_cd"),
            });
        }
        let ult_unlocked = has_buff(owner, crate::ULT_LEARNED);
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
            stunned: hard_cc(sim, owner, true),
            ult_unlocked,
            flash_ready: !has_buff(owner, "flash_cd"),
            home: Point::new(home.0, home.1),
            units,
            terrain: self.grid(tick),
            profile: values,
            basic_hits: std::mem::take(&mut self.basic_hits),
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
        let not_tower = |sim: &StableSim, id: usize| sim.get_entity(id).is_some_and(|e| !e.is_tower());
        // a parried strike cancels what it set off (Strike .. StrikeEnd)
        let mut parried: Vec<usize> = vec![];
        let mut strike: Option<usize> = None;
        for e in effects {
            if let Some(t) = strike {
                if parried.contains(&t) {
                    if e == Effect::StrikeEnd {
                        strike = None;
                    }
                    continue;
                }
            }
            match e {
                Effect::Strike { target } => strike = Some(target),
                Effect::StrikeEnd => strike = None,
                Effect::Damage { target, amount, direct } => {
                    if !reach(owner, target) || !not_tower(sim, target) {
                        continue;
                    }
                    if direct {
                        if let Some(c) = all.iter().find(|c| c.id == target) {
                            if crate::batch2::try_parry(sim, c, owner, amount.max(0) as usize) {
                                parried.push(target);
                                continue;
                            }
                        }
                    }
                    self.in_skill = true;
                    sim.deal_damage(owner, target, amount.max(0) as usize, 0, AttackTypeV1::Skill);
                    self.in_skill = false;
                }
                Effect::KnockUp { target, ticks } => {
                    if reach(owner, target) && not_tower(sim, target) {
                        sim.apply_cc(target, &named(CcKindV1::Airborne, ticks, "kk_knockup"));
                    }
                }
                Effect::Stun { target, ticks } => {
                    if reach(owner, target) && not_tower(sim, target) {
                        sim.apply_cc(target, &named(CcKindV1::Stun, ticks, "kk_stun"));
                    }
                }
                Effect::Knock { target, to, ticks } => {
                    if reach(owner, target) && not_tower(sim, target) {
                        crate::motion::glide(sim, target, (to.x, to.y), ticks, crate::motion::Kind::Knock);
                    }
                }
                Effect::Carry { target, to, ticks } => {
                    if reach(owner, target) && not_tower(sim, target) {
                        let mut b = crate::timed(CARRY, ticks as usize + 2);
                        b.ignore_wall = true;
                        replace_buff(sim, target, &b);
                        crate::motion::glide(sim, target, (to.x, to.y), ticks, crate::motion::Kind::Knock);
                    }
                }
                Effect::Bruise { target, stacks, ticks } => {
                    if !not_tower(sim, target) {
                        continue;
                    }
                    let regen = sim.get_entity(target).map_or(0, |e| e.stat().hp_regen as i32);
                    remove_prefix(sim, target, BRUISED);
                    let mut b = crate::timed(&format!("{BRUISED}{stacks}"), ticks as usize);
                    b.heal_reduce = d::BRUISED_HEAL_REDUCE_PCT.max(0) as usize;
                    b.damaged_amplify = (stacks * d::BRUISED_AMP_PER_STACK).max(0) as usize;
                    b.hp_regen = -regen;
                    sim.add_buff(target, &b);
                }
                Effect::BruiseEnd { target } => remove_prefix(sim, target, BRUISED),
                Effect::Mark { target, ticks } => {
                    sim.entity_remove_buff(target, "kk_mark");
                    if ticks > 0 {
                        sim.add_buff(target, &crate::timed("kk_mark", ticks as usize));
                    }
                }
                Effect::Roof { on } => {
                    if on {
                        replace_buff(sim, owner, &roof_buff());
                    } else {
                        sim.entity_remove_buff(owner, ROOF);
                    }
                }
                Effect::Hide => {
                    sim.entity_set_invisible(owner, 3);
                }
                Effect::Shield { amount, ticks } => {
                    sim.entity_add_shield(owner, amount.max(0) as usize, ticks as usize);
                }
                Effect::Invulnerable { ticks } => {
                    // damage only: nothing of his is CC immune
                    let mut b = crate::timed(INVULN, ticks as usize);
                    b.damaged_reduce = 100;
                    b.base_attack_damaged_reduce = 100;
                    b.skill_damaged_reduce = 100;
                    replace_buff(sim, owner, &b);
                }
                Effect::Glide { to, ticks } => {
                    crate::motion::glide(sim, owner, (to.x, to.y), ticks, crate::motion::Kind::SelfMove);
                }
                Effect::PassWalls { ticks } => {
                    let mut b = crate::timed(PASS, ticks as usize);
                    b.ignore_wall = true;
                    replace_buff(sim, owner, &b);
                }
                Effect::Flash { to } => {
                    if !crate::flash::kit_blink(sim, owner, (to.x, to.y)) {
                        // no Flash after all: the kick plays from where he stands
                        self.core.kick = None;
                    }
                }
                Effect::Undying { ticks } => {
                    let mut b = crate::timed(WIND, ticks as usize);
                    b.undying = true;
                    replace_buff(sim, owner, &b);
                }
                Effect::UndyingEnd => {
                    sim.entity_remove_buff(owner, WIND);
                }
                Effect::Return { to, hp } => {
                    // Second Wind: back to the tag (a teleport by design)
                    let (x, y) = (to.x.max(0), to.y.max(0));
                    sim.entity_set_pos(owner, x as u64, y as u64);
                    crate::steve::note_placed(sim, owner, x, y);
                    sim.add_buff(owner, &crate::timed(crate::place::MOVED, 2));
                    sim.entity_set_hp(owner, hp.max(1) as usize);
                }
                Effect::Steer { to } => {
                    remove_prefix(sim, owner, GO_PREFIX);
                    sim.add_buff(owner, &crate::timed(&format!("{GO_PREFIX}{}:{}", to.x, to.y), 6));
                }
                Effect::Fx { name, at, ticks, ambient } => {
                    if self.fx_ok(tick as u64, ambient) {
                        crate::fx_point(sim, &name, owner, at.x, at.y, ticks);
                    }
                }
                Effect::Pose { name, ticks } => {
                    // his own poses arrive with Codex's body; the placeholder body has none
                    if self.art_v1 {
                        sim.apply_cc(owner, &named(CcKindV1::Animation, ticks, &format!("kk_{name}")));
                    }
                }
                Effect::Log(_) => {}
            }
        }
    }
}

impl StablePassive for Kicker {
    fn configure(&mut self, params: &str) {
        self.art_v1 = serde_json::from_str::<serde_json::Value>(params).ok().and_then(|v| v.get("art_v1").and_then(|n| n.as_i64())).is_some_and(|n| n == 1);
    }
    fn clone_box(&self) -> Box<dyn StablePassive> {
        Box::new(self.clone())
    }
    fn on_spawn(&mut self, sim: &mut StableSim, _: usize, id: usize) {
        self.core.respawn(sim.tick() as u64);
        for b in [ROOF, PASS, INVULN, WIND] {
            sim.entity_remove_buff(id, b);
        }
        remove_prefix(sim, id, GO_PREFIX);
        self.basic_hits.clear();
        self.in_skill = false;
    }
    /// His hits never cross a cage or a Main Event ring; accepted basic attacks mark and bruise (in the core) and carry
    /// the Drop-in bonus.
    fn on_attack(&mut self, sim: &mut StableSim, _: usize, id: usize, target: usize, damage: &mut usize) {
        let all = crate::champions_shared(sim);
        let realms = crate::realm::realms(sim);
        if !crate::grimhald_cage::can_interact(&all, id, target) || !crate::realm::can_interact(&realms, id, target) {
            *damage = 0;
            return;
        }
        if !self.in_skill {
            let mult = self.core.basic_multiplier(sim.tick() as u64);
            if mult != 100 {
                *damage = (*damage as i64 * mult / 100).max(0) as usize;
                self.core.take_bonus();
            }
            if !self.basic_hits.contains(&target) {
                self.basic_hits.push(target);
            }
        }
    }
    fn on_update(&mut self, sim: &mut StableSim, _: u64, _: usize, id: usize) {
        let f = self.frame(sim, id);
        if !f.alive {
            let _ = self.core.step(&f);
            sim.entity_remove_buff(id, ROOF);
            return;
        }
        let effects = self.core.step(&f);
        self.apply(sim, id, effects);
        // on a wall: the roof buff is renewed a few ticks at a time, so leaving it ends it at once
        if self.core.roof {
            replace_buff(sim, id, &roof_buff());
        }
    }
}
