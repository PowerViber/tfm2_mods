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
                .filter(|b| b.name().starts_with(prefix))
                .map(|b| b.name().to_owned())
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
/// Score bonus for the plan already running.
const STICKY_SCORE: i64 = 15;
/// `movement` hands back to the engine this close to the job point...
const GUIDE_STOP: i64 = 24_000;
/// ...and takes over again only beyond this.
const GUIDE_START: i64 = 40_000;
/// It also stops guiding once an enemy is this close, and starts again only beyond the second distance.
const GUIDE_ENEMY: (i64, i64) = (45_000, 55_000);


/// Whether `movement` keeps (or starts) guiding a champion, with separate start and stop distances.
fn guide(guiding: bool, to_point: i64, nearest_enemy: i64) -> bool {
    if guiding {
        to_point >= GUIDE_STOP && nearest_enemy >= GUIDE_ENEMY.0
    } else {
        to_point > GUIDE_START && nearest_enemy >= GUIDE_ENEMY.1
    }
}

pub fn tick(sim: &mut StableSim, heroes: &[Champ]) {
    let tick = sim.tick();
    // the epic monsters, found once a tick for both teams (each team used to scan every unit for them)
    let epics: Vec<usize> = (0..sim.entity_count())
        .filter_map(|n| sim.entity_at(n))
        .filter(|e| matches!(e.kind(), Some(Kind::Epic | Kind::Serpen)))
        .map(|e| e.id())
        .collect();
    for team in 0..2 {
        let members: Vec<_> = heroes
            .iter()
            .filter(|c| c.team == team && c.hp > 0)
            .collect();
        if members.is_empty() {
            continue;
        }
        // Own champions' low-HP flags and who is alive. Enemy visibility is left out: fog flicker in a fight
        // changed the signature every few ticks and re-planned (and reassigned jobs) every 10 ticks.
        let mut signature = heroes.iter().fold(0u64, |n, c| {
            let low = c.team == team && c.hp * 100 < c.max_hp * 30;
            n.wrapping_add((c.id as u64 + 1) * (if low { 3 } else { 1 }) * (if c.hp > 0 { 1 } else { 7 }))
        });
        for &id in &epics {
            if let Some(e) = sim.get_entity(id) {
                if sim.is_visible(team, e.id()) {
                    let (hp, max) = e.hp();
                    // quarter-HP steps: finer steps re-planned every 10 ticks through a whole objective fight
                    signature = signature.wrapping_add(
                        (e.id() as u64 + 1)
                            .wrapping_mul((hp * 4 / max.max(1)) as u64 + 1)
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
            // an ally worth reinforcing: hurt and pressed by several enemies (not every lane trade)
            let pressing = heroes
                .iter()
                .filter(|e| {
                    e.team != team
                        && sim.is_visible(team, e.id)
                        && d2(ally.x, ally.y, e.x, e.y) <= sq(core::REINFORCE_RANGE)
                })
                .count();
            if (ally.hp as i64) * 100 < (ally.max_hp as i64) * core::REINFORCE_HP
                && pressing >= core::REINFORCE_ENEMIES
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
                // a tower under real pressure: several enemies at it, or already badly hurt with one there
                Kind::Tower
                    if e.team() == team && {
                        let near = heroes
                            .iter()
                            .filter(|c| {
                                c.team != team
                                    && sim.is_visible(team, c.id)
                                    && d2(c.x, c.y, point.0, point.1) <= sq(core::DEFEND_RANGE)
                            })
                            .count();
                        near >= core::DEFEND_ENEMIES
                            || near > 0 && (hp as i64) * 100 < (max as i64) * core::DEFEND_HP
                    } =>
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
        let coach: [(&str, Option<String>); 4] = ["minion_wave", "object_buildup", "object_battle", "object_finish"]
            .map(|field| (field, coach_choice(sim, strategy_owner, team, field)));
        let wave = coach[0].1.as_deref() == Some("WavePriority");
        let gather = coach[1].1.as_deref() == Some("Gather");
        let finish = coach[3].1.as_deref() == Some("KillPriority");
        // The plan already running keeps a small edge, so near-equal plans don't swap every re-plan. It is kept
        // once on the nexus: a member's job can be a wave detour with another goal and target.
        let running = names(sim, strategy_owner, "ua_plan:").first().and_then(|n| {
            let v: Vec<usize> = n[8..].split(':').filter_map(|v| v.parse().ok()).collect();
            (v.len() == 2).then(|| (v[0] as u8, v[1]))
        });
        for plan in &mut plans {
            plan.score += crate::strategic_policy::priority(plan.goal, wave, gather, finish);
            if running == Some((plan.goal.code(), plan.target)) {
                plan.score += STICKY_SCORE;
            }
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
        let leading = plans.first().filter(|p| p.score > 0);
        if crate::settings::assist().team_strategy {
            write_strategy(sim, team, strategy_owner, leading.map(|p| (p.goal, p.score)), &coach, tick);
        }
        let Some(plan) = leading else {
            for n in names(sim, strategy_owner, "ua_plan:") {
                sim.entity_remove_buff(strategy_owner, &n);
            }
            continue;
        };
        replace(
            sim,
            strategy_owner,
            "ua_plan:",
            &format!("ua_plan:{}:{}", plan.goal.code(), plan.target),
            rules::ASSIGNMENT_LIFETIME_TICKS * 2,
        );
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
/// How long (ticks) a strategy field keeps a value the planner wrote before it may change it again.
const STRATEGY_HOLD: usize = 600;
/// A plan's goal class must lead this many re-plans in a row before the team's strategy fields follow it.
const STRATEGY_STREAK: u32 = 2;
/// Plans weaker than this never touch the team's strategy fields (the same bar `movement` guidance needs).
const STRATEGY_MIN_SCORE: i64 = 40;

/// Which strategy fields a goal sets: goals with the same table are one class.
fn class_of(goal: Goal) -> u8 {
    match goal {
        Goal::Wave | Goal::Camp | Goal::Trade | Goal::Push => 0,
        Goal::Prepare | Goal::Reinforce => 1,
        Goal::Start | Goal::Contest => 2,
        Goal::Finish => 3,
        Goal::Stall => 4,
        Goal::Defend => 5,
        Goal::Withdraw | Goal::Reset => 6,
    }
}

/// What each of the four strategy fields should hold: the leading goal's table, the coach's own choice for every
/// field the goal doesn't set, and the coach's choices everywhere when no plan is strong enough.
pub(crate) fn wanted_strategy(goal: Option<Goal>, coach: &[(&str, Option<String>); 4]) -> Vec<(String, String)> {
    let table = goal.map_or(&[][..], crate::strategic_policy::settings);
    coach
        .iter()
        .filter_map(|(field, own)| {
            let want = table.iter().find(|(f, _)| f == field).map(|(_, v)| v.to_string()).or_else(|| own.clone())?;
            Some((field.to_string(), want))
        })
        .collect()
}

/// Writes the team's strategy fields, slowly: only for a strong plan whose goal class has led two re-plans in a row,
/// each field held at least `STRATEGY_HOLD` ticks, and the coach's own choices restored when no plan qualifies.
/// A field the coach edits by hand during the match is left to the coach for the rest of it.
fn write_strategy(
    sim: &mut StableSim,
    team: usize,
    owner: usize,
    leading: Option<(Goal, i64)>,
    coach: &[(&str, Option<String>); 4],
    tick: usize,
) {
    let goal = leading.filter(|(_, score)| *score >= STRATEGY_MIN_SCORE).map(|(g, _)| g);
    let class = goal.map_or(255, class_of);
    let streak = names(sim, owner, "ua_class:")
        .first()
        .and_then(|n| {
            let v: Vec<u32> = n[9..].split(':').filter_map(|v| v.parse().ok()).collect();
            (v.len() == 2 && v[0] == class as u32).then(|| v[1] + 1)
        })
        .unwrap_or(1);
    replace(sim, owner, "ua_class:", &format!("ua_class:{class}:{streak}"), 5184000);
    if streak < STRATEGY_STREAK {
        return;
    }
    for (field, want) in wanted_strategy(goal, coach) {
        let Some(current) = sim
            .strategy_get_json(team, &field)
            .and_then(|s| serde_json::from_str::<String>(&s).ok())
        else {
            continue;
        };
        if !crate::strategic_policy::known(&field, &current) || !crate::strategic_policy::known(&field, &want) {
            continue;
        }
        let stop = format!("ua_coach_stop:{field}");
        if !names(sim, owner, &stop).is_empty() {
            continue;
        }
        let prefix = format!("ua_strategy:{field}:");
        if let Some(previous) = names(sim, owner, &prefix).first() {
            if previous.strip_prefix(&prefix) != Some(current.as_str()) {
                sim.add_buff(owner, &timed(&stop, 5184000));
                continue;
            }
        }
        if current == want {
            replace(sim, owner, &prefix, &format!("{prefix}{want}"), 5184000);
            continue;
        }
        let at = format!("ua_strategy_at:{field}:");
        let last = names(sim, owner, &at).first().and_then(|n| n[at.len()..].parse::<usize>().ok());
        if last.is_some_and(|t| tick < t + STRATEGY_HOLD) {
            continue;
        }
        if !sim.strategy_set_json(team, &field, &format!("\"{want}\"")) {
            continue;
        }
        replace(sim, owner, &prefix, &format!("{prefix}{want}"), 5184000);
        replace(sim, owner, &at, &format!("{at}{tick}"), 5184000);
    }
}

/// The last tick each player's engine ordered an attack (per simulation): a player who is fighting or farming
/// keeps the engine's moves.
static ATTACKS: std::sync::Mutex<Option<std::collections::HashMap<(u64, usize), usize>>> = std::sync::Mutex::new(None);

/// Called by the input AI for every engine attack order.
pub fn note_attack(key: u64, owner: usize, tick: usize) {
    if let Ok(mut g) = ATTACKS.lock() {
        let m = g.get_or_insert_with(std::collections::HashMap::new);
        if m.len() > 256 {
            m.retain(|k, _| k.0 == key);
        }
        m.insert((key, owner), tick);
    }
}

fn attacked_since(key: u64, owner: usize, since: usize) -> bool {
    ATTACKS
        .lock()
        .ok()
        .and_then(|g| g.as_ref()?.get(&(key, owner)).copied())
        .is_some_and(|t| t >= since)
}

/// Guidance only for plans at least this strong; Macro lowers the bar by at most 10 (a high-Macro athlete follows
/// a little more of the team's plans) and never below what an average athlete needed before round 141.
pub(crate) fn guide_gate(macro_rating: i64) -> i64 {
    130 - (macro_rating - 50).clamp(0, 50) / 5
}
/// The engine keeps a move whose destination is this close to something it is farming or attacking.
const FARM_NEAR: i64 = 30_000;
/// ...or when its player attacked within this many ticks.
const ATTACK_RECENT: usize = 30;
/// A guided step never lands this close to where the champion already stands (that would freeze it).
pub(crate) const MIN_STEP: i64 = 3_000;

/// One assignment as guidance remembers it: (goal, job, target, stored point).
pub(crate) type JobKey = (u8, u8, usize, (i64, i64));

/// What guidance knows about one champion this tick.
pub(crate) struct GuideIn {
    pub at: (i64, i64),
    /// where the engine is walking it
    pub dest: (i64, i64),
    pub job: JobKey,
    pub goal: Goal,
    /// the job point, read now (a moving target's current position)
    pub point: (i64, i64),
    pub score: i64,
    pub macro_rating: i64,
    pub micro_rating: i64,
    pub nearest_enemy: i64,
    /// the engine's destination is next to a hostile minion, camp or objective
    pub near_farm: bool,
    pub attacked_recently: bool,
}

/// Guidance memory for one champion: which assignment, whether it is guiding now, and whether it already handed
/// this assignment back (guidance takes over once per assignment, never back and forth).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct GuideState {
    pub job: Option<JobKey>,
    pub guiding: bool,
    pub done: bool,
}

/// The next step toward the job point, or None to leave the engine's move alone. Pure: the tests drive it.
pub(crate) fn guide_step(g: &GuideIn, st: &mut GuideState, wall: &dyn Fn(i64, i64) -> bool) -> Option<(i64, i64)> {
    if st.job != Some(g.job) {
        *st = GuideState { job: Some(g.job), guiding: false, done: false };
    }
    if st.done {
        return None;
    }
    let to_point = core::distance(g.at, g.point);
    let hand_back = |st: &mut GuideState| {
        if st.guiding {
            st.done = true;
        }
        st.guiding = false;
    };
    if g.score < guide_gate(g.macro_rating)
        || matches!(g.goal, Goal::Withdraw | Goal::Reset) && to_point < GUIDE_STOP
        || g.near_farm
        || g.attacked_recently
    {
        hand_back(st);
        return None;
    }
    if !guide(st.guiding, to_point, g.nearest_enemy) {
        hand_back(st);
        return None;
    }
    st.guiding = true;
    if core::distance(g.dest, g.point) < 18000 {
        return None;
    }
    let step = (24000 + g.micro_rating.clamp(0, 100) * 240).min(to_point);
    let toward = |to: (i64, i64)| {
        let d = core::distance(g.at, to).max(1);
        let s = step.min(d);
        (g.at.0 + (to.0 - g.at.0) * s / d, g.at.1 + (to.1 - g.at.1) * s / d)
    };
    let straight = toward(g.point);
    let clear = |a: (i64, i64), b: (i64, i64)| {
        // fine samples: a 3,000 spacing could cut the corner of a wall cell
        let n = (core::distance(a, b) / 1000).max(1);
        (1..=n).all(|i| !wall(a.0 + (b.0 - a.0) * i / n, a.1 + (b.1 - a.1) * i / n))
    };
    let out = if clear(g.at, straight) {
        straight
    } else {
        // around the wall along the terrain grid, one cell at a time
        let cell = crate::strategic_routes::next_step(g.at, g.point, &|p| wall(p.0, p.1))?;
        let next = toward(cell);
        if !clear(g.at, next) {
            return None;
        }
        next
    };
    (core::distance(g.at, out) >= MIN_STEP).then_some(out)
}

/// Whether `guide_step` will look at `near_farm` for this input: not when the assignment was already handed back,
/// and not when a cheaper reason (a weak plan, a close withdrawal, a recent attack) already hands it back.
pub(crate) fn reads_near_farm(g: &GuideIn, st: &GuideState) -> bool {
    let done = st.job == Some(g.job) && st.done;
    !done
        && g.score >= guide_gate(g.macro_rating)
        && !(matches!(g.goal, Goal::Withdraw | Goal::Reset) && core::distance(g.at, g.point) < GUIDE_STOP)
        && !g.attacked_recently
}

static GUIDES: std::sync::Mutex<Option<std::collections::HashMap<(u64, usize), GuideState>>> =
    std::sync::Mutex::new(None);

/// Movement-only advice. A danger retreat, cast, attack or teleport channel keeps its own authority, and so does
/// the engine whenever its player is farming, fighting or already heading to the job.
pub fn movement(sim: &StableSim, owner: usize, dest: (i64, i64)) -> Option<(i64, i64)> {
    if !crate::settings::assist().team_guidance {
        return None;
    }
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
                    | mod_api_stable::CcKindV1::Airborne
            )
        )
    }) {
        return None;
    }
    let (goal, job, target, stored, score) = assignment(sim, owner)?;
    let profile = profile(sim, owner);
    if profile.version < 2 {
        return None;
    }
    // Damage, Frontline and CatchWave jobs follow their target as it moves; a target that is gone ends the job.
    let point = if matches!(job, 0 | 1 | 5) {
        match sim.get_entity(target) {
            Some(t) if t.is_alive() => {
                let p = t.pos();
                (p.0 as i64, p.1 as i64)
            }
            Some(_) => return None,
            None => stored,
        }
    } else {
        stored
    };
    let nearest = (0..sim.champion_count())
        .filter_map(|n| sim.get_entity(sim.champion_id_at(n)))
        .filter(|e| e.team() != me.team() && e.is_alive() && sim.is_visible(me.team(), e.id()))
        .map(|e| e.pos())
        .map(|p| core::distance(at, (p.0 as i64, p.1 as i64)))
        .min()
        .unwrap_or(i64::MAX);
    let key = crate::match_key(sim);
    let tick = sim.tick();
    let mut input = GuideIn {
        at,
        dest,
        job: (goal.code(), job, target, stored),
        goal,
        point,
        score,
        macro_rating: profile.sense.0[0],
        micro_rating: profile.sense.0[1],
        nearest_enemy: nearest,
        near_farm: false,
        attacked_recently: attacked_since(key, owner, tick.saturating_sub(ATTACK_RECENT)),
    };
    let mut memory = GUIDES.lock().ok()?;
    let memory = memory.get_or_insert_with(std::collections::HashMap::new);
    if memory.len() > 256 {
        memory.retain(|k, _| k.0 == key);
    }
    let st = memory.entry((key, owner)).or_default();
    // round 141: the farm check scans every unit, so it runs only when guide_step would read it
    if reads_near_farm(&input, st) {
        input.near_farm = (0..sim.entity_count()).filter_map(|n| sim.entity_at(n)).any(|e| {
            e.is_alive()
                && e.team() != me.team()
                && matches!(e.kind(), Some(Kind::Minion | Kind::Jungle | Kind::Epic | Kind::Serpen))
                && {
                    let p = e.pos();
                    core::distance(dest, (p.0 as i64, p.1 as i64)) <= FARM_NEAR
                }
        });
    }
    guide_step(&input, st, &crate::walls::wall_at)
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

#[cfg(test)]
mod coach_tests {
    use super::*;
    use mod_api_stable::{BuffV1, EntityHandleV1 as E, SimCtxV1, SimVtableV1};
    use std::{cell::RefCell, ffi::c_void, mem::size_of};

    struct Host {
        buffs: RefCell<Vec<BuffV1>>,
        strategy: RefCell<String>,
    }
    unsafe fn host<'a>(p: *const c_void) -> &'a Host { &*(p as *const Host) }
    unsafe extern "C" fn valid(_: *const c_void, h: E) -> bool { h.id() == Some(1) }
    unsafe extern "C" fn count(p: *const c_void, _: E) -> usize { host(p).buffs.borrow().len() }
    unsafe extern "C" fn at(p: *const c_void, _: E, i: usize, out: *mut BuffV1) -> bool {
        host(p).buffs.borrow().get(i).map(|b| *out = *b).is_some()
    }
    unsafe extern "C" fn add(p: *mut c_void, _: usize, b: *const BuffV1) { host(p).buffs.borrow_mut().push(*b) }
    unsafe extern "C" fn get(p: *const c_void, _: usize, _: *const u8, _: usize, out: *mut u8, cap: usize, len: *mut usize) -> bool {
        let json = format!("\"{}\"", host(p).strategy.borrow());
        *len = json.len();
        if !out.is_null() {
            std::ptr::copy_nonoverlapping(json.as_ptr(), out, json.len().min(cap));
        }
        true
    }

    #[test]
    fn the_coach_choice_is_read_once_and_kept_when_the_planner_overwrites_the_field() {
        let h = Host { buffs: RefCell::new(vec![]), strategy: RefCell::new("WavePriority".into()) };
        unsafe {
            let mut table: SimVtableV1 = std::mem::zeroed();
            table.size = size_of::<SimVtableV1>();
            table.entity_is_valid = Some(valid);
            table.entity_buff_count = Some(count);
            table.entity_buff_at = Some(at);
            table.add_buff = Some(add);
            table.strategy_get_json = Some(get);
            let mut raw = SimCtxV1 { size: size_of::<SimCtxV1>(), sim: &table, frame: std::ptr::null(), state: &h as *const Host as *mut c_void };
            let mut sim = StableSim::from_raw(&mut raw).unwrap();
            assert_eq!(coach_choice(&mut sim, 1, 0, "minion_wave").as_deref(), Some("WavePriority"));
            // the planner writes its own choice; the coach's original still scores
            *h.strategy.borrow_mut() = "Balanced".into();
            assert_eq!(coach_choice(&mut sim, 1, 0, "minion_wave").as_deref(), Some("WavePriority"));
            assert_eq!(h.buffs.borrow().len(), 1, "recorded once");
        }
    }
}

#[cfg(test)]
mod guide_tests {
    use super::*;

    #[test]
    fn guiding_has_separate_start_and_stop_distances() {
        // not guiding: needs more than 40,000 to the point and no enemy within 55,000
        assert!(!guide(false, 30_000, 100_000));
        assert!(guide(false, 50_000, 100_000));
        assert!(!guide(false, 50_000, 50_000));
        // guiding: keeps going down to 24,000 and with an enemy down to 45,000
        assert!(guide(true, 30_000, 50_000));
        assert!(!guide(true, 20_000, 100_000));
        assert!(!guide(true, 50_000, 40_000));
    }
}
