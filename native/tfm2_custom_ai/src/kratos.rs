//! Kratos's bridge: reads the sim into a `kratos_core::Frame` once per tick and applies the core's effects.
//! The engine walks him and swings his basic attack (the data attack); native code casts Heimdall's Sight, the
//! Rampage, Spartan Guard, the Chain Dash and Fenrir (never() data shells).
//!
//! * Every visible move goes through `crate::motion` (round 156); the Chain Dash's hop over a wall is the one hand-set
//!   position (a blink over terrain, allowlisted in `motion_guard_tests.rs`).
//! * CC immunity and damage reduction are named own buffs (`krt_immune`, `krt_reduce`); shields go through
//!   `entity_add_shield` (the white bar over his HP).
//! * Heimdall's dodges: `on_damaged` refunds the first hit from each enemy champion during the window.
//! * Units behind a Grimhald cage or in another Main Event realm are untargetable in his frame, so no skill reaches them;
//!   single strikes (the Chain Dash) can be parried by V1.
use crate::kratos_core::{Core, Effect, Frame, Point, Unit};
use crate::kratos_data as d;
use mod_api_stable::{AttackTypeV1, BuffV1, CcKindV1, CcV1, StablePassive, StableSim};

pub(crate) const OWN_CC: &str = "krt_";
const BLADES: &str = "krt_blades";
const IMMUNE: &str = "krt_immune";
const REDUCE: &str = "krt_reduce";
const SIGHT: &str = "krt_sight";
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

#[derive(Clone, Default)]
pub struct Kratos {
    pub core: Core,
    presentation: crate::kratos_presentation::Presentation,
    profile: Option<crate::athlete_ai_core::Profile>,
    terrain: Option<(u64, Vec<bool>)>,
    basic_hits: Vec<usize>,
    in_skill: bool,
    fx_tokens: usize,
    fx_tick: u64,
}

impl Kratos {
    fn terrain(&mut self, tick: u64) -> Vec<bool> {
        if let Some((t, g)) = &self.terrain {
            if tick < t + 600 {
                return g.clone();
            }
        }
        let n = crate::kratos_core::CELLS;
        let c = crate::kratos_core::CELL;
        let g: Vec<bool> = (0..n * n).map(|i| crate::walls::wall_at(i % n * c + c / 2, i / n * c + c / 2)).collect();
        self.terrain = Some((tick, g.clone()));
        g
    }

