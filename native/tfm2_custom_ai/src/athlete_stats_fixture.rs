//! Do the athlete stats change how a match goes? A deterministic macro rollout on the real team planner.
//!
//! Two teams of five play 20 minutes on a three-lane map. Waves walk the lanes, camps respawn, the
//! epic and the serpent sit in the river. Every 2 s each team runs `strategic_core::evaluate` (the same
//! call the game makes) and its athletes walk to the jobs it hands out; without a job they hold their
//! lane. Fights, clears and takes are resolved by a simple power model that never reads a rating, so
//! any difference between the teams comes from the decisions the ratings and profile values change.
//! This is a model of the macro layer, not the engine: it measures direction, not exact numbers.

use crate::game_sense_core::Ratings;
use crate::strategic_core::{self as core, distance, Actor, Goal, Point, Target};

const STEP: i64 = 60; // ticks per step (1 s)
const MINUTES: i64 = 20;
const SIGHT: i64 = 70_000;
const FIGHT: i64 = 40_000;
const PLAN_EVERY: i64 = 2;
const HOME: [Point; 2] = [(60_000, 900_000), (900_000, 60_000)];
const EPIC: Point = (330_000, 330_000);
const SERPEN: Point = (630_000, 630_000);
const CAMPS: [Point; 4] = [(250_000, 520_000), (420_000, 700_000), (520_000, 250_000), (700_000, 420_000)];

/// Lane paths from team 0's base to team 1's: top, mid, bottom.
fn lane_point(lane: usize, f: i64) -> Point {
    let f = f.clamp(0, 1000);
    let (a, b, c) = match lane {
        0 => (HOME[0], (60_000, 60_000), HOME[1]),
        1 => (HOME[0], (480_000, 480_000), HOME[1]),
        _ => (HOME[0], (900_000, 900_000), HOME[1]),
    };
    let lerp = |p: Point, q: Point, t: i64| (p.0 + (q.0 - p.0) * t / 500, p.1 + (q.1 - p.1) * t / 500);
    if f <= 500 { lerp(a, b, f) } else { lerp(b, c, f - 500) }
}

#[derive(Clone, Copy)]
pub struct Side {
    pub ratings: [i64; 4],
    pub judgement: i64,
    pub ego: i64,
}

