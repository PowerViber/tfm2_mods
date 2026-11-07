//! Grimhald core tests: hand-built frames, the core's own rules.
use crate::grimhald_core::*;
use crate::grimhald_data as d;

const ME: usize = 1;

fn unit(id: usize, team: usize, x: i64, y: i64) -> Unit {
    Unit {
        id,
        team,
        pos: Point::new(x, y),
        hp: 1000,
        max_hp: 1000,
        radius: 4_000,
        champion: true,
        tower: false,
        visible: true,
        targetable: true,
        farmable: false,
        controlled: false,
        velocity: Point::default(),
        caged_by: None,
    }
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
        attack: 100,
        move_speed: 1000,
        hard_cc_ticks: 0,
        ult_unlocked: true,
        units,
        terrain: vec![false; 900],
        walls: vec![],
        profile: [50; 13],
        danger: false,
        cage: None,
        manual: true,
        order: None,
    }
}

/// Run the core for `n` ticks from `f`, moving Grimhald and placed units like the bridge does.
fn run(core: &mut Core, f: &mut Frame, n: u64, log: &mut Vec<Effect>) {
    for _ in 0..n {
        let out = core.step(f);
        for e in &out {
            match e {
                Effect::Move { pos } => f.pos = *pos,
                Effect::Place { target, pos, .. } => {
                    if let Some(u) = f.units.iter_mut().find(|u| u.id == *target) {
                        u.pos = *pos;
                    }
                }
                Effect::Damage { target, amount } => {
                    if let Some(u) = f.units.iter_mut().find(|u| u.id == *target) {
                        u.hp -= amount;
                    }
                }
                _ => {}
            }
        }
        log.extend(out);
        f.order = None;
        f.tick += 1;
    }
}

fn damage_to(log: &[Effect], id: usize) -> i64 {
    log.iter().filter_map(|e| if let Effect::Damage { target, amount } = e { (*target == id).then_some(*amount) } else { None }).sum()
}
fn stuns(log: &[Effect], id: usize) -> Vec<u64> {
    log.iter().filter_map(|e| if let Effect::Stun { target, ticks } = e { (*target == id).then_some(*ticks) } else { None }).collect()
}
fn airborne(log: &[Effect], id: usize) -> Vec<u64> {
    log.iter().filter_map(|e| if let Effect::Airborne { target, ticks } = e { (*target == id).then_some(*ticks) } else { None }).collect()
}

// ------------------------------------------------------------------ geometry

#[test]
fn pentagon_inside_clamp_and_push() {
    let c = Point::new(480_000, 480_000);
    let r = 55_000;
    assert!(inside(c, r, c, 0));
    assert!(!inside(c, r, Point::new(480_000 + 60_000, 480_000), 0));
    let out = Point::new(480_000 + 90_000, 480_000 + 10_000);
    let back = clamp_inside(c, r, out, 11_000);
    assert!(inside(c, r, back, 10_900), "clamped point is inside");
    let inner = Point::new(480_000 + 5_000, 480_000);
    let pushed = push_outside(c, r, inner, 11_000);
    assert!(depth(c, r, pushed) <= -10_900.0, "pushed point is outside");
    // the apothem is r cos 36°
    let apothem = depth(c, r, c);
    assert!((apothem - r as f64 * 36f64.to_radians().cos()).abs() < 50.0);
}

// ------------------------------------------------------------------ passive

#[test]
fn bloodrage_scales_with_missing_hp_and_caps_at_30_percent() {
    let mut core = Core::default();
    for (hp, dmg, ms, tier) in [(1000, 0, 0, 0u8), (500, 25, 15, 2), (300, 35, 21, 3), (100, 35, 21, 3)] {
        let mut f = frame(0, 100_000, 100_000, vec![]);
        f.hp = hp;
        let out = core.step(&f);
        let got = out.iter().find_map(|e| if let Effect::Passive { damage_pct, move_pct, tier } = e { Some((*damage_pct, *move_pct, *tier)) } else { None });
        if let Some(g) = got {
            assert_eq!(g, (dmg, ms, tier), "at {hp} HP");
        } else {
            assert_eq!(core.passive, (dmg, ms, tier));
        }
    }
}

