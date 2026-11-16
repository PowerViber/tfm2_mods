//! Kagami, the Mirror Priestess: a deterministic core (snapshot in, effects out). `kagami.rs` builds the frame from the
//! sim and applies the effects; the Lab (`native/kagami_lab`) runs this same file.
//!
//! `kagami-2.0` (round 175, "she is too weak"):
//!   P1 Reflection: three mirror clones (25% of her HP each) with no time limit. Each one guards a champion on her team
//!      (her included), picked automatically by danger; several can guard the same champion, with no range limit (a far
//!      clone catches up at dash speed). Each shoots for 35% of her damage and casts its own Moonlit Mend (50%) and
//!      Kindred Charm. A lost clone comes back at her side 12 s later, each on its own timer.
//!   P2 Clone Save: a clone is its champion's save point. A hit that would drop them under 10% holds them at 1 HP and
//!      swap-locks them with that clone; then they trade places, the champion gets back to 30% HP with a 20% shield,
//!      and the clone is spent.
//!   P3 Guardian Angel (2 charges, 30 s each): any champion on her team crossing below 50% HP knocks back enemies
//!      around them and gets a 15% shield; the same champion only once every 30 s.
//!   Every swap takes 0.5 s: it locks on to both sides (nobody is rooted) and trades them wherever they are, then
//!      stuns enemies within 50,000 of the clone's new spot for 2 s. A cancelled swap refunds half of its own cooldown.
//!   S1 Mirror Step (two cooldowns): a teammate anywhere swaps with her safest clone; on a point within 120,000 of her,
//!      her nearest clone dashes there and she swaps with it.
//!   S2 Kindred Charm: +25% attack speed, +15% move speed, +10% Attack/Magic Power for 5 s on a teammate.
//!   S3 Moonlit Mend: heal a teammate (or herself) for 80 + 60% Attack.
//!   Ult Last Reflection (automatic, global, the backup save): while ready every teammate but her with no clone save
//!      ready is warded; the first one who'd drop under 10% HP is sealed 2.5 s, then revives at full HP with Kindred
//!      Charm, a 20% shield, +30% Defence and CC immunity for 4 s, and enemies around are stunned for 2 s.
//!
//! Her AI reads the frame and the 13 athlete values shape its thresholds. Nothing of hers hits a tower.
use crate::kagami_data as d;
use serde::{Deserialize, Serialize};

pub const MAP: i64 = 960_000;

// the 13 athlete values in the frame's profile
pub const EGO: usize = 0;
pub const JUDGEMENT: usize = 1;
pub const POSITIONING: usize = 2;
pub const CONTROL_SPEED: usize = 3;
pub const ACCURACY: usize = 4;
pub const DODGE: usize = 5;
pub const AGGRESSIVE: usize = 6;
pub const ORDER: usize = 7;
pub const ROAMING: usize = 8;
pub const MACRO: usize = 9;
pub const MICRO: usize = 10;
pub const AWARENESS: usize = 11;
pub const STRATEGY: usize = 12;

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
        Point::new(
            self.x + ((to.x - self.x) as f64 * k).round() as i64,
            self.y + ((to.y - self.y) as f64 * k).round() as i64,
        )
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
    /// sealed by any Kagami's ult (`kg_sealed`)
    #[serde(default)]
    pub sealed: bool,
    /// a minion or jungle monster her farming may work on (the shared farming rules say so, and it's calm)
    #[serde(default)]
    pub farmable: bool,
}

/// One of her clones as the bridge sees it (alive, in the world), with the slot it was spawned for
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CloneView {
    #[serde(default)]
    pub slot: usize,
    pub id: usize,
    pub pos: Point,
    pub hp: i64,
    pub max_hp: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "aim", rename_all = "snake_case")]
