//! Round 180 level unlocks across the native kits (Reihan: S1 at 1, S2 at 2, S3 at 3, the ult at 5). Each core holds a
//! locked skill "on cooldown" until its level, so the AI and orders skip it; level 0 (Labs, older tests) unlocks all.

macro_rules! frame {
    ($m:ident, $level:expr, $tick:expr) => {
        $m::Frame { tick: $tick, alive: true, manual: true, level: $level, ..Default::default() }
    };
}

/// step one frame at `level` and report which S2/S3 timers are ready (true) through the given readers
macro_rules! check {
    ($m:ident, $s2:expr, $s3:expr) => {{
        let mut out = vec![];
        for level in [1i64, 2, 3, 0] {
            let mut c = $m::Core::default();
            let f = frame!($m, level, 100);
            c.step(&f);
            out.push((level, $s2(&c, 100u64), $s3(&c, 100u64)));
        }
        assert_eq!(out, vec![(1, false, false), (2, true, false), (3, true, true), (0, true, true)], "{}", stringify!($m));
    }};
}

use crate::{akane_core, basalt_core, grimhald_core, kagami_core, kicker_core, kratos_core, thalassa_core, wright_core};

#[test]
fn every_kit_holds_s2_until_level_2_and_s3_until_level_3() {
    check!(akane_core, |c: &akane_core::Core, t| c.ready(akane_core::KUNAI, t), |c: &akane_core::Core, t| c.ready(akane_core::EDGE, t));
    check!(grimhald_core, |c: &grimhald_core::Core, t| c.ready(1, t), |c: &grimhald_core::Core, t| c.ready(2, t));
    check!(thalassa_core, |c: &thalassa_core::Core, t| c.ready(1, t), |c: &thalassa_core::Core, t| c.ready(2, t));
    check!(wright_core, |c: &wright_core::Core, t| c.ready(1, t), |c: &wright_core::Core, t| c.ready(2, t));
    check!(kratos_core, |c: &kratos_core::Core, t| c.ready(kratos_core::RAMPAGE, t), |c: &kratos_core::Core, t| c.ready(kratos_core::GUARD, t) && c.ready(kratos_core::DASH, t));
    check!(kicker_core, |c: &kicker_core::Core, t| t >= c.rock_ready, |c: &kicker_core::Core, _| c.kicks > 0);
    check!(kagami_core, |c: &kagami_core::Core, t| t >= c.charm_at, |c: &kagami_core::Core, t| t >= c.mend_at);
    check!(basalt_core, |c: &basalt_core::Core, t| t >= c.s2_at, |c: &basalt_core::Core, t| t >= c.s3_at);
}

#[test]
fn kicker_gets_both_kick_charges_the_moment_s3_unlocks() {
    let mut c = kicker_core::Core::default();
    c.step(&frame!(kicker_core, 2, 100));
    assert_eq!(c.kicks, 0);
    c.step(&frame!(kicker_core, 3, 101));
    assert_eq!(c.kicks, crate::kicker_data::KICK_CHARGES);
}

#[test]
fn contender_rejects_locked_orders_and_only_considers_unlocked_skills_at_level_one() {
    use crate::contender_shadow_core::*;
    let foe = Unit { id: 10, team: 2, pos: Point::new(430_000, 500_000), hp: 1000, max_hp: 1000, radius: 3000, champion: true, tower: false, visible: true, targetable: true, farmable: false };
    let base = |level: i64, tick: u64, order: Option<Order>| Frame {
        tick, owner: 1, team: 1, alive: true, pos: Point::new(400_000, 500_000), hp: 1000, max_hp: 1000, attack: 100, ult_unlocked: true,
        units: vec![foe.clone()], terrain: vec![false; (CELLS * CELLS) as usize], profile: [50; 13], manual: order.is_some(), order, level,
        ..Frame::default()
    };
    let mut c = Core::default();
    c.step(&base(1, 1, Some(Order::S2)));
    assert_eq!(c.rejection, "s2: locked until level 2");
    c.step(&base(2, 2, Some(Order::S2)));
    assert!(c.flurrying(), "Flurry unlocks in S2");
    let mut c = Core::default();
    c.step(&base(4, 3, Some(Order::Ult { aim: foe.pos })));
    assert!(c.shadow_link.is_none(), "the ult waits for level 5 even off cooldown");
    c.step(&base(5, 4, Some(Order::Ult { aim: foe.pos })));
    assert!(c.shadow_link.is_some());
    // A level-one AI cannot spend Flurry or the ultimate.
    let mut c = Core::default();
    for t in 1..400 {
        let mut f = base(1, t, None);
        f.units.push(Unit { id: 11, pos: Point::new(435_000, 505_000), ..foe.clone() });
        c.step(&f);
        assert_eq!(c.flurry_charges, crate::contender_shadow_data::FLURRY_MAX_CHARGES);
        assert!(c.shadow_link.is_none());
    }
}

#[test]
fn scribble_spells_open_by_size_with_level() {
    use crate::scribble::tier_open;
    assert!(tier_open(1, 2) && !tier_open(1, 3));
    assert!(tier_open(2, 3) && !tier_open(2, 4));
    assert!(tier_open(3, 4) && !tier_open(4, 5));
    assert!(tier_open(5, 6) && tier_open(0, 6));
}
