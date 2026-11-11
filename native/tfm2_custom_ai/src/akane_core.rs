//! Akane, the crimson katana assassin: a deterministic core (snapshot in, effects out). `akane.rs` builds the frame
//! from the sim and applies the effects; the Lab (`native/akane_lab`) runs this same file.
//!
//!   passive Soul Harvest: a champion takedown leaves a Soul Orb (15 s). After 0.5 s, 0.75 s within 25,000 absorbs it:
//!           one Soul charge back, Kunai Flip reset, Crimson Edge reset at most once every 30 s. Never the ult.
//!   S1      Dismiss & Devour (2 charges, 10 s each): 1.5 s invisible, invulnerable, +60% speed, move only; a heal.
//!   S2      Kunai Flip: flip back and throw; the first enemy hit (never a tower) is marked. Recast: dash to it, over
//!           walls too (glide, hop, glide), and slash (a parryable strike).
//!   S3      Crimson Edge: the next 5 basic attacks: bleed; bleed + slow; bleed + vulnerable; the 4th stack bursts
//!           (% max HP, half around it, crimson bleed); the 5th: invulnerable, behind the target, area knock-up.
//!   Ult     Crimson Night: 4 s of nearsight on every enemy (her team hidden beyond 20,000 of their eyes); one recast
//!           dash, over walls, to a visible enemy champion within 120,000.
//!
//! Her AI is her own (Reihan: the global AI is a separate project). It reads what the bridge puts in the frame
//! (Flash, skill readiness, roster abilities, protections, towers) and every one of the 13 athlete values shapes a
//! choice, never a number. Her skills never touch towers.
use crate::akane_data as d;
use serde::{Deserialize, Serialize};

pub const MAP: i64 = 960_000;
pub const CELL: i64 = 32_000;
pub const CELLS: i64 = 30;

// the 13 athlete values in the frame's profile
pub const EGO: usize = 0;
pub const JUDGEMENT: usize = 1;
pub const POSITIONING: usize = 2;
pub const CONTROL_SPEED: usize = 3;
pub const SKILL_HIT: usize = 4;
pub const SKILL_AVOID: usize = 5;
pub const AGGRESSIVE: usize = 6;
pub const ORDER: usize = 7;
pub const ROAMING: usize = 8;
pub const MACRO: usize = 9;
pub const MICRO: usize = 10;
pub const AWARENESS: usize = 11;
pub const STRATEGY: usize = 12;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Point {
    pub x: i64,
    pub y: i64,
}
impl Point {
    pub const fn new(x: i64, y: i64) -> Self {
        Point { x, y }
    }
    pub fn d2(self, o: Point) -> i128 {
        let (dx, dy) = ((self.x - o.x) as i128, (self.y - o.y) as i128);
        dx * dx + dy * dy
    }
    pub fn dist(self, o: Point) -> f64 {
        (self.d2(o) as f64).sqrt()
    }
    pub fn within(self, o: Point, r: i64) -> bool {
        self.d2(o) <= (r as i128) * (r as i128)
    }
    pub fn toward(self, to: Point, len: i64) -> Point {
        let dd = self.dist(to);
        if dd < 1.0 {
            return Point::new(self.x + len, self.y);
        }
        let k = len as f64 / dd;
        Point::new(self.x + ((to.x - self.x) as f64 * k).round() as i64, self.y + ((to.y - self.y) as f64 * k).round() as i64)
    }
    pub fn clamp_map(self) -> Point {
        Point::new(self.x.clamp(1_000, MAP - 1_000), self.y.clamp(1_000, MAP - 1_000))
    }
    pub fn on_map(self) -> bool {
        (0..=MAP).contains(&self.x) && (0..=MAP).contains(&self.y)
    }
    /// distance from self to the segment a-b, and how far along it (0..=1)
    pub fn seg(self, a: Point, b: Point) -> (f64, f64) {
        let (vx, vy) = ((b.x - a.x) as f64, (b.y - a.y) as f64);
        let len2 = vx * vx + vy * vy;
        let k = if len2 == 0.0 { 0.0 } else { (((self.x - a.x) as f64 * vx + (self.y - a.y) as f64 * vy) / len2).clamp(0.0, 1.0) };
        let q = (a.x as f64 + vx * k, a.y as f64 + vy * k);
        (((self.x as f64 - q.0).powi(2) + (self.y as f64 - q.1).powi(2)).sqrt(), k)
    }
}

fn pct(base: i64, ratio: i64, ad: i64) -> i64 {
    base + ratio * ad / 100
}
fn t(n: i64) -> u64 {
    n.max(0) as u64
}

// ------------------------------------------------------------------ frame

/// What a champion's skills can do (from the roster), as bits.
pub const CAN_DASH: u8 = 1;
pub const CAN_BLINK: u8 = 2;
pub const CAN_CC: u8 = 4;
pub const CAN_HEAL: u8 = 8;
pub const CAN_SHIELD: u8 = 16;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Unit {
    pub id: usize,
    pub team: usize,
    pub pos: Point,
    pub hp: i64,
    pub max_hp: i64,
    #[serde(default)]
    pub shield: i64,
    pub radius: i64,
    #[serde(default)]
    pub armor: i64,
    #[serde(default)]
    pub attack: i64,
    pub champion: bool,
    pub tower: bool,
    pub visible: bool,
    pub targetable: bool,
    #[serde(default)]
    pub farmable: bool,
    #[serde(default)]
    pub stunned: bool,
    #[serde(default)]
    pub velocity: Point,
    /// champions: Flash off cooldown
    #[serde(default)]
    pub flash_ready: bool,
    /// champions: a skill (S1, S2 or ult) ready to cast
    #[serde(default)]
    pub skill_ready: bool,
    /// champions: what their skills can do (CAN_* bits, from the roster)
    #[serde(default)]
    pub can: u8,
    #[serde(default)]
    pub cc_immune: bool,
    /// incoming damage reduction, percent
    #[serde(default)]
    pub reduce: i64,
    /// V1's parry is up: a committed strike would be returned
    #[serde(default)]
    pub parry: bool,
    /// towers: someone of her team (a minion or champion) is inside its range for it to shoot first
    #[serde(default)]
    pub tanked: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "aim", rename_all = "snake_case")]
