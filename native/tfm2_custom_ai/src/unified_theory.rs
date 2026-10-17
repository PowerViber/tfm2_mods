//! The Unified Theory: Einstein, Newton and Curie in one body, eighteen equations on five charges.
//! This passive only translates: it reads the match into `unified_core::Input`, applies the core's
//! commands to the game, and draws the scientist. Every rule lives in `unified_core` (shared with
//! Equation Lab), so the game and the lab cannot drift apart.
use crate::mastery::{signature, Book, Record};
use crate::unified_core::{Cmd, Core, Fighter, Input, RANKS};
use crate::unified_eq_data::Kind;
use crate::{champions, d2, fx_point, fx_unit, sq, timed, Champ};
use mod_api_stable::{AttackTypeV1, CcKindV1, CcV1, DamageTypeV1, StablePassive, StableSim};
use std::collections::VecDeque;
use std::io::Write;

pub static BOOK: Book = Book::new("unified_theory");
const ID: &str = "tfm2_custom_unified_theory";
/// Ticks of the outfit morph between two scientists.
const TRANSFORM_ART_TICKS: usize = 24;
/// View effects per tick; labels and numbers wait their turn, moving art is never delayed.
const FX_PER_TICK: usize = 6;
/// The charge bar floats this far above the scientist; labels stack above it.
const HUD_HEIGHT: i64 = 65000;

#[derive(Clone)]
pub struct UnifiedTheory {
    core: Option<Core>,
    owner: usize,
    started: bool,
    last_tick: Option<usize>,
    seed: u64,
    rank: usize,
    // Cosmetic only: the position mastery already returned.
    top_pos: Option<usize>,
    athlete: Option<usize>,
    rank_set: bool,
    record: Record,
    sig: Option<String>,
    trace: Vec<String>,
    dropped: usize,
    visual_buffs: Vec<String>,
    /// Which way the scientist faces: the body is an overlay the game never mirrors, so facing left picks `_l`.
    facing: crate::facing::Facing,
    visible_persona: usize,
    transformation: Option<(usize, usize, usize)>,
    /// Labels and numbers waiting for an effect slot.
    queued: VecDeque<(String, i64, i64, Option<usize>, usize)>,
}
impl Default for UnifiedTheory {
    fn default() -> Self {
        Self {
            core: None,
            owner: 0,
            started: false,
            last_tick: None,
            seed: 0,
            rank: 0,
            top_pos: None,
            athlete: None,
            rank_set: false,
            record: Record::default(),
            sig: None,
            trace: vec![],
            dropped: 0,
            visual_buffs: vec![],
            facing: crate::facing::Facing::default(),
            visible_persona: 0,
            transformation: None,
            queued: VecDeque::new(),
        }
    }
}

