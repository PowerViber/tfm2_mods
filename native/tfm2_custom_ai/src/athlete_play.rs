//! The athlete's profile values, applied to every champion's movement.
//!
//! The per-player input layer sees what the engine wants each player to do next, for every champion,
//! base game and custom alike. This module only ever rewrites a **Move into another Move**. Casts and
//! attacks always pass through untouched: a player input AI that holds or injects casts breaks the
//! engine's planner and freezes the player (confirmed in playtests, `docs/playtest-notes.md`). It never
//! touches damage or stats, and an average athlete (50) is never rewritten.
//!
//! Every rule commits: once it takes over it keeps its point for `COMMIT_TICKS` (or until within
//! `COMMIT_REACHED` of it), then the same rule waits `REFIRE_TICKS` (`LONG_REFIRE_TICKS` for roaming and
//! grouping, which move a champion far) before it can take over again. Rules
//! that switch on a distance use separate start and stop distances, so a player never walks back and
//! forth across one line.
//!
//! | Value | Every champion |
//! |---|---|
//! | ego | Below 50, a hurt champion walking near an enemy goes home earlier: from `retreat_pct` HP with an enemy within 50,000, until none is within 75,000 and HP is 10 points higher (or it is home). |
//! | positioning | Above 55, a ranged champion between shots steps back 15,000 from an enemy too close, at most once per 30 ticks and never into a wall. |
//! | aggressive | Above 60, a move that already heads for an enemy in reach goes all the way. Below 40, chasing a hurt enemy (below 50% HP) out of reach turns back home until it comes back. |
//! | order | From 65 (two allies from 85), a move into two or more enemies first goes to the nearest ally; once grouped, it stays grouped until the allies are 90,000 away. |
//! | roaming | Above 60, an idle champion with no enemy within 100,000 joins a nearby ally's fight, and stops once an enemy is within 70,000. |
//! | skill_avoid | Below 50, the step out of a telegraphed zone is sometimes missed: rolled once per zone. |
//!
//! control_speed, skill_hit and judgement act on casts. Casts belong to the engine or to the kits that own
//! them (Gojo, Omen, DIO read them inside their controllers), so they are not applied here.

use crate::athlete_ai::Traits;
use crate::strategic_core::{distance, Point};
use std::collections::HashMap;
use std::sync::Mutex;

/// Ticks a started override keeps its point.
pub const COMMIT_TICKS: usize = 45;
/// An override ends once the player is this close to its point.
pub const COMMIT_REACHED: i64 = 8_000;
/// Ticks before the same rule may take over again.
pub const REFIRE_TICKS: usize = 60;
/// Ticks before roaming or grouping may start again: they move a champion far, so they wait longer.
pub const LONG_REFIRE_TICKS: usize = 300;

/// ego below this retreats early: at `EGO_RETREAT_BASE + (EGO_NEUTRAL - ego) * 4 / 5` percent HP.
pub const EGO_NEUTRAL: i64 = 50;
pub const EGO_RETREAT_BASE: i64 = 15;
pub const EGO_ENTER_RANGE: i64 = 50_000;
pub const EGO_EXIT_RANGE: i64 = 75_000;
pub const EGO_EXIT_MARGIN: i64 = 10;
pub const HOME_REACHED: i64 = 20_000;

pub const POSITIONING_KITE: i64 = 55;
pub const KITE_STEP: i64 = 15_000;
pub const KITE_EVERY: usize = 30;
/// Kiting ends once no enemy is within the kite distance plus this.
pub const KITE_RELEASE: i64 = 20_000;

pub const AGGRESSIVE_HIGH: i64 = 60;
pub const AGGRESSIVE_LOW: i64 = 40;
/// Extra reach above `AGGRESSIVE_HIGH`, at 100.
pub const AGGRESSIVE_REACH: i64 = 10_000;
/// How close a move's destination must be to an enemy to count as heading for it.
pub const HEADING_FOR: i64 = 15_000;
/// A chase is following a hurt enemy (below this HP percent) out of reach.
pub const CHASE_HP: i64 = 50;
/// How far a refused chase turns back toward home each tick.
pub const CHASE_BACK_STEP: i64 = 10_000;

