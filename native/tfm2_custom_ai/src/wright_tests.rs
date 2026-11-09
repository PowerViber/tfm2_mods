//! Wright: core tests (shared with the Lab crate) and glue tests through the mock host.
include!("wright_core_tests.rs");

mod glue {
    use crate::glue_tests::{open_spot, with_sim, Host, Unit as HostUnit};
    use crate::wright_core::{Effect, Order, Point};
    use crate::wright_data as d;
    use mod_api_stable::{CcKindV1, EntityKindV1 as K, StablePassive};

    fn champ(team: usize, pos: (u64, u64), name: &'static str) -> HostUnit {
        HostUnit { kind: K::Champion, team, pos, hp: 2000, name, targetable: true }
    }

    #[test]
    fn fighters_are_real_carriers_with_their_hp_moved_by_engine_glides_and_can_be_shot_down() {
        let _g = crate::walls::TEST_LOCK.lock();
        let (x, y) = open_spot(90_000);
        let host = Host::new(vec![champ(0, (x, y), "tfm2_custom_wright"), champ(1, (x + 40_000, y), "fighter")]);
        let mut w = crate::wright::Wright::default();
        with_sim(&host, |sim| w.on_spawn(sim, 0, 0));
        for tick in 100..140 {
            host.borrow_mut().tick = tick;
            with_sim(&host, |sim| {
                let mut f = w.frame(sim, 0);
                f.manual = true;
                if tick == 100 {
                    f.order = Some(Order::Fighters(Point::default()));
                }
                assert!(!f.units.iter().any(|u| u.id >= 2), "his carriers are not in his own unit list");
                let e = w.core.step(&f);
                w.apply(sim, 0, e);
            });
        }
        {
            let h = host.borrow();
            assert_eq!(h.spawned.len(), d::FIGHTER_PER_CAST as usize);
            for (_, _, stat) in &h.spawned {
                assert_eq!(stat.hp as i64, 2000 * d::FIGHTER_HP_PCT / 100);
            }
            let carrier = h.spawned[0].0;
            assert!(h.ccs[carrier].iter().any(|c| c.0.kind == CcKindV1::ForceMove.code()), "flies by engine glides");
            assert!(!h.moves.iter().any(|m| m.0 == carrier), "never a hand-set position");
        }
        // shoot one down
        let first = host.borrow().spawned[0].0;
        host.borrow_mut().units[first].hp = 0;
        host.borrow_mut().tick = 141;
        with_sim(&host, |sim| {
            let f = w.frame(sim, 0);
            let e = w.core.step(&f);
            w.apply(sim, 0, e);
        });
        assert_eq!(w.core.own_fighters(), d::FIGHTER_PER_CAST as usize - 1, "shot down");
    }

    #[test]
    fn a_v1_parry_cancels_a_rocket_and_knockbacks_are_engine_glides() {
        let _g = crate::walls::TEST_LOCK.lock();
        let (x, y) = open_spot(90_000);
        let host = Host::new(vec![champ(0, (x, y), "tfm2_custom_wright"), champ(1, (x + 30_000, y), "tfm2_ultrakill_v1")]);
        with_sim(&host, |sim| sim.add_buff(1, &crate::timed("v1_guard", 600)));
        let mut w = crate::wright::Wright::default();
        host.borrow_mut().tick = 50;
        with_sim(&host, |sim| {
            w.apply(sim, 0, vec![
                Effect::Damage { target: 1, amount: 75, direct: true },
                Effect::Knock { target: 1, to: Point::new(x as i64 + 50_000, y as i64), ticks: 12 },
            ])
        });
        let h = host.borrow();
        assert!(!h.damage.iter().any(|d| d.1 == 1), "no damage");
        assert!(h.buffs[1].iter().any(|b| b.name() == "v1_parry"), "parried");
        assert!(h.ccs[1].iter().any(|c| c.0.kind == CcKindV1::ForceMove.code()), "the knockback is an engine glide");
        assert!(!h.moves.iter().any(|m| m.0 == 1));
    }
}
