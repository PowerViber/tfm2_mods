//! Native adapter for the deterministic Contender combat core.
use crate::contender_shadow_core::{Barrier, Cast, Caster, Core, Effect, Frame, Grab, HitSource, IncomingHit, Order, Point, Unit, CELL, CELLS};
use crate::contender_shadow_data as d;
use mod_api_stable::{AttackTypeV1, BuffV1, CcKindV1, CcV1, InputTargetV1, StableEffectType, StablePassive, StableSim, StatV1};
use std::cell::Cell;

pub(crate) const GO_PREFIX: &str = "ct_go:";
const ZONE_PREFIX: &str = "ctw:";
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
/// Clockwise eighth-turns from east (screen y down) from `from` toward `to`, as the baked grab art.
pub(crate) fn grab_heading(from: Point, to: Point) -> u8 {
    let (dx, dy) = ((to.x - from.x) as f64, (to.y - from.y) as f64);
    if dx == 0.0 && dy == 0.0 {
        return 0;
    }
    ((dy.atan2(dx).to_degrees() / 45.0).round() as i64).rem_euclid(8) as u8
}

/// Where the reaching hand sits: 14 px ahead of the flying shadow and 3 px up (Codex's review page).
pub(crate) fn grab_hand_at(from: Point, at: Point, cursor: Point) -> (i64, i64) {
    let ahead = cursor.toward(at, 14_000);
    let ahead = if ahead == cursor { cursor.toward(Point::new(cursor.x + (at.x - from.x).signum(), cursor.y), 14_000) } else { ahead };
    (ahead.x, ahead.y - 3_000)
}

