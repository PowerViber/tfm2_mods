include!("grimhald_core_tests.rs");

mod glue {
    use crate::glue_tests::{open_spot, with_sim, Host, Unit as HostUnit};
    use crate::grimhald_cage as cage;
    use crate::grimhald_core::{self as core, Cage, Point};
    use mod_api_stable::{AttackTypeV1, CcKindV1, CcV1, EntityKindV1 as K, StablePassive};
    use std::cell::RefCell;

    fn champ(team: usize, pos: (u64, u64), name: &'static str) -> HostUnit {
        HostUnit { kind: K::Champion, team, pos, hp: 2000, name, targetable: true }
    }
    fn at(host: &RefCell<Host>, i: usize) -> Point {
        let p = host.borrow().units[i].pos;
        Point::new(p.0 as i64, p.1 as i64)
    }
    fn cage_tick(host: &RefCell<Host>, t: usize) {
        host.borrow_mut().tick = t;
        with_sim(host, |sim| {
            let all = crate::champions(sim);
            cage::tick(sim, &all, t);
        });
    }

    #[test]
    fn cage_holds_members_in_and_puts_outsiders_and_minions_out() {
        let _g = crate::walls::TEST_LOCK.lock();
        let (x, y) = open_spot(150_000);
        let c = Point::new(x as i64, y as i64);
        let host = Host::new(vec![
            champ(0, (x, y), "tfm2_custom_grimhald"),
            // a caged enemy that blinked out
            champ(1, (x + 90_000, y), "fighter"),
            // an outsider who got in
            champ(0, (x + 10_000, y + 5_000), "archer"),
            HostUnit { kind: K::Minion, team: 1, pos: (x, y + 20_000), hp: 400, name: "minion", targetable: true },
        ]);
        let cg = Cage { owner: 0, center: c, radius: 55_000, until: 1_000 };
        with_sim(&host, |sim| {
            sim.add_buff(0, &crate::timed(&cage::cage_buff(&cg), 900));
            sim.add_buff(0, &crate::timed(&cage::member_buff(0), 900));
            sim.add_buff(1, &crate::timed(&cage::member_buff(0), 900));
        });
        cage_tick(&host, 100);
        assert!(core::inside(c, 55_000, at(&host, 1), 10_000), "the caged enemy is back inside: {:?}", at(&host, 1));
        assert!(core::depth(c, 55_000, at(&host, 2)) <= -10_000.0, "the outsider is out: {:?}", at(&host, 2));
        assert!(core::depth(c, 55_000, at(&host, 3)) <= -29_000.0, "the minion is out of reach: {:?}", at(&host, 3));
        // can_interact: in-cage pairs only
        with_sim(&host, |sim| {
            let all = crate::champions(sim);
            assert!(cage::can_interact(&all, 0, 1));
            assert!(!cage::can_interact(&all, 0, 2));
            assert!(!cage::can_interact(&all, 1, 2));
        });
        // after it falls: nothing moves anyone
        host.borrow_mut().units[1].pos = (x + 90_000, y);
        cage_tick(&host, 1_000);
        assert_eq!(at(&host, 1), Point::new(x as i64 + 90_000, y as i64));
    }

    #[test]
    fn his_own_named_cc_is_not_read_as_held_but_a_real_stun_is() {
        let _g = crate::walls::TEST_LOCK.lock();
        let (x, y) = open_spot(70_000);
        let host = Host::new(vec![champ(0, (x, y), "tfm2_custom_grimhald"), champ(1, (x + 200_000, y), "fighter")]);
        let mut g = crate::grimhald::Grimhald::default();
        host.borrow_mut().tick = 10;
        with_sim(&host, |sim| {
            let mut own = CcV1::of_kind(CcKindV1::Stun, 30);
            own.set_name("grh_stun");
            sim.apply_cc(0, &own);
        });
        let f = with_sim(&host, |sim| g.frame(sim, 0));
        assert_eq!(f.hard_cc_ticks, 0);
        with_sim(&host, |sim| sim.apply_cc(0, &CcV1::of_kind(CcKindV1::Stun, 30)));
        let f = with_sim(&host, |sim| g.frame(sim, 0));
        assert!(f.hard_cc_ticks > 0);
    }

    #[test]
    fn grimhald_fights_through_the_host_and_lands_skill_damage() {
        let _g = crate::walls::TEST_LOCK.lock();
        let (x, y) = open_spot(80_000);
        let host = Host::new(vec![champ(0, (x, y), "tfm2_custom_grimhald"), champ(1, (x + 20_000, y), "fighter")]);
        let mut g = crate::grimhald::Grimhald::default();
        with_sim(&host, |sim| g.on_spawn(sim, 0, 0));
        for t in 100..400 {
            host.borrow_mut().tick = t;
            with_sim(&host, |sim| g.on_update(sim, t as u64, 0, 0));
        }
        let h = host.borrow();
        assert!(h.damage.iter().any(|d| d.0 == 0 && d.1 == 1 && d.3 == AttackTypeV1::Skill.code()), "{:?}", h.damage);
        // the pillar and its wall buff showed up at some point, or the bash moved him
        // round 156: dashes and knockbacks are engine forced moves now, not hand-set positions
        let forced = |i: usize| h.ccs[i].iter().any(|c| c.0.kind == mod_api_stable::CcKindV1::ForceMove.code());
        assert!(forced(0) || forced(1), "something moved through the engine");
    }
}

mod glue_round2 {
    use crate::glue_tests::{open_spot, with_sim, Host, Unit as HostUnit};
    use crate::grimhald_core::{Planted, Point};
    use crate::grimhald_data as d;
    use mod_api_stable::{EntityKindV1 as K, StablePassive};

    #[test]
    fn unarmed_basics_are_recorded_as_punches_and_the_buff_carries_its_fields() {
        let _g = crate::walls::TEST_LOCK.lock();
        let (x, y) = open_spot(80_000);
        let host = Host::new(vec![
            HostUnit { kind: K::Champion, team: 0, pos: (x, y), hp: 2000, name: "tfm2_custom_grimhald", targetable: true },
            HostUnit { kind: K::Champion, team: 1, pos: (x + 20_000, y), hp: 2000, name: "fighter", targetable: true },
        ]);
        let mut g = crate::grimhald::Grimhald::default();
        g.core.planted = Some(Planted { center: Point::new(x as i64, y as i64), radius: d::ULT_CAGE_RADIUS, until: 10_000 });
        host.borrow_mut().tick = 50;
        with_sim(&host, |sim| {
            let mut dmg = 100usize;
            g.on_attack(sim, 0, 0, 1, &mut dmg);
            g.apply(sim, 0, vec![crate::grimhald_core::Effect::Unarmed { on: true }]);
        });
        let f = with_sim(&host, |sim| g.frame(sim, 0));
        assert_eq!(f.punched, vec![1]);
        let h = host.borrow();
        let b = h.buffs[0].iter().find(|b| b.name() == crate::grimhald::UNARMED).expect("buff");
        assert_eq!(b.attack_speed_mult, d::UNARMED_ATTACK_SPEED_PCT as i32);
        assert_eq!(b.attack_mult, d::UNARMED_DAMAGE_PCT as i32);
        assert_eq!(b.vamp, d::UNARMED_LIFESTEAL_PCT as i32);
        assert_eq!(b.damaged_reduce, d::UNARMED_DAMAGE_REDUCTION_PCT as usize);
    }
}
