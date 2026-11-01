use crate::rally_core::*;
fn unit(id: usize, team: usize, x: i64, y: i64) -> Unit {
    Unit {
        id,
        team,
        pos: Point { x, y },
        velocity: Point::default(),
        hp: 5000,
        max_hp: 5000,
        attack: 104,
        radius: 10000,
        champion: true,
        damageable: true,
        visible: true,
        targetable: true,
        farmable: false,
        controlled: false,
        cc_immune: false,
        control_eligible: true,
    }
}
fn frame() -> Frame {
    Frame {
        units: vec![
            unit(1, 0, 150000, 300000),
            unit(2, 1, 220000, 300000),
            unit(3, 0, 175000, 320000),
        ],
        steer: Point { x: 1, y: 0 },
        incoming_verified: true,
        support_verified: true,
        ..Frame::default()
    }
}
fn action(f: &mut Frame, c: &mut Core, t: u64, kind: ActionKind) -> Vec<Effect> {
    f.tick = t;
    f.action = Some(Action {
        kind,
        target: Some(if kind == ActionKind::AllySet { 3 } else { 2 }),
        aim: Point {
            x: 950000,
            y: 300000,
        },
    });
    let out = c.step(f);
    moves(f, &out);
    out
}
fn tick(f: &mut Frame, c: &mut Core, t: u64) -> Vec<Effect> {
    f.tick = t;
    f.action = None;
    let out = c.step(f);
    moves(f, &out);
    out
}
fn moves(f: &mut Frame, out: &[Effect]) {
    for e in out {
        if let Effect::Move { pos } = e {
            f.units[0].pos = *pos;
        }
    }
}
fn arm(f: &mut Frame, c: &mut Core, t: u64, n: u8) {
    f.burst_bars = n;
    action(f, c, t, ActionKind::ArmBurst);
}
fn shot_fixture(h: i64, form: AttackTier, set: u8, spike: u8) -> (Frame, Core, Vec<Effect>) {
    let mut f = frame();
    f.steer = Point::default();
    let mut c = Core::default();
    tick(&mut f, &mut c, 0);
    let p = f.units[0].pos;
    c.thunder_earned = form >= AttackTier::Thunder;
    c.transformed = form >= AttackTier::BlackThunder;
    c.white_storm = form == AttackTier::WhiteStorm;
    c.phase = Phase::Air {
        start: 1,
        origin: p,
        aim: p,
        max_height: h,
        perfect: false,
        enhanced: set > 0,
        set_bars: set,
        spike_bars: spike,
        white_before: c.white_storm,
        attempted: false,
        intended: 24,
        hold_start: None,
        hold_ticks: 0,
        serve: false,
        white: false,
    };
    c.ball = Ball::Toss {
        start: 1,
        origin: p,
        to: p,
        peak: h + 100,
    };
    let out = action(&mut f, &mut c, 25, ActionKind::Spike);
    assert!(matches!(c.ball, Ball::Flying { .. }), "{}", c.rejection);
    (f, c, out)
}
fn impossible() -> (Frame, Core, u64) {
    let mut f = frame();
    let mut c = Core {
        pace: 100,
        white_storm: true,
        transformed: true,
        thunder_earned: true,
        ..Core::default()
    };
    arm(&mut f, &mut c, 0, 5);
    action(&mut f, &mut c, 1, ActionKind::SelfSet);
    for t in 2..=13 {
        tick(&mut f, &mut c, t);
    }
    action(&mut f, &mut c, 14, ActionKind::Ultimate);
    for t in 15..62 {
        tick(&mut f, &mut c, t);
    }
    action(&mut f, &mut c, 62, ActionKind::Takeoff);
    for t in 63..86 {
        tick(&mut f, &mut c, t);
    }
    action(&mut f, &mut c, 86, ActionKind::Spike);
    assert!(
        matches!(c.phase, Phase::Impossible { height: 18020, .. }),
        "{}",
        c.rejection
    );
    (f, c, 86)
}
fn receipt(serial: u64, force: Option<i64>) -> Receipt {
    Receipt {
        serial,
        source: 2,
        delivered: 5,
        reduction: 95,
        incoming_force: force,
        champion: true,
        verified: true,
        reflected: false,
    }
}
#[test]
fn no_timed_refill_and_five_bars_replace_old_resources() {
    let (mut f, mut c) = (frame(), Core::default());
    tick(&mut f, &mut c, 0);
    tick(&mut f, &mut c, 3600);
    assert_eq!(c.pace, 0);
    let state = serde_json::to_value(&c).unwrap();
    for key in ["flow", "charges", "charge_progress", "black_until"] {
        assert!(state.get(key).is_none());
    }
    action(&mut f, &mut c, 3601, ActionKind::BlackThunder);
    assert!(c.rejection.contains("Retired"));
    assert_eq!(c.cds, [0; 4]);
}
#[test]
fn every_bar_cost_is_reserved_on_the_actual_commitment() {
    for n in [1u8, 3, 5] {
        let (mut f, mut c) = (
            frame(),
            Core {
                pace: 100,
                ..Core::default()
            },
        );
        arm(&mut f, &mut c, 0, n);
        assert_eq!(c.pace, 100);
        action(&mut f, &mut c, 1, ActionKind::Approach);
        assert_eq!(c.pace, 100);
        for t in 2..25 {
            tick(&mut f, &mut c, t);
        }
        action(&mut f, &mut c, 25, ActionKind::Takeoff);
        assert_eq!(c.pace, 100 - 20 * n as i64);
        assert!(
            matches!(c.phase,Phase::Air{max_height,spike_bars,..}if max_height==4800+100*n as i64&&spike_bars==n)
        );
    }
}
#[test]
fn partial_bars_invalid_arming_and_cancel_do_not_spend() {
    let (mut f, mut c) = (
        frame(),
        Core {
            pace: 55,
            ..Core::default()
        },
    );
    arm(&mut f, &mut c, 0, 3);
    assert_eq!(c.pace, 55);
    assert_eq!(c.armed_bars, 0);
    arm(&mut f, &mut c, 1, 2);
    action(&mut f, &mut c, 2, ActionKind::CancelBurst);
    assert_eq!(c.pace, 55);
    arm(&mut f, &mut c, 3, 1);
    action(&mut f, &mut c, 4, ActionKind::Receive);
    assert_eq!(c.pace, 35);
}
#[test]
fn armed_expiry_leaves_ordinary_skills_and_money_untouched() {
    let (mut f, mut c) = (
        frame(),
        Core {
            pace: 100,
            ..Core::default()
        },
    );
    arm(&mut f, &mut c, 0, 5);
    tick(&mut f, &mut c, 480);
    action(&mut f, &mut c, 481, ActionKind::SelfSet);
    assert_eq!(c.pace, 100);
    assert!(matches!(c.phase, Phase::Prepare { burst: 0, .. }));
}
#[test]
fn burst_set_scales_height_and_power_for_every_spend() {
    for n in [1u8, 3, 5] {
        let (mut f, mut c) = (
            frame(),
            Core {
                pace: 100,
                ..Core::default()
            },
        );
        arm(&mut f, &mut c, 0, n);
        action(&mut f, &mut c, 1, ActionKind::SelfSet);
        tick(&mut f, &mut c, 13);
        assert_eq!(c.set_bars, n);
        assert_eq!(c.exhaustion_until, 0);
        action(&mut f, &mut c, 14, ActionKind::Approach);
        for t in 15..38 {
            tick(&mut f, &mut c, t);
        }
        action(&mut f, &mut c, 38, ActionKind::Takeoff);
        assert!(matches!(c.phase,Phase::Air{max_height,..}if max_height==5100+600*n as i64));
        assert_eq!(c.power(true, n, 0), 115 + 15 * n as i64);
    }
}
#[test]
fn set_and_spike_bursts_cannot_share_a_play_or_spend_twice() {
    let (mut f, mut c) = (
        frame(),
        Core {
            pace: 100,
            ..Core::default()
        },
    );
    arm(&mut f, &mut c, 0, 1);
    action(&mut f, &mut c, 1, ActionKind::SelfSet);
    tick(&mut f, &mut c, 13);
    let id = c.burst_play.as_ref().unwrap().id;
    assert_eq!(c.pace, 80);
    arm(&mut f, &mut c, 14, 3);
    assert_eq!(c.armed_bars, 0);
    assert_eq!(c.burst_play.as_ref().unwrap().id, id);
    action(&mut f, &mut c, 15, ActionKind::Approach);
    for t in 16..39 {
        tick(&mut f, &mut c, t);
    }
    action(&mut f, &mut c, 39, ActionKind::Takeoff);
    assert_eq!(c.pace, 80);
    assert!(matches!(
        c.phase,
        Phase::Air {
            set_bars: 1,
            spike_bars: 0,
            ..
        }
    ));
    for t in 40..63 {
        tick(&mut f, &mut c, t);
    }
    action(&mut f, &mut c, 63, ActionKind::Spike);
    assert!(c.burst_play.is_none());
    assert_eq!(c.exhaustion_until, 153);
}
#[test]
fn failed_preparation_keeps_spent_pace_and_starts_exhaustion() {
    let (mut f, mut c) = (
        frame(),
        Core {
            pace: 100,
            ..Core::default()
        },
    );
    arm(&mut f, &mut c, 0, 3);
    action(&mut f, &mut c, 1, ActionKind::SelfSet);
    f.interrupted = true;
    tick(&mut f, &mut c, 2);
    assert_eq!(c.pace, 40);
    assert_eq!(c.phase, Phase::Idle);
    assert_eq!(c.set_ready, 0);
    assert!(c.exhaustion_until > 2);
}
#[test]
fn rejected_targets_learning_and_roots_spend_no_resources() {
    for kind in [
        ActionKind::AllySet,
        ActionKind::Takeoff,
        ActionKind::SelfSet,
    ] {
        let (mut f, mut c) = (
            frame(),
            Core {
                pace: 100,
                ..Core::default()
            },
        );
        arm(&mut f, &mut c, 0, 5);
        if kind == ActionKind::AllySet {
            f.units[2].visible = false;
        } else if kind == ActionKind::SelfSet {
            f.skill2 = false;
        } else {
            f.rooted = true;
        }
        action(&mut f, &mut c, 1, kind);
        assert_eq!(c.pace, 100);
        assert_eq!(c.armed_bars, 5);
    }
}
#[test]
fn receive_burst_uses_exact_capture_perfect_and_exhaustion_boundaries() {
    for n in [1u8, 3, 5] {
        let (mut f, mut c) = (
            frame(),
            Core {
                pace: 100,
                ..Core::default()
            },
        );
        arm(&mut f, &mut c, 0, n);
        let out = action(&mut f, &mut c, 1, ActionKind::Receive);
        let end = 1 + 60 + 12 * n as u64;
        let perfect = 1 + ((90 + 18 * n as u64 + 9) / 10);
        assert_eq!(c.receive_until, end);
        assert_eq!(c.receive_perfect_until, perfect);
        assert!(out.iter().any(|e|matches!(e,Effect::Shield{amount,ticks}if *amount==131*(100+15*n as i64)/100&&*ticks==end-1)));
        assert_eq!(c.reduction(perfect - 1), 95);
        assert!(c.reduction(perfect) < 95);
        tick(&mut f, &mut c, end - 1);
        assert_eq!(c.exhaustion_until, 0);
        tick(&mut f, &mut c, end);
        assert_eq!(c.exhaustion_until, end + 90);
    }
}
#[test]
fn ordinary_receive_stores_only_verified_pre_reduction_champion_force_in_its_window() {
    let (mut f, mut c) = (frame(), Core::default());
    action(&mut f, &mut c, 0, ActionKind::Receive);
    f.receipts = vec![receipt(1, Some(200)), receipt(2, None)];
    tick(&mut f, &mut c, 1);
    assert_eq!(c.caught, 200);
    assert_eq!(c.bank_base, 194);
    assert_eq!(c.pace, 15);
    f.receipts = vec![receipt(3, Some(99999))];
    tick(&mut f, &mut c, 59);
    assert_eq!(c.caught, 582);
    assert_eq!(c.bank_percent(), 300);
    f.receipts = vec![receipt(4, Some(100))];
    tick(&mut f, &mut c, 60);
    assert_eq!(c.caught, 582);
    assert_eq!(c.pace, 15);
}
#[test]
fn unverified_self_reflection_ally_and_farm_receipts_never_create_force() {
    for scenario in 0..6 {
        let (mut f, mut c) = (frame(), Core::default());
        action(&mut f, &mut c, 0, ActionKind::Receive);
        let mut r = receipt(1, Some(100));
        match scenario {
            0 => f.incoming_verified = false,
            1 => r.verified = false,
            2 => r.source = 1,
            3 => r.reflected = true,
            4 => r.champion = false,
            _ => r.source = 3,
        }
        f.receipts = vec![r];
        tick(&mut f, &mut c, 1);
        assert_eq!(c.caught, 0);
        assert_eq!(c.pace, 0);
    }
}
#[test]
fn receipt_ids_are_deduplicated_and_same_frame_order_does_not_drop_force() {
    let (mut f, mut c) = (frame(), Core::default());
    action(&mut f, &mut c, 0, ActionKind::Receive);
    f.receipts = vec![
        receipt(2, Some(100)),
        receipt(1, Some(100)),
        receipt(2, Some(100)),
    ];
    tick(&mut f, &mut c, 1);
    tick(&mut f, &mut c, 2);
    assert_eq!(c.caught, 200);
    assert_eq!(c.pace, 15);
}
#[test]
fn receive_force_survives_role_changes_but_expires_without_refresh() {
    let (mut f, mut c) = (frame(), Core::default());
    action(&mut f, &mut c, 0, ActionKind::Receive);
    f.receipts = vec![receipt(1, Some(400))];
    tick(&mut f, &mut c, 1);
    f.receipts.clear();
    action(&mut f, &mut c, 2, ActionKind::SelfSet);
    tick(&mut f, &mut c, 14);
    action(&mut f, &mut c, 15, ActionKind::Approach);
    assert_eq!(c.caught, 400);
    tick(&mut f, &mut c, 480);
    assert_eq!(c.caught, 400);
    tick(&mut f, &mut c, 481);
    assert_eq!(c.caught, 0);
}
#[test]
fn exhaustion_blocks_gain_and_new_bursts_but_keeps_ordinary_actions() {
    let (mut f, mut c) = (
        frame(),
        Core {
            pace: 100,
            exhaustion_until: 100,
            ..Core::default()
        },
    );
    arm(&mut f, &mut c, 0, 1);
    assert_eq!(c.armed_bars, 0);
    action(&mut f, &mut c, 1, ActionKind::Receive);
    f.receipts = vec![receipt(1, Some(200))];
    tick(&mut f, &mut c, 2);
    assert_eq!(c.caught, 200);
    assert_eq!(c.pace, 100);
    assert!(c
        .buffs
        .values()
        .any(|b| b.channel == "exhaustion" && b.speed == -20));
    f.receipts.clear();
    tick(&mut f, &mut c, 100);
    assert!(!c.buffs.values().any(|b| b.channel == "exhaustion"));
}
#[test]
fn burst_origins_cannot_pay_their_own_pace_after_exhaustion_ends() {
    let (mut f, mut c) = (frame(), Core::default());
    tick(&mut f, &mut c, 0);
    c.pending_hits.insert(1, (true, true, true, false));
    f.hits = vec![Hit {
        serial: 1,
        target: 2,
        delivered: true,
        champion: true,
    }];
    tick(&mut f, &mut c, 500);
    assert_eq!(c.pace, 0);
    c.pending_hits.insert(2, (true, false, true, false));
    f.hits[0].serial = 2;
    tick(&mut f, &mut c, 501);
    assert_eq!(c.pace, 15);
    tick(&mut f, &mut c, 502);
    assert_eq!(c.pace, 15);
}
#[test]
fn teammate_set_burst_scales_without_taking_combat_ball_and_lingers() {
    let (mut f, mut c) = (
        frame(),
        Core {
            pace: 100,
            ..Core::default()
        },
    );
    arm(&mut f, &mut c, 0, 3);
    action(&mut f, &mut c, 1, ActionKind::AllySet);
    tick(&mut f, &mut c, 13);
    assert_eq!(c.ball, Ball::Held);
    assert!(c
        .buffs
        .values()
        .any(|b| b.channel == "support" && b.attack == 21 && b.speed == 11));
    tick(&mut f, &mut c, 193);
    assert!(c.support.is_none());
    assert!(c.buffs.values().any(|b| b.channel == "support"));
    tick(&mut f, &mut c, 313);
    assert!(!c.buffs.values().any(|b| b.channel == "support"));
}
#[test]
fn support_flow_requires_verified_contribution_and_excludes_burst_set() {
    for burst in [false, true] {
        let (mut f, mut c) = (
            frame(),
            Core {
                pace: if burst { 20 } else { 0 },
                ..Core::default()
            },
        );
        if burst {
            arm(&mut f, &mut c, 0, 1);
        } else {
            tick(&mut f, &mut c, 0);
        }
        action(&mut f, &mut c, 1, ActionKind::AllySet);
        tick(&mut f, &mut c, 13);
        f.support_receipts = vec![SupportReceipt {
            serial: 1,
            ally: 3,
            champion: true,
            positive: true,
            verified: true,
        }];
        tick(&mut f, &mut c, 14);
        assert_eq!(c.pace, if burst { 0 } else { 15 });
    }
}
#[test]
fn height_bands_permanent_power_and_damage_order_are_exact() {
    for (h, tier, mult, power) in [
        (3999, AttackTier::Normal, 100, 100),
        (4000, AttackTier::Thunder, 180, 110),
        (5999, AttackTier::Thunder, 230, 110),
        (6000, AttackTier::BlackThunder, 500, 125),
        (9999, AttackTier::BlackThunder, 500, 125),
        (10000, AttackTier::WhiteStorm, 600, 140),
    ] {
        let (_, c, _) = shot_fixture(h, AttackTier::Normal, 0, 0);
        assert_eq!(c.permanent_tier(), tier);
        let power = power + (h - 2000).max(0) * 15 / 1000;
        assert_eq!(c.last_power, power);
        assert!(matches!(c.ball,Ball::Flying{amount,..}if amount==194*power*mult/10000));
        assert_eq!(attack_tier(height_label(h)), tier);
    }
}
#[test]
fn white_body_low_spike_keeps_white_power_without_white_height_damage() {
    let (_, c, _) = shot_fixture(3500, AttackTier::WhiteStorm, 0, 0);
    assert!(c.white_storm);
    assert_eq!(c.last_power, 162);
    assert_eq!(c.last_spike_tier, AttackTier::Normal);
    assert!(matches!(
        c.ball,
        Ball::Flying {
            white: false,
            amount: 314,
            ..
        }
    ));
}
#[test]
fn spike_power_velocity_received_force_bonus_and_600_kmh_cap_are_exact() {
    let c = Core {
        white_storm: true,
        ..Core::default()
    };
    assert_eq!(c.velocity(230, 300), 64400);
    assert_eq!(Core::kmh_tenths(64400), 4528);
    assert_eq!(c.velocity(99999, 300), 5120000 / 60);
    assert!(Core::kmh_tenths(c.velocity(99999, 300)) <= 6000);
}
#[test]
fn receive_bank_is_not_multiplied_by_height_power_or_splash() {
    let (mut f, mut c, _) = shot_fixture(10000, AttackTier::WhiteStorm, 0, 0);
    if let Ball::Flying { caught, .. } = &mut c.ball {
        *caught = 582;
    }
    f.units.push(unit(4, 1, 225000, 300000));
    let mut got = vec![];
    for t in 26..70 {
        for e in tick(&mut f, &mut c, t) {
            if let Effect::Damage { amount, .. } = e {
                got.push(amount);
            }
        }
    }
    assert_eq!(got.len(), 2);
    assert_eq!(got.iter().sum::<i64>(), 3026 * 2 + 582);
}
#[test]
fn basic_volley_consumes_saved_force_once_and_uses_ordinary_primary_packet() {
    let (mut f, mut c) = (frame(), Core::default());
    tick(&mut f, &mut c, 0);
    c.caught = 582;
    c.bank_base = 194;
    c.caught_until = 500;
    f.basics = vec![Basic {
        serial: 1,
        target: 2,
    }];
    tick(&mut f, &mut c, 1);
    f.basics.clear();
    assert_eq!(c.caught, 0);
    let mut damage = vec![];
    for t in 2..30 {
        for e in tick(&mut f, &mut c, t) {
            if let Effect::Damage { amount, basic, .. } = e {
                assert!(basic);
                damage.push(amount);
            }
        }
    }
    assert_eq!(damage, vec![660]);
}
#[test]
fn overpressure_changes_warned_radius_without_an_extra_damage_packet() {
    let (_, c, out) = shot_fixture(10000, AttackTier::WhiteStorm, 5, 0);
    assert!(matches!(
        c.ball,
        Ball::Flying {
            overpressure: true,
            radius: 78000,
            ..
        }
    ));
    assert!(out
        .iter()
        .any(|e| matches!(e, Effect::Warning { radius: 78000, .. })));
}
#[test]
fn every_released_spike_returns_after_miss_or_wall_and_basics_still_ground() {
    for wall in [false, true] {
        let (mut f, mut c, _) = shot_fixture(4800, AttackTier::Normal, 0, 0);
        f.units.retain(|u| u.id == 1);
        if wall {
            f.blocks.push(Rect {
                min: Point {
                    x: 180000,
                    y: 200000,
                },
                max: Point {
                    x: 185000,
                    y: 400000,
                },
            });
        }
        for t in 26..240 {
            let out = tick(&mut f, &mut c, t);
            assert!(!out.iter().any(|e| matches!(e, Effect::Damage { .. })));
        }
        assert_eq!(c.ball, Ball::Held);
    }
    let (mut f, mut c) = (frame(), Core::default());
    tick(&mut f, &mut c, 0);
    f.units[1].pos.x = 300000;
    f.basics = vec![Basic {
        serial: 1,
        target: 2,
    }];
    tick(&mut f, &mut c, 1);
    f.basics.clear();
    f.units[1].pos.y = 800000;
    for t in 2..100 {
        tick(&mut f, &mut c, t);
    }
    assert!(matches!(c.ball, Ball::Grounded { .. }));
}
#[test]
fn perfect_takeoff_repeats_and_requires_fresh_cycle_displacement() {
    for stop in [false, true] {
        let (mut f, mut c) = (frame(), Core::default());
        action(&mut f, &mut c, 0, ActionKind::Approach);
        for t in 1..36 {
            tick(&mut f, &mut c, t);
        }
        if stop {
            f.steer = Point::default();
        }
        for t in 36..60 {
            tick(&mut f, &mut c, t);
        }
        action(&mut f, &mut c, 60, ActionKind::Takeoff);
        assert!(matches!(c.phase,Phase::Air{perfect,..}if perfect!=stop));
    }
}
#[test]
fn impossible_power_grows_from_470_to_670_then_releases_at_300_ticks() {
    let (mut f, mut c, t) = impossible();
    assert_eq!(c.pace, 0);
    assert_eq!(c.last_power, 470);
    assert_eq!(c.reduction(t), 50);
    let mut previous = 470;
    for age in 1..300 {
        tick(&mut f, &mut c, t + age);
        assert_eq!(c.height(t + age), 18020);
        assert_eq!(c.last_power, 470 + 200 * age as i64 / 300);
        assert!(c.last_power >= previous);
        previous = c.last_power;
    }
    tick(&mut f, &mut c, t + 300);
    assert!(matches!(
        c.ball,
        Ball::Flying {
            impossible: true,
            amount: 10398,
            power: 670,
            ..
        }
    ));
    assert_eq!(c.reduction(t + 300), 0);
    assert_eq!(c.exhaustion_until, t + 390);
    assert_eq!(Core::kmh_tenths(c.last_speed), 3297);
}
#[test]
fn serve_impossible_aims_for_four_seconds_then_locks_for_60_ticks() {
    let (mut f, mut c, t) = impossible();
    f.aim = Some(Point {
        x: 100000,
        y: 800000,
    });
    for at in t + 1..t + 240 {
        tick(&mut f, &mut c, at);
    }
    f.aim = Some(Point {
        x: 900000,
        y: 100000,
    });
    let out = tick(&mut f, &mut c, t + 240);
    assert!(out.iter().any(|e|matches!(e,Effect::LineWarning{locked:true,public:true,to,..}if to.x==100000&&to.y==800000)));
    for at in t + 241..=t + 300 {
        tick(&mut f, &mut c, at);
    }
    assert!(matches!(c.ball,Ball::Flying{to,..}if to.x==100000&&to.y==800000));
}
#[test]
fn provisional_warning_updates_and_final_lock_use_final_speed_and_public_audience() {
    let (mut f, mut c, t) = impossible();
    f.aim = Some(Point {
        x: 700000,
        y: 300000,
    });
    let out = tick(&mut f, &mut c, t + 1);
    assert!(out.iter().any(|e| matches!(
        e,
        Effect::Warning {
            locked: false,
            public: true,
            radius: 144000,
            ..
        }
    )));
    for at in t + 2..t + 240 {
        assert!(!tick(&mut f, &mut c, at)
            .iter()
            .any(|e| matches!(e, Effect::LineWarning { .. })));
    }
    let out = tick(&mut f, &mut c, t + 240);
    let final_speed = if let Phase::Impossible { ref shot, .. } = c.phase {
        Core::charge_shot(shot, 300).speed
    } else {
        panic!()
    };
    assert!(out.iter().any(|e|matches!(e,Effect::LineWarning{locked:true,until,..}if *until==t+300+(550000+final_speed-1)as u64/final_speed as u64)));
    for at in t + 241..t + 300 {
        assert!(!tick(&mut f, &mut c, at)
            .iter()
            .any(|e| matches!(e, Effect::Damage { .. })));
    }
}
#[test]
fn impossible_damage_does_not_interrupt_but_enemy_control_cancels_without_refund() {
    for at in [1, 59, 60, 149] {
        let (mut f, mut c, t) = impossible();
        tick(&mut f, &mut c, t + at - 1);
        f.units[0].hp -= 100;
        assert!(matches!(c.phase, Phase::Impossible { .. }));
        f.interrupted = true;
        let out = tick(&mut f, &mut c, t + at);
        assert_eq!(c.phase, Phase::Idle);
        assert_eq!(c.reduction(t + at), 0);
        assert_eq!(c.pace, 0);
        assert!(out.iter().any(|e| matches!(e, Effect::ClearWarning { .. })));
        assert!(!out.iter().any(|e| matches!(e, Effect::Damage { .. })));
        assert!(c.exhaustion_until >= t + at + 90);
    }
}
#[test]
fn impossible_death_removes_charge_and_reserved_force_without_explosion() {
    let (mut f, mut c, t) = impossible();
    f.units[0].hp = 0;
    let out = tick(&mut f, &mut c, t + 1);
    assert_eq!(c.ball, Ball::Absent);
    assert!(!c.white_storm);
    assert!(!out.iter().any(|e| matches!(e, Effect::Damage { .. })));
    assert!(c.serve.is_none());
}
#[test]
fn impossible_ignores_units_and_walls_until_the_endpoint_explosion() {
    let (mut f, mut c, t) = impossible();
    f.units[1].pos.x = 300000;
    f.units[2] = unit(3, 1, 500000, 300000);
    f.units.push(unit(4, 1, 950000, 300000));
    f.blocks.push(Rect {
        min: Point {
            x: 240000,
            y: 200000,
        },
        max: Point {
            x: 280000,
            y: 400000,
        },
    });
    let mut counts = std::collections::BTreeMap::new();
    let mut saw_crater = false;
    for at in t + 1..t + 500 {
        for e in tick(&mut f, &mut c, at) {
            if let Effect::Damage { target, .. } = e {
                *counts.entry(target).or_insert(0) += 1;
            }
        }
        saw_crater |= c
            .craters
            .iter()
            .any(|c| c.pos.x == 950000 && c.radius == 144000);
    }
    assert_eq!(counts, std::collections::BTreeMap::from([(4, 1)]));
    assert!(saw_crater);
    assert_eq!(c.ball, Ball::Held);
}
#[test]
fn impossible_reserves_force_during_charge_so_bank_expiry_cannot_erase_it() {
    let (mut f, mut c, t) = impossible();
    f.units[1].pos = Point {
        x: 950000,
        y: 300000,
    };
    if let Phase::Impossible { shot, .. } = &mut c.phase {
        shot.caught = 582;
        shot.bank_percent = 300;
    }
    c.caught = 0;
    c.caught_until = t + 1;
    let mut amounts = vec![];
    for at in t + 1..t + 600 {
        for e in tick(&mut f, &mut c, at) {
            if let Effect::Damage { amount, .. } = e {
                amounts.push(amount);
            }
        }
    }
    assert_eq!(amounts[0], 10980);
}
#[test]
fn lower_valid_contact_enters_impossible_without_snapping_to_maximum_height() {
    let (mut f, mut c, t) = impossible();
    c.phase = Phase::Air {
        start: t,
        origin: f.units[0].pos,
        aim: f.units[0].pos,
        max_height: 18020,
        perfect: true,
        enhanced: true,
        set_bars: 5,
        spike_bars: 0,
        white_before: true,
        attempted: false,
        intended: 24,
        hold_start: None,
        hold_ticks: 0,
        serve: true,
        white: false,
    };
    c.ball = Ball::Toss {
        start: t,
        origin: f.units[0].pos,
        to: f.units[0].pos,
        peak: 18120,
    };
    action(&mut f, &mut c, t + 3, ActionKind::Spike);
    assert!(matches!(c.phase,Phase::Impossible{height,..}if height<6000));
    assert_eq!(c.height(t + 4), c.contact_height);
    assert!(c.white_storm);
}
#[test]
fn permanent_form_requires_contact_and_receipts_cannot_manufacture_it() {
    let (mut f, mut c) = (frame(), Core::default());
    tick(&mut f, &mut c, 0);
    c.pending_hits.insert(1, (true, false, true, true));
    f.hits = vec![Hit {
        serial: 1,
        target: 2,
        delivered: true,
        champion: true,
    }];
    tick(&mut f, &mut c, 1);
    assert!(!c.white_storm && !c.transformed);
}
#[test]
fn same_manual_inputs_ignore_every_athlete_stat_and_clones_remain_independent() {
    fn run(profile: [i64; 13]) -> Vec<String> {
        let (mut f, mut c) = (
            frame(),
            Core {
                pace: 100,
                white_storm: true,
                transformed: true,
                ..Core::default()
            },
        );
        f.profile = profile;
        let mut trace = vec![];
        for t in 0..420 {
            let out = match t {
                0 => {
                    f.burst_bars = 5;
                    action(&mut f, &mut c, t, ActionKind::ArmBurst)
                }
                1 => action(&mut f, &mut c, t, ActionKind::SelfSet),
                14 => action(&mut f, &mut c, t, ActionKind::Ultimate),
                62 => action(&mut f, &mut c, t, ActionKind::Takeoff),
                86 => action(&mut f, &mut c, t, ActionKind::Spike),
                _ => tick(&mut f, &mut c, t),
            };
            trace.push(serde_json::to_string(&(&c, &out)).unwrap());
        }
        trace
    }
    let reference = run([50; 13]);
    for i in 0..13 {
        for value in [20, 80] {
            let mut p = [50; 13];
            p[i] = value;
            assert_eq!(run(p), reference, "stat {i}");
        }
    }
    let (_, mut a, _) = impossible();
    let b = a.clone();
    a.pace = 77;
    assert_ne!(a.pace, b.pace);
}
#[test]
fn geometry_sweeps_thin_walls_and_fast_units_without_tunnelling() {
    let a = Point { x: 0, y: 10000 };
    let b = Point {
        x: 100000,
        y: 10000,
    };
    assert!(segment_disk(a, b, Point { x: 50000, y: 10000 }, 1000).is_some());
    assert!(segment_disk(a, b, Point { x: 50000, y: 12000 }, 1000).is_none());
    assert!(segment_rect(
        a,
        b,
        Rect {
            min: Point { x: 51000, y: 0 },
            max: Point { x: 51001, y: 20000 }
        },
        3000
    )
    .is_some());
    assert_eq!(
        closest_point(a, b, Point { x: 45000, y: 80000 }),
        Point { x: 45000, y: 10000 }
    );
}
#[test]
fn aura_role_strength_and_own_reduction_do_not_stack_with_impossible_protection() {
    let (mut f, mut c) = (
        frame(),
        Core {
            white_storm: true,
            transformed: true,
            role: Role::Setter,
            ..Core::default()
        },
    );
    tick(&mut f, &mut c, 0);
    assert!(c
        .buffs
        .values()
        .any(|b| b.channel == "aura" && b.attack == 23 && b.speed == 20 && b.reduce == 23));
    action(&mut f, &mut c, 1, ActionKind::Receive);
    assert_eq!(c.reduction(1), 95);
    assert_eq!(c.reduction(20), 40);
}
#[test]
fn manual_replay_tick_dedup_and_owner_ids_are_independent() {
    let (mut f, mut c) = (frame(), Core::default());
    let first = action(&mut f, &mut c, 0, ActionKind::Receive);
    assert!(!first.is_empty());
    let before = serde_json::to_value(&c).unwrap();
    assert!(c.step(&f).is_empty());
    assert_eq!(serde_json::to_value(&c).unwrap(), before);
    let mut other = c.clone();
    other.pace = 55;
    assert_eq!(c.pace, 0);
    f.units[0].hp = 0;
    tick(&mut f, &mut other, 1);
    assert!(other.buffs.is_empty());
    assert_eq!(c.pace, 0);
}
#[test]
fn seeded_development_and_holdout_sessions_are_deterministic() {
    for seed in 0..36 {
        let mut f = frame();
        f.seed = seed;
        f.manual = false;
        let (mut a, mut b) = (Core::default(), Core::default());
        for t in 0..180 {
            f.tick = t;
            assert_eq!(
                serde_json::to_value(a.step(&f)).unwrap(),
                serde_json::to_value(b.step(&f)).unwrap()
            );
        }
    }
}

