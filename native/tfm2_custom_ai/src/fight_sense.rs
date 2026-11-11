//! Fight Sense, host side (round 163): builds one team's [`Sense`] from the simulation, once per tick per
//! team, and shares it between every caller in that tick. Memory (who used Flash, who is missing, last
//! positions for velocity) lives per match and team and only learns from what that team can see.
//!
//! Nothing calls this in game yet: Akane is the first planned user, and other kits adopt it one by one.
//! The scores are in `fight_sense_core.rs`.

#![allow(dead_code)] // the reader ships before its first caller (round 163)

use crate::fight_sense_core::{tool, Creep, CreepKind, Memory, Protection, Seen, Sense, Terrain, Tower, Unit, TOWER_RANGE, STEALTH_PROBE};
use crate::{champions_shared, match_key, walls};
use mod_api_stable::{EntityKindV1 as K, SkillSlotV1, StableSim};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

/// Snapshots kept at once (one per match, tick and team).
const CACHE_SIZE: usize = 8;
/// Matches whose memory is kept.
const MEMORY_MATCHES: usize = 16;

type Key = (u64, u64, usize);

/// A small keyed cache: the first caller for a key builds, the rest share. `builds` counts builds (tests).
pub(crate) struct Cache<T> {
    entries: Vec<(Key, Arc<T>)>,
    pub(crate) builds: u64,
}

impl<T> Cache<T> {
    pub(crate) const fn new() -> Self {
        Self { entries: Vec::new(), builds: 0 }
    }
    pub(crate) fn get_or_build(&mut self, key: Key, build: impl FnOnce() -> T) -> Arc<T> {
        if let Some(hit) = self.entries.iter().find(|e| e.0 == key) {
            return hit.1.clone();
        }
        let fresh = Arc::new(build());
        self.builds += 1;
        // an older tick of the same match and team is never asked for again
        self.entries.retain(|e| !(e.0 .0 == key.0 && e.0 .2 == key.2));
        if self.entries.len() >= CACHE_SIZE {
            self.entries.remove(0);
        }
        self.entries.push((key, fresh.clone()));
        fresh
    }
}

/// Per match and team: what that team has learned.
#[derive(Default)]
struct TeamMemory {
    tick: u64,
    memory: Memory,
    /// Last position and tick of each champion, for velocity.
    last_pos: BTreeMap<usize, ((i64, i64), u64)>,
    /// The last visible view of each enemy, kept while it is out of sight.
    ghosts: BTreeMap<usize, Unit>,
}

static CACHE: Mutex<Cache<Sense>> = Mutex::new(Cache::new());
static MEMORY: Mutex<Vec<((u64, usize), TeamMemory)>> = Mutex::new(Vec::new());

/// `team`'s view of the fight this tick, built at most once per tick.
pub fn read(sim: &StableSim<'_>, team: usize) -> Arc<Sense> {
    let key = (match_key(sim), sim.tick() as u64, team);
    let mut cache = match CACHE.lock() {
        Ok(c) => c,
        Err(e) => e.into_inner(),
    };
    cache.get_or_build(key, || build(sim, team))
}

fn roster_tools(name: &str) -> ([u8; 3], i64) {
    use crate::roster::Kind;
    let Some(def) = crate::roster::def(name) else { return ([0; 3], crate::farming_data::attack_range(name)) };
    let mut tools = [0u8; 3];
    for s in def.skills {
        let i = match s.slot {
            SkillSlotV1::Skill => 0,
            SkillSlotV1::Skill2 => 1,
            SkillSlotV1::Ult => 2,
            _ => continue,
        };
        let pairs = [
            (Kind::Dash, tool::DASH),
            (Kind::Blink, tool::BLINK),
            (Kind::Cc, tool::CC),
            (Kind::Heal, tool::HEAL),
            (Kind::Shield, tool::SHIELD),
            (Kind::Projectile, tool::DAMAGE),
            (Kind::Line, tool::DAMAGE),
            (Kind::Aoe, tool::DAMAGE),
            (Kind::Zone, tool::DAMAGE),
        ];
        for (k, bit) in pairs {
            if s.is(k) {
                tools[i] |= bit;
            }
        }
    }
    (tools, def.basic.range)
}

