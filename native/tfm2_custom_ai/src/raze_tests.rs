use crate::raze_core::*;
use crate::raze_data::*;
use crate::valorant_guns::Gun;
fn unit(id: usize, team: usize, x: i64, y: i64) -> Unit {
    Unit {
        id,
        team,
        pos: Point::new(x, y),
        hp: 2000,
        max_hp: 2000,
        ap: 100,
        attack: 70,
        visible: true,
        targetable: true,
        ..Default::default()
    }
}
fn frame() -> Frame {
    Frame {
        units: vec![unit(1, 0, 300000, 300000), unit(2, 1, 350000, 300000)],
        ..Default::default()
    }
}
fn ready() -> Core {
    Core {
        stock: [2, 1, 1],
        ..Default::default()
    }
}
fn tick(c: &mut Core, f: &mut Frame, a: Option<Action>) -> Vec<Cmd> {
    f.action = a;
    f.tick += 1;
    let out = c.step(f);
    for cmd in &out {
        match cmd {
            Cmd::MoveOwner { to } => f.units[0].pos = *to,
            Cmd::Spawn { id, hp, .. } => f.utility.push(UtilityStatus { id: *id, hp: *hp }),
            Cmd::Remove { id } => f.utility.retain(|s| s.id != *id),
            _ => {}
        }
    }
    out
}
fn advance(c: &mut Core, f: &mut Frame, n: usize) -> Vec<Cmd> {
    let mut all = vec![];
    for _ in 0..n {
        all.extend(tick(c, f, None));
    }
    all
}
fn pack(c: &mut Core, f: &mut Frame) {
    let aim = f.units[0].pos.add(Point::new(
        if c.flight.is_some() { 10000 } else { -10000 },
        0,
    ));
    tick(c, f, Some(Action::Pack { aim }));
    advance(c, f, PACK_THROW as usize + 1);
}

