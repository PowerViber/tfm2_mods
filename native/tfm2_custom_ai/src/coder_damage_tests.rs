//! Exercise native damage emissions, queues and support through the stable host API.
use super::*;
use mod_api_stable::{EntityHandleV1, SimCtxV1, SimVtableV1, StatV1};
use std::{
    cell::{Cell, RefCell},
    ffi::c_void,
    mem::size_of,
};

#[derive(Default)]
struct Host {
    tick: Cell<usize>,
    ap: usize,
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
}
unsafe extern "C" fn tick(p: *const c_void) -> usize {
    (*(p as *const Host)).tick.get()
}
unsafe extern "C" fn valid(_: *const c_void, _: EntityHandleV1) -> bool {
    true
}
unsafe extern "C" fn stat(p: *const c_void, _: EntityHandleV1, out: *mut StatV1) -> bool {
    *out = StatV1 {
        magic_power: (*(p as *const Host)).ap,
        ..StatV1::default()
    };
    true
}
unsafe extern "C" fn damage(p: *mut c_void, _: usize, _: usize, _: usize, n: usize, kind: u32) {
    let s = &*(p as *const Host);
    s.hits.borrow_mut().push((s.tick.get(), n, kind));
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
    let state = Host {
        ap,
        ..Host::default()
    };
    // Nullable vtable slots model absent capabilities. Interior mutability lets callbacks share host state safely.
    let mut table: SimVtableV1 = unsafe { std::mem::zeroed() };
    table.size = size_of::<SimVtableV1>();
    table.tick = Some(tick);
    table.entity_is_valid = Some(valid);
    table.entity_stat = Some(stat);
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
    }
}
fn amount(ap: usize, base: usize, ratio: usize) -> usize {
    (base + ap * ratio / 100) * 104 / 100
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
                    old * 65 / 100 * 80 / 100
                } else {
                    old * 65 / 100
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
            assert_eq!((h.0, h.1), (k * gap, amount(120, base, ratio) * 65 / 100));
        }
    }
}

#[test]
fn replays_retain_damage_scaling_in_drones_tethers_and_walls() {
    let me = unit(0, 0, 0, 1000);
    let all = vec![me.clone(), unit(5, 1, 18_000, 1000)];
    for scale in [0, 50, 60, 100] {
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
        assert!(got.hits.iter().all(|h| h.1 == persistent_damage(39, scale)));
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
            vec![
                (0, persistent_damage(44, scale)),
                (30, persistent_damage(44, scale))
            ]
        );
        let mut c = coder();
        c.scale = scale;
        let got = capture(120, |sim, st| {
            let mut all = all.clone();
            all.push(unit(6, 1, 10_800, 1000));
            c.execute(sim, &all, &me, &compiled(FIREWALL), 5, 0, 0);
            // Existing run power rounds before the amount; replay damage is then reduced once.
            let power = 104 * if scale == 0 { 100 } else { scale } / 100;
            let expected = damage_amount(FIREWALL, (15 + 18) * power / 100, scale);
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
            (F_ZERO_DAY, replay_amount(120, scale)),
            (F_BINARY_SEARCH, replay_amount(40, scale)),
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
            let got = capture(120, |sim, _| {
                c.execute(sim, &all, &me, &compiled(KILL9), 5, 0, 0);
            });
            if hp == 79 && scale == 0 {
                assert_eq!(got.hits[0].1, hp + 1000);
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
        .any(|b| b.damage_reflect == 20 && b.duration_tick == 180));
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
        vec![63, 63, 63]
    );
    assert_eq!(cooldown_ticks(PING), 36);
}

#[test]
fn replay_damage_is_halved_without_halving_support_and_quantum_stays_a_coin_flip() {
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
        assert_eq!(got.hits[0].1, (35 + 60) * power / 100 * 65 / 100 * 50 / 100);
        assert_eq!(got.heals, vec![(80 + 48) * power / 100]);
        assert_eq!(got.shields, vec![((120 + 60) * power / 100, 180)]);
        assert!(got
            .buffs
            .iter()
            .any(|b| b.damage_reflect == 20 * scale / 100 * 50 / 100));
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
                (120 + 72) * (104 * 25 / 100) / 100 * 65 / 100 * 80 / 100
            );
        }
    }
    assert!(landed > 0 && landed < 30, "quantum still has both outcomes");
}
