use crate::steve_craft_core::*;
use crate::steve_craft_geometry::*;
#[test]
fn rod_rechecks_charged_reach_and_cannot_grapple_empty_air(){
 let mut u=[unit(0,0,(480000,480000)),unit(1,1,(580000,480000))];let mut c=Core::default();c.step(&input(0,&u,Some(recipe(8,u[1].pos(),Some(1)))));let end=c.pending.as_ref().unwrap().end;u[1].x=700000;c.step(&input(end,&u,None));assert!(c.shots.is_empty());assert_eq!(c.reason,"Rod left charged reach");assert_eq!(c.available(end),2);assert!(c.cooldowns[8]>end);
 let mut c=Core::default();c.step(&input(0,&u,Some(recipe(8,(520000,480000),None))));assert_eq!(c.available(0),3);assert!(c.pending.is_none());assert_eq!(c.reason,"Grapple needs terrain or owned TNT");
 u[1].x=580000;let mut c=Core::default();resolved(&mut c,&u,recipe(8,u[1].pos(),Some(1)));assert_eq!(c.shots.len(),1,"{}",c.reason);
}
#[test]
fn external_wall_blocks_heavy_landing_and_short_boat_but_not_pearl_travel(){
 let u=[unit(0,0,(480000,480000))];let barriers=[((510000,450000),(510000,530000))];
 let mut c=Core::default();resolved(&mut c,&u,Action::Prepare(6));let t=c.last_tick.unwrap()+1;let mut i=input(t,&u,Some(Action::Deploy((510000,480000))));i.barriers=&barriers;c.step(&i);assert!(c.prepared.is_some());assert!(c.objects.is_empty());
 for r in [9,14]{let mut c=Core::default();let mut i=input(0,&u,Some(recipe(r,(540000,480000),None)));i.barriers=&barriers;c.step(&i);let mut moved=false;for t in 1..65{let mut i=input(t,&u,None);i.barriers=&barriers;let out=c.step(&i);moved|=out.iter().any(|v|matches!(v,Cmd::Move(0,p)if p.0>510000));}assert_eq!(moved,r==9,"recipe {r}");}
}
fn unit(id: usize, team: usize, p: Point) -> Unit {
    Unit {
        id,
        team,
        x: p.0,
        y: p.1,
        hp: 1150,
        max_hp: 1150,
        ad: 62,
        shield: 0,
        visible: true,
        targetable: true,
        champion: true,
        farmable: false,
        immune: false,
        cc_immune: false,
        immovable: false,
        tower: false,
        gadget: false,
        vx: 0,
        vy: 0,
        swing: false,
    }
}
fn input<'a>(t: usize, u: &'a [Unit], action: Option<Action>) -> Input<'a> {
    Input {
        tick: t,
        me: &u[0],
        units: u,
        unlocked: [true; 3],
        controlled: false,
        bound: false,
        automatic: false,
        action,
        basics: &[],
        objects: &[],
        wall: &|_, _| false,
        barriers: &[],
        home: (200000, 480000),
        haste: 0,
        ult_haste: 0,
    }
}
fn recipe(r: u8, p: Point, target: Option<usize>) -> Action {
    Action::Recipe {
        recipe: r,
        target,
        point: p,
    }
}
fn resolved(c: &mut Core, u: &[Unit], a: Action) -> Vec<Cmd> {
    let t = c.last_tick.map_or(0, |t| t + 1);
    let mut out = c.step(&input(t, u, Some(a)));
    if let Some(p) = &c.pending {
        out.extend(c.step(&input(p.end.max(t + 1), u, None)));
    }
    out
}
fn ack(c: &mut Core) {
    let ids: Vec<_> = c.objects.iter().map(|o| o.id).collect();
    for id in ids {
        c.allocated(id, true)
    }
}
#[test]
fn explosive_cover_and_non_recursive_pedestals() {
    let u = [unit(0, 0, (480000, 480000)), unit(1, 1, (520000, 480000))];
    let mut c = Core::default();
    resolved(&mut c, &u, Action::Prepare(6));
    resolved(&mut c, &u, Action::Deploy((500000, 480000)));
    ack(&mut c);
    let id = c.objects[0].id;
    resolved(&mut c, &u, Action::Detonate(id));
    let t = c.objects[0].blast.unwrap();
    let mut i = input(t, &u, None);
    i.wall = &|x, _| (508000..=512000).contains(&x);
    let out = c.step(&i);
    assert!(!out
        .iter()
        .any(|v| matches!(v, Cmd::Damage { target: 1, .. })));
    assert!(out
        .iter()
        .any(|v| matches!(v, Cmd::Damage { target: 0, .. })));
}
#[test]
fn tnt_and_ram_knockbacks_keep_twelve_tick_duration() {
    let u = [unit(0, 0, (480000, 480000)), unit(1, 1, (530000, 480000))];
    let mut c = Core::default();
    resolved(&mut c, &u, recipe(3, u[1].pos(), None));
    let t = c.last_tick.unwrap();
    c.step(&input(t + 15, &u, None));
    ack(&mut c);
    let t = c.objects[0].blast.unwrap();
    let out = c.step(&input(t, &u, None));
    assert!(out.iter().any(|v| matches!(v, Cmd::Push(1, _, 12))));
}
#[test]
fn immune_and_hidden_hits_do_not_grant_followup() {
    let mut u = [unit(0, 0, (480000, 480000)), unit(1, 1, (520000, 480000))];
    u[1].visible = false;
    let mut c = Core::default();
    c.form = Form::Herobrine;
    resolved(&mut c, &u, recipe(5, u[1].pos(), None));
    let ids: Vec<_> = c.receipts.iter().map(|r| r.id).collect();
    for id in ids {
        c.receipt(id, 20, 22)
    }
    assert!(c.followup.is_none());
    u[1].visible = true;
    u[1].cc_immune = true;
    let mut c = Core::default();
    c.form = Form::Herobrine;
    resolved(&mut c, &u, recipe(2, u[1].pos(), None));
    c.step(&input(23, &u, None));
    assert!(c.followup.is_none());
}
#[test]
fn tnt_swing_lights_mine_without_counting_as_melee_followup() {
    let mut u = [unit(0, 0, (480000, 480000))];
    let mut c = Core::default();
    resolved(&mut c, &u, recipe(3, (500000, 480000), None));
    let t = c.last_tick.unwrap() + 15;
    c.step(&input(t, &u, None));
    ack(&mut c);
    u[0].swing = true;
    c.step(&input(t + 1, &u, None));
    assert_eq!(c.objects[0].blast, Some(t + 13));
    assert_eq!(c.metrics.combos, 0);
}
#[test]
fn malformed_protocol_extremes_reject_without_panicking() {
    let mut a = vec![0i64; 22];
    a[0] = 1;
    a[1] = 1;
    a[7] = i64::MIN;
    assert!(crate::steve_craft_protocol::Frame::decode(&a).is_none());
    a[7] = 0;
    a[15] = 1025;
    assert!(crate::steve_craft_protocol::Frame::decode(&a).is_none());
}
#[test]
fn charge_bank_independent_and_reserved() {
    let u = [unit(0, 0, (480000, 480000))];
    let mut c = Core::default();
    resolved(&mut c, &u, Action::Prepare(6));
    assert_eq!(c.available(20000), 1);
    assert!(c.prepared.is_some());
    let at = c.last_tick.unwrap() + 1;
    c.step(&input(at, &u, Some(Action::Discard)));
    assert_eq!(c.available(at + 299), 1);
    assert_eq!(c.available(at + 300), 3);
}
#[test]
fn interrupted_prepare_never_releases_gadget() {
    let u = [unit(0, 0, (480000, 480000))];
    let mut c = Core::default();
    c.step(&input(0, &u, Some(Action::Prepare(7))));
    let mut i = input(10, &u, None);
    i.controlled = true;
    c.step(&i);
    assert!(c.prepared.is_none());
    assert!(c.pending.is_none());
    assert_eq!(c.available(309), 1);
    assert_eq!(c.available(310), 3);
}
#[test]
fn invalid_actions_spend_nothing() {
    let u = [unit(0, 0, (480000, 480000))];
    for a in [
        Action::Deploy((480000, 480000)),
        Action::Prepare(3),
        recipe(99, (480000, 480000), None),
        recipe(0, (480000, 480000), None),
        recipe(9, (-1, 480000), None),
    ] {
        let mut c = Core::default();
        c.step(&input(0, &u, Some(a)));
        assert_eq!(c.available(0), 3);
        assert!(c.pending.is_none());
    }
}
#[test]
fn all_recipes_are_reachable_without_mastery() {
    let u = [unit(0, 0, (480000, 480000)), unit(1, 1, (530000, 480000))];
    for r in 0..15 {
        let mut c = Core::default();
        let target = if r == 10 {
            Some(0)
        } else if r == 8 {
            Some(1)
        } else {
            None
        };
        if r == 6 || r == 7 {
            resolved(&mut c, &u, Action::Prepare(r));
            resolved(&mut c, &u, Action::Deploy((520000, 510000)));
        } else {
            resolved(&mut c, &u, recipe(r, (520000, 510000), target));
        }
        assert_eq!(c.metrics.casts[r as usize], 1, "recipe {r}: {}", c.reason);
    }
}
#[test]
fn failure_restores_resource_without_invisible_object() {
    let u = [unit(0, 0, (480000, 480000))];
    let mut c = Core::default();
    resolved(&mut c, &u, recipe(0, (530000, 480000), None));
    let id = c.objects[0].id;
    assert!(c.segments(30).is_empty());
    c.allocated(id, false);
    assert!(c.objects.is_empty());
    assert!(c.walls.is_empty());
    assert_eq!(c.cooldowns[0], 0);
    assert_eq!(c.available(30), 3);
}
#[test]
fn steve_object_hp_and_support_bonus() {
    let u = [unit(0, 0, (480000, 480000))];
    for (form, hp, heal, shield) in [
        (Form::Steve, 287, 182, 138),
        (Form::Herobrine, 230, 152, 115),
    ] {
        let mut c = Core::default();
        c.form = form;
        resolved(&mut c, &u, recipe(0, (530000, 480000), None));
        assert_eq!(c.objects[0].hp, hp);
        let out = resolved(&mut c, &u, recipe(10, u[0].pos(), Some(0)));
        assert!(out.contains(&Cmd::Heal(0, heal)));
        assert!(out.contains(&Cmd::Shield(0, shield, 240)));
    }
}
#[test]
fn form_switch_is_not_ultimate_and_interrupt_keeps_form() {
    let u = [unit(0, 0, (480000, 480000))];
    let mut c = Core::default();
    c.step(&input(0, &u, Some(Action::Stance)));
    assert_eq!(c.ult_at, 0);
    assert_eq!(c.stance_at, 720);
    let mut i = input(12, &u, None);
    i.controlled = true;
    c.step(&i);
    assert_eq!(c.form, Form::Steve);
    assert_eq!(c.stance_at, 720);
}
#[test]
fn unlocks_gate_stance_wall_and_recipes() {
    let u = [unit(0, 0, (480000, 480000))];
    for a in [
        Action::Stance,
        Action::Wall {
            centre: (650000, 480000),
            heading: 0,
            bend: 0,
        },
        recipe(3, (520000, 480000), None),
        recipe(12, (520000, 480000), None),
    ] {
        let mut c = Core::default();
        let mut i = input(0, &u, Some(a));
        i.unlocked = [false; 3];
        c.step(&i);
        assert!(c.pending.is_none());
        assert_eq!(c.available(0), 3);
        assert_eq!(c.ult_at, 0);
    }
}
#[test]
fn followup_requires_confirmed_champion_hit_not_requested_damage() {
    let u = [unit(0, 0, (480000, 480000)), unit(1, 1, (520000, 480000))];
    let mut c = Core::default();
    c.form = Form::Herobrine;
    let out = resolved(&mut c, &u, recipe(12, u[1].pos(), None));
    let _ = out;
    let t = c.last_tick.unwrap() + 10;
    let out = c.step(&input(t, &u, None));
    let event = out
        .iter()
        .find_map(|v| {
            if let Cmd::Damage { event, .. } = v {
                Some(*event)
            } else {
                None
            }
        })
        .unwrap();
    assert!(c.followup.is_none());
    c.receipt(event, 0, t);
    assert!(c.followup.is_none());
    let mut c = Core::default();
    c.form = Form::Herobrine;
    c.last_owner = 0;
    c.receipts.push(Receipt {
        id: 7,
        target: 1,
        champion: true,
        utility: true,
        at: 1,
    });
    c.receipt(7, 20, 1);
    assert_eq!(c.followup, Some((1, 241)));
    c.receipt(7, 20, 2);
    assert_eq!(c.followup, Some((1, 241)));
}
#[test]
fn followup_target_binding_expiry_and_single_basic_proc() {
    let u = [
        unit(0, 0, (480000, 480000)),
        unit(1, 1, (490000, 480000)),
        unit(2, 1, (500000, 480000)),
    ];
    let mut c = Core::default();
    c.form = Form::Herobrine;
    c.followup = Some((1, 240));
    let mut i = input(1, &u, None);
    i.basics = &[(1, 2)];
    assert!(c.step(&i).is_empty());
    let mut i = input(2, &u, None);
    i.basics = &[(2, 1), (2, 1)];
    let out = c.step(&i);
    assert_eq!(
        out.iter()
            .filter(|v| matches!(v, Cmd::Damage { .. }))
            .count(),
        1
    );
    assert!(out.iter().any(|v| matches!(v, Cmd::Damage { ad: 41, .. })));
    assert_eq!(c.metrics.combos, 1);
    assert_eq!(c.followup_at, 362);
}
#[test]
fn farming_cannot_qualify_followup() {
    let u = [unit(0, 0, (480000, 480000))];
    let mut c = Core::default();
    c.form = Form::Herobrine;
    c.last_owner = 0;
    for (champion, utility) in [(false, true), (true, false)] {
        c.receipts.push(Receipt {
            id: 1,
            target: 2,
            champion,
            utility,
            at: 0,
        });
        c.receipt(1, 100, 0);
        assert!(c.followup.is_none());
    }
    let _ = u;
}
#[test]
fn axe_bonus_is_consumed_only_once() {
    let u = [unit(0, 0, (480000, 480000)), unit(1, 1, (490000, 480000))];
    let mut c = Core::default();
    resolved(&mut c, &u, recipe(13, u[0].pos(), None));
    let mut i = input(30, &u, None);
    i.basics = &[(1, 1), (1, 1)];
    let out = c.step(&i);
    assert_eq!(
        out.iter()
            .filter(|v| matches!(v, Cmd::Damage { ad: 15, .. }))
            .count(),
        1
    );
}
#[test]
fn crystals_warn_on_destruction_anchors_defuse() {
    let u = [unit(0, 0, (480000, 480000)), unit(1, 1, (530000, 480000))];
    for r in [6, 7] {
        let mut c = Core::default();
        resolved(&mut c, &u, Action::Prepare(r));
        resolved(&mut c, &u, Action::Deploy((500000, 480000)));
        ack(&mut c);
        let t = c.last_tick.unwrap() + 1;
        let id = c.objects[0].id;
        let hp = [(id, 0)];
        let mut i = input(t, &u, None);
        i.objects = &hp;
        let out = c.step(&i);
        if r == 6 {
            assert!(out.iter().any(|v| matches!(
                v,
                Cmd::Warning {
                    owner_hazard: true,
                    ..
                }
            )));
            let out = c.step(&input(t + 36, &u, None));
            assert!(out
                .iter()
                .any(|v| matches!(v, Cmd::Damage { target: 1, .. })));
        } else {
            assert!(c.objects.is_empty());
            assert!(!out.iter().any(|v| matches!(v, Cmd::Damage { .. })));
        }
    }
}
#[test]
fn heavy_self_damage_half_raw_and_blast_occlusion() {
    let u = [unit(0, 0, (480000, 480000)), unit(1, 1, (520000, 480000))];
    let mut c = Core::default();
    resolved(&mut c, &u, Action::Prepare(6));
    resolved(&mut c, &u, Action::Deploy((500000, 480000)));
    ack(&mut c);
    let id = c.objects[0].id;
    resolved(&mut c, &u, Action::Detonate(id));
    let t = c.objects[0].blast.unwrap();
    let out = c.step(&input(t, &u, None));
    assert!(out.iter().any(|v| matches!(
        v,
        Cmd::Damage {
            target: 0,
            ap: 82,
            ..
        }
    )));
    assert!(out.iter().any(|v| matches!(
        v,
        Cmd::Damage {
            target: 1,
            ap: 165,
            ..
        }
    )));
    assert_eq!(
        out.iter()
            .filter(|v| matches!(v, Cmd::Damage { target: 1, .. }))
            .count(),
        1
    );
}
#[test]
fn cleanup_never_explodes_and_cooldowns_survive() {
    let u = [unit(0, 0, (480000, 480000))];
    let mut c = Core::default();
    resolved(&mut c, &u, Action::Prepare(7));
    resolved(&mut c, &u, Action::Deploy((520000, 480000)));
    ack(&mut c);
    let cd = c.cooldowns;
    c.ult_at = 1000;
    c.stance_at = 700;
    let out = c.died(100);
    assert!(out.iter().all(|v| matches!(v, Cmd::Remove(_))));
    assert_eq!(c.cooldowns, cd);
    assert_eq!(c.ult_at, 1000);
    assert_eq!(c.form, Form::Steve);
}
#[test]
fn curves_open_bounded_and_shared_path() {
    for h in 0..16 {
        for b in -4i8..=4 {
            let p = arc((480000, 480000), h, b);
            assert!(p.len() <= 17);
            assert!(
                path_length(&p) <= 300000 && (path_length(&p) - 300000).abs() < 100,
                "h{h}b{b}: {}",
                path_length(&p)
            );
            if b != 0 {
                assert!(length(p[0], *p.last().unwrap()) >= 48000)
            }
            for i in 0..p.len() - 1 {
                for j in i + 2..p.len() - 1 {
                    assert!(!crosses(p[i], p[i + 1], p[j], p[j + 1]));
                }
            }
        }
    }
}
#[test]
fn steve_rejects_curve_without_spending_ultimate() {
    let u = [unit(0, 0, (300000, 400000))];
    let mut c = Core::default();
    c.step(&input(
        0,
        &u,
        Some(Action::Wall {
            centre: (480000, 480000),
            heading: 0,
            bend: 4,
        }),
    ));
    assert_eq!(c.ult_at, 0);
    assert!(c.pending.is_none());
}
#[test]
fn path_warning_precedes_wall_and_initial_immunity_is_short() {
    let u = [unit(0, 0, (300000, 400000))];
    let mut c = Core::default();
    let out = c.step(&input(
        0,
        &u,
        Some(Action::Wall {
            centre: (480000, 480000),
            heading: 0,
            bend: 0,
        }),
    ));
    assert!(out
        .iter()
        .any(|v| matches!(v, Cmd::PathWarning { until: 30, .. })));
    assert!(c.walls.is_empty());
    let out = c.step(&input(30, &u, None));
    assert_eq!(c.walls.len(), 1);
    assert!(!out.iter().any(|v| matches!(v,Cmd::Immune(t)if *t>2)));
}
#[test]
fn empty_boat_completes_and_parry_removes_whole_curve() {
    let u = [unit(0, 0, (300000, 400000))];
    let mut c = Core::default();
    c.form = Form::Herobrine;
    resolved(
        &mut c,
        &u,
        Action::Wall {
            centre: (480000, 480000),
            heading: 0,
            bend: 4,
        },
    );
    assert!(c.ride.is_some(), "{}", c.reason);
    let t = c.last_tick.unwrap() + 1;
    c.step(&input(t, &u, Some(Action::Dismount((310000, 400000)))));
    assert!(!c.ride.as_ref().unwrap().rider);
    assert!(c.walls.iter().any(|w| w.ultimate));
    c.parried();
    assert!(c.ride.is_none());
    assert!(c.walls.is_empty());
}
#[test]
fn union_routes_around_u_mouth_without_crossing_any_segment() {
    let p = arc((480000, 480000), 0, 4);
    let seg: Vec<_> = p.windows(2).map(|p| (p[0], p[1])).collect();
    let a = (480000, 520000);
    let b = (480000, 640000);
    let dest = detour(a, b, &seg, &|_, _| false);
    if let Some(d) = dest {
        assert_ne!(d, a);
        assert!(!blocked(a, d, &seg));
    }
}
#[test]
fn solid_terrain_shortens_or_rejects_without_cd() {
    let u = [unit(0, 0, (300000, 400000))];
    let mut c = Core::default();
    let mut i = input(
        0,
        &u,
        Some(Action::Wall {
            centre: (480000, 480000),
            heading: 0,
            bend: 0,
        }),
    );
    i.wall = &|x, _| x >= 350000;
    c.step(&i);
    assert_eq!(c.ult_at, 0);
}
#[test]
fn capacity_limit_does_not_replace_existing_object() {
    let u = [unit(0, 0, (480000, 480000))];
    let mut c = Core::default();
    resolved(&mut c, &u, recipe(1, (520000, 480000), None));
    ack(&mut c);
    c.cooldowns[1] = 0;
    let ids: Vec<_> = c.objects.iter().map(|v| v.id).collect();
    resolved(&mut c, &u, recipe(1, (520000, 500000), None));
    assert_eq!(ids, c.objects.iter().map(|v| v.id).collect::<Vec<_>>());
}
#[test]
fn water_removes_only_owned_lava() {
    let u = [unit(0, 0, (480000, 480000))];
    let mut c = Core::default();
    resolved(&mut c, &u, recipe(5, (520000, 480000), None));
    resolved(&mut c, &u, recipe(4, (520000, 480000), None));
    assert!(!c.objects.iter().any(|o| o.recipe == 5));
    assert!(c.objects.iter().any(|o| o.recipe == 4));
}
#[test]
fn independent_owners_clones_and_duplicate_ticks() {
    let u = [unit(0, 0, (480000, 480000))];
    let mut a = Core::default();
    let mut b = a.clone();
    a.step(&input(0, &u, Some(Action::Prepare(6))));
    assert_eq!(b.available(1), 3);
    assert_eq!(a.available(1), 1);
    let snap = format!("{:?}", a);
    assert!(a.step(&input(0, &u, None)).is_empty());
    assert_eq!(a.available(1), 1);
    assert!(!snap.is_empty());
    b.step(&input(0, &u, Some(recipe(11, u[0].pos(), None))));
    assert_eq!(a.cooldowns[11], 0);
}
#[test]
fn bounded_planner_preserves_input_and_visiblity() {
    let mut u = vec![unit(0, 0, (480000, 480000)), unit(1, 1, (530000, 480000))];
    let mut c = Core::default();
    let before = format!("{:?}", c);
    let p = c.plan_lines(&input(0, &u, None));
    assert!(p.len() <= 8);
    assert!(p.iter().all(|v| v.actions.len() <= 3));
    assert!(c.metrics.branches <= 128);
    assert!(before.contains("charges"));
    u[1].visible = false;
    let mut c = Core::default();
    assert!(c.plan_lines(&input(0, &u, None)).is_empty());
}
