//! Supported ABI evidence, separate from real-game rendering/targeting acceptance.
use super::*;
use mod_api_stable::{EntityHandleV1 as E, InputTargetV1, SimCtxV1, SimVtableV1};
use std::{ffi::c_void, mem::size_of};

#[test]
fn wall_attachment_identities_survive_growth_translation_and_rotation() {
    let w = crate::steve::Wall {
        ax: 300000,
        ay: 300000,
        bx: 340000,
        by: 300000,
        t0: 0,
        tb: 20,
        tend: 600,
    };
    let growing = wall_surfaces(&w, 10, Some((3, 7, 0)));
    let full = wall_surfaces(&w, 20, Some((3, 7, 0)));
    for (a, b) in growing.iter().zip(&full) {
        assert_eq!(a.id, b.id);
        assert_eq!(a.bounds.min, b.bounds.min);
    }
    let shifted = crate::steve::Wall {
        ax: 300000,
        ay: 312000,
        bx: 340000,
        by: 312000,
        ..w.clone()
    };
    let moved = wall_surfaces(&shifted, 20, Some((3, 7, 0)));
    assert_eq!(full.len(), moved.len());
    for (a, b) in full.iter().zip(&moved) {
        assert_eq!(a.id, b.id);
        assert_eq!(a.bounds.min.y + 12000, b.bounds.min.y);
    }
    let turned = crate::steve::Wall {
        ax: 300000,
        ay: 300000,
        bx: 300000,
        by: 340000,
        ..w
    };
    let rotated = wall_surfaces(&turned, 20, Some((3, 7, 0)));
    assert_eq!(
        full.iter().map(|s| s.id).collect::<Vec<_>>(),
        rotated.iter().map(|s| s.id).collect::<Vec<_>>()
    );
}
#[derive(Default)]
struct Host {
    hp: BTreeMap<usize, usize>,
    buffs: BTreeMap<usize, Vec<BuffV1>>,
    cc: Vec<(usize, CcV1)>,
    spawns: Vec<(String, StatV1, UnitAttackV1)>,
    effects: Vec<(String, u64)>,
    invisible: usize,
    immune: bool,
}
unsafe fn host<'a>(p: *mut c_void) -> &'a mut Host {
    &mut *(p as *mut Host)
}
unsafe extern "C" fn valid(p: *const c_void, e: E) -> bool {
    (*(p as *const Host)).hp.contains_key(&e.id().unwrap_or(0))
}
unsafe extern "C" fn alive(p: *const c_void, e: E) -> bool {
    (*(p as *const Host))
        .hp
        .get(&e.id().unwrap_or(0))
        .copied()
        .unwrap_or(0)
        > 0
}
unsafe extern "C" fn targetable(p: *const c_void, e: E) -> bool {
    alive(p, e)
}
unsafe extern "C" fn hp(p: *const c_void, e: E, a: *mut usize, b: *mut usize) -> bool {
    *a = (*(p as *const Host))
        .hp
        .get(&e.id().unwrap_or(0))
        .copied()
        .unwrap_or(0);
    *b = 1000;
    true
}
unsafe extern "C" fn pos(_: *const c_void, _: E, x: *mut u64, y: *mut u64) -> bool {
    *x = 320000;
    *y = 300000;
    true
}
unsafe extern "C" fn kind(_: *const c_void, _: E) -> u32 {
    EntityKindV1::Champion.code()
}
unsafe extern "C" fn buff_count(p: *const c_void, e: E) -> usize {
    (*(p as *const Host))
        .buffs
        .get(&e.id().unwrap_or(0))
        .map_or(0, Vec::len)
}
unsafe extern "C" fn buff_at(p: *const c_void, e: E, n: usize, b: *mut BuffV1) -> bool {
    let h = &*(p as *const Host);
    if h.immune && n == 0 {
        let mut v = BuffV1::named("immune");
        v.cc_immune = true;
        *b = v;
        return true;
    }
    if let Some(v) = h.buffs.get(&e.id().unwrap_or(0)).and_then(|v| v.get(n)) {
        *b = *v;
        true
    } else {
        false
    }
}
unsafe extern "C" fn add(p: *mut c_void, id: usize, b: *const BuffV1) {
    host(p).buffs.entry(id).or_default().push(*b);
}
unsafe extern "C" fn remove(p: *mut c_void, e: E, n: *const u8, len: usize) -> usize {
    let name = std::str::from_utf8(std::slice::from_raw_parts(n, len)).unwrap();
    let v = host(p).buffs.entry(e.id().unwrap()).or_default();
    let before = v.len();
    v.retain(|b| b.name() != name);
    before - v.len()
}
unsafe extern "C" fn cc(p: *mut c_void, id: usize, c: *const CcV1) {
    host(p).cc.push((id, *c));
}
unsafe extern "C" fn damage(p: *mut c_void, _: usize, id: usize, ad: usize, ap: usize, _: u32) {
    if let Some(hp) = host(p).hp.get_mut(&id) {
        *hp = hp.saturating_sub(ad + ap);
    }
}
unsafe extern "C" fn spawn(
    p: *mut c_void,
    n: *const u8,
    len: usize,
    _: usize,
    _: usize,
    _: u64,
    _: u64,
    _: u64,
    s: *const StatV1,
    a: *const UnitAttackV1,
) -> usize {
    let h = host(p);
    h.spawns.push((
        String::from_utf8_lossy(std::slice::from_raw_parts(n, len)).into(),
        *s,
        *a,
    ));
    h.hp.insert(3, (*s).hp);
    3
}
unsafe extern "C" fn invisible(p: *mut c_void, _: E, _: usize) -> bool {
    host(p).invisible += 1;
    true
}
unsafe extern "C" fn effect(
    p: *mut c_void,
    n: *const u8,
    len: usize,
    _: usize,
    _: *const InputTargetV1,
    _: u64,
    _: u64,
    ticks: u64,
) -> bool {
    host(p).effects.push((
        String::from_utf8_lossy(std::slice::from_raw_parts(n, len)).into(),
        ticks,
    ));
    true
}
unsafe fn table() -> SimVtableV1 {
    let mut t: SimVtableV1 = std::mem::zeroed();
    t.size = size_of::<SimVtableV1>();
    t.entity_is_valid = Some(valid);
    t.entity_is_alive = Some(alive);
    t.entity_is_targetable = Some(targetable);
    t.entity_hp = Some(hp);
    t.entity_pos = Some(pos);
    t.entity_kind = Some(kind);
    t.entity_buff_count = Some(buff_count);
    t.entity_buff_at = Some(buff_at);
    t.add_buff = Some(add);
    t.entity_remove_buff = Some(remove);
    t.apply_cc = Some(cc);
    t.deal_damage = Some(damage);
    t.spawn_unit = Some(spawn);
    t.entity_set_invisible = Some(invisible);
    t.play_view_effect = Some(effect);
    t
}
#[test]
fn compact_carrier_is_real_damageable_and_never_uses_entity_invisibility() {
    unsafe {
        let mut h = Host::default();
        h.hp.insert(1, 1000);
        h.hp.insert(2, 1000);
        let t = table();
        let mut raw = SimCtxV1 {
            size: size_of::<SimCtxV1>(),
            sim: &t,
            frame: std::ptr::null(),
            state: &mut h as *mut _ as *mut c_void,
        };
        let mut sim = StableSim::from_raw(&mut raw).unwrap();
        let mut r = Raze {
            owner: 1,
            ..Default::default()
        };
        r.commands(
            &mut sim,
            vec![Cmd::Spawn {
                id: 7,
                bot: false,
                pos: Point::new(320000, 300000),
                hp: 40,
                ticks: 300,
            }],
            0,
            1,
            &[],
        );
        assert_eq!(h.spawns[0].0, "alchemist");
        assert_eq!(h.spawns[0].1.hp, 40);
        assert_eq!(h.spawns[0].2.attack, 0);
        assert!(sim.get_entity(3).unwrap().is_targetable());
        sim.deal_damage(2, 3, 0, 11, AttackTypeV1::Skill);
        assert_eq!(h.hp[&3], 29);
        assert_eq!(h.invisible, 0);
        assert!(h.cc.iter().any(|(id, c)| *id == 3
            && c.kind == CcKindV1::Animation.code()
            && &c.name_buf[..c.name_len] == b"rz_satchel_hidden"));
        for k in [CcKindV1::Bind, CcKindV1::BlockAttack, CcKindV1::BlockSkill] {
            assert!(h.cc.iter().any(|(id, c)| *id == 3 && c.kind == k.code()));
        }
    }
}
#[test]
fn rocket_warning_replaces_and_cleans_its_exact_owner_ledger() {
    unsafe {
        let mut h = Host::default();
        h.hp.insert(1, 1000);
        h.hp.insert(2, 1000);
        let t = table();
        let mut raw = SimCtxV1 {
            size: size_of::<SimCtxV1>(),
            sim: &t,
            frame: std::ptr::null(),
            state: &mut h as *mut _ as *mut c_void,
        };
        let mut sim = StableSim::from_raw(&mut raw).unwrap();
        let mut r = Raze {
            owner: 1,
            ..Default::default()
        };
        for (tick, x) in [(1, 400000), (2, 410000)] {
            r.commands(
                &mut sim,
                vec![
                    Cmd::Warning {
                        id: 7,
                        tag: "rocket_impact_warning".into(),
                        pos: Point::new(x, 300000),
                        radius: 50000,
                        until: 32,
                    },
                    Cmd::EnemyGuide {
                        target: 2,
                        rocket: 7,
                        to: Point::new(450000, 350000),
                        until: 32,
                    },
                ],
                0,
                tick,
                &[],
            );
        }
        assert_eq!(r.zones.len(), 1);
        assert_eq!(
            h.buffs[&1]
                .iter()
                .filter(|b| b.name().starts_with("rzrw:7:"))
                .count(),
            1
        );
        assert!(h.effects.iter().all(|(_, ticks)| *ticks == 1));
        h.buffs
            .entry(2)
            .or_default()
            .push(BuffV1::named("rze:99:7:1:1:32"));
        h.hp.insert(1, 0);
        r.commands(&mut sim, vec![Cmd::ClearWarning { id: 7 }], 0, 3, &[]);
        assert!(r.zones.is_empty());
        assert!(r.enemy_guides.is_empty());
        assert_eq!(h.buffs[&2].len(), 1);
        assert!(h.buffs[&2][0].name().starts_with("rze:99:"));
    }
}

