//! Aegis Zero: the data champion owns targeting, cooldowns and the ally's immediate shield. This passive supplies the
//! charge (terrain push), the melee basic attack and the ally-bound arrival.
//!
//! Round 88 (Rian: "the skill does nothing, just some animation"; redrawn after Destiny Gundam):
//! - One wing visual at a time: the Wings of Light are a single buff (`gdm_wings`, then `gdm_fade` for the last second
//!   of Zero Protection), opened by a one-shot `deploy` effect and closed by a one-shot `retract` effect. The old
//!   overlapping wing buffs, the flight effect spam and the CasterAnimation poses (which never showed) are gone; in
//!   flight he leaves pink afterimages.
//! - The basic attack is melee and only real basic attacks count (on_base_attack): a beam-saber cut, every third one a
//!   Palma Fiocina palm blast for 25% more.
//! - Crowd control stops the charge and the flight.
//! - While the ult runs his S1 / S2 presses would be wasted: the input AI turns them into basic attacks (press flags).
//! - gundam_log.txt (next to the DLL) gets a line for every cast the passive sees and every phase it runs, so the first
//!   live game shows whether the skills fire.
use mod_api_stable::{AttackTypeV1, BuffV1, CastingTargetV1, CcKindV1, CcV1, InputTargetV1, ProjectileMoveKindV1,
    ProjectileSpawnV1, StablePassive, StableSim};
use crate::{champions, d2, sq, walls, Champ, MOD_ID};

const ID: &str = "tfm2_gundam_aegis_zero";
const PROTECT_R: i64 = 60_000;
/// Round 90 (Rian: "like Galio"): the landing zone, marked on the ground for the whole flight: enemies in the inner
/// circle are knocked up and take the damage, those in the outer ring are only slowed. Round 93 (Rian: "Aegis Zero
/// needs a buff on the ult ... the area of Gojo's domain and DIO's time stop"): the inner circle is their size (76000,
/// was 35000) and the outer ring 1.5x that (114000, was 75000).
const KNOCK_R: i64 = 76_000;
const SLOW_R: i64 = 114_000;
const SLOW_TICKS: usize = 90;
const SLOW_PCT: i32 = 35;
/// He rises out of sight on the Wings of Light (banished: untargetable, unseen) before the flight.
const ASCEND_TICKS: usize = 18;
/// The last part of the flight: he dives onto the mark.
const DIVE_TICKS: usize = 14;
/// The zone's countdown frames (zone_f0..7) and how often the sky / zone visuals are re-emitted.
const ZONE_FRAMES: usize = 8;
const SKY_EVERY: usize = 6;
/// When he flies in (defense only): an ally at this HP or less with a visible enemy champion this close, or a teamfight
/// (2+ allies and 2+ visible enemies within FIGHT_R of an ally at least AWAY_R from him).
const SAVE_PCT: usize = 35;
const HUNT_R: i64 = 90_000;
const FIGHT_R: i64 = 120_000;
const AWAY_R: i64 = 60_000;
const AURA_R: i64 = 52_000;
const CHARGE_SPEED: i64 = 3_500;
const CHARGE_TICKS: usize = 21;
/// The wings open (the 6-frame deploy effect, ~22 ticks), then he takes off.
const DEPLOY_TICKS: usize = 26;
const FLIGHT_TICKS: usize = 80;
const ZERO_TICKS: usize = 360;
const FADE_TICKS: usize = 60;
/// The 6-frame retract effect (~25 ticks).
const RETRACT_TICKS: usize = 25;
/// A real basic attack lands within this many ticks of on_base_attack.
const BASE_WINDOW: usize = 30;
/// Every third basic attack is the palm blast.
const PALM_EVERY: usize = 3;
/// Round 90 S1, Beam Saber Unleash: the beam saber cuts a strip of ground toward the target; everyone in it is hurt and
/// slowed.
const SLICE_LEN: i64 = 65_000;
const SLICE_W: i64 = 11_000;
const SLICE_TICKS: usize = 8;
const SLICED_TICKS: usize = 90;
const SLICED_PCT: i32 = 30;
/// Round 90 S2, Arondight: the dash ends in a 37000 cut (damage, a 1 s taunt, the shield), then he holds the great
/// sword for STANCE_TICKS: his basic attacks swing it (+15%), burn (BURN_HITS ticks of damage, one every
/// BURN_EVERY, refreshed by each hit, never stacked) and pull the target a little toward him.
const CUT_R: i64 = 37_000;
const STANCE_TICKS: usize = 300;
const BURN_EVERY: usize = 30;
const BURN_HITS: usize = 3;
const PULL_SPEED: usize = 1_000;
const PULL_TICKS: usize = 6;
const WINGS: &str = "gdm_wings";
const FADE: &str = "gdm_fade";

#[derive(Clone, Debug, PartialEq)]
enum Phase {
    Charge { x: i64, y: i64, dx: f64, dy: f64, start: usize, pushed: Option<usize> },
    Deploy { target: usize, start: usize },
    Ascend { target: usize, start: usize },
    Flight { target: usize, start: usize, from: (i64, i64), last: (i64, i64) },
    Empowered,
    Retract { start: usize },
}

#[derive(Clone, Default)]
pub struct Gundam {
    me: Option<usize>,
    phase: Option<Phase>,
    base_until: usize,
    hits: usize,
    /// round 90: the ally the ult would fly to now (save or teamfight), refreshed every 10 ticks
    ult_ally: Option<usize>,
    /// round 90: Arondight held until this tick; burning enemies (id, next burn tick, burns left)
    stance_until: usize,
    burns: Vec<(usize, usize, usize)>,
}

fn fx(sim: &mut StableSim<'_>, caster: usize, tag: &str, target: InputTargetV1) {
    let name = format!("{ID}_{tag}");
    let _ = sim.play_view_effect(&name, caster, &target, 0, 0, 0);
}

