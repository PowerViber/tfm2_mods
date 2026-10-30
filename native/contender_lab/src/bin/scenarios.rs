//! Development simulator, not host acceptance or a balance comparison with another champion.
use contender_lab::contender_core::*;
use serde_json::json;
use std::time::Instant;
fn frame(seed: u64, profile: [i64; 13]) -> Frame {
    let owner = Unit {
        id: 1,
        team: 0,
        pos: Point {
            x: 200000,
            y: 200000,
        },
        hp: 1102,
        max_hp: 1102,
        attack: 110,
        radius: 10000,
        champion: true,
        visible: true,
        targetable: true,
        cc_immune: false,
        farmable: false,
        velocity: Point::default(),
    };
    let mut enemy = owner.clone();
    enemy.id = 2;
    enemy.team = 1;
    enemy.pos.x += 18000 + (seed % 5) as i64 * 7000;
    enemy.attack = 65 + (seed % 4) as i64 * 15;
    enemy.hp = 1800;
    enemy.max_hp = 1800;
    Frame {
        version: 1,
        tick: 0,
        owner: 1,
        units: vec![owner, enemy],
        blocks: vec![],
        bounds: map(),
        profile,
        manual: false,
        action: None,
        skill1: true,
        skill2: true,
        ultimate: true,
        interrupted: false,
        rooted: false,
        receipts: vec![],
        basics: vec![],
        arena_started: None,
        arena_ended: false,
        basics_validated_by_host: false,
        dealt_at: None,
        duel_unavailable: vec![],
    }
}
fn simulate(seed: u64, profile: [i64; 13]) -> serde_json::Value {
    let mut f = frame(seed, profile);
    let mut c = Core::default();
    let mut shield = 0;
    let mut shield_until = 0;
    let mut guard = 0;
    let mut dodge = 0;
    let mut stunned = 0;
    let mut slow = 0;
    let mut next_attack = 0;
    let mut damage = 0;
    let mut requested = 0;
    let mut received = 0;
    let mut casts = 0;
    let mut serial = 0;
    let mut entry = None;
    let mut first = None;
    let mut max_candidates = 0;
    let mut ult = 0;
    for tick in 0..1800u64 {
        f.tick = tick;
        f.receipts.clear();
        f.basics.clear();
        f.arena_started = None;
        f.arena_ended = false;
        f.dealt_at = None;
        if let Some((until, positions)) = entry {
            if tick >= until || f.units.iter().any(|u| u.hp <= 0) {
                for (u, p) in f.units.iter_mut().zip(positions) {
                    u.pos = p;
                }
                f.bounds = map();
                f.arena_ended = true;
                entry = None;
            }
        }
        if tick >= shield_until {
            shield = 0
        }
        if f.units.iter().any(|u| u.hp <= 0) {
            break;
        }
        f.units[1].visible = !(seed % 6 == 0 && (180..240).contains(&tick));
        let range = f.units[0].pos.distance2(f.units[1].pos);
        if tick >= stunned && range <= 30000_i128.pow(2) && tick >= next_attack {
            next_attack = tick + if tick < slow { 64 } else { 54 };
            let raw = f.units[1].attack;
            let value = if tick < dodge {
                0
            } else {
                raw * 100 / 126 * if tick < guard { 60 } else { 100 } / 100
            };
            let absorbed = shield.min(value);
            shield -= absorbed;
            let n = (value - absorbed).min(f.units[0].hp);
            f.units[0].hp -= n;
            received += n;
            serial += 1;
            f.receipts.push(Receipt {
                serial,
                source: 2,
                champion: true,
                positive_damage: value > 0,
            });
        }
        if !c.busy() && range <= 23000_i128.pow(2) {
            f.basics.push(Basic {
                serial: tick + 1,
                target: 2,
            });
        }
        let effects = c.step(&f);
        max_candidates = max_candidates.max(c.decision.candidates);
        for e in effects {
            match e {
                Effect::Damage {
                    target: 2, amount, ..
                } => {
                    requested += amount;
                    let n = (amount * 100 / 124).min(f.units[1].hp);
                    f.units[1].hp -= n;
                    damage += n;
                    c.dealt(tick, true, n > 0);
                }
                Effect::Shield { amount, ticks } => {
                    shield += amount;
                    shield_until = tick + ticks;
                }
                Effect::Defence { ticks, .. } => guard = tick + ticks,
                Effect::Dodge { ticks } => dodge = tick + ticks,
                Effect::Move { pos } => f.units[0].pos = pos,
                Effect::Airborne { ticks, .. } => stunned = tick + ticks,
                Effect::AttackSlow { ticks, .. } => slow = tick + ticks,
                Effect::Pose { .. } => {
                    casts += 1;
                    if first.is_none() {
                        first = Some(tick);
                    }
                }
                Effect::Transfer {
                    bounds, duration, ..
                } => {
                    let positions = [f.units[0].pos, f.units[1].pos];
                    f.bounds = bounds;
                    let x = (bounds.min.x + bounds.max.x) / 2;
                    let y = (bounds.min.y + bounds.max.y) / 2;
                    f.units[0].pos = Point { x: x - 24000, y };
                    f.units[1].pos = Point { x: x + 24000, y };
                    entry = Some((tick + duration, positions));
                    c.arena_target = Some(2);
                    c.arena_until = tick + duration;
                    ult += 1;
                }
                _ => {}
            }
        }
        // Simulator approach; native movement remains engine owned.
        if !c.busy() && f.units[1].visible {
            let from = f.units[0].pos;
            let p = from.toward(f.units[1].pos, 950);
            if from.distance2(f.units[1].pos) > 18000_i128.pow(2) && legal_move(&f, from, p, 10000)
            {
                f.units[0].pos = p;
            }
        }
    }
    json!({"seed":seed,"set":if seed<24{"development"}else{"holdout"},"requestedRawDamage":requested,"mitigatedHpLoss":damage,"receivedHpLoss":received,"survivingHp":f.units[0].hp,"enemyHp":f.units[1].hp,"resolvedPoses":casts,"firstActionTick":first,"mainEvents":ult,"maxCandidates":max_candidates,"selected":c.decision.selected})
}
fn main() {
    let development: Vec<_> = (0..36).map(|seed| simulate(seed, [50; 13])).collect();
    let attributes: Vec<_> = (0..13)
        .map(|stat| {
            let pairs: Vec<_> = (0..3)
                .map(|case| {
                    let mut low = [50; 13];
                    let mut high = low;
                    low[stat] = 20;
                    high[stat] = 80;
                    json!({"case":case,"low":simulate(case,low),"high":simulate(case,high)})
                })
                .collect();
            json!({"stat":stat,"pairs":pairs})
        })
        .collect();
    let mut runs = vec![];
    for _ in 0..5 {
        let mut f = frame(3, [80; 13]);
        f.units[0].hp = 100_000_000;
        f.units[0].max_hp = 100_000_000;
        for id in 3..129 {
            let mut u = f.units[1].clone();
            u.id = id;
            u.pos.x += (id % 10) as i64 * 7000;
            u.pos.y += (id / 10) as i64 * 7000;
            f.units.push(u);
        }
        let mut c = Core::default();
        let mut micros = vec![];
        for tick in 0..1800 {
            f.tick = tick;
            let now = Instant::now();
            std::hint::black_box(c.step(&f));
            micros.push(now.elapsed().as_secs_f64() * 1e6);
        }
        micros.sort_by(f64::total_cmp);
        runs.push(json!({"medianUs":micros[900],"p95Us":micros[1710],"p99Us":micros[1782]}));
    }
    println!(
        "{}",
        json!({"scope":"Shared-core development simulator; synthetic enemy and approach model; not native host/live balance acceptance","scenarios":development,"attributePairs":attributes,"release128UnitStepFiveRuns":runs})
    );
}
