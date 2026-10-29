use super::*;

#[test]
fn child_blasts_and_bot_impacts_use_the_enemy_multiplier_once() {
    let (mut c, mut f) = (ready(), frame());
    f.units[0].attack = 65;
    let aim = f.units[1].pos;
    tick(&mut c, &mut f, Some(Action::Grenade { aim }));
    advance(&mut c, &mut f, GRENADE_PREP as usize);
    let wait = c.grenades[0].explosion - f.tick;
    advance(&mut c, &mut f, wait as usize);
    f.units[1].pos = c.grenades[0].centres[0];
    let wait = c.grenades[0].cluster - f.tick;
    let out = advance(&mut c, &mut f, wait as usize);
    assert!(out
        .iter()
        .any(|v| matches!(v,Cmd::Damage{target:2,raw:99,source,..}if source=="cluster")));
    let (mut c, mut f) = (ready(), frame());
    f.units[0].attack = 65;
    f.units[1].pos = Point::new(330000, 300000);
    tick(
        &mut c,
        &mut f,
        Some(Action::Bot {
            aim: Point::new(400000, 300000),
        }),
    );
    let out = advance(&mut c, &mut f, 100);
    assert!(out
        .iter()
        .any(|v| matches!(v,Cmd::Damage{target:2,raw:345,source,..}if source=="bot")));
}

#[test]
fn rocket_terminal_impact_has_triple_enemy_damage_and_original_self_damage() {
    for kind in [0, 1, 2, 3, 5] {
        let (mut c, mut f) = (ready(), frame());
        f.units[0].attack = 65;
        f.units[1].pos = Point::new(800000, 800000);
        f.units[1].kind = kind;
        equip(&mut c, &mut f);
        let aim = Point::new(360000, 300000);
        tick(&mut c, &mut f, Some(Action::FireRocket { aim }));
        // Last segment of a projectile reaching its already committed endpoint.
        let r = c.rocket.as_mut().unwrap();
        r.pos = aim;
        r.remaining = 0;
        f.units[0].pos = aim;
        f.units[1].pos = aim;
        let out = tick(&mut c, &mut f, None);
        assert!(out.iter().any(|v|matches!(v,Cmd::Damage{target:2,raw:1011,self_hit:false,source,..}if source=="rocket")),"{kind}: {out:?}");
        assert!(
            out.iter().any(|v| matches!(
                v,
                Cmd::Damage {
                    target: 1,
                    raw: 50,
                    self_hit: true,
                    ..
                }
            )),
            "{kind}: {out:?}"
        );
    }
}

#[test]
fn exact_triple_scaling_preserves_the_original_rounding_and_falloff() {
    for ap in [0, 1, 50, 65, 100, 200, 999] {
        for old in [
            PACK_DAMAGE + ap * PACK_ATTACK / 100,
            GRENADE_DAMAGE + ap * GRENADE_ATTACK / 100,
            CLUSTER_DAMAGE + ap * CLUSTER_ATTACK_NUM / CLUSTER_ATTACK_DEN,
            BOT_DAMAGE + ap * BOT_ATTACK / 100,
        ] {
            assert_eq!(skill_raw(old), old * 3);
        }
        for d in [0, 1, 8000, 25000, 49999, 50000, 50001] {
            let old = rocket_damage(ROCKET_DAMAGE + ap * ROCKET_ATTACK / 100, d);
            assert_eq!(skill_raw(old), old * 3);
        }
    }
    assert_eq!(
        [
            PACK_DAMAGE + 65 * PACK_ATTACK / 100,
            GRENADE_DAMAGE + 65 * GRENADE_ATTACK / 100,
            CLUSTER_DAMAGE + 65 * CLUSTER_ATTACK_NUM / CLUSTER_ATTACK_DEN,
            BOT_DAMAGE + 65 * BOT_ATTACK / 100,
            ROCKET_DAMAGE + 65 * ROCKET_ATTACK / 100
        ]
        .map(skill_raw),
        [138, 399, 99, 345, 1011]
    );
}

#[test]
fn grenade_packets_are_tripled_once_but_self_damage_is_the_old_amount() {
    let (mut c, mut f) = (ready(), frame());
    f.units[0].attack = 65;
    let aim = f.units[1].pos;
    tick(&mut c, &mut f, Some(Action::Grenade { aim }));
    advance(&mut c, &mut f, GRENADE_PREP as usize);
    f.units[0].pos = aim;
    let out = advance(&mut c, &mut f, 100);
    assert!(out.iter().any(
        |v| matches!(v,Cmd::Damage{target:2,raw:399,self_hit:false,source,..}if source=="grenade")
    ));
    assert!(out.iter().any(|v| matches!(
        v,
        Cmd::Damage {
            target: 1,
            raw: 19,
            self_hit: true,
            ..
        }
    )));
    assert_eq!(c.metrics.damage_by_source["grenade"], 399);
    assert_eq!(c.metrics.requested_damage, 399);
}