#[test]
fn all_thirteen_attributes_change_relevant_ai_traces_in_three_review_fixtures() {
    fn run(index: usize, value: i64, scene: i64) -> Vec<String> {
        let (mut f, mut c) = (frame(), Core::default());
        f.manual = false;
        f.profile[index] = value;
        f.seed = [131, 149, 167][scene as usize];
        f.units[1].pos.x = 225000 + scene * 15000;
        f.units[1].velocity = Point { x: 200, y: 300 };
        let mut camp = unit(4, 1, 240000 + scene * 10000, 320000);
        camp.champion = false;
        camp.farmable = true;
        f.units.push(camp);
        f.danger = 20;
        f.uncertainty = 20 + scene * 10;
        if index == 10 {
            f.danger = 0;
            f.uncertainty = 0;
            f.units.retain(|u| u.id != 3);
        }
        let mut trace = vec![];
        for t in 0..180 {
            f.tick = t;
            f.action = None;
            let out = c.step(&f);
            for e in &out {
                if let Effect::Move { pos } = e {
                    f.units[0].pos = *pos;
                }
            }
            trace.push(
                serde_json::to_string(&(&c.plans, &c.phase, c.reaction_until, &out)).unwrap(),
            );
        }
        trace
    }
    for i in 0..13 {
        let differences = (0..3).filter(|s| run(i, 20, *s) != run(i, 80, *s)).count();
        assert!(
            differences > 0,
            "Stat {i} had no planning/execution effect across three fixtures"
        );
    }
}

