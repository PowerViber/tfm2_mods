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
fn fighters_machine_gun_and_fire_lock_on_rockets() {
    let mut core = Core::default();
    let mut f = frame(0, 300_000, 300_000, vec![unit(7, 1, 330_000, 300_000)]);
    let mut log = vec![];
    order(&mut core, &mut f, Order::Fighters(Point::default()), &mut log);
    run(&mut core, &mut f, 400, &mut log);
    let hits = damage(&log, 7);
    let gun = ap(d::FIGHTER_GUN_BASE, d::FIGHTER_GUN_AP_RATIO);
    let rocket = ap(d::FIGHTER_ROCKET_BASE, d::FIGHTER_ROCKET_AP_RATIO);
    assert!(hits.iter().any(|h| *h == (gun, false)), "machine gun: {hits:?}");
    assert!(hits.iter().any(|h| *h == (rocket, true)), "rocket: {hits:?}");
    assert!(log.iter().any(|e| matches!(e, Effect::Fx { name, .. } if name == "wrt_lock")), "lock-on reticle first");
    // guns on their own clock: at most one burst per fighter per interval
    let bursts = hits.iter().filter(|h| h.0 == gun).count() as u64;
    assert!(bursts <= 2 * (400 / d::FIGHTER_GUN_EVERY_TICKS as u64 + 1), "{bursts}");
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

// ------------------------------------------------------------------ S2 Carpet Bomber

#[test]
fn the_carpet_bomber_circles_six_seconds_and_caps_hits_per_enemy() {
    let mut core = Core::default();
    let center = Point::new(380_000, 300_000);
    // an enemy right on the orbit path
    let mut f = frame(0, 300_000, 300_000, vec![unit(7, 1, 380_000 - d::BOMBER_ORBIT_RADIUS, 300_000)]);
    let mut log = vec![];
    order(&mut core, &mut f, Order::Bomber(center), &mut log);
    assert!(log.iter().any(|e| matches!(e, Effect::Fx { name, .. } if name == "wrt_warn")), "a warning first");
    run(&mut core, &mut f, (d::BOMBER_WARN_TICKS + d::BOMBER_TICKS) as u64 + 2, &mut log);
    let bombs = log.iter().filter(|e| matches!(e, Effect::Fx { name, .. } if name == "wrt_bomb")).count() as i64;
    assert!(bombs >= d::BOMBER_TICKS / d::BOMBER_BOMB_EVERY_TICKS - 1, "carpet: {bombs}");
    let hits = damage(&log, 7);
    let per = ap(d::BOMBER_BOMB_BASE, d::BOMBER_BOMB_AP_RATIO);
    assert!(!hits.is_empty() && hits.iter().all(|h| *h == (per, false)));
    assert!(hits.len() as i64 <= d::BOMBER_HITS_PER_SECOND * (d::BOMBER_TICKS / 60 + 1), "capped: {}", hits.len());
    assert!(log.iter().any(|e| matches!(e, Effect::Slow { target: 7, percent, .. } if *percent == d::BOMBER_SLOW_PCT)));
    assert!(!core.planes.iter().any(|p| p.kind == Kind::Bomber), "gone after 6 s");
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

#[test]
fn the_carrier_drops_with_no_damage_and_knocks_the_closest_farthest() {
    let mut core = Core::default();
    let center = Point::new(420_000, 300_000);
    let mut f = frame(0, 300_000, 300_000, vec![unit(7, 1, 425_000, 300_000), unit(8, 1, 470_000, 300_000)]);
    let mut log = vec![];
    order(&mut core, &mut f, Order::Carrier(center), &mut log);
    assert!(log.iter().any(|e| matches!(e, Effect::Fx { name, .. } if name == "wrt_carrier_shadow")));
    log.clear();
    run(&mut core, &mut f, d::CARRIER_WARN_TICKS as u64, &mut log);
    let knock = |id: usize| log.iter().find_map(|e| if let Effect::Knock { target, to, .. } = e { (*target == id).then_some(*to) } else { None });
    let (a, b) = (knock(7).expect("near knocked"), knock(8).expect("far knocked"));
    let pushed = |from: i64, to: Point| (to.x - from).abs();
    assert!(pushed(425_000, a) > pushed(470_000, b), "the closer, the farther: {a:?} {b:?}");
    assert!(pushed(425_000, a) <= d::CARRIER_KNOCK_MAX);
    assert!(damage(&log, 7).is_empty() && damage(&log, 8).is_empty(), "no damage on the drop");
    for id in [7, 8] {
        assert!(log.iter().any(|e| matches!(e, Effect::Slow { target, percent, .. } if *target == id && *percent == d::CARRIER_SLOW_PCT)));
    }
}

#[test]
fn the_carrier_hovers_launching_fighters_bombers_and_choppers_then_leaves() {
    let mut core = Core::default();
    let center = Point::new(420_000, 300_000);
    let mut f = frame(0, 300_000, 300_000, vec![unit(7, 1, 440_000, 320_000)]);
    let mut log = vec![];
    order(&mut core, &mut f, Order::Carrier(center), &mut log);
    let mut most = 0;
    for _ in 0..(d::CARRIER_WARN_TICKS + d::CARRIER_TICKS + 2) {
        run(&mut core, &mut f, 1, &mut log);
        most = most.max(core.planes.iter().filter(|p| p.kind == Kind::Fighter && p.from_carrier && !p.kamikaze).count() as i64);
    }
    assert!(most >= 3 && most <= d::CARRIER_MAX_FIGHTERS, "fighters: {most}");
    assert!(log.iter().filter(|e| matches!(e, Effect::Fx { name, .. } if name == "wrt_bomb")).count() > 5, "carpet passes");
    assert!(log.iter().any(|e| matches!(e, Effect::Heal { target: ME, .. })) || core.planes.iter().any(|p| p.kind == Kind::Chopper) || log.iter().any(|e| matches!(e, Effect::Fx { name, .. } if name.starts_with("wrt_chopper_p"))), "choppers");
    assert!(core.carrier.is_none(), "gone after 8 s");
    assert!(core.own_fighters() == 0, "carrier fighters don't count as his");
}

#[test]
fn knockbacks_never_land_on_terrain() {
    let mut core = Core::default();
    let center = Point::new(420_000, 300_000);
    let mut f = frame(0, 300_000, 300_000, vec![unit(7, 1, 430_000, 300_000)]);
    // a wall right where the push would land
    for x in [448_000, 480_000] {
        f.terrain[(300_000 / CELL * CELLS + x / CELL) as usize] = true;
    }
    let mut log = vec![];
    order(&mut core, &mut f, Order::Carrier(center), &mut log);
    run(&mut core, &mut f, d::CARRIER_WARN_TICKS as u64, &mut log);
    let to = log.iter().find_map(|e| if let Effect::Knock { target: 7, to, .. } = e { Some(*to) } else { None }).expect("knocked");
    assert!(!f.terrain_at(to), "{to:?}");
}

// ------------------------------------------------------------------ AI

#[test]
fn the_ai_launches_fighters_bombs_groups_and_repairs_the_hurt() {
    let mut core = Core::default();
    core.stock = 3;
    let mut f = frame(0, 300_000, 300_000, vec![unit(7, 1, 360_000, 300_000), unit(8, 1, 365_000, 305_000)]);
    f.manual = false;
    f.ult_unlocked = false;
    let mut log = vec![];
    run(&mut core, &mut f, 5, &mut log);
    assert!(core.planes.iter().any(|p| p.kind == Kind::Bomber), "bombs the pair");
    run(&mut core, &mut f, 5, &mut log);
    assert!(core.own_fighters() > 0, "fighters out");
    f.hp = 300;
    run(&mut core, &mut f, 5, &mut log);
    assert!(core.planes.iter().any(|p| p.kind == Kind::Chopper), "repairs himself");
    // the ult on the group once unlocked
    f.ult_unlocked = true;
    run(&mut core, &mut f, 2, &mut log);
    assert!(core.carrier.is_some(), "the carrier on 2 champions");
}
