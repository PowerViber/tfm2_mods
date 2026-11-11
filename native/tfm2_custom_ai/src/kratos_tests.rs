//! Kratos's core: the passive, each skill, the Guard/Dash links, the CC-immunity cap, Fenrir and the athlete.
use crate::kratos_core::*;
use crate::kratos_data as d;

const ME: usize = 1;
const FOE: usize = 10;

fn unit(id: usize, team: usize, x: i64, y: i64) -> Unit {
    Unit { id, team, pos: Point::new(x, y), hp: 1000, max_hp: 1000, radius: 3000, champion: true, tower: false, visible: true, targetable: true, farmable: false, stunned: false }
}
fn minion(id: usize, x: i64, y: i64) -> Unit {
    Unit { champion: false, farmable: true, hp: 300, max_hp: 300, ..unit(id, 2, x, y) }
}
fn tower(id: usize, x: i64, y: i64) -> Unit {
    Unit { champion: false, tower: true, hp: 3000, max_hp: 3000, ..unit(id, 2, x, y) }
}
fn frame(tick: u64, units: Vec<Unit>) -> Frame {
    Frame {
        tick,
        owner: ME,
        team: 1,
        alive: true,
        pos: Point::new(400_000, 400_000),
        hp: 2000,
        max_hp: 2000,
        attack: 100,
        stunned: false,
        ult_unlocked: true,
        units,
        terrain: vec![],
        profile: [50; 13],
        basic_hits: vec![],
        manual: true,
        order: None,
    }
}
fn order(core: &mut Core, f: &Frame, o: Order) -> Vec<Effect> {
    let mut f = f.clone();
    f.order = Some(o);
    core.step(&f)
}
/// run ticks from..to with no orders, moving `pos` along the glides like the engine would
fn run(core: &mut Core, f: &mut Frame, from: u64, to: u64) -> Vec<Effect> {
    run_order(core, f, None, from, to)
}
/// `run` with an order on the first tick
fn run_order(core: &mut Core, f: &mut Frame, first: Option<Order>, from: u64, to: u64) -> Vec<Effect> {
    let mut all = vec![];
    let mut glide: Option<(Point, Point, u64, u64)> = None;
    for tick in from..to {
        f.tick = tick;
        f.order = if tick == from { first } else { None };
        if let Some((a, b, s, n)) = glide {
            let k = (tick - s).min(n) as i64;
            f.pos = Point::new(a.x + (b.x - a.x) * k / n as i64, a.y + (b.y - a.y) * k / n as i64);
        }
        let out = core.step(f);
        for e in &out {
            match e {
                Effect::Glide { to, ticks } => glide = Some((f.pos, *to, tick, *ticks)),
                Effect::Hop { to } => {
                    f.pos = *to;
                    glide = None;
                }
                _ => {}
            }
        }
        all.extend(out);
    }
    all
}
fn damage_to(out: &[Effect], id: usize) -> i64 {
    out.iter().filter_map(|e| match e { Effect::Damage { target, amount, .. } if *target == id => Some(*amount), _ => None }).sum()
}
fn shields(out: &[Effect]) -> Vec<i64> {
    out.iter().filter_map(|e| match e { Effect::Shield { amount, .. } => Some(*amount), _ => None }).collect()
}
fn immune(out: &[Effect]) -> Option<u64> {
    out.iter().find_map(|e| match e { Effect::Immune { ticks } => Some(*ticks), _ => None })
}
fn wall_grid(cells: &[(i64, i64)]) -> Vec<bool> {
    let mut g = vec![false; (CELLS * CELLS) as usize];
    for (cx, cy) in cells {
        g[(cy * CELLS + cx) as usize] = true;
    }
    g
}

// ------------------------------------------------------------------ passive

#[test]
fn he_starts_on_the_axe_and_only_s3_changes_weapons() {
    let mut c = Core::default();
    assert_eq!(c.weapon, Weapon::Axe);
    let f = frame(0, vec![unit(FOE, 2, 430_000, 400_000)]);
    order(&mut c, &f, Order::Sight);
    assert_eq!(c.weapon, Weapon::Axe);
    let mut f2 = frame(10, vec![unit(FOE, 2, 410_000, 400_000)]);
    order(&mut c, &f2, Order::Rampage(Point::new(410_000, 400_000)));
    run(&mut c, &mut f2, 11, 100);
    assert_eq!(c.weapon, Weapon::Axe, "the Rampage uses both weapons but leaves him where he was");
    let f3 = frame(200, vec![unit(FOE, 2, 440_000, 400_000)]);
    let out = order(&mut c, &f3, Order::Dash(Point::new(440_000, 400_000)));
    assert_eq!(c.weapon, Weapon::Blades);
    assert!(out.contains(&Effect::Weapon { blades: true }));
    let f4 = frame(1000, vec![]);
    let out = order(&mut c, &f4, Order::Guard);
    assert_eq!(c.weapon, Weapon::Axe);
    assert!(out.contains(&Effect::Weapon { blades: false }));
}

#[test]
fn every_swap_shields_five_percent_and_swaps_never_exceed_ten_percent() {
    let mut c = Core::default();
    let max = 2000;
    // Dash (to Blades), Guard (to axe), Dash again: three swaps inside the 5 s shield
    let f = frame(0, vec![]);
    let out = order(&mut c, &f, Order::Dash(Point::new(450_000, 400_000)));
    assert_eq!(shields(&out), vec![max * 5 / 100]);
    let f = frame(10, vec![]);
    let out = order(&mut c, &f, Order::Guard);
    // the swap shield (5%) and Guard's own shield (stronger: linked from the Dash)
    assert_eq!(shields(&out), vec![max * 5 / 100, max * d::GUARD_LINKED_SHIELD_PCT / 100]);
    c.ready[DASH] = 0;
    let f = frame(20, vec![]);
    let out = order(&mut c, &f, Order::Dash(Point::new(450_000, 400_000)));
    assert!(shields(&out).is_empty(), "already 10% from swaps: {out:?}");
    assert_eq!(c.swap_shield_total(20), max * 10 / 100);
    // after the first ones expire, a swap shields again
    c.ready[GUARD] = 0;
    let f = frame(20 + d::PASSIVE_SWAP_SHIELD_TICKS as u64, vec![]);
    let out = order(&mut c, &f, Order::Guard);
    assert_eq!(shields(&out)[0], max * 5 / 100);
}