impl UnifiedTheory {
    fn log(&mut self, s: String) {
        if self.trace.len() < 64 {
            self.trace.push(s)
        } else {
            self.dropped = self.dropped.saturating_add(1);
        }
    }
    fn flush(&mut self, sim: &StableSim<'_>, me: usize, force: bool) {
        if self.trace.is_empty() || (!force && !sim.tick().is_multiple_of(120)) {
            return;
        }
        let mut lines = std::mem::take(&mut self.trace);
        if self.dropped > 0 {
            lines.push(format!("{} additional diagnostic events omitted by the bounded trace buffer", self.dropped));
            self.dropped = 0;
        }
        if sim.sim_origin().is_some_and(|o| o.kind == mod_api_stable::SimOriginKindV1::ServerPresim.code()) {
            return;
        }
        let fingerprint = lines.iter().flat_map(|s| s.bytes()).fold(0xcbf29ce484222325u64, |hash, byte| (hash ^ byte as u64).wrapping_mul(0x100000001b3));
        let key = format!("ut-trace:{}:{me}:{}:{}:{fingerprint}", sim.seed(), sim.tick(), sim.sim_origin().map_or(0, |o| o.kind));
        if !BOOK.first(key) {
            return;
        }
        if let Some(dir) = crate::mod_dir() {
            let _ = append_trace(&dir.join("unified_theory_damage_log.txt"), &lines, 8 * 1024 * 1024);
        }
    }
    /// The body layer (outfit, writing pose, release pose or morph), the rank emblem and the aura.
    fn style_names(&self, tick: usize) -> Vec<String> {
        let side = if self.facing.left() { "_l" } else { "" };
        let core = self.core.as_ref();
        let persona = core.map_or(0, |c| c.persona);
        let since_cast = core.map_or(usize::MAX, |c| tick.saturating_sub(c.last_cast));
        let writing = core.is_some_and(|c| !c.reserve.is_empty() || !c.plan.is_empty());
        let mut names = vec![if let Some((from, to, _)) = self.transformation {
            format!("ut_transform{from}_{to}_r{}{side}", self.rank)
        } else if core.is_some_and(|c| c.last_cast > 0) && since_cast < 24 {
            format!("ut_pose{persona}_r{}_p{}{side}", self.rank, if since_cast < 12 { 1 } else { 2 })
        } else if writing {
            format!("ut_pose{persona}_r{}_p0{side}", self.rank)
        } else {
            format!("ut_outfit{persona}_r{}{side}", self.rank)
        }];
        names.push(if self.rank == 7 { format!("ut_top{}", self.top_pos.unwrap_or(10).clamp(1, 10)) } else { format!("ut_rank{}", self.rank) });
        let science = self.transformation.map_or(persona, |(_, to, _)| to);
        let suffix = if self.rank == 7 { format!("_p{}", self.top_pos.unwrap_or(10).clamp(1, 4)) } else { String::new() };
        names.push(format!("ut_gearback{science}_r{}{suffix}", self.rank));
        names
    }
    fn sync_visual_buffs(&mut self, sim: &mut StableSim<'_>, m: &Champ) {
        let tick = sim.tick();
        let persona = self.core.as_ref().map_or(0, |c| c.persona);
        if let Some((_, to, until)) = self.transformation {
            if tick >= until {
                self.visible_persona = to;
                self.transformation = None;
            }
        }
        if self.transformation.is_none() && self.visible_persona != persona {
            self.transformation = Some((self.visible_persona, persona, tick + TRANSFORM_ART_TICKS));
        }
        let want = self.style_names(tick);
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
                let life = if name.starts_with("ut_transform") { self.transformation.map_or(1, |t| t.2.saturating_sub(tick)) } else { 5184000 };
                sim.add_buff(m.id, &timed(name, life));
            }
        }
        self.visual_buffs = want;
    }
    /// Champions (enemies only while visible) and eligible farm units, as the core sees them.
    fn fighters(&self, sim: &StableSim<'_>, all: &[Champ], m: &Champ) -> Vec<Fighter> {
        let mut out: Vec<Fighter> = vec![];
        for c in all {
            let Some(e) = sim.get_entity(c.id) else { continue };
            let stat = e.stat();
            out.push(Fighter {
                id: c.id,
                team: c.team,
                x: c.x,
                y: c.y,
                hp: c.hp as i64,
                max_hp: c.max_hp as i64,
                ad: stat.attack as i64,
                ap: stat.magic_power as i64,
                champion: true,
                visible: c.team == m.team || sim.is_visible(m.team, c.id),
                targetable: e.is_targetable(),
                cc_immune: c.buffs.iter().any(|b| b.cc_immune),
            });
        }
        let mut wave = crate::units(sim);
        wave.sort_by_key(|u| (d2(m.x, m.y, u.2, u.3), u.0));
        for (id, team, x, y) in wave.into_iter().filter(|u| u.1 != m.team && d2(m.x, m.y, u.2, u.3) <= sq(100000)).take(24) {
            if !crate::farming::eligible(sim, m.id, id) {
                continue;
            }
            let Some(e) = sim.get_entity(id) else { continue };
            if !e.is_targetable() {
                continue;
            }
            let (hp, max_hp) = e.hp();
            out.push(Fighter { id, team, x, y, hp: hp as i64, max_hp: max_hp as i64, ad: 0, ap: 0, champion: false, visible: true, targetable: true, cc_immune: false });
        }
        out.sort_by_key(|f| f.id);
        out
    }
    fn apply(&mut self, sim: &mut StableSim<'_>, me: usize, cmds: Vec<Cmd>) {
        let tick = sim.tick();
        let mut moving = 0;
        for c in cmds {
            match c {
                Cmd::Damage { target, amount, kind } => {
                    let n = amount.max(0) as usize;
                    if !sim.get_entity(target).is_some_and(|e| e.is_alive()) {
                        continue;
                    }
                    let before = sim.get_entity(target).map_or(0, |e| e.hp().0);
                    match kind {
                        Kind::Phys => sim.deal_damage(me, target, n, 0, AttackTypeV1::Skill),
                        Kind::Magic => sim.deal_damage(me, target, 0, n, AttackTypeV1::Skill),
                        Kind::True => {
                            sim.deal_damage_typed(me, target, n, DamageTypeV1::Fixed, AttackTypeV1::Skill);
                        }
                    }
                    let after = sim.get_entity(target).map_or(before, |e| e.hp().0);
                    self.log(format!("seed {} tick {tick} caster {me} target {target} {kind:?} raw {n} HP {before}->{after}", sim.seed()));
                }
                Cmd::Heal { target, amount } => sim.heal(me, target, amount.max(0) as usize),
                Cmd::Shield { target, amount, ticks } => {
                    if amount > 0 {
                        sim.entity_add_shield(target, amount as usize, ticks);
                    }
                }
                Cmd::Root { target, ticks } => sim.apply_cc(target, &CcV1::of_kind(CcKindV1::Bind, ticks as u64)),
                Cmd::Speed { target, pct, ticks } => {
                    let name = format!("ut{me}_{}", if pct < 0 { "slow" } else { "haste" });
                    sim.entity_remove_buff(target, &name);
                    let mut b = timed(&name, ticks.min(600));
                    b.move_speed_mult = pct as i32;
                    sim.add_buff(target, &b);
                }
                Cmd::SetPos { target, x, y } => {
                    if sim.get_entity(target).is_some_and(|e| e.is_alive()) {
                        sim.entity_set_pos(target, x.clamp(0, 960_000) as u64, y.clamp(0, 960_000) as u64);
                    }
                }
                Cmd::Fx { tag, x, y, unit, ticks } => {
                    // Projectiles and zones are the fight itself: drawn at once (bounded). Labels queue.
                    let label = is_label(&tag);
                    if label {
                        if self.queued.len() < 12 {
                            self.queued.push_back((tag, x, y, unit, ticks));
                        }
                    } else if moving < FX_PER_TICK + 4 {
                        moving += 1;
                        self.draw(sim, me, &tag, x, y, unit, ticks);
                    }
                }
                Cmd::Log(s) => self.log(format!("seed {} {s}", sim.seed())),
            }
        }
        for _ in 0..FX_PER_TICK.saturating_sub(moving.min(FX_PER_TICK)) {
            let Some((tag, x, y, unit, ticks)) = self.queued.pop_front() else { break };
            self.draw(sim, me, &tag, x, y, unit, ticks);
        }
    }
    #[allow(clippy::too_many_arguments)]
    fn draw(&self, sim: &mut StableSim<'_>, me: usize, tag: &str, x: i64, y: i64, unit: Option<usize>, ticks: usize) {
        let name = format!("{ID}_{tag}");
        // Above the head, bottom to top: charge bar, reserved traits, equation, its value, the banner.
        let height = if tag.starts_with("ch_") {
            Some(HUD_HEIGHT)
        } else if tag.starts_with("rv") {
            Some(HUD_HEIGHT + 9000)
        } else if tag.starts_with("eq") {
            Some(HUD_HEIGHT + 19000)
        } else if tag.starts_with('n') && tag.as_bytes().get(1).is_some_and(|b| b.is_ascii_digit()) {
            Some(HUD_HEIGHT + 30000)
        } else if tag.starts_with("sig") || tag.starts_with("uni") {
            Some(HUD_HEIGHT + 42000)
        } else {
            None
        };
        if let Some(h) = height {
            fx_point(sim, &name, me, x, y - h, ticks as u64);
            return;
        }
        match unit {
            Some(u) => {
                fx_unit(sim, &name, me, u, ticks as u64);
            }
            None => {
                fx_point(sim, &name, me, x, y, ticks as u64);
            }
        }
    }
}

