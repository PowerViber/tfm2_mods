//! Fight Sense: a shared, host-free read of a fight and the scores built on it (round 163).
//!
//! `fight_sense.rs` reads the simulation once per tick per team into a [`Sense`]; any champion's AI can then
//! ask the same questions of it: how much does it take to kill that target, what is waiting at that spot,
//! where can I get out to, who is missing and could be here. Nothing here touches the host, so the Labs can
//! run it on their own frames (it needs `athlete_ai_core.rs` and `game_sense_core.rs` next to it).
//!
//! The athlete matters only through [`Profile`]: aggression lowers the lethal margin, caution lowers the
//! damage it will accept, skill accuracy sets how far aims lead, Map Awareness sets how long and how far a
//! missing enemy stays a worry, Micro sets how many escape spots are tried. Combat numbers never change.
//!
//! Estimates that the engine doesn't expose (skill burst, CC length, reach of a dash) are named constants
//! below; the armour formula is the one the repo's viability models use (`raze_viability::mitigate`),
//! still to be confirmed against the engine (docs/engine-athlete-stats.md).

#![allow(dead_code)] // ships before its first caller (round 163)

use crate::athlete_ai_core::Profile;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The playable map, 0..MAP on both axes.
pub const MAP: i64 = 960_000;
/// One wall/bush cell, and the grid's width in cells (the same grid as `lib.rs` `walls`).
pub const CELL: i64 = 32_000;
pub const GRID: i64 = 30;
/// Ticks per second.
pub const SECOND: u64 = 60;
/// Tower attack range (the value flash.rs and Kratos use).
pub const TOWER_RANGE: i64 = 85_000;
/// The global Flash: reach and cooldown (flash.rs).
pub const FLASH_RANGE: i64 = 40_000;
pub const FLASH_CD: u64 = 7_200;
/// Reach estimates for a ready dash or blink when the roster gives no number.
pub const DASH_REACH: i64 = 30_000;
pub const BLINK_REACH: i64 = 40_000;
/// Extra burst a kill needs, in percent, for each of the target's ready outs.
pub const MARGIN_FLASH: i64 = 25;
pub const MARGIN_MOBILITY: i64 = 15;
pub const MARGIN_SUSTAIN: i64 = 15;
/// A ready damaging skill's burst estimate: this share of attack plus this share of magic power.
pub const SKILL_BURST_ATTACK_PCT: i64 = 100;
pub const SKILL_BURST_AP_PCT: i64 = 80;
/// A ready CC skill's length estimate.
pub const SKILL_CC_TICKS: u64 = 60;
/// Default time between basic attacks when a unit doesn't say.
pub const ATTACK_PERIOD: u64 = 60;
/// A missing enemy's possible reach never grows past this.
pub const MISSING_REACH_CAP: i64 = 200_000;
/// Base radius within which missing enemies are a worry (scaled by Map Awareness).
pub const THREAT_RADIUS: i64 = 120_000;
/// How far an enemy can see a unit that isn't in a bush (for bush scoring).
pub const SIGHT: i64 = 60_000;
/// A hidden unit this close to an enemy that should see it counts as invisible rather than in the fog.
pub const STEALTH_PROBE: i64 = 30_000;

/// Skill tools per slot (bit set), taken from the roster's kinds.
pub mod tool {
    pub const DASH: u8 = 1;
    pub const BLINK: u8 = 2;
    pub const CC: u8 = 4;
    pub const HEAL: u8 = 8;
    pub const SHIELD: u8 = 16;
    /// Projectile, Line, Aoe or Zone: it deals damage.
    pub const DAMAGE: u8 = 32;
}

pub type Point = (i64, i64);

