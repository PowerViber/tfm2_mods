//! Exercise native damage emissions, queues and support through the stable host API.
use super::*;
use mod_api_stable::{EntityHandleV1, SimCtxV1, SimOriginKindV1, SimOriginV1, SimVtableV1, StatV1};
use std::{
    cell::{Cell, RefCell},
    collections::HashSet,
    ffi::c_void,
    mem::size_of,
};

#[derive(Default)]
struct Host {
    tick: Cell<usize>,
    ap: Cell<usize>,
    hp: RefCell<HashMap<usize, usize>>,
    origin: Cell<u32>,
    hits: RefCell<Vec<(usize, usize, u32)>>,
    heals: RefCell<Vec<usize>>,
    shields: RefCell<Vec<(usize, usize)>>,
    buffs: RefCell<Vec<BuffV1>>,
    cc: RefCell<Vec<CcV1>>,
}
struct Capture {
    hits: Vec<(usize, usize, u32)>,
    heals: Vec<usize>,
    shields: Vec<(usize, usize)>,
    buffs: Vec<BuffV1>,
    cc: Vec<CcV1>,
    trace: Vec<String>,
}
unsafe extern "C" fn tick(p: *const c_void) -> usize {
    (*(p as *const Host)).tick.get()
}
unsafe extern "C" fn valid(_: *const c_void, _: EntityHandleV1) -> bool {
    true
}
unsafe extern "C" fn stat(p: *const c_void, _: EntityHandleV1, out: *mut StatV1) -> bool {
    *out = StatV1 {
        magic_power: (*(p as *const Host)).ap.get(),
        ..StatV1::default()
    };
    true
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
unsafe extern "C" fn alive(p: *const c_void, h: EntityHandleV1) -> bool {
    (*(p as *const Host))
        .hp
        .borrow()
        .get(&h.id().unwrap())
        .copied()
        .unwrap_or(1000)
        > 0
}
unsafe extern "C" fn set_hp(p: *mut c_void, h: EntityHandleV1, n: usize) -> bool {
    (*(p as *const Host))
        .hp
        .borrow_mut()
        .insert(h.id().unwrap(), n);
    true
}
unsafe extern "C" fn origin(p: *const c_void, out: *mut SimOriginV1) -> bool {
    *out = SimOriginV1 {
        kind: (*(p as *const Host)).origin.get(),
        ..SimOriginV1::default()
    };
    true
}
unsafe extern "C" fn damage(
    p: *mut c_void,
    _: usize,
    target: usize,
    _: usize,
    n: usize,
    kind: u32,
) {
    let s = &*(p as *const Host);
    s.hits.borrow_mut().push((s.tick.get(), n, kind));
    let mut hp = s.hp.borrow_mut();
    let hp = hp.entry(target).or_insert(1000);
    *hp = hp.saturating_sub(n);
}
unsafe extern "C" fn heal(p: *mut c_void, _: usize, _: usize, n: usize) {
    (*(p as *const Host)).heals.borrow_mut().push(n);
}
unsafe extern "C" fn shield(p: *mut c_void, _: EntityHandleV1, n: usize, duration: usize) -> bool {
    (*(p as *const Host))
        .shields
        .borrow_mut()
        .push((n, duration));
    true
}
unsafe extern "C" fn buff(p: *mut c_void, _: usize, b: *const BuffV1) {
    (*(p as *const Host)).buffs.borrow_mut().push(*b);
}
unsafe extern "C" fn cc(p: *mut c_void, _: usize, c: *const CcV1) {
    (*(p as *const Host)).cc.borrow_mut().push(*c);
}
fn capture(ap: usize, f: impl FnOnce(&mut StableSim<'_>, &Host)) -> Capture {
    trace::take_test_lines();
    let state = Host {
        ap: Cell::new(ap),
        ..Host::default()
    };
    // Nullable vtable slots model absent capabilities. Interior mutability lets callbacks share host state safely.
    let mut table: SimVtableV1 = unsafe { std::mem::zeroed() };
    table.size = size_of::<SimVtableV1>();
    table.tick = Some(tick);
    table.entity_is_valid = Some(valid);
    table.entity_stat = Some(stat);
    table.entity_hp = Some(hp);
    table.entity_is_alive = Some(alive);
    table.entity_set_hp = Some(set_hp);
    table.sim_origin = Some(origin);
    table.deal_damage = Some(damage);
    table.heal = Some(heal);
    table.entity_add_shield = Some(shield);
    table.add_buff = Some(buff);
    table.apply_cc = Some(cc);
    let mut raw = SimCtxV1 {
        size: size_of::<SimCtxV1>(),
        sim: &table,
        frame: std::ptr::null(),
        state: (&state as *const Host).cast_mut().cast(),
    };
    let mut sim = unsafe { StableSim::from_raw(&mut raw).unwrap() };
    f(&mut sim, &state);
    Capture {
        hits: state.hits.into_inner(),
        heals: state.heals.into_inner(),
        shields: state.shields.into_inner(),
        buffs: state.buffs.into_inner(),
        cc: state.cc.into_inner(),
        trace: trace::take_test_lines(),
    }
}
fn unit(id: usize, team: usize, x: i64, hp: usize) -> Champ {
    Champ {
        id,
        team,
        x,
        y: 0,
        hp,
        max_hp: 1000,
        attack: 60,
        buffs: vec![],
        stunned: false,
        pushed: false,
        name: String::new(),
    }
}
fn coder() -> Coder {
    Coder {
        rank: Some(3),
        heat: 4000,
        tiers: [0; NPARTS],
        cooldown: vec![0; NF],
        last_run: vec![0; NF],
        ..Coder::default()
    }
}
fn compiled(f: usize) -> Compiled {
    Compiled {
        f,
        lang: PY,
        bugs: vec![],
        saved: true,
        reloaded: false,
    }
}
fn amount(ap: usize, base: usize, ratio: usize) -> usize {
    (base + ap * ratio / 100) * 104 / 100
}

#[test]
fn loaded_code_reduces_every_emitted_hit_and_preserves_support_and_delivery_times() {
    let me = unit(0, 0, 0, 1000);
    let all = vec![
        me.clone(),
        unit(1, 0, 20_000, 400),
        unit(5, 1, 18_000, 79),
        unit(6, 1, 10_800, 160),
        unit(7, 1, 25_000, 1000),
    ];
    let mut damaging = HashSet::new();
    for f in 0..NF {
        for scale in [0, 50, 60, 100] {
            let run = |reloaded| {
                let mut c = coder();
                c.rng = Rng(99);
                c.scale = scale;
                c.btc = 100_000;
                c.program.push(compiled(PING));
                c.last_script = Some(compiled(F_CUDA_KERNEL));
                c.last_effect = Some((compiled(PING), 5));
                let mut code = compiled(f);
                code.reloaded = reloaded;
                capture(600, |sim, st| {
                    for id in [0, 1, 5, 6, 7] {
                        st.hp.borrow_mut().insert(id, 100_000);
                    }
                    c.execute(
                        sim,
                        &all,
                        &me,
                        &code,
                        if matches!(f, HEAL | SHIELD | F_HONEYPOT) {
                            1
                        } else {
                            5
                        },
                        0,
                        0,
                    );
                    assert!(!c.saved_run, "saved context must not leak past a cast");
                    // Delayed emissions must keep their origin after the live context and scale change.
                    c.scale = 0;
                    for t in 0..=90 {
                        st.tick.set(t);
                        c.step_effects(sim, &all, &me, t);
                        c.step_new(sim, &all, &me, t);
                        c.step_walls(sim, &all, &me, t);
                    }
                })
            };
            let fresh = run(false);
            let loaded = run(true);
            assert_eq!(
                fresh.hits.len(),
                loaded.hits.len(),
                "function {f}, scale {scale}"
            );
            for (a, b) in fresh.hits.iter().zip(&loaded.hits) {
                assert_eq!(
                    (b.0, b.1, b.2),
                    (a.0, a.1 / 10, a.2),
                    "function {f}, scale {scale}"
                );
                damaging.insert(f);
            }
            assert_eq!(fresh.heals, loaded.heals, "healing from {f}");
            assert_eq!(fresh.shields, loaded.shields, "shields from {f}");
            assert_eq!(fresh.cc.len(), loaded.cc.len(), "CC from {f}");
            for (a, b) in fresh.buffs.iter().zip(&loaded.buffs) {
                assert_eq!(
                    b.damage_reflect,
                    a.damage_reflect / 10,
                    "reflection from {f}"
                );
                assert_eq!(b.damaged_reduce, a.damaged_reduce, "defense from {f}");
                assert_eq!(b.duration_tick, a.duration_tick, "duration from {f}");
            }
            assert!(loaded
                .trace
                .iter()
                .filter(|s| s.contains(" raw_ap="))
                .all(|s| s.contains("saved_reload=1")));
        }
    }
    for f in [
        PING,
        SPRAY,
        CHAIN,
        FIREWALL,
        DDOS,
        DDOS_ALL,
        RECURSE,
        FORK,
        KILL9,
        F_CLOUD_DEPLOY,
        F_SERVERLESS,
        F_CRON,
        F_CANARY_DEPLOY,
        F_CHAOS_MONKEY,
        F_BOTNET,
        F_BUFFER_OVERFLOW,
        F_FORK_BOMB,
        F_UDP_FLOOD,
        F_ZERO_DAY,
        F_BRUTE_FORCE,
        F_CUDA_KERNEL,
        F_OVERFIT,
        F_SHARDING,
        F_MAP_REDUCE,
        F_TCP_HANDSHAKE,
        F_WEBSOCKET,
        F_WEBHOOK,
        F_KILL_ALL,
        F_BINARY_SEARCH,
        F_DYNAMIC_PROG,
    ] {
        assert!(
            damaging.contains(&f),
            "fixture must exercise damage from function {f}"
        );
    }
}

#[test]
fn saving_a_fresh_compile_and_loading_a_saved_compile_have_distinct_damage() {
    let me = unit(0, 0, 0, 1000);
    let all = vec![me.clone(), unit(5, 1, 30_000, 1000)];
    let mut c = coder();
    c.rank = Some(ROOT);
    c.root = Some(1); // Guaranteed good saving habits.
    let got = capture(120, |sim, st| {
        c.ship(sim, 0, &all, &me, PING, PY, vec![], false);
        assert!(c.program[0].saved && !c.program[0].reloaded);
        assert_eq!(c.storage.len(), 1);
        c.run_program(sim, &all, &me, 0);
        c.program.clear();
        c.typing = Some(Typing::reload(PING, PY, &[], 100, RELOAD[0]));
        for t in 100..=160 {
            st.tick.set(t);
            c.step_typing(sim, 0, &all, &me, t);
            if c.typing.is_none() {
                break;
            }
        }
        assert!(c.program[0].saved && c.program[0].reloaded);
        let t = st.tick.get();
        c.run_program(sim, &all, &me, t);
    });
    assert_eq!(
        got.hits.iter().map(|h| h.1).collect::<Vec<_>>(),
        vec![31, 3]
    );
    assert!(got.trace[0].contains("saved_reload=0"));
    assert!(got.trace[1].contains("saved_reload=1"));
}

#[test]
fn a_waiting_saved_script_keeps_its_penalty_until_the_trigger_fires() {
    let me = unit(0, 0, 0, 1000);
    let all = vec![me.clone(), unit(5, 1, 30_000, 100)];
    let mut c = coder();
    c.cooldown[F_ZERO_DAY] = 100;
    let got = capture(120, |sim, st| {
        c.ship(sim, 0, &all, &me, F_ZERO_DAY, PY, vec![], true);
        assert!(c.pending_script.as_ref().unwrap().0.reloaded);
        st.tick.set(100);
        c.run_program(sim, &all, &me, 100);
        assert!(c.pending_script.is_none());
    });
    assert_eq!(got.hits[0], (100, 6, AttackTypeV1::DotIgnoreShield.code()));
    assert!(got.trace[0].contains("saved_reload=1"));
}

#[test]
fn saved_kubernetes_only_penalizes_its_replica_and_support_stays_full_strength() {
    let me = unit(0, 0, 0, 1000);
    let all = vec![
        me.clone(),
        unit(1, 0, 20_000, 400),
        unit(5, 1, 30_000, 1000),
    ];
    let mut c = coder();
    let mut kube = compiled(F_KUBERNETES);
    kube.reloaded = true;
    c.program.push(kube);
    let got = capture(600, |sim, _| {
        c.run_one(sim, &all, &me, &compiled(PING), 0);
        c.run_one(sim, &all, &me, &compiled(HEAL), 0);
    });
    assert_eq!(
        got.hits.iter().map(|h| h.1).collect::<Vec<_>>(),
        vec![113, 2]
    );
    assert_eq!(got.heals, vec![332, 166]);
    assert!(got.trace[0].contains("saved_reload=0"));
    assert!(got.trace[1].contains("saved_reload=1"));
}

#[test]
fn immediate_damage_and_burst_are_reduced_across_rigs() {
    for ap in [40, 120, 600] {
        for tier in [0, 2, 4] {
            let me = unit(0, 0, 0, 1000);
            let all = vec![me.clone(), unit(5, 1, 30_000, 1000)];
            for (f, base, ratio, burst) in [
                (PING, 35, 50, false),
                (F_BUFFER_OVERFLOW, 150, 70, true),
                (F_CUDA_KERNEL, 120, 80, true),
                (F_OVERFIT, 140, 60, true),
            ] {
                let mut c = coder();
                c.tiers = [tier; NPARTS];
                let power = LANG[PY].power * c.ghz() / 300 * POWER_BUFF / 100;
                let power = if needs_gpu(f) {
                    power * GPU_POWER[tier] / 100
                } else {
                    power
                };
                let old = (base + ap * ratio / 100) * power / 100;
                let got = capture(ap, |sim, _| {
                    assert!(c.execute(sim, &all, &me, &compiled(f), 5, 0, 0));
                });
                let expected = if burst {
                    old * 65 / 100 * 80 / 100 * 50 / 100
                } else {
                    old * 65 / 100 * 50 / 100
                };
                assert_eq!(got.hits[0].1, expected, "{f} AP={ap} tier={tier}");
            }
        }
    }
}

#[test]
fn packets_keep_their_count_schedule_and_are_not_nerfed_twice() {
    for (f, n, gap, base, ratio) in [
        (DDOS, 20, 3, 4, 6),
        (RECURSE, 8, 6, 12, 15),
        (F_TCP_HANDSHAKE, 3, 8, 30, 25),
    ] {
        let me = unit(0, 0, 0, 1000);
        let all = vec![me.clone(), unit(5, 1, 30_000, 1000)];
        let mut c = coder();
        let got = capture(120, |sim, st| {
            c.execute(sim, &all, &me, &compiled(f), 5, 0, 0);
            assert_eq!(c.hits.len(), n);
            for k in 0..n {
                assert_eq!(c.hits[k].at, k * gap);
            }
            for t in 0..=n * gap {
                st.tick.set(t);
                c.step_effects(sim, &all, &me, t);
            }
        });
        assert_eq!(got.hits.len(), n);
        for (k, h) in got.hits.iter().enumerate() {
            assert_eq!(
                (h.0, h.1),
                (k * gap, amount(120, base, ratio) * 65 / 100 * 50 / 100)
            );
        }
    }
}

#[test]
fn replays_retain_damage_scaling_in_drones_tethers_and_walls() {
    let me = unit(0, 0, 0, 1000);
    let all = vec![me.clone(), unit(5, 1, 18_000, 1000)];
    for (scale, drone_damage, tether_damage) in [(0, 12, 14), (50, 3, 3), (60, 3, 4), (100, 6, 7)] {
        let mut c = coder();
        c.scale = scale;
        let got = capture(120, |sim, st| {
            c.execute(sim, &all, &me, &compiled(FORK), 5, 0, 0);
            assert_eq!(
                c.drones.iter().map(|d| d.1).collect::<Vec<_>>(),
                vec![0, 15]
            );
            for t in [0, 15, 30] {
                st.tick.set(t);
                c.step_effects(sim, &all, &me, t);
            }
        });
        assert_eq!(
            got.hits.iter().map(|h| h.0).collect::<Vec<_>>(),
            vec![0, 15, 30]
        );
        assert!(got.hits.iter().all(|h| h.1 == drone_damage));
        let mut c = coder();
        c.scale = scale;
        let got = capture(120, |sim, st| {
            c.execute(sim, &all, &me, &compiled(F_WEBSOCKET), 5, 0, 0);
            for t in 0..=30 {
                st.tick.set(t);
                c.step_new(sim, &all, &me, t);
            }
        });
        assert_eq!(
            got.hits.iter().map(|h| (h.0, h.1)).collect::<Vec<_>>(),
            vec![(0, tether_damage), (30, tether_damage)]
        );
        let mut c = coder();
        c.scale = scale;
        let got = capture(120, |sim, st| {
            let mut all = all.clone();
            all.push(unit(6, 1, 10_800, 1000));
            c.execute(sim, &all, &me, &compiled(FIREWALL), 5, 0, 0);
            // Existing run power rounds before the amount; replay damage is then reduced once.
            let power = 104 * if scale == 0 { 100 } else { scale } / 100;
            let expected = damage_amount(FIREWALL, (15 + 18) * power / 100, scale, false);
            assert_eq!(c.walls[0].dmg, expected);
            for t in 0..=20 {
                st.tick.set(t);
                c.step_walls(sim, &all, &me, t);
            }
        });
        assert_eq!(
            got.hits.iter().map(|h| h.0).collect::<Vec<_>>(),
            vec![0, 20],
            "scale={scale}, hits={:?}",
            got.hits
        );
        assert_eq!(got.hits[0].1, got.hits[1].1);
    }
}

#[test]
fn percentage_attacks_and_execute_boundaries_cannot_bypass_replay_nerfs() {
    let me = unit(0, 0, 0, 1000);
    for scale in [0, 50, 100] {
        let all = vec![me.clone(), unit(5, 1, 30_000, 160)];
        for (f, expected) in [
            (
                F_ZERO_DAY,
                if scale == 0 {
                    60
                } else {
                    120 * scale / 100 / 2 / 2
                },
            ),
            (
                F_BINARY_SEARCH,
                if scale == 0 {
                    20
                } else {
                    40 * scale / 100 / 2 / 2
                },
            ),
        ] {
            let mut c = coder();
            c.scale = scale;
            let got = capture(120, |sim, _| {
                c.execute(sim, &all, &me, &compiled(f), 5, 0, 0);
            });
            assert_eq!(got.hits[0].1, expected);
            assert_eq!(got.hits[0].2, AttackTypeV1::DotIgnoreShield.code());
        }
    }
    for hp in [79, 80, 81] {
        for scale in [0, 50, 100] {
            let all = vec![me.clone(), unit(5, 1, 30_000, hp)];
            let mut c = coder();
            c.scale = scale;
            let got = capture(120, |sim, st| {
                st.hp.borrow_mut().insert(5, hp);
                c.execute(sim, &all, &me, &compiled(KILL9), 5, 0, 0);
                if hp == 79 && scale == 0 {
                    assert_eq!(st.hp.borrow().get(&5), Some(&40));
                }
            });
            if hp == 79 && scale == 0 {
                assert_eq!(got.hits[0].1, hp / 2);
            } else {
                assert!(
                    got.hits[0].1 < 100,
                    "replay must use ordinary fallback damage"
                );
            }
            c.scan_until = 100;
            assert_eq!(
                c.trigger(KILL9, &all, &me, 0, false, false).is_some(),
                hp < 80
            );
        }
    }
}

#[test]
fn support_and_sign_flipped_healing_keep_their_original_strength() {
    let me = unit(0, 0, 0, 1000);
    let all = vec![
        me.clone(),
        unit(1, 0, 20_000, 400),
        unit(5, 1, 30_000, 1000),
    ];
    let mut c = coder();
    let got = capture(120, |sim, _| {
        c.execute(sim, &all, &me, &compiled(HEAL), 1, 0, 0);
        c.execute(sim, &all, &me, &compiled(SHIELD), 1, 0, 0);
        c.execute(sim, &all, &me, &compiled(INJECT), 5, 0, 0);
        let mut p = compiled(PING);
        p.reloaded = true;
        p.bugs.push(Bug::SignFlip);
        c.execute(sim, &all, &me, &p, 5, 0, 0);
        c.execute(sim, &all, &me, &compiled(F_HONEYPOT), 1, 0, 0);
    });
    assert_eq!(got.heals, vec![amount(120, 80, 40), amount(120, 35, 50)]);
    assert_eq!(got.shields, vec![(amount(120, 120, 50), 180)]);
    assert_eq!(got.cc[0].tick, 60);
    assert!(got
        .buffs
        .iter()
        .any(|b| b.damage_reflect == 10 && b.duration_tick == 180));
}

#[test]
fn daemon_and_cooldown_schedules_are_unchanged() {
    let me = unit(0, 0, 0, 1000);
    let all = vec![me.clone(), unit(5, 1, 30_000, 1000)];
    let mut c = coder();
    c.program.push(compiled(PING));
    let got = capture(120, |sim, st| {
        for t in 0..=120 {
            st.tick.set(t);
            c.run_program(sim, &all, &me, t);
        }
    });
    assert_eq!(
        got.hits.iter().map(|h| h.0).collect::<Vec<_>>(),
        vec![0, 48, 96]
    );
    assert_eq!(
        got.hits.iter().map(|h| h.1).collect::<Vec<_>>(),
        vec![31, 31, 31]
    );
    assert_eq!(cooldown_ticks(PING), 36);
}

#[test]
fn replay_damage_retains_both_reductions_without_halving_support_and_quantum_stays_a_coin_flip() {
    let me = unit(0, 0, 0, 1000);
    let all = vec![
        me.clone(),
        unit(1, 0, 20_000, 400),
        unit(5, 1, 30_000, 1000),
    ];
    for scale in [50, 60, 100] {
        let mut c = coder();
        c.scale = scale;
        let got = capture(120, |sim, _| {
            c.execute(sim, &all, &me, &compiled(PING), 5, 0, 0);
            c.execute(sim, &all, &me, &compiled(HEAL), 1, 0, 0);
            c.execute(sim, &all, &me, &compiled(SHIELD), 1, 0, 0);
            c.execute(sim, &all, &me, &compiled(F_HONEYPOT), 1, 0, 0);
        });
        let power = 104 * scale / 100;
        assert_eq!(
            got.hits[0].1,
            (35 + 60) * power / 100 * 65 / 100 * 50 / 100 * 50 / 100
        );
        assert_eq!(got.heals, vec![(80 + 48) * power / 100]);
        assert_eq!(got.shields, vec![((120 + 60) * power / 100, 180)]);
        assert!(got
            .buffs
            .iter()
            .any(|b| b.damage_reflect == 20 * scale / 100 * 50 / 100 * 50 / 100));
    }
    let mut landed = 0;
    for seed in 1..=30 {
        let mut c = coder();
        c.rng = Rng(seed);
        let got = capture(120, |sim, _| {
            c.execute(sim, &all, &me, &compiled(F_QUANTUM), 5, 0, 0);
        });
        if !got.hits.is_empty() {
            landed += 1;
            // Python, stock GPU (25%), 120 base + 60% AP, same original success chance.
            assert_eq!(
                got.hits[0].1,
                (120 + 72) * (104 * 25 / 100) / 100 * 65 / 100 * 80 / 100 * 50 / 100
            );
        }
    }
    assert!(landed > 0 && landed < 30, "quantum still has both outcomes");
}

#[test]
fn diagnostics_attribute_chain_packets_and_live_scaled_persistent_hits() {
    let me = unit(0, 0, 0, 1000);
    let all = vec![
        me.clone(),
        unit(5, 1, 18_000, 1000),
        unit(6, 1, 30_000, 1000),
        unit(7, 1, 40_000, 1000),
        unit(8, 1, 50_000, 1000),
    ];
    let mut c = coder();
    let got = capture(120, |sim, st| {
        st.tick.set(10);
        c.execute(sim, &all, &me, &compiled(CHAIN), 5, 10, 0);
        st.tick.set(20);
        c.scale = 50;
        c.execute(sim, &all, &me, &compiled(DDOS), 5, 20, 0);
        c.scale = 0;
        for t in 20..=80 {
            st.tick.set(t);
            c.step_effects(sim, &all, &me, t);
        }
        st.tick.set(100);
        c.scale = 50;
        c.execute(sim, &all, &me, &compiled(F_BOTNET), 5, 100, 0);
        c.scale = 0;
        st.ap.set(600);
        c.step_effects(sim, &all, &me, 100);
        st.tick.set(200);
        c.scale = 60;
        c.execute(sim, &all, &me, &compiled(F_WEBSOCKET), 5, 200, 0);
        c.scale = 0;
        st.ap.set(800);
        c.step_new(sim, &all, &me, 200);
    });
    let chain: Vec<_> = got
        .trace
        .iter()
        .filter(|s| s.contains("chain.py "))
        .collect();
    assert_eq!(chain.len(), 4);
    for (i, line) in chain.iter().enumerate() {
        assert!(line.contains(&format!("target {}:", i + 5)));
        assert!(line.contains("raw_ap=24 ") && line.contains("hp=1000->976 "));
        assert!(line.contains("hp_loss=24 killed=false"));
    }
    let packets: Vec<_> = got
        .trace
        .iter()
        .filter(|s| s.contains("delivery=packet "))
        .collect();
    assert_eq!(packets.len(), 20);
    assert!(packets
        .iter()
        .all(|s| s.contains("ddos.py ") && s.contains("cast_tick=20 replay_scale=50 ")));
    let drone = got
        .trace
        .iter()
        .find(|s| s.contains("delivery=drone "))
        .unwrap();
    assert!(drone.contains("botnet.py ") && drone.contains("cast_tick=100 replay_scale=50 "));
    assert!(drone.contains("cast_ap=120 ap_now=600 drones=3 raw_ap=10 "));
    let tether = got
        .trace
        .iter()
        .find(|s| s.contains("delivery=tether "))
        .unwrap();
    assert!(tether.contains("websocket.py ") && tether.contains("cast_tick=200 replay_scale=60 "));
    assert!(tether.contains("cast_ap=600 ap_now=800 ") && tether.contains("raw_ap=17 "));
    assert_eq!(got.trace.len(), got.hits.len());
}

#[test]
fn diagnostics_preserve_support_and_report_nonlethal_direct_hp_and_engine_kill_credit() {
    let me = unit(0, 0, 0, 1000);
    let all = vec![
        me.clone(),
        unit(1, 0, 20_000, 400),
        unit(5, 1, 30_000, 1000),
    ];
    let mut c = coder();
    let got = capture(120, |sim, st| {
        c.execute(sim, &all, &me, &compiled(HEAL), 1, 0, 0);
        let mut ping = compiled(PING);
        ping.bugs.push(Bug::SignFlip);
        c.execute(sim, &all, &me, &ping, 5, 0, 0);
        st.hp.borrow_mut().insert(1, 20);
        let mut heal = compiled(HEAL);
        heal.bugs.push(Bug::SignFlip);
        c.execute(sim, &all, &me, &heal, 1, 0, 0);
        assert_eq!(st.hp.borrow().get(&1), Some(&1));
        st.hp.borrow_mut().insert(5, 0);
        c.on_kill(sim, 0, 0, 5);
    });
    assert_eq!(got.heals, vec![amount(120, 80, 40), amount(120, 35, 50)]);
    assert_eq!(got.trace.len(), 2);
    assert!(got.trace[0].contains("heal.py delivery=direct_hp "));
    assert!(got.trace[0].contains("raw_ap=21 ") && got.trace[0].contains("hp=20->1 "));
    assert!(got.trace[0].contains("hp_loss=19 killed=false"));
    assert!(got.trace[1].contains("engine_credit delivery=kill_credit "));
    assert!(got.trace[1].contains("raw_ap=? attack_type=? hp=?->0 "));
    assert!(got.trace[1].contains("hp_loss=? killed=true"));
}

#[test]
fn diagnostic_presim_suppression_preserves_damage_and_does_not_consume_extra_boosts_or_rng() {
    let me = unit(0, 0, 0, 1000);
    let all = vec![me.clone(), unit(5, 1, 30_000, 1000)];
    let run = |kind: SimOriginKindV1| {
        let mut c = coder();
        c.boost_next = 150;
        let old_rng = c.rng.0;
        let got = capture(120, |sim, st| {
            st.origin.set(kind.code());
            c.execute(sim, &all, &me, &compiled(PING), 5, 0, 0);
        });
        assert_eq!(c.rng.0, old_rng);
        assert_eq!(c.boost_next, 0);
        got
    };
    let live = run(SimOriginKindV1::ClientMatchView);
    let presim = run(SimOriginKindV1::ServerPresim);
    assert_eq!(live.hits, presim.hits);
    assert_eq!(live.hits[0].1, 48);
    assert!(presim.trace.is_empty());
    assert_eq!(live.trace.len(), 1);
    assert!(live.trace[0].contains("power_pct=156 ") && live.trace[0].contains("raw_ap=48 "));
}
