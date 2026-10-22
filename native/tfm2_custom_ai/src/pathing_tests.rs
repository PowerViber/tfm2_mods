//! Round 141: the engine and the move rewrites reacting to each other over whole stretches of play, not one
//! frame at a time. A scripted engine walks a laner (farming, trading, walking up the lane) while team guidance
//! (universal_ai::guide_step) has its say every tick; the champion walks 1,000 a tick toward whichever wins.
use crate::strategic_core::{distance, Goal, Point};
use crate::universal_ai::{guide_gate, guide_step, GuideIn, GuideState, MIN_STEP};

const SPEED: i64 = 1_000;

fn step(at: Point, to: Point) -> Point {
    let d = distance(at, to);
    if d <= SPEED { to } else { (at.0 + (to.0 - at.0) * SPEED / d, at.1 + (to.1 - at.1) * SPEED / d) }
}

#[derive(Default, Debug)]
struct Run {
    moves: usize,
    guided: usize,
    flips: usize,
    frozen: usize,
    through_wall: usize,
}

/// 600 ticks. `engine(tick, at)` returns (destination, near_farm, attacking).
fn play(
    macro_rating: i64,
    score: i64,
    point: Point,
    start: Point,
    engine: &dyn Fn(usize, Point) -> (Point, bool, bool),
    wall: &dyn Fn(i64, i64) -> bool,
) -> Run {
    let mut st = GuideState::default();
    let mut at = start;
    let mut run = Run::default();
    let mut last_attack = None;
    let mut was_guided = false;
    for tick in 0..600 {
        let (dest, near_farm, attacking) = engine(tick, at);
        if attacking {
            last_attack = Some(tick);
        }
        run.moves += 1;
        let g = GuideIn {
            at,
            dest,
            job: (Goal::Contest.code(), 2, 77, point),
            goal: Goal::Contest,
            point,
            score,
            macro_rating,
            micro_rating: 50,
            nearest_enemy: i64::MAX,
            near_farm,
            attacked_recently: last_attack.is_some_and(|t| tick <= t + 30),
        };
        let out = guide_step(&g, &mut st, wall);
        let to = out.unwrap_or(dest);
        if let Some(p) = out {
            run.guided += 1;
            if distance(p, at) < MIN_STEP {
                run.frozen += 1;
            }
            let n = (distance(at, p) / 1000).max(1);
            if (1..=n).any(|i| wall(at.0 + (p.0 - at.0) * i / n, at.1 + (p.1 - at.1) * i / n)) {
                run.through_wall += 1;
            }
        }
        if out.is_some() != was_guided {
            run.flips += 1;
            was_guided = out.is_some();
        }
        at = step(at, to);
    }
    run
}

fn no_walls(_: i64, _: i64) -> bool {
    false
}

/// A laner farming its wave: the wave front bobs around (300,000, 700,000), with trades now and then.
fn farming(tick: usize, at: Point) -> (Point, bool, bool) {
    let wave = (300_000 + (tick as i64 % 120) * 200, 700_000);
    let trade = tick % 90 < 10;
    if distance(at, wave) > 20_000 { (wave, true, false) } else { ((wave.0 + 5_000, wave.1), true, trade) }
}

#[test]
fn a_farming_laner_keeps_the_engines_moves_at_every_macro() {
    for macro_rating in [20, 50, 80] {
        for score in [60, 100, 140, 200] {
            let r = play(macro_rating, score, (480_000, 480_000), (300_000, 700_000), &farming, &no_walls);
            assert!(r.guided * 10 <= r.moves, "macro {macro_rating} score {score}: {r:?}");
            assert!(r.flips <= 3, "macro {macro_rating} score {score}: {r:?}");
            assert_eq!(r.frozen, 0, "{r:?}");
        }
    }
}

/// An idle laner walking up its lane, away from a strong plan's point.
fn wandering(_: usize, at: Point) -> (Point, bool, bool) {
    ((at.0 - 40_000, at.1 + 40_000), false, false)
}