pub enum Order {
    /// S1 on a teammate: the teammate and her safest clone swap
    StepAlly(usize),
    /// S1 on a point: her nearest clone dashes there, then she swaps with it
    StepPoint(Point),
    /// S2 on a teammate (or herself)
    Charm(usize),
    /// S3 on a teammate (or herself)
    Mend(usize),
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Hazard {
    pub id: usize,
    pub at: Point,
    pub radius: i64,
    pub until: u64,
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
    /// her clones alive in the world
    #[serde(default)]
    pub clones: Vec<CloneView>,
    pub profile: [i64; 13],
    pub manual: bool,
    pub order: Option<Order>,
    #[serde(default)]
    pub terrain: Vec<bool>,
    #[serde(default)]
    pub hazards: Vec<Hazard>,
}
impl Frame {
    pub fn enemy(&self, u: &Unit) -> bool {
        u.team != self.team && u.visible && u.targetable && u.hp > 0
    }
    pub fn enemy_champions(&self) -> impl Iterator<Item = &Unit> {
        self.units.iter().filter(move |u| u.champion && self.enemy(u))
    }
    /// her teammates (never herself, never a clone)
    pub fn allies(&self) -> impl Iterator<Item = &Unit> {
        self.units
            .iter()
            .filter(move |u| u.champion && u.team == self.team && u.hp > 0 && u.id != self.owner)
    }
    /// every champion on her team, her first (while she is alive)
    pub fn team_champions(&self) -> Vec<Unit> {
        let mut v = Vec::with_capacity(5);
        if self.alive {
            v.push(self.me_as_unit());
        }
        v.extend(self.allies().cloned());
        v
    }
    pub fn p(&self, i: usize) -> i64 {
        self.profile.get(i).copied().unwrap_or(50).clamp(0, 100)
    }
    pub fn unit(&self, id: usize) -> Option<&Unit> {
        self.units.iter().find(|u| u.id == id)
    }
    pub fn clone_view(&self, id: usize) -> Option<CloneView> {
        self.clones.iter().copied().find(|c| c.id == id && c.hp > 0)
    }
    /// where `id` stands now: herself, one of her clones or a unit
    pub fn pos_of(&self, id: usize) -> Option<Point> {
        if id == self.owner {
            return Some(self.pos);
        }
        if let Some(c) = self.clone_view(id) {
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
            farmable: false,
        }
    }
}

// ------------------------------------------------------------------ effects

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Effect {
    /// a new clone for this slot at this spot with this HP
    SpawnClone {
        slot: usize,
        at: Point,
        hp: i64,
    },
    /// a spent clone breaks (it comes back on its own timer)
    DespawnClone {
        clone: usize,
    },
    /// the clone glides to `to` over `ticks`
    CloneMove {
        clone: usize,
        to: Point,
        ticks: u64,
    },
    /// the clone's light shard: damage from her (as the clone's), never a tower
    CloneShot {
        clone: usize,
        target: usize,
        amount: i64,
        from: Point,
    },
    /// a heal on `target` (`from` draws the light)
    Heal {
        target: usize,
        amount: i64,
        from: Point,
    },
    /// Kindred Charm on `target` for `ticks`
    Charm {
        target: usize,
        ticks: u64,
    },
    /// a swap has locked on to these two (draw the thread)
    SwapLock {
        a: usize,
        b: usize,
        ticks: u64,
    },
    /// trade the places of `a` and `b` (read where they are now)
    Swap {
        a: usize,
        b: usize,
    },
    Stun {
        target: usize,
        ticks: u64,
    },
    /// Guardian Angel: `target` is knocked away from `from` (the champion it saves)
    Knockback {
        target: usize,
        from: usize,
    },
    Shield {
        target: usize,
        amount: i64,
        ticks: u64,
    },
    /// the ult's ward on a teammate (an `undying` buff), renewed each tick it lasts
    Ward {
        target: usize,
        ticks: u64,
    },
    /// a clone save's ward on a guarded champion (an `undying` buff), renewed each tick it lasts
    CloneWard {
        target: usize,
        ticks: u64,
    },
    /// sealed: untargetable, unseen, unable to act
    Seal {
        target: usize,
        ticks: u64,
    },
    /// back to `hp` (never lowered); `blessed` adds the ult's Charm, shield, Defence and CC immunity
    Revive {
        target: usize,
        hp: i64,
        blessed: bool,
    },
    /// the AI's walk
    Steer {
        to: Point,
    },
    Fx {
        name: String,
        at: Point,
        ticks: u64,
    },
    Log {
        text: String,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SwapKind {
    /// Mirror Step on a teammate: a clone and the teammate
    StepAlly,
    /// Mirror Step on a point: a clone dashes, then she and the clone
    StepPoint,
    /// Clone Save: a guarding clone and its champion, then the champion's heal and shield and the clone breaks
    Save,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PendingSwap {
    pub kind: SwapKind,
    /// the other side (a clone is always one side)
    pub other: usize,
    pub clone: usize,
    pub started: u64,
    pub at: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Seal {
    pub id: usize,
    pub until: u64,
}

/// One clone slot: the clone in the world (if any) and everything that is its own.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CloneSlot {
    pub id: Option<usize>,
    /// the champion it guards (her id by default)
    pub guard: usize,
    /// a lost clone comes back at this tick
    pub back_at: u64,
    pub shot_at: u64,
    pub mend_at: u64,
    pub charm_at: u64,
    /// dashing for Mirror Step until this tick
    pub dash_until: u64,
    /// keeps its spot (after a swap into a fight) until this tick
    pub hold_until: u64,
    pub moved_at: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Core {
    pub clones: Vec<CloneSlot>,
    pub next_assign: u64,
    /// champions that had an enemy champion close last tick (a new one reassigns at once)
    pub endangered: Vec<usize>,
    /// Mirror Step on a teammate (its own cooldown)
    pub step_ally_at: u64,
    /// Mirror Step on a point: her own swap (its own cooldown)
    pub step_self_at: u64,
    pub charm_at: u64,
    pub mend_at: u64,
    /// Guardian Angel: when each charge is ready again
    pub angel_charges: Vec<u64>,
    /// Guardian Angel: (champion, ready again at)
    pub angel_ready: Vec<(usize, u64)>,
    pub ult_at: u64,
    pub swaps: Vec<PendingSwap>,
    pub seals: Vec<Seal>,
    /// (teammate, until): Kindred Charm already on them
    pub charmed: Vec<(usize, u64)>,
    /// last tick's HP of every champion on her team (her included): only a drop triggers a save
    pub last_hp: Vec<(usize, i64)>,
    pub hp_log: Vec<(u64, i64)>,
    pub steer_at: u64,
    #[serde(default)]
    pub rejection: String,
    #[serde(default)]
    pub ai: crate::kagami_ai::State,
}

impl Default for Core {
    fn default() -> Self {
        Core {
            clones: vec![CloneSlot::default(); d::CLONE_COUNT.max(1) as usize],
            next_assign: 0,
            endangered: vec![],
            step_ally_at: 0,
            step_self_at: 0,
            charm_at: 0,
            mend_at: 0,
            angel_charges: vec![0; d::ANGEL_CHARGES.max(1) as usize],
            angel_ready: vec![],
            ult_at: 0,
            swaps: vec![],
            seals: vec![],
            charmed: vec![],
            last_hp: vec![],
            hp_log: vec![],
            steer_at: 0,
            rejection: String::new(),
            ai: crate::kagami_ai::State::default(),
        }
    }
}

fn travel(from: Point, to: Point, speed: i64) -> u64 {
    ((from.dist(to) / speed.max(1) as f64).ceil() as u64).max(1)
}

impl Core {
    /// She came back from death: any missing clone comes back with her at once; timers keep running.
    pub fn respawn(&mut self, tick: u64) {
        self.ai = crate::kagami_ai::State::default();
        for s in self.clones.iter_mut().filter(|s| s.id.is_none()) {
            s.back_at = tick;
        }
        self.swaps.retain(|s| s.kind == SwapKind::Save);
        self.hp_log.clear();
        self.last_hp.clear();
    }
    /// her live clones as the frame has them (slot, view)
    pub fn live_clones(&self, f: &Frame) -> Vec<(usize, CloneView)> {
        self.clones
            .iter()
            .enumerate()
            .filter_map(|(i, s)| s.id.and_then(|id| f.clone_view(id)).map(|v| (i, v)))
            .collect()
    }
    pub fn clone_alive(&self, f: &Frame) -> bool {
        !self.live_clones(f).is_empty()
    }
    /// a clone or champion that is a side of a pending swap
    pub fn in_swap(&self, id: usize) -> bool {
        self.swaps.iter().any(|s| s.clone == id || s.other == id)
    }
    /// a Mirror Step (either kind) is in flight
    pub fn stepping(&self) -> bool {
        self.swaps.iter().any(|s| matches!(s.kind, SwapKind::StepAlly | SwapKind::StepPoint))
    }
    /// she herself is a side of a swap (her walk and casts wait)
    pub fn self_swapping(&self, f: &Frame) -> bool {
        self.swaps.iter().any(|s| s.other == f.owner)
    }
    /// live clones not busy in a swap
    pub fn free_clones(&self, f: &Frame) -> Vec<(usize, CloneView)> {
        self.live_clones(f).into_iter().filter(|(_, c)| !self.in_swap(c.id)).collect()
    }
    /// a ready clone save for this champion: a live, free clone guarding them (the nearest one)
    pub fn save_clone(&self, f: &Frame, champion: usize) -> Option<CloneView> {
        let at = f.pos_of(champion)?;
        self.free_clones(f)
            .into_iter()
            .filter(|(i, _)| self.clones[*i].guard == champion)
            .map(|(_, c)| c)
            .min_by_key(|c| (c.pos.d2(at), c.id))
    }
    pub fn sealed(&self, id: usize) -> bool {
        self.seals.iter().any(|s| s.id == id)
    }
    pub fn ult_ready(&self, f: &Frame) -> bool {
        f.ult_unlocked && f.tick >= self.ult_at
    }
    pub(crate) fn charmed(&self, id: usize, tick: u64) -> bool {
        self.charmed.iter().any(|c| c.0 == id && tick < c.1)
    }
    fn last_hp_of(&self, id: usize) -> Option<i64> {
        self.last_hp.iter().find(|h| h.0 == id).map(|h| h.1)
    }
    fn slot_of(&self, clone: usize) -> Option<usize> {
        self.clones.iter().position(|s| s.id == Some(clone))
    }

    pub fn step(&mut self, f: &Frame) -> Vec<Effect> {
        let mut out = vec![];
        self.rejection.clear();
        self.sync_clones(f, &mut out);
        self.charmed.retain(|c| f.tick < c.1);
        self.angel_ready.retain(|a| f.tick < a.1);
        self.run_seals(f, &mut out);
        // the saves and the swaps run while she is down too: her clones keep guarding
        self.run_clone_saves(f, &mut out);
        self.run_ult(f, &mut out);
        self.run_angel(f, &mut out);
        self.run_swaps(f, &mut out);
        if !f.alive {
            self.hp_log.clear();
            self.ai = crate::kagami_ai::State::default();
            self.note_hp(f);
            return out;
        }
        self.hp_log.retain(|h| f.tick < h.0 + t(d::AI_HURT_TICKS));
        self.hp_log.push((f.tick, f.hp));
        self.assign_clones(f);
        self.run_clones(f, &mut out);

        let order = if f.manual { f.order } else { self.decide(f) };
        if let Some(o) = order {
            if !f.stunned && !self.sealed(f.owner) {
                self.execute(f, o, &mut out);
            }
        }
        if !f.manual && !self.sealed(f.owner) && !self.self_swapping(f) {
            if let Some(to) = self.steer(f) {
                out.push(Effect::Steer { to });
            }
        }
        self.note_hp(f);
        out
    }

    /// Her clones as the world has them: a lost clone starts its own respawn timer and cancels its swaps.
    fn sync_clones(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        for i in 0..self.clones.len() {
            match self.clones[i].id {
                Some(id) => {
                    if f.clone_view(id).is_none() {
                        // killed: its own timer starts now (a spent clone is already cleared by its save)
                        self.clones[i].id = None;
                        self.clones[i].back_at = f.tick + t(d::CLONE_RESPAWN_TICKS);
                        let lost: Vec<PendingSwap> = self.swaps.iter().filter(|s| s.clone == id).copied().collect();
                        for s in lost {
                            self.cancel_swap(f, s, out);
                        }
                    }
                }
                None => {
                    if let Some(c) = f.clones.iter().find(|c| c.slot == i && c.hp > 0) {
                        self.clones[i].id = Some(c.id);
                    }
                }
            }
        }
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
            let Some(u) = f.unit(s.id).cloned() else { continue };
            if u.max_hp <= 0 {
                continue;
            }
            out.push(Effect::Revive {
                target: s.id,
                hp: u.max_hp,
                blessed: true,
            });
            // the revive stuns the enemies around them
            for e in f
                .units
                .iter()
                .filter(|e| f.enemy(e) && !e.tower && e.pos.within(u.pos, d::ULT_REVIVE_STUN_RADIUS))
            {
                out.push(Effect::Stun {
                    target: e.id,
                    ticks: t(d::ULT_REVIVE_STUN_TICKS),
                });
            }
            out.push(Effect::Fx {
                name: "kg_revive".into(),
                at: u.pos,
                ticks: 30,
            });
            out.push(Effect::Fx {
                name: "kg_stun_burst".into(),
                at: u.pos,
                ticks: 30,
            });
        }
        // a sealed teammate stays held at 1 HP until the revive
        for s in &self.seals {
            out.push(Effect::Ward { target: s.id, ticks: 3 });
        }
    }

    /// Clone Save: every champion with a ready guarding clone is warded; one who drops under the line is held at 1 HP
    /// and swap-locked with that clone.
    fn run_clone_saves(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        for u in f.team_champions() {
            if u.sealed || self.sealed(u.id) {
                continue;
            }
            // a save already locked on to them keeps them held
            if self.swaps.iter().any(|s| s.kind == SwapKind::Save && s.other == u.id) {
                out.push(Effect::CloneWard { target: u.id, ticks: 3 });
                continue;
            }
            let Some(c) = self.save_clone(f, u.id) else { continue };
            out.push(Effect::CloneWard { target: u.id, ticks: 3 });
            let dropped = self.last_hp_of(u.id).is_some_and(|h| u.hp < h);
            if dropped && u.hp * 100 <= u.max_hp * d::SAVE_TRIGGER_PCT {
                // a save outranks a Mirror Step in flight on either side
                let stale: Vec<PendingSwap> = self.swaps.iter().filter(|s| s.clone == c.id || s.other == u.id).copied().collect();
                for s in stale {
                    self.cancel_swap(f, s, out);
                }
                if let Some(i) = self.slot_of(c.id) {
                    self.clones[i].dash_until = 0;
                }
                self.lock(f, SwapKind::Save, c.id, u.id, f.tick + t(d::SWAP_LOCK_TICKS), out);
                out.push(Effect::Log {
                    text: format!("Clone Save locks on to {}", u.id),
                });
            }
        }
    }

    /// Last Reflection, the backup: wards every teammate (not her) with no clone save ready; the first one who drops
    /// under the line is sealed.
    fn run_ult(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        if !self.ult_ready(f) {
            return;
        }
        let covered = |c: &Core, id: usize| c.save_clone(f, id).is_some() || c.swaps.iter().any(|s| s.kind == SwapKind::Save && s.other == id);
        let line = |u: &Unit| u.hp * 100 <= u.max_hp * d::ULT_TRIGGER_PCT;
        let hit = f
            .allies()
            .filter(|u| !u.sealed && !self.sealed(u.id) && !covered(self, u.id))
            .filter(|u| line(u) && self.last_hp_of(u.id).is_some_and(|h| u.hp < h))
            .min_by_key(|u| (u.hp * 1000 / u.max_hp.max(1), u.id))
            .map(|u| (u.id, u.pos));
        if let Some((id, at)) = hit {
            self.ult_at = f.tick + t(d::ULT_COOLDOWN_TICKS);
            self.seals.push(Seal {
                id,
                until: f.tick + t(d::ULT_SEAL_TICKS),
            });
            out.push(Effect::Seal {
                target: id,
                ticks: t(d::ULT_SEAL_TICKS),
            });
            out.push(Effect::Ward { target: id, ticks: 3 });
            out.push(Effect::Fx {
                name: "kg_seal".into(),
                at,
                ticks: t(d::ULT_SEAL_TICKS),
            });
            out.push(Effect::Log {
                text: format!("Last Reflection seals {id}"),
            });
            return;
        }
        let warded: Vec<usize> = f
            .allies()
            .filter(|u| !u.sealed && !covered(self, u.id))
            .map(|u| u.id)
            .collect();
        for id in warded {
            out.push(Effect::Ward { target: id, ticks: 3 });
        }
    }

    /// Guardian Angel (2 charges): any champion on her team crossing below the line, off their own cooldown, knocks
    /// enemies away from them and is shielded.
    fn run_angel(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        for u in f.team_champions() {
            if u.sealed || self.sealed(u.id) || self.angel_ready.iter().any(|a| a.0 == u.id) {
                continue;
            }
            let line = |hp: i64| hp * 100 < u.max_hp * d::ANGEL_THRESHOLD_PCT;
            if !line(u.hp) || self.last_hp_of(u.id).is_none_or(line) {
                continue;
            }
            // the charge that has been ready longest
            let Some(charge) = self
                .angel_charges
                .iter()
                .enumerate()
                .filter(|(_, at)| f.tick >= **at)
                .min_by_key(|(i, at)| (**at, *i))
                .map(|(i, _)| i)
            else {
                return;
            };
            self.angel_charges[charge] = f.tick + t(d::ANGEL_COOLDOWN_TICKS);
            self.angel_ready.push((u.id, f.tick + t(d::ANGEL_PER_CHAMPION_COOLDOWN_TICKS)));
            for e in f.units.iter().filter(|e| f.enemy(e) && !e.tower && e.pos.within(u.pos, d::ANGEL_RADIUS)) {
                out.push(Effect::Knockback { target: e.id, from: u.id });
            }
            out.push(Effect::Shield {
                target: u.id,
                amount: pct_of(u.max_hp, d::ANGEL_SHIELD_PCT_MAX_HP),
                ticks: t(d::ANGEL_SHIELD_TICKS),
            });
            out.push(Effect::Fx {
                name: "kg_angel".into(),
                at: u.pos,
                ticks: 36,
            });
        }
    }

    // -------------------------------------------------------------- swaps

    fn lock(&mut self, f: &Frame, kind: SwapKind, clone: usize, other: usize, at: u64, out: &mut Vec<Effect>) {
        self.swaps.push(PendingSwap {
            kind,
            other,
            clone,
            started: f.tick,
            at,
        });
        out.push(Effect::SwapLock {
            a: clone,
            b: other,
            ticks: at.saturating_sub(f.tick),
        });
    }

    fn cancel_swap(&mut self, f: &Frame, s: PendingSwap, out: &mut Vec<Effect>) {
        let before = self.swaps.len();
        self.swaps.retain(|x| *x != s);
        if self.swaps.len() == before {
            return;
        }
        // only the cancelled swap's own cooldown is refunded
        match s.kind {
            SwapKind::StepAlly => {
                let back = t(d::STEP_ALLY_COOLDOWN_TICKS) * t(d::SWAP_REFUND_PCT) / 100;
                self.step_ally_at = self.step_ally_at.saturating_sub(back).max(f.tick);
            }
            SwapKind::StepPoint => {
                let back = t(d::STEP_SELF_COOLDOWN_TICKS) * t(d::SWAP_REFUND_PCT) / 100;
                self.step_self_at = self.step_self_at.saturating_sub(back).max(f.tick);
            }
            SwapKind::Save => {}
        }
        if let Some(i) = self.slot_of(s.clone) {
            self.clones[i].dash_until = 0;
        }
        out.push(Effect::Log { text: "swap cancelled".into() });
    }

    fn side_ok(&self, f: &Frame, id: usize) -> bool {
        if id == f.owner {
            f.alive
        } else {
            f.unit(id).is_some_and(|u| u.hp > 0 && u.targetable && !u.sealed && !self.sealed(u.id))
        }
    }

    fn run_swaps(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        let pending = self.swaps.clone();
        for s in pending {
            let clone = f.clone_view(s.clone);
            let (Some(c), true) = (clone, self.side_ok(f, s.other)) else {
                self.cancel_swap(f, s, out);
                continue;
            };
            if f.tick < s.at {
                continue;
            }
            let Some(other_pos) = f.pos_of(s.other) else {
                self.cancel_swap(f, s, out);
                continue;
            };
            self.swaps.retain(|x| *x != s);
            out.push(Effect::Swap { a: c.id, b: s.other });
            out.push(Effect::Fx {
                name: "kg_swap".into(),
                at: c.pos,
                ticks: 24,
            });
            out.push(Effect::Fx {
                name: "kg_swap".into(),
                at: other_pos,
                ticks: 24,
            });
            // the clone lands where the other side stood: the stun goes off there
            let stun = t(d::SWAP_STUN_TICKS);
            for u in f
                .units
                .iter()
                .filter(|u| f.enemy(u) && !u.tower && u.pos.within(other_pos, d::SWAP_STUN_RADIUS))
            {
                out.push(Effect::Stun { target: u.id, ticks: stun });
            }
            out.push(Effect::Fx {
                name: "kg_stun_burst".into(),
                at: other_pos,
                ticks: 30,
            });
            let slot = self.slot_of(c.id);
            if let Some(i) = slot {
                self.clones[i].dash_until = 0;
                // after a swap into a fight the clone keeps its new spot a while
                self.clones[i].hold_until = f.tick + 180;
            }
            if s.kind == SwapKind::Save {
                let max = if s.other == f.owner { f.max_hp } else { f.unit(s.other).map_or(0, |u| u.max_hp) };
                out.push(Effect::Revive {
                    target: s.other,
                    hp: pct_of(max, d::SAVE_HEAL_PCT).max(1),
                    blessed: false,
                });
                out.push(Effect::Shield {
                    target: s.other,
                    amount: pct_of(max, d::SAVE_SHIELD_PCT),
                    ticks: t(d::SAVE_SHIELD_TICKS),
                });
                out.push(Effect::Fx {
                    name: "kg_revive".into(),
                    at: c.pos,
                    ticks: 30,
                });
                // the clone is spent: it breaks and comes back on its own timer
                out.push(Effect::DespawnClone { clone: c.id });
                if let Some(i) = slot {
                    self.clones[i].id = None;
                    self.clones[i].back_at = f.tick + t(d::CLONE_RESPAWN_TICKS);
                }
            }
        }
    }

    // -------------------------------------------------------------- the clones

    /// How much a champion needs a guarding clone right now (higher is more).
    pub fn need(f: &Frame, u: &Unit) -> i64 {
        let danger = Self::danger_at(f, u.pos) / 1000;
        let low = 100 - Self::hp_pct(u);
        danger + low + u.attack / 10
    }

    /// Automatic by danger: every few seconds (or at once when someone new is in danger) each clone picks the champion
    /// that needs it most; every clone already on a champion lowers their need, and a clone only switches for a
    /// clearly better pick.
    fn assign_clones(&mut self, f: &Frame) {
        let team = f.team_champions();
        if team.is_empty() {
            return;
        }
        let now: Vec<usize> = team
            .iter()
            .filter(|u| f.enemies_near(u.pos, d::AI_DANGER_RANGE) > 0)
            .map(|u| u.id)
            .collect();
        let newly = now.iter().any(|id| !self.endangered.contains(id));
        let gone = self.clones.iter().any(|s| s.id.is_some() && !team.iter().any(|u| u.id == s.guard && !u.sealed));
        self.endangered = now;
        if f.tick < self.next_assign && !newly && !gone {
            return;
        }
        self.next_assign = f.tick + t(d::CLONE_ASSIGN_TICKS);
        let fighting = team.iter().any(|u| f.enemies_near(u.pos, d::AI_FIGHT_RANGE) > 0);
        let candidates: Vec<&Unit> = team.iter().filter(|u| !u.sealed && !self.sealed(u.id)).collect();
        if candidates.is_empty() {
            return;
        }
        if !fighting {
            // calm: one clone on her, the rest on the hardest hitter
            let carry = f
                .allies()
                .filter(|u| !u.sealed)
                .max_by_key(|u| (u.attack, std::cmp::Reverse(u.id)))
                .map_or(f.owner, |u| u.id);
            for (i, s) in self.clones.iter_mut().enumerate() {
                s.guard = if i == 0 { f.owner } else { carry };
            }
            return;
        }
        let mut count: Vec<(usize, i64)> = candidates.iter().map(|u| (u.id, 0)).collect();
        let penalty = |n: i64| n * 60;
        for i in 0..self.clones.len() {
            let current = self.clones[i].guard;
            let score = |u: &Unit, count: &Vec<(usize, i64)>| {
                let n = count.iter().find(|c| c.0 == u.id).map_or(0, |c| c.1);
                Self::need(f, u) - penalty(n)
            };
            let best = candidates
                .iter()
                .max_by_key(|u| (score(u, &count), std::cmp::Reverse(u.id)))
                .map(|u| (u.id, score(u, &count)));
            let keep = candidates.iter().find(|u| u.id == current).map(|u| score(u, &count));
            let pick = match (best, keep) {
                (Some((b, sb)), Some(sk)) if sb < sk + d::CLONE_SWITCH_MARGIN => {
                    let _ = b;
                    current
                }
                (Some((b, _)), _) => b,
                (None, _) => f.owner,
            };
            self.clones[i].guard = pick;
            if let Some(c) = count.iter_mut().find(|c| c.0 == pick) {
                c.1 += 1;
            }
        }
    }

    fn run_clones(&mut self, f: &Frame, out: &mut Vec<Effect>) {
        let cadence = if f.manual { 6 } else { crate::kagami_ai::cadence(f) };
        for i in 0..self.clones.len() {
            let slot = self.clones[i];
            let Some(c) = slot.id.and_then(|id| f.clone_view(id)) else {
                if slot.id.is_none() && f.tick >= slot.back_at {
                    // the bridge reports the new id next frame; until then no second one is asked for
                    self.clones[i].back_at = f.tick + 30;
                    out.push(Effect::SpawnClone {
                        slot: i,
                        at: f.pos,
                        hp: pct_of(f.max_hp, d::CLONE_HP_PCT).max(1),
                    });
                    out.push(Effect::Fx {
                        name: "kg_clone_in".into(),
                        at: f.pos,
                        ticks: 24,
                    });
                }
                continue;
            };
            let mine = c.pos;
            let guard_pos = f.pos_of(slot.guard).unwrap_or(f.pos);
            // its shard: champions first, never a tower; minions only near the champion it guards
            if f.tick >= slot.shot_at {
                let target = f
                    .units
                    .iter()
                    .filter(|u| f.enemy(u) && !u.tower && u.pos.within(mine, d::CLONE_ATTACK_RANGE))
                    .min_by_key(|u| (!u.champion, u.pos.d2(mine), u.id))
                    .filter(|u| {
                        u.champion || f.enemies_near(guard_pos, d::AI_FIGHT_RANGE) == 0 || u.pos.within(guard_pos, d::CLONE_ATTACK_RANGE)
                    });
                if let Some(u) = target {
                    self.clones[i].shot_at = f.tick + t(d::CLONE_ATTACK_TICKS);
                    out.push(Effect::CloneShot {
                        clone: c.id,
                        target: u.id,
                        amount: pct_of(f.attack, d::CLONE_DAMAGE_PCT).max(1),
                        from: mine,
                    });
                }
            }
            // its own Mend: the most hurt champion near it (her included)
            if f.tick >= slot.mend_at {
                let team: Vec<Unit> = f.team_champions().into_iter().filter(|u| !u.sealed && !self.sealed(u.id)).collect();
                if let Some(u) = team
                    .iter()
                    .filter(|u| u.pos.within(mine, d::MEND_RANGE) && u.hp * 100 < u.max_hp * d::AI_HEAL_BELOW_PCT)
                    .min_by_key(|u| (u.hp * 1000 / u.max_hp.max(1), u.id))
                {
                    self.clones[i].mend_at = f.tick + t(d::CLONE_MEND_COOLDOWN_TICKS);
                    let amount = pct_of(self.mend_amount(f), d::CLONE_MEND_PCT);
                    out.push(Effect::Heal {
                        target: u.id,
                        amount,
                        from: mine,
                    });
                }
            }
            // its own Charm: a teammate near it who is fighting
            if f.tick >= slot.charm_at {
                if let Some(id) = self.charm_pick(f, mine) {
                    self.clones[i].charm_at = f.tick + t(d::CLONE_CHARM_COOLDOWN_TICKS);
                    self.charmed.push((id, f.tick + t(d::CHARM_TICKS)));
                    out.push(Effect::Charm {
                        target: id,
                        ticks: t(d::CHARM_TICKS),
                    });
                    out.push(Effect::Fx {
                        name: "kg_charm".into(),
                        at: f.pos_of(id).unwrap_or(mine),
                        ticks: 24,
                    });
                }
            }
            // movement: never while a swap holds it (the lock follows ids, so it may still be pushed by others)
            if self.in_swap(c.id) || f.tick < slot.dash_until || f.tick < slot.moved_at.saturating_add(cadence) {
                continue;
            }
            let shooting = f
                .units
                .iter()
                .any(|u| f.enemy(u) && !u.tower && u.champion && u.pos.within(mine, d::CLONE_ATTACK_RANGE));
            // no range limit: a far clone races to its champion
            let far = !mine.within(guard_pos, d::CLONE_CATCH_UP_RANGE);
            if !far && f.tick < slot.hold_until && shooting {
                continue;
            }
            let spot = self.clone_spot_for(f, guard_pos, i);
            if far || (!mine.within(spot, d::CLONE_FOLLOW_GAP / 2) && !(shooting && mine.within(guard_pos, d::CLONE_CATCH_UP_RANGE / 2))) {
                let speed = if far { d::CLONE_DASH_SPEED } else { d::CLONE_MOVE_SPEED };
                let step = mine.toward(spot, speed * cadence as i64);
                if step != mine {
                    self.clones[i].moved_at = f.tick;
                    out.push(Effect::CloneMove {
                        clone: c.id,
                        to: step,
                        ticks: cadence,
                    });
                }
            }
        }
    }

    /// Where a clone stands when calm: a step behind the champion it guards, away from the nearest enemy champion (or
    /// toward home); clones on the same champion fan out by slot.
    pub fn clone_spot_for(&self, f: &Frame, at: Point, slot: usize) -> Point {
        let away = f
            .enemy_champions()
            .min_by_key(|u| u.pos.d2(at))
            .filter(|u| u.pos.within(at, d::AI_ESCAPE_RANGE))
            .map(|u| u.pos)
            .unwrap_or_else(|| Point::new(2 * at.x - f.home.x, 2 * at.y - f.home.y));
        let d = at.dist(away).max(1.0);
        let (ux, uy) = ((at.x - away.x) as f64 / d, (at.y - away.y) as f64 / d);
        let turn = [0.0f64, 0.7, -0.7][slot % 3];
        let (rx, ry) = (ux * turn.cos() - uy * turn.sin(), ux * turn.sin() + uy * turn.cos());
        let gap = d::CLONE_FOLLOW_GAP as f64;
        Point::new(at.x + (rx * gap) as i64, at.y + (ry * gap) as i64).clamp_map()
    }

    /// The calm spot of a clone guarding her (kept for the AI and the Lab).
    pub fn clone_spot(&self, f: &Frame) -> Point {
        self.clone_spot_for(f, f.pos, 0)
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

    /// The clone a teammate is swapped with: the safest free clone (any distance), never one already beside them.
    pub fn rescue_clone(&self, f: &Frame, ally: Point) -> Option<CloneView> {
        self.free_clones(f)
            .into_iter()
            .map(|(_, c)| c)
            .filter(|c| !c.pos.within(ally, 20_000))
            .min_by_key(|c| (Self::danger_at(f, c.pos), c.pos.d2(f.pos), c.id))
    }

    /// The clone that serves her own swap to `p`: one already there, else her nearest free clone.
    pub fn step_clone(&self, f: &Frame, p: Point) -> Option<CloneView> {
        let free: Vec<CloneView> = self.free_clones(f).into_iter().map(|(_, c)| c).collect();
        free.iter()
            .copied()
            .find(|c| c.pos.within(p, 2_000))
            .or_else(|| free.iter().copied().min_by_key(|c| (c.pos.d2(f.pos), c.id)))
    }

    pub fn execute(&mut self, f: &Frame, o: Order, out: &mut Vec<Effect>) {
        match o {
            Order::StepAlly(id) => {
                let ok = f
                    .unit(id)
                    .is_some_and(|u| u.team == f.team && u.champion && u.hp > 0 && u.targetable && !u.sealed && !self.in_swap(u.id));
                if f.tick < self.step_ally_at || !ok || id == f.owner {
                    return self.reject("step");
                }
                let at = f.unit(id).map(|u| u.pos).unwrap_or_default();
                let Some(c) = self.rescue_clone(f, at) else {
                    return self.reject("no clone");
                };
                self.step_ally_at = f.tick + t(d::STEP_ALLY_COOLDOWN_TICKS);
                self.lock(f, SwapKind::StepAlly, c.id, id, f.tick + t(d::SWAP_LOCK_TICKS), out);
            }
            Order::StepPoint(p) => {
                if f.tick < self.step_self_at || self.self_swapping(f) || !p.within(f.pos, d::STEP_RANGE) || !p.on_map() {
                    return self.reject("step");
                }
                let Some(c) = self.step_clone(f, p) else {
                    return self.reject("no clone");
                };
                self.step_self_at = f.tick + t(d::STEP_SELF_COOLDOWN_TICKS);
                let dash = if c.pos.within(p, 2_000) { 0 } else { travel(c.pos, p, d::CLONE_DASH_SPEED) };
                if dash > 0 {
                    if let Some(i) = self.slot_of(c.id) {
                        self.clones[i].dash_until = f.tick + dash;
                    }
                    out.push(Effect::CloneMove {
                        clone: c.id,
                        to: p,
                        ticks: dash,
                    });
                }
                self.lock(f, SwapKind::StepPoint, c.id, f.owner, f.tick + dash + t(d::SWAP_LOCK_TICKS), out);
            }
            Order::Charm(id) => {
                let target = if id == f.owner {
                    Some(f.me_as_unit())
                } else {
                    f.unit(id).filter(|u| u.team == f.team && u.champion).cloned()
                };
                let Some(u) = target.filter(|u| u.hp > 0 && !u.sealed && u.pos.within(f.pos, d::CHARM_RANGE)) else {
                    return self.reject("charm");
                };
                if f.tick < self.charm_at {
                    return self.reject("charm");
                }
                self.charm_at = f.tick + t(d::CHARM_COOLDOWN_TICKS);
                self.charmed.retain(|c| c.0 != u.id);
                self.charmed.push((u.id, f.tick + t(d::CHARM_TICKS)));
                out.push(Effect::Charm {
                    target: u.id,
                    ticks: t(d::CHARM_TICKS),
                });
                out.push(Effect::Fx {
                    name: "kg_charm".into(),
                    at: u.pos,
                    ticks: 24,
                });
            }
            Order::Mend(id) => {
                let target = if id == f.owner {
                    Some(f.me_as_unit())
                } else {
                    f.unit(id).filter(|u| u.team == f.team && u.champion).cloned()
                };
                let Some(u) = target.filter(|u| u.hp > 0 && !u.sealed && u.pos.within(f.pos, d::MEND_RANGE)) else {
                    return self.reject("mend");
                };
                if f.tick < self.mend_at {
                    return self.reject("mend");
                }
                self.mend_at = f.tick + t(d::MEND_COOLDOWN_TICKS);
                out.push(Effect::Heal {
                    target: u.id,
                    amount: self.mend_amount(f),
                    from: f.pos,
                });
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
        f.enemy_champions()
            .filter(|u| u.pos.within(p, d::AI_ESCAPE_RANGE))
            .map(|u| (d::AI_ESCAPE_RANGE as f64 - u.pos.dist(p)).max(0.0) as i64)
            .sum()
    }

    /// she lost a real share of her HP lately
    pub fn hurt(&self, f: &Frame) -> bool {
        let top = self.hp_log.iter().map(|h| h.1).max().unwrap_or(f.hp);
        (top - f.hp) * 100 >= f.max_hp * d::AI_HURT_PCT
    }

    /// The teammate to rescue with Mirror Step: low, enemies on them, and a clone somewhere safer than their spot.
    pub fn rescue_target(&self, f: &Frame) -> Option<usize> {
        // judgement moves the line by up to 10 points either way
        let line = d::AI_DANGER_HP_PCT + (f.p(JUDGEMENT) - 50) / 5;
        f.allies()
            .filter(|u| !u.sealed && u.targetable && !self.in_swap(u.id))
            .filter(|u| Self::hp_pct(u) <= line && f.enemies_near(u.pos, d::AI_DANGER_RANGE) > 0)
            .filter(|u| self.rescue_clone(f, u.pos).is_some_and(|c| Self::danger_at(f, c.pos) < Self::danger_at(f, u.pos)))
            .min_by_key(|u| (Self::hp_pct(u), u.id))
            .map(|u| u.id)
    }

    /// Her own escape: dived and hurt, the safest clone in reach (or a point toward home) is the way out.
    pub fn escape_point(&self, f: &Frame) -> Option<Point> {
        let free: Vec<CloneView> = self.free_clones(f).into_iter().map(|(_, c)| c).collect();
        if free.is_empty() {
            return None;
        }
        let dived = f.enemies_near(f.pos, d::AI_DIVED_RANGE) > 0;
        if !dived || !(self.hurt(f) || f.hp * 100 < f.max_hp * 50) {
            return None;
        }
        let here = Self::danger_at(f, f.pos);
        if let Some(c) = free
            .iter()
            .filter(|c| c.pos.within(f.pos, d::STEP_RANGE) && !c.pos.within(f.pos, 30_000) && Self::danger_at(f, c.pos) * 2 < here)
            .min_by_key(|c| (Self::danger_at(f, c.pos), c.id))
        {
            return Some(c.pos);
        }
        let home = f.pos.toward(f.home, d::STEP_RANGE - 5_000).clamp_map();
        (Self::danger_at(f, home) * 2 < here).then_some(home)
    }

    /// The teammate she keeps close to: the strongest hitter within reach.
    pub fn guard_target(&self, f: &Frame) -> Option<Unit> {
        f.allies()
            .filter(|u| !u.sealed && u.pos.within(f.pos, 160_000))
            .max_by_key(|u| (u.attack, std::cmp::Reverse(u.id)))
            .cloned()
    }

    pub fn decide(&mut self, f: &Frame) -> Option<Order> {
        crate::kagami_ai::decide(self, f)
    }

    /// Movement intentions never replace host attacks or casts.
    pub fn steer(&mut self, f: &Frame) -> Option<Point> {
        crate::kagami_ai::steer(self, f)
    }
}
