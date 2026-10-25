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
    // the free charge of each utility plus one bought (Omen's Buy Phase, round 143)
    Core {
        stock: [1, 1, 1],
        ..Default::default()
    }
}
/// Two Blast Packs (the free one and a bought one) and no grenade or bot charge.
fn packs_only(c: &mut Core) {
    c.stock = [1, 0, 0];
    c.cooldowns[0] = 1_000_000;
    c.cooldowns[1] = 1_000_000;
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
            aim: Point::new(270000, 300000),
        }),
    );
    assert_eq!(before, c.stock);
    assert!(c.cast.is_none());
    f.unlocked = [false; 3];
    tick(&mut c, &mut f, Some(Action::EquipRocket));
    assert_eq!(c.cooldowns, [0; 3]);
}
#[test]
fn free_charge_recharges_on_its_cooldown_and_a_bought_one_never_refills() {
    // round 143: Omen's free smoke model on every utility
    let mut c = Core::default();
    let mut f = frame();
    assert_eq!([0, 1, 2].map(|n| c.charges(n, f.tick)), [1, 1, 1]);
    tick(
        &mut c,
        &mut f,
        Some(Action::Grenade {
            aim: Point::new(350000, 300000),
        }),
    );
    assert_eq!(c.charges(1, f.tick), 0);
    assert_eq!(c.cooldowns[0], f.tick + GRENADE_COOLDOWN as u64);
    let thrown = f.tick;
    advance(&mut c, &mut f, GRENADE_COOLDOWN as usize - 1);
    assert_eq!(c.charges(1, f.tick), 0);
    tick(&mut c, &mut f, None);
    assert_eq!(f.tick, thrown + GRENADE_COOLDOWN as u64);
    assert_eq!(c.charges(1, f.tick), 1);
    assert_eq!(c.stock, [0; 3]);
    // the pack's free charge comes back after PACK_COOLDOWN, the bought one is spent for good
    let (mut c, mut f) = (ready(), frame());
    pack(&mut c, &mut f);
    tick(&mut c, &mut f, Some(Action::Detonate { id: 0 }));
    assert_eq!(c.stock[0], 1);
    advance(&mut c, &mut f, 70);
    pack(&mut c, &mut f);
    assert_eq!(c.stock[0], 0);
    assert_eq!(c.charges(0, f.tick), 0);
    advance(&mut c, &mut f, PACK_COOLDOWN as usize);
    assert_eq!(c.charges(0, f.tick), 1);
    assert_eq!(c.stock[0], 0);
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
    assert_eq!((c.stock[0], c.charges(0, f.tick)), (1, 2));
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
    // round 143: 30 + 25% AP (AP 100)
    assert!(out.contains(&Cmd::Damage {
        id: 2,
        target: 2,
        raw: 55,
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
    assert!(after.y > 0 && after.y <= AIR_ACCEL / TPS);
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
    // the free charge was committed (its cooldown runs); the bought one is kept
    assert_eq!(c.stock[1], 1);
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
    assert!(
        raw > 0
            && raw
                <= GRENADE_DAMAGE
                    + GRENADE_AP
                    + 2 * (CLUSTER_DAMAGE + 100 * CLUSTER_AP_NUM / CLUSTER_AP_DEN)
    );
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
    let out = advance(&mut c, &mut f, rocket_eta(42000, 0) as usize);
    let hits: Vec<_> = out
        .iter()
        .filter(|x| matches!(x, Cmd::Damage { target: 2, .. }))
        .collect();
    assert_eq!(hits.len(), 1);
    // round 143: (240 + 150% AP) at 8,000 from the centre
    assert!(
        matches!(hits[0], Cmd::Damage { raw: 343, .. }),
        "{:?}",
        hits[0]
    );
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
    assert_eq!(c.bot.as_ref().unwrap().target, None);
    f.units[1].pos = c.bot.as_ref().unwrap().pos.add(Point::new(25000, 0));
    tick(&mut c, &mut f, None);
    assert_eq!(c.bot.as_ref().unwrap().target, Some(2));
    f.units[1].visible = false;
    tick(&mut c, &mut f, None);
    assert_eq!(c.bot.as_ref().unwrap().target, None);
}
#[test]
fn death_keeps_weapon_and_ult_loses_bought_charges_and_restores_free_ones() {
    let mut c = ready();
    let mut f = frame();
    c.gun = 4;
    pack(&mut c, &mut f);
    c.cooldowns[0] = 5000;
    c.cooldowns[2] = 4000;
    f.units[0].hp = 0;
    let out = tick(&mut c, &mut f, None);
    assert!(c.packs.is_empty());
    // round 143, as Omen: bought charges are lost, the free ones come back at once
    assert_eq!(c.stock, [0; 3]);
    assert_eq!([0, 1, 2].map(|n| c.charges(n, f.tick)), [1, 1, 1]);
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
        packs_only(&mut c);
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
    assert_eq!(c.stock[1], 1);
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

fn equip(c: &mut Core, f: &mut Frame) {
    tick(c, f, Some(Action::EquipRocket));
    advance(c, f, ROCKET_PREP as usize);
}
#[test]
fn v2_rocket_falloff_is_exact_and_continuous() {
    assert_eq!(rocket_damage(400, 0), 400);
    assert_eq!(rocket_damage(400, ROCKET_RADIUS / 2), 250);
    assert_eq!(rocket_damage(400, ROCKET_RADIUS), 100);
    assert_eq!(rocket_damage(400, ROCKET_RADIUS + 1), 0);
    for d in 0..ROCKET_RADIUS {
        assert!(rocket_damage(400, d) >= rocket_damage(400, d + 1));
    }
}
#[test]
fn v2_rocket_explodes_at_empty_ground_endpoint() {
    let (mut c, mut f) = (ready(), frame());
    f.units[1].pos = Point::new(600000, 600000);
    equip(&mut c, &mut f);
    let end = Point::new(400000, 300000);
    tick(&mut c, &mut f, Some(Action::FireRocket { aim: end }));
    assert_eq!(c.rocket.as_ref().unwrap().endpoint, end);
    let out = advance(&mut c, &mut f, rocket_eta(100000, 0) as usize);
    assert!(out
        .iter()
        .any(|v| matches!(v,Cmd::Visual{tag,pos,..}if tag=="rocket_blast"&&*pos==end)));
    assert!(c.rocket.is_none());
}
#[test]
fn v2_rocket_sweeps_first_unit_wall_and_visibility_independently() {
    let mut f = frame();
    f.units[1].pos = Point::new(350000, 300000);
    let from = f.units[0].pos;
    let to = Point::new(450000, 300000);
    assert_eq!(f.rocket_impact(0, from, to, false).0.x, 342000);
    f.units[1].visible = false;
    assert_eq!(f.rocket_impact(0, from, to, true), (to, false));
    f.blocks.push(Rect {
        min: Point::new(330000, 290000),
        max: Point::new(331000, 310000),
    });
    let hit = f.rocket_impact(0, from, to, false);
    assert!(hit.1 && hit.0.x < 330000);
}
#[test]
fn v2_ground_recoil_is_ten_ticks_without_air_protection() {
    let (mut c, mut f) = (ready(), frame());
    equip(&mut c, &mut f);
    tick(
        &mut c,
        &mut f,
        Some(Action::FireRocket {
            aim: Point::new(450000, 300000),
        }),
    );
    assert!(c.flight.as_ref().unwrap().grounded);
    assert!(!c.air_protection);
    advance(&mut c, &mut f, 12);
    assert_eq!(f.units[0].pos.x, 290000);
    assert!(c.flight.is_none());
}
#[test]
fn v2_bound_raze_can_fire_but_cannot_recoil() {
    let (mut c, mut f) = (ready(), frame());
    equip(&mut c, &mut f);
    f.bound = true;
    tick(
        &mut c,
        &mut f,
        Some(Action::FireRocket {
            aim: Point::new(450000, 300000),
        }),
    );
    assert!(c.rocket.is_some());
    assert!(c.flight.is_none());
    assert!(!c.air_protection);
}
#[test]
fn v2_two_packs_and_backward_rocket_extend_real_flight() {
    let (mut c, mut f) = (ready(), frame());
    equip(&mut c, &mut f);
    pack(&mut c, &mut f);
    tick(&mut c, &mut f, Some(Action::Detonate { id: 0 }));
    pack(&mut c, &mut f);
    tick(&mut c, &mut f, Some(Action::Detonate { id: 0 }));
    let before = c.flight.clone().unwrap();
    let height = before.height(f.tick + 1);
    let aim = f.units[0].pos.sub(Point::new(80000, 0));
    tick(&mut c, &mut f, Some(Action::FireRocket { aim }));
    let after = c.flight.as_ref().unwrap();
    assert!(after.boosted);
    assert!(after.velocity.length() > before.velocity.length());
    assert!(after.velocity.length() <= ROCKET_MAX_SPEED);
    assert_eq!(after.height(f.tick), height);
    assert_eq!(after.until, (before.until + 18).min(after.born + 84));
    assert_eq!(c.metrics.triple_boosts, 1);
    assert_eq!(c.stock[0], 0);
    assert_eq!(c.armed_until, 0);
}
#[test]
fn v2_air_guard_is_single_and_clears_before_interrupted_self_damage() {
    let (mut c, mut f) = (ready(), frame());
    pack(&mut c, &mut f);
    let takeoff = tick(&mut c, &mut f, Some(Action::Detonate { id: 0 }));
    assert_eq!(
        takeoff
            .iter()
            .filter(|v| matches!(v, Cmd::AirProtection { active: true }))
            .count(),
        1
    );
    assert_eq!(c.protected_damage(100), 85);
    assert!(!advance(&mut c, &mut f, 3)
        .iter()
        .any(|v| matches!(v, Cmd::AirProtection { active: true })));
    let p = f.units[0].pos;
    c.grenades.push(Grenade {
        id: 999,
        from: p,
        aim: p,
        arrival: f.tick,
        explosion: f.tick + 1,
        cluster: f.tick + 30,
        born: f.tick - 1,
        ap: 0,
        main_done: false,
        centres: vec![],
        pos: p,
        direction: Point::default(),
        remaining: 0,
        bounces: 0,
    });
    f.stunned = true;
    let out = tick(&mut c, &mut f, None);
    let off = out
        .iter()
        .position(|v| matches!(v, Cmd::AirProtection { active: false }))
        .unwrap();
    let damage = out
        .iter()
        .position(|v| matches!(v, Cmd::Damage { self_hit: true, .. }))
        .unwrap();
    assert!(off < damage);
    assert_eq!(c.protected_damage(100), 100);
}
#[test]
fn v2_air_guard_ends_on_expiry_collision_and_death() {
    for mode in 0..3 {
        let (mut c, mut f) = (ready(), frame());
        pack(&mut c, &mut f);
        tick(&mut c, &mut f, Some(Action::Detonate { id: 0 }));
        let out = if mode == 0 {
            advance(&mut c, &mut f, 50)
        } else if mode == 1 {
            let p = f.units[0].pos;
            f.blocks.push(Rect {
                min: p.add(Point::new(3000, -30000)),
                max: p.add(Point::new(4000, 30000)),
            });
            tick(&mut c, &mut f, None)
        } else {
            c.died()
        };
        assert!(!c.air_protection);
        assert!(out
            .iter()
            .any(|v| matches!(v, Cmd::AirProtection { active: false })));
    }
}
#[test]
fn v2_steering_and_grenade_can_be_submitted_together() {
    let (mut c, mut f) = (ready(), frame());
    pack(&mut c, &mut f);
    tick(&mut c, &mut f, Some(Action::Detonate { id: 0 }));
    f.steering = Some(Point::new(0, 1000));
    let before = c.flight.as_ref().unwrap().velocity;
    tick(
        &mut c,
        &mut f,
        Some(Action::Grenade {
            aim: Point::new(350000, 300000),
        }),
    );
    assert!(c.cast.is_some());
    assert!(c.flight.as_ref().unwrap().velocity.y > before.y);
}
#[test]
fn v2_shared_judge_curve_and_identity_are_consistent() {
    use crate::valorant_guns::{basic_raw, judge_percent};
    assert_eq!(
        [
            judge_percent(0),
            judge_percent(10000),
            judge_percent(15000),
            judge_percent(20000)
        ],
        [300, 300, 185, 70]
    );
    assert_eq!(basic_raw(Gun::Judge, 100, 9000), 300);
    assert_eq!(Gun::Judge.price(), 1850);
    assert_eq!(Gun::Judge.modifiers(), (0, -25));
}
#[test]
fn buys_the_same_gun_and_armour_as_omen_at_equal_credits() {
    // round 143: valorant.rs Omen::buy, apart from the per-utility charges (his second smoke). Cases:
    // (credits, close fighters, gun, armour level, credits left, bought charges)
    for (credits, close, gun, armor, left, stock) in [
        (1000, 0, Gun::Sheriff, 0, 0, [1, 0, 0]),
        (2050, 0, Gun::Spectre, 0, 250, [1, 0, 0]),
        (2050, 2, Gun::Judge, 0, 0, [1, 0, 0]),
        (2900, 0, Gun::Vandal, 0, 0, [0, 0, 0]),
        (3700, 0, Gun::Vandal, 1, 200, [1, 0, 0]),
        (4000, 0, Gun::Vandal, 2, 100, [0, 0, 0]),
        (6000, 0, Gun::Operator, 2, 100, [1, 0, 0]),
        (7500, 0, Gun::Operator, 2, 1000, [1, 1, 1]),
    ] {
        let (mut c, mut f) = (Core::default(), frame());
        f.buy_allowed = true;
        for n in 0..close {
            let mut u = unit(10 + n, 1, 500000, 500000);
            u.melee = true;
            f.units.push(u);
        }
        c.credits = credits;
        tick(&mut c, &mut f, Some(Action::Buy));
        assert_eq!(
            (Gun::from_index(c.gun), c.armor, c.credits, c.stock),
            (gun, armor, left, stock),
            "{credits} credits"
        );
    }
    // no hysteresis or entry tweak: a richer visit upgrades exactly as Omen does
    let (mut c, mut f) = (Core::default(), frame());
    c.gun = Gun::Vandal.idx();
    c.credits = 8000;
    f.buy_allowed = true;
    let out = tick(&mut c, &mut f, Some(Action::Buy));
    assert!(out.contains(&Cmd::Gun {
        index: Gun::Operator.idx()
    }));
}
#[test]
fn credit_income_is_omens() {
    // start 1000, kill +300, assist +150, team kill +50, death +400, +200 every 20 s alive, cap 9000
    let (mut c, mut f) = (Core::default(), frame());
    f.credits = (0..4)
        .map(|kind| CreditEvent {
            id: kind as u64 + 1,
            kind,
        })
        .collect();
    tick(&mut c, &mut f, None);
    assert_eq!(c.credits, 1000 + 300 + 150 + 50 + 400);
    f.credits.clear();
    advance(&mut c, &mut f, 2 * ALIVE_TICKS as usize);
    assert_eq!(c.credits, 1900 + 2 * 200);
    c.credits = 8950;
    f.credits = vec![CreditEvent { id: 9, kind: 0 }];
    tick(&mut c, &mut f, None);
    assert_eq!(c.credits, MAX_CREDITS);
}
#[test]
fn v2_objective_rocket_requires_secure_or_champion_value() {
    let (mut c, mut f) = (ready(), frame());
    c.stock = [0; 3];
    c.pack_cooldown = 1_000_000;
    c.cooldowns[0] = 1_000_000;
    c.cooldowns[1] = 1_000_000;
    f.units[1].kind = 3;
    f.units[1].hp = 20000;
    f.units[1].max_hp = 20000;
    f.units[1].farmable = true;
    f.objective = Some(2);
    f.farm_window = false;
    assert!(!c
        .evaluate(&f, &f.units[0])
        .iter()
        .any(|a| a.family == "rocket" && a.legal));
    f.units[1].hp = 100;
    f.automatic = true;
    let mut fired = false;
    for _ in 0..100 {
        tick(&mut c, &mut f, None);
        fired |= c.metrics.rocket_shots > 0;
    }
    assert!(fired, "{:?}", c.candidates);
    assert!(c.metrics.rocket_objective_hits > 0);
}
#[test]
fn v2_rocket_cooldown_starts_on_firing_and_an_interrupted_equip_is_refunded() {
    // round 143: the cooldown is charged when the rocket fires, so a stun or silence wastes nothing
    let (mut c, mut f) = (ready(), frame());
    let out = tick(&mut c, &mut f, Some(Action::EquipRocket));
    assert!(c.cast.is_some());
    assert!(!out.is_empty());
    assert_eq!(c.cooldowns[2], 0);
    f.stunned = true;
    tick(&mut c, &mut f, None);
    assert!(c.last_rejection.contains("not charged"));
    f.stunned = false;
    tick(&mut c, &mut f, Some(Action::EquipRocket));
    assert!(c.cast.is_some());
    advance(&mut c, &mut f, ROCKET_PREP as usize);
    tick(
        &mut c,
        &mut f,
        Some(Action::FireRocket {
            aim: Point::new(450000, 300000),
        }),
    );
    let cd = c.cooldowns[2];
    assert_eq!(cd, f.tick + ROCKET_COOLDOWN as u64);
    f.tick = cd - 2;
    tick(&mut c, &mut f, Some(Action::EquipRocket));
    assert!(c.cast.is_none());
    tick(&mut c, &mut f, Some(Action::EquipRocket));
    assert!(c.cast.is_some());
}

#[test]
fn v2_rocket_range_clamps_once_and_does_not_follow_the_owner() {
    let (mut c, mut f) = (ready(), frame());
    f.units[1].pos = Point::new(700000, 700000);
    equip(&mut c, &mut f);
    let origin = f.units[0].pos;
    tick(
        &mut c,
        &mut f,
        Some(Action::FireRocket {
            aim: Point::new(900000, 300000),
        }),
    );
    assert_eq!(
        c.rocket.as_ref().unwrap().endpoint,
        origin.add(Point::new(ROCKET_RANGE, 0))
    );
    f.units[0].pos.y += 100000;
    let out = advance(&mut c, &mut f, rocket_eta(ROCKET_RANGE, 0) as usize);
    assert!(out.iter().any(|v|matches!(v,Cmd::Visual{tag,pos,..}if tag=="rocket_blast"&&*pos==origin.add(Point::new(ROCKET_RANGE,0)))));
}

#[test]
fn v2_offensive_rocket_entry_finds_a_side_angle_and_executes() {
    let (mut c, mut f) = (ready(), frame());
    packs_only(&mut c);
    f.automatic = true;
    f.profile.values = [80; 13];
    f.units[0].ap = 250;
    f.units[1].pos = Point::new(430000, 300000);
    f.units.push(unit(3, 1, 440000, 302000));
    f.units.push(unit(4, 1, 435000, 310000));
    f.units.push(unit(5, 0, 405000, 330000));
    f.units.push(unit(6, 0, 410000, 330000));
    f.blocks.push(Rect {
        min: Point::new(360000, 294000),
        max: Point::new(362000, 305000),
    });
    for _ in 0..160 {
        tick(&mut c, &mut f, None);
    }
    assert!(c.metrics.rocket_entries > 0, "{:?}", c.candidates);
    assert!(c.metrics.rocket_hits > 0);
    assert_eq!(
        c.metrics.triple_boosts, 0,
        "This fixture must fire offensively, not spend its rocket backward"
    );
}

#[test]
fn v2_manual_purchase_does_not_apply_athlete_biases_or_memory() {
    for seed in 1..=40 {
        let (mut a, mut b) = (Core::default(), Core::default());
        a.credits = 4000;
        b.credits = 4000;
        a.sightings.insert(
            9,
            Sighting {
                pos: Point::new(340000, 300000),
                velocity: Point::default(),
                tick: 0,
            },
        );
        b.sightings = a.sightings.clone();
        let mut f = frame();
        f.seed = seed;
        f.buy_allowed = true;
        f.profile.values = [20; 13];
        f.hazards.push(Hazard {
            id: seed,
            pos: Point::new(340000, 300000),
            radius: 40000,
            until: 500,
            severity: 30,
        });
        let mut g = f.clone();
        g.profile.values = [80; 13];
        assert_eq!(
            tick(&mut a, &mut f, Some(Action::Buy)),
            tick(&mut b, &mut g, Some(Action::Buy))
        );
    }
}

#[test]
fn v3_boost_preserves_caps_duration_and_gradual_strafing() {
    let (mut c, mut f) = (ready(), frame());
    pack(&mut c, &mut f);
    tick(&mut c, &mut f, Some(Action::Detonate { id: 0 }));
    let a = c.flight.clone().unwrap();
    assert_eq!(a.velocity.length(), 200000);
    assert_eq!(a.until - f.tick, FLIGHT_TICKS as u64);
    f.steering = Some(Point::new(0, 1000));
    tick(&mut c, &mut f, None);
    let b = c.flight.as_ref().unwrap();
    assert!(b.velocity.y > 0 && b.velocity.y <= AIR_ACCEL / TPS);
    assert!(distance(b.velocity, a.velocity) <= AIR_ACCEL / TPS);
    assert!(b.velocity.x > 180000 && b.velocity.length() <= MAX_SPEED);
    let born = b.born;
    pack(&mut c, &mut f);
    tick(&mut c, &mut f, Some(Action::Detonate { id: 0 }));
    let b = c.flight.as_ref().unwrap();
    assert!(b.velocity.length() <= MAX_SPEED);
    assert!(b.until <= born + MAX_FLIGHT_TICKS as u64);
}

#[test]
fn v3_packs_attach_to_first_face_and_ignore_moving_units() {
    let mut f = frame();
    f.blocks.push(Rect {
        min: Point::new(310000, 280000),
        max: Point::new(314000, 320000),
    });
    let (p, a) = f
        .pack_attachment(Point::new(300000, 300000), Point::new(324000, 300000))
        .unwrap();
    assert_eq!(a.kind, "wall");
    assert_eq!(a.normal, Point::new(-1, 0));
    assert!((a.point.x - 310000).abs() <= 1);
    assert!(p.x < 310000);
    assert!(!f.blocked(p, PACK_CARRIER_RADIUS));
    let (_, a) = f
        .pack_attachment(Point::new(324000, 300000), Point::new(300000, 300000))
        .unwrap();
    assert_eq!(a.normal, Point::new(1, 0));
    let (_, a) = f
        .pack_attachment(Point::new(312000, 270000), Point::new(312000, 294000))
        .unwrap();
    assert_eq!(a.normal, Point::new(0, -1));
    let (_, a) = f
        .pack_attachment(Point::new(312000, 330000), Point::new(312000, 306000))
        .unwrap();
    assert_eq!(a.normal, Point::new(0, 1));
    f.blocks.clear();
    f.units[1].pos = Point::new(310000, 300000);
    let (_, a) = f
        .pack_attachment(f.units[0].pos, Point::new(320000, 300000))
        .unwrap();
    assert_eq!(a.kind, "ground");
}

#[test]
fn v3_structure_attachment_follows_then_drops_without_refresh() {
    let (mut c, mut f) = (ready(), frame());
    f.surfaces.push(Surface {
        id: 7,
        bounds: Rect {
            min: Point::new(312000, 280000),
            max: Point::new(316000, 320000),
        },
    });
    tick(
        &mut c,
        &mut f,
        Some(Action::Pack {
            aim: Point::new(320000, 300000),
        }),
    );
    advance(&mut c, &mut f, 7);
    assert_eq!(c.packs[0].attachment.surface, Some(7));
    let before = c.packs[0].clone();
    f.utility[0].hp = 19;
    f.surfaces[0].bounds.min.y += 12000;
    f.surfaces[0].bounds.max.y += 12000;
    let out = tick(&mut c, &mut f, None);
    assert_eq!(c.packs[0].pos.y, before.pos.y + 12000);
    assert!(out.iter().any(|v| matches!(v, Cmd::MoveUnit { .. })));
    assert_eq!(c.packs[0].hp, 19);
    f.surfaces.clear();
    tick(&mut c, &mut f, None);
    let p = &c.packs[0];
    assert_eq!(p.attachment.kind, "ground");
    assert_eq!((p.born, p.until), (before.born, before.until));
    assert_eq!(p.hp, 19);
}

#[test]
fn v3_commitment_revalidates_faces_and_trapped_detachment_removes_harmlessly() {
    let (mut c, mut f) = (ready(), frame());
    tick(
        &mut c,
        &mut f,
        Some(Action::Pack {
            aim: Point::new(290000, 300000),
        }),
    );
    f.blocks.push(Rect {
        min: Point::new(270000, 270000),
        max: Point::new(330000, 330000),
    });
    let out = advance(&mut c, &mut f, 7);
    assert!(c.packs.is_empty());
    assert_eq!(c.stock[0], 1);
    assert!(!out
        .iter()
        .any(|v| matches!(v, Cmd::Damage { .. } | Cmd::Spawn { .. })));
    f.blocks.clear();
    f.surfaces.push(Surface {
        id: 7,
        bounds: Rect {
            min: Point::new(312000, 280000),
            max: Point::new(316000, 320000),
        },
    });
    tick(
        &mut c,
        &mut f,
        Some(Action::Pack {
            aim: Point::new(320000, 300000),
        }),
    );
    advance(&mut c, &mut f, 7);
    f.surfaces.clear();
    f.blocks.push(Rect {
        min: Point::new(200000, 200000),
        max: Point::new(400000, 400000),
    });
    let out = tick(&mut c, &mut f, None);
    assert!(c.packs.is_empty());
    assert!(out.iter().any(|v| matches!(v, Cmd::Remove { .. })));
    assert!(!out.iter().any(|v| matches!(v, Cmd::Damage { .. })));
}

#[test]
fn v3_cluster_landings_are_committed_before_terrain_changes() {
    let (mut c, mut f) = (ready(), frame());
    let aim = Point::new(410000, 300000);
    tick(&mut c, &mut f, Some(Action::Grenade { aim }));
    advance(&mut c, &mut f, GRENADE_PREP as usize);
    let n = c.grenades[0].explosion - f.tick;
    advance(&mut c, &mut f, n as usize);
    let centres = c.grenades[0].centres.clone();
    assert_eq!(centres.len(), 4);
    assert_eq!(
        centres[0],
        aim.add(Point::new(CLUSTER_OFFSET, CLUSTER_OFFSET))
    );
    f.blocks.push(Rect {
        min: Point::new(410000, 290000),
        max: Point::new(440000, 330000),
    });
    let out = advance(&mut c, &mut f, CLUSTER_FUSE as usize);
    let actual: Vec<_> = out
        .iter()
        .filter_map(|v| match v {
            Cmd::Visual { tag, pos, .. } if tag == "cluster_blast" => Some(*pos),
            _ => None,
        })
        .collect();
    assert_eq!(actual, centres);
    assert!(c.grenades.is_empty());
}

#[test]
fn v3_rocket_acceleration_eta_and_fixed_endpoint_agree() {
    assert_eq!(rocket_tick_distance(0), 2222);
    assert_eq!(rocket_tick_distance(18), 10000);
    for n in 1..18 {
        assert!(rocket_tick_distance(n) > rocket_tick_distance(n - 1));
    }
    assert_eq!(rocket_eta(ROCKET_RANGE, 0), 20);
    let (mut c, mut f) = (ready(), frame());
    f.units[1].pos = Point::new(800000, 800000);
    equip(&mut c, &mut f);
    let out = tick(
        &mut c,
        &mut f,
        Some(Action::FireRocket {
            aim: Point::new(540000, 300000),
        }),
    );
    let id = c.rocket.as_ref().unwrap().id;
    assert!(out
        .iter()
        .any(|v| matches!(v,Cmd::Warning{until,..}if *until==f.tick+20)));
    let out = advance(&mut c, &mut f, 19);
    assert!(c.rocket.is_some());
    assert!(!out.iter().any(|v| matches!(v, Cmd::ClearWarning { .. })));
    let out = tick(&mut c, &mut f, None);
    assert!(c.rocket.is_none());
    assert!(out
        .iter()
        .any(|v| matches!(v,Cmd::ClearWarning{id:i}if *i==id)));
}

#[test]
fn v3_warning_repredicts_visible_blockers_without_retargeting_or_hidden_leaks() {
    let (mut c, mut f) = (ready(), frame());
    f.units[1].pos = Point::new(800000, 800000);
    equip(&mut c, &mut f);
    let end = Point::new(420000, 300000);
    tick(&mut c, &mut f, Some(Action::FireRocket { aim: end }));
    f.units[1].pos = Point::new(390000, 300000);
    tick(&mut c, &mut f, None);
    assert_eq!(c.rocket.as_ref().unwrap().impact.x, 382000);
    assert_eq!(c.rocket.as_ref().unwrap().endpoint, end);
    f.units[1].visible = false;
    tick(&mut c, &mut f, None);
    assert_eq!(c.rocket.as_ref().unwrap().impact, end);
    f.units[1].pos.x = 470000;
    tick(&mut c, &mut f, None);
    assert_eq!(c.rocket.as_ref().unwrap().impact, end);
    let id = c.rocket.as_ref().unwrap().id;
    f.units[0].hp = 0;
    let out = tick(&mut c, &mut f, None);
    assert!(out
        .iter()
        .any(|v| matches!(v,Cmd::ClearWarning{id:i}if *i==id)));
}

#[test]
fn v3_rocket_pushes_only_eligible_victims_once_after_damage() {
    let (mut c, mut f) = (ready(), frame());
    f.units[1].pos = Point::new(420000, 310000);
    for (id, kind) in [(3, 1), (4, 5), (5, 3), (6, 4), (7, 0)] {
        let mut u = unit(id, 1, 420000, 312000);
        u.kind = kind;
        u.push_immune = id == 7;
        f.units.push(u);
    }
    f.units.push(unit(8, 0, 420000, 312000));
    equip(&mut c, &mut f);
    tick(
        &mut c,
        &mut f,
        Some(Action::FireRocket {
            aim: Point::new(420000, 300000),
        }),
    );
    let out = advance(&mut c, &mut f, 40);
    let ids: Vec<_> = out
        .iter()
        .filter_map(|v| match v {
            Cmd::Knockback { target, ticks, to } => {
                assert_eq!(*ticks, 12);
                assert!(to.y > 310000);
                Some(*target)
            }
            _ => None,
        })
        .collect();
    assert_eq!(ids, vec![2, 3, 4]);
    for id in ids {
        let damage = out
            .iter()
            .position(|v| matches!(v,Cmd::Damage{target,..}if *target==id))
            .unwrap();
        let push = out
            .iter()
            .position(|v| matches!(v,Cmd::Knockback{target,..}if *target==id))
            .unwrap();
        assert!(damage < push);
    }
}

fn dodge_fixture(level: i64) -> (Core, Frame) {
    let (mut c, mut f) = (ready(), frame());
    f.units[1].pos = Point::new(400000, 342000);
    f.defenders.push(Defender {
        id: 2,
        profile: Profile {
            values: [level; 13],
            known: true,
        },
        observed: true,
    });
    equip(&mut c, &mut f);
    tick(
        &mut c,
        &mut f,
        Some(Action::FireRocket {
            aim: Point::new(420000, 300000),
        }),
    );
    (c, f)
}

#[test]
fn v3_dodge_reaction_is_once_per_rocket_and_profiles_change_timing() {
    let (mut a, mut f) = dodge_fixture(20);
    let (mut b, mut g) = dodge_fixture(80);
    let low = a.dodges[&2].ready;
    let high = b.dodges[&2].ready;
    assert!(high < low);
    for _ in 0..16 {
        tick(&mut a, &mut f, None);
        tick(&mut b, &mut g, None);
        assert_eq!(a.dodges[&2].ready, low);
        assert_eq!(b.dodges[&2].ready, high);
    }
    assert!(b.dodges[&2].goal.is_some());
}

#[test]
fn v3_dodge_observation_control_and_blocked_routes_are_respected() {
    let (mut c, mut f) = dodge_fixture(100);
    c.dodges.clear();
    f.defenders[0].observed = false;
    advance(&mut c, &mut f, 2);
    assert!(c.dodges.is_empty());
    f.defenders[0].observed = true;
    tick(&mut c, &mut f, None);
    let ready = c.dodges[&2].ready;
    f.units[1].controlled = true;
    let n = ready - f.tick;
    advance(&mut c, &mut f, n as usize);
    assert_eq!(c.dodges[&2].reason, "Movement disabled");
    f.units[1].controlled = false;
    f.blocks.push(Rect {
        min: Point::new(385000, 342000),
        max: Point::new(415000, 372000),
    });
    tick(&mut c, &mut f, None);
    assert_eq!(c.dodges[&2].reason, "No clear escape route");
}

#[test]
fn v3_slowed_defenders_do_not_get_normal_speed_escape_estimates() {
    let (mut slow, mut f) = dodge_fixture(100);
    let (mut fast, mut g) = (slow.clone(), f.clone());
    f.units[1].move_speed = 300;
    g.units[1].move_speed = 1020;
    advance(&mut slow, &mut f, 6);
    advance(&mut fast, &mut g, 6);
    assert_eq!(slow.dodges[&2].reason, "Late escape; reduce blast exposure");
    assert_eq!(fast.dodges[&2].reason, "Escape reachable");
}

#[test]
fn v3_manual_combat_and_rng_independence_include_attached_packs_and_rocket() {
    let (mut a, mut f) = dodge_fixture(50);
    let (mut b, mut g) = (a.clone(), f.clone());
    f.profile.values = [20; 13];
    g.profile.values = [80; 13];
    for _ in 0..40 {
        assert_eq!(tick(&mut a, &mut f, None), tick(&mut b, &mut g, None));
    }
}

#[test]
fn v32_opposite_input_reverses_within_one_flight_and_release_coasts() {
    let (mut c, mut f) = (ready(), frame());
    pack(&mut c, &mut f);
    tick(&mut c, &mut f, Some(Action::Detonate { id: 0 }));
    let born = c.flight.as_ref().unwrap().born;
    let old = c.flight.as_ref().unwrap().velocity;
    f.steering = Some(Point::new(-1000, 0));
    tick(&mut c, &mut f, None);
    let first = c.flight.as_ref().unwrap().velocity;
    assert!(first.x > 0 && first.x < old.x);
    assert!(distance(first, old) <= AIR_ACCEL / TPS);
    for _ in 0..24 {
        let old = c.flight.as_ref().unwrap().velocity;
        tick(&mut c, &mut f, None);
        let new = c.flight.as_ref().unwrap().velocity;
        assert!(distance(new, old) <= AIR_ACCEL / TPS);
        assert!(new.length() <= MAX_SPEED);
    }
    let reversed = c.flight.as_ref().unwrap().velocity;
    assert!(reversed.x < -190000 && reversed.y == 0);
    assert_eq!(c.flight.as_ref().unwrap().born, born);
    f.steering = Some(Point::default());
    advance(&mut c, &mut f, 3);
    assert_eq!(c.flight.as_ref().unwrap().velocity, reversed);
}

#[test]
fn v32_grenade_bounces_from_first_thin_wall_without_resetting_fuse() {
    let (mut c, mut f) = (ready(), frame());
    f.units[1].pos = Point::new(800000, 800000);
    f.blocks.push(Rect {
        min: Point::new(350000, 200000),
        max: Point::new(351000, 400000),
    });
    // The aimed point may lie inside a wall: it supplies direction and distance.
    tick(
        &mut c,
        &mut f,
        Some(Action::Grenade {
            aim: Point::new(410000, 300000),
        }),
    );
    advance(&mut c, &mut f, GRENADE_PREP as usize);
    let first = c.grenades[0].clone();
    assert!(first.aim.x < 300000 && first.aim.y == 300000);
    let mut saw_bounce = false;
    while f.tick < first.arrival {
        tick(&mut c, &mut f, None);
        let g = &c.grenades[0];
        assert!(!f.blocked(g.pos, 1000));
        assert_eq!(
            (g.arrival, g.explosion, g.cluster),
            (first.arrival, first.explosion, first.cluster)
        );
        saw_bounce |= g.bounces > 0;
    }
    assert!(saw_bounce);
    assert!(distance(c.grenades[0].pos, first.aim) < 100);
    let settled = c.grenades[0].pos;
    let out = advance(&mut c, &mut f, GRENADE_FUSE as usize);
    assert!(out
        .iter()
        .any(|v| matches!(v,Cmd::Visual{tag,pos,..}if tag=="grenade_blast"&&*pos==settled)));
    assert_eq!(
        c.grenades[0].centres[0],
        settled.add(Point::new(30000, 30000))
    );
}

#[test]
fn v32_reflection_handles_map_corners_structures_and_multiple_faces() {
    let mut f = frame();
    let (p, d, n) = f.ricochet(Point::new(950000, 950000), Point::new(1, 1), 30000, 1000);
    assert!(p.x < 950000 && p.y < 950000 && d.x < 0 && d.y < 0 && n == 1);
    assert!(!f.blocked(p, 1000));
    f.surfaces.push(Surface {
        id: 7,
        bounds: Rect {
            min: Point::new(350000, 280000),
            max: Point::new(351000, 320000),
        },
    });
    let (p, d, n) = f.ricochet(
        Point::new(300000, 300000),
        Point::new(1000, 0),
        100000,
        1000,
    );
    assert!(p.x < 300000 && d.x < 0 && n == 1);
    f.blocks.push(Rect {
        min: Point::new(250000, 200000),
        max: Point::new(251000, 400000),
    });
    let (p, _, n) = f.ricochet(
        Point::new(300000, 300000),
        Point::new(1000, 0),
        250000,
        1000,
    );
    assert!(n >= 2 && !f.blocked(p, 1000));
}

#[test]
fn v32_bot_preserves_throw_direction_acquires_near_front_and_retains_identity() {
    let (mut c, mut f) = (ready(), frame());
    f.units[1].pos = Point::new(380000, 300000);
    tick(
        &mut c,
        &mut f,
        Some(Action::Bot {
            aim: Point::new(400000, 300000),
        }),
    );
    advance(&mut c, &mut f, BOT_PREP as usize + 2);
    let b = c.bot.as_ref().unwrap();
    assert!(b.direction.x > 0 && b.direction.y == 0 && b.target.is_none());
    assert!(b.pos.x > 300000);
    let pos = b.pos;
    f.units[1].pos = pos.add(Point::new(-20000, 0));
    tick(&mut c, &mut f, None);
    assert!(c.bot.as_ref().unwrap().target.is_none());
    f.units[1].pos = c.bot.as_ref().unwrap().pos.add(Point::new(30000, 0));
    tick(&mut c, &mut f, None);
    assert_eq!(c.bot.as_ref().unwrap().target, Some(2));
    let pos = c.bot.as_ref().unwrap().pos;
    f.units[1].pos = pos.add(Point::new(40000, 0));
    f.units.push(unit(4, 1, pos.x + 25000, pos.y));
    tick(&mut c, &mut f, None);
    assert_eq!(c.bot.as_ref().unwrap().target, Some(2));
    f.units[1].visible = false;
    tick(&mut c, &mut f, None);
    assert_eq!(c.bot.as_ref().unwrap().target, Some(4));
}

#[test]
fn v32_bot_bounces_while_scouting_and_never_tracks_through_walls() {
    let (mut c, mut f) = (ready(), frame());
    f.blocks.push(Rect {
        min: Point::new(310000, 270000),
        max: Point::new(311000, 330000),
    });
    f.units[1].pos = Point::new(325000, 300000);
    tick(
        &mut c,
        &mut f,
        Some(Action::Bot {
            aim: Point::new(400000, 300000),
        }),
    );
    advance(&mut c, &mut f, BOT_PREP as usize + 12);
    let b = c.bot.as_ref().unwrap();
    assert!(b.direction.x < 0 && b.target.is_none() && b.pos.x < 308000);
    assert!(!f.blocked(b.pos, 2000));
}

#[test]
fn v32_short_range_requires_entry_and_entries_fire_airborne() {
    let (mut c, mut f) = (ready(), frame());
    f.profile.values = [80; 13];
    f.automatic = true;
    f.units[1].pos = Point::new(470000, 300000);
    let candidates = c.evaluate(&f, &f.units[0]);
    assert!(!candidates.iter().any(|v| v.family == "rocket" && v.legal));
    let entry = candidates
        .iter()
        .find(|v| v.family == "rocket_entry" && v.legal)
        .expect("reachable two-pack entry");
    let p = c.make_plan(&f, &f.units[0], entry, f.tick);
    assert!(
        p.steps
            .iter()
            .position(|s| matches!(s.action, Action::EquipRocket))
            .unwrap()
            < p.steps
                .iter()
                .position(|s| matches!(s.action, Action::Detonate { .. }))
                .unwrap()
    );
    // Execute the exact authored branch, rather than requiring all profiles to
    // choose it over a safe grenade in this artificial fixture.
    c.plan = Some(p);
    c.reaction_until = 0;
    let mut shot = false;
    for _ in 0..120 {
        let before = c.metrics.rocket_shots;
        tick(&mut c, &mut f, None);
        if c.metrics.rocket_shots > before {
            assert!(c.airborne());
            shot = true;
            break;
        }
    }
    assert!(shot);
}

#[test]
fn v32_athlete_can_commit_a_real_bank_shot_around_cover() {
    let (mut c, mut f) = (ready(), frame());
    f.profile.values = [100; 13];
    f.automatic = true;
    f.units[1].pos = Point::new(330000, 360000);
    f.blocks = vec![
        Rect {
            min: Point::new(350000, 200000),
            max: Point::new(351000, 420000),
        },
        Rect {
            min: Point::new(315000, 330000),
            max: Point::new(318000, 370000),
        },
    ];
    let candidates = c.evaluate(&f, &f.units[0]);
    let bank = candidates
        .iter()
        .find(|v| v.family == "grenade" && v.legal && v.reflection.is_some())
        .expect("wall bank clears cover");
    assert!(bank.reason.contains("wall bounces"));
    c.plan = Some(c.make_plan(&f, &f.units[0], bank, f.tick));
    c.reaction_until = 0;
    let mut hit = false;
    let mut bounced = false;
    for _ in 0..120 {
        let out = tick(&mut c, &mut f, None);
        bounced |= c.grenades.iter().any(|g| g.bounces > 0);
        hit |= out.iter().any(|v| {
            matches!(
                v,
                Cmd::Damage {
                    target: 2,
                    self_hit: false,
                    ..
                }
            )
        });
    }
    assert!(
        hit && bounced,
        "hit {hit}, bounced {bounced}, reason {}, rejection {}",
        bank.reason,
        c.last_rejection
    );
    assert!(c.evaluated <= PLANNER_CANDIDATES as usize);
}

#[test]
fn v32_new_terrain_repredicts_grenade_but_death_clears_its_warning() {
    let (mut c, mut f) = (ready(), frame());
    tick(
        &mut c,
        &mut f,
        Some(Action::Grenade {
            aim: Point::new(410000, 300000),
        }),
    );
    advance(&mut c, &mut f, GRENADE_PREP as usize + 4);
    let old = c.grenades[0].clone();
    f.blocks.push(Rect {
        min: Point::new(345000, 200000),
        max: Point::new(346000, 400000),
    });
    let out = tick(&mut c, &mut f, None);
    let g = &c.grenades[0];
    assert!(g.aim.x < 300000 && g.explosion == old.explosion && g.cluster == old.cluster);
    assert!(out
        .iter()
        .any(|v| matches!(v,Cmd::Warning{id,pos,..}if *id==g.id&&*pos==g.aim)));
    f.units[0].hp = 0;
    let out = tick(&mut c, &mut f, None);
    assert!(c.grenades.is_empty());
    assert!(out
        .iter()
        .any(|v| matches!(v,Cmd::ClearWarning{id}if *id==old.id)));
    assert!(!out.iter().any(|v| matches!(v, Cmd::Damage { .. })));
}

// ---- Round 143 ---------------------------------------------------------------------------------------------
fn flying(c: &mut Core, f: &Frame, velocity: Point, goal: Point) {
    c.flight = Some(Flight {
        satchel_count: 1,
        grounded: false,
        boosted: false,
        arc_born: f.tick,
        arc_height: 0,
        velocity,
        remainder: Point::default(),
        born: f.tick,
        until: f.tick + FLIGHT_TICKS as u64,
        input: Point::default(),
        next_input: f.tick,
        goal,
    });
}
#[test]
fn r143_strafe_turns_at_most_155_degrees_and_never_reverses_outright() {
    assert_eq!(AIR_STRAFE_MAX_DEG, 155);
    assert_eq!((AIR_STRAFE_COS, AIR_STRAFE_SIN), (-906, 423));
    // a straight-back input is turned to 155 degrees off the heading, keeping its strength
    let turned = strafe_limit(Point::new(200000, 0), Point::new(-240000, 0));
    assert_eq!(turned, Point::new(-217440, 101520));
    // anything inside the cone is untouched
    let inside = Point::new(-100000, 100000);
    assert_eq!(strafe_limit(Point::new(200000, 0), inside), inside);
    // held opposite input in flight: she curves round, never brakes through zero along the old line
    let (mut c, mut f) = (ready(), frame());
    flying(&mut c, &f, Point::new(200000, 0), Point::new(500000, 300000));
    let (mut slowest, mut widest) = (i64::MAX, 0);
    for _ in 0..30 {
        tick(
            &mut c,
            &mut f,
            Some(Action::Steer {
                input: Point::new(-1000, 0),
            }),
        );
        let Some(v) = c.flight.as_ref().map(|v| v.velocity) else {
            break;
        };
        slowest = slowest.min(v.length());
        widest = widest.max(v.y.abs());
    }
    // she curves round (sideways speed) and never stops dead on the old line, as an unlimited
    // straight-back input did
    assert!(slowest > 10000, "{slowest}");
    assert!(widest > 10000, "{widest}");
}
#[test]
fn r143_in_air_braking_lands_on_the_goal_without_reversing() {
    for far in [30000, 60000, 90000] {
        let (mut c, mut f) = (ready(), frame());
        f.automatic = true;
        f.units.truncate(1);
        let goal = f.units[0].pos.add(Point::new(far, 0));
        flying(&mut c, &f, Point::new(LAUNCH_SPEED, 0), goal);
        let mut furthest = 0;
        while c.flight.is_some() {
            tick(&mut c, &mut f, None);
            furthest = furthest.max(f.units[0].pos.x);
        }
        let miss = distance(f.units[0].pos, goal);
        assert!(miss <= 3000, "{far}: landed {miss} away");
        // no overshoot and fly back
        assert!(furthest - f.units[0].pos.x <= 3000, "{far}: came back {}", furthest - f.units[0].pos.x);
    }
}
#[test]
fn r143_a_crowd_in_the_blast_gets_the_rocket_within_reaction_plus_12_ticks() {
    let (mut c, mut f) = (ready(), frame());
    f.automatic = true;
    f.units.truncate(1);
    for n in 0..5 {
        f.units.push(unit(
            2 + n,
            1,
            390000 + (n as i64 % 2) * 12000,
            285000 + n as i64 * 8000,
        ));
    }
    let limit = f.profile.reaction() + 12 + 1;
    let mut decided = None;
    for t in 1..=limit {
        // a changing stream of bullets passing nearby must not keep re-arming her reaction
        f.hazards = vec![Hazard {
            id: 1_000_000 + t,
            pos: Point::new(300000, 330000),
            radius: 9000,
            until: f.tick + 8,
            severity: 55,
        }];
        tick(&mut c, &mut f, None);
        if c.decision > 0 && decided.is_none() {
            decided = Some(t);
        }
        if c
            .cast
            .as_ref()
            .is_some_and(|k| matches!(k.action, Action::EquipRocket))
        {
            break;
        }
    }
    assert!(decided.is_some_and(|t| t <= limit), "{decided:?}");
    assert!(
        c.cast
            .as_ref()
            .is_some_and(|k| matches!(k.action, Action::EquipRocket)),
        "{:?}",
        c.candidates
            .iter()
            .take(3)
            .map(|c| (&c.family, c.score))
            .collect::<Vec<_>>()
    );
}
#[test]
fn r143_bullets_and_hazards_alone_never_postpone_a_decision() {
    // a new bullet id every tick used to re-arm her reaction every tick, so she never decided at all
    let (mut c, mut f) = (ready(), frame());
    f.automatic = true;
    let mut first = None;
    for t in 1..=200u64 {
        f.hazards = vec![Hazard {
            id: 7_000_000 + t,
            pos: Point::new(320000, 340000),
            radius: 9000,
            until: f.tick + 8,
            severity: 55,
        }];
        tick(&mut c, &mut f, None);
        if c.decision > 0 && first.is_none() {
            first = Some(t);
        }
    }
    assert!(first.is_some_and(|t| t <= f.profile.reaction() + 13), "{first:?}");
    assert!(c.decision >= 2, "{}", c.decision);
}
#[test]
fn r143_an_armed_rocket_retargets_the_best_visible_champion() {
    let (mut c, mut f) = (ready(), frame());
    equip(&mut c, &mut f);
    f.automatic = true;
    f.units.push(unit(3, 1, 360000, 330000));
    c.plan = Some(Plan {
        family: "rocket".into(),
        target: Some(2),
        landing: f.units[0].pos,
        steps: vec![PlannedStep {
            at: f.tick + 5,
            action: Action::FireRocket {
                aim: Point::new(350000, 300000),
            },
            relative: false,
            intended: f.tick + 5,
            target_offset: Point::default(),
            reflection: None,
        }],
        score: 1,
        reason: String::new(),
    });
    f.units[1].visible = false;
    let mut fired = None;
    for _ in 0..20 {
        tick(&mut c, &mut f, None);
        if let Some(r) = c.rocket.as_ref() {
            fired = Some(r.endpoint);
            break;
        }
    }
    let end = fired.expect("the armed rocket was dropped instead of re-targeted");
    assert!(
        distance(end, Point::new(360000, 330000)) < ROCKET_RADIUS,
        "{end:?}"
    );
    assert_eq!(c.metrics.rocket_shots, 1);
}
#[test]
fn r143_armed_rocket_scores_60_more_and_ignores_the_danger_she_stands_in() {
    let (mut c, mut f) = (ready(), frame());
    let score = |c: &Core, f: &Frame| {
        c.evaluate(f, &f.units[0])
            .into_iter()
            .filter(|a| a.family == "rocket")
            .map(|a| a.score)
            .max()
            .unwrap()
    };
    let base = score(&c, &f);
    f.hazards.push(Hazard {
        id: 5,
        pos: f.units[0].pos,
        radius: 20000,
        until: 9999,
        severity: 300,
    });
    assert_eq!(score(&c, &f), base);
    f.hazards.clear();
    c.armed_until = f.tick + 100;
    assert_eq!(score(&c, &f), base + 60);
}
#[test]
fn r143_the_guide_is_held_every_tick_without_flipping_back_to_the_engine() {
    // a retreat with the enemy on her heels: decided again and again, guided on every tick in between
    let (mut c, mut f) = (Core::default(), frame());
    c.stock = [0; 3];
    c.pack_cooldown = 1_000_000;
    c.cooldowns[0] = 1_000_000;
    c.cooldowns[1] = 1_000_000;
    f.automatic = true;
    f.unlocked[2] = false;
    f.units[0].hp = 300;
    let (mut guided, mut flips, mut last) = (0, 0, false);
    for _ in 0..600 {
        f.units[1].pos = f.units[0].pos.add(Point::new(40000, 0));
        let out = tick(&mut c, &mut f, None);
        let to = out.iter().find_map(|v| match v {
            Cmd::Guide { to } => Some(*to),
            _ => None,
        });
        if let Some(to) = to {
            guided += 1;
            // walk toward it as the engine would
            let me = f.units[0].pos;
            f.units[0].pos = me.toward(to, distance(me, to).min(300));
        }
        if to.is_some() != last && guided > 1 {
            flips += 1;
        }
        last = to.is_some();
    }
    assert!(guided > 550, "{guided}");
    assert_eq!(flips, 0);
    // on arrival the guide holds the point she reached (no hand-back to the engine)
    let (mut c, mut f) = (Core::default(), frame());
    f.automatic = true;
    f.units.truncate(1);
    let here = f.units[0].pos;
    c.guide = Some(Guide {
        family: "retreat".into(),
        target: None,
        to: here,
    });
    c.reaction_until = 1_000_000;
    c.last_think = 1_000_000;
    let out = advance(&mut c, &mut f, 30);
    assert_eq!(
        out.iter()
            .filter(|v| **v == Cmd::Guide { to: here })
            .count(),
        30
    );
}
#[test]
fn r143_rotate_follows_team_guidance_and_hands_back_when_it_stops() {
    let (mut c, mut f) = (Core::default(), frame());
    f.automatic = true;
    f.units.truncate(1);
    f.assignment = Some(Point::new(330000, 300000));
    let out = advance(&mut c, &mut f, 40);
    assert!(out.contains(&Cmd::Guide {
        to: Point::new(330000, 300000)
    }));
    // the guidance gate refuses (or hands the job back): the engine walks her again at once
    f.assignment = None;
    let out = tick(&mut c, &mut f, None);
    assert!(!out.iter().any(|v| matches!(v, Cmd::Guide { .. })));
    assert!(c.guide.is_none());
}
#[test]
fn r143_takeoff_gate_blocks_launches_away_from_the_engines_destination() {
    let (c, mut f) = (ready(), frame());
    f.units[1].pos = Point::new(420000, 300000);
    let legal = |f: &Frame, family: &str| {
        c.evaluate(f, &f.units[0])
            .iter()
            .any(|a| a.family == family && a.legal)
    };
    assert!(legal(&f, "single_launch"));
    f.walk = Some(Point::new(100000, 300000));
    assert!(!legal(&f, "single_launch"));
    assert!(!legal(&f, "double_launch"));
    assert!(legal(&f, "escape"));
    f.walk = Some(Point::new(500000, 320000));
    assert!(legal(&f, "single_launch"));
}
#[test]
fn r143_entries_that_spend_every_pack_alone_cost_an_escape() {
    let (mut c, f) = (ready(), frame());
    let cost = |c: &Core| {
        c.evaluate(&f, &f.units[0])
            .into_iter()
            .find(|a| a.family == "double_launch")
            .unwrap()
            .immediate
    };
    let both = cost(&c);
    c.stock[0] = 2;
    let spare = cost(&c);
    assert_eq!(spare - both, 20 + (100 - 50) / 5);
}
#[test]
fn r143_damage_table() {
    // AP 100: Paint Shells 85 + 75%, clusters 20 + 20%, Boom Bot 70 + 70%, Showstopper 240 + 150%, Blast Pack
    // 30 + 25% (armed after 30 ticks), self-damage 15%
    assert_eq!(GRENADE_DAMAGE + 100 * GRENADE_AP / 100, 160);
    assert_eq!(CLUSTER_DAMAGE + 100 * CLUSTER_AP_NUM / CLUSTER_AP_DEN, 40);
    assert_eq!(BOT_DAMAGE + 100 * BOT_AP / 100, 140);
    assert_eq!(ROCKET_DAMAGE + 100 * ROCKET_AP / 100, 390);
    assert_eq!(PACK_DAMAGE + 100 * PACK_AP / 100, 55);
    assert_eq!(
        (PACK_ARM, SELF_DAMAGE_PERCENT, ROCKET_COOLDOWN),
        (30, 15, 3000)
    );
    let (mut c, mut f) = (ready(), frame());
    tick(
        &mut c,
        &mut f,
        Some(Action::Grenade {
            aim: Point::new(350000, 300000),
        }),
    );
    let out = advance(&mut c, &mut f, 130);
    let hits: Vec<_> = out
        .iter()
        .filter_map(|v| match v {
            Cmd::Damage { target: 2, raw, .. } => Some(*raw),
            _ => None,
        })
        .collect();
    assert_eq!(hits[0], 160);
}