#[test]
fn guns_match_omens_actual_table() {
    for (n, (price, range, mods)) in [
        (0, 30000, (0, 0)),
        (800, 38000, (35, -30)),
        (1600, 28000, (-15, 60)),
        (1850, 18000, (0, -25)),
        (2900, 42000, (30, 15)),
        (4700, 80000, (200, -75)),
    ]
    .into_iter()
    .enumerate()
    {
        let g = Gun::from_index(n);
        assert_eq!((g.price(), g.reach(), g.modifiers()), (price, range, mods));
    }
    assert_eq!(Gun::upgrade(5499, 0, Gun::Vandal), Gun::Vandal);
    assert_eq!(Gun::upgrade(5500, 0, Gun::Vandal), Gun::Operator);
}
#[test]
fn gun_first_purchase_and_paid_utility() {
    let mut c = Core::default();
    let mut f = frame();
    f.buy_allowed = true;
    tick(&mut c, &mut f, Some(Action::Buy));
    assert_eq!(c.gun, 1);
    assert_eq!(c.stock, [1, 0, 0]);
    assert_eq!(c.credits, 0);
    advance(&mut c, &mut f, 1199);
    assert_eq!(c.stock, [1, 0, 0]);
}
#[test]
fn unavailable_utility_and_invalid_placement_spend_nothing() {
    let mut c = ready();
    let mut f = frame();
    f.blocks.push(Rect {
        min: Point::new(285000, 285000),
        max: Point::new(295000, 315000),
    });
    let before = c.stock;
    tick(
        &mut c,
        &mut f,
        Some(Action::Pack {
            aim: Point::new(290000, 300000),
        }),
    );
    assert_eq!(before, c.stock);
    assert!(c.cast.is_none());
    f.unlocked = [false; 3];
    tick(&mut c, &mut f, Some(Action::EquipRocket));
    assert_eq!(c.cooldowns, [0; 3]);
}
#[test]
fn utility_is_not_automatically_refilled() {
    let mut c = ready();
    let mut f = frame();
    tick(
        &mut c,
        &mut f,
        Some(Action::Grenade {
            aim: Point::new(350000, 300000),
        }),
    );
    advance(&mut c, &mut f, 1800);
    assert_eq!(c.stock[1], 0);
}
#[test]
fn credit_events_and_same_tick_are_idempotent() {
    let mut c = Core::default();
    let mut f = frame();
    f.credits = vec![CreditEvent { id: 42, kind: 0 }];
    tick(&mut c, &mut f, None);
    let credit = c.credits;
    assert!(c.step(&f).is_empty());
    tick(&mut c, &mut f, None);
    assert_eq!(c.credits, credit);
}
#[test]
fn carrier_failure_refunds_only_failed_summon() {
    let mut c = ready();
    let mut f = frame();
    pack(&mut c, &mut f);
    let id = c.packs[0].id;
    f.utility = vec![UtilityStatus { id, hp: -1 }];
    tick(&mut c, &mut f, None);
    assert_eq!(c.stock[0], 2);
    assert!(c.packs.is_empty());
    assert!(c.last_rejection.contains("spawn failed"));
}
#[test]
fn destroyed_or_expired_pack_never_launches_or_explodes() {
    for hp in [0, -1] {
        let mut c = ready();
        let mut f = frame();
        pack(&mut c, &mut f);
        let id = c.packs[0].id;
        f.utility = vec![UtilityStatus { id, hp }];
        let mut out = tick(&mut c, &mut f, None);
        out.extend(tick(&mut c, &mut f, Some(Action::Detonate { id })));
        assert!(c.flight.is_none());
        assert!(!out.iter().any(|v| matches!(v, Cmd::Damage { .. })));
    }
}
#[test]
fn early_satchel_launch_has_no_damage() {
    let mut c = ready();
    let mut f = frame();
    f.units[1].pos = Point::new(294000, 300000);
    pack(&mut c, &mut f);
    let out = tick(&mut c, &mut f, Some(Action::Detonate { id: 0 }));
    assert!(c.flight.is_some());
    assert!(!out.iter().any(|v| matches!(v, Cmd::Damage { .. })));
    assert!(out.iter().any(|v| matches!(v, Cmd::Push { .. })));
}
#[test]
fn armed_satchel_damage_and_enemy_displacement() {
    let mut c = ready();
    let mut f = frame();
    f.units[1].pos = Point::new(294000, 300000);
    pack(&mut c, &mut f);
    advance(&mut c, &mut f, 60);
    let out = tick(&mut c, &mut f, Some(Action::Detonate { id: 0 }));
    assert!(out.contains(&Cmd::Damage {
        id: 2,
        target: 2,
        raw: 35,
        self_hit: false
    }));
}
#[test]
fn strafing_bends_momentum_without_instant_turn() {
    let mut c = ready();
    let mut f = frame();
    pack(&mut c, &mut f);
    tick(&mut c, &mut f, Some(Action::Detonate { id: 0 }));
    let before = c.flight.as_ref().unwrap().velocity;
    tick(
        &mut c,
        &mut f,
        Some(Action::Steer {
            input: Point::new(0, 1000),
        }),
    );
    let after = c.flight.as_ref().unwrap().velocity;
    assert!(after.x > before.x * 9 / 10);
    assert!(after.y > 0 && after.y < 10000);
    advance(&mut c, &mut f, 15);
    assert!(f.units[0].pos.y > 300000);
}
#[test]
fn second_satchel_adds_impulse_and_preserves_tangent() {
    let mut c = ready();
    let mut f = frame();
    pack(&mut c, &mut f);
    tick(&mut c, &mut f, Some(Action::Detonate { id: 0 }));
    tick(
        &mut c,
        &mut f,
        Some(Action::Steer {
            input: Point::new(0, 1000),
        }),
    );
    pack(&mut c, &mut f);
    let old = c.flight.as_ref().unwrap().velocity;
    tick(&mut c, &mut f, Some(Action::Detonate { id: 0 }));
    let v = c.flight.as_ref().unwrap().velocity;
    assert!(v.y > 0);
    assert!(v.x >= old.x);
    assert!(v.length() <= MAX_SPEED);
    assert!(
        c.flight.as_ref().unwrap().until - c.flight.as_ref().unwrap().born
            <= MAX_FLIGHT_TICKS as u64
    );
}
#[test]
fn thin_wall_sweeps_prevent_tunnelling() {
    let mut c = ready();
    let mut f = frame();
    pack(&mut c, &mut f);
    f.blocks.push(Rect {
        min: Point::new(310000, 200000),
        max: Point::new(311000, 400000),
    });
    tick(&mut c, &mut f, Some(Action::Detonate { id: 0 }));
    advance(&mut c, &mut f, 30);
    assert!(f.units[0].pos.x < 310000 - BODY_RADIUS);
    assert!(c.flight.is_none());
    assert_eq!(c.metrics.collisions, 1);
}
#[test]
fn bind_stops_propulsion_without_restoring_stock() {
    let mut c = ready();
    let mut f = frame();
    pack(&mut c, &mut f);
    tick(&mut c, &mut f, Some(Action::Detonate { id: 0 }));
    f.bound = true;
    let at = f.units[0].pos;
    advance(&mut c, &mut f, 10);
    assert_eq!(at, f.units[0].pos);
    assert_eq!(c.stock[0], 1);
}
#[test]
fn interruption_spends_commitment_and_cancels_unreleased_effect() {
    let mut c = ready();
    let mut f = frame();
    tick(
        &mut c,
        &mut f,
        Some(Action::Grenade {
            aim: Point::new(350000, 300000),
        }),
    );
    f.silenced = true;
    let out = advance(&mut c, &mut f, 120);
    assert!(c.grenades.is_empty());
    assert_eq!(c.stock[1], 0);
    assert!(c.cooldowns[0] > f.tick);
    assert!(!out.iter().any(|x| matches!(x, Cmd::Damage { .. })));
}
#[test]
fn grenade_overlap_is_bounded_and_allies_unhurt() {
    let mut c = ready();
    let mut f = frame();
    f.units[1].pos = Point::new(350000, 300000);
    f.units.push(unit(3, 0, 350000, 300000));
    tick(
        &mut c,
        &mut f,
        Some(Action::Grenade {
            aim: Point::new(350000, 300000),
        }),
    );
    let out = advance(&mut c, &mut f, 130);
    let raw: i64 = out
        .iter()
        .filter_map(|x| match x {
            Cmd::Damage { target: 2, raw, .. } => Some(*raw),
            _ => None,
        })
        .sum();
    assert!(raw > 0 && raw <= 95 + 100 * 80 / 100);
    assert!(!out
        .iter()
        .any(|x| matches!(x, Cmd::Damage { target: 3, .. })));
}
#[test]
fn concealed_enemies_can_take_committed_area_damage() {
    let mut c = ready();
    let mut f = frame();
    tick(
        &mut c,
        &mut f,
        Some(Action::Grenade {
            aim: Point::new(350000, 300000),
        }),
    );
    f.units[1].visible = false;
    let out = advance(&mut c, &mut f, 130);
    assert!(out
        .iter()
        .any(|x| matches!(x, Cmd::Damage { target: 2, .. })));
}
#[test]
fn rocket_is_a_single_fixed_collision_not_homing() {
    let mut c = ready();
    let mut f = frame();
    tick(&mut c, &mut f, Some(Action::EquipRocket));
    advance(&mut c, &mut f, 42);
    tick(
        &mut c,
        &mut f,
        Some(Action::FireRocket {
            aim: Point::new(450000, 300000),
        }),
    );
    f.units[1].pos.y += 90000;
    let out = advance(&mut c, &mut f, 60);
    assert!(!out
        .iter()
        .any(|x| matches!(x, Cmd::Damage { target: 2, .. })));
    assert!(c.rocket.is_none());
    assert!(c.cooldowns[2] > f.tick);
}
#[test]
fn rocket_reaches_its_target_at_configured_speed_and_hits_once() {
    let mut c = ready();
    let mut f = frame();
    tick(&mut c, &mut f, Some(Action::EquipRocket));
    advance(&mut c, &mut f, 42);
    tick(
        &mut c,
        &mut f,
        Some(Action::FireRocket {
            aim: Point::new(350000, 300000),
        }),
    );
    let out = advance(&mut c, &mut f, 8);
    let hits: Vec<_> = out
        .iter()
        .filter(|x| matches!(x, Cmd::Damage { target: 2, .. }))
        .collect();
    assert_eq!(hits.len(), 1);
    assert!(matches!(hits[0], Cmd::Damage { raw: 300, .. }));
    assert!(!out
        .iter()
        .any(|x| matches!(x, Cmd::Damage { self_hit: true, .. })));
    assert!(c.rocket.is_none());
}
#[test]
fn bot_loses_hidden_tracking_and_requires_actual_carrier() {
    let mut c = ready();
    let mut f = frame();
    tick(
        &mut c,
        &mut f,
        Some(Action::Bot {
            aim: Point::new(350000, 300000),
        }),
    );
    advance(&mut c, &mut f, 22);
    assert!(c.bot.is_some());
    tick(&mut c, &mut f, None);
    assert_eq!(c.bot.as_ref().unwrap().target, Some(2));
    f.units[1].visible = false;
    tick(&mut c, &mut f, None);
    assert_eq!(c.bot.as_ref().unwrap().target, None);
}
#[test]
fn death_keeps_weapon_stock_cooldowns_and_never_explodes() {
    let mut c = ready();
    let mut f = frame();
    c.gun = 4;
    pack(&mut c, &mut f);
    c.cooldowns[2] = 4000;
    f.units[0].hp = 0;
    let out = tick(&mut c, &mut f, None);
    assert!(c.packs.is_empty());
    assert_eq!(c.stock, [1, 1, 1]);
    assert_eq!(c.gun, 4);
    assert_eq!(c.cooldowns[2], 4000);
    assert_eq!(c.credits, 1400);
    assert!(!out.iter().any(|x| matches!(x, Cmd::Damage { .. })));
}
#[test]
fn manual_commands_are_identical_across_all_profiles() {
    let mut a = ready();
    let mut b = ready();
    let mut fa = frame();
    let mut fb = frame();
    fa.profile.values = [0; 13];
    fb.profile.values = [100; 13];
    for t in 1..180 {
        let action = match t {
            1 => Some(Action::Pack {
                aim: Point::new(290000, 300000),
            }),
            9 => Some(Action::Detonate { id: 0 }),
            10 => Some(Action::Steer {
                input: Point::new(0, 1000),
            }),
            12 => Some(Action::Grenade {
                aim: Point::new(350000, 300000),
            }),
            _ => None,
        };
        assert_eq!(
            tick(&mut a, &mut fa, action.clone()),
            tick(&mut b, &mut fb, action)
        );
    }
    assert_eq!(fa.units[0].pos, fb.units[0].pos);
    assert_eq!(a.metrics.requested_damage, b.metrics.requested_damage);
}
#[test]
fn independent_cores_and_replays_have_identical_commands() {
    let mut c = ready();
    let mut d = ready();
    let mut f = frame();
    let mut g = frame();
    f.automatic = true;
    g.automatic = true;
    for _ in 0..240 {
        assert_eq!(tick(&mut c, &mut f, None), tick(&mut d, &mut g, None));
        assert_eq!(
            serde_json::to_string(&c).unwrap(),
            serde_json::to_string(&d).unwrap()
        );
    }
}
#[test]
fn reaction_is_applied_once_for_a_stable_situation() {
    let mut c = ready();
    let mut f = frame();
    f.automatic = true;
    f.profile.values[3] = 0;
    advance(&mut c, &mut f, 90);
    assert_eq!(c.reaction_events, 1);
    assert!(c.metrics.casts > 0 || !c.candidates.is_empty());
}
#[test]
fn every_stat_level_considers_all_action_families() {
    for n in [0, 20, 50, 80, 100] {
        let c = ready();
        let mut f = frame();
        f.profile.values = [n; 13];
        let all = c.evaluate(&f, &f.units[0]);
        for family in [
            "shoot",
            "grenade",
            "bot",
            "single_launch",
            "double_launch",
            "rocket",
            "escape",
        ] {
            assert!(all.iter().any(|c| c.family == family), "{family} at {n}");
        }
    }
}
#[test]
fn hidden_current_positions_health_and_velocity_do_not_change_decisions() {
    let mut c = ready();
    let mut f = frame();
    f.units.push(unit(9, 1, 800000, 800000));
    f.units[2].visible = false;
    let a = serde_json::to_string(&c.evaluate(&f, &f.units[0])).unwrap();
    f.units[2].pos = Point::new(310000, 300000);
    f.units[2].hp = 1;
    f.units[2].velocity = Point::new(99999, 99999);
    let b = serde_json::to_string(&c.evaluate(&f, &f.units[0])).unwrap();
    assert_eq!(a, b);
    c.sightings.clear();
}
#[test]
fn planning_is_bounded_and_continuation_uses_selected_branch() {
    let mut c = ready();
    let mut f = frame();
    f.automatic = true;
    f.profile.values[12] = 100;
    for n in 3..20 {
        f.units.push(unit(n, 1, 350000 + n as i64 * 1000, 300000));
    }
    advance(&mut c, &mut f, 40);
    assert!(c.evaluated <= 64);
    assert!(c.continuation.len() <= 3);
    if let Some(p) = &c.plan {
        assert_eq!(p.family, c.continuation[0]);
    }
}

