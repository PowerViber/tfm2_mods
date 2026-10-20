//! Stable-ABI adapter fixtures. These are separate from live-engine acceptance.
use super::*;
use mod_api_stable::{EntityHandleV1, SimCtxV1, SimVtableV1};
use std::{ffi::c_void, mem::size_of};
#[derive(Default)]
struct Host {
    fail_spawn: bool,
    next: usize,
    spawns: Vec<(usize, u64, u64)>,
    removed: Vec<usize>,
    cc: Vec<(usize, u32)>,
    views: Vec<String>,
}
unsafe extern "C" fn spawn(
    p: *mut c_void,
    _: *const u8,
    _: usize,
    owner: usize,
    _: usize,
    x: u64,
    y: u64,
    _: u64,
    _: *const StatV1,
    _: *const UnitAttackV1,
) -> usize {
    let h = &mut *(p as *mut Host);
    if h.fail_spawn {
        return usize::MAX;
    }
    h.next += 1;
    h.spawns.push((owner, x, y));
    h.next
}
unsafe extern "C" fn cc(p: *mut c_void, id: usize, c: *const CcV1) {
    (*(p as *mut Host)).cc.push((id, (*c).kind));
}
unsafe extern "C" fn hp(p: *mut c_void, id: EntityHandleV1, _: usize) -> bool {
    (*(p as *mut Host)).removed.push(id.id().unwrap());
    true
}
unsafe extern "C" fn view(
    p: *mut c_void,
    n: *const u8,
    len: usize,
    _: usize,
    _: *const InputTargetV1,
    _: u64,
    _: u64,
    _: u64,
) -> bool {
    (*(p as *mut Host))
        .views
        .push(String::from_utf8_lossy(std::slice::from_raw_parts(n, len)).into());
    true
}
fn table() -> SimVtableV1 {
    unsafe {
        let mut t: SimVtableV1 = std::mem::zeroed();
        t.size = size_of::<SimVtableV1>();
        t.spawn_unit = Some(spawn);
        t.apply_cc = Some(cc);
        t.entity_set_hp = Some(hp);
        t.play_view_effect = Some(view);
        t
    }
}
fn champion(id: usize) -> Champ {
    Champ {
        id,
        team: 0,
        x: 100000,
        y: 100000,
        hp: 1000,
        max_hp: 1000,
        attack: 100,
        name: OMEN_ID.into(),
        buffs: vec![],
        stunned: false,
        pushed: false,
    }
}
#[test]
fn failed_shade_never_creates_an_invisible_teleport() {
    unsafe {
        let mut h = Host {
            fail_spawn: true,
            ..Default::default()
        };
        let t = table();
        let mut raw = SimCtxV1 {
            size: size_of::<SimCtxV1>(),
            sim: &t,
            frame: std::ptr::null(),
            state: &mut h as *mut _ as *mut c_void,
        };
        let mut sim = StableSim::from_raw(&mut raw).unwrap();
        let mut o = Omen::reworked();
        assert!(!o.begin_ultimate(&mut sim, &champion(1), (600000, 500000), 10));
        assert!(o.ult.is_none());
        assert!(h.views.is_empty());
        assert!(o.clock.ult_ready(10));
    }
}
#[test]
fn committed_destination_and_owned_shade_are_independent() {
    unsafe {
        let mut h = Host {
            next: 10,
            ..Default::default()
        };
        let t = table();
        let mut raw = SimCtxV1 {
            size: size_of::<SimCtxV1>(),
            sim: &t,
            frame: std::ptr::null(),
            state: &mut h as *mut _ as *mut c_void,
        };
        let mut sim = StableSim::from_raw(&mut raw).unwrap();
        let mut a = Omen::reworked();
        let mut b = Omen::reworked();
        assert!(a.begin_ultimate(&mut sim, &champion(1), (600000, 500000), 10));
        assert!(b.begin_ultimate(&mut sim, &champion(2), (650000, 550000), 10));
        let u = a.ult.clone().unwrap();
        assert_eq!(
            (u.x, u.y, u.t_end, u.shade),
            (600000, 500000, 130, Some(11))
        );
        a.on_dead(&mut sim, 0);
        assert!(a.ult.is_none());
        assert_eq!(h.removed, vec![11]);
        assert_eq!(b.ult.unwrap().shade, Some(12));
        assert_eq!(h.spawns, vec![(1, 600000, 500000), (2, 650000, 550000)]);
    }
}
#[test]
fn step_has_one_channel_and_does_not_bind_an_enemy() {
    unsafe {
        let mut h = Host::default();
        let t = table();
        let mut raw = SimCtxV1 {
            size: size_of::<SimCtxV1>(),
            sim: &t,
            frame: std::ptr::null(),
            state: &mut h as *mut _ as *mut c_void,
        };
        let mut sim = StableSim::from_raw(&mut raw).unwrap();
        let mut o = Omen::reworked();
        o.begin_step(&mut sim, &champion(1), (140000, 100000), 20);
        assert_eq!(o.step, Some((140000, 100000, 56)));
        assert_eq!(o.cast_until, 56);
        assert!(h.cc.iter().all(|(id, _)| *id == 1));
        assert_eq!(h.views.len(), 2);
    }
}
