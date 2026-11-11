//! Akane's bridge: reads the sim into an `akane_core::Frame` once per tick and applies the core's effects.
//! The engine walks her and swings her katana (the data basic attack); native code casts Dismiss & Devour, Kunai
//! Flip, Crimson Edge and Crimson Night (never() data shells).
//!
//! * Orbs: `on_kill` and `on_assist` mark a fresh takedown; the frame turns enemy champions that just died into
//!   orb spots (an assist doesn't say who died).
//! * Dismiss & Devour: `entity_set_invisible`, an invulnerable/fast named buff and her own `akn_` attack/skill locks
//!   (never read as being stunned).
//! * Crimson Night: each tick of it, her team's champions with no enemy eye within the nearsight radius are hidden
//!   (the same eye test as `valorant::fog`).
//! * Every visible move goes through `crate::motion`; the dash's hop over a wall and the finale teleport are the two
//!   hand-set positions (allowlisted in `motion_guard_tests.rs`).
//! * The AI's walk (`Effect::Steer`) is a short `akn_go:x:y` buff that the shared input AI (`steve::WallAi`) turns into
//!   a move order for her only.
use crate::akane_core::{self as core, Core, Effect, Frame, Point, Unit};
use crate::akane_data as d;
use crate::akane_presentation as art;
use mod_api_stable::{AttackTypeV1, BuffV1, CcKindV1, CcV1, SkillSlotV1, StablePassive, StableSim};

pub(crate) const OWN_CC: &str = "akn_";
pub(crate) const GO_PREFIX: &str = "akn_go:";
const SOUL: &str = "akn_soul";
const INVULN: &str = "akn_invuln";
const HOME: [(i64, i64); 2] = [(96_000, 864_000), (864_000, 96_000)];
const HARD: [CcKindV1; 6] = [CcKindV1::Stun, CcKindV1::Airborne, CcKindV1::Taunt, CcKindV1::Fear, CcKindV1::Charm, CcKindV1::BlockSkill];

fn named(kind: CcKindV1, ticks: u64, name: &str) -> CcV1 {
    let mut c = CcV1::of_kind(kind, ticks);
    c.set_name(name);
    c
}
fn hard_cc(sim: &StableSim, id: usize, own: bool) -> bool {
    sim.get_entity(id).is_some_and(|e| {
        (0..e.cc_count()).filter_map(|i| e.cc_at(i)).any(|c| {
            !(own && c.name().starts_with(OWN_CC)) && !crate::motion::is_self_motion(&c) && HARD.iter().any(|k| k.code() == c.kind)
        })
    })
}
fn replace_buff(sim: &mut StableSim, id: usize, b: &BuffV1) {
    sim.entity_remove_buff(id, b.name());
    sim.add_buff(id, b);
}
fn remove_prefix(sim: &mut StableSim, id: usize, prefix: &str) {
    let names: Vec<String> = sim
        .get_entity(id)
        .map(|e| (0..e.buff_count()).filter_map(|i| e.buff_at(i)).filter(|b| b.name().starts_with(prefix)).map(|b| b.name().to_owned()).collect())
        .unwrap_or_default();
    for n in names {
        sim.entity_remove_buff(id, &n);
    }
}

/// The move a steering buff asks for (read by the shared input AI).
pub fn steer_of(name: &str) -> Option<(i64, i64)> {
    let mut it = name.strip_prefix(GO_PREFIX)?.split(':').filter_map(|s| s.parse::<i64>().ok());
    Some((it.next()?, it.next()?))
}

#[derive(Clone, Default)]
pub struct Akane {
    pub core: Core,
    profile: Option<crate::athlete_ai_core::Profile>,
    terrain: Option<(u64, Vec<bool>, Vec<bool>)>,
    basic_hits: Vec<usize>,
    in_skill: bool,
    /// a kill or assist was credited at this tick
    takedown_at: Option<usize>,
    killed: Vec<usize>,
    /// enemy champions alive last frame: (id, position)
    alive_last: Vec<(usize, Point)>,
    night_until: usize,
    fx_tokens: usize,
    fx_tick: u64,
    art_v1: bool,
    art_pos: Option<Point>,
    art_heading: &'static str,
}

impl Akane {
    fn grids(&mut self, tick: u64) -> (Vec<bool>, Vec<bool>) {
        if let Some((t, g, b)) = &self.terrain {
            if tick < t + 600 {
                return (g.clone(), b.clone());
            }
        }
        let n = core::CELLS;
        let c = core::CELL;
        let g: Vec<bool> = (0..n * n).map(|i| crate::walls::wall_at(i % n * c + c / 2, i / n * c + c / 2)).collect();
        let b: Vec<bool> = (0..n * n).map(|i| crate::walls::bush_at(i % n * c + c / 2, i / n * c + c / 2)).collect();
        self.terrain = Some((tick, g.clone(), b.clone()));
        (g, b)
    }

