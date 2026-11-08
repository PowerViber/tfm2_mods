// Thalassa core tests: hand-built frames, the core's own rules. Included by thalassa_tests.rs and by the Lab crate.
use crate::thalassa_core::*;
use crate::thalassa_data as d;

const ME: usize = 1;

fn unit(id: usize, team: usize, x: i64, y: i64) -> Unit {
    Unit { id, team, pos: Point::new(x, y), hp: 1000, max_hp: 1000, radius: 4_000, champion: true, tower: false, visible: true, targetable: true, farmable: false, stunned: false, velocity: Point::default() }
}

fn frame(tick: u64, x: i64, y: i64, units: Vec<Unit>) -> Frame {
    Frame {
        tick,
        owner: ME,
        team: 0,
        alive: true,
        pos: Point::new(x, y),
        hp: 1000,
        max_hp: 1000,
        ap: 100,
        stunned: false,
        ult_unlocked: true,
        units,
        terrain: vec![false; 900],
        profile: [50; 13],
        clone: None,
        basic_hits: vec![],
        manual: true,
        order: None,
    }
}

/// Run `n` ticks, moving her and pulled units like the bridge would, and tracking the clone.
fn run(core: &mut Core, f: &mut Frame, n: u64, log: &mut Vec<Effect>) {
    for _ in 0..n {
        let out = core.step(f);
        for e in &out {
            match e {
                Effect::Glide { to, .. } => f.pos = *to,
                Effect::Pull { target, to, .. } => {
                    if let Some(u) = f.units.iter_mut().find(|u| u.id == *target) {
                        u.pos = *to;
                    }
                }
                Effect::SpawnClone { pos, .. } => f.clone = Some(CloneView { id: 99, pos: *pos, hp: 200 }),
                Effect::CloneMove { to } => {
                    if let Some(c) = f.clone.as_mut() {
                        c.pos = *to;
                    }
                }
                Effect::RemoveClone => f.clone = None,
                Effect::Swap { owner_to, clone_to } => {
                    f.pos = *owner_to;
                    if let Some(c) = f.clone.as_mut() {
                        c.pos = *clone_to;
                    }
                }
                Effect::Damage { target, amount, .. } => {
                    if let Some(u) = f.units.iter_mut().find(|u| u.id == *target) {
                        u.hp -= amount;
                    }
                }
                _ => {}
            }
        }
        log.extend(out);
        f.order = None;
        f.basic_hits.clear();
        f.tick += 1;
    }
}

fn stuns(log: &[Effect], id: usize) -> Vec<u64> {
    log.iter().filter_map(|e| if let Effect::Stun { target, ticks } = e { (*target == id).then_some(*ticks) } else { None }).collect()
}
fn damage_to(log: &[Effect], id: usize) -> i64 {
    log.iter().filter_map(|e| if let Effect::Damage { target, amount, .. } = e { (*target == id).then_some(*amount) } else { None }).sum()
}
fn cast_s1(core: &mut Core, f: &mut Frame, aim: Point, log: &mut Vec<Effect>) {
    f.order = Some(Order::TidePool(aim));
    run(core, f, d::TIDE_POOL_CAST_TICKS as u64 + 1, log);
}

// ------------------------------------------------------------------ passive

#[test]
fn water_mode_heals_then_overheals_up_to_ten_percent() {
    let mut core = Core::default();
    let mut f = frame(0, 300_000, 300_000, vec![]);
    let mut log = vec![];
    let here = f.pos;
    cast_s1(&mut core, &mut f, here, &mut log);
    assert!(log.contains(&Effect::Water { on: true }), "a pool forms below her and she is in water");
    f.hp = 900;
    run(&mut core, &mut f, 60, &mut log);
    assert!(log.iter().any(|e| matches!(e, Effect::Heal { .. })));
    // full HP: the regen becomes a shield, never past 10% max HP
    f.hp = 1000;
    let n = log.len();
    run(&mut core, &mut f, 60 * 30, &mut log);
    let shield: i64 = log[n..].iter().filter_map(|e| if let Effect::Shield { amount, .. } = e { Some(*amount) } else { None }).sum();
    assert!(shield > 0 && shield <= f.max_hp * d::WATER_OVERHEAL_CAP_PCT / 100, "overheal {shield}");
}

