use super::gojo_core::*;
fn units() -> Vec<Fighter> {
    vec![
        Fighter {
            id: 1,
            team: 0,
            x: 100000,
            y: 100000,
            hp: 850,
            max_hp: 850,
            ad: 70,
            ap: 60,
            level: 3,
            champion: true,
            visible: true,
            targetable: true,
            immune: false,
            cc_immune: false,
            farmable: false,
            tower: false,
        },
        Fighter {
            id: 2,
            team: 1,
            x: 120000,
            y: 100000,
            hp: 9000,
            max_hp: 9000,
            ad: 50,
            ap: 50,
            level: 3,
            champion: true,
            visible: true,
            targetable: true,
            immune: false,
            cc_immune: false,
            farmable: false,
            tower: false,
        },
    ]
}
fn input<'a>(t: usize, u: &'a [Fighter], a: Option<Action>, b: &'a [(u64, usize)]) -> Input<'a> {
    Input {
        tick: t,
        me: &u[0],
        all: u,
        unlocked: [true; 3],
        controlled: false,
        bound: false,
        automatic: false,
        haste: 0,
        ult_haste: 0,
        action: a,
        basics: b,
        champion_damage: false,
        shield: None,
        wall: &|_, _| false,
        barrier: &|_, _| false,
        sight: &|_, _| true,
    }
}
fn blue(c: &mut Core, u: &[Fighter]) {
    for t in 0..=36 {
        c.step(&input(
            t,
            u,
            if t == 0 { Some(Action::Blue(2)) } else { None },
            &[],
        ));
    }
}
fn dmg(a: &[Cmd]) -> Vec<(i64, i64)> {
    a.iter()
        .filter_map(|v| {
            if let Cmd::Damage { ad, ap, .. } = v {
                Some((*ad, *ap))
            } else {
                None
            }
        })
        .collect()
}
#[test]
fn earns_one_black_flash_then_amplifies_both_damage_components() {
    let u = units();
    let mut c = Core::default();
    blue(&mut c, &u);
    assert_eq!(c.punches, 2);
    assert!(c.combo.is_some());
    let a = c.step(&input(37, &u, None, &[(1, 2)]));
    assert_eq!(dmg(&a), vec![(70, 48)]);
    let b = c.step(&input(92, &u, None, &[(2, 2)]));
    assert_eq!(dmg(&b), vec![(70, 99)]);
    assert_eq!(c.flashes, 1);
    assert_eq!(c.amp_until, 572);
    let a = c.step(&input(147, &u, None, &[(3, 2)]));
    assert_eq!(dmg(&a), vec![(84, 25)]);
    assert_eq!(c.flashes, 1);
}
#[test]
fn duplicate_impacts_wrong_targets_and_expiry_never_flash() {
    let mut u = units();
    u.push(Fighter {
        id: 3,
        ..u[1].clone()
    });
    let mut c = Core::default();
    blue(&mut c, &u);
    c.step(&input(37, &u, None, &[(1, 3), (1, 2)]));
    assert_eq!(c.combo.as_ref().unwrap().punches, 0);
    c.step(&input(276, &u, None, &[(2, 2)]));
    assert!(c.combo.is_none());
    assert_eq!(c.flashes, 0);
}
#[test]
fn immune_farming_hidden_and_cancelled_impacts_do_not_earn() {
    for mode in 0..4 {
        let mut u = units();
        if mode == 0 {
            u[1].immune = true
        }
        if mode == 1 {
            u[1].champion = false;
            u[1].farmable = true
        }
        let mut c = Core::default();
        blue(&mut c, &u);
        if mode == 2 {
            u[1].visible = false
        }
        for t in [37, 92] {
            let mut i = input(t, &u, None, &[]);
            let b = [(t as u64, 2)];
            i.basics = &b;
            i.controlled = mode == 3;
            c.step(&i);
        }
        assert_eq!(c.flashes, 0);
    }
}
#[test]
fn refresh_is_not_a_stack_and_first_flash_does_not_amplify_itself() {
    let u = units();
    let mut c = Core::default();
    blue(&mut c, &u);
    c.amp_until = 1000;
    c.step(&input(37, &u, None, &[(1, 2)]));
    let out = c.step(&input(92, &u, None, &[(2, 2)]));
    assert_eq!(dmg(&out), vec![(84, 118)]);
    assert_eq!(c.amp_until, 572);
    assert_eq!(c.flashes, 1);
}
#[test]
fn ce_sustains_routine_casts_but_rejects_unaffordable_commitments() {
    let u = units();
    let mut c = Core::default();
    for t in 0..3600 {
        let a = if t % 600 == 0 {
            Some(Action::Blue(2))
        } else if t % 600 == 60 {
            Some(Action::Red(2))
        } else {
            None
        };
        let mut i = input(t, &u, a, &[]);
        i.champion_damage = true;
        c.step(&i);
    }
    assert!(c.ce > 50000, "{}", c.ce);
    let before = c.ready;
    c.ce = 0;
    c.step(&input(3601, &u, Some(Action::Purple(2)), &[]));
    assert_eq!(c.ready, before);
}
#[test]
fn infinity_break_pause_and_reform_never_restore_paused_health() {
    let u = units();
    let mut c = Core::default();
    c.step(&input(0, &u, None, &[]));
    assert_eq!(c.shield, 230);
    let mut i = input(1, &u, None, &[]);
    i.shield = Some(120);
    c.step(&i);
    c.burnout = 100;
    c.step(&input(2, &u, None, &[]));
    assert!(!c.infinity);
    assert_eq!(c.shield, 120);
    c.step(&input(100, &u, None, &[]));
    assert!(c.infinity);
    assert_eq!(c.shield, 120);
    let mut i = input(101, &u, None, &[]);
    i.shield = Some(0);
    c.step(&i);
    assert!(!c.infinity);
    assert_eq!(c.infinity_ready, 1001);
    c.step(&input(1001, &u, None, &[]));
    assert_eq!(c.shield, 230);
    assert!(c.infinity);
}
#[test]
fn strain_overload_blocks_limitless_but_allows_rct_then_finishes_at_sixty() {
    let mut u = units();
    u[0].hp = 300;
    let mut c = Core::default();
    c.step(&input(0, &u, None, &[]));
    c.strain = 99000;
    c.step(&input(1, &u, Some(Action::Blue(2)), &[]));
    assert!(c.pending.is_none());
    assert_eq!(c.overload, 241);
    c.step(&input(2, &u, Some(Action::Heal), &[]));
    assert!(matches!(
        c.pending.as_ref().map(|c| c.action),
        Some(Action::Heal)
    ));
    c.step(&input(241, &u, None, &[]));
    assert_eq!(c.strain, 60000);
}
#[test]
fn interruption_spends_resources_and_releases_no_projectile() {
    let u = units();
    let mut c = Core::default();
    c.step(&input(0, &u, Some(Action::MaxBlue(2)), &[]));
    let mut i = input(20, &u, None, &[]);
    i.controlled = true;
    c.step(&i);
    c.step(&input(60, &u, None, &[]));
    assert!(c.shots.is_empty());
    assert_eq!(c.ready[0], 600);
    assert_eq!(c.casts[1], 1);
}
#[test]
fn purple_requires_both_slots_and_ult_spends_once_and_hits_once() {
    let mut u = units();
    u[1].x = 200000;
    let mut c = Core::default();
    let mut i = input(0, &u, Some(Action::Purple(2)), &[]);
    i.unlocked[2] = false;
    c.step(&i);
    assert_eq!(c.casts[4], 0);
    let out = c.step(&input(1, &u, Some(Action::Purple(2)), &[]));
    assert!(out.iter().any(|v| matches!(v, Cmd::Lock(34))));
    assert_eq!(c.ce, 65984);
    let mut hits = 0;
    for t in 2..200 {
        hits += dmg(&c.step(&input(t, &u, None, &[]))).len();
    }
    assert_eq!(hits, 1);
    assert_eq!(c.ready[0], 601);
    assert_eq!(c.ready[2], 541);
}
#[test]
fn spatial_step_revalidates_walls_binding_range_and_map() {
    let u = units();
    for mode in 0..4 {
        let mut c = Core::default();
        let mut i = input(
            0,
            &u,
            Some(Action::Blink(
                if mode == 0 { 999999 } else { 125000 },
                100000,
            )),
            &[],
        );
        i.bound = mode == 1;
        if mode == 2 {
            i.wall = &|_, _| true
        }
        c.step(&i);
        if mode < 3 {
            assert!(c.pending.is_none())
        } else {
            let mut i = input(12, &u, None, &[]);
            i.wall = &|x, _| x > 110000;
            let out = c.step(&i);
            assert!(!out.iter().any(|v| matches!(v, Cmd::Move { target: 1, .. })));
        }
    }
}
#[test]
fn domain_chooses_one_guest_once_and_seals_every_other_champion() {
    let mut u = units();
    u.push(Fighter {
        id: 4,
        team: 0,
        x: 104000,
        ..u[1].clone()
    });
    u.push(Fighter {
        id: 3,
        team: 0,
        x: 104000,
        ..u[1].clone()
    });
    let mut c = Core::default();
    c.step(&input(0, &u, Some(Action::Domain), &[]));
    assert_eq!(c.domain.as_ref().unwrap().guest, Some(3));
    let out = c.step(&input(6, &u, None, &[]));
    assert!(out.iter().any(|v| matches!(v, Cmd::Seal { target: 4, .. })));
    assert!(!out.iter().any(|v| matches!(v, Cmd::Seal { target: 3, .. })));
    u[3].hp = 0;
    c.step(&input(12, &u, None, &[]));
    assert_eq!(c.domain.as_ref().unwrap().guest, Some(3));
    c.step(&input(390, &u, None, &[]));
    assert!(c.domain.is_none());
    assert_eq!(c.burnout, 570);
}
#[test]
fn death_cleans_all_owned_state_preserves_cooldowns_and_clones_are_independent() {
    let mut u = units();
    let mut a = Core::default();
    a.step(&input(0, &u, Some(Action::Domain), &[]));
    let mut b = a.clone();
    u[0].hp = 0;
    let out = a.step(&input(1, &u, None, &[]));
    assert!(out.contains(&Cmd::DomainEnd));
    assert!(a.domain.is_none());
    assert!(b.domain.is_some());
    u[0].hp = 850;
    a.step(&input(10, &u, None, &[]));
    assert_eq!(a.ready[7], 3600);
    assert_eq!(a.ce, 100000);
    assert_eq!(a.strain, 0);
    b.ce = 0;
    assert_ne!(a.ce, b.ce);
}
#[test]
fn ai_penalizes_allied_domain_risk_and_never_uses_major_spells_for_farming() {
    let mut u = units();
    u.push(Fighter {
        id: 3,
        team: 1,
        ..u[1].clone()
    });
    for id in 4..7 {
        u.push(Fighter {
            id,
            team: 0,
            ..u[1].clone()
        })
    }
    let mut c = Core::default();
    let mut i = input(0, &u, None, &[]);
    i.automatic = true;
    c.step(&i);
    assert!(c.domain.is_none());
    let mut u = units();
    u[1].champion = false;
    u[1].farmable = true;
    let mut c = Core::default();
    for t in 0..900 {
        let mut i = input(t, &u, None, &[]);
        i.automatic = true;
        c.step(&i);
    }
    assert_eq!(c.casts[4] + c.casts[7], 0);
}
#[test]
fn manual_results_have_no_rank_or_athlete_dependency() {
    let u = units();
    let mut a = Core::default();
    let mut b = Core::default();
    for t in 0..500 {
        let i = input(
            t,
            &u,
            if t == 0 { Some(Action::Blue(2)) } else { None },
            &[],
        );
        assert_eq!(a.step(&i), b.step(&i));
        assert_eq!(a, b);
    }
}