#[test]
fn three_frost_stacks_freeze_then_the_target_cannot_freeze_for_three_seconds() {
    let mut c = Core::default();
    let mut f = frame(0, vec![unit(FOE, 2, 410_000, 400_000)]);
    let mut freezes = 0;
    for i in 0..3 {
        f.tick = i * 30;
        f.basic_hits = vec![FOE];
        freezes += c.step(&f).iter().filter(|e| matches!(e, Effect::Freeze { .. })).count();
    }
    assert_eq!(freezes, 1);
    assert_eq!(c.frost_of(FOE, 60), 0);
    for i in 3..6 {
        f.tick = i * 30;
        f.basic_hits = vec![FOE];
        freezes += c.step(&f).iter().filter(|e| matches!(e, Effect::Freeze { .. })).count();
    }
    assert_eq!(freezes, 1, "no Frost builds during the freeze immunity");
    f.tick = 60 + d::PASSIVE_FREEZE_IMMUNE_TICKS as u64 + 1;
    for _ in 0..3 {
        f.tick += 10;
        f.basic_hits = vec![FOE];
        freezes += c.step(&f).iter().filter(|e| matches!(e, Effect::Freeze { .. })).count();
    }
    assert_eq!(freezes, 2);
}

#[test]
fn frost_stacks_expire() {
    let mut c = Core::default();
    let mut f = frame(0, vec![unit(FOE, 2, 410_000, 400_000)]);
    f.basic_hits = vec![FOE];
    c.step(&f);
    f.tick = 10;
    c.step(&f);
    assert_eq!(c.frost_of(FOE, 10), 2);
    f.tick = 10 + d::PASSIVE_FROST_TICKS as u64 + 1;
    f.basic_hits = vec![FOE];
    let out = c.step(&f);
    assert!(!out.iter().any(|e| matches!(e, Effect::Freeze { .. })));
    assert_eq!(c.frost_of(FOE, f.tick), 1);
}

#[test]
fn blades_burn_refreshes_and_ticks_each_second_and_never_on_towers() {
    let mut c = Core::default();
    c.weapon = Weapon::Blades;
    let mut f = frame(0, vec![unit(FOE, 2, 410_000, 400_000), tower(20, 420_000, 400_000)]);
    f.basic_hits = vec![FOE, 20];
    let out = c.step(&f);
    assert!(out.contains(&Effect::Burn { target: FOE, ticks: d::PASSIVE_BURN_TICKS as u64 }));
    assert!(!c.burning(20, 0), "towers never burn");
    let per = d::PASSIVE_BURN_BASE + d::PASSIVE_BURN_AD_RATIO * 100 / 100;
    let mut total = 0;
    f.basic_hits.clear();
    for tick in 1..=120 {
        f.tick = tick;
        total += damage_to(&c.step(&f), FOE);
    }
    assert_eq!(total, per * 2);
    f.tick = 121;
    f.basic_hits = vec![FOE];
    c.step(&f);
    assert_eq!(c.burns.len(), 1, "refreshes, never stacks");
    assert_eq!(c.burns[0].1, 121 + d::PASSIVE_BURN_TICKS as u64);
}

// ------------------------------------------------------------------ S1

#[test]
fn heimdalls_sight_gives_immunity_and_dodges_the_first_hit_per_champion() {
    let mut c = Core::default();
    let f = frame(0, vec![unit(FOE, 2, 430_000, 400_000)]);
    let out = order(&mut c, &f, Order::Sight);
    assert_eq!(immune(&out), Some(d::SIGHT_TICKS as u64));
    assert!(out.contains(&Effect::Sight { ticks: d::SIGHT_TICKS as u64 }));
    assert!(c.dodge(5, FOE, true));
    assert!(!c.dodge(6, FOE, true), "only the first hit from each champion");
    assert!(c.dodge(7, 11, true));
    assert!(!c.dodge(8, 50, false), "minions and towers are never dodged");
    assert!(!c.dodge(d::SIGHT_TICKS as u64, 12, true), "the window is over");
}

// ------------------------------------------------------------------ S2

#[test]
fn rampage_plays_three_advancing_swings_and_hits_each_enemy_once_per_swing() {
    let mut c = Core::default();
    let mut f = frame(0, vec![unit(FOE, 2, 478_000, 400_000)]);
    let out = run_order(&mut c, &mut f, Some(Order::Rampage(Point::new(478_000, 400_000))), 0, 3 * d::RAMPAGE_STAGE_TICKS as u64 + 5);
    assert!(out.contains(&Effect::Cleanse));
    assert_eq!(immune(&out), Some(3 * d::RAMPAGE_STAGE_TICKS as u64));
    assert!(out.contains(&Effect::Reduce { percent: d::RAMPAGE_REDUCE_PCT, ticks: 3 * d::RAMPAGE_STAGE_TICKS as u64 }));
    assert!(c.rampage.is_none(), "over in 1.8 s");
    let hits = out.iter().filter(|e| matches!(e, Effect::Damage { target: FOE, direct: false, .. })).count();
    assert_eq!(hits, 3, "{out:?}");
    let knock = out.iter().filter(|e| matches!(e, Effect::KnockUp { target: FOE, ticks } if *ticks == d::RAMPAGE_KNOCK_UP_TICKS as u64)).count();
    assert_eq!(knock, 1);
    // round 162: the advances stop at the foe (the stick gap short of it) instead of running past it
    let advanced = f.pos.x - 400_000;
    assert!((78_000 - d::RAMPAGE_STICK_GAP - 3..=78_000 - d::RAMPAGE_STICK_GAP + 3).contains(&advanced), "advanced {advanced}");
    // the cleave freezes nothing yet (1 stack) and the finisher adds Frost and a burn
    assert!(out.iter().any(|e| matches!(e, Effect::Burn { target: FOE, .. })));
    assert!(out.iter().any(|e| matches!(e, Effect::Slow { target: FOE, percent, .. } if *percent == d::RAMPAGE_SLOW_PCT)));
}

