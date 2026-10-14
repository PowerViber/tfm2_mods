#![allow(dead_code)]
#[path = "../../../tfm2_custom_ai/src/magnus_core.rs"]
mod magnus_core;
#[path = "../../../tfm2_custom_ai/src/magnus_data.rs"]
mod magnus_data;
#[path = "../../../tfm2_custom_ai/src/magnus_fixtures.rs"]
mod magnus_fixtures;
use magnus_core::*;
fn raw(input: &Input, rects: &[[i64; 4]], rank: usize) -> Vec<i64> {
    let me = input.owner;
    let (a, p, x, y, h) = match input.action {
        Some(Action::Deploy { heading }) => (0, 0, 0, 0, heading as i64),
        Some(Action::Move { piece, to }) => (1, piece as i64, to.x as i64, to.y as i64, 0),
        Some(Action::Castle { piece }) => (2, piece as i64, 0, 0, 0),
        Some(Action::Dismiss) => (3, 0, 0, 0, 0),
        None => (-1, 0, 0, 0, 0),
    };
    let mut v = vec![
        input.tick as i64,
        (me.hp > 0) as i64,
        me.team as i64,
        me.x,
        me.y,
        me.hp,
        me.max_hp,
        me.attack,
        me.ap,
        input.controlled as i64,
        input.automatic as i64,
        a,
        p,
        x,
        y,
        h,
        rank as i64,
        input.fighters.len() as i64,
        rects.len() as i64,
        input.captures.len() as i64,
    ];
    v.extend(input.piece_hp.iter().map(|n| n.unwrap_or(-1)));
    for f in input.fighters {
        v.extend([
            f.id as i64,
            f.team as i64,
            f.x,
            f.y,
            f.hp,
            f.max_hp,
            f.attack,
            f.ap,
            f.champion as i64,
            f.visible as i64,
            f.targetable as i64,
            (f.tank as i64) | ((f.ranged as i64) << 1) | ((f.friendly_chess as i64) << 2),
        ]);
    }
    for r in rects {
        v.extend(r);
    }
    for c in input.captures {
        v.extend([
            c.piece as i64,
            c.by.map_or(-1, |id| id as i64),
            c.combat as i64,
        ]);
    }
    v
}
fn vectors() {
    for id in 0..6 {
        let mut f = magnus_fixtures::historical(id);
        for tick in 0..=f.end {
            for &(t, id, c) in &f.relocate {
                if t == tick {
                    let (x, y) = f.core.board.unwrap().world(c);
                    let foe = f.fighters.iter_mut().find(|e| e.id == id).unwrap();
                    foe.x = x;
                    foe.y = y;
                }
            }
            let caps = f
                .captures
                .iter()
                .filter(|c| c.0 == tick)
                .map(|c| Capture {
                    piece: c.1,
                    by: Some(5),
                    combat: true,
                })
                .collect::<Vec<_>>();
            let wall = |x, y| {
                x < 0
                    || y < 0
                    || x >= 960_000
                    || y >= 960_000
                    || f.walls
                        .iter()
                        .any(|r| x >= r[0] && y >= r[1] && x <= r[2] && y <= r[3])
            };
            let input = Input {
                tick,
                owner: &f.me,
                fighters: &f.fighters,
                wall: &wall,
                captures: &caps,
                piece_hp: [None; 7],
                controlled: false,
                automatic: false,
                action: f.moves.iter().find(|m| m.0 == tick).map(|m| Action::Move {
                    piece: m.1,
                    to: m.2,
                }),
            };
            let commands = f.core.step(&input);
            println!(
                "{{\"fixture\":{id},\"values\":{:?},\"expected\":{}}}",
                raw(&input, &f.walls, 3),
                f.core.json(&commands)
            );
        }
    }
    // Deployment, random layouts, blockers, interrupt, castling, cleanup and cooldown
    // snapshots supplement the six complete historical trajectories.
    for seed in 1..=16 {
        let rank = ((seed - 1) % 8) as usize;
        let mut c = Core::new(seed, 1, rank);
        let mut me = magnus_fixtures::owner();
        let b = Board {
            x: me.x,
            y: me.y,
            heading: 0,
            serial: 0,
        };
        let mut fs = vec![
            magnus_fixtures::enemy(b, 2, Cell::new(4, 4)),
            magnus_fixtures::enemy(b, 3, Cell::new(6, 6)),
        ];
        for tick in 0..=200 {
            if tick == 170 {
                me.hp = 0;
            }
            if tick == 175 {
                me.hp = 800;
            }
            if tick == 120 {
                fs[0].visible = false;
            }
            let input = Input {
                tick,
                owner: &me,
                fighters: &fs,
                wall: &|x, y| x < 0 || y < 0 || x >= 960_000 || y >= 960_000,
                captures: &[],
                piece_hp: [None; 7],
                controlled: (85..95).contains(&tick),
                automatic: seed > 8,
                action: match tick {
                    0 => Some(Action::Deploy { heading: 0 }),
                    61 => Some(Action::Castle { piece: 0 }),
                    81 => Some(Action::Move {
                        piece: 4,
                        to: Cell::new(3, 2),
                    }),
                    100 => Some(Action::Dismiss),
                    110 => Some(Action::Deploy { heading: 0 }),
                    _ => None,
                },
            };
            let commands = c.step(&input);
            println!(
                "{{\"seed\":{seed},\"rank\":{rank},\"values\":{:?},\"expected\":{}}}",
                raw(&input, &[], rank),
                c.json(&commands)
            );
        }
    }
}
fn scenarios() {
    println!("[");
    for scenario in 0..36 {
        let holdout = scenario >= 24;
        let seed = if holdout {
            50003 + scenario as u64 * 19927
        } else {
            101 + scenario as u64 * 31
        };
        let rank = scenario % 8;
        let mut core = Core::new(seed, 1, rank);
        let me = magnus_fixtures::owner();
        let b = Board {
            x: me.x,
            y: me.y,
            heading: 0,
            serial: 1,
        };
        let mut fs = vec![
            magnus_fixtures::enemy(b, 2, Cell::new(4, 3)),
            magnus_fixtures::enemy(b, 3, Cell::new(6, 5)),
            magnus_fixtures::enemy(b, 4, Cell::new(2, 5)),
        ];
        for f in &mut fs {
            f.hp = 3000;
            f.max_hp = 3000;
        }
        let start = std::time::Instant::now();
        let mut max_ns = 0;
        let (mut living_ticks, mut fx_count) = (0, 0);
        for tick in 0..1200 {
            let mut hp = [None; 7];
            let mut caps = vec![];
            for (i, f) in fs.iter_mut().enumerate() {
                let x = 3 + i as i8 + (if (tick / 150 + i) % 2 == 0 { 0 } else { 1 });
                let y = 3 + i as i8;
                let (p, q) = b.world(Cell::new(x, y));
                f.x = p;
                f.y = q;
            }
            if tick % 90 == 0 {
                for i in 0..7 {
                    let p = &core.pieces[i];
                    if p.hp > 0 {
                        let (x, y) = b.world(p.cell);
                        let n = fs
                            .iter()
                            .filter(|f| f.hp > 0 && (f.x - x).abs() + (f.y - y).abs() < 55000)
                            .count() as i64;
                        let remaining = (p.hp - n * 39).max(0);
                        hp[i] = Some(remaining);
                        if remaining == 0 {
                            caps.push(Capture {
                                piece: i,
                                by: Some(2),
                                combat: true,
                            });
                        }
                    }
                }
            }
            let input = Input {
                tick,
                owner: &me,
                fighters: &fs,
                wall: &|x, y| x < 0 || y < 0 || x >= 960_000 || y >= 960_000,
                captures: &caps,
                piece_hp: hp,
                controlled: false,
                automatic: true,
                action: None,
            };
            let s = std::time::Instant::now();
            let commands = core.step(&input);
            max_ns = max_ns.max(s.elapsed().as_nanos());
            for cmd in commands {
                match cmd {
                    Cmd::Damage { target, amount, .. } => {
                        if let Some(f) = fs.iter_mut().find(|f| f.id == target) {
                            f.hp = (f.hp - amount).max(0)
                        }
                    }
                    Cmd::Fx { .. } => fx_count += 1,
                    _ => {}
                }
            }
            living_ticks += core.pieces.iter().filter(|p| p.hp > 0).count();
        }
        if scenario > 0 {
            println!(",");
        }
        println!("{{\"seed\":{seed},\"holdout\":{holdout},\"rank\":{rank},\"pieceSurvival\":{:.5},\"checkmates\":{},\"challengeCompletions\":{},\"rawDamage\":{},\"captures\":{},\"commands\":{:?},\"effects\":{fx_count},\"ms\":{},\"maxStepUs\":{}}}",living_ticks as f64/(1200.0*7.0),core.stats.mates,core.stats.rewards,core.stats.damage,core.stats.captures,core.stats.commands,start.elapsed().as_millis(),max_ns/1000);
    }
    println!("]");
}
fn main() {
    if std::env::args().any(|a| a == "--scenarios") {
        scenarios()
    } else {
        vectors()
    }
}
