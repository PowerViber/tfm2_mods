use super::*;

fn staged(c: &mut Core, f: &mut Frame, offset: i64) -> u64 {
    let id = 1000 + c.metrics.launches;
    let centre = f.units[0].pos.sub(Point::new(offset, 0));
    c.packs.push(Pack {
        id,
        pos: centre,
        born: f.tick,
        until: f.tick + 1000,
        hp: PACK_HP,
        ready: true,
        attachment: Attachment {
            surface: None,
            kind: "ground".into(),
            point: centre,
            normal: Point::default(),
            offset: Point::default(),
        },
    });
    id
}
fn launch(c: &mut Core, f: &mut Frame, offset: i64) -> Vec<Cmd> {
    let id = staged(c, f, offset);
    tick(c, f, Some(Action::Detonate { id }))
}
#[test]
fn dome_knots_and_edge_are_exact_and_the_curve_is_bounded() {
    for (d, h, z, t) in [
        (0, 0, 24000, 39),
        (2800, 0, 24000, 39),
        (14000, 200000, 20000, 48),
        (28000, 133333, 13333, 32),
        (56000, 0, 0, 0),
        (57000, 0, 0, 0),
    ] {
        let p = launch_profile(d);
        assert_eq!((p.horizontal_speed, p.peak, p.ticks), (h, z, t));
    }
    for d in 0..=57000 {
        let p = launch_profile(d);
        assert!((0..=200000).contains(&p.horizontal_speed));
        assert!(p.peak <= 24000 && p.ticks <= 48);
    }
    assert_eq!(AIR_ACCEL, 600000);
}
#[test]
fn centre_goes_up_without_inventing_horizontal_motion() {
    let (mut c, mut f) = (ready(), frame());
    launch(&mut c, &mut f, 0);
    assert_eq!(c.flight.as_ref().unwrap().velocity, Point::default());
    let start = f.units[0].pos;
    advance(&mut c, &mut f, 10);
    assert_eq!(f.units[0].pos, start);
    assert!(c.flight.as_ref().unwrap().height(f.tick) > 0);
}
#[test]
fn underfoot_drop_follows_preparation_then_stays_at_its_release_point() {
    let (mut c, mut f) = (ready(), frame());
    tick(&mut c, &mut f, Some(Action::DropPack));
    f.units[0].pos = Point::new(308000, 307000);
    advance(&mut c, &mut f, PACK_THROW as usize);
    assert_eq!(c.packs[0].attachment.point, f.units[0].pos);
    assert_eq!(c.packs[0].pos, f.units[0].pos);
    let at = c.packs[0].pos;
    f.units[0].pos.x += 5000;
    tick(&mut c, &mut f, None);
    assert_eq!(c.packs[0].pos, at);
}
#[test]
fn grounded_walking_momentum_survives_a_centred_launch() {
    let (mut c, mut f) = (ready(), frame());
    f.units[0].velocity = Point::new(61200, 0);
    launch(&mut c, &mut f, 0);
    assert_eq!(c.flight.as_ref().unwrap().velocity, Point::new(61200, 0));
}
#[test]
fn enlarged_satchel_hits_newly_covered_targets_but_not_beyond_its_boundary() {
    let (mut c, mut f) = (ready(), frame());
    f.tick = 61;
    f.units[1].pos = Point::new(340000, 300000);
    f.units.push(unit(3, 1, 356001, 300000));
    let id = staged(&mut c, &mut f, 0);
    c.packs[0].born = 0;
    let out = tick(&mut c, &mut f, Some(Action::Detonate { id }));
    assert!(out.iter().any(|v| matches!(
        v,
        Cmd::Damage {
            target: 2,
            raw: 141,
            ..
        }
    )));
    assert!(!out
        .iter()
        .any(|v| matches!(v, Cmd::Damage { target: 3, .. })));
    assert!(launch_profile(40000).horizontal_speed > 0);
}
#[test]
fn centre_keeps_existing_velocity_and_second_launch_keeps_height() {
    let (mut c, mut f) = (ready(), frame());
    launch(&mut c, &mut f, 14000);
    advance(&mut c, &mut f, 3);
    let old = c.flight.as_ref().unwrap().clone();
    let expected = old.height(f.tick + 1);
    launch(&mut c, &mut f, 0);
    let v = c.flight.as_ref().unwrap();
    assert_eq!(v.velocity, old.velocity);
    assert_eq!(v.height(f.tick), expected);
    assert_eq!(v.satchel_count, 2);
    assert!(v.until <= v.born + MAX_FLIGHT_TICKS as u64);
}
#[test]
fn sweet_spot_travels_farther_than_old_single_launch_without_steering() {
    let (mut c, mut f) = (ready(), frame());
    let start = f.units[0].pos;
    launch(&mut c, &mut f, 14000);
    assert_eq!(c.flight.as_ref().unwrap().velocity.x, 200000);
    assert_eq!(c.flight.as_ref().unwrap().arc_peak, 20000);
    advance(&mut c, &mut f, 48);
    assert!(distance(start, f.units[0].pos) >= 155000);
    assert!(c.flight.is_none());
    let (mut c, mut f) = (ready(), frame());
    launch(&mut c, &mut f, 56000);
    assert!(c.flight.is_none() && !c.air_protection);
}
#[test]
fn landing_scales_to_visible_champions_and_expires_on_the_exact_tick() {
    for n in 0..=6usize {
        let (mut c, mut f) = (ready(), frame());
        f.units.truncate(1);
        for k in 0..n {
            f.units.push(unit(2 + k, 1, 310000, 300000));
        }
        // These must not increase the duration.
        for (k, kind, visible, hp) in [
            (20, 1, true, 100),
            (21, 5, true, 100),
            (22, 0, false, 100),
            (23, 0, true, 0),
        ] {
            let mut u = unit(k, 1, 300000, 300000);
            u.kind = kind;
            u.visible = visible;
            u.hp = hp;
            f.units.push(u);
        }
        launch(&mut c, &mut f, 0);
        let end = c.flight.as_ref().unwrap().until;
        f.tick = end - 1;
        tick(&mut c, &mut f, None);
        let ticks = landing_protection_ticks(n);
        assert_eq!(ticks, [0, 180, 315, 450, 585, 720, 720][n]);
        assert_eq!(c.air_protection_until, end + ticks);
        assert_eq!(c.air_protection, n > 0);
        for u in f.units.iter_mut().skip(1) {
            u.visible = false;
        }
        if ticks > 0 {
            f.tick = end + ticks - 2;
            tick(&mut c, &mut f, None);
            assert!(c.air_protection);
            let out = tick(&mut c, &mut f, None);
            assert!(!c.air_protection);
            assert_eq!(
                out.iter()
                    .filter(|v| matches!(v, Cmd::AirProtection { active: false }))
                    .count(),
                1
            );
        }
    }
}
#[test]
fn interruption_grants_the_tail_and_death_removes_it_immediately() {
    let (mut c, mut f) = (ready(), frame());
    launch(&mut c, &mut f, 0);
    f.stunned = true;
    tick(&mut c, &mut f, None);
    assert!(c.flight.is_none() && c.air_protection);
    assert_eq!(c.air_protection_until, f.tick + 180);
    assert_eq!(c.protected_damage(100), 85);
    let out = c.died();
    assert_eq!(c.air_protection_until, 0);
    assert!(!c.air_protection);
    assert!(out
        .iter()
        .any(|v| matches!(v, Cmd::AirProtection { active: false })));
}
#[test]
fn a_new_landing_never_stacks_strength_or_shortens_existing_protection() {
    let (mut c, mut f) = (ready(), frame());
    launch(&mut c, &mut f, 0);
    c.flight.as_mut().unwrap().until = f.tick + 1;
    tick(&mut c, &mut f, None);
    let expiry = c.air_protection_until;
    f.units[1].visible = false;
    let out = launch(&mut c, &mut f, 0);
    assert!(!out
        .iter()
        .any(|v| matches!(v, Cmd::AirProtection { active: true })));
    c.flight.as_mut().unwrap().until = f.tick + 1;
    tick(&mut c, &mut f, None);
    assert_eq!(c.air_protection_until, expiry);
    assert_eq!(c.protected_damage(100), 85);
}
#[test]
fn skill_scaling_ignores_ap_and_reads_attack_once() {
    for ap in [0, 65, 999] {
        let (mut c, mut f) = (ready(), frame());
        f.units[0].attack = 65;
        f.units[0].ap = ap;
        let aim = f.units[1].pos;
        tick(&mut c, &mut f, Some(Action::Grenade { aim }));
        f.units[0].attack = 999;
        let out = advance(&mut c, &mut f, 130);
        assert!(out
            .iter()
            .any(|v| matches!(v,Cmd::Damage{target:2,raw:399,source,..}if source=="grenade")));
    }
    assert_eq!(
        crate::valorant_guns::current_basic_raw(Gun::Vandal, 100, 30000),
        100
    );
    assert_eq!(
        crate::valorant_guns::basic_raw(Gun::Vandal, 100, 30000),
        130
    );
}
