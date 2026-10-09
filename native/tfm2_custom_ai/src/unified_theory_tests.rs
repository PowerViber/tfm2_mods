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
unsafe extern "C" fn buff(p: *mut c_void, _: usize, b: *const BuffV1) {
    (*(p as *const Host))
        .buffs
        .borrow_mut()
        .push((*b).name().to_string());
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
