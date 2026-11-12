// Wright core tests: hand-built frames, the core's own rules. Included by wright_tests.rs and by the Lab crate.
use crate::wright_core::*;
use crate::wright_data as d;

const ME: usize = 1;

#[test]
fn unreported_carriers_are_removed_after_the_existing_receipt_grace() {
    let mut core=Core::default();let mut f=frame(0,300_000,300_000,vec![unit(5,1,320_000,300_000)]);
    f.order=Some(Order::Fighters(Point::default()));let first=core.step(&f);
    assert_eq!(first.iter().filter(|e|matches!(e,Effect::SpawnPlane{..})).count(),2);
    // Host rejected both spawns: the next snapshot contains no carrier receipts.
    f.order=None;f.tick=2;let next=core.step(&f);
    assert_eq!(core.own_fighters(),0);assert!(!next.iter().any(|e|matches!(e,Effect::Damage{..})));
}

fn unit(id: usize, team: usize, x: i64, y: i64) -> Unit {
    Unit { id, team, pos: Point::new(x, y), hp: 1000, max_hp: 1000, radius: 4_000, champion: true, tower: false, visible: true, targetable: true, farmable: false }
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
        planes: vec![],
        attack_target: None,
        damage_taken: 0,
        manual: true,
        order: None,
    }
}