#[test]
fn burst_receive_force_keeps_its_origin_until_spent_and_cannot_refund_pace() {
    let (mut f, mut c) = (
        frame(),
        Core {
            pace: 100,
            ..Core::default()
        },
    );
    arm(&mut f, &mut c, 0, 1);
    action(&mut f, &mut c, 1, ActionKind::Receive);
    f.receipts = vec![receipt(1, Some(300))];
    tick(&mut f, &mut c, 2);
    assert!(c.bank_burst);
    f.receipts.clear();
    tick(&mut f, &mut c, 160); // exhaustion is over, force still valid
    let p = f.units[0].pos;
    c.phase = Phase::Air {
        start: 161,
        origin: p,
        aim: p,
        max_height: 4800,
        perfect: true,
        enhanced: false,
        set_bars: 0,
        spike_bars: 0,
        white_before: false,
        attempted: false,
        intended: 24,
        hold_start: None,
        hold_ticks: 0,
        serve: false,
        white: false,
    };
    c.ball = Ball::Toss {
        start: 161,
        origin: p,
        to: p,
        peak: 4900,
    };
    action(&mut f, &mut c, 185, ActionKind::Spike);
    let serial = match c.ball {
        Ball::Flying {
            serial,
            burst,
            caught,
            ..
        } => {
            assert!(burst);
            assert_eq!(caught, 300);
            serial
        }
        _ => panic!(),
    };
    let mut delivered = false;
    for t in 186..260 {
        let out = tick(&mut f, &mut c, t);
        if out
            .iter()
            .any(|e| matches!(e, Effect::Damage { target: 2, .. }))
        {
            f.hits = vec![Hit {
                serial,
                target: 2,
                delivered: true,
                champion: true,
            }];
            tick(&mut f, &mut c, t + 1);
            delivered = true;
            break;
        }
    }
    assert!(delivered);
    assert_eq!(c.pace, 80);
}
#[test]
fn realm_transfer_cancels_charge_and_owned_cross_realm_work_without_damage() {
    let (mut f, mut c, start) = impossible();
    tick(&mut f, &mut c, start + 20);
    f.realm = Some(9);
    let out = tick(&mut f, &mut c, start + 21);
    assert_eq!(c.phase, Phase::Idle);
    assert_eq!(c.ball, Ball::Held);
    assert!(out.iter().any(|e| matches!(e, Effect::ClearWarning { .. })));
    assert!(!out
        .iter()
        .any(|e| matches!(e, Effect::Damage { .. } | Effect::KnockUp { .. })));
    assert!(c.exhaustion_until >= start + 21 + 90);
    assert!(c.craters.is_empty());
    assert!(c.serve.is_none());
    let before = c.clone();
    let clone_out = tick(&mut f, &mut c, start + 600);
    assert!(!clone_out.iter().any(|e| matches!(e, Effect::Damage { .. })));
    assert_eq!(c.cds, before.cds);
}
#[test]
fn impossible_requires_every_qualification_and_keeps_existing_serve_deadline() {
    for missing in 0..3 {
        let (mut f, mut c) = (frame(), Core::default());
        tick(&mut f, &mut c, 0);
        c.white_storm = true;
        c.transformed = true;
        c.thunder_earned = true;
        c.serve = Some(Serve {
            committed: true,
            origin: f.units[0].pos,
            aim: Point {
                x: 950000,
                y: 300000,
            },
            until: 2000,
        });
        c.ultimate_until = 2000;
        let p = f.units[0].pos;
        c.phase = Phase::Air {
            start: 1,
            origin: p,
            aim: p,
            max_height: 18020,
            perfect: missing != 1,
            enhanced: true,
            set_bars: if missing == 2 { 3 } else { 5 },
            spike_bars: 0,
            white_before: missing != 0,
            attempted: false,
            intended: 24,
            hold_start: None,
            hold_ticks: 0,
            serve: missing != 3,
            white: false,
        };
        let t = if missing == 4 { 19 } else { 25 };
        let h = c.height(t);
        c.ball = Ball::Toss {
            start: 1,
            origin: p,
            to: p,
            peak: h + 100,
        };
        action(&mut f, &mut c, t, ActionKind::Spike);
        assert!(
            !matches!(c.phase, Phase::Impossible { .. }),
            "near miss {missing}"
        );
    }
    let (mut f, mut c, start) = impossible();
    assert!(c.ultimate_until >= start + 300 + 24);
    let until = c.ultimate_until;
    tick(&mut f, &mut c, start + 301);
    assert_eq!(c.ultimate_until, until);
}

