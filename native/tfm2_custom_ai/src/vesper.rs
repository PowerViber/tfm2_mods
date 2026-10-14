//! Vesper, the Rift Gauntlets: the stable-API adapter around `vesper_core`.
//!
//! Each tick it snapshots the fight, lets the core decide, then applies the core's commands: damage, crowd control,
//! forced movement, buffs and view effects. Mastery (the athlete's rank) only reaches the core's planner and the
//! visual layers chosen here. The athlete also tunes the Hexcore: which 6 of the 8 tech modules its hexorb slots
//! carry (derived from the athlete; `vesper_modules.txt` next to the mod overrides it).
use crate::mastery::{signature, Book, Record};
use crate::vesper_core::{Cc, Cmd, Core, Dmg, Fighter, Input, Mods};
use crate::vesper_data::{MODULES, MOVE_IDS, RANKS, STRESS_CAP, TOP_NAME};
use crate::vesper_math::modules_of;
use crate::{champions, fx_point, fx_unit, timed, walls, Champ};
use mod_api_stable::{AttackTypeV1, BuffV1, CcKindV1, CcV1, DamageTypeV1, StablePassive, StableSim};
use std::io::Write;

pub static BOOK: Book = Book::new("vesper");
pub const ID: &str = "tfm2_custom_vesper";

#[derive(Clone, Default)]
pub struct Vesper {
    owner: usize,
    core: Option<Core>,
    rank: usize,
    top: Option<usize>,
    rank_set: bool,
    athlete: Option<usize>,
    record: Record,
    sig: Option<String>,
    visual_buffs: Vec<String>,
    trace: Vec<String>,
    dropped: usize,
    last_tick: Option<usize>,
    dead: bool,
    modules: [usize; 6],
}

/// An athlete's Hexcore modules: `vesper_modules.txt` lines `<athlete> <module> x6` (ids or names), else derived.
pub fn modules_for(athlete: Option<usize>) -> [usize; 6] {
    let fallback = modules_of(athlete);
    let (Some(a), Some(dir)) = (athlete, crate::mod_dir()) else { return fallback };
    let Ok(text) = std::fs::read_to_string(dir.join("vesper_modules.txt")) else { return fallback };
    parse_modules(&text, a).unwrap_or(fallback)
}
pub fn parse_modules(text: &str, athlete: usize) -> Option<[usize; 6]> {
    for line in text.lines() {
        let mut it = line.split_whitespace();
        if it.next().and_then(|w| w.parse::<usize>().ok()) != Some(athlete) {
            continue;
        }
        let ids: Vec<usize> = it.filter_map(|w| w.parse::<usize>().ok().filter(|n| *n < 8).or_else(|| MODULES.iter().position(|m| m.id.eq_ignore_ascii_case(w)))).collect();
        let mut seen = ids.clone();
        seen.sort();
        seen.dedup();
        if ids.len() == 6 && seen.len() == 6 {
            return Some([ids[0], ids[1], ids[2], ids[3], ids[4], ids[5]]);
        }
    }
    None
}

/// The look of her pose overlays: 0-6 by rank, 7 Worldbreaker #4-10, 8 #1, 9 #3, 10 #2.
pub fn pose_look(rank: usize, top: Option<usize>) -> usize {
    if rank < 7 {
        return rank;
    }
    match top {
        Some(1) => 8,
        Some(2) => 10,
        Some(3) => 9,
        _ => 7,
    }
}

/// Her animated pose overlay: Shatter > swipe > rift punch > jab > block > guard.
pub fn pose_of(core: &Core, tick: usize) -> &'static str {
    if tick < core.shatter_until {
        "shatter"
    } else if tick < core.swipe_until {
        "swipe"
    } else if core.punching {
        "punch"
    } else if tick < core.jab_until {
        "jab"
    } else if tick < core.block_until {
        "block"
    } else {
        "outfit"
    }
}