#[test]
fn rampage_stops_at_terrain_and_can_be_redirected() {
    let mut c = Core::default();
    let mut f = frame(0, vec![]);
    // a wall cell just east of him
    f.terrain = wall_grid(&[(13, 12)]);
    f.pos = Point::new(410_000, 400_000);
    run_order(&mut c, &mut f, Some(Order::Rampage(Point::new(500_000, 400_000))), 0, 70);
    assert!(f.pos.x < 13 * CELL, "stopped before the wall: {:?}", f.pos);
    let mut f2 = frame(0, vec![]);
    let mut c2 = Core::default();
    run_order(&mut c2, &mut f2, Some(Order::Rampage(Point::new(500_000, 400_000))), 0, d::RAMPAGE_STAGE_TICKS as u64 - 1);
    let mut g = f2.clone();
    g.tick = d::RAMPAGE_STAGE_TICKS as u64 - 1;
    g.order = Some(Order::Redirect(Point::new(f2.pos.x, 500_000)));
    c2.step(&g);
    run(&mut c2, &mut f2, d::RAMPAGE_STAGE_TICKS as u64, 3 * d::RAMPAGE_STAGE_TICKS as u64 + 2);
    assert!(f2.pos.y > 400_000 + d::RAMPAGE_TRAVEL / 2, "turned south: {:?}", f2.pos);
}

#[test]
fn rampage_and_skills_never_hit_towers_or_allies() {
    let mut c = Core::default();
    let mut f = frame(0, vec![tower(20, 410_000, 400_000), unit(30, 1, 410_000, 400_000)]);
    let mut out = order(&mut c, &f, Order::Rampage(Point::new(410_000, 400_000)));
    out.extend(run(&mut c, &mut f, 1, 80));
    assert_eq!(damage_to(&out, 20), 0);
    assert_eq!(damage_to(&out, 30), 0);
    let f = frame(100, vec![tower(20, 430_000, 400_000)]);
    let out = order(&mut c, &f, Order::Dash(Point::new(430_000, 400_000)));
    assert!(c.dash.is_none(), "never hooks a tower: {out:?}");
}

// ------------------------------------------------------------------ S3

#[test]
fn guard_alone_cleanses_shields_fifteen_percent_and_one_second() {
    let mut c = Core::default();
    c.weapon = Weapon::Blades;
    let f = frame(0, vec![]);
    let out = order(&mut c, &f, Order::Guard);
    assert!(out.contains(&Effect::Cleanse));
    assert!(shields(&out).contains(&(2000 * d::GUARD_SHIELD_PCT / 100)));
    assert_eq!(immune(&out), Some(d::GUARD_IMMUNE_TICKS as u64));
}

#[test]
fn guard_then_dash_reaches_ninety_thousand() {
    let far = 400_000 + 80_000;
    let mut c = Core::default();
    let f = frame(0, vec![unit(FOE, 2, far, 400_000)]);
    // alone: too far
    let out = order(&mut c, &f, Order::Dash(Point::new(far, 400_000)));
    assert!(c.dash.is_none() && out.iter().any(|e| matches!(e, Effect::Log(l) if l.starts_with("chain missed"))));
    let mut c = Core::default();
    order(&mut c, &f, Order::Guard);
    let f = frame(30, vec![unit(FOE, 2, far, 400_000)]);
    order(&mut c, &f, Order::Dash(Point::new(far, 400_000)));
    assert_eq!(c.dash.as_ref().map(|p| p.target), Some(FOE));
}

#[test]
fn dash_then_guard_is_stronger_and_the_link_expires_after_three_seconds() {
    let mut c = Core::default();
    let f = frame(0, vec![unit(FOE, 2, 430_000, 400_000)]);
    order(&mut c, &f, Order::Dash(Point::new(430_000, 400_000)));
    let mut f = frame(1, vec![unit(FOE, 2, 430_000, 400_000)]);
    run(&mut c, &mut f, 1, 20);
    let f = frame(20 + d::IMMUNE_WINDOW_TICKS as u64, vec![]);
    let out = order(&mut c, &f, Order::Guard);
    // the link was made at tick 0: 3 s window -> expired at 380
    assert!(shields(&out).contains(&(2000 * d::GUARD_SHIELD_PCT / 100)), "expired link: plain Guard {out:?}");
    let mut c = Core::default();
    let f = frame(0, vec![unit(FOE, 2, 430_000, 400_000)]);
    order(&mut c, &f, Order::Dash(Point::new(430_000, 400_000)));
    let mut f = frame(1, vec![unit(FOE, 2, 430_000, 400_000)]);
    run(&mut c, &mut f, 1, 30);
    let f = frame(60, vec![]);
    let out = order(&mut c, &f, Order::Guard);
    assert!(shields(&out).contains(&(2000 * d::GUARD_LINKED_SHIELD_PCT / 100)), "{out:?}");
    assert_eq!(immune(&out), Some(d::GUARD_LINKED_IMMUNE_TICKS as u64));
}

