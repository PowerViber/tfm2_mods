use crate::contender_core::*;
fn fixture() -> Frame {
    Frame {
        version: 1,
        tick: 0,
        owner: 1,
        units: vec![
            Unit {
                id: 1,
                team: 0,
                pos: Point {
                    x: 100000,
                    y: 100000,
                },
                hp: 1000,
                max_hp: 1000,
                attack: 100,
                radius: 10000,
                champion: true,
                visible: true,
                targetable: true,
                cc_immune: false,
                farmable: false,
                velocity: Point::default(),
            },
            Unit {
                id: 2,
                team: 1,
                pos: Point {
                    x: 122000,
                    y: 100000,
                },
                hp: 1000,
                max_hp: 1000,
                attack: 100,
                radius: 10000,
                champion: true,
                visible: true,
                targetable: true,
                cc_immune: false,
                farmable: false,
                velocity: Point::default(),
            },
        ],
        blocks: vec![],
        bounds: map(),
        profile: [50; 13],
        manual: true,
        action: None,
        skill1: true,
        skill2: true,
        ultimate: true,
        interrupted: false,
        rooted: false,
        receipts: vec![],
        basics: vec![],
        arena_started: None,
        arena_ended: false,
        basics_validated_by_host: false,
        dealt_at: None,
        duel_unavailable: vec![],
    }
}
#[test]
fn basics_obey_walls_and_first_collision() {
    let mut f = fixture();
    let mut c = Core::default();
    f.basics = vec![Basic {
        serial: 1,
        target: 2,
    }];
    f.blocks.push(Rect {
        min: Point {
            x: 110000,
            y: 80000,
        },
        max: Point {
            x: 112000,
            y: 120000,
        },
    });
    assert_eq!(damage(&c.step(&f)), 0);
}
#[test]
fn already_committed_participant_rejects_without_cost() {
    let mut f = fixture();
    f.duel_unavailable = vec![2];
    f.action = Some(action(Technique::MainEvent));
    let mut c = Core::default();
    let out = c.step(&f);
    assert!(out.is_empty());
    assert_eq!(c.stamina, 100000);
    assert_eq!(c.cooldowns[3], 0);
}
#[test]
fn distant_visible_fight_does_not_suppress_local_role_eligible_farming() {
    let mut c = Core::default();
    let mut f = fixture();
    f.manual = false;
    f.units[1].pos.x = 500000;
    let mut minion = f.units[1].clone();
    minion.id = 3;
    minion.champion = false;
    minion.farmable = true;
    minion.pos.x = 122000;
    f.units.push(minion);
    for tick in 0..25 {
        f.tick = tick;
        c.step(&f);
    }
    assert_eq!(c.situation, Some(3));
    assert!(c.cast.is_some());
}
#[test]
fn control_speed_reacts_once_and_search_is_bounded() {
    let mut first = vec![];
    for speed in [20, 80] {
        let mut c = Core::default();
        let mut f = fixture();
        f.manual = false;
        f.profile[3] = speed;
        let mut at = None;
        for tick in 0..50 {
            f.tick = tick;
            let effects = c.step(&f);
            assert!(c.decision.candidates <= 64 && c.decision.selected.len() <= 3);
            if effects.iter().any(|e| matches!(e, Effect::Pose { .. })) {
                at = Some(tick);
                break;
            }
        }
        first.push(at.unwrap());
    }
    assert!(first[1] < first[0]);
}
fn action(t: Technique) -> Action {
    Action {
        technique: t,
        target: Some(2),
        aim: Point {
            x: 122000,
            y: 100000,
        },
    }
}
fn cast(t: Technique) -> (Core, Frame, Vec<Effect>) {
    let mut c = Core::default();
    let mut f = fixture();
    f.action = Some(action(t));
    let e = c.step(&f);
    f.action = None;
    (c, f, e)
}
fn damage(e: &[Effect]) -> i64 {
    e.iter()
        .filter_map(|e| {
            if let Effect::Damage { amount, .. } = e {
                Some(amount)
            } else {
                None
            }
        })
        .sum()
}
#[test]
fn combination_two_single_hits() {
    let (mut c, mut f, _) = cast(Technique::Combination);
    f.tick = 9;
    assert_eq!(damage(&c.step(&f)), 75);
    f.tick = 10;
    assert_eq!(damage(&c.step(&f)), 0);
    f.tick = 27;
    assert_eq!(damage(&c.step(&f)), 105);
    f.tick = 28;
    assert_eq!(damage(&c.step(&f)), 0);
}
#[test]
fn body_shot_value_and_slow() {
    let (mut c, mut f, _) = cast(Technique::BodyShot);
    f.tick = 21;
    let e = c.step(&f);
    assert_eq!(damage(&e), 160);
    assert!(e.iter().any(|e| matches!(
        e,
        Effect::AttackSlow {
            percent: 15,
            ticks: 90,
            ..
        }
    )));
}
#[test]
fn uppercut_value_and_control() {
    let (mut c, mut f, _) = cast(Technique::Uppercut);
    f.tick = 18;
    let e = c.step(&f);
    assert_eq!(damage(&e), 140);
    assert!(e
        .iter()
        .any(|e| matches!(e, Effect::Airborne { ticks: 24, .. })));
}
#[test]
fn uppercut_respects_immunity() {
    let (mut c, mut f, _) = cast(Technique::Uppercut);
    f.tick = 18;
    f.units[1].cc_immune = true;
    let e = c.step(&f);
    assert_eq!(damage(&e), 140);
    assert!(!e.iter().any(|e| matches!(e, Effect::Airborne { .. })));
}
#[test]
fn stamina_spent_once_and_preparation_pauses_recovery() {
    let (mut c, mut f, _) = cast(Technique::Combination);
    assert_eq!(c.stamina, 78000);
    f.tick = 1;
    c.step(&f);
    assert_eq!(c.stamina, 78000);
}
#[test]
fn starvation_rejects_without_cooldown() {
    let mut c = Core {
        stamina: 1000,
        ..Core::default()
    };
    let mut f = fixture();
    f.action = Some(action(Technique::Uppercut));
    c.step(&f);
    assert_eq!(c.cooldowns, [0; 4]);
    assert!(c.rejection.contains("stamina"));
}
#[test]
fn shared_combo_cooldown() {
    let (mut c, mut f, _) = cast(Technique::BodyShot);
    f.tick = 22;
    f.action = Some(action(Technique::Combination));
    c.step(&f);
    assert!(c.rejection.contains("cooldown"));
}
#[test]
fn shield_and_guard_applied_once() {
    let (mut c, mut f, e) = cast(Technique::Guard);
    assert!(e.contains(&Effect::Shield {
        amount: 135,
        ticks: 75
    }));
    assert!(e.contains(&Effect::Defence {
        percent: 40,
        move_penalty: 50,
        ticks: 27
    }));
    f.tick = 1;
    assert!(!c
        .step(&f)
        .iter()
        .any(|e| matches!(e, Effect::Shield { .. })));
}
#[test]
fn shield_absorbed_receipt_readies_counter() {
    let (mut c, mut f, _) = cast(Technique::Guard);
    f.tick = 1;
    f.receipts = vec![Receipt {
        serial: 1,
        source: 2,
        champion: true,
        positive_damage: true,
    }];
    c.step(&f);
    assert_eq!(c.counter, Some((2, 121)));
}
#[test]
fn counter_uses_same_primary_packet_once() {
    let (mut c, mut f, _) = cast(Technique::Guard);
    f.tick = 1;
    f.receipts = vec![Receipt {
        serial: 1,
        source: 2,
        champion: true,
        positive_damage: true,
    }];
    c.step(&f);
    f.receipts.clear();
    f.tick = 28;
    f.basics = vec![Basic {
        serial: 1,
        target: 2,
    }];
    let e = c.step(&f);
    assert_eq!(damage(&e), 160);
    assert_eq!(
        e.iter()
            .filter(|e| matches!(e, Effect::Damage { .. }))
            .count(),
        1
    );
    f.tick = 29;
    assert_eq!(damage(&c.step(&f)), 0);
}
#[test]
fn wrong_attacker_does_not_consume_counter() {
    let (mut c, mut f, _) = cast(Technique::Guard);
    f.tick = 1;
    f.receipts = vec![Receipt {
        serial: 1,
        source: 9,
        champion: true,
        positive_damage: true,
    }];
    c.step(&f);
    f.receipts.clear();
    f.tick = 28;
    f.basics = vec![Basic {
        serial: 1,
        target: 2,
    }];
    assert_eq!(damage(&c.step(&f)), 100);
    assert_eq!(c.counter, Some((9, 121)));
}
#[test]
fn minion_and_zero_damage_do_not_ready_counter() {
    for (champion, positive_damage) in [(false, true), (true, false)] {
        let (mut c, mut f, _) = cast(Technique::Guard);
        f.tick = 1;
        f.receipts = vec![Receipt {
            serial: 1,
            source: 2,
            champion,
            positive_damage,
        }];
        c.step(&f);
        assert_eq!(c.counter, None);
    }
}
#[test]
fn slip_fixed_duration_and_path() {
    let (mut c, mut f, e) = cast(Technique::Slip);
    assert!(e.contains(&Effect::Dodge { ticks: 12 }));
    f.tick = 6;
    assert!(c.step(&f).contains(&Effect::Move {
        pos: Point {
            x: 108000,
            y: 100000
        }
    }));
    f.tick = 12;
    assert!(c.step(&f).contains(&Effect::Move {
        pos: Point {
            x: 116000,
            y: 100000
        }
    }));
    assert!(!c.busy());
}
#[test]
fn roots_reject_slip_without_spending() {
    let mut c = Core::default();
    let mut f = fixture();
    f.rooted = true;
    f.action = Some(action(Technique::Slip));
    c.step(&f);
    assert_eq!(c.stamina, 100000);
    assert!(!c.busy());
}
#[test]
fn walls_reject_slip() {
    let mut c = Core::default();
    let mut f = fixture();
    f.blocks.push(Rect {
        min: Point {
            x: 110000,
            y: 50000,
        },
        max: Point {
            x: 112000,
            y: 150000,
        },
    });
    f.action = Some(action(Technique::Slip));
    c.step(&f);
    assert_eq!(c.stamina, 100000);
}
#[test]
fn walls_block_punches() {
    let (mut c, mut f, _) = cast(Technique::Combination);
    f.blocks.push(Rect {
        min: Point {
            x: 110000,
            y: 50000,
        },
        max: Point {
            x: 112000,
            y: 150000,
        },
    });
    f.tick = 9;
    assert_eq!(damage(&c.step(&f)), 0);
}
#[test]
fn only_first_enemy_is_hit() {
    let (mut c, mut f, _) = cast(Technique::Combination);
    let mut u = f.units[1].clone();
    u.id = 3;
    u.pos.x = 125000;
    f.units.push(u);
    f.tick = 9;
    let e = c.step(&f);
    assert_eq!(
        e.iter()
            .filter(|e| matches!(e, Effect::Damage { .. }))
            .count(),
        1
    );
    assert!(e
        .iter()
        .any(|e| matches!(e, Effect::Damage { target: 2, .. })));
}
#[test]
fn concealment_and_invulnerability_reject_hits() {
    for visible in [true, false] {
        let (mut c, mut f, _) = cast(Technique::Combination);
        f.units[1].visible = visible;
        f.units[1].targetable = false;
        f.tick = 9;
        assert_eq!(damage(&c.step(&f)), 0);
    }
}
#[test]
fn interruption_wastes_commitment_without_a_hit() {
    let (mut c, mut f, _) = cast(Technique::Combination);
    f.tick = 8;
    f.interrupted = true;
    assert_eq!(damage(&c.step(&f)), 0);
    assert!(!c.busy());
    assert_eq!(c.stamina, 78000);
    f.tick = 9;
    f.interrupted = false;
    assert_eq!(damage(&c.step(&f)), 0);
}
#[test]
fn ultimate_has_warning_and_requires_acknowledged_transfer() {
    let (mut c, mut f, e) = cast(Technique::MainEvent);
    assert!(e
        .iter()
        .any(|e| matches!(e, Effect::Warn { until: 39, .. })));
    f.tick = 39;
    let e = c.step(&f);
    assert!(e.iter().any(|e| matches!(
        e,
        Effect::Transfer {
            target: 2,
            duration: 480,
            ..
        }
    )));
    assert_eq!(c.arena_target, None);
    f.tick = 40;
    f.arena_started = Some(2);
    c.step(&f);
    assert_eq!(c.arena_target, Some(2));
}
#[test]
fn ultimate_lock_and_target_loss() {
    let mut c = Core::default();
    let mut f = fixture();
    f.ultimate = false;
    f.action = Some(action(Technique::MainEvent));
    c.step(&f);
    assert_eq!(c.cooldowns, [0; 4]);
    let (mut c, mut f, _) = cast(Technique::MainEvent);
    f.tick = 39;
    f.units[1].visible = false;
    assert!(!c
        .step(&f)
        .iter()
        .any(|e| matches!(e, Effect::Transfer { .. })));
    assert!(c.cooldowns[3] > 0);
}
#[test]
fn cooldown_acceleration_preserves_ultimate() {
    let mut c = Core {
        cooldowns: [720, 1080, 1440, 13500],
        last_tick: Some(0),
        ..Core::default()
    };
    let mut f = fixture();
    f.tick = 60;
    f.arena_started = Some(2);
    c.step(&f);
    assert_eq!(c.cooldowns, [540, 900, 1260, 13320]);
    f.tick = 120;
    f.arena_started = None;
    f.arena_ended = true;
    c.step(&f);
    assert_eq!(c.cooldowns, [300, 660, 1020, 13140]);
}
#[test]
fn cooldown_expiry_splits_elapsed_time_without_losing_fractional_progress() {
    let mut c = Core {
        cooldowns: [1000; 4],
        last_tick: Some(0),
        arena_target: Some(2),
        arena_until: 10,
        ..Core::default()
    };
    let mut f = fixture();
    f.tick = 20;
    c.step(&f);
    assert_eq!(c.cooldowns, [930, 930, 930, 940]);
}
#[test]
fn death_and_respawn_preserve_cooldowns() {
    let (mut c, mut f, _) = cast(Technique::Uppercut);
    f.tick = 1;
    f.units[0].hp = 0;
    c.step(&f);
    assert!(!c.busy());
    c.respawn(61);
    assert_eq!(c.stamina, 100000);
    assert_eq!(c.cooldowns[2], 1440 - 3 * 61);
}
#[test]
fn identical_manual_inputs_ignore_all_athlete_profiles() {
    let mut outputs = vec![];
    for p in [[20; 13], [80; 13]] {
        let mut c = Core::default();
        let mut f = fixture();
        f.profile = p;
        f.action = Some(action(Technique::Combination));
        let mut e = c.step(&f);
        f.action = None;
        for t in 1..=40 {
            f.tick = t;
            e.extend(c.step(&f));
        }
        outputs.push(serde_json::to_string(&e).unwrap());
    }
    assert_eq!(outputs[0], outputs[1]);
}
#[test]
fn independent_owners_and_clones() {
    let (mut a, mut f, _) = cast(Technique::Guard);
    let b = Core::default();
    let mut clone = a.clone();
    f.tick = 1;
    f.receipts = vec![Receipt {
        serial: 1,
        source: 2,
        champion: true,
        positive_damage: true,
    }];
    a.step(&f);
    assert_eq!(b.counter, None);
    assert_eq!(clone.counter, None);
    clone.step(&f);
    assert_eq!(clone.counter, a.counter);
}
#[test]
fn development_and_holdout_vectors() {
    for seed in 0..36 {
        let mut f = fixture();
        f.units[1].pos.y += ((seed * 7919) % 18000) - 9000;
        f.units[1].pos.x += ((seed * 3571) % 16000) - 8000;
        let t = [
            Technique::Combination,
            Technique::BodyShot,
            Technique::Uppercut,
            Technique::Guard,
            Technique::Slip,
            Technique::MainEvent,
        ][seed as usize % 6];
        f.action = Some(action(t));
        let mut a = Core::default();
        let mut b = a.clone();
        for tick in 0..100 {
            f.tick = tick;
            let x = a.step(&f);
            let y = b.step(&f);
            assert_eq!(x, y, "seed {seed}");
            f.action = None;
            assert!((0..=100000).contains(&a.stamina));
        }
        assert_eq!(a.cooldowns, b.cooldowns);
    }
}