fn has(e: &mod_api_stable::StableEntity<'_, '_>, name: &str) -> bool {
    (0..e.buff_count()).filter_map(|i| e.buff_at(i)).any(|b| b.name() == name)
}

fn buff_ticks(e: &mod_api_stable::StableEntity<'_, '_>, name: &str) -> Option<usize> {
    (0..e.buff_count()).filter_map(|i| e.buff_at(i)).find(|b| b.name() == name).map(|b| b.duration_tick)
}

fn living(all: &[Champ], id: usize) -> Option<&Champ> {
    all.iter().find(|c| c.id == id)
}

fn aim_target<'a>(me: &Champ, all: &'a [Champ], marker: &str, ally: bool) -> Option<&'a Champ> {
    all.iter()
        .filter(|c| c.id != me.id && (c.team == me.team) == ally && c.has(marker))
        .min_by_key(|c| (d2(me.x, me.y, c.x, c.y), c.id))
}

fn put(sim: &mut StableSim<'_>, id: usize, x: i64, y: i64) {
    let _ = sim.entity_set_pos(id, x.max(0) as u64, y.max(0) as u64);
}

fn remove(sim: &mut StableSim<'_>, id: usize, names: &[&str]) {
    for n in names { sim.entity_remove_buff(id, n); }
}

/// The wing visual for each phase: exactly one of none / the Wings of Light / their last-second flicker.
fn wing_buff(phase: Option<&Phase>, zero_left: Option<usize>, deploy_elapsed: usize) -> Option<&'static str> {
    match phase {
        Some(Phase::Deploy { .. }) if deploy_elapsed >= DEPLOY_TICKS - 4 => Some(WINGS),
        // round 90: risen out of sight: the ascend / sky / dive effects carry the wings
        Some(Phase::Ascend { .. } | Phase::Flight { .. }) => None,
        Some(Phase::Empowered) => Some(if zero_left.is_some_and(|t| t <= FADE_TICKS) { FADE } else { WINGS }),
        _ => None,
    }
}

fn show_wings(sim: &mut StableSim<'_>, me: usize, want: Option<&str>) {
    for name in [WINGS, FADE] {
        let on = sim.get_entity(me).is_some_and(|e| has(&e, name));
        if Some(name) == want {
            if !on { sim.add_buff(me, &BuffV1::named(name)); }
        } else if on {
            sim.entity_remove_buff(me, name);
        }
    }
}

fn clear_ally_aura(sim: &mut StableSim<'_>, me: usize, team: usize) {
    for i in 0..sim.champion_count() {
        let id = sim.champion_id_at(i);
        if id != me && sim.get_entity(id).is_some_and(|e| e.team() == team) {
            sim.entity_remove_buff(id, "gdm_zero_ally");
        }
    }
}

fn retract(sim: &mut StableSim<'_>, me: usize, team: usize, tick: usize) -> Phase {
    remove(sim, me, &[WINGS, FADE, "gdm_zero"]);
    clear_ally_aura(sim, me, team);
    fx(sim, me, "retract", InputTargetV1::target(me));
    Phase::Retract { start: tick }
}

/// Where he lands for an ally: 14000 from them toward the nearest enemy (onto the ally if that's a wall).
fn landing_spot(target: &Champ, all: &[Champ]) -> (i64, i64) {
    let nearest = all.iter().filter(|e| e.team != target.team)
        .min_by_key(|e| (d2(e.x, e.y, target.x, target.y), e.id));
    let Some(enemy) = nearest else { return (target.x + 14_000, target.y) };
    let vx = (enemy.x - target.x) as f64;
    let vy = (enemy.y - target.y) as f64;
    let len = vx.hypot(vy).max(1.0);
    let wanted = (target.x + (vx / len * 14_000.0) as i64,
                  target.y + (vy / len * 14_000.0) as i64);
    if walls::wall_at(wanted.0, wanted.1) { (target.x, target.y) } else { wanted }
}

/// Round 90: the enemies the landing touches, split into the inner circle (knocked up) and the outer ring (slowed).
fn zone_split(center: (i64, i64), enemies: &[(usize, (i64, i64), bool)]) -> (Vec<(usize, bool)>, Vec<usize>) {
    let mut knocked = Vec::new();
    let mut slowed = Vec::new();
    for &(id, (x, y), champion) in enemies {
        let d = d2(x, y, center.0, center.1);
        if d <= sq(KNOCK_R) { knocked.push((id, champion)); } else if d <= sq(SLOW_R) { slowed.push(id); }
    }
    (knocked, slowed)
}

/// The landing: the inner circle takes 90 + 70% Attack and champions there are knocked up; the outer ring is slowed.
fn impact(sim: &mut StableSim<'_>, me: &Champ, x: i64, y: i64) -> (usize, usize) {
    fx(sim, me.id, "landing", InputTargetV1::pos(x.max(0) as u64, y.max(0) as u64));
    let attack = sim.get_entity(me.id).map(|e| e.stat().attack).unwrap_or(68);
    let damage = 90 + attack * 70 / 100;
    // collect first: damage can kill and reshuffle the entity list
    let mut enemies: Vec<(usize, (i64, i64), bool)> = Vec::new();
    for i in 0..sim.entity_count() {
        let Some(e) = sim.entity_at(i) else { continue };
        if !e.is_alive() || e.team() == me.team || e.is_tower() { continue; }
        let (ex, ey) = e.pos();
        enemies.push((e.id(), (ex as i64, ey as i64), e.is_champion()));
    }
    let (knocked, slowed) = zone_split((x, y), &enemies);
    for &(id, champion) in &knocked {
        sim.deal_damage(me.id, id, damage, 0, AttackTypeV1::Skill);
        if champion { sim.apply_cc(id, &CcV1::of_kind(CcKindV1::Airborne, 60)); }
    }
    for &id in &slowed {
        sim.entity_remove_buff(id, "gdm_slowed");
        let mut b = BuffV1::timed("gdm_slowed", SLOW_TICKS);
        b.move_speed_mult = -SLOW_PCT;
        sim.add_buff(id, &b);
    }
    (knocked.iter().filter(|h| h.1).count(), slowed.len())
}