pub enum Order {
    Soul,
    Kunai(Point),
    KunaiRecast,
    Edge,
    Night,
    /// the ult's recast: dash to this enemy champion
    NightDash(usize),
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Frame {
    pub tick: u64,
    pub owner: usize,
    pub team: usize,
    pub alive: bool,
    pub pos: Point,
    pub hp: i64,
    pub max_hp: i64,
    pub attack: i64,
    #[serde(default)]
    pub armor: i64,
    pub stunned: bool,
    pub ult_unlocked: bool,
    #[serde(default)]
    pub flash_ready: bool,
    /// her side's base (where she runs home to)
    #[serde(default)]
    pub home: Point,
    pub units: Vec<Unit>,
    #[serde(default)]
    pub terrain: Vec<bool>,
    #[serde(default)]
    pub bushes: Vec<bool>,
    pub profile: [i64; 13],
    /// units her accepted basic attacks hit since the last frame
    #[serde(default)]
    pub basic_hits: Vec<usize>,
    /// champion takedowns (kills and assists) since the last frame: where the victim fell
    #[serde(default)]
    pub takedowns: Vec<(usize, Point)>,
    pub manual: bool,
    pub order: Option<Order>,
}
impl Frame {
    pub fn enemy(&self, u: &Unit) -> bool {
        u.team != self.team && u.visible && u.targetable && u.hp > 0
    }
    /// enemies her skills may hit: never a tower
    pub fn skill_target(&self, u: &Unit) -> bool {
        self.enemy(u) && !u.tower
    }
    pub fn enemy_champions(&self) -> impl Iterator<Item = &Unit> {
        self.units.iter().filter(move |u| u.champion && self.enemy(u))
    }
    pub fn allies(&self) -> impl Iterator<Item = &Unit> {
        self.units.iter().filter(move |u| u.champion && u.team == self.team && u.hp > 0)
    }
    pub fn p(&self, i: usize) -> i64 {
        self.profile.get(i).copied().unwrap_or(50).clamp(0, 100)
    }
    fn grid(g: &[bool], p: Point) -> bool {
        g.get(((p.y / CELL) * CELLS + p.x / CELL) as usize).copied().unwrap_or(false)
    }
    pub fn terrain_at(&self, p: Point) -> bool {
        !p.on_map() || Self::grid(&self.terrain, p)
    }
    pub fn bush_at(&self, p: Point) -> bool {
        p.on_map() && Self::grid(&self.bushes, p)
    }
    /// walking from a toward b: the last free point before terrain
    pub fn clip(&self, a: Point, b: Point) -> Point {
        let len = a.dist(b) as i64;
        let steps = (len / 2_000).max(1);
        let mut last = a;
        for i in 1..=steps {
            let p = Point::new(a.x + (b.x - a.x) * i / steps, a.y + (b.y - a.y) * i / steps);
            if self.terrain_at(p) {
                return last;
            }
            last = p;
        }
        b
    }
    pub fn unit(&self, id: usize) -> Option<&Unit> {
        self.units.iter().find(|u| u.id == id)
    }
    /// no enemy champion near her: her skills may farm
    pub fn calm(&self) -> bool {
        !self.enemy_champions().any(|u| u.pos.within(self.pos, d::AI_CALM_RANGE))
    }
    /// inside a living enemy tower's range
    pub fn enemy_tower_at(&self, p: Point) -> Option<&Unit> {
        self.units.iter().find(|u| u.tower && u.team != self.team && u.hp > 0 && u.pos.within(p, d::AI_TOWER_RANGE))
    }
}

// ------------------------------------------------------------------ effects

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case")]
pub enum Effect {
    /// `direct`: a single committed strike on one target (a V1 parry may cancel it)
    Damage { target: usize, amount: i64, direct: bool },
    Slow { target: usize, percent: i64, ticks: u64 },
    KnockUp { target: usize, ticks: u64 },
    /// takes `percent` more damage from everyone
    Vulnerable { target: usize, percent: i64, ticks: u64 },
    /// bleed stacks shown on the target (0 clears)
    Bleed { target: usize, stacks: i64, ticks: u64 },
    Crimson { target: usize, ticks: u64 },
    /// the kunai's mark shown on the target (0 ticks clears)
    Mark { target: usize, ticks: u64 },
    /// Dismiss & Devour: invisible, invulnerable, fast and move-only until `ticks` from now (replaces a shorter one)
    Soul { ticks: u64, speed_pct: i64 },
    Heal { amount: i64 },
    /// invulnerable (damage and CC) for `ticks` (the Crimson Edge finale)
    Invulnerable { ticks: u64 },
    /// a teleport (the finale: behind the target)
    Teleport { to: Point },
    /// she moves herself to `to` over `ticks` (an engine glide)
    Glide { to: Point, ticks: u64 },
    /// a hop across terrain in one tick (only inside a dash over a wall)
    Hop { to: Point },
    /// Crimson Night: her team is hidden beyond the nearsight radius of enemy eyes for `ticks`
    Night { ticks: u64 },
    /// the AI walks her toward a point (the bridge turns it into a move order)
    Steer { to: Point },
    Fx { name: String, at: Point, ticks: u64, ambient: bool },
    Pose { name: &'static str, ticks: u64 },
    Log(String),
}

// ------------------------------------------------------------------ state

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Orb {
    pub pos: Point,
    pub born: u64,
    pub until: u64,
    /// absorbing since (within range, not stunned)
    pub since: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct DashPlan {
    pub target: usize,
    pub hop: Option<(u64, Point)>,
    pub after_hop: Option<(Point, u64)>,
    pub strike_at: u64,
    pub base: i64,
    pub ratio: i64,
    pub fx: &'static str,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Bleed {
    pub target: usize,
    pub stacks: i64,
    pub until: u64,
    pub next: u64,
    pub per_stack: i64,
}

/// A released non-homing kunai. Its Attack and endpoint are fixed on acceptance.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct KunaiFlight {
    pub born: u64,
    pub from: Point,
    pub end: Point,
    pub pos: Point,
    pub attack: i64,
}

#[derive(Clone, Debug, Serialize)]
pub struct Core {
    /// tick each skill is ready again: [Kunai, Edge, Night]
    pub ready: [u64; 3],
    pub charges: i64,
    pub next_charge: u64,
    /// Dismiss & Devour's window (invisible, invulnerable, move only)
    pub soul_until: u64,
    pub invuln_until: u64,
    pub orbs: Vec<Orb>,
    /// an orb may reset Crimson Edge again from
    pub edge_reset_ready: u64,
    /// the kunai's mark: (target, until)
    pub mark: Option<(usize, u64)>,
    pub kunai_flights: Vec<KunaiFlight>,
    pub dash: Option<DashPlan>,
    /// Crimson Edge: empowered hits landed so far, and the window's end
    pub edge: Option<(u8, u64)>,
    pub bleeds: Vec<Bleed>,
    /// crimson bleed: (target, until, damage per second, next)
    pub crimson: Vec<(usize, u64, i64, u64)>,
    pub night_until: u64,
    pub night_dash_used: bool,
    pub busy_until: u64,
    // ---- the AI's memory (her team's vision only)
    /// (enemy, last seen, tick)
    pub seen: Vec<(usize, Point, u64)>,
    /// (enemy, tick it used Flash)
    pub flashed: Vec<(usize, u64)>,
    /// enemies' Flash readiness last frame (to notice a Flash being used)
    pub flash_was: Vec<(usize, bool)>,
    /// kills chained in this fight, and when the fight calms down
    pub chain: u32,
    pub chain_until: u64,
    /// running home after the objective until
    pub disengage_until: u64,
    /// danger first noticed at (her reaction time)
    pub alarm_since: Option<u64>,
    pub rejection: String,
    pub last_skill: &'static str,
}
impl Default for Core {
    fn default() -> Self {
        Core {
            ready: [0; 3],
            charges: d::SOUL_CHARGES,
            next_charge: 0,
            soul_until: 0,
            invuln_until: 0,
            orbs: vec![],
            edge_reset_ready: 0,
            mark: None,
            kunai_flights: vec![],
            dash: None,
            edge: None,
            bleeds: vec![],
            crimson: vec![],
            night_until: 0,
            night_dash_used: false,
            busy_until: 0,
            seen: vec![],
            flashed: vec![],
            flash_was: vec![],
            chain: 0,
            chain_until: 0,
            disengage_until: 0,
            alarm_since: None,
            rejection: String::new(),
            last_skill: "",
        }
    }
}

pub const KUNAI: usize = 0;
pub const EDGE: usize = 1;
pub const NIGHT: usize = 2;

impl Core {
    pub fn respawn(&mut self, tick: u64) {
        let ready = self.ready;
        let edge_reset_ready = self.edge_reset_ready;
        *self = Core::default();
        self.ready = ready.map(|r| r.max(tick));
        self.edge_reset_ready = edge_reset_ready;
    }
    pub fn ready(&self, slot: usize, tick: u64) -> bool {
        tick >= self.ready[slot]
    }
    pub fn in_soul(&self, tick: u64) -> bool {
        tick < self.soul_until
    }
    pub fn night_on(&self, tick: u64) -> bool {
        tick < self.night_until
    }
    pub fn stacks_on(&self, id: usize, tick: u64) -> i64 {
        self.bleeds.iter().find(|b| b.target == id && b.until > tick).map_or(0, |b| b.stacks)
    }
    pub fn marked(&self, tick: u64) -> Option<usize> {
        self.mark.filter(|m| tick < m.1).map(|m| m.0)
    }