#[test]
fn chain_dash_hooks_minions_and_creeps_and_misses_without_a_dash() {
    let mut c = Core::default();
    let mut f = frame(0, vec![minion(40, 440_000, 400_000)]);
    let out = run_order(&mut c, &mut f, Some(Order::Dash(Point::new(440_000, 400_000))), 0, 40);
    assert!(f.pos.x > 430_000, "dashed to the minion: {:?}", f.pos);
    assert!(damage_to(&out, 40) >= d::DASH_BASE + d::DASH_AD_RATIO);
    let mut c = Core::default();
    let f = frame(0, vec![]);
    let out = order(&mut c, &f, Order::Dash(Point::new(440_000, 400_000)));
    assert!(!out.iter().any(|e| matches!(e, Effect::Glide { .. })), "a miss means no dash");
    assert_eq!(c.weapon, Weapon::Blades, "he still threw the Blades");
}

#[test]
fn chain_dash_crosses_a_wall_and_lands_on_open_ground() {
    let mut c = Core::default();
    // him in cell 12, a wall in cell 13, the target in cell 14
    let mut f = frame(0, vec![unit(FOE, 2, 14 * CELL + 16_000, 400_000)]);
    f.pos = Point::new(12 * CELL + 16_000, 400_000);
    f.terrain = wall_grid(&[(13, 12)]);
    let out = run_order(&mut c, &mut f, Some(Order::Dash(Point::new(14 * CELL + 16_000, 400_000))), 0, 40);
    assert!(out.iter().any(|e| matches!(e, Effect::Hop { .. })), "{out:?}");
    assert!(f.pos.x > 14 * CELL && !f.terrain_at(f.pos), "landed past the wall on open ground: {:?}", f.pos);
    assert!(damage_to(&out, FOE) > 0);
}

// ------------------------------------------------------------------ immunity cap

#[test]
fn chained_immunity_never_exceeds_three_seconds_in_six_and_cleanses_still_work() {
    let mut c = Core::default();
    let mut f = frame(0, vec![unit(FOE, 2, 412_000, 400_000)]);
    let mut out = order(&mut c, &f, Order::Sight); // 90
    f.tick = 89;
    out.extend(order(&mut c, &f, Order::Rampage(Point::new(412_000, 400_000)))); // +66 -> 155
    out.extend(run(&mut c, &mut f, 90, 200));
    f.tick = 200;
    let g = order(&mut c, &f, Order::Guard);
    assert!(g.contains(&Effect::Cleanse), "the cleanse always works");
    out.extend(g);
    // a second Sight/Guard pair as soon as possible
    c.ready[SIGHT] = 0;
    c.ready[GUARD] = 0;
    f.tick = 230;
    out.extend(order(&mut c, &f, Order::Sight));
    f.tick = 320;
    out.extend(order(&mut c, &f, Order::Guard));
    for end in 0..800u64 {
        let used = c.immune_in(end.saturating_sub(d::IMMUNE_WINDOW_TICKS as u64), end);
        assert!(used <= d::IMMUNE_CAP_TICKS as u64, "{used} immune ticks in the 6 s before {end}");
    }
    // overlapping windows never stack: Sight then an immediate Rampage only extends to the Rampage's end
    let mut c = Core::default();
    let f = frame(0, vec![unit(FOE, 2, 412_000, 400_000)]);
    order(&mut c, &f, Order::Sight);
    let mut f = frame(10, vec![unit(FOE, 2, 412_000, 400_000)]);
    order(&mut c, &f, Order::Rampage(Point::new(412_000, 400_000)));
    f.tick = 11;
    // Overlap extends only to the later endpoint, rather than adding durations.
    let end=(10+3*d::RAMPAGE_STAGE_TICKS) as u64;
    assert_eq!(c.immune_until(11), end.max(d::SIGHT_TICKS as u64));
    assert_eq!(c.immune_in(0, 1000), end.max(d::SIGHT_TICKS as u64));
}

// ------------------------------------------------------------------ Fenrir

#[test]
fn fenrir_charges_in_from_far_away_and_slows() {
    let mut c = Core::default();
    let mut f = frame(0, vec![unit(FOE, 2, 450_000, 400_000)]);
    let mut out = order(&mut c, &f, Order::Fenrir(Point::new(500_000, 400_000)));
    let w = c.fenrir.clone().unwrap();
    assert_eq!(w.from.x, 400_000 + d::FENRIR_DISTANCE);
    out.extend(run(&mut c, &mut f, 1, 140));
    assert_eq!(damage_to(&out, FOE), d::FENRIR_BASE + d::FENRIR_AD_RATIO);
    assert!(out.iter().any(|e| matches!(e, Effect::Slow { target: FOE, percent, .. } if *percent == d::FENRIR_SLOW_PCT)));
    // no recast: it fades after the window
    out = run(&mut c, &mut f, 140, d::FENRIR_RECAST_TICKS as u64 + 5);
    assert!(c.fenrir.is_none());
    assert!(out.iter().any(|e| matches!(e, Effect::Fx { name, .. } if name == "krt_fenrir_fade")));
}

