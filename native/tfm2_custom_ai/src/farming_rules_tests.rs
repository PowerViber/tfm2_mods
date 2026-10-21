//! Round 139 farming rules on a small scripted world: collector radius, contested-only
//! jungle objectives, assigned objectives first, and the safety window.
use super::*;
use mod_api_stable::{
    BuffV1, EntityHandleV1 as E, EntityKindV1 as K, LaneV1, PlayerHandleV1 as P, SimCtxV1,
    SimVtableV1,
};
use std::{ffi::c_void, mem::size_of};

struct Unit {
    team: usize,
    kind: K,
    pos: (u64, u64),
    hp: (usize, usize),
    buffs: Vec<String>,
}
struct World {
    units: Vec<Unit>,
    jungle: usize,
}
unsafe fn world<'a>(p: *const c_void) -> &'a World {
    &*(p as *const World)
}
unsafe fn unit<'a>(p: *const c_void, e: E) -> Option<&'a Unit> {
    world(p).units.get(e.id()?)
}
unsafe extern "C" fn valid(p: *const c_void, e: E) -> bool {
    unit(p, e).is_some()
}
unsafe extern "C" fn alive(p: *const c_void, e: E) -> bool {
    unit(p, e).is_some_and(|u| u.hp.0 > 0)
}
unsafe extern "C" fn yes(p: *const c_void, e: E) -> bool {
    unit(p, e).is_some()
}
unsafe extern "C" fn team(p: *const c_void, e: E) -> usize {
    unit(p, e).map_or(9, |u| u.team)
}
unsafe extern "C" fn kind(p: *const c_void, e: E) -> u32 {
    unit(p, e).map_or(u32::MAX, |u| u.kind.code())
}
unsafe extern "C" fn minion(p: *const c_void, e: E) -> bool {
    unit(p, e).is_some_and(|u| u.kind == K::Minion)
}
unsafe extern "C" fn champion_flag(p: *const c_void, e: E) -> bool {
    unit(p, e).is_some_and(|u| u.kind == K::Champion)
}
unsafe extern "C" fn visible(_: *const c_void, _: usize, _: usize) -> bool {
    true
}
unsafe extern "C" fn pos(p: *const c_void, e: E, x: *mut u64, y: *mut u64) -> bool {
    let Some(u) = unit(p, e) else { return false };
    *x = u.pos.0;
    *y = u.pos.1;
    true
}
unsafe extern "C" fn hp(p: *const c_void, e: E, a: *mut usize, b: *mut usize) -> bool {
    let Some(u) = unit(p, e) else { return false };
    *a = u.hp.0;
    *b = u.hp.1;
    true
}
unsafe extern "C" fn buffs(p: *const c_void, e: E) -> usize {
    unit(p, e).map_or(0, |u| u.buffs.len())
}
unsafe extern "C" fn buff(p: *const c_void, e: E, i: usize, out: *mut BuffV1) -> bool {
    let Some(name) = unit(p, e).and_then(|u| u.buffs.get(i)) else { return false };
    *out = BuffV1::named(name);
    true
}
unsafe extern "C" fn entities(p: *const c_void) -> usize {
    world(p).units.len()
}
unsafe extern "C" fn entity_at(_: *const c_void, i: usize) -> E {
    E::from_id(i)
}
unsafe extern "C" fn champions(_: *const c_void) -> usize {
    4
}
unsafe extern "C" fn champion_at(_: *const c_void, i: usize) -> usize {
    i
}
unsafe extern "C" fn player_valid(_: *const c_void, h: P) -> bool {
    h.id().is_some_and(|id| id < 4)
}
unsafe extern "C" fn player_at(_: *const c_void, i: usize) -> P {
    P::from_id(i)
}
unsafe extern "C" fn player_champion(_: *const c_void, h: P) -> E {
    E::from_id(h.id().unwrap())
}
unsafe extern "C" fn lane(p: *const c_void, h: P) -> u32 {
    if h.id() == Some(world(p).jungle) {
        LaneV1::Jungle.code()
    } else {
        LaneV1::Mid.code()
    }
}
fn with<T>(w: &World, f: impl FnOnce(&StableSim) -> T) -> T {
    unsafe {
        let mut t: SimVtableV1 = std::mem::zeroed();
        t.size = size_of::<SimVtableV1>();
        t.entity_is_valid = Some(valid);
        t.entity_is_alive = Some(alive);
        t.entity_is_targetable = Some(yes);
        t.entity_team = Some(team);
        t.entity_kind = Some(kind);
        t.entity_is_minion = Some(minion);
        t.entity_is_champion = Some(champion_flag);
        t.is_visible = Some(visible);
        t.entity_pos = Some(pos);
        t.entity_hp = Some(hp);
        t.entity_buff_count = Some(buffs);
        t.entity_buff_at = Some(buff);
        t.entity_count = Some(entities);
        t.entity_at = Some(entity_at);
        t.champion_count = Some(champions);
        t.champion_id_at = Some(champion_at);
        t.player_count = Some(champions);
        t.player_at = Some(player_at);
        t.player_is_valid = Some(player_valid);
        t.player_champion = Some(player_champion);
        t.player_lane = Some(lane);
        let mut raw = SimCtxV1 {
            size: size_of::<SimCtxV1>(),
            sim: &t,
            frame: std::ptr::null(),
            state: w as *const World as *mut c_void,
        };
        f(&StableSim::from_raw(&mut raw).unwrap())
    }
}
fn champ(team: usize, x: u64) -> Unit {
    Unit { team, kind: K::Champion, pos: (x, 100000), hp: (1000, 1000), buffs: vec![] }
}
/// Units 0 and 1 are allies (team 0), 2 and 3 enemies far away, 4 a minion, 5 an Epic.
fn base() -> World {
    World {
        units: vec![
            champ(0, 100000),
            champ(0, 100000),
            champ(1, 900000),
            champ(1, 900000),
            Unit { team: 1, kind: K::Minion, pos: (110000, 100000), hp: (300, 300), buffs: vec![] },
            Unit { team: 2, kind: K::Epic, pos: (150000, 100000), hp: (20000, 20000), buffs: vec![] },
        ],
        jungle: 9,
    }
}
fn job(goal: crate::strategic_core::Goal, job: u8, target: usize, point: (i64, i64)) -> String {
    format!("ua_job:{}:{job}:{target}:{}:{}:50", goal.code(), point.0, point.1)
}