/// The gauntlet HUD and rank layers shown on Vesper (buff names `vs_*`).
pub fn style_names(rank: usize, top: Option<usize>, core: &Core, tick: usize) -> Vec<String> {
    let l = pose_look(rank, top);
    let mut names = vec![format!("vs_{}_r{l}", pose_of(core, tick))];
    // the animated rank emblem at her top right (Top 10: the medallion with her number)
    names.push(if rank >= 7 { format!("vs_top{}", top.unwrap_or(10).clamp(1, 10)) } else { format!("vs_rank{rank}") });
    if rank >= 7 && top == Some(1) {
        names.push("vs_back_top1".into());
    } else if rank >= 6 {
        names.push("vs_back_wound".into());
    }
    names.push(if core.left_broken(tick) { "vs_orbs_x".into() } else { format!("vs_orbs{}_{}", core.pure_ready(tick), core.sockets()) });
    names.push(if core.right_broken(tick) { "vs_hex_x".into() } else { format!("vs_hex{}_{}", core.hex_ready(tick), core.hex_slots(tick)) });
    names.push(if core.right_broken(tick) {
        "vs_core_x".into()
    } else if core.awakened(tick) {
        format!("vs_core_aw{}", (core.corruption / 20).clamp(0, 4))
    } else {
        format!("vs_core{}", ((core.completion - 70) / 5).clamp(0, 6))
    });
    names.push(format!("vs_stress{}", (core.stress * 10 / STRESS_CAP).clamp(0, 10)));
    names
}

impl Vesper {
    fn sync_visuals(&mut self, sim: &mut StableSim<'_>, m: &Champ, core: &Core) {
        let want = style_names(self.rank, self.top, core, sim.tick());
        if want == self.visual_buffs && want.iter().all(|n| m.has(n)) {
            return;
        }
        for name in &self.visual_buffs {
            if !want.contains(name) {
                sim.entity_remove_buff(m.id, name);
            }
        }
        for name in &want {
            if !self.visual_buffs.contains(name) || !m.has(name) {
                sim.add_buff(m.id, &timed(name, 5_184_000));
            }
        }
        self.visual_buffs = want;
    }
    fn log(&mut self, line: String) {
        if self.trace.len() < 96 {
            self.trace.push(line);
        } else {
            self.dropped += 1;
        }
    }
    fn flush(&mut self, sim: &StableSim<'_>, force: bool) {
        if self.trace.is_empty() || (!force && !sim.tick().is_multiple_of(120)) {
            return;
        }
        let mut lines = std::mem::take(&mut self.trace);
        if self.dropped > 0 {
            lines.push(format!("{} more events omitted by the bounded trace", self.dropped));
            self.dropped = 0;
        }
        if sim.sim_origin().is_some_and(|o| o.kind == mod_api_stable::SimOriginKindV1::ServerPresim.code()) {
            return;
        }
        let fingerprint = lines.iter().flat_map(|s| s.bytes()).fold(0xcbf2_9ce4_8422_2325u64, |h, b| (h ^ b as u64).wrapping_mul(0x100_0000_01b3));
        if !BOOK.first(format!("vs-trace:{}:{}:{}:{fingerprint}", sim.seed(), self.owner, sim.tick())) {
            return;
        }
        if let Some(dir) = crate::mod_dir() {
            let path = dir.join("vesper_damage_log.txt");
            if std::fs::metadata(&path).is_ok_and(|m| m.len() >= 8 * 1024 * 1024) {
                let _ = std::fs::rename(&path, path.with_extension("previous.txt"));
            }
            if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
                let _ = writeln!(f, "{}", lines.join("\n"));
            }
        }
    }
    fn apply(&mut self, sim: &mut StableSim<'_>, me: usize, cmds: Vec<Cmd>) {
        let tick = sim.tick();
        for c in cmds {
            match c {
                Cmd::Damage { target, amount, kind } => {
                    let n = amount.max(0) as usize;
                    let before = sim.get_entity(target).map_or(0, |e| e.hp().0);
                    match kind {
                        Dmg::Phys => sim.deal_damage(me, target, n, 0, AttackTypeV1::Skill),
                        Dmg::Magic => sim.deal_damage(me, target, 0, n, AttackTypeV1::Skill),
                        Dmg::True => {
                            sim.deal_damage_typed(me, target, n, DamageTypeV1::Fixed, AttackTypeV1::Skill);
                        }
                    }
                    let after = sim.get_entity(target).map_or(before, |e| e.hp().0);
                    self.log(format!("seed {} tick {tick} caster {me} target {target} {:?} raw {n} HP {before}->{after}", sim.seed(), kind));
                }
                Cmd::Heal { target, amount } => sim.heal(me, target, amount.max(0) as usize),
                Cmd::Shield { target, amount, ticks } => {
                    sim.entity_add_shield(target, amount.max(0) as usize, ticks);
                }
                Cmd::Cc { target, kind, ticks } => {
                    let k = match kind {
                        Cc::Stun => CcKindV1::Stun,
                        Cc::Root => CcKindV1::Bind,
                        Cc::Silence => CcKindV1::BlockSkill,
                        Cc::Disarm => CcKindV1::BlockAttack,
                    };
                    sim.apply_cc(target, &CcV1::of_kind(k, ticks as u64));
                }
                Cmd::Buff { target, name, ticks, mods } => {
                    let name = format!("vs{}_{}", self.owner, name);
                    sim.entity_remove_buff(target, &name);
                    sim.add_buff(target, &buff(&name, ticks.min(600), &mods));
                }
                Cmd::SetPos { target, x, y } => {
                    sim.entity_set_pos(target, x.max(0) as u64, y.max(0) as u64);
                }
                Cmd::Banish { target, ticks } => {
                    sim.entity_banish(me, target, ticks, "", "");
                }
                Cmd::Invisible { target, ticks } => {
                    sim.entity_set_invisible(target, ticks);
                }
                Cmd::ClearCc { target } => {
                    sim.entity_clear_cc(target);
                }
                Cmd::ClearShield { target } => {
                    sim.entity_clear_shield(target);
                }
                Cmd::LoseHp { target, amount } => {
                    if let Some((hp, _)) = sim.get_entity(target).map(|e| e.hp()) {
                        if hp > 1 {
                            sim.entity_set_hp(target, hp.saturating_sub(amount.max(0) as usize).max(1));
                        }
                    }
                }
                Cmd::Fx { tag, x, y, unit, ticks } => {
                    let name = format!("{ID}_{tag}");
                    match unit {
                        Some(u) => {
                            fx_unit(sim, &name, me, u, ticks as u64);
                        }
                        None => {
                            fx_point(sim, &name, me, x, y, ticks as u64);
                        }
                    }
                }
                Cmd::Log(s) => self.log(format!("seed {} {s}", sim.seed())),
            }
        }
    }
}

