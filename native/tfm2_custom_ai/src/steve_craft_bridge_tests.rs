//! Stable API fixture. Actual engine/carrier acceptance is recorded separately.
use super::*;
use mod_api_stable::{BuffV1, EntityHandleV1 as E, SimCtxV1, SimVtableV1};
use std::{ffi::c_void, mem::size_of};
#[derive(Default)]
struct Host {
    hp: usize,
    damage: Vec<(usize, usize, usize, usize, u32)>,
    buffs: Vec<BuffV1>,
    immune: bool,
}
unsafe extern "C" fn valid(_: *const c_void, e: E) -> bool {
    e.id().is_some_and(|v| v < 3)
}
unsafe extern "C" fn alive(p: *const c_void, e: E) -> bool {
    e.id() != Some(2) || (*(p as *const Host)).hp > 0
}
unsafe extern "C" fn targetable(_: *const c_void, _: E) -> bool {
    true
}
unsafe extern "C" fn team(_: *const c_void, e: E) -> usize {
    if e.id() == Some(1) {
        0
    } else {
        1
    }
}
unsafe extern "C" fn stat(_: *const c_void, _: E, out: *mut StatV1) -> bool {
    *out = StatV1 {
        attack: 62,
        ..Default::default()
    };
    true
}
unsafe extern "C" fn hp(p: *const c_void, e: E, current: *mut usize, max: *mut usize) -> bool {
    *current = if e.id() == Some(2) {
        (*(p as *const Host)).hp
    } else {
        1150
    };
    *max = 1150;
    true
}
unsafe extern "C" fn count(p: *const c_void, _: E) -> usize {
    (*(p as *const Host)).buffs.len()
}
unsafe extern "C" fn buff(p: *const c_void, _: E, n: usize, out: *mut BuffV1) -> bool {
    if let Some(b) = (&(*(p as *const Host)).buffs).get(n) {
        *out = *b;
        true
    } else {
        false
    }
}
unsafe extern "C" fn add(p: *mut c_void, _: usize, b: *const BuffV1) {
    (*(p as *mut Host)).buffs.push(*b);
}
unsafe extern "C" fn damage(p: *mut c_void, by: usize, to: usize, ad: usize, ap: usize, kind: u32) {
    let h = &mut *(p as *mut Host);
    h.damage.push((by, to, ad, ap, kind));
    if !h.immune {
        h.hp = h.hp.saturating_sub(ad + ap);
    }
}
#[test]
fn primary_impact_has_one_base_attack_and_bonus_has_no_second_proc_identity() {
    unsafe {
        let mut host = Host {
            hp: 1150,
            ..Default::default()
        };
        let mut table: SimVtableV1 = std::mem::zeroed();
        table.size = size_of::<SimVtableV1>();
        table.entity_is_valid = Some(valid);
        table.entity_is_alive = Some(alive);
        table.entity_is_targetable = Some(targetable);
        table.entity_team = Some(team);
        table.entity_stat = Some(stat);
        table.entity_hp = Some(hp);
        table.entity_buff_count = Some(count);
        table.entity_buff_at = Some(buff);
        table.add_buff = Some(add);
        table.deal_damage = Some(damage);
        let mut raw = SimCtxV1 {
            size: size_of::<SimCtxV1>(),
            sim: &table,
            frame: std::ptr::null(),
            state: &mut host as *mut Host as *mut c_void,
        };
        let mut sim = StableSim::from_raw(&mut raw).unwrap();
        BasicImpact.apply(&mut sim, 0, 1, InputTargetV1::target(2));
        BasicImpact.apply(&mut sim, 0, 1, InputTargetV1::target(2));
        assert_eq!(host.damage.len(), 1);
        assert_eq!(
            host.damage[0],
            (1, 2, 73, 0, AttackTypeV1::BaseAttack.code())
        );
        assert_eq!(host.buffs.len(), 1);
        let mut adapter = Steve {
            owner: 1,
            ..Default::default()
        };
        adapter.execute(
            &mut sim,
            vec![Cmd::Damage {
                event: 1,
                target: 2,
                ad: 41,
                ap: 0,
                utility: false,
            }],
        );
        assert_eq!(host.damage.len(), 2);
        assert_eq!(host.damage[1].4, AttackTypeV1::Skill.code());
        BasicImpact.apply(&mut sim, 0, 1, InputTargetV1::NONE);
        assert_eq!(host.damage.len(), 2);
    }
}
#[test]
fn invulnerable_impact_never_enqueues_qualifying_hit() {
    unsafe {
        let mut host = Host {
            hp: 1150,
            immune: true,
            ..Default::default()
        };
        let mut table: SimVtableV1 = std::mem::zeroed();
        table.size = size_of::<SimVtableV1>();
        table.entity_is_valid = Some(valid);
        table.entity_is_alive = Some(alive);
        table.entity_is_targetable = Some(targetable);
        table.entity_team = Some(team);
        table.entity_stat = Some(stat);
        table.entity_hp = Some(hp);
        table.entity_buff_count = Some(count);
        table.entity_buff_at = Some(buff);
        table.add_buff = Some(add);
        table.deal_damage = Some(damage);
        let mut raw = SimCtxV1 {
            size: size_of::<SimCtxV1>(),
            sim: &table,
            frame: std::ptr::null(),
            state: &mut host as *mut Host as *mut c_void,
        };
        let mut sim = StableSim::from_raw(&mut raw).unwrap();
        BasicImpact.apply(&mut sim, 0, 1, InputTargetV1::target(2));
        assert_eq!(host.hp, 1150);
        assert!(host.buffs.is_empty());
    }
}