    pub fn step(&mut self, f: &Frame) -> Vec<Effect> {
        let mut out = vec![];
        if !f.alive {
            self.kunai_flights.clear();
            return out;
        }
        self.recharge(f);
        self.remember(f);
        for &(_, at) in &f.takedowns {
            self.orbs.push(Orb { pos: at, born: f.tick, until: f.tick + t(d::ORB_TICKS), since: None });
            out.push(Effect::Fx { name: "akn_orb".into(), at, ticks: t(d::ORB_TICKS), ambient: false });
            if f.tick < self.chain_until || self.chain == 0 {
                self.chain += 1;
            }
            self.chain_until = f.tick + 300;
        }
        if f.tick >= self.chain_until {
            self.chain = 0;
        }
        self.run_orbs(f, &mut out);
        if self.mark.is_some_and(|m| f.tick >= m.1) {
            self.mark = None;
        }
        if self.edge.is_some_and(|e| f.tick >= e.1) {
            self.edge = None;
        }
        for &target in &f.basic_hits {
            if let Some(u) = f.unit(target).cloned() {
                self.edge_hit(f, &u, &mut out);
            }
        }
        self.run_bleeds(f, &mut out);
        self.run_kunai(f, &mut out);
        self.run_dash(f, &mut out);
        let order = if f.manual { f.order } else { self.decide(f, &mut out) };
        if let Some(o) = order {
            self.execute(f, o, &mut out);
        }
        out
    }

    fn recharge(&mut self, f: &Frame) {
        if self.charges >= d::SOUL_CHARGES {
            self.next_charge = 0;
        } else if self.next_charge == 0 {
            self.next_charge = f.tick + t(d::SOUL_RECHARGE_TICKS);
        } else if f.tick >= self.next_charge {
            self.charges += 1;
            self.next_charge = if self.charges < d::SOUL_CHARGES { f.tick + t(d::SOUL_RECHARGE_TICKS) } else { 0 };
        }
    }

    // ---------------------------------------------------------------- passive: Soul Harvest

    fn run_orbs(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        let mut absorbed = vec![];
        for (i, o) in self.orbs.iter_mut().enumerate() {
            let armed = f.tick >= o.born + t(d::ORB_ARM_TICKS);
            if armed && !f.stunned && o.pos.within(f.pos, d::ORB_RANGE) {
                let since = *o.since.get_or_insert(f.tick);
                if f.tick >= since + t(d::ORB_ABSORB_TICKS) {
                    absorbed.push(i);
                }
            } else {
                o.since = None;
            }
        }
        for &i in absorbed.iter().rev() {
            let o = self.orbs.remove(i);
            self.absorb(f, o.pos, out);
        }
        self.orbs.retain(|o| f.tick < o.until);
    }

    /// One Soul charge back (never above the cap), Kunai Flip reset, Crimson Edge reset at most once every 30 s.
    fn absorb(&mut self, f: &Frame, at: Point, out: &mut Vec<Effect>) {
        if self.charges < d::SOUL_CHARGES {
            self.charges += 1;
            if self.charges >= d::SOUL_CHARGES {
                self.next_charge = 0;
            }
        }
        self.ready[KUNAI] = self.ready[KUNAI].min(f.tick);
        let mut edge = false;
        if f.tick >= self.edge_reset_ready {
            self.ready[EDGE] = self.ready[EDGE].min(f.tick);
            self.edge_reset_ready = f.tick + t(d::ORB_S3_RESET_COOLDOWN_TICKS);
            edge = true;
        }
        out.push(Effect::Fx { name: "akn_orb_absorb".into(), at, ticks: 18, ambient: false });
        out.push(Effect::Log(format!("soul absorbed: charges {}, kunai reset, edge reset {edge}", self.charges)));
    }

    // ---------------------------------------------------------------- S3: Crimson Edge

    fn add_bleed(&mut self, f: &Frame, id: usize, out: &mut Vec<Effect>) -> i64 {
        let per = pct(d::EDGE_BLEED_BASE, d::EDGE_BLEED_AD_RATIO, f.attack);
        let until = f.tick + t(d::EDGE_BLEED_TICKS);
        let stacks = match self.bleeds.iter_mut().find(|b| b.target == id) {
            Some(b) => {
                b.stacks = if b.until > f.tick { (b.stacks + 1).min(d::EDGE_MAX_STACKS) } else { 1 };
                b.until = until;
                b.per_stack = per;
                b.stacks
            }
            None => {
                self.bleeds.push(Bleed { target: id, stacks: 1, until, next: f.tick + 60, per_stack: per });
                1
            }
        };
        out.push(Effect::Bleed { target: id, stacks, ticks: t(d::EDGE_BLEED_TICKS) });
        stacks
    }

    /// An empowered basic attack. Towers take the hit but no debuff, and don't use an empowered attack.
    fn edge_hit(&mut self, f: &Frame, u: &Unit, out: &mut Vec<Effect>) {
        let Some((done, until)) = self.edge else { return };
        if u.tower || !f.skill_target(u) {
            return;
        }
        let n = done + 1;
        self.edge = if n as i64 >= d::EDGE_HITS { None } else { Some((n, until)) };
        match n {
            1 => {
                self.add_bleed(f, u.id, out);
            }
            2 => {
                self.add_bleed(f, u.id, out);
                out.push(Effect::Slow { target: u.id, percent: d::EDGE_SLOW_PCT, ticks: t(d::EDGE_SLOW_TICKS) });
            }
            3 => {
                self.add_bleed(f, u.id, out);
                out.push(Effect::Vulnerable { target: u.id, percent: d::EDGE_VULNERABLE_PCT, ticks: t(d::EDGE_VULNERABLE_TICKS) });
            }
            4 => {
                let stacks = self.add_bleed(f, u.id, out);
                if stacks >= d::EDGE_MAX_STACKS {
                    self.burst(f, u, out);
                }
            }
            _ => self.finale(f, u, out),
        }
    }

