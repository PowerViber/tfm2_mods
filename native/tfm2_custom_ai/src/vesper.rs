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
    /// The "vsr:"/"vsk:" buffs that tell the game's AI where her rifts and Kidnap fields are.
    zone_buffs: Vec<String>,
    /// Which way she faces: her body is an overlay, which the game never mirrors, so she picks the `_l` copy.
    facing: crate::facing::Facing,
    profile: crate::vesper_perf::Profile,
    visual_key: Option<[usize;14]>,
    zone_state: Vec<Zone>,
}

// ------------------------------------------------------------------ rifts are dangerous: telling the game's AI

/// A danger zone for the game's AI: a visible live rift, or an armed Kidnap field (which closes and stuns).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Zone {
    pub x: i64,
    pub y: i64,
    pub r: i64,
    pub until: usize,
    pub field: bool,
}

/// Every visible live rift and armed Kidnap field. Slash and Mirror Pane count from their middle, out to half their
/// length plus their width. Hidden Seed traps are never published.
pub fn zones_of(core: &Core, tick: usize) -> Vec<Zone> {
    let mut z: Vec<Zone> = core
        .rifts
        .iter()
        .filter(|r| r.live() && !r.hidden(tick))
        .map(|r| Zone { x: (r.x + r.ex) / 2, y: (r.y + r.ey) / 2, r: r.radius + crate::vesper_math::dist(r.x, r.y, r.ex, r.ey) / 2, until: r.expires, field: false })
        .collect();
    z.extend(core.kidnap_zones().into_iter().map(|(x, y, r, until)| Zone { x, y, r, until, field: true }));
    z
}

/// The buffs publishing them: "vsr:x:y:r:until" (rifts) and "vsk:x:y:r:until" (Kidnap fields). Positions are rounded
/// to 4000 so a rift riding its host only republishes when it really moved.
pub fn zone_names(core: &Core, tick: usize) -> Vec<(String, usize)> {
    zones_of(core, tick)
        .into_iter()
        .map(|z| (format!("{}:{}:{}:{}:{}", if z.field { "vsk" } else { "vsr" }, z.x / 4000 * 4000, z.y / 4000 * 4000, z.r, z.until), z.until.saturating_sub(tick).max(1)))
        .collect()
}

/// Vesper zones read back from any champion's buffs, with the team that owns each.
pub fn rifts_up(all: &[Champ], tick: usize) -> Vec<(Zone, usize)> {
    let mut out = Vec::new();
    for c in all {
        for b in &c.buffs {
            let n = b.name();
            let field = n.starts_with("vsk:");
            let Some(rest) = n.strip_prefix("vsr:").or_else(|| n.strip_prefix("vsk:")) else { continue };
            let v: Vec<i64> = rest.split(':').filter_map(|s| s.parse().ok()).collect();
            if v.len() >= 4 && (v[3].max(0) as usize) > tick {
                out.push((Zone { x: v[0], y: v[1], r: v[2], until: v[3].max(0) as usize, field }, c.team));
            }
        }
    }
    out
}

/// The input AI's rule for her enemies (moves only): never walk into a rift or an armed Kidnap field, and walk
/// straight out of an armed field (that is the dodge). A rift that opens under them is fought in, not fled.
pub fn steer(zones: &[Zone], me: (i64, i64), dest: (i64, i64)) -> Option<(i64, i64)> {
    let out_of = |z: &Zone, from: (i64, i64)| {
        let (dx, dy) = (from.0 - z.x, from.1 - z.y);
        let (dx, dy) = if dx == 0 && dy == 0 { (1, 0) } else { (dx, dy) };
        crate::vesper_math::along(z.x, z.y, dx, dy, z.r + 8000)
    };
    let inside = |z: &Zone, p: (i64, i64)| crate::vesper_math::dist(p.0, p.1, z.x, z.y) <= z.r;
    if let Some(z) = zones.iter().find(|z| z.field && inside(z, me)) {
        return Some(out_of(z, me));
    }
    // a destination inside: wait at the edge on the champion's own side (the far side was a walk through it)
    zones.iter().find(|z| inside(z, dest) && !inside(z, me)).map(|z| out_of(z, me))
}

