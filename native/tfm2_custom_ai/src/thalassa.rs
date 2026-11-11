//! Thalassa's bridge: reads the sim into a `thalassa_core::Frame` once per tick and applies the core's effects.
//! The engine walks her and fires her harp bolts (the data basic attack); native code casts Tide Pool, Dive, Tide Twin
//! and the Trident (never() data shells).
//!
//! Her pools are buffs on her (`tls_pool:<id>:x:y:r:until:w`), so both simulations agree; whirlpools are published as
//! danger zones (`batch2::danger_zones`). Her clone is a hidden carrier unit (Raze's pattern) whose body is drawn as an
//! effect; it can be attacked and killed. Every visible move goes through `crate::motion` (round 156).
use crate::thalassa_core::{CloneView, Core, Effect, Frame, Point, Unit};
use crate::thalassa_data as d;
use mod_api_stable::{AttackTypeV1, BuffV1, CcKindV1, CcV1, StablePassive, StableSim, StatV1, UnitAttackV1};
use std::collections::BTreeMap;

pub(crate) const OWN_CC: &str = "tls_";
pub(crate) const POOL_PREFIX: &str = "tls_pool:";
const WATER: &str = "tls_water";
const HARD: [CcKindV1; 6] = [CcKindV1::Stun, CcKindV1::Airborne, CcKindV1::Taunt, CcKindV1::Fear, CcKindV1::Charm, CcKindV1::BlockSkill];

/// (x, y, r, until, whirl) for every pool buff on `c`.
pub fn pools_of(c: &crate::Champ) -> Vec<(i64, i64, i64, usize, bool)> {
    c.buffs
        .iter()
        .filter_map(|b| {
            let v: Vec<i64> = b.name().strip_prefix(POOL_PREFIX)?.split(':').filter_map(|s| s.parse().ok()).collect();
            (v.len() == 6).then(|| (v[1], v[2], v[3], v[4].max(0) as usize, v[5] == 1))
        })
        .collect()
}

#[derive(Clone, Default)]
pub struct Thalassa {
    pub core: Core,
    r165_art: bool,
    profile: Option<crate::athlete_ai_core::Profile>,
    history: BTreeMap<usize, (u64, Point)>,
    terrain: Option<(u64, Vec<bool>)>,
    clone: Option<usize>,
    basic_hits: Vec<usize>,
    in_skill: bool,
    fx_tokens: usize,
    fx_tick: u64,
    // Presentation clocks never participate in core decisions or combat randomness.
    pool_shown: BTreeMap<u32, usize>,
    twin_pose: &'static str,
    twin_pose_start: usize,
    twin_pose_until: usize,
    twin_last_pos: Option<(i64, i64)>,
    twin_left: bool,
    trident_heading: i64,
    dive_exit_at: Option<usize>,
    twin_dive_exit_at: Option<usize>,
    /// the trident's clone of her (a second hidden carrier) and its presentation state
    echo: Option<usize>,
    echo_pose: &'static str,
    echo_pose_start: usize,
    echo_pose_until: usize,
    echo_last_pos: Option<(i64, i64)>,
    echo_left: bool,
}