#[test]
fn round_bot_keeps_a_real_damageable_carrier_with_an_owner_only_pose() {
    unsafe {
        let mut h = Host::default();
        h.hp.insert(1, 1000);
        let t = table();
        let mut raw = SimCtxV1 {
            size: size_of::<SimCtxV1>(),
            sim: &t,
            frame: std::ptr::null(),
            state: &mut h as *mut _ as *mut c_void,
        };
        let mut sim = StableSim::from_raw(&mut raw).unwrap();
        let mut r = Raze {
            owner: 1,
            ..Default::default()
        };
        r.commands(
            &mut sim,
            vec![Cmd::Spawn {
                id: 8,
                bot: true,
                pos: Point::new(320000, 300000),
                hp: 250,
                ticks: 300,
            }],
            0,
            1,
            &[],
        );
        assert_eq!(h.spawns[0].0, "alchemist");
        assert_eq!(h.spawns[0].1.hp, 250);
        assert!(sim.get_entity(3).unwrap().is_targetable());
        sim.deal_damage(2, 3, 0, 11, AttackTypeV1::Skill);
        assert_eq!(h.hp[&3], 239);
        assert_eq!(h.invisible, 0);
        assert!(h.cc.iter().any(|(id, c)| *id == 3
            && c.kind == CcKindV1::Animation.code()
            && &c.name_buf[..c.name_len] == b"rz_bot_hidden"));
        assert!(h
            .effects
            .iter()
            .any(|(name, _)| name == "tfm2_valorant_raze_bot_round_0"));
    }
}
#[test]
fn rocket_knockback_skips_lethal_damage_and_control_immunity() {
    unsafe {
        let mut h = Host::default();
        h.hp.insert(1, 1000);
        h.hp.insert(2, 10);
        let t = table();
        let mut raw = SimCtxV1 {
            size: size_of::<SimCtxV1>(),
            sim: &t,
            frame: std::ptr::null(),
            state: &mut h as *mut _ as *mut c_void,
        };
        let mut sim = StableSim::from_raw(&mut raw).unwrap();
        let mut r = Raze {
            owner: 1,
            ..Default::default()
        };
        let push = Cmd::Knockback {
            target: 2,
            to: Point::new(330000, 300000),
            ticks: 12,
        };
        r.commands(
            &mut sim,
            vec![
                Cmd::Damage {
                    id: 7,
                    target: 2,
                    raw: 100,
                    self_hit: false,
                },
                push.clone(),
            ],
            0,
            1,
            &[],
        );
        assert!(h.cc.is_empty());
        h.hp.insert(2, 1000);
        h.immune = true;
        h.buffs.insert(2, vec![BuffV1::named("immune")]);
        r.commands(&mut sim, vec![push.clone()], 0, 2, &[]);
        assert!(h.cc.is_empty());
        h.immune = false;
        h.buffs.clear();
        r.commands(&mut sim, vec![push], 0, 3, &[]);
        assert_eq!(h.cc.len(), 1);
        assert_eq!(h.cc[0].1.kind, CcKindV1::ForceMove.code());
        assert_eq!(h.cc[0].1.tick, 12);
    }
}