fn paired_case(seed: u64, stat: usize, value: i64) -> Vec<(u64, Cmd)> {
    let mut c = ready();
    let mut f = frame();
    f.seed = seed;
    f.tick = 300;
    f.automatic = true;
    f.profile.known = true;
    f.profile.values[stat] = value;
    f.units[0].hp = if seed % 5 == 0 { 500 } else { 1800 };
    f.units[1].pos.x = 320000 + (seed % 12) as i64 * 10000;
    f.units[1].pos.y = 280000 + (seed % 5) as i64 * 10000;
    f.units[1].velocity = Point::new(0, 18000);
    if seed % 2 == 0 {
        let mut a = unit(3, 0, 330000 + (seed % 8) as i64 * 10000, 310000);
        a.hp = 1000;
        f.units.push(a);
    }
    if seed % 3 == 0 {
        f.units.push(unit(4, 1, 340000, 320000));
    }
    f.assignment = Some(Point::new(250000, 380000));
    f.missed_wave = (seed % 7) as i64 * 15;
    if seed % 4 == 0 {
        f.hazards.push(Hazard {
            id: seed,
            pos: Point::new(335000, 300000),
            radius: 30000,
            until: 600,
            severity: 80,
        });
    }
    if stat == 11 {
        c.sightings.insert(
            9,
            Sighting {
                pos: Point::new(335000 + (seed % 4) as i64 * 15000, 300000),
                velocity: Point::default(),
                tick: 0,
            },
        );
    }
    if seed % 5 == 1 {
        f.units[1].kind = 1;
        f.units[1].farmable = true;
        for n in 5..8 {
            let mut u = unit(n, 1, f.units[1].pos.x, 295000 + (n - 5) as i64 * 5000);
            u.kind = 1;
            u.farmable = true;
            f.units.push(u);
        }
    }
    if seed % 7 == 0 {
        c.stock = [2, 0, 0];
        f.unlocked[2] = false;
    }
    let mut commands = vec![];
    for _ in 0..100 {
        for cmd in tick(&mut c, &mut f, None) {
            if !matches!(cmd, Cmd::Rejected { .. }) {
                commands.push((f.tick, cmd));
            }
        }
    }
    commands
}
#[test]
fn all_thirteen_stats_have_three_independent_behavior_pairs() {
    for stat in 0..13 {
        let mut cases = vec![];
        for seed in 1..=240 {
            if paired_case(seed, stat, 20) != paired_case(seed, stat, 80) {
                cases.push(seed);
                if cases.len() == 3 {
                    break;
                }
            }
        }
        assert_eq!(
            cases.len(),
            3,
            "{} has only {:?} behavior pairs",
            PROFILE_NAMES[stat],
            cases
        );
        println!("{}: paired fixture seeds {:?}", PROFILE_NAMES[stat], cases);
    }
}
#[test]
fn stats_change_timing_quality_and_horizon_separately() {
    let mut p = Profile::default();
    p.values[3] = 20;
    let a = p.reaction();
    p.values[3] = 80;
    assert!(p.reaction() < a);
    let interval = p.steering_ticks();
    p.values[10] = 80;
    assert!(p.steering_ticks() < interval);
    assert_eq!(p.reaction(), 10);
    p.values[12] = 100;
    assert_eq!(p.horizon(), 3);
    for seed in 0..100 {
        assert!(p.timing_error(seed, 1).abs() <= 1);
    }
}
#[test]
fn frozen_state_roundtrip_preserves_future_commands() {
    let mut c = ready();
    let mut f = frame();
    pack(&mut c, &mut f);
    tick(&mut c, &mut f, Some(Action::Detonate { id: 0 }));
    let mut restored: Core = serde_json::from_str(&serde_json::to_string(&c).unwrap()).unwrap();
    let mut other = f.clone();
    for _ in 0..90 {
        assert_eq!(
            tick(&mut c, &mut f, None),
            tick(&mut restored, &mut other, None)
        );
    }
}
#[test]
fn stronger_review_and_accuracy_reduce_average_error() {
    let mut a = Profile::default();
    let mut b = a.clone();
    a.values[1] = 20;
    b.values[1] = 80;
    let low: i64 = (1..1000).map(|s| a.estimation_error(s, 7).abs()).sum();
    let high: i64 = (1..1000).map(|s| b.estimation_error(s, 7).abs()).sum();
    assert!(high < low / 2);
    a.values[4] = 20;
    b.values[4] = 80;
    let mut u = unit(2, 1, 350000, 300000);
    u.velocity = Point::new(0, 30000);
    let expected = u.pos.add(Point::new(0, 30000));
    let low: i64 = (1..100)
        .map(|s| distance(a.aim(Point::new(300000, 300000), &u, 60, s, 1), expected))
        .sum();
    let high: i64 = (1..100)
        .map(|s| distance(b.aim(Point::new(300000, 300000), &u, 60, s, 1), expected))
        .sum();
    assert!(high < low / 2);
}
#[test]
fn ego_and_aggression_are_different_preferences() {
    let c = ready();
    let mut f = frame();
    f.units[1].pos.x = 380000;
    f.hazards.push(Hazard {
        id: 1,
        pos: Point::new(350000, 300000),
        radius: 100000,
        until: 900,
        severity: 50,
    });
    let score = |f: &Frame, family: &str| {
        c.evaluate(f, &f.units[0])
            .into_iter()
            .find(|a| a.family == family)
            .unwrap()
            .score
    };
    f.profile.values[0] = 20;
    let low = score(&f, "single_launch");
    f.profile.values[0] = 80;
    assert!(score(&f, "single_launch") > low);
    f.profile.values[0] = 50;
    f.profile.values[6] = 20;
    let (entry, shoot) = (score(&f, "single_launch"), score(&f, "shoot"));
    f.profile.values[6] = 80;
    assert!(score(&f, "single_launch") > entry);
    assert_eq!(score(&f, "shoot"), shoot);
}
#[test]
fn macro_prices_missed_waves_and_roaming_prices_rotations() {
    let c = ready();
    let mut f = frame();
    f.assignment = Some(Point::new(250000, 400000));
    f.missed_wave = 60;
    let score = |f: &Frame| {
        c.evaluate(f, &f.units[0])
            .into_iter()
            .find(|a| a.family == "rotate")
            .unwrap()
            .score
    };
    f.profile.values[9] = 20;
    let low = score(&f);
    f.profile.values[9] = 80;
    assert!(score(&f) < low);
    f.profile.values[9] = 50;
    f.profile.values[8] = 20;
    let low = score(&f);
    f.profile.values[8] = 80;
    assert!(score(&f) > low);
}
#[test]
fn order_distinguishes_supported_and_solo_entries() {
    let c = ready();
    let mut f = frame();
    f.units[1].pos.x = 380000;
    let score = |f: &Frame| {
        c.evaluate(f, &f.units[0])
            .into_iter()
            .find(|a| a.family == "single_launch")
            .unwrap()
            .score
    };
    f.profile.values[7] = 20;
    let low = score(&f);
    f.profile.values[7] = 80;
    assert!(score(&f) < low);
    f.units.push(unit(3, 0, 350000, 320000));
    f.units.push(unit(4, 0, 360000, 310000));
    f.profile.values[7] = 20;
    let low = score(&f);
    f.profile.values[7] = 80;
    assert!(score(&f) > low);
}
#[test]
fn high_positioning_chooses_a_safer_landing() {
    let c = ready();
    let mut f = frame();
    f.units[1].pos.x = 400000;
    f.hazards.push(Hazard {
        id: 2,
        pos: Point::new(378000, 300000),
        radius: 17000,
        until: 900,
        severity: 20,
    });
    let landing = |f: &Frame| {
        c.evaluate(f, &f.units[0])
            .into_iter()
            .find(|a| a.family == "single_launch")
            .unwrap()
            .landing
    };
    f.profile.values[2] = 20;
    let low = landing(&f);
    f.profile.values[2] = 80;
    let high = landing(&f);
    assert!(distance(high, f.hazards[0].pos) > distance(low, f.hazards[0].pos));
}
#[test]
fn cooldown_haste_never_changes_preparation_or_stock_cost() {
    let mut c = ready();
    let mut f = frame();
    f.haste = 100;
    tick(
        &mut c,
        &mut f,
        Some(Action::Grenade {
            aim: Point::new(350000, 300000),
        }),
    );
    assert_eq!(c.cooldowns[0], f.tick + GRENADE_COOLDOWN as u64 / 2);
    assert_eq!(
        c.cast.as_ref().unwrap().release,
        f.tick + GRENADE_PREP as u64
    );
    assert_eq!(c.stock[1], 0);
}
#[test]
fn expired_credit_journal_entries_cannot_be_rewarded_again() {
    let mut c = Core::default();
    let mut f = frame();
    for id in 1..=300 {
        f.credits = vec![CreditEvent { id, kind: 2 }];
        tick(&mut c, &mut f, None);
    }
    c.credits = 100;
    f.credits = vec![CreditEvent { id: 1, kind: 0 }];
    tick(&mut c, &mut f, None);
    assert_eq!(c.credits, 100);
    assert!(c.seen_credits.len() <= 256);
}
#[test]
fn an_armed_rocket_can_be_replanned_after_target_loss() {
    let mut c = ready();
    let mut f = frame();
    tick(&mut c, &mut f, Some(Action::EquipRocket));
    advance(&mut c, &mut f, 42);
    let choices = c.evaluate(&f, &f.units[0]);
    assert!(choices.iter().any(|c| c.family == "rocket" && c.legal));
}