#[test]
fn fenrir_recast_leaps_in_and_slams_it_back_with_a_knock_up() {
    let mut c = Core::default();
    let mut f = frame(0, vec![unit(FOE, 2, 440_000, 400_000)]);
    let mut out = run_order(&mut c, &mut f, Some(Order::Fenrir(Point::new(500_000, 400_000))), 0, 135);
    assert!(c.fenrir.as_ref().is_some_and(|w| w.pos.within(f.pos, 8_000)), "it ran through him");
    out.extend(run_order(&mut c, &mut f, Some(Order::Slam), 135, 300));
    let knock = out.iter().filter(|e| matches!(e, Effect::KnockUp { target: FOE, ticks } if *ticks == d::FENRIR_KNOCK_UP_TICKS as u64)).count();
    assert_eq!(knock, 1, "{out:?}");
    assert_eq!(damage_to(&out, FOE), d::FENRIR_BASE + d::FENRIR_AD_RATIO + d::FENRIR_RETURN_BASE + d::FENRIR_RETURN_AD_RATIO);
    // too late: a recast after the window is refused
    let mut c = Core::default();
    let mut f = frame(0, vec![]);
    run_order(&mut c, &mut f, Some(Order::Fenrir(Point::new(500_000, 400_000))), 0, d::FENRIR_RECAST_TICKS as u64 + 2);
    run_order(&mut c, &mut f, Some(Order::Slam), d::FENRIR_RECAST_TICKS as u64 + 2, d::FENRIR_RECAST_TICKS as u64 + 3);
    assert!(c.fenrir.is_none());
}

#[test]
fn fenrir_never_hits_towers() {
    let mut c = Core::default();
    let mut f = frame(0, vec![tower(20, 450_000, 400_000)]);
    let mut out = order(&mut c, &f, Order::Fenrir(Point::new(500_000, 400_000)));
    out.extend(run(&mut c, &mut f, 1, 200));
    assert_eq!(damage_to(&out, 20), 0);
}

// ------------------------------------------------------------------ the athlete

#[test]
fn fenrir_never_parks_at_kratos_and_aimed_return_can_turn_ninety_degrees() {
    let mut c=Core::default();let mut f=frame(0,vec![unit(FOE,2,400_000,460_000)]);
    order(&mut c,&f,Order::Fenrir(Point::new(500_000,400_000)));
    let mut prev=c.fenrir.as_ref().unwrap().pos;
    for tick in 1..135 {f.tick=tick;c.step(&f);let p=c.fenrir.as_ref().unwrap().pos;assert_ne!(p,prev,"wolf parked at {tick}");prev=p;}
    f.tick=135;let mut out=order(&mut c,&f,Order::SlamAim(Point::new(400_000,700_000)));
    out.extend(run(&mut c,&mut f,136,320));
    assert_eq!(damage_to(&out,FOE),d::FENRIR_RETURN_BASE+d::FENRIR_RETURN_AD_RATIO);
    assert_eq!(out.iter().filter(|e|matches!(e,Effect::KnockUp{target,..}if *target==FOE)).count(),1);
    assert!(c.fenrir.is_none());
}

#[test]
fn linked_guard_taunts_only_after_a_connected_dash_and_only_enemy_champions() {
    let mut c=Core::default();let mut f=frame(0,vec![unit(FOE,2,430_000,400_000),minion(30,430_000,400_000),tower(31,430_000,400_000),unit(32,1,430_000,400_000)]);
    run_order(&mut c,&mut f,Some(Order::Dash(Point::new(430_000,400_000))),0,25);f.tick=25;
    let out=order(&mut c,&f,Order::Guard);
    let taunts:Vec<_>=out.iter().filter_map(|e|if let Effect::Taunt{target,ticks}=e{Some((*target,*ticks))}else{None}).collect();
    assert_eq!(taunts,vec![(FOE,d::GUARD_TAUNT_TICKS as u64)]);
    let mut miss=Core::default();let empty=frame(0,vec![]);order(&mut miss,&empty,Order::Dash(Point::new(450_000,400_000)));
    let out=order(&mut miss,&frame(20,vec![unit(FOE,2,420_000,400_000)]),Order::Guard);
    assert!(!out.iter().any(|e|matches!(e,Effect::Taunt{..})),"a missed throw cannot arm the taunt");
}

#[test]
fn rampage_and_linked_guard_do_not_reach_through_a_wall() {
    let mut f=frame(0,vec![unit(FOE,2,450_000,400_000)]);f.pos=Point::new(415_000,400_000);f.terrain=wall_grid(&[(13,12)]);
    let mut c=Core::default();let out=run_order(&mut c,&mut f,Some(Order::Rampage(Point::new(500_000,400_000))),0,120);
    assert_eq!(damage_to(&out,FOE),0);assert!(f.pos.x<416_000);
    c.link=Some((Link::Dash,500));c.dash_connected=true;f.tick=120;
    let out=order(&mut c,&f,Order::Guard);assert!(!out.iter().any(|e|matches!(e,Effect::Taunt{..})));
}

// ------------------------------------------------------------------ the athlete

#[test]
fn the_athlete_engages_with_guard_then_the_long_dash() {
    let mut c = Core::default();
    let mut f = frame(0, vec![unit(FOE, 2, 400_000 + 75_000, 400_000)]);
    f.manual = false;
    c.step(&f);
    assert_eq!(c.last_skill, "guard");
    f.tick = 1;
    c.step(&f);
    assert_eq!(c.last_skill, "dash");
    assert!(c.dash.is_some());
}

#[test]
fn the_athlete_farms_only_when_calm() {
    let mut c = Core::default();
    let mut f = frame(0, vec![minion(40, 410_000, 400_000), minion(41, 412_000, 403_000)]);
    f.manual = false;
    c.step(&f);
    assert_eq!(c.last_skill, "rampage");
    let mut c = Core::default();
    let mut f = frame(0, vec![minion(40, 410_000, 400_000), minion(41, 412_000, 403_000), unit(FOE, 2, 470_000, 400_000)]);
    f.manual = false;
    // an enemy champion near: no farming Rampage on the wave (he may engage the champion instead)
    c.step(&f);
    // Its longer reach can now engage that visible champion. This is an engage,
    // not an attempt to farm a wave while ignoring nearby danger.
    assert!(c.last_skill!="rampage"||c.rampage.as_ref().is_some_and(|r|r.aim.x>r.at.x&&r.aim.y==r.at.y));
}

