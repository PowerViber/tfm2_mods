//! Wright: core tests (shared with the Lab crate) and glue tests through the mock host.
include!("wright_core_tests.rs");

mod glue {
    use crate::glue_tests::{open_spot, with_sim, Host, Unit as HostUnit};
    use crate::wright_core::{Effect, Order, Point};
    use crate::wright_data as d;
    use mod_api_stable::{CcKindV1, EntityKindV1 as K, StablePassive};

    #[test]
    fn body_clock_aligns_the_single_rivet_release_and_walking_cadence() {
        let frame=crate::wright_art::frame;
        assert_eq!(frame("attack",11,false),3);assert_eq!(frame("attack",12,false),4);
        assert_eq!(frame("attack",22,false),7);assert_eq!(frame("run",47,true),7);assert_eq!(frame("run",48,true),0);
        assert_eq!(frame("idle",72,true),0);assert_eq!(frame("skill1",18,false),7);
    }

    #[test]
    fn a_moving_rocket_target_gets_an_entity_attached_reticle() {
        let _g=crate::walls::TEST_LOCK.lock();let (x,y)=open_spot(90_000);
        let host=Host::new(vec![champ(0,(x,y),"tfm2_custom_wright"),champ(1,(x+40_000,y),"fighter")]);
        host.borrow_mut().record_view_effects=true;
        let mut w=crate::wright::Wright::default();with_sim(&host,|sim|w.on_spawn(sim,0,0));
        for tick in 100..420 {
            host.borrow_mut().tick=tick;
            host.borrow_mut().units[1].pos.1=y+(tick%10) as u64*100;
            with_sim(&host,|sim|{let mut f=w.frame(sim,0);f.manual=true;if tick==100{f.order=Some(Order::Fighters(Point::default()));}let effects=w.core.step(&f);w.apply(sim,0,effects);});
        }
        let h=host.borrow();let locks:Vec<_>=h.view_effects.iter().filter(|v|v.0=="wrt_lock").collect();assert!(!locks.is_empty());
        for lock in locks {assert_eq!(lock.2,mod_api_stable::InputTargetV1::target(1));assert_eq!(lock.3,d::FIGHTER_LOCK_TICKS as u64);}
    }

    #[test]
    fn timed_visual_variants_preserve_core_hp_control_and_lifetime() {
        let host=Host::new(vec![champ(0,(100_000,100_000),"tfm2_custom_wright")]);host.borrow_mut().record_view_effects=true;
        let mut w=crate::wright::Wright::default();let before=serde_json::to_string(&w.core).unwrap();
        with_sim(&host,|sim|w.apply(sim,0,vec![Effect::Fx{name:"wrt_bomb".into(),at:Point::new(100_000,100_000),ticks:14,ambient:false}]));
        assert_eq!(before,serde_json::to_string(&w.core).unwrap());let h=host.borrow();
        assert_eq!(h.units[0].hp,2000);assert!(h.damage.is_empty()&&h.ccs[0].is_empty()&&h.moves.is_empty());
        assert_eq!(h.view_effects[0].0,"wrt_bomb_t14");assert_eq!(h.view_effects[0].3,14);
    }

    fn champ(team: usize, pos: (u64, u64), name: &'static str) -> HostUnit {
        HostUnit { kind: K::Champion, team, pos, hp: 2000, name, targetable: true }
    }

    #[test]
    fn a_known_spawn_rejection_has_no_invisible_fighter_or_damage() {
        let _g=crate::walls::TEST_LOCK.lock();let(x,y)=open_spot(90_000);
        let host=Host::new(vec![champ(0,(x,y),"tfm2_custom_wright"),champ(1,(x+10_000,y),"fighter")]);host.borrow_mut().reject_spawns=true;
        let mut w=crate::wright::Wright::default();with_sim(&host,|sim|w.on_spawn(sim,0,0));
        for tick in 100..105 {host.borrow_mut().tick=tick;with_sim(&host,|sim|{let mut f=w.frame(sim,0);f.manual=true;if tick==100{f.order=Some(Order::Fighters(Point::default()));}let out=w.core.step(&f);w.apply(sim,0,out);});}
        assert_eq!(w.core.own_fighters(),0);let h=host.borrow();assert!(h.spawned.is_empty()&&h.damage.is_empty());
        assert_eq!(w.core.stock,0,"accepted airframe cost is preserved");
    }

    #[test]
    fn dead_owner_cleanup_removes_real_carriers_without_resetting_cooldowns() {
        let _g=crate::walls::TEST_LOCK.lock();let(x,y)=open_spot(90_000);let host=Host::new(vec![champ(0,(x,y),"tfm2_custom_wright")]);
        let mut w=crate::wright::Wright::default();host.borrow_mut().tick=100;
        with_sim(&host,|sim|{w.on_spawn(sim,0,0);let mut f=w.frame(sim,0);f.manual=true;f.order=Some(Order::Fighters(Point::default()));let out=w.core.step(&f);w.apply(sim,0,out);});
        let ready=w.core.ready;let ids:Vec<_>=host.borrow().spawned.iter().map(|s|s.0).collect();assert_eq!(ids.len(),2);
        host.borrow_mut().units[0].hp=0;host.borrow_mut().tick=101;
        with_sim(&host,|sim|w.on_update(sim,0,0,0));
        assert!(w.core.planes.is_empty()&&w.core.carrier.is_none());assert_eq!(w.core.ready[0],ready[0]);
        let h=host.borrow();assert!(ids.iter().all(|&id|h.units[id].hp==0));assert!(h.damage.is_empty());
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
