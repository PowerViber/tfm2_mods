//! Round 177: the universal-AI kits (DIO, V1, Darth Vader, David, Minato) finish a skill the engine cast on a minion
//! or camp monster. Every skill but the ult may go on creeps; champions always come first. Driven through the glue
//! mock host, so the passives read the same buffs, positions and kinds the game hands them.
use crate::glue_tests::{open_spot, with_sim, Host, Unit};
use mod_api_stable::{CcKindV1, EntityKindV1 as K, StablePassive};
use std::cell::RefCell;

fn champ(team: usize, pos: (u64, u64), name: &'static str) -> Unit {
    Unit { kind: K::Champion, team, pos, hp: 2000, name, targetable: true }
}
fn monster(pos: (u64, u64)) -> Unit {
    Unit { kind: K::Jungle, team: 2, pos, hp: 3000, name: "camp", targetable: true }
}
fn buff(host: &RefCell<Host>, id: usize, name: &str, ticks: usize) {
    host.borrow_mut().buffs[id].push(crate::timed(name, ticks));
}
fn hits_on(host: &RefCell<Host>, id: usize) -> usize {
    host.borrow().damage.iter().filter(|d| d.1 == id).count()
}
fn step<P: StablePassive>(host: &RefCell<Host>, p: &mut P, tick: usize) {
    host.borrow_mut().tick = tick;
    with_sim(host, |sim| p.on_update(sim, 11, 0, 0));
}

#[test]
fn vader_chokes_and_slashes_a_camp_monster_when_no_champion_is_there() {
    let _guard = crate::walls::TEST_LOCK.lock();
    let (x, y) = open_spot(90_000);
    let host = Host::new(vec![champ(0, (x, y), "tfm2_starwars_vader"), monster((x + 30_000, y))]);
    buff(&host, 0, "vader_choking", 6);
    buff(&host, 1, "vader_choke_aim", 6);
    let mut v = crate::batch2::Vader::default();
    for t in 1..=100 {
        step(&host, &mut v, t);
    }
    let g = host.borrow();
    assert!(g.ccs[1].iter().any(|c| c.0.kind == CcKindV1::Airborne.code()), "lifted");
    drop(g);
    assert!(hits_on(&host, 1) >= 3, "the lift and both slashes land on the monster: {}", hits_on(&host, 1));
}

#[test]
fn vader_still_chokes_the_champion_first() {
    let _guard = crate::walls::TEST_LOCK.lock();
    let (x, y) = open_spot(90_000);
    let host = Host::new(vec![champ(0, (x, y), "tfm2_starwars_vader"), monster((x + 30_000, y)), champ(1, (x + 40_000, y), "foe")]);
    buff(&host, 0, "vader_choking", 6);
    buff(&host, 1, "vader_choke_aim", 6);
    let mut v = crate::batch2::Vader::default();
    for t in 1..=100 {
        step(&host, &mut v, t);
    }
    assert!(hits_on(&host, 2) >= 1, "the champion is caught");
    assert!(!host.borrow().ccs[1].iter().any(|c| c.0.kind == CcKindV1::Airborne.code()), "creeps only when no champion is caught");
}

#[test]
fn v1_tosses_the_coin_at_a_creep_and_the_ricochet_hits_it() {
    let _guard = crate::walls::TEST_LOCK.lock();
    let (x, y) = open_spot(90_000);
    let host = Host::new(vec![champ(0, (x, y), "tfm2_ultrakill_v1"), monster((x + 40_000, y))]);
    buff(&host, 0, "v1_coin", 6);
    buff(&host, 1, "v1_coin_target", 6);
    let mut v = crate::batch2::V1::default();
    for t in 1..=60 {
        step(&host, &mut v, t);
    }
    assert!(hits_on(&host, 1) >= 1, "the coin shot hits the camp monster");
}

#[test]
fn dio_stand_out_lunges_and_grabs_a_creep() {
    let _guard = crate::walls::TEST_LOCK.lock();
    let (x, y) = open_spot(90_000);
    let host = Host::new(vec![champ(0, (x, y), "tfm2_jojo_dio"), monster((x + 30_000, y))]);
    buff(&host, 0, "dio_lunge", 6);
    buff(&host, 1, "dio_target", 6);
    let mut d = crate::batch2::Dio::default();
    for t in 1..=160 {
        step(&host, &mut d, t);
    }
    assert!(hits_on(&host, 1) >= 1, "the Stand grabs and punches the camp monster");
}

#[test]
fn david_bursts_sandevistan_while_clearing_a_camp_when_calm() {
    let _guard = crate::walls::TEST_LOCK.lock();
    let (x, y) = open_spot(90_000);
    let host = Host::new(vec![champ(0, (x, y), "tfm2_cyberpunk_david"), monster((x + 20_000, y))]);
    buff(&host, 0, crate::FARM_BUFF, 14);
    let mut d = crate::batch2::David::default();
    step(&host, &mut d, 1);
    assert!(host.borrow().buffs[0].iter().any(|b| b.name() == "dv_sande"), "a Sandevistan burst on the camp");
    // an enemy champion near: no farming burst
    let host = Host::new(vec![champ(0, (x, y), "tfm2_cyberpunk_david"), monster((x + 20_000, y)), champ(1, (x + 60_000, y), "foe")]);
    buff(&host, 0, crate::FARM_BUFF, 14);
    let mut d = crate::batch2::David::default();
    step(&host, &mut d, 1);
    assert!(!host.borrow().buffs[0].iter().any(|b| b.name() == "dv_sande"));
}

#[test]
fn minato_throws_flying_raijin_at_a_creep_and_the_kunai_hits_it() {
    let _guard = crate::walls::TEST_LOCK.lock();
    let (x, y) = open_spot(90_000);
    let host = Host::new(vec![champ(0, (x, y), "tfm2_custom_minato"), monster((x + 50_000, y))]);
    buff(&host, 0, "raijin_throw", 6);
    buff(&host, 1, "raijin_target", 6);
    for t in 1..=90 {
        host.borrow_mut().tick = t;
        with_sim(&host, |sim| {
            let all = crate::champions_shared(sim);
            let m = all.iter().find(|c| c.id == 0).cloned().unwrap();
            crate::raijin::run(sim, &m, &all, t);
        });
    }
    assert!(hits_on(&host, 1) >= 1, "the middle kunai strikes the camp monster");
}