pub fn buff(name: &str, ticks: usize, m: &Mods) -> BuffV1 {
    let mut b = timed(name, ticks.max(1));
    b.move_speed_mult = m.move_speed;
    b.attack_speed_mult = m.attack_speed;
    b.damaged_amplify = m.amplify.max(0) as usize;
    b.damaged_reduce = m.reduce.max(0) as usize;
    b.heal_reduce = m.heal_reduce.max(0) as usize;
    b.attack_mult = m.attack;
    b.defence_mult = m.defence;
    b.magic_resistance_mult = m.mr;
    b.magic_power_mult = m.ap;
    b.hp_regen = m.regen;
    b.crit_chance = m.crit;
    b.range = m.range.max(0) as usize;
    b.base_attack_damaged_reduce = m.ba_reduce.max(0) as usize;
    b.damage_reflect = m.reflect.max(0) as usize;
    b.skill_cooldown_mult = m.cooldown;
    b.vamp = m.vamp;
    b.ignore_wall = m.ignore_wall;
    b.undying = m.undying;
    b.cc_immune = m.cc_immune;
    b
}

/// The fight as the core sees it: Vesper, her allies, and the enemies her team can see.
pub fn snapshot(sim: &StableSim<'_>, all: &[Champ], me: &Champ) -> Vec<Fighter> {
    all.iter()
        .filter(|c| c.team == me.team || sim.is_visible(me.team, c.id))
        .map(|c| {
            let controlled = sim.get_entity(c.id).is_some_and(|e| e.cc_count() > 0);
            Fighter {
                id: c.id,
                team: c.team,
                x: c.x,
                y: c.y,
                hp: c.hp as i64,
                max_hp: c.max_hp.max(1) as i64,
                attack: c.attack as i64,
                cc_immune: c.buffs.iter().any(|b| b.cc_immune),
                controlled,
            }
        })
        .collect()
}

