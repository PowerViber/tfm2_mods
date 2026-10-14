//! Vesper, the Rift Marksman: the stable-API adapter around `vesper_core`.
//!
//! Each tick it snapshots the fight, lets the core decide, then applies the core's commands: damage, crowd control,
//! forced movement, buffs and view effects. Mastery (the athlete's rank) only reaches the core's planner and the
//! visual layers chosen here.
use crate::mastery::{signature, Book, Record};
use crate::vesper_core::{look, Cc, Cmd, Core, Dmg, Fighter, Input, Mods};
use crate::vesper_data::{RANKS, TOP_NAME};
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
}

fn podium(rank: usize, top: Option<usize>) -> usize {
    if rank >= 7 {
        top.unwrap_or(10).clamp(1, 10).min(4)
    } else {
        4
    }
}

impl Vesper {
    fn style_names(&self, core: &Core) -> Vec<String> {
        let l = look(self.rank, self.top);
        let mut names = vec![if core.aiming { format!("vs_aim_r{l}") } else { format!("vs_outfit_r{l}") }];
        names.push(if self.rank >= 7 { format!("vs_top{}", self.top.unwrap_or(10).clamp(1, 10)) } else { format!("vs_rank{}", self.rank) });
        names.push(if self.rank >= 7 { format!("vs_back_r7_p{}", podium(self.rank, self.top)) } else { format!("vs_back_r{}", self.rank) });
        names.push(format!("vs_charges{}", core.charges.available().min(5)));
        names.push(format!("vs_strain{}", (core.strain / 10).clamp(0, 10)));
        names
    }
    fn sync_visuals(&mut self, sim: &mut StableSim<'_>, m: &Champ, core: &Core) {
        let want = self.style_names(core);
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
            if let Some(core) = self.core.as_mut() {
                core.rank = self.rank;
                core.top = self.top;
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
            BOOK.log(&format!(
                "game {} native {} minute {} athlete {:?} rank {} ({rank_name}) strain {} charges {} rifts {} breaches {} breaks {} packets {} avg_eff {} kidnaps {}/{} gates {}/{} traps {}/{} uses {:?}",
                self.sig.as_deref().unwrap_or("warmup"),
                crate::VERSION,
                tick / 3600,
                self.athlete,
                self.rank,
                snapshot_core.strain,
                snapshot_core.charges.available(),
                s.rifts_opened,
                s.breaches,
                s.breaks,
                s.packets,
                if s.packets > 0 { s.eff_sum / s.packets as i64 / 100 } else { 0 },
                s.useful_kidnaps,
                s.kidnaps,
                s.engaged_gates,
                s.gates,
                s.trap_catches,
                s.traps,
                s.tech_uses
            ));
        }
        self.flush(sim, false);
    }
}
