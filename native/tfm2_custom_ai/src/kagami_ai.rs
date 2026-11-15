//! Athlete decisions only. Mechanics and automatic lifesaving passives remain in the core.
use crate::kagami_core::*;
use crate::kagami_data as d;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Sighting {
    id: usize,
    at: Point,
    previous: Point,
    tick: u64,
    previous_tick: u64,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct State {
    pub reaction_at: u64,
    pub initialized: bool,
    pub pending: bool,
    pub next_review: u64,
    pub reason: String,
    pub continuation: Vec<String>,
    pub evaluated: usize,
    pub horizon: usize,
    pub selected_score: i64,
    pub sightings: Vec<Sighting>,
    observed: Vec<(usize, i64)>,
    hazards: Vec<usize>,
}
#[derive(Clone)]
struct Candidate {
    order: Order,
    score: i64,
    why: &'static str,
    next: Vec<String>,
}
pub fn cadence(f: &Frame) -> u64 {
    // Neutral preserves the existing six-tick input interval.
    if f.p(MICRO) <= 50 {
        (12 - 6 * f.p(MICRO) / 50) as u64
    } else {
        (6 - 3 * (f.p(MICRO) - 50) / 50) as u64
    }
}
fn hash(f: &Frame, salt: usize) -> i64 {
    let mut n = (f.owner as u64).wrapping_mul(0x9e3779b97f4a7c15) ^ (salt as u64).wrapping_mul(0xbf58476d1ce4e5b9) ^ f.tick / 30;
    n ^= n >> 30;
    n = n.wrapping_mul(0xbf58476d1ce4e5b9);
    (n ^ (n >> 27)) as i64 & i64::MAX
}
pub fn legal(f: &Frame, p: Point) -> bool {
    if !p.on_map() {
        return false;
    }
    if f.terrain.len() != 900 {
        return true;
    }
    // Conservative champion footprint, including adjacent solid cells.
    for (dx, dy) in [(0, 0), (-10000, 0), (10000, 0), (0, -10000), (0, 10000)] {
        let q = Point::new(p.x + dx, p.y + dy);
        if !q.on_map() || f.terrain[(q.y / 32000) as usize * 30 + (q.x / 32000) as usize] {
            return false;
        }
    }
    true
}
fn clear(f: &Frame, a: Point, b: Point) -> bool {
    let n = (a.dist(b) / 4000.0).ceil().max(1.0) as i64;
    (0..=n).all(|i| legal(f, Point::new(a.x + (b.x - a.x) * i / n, a.y + (b.y - a.y) * i / n)))
}
fn remember(c: &mut Core, f: &Frame) {
    let mut novel = !c.ai.initialized;
    for u in f.enemy_champions().take(16) {
        if let Some(s) = c.ai.sightings.iter_mut().find(|s| s.id == u.id) {
            if f.tick > s.tick && s.at != u.pos {
                s.previous = s.at;
                s.previous_tick = s.tick;
            }
            s.at = u.pos;
            s.tick = f.tick;
        } else {
            novel = true;
            c.ai.sightings.push(Sighting {
                id: u.id,
                at: u.pos,
                previous: u.pos,
                tick: f.tick,
                previous_tick: f.tick,
            });
        }
    }
    c.ai.sightings.retain(|s| f.tick.saturating_sub(s.tick) <= d::AI_SIGHTING_TICKS as u64);
    c.ai.sightings.truncate(16);
    for (id, hp, max) in std::iter::once((f.owner, f.hp, f.max_hp)).chain(f.allies().map(|u| (u.id, u.hp, u.max_hp))) {
        if c.ai.observed.iter().find(|a| a.0 == id).is_some_and(|a| a.1 - hp >= max * 8 / 100) {
            novel = true;
        }
    }
    let hazards: Vec<_> = f
        .hazards
        .iter()
        .filter(|h| f.tick < h.until && h.at.within(f.pos, h.radius + 40000))
        .map(|h| h.id)
        .collect();
    novel |= hazards.iter().any(|id| !c.ai.hazards.contains(id));
    c.ai.hazards = hazards;
    c.ai.observed = std::iter::once((f.owner, f.hp)).chain(f.allies().map(|u| (u.id, u.hp))).take(16).collect();
    // Never postpone an already scheduled response, nor add a cast-by-cast delay.
    if novel && !c.ai.pending {
        c.ai.reaction_at = f.tick + (d::AI_REACTION_SLOW_TICKS - (d::AI_REACTION_SLOW_TICKS - d::AI_REACTION_FAST_TICKS) * f.p(CONTROL_SPEED) / 100) as u64;
        c.ai.pending = true;
    }
    c.ai.initialized = true;
}
fn uncertainty(c: &Core, f: &Frame, p: Point) -> i64 {
    c.ai.sightings
        .iter()
        .filter(|s| s.tick < f.tick && !f.enemy_champions().any(|u| u.id == s.id))
        .map(|s| {
            let age = f.tick.saturating_sub(s.tick) as i64;
            let radius = 30000 + age * 1000;
            if p.within(s.at, radius) {
                (f.p(AWARENESS) * (d::AI_SIGHTING_TICKS - age).max(0)) / d::AI_SIGHTING_TICKS.max(1) / 4
            } else {
                0
            }
        })
        .sum()
}
fn hazard(f: &Frame, p: Point) -> i64 {
    f.hazards
        .iter()
        .filter(|h| f.tick < h.until && p.within(h.at, h.radius + 10000))
        .map(|_| 20 + f.p(DODGE) * 3 / 5)
        .sum()
}
fn predicted(c: &Core, f: &Frame, u: &Unit) -> Point {
    let Some(s) = c.ai.sightings.iter().find(|s| s.id == u.id) else {
        return u.pos;
    };
    let dt = s.tick.saturating_sub(s.previous_tick).max(1) as i64;
    let lead = d::AI_LEAD_MIN_PCT + (100 - d::AI_LEAD_MIN_PCT) * f.p(ACCURACY) / 100;
    let error = d::AI_PLACEMENT_ERROR * (100 - f.p(ACCURACY)) / 100;
    let jitter = |salt| if error == 0 { 0 } else { hash(f, salt) % (error * 2 + 1) - error };
    Point::new(
        u.pos.x + ((s.at.x - s.previous.x) * 18 / dt).clamp(-18000, 18000) * lead / 100 + jitter(u.id),
        u.pos.y + ((s.at.y - s.previous.y) * 18 / dt).clamp(-18000, 18000) * lead / 100 + jitter(u.id + 731),
    )
    .clamp_map()
}
pub fn guard(c: &Core, f: &Frame) -> Option<Unit> {
    let radius = 100000 + f.p(ROAMING) * 1200;
    f.allies()
        .filter(|u| !u.sealed && u.targetable && u.pos.within(f.pos, radius))
        .max_by_key(|u| {
            let fight = f.enemies_near(u.pos, d::AI_FIGHT_RANGE) as i64;
            let distance = f.pos.dist(u.pos) as i64 / 4000;
            let rotation = f.p(ROAMING) * fight / 5;
            let macro_cost = if fight == 0 { distance * f.p(MACRO) / 100 } else { 0 };
            (
                u.attack + fight * f.p(ORDER) / 3 + rotation - distance - macro_cost - uncertainty(c, f, u.pos),
                std::cmp::Reverse(u.id),
            )
        })
        .cloned()
}
pub fn clone_spot(c: &Core, f: &Frame) -> Point {
    let base = c.clone_spot(f);
    let Some(g) = guard(c, f) else { return base };
    let target = g.pos.toward(f.pos, 18000 + f.p(POSITIONING) * 180);
    if legal(f, target) && Core::danger_at(f, target) < Core::danger_at(f, base) + 20000 {
        target
    } else {
        base
    }
}
pub fn decide(c: &mut Core, f: &Frame) -> Option<Order> {
    remember(c, f);
    if c.sealed(f.owner) || f.stunned || c.swap.is_some_and(|s| s.kind != SwapKind::StepAlly) {
        return None;
    }
    if c.ai.pending {
        c.ai.reason = format!("Reacting to a new situation: {} ticks", c.ai.reaction_at.saturating_sub(f.tick));
        if f.tick < c.ai.reaction_at {
            return None;
        }
        c.ai.pending = false;
    }
    if f.tick < c.ai.next_review {
        return None;
    }
    c.ai.next_review = f.tick + cadence(f);
    let horizon = (1 + 2 * f.p(STRATEGY) / 100).min(d::AI_MAX_CONTINUATION) as usize;
    let mut candidates = Vec::with_capacity(32);
    let mut push = |order, score, why, next: Vec<String>| {
        if candidates.len() < d::AI_CANDIDATE_LIMIT as usize {
            candidates.push(Candidate { order, score, why, next });
        }
    };
    if f.tick >= c.mend_at {
        for u in std::iter::once(f.me_as_unit())
            .chain(f.allies().cloned())
            .take(16)
            .filter(|u| !u.sealed && u.targetable && u.pos.within(f.pos, d::MEND_RANGE) && Core::hp_pct(u) < d::AI_HEAL_BELOW_PCT)
        {
            let missing = u.max_hp - u.hp;
            let wasted = (c.mend_amount(f) - missing).max(0) * 100 / c.mend_amount(f).max(1);
            let pressure = f.enemies_near(u.pos, d::AI_FIGHT_RANGE) as i64;
            push(
                Order::Mend(u.id),
                160 - Core::hp_pct(&u) + pressure * f.p(JUDGEMENT) / 20 - wasted * f.p(MACRO) / 100,
                "Heal a wounded champion; avoid overheal",
                vec!["Charm the protected carry".into(), "Follow behind cover".into()],
            );
        }
    }
    // Mirror Step's two cooldowns: a rescue and her own swap never wait on each other
    if f.tick >= c.step_ally_at && c.swap.is_none() {
        if let Some(id) = c.rescue_target(f) {
            push(
                Order::StepAlly(id),
                95 + f.p(ORDER) / 4 + f.p(JUDGEMENT) / 5,
                "Clone offers a safer rescue",
                vec!["Mend after rescue".into(), "Charm for disengagement".into()],
            );
        }
    }
    if f.tick >= c.step_self_at && c.swap.is_none() {
        if let Some(p) = c.escape_point(f).filter(|p| legal(f, *p)) {
            push(
                Order::StepPoint(p),
                110 + f.p(DODGE) / 3 - f.p(EGO) / 4,
                "Escape after real damage",
                vec!["Mend after escape".into()],
            );
        }
        if c.clone_alive(f) {
            for u in f.enemy_champions().take(8) {
                let p = predicted(c, f, u);
                if !p.within(f.pos, d::STEP_RANGE) || !legal(f, p) || !clear(f, f.clone.unwrap().pos, p) {
                    continue;
                }
                let support = f
                    .allies()
                    .filter(|a| a.targetable && !a.sealed && a.pos.within(p, 70000) && clear(f, a.pos, p))
                    .count() as i64;
                if support == 0 || f.hp * 100 < f.max_hp * 55 {
                    continue;
                }
                let danger = Core::danger_at(f, p) / 3000;
                let risk = danger * (150 - f.p(EGO)) / 100;
                let judgement_error = (hash(f, u.id + 91) % 41 - 20) * (100 - f.p(JUDGEMENT)) / 100;
                let score = 15 + f.p(AGGRESSIVE) * 3 / 5 + support * f.p(ORDER) / 5 - risk - hazard(f, p) - uncertainty(c, f, p) + judgement_error;
                push(
                    Order::StepPoint(p),
                    score,
                    "Offensive swap: follow-up versus landing risk",
                    vec!["Stun at clone landing".into(), "Charm and follow the carry".into()],
                );
            }
        }
    }
    if f.tick >= c.charm_at {
        for u in f
            .allies()
            .filter(|u| {
                !u.sealed && u.targetable && !c.charmed(u.id, f.tick) && u.pos.within(f.pos, d::CHARM_RANGE) && f.enemies_near(u.pos, d::AI_FIGHT_RANGE) > 0
            })
            .take(16)
        {
            push(
                Order::Charm(u.id),
                45 + u.attack / 10 + f.p(AGGRESSIVE) / 5 + f.p(ORDER) / 5,
                "Buff a fighting carry",
                vec!["Keep safe firing access".into(), "Mend incoming damage".into()],
            );
        }
    }
    c.ai.evaluated = candidates.len();
    c.ai.horizon = horizon;
    // A bounded beam compares authored, legal-action continuations. Revalidate on every acceptance.
    candidates.sort_by_key(|a| std::cmp::Reverse(a.score));
    candidates.truncate(d::AI_BEAM_WIDTH as usize);
    for a in &mut candidates {
        let follow = a.next.len().min(horizon.saturating_sub(1));
        let usable = match a.order {
            Order::StepAlly(_) | Order::StepPoint(_) => f.tick >= c.mend_at || f.tick >= c.charm_at,
            Order::Mend(_) => f.tick >= c.charm_at,
            Order::Charm(_) => f.tick >= c.mend_at,
        };
        if usable {
            a.score += follow as i64 * f.p(STRATEGY) / 5;
        }
    }
    candidates.sort_by_key(|a| std::cmp::Reverse(a.score));
    c.ai.selected_score = candidates.first().map_or(0, |a| a.score);
    let Some(best) = candidates.first().filter(|a| a.score > 35) else {
        c.ai.reason = "Hold resources; no worthwhile legal cast".into();
        c.ai.continuation.clear();
        return None;
    };
    c.ai.reason = best.why.into();
    c.ai.selected_score = best.score;
    c.ai.continuation = best.next.iter().take(horizon.saturating_sub(1)).cloned().collect();
    Some(best.order)
}
pub fn steer(c: &mut Core, f: &Frame) -> Option<Point> {
    if c.ai.pending || f.stunned || f.tick < c.steer_at {
        return None;
    }
    c.steer_at = f.tick + cadence(f);
    // Visible public warnings can prompt movement without replacing accepted attacks.
    if f.hazards.iter().any(|h| f.tick < h.until && f.pos.within(h.at, h.radius + 10000)) {
        let mut options = Vec::new();
        for i in 0..8 {
            let angle = i as f64 * std::f64::consts::TAU / 8.0;
            let p = Point::new(f.pos.x + (angle.cos() * 45000.0) as i64, f.pos.y + (angle.sin() * 45000.0) as i64);
            if legal(f, p) && clear(f, f.pos, p) {
                options.push(p);
            }
        }
        if let Some(p) = options.into_iter().min_by_key(|p| hazard(f, *p) + Core::danger_at(f, *p) / 4000) {
            if hazard(f, p) < hazard(f, f.pos) {
                return Some(p);
            }
        }
    }
    let g = guard(c, f)?;
    if g.pos.within(f.pos, d::AI_GUARD_RANGE) {
        return None;
    }
    let offset = 10000 + f.p(POSITIONING) * 250;
    let p = g.pos.toward(f.pos, offset).clamp_map();
    legal(f, p).then_some(p)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn unit(id: usize, team: usize, x: i64, hp: i64) -> Unit {
        Unit {
            id,
            team,
            pos: Point::new(x, 500000),
            hp,
            max_hp: 1000,
            attack: 180,
            champion: true,
            visible: true,
            targetable: true,
            ..Unit::default()
        }
    }
    fn fixture(variant: i64) -> (Core, Frame) {
        let f = Frame {
            tick: 100,
            owner: 1,
            team: 0,
            alive: true,
            pos: Point::new(400000, 500000),
            hp: 1000,
            max_hp: 1000,
            attack: 100,
            profile: [50; 13],
            home: Point::new(96000, 864000),
            clone: Some(CloneView {
                id: 99,
                pos: Point::new(380000, 500000),
                hp: 350,
                max_hp: 350,
            }),
            units: vec![unit(2, 0, 440000 + variant * 1000, 280), unit(10, 1, 470000 + variant * 1000, 1000)],
            ..Frame::default()
        };
        let mut c = Core {
            clone: Some(99),
            ..Core::default()
        };
        c.ai.initialized = true;
        c.ai.observed = vec![(1, 1000), (2, 280)];
        c.ai.sightings.push(Sighting {
            id: 10,
            at: f.units[1].pos,
            previous: Point::new(465000, 500000),
            tick: 100,
            previous_tick: 95,
        });
        (c, f)
    }
    #[test]
    fn thirteen_values_at_20_and_80_across_three_relevant_fixtures() {
        for stat in 0..13 {
            for variant in 0..3 {
                let metric = |value: i64| -> i64 {
                    let (mut c, mut f) = fixture(variant);
                    f.profile[stat] = value;
                    match stat {
                        CONTROL_SPEED => {
                            c.ai = State::default();
                            decide(&mut c, &f);
                            c.ai.reaction_at as i64
                        }
                        POSITIONING => {
                            f.units[0].pos.x = 520000;
                            steer(&mut c, &f).unwrap().x
                        }
                        ACCURACY => predicted(&c, &f, &f.units[1]).x,
                        MICRO => cadence(&f) as i64,
                        ROAMING => {
                            f.units[0].pos.x = 570000 + variant * 1000;
                            guard(&c, &f).map_or(0, |u| u.id as i64)
                        }
                        AWARENESS => {
                            c.ai.sightings[0].tick = 90;
                            f.units[1].visible = false;
                            uncertainty(&c, &f, Point::new(470000, 500000))
                        }
                        JUDGEMENT => {
                            f.units[0].hp = 350 + variant;
                            c.rescue_target(&f).map_or(0, |u| u as i64)
                        }
                        MACRO => {
                            f.units[0].max_hp = 100;
                            f.units[0].hp = 55;
                            c.ai.observed = vec![(1, 1000), (2, 55)];
                            decide(&mut c, &f);
                            c.ai.selected_score
                        }
                        STRATEGY => {
                            decide(&mut c, &f);
                            c.ai.continuation.len() as i64
                        }
                        EGO | AGGRESSIVE | DODGE => {
                            f.units[0].hp = 1000;
                            c.mend_at = u64::MAX;
                            c.charm_at = u64::MAX;
                            if stat == DODGE {
                                f.hazards.push(Hazard {
                                    id: 1,
                                    at: f.units[1].pos,
                                    radius: 20000,
                                    until: 200,
                                });
                                c.ai.hazards = vec![1];
                            }
                            // The full evaluated entry score is observable even when the hold threshold rejects it.
                            decide(&mut c, &f);
                            c.ai.selected_score
                        }
                        ORDER => {
                            c.mend_at = u64::MAX;
                            decide(&mut c, &f);
                            c.ai.selected_score
                        }
                        _ => unreachable!(),
                    }
                };
                let low = metric(20);
                let high = metric(80);
                assert_ne!(low, high, "stat {stat}, fixture {variant}: {low} vs {high}");
            }
        }
    }
    #[test]
    fn once_per_situation_reaction_is_not_restarted_by_moving_targets_or_followup_casts() {
        let (mut c, mut f) = fixture(0);
        c.ai = State::default();
        assert!(decide(&mut c, &f).is_none());
        let deadline = c.ai.reaction_at;
        for tick in 101..deadline {
            f.tick = tick;
            f.units[1].pos.x += 100;
            assert!(decide(&mut c, &f).is_none());
            assert_eq!(c.ai.reaction_at, deadline);
        }
        f.tick = deadline;
        assert!(matches!(decide(&mut c, &f), Some(Order::Mend(2))));
        c.mend_at = 1000;
        f.tick += cadence(&f);
        assert!(matches!(decide(&mut c, &f), Some(Order::StepAlly(2)) | Some(Order::Charm(2))));
        assert!(!c.ai.pending);
    }
    #[test]
    fn unseen_current_position_and_resources_cannot_change_a_decision() {
        let (c, mut f) = fixture(0);
        f.units[1].visible = false;
        let mut a = c.clone();
        let first = decide(&mut a, &f);
        f.units[1].pos = Point::new(1, 1);
        f.units[1].hp = 1;
        f.units[1].attack = 99999;
        let mut b = c;
        assert_eq!(first, decide(&mut b, &f));
        assert_eq!(a.ai.selected_score, b.ai.selected_score);
    }
    #[test]
    fn manual_commands_and_clone_mechanics_are_identical_across_all_profiles() {
        for stat in 0..13 {
            for variant in 0..3 {
                let (base, mut f) = fixture(variant);
                f.manual = true;
                let (mut low, mut high) = (base.clone(), base);
                for tick in 100..280 {
                    f.tick = tick;
                    f.order = match tick {
                        100 => Some(Order::Mend(2)),
                        120 => Some(Order::Charm(2)),
                        150 => Some(Order::StepAlly(2)),
                        _ => None,
                    };
                    f.profile[stat] = 20;
                    let a = low.step(&f);
                    f.profile[stat] = 80;
                    let b = high.step(&f);
                    assert_eq!(a, b, "manual stat {stat}, fixture {variant}, tick {tick}");
                }
            }
        }
    }
    #[test]
    fn budget_and_terrain_checks_hold_even_for_maximum_strategy() {
        let (mut c, mut f) = fixture(0);
        f.profile[STRATEGY] = 100;
        for i in 3..64 {
            f.units.push(unit(i, 0, 450000 + i as i64 * 30, 200));
        }
        decide(&mut c, &f);
        assert!(c.ai.evaluated <= 64);
        assert!(c.ai.continuation.len() <= 2);
        f.terrain = vec![true; 900];
        f.units[0].pos.x = 520000;
        assert!(steer(&mut c, &f).is_none());
    }
}
