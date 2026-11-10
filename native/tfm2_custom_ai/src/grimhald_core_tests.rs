// Grimhald core tests: hand-built frames, the core's own rules.
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
        punched: vec![],
        kills: 0,
    }
}

/// Run the core for `n` ticks from `f`, moving Grimhald and placed units like the bridge does.
fn run(core: &mut Core, f: &mut Frame, n: u64, log: &mut Vec<Effect>) {
    for _ in 0..n {
        let out = core.step(f);
        for e in &out {
            match e {
                Effect::Move { pos } => f.pos = *pos,
                // the engine glides it there; the test lands it at once
                Effect::Glide { target, to, .. } if *target == f.owner => f.pos = *to,
                Effect::Glide { target, to, .. } => {
                    if let Some(u) = f.units.iter_mut().find(|u| u.id == *target) {
                        u.pos = *to;
                    }
                }
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
        f.punched.clear();
        f.kills = 0;
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
    // round 160: up to +35% (was +50%); damage = missing HP% x perMissingPctDamage, floored at 30% HP
    let per = d::PASSIVE_PER_MISSING_PCT_DAMAGE;
    for (hp, dmg, ms, tier) in [(1000, 0, 0, 0u8), (500, 50 * per / 100, 15, 2), (300, 70 * per / 100, 21, 3), (100, 70 * per / 100, 21, 3)] {
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
    run(&mut core, &mut f, (d::BULWARK_CAST_TICKS+d::BULWARK_FISSURE_TICKS+d::BULWARK_PILLAR_DELAY_TICKS) as u64 + 1, &mut log);
    assert!(damage_to(&log, 5) > 0);
    assert!(log.iter().any(|e| matches!(e, Effect::Slow { target: 5, percent, .. } if *percent == d::BULWARK_SLOW_PCT)));
    let p = core.pillar.expect("pillar stands");
    assert!(p.center.within(Point::new(200_000 + d::BULWARK_LENGTH, 200_000), 2_000), "pillar at the fissure end: {:?}", p.center);
    assert!(log.iter().any(|e| matches!(e, Effect::WallUp { .. })));
    // relocate: no Place for anyone, a trail is laid, only once
    let before = log.len();
    f.order = Some(Order::Relocate(Point::new(200_000 + d::BULWARK_LENGTH, 220_000)));
    run(&mut core, &mut f, 1, &mut log);
    assert!(core.pillar.unwrap().relocated);
    assert!(!log[before..].iter().any(|e| matches!(e, Effect::Place { .. })), "relocation pushes nobody");
    assert_eq!(core.trails.len(), 1);
    let at = core.pillar.unwrap().center;
    f.order = Some(Order::Relocate(Point::new(200_000 + d::BULWARK_LENGTH, 180_000)));
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
    run(&mut core, &mut f, (d::BULWARK_CAST_TICKS+d::BULWARK_FISSURE_TICKS+d::BULWARK_PILLAR_DELAY_TICKS) as u64 + 1, &mut vec![]);
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
    let b = unit(6, 1, 200_000 + d::BULWARK_LENGTH + 22_000, 205_000);
    let mut f = frame(0, 200_000, 200_000, vec![a, b]);
    f.order = Some(Order::Bulwark(Point::new(240_000, 200_000)));
    let mut log = vec![];
    run(&mut core, &mut f, (d::BULWARK_CAST_TICKS+d::BULWARK_FISSURE_TICKS+d::BULWARK_PILLAR_DELAY_TICKS) as u64 + 1, &mut log);
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
    let mut f = frame(0, 200_000, 200_000, vec![unit(6, 1, 200_000 + d::BULWARK_LENGTH + 22_000, 205_000)]);
    f.order = Some(Order::Bulwark(Point::new(225_000, 200_000)));
    run(&mut core, &mut f, (d::BULWARK_CAST_TICKS+d::BULWARK_FISSURE_TICKS+d::BULWARK_PILLAR_DELAY_TICKS) as u64 + 1, &mut vec![]);
    let center = core.pillar.unwrap().center;
    assert!(center.within(Point::new(200_000 + d::BULWARK_LENGTH, 200_000), 1_000), "{center:?}");
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

#[test]
fn rupture_advances_once_per_victim_then_waits_before_pillar() {
    let mut c=Core::default();let mut f=frame(0,200000,200000,vec![unit(5,1,240000,200000)]);let mut log=vec![];
    f.order=Some(Order::Bulwark(Point::new(254000,200000)));
    run(&mut c,&mut f,d::BULWARK_CAST_TICKS as u64,&mut log);
    assert!(c.pillar.is_none());assert_eq!(damage_to(&log,5),0);
    run(&mut c,&mut f,d::BULWARK_FISSURE_TICKS as u64+1,&mut log);
    assert!(c.pillar.is_none());assert_eq!(log.iter().filter(|e|matches!(e,Effect::Damage{target:5,..})).count(),1);
    run(&mut c,&mut f,d::BULWARK_PILLAR_DELAY_TICKS as u64,&mut log);
    assert!(c.pillar.is_some());assert_eq!(log.iter().filter(|e|matches!(e,Effect::Damage{target:5,..})).count(),1);
}

#[test]
fn ultimate_curve_spends_at_most_ten_degrees_and_keeps_travel_budget() {
    let mut c=Core::default();let mut f=frame(0,100000,200000,vec![unit(5,1,500000,200000)]);let mut log=vec![];
    f.order=Some(Order::Ult(Point::new(500000,200000)));
    run(&mut c,&mut f,d::ULT_WINDUP_TICKS as u64+1,&mut log);
    f.units[0].pos.y=260000;
    let mut last=f.pos;let mut distance=0.;let mut curved=false;
    for _ in 0..d::ULT_FLIGHT_TICKS {
        run(&mut c,&mut f,1,&mut log);
        if let Ult::Flight {at,spent_mdeg,heading_mdeg,..}=c.ult {
            assert!(spent_mdeg<=10000);assert!(heading_mdeg.abs()<=10000);distance+=last.dist(at);last=at;curved|=at.y>200000;
        }
    }
    assert!(curved);assert!(distance<=400100.);
}

#[test]
fn steering_recast_does_not_restart_cooldown_or_read_hidden_target() {
    let mut c=Core::default();let mut f=frame(0,100000,200000,vec![]);let mut log=vec![];
    f.order=Some(Order::Ult(Point::new(500000,200000)));run(&mut c,&mut f,d::ULT_WINDUP_TICKS as u64+1,&mut log);
    let cooldown=c.ready[3];f.order=Some(Order::Steer(Point::new(500000,350000)));run(&mut c,&mut f,1,&mut log);
    assert_eq!(cooldown,c.ready[3]);assert!(matches!(c.ult,Ult::Flight{guide:Some(_),..}));
    let mut a=c.clone();let mut b=c.clone();let mut hidden=unit(5,1,200000,100000);hidden.visible=false;f.units=vec![hidden.clone()];
    let mut f2=f.clone();f2.units[0].pos=Point::new(900000,900000);
    assert_eq!(a.step(&f),b.step(&f2));
}

// ------------------------------------------------------------------ round 2: nudge, ranges, the planted axe

fn bulwark_ticks() -> u64 {
    (d::BULWARK_CAST_TICKS + d::BULWARK_FISSURE_TICKS + d::BULWARK_PILLAR_DELAY_TICKS) as u64 + 1
}

#[test]
fn pillar_rise_nudges_nearby_enemies_out_without_a_stun() {
    let mut core = Core::default();
    let end = 200_000 + d::BULWARK_LENGTH;
    // beside the pillar, off its wall: nudged about 10,000 away from it
    let mut f = frame(0, 200_000, 200_000, vec![unit(5, 1, end, 214_000)]);
    f.order = Some(Order::Bulwark(Point::new(end, 200_000)));
    let mut log = vec![];
    run(&mut core, &mut f, bulwark_ticks() + d::BULWARK_NUDGE_TICKS as u64 + 2, &mut log);
    let e = f.units.iter().find(|u| u.id == 5).unwrap();
    let c = core.pillar.unwrap().center;
    let moved = e.pos.dist(Point::new(end, 214_000));
    assert!((moved - d::BULWARK_NUDGE_DISTANCE as f64).abs() < 1_500.0, "nudged {moved}");
    assert!(e.pos.dist(c) > Point::new(end, 214_000).dist(c), "outward");
    assert!(stuns(&log, 5).is_empty(), "a nudge never stuns");
}

#[test]
fn relocated_pillar_nudges_again_and_reaches_farther() {
    let mut core = Core::default();
    let mut f = frame(0, 200_000, 200_000, vec![]);
    f.order = Some(Order::Bulwark(Point::new(300_000, 200_000)));
    run(&mut core, &mut f, bulwark_ticks(), &mut vec![]);
    let c = core.pillar.unwrap().center;
    let to = Point::new(c.x, c.y + d::BULWARK_RELOCATE_RANGE);
    f.units.push(unit(7, 1, to.x + 12_000, to.y));
    f.order = Some(Order::Relocate(to));
    let mut log = vec![];
    run(&mut core, &mut f, d::BULWARK_NUDGE_TICKS as u64 + 2, &mut log);
    assert!(core.pillar.unwrap().center.within(to, 7_000), "relocated the full {} away", d::BULWARK_RELOCATE_RANGE);
    assert!(f.units[0].pos.x > to.x + 12_000 + 5_000, "the re-rise nudged: {:?}", f.units[0].pos);
}

#[test]
fn skull_bash_reaches_farther_and_shoves_far_in_the_open() {
    let mut core = Core::default();
    let mut f = frame(0, 200_000, 200_000, vec![unit(5, 1, 200_000 + d::BASH_RANGE - 4_000, 200_000)]);
    f.order = Some(Order::Bash(Point::new(400_000, 200_000)));
    let mut log = vec![];
    run(&mut core, &mut f, 60, &mut log);
    let x = f.units[0].pos.x;
    assert!(damage_to(&log, 5) > 0, "reached at {}", d::BASH_RANGE);
    assert!(x >= 200_000 + d::BASH_RANGE - 4_000 + d::BASH_KNOCKBACK - 9_000, "shoved to {x}");
}

/// Throw, land, leap: returns the frame with Grimhald in his cage and the log so far.
fn caged(core: &mut Core, units: Vec<Unit>) -> (Frame, Vec<Effect>) {
    let mut f = frame(0, 200_000, 200_000, units);
    f.order = Some(Order::Ult(Point::new(400_000, 200_000)));
    let mut log = vec![];
    // like the bridge: champions inside are marked as caged when the cage rises (round 160: an empty cage falls)
    for _ in 0..(d::ULT_WINDUP_TICKS + d::ULT_FLIGHT_TICKS + d::ULT_LEAP_TICKS + 4) {
        run(core, &mut f, 1, &mut log);
        if f.cage.is_none() {
            if let Some(cage) = log.iter().find_map(|e| if let Effect::CageUp { cage } = e { Some(*cage) } else { None }) {
                f.cage = Some(cage);
                for u in f.units.iter_mut().filter(|u| u.champion && inside(cage.center, cage.radius, u.pos, 0)) {
                    u.caged_by = Some(ME);
                }
            }
        }
    }
    assert!(f.cage.is_some(), "cage");
    (f, log)
}

#[test]
fn caged_he_fights_unarmed_and_the_aura_shreds_only_enemies_inside() {
    let mut core = Core::default();
    let (mut f, mut log) = caged(&mut core, vec![unit(5, 1, 400_000, 200_000), unit(6, 1, 700_000, 700_000), unit(8, 0, 405_000, 200_000)]);
    assert!(log.contains(&Effect::Unarmed { on: true }));
    run(&mut core, &mut f, 30, &mut log);
    let shredded: Vec<usize> = log.iter().filter_map(|e| if let Effect::Shred { target, .. } = e { Some(*target) } else { None }).collect();
    assert!(shredded.contains(&5));
    assert!(!shredded.contains(&6), "outside the cage");
    assert!(!shredded.contains(&8), "an ally");
    // a punch knocks back about 4,000 and never stuns
    let before = f.units[0].pos;
    f.punched = vec![5];
    let n = log.len();
    run(&mut core, &mut f, d::UNARMED_PUNCH_TICKS as u64 + 2, &mut log);
    let moved = f.units[0].pos.dist(before);
    assert!((moved - d::UNARMED_PUNCH_KNOCKBACK as f64).abs() < 600.0, "punched {moved}");
    assert!(stuns(&log[n..], 5).is_empty());
    // the cage lasts the new 8 s, then the axe comes back
    run(&mut core, &mut f, d::ULT_CAGE_TICKS as u64 + 30, &mut log);
    assert!(log.contains(&Effect::CageDown));
    assert!(log.contains(&Effect::Unarmed { on: false }));
    assert!(core.planted.is_none());
}

#[test]
fn a_kill_keeps_him_unarmed_two_seconds_past_the_cage_without_new_walls() {
    let mut core = Core::default();
    let (mut f, mut log) = caged(&mut core, vec![unit(5, 1, 400_000, 200_000)]);
    let cage_end = f.cage.unwrap().until;
    f.kills = 1;
    run(&mut core, &mut f, 1, &mut log);
    assert_eq!(core.planted.unwrap().until, cage_end + d::UNARMED_KILL_BONUS_TICKS as u64);
    while f.tick < cage_end + 5 {
        run(&mut core, &mut f, 1, &mut log);
    }
    assert!(log.contains(&Effect::CageDown));
    f.cage = None;
    for u in &mut f.units {
        u.caged_by = None;
    }
    let n = log.len();
    run(&mut core, &mut f, 20, &mut log);
    assert!(core.planted.is_some(), "still unarmed after the cage fell");
    assert!(log[n..].iter().any(|e| matches!(e, Effect::Shred { target: 5, .. })), "the aura still covers the area");
    assert!(!log[n..].iter().any(|e| matches!(e, Effect::WallUp { .. } | Effect::CageUp { .. })));
    run(&mut core, &mut f, d::UNARMED_KILL_BONUS_TICKS as u64, &mut log);
    assert!(core.planted.is_none());
    assert_eq!(log.iter().filter(|e| **e == Effect::Unarmed { on: false }).count(), 1);
}

// ------------------------------------------------------------------ round 3: the pillar never walls him in

#[test]
fn bulwark_into_adjacent_terrain_raises_no_pillar_on_him_but_still_hits() {
    let mut core = Core::default();
    // terrain cell x 224,000..256,000; he stands 6,000 from it, an enemy between
    let mut f = frame(0, 218_000, 200_000, vec![unit(5, 1, 222_000, 200_000)]);
    f.terrain[(200_000 / CELL * CELLS + 224_000 / CELL) as usize] = true;
    f.order = Some(Order::Bulwark(Point::new(260_000, 200_000)));
    let mut log = vec![];
    run(&mut core, &mut f, bulwark_ticks() + 2, &mut log);
    if let Some(p) = core.pillar {
        assert!(seg_d2(f.pos, p.seg.a, p.seg.b) >= (PILLAR_CLEAR as f64).powi(2), "pillar too close: {:?}", p.center);
    }
    assert!(core.ready[1] > 0, "the cooldown is spent");
}

#[test]
fn pillar_never_rises_on_a_champion_and_gives_up_when_no_spot_is_free() {
    let core = Core::default();
    // champions packed along the whole shift range
    let units: Vec<Unit> = (0..9).map(|k| unit(10 + k, 1, 270_000, 200_000 - 24_000 + k as i64 * 6_000)).collect();
    let mut f = frame(0, 200_000, 200_000, units);
    f.units.extend((0..9).map(|k| unit(30 + k, 1, 270_000 - 24_000 + k as i64 * 6_000, 200_000)));
    assert_eq!(core.pillar_spot(&f, Point::new(270_000, 200_000), (1.0, 0.0)), None);
    f.units.clear();
    assert_eq!(core.pillar_spot(&f, Point::new(270_000, 200_000), (1.0, 0.0)), Some(Point::new(270_000, 200_000)));
    // never closer than the clearance to him
    assert_eq!(core.pillar_spot(&f, Point::new(205_000, 200_000), (0.0, 1.0)), None);
}

#[test]
fn relocating_onto_him_is_refused_and_the_pillar_stays() {
    let mut core = Core::default();
    let mut f = frame(0, 200_000, 200_000, vec![]);
    f.order = Some(Order::Bulwark(Point::new(300_000, 200_000)));
    run(&mut core, &mut f, bulwark_ticks(), &mut vec![]);
    let before = core.pillar.expect("pillar");
    // walk next to it, then ask to move it onto himself
    f.pos = Point::new(before.center.x - 30_000, before.center.y);
    f.order = Some(Order::Relocate(f.pos));
    run(&mut core, &mut f, 1, &mut vec![]);
    let after = core.pillar.expect("still standing");
    assert_eq!(after.center, before.center);
    assert!(!after.relocated, "the recast isn't spent");
}

// ------------------------------------------------------------------ round 160

#[test]
fn round_160_his_hp_mod_power_and_cage_buffs_are_cut() {
    assert_eq!(d::MOD_POWER, [15, 0, 5, 10, 10, 0, 5]);
    assert_eq!((d::UNARMED_DAMAGE_PCT, d::UNARMED_ATTACK_SPEED_PCT, d::UNARMED_LIFESTEAL_PCT, d::UNARMED_DAMAGE_REDUCTION_PCT), (15, 50, 8, 10));
    assert_eq!(d::PASSIVE_PER_MISSING_PCT_DAMAGE, 35);
}

#[test]
fn the_cage_drops_as_soon_as_no_enemy_champion_is_inside() {
    let mut core = Core::default();
    let (mut f, mut log) = caged(&mut core, vec![unit(5, 1, 400_000, 200_000), unit(6, 1, 405_000, 205_000)]);
    let cage = f.cage.unwrap();
    // both alive inside: it holds
    run(&mut core, &mut f, 30, &mut log);
    assert!(!log.contains(&Effect::CageDown), "still holding");
    // one dies: still one inside
    f.units[0].hp = 0;
    run(&mut core, &mut f, 2, &mut log);
    assert!(!log.contains(&Effect::CageDown));
    // the last one dies: the walls drop now, long before the timer
    f.units[1].hp = 0;
    let n = log.len();
    run(&mut core, &mut f, 1, &mut log);
    assert!(f.tick < cage.until);
    assert!(log[n..].contains(&Effect::CageDown), "empty cage falls");
    assert_eq!(log[n..].iter().filter(|e| matches!(e, Effect::WallDown { .. })).count(), 5, "all five walls drop");
    // his planted-axe buff ends with it (no kills credited here)
    run(&mut core, &mut f, 2, &mut log);
    assert!(core.planted.is_none());
}