/// The baked pull ribbon for Contender at `me` and the target at `them`: the length bucket closest to their distance
/// (pullStop..range in 10,000 steps, as tools/build-contender-grab-art.py), the heading toward the target, and the
/// point to draw it at (the middle of the ribbon, which starts 10 px in front of him, 3 px up).
pub(crate) fn grab_ribbon(me: Point, them: Point) -> (String, Point) {
    const STEP: i64 = 10_000;
    let lo = d::SHADOW_GRAB_PULL_STOP / STEP;
    let hi = (d::SHADOW_GRAB_RANGE + STEP - 1) / STEP;
    let gap = me.dist(them).round() as i64;
    let bucket = ((gap + STEP / 2) / STEP).clamp(lo, hi);
    let start = me.toward(them, 10_000);
    let mid = Point::new((start.x + them.x) / 2, (start.y + them.y) / 2 - 3_000);
    (format!("ct_grab_ribbon_L{bucket}_d{}", grab_heading(me, them)), mid)
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
/// contender-4.0: the game's own AI presses S1 (Shadow Grab), S2 (Shadow Anchor) and the ult (Clone Army). Each engine
/// slot's `Native` effect only drops a short marker with the aim (`ct_cast:<slot>:<x>:<y>:<tick>`); his passive consumes
/// it on its next update and runs the core's order. S3 Smother has no engine slot: the core casts it itself.
pub const CAST_PREFIX: &str = "ct_cast:";

fn cast_point(sim: &StableSim, input: InputTargetV1) -> Option<(i64, i64)> {
    match input.kind {
        1 => sim.get_entity(input.target_id).map(|e| { let p = e.pos(); (p.0 as i64, p.1 as i64) }),
        3 => Some((input.x as i64, input.y as i64)),
        _ => None,
    }
}

fn mark(sim: &mut StableSim, caster: usize, slot: &str, input: InputTargetV1) {
    if let Some((x, y)) = cast_point(sim, input) {
        let tick = sim.tick();
        sim.add_buff(caster, &crate::timed(&format!("{CAST_PREFIX}{slot}:{x}:{y}:{tick}"), 5));
    }
}

/// S1 Shadow Grab, cast by the engine (the type keeps its old name: it is registered as `contender_s1`).
pub struct CastMerge;
impl StableEffectType for CastMerge {
    fn apply(&self, sim: &mut StableSim, _: u64, caster: usize, input: InputTargetV1) {
        mark(sim, caster, "s1", input);
    }
    fn expected_damage(&self, s: &StatV1) -> (usize, usize) {
        (s.attack, 0)
    }
}

/// S2 Shadow Anchor, cast by the engine (`contender_s2`): drop, or teleport to the anchor.
pub struct CastFlurry;
impl StableEffectType for CastFlurry {
    fn apply(&self, sim: &mut StableSim, _: u64, caster: usize, input: InputTargetV1) {
        mark(sim, caster, "s2", input);
    }
    fn expected_damage(&self, s: &StatV1) -> (usize, usize) {
        (s.attack, 0)
    }
}

/// Ult Clone Army, cast by the engine (`contender_ult`).
pub struct CastLink;
impl StableEffectType for CastLink {
    fn apply(&self, sim: &mut StableSim, _: u64, caster: usize, input: InputTargetV1) {
        mark(sim, caster, "ult", input);
    }
    fn expected_damage(&self, s: &StatV1) -> (usize, usize) {
        (s.attack, 0)
    }
}

/// The order a cast marker asks for.
pub fn cast_order(name: &str) -> Option<Order> {
    let rest = name.strip_prefix(CAST_PREFIX)?;
    let mut it = rest.split(':');
    let slot = it.next()?;
    let x = it.next()?.parse().ok()?;
    let y = it.next()?.parse().ok()?;
    let aim = Point::new(x, y);
    match slot {
        "s1" => Some(Order::S1 { aim }),
        "s2" => Some(Order::S2 { aim }),
        "ult" => Some(Order::Ult { aim }),
        _ => None,
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
    shadow_art: bool,
    shadow_directions: bool,
    rhythm_art: bool,
    rhythm_presenter: crate::contender_rhythm::Presenter,
    art_drawn_at: u64,
    receipt_basic: bool,
    landed_hits: u16,
    /// enemies his basic attacks landed on since the last frame
    basic_hits: Vec<usize>,
    in_skill: bool,
    fx_tokens: usize,
    fx_tick: u64,
}

/// Round 192: the baked Smother trail from the line's origin to its front: the 10,000 length bucket nearest the
/// distance so far (as tools/build-contender-40-art.py), the heading, the shadow's paler variant, and its midpoint.
/// None before the line has moved.
pub(crate) fn smother_trail(from: Point, front: Point, shadow: bool) -> Option<(String, Point)> {
    const STEP: i64 = 10_000;
    let gap = from.dist(front).round() as i64;
    if gap < STEP / 2 {
        return None;
    }
    let bucket = ((gap + STEP / 2) / STEP).clamp(1, (d::SMOTHER_RANGE + STEP - 1) / STEP);
    let mid = Point::new((from.x + front.x) / 2, (from.y + front.y) / 2);
    Some((format!("ct_40_trail{}_L{bucket}_d{}", if shadow { "_s" } else { "" }, grab_heading(from, front)), mid))
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
        let barriers = crate::steve::walls_up(&snapshot, tick as usize).into_iter().map(|wall| {
            let (bx, by) = wall.built_end(tick as usize);
            Barrier {
                from: Point::new(wall.ax, wall.ay), to: Point::new(bx, by), half_width: crate::steve::WALL_HALF,
            }
        }).collect();
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
            if e_team != team {
                // never grabbed, feared, silenced or knocked up: structures and CC-immune units
                use mod_api_stable::EntityKindV1 as K;
                let eligible = champion || matches!(kind, Some(K::Minion | K::Jungle)) || (kind.is_none() && e.is_minion());
                let immune = (0..e.buff_count()).filter_map(|i| e.buff_at(i)).any(|b| b.cc_immune);
                if !eligible || immune { link_excluded.push(e.id()); }
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
            separated,
            barriers,
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
                Effect::Drag { target, to } => {
                    // the grab drags straight through walls: placed each tick, like motion's legacy mode
                    if reach(owner, target) && not_tower(sim, target) {
                        place(sim, target, to);
                    }
                }
                Effect::Blink { to } => place(sim, owner, to),
                Effect::Hold { target, ticks } => {
                    if reach(owner, target) && not_tower(sim, target) {
                        sim.apply_cc(target, &named(CcKindV1::Stun, ticks, &format!("ct_grab_hold_{owner}")));
                    }
                }
                Effect::Airborne { target, ticks } => {
                    if reach(owner, target) && not_tower(sim, target) {
                        sim.apply_cc(target, &named(CcKindV1::Airborne, ticks, &format!("ct_air_{owner}")));
                    }
                }
                Effect::Fear { target, ticks } => {
                    if reach(owner, target) && not_tower(sim, target) {
                        sim.apply_cc(target, &named(CcKindV1::Fear, ticks, &format!("ct_fear_{owner}")));
                    }
                }
                Effect::Silence { target, ticks } => {
                    if reach(owner, target) && not_tower(sim, target) {
                        sim.apply_cc(target, &named(CcKindV1::BlockSkill, ticks, &format!("ct_silence_{owner}")));
                    }
                }
                Effect::Slow { target, pct, ticks } => {
                    if reach(owner, target) && not_tower(sim, target) {
                        let mut b = crate::timed("ct_smother_slow", ticks as usize);
                        b.move_speed_mult = -(pct as i32);
                        replace_buff(sim, target, &b);
                    }
                }
                Effect::Smothered { target, armour_pct, attack_speed_pct, ticks } => {
                    if reach(owner, target) && not_tower(sim, target) {
                        let mut b = crate::timed("ct_smothered", ticks as usize);
                        b.defence_mult = -(armour_pct as i32);
                        b.attack_speed_mult = -(attack_speed_pct as i32);
                        replace_buff(sim, target, &b);
                    }
                }
                Effect::Haste { attack_speed_pct, move_pct, ticks } => {
                    let mut b = crate::timed("ct_haste", ticks as usize);
                    b.attack_speed_mult = attack_speed_pct as i32;
                    b.move_speed_mult = move_pct as i32;
                    replace_buff(sim, owner, &b);
                }
                Effect::Shield { amount, ticks } => {
                    sim.entity_add_shield(owner, amount.max(0) as usize, ticks as usize);
                }
                Effect::Glide { to, ticks } => {
                    crate::motion::glide(sim, owner, (to.x, to.y), ticks, crate::motion::Kind::SelfMove);
                }
                Effect::Charges { count } => {
                    // the same 5 pips, each worth `CHARGES_PIP_SHADOWS` shadows (a pip shows while any of its shadows is left)
                    remove_prefix(sim, owner, "ct_charges:");
                    let pips = (count.max(0) + d::CHARGES_PIP_SHADOWS - 1) / d::CHARGES_PIP_SHADOWS;
                    sim.add_buff(owner, &BuffV1::named(&format!("ct_charges:{}", pips.clamp(0, d::CHARGES_MAX / d::CHARGES_PIP_SHADOWS))));
                }
                Effect::Fx { name, at, ticks, heading } => {
                    if self.fx_ok(tick) {
                        // the Shadow Grab art carries its own names (no `_t`/`_l` variants)
                        let visual = if name.starts_with("ct_grab_") || name.starts_with("ct_40_") {
                            name
                        } else if self.shadow_art && self.shadow_directions && name == "ct_flurry" && heading.is_some() {
                            format!("ct_flurry_d{}_t{ticks}", heading.unwrap_or(0) % 8)
                        } else if self.shadow_art {
                            let left = sim.get_entity(owner).is_some_and(|e| (e.pos().0 as i64) > at.x);
                            format!("{name}_t{ticks}{}", if left { "_l" } else { "" })
                        } else { name };
                        crate::fx_point(sim, &visual, owner, at.x, at.y, ticks);
                    }
                }
                Effect::Log { .. } => {}
            }
        }
        if self.shadow_art && tick >= self.art_drawn_at {
            self.draw_shadows(sim, owner, tick);
            self.art_drawn_at = tick + 2;
        }
    }

    /// His shadows' art: each standing shadow (the anchor, the clones), each flying grab with Codex's reaching hand,
    /// each pull ribbon between its caster and the target, and the shadow throwing a flurry.
    fn draw_shadows(&mut self, sim: &mut StableSim, owner: usize, tick: u64) {
        let me = sim.get_entity(owner).map(|e| { let p = e.pos(); Point::new(p.0 as i64, p.1 as i64) });
        let Some(me) = me else { return };
        let at_of = |core: &Core, c: Caster| match c {
            Caster::Me => Some(me),
            Caster::Shadow { id } => core.shadows.iter().find(|s| s.id == id).map(|s| s.pos),
        };
        let mut draws: Vec<(String, Point, u64)> = vec![];
        if tick % 4 == 0 && !self.core.shadows.is_empty() {
            // round 192 (Codex's art): clones fade in, then stand, facing the nearest enemy champion; the anchor keeps
            // the dark shadow body
            let team = sim.get_entity(owner).map(|e| e.team());
            let foes: Vec<Point> = (0..sim.entity_count()).filter_map(|i| sim.entity_at(i))
                .filter(|e| e.is_alive() && e.is_champion() && Some(e.team()) != team)
                .map(|e| { let p = e.pos(); Point::new(p.0 as i64, p.1 as i64) }).collect();
            for s in &self.core.shadows {
                if s.anchor {
                    draws.push(("ct_body_idle0".into(), s.pos, 4));
                    continue;
                }
                let toward = foes.iter().min_by_key(|p| p.d2(s.pos)).copied().unwrap_or(me);
                let left = if toward == me { s.pos.x > me.x } else { toward.x < s.pos.x };
                let born = s.until.saturating_sub(d::CLONE_ARMY_LIFE_TICKS as u64);
                let fading = tick < born + crate::contender_shadow_core::CLONE_IN_TICKS;
                draws.push((format!("ct_40_clone{}{}", if fading { "_in" } else { "" }, if left { "_l" } else { "" }), s.pos, 4));
            }
        }
        for c in &self.core.casts {
            match c {
                Cast::Grab { grab: Grab::Flight { from, at, cursor, .. }, .. } if tick % 4 == 0 => {
                    let facing = if at.x < from.x { "_l" } else { "" };
                    draws.push((format!("ct_body_run{}{facing}", (tick / 10) % 6), *cursor, 4));
                    let (hx, hy) = grab_hand_at(*from, *at, *cursor);
                    draws.push((format!("ct_grab_hand_d{}", grab_heading(*from, *at)), Point::new(hx, hy), 4));
                }
                Cast::Grab { caster, grab: Grab::Pull { target, .. }, .. } => {
                    let them = sim.get_entity(*target).map(|e| { let p = e.pos(); Point::new(p.0 as i64, p.1 as i64) });
                    if let (Some(from), Some(them)) = (at_of(&self.core, *caster), them) {
                        let (name, mid) = grab_ribbon(from, them);
                        draws.push((name, mid, 2));
                    }
                }
                Cast::Smother { caster, from, cursor, .. } => {
                    if let Some((name, mid)) = smother_trail(*from, *cursor, *caster != Caster::Me) {
                        draws.push((name, mid, 2));
                    }
                }
                Cast::Flurry { caster: Caster::Me, at, start, .. } if tick >= *start && tick % 4 == 0 => {
                    draws.push(("ct_body_idle0".into(), *at, 4));
                }
                _ => {}
            }
        }
        for (name, at, ticks) in draws.into_iter().take(24) {
            crate::fx_point(sim, &name, owner, at.x, at.y, ticks);
        }
    }
}

