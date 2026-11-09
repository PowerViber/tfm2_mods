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
        shield: 0,
        stunned: false,
        ult_unlocked: true,
        units,
        terrain: vec![false; 900],
        profile: [50; 13],
        clone: None,
        echo: None,
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
                Effect::Shield { target, amount, .. } if *target == ME => f.shield += amount,
                Effect::CloneMove { to, .. } => {
                    if let Some(c) = f.clone.as_mut() {
                        c.pos = *to;
                    }
                }
                Effect::RemoveClone => f.clone = None,
                Effect::SpawnEcho { pos, .. } => f.echo = Some(CloneView { id: 98, pos: *pos, hp: f.max_hp * d::ULT_ECHO_HP_PCT / 100 }),
                Effect::EchoMove { to, .. } => {
                    if let Some(c) = f.echo.as_mut() {
                        c.pos = *to;
                    }
                }
                Effect::RemoveEcho => f.echo = None,
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
    log.iter()
        .filter_map(|e| match e {
            Effect::Stun { target, ticks } | Effect::TridentHit { target, ticks, .. } => (*target == id).then_some(*ticks),
            _ => None,
        })
        .collect()
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
fn twin_water_dive_copies_committed_endpoint_and_arrival_clock_at_both_ranges() {
    for far in [false, true] {
        let mut core = Core::default();
        let mut f = frame(0, 100_000, 480_000, vec![]);
        let mut log = vec![]; let here=f.pos;
        cast_s1(&mut core,&mut f,here,&mut log);
        f.order=Some(Order::Twin(Point::new(100_000,510_000)));
        run(&mut core,&mut f,1,&mut log);
        let target=Point::new(if far {900_000} else {200_000},480_000);
        if far { core.pools.push(Pool{id:900,center:target,radius:d::WATER_POOL_RADIUS,until:1000,whirl:false,whirl_until:0}); }
        f.order=Some(Order::Dive(target));
        let start=f.tick; let effects=core.step(&f);
        let (to,ticks)=effects.iter().find_map(|e|if let Effect::Glide{to,ticks}=e{Some((*to,*ticks))}else{None}).unwrap();
        let copy=effects.iter().find_map(|e|if let Effect::CloneMove{to,ticks}=e{Some((*to,*ticks))}else{None}).unwrap();
        assert_eq!(copy,(to,ticks),"same endpoint and arrival, far={far}");
        assert_eq!(core.twin.unwrap().moving_until,start+ticks);
        assert_eq!(ticks,(d::DIVE_DIVE_TICKS+(here.dist(to) as i64/20_000)) as u64,"travel formula retained");
        // thalassa-1.3: even her own far pool is clamped to Dive range (the trident leaves a clone instead)
        assert_eq!(to.x,here.x+d::DIVE_RANGE);
    }
}

#[test]
fn twin_dive_still_resolves_a_legal_landing_when_approach_is_blocked() {
    let mut core=Core::default();let mut f=frame(0,300_000,300_000,vec![]);let mut log=vec![];let here=f.pos;
    cast_s1(&mut core,&mut f,here,&mut log);
    f.order=Some(Order::Twin(Point::new(300_000,330_000)));run(&mut core,&mut f,1,&mut log);
    let target=Point::new(360_000,300_000);
    f.terrain[((target.y/CELL)*CELLS+target.x/CELL) as usize]=true;
    f.order=Some(Order::Dive(target));let effects=core.step(&f);
    let (to,ticks)=effects.iter().find_map(|e|if let Effect::Glide{to,ticks}=e{Some((*to,*ticks))}else{None}).unwrap();
    let (copy,clock)=effects.iter().find_map(|e|if let Effect::CloneMove{to,ticks}=e{Some((*to,*ticks))}else{None}).unwrap();
    assert!(!f.terrain_at(to));assert!(!f.terrain_at(copy));assert_eq!(clock,ticks);
}

