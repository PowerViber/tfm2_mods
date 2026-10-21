//! The athlete's nine profile values, applied to every champion.
//!
//! The per-player input layer sees what the engine wants each player to do next (move, attack, cast) for
//! every champion, base game and custom alike. This module turns the athlete's profile into small,
//! deterministic changes to that input. It never touches damage or stats, and an average athlete (50)
//! plays as the engine does today. Kits that drive their own casts (Gojo, Omen, DIO, Unified…) keep
//! reading the same values inside their own controllers.
//!
//! | Value | Every champion |
//! |---|---|
//! | control_speed | Below 50, a fresh cast waits up to 18 ticks before it goes out. |
//! | skill_hit | A ground-aimed cast leads a moving enemy (above 50) or trails it (below 50). |
//! | skill_avoid | Below 50, the step out of a telegraphed zone is sometimes missed. |
//! | positioning | Above 55, a ranged champion steps back between its own shots. |
//! | judgement | Above 55, an ultimate waits (up to 3 s) until it catches two or a low target. |
//! | ego | Below 50, a champion retreats earlier when hurt with an enemy close. |
//! | aggressive | Above 60, an enemy champion in reach is attacked; below 40, nobody is chased out of range. |
//! | order | From 65, a champion waits for allies before attacking into two or more enemies. |
//! | roaming | Above 60, an idle champion joins a nearby ally's fight. |