/// Set a unit at `to` at once (Shadowborne's teleport, the anchor, the grab's drag through walls).
fn place(sim: &mut StableSim, id: usize, to: Point) {
    sim.add_buff(id, &crate::timed(crate::place::MOVED, 2));
    sim.entity_set_pos(id, to.x.max(0) as u64, to.y.max(0) as u64);
    crate::steve::note_placed(sim, id, to.x, to.y);
}

impl StablePassive for Contender {
    fn configure(&mut self, params: &str) {
        let v = serde_json::from_str::<serde_json::Value>(params).ok();
        let on = |key: &str| v.as_ref().and_then(|v| v.get(key).and_then(|n| n.as_i64())) == Some(1);
        self.receipt_basic = on("shadow_receipts_v1");
        self.shadow_art = on("shadow_art_v1");
        self.shadow_directions = on("shadow_directions_v1");
        self.rhythm_art = on("shadow_rhythm_art_v1");
    }
    fn clone_box(&self) -> Box<dyn StablePassive> {
        Box::new(self.clone())
    }
    fn on_spawn(&mut self, sim: &mut StableSim, _: usize, id: usize) {
        self.rhythm_presenter.clear(sim);
        self.rhythm_presenter = crate::contender_rhythm::Presenter::default();
        self.core.respawn();
        // leftovers from 3.x careers
        for old in ["ct_merge", "ct_rhythm"] { sim.entity_remove_buff(id, old); }
        remove_prefix(sim, id, GO_PREFIX);
        remove_prefix(sim, id, BASIC_RECEIPT);
        remove_prefix(sim, id, ZONE_PREFIX);
        self.basic_hits.clear();
        self.in_skill = false;
        self.art_drawn_at = 0;
        self.landed_hits = 0;
    }
    /// A Shadow dodge: an enemy champion's hit, with a shadow ready, is healed straight back (Kratos's pattern; the hook
    /// reports damage after it lands) and the core slips him aside and counterpunches on its next step.
    fn on_damaged(&mut self, sim: &mut StableSim, _: usize, id: usize, attacker: usize, damage: usize) {
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
    fn on_dead(&mut self, sim: &mut StableSim, id: usize) {
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
        // the engine's casts (one per update; the first marker wins, the rest are dropped)
        let mut engine_order = None;
        if let Some(entity) = sim.get_entity(id) {
            let marks: Vec<String> = (0..entity.buff_count()).filter_map(|i| entity.buff_at(i))
                .map(|b| b.name().to_owned()).filter(|n| n.starts_with(CAST_PREFIX)).collect();
            for name in marks {
                if engine_order.is_none() {
                    engine_order = cast_order(&name);
                }
                sim.entity_remove_buff(id, &name);
            }
        }
        let mut f = self.frame(sim, id);
        // the game's AI owns S1, S2, the ult and movement; the core casts S3 itself (its `decide()` is Lab-only)
        f.manual = true;
        f.order = engine_order;
        let effects = self.core.step(&f);
        // tell the input AI which presses his kit can honour next tick: an S2 press while the anchor is cooling down
        // becomes a basic attack instead of a wasted cast
        let tick = sim.tick();
        let s1 = self.core.grab_ready(&f);
        let s2 = self.core.anchor_ready(&f);
        crate::press::note(sim.seed(), player, tick, if s1 { crate::press::S1 } else { 0 } | if s2 { crate::press::S2 } else { 0 });
        if self.rhythm_art { self.rhythm_presenter.sync(sim, id, player, &f, &self.core, &effects); }
        self.apply(sim, id, effects);
    }
}