// Unchanged non-resource regressions retained from the 2.2 baseline.
mod retained_v22 {
    use super::*;
    fn act(f: &mut Frame, c: &mut Core, t: u64, k: ActionKind) -> Vec<Effect> {
        f.tick = t;
        f.action = Some(Action {
            kind: k,
            target: Some(2),
            aim: Point {
                x: 400000,
                y: 300000,
            },
        });
        let out = c.step(f);
        moves(f, &out);
        out
    }
    #[test]
    fn canonical_damage_thresholds_are_strict() {
        for (h, tier, mult) in [
            (3999, AttackTier::Normal, 100),
            (4000, AttackTier::Thunder, 180),
            (5999, AttackTier::Thunder, 230),
            (6000, AttackTier::BlackThunder, 500),
            (9999, AttackTier::BlackThunder, 500),
            (10000, AttackTier::WhiteStorm, 600),
        ] {
            assert_eq!(attack_tier(h), tier);
            assert_eq!(thunder(h), mult);
            assert_eq!(attack_tier(height_label(h)), tier);
        }
    }
    #[test]
    fn farming_assignment_affects_selection_but_not_incidental_skill_damage() {
        let (mut f, mut c) = (frame(), Core::default());
        f.manual = false;
        f.units.retain(|u| u.id != 3);
        f.units[1].champion = false;
        f.units[1].farmable = false;
        tick(&mut f, &mut c, 0);
        tick(&mut f, &mut c, 30);
        assert!(
            c.plans.is_empty(),
            "An unassigned camp is not an AI farming target"
        );
        let p = f.units[1].pos;
        c.ball = Ball::Flying {
            serial: 1,
            origin: p,
            pos: p,
            to: p,
            amount: 300,
            basic: false,
            speed: 1000,
            caught: 0,
            height: 4800,
            black: false,
            thunder: true,
            burst: false,
            impossible: false,
            power: 100,
            radius: data::SPIKE_RADIUS,
            overpressure: false,
            hit: Default::default(),
            white: false,
            serve: false,
        };
        assert!(tick(&mut f, &mut c, 31).iter().any(|e| matches!(
            e,
            Effect::Damage {
                target: 2,
                amount: 300,
                ..
            }
        )));
    }
    #[test]
    fn accepted_volley_can_hit_a_building_while_spike_splash_excludes_it() {
        let (mut f, mut c) = (frame(), Core::default());
        f.units[1].damageable = false;
        f.units[1].champion = false;
        f.basics = vec![Basic {
            serial: 1,
            target: 2,
        }];
        let first = c.step(&f);
        assert!(!first.iter().any(|e| matches!(e, Effect::Damage { .. })));
        f.basics.clear();
        let mut damage = 0;
        for t in 1..30 {
            for e in tick(&mut f, &mut c, t) {
                if let Effect::Damage {
                    target: 2,
                    amount,
                    basic: true,
                    ..
                } = e
                {
                    damage += amount;
                }
            }
        }
        assert_eq!(damage, 78, "One 75%-Attack volley, no range re-rejection");
        let p = f.units[1].pos;
        c.ball = Ball::Flying {
            serial: 2,
            origin: p,
            pos: p,
            to: p,
            amount: 300,
            basic: false,
            speed: 1000,
            caught: 0,
            height: 4800,
            black: false,
            thunder: true,
            burst: false,
            impossible: false,
            power: 100,
            radius: data::SPIKE_RADIUS,
            overpressure: false,
            hit: Default::default(),
            white: false,
            serve: false,
        };
        assert!(!tick(&mut f, &mut c, 31)
            .iter()
            .any(|e| matches!(e, Effect::Damage { target: 2, .. })));
    }
    #[test]
    fn enlarged_spike_aoe_includes_the_boundary_once_and_excludes_the_next_unit() {
        let (mut f, mut c) = (frame(), Core::default());
        tick(&mut f, &mut c, 0);
        let p = Point {
            x: 250000,
            y: 300000,
        };
        f.units[1].pos = Point {
            x: 305000,
            y: 300000,
        };
        f.units[2].team = 1;
        f.units[2].pos = Point {
            x: 305001,
            y: 300000,
        };
        c.ball = Ball::Flying {
            serial: 1,
            origin: p,
            pos: p,
            to: p,
            amount: 300,
            basic: false,
            speed: 1000,
            caught: 0,
            height: 4800,
            black: false,
            thunder: true,
            burst: false,
            impossible: false,
            power: 100,
            radius: data::SPIKE_RADIUS,
            overpressure: false,
            hit: Default::default(),
            white: false,
            serve: false,
        };
        let out = tick(&mut f, &mut c, 1);
        assert_eq!(
            out.iter()
                .filter(|e| matches!(
                    e,
                    Effect::Damage {
                        target: 2,
                        amount: 300,
                        ..
                    }
                ))
                .count(),
            1
        );
        assert!(!out
            .iter()
            .any(|e| matches!(e, Effect::Damage { target: 3, .. })));
    }
    #[test]
    fn teammate_ball_and_buff_survive_rally_jump_then_buff_lingers_without_the_ball() {
        let (mut f, mut c) = (frame(), Core::default());
        f.action = Some(Action {
            kind: ActionKind::AllySet,
            target: Some(3),
            aim: f.units[2].pos,
        });
        c.step(&f);
        tick(&mut f, &mut c, 12);
        act(&mut f, &mut c, 13, ActionKind::Approach);
        for t in 14..37 {
            tick(&mut f, &mut c, t);
        }
        act(&mut f, &mut c, 37, ActionKind::Takeoff);
        for t in 38..90 {
            tick(&mut f, &mut c, t);
        }
        assert_eq!(c.support.unwrap().0, 3);
        assert!(c.buffs.values().any(|b| b.channel == "support_ball"));
        tick(&mut f, &mut c, 192);
        assert!(c.support.is_none());
        assert!(!c.buffs.values().any(|b| b.channel == "support_ball"));
        assert!(c
            .buffs
            .values()
            .any(|b| b.channel == "support" && b.until == 312));
        // Self Set does not cancel the teammate's remaining support benefit.
        c.ball = Ball::Held;
        act(&mut f, &mut c, 240, ActionKind::SelfSet);
        tick(&mut f, &mut c, 252);
        assert!(c.helper.is_some());
        assert!(c.buffs.values().any(|b| b.channel == "support"));
        tick(&mut f, &mut c, 312);
        assert!(!c.buffs.values().any(|b| b.channel == "support"));
    }
    #[test]
    fn set_ball_travels_a_complete_arc_without_freezing_at_apex() {
        let a = Point {
            x: 100000,
            y: 200000,
        };
        let b = Point {
            x: 200000,
            y: 200000,
        };
        let mut last = a;
        for t in 0..=48 {
            let (p, h) = toss(0, a, b, 5600, t);
            assert!(p.x >= last.x);
            assert!(h >= 2000);
            if t == 24 {
                assert_eq!((p.x, h), (150000, 5600));
            }
            last = p;
        }
        assert_eq!(toss(0, a, b, 5600, 48), (b, 2000));
        assert!(toss(0, a, b, 5600, 36).0.x > toss(0, a, b, 5600, 24).0.x);
    }
    #[test]
    fn falling_ball_visual_retains_contact_height_then_reaches_the_impact_plane() {
        let origin = Point {
            x: 100000,
            y: 100000,
        };
        let to = Point {
            x: 200000,
            y: 100000,
        };
        assert_eq!(flight_height(origin, origin, to, 6000), 6000);
        assert_eq!(
            flight_height(
                origin,
                Point {
                    x: 150000,
                    y: 100000
                },
                to,
                6000
            ),
            4000
        );
        assert_eq!(flight_height(origin, to, to, 6000), 2000);
        assert_eq!(flight_height(origin, origin, origin, 6000), 2000);
    }
    #[test]
    fn circular_smoke_uses_its_real_boundary_and_does_not_block_movement() {
        let mut f = frame();
        f.smoke_disks.push(Disk {
            pos: Point {
                x: 200000,
                y: 300000,
            },
            radius: 50000,
        });
        assert!(!f.clear(
            Point {
                x: 140000,
                y: 300000
            },
            Point {
                x: 260000,
                y: 300000
            },
            0,
            true
        ));
        assert!(f.clear(
            Point {
                x: 140000,
                y: 300000
            },
            Point {
                x: 260000,
                y: 300000
            },
            0,
            false
        ));
        assert!(f.clear(
            Point {
                x: 140000,
                y: 350001
            },
            Point {
                x: 260000,
                y: 350001
            },
            0,
            true
        ));
        assert!(!f.clear(
            Point {
                x: 140000,
                y: 350000
            },
            Point {
                x: 260000,
                y: 350000
            },
            0,
            true
        ));
    }
    #[test]
    fn locked_skill_rejects_without_spending_cooldown() {
        let (mut f, mut c) = (frame(), Core::default());
        f.skill2 = false;
        act(&mut f, &mut c, 0, ActionKind::SelfSet);
        assert_eq!(c.cds[1], 0);
        assert!(c.rejection.contains("learned"));
    }
    #[test]
    fn self_set_is_prepared_then_consumed_at_optional_takeoff() {
        let (mut f, mut c) = (frame(), Core::default());
        act(&mut f, &mut c, 0, ActionKind::SelfSet);
        assert_eq!(c.set_ready, 0);
        tick(&mut f, &mut c, 12);
        assert_eq!(c.set_ready, 252);
        act(&mut f, &mut c, 13, ActionKind::Approach);
        assert_eq!(c.set_ready, 252);
        assert!(matches!(c.phase, Phase::Approach { enhanced: true, .. }));
        assert_eq!(c.ball, Ball::Held);
        for t in 14..37 {
            tick(&mut f, &mut c, t);
        }
        act(&mut f, &mut c, 37, ActionKind::Takeoff);
        assert_eq!(c.set_ready, 0);
        assert!(matches!(c.ball, Ball::Toss { peak: 5200, .. }));
    }
    #[test]
    fn ally_support_never_takes_the_combat_ball() {
        let (mut f, mut c) = (frame(), Core::default());
        f.action = Some(Action {
            kind: ActionKind::AllySet,
            target: Some(3),
            aim: f.units[2].pos,
        });
        c.step(&f);
        tick(&mut f, &mut c, 12);
        assert_eq!(c.ball, Ball::Held);
        assert_eq!(c.support.unwrap().0, 3);
        assert_eq!(c.support.unwrap().1, 192);
        assert_eq!(c.buffs["3:support"].attack, 12);
        assert_eq!(c.buffs["3:support"].speed, 8);
    }
    #[test]
    fn ally_set_revalidates_visibility_and_never_refunds() {
        let (mut f, mut c) = (frame(), Core::default());
        f.action = Some(Action {
            kind: ActionKind::AllySet,
            target: Some(3),
            aim: f.units[2].pos,
        });
        c.step(&f);
        f.units[2].visible = false;
        tick(&mut f, &mut c, 12);
        assert_eq!(c.support, None);
        assert_eq!(c.cds[1], 240);
    }
    #[test]
    fn wall_or_smoke_rejects_ally_set_before_spending() {
        for smoke in [false, true] {
            let (mut f, mut c) = (frame(), Core::default());
            let r = Rect {
                min: Point {
                    x: 158000,
                    y: 290000,
                },
                max: Point {
                    x: 165000,
                    y: 330000,
                },
            };
            if smoke {
                f.smoke.push(r)
            } else {
                f.blocks.push(r)
            }
            f.action = Some(Action {
                kind: ActionKind::AllySet,
                target: Some(3),
                aim: f.units[2].pos,
            });
            c.step(&f);
            assert_eq!(c.cds[1], 0);
        }
    }
    #[test]
    fn perfect_takeoff_requires_real_displacement() {
        for blocked in [false, true] {
            let (mut f, mut c) = (frame(), Core::default());
            act(&mut f, &mut c, 0, ActionKind::Approach);
            for t in 1..24 {
                if blocked {
                    f.rooted = true
                }
                tick(&mut f, &mut c, t);
            }
            f.rooted = false;
            act(&mut f, &mut c, 24, ActionKind::Takeoff);
            assert!(matches!(c.phase,Phase::Air{perfect,..}if perfect!=blocked));
        }
    }
    #[test]
    fn takeoff_boundaries_and_expiry() {
        for (at, perfect) in [
            (17, false),
            (18, false),
            (23, false),
            (24, true),
            (29, true),
            (30, false),
            (35, false),
        ] {
            let (mut f, mut c) = (frame(), Core::default());
            act(&mut f, &mut c, 0, ActionKind::Approach);
            for t in 1..at {
                tick(&mut f, &mut c, t);
            }
            act(&mut f, &mut c, at, ActionKind::Takeoff);
            assert!(
                matches!(c.phase,Phase::Air{perfect:p,..}if p==perfect),
                "{at}"
            );
        }
        let (mut f, mut c) = (frame(), Core::default());
        act(&mut f, &mut c, 0, ActionKind::Approach);
        tick(&mut f, &mut c, 180);
        assert_eq!(c.phase, Phase::Idle);
        assert_eq!(c.cds[2], 300);
    }
    #[test]
    fn contact_miss_spends_the_single_attempt() {
        let (mut f, mut c) = (frame(), Core::default());
        act(&mut f, &mut c, 0, ActionKind::Approach);
        for t in 1..24 {
            tick(&mut f, &mut c, t);
        }
        act(&mut f, &mut c, 24, ActionKind::Takeoff);
        act(&mut f, &mut c, 25, ActionKind::Spike);
        assert!(c.rejection.contains("Contact missed"));
        assert!(matches!(
            c.phase,
            Phase::Air {
                attempted: true,
                ..
            }
        ));
        assert!(!c.flash_blocked() == false);
    }
    #[test]
    fn complete_timed_contact_produces_a_swept_physical_spike() {
        let (mut f, mut c) = (frame(), Core::default());
        act(&mut f, &mut c, 0, ActionKind::SelfSet);
        tick(&mut f, &mut c, 12);
        act(&mut f, &mut c, 13, ActionKind::Approach);
        for t in 14..37 {
            tick(&mut f, &mut c, t);
        }
        act(&mut f, &mut c, 37, ActionKind::Takeoff);
        for t in 38..61 {
            tick(&mut f, &mut c, t);
        }
        let out = act(&mut f, &mut c, 61, ActionKind::Spike);
        assert!(
            matches!(
                c.ball,
                Ball::Flying {
                    amount: 763,
                    height: 5100,
                    ..
                }
            ),
            "{:?}",
            c.ball
        );
        assert!(out
            .iter()
            .any(|e| matches!(e, Effect::Warning { radius: 45000, .. })));
    }
    #[test]
    fn grounded_or_empty_ball_cannot_be_duplicated() {
        let (mut f, mut c) = (frame(), Core::default());
        tick(&mut f, &mut c, 0);
        c.ball = Ball::Grounded {
            pos: Point {
                x: 800000,
                y: 800000,
            },
        };
        act(&mut f, &mut c, 1, ActionKind::Approach);
        assert_eq!(c.cds[2], 301, "Run-up is useful without a ball");
        assert!(matches!(c.ball, Ball::Grounded { .. }));
        tick(&mut f, &mut c, 91);
        act(&mut f, &mut c, 92, ActionKind::SelfSet);
        assert_eq!(c.cds[1], 0);
    }
    #[test]
    fn accepted_palm_ignores_coarse_wall_rechecks() {
        let (mut f, mut c) = (frame(), Core::default());
        f.units[1].pos.x = 185000;
        f.blocks.push(Rect {
            min: Point {
                x: 160000,
                y: 280000,
            },
            max: Point {
                x: 170000,
                y: 320000,
            },
        });
        f.basics.push(Basic {
            serial: 1,
            target: 2,
        });
        assert!(c.step(&f).iter().any(|e| matches!(
            e,
            Effect::Damage {
                target: 2,
                amount: 104,
                basic: true,
                ..
            }
        )));
    }
    #[test]
    fn projectile_sweeps_and_hits_each_victim_once() {
        let (mut f, mut c) = (frame(), Core::default());
        tick(&mut f, &mut c, 0);
        c.ball = Ball::Flying {
            serial: 1,
            origin: Point {
                x: 180000,
                y: 300000,
            },
            pos: Point {
                x: 180000,
                y: 300000,
            },
            to: Point {
                x: 250000,
                y: 300000,
            },
            amount: 300,
            basic: false,
            speed: 85000,
            caught: 80,
            height: 4800,
            black: false,
            thunder: true,
            burst: false,
            impossible: false,
            power: 100,
            radius: data::SPIKE_RADIUS,
            overpressure: false,
            hit: Default::default(),
            white: false,
            serve: false,
        };
        let out = tick(&mut f, &mut c, 1);
        let damage = out
            .iter()
            .filter(|e| {
                matches!(
                    e,
                    Effect::Damage {
                        target: 2,
                        amount: 380,
                        ..
                    }
                )
            })
            .count();
        assert_eq!(damage, 1);
        assert!(!tick(&mut f, &mut c, 2)
            .iter()
            .any(|e| matches!(e, Effect::Damage { .. })));
    }
    #[test]
    fn aura_linger_and_self_cleanup_are_explicit() {
        let (mut f, mut c) = (
            frame(),
            Core {
                role: Role::Setter,
                ..Core::default()
            },
        );
        tick(&mut f, &mut c, 0);
        assert_eq!(c.buffs["3:aura"].until, 0);
        c.role = Role::Spiker;
        tick(&mut f, &mut c, 1);
        assert!(!c.buffs.contains_key("1:aura"));
        assert_eq!(c.buffs["3:aura"].until, 121);
        tick(&mut f, &mut c, 121);
        assert!(!c.buffs.contains_key("3:aura"));
    }
    #[test]
    fn control_speed_reacts_once_to_a_new_visible_situation() {
        for (v, ready) in [(20, 21), (80, 10)] {
            let (mut f, mut c) = (frame(), Core::default());
            f.manual = false;
            f.profile[3] = v;
            tick(&mut f, &mut c, 0);
            assert_eq!(c.reaction_until, ready);
            tick(&mut f, &mut c, 1);
            assert_eq!(c.reaction_until, ready);
        }
    }
    #[test]
    fn hidden_enemy_positions_do_not_change_plans() {
        let mut a = frame();
        a.manual = false;
        a.units[1].visible = false;
        let mut b = a.clone();
        b.units[1].pos.x = 900000;
        let (mut ca, mut cb) = (Core::default(), Core::default());
        assert_eq!(
            serde_json::to_string(&ca.step(&a)).unwrap(),
            serde_json::to_string(&cb.step(&b)).unwrap()
        );
    }
    #[test]
    fn integer_geometry_remains_safe_at_map_extents() {
        assert!(
            Point { x: 0, y: 0 }.distance(Point {
                x: 960000,
                y: 960000
            }) < 1400000
        );
    }
    #[test]
    fn accepted_skills_select_roles_but_rejected_commands_do_not() {
        let (mut f, mut c) = (frame(), Core::default());
        act(&mut f, &mut c, 0, ActionKind::Receive);
        assert_eq!(c.role, Role::Receiver);
        f.skill2 = false;
        act(&mut f, &mut c, 1, ActionKind::SelfSet);
        assert_eq!(c.role, Role::Receiver);
        f.skill2 = true;
        act(&mut f, &mut c, 2, ActionKind::SelfSet);
        assert_eq!(c.role, Role::Setter);
        tick(&mut f, &mut c, 14);
        act(&mut f, &mut c, 15, ActionKind::Approach);
        assert_eq!(c.role, Role::Spiker);
        assert!(c.set_ready > 15);
        assert!(
            c.buffs.contains_key("3:aura"),
            "Teammate aura lingers after Run-up selects Spiker"
        );
        assert!(!c.buffs.contains_key("1:aura"));
    }
    #[test]
    fn retired_stance_commands_cannot_bypass_skill_cooldowns() {
        let (mut f, mut c) = (frame(), Core::default());
        for (t, k) in [ActionKind::Setter, ActionKind::Receiver, ActionKind::Spiker]
            .into_iter()
            .enumerate()
        {
            act(&mut f, &mut c, t as u64, k);
            assert_eq!(c.role, Role::Spiker);
            assert_eq!(c.cds, [0; 4]);
            assert!(c.rejection.contains("Retired"));
        }
    }
    #[test]
    fn run_up_follows_steering_without_throwing_or_chasing_the_ball() {
        let (mut f, mut c) = (frame(), Core::default());
        f.steer = Point { x: 0, y: -1 };
        let start = f.units[0].pos;
        act(&mut f, &mut c, 0, ActionKind::Approach);
        tick(&mut f, &mut c, 1);
        assert_eq!(f.units[0].pos.x, start.x);
        assert_eq!(start.y - f.units[0].pos.y, 2100);
        assert_eq!(c.ball, Ball::Held);
        f.steer = Point { x: 0, y: 0 };
        let stop = f.units[0].pos;
        for t in 2..=180 {
            tick(&mut f, &mut c, t);
        }
        assert_eq!(f.units[0].pos, stop);
        assert_eq!(c.ball, Ball::Held);
        assert_eq!(c.phase, Phase::Idle);
    }
    #[test]
    fn serve_channel_interruption_keeps_spent_cooldown_and_releases_movement() {
        let (mut f, mut c) = (frame(), Core::default());
        act(&mut f, &mut c, 0, ActionKind::Ultimate);
        assert!(tick(&mut f, &mut c, 1)
            .iter()
            .any(|e| matches!(e,Effect::Move{pos}if *pos==f.units[0].pos)));
        f.interrupted = true;
        tick(&mut f, &mut c, 2);
        assert_eq!(c.cds[3], 4500);
        assert!(c.serve.is_none());
        assert_eq!(c.phase, Phase::Idle);
    }
    #[test]
    fn black_crater_knocks_once_then_only_slows_and_expires() {
        let (mut f, mut c) = (frame(), Core::default());
        tick(&mut f, &mut c, 0);
        let p = f.units[1].pos;
        c.ball = Ball::Flying {
            serial: 1,
            origin: p,
            pos: p,
            to: p,
            amount: 500,
            basic: false,
            speed: 1000,
            caught: 0,
            height: 6000,
            black: true,
            thunder: true,
            burst: true,
            impossible: false,
            power: 100,
            radius: data::BLACK_RADIUS,
            overpressure: false,
            hit: Default::default(),
            white: false,
            serve: false,
        };
        let out = tick(&mut f, &mut c, 1);
        assert_eq!(
            out.iter()
                .filter(|e| matches!(
                    e,
                    Effect::KnockUp {
                        target: 2,
                        ticks: 30
                    }
                ))
                .count(),
            1
        );
        for t in [2, 60, 120] {
            let out = tick(&mut f, &mut c, t);
            assert!(out.iter().any(|e| matches!(
                e,
                Effect::Slow {
                    target: 2,
                    percent: 20,
                    ..
                }
            )));
            assert!(!out
                .iter()
                .any(|e| matches!(e, Effect::KnockUp { .. } | Effect::Damage { .. })));
        }
        assert!(!tick(&mut f, &mut c, 121)
            .iter()
            .any(|e| matches!(e, Effect::Slow { .. })));
    }
    #[test]
    fn crater_respects_immunity_and_does_not_launch_objectives() {
        let (mut f, mut c) = (frame(), Core::default());
        tick(&mut f, &mut c, 0);
        f.units[1].control_eligible = false;
        f.units[2].team = 1;
        f.units[2].cc_immune = true;
        f.units[2].pos = f.units[1].pos;
        let p = f.units[1].pos;
        c.ball = Ball::Flying {
            serial: 1,
            origin: p,
            pos: p,
            to: p,
            amount: 500,
            basic: false,
            speed: 1000,
            caught: 0,
            height: 6000,
            black: true,
            thunder: true,
            burst: true,
            impossible: false,
            power: 100,
            radius: data::BLACK_RADIUS,
            overpressure: false,
            hit: Default::default(),
            white: false,
            serve: false,
        };
        assert!(!tick(&mut f, &mut c, 1)
            .iter()
            .any(|e| matches!(e, Effect::KnockUp { .. })));
        assert!(!tick(&mut f, &mut c, 2)
            .iter()
            .any(|e| matches!(e, Effect::Slow { .. })));
    }
    #[test]
    fn cone_geometry_includes_axis_and_rejects_sideways_and_backwards() {
        let p = Point {
            x: 100000,
            y: 100000,
        };
        let a = Point {
            x: 200000,
            y: 100000,
        };
        assert!(in_serve_cone(
            p,
            a,
            Point {
                x: 300000,
                y: 110000
            }
        ));
        assert!(!in_serve_cone(
            p,
            a,
            Point {
                x: 100000,
                y: 200000
            }
        ));
        assert!(!in_serve_cone(p, a, Point { x: 0, y: 100000 }));
    }
    #[test]
    fn opening_serve_does_not_request_vision_or_expose_hidden_enemies() {
        let (mut f, mut c) = (frame(), Core::default());
        f.units[1].visible = false;
        act(&mut f, &mut c, 0, ActionKind::Ultimate);
        for t in 1..48 {
            let out = tick(&mut f, &mut c, t);
            assert!(!serde_json::to_string(&out).unwrap().contains("reveal"));
            assert!(!f.units[1].visible);
        }
    }
}

