//! Real SDK wrapper calls against a fake host. These tests do NOT prove behaviour of the game executable.
use super::*;
use mod_api_stable::{EntityHandleV1 as E, SimCtxV1, SimVtableV1};
use std::{
    cell::{Cell, RefCell},
    ffi::c_void,
    mem::size_of,
};

#[derive(Clone)]
struct Unit {
    team: usize,
    pos: Point,
    alive: bool,
    buffs: Vec<BuffV1>,
}
struct Fake {
    tick: Cell<usize>,
    units: RefCell<BTreeMap<usize, Unit>>,
    fail: Cell<Option<usize>>,
    clamp: Cell<bool>,
    retain_buffs: Cell<bool>,
    calls: RefCell<Vec<(usize, Point)>>,
}
impl Fake {
    fn new() -> Self {
        Self {
            tick: Cell::new(300),
            units: RefCell::new(
                [
                    (
                        1,
                        Unit {
                            team: 0,
                            pos: (123457, 234569),
                            alive: true,
                            buffs: vec![],
                        },
                    ),
                    (
                        9,
                        Unit {
                            team: 1,
                            pos: (654321, 765431),
                            alive: true,
                            buffs: vec![],
                        },
                    ),
                ]
                .into(),
            ),
            fail: Cell::new(None),
            clamp: Cell::new(false),
            retain_buffs: Cell::new(true),
            calls: RefCell::new(vec![]),
        }
    }
    fn position(&self, id: usize) -> Point {
        self.units.borrow()[&id].pos
    }
    fn step(&self) {
        unsafe {
            let mut v: SimVtableV1 = std::mem::zeroed();
            v.size = size_of::<SimVtableV1>();
            v.tick = Some(tick);
            v.entity_count = Some(count);
            v.entity_at = Some(at);
            v.entity_is_valid = Some(valid);
            v.entity_is_alive = Some(alive);
            v.entity_is_champion = Some(valid);
            v.entity_team = Some(team);
            v.entity_pos = Some(pos);
            v.entity_set_pos = Some(set_pos);
            v.entity_buff_count = Some(buff_count);
            v.entity_buff_at = Some(buff_at);
            v.add_buff = Some(add_buff);
            v.entity_remove_buff = Some(remove_buff);
            let mut raw = SimCtxV1 {
                size: size_of::<SimCtxV1>(),
                sim: &v,
                frame: std::ptr::null(),
                state: self as *const Self as *mut c_void,
            };
            Probe.on_match_tick(&mut StableSim::from_raw(&mut raw).unwrap(), 0);
        }
    }
    fn has_ledger(&self, id: usize) -> bool {
        self.units.borrow()[&id]
            .buffs
            .iter()
            .any(|b| Pair::parse(b.name()).is_some())
    }
}
unsafe fn world<'a>(p: *const c_void) -> &'a Fake {
    &*(p as *const Fake)
}
unsafe extern "C" fn tick(p: *const c_void) -> usize {
    world(p).tick.get()
}
unsafe extern "C" fn count(p: *const c_void) -> usize {
    world(p).units.borrow().len()
}
unsafe extern "C" fn at(p: *const c_void, i: usize) -> E {
    world(p)
        .units
        .borrow()
        .keys()
        .nth(i)
        .copied()
        .map(E::from_id)
        .unwrap_or(E::NULL)
}
unsafe extern "C" fn valid(p: *const c_void, e: E) -> bool {
    e.id()
        .is_some_and(|id| world(p).units.borrow().contains_key(&id))
}
unsafe extern "C" fn alive(p: *const c_void, e: E) -> bool {
    e.id()
        .and_then(|id| world(p).units.borrow().get(&id).map(|u| u.alive))
        .unwrap_or(false)
}
unsafe extern "C" fn team(p: *const c_void, e: E) -> usize {
    world(p).units.borrow()[&e.id().unwrap()].team
}
unsafe extern "C" fn pos(p: *const c_void, e: E, x: *mut u64, y: *mut u64) -> bool {
    let w = world(p);
    let u = w.units.borrow();
    let Some(u) = e.id().and_then(|id| u.get(&id)) else {
        return false;
    };
    *x = u.pos.0;
    *y = u.pos.1;
    true
}
unsafe extern "C" fn set_pos(p: *mut c_void, e: E, x: u64, y: u64) -> bool {
    let w = world(p);
    let Some(id) = e.id() else { return false };
    w.calls.borrow_mut().push((id, (x, y)));
    if w.fail.get() == Some(id) {
        return false;
    }
    let mut u = w.units.borrow_mut();
    let Some(u) = u.get_mut(&id) else {
        return false;
    };
    u.pos = if w.clamp.get() {
        (x.min(960000), y.min(960000))
    } else {
        (x, y)
    };
    true
}
unsafe extern "C" fn buff_count(p: *const c_void, e: E) -> usize {
    world(p).units.borrow()[&e.id().unwrap()].buffs.len()
}
unsafe extern "C" fn buff_at(p: *const c_void, e: E, i: usize, out: *mut BuffV1) -> bool {
    let w = world(p);
    let u = w.units.borrow();
    let Some(b) = u[&e.id().unwrap()].buffs.get(i) else {
        return false;
    };
    *out = *b;
    true
}
unsafe extern "C" fn add_buff(p: *mut c_void, id: usize, b: *const BuffV1) {
    let w = world(p);
    if w.retain_buffs.get() {
        if let Some(u) = w.units.borrow_mut().get_mut(&id) {
            u.buffs.push(*b)
        }
    }
}
unsafe extern "C" fn remove_buff(p: *mut c_void, e: E, n: *const u8, len: usize) -> usize {
    let name = std::slice::from_raw_parts(n, len);
    let w = world(p);
    let mut u = w.units.borrow_mut();
    let Some(u) = e.id().and_then(|id| u.get_mut(&id)) else {
        return 0;
    };
    let before = u.buffs.len();
    u.buffs.retain(|b| b.name().as_bytes() != name);
    before - u.buffs.len()
}