    fn can_bits(name: Option<String>) -> u8 {
        use crate::roster::Kind;
        let Some(def) = name.as_deref().and_then(crate::roster::def) else { return 0 };
        let mut bits = 0;
        for (kind, bit) in [(Kind::Dash, core::CAN_DASH), (Kind::Blink, core::CAN_BLINK), (Kind::Cc, core::CAN_CC), (Kind::Heal, core::CAN_HEAL), (Kind::Shield, core::CAN_SHIELD)] {
            if def.has(kind) {
                bits |= bit;
            }
        }
        bits
    }

    pub(crate) fn frame(&mut self, sim: &StableSim, owner: usize) -> Frame {
        let tick = sim.tick() as u64;
        let me = sim.get_entity(owner);
        let team = me.as_ref().map_or(usize::MAX, |e| e.team());
        let (pos, hp, max_hp, attack, armor, alive) = me.as_ref().map_or((Point::default(), 0, 1, 0, 0, false), |e| {
            let p = e.pos();
            let h = e.hp();
            (Point::new(p.0 as i64, p.1 as i64), h.0 as i64, h.1.max(1) as i64, e.stat().attack as i64, e.stat().defence as i64, e.is_alive())
        });
        let has_buff = |id: usize, name: &str| sim.get_entity(id).is_some_and(|e| (0..e.buff_count()).filter_map(|i| e.buff_at(i)).any(|b| b.name() == name));
        let profile = crate::athlete_ai::pinned_owner(sim, owner, &mut self.profile);
        let mut values = [50; 13];
        values[..9].copy_from_slice(&profile.values);
        values[9..].copy_from_slice(&profile.sense.0);
        let farming = crate::farming::Context::new(sim, owner);
        let realms = crate::realm::realms(sim);
        let all = crate::champions_shared(sim);
        let mut units = vec![];
        let mut alive_now = vec![];
        let mut dead_enemy = vec![];
        for e in (0..sim.entity_count()).filter_map(|i| sim.entity_at(i)) {
            if e.id() == owner {
                continue;
            }
            let e_team = e.team();
            if e.is_champion() && e_team != team {
                let p = e.pos();
                if e.is_alive() {
                    alive_now.push((e.id(), Point::new(p.0 as i64, p.1 as i64)));
                } else if let Some(&(_, last)) = self.alive_last.iter().find(|a| a.0 == e.id()) {
                    dead_enemy.push((e.id(), last));
                }
            }
            if !e.is_alive() {
                continue;
            }
            let visible = e_team == team || sim.is_visible(team, e.id());
            let p = e.pos();
            let h = e.hp();
            let buffs: Vec<BuffV1> = (0..e.buff_count()).filter_map(|i| e.buff_at(i)).collect();
            // a cage wall or another Main Event realm between them: nothing of hers reaches it
            let reach = crate::realm::can_interact(&realms, owner, e.id()) && crate::grimhald_cage::can_interact(&all, owner, e.id());
            let targetable = e.is_targetable() && reach;
            let kind = e.kind();
            let champion = e.is_champion();
            let skill_ready = champion
                && [SkillSlotV1::Skill, SkillSlotV1::Skill2, SkillSlotV1::Ult].iter().any(|&s| {
                    // a native kit's data slot never comes off cooldown: its real readiness is unknown, so assume ready
                    e.cooldown(s).is_some_and(|c| c.remaining == 0 || c.cooltime >= 5_000_000)
                });
            let stat = e.stat();
            units.push(Unit {
                id: e.id(),
                team: e_team,
                pos: if visible { Point::new(p.0 as i64, p.1 as i64) } else { Point::default() },
                hp: h.0 as i64,
                max_hp: h.1.max(1) as i64,
                shield: e.shield() as i64,
                radius: e.radius().max(1000) as i64,
                armor: stat.defence as i64,
                attack: stat.attack.max(stat.magic_power) as i64,
                champion,
                tower: e.is_tower(),
                visible,
                targetable,
                farmable: farming.calm()
                    && e_team != team
                    && targetable
                    && visible
                    && !champion
                    && farming.eligible_known(e.id(), e_team, kind, p, kind.is_none() && e.is_minion(), &buffs),
                stunned: champion && hard_cc(sim, e.id(), false),
                velocity: Point::default(),
                flash_ready: champion && !buffs.iter().any(|b| b.name() == "flash_cd"),
                skill_ready,
                can: if champion { Self::can_bits(e.name()) } else { 0 },
                cc_immune: buffs.iter().any(|b| b.cc_immune),
                reduce: buffs.iter().map(|b| b.damaged_reduce as i64).max().unwrap_or(0),
                parry: champion && buffs.iter().any(|b| b.name() == "v1_guard") && !buffs.iter().any(|b| b.name() == "v1_parry_cd"),
                tanked: false,
            });
        }
        // towers: is someone of her team (a minion or a champion, not her) inside for it to shoot first?
        let mine: Vec<Point> = units.iter().filter(|u| u.team == team && !u.tower).map(|u| u.pos).collect();
        for u in units.iter_mut().filter(|u| u.tower && u.team != team) {
            u.tanked = mine.iter().any(|p| p.within(u.pos, d::AI_TOWER_RANGE));
        }
        // velocities from last frame's positions
        for u in units.iter_mut().filter(|u| u.champion && u.visible) {
            if let Some(&(_, last)) = self.alive_last.iter().find(|a| a.0 == u.id) {
                u.velocity = Point::new(u.pos.x - last.x, u.pos.y - last.y);
            }
        }
        self.alive_last = alive_now;
        // takedowns: a credited kill or assist this tick turns enemy champions that just died into orb spots
        let mut takedowns = vec![];
        if self.takedown_at.is_some_and(|t| (tick as usize) <= t + 2) {
            for (id, at) in dead_enemy {
                if !self.killed.contains(&id) {
                    takedowns.push((id, at));
                }
            }
            self.killed = takedowns.iter().map(|t| t.0).collect();
            if !takedowns.is_empty() {
                self.takedown_at = None;
            }
        }
        let ult_unlocked = me.as_ref().is_some_and(|e| (0..e.buff_count()).filter_map(|i| e.buff_at(i)).any(|b| b.name() == crate::ULT_LEARNED));
        let (terrain, bushes) = self.grids(tick);
        let home = HOME.get(team).copied().unwrap_or(HOME[0]);
        Frame {
            tick,
            owner,
            team,
            alive,
            pos,
            hp,
            max_hp,
            attack,
            armor,
            stunned: hard_cc(sim, owner, true),
            ult_unlocked,
            flash_ready: !has_buff(owner, "flash_cd"),
            home: Point::new(home.0, home.1),
            units,
            terrain,
            bushes,
            profile: values,
            basic_hits: std::mem::take(&mut self.basic_hits),
            takedowns,
            manual: false,
            order: None,
        }
    }