/// Run `n` ticks, playing the host: carriers spawn, glide and vanish; knockbacks move units; damage lands.
fn run(core: &mut Core, f: &mut Frame, n: u64, log: &mut Vec<Effect>) {
    for _ in 0..n {
        let out = core.step(f);
        for e in &out {
            match e {
                Effect::SpawnPlane { key, pos, hp_pct, .. } => {
                    let max_hp = f.max_hp * hp_pct / 100;
                    f.planes.push(PlaneView { key: *key, pos: *pos, hp: max_hp, max_hp });
                }
                Effect::PlaneMove { key, to, .. } => {
                    if let Some(p) = f.planes.iter_mut().find(|p| p.key == *key) {
                        p.pos = *to;
                    }
                }
                Effect::RemovePlane { key } => f.planes.retain(|p| p.key != *key),
                Effect::Knock { target, to, .. } => {
                    if let Some(u) = f.units.iter_mut().find(|u| u.id == *target) {
                        u.pos = *to;
                    }
                }
                Effect::Damage { target, amount, .. } => {
                    if let Some(u) = f.units.iter_mut().find(|u| u.id == *target) {
                        u.hp = (u.hp - amount).max(1);
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

fn order(core: &mut Core, f: &mut Frame, o: Order, log: &mut Vec<Effect>) -> Vec<Effect> {
    f.order = Some(o);
    let mut out = vec![];
    run(core, f, 1, &mut out);
    log.extend(out.iter().cloned());
    out
}

fn spawns(log: &[Effect], kind: Kind) -> Vec<u32> {
    log.iter().filter_map(|e| if let Effect::SpawnPlane { key, kind: k, .. } = e { (*k == kind).then_some(*key) } else { None }).collect()
}
fn damage(log: &[Effect], id: usize) -> Vec<(i64, bool)> {
    log.iter().filter_map(|e| if let Effect::Damage { target, amount, direct } = e { (*target == id).then_some((*amount, *direct)) } else { None }).collect()
}
fn ap(base: i64, ratio: i64) -> i64 {
    base + ratio * 100 / 100
}

// ------------------------------------------------------------------ the workshop

#[test]
fn the_workshop_builds_airframes_to_its_cap_and_skills_need_one() {
    let mut core = Core::default();
    let mut f = frame(0, 300_000, 300_000, vec![]);
    let mut log = vec![];
    run(&mut core, &mut f, 1, &mut log);
    assert_eq!(core.stock, d::WORKSHOP_START_STOCK);
    run(&mut core, &mut f, d::WORKSHOP_BUILD_TICKS as u64 * 5, &mut log);
    assert_eq!(core.stock, d::WORKSHOP_MAX_STOCK, "capped");
    core.stock = 0;
    let out = order(&mut core, &mut f, Order::Fighters(Point::default()), &mut log);
    assert!(spawns(&out, Kind::Fighter).is_empty(), "no airframe: no fighters");
    assert!(core.rejection.contains("Fighters"));
    core.stock = 1;
    let out = order(&mut core, &mut f, Order::Fighters(Point::default()), &mut log);
    assert_eq!(spawns(&out, Kind::Fighter).len(), d::FIGHTER_PER_CAST as usize);
    assert_eq!(core.stock, 0, "one airframe used");
}

#[test]
fn a_scout_drone_flies_to_the_nearest_enemy_and_taunts_once() {
    let mut core = Core::default();
    let mut f = frame(0, 300_000, 300_000, vec![unit(7, 1, 400_000, 300_000), unit(8, 1, 405_000, 310_000)]);
    let mut log = vec![];
    run(&mut core, &mut f, d::WORKSHOP_DRONE_EVERY_TICKS as u64 + 2, &mut log);
    let drone = spawns(&log, Kind::Drone);
    assert_eq!(drone.len(), 1);
    assert!(log.iter().any(|e| matches!(e, Effect::SpawnPlane { kind: Kind::Drone, hp_pct, .. } if *hp_pct == d::DRONE_HP_PCT)));
    run(&mut core, &mut f, 300, &mut log);
    let taunts: Vec<usize> = log.iter().filter_map(|e| if let Effect::Taunt { target, key, .. } = e { (*key == drone[0]).then_some(*target) } else { None }).collect();
    assert!(taunts.contains(&7) && taunts.contains(&8), "{taunts:?}");
    assert_eq!(taunts.len(), 2, "once");
}

// ------------------------------------------------------------------ S1 Fighter Wing

#[test]
fn fighters_stay_out_capped_at_four_and_the_oldest_leaves() {
    let mut core = Core::default();
    core.stock = 3;
    let mut f = frame(0, 300_000, 300_000, vec![]);
    let mut log = vec![];
    let first = spawns(&order(&mut core, &mut f, Order::Fighters(Point::default()), &mut log), Kind::Fighter);
    core.ready[0] = 0;
    order(&mut core, &mut f, Order::Fighters(Point::default()), &mut log);
    assert_eq!(core.own_fighters(), 4);
    core.ready[0] = 0;
    let out = order(&mut core, &mut f, Order::Fighters(Point::default()), &mut log);
    assert_eq!(core.own_fighters(), 4, "still 4");
    for k in first {
        assert!(out.iter().any(|e| matches!(e, Effect::RemovePlane { key } if *key == k)), "the oldest pair leaves");
    }
    // they last 20 s (no target: they just leave, no kamikaze)
    run(&mut core, &mut f, d::FIGHTER_TICKS as u64 + 2, &mut log);
    assert_eq!(core.own_fighters(), 0);
}

#[test]
fn jets_fly_sorties_one_strike_each_and_every_third_is_a_rocket() {
    let mut core = Core::default();
    let mut f = frame(0, 300_000, 300_000, vec![unit(7, 1, 330_000, 300_000)]);
    let mut log = vec![];
    let keys = spawns(&order(&mut core, &mut f, Order::Fighters(Point::default()), &mut log), Kind::Fighter);
    log.clear();
    // follow one jet: out, strike, home, repeat
    let mut phases = vec![];
    for _ in 0..600 {
        run(&mut core, &mut f, 1, &mut log);
        if let Some(p) = core.planes.iter().find(|p| p.key == keys[0]) {
            if phases.last() != Some(&p.phase) {
                phases.push(p.phase);
            }
        }
    }
    assert!(phases.windows(3).any(|w| w == [Phase::Run, Phase::Break, Phase::Patrol]), "out, back, home: {phases:?}");
    let sorties = core.planes.iter().find(|p| p.key == keys[0]).unwrap().waves;
    assert!(sorties >= 3, "{sorties}");
    let gun = ap(d::FIGHTER_SORTIE_BASE, d::FIGHTER_SORTIE_AP_RATIO);
    let rocket = ap(d::FIGHTER_ROCKET_BASE, d::FIGHTER_ROCKET_AP_RATIO);
    let hits = damage(&log, 7);
    let (guns, rockets) = (hits.iter().filter(|h| **h == (gun, false)).count(), hits.iter().filter(|h| **h == (rocket, true)).count());
    let total: i64 = core.planes.iter().filter(|p| p.kind == Kind::Fighter).map(|p| p.waves).sum();
    assert_eq!((guns + rockets) as i64, total, "exactly one strike per sortie");
    assert_eq!(rockets as i64, core.planes.iter().filter(|p| p.kind == Kind::Fighter).map(|p| p.waves / d::FIGHTER_ROCKET_EVERY).sum::<i64>(), "every third");
    assert!(!log.iter().any(|e| matches!(e, Effect::Fx { name, .. } if name == "wrt_lock")), "no lock-on");
}

#[test]
fn the_closer_he_is_the_faster_they_strike() {
    let strikes = |dist: i64| {
        let mut core = Core::default();
        let mut target = unit(7, 1, 300_000 + dist, 300_000);
        target.hp = 100_000;
        target.max_hp = 100_000;
        let mut f = frame(0, 300_000, 300_000, vec![target]);
        let mut log = vec![];
        order(&mut core, &mut f, Order::Fighters(Point::default()), &mut log);
        run(&mut core, &mut f, 600, &mut log);
        damage(&log, 7).len()
    };
    let (near, far) = (strikes(20_000), strikes(70_000));
    // (round 160: 3 strikes per trip and a 2 s rearm; the trip still makes the near target faster)
    assert!(near > far, "near {near} vs far {far}");
}

#[test]
fn no_lock_a_target_that_moves_away_is_missed_and_the_jet_still_returns() {
    let mut core = Core::default();
    let mut f = frame(0, 300_000, 300_000, vec![unit(7, 1, 360_000, 300_000)]);
    let mut log = vec![];
    let keys = spawns(&order(&mut core, &mut f, Order::Fighters(Point::default()), &mut log), Kind::Fighter);
    // wait until the first jet commits, then the target steps 40,000 away
    for _ in 0..60 {
        run(&mut core, &mut f, 1, &mut log);
        if core.planes.iter().any(|p| p.key == keys[0] && p.phase == Phase::Run) {
            break;
        }
    }
    let spot = core.planes.iter().find(|p| p.key == keys[0]).unwrap().goal;
    f.units[0].pos = Point::new(spot.x, spot.y + 40_000);
    // a bystander stands on the spot: it takes the hit instead
    f.units.push(unit(8, 1, spot.x + 2_000, spot.y));
    log.clear();
    for _ in 0..40 {
        run(&mut core, &mut f, 1, &mut log);
        if core.planes.iter().any(|p| p.key == keys[0] && p.phase == Phase::Break) {
            break;
        }
    }
    assert!(core.planes.iter().any(|p| p.key == keys[0] && p.phase == Phase::Break), "struck and heading home");
    assert!(!damage(&log, 8).is_empty(), "the one on the spot is hit");
    // nobody on the spot: a miss, and it still comes home
    let mut core = Core::default();
    let mut f = frame(0, 300_000, 300_000, vec![unit(7, 1, 360_000, 300_000)]);
    let mut log = vec![];
    order(&mut core, &mut f, Order::Fighters(Point::default()), &mut log);
    for _ in 0..30 {
        run(&mut core, &mut f, 1, &mut log);
        if core.planes.iter().any(|p| p.phase == Phase::Run) {
            break;
        }
    }
    f.units[0].pos = Point::new(360_000, 340_000);
    log.clear();
    run(&mut core, &mut f, 30, &mut log);
    assert!(log.iter().any(|e| matches!(e, Effect::Log(l) if l.contains("sortie missed"))));
}

#[test]
fn fighters_kamikaze_when_time_runs_out_with_knockback_and_burn() {
    let mut core = Core::default();
    let mut f = frame(0, 300_000, 300_000, vec![unit(7, 1, 330_000, 300_000)]);
    let mut log = vec![];
    order(&mut core, &mut f, Order::Fighters(Point::default()), &mut log);
    for p in core.planes.iter_mut() {
        p.until = f.tick + 30;
    }
    log.clear();
    run(&mut core, &mut f, 120, &mut log);
    let boom = ap(d::FIGHTER_KAMIKAZE_BASE, d::FIGHTER_KAMIKAZE_AP_RATIO);
    assert!(damage(&log, 7).iter().any(|h| *h == (boom, true)), "{:?}", damage(&log, 7));
    assert!(log.iter().any(|e| matches!(e, Effect::Knock { target: 7, .. })), "knocked back");
    assert!(log.iter().any(|e| matches!(e, Effect::Burn { target: 7, .. })), "burning");
    let burn = (1000 * d::FIGHTER_BURN_PERMILLE_PER_SECOND / 1000 / 2).max(1);
    assert!(damage(&log, 7).iter().any(|h| *h == (burn, false)), "burn ticks");
    assert_eq!(core.own_fighters(), 0);
}

#[test]
fn a_nearly_shot_down_fighter_kamikazes_and_a_shot_down_one_is_gone() {
    let mut core = Core::default();
    let mut f = frame(0, 300_000, 300_000, vec![unit(7, 1, 330_000, 300_000)]);
    let mut log = vec![];
    let keys = spawns(&order(&mut core, &mut f, Order::Fighters(Point::default()), &mut log), Kind::Fighter);
    // the first is nearly shot down, the second is shot down
    f.planes.iter_mut().find(|p| p.key == keys[0]).unwrap().hp = 1;
    f.planes.retain(|p| p.key != keys[1]);
    log.clear();
    run(&mut core, &mut f, 2, &mut log);
    assert!(core.planes.iter().any(|p| p.key == keys[0] && p.kamikaze), "diving");
    assert!(!core.planes.iter().any(|p| p.key == keys[1]), "shot down");
    assert!(log.iter().any(|e| matches!(e, Effect::Log(l) if l.contains("shot down"))));
}

#[test]
fn fighters_farm_only_when_calm() {
    let mut core = Core::default();
    let mut camp = unit(20, 9, 320_000, 300_000);
    camp.champion = false;
    camp.farmable = true;
    let mut f = frame(0, 300_000, 300_000, vec![camp]);
    let mut log = vec![];
    order(&mut core, &mut f, Order::Fighters(Point::default()), &mut log);
    run(&mut core, &mut f, 120, &mut log);
    assert!(!damage(&log, 20).is_empty(), "calm: they farm");
    // an enemy champion close by: they leave the camp for the champion
    f.units.push(unit(7, 1, 340_000, 320_000));
    // (a rocket already in flight may still land on the camp)
    run(&mut core, &mut f, 90, &mut log);
    log.clear();
    run(&mut core, &mut f, 120, &mut log);
    assert!(damage(&log, 20).is_empty() && !damage(&log, 7).is_empty());
}

// ------------------------------------------------------------------ S2 Carpet Bomber (Nana-style)

fn waves_on(log: &[Effect], id: usize) -> (Vec<i64>, usize, usize) {
    let dmg: Vec<i64> = damage(log, id).iter().map(|h| h.0).collect();
    let slows = log.iter().filter(|e| matches!(e, Effect::Slow { target, percent, .. } if *target == id && *percent == d::BOMBER_SLOW_PCT)).count();
    let stuns = log.iter().filter(|e| matches!(e, Effect::Stun { target, .. } if *target == id)).count();
    (dmg, slows, stuns)
}

#[test]
fn the_carpet_bomber_readies_up_then_drops_three_waves_the_last_stuns() {
    let mut core = Core::default();
    let center = Point::new(380_000, 300_000);
    let mut f = frame(0, 300_000, 300_000, vec![unit(7, 1, 380_000, 300_000), unit(8, 1, 380_000, 300_000 + d::BOMBER_RADIUS + 20_000)]);
    let mut log = vec![];
    order(&mut core, &mut f, Order::Bomber(center), &mut log);
    assert_eq!(log.iter().filter(|e| matches!(e, Effect::Fx { name, .. } if name.starts_with("wrt_warn"))).count(),3,"one ready-up warning per explosion");
    log.clear();
    run(&mut core, &mut f, d::BOMBER_WARN_TICKS as u64 - 2, &mut log);
    assert!(damage(&log, 7).is_empty(), "nothing during the ready-up");
    run(&mut core, &mut f, (d::BOMBER_WAVES * d::BOMBER_WAVE_EVERY_TICKS) as u64 + 100, &mut log);
    let (dmg, slows, stuns) = waves_on(&log, 7);
    let per = ap(d::BOMBER_WAVE_BASE, d::BOMBER_WAVE_AP_RATIO);
    assert_eq!(dmg, vec![per; d::BOMBER_WAVES as usize], "hit once per wave");
    assert_eq!((slows, stuns), (d::BOMBER_WAVES as usize - 1, 1), "the first waves slow, the last stuns");
    assert!(damage(&log, 8).is_empty(), "outside the area: untouched");
    assert!(!core.planes.iter().any(|p| p.kind == Kind::Bomber), "flown off");
}

#[test]
fn bomber_strikes_advance_and_only_damage_their_own_warned_footprints() {
    let mut core=Core::default();let mut f=frame(0,300_000,480_000,vec![]);
    let centres=bomber_centres(f.pos,Point::new(370_000,480_000)).unwrap();
    // round 160: the middle explosion on the aim, the others one spacing either side
    let sp=d::BOMBER_SPACING;
    assert_eq!(centres,[Point::new(370_000-sp,480_000),Point::new(370_000,480_000),Point::new(370_000+sp,480_000)]);
    // each unit inside its own footprint only
    for (i,c) in centres.iter().enumerate(){f.units.push(unit(7+i,1,c.x,c.y+d::BOMBER_RADIUS-1_000));}
    let mut log=vec![];order(&mut core,&mut f,Order::Bomber(Point::new(370_000,480_000)),&mut log);
    let warnings:Vec<_>=log.iter().filter_map(|e|match e {Effect::Fx{name,at,ticks,..} if name.starts_with("wrt_warn")=>Some((name.clone(),*at,*ticks)),_=>None}).collect();
    assert_eq!(warnings.iter().map(|w|w.1).collect::<Vec<_>>(),centres);
    let (w,e)=(d::BOMBER_WARN_TICKS as u64,d::BOMBER_WAVE_EVERY_TICKS as u64);
    assert_eq!(warnings.iter().map(|w|w.2).collect::<Vec<_>>(),vec![w,w+e,w+2*e]);
    // Later caster/target movement cannot rotate or move an accepted line.
    f.pos=Point::new(200_000,200_000);log.clear();run(&mut core,&mut f,w+3*e+10,&mut log);
    let impacts:Vec<_>=log.iter().filter_map(|e|match e {Effect::Fx{name,at,..} if name.starts_with("wrt_bomber_wave")=>Some(*at),_=>None}).collect();
    assert_eq!(impacts,centres);
    for id in 7..10 {assert_eq!(damage(&log,id).len(),1,"one packet in the isolated footprint");}
    assert_eq!(waves_on(&log,7).1,1);assert_eq!(waves_on(&log,8).1,1);assert_eq!(waves_on(&log,9).2,1);
}

#[test]
fn bomber_lines_keep_direction_range_and_distinct_centres_near_map_edges() {
    for from in [Point::new(1000,1000),Point::new(300_000,480_000),Point::new(940_000,940_000)] {
        for aim in [Point::new(0,0),Point::new(MAP,MAP),Point::new(MAP,0),Point::new(0,MAP),from] {
            if let Some(cs)=bomber_centres(from,aim) {
                assert!(cs.iter().all(|p|p.on_map()&&p.within(from,d::BOMBER_RANGE+2)));
                assert!(cs[0]!=cs[1]&&cs[1]!=cs[2]);
                let cross=(cs[1].x-cs[0].x)*(cs[2].y-cs[0].y)-(cs[1].y-cs[0].y)*(cs[2].x-cs[0].x);
                assert!(cross.abs()<=2*d::BOMBER_SPACING,"rounding stays on the same line");
            }
        }
    }
    let mut core=Core::default();let mut f=frame(0,MAP-1000,480_000,vec![]);f.order=Some(Order::Bomber(Point::new(MAP,480_000)));
    let out=core.step(&f);assert_eq!(core.stock,d::WORKSHOP_START_STOCK);assert_eq!(core.ready[1],0);
    assert!(!out.iter().any(|e|matches!(e,Effect::Fx{name,..} if name.starts_with("wrt_warn"))));
}

// ------------------------------------------------------------------ S3 Repair Wing

#[test]
fn repair_wing_sends_a_chopper_around_him_and_a_plane_to_the_most_hurt_ally_anywhere() {
    let mut core = Core::default();
    let mut far = unit(9, 0, 850_000, 850_000);
    far.hp = 200;
    let mut f = frame(0, 300_000, 300_000, vec![far]);
    f.hp = 600;
    let mut log = vec![];
    order(&mut core, &mut f, Order::Repair(Point::default()), &mut log);
    assert!(core.planes.iter().any(|p| p.kind == Kind::Chopper && p.anchor == Anchor::Owner));
    assert!(core.planes.iter().any(|p| p.kind == Kind::Repair && p.anchor == Anchor::Ally(9)), "to the most hurt ally");
    let cap = ap(d::REPAIR_SHIELD_CAP_BASE, d::REPAIR_SHIELD_CAP_AP_RATIO);
    for _ in 0..d::REPAIR_TICKS {
        run(&mut core, &mut f, 1, &mut log);
        let active: i64 = core.repair_shields.iter().filter(|s| s.0 == ME).map(|s| s.1).sum();
        assert!(active <= cap, "shields stack only to the cap: {active} > {cap}");
    }
    let healed = |id: usize| log.iter().any(|e| matches!(e, Effect::Heal { target, .. } if *target == id));
    assert!(healed(ME), "the chopper repairs him");
    assert!(healed(9), "the plane reached the far ally");
    assert!(log.iter().any(|e| matches!(e, Effect::Shield { target: ME, .. })));
}

#[test]
fn the_repair_chopper_patches_up_his_fighters() {
    let mut core = Core::default();
    core.stock = 3;
    let mut f = frame(0, 300_000, 300_000, vec![]);
    let mut log = vec![];
    order(&mut core, &mut f, Order::Fighters(Point::default()), &mut log);
    order(&mut core, &mut f, Order::Repair(Point::default()), &mut log);
    run(&mut core, &mut f, 120, &mut log);
    assert!(log.iter().any(|e| matches!(e, Effect::RepairPlane { pct, .. } if *pct == d::REPAIR_FIGHTER_REPAIR_PCT)));
}

// ------------------------------------------------------------------ ult: Sky Carrier

fn land_carrier(core: &mut Core, f: &mut Frame, log: &mut Vec<Effect>) {
    order(core, f, Order::Carrier(Point::default()), log);
    run(core, f, d::CARRIER_WARN_TICKS as u64, log);
}

#[test]
fn the_carrier_drops_on_him_knocking_the_closest_farthest_with_no_damage() {
    let mut core = Core::default();
    let mut f = frame(0, 400_000, 400_000, vec![unit(7, 1, 410_000, 400_000), unit(8, 1, 480_000, 400_000)]);
    let mut log = vec![];
    order(&mut core, &mut f, Order::Carrier(Point::new(900_000, 900_000)), &mut log);
    assert!(log.iter().any(|e| matches!(e, Effect::Fx { name, at, .. } if name == "wrt_carrier_shadow" && *at == Point::new(400_000, 400_000))), "on him, whatever the aim");
    log.clear();
    run(&mut core, &mut f, d::CARRIER_WARN_TICKS as u64, &mut log);
    let knock = |id: usize| log.iter().find_map(|e| if let Effect::Knock { target, to, .. } = e { (*target == id).then_some(*to) } else { None });
    let (a, b) = (knock(7).expect("near knocked"), knock(8).expect("far knocked"));
    assert!(a.x - 410_000 > b.x - 480_000, "the closer, the farther: {a:?} {b:?}");
    assert!(a.x - 410_000 <= d::CARRIER_KNOCK_MAX && a.x > 410_000 && b.x > 480_000, "away from him");
    assert!(damage(&log, 7).is_empty() && damage(&log, 8).is_empty(), "no damage on the drop");
    for id in [7, 8] {
        assert!(log.iter().any(|e| matches!(e, Effect::Slow { target, percent, .. } if *target == id && *percent == d::CARRIER_SLOW_PCT)));
    }
}

#[test]
fn the_carrier_follows_him_and_guards_until_its_pool_runs_out() {
    let mut core = Core::default();
    let mut f = frame(0, 400_000, 400_000, vec![]);
    let mut log = vec![];
    land_carrier(&mut core, &mut f, &mut log);
    assert!(log.iter().any(|e| matches!(e, Effect::Guard { on: true, pct } if *pct == d::CARRIER_GUARD_PCT)), "guard on at landing");
    f.pos = Point::new(450_000, 420_000);
    run(&mut core, &mut f, 2, &mut log);
    assert_eq!(core.carrier.unwrap().center, f.pos, "it follows him");
    // he takes 100 after the guard: the carrier absorbed 400 of it
    let pool = core.carrier.unwrap().pool;
    f.damage_taken = 100;
    run(&mut core, &mut f, 1, &mut log);
    f.damage_taken = 0;
    assert_eq!(pool - core.carrier.unwrap().pool, 100 * d::CARRIER_GUARD_PCT / (100 - d::CARRIER_GUARD_PCT));
    // drain it: it leaves early and the guard comes off
    log.clear();
    f.damage_taken = f.max_hp;
    run(&mut core, &mut f, 1, &mut log);
    f.damage_taken = 0;
    assert!(core.carrier.is_none(), "shot out of the sky");
    assert!(log.iter().any(|e| matches!(e, Effect::Guard { on: false, .. })));
}

#[test]
fn the_carrier_empowers_his_skills_no_airframe_half_cooldown_full_warning_stronger_bombs() {
    // round 165: it launches nothing; S1-S3 cost no airframe and recharge in half the time while it's up
    let mut core = Core::default();
    core.stock = 0;
    let mut f = frame(0, 400_000, 400_000, vec![unit(7, 1, 460_000, 400_000)]);
    let mut log = vec![];
    land_carrier(&mut core, &mut f, &mut log);
    assert!(core.empowered(f.tick));
    log.clear();
    let cast = f.tick;
    order(&mut core, &mut f, Order::Bomber(Point::new(460_000, 400_000)), &mut log);
    assert!(log.iter().any(|e| matches!(e, Effect::Pose { name: "skill2", .. })), "no airframe needed: {log:?}");
    assert_eq!(core.stock, 0);
    assert_eq!(core.ready[1], cast + (d::BOMBER_COOLDOWN_TICKS * d::CARRIER_COOLDOWN_PCT / 100) as u64, "half the cooldown");
    // the full 1 s warning, then stronger explosions on the same footprints
    let warned = log.iter().filter_map(|e| if let Effect::Fx { name, ticks, .. } = e { name.starts_with("wrt_warn1").then_some(*ticks) } else { None }).next();
    assert_eq!(warned, Some(d::BOMBER_WARN_TICKS as u64), "the full warning");
    run(&mut core, &mut f, (d::BOMBER_WARN_TICKS + d::BOMBER_WAVES * d::BOMBER_WAVE_EVERY_TICKS) as u64 + 5, &mut log);
    let per = ap(d::BOMBER_WAVE_BASE, d::BOMBER_WAVE_AP_RATIO) * d::CARRIER_BOMBER_DAMAGE_PCT / 100;
    let hits = damage(&log, 7);
    assert!(!hits.is_empty() && hits.iter().all(|h| h.0 == per), "{hits:?} vs {per}");
    // kits are stronger and jets rearm faster with an extra strike while it's up
    log.clear();
    { let at = f.pos; order(&mut core, &mut f, Order::Fighters(at), &mut log); }
    assert!(core.planes.iter().filter(|p| p.kind == Kind::Fighter).all(|p| p.ammo == d::FIGHTER_AMMO + d::CARRIER_BONUS_AMMO));
}

#[test]
fn the_carrier_launches_nothing_and_never_hurts_allies_or_turrets() {
    let mut core = Core::default();
    let mut tower = unit(30, 1, 430_000, 400_000);
    tower.tower = true;
    tower.champion = false;
    let mut f = frame(0, 400_000, 400_000, vec![unit(4, 0, 405_000, 405_000), unit(7, 1, 440_000, 410_000), unit(8, 1, 445_000, 395_000), tower]);
    let mut log = vec![];
    land_carrier(&mut core, &mut f, &mut log);
    let planes = core.planes.len();
    run(&mut core, &mut f, d::CARRIER_TICKS as u64, &mut log);
    assert!(core.planes.len() <= planes, "no planes from the carrier");
    assert!(!log.iter().any(|e| matches!(e, Effect::SpawnPlane { .. })), "it spawns nothing");
    for id in [ME, 4, 30] {
        assert!(damage(&log, id).is_empty(), "never {id}");
    }
    // his own jet cap holds while it's up
    let mut core = Core::default();
    core.stock = 3;
    let mut f = frame(0, 400_000, 400_000, vec![]);
    let mut log = vec![];
    land_carrier(&mut core, &mut f, &mut log);
    for _ in 0..4 {
        core.ready[0] = 0;
        { let at = f.pos; order(&mut core, &mut f, Order::Fighters(at), &mut log); }
        run(&mut core, &mut f, 2, &mut log);
    }
    assert!(core.own_fighters() as i64 <= d::FIGHTER_MAX_ALIVE, "still at most {} jets", d::FIGHTER_MAX_ALIVE);
}

#[test]
fn knockbacks_never_land_on_terrain() {
    let mut core = Core::default();
    let mut f = frame(0, 400_000, 400_000, vec![unit(7, 1, 410_000, 400_000)]);
    for x in [448_000, 480_000, 512_000] {
        f.terrain[(400_000 / CELL * CELLS + x / CELL) as usize] = true;
    }
    let mut log = vec![];
    land_carrier(&mut core, &mut f, &mut log);
    let to = log.iter().find_map(|e| if let Effect::Knock { target: 7, to, .. } = e { Some(*to) } else { None }).expect("knocked");
    assert!(!f.terrain_at(to), "{to:?}");
}

// ------------------------------------------------------------------ jets

#[test]
fn jets_wait_at_home_when_nothing_is_near() {
    let mut core = Core::default();
    let mut f = frame(0, 300_000, 300_000, vec![]);
    let mut log = vec![];
    order(&mut core, &mut f, Order::Fighters(Point::default()), &mut log);
    run(&mut core, &mut f, 120, &mut log);
    for p in core.planes.iter().filter(|p| p.kind == Kind::Fighter) {
        assert_eq!(p.phase, Phase::Patrol);
        assert!(p.pos.within(f.pos, d::FIGHTER_HOME_RADIUS + 8_000), "at home: {:?}", p.pos);
    }
    assert!(log.iter().all(|e| !matches!(e, Effect::PlaneMove { to, .. } if f.terrain_at(*to))));
}

#[test]
fn jets_clear_minions_when_no_champion_is_in_reach_and_never_hit_turrets() {
    let mut core = Core::default();
    let mut minion = unit(20, 1, 330_000, 300_000);
    minion.champion = false;
    let mut tower = unit(30, 1, 320_000, 310_000);
    tower.champion = false;
    tower.tower = true;
    // an enemy champion far off (not calm), but out of the jets' reach
    let mut f = frame(0, 300_000, 300_000, vec![minion, tower, unit(7, 1, 300_000 + d::FIGHTER_AGGRO_RANGE + 10_000, 300_000)]);
    let mut log = vec![];
    order(&mut core, &mut f, Order::Fighters(Point::default()), &mut log);
    run(&mut core, &mut f, 300, &mut log);
    assert!(!damage(&log, 20).is_empty(), "they clear the minion");
    assert!(damage(&log, 30).is_empty(), "never the turret");
}

// ------------------------------------------------------------------ AI

#[test]
fn the_ai_bombs_groups_launches_jets_repairs_and_calls_the_carrier_under_pressure() {
    let mut core = Core::default();
    core.stock = 3;
    let mut f = frame(0, 300_000, 300_000, vec![unit(7, 1, 380_000, 300_000), unit(8, 1, 385_000, 305_000)]);
    f.manual = false;
    f.ult_unlocked = false;
    let mut log = vec![];
    run(&mut core, &mut f, 5, &mut log);
    assert!(core.planes.iter().any(|p| p.kind == Kind::Bomber), "bombs the pair");
    run(&mut core, &mut f, 5, &mut log);
    assert!(core.own_fighters() > 0, "jets out");
    f.hp = 300;
    run(&mut core, &mut f, 5, &mut log);
    assert!(core.planes.iter().any(|p| p.kind == Kind::Chopper), "repairs himself");
    // not under pressure: no carrier; dived on: carrier
    f.ult_unlocked = true;
    f.hp = 1000;
    run(&mut core, &mut f, 2, &mut log);
    assert!(core.carrier.is_none(), "no pressure yet");
    for u in f.units.iter_mut() {
        u.pos = Point::new(u.pos.x - 60_000, u.pos.y);
    }
    run(&mut core, &mut f, 2, &mut log);
    assert!(core.carrier.is_some(), "2 enemies on him: the carrier");
}

// ------------------------------------------------------------------ round 160 nerf

#[test]
fn round_160_his_ap_growth_and_mod_power_are_cut() {
    assert_eq!(d::MOD_POWER[1], 10, "Mod Power AP 25 -> 10");
    assert_eq!((d::FIGHTER_SORTIE_BASE, d::FIGHTER_SORTIE_AP_RATIO), (12, 14));
    assert_eq!((d::BOMBER_RADIUS, d::BOMBER_SPACING, d::BOMBER_WAVE_EVERY_TICKS), (30_000, 28_000, 36));
}

#[test]
fn a_jet_strikes_three_times_near_itself_then_flies_home_to_rearm() {
    let mut core = Core::default();
    let mut minions = vec![];
    for i in 0..4 {
        let mut m = unit(20 + i, 1, 360_000 + i as i64 * 6_000, 300_000);
        m.champion = false;
        m.hp = 100_000;
        m.max_hp = 100_000;
        minions.push(m);
    }
    let mut f = frame(0, 300_000, 300_000, minions);
    let mut log = vec![];
    let keys = spawns(&order(&mut core, &mut f, Order::Fighters(Point::default()), &mut log), Kind::Fighter);
    let mut went_home_after = None;
    for _ in 0..400 {
        run(&mut core, &mut f, 1, &mut log);
        let p = *core.planes.iter().find(|p| p.key == keys[0]).unwrap();
        if p.phase == Phase::Break && went_home_after.is_none() {
            went_home_after = Some(p.waves);
        }
    }
    assert_eq!(went_home_after, Some(d::FIGHTER_AMMO), "3 strikes, then home");
    let p = core.planes.iter().find(|p| p.key == keys[0]).unwrap();
    assert!(p.ammo > 0, "rearmed");
}

#[test]
fn jets_cruise_slowly_and_use_afterburners_only_on_the_final_approach() {
    let mut core = Core::default();
    let mut far = unit(7, 1, 300_000 + 100_000, 300_000);
    far.hp = 100_000;
    let mut f = frame(0, 300_000, 300_000, vec![far]);
    let mut log = vec![];
    let keys = spawns(&order(&mut core, &mut f, Order::Fighters(Point::default()), &mut log), Kind::Fighter);
    log.clear();
    run(&mut core, &mut f, 60, &mut log);
    let steps: Vec<(Point, Point)> = {
        let mut from = None;
        let mut out = vec![];
        for e in &log {
            if let Effect::PlaneMove { key, to, .. } = e {
                if *key == keys[0] {
                    if let Some(a) = from {
                        out.push((a, *to));
                    }
                    from = Some(*to);
                }
            }
        }
        out
    };
    let cruise = d::FIGHTER_CRUISE_SPEED * 6;
    let burner = cruise * d::FIGHTER_AFTERBURNER_PCT / 100;
    assert!(steps.iter().all(|(a, b)| a.dist(*b) as i64 <= burner + 2), "never faster than the afterburner");
    assert!(steps.iter().any(|(a, b)| (a.dist(*b) as i64 - cruise).abs() <= 2), "cruise legs at 2,400/tick");
}
