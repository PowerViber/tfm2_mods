//! Bounded map decisions. Receives team-visible observations; emits intentions, never engine casts.
use crate::game_sense_core::{rules, Ratings};
pub type Point = (i64, i64);
pub fn distance(a: Point, b: Point) -> i64 {
    (((a.0 - b.0) as i128).pow(2) + ((a.1 - b.1) as i128).pow(2)).isqrt() as i64
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Goal {
    Wave,
    Camp,
    Defend,
    Push,
    Prepare,
    Start,
    Finish,
    Contest,
    Stall,
    Withdraw,
    Trade,
    Reinforce,
    Reset,
}
impl Goal {
    pub fn code(self) -> u8 {
        self as u8
    }
    pub fn from_code(n: u8) -> Option<Self> {
        Some(match n {
            0 => Self::Wave,
            1 => Self::Camp,
            2 => Self::Defend,
            3 => Self::Push,
            4 => Self::Prepare,
            5 => Self::Start,
            6 => Self::Finish,
            7 => Self::Contest,
            8 => Self::Stall,
            9 => Self::Withdraw,
            10 => Self::Trade,
            11 => Self::Reinforce,
            12 => Self::Reset,
            _ => return None,
        })
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Job {
    Damage,
    Frontline,
    Zone,
    Protect,
    Flank,
    CatchWave,
    Retreat,
}
#[derive(Clone, Debug)]
pub struct Actor {
    pub id: usize,
    pub athlete: usize,
    pub team: usize,
    pub point: Point,
    pub hp: i64,
    pub max_hp: i64,
    pub speed: i64,
    pub power: i64,
    pub range: i64,
    pub role: u8,
    pub controlled: bool,
    pub ultimate: bool,
    pub visible: bool,
    pub ratings: Ratings,
    pub judgement: i64,
    pub ego: i64,
}
#[derive(Clone, Debug)]
pub struct Target {
    pub id: usize,
    pub kind: u8,
    pub team: usize,
    pub point: Point,
    pub hp: i64,
    pub max_hp: i64,
    pub loss: i64,
    pub visible: bool,
    pub value: i64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Assignment {
    pub actor: usize,
    pub job: Job,
    pub point: Point,
    pub target: usize,
    pub goal: Goal,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Plan {
    pub caller: usize,
    pub goal: Goal,
    pub target: usize,
    pub point: Point,
    pub score: i64,
    pub reason: &'static str,
    pub jobs: Vec<Assignment>,
}
pub struct View<'a> {
    pub routes: Option<&'a crate::strategic_routes::Routes>,
    pub team: usize,
    pub actors: &'a [Actor],
    pub targets: &'a [Target],
    pub home: Point,
    pub recent_advantage: i64,
    pub unseen_risk: i64,
    pub blocked: &'a dyn Fn(Point, Point) -> bool,
}
fn eta(v: &View, u: &Actor, target: usize, p: Point) -> i64 {
    let d = if let Some(route) = v.routes.and_then(|r| r.get(&(u.id, target))) {
        let Some(d) = route else {
            return 100000;
        };
        *d
    } else {
        distance(u.point, p) * (1 + i64::from((v.blocked)(u.point, p)) * 2)
    };
    (d - 45000).max(0) / u.speed.max(1)
}
fn caller<'a>(v: &View<'a>) -> Option<&'a Actor> {
    v.actors
        .iter()
        .filter(|a| a.team == v.team && a.hp > 0)
        .max_by_key(|a| (a.ratings.0[3], a.judgement, std::cmp::Reverse(a.athlete)))
}
fn assignments(v: &View, goal: Goal, t: &Target) -> Vec<Assignment> {
    let p = t.point;
    let mut members: Vec<_> = v
        .actors
        .iter()
        .filter(|a| a.team == v.team && a.hp > 0)
        .collect();
    members.sort_by_key(|a| (eta(v, a, t.id, p), a.id));
    let damage = members
        .iter()
        .filter(|a| a.hp * 100 >= a.max_hp * 35 && !a.controlled)
        .max_by_key(|a| (a.power, std::cmp::Reverse(a.id)))
        .map(|a| a.id);
    let collector = members
        .iter()
        .filter(|a| goal != Goal::Camp || a.role == 1)
        .min_by_key(|a| (eta(v, a, t.id, p), a.id))
        .map(|a| a.id);
    let mut front = false;
    let mut wave = false;
    members
        .into_iter()
        .filter(|a| !matches!(goal, Goal::Wave | Goal::Camp) || Some(a.id) == collector)
        .map(|a| {
            if matches!(goal, Goal::Prepare | Goal::Start | Goal::Contest)
                && Some(a.id) != damage
                && a.role != 1
                && !wave
                && eta(v, a, t.id, p) > 240
            {
                if let Some(w) = v
                    .targets
                    .iter()
                    .filter(|w| {
                        w.kind == 0
                            && w.visible
                            && distance(a.point, w.point) < 120000
                            && distance(p, w.point) > 180000
                    })
                    .min_by_key(|w| (distance(a.point, w.point), w.id))
                {
                    wave = true;
                    return Assignment {
                        actor: a.id,
                        job: Job::CatchWave,
                        point: w.point,
                        target: w.id,
                        goal: Goal::Wave,
                    };
                }
            }
            let job = if a.hp * 100 < a.max_hp * 25 {
                Job::Retreat
            } else if matches!(goal, Goal::Withdraw | Goal::Reset) {
                Job::Retreat
            } else if matches!(
                goal,
                Goal::Wave | Goal::Camp | Goal::Defend | Goal::Trade | Goal::Push
            ) && !wave
            {
                wave = true;
                Job::CatchWave
            } else if Some(a.id) == damage {
                Job::Damage
            } else if a.role == 4 {
                Job::Protect
            } else if a.range < 35000 && !front {
                front = true;
                Job::Frontline
            } else if a.role == 1 {
                Job::Flank
            } else {
                Job::Zone
            };
            let point = match job {
                Job::Retreat => v.home,
                Job::Zone | Job::Flank => (
                    p.0 + (if v.team == 0 { 1 } else { -1 }) * (30000 + a.ratings.0[1] * 200),
                    p.1,
                ),
                Job::Protect => (p.0 - (if v.team == 0 { 1 } else { -1 }) * 40000, p.1),
                _ => p,
            };
            Assignment {
                actor: a.id,
                job,
                point: (point.0.clamp(1000, 959000), point.1.clamp(1000, 959000)),
                target: t.id,
                goal,
            }
        })
        .collect()
}
pub fn evaluate(v: &View) -> Vec<Plan> {
    let Some(c) = caller(v) else { return vec![] };
    let mut candidates = vec![];
    for t in v
        .targets
        .iter()
        .filter(|t| t.visible && t.hp > 0)
        .take(rules::TARGET_LIMIT)
    {
        let ours: Vec<_> = v
            .actors
            .iter()
            .filter(|a| a.team == v.team && a.hp > 0)
            .collect();
        let ready = ours
            .iter()
            .filter(|a| {
                !a.controlled && a.hp * 100 >= a.max_hp * 35 && eta(v, a, t.id, t.point) <= 180
            })
            .count() as i64;
        let incoming = ours
            .iter()
            .filter(|a| {
                !a.controlled && a.hp * 100 >= a.max_hp * 35 && eta(v, a, t.id, t.point) <= 480
            })
            .count() as i64;
        let enemies: Vec<_> = v
            .actors
            .iter()
            .filter(|a| {
                a.team != v.team && a.visible && a.hp > 0 && distance(a.point, t.point) < 180000
            })
            .collect();
        let enemy = enemies.len() as i64;
        let ult_edge = ours
            .iter()
            .filter(|a| a.ultimate && eta(v, a, t.id, t.point) <= 240)
            .count() as i64
            - enemies.iter().filter(|a| a.ultimate).count() as i64;
        let travel = ours
            .iter()
            .map(|a| eta(v, a, t.id, t.point))
            .min()
            .unwrap_or(1200);
        let advantage = ready - enemy;
        let uncertain = v.unseen_risk * (50 + c.ratings.0[2]) / 100;
        let caution = (160 - c.ego) * (50 + c.judgement) / 100;
        let (goal, base, reason) = match t.kind {
            6 if t.team==v.team&&enemy>0 => (
                Goal::Reinforce,
                80+enemy*12+incoming*5-travel/10-uncertain-(enemy-ready).max(0)*caution/25,
                "Reinforce a visible threatened teammate; compare the wave and arrival risk",
            ),
            7 if t.team==v.team&&ours.iter().filter(|a|a.hp*100<a.max_hp*40).count()*2>=ours.len() => (
                Goal::Reset,
                105+ours.iter().filter(|a|a.hp*100<a.max_hp*25).count()as i64*10,
                "Several teammates need a reset; stage at base while engine recalls remain authoritative",
            ),
            0 => (
                Goal::Wave,
                45 + t.value - travel / 12 - enemy * 25,
                "Collect the wave before rotating",
            ),
            1 if ours
                .iter()
                .any(|a| a.role == 1 && eta(v, a, t.id, t.point) <= 360) =>
            {
                (
                    Goal::Camp,
                    35 + t.value - travel / 12 - enemy * 25,
                    "Assigned jungler can clear this camp",
                )
            }
            2 | 3 => {
                let finish_ticks = if t.loss > 0 {
                    t.hp * 60 / t.loss.max(1)
                } else {
                    i64::MAX
                };
                if ready == 0 && finish_ticks < travel {
                    (
                        Goal::Withdraw,
                        if travel < 300 { 75 } else { -80 },
                        "Arrival is too late; compare safe cross-map alternatives",
                    )
                } else if advantage < 0
                    && incoming > ready
                    && (incoming >= enemy || ult_edge > 0)
                    && finish_ticks > 120
                {
                    (
                        Goal::Stall,
                        65 + incoming * 8 + ult_edge * 8 - uncertain,
                        "Delay safely until reinforcements arrive",
                    )
                } else if advantage < 0 && (incoming < enemy || finish_ticks < 60) {
                    (
                        Goal::Withdraw,
                        80 + enemy * 5 + uncertain,
                        "Contest lacks time or surviving follow-up",
                    )
                } else if (finish_ticks <= 120 || t.hp * 100 < t.max_hp * 8 && t.loss > 0)
                    && ready > 0
                {
                    (
                        Goal::Finish,
                        130 + advantage * 15 - uncertain,
                        "Finish window exists; reserve a suitable burst",
                    )
                } else if enemy > 0 {
                    (
                        Goal::Contest,
                        80 + advantage * 18 + ult_edge * 10 - uncertain - enemy * caution / 30,
                        "Split damage, zoning and carry protection",
                    )
                } else if ready >= 2
                    || ours.iter().any(|a| {
                        a.role == 1 && a.hp * 100 >= a.max_hp * 60 && eta(v, a, t.id, t.point) <= 90
                    })
                {
                    (
                        Goal::Start,
                        90 + t.value + v.recent_advantage * 8 - travel / 12 - uncertain,
                        "Ready participants can begin the objective",
                    )
                } else {
                    (
                        Goal::Prepare,
                        40 + t.value - travel / 12 - uncertain,
                        "Prepare a route and nearby waves first",
                    )
                }
            }
            4 if t.team == v.team => (
                Goal::Defend,
                75 + enemy * 20 - travel / 10,
                "Protect the threatened tower and its wave",
            ),
            5 if t.team != v.team => (
                if enemy == 0 && v.recent_advantage < 0 {
                    Goal::Trade
                } else {
                    Goal::Push
                },
                50 + t.value + v.recent_advantage * 10 - enemy * 25 - travel / 12,
                "Convert pressure into a tower or cross-map trade",
            ),
            _ => continue,
        };
        // Macro evaluates travel/opportunity cost; Strategy compares alternatives. This hash consumes no RNG.
        let salt = (t.id as u64).wrapping_mul(0x9e3779b97f4a7c15);
        let error = ((salt.rotate_left(17) % 31) as i64 - 15) * (100 - c.ratings.0[3]) / 100;
        let score = base + error - (travel / 30) * (100 - c.ratings.0[0]) / 100;
        candidates.push(Plan {
            caller: c.id,
            goal,
            target: t.id,
            point: t.point,
            score,
            reason,
            jobs: assignments(v, goal, t),
        });
    }
    candidates.sort_by_key(|p| (std::cmp::Reverse(p.score), p.goal.code(), p.target));
    candidates.truncate(rules::PLAN_LIMIT);
    candidates
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn visible_threats_enable_reinforcement_and_low_teams_reset() {
        let mut a = vec![
            actor(1, 0, 480000),
            actor(2, 0, 520000),
            actor(3, 1, 490000),
        ];
        let mut threatened = target(50, 6);
        threatened.team = 0;
        let evaluate_one = |a: &[Actor], t: &[Target]| {
            evaluate(&View {
                routes: None,
                team: 0,
                actors: a,
                targets: t,
                home: (96000, 864000),
                recent_advantage: 0,
                unseen_risk: 0,
                blocked: &|_, _| false,
            })
        };
        assert_eq!(evaluate_one(&a, &[threatened])[0].goal, Goal::Reinforce);
        a[0].hp = 200;
        a[1].hp = 200;
        let mut home = target(60, 7);
        home.team = 0;
        home.point = (96000, 864000);
        let p = evaluate_one(&a, &[home]);
        assert_eq!(p[0].goal, Goal::Reset);
        assert!(p[0].jobs.iter().all(|j| j.job == Job::Retreat));
    }
    fn actor(id: usize, team: usize, x: i64) -> Actor {
        Actor {
            id,
            athlete: id,
            team,
            point: (x, 480000),
            hp: 1000,
            max_hp: 1000,
            speed: 1000,
            power: 100,
            range: 40000,
            role: if id == 1 { 1 } else { 0 },
            controlled: false,
            ultimate: false,
            visible: true,
            ratings: Ratings([90; 4]),
            judgement: 90,
            ego: 50,
        }
    }
    fn target(id: usize, kind: u8) -> Target {
        Target {
            id,
            kind,
            team: 2,
            point: (480000, 480000),
            hp: 10000,
            max_hp: 20000,
            loss: 0,
            visible: true,
            value: 20,
        }
    }
    #[test]
    fn prepares_then_starts_and_finishes_with_real_arrivals() {
        let targets = [target(20, 2)];
        let mut actors = vec![actor(1, 0, 700000), actor(2, 0, 720000)];
        let view = |actors: &[Actor], ts: &[Target]| {
            evaluate(&View {
                routes: None,
                team: 0,
                actors,
                targets: ts,
                home: (96000, 864000),
                recent_advantage: 0,
                unseen_risk: 0,
                blocked: &|_, _| false,
            })
        };
        assert_eq!(view(&actors, &targets)[0].goal, Goal::Prepare);
        actors[0].point = (480000, 480000);
        actors[1].point = (500000, 480000);
        assert_eq!(view(&actors, &targets)[0].goal, Goal::Start);
        let mut ts = targets.clone();
        ts[0].hp = 300;
        ts[0].loss = 500;
        assert_eq!(view(&actors, &ts)[0].goal, Goal::Finish);
    }
    #[test]
    fn concealed_observations_and_ordinary_camps_do_not_override_roles() {
        let mut a = actor(1, 0, 480000);
        a.role = 0;
        let mut t = target(2, 2);
        t.visible = false;
        let ts = [t, target(3, 1)];
        assert!(evaluate(&View {
            routes: None,
            team: 0,
            actors: &[a],
            targets: &ts,
            home: (0, 0),
            recent_advantage: 0,
            unseen_risk: 0,
            blocked: &|_, _| false
        })
        .is_empty());
    }
    #[test]
    fn caller_ties_and_jobs_are_deterministic() {
        let a = [
            actor(3, 0, 480000),
            actor(1, 0, 480000),
            actor(2, 0, 480000),
        ];
        let ts = [target(7, 2)];
        let p = evaluate(&View {
            routes: None,
            team: 0,
            actors: &a,
            targets: &ts,
            home: (0, 0),
            recent_advantage: 1,
            unseen_risk: 0,
            blocked: &|_, _| false,
        });
        assert_eq!(p[0].caller, 1);
        assert_eq!(p[0].jobs.iter().filter(|j| j.job == Job::Damage).count(), 1);
    }
}
