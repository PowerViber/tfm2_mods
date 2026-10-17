use super::*;
fn unit(id: usize, team: usize, x: i64) -> Unit {
    Unit {
        id,
        team,
        x,
        y: 100000,
        hp: 1000,
        max_hp: 1000,
        ap: 100,
        visible: true,
        targetable: true,
        champion: true,
        farmable: false,
        controlled: false,
        vx: 0,
        vy: 0,
    }
}
fn ready() -> Core {
    Core {
        stark_requested: true,
        stark_until: 720,
        ..Core::default()
    }
}
fn run(
    c: &mut Core,
    t: usize,
    units: &[Unit],
    s: Option<Stark>,
    wall: &dyn Fn(i64, i64) -> bool,
) -> Vec<Cmd> {
    c.step(&Input {
        tick: t,
        owner: &units[0],
        units,
        ult_unlocked: true,
        spell_unlocked: true,
        barrier_unlocked: true,
        automatic: false,
        action: None,
        stark: s,
        wall,
    })
}
#[test]
fn leap_moves_real_carrier_warns_then_hits_once_and_preserves_melee_clock() {
    let mut c = ready();
    let mut s = Stark::at(100000, 100000);
    let u = [unit(1, 0, 100000), unit(2, 1, 160000)];
    let mut hits = vec![];
    let mut poses = 0;
    for t in 0..=50 {
        let out = run(&mut c, t, &u, Some(s), &|_, _| false);
        for cmd in out {
            match cmd {
                Cmd::Damage { target, amount } => hits.push((t, target, amount)),
                Cmd::MoveStark { x, y } => {
                    s.x = x;
                    s.y = y;
                }
                Cmd::StarkWarning { x, y, ticks } => {
                    assert_eq!((t, x, y, ticks), (0, 160000, 100000, 50))
                }
                Cmd::StarkPose { height, .. } => {
                    assert!(height >= 0 && height <= 36000);
                    poses += 1;
                }
                Cmd::Control {
                    target,
                    airborne,
                    slow,
                    ticks,
                } => assert_eq!((t, target, airborne, slow, ticks), (50, 2, 50, -35, 105)),
                _ => {}
            }
        }
    }
    assert_eq!((s.x, s.y), (160000, 100000));
    assert_eq!(hits, vec![(50, 2, 105)]);
    assert_eq!(poses, 25);
    assert_eq!(c.jumped, vec![2]);
    assert_eq!(c.next_leap, 120);
    assert!(c.leap.is_none());
    let out = run(&mut c, 51, &u, Some(s), &|_, _| false);
    assert!(out.contains(&Cmd::Damage {
        target: 2,
        amount: 85
    }));
    assert_eq!(c.next_stark, 141);
}
#[test]
fn escaped_hidden_dead_and_allied_units_do_not_take_landing_damage() {
    for mode in 0..4 {
        let mut c = ready();
        let mut u = vec![unit(1, 0, 100000), unit(2, 1, 160000)];
        let s = Some(Stark::at(100000, 100000));
        run(&mut c, 0, &u, s, &|_, _| false);
        match mode {
            0 => u[1].x = 240000,
            1 => u[1].visible = false,
            2 => u[1].hp = 0,
            _ => u[1].team = 0,
        };
        let out = run(&mut c, 50, &u, s, &|_, _| false);
        assert!(!out
            .iter()
            .any(|a| matches!(a, Cmd::Damage { .. } | Cmd::Control { .. })));
    }
}
#[test]
fn death_interrupt_expiry_and_carrier_replacement_cancel_without_impact() {
    for mode in 0..6 {
        let mut c = ready();
        let mut u = vec![unit(1, 0, 100000), unit(2, 1, 160000)];
        let mut s = Some(Stark::at(100000, 100000));
        run(&mut c, 0, &u, s, &|_, _| false);
        match mode {
            0 => s = None,
            1 => s.as_mut().unwrap().hp = 0,
            2 => s.as_mut().unwrap().interrupted = true,
            3 => u[0].hp = 0,
            4 => c.stark_until = 50,
            _ => s.as_mut().unwrap().id = 1000,
        };
        let out = run(&mut c, 50, &u, s, &|_, _| false);
        assert!(out.contains(&Cmd::StarkCancel));
        assert!(!out
            .iter()
            .any(|a| matches!(a, Cmd::Damage { .. } | Cmd::StarkLanding { .. })));
        assert!(c.leap.is_none());
    }
}
#[test]
fn walls_map_leash_and_remaining_lifetime_gate_the_jump() {
    for (x, t, blocked) in [
        (960000, 0, false),
        (200000, 0, false),
        (160000, 700, false),
        (160000, 0, true),
    ] {
        let mut c = ready();
        let u = [unit(1, 0, 100000), unit(2, 1, x)];
        let out = run(&mut c, t, &u, Some(Stark::at(100000, 100000)), &|_, _| {
            blocked
        });
        assert!(!out.iter().any(|a| matches!(a, Cmd::StarkWarning { .. })));
        assert!(c.jumped.is_empty());
    }
    let mut c = ready();
    let u = [unit(1, 0, 100000), unit(2, 1, 160000)];
    run(&mut c, 0, &u, Some(Stark::at(100000, 100000)), &|_, _| {
        false
    });
    let out = run(&mut c, 25, &u, Some(Stark::at(130000, 100000)), &|_, _| {
        true
    });
    assert!(out.contains(&Cmd::StarkCancel));
}
#[test]
fn cooldown_and_identity_limit_allow_only_one_committed_jump_per_champion() {
    let mut c = ready();
    let u = [unit(1, 0, 100000), unit(2, 1, 160000), unit(3, 1, 170000)];
    let mut s = Stark::at(100000, 100000);
    let mut jumps = vec![];
    for t in 0..400 {
        let out = run(&mut c, t, &u, Some(s), &|_, _| false);
        for cmd in out {
            match cmd {
                Cmd::MoveStark { x, y } => {
                    s.x = x;
                    s.y = y;
                }
                Cmd::StarkWarning { .. } => jumps.push(t),
                _ => {}
            }
        }
    }
    assert_eq!(jumps, vec![0, 120]);
    assert_eq!(c.jumped, vec![2, 3]);
}
#[test]
fn landing_hits_incidental_nonchampions_once_and_outer_ring_only_slows() {
    let mut c = ready();
    let mut u = vec![
        unit(1, 0, 100000),
        unit(2, 1, 160000),
        unit(3, 1, 170000),
        unit(4, 1, 190000),
    ];
    u[2].champion = false;
    u.push(u[2].clone());
    run(&mut c, 0, &u, Some(Stark::at(100000, 100000)), &|_, _| {
        false
    });
    let out = run(&mut c, 50, &u, Some(Stark::at(130000, 100000)), &|_, _| {
        false
    });
    assert_eq!(
        out.iter()
            .filter(|a| matches!(a, Cmd::Damage { target: 3, .. }))
            .count(),
        1
    );
    assert!(!out
        .iter()
        .any(|a| matches!(a, Cmd::Damage { target: 4, .. })));
    assert!(out.contains(&Cmd::Control {
        target: 4,
        airborne: 0,
        slow: -35,
        ticks: 105
    }));
}