impl Side {
    pub fn even() -> Self { Side { ratings: [50; 4], judgement: 50, ego: 50 } }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Report {
    pub objectives: [u32; 2],
    pub first_objective_s: [Option<i64>; 2],
    pub waves: [u32; 2],
    pub camps: [u32; 2],
    pub deaths: [u32; 2],
    pub unseen_deaths: [u32; 2],
    pub fights_ahead: [u32; 2],
    pub fights: u32,
    pub rotations: [u32; 2],
    pub towers: [u32; 2],
}

struct Unit { team: usize, point: Point, hp: i64, max_hp: i64, power: i64, speed: i64, role: u8, dead_until: i64, goal: Option<Point>, lane: usize }
struct Wave { team: usize, lane: usize, f: i64, hp: i64 }
struct Monster { point: Point, kind: u8, hp: i64, max_hp: i64, back_at: i64 }

fn rng(seed: &mut u64) -> i64 {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 7;
    *seed ^= *seed << 17;
    (*seed % 1000) as i64
}

pub fn play(sides: [Side; 2], seed: u64) -> Report {
    let mut s = seed.wrapping_mul(0x9e37_79b9_7f4a_7c15) | 1;
    let roles = [0u8, 1, 2, 3, 4];
    let lane_of = |role: u8| match role { 0 => 0, 3 | 4 => 2, _ => 1 };
    let mut units: Vec<Unit> = (0..10)
        .map(|i| {
            let team = i / 5;
            let role = roles[i % 5];
            let jitter = rng(&mut s) * 20;
            Unit {
                team, role, lane: lane_of(role),
                point: (HOME[team].0 + jitter, HOME[team].1 - jitter),
                hp: 1000, max_hp: 1000, power: 100 + rng(&mut s) / 50, speed: 450,
                dead_until: 0, goal: None,
            }
        })
        .collect();
    let mut monsters: Vec<Monster> = CAMPS.iter().map(|&p| Monster { point: p, kind: 1, hp: 1500, max_hp: 1500, back_at: 0 }).collect();
    monsters.push(Monster { point: EPIC, kind: 2, hp: 20_000, max_hp: 20_000, back_at: 300 });
    monsters.push(Monster { point: SERPEN, kind: 3, hp: 10_000, max_hp: 10_000, back_at: 300 });
    let mut towers = [[3i64; 3]; 2];
    let mut waves: Vec<Wave> = vec![];
    let mut r = Report::default();
    let mut last_clear: [[i64; 3]; 2] = [[-100; 3]; 2];
    let mut prev_visible = [[true; 10]; 2];
    for t in 0..MINUTES * 60 {
        if t % 30 == 0 {
            for team in 0..2 {
                for lane in 0..3 {
                    waves.push(Wave { team, lane, f: if team == 0 { 60 } else { 940 }, hp: 600 });
                }
            }
        }
        for m in monsters.iter_mut().filter(|m| m.hp <= 0 && t >= m.back_at) {
            m.hp = m.max_hp;
        }
        // Vision: an enemy is seen when any living ally is within sight of it.
        let alive = |u: &Unit| u.dead_until <= t;
        let seen = |team: usize, p: Point, units: &[Unit]| {
            units.iter().any(|a| a.team == team && a.dead_until <= t && distance(a.point, p) <= SIGHT)
        };
        let mut visible = [[false; 10]; 2];
        for team in 0..2 {
            for (i, u) in units.iter().enumerate() {
                visible[team][i] = u.team == team || (alive(u) && seen(team, u.point, &units));
            }
        }
        // Plans.
        if t % PLAN_EVERY == 0 {
            for team in 0..2 {
                let actors: Vec<Actor> = units
                    .iter()
                    .enumerate()
                    .map(|(i, u)| Actor {
                        id: i + 1, athlete: i + 1, team: u.team, point: u.point,
                        hp: if alive(u) { u.hp } else { 0 }, max_hp: u.max_hp, speed: u.speed,
                        power: u.power, range: 40_000, role: u.role, controlled: false, ultimate: true,
                        visible: visible[team][i],
                        ratings: Ratings(sides[u.team].ratings),
                        judgement: sides[u.team].judgement, ego: sides[u.team].ego,
                    })
                    .collect();
                let mut targets = vec![Target { id: 100, kind: 7, team, point: HOME[team], hp: 1, max_hp: 1, loss: 0, visible: true, value: 0 }];
                for (i, w) in waves.iter().enumerate().filter(|(_, w)| w.team != team && w.hp > 0) {
                    let p = lane_point(w.lane, w.f);
                    if seen(team, p, &units) || distance(p, HOME[team]) < 500_000 {
                        targets.push(Target { id: 200 + i, kind: 0, team: w.team, point: p, hp: w.hp, max_hp: 600, loss: 0, visible: true, value: 10 });
                    }
                }
                for (i, m) in monsters.iter().enumerate().filter(|(_, m)| m.hp > 0) {
                    let value = match m.kind { 2 => 35, 3 => 25, _ => 8 };
                    targets.push(Target { id: 900 + i, kind: m.kind, team: 2, point: m.point, hp: m.hp, max_hp: m.max_hp, loss: 0, visible: true, value });
                }
                for lane in 0..3 {
                    if towers[1 - team][lane] > 0 {
                        let f = if team == 0 { 1000 - towers[1 - team][lane] * 120 } else { towers[1 - team][lane] * 120 };
                        targets.push(Target { id: 950 + lane, kind: 5, team: 1 - team, point: lane_point(lane, f), hp: towers[1 - team][lane], max_hp: 3, loss: 0, visible: true, value: 15 });
                    }
                }
                let unseen = (0..10).filter(|&i| units[i].team != team && alive(&units[i]) && !visible[team][i]).count() as i64;
                let plans = core::evaluate(&core::View {
                    routes: None, team, actors: &actors, targets: &targets, home: HOME[team],
                    recent_advantage: 0, unseen_risk: unseen, blocked: &|_, _| false,
                });
                for u in units.iter_mut().filter(|u| u.team == team) {
                    u.goal = None;
                }
                if let Some(plan) = plans.first().filter(|p| p.score > 0) {
                    for job in &plan.jobs {
                        let u = &mut units[job.actor - 1];
                        let lane = u.lane;
                        if matches!(plan.goal, Goal::Wave | Goal::Camp | Goal::Start | Goal::Contest | Goal::Finish | Goal::Reinforce)
                            && distance(job.point, lane_point(lane, 500)) > 200_000
                            && t - last_clear[team][lane] <= 20
                        {
                            r.rotations[team] += 1;
                        }
                        u.goal = Some(job.point);
                    }
                }
            }
        }
        // Movement.
        for u in units.iter_mut() {
            if !alive(u) {
                if t == u.dead_until { u.hp = u.max_hp; u.point = HOME[u.team]; }
                continue;
            }
            let goal = u.goal.unwrap_or_else(|| {
                let front = waves.iter().filter(|w| w.team == u.team && w.lane == u.lane && w.hp > 0)
                    .map(|w| w.f).max_by_key(|f| if u.team == 0 { *f } else { -*f });
                lane_point(u.lane, front.unwrap_or(if u.team == 0 { 400 } else { 600 }))
            });
            let d = distance(u.point, goal);
            let step = u.speed * STEP / 60 * 25;
            if d > 0 {
                let k = step.min(d);
                u.point = (u.point.0 + (goal.0 - u.point.0) * k / d, u.point.1 + (goal.1 - u.point.1) * k / d);
            }
            if distance(u.point, HOME[u.team]) < 40_000 { u.hp = (u.hp + 80).min(u.max_hp); }
        }
        // Fights: everyone near an enemy hits the nearest one.
        let mut damage = [0i64; 10];
        let mut fighting = [false; 10];
        for i in 0..10 {
            if !alive(&units[i]) { continue; }
            let near = (0..10).filter(|&j| units[j].team != units[i].team && alive(&units[j]) && distance(units[i].point, units[j].point) <= FIGHT)
                .min_by_key(|&j| (units[j].hp, distance(units[i].point, units[j].point)));
            if let Some(j) = near { damage[j] += units[i].power * 3 / 2; fighting[i] = true; fighting[j] = true; }
        }
        if fighting.iter().any(|f| *f) {
            r.fights += 1;
            let n = |team: usize| (0..10).filter(|&i| fighting[i] && units[i].team == team).count();
            if n(0) > n(1) { r.fights_ahead[0] += 1 } else if n(1) > n(0) { r.fights_ahead[1] += 1 }
        }
        for i in 0..10 {
            if damage[i] == 0 || !alive(&units[i]) { continue; }
            units[i].hp -= damage[i];
            if units[i].hp <= 0 {
                let team = units[i].team;
                r.deaths[team] += 1;
                let killers_seen = (0..10).any(|j| units[j].team != team && distance(units[i].point, units[j].point) <= FIGHT && prev_visible[team][j]);
                if !killers_seen { r.unseen_deaths[team] += 1; }
                units[i].dead_until = t + 20 + t / 60;
                units[i].goal = None;
            }
        }
        prev_visible = visible;
        // Waves, camps and objectives.
        for w in waves.iter_mut().filter(|w| w.hp > 0) {
            let p = lane_point(w.lane, w.f);
            let hit: i64 = units.iter().filter(|u| u.team != w.team && alive(u) && distance(u.point, p) <= 45_000).map(|u| u.power).sum();
            w.hp -= hit;
            if w.hp <= 0 {
                let team = 1 - w.team;
                r.waves[team] += 1;
                last_clear[team][w.lane] = t;
                continue;
            }
            let enemy_tower = towers[1 - w.team][w.lane];
            let tower_f = if w.team == 0 { 1000 - enemy_tower * 120 } else { enemy_tower * 120 };
            let at_tower = enemy_tower > 0 && (if w.team == 0 { w.f >= tower_f } else { w.f <= tower_f });
            if at_tower {
                w.hp -= 60;
                if w.hp <= 0 { towers[1 - w.team][w.lane] -= 1; r.towers[w.team] += 1; }
            } else {
                w.f += if w.team == 0 { 12 } else { -12 };
            }
        }
        waves.retain(|w| w.hp > 0 && (0..=1000).contains(&w.f));
        for m in monsters.iter_mut().filter(|m| m.hp > 0) {
            let by_team = |team: usize| -> i64 {
                units.iter().filter(|u| u.team == team && alive(u) && distance(u.point, m.point) <= 45_000).map(|u| u.power).sum()
            };
            let (a, b) = (by_team(0), by_team(1));
            if a + b == 0 { m.hp = (m.hp + m.max_hp / 50).min(m.max_hp); continue; }
            m.hp -= a + b;
            if m.hp <= 0 {
                let team = if a >= b { 0 } else { 1 };
                m.back_at = t + if m.kind == 1 { 60 } else { 300 };
                if m.kind == 1 { r.camps[team] += 1 } else {
                    r.objectives[team] += 1;
                    r.first_objective_s[team].get_or_insert(t);
                }
            }
        }
    }
    r
}

/// Sums reports over seeds, with each side playing both bases half the time.
pub fn series(a: Side, b: Side, seeds: u64) -> Report {
    let mut sum = Report::default();
    for seed in 1..=seeds {
        let swap = seed % 2 == 0;
        let r = play(if swap { [b, a] } else { [a, b] }, seed);
        let (x, y) = if swap { (1, 0) } else { (0, 1) };
        for (dst, src) in [(0, x), (1, y)] {
            sum.objectives[dst] += r.objectives[src];
            sum.waves[dst] += r.waves[src];
            sum.camps[dst] += r.camps[src];
            sum.deaths[dst] += r.deaths[src];
            sum.unseen_deaths[dst] += r.unseen_deaths[src];
            sum.fights_ahead[dst] += r.fights_ahead[src];
            sum.rotations[dst] += r.rotations[src];
            sum.towers[dst] += r.towers[src];
            if let Some(f) = r.first_objective_s[src] {
                let e = sum.first_objective_s[dst].get_or_insert(0);
                *e += f;
            }
        }
        sum.fights += r.fights;
    }
    sum
}

#[cfg(test)]
mod tests {
    use super::*;