    /// The 4th stack bursts: % max HP + Attack on the target, half around it, crimson bleed on everyone hit.
    fn burst(&mut self, f: &Frame, u: &Unit, out: &mut Vec<Effect>) {
        self.bleeds.retain(|b| b.target != u.id);
        out.push(Effect::Bleed { target: u.id, stacks: 0, ticks: 0 });
        let main = u.max_hp * d::EDGE_BURST_PCT_MAX_HP / 100 + d::EDGE_BURST_AD_RATIO * f.attack / 100;
        out.push(Effect::Fx { name: "akn_burst".into(), at: u.pos, ticks: 20, ambient: false });
        out.push(Effect::Damage { target: u.id, amount: main, direct: false });
        self.add_crimson(f, u, out);
        let around: Vec<Unit> = f
            .units
            .iter()
            .filter(|o| o.id != u.id && f.skill_target(o) && o.pos.within(u.pos, d::EDGE_BURST_RADIUS + o.radius))
            .cloned()
            .collect();
        for o in around {
            let amount = (o.max_hp * d::EDGE_BURST_PCT_MAX_HP / 100 + d::EDGE_BURST_AD_RATIO * f.attack / 100) / 2;
            out.push(Effect::Damage { target: o.id, amount, direct: false });
            self.add_crimson(f, &o, out);
        }
    }

    fn add_crimson(&mut self, f: &Frame, u: &Unit, out: &mut Vec<Effect>) {
        let per = (u.max_hp * d::EDGE_CRIMSON_PERMILLE_PER_SECOND / 1000).max(1);
        let until = f.tick + t(d::EDGE_CRIMSON_TICKS);
        self.crimson.retain(|c| c.0 != u.id);
        self.crimson.push((u.id, until, per, f.tick + 60));
        out.push(Effect::Crimson { target: u.id, ticks: t(d::EDGE_CRIMSON_TICKS) });
    }

    /// Where the finale puts her: behind the target, on the side away from its allies when she reads the fight well.
    pub fn behind(f: &Frame, u: &Unit) -> Point {
        let allies: Vec<&Unit> = f.enemy_champions().filter(|o| o.id != u.id && o.pos.within(u.pos, 60_000)).collect();
        let from = if !allies.is_empty() && f.p(POSITIONING) >= 40 {
            // put the target between her and its allies
            let n = allies.len() as i64;
            Point::new(allies.iter().map(|a| a.pos.x).sum::<i64>() / n, allies.iter().map(|a| a.pos.y).sum::<i64>() / n)
        } else {
            f.pos
        };
        let spot = if from == u.pos { Point::new(u.pos.x + d::EDGE_FINALE_BEHIND, u.pos.y) } else { from.toward(u.pos, from.dist(u.pos) as i64 + d::EDGE_FINALE_BEHIND) };
        if f.terrain_at(spot) { u.pos } else { spot.clamp_map() }
    }

    fn finale(&mut self, f: &Frame, u: &Unit, out: &mut Vec<Effect>) {
        let ticks = t(d::EDGE_FINALE_INVULN_TICKS);
        self.invuln_until = self.invuln_until.max(f.tick + ticks);
        out.push(Effect::Invulnerable { ticks });
        let to = Self::behind(f, u);
        out.push(Effect::Teleport { to });
        out.push(Effect::Fx { name: "akn_finale".into(), at: to, ticks: 18, ambient: false });
        out.push(Effect::Pose { name: "skill2", ticks: 14 });
        for o in f.units.iter().filter(|o| f.skill_target(o) && o.pos.within(to, d::EDGE_KNOCK_RADIUS + o.radius)) {
            out.push(Effect::KnockUp { target: o.id, ticks: t(d::EDGE_KNOCK_TICKS) });
        }
    }

    fn run_bleeds(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        for b in self.bleeds.iter_mut() {
            if f.tick >= b.next && f.tick <= b.until {
                b.next = f.tick + 60;
                if f.unit(b.target).is_some_and(|u| u.hp > 0 && !u.tower) {
                    out.push(Effect::Damage { target: b.target, amount: b.per_stack * b.stacks, direct: false });
                }
            }
        }
        self.bleeds.retain(|b| b.until > f.tick && f.unit(b.target).is_some_and(|u| u.hp > 0));
        for c in self.crimson.iter_mut() {
            if f.tick >= c.3 && f.tick <= c.1 {
                c.3 = f.tick + 60;
                if f.unit(c.0).is_some_and(|u| u.hp > 0 && !u.tower) {
                    out.push(Effect::Damage { target: c.0, amount: c.2, direct: false });
                }
            }
        }
        self.crimson.retain(|c| c.1 > f.tick && f.unit(c.0).is_some_and(|u| u.hp > 0));
    }

    // ---------------------------------------------------------------- dashes (Kunai recast, the ult)

    /// Dash to a unit: a glide on open ground; across terrain, a glide to the wall, a hop over it, then a glide to the
    /// landing spot next to the unit (Kratos's Chain Dash plan, unchanged).
    fn start_dash(&mut self, f: &Frame, u: &Unit, speed: i64, base: i64, ratio: i64, fx: &'static str, out: &mut Vec<Effect>) {
        let gap = (u.radius + 4_000).min(f.pos.dist(u.pos) as i64);
        let mut land = u.pos.toward(f.pos, gap);
        if f.terrain_at(land) {
            land = u.pos;
        }
        let speed = speed.max(1);
        let clear = f.clip(f.pos, land);
        let mut plan = DashPlan { target: u.id, hop: None, after_hop: None, strike_at: 0, base, ratio, fx };
        if clear == land {
            let ticks = ((f.pos.dist(land) as i64) / speed).max(2) as u64;
            out.push(Effect::Glide { to: land, ticks });
            plan.strike_at = f.tick + ticks;
        } else {
            let t1 = ((f.pos.dist(clear) as i64) / speed).max(1) as u64;
            if clear != f.pos {
                out.push(Effect::Glide { to: clear, ticks: t1 });
            }
            let len = clear.dist(land) as i64;
            let steps = (len / 2_000).max(1);
            let mut over = land;
            let mut seen_wall = false;
            for i in 1..=steps {
                let p = Point::new(clear.x + (land.x - clear.x) * i / steps, clear.y + (land.y - clear.y) * i / steps);
                if f.terrain_at(p) {
                    seen_wall = true;
                } else if seen_wall {
                    over = p;
                    break;
                }
            }
            let t2 = ((over.dist(land) as i64) / speed).max(1) as u64;
            plan.hop = Some((f.tick + t1, over));
            plan.after_hop = (over != land).then_some((land, t2));
            plan.strike_at = f.tick + t1 + 1 + if over != land { t2 } else { 0 };
            out.push(Effect::Log(format!("dash over a wall to {}", u.id)));
        }
        self.busy_until = plan.strike_at;
        self.dash = Some(plan);
    }