pub fn d2(a: Point, b: Point) -> i128 {
    let (dx, dy) = ((a.0 - b.0) as i128, (a.1 - b.1) as i128);
    dx * dx + dy * dy
}
pub fn within(a: Point, b: Point, r: i64) -> bool {
    d2(a, b) <= (r as i128) * (r as i128)
}
pub fn dist(a: Point, b: Point) -> i64 {
    ((a.0 - b.0) as f64).hypot((a.1 - b.1) as f64) as i64
}
pub fn on_map(p: Point) -> bool {
    p.0 > 0 && p.1 > 0 && p.0 < MAP && p.1 < MAP
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Seen {
    pub pos: Point,
    pub tick: u64,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Protection {
    pub cc_immune: bool,
    /// Percent of all incoming damage removed (0..=100).
    pub damage_reduce: i64,
    /// Extra percent removed from skills / from basic attacks only.
    pub skill_reduce: i64,
    pub basic_reduce: i64,
    /// V1's parry is armed: an opening strike that can be parried does nothing.
    pub v1_guard: bool,
    pub invisible: bool,
}

/// One champion as the reading team knows it. An enemy out of sight keeps what was last seen
/// (`visible` false, `pos` its last-seen spot).
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Unit {
    pub id: usize,
    pub team: usize,
    pub name: String,
    pub hp: i64,
    pub max_hp: i64,
    pub shield: i64,
    pub armor: i64,
    pub mr: i64,
    pub attack: i64,
    pub magic_power: i64,
    /// World units per second.
    pub move_speed: i64,
    pub attack_range: i64,
    /// Ticks between basic attacks (0 = [`ATTACK_PERIOD`]).
    pub attack_period: u64,
    pub pos: Point,
    /// World units per tick.
    pub vel: Point,
    pub visible: bool,
    pub last_seen: Option<Seen>,
    pub flash_ready: bool,
    /// Skill, skill 2, ultimate ready.
    pub ready: [bool; 3],
    /// [`tool`] bits for skill, skill 2, ultimate.
    pub tools: [u8; 3],
    pub protection: Protection,
    pub stunned: bool,
}

impl Unit {
    pub fn hp_pct(&self) -> i64 {
        if self.max_hp <= 0 { 100 } else { self.hp * 100 / self.max_hp }
    }
    /// Any ready slot with one of these tool bits.
    pub fn ready_tool(&self, bits: u8) -> bool {
        (0..3).any(|i| self.ready[i] && self.tools[i] & bits != 0)
    }
    pub fn ready_count(&self, bits: u8) -> i64 {
        (0..3).filter(|&i| self.ready[i] && self.tools[i] & bits != 0).count() as i64
    }
    fn period(&self) -> u64 {
        if self.attack_period == 0 { ATTACK_PERIOD } else { self.attack_period }
    }
    /// How far it can get within `ticks`: walking, plus a ready blink or dash, plus Flash.
    pub fn reach(&self, ticks: u64) -> i64 {
        let walk = self.move_speed * ticks as i64 / SECOND as i64;
        let tool = if self.ready_tool(tool::BLINK) {
            BLINK_REACH
        } else if self.ready_tool(tool::DASH) {
            DASH_REACH
        } else {
            0
        };
        walk + tool + if self.flash_ready { FLASH_RANGE } else { 0 } + self.attack_range
    }
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Tower {
    pub id: usize,
    pub team: usize,
    pub pos: Point,
    pub range: i64,
    pub alive: bool,
    /// Damage per shot.
    pub damage: i64,
    pub period: u64,
    /// Minions of each team inside its range (someone else for it to shoot).
    pub minions_in_range: [u32; 2],
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum CreepKind {
    Minion,
    Jungle,
    Epic,
    Serpen,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Creep {
    pub id: usize,
    pub team: usize,
    pub kind: CreepKind,
    pub pos: Point,
    pub hp: i64,
    pub max_hp: i64,
}

/// The wall and bush grids (index cy * GRID + cx).
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Terrain {
    pub walls: Vec<bool>,
    pub bushes: Vec<bool>,
}

impl Terrain {
    fn cell(grid: &[bool], p: Point) -> Option<bool> {
        if p.0 < 0 || p.1 < 0 || p.0 / CELL >= GRID || p.1 / CELL >= GRID {
            return None;
        }
        Some(grid.get(((p.1 / CELL) * GRID + p.0 / CELL) as usize).copied().unwrap_or(false))
    }
    /// Off the grid counts as a wall, like `walls::wall_at`.
    pub fn wall_at(&self, p: Point) -> bool {
        Self::cell(&self.walls, p).unwrap_or(true)
    }
    pub fn bush_at(&self, p: Point) -> bool {
        Self::cell(&self.bushes, p).unwrap_or(false)
    }
    /// Walking from `a` toward `b`: the last free point before the first wall (`walls::clip`'s steps).
    pub fn clip(&self, a: Point, b: Point) -> Point {
        let steps = (dist(a, b) / 3_000).max(1);
        let mut last = a;
        for i in 1..=steps {
            let p = (a.0 + (b.0 - a.0) * i / steps, a.1 + (b.1 - a.1) * i / steps);
            if self.wall_at(p) {
                return last;
            }
            last = p;
        }
        b
    }
    pub fn blocked(&self, a: Point, b: Point) -> bool {
        self.clip(a, b) != b
    }
}

/// What a team remembers: who used Flash when, and who has been out of sight since when. Only updated
/// from what the team itself saw.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Memory {
    pub flash_used: BTreeMap<usize, u64>,
    pub missing: BTreeMap<usize, Seen>,
    /// Flash state at the last observation, to catch the moment it goes down.
    pub flash_seen: BTreeMap<usize, bool>,
}

impl Memory {
    /// Update from this tick's view of the enemies of `team`. Visible units refresh; units that just went
    /// out of sight start being missing from their last-seen spot.
    pub fn observe(&mut self, team: usize, units: &[Unit], tick: u64) {
        for u in units.iter().filter(|u| u.team != team) {
            if u.visible {
                self.missing.remove(&u.id);
                if self.flash_seen.get(&u.id) == Some(&true) && !u.flash_ready {
                    self.flash_used.insert(u.id, tick);
                }
                self.flash_seen.insert(u.id, u.flash_ready);
            } else if let Some(seen) = u.last_seen {
                self.missing.entry(u.id).or_insert(seen);
            }
        }
        self.flash_used.retain(|_, t| tick < *t + FLASH_CD);
    }
    /// Whether an enemy's Flash is up, as far as the team knows: down for FLASH_CD after it was seen used.
    pub fn flash_ready(&self, id: usize, tick: u64) -> bool {
        self.flash_used.get(&id).map_or(true, |&t| tick >= t + FLASH_CD)
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum DamageType {
    Physical,
    Magic,
    True,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum HitKind {
    Basic,
    Skill,
}

/// What a kill on this target needs beyond its effective HP.
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Margin {
    /// Extra burst in percent of effective HP.
    pub pct: i64,
    /// CC-immune or invisible: not now.
    pub not_now: bool,
    /// V1's guard is up: don't open with a strike it can parry.
    pub no_parry_open: bool,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Danger {
    pub damage: i64,
    pub cc_ticks: u64,
    /// Enemy champions (and towers) that can reach the point in time.
    pub sources: u32,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum EscapeTool {
    /// Over walls.
    Blink { range: i64 },
    /// Stopped by walls.
    Dash { range: i64 },
    /// Walking faster for a while (percent of own speed).
    Speed { pct: i64, ticks: u64 },
    Flash,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Route {
    pub tool: EscapeTool,
    pub to: Point,
    pub danger: Danger,
    /// The danger there is within what the athlete accepts.
    pub safe: bool,
    pub out_of_tower: bool,
    pub score: i64,
}

/// One team's view of the fight at one tick.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Sense {
    pub tick: u64,
    pub team: usize,
    pub units: Vec<Unit>,
    pub towers: Vec<Tower>,
    pub creeps: Vec<Creep>,
    pub terrain: Terrain,
    pub memory: Memory,
}

/// Danger looked at when judging a landing spot: one second.
const LANDING_WINDOW: u64 = SECOND;

impl Sense {
    pub fn unit(&self, id: usize) -> Option<&Unit> {
        self.units.iter().find(|u| u.id == id)
    }
    pub fn allies_of(&self, team: usize) -> impl Iterator<Item = &Unit> {
        self.units.iter().filter(move |u| u.team == team && u.hp > 0)
    }
    pub fn enemies_of(&self, team: usize) -> impl Iterator<Item = &Unit> {
        self.units.iter().filter(move |u| u.team != team && u.hp > 0)
    }
    pub fn creeps_near(&self, p: Point, r: i64) -> impl Iterator<Item = &Creep> {
        self.creeps.iter().filter(move |c| c.hp > 0 && within(c.pos, p, r))
    }

    /// HP plus shield after armour or magic resist and damage reduction.
    pub fn effective_hp(target: &Unit, kind: DamageType, hit: HitKind) -> i64 {
        let raw = (target.hp + target.shield).max(0);
        let resist = match kind {
            DamageType::Physical => target.armor,
            DamageType::Magic => target.mr,
            DamageType::True => 0,
        }
        .max(0);
        let p = &target.protection;
        let extra = match hit {
            HitKind::Basic => p.basic_reduce,
            HitKind::Skill => p.skill_reduce,
        };
        let reduce = (p.damage_reduce + extra).clamp(0, 100);
        if reduce >= 100 {
            return i64::MAX;
        }
        raw * (100 + resist) / 100 * 100 / (100 - reduce)
    }

    /// Extra burst a kill needs because of the target's outs.
    pub fn escape_margin(target: &Unit) -> Margin {
        let mut pct = 0;
        if target.flash_ready {
            pct += MARGIN_FLASH;
        }
        if target.ready_tool(tool::DASH | tool::BLINK) {
            pct += MARGIN_MOBILITY;
        }
        if target.ready_tool(tool::HEAL | tool::SHIELD) {
            pct += MARGIN_SUSTAIN;
        }
        Margin {
            pct,
            not_now: target.protection.cc_immune || target.protection.invisible,
            no_parry_open: target.protection.v1_guard,
        }
    }

    /// Whether `burst` (the caller's own damage numbers) kills the target with its outs counted.
    /// Aggression shrinks the margin, caution grows it.
    pub fn lethal(burst: i64, target: &Unit, kind: DamageType, hit: HitKind, profile: Profile) -> bool {
        let m = Self::escape_margin(target);
        if m.not_now {
            return false;
        }
        let ehp = Self::effective_hp(target, kind, hit);
        if ehp == i64::MAX {
            return false;
        }
        let pct = m.pct * profile.lethal_margin_scale() / 100;
        burst as i128 * 100 >= ehp as i128 * (100 + pct) as i128
    }

    /// How many of the target's allies are close enough to help it (last-seen spots for missing ones).
    pub fn isolation(&self, target: &Unit, radius: i64) -> usize {
        self.units
            .iter()
            .filter(|u| u.team == target.team && u.id != target.id && u.hp > 0 && within(u.pos, target.pos, radius))
            .count()
    }

    fn tower_tanked(&self, t: &Tower, victim_team: usize, me: Option<usize>) -> bool {
        t.minions_in_range[victim_team.min(1)] > 0
            || self
                .units
                .iter()
                .any(|u| u.team == victim_team && Some(u.id) != me && u.hp > 0 && u.visible && within(u.pos, t.pos, t.range))
    }

    /// What enemies of `team` (visible ones, and towers) can do at `p` within `ticks`. `me` is left out of
    /// the "someone else is tanking the tower" check.
    pub fn danger_at(&self, p: Point, ticks: u64, team: usize, me: Option<usize>) -> Danger {
        let mut out = Danger::default();
        for e in self.enemies_of(team).filter(|e| e.visible && !e.stunned) {
            if !within(e.pos, p, e.reach(ticks)) {
                continue;
            }
            out.sources += 1;
            let hits = (ticks / e.period()).max(1) as i64;
            out.damage += e.attack * hits;
            let skills = e.ready_count(tool::DAMAGE);
            out.damage += skills * (e.attack * SKILL_BURST_ATTACK_PCT + e.magic_power * SKILL_BURST_AP_PCT) / 100;
            out.cc_ticks += e.ready_count(tool::CC) as u64 * SKILL_CC_TICKS;
        }
        for t in self.towers.iter().filter(|t| t.alive && t.team != team && within(t.pos, p, t.range)) {
            if self.tower_tanked(t, team, me) {
                continue;
            }
            out.sources += 1;
            let period = if t.period == 0 { ATTACK_PERIOD } else { t.period };
            out.damage += t.damage * (ticks / period).max(1) as i64;
        }
        out
    }

    pub fn in_enemy_tower(&self, p: Point, team: usize) -> bool {
        self.towers.iter().any(|t| t.alive && t.team != team && within(t.pos, p, t.range))
    }
    pub fn in_allied_tower(&self, p: Point, team: usize) -> bool {
        self.towers.iter().any(|t| t.alive && t.team == team && within(t.pos, p, t.range))
    }

    /// Where each escape tool gets `me`, best first. Landing spots inside enemy tower range are left out.
    /// Bushes with no enemy in sight and allied towers score better; danger and nearby enemies worse.
    pub fn escape_routes(&self, me: &Unit, tools: &[EscapeTool], profile: Profile) -> Vec<Route> {
        let n = profile.route_candidates();
        let limit = profile.accepted_damage(Self::effective_hp(me, DamageType::Physical, HitKind::Basic).min(i64::MAX / 200));
        let mut out = Vec::new();
        for &t in tools {
            let reach = match t {
                EscapeTool::Blink { range } | EscapeTool::Dash { range } => range,
                EscapeTool::Speed { pct, ticks } => me.move_speed * pct / 100 * ticks as i64 / SECOND as i64,
                EscapeTool::Flash => FLASH_RANGE,
            };
            if reach <= 0 {
                continue;
            }
            for i in 0..n {
                let a = i as f64 * std::f64::consts::TAU / n as f64;
                let want = (me.pos.0 + (a.cos() * reach as f64) as i64, me.pos.1 + (a.sin() * reach as f64) as i64);
                let to = match t {
                    EscapeTool::Blink { .. } | EscapeTool::Flash => want,
                    EscapeTool::Dash { .. } | EscapeTool::Speed { .. } => self.terrain.clip(me.pos, want),
                };
                if !on_map(to) || self.terrain.wall_at(to) || to == me.pos || self.in_enemy_tower(to, me.team) {
                    continue;
                }
                let danger = self.danger_at(to, LANDING_WINDOW, me.team, Some(me.id));
                let nearest = self
                    .enemies_of(me.team)
                    .filter(|e| e.visible)
                    .map(|e| dist(e.pos, to))
                    .min()
                    .unwrap_or(MAP);
                let hidden = self.terrain.bush_at(to) && !self.enemies_of(me.team).any(|e| e.visible && within(e.pos, to, SIGHT));
                let score = -danger.damage - danger.cc_ticks as i64 * 2 + nearest.min(MAP) / 1_000
                    + if hidden { 40 } else { 0 }
                    + if self.in_allied_tower(to, me.team) { 60 } else { 0 };
                out.push(Route { tool: t, to, danger, safe: danger.damage <= limit, out_of_tower: true, score });
            }
        }
        out.sort_by(|a, b| b.score.cmp(&a.score).then(a.to.cmp(&b.to)));
        out
    }

    /// Enemies of `team` out of sight for less than the athlete's memory, whose possible reach since they
    /// were last seen covers `p` (capped by the athlete's threat radius).
    pub fn missing_threat(&self, p: Point, team: usize, profile: Profile) -> Vec<usize> {
        let memory = profile.missing_memory_ticks();
        let cap = profile.threat_radius(THREAT_RADIUS).min(MISSING_REACH_CAP);
        let mut out: Vec<usize> = self
            .memory
            .missing
            .iter()
            .filter(|(_, s)| self.tick.saturating_sub(s.tick) < memory)
            .filter(|(id, s)| {
                let Some(u) = self.unit(**id) else { return false };
                if u.team == team || u.hp <= 0 {
                    return false;
                }
                let gone = self.tick.saturating_sub(s.tick);
                let reach = (u.move_speed * gone as i64 / SECOND as i64 + u.attack_range).min(cap);
                within(s.pos, p, reach)
            })
            .map(|(id, _)| *id)
            .collect();
        out.sort_unstable();
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub(crate) fn unit(id: usize, team: usize, pos: Point) -> Unit {
        Unit {
            id,
            team,
            name: format!("u{id}"),
            hp: 1_000,
            max_hp: 1_000,
            armor: 0,
            mr: 0,
            attack: 100,
            magic_power: 0,
            move_speed: 30_000,
            attack_range: 20_000,
            pos,
            visible: true,
            last_seen: Some(Seen { pos, tick: 0 }),
            ..Unit::default()
        }
    }
    fn open() -> Terrain {
        Terrain { walls: vec![false; (GRID * GRID) as usize], bushes: vec![false; (GRID * GRID) as usize] }
    }
    fn sense(units: Vec<Unit>) -> Sense {
        Sense { tick: 1_000, team: 0, units, terrain: open(), ..Sense::default() }
    }
    fn athlete(values: [i64; 9], sense: [i64; 4]) -> Profile {
        Profile::new(values).with_sense(crate::game_sense_core::Ratings(sense))
    }
    fn with(index: usize, v: i64) -> Profile {
        let mut values = [50; 9];
        values[index] = v;
        athlete(values, [50; 4])
    }
    fn with_sense(index: usize, v: i64) -> Profile {
        let mut s = [50; 4];
        s[index] = v;
        athlete([50; 9], s)
    }

    #[test]
    fn margins_add_up_for_each_out() {
        let mut t = unit(1, 1, (0, 0));
        assert_eq!(Sense::escape_margin(&t).pct, 0);
        t.flash_ready = true;
        assert_eq!(Sense::escape_margin(&t).pct, 25);
        t.ready = [true, false, false];
        t.tools = [tool::DASH, 0, 0];
        assert_eq!(Sense::escape_margin(&t).pct, 40);
        t.ready = [true, true, false];
        t.tools = [tool::DASH, tool::HEAL, 0];
        assert_eq!(Sense::escape_margin(&t).pct, 55);
        // a tool on cooldown doesn't count
        t.ready = [false, true, false];
        assert_eq!(Sense::escape_margin(&t).pct, 40);
    }

    #[test]
    fn immunity_and_invisibility_mean_not_now_and_guard_means_no_parry_open() {
        let mut t = unit(1, 1, (0, 0));
        t.protection.cc_immune = true;
        assert!(Sense::escape_margin(&t).not_now);
        assert!(!Sense::lethal(1_000_000, &t, DamageType::True, HitKind::Skill, Profile::default()));
        t.protection.cc_immune = false;
        t.protection.invisible = true;
        assert!(Sense::escape_margin(&t).not_now);
        t.protection.invisible = false;
        t.protection.v1_guard = true;
        let m = Sense::escape_margin(&t);
        assert!(m.no_parry_open && !m.not_now);
    }

    #[test]
    fn effective_hp_counts_shield_resist_and_reduction() {
        let mut t = unit(1, 1, (0, 0));
        t.shield = 200;
        t.armor = 50;
        t.mr = 100;
        assert_eq!(Sense::effective_hp(&t, DamageType::True, HitKind::Basic), 1_200);
        assert_eq!(Sense::effective_hp(&t, DamageType::Physical, HitKind::Basic), 1_800);
        assert_eq!(Sense::effective_hp(&t, DamageType::Magic, HitKind::Basic), 2_400);
        t.protection.damage_reduce = 20;
        t.protection.skill_reduce = 30;
        assert_eq!(Sense::effective_hp(&t, DamageType::True, HitKind::Basic), 1_500);
        assert_eq!(Sense::effective_hp(&t, DamageType::True, HitKind::Skill), 2_400);
        t.protection.damage_reduce = 100;
        assert_eq!(Sense::effective_hp(&t, DamageType::True, HitKind::Basic), i64::MAX);
    }

    #[test]
    fn lethal_needs_the_margin_and_aggression_shrinks_it() {
        let mut t = unit(1, 1, (0, 0));
        t.flash_ready = true; // +25%
        let avg = Profile::default();
        assert!(!Sense::lethal(1_000, &t, DamageType::True, HitKind::Skill, avg));
        assert!(Sense::lethal(1_250, &t, DamageType::True, HitKind::Skill, avg));
        // 1,200 is enough for an aggressive athlete, not for a careful one
        assert!(Sense::lethal(1_200, &t, DamageType::True, HitKind::Skill, with(6, 100)));
        assert!(!Sense::lethal(1_200, &t, DamageType::True, HitKind::Skill, with(6, 20)));
    }

    #[test]
    fn isolation_counts_nearby_allies_only() {
        let s = sense(vec![unit(1, 1, (100_000, 100_000)), unit(2, 1, (130_000, 100_000)), unit(3, 1, (400_000, 100_000)), unit(4, 0, (110_000, 100_000))]);
        assert_eq!(s.isolation(s.unit(1).unwrap(), 50_000), 1);
        assert_eq!(s.isolation(s.unit(1).unwrap(), 400_000), 2);
    }

    fn tower_at(team: usize, pos: Point) -> Tower {
        Tower { id: 90, team, pos, range: TOWER_RANGE, alive: true, damage: 300, period: 60, minions_in_range: [0, 0] }
    }

    #[test]
    fn tower_danger_only_without_someone_else_to_shoot() {
        let mut s = sense(vec![unit(1, 0, (500_000, 500_000))]);
        s.towers.push(tower_at(1, (520_000, 500_000)));
        let alone = s.danger_at((500_000, 500_000), 60, 0, Some(1));
        assert_eq!((alone.damage, alone.sources), (300, 1));
        s.towers[0].minions_in_range = [3, 0];
        assert_eq!(s.danger_at((500_000, 500_000), 60, 0, Some(1)).damage, 0);
        // an allied champion in range tanks it too, but I don't tank it for myself
        s.towers[0].minions_in_range = [0, 0];
        s.units.push(unit(2, 0, (530_000, 500_000)));
        assert_eq!(s.danger_at((500_000, 500_000), 60, 0, Some(1)).damage, 0);
        // alone in range, unit 2 is the one it shoots
        s.units.remove(0);
        assert_eq!(s.danger_at((530_000, 500_000), 60, 0, Some(2)).damage, 300);
        // a dead tower does nothing
        s.towers[0].alive = false;
        assert_eq!(s.danger_at((530_000, 500_000), 60, 0, Some(2)).damage, 0);
    }

    #[test]
    fn danger_counts_reach_ready_skills_cc_and_flash() {
        let mut e = unit(9, 1, (600_000, 500_000)); // 100,000 away
        let mut s = sense(vec![unit(1, 0, (500_000, 500_000)), e.clone()]);
        // walks 30,000 + 20,000 range in a second: out of reach
        assert_eq!(s.danger_at((500_000, 500_000), 60, 0, Some(1)).sources, 0);
        e.flash_ready = true;
        e.ready = [true, true, false];
        e.tools = [tool::BLINK | tool::DAMAGE, tool::CC, 0];
        s.units[1] = e;
        let d = s.danger_at((500_000, 500_000), 60, 0, Some(1));
        assert_eq!(d.sources, 1);
        assert_eq!(d.damage, 100 + 100);
        assert_eq!(d.cc_ticks, SKILL_CC_TICKS);
        // stunned or unseen enemies are not counted here (missing ones are missing_threat's job)
        s.units[1].stunned = true;
        assert_eq!(s.danger_at((500_000, 500_000), 60, 0, Some(1)).sources, 0);
    }

    #[test]
    fn a_dash_stops_at_a_wall_and_a_blink_crosses_it() {
        let mut s = sense(vec![unit(1, 0, (400_000, 400_000))]);
        // a wall column at cx = 13 (416,000..448,000)
        for cy in 0..GRID {
            s.terrain.walls[(cy * GRID + 13) as usize] = true;
        }
        let me = s.unit(1).unwrap().clone();
        let p = with_sense(1, 100); // 32 candidates, one points straight east
        let dash = s.escape_routes(&me, &[EscapeTool::Dash { range: 60_000 }], p);
        assert!(dash.iter().all(|r| r.to.0 < 416_000), "a dash never ends past the wall");
        let blink = s.escape_routes(&me, &[EscapeTool::Blink { range: 60_000 }], p);
        assert!(blink.iter().any(|r| r.to.0 >= 448_000), "a blink lands beyond it");
        assert!(blink.iter().all(|r| !s.terrain.wall_at(r.to)));
    }

    #[test]
    fn routes_avoid_enemy_towers_and_like_allied_towers_and_bushes() {
        let mut s = sense(vec![unit(1, 0, (500_000, 500_000))]);
        s.towers.push(tower_at(1, (560_000, 500_000)));
        let me = s.unit(1).unwrap().clone();
        let routes = s.escape_routes(&me, &[EscapeTool::Flash], Profile::default());
        assert!(!routes.is_empty());
        assert!(routes.iter().all(|r| !s.in_enemy_tower(r.to, 0)));
        // an allied tower west: the west landing scores best
        s.towers.push(tower_at(0, (430_000, 500_000)));
        let best = s.escape_routes(&me, &[EscapeTool::Flash], Profile::default())[0];
        assert!(best.to.0 < 500_000 && s.in_allied_tower(best.to, 0));
        // with no towers, a bush wins
        s.towers.clear();
        let bush = (460_000 / CELL, 500_000 / CELL);
        s.terrain.bushes[(bush.1 * GRID + bush.0) as usize] = true;
        let best = s.escape_routes(&me, &[EscapeTool::Flash], Profile::default())[0];
        assert!(s.terrain.bush_at(best.to));
    }

    #[test]
    fn caution_lowers_the_danger_it_accepts() {
        let careful = athlete([10, 50, 90, 50, 50, 50, 50, 50, 50], [50; 4]);
        let bold = athlete([90, 50, 10, 50, 50, 50, 50, 50, 50], [50; 4]);
        assert!(careful.accepted_damage(1_000) < Profile::default().accepted_damage(1_000));
        assert!(bold.accepted_damage(1_000) > Profile::default().accepted_damage(1_000));
        assert_eq!(Profile::default().accepted_damage(1_000), 500);
    }

    #[test]
    fn flash_memory_records_use_and_expires() {
        let mut e = unit(9, 1, (0, 0));
        e.flash_ready = true;
        let mut m = Memory::default();
        m.observe(0, std::slice::from_ref(&e), 100);
        assert!(m.flash_ready(9, 100));
        e.flash_ready = false;
        m.observe(0, std::slice::from_ref(&e), 104);
        assert_eq!(m.flash_used.get(&9), Some(&104));
        assert!(!m.flash_ready(9, 104 + FLASH_CD - 1));
        assert!(m.flash_ready(9, 104 + FLASH_CD));
        m.observe(0, &[], 104 + FLASH_CD);
        assert!(m.flash_used.is_empty());
        // an enemy never seen with Flash up isn't marked as having used it
        let mut fresh = unit(8, 1, (0, 0));
        fresh.flash_ready = false;
        m.observe(0, &[fresh], 200);
        assert!(!m.flash_used.contains_key(&8));
    }

    #[test]
    fn missing_enemies_are_remembered_and_fade_by_map_awareness() {
        let mut e = unit(9, 1, (300_000, 300_000));
        e.visible = false;
        e.last_seen = Some(Seen { pos: (300_000, 300_000), tick: 1_000 });
        let mut s = sense(vec![unit(1, 0, (360_000, 300_000)), e.clone()]);
        s.memory.observe(0, &s.units.clone(), 1_000);
        assert_eq!(s.memory.missing.get(&9).map(|x| x.tick), Some(1_000));
        // 2 s later it could have walked 60,000 + 20,000 range: covers a spot 60,000 away
        s.tick = 1_120;
        assert_eq!(s.missing_threat((360_000, 300_000), 0, Profile::default()), vec![9]);
        assert!(s.missing_threat((600_000, 300_000), 0, Profile::default()).is_empty());
        // 5 s later: a low-awareness athlete has stopped worrying, a high one still does
        s.tick = 1_000 + 300;
        assert!(s.missing_threat((360_000, 300_000), 0, with_sense(2, 20)).is_empty());
        assert_eq!(s.missing_threat((360_000, 300_000), 0, with_sense(2, 80)), vec![9]);
        // seen again: no longer missing
        e.visible = true;
        s.memory.observe(0, &[e], 1_400);
        assert!(s.memory.missing.is_empty());
    }

    #[test]
    fn attribute_pairs_move_their_helpers_and_fifty_is_the_base() {
        let avg = Profile::default();
        assert_eq!(avg.lethal_margin_scale(), 100);
        assert!(with(6, 20).lethal_margin_scale() > with(6, 80).lethal_margin_scale());
        assert!(with(2, 80).accepted_damage(1_000) < with(2, 20).accepted_damage(1_000));
        assert!(with(0, 20).accepted_damage(1_000) < with(0, 80).accepted_damage(1_000));
        assert!(with(4, 80).aim_lead((100, 0), 30).0 > with(4, 20).aim_lead((100, 0), 30).0);
        assert_eq!(avg.threat_radius(THREAT_RADIUS), THREAT_RADIUS);
        assert!(with_sense(2, 80).threat_radius(100) > with_sense(2, 20).threat_radius(100));
        assert!(with_sense(2, 80).missing_memory_ticks() > with_sense(2, 20).missing_memory_ticks());
        assert_eq!(avg.route_candidates(), 16);
        assert_eq!(with_sense(1, 20).route_candidates(), 8);
        assert_eq!(with_sense(1, 80).route_candidates(), 32);
    }

    #[test]
    fn a_sense_round_trips_through_json() {
        let mut s = sense(vec![unit(1, 0, (1, 2)), unit(2, 1, (3, 4))]);
        s.towers.push(tower_at(1, (5, 6)));
        s.creeps.push(Creep { id: 7, team: 2, kind: CreepKind::Epic, pos: (7, 8), hp: 9, max_hp: 10 });
        s.memory.flash_used.insert(2, 5);
        let text = serde_json::to_string(&s).unwrap();
        assert_eq!(serde_json::from_str::<Sense>(&text).unwrap(), s);
    }
}