pub const ORDER_ONE: i64 = 65;
pub const ORDER_TWO: i64 = 85;
pub const ORDER_GROUP: i64 = 60_000;
pub const ORDER_RELEASE: i64 = 90_000;
pub const ORDER_ENEMIES: usize = 2;
pub const ORDER_ENEMY_RADIUS: i64 = 40_000;

pub const ROAM_THRESHOLD: i64 = 60;
pub const ROAM_START: i64 = 100_000;
pub const ROAM_STOP: i64 = 70_000;
pub const ROAM_REACH_BASE: i64 = 40_000;
pub const ROAM_REACH_EXTRA: i64 = 150_000;
pub const ROAM_FIGHT: i64 = 35_000;

/// skill_avoid at or above this always steps out; below, the chance is `SKILL_AVOID_BASE + skill_avoid`.
pub const SKILL_AVOID_BASE: i64 = 50;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Rule {
    Ego,
    Kite,
    Press,
    Chase,
    Order,
    Roam,
}

impl Rule {
    #[cfg(test)]
    const ALL: [Rule; 6] = [Rule::Ego, Rule::Kite, Rule::Press, Rule::Chase, Rule::Order, Rule::Roam];
    fn index(self) -> usize {
        self as usize
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Change {
    Keep,
    Move(Point),
    /// Walk home (the engine's run-away move, never an escape skill).
    RunAway,
}

#[derive(Clone, Copy, Debug)]
pub struct Body {
    pub id: usize,
    pub point: Point,
    pub hp_pct: i64,
}

#[derive(Clone, Debug)]
pub struct Scene {
    pub tick: usize,
    pub me: Body,
    pub home: Point,
    pub range: i64,
    pub ranged: bool,
    pub attack_ready: bool,
    /// Visible living enemy champions.
    pub enemies: Vec<Body>,
    /// Living allied champions, not including me.
    pub allies: Vec<Body>,
    /// Whether a point is inside a wall.
    pub blocked: fn(i64, i64) -> bool,
}

/// No walls (tests and the stats rollout).
#[cfg(test)]
pub fn open(_: i64, _: i64) -> bool {
    false
}

#[derive(Clone, Debug, Default)]
pub struct State {
    committed: Option<(Point, usize, Rule)>,
    refire: [usize; 6],
    retreating: bool,
    grouped: bool,
    refused: Option<usize>,
    kiting: bool,
    last_kite: Option<usize>,
    /// The ally a committed Order or Roam follows: its point is read again every tick.
    follow: Option<usize>,
    /// Where the champion stood on the last call (to notice teleports, blinks and cables).
    seen: Option<(Point, usize)>,
    /// The rule behind the last change (for tests and logs).
    pub last: Option<Rule>,
}

pub fn retreat_pct(ego: i64) -> i64 {
    if ego >= EGO_NEUTRAL { 0 } else { EGO_RETREAT_BASE + (EGO_NEUTRAL - ego) * 4 / 5 }
}

fn near(list: &[Body], p: Point, r: i64) -> usize {
    list.iter().filter(|b| distance(b.point, p) <= r).count()
}

fn nearest(list: &[Body], p: Point) -> Option<&Body> {
    list.iter().min_by_key(|b| (distance(b.point, p), b.id))
}

fn away(from: Point, me: Point, d: i64) -> Point {
    let (dx, dy) = (me.0 - from.0, me.1 - from.1);
    let n = distance((0, 0), (dx, dy)).max(1);
    (me.0 + dx * d / n, me.1 + dy * d / n)
}

fn toward(me: Point, to: Point, d: i64) -> Point {
    let n = distance(me, to).max(1);
    if n <= d { to } else { (me.0 + (to.0 - me.0) * d / n, me.1 + (to.1 - me.1) * d / n) }
}

impl State {
    fn ready(&self, rule: Rule, tick: usize) -> bool {
        tick >= self.refire[rule.index()]
    }
    fn end(&mut self, rule: Rule, tick: usize, wait: usize) {
        let wait = if matches!(rule, Rule::Roam | Rule::Order) { wait.max(LONG_REFIRE_TICKS) } else { wait };
        self.refire[rule.index()] = tick.saturating_add(wait);
    }
    fn commit(&mut self, rule: Rule, p: Point, tick: usize) -> Change {
        self.committed = Some((p, tick.saturating_add(COMMIT_TICKS), rule));
        self.last = Some(rule);
        Change::Move(p)
    }
}

/// A champion that moved more than this per tick was carried (teleport, blink, cable, knock): a committed point
/// chosen from where it stood no longer applies.
pub const JUMP_PER_TICK: i64 = 20_000;
/// A rewritten move never ends this close to the champion (that would hold it in place).
pub const MIN_MOVE: i64 = 3_000;

/// The walkable part of a straight move: up to the first wall, or None when that leaves almost no move at all.
fn walkable(s: &Scene, from: Point, to: Point) -> Option<Point> {
    let n = (distance(from, to) / 3_000).max(1);
    let mut last = from;
    for i in 1..=n {
        let q = (from.0 + (to.0 - from.0) * i / n, from.1 + (to.1 - from.1) * i / n);
        if (s.blocked)(q.0, q.1) {
            break;
        }
        last = q;
    }
    (distance(from, last) >= MIN_MOVE).then_some(last)
}

/// What the athlete does with the engine's move toward `dest`. Deterministic; `state` is this player's memory.
pub fn decide(t: &Traits, s: &Scene, dest: Point, state: &mut State) -> Change {
    let me = s.me.point;
    let tick = s.tick;
    state.last = None;
    if let Some((p, t0)) = state.seen {
        let dt = tick.saturating_sub(t0).max(1) as i64;
        if distance(p, me) > JUMP_PER_TICK * dt {
            state.committed = None;
            state.follow = None;
            state.kiting = false;
        }
    }
    state.seen = Some((me, tick));

    // ego: walk home earlier when hurt, and only stop once safe
    let threshold = retreat_pct(t.ego);
    if state.retreating {
        let safe = near(&s.enemies, me, EGO_EXIT_RANGE) == 0 && s.me.hp_pct >= threshold + EGO_EXIT_MARGIN;
        if threshold == 0 || safe || distance(me, s.home) <= HOME_REACHED {
            state.retreating = false;
            state.end(Rule::Ego, tick, REFIRE_TICKS);
        } else {
            state.last = Some(Rule::Ego);
            return Change::RunAway;
        }
    } else if threshold > 0
        && state.ready(Rule::Ego, tick)
        && s.me.hp_pct < threshold
        && near(&s.enemies, me, EGO_ENTER_RANGE) > 0
        && distance(me, s.home) > HOME_REACHED
    {
        state.retreating = true;
        state.committed = None;
        state.last = Some(Rule::Ego);
        return Change::RunAway;
    }

    // a committed override keeps its point (an ally it follows is followed where it is now)
    if let Some((stored, until, rule)) = state.committed {
        let p = state.follow.and_then(|id| s.allies.iter().find(|a| a.id == id)).map_or(stored, |a| a.point);
        let roam_arrived = rule == Rule::Roam && near(&s.enemies, me, ROAM_STOP) > 0;
        if roam_arrived {
            state.committed = None;
            state.end(rule, tick, LONG_REFIRE_TICKS);
        } else if tick >= until || distance(me, p) <= COMMIT_REACHED {
            state.committed = None;
            state.follow = None;
            state.end(rule, tick, REFIRE_TICKS);
        } else if let Some(q) = walkable(s, me, p) {
            state.last = Some(rule);
            return Change::Move(q);
        } else {
            state.committed = None;
            state.follow = None;
            state.end(rule, tick, REFIRE_TICKS);
        }
    }

    let heading = s.enemies.iter()
        .filter(|e| distance(e.point, dest) <= s.range + HEADING_FOR && distance(e.point, dest) + 5_000 < distance(e.point, me))
        .min_by_key(|e| (distance(e.point, dest), e.id));

    // aggressive (low): refuse a chase after an enemy out of reach; keep refusing until it comes back
    if t.aggressive < AGGRESSIVE_LOW {
        let limit = s.range * 12 / 10 + 5_000;
        if let Some(id) = state.refused {
            let still = heading.is_some_and(|e| e.id == id)
                && s.enemies.iter().any(|e| e.id == id && distance(e.point, me) > limit);
            if let Some(q) = still.then(|| walkable(s, me, toward(me, s.home, CHASE_BACK_STEP))).flatten() {
                state.last = Some(Rule::Chase);
                return Change::Move(q);
            }
            state.refused = None;
            state.end(Rule::Chase, tick, REFIRE_TICKS);
        } else if let Some(e) = heading {
            if e.hp_pct < CHASE_HP && distance(e.point, me) > limit && state.ready(Rule::Chase, tick) {
                if let Some(q) = walkable(s, me, toward(me, s.home, CHASE_BACK_STEP)) {
                    state.refused = Some(e.id);
                    state.last = Some(Rule::Chase);
                    return Change::Move(q);
                }
            }
        }
    }

    // order: into two or more enemies, group with allies first; once grouped, stay grouped
    if t.order >= ORDER_ONE {
        let need = if t.order >= ORDER_TWO { 2 } else { 1 };
        let grouped_now = near(&s.allies, me, ORDER_GROUP) >= need;
        if grouped_now {
            state.grouped = true;
        } else if state.grouped && near(&s.allies, me, ORDER_RELEASE) < need {
            state.grouped = false;
        }
        let into = near(&s.enemies, dest, ORDER_ENEMY_RADIUS) >= ORDER_ENEMIES;
        if into && !state.grouped && state.ready(Rule::Order, tick) {
            if let Some((id, q)) = nearest(&s.allies, me).and_then(|a| walkable(s, me, a.point).map(|q| (a.id, q))) {
                state.follow = Some(id);
                return state.commit(Rule::Order, q, tick);
            }
        }
    }

    // positioning: a ranged champion with an enemy too close kites: one mode that steps back between shots
    // (at most once per KITE_EVERY, never into a wall) and otherwise follows the engine's approach
    if s.ranged && t.positioning > POSITIONING_KITE {
        let keep = s.range * t.positioning.min(100) / 100;
        let close = nearest(&s.enemies, me).map(|e| (e.point, distance(e.point, me)));
        if state.kiting && close.map_or(true, |c| c.1 > keep + KITE_RELEASE) {
            state.kiting = false;
            state.end(Rule::Kite, tick, REFIRE_TICKS);
        } else if !state.kiting && close.is_some_and(|c| c.1 < keep) && state.ready(Rule::Kite, tick) {
            state.kiting = true;
        }
        if state.kiting {
            state.last = Some(Rule::Kite);
            if let Some((from, d)) = close {
                if !s.attack_ready && d < keep && state.last_kite.map_or(true, |k| tick >= k.saturating_add(KITE_EVERY)) {
                    let p = away(from, me, KITE_STEP);
                    if !(s.blocked)(p.0, p.1) {
                        state.last_kite = Some(tick);
                        return Change::Move(p);
                    }
                }
            }
            return Change::Move(dest);
        }
    }

    // aggressive (high): a move already heading for an enemy in reach goes all the way
    if t.aggressive > AGGRESSIVE_HIGH && s.me.hp_pct > 50 {
        let reach = s.range + AGGRESSIVE_REACH * (t.aggressive.min(100) - AGGRESSIVE_HIGH) / (100 - AGGRESSIVE_HIGH);
        if let Some(q) = heading
            .filter(|e| distance(e.point, me) <= reach && e.point != dest)
            .and_then(|e| walkable(s, me, e.point))
        {
            state.last = Some(Rule::Press);
            return Change::Move(q);
        }
    }

    // roaming: an idle champion joins a nearby ally's fight
    if t.roaming > ROAM_THRESHOLD && near(&s.enemies, me, ROAM_START) == 0 && state.ready(Rule::Roam, tick) {
        let reach = ROAM_REACH_BASE + ROAM_REACH_EXTRA * (t.roaming.min(100) - ROAM_THRESHOLD) / (100 - ROAM_THRESHOLD);
        let fighting = s.allies.iter()
            .filter(|a| distance(a.point, me) <= reach && near(&s.enemies, a.point, ROAM_FIGHT) > 0)
            .min_by_key(|a| (distance(a.point, me), a.id));
        if let Some((id, q)) = fighting.and_then(|a| walkable(s, me, a.point).map(|q| (a.id, q))) {
            state.follow = Some(id);
            return state.commit(Rule::Roam, q, tick);
        }
    }
    Change::Keep
}

/// skill_avoid: whether the athlete steps out of this zone. Rolled once per zone; always from 50 up.
pub fn avoids(t: &Traits, seed: u64, me: usize, zone: Point) -> bool {
    let chance = (SKILL_AVOID_BASE + t.skill_avoid.clamp(0, 100)).min(100) as u64;
    let mut h = seed
        ^ (me as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15)
        ^ ((zone.0 / 1000) as u64).wrapping_mul(0xbf58_476d_1ce4_e5b9)
        ^ ((zone.1 / 1000) as u64).wrapping_mul(0xd6e8_feb8_6659_fd93);
    h ^= h >> 31;
    h = h.wrapping_mul(0x94d0_49bb_1331_11eb);
    h ^= h >> 29;
    h % 100 < chance
}

// ---------------------------------------------------------------------------------------------------------
// Glue: the per-player input layer.

use mod_api_stable::{EntityKindV1, InputKindV1, InputV1, SkillSlotV1, StableAiContext};

static MEMORY: Mutex<Option<HashMap<(u64, usize), State>>> = Mutex::new(None);

/// Applies the athlete's profile to the engine's move for this player. `None` keeps the input; only a
/// Move is ever rewritten, and only into a Move.
pub fn adjust(ctx: &mut StableAiContext<'_>, input: InputV1) -> Option<InputV1> {
    let out = match change_for(ctx, input)? {
        Change::Keep => return None,
        Change::RunAway => ctx.run_away_without_skill_input()?,
        Change::Move(p) => InputV1::move_to(p.0.max(0) as u64, p.1.max(0) as u64),
    };
    (out.kind == InputKindV1::Move.code() && ctx.is_valid_input(&out)).then_some(out)
}

/// What the athlete's profile would make of the engine's move (None: not a move, switched off, or no profile).
/// Levi uses this directly: his own brain walks him, so the change becomes a destination it reads.
pub fn change_for(ctx: &mut StableAiContext<'_>, input: InputV1) -> Option<Change> {
    if input.kind != InputKindV1::Move.code() || !crate::settings::assist().athlete_moves {
        return None;
    }
    let dest = (input.x as i64, input.y as i64);
    let pid = ctx.player_id();
    let change = {
        let sim = ctx.sim()?;
        let me_e = sim.get_player(pid)?.champion()?;
        if !me_e.is_alive() {
            return None;
        }
        let traits = crate::athlete_ai::traits(&sim, me_e.id());
        if !traits.known {
            return None;
        }
        let (key, tick) = (crate::match_key(&sim), sim.tick());
        let all = crate::champions_shared(&sim);
        let me = all.iter().find(|c| c.id == me_e.id())?;
        let def = crate::roster::def(&me.name);
        let range = def.map_or(crate::farming_data::attack_range(&me.name), |d| d.basic.range);
        let ranged = def.map_or(range > 30_000, |d| d.basic.reach == crate::roster::Range::Ranged);
        let attack_ready = me_e.cooldown(SkillSlotV1::Attack).map_or(true, |c| c.remaining == 0);
        let home = (0..sim.entity_count())
            .filter_map(|n| sim.entity_at(n))
            .find(|e| e.team() == me.team && e.kind() == Some(EntityKindV1::Nexus))
            .map_or((me.x, me.y), |e| { let p = e.pos(); (p.0 as i64, p.1 as i64) });
        let body = |c: &crate::Champ| Body { id: c.id, point: (c.x, c.y), hp_pct: (c.hp * 100 / c.max_hp.max(1)) as i64 };
        let scene = Scene {
            tick,
            me: body(me),
            home,
            range,
            ranged,
            attack_ready,
            enemies: all.iter().filter(|c| c.team != me.team && sim.is_visible(me.team, c.id)).map(body).collect(),
            allies: all.iter().filter(|c| c.team == me.team && c.id != me.id).map(body).collect(),
            blocked: crate::walls::wall_at,
        };
        let mut guard = MEMORY.lock().ok()?;
        let memory = guard.get_or_insert_with(HashMap::new);
        if memory.len() > 64 {
            memory.retain(|k, _| k.0 == key);
        }
        decide(&traits, &scene, dest, memory.entry((key, pid)).or_default())
    };
    Some(change)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn traits(k: usize, v: i64) -> Traits {
        let mut t = Traits::default();
        t.known = true;
        let slot = match k {
            0 => &mut t.ego,
            1 => &mut t.judgement,
            2 => &mut t.positioning,
            3 => &mut t.control_speed,
            4 => &mut t.skill_hit,
            5 => &mut t.skill_avoid,
            6 => &mut t.aggressive,
            7 => &mut t.order,
            _ => &mut t.roaming,
        };
        *slot = v;
        t
    }

    fn average() -> Traits {
        let mut t = Traits::default();
        t.known = true;
        t
    }

    fn body(id: usize, x: i64, y: i64) -> Body { Body { id, point: (x, y), hp_pct: 100 } }

    /// One representative of each kind: a base-game ranged champion, a native melee kit, a dash champion.
    fn scenes() -> Vec<(&'static str, Scene)> {
        ["archer", "tfm2_custom_gojo", "tfm2_levi_levi"]
            .into_iter()
            .map(|name| {
                let d = crate::roster::def(name).unwrap();
                let scene = Scene {
                    tick: 1000,
                    me: body(1, 100_000, 100_000),
                    home: (-400_000, 100_000),
                    range: d.basic.range,
                    ranged: d.basic.reach == crate::roster::Range::Ranged,
                    attack_ready: false,
                    enemies: vec![body(10, 100_000 + d.basic.range / 2, 100_000), body(11, 100_000 + d.basic.range / 2 + 10_000, 105_000)],
                    allies: vec![body(2, 20_000, 100_000)],
                    blocked: open,
                };
                (name, scene)
            })
            .collect()
    }

    fn once(t: &Traits, s: &Scene, dest: Point) -> Change {
        decide(t, s, dest, &mut State::default())
    }

    #[test]
    fn an_average_athlete_changes_nothing() {
        for (name, s) in scenes() {
            for dest in [(0, 0), s.enemies[0].point, (300_000, 100_000)] {
                assert_eq!(once(&average(), &s, dest), Change::Keep, "{name} {dest:?}");
            }
        }
    }

    #[test]
    fn ego_low_retreats_earlier_and_stays_retreating_until_safe() {
        for (name, mut s) in scenes() {
            s.me.hp_pct = 35;
            assert_eq!(once(&traits(0, 20), &s, (0, 0)), Change::RunAway, "{name}");
            assert_eq!(once(&traits(0, 80), &s, (0, 0)), Change::Keep, "{name}");
            let mut st = State::default();
            decide(&traits(0, 20), &s, (0, 0), &mut st);
            // the enemy is 60,000 away now: inside the exit range, so still retreating
            s.enemies.truncate(1);
            s.enemies[0].point = (160_000, 100_000);
            assert_eq!(decide(&traits(0, 20), &s, (0, 0), &mut st), Change::RunAway, "{name}");
            s.enemies[0].point = (200_000, 100_000);
            s.me.hp_pct = 60;
            assert_eq!(decide(&traits(0, 20), &s, (0, 0), &mut st), Change::Keep, "{name}");
        }
    }

    #[test]
    fn positioning_high_kites_for_ranged_only_and_never_into_a_wall() {
        for (name, mut s) in scenes() {
            let high = once(&traits(2, 90), &s, s.enemies[0].point);
            assert_eq!(matches!(high, Change::Move(p) if p.0 < s.me.point.0), s.ranged, "{name}");
            assert_eq!(once(&traits(2, 20), &s, s.enemies[0].point), Change::Keep, "{name}");
            s.blocked = |x, _| x < 100_000;
            let dest = s.enemies[0].point;
            if s.ranged {
                assert_eq!(once(&traits(2, 90), &s, dest), Change::Move(dest), "{name}: no step into a wall");
            }
        }
    }

    #[test]
    fn aggressive_high_finishes_the_approach_and_low_turns_back_from_a_chase() {
        for (name, mut s) in scenes() {
            s.enemies.truncate(1);
            s.enemies[0].point.0 = s.me.point.0 + s.range;
            let short = (s.enemies[0].point.0 - s.range / 2, s.enemies[0].point.1);
            assert_eq!(once(&traits(6, 90), &s, short), Change::Move(s.enemies[0].point), "{name}");
            assert_eq!(once(&traits(6, 20), &s, short), Change::Keep, "{name}");
            s.enemies[0].point.0 = s.me.point.0 + s.range * 2 + 10_000;
            s.enemies[0].hp_pct = 30;
            let chase = (s.enemies[0].point.0 - s.range / 2, s.enemies[0].point.1);
            assert!(matches!(once(&traits(6, 20), &s, chase), Change::Move(p) if p.0 < s.me.point.0), "{name}");
            assert_eq!(once(&traits(6, 80), &s, chase), Change::Keep, "{name}");
        }
    }

    #[test]
    fn order_high_groups_before_walking_into_two() {
        for (name, s) in scenes() {
            assert_eq!(once(&traits(7, 90), &s, s.enemies[0].point), Change::Move(s.allies[0].point), "{name}");
            assert_eq!(once(&traits(7, 20), &s, s.enemies[0].point), Change::Keep, "{name}");
        }
    }

    #[test]
    fn roaming_high_joins_a_nearby_fight() {
        for (name, mut s) in scenes() {
            s.enemies = vec![body(10, -60_000, 100_000)];
            s.allies = vec![body(2, -40_000, 100_000)];
            s.me.point = (100_000, 100_000);
            assert_eq!(once(&traits(8, 90), &s, (200_000, 100_000)), Change::Move((-40_000, 100_000)), "{name}");
            assert_eq!(once(&traits(8, 20), &s, (200_000, 100_000)), Change::Keep, "{name}");
        }
    }

    #[test]
    fn skill_avoid_low_misses_whole_zones_not_half_seconds() {
        let zones: Vec<Point> = (0..200).map(|n| (n * 37_000, n * 11_000)).collect();
        let count = |v: i64| zones.iter().filter(|z| avoids(&traits(5, v), 7, 1, **z)).count();
        assert_eq!(count(50), 200);
        assert_eq!(count(80), 200);
        assert!((100..170).contains(&count(10)), "{}", count(10));
        // the same zone always gives the same answer
        assert!((0..50).all(|_| avoids(&traits(5, 10), 7, 1, zones[3]) == avoids(&traits(5, 10), 7, 1, zones[3])));
    }

    // ----- the engine against the rewrite, over time -----

    /// A scripted engine and world: each tick the engine picks a move for the player (it answers whatever
    /// the rewrite did last tick), the player walks toward the final command at 1,000 per tick, takes
    /// damage near enemies and heals at home. Returns (per-rule override starts, longest stand-still while
    /// the engine asked to move elsewhere, total rewrites).
    fn play(t: &Traits, scenario: usize, name: &str) -> ([usize; 6], usize, usize) {
        let d = crate::roster::def(name).unwrap();
        let home = (-400_000, 100_000);
        let mut me = (0i64, 100_000i64);
        let mut hp = 100i64;
        let mut st = State::default();
        let mut starts = [0usize; 6];
        let mut prev: Option<Rule> = None;
        let (mut still, mut longest, mut rewrites) = (0usize, 0usize, 0usize);
        for tick in 0..600usize {
            // the world
            let (enemies, allies): (Vec<Body>, Vec<Body>) = match scenario {
                // lane: two enemies holding a front that pushes and recedes
                0 => {
                    let x = 120_000 + ((tick as i64 / 4) % 80_000 - 40_000).abs();
                    (vec![Body { id: 10, point: (x, 100_000), hp_pct: 80 }, Body { id: 11, point: (x + 12_000, 108_000), hp_pct: 90 }],
                     vec![Body { id: 2, point: (-30_000, 120_000), hp_pct: 100 }])
                }
                // chase: one enemy running away at 900 per tick
                1 => (vec![Body { id: 10, point: (60_000 + tick as i64 * 900, 100_000), hp_pct: 30 }],
                      vec![Body { id: 2, point: (-60_000, 100_000), hp_pct: 100 }]),
                // roam: a quiet lane and an ally fighting 140,000 away
                _ => (vec![Body { id: 10, point: (0, 260_000), hp_pct: 70 }],
                      vec![Body { id: 2, point: (0, 240_000), hp_pct: 70 }]),
            };
            // the engine: approach the nearest enemy within 150,000, otherwise walk the lane
            let target = enemies.iter().filter(|e| distance(e.point, me) <= 150_000).min_by_key(|e| distance(e.point, me));
            let dest = match target {
                Some(e) => toward(e.point, me, d.basic.range * 9 / 10),
                None => (80_000, 100_000),
            };
            let scene = Scene {
                tick, me: Body { id: 1, point: me, hp_pct: hp }, home, range: d.basic.range,
                ranged: d.basic.reach == crate::roster::Range::Ranged, attack_ready: tick % 3 == 0,
                enemies: enemies.clone(), allies, blocked: open,
            };
            let change = decide(t, &scene, dest, &mut st);
            if change != Change::Keep {
                rewrites += 1;
            }
            if st.last != prev {
                if let Some(r) = st.last {
                    starts[r.index()] += 1;
                }
                prev = st.last;
            }
            let goal = match change {
                Change::Keep => dest,
                Change::Move(p) => p,
                Change::RunAway => home,
            };
            let next = toward(me, goal, 1_000);
            let asked = distance(me, dest) > 2_000;
            if next == me && asked && distance(me, home) > HOME_REACHED {
                still += 1;
                longest = longest.max(still);
            } else {
                still = 0;
            }
            me = next;
            // damage near enemies, healing at home
            if near(&enemies, me, 40_000) > 0 {
                hp = (hp - 1).max(1);
            }
            if distance(me, home) <= HOME_REACHED {
                hp = (hp + 5).min(100);
            }
        }
        (starts, longest, rewrites)
    }

    #[test]
    fn the_rewrite_never_flip_flops_or_freezes_against_the_engine() {
        for name in ["archer", "tfm2_custom_gojo", "tfm2_levi_levi", "tfm2_blockcraft_steve"] {
            for scenario in 0..3 {
                for k in 0..9 {
                    for v in [0, 20, 80, 100] {
                        let (starts, longest, _) = play(&traits(k, v), scenario, name);
                        for r in Rule::ALL {
                            assert!(starts[r.index()] <= 3, "{name} scenario {scenario} value {k}={v}: {r:?} started {} times", starts[r.index()]);
                        }
                        assert!(longest <= 30, "{name} scenario {scenario} value {k}={v}: stood still {longest} ticks");
                    }
                }
            }
        }
    }

    #[test]
    fn an_average_athlete_is_never_rewritten_over_a_whole_game() {
        for name in ["archer", "tfm2_custom_gojo", "tfm2_levi_levi", "tfm2_blockcraft_steve"] {
            for scenario in 0..3 {
                assert_eq!(play(&average(), scenario, name).2, 0, "{name} scenario {scenario}");
            }
        }
    }

    #[test]
    fn every_movement_value_does_something_over_a_game() {
        for k in [0, 2, 6, 7, 8] {
            let moved = ["archer", "tfm2_custom_gojo"].iter().any(|n| (0..3).any(|s| {
                [0, 20, 80, 100].iter().any(|v| play(&traits(k, *v), s, n).2 > 0)
            }));
            assert!(moved, "value {k} never rewrote a move");
        }
    }
}