use crate::athlete_ai::Traits;
use crate::strategic_core::{distance, Point};
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Aim {
    Unit(usize),
    Spot(Point),
    Other,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Want {
    Move(Point),
    Return,
    Attack(usize),
    /// slot: 1 skill, 2 skill2, 3 ult
    Cast(u8, Aim),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Change {
    Keep,
    Move(Point),
    Attack(usize),
    Cast(u8, Aim),
    RunAway,
}

#[derive(Clone, Copy, Debug)]
pub struct Body {
    pub id: usize,
    pub point: Point,
    pub hp_pct: i64,
    /// Observed motion per tick.
    pub velocity: Point,
}

#[derive(Clone, Debug)]
pub struct Scene {
    pub tick: usize,
    pub me: Body,
    pub range: i64,
    pub ranged: bool,
    pub attack_ready: bool,
    /// Visible living enemy champions.
    pub enemies: Vec<Body>,
    /// Living allied champions, not including me.
    pub allies: Vec<Body>,
}

#[derive(Clone, Debug, Default)]
pub struct State {
    pending: Option<(u8, Aim, usize)>,
    ult_since: Option<usize>,
}

const LEAD_TICKS: i64 = 15;
const MAX_ULT_HOLD: usize = 180;

pub fn retreat_pct(ego: i64) -> i64 {
    if ego >= 50 { 0 } else { 15 + (50 - ego) * 4 / 5 }
}

/// Extra ticks before a fresh cast goes out: the profile's reaction time beyond an average athlete's.
pub fn cast_delay(t: &Traits) -> usize {
    t.reaction_ticks().saturating_sub(15) * 2
}

fn near(list: &[Body], p: Point, r: i64) -> usize {
    list.iter().filter(|b| distance(b.point, p) <= r).count()
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

/// What the athlete does with what the engine wants. Deterministic; `state` is this player's memory.
pub fn decide(t: &Traits, s: &Scene, want: Want, state: &mut State) -> Change {
    let me = s.me.point;
    if !matches!(want, Want::Cast(3, _)) {
        state.ult_since = None;
    }
    // ego: retreat earlier when hurt with an enemy close
    if !matches!(want, Want::Return) && s.me.hp_pct < retreat_pct(t.ego) && near(&s.enemies, me, 50_000) > 0 {
        return Change::RunAway;
    }
    let target_of = |aim: Aim| match aim {
        Aim::Unit(id) => s.enemies.iter().find(|e| e.id == id).map(|e| e.point),
        Aim::Spot(p) => Some(p),
        Aim::Other => None,
    };
    let hold = |aim: Aim| match aim {
        Aim::Unit(id) if s.enemies.iter().any(|e| e.id == id) => Change::Attack(id),
        _ => Change::Move(me),
    };
    match want {
        Want::Cast(slot, aim) => {
            // judgement: an ultimate waits until it is worth it
            if slot == 3 && t.judgement > 55 {
                let worth = target_of(aim).map_or(true, |p| {
                    near(&s.enemies, p, 30_000) >= 2
                        || s.enemies.iter().any(|e| distance(e.point, p) <= 20_000 && e.hp_pct <= 40)
                });
                let since = *state.ult_since.get_or_insert(s.tick);
                let patience = MAX_ULT_HOLD * (t.judgement - 55).min(45) as usize / 45;
                if !worth && s.tick - since < patience {
                    return hold(aim);
                }
            }
            // control_speed: a fresh decision takes a moment to act on
            let delay = cast_delay(t);
            match state.pending {
                Some((k, a, since)) if k == slot && a == aim => {
                    if s.tick - since < delay {
                        return hold(aim);
                    }
                }
                _ if delay > 0 => {
                    state.pending = Some((slot, aim, s.tick));
                    return hold(aim);
                }
                _ => {}
            }
            state.pending = None;
            // skill_hit: lead (or trail) a moving enemy at the aimed spot
            if let Aim::Spot(p) = aim {
                if let Some(e) = s.enemies.iter().filter(|e| distance(e.point, p) <= 25_000).min_by_key(|e| distance(e.point, p)) {
                    let k = (t.aim_lead_percent() - 50) * LEAD_TICKS;
                    let shift = (e.velocity.0 * k / 50, e.velocity.1 * k / 50);
                    if shift != (0, 0) {
                        return Change::Cast(slot, Aim::Spot((p.0 + shift.0, p.1 + shift.1)));
                    }
                }
            }
            Change::Keep
        }
        Want::Attack(id) => {
            let Some(target) = s.enemies.iter().find(|e| e.id == id) else { return Change::Keep };
            let d = distance(me, target.point);
            // order: wait for allies before attacking into two or more enemies
            if t.order >= 65 {
                let need = if t.order >= 85 { 2 } else { 1 };
                if near(&s.enemies, target.point, 40_000) >= 2 && near(&s.allies, me, 60_000) < need {
                    if let Some(a) = s.allies.iter().min_by_key(|a| (distance(a.point, me), a.id)) {
                        return Change::Move(toward(me, a.point, 20_000));
                    }
                }
            }
            // aggressive (low): no chasing out of range
            if t.aggressive < 40 && d > s.range * 12 / 10 + 5_000 {
                return Change::Move(me);
            }
            // positioning: ranged champions step back between shots
            if s.ranged && t.positioning > 55 && !s.attack_ready && d < s.range * t.positioning.min(100) / 100 {
                return Change::Move(away(target.point, me, 15_000));
            }
            Change::Keep
        }
        Want::Move(_) => {
            // aggressive (high): take the fight that is already in reach
            if t.aggressive > 60 && s.me.hp_pct > 50 {
                let reach = s.range + 10_000 * (t.aggressive.min(100) - 60) / 40;
                if let Some(e) = s.enemies.iter().filter(|e| distance(e.point, me) <= reach).min_by_key(|e| (e.hp_pct, e.id)) {
                    return Change::Attack(e.id);
                }
            }
            // roaming: an idle champion joins a nearby ally's fight
            if t.roaming > 60 && near(&s.enemies, me, 80_000) == 0 {
                let reach = 40_000 + 150_000 * (t.roaming.min(100) - 60) / 40;
                let fighting = s.allies.iter()
                    .filter(|a| distance(a.point, me) <= reach && near(&s.enemies, a.point, 35_000) > 0)
                    .min_by_key(|a| (distance(a.point, me), a.id));
                if let Some(a) = fighting {
                    return Change::Move(a.point);
                }
            }
            Change::Keep
        }
        Want::Return => Change::Keep,
    }
}

/// skill_avoid: whether the athlete steps out of a telegraphed zone this half-second. Always from 50 up.
pub fn avoids(t: &Traits, seed: u64, me: usize, tick: usize) -> bool {
    let chance = (50 + t.skill_avoid.clamp(0, 100)).min(100) as u64;
    let mut h = seed ^ (me as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15) ^ ((tick / 30) as u64).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    h ^= h >> 31;
    h = h.wrapping_mul(0x94d0_49bb_1331_11eb);
    h ^= h >> 29;
    h % 100 < chance
}

// ---------------------------------------------------------------------------------------------------------
// Glue: the per-player input layer.

use mod_api_stable::{InputKindV1, InputTargetKindV1, InputTargetV1, InputV1, SkillSlotV1, StableAiContext};

#[derive(Default)]
struct Memory {
    state: HashMap<(u64, usize), State>,
    seen: HashMap<(u64, usize), (Point, usize)>,
}

static MEMORY: Mutex<Option<Memory>> = Mutex::new(None);

fn want_of(input: &InputV1) -> Option<Want> {
    let k = input.kind;
    let aim = || match input.target.kind {
        x if x == InputTargetKindV1::Target.code() => Aim::Unit(input.target.target_id),
        x if x == InputTargetKindV1::Pos.code() => Aim::Spot((input.target.x as i64, input.target.y as i64)),
        _ => Aim::Other,
    };
    Some(if k == InputKindV1::Move.code() {
        Want::Move((input.x as i64, input.y as i64))
    } else if k == InputKindV1::Return.code() {
        Want::Return
    } else if k == InputKindV1::Attack.code() {
        match aim() { Aim::Unit(id) => Want::Attack(id), _ => return None }
    } else if k == InputKindV1::Skill.code() {
        Want::Cast(1, aim())
    } else if k == InputKindV1::Skill2.code() {
        Want::Cast(2, aim())
    } else if k == InputKindV1::Ult.code() {
        Want::Cast(3, aim())
    } else {
        return None;
    })
}

/// Applies the athlete's profile to the engine's input for this player. `None` keeps the input.
pub fn adjust(ctx: &mut StableAiContext<'_>, input: InputV1) -> Option<InputV1> {
    let want = want_of(&input)?;
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
        let (seed, tick) = (sim.seed(), sim.tick());
        let all = crate::champions(&sim);
        let me = all.iter().find(|c| c.id == me_e.id())?;
        let name = me.name.clone();
        let def = crate::roster::def(&name);
        let range = def.map_or(crate::farming_data::attack_range(&name), |d| d.basic.range);
        let ranged = def.map_or(range > 30_000, |d| d.basic.reach == crate::roster::Range::Ranged);
        let attack_ready = me_e.cooldown(SkillSlotV1::Attack).map_or(true, |c| c.remaining == 0);
        let mut guard = MEMORY.lock().ok()?;
        let memory = guard.get_or_insert_with(Memory::default);
        if memory.seen.len() > 512 {
            memory.seen.retain(|k, v| k.0 == seed && v.1 + 600 > tick);
        }
        let mut body = |c: &crate::Champ| {
            let p = (c.x, c.y);
            let velocity = match memory.seen.insert((seed, c.id), (p, tick)) {
                Some((q, t0)) if tick > t0 && tick - t0 <= 30 => {
                    let dt = (tick - t0) as i64;
                    ((p.0 - q.0) / dt, (p.1 - q.1) / dt)
                }
                _ => (0, 0),
            };
            Body { id: c.id, point: p, hp_pct: (c.hp * 100 / c.max_hp.max(1)) as i64, velocity }
        };
        let scene = Scene {
            tick,
            me: body(me),
            range,
            ranged,
            attack_ready,
            enemies: all.iter().filter(|c| c.team != me.team && sim.is_visible(me.team, c.id)).map(&mut body).collect(),
            allies: all.iter().filter(|c| c.team == me.team && c.id != me.id).map(&mut body).collect(),
        };
        if memory.state.len() > 64 {
            memory.state.retain(|k, _| k.0 == seed);
        }
        let state = memory.state.entry((seed, pid)).or_default();
        decide(&traits, &scene, want, state)
    };
    let out = match change {
        Change::Keep => return None,
        Change::RunAway => ctx.run_away_input()?,
        Change::Move(p) => InputV1::move_to(p.0.max(0) as u64, p.1.max(0) as u64),
        Change::Attack(id) => InputV1::action(InputKindV1::Attack, InputTargetV1::target(id)),
        Change::Cast(_, Aim::Spot(p)) => InputV1 { target: InputTargetV1::pos(p.0.max(0) as u64, p.1.max(0) as u64), ..input },
        Change::Cast(..) => return None,
    };
    ctx.is_valid_input(&out).then_some(out)
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

    fn body(id: usize, x: i64, y: i64) -> Body { Body { id, point: (x, y), hp_pct: 100, velocity: (0, 0) } }

    /// One representative of each kind: a base-game ranged champion, a native melee kit, a dash champion.
    fn scenes() -> Vec<(&'static str, Scene)> {
        ["archer", "tfm2_custom_gojo", "tfm2_levi_levi"]
            .into_iter()
            .map(|name| {
                let d = crate::roster::def(name).unwrap();
                let scene = Scene {
                    tick: 1000,
                    me: body(1, 100_000, 100_000),
                    range: d.basic.range,
                    ranged: d.basic.reach == crate::roster::Range::Ranged,
                    attack_ready: false,
                    enemies: vec![body(10, 100_000 + d.basic.range / 2, 100_000), body(11, 100_000 + d.basic.range / 2 + 10_000, 105_000)],
                    allies: vec![body(2, 60_000, 100_000)],
                };
                (name, scene)
            })
            .collect()
    }

    fn run(t: &Traits, s: &Scene, want: Want, ticks: usize) -> Vec<Change> {
        let mut st = State::default();
        (0..ticks).map(|n| { let mut s = s.clone(); s.tick += n; decide(t, &s, want, &mut st) }).collect()
    }

    #[test]
    fn an_average_athlete_changes_nothing() {
        let t = { let mut t = Traits::default(); t.known = true; t };
        for (name, s) in scenes() {
            for want in [Want::Move((0, 0)), Want::Attack(10), Want::Cast(1, Aim::Unit(10)), Want::Cast(3, Aim::Unit(10)), Want::Return] {
                assert_eq!(run(&t, &s, want, 3), vec![Change::Keep; 3], "{name} {want:?}");
            }
        }
    }

    #[test]
    fn ego_low_retreats_earlier() {
        for (name, mut s) in scenes() {
            s.me.hp_pct = 35;
            assert_eq!(run(&traits(0, 20), &s, Want::Attack(10), 1)[0], Change::RunAway, "{name}");
            assert_eq!(run(&traits(0, 80), &s, Want::Attack(10), 1)[0], Change::Keep, "{name}");
        }
    }

    #[test]
    fn judgement_high_holds_a_wasted_ultimate() {
        for (name, mut s) in scenes() {
            s.enemies.truncate(1);
            let high = run(&traits(1, 80), &s, Want::Cast(3, Aim::Unit(10)), 200);
            assert_eq!(high[0], Change::Attack(10), "{name}");
            assert_eq!(high[199], Change::Keep, "{name}: never held past the patience");
            assert_eq!(run(&traits(1, 20), &s, Want::Cast(3, Aim::Unit(10)), 1)[0], Change::Keep, "{name}");
            s.enemies = scenes()[0].1.enemies.clone();
            s.enemies[1].point = s.enemies[0].point;
            assert_eq!(run(&traits(1, 80), &s, Want::Cast(3, Aim::Unit(10)), 1)[0], Change::Keep, "{name}: two caught");
        }
    }

    #[test]
    fn positioning_high_kites_between_shots_for_ranged_only() {
        for (name, s) in scenes() {
            let high = run(&traits(2, 90), &s, Want::Attack(10), 1)[0];
            let low = run(&traits(2, 20), &s, Want::Attack(10), 1)[0];
            assert_eq!(low, Change::Keep, "{name}");
            assert_eq!(matches!(high, Change::Move(p) if p.0 < s.me.point.0), s.ranged, "{name}");
        }
    }

    #[test]
    fn control_speed_low_reacts_later() {
        for (name, s) in scenes() {
            let slow = run(&traits(3, 0), &s, Want::Cast(1, Aim::Unit(10)), 30);
            let fast = run(&traits(3, 90), &s, Want::Cast(1, Aim::Unit(10)), 30);
            let first = |v: &[Change]| v.iter().position(|c| *c == Change::Keep);
            assert_eq!(first(&fast), Some(0), "{name}");
            assert_eq!(first(&slow), Some(cast_delay(&traits(3, 0))), "{name}");
            assert_eq!(cast_delay(&traits(3, 0)), 18);
            assert_eq!(cast_delay(&traits(3, 50)), 0);
        }
    }

    #[test]
    fn skill_hit_leads_or_trails_a_moving_target() {
        for (name, mut s) in scenes() {
            s.enemies[0].velocity = (100, 0);
            let spot = s.enemies[0].point;
            let lead = |v: i64| match run(&traits(4, v), &s, Want::Cast(1, Aim::Spot(spot)), 1)[0] {
                Change::Cast(1, Aim::Spot(p)) => p.0 - spot.0,
                Change::Keep => 0,
                c => panic!("{name}: {c:?}"),
            };
            assert!(lead(90) > 0 && lead(20) < 0 && lead(50) == 0, "{name}");
        }
    }

    #[test]
    fn skill_avoid_low_sometimes_misses_the_step() {
        let windows = |v: i64| (0..200).filter(|w| avoids(&traits(5, v), 7, 1, w * 30)).count();
        assert_eq!(windows(50), 200);
        assert_eq!(windows(80), 200);
        let low = windows(10);
        assert!((100..170).contains(&low), "{low}");
    }

    #[test]
    fn aggressive_takes_fights_in_reach_and_low_never_chases() {
        for (name, mut s) in scenes() {
            assert_eq!(run(&traits(6, 90), &s, Want::Move((0, 0)), 1)[0], Change::Attack(10), "{name}");
            assert_eq!(run(&traits(6, 20), &s, Want::Move((0, 0)), 1)[0], Change::Keep, "{name}");
            s.enemies[0].point.0 = s.me.point.0 + s.range * 2 + 10_000;
            assert_eq!(run(&traits(6, 20), &s, Want::Attack(10), 1)[0], Change::Move(s.me.point), "{name}");
            assert_eq!(run(&traits(6, 80), &s, Want::Attack(10), 1)[0], Change::Keep, "{name}");
        }
    }

    #[test]
    fn order_high_waits_for_allies() {
        for (name, s) in scenes() {
            assert!(matches!(run(&traits(7, 90), &s, Want::Attack(10), 1)[0], Change::Move(_)), "{name}");
            assert_eq!(run(&traits(7, 20), &s, Want::Attack(10), 1)[0], Change::Keep, "{name}");
        }
    }

    #[test]
    fn roaming_high_joins_a_nearby_fight() {
        for (name, mut s) in scenes() {
            s.enemies = vec![body(10, -60_000, 100_000)];
            s.allies = vec![body(2, -40_000, 100_000)];
            assert_eq!(run(&traits(8, 90), &s, Want::Move((200_000, 100_000)), 1)[0], Change::Move((-40_000, 100_000)), "{name}");
            assert_eq!(run(&traits(8, 20), &s, Want::Move((200_000, 100_000)), 1)[0], Change::Keep, "{name}");
        }
    }
}