    fn run_dash(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        let Some(p) = self.dash.clone() else { return };
        if let Some((at, to)) = p.hop {
            if f.tick == at {
                out.push(Effect::Hop { to });
                if let Some((land, ticks)) = p.after_hop {
                    out.push(Effect::Glide { to: land, ticks });
                }
            }
        }
        if f.tick >= p.strike_at {
            self.dash = None;
            if let Some(u) = f.unit(p.target).filter(|u| f.skill_target(u)).cloned() {
                out.push(Effect::Fx { name: p.fx.into(), at: u.pos, ticks: 14, ambient: false });
                out.push(Effect::Damage { target: u.id, amount: pct(p.base, p.ratio, f.attack), direct: true });
            }
        }
    }

    // ---------------------------------------------------------------- orders

    pub fn execute(&mut self, f: &Frame, o: Order, out: &mut Vec<Effect>) {
        self.rejection.clear();
        let reject = |s: &mut Self, why: &str| s.rejection = why.into();
        if f.stunned {
            return reject(self, "crowd-controlled");
        }
        // Dismiss & Devour: move only, until it ends (another charge may extend it)
        if self.in_soul(f.tick) && o != Order::Soul {
            return reject(self, "Dismiss: she can only move");
        }
        if f.tick < self.busy_until && o != Order::Soul {
            return reject(self, "mid-dash");
        }
        match o {
            Order::Soul => {
                if self.charges <= 0 {
                    return reject(self, "no Soul charge");
                }
                self.charges -= 1;
                if self.next_charge == 0 {
                    self.next_charge = f.tick + t(d::SOUL_RECHARGE_TICKS);
                }
                // windows never stack: an overlap only extends to the new end
                self.soul_until = self.soul_until.max(f.tick + t(d::SOUL_TICKS));
                self.invuln_until = self.invuln_until.max(self.soul_until);
                out.push(Effect::Soul { ticks: self.soul_until - f.tick, speed_pct: d::SOUL_SPEED_PCT });
                out.push(Effect::Heal { amount: f.max_hp * d::SOUL_HEAL_PCT_MAX_HP / 100 + d::SOUL_HEAL_AD_RATIO * f.attack / 100 });
                out.push(Effect::Fx { name: "akn_dismiss".into(), at: f.pos, ticks: 20, ambient: false });
                self.last_skill = "soul";
            }
            Order::Kunai(aim) => {
                if !self.ready(KUNAI, f.tick) {
                    return reject(self, "Kunai Flip is on cooldown");
                }
                self.ready[KUNAI] = f.tick + t(d::KUNAI_COOLDOWN_TICKS);
                let aim = if aim == f.pos { Point::new(f.pos.x + 1, f.pos.y) } else { aim };
                let back = f.clip(f.pos, f.pos.toward(aim, -d::KUNAI_FLIP));
                if back != f.pos {
                    out.push(Effect::Glide { to: back, ticks: t(d::KUNAI_FLIP_TICKS) });
                    self.busy_until = f.tick + t(d::KUNAI_FLIP_TICKS);
                }
                let end = f.pos.toward(aim, d::KUNAI_RANGE).clamp_map();
                out.push(Effect::Fx { name: "akn_flip".into(), at: f.pos, ticks: t(d::KUNAI_FLIP_TICKS), ambient: false });
                out.push(Effect::Fx { name: "akn_kunai".into(), at: f.pos, ticks: 1, ambient: false });
                out.push(Effect::Pose { name: "skill1", ticks: t(d::KUNAI_FLIP_TICKS) });
                self.last_skill = "kunai";
                self.kunai_flights.push(KunaiFlight{born:f.tick,from:f.pos,end,pos:f.pos,attack:f.attack});
            }
            Order::KunaiRecast => {
                let Some(id) = self.marked(f.tick) else { return reject(self, "nothing marked") };
                let Some(u) = f.unit(id).filter(|u| f.skill_target(u)).cloned() else { return reject(self, "the mark is out of reach") };
                self.mark = None;
                out.push(Effect::Mark { target: id, ticks: 0 });
                out.push(Effect::Pose { name: "skill1", ticks: 12 });
                self.start_dash(f, &u, d::KUNAI_DASH_SPEED, d::KUNAI_SLASH_BASE, d::KUNAI_SLASH_AD_RATIO, "akn_dash_slash", out);
                self.last_skill = "kunai_recast";
            }
            Order::Edge => {
                if !self.ready(EDGE, f.tick) {
                    return reject(self, "Crimson Edge is on cooldown");
                }
                self.ready[EDGE] = f.tick + t(d::EDGE_COOLDOWN_TICKS);
                self.edge = Some((0, f.tick + t(d::EDGE_WINDOW_TICKS)));
                out.push(Effect::Pose { name: "skill2", ticks: 10 });
                self.last_skill = "edge";
            }
            Order::Night => {
                if !f.ult_unlocked {
                    return reject(self, "ult not learned");
                }
                if !self.ready(NIGHT, f.tick) {
                    return reject(self, "Crimson Night is on cooldown");
                }
                self.ready[NIGHT] = f.tick + t(d::NIGHT_COOLDOWN_TICKS);
                self.night_until = f.tick + t(d::NIGHT_TICKS);
                self.night_dash_used = false;
                out.push(Effect::Night { ticks: t(d::NIGHT_TICKS) });
                out.push(Effect::Fx { name: "akn_night".into(), at: f.pos, ticks: t(d::NIGHT_TICKS), ambient: false });
                out.push(Effect::Pose { name: "ult", ticks: 18 });
                self.last_skill = "night";
            }
            Order::NightDash(id) => {
                if !self.night_on(f.tick) || self.night_dash_used {
                    return reject(self, "no Crimson Night dash left");
                }
                let Some(u) = f.unit(id).filter(|u| u.champion && f.skill_target(u) && u.pos.within(f.pos, d::NIGHT_DASH_RANGE)).cloned() else {
                    return reject(self, "no visible enemy champion in reach");
                };
                self.night_dash_used = true;
                out.push(Effect::Pose { name: "ult", ticks: 14 });
                self.start_dash(f, &u, d::NIGHT_DASH_SPEED, d::NIGHT_BASE, d::NIGHT_AD_RATIO, "akn_night_slash", out);
                self.last_skill = "night_dash";
            }
        }
    }

    fn run_kunai(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        let mut active=vec![];
        for mut shot in std::mem::take(&mut self.kunai_flights) {
            let distance=(f.tick.saturating_sub(shot.born) as i64).saturating_mul(d::KUNAI_SPEED);
            let end_distance=shot.from.dist(shot.end);
            let to=if distance as f64>=end_distance {shot.end}else{shot.from.toward(shot.end,distance)};
            // Each tick sweeps the traversed segment; it never homes or hits a tower.
            if let Some(u)=Self::kunai_target(f,shot.pos,to) {
                out.push(Effect::Damage{target:u.id,amount:pct(d::KUNAI_BASE,d::KUNAI_AD_RATIO,shot.attack),direct:false});
                if let Some((old,_))=self.mark.filter(|m|m.0!=u.id){out.push(Effect::Mark{target:old,ticks:0});}
                self.mark=Some((u.id,f.tick+t(d::KUNAI_MARK_TICKS)));
                out.push(Effect::Mark{target:u.id,ticks:t(d::KUNAI_MARK_TICKS)});
                out.push(Effect::Fx{name:"akn_kunai_hit".into(),at:u.pos,ticks:12,ambient:false});
            }else if to!=shot.end {shot.pos=to;active.push(shot);}
            else {out.push(Effect::Log("kunai missed".into()));}
        }
        self.kunai_flights=active;
    }