    pub(crate) fn frame(&mut self, sim: &StableSim, owner: usize) -> Frame {
        let tick = sim.tick() as u64;
        let me = sim.get_entity(owner);
        let team = me.as_ref().map_or(usize::MAX, |e| e.team());
        let (pos, hp, max_hp, attack, alive) = me.as_ref().map_or((Point::default(), 0, 1, 0, false), |e| {
            let p = e.pos();
            let h = e.hp();
            (Point::new(p.0 as i64, p.1 as i64), h.0 as i64, h.1.max(1) as i64, e.stat().attack as i64, e.is_alive())
        });
        let profile = crate::athlete_ai::pinned_owner(sim, owner, &mut self.profile);
        let mut values = [50; 13];
        values[..9].copy_from_slice(&profile.values);
        values[9..].copy_from_slice(&profile.sense.0);
        let farming = crate::farming::Context::new(sim, owner);
        let realms = crate::realm::realms(sim);
        let all = crate::champions_shared(sim);
        let mut units = vec![];
        for e in (0..sim.entity_count()).filter_map(|i| sim.entity_at(i)) {
            if e.id() == owner || !e.is_alive() {
                continue;
            }
            let e_team = e.team();
            let visible = e_team == team || sim.is_visible(team, e.id());
            let p = e.pos();
            let h = e.hp();
            let buffs: Vec<_> = (0..e.buff_count()).filter_map(|i| e.buff_at(i)).collect();
            // a cage wall or another Main Event realm between them: nothing of his reaches it
            let reach = crate::realm::can_interact(&realms, owner, e.id()) && crate::grimhald_cage::can_interact(&all, owner, e.id());
            let targetable = e.is_targetable() && reach;
            let kind = e.kind();
            units.push(Unit {
                id: e.id(),
                team: e_team,
                pos: if visible { Point::new(p.0 as i64, p.1 as i64) } else { Point::default() },
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
                stunned: e.is_champion() && hard_cc(sim, e.id(), false),
            });
        }
        let ult_unlocked = me.as_ref().is_some_and(|e| (0..e.buff_count()).filter_map(|i| e.buff_at(i)).any(|b| b.name() == crate::ULT_LEARNED));
        Frame {
            tick,
            owner,
            team,
            alive,
            pos,
            hp,
            max_hp,
            attack,
            stunned: hard_cc(sim, owner, true),
            ult_unlocked,
            units,
            terrain: self.terrain(tick),
            profile: values,
            basic_hits: std::mem::take(&mut self.basic_hits),
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
                                crate::mod_log(sim, "kratos_log.txt", &format!("parry{tick}"), &format!("chain strike parried by {target}"));
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
                        let mut b = crate::timed("krt_slow", ticks as usize);
                        b.move_speed_mult = -(percent as i32);
                        replace_buff(sim, target, &b);
                    }
                }
                Effect::Freeze { target, ticks } => {
                    if reach(owner, target) && not_tower(sim, target) {
                        sim.apply_cc(target, &named(CcKindV1::Stun, ticks, "krt_freeze"));
                        replace_buff(sim, target, &crate::timed("krt_frozen", ticks as usize));
                    }
                }
                Effect::KnockUp { target, ticks } => {
                    if reach(owner, target) && not_tower(sim, target) {
                        sim.apply_cc(target, &named(CcKindV1::Airborne, ticks, "krt_knockup"));
                    }
                }
                Effect::Frost { target, stacks, ticks } => {
                    for i in 1..d::PASSIVE_FROST_STACKS {
                        sim.entity_remove_buff(target, &format!("krt_frost:{i}"));
                    }
                    if stacks > 0 {
                        sim.add_buff(target, &crate::timed(&format!("krt_frost:{stacks}"), ticks as usize));
                    }
                }
                Effect::Burn { target, ticks } => replace_buff(sim, target, &crate::timed("krt_burn", ticks as usize)),
                Effect::Weapon { blades } => {
                    sim.entity_remove_buff(owner, BLADES);
                    if blades {
                        let mut b = BuffV1::named(BLADES);
                        b.range = (d::BASIC_BLADES_RANGE - d::BASIC_AXE_RANGE).max(0) as usize;
                        sim.add_buff(owner, &b);
                    }
                }
                Effect::Shield { amount, ticks } => {
                    sim.entity_add_shield(owner, amount.max(0) as usize, ticks as usize);
                }
                Effect::Immune { ticks } => {
                    let mut b = crate::timed(IMMUNE, ticks as usize);
                    b.cc_immune = true;
                    replace_buff(sim, owner, &b);
                }
                Effect::Reduce { percent, ticks } => {
                    let mut b = crate::timed(REDUCE, ticks as usize);
                    b.damaged_reduce = percent.max(0) as usize;
                    replace_buff(sim, owner, &b);
                }
                Effect::Cleanse => {
                    sim.entity_clear_cc(owner);
                }
                Effect::Sight { ticks } => replace_buff(sim, owner, &crate::timed(SIGHT, ticks as usize)),
                Effect::Glide { to, ticks } => {
                    crate::motion::glide(sim, owner, (to.x, to.y), ticks, crate::motion::Kind::SelfMove);
                }
                Effect::Hop { to } => {
                    // the Chain Dash over a wall: a blink across terrain, onto open ground only
                    let (ox, oy) = crate::place::safe(&all, tick, (to.x, to.y), (to.x, to.y));
                    sim.entity_set_pos(owner, ox.max(0) as u64, oy.max(0) as u64);
                    crate::steve::note_placed(sim, owner, ox, oy);
                    sim.add_buff(owner, &crate::timed(crate::place::MOVED, 2));
                }
                Effect::Fx { name, at, ticks, ambient } => {
                    if name.starts_with("krt_fenrir_p") || self.fx_ok(tick as u64, ambient) {
                        let mut at=at;
                        let chain=if self.presentation.enabled && name=="krt_chain" {
                            sim.get_entity(owner).map(|e|{let p=e.pos();let from=Point::new(p.0 as i64,p.1 as i64);let heading=crate::kratos_core::heading(from,at);
                                let mode=if from.dist(at)>d::DASH_RANGE as f64+1000.0{"long"}else{"short"};
                                at=Point::new((from.x+at.x)/2,(from.y+at.y)/2);format!("krt_chain_{mode}_p{heading}")})
                        }else{None};
                        let name=if self.presentation.enabled && name.starts_with("krt_fenrir_p") {
                            let phase=(tick/3)%8;
                            if phase>0{format!("{name}_f{phase}")}else{name}
                        } else{chain.unwrap_or(name)};
                        crate::fx_point(sim, &name, owner, at.x, at.y, ticks);
                    }
                }
                Effect::Pose { name, ticks } => {
                    let name=if self.presentation.enabled {crate::kratos_presentation::pose(&self.core,name)}else{name};
                    self.presentation.action(name,tick as u64,ticks);
                    sim.apply_cc(owner, &named(CcKindV1::Animation, ticks, name));
                }
                Effect::Log(line) => crate::mod_log(sim, "kratos_log.txt", &format!("{tick}:{line}"), &line),
            }
        }
    }
}

