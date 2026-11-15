//! Basalt's bridge: reads the sim into a `basalt_core::Frame` once per tick, applies the core's effects and keeps his
//! walls in the world. The engine walks him and cracks his whip (the data basic attack); native code casts everything
//! else (the data slots are never-cast shells, like Kicker's and Kagami's).
//!
//! * His walls are the shared walls: one `sbw:ax:ay:bx:by:t0:tb:tend` buff on him per wall (Steve's format, also used
//!   by Grimhald's cage), renewed while it stands and rewritten when it moves. So everyone paths around them, walks,
//!   dashes and knockbacks stop at them, blinks never land in them and they block sight (`steve::enforce_walls`,
//!   `steve::wall_fog`). `steve::note_map_buff` switches the shared wall pathing on for the match.
//! * Airborne (Jump, double jump, wall kick, Whip Pull, Air Dash): `bs_air` (`cc_immune`, `ignore_wall`, basics miss)
//!   plus `place::MOVED` each tick, so the shared wall code lets him fly over every wall; his moves are
//!   `crate::motion::glide` (no hand-set positions here).
//! * Pushes and swings: `motion::glide` (a Knock); slams: a named stun; the ult's slows: `bs_slow`; its charge: a named
//!   Bind on himself (`bs_charge`).
//! * The AI's walk (`Effect::Steer`) is a short `bs_go:x:y` buff the shared input AI turns into a move order.
use crate::basalt_core::{self as core, Core, Effect, Frame, Point, Seg, Unit};
use crate::basalt_data as d;
use mod_api_stable::{AttackTypeV1, BuffV1, CcKindV1, CcV1, StablePassive, StableSim};

pub(crate) const GO_PREFIX: &str = "bs_go:";
pub(crate) const AIR: &str = "bs_air";
const WALL_PREFIX: &str = "sbw:";
/// how long a wall buff lives between renewals
const WALL_LEASE: u64 = 40;

