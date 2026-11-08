//! Thalassa: core tests (shared with the Lab crate) and glue tests through the mock host.
include!("thalassa_core_tests.rs");

mod glue {
    use crate::glue_tests::{open_spot, with_sim, Host, Unit as HostUnit};
    use crate::thalassa_core::Order;
    use mod_api_stable::{CcKindV1, EntityKindV1 as K, StablePassive};

    #[test]
    fn moving_link_has_live_endpoints_and_bounded_one_tick_geometry() {
        let parts=crate::thalassa_art::link_segments;
        assert!(parts((0,0),(0,0)).is_empty());
        for end in [(100_000,100_000),(900_000,900_000),(100_000,900_000)] {
            let a=parts((100_000,100_000),end);
            let b=parts((110_000,120_000),(end.0+10_000,end.1+20_000));
            assert!(a.len()<=6);assert_eq!(a.len(),b.len());
            for (a,b) in a.iter().zip(&b) {assert_eq!(a.0,b.0);assert_eq!((b.1-a.1,b.2-a.2),(10_000,20_000));}
        }
        assert_ne!(parts((0,0),(40_000,0))[0].0,parts((0,0),(0,40_000))[0].0);
    }

    fn champ(team: usize, pos: (u64, u64), name: &'static str) -> HostUnit {
        HostUnit { kind: K::Champion, team, pos, hp: 2000, name, targetable: true }
    }

    #[test]
    fn presentation_renewals_do_not_change_combat_state_or_add_control() {
        let _g = crate::walls::TEST_LOCK.lock();
        let (x, y) = open_spot(90_000);
        let host = Host::new(vec![champ(0, (x, y), "tfm2_custom_thalassa")]);
        let mut t = crate::thalassa::Thalassa::default();
        with_sim(&host, |sim| t.on_spawn(sim, 0, 0));
        for tick in 100..115 {
            host.borrow_mut().tick = tick;
            with_sim(&host, |sim| {
                let mut f = t.frame(sim, 0); f.manual = true;
                if tick == 100 { f.order = Some(Order::TidePool(f.pos)); }
                let effects = t.core.step(&f); t.apply(sim, 0, effects);
            });
        }
        let core = serde_json::to_string(&t.core).unwrap();
        let snapshot = {
            let h = host.borrow();
            (h.buffs.clone(), h.ccs.clone(), h.damage.clone(), h.moves.clone(), h.units.iter().map(|u| (u.pos,u.hp)).collect::<Vec<_>>())
        };
        for tick in 115..180 { host.borrow_mut().tick=tick; with_sim(&host, |sim| t.present(sim,0)); }
        let h=host.borrow();
        assert_eq!(snapshot.0,h.buffs); assert_eq!(snapshot.1,h.ccs);
        assert_eq!(snapshot.2,h.damage); assert_eq!(snapshot.3,h.moves);
        assert_eq!(snapshot.4,h.units.iter().map(|u| (u.pos,u.hp)).collect::<Vec<_>>());
        assert_eq!(core,serde_json::to_string(&t.core).unwrap());
    }

    #[test]
    fn presentation_clock_keeps_cast_boundaries_and_fractional_idle_periods() {
        let frame=crate::thalassa_art::frame;
        assert_eq!(frame("skill1",2,false),0); assert_eq!(frame("skill1",3,false),1);
        assert_eq!(frame("skill1",12,false),3);
        assert_eq!(frame("idle",10,true),0); assert_eq!(frame("idle",11,true),1);
        assert_eq!(frame("idle",216,true),0);
        assert_eq!(frame("run",7,true),0); assert_eq!(frame("run",8,true),1);
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

    #[test]
    fn a_v1_parry_cancels_the_trident_stun_and_damage() {
        let _g = crate::walls::TEST_LOCK.lock();
        let (x, y) = open_spot(90_000);
        let host = Host::new(vec![champ(0, (x, y), "tfm2_custom_thalassa"), champ(1, (x + 30_000, y), "tfm2_ultrakill_v1")]);
        with_sim(&host, |sim| sim.add_buff(1, &crate::timed("v1_guard", 600)));
        let mut t = crate::thalassa::Thalassa::default();
        host.borrow_mut().tick = 50;
        with_sim(&host, |sim| {
            t.apply(sim, 0, vec![
                crate::thalassa_core::Effect::TridentHit { target: 1, at: crate::thalassa_core::Point::new(x as i64 + 30_000, y as i64), ticks: 180 },
                crate::thalassa_core::Effect::Damage { target: 1, amount: 300, from_clone: false },
            ])
        });
        let h = host.borrow();
        assert!(!h.ccs[1].iter().any(|c| c.0.kind == CcKindV1::Stun.code()), "no stun");
        assert!(!h.damage.iter().any(|d| d.1 == 1), "no damage");
        assert!(h.buffs[1].iter().any(|b| b.name() == "v1_parry"), "V1 parried");
    }
}
