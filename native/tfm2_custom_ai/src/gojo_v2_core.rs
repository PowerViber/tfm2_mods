//! Host-free Gojo. All values use integer ticks / milli-points. Presentation never consumes RNG.
use crate::gojo_v2_data::*;
use crate::unified_theory_math::unit;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fighter {
    pub id: usize,
    pub team: usize,
    pub x: i64,
    pub y: i64,
    pub hp: i64,
    pub max_hp: i64,
    pub ad: i64,
    pub ap: i64,
    pub level: i64,
    pub champion: bool,
    pub visible: bool,
    pub targetable: bool,
    pub immune: bool,
    pub cc_immune: bool,
    pub farmable: bool,
    pub tower: bool,
    pub immovable: bool,
    pub cleansed: bool,
}
impl Fighter {
    pub fn pos(&self) -> (i64, i64) {
        (self.x, self.y)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Blue(usize),
    MaxBlue(usize),
    Red(usize),
    MaxRed(usize),
    Purple(usize),
    Heal,
    Blink(i64, i64),
    Domain,
    QuickDomain,
    OrbRed,
}
impl Action {
    pub fn index(self) -> usize {
        match self {
            Self::Blue(_) => 0,
            Self::MaxBlue(_) => 1,
            Self::Red(_) => 2,
            Self::MaxRed(_) => 3,
            Self::Purple(_) => 4,
            Self::Heal => 5,
            Self::Blink(..) => 6,
            Self::Domain | Self::QuickDomain => 7,
            Self::OrbRed => 3,
        }
    }
    pub fn target(self) -> Option<usize> {
        match self {
            Self::Blue(t) | Self::MaxBlue(t) | Self::Red(t) | Self::MaxRed(t) | Self::Purple(t) => {
                Some(t)
            }
            _ => None,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Cmd {
    SelfCost,
    Damage {
        target: usize,
        ad: i64,
        ap: i64,
        basic: bool,
        event: u64,
    },
    Move {
        target: usize,
        x: i64,
        y: i64,
    },
    Heal(i64),
    Infinity {
        active: bool,
        shield: i64,
    },
    Seal {
        target: usize,
        ticks: usize,
    },
    Guard {
        target: usize,
        ticks: usize,
        slow: bool,
    },
    Fx {
        kind: usize,
        x: i64,
        y: i64,
        ticks: usize,
    },
    Warning {
        x: i64,
        y: i64,
        r: i64,
        ticks: usize,
    },
    Lock(usize),
    DomainEnd,
    Hold {
        target: usize,
        ticks: usize,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cast {
    pub action: Action,
    pub end: usize,
    pub point: (i64, i64),
    pub origin: (i64, i64),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Shot {
    pub id: u64,
    pub kind: usize,
    pub primary: usize,
    pub start: (i64, i64),
    pub point: (i64, i64),
    pub dx: i64,
    pub dy: i64,
    pub remaining: i64,
    pub at: usize,
    pub max: bool,
    pub hits: Vec<usize>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Domain {
    pub centre: (i64, i64),
    pub start: usize,
    pub expand: usize,
    pub end: usize,
    pub guest: Option<usize>,
    pub trapped: Vec<(usize, i64, i64)>,
    pub slow: bool,
    pub empowered: bool,
    pub radius: i64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Motion {
    pub target: usize,
    pub start: (i64, i64),
    pub point: (i64, i64),
    pub begin: usize,
    pub end: usize,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Combo {
    pub target: usize,
    pub end: usize,
    pub punches: usize,
    pub cast: u64,
}
pub struct Input<'a> {
    pub tick: usize,
    pub me: &'a Fighter,
    pub all: &'a [Fighter],
    pub unlocked: [bool; 3],
    pub controlled: bool,
    pub bound: bool,
    pub automatic: bool,
    pub haste: i64,
    pub ult_haste: i64,
    pub action: Option<Action>,
    pub basics: &'a [(u64, usize)],
    pub champion_damage: bool,
    pub shield: Option<i64>,
    pub wall: &'a dyn Fn(i64, i64) -> bool,
    pub barrier: &'a dyn Fn(i64, i64) -> bool,
    pub sight: &'a dyn Fn(i64, i64) -> bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Orb {
    pub point: (i64, i64),
    pub end: usize,
    pub affected: Vec<usize>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fusion {
    pub point: (i64, i64),
    pub at: usize,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Spiral {
    pub target: usize,
    pub centre: (i64, i64),
    pub vector: (i64, i64),
    pub begin: usize,
    pub end: usize,
    pub turns: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Core {
    pub cooldown_fraction: [i64; 8],
    pub eyes: bool,
    pub orb: Option<Orb>,
    pub fusion: Option<Fusion>,
    pub spirals: Vec<Spiral>,
    pub ce: i64,
    pub strain: i64,
    pub power_until: usize,
    pub lock_until: usize,
    pub ready: [usize; 8],
    pub pending: Option<Cast>,
    pub shots: Vec<Shot>,
    pub motions: Vec<Motion>,
    pub haste_remainder: i64,
    pub ult_remainder: i64,
    pub domain: Option<Domain>,
    pub burnout: usize,
    pub overload: usize,
    pub amp_until: usize,
    pub flash_ready: usize,
    pub combo: Option<Combo>,
    pub punches: usize,
    pub punch_until: usize,
    pub shield: i64,
    pub infinity: bool,
    pub infinity_ready: usize,
    pub last_technique: usize,
    pub last_combat: usize,
    pub heal_at: usize,
    pub heal_left: usize,
    pub heal_total: i64,
    pub heal_delivered: i64,
    pub next_event: u64,
    pub last_basic: u64,
    pub last_tick: Option<usize>,
    pub alive: bool,
    pub reason: &'static str,
    pub upkeep_remainder: i64,
    pub recovery_remainder: i64,
    pub casts: [u64; 8],
    pub flashes: u64,
    pub requested: i64,
    pub ally_seal_ticks: u64,
}
impl Default for Core {
    fn default() -> Self {
        Self {
            cooldown_fraction: [0; 8],
            eyes: true,
            orb: None,
            fusion: None,
            spirals: vec![],
            ce: CE_MAX,
            strain: 0,
            power_until: 0,
            lock_until: 0,
            ready: [0; 8],
            pending: None,
            shots: vec![],
            motions: vec![],
            haste_remainder: 0,
            ult_remainder: 0,
            domain: None,
            burnout: 0,
            overload: 0,
            amp_until: 0,
            flash_ready: 0,
            combo: None,
            punches: 0,
            punch_until: 0,
            shield: 0,
            infinity: false,
            infinity_ready: 0,
            last_technique: 0,
            last_combat: 0,
            heal_at: 0,
            heal_left: 0,
            heal_total: 0,
            heal_delivered: 0,
            next_event: 1,
            last_basic: 0,
            last_tick: None,
            alive: false,
            reason: "Ready",
            upkeep_remainder: 0,
            recovery_remainder: 0,
            casts: [0; 8],
            flashes: 0,
            requested: 0,
            ally_seal_ticks: 0,
        }
    }
}
pub fn distance(a: (i64, i64), b: (i64, i64)) -> i128 {
    (a.0 - b.0) as i128 * (a.0 - b.0) as i128 + (a.1 - b.1) as i128 * (a.1 - b.1) as i128
}
pub fn within(a: (i64, i64), b: (i64, i64), r: i64) -> bool {
    distance(a, b) <= r as i128 * r as i128
}
fn valid(u: &Fighter, team: usize) -> bool {
    impactable(u, team) && u.visible
}
fn impactable(u: &Fighter, team: usize) -> bool {
    u.hp > 0 && u.team != team && u.targetable && !u.tower
}
fn segment(a: (i64, i64), b: (i64, i64), p: (i64, i64), r: i64) -> bool {
    let dx = (b.0 - a.0) as i128;
    let dy = (b.1 - a.1) as i128;
    let n = dx * dx + dy * dy;
    let d = if n == 0 {
        0
    } else {
        (((p.0 - a.0) as i128 * dx + (p.1 - a.1) as i128 * dy) * 10000 / n).clamp(0, 10000)
    };
    within(
        (a.0 + (dx * d / 10000) as i64, a.1 + (dy * d / 10000) as i64),
        p,
        r,
    )
}
impl Core {
    fn event(&mut self) -> u64 {
        let n = self.next_event;
        self.next_event += 1;
        n
    }
    pub(super) fn fx(out: &mut Vec<Cmd>, kind: usize, p: (i64, i64), ticks: usize) {
        out.push(Cmd::Fx {
            kind,
            x: p.0,
            y: p.1,
            ticks,
        })
    }
    pub(super) fn damage(
        &mut self,
        input: &Input,
        u: &Fighter,
        ad: i64,
        ap: i64,
        basic: bool,
        out: &mut Vec<Cmd>,
    ) {
        let scale = if input.tick < self.amp_until {
            AMPLIFICATION
        } else {
            100
        };
        let eye_scale = if self.eyes { EYE_DAMAGE_PERCENT } else { 100 };
        let ad = ad * scale * eye_scale / 10000;
        let ap = ap * scale * eye_scale / 10000;
        let event = self.event();
        self.requested += ad + ap;
        out.push(Cmd::Damage {
            target: u.id,
            ad,
            ap,
            basic,
            event,
        });
        if u.champion && !u.immune {
            self.last_combat = input.tick;
        }
    }
    fn legal(&self, input: &Input, p: (i64, i64)) -> bool {
        p.0 >= 0
            && p.1 >= 0
            && p.0 < 960000
            && p.1 < 960000
            && !(input.wall)(p.0, p.1)
            && !(input.barrier)(p.0, p.1)
    }
    pub(super) fn route(&self, input: &Input, a: (i64, i64), b: (i64, i64)) -> bool {
        let n = ((b.0 - a.0).abs().max((b.1 - a.1).abs()) / 2000 + 1).clamp(1, 512);
        (0..=n).all(|i| {
            self.legal(
                input,
                (a.0 + (b.0 - a.0) * i / n, a.1 + (b.1 - a.1) * i / n),
            )
        })
    }
    fn displace(
        &mut self,
        input: &Input,
        u: &Fighter,
        goal: (i64, i64),
        amount: i64,
        ticks: usize,
        out: &mut Vec<Cmd>,
    ) {
        if goal == u.pos() || u.immune || u.cc_immune || u.immovable {
            return;
        }
        let d = unit(goal.0 - u.x, goal.1 - u.y);
        let p = if within(u.pos(), goal, amount) {
            goal
        } else {
            (u.x + d.0 * amount / 10000, u.y + d.1 * amount / 10000)
        };
        let mut last = u.pos();
        for n in 1..=16 {
            let q = (u.x + (p.0 - u.x) * n / 16, u.y + (p.1 - u.y) * n / 16);
            if !self.legal(input, q) {
                break;
            }
            last = q;
        }
        if last != u.pos() {
            self.motions.retain(|m| m.target != u.id);
            self.motions.push(Motion {
                target: u.id,
                start: u.pos(),
                point: last,
                begin: input.tick,
                end: input.tick + ticks,
            });
            out.push(Cmd::Hold {
                target: u.id,
                ticks,
            });
        }
    }
    fn close_domain(&mut self, t: usize, out: &mut Vec<Cmd>) {
        if self.domain.take().is_some() {
            self.burnout = self.burnout.max(t + 180);
            if self.pending.as_ref().is_some_and(|c| c.action.index() != 5) {
                self.pending = None;
                self.lock_until = t;
            }
            out.push(Cmd::DomainEnd);
        }
    }
    pub fn died(&mut self, t: usize) -> Vec<Cmd> {
        let mut out = vec![];
        self.close_domain(t, &mut out);
        self.pending = None;
        self.shots.clear();
        self.orb = None;
        self.fusion = None;
        self.spirals.clear();
        self.motions.clear();
        self.combo = None;
        self.punches = 0;
        self.amp_until = 0;
        self.heal_left = 0;
        self.power_until = 0;
        self.lock_until = 0;
        self.infinity = false;
        self.alive = false;
        out.push(Cmd::Infinity {
            active: false,
            shield: 0,
        });
        out
    }
    fn arm(&mut self, input: &Input, target: usize, blue: bool, cast: u64) {
        if blue {
            self.punches = 2;
            self.punch_until = input.tick + COMBO_TICKS as usize;
        }
        if input.tick >= self.flash_ready && self.combo.is_none() {
            self.combo = Some(Combo {
                target,
                end: input.tick + COMBO_TICKS as usize,
                punches: 0,
                cast,
            });
        }
    }
    fn melee(&mut self, input: &Input, serial: u64, target: usize, out: &mut Vec<Cmd>) {
        if serial <= self.last_basic {
            return;
        }
        self.last_basic = serial;
        if input.controlled || self.pending.is_some() || input.tick < self.lock_until {
            return;
        }
        let Some(u) = input.all.iter().find(|u| {
            u.id == target
                && u.hp > 0
                && u.team != input.me.team
                && u.visible
                && u.targetable
                && within(input.me.pos(), u.pos(), 23000)
        }) else {
            return;
        };
        let eligible = u.champion && !u.immune;
        let combo = eligible
            && self
                .combo
                .as_ref()
                .is_some_and(|c| c.target == target && input.tick < c.end)
            && input.tick >= self.flash_ready;
        let enhanced = self.punches > 0
            && input.tick < self.punch_until
            && input.tick >= self.burnout
            && !u.tower;
        let cost = if self.eyes { 100 } else { 1000 };
        let powered = (combo || enhanced) && self.ce >= cost;
        let mut ap = input.me.ap * 35 / 100;
        let mut flash = false;
        if powered {
            self.ce -= cost;
            if self.eyes {
                self.strain = (self.strain + 1000).min(STRAIN_MAX);
                self.last_technique = input.tick;
            }
            if enhanced {
                ap += PUNCH_DAMAGE[0] + input.me.ap * PUNCH_DAMAGE[1] / 100;
                self.punches -= 1;
                self.displace(input, u, input.me.pos(), 4000, 4, out);
                Self::fx(out, 1, u.pos(), 12);
            }
            if combo {
                let c = self.combo.as_mut().unwrap();
                c.punches += 1;
                if c.punches == 2 {
                    flash = true;
                    ap += FLASH_DAMAGE[0] + input.me.ap * FLASH_DAMAGE[1] / 100;
                }
            }
        }
        self.damage(input, u, input.me.ad, ap, true, out);
        if flash {
            self.combo = None;
            self.flash_ready = input.tick + FLASH_COOLDOWN as usize;
            self.amp_until = input.tick + AMPLIFICATION_TICKS as usize;
            self.strain = (self.strain - 10000).max(0);
            self.flashes += 1;
            Self::fx(out, 2, u.pos(), 18);
        }
    }
    fn available(&self, input: &Input, a: Action) -> bool {
        let k = a.index();
        let strain = if a == Action::QuickDomain {
            50000
        } else {
            STRAINS[k]
        };
        let eye_required = matches!(k, 1 | 3 | 4 | 5 | 7);
        if self.pending.is_some()
            || input.controlled
            || input.tick < self.lock_until
            || input.tick < self.ready[k]
            || self.ce < self.cost(k)
            || (k != 5 && input.tick < self.burnout)
            || (eye_required && (!self.eyes || self.strain + strain >= STRAIN_MAX))
        {
            return false;
        }
        if matches!(k, 1 | 3) && input.tick >= self.amp_until {
            return false;
        }
        if k == 1 && self.orb.is_some() {
            return false;
        }
        if a == Action::OrbRed && self.orb.is_none() {
            return false;
        }
        match k {
            0 | 1 => input.unlocked[0],
            2 | 3 => input.unlocked[1],
            4 => {
                input.unlocked.iter().all(|x| *x)
                    && input.tick >= self.ready[0]
                    && input.tick >= self.ready[2]
            }
            5 | 6 => input.unlocked[1],
            7 => input.unlocked[2] && self.domain.is_none(),
            _ => false,
        }
    }
    fn cost(&self, k: usize) -> i64 {
        if self.eyes {
            COSTS[k] * EYE_CE_PERCENT / 100
        } else {
            COSTS[k]
        }
    }
    fn begin(&mut self, input: &Input, a: Action, out: &mut Vec<Cmd>) {
        if !self.available(input, a) {
            self.reason = if !self.eyes && matches!(a.index(), 1 | 3 | 4 | 5 | 7) {
                "Six Eyes exhausted"
            } else if matches!(a.index(), 1 | 3) && input.tick >= self.amp_until {
                "Maximum output requires Black Flash"
            } else if a.index() == 1 && self.orb.is_some() {
                "An owned Blue orb is already active"
            } else if a == Action::OrbRed && self.orb.is_none() {
                "No owned Blue orb"
            } else if matches!(a.index(), 1 | 3 | 4 | 5 | 7)
                && self.strain + STRAINS[a.index()] >= STRAIN_MAX
            {
                "Insufficient eye strain headroom"
            } else {
                "Locked, recovering, controlled or insufficient CE"
            };
            return;
        }
        let k = a.index();
        let mut point = input.me.pos();
        if let Some(target) = a.target() {
            let Some(u) = input
                .all
                .iter()
                .find(|u| u.id == target && valid(u, input.me.team))
            else {
                self.reason = "Target unavailable";
                return;
            };
            let range = if k == 4 {
                400000
            } else if k < 2 {
                80000
            } else {
                70000
            };
            if !within(input.me.pos(), u.pos(), range) {
                self.reason = "Out of range";
                return;
            }
            point = u.pos();
        }
        if let Action::Blink(x, y) = a {
            point = (x, y);
            if input.bound
                || !(input.sight)(point.0, point.1)
                || !within(input.me.pos(), point, 32000)
                || !self.route(input, input.me.pos(), point)
            {
                self.reason = "Illegal spatial step";
                return;
            }
        }
        if k == 5 && input.me.hp >= input.me.max_hp {
            self.reason = "Already healthy";
            return;
        }
        if a == Action::OrbRed {
            point = self.orb.as_ref().unwrap().point;
            if !within(input.me.pos(), point, 70000) || !self.route(input, input.me.pos(), point) {
                self.reason = "Orb out of range or blocked";
                return;
            }
        }
        self.ce -= self.cost(k);
        if self.eyes {
            self.strain = (self.strain
                + if a == Action::QuickDomain {
                    50000
                } else {
                    STRAINS[k]
                })
            .min(STRAIN_MAX);
            self.last_technique = input.tick;
        }
        self.casts[k] += 1;
        self.reason = "Committed";
        self.cooldown_fraction[k] = 0;
        self.ready[k] = input.tick + COOLDOWNS[k] as usize;
        if k < 2 {
            self.cooldown_fraction[0] = 0;
            self.cooldown_fraction[1] = 0;
            self.ready[0] = self.ready[k];
            self.ready[1] = self.ready[k]
        }
        if k == 2 || k == 3 {
            self.cooldown_fraction[2] = 0;
            self.cooldown_fraction[3] = 0;
            self.ready[2] = self.ready[k];
            self.ready[3] = self.ready[k]
        }
        if k == 4 {
            self.cooldown_fraction[..4].fill(0);
            self.ready[0] = input.tick + 600;
            self.ready[1] = self.ready[0];
            self.ready[2] = input.tick + 540;
            self.ready[3] = self.ready[2];
        }
        if k == 7 {
            let slow = a == Action::Domain;
            let timing = if slow { DOMAIN_SLOW } else { DOMAIN_FAST };
            let radius = if slow { DOMAIN_RADIUS } else { QUICK_RADIUS };
            let guest = input
                .all
                .iter()
                .filter(|u| {
                    u.id != input.me.id
                        && u.team == input.me.team
                        && u.champion
                        && u.hp > 0
                        && within(input.me.pos(), u.pos(), radius)
                })
                .min_by_key(|u| (distance(input.me.pos(), u.pos()), u.id))
                .map(|u| u.id);
            self.domain = Some(Domain {
                centre: point,
                start: input.tick,
                expand: timing[0] as usize,
                end: input.tick + (timing[0] + timing[1]) as usize,
                guest,
                trapped: vec![],
                slow,
                empowered: false,
                radius,
            });
            self.lock_until = input.tick + timing[2] as usize;
            out.push(Cmd::Lock(timing[2] as usize));
            out.push(Cmd::Warning {
                x: point.0,
                y: point.1,
                r: radius,
                ticks: timing[0] as usize,
            });
            Self::fx(out, 10, point, (timing[0] + timing[1]) as usize);
            if let Some(g) = guest {
                if let Some(u) = input.all.iter().find(|u| u.id == g) {
                    Self::fx(out, 11, u.pos(), timing[0] as usize);
                }
            }
        } else {
            let ticks = PREPARATIONS[k] as usize;
            self.lock_until = input.tick + ticks;
            self.pending = Some(Cast {
                action: a,
                end: input.tick + ticks,
                point,
                origin: input.me.pos(),
            });
            out.push(Cmd::Lock(ticks));
            Self::fx(
                out,
                if k == 5 {
                    3
                } else if k == 6 {
                    4
                } else if k < 2 {
                    5
                } else if k < 4 {
                    6
                } else {
                    7
                },
                input.me.pos(),
                ticks,
            );
            out.push(Cmd::Warning {
                x: point.0,
                y: point.1,
                r: if k == 4 {
                    26000
                } else if k == 1 {
                    36000
                } else if k < 2 {
                    32000
                } else if k == 3 {
                    26000
                } else if k < 4 {
                    20000
                } else {
                    3000
                },
                ticks: ticks + if k < 2 { 12 } else { 0 },
            });
            if k == 4 {
                let (dx, dy) = unit(point.0 - input.me.x, point.1 - input.me.y);
                // Eight overlapping circles describe the committed firing line.
                // They are bounded warning geometry, never damage or extra search.
                for n in 0..8 {
                    let distance = 25000 + 50000 * n;
                    out.push(Cmd::Warning {
                        x: input.me.x + dx * distance / 10000,
                        y: input.me.y + dy * distance / 10000,
                        r: 26000,
                        ticks,
                    });
                }
            }
        }
    }
    fn release(&mut self, input: &Input, cast: Cast, out: &mut Vec<Cmd>) {
        let k = cast.action.index();
        if let Some(id) = cast.action.target() {
            if !input
                .all
                .iter()
                .any(|u| u.id == id && valid(u, input.me.team))
            {
                self.reason = "Preparation lost target";
                return;
            }
        }
        if k == 5 {
            self.heal_total = HEAL_DAMAGE[0] + input.me.ap * HEAL_DAMAGE[1] / 100;
            self.heal_left = 4;
            self.heal_delivered = 0;
            self.heal_at = input.tick + 15;
            return;
        }
        if k == 6 {
            if !input.bound
                && (input.sight)(cast.point.0, cast.point.1)
                && self.route(input, input.me.pos(), cast.point)
            {
                out.push(Cmd::Move {
                    target: input.me.id,
                    x: cast.point.0,
                    y: cast.point.1,
                });
                Self::fx(out, 4, cast.point, 18)
            } else {
                self.reason = "Spatial step invalidated"
            }
            return;
        }
        let (dx, dy) = unit(cast.point.0 - cast.origin.0, cast.point.1 - cast.origin.1);
        let id = self.event();
        self.shots.push(Shot {
            id,
            kind: k,
            primary: cast.action.target().unwrap_or(usize::MAX),
            start: cast.origin,
            point: cast.point,
            dx,
            dy,
            remaining: if k == 4 { 400000 } else { 70000 },
            at: if k < 2 { input.tick + 12 } else { input.tick },
            max: k == 1 || k == 3,
            hits: vec![],
        });
        if k == 4 {
            let goal = (
                input.me.x - dx * 36000 / 10000,
                input.me.y - dy * 36000 / 10000,
            );
            self.displace(input, input.me, goal, 36000, 12, out);
        }
    }
    fn projectiles(&mut self, input: &Input, out: &mut Vec<Cmd>) {
        let mut remaining = vec![];
        for mut s in std::mem::take(&mut self.shots) {
            let k = s.kind;
            if k < 2 {
                if input.tick < s.at {
                    let elapsed = (12 - (s.at - input.tick).min(12)) as i64;
                    Self::fx(
                        out,
                        13,
                        (
                            s.start.0 + (s.point.0 - s.start.0) * elapsed / 12,
                            s.start.1 + (s.point.1 - s.start.1) * elapsed / 12,
                        ),
                        2,
                    );
                    remaining.push(s);
                    continue;
                }
                let radius = if s.max { 36000 } else { 32000 };
                for u in input
                    .all
                    .iter()
                    .filter(|u| impactable(u, input.me.team) && within(s.point, u.pos(), radius))
                {
                    let inner = within(s.point, u.pos(), 8000);
                    let ap = (if inner {
                        BLUE_DAMAGE[0] + input.me.ap * BLUE_DAMAGE[1] / 100
                    } else {
                        0
                    }) + if input.me.level >= 3 {
                        BLUE_DAMAGE[3]
                    } else {
                        BLUE_DAMAGE[2]
                    } + input.me.ap * BLUE_DAMAGE[4] / 100;
                    self.damage(
                        input,
                        u,
                        0,
                        ap * if s.max { 135 } else { 100 } / 100,
                        false,
                        out,
                    );
                    self.spiral(input, u, s.point, s.max, out);
                    if u.id == s.primary && u.champion && u.visible && !u.immune {
                        self.arm(input, u.id, true, s.id)
                    }
                }
                if s.max {
                    self.orb = Some(Orb {
                        point: s.point,
                        end: input.tick + ORB_TICKS as usize,
                        affected: input
                            .all
                            .iter()
                            .filter(|u| {
                                impactable(u, input.me.team) && within(s.point, u.pos(), radius)
                            })
                            .map(|u| u.id)
                            .collect(),
                    });
                }
                Self::fx(out, 8, s.point, 20);
            } else {
                let speed = if k == 4 { 4200 } else { 8000 };
                let n = s.remaining.min(speed);
                let next = (s.start.0 + s.dx * n / 10000, s.start.1 + s.dy * n / 10000);
                let r = if k == 4 { 26000 } else { 11000 };
                if input.tick % 6 == 0 {
                    out.push(Cmd::Warning {
                        x: next.0 + s.dx * speed * 2 / 10000,
                        y: next.1 + s.dy * speed * 2 / 10000,
                        r,
                        ticks: 6,
                    });
                }
                let mut targets: Vec<_> = input
                    .all
                    .iter()
                    .filter(|u| {
                        impactable(u, input.me.team)
                            && !s.hits.contains(&u.id)
                            && segment(s.start, next, u.pos(), r)
                    })
                    .collect();
                targets.sort_by_key(|u| (entry(s.start, next, u.pos(), r), u.id));
                if k == 3 {
                    if let Some(o) = &self.orb {
                        let orb_entry = entry(s.start, next, o.point, 4000);
                        if input.tick < o.end
                            && segment(s.start, next, o.point, 4000)
                            && targets
                                .first()
                                .is_none_or(|u| entry(s.start, next, u.pos(), r) > orb_entry)
                        {
                            let point = o.point;
                            self.orb = None;
                            self.fusion = Some(Fusion {
                                point,
                                at: input.tick + NUKE_WARNING as usize,
                            });
                            out.push(Cmd::Warning {
                                x: point.0,
                                y: point.1,
                                r: NUKE_RADIUS,
                                ticks: NUKE_WARNING as usize,
                            });
                            Self::fx(out, 15, point, NUKE_WARNING as usize);
                            continue;
                        }
                    }
                }
                if k == 4 {
                    for u in targets {
                        self.damage(
                            input,
                            u,
                            0,
                            PURPLE_DAMAGE[0] + input.me.ap * PURPLE_DAMAGE[1] / 100,
                            false,
                            out,
                        );
                        self.displace(input, u, (u.x + s.dx * 10, u.y + s.dy * 10), 45000, 10, out);
                        s.hits.push(u.id);
                    }
                } else if let Some(u) = targets.first() {
                    self.damage(
                        input,
                        u,
                        0,
                        (RED_DAMAGE[0] + input.me.ap * RED_DAMAGE[1] / 100)
                            * if s.max { 135 } else { 100 }
                            / 100,
                        false,
                        out,
                    );
                    self.displace(input, u, (u.x + s.dx * 10, u.y + s.dy * 10), 54000, 9, out);
                    s.remaining = 0;
                    s.point = u.pos();
                    if u.id == s.primary && u.champion && u.visible && !u.immune {
                        self.arm(input, u.id, false, s.id)
                    }
                }
                s.start = next;
                if s.remaining > 0 {
                    s.remaining -= n;
                    s.point = next
                }
                Self::fx(out, if k == 4 { 7 } else { 12 }, s.start, 2);
                if s.remaining <= 0 {
                    if k != 4 {
                        for u in input.all.iter().filter(|u| {
                            impactable(u, input.me.team)
                                && within(s.point, u.pos(), if s.max { 26000 } else { 20000 })
                        }) {
                            self.damage(
                                input,
                                u,
                                0,
                                (RED_DAMAGE[2] + input.me.ap * RED_DAMAGE[3] / 100)
                                    * if s.max { 135 } else { 100 }
                                    / 100,
                                false,
                                out,
                            );
                            self.displace(
                                input,
                                u,
                                (u.x + s.dx * 10, u.y + s.dy * 10),
                                24000,
                                8,
                                out,
                            );
                            if u.id == s.primary && u.champion && u.visible && !u.immune {
                                self.arm(input, u.id, false, s.id)
                            }
                        }
                        Self::fx(out, 9, s.point, 18);
                    }
                } else {
                    remaining.push(s)
                }
            }
        }
        self.shots = remaining;
    }
    fn motion_tick(&mut self, input: &Input, out: &mut Vec<Cmd>) {
        let mut keep = vec![];
        for m in std::mem::take(&mut self.motions) {
            let Some(u) = input.all.iter().find(|u| {
                u.id == m.target
                    && u.hp > 0
                    && !u.immune
                    && !u.cc_immune
                    && !u.cleansed
                    && !u.immovable
            }) else {
                continue;
            };
            let n = (input.tick - m.begin).min(m.end - m.begin) as i64;
            let duration = (m.end - m.begin).max(1) as i64;
            let p = (
                m.start.0 + (m.point.0 - m.start.0) * n / duration,
                m.start.1 + (m.point.1 - m.start.1) * n / duration,
            );
            if self.route(input, u.pos(), p) {
                out.push(Cmd::Move {
                    target: u.id,
                    x: p.0,
                    y: p.1,
                });
                if input.tick < m.end {
                    keep.push(m)
                }
            }
        }
        self.motions = keep;
    }
    fn domain_tick(&mut self, input: &Input, out: &mut Vec<Cmd>) {
        let Some(mut d) = self.domain.take() else {
            return;
        };
        if input.tick >= d.end || (!d.empowered && input.controlled) {
            self.domain = Some(d);
            self.close_domain(input.tick, out);
            return;
        }
        if input.tick < d.start + d.expand {
            self.domain = Some(d);
            return;
        }
        d.empowered = true;
        for u in input.all.iter().filter(|u| u.hp > 0 && u.champion) {
            if u.id == input.me.id || Some(u.id) == d.guest {
                out.push(Cmd::Guard {
                    target: u.id,
                    ticks: (d.end - input.tick).min(2),
                    slow: d.slow,
                });
                continue;
            }
            let caught = d.trapped.iter().any(|v| v.0 == u.id);
            if !caught && within(d.centre, u.pos(), d.radius) {
                d.trapped.push((u.id, u.x, u.y));
                out.push(Cmd::Seal {
                    target: u.id,
                    ticks: if d.slow {
                        (d.end - input.tick).min(7)
                    } else {
                        60
                    },
                });
            }
            if d.slow {
                if let Some((_, x, y)) = d.trapped.iter().find(|v| v.0 == u.id) {
                    if input.tick % 6 == 0 {
                        out.push(Cmd::Seal {
                            target: u.id,
                            ticks: (d.end - input.tick).min(7),
                        });
                    }
                    if !within((*x, *y), u.pos(), 400) {
                        out.push(Cmd::Move {
                            target: u.id,
                            x: *x,
                            y: *y,
                        });
                    }
                    if u.team == input.me.team {
                        self.ally_seal_ticks += 1;
                    }
                } else if within(d.centre, u.pos(), d.radius + 1500) {
                    let (dx, dy) = unit(u.x - d.centre.0, u.y - d.centre.1);
                    out.push(Cmd::Move {
                        target: u.id,
                        x: d.centre.0 + dx * (d.radius + 1500) / 10000,
                        y: d.centre.1 + dy * (d.radius + 1500) / 10000,
                    });
                }
            }
        }
        self.domain = Some(d);
    }
    fn offensive(&self, input: &Input, a: Action) -> bool {
        let reserve = if input.me.hp * 100 < input.me.max_hp * 55 {
            20000
        } else {
            8000
        };
        self.available(input, a)
            && self.ce >= self.cost(a.index()) + reserve
            && (!self.eyes || self.strain + STRAINS[a.index()] < STRAIN_MAX)
    }
    fn choose(&self, input: &Input) -> Option<Action> {
        let m = input.me;
        let mut enemies: Vec<_> = input
            .all
            .iter()
            .filter(|u| u.champion && valid(u, m.team))
            .collect();
        enemies.sort_by_key(|u| (distance(m.pos(), u.pos()), u.id));
        let danger = enemies
            .iter()
            .filter(|u| within(m.pos(), u.pos(), 40000))
            .count();
        // A long preparation needs breathing room or an already sealed target.
        let safe_max = |e: &&Fighter| {
            !within(m.pos(), e.pos(), 30000)
                || self
                    .domain
                    .as_ref()
                    .is_some_and(|d| d.slow && d.empowered && d.trapped.iter().any(|u| u.0 == e.id))
        };
        if danger >= 2 && m.hp * 100 < m.max_hp * 55 && self.available(input, Action::Blink(0, 0)) {
            if let Some(e) = enemies.first() {
                let (dx, dy) = unit(m.x - e.x, m.y - e.y);
                let p = (m.x + dx * 32000 / 10000, m.y + dy * 32000 / 10000);
                if (input.sight)(p.0, p.1) && self.route(input, m.pos(), p) {
                    return Some(Action::Blink(p.0, p.1));
                }
            }
        }
        if m.hp * 100 <= m.max_hp * if danger >= 2 { 60 } else { 40 }
            && self.available(input, Action::Heal)
        {
            return Some(Action::Heal);
        }
        if m.hp * 100 < m.max_hp * 30 && self.available(input, Action::Blink(0, 0)) {
            if let Some(e) = enemies.first() {
                let (dx, dy) = unit(m.x - e.x, m.y - e.y);
                let a = Action::Blink(m.x + dx * 32000 / 10000, m.y + dy * 32000 / 10000);
                if let Action::Blink(x, y) = a {
                    if self.route(input, m.pos(), (x, y)) {
                        return Some(a);
                    }
                }
            }
        }
        if let Some(c) = &self.combo {
            if input.tick < c.end
                && enemies
                    .iter()
                    .any(|u| u.id == c.target && within(m.pos(), u.pos(), 23000))
            {
                return None;
            }
        }
        if let Some(o) = &self.orb {
            if m.hp * 2 >= m.max_hp
                && enemies
                    .iter()
                    .filter(|u| within(o.point, u.pos(), NUKE_RADIUS))
                    .count()
                    >= 2
                && self.offensive(input, Action::OrbRed)
                && within(m.pos(), o.point, 70000)
                && self.route(input, m.pos(), o.point)
            {
                return Some(Action::OrbRed);
            }
        }
        let caught: Vec<_> = enemies
            .iter()
            .filter(|u| within(m.pos(), u.pos(), DOMAIN_RADIUS))
            .collect();
        let allies = input
            .all
            .iter()
            .filter(|u| {
                u.id != m.id
                    && u.champion
                    && u.team == m.team
                    && u.hp > 0
                    && within(m.pos(), u.pos(), DOMAIN_RADIUS)
            })
            .count();
        if caught.len() >= 2 && allies <= 1 && self.offensive(input, Action::Domain) {
            return Some(Action::Domain);
        }
        let quick_caught = enemies
            .iter()
            .filter(|u| within(m.pos(), u.pos(), QUICK_RADIUS))
            .count();
        let quick_allies = input
            .all
            .iter()
            .filter(|u| {
                u.id != m.id
                    && u.champion
                    && u.team == m.team
                    && u.hp > 0
                    && within(m.pos(), u.pos(), QUICK_RADIUS)
            })
            .count();
        if quick_caught >= 2 && quick_allies <= 1 && self.offensive(input, Action::QuickDomain) {
            return Some(Action::QuickDomain);
        }
        if let Some(e) = enemies.first() {
            if enemies
                .iter()
                .filter(|u| segment(m.pos(), e.pos(), u.pos(), 26000))
                .count()
                >= 2
                && self.offensive(input, Action::Purple(e.id))
            {
                return Some(Action::Purple(e.id));
            }
            if within(m.pos(), e.pos(), 80000) {
                let crowd = enemies
                    .iter()
                    .filter(|u| within(e.pos(), u.pos(), 32000))
                    .count();
                let a = if crowd >= 2 && safe_max(e) && self.ce >= 35000 && self.strain < 140000 {
                    Action::MaxBlue(e.id)
                } else {
                    Action::Blue(e.id)
                };
                if self.offensive(input, a) {
                    return Some(a);
                }
                let normal = if a.index() < 2 {
                    Action::Blue(e.id)
                } else {
                    Action::Red(e.id)
                };
                if self.offensive(input, normal) {
                    return Some(normal);
                }
            }
            if within(m.pos(), e.pos(), 70000) {
                let a = if enemies
                    .iter()
                    .filter(|u| within(e.pos(), u.pos(), 20000))
                    .count()
                    >= 2
                    && safe_max(e)
                    && self.ce >= 40000
                    && self.strain < 140000
                {
                    Action::MaxRed(e.id)
                } else {
                    Action::Red(e.id)
                };
                if self.offensive(input, a) {
                    return Some(a);
                }
                let normal = if a.index() < 2 {
                    Action::Blue(e.id)
                } else {
                    Action::Red(e.id)
                };
                if self.offensive(input, normal) {
                    return Some(normal);
                }
            }
            if self.punches > 0
                && self.punch_until > input.tick
                && within(m.pos(), e.pos(), 55000)
                && !within(m.pos(), e.pos(), 23000)
                && self.ce >= 20000
                && self.available(input, Action::Blink(0, 0))
            {
                let (dx, dy) = unit(e.x - m.x, e.y - m.y);
                let p = (e.x - dx * 18000 / 10000, e.y - dy * 18000 / 10000);
                if self.route(input, m.pos(), p) && within(m.pos(), p, 32000) {
                    return Some(Action::Blink(p.0, p.1));
                }
            }
        }
        if enemies.iter().any(|u| within(m.pos(), u.pos(), 140000)) || m.hp * 100 < m.max_hp * 55 {
            return None;
        }
        input
            .all
            .iter()
            .filter(|u| u.farmable && valid(u, m.team) && within(m.pos(), u.pos(), 70000))
            .min_by_key(|u| (distance(m.pos(), u.pos()), u.id))
            .and_then(|u| {
                [Action::Blue(u.id), Action::Red(u.id)]
                    .into_iter()
                    .find(|a| self.ce >= 40000 && self.available(input, *a))
            })
    }
    pub fn step(&mut self, input: &Input) -> Vec<Cmd> {
        let t = input.tick;
        if self.last_tick.is_some_and(|n| t <= n) {
            return vec![];
        }
        let elapsed = self.last_tick.map_or(0, |n| t - n);
        self.last_tick = Some(t);
        if input.me.hp <= 0 {
            return if self.alive { self.died(t) } else { vec![] };
        }
        let mut out = vec![];
        if !self.alive {
            self.alive = true;
            self.ce = CE_MAX;
            self.strain = 0;
            self.overload = 0;
            self.eyes = true;
            self.burnout = 0;
            self.shield = INFINITY_SHIELD + input.me.ap * INFINITY_AP / 100;
            self.infinity = false;
        }
        let previous = self.last_tick.unwrap_or(t).saturating_sub(elapsed);
        let speed = if self.eyes { 2 } else { 1 };
        // Ready timestamps count down normally with time. Add only the extra progress.
        for k in 0..8 {
            let haste = if k == 7 { input.ult_haste } else { input.haste };
            let rate = (100 + haste.clamp(-90, 500)) * speed;
            let old = self.ready[k].saturating_sub(previous) as i64;
            self.cooldown_fraction[k] += elapsed as i64 * rate;
            let progress = self.cooldown_fraction[k] / 100;
            self.cooldown_fraction[k] %= 100;
            self.ready[k] = t + (old - progress).max(0) as usize;
        }
        if input.champion_damage {
            self.last_combat = t;
        }
        let from = t.saturating_sub(elapsed);
        let rested = if input.champion_damage {
            0
        } else {
            t.saturating_sub(from.max(self.last_combat + 239))
                .min(elapsed)
        };
        self.ce = (self.ce + (elapsed - rested) as i64 * REGEN_COMBAT + rested as i64 * REGEN_REST)
            .min(CE_MAX);
        let recovered = if self.eyes {
            t.saturating_sub(from.max(self.last_technique + 89))
                .min(elapsed)
        } else {
            elapsed
        };
        self.recovery_remainder += recovered as i64
            * if self.eyes {
                STRAIN_RECOVERY_PER_SECOND
            } else {
                EXHAUSTED_RECOVERY_PER_SECOND
            };
        self.strain = (self.strain - self.recovery_remainder / 60).max(0);
        self.recovery_remainder %= 60;
        if !self.eyes && self.strain <= EYE_RESTORE {
            self.eyes = true;
            self.overload = 0;
            self.reason = "Six Eyes restored";
        }
        if self.combo.as_ref().is_some_and(|c| {
            t >= c.end
                || !input
                    .all
                    .iter()
                    .any(|u| u.id == c.target && u.hp > 0 && u.visible && u.targetable)
        }) {
            self.combo = None
        }
        if t >= self.punch_until {
            self.punches = 0
        }
        if let Some(shield) = input.shield {
            let before = self.shield;
            self.shield = self.shield.min(shield);
            if before > 0 && self.shield <= 0 {
                self.infinity_ready = t + INFINITY_RECOVERY as usize;
                out.push(Cmd::Infinity {
                    active: false,
                    shield: 0,
                });
            }
        }
        if self.infinity && self.eyes && t >= self.burnout {
            self.upkeep_remainder += elapsed as i64 * if self.eyes { 100 } else { 1000 };
            self.ce = (self.ce - self.upkeep_remainder / 60).max(0);
            self.upkeep_remainder %= 60;
        }
        let active = self.eyes
            && t >= self.burnout
            && t >= self.infinity_ready
            && self.ce >= if self.infinity { 1 } else { 500 };
        if active && self.shield <= 0 {
            if self.ce >= 500 {
                self.ce -= 500;
                self.shield = INFINITY_SHIELD + input.me.ap * INFINITY_AP / 100;
            }
        }
        let active = active && self.shield > 0;
        if active != self.infinity {
            self.infinity = active;
            out.push(Cmd::Infinity {
                active,
                shield: self.shield,
            })
        }
        if input.controlled && self.pending.take().is_some() {
            self.reason = "Interrupted"
        }
        if let Some(c) = &self.pending {
            if input.me.pos() != c.origin {
                out.push(Cmd::Move {
                    target: input.me.id,
                    x: c.origin.0,
                    y: c.origin.1,
                });
            }
        }
        if self.pending.as_ref().is_some_and(|c| t >= c.end) {
            let c = self.pending.take().unwrap();
            self.release(input, c, &mut out)
        }
        if self.heal_left > 0 && t >= self.heal_at {
            let done = 5 - self.heal_left;
            let n = self.heal_total * done as i64 / 4 - self.heal_delivered;
            self.heal_delivered += n;
            self.heal_left -= 1;
            self.heal_at += 15;
            out.push(Cmd::Heal(n));
            Self::fx(&mut out, 3, input.me.pos(), 15)
        }
        self.motion_tick(input, &mut out);
        self.spiral_tick(input, &mut out);
        self.projectiles(input, &mut out);
        self.field_tick(input, &mut out);
        self.domain_tick(input, &mut out);
        for &(serial, target) in input.basics {
            self.melee(input, serial, target, &mut out)
        }
        if let Some(a) = input.action.or_else(|| {
            if input.automatic {
                self.choose(input)
            } else {
                None
            }
        }) {
            self.begin(input, a, &mut out)
        }
        if self.strain >= STRAIN_MAX && self.eyes {
            self.eyes = false;
            self.overload = t.max(1);
            if self
                .pending
                .as_ref()
                .is_some_and(|c| matches!(c.action.index(), 1 | 3 | 4 | 5 | 7))
            {
                self.pending = None;
                self.lock_until = t;
            }
            self.close_domain(t, &mut out);
            self.infinity = false;
            out.push(Cmd::Infinity {
                active: false,
                shield: self.shield,
            });
            self.reason = "Six Eyes exhausted: recover to 50 strain";
        }
        if (t < self.burnout || self.ce == 0) && self.infinity {
            self.infinity = false;
            out.push(Cmd::Infinity {
                active: false,
                shield: self.shield,
            });
        }
        out
    }
}

fn entry(a: (i64, i64), b: (i64, i64), p: (i64, i64), r: i64) -> i64 {
    let dx = (b.0 - a.0) as i128;
    let dy = (b.1 - a.1) as i128;
    let n = (dx * dx + dy * dy).max(1);
    let closest = ((((p.0 - a.0) as i128 * dx + (p.1 - a.1) as i128 * dy) * 1000000 / n)
        .clamp(0, 1000000)) as i64;
    let mut lo = 0;
    let mut hi = closest;
    for _ in 0..20 {
        let m = (lo + hi) / 2;
        let q = (
            a.0 + (dx * m as i128 / 1000000) as i64,
            a.1 + (dy * m as i128 / 1000000) as i64,
        );
        if within(q, p, r) {
            hi = m;
        } else {
            lo = m + 1;
        }
    }
    hi
}