/// Round 90: who the ult should fly to now, if anyone (defense only). First an ally at SAVE_PCT HP or less with a
/// visible enemy champion within HUNT_R (the lowest first); else a teamfight he isn't in yet: an ally at least AWAY_R
/// away with another ally and 2+ visible enemy champions within FIGHT_R (the one with the most enemies near, then
/// the lowest HP). Never for farming or a chase.
fn ult_choice(me: &Champ, all: &[Champ], visible: impl Fn(usize) -> bool) -> Option<usize> {
    let allies: Vec<&Champ> = all.iter().filter(|a| a.team == me.team && a.id != me.id).collect();
    let foes: Vec<&Champ> = all.iter().filter(|e| e.team != me.team && visible(e.id)).collect();
    let near = |a: &Champ, r: i64| foes.iter().filter(|e| d2(e.x, e.y, a.x, a.y) <= sq(r)).count();
    let pct = |a: &Champ| a.hp * 100 / a.max_hp.max(1);
    if let Some(a) = allies.iter().filter(|a| pct(a) <= SAVE_PCT && near(a, HUNT_R) > 0)
        .min_by_key(|a| (pct(a), a.id)) {
        return Some(a.id);
    }
    allies.iter().filter(|a| d2(me.x, me.y, a.x, a.y) >= sq(AWAY_R))
        .filter(|a| near(a, FIGHT_R) >= 2 && allies.iter().any(|b| b.id != a.id && d2(a.x, a.y, b.x, b.y) <= sq(FIGHT_R)))
        .max_by_key(|a| (near(a, FIGHT_R), std::cmp::Reverse(pct(a)), std::cmp::Reverse(a.id)))
        .map(|a| a.id)
}

/// Round 90: whether `p` is in the beam-saber strip from `from` along the unit direction `dir`.
fn in_slice(from: (i64, i64), dir: (f64, f64), p: (i64, i64)) -> bool {
    let (vx, vy) = ((p.0 - from.0) as f64, (p.1 - from.1) as f64);
    let along = vx * dir.0 + vy * dir.1;
    let across = (vx * dir.1 - vy * dir.0).abs();
    along >= -4_000.0 && along <= SLICE_LEN as f64 && across <= SLICE_W as f64
}

/// The 16 ground-cut angles (0..pi, like Isliid's strokes).
fn cut_angle(dir: (f64, f64)) -> usize {
    ((dir.1.atan2(dir.0).rem_euclid(std::f64::consts::PI) * 16.0 / std::f64::consts::PI).round() as usize) % 16
}

/// A sword hit (re)lights the burn: BURN_HITS burns, the first BURN_EVERY from now; never a second burn on one target.
fn refresh_burn(burns: &mut Vec<(usize, usize, usize)>, target: usize, tick: usize) {
    burns.retain(|b| b.0 != target);
    burns.push((target, tick + BURN_EVERY, BURN_HITS));
}

/// Round 91: the ult phases in which he must not attack (opening the wings, rising, flying, diving).
fn holds(phase: Option<&Phase>) -> bool {
    matches!(phase, Some(Phase::Deploy { .. } | Phase::Ascend { .. } | Phase::Flight { .. }))
}

/// How long the banish re-applied on `elapsed` (every 10 ticks) lasts: never past the landing.
fn banish_ticks(elapsed: usize, total: usize) -> usize {
    total.saturating_sub(elapsed).min(12)
}

// ------------------------------------------------------------------ diagnostics