    /// The first enemy the kunai's segment touches (never a tower; it flies over terrain).
    pub fn kunai_target(f: &Frame, from: Point, end: Point) -> Option<Unit> {
        f.units
            .iter()
            .filter(|u| f.skill_target(u))
            .filter_map(|u| {
                let (dist, k) = u.pos.seg(from, end);
                (dist <= (d::KUNAI_WIDTH + u.radius) as f64 && (k > 0.0 || from.within(u.pos,d::KUNAI_WIDTH+u.radius))).then_some((k, u))
            })
            .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal).then(a.1.id.cmp(&b.1.id)))
            .map(|(_, u)| u.clone())
    }

    // ================================================================ the athlete (her own AI)

    /// Remember who was seen where, and who just used Flash.
    fn remember(&mut self, f: &Frame) {
        for u in f.units.iter().filter(|u| u.champion && u.team != f.team && u.visible && u.hp > 0) {
            match self.seen.iter_mut().find(|s| s.0 == u.id) {
                Some(s) => {
                    s.1 = u.pos;
                    s.2 = f.tick;
                }
                None => self.seen.push((u.id, u.pos, f.tick)),
            }
            let was = self.flash_was.iter().find(|w| w.0 == u.id).map(|w| w.1);
            if was == Some(true) && !u.flash_ready {
                self.flashed.retain(|x| x.0 != u.id);
                self.flashed.push((u.id, f.tick));
            }
            match self.flash_was.iter_mut().find(|w| w.0 == u.id) {
                Some(w) => w.1 = u.flash_ready,
                None => self.flash_was.push((u.id, u.flash_ready)),
            }
        }
        self.flashed.retain(|x| f.tick < x.1 + 600);
    }

    /// how long a missing enemy stays a worry (awareness)
    pub fn memory_ticks(f: &Frame) -> u64 {
        180 + f.p(AWARENESS) as u64 * 3
    }
    pub fn just_flashed(&self, id: usize, tick: u64, within: u64) -> bool {
        self.flashed.iter().any(|x| x.0 == id && tick < x.1 + within)
    }

    /// A deterministic estimate error: the lower her judgement, the further off (up to ±25%).
    pub fn noise(f: &Frame, salt: u64) -> i64 {
        let spread = (100 - f.p(JUDGEMENT)) / 4;
        if spread == 0 {
            return 0;
        }
        let h = (f.tick / 60).wrapping_mul(0x9E37_79B9).wrapping_add(salt.wrapping_mul(0x85EB_CA6B)) % (2 * spread as u64 + 1);
        h as i64 - spread
    }

    /// Effective HP against her physical damage (shield, armor, damage reduction). None: untouchable now.
    pub fn effective_hp(u: &Unit) -> Option<i64> {
        if u.reduce >= 100 || u.cc_immune && u.reduce > 0 {
            return None;
        }
        let raw = (u.hp + u.shield) * (100 + u.armor.max(0)) / 100;
        Some(raw * 100 / (100 - u.reduce.clamp(0, 95)))
    }

    /// Extra burst a kill needs for the target's outs, in percent (aggression shrinks it).
    pub fn escape_margin(&self, f: &Frame, u: &Unit) -> i64 {
        let mut m = 10;
        if u.flash_ready && !self.just_flashed(u.id, f.tick, 600) {
            m += d::AI_FLASH_MARGIN_PCT;
        }
        if u.skill_ready && u.can & (CAN_DASH | CAN_BLINK) != 0 {
            m += d::AI_DASH_MARGIN_PCT;
        }
        if u.skill_ready && u.can & (CAN_HEAL | CAN_SHIELD) != 0 {
            m += d::AI_SUSTAIN_MARGIN_PCT;
        }
        m * (130 - f.p(AGGRESSIVE) * 6 / 10) / 100
    }

    /// The damage of the combo she has right now on `u` (before its armor), with her judgement's error.
    pub fn combo_damage(&self, f: &Frame, u: &Unit) -> i64 {
        let atk = f.attack;
        let mut dmg = atk * 3; // basic attacks she'll get in
        if self.ready(KUNAI, f.tick) || self.marked(f.tick) == Some(u.id) {
            if self.marked(f.tick) != Some(u.id) {
                dmg += pct(d::KUNAI_BASE, d::KUNAI_AD_RATIO, atk);
            }
            if !u.parry {
                dmg += pct(d::KUNAI_SLASH_BASE, d::KUNAI_SLASH_AD_RATIO, atk);
            }
        }
        if self.ready(EDGE, f.tick) || self.edge.is_some() {
            let bleed = pct(d::EDGE_BLEED_BASE, d::EDGE_BLEED_AD_RATIO, atk);
            let burst = u.max_hp * d::EDGE_BURST_PCT_MAX_HP / 100 + d::EDGE_BURST_AD_RATIO * atk / 100;
            let crimson = u.max_hp * d::EDGE_CRIMSON_PERMILLE_PER_SECOND / 1000 * (d::EDGE_CRIMSON_TICKS / 60);
            // two more basics, bleeds (1+2+3 stacks over the hits), the burst; vulnerable for the burst
            dmg += atk * 2 + bleed * 6 + (burst + crimson) * (100 + d::EDGE_VULNERABLE_PCT) / 100;
        }
        if self.night_on(f.tick) && !self.night_dash_used || f.ult_unlocked && self.ready(NIGHT, f.tick) {
            if !u.parry {
                dmg += pct(d::NIGHT_BASE, d::NIGHT_AD_RATIO, atk);
            }
        }
        dmg * (100 + Self::noise(f, u.id as u64)) / 100
    }

    /// Lethal: her burst covers the target's effective HP plus the margin for its outs.
    pub fn lethal(&self, f: &Frame, u: &Unit, extra_pct: i64) -> bool {
        let Some(ehp) = Self::effective_hp(u) else { return false };
        self.combo_damage(f, u) * 100 >= ehp * (100 + self.escape_margin(f, u) + extra_pct)
    }
    /// how much of the target's effective HP her burst covers, percent
    pub fn kill_share(&self, f: &Frame, u: &Unit) -> i64 {
        Self::effective_hp(u).map_or(0, |e| self.combo_damage(f, u) * 100 / e.max(1))
    }
    pub fn isolated(f: &Frame, u: &Unit) -> bool {
        !f.enemy_champions().any(|o| o.id != u.id && o.pos.within(u.pos, d::AI_ISOLATION))
    }