// Round 146: approach → pressure → punish → defend, across athlete profiles.
fn duel(distance: i64, profile: i64) -> (Core, Frame) {
    let mut f = fixture();
    f.manual = false;
    f.profile = [profile; 13];
    f.units[1].pos.x = f.units[0].pos.x + distance;
    let mut c = Core::default();
    f.arena_started = Some(2);
    c.step(&f);
    f.arena_started = None;
    (c, f)
}
fn decide(c: &mut Core, f: &mut Frame, until: u64) -> Vec<Effect> {
    let mut all = vec![];
    for t in f.tick + 1..=until {
        f.tick = t;
        all.extend(c.step(f));
        if c.busy() {
            break;
        }
    }
    all
}
#[test]
fn arena_started_tells_the_core_a_duel_began() {
    let (c, _) = duel(48000, 50);
    assert_eq!(c.arena_target, Some(2));
    assert!(c.arena_until > 0);
}
#[test]
fn far_apart_in_a_duel_he_approaches_with_slip() {
    // Round 148: edge 50,000 is beyond a dash punch's 37,000 (Combination reach 23,000 + dash 14,000)
    for profile in [0, 50, 100] {
        let (mut c, mut f) = duel(70000, profile);
        decide(&mut c, &mut f, 90);
        assert_eq!(c.decision.intent, "approach", "profile {profile}");
        let cast = c.cast.as_ref().expect("he commits to closing the gap");
        assert_eq!(cast.technique, Technique::Slip, "profile {profile}");
        assert!(cast.aim.x > f.units[0].pos.x, "toward the opponent (profile {profile})");
    }
}
#[test]
fn in_reach_in_a_duel_he_pressures_and_does_not_guard_without_a_threat() {
    for profile in [0, 50, 100] {
        let (mut c, mut f) = duel(20000, profile);
        decide(&mut c, &mut f, 90);
        assert_eq!(c.decision.intent, "pressure", "profile {profile}");
        let t = c.cast.as_ref().expect("he punches").technique;
        assert!(matches!(t, Technique::Combination | Technique::BodyShot | Technique::Uppercut), "{t:?} at profile {profile}");
    }
}
#[test]
fn guard_needs_an_observed_hit() {
    let (mut c, mut f) = duel(20000, 100);
    c.stamina = 30_000; // enough for Guard alone
    c.cooldowns = [9999, 0, 9999, 0];
    decide(&mut c, &mut f, 60);
    assert!(c.cast.is_none(), "no threat, no guard");
    f.receipts = vec![Receipt { serial: 1, source: 2, champion: true, positive_damage: true }];
    f.units[0].hp = 400;
    let mut c2 = c.clone();
    decide(&mut c2, &mut f, 120);
    assert_eq!(c2.decision.intent, "defend");
    assert_eq!(c2.cast.as_ref().map(|x| x.technique), Some(Technique::Guard));
}
#[test]
fn a_landed_uppercut_opens_a_punish_window() {
    let (mut c, mut f) = duel(15000, 50);
    f.manual = true;
    f.action = Some(action(Technique::Uppercut));
    f.tick += 1;
    c.step(&f);
    f.action = None;
    for t in f.tick + 1..=f.tick + 30 {
        f.tick = t;
        c.step(&f);
    }
    assert!(c.punish_until > f.tick, "the hit opened a window");
}
#[test]
fn manual_mechanics_are_unchanged_by_the_duel_planner() {
    // A queued manual action costs and does exactly the same in and out of a duel.
    for started in [None, Some(2)] {
        let mut f = fixture();
        f.arena_started = started;
        let mut c = Core::default();
        f.action = Some(action(Technique::Combination));
        let out = c.step(&f);
        assert_eq!(c.stamina, 100_000 - number("combination", "stamina") * 1000);
        assert_eq!(c.cooldowns[0], number("combination", "cooldownTicks") * 3);
        assert!(out.iter().any(|e| matches!(e, Effect::Pose { name } if name == "combination")));
    }
}
#[test]
fn version_two_frames_play_like_version_one() {
    let mut a = fixture();
    a.manual = false;
    let mut b = a.clone();
    b.version = 2;
    let (mut ca, mut cb) = (Core::default(), Core::default());
    for t in 0..120 {
        a.tick = t;
        b.tick = t;
        assert_eq!(ca.step(&a), cb.step(&b), "tick {t}");
    }
}