fn build(sim: &StableSim<'_>, team: usize) -> Sense {
    let tick = sim.tick() as u64;
    let key = (match_key(sim), team);
    let all = champions_shared(sim);

    // cooldowns by champion id
    let mut ready: BTreeMap<usize, [bool; 3]> = BTreeMap::new();
    for i in 0..sim.player_count() {
        let Some(p) = sim.player_at(i) else { continue };
        let Some(c) = p.champion() else { continue };
        if let Some((_, s1, s2, u)) = p.cooldowns() {
            let unlocked = c.cooldown(SkillSlotV1::Ult).is_some_and(|x| x.unlocked) || all.iter().any(|a| a.id == c.id() && a.has("ult_learned"));
            ready.insert(c.id(), [s1 == 0, s2 == 0, u == 0 && unlocked]);
        }
    }

    let mut memories = match MEMORY.lock() {
        Ok(m) => m,
        Err(e) => e.into_inner(),
    };
    let slot = match memories.iter().position(|m| m.0 == key) {
        Some(i) => i,
        None => {
            if memories.len() >= MEMORY_MATCHES {
                memories.remove(0);
            }
            memories.push((key, TeamMemory::default()));
            memories.len() - 1
        }
    };
    let mem = &mut memories[slot].1;
    if tick < mem.tick {
        *mem = TeamMemory::default(); // a new match with the same champions (or a replay from the start)
    }
    mem.tick = tick;

    let visible_allies: Vec<(i64, i64)> = all.iter().filter(|c| c.team == team).map(|c| (c.x, c.y)).collect();
    let mut units = Vec::with_capacity(all.len());
    for c in all.iter() {
        let pos = (c.x, c.y);
        let seen = c.team == team || sim.is_visible(team, c.id);
        let vel = match mem.last_pos.get(&c.id) {
            Some(&(p, t)) if tick > t && tick - t <= 10 => ((pos.0 - p.0) / (tick - t) as i64, (pos.1 - p.1) / (tick - t) as i64),
            _ => (0, 0),
        };
        if seen {
            mem.last_pos.insert(c.id, (pos, tick));
        }
        if !seen {
            // fog: the team only knows what it saw last
            if let Some(g) = mem.ghosts.get(&c.id) {
                let mut g = g.clone();
                g.visible = false;
                g.vel = (0, 0);
                g.flash_ready = mem.memory.flash_ready(c.id, tick);
                g.protection.invisible = visible_allies.iter().any(|a| crate::fight_sense_core::within(*a, pos, STEALTH_PROBE)) && !walls::bush_at(pos.0, pos.1);
                units.push(g);
            }
            continue;
        }
        let entity = sim.get_entity(c.id);
        let stat = entity.as_ref().map(|e| e.stat()).unwrap_or_default();
        let shield = entity.as_ref().map_or(0, |e| e.shield()) as i64;
        let (tools, range) = roster_tools(&c.name);
        let mut protection = Protection::default();
        for b in &c.buffs {
            protection.cc_immune |= b.cc_immune;
            protection.damage_reduce += b.damaged_reduce as i64;
            protection.skill_reduce += b.skill_damaged_reduce as i64;
            protection.basic_reduce += b.base_attack_damaged_reduce as i64;
        }
        protection.damage_reduce = protection.damage_reduce.min(100);
        protection.skill_reduce = protection.skill_reduce.min(100);
        protection.basic_reduce = protection.basic_reduce.min(100);
        protection.v1_guard = c.has("v1_guard") && !c.has("v1_parry_cd");
        // an ally hidden from an enemy standing next to it, outside a bush, is stealthed
        if c.team == team {
            protection.invisible = all.iter().any(|e| e.team != team && crate::fight_sense_core::within((e.x, e.y), pos, STEALTH_PROBE))
                && !sim.is_visible(1 - team.min(1), c.id)
                && !walls::bush_at(pos.0, pos.1);
        }
        let u = Unit {
            id: c.id,
            team: c.team,
            name: c.name.clone(),
            hp: c.hp as i64,
            max_hp: c.max_hp as i64,
            shield,
            armor: stat.defence as i64,
            mr: stat.magic_resistance as i64,
            attack: stat.attack.max(c.attack) as i64,
            magic_power: stat.magic_power as i64,
            move_speed: stat.move_speed as i64,
            attack_range: range,
            attack_period: 0,
            pos,
            vel,
            visible: true,
            last_seen: Some(Seen { pos, tick }),
            flash_ready: !c.has("flash_cd"),
            ready: ready.get(&c.id).copied().unwrap_or([false; 3]),
            tools,
            protection,
            stunned: c.stunned,
        };
        if c.team != team {
            mem.ghosts.insert(c.id, u.clone());
        }
        units.push(u);
    }
    mem.memory.observe(team, &units, tick);

    // creeps the team can see (its own always)
    let mut creeps = Vec::new();
    for i in 0..sim.entity_count() {
        let Some(e) = sim.entity_at(i) else { continue };
        let kind = match e.kind() {
            Some(K::Minion) => CreepKind::Minion,
            Some(K::Jungle) => CreepKind::Jungle,
            Some(K::Epic) => CreepKind::Epic,
            Some(K::Serpen) => CreepKind::Serpen,
            _ => continue,
        };
        if !e.is_alive() || (e.team() != team && !sim.is_visible(team, e.id())) {
            continue;
        }
        let (x, y) = e.pos();
        let (hp, max_hp) = e.hp();
        creeps.push(Creep { id: e.id(), team: e.team(), kind, pos: (x as i64, y as i64), hp: hp as i64, max_hp: max_hp as i64 });
    }

    let mut towers = Vec::new();
    for i in 0..sim.tower_count() {
        let Some(t) = sim.get_entity(sim.tower_id_at(i)) else { continue };
        let (x, y) = t.pos();
        let pos = (x as i64, y as i64);
        let mut minions_in_range = [0u32; 2];
        for c in creeps.iter().filter(|c| c.kind == CreepKind::Minion && c.team < 2) {
            if crate::fight_sense_core::within(c.pos, pos, TOWER_RANGE) {
                minions_in_range[c.team] += 1;
            }
        }
        towers.push(Tower {
            id: t.id(),
            team: t.team(),
            pos,
            range: TOWER_RANGE,
            alive: t.is_alive(),
            damage: t.stat().attack as i64,
            period: 0,
            minions_in_range,
        });
    }

    let (wall_grid, bush_grid, _) = walls::snapshot();
    Sense {
        tick,
        team,
        units,
        towers,
        creeps,
        terrain: Terrain { walls: wall_grid, bushes: bush_grid },
        memory: mem.memory.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_callers_in_one_tick_share_one_build_and_the_next_tick_rebuilds() {
        let mut cache: Cache<u64> = Cache::new();
        let a = cache.get_or_build((7, 100, 0), || 1);
        let b = cache.get_or_build((7, 100, 0), || 2);
        assert!(Arc::ptr_eq(&a, &b));
        assert_eq!((cache.builds, *b), (1, 1));
        // the other team builds its own view
        cache.get_or_build((7, 100, 1), || 3);
        assert_eq!(cache.builds, 2);
        // next tick: one new build, and the old tick of that team is dropped
        let c = cache.get_or_build((7, 101, 0), || 4);
        assert_eq!((cache.builds, *c), (3, 4));
        assert_eq!(cache.entries.iter().filter(|e| e.0 .0 == 7 && e.0 .2 == 0).count(), 1);
        // the cache stays small across many matches
        for m in 0..40 {
            cache.get_or_build((m, 1, 0), || 0);
        }
        assert!(cache.entries.len() <= CACHE_SIZE);
    }

    #[test]
    fn every_roster_champion_gets_tools_from_its_kinds() {
        // Kratos has dashes, Omen a blink (both in rules/champions.json)
        let (kratos, _) = roster_tools("tfm2_custom_kratos");
        assert!(kratos.iter().any(|t| t & (tool::DASH | tool::BLINK) != 0), "{kratos:?}");
        let (unknown, range) = roster_tools("no_such_champion");
        assert_eq!(unknown, [0; 3]);
        assert!(range >= 0);
    }
}