#[test]
fn land_is_neutral() {
    let mut core = Core::default();
    let mut f = frame(0, 300_000, 300_000, vec![]);
    let mut log = vec![];
    run(&mut core, &mut f, 120, &mut log);
    assert!(log.is_empty(), "nothing on land without orders: {log:?}");
}

#[test]
fn in_water_harp_bolts_bounce_to_a_second_enemy() {
    let mut core = Core::default();
    let mut f = frame(0, 300_000, 300_000, vec![unit(5, 1, 340_000, 300_000), unit(6, 1, 355_000, 300_000)]);
    let mut log = vec![];
    let here = f.pos;
    cast_s1(&mut core, &mut f, here, &mut log);
    f.basic_hits = vec![5];
    log.clear();
    run(&mut core, &mut f, 1, &mut log);
    assert!(damage_to(&log, 6) > 0, "bounced");
}

// ------------------------------------------------------------------ S1

#[test]
fn tide_pool_on_land_forms_below_her() {
    let mut core = Core::default();
    let mut f = frame(0, 300_000, 300_000, vec![]);
    let mut log = vec![];
    cast_s1(&mut core, &mut f, Point::new(350_000, 300_000), &mut log);
    let p = core.pools[0];
    assert_eq!(p.center, Point::new(300_000, 300_000));
    assert!(!p.whirl);
}

#[test]
fn surge_moves_her_pool_makes_a_bigger_whirlpool_bumps_two_champions_and_leaves_one_below_her() {
    let mut core = Core::default();
    let mut f = frame(0, 300_000, 300_000, vec![unit(5, 1, 350_000, 300_000), unit(6, 1, 356_000, 306_000)]);
    let mut log = vec![];
    let here = f.pos;
    cast_s1(&mut core, &mut f, here, &mut log);
    let first = core.pools[0].id;
    f.tick = core.ready[0];
    log.clear();
    cast_s1(&mut core, &mut f, Point::new(352_000, 302_000), &mut log);
    assert!(!core.pools.iter().any(|p| p.id == first), "the pool she stood in moved");
    let far = core.pools.iter().find(|p| p.center.within(Point::new(352_000, 302_000), 1_000)).expect("surged whirlpool");
    assert!(far.whirl && far.radius > d::WATER_POOL_RADIUS);
    assert!(core.pools.iter().any(|p| p.whirl && p.holds(f.pos)), "a whirlpool below her");
    assert!(stuns(&log, 5).contains(&(d::TIDE_POOL_BUMP_STUN_TICKS as u64)) && stuns(&log, 6).contains(&(d::TIDE_POOL_BUMP_STUN_TICKS as u64)));
    assert!(log.iter().any(|e| matches!(e, Effect::Pull { target: 5, .. })));
    assert!(log.iter().any(|e| matches!(e, Effect::Slow { target: 5, .. })));
}

#[test]
fn one_champion_in_a_surge_is_not_bumped() {
    let mut core = Core::default();
    let mut f = frame(0, 300_000, 300_000, vec![unit(5, 1, 350_000, 300_000)]);
    let mut log = vec![];
    let here = f.pos;
    cast_s1(&mut core, &mut f, here, &mut log);
    f.tick = core.ready[0];
    log.clear();
    cast_s1(&mut core, &mut f, Point::new(350_000, 300_000), &mut log);
    assert!(stuns(&log, 5).is_empty());
    assert!(damage_to(&log, 5) > 0);
}

// ------------------------------------------------------------------ S2

#[test]
fn dive_in_water_opens_a_pool_at_the_target_and_glides_there_unseen() {
    let mut core = Core::default();
    let mut f = frame(0, 300_000, 300_000, vec![]);
    let mut log = vec![];
    let here = f.pos;
    cast_s1(&mut core, &mut f, here, &mut log);
    f.order = Some(Order::Dive(Point::new(400_000, 300_000)));
    log.clear();
    run(&mut core, &mut f, 1, &mut log);
    let to = Point::new(300_000 + d::DIVE_RANGE, 300_000);
    assert!(core.pools.iter().any(|p| p.holds(to)), "a pool at the target");
    assert!(log.iter().any(|e| matches!(e, Effect::Submerge { .. })));
    assert!(log.iter().any(|e| matches!(e, Effect::Glide { to: t, .. } if *t == to)));
}