/// A hidden, unattackable-by-itself carrier unit for one of her clones (Raze's pattern); its body is drawn as effects.
fn spawn_carrier(sim: &mut StableSim, owner: usize, team: usize, pos: Point, ticks: u64, hp_pct: i64, tag: &str) -> Option<usize> {
    let max = sim.get_entity(owner).map_or(1000, |e| e.hp().1);
    let stat = StatV1 { hp: (max as i64 * hp_pct / 100).max(1) as usize, defence: 15, magic_resistance: 15, move_speed: 1, ..StatV1::default() };
    let attack = UnitAttackV1 { attack: 0, attack_ratio: 0, range: 1, cooltime: 5_184_000, duration: 1, start_timing: 0, ..UnitAttackV1::default() };
    let host = sim.spawn_unit("alchemist", owner, team, pos.x.max(0) as u64, pos.y.max(0) as u64, ticks, &stat, &attack)?;
    // no Bind: it would hold the carrier against its own engine glides (flights, dives, steps)
    for k in [CcKindV1::BlockAttack, CcKindV1::BlockSkill] {
        sim.apply_cc(host, &named(k, ticks, &format!("tls_{tag}_hold")));
    }
    sim.apply_cc(host, &named(CcKindV1::Animation, ticks, &format!("tls_{tag}_hidden")));
    sim.add_buff(host, &crate::timed(&format!("tls_{tag}:{owner}"), ticks as usize));
    Some(host)
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
fn held(sim: &StableSim, id: usize, own: bool) -> bool {
    sim.get_entity(id).is_some_and(|e| {
        (0..e.cc_count()).filter_map(|i| e.cc_at(i)).any(|c| {
            !(own && c.name().starts_with(OWN_CC)) && !crate::motion::is_self_motion(&c) && [CcKindV1::Stun, CcKindV1::Airborne].iter().any(|k| k.code() == c.kind)
        })
    })
}

impl Thalassa {
    fn terrain(&mut self, tick: u64) -> Vec<bool> {
        if let Some((t, g)) = &self.terrain {
            if tick < t + 600 {
                return g.clone();
            }
        }
        let n = crate::thalassa_core::CELLS;
        let c = crate::thalassa_core::CELL;
        let g: Vec<bool> = (0..n * n).map(|i| crate::walls::wall_at(i % n * c + c / 2, i / n * c + c / 2)).collect();
        self.terrain = Some((tick, g.clone()));
        g
    }

    pub(crate) fn frame(&mut self, sim: &StableSim, owner: usize) -> Frame {
        let tick = sim.tick() as u64;
        let me = sim.get_entity(owner);
        let team = me.as_ref().map_or(usize::MAX, |e| e.team());
        let (pos, hp, max_hp, ap, alive, shield) = me.as_ref().map_or((Point::default(), 0, 1, 0, false, 0), |e| {
            let p = e.pos();
            let h = e.hp();
            (Point::new(p.0 as i64, p.1 as i64), h.0 as i64, h.1.max(1) as i64, e.stat().magic_power as i64, e.is_alive(), e.shield() as i64)
        });
        let stunned = me.as_ref().is_some_and(|e| {
            (0..e.cc_count()).filter_map(|i| e.cc_at(i)).any(|c| !c.name().starts_with(OWN_CC) && !crate::motion::is_self_motion(&c) && HARD.iter().any(|k| k.code() == c.kind))
        });
        let profile = crate::athlete_ai::pinned_owner(sim, owner, &mut self.profile);
        let mut values = [50; 13];
        values[..9].copy_from_slice(&profile.values);
        values[9..].copy_from_slice(&profile.sense.0);
        let farming = crate::farming::Context::new(sim, owner);
        let realms = crate::realm::realms(sim);
        let my_realm = crate::realm::realm_of(&realms, owner);
        let mut history = BTreeMap::new();
        let mut units = vec![];
        for e in (0..sim.entity_count()).filter_map(|i| sim.entity_at(i)) {
            if e.id() == owner || Some(e.id()) == self.clone || Some(e.id()) == self.echo || !e.is_alive() || crate::realm::realm_of(&realms, e.id()) != my_realm {
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
                stunned: e.is_champion() && held(sim, e.id(), false),
                velocity,
            });
        }
        self.history = history;
        let clone = self.clone.and_then(|id| sim.get_entity(id).filter(|e| e.is_alive())).map(|e| {
            let p = e.pos();
            CloneView { id: e.id(), pos: Point::new(p.0 as i64, p.1 as i64), hp: e.hp().0 as i64 }
        });
        if clone.is_none() {
            self.clone = None;
        }
        let echo = self.echo.and_then(|id| sim.get_entity(id).filter(|e| e.is_alive())).map(|e| {
            let p = e.pos();
            CloneView { id: e.id(), pos: Point::new(p.0 as i64, p.1 as i64), hp: e.hp().0 as i64 }
        });
        if echo.is_none() {
            self.echo = None;
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
            ap,
            shield,
            stunned,
            ult_unlocked,
            units,
            terrain: self.terrain(tick),
            profile: values,
            clone,
            echo,
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
        let team = sim.get_entity(owner).map_or(usize::MAX, |e| e.team());
        // a V1 who parries the trident takes nothing from that throw
        let mut parried: Option<usize> = None;
        let mut dive_started = false;
        for e in effects {
            match e {
                Effect::TridentHit { target, ticks, .. } => {
                    if reach(owner, target) {
                        let estimate = d::ULT_BASE as usize + sim.get_entity(owner).map_or(0, |e| e.stat().magic_power) * d::ULT_AP_RATIO as usize / 100;
                        let champ = all.iter().find(|c| c.id == target).cloned();
                        if champ.is_some_and(|c| crate::batch2::try_parry(sim, &c, owner, estimate)) {
                            parried = Some(target);
                            crate::mod_log(sim, "thalassa_log.txt", &format!("parry{tick}"), &format!("trident parried by {target}"));
                        } else {
                            sim.apply_cc(target, &named(CcKindV1::Stun, ticks, "tls_stun"));
                        }
                    }
                }
                Effect::Damage { target, amount, .. } => {
                    if parried != Some(target) && reach(owner, target) {
                        self.in_skill = true;
                        sim.deal_damage(owner, target, 0, amount.max(0) as usize, AttackTypeV1::Skill);
                        self.in_skill = false;
                    }
                }
                Effect::Stun { target, ticks } => {
                    if reach(owner, target) {
                        sim.apply_cc(target, &named(CcKindV1::Stun, ticks, "tls_stun"));
                    }
                }
                Effect::KnockUp { target, ticks } => {
                    if reach(owner, target) && sim.get_entity(target).is_some_and(|e| !e.is_tower()) {
                        sim.apply_cc(target, &named(CcKindV1::Airborne, ticks, "tls_knockup"));
                    }
                }
                Effect::Slow { target, percent, ticks } => {
                    if reach(owner, target) {
                        sim.entity_remove_buff(target, "tls_slow");
                        let mut b = crate::timed("tls_slow", ticks as usize);
                        b.move_speed_mult = -(percent as i32);
                        sim.add_buff(target, &b);
                    }
                }
                Effect::Pull { target, to, ticks } => {
                    if reach(owner, target) {
                        // the surge's single pull: never through walls, cages or onto terrain
                        let from = sim.get_entity(target).map(|e| e.pos()).map_or((to.x, to.y), |p| (p.0 as i64, p.1 as i64));
                        let to = crate::place::safe(&all, tick, from, (to.x, to.y));
                        crate::motion::glide(sim, target, to, ticks, crate::motion::Kind::Knock);
                    }
                }
                Effect::Glide { to, ticks } => {
                    crate::motion::glide(sim, owner, (to.x, to.y), ticks, crate::motion::Kind::SelfMove);
                }
                Effect::Submerge { ticks } => {
                    dive_started = true;
                    // under water: unseen and untouchable, but not crowd-control immune (that would stop the glide)
                    sim.entity_set_invisible(owner, ticks as usize);
                    sim.entity_remove_buff(owner, "tls_dive");
                    let mut b = crate::timed("tls_dive", ticks as usize);
                    b.damaged_reduce = 100;
                    sim.add_buff(owner, &b);
                    self.dive_exit_at = Some(tick + ticks as usize);
                }
                Effect::Heal { amount } => sim.heal(owner, owner, amount.max(0) as usize),
                Effect::Shield { target, amount, ticks } => {
                    sim.entity_add_shield(target, amount.max(0) as usize, ticks as usize);
                    if target != owner {
                        sim.add_buff(target, &crate::timed("tls_rescued", ticks as usize));
                    }
                }
                Effect::Cleanse { target } => {
                    sim.entity_clear_cc(target);
                }
                Effect::CcImmune { target, ticks } => {
                    let mut b = crate::timed("tls_immune", ticks as usize);
                    b.cc_immune = true;
                    sim.add_buff(target, &b);
                }
                Effect::Water { on } => {
                    sim.entity_remove_buff(owner, WATER);
                    if on {
                        let mut b = BuffV1::named(WATER);
                        b.move_speed_mult = d::WATER_MOVE_SPEED_PCT as i32;
                        b.range = (d::BASIC_RANGE * d::WATER_RANGE_PCT / 100) as usize;
                        sim.add_buff(owner, &b);
                    }
                }
                Effect::PoolUp { id, center, radius, until, whirl } => {
                    let name = format!("{POOL_PREFIX}{id}:{}:{}:{radius}:{until}:{}", center.x, center.y, u8::from(whirl));
                    sim.add_buff(owner, &crate::timed(&name, (until as usize).saturating_sub(tick) + 2));
                    crate::steve::note_map_buff(sim.seed());
                    // The pool is drawn below with short, stable-clock phases so a removed
                    // pool cannot leave a ten-second visual copy behind.
                    let _ = whirl;
                }
                Effect::PoolDown { id } => {
                    remove_prefix(sim, owner, &format!("{POOL_PREFIX}{id}:"));
                    self.pool_shown.remove(&id);
                }
                Effect::PoolCalm { id } => {
                    // the swirl is over: republish it as a normal pool so other AIs stop reading a whirlpool
                    remove_prefix(sim, owner, &format!("{POOL_PREFIX}{id}:"));
                    if let Some(p) = self.core.pools.iter().find(|p| p.id == id) {
                        let name = format!("{POOL_PREFIX}{id}:{}:{}:{}:{}:0", p.center.x, p.center.y, p.radius, p.until);
                        sim.add_buff(owner, &crate::timed(&name, (p.until as usize).saturating_sub(tick) + 2));
                    }
                    crate::steve::note_map_buff(sim.seed());
                }
                Effect::SpawnClone { pos, ticks } => {
                    if let Some(old) = self.clone.take() {
                        sim.entity_set_hp(old, 0);
                    }
                    self.clone = spawn_carrier(sim, owner, team, pos, ticks, d::TWIN_HP_PCT, "twin");
                    if self.clone.is_none() {
                        crate::mod_log(sim, "thalassa_log.txt", &format!("twin{tick}"), "tide twin: the host refused the clone unit");
                    }
                }
                Effect::SpawnEcho { pos, ticks } => {
                    if let Some(old) = self.echo.take() {
                        sim.entity_set_hp(old, 0);
                    }
                    self.echo = spawn_carrier(sim, owner, team, pos, ticks, d::ULT_ECHO_HP_PCT, "echo");
                    self.echo_last_pos = None;
                    self.echo_pose_until = 0;
                    if self.echo.is_none() {
                        crate::mod_log(sim, "thalassa_log.txt", &format!("echo{tick}"), "trident echo: the host refused the clone unit");
                    }
                }
                Effect::EchoMove { to, ticks } => {
                    if let Some(c) = self.echo {
                        let from = sim.get_entity(c).map(|e| e.pos()).map_or((to.x, to.y), |p| (p.0 as i64, p.1 as i64));
                        let to = crate::place::safe(&all, tick, from, (to.x, to.y));
                        crate::motion::glide(sim, c, to, ticks, crate::motion::Kind::Knock);
                    }
                }
                Effect::RemoveEcho => {
                    if let Some(c) = self.echo.take() {
                        sim.entity_set_hp(c, 0);
                    }
                }
                Effect::EchoPose { name, ticks } => {
                    self.echo_pose = name;
                    self.echo_pose_start = tick;
                    self.echo_pose_until = tick + ticks as usize;
                }
                Effect::CloneMove { to, ticks } => {
                    // an engine glide of the carrier; its body effect is drawn every tick meanwhile, so it reads as a flight
                    if let Some(c) = self.clone {
                        let from = sim.get_entity(c).map(|e| e.pos()).map_or((to.x, to.y), |p| (p.0 as i64, p.1 as i64));
                        let to = crate::place::safe(&all, tick, from, (to.x, to.y));
                        if dive_started {
                            if self.fx_ok(tick as u64, false) { crate::fx_point(sim,"tls_dive_in",owner,from.0,from.1,18); }
                            self.twin_dive_exit_at=Some(tick+ticks as usize);
                        }
                        crate::motion::glide(sim, c, to, ticks, crate::motion::Kind::Knock);
                    }
                }
                Effect::RemoveClone => {
                    self.twin_dive_exit_at=None;
                    if let Some(c) = self.clone.take() {
                        sim.entity_set_hp(c, 0);
                    }
                }
                Effect::Swap { owner_to, clone_to } => {
                    // a deliberate swap through the tide link (a teleport by design), only onto open ground
                    let from = sim.get_entity(owner).map(|e| e.pos()).map_or((owner_to.x, owner_to.y), |p| (p.0 as i64, p.1 as i64));
                    let (ox, oy) = crate::place::safe(&all, tick, from, (owner_to.x, owner_to.y));
                    sim.entity_set_pos(owner, ox.max(0) as u64, oy.max(0) as u64);
                    crate::steve::note_placed(sim, owner, ox, oy);
                    sim.add_buff(owner, &crate::timed(crate::place::MOVED, 2));
                    if let Some(c) = self.clone {
                        let (cx, cy) = crate::place::safe(&all, tick, (ox, oy), (clone_to.x, clone_to.y));
                        sim.entity_set_pos(c, cx.max(0) as u64, cy.max(0) as u64);
                    }
                }
                Effect::Fx { name, at, ticks, ambient } => {
                    // The 18-tick surge is the accepted water-basic footprint;
                    // ordinary Tide Pool's surge retains its original artwork.
                    if self.r165_art && name=="tls_surge" && ticks==18 {
                        if let Some(p)=sim.get_entity(owner).map(|e|e.pos()) {
                            for (tag,x,y) in crate::round_165_art::wave_parts((p.0 as i64,p.1 as i64),(at.x,at.y)) {
                                if !self.fx_ok(tick as u64,false){break;}
                                crate::fx_point(sim,&format!("tls_{tag}"),owner,x,y,ticks);
                            }
                        }
                        continue;
                    }
                    if name.starts_with("tls_twin_p") || name == "tls_tide_link" {
                        // One body frame per tick is emitted by present(), including both facings.
                        continue;
                    }
                    if name.starts_with("tls_trident_p") {
                        if let Some(t) = self.core.trident { self.trident_heading = t.heading_mdeg; }
                        let phase = ((self.trident_heading + 22_500).rem_euclid(360_000) / 45_000) as usize;
                        crate::fx_point(sim, &format!("tls_trident_p{phase}"), owner, at.x, at.y, 1);
                    } else if self.fx_ok(tick as u64, ambient) {
                        crate::fx_point(sim, &name, owner, at.x, at.y, ticks);
                    }
                }
                Effect::Pose { name, ticks } => {
                    if name == "skill1" || name == "skill2" {
                        self.twin_pose = name;
                        self.twin_pose_start = tick;
                        self.twin_pose_until = tick + ticks as usize;
                    }
                    sim.apply_cc(owner, &named(CcKindV1::Animation, ticks, name));
                }
                Effect::Log(line) => crate::mod_log(sim, "thalassa_log.txt", &format!("{tick}:{line}"), &line),
            }
        }
    }

    pub(crate) fn present(&mut self, sim: &mut StableSim, owner: usize) {
        let tick = sim.tick();
        if self.dive_exit_at.is_some_and(|at| tick >= at) {
            self.dive_exit_at = None;
            if let Some(p) = sim.get_entity(owner).map(|e| e.pos()) {
                crate::fx_point(sim, "tls_dive_out", owner, p.0 as i64, p.1 as i64, 18);
            }
        }
        let every = crate::thalassa_art::POOL_RENEW_TICKS;
        let mut emitted = 0;
        for i in 0..self.core.pools.len() {
            let p = self.core.pools[i];
            if tick as u64 >= p.until || self.pool_shown.get(&p.id).is_some_and(|last| tick < last + every) { continue; }
            if emitted >= 4 || !self.fx_ok(tick as u64, false) { break; }
            let start = p.until.saturating_sub(d::WATER_POOL_TICKS as u64) as usize;
            let phase = tick.saturating_sub(start) / every % 8;
            let kind = if p.whirl { "whirlpool" } else { "pool" };
            crate::fx_point(sim, &format!("tls_{kind}_p{phase}"), owner, p.center.x, p.center.y, (every + 1) as u64);
            self.pool_shown.insert(p.id, tick);
            emitted += 1;
        }
        self.present_echo(sim, owner, tick);
        let Some(twin) = self.core.twin else { self.twin_last_pos = None; self.twin_dive_exit_at = None; return };
        let Some(c) = self.clone.and_then(|id| sim.get_entity(id)).filter(|e| e.is_alive()) else { return };
        let pos = c.pos();
        let pos = (pos.0 as i64, pos.1 as i64);
        if self.twin_dive_exit_at.is_some_and(|at|tick>=at) {
            self.twin_dive_exit_at=None;
            if self.fx_ok(tick as u64,false) { crate::fx_point(sim,"tls_dive_out",owner,pos.0,pos.1,18); }
        }
        if twin.linked && !twin.swapped {
            if let Some(p) = sim.get_entity(owner).map(|e| e.pos()) {
                for (tag,x,y) in crate::thalassa_art::link_segments((p.0 as i64,p.1 as i64),pos) {
                    crate::fx_point(sim,&format!("tls_{tag}"),owner,x,y,1);
                }
            }
        }
        if let Some(old) = self.twin_last_pos { if old.0 != pos.0 { self.twin_left = pos.0 < old.0; } }
        self.twin_last_pos = Some(pos);
        let moving = (tick as u64) < twin.moving_until;
        let (pose, elapsed, looping) = if tick < self.twin_pose_until {
            (self.twin_pose, tick.saturating_sub(self.twin_pose_start), false)
        } else if moving { ("run", tick, true) } else { ("idle", tick, true) };
        let frame = crate::thalassa_art::frame(pose, elapsed, looping);
        let facing = if self.twin_left { "_l" } else { "" };
        crate::fx_point(sim, &format!("tls_twin_{pose}_{frame}{facing}"), owner, pos.0, pos.1, 1);
    }
}

impl Thalassa {
    /// The trident's clone of her: one body frame per tick at its carrier, its own pose and facing.
    fn present_echo(&mut self, sim: &mut StableSim, owner: usize, tick: usize) {
        let Some(echo) = self.core.echo else { self.echo_last_pos = None; return };
        let Some(c) = self.echo.and_then(|id| sim.get_entity(id)).filter(|e| e.is_alive()) else { return };
        let pos = c.pos();
        let pos = (pos.0 as i64, pos.1 as i64);
        if let Some(old) = self.echo_last_pos { if old.0 != pos.0 { self.echo_left = pos.0 < old.0; } }
        self.echo_last_pos = Some(pos);
        let moving = (tick as u64) < echo.moving_until;
        let (pose, elapsed, looping) = if tick < self.echo_pose_until {
            (self.echo_pose, tick.saturating_sub(self.echo_pose_start), false)
        } else if moving { ("run", tick, true) } else { ("idle", tick, true) };
        let frame = crate::thalassa_art::frame(pose, elapsed, looping);
        let facing = if self.echo_left { "_l" } else { "" };
        crate::fx_point(sim, &format!("tls_twin_{pose}_{frame}{facing}"), owner, pos.0, pos.1, 1);
    }
}

impl StablePassive for Thalassa {
    fn configure(&mut self,params:&str){self.r165_art=serde_json::from_str::<serde_json::Value>(params).ok().and_then(|v|v.get("r165_art").and_then(|x|x.as_i64())).is_some_and(|v|v==1);}
    fn clone_box(&self) -> Box<dyn StablePassive> {
        Box::new(self.clone())
    }
    fn on_spawn(&mut self, sim: &mut StableSim, _: usize, id: usize) {
        self.core.respawn(sim.tick() as u64);
        remove_prefix(sim, id, POOL_PREFIX);
        sim.entity_remove_buff(id, WATER);
        if let Some(c) = self.clone.take() {
            sim.entity_set_hp(c, 0);
        }
        if let Some(c) = self.echo.take() {
            sim.entity_set_hp(c, 0);
        }
        self.basic_hits.clear();
        self.pool_shown.clear();
        self.twin_pose_until = 0;
        self.twin_last_pos = None;
        self.dive_exit_at = None;
        self.twin_dive_exit_at = None;
    }
    /// Her own hits never cross a cage or a Main Event ring; accepted harp bolts feed the tsunami and the twin's echo.
    fn on_attack(&mut self, sim: &mut StableSim, _: usize, id: usize, target: usize, damage: &mut usize) {
        let all = crate::champions_shared(sim);
        if !crate::grimhald_cage::can_interact(&all, id, target) {
            *damage = 0;
            return;
        }
        if !self.in_skill && !self.basic_hits.contains(&target) {
            self.basic_hits.push(target);
            self.twin_pose = "attack";
            self.twin_pose_start = sim.tick();
            self.twin_pose_until = sim.tick() + 22;
            if let (Some(c), Some(t)) = (self.clone.and_then(|c| sim.get_entity(c)), sim.get_entity(target)) {
                self.twin_left = t.pos().0 < c.pos().0;
            }
        }
    }
    fn on_update(&mut self, sim: &mut StableSim, _: u64, _: usize, id: usize) {
        let f = self.frame(sim, id);
        if !f.alive {
            return;
        }
        let effects = self.core.step(&f);
        self.apply(sim, id, effects);
        self.present(sim, id);
    }
}