// ---- Round 148: host-validated basics, edge reach, line of sight, dash punches, no CC for basic poses
#[test]
fn host_validated_basic_on_a_big_camp_behind_a_wall_cell_always_lands() {
    let mut f = fixture();
    f.basics_validated_by_host = true;
    // a jungle monster, radius 18,000, at the game's edge range (23,000) -> centre 51,000 away
    f.units[1].radius = 18000;
    f.units[1].champion = false;
    f.units[1].team = 2;
    f.units[1].pos.x = f.units[0].pos.x + 10000 + 23000 + 18000;
    // the camp's own wall cell, and the coarse cell under him
    f.blocks = vec![
        Rect { min: Point { x: 140000, y: 90000 }, max: Point { x: 172000, y: 122000 } },
        Rect { min: Point { x: 90000, y: 90000 }, max: Point { x: 112000, y: 112000 } },
    ];
    let mut c = Core::default();
    let mut landed = 0;
    for n in 1..=5u64 {
        f.tick = n * 60;
        f.basics = vec![Basic { serial: n * 60, target: 2 }];
        let out = c.step(&f);
        landed += out.iter().filter(|e| matches!(e, Effect::Damage { target: 2, basic: true, .. })).count();
    }
    assert_eq!(landed, 5, "every accepted basic deals damage");
}
#[test]
fn the_lab_still_checks_its_own_basics() {
    let mut f = fixture();
    f.units[1].pos.x = f.units[0].pos.x + 60000;
    f.tick = 60;
    f.basics = vec![Basic { serial: 60, target: 2 }];
    let out = Core::default().step(&f);
    assert!(!out.iter().any(|e| matches!(e, Effect::Damage { basic: true, .. })), "60,000 away: no hit without a host");
}
#[test]
fn a_wall_strictly_between_still_blocks_a_hit() {
    let mut f = fixture();
    f.units[1].pos.x = f.units[0].pos.x + 60000;
    f.blocks = vec![Rect { min: Point { x: 125000, y: 80000 }, max: Point { x: 135000, y: 120000 } }];
    assert!(!clear_line(&f, f.units[0].pos, f.units[1].pos));
    f.blocks = vec![Rect { min: Point { x: 150000, y: 80000 }, max: Point { x: 182000, y: 120000 } }];
    assert!(clear_line(&f, f.units[0].pos, f.units[1].pos), "the target's own cell does not block");
}
#[test]
fn at_edge_range_against_a_big_target_he_punches_not_slips() {
    for profile in [0, 50, 100] {
        let (mut c, mut f) = duel(10000 + 20000 + 15000, profile);
        f.units[1].radius = 15000;
        decide(&mut c, &mut f, 90);
        let t = c.cast.as_ref().expect("he commits").technique;
        assert!(matches!(t, Technique::Combination | Technique::BodyShot | Technique::Uppercut), "{t:?} at profile {profile}");
    }
}
#[test]
fn dash_punches_carry_him_in_and_stop_at_the_target_edge() {
    for (t, travel) in [(Technique::Combination, 14000), (Technique::BodyShot, 10000), (Technique::Uppercut, 12000)] {
        assert_eq!(dash_travel(t), travel);
        let mut f = fixture();
        f.units[1].pos.x = f.units[0].pos.x + 50000; // edge 30,000
        f.tick = 1;
        f.action = Some(Action { technique: t, target: Some(2), aim: f.units[1].pos });
        let mut c = Core::default();
        c.step(&f);
        let to = c.cast.as_ref().unwrap().dash_to.expect("a dash");
        assert_eq!(to.x - f.units[0].pos.x, travel, "{t:?} dashes its full travel");
        // walk the cast: Moves arrive before the first hit and end at the dash point
        f.action = None;
        let mut last = None;
        for tick in 2..=40u64 {
            f.tick = tick;
            for e in c.step(&f) {
                if let Effect::Move { pos } = e { last = Some(pos); f.units[0].pos = pos; }
            }
        }
        assert_eq!(last, Some(to), "{t:?}");
        // close in: the dash stops short of the target's edge
        let mut f = fixture();
        f.units[1].pos.x = f.units[0].pos.x + 26000; // edge 6,000
        f.tick = 1;
        f.action = Some(Action { technique: t, target: Some(2), aim: f.units[1].pos });
        let mut c = Core::default();
        c.step(&f);
        let to = c.cast.as_ref().unwrap().dash_to.expect("a short dash");
        assert!(to.x - f.units[0].pos.x <= 6000 - 1500, "{t:?} never passes the target's edge");
    }
    assert_eq!(dash_travel(Technique::Slip), 0);
    assert_eq!(dash_travel(Technique::Guard), 0);
}
#[test]
fn a_rooted_boxer_does_not_dash() {
    let mut f = fixture();
    f.units[1].pos.x = f.units[0].pos.x + 50000;
    f.rooted = true;
    f.tick = 1;
    f.action = Some(Action { technique: Technique::Combination, target: Some(2), aim: f.units[1].pos });
    let mut c = Core::default();
    c.step(&f);
    assert!(c.cast.as_ref().unwrap().dash_to.is_none());
}
#[test]
fn a_basic_pose_has_no_cast_so_the_bridge_sends_no_cc() {
    let mut f = fixture();
    f.basics_validated_by_host = true;
    f.tick = 60;
    f.basics = vec![Basic { serial: 60, target: 2 }];
    let mut c = Core::default();
    let out = c.step(&f);
    assert!(out.iter().any(|e| matches!(e, Effect::Pose { .. })), "the jab/cross pose is still reported");
    assert!(c.cast.is_none(), "with no cast running the bridge skips the Animation CC (contender.rs Pose)");
}
#[test]
fn a_farmable_camp_in_dash_reach_gets_punched() {
    let mut f = fixture();
    f.manual = false;
    f.units[1].champion = false;
    f.units[1].farmable = true;
    f.units[1].team = 2;
    f.units[1].radius = 18000;
    f.units[1].pos.x = f.units[0].pos.x + 10000 + 18000 + 30000; // edge 30,000: only a dash punch reaches
    let mut c = Core::default();
    decide(&mut c, &mut f, 90);
    let t = c.cast.as_ref().expect("he works the camp").technique;
    assert!(matches!(t, Technique::Combination | Technique::BodyShot | Technique::Uppercut), "{t:?}");
}
#[test]
fn the_bridge_never_sends_a_cc_for_a_pose_without_a_cast() {
    let source = include_str!("contender.rs");
    let pose = &source[source.find("Effect::Pose { name } =>").unwrap()..];
    let pose = &pose[..pose.find("sim.apply_cc").unwrap()];
    let compact: String = pose.split_whitespace().collect();
    assert!(compact.contains("else{continue;}"), "basic-attack poses skip the Animation CC");
}