/// One line in gundam_log.txt (next to the DLL); `key` keeps the same event from being written twice (the game runs
/// two simulations of each match).
fn log(sim: &StableSim<'_>, key: &str, line: &str) { crate::mod_log(sim, "gundam_log.txt", key, line); }

impl Gundam {
    fn clean(&mut self, sim: &mut StableSim<'_>) {
        if let Some(me) = self.me {
            let team = sim.get_entity(me).map(|e| e.team()).unwrap_or(usize::MAX);
            remove(sim, me, &["gdm_armor", "gdm_protect", "gdm_charge", "gdm_slice", "gdm_arondight", "gdm_arrival_cast", "gdm_ult_ok",
                WINGS, FADE, "gdm_zero"]);
            clear_ally_aura(sim, me, team);
        }
        self.phase = None;
        self.stance_until = 0;
        self.burns.clear();
    }

    fn passive(&self, sim: &mut StableSim<'_>, me: &Champ, all: &[Champ]) {
        let Some(e) = sim.get_entity(me.id) else { return };
        if !has(&e, "gdm_armor") {
            let mut b = BuffV1::named("gdm_armor");
            b.defence = 8;
            b.magic_resistance = 8;
            sim.add_buff(me.id, &b);
        }
        let endangered = all.iter().any(|a| a.id != me.id && a.team == me.team &&
            a.hp * 100 <= a.max_hp * 40 && d2(me.x, me.y, a.x, a.y) <= sq(PROTECT_R));
        let active = sim.get_entity(me.id).is_some_and(|e| has(&e, "gdm_protect"));
        if endangered && !active {
            let mut b = BuffV1::named("gdm_protect");
            b.defence = 10;
            b.magic_resistance = 10;
            sim.add_buff(me.id, &b);
        } else if !endangered && active {
            sim.entity_remove_buff(me.id, "gdm_protect");
        }
    }

    /// Round 90 S1: Beam Saber Unleash, on the data's gdm_slice marker: the cut runs from him through the target.
    fn slice(&mut self, sim: &mut StableSim<'_>, me: &Champ, all: &[Champ]) {
        if !sim.get_entity(me.id).is_some_and(|e| has(&e, "gdm_slice")) { return; }
        sim.entity_remove_buff(me.id, "gdm_slice");
        let Some(target) = aim_target(me, all, "gdm_slice_target", false) else {
            log(sim, &format!("s1.none.{}", sim.tick()), "Beam Saber Unleash cast, but the target marker wasn't found");
            return;
        };
        sim.entity_remove_buff(target.id, "gdm_slice_target");
        let (vx, vy) = ((target.x - me.x) as f64, (target.y - me.y) as f64);
        let len = vx.hypot(vy).max(1.0);
        let dir = (vx / len, vy / len);
        let end = (me.x + (dir.0 * SLICE_LEN as f64) as i64, me.y + (dir.1 * SLICE_LEN as f64) as i64);
        // the wave (a cosmetic projectile the engine turns to its heading) and the cut it leaves
        let spec = ProjectileSpawnV1 {
            caster_id: me.id, team: me.team, x: me.x.max(0) as u64, y: me.y.max(0) as u64, radius: 1_000,
            speed: (SLICE_LEN as usize / SLICE_TICKS) as u64, move_kind: ProjectileMoveKindV1::Linear.code(),
            target_x: end.0.max(0) as u64, target_y: end.1.max(0) as u64,
            penetrate: true, casting_target: CastingTargetV1::None.code(), ..ProjectileSpawnV1::default()
        };
        sim.spawn_projectile(&format!("{ID}_slice_wave"), &format!("{MOD_ID}:noop"), &spec);
        let a = cut_angle(dir);
        for k in 0..3 {
            let u = (2 * k + 1) as f64 / 6.0 * SLICE_LEN as f64;
            let p = (me.x + (dir.0 * u) as i64, me.y + (dir.1 * u) as i64);
            fx(sim, me.id, &format!("slice_cut_a{a}"), InputTargetV1::pos(p.0.max(0) as u64, p.1.max(0) as u64));
        }
        let attack = sim.get_entity(me.id).map(|e| e.stat().attack).unwrap_or(me.attack);
        let mut hit = Vec::new();
        for i in 0..sim.entity_count() {
            let Some(e) = sim.entity_at(i) else { continue };
            if !e.is_alive() || e.team() == me.team || e.is_tower() { continue; }
            let (x, y) = e.pos();
            if in_slice((me.x, me.y), dir, (x as i64, y as i64)) { hit.push(e.id()); }
        }
        for &id in &hit {
            sim.deal_damage(me.id, id, 45 + attack * 55 / 100, 0, AttackTypeV1::Skill);
            sim.entity_remove_buff(id, "gdm_sliced");
            let mut b = BuffV1::timed("gdm_sliced", SLICED_TICKS);
            b.move_speed_mult = -SLICED_PCT;
            sim.add_buff(id, &b);
            fx(sim, me.id, "saber", InputTargetV1::target(id));
        }
        log(sim, &format!("s1.{}", sim.tick()), &format!("Beam Saber Unleash: {} hit and slowed", hit.len()));
    }

    /// Round 90 S2: the dash ends in Arondight's cut: damage and a 1 s taunt in CUT_R, the shield (120 + 65 per champion),
    /// then the sword stance.
    fn finisher(&mut self, sim: &mut StableSim<'_>, me: &Champ, at: (i64, i64), tick: usize) {
        fx(sim, me.id, "sword_draw", InputTargetV1::target(me.id));
        fx(sim, me.id, "sweep", InputTargetV1::target(me.id));
        let attack = sim.get_entity(me.id).map(|e| e.stat().attack).unwrap_or(me.attack);
        let mut hit: Vec<(usize, bool)> = Vec::new();
        for i in 0..sim.entity_count() {
            let Some(e) = sim.entity_at(i) else { continue };
            if !e.is_alive() || e.team() == me.team || e.is_tower() { continue; }
            let (x, y) = e.pos();
            if d2(x as i64, y as i64, at.0, at.1) <= sq(CUT_R) { hit.push((e.id(), e.is_champion())); }
        }
        let champs = hit.iter().filter(|h| h.1).count();
        for &(id, champion) in &hit {
            sim.deal_damage(me.id, id, 40 + attack * 55 / 100, 0, AttackTypeV1::Skill);
            if champion {
                let mut taunt = CcV1::of_kind(CcKindV1::Taunt, 60);
                taunt.target = me.id;
                sim.apply_cc(id, &taunt);
            }
        }
        if champs > 0 { let _ = sim.entity_add_shield(me.id, 120 + 65 * champs, 240); }
        if sim.get_entity(me.id).is_some_and(|e| has(&e, "gdm_zero")) {
            fx(sim, me.id, "flare", InputTargetV1::target(me.id));
        }
        self.stance_until = tick + STANCE_TICKS;
        sim.entity_remove_buff(me.id, "gdm_arondight");
        sim.add_buff(me.id, &BuffV1::timed("gdm_arondight", STANCE_TICKS));
        log(sim, &format!("s2.{tick}"), &format!("Arondight: cut {} ({champs} champion(s) taunted), sword drawn", hit.len()));
    }

    /// Round 90: the burns from Arondight's swings tick on.
    fn burn(&mut self, sim: &mut StableSim<'_>, me: &Champ, tick: usize) {
        let attack = sim.get_entity(me.id).map(|e| e.stat().attack).unwrap_or(me.attack);
        let mut due = Vec::new();
        for b in &mut self.burns {
            if tick >= b.1 && b.2 > 0 { due.push(b.0); b.1 = tick + BURN_EVERY; b.2 -= 1; }
        }
        self.burns.retain(|b| b.2 > 0);
        for id in due {
            if sim.get_entity(id).is_some_and(|e| e.is_alive()) {
                sim.deal_damage(me.id, id, 10 + attack / 10, 0, AttackTypeV1::Skill);
            }
        }
    }

    fn aura(&self, sim: &mut StableSim<'_>, me: &Champ, all: &[Champ]) {
        for a in all.iter().filter(|a| a.team == me.team && a.id != me.id &&
            d2(a.x, a.y, me.x, me.y) <= sq(AURA_R)) {
            let refresh = sim.get_entity(a.id).and_then(|e| buff_ticks(&e, "gdm_zero_ally"))
                .is_none_or(|t| t < 5);
            if refresh {
                sim.entity_remove_buff(a.id, "gdm_zero_ally");
                let mut b = BuffV1::timed("gdm_zero_ally", 20);
                b.defence = 8;
                b.magic_resistance = 8;
                sim.add_buff(a.id, &b);
            }
        }
    }

    /// Advance the running phase one tick.
    fn step(&mut self, sim: &mut StableSim<'_>, me: &Champ, all: &[Champ], tick: usize) {
        let held = me.stunned;
        let state = self.phase.take();
        self.phase = match state {
            Some(Phase::Charge { x, y, dx, dy, start, mut pushed }) => {
                let nx = x + (dx * CHARGE_SPEED as f64) as i64;
                let ny = y + (dy * CHARGE_SPEED as f64) as i64;
                let wall = walls::wall_at(nx + (dx * 13_000.0) as i64, ny + (dy * 13_000.0) as i64);
                if pushed.is_none() {
                    pushed = all.iter().filter(|c| c.team != me.team && d2(c.x, c.y, nx, ny) <= sq(15_000))
                        .min_by_key(|c| (d2(c.x, c.y, nx, ny), c.id)).map(|c| c.id);
                    if let Some(id) = pushed { fx(sim, me.id, "charge_hit", InputTargetV1::target(id)); }
                }
                let done = held || tick.saturating_sub(start) >= CHARGE_TICKS || wall;
                if wall {
                    if let Some(id) = pushed {
                        sim.apply_cc(id, &CcV1::stun(60));
                        fx(sim, me.id, "wall_hit", InputTargetV1::target(id));
                    }
                } else if !held {
                    put(sim, me.id, nx, ny);
                    if tick.is_multiple_of(3) {
                        fx(sim, me.id, if dx >= 0.0 { "after_r" } else { "after_l" }, InputTargetV1::pos(x.max(0) as u64, y.max(0) as u64));
                    }
                    if let Some(id) = pushed {
                        if living(all, id).is_some() {
                            put(sim, id, nx + (dx * 12_000.0) as i64, ny + (dy * 12_000.0) as i64);
                            sim.apply_cc(id, &CcV1::stun(2));
                        } else { pushed = None; }
                    }
                }
                if done {
                    sim.entity_remove_buff(me.id, "gdm_charge");
                    log(sim, &format!("s2dash.{start}"), &format!("Arondight dash ended ({}), carried {:?}",
                        if held { "crowd control" } else if wall { "wall" } else { "distance" }, pushed));
                    // round 90: the dash ends in Arondight's cut and the sword stance (crowd control still cuts it short)
                    if !held { self.finisher(sim, me, if wall { (x, y) } else { (nx, ny) }, tick); }
                    None
                } else { Some(Phase::Charge { x: nx, y: ny, dx, dy, start, pushed }) }
            }
            Some(Phase::Deploy { target, start }) => {
                if living(all, target).is_none() {
                    log(sim, &format!("ult.dead.{start}"), "Wings of Light: the ally died before take-off");
                    Some(retract(sim, me.id, me.team, tick))
                } else if held {
                    log(sim, &format!("ult.cc.{start}"), "Wings of Light: stopped by crowd control before take-off");
                    Some(retract(sim, me.id, me.team, tick))
                } else if tick.saturating_sub(start) >= DEPLOY_TICKS {
                    // round 90: he rises out of sight on the wings (the ascend effect draws him going up)
                    log(sim, &format!("ult.rise.{start}"), &format!("Wings of Light: rising, bound for ally {target}"));
                    fx(sim, me.id, "ascend", InputTargetV1::pos(me.x.max(0) as u64, me.y.max(0) as u64));
                    sim.entity_banish(me.id, me.id, banish_ticks(0, ASCEND_TICKS + FLIGHT_TICKS), "", "");
                    Some(Phase::Ascend { target, start: tick })
                } else { Some(Phase::Deploy { target, start }) }
            }
            Some(Phase::Ascend { target, start }) => {
                let elapsed = tick.saturating_sub(start);
                if elapsed % 10 == 0 {
                    sim.entity_banish(me.id, me.id, banish_ticks(elapsed, ASCEND_TICKS + FLIGHT_TICKS), "", "");
                }
                if elapsed >= ASCEND_TICKS {
                    log(sim, &format!("ult.fly.{start}"), &format!("Wings of Light: in the sky toward ally {target}"));
                    let last = living(all, target).map(|a| landing_spot(a, all)).unwrap_or((me.x, me.y));
                    Some(Phase::Flight { target, start: tick, from: (me.x, me.y), last })
                } else { Some(Phase::Ascend { target, start }) }
            }
            Some(Phase::Flight { target, start, from, mut last }) => {
                // the mark follows the ally until he dives (then it's set)
                let elapsed = tick.saturating_sub(start);
                if elapsed < FLIGHT_TICKS - DIVE_TICKS {
                    if let Some(a) = living(all, target) { last = landing_spot(a, all); }
                }
                if (elapsed + ASCEND_TICKS) % 10 == 0 {
                    sim.entity_banish(me.id, me.id, banish_ticks(elapsed + ASCEND_TICKS, ASCEND_TICKS + FLIGHT_TICKS), "", "");
                }
                if elapsed >= FLIGHT_TICKS {
                    let pos = if walls::wall_at(last.0, last.1) { from } else { last };
                    put(sim, me.id, pos.0, pos.1);
                    let (n, slowed) = impact(sim, me, pos.0, pos.1);
                    let mut b = BuffV1::timed("gdm_zero", ZERO_TICKS);
                    b.defence = 20;
                    b.magic_resistance = 20;
                    sim.add_buff(me.id, &b);
                    log(sim, &format!("ult.land.{start}"), &format!("Wings of Light: landed, {n} champion(s) knocked up, {slowed} slowed"));
                    Some(Phase::Empowered)
                } else {
                    // ease in and out across the map
                    let f = elapsed as f64 / FLIGHT_TICKS as f64;
                    let e = f * f * (3.0 - 2.0 * f);
                    let x = from.0 + ((last.0 - from.0) as f64 * e) as i64;
                    let y = from.1 + ((last.1 - from.1) as f64 * e) as i64;
                    // round 90: high in the sky (the figure drawn well above a ground shadow), the zone marked where
                    // he'll land, then the dive
                    if elapsed.is_multiple_of(SKY_EVERY) {
                        if elapsed < FLIGHT_TICKS - DIVE_TICKS {
                            fx(sim, me.id, if last.0 >= from.0 { "sky_r" } else { "sky_l" }, InputTargetV1::pos(x.max(0) as u64, y.max(0) as u64));
                        }
                        let k = (elapsed * ZONE_FRAMES / FLIGHT_TICKS).min(ZONE_FRAMES - 1);
                        fx(sim, me.id, &format!("zone_f{k}"), InputTargetV1::pos(last.0.max(0) as u64, last.1.max(0) as u64));
                    }
                    if elapsed == FLIGHT_TICKS - DIVE_TICKS {
                        log(sim, &format!("ult.dive.{start}"), "Wings of Light: diving");
                        fx(sim, me.id, "dive", InputTargetV1::pos(last.0.max(0) as u64, last.1.max(0) as u64));
                    }
                    put(sim, me.id, x, y);
                    Some(Phase::Flight { target, start, from, last })
                }
            }
            Some(Phase::Empowered) => {
                if sim.get_entity(me.id).and_then(|e| buff_ticks(&e, "gdm_zero")).is_some() {
                    self.aura(sim, me, all);
                    Some(Phase::Empowered)
                } else { Some(retract(sim, me.id, me.team, tick)) }
            }
            Some(Phase::Retract { start }) => {
                if tick.saturating_sub(start) >= RETRACT_TICKS { None } else { Some(Phase::Retract { start }) }
            }
            None => None,
        };
    }

    /// Start a phase from the data's cast markers (only when none is running).
    fn start(&mut self, sim: &mut StableSim<'_>, me: &Champ, all: &[Champ], tick: usize) {
        if sim.get_entity(me.id).is_some_and(|e| has(&e, "gdm_arrival_cast")) {
            sim.entity_remove_buff(me.id, "gdm_arrival_cast");
            let target = if let Some(a) = aim_target(me, all, "gdm_arrival_target", true) {
                sim.entity_remove_buff(a.id, "gdm_arrival_target");
                Some(a.id)
            } else if sim.get_entity(me.id).is_some_and(|e| has(&e, "gdm_arrival_target")) {
                // some game builds include himself in AllyChampion: redirect to the nearest real ally, with its shield
                sim.entity_remove_buff(me.id, "gdm_arrival_target");
                let a = all.iter().filter(|a| a.id != me.id && a.team == me.team).min_by_key(|a| (d2(me.x, me.y, a.x, a.y), a.id));
                if let Some(a) = a { let _ = sim.entity_add_shield(a.id, 320, 180); }
                a.map(|a| a.id)
            } else { None };
            // round 90: fly to the ally the defense read chose (the save or the fight); the cast's own shield went to
            // the game AI's pick, so the chosen ally gets the same shield
            let target = match (self.ult_ally.filter(|&id| living(all, id).is_some()), target) {
                (Some(chosen), Some(t)) if chosen != t => { let _ = sim.entity_add_shield(chosen, 320, 180); Some(chosen) }
                (Some(chosen), None) => { let _ = sim.entity_add_shield(chosen, 320, 180); Some(chosen) }
                (_, t) => t,
            };
            match target {
                Some(t) => {
                    fx(sim, me.id, "deploy", InputTargetV1::target(me.id));
                    log(sim, &format!("ult.cast.{tick}"), &format!("Wings of Light cast for ally {t}"));
                    self.phase = Some(Phase::Deploy { target: t, start: tick });
                }
                None => log(sim, &format!("ult.none.{tick}"), "Wings of Light cast, but no ally found (no flight)"),
            }
        } else if sim.get_entity(me.id).is_some_and(|e| has(&e, "gdm_charge")) {
            if let Some(enemy) = aim_target(me, all, "gdm_charge_target", false) {
                sim.entity_remove_buff(enemy.id, "gdm_charge_target");
                let dx = (enemy.x - me.x) as f64;
                let dy = (enemy.y - me.y) as f64;
                let len = dx.hypot(dy).max(1.0);
                log(sim, &format!("s2.dash.{tick}"), &format!("Arondight: dash at enemy {}", enemy.id));
                self.phase = Some(Phase::Charge { x: me.x, y: me.y, dx: dx / len, dy: dy / len, start: tick, pushed: None });
            } else {
                sim.entity_remove_buff(me.id, "gdm_charge");
                log(sim, &format!("s2.none.{tick}"), "Arondight cast, but the target marker wasn't found");
            }
        }
    }
}

impl StablePassive for Gundam {
    fn clone_box(&self) -> Box<dyn StablePassive> { Box::new(self.clone()) }

    fn on_spawn(&mut self, sim: &mut StableSim<'_>, _player: usize, entity: usize) {
        self.me = Some(entity);
        self.clean(sim);
    }

    fn on_dead(&mut self, sim: &mut StableSim<'_>, _player: usize) { self.clean(sim); }

    fn on_base_attack(&mut self, sim: &mut StableSim<'_>, _seed: u64, _player: usize, _entity: usize) {
        self.base_until = sim.tick() + BASE_WINDOW;
    }

    fn on_attack(&mut self, sim: &mut StableSim<'_>, _player: usize, entity: usize,
                 target: usize, damage: &mut usize) {
        // only his real basic attacks (skills and the landing also land hits)
        if sim.tick() > self.base_until { return; }
        self.base_until = 0;
        self.hits += 1;
        if sim.tick() < self.stance_until {
            // round 90: Arondight drawn: a great-sword swing that burns and pulls the target a little toward him
            *damage = *damage * 115 / 100;
            fx(sim, entity, "arondight_swing", InputTargetV1::target(target));
            refresh_burn(&mut self.burns, target, sim.tick());
            sim.entity_remove_buff(target, "gdm_burn");
            sim.add_buff(target, &BuffV1::timed("gdm_burn", BURN_EVERY * BURN_HITS));
            let _ = sim.entity_pull(entity, target, PULL_SPEED, PULL_TICKS);
        } else if self.hits.is_multiple_of(PALM_EVERY) {
            *damage = *damage * 125 / 100;
            fx(sim, entity, "palm", InputTargetV1::target(target));
        } else {
            fx(sim, entity, "saber", InputTargetV1::target(target));
        }
    }

    fn on_update(&mut self, sim: &mut StableSim<'_>, _seed: u64, player: usize, entity: usize) {
        self.me = Some(entity);
        if sim.is_end() { self.clean(sim); return; }
        let all = champions(sim);
        let Some(me) = living(&all, entity).cloned() else { self.clean(sim); return };
        let tick = sim.tick();
        if tick < 3 { log(sim, "spawn", &format!("Aegis Zero native {} is running (player {player})", crate::VERSION)); }
        self.passive(sim, &me, &all);
        self.slice(sim, &me, &all);
        self.burn(sim, &me, tick);
        // round 90: the ult is only for saving an ally or joining a teamfight: gdm_ult_ok lets the data cast it
        if tick.is_multiple_of(10) {
            self.ult_ally = if self.phase.is_none() { ult_choice(&me, &all, |id| sim.is_visible(me.team, id)) } else { None };
            sim.entity_remove_buff(me.id, "gdm_ult_ok");
            if self.ult_ally.is_some() { sim.add_buff(me.id, &BuffV1::timed("gdm_ult_ok", 15)); }
        }
        self.step(sim, &me, &all, tick);
        if self.phase.is_none() { self.start(sim, &me, &all, tick); }
        let zero_left = sim.get_entity(me.id).and_then(|e| buff_ticks(&e, "gdm_zero"));
        let deploy_elapsed = match self.phase { Some(Phase::Deploy { start, .. }) => tick.saturating_sub(start), _ => 0 };
        show_wings(sim, me.id, wing_buff(self.phase.as_ref(), zero_left, deploy_elapsed));
        // the presses he'd use: none while the ult (or a charge) is running; round 91: while he rises and flies he can't
        // attack at all (Rian: "he can attack, make him can't"): no press becomes a basic attack, and the engine blocks
        // his attacks and skills
        let free = self.phase.is_none() || matches!(self.phase, Some(Phase::Empowered));
        let hold = holds(self.phase.as_ref());
        if hold && tick.is_multiple_of(10) {
            sim.apply_cc(me.id, &CcV1::of_kind(CcKindV1::BlockAttack, 12));
            sim.apply_cc(me.id, &CcV1::of_kind(CcKindV1::BlockSkill, 12));
        }
        let flags = if free { crate::press::S1 | crate::press::S2 } else if hold { crate::press::HOLD } else { 0 };
        crate::press::note(sim.seed(), player, tick, flags);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn champ(id: usize, team: usize, x: i64) -> Champ {
        Champ { id, team, x, y: 100_000, buffs: vec![], stunned: false, pushed: false, hp: 100, max_hp: 200, attack: 10, name: "c".into() }
    }
    #[test]
    fn landing_blocks_for_ally() {
        let ally = champ(1, 0, 100_000);
        let enemy = champ(2, 1, 130_000);
        assert_eq!(landing_spot(&ally, &[ally.clone(), enemy]), (114_000, 100_000));
    }
    fn at(id: usize, team: usize, x: i64, y: i64, hp: usize) -> Champ {
        Champ { id, team, x, y, buffs: vec![], stunned: false, pushed: false, hp, max_hp: 100, attack: 10, name: "c".into() }
    }

    #[test]
    fn every_visual_name_exists_in_the_data() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../mods/tfm2_custom/champion/tfm2_gundam_aegis_zero.data_champion");
        let Ok(text) = std::fs::read_to_string(&path) else { return };
        let has = |name: &str| text.contains(&format!("\"name\": \"{name}\""));
        for fx in ["ascend", "sky_r", "sky_l", "dive", "landing", "deploy", "retract", "flare", "sweep", "sword_draw",
                   "arondight_swing", "saber", "palm", "charge_hit", "wall_hit", "after_r", "after_l", "slice_wave"] {
            assert!(has(&format!("{ID}_{fx}")), "{fx}");
        }
        for k in 0..ZONE_FRAMES { assert!(has(&format!("{ID}_zone_f{k}"))); }
        for a in 0..16 { assert!(has(&format!("{ID}_slice_cut_a{a}"))); }
        for b in ["gdm_arondight", "gdm_burn", "gdm_sliced", "gdm_slowed", WINGS, FADE] { assert!(has(b), "{b}"); }
        assert!(text.contains("\"buff_name\": \"gdm_ult_ok\""), "the ult is gated by the defense read");
    }

    #[test]
    fn no_attacks_while_rising_or_flying() {
        assert!(holds(Some(&Phase::Deploy { target: 1, start: 0 })));
        assert!(holds(Some(&Phase::Ascend { target: 1, start: 0 })));
        assert!(holds(Some(&Phase::Flight { target: 1, start: 0, from: (0, 0), last: (0, 0) })));
        assert!(!holds(Some(&Phase::Empowered)) && !holds(None) && !holds(Some(&Phase::Retract { start: 0 })));
        assert!(!holds(Some(&Phase::Charge { x: 0, y: 0, dx: 1.0, dy: 0.0, start: 0, pushed: None })));
    }

    #[test]
    fn slice_hits_the_strip_only() {
        let from = (100_000, 100_000);
        let dir = (1.0, 0.0);
        assert!(in_slice(from, dir, (160_000, 105_000)), "along the cut, inside the width");
        assert!(in_slice(from, dir, (100_000, 100_000)), "at his feet");
        assert!(!in_slice(from, dir, (170_000, 100_000)), "past its length");
        assert!(!in_slice(from, dir, (140_000, 115_000)), "beside it");
        assert!(!in_slice(from, dir, (60_000, 100_000)), "behind him");
        let diag = (std::f64::consts::FRAC_1_SQRT_2, std::f64::consts::FRAC_1_SQRT_2);
        assert!(in_slice(from, diag, (130_000, 130_000)));
        assert_eq!(cut_angle((1.0, 0.0)), 0);
        assert_eq!(cut_angle((0.0, 1.0)), 8);
        assert_eq!(cut_angle((-1.0, 0.0)), 0, "a line looks the same both ways");
    }

    #[test]
    fn burn_refreshes_not_stacks() {
        let mut burns = Vec::new();
        refresh_burn(&mut burns, 7, 100);
        refresh_burn(&mut burns, 7, 140);
        refresh_burn(&mut burns, 9, 140);
        assert_eq!(burns.len(), 2);
        assert_eq!(burns.iter().find(|b| b.0 == 7), Some(&(7, 140 + BURN_EVERY, BURN_HITS)));
    }

    #[test]
    fn stance_swings_burn_and_pull() {
        // five seconds of sword, three burns a hit, a short pull: the numbers Rian's brief asks for
        assert_eq!(STANCE_TICKS, 300);
        assert_eq!(BURN_EVERY * BURN_HITS, 90);
        assert!(PULL_SPEED * PULL_TICKS >= 5_000 && PULL_SPEED * PULL_TICKS <= 8_000, "pulls a little");
        assert!(SLICE_LEN > SLICE_W * 4 && SLICED_PCT == 30);
    }

    #[test]
    fn inner_knocks_outer_slows() {
        let c = (500_000, 500_000);
        let enemies = [(1, (500_000, 570_000), true), (2, (600_000, 500_000), true), (3, (500_000, 620_000), true),
                       (4, (510_000, 500_000), false)];
        let (knocked, slowed) = zone_split(c, &enemies);
        assert_eq!(knocked, vec![(1, true), (4, false)]);
        assert_eq!(slowed, vec![2]);   // 3 is outside the outer ring
        // round 93: the inner circle is Gojo's domain / DIO's time stop size, the outer ring 1.5x that
        assert_eq!(KNOCK_R, crate::DOMAIN_R);
        assert_eq!(KNOCK_R, crate::batch2::TS_R);
        assert_eq!(SLOW_R, KNOCK_R * 3 / 2);
    }

    #[test]
    fn banish_ends_on_landing() {
        // re-applied every 10 ticks from the rise: covered without a gap, and never past the landing
        let total = ASCEND_TICKS + FLIGHT_TICKS;
        let mut until = 0;
        for e in (0..total).step_by(10) {
            assert!(until >= e || e == 0, "a gap at {e}");
            until = e + banish_ticks(e, total);
        }
        assert_eq!(until, total);
    }

    #[test]
    fn ult_saves_a_low_ally_under_attack() {
        let me = at(0, 0, 100_000, 100_000, 100);
        let hunted = at(1, 0, 600_000, 100_000, 30);
        let fine = at(2, 0, 300_000, 100_000, 90);
        let hunter = at(5, 1, 650_000, 100_000, 100);
        let all = [me.clone(), hunted, fine, hunter];
        assert_eq!(ult_choice(&me, &all, |_| true), Some(1));
        assert_eq!(ult_choice(&me, &all, |_| false), None, "an enemy he can't see doesn't count");
    }

    #[test]
    fn ult_supports_a_teamfight() {
        let me = at(0, 0, 100_000, 100_000, 100);
        let all = [me.clone(), at(1, 0, 500_000, 100_000, 80), at(2, 0, 540_000, 120_000, 60),
                   at(5, 1, 560_000, 100_000, 100), at(6, 1, 580_000, 130_000, 100)];
        assert!(matches!(ult_choice(&me, &all, |_| true), Some(1 | 2)));
        // he's already in that fight: no flight
        let near = at(0, 0, 520_000, 100_000, 100);
        let mut all2 = all.clone(); all2[0] = near.clone();
        assert_eq!(ult_choice(&near, &all2, |_| true), None);
    }

    #[test]
    fn ult_not_for_farming() {
        let me = at(0, 0, 100_000, 100_000, 100);
        // allies healthy, one enemy near one ally (a lane, not a fight): no flight
        let all = [me.clone(), at(1, 0, 500_000, 100_000, 80), at(5, 1, 560_000, 100_000, 100), at(2, 0, 900_000, 900_000, 100)];
        assert_eq!(ult_choice(&me, &all, |_| true), None);
    }

    #[test]
    fn exactly_one_wing_visual_per_phase() {
        assert_eq!(wing_buff(None, None, 0), None);
        assert_eq!(wing_buff(Some(&Phase::Deploy { target: 1, start: 0 }), None, 2), None);   // the deploy effect plays
        assert_eq!(wing_buff(Some(&Phase::Deploy { target: 1, start: 0 }), None, DEPLOY_TICKS), Some(WINGS));
        let flight = Phase::Flight { target: 1, start: 0, from: (0, 0), last: (0, 0) };
        assert_eq!(wing_buff(Some(&flight), None, 0), None);   // up in the sky: the sky effect carries the wings
        assert_eq!(wing_buff(Some(&Phase::Ascend { target: 1, start: 0 }), None, 0), None);
        assert_eq!(wing_buff(Some(&Phase::Empowered), Some(200), 0), Some(WINGS));
        assert_eq!(wing_buff(Some(&Phase::Empowered), Some(FADE_TICKS), 0), Some(FADE));
        assert_eq!(wing_buff(Some(&Phase::Retract { start: 0 }), None, 0), None);   // the retract effect plays
        assert_eq!(wing_buff(Some(&Phase::Charge { x: 0, y: 0, dx: 1.0, dy: 0.0, start: 0, pushed: None }), None, 0), None);
    }
}
