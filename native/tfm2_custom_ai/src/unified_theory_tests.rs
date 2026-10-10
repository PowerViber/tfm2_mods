//! Stable-host integration tests: invoke every actual skill handler, plus notebook and damage invariants.
use super::*;
use mod_api_stable::{EntityHandleV1, SimCtxV1, SimVtableV1};
use std::{
    cell::{Cell, RefCell},
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
    buffs: RefCell<Vec<String>>,
    active_buffs: RefCell<BTreeMap<usize, Vec<BuffV1>>>,
    removed_buffs: RefCell<Vec<String>>,
    effects: RefCell<Vec<String>>,
    effect_events: RefCell<Vec<(usize, String, u64, u64, u64)>>,
    positions: RefCell<Vec<(u64, u64)>>,
    visible: Cell<bool>,
}
unsafe extern "C" fn tick(p: *const c_void) -> usize {
    (*(p as *const Host)).tick.get()
}
unsafe extern "C" fn valid(_: *const c_void, _: EntityHandleV1) -> bool {
    true
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
unsafe extern "C" fn pos(_: *const c_void, h: EntityHandleV1, x: *mut u64, y: *mut u64) -> bool {
    *x = 100000 + h.id().unwrap() as u64 * 25000;
    *y = 100000;
    true
}
unsafe extern "C" fn team(_: *const c_void, h: EntityHandleV1) -> usize {
    usize::from(h.id() != Some(0))
}
unsafe extern "C" fn count(_: *const c_void) -> usize {
    5
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
    _: usize,
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
    true
}
unsafe extern "C" fn cc(p: *mut c_void, _: usize, c: *const CcV1) {
    (*(p as *const Host))
        .cc
        .borrow_mut()
        .push((*c).tick as usize);
}
unsafe extern "C" fn setpos(p: *mut c_void, _: EntityHandleV1, x: u64, y: u64) -> bool {
    (*(p as *const Host)).positions.borrow_mut().push((x, y));
    true
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
        u.pool.reserve(3300);
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
        assert_eq!(u.pool.free, 8100);
        assert_eq!(u.pool.reserved, 0);
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
        u.pool.reserve(1200);
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
        assert_eq!(u.pool.free, 10000);
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
            u.pool.reserve(1200);
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
        u.pool.free = 7300;
        let before_rng = u.rng;
        h.visible.set(false);
        h.tick.set(1);
        u.on_update(sim, 0, 0, 0);
        assert_eq!(u.pool.free, 7306);
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
                    assert!(u.pool.free + u.pool.reserved <= 10000);
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
                    u.cooldown.clone(),
                    u.pool.free,
                    u.pool.reserved,
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
        }
        for k in [1, 5, 12, 13, 18, 20, 45, 57, 58, 65, 66] {
            exists(&format!("{ID}_field{k}_t{t}"));
        }
        for s in [0, 16, 19, 29, 34, 39, 40, 54, 59, 63, 71, 72, 73, 74] {
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
            exists(&format!("{ID}_complete_t{t}_p{p}"));
        }
    }
    for s in 0..3 {
        exists(&format!("{ID}_echo{s}"));
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
            u.visual_buffs.iter().filter(|n| n.starts_with("ut_gearback")).count(),
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
        let mut u = UnifiedTheory { owner: 0, ..UnifiedTheory::default() };
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
            assert_eq!(buffs[&0].iter().filter(|b| b.name().starts_with("ut_gearback")).count(), 1);
        }
        for position in [3, 2, 1] {
            u.top_pos = Some(position);
            let m = champions(sim)[0].clone();
            u.sync_visual_buffs(sim, &m);
            let buffs = h.active_buffs.borrow();
            assert!(buffs[&0].iter().any(|b| b.name() == format!("ut_gearback0_r7_p{position}")));
            assert_eq!(buffs[&0].iter().filter(|b| b.name().starts_with("ut_gearback")).count(), 1);
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
            assert_eq!(h.buffs.borrow().len(), n, "aura followed a queued selection");
            assert!(u.visual_buffs.iter().any(|name| name == "ut_gearback1_r7_p1"));
        }
        h.tick.set(25);
        u.persona = 2;
        let m = champions(sim)[0].clone();
        u.sync_visual_buffs(sim, &m);
        assert!(u.visual_buffs.iter().any(|name| name == "ut_gearback2_r7_p1"));
        let buffs = h.active_buffs.borrow();
        assert_eq!(buffs[&0].iter().filter(|b| b.name().starts_with("ut_gearback")).count(), 1);
        assert!(!buffs[&0].iter().any(|b| b.name() == "ut_gearback1_r7_p1"));
        drop(buffs);
        sim.entity_remove_buff(0, "ut_gearback2_r7_p1");
        let n = h.buffs.borrow().len();
        let m = champions(sim)[0].clone();
        u.sync_visual_buffs(sim, &m);
        assert_eq!(h.buffs.borrow().len(), n + 1, "restore only the missing aura");
        u.on_dead(sim, 0);
        assert!(!h.active_buffs.borrow()[&0].iter().any(|b| b.name().starts_with("ut_")));
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
            u.pool.reserve(stages.iter().map(|s| cost(&s.charge)).sum());
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
                assert!(born < 137, "expired world object emitted art");
            }
            for (i, q) in u.packets.iter().enumerate() {
                if q.visual_until != packet_until[i] {
                    assert!(
                        t >= packet_until[i],
                        "packet {i} restarted before its previous pair expired"
                    );
                    assert!(q.visual_until <= 137 && q.visual_until <= t + 12);
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
        assert!(u.field_visuals.is_empty());
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
                u.queue_art(format!("{ID}_complete_t3_p1"), 48);
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
        for t in 0..48 {
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
            assert_eq!(*t, i * 12);
            assert_eq!(*life, 12);
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
            .all(|p| p.payload == 47 && p.hits.is_empty()));
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
                u.queue_art(format!("{ID}_complete_t3_p1"), 48);
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
                assert_eq!(u.pool.free, Pool::default().free);
                assert_eq!(u.rng, 1);
                assert!(u.cooldown.iter().all(|&n| n == 0));
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
            completed |= u.cooldown[74] > 0;
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
            let charge = stages.iter().map(|s| cost(&s.charge)).sum();
            assert!(u.pool.reserve(charge));
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