#[test]
fn item_attack_is_read_at_the_existing_cast_snapshot_without_a_second_multiplier() {
    for (ap, raw) in [(65, 399), (165, 624)] {
        let (mut c, mut f) = (ready(), frame());
        f.units[0].attack = ap;
        let aim = f.units[1].pos;
        tick(&mut c, &mut f, Some(Action::Grenade { aim }));
        f.units[0].attack = 999;
        let out = advance(&mut c, &mut f, 130);
        assert!(out.iter().any(
            |v| matches!(v,Cmd::Damage{target:2,raw:r,source,..}if *r==raw&&source=="grenade")
        ));
    }
}

#[test]
fn bucky_is_an_appended_starter_and_omens_ids_and_upgrades_are_unchanged() {
    use crate::valorant_guns::{basic_raw, shotgun_percent};
    assert_eq!(Core::default().gun, 6);
    assert_eq!(Core::default().credits, 1000);
    assert_eq!(
        (Gun::Bucky.reach(), Gun::Bucky.modifiers()),
        (18000, (0, -40))
    );
    for (d, p) in [(0, 300), (8000, 300), (13000, 180), (18000, 60)] {
        assert_eq!(shotgun_percent(Gun::Bucky, d), p);
        assert_eq!(basic_raw(Gun::Bucky, 100, d), p);
    }
    for (i, g) in [
        Gun::Classic,
        Gun::Sheriff,
        Gun::Spectre,
        Gun::Judge,
        Gun::Vandal,
        Gun::Operator,
    ]
    .into_iter()
    .enumerate()
    {
        assert_eq!(Gun::from_index(i), g);
        assert_eq!(g.idx(), i);
    }
    assert_eq!(Gun::default(), Gun::Classic);
    assert_eq!(Gun::upgrade(1000, 0, Gun::Classic), Gun::Sheriff);
    let mut c = Core::default();
    c.gun = Gun::Judge.idx();
    c.died();
    assert_eq!(c.gun, Gun::Judge.idx());
}

#[test]
fn a_long_buy_sequence_reaches_the_weapon_instead_of_refilling_satchel_stock() {
    let (mut c, mut f) = (Core::default(), frame());
    f.units.truncate(1);
    f.buy_allowed = true;
    for n in 0..5 {
        tick(&mut c, &mut f, Some(Action::Buy));
        assert_eq!(c.credits, 1000 + n * 200);
        assert_eq!(c.stock, [0; 3]);
        c.credits += 200;
    }
    let out = tick(&mut c, &mut f, Some(Action::Buy));
    assert!(out.iter().any(|v| matches!(v, Cmd::Gun { index: 3 })));
    assert_eq!(c.gun, Gun::Judge.idx());
    assert_eq!(c.credits, 150);
    assert_eq!(c.savings.reserve, 0);
}

#[test]
fn unsafe_or_blocked_entries_save_for_a_vandal_instead() {
    let (mut c, mut f) = (Core::default(), frame());
    f.buy_allowed = true;
    f.units[1].attack = 10000;
    tick(&mut c, &mut f, Some(Action::Buy));
    assert_eq!(c.savings.goal, Some(Gun::Vandal.idx()));
    assert_eq!(c.savings.reserve, 2900);
    assert_eq!(c.credits, 1000);
    c.credits = 3000;
    f.tick += UPGRADE_TICKS as u64;
    tick(&mut c, &mut f, Some(Action::Buy));
    assert_eq!(c.gun, Gun::Vandal.idx());
    assert_eq!(c.credits, 100);
}

#[test]
fn paid_weapon_replacements_respect_the_clock_and_cannot_downgrade() {
    let (mut c, mut f) = (Core::default(), frame());
    f.buy_allowed = true;
    c.gun = Gun::Judge.idx();
    c.last_weapon_buy = Some(1);
    c.credits = 4000;
    f.units[1].attack = 10000;
    tick(&mut c, &mut f, Some(Action::Buy));
    assert_eq!(c.gun, Gun::Judge.idx());
    assert!(c.credits >= 2900);
    f.tick += 900;
    tick(&mut c, &mut f, Some(Action::Buy));
    assert_eq!(c.gun, Gun::Vandal.idx());
    f.units.truncate(1);
    c.credits = 5000;
    f.tick += 900;
    tick(&mut c, &mut f, Some(Action::Buy));
    assert_eq!(c.gun, Gun::Vandal.idx());
}

fn entry_scene() -> (Core, Frame, Point) {
    let (c, mut f) = (ready(), frame());
    f.units[1].hp = 10000;
    let landing = f.units[1].pos.toward(f.units[0].pos, 9000);
    f.automatic = true;
    (c, f, landing)
}