/// Equation, signature and Unification labels, live values and reserve icons: these may wait a tick.
fn is_label(tag: &str) -> bool {
    tag.starts_with("eq") || tag.starts_with("sig") || tag.starts_with("uni") || (tag.starts_with('n') && tag.as_bytes().get(1).is_some_and(|b| b.is_ascii_digit()))
}
fn append_trace(path: &std::path::Path, lines: &[String], limit: u64) -> std::io::Result<()> {
    if std::fs::metadata(path).is_ok_and(|m| m.len() >= limit) {
        let previous = path.with_extension("previous.txt");
        if previous.exists() {
            std::fs::remove_file(&previous)?;
        }
        std::fs::rename(path, previous)?;
    }
    let mut f = std::fs::OpenOptions::new().create(true).append(true).open(path)?;
    writeln!(f, "{}", lines.join("\n"))
}

impl StablePassive for UnifiedTheory {
    fn clone_box(&self) -> Box<dyn StablePassive> {
        Box::new(self.clone())
    }
    fn on_dead(&mut self, sim: &mut StableSim<'_>, player: usize) {
        if let Some(c) = &mut self.core {
            c.on_dead();
        }
        self.queued.clear();
        self.transformation = None;
        self.visible_persona = self.core.as_ref().map_or(0, |c| c.persona);
        let id = sim.get_player(player).and_then(|p| p.champion()).map(|e| e.id()).unwrap_or(self.owner);
        let exists = sim.get_entity(id).is_some();
        for name in self.visual_buffs.drain(..) {
            if exists {
                sim.entity_remove_buff(id, &name);
            }
        }
        self.flush(sim, self.owner, true);
    }
    fn on_kill(&mut self, sim: &mut StableSim<'_>, _: usize, entity: usize, victim: usize) {
        self.log(format!("seed {} tick {} caster {entity} engine kill credit victim {victim}", sim.seed(), sim.tick()));
    }
    fn on_update(&mut self, sim: &mut StableSim<'_>, _: u64, player: usize, entity: usize) {
        self.owner = entity;
        let tick = sim.tick();
        if self.last_tick == Some(tick) {
            return;
        }
        self.last_tick = Some(tick);
        if !self.started {
            self.started = true;
            self.seed = sim.seed() ^ 0x534349454e4345;
        }
        if !self.rank_set && tick >= 60 {
            self.athlete = BOOK.athlete_of(sim.seed(), player);
            let (rank, position) = BOOK.pinned(sim.seed()).rank_for(self.athlete);
            self.rank = rank.min(7);
            self.top_pos = position;
            self.rank_set = true;
            self.core = Some(Core::new(self.seed, entity, self.rank));
            self.log(format!("seed {} athlete {:?} rank {} ({}) equation kit", sim.seed(), self.athlete, self.rank, RANKS[self.rank]));
        }
        let all = champions(sim);
        let Some(m) = all.iter().find(|c| c.id == entity).cloned() else {
            // Some hosts remove an entity before its next passive callback.
            if !self.visual_buffs.is_empty() || self.core.as_ref().is_some_and(|c| !c.live.is_empty()) {
                self.on_dead(sim, player);
            }
            return;
        };
        self.facing.update(tick, (m.x, m.y), crate::facing::nearest_enemy(&all, &m));
        if self.sig.is_none() && tick >= 1800 {
            self.sig = Some(signature(&BOOK, sim, &all, self.athlete));
        }
        if let (Some(sig), Some(a)) = (&self.sig, self.athlete) {
            if tick.is_multiple_of(30) {
                self.record.track(&BOOK, sim, sig, m.team, a, tick);
            }
        }
        let fighters = self.fighters(sim, &all, &m);
        let unlocked = m.has("ut_ult_learned") || sim.get_entity(entity).and_then(|e| e.cooldown(mod_api_stable::SkillSlotV1::Ult)).is_some_and(|c| c.unlocked);
        let wall = |x: i64, y: i64| crate::walls::wall_at(x, y);
        let cmds = match &mut self.core {
            Some(core) => core.step(&Input { tick, me: entity, fighters: &fighters, automatic: true, action: None, ult_unlocked: unlocked, stunned: m.stunned, wall: &wall }),
            None => vec![],
        };
        self.apply(sim, entity, cmds);
        self.sync_visual_buffs(sim, &m);
        if tick.is_multiple_of(3600) {
            if let Some(c) = &self.core {
                let s = &c.stats;
                let sig = self.sig.as_deref().unwrap_or("warmup");
                if BOOK.first(format!("ut-summary:{}:{entity}:{tick}", sim.seed())) {
                    BOOK.log(&format!(
                        "game {sig} native {} minute {} athlete {:?} rank {} ({}) casts {} steps {} combos {} signatures {} unifications {} insights {} fizzles {} ults {} damage {} misplays {}",
                        crate::VERSION, tick / 3600, self.athlete, self.rank, RANKS[self.rank], s.casts, s.steps, s.combos, s.signatures, s.unifications, s.insights, s.fizzles, s.ults, s.damage, s.misplays
                    ));
                }
            }
        }
        self.flush(sim, entity, false);
    }
}

/// Learned with the ultimate: marks Unified so the passive can cast Theory of Everything.
pub struct UltimateLearned;
impl StablePassive for UltimateLearned {
    fn clone_box(&self) -> Box<dyn StablePassive> {
        Box::new(Self)
    }
    fn on_update(&mut self, sim: &mut StableSim, _: u64, _: usize, id: usize) {
        let learned = sim.get_entity(id).is_some_and(|e| (0..e.buff_count()).filter_map(|i| e.buff_at(i)).any(|b| b.name() == "ut_ult_learned"));
        if !learned {
            sim.add_buff(id, &timed("ut_ult_learned", 5_184_000));
        }
    }
}
