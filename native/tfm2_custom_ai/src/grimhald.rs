//! Grimhald's bridge: reads the sim into a `grimhald_core::Frame` once per tick and applies the core's effects.
//! The engine walks him and swings his basic attacks; native code casts Iron Ghost, Stone Bulwark, Skull Bash and
//! Warden's Verdict (the data slots are never() shells). The pentagon itself is held by `grimhald_cage::tick` in the
//! match hook.
use crate::grimhald_cage as cage;
use crate::grimhald_core::{self as core, Core, Effect, Frame, Point, Seg, Unit};
use crate::grimhald_data as d;
use mod_api_stable::{AttackTypeV1, BuffV1, CcKindV1, CcV1, StablePassive, StableSim};
use std::collections::BTreeMap;

/// The CCs Grimhald puts on himself or others are named, so his own never read as "held" (Contender's lesson).
pub(crate) const OWN_CC: &str = "grh_";

#[derive(Clone, Default)]
pub struct Grimhald {
    pub core: Core,
    profile: Option<crate::athlete_ai_core::Profile>,
    history: BTreeMap<usize, (u64, Point)>,
    terrain: Option<(u64, Vec<bool>)>,
    /// set while his own skill damage is dealt, so on_attack doesn't treat it as a basic
    in_skill: bool,
    fx_tokens: usize,
    fx_tick: u64,
    tail_pct: i64,
    cage_owned: bool,
    visual_key: Vec<(String, Point, u64)>,
    next_visual: u64,
    weapon:crate::grimhald_weapon::Weapon,
}

