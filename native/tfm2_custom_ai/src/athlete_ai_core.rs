//! Shared, host-free decision policy. Athlete tendencies never change combat arithmetic.
pub const PROFILE_VERSION: i64 = 1;
pub const AEGIS_RADIUS: i64 = 114000;
pub const AEGIS_SHIELD: usize = 320;
pub const AEGIS_SHIELD_TICKS: usize = 180;
pub const PROFILE_FIELDS: [&str; 9] = [
    "ego",
    "judgement",
    "positioning",
    "control_speed",
    "skill_hit",
    "skill_avoid",
    "aggressive",
    "order",
    "roaming",
];
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Profile {
    pub values: [i64; 9],
    pub known: bool,
}
impl Default for Profile {
    fn default() -> Self {
        Self {
            values: [50; 9],
            known: false,
        }
    }
}
impl Profile {
    pub fn new(values: [i64; 9]) -> Self {
        Self {
            values: values.map(|v| v.clamp(0, 100)),
            known: true,
        }
    }
    pub fn ego(self) -> i64 {
        self.values[0]
    }
    pub fn judgement(self) -> i64 {
        self.values[1]
    }
    pub fn positioning(self) -> i64 {
        self.values[2]
    }
    pub fn reaction_ticks(self) -> usize {
        (24 - self.values[3] * 18 / 100) as usize
    }
    /// Confidence changes the price of danger; judgement changes estimation error.
    pub fn score(self, benefit: i64, danger: i64, allied_loss: i64, seed: u64, salt: u64) -> i64 {
        let mut h = seed ^ salt.wrapping_mul(0x9e3779b97f4a7c15);
        h ^= h >> 30;
        h = h.wrapping_mul(0xbf58476d1ce4e5b9);
        h ^= h >> 27;
        let error = ((h % 41) as i64 - 20) * (100 - self.judgement()) / 100;
        benefit + error - danger.max(0) * (160 - self.ego()) / 100 - allied_loss.max(0)
    }
    pub fn prediction_ticks(self, opening: usize) -> usize {
        opening * (30 + self.values[4] as usize * 70 / 100) / 100
    }
}
pub fn sq_distance(a: (i64, i64), b: (i64, i64)) -> i128 {
    (a.0 - b.0) as i128 * (a.0 - b.0) as i128 + (a.1 - b.1) as i128 * (a.1 - b.1) as i128
}
pub fn within(a: (i64, i64), b: (i64, i64), r: i64) -> bool {
    sq_distance(a, b) <= r as i128 * r as i128
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Actor {
    pub id: usize,
    pub team: usize,
    pub point: (i64, i64),
    pub hp: i64,
    pub max_hp: i64,
    pub attack: i64,
    pub speed: i64,
    pub vx: i64,
    pub vy: i64,
    pub visible: bool,
    pub controlled: bool,
    pub immune: bool,
    pub pressure: i64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Objective {
    pub id: usize,
    pub kind: u8,
    pub point: (i64, i64),
    pub hp: i64,
    pub max_hp: i64,
    pub loss_per_second: i64,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Scene {
    pub actors: Vec<Actor>,
    pub objectives: Vec<Objective>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Engage {
    pub centre: (i64, i64),
    pub guest: Option<usize>,
    pub caught: usize,
    pub ready: usize,
    pub incoming: usize,
    pub allied_loss_ticks: usize,
    pub score: i64,
    pub objective: Option<usize>,
    pub reason: &'static str,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TeleportClock {
    pub step: usize,
    pub para: usize,
    pub ultimate: usize,
    pub smoke: usize,
}
#[derive(Clone, Debug, Default)]
pub struct ShieldLedger {
    pub published: bool,
    pub served: Vec<usize>,
}
impl ShieldLedger {
    pub fn new(selected: usize) -> Self {
        Self {
            published: false,
            served: vec![selected],
        }
    }
    pub fn first_marker(
        &mut self,
        team: usize,
        centre: (i64, i64),
        all: &[(usize, usize, i64, (i64, i64))],
    ) -> Vec<usize> {
        if self.published {
            return vec![];
        }
        self.published = true;
        let mut ids = all
            .iter()
            .filter(|u| {
                u.1 == team
                    && u.2 > 0
                    && within(u.3, centre, AEGIS_RADIUS)
                    && !self.served.contains(&u.0)
            })
            .map(|u| u.0)
            .collect::<Vec<_>>();
        ids.sort_unstable();
        ids.dedup();
        self.served.extend_from_slice(&ids);
        ids
    }
}
impl TeleportClock {
    fn cooldown(t: usize, base: usize, haste: i64) -> usize {
        t + base * 100 / (100 + haste.clamp(-90, 900)) as usize
    }
    pub fn step_ready(&self, t: usize) -> bool {
        t >= self.step
    }
    pub fn para_ready(&self, t: usize) -> bool {
        t >= self.para
    }
    pub fn ult_ready(&self, t: usize) -> bool {
        t >= self.ultimate
    }
    pub fn commit_step(&mut self, t: usize, haste: i64) {
        self.step = Self::cooldown(t, 420, haste);
    }
    pub fn commit_para(&mut self, t: usize, haste: i64) {
        self.para = Self::cooldown(t, 420, haste);
    }
    pub fn commit_ult(&mut self, t: usize, haste: i64) {
        self.ultimate = Self::cooldown(t, 4800, haste);
    }
}
impl Scene {
    /// Without a public path-length query, a blocked direct approach receives
    /// a conservative detour estimate. It never makes a blocked route faster.
    pub fn route_adjusted(
        &self,
        team: usize,
        centre: (i64, i64),
        radius: i64,
        wall: &dyn Fn(i64, i64) -> bool,
    ) -> Self {
        let mut result = self.clone();
        for ally in result
            .actors
            .iter_mut()
            .filter(|u| u.team == team && !within(u.point, centre, radius + 12000))
        {
            let d = sq_distance(ally.point, centre).isqrt().max(1) as i64;
            let target = (
                centre.0 + (ally.point.0 - centre.0) * (radius + 12000) / d,
                centre.1 + (ally.point.1 - centre.1) * (radius + 12000) / d,
            );
            if (1..=16).any(|n| {
                wall(
                    ally.point.0 + (target.0 - ally.point.0) * n / 16,
                    ally.point.1 + (target.1 - ally.point.1) * n / 16,
                )
            }) {
                ally.speed = (ally.speed / 4).max(1);
            }
        }
        result
    }
    /// Scores actual control windows, including the allies who cannot move in them.
    pub fn engage(
        &self,
        p: Profile,
        owner: usize,
        centre: (i64, i64),
        radius: i64,
        opening: usize,
        control: usize,
        escape: bool,
        seed: u64,
    ) -> Option<Engage> {
        let me = self.actors.iter().find(|u| u.id == owner && u.hp > 0)?;
        let guest = self
            .actors
            .iter()
            .filter(|u| {
                u.id != owner && u.team == me.team && u.hp > 0 && within(u.point, centre, radius)
            })
            .min_by_key(|u| (sq_distance(u.point, centre), u.id))
            .map(|u| u.id);
        let forecast = p.prediction_ticks(opening) as i64;
        let caught = self
            .actors
            .iter()
            .filter(|u| {
                u.team != me.team
                    && u.hp > 0
                    && u.visible
                    && !u.immune
                    && within(
                        (
                            u.point.0 + u.vx * if u.controlled { 0 } else { forecast },
                            u.point.1 + u.vy * if u.controlled { 0 } else { forecast },
                        ),
                        centre,
                        radius - 1500,
                    )
            })
            .count();
        if caught == 0 {
            return None;
        }
        let mut ready = 0;
        let mut incoming = 0;
        let mut ally_loss = 0;
        let mut closest = usize::MAX;
        for u in self
            .actors
            .iter()
            .filter(|u| u.id != owner && u.team == me.team && u.hp > 0)
        {
            if Some(u.id) != guest && within(u.point, centre, radius) {
                ally_loss += control;
                continue;
            }
            if u.controlled || u.hp * 100 < u.max_hp * 20 {
                continue;
            }
            let distance = sq_distance(u.point, centre).isqrt() as i64;
            let eta = ((distance - radius - 12000).max(0) + u.speed.max(1) - 1) / u.speed.max(1);
            closest = closest.min(eta as usize);
            if eta as usize <= opening + control {
                incoming += 1;
                if eta as usize <= opening + 12 {
                    ready += 1;
                }
            }
        }
        let objective = self
            .objectives
            .iter()
            .filter(|o| {
                o.hp > 0
                    && within(o.point, centre, 150000)
                    && self.actors.iter().any(|u| {
                        u.team != me.team
                            && u.visible
                            && u.hp > 0
                            && within(u.point, o.point, 120000)
                    })
            })
            .min_by_key(|o| (sq_distance(o.point, centre), o.id));
        // Damage already released is not stopped by either field. Observed
        // objective loss is a conservative estimate, not hidden projectile data.
        let urgency = objective.map_or(0, |o| {
            if o.loss_per_second > 0 {
                let arrival = if closest == usize::MAX {
                    opening + control
                } else {
                    closest.min(opening + control)
                };
                if o.hp <= o.loss_per_second * (opening as i64 + arrival as i64 / 4) / 60 {
                    -35
                } else {
                    (60 - o.hp * 10 / o.loss_per_second).clamp(0, 60)
                }
            } else {
                8
            }
        });
        let enemies_after = self
            .actors
            .iter()
            .filter(|u| {
                u.team != me.team
                    && u.visible
                    && u.hp > 0
                    && within(u.point, centre, radius + 35000)
            })
            .count() as i64;
        let hp_pct = (me.hp - me.pressure).max(0) * 100 / me.max_hp.max(1);
        let danger = (enemies_after - incoming as i64 - 1).max(0) * 12
            + (55 - hp_pct).max(0)
            + if escape { 0 } else { 18 };
        let bridge = if ready == 0 && incoming > 0 { 20 } else { 0 };
        let duel = if caught == 1
            && enemies_after == 1
            && self.actors.iter().any(|u| {
                u.team != me.team
                    && u.visible
                    && u.hp > me.attack * 3
                    && within(u.point, centre, radius)
            }) {
            16
        } else {
            0
        };
        let benefit = caught as i64 * control as i64 / 12
            + ready as i64 * 12
            + incoming as i64 * 5
            + bridge
            + urgency
            + duel
            + p.values[6] / 15
            + p.values[7] * incoming as i64 / 50
            + if enemies_after > 1 && incoming == 0 && closest > opening + control + 60 {
                -30
            } else {
                0
            };
        let allied_cost = ally_loss as i64 / 3;
        let score = p.score(
            benefit,
            danger,
            allied_cost,
            seed,
            centre.0 as u64 ^ ((centre.1 as u64) << 16) ^ control as u64,
        );
        Some(Engage {
            centre,
            guest,
            caught,
            ready,
            incoming,
            allied_loss_ticks: ally_loss,
            score,
            objective: objective.map(|o| o.id),
            reason: if score < 0 {
                "Wait: arrival or survival cost"
            } else if objective.is_some() && ready == 0 {
                "Stall objective until allies arrive"
            } else if ready > 0 {
                "Engage with immediate follow-up"
            } else {
                "Catch visible enemies"
            },
        })
    }
    pub fn teleport_score(
        &self,
        p: Profile,
        owner: usize,
        to: (i64, i64),
        range: i64,
        channel: usize,
        seed: u64,
    ) -> i64 {
        let Some(me) = self.actors.iter().find(|u| u.id == owner) else {
            return i64::MIN / 2;
        };
        let near = self
            .actors
            .iter()
            .filter(|u| {
                u.team != me.team && u.visible && u.hp > 0 && within(u.point, to, range + 15000)
            })
            .count() as i64;
        let allies = self
            .actors
            .iter()
            .filter(|u| {
                u.team == me.team && u.id != owner && u.hp > 0 && within(u.point, to, 90000)
            })
            .count() as i64;
        let exposed = self
            .actors
            .iter()
            .filter(|u| u.team != me.team && u.visible && u.hp > 0 && within(u.point, to, 30000))
            .count() as i64;
        let saving = self
            .actors
            .iter()
            .filter(|u| {
                u.team == me.team && u.hp * 100 < u.max_hp * 40 && within(u.point, to, 70000)
            })
            .count() as i64;
        let objective = self
            .objectives
            .iter()
            .any(|o| within(o.point, to, 100000) && o.loss_per_second > 0);
        let chasing = self
            .actors
            .iter()
            .filter(|u| {
                u.team != me.team && u.visible && u.hp > 0 && within(u.point, me.point, 45000)
            })
            .count() as i64;
        let escape = chasing > 0 && exposed < chasing && me.hp * 100 < me.max_hp * 55;
        let health = (me.hp - me.pressure).max(0) * 100 / me.max_hp.max(1);
        let danger = (exposed - allies).max(0) * 25
            + (50 - health).max(0)
            + if channel > 60 { exposed * 5 } else { 0 };
        let benefit = near * 16
            + allies * 12
            + saving * 15
            + if objective { 25 } else { 0 }
            + if escape { 70 } else { 0 }
            + p.values[8] / 12
            + p.values[6] / 15;
        p.score(
            benefit,
            danger,
            0,
            seed,
            to.0 as u64 ^ ((to.1 as u64) << 16),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn actor(id: usize, team: usize, x: i64) -> Actor {
        Actor {
            id,
            team,
            point: (x, 200000),
            hp: 1000,
            max_hp: 1000,
            attack: 100,
            speed: 1000,
            vx: 0,
            vy: 0,
            visible: true,
            controlled: false,
            immune: false,
            pressure: 0,
        }
    }
    #[test]
    fn ego_changes_risk_price_without_changing_capability() {
        let a = Profile::new([0, 100, 50, 50, 50, 50, 50, 50, 50]);
        let b = Profile::new([100, 100, 50, 50, 50, 50, 50, 50, 50]);
        assert!(b.score(60, 50, 0, 0, 0) > a.score(60, 50, 0, 0, 0));
        assert!(b.score(60, 50, 200, 0, 0) < 0);
    }
    #[test]
    fn judgement_is_deterministic_and_improves_estimation() {
        let low = Profile::new([50, 0, 50, 50, 50, 50, 50, 50, 50]);
        let high = Profile::new([50, 100, 50, 50, 50, 50, 50, 50, 50]);
        assert_eq!(high.score(100, 0, 0, 1, 2), 100);
        assert_eq!(low.score(100, 0, 0, 1, 2), low.score(100, 0, 0, 1, 2));
        assert!((0..16).any(|s| low.score(100, 0, 0, s, 2) != 100));
    }
    #[test]
    fn arriving_allies_make_a_stall_useful() {
        let p = Profile::new([50, 100, 80, 80, 80, 80, 50, 50, 50]);
        let mut s = Scene {
            actors: vec![
                actor(1, 0, 200000),
                actor(2, 1, 210000),
                actor(3, 1, 220000),
            ],
            objectives: vec![],
        };
        let without = s
            .engage(p, 1, (200000, 200000), 76000, 0, 120, true, 0)
            .unwrap();
        s.actors.push(actor(4, 0, 350000));
        let with = s
            .engage(p, 1, (200000, 200000), 76000, 0, 120, true, 0)
            .unwrap();
        assert_eq!(with.incoming, 1);
        assert!(with.score > without.score);
    }
    #[test]
    fn only_one_guest_is_free_and_hidden_foes_are_ignored() {
        let mut s = Scene {
            actors: vec![
                actor(1, 0, 200000),
                actor(2, 1, 210000),
                actor(3, 0, 201000),
                actor(4, 0, 202000),
            ],
            objectives: vec![],
        };
        let e = s
            .engage(
                Profile::default(),
                1,
                (200000, 200000),
                76000,
                0,
                120,
                true,
                0,
            )
            .unwrap();
        assert_eq!(e.guest, Some(3));
        assert_eq!(e.allied_loss_ticks, 120);
        s.actors[1].visible = false;
        assert!(s
            .engage(
                Profile::default(),
                1,
                (200000, 200000),
                76000,
                0,
                120,
                true,
                0
            )
            .is_none());
    }
    #[test]
    fn dead_or_controlled_allies_cannot_supply_follow_up() {
        let mut s = Scene {
            actors: vec![
                actor(1, 0, 200000),
                actor(2, 1, 210000),
                actor(3, 0, 350000),
            ],
            objectives: vec![],
        };
        s.actors[2].controlled = true;
        assert_eq!(
            s.engage(
                Profile::default(),
                1,
                (200000, 200000),
                76000,
                0,
                120,
                true,
                0
            )
            .unwrap()
            .incoming,
            0
        );
    }
    #[test]
    fn prediction_tracks_a_visible_escape_instead_of_granting_hidden_knowledge() {
        let mut s = Scene {
            actors: vec![actor(1, 0, 200000), actor(2, 1, 210000)],
            objectives: vec![],
        };
        s.actors[1].vx = 1000;
        let low = Profile::new([50, 100, 50, 50, 0, 50, 50, 50, 50]);
        let high = Profile::new([50, 100, 50, 50, 100, 50, 50, 50, 50]);
        assert!(s
            .engage(low, 1, (200000, 200000), 64000, 60, 240, true, 0)
            .is_some());
        assert!(s
            .engage(high, 1, (200000, 200000), 64000, 60, 240, true, 0)
            .is_none());
    }
    #[test]
    fn incoming_damage_can_invalidate_an_otherwise_healthy_entry() {
        let p = Profile::new([50, 100, 50, 50, 50, 50, 50, 50, 50]);
        let mut s = Scene {
            actors: vec![actor(1, 0, 200000), actor(2, 1, 210000)],
            objectives: vec![],
        };
        let before = s
            .engage(p, 1, (200000, 200000), 76000, 0, 120, false, 0)
            .unwrap()
            .score;
        s.actors[0].pressure = 850;
        assert!(
            s.engage(p, 1, (200000, 200000), 76000, 0, 120, false, 0)
                .unwrap()
                .score
                < before
        );
    }
    #[test]
    fn objective_loss_that_continues_through_control_reduces_stall_value() {
        let p = Profile::new([50, 100, 50, 50, 50, 50, 50, 50, 50]);
        let mut s = Scene {
            actors: vec![
                actor(1, 0, 200000),
                actor(2, 1, 210000),
                actor(3, 0, 420000),
            ],
            objectives: vec![Objective {
                id: 99,
                kind: 0,
                point: (210000, 200000),
                hp: 10000,
                max_hp: 20000,
                loss_per_second: 1000,
            }],
        };
        let viable = s
            .engage(p, 1, (200000, 200000), 64000, 60, 240, false, 0)
            .unwrap();
        s.objectives[0].hp = 200;
        let dying = s
            .engage(p, 1, (200000, 200000), 64000, 60, 240, false, 0)
            .unwrap();
        assert_eq!(viable.objective, Some(99));
        assert!(dying.score < viable.score);
    }
    #[test]
    fn full_control_can_bridge_an_arrival_that_quick_void_cannot() {
        let s = Scene {
            actors: vec![
                actor(1, 0, 200000),
                actor(2, 1, 210000),
                actor(3, 0, 470000),
            ],
            objectives: vec![],
        };
        let full = s
            .engage(
                Profile::default(),
                1,
                (200000, 200000),
                64000,
                60,
                240,
                false,
                0,
            )
            .unwrap();
        let quick = s
            .engage(
                Profile::default(),
                1,
                (200000, 200000),
                96000,
                12,
                60,
                false,
                0,
            )
            .unwrap();
        assert_eq!(full.incoming, 1);
        assert_eq!(quick.incoming, 0);
    }
    #[test]
    fn remote_support_does_not_require_a_local_foe() {
        let s = Scene {
            actors: vec![
                actor(1, 0, 100000),
                actor(2, 1, 500000),
                actor(3, 0, 520000),
            ],
            objectives: vec![],
        };
        let p = Profile::new([50, 100, 50, 50, 50, 50, 50, 50, 50]);
        assert!(
            s.teleport_score(p, 1, (465000, 200000), 42000, 120, 0)
                > s.teleport_score(p, 1, (100000, 200000), 42000, 120, 0)
        );
    }
    #[test]
    fn omen_clocks_are_independent_and_keep_commitments_after_interruption() {
        let mut c = TeleportClock::default();
        c.commit_para(10, 0);
        assert!(c.step_ready(10));
        c.commit_step(10, 0);
        assert!(!c.step_ready(429));
        assert!(c.step_ready(430));
        assert!(!c.para_ready(429));
        assert!(c.para_ready(430));
        c.commit_ult(20, 0);
        assert!(!c.ult_ready(4819));
        assert!(c.ult_ready(4820));
        let other = c.clone();
        c.commit_step(500, 100);
        assert_eq!(c.step, 710);
        assert_eq!(other.step, 430);
    }
    #[test]
    fn first_marker_deduplicates_and_never_refreshes_the_selected_ally() {
        let mut l = ShieldLedger::new(1);
        let all = [
            (1, 0, 100, (0, 0)),
            (2, 0, 100, (114000, 0)),
            (2, 0, 100, (114000, 0)),
            (3, 0, 100, (114001, 0)),
            (4, 1, 100, (0, 0)),
            (5, 0, 0, (0, 0)),
        ];
        assert_eq!(l.first_marker(0, (0, 0), &all), vec![2]);
        assert!(l.first_marker(0, (114001, 0), &all).is_empty());
        assert_eq!(l.served, vec![1, 2]);
    }
}
