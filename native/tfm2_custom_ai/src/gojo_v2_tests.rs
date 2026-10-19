use super::gojo_v2_core::*;

#[test]
fn v2_protocol_refuses_v1_recordings_and_unknown_actions() {
    let mut frame=vec![1,0,1,0,0,2,0,0,7,0,0,0,0,0,-1,1];
    frame.extend([1,0,100000,100000,935,935,70,81,3,1,1,1,0,0,0,0]);
    assert!(super::gojo_v2_protocol::Frame::decode(&frame).is_none());
    frame[0]=2;assert!(super::gojo_v2_protocol::Frame::decode(&frame).is_some());
    frame[4]=99;assert!(super::gojo_v2_protocol::Frame::decode(&frame).is_none());
}
fn units() -> Vec<Fighter> {
    (1..=5)
        .map(|id| Fighter {
            id,
            team: if id == 1 || id == 3 || id == 4 { 0 } else { 1 },
            x: 100000 + if id == 1 { 0 } else { id as i64 * 5000 },
            y: 100000,
            hp: 10000,
            max_hp: 10000,
            ad: 100,
            ap: 100,
            level: 3,
            champion: true,
            visible: true,
            targetable: true,
            immune: false,
            cc_immune: false,
            farmable: false,
            tower: false,
            immovable: false,
            cleansed: false,
        })
        .collect()
}
fn input<'a>(
    tick: usize,
    u: &'a [Fighter],
    action: Option<Action>,
    basics: &'a [(u64, usize)],
) -> Input<'a> {
    Input {
        tick,
        me: &u[0],
        all: u,
        unlocked: [true; 3],
        controlled: false,
        bound: false,
        automatic: false,
        haste: 0,
        ult_haste: 0,
        action,
        basics,
        champion_damage: false,
        shield: None,
        wall: &|_, _| false,
        barrier: &|_, _| false,
        sight: &|_, _| true,
    }
}
fn boot(c: &mut Core, u: &[Fighter]) {
    c.step(&input(0, u, None, &[]));
}
fn tick(c: &mut Core, u: &[Fighter], t: usize) -> Vec<Cmd> {
    c.step(&input(t, u, None, &[]))
}
fn damage(out: &[Cmd]) -> i64 {
    out.iter()
        .filter_map(|c| {
            if let Cmd::Damage { ad, ap, .. } = c {
                Some(ad + ap)
            } else {
                None
            }
        })
        .sum()
}
#[test]
fn eyes_discount_costs_and_double_cooldown_progress() {
    let u = units();
    let mut c = Core::default();
    boot(&mut c, &u);
    let ce = c.ce;
    c.step(&input(1, &u, Some(Action::Blue(2)), &[]));
    assert_eq!(c.ce, ce - 600 - 1);
    assert_eq!(c.strain, 5000);
    tick(&mut c, &u, 301);
    assert!(c.ready[0] <= 301);
}
#[test]
fn fractional_item_haste_is_preserved() {
    let u = units();
    let mut c = Core::default();
    boot(&mut c, &u);
    c.ready[0] = 1000;
    for t in 1..=100 {
        let mut i = input(t, &u, None, &[]);
        i.haste = 13;
        c.step(&i);
    }
    assert_eq!(c.ready[0], 874);
}
#[test]
fn cooldown_progress_slows_when_eyes_exhaust() {
    let u = units();
    let mut c = Core::default();
    boot(&mut c, &u);
    c.ready[0] = 1000;
    tick(&mut c, &u, 100);
    assert_eq!(c.ready[0], 900);
    c.eyes = false;
    c.strain = 200000;
    tick(&mut c, &u, 101);
    assert_eq!(c.ready[0], 900);
}
#[test]
fn impact_penalty_and_amplification_multiply_once() {
    let u = units();
    let mut c = Core::default();
    boot(&mut c, &u);
    let o = c.step(&input(1, &u, None, &[(1, 2)]));
    assert_eq!(damage(&o), 108);
    c.amp_until = 100;
    let o = c.step(&input(2, &u, None, &[(2, 2)]));
    assert_eq!(damage(&o), 129);
    c.eyes = false;
    c.strain = 200000;
    let o = c.step(&input(3, &u, None, &[(3, 2)]));
    assert_eq!(damage(&o), 162);
}
#[test]
fn amplification_expires_at_impact_boundary() {
    let u = units();
    let mut c = Core::default();
    boot(&mut c, &u);
    c.amp_until = 2;
    assert_eq!(damage(&c.step(&input(2, &u, None, &[(1, 2)]))), 108);
}
#[test]
fn overload_recovers_250_to_50_without_rct() {
    let u = units();
    let mut c = Core::default();
    boot(&mut c, &u);
    c.strain = 250000;
    tick(&mut c, &u, 1);
    assert!(!c.eyes);
    c.step(&input(2, &u, Some(Action::Heal), &[]));
    assert!(c.pending.is_none());
    tick(&mut c, &u, 600);
    assert!(!c.eyes);
    tick(&mut c, &u, 601);
    assert!(c.eyes);
    assert!(c.strain <= 50000);
}
#[test]
fn fallback_uses_full_cost_without_delaying_eye_recovery() {
    let u = units();
    let mut c = Core::default();
    boot(&mut c, &u);
    c.eyes = false;
    c.strain = 200000;
    c.last_technique = 0;
    let ce = c.ce;
    c.step(&input(1, &u, Some(Action::Red(2)), &[]));
    assert_eq!(c.ce, ce - 10000);
    assert_eq!(c.last_technique, 0);
    assert!(c.strain < 200000);
}
#[test]
fn maximums_require_flash_eyes_and_strain_headroom() {
    let u = units();
    for (eyes, amp, strain) in [(true, 0, 0), (false, 100, 100000), (true, 100, 180000)] {
        let mut c = Core::default();
        boot(&mut c, &u);
        c.eyes = eyes;
        c.amp_until = amp;
        c.strain = strain;
        let n = c.casts;
        c.step(&input(1, &u, Some(Action::MaxBlue(2)), &[]));
        assert_eq!(c.casts, n);
    }
}
#[test]
fn normal_cast_can_exhaust_eyes_and_still_release() {
    let u = units();
    let mut c = Core::default();
    boot(&mut c, &u);
    c.strain = 246000;
    c.step(&input(1, &u, Some(Action::Blue(2)), &[]));
    assert!(!c.eyes);
    assert!(c.pending.is_some());
    tick(&mut c, &u, 25);
    assert!(!c.shots.is_empty());
}
#[test]
fn fatigue_and_eye_exhaustion_restrictions_union() {
    let mut u = units();
    u[0].hp = 5000;
    let mut c = Core::default();
    boot(&mut c, &u);
    c.burnout = 180;
    c.step(&input(1, &u, Some(Action::Blue(2)), &[]));
    assert!(c.pending.is_none());
    c.step(&input(2, &u, Some(Action::Heal), &[]));
    assert!(c.pending.is_some());
}
#[test]
fn residual_shield_never_refills_on_pause() {
    let u = units();
    let mut c = Core::default();
    boot(&mut c, &u);
    let mut i = input(1, &u, None, &[]);
    i.shield = Some(75);
    c.step(&i);
    c.eyes = false;
    c.strain = 200000;
    tick(&mut c, &u, 2);
    assert!(!c.infinity);
    assert_eq!(c.shield, 75);
    c.strain = 50000;
    tick(&mut c, &u, 3);
    assert!(c.infinity);
    assert_eq!(c.shield, 75);
}
#[test]
fn quick_domain_guest_and_single_residual_stun() {
    let u = units();
    let mut c = Core::default();
    boot(&mut c, &u);
    c.step(&input(1, &u, Some(Action::QuickDomain), &[]));
    assert_eq!(c.domain.as_ref().unwrap().guest, Some(3));
    let o = tick(&mut c, &u, 13);
    assert!(o.iter().any(|v| matches!(
        v,
        Cmd::Seal {
            target: 4,
            ticks: 60
        }
    )));
    assert!(!o.iter().any(|v| matches!(v, Cmd::Seal { target: 3, .. })));
    assert!(!tick(&mut c, &u, 14)
        .iter()
        .any(|v| matches!(v, Cmd::Seal { .. })));
    tick(&mut c, &u, 25);
    assert!(c.domain.is_none());
    assert_eq!(c.burnout, 205);
    assert_eq!(c.power_until, 0);
}
#[test]
fn full_domain_has_smaller_radius_and_maintained_seal() {
    let u = units();
    let mut c = Core::default();
    boot(&mut c, &u);
    c.step(&input(1, &u, Some(Action::Domain), &[]));
    assert_eq!(c.domain.as_ref().unwrap().radius, 64000);
    assert!(!tick(&mut c, &u, 60)
        .iter()
        .any(|v| matches!(v, Cmd::Seal { .. })));
    let o = tick(&mut c, &u, 61);
    assert!(o.iter().any(|v| matches!(
        v,
        Cmd::Seal {
            target: 4,
            ticks: 7
        }
    )));
    assert_eq!(c.domain.as_ref().unwrap().end, 301);
}
#[test]
fn interrupted_domain_commitment_is_not_refunded() {
    let u = units();
    let mut c = Core::default();
    boot(&mut c, &u);
    c.step(&input(1, &u, Some(Action::Domain), &[]));
    let mut i = input(2, &u, None, &[]);
    i.controlled = true;
    c.step(&i);
    assert!(c.domain.is_none());
    assert!(c.ready[7] > 2);
    assert!(c.ce < 99000);
}
#[test]
fn guest_identity_does_not_transfer() {
    let mut u = units();
    let mut c = Core::default();
    boot(&mut c, &u);
    c.step(&input(1, &u, Some(Action::Domain), &[]));
    u[2].hp = 0;
    tick(&mut c, &u, 61);
    assert_eq!(c.domain.as_ref().unwrap().guest, Some(3));
}
fn max_blue(c: &mut Core, u: &[Fighter]) {
    boot(c, u);
    c.amp_until = 480;
    c.step(&input(1, u, Some(Action::MaxBlue(2)), &[]));
    for t in 2..=67 {
        tick(c, u, t);
    }
}
#[test]
fn max_blue_leaves_single_orb_with_no_periodic_damage() {
    let u = units();
    let mut c = Core::default();
    max_blue(&mut c, &u);
    assert!(c.orb.is_some());
    assert_eq!(damage(&tick(&mut c, &u, 68)), 0);
    c.ready[0] = 0;
    c.ready[1] = 0;
    c.step(&input(69, &u, Some(Action::MaxBlue(2)), &[]));
    assert!(c.pending.is_none());
}
#[test]
fn blue_moves_inward_and_around_not_directly_to_centre() {
    let u = units();
    let mut c = Core::default();
    max_blue(&mut c, &u);
    let o = tick(&mut c, &u, 75);
    assert!(o
        .iter()
        .any(|v| matches!(v,Cmd::Move{target:5,y,..} if *y!=100000)));
}
#[test]
fn objectives_and_cc_immune_units_never_spiral() {
    let mut u = units();
    u[1].immovable = true;
    u[4].cc_immune = true;
    let mut c = Core::default();
    max_blue(&mut c, &u);
    assert!(c.spirals.is_empty());
}
#[test]
fn cleanse_ends_spiral_immediately() {
    let mut u = units();
    let mut c = Core::default();
    max_blue(&mut c, &u);
    u[1].cleansed = true;
    let o = tick(&mut c, &u, 68);
    assert!(!o.iter().any(|v| matches!(v, Cmd::Move { target: 2, .. })));
}
#[test]
fn walls_stop_spiral_motion() {
    let u = units();
    let mut c = Core::default();
    max_blue(&mut c, &u);
    let mut i = input(68, &u, None, &[]);
    i.wall = &|_, _| true;
    let o = c.step(&i);
    assert!(!o.iter().any(|v| matches!(v, Cmd::Move { .. })));
}
#[test]
fn orb_entrants_spiral_once_without_damage() {
    let mut u = units();
    u[4].x = 300000;
    let mut c = Core::default();
    max_blue(&mut c, &u);
    u[4].x = 120000;
    assert_eq!(damage(&tick(&mut c, &u, 68)), 0);
    let begin = c.spirals.iter().find(|m| m.target == 5).unwrap().begin;
    tick(&mut c, &u, 69);
    assert_eq!(
        c.spirals.iter().find(|m| m.target == 5).unwrap().begin,
        begin
    );
}
fn orb_caster() -> (Core, Vec<Fighter>) {
    let mut u = units();
    u.retain(|u| u.id == 1);
    let mut c = Core::default();
    boot(&mut c, &u);
    c.amp_until = 480;
    c.orb = Some(Orb {
        point: (140000, 100000),
        end: 360,
        affected: vec![],
    });
    (c, u)
}
#[test]
fn own_max_red_fuses_once_and_cost_is_separate() {
    let (mut c, u) = orb_caster();
    c.step(&input(1, &u, Some(Action::OrbRed), &[]));
    for t in 2..=70 {
        tick(&mut c, &u, t);
    }
    assert!(c.orb.is_none());
    let at = c.fusion.as_ref().unwrap().at;
    let o = tick(&mut c, &u, at);
    assert_eq!(o.iter().filter(|v| matches!(v, Cmd::SelfCost)).count(), 1);
    assert!(c.fusion.is_none());
    assert!(!tick(&mut c, &u, at + 1).contains(&Cmd::SelfCost));
}
#[test]
fn red_bodyblock_prevents_fusion() {
    let (mut c, mut u) = orb_caster();
    let mut e = units()[1].clone();
    e.x = 120000;
    u.push(e);
    c.step(&input(1, &u, Some(Action::OrbRed), &[]));
    for t in 2..=80 {
        tick(&mut c, &u, t);
    }
    assert!(c.orb.is_some());
    assert!(c.fusion.is_none());
}
#[test]
fn ordinary_red_does_not_fuse() {
    let (mut c, mut u) = orb_caster();
    let mut e = units()[1].clone();
    e.x = 150000;
    u.push(e);
    c.step(&input(1, &u, Some(Action::Red(2)), &[]));
    for t in 2..=90 {
        tick(&mut c, &u, t);
    }
    assert!(c.orb.is_some());
    assert!(c.fusion.is_none());
}
#[test]
fn formed_blast_survives_exhaustion_and_uses_impact_scale() {
    let (mut c, mut u) = orb_caster();
    let mut e = units()[1].clone();
    e.x = 140000;
    u.push(e);
    c.fusion = Some(Fusion {
        point: (140000, 100000),
        at: 50,
    });
    c.eyes = false;
    c.strain = 200000;
    let o = tick(&mut c, &u, 50);
    assert_eq!(damage(&o), 768);
    assert!(o.contains(&Cmd::SelfCost));
}
#[test]
fn death_cancels_fields_and_pending_self_cost() {
    let (mut c, mut u) = orb_caster();
    c.fusion = Some(Fusion {
        point: (140000, 100000),
        at: 50,
    });
    u[0].hp = 0;
    let o = tick(&mut c, &u, 50);
    assert!(!o.contains(&Cmd::SelfCost));
    assert!(c.fusion.is_none());
    assert!(c.orb.is_none());
}
#[test]
fn no_orb_or_foreign_owner_cannot_fuse() {
    let u = units();
    let mut c = Core::default();
    boot(&mut c, &u);
    c.amp_until = 480;
    c.step(&input(1, &u, Some(Action::OrbRed), &[]));
    assert!(c.pending.is_none());
}
#[test]
fn auto_nuke_requires_half_health_and_two_champions() {
    for hp in [4999, 5000] {
        let mut u = units();
        u[0].hp = hp;
        // Keep the target pair away from melee: survival can override a legal nuke.
        u[1].x=160000;
        u[4].x=170000;
        let mut c = Core::default();
        boot(&mut c, &u);
        c.amp_until = 480;
        c.orb = Some(Orb {
            point: (140000, 100000),
            end: 360,
            affected: vec![],
        });
        let mut i = input(1, &u, None, &[]);
        i.automatic = true;
        c.step(&i);
        assert_eq!(
            c.pending
                .as_ref()
                .is_some_and(|p| p.action == Action::OrbRed),
            hp >= 5000
        );
    }
}
#[test]
fn duplicate_ticks_are_inert_and_clones_independent() {
    let u = units();
    let mut a = Core::default();
    boot(&mut a, &u);
    let mut b = a.clone();
    a.step(&input(1, &u, Some(Action::Blue(2)), &[]));
    assert!(a.step(&input(1, &u, None, &[])).is_empty());
    assert_ne!(a, b);
    b.step(&input(1, &u, None, &[]));
    assert!(b.pending.is_none());
}

