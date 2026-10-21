//! Exact policy equivalence and bounded synthetic host-call benchmarks.
use super::*;
use mod_api_stable::{
    BuffV1, EntityHandleV1 as E, EntityKindV1 as K, LaneV1, PlayerHandleV1 as P, SimCtxV1,
    SimVtableV1,
};
use std::{cell::Cell, ffi::c_void, mem::size_of, time::Instant};
struct Host {
    calls: Cell<usize>,
    jungle: bool,
    variant: usize,
}
unsafe fn host<'a>(p: *const c_void) -> &'a Host {
    let h = &*(p as *const Host);
    h.calls.set(h.calls.get() + 1);
    h
}
unsafe extern "C" fn valid(p: *const c_void, h: E) -> bool {
    host(p);
    h.id().is_some_and(|id| id < 310)
}
unsafe extern "C" fn player_valid(p: *const c_void, h: P) -> bool {
    host(p);
    h.id().is_some_and(|id| id < 10)
}
unsafe extern "C" fn alive(p: *const c_void, e: E) -> bool {
    let h = host(p);
    e.id()
        .is_some_and(|id| id < 10 || (id + h.variant) % 19 != 0)
}
unsafe extern "C" fn targetable(p: *const c_void, e: E) -> bool {
    let h = host(p);
    (e.id().unwrap() + h.variant) % 23 != 0
}
unsafe extern "C" fn team(p: *const c_void, e: E) -> usize {
    host(p);
    let id = e.id().unwrap();
    if id < 10 {
        usize::from(id >= 5)
    } else if id % 6 < 2 {
        1
    } else {
        2
    }
}
unsafe extern "C" fn visible(p: *const c_void, _: usize, id: usize) -> bool {
    let h = host(p);
    (id + h.variant) % 17 != 0
}
unsafe extern "C" fn kind(p: *const c_void, e: E) -> u32 {
    host(p);
    let id = e.id().unwrap();
    if id < 10 {
        return K::Champion.code();
    }
    match id % 6 {
        0 => K::Minion.code(),
        1 => K::Summon.code(),
        2 => K::Jungle.code(),
        3 => K::Epic.code(),
        4 => K::Serpen.code(),
        _ => u32::MAX,
    }
}
unsafe extern "C" fn minion(p: *const c_void, e: E) -> bool {
    host(p);
    e.id().unwrap() % 6 == 5
}
unsafe extern "C" fn count(p: *const c_void) -> usize {
    host(p);
    10
}
unsafe extern "C" fn at(p: *const c_void, i: usize) -> usize {
    host(p);
    i
}
unsafe extern "C" fn player_at(p: *const c_void, i: usize) -> P {
    host(p);
    P::from_id(i)
}
unsafe extern "C" fn champion(p: *const c_void, h: P) -> E {
    host(p);
    E::from_id(h.id().unwrap())
}
unsafe extern "C" fn lane(p: *const c_void, _: P) -> u32 {
    let h = host(p);
    if h.jungle {
        LaneV1::Jungle.code()
    } else {
        LaneV1::Mid.code()
    }
}
unsafe extern "C" fn pos(p: *const c_void, e: E, x: *mut u64, y: *mut u64) -> bool {
    let h = host(p);
    *x = 100000 + (e.id().unwrap() % 10) as u64 * 5000;
    *y = 100000 + h.variant as u64 * 1000;
    true
}
unsafe extern "C" fn buffs(p: *const c_void, e: E) -> usize {
    let h = host(p);
    usize::from((e.id().unwrap() + h.variant) % 3 != 0)
}
unsafe extern "C" fn buff(p: *const c_void, _: E, _: usize, out: *mut BuffV1) -> bool {
    host(p);
    // Team 0 marks its objective; team 1 does not, so both outcomes are exercised.
    *out = BuffV1::named("farm_objective_active:0");
    true
}
fn with_sim<T>(jungle: bool, variant: usize, f: impl FnOnce(&StableSim, &Host) -> T) -> T {
    unsafe {
        let h = Host {
            calls: Cell::new(0),
            jungle,
            variant,
        };
        let mut table: SimVtableV1 = std::mem::zeroed();
        table.size = size_of::<SimVtableV1>();
        table.entity_is_valid = Some(valid);
        table.entity_is_alive = Some(alive);
        table.entity_is_targetable = Some(targetable);
        table.entity_team = Some(team);
        table.entity_kind = Some(kind);
        table.entity_is_minion = Some(minion);
        table.is_visible = Some(visible);
        table.entity_pos = Some(pos);
        table.entity_buff_count = Some(buffs);
        table.entity_buff_at = Some(buff);
        table.champion_count = Some(count);
        table.champion_id_at = Some(at);
        table.player_count = Some(count);
        table.player_at = Some(player_at);
        table.player_is_valid = Some(player_valid);
        table.player_champion = Some(champion);
        table.player_lane = Some(lane);
        let mut raw = SimCtxV1 {
            size: size_of::<SimCtxV1>(),
            sim: &table,
            frame: std::ptr::null(),
            state: &h as *const Host as *mut c_void,
        };
        let sim = StableSim::from_raw(&mut raw).unwrap();
        f(&sim, &h)
    }
}
include!("query_reference.rs");
#[test]
fn cached_farming_decisions_match_the_original_across_roles_visibility_and_worlds() {
    for jungle in [false, true] {
        for variant in 0..24 {
            with_sim(jungle, variant, |sim, _| {
                for owner in [0, 5] {
                    let context = farming::Context::new(sim, owner);
                    for target in 0..315 {
                        assert_eq!(
                            reference_eligible(sim, owner, target),
                            context.eligible(target),
                            "role {jungle} world {variant} owner {owner} target {target}"
                        );
                    }
                }
            });
        }
    }
}
#[test]
#[ignore = "Explicit five-run performance comparison; not a correctness gate"]
fn query_benchmark() {
    with_sim(true, 2, |sim, host| {
        let mut times = [vec![], vec![]];
        let mut calls = [0, 0];
        let n = 1000;
        for iteration in 0..n {
            for k in [iteration % 2, 1 - iteration % 2] {
                host.calls.set(0);
                let at = Instant::now();
                let count = if k == 0 {
                    (10..310)
                        .filter(|id| std::hint::black_box(reference_eligible(sim, 0, *id)))
                        .count()
                } else {
                    let c = farming::Context::new(sim, 0);
                    (10..310)
                        .filter(|id| std::hint::black_box(c.eligible(*id)))
                        .count()
                };
                std::hint::black_box(count);
                times[k].push(at.elapsed().as_nanos() as u64);
                calls[k] = host.calls.get();
            }
        }
        for a in &mut times {
            a.sort_unstable();
        }
        let row = |i: usize| serde_json::json!({"medianNs":times[i][n/2],"p95Ns":times[i][n*95/100],"p99Ns":times[i][n*99/100],"hostCalls":calls[i]});
        println!(
            "QUERY_BENCH {}",
            serde_json::json!({"targets":300,"samples":n,"baseline":row(0),"candidate":row(1)})
        );
        assert!(calls[1] < calls[0]);
    });
}