#[test]
fn a_wave_is_reserved_only_once_its_collector_is_near() {
    use crate::strategic_core::{Goal, Job};
    let mut w = base();
    w.units[1].buffs = vec![job(Goal::Wave, Job::CatchWave as u8, 0, (110000, 100000))];
    // The collector is still walking: both allies may clear the wave.
    w.units[1].pos = (110000 + RESERVE_RADIUS as u64 + 5000, 100000);
    with(&w, |sim| {
        assert!(Context::new(sim, 0).eligible(4));
        assert!(Context::new(sim, 1).eligible(4));
    });
    // The collector has arrived: the wave is theirs.
    w.units[1].pos = (110000 + RESERVE_RADIUS as u64 - 5000, 100000);
    with(&w, |sim| {
        assert!(!Context::new(sim, 0).eligible(4));
        assert!(Context::new(sim, 1).eligible(4));
    });
}

#[test]
fn a_jungler_joins_contested_objectives_but_opens_one_only_when_assigned() {
    use crate::strategic_core::Goal;
    let mut w = base();
    w.jungle = 0;
    with(&w, |sim| assert!(!Context::new(sim, 0).eligible(5), "untouched Epic, no job"));
    w.units[5].buffs = vec!["farm_objective_active:0".into()];
    with(&w, |sim| assert!(Context::new(sim, 0).eligible(5), "contested Epic"));
    w.units[5].buffs.clear();
    w.units[0].buffs = vec![job(Goal::Start, 0, 5, (150000, 100000))];
    with(&w, |sim| assert!(Context::new(sim, 0).eligible(5), "assigned to start it"));
    // A laner still needs the objective contested and an ally beside it.
    w.jungle = 9;
    w.units[0].buffs.clear();
    w.units[5].buffs = vec!["farm_objective_active:0".into()];
    w.units[1].pos = (400000, 100000);
    with(&w, |sim| assert!(!Context::new(sim, 0).eligible(5), "laner alone"));
    w.units[1].pos = (150000, 100000);
    with(&w, |sim| assert!(Context::new(sim, 0).eligible(5), "laner with an ally"));
}

#[test]
fn an_assigned_contest_goes_before_a_nearer_minion() {
    use crate::strategic_core::Goal;
    let mut w = base();
    w.jungle = 0;
    w.units[5].buffs = vec!["farm_objective_active:0".into()];
    with(&w, |sim| {
        let near = for_owner(sim, 0, 80000);
        assert_eq!(near.iter().map(|c| c.id).collect::<Vec<_>>(), vec![4, 5], "nearest first");
    });
    w.units[0].buffs = vec![job(Goal::Contest, 0, 5, (150000, 100000))];
    with(&w, |sim| {
        let near = for_owner(sim, 0, 80000);
        assert_eq!(near.iter().map(|c| c.id).collect::<Vec<_>>(), vec![5, 4], "objective first");
    });
    // A wave job does not reorder.
    w.units[0].buffs = vec![job(Goal::Wave, 5, 0, (110000, 100000))];
    with(&w, |sim| assert_eq!(for_owner(sim, 0, 80000)[0].id, 4));
}

#[test]
fn kits_farm_only_inside_the_safety_window() {
    let mut w = base();
    with(&w, |sim| assert!(Context::new(sim, 0).farmable(4)));
    // A visible enemy close by and no objective job: no farming on the kit's own initiative.
    w.units[2].pos = (120000, 100000);
    with(&w, |sim| {
        let c = Context::new(sim, 0);
        assert!(c.eligible(4), "still a legal target");
        assert!(!c.farmable(4));
    });
    w.units[2].pos = (900000, 100000);
    w.units[0].hp = (300, 1000);
    with(&w, |sim| assert!(!Context::new(sim, 0).farmable(4), "low health"));
}
