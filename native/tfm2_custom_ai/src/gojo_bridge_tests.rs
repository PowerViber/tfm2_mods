//! Stable-ABI fixture, not a claim about a live engine run.
use super::*;
use mod_api_stable::{BuffV1, EntityHandleV1 as E, SimCtxV1, SimVtableV1};
use std::{ffi::c_void, mem::size_of};
#[derive(Default)]
struct Host {
    buffs: Vec<BuffV1>,
    damage: Vec<(usize, usize, usize, usize, u32)>,
    shields: Vec<usize>,
    clear_shields: usize,
}
unsafe extern "C" fn remove(p: *mut c_void, _: E, name: *const u8, n: usize) -> usize {
    let h = &mut *(p as *mut Host);
    let name = std::str::from_utf8(std::slice::from_raw_parts(name, n)).unwrap();
    let before = h.buffs.len();
    h.buffs.retain(|b| b.name() != name);
    before - h.buffs.len()
}
unsafe extern "C" fn shield(p: *mut c_void, _: E, n: usize, _: usize) -> bool {
    let h = &mut *(p as *mut Host);
    h.shields.push(n);
    true
}
unsafe extern "C" fn clear_shields(p: *mut c_void, _: E) -> usize {
    let h = &mut *(p as *mut Host);
    h.clear_shields += 1;
    h.shields.clear();
    1
}

#[test]
fn infinity_pause_keeps_existing_and_allied_layers_without_republishing() {
    unsafe {
        let mut h = Host {
            shields: vec![125],
            ..Default::default()
        };
        let mut table: SimVtableV1 = std::mem::zeroed();
        table.size = size_of::<SimVtableV1>();
        table.entity_is_valid = Some(valid);
        table.add_buff = Some(add);
        table.entity_remove_buff = Some(remove);
        table.entity_add_shield = Some(shield);
        table.entity_clear_shield = Some(clear_shields);
        let mut raw = SimCtxV1 {
            size: size_of::<SimCtxV1>(),
            sim: &table,
            frame: std::ptr::null(),
            state: &mut h as *mut Host as *mut c_void,
        };
        let mut sim = StableSim::from_raw(&mut raw).unwrap();
        let mut g = Gojo {
            owner: 1,
            ..Default::default()
        };
        g.infinity(&mut sim, true, 230);
        assert_eq!(h.shields, vec![125, 230]);
        assert_eq!(h.buffs.len(), 1);
        h.shields[1] = 90;
        g.infinity(&mut sim, false, 90);
        assert_eq!(h.shields, vec![125, 90]);
        assert!(h.buffs.is_empty());
        g.infinity(&mut sim, true, 90);
        assert_eq!(h.shields, vec![125, 90]);
        assert_eq!(h.clear_shields, 0);
    }
}
unsafe extern "C" fn valid(_: *const c_void, e: E) -> bool {
    e.id().is_some_and(|n| n == 1 || n == 2)
}
unsafe extern "C" fn tick(_: *const c_void) -> usize {
    0
}
unsafe extern "C" fn count(p: *const c_void, _: E) -> usize {
    (*(p as *const Host)).buffs.len()
}
unsafe extern "C" fn buff(p: *const c_void, _: E, i: usize, out: *mut BuffV1) -> bool {
    if let Some(b) = (&(*(p as *const Host)).buffs).get(i) {
        *out = *b;
        true
    } else {
        false
    }
}
unsafe extern "C" fn add(p: *mut c_void, _: usize, b: *const BuffV1) {
    (*(p as *mut Host)).buffs.push(*b)
}
unsafe extern "C" fn damage(p: *mut c_void, a: usize, t: usize, ad: usize, ap: usize, k: u32) {
    (*(p as *mut Host)).damage.push((a, t, ad, ap, k))
}
#[test]
fn native_impact_is_an_inbox_not_a_second_damage_effect() {
    unsafe {
        let mut h = Host::default();
        let mut table: SimVtableV1 = std::mem::zeroed();
        table.size = size_of::<SimVtableV1>();
        table.entity_is_valid = Some(valid);
        table.entity_buff_count = Some(count);
        table.entity_buff_at = Some(buff);
        table.tick = Some(tick);
        table.add_buff = Some(add);
        table.deal_damage = Some(damage);
        let mut raw = SimCtxV1 {
            size: size_of::<SimCtxV1>(),
            sim: &table,
            frame: std::ptr::null(),
            state: &mut h as *mut Host as *mut c_void,
        };
        let mut sim = StableSim::from_raw(&mut raw).unwrap();
        BasicImpact.apply(&mut sim, 0, 1, InputTargetV1::target(2));
        BasicImpact.apply(&mut sim, 0, 1, InputTargetV1::target(2));
        assert_eq!(h.buffs.len(), 1);
        assert!(h.damage.is_empty());
        let mut g = Gojo {
            owner: 1,
            ..Default::default()
        };
        g.execute(
            &mut sim,
            vec![Cmd::Damage {
                target: 2,
                ad: 84,
                ap: 25,
                basic: true,
                event: 1,
            }],
        );
        assert_eq!(
            h.damage,
            vec![(1, 2, 84, 25, AttackTypeV1::BaseAttack.code())]
        );
        let before = h.damage.clone();
        BasicImpact.apply(&mut sim, 0, 1, InputTargetV1::NONE);
        assert_eq!(h.damage, before);
    }
}
