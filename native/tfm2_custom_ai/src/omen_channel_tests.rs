//! Round 139 Omen fixes: channel interrupts, crowd-control immunity, landing checks and the Paranoia wave clear.
use super::*;
use mod_api_stable::{EntityHandleV1 as E, EntityKindV1 as K, SimCtxV1, SimVtableV1};
use std::{cell::RefCell, ffi::c_void, mem::size_of};

/// A small world: entity 1 is Omen (team 0); the rest are units with a kind.
struct World {
    units: Vec<(usize, usize, u64, u64, K)>,
    cc: Vec<CcV1>,
    moved: RefCell<Vec<(usize, u64, u64)>>,
}
unsafe fn w<'a>(p: *const c_void) -> &'a World {
    &*(p as *const World)
}
unsafe fn unit<'a>(p: *const c_void, e: E) -> Option<&'a (usize, usize, u64, u64, K)> {
    w(p).units.iter().find(|u| Some(u.0) == e.id())
}
unsafe extern "C" fn valid(p: *const c_void, e: E) -> bool {
    unit(p, e).is_some()
}
unsafe extern "C" fn yes(_: *const c_void, _: E) -> bool {
    true
}
unsafe extern "C" fn no(_: *const c_void, _: E) -> bool {
    false
}
unsafe extern "C" fn is_champion(_: *const c_void, e: E) -> bool {
    e.id() == Some(1)
}
unsafe extern "C" fn team(p: *const c_void, e: E) -> usize {
    unit(p, e).map_or(9, |u| u.1)
}
unsafe extern "C" fn kind(p: *const c_void, e: E) -> u32 {
    unit(p, e).map_or(u32::MAX, |u| u.4.code())
}
unsafe extern "C" fn pos(p: *const c_void, e: E, x: *mut u64, y: *mut u64) -> bool {
    let Some(u) = unit(p, e) else { return false };
    *x = u.2;
    *y = u.3;
    true
}
unsafe extern "C" fn visible(_: *const c_void, _: usize, _: usize) -> bool {
    true
}
unsafe extern "C" fn count(p: *const c_void) -> usize {
    w(p).units.len()
}
unsafe extern "C" fn at(p: *const c_void, i: usize) -> E {
    E::from_id(w(p).units[i].0)
}
unsafe extern "C" fn one(_: *const c_void) -> usize {
    1
}
unsafe extern "C" fn first(_: *const c_void, _: usize) -> usize {
    1
}
unsafe extern "C" fn cc_count(p: *const c_void, e: E) -> usize {
    if e.id() == Some(1) {
        w(p).cc.len()
    } else {
        0
    }
}
unsafe extern "C" fn cc_at(p: *const c_void, _: E, i: usize, out: *mut CcV1) -> bool {
    *out = w(p).cc[i];
    true
}
unsafe extern "C" fn set_pos(p: *mut c_void, e: E, x: u64, y: u64) -> bool {
    w(p).moved.borrow_mut().push((e.id().unwrap(), x, y));
    true
}
fn run<T>(world: &World, f: impl FnOnce(&mut StableSim) -> T) -> T {
    unsafe {
        let mut t: SimVtableV1 = std::mem::zeroed();
        t.size = size_of::<SimVtableV1>();
        t.entity_is_valid = Some(valid);
        t.entity_is_alive = Some(yes);
        t.entity_is_targetable = Some(yes);
        t.entity_is_champion = Some(is_champion);
        t.entity_is_tower = Some(no);
        t.entity_is_minion = Some(no);
        t.entity_team = Some(team);
        t.entity_kind = Some(kind);
        t.entity_pos = Some(pos);
        t.is_visible = Some(visible);
        t.entity_count = Some(count);
        t.entity_at = Some(at);
        t.champion_count = Some(one);
        t.champion_id_at = Some(first);
        t.entity_cc_count = Some(cc_count);
        t.entity_cc_at = Some(cc_at);
        t.entity_set_pos = Some(set_pos);
        let mut raw = SimCtxV1 {
            size: size_of::<SimCtxV1>(),
            sim: &t,
            frame: std::ptr::null(),
            state: world as *const World as *mut c_void,
        };
        let mut sim = StableSim::from_raw(&mut raw).unwrap();
        f(&mut sim)
    }
}
fn omen_only(cc: Vec<CcV1>) -> World {
    World {
        units: vec![(1, 0, 100000, 100000, K::Champion)],
        cc,
        moved: RefCell::new(vec![]),
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
fn enemy_taunt_root_and_move_block_cancel_the_channel_but_his_own_root_does_not() {
    let interrupted = |cc: Vec<CcV1>| run(&omen_only(cc), |sim| Omen::interrupting(sim, 1));
    assert!(!interrupted(vec![]));
    assert!(!interrupted(vec![CcV1::of_kind(CcKindV1::Bind, 2)]), "the 2-tick self-root");
    assert!(!interrupted(vec![CcV1::of_kind(CcKindV1::BlockAttack, 36)]));
    assert!(interrupted(vec![CcV1::of_kind(CcKindV1::Bind, 30)]), "an enemy root");
    assert!(interrupted(vec![CcV1::of_kind(CcKindV1::Taunt, 20)]));
    assert!(interrupted(vec![CcV1::of_kind(CcKindV1::BlockMoveSkill, 20)]));
}

#[test]
fn crowd_control_immune_omen_is_held_at_the_channel_start() {
    let w = omen_only(vec![]);
    let mut immune = champion(1);
    immune.buffs.push(BuffV1 {
        cc_immune: true,
        ..BuffV1::named("unstoppable")
    });
    immune.x = 112000;
    let mut normal = champion(1);
    normal.x = 112000;
    run(&w, |sim| {
        let mut o = Omen::reworked();
        o.anchor = Some((100000, 100000));
        o.hold_anchor(sim, &immune);
        o.hold_anchor(sim, &normal);
    });
    assert_eq!(
        *w.moved.borrow(),
        vec![(1, 100000, 100000)],
        "only the immune Omen is put back; the root holds the other"
    );
}

#[test]
fn teleports_never_land_on_a_steve_wall_or_in_an_enemy_zone() {
    let me = champion(1);
    let mut steve = champion(2);
    steve.team = 1;
    steve.buffs.push(BuffV1::named("sbw:200000:100000:200000:200000:0:10:900"));
    let mut vesper = champion(3);
    vesper.team = 1;
    vesper.buffs.push(BuffV1::named("vsr:400000:400000:30000:0"));
    let all = vec![me.clone(), steve, vesper];
    assert!(landing_blocked(&all, &me, (204000, 150000), 50), "on the wall line");
    assert!(!landing_blocked(&all, &me, (230000, 150000), 50), "beside it");
    assert!(!landing_blocked(&all, &me, (204000, 150000), 1000), "after it falls");
    assert!(landing_blocked(&all, &me, (410000, 400000), 50), "inside the rift");
    assert!(!landing_blocked(&all, &me, (450000, 400000), 50));
    let mut ally = champion(4);
    ally.buffs.push(BuffV1::named("vsr:600000:600000:30000:0"));
    assert!(
        !landing_blocked(&[me.clone(), ally], &me, (600000, 600000), 50),
        "his own team's zones are not danger"
    );
}

#[test]
fn landing_is_checked_again_when_the_jump_completes() {
    let me = champion(1);
    let spot = (204000, 150000);
    // nothing in the way when he lands: the planned spot
    assert_eq!(landing(&[me.clone()], &me, spot, 50), Some(spot));
    // a Steve wall rose on the spot during the channel: the nearest open spot back toward him
    let mut steve = champion(2);
    steve.team = 1;
    steve.buffs.push(BuffV1::named("sbw:200000:100000:200000:200000:0:10:900"));
    let all = vec![me.clone(), steve];
    let q = landing(&all, &me, spot, 50).expect("a spot short of the wall");
    assert!(q.0 < 200000 - crate::steve::WALL_HALF, "{q:?}");
    assert!(!landing_blocked(&all, &me, q, 50));
    // an enemy zone over the spot: the jump fizzles
    let mut vesper = champion(3);
    vesper.team = 1;
    vesper.buffs.push(BuffV1::named("vsr:400000:400000:30000:0"));
    let mut far = me.clone();
    far.x = 400000;
    far.y = 360000;
    assert_eq!(landing(&[far.clone(), vesper], &far, (400000, 400000), 50), None);
}

#[test]
fn defending_a_tower_never_lands_on_the_tower() {
    let mut me = champion(1);
    me.x = 300000;
    me.y = 300000;
    let tower = (400000, 300000);
    let p = tower_side(&me, tower);
    assert_eq!(p, (400000 - TOWER_SIDE, 300000), "on his side of the tower, outside its body");
}

#[test]
fn paranoia_clears_the_wave_line_and_holds_for_an_ally_on_it() {
    let mut w = omen_only(vec![]);
    for (i, x) in [130000u64, 150000, 170000, 190000].iter().enumerate() {
        w.units.push((10 + i, 1, *x, 100000, K::Minion));
    }
    w.units.push((20, 1, 100000, 40000, K::Minion));
    w.units.push((21, 0, 140000, 100000, K::Minion));
    let me = champion(1);
    let plan = run(&w, |sim| Omen::reworked().clear_para(sim, &me, &[me.clone()]));
    let (d, ids, len) = plan.expect("a line through the wave");
    assert!(d.0 > 0.99 && d.1.abs() < 0.01, "{d:?}");
    assert_eq!(ids, vec![10, 11, 12, 13], "own minions and the lone one stay out");
    assert!((90000..=PARA_RANGE).contains(&len));
    let mut ally = champion(2);
    ally.x = 160000;
    assert!(run(&w, |sim| Omen::reworked().clear_para(sim, &me, &[me.clone(), ally.clone()])).is_none());
    assert_eq!(para_clear_damage(100), 100);
}