    fn fx_ok(&mut self, tick: u64, ambient: bool) -> bool {
        if tick != self.fx_tick {
            let gained = (tick.saturating_sub(self.fx_tick) / 2) as usize;
            self.fx_tokens = (self.fx_tokens + gained.max(1)).min(d::FX_TOKENS);
            self.fx_tick = tick;
        }
        let floor = if ambient { 4 } else { 0 };
        if self.fx_tokens > floor {
            self.fx_tokens -= 1;
            true
        } else {
            false
        }
    }

    pub(crate) fn apply(&mut self, sim: &mut StableSim, owner: usize, effects: Vec<Effect>) {
        let tick = sim.tick();
        let all = crate::champions_shared(sim);
        let realms = crate::realm::realms(sim);
        let reach = |a: usize, b: usize| crate::realm::can_interact(&realms, a, b) && crate::grimhald_cage::can_interact(&all, a, b);
        let not_tower = |sim: &StableSim, id: usize| sim.get_entity(id).is_some_and(|e| !e.is_tower());
        for e in effects {
            match e {
                Effect::Damage { target, amount, direct } => {
                    if !reach(owner, target) || !not_tower(sim, target) {
                        continue;
                    }
                    if direct {
                        if let Some(c) = all.iter().find(|c| c.id == target) {
                            if crate::batch2::try_parry(sim, c, owner, amount.max(0) as usize) {
                                crate::mod_log(sim, "akane_log.txt", &format!("parry{tick}"), &format!("slash parried by {target}"));
                                continue;
                            }
                        }
                    }
                    self.in_skill = true;
                    sim.deal_damage(owner, target, amount.max(0) as usize, 0, AttackTypeV1::Skill);
                    self.in_skill = false;
                }
                Effect::Slow { target, percent, ticks } => {
                    if reach(owner, target) && not_tower(sim, target) {
                        let mut b = crate::timed("akn_slow", ticks as usize);
                        b.move_speed_mult = -(percent as i32);
                        replace_buff(sim, target, &b);
                    }
                }
                Effect::KnockUp { target, ticks } => {
                    if reach(owner, target) && not_tower(sim, target) {
                        sim.apply_cc(target, &named(CcKindV1::Airborne, ticks, "akn_knockup"));
                    }
                }
                Effect::Vulnerable { target, percent, ticks } => {
                    if reach(owner, target) && not_tower(sim, target) {
                        let mut b = crate::timed("akn_vulnerable", ticks as usize);
                        b.damaged_amplify = percent.max(0) as usize;
                        replace_buff(sim, target, &b);
                    }
                }
                Effect::Bleed { target, stacks, ticks } => {
                    remove_prefix(sim, target, "akn_bleed:");
                    if stacks > 0 {
                        sim.add_buff(target, &crate::timed(&format!("akn_bleed:{stacks}"), ticks as usize));
                    }
                }
                Effect::Crimson { target, ticks } => replace_buff(sim, target, &crate::timed("akn_crimson", ticks as usize)),
                Effect::Mark { target, ticks } => {
                    sim.entity_remove_buff(target, "akn_mark");
                    if ticks > 0 {
                        sim.add_buff(target, &crate::timed("akn_mark", ticks as usize));
                    }
                }
                Effect::Soul { ticks, speed_pct } => {
                    // invisible, invulnerable and fast; her own locks make her move-only until it ends
                    sim.entity_set_invisible(owner, ticks as usize);
                    let mut b = crate::timed(SOUL, ticks as usize);
                    b.damaged_reduce = 100;
                    b.base_attack_damaged_reduce = 100;
                    b.skill_damaged_reduce = 100;
                    b.cc_immune = true;
                    b.move_speed_mult = speed_pct as i32;
                    replace_buff(sim, owner, &b);
                    for k in [CcKindV1::BlockAttack, CcKindV1::BlockSkill] {
                        sim.apply_cc(owner, &named(k, ticks, "akn_soul_lock"));
                    }
                }
                Effect::Heal { amount } => sim.heal(owner, owner, amount.max(0) as usize),
                Effect::Invulnerable { ticks } => {
                    let mut b = crate::timed(INVULN, ticks as usize);
                    b.damaged_reduce = 100;
                    b.base_attack_damaged_reduce = 100;
                    b.skill_damaged_reduce = 100;
                    b.cc_immune = true;
                    replace_buff(sim, owner, &b);
                }
                Effect::Teleport { to } | Effect::Hop { to } => {
                    // the finale behind the target, or the dash's hop over a wall: onto open ground only
                    let (ox, oy) = crate::place::safe(&all, tick, (to.x, to.y), (to.x, to.y));
                    sim.entity_set_pos(owner, ox.max(0) as u64, oy.max(0) as u64);
                    crate::steve::note_placed(sim, owner, ox, oy);
                    sim.add_buff(owner, &crate::timed(crate::place::MOVED, 2));
                }
                Effect::Glide { to, ticks } => {
                    crate::motion::glide(sim, owner, (to.x, to.y), ticks, crate::motion::Kind::SelfMove);
                }
                Effect::Night { ticks } => {
                    self.night_until = tick + ticks as usize;
                    if self.art_v1 { replace_buff(sim, owner, &crate::timed(art::NIGHT_AURA, ticks as usize)); }
                }
                Effect::Steer { to } => {
                    remove_prefix(sim, owner, GO_PREFIX);
                    sim.add_buff(owner, &crate::timed(&format!("{GO_PREFIX}{}:{}", to.x, to.y), 6));
                }
                Effect::Fx { name, at, ticks, ambient } => {
                    // The original long orb remains the fallback for old definitions.
                    // New sheets renew one-frame phases from the orb's birth clock.
                    if self.art_v1 && name == "akn_orb" { continue; }
                    let (name, ticks) = if self.art_v1 {
                        let direction = if name == "akn_kunai" {
                            let p = sim.get_entity(owner).map(|e|e.pos()).unwrap_or_default();
                            art::heading(at.x-p.0 as i64, at.y-p.1 as i64)
                        } else if self.art_heading.is_empty() { "e" } else { self.art_heading };
                        let tag = if matches!(name.as_str(), "akn_kunai"|"akn_dash_slash"|"akn_night_slash") {format!("{name}_{direction}")} else {name};
                        let duration = if tag == "akn_night" { ticks.min(24) } else { ticks };
                        (tag, duration)
                    } else { (name, ticks) };
                    if self.fx_ok(tick as u64, ambient) {
                        crate::fx_point(sim, &name, owner, at.x, at.y, ticks);
                    }
                }
                Effect::Pose { name, ticks } => {
                    let name = if self.art_v1 { art::pose(name, ticks, self.core.dash.is_some()) } else {name};
                    sim.apply_cc(owner, &named(CcKindV1::Animation, ticks, name));
                }
                Effect::Log(line) => crate::mod_log(sim, "akane_log.txt", &format!("{tick}:{line}"), &line),
            }
        }
    }

