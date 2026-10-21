use crate::rework_data::*;
// Deterministic Frieren rules. No host, presentation, persistence or wall-clock state.
#[derive(Clone, Debug)]
pub struct Unit {
    pub id: usize,
    pub team: usize,
    pub x: i64,
    pub y: i64,
    pub hp: i64,
    pub max_hp: i64,
    pub ap: i64,
    pub visible: bool,
    pub targetable: bool,
    pub champion: bool,
    pub farmable: bool,
    pub controlled: bool,
    pub vx: i64,
    pub vy: i64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Quick(usize),
    Charged(usize),
    Barrier(usize),
    Ultimate,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Cmd {
    StarkWarning {
        x: i64,
        y: i64,
        ticks: usize,
    },
    StarkPose {
        x: i64,
        y: i64,
        height: i64,
        descending: bool,
    },
    StarkLanding {
        x: i64,
        y: i64,
    },
    StarkCancel,
    Control {
        target: usize,
        airborne: usize,
        slow: i64,
        ticks: usize,
    },
    Damage {
        target: usize,
        amount: i64,
    },
    Shield {
        target: usize,
        amount: i64,
        ticks: usize,
    },
    Empower {
        ticks: usize,
    },
    SpawnStark {
        hp: i64,
        attack: i64,
    },
    RemoveStark,
    MoveStark {
        x: i64,
        y: i64,
    },
    Warning {
        target: usize,
        ticks: usize,
        charged: bool,
    },
    Shot {
        x: i64,
        y: i64,
        dx: i64,
        dy: i64,
        charged: bool,
    },
    BarrierArt {
        target: usize,
    },
    PartyArt,
}
pub struct Input<'a> {
    pub tick: usize,
    pub owner: &'a Unit,
    pub units: &'a [Unit],
    pub ult_unlocked: bool,
    pub spell_unlocked: bool,
    pub barrier_unlocked: bool,
    pub automatic: bool,
    pub action: Option<Action>,
    pub stark: Option<Stark>,
    pub wall: &'a dyn Fn(i64, i64) -> bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Stark {
    pub id: usize,
    pub x: i64,
    pub y: i64,
    pub hp: i64,
    pub interrupted: bool,
}
impl Stark {
    pub fn at(x: i64, y: i64) -> Self {
        Self {
            id: 999,
            x,
            y,
            hp: 1,
            interrupted: false,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Leap {
    pub stark: usize,
    pub target: usize,
    pub from: (i64, i64),
    pub to: (i64, i64),
    pub start: usize,
    pub end: usize,
}
#[derive(Clone, Debug)]
pub struct Cast {
    pub target: usize,
    pub end: usize,
    pub charged: bool,
}
#[derive(Clone, Debug)]
pub struct Ray {
    #[allow(dead_code)] // exported by the Frieren Lab state (frieren_protocol.rs)
    pub id: usize,
    pub x: i64,
    pub y: i64,
    pub dx: i64,
    pub dy: i64,
    pub remaining: i64,
    pub damage: i64,
    pub hits: Vec<usize>,
}
#[derive(Clone, Debug)]
pub struct Core {
    pub leap: Option<Leap>,
    pub next_leap: usize,
    pub jumped: Vec<usize>,
    /// Controlled regression/lab switch. Native gameplay always keeps the default enabled.
    pub leaps_enabled: bool,
    pub mana: i64,
    pub skill_at: usize,
    pub barrier_at: usize,
    pub ult_at: usize,
    pub ult_until: usize,
    pub stark_until: usize,
    pub guard: Option<usize>,
    pub guard_until: usize,
    pub selected: Option<usize>,
    pub pending: Option<Cast>,
    pub rays: Vec<Ray>,
    pub reason: &'static str,
    pub casts: [usize; 4],
    pub interrupted: usize,
    pub requested_damage: i64,
    pub next_fern: usize,
    pub next_stark: usize,
    last_tick: Option<usize>,
    was_alive: bool,
    stark_requested: bool,
    next_ray: usize,
}
impl Default for Core {
    fn default() -> Self {
        Self {
            leap: None,
            next_leap: 0,
            jumped: vec![],
            leaps_enabled: true,
            mana: FR_MANA_MAX,
            skill_at: 0,
            barrier_at: 0,
            ult_at: 0,
            ult_until: 0,
            stark_until: 0,
            guard: None,
            guard_until: 0,
            selected: None,
            pending: None,
            rays: vec![],
            reason: "Ready",
            casts: [0; 4],
            interrupted: 0,
            requested_damage: 0,
            next_fern: 0,
            next_stark: 0,
            last_tick: None,
            was_alive: false,
            stark_requested: false,
            next_ray: 0,
        }
    }
}
fn distance(a: (i64, i64), b: (i64, i64)) -> i128 {
    let (x, y) = (a.0 - b.0, a.1 - b.1);
    x as i128 * x as i128 + y as i128 * y as i128
}
fn direction(dx: i64, dy: i64) -> (i64, i64) {
    crate::unified_theory_math::unit(dx, dy)
}
fn near(a: (i64, i64), b: (i64, i64), p: (i64, i64)) -> bool {
    let (dx, dy) = ((b.0 - a.0) as i128, (b.1 - a.1) as i128);
    let n = dx * dx + dy * dy;
    let dot = if n == 0 {
        0
    } else {
        ((p.0 - a.0) as i128 * dx + (p.1 - a.1) as i128 * dy).clamp(0, n)
    };
    let q = if n == 0 {
        a
    } else {
        (a.0 + (dx * dot / n) as i64, a.1 + (dy * dot / n) as i64)
    };
    distance(q, p) <= 15_000i128.pow(2)
}
impl Core {
    pub fn died(&mut self) {
        self.leap = None;
        self.jumped.clear();
        self.pending = None;
        self.rays.clear();
        self.ult_until = 0;
        self.stark_until = 0;
        self.stark_requested = false;
        self.was_alive = false;
    }

    fn enemy<'a>(input: &'a Input, id: usize) -> Option<&'a Unit> {
        input.units.iter().find(|u| {
            u.id == id && u.team != input.owner.team && u.hp > 0 && u.visible && u.targetable
        })
    }
    fn target<'a>(&self, input: &'a Input) -> Option<&'a Unit> {
        input
            .units
            .iter()
            .filter(|u| {
                u.team != input.owner.team
                    && u.hp > 0
                    && u.visible
                    && u.targetable
                    && (u.champion || u.farmable)
                    && distance((u.x, u.y), (input.owner.x, input.owner.y))
                        <= (FR_SKILL_RANGE as i128).pow(2)
            })
            .filter(|u| {
                let m = input.owner;
                let n = ((u.x - m.x).abs().max((u.y - m.y).abs()) / 4000 + 1).clamp(1, 64);
                (1..=n).all(|i| !(input.wall)(m.x + (u.x - m.x) * i / n, m.y + (u.y - m.y) * i / n))
            })
            .min_by_key(|u| {
                (
                    !u.champion,
                    distance((u.x, u.y), (input.owner.x, input.owner.y)),
                    u.hp,
                    u.id,
                )
            })
    }
    fn shoot(
        &mut self,
        from: (i64, i64),
        target: &Unit,
        damage: i64,
        range: i64,
        charged: bool,
        out: &mut Vec<Cmd>,
    ) {
        let (dx, dy) = direction(target.x - from.0, target.y - from.1);
        if (dx, dy) == (0, 0) {
            return;
        }
        self.next_ray += 1;
        self.rays.push(Ray {
            id: self.next_ray,
            x: from.0,
            y: from.1,
            dx,
            dy,
            remaining: range,
            damage: damage * FR_DAMAGE_PERCENT / 100,
            hits: vec![],
        });
        out.push(Cmd::Shot {
            x: from.0,
            y: from.1,
            dx,
            dy,
            charged,
        });
    }
    fn begin(&mut self, input: &Input, action: Action, out: &mut Vec<Cmd>) -> bool {
        let t = input.tick;
        let m = input.owner;
        if m.hp <= 0 || m.controlled {
            return false;
        }
        match action {
            Action::Quick(id) | Action::Charged(id) => {
                let charged = matches!(action, Action::Charged(_));
                let cost = if charged {
                    FR_CHARGED_COST
                } else {
                    FR_QUICK_COST
                };
                if !input.spell_unlocked
                    || self.pending.is_some()
                    || t < self.skill_at
                    || self.mana < cost
                    || Self::enemy(input, id).is_none_or(|u| {
                        distance((u.x, u.y), (m.x, m.y)) > (FR_SKILL_RANGE as i128).pow(2)
                    })
                {
                    return false;
                }
                self.mana -= cost;
                self.skill_at = t + FR_SKILL_TICKS;
                self.selected = Some(id);
                let ticks = if charged {
                    FR_CHARGED_TICKS
                } else {
                    FR_QUICK_TICKS
                };
                self.pending = Some(Cast {
                    target: id,
                    end: t + ticks,
                    charged,
                });
                self.casts[usize::from(charged)] += 1;
                self.reason = if charged {
                    "Charging a predicted opening"
                } else {
                    "Maintaining ranged pressure"
                };
                out.push(Cmd::Warning {
                    target: id,
                    ticks,
                    charged,
                });
                true
            }
            Action::Barrier(id) => {
                if !input.barrier_unlocked || t < self.barrier_at || self.mana < FR_BARRIER_COST {
                    return false;
                }
                let Some(u) = input
                    .units
                    .iter()
                    .find(|u| {
                        u.id == id
                            && u.team == m.team
                            && u.hp > 0
                            && distance((u.x, u.y), (m.x, m.y)) <= (FR_BARRIER_RANGE as i128).pow(2)
                    })
                    .or_else(|| (id == m.id).then_some(m))
                else {
                    return false;
                };
                self.mana -= FR_BARRIER_COST;
                self.barrier_at = t + FR_BARRIER_TICKS;
                self.guard = Some(u.id);
                self.guard_until = t + FR_SHIELD_TICKS;
                self.casts[2] += 1;
                self.reason = "Protecting an endangered ally";
                out.push(Cmd::Shield {
                    target: u.id,
                    amount: FR_BARRIER_BASE + m.ap * FR_BARRIER_AP / 100,
                    ticks: FR_SHIELD_TICKS,
                });
                out.push(Cmd::BarrierArt { target: u.id });
                true
            }
            Action::Ultimate => {
                if !input.ult_unlocked || t < self.ult_at {
                    return false;
                }
                self.ult_at = t + FR_ULT_TICKS;
                self.ult_until = t + FR_EMPOWER_TICKS;
                self.stark_until = t + FR_STARK_TICKS;
                self.stark_requested = true;
                self.leap = None;
                self.jumped.clear();
                self.next_leap = t;
                self.next_fern = t + 30;
                self.next_stark = t + 30;
                self.casts[3] += 1;
                self.reason = "Revealing mana with the party";
                out.push(Cmd::Empower {
                    ticks: FR_EMPOWER_TICKS,
                });
                out.push(Cmd::SpawnStark {
                    hp: FR_STARK_BASE_HP + m.max_hp / FR_STARK_HP_DIVISOR,
                    attack: FR_STARK_BASE + m.ap * FR_STARK_AP / 100,
                });
                out.push(Cmd::PartyArt);
                true
            }
        }
    }
    pub fn step(&mut self, input: &Input) -> Vec<Cmd> {
        let mut out = vec![];
        let t = input.tick;
        let m = input.owner;
        if self.last_tick == Some(t) {
            return out;
        }
        let old = self.last_tick.unwrap_or(t);
        self.last_tick = Some(t);
        if m.hp <= 0 {
            if self.leap.take().is_some() {
                out.push(Cmd::StarkCancel);
            }
            self.pending = None;
            self.rays.clear();
            self.ult_until = 0;
            self.stark_until = 0;
            if self.stark_requested {
                out.push(Cmd::RemoveStark);
            }
            self.stark_requested = false;
            self.was_alive = false;
            self.reason = "Down; cooldowns preserved";
            return out;
        }
        if self.was_alive {
            let dt = t.saturating_sub(old);
            let empowered = t
                .min(self.ult_until)
                .saturating_sub(old.min(self.ult_until));
            self.mana = (self.mana
                + dt as i64 * FR_MANA_REGEN
                + empowered as i64 * (FR_MANA_ULT_REGEN - FR_MANA_REGEN))
                .min(FR_MANA_MAX);
        }
        if !self.was_alive {
            self.reason = "Ready";
        }
        self.was_alive = true;
        if self.stark_requested && t >= self.stark_until {
            if self.leap.take().is_some() {
                out.push(Cmd::StarkCancel);
            }
            out.push(Cmd::RemoveStark);
            self.stark_requested = false;
        }
        if t >= self.guard_until {
            self.guard = None;
        }
        if m.controlled {
            if self.pending.take().is_some() {
                self.interrupted += 1;
                self.reason = "Interrupted; mana and cooldown spent";
            }
        }
        if let Some(p) = self.pending.clone().filter(|p| t >= p.end) {
            self.pending = None;
            if let Some(u) = Self::enemy(input, p.target)
                .filter(|u| distance((u.x, u.y), (m.x, m.y)) <= (FR_SKILL_RANGE as i128).pow(2))
            {
                let amount = if p.charged {
                    FR_CHARGED_BASE + m.ap * FR_CHARGED_AP / 100
                } else {
                    FR_QUICK_BASE + m.ap * FR_QUICK_AP / 100
                };
                self.shoot((m.x, m.y), u, amount, FR_SKILL_RANGE, p.charged, &mut out);
            } else {
                self.interrupted += 1;
                self.reason = "Target lost before release";
            }
        }
        if input.automatic && !m.controlled {
            let ally = input
                .units
                .iter()
                .filter(|u| {
                    u.team == m.team
                        && u.champion
                        && u.hp > 0
                        && u.hp * 2 < u.max_hp
                        && distance((u.x, u.y), (m.x, m.y)) <= (FR_BARRIER_RANGE as i128).pow(2)
                        && input.units.iter().any(|e| {
                            e.team != m.team
                                && e.champion
                                && e.hp > 0
                                && e.visible
                                && distance((e.x, e.y), (u.x, u.y))
                                    <= (FR_DANGER_RADIUS as i128).pow(2)
                        })
                })
                .min_by_key(|u| (u.hp * 100 / u.max_hp.max(1), u.id))
                .map(|u| u.id);
            if let Some(id) = ally {
                self.begin(input, Action::Barrier(id), &mut out);
            }
            if let Some(u) = self.target(input) {
                self.selected = Some(u.id);
                if u.champion && m.hp * 100 >= m.max_hp * FR_ULT_MIN_HP_PERCENT {
                    self.begin(input, Action::Ultimate, &mut out);
                }
                let threatened = input.units.iter().any(|e| {
                    e.team != m.team
                        && e.champion
                        && e.hp > 0
                        && e.visible
                        && distance((e.x, e.y), (m.x, m.y)) < (FR_THREAT_RADIUS as i128).pow(2)
                });
                let stable = u.controlled || (u.vx.abs() + u.vy.abs() < 500);
                let charged = u.champion && stable && !threatened && self.mana >= FR_CHARGED_COST;
                self.begin(
                    input,
                    if charged {
                        Action::Charged(u.id)
                    } else {
                        Action::Quick(u.id)
                    },
                    &mut out,
                );
            } else if self.pending.is_none() {
                self.reason = "No legal visible target";
            }
        }
        if let Some(action) = input.action {
            self.begin(input, action, &mut out);
        }
        if t < self.ult_until && t >= self.next_fern {
            if let Some(u) = self
                .selected
                .and_then(|id| Self::enemy(input, id))
                .or_else(|| self.target(input))
                .filter(|u| u.champion)
            {
                self.shoot(
                    (m.x + FR_FERN_OFFSET, m.y),
                    u,
                    FR_FERN_BASE + m.ap * FR_FERN_AP / 100,
                    FR_FERN_RANGE,
                    true,
                    &mut out,
                );
                self.next_fern = t + FR_FERN_INTERVAL_TICKS;
            }
        }
        self.update_stark(input, &mut out);
        for ray in &mut self.rays {
            let from = (ray.x, ray.y);
            let speed = FR_RAY_SPEED.min(ray.remaining);
            let to = (
                ray.x + ray.dx * speed / 10000,
                ray.y + ray.dy * speed / 10000,
            );
            if (1..=8).any(|i| {
                (input.wall)(
                    from.0 + (to.0 - from.0) * i / 8,
                    from.1 + (to.1 - from.1) * i / 8,
                )
            }) {
                ray.remaining = 0;
                continue;
            }
            for u in input
                .units
                .iter()
                .filter(|u| u.team != m.team && u.hp > 0 && u.visible && u.targetable)
            {
                if !ray.hits.contains(&u.id) && near(from, to, (u.x, u.y)) {
                    ray.hits.push(u.id);
                    out.push(Cmd::Damage {
                        target: u.id,
                        amount: ray.damage,
                    });
                    self.requested_damage += ray.damage;
                }
            }
            (ray.x, ray.y) = to;
            ray.remaining -= speed;
        }
        self.rays.retain(|r| r.remaining > 0);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn unit(id: usize, team: usize, x: i64) -> Unit {
        Unit {
            id,
            team,
            x,
            y: 100_000,
            hp: 1000,
            max_hp: 1000,
            ap: 100,
            visible: true,
            targetable: true,
            champion: true,
            farmable: false,
            controlled: false,
            vx: 0,
            vy: 0,
        }
    }
    fn step(
        c: &mut Core,
        t: usize,
        m: &Unit,
        units: &[Unit],
        action: Option<Action>,
        ult: bool,
    ) -> Vec<Cmd> {
        c.step(&Input {
            tick: t,
            owner: m,
            units,
            action,
            automatic: false,
            ult_unlocked: ult,
            spell_unlocked: true,
            barrier_unlocked: true,
            stark: None,
            wall: &|_, _| false,
        })
    }
    #[test]
    fn quick_and_charged_damage_have_one_hit_per_ray() {
        for charged in [false, true] {
            let mut c = Core::default();
            let m = unit(1, 0, 100_000);
            let u = unit(2, 1, 150_000);
            let mut hits = 0;
            for t in 0..100 {
                let cmd = step(
                    &mut c,
                    t,
                    &m,
                    &[m.clone(), u.clone()],
                    (t == 0).then_some(if charged {
                        Action::Charged(2)
                    } else {
                        Action::Quick(2)
                    }),
                    true,
                );
                for a in cmd {
                    if let Cmd::Damage { amount, .. } = a {
                        assert_eq!(amount, if charged { 148 } else { 96 });
                        hits += 1;
                    }
                }
            }
            assert_eq!(hits, 1);
        }
    }
    #[test]
    fn spellcraft_waits_for_its_skill_to_be_learned() {
        let mut c = Core::default();
        let m = unit(1, 0, 100_000);
        let u = unit(2, 1, 150_000);
        let units = [m.clone(), u.clone()];
        let input = |t, spell| Input {
            tick: t,
            owner: &m,
            units: &units,
            action: Some(Action::Quick(2)),
            automatic: false,
            ult_unlocked: false,
            spell_unlocked: spell,
            barrier_unlocked: false,
            stark: None,
            wall: &|_, _| false,
        };
        let out = c.step(&input(0, false));
        assert!(!out.iter().any(|c| matches!(c, Cmd::Warning { .. })));
        assert_eq!(c.mana, FR_MANA_MAX);
        let out = c.step(&input(1, true));
        assert!(out
            .iter()
            .any(|c| matches!(c, Cmd::Warning { target: 2, .. })));
    }
    #[test]
    fn interrupted_cast_spends_mana_and_keeps_cooldown() {
        let mut c = Core::default();
        let mut m = unit(1, 0, 100_000);
        let u = unit(2, 1, 150_000);
        step(&mut c, 0, &m, &[u.clone()], Some(Action::Charged(2)), true);
        m.controlled = true;
        step(&mut c, 1, &m, &[u], None, true);
        assert!(c.pending.is_none());
        assert_eq!(c.skill_at, 240);
        assert_eq!(c.mana, 3908);
        assert_eq!(c.interrupted, 1);
    }
    #[test]
    fn mana_regeneration_crosses_ultimate_boundary_exactly() {
        let mut c = Core::default();
        let m = unit(1, 0, 100_000);
        step(&mut c, 0, &m, &[], Some(Action::Ultimate), true);
        c.mana = 0;
        step(&mut c, 500, &m, &[], None, true);
        assert_eq!(c.mana, 5920);
    }
    #[test]
    fn ultimate_requires_unlock_and_does_not_block_normal_spells() {
        let mut c = Core::default();
        let m = unit(1, 0, 100_000);
        let u = unit(2, 1, 150_000);
        assert!(step(&mut c, 0, &m, &[], Some(Action::Ultimate), false).is_empty());
        assert_eq!(c.ult_at, 0);
        step(&mut c, 1, &m, &[u.clone()], Some(Action::Ultimate), true);
        step(&mut c, 2, &m, &[u], Some(Action::Quick(2)), true);
        assert!(c.pending.is_some());
        assert_eq!(c.casts, [1, 0, 0, 1]);
    }
    #[test]
    fn death_preserves_cooldowns_and_removes_owned_effects() {
        let mut c = Core::default();
        let mut m = unit(1, 0, 100_000);
        step(&mut c, 0, &m, &[], Some(Action::Ultimate), true);
        c.skill_at = 240;
        m.hp = 0;
        assert!(step(&mut c, 2, &m, &[], None, true).contains(&Cmd::RemoveStark));
        assert_eq!(c.ult_at, 3600);
        assert_eq!(c.skill_at, 240);
        assert!(c.rays.is_empty());
    }
    #[test]
    fn hidden_targets_cannot_receive_a_charged_release() {
        let mut c = Core::default();
        let m = unit(1, 0, 100_000);
        let mut u = unit(2, 1, 150_000);
        step(&mut c, 0, &m, &[u.clone()], Some(Action::Charged(2)), true);
        u.visible = false;
        assert!(step(&mut c, 60, &m, &[u], None, true)
            .iter()
            .all(|c| !matches!(c, Cmd::Shot { .. })));
    }
    #[test]
    fn barriers_validate_team_range_cost_and_are_not_immunity() {
        let mut c = Core::default();
        let m = unit(1, 0, 100_000);
        let a = unit(2, 0, 130_000);
        let cmd = step(&mut c, 0, &m, &[a], Some(Action::Barrier(2)), true);
        assert!(cmd.contains(&Cmd::Shield {
            target: 2,
            amount: 135,
            ticks: 90
        }));
        assert_eq!(c.mana, 4200);
        assert_eq!(c.barrier_at, 600);
    }
    #[test]
    fn simulations_clone_independently() {
        let mut a = Core::default();
        let b = a.clone();
        let m = unit(1, 0, 100_000);
        step(&mut a, 0, &m, &[], Some(Action::Ultimate), true);
        assert_eq!(b.ult_at, 0);
        assert!(!b.stark_requested);
    }
    #[test]
    fn no_regeneration_is_awarded_for_a_gap_while_dead() {
        let mut c = Core::default();
        let m = unit(1, 0, 100000);
        step(&mut c, 0, &m, &[], None, true);
        c.mana = 100;
        c.died();
        step(&mut c, 600, &m, &[], None, true);
        assert_eq!(c.mana, 100);
        step(&mut c, 601, &m, &[], None, true);
        assert_eq!(c.mana, 108);
    }
    #[test]
    fn mana_starvation_does_not_start_a_cooldown() {
        let mut c = Core::default();
        c.mana = 0;
        let m = unit(1, 0, 100000);
        let e = unit(2, 1, 150000);
        assert!(step(&mut c, 0, &m, &[e], Some(Action::Quick(2)), true).is_empty());
        assert_eq!(c.skill_at, 0);
    }
    #[test]
    fn walls_stop_a_piercing_ray_before_damage() {
        let mut c = Core::default();
        let m = unit(1, 0, 100000);
        let e = unit(2, 1, 150000);
        let units = [m.clone(), e];
        let wall = |x, _| x >= 120000 && x <= 125000;
        for tick in 0..80 {
            let commands = c.step(&Input {
                tick,
                owner: &m,
                units: &units,
                ult_unlocked: true,
                spell_unlocked: true,
                barrier_unlocked: true,
                automatic: false,
                action: (tick == 0).then_some(Action::Quick(2)),
                stark: None,
                wall: &wall,
            });
            assert!(!commands.iter().any(|a| matches!(a, Cmd::Damage { .. })));
        }
    }
    #[test]
    fn failed_stark_spawn_has_no_damaging_substitute() {
        let mut c = Core::default();
        let m = unit(1, 0, 100000);
        let e = unit(2, 1, 101000);
        let units = [m.clone(), e];
        let mut spawns = 0;
        for tick in 0..721 {
            let commands = c.step(&Input {
                tick,
                owner: &m,
                units: &units,
                ult_unlocked: true,
                spell_unlocked: true,
                barrier_unlocked: true,
                automatic: false,
                action: (tick == 0).then_some(Action::Ultimate),
                stark: None,
                wall: &|_, _| false,
            });
            spawns += commands
                .iter()
                .filter(|a| matches!(a, Cmd::SpawnStark { .. }))
                .count();
            assert!(!commands.iter().any(|a| matches!(a, Cmd::MoveStark { .. })));
        }
        assert_eq!(spawns, 1);
        assert_eq!(c.next_stark, 30);
    }
    #[test]
    fn stark_respects_the_guard_leash_and_real_melee_clock() {
        let mut c = Core::default();
        c.leaps_enabled = false;
        let m = unit(1, 0, 100000);
        let mut e = unit(2, 1, 110000);
        let ally = unit(3, 0, 150000);
        let mut units = vec![m.clone(), e.clone(), ally];
        step(&mut c, 0, &m, &units, Some(Action::Ultimate), true);
        c.guard = Some(3);
        c.guard_until = 90;
        let commands = c.step(&Input {
            tick: 30,
            owner: &m,
            units: &units,
            ult_unlocked: true,
            spell_unlocked: true,
            barrier_unlocked: true,
            automatic: false,
            action: None,
            stark: Some(Stark::at(110000, 100000)),
            wall: &|_, _| false,
        });
        assert!(commands.contains(&Cmd::Damage {
            target: 2,
            amount: 68
        }));
        assert_eq!(c.next_stark, 120);
        e.x = 300000;
        units[1] = e;
        let commands = c.step(&Input {
            tick: 31,
            owner: &m,
            units: &units,
            ult_unlocked: true,
            spell_unlocked: true,
            barrier_unlocked: true,
            automatic: false,
            action: None,
            stark: Some(Stark::at(120000, 100000)),
            wall: &|_, _| false,
        });
        assert!(commands.contains(&Cmd::MoveStark {
            x: 121000,
            y: 100000
        }));
        assert!(!commands.iter().any(|a| matches!(a, Cmd::Damage { .. })));
    }
}

include!("frieren_stark.rs");
#[cfg(test)]
#[path = "frieren_stark_tests.rs"]
mod stark_tests;