/// WallAi's hook: a move order of a champion whose enemy Vesper has a rift or field in the way.
pub fn avoid_rifts(all: &[Champ], me: &Champ, dest: (i64, i64), tick: usize) -> Option<(i64, i64)> {
    let zones: Vec<Zone> = rifts_up(all, tick).into_iter().filter(|(_, team)| *team != me.team).map(|(z, _)| z).collect();
    if zones.is_empty() {
        return None;
    }
    steer(&zones, (me.x, me.y), dest).map(|p| walls::pull_back(me.x, me.y, p.0.clamp(0, 960_000), p.1.clamp(0, 960_000)))
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

/// The gauntlet HUD and rank layers shown on Vesper (buff names `vs_*`). Facing left, her body and the scars behind
/// her use their mirrored `_l` copies; the emblem and the HUD stay put, like every champion's rank badge.
pub fn style_names(rank: usize, top: Option<usize>, core: &Core, tick: usize, left: bool) -> Vec<String> {
    let l = pose_look(rank, top);
    let side = if left { "_l" } else { "" };
    let mut names = vec![format!("vs_{}_r{l}{side}", pose_of(core, tick))];
    // the animated rank emblem at her top right (Top 10: the medallion with her number)
    names.push(if rank >= 7 { format!("vs_top{}", top.unwrap_or(10).clamp(1, 10)) } else { format!("vs_rank{rank}") });
    if rank >= 7 && top == Some(1) {
        names.push(format!("vs_back_top1{side}"));
    } else if rank >= 6 {
        names.push(format!("vs_back_wound{side}"));
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
    /// Publish her rifts and Kidnap fields for the game's AI: added when they open, removed when they close.
    fn sync_zones(&mut self, sim: &mut StableSim<'_>, me: usize, core: &Core) {
        let tick = sim.tick();
        let mut state=zones_of(core,tick);
        for z in &mut state{z.x=z.x/4000*4000;z.y=z.y/4000*4000;}
        if state==self.zone_state{return;}
        self.zone_state=state;
        let want = zone_names(core, tick);
        if want.is_empty() && self.zone_buffs.is_empty() {
            return;
        }
        crate::steve::note_map_buff(sim.seed());
        for name in &self.zone_buffs {
            if !want.iter().any(|(n, _)| n == name) {
                sim.entity_remove_buff(me, name);
            }
        }
        for (name, ticks) in &want {
            if !self.zone_buffs.contains(name) {
                sim.add_buff(me, &timed(name, *ticks));
            }
        }
        self.zone_buffs = want.into_iter().map(|(n, _)| n).collect();
    }
    fn sync_visuals(&mut self, sim: &mut StableSim<'_>, m: &Champ, core: &Core) {
        let tick=sim.tick();
        let pose=match pose_of(core,tick){"shatter"=>0,"swipe"=>1,"punch"=>2,"jab"=>3,"block"=>4,_=>5};
        let key=[self.rank,self.top.unwrap_or(0),pose,core.left_broken(tick)as usize,core.right_broken(tick)as usize,core.pure_ready(tick),core.sockets(),core.hex_ready(tick),core.hex_slots(tick),core.awakened(tick)as usize,(core.corruption/20).clamp(0,4)as usize,((core.completion-70)/5).clamp(0,6)as usize,(core.stress*10/STRESS_CAP).clamp(0,10)as usize,self.facing.left()as usize];
        if self.visual_key==Some(key)&&self.visual_buffs.iter().all(|n|m.has(n)){return;}
        self.visual_key=Some(key);
        let want = style_names(self.rank, self.top, core, sim.tick(), self.facing.left());
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
                self.profile.asset(name.strip_prefix("vs_").unwrap_or(name));
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
                    if self.trace.len() >= 96 {
                        self.dropped += 1;
                        continue;
                    }
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
                    let at=self.profile.start();
                    let name = format!("{ID}_{tag}");
                    match unit {
                        Some(u) => {
                            fx_unit(sim, &name, me, u, ticks as u64);
                        }
                        None => {
                            fx_point(sim, &name, me, x, y, ticks as u64);
                        }
                    }
                    self.profile.effect(&tag,x,y,unit,tick,ticks,at);
                }
                Cmd::Log(s) if self.trace.len() < 96 => self.log(format!("seed {} {s}", sim.seed())),
                Cmd::Log(_) => self.dropped += 1,
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
                champion: true,
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
        self.profile.bind(sim,self.owner);
        let tick = sim.tick();
        if let Some(core) = self.core.as_mut() {
            core.on_dead(tick);
        }
        self.dead = true;
        self.visual_key=None;
        self.zone_state.clear();
        let id = sim.get_player(player).and_then(|p| p.champion()).map(|e| e.id()).unwrap_or(self.owner);
        let exists = sim.get_entity(id).is_some();
        for name in self.visual_buffs.drain(..).chain(self.zone_buffs.drain(..)) {
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
        self.profile.bind(sim,entity);
        let snapshot_at=self.profile.start();
        let all = champions(sim);
        let Some(m) = all.iter().find(|c| c.id == entity).cloned() else {
            if self.core.as_ref().is_some_and(|c| !c.rifts.is_empty()) || !self.visual_buffs.is_empty() {
                self.on_dead(sim, player);
            }
            return;
        };
        self.dead = false;
        self.facing.update(tick, (m.x, m.y), crate::facing::nearest_enemy(&all, &m));
        // taken out for the update and put back at the end: no per-tick clone of the whole core
        let mut core = self.core.take().unwrap_or_else(|| Core::new(sim.seed(), entity, 0, None));
        if self.rank_set {
            core.rank = self.rank;
            core.top = self.top;
            core.modules = self.modules;
        }
        let mut fighters = snapshot(sim, &all, &m);
        let farm=crate::farming::nearby(sim,&m,&all,90_000);
        core.farm_only=!farm.is_empty();
        fighters.extend(farm.into_iter().map(|f|Fighter{champion:false,id:f.id,team:f.team,
            x:f.x,y:f.y,hp:f.hp as i64,max_hp:f.max_hp.max(1) as i64,attack:f.attack as i64,
            cc_immune:f.buffs.iter().any(|b|b.cc_immune),controlled:f.stunned||f.pushed}));
        self.profile.finish("snapshot",snapshot_at);
        let interval = sim.get_entity(entity).map_or(84, |e| e.attack_interval()).max(1);
        let wall = |x: i64, y: i64| walls::wall_at(x, y);
        let input = Input { tick, me: entity, fighters: &fighters, attack_interval: if interval == 0 { 84 } else { interval }, stunned: m.stunned, wall: &wall };
        let core_at=self.profile.start();
        let cmds = core.step_observed(&input,&mut |name,start|self.profile.phase(name,start));
        self.profile.finish("core",core_at);
        let host_at=self.profile.start();
        self.apply(sim, entity, cmds);
        self.profile.finish("host_commands",host_at);
        let visual_at=self.profile.start();
        self.sync_visuals(sim, &m, &core);
        self.sync_zones(sim, entity, &core);
        self.profile.finish("visual_sync",visual_at);
        if self.sig.is_none() && tick >= 1800 {
            self.sig = Some(signature(&BOOK, sim, &all, self.athlete));
        }
        if let (Some(sig), Some(a)) = (&self.sig, self.athlete) {
            if tick.is_multiple_of(30) {
                self.record.track(&BOOK, sim, sig, m.team, a, tick);
            }
        }
        if tick.is_multiple_of(3600) && BOOK.first(format!("vs-summary:{}:{entity}:{tick}", sim.seed())) {
            let s = &core.stats;
            let rank_name = if self.rank >= 7 && self.top == Some(1) { TOP_NAME } else { RANKS[self.rank] };
            let moves: Vec<String> = MOVE_IDS.iter().zip(s.move_uses).map(|(m, n)| format!("{m}={n}")).collect();
            BOOK.log(&format!(
                "game {} native {} minute {} athlete {:?} rank {} ({rank_name}) modules {:?} stress {} orbs {}/{} hexcore {}% rifts {} punches {} swipes {} orbs_spent {}+{}hex fused {} twists {} best_combo {} shatters {}/{} breaks L{} R{} awakens {}/{} takedowns {} broken_rifts {} kidnaps {}/{} gates {}/{} traps {}/{} moves {} uses {:?}",
                self.sig.as_deref().unwrap_or("warmup"),
                crate::VERSION,
                tick / 3600,
                self.athlete,
                self.rank,
                core.modules.map(|m| MODULES[m].id),
                core.stress,
                core.pure_ready(tick),
                core.sockets(),
                core.completion,
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
        self.core = Some(core);
        self.flush(sim, false);
        self.profile.flush(sim,entity);
    }
}