#[test]
fn athlete_ai_can_choose_and_complete_an_impossible_sequence_with_real_resources() {
    let (mut f, mut c) = (
        frame(),
        Core {
            pace: 100,
            white_storm: true,
            transformed: true,
            thunder_earned: true,
            ..Core::default()
        },
    );
    f.manual = false;
    f.profile = [100; 13];
    f.units[1].pos.x = 550000;
    f.units[1].hp = 100000;
    f.units[1].max_hp = 100000;
    f.danger = 0;
    let mut offered = false;
    let mut began = false;
    for t in 0..700 {
        tick(&mut f, &mut c, t);
        offered |= c
            .plans
            .iter()
            .any(|p| p.reason.contains("Impossible") && p.bars == 5);
        if matches!(c.phase, Phase::Impossible { .. }) {
            began = true;
            break;
        }
    }
    assert!(offered);
    assert!(began, "{}", c.rejection);
    assert_eq!(c.pace, 0);
}

#[test]
fn impossible_clips_the_committed_ray_to_realm_bounds_without_bending_the_aim() {
    let (mut f, mut c, start) = impossible();
    f.bounds = Rect {
        min: Point {
            x: 100000,
            y: 250000,
        },
        max: Point {
            x: 400000,
            y: 350000,
        },
    };
    let origin = f.units[0].pos;
    let desired = Point {
        x: 900000,
        y: 600000,
    };
    f.aim = Some(desired);
    tick(&mut f, &mut c, start + 1);
    let aim = match c.phase {
        Phase::Impossible { aim, .. } => aim,
        _ => panic!(),
    };
    assert!(aim.x <= 400000 - data::BALL_RADIUS && aim.y <= 350000 - data::BALL_RADIUS);
    let cross = (aim.x - origin.x) as i128 * (desired.y - origin.y) as i128
        - (aim.y - origin.y) as i128 * (desired.x - origin.x) as i128;
    assert!(
        cross.abs() < 1_000_000,
        "axis clipping bent the shot: {cross}"
    );
    assert!(in_serve_cone(origin, c.serve.as_ref().unwrap().aim, aim));
}

