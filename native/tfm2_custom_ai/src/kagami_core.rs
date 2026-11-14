//! Kagami, the Mirror Priestess: a deterministic core (snapshot in, effects out). `kagami.rs` builds the frame from the
//! sim and applies the effects; the Lab (`native/kagami_lab`) runs this same file.
//!
//!   P1 Reflection: a mirror clone of her (35% of her HP) with no time limit. It follows her, shoots for 40% of her
//!      damage and casts its own Moonlit Mend (50%) and Kindred Charm on teammates near it; never Mirror Step or the
//!      ult. A dead clone comes back at her side 15 s later.
//!   P2 Guardian Angel: dropping below 50% HP knocks back enemies within 50,000 and shields her (15% max HP, 3 s).
//!   P3 Mirror Swap: a hit that would drop her under 10% with her clone alive holds her at 1 HP and swaps her with
//!      the clone; she's sealed 1 s, then revives at 40%.
//!   Every swap takes 0.5 s: it locks on to both sides (nobody is rooted) and trades them wherever they are, then
//!      stuns enemies within 50,000 of the clone's new spot for 1 s. A cancelled swap refunds half the cooldown.
//!   S1 Mirror Step: on a teammate within 120,000 of the clone, they swap; on a point within 120,000 of her, the
//!      clone dashes there and she swaps with it.
//!   S2 Kindred Charm: +25% attack speed, +15% move speed, +10% Attack/Magic Power for 5 s on a teammate.
//!   S3 Moonlit Mend: heal a teammate (or herself) for 80 + 60% Attack.
//!   Ult Last Reflection (automatic, global): while ready every teammate but her is warded; the first one who'd drop
//!      under 10% HP is sealed 2.5 s, then revives at full HP with Kindred Charm, a 20% shield, +30% Defence and CC
//!      immunity for 4 s.
//!
//! Her AI reads the frame and the 13 athlete values shape its thresholds. Nothing of hers hits a tower.
use crate::kagami_data as d;
use serde::{Deserialize, Serialize};

pub const MAP: i64 = 960_000;

// the 13 athlete values in the frame's profile
pub const EGO: usize = 0;
pub const JUDGEMENT: usize = 1;
pub const POSITIONING: usize = 2;
pub const AWARENESS: usize = 11;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Point {
    pub x: i64,
    pub y: i64,
}
impl Point {
    pub const fn new(x: i64, y: i64) -> Self {
        Point { x, y }
    }
    pub fn d2(self, o: Point) -> i128 {
        let (dx, dy) = ((self.x - o.x) as i128, (self.y - o.y) as i128);
        dx * dx + dy * dy
    }
    pub fn dist(self, o: Point) -> f64 {
        (self.d2(o) as f64).sqrt()
    }
    pub fn within(self, o: Point, r: i64) -> bool {
        self.d2(o) <= r as i128 * r as i128
    }
    /// `len` from here toward `to` (or `to` itself when it is closer)
    pub fn toward(self, to: Point, len: i64) -> Point {
        let d = self.dist(to);
        if d <= len as f64 || d < 1.0 {
            return to;
        }
        let k = len as f64 / d;
        Point::new(self.x + ((to.x - self.x) as f64 * k).round() as i64, self.y + ((to.y - self.y) as f64 * k).round() as i64)
    }
    pub fn clamp_map(self) -> Point {
        Point::new(self.x.clamp(1_000, MAP - 1_000), self.y.clamp(1_000, MAP - 1_000))
    }
    pub fn on_map(self) -> bool {
        (0..=MAP).contains(&self.x) && (0..=MAP).contains(&self.y)
    }
}

fn t(n: i64) -> u64 {
    n.max(0) as u64
}
fn pct_of(v: i64, p: i64) -> i64 {
    v * p / 100
}

// ------------------------------------------------------------------ frame

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Unit {
    pub id: usize,
    pub team: usize,
    pub pos: Point,
    pub hp: i64,
    pub max_hp: i64,
    #[serde(default)]
    pub attack: i64,
    pub champion: bool,
    pub tower: bool,
    pub visible: bool,
    pub targetable: bool,
    /// sealed by any Kagami's ult or Mirror Swap (`kg_sealed`)
    #[serde(default)]
    pub sealed: bool,
}

/// Her clone as the bridge sees it (alive, in the world)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CloneView {
    pub id: usize,
    pub pos: Point,
    pub hp: i64,
    pub max_hp: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "aim", rename_all = "snake_case")]