#[test]
fn orb_expiry_is_inclusive_and_cannot_fuse_at_boundary() {
    let (mut c, u) = orb_caster();
    c.orb.as_mut().unwrap().end = 1;
    c.step(&input(1, &u, None, &[]));
    assert!(c.orb.is_none());
}
#[test]
fn nuke_hits_enemies_only_and_never_structures() {
    let mut u = units();
    u[1].tower = true;
    let mut c = Core::default();
    boot(&mut c, &u);
    c.fusion = Some(Fusion {
        point: (120000, 100000),
        at: 1,
    });
    let o = tick(&mut c, &u, 1);
    let targets = o
        .iter()
        .filter_map(|v| {
            if let Cmd::Damage { target, .. } = v {
                Some(*target)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(targets, vec![5]);
}
#[test]
fn interrupted_maximum_blue_leaves_no_orb() {
    let u = units();
    let mut c = Core::default();
    boot(&mut c, &u);
    c.amp_until = 480;
    c.step(&input(1, &u, Some(Action::MaxBlue(2)), &[]));
    let mut i = input(2, &u, None, &[]);
    i.controlled = true;
    c.step(&i);
    tick(&mut c, &u, 70);
    assert!(c.orb.is_none());
    assert!(c.ready[0] > 70);
}
#[test]
fn rct_and_step_are_unavailable_before_skill_two_unlock() {
    let u = units();
    for a in [Action::Heal, Action::Blink(110000, 100000)] {
        let mut c = Core::default();
        boot(&mut c, &u);
        let mut i = input(1, &u, Some(a), &[]);
        i.unlocked = [true, false, false];
        c.step(&i);
        assert!(c.pending.is_none());
    }
}
#[test]
fn ordinary_purple_needs_eyes_and_both_cooldowns_but_not_flash() {
    let u = units();
    let mut c = Core::default();
    boot(&mut c, &u);
    c.step(&input(1, &u, Some(Action::Purple(2)), &[]));
    assert!(c.pending.is_some());
    assert_eq!(c.casts[4], 1);
    assert!(c.ready[0] > 1 && c.ready[2] > 1);
}
#[test]
fn partial_infinity_rebuild_is_not_free_after_break() {
    let u = units();
    let mut c = Core::default();
    boot(&mut c, &u);
    let mut i = input(1, &u, None, &[]);
    i.shield = Some(0);
    c.step(&i);
    assert_eq!(c.infinity_ready, 901);
    tick(&mut c, &u, 900);
    assert!(!c.infinity);
    let out = tick(&mut c, &u, 901);
    assert!(out.iter().any(|v| matches!(
        v,
        Cmd::Infinity {
            active: true,
            shield: 250
        }
    )));
}
#[test]
fn ordinary_attack_and_black_flash_clocks_do_not_double() {
    let u = units();
    let mut c = Core::default();
    boot(&mut c, &u);
    c.flash_ready = 360;
    tick(&mut c, &u, 180);
    assert_eq!(c.flash_ready, 360);
}
#[test]
fn independent_owner_orbs_and_fusions_do_not_leak() {
    let (mut a, u) = orb_caster();
    let mut b = Core::default();
    boot(&mut b, &u);
    b.amp_until = 480;
    b.step(&input(1, &u, Some(Action::OrbRed), &[]));
    assert!(b.pending.is_none());
    a.step(&input(1, &u, Some(Action::OrbRed), &[]));
    assert!(a.pending.is_some());
}