#[test]
fn a_committed_satchel_hits_hidden_targets_but_cannot_plan_from_them() {
    let mut c = ready();
    let mut f = frame();
    pack(&mut c, &mut f);
    advance(&mut c, &mut f, PACK_ARM as usize);
    f.units[1].pos = c.packs[0].pos;
    f.units[1].visible = false;
    assert!(c
        .evaluate(&f, &f.units[0])
        .iter()
        .all(|a| a.target != Some(2)));
    let out = tick(&mut c, &mut f, Some(Action::Detonate { id: 0 }));
    assert_eq!(
        out.iter()
            .filter(|v| matches!(v, Cmd::Damage { target: 2, .. }))
            .count(),
        1
    );
}

#[test]
fn an_empty_rocket_explosion_does_not_count_as_an_enemy_hit() {
    let mut c = ready();
    let mut f = frame();
    f.units[1].pos = Point::new(600000, 600000);
    tick(&mut c, &mut f, Some(Action::EquipRocket));
    advance(&mut c, &mut f, ROCKET_PREP as usize);
    tick(
        &mut c,
        &mut f,
        Some(Action::FireRocket {
            aim: Point::new(450000, 300000),
        }),
    );
    advance(&mut c, &mut f, 60);
    assert!(c.rocket.is_none());
    assert_eq!(c.metrics.rocket_hits, 0);
}