impl StablePassive for Vesper {
    fn clone_box(&self) -> Box<dyn StablePassive> {
        Box::new(self.clone())
    }
    fn on_attack(&mut self, sim: &mut StableSim<'_>, _: usize, _: usize, _: usize, _: &mut usize) {
        // the normal jab: show the jab overlay (visual only)
        let tick = sim.tick();
        if let Some(core) = self.core.as_mut() {
            core.jab_until = tick + 24;
        }
    }
    fn on_damaged(&mut self, sim: &mut StableSim<'_>, _: usize, _: usize, _: usize, damage: usize) {
        // she takes a hit: raise the block overlay (visual only)
        let tick = sim.tick();
        if let Some(core) = self.core.as_mut().filter(|_| damage > 0) {
            core.block_until = tick + 18;
        }
    }
    fn on_dead(&mut self, sim: &mut StableSim<'_>, player: usize) {
        let tick = sim.tick();
        if let Some(core) = self.core.as_mut() {
            core.on_dead(tick);
        }
        self.dead = true;
        let id = sim.get_player(player).and_then(|p| p.champion()).map(|e| e.id()).unwrap_or(self.owner);
        let exists = sim.get_entity(id).is_some();
        for name in self.visual_buffs.drain(..) {
            if exists {
                sim.entity_remove_buff(id, &name);
            }
        }
        self.flush(sim, true);
    }
    fn on_update(&mut self, sim: &mut StableSim<'_>, _: u64, player: usize, entity: usize) {
        let tick = sim.tick();
        if self.last_tick == Some(tick) {
            return;
        }
        self.last_tick = Some(tick);
        self.owner = entity;
        if !self.rank_set && tick >= 60 {
            self.athlete = BOOK.athlete_of(sim.seed(), player);
            let (rank, position) = BOOK.pinned(sim.seed()).rank_for(self.athlete);
            self.rank = rank.min(7);
            self.top = position;
            self.rank_set = true;
            self.modules = modules_for(self.athlete);
            if let Some(core) = self.core.as_mut() {
                core.rank = self.rank;
                core.top = self.top;
                core.modules = self.modules;
            }
        }
        let all = champions(sim);
        let Some(m) = all.iter().find(|c| c.id == entity).cloned() else {
            if self.core.as_ref().is_some_and(|c| !c.rifts.is_empty()) || !self.visual_buffs.is_empty() {
                self.on_dead(sim, player);
            }
            return;
        };
        self.dead = false;
        let core = self.core.get_or_insert_with(|| Core::new(sim.seed(), entity, 0, None));
        if self.rank_set {
            core.rank = self.rank;
            core.top = self.top;
            core.modules = self.modules;
        }
        let fighters = snapshot(sim, &all, &m);
        let interval = sim.get_entity(entity).map_or(84, |e| e.attack_interval()).max(1);
        let wall = |x: i64, y: i64| walls::wall_at(x, y);
        let input = Input { tick, me: entity, fighters: &fighters, attack_interval: if interval == 0 { 84 } else { interval }, stunned: m.stunned, wall: &wall };
        let cmds = core.step(&input);
        let snapshot_core = core.clone();
        self.apply(sim, entity, cmds);
        self.sync_visuals(sim, &m, &snapshot_core);
        if self.sig.is_none() && tick >= 1800 {
            self.sig = Some(signature(&BOOK, sim, &all, self.athlete));
        }
        if let (Some(sig), Some(a)) = (&self.sig, self.athlete) {
            if tick.is_multiple_of(30) {
                self.record.track(&BOOK, sim, sig, m.team, a, tick);
            }
        }
        if tick.is_multiple_of(3600) && BOOK.first(format!("vs-summary:{}:{entity}:{tick}", sim.seed())) {
            let s = &snapshot_core.stats;
            let rank_name = if self.rank >= 7 && self.top == Some(1) { TOP_NAME } else { RANKS[self.rank] };
            let moves: Vec<String> = MOVE_IDS.iter().zip(s.move_uses).map(|(m, n)| format!("{m}={n}")).collect();
            BOOK.log(&format!(
                "game {} native {} minute {} athlete {:?} rank {} ({rank_name}) modules {:?} stress {} orbs {}/{} hexcore {}% rifts {} punches {} swipes {} orbs_spent {}+{}hex fused {} twists {} best_combo {} shatters {}/{} breaks L{} R{} awakens {}/{} takedowns {} broken_rifts {} kidnaps {}/{} gates {}/{} traps {}/{} moves {} uses {:?}",
                self.sig.as_deref().unwrap_or("warmup"),
                crate::VERSION,
                tick / 3600,
                self.athlete,
                self.rank,
                snapshot_core.modules.map(|m| MODULES[m].id),
                snapshot_core.stress,
                snapshot_core.pure_ready(tick),
                snapshot_core.sockets(),
                snapshot_core.completion,
                s.rifts_opened,
                s.punches,
                s.swipes,
                s.orbs,
                s.hexorbs,
                s.fused,
                s.twists,
                s.best_combo,
                s.useful_shatters,
                s.shatters,
                s.left_breaks,
                s.right_breaks,
                s.good_awakens,
                s.awakens,
                s.takedowns,
                s.breaks,
                s.useful_kidnaps,
                s.kidnaps,
                s.engaged_gates,
                s.gates,
                s.trap_catches,
                s.traps,
                moves.join(" "),
                s.tech_uses
            ));
        }
        self.flush(sim, false);
    }
}