#[test]
fn guidance_takes_over_once_and_hands_back_for_good() {
    let point = (480_000, 480_000);
    let r = play(80, 200, point, (250_000, 750_000), &wandering, &no_walls);
    assert!(r.guided > 0, "{r:?}");
    assert!(r.flips <= 2, "one take-over, one hand-back: {r:?}");
    assert_eq!(r.frozen, 0, "{r:?}");
}

#[test]
fn weak_plans_never_guide_and_macro_only_nudges_the_bar() {
    assert_eq!(guide_gate(50), 130);
    assert_eq!(guide_gate(0), 130, "low Macro never guides more than average");
    assert_eq!(guide_gate(100), 120);
    let r = play(100, 115, (480_000, 480_000), (250_000, 750_000), &wandering, &no_walls);
    assert_eq!(r.guided, 0, "{r:?}");
}

#[test]
fn guided_steps_never_cross_a_wall() {
    // a wall band across the straight line from the champion to the point, with a gap in the south
    let wall = |x: i64, y: i64| (380_000..412_000).contains(&x) && y < 600_000 || x < 0 || y < 0 || x >= 960_000 || y >= 960_000;
    let r = play(80, 200, (480_000, 480_000), (300_000, 480_000), &wandering, &wall);
    assert_eq!(r.through_wall, 0, "{r:?}");
    assert_eq!(r.frozen, 0, "{r:?}");
}

#[test]
fn route_steps_lead_around_a_wall() {
    let wall = |p: Point| (384_000..416_000).contains(&p.0) && p.1 < 600_000;
    let next = crate::strategic_routes::next_step((300_000, 480_000), (480_000, 480_000), &wall).unwrap();
    assert!(!wall(next));
    assert!(next.1 > 480_000 || next.0 < 384_000, "toward the gap: {next:?}");
    assert_eq!(crate::strategic_routes::next_step((481_000, 481_000), (490_000, 490_000), &wall), None, "same cell");
}

#[test]
fn a_new_assignment_starts_guidance_afresh() {
    let mut st = GuideState::default();
    let mut g = GuideIn {
        at: (200_000, 800_000),
        dest: (100_000, 900_000),
        job: (Goal::Contest.code(), 2, 77, (480_000, 480_000)),
        goal: Goal::Contest,
        point: (480_000, 480_000),
        score: 200,
        macro_rating: 50,
        micro_rating: 50,
        nearest_enemy: i64::MAX,
        near_farm: false,
        attacked_recently: false,
    };
    assert!(guide_step(&g, &mut st, &no_walls).is_some());
    g.attacked_recently = true;
    assert!(guide_step(&g, &mut st, &no_walls).is_none());
    g.attacked_recently = false;
    assert!(guide_step(&g, &mut st, &no_walls).is_none(), "handed back for this assignment");
    g.job.2 = 78;
    assert!(guide_step(&g, &mut st, &no_walls).is_some(), "a new assignment guides again");
}

#[test]
fn strategy_fields_restore_the_coach_and_fill_what_a_goal_leaves() {
    let coach: [(&str, Option<String>); 4] = [
        ("minion_wave", Some("WavePriority".into())),
        ("object_buildup", Some("Split".into())),
        ("object_battle", Some("Poking".into())),
        ("object_finish", Some("KillPriority".into())),
    ];
    let none = crate::universal_ai::wanted_strategy(None, &coach);
    assert_eq!(none.len(), 4);
    assert!(none.iter().all(|(f, v)| coach.iter().any(|(cf, cv)| cf == f && cv.as_deref() == Some(v.as_str()))));
    // Defend sets only the wave field; the other three keep the coach's choices
    let defend = crate::universal_ai::wanted_strategy(Some(Goal::Defend), &coach);
    assert_eq!(defend.iter().find(|(f, _)| f == "minion_wave").unwrap().1, "JoinPriority");
    assert_eq!(defend.iter().find(|(f, _)| f == "object_buildup").unwrap().1, "Split");
    assert_eq!(defend.iter().find(|(f, _)| f == "object_battle").unwrap().1, "Poking");
}