    /// Expected damage on her at `p` in the next couple of seconds: enemies who can reach it (their attacks, a CC
    /// chain if ready, Flash), enemies she can't see but remembers nearby, and a tower with nobody else to shoot.
    pub fn danger_at(&self, f: &Frame, p: Point) -> i64 {
        let awareness = 80 + f.p(AWARENESS) * 2 / 5; // 80..120% of the threat radius
        let mut danger = 0;
        for u in f.enemy_champions() {
            let mut reach = 35_000 + if u.skill_ready && u.can & (CAN_DASH | CAN_BLINK) != 0 { 30_000 } else { 0 };
            if u.flash_ready && !self.just_flashed(u.id, f.tick, 600) {
                reach += 40_000;
            }
            if u.pos.within(p, reach * awareness / 100) && !u.stunned {
                danger += u.attack.max(60) * d::AI_THREAT_SECONDS * 2 + if u.skill_ready && u.can & CAN_CC != 0 { 200 } else { 0 };
            }
        }
        let memory = Self::memory_ticks(f);
        for s in self.seen.iter().filter(|s| f.tick > s.2 && f.tick < s.2 + memory) {
            let visible = f.units.iter().any(|u| u.id == s.0 && u.visible && u.hp > 0);
            if !visible && s.1.within(p, 70_000 * awareness / 100) {
                danger += 150;
            }
        }
        if let Some(tw) = f.enemy_tower_at(p) {
            if !tw.tanked {
                danger += 400;
            }
        }
        danger
    }
    /// the danger she'll stand: her HP, more for bolder athletes, less for cautious positioning
    pub fn danger_limit(f: &Frame, escape: bool) -> i64 {
        let pct = 35 + f.p(AGGRESSIVE) * 3 / 10 - f.p(POSITIONING) / 10 + if escape { 25 } else { 0 };
        f.hp * pct / 100
    }
    /// an out after a dive: a Soul charge, Flash, or Kunai Flip's backflip
    pub fn has_escape(&self, f: &Frame) -> bool {
        self.charges > 0 || f.flash_ready || self.ready(KUNAI, f.tick)
    }

    /// The safest nearby spot: least danger, out of tower range, toward home, bushes preferred.
    pub fn safe_spot(&self, f: &Frame) -> Point {
        let mut best = (i64::MAX, f.pos);
        for r in [30_000i64, 60_000] {
            for i in 0..8 {
                let a = (i as f64) * std::f64::consts::TAU / 8.0;
                let p = f.clip(f.pos, Point::new(f.pos.x + (r as f64 * a.cos()) as i64, f.pos.y + (r as f64 * a.sin()) as i64)).clamp_map();
                if p == f.pos {
                    continue;
                }
                let mut score = self.danger_at(f, p) * 10 - (f.pos.dist(f.home) - p.dist(f.home)) as i64 / 100;
                if f.bush_at(p) {
                    score -= 300;
                }
                if f.enemy_tower_at(p).is_some() {
                    score += 4_000;
                }
                if score < best.0 {
                    best = (score, p);
                }
            }
        }
        best.1
    }

    /// The champion she'd hunt: lethal or nearly, isolated and low preferred, within her roaming reach; teammates'
    /// fights first for an orderly athlete, kills left to a nearby ally for a modest one.
    pub fn hunt(&self, f: &Frame) -> Option<Unit> {
        let reach = 60_000 + f.p(ROAMING) * 600;
        f.enemy_champions()
            .filter(|u| u.pos.within(f.pos, reach) && Self::effective_hp(u).is_some())
            .map(|u| {
                // overkill doesn't make a better target: past 150% they all die
                let mut score = self.kill_share(f, u).min(150) * 2 - u.hp * 100 / u.max_hp.max(1);
                if Self::isolated(f, u) {
                    score += 60;
                }
                let ally_on_it = f.allies().any(|a| a.pos.within(u.pos, 30_000));
                if ally_on_it {
                    score += f.p(ORDER) / 2; // join the team's engage
                    if f.p(EGO) < 40 && f.allies().any(|a| a.pos.dist(u.pos) < f.pos.dist(u.pos)) {
                        score -= 80; // leave it to the closer teammate
                    }
                }
                (score, u.clone())
            })
            .max_by_key(|(s, u)| (*s, std::cmp::Reverse(u.id)))
            .map(|(_, u)| u)
    }

    /// Where to aim the kunai at `u`: lead its movement by how well she hits skill shots.
    pub fn aim_at(f: &Frame, u: &Unit) -> Point {
        let travel = (f.pos.dist(u.pos) as i64 / d::KUNAI_SPEED.max(1)).max(1);
        let lead = travel * f.p(SKILL_HIT) / 100;
        Point::new(u.pos.x + u.velocity.x * lead, u.pos.y + u.velocity.y * lead)
    }

    /// Her reaction: danger must have been noticed this many ticks ago before she reacts (control speed, skill avoid).
    pub fn reaction_ticks(f: &Frame) -> u64 {
        ((200 - f.p(CONTROL_SPEED) - f.p(SKILL_AVOID)) / 20).max(0) as u64
    }

    /// The chain bar: each extra kill in a fight asks for more (less for aggressive athletes).
    pub fn chain_extra(&self, f: &Frame) -> i64 {
        self.chain as i64 * (20 - f.p(AGGRESSIVE) / 10)
    }

