//! Scripted development opponents and simplified mitigation, never engine-attributed results.
use rally_lab::rally_core::*;
use serde_json::json;
use std::collections::BTreeMap;
use std::time::Instant;
fn unit(id: usize, team: usize, x: i64, y: i64) -> Unit {
    Unit {
        id,
        team,
        pos: Point { x, y },
        velocity: Point::default(),
        hp: 5000,
        max_hp: 5000,
        attack: 85,
        radius: 10000,
        champion: true,
        damageable: true,
        visible: true,
        targetable: true,
        farmable: false,
        controlled: false,
        cc_immune: false,
        control_eligible: true,
    }
}
fn frame(seed: u64, profile: [i64; 13]) -> Frame {
    let mut me = unit(1, 0, 150000, 300000);
    me.hp = 1102;
    me.max_hp = 1102;
    me.attack = 104;
    let enemy = unit(2, 1, 215000 + (seed % 6) as i64 * 10000, 300000);
    let ally = unit(3, 0, 175000, 330000);
    let mut f = Frame {
        seed,
        profile,
        manual: false,
        units: vec![me, enemy, ally],
        incoming_verified: true,
        support_verified: true,
        ..Frame::default()
    };
    if seed % 6 == 2 {
        f.blocks.push(Rect {
            min: Point {
                x: 190000,
                y: 265000,
            },
            max: Point {
                x: 197000,
                y: 325000,
            },
        });
    }
    if seed % 6 == 3 {
        f.smoke_disks.push(Disk {
            pos: Point {
                x: 215000,
                y: 315000,
            },
            radius: 18000,
        });
    }
    if seed % 6 == 4 {
        for id in 4..8 {
            let mut u = unit(id, 1, 235000 + (id - 4) as i64 * 6000, 320000);
            u.champion = false;
            u.farmable = true;
            u.hp = 500;
            u.max_hp = 500;
            f.units.push(u);
        }
    }
    if seed % 6 == 5 {
        f.units.push(unit(8, 1, 250000, 330000));
        f.units.push(unit(9, 0, 180000, 285000));
    }
    f
}
fn simulate(seed: u64, profile: [i64; 13]) -> serde_json::Value {
    let mut f = frame(seed, profile);
    let mut c = Core::default();
    if seed % 9 == 7 {
        c.role = Role::Setter;
    } else if seed % 9 == 8 {
        c.role = Role::Receiver;
    }
    let mut shield = 0i64;
    let mut shield_end = 0;
    let mut next_basic = 0;
    let mut next_enemy = 0;
    let mut next_ally = 0;
    let mut serial = 0;
    let (mut requested, mut hp_loss, mut received, mut absorbed, mut ally_contribution) =
        (0, 0, 0, 0, 0);
    let (mut rejected_contacts, mut high_contacts, mut black_hits, mut flow_earned) = (0, 0, 0, 0);
    let mut sources = BTreeMap::<String, i64>::new();
    let mut roles = [0usize; 3];
    let mut hits = vec![];
    let mut receipts = vec![];
    let mut support = vec![];
    for tick in 0..1800u64 {
        f.tick = tick;
        f.hits = std::mem::take(&mut hits);
        f.receipts = std::mem::take(&mut receipts);
        f.support_receipts = std::mem::take(&mut support);
        f.basics.clear();
        if tick >= shield_end {
            shield = 0;
        }
        let old_flow = c.flow;
        f.units[1].visible = !(seed % 4 == 0 && (180..260).contains(&(tick % 600)));
        if seed % 3 != 0 && f.units[1].hp > 0 {
            let dy = if tick % 240 < 120 { 300 } else { -300 };
            let p = Point {
                x: f.units[1].pos.x,
                y: f.units[1].pos.y + dy,
            };
            if f.legal(p, 10000) {
                f.units[1].velocity = Point { x: 0, y: dy };
                f.units[1].pos = p;
            }
        }
        f.uncertainty = if f.units[1].visible {
            0
        } else {
            ((tick % 600).saturating_sub(180) / 3).min(100) as i64
        };
        f.danger = if f.units[1].hp > 0
            && f.units[1].visible
            && f.units[0].pos.distance(f.units[1].pos) < 55000
        {
            40 + (seed % 4) as i64 * 15
        } else {
            0
        };
        if f.units[0].hp <= 0 {
            c.step(&f);
            break;
        }
        let base_attack = 104;
        f.units[0].attack = base_attack
            * (100
                + c.buffs
                    .values()
                    .filter(|b| b.target == 1)
                    .map(|b| b.attack)
                    .sum::<i64>())
            / 100;
        if !c.busy()
            && tick >= next_basic
            && f.units[1].visible
            && f.units[1].hp > 0
            && f.units[0].pos.distance(f.units[1].pos) <= 75000
            && f.clear(f.units[0].pos, f.units[1].pos, 0, false)
        {
            serial += 1;
            f.basics.push(Basic { serial, target: 2 });
            next_basic = tick + 55;
        }
        if tick >= next_enemy && f.danger > 0 && f.clear(f.units[1].pos, f.units[0].pos, 0, false) {
            next_enemy = tick + 60;
            let reduction = c.reduction(tick);
            let amount = 85 * 100 / 126 * (100 - reduction) / 100;
            let blocked = shield.min(amount);
            shield -= blocked;
            absorbed += blocked;
            let loss = (amount - blocked).min(f.units[0].hp);
            f.units[0].hp -= loss;
            received += loss;
            serial += 1;
            receipts.push(Receipt {
                serial,
                source: 2,
                delivered: loss + blocked,
                reduction,
                champion: true,
                verified: true,
                reflected: false,
            });
        }
        if tick >= next_ally
            && f.units[1].visible
            && f.units[1].hp > 0
            && f.units[2].pos.distance(f.units[1].pos) < 80000
        {
            next_ally = tick + 60;
            let bonus = c
                .buffs
                .values()
                .filter(|b| b.target == 3)
                .map(|b| b.attack)
                .sum::<i64>();
            let loss = (70 * (100 + bonus) / 120).min(f.units[1].hp);
            f.units[1].hp -= loss;
            ally_contribution += loss;
            serial += 1;
            support.push(SupportReceipt {
                serial,
                ally: 3,
                champion: true,
                positive: loss > 0,
                verified: true,
            });
        }
        let out = c.step(&f);
        flow_earned += (c.flow - old_flow).max(0);
        roles[match c.role {
            Role::Spiker => 0,
            Role::Setter => 1,
            Role::Receiver => 2,
        }] += 1;
        let mut by_serial = BTreeMap::<u64, Hit>::new();
        for e in out {
            match e {
                Effect::Move { pos } => f.units[0].pos = pos,
                Effect::Shield { amount, ticks } => {
                    shield += amount;
                    shield_end = tick + ticks;
                }
                Effect::Damage {
                    serial,
                    target,
                    amount,
                    source,
                    ..
                } => {
                    requested += amount;
                    *sources.entry(source.clone()).or_default() += amount;
                    if let Some(u) = f.units.iter_mut().find(|u| u.id == target && u.hp > 0) {
                        let loss = (amount * 100 / 120).min(u.hp);
                        u.hp -= loss;
                        hp_loss += loss;
                        let h = by_serial.entry(serial).or_insert(Hit {
                            serial,
                            target,
                            delivered: false,
                            champion: false,
                        });
                        h.delivered |= loss > 0;
                        h.champion |= loss > 0 && u.champion;
                        if source == "black_thunder" && loss > 0 {
                            black_hits += 1;
                        }
                    }
                }
                Effect::Reject { reason } if reason.contains("Contact missed") => {
                    rejected_contacts += 1
                }
                Effect::Visual { name, height, .. }
                    if name == "spike_release" || name == "black_release" =>
                {
                    if height > 4000 {
                        high_contacts += 1;
                    }
                }
                _ => {}
            }
        }
        hits.extend(by_serial.into_values());
        // Scripted stock-like approach is explicitly outside the Rally controller.
        if !c.busy()
            && f.units[1].visible
            && f.units[1].hp > 0
            && f.units[0].pos.distance(f.units[1].pos) > 60000
        {
            let p = f.units[0].pos.toward(f.units[1].pos, 1050);
            if f.clear(f.units[0].pos, p, 10000, false) {
                f.units[0].pos = p;
            }
        }
        if f.units.iter().filter(|u| u.team == 1).all(|u| u.hp <= 0) {
            break;
        }
    }
    json!({"seed":seed,"split":if seed<24 {"development"}else{"holdout"},"requestedRawDamage":requested,"simulatedHpLoss":hp_loss,"simulatedReceivedHpLoss":received,"simulatedShieldAbsorbed":absorbed,"simulatedAllyHpContribution":ally_contribution,"remainingHp":f.units[0].hp,"sources":sources,"roleTicks":roles,"highContacts":high_contacts,"missedContacts":rejected_contacts,"blackHits":black_hits,"flowEarned":flow_earned,"transformed":c.transformed})
}
fn main() {
    let scenarios = (0..36).map(|s| simulate(s, [50; 13])).collect::<Vec<_>>();
    let mut runs = vec![];
    for _ in 0..5 {
        let mut f = frame(5, [80; 13]);
        for id in 10..133 {
            let mut u = unit(
                id,
                1,
                200000 + (id % 10) as i64 * 8000,
                250000 + (id / 10) as i64 * 8000,
            );
            u.hp = 1000000;
            u.max_hp = 1000000;
            f.units.push(u);
        }
        f.units.truncate(128);
        let mut c = Core::default();
        let mut times = vec![];
        for tick in 0..1800 {
            f.tick = tick;
            let start = Instant::now();
            std::hint::black_box(c.step(&f));
            times.push(start.elapsed().as_secs_f64() * 1e6);
        }
        times.sort_by(f64::total_cmp);
        runs.push(json!({"units":f.units.len(),"medianUs":times[900],"p95Us":times[1710],"p99Us":times[1782]}));
    }
    println!(
        "{}",
        json!({"rulesRevision":rally_lab::rally_core::data::RULES_REVISION,"scope":"Scripted development simulation with simplified mitigation. Receipts are simulated; not engine-attributed damage, matched peer balance or live acceptance.","scenarios":scenarios,"release128UnitStepFiveRuns":runs})
    );
}