// ------------------------------------------------------------------ S1 Iron Ghost

#[test]
fn ghost_banishes_cleanses_and_takes_the_longer_cooldown_when_it_cleansed() {
    let mut core = Core::default();
    let mut f = frame(100, 200_000, 200_000, vec![]);
    f.hard_cc_ticks = 90;
    f.order = Some(Order::Ghost(Point::new(240_000, 200_000)));
    let mut log = vec![];
    run(&mut core, &mut f, 1, &mut log);
    assert!(log.contains(&Effect::ClearCc));
    assert!(log.iter().any(|e| matches!(e, Effect::Banish { .. })));
    assert!(log.iter().any(|e| matches!(e, Effect::Invulnerable { .. })));
    assert_eq!(core.ready[0], 100 + d::GHOST_CLEANSE_COOLDOWN_TICKS as u64);

    let mut core = Core::default();
    let mut f = frame(100, 200_000, 200_000, vec![]);
    f.order = Some(Order::Ghost(Point::new(240_000, 200_000)));
    run(&mut core, &mut f, 1, &mut vec![]);
    assert_eq!(core.ready[0], 100 + d::GHOST_COOLDOWN_TICKS as u64);
}

#[test]
fn ghost_moves_at_most_max_distance_and_recast_ends_it_with_the_speed_tail() {
    let mut core = Core::default();
    let mut f = frame(0, 200_000, 200_000, vec![]);
    f.order = Some(Order::Ghost(Point::new(400_000, 200_000)));
    let mut log = vec![];
    run(&mut core, &mut f, 20, &mut log);
    assert!(core.ghost.is_some());
    f.order = Some(Order::GhostEnd);
    run(&mut core, &mut f, 1, &mut log);
    assert!(core.ghost.is_none());
    assert!(log.iter().any(|e| matches!(e, Effect::Speed { .. })));
    assert!(core.tail.is_some());
    // no second recast: the ghost is over
    f.order = Some(Order::GhostEnd);
    let before = log.len();
    run(&mut core, &mut f, 1, &mut log);
    assert!(!log[before..].iter().any(|e| matches!(e, Effect::Speed { .. })));
    run(&mut core, &mut f, 120, &mut log);
    assert!(f.pos.x - 200_000 <= d::GHOST_MAX_DISTANCE, "travelled {}", f.pos.x - 200_000);
}

#[test]
fn ghost_reappearing_near_an_enemy_cuts_the_bash_cooldown() {
    let mut core = Core::default();
    core.ready[2] = 1_000;
    let mut f = frame(0, 200_000, 200_000, vec![unit(5, 1, 230_000, 200_000)]);
    f.order = Some(Order::Ghost(Point::new(226_000, 200_000)));
    run(&mut core, &mut f, 61, &mut vec![]);
    assert_eq!(core.ready[2], 1_000 - d::GHOST_RESET_TICKS as u64);
}

// ------------------------------------------------------------------ S2 Stone Bulwark