fn named(kind: CcKindV1, ticks: u64, name: &str) -> CcV1 {
    let mut c = CcV1::of_kind(kind, ticks);
    c.set_name(name);
    c
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
const HARD: [CcKindV1; 5] = [CcKindV1::Stun, CcKindV1::Airborne, CcKindV1::Taunt, CcKindV1::Fear, CcKindV1::Charm];
fn hard_cc(sim: &StableSim, id: usize) -> bool {
    sim.get_entity(id).is_some_and(|e| {
        (0..e.cc_count())
            .filter_map(|i| e.cc_at(i))
            .any(|c| !c.name().starts_with("bs_") && !crate::motion::is_self_motion(&c) && HARD.iter().any(|k| k.code() == c.kind))
    })
}

/// The move a steering buff asks for (read by the shared input AI).
pub fn steer_of(name: &str) -> Option<(i64, i64)> {
    let mut it = name.strip_prefix(GO_PREFIX)?.split(':').filter_map(|s| s.parse::<i64>().ok());
    Some((it.next()?, it.next()?))
}

pub(crate) fn air_buff(ticks: u64) -> BuffV1 {
    let mut b = crate::timed(AIR, ticks as usize);
    b.cc_immune = true;
    b.ignore_wall = true;
    b.base_attack_damaged_reduce = 100;
    b
}

/// The shared-wall buff name for one of his walls.
pub(crate) fn wall_name(a: Point, b: Point, t0: u64, tend: u64) -> String {
    format!("{WALL_PREFIX}{}:{}:{}:{}:{t0}:{t0}:{tend}", a.x, a.y, b.x, b.y)
}

#[derive(Clone, Default)]
pub struct Basalt {
    pub core: Core,
    profile: Option<crate::athlete_ai_core::Profile>,
    terrain: Option<(u64, Vec<bool>)>,
    in_skill: bool,
    fx_tokens: usize,
    fx_tick: u64,
    /// the shared-wall buffs on him now: (wall id, buff name, lease end, drawn at)
    shown: Vec<(u32, String, u64, u64)>,
}

impl Basalt {
    fn grid(&mut self, tick: u64) -> Vec<bool> {
        if let Some((t, g)) = &self.terrain {
            if tick < t + 600 {
                return g.clone();
            }
        }
        let n = core::CELLS;
        let c = core::CELL;
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
        let snapshot = crate::champions_shared(sim);
        let realms = crate::realm::realms(sim);
        let mut units = vec![];
        for e in (0..sim.entity_count()).filter_map(|i| sim.entity_at(i)) {
            if e.id() == owner || !e.is_alive() {
                continue;
            }
            let e_team = e.team();
            let visible = e_team == team || sim.is_visible(team, e.id());
            let p = e.pos();
            let h = e.hp();
            let reach = crate::realm::can_interact(&realms, owner, e.id()) && crate::grimhald_cage::can_interact(&snapshot, owner, e.id());
            let kind = e.kind();
            units.push(Unit {
                id: e.id(),
                team: e_team,
                pos: if visible { Point::new(p.0 as i64, p.1 as i64) } else { Point::default() },
                hp: h.0 as i64,
                max_hp: h.1.max(1) as i64,
                attack: e.stat().attack as i64,
                champion: e.is_champion(),
                tower: e.is_tower() || kind == Some(mod_api_stable::EntityKindV1::Nexus),
                visible,
                targetable: e.is_targetable() && reach,
            });
        }
        // other champions' shared walls (Steve's boats, Grimhald's cage): his own are his core's
        let others: Vec<crate::Champ> = snapshot.iter().filter(|c| c.id != owner).cloned().collect();
        let other_walls = crate::steve::walls_up(&others, tick as usize)
            .into_iter()
            .map(|w| {
                let e = w.built_end(tick as usize);
                Seg {
                    a: Point::new(w.ax, w.ay),
                    b: Point::new(e.0, e.1),
                }
            })
            .collect();
        const HOME: [(i64, i64); 2] = [(96_000, 864_000), (864_000, 96_000)];
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
            stunned: hard_cc(sim, owner),
            ult_unlocked: sim.get_entity(owner).is_some_and(|e| (0..e.buff_count()).filter_map(|i| e.buff_at(i)).any(|b| b.name() == crate::ULT_LEARNED)),
            home: Point::new(home.0, home.1),
            units,
            terrain: self.grid(tick),
            other_walls,
            profile: values,
            manual: false,
            order: None,
        }
    }

    fn fx_ok(&mut self, tick: u64) -> bool {
        if tick != self.fx_tick {
            let gained = (tick.saturating_sub(self.fx_tick) / 2) as usize;
            self.fx_tokens = (self.fx_tokens + gained.max(1)).min(d::FX_TOKENS);
            self.fx_tick = tick;
        }
        if self.fx_tokens > 0 {
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
        let not_tower = |sim: &StableSim, id: usize| sim.get_entity(id).is_some_and(|e| !e.is_tower() && e.kind() != Some(mod_api_stable::EntityKindV1::Nexus));
        for e in effects {
            match e {
                Effect::Damage { target, amount } => {
                    if reach(owner, target) && not_tower(sim, target) {
                        self.in_skill = true;
                        sim.deal_damage(owner, target, amount.max(0) as usize, 0, AttackTypeV1::Skill);
                        self.in_skill = false;
                    }
                }
                Effect::Stun { target, ticks } => {
                    if reach(owner, target) && not_tower(sim, target) {
                        sim.apply_cc(target, &named(CcKindV1::Stun, ticks, "bs_slam"));
                    }
                }
                Effect::KnockUp { target, ticks } => {
                    if reach(owner, target) && not_tower(sim, target) {
                        sim.apply_cc(target, &named(CcKindV1::Airborne, ticks, "bs_knockup"));
                    }
                }
                Effect::Knock { target, to, ticks } => {
                    if reach(owner, target) && not_tower(sim, target) {
                        crate::motion::glide(sim, target, (to.x, to.y), ticks, crate::motion::Kind::Knock);
                    }
                }
                Effect::Glide { to, ticks } => {
                    crate::motion::glide(sim, owner, (to.x, to.y), ticks, crate::motion::Kind::SelfMove);
                }
                Effect::Air { ticks } => {
                    replace_buff(sim, owner, &air_buff(ticks));
                    // the shared wall code lets a deliberately moved unit through
                    replace_buff(sim, owner, &crate::timed(crate::place::MOVED, ticks as usize));
                }
                Effect::Slow { target, pct, ticks } => {
                    if reach(owner, target) && not_tower(sim, target) {
                        let mut b = crate::timed("bs_slow", ticks as usize);
                        b.move_speed_mult = -(pct.clamp(0, 90) as i32);
                        replace_buff(sim, target, &b);
                    }
                }
                Effect::Root { ticks } => {
                    // the ult's charge: he holds still (a named Bind on himself, never read as enemy control)
                    sim.apply_cc(owner, &named(CcKindV1::Bind, ticks, "bs_charge"));
                }
                Effect::Rubble { .. } => {}
                Effect::Steer { to } => {
                    remove_prefix(sim, owner, GO_PREFIX);
                    sim.add_buff(owner, &crate::timed(&format!("{GO_PREFIX}{}:{}", to.x, to.y), 6));
                }
                Effect::Fx { name, at, ticks } => {
                    if self.fx_ok(tick as u64) {
                        crate::fx_point(sim, &name, owner, at.x, at.y, ticks);
                    }
                }
                Effect::Log { .. } => {}
            }
        }
    }

    /// His walls in the world: one shared-wall buff each, renewed before its lease ends and rewritten when it moves;
    /// gone walls lose theirs. Each wall is drawn as a row of stone blocks.
    pub(crate) fn sync_walls(&mut self, sim: &mut StableSim, owner: usize) {
        let tick = sim.tick() as u64;
        let walls: Vec<core::StoneWall> = self.core.walls.clone();
        let mut next: Vec<(u32, String, u64, u64)> = vec![];
        let mut added = false;
        for w in &walls {
            let old = self.shown.iter().find(|s| s.0 == w.id).cloned();
            let moving = w.slide.is_some() || self.core.claps.iter().any(|c| c.left == w.id || c.right == w.id);
            let same_spot = old.as_ref().is_some_and(|o| o.1.starts_with(&format!("{WALL_PREFIX}{}:{}:{}:{}:", w.a.x, w.a.y, w.b.x, w.b.y)));
            let (name, tend, drawn) = match old {
                Some(o) if same_spot && tick + 10 < o.2 => (o.1, o.2, o.3),
                o => {
                    if let Some(o) = &o {
                        sim.entity_remove_buff(owner, &o.1);
                    }
                    let tend = tick + WALL_LEASE;
                    let name = wall_name(w.a, w.b, w.born.min(tick), tend);
                    sim.add_buff(owner, &crate::timed(&name, (WALL_LEASE + 5) as usize));
                    added = true;
                    (name, tend, o.map_or(0, |o| o.3))
                }
            };
            // the blocks: redrawn every 6 ticks while it moves, every 28 when it stands
            let every = if moving { 6 } else { 28 };
            let drawn = if tick >= drawn + every || drawn == 0 {
                let len = w.a.dist(w.b);
                let n = ((len / 14_000.0).ceil() as i64).max(1);
                for k in 0..=n {
                    let p = Point::new(w.a.x + (w.b.x - w.a.x) * k / n, w.a.y + (w.b.y - w.a.y) * k / n);
                    crate::fx_point(sim, "bs_block", owner, p.x, p.y, every + 2);
                }
                tick
            } else {
                drawn
            };
            next.push((w.id, name, tend, drawn));
        }
        for s in self.shown.iter().filter(|s| !walls.iter().any(|w| w.id == s.0)) {
            sim.entity_remove_buff(owner, &s.1);
        }
        self.shown = next;
        if added {
            crate::steve::note_map_buff(sim.seed());
        }
    }
}

impl StablePassive for Basalt {
    fn configure(&mut self, _: &str) {}
    fn clone_box(&self) -> Box<dyn StablePassive> {
        Box::new(self.clone())
    }
    fn on_spawn(&mut self, sim: &mut StableSim, _: usize, id: usize) {
        self.core.respawn(sim.tick() as u64);
        self.core.walls.clear();
        sim.entity_remove_buff(id, AIR);
        remove_prefix(sim, id, GO_PREFIX);
        remove_prefix(sim, id, WALL_PREFIX);
        self.shown.clear();
        self.in_skill = false;
    }
    /// His whip never crosses a cage or a Main Event ring; rubble adds to his next crack.
    fn on_attack(&mut self, sim: &mut StableSim, _: usize, id: usize, target: usize, damage: &mut usize) {
        let all = crate::champions_shared(sim);
        let realms = crate::realm::realms(sim);
        if !crate::grimhald_cage::can_interact(&all, id, target) || !crate::realm::can_interact(&realms, id, target) {
            *damage = 0;
            return;
        }
        if !self.in_skill {
            let mult = self.core.basic_multiplier(sim.tick() as u64);
            if mult != 100 {
                *damage = (*damage as i64 * mult / 100).max(0) as usize;
                self.core.take_rubble();
            }
        }
    }
    fn on_update(&mut self, sim: &mut StableSim, _: u64, _: usize, id: usize) {
        let f = self.frame(sim, id);
        if !f.alive {
            let _ = self.core.step(&f);
            self.core.walls.clear();
            self.sync_walls(sim, id);
            return;
        }
        let effects = self.core.step(&f);
        self.apply(sim, id, effects);
        self.sync_walls(sim, id);
    }
}