#[test]
fn trident_moves_at_revised_speed_with_same_water_bonus_and_damage() {
    for water in [false,true] {
        let mut core=Core::default();let mut f=frame(0,100_000,480_000,vec![]);let mut log=vec![];
        if water {let here=f.pos;cast_s1(&mut core,&mut f,here,&mut log);}
        f.order=Some(Order::Ult(Point::new(900_000,480_000)));
        run(&mut core,&mut f,d::ULT_WINDUP_TICKS as u64+1,&mut log);
        let before=core.trident.unwrap();run(&mut core,&mut f,1,&mut log);let after=core.trident.unwrap();
        let speed=if water {2600}else{2000};
        assert_eq!(after.speed,speed);assert_eq!(after.at.x-before.at.x,speed);
        assert_eq!(d::ULT_BASE,150);assert_eq!(d::ULT_AP_RATIO,90);
    }
}

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
fn surge_moves_her_pool_matches_whirlpool_size_bumps_two_champions_and_leaves_one_below_her() {
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
    assert!(far.whirl && far.radius == d::WATER_POOL_RADIUS);
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
    run(&mut core, &mut f, 500, &mut log);
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

// ------------------------------------------------------------------ review fixes

#[test]
fn overheal_refills_absorbed_shield_and_expires_after_leaving_water() {
    let mut core = Core::default();
    let mut f = frame(0, 300_000, 300_000, vec![]);
    let mut log = vec![];
    let here = f.pos;
    cast_s1(&mut core, &mut f, here, &mut log);
    run(&mut core, &mut f, 300, &mut log);
    let cap = f.max_hp * d::WATER_OVERHEAL_CAP_PCT / 100;
    assert!(core.overheal <= cap && core.overheal > 0);
    // a hit takes the shield away: it no longer counts, and the regen fills it again
    f.shield = 0;
    run(&mut core, &mut f, 1, &mut log);
    assert_eq!(core.overheal, 0, "absorbed shielding no longer counts");
    let n = log.len();
    run(&mut core, &mut f, 60, &mut log);
    assert!(log[n..].iter().any(|e| matches!(e, Effect::Shield { target: ME, .. })), "refilled");
    // out of the water: about 2 s later nothing of it counts any more
    f.pos = Point::new(800_000, 800_000);
    run(&mut core, &mut f, (d::WATER_OVERHEAL_LINGER_TICKS + 40) as u64, &mut log);
    assert!(core.overheal_parts.is_empty(), "expired");
    assert_eq!(core.overheal, 0);
}

#[test]
fn dive_stays_in_range_even_into_her_far_pool() {
    let mut core = Core::default();
    let mut f = frame(0, 100_000, 480_000, vec![]);
    let mut log = vec![];
    let here = f.pos;
    cast_s1(&mut core, &mut f, here, &mut log);
    // the trident's pool, across the map
    f.order = Some(Order::Ult(Point::new(900_000, 480_000)));
    run(&mut core, &mut f, 550, &mut log);
    let far = core.pools.iter().find(|p| p.center.x > 700_000).copied().expect("trident pool");
    f.order = Some(Order::Dive(far.center));
    run(&mut core, &mut f, 1, &mut log);
    assert_eq!(f.pos.x, 100_000 + d::DIVE_RANGE, "clamped, not a global dive: {:?}", f.pos);
    // a plain far point: clamped
    let mut core = Core::default();
    let mut f = frame(0, 100_000, 480_000, vec![]);
    cast_s1(&mut core, &mut f, here, &mut log);
    f.order = Some(Order::Dive(Point::new(900_000, 480_000)));
    run(&mut core, &mut f, 1, &mut log);
    assert_eq!(f.pos.x, 100_000 + d::DIVE_RANGE);
}

#[test]
fn twin_and_swap_never_land_on_terrain() {
    let mut core = Core::default();
    let mut f = frame(0, 300_000, 300_000, vec![]);
    // terrain cell x 320,000..352,000 at this row
    f.terrain[(300_000 / CELL * CELLS + 320_000 / CELL) as usize] = true;
    let mut log = vec![];
    f.order = Some(Order::Twin(Point::new(330_000, 300_000)));
    run(&mut core, &mut f, 1, &mut log);
    let c = f.clone.expect("twin").pos;
    assert!(!f.terrain_at(c), "twin on open ground: {c:?}");
    // a swap whose destination is terrain stops short of it
    let mut core = Core::default();
    let mut f = frame(0, 300_000, 300_000, vec![]);
    let here = f.pos;
    cast_s1(&mut core, &mut f, here, &mut log);
    f.order = Some(Order::Twin(Point::new(322_000, 300_000)));
    run(&mut core, &mut f, 1, &mut log);
    assert_eq!(f.clone.expect("twin").pos.x, 322_000);
    // the twin's spot turns out to be terrain (a wall rose there)
    f.terrain[(300_000 / CELL * CELLS + 322_000 / CELL) as usize] = true;
    log.clear();
    f.order = Some(Order::Swap);
    run(&mut core, &mut f, 1, &mut log);
    let to = log.iter().find_map(|e| if let Effect::Swap { owner_to, .. } = e { Some(*owner_to) } else { None }).expect("swap");
    assert!(!f.terrain_at(to), "{to:?}");
}

#[test]
fn twin_flights_are_timed_glides_drawn_every_tick() {
    let mut core = Core::default();
    let mut ally = unit(7, 0, 330_000, 300_000);
    ally.stunned = true;
    let mut f = frame(0, 300_000, 300_000, vec![ally]);
    let mut log = vec![];
    f.order = Some(Order::Twin(Point::new(310_000, 300_000)));
    run(&mut core, &mut f, 12, &mut log);
    assert!(log.iter().any(|e| matches!(e, Effect::CloneMove { ticks, .. } if *ticks == d::TWIN_FLY_TICKS as u64)));
    // standing still it is drawn every 6 ticks (2 in 12); in flight every tick
    let bodies = log.iter().filter(|e| matches!(e, Effect::Fx { name, .. } if name.starts_with("tls_twin_p"))).count();
    assert!(bodies >= 6, "drawn every tick in flight ({bodies})");
}

// ------------------------------------------------------------------ the twin plays on its own (thalassa-1.2)

fn clone_moves(log: &[Effect]) -> Vec<Point> {
    log.iter().filter_map(|e| if let Effect::CloneMove { to, .. } = e { Some(*to) } else { None }).collect()
}
fn twin_hits(log: &[Effect], id: usize) -> Vec<i64> {
    log.iter().filter_map(|e| if let Effect::Damage { target, amount, from_clone: true } = e { (*target == id).then_some(*amount) } else { None }).collect()
}

#[test]
fn twin_walks_into_range_and_fires_its_own_bolts_without_her() {
    let mut core = Core::default();
    let mut f = frame(0, 300_000, 300_000, vec![unit(7, 1, 392_000, 300_000)]);
    let mut log = vec![];
    f.order = Some(Order::Twin(Point::new(330_000, 300_000)));
    run(&mut core, &mut f, 120, &mut log);
    assert!(!clone_moves(&log).is_empty(), "it walked");
    let hits = twin_hits(&log, 7);
    assert!(!hits.is_empty(), "it attacked on its own while she attacked nothing");
    let hers = (d::BASIC_BASE + d::BASIC_AP_RATIO * 100 / 100) as i64;
    assert!(hits.iter().all(|&h| h == hers * d::TWIN_COPY_PCT / 100), "{hits:?} vs {hers}");
    assert!(hits.len() as u64 <= 120 / d::BASIC_COOLDOWN_TICKS as u64 + 1, "on its own clock");
}

#[test]
fn twin_no_longer_echoes_her_bolts() {
    let mut core = Core::default();
    // the enemy is in her range but out of the twin's sight
    let mut f = frame(0, 300_000, 300_000, vec![unit(7, 1, 250_000, 300_000)]);
    let mut log = vec![];
    f.order = Some(Order::Twin(Point::new(330_000, 300_000)));
    run(&mut core, &mut f, 1, &mut log);
    for _ in 0..5 {
        f.basic_hits = vec![7];
        run(&mut core, &mut f, 1, &mut log);
    }
    // the twin only joins once it has walked within its own range (it may start walking, but no echo per her hit)
    assert!(twin_hits(&log, 7).len() <= 1, "{:?}", twin_hits(&log, 7));
}

#[test]
fn twin_kites_back_from_a_close_enemy() {
    let mut core = Core::default();
    let mut f = frame(0, 300_000, 300_000, vec![unit(7, 1, 342_000, 300_000)]);
    let mut log = vec![];
    f.order = Some(Order::Twin(Point::new(330_000, 300_000)));
    run(&mut core, &mut f, 13, &mut log);
    let c = f.clone.expect("twin").pos;
    assert!(c.x < 330_000, "backed off: {c:?}");
}

#[test]
fn twin_never_walks_past_the_leash_and_comes_back_when_alone() {
    let mut core = Core::default();
    let mut f = frame(0, 300_000, 300_000, vec![unit(7, 1, 420_000, 300_000)]);
    let mut log = vec![];
    f.order = Some(Order::Twin(Point::new(330_000, 300_000)));
    run(&mut core, &mut f, 300, &mut log);
    for to in clone_moves(&log) {
        assert!(to.within(f.pos, d::TWIN_LEASH + 1), "leash: {to:?}");
    }
    // the enemy leaves; she walks off: it follows her back
    f.units.clear();
    f.pos = Point::new(260_000, 300_000);
    run(&mut core, &mut f, 120, &mut log);
    let c = f.clone.expect("twin").pos;
    assert!(c.within(f.pos, 22_000), "back near her: {c:?}");
    assert!(clone_moves(&log).iter().all(|p| !f.terrain_at(*p)));
}

#[test]
fn twin_still_casts_skills_only_when_she_does() {
    let mut core = Core::default();
    let mut f = frame(0, 300_000, 300_000, vec![unit(7, 1, 380_000, 300_000), unit(8, 1, 385_000, 305_000)]);
    let mut log = vec![];
    f.order = Some(Order::Twin(Point::new(330_000, 300_000)));
    run(&mut core, &mut f, 200, &mut log);
    let surges = |l: &[Effect]| l.iter().filter(|e| matches!(e, Effect::Fx { name, .. } if name == "tls_surge")).count();
    assert_eq!(surges(&log), 0, "no skill of its own");
}

#[test]
fn a_water_twin_rescues_thalassa_anywhere_instantly_a_land_twin_does_not() {
    for water in [true, false] {
        let mut core = Core::default();
        let mut f = frame(0, 300_000, 300_000, vec![]);
        let mut log = vec![];
        if water {
            let here = f.pos;
            cast_s1(&mut core, &mut f, here, &mut log);
        }
        f.order = Some(Order::Twin(Point::new(320_000, 300_000)));
        run(&mut core, &mut f, 1, &mut log);
        assert!(f.clone.is_some());
        f.pos = Point::new(800_000, 300_000);
        f.stunned = true;
        log.clear();
        run(&mut core, &mut f, 1, &mut log);
        let cleansed = log.iter().any(|e| matches!(e, Effect::Cleanse { target: ME }));
        assert_eq!(cleansed, water, "water twin: {water}");
        if water {
            assert!(f.clone.is_none(), "the twin is used up");
            assert!(log.iter().any(|e| matches!(e, Effect::CcImmune { target: ME, .. })));
            assert!(core.pools.iter().any(|p| p.holds(f.pos)), "rescue pool under her");
        } else {
            run(&mut core, &mut f, 30, &mut log);
            assert!(!log.iter().any(|e| matches!(e, Effect::Cleanse { .. })));
        }
    }
}

#[test]
fn allies_keep_the_rescue_range_and_the_flight() {
    let mut core = Core::default();
    let mut far = unit(7, 0, 620_000, 300_000);
    far.stunned = true;
    let mut f = frame(0, 300_000, 300_000, vec![far]);
    let mut log = vec![];
    let here = f.pos;
    cast_s1(&mut core, &mut f, here, &mut log);
    f.order = Some(Order::Twin(Point::new(320_000, 300_000)));
    run(&mut core, &mut f, 30, &mut log);
    assert!(!log.iter().any(|e| matches!(e, Effect::Cleanse { target: 7 })), "300,000 away: no rescue");
    f.units[0].pos = Point::new(345_000, 300_000);
    log.clear();
    run(&mut core, &mut f, 30, &mut log);
    assert!(log.iter().any(|e| matches!(e, Effect::CloneMove { ticks, .. } if *ticks == d::TWIN_FLY_TICKS as u64)), "flew");
    assert!(log.iter().any(|e| matches!(e, Effect::Cleanse { target: 7 })));
}

// ------------------------------------------------------------------ the trident's clone of her (thalassa-1.3)

/// Throw the ult at `aim` and run until the trident lands and its clone rises; returns the landing log.
fn land_trident(core: &mut Core, f: &mut Frame, aim: Point, log: &mut Vec<Effect>) -> Vec<Effect> {
    f.order = Some(Order::Ult(aim));
    for _ in 0..1200 {
        let mut step = vec![];
        run(core, f, 1, &mut step);
        let landed = step.iter().any(|e| matches!(e, Effect::SpawnEcho { .. }));
        log.extend(step.iter().cloned());
        if landed {
            return step;
        }
    }
    panic!("the trident never landed");
}
fn echo_moves(log: &[Effect]) -> Vec<(Point, u64)> {
    log.iter().filter_map(|e| if let Effect::EchoMove { to, ticks } = e { Some((*to, *ticks)) } else { None }).collect()
}

#[test]
fn the_trident_leaves_a_clone_of_her_for_8_seconds() {
    let mut core = Core::default();
    let mut f = frame(0, 100_000, 480_000, vec![]);
    let mut log = vec![];
    let landing = land_trident(&mut core, &mut f, Point::new(950_000, 480_000), &mut log);
    let e = f.echo.expect("echo spawned");
    assert!(core.pools.iter().any(|p| p.holds(e.pos)), "it rises from the landing pool: {:?}", e.pos);
    assert!(!f.terrain_at(e.pos));
    assert!(landing.iter().any(|x| matches!(x, Effect::SpawnEcho { ticks, .. } if *ticks == d::ULT_ECHO_TICKS as u64)));
    assert_eq!(e.hp, f.max_hp * d::ULT_ECHO_HP_PCT / 100);
    assert!(f.pos.x == 100_000, "she stays where she is");
    run(&mut core, &mut f, d::ULT_ECHO_TICKS as u64 + 1, &mut log);
    assert!(f.echo.is_none() && core.echo.is_none(), "gone after 8 s");
}

#[test]
fn a_new_trident_replaces_the_standing_clone() {
    let mut core = Core::default();
    let mut f = frame(0, 100_000, 480_000, vec![]);
    let mut log = vec![];
    land_trident(&mut core, &mut f, Point::new(950_000, 480_000), &mut log);
    core.ready[3] = 0;
    log.clear();
    let landing = land_trident(&mut core, &mut f, Point::new(100_000, 950_000), &mut log);
    assert!(log.iter().any(|x| matches!(x, Effect::RemoveEcho)), "the old one is removed");
    assert_eq!(landing.iter().filter(|x| matches!(x, Effect::SpawnEcho { .. })).count(), 1);
    assert!(f.echo.expect("new echo").pos.y > 800_000);
}

#[test]
fn the_clone_surges_its_own_tide_pool_and_bumps_at_half_damage() {
    let mut core = Core::default();
    let mut f = frame(0, 100_000, 480_000, vec![unit(7, 1, 700_000, 480_000), unit(8, 1, 706_000, 490_000)]);
    let mut log = vec![];
    land_trident(&mut core, &mut f, Point::new(700_000, 480_000), &mut log);
    log.clear();
    run(&mut core, &mut f, 60, &mut log);
    assert!(log.iter().any(|x| matches!(x, Effect::Fx { name, .. } if name == "tls_surge")), "its own surge, no order from her");
    for id in [7, 8] {
        assert!(log.iter().any(|x| matches!(x, Effect::Stun { target, .. } if *target == id)), "{id} bumped");
    }
    let surge = pct_of(d::TIDE_POOL_BASE, d::TIDE_POOL_AP_RATIO) * d::ULT_ECHO_DAMAGE_PCT / 100;
    assert!(log.iter().any(|x| matches!(x, Effect::Damage { target: 7, amount, from_clone: true } if *amount == surge)), "half damage {surge}");
}

fn pct_of(base: i64, ratio: i64) -> i64 {
    base + ratio * 100 / 100
}

#[test]
fn the_clone_dives_onto_a_far_champion_and_away_when_low() {
    let mut core = Core::default();
    let mut f = frame(0, 100_000, 480_000, vec![]);
    let mut log = vec![];
    land_trident(&mut core, &mut f, Point::new(950_000, 480_000), &mut log);
    let at = f.echo.unwrap().pos;
    f.units.push(unit(7, 1, at.x - 110_000, at.y));
    log.clear();
    run(&mut core, &mut f, 40, &mut log);
    let (to, ticks) = *echo_moves(&log).iter().find(|m| m.1 >= d::DIVE_DIVE_TICKS as u64).expect("a dive");
    assert!(to.x < at.x - 40_000 && ticks <= 90, "onto the champion: {to:?}");
    assert!(core.pools.iter().any(|p| p.holds(to)), "it opened a pool there");
    // low and caught: it dives away
    let mut core = Core::default();
    let mut f = frame(0, 100_000, 480_000, vec![]);
    land_trident(&mut core, &mut f, Point::new(950_000, 480_000), &mut log);
    let at = f.echo.unwrap().pos;
    f.units.push(unit(7, 1, at.x, at.y - 20_000));
    f.echo.as_mut().unwrap().hp = 10;
    log.clear();
    run(&mut core, &mut f, 40, &mut log);
    let away = echo_moves(&log).into_iter().find(|m| m.1 >= d::DIVE_DIVE_TICKS as u64).expect("an escape dive").0;
    let foe = Point::new(at.x, at.y - 20_000);
    assert!(away.dist(foe) > at.dist(foe) + 20_000.0, "away: {away:?}");
}

#[test]
fn the_clone_fires_bolts_at_half_damage_and_stays_near_its_landing() {
    let mut core = Core::default();
    let mut f = frame(0, 100_000, 480_000, vec![]);
    let mut log = vec![];
    land_trident(&mut core, &mut f, Point::new(950_000, 480_000), &mut log);
    let anchor = core.echo.unwrap().anchor;
    f.units.push(unit(7, 1, anchor.x - 50_000, anchor.y));
    log.clear();
    run(&mut core, &mut f, 200, &mut log);
    let bolt = pct_of(d::BASIC_BASE, d::BASIC_AP_RATIO) * d::ULT_ECHO_DAMAGE_PCT / 100;
    assert!(log.iter().any(|x| matches!(x, Effect::Damage { target: 7, amount, from_clone: true } if *amount == bolt)), "bolts at {bolt}");
    // the enemy runs off across the map: it doesn't chase past its leash
    f.units[0].pos = Point::new(anchor.x - 200_000, anchor.y);
    run(&mut core, &mut f, 200, &mut log);
    for (to, _) in echo_moves(&log) {
        assert!(to.within(anchor, d::ULT_ECHO_LEASH + 1), "leash: {to:?}");
    }
}

#[test]
fn the_clone_and_the_twin_coexist_and_only_the_twin_rescues() {
    let mut core = Core::default();
    let mut f = frame(0, 100_000, 480_000, vec![]);
    let mut log = vec![];
    f.order = Some(Order::Twin(Point::new(120_000, 480_000)));
    run(&mut core, &mut f, 1, &mut log);
    land_trident(&mut core, &mut f, Point::new(950_000, 480_000), &mut log);
    assert!(f.clone.is_some() && f.echo.is_some(), "both stand");
    // she is stunned right next to the clone, far from the (land) twin: the clone never rescues
    f.order = None;
    f.pos = f.echo.unwrap().pos;
    f.stunned = true;
    log.clear();
    run(&mut core, &mut f, 30, &mut log);
    assert!(!log.iter().any(|x| matches!(x, Effect::Cleanse { .. })));
    assert!(!log.iter().any(|x| matches!(x, Effect::SpawnClone { .. } | Effect::Swap { .. })), "no twin or swap from the clone");
}

// ------------------------------------------------------------------ calmer whirlpools (thalassa-1.4)

/// A pool below her, then an immediate surge onto `aim`; returns the surge tick's log.
fn surge_at(core: &mut Core, f: &mut Frame, aim: Point, log: &mut Vec<Effect>) -> Vec<Effect> {
    let here = f.pos;
    cast_s1(core, f, here, log);
    core.ready[0] = 0;
    let mut out = vec![];
    cast_s1(core, f, aim, &mut out);
    log.extend(out.iter().cloned());
    out
}

#[test]
fn a_surge_bumps_at_most_two_champions_the_closest() {
    let mut core = Core::default();
    let aim = Point::new(340_000, 300_000);
    let foes = vec![unit(7, 1, 352_000, 300_000), unit(8, 1, 342_000, 300_000), unit(9, 1, 340_000, 305_000), unit(10, 1, 340_000, 322_000)];
    let mut f = frame(0, 300_000, 300_000, foes);
    let mut log = vec![];
    let out = surge_at(&mut core, &mut f, aim, &mut log);
    let stunned: Vec<usize> = out.iter().filter_map(|e| if let Effect::Stun { target, .. } = e { Some(*target) } else { None }).collect();
    assert_eq!(stunned.len(), d::TIDE_POOL_BUMP_MAX_CHAMPIONS as usize, "{stunned:?}");
    assert!(stunned.contains(&8) && stunned.contains(&9), "the closest two: {stunned:?}");
    for id in [7, 8, 9, 10] {
        assert!(out.iter().any(|e| matches!(e, Effect::Damage { target, .. } if *target == id)), "{id} still takes the surge");
    }
}

#[test]
fn whirlpools_pull_once_on_the_surge_and_never_after() {
    let mut core = Core::default();
    let aim = Point::new(340_000, 300_000);
    let mut f = frame(0, 300_000, 300_000, vec![unit(7, 1, 360_000, 300_000), unit(8, 1, 345_000, 330_000)]);
    // terrain right at the surge center: the pull stops short of it
    f.terrain[(300_000 / CELL * CELLS + 340_000 / CELL) as usize] = true;
    let mut log = vec![];
    let out = surge_at(&mut core, &mut f, aim, &mut log);
    let pulls: Vec<Point> = out.iter().filter_map(|e| if let Effect::Pull { to, .. } = e { Some(*to) } else { None }).collect();
    assert_eq!(pulls.len(), 2, "one pull per caught enemy");
    assert!(pulls.iter().all(|p| !f.terrain_at(*p)), "{pulls:?}");
    let mut later = vec![];
    run(&mut core, &mut f, 300, &mut later);
    assert!(!later.iter().any(|e| matches!(e, Effect::Pull { .. })), "no standing pull");
}

#[test]
fn a_whirlpool_swirls_two_seconds_then_calms_into_a_pool() {
    let mut core = Core::default();
    let mut f = frame(0, 300_000, 300_000, vec![]);
    let mut log = vec![];
    surge_at(&mut core, &mut f, Point::new(340_000, 300_000), &mut log);
    let w = *core.pools.iter().find(|p| p.whirl).expect("a whirlpool");
    let born = w.whirl_until - d::TIDE_POOL_WHIRL_TICKS as u64;
    log.clear();
    let n = w.whirl_until - 1 - f.tick;
    run(&mut core, &mut f, n, &mut log);
    assert!(core.pools.iter().any(|p| p.id == w.id && p.whirl), "still swirling just before 2 s");
    run(&mut core, &mut f, 2, &mut log);
    let p = core.pools.iter().find(|p| p.id == w.id).expect("still a pool");
    assert!(!p.whirl, "calm after 2 s");
    assert_eq!(log.iter().filter(|e| matches!(e, Effect::PoolCalm { id } if *id == w.id)).count(), 1);
    assert_eq!(p.until, born + d::WATER_POOL_TICKS as u64, "keeps its full life");
}

#[test]
fn every_pool_slows_once_whirlpools_more() {
    let slows = |log: &[Effect], id: usize| -> Vec<i64> { log.iter().filter_map(|e| if let Effect::Slow { target, percent, .. } = e { (*target == id).then_some(*percent) } else { None }).collect() };
    let mut core = Core::default();
    // 7 stands in two overlapping normal pools, 8 is out of every pool
    let mut f = frame(0, 300_000, 300_000, vec![unit(7, 1, 310_000, 300_000), unit(8, 1, 600_000, 600_000)]);
    let mut log = vec![];
    let here = f.pos;
    cast_s1(&mut core, &mut f, here, &mut log);
    core.ready[0] = 0;
    f.pos = Point::new(320_000, 300_000);
    // on land? she is still in her pool, so this surges; use a plain second pool instead
    core.pools.push(Pool { id: 77, center: Point::new(315_000, 300_000), radius: d::WATER_POOL_RADIUS, until: 5_000, whirl: false, whirl_until: 0 });
    log.clear();
    run(&mut core, &mut f, 10, &mut log);
    assert_eq!(slows(&log, 7), vec![d::WATER_POOL_SLOW_PCT], "one 20% slow, not stacked");
    assert!(slows(&log, 8).is_empty(), "no slow out of the pools");
    // a fresh whirlpool: 30%
    let mut core = Core::default();
    let mut f = frame(0, 300_000, 300_000, vec![unit(7, 1, 345_000, 300_000)]);
    surge_at(&mut core, &mut f, Point::new(340_000, 300_000), &mut log);
    log.clear();
    run(&mut core, &mut f, 10, &mut log);
    assert!(slows(&log, 7).contains(&d::TIDE_POOL_SLOW_PCT), "{:?}", slows(&log, 7));
}
