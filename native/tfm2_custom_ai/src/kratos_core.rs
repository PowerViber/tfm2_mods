//! Kratos, the God of War: a deterministic core (snapshot in, effects out). `kratos.rs` builds the frame from the sim
//! and applies the effects; the Lab (`native/kratos_lab`) runs this same file.
//!
//!   passive Frost & Flame + Spartan Armor: he holds the Leviathan Axe (Frost stacks, 3 freeze) or the Blades of Chaos
//!           (burn, longer basic reach). He changes weapons only through S3 (Guard -> axe, Chain Dash -> Blades); every
//!           swap gives a shield of 5% max HP, at most 10% from swaps at once.
//!   S1      Heimdall's Sight: 1.5 s of CC immunity; the first hit from each enemy champion is dodged.
//!   S2      Leviathan Rampage: cleave (Frost, slow), sweep (burn; redirectable), frost-fire smash (knock-up), each
//!           advancing and stopped by terrain; cleanse, CC immunity and 25% damage reduction for the sequence.
//!   S3a     Spartan Guard: cleanse, shield, 1 s CC immunity; swaps to the axe.
//!   S3b     Chain Dash: a line throw; on a champion, minion or creep (never a tower) he dashes to it over any wall and
//!           strikes; swaps to the Blades. Guard->Dash within 3 s: longer throw. Dash->Guard: stronger Guard.
//!   Ult     Fenrir's Charge: the wolf appears far away and charges back to him; recast: he leaps into it and slams it
//!           back (knock-up on the return).
//!
//! CC immunity from S1, S2 and Guard shares one clock: windows never stack and he is immune for at most 3 s in any
//! 6 s. His skills, burns, freezes and Fenrir never touch towers; his basic attacks hit towers normally.
use crate::kratos_data as d;
use serde::{Deserialize, Serialize};

pub const MAP: i64 = 960_000;
pub const CELL: i64 = 32_000;
pub const CELLS: i64 = 30;

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
    /// Intersect a direction with the map bounds without bending its heading.
    pub fn ray_end(self, aim: Point, length: i64) -> Point {
        let end = self.toward(aim, length);
        let mut k = 1.0_f64;
        for (start, finish) in [(self.x, end.x), (self.y, end.y)] {
            let delta = finish - start;
            if finish < 1_000 && delta != 0 { k = k.min((1_000 - start) as f64 / delta as f64); }
            if finish > MAP - 1_000 && delta != 0 { k = k.min((MAP - 1_000 - start) as f64 / delta as f64); }
        }
        Point::new(self.x + ((end.x-self.x) as f64*k).round() as i64, self.y + ((end.y-self.y) as f64*k).round() as i64).clamp_map()
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
/// one of 8 headings (0 = east, clockwise in screen space)
pub fn heading(from: Point, to: Point) -> u8 {
    if from == to {
        return 0;
    }
    let mdeg = (((to.y - from.y) as f64).atan2((to.x - from.x) as f64).to_degrees() * 1000.0).round() as i64;
    (((mdeg + 22_500).rem_euclid(360_000)) / 45_000) as u8
}

// ------------------------------------------------------------------ frame

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Unit {
    pub id: usize,
    pub team: usize,
    pub pos: Point,
    pub hp: i64,
    pub max_hp: i64,
    pub radius: i64,
    pub champion: bool,
    pub tower: bool,
    pub visible: bool,
    pub targetable: bool,
    /// a minion or creep he may farm (calm, eligible)
    #[serde(default)]
    pub farmable: bool,
    /// under someone's hard CC (a stun or knock-up that isn't his own dash)
    #[serde(default)]
    pub stunned: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "aim", rename_all = "snake_case")]