#[test]
fn ordinary_displacement_respects_cc_immunity_while_domain_overload_still_seals() {
    let mut u = units();
    u[1].cc_immune = true;
    let mut c = Core::default();
    blue(&mut c, &u);
    assert!(c.requested > 0);
    assert!(c.motions.is_empty());
    c.step(&input(37, &u, Some(Action::Domain), &[]));
    let out = c.step(&input(67, &u, None, &[]));
    // Seals renew on six-tick boundaries, independently of ordinary displacement.
    let out2 = c.step(&input(72, &u, None, &[]));
    assert!(out
        .iter()
        .chain(out2.iter())
        .any(|v| matches!(v, Cmd::Seal { target: 2, .. })));
}

#[test]
fn a_released_area_attack_can_hit_fog_without_earning_a_hidden_combination() {
    let mut u = units();
    let mut c = Core::default();
    let mut damage = 0;
    for t in 0..=36 {
        if t == 25 {
            u[1].visible = false;
        }
        damage += dmg(&c.step(&input(
            t,
            &u,
            if t == 0 { Some(Action::Blue(2)) } else { None },
            &[],
        )))
        .iter()
        .map(|v| v.0 + v.1)
        .sum::<i64>();
    }
    assert!(damage > 0);
    assert!(c.combo.is_none());
    assert_eq!(c.punches, 0);
}