fn local_impossible(contact_age: u64) -> (Frame, Core, u64) {
    let mut f = frame();
    let mut c = Core {
        pace: 100,
        white_storm: true,
        transformed: true,
        thunder_earned: true,
        ..Core::default()
    };
    f.ultimate = false;
    arm(&mut f, &mut c, 0, 5);
    action(&mut f, &mut c, 1, ActionKind::SelfSet);
    for t in 2..=13 {
        tick(&mut f, &mut c, t);
    }
    action(&mut f, &mut c, 14, ActionKind::Approach);
    for t in 15..38 {
        tick(&mut f, &mut c, t);
    }
    action(&mut f, &mut c, 38, ActionKind::Takeoff);
    for t in 39..38 + contact_age {
        tick(&mut f, &mut c, t);
    }
    let t = 38 + contact_age;
    action(&mut f, &mut c, t, ActionKind::Spike);
    assert!(
        matches!(c.phase, Phase::Impossible { .. }),
        "{}",
        c.rejection
    );
    (f, c, t)
}
#[test]
fn nonserve_five_bar_set_reaches_10m_without_ultimate_and_is_not_global() {
    let (mut f, mut c, t) = local_impossible(24);
    assert_eq!(c.height(t), 10600);
    assert_eq!(c.last_power, 359);
    assert_eq!(c.pace, 0);
    assert!(c.serve.is_none());
    f.aim = Some(Point {
        x: 950000,
        y: 300000,
    });
    for age in 1..=300 {
        tick(&mut f, &mut c, t + age);
    }
    let origin = f.units[0].pos;
    assert!(
        matches!(c.ball,Ball::Flying{to,radius:80000,serve:false,impossible:true,..} if origin.distance(to)<=400000)
    );
    assert_eq!(c.exhaustion_until, t + 390);
}
#[test]
fn five_bar_takeoff_below_10m_releases_an_ordinary_attack() {
    let (mut f, mut c) = (
        frame(),
        Core {
            pace: 100,
            white_storm: true,
            transformed: true,
            ..Core::default()
        },
    );
    arm(&mut f, &mut c, 0, 5);
    action(&mut f, &mut c, 1, ActionKind::Approach);
    for t in 2..25 {
        tick(&mut f, &mut c, t);
    }
    action(&mut f, &mut c, 25, ActionKind::Takeoff);
    for t in 26..49 {
        tick(&mut f, &mut c, t);
    }
    action(&mut f, &mut c, 49, ActionKind::Spike);
    assert!(matches!(
        c.ball,
        Ball::Flying {
            impossible: false,
            height: 7800,
            ..
        }
    ));
    assert!(!matches!(c.phase, Phase::Impossible { .. }));
    assert_eq!(c.pace, 0);
}
#[test]
fn serve_early_contact_keeps_actual_height_without_a_10m_requirement() {
    let (mut f, mut c, t) = impossible();
    c.phase = Phase::Air {
        start: t,
        origin: f.units[0].pos,
        aim: f.units[0].pos,
        max_height: 18020,
        perfect: true,
        enhanced: true,
        set_bars: 5,
        spike_bars: 0,
        white_before: true,
        attempted: false,
        intended: 24,
        hold_start: None,
        hold_ticks: 0,
        serve: true,
        white: false,
    };
    c.ball = Ball::Toss {
        start: t,
        origin: f.units[0].pos,
        to: f.units[0].pos,
        peak: 18120,
    };
    action(&mut f, &mut c, t + 3, ActionKind::Spike);
    let height = c.height(t + 3);
    assert!(height < 6000);
    tick(&mut f, &mut c, t + 153);
    assert_eq!(c.height(t + 153), height);
}
#[test]
fn unsupported_costs_and_incomplete_thresholds_never_spend() {
    for pace in [0, 19, 20, 55, 59, 60, 99, 100] {
        for bars in [1u8, 2, 3, 4, 5] {
            let (mut f, mut c) = (
                frame(),
                Core {
                    pace,
                    ..Core::default()
                },
            );
            arm(&mut f, &mut c, 0, bars);
            let allowed = [1, 3, 5].contains(&bars) && pace >= bars as i64 * 20;
            assert_eq!(c.armed_bars, if allowed { bars } else { 0 });
            assert_eq!(c.pace, pace);
        }
    }
}
#[test]
fn burst_set_expiry_exhausts_once_and_releases_the_play_lock() {
    let (mut f, mut c) = (
        frame(),
        Core {
            pace: 100,
            ..Core::default()
        },
    );
    arm(&mut f, &mut c, 0, 3);
    action(&mut f, &mut c, 1, ActionKind::SelfSet);
    tick(&mut f, &mut c, 13);
    assert_eq!(c.exhaustion_until, 0);
    assert!(c.burst_play.is_some());
    tick(&mut f, &mut c, 252);
    assert_eq!(c.exhaustion_until, 0);
    tick(&mut f, &mut c, 253);
    assert_eq!(c.exhaustion_until, 343);
    assert!(c.burst_play.is_none());
    tick(&mut f, &mut c, 254);
    assert_eq!(c.exhaustion_until, 343);
}
#[test]
fn charge_uses_immutable_contact_stats_and_never_compounds_power() {
    let (mut f, mut c, t) = impossible();
    f.units[0].attack = 99999;
    for age in [1, 2, 149, 239, 240, 299] {
        tick(&mut f, &mut c, t + age);
        assert_eq!(c.last_power, 470 + 200 * age as i64 / 300);
        if let Phase::Impossible { ref shot, .. } = c.phase {
            assert_eq!(shot.base_damage, 194);
            assert_eq!(shot.starting_power, 470);
        }
    }
    tick(&mut f, &mut c, t + 300);
    assert!(matches!(c.ball, Ball::Flying { amount: 10398, .. }));
}
#[test]
fn impossible_half_speed_cap_is_600_and_receive_does_not_multiply_damage() {
    let (_, c, _) = impossible();
    let shot = if let Phase::Impossible { shot, .. } = c.phase {
        shot
    } else {
        panic!()
    };
    let mut full = shot.clone();
    full.bank_percent = 300;
    full.caught = 582;
    let final_shot = Core::charge_shot(&full, 300);
    assert_eq!(final_shot.amount, 10398);
    assert_eq!(final_shot.caught, 582);
    assert_eq!(final_shot.speed, data::IMPOSSIBLE_MAX_SPEED / 60);
    assert!(Core::kmh_tenths(final_shot.speed) <= 6000);
    assert_eq!(Core::charge_shot(&full, 9999).power, 670);
}
#[test]
fn charge_ai_does_not_aim_using_hidden_enemy_positions() {
    let (mut a, mut ca, t) = local_impossible(24);
    a.manual = false;
    a.units[1].visible = false;
    let mut b = a.clone();
    b.units[1].pos = Point {
        x: 900000,
        y: 900000,
    };
    let mut cb = ca.clone();
    assert_eq!(
        serde_json::to_value(tick(&mut a, &mut ca, t + 1)).unwrap(),
        serde_json::to_value(tick(&mut b, &mut cb, t + 1)).unwrap()
    );
}