impl StablePassive for Kratos {
    fn configure(&mut self,params:&str){self.presentation.enabled=serde_json::from_str::<serde_json::Value>(params).ok().and_then(|v|v.get("weapon_layer").and_then(|n|n.as_i64())).is_some_and(|n|n==1);}
    fn clone_box(&self) -> Box<dyn StablePassive> {
        Box::new(self.clone())
    }
    fn on_spawn(&mut self, sim: &mut StableSim, _: usize, id: usize) {
        self.presentation.respawn(sim);
        self.core.respawn(sim.tick() as u64);
        for b in [BLADES, IMMUNE, REDUCE, SIGHT] {
            sim.entity_remove_buff(id, b);
        }
        self.basic_hits.clear();
        self.in_skill = false;
    }
    /// His own hits never cross a cage or a Main Event ring; accepted basic attacks carry his weapon's mark.
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
    /// Heimdall's Sight: the first hit from each enemy champion while it's up is dodged (refunded).
    fn on_damaged(&mut self, sim: &mut StableSim, _: usize, id: usize, attacker: usize, damage: usize) {
        if damage == 0 {
            return;
        }
        let team = sim.get_entity(id).map(|e| e.team());
        let champion = sim.get_entity(attacker).is_some_and(|a| a.is_champion() && Some(a.team()) != team);
        if self.core.dodge(sim.tick() as u64, attacker, champion) {
            sim.heal(id, id, damage);
            if let Some(p) = sim.get_entity(id).map(|e| e.pos()) {
                crate::fx_point(sim, "krt_sight", id, p.0 as i64, p.1 as i64, 12);
            }
            crate::mod_log(sim, "kratos_log.txt", &format!("dodge{}:{attacker}", sim.tick()), &format!("Heimdall's Sight dodged {damage} from {attacker}"));
        }
    }
    fn on_update(&mut self, sim: &mut StableSim, _: u64, player: usize, id: usize) {
        let f = self.frame(sim, id);
        if !f.alive {
            self.presentation.clear(sim);
            return;
        }
        let effects = self.core.step(&f);
        self.apply(sim, id, effects);
        self.presentation.update(sim,player,&f,&self.core);
    }
}