pub enum Order {
    /// S1 on a teammate: the teammate and the clone swap
    StepAlly(usize),
    /// S1 on a point: the clone dashes there, then she swaps with it
    StepPoint(Point),
    /// S2 on a teammate (or herself)
    Charm(usize),
    /// S3 on a teammate (or herself)
    Mend(usize),
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Frame {
    pub tick: u64,
    pub owner: usize,
    pub team: usize,
    pub alive: bool,
    pub pos: Point,
    pub hp: i64,
    pub max_hp: i64,
    pub attack: i64,
    #[serde(default)]
    pub stunned: bool,
    pub ult_unlocked: bool,
    #[serde(default)]
    pub home: Point,
    pub units: Vec<Unit>,
    /// her clone, when it is alive in the world
    #[serde(default)]
    pub clone: Option<CloneView>,
    pub profile: [i64; 13],
    pub manual: bool,
    pub order: Option<Order>,
}
impl Frame {
    pub fn enemy(&self, u: &Unit) -> bool {
        u.team != self.team && u.visible && u.targetable && u.hp > 0
    }
    pub fn enemy_champions(&self) -> impl Iterator<Item = &Unit> {
        self.units.iter().filter(move |u| u.champion && self.enemy(u))
    }
    /// her teammates (never herself, never the clone)
    pub fn allies(&self) -> impl Iterator<Item = &Unit> {
        self.units.iter().filter(move |u| u.champion && u.team == self.team && u.hp > 0 && u.id != self.owner)
    }
    pub fn p(&self, i: usize) -> i64 {
        self.profile.get(i).copied().unwrap_or(50).clamp(0, 100)
    }
    pub fn unit(&self, id: usize) -> Option<&Unit> {
        self.units.iter().find(|u| u.id == id)
    }
    /// where `id` stands now: herself, her clone or a unit
    pub fn pos_of(&self, id: usize) -> Option<Point> {
        if id == self.owner {
            return Some(self.pos);
        }
        if let Some(c) = self.clone.filter(|c| c.id == id) {
            return Some(c.pos);
        }
        self.unit(id).filter(|u| u.hp > 0).map(|u| u.pos)
    }
    /// how many enemy champions are within `r` of `p`
    pub fn enemies_near(&self, p: Point, r: i64) -> usize {
        self.enemy_champions().filter(|u| u.pos.within(p, r)).count()
    }
    pub fn me_as_unit(&self) -> Unit {
        Unit {
            id: self.owner,
            team: self.team,
            pos: self.pos,
            hp: self.hp,
            max_hp: self.max_hp,
            attack: self.attack,
            champion: true,
            tower: false,
            visible: true,
            targetable: true,
            sealed: false,
        }
    }
}

// ------------------------------------------------------------------ effects

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Effect {
    /// a new clone at this spot with this HP
    SpawnClone { at: Point, hp: i64 },
    /// the clone glides to `to` over `ticks`
    CloneMove { to: Point, ticks: u64 },
    /// the clone's light shard: damage from her (as the clone's), never a tower
    CloneShot { target: usize, amount: i64, from: Point },
    /// a heal on `target` (`from` draws the light)
    Heal { target: usize, amount: i64, from: Point },
    /// Kindred Charm on `target` for `ticks`
    Charm { target: usize, ticks: u64 },
    /// a swap has locked on to these two (draw the thread)
    SwapLock { a: usize, b: usize, ticks: u64 },
    /// trade the places of `a` and `b` (read where they are now)
    Swap { a: usize, b: usize },
    Stun { target: usize, ticks: u64 },
    Knockback { target: usize },
    Shield { target: usize, amount: i64, ticks: u64 },
    /// the ult's ward on a teammate (an `undying` buff), renewed each tick it lasts
    Ward { target: usize, ticks: u64 },
    /// her own Mirror Swap ward (an `undying` buff), renewed each tick it lasts
    SelfWard { ticks: u64 },
    /// sealed: untargetable, unseen, unable to act
    Seal { target: usize, ticks: u64 },
    /// back to `hp` (never lowered); `blessed` adds the ult's Charm, shield, Defence and CC immunity
    Revive { target: usize, hp: i64, blessed: bool },
    /// the AI's walk
    Steer { to: Point },
    Fx { name: String, at: Point, ticks: u64 },
    Log { text: String },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SwapKind {
    /// Mirror Step on a teammate: the clone and the teammate
    StepAlly,
    /// Mirror Step on a point: the clone dashes, then she and the clone
    StepPoint,
    /// Mirror Swap: she and the clone, then her seal
    SelfSave,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PendingSwap {
    pub kind: SwapKind,
    /// the other side (the clone is always one side)
    pub other: usize,
    pub clone: usize,
    pub started: u64,
    pub at: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Seal {
    pub id: usize,
    pub until: u64,
    /// the ult's full-HP revive (true) or her own 40% (false)
    pub ult: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Core {
    pub clone: Option<usize>,
    /// the clone may come back at this tick (when there is none)
    pub clone_back_at: u64,
    pub clone_shot_at: u64,
    pub clone_mend_at: u64,
    pub clone_charm_at: u64,
    /// the clone is dashing for Mirror Step until this tick
    pub clone_dash_until: u64,
    /// the clone holds its spot (after being swapped into a fight) until this tick
    pub clone_hold_until: u64,
    pub clone_moved_at: u64,
    pub step_at: u64,
    pub charm_at: u64,
    pub mend_at: u64,
    pub angel_at: u64,
    pub self_save_at: u64,
    pub ult_at: u64,
    pub swap: Option<PendingSwap>,
    pub seals: Vec<Seal>,
    /// (teammate, until): Kindred Charm already on them
    pub charmed: Vec<(usize, u64)>,
    /// last tick's HP of every champion on her team (her included): only a drop triggers a save
    pub last_hp: Vec<(usize, i64)>,
    pub hp_log: Vec<(u64, i64)>,
    pub steer_at: u64,
    #[serde(default)]
    pub rejection: String,
}

impl Default for Core {
    fn default() -> Self {
        Core {
            clone: None,
            clone_back_at: 0,
            clone_shot_at: 0,
            clone_mend_at: 0,
            clone_charm_at: 0,
            clone_dash_until: 0,
            clone_hold_until: 0,
            clone_moved_at: 0,
            step_at: 0,
            charm_at: 0,
            mend_at: 0,
            angel_at: 0,
            self_save_at: 0,
            ult_at: 0,
            swap: None,
            seals: vec![],
            charmed: vec![],
            last_hp: vec![],
            hp_log: vec![],
            steer_at: 0,
            rejection: String::new(),
        }
    }
}

fn travel(from: Point, to: Point, speed: i64) -> u64 {
    ((from.dist(to) / speed.max(1) as f64).ceil() as u64).max(1)
}

impl Core {
    /// She came back from death: her clone comes back with her (a fresh one at once), timers keep running.
    pub fn respawn(&mut self, tick: u64) {
        self.clone_back_at = tick;
        self.swap = None;
        self.seals.retain(|s| s.ult);
        self.hp_log.clear();
        self.last_hp.clear();
    }
    pub fn clone_alive(&self, f: &Frame) -> bool {
        self.clone.is_some() && f.clone.is_some_and(|c| Some(c.id) == self.clone && c.hp > 0)
    }
    pub fn sealed(&self, id: usize) -> bool {
        self.seals.iter().any(|s| s.id == id)
    }
    pub fn ult_ready(&self, f: &Frame) -> bool {
        f.ult_unlocked && f.tick >= self.ult_at
    }
    pub fn self_save_ready(&self, f: &Frame) -> bool {
        f.tick >= self.self_save_at && self.clone_alive(f)
    }
    fn charmed(&self, id: usize, tick: u64) -> bool {
        self.charmed.iter().any(|c| c.0 == id && tick < c.1)
    }
    fn last_hp_of(&self, id: usize) -> Option<i64> {
        self.last_hp.iter().find(|h| h.0 == id).map(|h| h.1)
    }

    pub fn step(&mut self, f: &Frame) -> Vec<Effect> {
        let mut out = vec![];
        self.rejection.clear();
        // her clone as the world has it (a dead or removed clone starts the respawn timer)
        if let Some(id) = self.clone {
            if !f.clone.is_some_and(|c| c.id == id && c.hp > 0) {
                self.clone = None;
                self.clone_back_at = f.tick + t(d::CLONE_RESPAWN_TICKS);
                if self.swap.is_some_and(|s| s.clone == id) {
                    self.cancel_swap(f, &mut out);
                }
            }
        } else if let Some(c) = f.clone.filter(|c| c.hp > 0) {
            self.clone = Some(c.id);
        }
        self.charmed.retain(|c| f.tick < c.1);
        self.run_seals(f, &mut out);
        if !f.alive {
            // her own seal ends with her; the ult's keep their revive (the bridge applies them anyway)
            self.swap = None;
            self.hp_log.clear();
            self.note_hp(f);
            return out;
        }
        self.hp_log.retain(|h| f.tick < h.0 + t(d::AI_HURT_TICKS));
        self.hp_log.push((f.tick, f.hp));

        self.run_ult(f, &mut out);
        self.run_self_save(f, &mut out);
        self.run_angel(f, &mut out);
        self.run_swap(f, &mut out);
        self.run_clone(f, &mut out);

        let order = if f.manual { f.order } else { self.decide(f) };
        if let Some(o) = order {
            if !f.stunned && !self.sealed(f.owner) {
                self.execute(f, o, &mut out);
            }
        }
        if !f.manual && !self.sealed(f.owner) && self.swap.is_none_or_ally() {
            if let Some(to) = self.steer(f) {
                out.push(Effect::Steer { to });
            }
        }
        self.note_hp(f);
        out
    }

    fn note_hp(&mut self, f: &Frame) {
        self.last_hp = f.allies().map(|u| (u.id, u.hp)).collect();
        if f.alive {
            self.last_hp.push((f.owner, f.hp));
        }
    }

    // -------------------------------------------------------------- seals and saves

    fn run_seals(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        let due: Vec<Seal> = self.seals.iter().filter(|s| f.tick >= s.until).copied().collect();
        self.seals.retain(|s| f.tick < s.until);
        for s in due {
            let max = if s.id == f.owner { f.max_hp } else { f.unit(s.id).map_or(0, |u| u.max_hp) };
            if max <= 0 {
                continue;
            }
            if s.ult {
                out.push(Effect::Revive { target: s.id, hp: max, blessed: true });
            } else {
                out.push(Effect::Revive { target: s.id, hp: pct_of(max, d::SELF_SAVE_REVIVE_PCT).max(1), blessed: false });
            }
            out.push(Effect::Fx { name: "kg_revive".into(), at: f.pos_of(s.id).unwrap_or(f.pos), ticks: 30 });
        }
        // a sealed teammate stays held at 1 HP until the revive
        for s in &self.seals {
            if s.ult {
                out.push(Effect::Ward { target: s.id, ticks: 3 });
            } else {
                out.push(Effect::SelfWard { ticks: 3 });
            }
        }
    }

    /// Last Reflection: wards every teammate (not her) while ready; the first one who drops under the line is sealed.
    fn run_ult(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        if !self.ult_ready(f) {
            return;
        }
        let line = |u: &Unit| u.hp * 100 <= u.max_hp * d::ULT_TRIGGER_PCT;
        let hit = f
            .allies()
            .filter(|u| !u.sealed && !self.sealed(u.id) && line(u) && self.last_hp_of(u.id).is_some_and(|h| u.hp < h))
            .min_by_key(|u| (u.hp * 1000 / u.max_hp.max(1), u.id))
            .map(|u| (u.id, u.pos));
        if let Some((id, at)) = hit {
            self.ult_at = f.tick + t(d::ULT_COOLDOWN_TICKS);
            self.seals.push(Seal { id, until: f.tick + t(d::ULT_SEAL_TICKS), ult: true });
            out.push(Effect::Seal { target: id, ticks: t(d::ULT_SEAL_TICKS) });
            out.push(Effect::Ward { target: id, ticks: 3 });
            out.push(Effect::Fx { name: "kg_seal".into(), at, ticks: t(d::ULT_SEAL_TICKS) });
            out.push(Effect::Log { text: format!("Last Reflection seals {id}") });
            return;
        }
        for u in f.allies().filter(|u| !u.sealed) {
            out.push(Effect::Ward { target: u.id, ticks: 3 });
        }
    }

    /// Mirror Swap: held at 1 HP, swap-locked with the clone, then sealed and revived.
    fn run_self_save(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        if self.swap.is_some_and(|s| s.kind == SwapKind::SelfSave) {
            out.push(Effect::SelfWard { ticks: 3 });
            return;
        }
        if !self.self_save_ready(f) || self.sealed(f.owner) {
            return;
        }
        let dropped = self.last_hp_of(f.owner).is_some_and(|h| f.hp < h);
        if f.hp * 100 <= f.max_hp * d::SELF_SAVE_TRIGGER_PCT && dropped {
            let clone = self.clone.unwrap_or_default();
            // her own save outranks a Mirror Step in flight
            self.swap = None;
            self.clone_dash_until = 0;
            self.self_save_at = f.tick + t(d::SELF_SAVE_COOLDOWN_TICKS);
            self.lock(f, SwapKind::SelfSave, clone, f.owner, f.tick + t(d::SWAP_LOCK_TICKS), out);
            out.push(Effect::SelfWard { ticks: 3 });
            out.push(Effect::Log { text: "Mirror Swap".into() });
        } else {
            out.push(Effect::SelfWard { ticks: 3 });
        }
    }

    /// Guardian Angel: crossing below the line knocks enemies away and shields her.
    fn run_angel(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        if f.tick < self.angel_at {
            return;
        }
        let line = |hp: i64| hp * 100 < f.max_hp * d::ANGEL_THRESHOLD_PCT;
        if !line(f.hp) || self.last_hp_of(f.owner).is_none_or(line) {
            return;
        }
        self.angel_at = f.tick + t(d::ANGEL_COOLDOWN_TICKS);
        for u in f.units.iter().filter(|u| f.enemy(u) && !u.tower && u.pos.within(f.pos, d::ANGEL_RADIUS)) {
            out.push(Effect::Knockback { target: u.id });
        }
        out.push(Effect::Shield { target: f.owner, amount: pct_of(f.max_hp, d::ANGEL_SHIELD_PCT_MAX_HP), ticks: t(d::ANGEL_SHIELD_TICKS) });
        out.push(Effect::Fx { name: "kg_angel".into(), at: f.pos, ticks: 36 });
    }

    // -------------------------------------------------------------- swaps

    fn lock(&mut self, f: &Frame, kind: SwapKind, clone: usize, other: usize, at: u64, out: &mut Vec<Effect>) {
        self.swap = Some(PendingSwap { kind, other, clone, started: f.tick, at });
        out.push(Effect::SwapLock { a: clone, b: other, ticks: at.saturating_sub(f.tick) });
    }

    fn cancel_swap(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        if let Some(s) = self.swap.take() {
            let cd = t(d::STEP_COOLDOWN_TICKS);
            if matches!(s.kind, SwapKind::StepAlly | SwapKind::StepPoint) {
                self.step_at = self.step_at.saturating_sub(cd * t(d::SWAP_REFUND_PCT) / 100).max(f.tick);
            }
            self.clone_dash_until = 0;
            out.push(Effect::Log { text: "swap cancelled".into() });
        }
    }

    fn run_swap(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        let Some(s) = self.swap else { return };
        let clone = f.clone.filter(|c| c.id == s.clone && c.hp > 0);
        let other_ok = if s.other == f.owner {
            f.alive
        } else {
            f.unit(s.other).is_some_and(|u| u.hp > 0 && u.targetable && !u.sealed && !self.sealed(u.id))
        };
        let (Some(c), true) = (clone, other_ok) else {
            self.cancel_swap(f, out);
            return;
        };
        if f.tick < s.at {
            return;
        }
        let Some(other_pos) = f.pos_of(s.other) else {
            self.cancel_swap(f, out);
            return;
        };
        self.swap = None;
        self.clone_dash_until = 0;
        out.push(Effect::Swap { a: c.id, b: s.other });
        out.push(Effect::Fx { name: "kg_swap".into(), at: c.pos, ticks: 24 });
        out.push(Effect::Fx { name: "kg_swap".into(), at: other_pos, ticks: 24 });
        // the clone lands where the other side stood: the stun goes off there
        let stun = t(d::SWAP_STUN_TICKS);
        for u in f.units.iter().filter(|u| f.enemy(u) && !u.tower && u.pos.within(other_pos, d::SWAP_STUN_RADIUS)) {
            out.push(Effect::Stun { target: u.id, ticks: stun });
        }
        out.push(Effect::Fx { name: "kg_stun_burst".into(), at: other_pos, ticks: 30 });
        // after a swap into a fight the clone keeps its new spot a while
        self.clone_hold_until = f.tick + 180;
        if s.kind == SwapKind::SelfSave {
            let ticks = t(d::SELF_SAVE_SEAL_TICKS);
            self.seals.push(Seal { id: f.owner, until: f.tick + ticks, ult: false });
            out.push(Effect::Seal { target: f.owner, ticks });
            out.push(Effect::SelfWard { ticks: 3 });
        }
    }

    // -------------------------------------------------------------- the clone

    fn run_clone(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        let Some(c) = f.clone.filter(|c| Some(c.id) == self.clone && c.hp > 0) else {
            if self.clone.is_none() && f.alive && f.tick >= self.clone_back_at {
                // the bridge reports the new id next frame; until then no second one is asked for
                self.clone_back_at = f.tick + 30;
                out.push(Effect::SpawnClone { at: f.pos, hp: pct_of(f.max_hp, d::CLONE_HP_PCT).max(1) });
                out.push(Effect::Fx { name: "kg_clone_in".into(), at: f.pos, ticks: 24 });
            }
            return;
        };
        let mine = c.pos;
        // its shard: champions first, never a tower
        if f.tick >= self.clone_shot_at {
            let target = f
                .units
                .iter()
                .filter(|u| f.enemy(u) && !u.tower && u.pos.within(mine, d::CLONE_ATTACK_RANGE))
                .min_by_key(|u| (!u.champion, u.pos.d2(mine), u.id))
                .filter(|u| u.champion || f.enemies_near(f.pos, d::AI_FIGHT_RANGE) == 0 || u.pos.within(f.pos, d::CLONE_ATTACK_RANGE));
            if let Some(u) = target {
                self.clone_shot_at = f.tick + t(d::CLONE_ATTACK_TICKS);
                out.push(Effect::CloneShot { target: u.id, amount: pct_of(f.attack, d::CLONE_DAMAGE_PCT).max(1), from: mine });
            }
        }
        // its own Mend: the most hurt teammate near it (her included)
        if f.tick >= self.clone_mend_at {
            let mut team: Vec<Unit> = f.allies().filter(|u| !u.sealed).cloned().collect();
            if f.alive && !self.sealed(f.owner) {
                team.push(f.me_as_unit());
            }
            if let Some(u) = team
                .iter()
                .filter(|u| u.pos.within(mine, d::MEND_RANGE) && u.hp * 100 < u.max_hp * d::AI_HEAL_BELOW_PCT)
                .min_by_key(|u| (u.hp * 1000 / u.max_hp.max(1), u.id))
            {
                self.clone_mend_at = f.tick + t(d::CLONE_MEND_COOLDOWN_TICKS);
                let amount = pct_of(self.mend_amount(f), d::CLONE_MEND_PCT);
                out.push(Effect::Heal { target: u.id, amount, from: mine });
            }
        }
        // its own Charm: a teammate near it who is fighting
        if f.tick >= self.clone_charm_at {
            if let Some(id) = self.charm_pick(f, mine) {
                self.clone_charm_at = f.tick + t(d::CLONE_CHARM_COOLDOWN_TICKS);
                self.charmed.push((id, f.tick + t(d::CHARM_TICKS)));
                out.push(Effect::Charm { target: id, ticks: t(d::CHARM_TICKS) });
                out.push(Effect::Fx { name: "kg_charm".into(), at: f.pos_of(id).unwrap_or(mine), ticks: 24 });
            }
        }
        // movement: never while a swap holds it (the lock follows ids, so it may still be pushed by others)
        if self.swap.is_some() || f.tick < self.clone_dash_until || !f.alive || f.tick < self.clone_moved_at + 6 {
            return;
        }
        let far = !mine.within(f.pos, d::CLONE_RANGE - 10_000);
        let shooting = f.units.iter().any(|u| f.enemy(u) && !u.tower && u.champion && u.pos.within(mine, d::CLONE_ATTACK_RANGE));
        if !far && (f.tick < self.clone_hold_until && shooting) {
            return;
        }
        let spot = self.clone_spot(f);
        if far || (!mine.within(spot, d::CLONE_FOLLOW_GAP / 2) && !(shooting && mine.within(f.pos, d::CLONE_RANGE / 2))) {
            let step = mine.toward(spot, d::CLONE_MOVE_SPEED * 6);
            if step != mine {
                self.clone_moved_at = f.tick;
                out.push(Effect::CloneMove { to: step, ticks: 6 });
            }
        }
    }

    /// Where the clone stands when calm: a step behind her, away from the nearest enemy champion (or toward home).
    pub fn clone_spot(&self, f: &Frame) -> Point {
        let away = f
            .enemy_champions()
            .min_by_key(|u| u.pos.d2(f.pos))
            .filter(|u| u.pos.within(f.pos, d::AI_ESCAPE_RANGE))
            .map(|u| u.pos)
            .unwrap_or_else(|| Point::new(2 * f.pos.x - f.home.x, 2 * f.pos.y - f.home.y));
        let d = f.pos.dist(away).max(1.0);
        let k = d::CLONE_FOLLOW_GAP as f64 / d;
        Point::new(f.pos.x + ((f.pos.x - away.x) as f64 * k) as i64, f.pos.y + ((f.pos.y - away.y) as f64 * k) as i64).clamp_map()
    }

    // -------------------------------------------------------------- casts

    pub fn mend_amount(&self, f: &Frame) -> i64 {
        d::MEND_BASE + d::MEND_AD_RATIO * f.attack / 100
    }

    /// a teammate near `from` who is fighting and isn't charmed yet (the strongest hitter first)
    fn charm_pick(&self, f: &Frame, from: Point) -> Option<usize> {
        f.allies()
            .filter(|u| !u.sealed && !self.charmed(u.id, f.tick) && u.pos.within(from, d::CHARM_RANGE))
            .filter(|u| f.enemies_near(u.pos, d::AI_FIGHT_RANGE) > 0)
            .max_by_key(|u| (u.attack, std::cmp::Reverse(u.id)))
            .map(|u| u.id)
    }

    pub fn execute(&mut self, f: &Frame, o: Order, out: &mut Vec<Effect>) {
        match o {
            Order::StepAlly(id) => {
                let Some(c) = f.clone.filter(|c| Some(c.id) == self.clone && c.hp > 0) else { return self.reject("no clone") };
                let ok = f.unit(id).is_some_and(|u| u.team == f.team && u.champion && u.hp > 0 && u.targetable && !u.sealed && u.pos.within(c.pos, d::STEP_RANGE));
                if f.tick < self.step_at || self.swap.is_some() || !ok || id == f.owner {
                    return self.reject("step");
                }
                self.step_at = f.tick + t(d::STEP_COOLDOWN_TICKS);
                self.lock(f, SwapKind::StepAlly, c.id, id, f.tick + t(d::SWAP_LOCK_TICKS), out);
            }
            Order::StepPoint(p) => {
                let Some(c) = f.clone.filter(|c| Some(c.id) == self.clone && c.hp > 0) else { return self.reject("no clone") };
                if f.tick < self.step_at || self.swap.is_some() || !p.within(f.pos, d::STEP_RANGE) || !p.on_map() {
                    return self.reject("step");
                }
                self.step_at = f.tick + t(d::STEP_COOLDOWN_TICKS);
                let dash = if c.pos.within(p, 2_000) { 0 } else { travel(c.pos, p, d::CLONE_DASH_SPEED) };
                if dash > 0 {
                    self.clone_dash_until = f.tick + dash;
                    out.push(Effect::CloneMove { to: p, ticks: dash });
                }
                self.lock(f, SwapKind::StepPoint, c.id, f.owner, f.tick + dash + t(d::SWAP_LOCK_TICKS), out);
            }
            Order::Charm(id) => {
                let target = if id == f.owner { Some(f.me_as_unit()) } else { f.unit(id).filter(|u| u.team == f.team && u.champion).cloned() };
                let Some(u) = target.filter(|u| u.hp > 0 && !u.sealed && u.pos.within(f.pos, d::CHARM_RANGE)) else { return self.reject("charm") };
                if f.tick < self.charm_at {
                    return self.reject("charm");
                }
                self.charm_at = f.tick + t(d::CHARM_COOLDOWN_TICKS);
                self.charmed.retain(|c| c.0 != u.id);
                self.charmed.push((u.id, f.tick + t(d::CHARM_TICKS)));
                out.push(Effect::Charm { target: u.id, ticks: t(d::CHARM_TICKS) });
                out.push(Effect::Fx { name: "kg_charm".into(), at: u.pos, ticks: 24 });
            }
            Order::Mend(id) => {
                let target = if id == f.owner { Some(f.me_as_unit()) } else { f.unit(id).filter(|u| u.team == f.team && u.champion).cloned() };
                let Some(u) = target.filter(|u| u.hp > 0 && !u.sealed && u.pos.within(f.pos, d::MEND_RANGE)) else { return self.reject("mend") };
                if f.tick < self.mend_at {
                    return self.reject("mend");
                }
                self.mend_at = f.tick + t(d::MEND_COOLDOWN_TICKS);
                out.push(Effect::Heal { target: u.id, amount: self.mend_amount(f), from: f.pos });
            }
        }
    }

    fn reject(&mut self, why: &str) {
        self.rejection = why.into();
    }

    // -------------------------------------------------------------- AI

    pub fn hp_pct(u: &Unit) -> i64 {
        u.hp * 100 / u.max_hp.max(1)
    }

    /// enemy champions near a spot, weighted by closeness
    pub fn danger_at(f: &Frame, p: Point) -> i64 {
        f.enemy_champions().filter(|u| u.pos.within(p, d::AI_ESCAPE_RANGE)).map(|u| (d::AI_ESCAPE_RANGE as f64 - u.pos.dist(p)).max(0.0) as i64).sum()
    }

    /// she lost a real share of her HP lately
    pub fn hurt(&self, f: &Frame) -> bool {
        let top = self.hp_log.iter().map(|h| h.1).max().unwrap_or(f.hp);
        (top - f.hp) * 100 >= f.max_hp * d::AI_HURT_PCT
    }

    /// The teammate to rescue with Mirror Step: low, enemies on them, and the clone's spot is safer than theirs.
    pub fn rescue_target(&self, f: &Frame) -> Option<usize> {
        let c = f.clone.filter(|c| Some(c.id) == self.clone && c.hp > 0)?;
        // judgement moves the line by up to 10 points either way
        let line = d::AI_DANGER_HP_PCT + (f.p(JUDGEMENT) - 50) / 5;
        f.allies()
            .filter(|u| !u.sealed && u.targetable && u.pos.within(c.pos, d::STEP_RANGE))
            .filter(|u| Self::hp_pct(u) <= line && f.enemies_near(u.pos, d::AI_DANGER_RANGE) > 0)
            .filter(|u| !u.pos.within(c.pos, 20_000) && Self::danger_at(f, c.pos) < Self::danger_at(f, u.pos))
            .min_by_key(|u| (Self::hp_pct(u), u.id))
            .map(|u| u.id)
    }

    /// Her own escape: dived and hurt, the clone (or a point toward home) is the way out.
    pub fn escape_point(&self, f: &Frame) -> Option<Point> {
        let c = f.clone.filter(|c| Some(c.id) == self.clone && c.hp > 0)?;
        let dived = f.enemies_near(f.pos, d::AI_DIVED_RANGE) > 0;
        if !dived || !(self.hurt(f) || f.hp * 100 < f.max_hp * 50) {
            return None;
        }
        let here = Self::danger_at(f, f.pos);
        if c.pos.within(f.pos, d::STEP_RANGE) && !c.pos.within(f.pos, 30_000) && Self::danger_at(f, c.pos) * 2 < here {
            return Some(c.pos);
        }
        let home = f.pos.toward(f.home, d::STEP_RANGE - 5_000).clamp_map();
        (Self::danger_at(f, home) * 2 < here).then_some(home)
    }

    /// The teammate she keeps close to: the strongest hitter within reach.
    pub fn guard_target(&self, f: &Frame) -> Option<Unit> {
        f.allies().filter(|u| !u.sealed && u.pos.within(f.pos, 160_000)).max_by_key(|u| (u.attack, std::cmp::Reverse(u.id))).cloned()
    }

    pub fn decide(&mut self, f: &Frame) -> Option<Order> {
        if self.sealed(f.owner) || f.stunned || self.swap.is_some_and(|s| s.kind != SwapKind::StepAlly) {
            return None;
        }
        // 1. heal the lowest teammate (her included)
        if f.tick >= self.mend_at {
            let mut team: Vec<Unit> = f.allies().filter(|u| !u.sealed).cloned().collect();
            team.push(f.me_as_unit());
            if let Some(u) = team
                .iter()
                .filter(|u| u.pos.within(f.pos, d::MEND_RANGE) && Self::hp_pct(u) < d::AI_HEAL_BELOW_PCT)
                .min_by_key(|u| (Self::hp_pct(u), u.id))
            {
                return Some(Order::Mend(u.id));
            }
        }
        if f.tick >= self.step_at && self.swap.is_none() {
            // 2. rescue a dived teammate
            if let Some(id) = self.rescue_target(f) {
                return Some(Order::StepAlly(id));
            }
            // 3. her own way out
            if let Some(p) = self.escape_point(f) {
                return Some(Order::StepPoint(p));
            }
        }
        // 4. charm the hitter who is fighting
        if f.tick >= self.charm_at {
            if let Some(id) = self.charm_pick(f, f.pos) {
                return Some(Order::Charm(id));
            }
        }
        None
    }

    /// Her walk: stay near the teammate she guards (the engine handles her basics and the rest).
    pub fn steer(&mut self, f: &Frame) -> Option<Point> {
        if f.tick < self.steer_at {
            return None;
        }
        let g = self.guard_target(f)?;
        if g.pos.within(f.pos, d::AI_GUARD_RANGE) {
            return None;
        }
        self.steer_at = f.tick + 6;
        // a step behind them, on her side of the fight
        let toward_me = g.pos.toward(f.pos, d::AI_GUARD_RANGE / 2);
        Some(toward_me.clamp_map())
    }
}

trait SwapIdle {
    fn is_none_or_ally(&self) -> bool;
}
impl SwapIdle for Option<PendingSwap> {
    /// her walk is free unless she is a side of a swap (Mirror Step on a point, Mirror Swap)
    fn is_none_or_ally(&self) -> bool {
        self.is_none_or(|s| s.kind == SwapKind::StepAlly)
    }
}