#[test]
fn ai_does_not_fabricate_global_impossible_without_set_or_serve() {
    let (mut f, mut c) = (
        frame(),
        Core {
            pace: 100,
            white_storm: true,
            transformed: true,
            thunder_earned: true,
            ..Core::default()
        },
    );
    f.manual = false;
    f.profile = [100; 13];
    f.ultimate = false;
    f.skill2 = false;
    f.units[1].pos = Point {
        x: 900000,
        y: 300000,
    };
    let mut began = false;
    for tick_at in 0..200 {
        tick(&mut f, &mut c, tick_at);
        assert!(c
            .plans
            .iter()
            .all(|p| !p.actions.contains(&ActionKind::Ultimate) && [0, 1, 3, 5].contains(&p.bars)));
        if matches!(c.phase, Phase::Impossible { .. }) {
            began = true;
            break;
        }
    }
    assert!(!began);
    assert_eq!(c.cds[3], 0);
    assert!(c.serve.is_none());
    assert_eq!(c.ultimate_until, 0);
}

#[test]
fn early_charge_warns_the_final_overpressure_radius_before_speed_crosses_300() {
    let (mut f, mut c, t) = impossible();
    assert!(Core::kmh_tenths(c.last_speed) < 3000);
    f.aim = Some(Point {
        x: 800000,
        y: 600000,
    });
    let out = tick(&mut f, &mut c, t + 1);
    assert!(Core::kmh_tenths(c.last_speed) < 3000);
    assert!(out.iter().any(|e| matches!(
        e,
        Effect::Warning {
            public: true,
            locked: false,
            radius: 144000,
            ..
        }
    )));
}
#[test]
fn every_teammate_burst_resolves_to_the_same_ninety_tick_exhaustion() {
    for n in [1u8, 3, 5] {
        let (mut f, mut c) = (
            frame(),
            Core {
                pace: 100,
                ..Core::default()
            },
        );
        arm(&mut f, &mut c, 0, n);
        action(&mut f, &mut c, 1, ActionKind::AllySet);
        tick(&mut f, &mut c, 12);
        assert_eq!(c.exhaustion_until, 0);
        tick(&mut f, &mut c, 13);
        assert_eq!(c.exhaustion_until, 103);
        assert!(c.burst_play.is_none());
        tick(&mut f, &mut c, 14);
        assert_eq!(c.exhaustion_until, 103);
    }
}

mod revision_32 {
    use super::*;

    fn endpoint_fixture() -> (Frame, Core) {
        let mut f = frame();
        f.steer = Point::default();
        let mut c = Core::default();
        tick(&mut f, &mut c, 0);
        let origin = f.units[0].pos;
        c.ball = Ball::Flying {
            serial: 99,
            origin,
            pos: origin,
            to: Point {
                x: 700000,
                y: 300000,
            },
            amount: 1000,
            basic: false,
            speed: 100000,
            caught: 0,
            height: 10600,
            black: false,
            thunder: true,
            burst: true,
            white: true,
            serve: false,
            impossible: true,
            power: 100,
            radius: 80000,
            overpressure: false,
            hit: Default::default(),
        };
        (f, c)
    }

    #[test]
    fn blast_falloff_is_exact_before_mitigation_with_boundary_clamping() {
        for (distance, expected) in [(0, 1000), (40000, 625), (80000, 250), (90000, 250)] {
            assert_eq!(blast_damage(1000, distance, 80000), expected);
        }
        assert_eq!(blast_damage(10398, 72000, 144000), 6498);
        assert_eq!(blast_damage(1, 80000, 80000), 0);
    }

    #[test]
    fn no_line_damage_and_endpoint_packets_falloff_and_bank_are_delivered_once() {
        let (mut f, mut c) = endpoint_fixture();
        f.units[1].pos = Point {
            x: 220000,
            y: 300000,
        };
        for (id, x) in [
            (4, 700000),
            (5, 740000),
            (6, 780000),
            (7, 790000),
            (8, 790001),
        ] {
            f.units.push(unit(id, 1, x, 300000));
        }
        if let Ball::Flying { caught, .. } = &mut c.ball {
            *caught = 582;
        }
        for t in 1..6 {
            assert!(!tick(&mut f, &mut c, t)
                .iter()
                .any(|e| matches!(e, Effect::Damage { .. })));
        }
        let out = tick(&mut f, &mut c, 6);
        let got = out
            .iter()
            .filter_map(|e| {
                if let Effect::Damage { target, amount, .. } = e {
                    Some((*target, *amount))
                } else {
                    None
                }
            })
            .collect::<std::collections::BTreeMap<_, _>>();
        assert_eq!(
            got,
            std::collections::BTreeMap::from([(4, 1582), (5, 625), (6, 250), (7, 250)])
        );
        assert_eq!(
            out.iter()
                .filter(|e| matches!(e, Effect::KnockUp { .. }))
                .count(),
            4
        );
        assert!(matches!(c.ball, Ball::Returning { .. }));
        assert!(c.storm.is_empty());
        assert!(c.craters.iter().all(|cr| cr.slow == 30 && cr.until == 246));
        assert!(!tick(&mut f, &mut c, 7)
            .iter()
            .any(|e| matches!(e, Effect::Damage { .. } | Effect::KnockUp { .. })));
    }

    #[test]
    fn blast_ignores_allies_and_invulnerable_targets_and_respects_control_immunity() {
        let (mut f, mut c) = endpoint_fixture();
        f.units[1].pos = Point {
            x: 700000,
            y: 300000,
        };
        f.units[1].cc_immune = true;
        f.units[2].pos = f.units[1].pos;
        let mut invulnerable = unit(4, 1, 700000, 300000);
        invulnerable.damageable = false;
        f.units.push(invulnerable);
        let mut events = vec![];
        for t in 1..=6 {
            events.extend(tick(&mut f, &mut c, t));
        }
        assert!(events.iter().any(|e| matches!(
            e,
            Effect::Damage {
                target: 2,
                amount: 1000,
                ..
            }
        )));
        assert!(!events.iter().any(|e| matches!(
            e,
            Effect::Damage { target: 3 | 4, .. } | Effect::KnockUp { target: 2, .. }
        )));
    }

    #[test]
    fn receive_tie_uses_entity_identity_not_input_order_and_is_added_after_falloff() {
        let (mut f, mut c) = endpoint_fixture();
        f.units.retain(|u| u.id == 1);
        f.units.push(unit(9, 1, 740000, 300000));
        f.units.push(unit(4, 1, 740000, 300000));
        if let Ball::Flying { caught, .. } = &mut c.ball {
            *caught = 100;
        }
        let mut damage = vec![];
        for t in 1..=6 {
            for e in tick(&mut f, &mut c, t) {
                if let Effect::Damage { target, amount, .. } = e {
                    damage.push((target, amount));
                }
            }
        }
        assert_eq!(damage, vec![(4, 725), (9, 625)]);
    }

    #[test]
    fn nonserve_contact_height_gate_is_actual_height_with_an_inclusive_10m_boundary() {
        for h in [9999, 10000, 10001] {
            let (mut f, mut c, _) = shot_fixture(h, AttackTier::WhiteStorm, 5, 0);
            let p = f.units[0].pos;
            c.phase = Phase::Air {
                start: 26,
                origin: p,
                aim: p,
                max_height: h,
                perfect: true,
                enhanced: true,
                set_bars: 5,
                spike_bars: 0,
                white_before: true,
                attempted: false,
                intended: 24,
                hold_start: None,
                hold_ticks: 0,
                serve: false,
                white: false,
            };
            c.ball = Ball::Toss {
                start: 26,
                origin: p,
                to: p,
                peak: h + 100,
            };
            action(&mut f, &mut c, 50, ActionKind::Spike);
            assert_eq!(
                matches!(c.phase, Phase::Impossible { .. }),
                h >= 10000,
                "height {h}"
            );
        }
    }

