//! Thalassa: core tests (shared with the Lab crate) and glue tests through the mock host.
include!("thalassa_core_tests.rs");

mod glue {
    use crate::glue_tests::{open_spot, with_sim, Host, Unit as HostUnit};
    use crate::thalassa_core::Order;
    use mod_api_stable::{CcKindV1, EntityKindV1 as K, StablePassive};

    fn champ(team: usize, pos: (u64, u64), name: &'static str) -> HostUnit {
        HostUnit { kind: K::Champion, team, pos, hp: 2000, name, targetable: true }
    }

    #[test]
    fn pool_and_dive_go_through_the_host_as_buffs_and_engine_motion() {
        let _g = crate::walls::TEST_LOCK.lock();
        let (x, y) = open_spot(90_000);
        let host = Host::new(vec![champ(0, (x, y), "tfm2_custom_thalassa")]);
        let mut t = crate::thalassa::Thalassa::default();
        with_sim(&host, |sim| t.on_spawn(sim, 0, 0));
        for tick in 100..120 {
            host.borrow_mut().tick = tick;
            with_sim(&host, |sim| {
                let mut f = t.frame(sim, 0);
                f.manual = true;
                if tick == 100 {
                    f.order = Some(Order::TidePool(f.pos));
                }
                if tick == 115 {
                    f.order = Some(Order::Dive(crate::thalassa_core::Point::new(x as i64 + 40_000, y as i64)));
                }
                let e = t.core.step(&f);
                t.apply(sim, 0, e);
            });
        }
        let h = host.borrow();
        assert!(h.buffs[0].iter().any(|b| b.name().starts_with(crate::thalassa::POOL_PREFIX)), "pools are buffs");
        assert!(h.buffs[0].iter().any(|b| b.name() == "tls_water"), "water mode on");
        assert!(h.ccs[0].iter().any(|c| c.0.kind == CcKindV1::ForceMove.code() && crate::motion::is_self_motion(&c.0)), "the dive is an engine glide");
        assert!(!h.moves.iter().any(|m| m.0 == 0), "never a hand-set position for her");
    }

    #[test]
    fn her_own_motion_never_reads_as_being_stunned() {
        let _g = crate::walls::TEST_LOCK.lock();
        let (x, y) = open_spot(90_000);
        let host = Host::new(vec![champ(0, (x, y), "tfm2_custom_thalassa")]);
        let mut t = crate::thalassa::Thalassa::default();
        host.borrow_mut().tick = 10;
        with_sim(&host, |sim| {
            crate::motion::glide(sim, 0, (x as i64 + 20_000, y as i64), 10, crate::motion::Kind::SelfMove);
        });
        let f = with_sim(&host, |sim| t.frame(sim, 0));
        assert!(!f.stunned);
    }
}
