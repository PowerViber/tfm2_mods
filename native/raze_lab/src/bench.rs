use raze_lab::raze_data::BODY_RADIUS;
use raze_lab::{raze_core::*, valorant_guns::Gun};
use std::time::Instant;
fn unit(id: usize, team: usize, x: i64, y: i64, kind: u8) -> Unit {
    Unit {
        id,
        team,
        pos: Point::new(x, y),
        hp: if id == 1 {
            935
        } else if kind == 3 {
            20000
        } else {
            1400
        },
        max_hp: if id == 1 {
            935
        } else if kind == 3 {
            20000
        } else {
            1400
        },
        ap: if id == 1 { 65 } else { 50 },
        attack: if id == 1 { 60 } else { 50 },
        defence: if id == 1 { 27 } else { 20 },
        mr: if id == 1 { 22 } else { 20 },
        attack_range: 30000,
        move_speed: 1020,
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
    let (mut airborne_ticks, mut landing_guard_ticks) = (0u64, 0u64);
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
    if mode == 0 {
        f.units
            .push(unit(6, enemy, f.units[1].pos.x + 18000, 320000, 0));
        f.units
            .push(unit(7, enemy, f.units[1].pos.x + 24000, 280000, 0));
    }
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
    let (
        mut weapon_spent,
        mut armor_spent,
        mut utility_spent,
        mut followup_damage,
        mut entries_with_followup,
    ) = (0i64, 0i64, 0i64, 0i64, 0u64);
    let mut entry_window: Option<(u64, usize, bool)> = None;
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
        f.units[0].attack = 60 * (100 + Gun::from_index(core.gun).modifiers().0 as i64) / 100;
        let before_air = core.airborne();
        let cmds = core.step(&f);
        samples.push(at.elapsed().as_nanos());
        if !before_air && core.airborne() {
            entry_window = Some((
                tick,
                core.plan.as_ref().and_then(|p| p.target).unwrap_or(2),
                false,
            ));
        }
        let (mut bought_gun, mut bought_armor) = (0i64, 0i64);
        if first_choice.is_none() && !core.continuation.is_empty() {
            first_choice = Some(core.continuation.clone());
        }
        for c in cmds {
            match c {
                Cmd::Gun { index } => {
                    bought_gun = Gun::from_index(index).price();
                    weapon_spent += bought_gun;
                }
                Cmd::Armor { amount } => {
                    bought_armor = if amount == f.units[0].max_hp * 25 / 100 {
                        1000
                    } else {
                        400
                    };
                    armor_spent += bought_armor;
                }
                Cmd::Buy { spent } => {
                    utility_spent += (spent - bought_gun - bought_armor).max(0);
                }
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
                        let resist = if raze_lab::raze_data::RULES_REVISION != "raze-3.3" {
                            u.defence
                        } else {
                            u.mr
                        };
                        let amount = reduced * 100 / (100 + resist.max(0));
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
        if guard {
            if core.airborne() {
                airborne_ticks += 1;
            } else {
                landing_guard_ticks += 1;
            }
        }
        if f.units[0].hp > 0 && core.cast.is_none() && tick >= basic {
            let g = Gun::from_index(core.gun);
            let pos = f.units[0].pos;
            if let Some(n) = f.units.iter().position(|u| {
                u.team != team && u.visible && u.damageable() && distance(pos, u.pos) <= g.reach()
            }) {
                let raw_basic =
                    raze_lab::valorant_guns::basic_raw(g, 60, distance(pos, f.units[n].pos));
                let amount = raw_basic * 100 / 120;
                let u = &mut f.units[n];
                let dealt = amount.min(u.hp);
                u.hp = (u.hp - amount).max(0);
                raw += raw_basic;
                actual += dealt;
                basic = tick + (6000 / (110 + g.modifiers().1) as u64).max(1);
            }
        }
        // Shared baseline/candidate ally model: approach and contribute basics.
        for i in 1..f.units.len() {
            if f.units[i].team != team || !f.units[i].champion() || !f.units[i].alive() {
                continue;
            }
            if let Some(n) = f
                .units
                .iter()
                .position(|u| u.team != team && u.champion() && u.visible && u.alive())
            {
                let from = f.units[i].pos;
                let target = f.units[n].pos;
                if distance(from, target) > f.units[i].attack_range {
                    let to = from.toward(target, 1020);
                    if !f.sweep(from, to, BODY_RADIUS).1 {
                        f.units[i].pos = to;
                    }
                } else if tick % 60 == 0 && !f.sweep(from, target, 1000).1 {
                    let dealt = (f.units[i].attack * 100 / 120).min(f.units[n].hp);
                    f.units[n].hp -= dealt;
                    if let Some((born, id, counted)) = entry_window.as_mut() {
                        if tick <= *born + 90 && *id == f.units[n].id {
                            followup_damage += dealt;
                            if !*counted {
                                entries_with_followup += 1;
                                *counted = true;
                            }
                        }
                    }
                }
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
        serde_json::json!({"seed":seed,"set":if seed<=24{"development"}else{"holdout"},"team":team,"role":role,"level":level,"firstContinuation":first_choice,"requestedRaw":raw,"labEstimatedDelivered":actual,"core":core.metrics,"exposureTicks":exposure,"missedWaveObservations":missed,"waveUnitsDefeated":waves,"objectiveEstimatedDamage":objectives,"deaths":deaths,"controlRequests":control,"airDamagePreventedEstimate":prevented,"finalHp":f.units[0].hp,"weaponSpent":weapon_spent,"armourSpent":armor_spent,"utilitySpent":utility_spent,"weaponAcquiredAt":core.last_weapon_buy,"finalWeapon":core.gun,"finalCredits":core.credits,"airborneTicks":airborne_ticks,"landingGuardTicks":landing_guard_ticks,"followupDamage":followup_damage,"entriesWithFollowup":entries_with_followup}),
        samples,
    )
}
fn economy() -> serde_json::Value {
    let mut core = Core::default();
    let mut f = Frame {
        units: vec![unit(1, 0, 300000, 300000, 0)],
        buy_allowed: true,
        at_spawn: true,
        bound: true,
        ..Default::default()
    };
    let (mut utility_spent, mut upgrade_tick) = (0i64, None);
    for tick in 1..=6500 {
        f.tick = tick;
        f.action = if tick % 120 == 1 {
            Some(Action::Buy)
        } else if matches!(tick % 720, 120 | 240) {
            Some(Action::Pack {
                aim: Point::new(290000, 300000),
            })
        } else if matches!(tick % 720, 180 | 300) {
            Some(Action::Detonate { id: 0 })
        } else {
            None
        };
        let mut weapon_cost = 0;
        for cmd in core.step(&f) {
            match cmd {
                Cmd::Gun { index } => {
                    let gun = Gun::from_index(index);
                    weapon_cost = gun.price();
                    if gun.tier() >= 2 && upgrade_tick.is_none() {
                        upgrade_tick = Some(tick);
                    }
                }
                Cmd::Buy { spent } => utility_spent += (spent - weapon_cost).max(0),
                Cmd::Spawn { id, hp, .. } => f.utility.push(UtilityStatus { id, hp }),
                Cmd::Remove { id } => f.utility.retain(|u| u.id != id),
                _ => {}
            }
        }
    }
    serde_json::json!({"ticks":6500,"finalGun":core.gun,"finalCredits":core.credits,"utilitySpent":utility_spent,"tierTwoUpgradeTick":upgrade_tick,"scope":"Identical scripted utility demand and buy visits; free charges, prices and ordinary passive income."})
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
        serde_json::json!({"runs":runs,"scenarios":cases,"economy":economy(),"scope":"Core-only timings. Damage uses a controlled armour/MR Lab model; this is not engine-attributed damage or native-host performance."})
    );
}
