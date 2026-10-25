use raze_lab::{raze_core::*, valorant_guns::Gun};
use std::time::Instant;
fn unit(id: usize, team: usize, x: i64, y: i64, kind: u8) -> Unit {
    Unit {
        id,
        team,
        pos: Point::new(x, y),
        hp: if id == 1 {
            850
        } else if kind == 3 {
            20000
        } else {
            1400
        },
        max_hp: if id == 1 {
            850
        } else if kind == 3 {
            20000
        } else {
            1400
        },
        ap: 50,
        attack: 50,
        visible: true,
        targetable: true,
        kind,
        farmable: kind > 0 && kind < 4,
        ..Default::default()
    }
}
fn run(seed: u64, level: i64, team: usize) -> (serde_json::Value, Vec<u128>) {
    let mut core = Core::default();
    let mut guard = false;
    let mut prevented = 0;
    let enemy = 1 - team;
    let role = (seed - 1) % 5;
    let mut f = Frame {
        automatic: true,
        seed,
        practice_inventory: true,
        profile: Profile {
            values: [level; 13],
            known: true,
        },
        ..Default::default()
    };
    f.units = vec![
        unit(1, team, 300000, 300000, 0),
        unit(2, enemy, 350000 + (seed % 8) as i64 * 10000, 300000, 0),
        unit(3, team, 330000, 330000, 0),
    ];
    let mode = seed % 6;
    if mode == 1 {
        f.units[1].kind = 1;
        f.units[1].farmable = true;
        for n in 5..9 {
            f.units.push(unit(
                n,
                enemy,
                355000 + (n % 2) as i64 * 12000,
                280000 + (n % 3) as i64 * 10000,
                1,
            ));
        }
    }
    if mode == 2 {
        f.units[1].kind = 2;
        f.units[1].farmable = role == 1;
    }
    if mode == 3 {
        f.units.push(unit(99, 2, 380000, 350000, 3));
        f.objective = Some(99);
        f.assignment = Some(Point::new(380000, 350000));
    }
    if mode == 4 {
        f.assignment = Some(Point::new(250000, 420000));
        f.missed_wave = 60;
    }
    if mode == 5 {
        f.blocks.push(Rect {
            min: Point::new(335000, 200000),
            max: Point::new(338000, 360000),
        });
    }
    let mut samples = vec![];
    let (
        mut raw,
        mut actual,
        mut control,
        mut exposure,
        mut missed,
        mut waves,
        mut objectives,
        mut deaths,
    ) = (0i64, 0i64, 0u64, 0u64, 0u64, 0u64, 0i64, 0u64);
    let mut ground = None;
    let mut basic = 0;
    let mut dead = false;
    let mut first_choice = None;
    let mut guides = std::collections::BTreeMap::<usize, (u64, Point, u64)>::new();
    let mut pushes = std::collections::BTreeMap::<usize, (Point, Point, u64, u64)>::new();
    for tick in 1..=900 {
        f.tick = tick;
        f.buy_allowed = tick <= 20;
        f.at_spawn = tick <= 20;
        if let Some(goal) = ground {
            if core.flight.is_none() && f.units[0].hp > 0 {
                let d: Point = Point::sub(goal, f.units[0].pos);
                if d.length() > 1000 {
                    let p = f.units[0].pos.add(d.scale(1020, d.length()));
                    if !f.blocked(p, 10000) {
                        f.units[0].pos = p;
                    }
                }
            }
        }
        for u in f.units.iter_mut().filter(|u| u.team != team && u.kind == 0) {
            u.visible = !(mode == 0 && tick % 240 > 160);
            if let Some((from, to, begins, end)) = pushes.get(&u.id).copied() {
                u.controlled = tick <= end;
                u.pos = from.add(to.sub(from).scale(
                    (tick - begins).min(end - begins) as i64,
                    (end - begins) as i64,
                ));
                if tick >= end {
                    pushes.remove(&u.id);
                }
                continue;
            }
            u.controlled = false;
            if let Some((_, to, end)) = guides.get(&u.id).copied().filter(|v| v.2 > tick) {
                let d = to.sub(u.pos);
                let step = d.capped(1020);
                u.pos = u.pos.add(step);
                u.velocity = step.scale(60, 1);
                let _ = end;
                continue;
            }
            let dy = if tick % 120 < 60 { 180 } else { -180 };
            u.pos.y += dy;
            u.velocity = Point::new(0, dy * 60);
        }
        f.defenders = f
            .units
            .iter()
            .filter(|u| u.team != team && u.champion() && u.alive())
            .map(|u| Defender {
                id: u.id,
                profile: Profile {
                    values: [level; 13],
                    known: true,
                },
                observed: u.visible,
            })
            .collect();
        f.hazards = if tick % 120 < 70 {
            vec![Hazard {
                id: (tick / 120) + 1,
                pos: Point::new(340000, 300000),
                radius: 22000,
                until: (tick / 120) * 120 + 70,
                severity: 75,
            }]
        } else {
            vec![]
        };
        let at = Instant::now();
        let cmds = core.step(&f);
        samples.push(at.elapsed().as_nanos());
        if first_choice.is_none() && !core.continuation.is_empty() {
            first_choice = Some(core.continuation.clone());
        }
        for c in cmds {
            match c {
                Cmd::EnemyGuide {
                    target,
                    rocket,
                    to,
                    until,
                } => {
                    guides.insert(target, (rocket, to, until));
                }
                Cmd::ClearWarning { id } => {
                    guides.retain(|_, g| g.0 != id);
                }
                Cmd::Knockback { target, to, ticks } => {
                    if let Some(u) = f.units.iter().find(|u| u.id == target && u.hp > 0) {
                        pushes.insert(target, (u.pos, to, tick, tick + ticks));
                        control += 1;
                    }
                }
                Cmd::AirProtection { active } => guard = active,
                Cmd::MoveOwner { to } => f.units[0].pos = to,
                Cmd::Guide { to } => ground = Some(to),
                Cmd::Spawn { id, hp, .. } => f.utility.push(UtilityStatus { id, hp }),
                Cmd::Remove { id } => f.utility.retain(|u| u.id != id),
                Cmd::Damage {
                    target,
                    raw: n,
                    self_hit,
                    ..
                } => {
                    if let Some(u) = f.units.iter_mut().find(|u| u.id == target) {
                        let reduced = if target == 1 && guard {
                            n * 85 / 100
                        } else {
                            n
                        };
                        if target == 1 {
                            prevented += n - reduced;
                        }
                        let amount = reduced * 100 / 120;
                        let dealt = amount.min(u.hp);
                        u.hp = (u.hp - amount).max(0);
                        if !self_hit {
                            raw += n;
                            actual += dealt;
                            if u.kind == 3 {
                                objectives += dealt;
                            }
                            if u.hp == 0 && u.kind == 1 {
                                waves += 1;
                            }
                        }
                    }
                }
                Cmd::Push { target, to } => {
                    if let Some(u) = f.units.iter_mut().find(|u| u.id == target) {
                        u.pos = to;
                        control += 1;
                    }
                }
                _ => {}
            }
        }
        if f.units[0].hp > 0 && core.cast.is_none() && tick >= basic {
            let g = Gun::from_index(core.gun);
            let pos = f.units[0].pos;
            if let Some(n) = f.units.iter().position(|u| {
                u.team != team && u.visible && u.damageable() && distance(pos, u.pos) <= g.reach()
            }) {
                let raw_basic = raze_lab::valorant_guns::basic_raw(
                    g,
                    f.units[0].attack,
                    distance(pos, f.units[n].pos),
                );
                let amount = raw_basic * 100 / 120;
                let u = &mut f.units[n];
                let dealt = amount.min(u.hp);
                u.hp = (u.hp - amount).max(0);
                raw += raw_basic;
                actual += dealt;
                basic = tick + (6000 / (100 + g.modifiers().1) as u64).max(1);
            }
        }
        let exposed = f.units.iter().any(|u| {
            u.team != team && u.kind == 0 && u.hp > 0 && distance(u.pos, f.units[0].pos) < 60000
        });
        if exposed {
            exposure += 1;
            if tick % 60 == 0 {
                let amount = core.protected_damage(45);
                prevented += 45 - amount;
                f.units[0].hp = (f.units[0].hp - amount).max(0);
            }
        }
        if f.units[0].hp == 0 && !dead {
            deaths += 1;
            dead = true;
        }
        if tick % 120 == 0
            && f.units
                .iter()
                .any(|u| u.kind == 1 && u.hp > 0 && distance(u.pos, f.units[0].pos) > 100000)
        {
            missed += 1;
        }
    }
    (
        serde_json::json!({"seed":seed,"set":if seed<=24{"development"}else{"holdout"},"team":team,"role":role,"level":level,"firstContinuation":first_choice,"requestedRaw":raw,"labEstimatedDelivered":actual,"core":core.metrics,"exposureTicks":exposure,"missedWaveObservations":missed,"waveUnitsDefeated":waves,"objectiveEstimatedDamage":objectives,"deaths":deaths,"controlRequests":control,"airDamagePreventedEstimate":prevented,"finalHp":f.units[0].hp}),
        samples,
    )
}
fn main() {
    let mut runs = vec![];
    let mut cases = vec![];
    for repetition in 0..5 {
        let mut ns = vec![];
        let mut these = vec![];
        for seed in 1..=36 {
            for level in [20, 50, 80] {
                for team in 0..2 {
                    let (c, t) = run(seed, level, team);
                    these.push(c);
                    ns.extend(t);
                }
            }
        }
        ns.sort_unstable();
        let at = |p: usize| ns[(ns.len() - 1) * p / 100] as f64 / 1000000.;
        runs.push(serde_json::json!({"run":repetition+1,"samples":ns.len(),"medianMs":at(50),"p95Ms":at(95),"p99Ms":at(99)}));
        if repetition == 0 {
            cases = these;
        }
    }
    println!(
        "{}",
        serde_json::json!({"runs":runs,"scenarios":cases,"scope":"Core-only timings. Damage uses a simple 20-resistance Lab model; this is not engine-attributed damage or native-host performance."})
    );
}