#[test]
fn stunned_he_casts_nothing() {
    let mut c = Core::default();
    let mut f = frame(0, vec![unit(FOE, 2, 410_000, 400_000)]);
    f.stunned = true;
    f.manual = false;
    assert!(c.step(&f).is_empty());
}

// ------------------------------------------------------------------ glue (the mock host)

mod glue {
    use crate::glue_tests::{open_spot, with_sim, Host, Unit as HostUnit};
    use crate::kratos::Kratos;
    use crate::kratos_core::{Effect, Point};
    use crate::kratos_data as d;
    use mod_api_stable::{AttackTypeV1, CcKindV1, EntityKindV1 as K, StablePassive};

    fn champ(team: usize, pos: (u64, u64), name: &'static str) -> HostUnit {
        HostUnit { kind: K::Champion, team, pos, hp: 2000, name, targetable: true }
    }
    fn run(host: &std::cell::RefCell<Host>, k: &mut Kratos, from: usize, ticks: usize) {
        for t in from..from + ticks {
            host.borrow_mut().tick = t;
            with_sim(host, |sim| k.on_update(sim, t as u64, 0, 0));
        }
    }

    #[test]
    fn the_chain_dash_glides_through_the_engine_and_strikes_with_a_skill_hit() {
        let (x, y) = open_spot(80_000);
        let host = Host::new(vec![champ(0, (x, y), "tfm2_custom_kratos"), champ(1, (x + 40_000, y), "foe")]);
        let mut k = Kratos::default();
        with_sim(&host, |sim| k.on_spawn(sim, 0, 0));
        host.borrow_mut().tick = 100;
        let f = with_sim(&host, |sim| k.frame(sim, 0));
        let mut out = vec![];
        k.core.execute(&f, crate::kratos_core::Order::Dash(Point::new((x + 40_000) as i64, y as i64)), &mut out);
        with_sim(&host, |sim| k.apply(sim, 0, out));
        // the dash itself is an engine glide (checked now: a later Rampage's cleanse clears the mock's CC list)
        assert!(host.borrow().ccs[0].iter().any(|c| c.0.kind == CcKindV1::ForceMove.code() && c.0.name() == crate::motion::SELF), "an engine glide");
        // the mock host never moves units on ForceMove: put him next to the foe as the engine would
        host.borrow_mut().units[0].pos = (x + 30_000, y);
        run(&host, &mut k, 101, 30);
        let g = host.borrow();
        assert!(g.moves.iter().all(|m| m.0 != 0), "no hand-set position on a clear line: {:?}", g.moves);
        assert!(g.damage.iter().any(|dm| dm.0 == 0 && dm.1 == 1 && dm.3 == AttackTypeV1::Skill.code()), "{:?}", g.damage);
        // and he now holds the Blades: the longer basic reach
        let blades = g.buffs[0].iter().find(|b| b.name() == "krt_blades").expect("blades buff");
        assert_eq!(blades.range as i64, d::BASIC_BLADES_RANGE - d::BASIC_AXE_RANGE);
        assert!(g.shields.iter().any(|s| s.0 == 0 && s.1 == 2000 * d::PASSIVE_SWAP_SHIELD_PCT as usize / 100), "swap shield: {:?}", g.shields);
    }

    #[test]
    fn connected_guard_uses_native_taunt_with_kratos_as_its_target() {
        let (x,y)=open_spot(80_000);
        let host=Host::new(vec![champ(0,(x,y),"tfm2_custom_kratos"),champ(1,(x+5_000,y),"foe"),champ(0,(x+8_000,y),"ally")]);
        let mut k=Kratos::default();host.borrow_mut().tick=100;
        let f=with_sim(&host,|s|k.frame(s,0));k.core.link=Some((crate::kratos_core::Link::Dash,200));k.core.dash_connected=true;
        let mut out=vec![];k.core.execute(&f,crate::kratos_core::Order::Guard,&mut out);with_sim(&host,|s|k.apply(s,0,out));
        let h=host.borrow();assert!(h.ccs[1].iter().any(|c|c.0.kind==CcKindV1::Taunt.code()&&c.0.target==0&&c.0.tick==d::GUARD_TAUNT_TICKS as u64));
        assert!(!h.ccs[2].iter().any(|c|c.0.kind==CcKindV1::Taunt.code()));
    }

    #[test]
    fn engage_speed_is_one_replaced_buff() {
        let (x, y) = open_spot(80_000);
        let host = Host::new(vec![champ(0, (x, y), "tfm2_custom_kratos")]);
        let mut k = Kratos::default();
        with_sim(&host, |sim| k.apply(sim, 0, vec![Effect::Haste { percent: 20, ticks: 120 }, Effect::Haste { percent: 25, ticks: 90 }]));
        let g = host.borrow();
        let hastes: Vec<_> = g.buffs[0].iter().filter(|b| b.name() == "krt_haste").collect();
        assert_eq!(hastes.len(), 1);
        assert_eq!(hastes[0].move_speed_mult, 25);
    }

    #[test]
    fn guard_shields_the_white_bar_cleanses_and_grants_named_immunity() {
        let (x, y) = open_spot(80_000);
        let host = Host::new(vec![champ(0, (x, y), "tfm2_custom_kratos")]);
        let mut k = Kratos::default();
        with_sim(&host, |sim| k.on_spawn(sim, 0, 0));
        host.borrow_mut().tick = 50;
        let f = with_sim(&host, |sim| k.frame(sim, 0));
        let mut out = vec![];
        k.core.execute(&f, crate::kratos_core::Order::Guard, &mut out);
        with_sim(&host, |sim| k.apply(sim, 0, out));
        let g = host.borrow();
        assert_eq!(g.cleansed, vec![0]);
        assert!(g.shields.contains(&(0, 2000 * d::GUARD_SHIELD_PCT as usize / 100, d::GUARD_SHIELD_TICKS as usize)), "{:?}", g.shields);
        assert!(g.buffs[0].iter().any(|b| b.name() == "krt_immune" && b.cc_immune));
    }

