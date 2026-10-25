//! Thin native Raze adapter. Engine owns gun attacks; core owns utility and movement.
use crate::raze_core::{
    self as core, Cmd, Core, CreditEvent, Frame, Hazard, Point, Profile, Rect, Unit, UtilityStatus,
};
use crate::valorant_guns::Gun;
use mod_api_stable::{
    AttackTypeV1, BuffV1, CastingTargetV1, CcKindV1, CcV1, EntityKindV1, InputV1,
    ProjectileMoveKindV1, ProjectileSpawnV1, SkillSlotV1, StableAiContext, StablePassive,
    StableSim, StatV1, UnitAttackV1,
};
use std::collections::BTreeMap;
pub const ID: &str = "tfm2_valorant_raze";
#[derive(Clone)]
struct ProjectileTrack {
    id: u64,
    caster: usize,
    pos: Point,
    tick: usize,
}
#[derive(Clone, Default)]
pub struct Raze {
    pub core: Core,
    profile: Option<crate::athlete_ai_core::Profile>,
    owner: usize,
    last_tick: Option<usize>,
    spawn: Option<Point>,
    carriers: BTreeMap<u64, usize>,
    failed: Vec<UtilityStatus>,
    seen: BTreeMap<usize, (usize, Point)>,
    kills_seen: usize,
    credits: Vec<CreditEvent>,
    credit_id: u64,
    zones: Vec<String>,
    guide: Option<String>,
    terrain: Vec<Rect>,
    perf: crate::adapter_perf::Profile,
    projectiles: Vec<ProjectileTrack>,
    next_projectile: u64,
    hud: Option<(i64, usize, bool)>,
}
fn has(sim: &StableSim, id: usize, name: &str) -> bool {
    sim.get_entity(id).is_some_and(|e| {
        (0..e.buff_count())
            .filter_map(|i| e.buff_at(i))
            .any(|b| b.name() == name)
    })
}
impl Raze {
    fn hud(&mut self, sim: &mut StableSim) {
        let shield = sim.get_entity(self.owner).is_some_and(|e| e.shield() > 0);
        let want = (self.core.credits, self.core.armor, shield);
        if self.hud == Some(want) {
            return;
        }
        let credits = self.core.credits.clamp(0, 9999);
        for (tag, digit) in [
            (
                "k",
                if credits >= 1000 {
                    Some(credits / 1000)
                } else {
                    None
                },
            ),
            ("h", Some((credits / 100) % 10)),
        ] {
            for d in 0..10 {
                let n = format!("omn_cr{tag}{d}");
                if digit == Some(d) {
                    if !has(sim, self.owner, &n) {
                        sim.add_buff(self.owner, &BuffV1::named(&n));
                    }
                } else {
                    sim.entity_remove_buff(self.owner, &n);
                }
            }
        }
        if !has(sim, self.owner, "omn_cash") {
            sim.add_buff(self.owner, &BuffV1::named("omn_cash"));
        }
        for k in 1..3 {
            let n = format!("omn_arm{k}");
            if self.core.armor == k && shield {
                if !has(sim, self.owner, &n) {
                    sim.add_buff(self.owner, &BuffV1::named(&n));
                }
            } else {
                sim.entity_remove_buff(self.owner, &n);
            }
        }
        self.hud = Some(want);
    }
    fn credit(&mut self, kind: u8) {
        self.credit_id += 1;
        self.credits.push(CreditEvent {
            id: self.credit_id,
            kind,
        });
    }
    fn clean(&mut self, sim: &mut StableSim) {
        for (_, id) in std::mem::take(&mut self.carriers) {
            if sim.get_entity(id).is_some_and(|e| e.is_alive()) {
                sim.entity_set_hp(id, 0);
            }
        }
        for n in self.zones.drain(..) {
            sim.entity_remove_buff(self.owner, &n);
        }
        if let Some(n) = self.guide.take() {
            sim.entity_remove_buff(self.owner, &n);
        }
        sim.entity_remove_buff(self.owner, "rz_air");
        self.seen.clear();
    }
    fn commands(
        &mut self,
        sim: &mut StableSim,
        commands: Vec<Cmd>,
        team: usize,
        tick: usize,
        all: &[crate::Champ],
    ) {
        for c in commands {
            match c {
                Cmd::Damage { target, raw, .. } => {
                    if sim
                        .get_entity(target)
                        .is_some_and(|e| e.is_alive() && e.is_targetable())
                    {
                        sim.deal_damage(
                            self.owner,
                            target,
                            0,
                            raw.max(0) as usize,
                            AttackTypeV1::Skill,
                        );
                    }
                }
                Cmd::Push { target, to } => {
                    if let Some(e) = sim.get_entity(target) {
                        let cc_immune = (0..e.buff_count())
                            .filter_map(|i| e.buff_at(i))
                            .any(|b| b.cc_immune);
                        let (x, y) = e.pos();
                        if !cc_immune {
                            crate::place::move_unit(
                                sim,
                                all,
                                tick,
                                target,
                                (x as i64, y as i64),
                                (to.x, to.y),
                            );
                        }
                    }
                }
                Cmd::MoveOwner { to } => {
                    if let Some(e) = sim.get_entity(self.owner) {
                        let (x, y) = e.pos();
                        crate::place::move_unit(
                            sim,
                            all,
                            tick,
                            self.owner,
                            (x as i64, y as i64),
                            (to.x, to.y),
                        );
                    }
                }
                Cmd::Guide { to } => {
                    if let Some(n) = self.guide.take() {
                        sim.entity_remove_buff(self.owner, &n);
                    }
                    let n = format!("rz_guide:{}:{}:{}", to.x, to.y, tick + 18);
                    sim.add_buff(self.owner, &crate::timed(&n, 18));
                    self.guide = Some(n);
                }
                Cmd::Spawn {
                    id,
                    bot,
                    pos,
                    hp,
                    ticks,
                } => {
                    let stat = StatV1 {
                        hp: hp.max(1) as usize,
                        defence: 15,
                        magic_resistance: 15,
                        move_speed: 1,
                        ..Default::default()
                    };
                    let attack = UnitAttackV1 {
                        attack: 0,
                        attack_ratio: 0,
                        range: 1,
                        cooltime: 5_184_000,
                        duration: 1,
                        start_timing: 0,
                        ..Default::default()
                    };
                    if let Some(host) = sim.spawn_unit(
                        "alchemist",
                        self.owner,
                        team,
                        pos.x.max(0) as u64,
                        pos.y.max(0) as u64,
                        ticks,
                        &stat,
                        &attack,
                    ) {
                        for k in [CcKindV1::Bind, CcKindV1::BlockAttack, CcKindV1::BlockSkill] {
                            sim.apply_cc(host, &CcV1::of_kind(k, ticks));
                        }
                        sim.add_buff(
                            host,
                            &crate::timed(
                                &format!("rz_owned:{}:{}", self.owner, id),
                                ticks as usize,
                            ),
                        );
                        self.carriers.insert(id, host);
                        crate::fx_unit(
                            sim,
                            if bot {
                                "tfm2_valorant_raze_bot"
                            } else {
                                "tfm2_valorant_raze_satchel"
                            },
                            self.owner,
                            host,
                            6,
                        );
                    } else {
                        self.failed.push(UtilityStatus { id, hp: -1 });
                    }
                }
                Cmd::MoveUnit { id, to } => {
                    if let Some(host) = self.carriers.get(&id).copied() {
                        sim.entity_set_pos(host, to.x.max(0) as u64, to.y.max(0) as u64);
                    }
                }
                Cmd::Remove { id } => {
                    if let Some(host) = self.carriers.remove(&id) {
                        if sim.get_entity(host).is_some_and(|e| e.is_alive()) {
                            sim.entity_set_hp(host, 0);
                        }
                    }
                }
                Cmd::Warning {
                    id,
                    tag,
                    pos,
                    radius,
                    until,
                } => {
                    if radius > 0 && tag != "pack_throw" {
                        let n = format!("rzw:{id}:{}:{}:{radius}:{until}", pos.x, pos.y);
                        sim.add_buff(
                            self.owner,
                            &crate::timed(&n, (until.saturating_sub(tick as u64) + 1) as usize),
                        );
                        self.zones.push(n);
                    }
                    crate::fx_point(
                        sim,
                        &format!("{ID}_{tag}"),
                        self.owner,
                        pos.x,
                        pos.y,
                        until.saturating_sub(tick as u64).max(1),
                    );
                }
                Cmd::Visual {
                    tag,
                    pos,
                    height,
                    until,
                    ..
                } => {
                    let name = if tag == "airborne" {
                        let facing = if self.core.flight.as_ref().is_some_and(|v| v.velocity.x < 0)
                        {
                            "l"
                        } else {
                            "r"
                        };
                        {
                            let variant = if self.core.armed_until > tick as u64 {
                                "_rocket"
                            } else if let Some(c) = self.core.cast.as_ref() {
                                match c.action {
                                    core::Action::Pack { .. } => "_pack",
                                    core::Action::Grenade { .. } | core::Action::Bot { .. } => {
                                        "_grenade"
                                    }
                                    _ => "_rocket",
                                }
                            } else {
                                ""
                            };
                            format!("{ID}_airborne{variant}_{facing}")
                        }
                    } else if tag == "rocket_flight" {
                        let v = self
                            .core
                            .rocket
                            .as_ref()
                            .map_or(Point::new(1, 0), |r| r.direction);
                        let d = if v.x.abs() > v.y.abs() * 2 {
                            if v.x >= 0 {
                                0
                            } else {
                                4
                            }
                        } else if v.y.abs() > v.x.abs() * 2 {
                            if v.y >= 0 {
                                2
                            } else {
                                6
                            }
                        } else {
                            match (v.x >= 0, v.y >= 0) {
                                (true, true) => 1,
                                (false, true) => 3,
                                (false, false) => 5,
                                _ => 7,
                            }
                        };
                        format!("{ID}_rocket_flight_{d}")
                    } else {
                        format!("{ID}_{tag}")
                    };
                    crate::fx_point(
                        sim,
                        &name,
                        self.owner,
                        pos.x,
                        pos.y - height,
                        until.saturating_sub(tick as u64).max(1),
                    );
                }
                Cmd::Gun { index } => {
                    for n in 0..6 {
                        sim.entity_remove_buff(self.owner, &format!("omn_gun{n}"));
                    }
                    sim.add_buff(self.owner, &Gun::from_index(index).buff());
                }
                Cmd::Armor { amount } => {
                    // The public API cannot remove only a purchased shield. Preserve allied shields.
                    let shield = sim.get_entity(self.owner).map_or(0, |e| e.shield()) as i64;
                    if amount > shield {
                        sim.entity_add_shield(self.owner, (amount - shield) as usize, 1_000_000);
                    }
                }
                Cmd::Buy { .. } => {
                    crate::fx_unit(sim, "tfm2_valorant_raze_buy", self.owner, self.owner, 30);
                }
                Cmd::Rejected { .. } => {}
            }
        }
    }
}
impl StablePassive for Raze {
    fn clone_box(&self) -> Box<dyn StablePassive> {
        Box::new(self.clone())
    }
    fn on_spawn(&mut self, sim: &mut StableSim, _: usize, owner: usize) {
        self.owner = owner;
        self.hud = None;
        self.spawn = sim.get_entity(owner).map(|e| {
            let (x, y) = e.pos();
            Point::new(x as i64, y as i64)
        });
        sim.add_buff(owner, &Gun::from_index(self.core.gun).buff());
    }
    fn on_dead(&mut self, sim: &mut StableSim, _: usize) {
        if self.core.was_alive {
            self.core.credits = (self.core.credits + crate::raze_data::DEATH_CREDITS)
                .min(crate::raze_data::MAX_CREDITS);
        }
        self.core.died();
        self.clean(sim);
    }
    fn on_kill(&mut self, sim: &mut StableSim, _: usize, owner: usize, victim: usize) {
        if let (Some(me), Some(v)) = (sim.get_entity(owner), sim.get_entity(victim)) {
            if v.is_champion() && v.team() != me.team() {
                self.credit(0);
            }
        }
    }
    fn on_assist(&mut self, _: &mut StableSim, _: usize, _: usize) {
        self.credit(1);
    }
    fn on_attack(
        &mut self,
        sim: &mut StableSim,
        _: usize,
        owner: usize,
        target: usize,
        damage: &mut usize,
    ) {
        // Same primary attack identity, gun modifiers and Judge falloff as Omen; no injected attacks.
        let Some(me) = sim.get_entity(owner) else {
            return;
        };
        let Some(t) = sim.get_entity(target) else {
            return;
        };
        let ((mx, my), (tx, ty)) = (me.pos(), t.pos());
        let team = me.team();
        let tag = if self.core.gun == Gun::Judge.idx() {
            "pellets"
        } else if self.core.gun == Gun::Operator.idx() {
            "tracer_op"
        } else {
            "tracer"
        };
        let spec = ProjectileSpawnV1 {
            caster_id: owner,
            team,
            x: mx,
            y: my.saturating_sub(3000),
            radius: 1000,
            speed: 14000,
            move_kind: ProjectileMoveKindV1::Linear.code(),
            target_x: tx,
            target_y: ty,
            penetrate: true,
            casting_target: CastingTargetV1::None.code(),
            ..Default::default()
        };
        sim.spawn_projectile(&format!("{ID}_{tag}"), "tfm2_custom_ai:noop", &spec);
        if self.core.gun == Gun::Operator.idx() {
            crate::fx_point(
                sim,
                &format!("{ID}_op_flash"),
                owner,
                mx as i64,
                my as i64 - 3000,
                12,
            );
        }
        if self.core.gun != Gun::Judge.idx() {
            return;
        }
        let d = core::distance(
            Point::new(mx as i64, my as i64),
            Point::new(tx as i64, ty as i64),
        );
        let k = if d <= 10000 {
            220
        } else if d >= 20000 {
            70
        } else {
            220 - (d - 10000) * 150 / 10000
        };
        let base = *damage;
        *damage = base * k as usize / 100;
        let all = crate::champions(sim);
        for e in all.iter().filter(|c| {
            c.team != team
                && c.id != target
                && crate::d2(c.x, c.y, tx as i64, ty as i64) <= crate::sq(9000)
        }) {
            sim.deal_damage(
                owner,
                e.id,
                base * k as usize * 35 / 10000,
                0,
                AttackTypeV1::Skill,
            );
            crate::wave_near(sim, owner, e.id, 9000, base * k as usize * 35 / 10000, 0);
        }
    }
    fn on_update(&mut self, sim: &mut StableSim, _: u64, _: usize, owner: usize) {
        let tick = sim.tick();
        if self.last_tick == Some(tick) {
            return;
        }
        self.last_tick = Some(tick);
        self.owner = owner;
        let Some(owner_entity) = sim.get_entity(owner) else {
            return;
        };
        if !owner_entity.is_alive() {
            return;
        }
        let team = owner_entity.team();
        let (x, y) = owner_entity.pos();
        let pos = Point::new(x as i64, y as i64);
        let shield = owner_entity.shield() as i64;
        let buffs: Vec<_> = (0..owner_entity.buff_count())
            .filter_map(|i| owner_entity.buff_at(i))
            .collect();
        let haste = buffs.iter().map(|b| b.skill_cooldown_mult as i64).sum();
        let ult_haste = buffs.iter().map(|b| b.ult_cooldown_mult as i64).sum();
        let unlocked = [SkillSlotV1::Skill, SkillSlotV1::Skill2, SkillSlotV1::Ult]
            .map(|s| owner_entity.cooldown(s).is_some_and(|c| c.unlocked));
        let cc: Vec<_> = (0..owner_entity.cc_count())
            .filter_map(|i| owner_entity.cc_at(i))
            .map(|c| c.kind)
            .collect();
        let stunned = [
            CcKindV1::Stun,
            CcKindV1::Airborne,
            CcKindV1::Fear,
            CcKindV1::Charm,
            CcKindV1::Taunt,
        ]
        .iter()
        .any(|k| cc.contains(&k.code()));
        let silenced = cc.contains(&CcKindV1::BlockSkill.code());
        let bound = [
            CcKindV1::Bind,
            CcKindV1::BlockMoveSkill,
            CcKindV1::ForceMove,
        ]
        .iter()
        .any(|k| cc.contains(&k.code()));
        self.perf.bind(sim, owner);
        let snapshot_at = self.perf.start();
        let profile = crate::athlete_ai::pinned_owner(sim, owner, &mut self.profile);
        let mut values = [50; 13];
        values[..9].copy_from_slice(&profile.values);
        values[9..].copy_from_slice(&profile.sense.0);
        let context = crate::farming::Context::new(sim, owner);
        let farm_window = context.calm();
        let mut units = vec![];
        for i in 0..sim.entity_count() {
            let Some(e) = sim.entity_at(i) else { continue };
            if !e.is_alive() {
                continue;
            }
            let visible = e.team() == team || sim.is_visible(team, e.id());
            let (x, y) = e.pos();
            let p = Point::new(x as i64, y as i64);
            let (hp, max_hp) = e.hp();
            let s = e.stat();
            let velocity = if visible {
                self.seen
                    .get(&e.id())
                    .filter(|(t, _)| *t + 1 == tick)
                    .map_or(Point::default(), |(_, prev)| p.sub(*prev).scale(60, 1))
            } else {
                Point::default()
            };
            let kind = if e.is_champion() {
                0
            } else {
                match e.kind() {
                    Some(EntityKindV1::Minion) => 1,
                    Some(EntityKindV1::Epic | EntityKindV1::Serpen) => 3,
                    Some(EntityKindV1::Tower | EntityKindV1::Nexus) => 4,
                    Some(EntityKindV1::Summon) => 5,
                    _ => 2,
                }
            };
            units.push(Unit {
                id: e.id(),
                team: e.team(),
                pos: p,
                velocity,
                hp: hp as i64,
                max_hp: max_hp as i64,
                ap: s.magic_power as i64,
                attack: s.attack as i64,
                visible,
                targetable: e.is_targetable(),
                kind,
                farmable: visible && context.eligible(e.id()),
                controlled: visible
                    && (0..e.cc_count()).filter_map(|i| e.cc_at(i)).any(|c| {
                        [CcKindV1::Stun, CcKindV1::Bind, CcKindV1::Airborne]
                            .iter()
                            .any(|k| k.code() == c.kind)
                    }),
                melee: crate::valorant_guns::close_fighter(&e.name().unwrap_or_default()),
            });
        }
        self.seen = units
            .iter()
            .filter(|u| u.visible)
            .map(|u| (u.id, (tick, u.pos)))
            .collect();
        let mut utility = std::mem::take(&mut self.failed);
        utility.extend(self.carriers.iter().map(|(id, host)| UtilityStatus {
            id: *id,
            hp: sim.get_entity(*host).map_or(0, |e| e.hp().0 as i64),
        }));
        let all = crate::champions_shared(sim);
        let mut blocks = self.terrain.clone();
        if self.terrain.is_empty() || tick % 60 == 0 {
            self.terrain.clear();
            for gy in 0..30 {
                for gx in 0..30 {
                    if crate::walls::wall_at(gx * 32000 + 16000, gy * 32000 + 16000) {
                        self.terrain.push(Rect {
                            min: Point::new(gx * 32000, gy * 32000),
                            max: Point::new((gx + 1) * 32000, (gy + 1) * 32000),
                        });
                    }
                }
            }
            blocks = self.terrain.clone();
        }
        for w in crate::steve::walls_up(&all, tick) {
            let (ex, ey) = w.built_end(tick);
            let a = Point::new(w.ax, w.ay);
            let b = Point::new(ex, ey);
            let n = (core::distance(a, b) / 4000).max(1);
            for k in 0..=n {
                let p = a.add(b.sub(a).scale(k, n));
                blocks.push(Rect {
                    min: p.sub(Point::new(crate::steve::WALL_HALF, crate::steve::WALL_HALF)),
                    max: p.add(Point::new(crate::steve::WALL_HALF, crate::steve::WALL_HALF)),
                });
            }
        }
        let mut hazards = vec![];
        if let Some(me) = all.iter().find(|c| c.id == owner) {
            let visible: Vec<_> = all
                .iter()
                .filter(|c| c.team == team || sim.is_visible(team, c.id))
                .cloned()
                .collect();
            let mut zones: Vec<_> = crate::gojo::danger_zones(&visible, me, tick)
                .into_iter()
                .chain(crate::gojo_v2::danger_zones(&visible, me, tick))
                .map(|z| (z.0, z.1, z.2, z.3))
                .collect();
            zones.extend(
                crate::magnus::danger_zones(&visible, me, tick)
                    .into_iter()
                    .map(|z| (z.0, z.1, z.2, tick + 12)),
            );
            zones.extend(
                crate::batch2::danger_zones(&visible, team)
                    .into_iter()
                    .map(|z| (z.0, z.1, z.2, tick + 12)),
            );
            zones.extend(
                danger_zones(&visible, team, tick)
                    .into_iter()
                    .map(|z| (z.0, z.1, z.2, tick + 12)),
            );
            zones.extend(
                crate::steve_craft::danger_zones(&visible, me, tick)
                    .into_iter()
                    .map(|z| (z.0, z.1, z.2, z.3)),
            );
            zones.extend(
                crate::vesper::rifts_up(&visible, tick)
                    .into_iter()
                    .filter(|(_, t)| *t != team)
                    .map(|(z, _)| (z.x, z.y, z.r, z.until)),
            );
            for (x, y, r, end) in zones {
                hazards.push(Hazard {
                    id: ((x as u64) << 20) ^ (y as u64) ^ r as u64,
                    pos: Point::new(x, y),
                    radius: r,
                    until: end as u64,
                    severity: 70,
                });
            }
        }
        let assignment = crate::universal_ai::assignment(sim, owner).filter(|a| a.4 > 0);
        let objective = assignment.as_ref().and_then(|a| {
            sim.get_entity(a.2)
                .filter(|e| matches!(e.kind(), Some(EntityKindV1::Epic | EntityKindV1::Serpen)))
                .map(|e| e.id())
        });
        let buy_allowed = self.spawn.is_some_and(|s| core::distance(s, pos) <= 45000)
            || !units.iter().any(|u| {
                u.team != team && u.champion() && u.visible && core::distance(u.pos, pos) <= 110000
            });
        let n = sim.kill_log_count();
        if self.kills_seen > n {
            self.kills_seen = n;
        }
        while self.kills_seen < n {
            if sim
                .kill_log_at(self.kills_seen)
                .is_some_and(|k| k.killer_team == team)
            {
                self.credit(2);
            }
            self.kills_seen += 1;
        }
        // The SDK exposes projectile positions but no point-visibility or projectile identity query.
        // Observe only nearby shots from a visible caster, along a clear, unsmoked line.
        let smokes = crate::valorant::smokes_up(&all, tick);
        let mut tracks = vec![];
        let mut used = std::collections::BTreeSet::new();
        for i in 0..sim.projectile_count().min(128) {
            let Some(p) = sim.projectile_at(i) else {
                continue;
            };
            let at = Point::new(p.x as i64, p.y as i64);
            if p.is_end
                || p.team == team
                || core::distance(at, pos) > 100000
                || !sim.is_visible(team, p.caster_id)
                || crate::walls::clip(pos.x, pos.y, at.x, at.y) != (at.x, at.y)
            {
                continue;
            }
            if smokes.iter().any(|s| {
                crate::place::seg_dist2((s.x, s.y), (pos.x, pos.y), (at.x, at.y))
                    <= crate::sq(crate::valorant::SMOKE_R)
                    && core::distance(at, pos) > 15000
            }) {
                continue;
            }
            let old = self
                .projectiles
                .iter()
                .filter(|t| {
                    t.caster == p.caster_id
                        && t.tick + 1 == tick
                        && !used.contains(&t.id)
                        && core::distance(t.pos, at) < 24000
                })
                .min_by_key(|t| (core::distance(t.pos, at), t.id));
            let (id, velocity) = if let Some(t) = old {
                used.insert(t.id);
                (t.id, at.sub(t.pos))
            } else {
                self.next_projectile += 1;
                (1000000000 + self.next_projectile, Point::default())
            };
            tracks.push(ProjectileTrack {
                id,
                caster: p.caster_id,
                pos: at,
                tick,
            });
            hazards.push(Hazard {
                id,
                pos: at.add(velocity.scale(6, 1)),
                radius: 9000,
                until: (tick + 8) as u64,
                severity: 55,
            });
            if tracks.len() >= 32 {
                break;
            }
        }
        self.projectiles = tracks;
        let missed_wave = if assignment
            .as_ref()
            .is_some_and(|a| core::distance(pos, Point::new(a.3 .0, a.3 .1)) > 120000)
        {
            units
                .iter()
                .filter(|u| {
                    u.kind == 1 && u.farmable && u.visible && core::distance(u.pos, pos) < 90000
                })
                .map(|u| u.hp * 15 / u.max_hp.max(1))
                .sum::<i64>()
                .min(100)
        } else {
            0
        };
        let frame = Frame {
            version: 1,
            tick: tick as u64,
            owner,
            seed: sim.seed(),
            automatic: true,
            profile: Profile {
                values,
                known: profile.known,
            },
            units,
            blocks,
            hazards,
            utility,
            credits: std::mem::take(&mut self.credits),
            unlocked,
            stunned,
            silenced,
            bound,
            farm_window,
            buy_allowed,
            at_spawn: self.spawn.is_some_and(|s| core::distance(s, pos) <= 45000),
            shield,
            assignment: assignment.map(|a| Point::new(a.3 .0, a.3 .1)),
            objective,
            missed_wave,
            intent: None,
            action: None,
            practice_inventory: false,
            haste,
            ult_haste,
        };
        self.perf.finish("snapshot", snapshot_at);
        let core_at = self.perf.start();
        let before = (
            self.core.metrics.casts,
            self.core.reaction_events,
            self.core.decision,
        );
        let commands = self.core.step(&frame);
        self.perf.finish("core", core_at);
        let host_at = self.perf.start();
        self.commands(sim, commands, team, tick, &all);
        if self.core.metrics.casts > before.0 {
            crate::athlete_ai::participate(sim, owner, 2);
        }
        if self.core.reaction_events > before.1 {
            crate::athlete_ai::participate(sim, owner, 4);
        }
        if self.core.decision > before.2 {
            crate::athlete_ai::participate(sim, owner, 8);
        }
        if let Some(c) = self.core.cast.as_ref() {
            let ticks = c.release.saturating_sub(tick as u64).max(1);
            sim.apply_cc(owner, &CcV1::of_kind(CcKindV1::BlockAttack, 2));
            let mut pose = CcV1::of_kind(CcKindV1::Animation, ticks);
            pose.set_name(match c.action {
                core::Action::Pack { .. } => "skill2",
                core::Action::EquipRocket => "ult",
                _ => "skill1",
            });
            sim.apply_cc(owner, &pose);
        }
        if self.core.flight.is_some() {
            let mut b = crate::timed("rz_air", 2);
            b.move_speed_mult = -10000;
            sim.add_buff(owner, &b);
            let mut pose = CcV1::of_kind(CcKindV1::Animation, 2);
            pose.set_name("air_hidden");
            sim.apply_cc(owner, &pose);
        } else {
            sim.entity_remove_buff(owner, "rz_air");
        }
        if tick % 6 == 0 {
            for (id, host) in &self.carriers {
                if sim.get_entity(*host).is_some_and(|e| e.is_alive()) {
                    let bot = self.core.bot.as_ref().is_some_and(|b| b.id == *id);
                    crate::fx_unit(
                        sim,
                        if bot {
                            "tfm2_valorant_raze_bot"
                        } else {
                            "tfm2_valorant_raze_satchel"
                        },
                        owner,
                        *host,
                        6,
                    );
                }
            }
        }
        self.zones.retain(|n| {
            let live = n
                .rsplit(':')
                .next()
                .and_then(|v| v.parse::<usize>().ok())
                .is_some_and(|end| end > tick);
            if !live {
                sim.entity_remove_buff(owner, n);
            }
            live
        });
        self.hud(sim);
        self.perf.finish("host", host_at);
    }
}
pub fn airborne(ctx: &mut StableAiContext) -> bool {
    let player = ctx.player_id();
    ctx.sim().is_some_and(|sim| {
        sim.get_player(player)
            .and_then(|p| p.champion())
            .is_some_and(|e| has(&sim, e.id(), "rz_air"))
    })
}
pub fn guide_input(ctx: &mut StableAiContext, input: &InputV1) -> Option<InputV1> {
    let player = ctx.player_id();
    let dest = {
        let sim = ctx.sim()?;
        let me = sim.get_player(player)?.champion()?;
        if has(&sim, me.id(), "rz_air") {
            return None;
        }
        let mut dest = None;
        for i in 0..me.buff_count() {
            let Some(b) = me.buff_at(i) else { continue };
            if let Some(raw) = b.name().strip_prefix("rz_guide:") {
                let a: Vec<_> = raw
                    .split(':')
                    .filter_map(|x| x.parse::<u64>().ok())
                    .collect();
                if a.len() == 3 && a[2] > sim.tick() as u64 {
                    dest = Some((a[0], a[1]));
                    break;
                }
            }
        }
        dest
    }?;
    let mut out = *input;
    out.x = dest.0;
    out.y = dest.1;
    ctx.is_valid_input(&out).then_some(out)
}
pub fn danger_zones(all: &[crate::Champ], team: usize, tick: usize) -> Vec<(i64, i64, i64)> {
    let mut out = vec![];
    for c in all.iter().filter(|c| c.team != team) {
        for b in &c.buffs {
            if let Some(raw) = b.name().strip_prefix("rzw:") {
                let a: Vec<_> = raw
                    .split(':')
                    .filter_map(|x| x.parse::<i64>().ok())
                    .collect();
                if a.len() == 5 && a[4] > tick as i64 {
                    out.push((a[1], a[2], a[3]));
                }
            }
        }
    }
    out
}
pub fn avoid(
    all: &[crate::Champ],
    me: &crate::Champ,
    dest: (i64, i64),
    tick: usize,
) -> Option<(i64, i64)> {
    let zones = danger_zones(all, me.team, tick);
    let bad = |p: (i64, i64)| {
        zones
            .iter()
            .any(|z| crate::d2(p.0, p.1, z.0, z.1) <= crate::sq(z.2 + 6000))
    };
    if !bad((me.x, me.y)) && !bad(dest) {
        return None;
    }
    let mut candidates = vec![];
    for (x, y, r) in &zones {
        let from = Point::new(me.x, me.y);
        let delta = from.sub(Point::new(*x, *y));
        let delta = if delta.length() == 0 {
            Point::new(1000, 0)
        } else {
            delta
        };
        for d in [
            delta,
            Point::new(-delta.y, delta.x),
            Point::new(delta.y, -delta.x),
        ] {
            let p = Point::new(*x, *y)
                .add(d.scale(r + 12000, d.length().max(1)))
                .bounded();
            if !bad((p.x, p.y)) && !crate::walls::wall_at(p.x, p.y) {
                candidates.push((p.x, p.y));
            }
        }
    }
    candidates
        .into_iter()
        .min_by_key(|p| crate::d2(p.0, p.1, me.x, me.y))
}