    /// Crimson Night: her team's champions with no enemy eye (champion, minion or tower) within the nearsight radius
    /// are hidden from the enemy for the next few ticks.
    pub(crate) fn darkness(&self, sim: &mut StableSim, owner: usize) {
        if sim.tick() >= self.night_until {
            return;
        }
        let Some(team) = sim.get_entity(owner).map(|e| e.team()) else { return };
        let mut eyes = vec![];
        let mut mine = vec![];
        for e in (0..sim.entity_count()).filter_map(|i| sim.entity_at(i)) {
            if !e.is_alive() {
                continue;
            }
            let (x, y) = e.pos();
            if e.team() != team && (e.is_champion() || e.is_minion() || e.is_tower()) {
                eyes.push(Point::new(x as i64, y as i64));
            } else if e.team() == team && e.is_champion() {
                mine.push((e.id(), Point::new(x as i64, y as i64)));
            }
        }
        for (id, p) in mine {
            if !eyes.iter().any(|eye| eye.within(p, d::NIGHT_SIGHT)) {
                sim.entity_set_invisible(id, 3);
            }
        }
    }
}

impl StablePassive for Akane {
    fn configure(&mut self, params: &str) {
        self.art_v1=serde_json::from_str::<serde_json::Value>(params).ok().and_then(|v|v.get("art_v1").and_then(|n|n.as_i64())).is_some_and(|n|n==1);
    }
    fn clone_box(&self) -> Box<dyn StablePassive> {
        Box::new(self.clone())
    }
    fn on_spawn(&mut self, sim: &mut StableSim, _: usize, id: usize) {
        self.core.respawn(sim.tick() as u64);
        for b in [SOUL, INVULN, art::NIGHT_AURA] {
            sim.entity_remove_buff(id, b);
        }
        remove_prefix(sim, id, GO_PREFIX);
        self.basic_hits.clear();
        self.in_skill = false;
        self.night_until = 0;
        self.art_pos = None;
        self.art_heading = "e";
    }
    /// Her own hits never cross a cage or a Main Event ring; accepted basic attacks feed Crimson Edge.
    fn on_attack(&mut self, sim: &mut StableSim, _: usize, id: usize, target: usize, damage: &mut usize) {
        let all = crate::champions_shared(sim);
        let realms = crate::realm::realms(sim);
        if !crate::grimhald_cage::can_interact(&all, id, target) || !crate::realm::can_interact(&realms, id, target) {
            *damage = 0;
            return;
        }
        if !self.in_skill && !self.basic_hits.contains(&target) {
            self.basic_hits.push(target);
        }
    }
    fn on_kill(&mut self, sim: &mut StableSim, _: usize, _: usize, victim: usize) {
        if sim.get_entity(victim).is_some_and(|e| e.is_champion()) {
            self.takedown_at = Some(sim.tick());
            self.killed.retain(|k| *k != victim);
        }
    }
    fn on_assist(&mut self, sim: &mut StableSim, _: usize, _: usize) {
        self.takedown_at = Some(sim.tick());
    }
    fn on_update(&mut self, sim: &mut StableSim, _: u64, _: usize, id: usize) {
        let f = self.frame(sim, id);
        if !f.alive {
            if self.art_v1 { sim.entity_remove_buff(id, art::NIGHT_AURA); }
            self.art_pos=None;
            return;
        }
        if self.art_v1 {
            if let Some(p)=self.art_pos {if p!=f.pos {self.art_heading=art::heading(f.pos.x-p.x,f.pos.y-p.y);}}
            self.art_pos=Some(f.pos);
        }
        let effects = self.core.step(&f);
        self.apply(sim, id, effects);
        self.darkness(sim, id);
        if self.art_v1 && f.tick % art::ORB_RENEW == 0 {
            let phases: Vec<_>=self.core.orbs.iter().map(|o|(o.pos,art::orb_phase(f.tick,o.born))).collect();
            for (p,phase) in phases {
                if self.fx_ok(f.tick,false) {crate::fx_point(sim,&format!("akn_orb_p{phase}"),id,p.x,p.y,art::ORB_TAIL);}
            }
        }
    }
}