#[test]
fn bulwark_fissure_slows_and_raises_a_pillar_that_relocates_once_without_pushing() {
    let mut core = Core::default();
    let enemy = unit(5, 1, 220_000, 200_000);
    let mut f = frame(0, 200_000, 200_000, vec![enemy]);
    f.order = Some(Order::Bulwark(Point::new(240_000, 200_000)));
    let mut log = vec![];
    run(&mut core, &mut f, d::BULWARK_CAST_TICKS as u64 + 1, &mut log);
    assert!(damage_to(&log, 5) > 0);
    assert!(log.iter().any(|e| matches!(e, Effect::Slow { target: 5, percent, .. } if *percent == d::BULWARK_SLOW_PCT)));
    let p = core.pillar.expect("pillar stands");
    assert!(p.center.within(Point::new(238_000, 200_000), 2_000), "pillar at the fissure end: {:?}", p.center);
    assert!(log.iter().any(|e| matches!(e, Effect::WallUp { .. })));
    // relocate: no Place for anyone, a trail is laid, only once
    let before = log.len();
    f.order = Some(Order::Relocate(Point::new(238_000, 220_000)));
    run(&mut core, &mut f, 1, &mut log);
    assert!(core.pillar.unwrap().relocated);
    assert!(!log[before..].iter().any(|e| matches!(e, Effect::Place { .. })), "relocation pushes nobody");
    assert_eq!(core.trails.len(), 1);
    let at = core.pillar.unwrap().center;
    f.order = Some(Order::Relocate(Point::new(238_000, 180_000)));
    run(&mut core, &mut f, 1, &mut log);
    assert_eq!(core.pillar.unwrap().center, at, "a second relocation is refused");
    // the pillar's 4 s are not extended
    assert_eq!(core.pillar.unwrap().until, p.until);
}

#[test]
fn bulwark_stops_at_terrain() {
    let mut core = Core::default();
    let mut f = frame(0, 200_000, 200_000, vec![]);
    // terrain cell from x = 224,000 to 256,000
    f.terrain[(200_000 / CELL * CELLS + 224_000 / CELL) as usize] = true;
    f.order = Some(Order::Bulwark(Point::new(250_000, 200_000)));
    run(&mut core, &mut f, d::BULWARK_CAST_TICKS as u64 + 1, &mut vec![]);
    assert!(core.pillar.unwrap().center.x < 224_000);
}

// ------------------------------------------------------------------ S3 Skull Bash

#[test]
fn bash_knocks_back_and_stuns_briefly_without_a_wall() {
    let mut core = Core::default();
    let mut f = frame(0, 200_000, 200_000, vec![unit(5, 1, 212_000, 200_000)]);
    f.order = Some(Order::Bash(Point::new(212_000, 200_000)));
    let mut log = vec![];
    run(&mut core, &mut f, 40, &mut log);
    let e = f.units.iter().find(|u| u.id == 5).unwrap();
    assert!(e.pos.x >= 212_000 + d::BASH_KNOCKBACK - 9_000, "knocked back to {}", e.pos.x);
    assert!(stuns(&log, 5).contains(&(d::BASH_LAND_STUN_TICKS as u64)));
    assert!(!stuns(&log, 5).contains(&(d::BASH_SLAM_STUN_TICKS as u64)));
}

#[test]
fn bash_into_terrain_slams_and_stuns_long() {
    let mut core = Core::default();
    let mut f = frame(0, 200_000, 200_000, vec![unit(5, 1, 210_000, 200_000)]);
    f.terrain[(200_000 / CELL * CELLS + 224_000 / CELL) as usize] = true;
    f.order = Some(Order::Bash(Point::new(210_000, 200_000)));
    let mut log = vec![];
    run(&mut core, &mut f, 40, &mut log);
    assert!(stuns(&log, 5).contains(&(d::BASH_SLAM_STUN_TICKS as u64)));
    assert!(damage_to(&log, 5) >= (d::BASH_BASE + d::BASH_RATIO) + (d::BASH_SLAM_BASE + d::BASH_SLAM_RATIO));
}