    #[test]
    fn ordinary_serve_accepts_sideways_and_backwards_aim() {
        for aim in [
            Point {
                x: 50000,
                y: 300000,
            },
            Point {
                x: 150000,
                y: 700000,
            },
        ] {
            let (mut f, mut c, _) = shot_fixture(4800, AttackTier::Normal, 0, 0);
            let p = f.units[0].pos;
            c.serve = Some(Serve {
                origin: p,
                aim: Point {
                    x: 900000,
                    y: 300000,
                },
                until: 2000,
                committed: false,
            });
            c.ultimate_until = 2000;
            c.phase = Phase::Air {
                start: 26,
                origin: p,
                aim: p,
                max_height: 4800,
                perfect: false,
                enhanced: false,
                set_bars: 0,
                spike_bars: 0,
                white_before: false,
                attempted: false,
                intended: 24,
                hold_start: None,
                hold_ticks: 0,
                serve: true,
                white: false,
            };
            c.ball = Ball::Toss {
                start: 26,
                origin: p,
                to: p,
                peak: 4900,
            };
            f.tick = 50;
            f.action = Some(Action {
                kind: ActionKind::Spike,
                target: None,
                aim,
            });
            c.step(&f);
            assert!(
                matches!(c.ball,Ball::Flying{to,serve:true,..}if to==aim),
                "{}",
                c.rejection
            );
        }
    }

    #[test]
    fn storm_patches_warn_before_slowing_without_damage_or_knockup() {
        let (mut f, mut c, t) = impossible();
        assert_eq!(c.storm.len(), 1);
        let patch = c.storm[0].clone();
        assert_eq!(patch.active_at, t + 24);
        assert_eq!(patch.until, t + 60);
        f.units[1].pos = patch.pos;
        let before = tick(&mut f, &mut c, t + 23);
        assert!(!before
            .iter()
            .any(|e| matches!(e, Effect::Slow { target: 2, .. })));
        let active = tick(&mut f, &mut c, t + 24);
        assert!(active.iter().any(|e| matches!(
            e,
            Effect::Slow {
                target: 2,
                percent: 15,
                ticks: 2
            }
        )));
        assert!(!active
            .iter()
            .any(|e| matches!(e, Effect::Damage { .. } | Effect::KnockUp { .. })));
    }

    #[test]
    fn storm_overlap_selects_strongest_slow_once_and_immune_units_are_excluded() {
        let (mut f, mut c, t) = impossible();
        let p = c.storm[0].pos;
        f.units[1].pos = p;
        c.craters.push(Crater {
            pos: p,
            radius: 12000,
            until: t + 200,
            slow: 30,
            tier: AttackTier::WhiteStorm,
            impossible: false,
        });
        let out = tick(&mut f, &mut c, t + 24);
        let slow = out
            .iter()
            .filter_map(|e| {
                if let Effect::Slow {
                    target: 2, percent, ..
                } = e
                {
                    Some(*percent)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        assert_eq!(slow, vec![30]);
        f.units[1].cc_immune = true;
        assert!(!tick(&mut f, &mut c, t + 25)
            .iter()
            .any(|e| matches!(e, Effect::Slow { target: 2, .. })));
    }

    #[test]
    fn storm_retargeting_preserves_warned_centres_and_never_exceeds_ten() {
        let (mut f, mut c, t) = impossible();
        let original = c.storm.clone();
        f.aim = Some(Point {
            x: 150000,
            y: 800000,
        });
        for age in 1..300 {
            tick(&mut f, &mut c, t + age);
            assert!(c.storm.len() <= 10);
            for old in &original {
                if let Some(now) = c.storm.iter().find(|p| p.serial == old.serial) {
                    assert_eq!(now, old);
                }
            }
        }
        assert!(c
            .storm
            .iter()
            .all(|p| p.pos.distance(f.units[1].pos) <= 9000));
    }

    #[test]
    fn storm_does_not_use_hidden_positions_or_consume_combat_serials() {
        let (mut a, mut ca, t) = impossible();
        let mut b = a.clone();
        let mut cb = ca.clone();
        a.units[1].visible = false;
        b.units[1].visible = false;
        b.units[1].pos = Point { x: 40000, y: 40000 };
        let serial = ca.serial;
        for age in 1..=100 {
            tick(&mut a, &mut ca, t + age);
            tick(&mut b, &mut cb, t + age);
            assert_eq!(ca.storm, cb.storm);
        }
        assert_eq!(ca.serial, serial);
        assert_eq!(cb.serial, serial);
    }

    #[test]
    fn storm_points_are_legal_and_local_to_the_current_realm() {
        let (mut f, mut c, t) = impossible();
        f.bounds = Rect {
            min: Point {
                x: 100000,
                y: 250000,
            },
            max: Point {
                x: 400000,
                y: 350000,
            },
        };
        f.aim = Some(Point {
            x: 300000,
            y: 300000,
        });
        // Existing warned work is cleared by a genuine realm transition.
        f.realm = Some(3);
        tick(&mut f, &mut c, t + 1);
        assert!(c.storm.is_empty());
        assert!(c.presentation.is_empty());
    }

    #[test]
    fn storm_is_removed_on_interrupt_death_and_impact_without_cleanup_damage() {
        for death in [false, true] {
            let (mut f, mut c, t) = impossible();
            let ids = c.storm.iter().map(|p| p.serial).collect::<Vec<_>>();
            if death {
                f.units[0].hp = 0;
            } else {
                f.interrupted = true;
            }
            let out = tick(&mut f, &mut c, t + 1);
            assert!(c.storm.is_empty());
            for id in ids {
                assert!(out
                    .iter()
                    .any(|e| matches!(e,Effect::ClearWarning{serial}if *serial==id)));
            }
            assert!(!out
                .iter()
                .any(|e| matches!(e, Effect::Damage { .. } | Effect::KnockUp { .. })));
        }
    }

    #[test]
    fn storm_remains_live_in_flight_and_trajectory_is_explicitly_harmless() {
        let (mut f, mut c, t) = impossible();
        f.aim = Some(Point {
            x: 900000,
            y: 300000,
        });
        let warning = tick(&mut f, &mut c, t + 1);
        assert!(warning
            .iter()
            .any(|e| matches!(e, Effect::LineWarning { harmless: true, .. })));
        for age in 2..=300 {
            tick(&mut f, &mut c, t + age);
        }
        assert!(matches!(
            c.ball,
            Ball::Flying {
                impossible: true,
                ..
            }
        ));
        assert!(!c.storm.is_empty());
        for age in 301..=305 {
            tick(&mut f, &mut c, t + age);
            assert!(!c.storm.is_empty());
        }
        for age in 306..=340 {
            tick(&mut f, &mut c, t + age);
        }
        assert!(c.storm.is_empty());
    }

    #[test]
    fn one_large_contact_frame_lasts_six_ticks_without_a_landing_frame() {
        let (mut f, mut c, t) = impossible();
        for age in 1..=300 {
            tick(&mut f, &mut c, t + age);
        }
        let cue = c.presentation[0].clone();
        assert_eq!(cue.phase, "release");
        assert_eq!(cue.start, t + 300);
        for age in 301..306 {
            tick(&mut f, &mut c, t + age);
            assert_eq!(c.presentation[0], cue);
        }
        tick(&mut f, &mut c, t + 306);
        assert!(c.presentation.is_empty());
        for age in 307..400 {
            tick(&mut f, &mut c, t + age);
            assert!(c.presentation.is_empty());
        }
        assert_eq!(c.last_power, 670);
    }

    #[test]
    fn external_position_changes_cannot_move_the_charge_origin_or_extend_its_range() {
        let (mut f, mut c, t) = local_impossible(24);
        let origin = match c.phase {
            Phase::Impossible { origin, .. } => origin,
            _ => panic!(),
        };
        f.units[0].pos.x += 50000;
        f.aim = Some(Point {
            x: 950000,
            y: 300000,
        });
        tick(&mut f, &mut c, t + 1);
        assert_eq!(f.units[0].pos, origin);
        assert!(matches!(c.phase,Phase::Impossible{aim,..}if origin.distance(aim)<=400000));
    }
    #[test]
    fn storm_follows_visible_champions_not_aim_and_leaves_fixed_warnings() {
        let (mut a, mut ca, t) = impossible();
        let mut b = a.clone();
        let mut cb = ca.clone();
        a.aim = Some(Point { x: 10000, y: 10000 });
        b.aim = Some(Point {
            x: 800000,
            y: 800000,
        });
        let original = ca.storm[0].clone();
        for age in 1..=30 {
            tick(&mut a, &mut ca, t + age);
            tick(&mut b, &mut cb, t + age);
            assert_eq!(ca.storm, cb.storm);
        }
        assert_eq!(ca.storm[0], original);
        assert!(ca.storm.iter().all(|p| p.target == 2 && p.radius == 18000));
    }
    #[test]
    fn five_champions_are_marked_and_large_rosters_rotate_within_ten_live_patches() {
        let (mut f, mut c, t) = impossible();
        for id in 4..16 {
            f.units
                .push(unit(id, 1, 250000 + id as i64 * 10000, 350000));
        }
        let mut seen = std::collections::BTreeSet::new();
        for age in 1..180 {
            tick(&mut f, &mut c, t + age);
            assert!(c.storm.len() <= 10);
            for p in &c.storm {
                seen.insert(p.target);
            }
        }
        assert_eq!(seen.len(), 13);
        for id in 4..16 {
            let u = f.units.iter_mut().find(|u| u.id == id).unwrap();
            u.visible = false;
        }
        for age in 180..260 {
            tick(&mut f, &mut c, t + age);
        }
        assert!(c.storm.iter().all(|p| p.target == 2));
    }
    #[test]
    fn local_storm_excludes_minions_allies_and_unreachable_champions() {
        let (mut f, mut c, t) = local_impossible(24);
        let mut minion = unit(4, 1, 240000, 300000);
        minion.champion = false;
        f.units.push(minion);
        f.units.push(unit(5, 0, 240000, 300000));
        f.units.push(unit(6, 1, 900000, 900000));
        for age in 1..=60 {
            tick(&mut f, &mut c, t + age);
        }
        assert!(c.storm.iter().all(|p| p.target == 2));
        let mut hidden = f.clone();
        let mut copy = c.clone();
        f.units[1].visible = false;
        hidden.units[1].visible = false;
        hidden.units[1].pos = Point {
            x: 800000,
            y: 300000,
        };
        for age in 61..100 {
            tick(&mut f, &mut c, t + age);
            tick(&mut hidden, &mut copy, t + age);
            assert_eq!(c.storm, copy.storm);
        }
    }
    #[test]
    fn five_second_charge_strengthens_surrounding_aura_without_growing_the_ball() {
        let (mut f, mut c, t) = impossible();
        let initial = c.charge_presentation.clone().unwrap();
        assert_eq!(initial.stage, 0);
        assert_eq!(initial.bolts.len(), 2);
        tick(&mut f, &mut c, t + 299);
        let final_frame = c.charge_presentation.clone().unwrap();
        assert_eq!(final_frame.ball, initial.ball);
        assert_eq!(final_frame.height, initial.height);
        assert_eq!(final_frame.stage, 6);
        assert_eq!(final_frame.bolts.len(), 8);
        assert!(final_frame.ring_radius > initial.ring_radius);
        tick(&mut f, &mut c, t + 300);
        assert!(c.charge_presentation.is_none());
    }
    #[test]
    fn crater_style_follows_attack_not_highest_body_and_fragments_fit_footprint() {
        for (tier, expected) in [
            (AttackTier::Normal, "normal"),
            (AttackTier::Thunder, "yellow"),
            (AttackTier::BlackThunder, "black"),
            (AttackTier::WhiteStorm, "white"),
        ] {
            assert_eq!(crater_style(tier, false), expected);
            assert_eq!(crater_style(tier, true), "impossible");
            let c = Crater {
                pos: Point {
                    x: 100000,
                    y: 100000,
                },
                radius: 80000,
                until: 99,
                slow: 20,
                tier,
                impossible: false,
            };
            assert!(crater_fragments(&c)
                .iter()
                .all(|p| p.distance(c.pos) <= 60000));
        }
    }
}
