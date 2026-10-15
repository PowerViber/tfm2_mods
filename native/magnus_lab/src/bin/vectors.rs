//! Reproducible native trajectories, development/holdout scenarios and core timings.
#![allow(dead_code)]
#[path = "../../../tfm2_custom_ai/src/magnus_core.rs"]
mod magnus_core;
#[path = "../../../tfm2_custom_ai/src/magnus_data.rs"]
mod magnus_data;
#[path = "../../../tfm2_custom_ai/src/magnus_fixtures.rs"]
mod magnus_fixtures;
#[path = "../../../tfm2_custom_ai/src/magnus_protocol.rs"]
mod magnus_protocol;
use magnus_core::*;
use magnus_protocol::Frame;
use std::{
    collections::BTreeMap,
    io::{BufWriter, Write},
    time::Instant,
};
fn vector(
    w: &mut impl Write,
    group: &str,
    fixture: Option<usize>,
    seed: u64,
    rank: usize,
    values: &[i64],
    expected: &str,
) {
    writeln!(w,"{{\"group\":{},\"fixture\":{},\"seed\":{seed},\"rank\":{rank},\"values\":{:?},\"expected\":{expected}}}",quote(group),fixture.map_or_else(||"null".into(),|n|n.to_string()),values).unwrap();
}
fn trajectories(w: &mut impl Write) {
    for id in 0..12 {
        let mut f = magnus_fixtures::fixture(id).unwrap();
        for tick in 0..=f.end {
            f.prepare(tick);
            let values = f.frame.values();
            let (expected, commands) = f.frame.run(&mut f.core);
            vector(
                w,
                &format!("fixture/{id}"),
                Some(id),
                123,
                3,
                &values,
                &expected,
            );
            f.frame.apply(&commands);
        }
        assert!(
            f.core.stats.rewards > 0,
            "Historical trajectory {id} failed"
        );
    }
    for rank in 0..8 {
        for automatic in [false, true] {
            let seed = 100 + rank as u64;
            let mut core = Core::new(seed, 1, rank);
            let mut f = Frame::new(0, magnus_fixtures::owner());
            let b = Board {
                x: f.owner.x,
                y: f.owner.y,
                heading: 0,
                serial: 0,
            };
            f.fighters = vec![
                magnus_fixtures::enemy(b, 2, Cell::new(6, 4)),
                magnus_fixtures::enemy(b, 3, Cell::new(2, 6)),
            ];
            f.automatic = automatic;
            f.rank = rank;
            for tick in 0..=360 {
                f.tick = tick;
                f.action = None;
                if !automatic {
                    if tick == 0 {
                        f.action = Some(Action::Deploy { heading: 0 });
                    } else if tick == 70 {
                        f.action = Some(Action::Summon {
                            kind: Kind::Pawn,
                            to: Cell::new(17, 12),
                            direction: 0,
                        });
                    } else if tick == 110 {
                        f.action = Some(Action::Summon {
                            kind: Kind::Knight,
                            to: Cell::new(15, 12),
                            direction: 0,
                        });
                    } else if tick % 50 == 0 {
                        if let Some(piece) = core.pieces.values().next() {
                            let wall = |x, y| f.blocked(x, y);
                            let sight = |a, b, c, d| f.sight_clear(a, b, c, d);
                            let input = Input {
                                tick,
                                owner: &f.owner,
                                fighters: &f.fighters,
                                wall: &wall,
                                sight: &sight,
                                captures: &[],
                                hits: &f.hits,
                                piece_hp: &[],
                                controlled: false,
                                automatic: false,
                                action: None,
                            };
                            if let Some(to) = core.destinations(piece.id, &input).first() {
                                f.action = Some(Action::Move {
                                    piece: piece.id,
                                    to: *to,
                                });
                            }
                        }
                    }
                }
                let values = f.values();
                let (expected, commands) = f.run(&mut core);
                vector(
                    w,
                    &format!("rank/{rank}/{automatic}"),
                    None,
                    seed,
                    rank,
                    &values,
                    &expected,
                );
                f.apply(&commands);
            }
        }
    }
}
fn scenario(seed: u64, holdout: bool) -> String {
    let rank = seed as usize % 8;
    let mut core = Core::new(seed, 1, rank);
    let mut f = Frame::new(0, magnus_fixtures::owner());
    let b = Board {
        x: f.owner.x,
        y: f.owner.y,
        heading: 0,
        serial: 0,
    };
    f.fighters = (0..4)
        .map(|n| {
            let mut foe =
                magnus_fixtures::enemy(b, n + 2, Cell::new(1 + n as i32 * 2, 3 + n as i32 % 2));
            foe.tank = n == 2;
            foe.ranged = n == 3;
            foe
        })
        .collect();
    f.automatic = true;
    f.rank = rank;
    let mut stun = BTreeMap::new();
    let (mut peak, mut pieces_seen, mut survival_sum, mut samples) = (0, 0, 0.0, 0);
    let mut times = vec![];
    let mut planning = vec![];
    for tick in 0..1800 {
        f.tick = tick;
        if tick == 900 && seed % 3 == 0 {
            f.owner.x += TILE * 9;
        }
        if tick == 1260 && seed % 3 == 0 {
            f.owner.x -= TILE * 9;
        }
        for (n, foe) in f.fighters.iter_mut().enumerate() {
            if tick >= *stun.get(&foe.id).unwrap_or(&0) && foe.hp > 0 {
                let phase = tick as f64 / (110.0 + seed as f64 % 13.0) + n as f64;
                let c = Cell::new(13 + n as i32 * 2, 16);
                foe.x = c.world().0 + (phase.sin() * TILE as f64) as i64;
                foe.y = c.world().1 + (phase.cos() * TILE as f64) as i64;
            }
        }
        if tick % 90 == 0 {
            for foe in f.fighters.iter().filter(|f| f.hp > 0) {
                if tick < *stun.get(&foe.id).unwrap_or(&0) {
                    continue;
                }
                if let Some(p) = core
                    .pieces
                    .values()
                    .min_by_key(|p| p.cell.distance(foe.cell()))
                {
                    if p.cell.distance(foe.cell()) <= 2 {
                        let damage = if core
                            .reward
                            .as_ref()
                            .is_some_and(|r| r.id == 4 && tick < r.until)
                        {
                            29
                        } else {
                            39
                        };
                        let hp = (p.hp - damage).max(0);
                        f.hp.push((p.id, hp));
                        if hp == 0 {
                            f.captures.push(Capture {
                                piece: p.id,
                                by: Some(foe.id),
                                combat: true,
                            });
                        }
                    }
                }
            }
        }
        let decision = core.pending.is_none();
        let start = Instant::now();
        let commands = f.step(&mut core);
        let us = start.elapsed().as_micros() as u64;
        times.push(us);
        if decision {
            planning.push(us);
        }
        for c in &commands {
            if let Cmd::Stun { target, ticks } = c {
                stun.insert(*target, tick + ticks);
            }
        }
        f.apply(&commands);
        peak = peak.max(core.pieces.len());
        pieces_seen = pieces_seen.max(core.next_id);
        if tick % 60 == 0 {
            samples += 1;
            survival_sum += core.pieces.len() as f64 / core.next_id.max(1) as f64;
        }
    }
    times.sort_unstable();
    planning.sort_unstable();
    format!("{{\"seed\":{seed},\"holdout\":{holdout},\"rank\":{rank},\"pieceSurvival\":{:.4},\"checkmates\":{},\"fieldCheckmates\":{},\"challengeCompletions\":{},\"rawDamage\":{},\"commands\":{:?},\"peakArmy\":{peak},\"piecesCreated\":{pieces_seen},\"p95StepUs\":{},\"maxStepUs\":{},\"p95PlanningUs\":{}}}",survival_sum/samples as f64,core.stats.mates,core.stats.field_mates,core.stats.rewards,core.stats.damage,core.stats.commands,times[times.len()*95/100],times.last().unwrap(),planning.get(planning.len()*95/100).unwrap_or(&0))
}
fn stress(count: usize) -> String {
    let mut core = Core::new(999, 1, 7);
    core.reset_at = 1_000_000;
    core.pawn_at = 1_000_000;
    core.develop_at = 1_000_000;
    for i in 0..count {
        let kind = Kind::from_code(i % 5).unwrap();
        core.add_piece(
            kind,
            Cell::new(3 + (i % 16) as i32, 22 + (i / 16) as i32),
            800 * kind.hp_pct() / 100,
            None,
            0,
        );
    }
    let mut f = Frame::new(0, magnus_fixtures::owner());
    f.rank = 7;
    f.automatic = true;
    f.fighters = vec![magnus_fixtures::enemy(
        Board {
            x: 20 * TILE,
            y: 20 * TILE,
            heading: 0,
            serial: 0,
        },
        2,
        Cell::new(4, 0),
    )];
    let (mut times, mut planning) = (vec![], vec![]);
    for tick in 0..960 {
        f.tick = tick;
        let decision = core.pending.is_none();
        let start = Instant::now();
        let commands = f.step(&mut core);
        let us = start.elapsed().as_micros() as u64;
        times.push(us);
        if decision {
            planning.push(us);
        }
        f.apply(&commands);
    }
    times.sort_unstable();
    planning.sort_unstable();
    format!("{{\"pieces\":{count},\"finalArmy\":{},\"p95StepUs\":{},\"maxStepUs\":{},\"p95PlanningUs\":{},\"commands\":{:?}}}",core.pieces.len(),times[times.len()*95/100],times.last().unwrap(),planning[planning.len()*95/100],core.stats.commands)
}
fn main() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let out = root.join("backups/magnus-vectors-v2.jsonl");
    std::fs::create_dir_all(out.parent().unwrap()).unwrap();
    let mut w = BufWriter::new(std::fs::File::create(&out).unwrap());
    trajectories(&mut w);
    w.flush().unwrap();
    let mut scenarios = vec![];
    for n in 0..36 {
        let holdout = n >= 24;
        scenarios.push(scenario(
            if holdout {
                1000 + n as u64
            } else {
                200 + n as u64
            },
            holdout,
        ));
    }
    std::fs::write(
        root.join("docs/magnus-scenarios.json"),
        format!("[{}]\n", scenarios.join(",\n")),
    )
    .unwrap();
    let stress: [String; 3] = [32, 64, 128].map(stress);
    std::fs::write(
        root.join("docs/magnus-stress.json"),
        format!("[{}]\n", stress.join(",\n")),
    )
    .unwrap();
    println!("Native version 2 vectors: {}", out.display());
    for line in stress {
        println!("{line}");
    }
}