pub enum Order {
    Sight,
    Rampage(Point),
    /// turn the Rampage's next advance toward a point
    Redirect(Point),
    Guard,
    Dash(Point),
    Fenrir(Point),
    /// the ult's recast: leap into Fenrir and slam it back
    Slam,
    /// Aimed recast; legacy `slam` retains its original default direction.
    SlamAim(Point),
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
    /// under someone else's hard CC (stun, knock-up, taunt...): no casts
    pub stunned: bool,
    pub ult_unlocked: bool,
    pub units: Vec<Unit>,
    #[serde(default)]
    pub terrain: Vec<bool>,
    pub profile: [i64; 13],
    /// units his accepted basic attacks hit since the last frame
    #[serde(default)]
    pub basic_hits: Vec<usize>,
    pub manual: bool,
    pub order: Option<Order>,
}
impl Frame {
    pub fn enemy(&self, u: &Unit) -> bool {
        u.team != self.team && u.visible && u.targetable && u.hp > 0
    }
    /// enemies his skills may hit: never a tower
    pub fn skill_target(&self, u: &Unit) -> bool {
        self.enemy(u) && !u.tower
    }
    pub fn enemy_champions(&self) -> impl Iterator<Item = &Unit> {
        self.units.iter().filter(move |u| u.champion && self.enemy(u))
    }
    pub fn profile(&self, i: usize) -> i64 {
        self.profile.get(i).copied().unwrap_or(50).clamp(0, 100)
    }
    pub fn terrain_at(&self, p: Point) -> bool {
        if !p.on_map() {
            return true;
        }
        self.terrain.get(((p.y / CELL) * CELLS + p.x / CELL) as usize).copied().unwrap_or(false)
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
    /// no enemy champion near him: his skills may farm
    pub fn calm(&self) -> bool {
        !self.enemy_champions().any(|u| u.pos.within(self.pos, d::AI_CALM_RANGE))
    }
    fn unit(&self, id: usize) -> Option<&Unit> {
        self.units.iter().find(|u| u.id == id)
    }
}

/// how far a direction point sits from where he is
pub const AIM_FAR: i64 = 1_000_000;
pub const PROFILE_AGGRESSIVE: usize = 6;
pub const PROFILE_CAUTIOUS: usize = 2;

// ------------------------------------------------------------------ effects

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case")]
pub enum Effect {
    /// `direct`: a single strike on one target (a V1 parry may cancel it)
    Damage { target: usize, amount: i64, direct: bool },
    Slow { target: usize, percent: i64, ticks: u64 },
    /// the axe's freeze (a stun)
    Freeze { target: usize, ticks: u64 },
    KnockUp { target: usize, ticks: u64 },
    Taunt { target: usize, ticks: u64 },
    /// Frost stacks shown on the target (0 clears)
    Frost { target: usize, stacks: i64, ticks: u64 },
    /// the burn's look on the target
    Burn { target: usize, ticks: u64 },
    /// his weapon changed (the bridge sets his basic range)
    Weapon { blades: bool },
    Shield { amount: i64, ticks: u64 },
    /// crowd-control immunity until `ticks` from now (replaces any shorter window; never stacks)
    Immune { ticks: u64 },
    /// damage reduction for `ticks`
    Reduce { percent: i64, ticks: u64 },
    Cleanse,
    /// Heimdall's Sight is up (the bridge's dodge window) for `ticks`
    Sight { ticks: u64 },
    /// he moves himself to `to` over `ticks` (an engine glide)
    Glide { to: Point, ticks: u64 },
    /// a hop across terrain in one tick (a blink over the wall: only inside the Chain Dash)
    Hop { to: Point },
    Fx { name: String, at: Point, ticks: u64, ambient: bool },
    Pose { name: &'static str, ticks: u64 },
    Log(String),
}

// ------------------------------------------------------------------ state

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Weapon {
    Axe,
    Blades,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Link {
    Guard,
    Dash,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Rampage {
    pub start: u64,
    /// the next stage to play (0 cleave, 1 sweep, 2 smash; 3 = done)
    pub stage: usize,
    /// the stage's swing has landed
    pub swung: bool,
    pub aim: Point,
    pub at: Point,
    pub hit: [Vec<usize>; 3],
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Fenrir {
    pub cast: u64,
    /// where the wolf is now
    pub pos: Point,
    pub from: Point,
    pub to: Point,
    /// where it appeared (the slam sends it back that way)
    pub origin: Point,
    /// 0 = charging in, 1 = slammed back
    pub leg: u8,
    pub leg_start: u64,
    pub leg_ticks: u64,
    pub hit: Vec<usize>,
    pub recast_until: u64,
    /// the slam lands when his leap ends
    pub slam_at: Option<u64>,
    pub slam_aim: Option<Point>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct DashPlan {
    pub target: usize,
    /// hop over a wall at this tick, to this point
    pub hop: Option<(u64, Point)>,
    /// then glide on to the landing spot
    pub after_hop: Option<(Point, u64)>,
    pub strike_at: u64,
}

#[derive(Clone, Debug, Serialize)]
pub struct Core {
    /// tick each skill is ready again: [Sight, Rampage, Guard, Dash, Fenrir]
    pub ready: [u64; 5],
    pub weapon: Weapon,
    /// swap shields: (amount, until)
    pub swap_shields: Vec<(i64, u64)>,
    /// (target, stacks, until)
    pub frost: Vec<(usize, i64, u64)>,
    /// (target, until): can't freeze again
    pub freeze_immune: Vec<(usize, u64)>,
    /// (target, until, damage per second, next tick)
    pub burns: Vec<(usize, u64, i64, u64)>,
    /// granted CC-immunity intervals [from, to)
    pub immune: Vec<(u64, u64)>,
    pub sight_until: u64,
    pub dodged: Vec<usize>,
    pub rampage: Option<Rampage>,
    pub link: Option<(Link, u64)>,
    pub dash: Option<DashPlan>,
    pub dash_connected: bool,
    pub fenrir: Option<Fenrir>,
    /// he's moving himself until (no new casts but the Rampage's redirect)
    pub busy_until: u64,
    pub rejection: String,
    pub last_skill: &'static str,
}
impl Default for Core {
    fn default() -> Self {
        Core {
            ready: [0; 5],
            weapon: Weapon::Axe,
            swap_shields: vec![],
            frost: vec![],
            freeze_immune: vec![],
            burns: vec![],
            immune: vec![],
            sight_until: 0,
            dodged: vec![],
            rampage: None,
            link: None,
            dash: None,
            dash_connected: false,
            fenrir: None,
            busy_until: 0,
            rejection: String::new(),
            last_skill: "",
        }
    }
}

pub const SIGHT: usize = 0;
pub const RAMPAGE: usize = 1;
pub const GUARD: usize = 2;
pub const DASH: usize = 3;
pub const FENRIR: usize = 4;

impl Core {
    pub fn respawn(&mut self, tick: u64) {
        let ready = self.ready;
        *self = Core::default();
        self.ready = ready.map(|r| r.max(tick));
    }
    pub fn ready(&self, slot: usize, tick: u64) -> bool {
        tick >= self.ready[slot]
    }
    pub fn immune_now(&self, tick: u64) -> bool {
        self.immune.iter().any(|&(a, b)| a <= tick && tick < b)
    }
    pub fn immune_until(&self, tick: u64) -> u64 {
        self.immune.iter().filter(|&&(a, b)| a <= tick && tick < b).map(|i| i.1).max().unwrap_or(tick)
    }
    /// CC-immune ticks inside [from, to)
    pub fn immune_in(&self, from: u64, to: u64) -> u64 {
        self.immune.iter().map(|&(a, b)| b.min(to).saturating_sub(a.max(from))).sum()
    }
    pub fn swap_shield_total(&self, tick: u64) -> i64 {
        self.swap_shields.iter().filter(|s| s.1 > tick).map(|s| s.0).sum()
    }
    pub fn frost_of(&self, id: usize, tick: u64) -> i64 {
        self.frost.iter().find(|f| f.0 == id && f.2 > tick).map_or(0, |f| f.1)
    }
    pub fn burning(&self, id: usize, tick: u64) -> bool {
        self.burns.iter().any(|b| b.0 == id && b.1 > tick)
    }
    pub fn link_active(&self, kind: Link, tick: u64) -> bool {
        self.link.is_some_and(|(k, until)| k == kind && tick < until)
    }

    /// Heimdall's Sight: is this hit dodged? The first hit from each enemy champion while it's up (it records it).
    pub fn dodge(&mut self, tick: u64, attacker: usize, enemy_champion: bool) -> bool {
        if !enemy_champion || tick >= self.sight_until || self.dodged.contains(&attacker) {
            return false;
        }
        self.dodged.push(attacker);
        true
    }

    /// Ask for `want` ticks of CC immunity from now. Windows never stack (an overlap only extends to the longer end),
    /// and he is never immune for more than the cap in any window. Returns the ticks granted from now (0 = refused).
    pub fn grant_immune(&mut self, tick: u64, want: u64) -> u64 {
        let cap = t(d::IMMUNE_CAP_TICKS);
        let window = t(d::IMMUNE_WINDOW_TICKS);
        let have = self.immune_until(tick);
        let goal = tick + want;
        let mut end = have;
        // extend one tick at a time while every window ending at the new tick stays within the cap
        while end < goal {
            let next = end + 1;
            let used = self.immune_in(next.saturating_sub(window), end) + 1;
            if used > cap {
                break;
            }
            end = next;
            if self.immune.last().is_some_and(|i| i.1 == end - 1) {
                self.immune.last_mut().unwrap().1 = end;
            } else {
                self.immune.push((end - 1, end));
            }
        }
        self.immune.retain(|i| i.1 + window > tick);
        end.saturating_sub(tick)
    }

    fn immune_effect(&mut self, tick: u64, want: u64, out: &mut Vec<Effect>) {
        let before = self.immune_until(tick);
        let got = self.grant_immune(tick, want);
        if got > 0 && tick + got > before {
            out.push(Effect::Immune { ticks: got });
        } else if got == 0 {
            out.push(Effect::Log(format!("immunity refused: {} of the last {} ticks used", self.immune_in(tick.saturating_sub(t(d::IMMUNE_WINDOW_TICKS)), tick), d::IMMUNE_WINDOW_TICKS)));
        }
    }

    pub fn step(&mut self, f: &Frame) -> Vec<Effect> {
        let mut out = vec![];
        if !f.alive {
            return out;
        }
        self.frost.retain(|x| x.2 > f.tick);
        self.freeze_immune.retain(|x| x.1 > f.tick);
        self.swap_shields.retain(|x| x.1 > f.tick);
        if self.link.is_some_and(|l| f.tick >= l.1) {
            self.link = None;
        }
        for &target in &f.basic_hits {
            if let Some(u) = f.unit(target).cloned() {
                if f.skill_target(&u) {
                    self.weapon_hit(f, &u, self.weapon, &mut out);
                }
            }
        }
        self.run_burns(f, &mut out);
        self.run_dash(f, &mut out);
        self.run_rampage(f, &mut out);
        self.run_fenrir(f, &mut out);
        let order = if f.manual { f.order } else { self.decide(f) };
        if let Some(o) = order {
            self.execute(f, o, &mut out);
        }
        out
    }

    // ---------------------------------------------------------------- passive

    /// the weapon's mark on a hit: the axe adds Frost (3 freeze), the Blades burn
    fn weapon_hit(&mut self, f: &Frame, u: &Unit, w: Weapon, out: &mut Vec<Effect>) {
        if u.tower {
            return;
        }
        match w {
            Weapon::Axe => self.add_frost(f, u.id, out),
            Weapon::Blades => self.add_burn(f, u.id, out),
        }
    }

    fn add_frost(&mut self, f: &Frame, id: usize, out: &mut Vec<Effect>) {
        if self.freeze_immune.iter().any(|x| x.0 == id) {
            return;
        }
        let stacks = self.frost_of(id, f.tick) + 1;
        self.frost.retain(|x| x.0 != id);
        if stacks >= d::PASSIVE_FROST_STACKS {
            out.push(Effect::Frost { target: id, stacks: 0, ticks: 0 });
            out.push(Effect::Freeze { target: id, ticks: t(d::PASSIVE_FREEZE_TICKS) });
            self.freeze_immune.push((id, f.tick + t(d::PASSIVE_FREEZE_IMMUNE_TICKS)));
            if let Some(u) = f.unit(id) {
                out.push(Effect::Fx { name: "krt_freeze".into(), at: u.pos, ticks: t(d::PASSIVE_FREEZE_TICKS), ambient: false });
            }
        } else {
            self.frost.push((id, stacks, f.tick + t(d::PASSIVE_FROST_TICKS)));
            out.push(Effect::Frost { target: id, stacks, ticks: t(d::PASSIVE_FROST_TICKS) });
        }
    }

    fn add_burn(&mut self, f: &Frame, id: usize, out: &mut Vec<Effect>) {
        let per_second = pct(d::PASSIVE_BURN_BASE, d::PASSIVE_BURN_AD_RATIO, f.attack);
        let until = f.tick + t(d::PASSIVE_BURN_TICKS);
        match self.burns.iter_mut().find(|b| b.0 == id) {
            // refreshes, never stacks
            Some(b) => {
                b.1 = until;
                b.2 = per_second;
            }
            None => self.burns.push((id, until, per_second, f.tick + 60)),
        }
        out.push(Effect::Burn { target: id, ticks: t(d::PASSIVE_BURN_TICKS) });
    }

    fn run_burns(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        for b in self.burns.iter_mut() {
            if f.tick >= b.3 && f.tick <= b.1 {
                b.3 = f.tick + 60;
                if f.unit(b.0).is_some_and(|u| u.hp > 0 && !u.tower) {
                    out.push(Effect::Damage { target: b.0, amount: b.2, direct: false });
                }
            }
        }
        self.burns.retain(|b| b.1 > f.tick && f.unit(b.0).is_some_and(|u| u.hp > 0));
    }

    /// He changes weapons only through S3. A real change gives a swap shield (5% max HP, at most 10% at once).
    fn swap_to(&mut self, f: &Frame, w: Weapon, out: &mut Vec<Effect>) {
        if self.weapon == w {
            return;
        }
        self.weapon = w;
        out.push(Effect::Weapon { blades: w == Weapon::Blades });
        let cap = f.max_hp * d::PASSIVE_SWAP_SHIELD_CAP_PCT / 100;
        let amount = (f.max_hp * d::PASSIVE_SWAP_SHIELD_PCT / 100).min(cap - self.swap_shield_total(f.tick));
        if amount > 0 {
            let ticks = t(d::PASSIVE_SWAP_SHIELD_TICKS);
            self.swap_shields.push((amount, f.tick + ticks));
            out.push(Effect::Shield { amount, ticks });
        }
        let tag = if w == Weapon::Axe { "krt_swap_axe" } else { "krt_swap_blades" };
        out.push(Effect::Fx { name: tag.into(), at: f.pos, ticks: 24, ambient: false });
    }

    // ---------------------------------------------------------------- orders

    pub fn execute(&mut self, f: &Frame, o: Order, out: &mut Vec<Effect>) {
        self.rejection.clear();
        let in_rampage = self.rampage.is_some();
        let reject = |s: &mut Self, why: &str| {
            s.rejection = why.into();
        };
        if f.stunned && !matches!(o, Order::Redirect(_)) {
            return reject(self, "crowd-controlled");
        }
        if (in_rampage || f.tick < self.busy_until) && !matches!(o, Order::Redirect(_)) {
            return reject(self, "mid-move");
        }
        match o {
            Order::Sight => {
                if !self.ready(SIGHT, f.tick) {
                    return reject(self, "Heimdall's Sight is on cooldown");
                }
                self.ready[SIGHT] = f.tick + t(d::SIGHT_COOLDOWN_TICKS);
                self.sight_until = f.tick + t(d::SIGHT_TICKS);
                self.dodged.clear();
                out.push(Effect::Sight { ticks: t(d::SIGHT_TICKS) });
                self.immune_effect(f.tick, t(d::SIGHT_TICKS), out);
                out.push(Effect::Fx { name: "krt_sight".into(), at: f.pos, ticks: t(d::SIGHT_TICKS), ambient: false });
                out.push(Effect::Pose { name: "skill1", ticks: 12 });
                self.last_skill = "sight";
            }
            Order::Rampage(aim) => {
                if !self.ready(RAMPAGE, f.tick) {
                    return reject(self, "Leviathan Rampage is on cooldown");
                }
                self.ready[RAMPAGE] = f.tick + t(d::RAMPAGE_COOLDOWN_TICKS);
                let total = t(3 * d::RAMPAGE_STAGE_TICKS);
                out.push(Effect::Cleanse);
                self.immune_effect(f.tick, total, out);
                out.push(Effect::Reduce { percent: d::RAMPAGE_REDUCE_PCT, ticks: total });
                let aim = if aim == f.pos { Point::new(f.pos.x + 1, f.pos.y) } else { aim };
                // a direction, not a spot: each advance keeps going the same way past the target
                let aim = f.pos.toward(aim, AIM_FAR);
                self.rampage = Some(Rampage { start: f.tick, stage: 0, swung: false, aim, at: f.pos, hit: Default::default() });
                self.last_skill = "rampage";
                self.run_rampage(f, out);
            }
            Order::Redirect(aim) => {
                match self.rampage.as_mut() {
                    Some(r) if r.stage < 3 && aim != r.at => r.aim = r.at.toward(aim, AIM_FAR),
                    _ => return reject(self, "no advance to redirect"),
                }
            }
            Order::Guard => {
                if !self.ready(GUARD, f.tick) {
                    return reject(self, "Spartan Guard is on cooldown");
                }
                self.ready[GUARD] = f.tick + t(d::GUARD_COOLDOWN_TICKS);
                let linked = self.link_active(Link::Dash, f.tick);
                self.swap_to(f, Weapon::Axe, out);
                out.push(Effect::Cleanse);
                let (shield, immune) = if linked {
                    (d::GUARD_LINKED_SHIELD_PCT, d::GUARD_LINKED_IMMUNE_TICKS)
                } else {
                    (d::GUARD_SHIELD_PCT, d::GUARD_IMMUNE_TICKS)
                };
                out.push(Effect::Shield { amount: f.max_hp * shield / 100, ticks: t(d::GUARD_SHIELD_TICKS) });
                self.immune_effect(f.tick, t(immune), out);
                if linked && self.dash_connected {
                    for u in f.enemy_champions().filter(|u| u.pos.within(f.pos, d::GUARD_TAUNT_RANGE + u.radius) && f.clip(f.pos,u.pos)==u.pos) {
                        out.push(Effect::Taunt {target:u.id,ticks:t(d::GUARD_TAUNT_TICKS)});
                    }
                }
                self.dash_connected=false;
                out.push(Effect::Fx { name: "krt_guard".into(), at: f.pos, ticks: t(d::GUARD_SHIELD_TICKS), ambient: false });
                out.push(Effect::Pose { name: "skill2", ticks: 14 });
                self.link = if linked { None } else { Some((Link::Guard, f.tick + t(d::LINK_WINDOW_TICKS))) };
                out.push(Effect::Log(format!("guard{}", if linked { " (linked)" } else { "" })));
                self.last_skill = "guard";
            }
            Order::Dash(aim) => {
                if !self.ready(DASH, f.tick) {
                    return reject(self, "Chain Dash is on cooldown");
                }
                self.ready[DASH] = f.tick + t(d::DASH_COOLDOWN_TICKS);
                self.dash_connected=false;
                let linked = self.link_active(Link::Guard, f.tick);
                let range = if linked { d::DASH_LINKED_RANGE } else { d::DASH_RANGE };
                self.swap_to(f, Weapon::Blades, out);
                let end = f.pos.toward(if aim == f.pos { Point::new(f.pos.x + 1, f.pos.y) } else { aim }, range);
                out.push(Effect::Fx { name: "krt_chain".into(), at: end, ticks: 12, ambient: false });
                out.push(Effect::Pose { name: "skill2", ticks: 12 });
                self.link = if linked { None } else { Some((Link::Dash, f.tick + t(d::LINK_WINDOW_TICKS))) };
                self.last_skill = "dash";
                match self.chain_target(f, end) {
                    Some(u) => self.start_dash(f, &u, out),
                    None => out.push(Effect::Log(format!("chain missed{}", if linked { " (linked)" } else { "" }))),
                }
            }
            Order::Fenrir(aim) => {
                if !f.ult_unlocked {
                    return reject(self, "ult not learned");
                }
                if !self.ready(FENRIR, f.tick) {
                    return reject(self, "Fenrir's Charge is on cooldown");
                }
                self.ready[FENRIR] = f.tick + t(d::FENRIR_COOLDOWN_TICKS);
                let aim = if aim == f.pos { Point::new(f.pos.x + 1, f.pos.y) } else { aim };
                let far = f.pos.ray_end(aim, d::FENRIR_DISTANCE);
                let mut spawn = far;
                // The spirit crosses walls in flight, but appears on open ground.
                for i in 0..=160 {
                    let candidate=far.toward(f.pos,(i*2_000).min(far.dist(f.pos) as i64));
                    if !f.terrain_at(candidate) {spawn=candidate;break;}
                }
                let through = f.pos.ray_end(spawn, -d::FENRIR_THROUGH_DISTANCE);
                let ticks = (spawn.dist(through) / d::FENRIR_SPEED.max(1) as f64).ceil().max(1.0) as u64;
                self.fenrir = Some(Fenrir {
                    cast: f.tick,
                    pos: spawn,
                    from: spawn,
                    to: through,
                    origin: spawn,
                    leg: 0,
                    leg_start: f.tick,
                    leg_ticks: ticks,
                    hit: vec![],
                    recast_until: f.tick + t(d::FENRIR_RECAST_TICKS),
                    slam_at: None,
                    slam_aim: None,
                });
                out.push(Effect::Fx { name: "krt_fenrir_spawn".into(), at: spawn, ticks: 24, ambient: false });
                out.push(Effect::Pose { name: "ult", ticks: 18 });
                self.last_skill = "fenrir";
            }
            Order::Slam | Order::SlamAim(_) => {
                let Some(w) = self.fenrir.as_ref() else { return reject(self, "Fenrir isn't out") };
                if w.leg != 0 || w.slam_at.is_some() || f.tick >= w.recast_until {
                    return reject(self, "the recast window is over");
                }
                let gap = w.pos.dist(f.pos) as i64;
                if gap > d::FENRIR_DASH_MAX + d::FENRIR_WIDTH {
                    return reject(self, "Fenrir is too far to leap into");
                }
                let land = f.clip(f.pos, f.pos.toward(w.pos, (gap - d::FENRIR_WIDTH).clamp(0, d::FENRIR_DASH_MAX)));
                let ticks = ((land.dist(f.pos) as i64) / d::DASH_SPEED).max(4) as u64;
                if land != f.pos {
                    out.push(Effect::Glide { to: land, ticks });
                }
                self.busy_until = f.tick + ticks;
                if let Some(w) = self.fenrir.as_mut() {
                    w.slam_at = Some(f.tick + ticks);
                    w.slam_aim = match o { Order::SlamAim(aim) if aim!=w.pos => Some(w.pos.toward(aim,AIM_FAR)), _=>None };
                }
                out.push(Effect::Pose { name: "ult", ticks: ticks + 10 });
            }
        }
    }

    // ---------------------------------------------------------------- Chain Dash

    /// The first champion, minion or creep the Blades hit on the line (never a tower or an ally).
    pub fn chain_target(&self, f: &Frame, end: Point) -> Option<Unit> {
        f.units
            .iter()
            .filter(|u| f.skill_target(u))
            .filter_map(|u| {
                let (dist, k) = u.pos.seg(f.pos, end);
                (dist <= (d::DASH_WIDTH + u.radius) as f64 && (k > 0.0 || u.pos.within(f.pos, u.radius + d::DASH_WIDTH))).then_some((k, u))
            })
            .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal).then(a.1.id.cmp(&b.1.id)))
            .map(|(_, u)| u.clone())
    }

    /// Dash to the hooked unit: through open ground a glide; across terrain, a glide to the wall, a hop over it, then
    /// a glide to the landing spot (next to the unit, on open ground).
    fn start_dash(&mut self, f: &Frame, u: &Unit, out: &mut Vec<Effect>) {
        let gap = (u.radius + 4_000).min(f.pos.dist(u.pos) as i64);
        let mut land = u.pos.toward(f.pos, gap);
        if f.terrain_at(land) {
            land = u.pos;
        }
        let speed = d::DASH_SPEED.max(1);
        let clear = f.clip(f.pos, land);
        let mut plan = DashPlan { target: u.id, hop: None, after_hop: None, strike_at: 0 };
        if clear == land {
            let ticks = ((f.pos.dist(land) as i64) / speed).max(2) as u64;
            out.push(Effect::Glide { to: land, ticks });
            plan.strike_at = f.tick + ticks;
        } else {
            // across a wall: up to the wall, over it, then on
            let t1 = ((f.pos.dist(clear) as i64) / speed).max(1) as u64;
            if clear != f.pos {
                out.push(Effect::Glide { to: clear, ticks: t1 });
            }
            // the first free point past the wall on the line
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
            out.push(Effect::Log(format!("chain dash over a wall to {}", u.id)));
        }
        self.busy_until = plan.strike_at;
        out.push(Effect::Fx { name: "krt_chain_hit".into(), at: u.pos, ticks: 12, ambient: false });
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
            out.push(Effect::Fx { name: "krt_dash_trail".into(), at: f.pos, ticks: 12, ambient: false });
            if let Some(u) = f.unit(p.target).filter(|u| f.skill_target(u)).cloned() {
                self.dash_connected=true;
                out.push(Effect::Damage { target: u.id, amount: pct(d::DASH_BASE, d::DASH_AD_RATIO, f.attack), direct: true });
                self.weapon_hit(f, &u, Weapon::Blades, out);
            }
        }
    }

    // ---------------------------------------------------------------- Leviathan Rampage

    fn run_rampage(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        let Some(mut r) = self.rampage.take() else { return };
        let stage_ticks = t(d::RAMPAGE_STAGE_TICKS);
        let move_ticks = stage_ticks / 2;
        while r.stage < 3 {
            let begin = r.start + r.stage as u64 * stage_ticks;
            if f.tick < begin {
                break;
            }
            if !r.swung && f.tick == begin {
                // advance (stopped by terrain), then swing when the step ends
                let goal = r.at.toward(r.aim, d::RAMPAGE_TRAVEL / 3);
                let to = f.clip(r.at, goal);
                if to != r.at {
                    out.push(Effect::Glide { to, ticks: move_ticks });
                }
                r.at = to;
                let pose: &'static str = match r.stage { 0 => "skill2", 1 => "skill2", _ => "ult" };
                out.push(Effect::Pose { name: pose, ticks: stage_ticks });
            }
            if !r.swung && f.tick >= begin + move_ticks {
                r.swung = true;
                self.swing(f, &mut r, out);
            }
            if f.tick >= begin + stage_ticks {
                r.stage += 1;
                r.swung = false;
                continue;
            }
            break;
        }
        if r.stage < 3 {
            self.rampage = Some(r);
        }
    }

    fn swing(&mut self, f: &Frame, r: &mut Rampage, out: &mut Vec<Effect>) {
        let stage = r.stage;
        let ahead = f.clip(r.at,r.at.toward(r.aim, d::RAMPAGE_SWING_RADIUS / 2));
        let (center, radius) = if stage == 2 { (r.at, d::RAMPAGE_SMASH_RADIUS) } else { (ahead, d::RAMPAGE_SWING_RADIUS) };
        let left = r.aim.x < r.at.x;
        let fx = match stage {
            0 => if left { "krt_cleave_l" } else { "krt_cleave" },
            1 => if left { "krt_sweep" } else { "krt_sweep_l" },
            _ => "krt_smash",
        };
        out.push(Effect::Fx { name: fx.into(), at: center, ticks: t(d::RAMPAGE_STAGE_TICKS / 2), ambient: false });
        let targets: Vec<Unit> = f
            .units
            .iter()
            .filter(|u| f.skill_target(u) && u.pos.within(center, radius + u.radius) && !r.hit[stage].contains(&u.id) && f.clip(r.at,u.pos)==u.pos)
            .cloned()
            .collect();
        for u in targets {
            r.hit[stage].push(u.id);
            match stage {
                0 => {
                    out.push(Effect::Damage { target: u.id, amount: pct(d::RAMPAGE_CLEAVE_BASE, d::RAMPAGE_CLEAVE_AD_RATIO, f.attack), direct: false });
                    out.push(Effect::Slow { target: u.id, percent: d::RAMPAGE_SLOW_PCT, ticks: t(d::RAMPAGE_SLOW_TICKS) });
                    self.weapon_hit(f, &u, Weapon::Axe, out);
                }
                1 => {
                    out.push(Effect::Damage { target: u.id, amount: pct(d::RAMPAGE_SWEEP_BASE, d::RAMPAGE_SWEEP_AD_RATIO, f.attack), direct: false });
                    self.weapon_hit(f, &u, Weapon::Blades, out);
                }
                _ => {
                    out.push(Effect::Damage { target: u.id, amount: pct(d::RAMPAGE_SMASH_BASE, d::RAMPAGE_SMASH_AD_RATIO, f.attack), direct: false });
                    out.push(Effect::KnockUp { target: u.id, ticks: t(d::RAMPAGE_KNOCK_UP_TICKS) });
                    self.weapon_hit(f, &u, Weapon::Axe, out);
                    self.weapon_hit(f, &u, Weapon::Blades, out);
                }
            }
        }
    }

    // ---------------------------------------------------------------- Fenrir

    fn run_fenrir(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        let Some(mut w) = self.fenrir.take() else { return };
        // the recast leap ends: the slam sends him back the way he came
        if w.slam_at.is_some_and(|at| f.tick >= at) && w.leg == 0 {
            w.slam_at = None;
            w.leg = 1;
            w.leg_start = f.tick;
            w.from = w.pos;
            w.to = w.pos.ray_end(w.slam_aim.unwrap_or(w.origin), d::FENRIR_RETURN_DISTANCE);
            // slammed, it flies back much faster than it charged in (like Ornn's ram)
            w.leg_ticks = (w.from.dist(w.to) / d::FENRIR_RETURN_SPEED.max(1) as f64).ceil().max(1.0) as u64;
            w.hit.clear();
            out.push(Effect::Fx { name: "krt_fenrir_slam".into(), at: w.pos, ticks: 18, ambient: false });
        }
        let elapsed = f.tick.saturating_sub(w.leg_start).min(w.leg_ticks);
        let prev = w.pos;
        w.pos = Point::new(
            w.from.x + (w.to.x - w.from.x) * elapsed as i64 / w.leg_ticks.max(1) as i64,
            w.from.y + (w.to.y - w.from.y) * elapsed as i64 / w.leg_ticks.max(1) as i64,
        );
        let hits: Vec<Unit> = f
            .units
            .iter()
            .filter(|u| f.skill_target(u) && !w.hit.contains(&u.id))
            .filter(|u| u.pos.seg(prev, w.pos).0 <= (d::FENRIR_WIDTH + u.radius) as f64)
            .cloned()
            .collect();
        for u in hits {
            w.hit.push(u.id);
            if w.leg == 0 {
                out.push(Effect::Damage { target: u.id, amount: pct(d::FENRIR_BASE, d::FENRIR_AD_RATIO, f.attack), direct: false });
                out.push(Effect::Slow { target: u.id, percent: d::FENRIR_SLOW_PCT, ticks: t(d::FENRIR_SLOW_TICKS) });
            } else {
                out.push(Effect::Damage { target: u.id, amount: pct(d::FENRIR_RETURN_BASE, d::FENRIR_RETURN_AD_RATIO, f.attack), direct: false });
                out.push(Effect::KnockUp { target: u.id, ticks: t(d::FENRIR_KNOCK_UP_TICKS) });
            }
        }
        let h = heading(w.from, w.to);
        out.push(Effect::Fx { name: format!("krt_fenrir_p{h}"), at: w.pos, ticks: 1, ambient: false });
        let leg_done = elapsed >= w.leg_ticks;
        let over = leg_done || (w.leg == 0 && f.tick >= w.recast_until && w.slam_at.is_none());
        if over {
            out.push(Effect::Fx { name: "krt_fenrir_fade".into(), at: w.pos, ticks: 18, ambient: false });
        } else {
            self.fenrir = Some(w);
        }
    }

    // ---------------------------------------------------------------- the athlete

    pub fn decide(&self, f: &Frame) -> Option<Order> {
        if f.stunned {
            return None;
        }
        // the Rampage steers its next advance at the nearest champion
        if let Some(r) = &self.rampage {
            let near = f.enemy_champions().filter(|u| u.pos.within(r.at, d::RAMPAGE_TRAVEL)).min_by_key(|u| (u.pos.d2(r.at), u.id));
            // only a real turn (more than about 15 degrees) is worth a redirect
            return near
                .filter(|u| r.stage < 3 && u.pos != r.at && r.at.toward(u.pos, AIM_FAR).d2(r.aim) > (AIM_FAR as i128 / 4).pow(2))
                .map(|u| Order::Redirect(u.pos));
        }
        if let Some(w) = &self.fenrir {
            if w.leg == 0 && w.slam_at.is_none() && f.tick < w.recast_until && w.pos.within(f.pos, d::FENRIR_DASH_MAX + d::FENRIR_WIDTH) {
                let best=f.enemy_champions().map(|u| {
                    let end=w.pos.ray_end(u.pos,d::FENRIR_RETURN_DISTANCE);
                    let n=f.enemy_champions().filter(|v|v.pos.seg(w.pos,end).0<=(d::FENRIR_WIDTH+v.radius) as f64).count();
                    (n,u)
                }).max_by_key(|(n,u)|(*n,std::cmp::Reverse(u.id)));
                let contact=w.pos.within(f.pos,d::FENRIR_WIDTH+4_000);
                if let Some((n,u))=best.filter(|(n,_)|*n>=2||(*n>=1&&contact)) {
                    let _=n;return Some(Order::SlamAim(u.pos));
                }
            }
        }
        if f.tick < self.busy_until {
            return None;
        }
        let near: Vec<&Unit> = f.enemy_champions().filter(|u| u.pos.within(f.pos, 120_000)).collect();
        let aggressive = f.profile(PROFILE_AGGRESSIVE);
        let hp_pct = f.hp * 100 / f.max_hp.max(1);
        // the ult down a line at a group
        if f.ult_unlocked && self.ready(FENRIR, f.tick) && self.fenrir.is_none() {
            let best = near
                .iter()
                .filter(|u| u.pos.within(f.pos, d::AI_ULT_RANGE))
                .map(|u| {
                    let end = f.pos.toward(u.pos, d::FENRIR_DISTANCE);
                    (near.iter().filter(|o| o.pos.seg(f.pos, end).0 <= (d::FENRIR_WIDTH + o.radius) as f64).count(), *u)
                })
                .max_by_key(|(n, u)| (*n, std::cmp::Reverse(u.id)));
            if let Some((n, u)) = best {
                let low = u.hp * 100 < u.max_hp * 40;
                if n >= 2 || (n >= 1 && (low || aggressive >= 80)) {
                    return Some(Order::Fenrir(u.pos));
                }
            }
        }
        let close = near.iter().filter(|u| u.pos.within(f.pos, 45_000)).count();
        // Heimdall's Sight when focused or about to be caught
        if self.ready(SIGHT, f.tick) && !self.immune_now(f.tick) && close >= 1 && (close >= 2 || hp_pct < 60) {
            return Some(Order::Sight);
        }
        // Dash -> Guard when diving into a crowd (or hurt)
        if self.link_active(Link::Dash, f.tick) && self.ready(GUARD, f.tick) && close >= 1 && (close >= 2 || hp_pct < 70) {
            return Some(Order::Guard);
        }
        let nearest = near.iter().min_by_key(|u| (u.pos.d2(f.pos), u.id)).copied();
        if let Some(u) = nearest {
            let dist = f.pos.dist(u.pos) as i64;
            let threshold = 45 - aggressive / 5; // bolder athletes engage while hurt
            let healthy = hp_pct >= threshold;
            // Guard -> Dash: the long engage
            if healthy && self.ready(GUARD, f.tick) && self.ready(DASH, f.tick) && dist > d::DASH_RANGE && dist <= d::DASH_LINKED_RANGE.min(d::AI_ENGAGE_RANGE + 10_000) {
                let end = f.pos.toward(u.pos, d::DASH_LINKED_RANGE);
                if self.chain_target(f, end).is_some_and(|t| t.id == u.id) {
                    return Some(Order::Guard);
                }
            }
            if self.ready(DASH, f.tick) && healthy {
                let range = if self.link_active(Link::Guard, f.tick) { d::DASH_LINKED_RANGE } else { d::DASH_RANGE };
                if dist > 25_000 && dist <= range {
                    let end = f.pos.toward(u.pos, range);
                    if self.chain_target(f, end).is_some_and(|t| t.id == u.id) {
                        return Some(Order::Dash(u.pos));
                    }
                }
            }
            if self.ready(RAMPAGE, f.tick) && dist <= d::RAMPAGE_RANGE + u.radius {
                return Some(Order::Rampage(u.pos));
            }
        }
        if self.ready(RAMPAGE, f.tick) {
            let around: Vec<&Unit> = f.units.iter().filter(|u| f.skill_target(u) && u.pos.within(f.pos, d::RAMPAGE_RANGE + u.radius)).collect();
            // 2+ enemies in reach in a fight, or a calm farm on a wave or camp
            let fight = around.len() >= 2 && around.iter().any(|u| u.champion);
            let farm = f.calm() && around.iter().filter(|u| u.farmable).count() >= 2;
            if fight || farm {
                let c = around.iter().find(|u| u.champion).or(around.first()).map(|u| u.pos).unwrap_or(f.pos);
                return Some(Order::Rampage(c));
            }
        }
        // a calm farm: the Chain Dash on a far camp or wave is movement too
        if f.calm() && self.ready(DASH, f.tick) {
            let far = f
                .units
                .iter()
                .filter(|u| f.skill_target(u) && u.farmable && u.pos.within(f.pos, d::DASH_RANGE) && !u.pos.within(f.pos, 35_000))
                .min_by_key(|u| (u.pos.d2(f.pos), u.id));
            if let Some(u) = far {
                if self.chain_target(f, f.pos.toward(u.pos, d::DASH_RANGE)).is_some_and(|t| t.id == u.id) {
                    return Some(Order::Dash(u.pos));
                }
            }
        }
        None
    }
}