#[test]
fn minions_dead_allies_and_controlled_allies_do_not_qualify_as_follow_up() {
    let (c, mut f, landing) = entry_scene();
    let mut ally = unit(3, 0, landing.x, landing.y);
    ally.kind = 1;
    f.units.push(ally);
    assert!(
        !c.entry_review(&f, &f.units[0], &f.units[1], landing, 2, false)
            .allowed
    );
    f.units[2].kind = 0;
    let r = c.entry_review(&f, &f.units[0], &f.units[1], landing, 2, false);
    assert!(r.allowed && r.ally == Some(3));
    f.units[2].controlled = true;
    assert!(
        !c.entry_review(&f, &f.units[0], &f.units[1], landing, 2, false)
            .allowed
    );
    f.units[2].controlled = false;
    f.units[2].hp = 0;
    assert!(
        !c.entry_review(&f, &f.units[0], &f.units[1], landing, 2, false)
            .allowed
    );
}

#[test]
fn ally_travel_time_and_blocked_approaches_matter() {
    let (c, mut f, landing) = entry_scene();
    f.units.push(unit(3, 0, landing.x, 450000));
    assert!(
        !c.entry_review(&f, &f.units[0], &f.units[1], landing, 2, false)
            .allowed
    );
    f.units[2].pos.y = 340000;
    assert!(
        c.entry_review(&f, &f.units[0], &f.units[1], landing, 2, false)
            .allowed
    );
    f.blocks.push(Rect {
        min: Point::new(330000, 320000),
        max: Point::new(370000, 325000),
    });
    assert!(
        !c.entry_review(&f, &f.units[0], &f.units[1], landing, 2, false)
            .allowed
    );
}

#[test]
fn retaining_a_satchel_requires_a_usable_exit_and_double_entries_need_support() {
    let (c, mut f, landing) = entry_scene();
    let r = c.entry_review(&f, &f.units[0], &f.units[1], landing, 1, false);
    assert!(r.allowed && r.retained_packs >= 1 && r.escape.is_some());
    let r = c.entry_review(&f, &f.units[0], &f.units[1], landing, 2, false);
    assert!(!r.allowed);
    f.units[1].attack = 10000;
    assert!(
        !c.entry_review(&f, &f.units[0], &f.units[1], landing, 1, false)
            .allowed
    );
}

#[test]
fn remote_deployed_satchel_is_not_a_retained_escape_charge() {
    let (mut c, f, landing) = entry_scene();
    c.stock[0] = 0;
    c.packs.push(Pack {
        id: 999,
        pos: Point::new(800000, 800000),
        born: 0,
        until: 1000,
        hp: PACK_HP,
        ready: true,
        attachment: Attachment {
            surface: None,
            kind: "ground".into(),
            point: Point::default(),
            normal: Point::default(),
            offset: Point::default(),
        },
    });
    let r = c.entry_review(&f, &f.units[0], &f.units[1], landing, 1, false);
    assert_eq!(r.retained_packs, 0);
    assert!(!r.allowed);
    c.packs[0].pos = f.units[0].pos;
    c.stock[0] = 1;
    let r = c.entry_review(&f, &f.units[0], &f.units[1], landing, 1, false);
    assert!(r.allowed && r.retained_packs >= 1);
}

#[test]
fn an_enemy_shield_prevents_a_false_isolated_finisher() {
    let (c, mut f, landing) = entry_scene();
    f.units[1].hp = 1;
    f.units[1].shield = 10000;
    let r = c.entry_review(&f, &f.units[0], &f.units[1], landing, 2, false);
    assert!(!r.solo_pick && !r.allowed);
}

#[test]
fn isolated_finishing_entries_require_a_walking_retreat_and_no_close_reinforcement() {
    let (c, mut f, landing) = entry_scene();
    f.units[1].hp = 1;
    let r = c.entry_review(&f, &f.units[0], &f.units[1], landing, 2, false);
    assert!(r.allowed && r.solo_pick);
    f.units
        .push(unit(4, 1, f.units[1].pos.x + 80000, f.units[1].pos.y));
    assert!(
        !c.entry_review(&f, &f.units[0], &f.units[1], landing, 2, false)
            .solo_pick
    );
}

#[test]
fn support_loss_cancels_the_second_launch_without_cancelling_released_flight() {
    let (mut c, mut f, landing) = entry_scene();
    f.units.push(unit(3, 0, landing.x, landing.y + 10000));
    let candidates = c.evaluate(&f, &f.units[0]);
    let best = candidates
        .iter()
        .find(|c| c.family == "double_launch" && c.legal)
        .unwrap();
    c.plan = Some(c.make_plan(&f, &f.units[0], best, f.tick));
    for _ in 0..30 {
        tick(&mut c, &mut f, None);
        if c.flight.is_some() {
            break;
        }
    }
    assert_eq!(c.metrics.launches, 1);
    f.units[2].hp = 0;
    for _ in 0..10 {
        tick(&mut c, &mut f, None);
        if c.last_rejection.starts_with("Entry cancelled") {
            break;
        }
    }
    assert!(c.last_rejection.starts_with("Entry cancelled"));
    assert_eq!(c.metrics.launches, 1);
    assert!(c.flight.is_some());
}
