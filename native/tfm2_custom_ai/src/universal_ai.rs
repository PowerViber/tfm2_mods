//! Shared map intentions are stored in simulation buffs: clones/owners/replays never share mutable worlds.
use crate::game_sense_core::rules;
use crate::strategic_core::{self as core, Actor, Goal, Target};
use crate::{d2, sq, timed, Champ};
use mod_api_stable::{EntityKindV1 as Kind, SkillSlotV1, StableSim};
pub fn profile(sim: &StableSim, owner: usize) -> crate::athlete_ai_core::Profile {
    crate::athlete_ai::pinned_owner(sim, owner, &mut None)
}
fn names(sim: &StableSim, id: usize, prefix: &str) -> Vec<String> {
    sim.get_entity(id)
        .map(|e| {
            (0..e.buff_count())
                .filter_map(|i| e.buff_at(i))
                .map(|b| b.name().to_owned())
                .filter(|s| s.starts_with(prefix))
                .collect()
        })
        .unwrap_or_default()
}
fn replace(sim: &mut StableSim, id: usize, prefix: &str, name: &str, life: usize) {
    for n in names(sim, id, prefix) {
        sim.entity_remove_buff(id, &n);
    }
    sim.add_buff(id, &timed(name, life));
}
pub fn assignment(sim: &StableSim, owner: usize) -> Option<(Goal, u8, usize, (i64, i64), i64)> {
    let name = names(sim, owner, "ua_job:").into_iter().next()?;
    let p: Vec<i64> = name
        .strip_prefix("ua_job:")?
        .split(':')
        .map(str::parse)
        .collect::<Result<_, _>>()
        .ok()?;
    if p.len() != 6 {
        return None;
    }
    Some((
        Goal::from_code(p[0] as u8)?,
        p[1] as u8,
        p[2] as usize,
        (p[3], p[4]),
        p[5],
    ))
}
pub fn objective_assignment(sim: &StableSim, owner: usize, target: usize) -> bool {
    assignment(sim, owner).is_some_and(|(g, j, id, _, score)| {
        id == target
            && score > 0
            && j <= 1
            && matches!(g, Goal::Start | Goal::Finish | Goal::Contest)
    })
}
fn blocked(a: (i64, i64), b: (i64, i64)) -> bool {
    (1..=24).any(|n| crate::walls::wall_at(a.0 + (b.0 - a.0) * n / 24, a.1 + (b.1 - a.1) * n / 24))
}
pub fn tick(sim: &mut StableSim, heroes: &[Champ]) {
    let tick = sim.tick();
    for team in 0..2 {
        let members: Vec<_> = heroes
            .iter()
            .filter(|c| c.team == team && c.hp > 0)
            .collect();
        if members.is_empty() {
            continue;
        }
        let mut signature = heroes
            .iter()
            .filter(|c| c.team == team || sim.is_visible(team, c.id))
            .fold(0u64, |n, c| {
                n.wrapping_add((c.id as u64 + 1) * (if c.hp * 100 < c.max_hp * 30 { 3 } else { 1 }))
            });
        for n in 0..sim.entity_count() {
            if let Some(e) = sim.entity_at(n) {
                if matches!(e.kind(), Some(Kind::Epic | Kind::Serpen))
                    && sim.is_visible(team, e.id())
                {
                    let (hp, max) = e.hp();
                    signature = signature.wrapping_add(
                        (e.id() as u64 + 1)
                            .wrapping_mul((hp * 16 / max.max(1)) as u64 + 1)
                            .wrapping_mul(0x9e3779b97f4a7c15),
                    );
                }
            }
        }
        let previous = members
            .iter()
            .flat_map(|c| names(sim, c.id, "ua_clock:"))
            .next();
        if let Some(n) = previous {
            let v: Vec<u64> = n[9..].split(':').filter_map(|v| v.parse().ok()).collect();
            if v.len() == 2
                && tick < v[0] as usize + rules::INTERVAL_TICKS
                && (v[1] == signature || tick < v[0] as usize + rules::URGENT_INTERVAL_TICKS)
            {
                continue;
            }
        }
        let mut actors = vec![];
        let started = crate::perf::enabled().then(std::time::Instant::now);
        for c in heroes
            .iter()
            .filter(|c| c.team == team || sim.is_visible(team, c.id))
        {
            let Some(e) = sim.get_entity(c.id) else {
                continue;
            };
            let stat = e.stat();
            let player = (0..sim.player_count())
                .filter_map(|n| sim.player_at(n))
                .find(|p| p.champion().is_some_and(|e| e.id() == c.id));
            let p = profile(sim, c.id);
            actors.push(Actor {
                id: c.id,
                athlete: crate::athlete_ai::owner_athlete(sim, c.id).unwrap_or(c.id),
                team: c.team,
                point: (c.x, c.y),
                hp: c.hp as i64,
                max_hp: c.max_hp as i64,
                speed: stat.move_speed as i64,
                power: (stat.attack + stat.magic_power) as i64,
                range: crate::farming_data::attack_range(&c.name),
                role: player.and_then(|p| p.lane()).map_or(0, |l| l.code() as u8),
                controlled: c.stunned || c.pushed,
                ultimate: if c.team == team {
                    e.cooldown(SkillSlotV1::Ult)
                        .is_some_and(|v| v.unlocked && v.remaining == 0)
                } else {
                    true
                },
                visible: true,
                ratings: p.sense,
                judgement: p.judgement(),
                ego: p.ego(),
            });
        }
        let Some(caller) = actors
            .iter()
            .filter(|a| a.team == team)
            .max_by_key(|a| (a.ratings.0[3], a.judgement, std::cmp::Reverse(a.athlete)))
            .map(|a| a.id)
        else {
            continue;
        };
        let mut targets = vec![];
        let (home_id, home) = (0..sim.entity_count())
            .filter_map(|n| sim.entity_at(n))
            .find(|e| e.team() == team && e.kind() == Some(Kind::Nexus))
            .map(|e| {
                let p = e.pos();
                (e.id(), (p.0 as i64, p.1 as i64))
            })
            .unwrap_or((
                caller,
                if team == 0 {
                    (96000, 864000)
                } else {
                    (864000, 96000)
                },
            ));
        targets.push(Target {
            id: home_id,
            kind: 7,
            team,
            point: home,
            hp: 1,
            max_hp: 1,
            loss: 0,
            visible: true,
            value: 0,
        });
        for ally in &members {
            if (ally.hp * 100 < ally.max_hp * 85 || ally.stunned || ally.pushed)
                && heroes.iter().any(|e| {
                    e.team != team
                        && sim.is_visible(team, e.id)
                        && d2(ally.x, ally.y, e.x, e.y) <= sq(70000)
                })
            {
                targets.push(Target {
                    id: ally.id,
                    kind: 6,
                    team,
                    point: (ally.x, ally.y),
                    hp: ally.hp as i64,
                    max_hp: ally.max_hp as i64,
                    loss: 0,
                    visible: true,
                    value: 20,
                });
            }
        }
        for n in 0..sim.entity_count() {
            let Some(e) = sim.entity_at(n) else { continue };
            if !e.is_alive() || !e.is_targetable() || !sim.is_visible(team, e.id()) {
                continue;
            }
            let Some(kind) = e.kind() else { continue };
            let p = e.pos();
            let point = (p.0 as i64, p.1 as i64);
            let (hp, max) = e.hp();
            let ty = match kind {
                Kind::Minion if e.team() != team => 0,
                Kind::Jungle => 1,
                Kind::Epic => 2,
                Kind::Serpen => 3,
                Kind::Tower
                    if e.team() == team
                        && heroes.iter().any(|c| {
                            c.team != team
                                && sim.is_visible(team, c.id)
                                && d2(c.x, c.y, point.0, point.1) <= sq(110000)
                        }) =>
                {
                    4
                }
                Kind::Tower if e.team() != team => 5,
                _ => continue,
            };
            if ty == 0
                && targets
                    .iter()
                    .any(|t: &Target| t.kind == 0 && core::distance(t.point, point) < 45000)
            {
                continue;
            }
            let mut loss = 0;
            if ty == 2 || ty == 3 {
                let prefix = format!("ua_hp:{}:", e.id());
                if let Some(old) = names(sim, caller, &prefix).first() {
                    let v: Vec<i64> = old[prefix.len()..]
                        .split(':')
                        .filter_map(|n| n.parse().ok())
                        .collect();
                    if v.len() == 2 && tick as i64 > v[1] {
                        loss = (v[0] - hp as i64).max(0) * 60 / (tick as i64 - v[1]);
                    }
                }
            }
            targets.push(Target {
                id: e.id(),
                kind: ty,
                team: e.team(),
                point,
                hp: hp as i64,
                max_hp: max as i64,
                loss,
                visible: true,
                value: if ty == 2 {
                    35
                } else if ty == 3 {
                    25
                } else {
                    10
                },
            });
        }
        targets.sort_by_key(|t| (std::cmp::Reverse(t.kind >= 2), t.id));
        targets.truncate(rules::TARGET_LIMIT);
        for t in targets.iter().filter(|t| t.kind == 2 || t.kind == 3) {
            let prefix = format!("ua_hp:{}:", t.id);
            replace(
                sim,
                caller,
                &prefix,
                &format!("{prefix}{}:{tick}", t.hp),
                600,
            );
        }
        let mut unseen = 0;
        for c in heroes.iter().filter(|c| c.team != team) {
            let prefix = format!("ua_seen:{}:", c.id);
            if sim.is_visible(team, c.id) {
                replace(
                    sim,
                    caller,
                    &prefix,
                    &format!("{prefix}{}:{}:{tick}", c.x, c.y),
                    600,
                );
            } else {
                let old = names(sim, caller, &prefix);
                let age = old
                    .first()
                    .and_then(|s| s.rsplit(':').next())
                    .and_then(|s| s.parse::<usize>().ok())
                    .map_or(600, |t| tick.saturating_sub(t));
                unseen += (5 + age as i64 / 60).min(20);
            }
        }
        let recent = (0..sim.kill_log_count())
            .rev()
            .filter_map(|n| sim.kill_log_at(n))
            .take_while(|k| k.tick + 900 >= tick)
            .map(|k| if k.killer_team == team { 1 } else { -1 })
            .sum();
        let routes =
            crate::strategic_routes::build(&actors, &targets, &|p| crate::walls::wall_at(p.0, p.1));
        let snapshot_ns = started.as_ref().map_or(0, |s| s.elapsed().as_nanos());
        let mut plans = core::evaluate(&core::View {
            routes: Some(&routes),
            team,
            actors: &actors,
            targets: &targets,
            home,
            recent_advantage: recent,
            unseen_risk: unseen,
            blocked: &blocked,
        });
        let strategy_owner = (0..sim.entity_count())
            .filter_map(|n| sim.entity_at(n))
            .find(|e| e.team() == team && e.kind() == Some(Kind::Nexus))
            .map_or(caller, |e| e.id());
        // Score against the coach's own choices, recorded once per match. Reading the live fields
        // instead would reward whatever this planner wrote last and lock the first plan in.
        let mut value = |field: &str| coach_choice(sim, strategy_owner, team, field);
        let wave = value("minion_wave").as_deref() == Some("WavePriority");
        let gather = value("object_buildup").as_deref() == Some("Gather");
        let finish = value("object_finish").as_deref() == Some("KillPriority");
        for plan in &mut plans {
            plan.score += crate::strategic_policy::priority(plan.goal, wave, gather, finish);
        }
        plans.sort_by_key(|p| (std::cmp::Reverse(p.score), p.goal.code(), p.target));
        let planning_ns = started
            .as_ref()
            .map_or(0, |s| s.elapsed().as_nanos().saturating_sub(snapshot_ns));
        replace(
            sim,
            caller,
            "ua_clock:",
            &format!("ua_clock:{tick}:{signature}"),
            121,
        );
        for member in &members {
            for name in names(sim, member.id, "ua_job:") {
                sim.entity_remove_buff(member.id, &name);
            }
        }
        let Some(plan) = plans.first().filter(|p| p.score > 0) else {
            continue;
        };
        for (field, want) in crate::strategic_policy::settings(plan.goal) {
            let Some(current) = sim
                .strategy_get_json(team, field)
                .and_then(|s| serde_json::from_str::<String>(&s).ok())
            else {
                continue;
            };
            if !crate::strategic_policy::known(field, &current) {
                continue;
            }
            let stop = format!("ua_coach_stop:{field}");
            if !names(sim, strategy_owner, &stop).is_empty() {
                continue;
            }
            let prefix = format!("ua_strategy:{field}:");
            if let Some(previous) = names(sim, strategy_owner, &prefix).first() {
                if previous.strip_prefix(&prefix) != Some(current.as_str()) {
                    sim.add_buff(strategy_owner, &timed(&stop, 5184000));
                    continue;
                }
            }
            if current != *want {
                if !sim.strategy_set_json(team, field, &format!("\"{want}\"")) {
                    continue;
                }
            }
            replace(
                sim,
                strategy_owner,
                &prefix,
                &format!("{prefix}{want}"),
                5184000,
            );
        }
        crate::farming::decision(sim, team, plan);
        crate::athlete_ai::participate(sim, caller, 8);
        for job in &plan.jobs {
            replace(
                sim,
                job.actor,
                "ua_job:",
                &format!(
                    "ua_job:{}:{}:{}:{}:{}:{}",
                    job.goal.code(),
                    job.job as u8,
                    job.target,
                    job.point.0,
                    job.point.1,
                    plan.score
                ),
                rules::ASSIGNMENT_LIFETIME_TICKS,
            );
            crate::athlete_ai::participate(sim, job.actor, 1 | if unseen > 0 { 4 } else { 0 });
        }
        if let Some(start) = started {
            crate::farming::planning_timing(
                sim,
                caller,
                team,
                snapshot_ns,
                planning_ns,
                start
                    .elapsed()
                    .as_nanos()
                    .saturating_sub(snapshot_ns + planning_ns),
            );
        }
    }
}
/// Movement-only advice. A danger retreat, cast, attack or teleport channel keeps its own authority.
pub fn movement(sim: &StableSim, owner: usize, dest: (i64, i64)) -> Option<(i64, i64)> {
    let me = sim.get_entity(owner)?;
    let hp = me.hp();
    if !me.is_alive() || hp.0 * 100 < hp.1 * 35 {
        return None;
    }
    let at = me.pos();
    let at = (at.0 as i64, at.1 as i64);
    if (0..me.cc_count()).filter_map(|n| me.cc_at(n)).any(|c| {
        matches!(
            mod_api_stable::CcKindV1::from_code(c.kind),
            Some(
                mod_api_stable::CcKindV1::Stun
                    | mod_api_stable::CcKindV1::Bind
                    | mod_api_stable::CcKindV1::ForceMove
            )
        )
    }) {
        return None;
    }
    let (goal, _, _, p, score) = assignment(sim, owner)?;
    if score < 40 || matches!(goal, Goal::Withdraw | Goal::Reset) && core::distance(at, p) < 24000 {
        return None;
    }
    let profile = profile(sim, owner);
    if profile.version < 2 {
        return None;
    }
    let nearest = (0..sim.champion_count())
        .filter_map(|n| sim.get_entity(sim.champion_id_at(n)))
        .filter(|e| e.team() != me.team() && e.is_alive() && sim.is_visible(me.team(), e.id()))
        .map(|e| e.pos())
        .map(|p| core::distance(at, (p.0 as i64, p.1 as i64)))
        .min()
        .unwrap_or(i64::MAX);
    // Keep urgent retreats and immediate attack positioning intact.
    if nearest < 55000 || core::distance(at, p) < 24000 {
        return None;
    }
    let min = 30 + (100 - profile.sense.0[0]) * 2;
    if score < min || blocked(at, p) || core::distance(dest, p) < 18000 {
        return None;
    }
    let d = core::distance(at, p).max(1);
    let step = (24000 + profile.sense.0[1] * 240).min(d);
    Some((
        at.0 + (p.0 - at.0) * step / d,
        at.1 + (p.1 - at.1) * step / d,
    ))
}

/// The coach's original value for a strategy field, recorded on the nexus the first time it is read.
fn coach_choice(sim: &mut StableSim, owner: usize, team: usize, field: &str) -> Option<String> {
    let prefix = format!("ua_coach0:{field}:");
    if let Some(name) = names(sim, owner, &prefix).first() {
        return name.strip_prefix(&prefix).map(str::to_owned);
    }
    let current = sim
        .strategy_get_json(team, field)
        .and_then(|v| serde_json::from_str::<String>(&v).ok())?;
    sim.add_buff(owner, &timed(&format!("{prefix}{current}"), 5184000));
    Some(current)
}
