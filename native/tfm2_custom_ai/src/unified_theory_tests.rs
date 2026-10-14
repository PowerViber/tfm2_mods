//! Stable-host integration tests: invoke every actual skill handler, plus notebook and damage invariants.
use super::*;
use mod_api_stable::{EntityHandleV1, SimCtxV1, SimVtableV1};
use std::{
    cell::{Cell, RefCell},
    collections::BTreeSet,
    ffi::c_void,
    mem::size_of,
};
#[derive(Default)]
struct Host {
    tick: Cell<usize>,
    hp: RefCell<BTreeMap<usize, usize>>,
    hits: RefCell<Vec<(usize, usize, usize)>>,
    shields: RefCell<Vec<usize>>,
    cc: RefCell<Vec<usize>>,
    cc_targets: RefCell<Vec<usize>>,
    buffs: RefCell<Vec<String>>,
    active_buffs: RefCell<BTreeMap<usize, Vec<BuffV1>>>,
    removed_buffs: RefCell<Vec<String>>,
    effects: RefCell<Vec<String>>,
    effect_events: RefCell<Vec<(usize, String, u64, u64, u64)>>,
    effect_owners: RefCell<Vec<usize>>,
    positions: RefCell<Vec<(u64, u64)>>,
    visible: Cell<bool>,
    champion_count: Cell<usize>,
    teams: RefCell<BTreeMap<usize, usize>>,
    location: RefCell<BTreeMap<usize, (u64, u64)>>,
    invalid: RefCell<BTreeSet<usize>>,
    spawn_fail: Cell<bool>,
    next_unit: Cell<usize>,
    spawns: RefCell<Vec<(String, usize, usize)>>,
    hp_writes: RefCell<Vec<usize>>,
}
unsafe extern "C" fn tick(p: *const c_void) -> usize {
    (*(p as *const Host)).tick.get()
}
unsafe extern "C" fn valid(p: *const c_void, entity: EntityHandleV1) -> bool {
    entity
        .id()
        .is_some_and(|id| !(*(p as *const Host)).invalid.borrow().contains(&id))
}
unsafe extern "C" fn alive(p: *const c_void, h: EntityHandleV1) -> bool {
    (*(p as *const Host))
        .hp
        .borrow()
        .get(&h.id().unwrap())
        .copied()
        .unwrap_or(1000)
        > 0
}
unsafe extern "C" fn hp(
    p: *const c_void,
    h: EntityHandleV1,
    current: *mut usize,
    max: *mut usize,
) -> bool {
    *current = (*(p as *const Host))
        .hp
        .borrow()
        .get(&h.id().unwrap())
        .copied()
        .unwrap_or(1000);
    *max = 1000;
    true
}
unsafe extern "C" fn stat(_: *const c_void, _: EntityHandleV1, out: *mut StatV1) -> bool {
    *out = StatV1 {
        magic_power: 400,
        attack: 300,
        ..StatV1::default()
    };
    true
}
unsafe extern "C" fn pos(p: *const c_void, h: EntityHandleV1, x: *mut u64, y: *mut u64) -> bool {
    if let Some(&(px, py)) = (*(p as *const Host))
        .location
        .borrow()
        .get(&h.id().unwrap())
    {
        (*x, *y) = (px, py);
        return true;
    }
    *x = 100000 + h.id().unwrap() as u64 * 25000;
    *y = 100000;
    true
}
unsafe extern "C" fn team(p: *const c_void, h: EntityHandleV1) -> usize {
    (*(p as *const Host))
        .teams
        .borrow()
        .get(&h.id().unwrap())
        .copied()
        .unwrap_or_else(|| usize::from(h.id() != Some(0)))
}
unsafe extern "C" fn count(p: *const c_void) -> usize {
    let n = (*(p as *const Host)).champion_count.get();
    if n == 0 {
        5
    } else {
        n
    }
}
unsafe extern "C" fn at(_: *const c_void, i: usize) -> usize {
    i
}
unsafe extern "C" fn visible(p: *const c_void, _: usize, _: usize) -> bool {
    (*(p as *const Host)).visible.get()
}
unsafe extern "C" fn seed(_: *const c_void) -> u64 {
    42
}
unsafe extern "C" fn damage(p: *mut c_void, _: usize, id: usize, ad: usize, ap: usize, kind: u32) {
    assert_eq!(kind, AttackTypeV1::Skill.code());
    let h = &*(p as *const Host);
    let n = ad + ap;
    h.hits.borrow_mut().push((h.tick.get(), id, n));
    let mut hp = h.hp.borrow_mut();
    let hp = hp.entry(id).or_insert(1000);
    *hp = hp.saturating_sub(n);
}
unsafe extern "C" fn shield(p: *mut c_void, _: EntityHandleV1, n: usize, _: usize) -> bool {
    (*(p as *const Host)).shields.borrow_mut().push(n);
    true
}
unsafe extern "C" fn buff(p: *mut c_void, id: usize, b: *const BuffV1) {
    let h = &*(p as *const Host);
    h.buffs.borrow_mut().push((*b).name().to_string());
    let mut all = h.active_buffs.borrow_mut();
    let buffs = all.entry(id).or_default();
    buffs.retain(|old| old.name() != (*b).name());
    buffs.push(*b);
}
unsafe extern "C" fn buff_count(p: *const c_void, entity: EntityHandleV1) -> usize {
    (*(p as *const Host))
        .active_buffs
        .borrow()
        .get(&entity.id().unwrap())
        .map_or(0, Vec::len)
}
unsafe extern "C" fn buff_at(
    p: *const c_void,
    entity: EntityHandleV1,
    index: usize,
    out: *mut BuffV1,
) -> bool {
    let h = &*(p as *const Host);
    let buffs = h.active_buffs.borrow();
    let Some(b) = buffs.get(&entity.id().unwrap()).and_then(|v| v.get(index)) else {
        return false;
    };
    *out = *b;
    true
}
unsafe extern "C" fn remove_buff(
    p: *mut c_void,
    entity: EntityHandleV1,
    name: *const u8,
    len: usize,
) -> usize {
    let h = &*(p as *const Host);
    let name = std::str::from_utf8(std::slice::from_raw_parts(name, len)).unwrap();
    h.removed_buffs.borrow_mut().push(name.to_owned());
    let mut all = h.active_buffs.borrow_mut();
    let buffs = all.entry(entity.id().unwrap()).or_default();
    let n = buffs.len();
    buffs.retain(|b| b.name() != name);
    n - buffs.len()
}
unsafe extern "C" fn effect(
    p: *mut c_void,
    name: *const u8,
    len: usize,
    owner: usize,
    target: *const mod_api_stable::InputTargetV1,
    _: u64,
    _: u64,
    life: u64,
) -> bool {
    let h = &*(p as *const Host);
    let name = std::str::from_utf8(std::slice::from_raw_parts(name, len))
        .unwrap()
        .to_owned();
    h.effects.borrow_mut().push(name.clone());
    h.effect_events
        .borrow_mut()
        .push((h.tick.get(), name, (*target).x, (*target).y, life));
    h.effect_owners.borrow_mut().push(owner);
    true
}
unsafe extern "C" fn cc(p: *mut c_void, id: usize, c: *const CcV1) {
    (*(p as *const Host))
        .cc
        .borrow_mut()
        .push((*c).tick as usize);
    (*(p as *const Host)).cc_targets.borrow_mut().push(id);
}
unsafe extern "C" fn setpos(p: *mut c_void, entity: EntityHandleV1, x: u64, y: u64) -> bool {
    let h = &*(p as *const Host);
    h.positions.borrow_mut().push((x, y));
    // Existing tests inspect requested displacements without resnapshotting;
    // dynamic fixtures opt into positions by inserting their entity first.
    if h.location.borrow().contains_key(&entity.id().unwrap()) {
        h.location.borrow_mut().insert(entity.id().unwrap(), (x, y));
    }
    true
}
unsafe extern "C" fn set_hp(p: *mut c_void, entity: EntityHandleV1, hp: usize) -> bool {
    let h = &*(p as *const Host);
    let id = entity.id().unwrap();
    assert!(
        !h.invalid.borrow().contains(&id),
        "stale entity passed to set_hp"
    );
    h.hp_writes.borrow_mut().push(id);
    h.hp.borrow_mut().insert(id, hp);
    true
}
unsafe extern "C" fn spawn(
    p: *mut c_void,
    name: *const u8,
    len: usize,
    _: usize,
    team: usize,
    x: u64,
    y: u64,
    duration: u64,
    stat: *const StatV1,
    attack: *const UnitAttackV1,
) -> usize {
    let h = &*(p as *const Host);
    let name = std::str::from_utf8(std::slice::from_raw_parts(name, len)).unwrap();
    assert_eq!(
        name, "bombardier",
        "spawn_unit must resolve an existing base animation"
    );
    assert_eq!((*stat).move_speed, 1);
    assert_eq!((*attack).attack, 0);
    if h.spawn_fail.get() {
        return usize::MAX;
    }
    let id = h.next_unit.get().max(100);
    h.next_unit.set(id + 1);
    h.hp.borrow_mut().insert(id, (*stat).hp);
    h.location.borrow_mut().insert(id, (x, y));
    h.teams.borrow_mut().insert(id, team);
    h.spawns
        .borrow_mut()
        .push((name.to_owned(), id, h.tick.get() + duration as usize));
    id
}
fn capture(f: impl FnOnce(&mut StableSim<'_>, &Host)) {
    capture_with_visuals(false, f);
}
fn capture_with_visuals(visuals: bool, f: impl FnOnce(&mut StableSim<'_>, &Host)) {
    let h = Host {
        visible: Cell::new(true),
        ..Host::default()
    };
    let mut v: SimVtableV1 = unsafe { std::mem::zeroed() };
    v.size = size_of::<SimVtableV1>();
    v.tick = Some(tick);
    v.seed = Some(seed);
    v.entity_is_valid = Some(valid);
    v.entity_is_alive = Some(alive);
    v.entity_hp = Some(hp);
    v.entity_pos = Some(pos);
    v.entity_stat = Some(stat);
    v.entity_team = Some(team);
    v.champion_count = Some(count);
    v.champion_id_at = Some(at);
    v.is_visible = Some(visible);
    v.deal_damage = Some(damage);
    v.entity_add_shield = Some(shield);
    v.add_buff = Some(buff);
    v.apply_cc = Some(cc);
    v.entity_set_pos = Some(setpos);
    v.entity_set_hp = Some(set_hp);
    v.spawn_unit = Some(spawn);
    v.play_view_effect = Some(effect);
    if visuals {
        v.entity_buff_count = Some(buff_count);
        v.entity_buff_at = Some(buff_at);
        v.entity_remove_buff = Some(remove_buff);
    }
    let mut raw = SimCtxV1 {
        size: size_of::<SimCtxV1>(),
        sim: &v,
        frame: std::ptr::null(),
        state: (&h as *const Host).cast_mut().cast(),
    };
    let mut sim = unsafe { StableSim::from_raw(&mut raw).unwrap() };
    f(&mut sim, &h);
}
fn prepared() -> UnifiedTheory {
    let mut u = UnifiedTheory {
        rank: 7,
        rank_set: true,
        momentum: 100,
        material: 12,
        ..UnifiedTheory::default()
    };
    u.theory = Theory {
        target: Some(1),
        until: 480,
        origin: Some((100000, 100000)),
        vector: (10000, 0),
        aim: Some((125000, 100000)),
        cone: true,
        sample: true,
        label: true,
        half: 3,
        acid: true,
        base: true,
        impulse: 20,
        ..Theory::default()
    };
    u.anchors = vec![
        Anchor {
            x: 100000,
            y: 100000,
            expires: 960,
        },
        Anchor {
            x: 160000,
            y: 100000,
            expires: 960,
        },
    ];
    u.fields = vec![
        Field {
            kind: 57,
            x: 115000,
            y: 100000,
            expires: 360,
            experiment: 1,
            payload: 0,
            entity: None,
        },
        Field {
            kind: 65,
            x: 125000,
            y: 100000,
            expires: 360,
            experiment: 1,
            payload: 50,
            entity: None,
        },
    ];
    u.packets = vec![Packet {
        launch: None,
        skill: 0,
        experiment: 1,
        x: 100000,
        y: 100000,
        dx: 10000,
        dy: 0,
        payload: 50,
        physical: false,
        class: 0,
        expires: 180,
        release: 0,
        age: 0,
        split: 1,
        curve: 0,
        orbit: false,
        hits: vec![],
        visual_until: 0,
        visual_pair: 0,
        visual_motion: (10000, 0),
    }];
    u.budgets.insert(1, 400);
    u
}
#[test]
fn every_skill_has_an_executable_native_handler() {
    capture(|sim, _| {
        let all = champions(sim);
        let m = &all[0];
        let target = &all[1];
        for s in 0..75 {
            let mut u = prepared();
            let p = Experiment {
                id: 1,
                target: 1,
                power: 400,
                physical: 300,
                stages: VecDeque::new(),
            };
            assert!(u.execute(sim, m, target, &p, s, 100, 0), "{}", SKILLS[s].id);
            assert!(u.packets.len() <= 8);
            assert!(u.fields.len() <= 12);
            assert!(u.material <= 12);
            assert!(u.budgets[&1] <= 400);
        }
    });
}

// Export actual native outcomes for the browser practice model, using the same
// prepared fixture as the handler tests. No game files or mastery are touched.
#[test]
fn science_lab_native_handler_traces() {
    fn point(p: Option<(i64, i64)>) -> String {
        p.map_or("null".into(), |(x, y)| format!("[{x},{y}]"))
    }
    fn trace(u: &UnifiedTheory, h: &Host, skill: usize, phase: &str) -> String {
        let t = &u.theory;
        let packets = u
            .packets
            .iter()
            .map(|q| {
                format!(
                    "[{},{},{},{},{},{},{},{},{},{},{},{},{}]",
                    q.skill,
                    q.x,
                    q.y,
                    q.dx,
                    q.dy,
                    q.payload,
                    q.release,
                    q.expires,
                    q.physical,
                    q.class,
                    q.curve,
                    q.orbit,
                    q.split
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        let deliveries = u
            .deliveries
            .iter()
            .map(|q| {
                format!(
                    "[{},{},{},{},{},{},{},{},\"{}\",{}]",
                    q.skill,
                    q.x,
                    q.y,
                    q.destination.0,
                    q.destination.1,
                    q.payload,
                    q.release,
                    q.expires,
                    q.placement,
                    q.contact
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        let fields = u
            .fields
            .iter()
            .map(|f| format!("[{},{},{},{},{}]", f.kind, f.x, f.y, f.payload, f.expires))
            .collect::<Vec<_>>()
            .join(",");
        let anchors = u
            .anchors
            .iter()
            .map(|a| format!("[{},{},{}]", a.x, a.y, a.expires))
            .collect::<Vec<_>>()
            .join(",");
        let hits = h
            .hits
            .borrow()
            .iter()
            .map(|(tick, id, n)| format!("[{tick},{id},{n}]"))
            .collect::<Vec<_>>()
            .join(",");
        let buffs = h
            .buffs
            .borrow()
            .iter()
            .filter_map(|b| b.strip_prefix("ut0_"))
            .map(|b| format!("\"{b}\""))
            .collect::<Vec<_>>()
            .join(",");
        let position = h
            .location
            .borrow()
            .get(&0)
            .copied()
            .unwrap_or((100000, 100000));
        let state = vec![
            point(t.origin),
            point(t.aim),
            format!("[{},{}]", t.vector.0, t.vector.1),
            format!("[{},{}]", t.velocity.0, t.velocity.1),
            format!("[{},{}]", t.acceleration.0, t.acceleration.1),
            t.sync.to_string(),
            t.proper.to_string(),
            t.cone.to_string(),
            t.link.to_string(),
            t.curve.to_string(),
            t.blue.to_string(),
            t.pressure.to_string(),
            t.exchange.to_string(),
            t.drag.to_string(),
            t.scalar.to_string(),
            t.orbit.to_string(),
            t.elastic.to_string(),
            t.root.to_string(),
            t.boundary.to_string(),
            t.work.to_string(),
            t.series.to_string(),
            t.impulse.to_string(),
            t.sample.to_string(),
            t.purity.to_string(),
            t.acid.to_string(),
            t.base.to_string(),
            t.catalyst.to_string(),
            t.order.to_string(),
            t.label.to_string(),
            t.half.to_string(),
        ]
        .join(",");
        format!(
            concat!(
                "{{\"skill\":{},\"phase\":\"{}\",\"material\":{},\"momentum\":{},",
                "\"state\":[{}],",
                "\"packets\":[{}],\"deliveries\":[{}],\"fields\":[{}],\"anchors\":[{}],",
                "\"hits\":[{}],\"shields\":{:?},\"buffs\":[{}],\"position\":[{},{}]}}"
            ),
            skill,
            phase,
            u.material,
            u.momentum,
            state,
            packets,
            deliveries,
            fields,
            anchors,
            hits,
            *h.shields.borrow(),
            buffs,
            position.0,
            position.1
        )
    }
    let mut rows = Vec::new();
    for skill in 0..75 {
        capture(|sim, h| {
            h.location.borrow_mut().insert(0, (100000, 100000));
            let all = champions(sim);
            let mut u = prepared();
            let p = Experiment {
                id: 1,
                target: 1,
                power: 400,
                physical: 300,
                stages: VecDeque::new(),
            };
            assert!(u.execute(sim, &all[0], &all[1], &p, skill, 100, 0));
            rows.push(trace(&u, h, skill, "commit"));
            for tick in 1..=24 {
                h.tick.set(tick);
                u.vfx_left = 6;
                let all = champions(sim);
                u.step_world(sim, &all[0], &all[1..]);
            }
            rows.push(trace(&u, h, skill, "world24"));
        });
    }
    assert_eq!(rows.len(), 150);
    if let Some(path) = std::env::var_os("SCIENCE_NATIVE_TRACE_PATH") {
        std::fs::write(path, format!("[{}]\n", rows.join(","))).unwrap();
    }
}

#[test]
fn compact_hud_emits_only_an_orb_and_reserved_resource_bar() {
    capture(|sim, h| {
        let mut u = UnifiedTheory {
            persona: 2,
            failure_until: 24,
            ..UnifiedTheory::default()
        };
        u.charges.slots = [
            crate::unified_theory_math::Charge::Ready,
            crate::unified_theory_math::Charge::Reserved,
            crate::unified_theory_math::Charge::Recovering(150),
        ];
        let m = champions(sim)[0].clone();
        u.show(sim, &m);
        let events = h.effect_events.borrow();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].1, format!("{ID}_orb_2_0_3"));
        assert_eq!(events[1].1, format!("{ID}_bar_10_11_5"));
        assert_eq!(events[0].2, (m.x - 22500) as u64);
        assert_eq!(events[1].2, (m.x + 7500) as u64);
        assert_eq!(events[0].3, (m.y - 65000) as u64);
        assert!(events.iter().all(|e| e.4 == 8));
        drop(events);
        h.tick.set(24);
        u.vfx_left = 6;
        u.show(sim, &m);
        assert_eq!(h.effect_events.borrow()[2].1, format!("{ID}_orb_2_0_0"));
    });
}

#[test]
fn every_thrown_field_is_deployed_only_after_instrument_arrival() {
    for skill in [5, 12, 13, 20, 45, 57, 58, 65, 66] {
        capture(|sim, h| {
            let all = champions(sim);
            let mut u = prepared();
            u.fields.clear();
            u.packets.clear();
            u.anchors.clear();
            let p = Experiment {
                id: 1,
                target: 1,
                stages: VecDeque::new(),
                power: 400,
                physical: 300,
            };
            assert!(u.execute(sim, &all[0], &all[1], &p, skill, 100, 0));
            assert!(
                u.fields.is_empty(),
                "{} deployed at commitment",
                SKILLS[skill].id
            );
            assert_eq!(u.deliveries.len(), 1);
            for t in 0..RELEASE_TICKS {
                h.tick.set(t);
                u.vfx_left = 6;
                u.step_world(sim, &all[0], &all[1..]);
                assert!(u.fields.is_empty(), "instrument moved before release");
                assert!(h.hits.borrow().is_empty());
            }
            for t in RELEASE_TICKS..40 {
                h.tick.set(t);
                u.vfx_left = 6;
                u.step_world(sim, &all[0], &all[1..]);
            }
            assert!(u.deliveries.is_empty());
            assert!(
                u.fields.iter().any(|f| f.kind == skill),
                "{} never arrived",
                SKILLS[skill].id
            );
            assert!(h
                .effects
                .borrow()
                .iter()
                .any(|name| name == &format!("{ID}_impact_{}_t3", SKILLS[skill].id)));
            assert!(
                u.budgets.contains_key(&1),
                "deployed experiment lost its budget"
            );
        });
    }
}

#[test]
fn contact_actions_keep_their_budget_through_windup_and_can_be_evaded() {
    for skill in [29, 34, 71] {
        capture(|sim, h| {
            let all = champions(sim);
            let mut u = prepared();
            u.fields.clear();
            u.packets.clear();
            u.anchors.clear();
            let p = Experiment {
                id: 1,
                target: 1,
                stages: VecDeque::new(),
                power: 400,
                physical: 300,
            };
            assert!(u.execute(sim, &all[0], &all[1], &p, skill, 100, 0));
            assert!(h.hits.borrow().is_empty());
            for t in 0..RELEASE_TICKS {
                h.tick.set(t);
                u.vfx_left = 6;
                u.step_world(sim, &all[0], &all[1..]);
                assert!(h.hits.borrow().is_empty());
                assert!(u.budgets.contains_key(&1));
            }
            h.tick.set(RELEASE_TICKS);
            u.step_world(sim, &all[0], &all[1..]);
            assert_eq!(h.hits.borrow().len(), 1);
            assert_eq!(h.hits.borrow()[0].0, RELEASE_TICKS);
            assert!(u.deliveries.is_empty());
        });
    }
    capture(|sim, h| {
        let all = champions(sim);
        let mut u = prepared();
        u.fields.clear();
        u.packets.clear();
        let p = Experiment {
            id: 1,
            target: 1,
            stages: VecDeque::new(),
            power: 400,
            physical: 300,
        };
        assert!(u.execute(sim, &all[0], &all[1], &p, 34, 100, 0));
        h.location.borrow_mut().insert(1, (800000, 800000));
        h.tick.set(RELEASE_TICKS);
        u.step_world(sim, &all[0], &all[1..]);
        assert!(
            h.hits.borrow().is_empty(),
            "contact ignored the live target's escape"
        );
        assert!(u.deliveries.is_empty());
    });
}

#[test]
fn acid_and_oxidation_debuffs_land_with_the_projectile() {
    for (skill, suffix) in [(54, "_acid"), (59, "_oxidize")] {
        capture_with_visuals(true, |sim, h| {
            let all = champions(sim);
            let mut u = prepared();
            u.packets.clear();
            u.fields.clear();
            u.anchors.clear();
            let p = Experiment {
                id: 1,
                target: 1,
                stages: VecDeque::new(),
                power: 400,
                physical: 300,
            };
            assert!(u.execute(sim, &all[0], &all[1], &p, skill, 100, 0));
            assert!(!h.buffs.borrow().iter().any(|b| b.ends_with(suffix)));
            for t in 0..RELEASE_TICKS {
                h.tick.set(t);
                u.vfx_left = 6;
                u.step_world(sim, &all[0], &all[1..]);
                assert!(!h.buffs.borrow().iter().any(|b| b.ends_with(suffix)));
            }
            for t in RELEASE_TICKS..30 {
                h.tick.set(t);
                u.vfx_left = 6;
                u.step_world(sim, &all[0], &all[1..]);
            }
            assert_eq!(
                h.buffs
                    .borrow()
                    .iter()
                    .filter(|b| b.ends_with(suffix))
                    .count(),
                1
            );
            assert_eq!(h.hits.borrow().len(), 1);
            assert!(h.hits.borrow()[0].0 >= RELEASE_TICKS);
        });
    }
}

#[test]
fn crystals_use_a_valid_base_sprite_and_clean_up_live_entities_only() {
    capture_with_visuals(true, |sim, h| {
        let m = champions(sim)[0].clone();
        let mut u = UnifiedTheory::default();
        for i in 0..3 {
            u.field(sim, &m, 57, 100000 + i * 20000, 100000, 1, 0);
        }
        assert_eq!(u.fields.len(), 2);
        assert_eq!(h.spawns.borrow().len(), 3);
        assert_eq!(h.hp_writes.borrow().as_slice(), &[100]);
        assert_eq!(h.hp.borrow()[&100], 0);
        assert!(h.active_buffs.borrow()[&101]
            .iter()
            .any(|b| b.name() == "ut_crystal_body"));
        assert_eq!(
            h.cc.borrow().len(),
            9,
            "each crystal is bound and cannot attack or cast"
        );
        h.invalid.borrow_mut().insert(101);
        h.tick.set(360);
        u.step_world(sim, &m, &[]);
        assert!(u.fields.is_empty());
        assert_eq!(h.hp_writes.borrow().as_slice(), &[100, 102]);
        h.spawn_fail.set(true);
        u.field(sim, &m, 57, 100000, 100000, 2, 0);
        assert_eq!(
            u.fields[0].entity, None,
            "failed spawn has no invalid entity ownership"
        );
        u.on_dead(sim, 0);
        assert!(u.fields.is_empty());
        assert_eq!(h.hp_writes.borrow().as_slice(), &[100, 102]);
    });
}

#[test]
fn vanished_targets_and_expired_windows_do_not_retain_or_receive_actions() {
    capture(|sim, h| {
        let all = champions(sim);
        let mut u = prepared();
        u.packets.clear();
        u.fields.clear();
        let p = Experiment {
            id: 1,
            target: 1,
            stages: VecDeque::new(),
            power: 400,
            physical: 300,
        };
        u.execute(sim, &all[0], &all[1], &p, 34, 100, 0);
        for id in 1..=500 {
            u.damage.insert(id, Window(VecDeque::from([(0, 10)])));
            u.controls.insert(id, Window(VecDeque::from([(0, 10)])));
            u.moves.insert(id, Window(VecDeque::from([(0, 1)])));
        }
        h.invalid.borrow_mut().insert(1);
        h.tick.set(120);
        u.step_world(sim, &all[0], &all[1..]);
        assert!(h.hits.borrow().is_empty());
        assert!(
            !u.damage.contains_key(&1) && !u.controls.contains_key(&1) && !u.moves.contains_key(&1)
        );
        assert!(u.damage.is_empty() && u.moves.is_empty());
        assert_eq!(
            u.controls.len(),
            499,
            "live unexpired control windows must remain"
        );
        h.tick.set(240);
        u.step_world(sim, &all[0], &all[1..]);
        assert!(u.controls.is_empty());
    });
}

#[test]
fn a_removed_caster_cleans_owned_units_even_without_a_death_callback() {
    capture_with_visuals(true, |sim, h| {
        let all = champions(sim);
        let mut u = prepared();
        u.fields.clear();
        u.field(sim, &all[0], 57, 120000, 100000, 1, 0);
        u.deliver(sim, &all[0], &all[1], 5, 1, (125000, 100000), 0, false);
        u.show(sim, &all[0]);
        assert!(!u.visual_buffs.is_empty());
        h.invalid.borrow_mut().insert(0);
        h.tick.set(1);
        u.on_update(sim, 0, 0, 0);
        assert!(u.packets.is_empty() && u.deliveries.is_empty() && u.fields.is_empty());
        assert!(u.visual_buffs.is_empty() && u.anchors.is_empty());
        assert_eq!(h.hp.borrow()[&100], 0);
    });
}

#[test]
fn a_packet_kill_prevents_field_control_on_the_dead_snapshot_target() {
    capture(|sim, h| {
        let all = champions(sim);
        let mut u = prepared();
        u.fields = vec![Field {
            kind: 12,
            x: all[1].x,
            y: all[1].y,
            expires: 360,
            experiment: 1,
            payload: 0,
            entity: None,
        }];
        u.packets[0].x = 115000;
        h.hp.borrow_mut().insert(1, 10);
        h.tick.set(12);
        u.step_world(sim, &all[0], &all[1..]);
        assert_eq!(h.hp.borrow()[&1], 0);
        assert!(
            !h.cc_targets.borrow().contains(&1),
            "Gravity Well acted on a target killed earlier in this update"
        );
        assert!(!u.controls.contains_key(&1));
    });
}

#[test]
fn splitting_preserves_every_original_payload_when_slots_are_reserved_for_deliveries() {
    let u = prepared();
    let mut packets = vec![u.packets[0].clone(); 5];
    for (i, q) in packets.iter_mut().enumerate() {
        q.payload = 101 + i;
        q.experiment = i as u64;
    }
    split_existing(&mut packets, 3, 6);
    assert_eq!(packets.len(), 6);
    for id in 0..5 {
        assert_eq!(
            packets
                .iter()
                .filter(|q| q.experiment == id)
                .map(|q| q.payload)
                .sum::<usize>(),
            101 + id as usize
        );
    }
}
#[test]
fn budget_and_rolling_guard_cover_multiple_enemies() {
    capture(|sim, h| {
        let all = champions(sim);
        let m = &all[0];
        let mut u = prepared();
        u.budgets.insert(1, 480);
        for target in 1..5 {
            u.emit(sim, m, target, 250, false, 1, 73);
        }
        assert_eq!(
            h.hits.borrow().iter().map(|(_, _, n)| n).sum::<usize>(),
            480
        );
        assert!(h.hp.borrow().values().all(|hp| *hp >= 650));
        u.budgets.insert(2, 10000);
        u.emit(sim, m, 1, 1000, false, 2, 73);
        assert_eq!(h.hp.borrow()[&1], 650);
        h.tick.set(120);
        u.emit(sim, m, 1, 1000, false, 2, 73);
        assert_eq!(h.hp.borrow()[&1], 300);
    });
}
#[test]
fn notebook_critical_failure_spends_current_and_refunds_dependents() {
    capture(|sim, h| {
        let all = champions(sim);
        let m = &all[0];
        let mut u = prepared();
        u.plan_charge = u.charges.reserve(0);
        u.plan = Some(Experiment {
            id: 1,
            target: 1,
            power: 400,
            physical: 300,
            stages: VecDeque::from([
                Stage {
                    skill: 0,
                    charge: vec![4, 12, 3],
                    token: 3,
                    ready: 0,
                    started: 0,
                    error: false,
                },
                Stage {
                    skill: 73,
                    charge: vec![5, 5, 4],
                    token: 0,
                    ready: 0,
                    started: 0,
                    error: false,
                },
            ]),
        });
        u.step_notebook(sim, m, &all[1..]);
        assert_eq!(u.charges.available(), 2);
        assert!(u.plan_charge.is_none());
        assert!(u.plan.is_none());
        assert_eq!(u.failures, 1);
        assert!(h.hits.borrow().is_empty());
    });
}
#[test]
fn target_loss_refunds_every_uncommitted_stage() {
    capture(|sim, _| {
        let all = champions(sim);
        let mut u = prepared();
        u.plan_charge = u.charges.reserve(0);
        u.plan = Some(Experiment {
            id: 1,
            target: 99,
            power: 400,
            physical: 300,
            stages: VecDeque::from([Stage {
                skill: 0,
                charge: vec![4, 5, 3],
                token: 2,
                ready: 0,
                started: 0,
                error: false,
            }]),
        });
        u.step_notebook(sim, &all[0], &all[1..]);
        assert_eq!(u.charges.available(), 3);
        assert!(u.plan.is_none());
    });
}
#[test]
fn masteries_keep_common_token_speed() {
    for rank in 0..8 {
        capture(|sim, h| {
            let all = champions(sim);
            let mut u = prepared();
            u.rank = rank;
            u.plan_charge = u.charges.reserve(0);
            u.plan = Some(Experiment {
                id: 1,
                target: 1,
                power: 400,
                physical: 300,
                stages: VecDeque::from([Stage {
                    skill: 0,
                    charge: vec![4, 5, 3],
                    token: 0,
                    ready: 6,
                    started: 0,
                    error: false,
                }]),
            });
            for tick in 0..38 {
                h.tick.set(tick);
                u.step_notebook(sim, &all[0], &all[1..]);
                if tick < 38 {
                    assert!(u.plan.is_some());
                }
            }
            h.tick.set(38);
            u.step_notebook(sim, &all[0], &all[1..]);
            assert!(u.plan.is_none());
        });
    }
}
#[test]
fn cloned_simulations_preserve_the_notebook_and_rng() {
    capture(|sim, h| {
        let mut u = prepared();
        u.started = true;
        u.key = u64::MAX;
        let slot = u.charges.reserve(0).unwrap();
        u.charges.commit(slot, 0);
        let before_rng = u.rng;
        h.visible.set(false);
        h.tick.set(1);
        u.on_update(sim, 0, 0, 0);
        assert_eq!(u.charges.available(), 2);
        assert_eq!(u.rng, before_rng);
    });
}
#[test]
fn prerequisite_preparation_is_finite_for_every_template() {
    for (_, steps) in COMBOS {
        let u = UnifiedTheory::default();
        if let Some(p) = u.preparation(steps) {
            assert!(p < 75);
            assert!(u.preparation(&[p]).is_none());
        }
    }
}
#[test]
fn fixtures_are_reproducible_for_all_ranks() {
    for rank in 0..8 {
        let run = || {
            let mut records = Vec::new();
            capture(|sim, h| {
                let mut u = UnifiedTheory {
                    rank,
                    rank_set: true,
                    ..UnifiedTheory::default()
                };
                for t in 0..1800 {
                    h.tick.set(t);
                    u.on_update(sim, 0, 0, 0);
                    assert_eq!(
                        u.charges
                            .slots
                            .iter()
                            .filter(|s| **s == crate::unified_theory_math::Charge::Reserved)
                            .count(),
                        usize::from(u.plan_charge.is_some())
                    );
                    assert!(u.packets.len() <= 8);
                    assert!(u.fields.len() <= 12);
                    assert!(u.budgets.len() <= 21);
                }
                records = h.hits.borrow().clone();
            });
            records
        };
        assert_eq!(run(), run());
    }
}

#[test]
fn mastery_art_does_not_change_combat_resources_or_rng() {
    for rank in 0..8 {
        let run = |position| {
            let mut result = None;
            capture(|sim, h| {
                let mut u = UnifiedTheory {
                    rank,
                    top_pos: position,
                    rank_set: true,
                    ..UnifiedTheory::default()
                };
                for t in 0..1800 {
                    h.tick.set(t);
                    let n = h.effects.borrow().len();
                    u.on_update(sim, 0, 0, 0);
                    assert!(h.effects.borrow().len() - n <= 6, "effect budget exceeded");
                    assert!(u.pending_art.len() <= 3);
                }
                result = Some((
                    h.hits.borrow().clone(),
                    h.shields.borrow().clone(),
                    h.cc.borrow().clone(),
                    h.positions.borrow().clone(),
                    u.rng,
                    u.charges.clone(),
                    u.casts,
                    u.failures,
                    u.material,
                    u.momentum,
                ));
            });
            result.unwrap()
        };
        assert_eq!(
            run(Some(1)),
            run(Some(10)),
            "podium art changed rank {rank} gameplay"
        );
    }
}

#[test]
fn every_mastery_visual_and_direction_resolves() {
    let data =
        include_str!("../../../mods/tfm2_custom/champion/tfm2_custom_unified_theory.data_champion");
    let exists = |name: &str| {
        assert!(
            data.contains(&format!("\"name\": \"{name}\"")),
            "missing {name}"
        )
    };
    for r in 0..8 {
        for science in 0..3 {
            for position in 1..=10 {
                for mask in 0..8 {
                    let mut u = UnifiedTheory {
                        rank: r,
                        persona: science,
                        top_pos: Some(position),
                        ..UnifiedTheory::default()
                    };
                    u.theory.scalar = if mask & 1 > 0 { 2 } else { 1 };
                    u.theory.catalyst = mask & 2 > 0;
                    u.theory.half = if mask & 4 > 0 { 3 } else { 0 };
                    for name in u.style_names() {
                        exists(&name);
                    }
                }
            }
        }
    }
    for t in 0..4 {
        for s in SKILLS {
            exists(&format!("{ID}_skill_{}_t{t}", s.id));
            exists(&format!("{ID}_impact_{}_t{t}", s.id));
        }
        for k in [1, 5, 12, 13, 18, 20, 45, 57, 58, 65, 66] {
            exists(&format!("{ID}_field{k}_t{t}"));
        }
        for s in [
            0, 5, 12, 13, 16, 19, 20, 29, 34, 39, 40, 45, 54, 57, 58, 59, 63, 65, 66, 71, 72, 73,
            74,
        ] {
            for angle in 0..8 {
                exists(&format!("{ID}_packet_{}_t{t}_a{angle}", SKILLS[s].id));
                for pair in 0..4 {
                    exists(&format!(
                        "{ID}_packet_{}_t{t}_a{angle}_pair{pair}",
                        SKILLS[s].id
                    ));
                }
            }
        }
        for p in if t == 3 { vec![1, 2, 3, 4] } else { vec![4] } {
            for science in 0..3 {
                exists(&format!("{ID}_complete_t{t}_p{p}_s{science}"));
            }
        }
    }
    for s in 0..3 {
        exists(&format!("{ID}_echo{s}"));
        for progress in 0..=8 {
            for status in 0..4 {
                exists(&format!("{ID}_orb_{s}_{progress}_{status}"));
            }
        }
    }
    for a in 0..12 {
        for b in 0..12 {
            for c in 0..12 {
                exists(&format!("{ID}_bar_{a}_{b}_{c}"));
            }
        }
    }
    for (dx, dy, want) in [
        (10000, 0, 0),
        (10000, 10000, 1),
        (0, 10000, 2),
        (-10000, 0, 4),
        (0, -10000, 6),
        (0, 0, 0),
    ] {
        assert_eq!(heading(dx, dy), want);
    }
    assert_eq!(
        (0..8).map(art_tier).collect::<Vec<_>>(),
        vec![0, 0, 0, 1, 1, 2, 2, 3]
    );
}

#[test]
fn orbit_art_tracks_motion_without_changing_the_packet_vector() {
    capture(|sim, h| {
        let mut u = prepared();
        u.packets[0].orbit = true;
        let all = champions(sim);
        let m = &all[0];
        u.step_world(sim, m, &[]);
        h.effects.borrow_mut().clear();
        h.tick.set(12);
        u.vfx_left = 6;
        u.step_world(sim, m, &[]);
        assert!(h
            .effects
            .borrow()
            .iter()
            .any(|n| n == &format!("{ID}_packet_E01_t3_a2_pair1")));
        assert_eq!((u.packets[0].dx, u.packets[0].dy), (10000, 0));
    });
}

#[test]
fn visual_layers_are_replaced_restored_and_cleared_without_stacking() {
    capture_with_visuals(true, |sim, h| {
        let mut u = UnifiedTheory {
            owner: 0,
            ..UnifiedTheory::default()
        };
        u.buff(sim, 0, "buffer", 180, |b| b.toughness = 15);
        let m = champions(sim)[0].clone();
        u.sync_visual_buffs(sim, &m);
        let n = h.buffs.borrow().len();
        for _ in 0..30 {
            let m = champions(sim)[0].clone();
            u.sync_visual_buffs(sim, &m);
        }
        assert_eq!(h.buffs.borrow().len(), n, "idle loops should not restart");
        u.rank = 7;
        u.top_pos = Some(1);
        u.persona = 2;
        u.theory.scalar = 3;
        u.theory.catalyst = true;
        u.theory.half = 3;
        let m = champions(sim)[0].clone();
        u.sync_visual_buffs(sim, &m);
        assert_eq!(
            u.visual_buffs.len(),
            4,
            "body, emblem, one cosmic aura and prepared-state marks"
        );
        assert_eq!(
            u.visual_buffs
                .iter()
                .filter(|n| n.starts_with("ut_gearback"))
                .count(),
            1
        );
        assert!(!u.visual_buffs.iter().any(|n| n.starts_with("ut_gearfront")));
        assert!(!h.active_buffs.borrow()[&0]
            .iter()
            .any(|b| b.name() == "ut_outfit0_r0"));
        assert!(h.active_buffs.borrow()[&0]
            .iter()
            .any(|b| b.name() == "ut0_buffer"));
        let lost = u.visual_buffs[0].clone();
        sim.entity_remove_buff(0, &lost);
        let before = h.buffs.borrow().len();
        let m = champions(sim)[0].clone();
        u.sync_visual_buffs(sim, &m);
        assert_eq!(
            h.buffs.borrow().len(),
            before + 1,
            "restore only the missing layer"
        );
        u.echo = Some((0, 100000, 100000, 2));
        u.queue_art("test".into(), 1);
        u.on_dead(sim, 0);
        assert!(u.visual_buffs.is_empty() && u.pending_art.is_empty() && u.echo.is_none());
        let remaining = h.active_buffs.borrow();
        assert_eq!(
            remaining[&0].iter().map(|b| b.name()).collect::<Vec<_>>(),
            vec!["ut0_buffer"]
        );
    });
}

#[test]
fn cosmic_auras_replace_once_follow_the_morph_and_never_stack() {
    capture_with_visuals(true, |sim, h| {
        let mut u = UnifiedTheory {
            owner: 0,
            ..UnifiedTheory::default()
        };
        for rank in 0..8 {
            u.rank = rank;
            u.top_pos = Some(4);
            let m = champions(sim)[0].clone();
            u.sync_visual_buffs(sim, &m);
            let n = h.buffs.borrow().len();
            for _ in 0..20 {
                let m = champions(sim)[0].clone();
                u.sync_visual_buffs(sim, &m);
            }
            assert_eq!(h.buffs.borrow().len(), n, "rank {rank} aura restarted");
            let buffs = h.active_buffs.borrow();
            assert_eq!(
                buffs[&0]
                    .iter()
                    .filter(|b| b.name().starts_with("ut_gearback"))
                    .count(),
                1
            );
        }
        for position in [3, 2, 1] {
            u.top_pos = Some(position);
            let m = champions(sim)[0].clone();
            u.sync_visual_buffs(sim, &m);
            let buffs = h.active_buffs.borrow();
            assert!(buffs[&0]
                .iter()
                .any(|b| b.name() == format!("ut_gearback0_r7_p{position}")));
            assert_eq!(
                buffs[&0]
                    .iter()
                    .filter(|b| b.name().starts_with("ut_gearback"))
                    .count(),
                1
            );
        }
        u.persona = 1;
        h.tick.set(1);
        let m = champions(sim)[0].clone();
        u.sync_visual_buffs(sim, &m);
        assert!(u.visual_buffs.iter().any(|n| n == "ut_gearback1_r7_p1"));
        let n = h.buffs.borrow().len();
        for tick in 2..25 {
            h.tick.set(tick);
            u.persona = if tick % 2 == 0 { 2 } else { 0 };
            let m = champions(sim)[0].clone();
            u.sync_visual_buffs(sim, &m);
            assert_eq!(
                h.buffs.borrow().len(),
                n,
                "aura followed a queued selection"
            );
            assert!(u
                .visual_buffs
                .iter()
                .any(|name| name == "ut_gearback1_r7_p1"));
        }
        h.tick.set(25);
        u.persona = 2;
        let m = champions(sim)[0].clone();
        u.sync_visual_buffs(sim, &m);
        assert!(u
            .visual_buffs
            .iter()
            .any(|name| name == "ut_gearback2_r7_p1"));
        let buffs = h.active_buffs.borrow();
        assert_eq!(
            buffs[&0]
                .iter()
                .filter(|b| b.name().starts_with("ut_gearback"))
                .count(),
            1
        );
        assert!(!buffs[&0].iter().any(|b| b.name() == "ut_gearback1_r7_p1"));
        drop(buffs);
        sim.entity_remove_buff(0, "ut_gearback2_r7_p1");
        let n = h.buffs.borrow().len();
        let m = champions(sim)[0].clone();
        u.sync_visual_buffs(sim, &m);
        assert_eq!(
            h.buffs.borrow().len(),
            n + 1,
            "restore only the missing aura"
        );
        u.on_dead(sim, 0);
        assert!(!h.active_buffs.borrow()[&0]
            .iter()
            .any(|b| b.name().starts_with("ut_")));
    });
}

#[test]
fn completion_art_requires_a_successful_combination_and_respects_budget() {
    for (count, fails) in [(1, false), (2, false), (2, true)] {
        capture(|sim, h| {
            let mut u = prepared();
            u.top_pos = Some(1);
            u.budgets.insert(1, 480);
            let skills = vec![3, 42];
            let stages = skills
                .into_iter()
                .take(count)
                .enumerate()
                .map(|(i, s)| Stage {
                    skill: s,
                    charge: if fails && i == 1 {
                        vec![1; SKILLS[s].charge.len()]
                    } else {
                        SKILLS[s].charge.to_vec()
                    },
                    token: SKILLS[s].tokens.len(),
                    ready: 0,
                    started: 0,
                    error: false,
                })
                .collect::<VecDeque<_>>();
            u.plan_charge = u.charges.reserve(0);
            u.plan = Some(Experiment {
                id: 1,
                target: 1,
                power: 400,
                physical: 300,
                stages,
            });
            let all = champions(sim);
            let m = &all[0];
            for t in 0..30 {
                h.tick.set(t);
                u.vfx_left = 6;
                u.step_notebook(sim, m, &all[1..]);
                u.flush_art(sim, m);
            }
            let completions = h
                .effects
                .borrow()
                .iter()
                .filter(|n| n.contains("_complete_"))
                .count();
            assert_eq!(completions, usize::from(count > 1 && !fails));
            assert_eq!(u.echo.is_some(), count > 1 && !fails);
            // Finish committed casts: the new non-overlap channel intentionally
            // keeps a fast second commitment queued until the first completes.
            for _ in 0..3 {
                h.tick
                    .set(u.art_until[0].max(u.art_until[1]).max(h.tick.get()));
                u.vfx_left = 6;
                u.flush_art(sim, m);
            }
            assert!(u.pending_art.is_empty());
            u.vfx_left = 0;
            u.queue_art(format!("{ID}_skill_E01_t3"), 15);
            u.flush_art(sim, m);
            assert_eq!(
                u.pending_art.len(),
                1,
                "busy frames keep the committed cast for the next frame"
            );
            u.vfx_left = 1;
            h.tick.set(u.art_until[0].max(h.tick.get()));
            u.flush_art(sim, m);
            assert!(u.pending_art.is_empty());
            assert_eq!(u.vfx_left, 0);
        });
    }
}

#[test]
fn completion_art_follows_the_subject_of_the_final_committed_stage() {
    for (last, science) in [(23, 0), (42, 1), (50, 2)] {
        capture(|sim, h| {
            let mut u = prepared();
            u.top_pos = Some(1);
            u.budgets.insert(1, 480);
            let stages = [3, last]
                .into_iter()
                .map(|s| Stage {
                    skill: s,
                    charge: SKILLS[s].charge.to_vec(),
                    token: SKILLS[s].tokens.len(),
                    ready: 0,
                    started: 0,
                    error: false,
                })
                .collect::<VecDeque<_>>();
            u.plan_charge = u.charges.reserve(0);
            u.plan = Some(Experiment {
                id: 1,
                target: 1,
                power: 400,
                physical: 300,
                stages,
            });
            let all = champions(sim);
            for tick in 0..30 {
                h.tick.set(tick);
                u.vfx_left = 6;
                u.step_notebook(sim, &all[0], &all[1..]);
                u.flush_art(sim, &all[0]);
            }
            let effects = h.effect_events.borrow();
            let completions = effects
                .iter()
                .filter(|e| e.1.contains("_complete_"))
                .collect::<Vec<_>>();
            assert_eq!(
                completions.len(),
                1,
                "a combination gets one completion channel"
            );
            assert_eq!(completions[0].1, format!("{ID}_complete_t3_p1_s{science}"));
            assert_eq!(
                completions[0].4, 48,
                "subject art keeps the existing bounded lifetime"
            );
        });
    }
}

#[test]
fn crowded_world_has_no_overlapping_replays_and_expired_art_stops() {
    capture(|sim, h| {
        let mut u = prepared();
        let all = champions(sim);
        let m = &all[0];
        // Twelve surfaces, three anchors, and all eight packets, including a
        // coincident field/anchor and a field replaced before its loop finishes.
        u.fields = (0..12)
            .map(|i| Field {
                kind: if i == 0 {
                    1
                } else {
                    [5, 12, 13, 18, 20, 45, 57, 58, 65, 66][(i - 1) % 10]
                },
                x: 100000 + i as i64 * 20000,
                y: 100000,
                expires: 137,
                experiment: 1,
                payload: 0,
                entity: None,
            })
            .collect();
        u.anchors = (0..3)
            .map(|i| Anchor {
                x: 100000 + i * 20000,
                y: 100000,
                expires: 137,
            })
            .collect();
        u.packets = (0..8)
            .map(|i| {
                let mut q = u.packets[0].clone();
                q.orbit = true;
                q.x = 100000 + i * 20000;
                q.expires = 137;
                q
            })
            .collect();
        let mut previous = BTreeMap::new();
        let mut packet_until = [0; 8];
        for t in 0..200 {
            h.tick.set(t);
            u.vfx_left = 6;
            // Compete with notebook and cast traffic. Delays must never restart
            // a live animation or spend gameplay resources to catch up.
            if t % 8 == 0 {
                u.vfx_left = 1;
            }
            let n = h.effect_events.borrow().len();
            if t == 20 {
                u.fields[0].expires = 137;
            } // same position, new experiment
            u.step_world(sim, m, &[]);
            assert!(h.effect_events.borrow().len() - n <= 6);
            for &(born, ref name, x, y, life) in &h.effect_events.borrow()[n..] {
                if name.contains("_field") {
                    let key = (name.clone(), x, y);
                    assert!(
                        previous.get(&key).is_none_or(|until| *until <= born),
                        "stacked surface {name} at {born}"
                    );
                    assert!(life > 0 && life <= 48 && born + life as usize <= 137);
                    previous.insert(key, born + life as usize);
                }
                // Setup marks follow the committed calculation (20 s), not
                // the shorter world objects this fixture expires.
                if !name.contains("_mark") {
                    assert!(born < 137, "expired world object emitted art");
                }
            }
            for (i, q) in u.packets.iter().enumerate() {
                if q.visual_until != packet_until[i] {
                    assert!(
                        t >= packet_until[i],
                        "packet {i} restarted before its previous pair expired"
                    );
                    assert!(q.visual_until <= 137 && q.visual_until <= t + PACKET_ART_TICKS);
                    packet_until[i] = q.visual_until;
                }
            }
            assert!(u.field_visuals.len() <= 24);
        }
        let events = h.effect_events.borrow();
        let fields: Vec<_> = events
            .iter()
            .filter(|(_, n, _, _, _)| n.contains("_field"))
            .collect();
        assert_eq!(
            previous.len(),
            14,
            "all unique surfaces eventually show; coincident anchor coalesces"
        );
        assert!(
            fields.len() <= 14 * 3,
            "at most three loops per surface over 137 ticks"
        );
        assert!(
            events.iter().any(|(_, n, _, _, _)| n.ends_with("_pair3")),
            "moving loops reach all eight sprites"
        );
        assert!(u.fields.is_empty() && u.anchors.is_empty() && u.packets.is_empty());
        // Only the committed calculation's setup marks may remain.
        assert!(u.field_visuals.keys().all(|(k, _, _)| matches!(k, 26 | 42)));
    });
}

#[test]
fn rapid_casts_have_one_active_animation_per_channel() {
    capture(|sim, h| {
        let mut u = UnifiedTheory::default();
        let m = champions(sim)[0].clone();
        let mut until = [0; 2];
        let mut shown = [0; 2];
        for t in 0..240 {
            h.tick.set(t);
            u.vfx_left = 6;
            if t < 120 && t % 6 == 0 {
                u.queue_art(format!("{ID}_skill_E01_t3"), 36);
            }
            if t < 120 && t % 18 == 0 {
                u.queue_art(format!("{ID}_complete_t3_p1_s0"), 48);
            }
            let n = h.effect_events.borrow().len();
            u.flush_art(sim, &m);
            for &(born, ref name, _, _, life) in &h.effect_events.borrow()[n..] {
                let channel = usize::from(name.contains("_complete_"));
                assert!(born >= until[channel], "caster's animation stacked: {name}");
                until[channel] = born + life as usize;
                shown[channel] += 1;
            }
            assert!(u.pending_art.len() <= 3);
        }
        assert!(
            shown[0] >= 3 && shown[1] >= 3,
            "both channels make progress under rapid commitments"
        );
        assert!(u.pending_art.is_empty());
    });
}

#[test]
fn coincident_split_packets_share_one_complete_eight_frame_cycle() {
    capture(|sim, h| {
        let mut u = prepared();
        let m = champions(sim)[0].clone();
        u.fields.clear();
        u.anchors.clear();
        u.theory = Theory::default();
        u.packets = vec![u.packets[0].clone(); 8];
        for t in 0..PACKET_ART_TICKS * 4 {
            h.tick.set(t);
            u.vfx_left = 6;
            u.step_world(sim, &m, &[]);
        }
        let events = h.effect_events.borrow();
        assert_eq!(
            events.len(),
            4,
            "eight identical packets must emit four shared pairs, not 32 animations"
        );
        for (i, (t, name, _, _, life)) in events.iter().enumerate() {
            assert_eq!(*t, i * PACKET_ART_TICKS);
            assert_eq!(*life, PACKET_ART_TICKS as u64);
            assert!(name.ends_with(&format!("_pair{i}")));
        }
        assert_eq!(
            u.packets.len(),
            8,
            "cosmetic coalescing must preserve all gameplay packets"
        );
        assert!(u
            .packets
            .iter()
            .all(|p| p.payload == 50 && p.hits.is_empty()));
    });
}

#[test]
fn eight_distinct_packets_finish_their_sprite_cycles_under_the_same_start_allowance() {
    capture(|sim, h| {
        let m = champions(sim)[0].clone();
        let mut u = prepared();
        u.fields.clear();
        u.anchors.clear();
        let prototype = u.packets[0].clone();
        let skills = [0, 16, 19, 29, 34, 39, 40, 54];
        u.packets = skills
            .into_iter()
            .enumerate()
            .map(|(index, skill)| {
                let mut q = prototype.clone();
                q.skill = skill;
                q.x += index as i64 * 15000;
                q.expires = 80;
                q
            })
            .collect();
        let mut seen: BTreeMap<usize, BTreeSet<usize>> = BTreeMap::new();
        let mut until: BTreeMap<usize, usize> = BTreeMap::new();
        for t in 0..40 {
            h.tick.set(t);
            u.vfx_left = 6;
            let start = h.effect_events.borrow().len();
            u.show(sim, &m);
            u.step_world(sim, &m, &[]);
            let events = h.effect_events.borrow();
            assert!(events.len() - start <= 6);
            for (born, name, _, _, life) in &events[start..] {
                for skill in skills {
                    if name.starts_with(&format!("{ID}_packet_{}_", SKILLS[skill].id)) {
                        let pair = name.rsplit("_pair").next().unwrap().parse().unwrap();
                        assert!(until.get(&skill).is_none_or(|expires| *expires <= *born));
                        until.insert(skill, *born + *life as usize);
                        seen.entry(skill).or_default().insert(pair);
                    }
                }
            }
        }
        for skill in skills {
            assert_eq!(
                seen.get(&skill),
                Some(&BTreeSet::from([0, 1, 2, 3])),
                "{} starved under simultaneous output",
                SKILLS[skill].id
            );
        }
    });
}

#[test]
fn full_cosmic_scene_bounds_live_animations_not_only_spawns() {
    capture_with_visuals(true, |sim, h| {
        let mut u = prepared();
        u.top_pos = Some(1);
        u.fields = (0..12)
            .map(|i| Field {
                kind: [5, 12, 13, 18, 20, 45, 57, 58, 65, 66][i % 10],
                x: 200000 + i as i64 * 20000,
                y: 150000,
                expires: 360,
                experiment: 1,
                payload: 0,
                entity: None,
            })
            .collect();
        u.anchors.push(Anchor {
            x: 180000,
            y: 100000,
            expires: 360,
        });
        for a in &mut u.anchors {
            a.expires = 360;
        }
        let prototype = u.packets[0].clone();
        u.packets = [0, 16, 19, 29, 34, 39, 40, 54]
            .into_iter()
            .map(|skill| {
                let mut q = prototype.clone();
                q.skill = skill;
                q.orbit = true;
                q.expires = 360;
                q
            })
            .collect();
        u.echo = Some((0, 100000, 100000, 0));
        let mut live = Vec::new();
        let mut peak = 0;
        let mut starts = 0;
        for t in 0..420 {
            h.tick.set(t);
            u.vfx_left = 6;
            let m = champions(sim)[0].clone();
            let n = h.effect_events.borrow().len();
            if t < 300 && t % 6 == 0 {
                u.queue_art(format!("{ID}_skill_E01_t3"), 36);
            }
            if t < 300 && t % 18 == 0 {
                u.queue_art(format!("{ID}_complete_t3_p1_s0"), 48);
            }
            u.show(sim, &m);
            u.step_world(sim, &m, &[]);
            u.flush_art(sim, &m);
            live.retain(|until| *until > t);
            for &(born, _, _, _, life) in &h.effect_events.borrow()[n..] {
                assert!(life > 0);
                live.push(born + life as usize);
                starts += 1;
            }
            assert!(h.effect_events.borrow().len() - n <= 6);
            assert!(
                live.len() <= 31,
                "world/UI/cast animations piled up at {t}: {}",
                live.len()
            );
            peak = peak.max(live.len());
        }
        assert!(
            peak >= 20,
            "fixture must exercise simultaneous world and UI artwork"
        );
        assert_eq!(
            h.buffs
                .borrow()
                .iter()
                .filter(|n| n.starts_with("ut_"))
                .count(),
            4,
            "outfit, badge, one cosmic aura and prepared Half-Life loop each start once"
        );
        eprintln!("Cosmic crowded scene: {starts} starts over 7 s, peak {peak} live temporary animations, max 6 starts/update; persistent loops added once");
    });
}

fn crowded_five_vs_five(mixed_coder: bool) {
    capture_with_visuals(true, |sim, h| {
        h.champion_count.set(10);
        for id in 0..10 {
            h.teams.borrow_mut().insert(id, usize::from(id >= 5));
            h.location.borrow_mut().insert(
                id,
                (
                    if id < 5 {
                        100000 + id as u64 * 3000
                    } else {
                        145000 + (id - 5) as u64 * 3000
                    },
                    100000 + (id % 5) as u64 * 7000,
                ),
            );
        }
        let count = if mixed_coder { 4 } else { 5 };
        let all = champions(sim);
        let mut scientists: Vec<_> = (0..count)
            .map(|id| {
                let mut u = prepared();
                u.owner = id;
                u.persona = id % 3;
                u.top_pos = Some(id + 1);
                u.fields.clear();
                let m = &all[id];
                for (i, kind) in [5, 12, 13, 13, 18, 20, 45, 57, 57, 58, 65, 66]
                    .into_iter()
                    .enumerate()
                {
                    u.field(sim, m, kind, m.x + i as i64 * 18000, m.y + 50000, 1, 0);
                }
                for f in &mut u.fields {
                    f.expires = 120;
                }
                u.anchors = (0..3)
                    .map(|i| Anchor {
                        x: m.x + i * 18000,
                        y: m.y + 25000,
                        expires: 120,
                    })
                    .collect();
                let prototype = u.packets[0].clone();
                u.packets = [0, 16, 19, 29, 34, 39, 40, 54]
                    .into_iter()
                    .enumerate()
                    .map(|(i, skill)| {
                        let mut q = prototype.clone();
                        q.skill = skill;
                        q.x = m.x + i as i64 * 18000;
                        q.y = m.y;
                        q.orbit = true;
                        q.expires = 120;
                        q
                    })
                    .collect();
                u
            })
            .collect();
        let mut coder = crate::coder::Coder::default();
        let mut live: Vec<(usize, usize, String)> = Vec::new();
        let mut peak = 0;
        let mut starts = 0;
        let mut hits_after_initial = 0;
        let mut last_hits = 0;
        for t in 0..1200 {
            h.tick.set(t);
            if t.is_multiple_of(120) {
                for id in 5..10 {
                    h.hp.borrow_mut().insert(id, 1000);
                }
            }
            // Real host IDs can disappear while a delivery, crystal or window
            // still owns a reference. Interleave that with scientist death and
            // a subsequent fresh instance to model respawn.
            if t == 240 {
                scientists[0].on_dead(sim, 0);
                h.invalid.borrow_mut().insert(0);
                let unit = h.spawns.borrow().last().map(|s| s.1);
                if let Some(id) = unit {
                    h.invalid.borrow_mut().insert(id);
                }
            }
            if t == 300 {
                h.invalid.borrow_mut().remove(&0);
                scientists[0] = prepared();
                scientists[0].owner = 0;
                scientists[0].fields.clear();
                scientists[0].packets.clear();
                scientists[0].anchors.clear();
            }
            h.spawn_fail.set((360..420).contains(&t));
            let all = champions(sim);
            let before = h.effect_events.borrow().len();
            for (id, u) in scientists.iter_mut().enumerate() {
                let Some(m) = all.iter().find(|m| m.id == id) else {
                    continue;
                };
                let enemies: Vec<_> = all.iter().filter(|e| e.team != m.team).cloned().collect();
                u.vfx_left = 6;
                if t >= 120 && t.is_multiple_of(30) {
                    if let Some(target) = enemies.first() {
                        let kinds = [
                            5, 12, 13, 20, 45, 57, 58, 65, 66, 0, 54, 59, 73, 74, 29, 34, 71,
                        ];
                        let skill = kinds[(t / 30 + id) % kinds.len()];
                        let exp = (t * 10 + id + 100) as u64;
                        u.budgets.insert(exp, 400);
                        u.material = 12;
                        u.momentum = 100;
                        let p = Experiment {
                            id: exp,
                            target: target.id,
                            stages: VecDeque::new(),
                            power: 400,
                            physical: 300,
                        };
                        u.execute(sim, m, target, &p, skill, 100, 0);
                    }
                }
                if t.is_multiple_of(90) {
                    u.persona = (t / 90 + id) % 3;
                    u.queue_art(
                        format!(
                            "{ID}_complete_t3_p{}_s{}",
                            podium(u.rank, u.top_pos),
                            u.persona
                        ),
                        48,
                    );
                }
                u.show(sim, m);
                u.step_world(sim, m, &enemies);
                u.flush_art(sim, m);
                assert!(u.packets.len() + u.deliveries.len() <= OWNED_DELIVERY_LIMIT);
                assert!(u.fields.len() <= 12 && u.anchors.len() <= 3);
                assert!(u.field_visuals.len() <= 24 && u.impact_visuals.len() <= 8);
                assert!(u.pending_art.len() <= 3);
            }
            if mixed_coder {
                let m = all.iter().find(|m| m.id == 4).unwrap();
                if let Some(target) = all.iter().find(|e| e.team != m.team) {
                    coder.stress_tick(sim, &all, m, target.id, t);
                }
            }
            live.retain(|(until, _, _)| *until > t);
            let events = h.effect_events.borrow();
            let owners = h.effect_owners.borrow();
            for index in before..events.len() {
                let (born, name, _, _, life) = &events[index];
                assert!(*life > 0);
                live.push((*born + *life as usize, owners[index], name.clone()));
                starts += 1;
            }
            for id in 0..count {
                assert!(
                    live.iter()
                        .filter(|(_, owner, name)| *owner == id && name.starts_with(ID))
                        .count()
                        <= 48,
                    "scientist {id}'s animations accumulated at tick {t}"
                );
                assert!(
                    events[before..]
                        .iter()
                        .enumerate()
                        .filter(|(index, (_, name, _, _, _))| owners[before + index] == id
                            && name.starts_with(ID))
                        .count()
                        <= 6
                );
            }
            peak = peak.max(live.len());
            let hits = h.hits.borrow().len();
            if t >= 120 {
                hits_after_initial += hits - last_hits;
            }
            last_hits = hits;
        }
        assert!(
            peak >= count * 15,
            "fixture did not exercise simultaneous champions' world effects"
        );
        assert!(
            hits_after_initial > 50,
            "fixture did not exercise contacts and damage after the initial objects expired"
        );
        for u in &mut scientists {
            u.on_dead(sim, 0);
        }
        assert!(scientists.iter().all(|u| u.packets.is_empty()
            && u.deliveries.is_empty()
            && u.fields.is_empty()
            && u.pending_art.is_empty()));
        let mut living = BTreeMap::new();
        for (_, id, _) in h.spawns.borrow().iter() {
            if h.hp.borrow().get(id).copied().unwrap_or(0) > 0 && !h.invalid.borrow().contains(id) {
                *living.entry(h.teams.borrow()[id]).or_insert(0usize) += 1;
            }
        }
        assert!(living.is_empty(), "scientist death left crystals alive");
        eprintln!("{} 5v5 host stress: {starts} temporary starts/20 s, peak {peak} live, {hits_after_initial} post-initial damage contacts; death, direct form changes, invalid IDs and spawn failure exercised", if mixed_coder { "4 UT + Coder" } else { "5 UT" });
    });
}

#[test]
fn five_scientists_bound_live_effects_through_impacts_deaths_and_form_changes() {
    crowded_five_vs_five(false);
}

#[test]
fn mixed_scientists_and_coder_bound_live_effects_in_a_five_vs_five_host() {
    crowded_five_vs_five(true);
}

#[test]
fn all_six_form_changes_are_direct_and_finish_without_stacking() {
    for from in 0..3 {
        for to in 0..3 {
            if from == to {
                continue;
            }
            capture_with_visuals(true, |sim, h| {
                let mut u = UnifiedTheory {
                    owner: 0,
                    persona: from,
                    visible_persona: from,
                    ..UnifiedTheory::default()
                };
                let m = champions(sim)[0].clone();
                u.sync_visual_buffs(sim, &m);
                u.persona = to;
                h.tick.set(1);
                let m = champions(sim)[0].clone();
                u.sync_visual_buffs(sim, &m);
                let tag = format!("ut_transform{from}_{to}_r0");
                assert_eq!(u.transformation, Some((from, to, 25)));
                assert_eq!(u.visual_buffs[0], tag);
                for tick in 2..25 {
                    h.tick.set(tick);
                    let m = champions(sim)[0].clone();
                    u.sync_visual_buffs(sim, &m);
                    let buffs = h.active_buffs.borrow();
                    assert_eq!(
                        buffs[&0]
                            .iter()
                            .filter(|b| b.name().starts_with("ut_transform")
                                || b.name().starts_with("ut_outfit"))
                            .count(),
                        1
                    );
                }
                assert_eq!(h.buffs.borrow().iter().filter(|n| *n == &tag).count(), 1);
                h.tick.set(25);
                let m = champions(sim)[0].clone();
                u.sync_visual_buffs(sim, &m);
                assert!(u.transformation.is_none());
                assert_eq!(u.visible_persona, to);
                assert_eq!(u.visual_buffs[0], format!("ut_outfit{to}_r0"));
                assert_eq!(u.charges, ChargeBank::default());
                assert_eq!(u.rng, 1);
                assert!(u.plan_charge.is_none());
            });
        }
    }
}

#[test]
fn rapid_selection_keeps_only_latest_destination_and_death_clears_morph() {
    capture_with_visuals(true, |sim, h| {
        let mut u = UnifiedTheory {
            owner: 0,
            ..UnifiedTheory::default()
        };
        u.persona = 2;
        let m = champions(sim)[0].clone();
        u.sync_visual_buffs(sim, &m);
        for tick in 1..24 {
            h.tick.set(tick);
            u.persona = if tick == 23 { 1 } else { tick % 3 };
            let m = champions(sim)[0].clone();
            u.sync_visual_buffs(sim, &m);
            assert_eq!(u.transformation, Some((0, 2, 24)));
        }
        h.tick.set(24);
        let m = champions(sim)[0].clone();
        u.sync_visual_buffs(sim, &m);
        assert_eq!(u.transformation, Some((2, 1, 48)));
        assert_eq!(
            h.buffs
                .borrow()
                .iter()
                .filter(|n| n.starts_with("ut_transform"))
                .count(),
            2
        );
        u.on_dead(sim, 0);
        assert!(u.transformation.is_none() && u.visual_buffs.is_empty());
        assert!(h.active_buffs.borrow()[&0].is_empty());
    });
}

#[test]
fn trace_rotation_preserves_two_complete_logs() {
    let path = std::env::temp_dir().join(format!("tfm2-science-log-{}.txt", std::process::id()));
    let previous = path.with_extension("previous.txt");
    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_file(&previous);
    append_trace(&path, &["first".into()], 1).unwrap();
    append_trace(&path, &["second".into()], 1).unwrap();
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "second\n");
    assert_eq!(std::fs::read_to_string(&previous).unwrap(), "first\n");
    std::fs::remove_file(path).unwrap();
    std::fs::remove_file(previous).unwrap();
}
#[test]
fn status_and_buff_names_do_not_depend_on_simulation_addresses() {
    capture(|sim, h| {
        let mut u = prepared();
        u.key = 1;
        u.buff(sim, 0, "buffer", 180, |b| b.toughness = 15);
        u.key = u64::MAX;
        u.buff(sim, 0, "buffer", 180, |b| b.toughness = 15);
        let names = h.buffs.borrow();
        assert_eq!(names[0], names[1]);
    });
}

#[test]
fn grand_experiment_can_charge_after_real_prerequisites() {
    capture(|sim, h| {
        let mut u = UnifiedTheory {
            rank: 7,
            rank_set: true,
            goal: Some(124),
            ..UnifiedTheory::default()
        };
        let mut planned = false;
        let mut completed = false;
        for t in 0..3600 {
            h.tick.set(t);
            u.on_update(sim, 0, 0, 0);
            planned |= u.plan.as_ref().is_some_and(|p| {
                p.stages.len() == 5 && p.stages.front().is_some_and(|s| s.skill == 45)
            });
            completed |= u
                .trace
                .iter()
                .any(|line| line.contains("C25 (Decay Chain)") && line.contains("accepted true"));
            if completed {
                break;
            }
        }
        assert!(planned, "96-CU experiment was starved by filler casts");
        assert!(completed, "Grand Experiment never reached Decay Chain");
        assert_eq!(u.failures, 0, "A committed Grand Experiment stage failed");
    });
}
#[test]
fn chemical_cleanup_is_limited_to_scientific_tags() {
    assert!(compatible_chemical("ut0_acid", "acid"));
    assert!(compatible_chemical("ut102_oxidize", "oxidize"));
    for name in [
        "coder_acid",
        "ut_acid",
        "utMagic_acid",
        "ut12_acid_extra",
        "unrelated",
    ] {
        assert!(!compatible_chemical(name, "acid"));
    }
}

#[test]
fn stuns_and_displacements_share_the_control_window() {
    capture(|sim, h| {
        let all = champions(sim);
        let mut u = prepared();
        u.cc(sim, 1, 50);
        u.push(sim, &all[0], 1, false);
        u.cc(sim, 1, 50);
        assert_eq!(u.controls[&1].0.iter().map(|(_, n)| *n).sum::<usize>(), 60);
        assert_eq!(h.cc.borrow().iter().sum::<usize>(), 50);
    });
}

#[test]
fn every_template_commits_all_stages_when_its_prerequisites_are_live() {
    for (name, steps) in COMBOS {
        capture(|sim, h| {
            let mut u = prepared();
            let stages: VecDeque<_> = steps
                .iter()
                .map(|s| Stage {
                    skill: *s,
                    charge: SKILLS[*s].charge.to_vec(),
                    token: 0,
                    ready: 6,
                    started: 0,
                    error: false,
                })
                .collect();
            u.plan_charge = u.charges.reserve(0);
            assert!(u.plan_charge.is_some());
            u.plan = Some(Experiment {
                id: 1,
                target: 1,
                power: 400,
                physical: 300,
                stages,
            });
            for t in 0..480 {
                h.tick.set(t);
                u.on_update(sim, 0, 0, 0);
                if u.plan.is_none() {
                    break;
                }
            }
            assert!(u.plan.is_none(), "{name} did not finish");
            assert_eq!(u.failures, 0, "{name} failed a prepared stage");
            assert_eq!(u.casts, steps.len(), "{name} did not commit every stage");
        });
    }
}

#[test]
fn travelling_field_recipes_commit_through_the_real_notebook_at_long_range() {
    for name in [
        "Exposure Chamber",
        "Forced Intersection",
        "Counter-Chemical Fortification",
        "Manufactured Firing Geometry",
        "Salvage and Escape",
        "Force Becomes Gravity",
        "Turn Prediction",
    ] {
        capture(|sim, h| {
            h.location.borrow_mut().insert(1, (210000, 100000));
            let steps = COMBOS.iter().find(|c| c.0 == name).unwrap().1;
            let mut u = prepared();
            u.fields.clear();
            u.packets.clear();
            let stages: VecDeque<_> = steps
                .iter()
                .map(|skill| Stage {
                    skill: *skill,
                    charge: SKILLS[*skill].charge.to_vec(),
                    token: 0,
                    ready: 6,
                    started: 0,
                    error: false,
                })
                .collect();
            u.plan_charge = u.charges.reserve(0);
            assert!(u.plan_charge.is_some());
            u.plan = Some(Experiment {
                id: 1,
                target: 1,
                stages,
                power: 400,
                physical: 300,
            });
            for t in 0..480 {
                h.tick.set(t);
                u.on_update(sim, 0, 0, 0);
                if u.plan.is_none() {
                    break;
                }
            }
            assert!(u.plan.is_none(), "{name} did not finish");
            assert_eq!(
                u.failures, 0,
                "{name} failed before its travelling setup arrived"
            );
            assert_eq!(u.casts, steps.len(), "{name} lost a committed stage");
            assert!(u.plan_charge.is_none());
            assert_eq!(
                u.charges
                    .slots
                    .iter()
                    .filter(|s| **s == crate::unified_theory_math::Charge::Reserved)
                    .count(),
                usize::from(u.plan_charge.is_some())
            );
        });
    }
}

#[test]
fn waiting_for_a_funded_crystal_preserves_and_then_refunds_its_dependent_charge() {
    capture(|sim, h| {
        let all = champions(sim);
        let mut u = prepared();
        u.fields.clear();
        u.packets.clear();
        u.deliver(sim, &all[0], &all[1], 57, 1, (210000, 100000), 0, false);
        let charge = SKILLS[47].charge.to_vec();

        u.plan_charge = u.charges.reserve(0);
        assert!(u.plan_charge.is_some());
        let free = u.charges.available();
        u.plan = Some(Experiment {
            id: 1,
            target: 1,
            power: 400,
            physical: 300,
            stages: VecDeque::from([Stage {
                skill: 47,
                charge,
                token: SKILLS[47].tokens.len(),
                ready: 0,
                started: 0,
                error: false,
            }]),
        });
        u.step_notebook(sim, &all[0], &all[1..]);
        assert_eq!(u.charges.available(), free);
        assert!(u.plan_charge.is_some());
        assert_eq!(u.charges.available(), 2);
        assert_eq!(u.casts + u.failures, 0);
        h.visible.set(false);
        h.tick.set(1);
        u.step_notebook(sim, &all[0], &all[1..]);
        assert!(u.plan.is_none());
        assert!(u.plan_charge.is_none());
        assert_eq!(u.charges.available(), free + 1);
    });
}

#[test]
fn quench_salvages_each_new_source_once() {
    capture(|sim, _| {
        let all = champions(sim);
        let mut u = prepared();
        u.material = 5;
        let p = Experiment {
            id: 1,
            target: 1,
            power: 400,
            physical: 300,
            stages: VecDeque::new(),
        };
        assert!(u.execute(sim, &all[0], &all[1], &p, 62, 100, 0));
        assert_eq!(u.material, 6);
        assert!(!u.execute(sim, &all[0], &all[1], &p, 62, 100, 0));
        assert_eq!(u.material, 6);
        u.field(sim, &all[0], 65, 125000, 100000, 2, 50);
        assert!(u.execute(sim, &all[0], &all[1], &p, 62, 100, 0));
        assert_eq!(u.material, 7);
    });
}

#[test]
fn release_refreshes_moving_caster_and_target_and_then_stops_tracking() {
    capture(|sim, h| {
        let all = champions(sim);
        let mut u = prepared();
        u.fields.clear();
        u.packets.clear();
        u.theory.aim = Some((900000, 900000));
        u.packet(sim, &all[0], &all[1], 0, 1, 100, false);
        h.location.borrow_mut().insert(0, (110000, 120000));
        h.location.borrow_mut().insert(1, (110000, 160000));
        h.tick.set(12);
        let live = champions(sim);
        u.step_world(sim, &live[0], &live[1..]);
        let q = &u.packets[0];
        assert!(q.launch.is_none());
        assert_eq!((q.dx, q.dy), (0, 10000));
        assert_eq!((q.x, q.y), (110000, 123600));
        h.location.borrow_mut().insert(1, (150000, 120000));
        h.tick.set(13);
        let live = champions(sim);
        u.step_world(sim, &live[0], &live[1..]);
        assert_eq!((u.packets[0].dx, u.packets[0].dy), (0, 10000));
    });
}
#[test]
fn reflection_before_release_is_applied_to_the_fresh_heading() {
    capture(|sim, h| {
        let all = champions(sim);
        let mut u = prepared();
        u.fields.clear();
        u.packets.clear();
        u.packet(sim, &all[0], &all[1], 0, 1, 100, false);
        u.packets[0].dx = -u.packets[0].dx;
        u.packets[0].dy = -u.packets[0].dy;
        h.location.borrow_mut().insert(1, (100000, 140000));
        h.tick.set(12);
        let live = champions(sim);
        u.step_world(sim, &live[0], &live[1..]);
        assert_eq!((u.packets[0].dx, u.packets[0].dy), (0, -10000));
    });
}
#[test]
fn field_destinations_refresh_once_and_forward_placement_stays_relative() {
    for (skill, expected) in [
        (5, (110000, 150000)),
        (13, (110000, 130000)),
        (57, (110000, 130000)),
    ] {
        capture(|sim, h| {
            let all = champions(sim);
            let mut u = prepared();
            u.fields.clear();
            u.packets.clear();
            u.deliver(sim, &all[0], &all[1], skill, 1, (125000, 100000), 0, false);
            h.location.borrow_mut().insert(0, (110000, 110000));
            h.location.borrow_mut().insert(1, (110000, 150000));
            h.tick.set(12);
            let live = champions(sim);
            u.step_deliveries(sim, &live[0]);
            assert_eq!(u.deliveries[0].destination, expected);
            assert!(u.deliveries[0].launched);
            assert_eq!((u.deliveries[0].x, u.deliveries[0].y), (110000, 113600));
            h.location.borrow_mut().insert(1, (140000, 110000));
            h.tick.set(13);
            u.step_deliveries(sim, &live[0]);
            assert_eq!(u.deliveries[0].destination, expected);
        });
    }
}
#[test]
fn invalid_launch_discards_output_without_refunding_a_spent_charge() {
    capture(|sim, h| {
        let all = champions(sim);
        let mut u = prepared();
        u.fields.clear();
        u.packets.clear();
        let slot = u.charges.reserve(0).unwrap();
        assert!(u.charges.commit(slot, 0));
        u.packet(sim, &all[0], &all[1], 0, 1, 100, false);
        h.visible.set(false);
        h.tick.set(12);
        u.step_world(sim, &all[0], &[]);
        assert!(u.packets.is_empty());
        assert_eq!(u.charges.available(), 2);
        assert!(h.hits.borrow().is_empty());
    });
}
#[test]
fn repeated_skill_stages_spend_one_charge_and_death_keeps_its_deadline() {
    capture(|sim, h| {
        let all = champions(sim);
        let mut u = prepared();
        u.fields.clear();
        u.packets.clear();
        u.plan_charge = u.charges.reserve(0);
        u.plan = Some(Experiment {
            id: 1,
            target: 1,
            power: 400,
            physical: 300,
            stages: VecDeque::from([0, 0].map(|skill| Stage {
                skill,
                charge: SKILLS[skill].charge.to_vec(),
                token: 0,
                ready: 6,
                started: 0,
                error: false,
            })),
        });
        for t in 0..=76 {
            h.tick.set(t);
            u.step_notebook(sim, &all[0], &all[1..]);
        }
        assert!(u.plan.is_none());
        assert_eq!(u.casts, 2);
        assert_eq!(u.charges.available(), 2);
        let before = u.charges.clone();
        u.on_dead(sim, 0);
        assert_eq!(u.charges, before);
        u.charges.advance(337);
        assert_eq!(u.charges.available(), 2);
        u.charges.advance(338);
        assert_eq!(u.charges.available(), 3);
    });
}
#[test]
fn planner_chooses_actionable_defense_and_rejects_out_of_range_heavy_attacks() {
    capture(|sim, h| {
        let mut u = prepared();
        u.fields.clear();
        u.packets.clear();
        h.hp.borrow_mut().insert(0, 250);
        let all = champions(sim);
        u.pick(sim, &all[0], &all[1..]);
        assert!(u.trace.iter().any(|line| line.contains("urgent defense")));
        assert_eq!(
            SKILLS[u.plan.as_ref().unwrap().stages[0].skill].purpose,
            "defend"
        );
        u.cancel();
        h.hp.borrow_mut().insert(0, 1000);
        h.location.borrow_mut().insert(1, (150000, 100000));
        u.goal = Some(34);
        let all = champions(sim);
        u.pick(sim, &all[0], &all[1..]);
        assert!(u
            .plan
            .as_ref()
            .unwrap()
            .stages
            .iter()
            .all(|s| s.skill != 34));
    });
}

// ---- Round 125: chain payoff, lab accidents, Unified resonance, tempo ----

fn ready_stage(skill: usize, charge: Vec<u32>) -> Stage {
    Stage {
        skill,
        token: SKILLS[skill].tokens.len(),
        charge,
        ready: 0,
        started: 0,
        error: false,
    }
}
fn run_plan(
    u: &mut UnifiedTheory,
    sim: &mut StableSim<'_>,
    h: &Host,
    skills: &[usize],
    from: usize,
) {
    let stages: VecDeque<_> = skills
        .iter()
        .map(|&s| ready_stage(s, SKILLS[s].charge.to_vec()))
        .collect();
    u.plan_charge = u.charges.reserve(from);
    u.plan = Some(Experiment {
        id: 7,
        target: 1,
        power: 400,
        physical: 300,
        stages,
    });
    u.budgets.insert(7, 2000);
    let all = champions(sim);
    for t in from..from + 40 {
        h.tick.set(t);
        u.vfx_left = 6;
        u.step_notebook(sim, &all[0], &all[1..]);
        u.flush_art(sim, &all[0]);
    }
}
#[test]
fn chain_percentages_are_bounded() {
    assert_eq!(
        (0..7).map(chain_pct).collect::<Vec<_>>(),
        vec![100, 120, 140, 160, 180, 180, 180]
    );
    assert_eq!(base_output(0, 400), 150);
    assert_eq!(base_output(73, 400), 270);
    assert_eq!(resonance_output(400, 300), 120);
}
#[test]
fn later_offense_in_a_successful_chain_carries_the_payoff() {
    for (skills, expected) in [(vec![0], 150), (vec![3, 0], 180), (vec![3, 42, 0], 210)] {
        capture(|sim, h| {
            let mut u = prepared();
            u.packets.clear();
            run_plan(&mut u, sim, h, &skills, 0);
            assert!(u.plan.is_none(), "every stage committed");
            let pulse: Vec<_> = u.packets.iter().filter(|q| q.skill == 0).collect();
            assert_eq!(pulse.len(), 1);
            assert_eq!(pulse[0].payload, expected, "chain {skills:?}");
        });
    }
}
#[test]
fn a_failed_stage_ends_the_chain_without_payoff() {
    capture(|sim, h| {
        let mut u = prepared();
        u.packets.clear();
        let stages = VecDeque::from([
            ready_stage(3, SKILLS[3].charge.to_vec()),
            ready_stage(42, vec![1; SKILLS[42].charge.len()]),
            ready_stage(0, SKILLS[0].charge.to_vec()),
        ]);
        u.plan_charge = u.charges.reserve(0);
        u.plan = Some(Experiment {
            id: 7,
            target: 1,
            power: 400,
            physical: 300,
            stages,
        });
        let all = champions(sim);
        for t in 0..40 {
            h.tick.set(t);
            u.step_notebook(sim, &all[0], &all[1..]);
        }
        assert!(u.packets.is_empty() && u.plan.is_none());
        assert_eq!(
            u.charges.available(),
            2,
            "one charge spent by the first commitment"
        );
        assert!(u.plan_charge.is_none());
    });
}
#[test]
fn pick_scales_the_budget_cap_with_the_planned_chain() {
    capture(|sim, _| {
        let all = champions(sim);
        for seed in 1..400u64 {
            let mut u = prepared();
            u.rng = seed;
            u.budgets.clear();
            u.pick(sim, &all[0], &all[1..]);
            let Some(p) = &u.plan else { continue };
            let cap = (160 + 400 * 80 / 100) * chain_pct(p.stages.len() - 1) / 100;
            assert!(u.budgets[&p.id] <= cap);
        }
    });
}
#[test]
fn critical_failures_are_visible_nonlethal_lab_accidents() {
    for science_skill in [0usize, 29, 54] {
        capture(|sim, h| {
            let all = champions(sim);
            let mut u = prepared();
            let s = science_skill;
            let mut charge = SKILLS[s].charge.to_vec();
            charge[1] *= 3; // critical instability
            u.plan = Some(Experiment {
                id: 1,
                target: 1,
                power: 400,
                physical: 300,
                stages: VecDeque::from([ready_stage(s, charge)]),
            });
            u.vfx_left = 6;
            u.step_notebook(sim, &all[0], &all[1..]);
            assert_eq!(u.accidents, 1);
            assert_eq!(h.hp.borrow()[&0], 960, "4% of maximum HP");
            assert!(h.cc_targets.borrow().contains(&0));
            assert!(h.cc.borrow().contains(&30));
            let science = SKILLS[s].science;
            assert!(h
                .effects
                .borrow()
                .iter()
                .any(|n| n.ends_with(&format!("_accident{science}"))));
            assert!(h.hits.borrow().is_empty());
        });
    }
    // Never lethal, even at 1 HP.
    capture(|sim, h| {
        h.hp.borrow_mut().insert(0, 1);
        let all = champions(sim);
        let mut u = prepared();
        let charge = vec![4, 12, 3];
        u.plan = Some(Experiment {
            id: 1,
            target: 1,
            power: 400,
            physical: 300,
            stages: VecDeque::from([ready_stage(0, charge)]),
        });
        u.step_notebook(sim, &all[0], &all[1..]);
        assert_eq!(h.hp.borrow()[&0], 1);
    });
}
#[test]
fn ordinary_invalid_commits_only_fizzle() {
    capture(|sim, h| {
        let all = champions(sim);
        let mut u = prepared();
        u.material = 0; // Radioactive Burst needs two material units.
        u.plan = Some(Experiment {
            id: 1,
            target: 1,
            power: 400,
            physical: 300,
            stages: VecDeque::from([ready_stage(63, SKILLS[63].charge.to_vec())]),
        });
        u.vfx_left = 6;
        u.step_notebook(sim, &all[0], &all[1..]);
        assert_eq!(u.accidents, 0);
        assert!(h.hp.borrow().get(&0).is_none());
        assert!(h.effects.borrow().iter().any(|n| n.ends_with("_fizzle")));
    });
}
#[test]
fn three_science_chains_release_one_unified_strike_with_a_cooldown() {
    capture(|sim, h| {
        let mut u = prepared();
        u.top_pos = Some(1);
        run_plan(&mut u, sim, h, &[3, 42, 50], 0);
        assert_eq!(u.resonances, 1);
        assert_eq!(
            h.hits
                .borrow()
                .iter()
                .filter(|(_, id, _)| *id == 1)
                .map(|x| x.2)
                .sum::<usize>(),
            120
        );
        assert!(h
            .effects
            .borrow()
            .iter()
            .any(|n| n.contains("_complete_unified_t3_p1")));
        assert!(!h
            .effects
            .borrow()
            .iter()
            .any(|n| n.contains("_complete_t3")));
        // Inside the cooldown, a second three-science chain is an ordinary completion.
        run_plan(&mut u, sim, h, &[3, 42, 50], 100);
        assert_eq!(u.resonances, 1);
        // After it, resonance returns.
        run_plan(&mut u, sim, h, &[3, 42, 50], 400);
        assert_eq!(u.resonances, 2);
    });
    capture(|sim, h| {
        let mut u = prepared();
        run_plan(&mut u, sim, h, &[3, 42, 23], 0);
        assert_eq!(u.resonances, 0, "two sciences do not resonate");
        assert!(h.hits.borrow().is_empty());
    });
}
#[test]
fn resonance_stays_inside_the_target_damage_window() {
    capture(|sim, h| {
        let mut u = prepared();
        h.tick.set(0);
        u.damage.entry(1).or_default().allow(0, 120, 350, 300);
        run_plan(&mut u, sim, h, &[3, 42, 50], 1);
        let total: usize = h.hits.borrow().iter().map(|x| x.2).sum();
        assert!(total <= 50, "window already held 300 of 350");
    });
}
#[test]
fn experienced_athletes_seek_three_science_chains() {
    capture(|sim, _| {
        let all = champions(sim);
        let mut count = [0usize; 8];
        for rank in [0, 7] {
            for seed in 1..600u64 {
                let mut u = prepared();
                u.rank = rank;
                u.rng = seed * 2654435761 % 1_000_003 + 1;
                u.pick(sim, &all[0], &all[1..]);
                if let Some(p) = &u.plan {
                    let mask = p
                        .stages
                        .iter()
                        .fold(0u8, |m, s| m | 1 << SKILLS[s.skill].science);
                    count[rank] += usize::from(mask == 7);
                }
            }
        }
        eprintln!(
            "three-science plans per 599 picks: Student {}, Unified Mind {}",
            count[0], count[7]
        );
        assert!(count[7] > count[0] * 2, "{count:?}");
    });
}
/// Full passive loop against one dummy for 20 seconds: the real planner,
/// notebook, charge, failures, chains and resonance. Raw damage only.
fn duel_for(rank: usize, dummy_x: u64, ticks: usize) -> (usize, usize, usize, usize, usize) {
    let mut out = (0, 0, 0, 0, 0);
    capture(|sim, h| {
        h.champion_count.set(2);
        h.location.borrow_mut().insert(0, (100000, 100000));
        h.location.borrow_mut().insert(1, (dummy_x, 100000));
        let mut u = UnifiedTheory {
            rank,
            rank_set: true,
            ..UnifiedTheory::default()
        };
        let mut worst = 0;
        for t in 0..ticks {
            h.tick.set(t);
            if h.hp.borrow().get(&1).copied().unwrap_or(1000) < 200 {
                h.hp.borrow_mut().insert(1, 1000);
            }
            if h.hp.borrow().get(&0).copied().unwrap_or(1000) < 300 {
                h.hp.borrow_mut().insert(0, 1000);
            }
            u.on_update(sim, 0, 0, 0);
            let hits = h.hits.borrow();
            let window: usize = hits
                .iter()
                .filter(|(at, id, _)| *id == 1 && at + 120 > t)
                .map(|x| x.2)
                .sum();
            worst = worst.max(window);
        }
        let raw = h
            .hits
            .borrow()
            .iter()
            .filter(|x| x.1 == 1)
            .map(|x| x.2)
            .sum();
        out = (raw, u.casts, u.accidents, u.resonances, worst);
    });
    out
}
#[test]
fn mastery_is_the_power_source_and_never_one_shots() {
    // 60 s against one dummy at four distances, on the three-charge tempo.
    for x in [120000u64, 140000, 160000, 180000] {
        let (low, high) = (duel_for(0, x, 3600), duel_for(7, x, 3600));
        eprintln!(
            "60 s duel at {} raw / casts / accidents / resonances / worst 2 s window: Student {low:?}, Unified Mind {high:?}",
            x - 100000
        );
        assert!(low.4 <= 350 && high.4 <= 350, "35% max HP per 2 s");
        assert!(
            high.0 > low.0,
            "a master deals more through chains and resonance"
        );
        assert!(low.2 > high.2, "students have more lab accidents");
        assert!(high.3 >= 5 && low.3 == 0, "resonance belongs to mastery");
    }
}

#[test]
fn facing_left_uses_the_mirrored_body_layer_only() {
    let mut u = UnifiedTheory::default();
    u.facing.update(0, (500_000, 0), Some((400_000, 0)));
    let names = u.style_names();
    assert!(names[0].starts_with("ut_outfit") && names[0].ends_with("_l"), "{names:?}");
    assert!(names.iter().filter(|n| n.ends_with("_l")).count() == 1, "badge and gear aura stay as drawn: {names:?}");
    u.plan = None;
    u.facing.update(10, (500_000, 0), Some((600_000, 0)));
    assert!(!u.style_names()[0].ends_with("_l"), "facing right again");
}