#[test]
fn slide_on_land_leaves_her_wet_so_water_skills_work() {
    let mut core = Core::default();
    let mut f = frame(0, 300_000, 300_000, vec![unit(5, 1, 340_000, 300_000)]);
    let mut log = vec![];
    f.order = Some(Order::Dive(Point::new(400_000, 300_000)));
    run(&mut core, &mut f, d::DIVE_SLIDE_TICKS as u64 + 1, &mut log);
    assert!(core.water_at(f.pos, f.tick), "wet after the slide");
    // S1 now surges from where she stands, as in water
    log.clear();
    cast_s1(&mut core, &mut f, Point::new(340_000, 300_000), &mut log);
    assert!(core.pools.iter().any(|p| p.whirl), "a surge, not a plain pool");
    run(&mut core, &mut f, d::WATER_WET_TICKS as u64 + 5, &mut log);
    let dry = Point::new(800_000, 800_000);
    assert!(!core.water_at(dry, f.tick));
}

// ------------------------------------------------------------------ S3

#[test]
fn twin_echoes_tide_pool_and_dive_to_the_same_point_and_lasts_twelve_seconds() {
    let mut core = Core::default();
    let mut f = frame(0, 300_000, 300_000, vec![]);
    let mut log = vec![];
    f.order = Some(Order::Twin(Point::new(320_000, 300_000)));
    run(&mut core, &mut f, 1, &mut log);
    assert!(f.clone.is_some());
    log.clear();
    cast_s1(&mut core, &mut f, Point::new(330_000, 330_000), &mut log);
    assert_eq!(core.pools.len(), 2, "her pool and the twin's");
    f.order = Some(Order::Dive(Point::new(320_000, 380_000)));
    run(&mut core, &mut f, 1, &mut log);
    assert!(log.iter().any(|e| matches!(e, Effect::CloneMove { .. })), "the twin dives too");
    run(&mut core, &mut f, d::TWIN_TICKS as u64, &mut log);
    assert!(f.clone.is_none(), "gone after 12 s");
}

#[test]
fn twin_never_copies_the_ult() {
    let mut core = Core::default();
    let mut f = frame(0, 300_000, 300_000, vec![]);
    let mut log = vec![];
    f.order = Some(Order::Twin(Point::new(320_000, 300_000)));
    run(&mut core, &mut f, 1, &mut log);
    f.order = Some(Order::Ult(Point::new(800_000, 300_000)));
    run(&mut core, &mut f, d::ULT_WINDUP_TICKS as u64 + 2, &mut log);
    assert!(core.trident.is_some());
    assert!(!log.iter().any(|e| matches!(e, Effect::CloneMove { .. })));
}

#[test]
fn a_stunned_ally_near_the_twin_gets_a_rescue_pool() {
    let mut core = Core::default();
    let mut ally = unit(7, 0, 330_000, 300_000);
    ally.stunned = true;
    let mut f = frame(0, 300_000, 300_000, vec![ally]);
    let mut log = vec![];
    f.order = Some(Order::Twin(Point::new(320_000, 300_000)));
    run(&mut core, &mut f, d::TWIN_FLY_TICKS as u64 + 3, &mut log);
    assert!(log.contains(&Effect::Cleanse { target: 7 }));
    assert!(log.contains(&Effect::CcImmune { target: 7, ticks: d::TWIN_IMMUNE_TICKS as u64 }));
    assert!(log.iter().any(|e| matches!(e, Effect::Shield { target: 7, .. })));
    assert!(f.clone.is_none(), "the twin became the pool");
}

