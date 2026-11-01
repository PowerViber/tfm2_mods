use crate::rally_core::*;
fn unit(id: usize, team: usize, x: i64, y: i64) -> Unit {
    Unit {
        id,
        team,
        pos: Point { x, y },
        velocity: Point::default(),
        hp: 1600,
        max_hp: 1600,
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
        incoming_verified: true,
        support_verified: true,
        steer: Point { x: 1, y: 0 },
        ..Frame::default()
    }
}
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
    for e in &out {
        if let Effect::Move { pos } = e {
            f.units[0].pos = *pos
        }
    }
    out
}
fn tick(f: &mut Frame, c: &mut Core, t: u64) -> Vec<Effect> {
    f.tick = t;
    f.action = None;
    let out = c.step(f);
    for e in &out {
        if let Effect::Move { pos } = e {
            f.units[0].pos = *pos
        }
    }
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
        awakened: false,
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
        awakened: false,
        white: false,
        serve: false,
    };
    assert!(!tick(&mut f, &mut c, 31)
        .iter()
        .any(|e| matches!(e, Effect::Damage { target: 2, .. })));
}
#[test]
fn empowered_next_black_thunder_reaches_7_5_metres_with_the_same_finisher_damage() {
    let (mut f, mut c) = (
        frame(),
        Core {
            transformed: true,
            charges: 3,
            flow: 100,
            ..Core::default()
        },
    );
    act(&mut f, &mut c, 0, ActionKind::BlackThunder);
    tick(&mut f, &mut c, 24);
    act(&mut f, &mut c, 25, ActionKind::SelfSet);
    tick(&mut f, &mut c, 37);
    act(&mut f, &mut c, 38, ActionKind::Approach);
    assert_eq!(
        c.ball,
        Ball::Held,
        "Sprinting does not toss or consume the ball"
    );
    for t in 39..62 {
        tick(&mut f, &mut c, t);
    }
    act(&mut f, &mut c, 62, ActionKind::Takeoff);
    for t in 63..86 {
        tick(&mut f, &mut c, t);
    }
    let out = act(&mut f, &mut c, 86, ActionKind::Spike);
    assert!(matches!(
        c.ball,
        Ball::Flying {
            height: 7500,
            black: true,
            amount: 1115,
            ..
        }
    ));
    assert!(out
        .iter()
        .any(|e| matches!(e, Effect::Warning { radius: 65000, .. })));
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
        awakened: false,
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
fn complete_manual_black_set_and_support_sequence_ignores_all_profile_biases() {
    fn run(profile: [i64; 13]) -> Vec<String> {
        let (mut f, mut c) = (
            frame(),
            Core {
                charges: 3,
                flow: 100,
                ..Core::default()
            },
        );
        f.profile = profile;
        let mut trace = vec![];
        let mut hits = vec![];
        for t in 0..451 {
            f.tick = t;
            f.hits = std::mem::take(&mut hits);
            f.basics = if t == 200 {
                vec![Basic {
                    serial: 99,
                    target: 2,
                }]
            } else {
                vec![]
            };
            f.action = match t {
                0 => Some(ActionKind::BlackThunder),
                25 => Some(ActionKind::SelfSet),
                38 => Some(ActionKind::Approach),
                62 => Some(ActionKind::Takeoff),
                86 => Some(ActionKind::Spike),
                130 => Some(ActionKind::Setter),
                250 => Some(ActionKind::AllySet),
                340 => Some(ActionKind::Approach),
                364 => Some(ActionKind::Takeoff),
                388 => Some(ActionKind::Spike),
                _ => None,
            }
            .map(|kind| Action {
                kind,
                target: Some(if kind == ActionKind::AllySet { 3 } else { 2 }),
                aim: Point {
                    x: 400000,
                    y: 300000,
                },
            });
            if t == 450 {
                f.units[0].hp = 0;
            }
            let out = c.step(&f);
            for e in &out {
                match e {
                    Effect::Move { pos } => f.units[0].pos = *pos,
                    Effect::Damage {
                        serial,
                        target,
                        amount,
                        ..
                    } => hits.push(Hit {
                        serial: *serial,
                        target: *target,
                        delivered: *amount > 0,
                        champion: true,
                    }),
                    _ => {}
                }
            }
            trace.push(serde_json::to_string(&(&c, &out)).unwrap());
        }
        trace
    }
    let reference = run([50; 13]);
    for i in 0..13 {
        for value in [20, 80] {
            let mut p = [50; 13];
            p[i] = value;
            assert_eq!(
                run(p),
                reference,
                "Manual mechanics changed for stat {i} at {value}"
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
fn black_power_up_changes_only_active_role_strength_and_keeps_receive_cap() {
    let (mut f, mut c) = (
        frame(),
        Core {
            transformed: true,
            charges: 3,
            role: Role::Receiver,
            ..Core::default()
        },
    );
    assert_eq!(c.reduction(0), 41);
    c.ultimate_until = 1000;
    assert_eq!(
        c.reduction(0),
        41,
        "Serve does not recreate the retired Receiver ultimate"
    );
    c.awakened = true;
    assert_eq!(c.reduction(0), 41);
    c.receive_start = Some(0);
    assert_eq!(c.reduction(1), 95);
    c.role = Role::Setter;
    c.awakened = false;
    c.ultimate_until = 0;
    let out = tick(&mut f, &mut c, 1);
    assert!(out.iter().any(|e|matches!(e,Effect::Buff{buff:Buff{channel,attack:21,speed:18,reduce:21,..},remove:false}if channel=="aura")));
    assert_eq!(f.units[0].attack, 104, "No general Attack buff");
    c.role = Role::Spiker;
    assert_eq!(
        c.reduction(2),
        35,
        "Only the still-active ordinary Receive guard remains"
    );
    c.receive_start = None;
    assert_eq!(c.reduction(2), 0);
}
#[test]
fn empowered_spiker_reaches_6_9_metres_and_earns_black_without_arming_the_boost() {
    let (mut f, mut c) = (
        frame(),
        Core {
            transformed: true,
            charges: 3,
            ..Core::default()
        },
    );
    act(&mut f, &mut c, 0, ActionKind::SelfSet);
    tick(&mut f, &mut c, 12);
    assert!(c.helper.is_some());
    let clone = c.helper.unwrap().0;
    assert_ne!(clone, f.units[0].pos);
    act(&mut f, &mut c, 13, ActionKind::Approach);
    assert_eq!(c.ball, Ball::Held);
    for t in 14..37 {
        tick(&mut f, &mut c, t);
    }
    act(&mut f, &mut c, 37, ActionKind::Takeoff);
    for t in 38..61 {
        tick(&mut f, &mut c, t);
    }
    act(&mut f, &mut c, 61, ActionKind::Spike);
    assert!(matches!(
        c.ball,
        Ball::Flying {
            height: 6900,
            amount: 1115,
            black: true,
            ..
        }
    ));
    assert_eq!(c.height(61), 6900);
    let contact = f.units[0].pos;
    for t in 62..67 {
        tick(&mut f, &mut c, t);
        assert_eq!(f.units[0].pos, contact);
        assert_eq!(c.height(t), 6900);
    }
    tick(&mut f, &mut c, 70);
    assert_eq!(c.height(70), 6900);
    tick(&mut f, &mut c, 71);
    assert!(c.height(71) < 6900);
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
fn delivered_hit_receipts_award_flow_but_cannot_manufacture_a_permanent_form() {
    let (mut f, mut c) = (frame(), Core::default());
    tick(&mut f, &mut c, 0);
    for (serial, champion) in [(1, false), (2, true)] {
        c.pending_hits.insert(serial, (true, false, true, true));
        f.hits = vec![Hit {
            serial,
            target: 2,
            delivered: true,
            champion,
        }];
        let out = tick(&mut f, &mut c, serial);
        assert!(!c.transformed && !c.white_storm);
        assert!(!out
            .iter()
            .any(|e| matches!(e, Effect::Transform | Effect::WhiteTransform)));
    }
    assert_eq!(c.flow, 15, "Only confirmed champion contribution pays Flow");
}

#[test]
fn misses_and_ordinary_thunder_or_unbound_receipts_never_transform() {
    let (mut f, mut c) = (frame(), Core::default());
    tick(&mut f, &mut c, 0);
    for (i, bound, thunder, delivered) in [
        (1, true, true, false),
        (2, true, false, true),
        (3, false, true, true),
    ] {
        if bound {
            c.pending_hits.insert(i, (true, false, thunder, false));
        }
        f.hits = vec![Hit {
            serial: i,
            target: 2,
            delivered,
            champion: true,
        }];
        let out = tick(&mut f, &mut c, i);
        assert!(!c.transformed);
        assert!(!out.iter().any(|e| matches!(e, Effect::Transform)));
    }
}
#[test]
fn duplicate_or_out_of_order_hit_receipts_award_only_the_bound_action() {
    let (mut f, mut c) = (frame(), Core::default());
    tick(&mut f, &mut c, 0);
    c.pending_hits.insert(5, (true, false, true, false));
    c.pending_hits.insert(9, (true, false, true, false));
    for (t, serial) in [(1, 9), (2, 5), (3, 9), (4, 5)] {
        f.hits = vec![Hit {
            serial,
            target: 2,
            delivered: true,
            champion: true,
        }];
        tick(&mut f, &mut c, t);
    }
    assert_eq!(c.flow, 30);
    assert!(!c.transformed, "Receipts cannot grant a contact-based form");
}
#[test]
fn independent_charge_recovery_is_sequential_alive_only() {
    let (mut f, mut c) = (frame(), Core::default());
    tick(&mut f, &mut c, 0);
    tick(&mut f, &mut c, 359);
    assert_eq!(c.charges, 0);
    tick(&mut f, &mut c, 360);
    assert_eq!(c.charges, 1);
    tick(&mut f, &mut c, 1080);
    assert_eq!(c.charges, 3);
    f.units[0].hp = 0;
    tick(&mut f, &mut c, 2161);
    assert_eq!((c.charges, c.flow, c.caught), (0, 0, 0));
    tick(&mut f, &mut c, 9999);
    assert_eq!(c.charges, 0);
    f.units[0].hp = 1000;
    tick(&mut f, &mut c, 10000);
    assert_eq!(c.charge_progress, 0);
}
#[test]
fn charges_only_change_active_passive() {
    let mut c = Core {
        charges: 3,
        ..Core::default()
    };
    assert_eq!(c.reduction(0), 0);
    c.role = Role::Receiver;
    assert_eq!(c.reduction(0), 26);
    c.receive_start = Some(0);
    assert_eq!(c.reduction(1), 95);
    c.receive_start = None;
    c.ultimate_until = 480;
    assert_eq!(c.reduction(1), 26);
    c.awakened = true;
    assert_eq!(c.reduction(1), 26);
}
#[test]
fn receive_shield_and_own_reduction_end_exactly() {
    let (mut f, mut c) = (frame(), Core::default());
    let out = act(&mut f, &mut c, 0, ActionKind::Receive);
    assert!(out.iter().any(|e| matches!(
        e,
        Effect::Shield {
            amount: 131,
            ticks: 60
        }
    )));
    assert_eq!(c.reduction(17), 35);
    assert_eq!(
        c.reduction(18),
        20,
        "Receive selects Receiver; the passive remains after the guard"
    );
    assert_eq!(c.cds[0], 480);
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
                amount: 513,
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
fn black_thunder_spends_resources_on_its_own_cooldown() {
    for flow in [60, 100] {
        let (mut f, mut c) = (
            frame(),
            Core {
                charges: 3,
                flow,
                ..Core::default()
            },
        );
        act(&mut f, &mut c, 0, ActionKind::BlackThunder);
        assert_eq!((c.charges, c.flow, c.cds[4], c.cds[3]), (0, 0, 1800, 0));
        tick(&mut f, &mut c, 24);
        assert_eq!(c.black_until, 504);
        assert!(c.awakened);
    }
}
#[test]
fn black_thunder_near_misses_do_not_spend_resources() {
    for (charges, flow, ult) in [(2, 100, true), (3, 59, true), (3, 100, false)] {
        let (mut f, mut c) = (
            frame(),
            Core {
                charges,
                flow,
                ..Core::default()
            },
        );
        f.ultimate = ult;
        act(&mut f, &mut c, 0, ActionKind::BlackThunder);
        assert_eq!((c.charges, c.flow, c.cds[3]), (charges, flow, 0));
    }
}
#[test]
fn black_six_metres_requires_perfect_and_multiplier_does_not_stack() {
    let (mut f, mut c) = (
        frame(),
        Core {
            charges: 3,
            flow: 100,
            ..Core::default()
        },
    );
    act(&mut f, &mut c, 0, ActionKind::BlackThunder);
    tick(&mut f, &mut c, 24);
    act(&mut f, &mut c, 25, ActionKind::SelfSet);
    tick(&mut f, &mut c, 37);
    act(&mut f, &mut c, 38, ActionKind::Approach);
    for t in 39..62 {
        tick(&mut f, &mut c, t);
    }
    act(&mut f, &mut c, 62, ActionKind::Takeoff);
    for t in 63..86 {
        tick(&mut f, &mut c, t);
    }
    act(&mut f, &mut c, 86, ActionKind::Spike);
    assert!(
        matches!(
            c.ball,
            Ball::Flying {
                black: true,
                height: 6000,
                amount: 1115,
                ..
            }
        ),
        "{:?} / {}",
        c.ball,
        c.rejection
    );
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
        awakened: false,
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
fn reliable_caught_power_is_capped_and_other_receipts_fail_closed() {
    let (mut f, mut c) = (
        frame(),
        Core {
            role: Role::Receiver,
            ..Core::default()
        },
    );
    f.receipts.push(Receipt {
        serial: 1,
        source: 2,
        delivered: 40,
        reduction: 20,
        champion: true,
        verified: true,
        reflected: false,
    });
    c.step(&f);
    assert_eq!(c.caught, 50);
    f.receipts[0].serial = 2;
    f.receipts[0].delivered = 10000;
    tick(&mut f, &mut c, 1);
    assert_eq!(c.caught, 290);
    f.incoming_verified = false;
    f.receipts[0].serial = 3;
    tick(&mut f, &mut c, 2);
    assert_eq!(c.caught, 290);
    f.incoming_verified = true;
    f.receipts[0].serial = 4;
    f.receipts[0].reflected = true;
    tick(&mut f, &mut c, 3);
    assert_eq!(c.caught, 290);
}
#[test]
fn support_flow_requires_verified_real_contribution_once() {
    let (mut f, mut c) = (frame(), Core::default());
    f.action = Some(Action {
        kind: ActionKind::AllySet,
        target: Some(3),
        aim: f.units[2].pos,
    });
    c.step(&f);
    tick(&mut f, &mut c, 12);
    assert_eq!(c.flow, 0);
    f.support_receipts.push(SupportReceipt {
        serial: 1,
        ally: 3,
        champion: true,
        positive: true,
        verified: true,
    });
    tick(&mut f, &mut c, 13);
    assert_eq!(c.flow, 15);
    f.support_receipts[0].serial = 2;
    tick(&mut f, &mut c, 14);
    assert_eq!(c.flow, 15);
}
#[test]
fn native_unverified_support_cannot_award_flow() {
    let (mut f, mut c) = (
        frame(),
        Core {
            support: Some((3, 300, 42)),
            ..Core::default()
        },
    );
    f.support_verified = false;
    f.support_receipts.push(SupportReceipt {
        serial: 1,
        ally: 3,
        champion: true,
        positive: true,
        verified: true,
    });
    c.step(&f);
    assert_eq!(c.flow, 0);
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
fn duplicate_ticks_events_and_clones_are_independent() {
    let (mut f, mut c) = (frame(), Core::default());
    act(&mut f, &mut c, 0, ActionKind::Receive);
    let before = serde_json::to_string(&c).unwrap();
    assert!(c.step(&f).is_empty());
    assert_eq!(serde_json::to_string(&c).unwrap(), before);
    let mut other = c.clone();
    tick(&mut f, &mut other, 720);
    assert_eq!(c.charges, 0);
    assert_eq!(other.charges, 2);
}
#[test]
fn manual_mechanics_are_independent_of_all_thirteen_attributes() {
    let mut baseline = None;
    for stat in 0..13 {
        for value in [20, 80] {
            let (mut f, mut c) = (frame(), Core::default());
            f.profile[stat] = value;
            let a = act(&mut f, &mut c, 0, ActionKind::Receive);
            tick(&mut f, &mut c, 12);
            let combat =
                serde_json::to_string(&(a, c.flow, c.charges, c.cds, c.reduction(12))).unwrap();
            if let Some(b) = &baseline {
                assert_eq!(&combat, b)
            } else {
                baseline = Some(combat)
            }
        }
    }
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
fn thirty_six_seeded_development_and_holdout_sessions() {
    for seed in 0..36 {
        let (mut f, mut c) = (frame(), Core::default());
        f.seed = seed;
        f.manual = false;
        for t in 0..1200 {
            tick(&mut f, &mut c, t);
            assert!((0..=100).contains(&c.flow));
            assert!((0..=3).contains(&c.charges));
            assert!(f.legal(f.units[0].pos, f.units[0].radius));
            assert!(c.plans.len() <= 8);
            assert!(c.plans.iter().all(|p| p.actions.len() <= 3));
        }
        assert!(c.cds[2] > 0 || c.cds[1] > 0, "seed {seed}");
    }
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
        assert_eq!(c.cds, [0; 5]);
        assert!(c.rejection.contains("Roles follow"));
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
fn black_thunder_and_serve_have_independent_acceptance_and_cooldowns() {
    let (mut f, mut c) = (
        frame(),
        Core {
            charges: 3,
            flow: 100,
            ..Core::default()
        },
    );
    act(&mut f, &mut c, 0, ActionKind::BlackThunder);
    tick(&mut f, &mut c, 24);
    assert_eq!(c.cds[4], 1800);
    assert_eq!(c.cds[3], 0);
    act(&mut f, &mut c, 25, ActionKind::Ultimate);
    assert_eq!(c.cds[3], 4525);
    assert_eq!(c.cds[4], 1800);
    tick(&mut f, &mut c, 49);
    assert!(c.serve.is_some());
    assert!(c.black_until > 49);
}
fn prepare_serve(transformed: bool, black: bool) -> (Frame, Core, u64) {
    let (mut f, mut c) = (
        frame(),
        Core {
            transformed,
            charges: 3,
            flow: 100,
            ..Core::default()
        },
    );
    let mut t = 0;
    if black {
        act(&mut f, &mut c, t, ActionKind::BlackThunder);
        tick(&mut f, &mut c, 24);
        t = 25;
    }
    act(&mut f, &mut c, t, ActionKind::SelfSet);
    tick(&mut f, &mut c, t + 12);
    act(&mut f, &mut c, t + 13, ActionKind::Ultimate);
    tick(&mut f, &mut c, t + 37);
    for at in t + 38..t + 61 {
        tick(&mut f, &mut c, at);
    }
    act(&mut f, &mut c, t + 61, ActionKind::Takeoff);
    (f, c, t + 61)
}
#[test]
fn serve_applies_seventy_percent_after_all_height_bonuses_once() {
    for (tier, black, expected) in [
        (false, false, 9180),
        (true, false, 11730),
        (false, true, 10200),
        (true, true, 12750),
    ] {
        let (f, c, _) = prepare_serve(tier, black);
        assert!(
            matches!(c.phase,Phase::Air{serve:true,max_height,white,..}if max_height==expected && !white),
            "{}",
            serde_json::to_string(&c.phase).unwrap()
        );
        assert_eq!(
            f.units[0].pos,
            Point {
                x: 150000,
                y: 300000
            },
            "Serve stays at the committed origin"
        );
    }
}
#[test]
fn white_blink_and_permanent_form_require_contact_but_not_a_later_hit() {
    let (mut f, mut c, t) = prepare_serve(true, false);
    for at in t + 1..t + 24 {
        let out = tick(&mut f, &mut c, at);
        assert!(c.contact_height > 2000);
        assert!(!out
            .iter()
            .any(|e| matches!(e,Effect::Visual{name,..}if name=="white_blink")));
        assert!(!c.white_storm);
    }
    let out = act(&mut f, &mut c, t + 24, ActionKind::Spike);
    assert!(matches!(
        c.ball,
        Ball::Flying {
            white: true,
            height: 11730,
            ..
        }
    ));
    assert!(c.white_storm && c.transformed);
    assert!(out.iter().any(|e| matches!(e, Effect::WhiteTransform)));
    assert!(out
        .iter()
        .any(|e| matches!(e,Effect::Visual{name,..}if name=="white_blink")));
    assert!(!out.iter().any(|e| matches!(e, Effect::Shield { .. })));
    // Miss every target; contact alone earned the form.
    f.units.retain(|u| u.id == 1);
    for at in t + 25..t + 150 {
        assert!(!tick(&mut f, &mut c, at)
            .iter()
            .any(|e| matches!(e, Effect::WhiteTransform)));
    }
    assert!(c.white_storm);
    f.units[0].hp = 0;
    tick(&mut f, &mut c, t + 150);
    assert!(!c.white_storm && !c.transformed && !c.thunder_earned);
    assert!(c.serve.is_none());
}

#[test]
fn serve_aim_is_fixed_and_outside_sector_rejection_does_not_spend_contact() {
    let (mut f, mut c, t) = prepare_serve(false, false);
    for at in t + 1..t + 24 {
        tick(&mut f, &mut c, at);
    }
    f.tick = t + 24;
    f.action = Some(Action {
        kind: ActionKind::Spike,
        target: None,
        aim: Point {
            x: 100000,
            y: 300000,
        },
    });
    c.step(&f);
    assert!(c.rejection.contains("outside"));
    assert!(matches!(
        c.phase,
        Phase::Air {
            attempted: false,
            ..
        }
    ));
    assert!(c.serve.as_ref().unwrap().aim.x > c.serve.as_ref().unwrap().origin.x);
    let out = act(&mut f, &mut c, t + 25, ActionKind::Spike);
    assert!(out.iter().any(|e| matches!(e, Effect::Warning { .. })));
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
        awakened: true,
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
    for t in [2, 120, 240] {
        let out = tick(&mut f, &mut c, t);
        assert!(out.iter().any(|e| matches!(
            e,
            Effect::Slow {
                target: 2,
                percent: 30,
                ..
            }
        )));
        assert!(!out
            .iter()
            .any(|e| matches!(e, Effect::KnockUp { .. } | Effect::Damage { .. })));
    }
    assert!(!tick(&mut f, &mut c, 241)
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
        awakened: true,
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
fn white_role_bonuses_replace_black_bonuses_and_keep_the_receive_cap() {
    let (mut f, mut c) = (
        frame(),
        Core {
            transformed: true,
            white_storm: true,
            role: Role::Receiver,
            charges: 3,
            ..Core::default()
        },
    );
    assert_eq!(c.reduction(0), 46);
    c.receive_start = Some(0);
    assert_eq!(c.reduction(1), 95);
    c.role = Role::Setter;
    tick(&mut f, &mut c, 0);
    assert_eq!(c.buffs["3:aura"].attack, 26);
    assert_eq!(c.buffs["3:aura"].speed, 23);
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
fn serve_and_finisher_end_remask_without_removing_permanent_power_or_resetting_cooldowns() {
    for black in [false, true] {
        let (mut f, mut c, t) = prepare_serve(true, black);
        let cds = c.cds;
        for at in t + 1..t + 49 {
            tick(&mut f, &mut c, at);
        }
        assert_eq!(c.phase, Phase::Idle);
        assert!(c.serve.is_none());
        assert!(!c.awakened);
        assert_eq!(c.black_until, 0);
        assert!(c.transformed);
        assert_eq!(c.cds, cds);
    }
}
#[test]
fn serve_ai_keeps_a_valid_target_in_the_fixed_sector_when_a_nearer_enemy_is_outside_it() {
    let (mut f, mut c, t) = prepare_serve(true, false);
    f.units.push(unit(4, 1, 149000, 300000));
    f.manual = false;
    for at in t + 1..=t + 24 {
        tick(&mut f, &mut c, at);
    }
    assert!(
        matches!(c.ball,Ball::Flying{to,..}if to.x>f.units[0].pos.x),
        "{}",
        c.rejection
    );
    assert!(!c.rejection.contains("outside"));
}
#[test]
fn all_serve_variants_commit_the_larger_warning_and_ground_radius() {
    assert_eq!(impact_radius(false, false, false), 45000);
    assert_eq!(impact_radius(false, true, false), 65000);
    for (tier, black) in [(false, false), (false, true), (true, false), (true, true)] {
        let (mut f, mut c, t) = prepare_serve(tier, black);
        for at in t + 1..t + 24 {
            tick(&mut f, &mut c, at);
        }
        let out = act(&mut f, &mut c, t + 24, ActionKind::Spike);
        assert!(out
            .iter()
            .any(|e| matches!(e, Effect::Warning { radius: 90000, .. })));
        assert!(matches!(c.ball, Ball::Flying { serve: true, .. }));
    }
}
#[test]
fn serve_crater_size_survives_the_firing_window_and_includes_its_exact_boundary() {
    let (mut f, mut c) = (frame(), Core::default());
    tick(&mut f, &mut c, 0);
    let p = Point {
        x: 250000,
        y: 300000,
    };
    f.units[1].pos = Point {
        x: 350000,
        y: 300000,
    };
    f.units[2].team = 1;
    f.units[2].pos = Point {
        x: 350001,
        y: 300000,
    };
    c.ball = Ball::Flying {
        serial: 1,
        origin: p,
        pos: p,
        to: p,
        amount: 513,
        basic: false,
        speed: 1000,
        caught: 0,
        height: 9180,
        black: false,
        thunder: true,
        awakened: true,
        white: false,
        serve: true,
    };
    assert!(
        c.serve.is_none(),
        "Released ball retains its identity after Serve closes"
    );
    let out = tick(&mut f, &mut c, 1);
    assert!(out.iter().any(|e| matches!(
        e,
        Effect::Damage {
            target: 2,
            amount: 513,
            ..
        }
    )));
    assert!(!out
        .iter()
        .any(|e| matches!(e, Effect::Damage { target: 3, .. })));
    assert!(out.iter().any(|e| matches!(
        e,
        Effect::KnockUp {
            target: 2,
            ticks: 30
        }
    )));
    assert_eq!(c.craters[0].radius, 90000);
    let next = tick(&mut f, &mut c, 2);
    assert!(next.iter().any(|e| matches!(
        e,
        Effect::Slow {
            target: 2,
            percent: 30,
            ..
        }
    )));
    assert!(!next.iter().any(|e| matches!(
        e,
        Effect::Slow { target: 3, .. } | Effect::KnockUp { .. } | Effect::Damage { .. }
    )));
}
#[test]
fn serve_endpoint_and_ai_use_the_extended_sector_without_extending_regular_spikes() {
    let (mut f, mut c, t) = prepare_serve(false, false);
    for at in t + 1..t + 24 {
        tick(&mut f, &mut c, at);
    }
    f.tick = t + 24;
    f.action = Some(Action {
        kind: ActionKind::Spike,
        target: None,
        aim: Point {
            x: 950000,
            y: 300000,
        },
    });
    c.step(&f);
    assert!(matches!(c.ball,Ball::Flying{to,..}if to.x==710000));
    let (mut f, mut c) = (frame(), Core::default());
    f.manual = false;
    f.units[1].pos = Point {
        x: 650000,
        y: 300000,
    };
    let mut served = false;
    for t in 0..100 {
        tick(&mut f, &mut c, t);
        served |= c.serve.is_some();
    }
    assert!(
        served,
        "AI must prepare and use a useful visible Serve beyond regular spike range"
    );
    assert_eq!(crate::rally_core::data::SPIKE_RANGE, 240000);
}

// A timed-contact fixture isolates exact integer thresholds from takeoff bonuses.
// Full accepted Set/run-up/Serve sequences are covered separately above.
fn contact_fixture(height: i64, permanent: AttackTier) -> (Frame, Core, Vec<Effect>) {
    let mut f = frame();
    f.steer = Point::default();
    let p = f.units[0].pos;
    let mut c = Core::default();
    tick(&mut f, &mut c, 0);
    c.thunder_earned = permanent >= AttackTier::Thunder;
    c.transformed = permanent >= AttackTier::BlackThunder;
    c.white_storm = permanent >= AttackTier::WhiteStorm;
    c.phase = Phase::Air {
        start: 1,
        origin: p,
        aim: p,
        max_height: height,
        perfect: false,
        enhanced: false,
        black: false,
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
        peak: height + 100,
    };
    let out = act(&mut f, &mut c, 25, ActionKind::Spike);
    assert!(matches!(c.ball, Ball::Flying { .. }), "{}", c.rejection);
    (f, c, out)
}
#[test]
fn height_alone_classifies_every_attack_at_exact_integer_boundaries() {
    for (h, expected, mult) in [
        (3999, AttackTier::Normal, 100),
        (4000, AttackTier::Thunder, 180),
        (5999, AttackTier::Thunder, 230),
        (6000, AttackTier::BlackThunder, 500),
        (9999, AttackTier::BlackThunder, 500),
        (10000, AttackTier::WhiteStorm, 600),
    ] {
        let (_, c, _) = contact_fixture(h, AttackTier::Normal);
        assert_eq!(c.last_spike_tier, expected);
        assert_eq!(c.permanent_tier(), expected);
        assert!(
            matches!(c.ball,Ball::Flying{height,amount,black,white,thunder,..}
            if height==h && amount==(70+104*120/100)*mult/100
            && black==(expected==AttackTier::BlackThunder)
            && white==(expected==AttackTier::WhiteStorm)
            && thunder==(expected>=AttackTier::Thunder))
        );
    }
}
#[test]
fn first_qualifying_contact_earns_speed_before_release_and_does_not_change_its_height() {
    for (h, tier) in [
        (4800, AttackTier::Thunder),
        (6000, AttackTier::BlackThunder),
        (10000, AttackTier::WhiteStorm),
    ] {
        let (_, c, out) = contact_fixture(h, AttackTier::Normal);
        assert!(
            matches!(c.ball,Ball::Flying{speed,height,..}if speed==tier_speed(tier)&&height==h)
        );
        assert!(matches!(c.phase,Phase::Air{max_height,..}if max_height==h));
        assert_eq!(
            out.iter()
                .filter(|e| matches!(e, Effect::Transform | Effect::WhiteTransform))
                .count(),
            usize::from(h >= 6000)
        );
    }
}
#[test]
fn white_form_releasing_a_low_contact_keeps_appearance_role_power_and_white_speed() {
    let (_, mut c, out) = contact_fixture(3500, AttackTier::WhiteStorm);
    assert!(c.white_storm && c.transformed);
    assert_eq!(c.permanent_tier(), AttackTier::WhiteStorm);
    assert_eq!(c.last_spike_tier, AttackTier::Normal);
    assert!(matches!(
        c.ball,
        Ball::Flying {
            speed: 14000,
            black: false,
            white: false,
            thunder: false,
            amount: 194,
            ..
        }
    ));
    assert!(!out
        .iter()
        .any(|e| matches!(e, Effect::Transform | Effect::WhiteTransform)));
    c.role = Role::Receiver;
    assert_eq!(c.reduction(25), 40);
}
#[test]
fn black_form_can_make_yellow_attacks_without_downgrading_or_using_black_damage() {
    let (_, c, _) = contact_fixture(5000, AttackTier::BlackThunder);
    assert!(c.transformed && !c.white_storm);
    assert!(matches!(
        c.ball,
        Ball::Flying {
            speed: 10000,
            amount: 446,
            black: false,
            thunder: true,
            white: false,
            ..
        }
    ));
}
#[test]
fn contact_form_persists_after_a_miss_but_does_not_stack_again_or_survive_death() {
    let (mut f, mut c, _) = contact_fixture(6000, AttackTier::Normal);
    f.units.retain(|u| u.id == 1);
    for t in 26..180 {
        tick(&mut f, &mut c, t);
    }
    assert!(c.transformed);
    assert_eq!(c.ball, Ball::Held, "A complete miss returns without pickup");
    let (_, again, out) = contact_fixture(6000, c.permanent_tier());
    assert_eq!(again.permanent_tier(), AttackTier::BlackThunder);
    assert!(!out.iter().any(|e| matches!(e, Effect::Transform)));
    f.units[0].hp = 0;
    tick(&mut f, &mut c, 180);
    assert_eq!(c.permanent_tier(), AttackTier::Normal);
}
#[test]
fn failed_contact_never_earns_a_form_or_speed_bonus() {
    let (mut f, mut c, _) = contact_fixture(3500, AttackTier::Normal);
    let p = f.units[0].pos;
    c.phase = Phase::Air {
        start: 30,
        origin: p,
        aim: p,
        max_height: 11000,
        perfect: true,
        enhanced: true,
        black: true,
        attempted: false,
        intended: 24,
        hold_start: None,
        hold_ticks: 0,
        serve: true,
        white: false,
    };
    c.serve = Some(Serve {
        origin: p,
        aim: Point {
            x: p.x + 100000,
            y: p.y,
        },
        until: 500,
        committed: true,
    });
    c.ball = Ball::Toss {
        start: 30,
        origin: Point {
            x: 800000,
            y: 800000,
        },
        to: Point {
            x: 800000,
            y: 800000,
        },
        peak: 11000,
    };
    act(&mut f, &mut c, 54, ActionKind::Spike);
    assert!(c.rejection.contains("Contact missed"));
    assert_eq!(c.permanent_tier(), AttackTier::Normal);
}
#[test]
fn every_released_spike_returns_after_unit_wall_or_range_without_extra_damage() {
    for wall in [false, true] {
        let (mut f, mut c, _) = contact_fixture(4800, AttackTier::Normal);
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
        let mut returned = false;
        for t in 26..240 {
            let out = tick(&mut f, &mut c, t);
            returned |= matches!(c.ball, Ball::Returning { .. });
            assert!(!matches!(c.ball, Ball::Grounded { .. }));
            assert!(!out.iter().any(|e| matches!(e, Effect::Damage { .. })));
        }
        assert!(returned);
        assert_eq!(c.ball, Ball::Held);
    }
    let (mut f, mut c, _) = contact_fixture(4800, AttackTier::Normal);
    let mut hits = 0;
    for t in 26..180 {
        hits += tick(&mut f, &mut c, t)
            .iter()
            .filter(|e| matches!(e, Effect::Damage { target: 2, .. }))
            .count();
    }
    assert_eq!(hits, 1);
    assert_eq!(c.ball, Ball::Held);
}
#[test]
fn ordinary_volley_misses_still_ground_and_death_cancels_return() {
    let (mut f, mut c) = (frame(), Core::default());
    tick(&mut f, &mut c, 0);
    let p = f.units[0].pos;
    c.ball = Ball::Flying {
        serial: 1,
        origin: p,
        pos: p,
        to: Point {
            x: p.x + 40000,
            y: p.y,
        },
        amount: 78,
        basic: true,
        speed: 5000,
        caught: 0,
        height: 2000,
        black: false,
        thunder: false,
        awakened: false,
        white: false,
        serve: false,
    };
    for t in 1..=8 { tick(&mut f, &mut c, t); }
    assert!(matches!(c.ball, Ball::Grounded { .. }));
    c.ball = Ball::Returning {
        pos: Point {
            x: 500000,
            y: 500000,
        },
    };
    f.units[0].hp = 0;
    let out = tick(&mut f, &mut c, 9);
    assert_eq!(c.ball, Ball::Absent);
    assert!(!out.iter().any(|e| matches!(e, Effect::Damage { .. })));
}
#[test]
fn later_gauge_cycles_allow_perfect_takeoff_without_restarting_the_skill() {
    for takeoff in [60, 96, 132, 168] {
        let (mut f, mut c) = (frame(), Core::default());
        act(&mut f, &mut c, 0, ActionKind::Approach);
        for t in 1..takeoff {
            tick(&mut f, &mut c, t);
        }
        act(&mut f, &mut c, takeoff, ActionKind::Takeoff);
        assert!(
            matches!(
                c.phase,
                Phase::Air {
                    perfect: true,
                    max_height: 4800,
                    ..
                }
            ),
            "{takeoff}"
        );
        assert_eq!(c.cds[2], 300);
    }
}
#[test]
fn earlier_cycle_displacement_cannot_pay_for_a_stationary_later_perfect_window() {
    let (mut f, mut c) = (frame(), Core::default());
    act(&mut f, &mut c, 0, ActionKind::Approach);
    for t in 1..36 {
        tick(&mut f, &mut c, t);
    }
    f.steer = Point::default();
    for t in 36..60 {
        tick(&mut f, &mut c, t);
    }
    act(&mut f, &mut c, 60, ActionKind::Takeoff);
    assert!(matches!(c.phase, Phase::Air { perfect: false, .. }));
}
#[test]
fn run_up_lasts_three_seconds_at_double_speed_with_no_forced_jump_or_ball_toss() {
    let (mut f, mut c) = (frame(), Core::default());
    let start = f.units[0].pos;
    act(&mut f, &mut c, 0, ActionKind::Approach);
    for t in 1..180 {
        tick(&mut f, &mut c, t);
    }
    assert_eq!(f.units[0].pos.x - start.x, 179 * 2100);
    assert!(matches!(c.phase, Phase::Approach { .. }));
    assert_eq!(c.ball, Ball::Held);
    tick(&mut f, &mut c, 180);
    assert_eq!(c.phase, Phase::Idle);
}
#[test]
fn white_capable_jumps_allow_yellow_and_black_early_contacts_before_white() {
    for (age, expected) in [
        (3, AttackTier::Thunder),
        (6, AttackTier::BlackThunder),
        (24, AttackTier::WhiteStorm),
    ] {
        let (mut f, mut c, t) = prepare_serve(true, false);
        for at in t + 1..t + age {
            tick(&mut f, &mut c, at);
        }
        let out = act(&mut f, &mut c, t + age, ActionKind::Spike);
        assert!(
            matches!(c.ball, Ball::Flying { .. }),
            "age {age}: {}",
            c.rejection
        );
        assert_eq!(
            c.last_spike_tier, expected,
            "age {age}, height {}",
            c.last_spike_height
        );
        assert_eq!(c.white_storm, expected == AttackTier::WhiteStorm);
        assert_eq!(
            out.iter()
                .any(|e| matches!(e,Effect::Visual{name,..}if name=="white_blink")),
            expected == AttackTier::WhiteStorm
        );
    }
}
#[test]
fn serve_pierces_map_and_construction_walls_but_keeps_first_unit_collision_and_bounds() {
    for serve in [false, true] {
        let (mut f, mut c, _) = contact_fixture(4800, AttackTier::Normal);
        // Walls represent both baked map rectangles and sampled construction segments.
        f.blocks = vec![
            Rect {
                min: Point {
                    x: 170000,
                    y: 200000,
                },
                max: Point {
                    x: 175000,
                    y: 400000,
                },
            },
            Rect {
                min: Point {
                    x: 185000,
                    y: 280000,
                },
                max: Point {
                    x: 190000,
                    y: 320000,
                },
            },
        ];
        f.units.retain(|u| u.id != 3);
        f.units[1].pos.x = 300000;
        if let Ball::Flying { serve: s, .. } = &mut c.ball {
            *s = serve;
        }
        let mut hits = 0;
        for t in 26..160 {
            hits += tick(&mut f, &mut c, t)
                .iter()
                .filter(|e| matches!(e, Effect::Damage { target: 2, .. }))
                .count();
        }
        assert_eq!(hits, usize::from(serve));
        assert_eq!(c.ball, Ball::Held);
    }
    let (mut f, mut c, _) = contact_fixture(4800, AttackTier::Normal);
    f.units.retain(|u| u.id == 1);
    if let Ball::Flying { serve, to, .. } = &mut c.ball {
        *serve = true;
        *to = Point {
            x: 1200000,
            y: 300000,
        };
    }
    for t in 26..400 {
        tick(&mut f, &mut c, t);
        if let Ball::Flying { pos, .. } = c.ball {
            assert!(f.bounds.contains(pos, 3000));
        }
    }
    assert_eq!(c.ball, Ball::Held);
}
#[test]
fn opening_serve_emits_one_full_sector_request_without_revealing_core_enemies() {
    let (mut f, mut c) = (frame(), Core::default());
    f.units[1].visible = false;
    act(&mut f, &mut c, 0, ActionKind::Ultimate);
    assert!(c.serve.is_none());
    let out = tick(&mut f, &mut c, 24);
    assert_eq!(
        out.iter()
            .filter(|e| matches!(e, Effect::RevealSector { range: 560000, .. }))
            .count(),
        1
    );
    assert!(
        !f.units[1].visible,
        "Only a host with a supported reveal operation may change visibility"
    );
    for t in 25..48 {
        assert!(!tick(&mut f, &mut c, t)
            .iter()
            .any(|e| matches!(e, Effect::RevealSector { .. })));
    }
    f.interrupted = true;
    tick(&mut f, &mut c, 48);
    assert!(c.serve.is_none());
}