fn champ_has(sim: &StableSim, id: usize, prefix: &str) -> bool {
    sim.get_entity(id).is_some_and(|e| (0..e.buff_count()).filter_map(|i| e.buff_at(i)).any(|b| b.name().starts_with(prefix)))
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
fn named(kind: CcKindV1, ticks: u64, name: &str) -> CcV1 {
    let mut c = CcV1::of_kind(kind, ticks);
    c.set_name(name);
    c
}
fn wall_name(a: Point, b: Point) -> String {
    format!("sbw:{}:{}:{}:{}:", a.x, a.y, b.x, b.y)
}

const HARD: [CcKindV1; 7] = [CcKindV1::Stun, CcKindV1::Airborne, CcKindV1::Bind, CcKindV1::BlockSkill, CcKindV1::Taunt, CcKindV1::Fear, CcKindV1::Charm];

impl Grimhald {
    fn terrain(&mut self, tick: u64) -> Vec<bool> {
        if let Some((t, g)) = &self.terrain {
            if tick < t + 600 {
                return g.clone();
            }
        }
        let n = core::CELLS;
        let g: Vec<bool> = (0..n * n).map(|i| crate::walls::wall_at(i % n * core::CELL + core::CELL / 2, i / n * core::CELL + core::CELL / 2)).collect();
        self.terrain = Some((tick, g.clone()));
        g
    }

    pub(crate) fn frame(&mut self, sim: &StableSim, owner: usize) -> Frame {
        let tick = sim.tick() as u64;
        let all = crate::champions_shared(sim);
        let me = sim.get_entity(owner);
        let team = me.as_ref().map_or(usize::MAX, |e| e.team());
        let (pos, hp, max_hp, attack, ms, alive) = me.as_ref().map_or((Point::default(), 0, 1, 0, 1000, false), |e| {
            let p = e.pos();
            let h = e.hp();
            (Point::new(p.0 as i64, p.1 as i64), h.0 as i64, h.1.max(1) as i64, e.stat().attack as i64, e.stat().move_speed as i64, e.is_alive())
        });
        let hard_cc_ticks = me.as_ref().map_or(0, |e| {
            (0..e.cc_count()).filter_map(|i| e.cc_at(i)).filter(|c| !c.name().starts_with(OWN_CC) && HARD.iter().any(|k| k.code() == c.kind)).map(|c| c.tick).max().unwrap_or(0)
        });
        let profile = crate::athlete_ai::pinned_owner(sim, owner, &mut self.profile);
        let mut values = [50; 13];
        values[..9].copy_from_slice(&profile.values);
        values[9..].copy_from_slice(&profile.sense.0);
        let farming = crate::farming::Context::new(sim, owner);
        let realms = crate::realm::realms(sim);
        let mine = |id| crate::realm::realm_of(&realms, id);
        let my_realm = mine(owner);
        let member = |id: usize| all.iter().find(|c| c.id == id).and_then(cage::member_of);
        let mut history = BTreeMap::new();
        let mut units = vec![];
        for e in (0..sim.entity_count()).filter_map(|i| sim.entity_at(i)) {
            if e.id() == owner || !e.is_alive() || mine(e.id()) != my_realm {
                continue;
            }
            let e_team = e.team();
            let visible = e_team == team || sim.is_visible(team, e.id());
            let p = e.pos();
            let pos = Point::new(p.0 as i64, p.1 as i64);
            let velocity = if visible {
                history.insert(e.id(), (tick, pos));
                self.history
                    .get(&e.id())
                    .filter(|(t, _)| tick > *t && tick - *t < 120)
                    .map(|(t, q)| Point::new((pos.x - q.x) / (tick - t) as i64, (pos.y - q.y) / (tick - t) as i64))
                    .unwrap_or_default()
            } else {
                Point::default()
            };
            let h = e.hp();
            let buffs: Vec<_> = (0..e.buff_count()).filter_map(|i| e.buff_at(i)).collect();
            let targetable = e.is_targetable();
            let controlled = (0..e.cc_count()).filter_map(|i| e.cc_at(i)).any(|c| c.kind == CcKindV1::Stun.code() || c.kind == CcKindV1::Airborne.code());
            let kind = e.kind();
            units.push(Unit {
                id: e.id(),
                team: e_team,
                pos: if visible { pos } else { Point::default() },
                hp: h.0 as i64,
                max_hp: h.1.max(1) as i64,
                radius: e.radius().max(1000) as i64,
                champion: e.is_champion(),
                tower: e.is_tower(),
                visible,
                targetable,
                farmable: farming.calm()
                    && e_team != team
                    && targetable
                    && visible
                    && !e.is_champion()
                    && farming.eligible_known(e.id(), e_team, kind, p, kind.is_none() && e.is_minion(), &buffs),
                controlled,
                velocity,
                caged_by: if e.is_champion() { member(e.id()) } else { None },
            });
        }
        self.history = history;
        let pillar = self.core.pillar_seg();
        let walls: Vec<Seg> = crate::steve::walls_up(&all, tick as usize)
            .iter()
            .map(|w| {
                let (ex, ey) = w.built_end(tick as usize);
                Seg { a: Point::new(w.ax, w.ay), b: Point::new(ex, ey) }
            })
            .filter(|s| pillar.is_none_or(|p| p != *s))
            .collect();
        let my_cage = all.iter().find(|c| c.id == owner).and_then(cage::member_of).and_then(|o| cage::cages(&all, tick as usize).into_iter().find(|c| c.owner == o));
        let danger = all.iter().find(|c| c.id == owner).is_some_and(|m| {
            crate::batch2::danger_zones(&all, team)
                .iter()
                .any(|z| (z.0 - pos.x).pow(2) + (z.1 - pos.y).pow(2) <= z.2.pow(2) && my_cage.is_none_or(|c| (z.0, z.1) != (c.center.x, c.center.y)))
                && m.hp > 0
        });
        Frame {
            tick,
            owner,
            team,
            alive,
            pos,
            hp,
            max_hp,
            attack,
            move_speed: ms,
            hard_cc_ticks,
            ult_unlocked: champ_has(sim, owner, crate::ULT_LEARNED),
            units,
            terrain: self.terrain(tick),
            walls,
            profile: values,
            danger,
            cage: my_cage,
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
        let reach = |a: usize, b: usize| crate::realm::can_interact(&realms, a, b) && cage::can_interact(&all, a, b);
        for e in effects {
            match e {
                Effect::Damage { target, amount } => {
                    if !reach(owner, target) {
                        continue;
                    }
                    self.in_skill = true;
                    sim.deal_damage(owner, target, amount.max(0) as usize, 0, AttackTypeV1::Skill);
                    self.in_skill = false;
                }
                Effect::Airborne { target, ticks } => {
                    if reach(owner, target) {
                        sim.apply_cc(target, &named(CcKindV1::Airborne, ticks, "grh_up"));
                    }
                }
                Effect::Stun { target, ticks } => {
                    if reach(owner, target) {
                        sim.apply_cc(target, &named(CcKindV1::Stun, ticks, "grh_stun"));
                    }
                }
                Effect::Slow { target, percent, ticks } => {
                    if reach(owner, target) {
                        sim.entity_remove_buff(target, "grh_slow");
                        let mut b = crate::timed("grh_slow", ticks as usize);
                        b.move_speed_mult = -(percent as i32);
                        sim.add_buff(target, &b);
                    }
                }
                Effect::Place { target, pos, .. } => {
                    if reach(owner, target) {
                        sim.entity_set_pos(target, pos.x.max(0) as u64, pos.y.max(0) as u64);
                        crate::steve::note_placed(sim, target, pos.x, pos.y);
                        sim.add_buff(target, &crate::timed(crate::place::MOVED, 2));
                    }
                }
                Effect::Move { pos } => {
                    sim.entity_set_pos(owner, pos.x.max(0) as u64, pos.y.max(0) as u64);
                    crate::steve::note_placed(sim, owner, pos.x, pos.y);
                    sim.add_buff(owner, &crate::timed(crate::place::MOVED, 2));
                }
                Effect::Banish { ticks } => {
                    sim.entity_banish(owner, owner, ticks as usize, "", "");
                }
                Effect::Invulnerable { ticks } => {
                    sim.entity_remove_buff(owner, "grh_inv");
                    let mut b = crate::timed("grh_inv", ticks as usize);
                    b.damaged_reduce = 100;
                    b.cc_immune = true;
                    sim.add_buff(owner, &b);
                }
                Effect::ClearCc => {
                    sim.entity_clear_cc(owner);
                }
                Effect::Speed { percent, .. } => {
                    self.tail_pct = percent;
                }
                Effect::Passive { damage_pct, move_pct, tier } => {
                    sim.entity_remove_buff(owner, "grh_bloodrage");
                    let mut b = BuffV1::named("grh_bloodrage");
                    b.attack_mult = damage_pct as i32;
                    b.move_speed_mult = move_pct as i32;
                    sim.add_buff(owner, &b);
                    remove_prefix(sim, owner, "grh_rage");
                    if tier > 0 {
                        sim.add_buff(owner, &BuffV1::named(&format!("grh_rage{tier}")));
                    }
                }
                Effect::WallUp { a, b, rise, until } => {
                    let tend = until as usize;
                    let name = format!("{}{}:{}:{}", wall_name(a, b), tick, tick + rise as usize, tend);
                    sim.add_buff(owner, &crate::timed(&name, tend.saturating_sub(tick) + 5));
                    crate::steve::note_map_buff(sim.seed());
                }
                Effect::WallDown { a, b } => {
                    remove_prefix(sim, owner, &wall_name(a, b));
                }
                Effect::CageUp { cage: c } => {
                    sim.add_buff(owner, &crate::timed(&cage::cage_buff(&c), c.until.saturating_sub(tick as u64) as usize + 1));
                    cage::mark_members(sim, &all, &c, tick);
                    let leap = d::ULT_LEAP_TICKS as usize + 1;
                    sim.add_buff(owner, &crate::timed(cage::LEAP_BUFF, leap));
                    self.cage_owned = true;
                    crate::mod_log(sim, "grimhald_log.txt", &format!("cage{tick}"), &format!("cage up at {},{} until {}", c.center.x, c.center.y, c.until));
                }
                Effect::CageDown => {
                    cage::release(sim, &crate::champions_shared(sim), owner);
                    self.cage_owned = false;
                }
                Effect::Fx { name, at, ticks, ambient } => {
                    // A single state-driven pentagon replaces the old five
                    // generic edge copies. The core event remains unchanged.
                    if name == "grh_cage_rise" || name == "grh_pillar_rise" {
                        continue;
                    }
                    // the flying axe is one sprite per tick and never budgeted
                    if name.starts_with("grh_axe_p") || self.fx_ok(tick as u64, ambient) {
                        let visual=crate::grimhald_presentation::effect_tag(&name,&self.core);
                        crate::fx_point(sim,visual.as_deref().unwrap_or(&name),owner,at.x,at.y,ticks);
                    }
                }
                Effect::Pose { name, ticks } => {
                    self.weapon.pose(name,tick as u64,ticks);
                    // an Animation CC is a real CC: only for the cast itself
                    sim.apply_cc(owner, &named(CcKindV1::Animation, ticks, name));
                }
                Effect::Log(line) => crate::mod_log(sim, "grimhald_log.txt", &format!("{tick}:{line}"), &line),
            }
        }
    }

    fn present(&mut self, sim: &mut StableSim, owner: usize, f: &Frame) {
        let cues=crate::grimhald_presentation::cues(&self.core,f.tick,f.alive);
        let key:Vec<_>=cues.iter().map(|c|(c.tag.clone(),if c.tag=="ult_marker" {Point::default()} else {c.at},c.start_tick)).collect();
        if key==self.visual_key && f.tick<self.next_visual {return;}
        self.visual_key=key;
        self.next_visual=f.tick+crate::grimhald_presentation::RENEW_TICKS;
        remove_prefix(sim,owner,"grh_warn:");
        if let Some((at,impact))=core::ult_projection(self.core.ult) {
            let name=format!("grh_warn:{}:{}:{}:{}",at.x,at.y,d::ULT_KNOCK_RADIUS,impact);
            sim.add_buff(owner,&crate::timed(&name,crate::grimhald_presentation::RENEW_TICKS as usize));
        }
        for c in cues {
            if self.fx_ok(f.tick,false) {
                crate::fx_point(sim,&format!("grh_{}",c.tag),owner,c.at.x,c.at.y,c.ticks);
            }
        }
    }

    /// Iron Ghost's speed tail, fading in three steps.
    fn tail(&mut self, sim: &mut StableSim, owner: usize) {
        let Some((start, until)) = self.core.tail else {
            if self.tail_pct != 0 {
                sim.entity_remove_buff(owner, "grh_tail");
                self.tail_pct = 0;
            }
            return;
        };
        let tick = sim.tick() as u64;
        let len = (until - start).max(1);
        let left = until.saturating_sub(tick);
        let pct = (d::GHOST_TAIL_SPEED_PCT * (left as i64 * 3 / len as i64 + 1) / 3).clamp(0, d::GHOST_TAIL_SPEED_PCT);
        if pct != self.tail_pct || (tick - start) % 50 == 0 {
            sim.entity_remove_buff(owner, "grh_tail");
            let mut b = crate::timed("grh_tail", left as usize + 1);
            b.move_speed_mult = pct as i32;
            sim.add_buff(owner, &b);
            self.tail_pct = pct;
        }
    }
}

impl StablePassive for Grimhald {
    fn configure(&mut self,params:&str){self.weapon.enabled=serde_json::from_str::<serde_json::Value>(params).ok().and_then(|v|v.get("weapon_layer").and_then(|n|n.as_i64())).is_some_and(|n|n==1);}
    fn clone_box(&self) -> Box<dyn StablePassive> {
        Box::new(self.clone())
    }
    fn on_spawn(&mut self, sim: &mut StableSim, _: usize, id: usize) {
        self.weapon.respawn(sim);
        self.core.respawn(sim.tick() as u64);
        self.visual_key.clear();
        self.next_visual=0;
        remove_prefix(sim, id, "sbw:");
        if self.cage_owned {
            cage::release(sim, &crate::champions_shared(sim), id);
            self.cage_owned = false;
        }
    }
    fn on_dead(&mut self, sim: &mut StableSim, player: usize) {
        self.weapon.clear(sim);
        if let Some(id)=sim.get_player(player).and_then(|p|p.champion()).map(|e|e.id()) {
            remove_prefix(sim,id,"grh_warn:");
        }
    }
    /// His own hits: never across a cage or a Main Event ring; basics +20% on knocked-up or stunned enemies.
    fn on_attack(&mut self, sim: &mut StableSim, _: usize, id: usize, target: usize, damage: &mut usize) {
        let all = crate::champions_shared(sim);
        if !cage::can_interact(&all, id, target) {
            *damage = 0;
            return;
        }
        if self.in_skill {
            return;
        }
        let controlled = sim
            .get_entity(target)
            .is_some_and(|e| (0..e.cc_count()).filter_map(|i| e.cc_at(i)).any(|c| c.kind == CcKindV1::Stun.code() || c.kind == CcKindV1::Airborne.code()));
        if controlled {
            *damage = *damage * (100 + d::BASIC_BRUISER_BONUS_PCT as usize) / 100;
        }
        if self.fx_ok(sim.tick() as u64,false) {
            if let (Some(a),Some(b))=(sim.get_entity(id),sim.get_entity(target)) {
                let a=a.pos();let b=b.pos();
                let tag=if b.0<a.0 {"grh_slash_l"} else {"grh_slash"};
                crate::fx_point(sim,tag,id,(a.0 as i64+b.0 as i64)/2,(a.1 as i64+b.1 as i64)/2,14);
            }
        }
    }
    fn on_update(&mut self, sim: &mut StableSim, _: u64, player: usize, id: usize) {
        let f = self.frame(sim, id);
        if !f.alive {
            self.weapon.clear(sim);
            self.visual_key.clear();
            remove_prefix(sim,id,"grh_warn:");
            return;
        }
        let effects = self.core.step(&f);
        self.apply(sim, id, effects);
        self.tail(sim, id);
        self.present(sim,id,&f);
        self.weapon.update(sim,player,id,&f,self.core.ult);
    }
}
