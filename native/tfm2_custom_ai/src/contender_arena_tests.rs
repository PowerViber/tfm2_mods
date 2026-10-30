//! Real SDK wrapper calls against a fake host. These tests do NOT prove behaviour of the game executable.
use super::*;
type Point = (u64, u64);
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
    fn call<T>(&self, action: impl FnOnce(&mut StableSim) -> T) -> T {
        unsafe {
            let mut v: SimVtableV1 = std::mem::zeroed();
            v.size = size_of::<SimVtableV1>();
            v.tick = Some(tick);
            v.entity_count = Some(count);
            v.champion_count = Some(count);
            v.champion_id_at = Some(champion);
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
            action(&mut StableSim::from_raw(&mut raw).unwrap())
        }
    }
    fn has_ledger(&self, id: usize) -> bool {
        self.units.borrow()[&id]
            .buffs
            .iter()
            .any(|b| Ring::parse(b.name()).is_some())
    }
}
unsafe extern "C" fn champion(p: *const c_void, i: usize) -> usize {
    world(p)
        .units
        .borrow()
        .keys()
        .nth(i)
        .copied()
        .unwrap_or(usize::MAX)
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
        // a host that silently moves the unit elsewhere: readback must catch it
        (x.min(300000), y.min(300000))
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
fn transfers_real_wrapper_handles_and_returns_exact_integer_positions() {
    let w = Fake::new();
    let before = [w.position(1), w.position(9)];
    assert!(w.call(|s| enter(s, 1, 9, 480)));
    assert!(w.has_ledger(1) && w.has_ledger(9));
    assert_ne!(w.position(1), before[0]);
    w.tick.set(780);
    w.call(super::tick);
    assert_eq!([w.position(1), w.position(9)], before);
    assert!(!w.has_ledger(1));
}
#[test]
fn partial_transfer_and_clamped_host_roll_back() {
    for clamped in [false, true] {
        let w = Fake::new();
        let before = [w.position(1), w.position(9)];
        if clamped {
            w.clamp.set(true)
        } else {
            w.fail.set(Some(9))
        };
        assert!(!w.call(|s| enter(s, 1, 9, 480)));
        assert_eq!([w.position(1), w.position(9)], before);
        assert!(!w.has_ledger(1));
    }
}
#[test]
fn rejected_ledgers_prevent_any_transfer() {
    let w = Fake::new();
    w.retain_buffs.set(false);
    assert!(!w.call(|s| enter(s, 1, 9, 480)));
    assert!(w.calls.borrow().is_empty());
}
#[test]
fn survivor_returns_even_after_dead_owner_loses_all_buffs() {
    let w = Fake::new();
    let p = w.position(9);
    assert!(w.call(|s| enter(s, 1, 9, 480)));
    {
        let mut u = w.units.borrow_mut();
        let a = u.get_mut(&1).unwrap();
        a.alive = false;
        a.buffs.clear();
    }
    w.call(super::tick);
    assert_eq!(w.position(9), p);
    assert!(!w.has_ledger(9));
}
#[test]
fn reservations_reject_overlap_and_release_only_own_lock() {
    let w = Fake::new();
    assert!(w.call(|s| reserve(s, 1, 9, 39)));
    assert!(!w.call(|s| reserve(s, 1, 9, 39)));
    w.call(|s| release_reservation(s, 1, 9));
    assert!(w.call(|s| available(s, 1) && available(s, 9)));
}
#[test]
fn return_footprints_intersect_whole_rect_not_only_centre() {
    let w = Fake::new();
    let p = w.position(1);
    assert!(w.call(|s| enter(s, 1, 9, 480)));
    let q = crate::contender_core::Point {
        x: p.0 as i64,
        y: p.1 as i64,
    };
    assert!(w.call(|s| reserved_rect(
        s,
        crate::contender_core::Rect {
            min: q,
            max: crate::contender_core::Point {
                x: q.x + 90000,
                y: q.y + 2000
            }
        }
    )));
}
#[test]
fn two_simulations_do_not_share_ring_state() {
    let a = Fake::new();
    let b = Fake::new();
    assert!(a.call(|s| enter(s, 1, 9, 480)));
    assert!(b.call(|s| rings(s).is_empty()));
    a.tick.set(780);
    a.call(super::tick);
    assert!(b.call(|s| available(s, 9)));
}

#[test]
fn ring_stands_on_the_map_not_past_its_edge() {
    // The first live test placed both at x 1,120,000; the engine pulled them to the east edge for the whole duel.
    let w = Fake::new();
    assert!(w.call(|s| enter(s, 1, 9, 480)));
    let m = crate::contender_core::map();
    let ring = w.call(|s| member(s, 1)).unwrap();
    let b = ring.bounds();
    assert!(b.min.x >= m.min.x && b.min.y >= m.min.y && b.max.x <= m.max.x && b.max.y <= m.max.y, "{b:?}");
    for id in [1, 9] {
        let (x, y) = w.position(id);
        assert!(b.contains(crate::contender_core::Point { x: x as i64, y: y as i64 }, 0), "{id} at {x},{y}");
    }
    // a ring cast at the map's corner still fits inside it
    let corner = place([crate::contender_core::Point { x: 950_000, y: 950_000 }, crate::contender_core::Point { x: 940_000, y: 955_000 }]);
    let c = square(corner);
    assert!(c.max.x <= m.max.x && c.max.y <= m.max.y, "{c:?}");
}
#[test]
fn outsiders_are_moved_out_of_a_standing_ring() {
    let w = Fake::new();
    w.units.borrow_mut().insert(5, Unit { team: 0, pos: (0, 0), alive: true, buffs: vec![] });
    assert!(w.call(|s| enter(s, 1, 9, 480)));
    let b = w.call(|s| member(s, 1)).unwrap().bounds();
    let inside = (((b.min.x + b.max.x) / 2) as u64, ((b.min.y + b.max.y) / 2) as u64);
    w.units.borrow_mut().get_mut(&5).unwrap().pos = inside;
    w.tick.set(301);
    w.call(super::tick);
    let (x, y) = w.position(5);
    assert!(!b.contains(crate::contender_core::Point { x: x as i64, y: y as i64 }, -1), "still inside at {x},{y}");
    assert!(w.has_ledger(1) && w.has_ledger(9), "the duel goes on");
}