#[test]
fn no_route_around_steve_walls_is_never_a_move_to_where_he_stands() {
    use crate::steve::Wall;
    // a long wall just beside the champion, across the way to the destination
    let w = Wall { ax: 400_000, ay: 300_000, bx: 400_000, by: 700_000, t0: 0, tb: 0, tend: 100_000 };
    for (x, y) in [(395_000, 500_000), (390_000, 500_000), (380_000, 500_000)] {
        if let Some(p) = crate::steve::detour_all(&[w.clone()], 10_000, x, y, 500_000, 500_000) {
            assert!(distance(p, (x, y)) >= MIN_STEP, "from ({x}, {y}): {p:?}");
        }
    }
}

mod dispatch {
    use crate::game_sense_core::Ratings;
    use crate::strategic_core::*;

    fn actor(id: usize, role: u8, at: Point) -> Actor {
        Actor {
            id, athlete: id, team: 0, point: at, hp: 1000, max_hp: 1000, speed: 1000, power: 100, range: 40000,
            role, controlled: false, ultimate: false, visible: true, ratings: Ratings([50; 4]), judgement: 50, ego: 50,
        }
    }
    fn target(id: usize, kind: u8, team: usize, at: Point) -> Target {
        Target { id, kind, team, point: at, hp: 20000, max_hp: 20000, loss: 0, visible: true, value: 30 }
    }

    #[test]
    fn far_members_and_laners_on_a_live_wave_stay_put() {
        let epic = target(50, 2, 2, (480_000, 480_000));
        let wave = target(60, 0, 1, (150_000, 820_000));
        let actors = [
            actor(1, 1, (470_000, 500_000)), // jungler, close
            actor(2, 2, (500_000, 460_000)), // mid, close, no wave near
            actor(3, 3, (140_000, 830_000)), // bot laner on its wave, far
            actor(4, 0, (700_000, 700_000)), // laner close-ish, 240+ ticks away at this speed
        ];
        let plans = evaluate(&View {
            routes: None, team: 0, actors: &actors, targets: &[epic, wave], home: (96_000, 864_000),
            recent_advantage: 0, unseen_risk: 0, blocked: &|_, _| false,
        });
        let plan = plans.iter().find(|p| p.target == 50).expect("an epic plan");
        let sent: Vec<_> = plan.jobs.iter().map(|j| j.actor).collect();
        assert!(sent.contains(&1) && sent.contains(&2), "{sent:?}");
        assert!(!sent.contains(&3), "the laner on its wave stays: {sent:?}");
    }

    #[test]
    fn one_enemy_near_a_tower_is_not_a_defence_call() {
        let mut tower = target(70, 4, 0, (200_000, 760_000));
        tower.hp = 2000;
        tower.max_hp = 2000;
        let mut lone = actor(9, 0, (230_000, 760_000));
        lone.team = 1;
        let actors = [actor(1, 0, (240_000, 780_000)), lone.clone()];
        let view = |actors: &[Actor], t: &[Target]| {
            evaluate(&View {
                routes: None, team: 0, actors, targets: t, home: (96_000, 864_000),
                recent_advantage: 0, unseen_risk: 0, blocked: &|_, _| false,
            })
        };
        assert!(view(&actors, &[tower.clone()]).iter().all(|p| p.goal != Goal::Defend));
        let mut second = lone.clone();
        second.id = 10;
        let actors = [actor(1, 0, (240_000, 780_000)), lone, second];
        assert!(view(&actors, &[tower]).iter().any(|p| p.goal == Goal::Defend));
    }

    #[test]
    fn a_withdrawal_steps_back_toward_home_instead_of_walking_to_base() {
        let mut low = actor(1, 2, (480_000, 480_000));
        low.hp = 200;
        let mut home = target(80, 7, 0, (96_000, 864_000));
        home.hp = 1;
        home.max_hp = 1;
        let mut low2 = actor(2, 3, (500_000, 480_000));
        low2.hp = 200;
        let plans = evaluate(&View {
            routes: None, team: 0, actors: &[low, low2], targets: &[home], home: (96_000, 864_000),
            recent_advantage: 0, unseen_risk: 0, blocked: &|_, _| false,
        });
        let job = plans[0].jobs.iter().find(|j| j.actor == 1).unwrap();
        assert_eq!(job.job, Job::Retreat);
        assert!(distance(job.point, (480_000, 480_000)) <= RETREAT_STEP + 1, "{:?}", job.point);
    }
}