#[test]
fn bash_into_the_pillar_shatters_it_with_an_area_knock_up() {
    let mut core = Core::default();
    let a = unit(5, 1, 212_000, 200_000);
    let b = unit(6, 1, 255_000, 205_000);
    let mut f = frame(0, 200_000, 200_000, vec![a, b]);
    f.order = Some(Order::Bulwark(Point::new(240_000, 200_000)));
    let mut log = vec![];
    run(&mut core, &mut f, d::BULWARK_CAST_TICKS as u64 + 1, &mut log);
    assert!(core.pillar.is_some());
    // put the first enemy back in line (the fissure slowed, nothing moved it)
    f.order = Some(Order::Bash(Point::new(212_000, 200_000)));
    log.clear();
    run(&mut core, &mut f, 40, &mut log);
    assert!(core.pillar.is_none(), "the pillar shattered");
    assert!(airborne(&log, 5).contains(&(d::BULWARK_SHATTER_TICKS as u64)));
    assert!(airborne(&log, 6).contains(&(d::BULWARK_SHATTER_TICKS as u64)), "the area knock-up catches the second enemy");
}

#[test]
fn charging_himself_into_the_pillar_shatters_it() {
    let mut core = Core::default();
    let mut f = frame(0, 200_000, 200_000, vec![unit(6, 1, 255_000, 205_000)]);
    f.order = Some(Order::Bulwark(Point::new(225_000, 200_000)));
    run(&mut core, &mut f, d::BULWARK_CAST_TICKS as u64 + 1, &mut vec![]);
    let center = core.pillar.unwrap().center;
    assert!(center.within(Point::new(238_000, 200_000), 1_000), "{center:?}");
    f.pos = Point::new(214_000, 200_000);
    f.order = Some(Order::Bash(center));
    let mut log = vec![];
    run(&mut core, &mut f, 30, &mut log);
    assert!(core.pillar.is_none());
    assert!(airborne(&log, 6).contains(&(d::BULWARK_SHATTER_TICKS as u64)));
}

// ------------------------------------------------------------------ Ult

#[test]
fn ult_that_hits_nobody_returns_the_axe_with_full_cooldown_and_no_cage() {
    let mut core = Core::default();
    let mut f = frame(0, 200_000, 200_000, vec![unit(5, 1, 600_000, 600_000)]);
    f.order = Some(Order::Ult(Point::new(500_000, 200_000)));
    let mut log = vec![];
    run(&mut core, &mut f, 200, &mut log);
    assert!(!log.iter().any(|e| matches!(e, Effect::CageUp { .. })));
    assert_eq!(core.ready[3], d::ULT_COOLDOWN_TICKS as u64);
    assert_eq!(f.pos, Point::new(200_000, 200_000), "he doesn't leap");
}

#[test]
fn ult_reaches_half_the_map_cages_and_leaps_in() {
    let mut core = Core::default();
    let target = unit(5, 1, 650_000, 200_000);
    let mut f = frame(0, 200_000, 200_000, vec![target]);
    f.order = Some(Order::Ult(Point::new(650_000, 200_000)));
    let mut log = vec![];
    run(&mut core, &mut f, (d::ULT_WINDUP_TICKS + d::ULT_FLIGHT_TICKS + d::ULT_LEAP_TICKS + 4) as u64, &mut log);
    assert!(airborne(&log, 5).contains(&(d::ULT_KNOCK_TICKS as u64)));
    let cage = log.iter().find_map(|e| if let Effect::CageUp { cage } = e { Some(*cage) } else { None }).expect("cage");
    assert_eq!(cage.radius, d::ULT_CAGE_RADIUS);
    assert_eq!(log.iter().filter(|e| matches!(e, Effect::WallUp { .. })).count(), 5);
    assert!(f.pos.within(cage.center, 2_000), "he leapt to the axe");
    run(&mut core, &mut f, d::ULT_CAGE_TICKS as u64 + 30, &mut log);
    assert!(log.contains(&Effect::CageDown));
}

#[test]
fn ult_range_is_capped_at_half_the_map() {
    let mut core = Core::default();
    let mut f = frame(0, 100_000, 100_000, vec![]);
    f.order = Some(Order::Ult(Point::new(900_000, 100_000)));
    run(&mut core, &mut f, 1, &mut vec![]);
    let Ult::Windup { aim, .. } = core.ult else { panic!("{:?}", core.ult) };
    assert!(aim.x - 100_000 <= d::ULT_RANGE);
}