    #[test]
    fn heimdalls_sight_refunds_the_first_hit_from_each_enemy_champion() {
        let (x, y) = open_spot(80_000);
        let host = Host::new(vec![
            champ(0, (x, y), "tfm2_custom_kratos"),
            champ(1, (x + 20_000, y), "foe"),
            HostUnit { kind: K::Minion, team: 1, pos: (x + 10_000, y), hp: 300, name: "minion", targetable: true },
        ]);
        let mut k = Kratos::default();
        with_sim(&host, |sim| k.on_spawn(sim, 0, 0));
        host.borrow_mut().tick = 10;
        let f = with_sim(&host, |sim| k.frame(sim, 0));
        let mut out = vec![];
        k.core.execute(&f, crate::kratos_core::Order::Sight, &mut out);
        with_sim(&host, |sim| k.apply(sim, 0, out));
        with_sim(&host, |sim| {
            k.on_damaged(sim, 0, 0, 1, 120);
            k.on_damaged(sim, 0, 0, 1, 80);
            k.on_damaged(sim, 0, 0, 2, 40);
        });
        assert_eq!(host.borrow().heals, vec![(0, 0, 120)], "only the first champion hit is refunded");
    }

    #[test]
    fn his_own_named_cc_never_reads_as_being_stunned() {
        let (x, y) = open_spot(80_000);
        let host = Host::new(vec![champ(0, (x, y), "tfm2_custom_kratos")]);
        let mut k = Kratos::default();
        with_sim(&host, |sim| {
            k.apply(sim, 0, vec![Effect::Pose { name: "skill2", ticks: 30 }, Effect::Glide { to: Point::new(x as i64 + 9_000, y as i64), ticks: 10 }]);
        });
        host.borrow_mut().tick = 1;
        let f = with_sim(&host, |sim| k.frame(sim, 0));
        assert!(!f.stunned);
    }

    #[test]
    fn towers_and_caged_or_parrying_targets_are_spared() {
        let (x, y) = open_spot(80_000);
        let host = Host::new(vec![
            champ(0, (x, y), "tfm2_custom_kratos"),
            HostUnit { kind: K::Tower, team: 1, pos: (x + 20_000, y), hp: 3000, name: "tower", targetable: true },
            champ(1, (x + 30_000, y), "tfm2_ultrakill_v1"),
        ]);
        let mut k = Kratos::default();
        with_sim(&host, |sim| sim.add_buff(2, &crate::timed("v1_guard", 600)));
        host.borrow_mut().tick = 5;
        with_sim(&host, |sim| {
            k.apply(
                sim,
                0,
                vec![
                    Effect::Damage { target: 1, amount: 200, direct: false },
                    Effect::Freeze { target: 1, ticks: 30 },
                    Effect::Damage { target: 2, amount: 150, direct: true },
                ],
            )
        });
        let g = host.borrow();
        assert!(g.damage.is_empty(), "no tower damage and the parried strike dealt nothing: {:?}", g.damage);
        assert!(g.ccs[1].is_empty(), "towers never freeze");
        assert!(g.buffs[2].iter().any(|b| b.name() == "v1_parry"), "V1 parried the chain strike");
    }
}

#[test]
fn slammed_fenrir_flies_back_faster_than_it_charged_in_and_still_hits_once() {
    // Reihan: the redirect is fast, like Ornn's ram after the headbutt
    assert!(d::FENRIR_RETURN_SPEED > d::FENRIR_SPEED);
    let mut c = Core::default();
    // a foe well along the return line (east of him, the way the wolf came from)
    let mut f = frame(0, vec![unit(FOE, 2, 400_000 + 200_000, 400_000 + 60_000)]);
    run_order(&mut c, &mut f, Some(Order::Fenrir(Point::new(500_000, 400_000))), 0, 135);
    let mut out = run_order(&mut c, &mut f, Some(Order::SlamAim(Point::new(600_000, 460_000))), 135, 160);
    let w = c.fenrir.clone().expect("the return leg is running");
    assert_eq!(w.leg, 1);
    let expected = (w.from.dist(w.to) / d::FENRIR_RETURN_SPEED as f64).ceil() as u64;
    assert_eq!(w.leg_ticks, expected);
    let charge_ticks = (w.from.dist(w.to) / d::FENRIR_SPEED as f64).ceil() as u64;
    assert!(w.leg_ticks * 2 <= charge_ticks + 1, "return {} vs charge {} ticks over the same distance", w.leg_ticks, charge_ticks);
    out.extend(run(&mut c, &mut f, 160, 400));
    assert_eq!(out.iter().filter(|e| matches!(e, Effect::KnockUp { target: FOE, .. })).count(), 1, "hit once on the fast return");
    assert_eq!(damage_to(&out, FOE), d::FENRIR_RETURN_BASE + d::FENRIR_RETURN_AD_RATIO);
}

// ------------------------------------------------------------------ round 162: Rampage on target, no tower dives, engage speed

#[test]
fn rampage_with_nobody_ahead_still_travels_its_full_length() {
    let mut c = Core::default();
    let mut f = frame(0, vec![]);
    run_order(&mut c, &mut f, Some(Order::Rampage(Point::new(500_000, 400_000))), 0, 3 * d::RAMPAGE_STAGE_TICKS as u64 + 5);
    let advanced = f.pos.x - 400_000;
    assert!((d::RAMPAGE_TRAVEL - 3..=d::RAMPAGE_TRAVEL + 3).contains(&advanced), "advanced {advanced}");
}