    const SEEDS: u64 = 96;

    fn json(label: &str, level: i64, r: &Report) -> String {
        format!(
            "{{\"stat\":\"{label}\",\"level\":{level},\"objectives\":{:?},\"waves\":{:?},\"camps\":{:?},\"deaths\":{:?},\"unseenDeaths\":{:?},\"fightsAhead\":{:?},\"fights\":{},\"rotations\":{:?},\"towers\":{:?}}}",
            r.objectives, r.waves, r.camps, r.deaths, r.unseen_deaths, r.fights_ahead, r.fights, r.rotations, r.towers
        )
    }

    #[test]
    fn rollout_is_deterministic_and_even_sides_are_balanced() {
        let even = Side::even();
        assert_eq!(play([even, even], 7), play([even, even], 7));
        let r = series(even, even, SEEDS);
        let gap = |x: [u32; 2]| (x[0] as i64 - x[1] as i64).abs() * 100 / (x[0] + x[1]).max(1) as i64;
        assert!(r.waves[0] + r.waves[1] > 0 && r.fights > 0, "{r:?}");
        assert!(gap(r.waves) <= 15, "{r:?}");
    }

    /// Prints one JSON line per stat and level (team A at that level, team B at 50).
    /// `tools/evaluate-game-sense.py` runs this and compares the result with reference games.
    #[test]
    #[ignore]
    fn athlete_stats_table() {
        let names = ["Macro", "Micro", "Map Awareness", "Strategy"];
        for (k, name) in names.iter().enumerate() {
            for level in [20, 50, 80] {
                let mut a = Side::even();
                a.ratings[k] = level;
                println!("STATS {}", json(name, level, &series(a, Side::even(), SEEDS)));
            }
        }
        for (name, set) in [("judgement", 0usize), ("ego", 1)] {
            for level in [20, 50, 80] {
                let mut a = Side::even();
                if set == 0 { a.judgement = level } else { a.ego = level }
                println!("STATS {}", json(name, level, &series(a, Side::even(), SEEDS)));
            }
        }
    }

    #[test]
    fn every_rating_changes_some_outcome() {
        for k in 0..4 {
            let (mut low, mut high) = (Side::even(), Side::even());
            low.ratings[k] = 20;
            high.ratings[k] = 80;
            let (l, h) = (series(low, Side::even(), 8), series(high, Side::even(), 8));
            assert_ne!(l, h, "rating {k} changed nothing");
        }
    }
}