#[test]
fn ai_never_throws_into_turret_range() {
    let mut core = Core::default();
    let mut tower = unit(9, 1, 600_000, 200_000);
    tower.tower = true;
    tower.champion = false;
    let mut f = frame(0, 200_000, 200_000, vec![unit(5, 1, 610_000, 200_000), unit(6, 1, 612_000, 210_000), tower]);
    f.manual = false;
    let mut log = vec![];
    run(&mut core, &mut f, 1, &mut log);
    assert!(!matches!(core.ult, Ult::Windup { .. }), "no throw at enemies under their tower");
    // away from the tower: thrown
    let mut core = Core::default();
    let mut f = frame(0, 200_000, 200_000, vec![unit(5, 1, 450_000, 500_000), unit(6, 1, 455_000, 505_000)]);
    f.manual = false;
    run(&mut core, &mut f, 1, &mut vec![]);
    assert!(matches!(core.ult, Ult::Windup { .. }));
}

#[test]
fn ai_cleanses_long_cc_near_enemies_but_not_short_cc() {
    let mut core = Core::default();
    let mut f = frame(0, 200_000, 200_000, vec![unit(5, 1, 230_000, 200_000)]);
    f.manual = false;
    f.ult_unlocked = false;
    f.hard_cc_ticks = 20;
    run(&mut core, &mut f, 1, &mut vec![]);
    assert!(core.ghost.is_none(), "a short stun is waited out");
    f.hard_cc_ticks = 80;
    run(&mut core, &mut f, 1, &mut vec![]);
    assert!(core.ghost.is_some_and(|g| g.cleansed));
}

#[test]
fn nothing_happens_without_orders_or_enemies() {
    let mut core = Core::default();
    let mut f = frame(0, 200_000, 200_000, vec![]);
    f.manual = false;
    let mut log = vec![];
    run(&mut core, &mut f, 300, &mut log);
    assert!(log.iter().all(|e| matches!(e, Effect::Passive { .. })), "{log:?}");
}

#[test]
fn caged_units_outside_the_cage_are_out_of_reach() {
    let mut core = Core::default();
    let mut caged = unit(5, 1, 212_000, 200_000);
    caged.caged_by = Some(77);
    let mut f = frame(0, 200_000, 200_000, vec![caged]);
    f.order = Some(Order::Bash(Point::new(212_000, 200_000)));
    let mut log = vec![];
    run(&mut core, &mut f, 30, &mut log);
    assert_eq!(damage_to(&log, 5), 0);
}

// ------------------------------------------------------------------ glue: the bridge and the cage through a mock host

mod glue {
    use crate::glue_tests::{open_spot, with_sim, Host, Unit as HostUnit};
    use crate::grimhald_cage as cage;
    use crate::grimhald_core::{self as core, Cage, Point};
    use mod_api_stable::{AttackTypeV1, CcKindV1, CcV1, EntityKindV1 as K, StablePassive};
    use std::cell::RefCell;

    fn champ(team: usize, pos: (u64, u64), name: &'static str) -> HostUnit {
        HostUnit { kind: K::Champion, team, pos, hp: 2000, name, targetable: true }
    }
    fn at(host: &RefCell<Host>, i: usize) -> Point {
        let p = host.borrow().units[i].pos;
        Point::new(p.0 as i64, p.1 as i64)
    }
    fn cage_tick(host: &RefCell<Host>, t: usize) {
        host.borrow_mut().tick = t;
        with_sim(host, |sim| {
            let all = crate::champions(sim);
            cage::tick(sim, &all, t);
        });
    }