    pub fn decide(&mut self, f: &Frame, out: &mut Vec<Effect>) -> Option<Order> {
        if f.stunned {
            return None;
        }
        let me_danger = self.danger_at(f, f.pos);
        let escape = self.has_escape(f);
        let limit = Self::danger_limit(f, escape);
        let _ = escape;
        // ---- in Dismiss: move only, to safety (or to an orb that's safe)
        if self.in_soul(f.tick) {
            let to = self.safe_orb(f).unwrap_or_else(|| self.safe_spot(f));
            out.push(Effect::Steer { to });
            return None;
        }
        if f.tick < self.busy_until {
            return None;
        }
        // ---- dodge: a ready CC or burst about to land on her, noticed in time
        let cc_close = f.enemy_champions().any(|u| u.skill_ready && u.can & CAN_CC != 0 && u.pos.within(f.pos, 40_000) && !u.stunned);
        let tower_on_me = f.enemy_tower_at(f.pos).is_some_and(|tw| !tw.tanked);
        let alarmed = (cc_close && me_danger > limit / 2) || me_danger > limit || tower_on_me || f.hp * 100 < f.max_hp * 25 && me_danger > 0;
        if alarmed {
            let since = *self.alarm_since.get_or_insert(f.tick);
            let reserve_ok = self.charges > 0;
            if f.tick >= since + Self::reaction_ticks(f) && reserve_ok {
                self.alarm_since = None;
                self.disengage_until = f.tick + 120;
                return Some(Order::Soul);
            }
        } else {
            self.alarm_since = None;
        }
        // ---- Crimson Night's dash: the best moment within the 4 s
        if self.night_on(f.tick) && !self.night_dash_used {
            let left = self.night_until - f.tick;
            let target = f
                .enemy_champions()
                .filter(|u| u.pos.within(f.pos, d::NIGHT_DASH_RANGE) && Self::effective_hp(u).is_some())
                .map(|u| (self.kill_share(f, u), u))
                .max_by_key(|(s, u)| (*s, std::cmp::Reverse(u.id)));
            if let Some((share, u)) = target {
                let flashed = self.just_flashed(u.id, f.tick, 60);
                let out_of_outs = !u.flash_ready && !(u.skill_ready && u.can & (CAN_DASH | CAN_BLINK) != 0);
                let patience = 30 + f.p(STRATEGY) / 2; // ticks before the end she waits for
                if self.lethal(f, u, 0) && (flashed || out_of_outs || left <= patience as u64) || left <= 20 && share >= 70 {
                    return Some(Order::NightDash(u.id));
                }
            }
        }
        // ---- the Kunai recast, judged at the moment
        if let Some(id) = self.marked(f.tick) {
            if let Some(u) = f.unit(id).filter(|u| f.skill_target(u)).cloned() {
                if self.recast_ok(f, &u) {
                    return Some(Order::KunaiRecast);
                }
            }
        }
        // ---- after the objective: chain if it's still worth it, else get out
        if f.tick < self.disengage_until {
            if let Some(next) = self.hunt(f).filter(|u| self.lethal(f, u, self.chain_extra(f)) && escape) {
                if self.danger_at(f, next.pos) <= limit {
                    self.disengage_until = 0;
                }
            }
            if f.tick < self.disengage_until {
                let to = self.safe_orb(f).unwrap_or_else(|| self.safe_spot(f));
                out.push(Effect::Steer { to });
                return None;
            }
        }
        if !f.takedowns.is_empty() && f.p(MACRO) < 70 {
            // the kill is done: re-score before going again
            let worth = self.hunt(f).is_some_and(|u| self.lethal(f, &u, self.chain_extra(f)) && escape && self.danger_at(f, u.pos) <= limit);
            if !worth {
                self.disengage_until = f.tick + 120;
                return None;
            }
        }
        // ---- orbs: absorb when the 0.75 s is safe
        if let Some(o) = self.safe_orb(f) {
            if !o.within(f.pos, d::ORB_RANGE - 3_000) {
                out.push(Effect::Steer { to: o });
            }
        }
        // ---- Crimson Night: pick or escape
        if f.ult_unlocked && self.ready(NIGHT, f.tick) {
            let pick = f.enemy_champions().any(|u| u.pos.within(f.pos, d::NIGHT_DASH_RANGE) && self.lethal(f, u, 0) && (Self::isolated(f, u) || f.p(STRATEGY) >= 60));
            let flee = f.hp * 100 < f.max_hp * 35 && me_danger > limit / 2 && self.charges == 0;
            if pick || flee {
                if flee {
                    self.disengage_until = f.tick + 240;
                }
                return Some(Order::Night);
            }
        }
        let target = self.hunt(f);
        // ---- Crimson Edge in melee on a killable champion (not if she'd be stunned before the burst)
        if let Some(u) = &target {
            let melee = u.pos.within(f.pos, d::BASIC_RANGE + u.radius + 3_000);
            let would_be_stopped = cc_close && self.charges == 0;
            let micro_ok = f.p(MICRO) < 50 || self.kill_share(f, u) >= 60;
            if melee && self.ready(EDGE, f.tick) && self.edge.is_none() && !would_be_stopped && micro_ok && self.kill_share(f, u) >= 45 {
                return Some(Order::Edge);
            }
        }
        // ---- the engage: Kunai Flip on a lethal or near-lethal target, with an out
        if let Some(u) = &target {
            // near-lethal: bolder athletes go in on less
            let near_lethal = self.lethal(f, u, -(20 + f.p(AGGRESSIVE) / 5));
            let tower_ok = f.enemy_tower_at(u.pos).map_or(true, |tw| tw.tanked) || self.lethal(f, u, 0);
            // an out that isn't the kunai itself (a Soul charge or Flash), and a landing she can stand
            let out_after = self.charges >= 1 || f.flash_ready;
            let survivable = self.danger_at(f, u.pos) <= Self::danger_limit(f, out_after) || self.lethal(f, u, 0) && self.charges >= 1;
            if self.ready(KUNAI, f.tick) && u.pos.within(f.pos, d::KUNAI_RANGE) && (near_lethal || Self::isolated(f, u)) && tower_ok && out_after && survivable {
                let aim = Self::aim_at(f, u);
                let end = f.pos.toward(aim, d::KUNAI_RANGE);
                if Self::kunai_target(f, f.pos, end).is_some_and(|first| first.id == u.id) {
                    return Some(Order::Kunai(aim));
                }
            }
        }
        // ---- calm: farm a wave or camp with the kunai and the edge
        if f.calm() {
            let farm: Vec<&Unit> = f.units.iter().filter(|u| f.skill_target(u) && u.farmable && u.pos.within(f.pos, d::KUNAI_RANGE)).collect();
            if let Some(u) = farm.iter().min_by_key(|u| (u.pos.d2(f.pos), u.id)) {
                if self.ready(KUNAI, f.tick) && f.enemy_tower_at(u.pos).is_none() {
                    return Some(Order::Kunai(u.pos));
                }
                if self.ready(EDGE, f.tick) && self.edge.is_none() && u.pos.within(f.pos, d::BASIC_RANGE + u.radius + 3_000) && farm.len() >= 2 {
                    return Some(Order::Edge);
                }
            }
        }
        None
    }

    /// An orb she can absorb safely (the 0.75 s inside its range shouldn't be too dangerous).
    pub fn safe_orb(&self, f: &Frame) -> Option<Point> {
        let limit = Self::danger_limit(f, self.has_escape(f));
        self.orbs
            .iter()
            .filter(|o| f.tick + 10 < o.until && o.pos.within(f.pos, 90_000))
            .filter(|o| self.danger_at(f, o.pos) <= limit && f.enemy_tower_at(o.pos).map_or(true, |tw| tw.tanked))
            .min_by_key(|o| o.pos.d2(f.pos))
            .map(|o| o.pos)
    }

    /// The Kunai recast, re-checked now: worth it, not into a tower unless it kills and she can leave, the landing
    /// survivable with an out, and never into a parry. A target that Flashed is chased only once it lands in reach.
    pub fn recast_ok(&self, f: &Frame, u: &Unit) -> bool {
        if u.parry || Self::effective_hp(u).is_none() {
            return false;
        }
        if !u.champion {
            // a minion or creep: farming when calm, or crossing a wall to get out
            return f.calm() || f.tick < self.disengage_until && self.danger_at(f, u.pos) < self.danger_at(f, f.pos);
        }
        if !u.pos.within(f.pos, 90_000) {
            return false; // it ran or Flashed away: hold the mark
        }
        let lethal = self.lethal(f, u, 0);
        if !(lethal || Self::isolated(f, u) && self.kill_share(f, u) >= 50) {
            return false;
        }
        let escape = self.charges > 0 || f.flash_ready;
        if let Some(tw) = f.enemy_tower_at(u.pos) {
            if !tw.tanked && !(lethal && self.charges > 0) {
                return false;
            }
        }
        self.danger_at(f, u.pos) <= Self::danger_limit(f, escape) || lethal && escape
    }
}