#[test]
fn residual_shield_can_break_while_paused_without_delaying_recovery_again() {
    let u = units();
    let mut c = Core::default();
    c.step(&input(0, &u, None, &[]));
    c.burnout = 100;
    let mut i = input(1, &u, None, &[]);
    i.shield = Some(90);
    c.step(&i);
    assert!(!c.infinity);
    assert_eq!(c.shield, 90);
    i.tick = 2;
    i.shield = Some(0);
    c.step(&i);
    assert_eq!(c.infinity_ready, 902);
    for t in 3..100 {
        i.tick = t;
        c.step(&i);
    }
    assert_eq!(c.infinity_ready, 902);
    assert_eq!(c.shield, 0);
}
#[test]
fn fractional_haste_is_not_lost_between_single_tick_callbacks() {
    let u = units();
    let mut c = Core::default();
    c.step(&input(0, &u, Some(Action::Blue(2)), &[]));
    for t in 1..=100 {
        let mut i = input(t, &u, None, &[]);
        i.haste = 35;
        i.ult_haste = 25;
        c.step(&i);
    }
    assert_eq!(c.ready[0], 565);
}
#[test]
fn amplification_uses_the_exact_impact_tick_and_never_healing() {
    let mut u = units();
    u[0].hp = 300;
    let mut c = Core::default();
    c.step(&input(0, &u, None, &[]));
    c.amp_until = 56;
    assert_eq!(
        dmg(&c.step(&input(55, &u, None, &[(1, 2)]))),
        vec![(84, 25)]
    );
    assert_eq!(
        dmg(&c.step(&input(56, &u, None, &[(2, 2)]))),
        vec![(70, 21)]
    );
    c.step(&input(57, &u, Some(Action::Heal), &[]));
    let mut healing = 0;
    for t in 58..155 {
        healing += c
            .step(&input(t, &u, None, &[]))
            .iter()
            .filter_map(|v| if let Cmd::Heal(n) = v { Some(*n) } else { None })
            .sum::<i64>();
    }
    assert_eq!(healing, 110);
}
#[test]
fn maximum_output_is_exactly_one_point_three_five_of_each_blue_payload() {
    let u = units();
    let mut ordinary = Core::default();
    let mut maximum = Core::default();
    let mut a = 0;
    let mut b = 0;
    for t in 0..=70 {
        a += dmg(&ordinary.step(&input(
            t,
            &u,
            if t == 0 { Some(Action::Blue(2)) } else { None },
            &[],
        )))
        .iter()
        .map(|v| v.1)
        .sum::<i64>();
        b += dmg(&maximum.step(&input(
            t,
            &u,
            if t == 0 {
                Some(Action::MaxBlue(2))
            } else {
                None
            },
            &[],
        )))
        .iter()
        .map(|v| v.1)
        .sum::<i64>();
    }
    assert_eq!(a, 212);
    assert_eq!(b, a * 135 / 100);
}
#[test]
fn starvation_does_not_create_enhanced_or_black_flash_damage() {
    let u = units();
    let mut c = Core::default();
    blue(&mut c, &u);
    c.ce = 0;
    c.burnout = 1000;
    let out = c.step(&input(37, &u, None, &[(1, 2)]));
    assert_eq!(dmg(&out), vec![(70, 21)]);
    assert_eq!(c.combo.as_ref().unwrap().punches, 0);
}
#[test]
fn unavailable_destination_or_new_barrier_cancels_spatial_step() {
    let u = units();
    let mut c = Core::default();
    let mut i = input(0, &u, Some(Action::Blink(125000, 100000)), &[]);
    i.sight = &|_, _| false;
    c.step(&i);
    assert_eq!(c.casts[6], 0);
    c.step(&input(1, &u, Some(Action::Blink(125000, 100000)), &[]));
    let mut i = input(13, &u, None, &[]);
    i.barrier = &|x, _| x > 110000;
    assert!(!c
        .step(&i)
        .iter()
        .any(|v| matches!(v, Cmd::Move { target: 1, .. })));
}
#[test]
fn domains_keep_their_centre_guest_and_ten_second_bonus() {
    let mut u = units();
    u.push(Fighter {
        id: 3,
        team: 0,
        x: 103000,
        ..u[1].clone()
    });
    let mut c = Core::default();
    c.step(&input(0, &u, Some(Action::Domain), &[]));
    u[0].x = 101000;
    assert!(c.step(&input(1, &u, None, &[])).contains(&Cmd::Move {
        target: 1,
        x: 100000,
        y: 100000
    }));
    let out = c.step(&input(150, &u, None, &[]));
    assert!(out.contains(&Cmd::Empower {
        target: 3,
        ticks: 600,
        slow: true
    }));
    assert_eq!(c.power_until, 750);
    c.step(&input(390, &u, None, &[]));
    assert_eq!(c.power_until, 750);
    assert!(!c.infinity);
}
#[test]
fn displaced_targets_move_over_time_and_new_walls_stop_the_motion() {
    let mut u = units();
    let mut c = Core::default();
    blue(&mut c, &u);
    c.step(&input(37, &u, None, &[(1, 2)]));
    assert_eq!(c.motions.len(), 1);
    let out = c.step(&input(38, &u, None, &[]));
    let moved = out
        .iter()
        .find_map(|v| {
            if let Cmd::Move { target: 2, x, y } = v {
                Some((*x, *y))
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(moved, (119000, 100000));
    u[1].x = moved.0;
    let mut i = input(39, &u, None, &[]);
    i.wall = &|x, _| x < 119000;
    c.step(&i);
    assert!(c.motions.is_empty());
}
#[test]
fn invalid_and_repeated_ticks_do_not_spend_or_duplicate_effects() {
    let u = units();
    let mut c = Core::default();
    c.step(&input(10, &u, Some(Action::Blue(2)), &[]));
    let before = c.clone();
    assert!(c.step(&input(10, &u, Some(Action::Domain), &[])).is_empty());
    assert_eq!(c, before);
    assert!(c.step(&input(9, &u, None, &[])).is_empty());
    assert_eq!(c, before);
}
#[test]
fn duplicate_red_components_do_not_reset_an_unfinished_combo() {
    let u = units();
    let mut c = Core::default();
    for t in 0..=30 {
        c.step(&input(
            t,
            &u,
            if t == 0 { Some(Action::Red(2)) } else { None },
            &[],
        ));
    }
    let combo = c.combo.clone().unwrap();
    assert_eq!(combo.punches, 0);
    assert_eq!(c.punches, 0);
    c.step(&input(31, &u, None, &[(1, 2)]));
    assert_eq!(c.combo.as_ref().unwrap().punches, 1);
    assert_eq!(c.combo.as_ref().unwrap().end, combo.end);
}
#[test]
fn transport_rejects_duplicate_entities_and_malformed_geometry() {
    let mut a = vec![1, 0, 1, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, -1, 1];
    a.extend([1, 0, 100000, 100000, 850, 850, 70, 60, 3, 1, 1, 1, 0, 0, 0]);
    a.push(0);
    assert!(super::gojo_protocol::Frame::decode(&a).is_some());
    let mut bad = a.clone();
    bad[0] = 2;
    assert!(super::gojo_protocol::Frame::decode(&bad).is_none());
    a.extend([0, 0, 1]);
    assert!(super::gojo_protocol::Frame::decode(&a).is_none());
}

#[test]
fn spending_the_last_ce_pauses_reductions_until_five_ce_is_available() {
    let u = units();
    let mut c = Core::default();
    c.step(&input(0, &u, None, &[]));
    c.ce = 6000;
    // No elapsed tick at a fresh spawn is normally spent here; balance the one
    // tick's net regeneration/upkeep so the accepted Blue spends exactly six CE.
    c.ce -= 34;
    c.step(&input(1, &u, Some(Action::Blue(2)), &[]));
    assert_eq!(c.ce, 0);
    assert!(!c.infinity);
    assert_eq!(c.shield, 230);
    for t in 2..100 {
        c.step(&input(t, &u, None, &[]));
    }
    assert!(!c.infinity);
    c.step(&input(102, &u, None, &[]));
    assert!(c.infinity);
}

#[test]
fn automatic_offence_keeps_escape_energy_but_manual_inputs_keep_the_full_catalogue() {
    let u = units();
    let mut c = Core::default();
    c.step(&input(0, &u, None, &[]));
    c.ce = 9000;
    let mut i = input(1, &u, None, &[]);
    i.automatic = true;
    c.step(&i);
    assert!(c.pending.is_none());
    c.step(&input(2, &u, Some(Action::Blue(2)), &[]));
    assert!(c.pending.is_some());
}