    #[test]
    fn cage_holds_members_in_and_puts_outsiders_and_minions_out() {
        let _g = crate::walls::TEST_LOCK.lock();
        let (x, y) = open_spot(150_000);
        let c = Point::new(x as i64, y as i64);
        let host = Host::new(vec![
            champ(0, (x, y), "tfm2_custom_grimhald"),
            // a caged enemy that blinked out
            champ(1, (x + 90_000, y), "fighter"),
            // an outsider who got in
            champ(0, (x + 10_000, y + 5_000), "archer"),
            HostUnit { kind: K::Minion, team: 1, pos: (x, y + 20_000), hp: 400, name: "minion", targetable: true },
        ]);
        let cg = Cage { owner: 0, center: c, radius: 55_000, until: 1_000 };
        with_sim(&host, |sim| {
            sim.add_buff(0, &crate::timed(&cage::cage_buff(&cg), 900));
            sim.add_buff(0, &crate::timed(&cage::member_buff(0), 900));
            sim.add_buff(1, &crate::timed(&cage::member_buff(0), 900));
        });
        cage_tick(&host, 100);
        assert!(core::inside(c, 55_000, at(&host, 1), 10_000), "the caged enemy is back inside: {:?}", at(&host, 1));
        assert!(core::depth(c, 55_000, at(&host, 2)) <= -10_000.0, "the outsider is out: {:?}", at(&host, 2));
        assert!(core::depth(c, 55_000, at(&host, 3)) <= -29_000.0, "the minion is out of reach: {:?}", at(&host, 3));
        // can_interact: in-cage pairs only
        with_sim(&host, |sim| {
            let all = crate::champions(sim);
            assert!(cage::can_interact(&all, 0, 1));
            assert!(!cage::can_interact(&all, 0, 2));
            assert!(!cage::can_interact(&all, 1, 2));
        });
        // after it falls: nothing moves anyone
        host.borrow_mut().units[1].pos = (x + 90_000, y);
        cage_tick(&host, 1_000);
        assert_eq!(at(&host, 1), Point::new(x as i64 + 90_000, y as i64));
    }

    #[test]
    fn his_own_named_cc_is_not_read_as_held_but_a_real_stun_is() {
        let _g = crate::walls::TEST_LOCK.lock();
        let (x, y) = open_spot(70_000);
        let host = Host::new(vec![champ(0, (x, y), "tfm2_custom_grimhald"), champ(1, (x + 200_000, y), "fighter")]);
        let mut g = crate::grimhald::Grimhald::default();
        host.borrow_mut().tick = 10;
        with_sim(&host, |sim| {
            let mut own = CcV1::of_kind(CcKindV1::Stun, 30);
            own.set_name("grh_stun");
            sim.apply_cc(0, &own);
        });
        let f = with_sim(&host, |sim| g.frame(sim, 0));
        assert_eq!(f.hard_cc_ticks, 0);
        with_sim(&host, |sim| sim.apply_cc(0, &CcV1::of_kind(CcKindV1::Stun, 30)));
        let f = with_sim(&host, |sim| g.frame(sim, 0));
        assert!(f.hard_cc_ticks > 0);
    }

    #[test]
    fn grimhald_fights_through_the_host_and_lands_skill_damage() {
        let _g = crate::walls::TEST_LOCK.lock();
        let (x, y) = open_spot(80_000);
        let host = Host::new(vec![champ(0, (x, y), "tfm2_custom_grimhald"), champ(1, (x + 20_000, y), "fighter")]);
        let mut g = crate::grimhald::Grimhald::default();
        with_sim(&host, |sim| g.on_spawn(sim, 0, 0));
        for t in 100..400 {
            host.borrow_mut().tick = t;
            with_sim(&host, |sim| g.on_update(sim, t as u64, 0, 0));
        }
        let h = host.borrow();
        assert!(h.damage.iter().any(|d| d.0 == 0 && d.1 == 1 && d.3 == AttackTypeV1::Skill.code()), "{:?}", h.damage);
        // the pillar and its wall buff showed up at some point, or the bash moved him
        assert!(h.moves.iter().any(|m| m.0 == 0 || m.0 == 1), "something moved");
    }
}
