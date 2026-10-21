//! Frieren's basic attack reaches structures with the same scaled formula.
use super::*;
use mod_api_stable::{EntityHandleV1 as E, EntityKindV1 as K, InputTargetKindV1, InputTargetV1, SimCtxV1, SimVtableV1, StableEffectType};
use std::{cell::RefCell, ffi::c_void, mem::size_of};
struct Host {
    target_kind: K,
    hits: RefCell<Vec<(usize, usize, usize)>>,
}
unsafe extern "C" fn valid(_: *const c_void, _: E) -> bool {
    true
}
unsafe extern "C" fn alive(_: *const c_void, _: E) -> bool {
    true
}
unsafe extern "C" fn targetable(_: *const c_void, _: E) -> bool {
    true
}
unsafe extern "C" fn team(_: *const c_void, e: E) -> usize {
    usize::from(e.id() != Some(1))
}
unsafe extern "C" fn kind(p: *const c_void, e: E) -> u32 {
    if e.id() == Some(1) {
        K::Champion.code()
    } else {
        (*(p as *const Host)).target_kind.code()
    }
}
unsafe extern "C" fn stat(_: *const c_void, _: E, out: *mut StatV1) -> bool {
    *out = StatV1 { magic_power: 100, ..StatV1::default() };
    true
}
unsafe extern "C" fn damage(p: *mut c_void, _: usize, target: usize, ad: usize, ap: usize, _: u32) {
    (*(p as *const Host)).hits.borrow_mut().push((target, ad, ap));
}
fn hit(target_kind: K) -> Vec<(usize, usize, usize)> {
    let h = Host { target_kind, hits: RefCell::new(vec![]) };
    unsafe {
        let mut t: SimVtableV1 = std::mem::zeroed();
        t.size = size_of::<SimVtableV1>();
        t.entity_is_valid = Some(valid);
        t.entity_is_alive = Some(alive);
        t.entity_is_targetable = Some(targetable);
        t.entity_team = Some(team);
        t.entity_kind = Some(kind);
        t.entity_stat = Some(stat);
        t.deal_damage = Some(damage);
        let mut raw = SimCtxV1 { size: size_of::<SimCtxV1>(), sim: &t, frame: std::ptr::null(), state: &h as *const Host as *mut c_void };
        let mut sim = StableSim::from_raw(&mut raw).unwrap();
        let input = InputTargetV1 { kind: InputTargetKindV1::Target.code(), target_id: 2, ..InputTargetV1::default() };
        BasicDamage.apply(&mut sim, 0, 1, input);
    }
    h.hits.into_inner()
}
#[test]
fn basic_attack_hits_towers_and_the_nexus_like_any_target() {
    let expected = BasicDamage.expected_damage(&StatV1 { magic_power: 100, ..StatV1::default() }).1;
    assert_eq!(expected, (20 + 45) * crate::balance_data::FRIEREN_DAMAGE_PERCENT as usize / 100);
    for k in [K::Tower, K::Nexus, K::Minion, K::Champion] {
        assert_eq!(hit(k), vec![(2, 0, expected)], "{k:?}");
    }
}