#[test]
fn a_stunned_enemy_never_triggers_the_rescue() {
    let mut core = Core::default();
    let mut enemy = unit(5, 1, 330_000, 300_000);
    enemy.stunned = true;
    let mut f = frame(0, 300_000, 300_000, vec![enemy]);
    let mut log = vec![];
    f.order = Some(Order::Twin(Point::new(320_000, 300_000)));
    run(&mut core, &mut f, 30, &mut log);
    assert!(!log.iter().any(|e| matches!(e, Effect::Cleanse { .. })));
    assert!(f.clone.is_some());
}

#[test]
fn tide_link_only_on_water_and_one_swap_with_a_whirlpool() {
    let mut core = Core::default();
    let mut f = frame(0, 300_000, 300_000, vec![]);
    let mut log = vec![];
    // on land: no link, the swap is refused
    f.order = Some(Order::Twin(Point::new(320_000, 300_000)));
    run(&mut core, &mut f, 1, &mut log);
    f.order = Some(Order::Swap);
    run(&mut core, &mut f, 1, &mut log);
    assert!(!log.iter().any(|e| matches!(e, Effect::Swap { .. })));
    // on water: linked, one swap, a whirlpool where she was
    let mut core = Core::default();
    let mut f = frame(0, 300_000, 300_000, vec![]);
    let here = f.pos;
    cast_s1(&mut core, &mut f, here, &mut log);
    f.pos = Point::new(330_000, 300_000);
    f.order = Some(Order::Twin(Point::new(310_000, 300_000)));
    run(&mut core, &mut f, 1, &mut log);
    assert!(core.twin.is_some_and(|t| t.linked));
    let before = f.pos;
    log.clear();
    f.order = Some(Order::Swap);
    run(&mut core, &mut f, 1, &mut log);
    assert!(log.iter().any(|e| matches!(e, Effect::Swap { .. })));
    assert!(core.pools.iter().any(|p| p.whirl && p.holds(before)));
    log.clear();
    f.order = Some(Order::Swap);
    run(&mut core, &mut f, 1, &mut log);
    assert!(!log.iter().any(|e| matches!(e, Effect::Swap { .. })), "once per twin");
}

// ------------------------------------------------------------------ Ult

#[test]
fn trident_crosses_the_map_stuns_three_seconds_and_leaves_a_pool() {
    let mut core = Core::default();
    let mut f = frame(0, 50_000, 480_000, vec![unit(5, 1, 900_000, 480_000)]);
    let mut log = vec![];
    f.order = Some(Order::Ult(Point::new(900_000, 480_000)));
    run(&mut core, &mut f, 400, &mut log);
    assert!(stuns(&log, 5).contains(&(d::ULT_STUN_TICKS as u64)));
    assert!(core.pools.iter().any(|p| p.holds(Point::new(900_000, 480_000))));
}

#[test]
fn trident_from_water_stuns_longer_and_leaves_a_whirlpool() {
    let mut core = Core::default();
    let mut f = frame(0, 50_000, 480_000, vec![unit(5, 1, 600_000, 480_000)]);
    let mut log = vec![];
    let here = f.pos;
    cast_s1(&mut core, &mut f, here, &mut log);
    f.order = Some(Order::Ult(Point::new(600_000, 480_000)));
    run(&mut core, &mut f, 300, &mut log);
    assert!(stuns(&log, 5).contains(&(d::ULT_WATER_STUN_TICKS as u64)));
    assert!(core.pools.iter().any(|p| p.whirl && p.holds(Point::new(600_000, 480_000))));
}

#[test]
fn trident_steering_is_capped_at_twenty_five_degrees() {
    let mut core = Core::default();
    let mut f = frame(0, 100_000, 100_000, vec![]);
    let mut log = vec![];
    f.order = Some(Order::Ult(Point::new(900_000, 100_000)));
    run(&mut core, &mut f, d::ULT_WINDUP_TICKS as u64 + 2, &mut log);
    for _ in 0..120 {
        f.order = Some(Order::Steer(Point::new(100_000, 900_000)));
        run(&mut core, &mut f, 1, &mut log);
        if let Some(t) = core.trident {
            assert!(t.spent_mdeg <= d::ULT_CURVE_MILLI_DEGREES);
        }
    }
}