#[test]
fn rampage_stays_on_a_close_target_instead_of_running_to_the_front() {
    let mut c = Core::default();
    let mut f = frame(0, vec![unit(FOE, 2, 420_000, 400_000)]);
    let out = run_order(&mut c, &mut f, Some(Order::Rampage(Point::new(420_000, 400_000))), 0, 3 * d::RAMPAGE_STAGE_TICKS as u64 + 5);
    assert!(f.pos.dist(Point::new(420_000, 400_000)) <= 25_000.0, "ended at {:?}, not 108,000 forward", f.pos);
    assert_eq!(out.iter().filter(|e| matches!(e, Effect::Damage { target: FOE, direct: false, .. })).count(), 3, "all three swings land");
}

#[test]
fn rampage_never_walks_into_enemy_tower_range_unless_already_inside() {
    let tower_x = 400_000 + 120_000;
    let mut c = Core::default();
    let mut f = frame(0, vec![tower(20, tower_x, 400_000)]);
    run_order(&mut c, &mut f, Some(Order::Rampage(Point::new(500_000, 400_000))), 0, 3 * d::RAMPAGE_STAGE_TICKS as u64 + 5);
    assert!(!f.pos.within(Point::new(tower_x, 400_000), d::RAMPAGE_TOWER_RANGE), "stopped outside the tower: {:?}", f.pos);
    assert!(f.pos.x - 400_000 >= 120_000 - d::RAMPAGE_TOWER_RANGE - 2_500, "but went as far as it could: {:?}", f.pos);
    // already under the tower: he chose to be there, the steps aren't cut
    let mut c = Core::default();
    let mut f = frame(0, vec![tower(20, 400_000 + 60_000, 400_000)]);
    run_order(&mut c, &mut f, Some(Order::Rampage(Point::new(500_000, 400_000))), 0, 3 * d::RAMPAGE_STAGE_TICKS as u64 + 5);
    assert!(f.pos.x - 400_000 >= d::RAMPAGE_TRAVEL - 3, "inside already: full travel {:?}", f.pos);
}

#[test]
fn the_athlete_wont_rampage_a_healthy_foe_under_their_tower_but_will_dive_a_low_one() {
    let mut c = Core::default();
    let mut f = frame(0, vec![unit(FOE, 2, 470_000, 400_000), tower(20, 530_000, 400_000)]);
    f.manual = false;
    c.ready[DASH] = u64::MAX;
    c.ready[GUARD] = u64::MAX;
    c.ready[SIGHT] = u64::MAX;
    c.ready[FENRIR] = u64::MAX;
    c.step(&f);
    assert_ne!(c.last_skill, "rampage", "no Rampage onto a healthy foe under the tower");
    let mut c = Core::default();
    let mut low = unit(FOE, 2, 470_000, 400_000);
    low.hp = 200;
    let mut f = frame(0, vec![low, tower(20, 530_000, 400_000)]);
    f.manual = false;
    c.ready[DASH] = u64::MAX;
    c.ready[GUARD] = u64::MAX;
    c.ready[SIGHT] = u64::MAX;
    c.ready[FENRIR] = u64::MAX;
    c.step(&f);
    assert_eq!(c.last_skill, "rampage", "a nearly dead foe is worth the dive");
}

fn haste(out: &[Effect]) -> Vec<(i64, u64)> {
    out.iter().filter_map(|e| match e { Effect::Haste { percent, ticks } => Some((*percent, *ticks)), _ => None }).collect()
}

#[test]
fn sight_and_swaps_give_speed_and_the_strongest_one_wins_without_stacking() {
    let mut c = Core::default();
    let out = order(&mut c, &frame(0, vec![]), Order::Sight);
    assert_eq!(haste(&out), vec![(d::SPEED_SIGHT_PCT, d::SIGHT_TICKS as u64)]);
    // a swap during Sight: the weaker 20% doesn't lower the 25%
    let out = order(&mut c, &frame(10, vec![]), Order::Guard);
    assert!(haste(&out).is_empty(), "weaker ask ignored while Sight's is up: {out:?}");
    assert_eq!(c.haste.0, d::SPEED_SIGHT_PCT);
    // after Sight ends, a swap gives its own 20% for 2 s
    let out = order(&mut c, &frame(200, vec![]), Order::Dash(Point::new(450_000, 400_000)));
    assert_eq!(haste(&out), vec![(d::SPEED_SWAP_PCT, d::SPEED_SWAP_TICKS as u64)]);
}

#[test]
fn spartan_charge_speeds_him_toward_a_champion_then_waits_its_cooldown() {
    let mut c = Core::default();
    let mut f = frame(0, vec![unit(FOE, 2, 450_000, 400_000)]);
    assert!(haste(&c.step(&f)).is_empty(), "standing still: no charge");
    f.tick = 1;
    f.pos.x += 1_000;
    assert_eq!(haste(&c.step(&f)), vec![(d::SPEED_CHARGE_PCT, d::SPEED_CHARGE_TICKS as u64)]);
    f.tick = 200;
    f.pos.x += 1_000;
    assert!(haste(&c.step(&f)).is_empty(), "on cooldown");
    f.tick = 1 + d::SPEED_CHARGE_COOLDOWN_TICKS as u64;
    f.pos.x += 1_000;
    assert_eq!(haste(&c.step(&f)).len(), 1);
    // walking away from it never triggers
    let mut c = Core::default();
    let mut f = frame(0, vec![unit(FOE, 2, 450_000, 400_000)]);
    c.step(&f);
    f.tick = 1;
    f.pos.x -= 1_000;
    assert!(haste(&c.step(&f)).is_empty());
}