#[test]
fn actual_wrapper_path_transfers_and_returns_exactly_on_timeout() {
    let w = Fake::new();
    let original = [w.position(1), w.position(9)];
    w.step();
    assert_eq!([w.position(1), w.position(9)], starts());
    assert!(w.has_ledger(1));
    w.tick.set(779);
    w.step();
    assert_eq!([w.position(1), w.position(9)], starts());
    w.tick.set(780);
    w.step();
    assert_eq!([w.position(1), w.position(9)], original);
    assert!(!w.has_ledger(1));
    assert!(!w.has_ledger(9));
}
#[test]
fn second_transfer_failure_restores_first_and_retains_no_success_claim() {
    let w = Fake::new();
    let original = w.position(1);
    w.fail.set(Some(9));
    w.step();
    assert_eq!(w.position(1), original);
    assert!(!w.has_ledger(1));
    w.fail.set(None);
    w.tick.set(301);
    w.step();
    assert!(!w.has_ledger(1));
}
#[test]
fn clamp_readback_aborts_instead_of_claiming_off_map_success() {
    let w = Fake::new();
    let original = [w.position(1), w.position(9)];
    w.clamp.set(true);
    w.step();
    assert_eq!([w.position(1), w.position(9)], original);
    assert!(!w.has_ledger(1));
}
#[test]
fn never_teleports_without_both_persistent_ledgers() {
    let w = Fake::new();
    w.retain_buffs.set(false);
    w.step();
    assert!(w.calls.borrow().is_empty());
}
#[test]
fn survivor_returns_after_partner_death_even_if_dead_partner_loses_buffs() {
    let w = Fake::new();
    let original = w.position(9);
    w.step();
    {
        let mut u = w.units.borrow_mut();
        u.get_mut(&1).unwrap().alive = false;
        u.get_mut(&1).unwrap().buffs.clear();
    }
    w.calls.borrow_mut().clear();
    w.tick.set(330);
    w.step();
    assert_eq!(w.position(9), original);
    assert_eq!(w.calls.borrow().as_slice(), &[(9, original)]);
}
#[test]
fn host_escape_aborts_probe_and_restores_both_instead_of_enforcing_a_fake_ring() {
    let w = Fake::new();
    let original = [w.position(1), w.position(9)];
    w.step();
    w.units.borrow_mut().get_mut(&1).unwrap().pos = (960000, 240000);
    w.tick.set(301);
    w.step();
    assert_eq!([w.position(1), w.position(9)], original);
}
#[test]
fn duplicate_callback_cannot_start_a_second_trial() {
    let w = Fake::new();
    w.step();
    let n = w.calls.borrow().len();
    w.step();
    assert_eq!(w.calls.borrow().len(), n);
}
#[test]
fn callbacks_from_two_simulations_remain_independent() {
    let a = Fake::new();
    let b = Fake::new();
    b.units.borrow_mut().get_mut(&1).unwrap().pos = (101, 203);
    a.step();
    assert_eq!(b.position(1), (101, 203));
    b.step();
    a.tick.set(780);
    b.tick.set(780);
    a.step();
    b.step();
    assert_eq!(a.position(1), (123457, 234569));
    assert_eq!(b.position(1), (101, 203));
}
