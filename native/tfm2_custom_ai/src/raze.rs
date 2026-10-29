//! Thin native Raze adapter. Engine owns gun attacks; core owns utility and movement.
use crate::raze_core::{
    self as core, Cmd, Core, CreditEvent, Defender, Frame, Hazard, Point, Profile, Rect, Surface,
    Unit, UtilityStatus,
};
use crate::valorant_guns::Gun;
use mod_api_stable::{
    AttackTypeV1, BuffV1, CastingTargetV1, CcKindV1, CcV1, EntityKindV1, InputV1,
    ProjectileMoveKindV1, ProjectileSpawnV1, SkillSlotV1, StableAiContext, StablePassive,
    StableSim, StatV1, UnitAttackV1,
};
use std::collections::{BTreeMap, HashMap};
use std::sync::Mutex;
#[cfg(test)]
#[path = "raze_bridge_tests.rs"]
mod bridge_tests;
pub const ID: &str = "tfm2_valorant_raze";
#[derive(Clone, Debug, serde::Serialize)]
pub struct DamageReceipt {
    pub tick: usize,
    pub id: u64,
    pub target: usize,
    pub source: String,
    pub requested: i64,
    pub submitted: bool,
    pub hp_before: usize,
    pub hp_after: usize,
    pub shield_before: usize,
    pub shield_after: usize,
}
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
    receipts: Vec<DamageReceipt>,
    receipt_lines: usize,
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
    enemy_guides: BTreeMap<(usize, u64), String>,
    guide: Option<String>,
    terrain: Vec<Rect>,
    perf: crate::adapter_perf::Profile,
    projectiles: Vec<ProjectileTrack>,
    next_projectile: u64,
    hud: Option<(i64, usize, bool)>,
    /// Round 143: (object id, family) -> (first tick, last tick) of a visual renewed every tick, which plays one
    /// baked single-frame phase per tick instead of stacking 24-tick copies.
    phases: BTreeMap<(u64, &'static str), (u64, u64)>,
    /// The tick the armed-rocket marker was last played (it lasts 24 ticks; renewed while armed).
    ready_at: u64,
    /// The carrier id that is the Boom Bot (the rest are satchels).
    bot_carrier: Option<u64>,
    /// Round 144: the athlete playing her and their mastery rank (0 Iron to 7 Radiant), latched at spawn.
    athlete: Option<usize>,
    rank: Option<usize>,
    sig: Option<String>,
    record: crate::mastery::Record,
    /// The unlock state and level of the latest frame, for rejection diagnostics.
    ult_unlocked: bool,
    level: i64,
    /// Round 144: the HUD buffs last shown (Hype bar, Carnaval aura, air shield).
    kit_hud: Vec<String>,
}
/// Round 144: Raze's mastery book (raze_memory.txt), as Magnus's: points per athlete from the games they played.
pub static BOOK: crate::mastery::Book = crate::mastery::Book::new("raze");
/// The Valorant rank names of her mastery.
pub const MASTERY: [&str; 8] = [
    "Iron", "Bronze", "Silver", "Gold", "Platinum", "Diamond", "Immortal", "Radiant",
];
/// Round 144: a view effect that fails to play is logged once per game: a career keeps its own copy of the view
/// list, so a name added since it was made is missing until the career is migrated.
fn fx_point(sim: &mut StableSim, name: &str, caster: usize, x: i64, y: i64, time: u64) -> bool {
    let ok = crate::fx_point(sim, name, caster, x, y, time);
    if !ok {
        missing_view(sim, name);
    }
    ok
}
fn fx_unit(sim: &mut StableSim, name: &str, caster: usize, target: usize, time: u64) -> bool {
    let ok = crate::fx_unit(sim, name, caster, target, time);
    if !ok {
        missing_view(sim, name);
    }
    ok
}
/// Round 144: the satchel combo's stun and Encore's knock-up, never on crowd-control-immune targets.
fn control(sim: &mut StableSim, target: usize, kind: CcKindV1, ticks: u64) {
    let immune = sim.get_entity(target).is_none_or(|e| {
        !e.is_alive()
            || (0..e.buff_count())
                .filter_map(|i| e.buff_at(i))
                .any(|b| b.cc_immune)
    });
    if !immune {
        sim.apply_cc(target, &CcV1::of_kind(kind, ticks));
    }
}
fn missing_view(sim: &StableSim, name: &str) {
    crate::mod_log(
        sim,
        "raze_log.txt",
        &format!("fx:{name}"),
        &format!(
            "view effect {name} did not play: run tools/migrate-champion-reworks.py on the career so its Raze view list is current"
        ),
    );
}
/// (match key, owner) -> the game's move orders by tick (round 143, as `levi::note_dest`): her takeoff gate and
/// team guidance read where the engine is walking her. Keyed by `crate::match_key`, so the precomputed and the
/// live simulation never read each other's orders.
type Walks = HashMap<(u64, usize), BTreeMap<usize, (i64, i64)>>;
static WALKS: Mutex<Option<Walks>> = Mutex::new(None);
/// A move order older than this no longer says where she is being walked.
const WALK_STALE: usize = 120;
pub fn note_walk(key: u64, owner: usize, x: i64, y: i64, tick: usize) {
    if let Ok(mut g) = WALKS.lock() {
        let m = g.get_or_insert_with(HashMap::new);
        if m.len() > 64 {
            m.clear();
        }
        let orders = m.entry((key, owner)).or_default();
        orders.insert(tick, (x, y));
        while orders.len() > 256 {
            orders.pop_first();
        }
    }
}
/// The latest fresh move order given before `tick`.
fn walk_of(key: u64, owner: usize, tick: usize) -> Option<(i64, i64)> {
    let g = WALKS.lock().ok()?;
    let (&t, &p) = g.as_ref()?.get(&(key, owner))?.range(..tick).next_back()?;
    (tick <= t + WALK_STALE).then_some(p)
}
fn air_guard() -> BuffV1 {
    let mut b = BuffV1::named("rz_air_guard");
    b.damaged_reduce = crate::raze_data::AIR_REDUCTION as usize;
    b
}
#[cfg(test)]
mod air_guard_tests {
    #[test]
    fn air_guard_uses_engine_reduction_without_immunity_or_healing() {
        let b = super::air_guard();
        assert_eq!(b.damaged_reduce, 15);
        assert!(!b.cc_immune);
        assert!(!b.undying);
        assert_eq!(b.defence, 0);
        assert_eq!(b.magic_resistance, 0);
    }
}
fn has(sim: &StableSim, id: usize, name: &str) -> bool {
    sim.get_entity(id).is_some_and(|e| {
        (0..e.buff_count())
            .filter_map(|i| e.buff_at(i))
            .any(|b| b.name() == name)
    })
}
fn surface_key(parts: &[i64]) -> u64 {
    let mut h = 1469598103934665603u64;
    for n in parts {
        for b in n.to_le_bytes() {
            h = (h ^ b as u64).wrapping_mul(1099511628211);
        }
    }
    (h & ((1 << 39) - 1)).max(1)
}
fn wall_surfaces(
    w: &crate::steve::Wall,
    tick: usize,
    identity: Option<(usize, i64, i64)>,
) -> Vec<Surface> {
    let a = Point::new(w.ax, w.ay);
    let full = Point::new(w.bx, w.by);
    let end = w.built_end(tick);
    let built = core::distance(a, Point::new(end.0, end.1));
    let length = core::distance(a, full);
    let steps = (length / 4000).max(1);
    (0..=steps)
        .filter(|k| length * k / steps <= built)
        .map(|k| {
            let point = a.add(full.sub(a).scale(k, steps));
            let id = identity.map_or_else(
                || surface_key(&[w.ax, w.ay, w.bx, w.by, w.t0 as i64, w.tend as i64, k]),
                |(owner, object, segment)| surface_key(&[owner as i64, object, segment, k]),
            );
            Surface {
                id,
                bounds: Rect {
                    min: point.sub(Point::new(crate::steve::WALL_HALF, crate::steve::WALL_HALF)),
                    max: point.add(Point::new(crate::steve::WALL_HALF, crate::steve::WALL_HALF)),
                },
            }
        })
        .collect()
}
fn pack_tag(p: &core::Pack) -> &'static str {
    match (p.attachment.normal.x, p.attachment.normal.y) {
        (0, -1) => "satchel_north",
        (1, 0) => "satchel_east",
        (0, 1) => "satchel_south",
        (-1, 0) => "satchel_west",
        _ => "satchel_ground",
    }
}
impl Raze {
    /// One of the eight baked phases (3 ticks each) for a visual renewed every tick.
    fn phase(&mut self, id: u64, family: &'static str, tick: u64) -> u64 {
        let e = self.phases.entry((id, family)).or_insert((tick, tick));
        if e.1 + 1 < tick {
            *e = (tick, tick);
        }
        e.1 = tick;
        (tick - e.0) / 3 % 8
    }
    /// Satchels and the Boom Bot: one baked phase per tick, kept across a bot turning (round 143; was a 24-tick
    /// copy every 6 ticks, so four stacked and an old facing lingered after a turn).
    fn draw_carriers(&mut self, sim: &mut StableSim, tick: u64) {
        let carriers: Vec<_> = self
            .carriers
            .iter()
            .map(|(id, host)| (*id, *host))
            .collect();
        for (id, host) in carriers {
            if !sim.get_entity(host).is_some_and(|e| e.is_alive()) {
                continue;
            }
            let tag = if self.bot_carrier == Some(id) {
                let direction = self
                    .core
                    .bot
                    .as_ref()
                    .map_or(Point::new(1, 0), |b| b.direction);
                format!("bot_round_{}", core::facing(direction))
            } else {
                self.core
                    .packs
                    .iter()
                    .find(|p| p.id == id)
                    .map_or("satchel_ground", pack_tag)
                    .into()
            };
            let k = self.phase(id, "carrier", tick);
            fx_unit(sim, &format!("{ID}_{tag}_p{k}"), self.owner, host, 1);
        }
    }
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
    /// Round 144: the Hype bar over her head (five steps), the Carnaval aura (with its move speed) and the air
    /// shield, as view buffs re-added only when they change.
    fn kit_hud(&mut self, sim: &mut StableSim, tick: u64) {
        use crate::raze_data::{CARNAVAL_SPEED, HYPE_MAX};
        let mut want = vec![];
        let step = (self.core.hype * 5 + HYPE_MAX - 1) / HYPE_MAX;
        if step > 0 && !self.core.carnaval(tick) {
            want.push(format!("rz_hype{}", step.min(5)));
        }
        if self.core.carnaval(tick) {
            want.push("rz_carnaval".to_string());
        }
        if self.core.air_shield.is_some() {
            want.push("rz_air_shield".to_string());
        }
        if want == self.kit_hud {
            return;
        }
        for n in std::mem::take(&mut self.kit_hud) {
            if !want.contains(&n) {
                sim.entity_remove_buff(self.owner, &n);
            }
        }
        for n in &want {
            if !has(sim, self.owner, n) {
                let mut b = BuffV1::named(n);
                if n == "rz_carnaval" {
                    b = crate::timed(n, self.core.carnaval_until.saturating_sub(tick).max(1) as usize);
                    b.move_speed_mult = CARNAVAL_SPEED as i32;
                }
                sim.add_buff(self.owner, &b);
            }
        }
        self.kit_hud = want;
    }
    fn credit(&mut self, kind: u8, victim: usize) {
        self.credit_id += 1;
        self.credits.push(CreditEvent {
            id: self.credit_id,
            kind,
            victim,
        });
    }
    fn clean(&mut self, sim: &mut StableSim) {
        for ((target, _), name) in std::mem::take(&mut self.enemy_guides) {
            sim.entity_remove_buff(target, &name);
        }
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
        sim.entity_remove_buff(self.owner, "rz_slide");
        sim.entity_remove_buff(self.owner, "rz_air_guard");
        for n in self.kit_hud.drain(..) {
            sim.entity_remove_buff(self.owner, &n);
        }
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
            // A self-blast can kill the owner midway through this ordered batch.
            // Only cleanup may continue; never spawn or reapply protection to a corpse.
            if !sim.get_entity(self.owner).is_some_and(|e| e.is_alive())
                && !matches!(
                    &c,
                    Cmd::Remove { .. }
                        | Cmd::ClearWarning { .. }
                        | Cmd::AirProtection { active: false }
                )
            {
                continue;
            }
            match c {
                Cmd::AirProtection { active } => {
                    if active {
                        sim.add_buff(self.owner, &air_guard());
                    } else {
                        sim.entity_remove_buff(self.owner, "rz_air_guard");
                    }
                }
                Cmd::Damage {
                    id,
                    target,
                    raw,
                    source,
                    vamp,
                    fixed,
                    self_hit,
                } => {
                    let before = sim
                        .get_entity(target)
                        .map(|e| (e.hp().0, e.shield(), e.is_alive() && e.is_targetable()));
                    if before.is_some_and(|v| v.2) {
                        if fixed {
                            // Round 144: Combustion is true damage
                            sim.deal_damage_typed(
                                self.owner,
                                target,
                                raw.max(0) as usize,
                                mod_api_stable::DamageTypeV1::Fixed,
                                AttackTypeV1::Skill,
                            );
                        } else {
                            sim.deal_damage(
                                self.owner,
                                target,
                                raw.max(0) as usize,
                                0,
                                AttackTypeV1::Skill,
                            );
                        }
                    }
                    let after = sim
                        .get_entity(target)
                        .map_or((0, 0), |e| (e.hp().0, e.shield()));
                    // Round 144: omnivamp heals a share of what the hit actually took (HP and shield).
                    if let Some(b) = before.filter(|_| vamp > 0 && !self_hit) {
                        let dealt = (b.0 + b.1).saturating_sub(after.0 + after.1);
                        if dealt > 0 {
                            sim.heal(self.owner, self.owner, dealt * vamp as usize / 100);
                        }
                    }
                    let receipt = DamageReceipt {
                        tick,
                        id,
                        target,
                        source,
                        requested: raw,
                        submitted: before.is_some_and(|v| v.2),
                        hp_before: before.map_or(0, |v| v.0),
                        hp_after: after.0,
                        shield_before: before.map_or(0, |v| v.1),
                        shield_after: after.1,
                    };
                    if crate::farming::diagnostics() && self.receipt_lines < 512 {
                        crate::magnus::BOOK.log(&format!(
                            "raze observed-damage {}",
                            serde_json::to_string(&receipt).unwrap_or_default()
                        ));
                        self.receipt_lines += 1;
                    }
                    if self.receipts.len() == 64 {
                        self.receipts.remove(0);
                    }
                    self.receipts.push(receipt);
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
                Cmd::Knockback { target, to, ticks } => {
                    if let Some(e) = sim.get_entity(target).filter(|e| e.is_alive()) {
                        let immune = (0..e.buff_count())
                            .filter_map(|i| e.buff_at(i))
                            .any(|b| b.cc_immune);
                        if !immune
                            && !matches!(
                                e.kind(),
                                Some(
                                    EntityKindV1::Epic
                                        | EntityKindV1::Serpen
                                        | EntityKindV1::Tower
                                        | EntityKindV1::Nexus
                                )
                            )
                        {
                            let p = e.pos();
                            let from = Point::new(p.0 as i64, p.1 as i64);
                            let end = crate::walls::clip(from.x, from.y, to.x, to.y);
                            let d = Point::new(end.0, end.1).sub(from);
                            let length = d.length();
                            if length > 0 {
                                let mut cc = CcV1::of_kind(CcKindV1::ForceMove, ticks);
                                cc.dx = d.x * 1000 / length;
                                cc.dy = d.y * 1000 / length;
                                cc.speed = (length / ticks.max(1) as i64) as u64;
                                sim.apply_cc(target, &cc);
                            }
                        }
                    }
                }
                Cmd::EnemyGuide {
                    target,
                    rocket,
                    to,
                    until,
                } => {
                    let n = format!("rze:{}:{rocket}:{}:{}:{until}", self.owner, to.x, to.y);
                    if self.enemy_guides.get(&(target, rocket)) != Some(&n) {
                        if let Some(old) = self.enemy_guides.insert((target, rocket), n.clone()) {
                            sim.entity_remove_buff(target, &old);
                        }
                        sim.add_buff(
                            target,
                            &crate::timed(&n, until.saturating_sub(tick as u64).max(1) as usize),
                        );
                    }
                }
                Cmd::ClearWarning { id } => {
                    for prefix in [format!("rzw:{id}:"), format!("rzrw:{id}:")] {
                        self.zones.retain(|n| {
                            if n.starts_with(&prefix) {
                                sim.entity_remove_buff(self.owner, n);
                                false
                            } else {
                                true
                            }
                        });
                    }
                    self.enemy_guides.retain(|(target, rocket), name| {
                        if *rocket == id {
                            sim.entity_remove_buff(*target, name);
                            false
                        } else {
                            true
                        }
                    });
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
                    // Round 143: re-issued every tick; a buff outlives the decision interval and is renewed only
                    // when the point moves or half its life has gone, so the guide never lapses mid-plan.
                    let prefix = format!("rz_guide:{}:{}:", to.x, to.y);
                    let fresh = self.guide.as_ref().is_some_and(|n| {
                        n.strip_prefix(&prefix)
                            .and_then(|t| t.parse::<usize>().ok())
                            .is_some_and(|end| end >= tick + GUIDE_LIFE / 2)
                    });
                    if !fresh {
                        if let Some(n) = self.guide.take() {
                            sim.entity_remove_buff(self.owner, &n);
                        }
                        let n = format!("{prefix}{}", tick + GUIDE_LIFE);
                        sim.add_buff(self.owner, &crate::timed(&n, GUIDE_LIFE));
                        self.guide = Some(n);
                    }
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
                        let mut pose = CcV1::of_kind(CcKindV1::Animation, ticks);
                        pose.set_name(if bot {
                            "rz_bot_hidden"
                        } else {
                            "rz_satchel_hidden"
                        });
                        sim.apply_cc(host, &pose);
                        sim.add_buff(
                            host,
                            &crate::timed(
                                &format!("rz_owned:{}:{}", self.owner, id),
                                ticks as usize,
                            ),
                        );
                        // drawn every tick by draw_carriers with a one-tick phase (round 143)
                        self.carriers.insert(id, host);
                        if bot {
                            self.bot_carrier = Some(id);
                        }
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
                    let rocket = tag == "rocket_impact_warning";
                    if radius > 0 {
                        crate::steve::note_map_buff(sim.seed());
                    }
                    if radius > 0 && tag != "pack_throw" {
                        let prefix = format!("{}:{id}:", if rocket { "rzrw" } else { "rzw" });
                        let n = format!("{prefix}{}:{}:{radius}:{until}", pos.x, pos.y);
                        self.zones.retain(|old| {
                            if old.starts_with(&prefix) && old != &n {
                                sim.entity_remove_buff(self.owner, old);
                                false
                            } else {
                                true
                            }
                        });
                        if !self.zones.contains(&n) {
                            sim.add_buff(
                                self.owner,
                                &crate::timed(&n, (until.saturating_sub(tick as u64) + 1) as usize),
                            );
                            self.zones.push(n);
                        }
                    }
                    let phase = self.core.rocket.as_ref().map_or(0, |r| {
                        ((tick as u64 - r.born) * 8 / (r.impact_tick - r.born).max(1)).min(7)
                    });
                    let visual = if rocket {
                        format!("{ID}_{tag}_{phase}")
                    } else if tag == "grenade_warning" {
                        // renewed every tick: one baked phase per tick (round 143)
                        let k = self.phase(id, "grenade_warning", tick as u64);
                        format!("{ID}_{tag}_p{k}")
                    } else {
                        format!("{ID}_{tag}")
                    };
                    fx_point(
                        sim,
                        &visual,
                        self.owner,
                        pos.x,
                        pos.y,
                        if rocket || tag == "grenade_warning" {
                            1
                        } else {
                            until.saturating_sub(tick as u64).max(1)
                        },
                    );
                }
                Cmd::Visual {
                    id,
                    tag,
                    pos,
                    height,
                    until,
                } => {
                    let name = if tag == "airborne" {
                        let facing = if self.core.flight.as_ref().is_some_and(|v| v.velocity.x < 0)
                        {
                            "l"
                        } else {
                            "r"
                        };
                        // Round 143, as the Lab: grenade, then pack, then an armed rocket, else plain (a Boom Bot
                        // throw keeps the plain airborne pose).
                        let variant = match self.core.cast.as_ref().map(|c| &c.action) {
                            Some(core::Action::Grenade { .. }) => "_grenade",
                            Some(core::Action::Pack { .. } | core::Action::DropPack) => "_pack",
                            Some(core::Action::EquipRocket) => "_rocket",
                            _ if self.core.armed_until > tick as u64 => "_rocket",
                            _ => "",
                        };
                        format!("{ID}_airborne{variant}_{facing}")
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
                        let k = self.phase(id, "rocket_flight", tick as u64);
                        format!("{ID}_rocket_flight_big_{d}_p{k}")
                    } else if tag == "grenade_flight" {
                        let k = self.phase(id, "grenade_flight", tick as u64);
                        format!("{ID}_grenade_flight_p{k}")
                    } else {
                        if tag == "rocket_ready" {
                            // Round 144: the armed marker is drawn 1.5x, as the rocket in flight
                            self.ready_at = tick as u64;
                            format!("{ID}_rocket_ready_big")
                        } else {
                            format!("{ID}_{tag}")
                        }
                    };
                    // Renewed every tick: exactly one tick (the game plays a whole animation regardless).
                    let per_tick = matches!(
                        tag.as_str(),
                        "airborne" | "rocket_flight" | "grenade_flight"
                    );
                    fx_point(
                        sim,
                        &name,
                        self.owner,
                        pos.x,
                        pos.y - height,
                        if per_tick {
                            1
                        } else {
                            until.saturating_sub(tick as u64).max(1)
                        },
                    );
                }
                Cmd::Gun { index } => {
                    for n in 0..7 {
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
                    fx_unit(sim, "tfm2_valorant_raze_buy", self.owner, self.owner, 30);
                }
                Cmd::Shield { amount, ticks } => {
                    sim.entity_add_shield(self.owner, amount.max(0) as usize, ticks as usize);
                }
                Cmd::Slow {
                    target,
                    percent,
                    ticks,
                } => {
                    // Graffiti: one slow, renewed (never stacked) while they stand in the paint
                    sim.entity_remove_buff(target, "rz_paint");
                    let mut b = crate::timed("rz_paint", ticks as usize);
                    b.move_speed_mult = -(percent as i32);
                    sim.add_buff(target, &b);
                }
                Cmd::Stun { target, ticks } => control(sim, target, CcKindV1::Stun, ticks),
                Cmd::KnockUp { target, ticks } => control(sim, target, CcKindV1::Airborne, ticks),
                Cmd::Reveal { target, ticks } => {
                    sim.entity_remove_buff(target, "rz_revealed");
                    sim.add_buff(target, &crate::timed("rz_revealed", ticks as usize));
                }
                Cmd::Tracer { to } => {
                    if let Some(e) = sim.get_entity(self.owner) {
                        let (x, y) = e.pos();
                        let spec = ProjectileSpawnV1 {
                            caster_id: self.owner,
                            team,
                            x,
                            y: y.saturating_sub(3000),
                            radius: 1000,
                            speed: 14000,
                            move_kind: ProjectileMoveKindV1::Linear.code(),
                            target_x: to.x.max(0) as u64,
                            target_y: to.y.max(0) as u64,
                            penetrate: true,
                            casting_target: CastingTargetV1::None.code(),
                            ..Default::default()
                        };
                        sim.spawn_projectile(&format!("{ID}_tracer"), "tfm2_custom_ai:noop", &spec);
                    }
                }
                Cmd::Rejected { reason } => {
                    // Round 144: why a skill (above all the rocket) was refused, with the unlock state and level
                    if crate::farming::diagnostics() {
                        crate::mod_log(
                            sim,
                            "raze_log.txt",
                            &format!("reject:{reason}:{}", self.level),
                            &format!(
                                "rejected: {reason} (ultimate unlocked {}, level {})",
                                self.ult_unlocked, self.level
                            ),
                        );
                    }
                }
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
                self.credit(0, victim);
            }
        }
    }
    fn on_assist(&mut self, _: &mut StableSim, _: usize, _: usize) {
        self.credit(1, 0);
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
        let tag = if Gun::from_index(self.core.gun).shotgun() {
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
            fx_point(
                sim,
                &format!("{ID}_op_flash"),
                owner,
                mx as i64,
                my as i64 - 3000,
                12,
            );
        }
        if !Gun::from_index(self.core.gun).shotgun() {
            return;
        }
        let d = core::distance(
            Point::new(mx as i64, my as i64),
            Point::new(tx as i64, ty as i64),
        );
        let k = crate::valorant_guns::shotgun_percent(Gun::from_index(self.core.gun), d);
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
                base * k as usize * crate::valorant_guns::JUDGE_SPLASH_PERCENT as usize / 10000,
                0,
                AttackTypeV1::Skill,
            );
            crate::wave_near(
                sim,
                owner,
                e.id,
                9000,
                base * k as usize * crate::valorant_guns::JUDGE_SPLASH_PERCENT as usize / 10000,
                0,
            );
        }
    }
    fn on_update(&mut self, sim: &mut StableSim, _: u64, player: usize, owner: usize) {
        let tick = sim.tick();
        if self.last_tick == Some(tick) {
            return;
        }
        self.last_tick = Some(tick);
        self.owner = owner;
        // Round 144: the athlete's mastery rank, latched once the input AI has said who plays her.
        if self.rank.is_none() || self.athlete.is_none() {
            let athlete = BOOK.athlete_of(sim.seed(), player);
            if self.rank.is_none() || athlete.is_some() {
                let (rank, _) = BOOK.pinned(sim.seed()).rank_for(athlete);
                self.athlete = athlete;
                self.rank = Some(rank.min(MASTERY.len() - 1));
                if athlete.is_some() {
                    crate::mod_log(
                        sim,
                        "raze_log.txt",
                        &format!("rank:{owner}"),
                        &format!(
                            "athlete {athlete:?} plays Raze at {} mastery",
                            MASTERY[self.rank.unwrap_or(0)]
                        ),
                    );
                }
            }
        }
        if self.sig.is_none() && tick >= 1800 {
            let champs = crate::champions_shared(sim);
            self.sig = Some(crate::mastery::signature(&BOOK, sim, &champs, self.athlete));
        }
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
        let mut unlocked = [SkillSlotV1::Skill, SkillSlotV1::Skill2, SkillSlotV1::Ult]
            .map(|s| owner_entity.cooldown(s).is_some_and(|c| c.unlocked));
        // Round 144, as Gojo, Magnus and Unified: the game runs her passive_ult (the ult_learned marker) once
        // the ultimate is learned, so the marker unlocks it even when the slot flag does not.
        unlocked[2] |= buffs.iter().any(|b| b.name() == crate::ULT_LEARNED);
        let level = (owner_entity.level() as i64).max(1);
        self.ult_unlocked = unlocked[2];
        self.level = level;
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
            // Round 144: a Combustion reveal shows its target to her for BURN_REVEAL_TICKS
            let visible = e.team() == team
                || sim.is_visible(team, e.id())
                || (0..e.buff_count())
                    .filter_map(|i| e.buff_at(i))
                    .any(|b| b.name() == "rz_revealed");
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
                mr: s.magic_resistance as i64,
                visible,
                targetable: e.is_targetable(),
                kind,
                farmable: visible && context.eligible(e.id()),
                controlled: visible
                    && (0..e.cc_count()).filter_map(|i| e.cc_at(i)).any(|c| {
                        [
                            CcKindV1::Stun,
                            CcKindV1::Bind,
                            CcKindV1::Airborne,
                            CcKindV1::ForceMove,
                            CcKindV1::Fear,
                            CcKindV1::Charm,
                            CcKindV1::Taunt,
                        ]
                        .iter()
                        .any(|k| k.code() == c.kind)
                    }),
                melee: crate::valorant_guns::close_fighter(&e.name().unwrap_or_default()),
                push_immune: (0..e.buff_count())
                    .filter_map(|i| e.buff_at(i))
                    .any(|b| b.cc_immune),
                move_speed: s.move_speed as i64,
                defence: s.defence as i64,
                shield: e.shield() as i64,
                attack_range: crate::farming_data::attack_range(&e.name().unwrap_or_default())
                    + (0..e.buff_count())
                        .filter_map(|i| e.buff_at(i))
                        .map(|b| b.range as i64)
                        .sum::<i64>(),
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
        let mut surfaces = vec![];
        let mut owned_segments = std::collections::BTreeSet::new();
        for c in all.iter() {
            for b in &c.buffs {
                if let Some(raw) = b.name().strip_prefix("scs:") {
                    let v: Vec<i64> = raw.split(':').filter_map(|n| n.parse().ok()).collect();
                    if v.len() == 4 && v[3] > tick as i64 {
                        surfaces.push(Surface {
                            id: surface_key(&[c.id as i64, v[0], -1]),
                            bounds: Rect {
                                min: Point::new(v[1] - 8000, v[2] - 8000),
                                max: Point::new(v[1] + 8000, v[2] + 8000),
                            },
                        });
                    }
                }
                if let Some(raw) = b.name().strip_prefix("scseg:") {
                    let v: Vec<i64> = raw.split(':').filter_map(|n| n.parse().ok()).collect();
                    if v.len() == 9 && v[8] > tick as i64 && v[6] <= tick as i64 {
                        owned_segments.insert((v[2], v[3], v[4], v[5], v[6], v[7], v[8]));
                        let w = crate::steve::Wall {
                            ax: v[2],
                            ay: v[3],
                            bx: v[4],
                            by: v[5],
                            t0: v[6] as usize,
                            tb: v[7] as usize,
                            tend: v[8] as usize,
                        };
                        surfaces.extend(wall_surfaces(&w, tick, Some((c.id, v[0], v[1]))));
                    }
                }
            }
        }
        for w in crate::steve::walls_up(&all, tick).iter() {
            if owned_segments.contains(&(
                w.ax,
                w.ay,
                w.bx,
                w.by,
                w.t0 as i64,
                w.tb as i64,
                w.tend as i64,
            )) {
                continue;
            }
            surfaces.extend(wall_surfaces(w, tick, None));
        }
        surfaces.sort_by_key(|s| s.id);
        surfaces.dedup_by_key(|s| s.id);
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
        // Round 143: where the game walks her, and team guidance's next step from there (round-141 rules: plan gate,
        // farm and attack awareness, once per job, HP floor, the team_guidance switch, live targets, walls).
        let walk = walk_of(crate::match_key(sim), owner, tick);
        let guidance = walk.and_then(|w| crate::universal_ai::movement(sim, owner, w));
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
                self.credit(2, 0);
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
        let smokes = crate::valorant::smokes_up(&all, tick);
        let defenders = units
            .iter()
            .filter(|u| u.champion() && u.team != team && u.alive())
            .map(|u| {
                let traits = crate::athlete_ai::traits(sim, u.id);
                let mut values = [
                    traits.ego,
                    traits.judgement,
                    traits.positioning,
                    traits.control_speed,
                    traits.skill_hit,
                    traits.skill_avoid,
                    traits.aggressive,
                    traits.order,
                    traits.roaming,
                    traits.sense[0],
                    traits.sense[1],
                    traits.sense[2],
                    traits.sense[3],
                ];
                for v in &mut values {
                    *v = (*v).clamp(0, 100);
                }
                let at = self.core.rocket.as_ref().map_or(pos, |r| r.pos);
                let observed = sim.is_visible(u.team, owner)
                    || units.iter().any(|ally| {
                        ally.team == u.team
                            && ally.alive()
                            && matches!(ally.kind, 0 | 1 | 4)
                            && core::distance(ally.pos, at) <= 24000
                            && crate::walls::clip(ally.pos.x, ally.pos.y, at.x, at.y)
                                == (at.x, at.y)
                            && !smokes.iter().any(|s| {
                                crate::place::seg_dist2(
                                    (s.x, s.y),
                                    (ally.pos.x, ally.pos.y),
                                    (at.x, at.y),
                                ) <= crate::sq(crate::valorant::SMOKE_R)
                            })
                    });
                Defender {
                    id: u.id,
                    profile: Profile {
                        values,
                        known: traits.known,
                    },
                    observed,
                }
            })
            .collect();
        let frame = Frame {
            version: 3,
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
            surfaces,
            defenders,
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
            assignment: guidance.map(|p| Point::new(p.0, p.1)),
            objective,
            missed_wave,
            intent: all
                .iter()
                .find(|c| c.id == owner)
                .and_then(|me| rocket_escape(sim, me, tick))
                .map(|p| Point::new(p.0, p.1)),
            action: None,
            steering: None,
            practice_inventory: false,
            haste,
            ult_haste,
            walk: walk.map(|p| Point::new(p.0, p.1)),
            level,
            mastery: self.rank.unwrap_or(0) as u8,
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
        let guided = commands.iter().any(|c| matches!(c, Cmd::Guide { .. }));
        self.commands(sim, commands, team, tick, &all);
        if !guided {
            // the core stopped guiding (a new decision, a flight or a cast): hand the walk back now
            if let Some(n) = self.guide.take() {
                sim.entity_remove_buff(owner, &n);
            }
        }
        if !sim.get_entity(owner).is_some_and(|e| e.is_alive()) {
            self.core.died();
            self.clean(sim);
            return;
        }
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
            // Round 143, as the Lab: a Boom Bot throw keeps the idle pose.
            pose.set_name(match c.action {
                core::Action::Pack { .. } | core::Action::DropPack => "skill2",
                core::Action::EquipRocket => "ult",
                core::Action::Bot { .. } => "idle",
                _ => "skill1",
            });
            sim.apply_cc(owner, &pose);
        } else if self.core.armed_until > tick as u64 && !self.core.airborne() {
            // an armed rocket shows the ult pose, as in the Lab
            let mut pose = CcV1::of_kind(CcKindV1::Animation, 2);
            pose.set_name("ult");
            sim.apply_cc(owner, &pose);
        }
        if self.core.armed_until > tick as u64 && tick as u64 >= self.ready_at + 24 {
            // the armed marker plays 24 ticks; renew it while the rocket is armed
            self.ready_at = tick as u64;
            if let Some((x, y)) = sim.get_entity(owner).map(|e| e.pos()) {
                fx_point(
                    sim,
                    &format!("{ID}_rocket_ready_big"),
                    owner,
                    x as i64,
                    y as i64,
                    24,
                );
            }
        }
        if self.core.airborne() {
            let mut b = crate::timed("rz_air", 2);
            b.move_speed_mult = -10000;
            sim.add_buff(owner, &b);
            let mut pose = CcV1::of_kind(CcKindV1::Animation, 2);
            pose.set_name("air_hidden");
            sim.apply_cc(owner, &pose);
        } else {
            sim.entity_remove_buff(owner, "rz_air");
        }
        if self.core.flight.as_ref().is_some_and(|v| v.grounded) {
            let mut b = crate::timed("rz_slide", 2);
            b.move_speed_mult = -10000;
            sim.add_buff(owner, &b);
        } else {
            sim.entity_remove_buff(owner, "rz_slide");
        }
        self.draw_carriers(sim, tick as u64);
        self.phases.retain(|_, (_, last)| *last + 1 >= tick as u64);
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
        self.kit_hud(sim, tick as u64);
        if let (Some(sig), Some(a)) = (&self.sig, self.athlete) {
            if tick.is_multiple_of(30) {
                self.record.track(&BOOK, sim, sig, team, a, tick);
            }
        }
        self.perf.finish("host", host_at);
    }
}
/// Ticks a guide buff lives: longer than the 12-tick decision interval (round 143; was 18 and lapsed mid-plan).
const GUIDE_LIFE: usize = 60;
pub fn controls_velocity(ctx: &mut StableAiContext) -> bool {
    let player = ctx.player_id();
    ctx.sim().is_some_and(|sim| {
        sim.get_player(player)
            .and_then(|p| p.champion())
            .is_some_and(|e| has(&sim, e.id(), "rz_air") || has(&sim, e.id(), "rz_slide"))
    })
}
pub fn guide_input(ctx: &mut StableAiContext, input: &InputV1) -> Option<InputV1> {
    let player = ctx.player_id();
    let dest = {
        let sim = ctx.sim()?;
        let me = sim.get_player(player)?.champion()?;
        // Round 143: remember where the game walks her (the takeoff gate and team guidance read it).
        note_walk(
            crate::match_key(&sim),
            me.id(),
            input.x as i64,
            input.y as i64,
            sim.tick(),
        );
        if has(&sim, me.id(), "rz_air") || has(&sim, me.id(), "rz_slide") {
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
            if let Some(raw) = b
                .name()
                .strip_prefix("rzw:")
                .or_else(|| b.name().strip_prefix("rzrw:"))
            {
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
pub fn rocket_escape(sim: &StableSim, me: &crate::Champ, tick: usize) -> Option<(i64, i64)> {
    if me.stunned {
        return None;
    }
    me.buffs
        .iter()
        .filter_map(|b| b.name().strip_prefix("rze:"))
        .filter_map(|s| {
            let v: Vec<i64> = s.split(':').filter_map(|n| n.parse().ok()).collect();
            (v.len() == 5 && v[4] > tick as i64).then(|| (v[4], v[0], (v[2], v[3])))
        })
        .filter(|(_, owner, _)| {
            sim.get_entity(*owner as usize)
                .is_some_and(|e| e.is_alive())
        })
        .min_by_key(|v| (v.0, v.1))
        .map(|v| v.2)
}
pub fn avoid(
    all: &[crate::Champ],
    me: &crate::Champ,
    dest: (i64, i64),
    tick: usize,
) -> Option<(i64, i64)> {
    // Rockets have their own once-per-projectile reaction and guidance; do not
    // let the legacy circular-zone path bypass it or reveal an unseen launch.
    let zones: Vec<_> = all
        .iter()
        .filter(|c| c.team != me.team)
        .flat_map(|c| c.buffs.iter())
        .filter_map(|b| b.name().strip_prefix("rzw:"))
        .filter_map(|s| {
            let a: Vec<i64> = s.split(':').filter_map(|n| n.parse().ok()).collect();
            (a.len() == 5 && a[4] > tick as i64).then(|| (a[1], a[2], a[3]))
        })
        .collect();
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
