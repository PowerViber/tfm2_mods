//! Vesper, the Rift Gauntlets: every rule as a pure, deterministic engine.
//!
//! `Core::step` takes one tick's snapshot (fighters, Vesper's state) and returns the commands the adapter
//! (`vesper.rs`) applies through the stable API. Nothing in here reads the simulation, so the native tests and the
//! Rift Lab (editor/vesper-combat.js) run exactly the same rules. Mastery only changes the planner (`think`): every
//! orb, dimension, rift type, technique and gauntlet move is available at every rank with identical numbers.
//!
//! The left gauntlet holds pure orbs (5, overloadable to 9); the right gauntlet holds hexorbs and the Hexcore
//! (70% at the start of a fight, researched up to an Awakening). Any orb opens any of the 100 dimensions: as a rift
//! (its zone), as a rift punch (the hole she punches) or as a rift swipe (the tear she rips).
use crate::unified_theory_math::Window;
use crate::vesper_data::*;
use crate::vesper_math::*;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

/// Rank look for the art: 0..7 per rank, 8 for the #1 athlete.
pub fn look(rank: usize, top: Option<usize>) -> usize {
    if rank >= 7 && top == Some(1) {
        8
    } else {
        rank.min(7)
    }
}
pub fn art_tier(rank: usize) -> usize {
    [0, 0, 0, 1, 1, 2, 2, 3][rank.min(7)]
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fighter {
    pub id: usize,
    pub team: usize,
    pub x: i64,
    pub y: i64,
    pub hp: i64,
    pub max_hp: i64,
    pub attack: i64,
    pub cc_immune: bool,
    pub controlled: bool,
}

/// One tick of input.
pub struct Input<'a> {
    pub tick: usize,
    pub me: usize,
    /// Champions in the fight (Vesper included). Enemies are only the visible ones.
    pub fighters: &'a [Fighter],
    /// Ticks between Vesper's basic attacks (after attack-speed buffs).
    pub attack_interval: usize,
    pub stunned: bool,
    pub wall: &'a dyn Fn(i64, i64) -> bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dmg {
    Phys,
    Magic,
    True,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cc {
    Stun,
    Root,
    Silence,
    Disarm,
}
/// Buff fields the adapter maps onto `BuffV1`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Mods {
    pub move_speed: i32,
    pub attack_speed: i32,
    pub amplify: i32,
    pub reduce: i32,
    pub heal_reduce: i32,
    pub attack: i32,
    pub defence: i32,
    pub mr: i32,
    pub ap: i32,
    pub regen: i32,
    pub crit: i32,
    pub range: i32,
    pub ba_reduce: i32,
    pub reflect: i32,
    pub cooldown: i32,
    pub vamp: i32,
    pub ignore_wall: bool,
    pub undying: bool,
    pub cc_immune: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Cmd {
    Damage { target: usize, amount: i64, kind: Dmg },
    Heal { target: usize, amount: i64 },
    Shield { target: usize, amount: i64, ticks: usize },
    Cc { target: usize, kind: Cc, ticks: usize },
    Buff { target: usize, name: String, ticks: usize, mods: Mods },
    SetPos { target: usize, x: i64, y: i64 },
    Banish { target: usize, ticks: usize },
    Invisible { target: usize, ticks: usize },
    ClearCc { target: usize },
    ClearShield { target: usize },
    /// Non-lethal self harm (a gauntlet breaking).
    LoseHp { target: usize, amount: i64 },
    /// A view effect `vesper_<tag>` at a point, or following a unit.
    Fx { tag: String, x: i64, y: i64, unit: Option<usize>, ticks: usize },
    Log(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rift {
    pub uid: u32,
    pub kind: usize,
    /// Technique index (0..28) or gauntlet move (28..34) that opened it.
    pub tech: usize,
    /// The dimension of the orb that opened it.
    pub dim: usize,
    /// Stitch: a second dimension carried by the same rift.
    pub dim2: Option<usize>,
    /// Opened by a hexorb: its tech module fires when it opens.
    pub module: Option<usize>,
    /// Infuse: a tech module pulsing from this rift for the rest of its life.
    pub infused: Option<usize>,
    pub x: i64,
    pub y: i64,
    /// Second end of a Slash or Mirror Pane, or the end of a Wandering Rift's path.
    pub ex: i64,
    pub ey: i64,
    pub radius: i64,
    pub opens: usize,
    pub expires: usize,
    /// Anchor host (follows it).
    pub host: Option<usize>,
    /// The paired rift (Gate / Kidnap / Twin exit and the like).
    pub peer: Option<u32>,
    /// Fighter this rift was opened for (Kidnap victim, Rescue ally, Hook target, ...).
    pub subject: Option<usize>,
    /// Chain order (chain nodes only).
    pub chain: Option<u32>,
    pub entrance: bool,
    pub opened: bool,
    /// One-shot techniques (Step, Phase, Kidnap, Rescue, Swap) fire once both ends are open.
    pub fired: bool,
    pub dwell: BTreeMap<usize, usize>,
    pub touched: BTreeSet<usize>,
    pub sides: BTreeMap<usize, i32>,
    pub inside: BTreeSet<usize>,
    pub done: bool,
    /// Pressure: charges stored by pulses with an enemy inside, released when the rift closes or implodes.
    pub charge: i64,
    /// When each ambient layer (shape, core, glyph, the link into it) runs out and is drawn again.
    pub drawn: [usize; 4],
}
impl Rift {
    pub fn live(&self) -> bool {
        self.opened && !self.done
    }
    pub fn dims(&self) -> Vec<usize> {
        let mut v = vec![self.dim];
        if let Some(d) = self.dim2 {
            v.push(d);
        }
        v
    }
    pub fn hidden(&self, tick: usize) -> bool {
        self.kind == R_SEED && tick < self.opens + SEED_HIDDEN as usize
    }
    /// Distance from a point to the rift's body (a segment for Slash / Mirror Pane).
    pub fn reach(&self, x: i64, y: i64) -> i64 {
        if matches!(self.kind, R_SLASH | R_MIRROR) {
            seg_dist(self.x, self.y, self.ex, self.ey, x, y)
        } else {
            dist(self.x, self.y, x, y)
        }
    }
    pub fn covers(&self, x: i64, y: i64) -> bool {
        self.reach(x, y) <= self.radius
    }
}

/// One orb socket of the left gauntlet. Sockets 0..5 are the base; 5..9 exist only while overloaded.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Socket {
    pub open: bool,
    pub over: bool,
    pub ready_at: usize,
    /// Family of the dimension it last opened (the art colours the socket), 255 for none yet.
    pub family: u8,
}
/// An orb taken from a gauntlet: a pure orb (`hex` None) or a hexorb with its slot's tech module.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Orb {
    pub hex: Option<usize>,
    pub over: bool,
    /// The pure orb's socket (unused for a hexorb).
    pub socket: usize,
}

/// A rift punch in flight: the fist goes in at Vesper (or the first rift) and comes out of the last one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Packet {
    pub x: i64,
    pub y: i64,
    /// Where the current link started (its full length is what costs efficiency).
    pub hop: (i64, i64),
    /// Remaining waypoints: (rift uid or 0 for the target, x, y).
    pub path: VecDeque<(u32, i64, i64)>,
    pub target: usize,
    pub eff: i64,
    pub bonus: i64,
    /// The punched dimension, and a second one when a pure orb and a hexorb fused.
    pub dim: usize,
    pub fused: Option<usize>,
    /// Orb power in percent: 100 for a pure orb, the Hexcore's power for a hexorb.
    pub power: i64,
    /// Tech module of the hexorb that powered it.
    pub module: Option<usize>,
    /// Packet-only zone effects picked up from rifts it passed (True Rift, Echo, Prism, ...).
    pub picked: Vec<usize>,
    pub true_pct: i64,
    pub ignore_wall: bool,
    pub kaleido: bool,
    /// The family-pair bonus this punch's orb completed.
    pub twist: Option<usize>,
    pub source: &'static str,
    pub base: i64,
    /// When its in-flight sprite runs out and is drawn again.
    pub drawn: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Later {
    /// A punch copy landing on a target (Echo, Paradox).
    Repeat { at: usize, target: usize, amount: i64 },
    /// The next rift in a Domino collapses.
    Domino { at: usize, uid: u32 },
    /// One Rift Rain drop.
    Drop { at: usize, x: i64, y: i64, amount: i64 },
    /// A swallowed fighter reappears (Kidnap, Seed, Sinkhole, Hook) at the exit.
    Spit { at: usize, target: usize, x: i64, y: i64, ally: bool },
    /// Vesper leaves a Phase Out.
    Unphase { at: usize, x: i64, y: i64 },
    /// A Sky Drop landing stun.
    Land { at: usize, x: i64, y: i64, ally: usize },
    /// Kidnap's armed stun field resolves: caught, both drop through to the exit; dodged, it fizzles.
    Kidnap { at: usize, target: usize, x: i64, y: i64, exit: (i64, i64), dim: usize, radius: i64 },
    /// A Pressure burst: every enemy within PRESSURE_RADIUS takes `pm` per mille of max HP.
    Burst { at: usize, x: i64, y: i64, pm: i64 },
}
impl Later {
    fn at(&self) -> usize {
        match self {
            Later::Repeat { at, .. }
            | Later::Domino { at, .. }
            | Later::Drop { at, .. }
            | Later::Spit { at, .. }
            | Later::Unphase { at, .. }
            | Later::Land { at, .. }
            | Later::Kidnap { at, .. }
            | Later::Burst { at, .. } => *at,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Dot {
    pub target: usize,
    pub per_pulse: i64,
    pub until: usize,
    pub drain: bool,
}

/// Counters the fixtures and the Rift Lab report per rank.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Stats {
    pub tech_uses: [u32; 28],
    /// The 6 gauntlet moves (Swipe, Shatter, Overload, Vent, Awaken, Overclock).
    pub move_uses: [u32; 6],
    pub rifts_opened: u32,
    pub rifts_touching: u32,
    pub breaks: u32,
    pub punches: u32,
    /// Punches that landed (their efficiency is summed in `eff_sum`).
    pub landed: u32,
    pub swipes: u32,
    pub swipe_hits: u32,
    pub eff_sum: i64,
    pub damage: i64,
    pub orbs: u32,
    pub hexorbs: u32,
    pub fused: u32,
    pub twists: u32,
    pub best_combo: u32,
    pub shatters: u32,
    pub useful_shatters: u32,
    pub left_breaks: u32,
    pub right_breaks: u32,
    pub overloads: u32,
    pub awakens: u32,
    pub good_awakens: u32,
    pub takedowns: u32,
    pub kidnaps: u32,
    pub useful_kidnaps: u32,
    /// Kidnap fields the target walked out of before they closed.
    pub kidnap_dodges: u32,
    pub gates: u32,
    pub engaged_gates: u32,
    pub lonely_gates: u32,
    pub traps: u32,
    pub trap_catches: u32,
    pub risky_moves: u32,
    pub self_deaths: u32,
    pub implodes: u32,
    pub implode_hits: u32,
    /// View effects actually started (the game's cost: each is a new animated sprite).
    pub effects: u32,
    pub habits: BTreeMap<&'static str, u32>,
    /// Her plain jabs' damage (counted by the host that lands them), part of `damage`.
    pub jab_damage: i64,
    /// Every dimension an orb opened (variety across a fight).
    pub dims: std::collections::BTreeSet<usize>,
    /// Planner picks and the wall time they took (test builds only: the core stays deterministic).
    pub picks: u32,
    pub pick_nanos: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RiftSpec {
    pub kind: usize,
    pub x: i64,
    pub y: i64,
    pub ex: i64,
    pub ey: i64,
    pub host: Option<usize>,
    pub chain: bool,
    pub entrance: bool,
    /// Spine upkeep: a refreshed node keeps the chain position of the node it replaces.
    pub order: Option<u32>,
    /// The dimension to open (the Rift Lab sets it; the planner picks one when it is None).
    pub dim: Option<usize>,
}
impl RiftSpec {
    pub fn at(kind: usize, x: i64, y: i64) -> RiftSpec {
        RiftSpec { kind, x, y, ex: x, ey: y, host: None, chain: false, entrance: false, order: None, dim: None }
    }
    pub fn line(kind: usize, x: i64, y: i64, ex: i64, ey: i64) -> RiftSpec {
        RiftSpec { ex, ey, ..RiftSpec::at(kind, x, y) }
    }
    pub fn node(kind: usize, x: i64, y: i64) -> RiftSpec {
        RiftSpec { chain: true, ..RiftSpec::at(kind, x, y) }
    }
    pub fn entrance(kind: usize, x: i64, y: i64) -> RiftSpec {
        RiftSpec { entrance: true, ..RiftSpec::at(kind, x, y) }
    }
}
/// A planned technique or gauntlet move: the rifts to open plus what they are for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Action {
    pub tech: usize,
    pub rifts: Vec<RiftSpec>,
    pub subject: Option<usize>,
    /// Existing rifts the technique acts on (Implode, Domino, Stitch, Crossfire, Infuse).
    pub uses: Vec<u32>,
    /// Where a swipe is aimed.
    pub point: Option<(i64, i64)>,
    /// The swipe's dimension (the Rift Lab sets it; the planner picks one when it is None).
    pub dim: Option<usize>,
    pub score: i64,
    pub habit: &'static str,
}
impl Action {
    pub fn new(tech: usize, rifts: Vec<RiftSpec>) -> Action {
        Action { tech, rifts, subject: None, uses: vec![], point: None, dim: None, score: 0, habit: "" }
    }
}
/// What an orb's dimension is being opened for: the planner values dimensions differently for each.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Purpose {
    Punch,
    Swipe,
    /// A rift on or near enemies.
    Enemy,
    /// A rift for allies (gate entrances, rescues, pockets).
    Ally,
    /// A chain node punches pass through.
    Route,
}

#[derive(Clone, Debug)]
pub struct Core {
    pub rank: usize,
    pub top: Option<usize>,
    pub rng: u64,
    /// Left gauntlet: pure orb sockets.
    pub pure: [Socket; 9],
    /// Right gauntlet: hexorb slots (ready tick) and the athlete's tech module per slot.
    pub hex: [usize; 6],
    pub modules: [usize; 6],
    /// Hexcore completion in percent (70..100) and its Awakening / corruption.
    pub completion: i64,
    pub awaken_until: usize,
    pub corruption: i64,
    pub left_broken_until: usize,
    pub right_broken_until: usize,
    pub stress: i64,
    /// Orb combo: steps in the current chain of activations, when the last orb fired and its family.
    pub combo: i64,
    pub combo_at: usize,
    pub last_family: Option<usize>,
    /// Static marks: enemy -> (until, percent of the next punch arcing to it).
    pub marks: BTreeMap<usize, (usize, i64)>,
    /// Frost: chill stacks per enemy and the tick of the last chill (they fade CHILL_FADE ticks after it).
    pub chill: BTreeMap<usize, (i64, usize)>,
    /// Radiance: exposed enemies (until, per mille): her next punch, swipe or implode on them detonates it.
    pub exposed: BTreeMap<usize, (usize, i64)>,
    pub rifts: Vec<Rift>,
    pub next_uid: u32,
    pub chain_seq: u32,
    pub packets: Vec<Packet>,
    later: Vec<Later>,
    pub dots: Vec<Dot>,
    /// Every 6 ticks: position and HP (Rewind, recall).
    pub history: BTreeMap<usize, VecDeque<(i64, i64, i64)>>,
    pub velocity: BTreeMap<usize, (i64, i64)>,
    damage: BTreeMap<usize, Window>,
    controls: BTreeMap<usize, Window>,
    moves: BTreeMap<usize, Window>,
    support: BTreeMap<usize, Window>,
    heals: BTreeMap<usize, Window>,
    pub cooldowns: BTreeMap<(u32, usize), usize>,
    pub self_move_until: usize,
    pub next_attack: usize,
    /// Rift punching (her basic attacks are punches) and aiming through a chain (feet planted).
    pub punching: bool,
    pub aiming: bool,
    /// Visual only (the pose overlays): the tick each one-shot pose ends. Never read by the simulation.
    pub jab_until: usize,
    pub block_until: usize,
    pub swipe_until: usize,
    pub shatter_until: usize,
    pub chain_target: Option<usize>,
    pub crossfire_until: usize,
    /// Overload Punch: bonus percent for the next punch.
    pub opunch: i64,
    /// The Rift Lab's chosen punch dimension (hand placement).
    pub armed: Option<usize>,
    pub plan: VecDeque<usize>,
    pub plan_until: usize,
    /// The techniques she used recently (technique, tick), for variety: repeats lose score, then are capped.
    pub recent: Vec<(usize, usize)>,
    pub last_think: usize,
    pub phased: bool,
    pub risky_until: usize,
    pub displaced: BTreeMap<usize, usize>,
    /// Enemies Vesper hurt recently (takedown research) and the HP she last saw on them.
    pub hurt: BTreeMap<usize, (usize, i64, i64)>,
    pub last_hp: i64,
    /// A displacement this tick was stopped short by a wall (Gale's wall slam).
    slammed: bool,
    pub stats: Stats,
    pub fx_left: usize,
    /// The effect budget across ticks: a token bucket (FX_TOKENS, one back every FX_REFILL ticks). Rift layers,
    /// links and packets only spend above FX_RESERVE, so opens, closes, hits and casts always show.
    pub fx_tokens: usize,
    /// Kidnap is ready again from this tick (one cast per KIDNAP_CD).
    pub kidnap_ready: usize,
    pub last_tick: Option<usize>,
    pub hand_placed: bool,
    /// Each dimension is ready again from this tick, and each family (FAMILY_RECOVERY after a cast). Both gauntlets
    /// share them; one cast's rifts and a fused punch's two orbs lock together.
    pub dim_ready: Vec<usize>,
    pub family_ready: Vec<usize>,
    /// The Rift Lab's "ignore cooldowns" (and effect tests): casts never lock.
    pub free_casts: bool,
}

impl Core {
    pub fn new(seed: u64, owner: usize, rank: usize, top: Option<usize>) -> Core {
        let mut pure = [Socket { family: 255, ..Socket::default() }; 9];
        for s in pure.iter_mut().take(ORBS_BASE as usize) {
            s.open = true;
        }
        Core {
            rank: rank.min(7),
            top,
            rng: (seed ^ (owner as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15) ^ 0x5645_5350_4552).max(1),
            pure,
            hex: [0; 6],
            modules: modules_of(None),
            completion: HEX_START,
            awaken_until: 0,
            corruption: 0,
            left_broken_until: 0,
            right_broken_until: 0,
            stress: 0,
            combo: 0,
            combo_at: 0,
            last_family: None,
            marks: BTreeMap::new(),
            chill: BTreeMap::new(),
            exposed: BTreeMap::new(),
            rifts: vec![],
            next_uid: 1,
            chain_seq: 0,
            packets: vec![],
            later: vec![],
            dots: vec![],
            history: BTreeMap::new(),
            velocity: BTreeMap::new(),
            damage: BTreeMap::new(),
            controls: BTreeMap::new(),
            moves: BTreeMap::new(),
            support: BTreeMap::new(),
            heals: BTreeMap::new(),
            cooldowns: BTreeMap::new(),
            self_move_until: 0,
            next_attack: 0,
            punching: false,
            aiming: false,
            jab_until: 0,
            block_until: 0,
            swipe_until: 0,
            shatter_until: 0,
            chain_target: None,
            crossfire_until: 0,
            opunch: 0,
            armed: None,
            plan: VecDeque::new(),
            plan_until: 0,
            recent: Vec::new(),
            last_think: 0,
            phased: false,
            risky_until: 0,
            displaced: BTreeMap::new(),
            hurt: BTreeMap::new(),
            last_hp: -1,
            slammed: false,
            stats: Stats::default(),
            fx_left: FX_PER_UPDATE as usize,
            fx_tokens: FX_TOKENS as usize,
            kidnap_ready: 0,
            last_tick: None,
            hand_placed: false,
            dim_ready: vec![0; DIMS.len()],
            family_ready: vec![0; FAMILIES.len()],
            free_casts: false,
        }
    }
    fn random(&mut self, n: u64) -> u64 {
        roll(&mut self.rng, n)
    }
    pub fn look(&self) -> usize {
        look(self.rank, self.top)
    }
    pub fn table(&self, t: &[i64; 8]) -> i64 {
        t[self.rank.min(7)]
    }
    pub fn gate(&self, g: usize) -> bool {
        self.rank >= g
    }
    fn fx(&mut self, out: &mut Vec<Cmd>, tag: String, x: i64, y: i64, unit: Option<usize>, ticks: usize) -> bool {
        if self.fx_left > 0 && self.fx_tokens > 0 {
            self.fx_left -= 1;
            self.fx_tokens -= 1;
            self.stats.effects += 1;
            out.push(Cmd::Fx { tag, x, y, unit, ticks });
            return true;
        }
        false
    }
    /// Ambient visuals (rift layers, links, packets in flight): only from the tokens above the reserve.
    fn fx_low(&mut self, out: &mut Vec<Cmd>, tag: String, x: i64, y: i64, unit: Option<usize>, ticks: usize) -> bool {
        self.fx_tokens > FX_RESERVE as usize && self.fx(out, tag, x, y, unit, ticks)
    }
    fn habit(&mut self, h: &'static str) {
        if !h.is_empty() && self.rank >= habit_gate(h) {
            *self.stats.habits.entry(h).or_default() += 1;
        }
    }
    pub fn rift(&self, uid: u32) -> Option<&Rift> {
        self.rifts.iter().find(|r| r.uid == uid && !r.done)
    }
    fn rift_mut(&mut self, uid: u32) -> Option<&mut Rift> {
        self.rifts.iter_mut().find(|r| r.uid == uid && !r.done)
    }
    pub fn open_count(&self) -> usize {
        self.rifts.iter().filter(|r| !r.done).count()
    }
    fn cooldown_ok(&self, key: u32, id: usize, tick: usize) -> bool {
        self.cooldowns.get(&(key, id)).is_none_or(|until| tick >= *until)
    }
    fn set_cooldown(&mut self, key: u32, id: usize, until: usize) {
        self.cooldowns.insert((key, id), until);
    }

    // ---------------------------------------------------------------- shared limits
    /// All of Vesper's damage: 35% of the target's max HP per 2 s.
    fn emit(&mut self, out: &mut Vec<Cmd>, f: &Fighter, amount: i64, kind: Dmg, tick: usize) -> i64 {
        if amount <= 0 || f.hp <= 0 {
            return 0;
        }
        let cap = (f.max_hp * DAMAGE_PCT / 100).max(1) as usize;
        let n = self.damage.entry(f.id).or_default().allow(tick, DAMAGE_WINDOW as usize, cap, amount as usize) as i64;
        if n > 0 {
            out.push(Cmd::Damage { target: f.id, amount: n, kind });
            self.stats.damage += n;
            self.hurt.insert(f.id, (tick, f.hp - n, f.max_hp));
        }
        n
    }
    /// Stuns, roots, silences, disarms and banishes share 60 ticks per 3 s per target.
    fn control(&mut self, out: &mut Vec<Cmd>, f: &Fighter, kind: Option<Cc>, ticks: i64, tick: usize) -> bool {
        if f.cc_immune || ticks <= 0 {
            return false;
        }
        let n = self.controls.entry(f.id).or_default().allow(tick, CONTROL_WINDOW as usize, CONTROL_CAP as usize, ticks as usize);
        if n == 0 {
            return false;
        }
        match kind {
            Some(k) => out.push(Cmd::Cc { target: f.id, kind: k, ticks: n }),
            None => out.push(Cmd::Banish { target: f.id, ticks: n }),
        }
        true
    }
    /// Forced movement: two per 2 s per target, never for crowd-control immune enemies.
    #[allow(clippy::too_many_arguments)]
    fn displace(&mut self, out: &mut Vec<Cmd>, f: &Fighter, x: i64, y: i64, tick: usize, wall: &dyn Fn(i64, i64) -> bool, enemy: bool) -> bool {
        if enemy && f.cc_immune {
            return false;
        }
        if self.moves.entry(f.id).or_default().allow(tick, MOVE_WINDOW as usize, MOVE_CAP as usize, 1) == 0 {
            return false;
        }
        let want = clamp_map(x, y);
        let (x, y) = land(f.x, f.y, x, y, wall);
        self.slammed = (x, y) != want;
        out.push(Cmd::SetPos { target: f.id, x, y });
        self.displaced.insert(f.id, tick);
        true
    }
    fn shield(&mut self, out: &mut Vec<Cmd>, f: &Fighter, amount: i64, ticks: usize, tick: usize) {
        let cap = (f.max_hp * SUPPORT_PCT / 100).max(1) as usize;
        let n = self.support.entry(f.id).or_default().allow(tick, SUPPORT_WINDOW as usize, cap, amount.max(0) as usize);
        if n > 0 {
            out.push(Cmd::Shield { target: f.id, amount: n as i64, ticks });
        }
    }
    /// Every heal she gives: HEAL_PCT% of the target's max HP per HEAL_WINDOW ticks.
    pub(crate) fn heal(&mut self, out: &mut Vec<Cmd>, t: &Tick, target: usize, amount: i64, tick: usize) {
        let Some(max_hp) = t.get(target).map(|f| f.max_hp) else { return };
        let cap = (max_hp * HEAL_PCT / 100).max(1) as usize;
        let n = self.heals.entry(target).or_default().allow(tick, HEAL_WINDOW as usize, cap, amount.max(0) as usize);
        if n > 0 {
            out.push(Cmd::Heal { target, amount: n as i64 });
        }
    }
    fn buff(&self, out: &mut Vec<Cmd>, target: usize, name: String, ticks: i64, mods: Mods) {
        if ticks > 0 {
            out.push(Cmd::Buff { target, name, ticks: ticks as usize, mods });
        }
    }
}
/// The rank from which a habit label is credited (the Rift Lab labels and the fixtures read these).
pub fn habit_gate(h: &str) -> usize {
    match h {
        "led target" => 1,
        "collapsed a rift before it broke" | "shattered while the gauntlet held" | "stopped overloading near enemies" => GATE_SAFETY,
        "solved the link count" | "implode on a pile" | "rain on a cluster" | "crossfire split" | "domino through the pile" | "bank shot around a wall" | "swipe through the line" => GATE_GEOMETRY,
        "Lifeline set before the fight" | "peeled a diver" | "saved an ally" | "rescued ally" | "phased out of a burst" | "escaped a dive" | "vented overload when dived" | "kept a spare orb" => GATE_SURVIVAL,
        "Escort on the diver" | "Gate synced with the tank" | "Fling toward the team" | "swapped the target into the team" | "dropped aim for a diver" | "Awakening timed with the engage" | "rotated an ally into the fight" => GATE_TEAM,
        "Kidnapped the carry into the team" | "Sinkhole at the chokepoint" | "Seed on the carry's path" | "stitched two worlds" | "dimension picked for the target" | "Infused the frontliner's rift" => GATE_PRIORITY,
        "spine refreshed" | "overload ridden into a Shatter" | "overclock stopped in time" | "combo families for the twist" | "planned a 5-step sequence" | "overloaded before the engage" => GATE_SPINE,
        "planned 2 steps ahead" => 3,
        "planned 3 steps ahead" => 5,
        "planned 4 steps ahead" => 6,
        _ => 0,
    }
}
/// A landing spot: inside the map and out of walls (walked back toward where the fighter stood).
pub fn land(fx: i64, fy: i64, x: i64, y: i64, wall: &dyn Fn(i64, i64) -> bool) -> (i64, i64) {
    // no teleport carries anyone farther than TELEPORT_MAX
    let (x, y) = if dist(fx, fy, x, y) > TELEPORT_MAX { along(fx, fy, x - fx, y - fy, TELEPORT_MAX) } else { (x, y) };
    let (x, y) = clamp_map(x, y);
    if !wall(x, y) {
        return (x, y);
    }
    let steps = (dist(fx, fy, x, y) / 3000).max(1);
    for i in 0..=steps {
        let (px, py) = (x + (fx - x) * i / steps, y + (fy - y) * i / steps);
        if !wall(px, py) {
            return (px, py);
        }
    }
    (fx, fy)
}

/// The tick being simulated: who is where.
pub struct Tick<'a> {
    pub tick: usize,
    pub me: Fighter,
    pub fighters: &'a [Fighter],
    pub wall: &'a dyn Fn(i64, i64) -> bool,
    pub attack_interval: usize,
    pub stunned: bool,
}
impl<'a> Tick<'a> {
    pub fn get(&self, id: usize) -> Option<&Fighter> {
        self.fighters.iter().find(|f| f.id == id && f.hp > 0)
    }
    pub fn enemies(&self) -> impl Iterator<Item = &Fighter> {
        let team = self.me.team;
        self.fighters.iter().filter(move |f| f.team != team && f.hp > 0)
    }
    pub fn allies(&self) -> impl Iterator<Item = &Fighter> {
        let (team, me) = (self.me.team, self.me.id);
        self.fighters.iter().filter(move |f| f.team == team && f.id != me && f.hp > 0)
    }
    pub fn friends(&self) -> impl Iterator<Item = &Fighter> {
        let team = self.me.team;
        self.fighters.iter().filter(move |f| f.team == team && f.hp > 0)
    }
    pub fn danger(&self, x: i64, y: i64, r: i64) -> usize {
        self.enemies().filter(|e| dist2(e.x, e.y, x, y) <= (r as i128) * (r as i128)).count()
    }
}
/// Sixteen unit directions (x10000) for seeded shoves.
const DIRS: [(i64, i64); 16] = [
    (10000, 0), (9239, 3827), (7071, 7071), (3827, 9239), (0, 10000), (-3827, 9239), (-7071, 7071), (-9239, 3827),
    (-10000, 0), (-9239, -3827), (-7071, -7071), (-3827, -9239), (0, -10000), (3827, -9239), (7071, -7071), (9239, -3827),
];
pub fn heading8(dx: i64, dy: i64) -> usize {
    if dx == 0 && dy == 0 {
        return 0;
    }
    const D8: [(i64, i64); 8] = [(10000, 0), (7071, 7071), (0, 10000), (-7071, 7071), (-10000, 0), (-7071, -7071), (0, -10000), (7071, -7071)];
    D8.iter()
        .enumerate()
        .max_by_key(|(_, (x, y))| *x as i128 * dx as i128 + *y as i128 * dy as i128)
        .map_or(0, |(i, _)| i)
}
/// The point of segment a-b nearest to (x, y).
pub fn nearest_on(ax: i64, ay: i64, bx: i64, by: i64, x: i64, y: i64) -> (i64, i64) {
    let (dx, dy) = ((bx - ax) as i128, (by - ay) as i128);
    let len = dx * dx + dy * dy;
    if len == 0 {
        return (ax, ay);
    }
    let t = (((x - ax) as i128) * dx + ((y - ay) as i128) * dy).clamp(0, len);
    (ax + (dx * t / len) as i64, ay + (dy * t / len) as i64)
}
/// How a dimension is delivered: its effect's centre and who counts as its target.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Src {
    Zone,
    /// A rift punch's hole: displacements are measured from Vesper.
    Punch,
    /// A swipe's tear line: displacements are measured from the line (`c`), the line's ends ride along.
    Swipe,
    /// Packet-only pickups and techniques that reuse a rift's dimension at a point.
    Imbue,
    /// A Shatter ripping a rift wide (enemies only).
    Shatter,
}
/// The swipe line a Swipe delivery acts on.
#[derive(Clone, Copy)]
struct Line {
    ax: i64,
    ay: i64,
    bx: i64,
    by: i64,
}

impl Core {
    // ---------------------------------------------------------------- effects
    /// Momentum: +a% per 10000 units Vesper moved in the last MOMENTUM_WINDOW ticks, at most +b%.
    pub(crate) fn momentum(&self, t: &Tick, a: i64, b: i64) -> i64 {
        let back = (MOMENTUM_WINDOW / 6).max(1) as usize;
        let Some(h) = self.history.get(&t.me.id) else { return 0 };
        if h.len() <= back {
            return 0;
        }
        let (x, y, _) = h[h.len() - 1 - back];
        (dist(x, y, t.me.x, t.me.y) * a / 10000).min(b)
    }
    /// Radiance: a punch, swipe or implode on an exposed enemy detonates the mark.
    fn detonate(&mut self, out: &mut Vec<Cmd>, f: &Fighter, tick: usize) {
        if let Some((until, pm)) = self.exposed.get(&f.id).copied() {
            self.exposed.remove(&f.id);
            if tick <= until {
                self.emit(out, f, f.max_hp * pm / 1000, Dmg::Magic, tick);
                let look = self.look();
                self.fx(out, format!("hole_l{look}"), f.x, f.y, None, 12);
            }
        }
    }
    /// Pressure: a rift that stored charges bursts on every enemy inside when it closes or implodes.
    fn release_pressure(&mut self, out: &mut Vec<Cmd>, t: &Tick, r: &Rift) {
        if r.charge <= 0 {
            return;
        }
        for dim in r.dims() {
            if DIMS[dim].prim != P_PRESSURE {
                continue;
            }
            let inside: Vec<Fighter> = t.enemies().filter(|e| r.covers(e.x, e.y)).cloned().collect();
            for e in inside {
                self.emit(out, &e, e.max_hp * DIMS[dim].a * r.charge / 1000, Dmg::Magic, t.tick);
            }
            let tier = self.tier4();
            self.fx(out, format!("tech_implode_t{tier}"), r.x, r.y, None, 24);
        }
    }
    /// Dice: a dimension from any other family, uniformly (dimensions are listed family by family).
    fn other_family_dim(&mut self, family: usize) -> usize {
        let mut other = self.random((N_DIMS - FAMILY_LEN[family]) as u64) as usize;
        if other >= FAMILY_START[family] {
            other += FAMILY_LEN[family];
        }
        other
    }
    /// A rift's zone effect: the dimension's own primitive and numbers.
    #[allow(clippy::too_many_arguments)]
    fn apply_dim(&mut self, out: &mut Vec<Cmd>, t: &Tick, dim: usize, f: &Fighter, s: i64, c: (i64, i64, i64), src: Src) {
        let d = DIMS[dim];
        if d.prim == P_DICE {
            let other = self.other_family_dim(d.family);
            if DIMS[other].prim != P_DICE {
                self.apply_dim(out, t, other, f, s, c, src);
            }
            return;
        }
        self.apply_eff(out, t, dim, Eff { prim: d.prim, a: d.a, b: d.b, c: d.c }, f, s, c, src, None);
    }
    /// Applies one effect to one fighter. `s` is the strength in percent; `c` is the effect's centre and radius
    /// (the rift, Vesper for a punch, the nearest point of the tear line for a swipe).
    #[allow(clippy::too_many_arguments)]
    fn apply_eff(&mut self, out: &mut Vec<Cmd>, t: &Tick, dim: usize, d: Eff, f: &Fighter, s: i64, c: (i64, i64, i64), src: Src, line: Option<Line>) {
        let p = d.prim;
        if p == 255 {
            return;
        }
        let side = PRIM_SIDE[p as usize];
        let enemy = f.team != t.me.team;
        let me = f.id == t.me.id;
        let ok = match side {
            b'E' => enemy,
            b'A' => !enemy && src != Src::Shatter,
            b'S' => me && src != Src::Shatter,
            b'X' => src != Src::Shatter || enemy,
            _ => false,
        };
        if !ok {
            return;
        }
        let tick = t.tick;
        let sc = |v: i64| v * s / 100;
        let key = 1000 + p as u32;
        let name = format!("vsd_{}", DIMS[dim].id);
        let mut m = Mods::default();
        match p {
            P_BANISH => {
                if self.cooldown_ok(key, f.id, tick) && self.control(out, f, None, sc(d.a).max(12), tick) {
                    self.set_cooldown(key, f.id, tick + 480);
                }
            }
            P_STUN | P_ROOT | P_SILENCE | P_DISARM | P_STATICBURST => {
                let kind = match p {
                    P_STUN | P_STATICBURST => Cc::Stun,
                    P_ROOT => Cc::Root,
                    P_SILENCE => Cc::Silence,
                    _ => Cc::Disarm,
                };
                let ticks = if p == P_STATICBURST { self.random(d.a as u64 + 1) as i64 } else { sc(d.a).max(8) };
                if self.cooldown_ok(key, f.id, tick) && self.control(out, f, Some(kind), ticks, tick) {
                    self.set_cooldown(key, f.id, tick + 240);
                }
            }
            P_SLOW => {
                m.move_speed = -(sc(d.a) as i32);
                m.attack_speed = -(sc(d.b) as i32);
                self.buff(out, f.id, name, d.c, m);
            }
            P_AMPLIFY => {
                m.amplify = sc(d.a) as i32;
                self.buff(out, f.id, name, d.b, m);
            }
            P_DOT | P_DRAIN | P_ENTROPY => {
                let per_mille = if p == P_ENTROPY { self.random(d.a as u64 + 1) as i64 } else { d.a };
                let per_pulse = f.max_hp * sc(per_mille) / 2000;
                if src != Src::Zone && d.b > 0 {
                    self.dots.retain(|x| !(x.target == f.id && x.drain == (p == P_DRAIN)));
                    self.dots.push(Dot { target: f.id, per_pulse, until: tick + d.b as usize, drain: p == P_DRAIN });
                } else {
                    let n = self.emit(out, f, per_pulse, Dmg::Magic, tick);
                    if p == P_DRAIN && n > 0 {
                        self.heal(out, t, t.me.id, n, tick);
                    }
                }
            }
            P_SHATTER => {
                if self.cooldown_ok(key, f.id, tick) {
                    out.push(Cmd::ClearShield { target: f.id });
                    self.set_cooldown(key, f.id, tick + 240);
                }
            }
            P_STATDOWN | P_STATUP => {
                let v = sc(d.b) as i32 * if p == P_STATDOWN { -1 } else { 1 };
                match d.a {
                    0 => m.attack = v,
                    1 => m.defence = v,
                    2 => m.mr = v,
                    3 => m.ap = v,
                    _ => {
                        if p == P_STATDOWN {
                            m.regen = -(d.b as i32 / 10).max(1)
                        } else {
                            m.crit = v
                        }
                    }
                }
                self.buff(out, f.id, name, d.c, m);
            }
            P_REFLECT | P_PULL | P_KNOCK | P_SHOVE | P_STORM | P_GRAB | P_REWIND | P_TOEND => {
                if !self.cooldown_ok(key, f.id, tick) {
                    return;
                }
                let (cx, cy) = (c.0, c.1);
                let to = match p {
                    P_REFLECT => (2 * cx - f.x, 2 * cy - f.y),
                    P_PULL => lerp(f.x, f.y, cx, cy, sc(d.a).min(100)),
                    P_KNOCK => {
                        let (dx, dy) = if (f.x, f.y) != (cx, cy) {
                            (f.x - cx, f.y - cy)
                        } else if let Some(l) = line {
                            (-(l.by - l.ay), l.bx - l.ax)
                        } else {
                            (1, 0)
                        };
                        along(f.x, f.y, dx, dy, sc(d.a))
                    }
                    P_SHOVE | P_STORM => {
                        let mut v = (0i64, 0i64);
                        for _ in 0..if p == P_STORM { 2 } else { 1 } {
                            let (ux, uy) = DIRS[self.random(16) as usize];
                            v.0 += ux * sc(d.a) / 10000;
                            v.1 += uy * sc(d.a) / 10000;
                        }
                        (f.x + v.0, f.y + v.1)
                    }
                    P_GRAB => along(f.x, f.y, t.me.x - f.x, t.me.y - f.y, sc(d.a).min(dist(f.x, f.y, t.me.x, t.me.y))),
                    P_TOEND => match line {
                        Some(l) => (l.bx, l.by),
                        None => return,
                    },
                    _ => {
                        let Some(h) = self.history.get(&f.id) else { return };
                        let back = (d.a as usize * s.clamp(30, 100) as usize / 100 / 6).max(1);
                        if h.len() <= back {
                            return;
                        }
                        let (x, y, _) = h[h.len() - 1 - back];
                        (x, y)
                    }
                };
                if self.displace(out, f, to.0, to.1, tick, t.wall, enemy) {
                    self.set_cooldown(key, f.id, tick + if p == P_REWIND { 480 } else { 240 });
                }
            }
            P_WALLSLAM => {
                if self.slammed {
                    self.control(out, f, Some(Cc::Stun), sc(d.a).max(8), tick);
                }
            }
            P_SELFSWAP => {
                if tick >= self.self_move_until && !f.cc_immune && self.displace(out, f, t.me.x, t.me.y, tick, t.wall, true) {
                    self.self_move_until = tick + SELF_MOVE_CD as usize;
                    let (x, y) = land(t.me.x, t.me.y, f.x, f.y, t.wall);
                    out.push(Cmd::SetPos { target: t.me.id, x, y });
                }
            }
            P_BLINK => {
                if tick >= self.self_move_until {
                    self.self_move_until = tick + SELF_MOVE_CD as usize;
                    let (x, y) = land(t.me.x, t.me.y, along(t.me.x, t.me.y, c.0 - t.me.x, c.1 - t.me.y, sc(d.a)).0, along(t.me.x, t.me.y, c.0 - t.me.x, c.1 - t.me.y, sc(d.a)).1, t.wall);
                    out.push(Cmd::SetPos { target: t.me.id, x, y });
                }
            }
            P_MARK => {
                self.marks.insert(f.id, (tick + d.b as usize, sc(d.a)));
            }
            P_ENCASE => {
                if self.control(out, f, Some(Cc::Root), sc(d.a).max(10), tick) {
                    m.reduce = d.b as i32;
                    self.buff(out, f.id, name, sc(d.a).max(10), m);
                }
            }
            P_RECALLHP => {
                let back = (d.a as usize / 6).max(1);
                if let Some(h) = self.history.get(&f.id) {
                    if h.len() > back {
                        let then = h[h.len() - 1 - back].2;
                        let lost = (then - f.hp).min(f.max_hp * d.b / 1000);
                        if lost > 0 && self.cooldown_ok(key, f.id, tick) {
                            self.heal(out, t, f.id, sc(lost), tick);
                            self.set_cooldown(key, f.id, tick + 600);
                        }
                    }
                }
            }
            P_CDUP | P_CDDOWN => {
                m.cooldown = sc(d.a) as i32 * if p == P_CDUP { 1 } else { -1 };
                self.buff(out, f.id, name, d.b, m);
            }
            P_HEALCUT => {
                m.heal_reduce = sc(d.a) as i32;
                self.buff(out, f.id, name, d.b, m);
            }
            P_SHELTER => {
                m.reduce = sc(d.a) as i32;
                self.buff(out, f.id, name, d.b, m);
            }
            P_HEAL => {
                if d.b == 0 {
                    let k = 3000 + dim as u32;
                    if self.cooldown_ok(k, f.id, tick) {
                        self.heal(out, t, f.id, f.max_hp * sc(d.a) / 1000, tick);
                        self.set_cooldown(k, f.id, tick + 600);
                    }
                } else {
                    self.heal(out, t, f.id, f.max_hp * sc(d.a) / 2000, tick);
                }
            }
            P_SHIELD => {
                if self.cooldown_ok(key, f.id, tick) {
                    self.shield(out, f, f.max_hp * sc(d.a) / 1000, d.b as usize, tick);
                    self.set_cooldown(key, f.id, tick + 180);
                }
            }
            P_UNDYING | P_CCIMMUNE => {
                if self.cooldown_ok(key, f.id, tick) {
                    if p == P_UNDYING {
                        m.undying = true;
                    } else {
                        m.cc_immune = true;
                    }
                    self.buff(out, f.id, name, sc(d.a).max(10), m);
                    self.set_cooldown(key, f.id, tick + IMMUNE_CD as usize);
                }
            }
            P_HASTE => {
                m.move_speed = sc(d.a) as i32;
                m.attack_speed = sc(d.b) as i32;
                self.buff(out, f.id, name, d.c, m);
            }
            P_RANGE => {
                m.range = sc(d.a) as i32;
                self.buff(out, f.id, name, d.b, m);
            }
            P_GHOST => {
                if self.cooldown_ok(key, f.id, tick) {
                    m.ignore_wall = true;
                    self.buff(out, f.id, name, d.a, m);
                    self.set_cooldown(key, f.id, tick + GHOST_CD as usize);
                }
            }
            P_CLEANSE => {
                if self.cooldown_ok(key, f.id, tick) {
                    out.push(Cmd::ClearCc { target: f.id });
                    self.set_cooldown(key, f.id, tick + 240);
                }
            }
            P_ALLYINVIS | P_SELFINVIS => {
                if self.cooldown_ok(key, f.id, tick) {
                    out.push(Cmd::Invisible { target: f.id, ticks: sc(d.a).max(10) as usize });
                    self.set_cooldown(key, f.id, tick + if p == P_ALLYINVIS { 600 } else { SELFINVIS_CD as usize });
                }
            }
            P_BADR => {
                m.ba_reduce = sc(d.a) as i32;
                self.buff(out, f.id, name, d.b, m);
            }
            P_REFLECTDMG => {
                m.reflect = sc(d.a) as i32;
                self.buff(out, f.id, name, d.b, m);
            }
            P_VAMP => {
                m.vamp = sc(d.a) as i32;
                self.buff(out, t.me.id, name, d.b, m);
            }
            P_TITHE => {
                if enemy && self.cooldown_ok(key, f.id, tick) {
                    out.push(Cmd::ClearShield { target: f.id });
                    self.set_cooldown(key, f.id, tick + 240);
                    let me = t.me.clone();
                    self.shield(out, &me, me.max_hp * TITHE_SHIELD / 100, 180, tick);
                }
            }
            P_WELL => {
                let to = lerp(f.x, f.y, c.0, c.1, sc(d.a).min(100));
                self.displace(out, f, to.0, to.1, tick, t.wall, enemy);
            }
            P_REPEL => {
                let (dx, dy) = if (f.x, f.y) != (c.0, c.1) {
                    (f.x - c.0, f.y - c.1)
                } else if let Some(l) = line {
                    (-(l.by - l.ay), l.bx - l.ax)
                } else {
                    (1, 0)
                };
                let to = along(f.x, f.y, dx, dy, sc(d.a));
                self.displace(out, f, to.0, to.1, tick, t.wall, enemy);
            }
            P_CHILL => {
                let (mut n, last) = self.chill.get(&f.id).copied().unwrap_or((0, 0));
                if tick > last + CHILL_FADE as usize {
                    n = 0;
                }
                n += 1;
                if n >= d.c.max(1) {
                    // frozen: the stacks clear
                    self.control(out, f, Some(Cc::Root), sc(d.a).max(8), tick);
                    n = 0;
                } else {
                    m.move_speed = -((sc(d.b) * n) as i32);
                    self.buff(out, f.id, name, 40, m);
                }
                self.chill.insert(f.id, (n, tick));
            }
            P_PRESSURE => {
                // a zone stores charges instead (step_zones) and bursts when it closes
                if src != Src::Zone {
                    self.later.push(Later::Burst { at: tick + d.b.max(1) as usize, x: f.x, y: f.y, pm: sc(d.a) });
                }
            }
            P_EXPOSE => {
                self.exposed.insert(f.id, (tick + d.b as usize, sc(d.a)));
            }
            P_COINFLIP => {
                if self.random(2) == 0 {
                    if enemy {
                        m.amplify = sc(d.a) as i32;
                        self.buff(out, f.id, name, 40, m);
                    }
                } else if !enemy {
                    m.reduce = sc(d.a) as i32;
                    self.buff(out, f.id, name, 40, m);
                }
            }
            _ => {}
        }
    }
    /// A hexorb's tech module going off at a point (a punch's hole, a rift opening, an Infused rift's pulse).
    fn module(&mut self, out: &mut Vec<Cmd>, t: &Tick, m: usize, x: i64, y: i64) {
        let md = MODULES[m];
        let tick = t.tick;
        let r = if self.completion >= HEX_RADIUS_AT { MODULE_RADIUS_UP } else { MODULE_RADIUS };
        let r2 = (r as i128) * (r as i128);
        let near_e: Vec<Fighter> = t.enemies().filter(|e| dist2(e.x, e.y, x, y) <= r2).cloned().collect();
        let near_a: Vec<Fighter> = t.friends().filter(|a| dist2(a.x, a.y, x, y) <= r2).cloned().collect();
        let name = format!("vsm_{}", md.id);
        match m {
            0 => {
                if let Some(a) = near_a.iter().min_by_key(|a| (a.hp * 100 / a.max_hp.max(1), a.id)).cloned() {
                    self.shield(out, &a, a.max_hp * md.a / 1000, md.b as usize, tick);
                }
            }
            1 => {
                for a in &near_a {
                    self.buff(out, a.id, name.clone(), md.b, Mods { move_speed: md.a as i32, ..Mods::default() });
                }
            }
            2 => {
                for e in &near_e {
                    self.buff(out, e.id, name.clone(), md.b, Mods { move_speed: -(md.a as i32), ..Mods::default() });
                }
            }
            3 => {
                for e in &near_e {
                    let to = lerp(e.x, e.y, x, y, md.a);
                    self.displace(out, e, to.0, to.1, tick, t.wall, true);
                }
            }
            4 => {
                for a in &near_a {
                    self.heal(out, t, a.id, a.max_hp * md.a / 1000, tick);
                }
            }
            5 => {
                for e in &near_e {
                    self.buff(out, e.id, name.clone(), md.b, Mods { amplify: md.a as i32, ..Mods::default() });
                }
            }
            6 => {
                let mut v = near_e.clone();
                v.sort_by_key(|e| (dist2(e.x, e.y, x, y), e.id));
                for e in v.iter().take(2) {
                    self.emit(out, e, md.a + t.me.attack * md.b / 100, Dmg::Magic, tick);
                }
            }
            _ => {
                if let Some(e) = near_e.iter().min_by_key(|e| (dist2(e.x, e.y, x, y), e.id)).cloned() {
                    self.control(out, &e, Some(Cc::Root), md.a, tick);
                }
            }
        }
        let tier = self.tier4();
        self.fx(out, format!("mod{m}_t{}", tier.min(3)), x, y, None, 18);
    }

    // ---------------------------------------------------------------- gauntlets
    pub fn left_broken(&self, tick: usize) -> bool {
        tick < self.left_broken_until
    }
    pub fn right_broken(&self, tick: usize) -> bool {
        tick < self.right_broken_until
    }
    pub fn awakened(&self, tick: usize) -> bool {
        tick < self.awaken_until
    }
    pub fn sockets(&self) -> usize {
        self.pure.iter().filter(|s| s.open).count()
    }
    pub fn over_sockets(&self) -> usize {
        self.pure.iter().filter(|s| s.open && s.over).count()
    }
    pub fn pure_ready(&self, tick: usize) -> usize {
        if self.left_broken(tick) {
            return 0;
        }
        self.pure.iter().filter(|s| s.open && tick >= s.ready_at).count()
    }
    pub fn hex_slots(&self, tick: usize) -> usize {
        if self.right_broken(tick) {
            0
        } else {
            hex_slots(self.completion)
        }
    }
    pub fn hex_ready(&self, tick: usize) -> usize {
        (0..self.hex_slots(tick)).filter(|i| tick >= self.hex[*i]).count()
    }
    pub fn orbs_ready(&self, tick: usize) -> usize {
        self.pure_ready(tick) + self.hex_ready(tick)
    }
    /// Hexorb power in percent: the Hexcore's completion, 100 while awakened, more while overclocked.
    pub fn hex_power(&self, tick: usize) -> i64 {
        if self.awakened(tick) {
            100 + CORRUPT_POWER * self.corruption / CORRUPT_PER_ORB
        } else {
            self.completion.min(100)
        }
    }
    fn hex_cd(&self) -> usize {
        if self.completion >= HEX_FAST_AT {
            HEX_CD_FAST as usize
        } else {
            HEX_CD as usize
        }
    }
    /// Takes one orb. `hex` prefers a hexorb; either way it falls back to the other gauntlet.
    pub fn take(&mut self, tick: usize, hex: bool) -> Option<Orb> {
        let take_pure = |c: &mut Core| -> Option<Orb> {
            if c.left_broken(tick) {
                return None;
            }
            // overloaded orbs heat while held, so they are spent first
            let i = (0..9).filter(|i| c.pure[*i].open && tick >= c.pure[*i].ready_at).max_by_key(|i| (c.pure[*i].over, 8 - *i))?;
            let over = c.pure[i].over;
            c.pure[i].ready_at = tick + if over { OVER_CD } else { ORB_CD } as usize;
            if !c.awakened(tick) {
                c.completion = (c.completion + RESEARCH_ORB).min(100);
            }
            c.stats.orbs += 1;
            Some(Orb { hex: None, over, socket: i })
        };
        let take_hex = |c: &mut Core| -> Option<Orb> {
            let n = c.hex_slots(tick);
            let i = (0..n).find(|i| tick >= c.hex[*i])?;
            c.hex[i] = tick + c.hex_cd();
            c.stats.hexorbs += 1;
            Some(Orb { hex: Some(i), over: false, socket: 0 })
        };
        if hex {
            take_hex(self).or_else(|| take_pure(self))
        } else {
            take_pure(self).or_else(|| take_hex(self))
        }
    }
    /// Remembers the family an orb opened (combo, twist, socket colour). Returns the combo multiplier (percent)
    /// and the family-pair twist this activation completes, if any.
    fn activate(&mut self, orb: Orb, dim: usize, tick: usize) -> (i64, Option<usize>) {
        let fam = DIMS[dim].family;
        let fresh = self.last_family.is_some() && tick <= self.combo_at + COMBO_WINDOW as usize;
        let twist = match (fresh, self.last_family) {
            (true, Some(prev)) if prev != fam => Some(twist_index(prev, fam)),
            _ => None,
        };
        self.combo = if fresh { (self.combo + 1).min(COMBO_MAX / COMBO_STEP) } else { 0 };
        self.stats.best_combo = self.stats.best_combo.max(self.combo as u32);
        self.combo_at = tick;
        self.last_family = Some(fam);
        self.stats.dims.insert(dim);
        if orb.hex.is_none() {
            self.pure[orb.socket].family = fam as u8;
        }
        (100 + self.combo * COMBO_STEP, twist)
    }
    /// Overload: forces an orb into the next free socket (6th to 9th).
    fn overload(&mut self, out: &mut Vec<Cmd>, t: &Tick) -> bool {
        let tick = t.tick;
        if self.left_broken(tick) {
            return false;
        }
        let Some(i) = (ORBS_BASE as usize..ORBS_MAX as usize).find(|i| !self.pure[*i].open) else { return false };
        self.pure[i] = Socket { open: true, over: true, ready_at: tick, family: 255 };
        self.stats.overloads += 1;
        self.add_stress(out, t, OVER_STRESS);
        let tier = self.tier4();
        self.fx(out, format!("overload_t{}", tier.min(3)), t.me.x, t.me.y, Some(t.me.id), 18);
        true
    }
    fn vent(&mut self) {
        for s in self.pure.iter_mut().skip(ORBS_BASE as usize) {
            *s = Socket { family: 255, ..Socket::default() };
        }
    }
    fn add_stress(&mut self, out: &mut Vec<Cmd>, t: &Tick, n: i64) {
        self.stress = (self.stress + n).max(0);
        if self.stress >= STRESS_CAP && !self.left_broken(t.tick) {
            out.push(Cmd::Log(format!("tick {} stress {} cracked the left gauntlet", t.tick, self.stress)));
            self.shatter(out, t, true);
        }
    }
    /// Shatter: every pure orb she holds is crushed; every open rift rips wide and swallows the enemies inside.
    /// Allies are never hurt (they are pushed out of the rip). Past the stress cap the left gauntlet breaks.
    fn shatter(&mut self, out: &mut Vec<Cmd>, t: &Tick, forced: bool) {
        let tick = t.tick;
        self.shatter_until = tick + 30;
        let held: Vec<usize> = (0..9).filter(|i| self.pure[*i].open && tick >= self.pure[*i].ready_at).collect();
        let over = held.iter().filter(|i| self.pure[**i].over).count() as i64;
        let n = held.len() as i64;
        let load = (n - over) * SHATTER_LOAD + over * SHATTER_LOAD_OVER;
        let amount = n.max(1) * (SHATTER_BASE + t.me.attack * SHATTER_AD / 100) * (100 + over * SHATTER_OVER_PCT) / 100;
        let tier = self.tier4();
        let mut hits = 0;
        let rifts: Vec<Rift> = self.rifts.iter().filter(|r| r.live() && !r.hidden(tick)).cloned().collect();
        let mut seen = BTreeSet::new();
        for r in &rifts {
            let reach = r.radius * 2;
            let inside: Vec<Fighter> = t.fighters.iter().filter(|f| f.hp > 0 && r.reach(f.x, f.y) <= reach).cloned().collect();
            for f in inside {
                if f.team == t.me.team {
                    if f.id != t.me.id {
                        let edge = along(r.x, r.y, f.x - r.x, f.y - r.y, reach + 6000);
                        let (x, y) = land(f.x, f.y, edge.0, edge.1, t.wall);
                        out.push(Cmd::SetPos { target: f.id, x, y });
                    }
                    continue;
                }
                if !seen.insert(f.id) {
                    continue;
                }
                hits += 1;
                self.control(out, &f, None, SHATTER_BANISH, tick);
                self.emit(out, &f, amount, Dmg::Phys, tick);
                for dim in r.dims() {
                    self.apply_dim(out, t, dim, &f, 100, (r.x, r.y, reach), Src::Shatter);
                }
            }
            self.fx(out, format!("shatter_t{tier}"), r.x, r.y, None, 30);
        }
        for i in held {
            if self.pure[i].over {
                self.pure[i] = Socket { family: 255, ..Socket::default() };
            } else {
                self.pure[i].ready_at = tick + ORB_CD as usize;
            }
        }
        for r in rifts {
            self.close(out, r.uid, "break");
        }
        self.stats.shatters += 1;
        if hits >= 2 {
            self.stats.useful_shatters += 1;
        }
        if !self.awakened(tick) {
            self.completion = (self.completion + RESEARCH_SHATTER).min(100);
        }
        self.stats.move_uses[M_SHATTER - 28] += u32::from(forced);
        out.push(Cmd::Log(format!("tick {tick} shatter orbs {n} overloaded {over} enemies {hits} stress {}+{load}{}", self.stress, if forced { " (forced)" } else { "" })));
        self.stress += load;
        if forced || self.stress > STRESS_CAP {
            self.left_break(out, t);
        }
    }
    fn left_break(&mut self, out: &mut Vec<Cmd>, t: &Tick) {
        let tick = t.tick;
        self.stats.left_breaks += 1;
        self.left_broken_until = tick + LEFT_BREAK_TICKS as usize;
        for (i, s) in self.pure.iter_mut().enumerate() {
            *s = if i < ORBS_BASE as usize { Socket { open: true, over: false, ready_at: tick + LEFT_BREAK_TICKS as usize, family: 255 } } else { Socket { family: 255, ..Socket::default() } };
        }
        out.push(Cmd::LoseHp { target: t.me.id, amount: t.me.max_hp * LEFT_BREAK_HP / 100 });
        out.push(Cmd::Cc { target: t.me.id, kind: Cc::Stun, ticks: BREAK_STAGGER as usize });
        out.push(Cmd::Log(format!("tick {tick} LEFT GAUNTLET BROKE (offline {} ticks)", LEFT_BREAK_TICKS)));
        self.stress = STRESS_CAP * 2 / 5;
        self.packets.retain(|p| p.module.is_some());
        let tier = self.tier4();
        self.fx(out, format!("gbreak_l_t{}", tier.min(3)), t.me.x, t.me.y, Some(t.me.id), 36);
    }
    fn awaken(&mut self, out: &mut Vec<Cmd>, t: &Tick) -> bool {
        let tick = t.tick;
        if self.completion < 100 || self.awakened(tick) || self.right_broken(tick) {
            return false;
        }
        self.awaken_until = tick + AWAKEN_TICKS as usize;
        self.corruption = 0;
        self.hex = [tick; 6];
        self.stats.awakens += 1;
        let fight = t.enemies().any(|e| dist(e.x, e.y, t.me.x, t.me.y) <= 90000) && t.allies().any(|a| t.danger(a.x, a.y, 60000) > 0);
        if fight {
            self.stats.good_awakens += 1;
        }
        let tier = self.tier4();
        self.fx(out, format!("awaken_t{tier}"), t.me.x, t.me.y, Some(t.me.id), AWAKEN_TICKS as usize);
        out.push(Cmd::Log(format!("tick {tick} HEXCORE AWAKENED{}", if fight { " in the fight" } else { "" })));
        true
    }
    /// Feeds a pure orb into an awakened Hexcore: stronger hexorbs, more corruption. At 100 the right gauntlet breaks.
    fn overclock(&mut self, out: &mut Vec<Cmd>, t: &Tick) -> bool {
        let tick = t.tick;
        if !self.awakened(tick) || self.pure_ready(tick) == 0 {
            return false;
        }
        self.take(tick, false);
        self.corruption += CORRUPT_PER_ORB;
        self.fx(out, "corrupt".into(), t.me.x, t.me.y, Some(t.me.id), 18);
        if self.corruption >= 100 {
            self.right_break(out, t);
        }
        true
    }
    fn right_break(&mut self, out: &mut Vec<Cmd>, t: &Tick) {
        let tick = t.tick;
        self.stats.right_breaks += 1;
        self.right_broken_until = tick + RIGHT_BREAK_TICKS as usize;
        self.completion = HEX_START;
        self.awaken_until = tick;
        self.corruption = 0;
        out.push(Cmd::LoseHp { target: t.me.id, amount: t.me.max_hp * RIGHT_BREAK_HP / 100 });
        let burst = HEXBURST_BASE + t.me.attack * HEXBURST_AD / 100;
        let near: Vec<Fighter> = t.enemies().filter(|e| dist(e.x, e.y, t.me.x, t.me.y) <= HEXBURST_RADIUS).cloned().collect();
        for e in near {
            self.emit(out, &e, burst, Dmg::Magic, tick);
        }
        out.push(Cmd::Log(format!("tick {tick} RIGHT GAUNTLET BROKE: Hexcore corrupted")));
        self.fx(out, "gbreak_r".into(), t.me.x, t.me.y, Some(t.me.id), 36);
    }
    /// The gauntlets each tick: overload heat, damage taken while overloaded or awakened, the Awakening ending,
    /// broken gauntlets coming back, takedown research.
    fn step_gauntlets(&mut self, out: &mut Vec<Cmd>, t: &Tick) {
        let tick = t.tick;
        let lost = if self.last_hp >= 0 { (self.last_hp - t.me.hp).max(0) } else { 0 };
        self.last_hp = t.me.hp;
        let lost_pct = lost * 100 / t.me.max_hp.max(1);
        if self.awaken_until > 0 && tick == self.awaken_until && !self.right_broken(tick) {
            self.completion = AWAKEN_AFTER;
            if self.corruption > 0 {
                self.habit("overclock stopped in time");
            }
            self.corruption = 0;
        }
        if self.awakened(tick) && lost_pct > 0 {
            self.corruption += lost_pct / HIT_CORRUPT_DIV;
            if self.corruption >= 100 {
                self.right_break(out, t);
            }
        }
        if self.left_broken_until > 0 && tick == self.left_broken_until {
            self.fx(out, "gfix_l".into(), t.me.x, t.me.y, Some(t.me.id), 18);
        }
        let held_over = self.pure.iter().filter(|s| s.open && s.over && tick >= s.ready_at).count() as i64;
        if self.over_sockets() > 0 && lost_pct > 0 {
            self.add_stress(out, t, lost_pct * HIT_STRESS_PCT);
        }
        if held_over > 0 && tick.is_multiple_of(OVER_HEAT_TICKS as usize) {
            self.add_stress(out, t, held_over);
        }
        if self.over_sockets() == 0 && self.open_count() <= 1 && tick.is_multiple_of(STRESS_DECAY_TICKS as usize) {
            self.stress = (self.stress - 1).max(0);
        }
        // takedowns: an enemy she hurt in the last 3 s who is gone or down
        let gone: Vec<usize> = self
            .hurt
            .iter()
            .filter(|(id, (at, hp, max))| tick <= at + TAKEDOWN_WINDOW as usize && *hp * 100 <= *max * 15 && t.get(**id).is_none())
            .map(|(id, _)| *id)
            .collect();
        for id in gone {
            self.hurt.remove(&id);
            self.stats.takedowns += 1;
            if !self.awakened(tick) {
                self.completion = (self.completion + RESEARCH_TAKEDOWN).min(100);
            }
        }
        self.hurt.retain(|_, (at, _, _)| tick <= *at + TAKEDOWN_WINDOW as usize);
        self.marks.retain(|_, (until, _)| *until > tick);
        if self.last_family.is_some() && tick > self.combo_at + COMBO_WINDOW as usize {
            self.combo = 0;
        }
    }

    // ---------------------------------------------------------------- rifts
    /// Opens a rift: its orbs are already spent; it opens after the travel delay.
    #[allow(clippy::too_many_arguments)]
    fn open(&mut self, out: &mut Vec<Cmd>, t: &Tick, spec: &RiftSpec, tech: usize, subject: Option<usize>, dim: usize, module: Option<usize>) -> u32 {
        let rt = RIFTS[spec.kind];
        let uid = self.next_uid;
        self.next_uid += 1;
        let d = dist(t.me.x, t.me.y, spec.x, spec.y);
        let delay = if (spec.kind == R_ANCHOR && spec.host.is_some()) || tech == M_SWIPE { open_delay(0, 100) } else { open_delay(d, rt.delay_pct) };
        let chain = if spec.chain {
            Some(spec.order.unwrap_or_else(|| {
                self.chain_seq += 1;
                self.chain_seq
            }))
        } else {
            None
        };
        let opens = t.tick + delay;
        self.rifts.push(Rift {
            uid,
            kind: spec.kind,
            tech,
            dim,
            dim2: None,
            module,
            infused: None,
            x: spec.x,
            y: spec.y,
            ex: spec.ex,
            ey: spec.ey,
            radius: rt.radius,
            opens,
            expires: opens + rt.life,
            host: spec.host,
            peer: None,
            subject,
            chain,
            entrance: spec.entrance,
            opened: false,
            fired: false,
            dwell: BTreeMap::new(),
            touched: BTreeSet::new(),
            sides: BTreeMap::new(),
            inside: BTreeSet::new(),
            done: false,
            charge: 0,
            drawn: [0; 4],
        });
        self.stats.rifts_opened += 1;
        let look = self.look();
        self.fx(out, format!("open_l{look}"), spec.x, spec.y, None, delay);
        uid
    }
    fn close(&mut self, out: &mut Vec<Cmd>, uid: u32, why: &str) {
        let look = self.look();
        let Some(r) = self.rifts.iter_mut().find(|r| r.uid == uid && !r.done) else { return };
        r.done = true;
        let (x, y) = (r.x, r.y);
        self.fx(out, format!("{why}_l{look}"), x, y, None, 24);
    }
    fn implode(&mut self, out: &mut Vec<Cmd>, t: &Tick, uid: u32) {
        let Some(r) = self.rift(uid).cloned() else { return };
        self.release_pressure(out, t, &r);
        let reach = r.radius * 3 / 2;
        let amount = IMPLODE_BASE + t.me.attack * IMPLODE_AD / 100;
        let mut hits = 0;
        let targets: Vec<Fighter> = t.enemies().filter(|e| r.reach(e.x, e.y) <= reach).cloned().collect();
        for e in &targets {
            hits += 1;
            self.displace(out, e, (e.x + r.x) / 2, (e.y + r.y) / 2, t.tick, t.wall, true);
            self.emit(out, e, amount, Dmg::Phys, t.tick);
            self.detonate(out, e, t.tick);
            for dim in r.dims() {
                self.apply_dim(out, t, dim, e, 200, (r.x, r.y, r.radius), Src::Imbue);
            }
        }
        self.stats.implodes += 1;
        self.stats.implode_hits += hits;
        // closing a rift on purpose releases part of the stress it cost
        self.stress = (self.stress - STRESS_TEAR / 2).max(0);
        let tier = self.tier4();
        self.fx(out, format!("tech_implode_t{tier}"), r.x, r.y, None, 30);
        self.close(out, uid, "close");
    }
    /// Art tier 0..3, or 4 for the #1 athlete.
    pub fn tier4(&self) -> usize {
        if self.look() == 8 {
            4
        } else {
            art_tier(self.rank)
        }
    }
}

impl Core {
    /// One simulation tick. Returns the commands to apply, in order.
    pub fn step(&mut self, input: &Input) -> Vec<Cmd> {
        let mut out = Vec::new();
        if self.last_tick == Some(input.tick) {
            return out;
        }
        self.last_tick = Some(input.tick);
        self.fx_left = FX_PER_UPDATE as usize;
        if input.tick.is_multiple_of(FX_REFILL as usize) && self.fx_tokens < FX_TOKENS as usize {
            self.fx_tokens += 1;
        }
        let Some(me) = input.fighters.iter().find(|f| f.id == input.me).cloned() else { return out };
        let t = Tick { tick: input.tick, me, fighters: input.fighters, wall: input.wall, attack_interval: input.attack_interval.max(1), stunned: input.stunned };
        let tick = t.tick;
        self.observe(&t);
        self.cooldowns.retain(|_, until| *until > tick);
        self.step_gauntlets(&mut out, &t);
        self.step_rifts(&mut out, &t);
        self.step_techniques(&mut out, &t);
        self.step_zones(&mut out, &t);
        self.step_breaking(&mut out, &t);
        self.step_later(&mut out, &t);
        self.step_dots(&mut out, &t);
        self.step_punch(&mut out, &t);
        self.step_packets(&mut out, &t);
        if !self.hand_placed && !self.phased && !t.stunned && tick >= self.last_think + self.table(&THINK_TICKS) as usize {
            self.last_think = tick;
            self.think(&mut out, &t);
        }
        self.render(&mut out, &t);
        self.rifts.retain(|r| !r.done);
        out
    }
    /// Positions and HP every 6 ticks: forecasting velocity, Rewind and HP recall history.
    fn observe(&mut self, t: &Tick) {
        if !t.tick.is_multiple_of(6) {
            return;
        }
        for f in t.fighters.iter().filter(|f| f.hp > 0) {
            let h = self.history.entry(f.id).or_default();
            if let Some(&(px, py, _)) = h.back() {
                if dist(f.x, f.y, px, py) > 30000 {
                    // a teleport, not a run: no velocity to lead rifts off the map with (the positions stay, so
                    // Rewind can still send them back to before the blink)
                    self.velocity.insert(f.id, (0, 0));
                } else {
                    self.velocity.insert(f.id, ((f.x - px) / 6, (f.y - py) / 6));
                }
            }
            h.push_back((f.x, f.y, f.hp));
            while h.len() > (HISTORY as usize / 6) + 1 {
                h.pop_front();
            }
        }
        let alive: BTreeSet<usize> = t.fighters.iter().filter(|f| f.hp > 0).map(|f| f.id).collect();
        self.history.retain(|id, _| alive.contains(id));
        self.velocity.retain(|id, _| alive.contains(id));
    }
    pub fn forecast(&self, f: &Fighter, ticks: i64) -> (i64, i64) {
        let (vx, vy) = self.velocity.get(&f.id).copied().unwrap_or((0, 0));
        clamp_map(f.x + (vx * ticks).clamp(-60000, 60000), f.y + (vy * ticks).clamp(-60000, 60000))
    }
    fn step_rifts(&mut self, out: &mut Vec<Cmd>, t: &Tick) {
        let tick = t.tick;
        for i in 0..self.rifts.len() {
            if self.rifts[i].done {
                continue;
            }
            // anchors ride their host; a dead or missing host ends them
            if let Some(h) = self.rifts[i].host {
                match t.get(h) {
                    Some(f) => {
                        self.rifts[i].x = f.x;
                        self.rifts[i].y = f.y;
                    }
                    None => {
                        let uid = self.rifts[i].uid;
                        self.close(out, uid, "close");
                        continue;
                    }
                }
            }
            if !self.rifts[i].opened && tick >= self.rifts[i].opens {
                self.rifts[i].opened = true;
                let r = self.rifts[i].clone();
                if t.enemies().any(|e| r.reach(e.x, e.y) <= r.radius * 2) {
                    self.stats.rifts_touching += 1;
                }
                if let Some(m) = r.module {
                    self.module(out, t, m, r.x, r.y);
                }
            }
            if self.rifts[i].opened && self.rifts[i].kind == R_WANDER {
                let r = &mut self.rifts[i];
                let (nx, ny) = if dist(r.x, r.y, r.ex, r.ey) <= WANDER_SPEED { (r.ex, r.ey) } else { along(r.x, r.y, r.ex - r.x, r.ey - r.y, WANDER_SPEED) };
                r.x = nx;
                r.y = ny;
            }
            if self.rifts[i].live() {
                if let Some(m) = self.rifts[i].infused {
                    let r = &self.rifts[i];
                    if (tick + r.uid as usize).is_multiple_of(60) {
                        let (x, y) = (r.x, r.y);
                        self.module(out, t, m, x, y);
                    }
                }
            }
            if tick >= self.rifts[i].expires {
                let r = self.rifts[i].clone();
                if r.kind != R_MAW {
                    self.release_pressure(out, t, &r);
                }
                if r.kind == R_MAW {
                    self.implode(out, t, r.uid);
                } else if r.tech == T_HOOK {
                    self.reel(out, t, &r);
                } else {
                    self.close(out, r.uid, "close");
                }
            }
        }
    }
    fn reel(&mut self, out: &mut Vec<Cmd>, t: &Tick, r: &Rift) {
        if let Some(target) = r.host.and_then(|h| t.get(h)).cloned() {
            let exit = r.peer.and_then(|p| self.rift(p)).map(|p| (p.x, p.y)).unwrap_or((t.me.x, t.me.y));
            if self.cooldown_ok(2000 + T_HOOK as u32, target.id, t.tick) && self.displace(out, &target, exit.0, exit.1, t.tick, t.wall, true) {
                self.control(out, &target, None, HOOK_BANISH, t.tick);
                self.set_cooldown(2000 + T_HOOK as u32, target.id, t.tick + TECHS[T_HOOK].cooldown);
                self.score_kidnap(t, exit);
                let tier = self.tier4();
                self.fx(out, format!("tech_hook_t{tier}"), exit.0, exit.1, None, 24);
            }
        }
        self.close(out, r.uid, "close");
    }
    /// Kidnap-style landings count as useful when they deliver an enemy into Vesper's team, not onto Vesper.
    fn score_kidnap(&mut self, t: &Tick, at: (i64, i64)) {
        self.stats.kidnaps += 1;
        let allies = t.allies().filter(|a| dist2(a.x, a.y, at.0, at.1) <= (45000i128).pow(2)).count();
        if allies >= 1 && dist(t.me.x, t.me.y, at.0, at.1) > 25000 {
            self.stats.useful_kidnaps += 1;
        }
    }
    /// Kidnap's field closes: every enemy still inside is stunned; if the target is one of them, the target drops
    /// onto the exit and Vesper lands beside it. A target who walked out leaves her where she blinked.
    #[allow(clippy::too_many_arguments)]
    fn resolve_kidnap(&mut self, out: &mut Vec<Cmd>, t: &Tick, target: usize, at: (i64, i64), exit: (i64, i64), dim: usize, radius: i64) {
        let tick = t.tick;
        let tier = self.tier4();
        let inside: Vec<Fighter> = t.enemies().filter(|e| dist(e.x, e.y, at.0, at.1) <= KIDNAP_RADIUS).cloned().collect();
        let mut caught = None;
        if !t.stunned {
            for e in &inside {
                if self.control(out, e, Some(Cc::Stun), KIDNAP_STUN, tick) && e.id == target {
                    caught = Some(e.clone());
                }
            }
        }
        let key = 2000 + T_KIDNAP as u32;
        let dropped = caught.filter(|v| self.cooldown_ok(key, v.id, tick)).is_some_and(|v| {
            if !self.displace(out, &v, exit.0, exit.1, tick, t.wall, true) {
                return false;
            }
            self.set_cooldown(key, v.id, tick + TECHS[T_KIDNAP].cooldown);
            let beside = along(exit.0, exit.1, exit.0 - at.0, exit.1 - at.1, 15000);   // behind the exit, the target between her and its team
            let (bx, by) = land(t.me.x, t.me.y, beside.0, beside.1, t.wall);
            out.push(Cmd::SetPos { target: t.me.id, x: bx, y: by });
            let landed = Fighter { x: exit.0, y: exit.1, ..v.clone() };
            self.apply_dim(out, t, dim, &landed, 100, (exit.0, exit.1, radius), Src::Imbue);
            true
        });
        self.stats.kidnaps += 1;
        if dropped {
            // useful when it drops the target among her team: her allies outnumber its own at the landing spot
            let allies = t.allies().filter(|a| a.id != t.me.id && dist2(a.x, a.y, exit.0, exit.1) <= (45000i128).pow(2)).count();
            let foes = t.enemies().filter(|e| e.id != target && dist2(e.x, e.y, exit.0, exit.1) <= (45000i128).pow(2)).count();
            if allies >= 1 && allies > foes {
                self.stats.useful_kidnaps += 1;
            }
            self.fx(out, format!("tech_kidnap_t{tier}"), at.0, at.1, None, 24);
            self.fx(out, format!("tech_kidnap_t{tier}"), exit.0, exit.1, None, 24);
        } else {
            self.stats.kidnap_dodges += 1;
            self.fx(out, format!("kidnapmiss_t{tier}"), at.0, at.1, None, 18);
        }
    }
    /// Kidnap fields armed right now, (x, y, radius, closes at): the game's AI is told to walk out of them.
    pub fn kidnap_zones(&self) -> Vec<(i64, i64, i64, usize)> {
        self.later.iter().filter_map(|l| if let Later::Kidnap { at, x, y, .. } = l { Some((*x, *y, KIDNAP_RADIUS, *at)) } else { None }).collect()
    }
    /// A one-shot technique has fired: its entrance seals at once and its exit lingers one more second.
    fn fire(&mut self, uid: u32) {
        let mut peer = None;
        if let Some(r) = self.rift_mut(uid) {
            r.fired = true;
            r.expires = r.expires.min(r.opens.max(1));
            peer = r.peer;
        }
        if let Some(p) = peer.and_then(|p| self.rift_mut(p)) {
            p.expires = p.expires.min(p.opens.max(p.expires.saturating_sub(RIFTS[p.kind].life)) + 60);
        }
    }
    fn exit_of(&self, r: &Rift) -> Option<(i64, i64)> {
        r.peer.and_then(|p| self.rift(p)).filter(|p| p.live()).map(|p| (p.x, p.y))
    }
    fn step_techniques(&mut self, out: &mut Vec<Cmd>, t: &Tick) {
        let tick = t.tick;
        let uids: Vec<u32> = self.rifts.iter().filter(|r| r.live()).map(|r| r.uid).collect();
        for uid in uids {
            let Some(r) = self.rift(uid).cloned() else { continue };
            let just_opened = tick == r.opens;
            let tier = self.tier4();
            match r.tech {
                T_GATE | T_SKYDROP | T_ROTATION if r.entrance => {
                    let Some(exit) = self.exit_of(&r) else { continue };
                    let movers: Vec<Fighter> = t.allies().filter(|a| r.covers(a.x, a.y)).cloned().collect();
                    for a in movers {
                        let key = 2000 + T_GATE as u32;
                        if !self.cooldown_ok(key, a.id, tick) {
                            continue;
                        }
                        self.set_cooldown(key, a.id, tick + TECHS[T_GATE].cooldown);
                        let (x, y) = land(a.x, a.y, exit.0, exit.1, t.wall);
                        out.push(Cmd::SetPos { target: a.id, x, y });
                        self.stats.gates += 1;
                        let idle = t.danger(a.x, a.y, 50000) == 0;
                        let foes = t.danger(x, y, 40000);
                        let friends = t.friends().filter(|f| f.id != a.id && dist2(f.x, f.y, x, y) <= (70000i128).pow(2)).count();
                        if idle && foes >= 1 && friends >= 1 {
                            self.stats.engaged_gates += 1;
                        } else if foes >= 3 {
                            self.stats.lonely_gates += 1;
                        }
                        self.fx(out, format!("tech_gate_t{tier}"), x, y, None, 24);
                        if r.tech == T_SKYDROP {
                            self.later.push(Later::Land { at: tick + 12, x, y, ally: a.id });
                        }
                    }
                }
                T_STEP if r.entrance && !r.fired => {
                    let Some(exit) = self.exit_of(&r) else { continue };
                    self.fire(uid);
                    if dist(t.me.x, t.me.y, r.x, r.y) <= STEP_REACH && tick >= self.self_move_until {
                        self.self_move_until = tick + SELF_MOVE_CD as usize;
                        let (x, y) = land(t.me.x, t.me.y, exit.0, exit.1, t.wall);
                        out.push(Cmd::SetPos { target: t.me.id, x, y });
                        self.fx(out, format!("tech_step_t{tier}"), x, y, None, 18);
                        if t.danger(x, y, 35000) >= 3 {
                            self.stats.risky_moves += 1;
                            self.risky_until = tick + 180;
                        }
                    }
                }
                T_PHASE if r.entrance && !r.fired => {
                    let Some(exit) = self.exit_of(&r) else { continue };
                    self.fire(uid);
                    if dist(t.me.x, t.me.y, r.x, r.y) <= STEP_REACH && tick >= self.self_move_until {
                        self.self_move_until = tick + SELF_MOVE_CD as usize;
                        out.push(Cmd::Banish { target: t.me.id, ticks: 45 });
                        self.phased = true;
                        self.aiming = false;
                        self.punching = false;
                        self.later.push(Later::Unphase { at: tick + 45, x: exit.0, y: exit.1 });
                        self.fx(out, format!("tech_phase_t{tier}"), r.x, r.y, None, 30);
                    }
                }
                T_KIDNAP if r.entrance && !r.fired => {
                    // both ends open: she steps through her tear and lands in front of the target, arming the field
                    let Some(exit) = self.exit_of(&r) else { continue };
                    self.fire(uid);
                    let Some(v) = r.subject.and_then(|s| t.get(s)).cloned() else { continue };
                    if t.stunned || self.phased || dist(t.me.x, t.me.y, v.x, v.y) > KIDNAP_REACH {
                        continue;
                    }
                    let front = along(v.x, v.y, t.me.x - v.x, t.me.y - v.y, KIDNAP_FRONT);
                    let (x, y) = land(t.me.x, t.me.y, front.0, front.1, t.wall);
                    out.push(Cmd::SetPos { target: t.me.id, x, y });
                    self.fx(out, format!("tech_kidnap_t{tier}"), t.me.x, t.me.y, None, 24);
                    self.fx(out, format!("kidnapwarn_t{tier}"), x, y, None, KIDNAP_ARM as usize);
                    self.later.push(Later::Kidnap { at: tick + KIDNAP_ARM as usize, target: v.id, x, y, exit, dim: r.dim, radius: r.radius });
                }
                T_SINKTRAP => {
                    let inside: Vec<Fighter> = t.enemies().filter(|e| r.covers(e.x, e.y)).cloned().collect();
                    let key = 2000 + T_SINKTRAP as u32;
                    for e in inside {
                        let n = {
                            let rr = self.rift_mut(uid).unwrap();
                            let n = rr.dwell.entry(e.id + 1_000_000).or_default();
                            *n += 1;
                            *n
                        };
                        if tick.is_multiple_of(PULSE as usize) {
                            self.buff(out, e.id, "vst_sink".into(), 40, Mods { move_speed: -30, ..Mods::default() });
                        }
                        if n >= 60 && self.cooldown_ok(key, e.id, tick) {
                            self.set_cooldown(key, e.id, tick + TECHS[T_SINKTRAP].cooldown);
                            self.stats.trap_catches += 1;
                            let exit = self.exit_of(&r);
                            if let Some(exit) = exit {
                                if self.displace(out, &e, exit.0, exit.1, tick, t.wall, true) {
                                    self.score_kidnap(t, exit);
                                }
                            }
                            self.control(out, &e, None, SINK_BANISH, tick);
                            self.fx(out, format!("tech_sinktrap_t{tier}"), e.x, e.y, None, 24);
                        }
                    }
                }
                T_SWAP if r.entrance && !r.fired => {
                    let Some(peer) = r.peer.and_then(|p| self.rift(p)).filter(|p| p.live()).cloned() else { continue };
                    self.fire(uid);
                    let near = |x: i64, y: i64, rad: i64| t.fighters.iter().filter(|f| f.hp > 0 && dist(f.x, f.y, x, y) <= rad).min_by_key(|f| (dist2(f.x, f.y, x, y), f.id)).cloned();
                    let (Some(a), Some(b)) = (near(r.x, r.y, r.radius * 2), near(peer.x, peer.y, peer.radius * 2)) else { continue };
                    if a.id == b.id || (a.team == t.me.team && b.team == t.me.team) {
                        continue;
                    }
                    let key = 2000 + T_SWAP as u32;
                    let enemies_ok = [&a, &b].iter().all(|f| f.team == t.me.team || (!f.cc_immune && self.cooldown_ok(key, f.id, tick)));
                    if !enemies_ok {
                        continue;
                    }
                    for f in [&a, &b] {
                        if f.team != t.me.team {
                            self.set_cooldown(key, f.id, tick + TECHS[T_SWAP].cooldown);
                        }
                    }
                    let (ax, ay) = land(a.x, a.y, b.x, b.y, t.wall);
                    let (bx, by) = land(b.x, b.y, a.x, a.y, t.wall);
                    out.push(Cmd::SetPos { target: a.id, x: ax, y: ay });
                    out.push(Cmd::SetPos { target: b.id, x: bx, y: by });
                    for (f, x, y) in [(&a, ax, ay), (&b, bx, by)] {
                        self.displaced.insert(f.id, tick);
                        if f.id == t.me.id && t.danger(x, y, 35000) >= 3 {
                            self.stats.risky_moves += 1;
                            self.risky_until = tick + 180;
                        }
                        if f.team != t.me.team {
                            self.score_kidnap(t, (x, y));
                        }
                    }
                    self.fx(out, format!("tech_swap_t{tier}"), r.x, r.y, None, 24);
                    self.fx(out, format!("tech_swap_t{tier}"), peer.x, peer.y, None, 24);
                }
                T_FLING => {
                    let key = 2000 + T_FLING as u32;
                    let caught: Vec<Fighter> = t.enemies().filter(|e| r.covers(e.x, e.y) && !r.touched.contains(&e.id)).cloned().collect();
                    for e in caught {
                        if let Some(rr) = self.rift_mut(uid) {
                            rr.touched.insert(e.id);
                        }
                        if e.cc_immune || !self.cooldown_ok(key, e.id, tick) {
                            continue;
                        }
                        let travel = (dist(r.x, r.y, r.ex, r.ey) / WANDER_SPEED).clamp(10, 60) as usize;
                        if self.control(out, &e, None, travel as i64, tick) {
                            self.set_cooldown(key, e.id, tick + TECHS[T_FLING].cooldown);
                            self.later.push(Later::Spit { at: tick + travel, target: e.id, x: r.ex, y: r.ey, ally: false });
                            self.fx(out, format!("tech_fling_t{tier}"), e.x, e.y, None, 24);
                        }
                    }
                }
                T_RESCUE if r.entrance && !r.fired => {
                    let Some(exit) = self.exit_of(&r) else { continue };
                    self.fire(uid);
                    if let Some(a) = r.subject.and_then(|s| t.get(s)).cloned() {
                        let key = 2000 + T_RESCUE as u32;
                        if r.covers(a.x, a.y) && self.cooldown_ok(key, a.id, tick) {
                            self.set_cooldown(key, a.id, tick + TECHS[T_RESCUE].cooldown);
                            let (x, y) = land(a.x, a.y, exit.0, exit.1, t.wall);
                            out.push(Cmd::SetPos { target: a.id, x, y });
                            self.fx(out, format!("tech_rescue_t{tier}"), x, y, None, 24);
                            self.habit("rescued ally");
                        }
                    }
                }
                T_POCKET if just_opened => {
                    if let Some(a) = r.subject.and_then(|s| t.get(s)).cloned() {
                        let key = 2000 + T_POCKET as u32;
                        if r.covers(a.x, a.y) && self.cooldown_ok(key, a.id, tick) {
                            self.set_cooldown(key, a.id, tick + TECHS[T_POCKET].cooldown);
                            out.push(Cmd::Banish { target: a.id, ticks: 60 });
                            self.fx(out, format!("tech_pocket_t{tier}"), a.x, a.y, None, 30);
                        }
                    }
                }
                T_LIFELINE => {
                    if t.me.hp * 4 < t.me.max_hp && tick >= self.self_move_until {
                        self.self_move_until = tick + SELF_MOVE_CD as usize;
                        let (x, y) = land(t.me.x, t.me.y, r.x, r.y, t.wall);
                        out.push(Cmd::SetPos { target: t.me.id, x, y });
                        self.fx(out, format!("tech_lifeline_t{tier}"), x, y, None, 24);
                        self.close(out, uid, "close");
                        self.aiming = false;
                    }
                }
                T_SEEDTRAP if !r.hidden(tick) => {
                    let victim = t.enemies().filter(|e| r.covers(e.x, e.y)).min_by_key(|e| (dist2(e.x, e.y, r.x, r.y), e.id)).cloned();
                    if let Some(v) = victim {
                        let key = 2000 + T_SEEDTRAP as u32;
                        if self.cooldown_ok(key, v.id, tick) {
                            self.set_cooldown(key, v.id, tick + TECHS[T_SEEDTRAP].cooldown);
                            self.stats.trap_catches += 1;
                            for dim in r.dims() {
                                self.apply_dim(out, t, dim, &v, 200, (r.x, r.y, r.radius), Src::Imbue);
                            }
                            match self.exit_of(&r) {
                                Some(exit) => {
                                    if self.displace(out, &v, exit.0, exit.1, tick, t.wall, true) {
                                        self.score_kidnap(t, exit);
                                    }
                                    self.control(out, &v, None, SEED_BANISH, tick);
                                }
                                None => {
                                    self.control(out, &v, None, SEED_BANISH_ALONE, tick);
                                }
                            }
                            self.fx(out, format!("tech_seedtrap_t{tier}"), r.x, r.y, None, 30);
                            self.close(out, uid, "close");
                        }
                    }
                }
                T_RAIN if just_opened => {
                    let amount = RAIN_BASE + t.me.attack * RAIN_AD / 100;
                    for k in 0..6usize {
                        let (ux, uy) = DIRS[self.random(16) as usize];
                        let reach = self.random(r.radius as u64) as i64;
                        self.later.push(Later::Drop { at: tick + 4 + k * 10, x: r.x + ux * reach / 10000, y: r.y + uy * reach / 10000, amount });
                    }
                }
                T_VOLLEY if just_opened => {
                    let amount = VOLLEY_BASE + t.me.attack * VOLLEY_AD / 100;
                    for k in 0..5i64 {
                        let (px, py) = (r.x + (r.ex - r.x) * k / 4, r.y + (r.ey - r.y) * k / 4);
                        if let Some(e) = t.enemies().filter(|e| dist(e.x, e.y, px, py) <= VOLLEY_RADIUS).min_by_key(|e| (dist2(e.x, e.y, px, py), e.id)).cloned() {
                            self.emit(out, &e, amount, Dmg::Phys, tick);
                            let dir = heading8(e.x - px, e.y - py);
                            self.fx(out, format!("packet_l{}_a{dir}", self.look()), px, py, None, 8);
                        }
                    }
                }
                _ => {}
            }
            // shape behaviours: Slash curtains and Mirror Panes throw crossers back
            if matches!(r.kind, R_SLASH | R_MIRROR) || (r.tech == T_CURTAIN && r.peer.is_some()) {
                self.crossing(out, t, uid);
            }
            // a Maw swallows everyone inside the moment it opens
            if r.kind == R_MAW && just_opened && r.tech != T_POCKET {
                let inside: Vec<Fighter> = t.enemies().filter(|e| r.covers(e.x, e.y)).cloned().collect();
                for e in inside {
                    self.control(out, &e, None, MAW_BANISH, tick);
                }
                self.fx(out, format!("tech_maw_t{tier}"), r.x, r.y, None, 30);
            }
        }
    }
    /// Enemies crossing a curtain line (a Slash, a Mirror Pane, or two linked curtain Tears) are thrown back.
    fn crossing(&mut self, out: &mut Vec<Cmd>, t: &Tick, uid: u32) {
        let Some(r) = self.rift(uid).cloned() else { return };
        let (ax, ay, bx, by) = if r.tech == T_CURTAIN && r.kind == R_TEAR {
            match r.peer.and_then(|p| self.rift(p)) {
                Some(p) => (r.x, r.y, p.x, p.y),
                None => return,
            }
        } else {
            (r.x, r.y, r.ex, r.ey)
        };
        let reach = r.radius.max(9000) * 3;
        let near: Vec<Fighter> = t.enemies().filter(|e| seg_dist(ax, ay, bx, by, e.x, e.y) <= reach).cloned().collect();
        for e in near {
            let s = side(ax, ay, bx, by, e.x, e.y);
            let prev = self.rift_mut(uid).and_then(|rr| rr.sides.insert(e.id, s));
            if let Some(p) = prev {
                if p != 0 && s != 0 && p != s {
                    // pushed back across, a little past the line
                    let (lx, ly) = (bx - ax, by - ay);
                    let (nx, ny) = (-ly, lx);
                    let push = if r.kind == R_MIRROR { 26000 } else { 14000 };
                    let dir = if p > 0 { 1 } else { -1 };
                    let to = along(e.x, e.y, nx * dir, ny * dir, push);
                    if self.displace(out, &e, to.0, to.1, t.tick, t.wall, true) {
                        if let Some(rr) = self.rift_mut(uid) {
                            rr.sides.insert(e.id, p);
                        }
                        let tier = self.tier4();
                        let tag = if r.kind == R_MIRROR { "reflect" } else { "curtain" };
                        self.fx(out, format!("tech_{tag}_t{tier}"), e.x, e.y, None, 18);
                    }
                }
            }
        }
    }
    fn step_zones(&mut self, out: &mut Vec<Cmd>, t: &Tick) {
        let tick = t.tick;
        let uids: Vec<u32> = self.rifts.iter().filter(|r| r.live() && !r.hidden(tick)).map(|r| r.uid).collect();
        for uid in uids {
            let Some(r) = self.rift(uid).cloned() else { continue };
            let dims = r.dims();
            let centre = (r.x, r.y, r.radius);
            let pulse = (tick + uid as usize).is_multiple_of(PULSE as usize);
            let inside: Vec<Fighter> = t.fighters.iter().filter(|f| f.hp > 0 && r.covers(f.x, f.y)).cloned().collect();
            let inside_ids: BTreeSet<usize> = inside.iter().map(|f| f.id).collect();
            for f in &inside {
                let entering = !r.inside.contains(&f.id);
                for &dim in &dims {
                    let p = DIMS[dim].prim;
                    if PRIM_ONCE[p as usize] {
                        if entering && !r.touched.contains(&(f.id ^ (dim << 24))) {
                            if let Some(rr) = self.rift_mut(uid) {
                                rr.touched.insert(f.id ^ (dim << 24));
                            }
                            if p == P_FOLD {
                                self.fold(out, t, &r, f);
                            } else if !matches!(p, P_SWAPSPACE | P_SCRAMBLE | P_WALL | P_TETHER | P_KALEIDO) && PRIM_SIDE[p as usize] != b'P' {
                                self.apply_dim(out, t, dim, f, 100, centre, Src::Zone);
                            }
                        }
                    } else if pulse && !matches!(p, P_VENT | P_UNSTABLE | P_SIPHON | P_PRESSURE) {
                        self.apply_dim(out, t, dim, f, 100, centre, Src::Zone);
                    }
                }
            }
            // tethered enemies who walk out are snapped back to the edge, once
            for &dim in &dims {
                if DIMS[dim].prim == P_TETHER {
                    let left: Vec<usize> = r.inside.iter().filter(|id| !inside_ids.contains(id)).copied().collect();
                    for id in left {
                        if let Some(e) = t.get(id).filter(|e| e.team != t.me.team).cloned() {
                            if !r.touched.contains(&(id ^ (dim << 24))) {
                                if let Some(rr) = self.rift_mut(uid) {
                                    rr.touched.insert(id ^ (dim << 24));
                                }
                                let edge = along(r.x, r.y, e.x - r.x, e.y - r.y, r.radius * 8 / 10);
                                self.displace(out, &e, edge.0, edge.1, tick, t.wall, true);
                            }
                        }
                    }
                }
            }
            if pulse {
                for &dim in &dims {
                    match DIMS[dim].prim {
                        P_VENT => self.stress = (self.stress - DIMS[dim].a).max(0),
                        P_PRESSURE if inside.iter().any(|f| f.team != t.me.team) => {
                            if let Some(rr) = self.rift_mut(uid) {
                                rr.charge = (rr.charge + 1).min(DIMS[dim].c.max(1));
                            }
                        }
                        P_SWAPSPACE | P_SCRAMBLE if !r.touched.contains(&(usize::MAX ^ dim)) => {
                            let enemies_only = DIMS[dim].prim == P_SCRAMBLE;
                            let mut pool: Vec<&Fighter> = inside.iter().filter(|f| !enemies_only || f.team != t.me.team).collect();
                            pool.sort_by_key(|f| (dist2(f.x, f.y, r.x, r.y), f.id));
                            if pool.len() >= 2 && pool.iter().take(2).any(|f| f.team != t.me.team) {
                                let (a, b) = (pool[0].clone(), pool[1].clone());
                                if let Some(rr) = self.rift_mut(uid) {
                                    rr.touched.insert(usize::MAX ^ dim);
                                }
                                let oka = self.displace(out, &a, b.x, b.y, tick, t.wall, a.team != t.me.team);
                                if oka {
                                    self.displace(out, &b, a.x, a.y, tick, t.wall, b.team != t.me.team);
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            // Glass Wall: a pane across the rift's width
            if dims.iter().any(|d| DIMS[*d].prim == P_WALL) {
                let (ax, ay, bx, by) = (r.x - r.radius, r.y, r.x + r.radius, r.y);
                let near: Vec<Fighter> = t.enemies().filter(|e| seg_dist(ax, ay, bx, by, e.x, e.y) <= r.radius).cloned().collect();
                for e in near {
                    let s = side(ax, ay, bx, by, e.x, e.y);
                    let prev = self.rift_mut(uid).and_then(|rr| rr.sides.insert(e.id + 2_000_000, s));
                    if let Some(p) = prev {
                        if p != 0 && s != 0 && p != s {
                            let to = (e.x, e.y - 12000 * s as i64);
                            if self.displace(out, &e, to.0, to.1, tick, t.wall, true) {
                                if let Some(rr) = self.rift_mut(uid) {
                                    rr.sides.insert(e.id + 2_000_000, p);
                                }
                            }
                        }
                    }
                }
            }
            if let Some(rr) = self.rift_mut(uid) {
                rr.inside = inside_ids;
            }
        }
    }
    /// Folded Map: an ally stepping in comes out of the nearest other live rift.
    fn fold(&mut self, out: &mut Vec<Cmd>, t: &Tick, r: &Rift, f: &Fighter) {
        if f.team != t.me.team {
            return;
        }
        let key = 2000 + T_GATE as u32;
        if !self.cooldown_ok(key, f.id, t.tick) {
            return;
        }
        let exit = self.rifts.iter().filter(|o| o.live() && o.uid != r.uid && !o.hidden(t.tick)).min_by_key(|o| (dist2(o.x, o.y, r.x, r.y), o.uid)).map(|o| (o.x, o.y));
        if let Some((x, y)) = exit {
            self.set_cooldown(key, f.id, t.tick + 180);
            let (x, y) = land(f.x, f.y, x, y, t.wall);
            out.push(Cmd::SetPos { target: f.id, x, y });
        }
    }
    /// Enemies standing on a rift's core for a second break it (never while the Hexcore is awakened).
    fn step_breaking(&mut self, out: &mut Vec<Cmd>, t: &Tick) {
        let tick = t.tick;
        if self.awakened(tick) {
            return;
        }
        let mut broken = vec![];
        for r in self.rifts.iter_mut().filter(|r| r.opened && !r.done) {
            if r.hidden(tick) || r.kind == R_MAW {
                continue;
            }
            let here: Vec<usize> = t
                .enemies()
                .filter(|e| Some(e.id) != r.host && r.reach(e.x, e.y) <= BREAK_CORE)
                .map(|e| e.id)
                .collect();
            r.dwell.retain(|id, _| *id >= 1_000_000 || here.contains(id));
            let mut snapped = false;
            for id in here {
                let n = r.dwell.entry(id).or_default();
                *n += 1;
                if *n >= BREAK_TICKS as usize {
                    snapped = true;
                }
            }
            if snapped {
                broken.push(r.uid);
            }
        }
        for uid in broken {
            if self.rift(uid).is_none() {
                continue;
            }
            self.stats.breaks += 1;
            out.push(Cmd::Log(format!("tick {tick} rift {uid} broken by an enemy (+{STRESS_BREAK} stress)")));
            // a broken Pressure rift still bursts: breaking it is closing it
            if let Some(r) = self.rift(uid).cloned() {
                self.release_pressure(out, t, &r);
            }
            self.close(out, uid, "break");
            self.add_stress(out, t, STRESS_BREAK);
        }
    }
    fn step_later(&mut self, out: &mut Vec<Cmd>, t: &Tick) {
        let tick = t.tick;
        let (due, rest): (Vec<Later>, Vec<Later>) = std::mem::take(&mut self.later).into_iter().partition(|l| l.at() <= tick);
        self.later = rest;
        let tier = self.tier4();
        for l in due {
            match l {
                Later::Repeat { target, amount, .. } => {
                    if let Some(f) = t.get(target).cloned() {
                        self.emit(out, &f, amount, Dmg::Phys, tick);
                        self.fx(out, format!("impact_l{}", self.look()), f.x, f.y, None, 12);
                    }
                }
                Later::Domino { uid, .. } => self.implode(out, t, uid),
                Later::Kidnap { target, x, y, exit, dim, radius, .. } => self.resolve_kidnap(out, t, target, (x, y), exit, dim, radius),
                Later::Drop { x, y, amount, .. } => {
                    let hit: Vec<Fighter> = t.enemies().filter(|e| dist(e.x, e.y, x, y) <= RAIN_RADIUS).cloned().collect();
                    for e in hit {
                        self.emit(out, &e, amount, Dmg::Phys, tick);
                    }
                    self.fx(out, format!("tech_rain_t{tier}"), x, y, None, 12);
                }
                Later::Spit { target, x, y, ally, .. } => {
                    if let Some(f) = t.get(target).cloned() {
                        if ally {
                            let (x, y) = land(f.x, f.y, x, y, t.wall);
                            out.push(Cmd::SetPos { target, x, y });
                        } else if self.displace(out, &f, x, y, tick, t.wall, true) {
                            self.score_kidnap(t, (x, y));
                        }
                    }
                }
                Later::Unphase { x, y, .. } => {
                    self.phased = false;
                    let (x, y) = land(t.me.x, t.me.y, x, y, t.wall);
                    out.push(Cmd::SetPos { target: t.me.id, x, y });
                    self.fx(out, format!("tech_phase_t{tier}"), x, y, None, 24);
                }
                Later::Burst { x, y, pm, .. } => {
                    let hit: Vec<Fighter> = t.enemies().filter(|e| dist(e.x, e.y, x, y) <= PRESSURE_RADIUS).cloned().collect();
                    for e in hit {
                        self.emit(out, &e, e.max_hp * pm / 1000, Dmg::Magic, tick);
                    }
                    self.fx(out, format!("tech_implode_t{tier}"), x, y, None, 18);
                }
                Later::Land { x, y, .. } => {
                    let hit: Vec<Fighter> = t.enemies().filter(|e| dist(e.x, e.y, x, y) <= SKYDROP_RADIUS).cloned().collect();
                    for e in hit {
                        self.control(out, &e, Some(Cc::Stun), SKYDROP_STUN, tick);
                    }
                    self.fx(out, format!("tech_skydrop_t{tier}"), x, y, None, 24);
                }
            }
        }
    }
    fn step_dots(&mut self, out: &mut Vec<Cmd>, t: &Tick) {
        let tick = t.tick;
        if !tick.is_multiple_of(PULSE as usize) {
            self.dots.retain(|d| d.until > tick);
            return;
        }
        let dots = std::mem::take(&mut self.dots);
        for d in &dots {
            if d.until <= tick {
                continue;
            }
            if let Some(f) = t.get(d.target).cloned() {
                let n = self.emit(out, &f, d.per_pulse, Dmg::Magic, tick);
                if d.drain && n > 0 {
                    self.heal(out, t, t.me.id, n, tick);
                }
            }
        }
        self.dots = dots.into_iter().filter(|d| d.until > tick && t.get(d.target).is_some()).collect();
    }
}

impl Core {
    /// Live chain nodes in chain order.
    pub fn chain_nodes(&self, tick: usize) -> Vec<Rift> {
        let mut v: Vec<Rift> = self.rifts.iter().filter(|r| r.live() && r.chain.is_some() && !r.hidden(tick)).cloned().collect();
        v.sort_by_key(|r| r.chain);
        v
    }
    /// Orbs the planner keeps out of rift punches for its next technique.
    fn punch_reserve(&self) -> usize {
        self.table(&RESERVE) as usize + usize::from(self.gate(GATE_GEOMETRY) && !self.plan.is_empty())
    }
    /// Rift punches: while an orb is ready and an enemy is in reach (through the chain's exit, or straight within
    /// punch reach), every basic attack is a punch that breaks a hole in the dimension.
    fn step_punch(&mut self, out: &mut Vec<Cmd>, t: &Tick) {
        let tick = t.tick;
        let nodes = self.chain_nodes(tick);
        let reserve = if self.hand_placed { 0 } else { self.punch_reserve() };
        // the left fist punches: while that gauntlet is broken she only has her normal attack
        let castable = match self.armed.filter(|_| self.hand_placed) {
            Some(d) => self.cast_ok(d, tick),
            None => (0..DIMS.len()).any(|d| self.cast_ok(d, tick)),
        };
        // every dimension resting: she falls back to her plain jab
        let ready = !self.left_broken(tick) && self.orbs_ready(tick) > reserve && castable;
        let mut through = !nodes.is_empty();
        let pick = |c: &Core, from: (i64, i64), reach: i64| -> Option<Fighter> {
            let ok = |e: &Fighter| dist(e.x, e.y, from.0, from.1) <= reach;
            let keep = c.chain_target.and_then(|id| t.get(id)).filter(|e| e.team != t.me.team && ok(e)).cloned();
            keep.or_else(|| t.enemies().filter(|e| ok(e)).min_by_key(|e| (dist2(e.x, e.y, from.0, from.1), e.id)).cloned())
        };
        let mut target = match nodes.last() {
            Some(exit) => pick(self, (exit.x, exit.y), ATTACK_RANGE),
            None => pick(self, (t.me.x, t.me.y), PUNCH_REACH),
        };
        if through && target.is_some() && self.gate(GATE_TEAM) && t.danger(t.me.x, t.me.y, DIVE_RADIUS) > 0 {
            if self.aiming {
                self.habit("dropped aim for a diver");
            }
            through = false;
            target = pick(self, (t.me.x, t.me.y), PUNCH_REACH);
        }
        let Some(target) = target.filter(|_| ready && !self.phased) else {
            self.punching = false;
            self.aiming = false;
            return;
        };
        self.chain_target = Some(target.id);
        self.punching = true;
        self.aiming = through;
        if tick.is_multiple_of(6) {
            out.push(Cmd::Cc { target: t.me.id, kind: Cc::Disarm, ticks: 8 });
            if through {
                out.push(Cmd::Buff { target: t.me.id, name: "vs_aim".into(), ticks: 8, mods: Mods { move_speed: -100, ..Mods::default() } });
            }
        }
        if tick < self.next_attack || self.packets.len() >= PACKET_LIMIT as usize {
            return;
        }
        self.next_attack = tick + t.attack_interval;
        // the orb: a pure orb and a hexorb fuse while the Hexcore is awakened
        let fuse = self.awakened(tick) && self.pure_ready(tick) > 0 && self.hex_ready(tick) > 0;
        let hex_pref = if fuse { false } else { self.prefers_hex(tick) };
        let Some(orb) = self.take(tick, hex_pref) else { return };
        let dim = self.armed.filter(|_| self.hand_placed).unwrap_or_else(|| self.pick_dim(t, Purpose::Punch, Some(&target), (target.x, target.y)));
        let (combo, mut twist) = self.activate(orb, dim, tick);
        let mut power = if orb.hex.is_some() { self.hex_power(tick) } else { 100 };
        let mut module = orb.hex.map(|i| self.modules[i]);
        let mut fused = None;
        if fuse {
            if let Some(h) = self.take(tick, true).filter(|o| o.hex.is_some()) {
                let d2 = self.pick_dim(t, Purpose::Punch, Some(&target), (target.x, target.y));
                let (_, tw) = self.activate(h, d2, tick);
                twist = twist.or(tw).or(Some(twist_index(DIMS[dim].family, DIMS[d2].family)));
                fused = Some(d2);
                power = FUSE_PCT;
                module = h.hex.map(|i| self.modules[i]);
                self.stats.fused += 1;
            }
        }
        let mut path = VecDeque::new();
        if through {
            for n in &nodes {
                path.push_back((n.uid, n.x, n.y));
            }
        }
        path.push_back((0, target.x, target.y));
        let cast: Vec<usize> = std::iter::once(dim).chain(fused).collect();
        self.commit_cast(&cast, tick);
        let bonus = std::mem::take(&mut self.opunch) + combo - 100;
        self.stats.punches += 1;
        let tier = self.tier4();
        self.fx(out, format!("punch_t{tier}"), t.me.x, t.me.y, Some(t.me.id), 12);
        self.packets.push(Packet {
            x: t.me.x,
            y: t.me.y,
            hop: (t.me.x, t.me.y),
            path,
            target: target.id,
            eff: 10000,
            bonus,
            dim,
            fused,
            power,
            module,
            picked: vec![],
            true_pct: 0,
            ignore_wall: false,
            kaleido: false,
            twist,
            source: if through { "chain" } else { "punch" },
            base: t.me.attack,
            drawn: 0,
        });
        let exit = nodes.last().cloned();
        if let (true, Some(exit)) = (through && tick < self.crossfire_until, exit) {
            let others: Vec<Rift> = self.rifts.iter().filter(|r| r.live() && r.uid != exit.uid && !r.hidden(tick)).cloned().collect();
            let mut used = vec![target.id];
            for r in others.iter().take(4) {
                if self.packets.len() >= PACKET_LIMIT as usize || used.len() >= 3 {
                    break;
                }
                let foe = t.enemies().filter(|e| !used.contains(&e.id) && dist(e.x, e.y, r.x, r.y) <= ATTACK_RANGE).min_by_key(|e| (dist2(e.x, e.y, r.x, r.y), e.id)).cloned();
                if let Some(e) = foe {
                    used.push(e.id);
                    self.packets.push(Packet {
                        x: r.x,
                        y: r.y,
                        hop: (r.x, r.y),
                        path: VecDeque::from([(0, e.x, e.y)]),
                        target: e.id,
                        eff: CROSSFIRE_PCT * 100,
                        bonus: 0,
                        dim,
                        fused: None,
                        power,
                        module: None,
                        picked: vec![],
                        true_pct: 0,
                        ignore_wall: true,
                        kaleido: false,
                        twist: None,
                        source: "crossfire",
                        base: t.me.attack,
                        drawn: 0,
                    });
                }
            }
        }
    }
    /// Whether the next punch should take a hexorb (weaker, faster to recover) before a pure orb.
    fn prefers_hex(&mut self, tick: usize) -> bool {
        if self.hand_placed {
            return false;
        }
        if self.rank < GATE_SAFETY {
            return self.random(2) == 0;
        }
        // keep pure orbs (full power, Shatter fuel) for rifts; hexorbs carry the punches once they recover fast
        self.hex_ready(tick) > 0 && (self.pure_ready(tick) <= 2 || self.completion >= HEX_FAST_AT)
    }
    fn step_packets(&mut self, out: &mut Vec<Cmd>, t: &Tick) {
        let tick = t.tick;
        let packets = std::mem::take(&mut self.packets);
        let mut keep = vec![];
        for mut p in packets {
            let mut budget = PACKET_SPEED;
            let mut alive = true;
            let mut hit = false;
            let mut bank = false;
            while budget > 0 && alive && !hit {
                let Some(&(uid, wx, wy)) = p.path.front() else {
                    alive = false;
                    break;
                };
                let (wx, wy) = if uid == 0 { t.get(p.target).map(|f| (f.x, f.y)).unwrap_or((wx, wy)) } else { (wx, wy) };
                let d = dist(p.x, p.y, wx, wy);
                if d > budget {
                    let (nx, ny) = along(p.x, p.y, wx - p.x, wy - p.y, budget);
                    p.x = nx;
                    p.y = ny;
                    break;
                }
                budget -= d;
                let len = dist(p.hop.0, p.hop.1, wx, wy);
                p.hop = (wx, wy);
                p.x = wx;
                p.y = wy;
                p.path.pop_front();
                if uid == 0 {
                    // the punch's own link (Vesper or the exit rift to the target) costs like any other
                    if p.source == "punch" {
                        p.eff = p.eff * keep_bp(len) / 10000;
                    }
                    hit = true;
                    break;
                }
                let Some(r) = self.rift(uid).filter(|r| r.live()).cloned() else {
                    alive = false;
                    self.fx(out, format!("close_l{}", self.look()), wx, wy, None, 12);
                    break;
                };
                let mut keep_bp_v = keep_bp(len);
                if bank {
                    keep_bp_v = (keep_bp_v * 12 / 10).min(9500);
                }
                p.eff = p.eff * keep_bp_v / 10000;
                bank = r.kind == R_MIRROR;
                if p.path.front().is_some_and(|w| w.0 == 0) {
                    // the fist bursts out of the last rift toward its target
                    self.fx(out, format!("exit_l{}", self.look()), wx, wy, None, 12);
                }
                // a rift's packet-only dimensions ride along with the punch
                for dim in r.dims() {
                    let dd = DIMS[dim];
                    match dd.prim {
                        P_TRUE => p.true_pct += dd.a,
                        P_LENS => p.bonus += dd.a,
                        P_MOMENTUM => p.bonus += self.momentum(t, dd.a, dd.b),
                        P_ECHO | P_WILDECHO | P_PRISM => p.picked.push(dim),
                        P_KALEIDO => p.kaleido = true,
                        P_UNSTABLE => {
                            p.bonus += dd.a - 100;
                            self.add_stress(out, t, dd.b);
                        }
                        _ => {}
                    }
                }
            }
            if hit {
                self.hit(out, t, &p);
            } else if alive {
                if tick >= p.drawn {
                    let dir = p.path.front().map_or(0, |w| heading8(w.1 - p.x, w.2 - p.y));
                    let look = self.look();
                    if self.fx_low(out, format!("packet_l{look}_a{dir}"), p.x, p.y, None, 4) {
                        p.drawn = tick + 4;
                    }
                }
                keep.push(p);
            }
        }
        self.packets = keep;
    }
    /// A rift punch lands: the hole it breaks carries the orb's dimension.
    fn hit(&mut self, out: &mut Vec<Cmd>, t: &Tick, p: &Packet) {
        let tick = t.tick;
        let Some(f) = t.get(p.target).cloned() else { return };
        let mut bonus = p.bonus;
        let mut true_pct = p.true_pct;
        let mut power = p.power;
        let dims: Vec<usize> = std::iter::once(p.dim).chain(p.fused).collect();
        for &dim in &dims {
            for e in [PUNCH[dim].main, PUNCH[dim].rider] {
                match e.prim {
                    P_LENS => bonus += e.a,
                    P_MOMENTUM => bonus += self.momentum(t, e.a, e.b),
                    P_TRUE => true_pct += e.a,
                    P_EXECUTE if f.hp * 100 < f.max_hp * e.b => bonus += e.a,
                    P_UNSTABLE => {
                        power = power * e.a / 100;
                        self.add_stress(out, t, e.b);
                    }
                    _ => {}
                }
            }
        }
        let amount = p.base * p.eff / 10000 * (100 + bonus).max(10) / 100 * power / 100;
        let dealt = self.emit(out, &f, amount, Dmg::Phys, tick);
        if true_pct > 0 {
            self.emit(out, &f, amount * true_pct / 100, Dmg::True, tick);
        }
        self.stats.eff_sum += p.eff;
        self.stats.landed += 1;
        let s = (p.eff / 100).clamp(30, 100) * power.min(FUSE_PCT) / 100;
        self.detonate(out, &f, tick);
        for &dim in &dims {
            self.deliver_punch(out, t, dim, &f, s, amount, dealt, 0);
        }
        // Static marks: the punch arcs to every marked enemy, then the marks are spent
        let marks: Vec<(usize, i64)> = self.marks.iter().filter(|(id, _)| **id != f.id).map(|(id, (_, pct))| (*id, *pct)).collect();
        if !marks.is_empty() {
            self.marks.clear();
            for (id, pct) in marks {
                if let Some(e) = t.get(id).cloned() {
                    self.emit(out, &e, amount * pct / 100, Dmg::Phys, tick);
                    self.fx(out, format!("hole_l{}", self.look()), e.x, e.y, None, 12);
                }
            }
        }
        for &dim in &p.picked {
            match DIMS[dim].prim {
                P_ECHO => self.later.push(Later::Repeat { at: tick + DIMS[dim].b as usize, target: f.id, amount: amount * DIMS[dim].a / 100 }),
                P_WILDECHO => {
                    let pct = WILDECHO_MIN + self.random(WILDECHO_SPAN as u64 + 1) as i64;
                    self.later.push(Later::Repeat { at: tick + 30, target: f.id, amount: amount * pct / 100 });
                }
                _ => {
                    let others: Vec<Fighter> = t.enemies().filter(|e| e.id != f.id && dist(e.x, e.y, f.x, f.y) <= PRISM_RADIUS).take(DIMS[dim].a as usize - 1).cloned().collect();
                    for e in others {
                        self.emit(out, &e, amount * DIMS[dim].b / 100, Dmg::Phys, tick);
                    }
                }
            }
        }
        if let Some(m) = p.module {
            self.module(out, t, m, f.x, f.y);
        }
        let twist = if p.kaleido { Some(self.random(N_TWISTS as u64) as usize) } else { p.twist };
        if let Some(idx) = twist {
            self.twist(out, t, idx, &f, amount);
        }
        let look = self.look();
        self.fx(out, format!("hole_l{look}"), f.x, f.y, None, 18);
    }
    /// A dimension's punch: its main effect and its rider, at the hole on `f`.
    #[allow(clippy::too_many_arguments)]
    fn deliver_punch(&mut self, out: &mut Vec<Cmd>, t: &Tick, dim: usize, f: &Fighter, s: i64, amount: i64, dealt: i64, depth: usize) {
        let tick = t.tick;
        let me = (t.me.x, t.me.y, 0);
        self.slammed = false;
        let d = PUNCH[dim];
        for e in [d.main, d.rider] {
            match e.prim {
                255 | P_LENS | P_TRUE | P_EXECUTE | P_UNSTABLE | P_MOMENTUM => {}
                P_ECHO => self.later.push(Later::Repeat { at: tick + e.b as usize, target: f.id, amount: amount * e.a / 100 }),
                P_WILDECHO => {
                    let pct = WILDECHO_MIN + self.random(WILDECHO_SPAN as u64 + 1) as i64;
                    self.later.push(Later::Repeat { at: tick + 30, target: f.id, amount: amount * pct / 100 });
                }
                P_PRISM => {
                    let others: Vec<Fighter> = t.enemies().filter(|o| o.id != f.id && dist(o.x, o.y, f.x, f.y) <= PRISM_RADIUS).take(e.a.max(1) as usize - 1).cloned().collect();
                    for o in others {
                        self.emit(out, &o, amount * e.b / 100, Dmg::Phys, tick);
                    }
                }
                P_SPLASH => {
                    let others: Vec<Fighter> = t.enemies().filter(|o| o.id != f.id && dist(o.x, o.y, f.x, f.y) <= e.b).cloned().collect();
                    for o in others {
                        self.emit(out, &o, amount * e.a / 100, Dmg::Phys, tick);
                    }
                }
                P_RICOCHET => {
                    if let Some(o) = t.enemies().filter(|o| o.id != f.id && dist(o.x, o.y, f.x, f.y) <= RICOCHET_RADIUS).min_by_key(|o| (dist2(o.x, o.y, f.x, f.y), o.id)).cloned() {
                        self.emit(out, &o, amount * e.a / 100, Dmg::Phys, tick);
                        self.fx(out, format!("hole_l{}", self.look()), o.x, o.y, None, 12);
                    }
                }
                P_CHAIN => {
                    if let Some(o) = t.enemies().filter(|o| o.id != f.id && dist(o.x, o.y, f.x, f.y) <= e.a).min_by_key(|o| (dist2(o.x, o.y, f.x, f.y), o.id)).cloned() {
                        self.apply_eff(out, t, dim, d.main, &o, s * e.b / 100, me, Src::Punch, None);
                    }
                }
                P_SIPHON => {
                    let allies: Vec<usize> = t.friends().filter(|a| dist(a.x, a.y, f.x, f.y) <= MODULE_RADIUS).map(|a| a.id).collect();
                    for a in allies {
                        self.heal(out, t, a, dealt * e.a / 100, tick);
                    }
                }
                P_KALEIDO => {
                    let idx = self.random(N_TWISTS as u64) as usize;
                    self.twist(out, t, idx, f, amount);
                }
                P_VENT => self.stress = (self.stress - e.a).max(0),
                P_DICE => {
                    let other = self.other_family_dim(DIMS[dim].family);
                    if depth == 0 && PUNCH[other].main.prim != P_DICE {
                        self.deliver_punch(out, t, other, f, s, amount, dealt, 1);
                    }
                }
                P_SCRAMBLE => {
                    if let Some(o) = t.enemies().filter(|o| o.id != f.id && dist(o.x, o.y, f.x, f.y) <= NEAR_RADIUS).min_by_key(|o| (dist2(o.x, o.y, f.x, f.y), o.id)).cloned() {
                        if self.displace(out, f, o.x, o.y, tick, t.wall, true) {
                            self.displace(out, &o, f.x, f.y, tick, t.wall, true);
                        }
                    }
                }
                _ => match PRIM_SIDE[e.prim as usize] {
                    b'E' => self.apply_eff(out, t, dim, e, f, s, me, Src::Punch, None),
                    b'S' => {
                        let mine = t.me.clone();
                        self.apply_eff(out, t, dim, e, &mine, s, (f.x, f.y, 0), Src::Punch, None);
                    }
                    side => {
                        // support and two-sided effects burst around the hole: allies near it (and the target)
                        let near: Vec<Fighter> = t.friends().filter(|a| dist(a.x, a.y, f.x, f.y) <= MODULE_RADIUS).cloned().collect();
                        for a in near {
                            self.apply_eff(out, t, dim, e, &a, s, me, Src::Punch, None);
                        }
                        if side == b'X' {
                            self.apply_eff(out, t, dim, e, f, s, me, Src::Punch, None);
                        }
                    }
                },
            }
        }
    }
    /// Rift Swipe: she tears the dimension along a line toward `point`. Two orbs; the tear stays as a Slash rift.
    fn swipe(&mut self, out: &mut Vec<Cmd>, t: &Tick, point: (i64, i64), dim: Option<usize>, subject: Option<usize>) -> bool {
        let tick = t.tick;
        if self.orbs_ready(tick) < MOVE_ORBS[M_SWIPE - 28] || self.open_count() >= MAX_RIFTS as usize {
            return false;
        }
        if !dim.map_or_else(|| (0..DIMS.len()).any(|d| self.cast_ok(d, tick)), |d| self.cast_ok(d, tick)) {
            return false;
        }
        let hex_pref = self.prefers_hex(tick);
        let Some(orb) = self.take(tick, hex_pref) else { return false };
        let _ = self.take(tick, orb.hex.is_some());
        let target = subject.and_then(|s| t.get(s)).cloned();
        let dim = dim.unwrap_or_else(|| self.pick_dim(t, Purpose::Swipe, target.as_ref(), point));
        let (combo, twist) = self.activate(orb, dim, tick);
        self.commit_cast(&[dim], tick);
        let power = if orb.hex.is_some() { self.hex_power(tick) } else { 100 };
        let module = orb.hex.map(|i| self.modules[i]);
        let (dx, dy) = if point == (t.me.x, t.me.y) { (1, 0) } else { (point.0 - t.me.x, point.1 - t.me.y) };
        let b = clamp_map(along(t.me.x, t.me.y, dx, dy, SWIPE_LEN).0, along(t.me.x, t.me.y, dx, dy, SWIPE_LEN).1);
        let l = Line { ax: t.me.x, ay: t.me.y, bx: b.0, by: b.1 };
        let spec = RiftSpec::line(R_SLASH, l.ax, l.ay, l.bx, l.by);
        let uid = self.open(out, t, &spec, M_SWIPE, subject, dim, module);
        let d = SWIPE[dim];
        if d.main.prim == P_WALL {
            if let Some(r) = self.rift_mut(uid) {
                r.expires += RIFTS[R_SLASH].life;
            }
        }
        self.add_stress(out, t, STRESS_TEAR);
        let mut bonus = combo - 100;
        let mut true_pct = 0;
        let mut power = power;
        for e in [d.main, d.rider] {
            match e.prim {
                P_LENS => bonus += e.a,
                P_MOMENTUM => bonus += self.momentum(t, e.a, e.b),
                P_TRUE => true_pct += e.a,
                P_UNSTABLE => {
                    power = power * e.a / 100;
                    self.add_stress(out, t, e.b);
                }
                _ => {}
            }
        }
        let amount = (SWIPE_BASE + t.me.attack * SWIPE_AD / 100) * (100 + bonus) / 100 * power / 100;
        let s = power.min(FUSE_PCT);
        let on_line: Vec<Fighter> = t.fighters.iter().filter(|f| f.hp > 0 && f.id != t.me.id && seg_dist(l.ax, l.ay, l.bx, l.by, f.x, f.y) <= SWIPE_WIDTH).cloned().collect();
        let enemies: Vec<Fighter> = on_line.iter().filter(|f| f.team != t.me.team).cloned().collect();
        let mut total = 0;
        for f in &on_line {
            let enemy = f.team != t.me.team;
            if enemy {
                total += self.emit(out, f, amount, Dmg::Phys, tick);
                if true_pct > 0 {
                    self.emit(out, f, amount * true_pct / 100, Dmg::True, tick);
                }
                self.detonate(out, f, tick);
            }
            let c = nearest_on(l.ax, l.ay, l.bx, l.by, f.x, f.y);
            self.slammed = false;
            for e in [d.main, d.rider] {
                match e.prim {
                    255 | P_LENS | P_TRUE | P_UNSTABLE | P_WALL | P_SWAPSPACE | P_SCRAMBLE | P_VENT | P_KALEIDO | P_DICE | P_SIPHON | P_MOMENTUM => {}
                    P_SPLASH if enemy => {
                        let near: Vec<Fighter> = t.enemies().filter(|o| o.id != f.id && dist(o.x, o.y, f.x, f.y) <= e.b && seg_dist(l.ax, l.ay, l.bx, l.by, o.x, o.y) > SWIPE_WIDTH).cloned().collect();
                        for o in near {
                            self.emit(out, &o, amount * e.a / 100, Dmg::Phys, tick);
                        }
                    }
                    P_ECHO if enemy => self.later.push(Later::Repeat { at: tick + e.b as usize, target: f.id, amount: amount * e.a / 100 }),
                    _ => self.apply_eff(out, t, dim, e, f, s, (c.0, c.1, SWIPE_WIDTH), Src::Swipe, Some(l)),
                }
            }
        }
        // once-per-swipe effects
        for e in [d.main, d.rider] {
            match e.prim {
                P_SWAPSPACE => {
                    let near_a = enemies.iter().min_by_key(|f| (dist2(f.x, f.y, l.ax, l.ay), f.id)).cloned();
                    let near_b = enemies.iter().min_by_key(|f| (dist2(f.x, f.y, l.bx, l.by), f.id)).cloned();
                    if let (Some(a), Some(b)) = (near_a, near_b) {
                        if a.id != b.id && self.displace(out, &a, b.x, b.y, tick, t.wall, true) {
                            self.displace(out, &b, a.x, a.y, tick, t.wall, true);
                        }
                    }
                }
                P_SCRAMBLE => {
                    // everyone on the line is flipped end for end along it
                    for f in &enemies {
                        let c = nearest_on(l.ax, l.ay, l.bx, l.by, f.x, f.y);
                        let to = (l.ax + l.bx - c.0 + (f.x - c.0), l.ay + l.by - c.1 + (f.y - c.1));
                        self.displace(out, f, to.0, to.1, tick, t.wall, true);
                    }
                }
                P_VENT => self.stress = (self.stress - e.a).max(0),
                P_SIPHON => {
                    let allies: Vec<usize> = on_line.iter().filter(|f| f.team == t.me.team).map(|f| f.id).chain(std::iter::once(t.me.id)).collect();
                    for a in allies {
                        self.heal(out, t, a, total * e.a / 100, tick);
                    }
                }
                P_KALEIDO => {
                    if let Some(f) = enemies.first() {
                        let idx = self.random(N_TWISTS as u64) as usize;
                        self.twist(out, t, idx, f, amount);
                    }
                }
                P_DICE => {
                    let other = self.other_family_dim(DIMS[dim].family);
                    if SWIPE[other].main.prim != P_DICE {
                        for f in &on_line {
                            let c = nearest_on(l.ax, l.ay, l.bx, l.by, f.x, f.y);
                            self.apply_eff(out, t, other, SWIPE[other].main, f, s, (c.0, c.1, SWIPE_WIDTH), Src::Swipe, Some(l));
                        }
                    }
                }
                _ => {}
            }
        }
        if let (Some(idx), Some(f)) = (twist, enemies.first()) {
            self.twist(out, t, idx, f, amount);
        }
        if let Some(m) = module {
            self.module(out, t, m, (l.ax + l.bx) / 2, (l.ay + l.by) / 2);
        }
        self.stats.swipes += 1;
        self.stats.swipe_hits += enemies.len() as u32;
        let tier = self.tier4();
        self.fx(out, format!("swipe_t{tier}_a{}", heading8(dx, dy) % 4), (l.ax + l.bx) / 2, (l.ay + l.by) / 2, None, 24);
        self.swipe_until = tick + 24;
        true
    }
    fn twist(&mut self, out: &mut Vec<Cmd>, t: &Tick, idx: usize, f: &Fighter, amount: i64) {
        let tick = t.tick;
        let w = TWISTS[idx];
        self.stats.twists += 1;
        let nodes = self.chain_nodes(tick);
        let exit = nodes.last().map(|n| (n.x, n.y)).unwrap_or((f.x, f.y));
        let first = nodes.first().map(|n| (n.x, n.y));
        let mag = w.mag;
        let within = |x: i64, y: i64, cx: i64, cy: i64, r: i64| dist2(x, y, cx, cy) <= (r as i128) * (r as i128);
        match w.kind {
            0 => {
                self.control(out, f, Some(Cc::Root), mag, tick);
            }
            1 => {
                if f.controlled {
                    self.emit(out, f, amount * mag / 100, Dmg::Phys, tick);
                }
            }
            2 => {
                if self.displaced.get(&f.id).is_some_and(|d| tick <= d + 60) {
                    self.control(out, f, Some(Cc::Stun), mag, tick);
                }
            }
            3 => {
                let allies: Vec<usize> = t.friends().filter(|a| within(a.x, a.y, exit.0, exit.1, TWIST_RADIUS)).map(|a| a.id).collect();
                for a in allies {
                    if self.cooldown_ok(5003, a, tick) {
                        self.set_cooldown(5003, a, tick + 600);
                        out.push(Cmd::Buff { target: a, name: "vst_sanctuary".into(), ticks: mag as usize, mods: Mods { cc_immune: true, ..Mods::default() } });
                    }
                }
            }
            4 => {
                if self.control(out, f, Some(Cc::Root), mag, tick) {
                    let to = lerp(f.x, f.y, exit.0, exit.1, 50);
                    self.displace(out, f, to.0, to.1, tick, t.wall, true);
                }
            }
            5 => {
                let second = t.enemies().filter(|e| e.id != f.id && within(e.x, e.y, f.x, f.y, OVERCHARGE_RADIUS)).min_by_key(|e| (dist2(e.x, e.y, f.x, f.y), e.id)).cloned();
                if let Some(e) = second {
                    self.emit(out, &e, amount * mag / 100, Dmg::Phys, tick);
                }
            }
            6 => {
                if self.displaced.get(&f.id).is_some_and(|d| tick <= d + 60) {
                    self.emit(out, f, f.max_hp * mag / 1000, Dmg::Phys, tick);
                }
            }
            7 | 12 => {
                let allies: Vec<Fighter> = t.friends().filter(|a| within(a.x, a.y, exit.0, exit.1, TWIST_RADIUS)).cloned().collect();
                for a in allies {
                    let n = if w.kind == 7 { amount * mag / 100 } else { a.max_hp * mag / 1000 };
                    self.heal(out, t, a.id, n, tick);
                }
            }
            8 => {
                let far = self.rifts.iter().filter(|r| r.live()).max_by_key(|r| (dist2(r.x, r.y, exit.0, exit.1), r.uid)).cloned();
                if let Some(r) = far {
                    let dim = r.dim;
                    let near: Vec<Fighter> = t.enemies().filter(|e| r.covers(e.x, e.y)).cloned().collect();
                    for e in near {
                        self.apply_dim(out, t, dim, &e, 100, (r.x, r.y, r.radius), Src::Zone);
                    }
                }
            }
            9 => {
                let near: Vec<Fighter> = t.enemies().filter(|e| e.id != f.id && within(e.x, e.y, f.x, f.y, TWIST_RADIUS)).cloned().collect();
                for e in near {
                    let to = lerp(e.x, e.y, f.x, f.y, mag.min(100));
                    self.displace(out, &e, to.0, to.1, tick, t.wall, true);
                }
            }
            10 => {
                let allies: Vec<usize> = t.friends().filter(|a| within(a.x, a.y, exit.0, exit.1, TWIST_RADIUS)).map(|a| a.id).collect();
                for a in allies {
                    out.push(Cmd::Buff { target: a, name: "vst_escort".into(), ticks: 60, mods: Mods { move_speed: mag as i32, ..Mods::default() } });
                }
            }
            11 => {
                if let Some((x, y)) = first {
                    self.displace(out, f, x, y, tick, t.wall, true);
                }
            }
            13 => {
                let allies: Vec<Fighter> = t.friends().filter(|a| within(a.x, a.y, exit.0, exit.1, TWIST_RADIUS)).cloned().collect();
                for a in allies {
                    self.shield(out, &a, a.max_hp * mag / 1000, 180, tick);
                }
            }
            _ => self.later.push(Later::Repeat { at: tick + 12, target: f.id, amount: amount * mag / 100 }),
        }
        self.fx(out, format!("twist{}", w.kind), f.x, f.y, None, 18);
    }
    /// Rift layers are redrawn when their 48-tick loop runs out, from the ambient budget: when it is short a
    /// layer waits a tick instead of skipping a whole loop.
    fn render(&mut self, out: &mut Vec<Cmd>, t: &Tick) {
        let tick = t.tick;
        let look = self.look();
        let tier = self.tier4().min(3);
        for i in 0..self.rifts.len() {
            if !self.rifts[i].live() {
                continue;
            }
            for layer in 0..3usize {
                let r = &self.rifts[i];
                if tick < r.drawn[layer] || (r.hidden(tick) && layer > 0) {
                    continue;
                }
                let tag = match layer {
                    0 if r.hidden(tick) => format!("seedhint_l{look}"),
                    0 if matches!(r.kind, R_SLASH | R_MIRROR) => format!("rift_{}_l{look}_a{}", RIFTS[r.kind].id, heading8(r.ex - r.x, r.ey - r.y) % 4),
                    0 => format!("rift_{}_l{look}", RIFTS[r.kind].id),
                    1 => format!("core_f{}_t{tier}", DIMS[r.dim].family),
                    _ => format!("glyph{:02}", r.dim),
                };
                let (x, y) = if matches!(r.kind, R_SLASH | R_MIRROR) { ((r.x + r.ex) / 2, (r.y + r.ey) / 2) } else { (r.x, r.y) };
                let host = r.host;
                if self.fx_low(out, tag, x, y, host, 48) {
                    self.rifts[i].drawn[layer] = tick + 48;
                }
            }
        }
        // link threads along the chain, from Vesper through every node
        let nodes = self.chain_nodes(tick);
        let mut prev = (t.me.x, t.me.y);
        for n in &nodes {
            let due = self.rift(n.uid).is_some_and(|r| tick >= r.drawn[3]);
            if due {
                let dir = heading8(n.x - prev.0, n.y - prev.1);
                if self.fx_low(out, format!("link_l{look}_a{dir}"), (n.x + prev.0) / 2, (n.y + prev.1) / 2, None, 48) {
                    if let Some(r) = self.rifts.iter_mut().find(|r| r.uid == n.uid && !r.done) {
                        r.drawn[3] = tick + 48;
                    }
                }
            }
            prev = (n.x, n.y);
        }
    }
}

/// The dimension picker's view of the fight around a spot, read once per pick.
#[derive(Clone, Copy, Debug)]
pub struct PickCtx {
    pub allies_near: i64,
    pub enemies_near: i64,
    /// Some ally under half health.
    pub ally_low: bool,
    /// Some ally within 50000 of Vesper (a grab or pull has someone to deliver to).
    pub ally_by_me: bool,
    /// The best case anywhere instead of this spot: every ally and enemy near, every target bonus earned.
    pub bound: bool,
}
impl PickCtx {
    pub fn read(t: &Tick, at: (i64, i64)) -> PickCtx {
        PickCtx {
            allies_near: t.allies().filter(|a| dist(a.x, a.y, at.0, at.1) <= 50000).count() as i64,
            enemies_near: t.enemies().filter(|o| dist(o.x, o.y, at.0, at.1) <= 30000).count() as i64,
            ally_low: t.allies().any(|a| a.hp * 2 < a.max_hp),
            ally_by_me: t.allies().any(|a| dist(a.x, a.y, t.me.x, t.me.y) <= 50000),
            bound: false,
        }
    }
    /// An upper bound on every spot's value: the planner skips a dimension whose bound cannot win.
    pub fn bound(t: &Tick) -> PickCtx {
        PickCtx { allies_near: t.allies().count() as i64, enemies_near: t.enemies().count() as i64, ally_low: true, ally_by_me: true, bound: true }
    }
}

/// What the planner sees when it thinks.
#[derive(Clone, Debug)]
pub struct View {
    pub enemies: Vec<Fighter>,
    pub allies: Vec<Fighter>,
    pub focus: Fighter,
    pub tank: usize,
    pub ally_centre: (i64, i64),
    pub enemy_centre: (i64, i64),
    pub front: Option<Fighter>,
    pub diver: bool,
    /// The visible live rifts and the chain, gathered once per think for every proposal.
    pub live: Vec<Rift>,
    pub nodes: Vec<Rift>,
}
fn centre(v: &[Fighter], or: (i64, i64)) -> (i64, i64) {
    if v.is_empty() {
        return or;
    }
    let n = v.len() as i64;
    (v.iter().map(|f| f.x).sum::<i64>() / n, v.iter().map(|f| f.y).sum::<i64>() / n)
}
fn is_control(p: u8) -> bool {
    matches!(p, P_BANISH | P_STUN | P_ROOT | P_SILENCE | P_DISARM | P_ENCASE | P_STATICBURST | P_CHILL)
}
fn is_damage(p: u8) -> bool {
    matches!(p, P_AMPLIFY | P_DOT | P_TRUE | P_LENS | P_EXECUTE | P_SPLASH | P_RICOCHET | P_CHAIN | P_ECHO | P_PRISM | P_WILDECHO | P_MARK | P_DRAIN | P_ENTROPY | P_SHATTER | P_STATDOWN | P_HEALCUT | P_MOMENTUM | P_PRESSURE | P_EXPOSE)
}
fn is_move(p: u8) -> bool {
    matches!(p, P_REFLECT | P_PULL | P_KNOCK | P_SHOVE | P_STORM | P_GRAB | P_REWIND | P_TOEND | P_SELFSWAP | P_SCRAMBLE | P_SWAPSPACE | P_TETHER | P_WELL | P_REPEL)
}

impl Core {
    fn view(&self, t: &Tick, enemies: Vec<Fighter>) -> View {
        let allies: Vec<Fighter> = t.allies().cloned().collect();
        let tank = enemies.iter().max_by_key(|e| (e.max_hp, e.id)).map_or(0, |e| e.id);
        let focus = if self.gate(GATE_PRIORITY) {
            enemies.iter().min_by_key(|e| (self.priority_rank(e, tank, t), e.id)).cloned()
        } else {
            enemies.iter().min_by_key(|e| (dist2(e.x, e.y, t.me.x, t.me.y), e.id)).cloned()
        }
        .unwrap_or_else(|| t.me.clone());
        let me = (t.me.x, t.me.y);
        let front = allies.iter().max_by_key(|a| (a.max_hp, a.id)).cloned();
        View {
            ally_centre: centre(&allies, me),
            enemy_centre: centre(&enemies, me),
            diver: t.danger(t.me.x, t.me.y, DIVE_RADIUS) > 0,
            focus,
            tank,
            front,
            allies,
            enemies,
            live: self.rifts.iter().filter(|r| r.live() && !r.hidden(t.tick)).cloned().collect(),
            nodes: self.chain_nodes(t.tick),
        }
    }
    /// Lower is a better target: low health, high damage, never the tank.
    fn priority_rank(&self, e: &Fighter, tank: usize, t: &Tick) -> i64 {
        let hp = e.hp * 100 / e.max_hp;
        hp - e.attack / 6 + if e.id == tank { 80 } else { 0 } + dist(e.x, e.y, t.me.x, t.me.y) / 20000
    }
    fn prio(&self, e: &Fighter, v: &View) -> i64 {
        if self.gate(GATE_PRIORITY) {
            30 + (100 - e.hp * 100 / e.max_hp.max(1)) / 2 + if e.id == v.tank { -40 } else { 0 }
        } else {
            20
        }
    }
    /// Where a fighter will be when a rift opening `delay` ticks from now arrives: a Spark aims at where they stand.
    fn lead(&self, f: &Fighter, delay: usize) -> (i64, i64) {
        if self.rank == 0 {
            return (f.x, f.y);
        }
        self.forecast(f, (delay as i64).min(self.table(&LOOK_AHEAD)))
    }
    fn delay_to(&self, t: &Tick, x: i64, y: i64, kind: usize) -> usize {
        open_delay(dist(t.me.x, t.me.y, x, y), RIFTS[kind].delay_pct)
    }
    /// From Safety up: rifts stay off the paths enemies are walking (they would be broken).
    fn off_path(&self, t: &Tick, x: i64, y: i64) -> bool {
        if !self.gate(GATE_SAFETY) {
            return true;
        }
        !t.enemies().any(|e| {
            let (fx, fy) = self.forecast(e, 60);
            seg_dist(e.x, e.y, fx, fy, x, y) <= BREAK_CORE * 2
        })
    }
    fn exit_ok(&self, t: &Tick, x: i64, y: i64) -> bool {
        !self.gate(GATE_SAFETY) || t.danger(x, y, 35000) < 3
    }
    /// How much the planner wants one effect for a purpose. `craft` (the ORB_CRAFT table) decides how much of the
    /// fight it reads: 0 only the effect's side, 1 its kind, 2 the target and the people around, 3 the combo too.
    /// `ctx` is the fight around the spot, read once per pick, or its `bound`: the best this effect could be worth.
    fn eff_value(&self, t: &Tick, ctx: &PickCtx, e: Eff, purpose: Purpose, target: Option<&Fighter>, craft: i64) -> i64 {
        let p = e.prim;
        if p == 255 {
            return 0;
        }
        let side = PRIM_SIDE[p as usize];
        let foe = matches!(purpose, Purpose::Punch | Purpose::Swipe | Purpose::Enemy);
        if purpose == Purpose::Route {
            return if side == b'P' { 16 } else { 3 };
        }
        let heal = matches!(p, P_HEAL | P_SHIELD | P_SHELTER | P_UNDYING | P_RECALLHP);
        if purpose == Purpose::Ally {
            return match side {
                b'A' => 14 + if craft >= 2 && heal && ctx.ally_low { 10 } else { 0 },
                b'S' => 6,
                b'X' => 4,
                _ => 2,
            };
        }
        if craft == 0 {
            return match side {
                b'E' | b'P' if foe => 12,
                b'X' => 6,
                _ => 3,
            };
        }
        let mut v = match side {
            b'A' => {
                if craft >= 2 {
                    if ctx.allies_near > 0 {
                        6 + 4 * ctx.allies_near
                    } else {
                        -4
                    }
                } else {
                    6
                }
            }
            b'S' => 6,
            _ => {
                if p == P_CHILL {
                    10 // a freeze takes several stacks
                } else if is_control(p) {
                    16
                } else if is_damage(p) {
                    14
                } else if is_move(p) {
                    6
                } else {
                    8
                }
            }
        };
        if craft >= 2 {
            match (ctx.bound, target) {
                (false, Some(f)) => {
                    let low = f.hp * 100 < f.max_hp * 40;
                    if is_control(p) && f.cc_immune {
                        v -= 28;
                    }
                    if is_control(p) && f.controlled {
                        v -= 10;
                    }
                    if matches!(p, P_EXECUTE | P_AMPLIFY) && low {
                        v += 8;
                    }
                    if p == P_HEALCUT && f.hp * 2 < f.max_hp {
                        v += 6;
                    }
                    if matches!(p, P_GRAB | P_PULL) && ctx.ally_by_me {
                        v += 12;
                    }
                    if matches!(p, P_KNOCK | P_REWIND) {
                        v += if dist(f.x, f.y, t.me.x, t.me.y) <= DIVE_RADIUS { 12 } else { -2 };
                    }
                    // Momentum is worth nothing to an athlete standing still
                    if p == P_MOMENTUM && self.momentum(t, e.a, e.b) == 0 {
                        v -= 10;
                    }
                }
                (true, _) => {
                    // the bound: every bonus this effect could earn against some target
                    v += match p {
                        P_EXECUTE | P_AMPLIFY => 8,
                        P_HEALCUT => 6,
                        P_GRAB | P_PULL | P_KNOCK | P_REWIND => 12,
                        _ => 0,
                    };
                }
                _ => {}
            }
            if target.is_some() && p == P_SELFSWAP {
                v -= 8;
            }
            if matches!(p, P_SPLASH | P_CHAIN | P_RICOCHET | P_MARK | P_PRISM) {
                v += 3 * (ctx.enemies_near - 1).max(0);
            }
            if p == P_UNSTABLE {
                v += if self.stress > STRESS_CAP * 3 / 5 { -25 } else { 4 };
            }
            if p == P_VENT {
                v += self.stress / 5;
            }
        }
        v
    }
    fn deliveries(dim: usize, purpose: Purpose) -> (Eff, Eff) {
        match purpose {
            Purpose::Punch => (PUNCH[dim].main, PUNCH[dim].rider),
            Purpose::Swipe => (SWIPE[dim].main, SWIPE[dim].rider),
            _ => {
                let d = DIMS[dim];
                (Eff { prim: d.prim, a: d.a, b: d.b, c: d.c }, NO_EFF)
            }
        }
    }
    pub fn dim_value(&self, t: &Tick, ctx: &PickCtx, dim: usize, purpose: Purpose, target: Option<&Fighter>, craft: i64) -> i64 {
        let (main, rider) = Core::deliveries(dim, purpose);
        let mut v = self.eff_value(t, ctx, main, purpose, target, craft) + self.eff_value(t, ctx, rider, purpose, target, craft) / 2;
        // the deepest planners chain families for their pair bonus
        if craft >= 3 && t.tick <= self.combo_at + COMBO_WINDOW as usize {
            if let Some(prev) = self.last_family {
                let fam = DIMS[dim].family;
                if fam != prev {
                    v += 6 + if matches!(TWISTS[twist_index(prev, fam)].kind, 0 | 1 | 5) { 4 } else { 0 };
                }
            }
        }
        v
    }
    /// Whether a dimension may be cast now: its own cooldown and its family's recovery are over.
    pub fn cast_ok(&self, dim: usize, tick: usize) -> bool {
        self.free_casts || (tick >= self.dim_ready[dim] && tick >= self.family_ready[DIMS[dim].family])
    }
    /// The dimensions an orb may open now.
    pub fn available_dims(&self, tick: usize) -> Vec<usize> {
        (0..DIMS.len()).filter(|&d| self.cast_ok(d, tick)).collect()
    }
    /// A cast committed: every dimension it opened goes on its cooldown and its family rests. The rifts of one
    /// action and a fused punch's two orbs lock together, once.
    fn commit_cast(&mut self, dims: &[usize], tick: usize) {
        if self.free_casts {
            return;
        }
        for &d in dims {
            self.dim_ready[d] = self.dim_ready[d].max(tick + DIMS[d].cooldown);
            let f = DIMS[d].family;
            self.family_ready[f] = self.family_ready[f].max(tick + FAMILY_RECOVERY as usize);
        }
    }
    /// Whether an action can get every dimension it needs: its chosen ones are ready, and something is ready for
    /// the rest.
    fn dims_for(&self, a: &Action, tick: usize) -> bool {
        let needs = |spec: Option<usize>| spec.map_or_else(|| (0..DIMS.len()).any(|d| self.cast_ok(d, tick)), |d| self.cast_ok(d, tick));
        if a.tech == M_SWIPE {
            return needs(a.dim);
        }
        a.tech >= 28 || a.rifts.iter().all(|r| needs(r.dim))
    }
    /// The dimension an orb opens. A Spark picks any available one; better athletes read the fight (`ORB_CRAFT`).
    /// The fight around the spot is read once. Each dimension draws its noise in order (the same stream as a full
    /// scan); one whose bound plus noise cannot beat the best so far is skipped without being scored, so the result
    /// is exactly the full scan's: the highest value, ties to the lowest index.
    pub fn pick_dim(&mut self, t: &Tick, purpose: Purpose, target: Option<&Fighter>, at: (i64, i64)) -> usize {
        let avail = self.available_dims(t.tick);
        if avail.is_empty() {
            return 0;
        }
        if self.rank == 0 {
            return avail[self.random(avail.len() as u64) as usize];
        }
        #[cfg(test)]
        let started = std::time::Instant::now();
        let craft = self.table(&ORB_CRAFT);
        let noise = self.table(&NOISE_PCT);
        let ctx = PickCtx::read(t, at);
        let bound = PickCtx::bound(t);
        let mut best: Option<(i64, usize)> = None;
        for dim in avail {
            let n = if noise > 0 { self.random((2 * noise + 1) as u64) as i64 - noise } else { 0 };
            if let Some(b) = best {
                if self.dim_value(t, &bound, dim, purpose, target, craft) + n <= b.0 {
                    continue;
                }
            }
            let v = self.dim_value(t, &ctx, dim, purpose, target, craft) + n;
            if best.is_none_or(|b| v > b.0) {
                best = Some((v, dim));
            }
        }
        let pick = best.map_or(0, |b| b.1);
        #[cfg(test)]
        {
            self.stats.picks += 1;
            self.stats.pick_nanos += started.elapsed().as_nanos() as u64;
        }
        if matches!(purpose, Purpose::Punch | Purpose::Swipe) {
            self.habit("dimension picked for the target");
            if craft >= 3 && self.last_family.is_some_and(|f| f != DIMS[pick].family) && t.tick <= self.combo_at + COMBO_WINDOW as usize {
                self.habit("combo families for the twist");
            }
        }
        pick
    }
    /// A chain from Vesper to a punching spot within range of `target`.
    fn plan_chain(&self, t: &Tick, target: &Fighter, kind_mid: usize) -> Option<(Vec<RiftSpec>, i64, usize)> {
        let guess = self.lead(target, 40);
        let reach = ATTACK_RANGE * 7 / 10;
        let d = dist(t.me.x, t.me.y, guess.0, guess.1);
        if d <= PUNCH_REACH {
            return None;
        }
        let exit = along(guess.0, guess.1, t.me.x - guess.0, t.me.y - guess.1, reach);
        let span = dist(t.me.x, t.me.y, exit.0, exit.1);
        let max = if self.gate(GATE_GEOMETRY) { self.table(&MAX_LINKS) as usize } else { 1 };
        let mut best = (i64::MIN, 1usize, 0i64);
        for n in 1..=max {
            let eff = chain_bp(&vec![span / n as i64; n]);
            let score = eff - n as i64 * 400;
            if score > best.0 {
                best = (score, n, eff);
            }
        }
        let n = best.1;
        let (px, py) = (-(exit.1 - t.me.y), exit.0 - t.me.x);
        let mut nodes = vec![];
        for i in 1..=n as i64 {
            let mut p = (t.me.x + (exit.0 - t.me.x) * i / n as i64, t.me.y + (exit.1 - t.me.y) * i / n as i64);
            if !self.off_path(t, p.0, p.1) {
                p = clamp_map(p.0 + along(0, 0, px, py, 24000).0, p.1 + along(0, 0, px, py, 24000).1);
            }
            let kind = if i == 1 && kind_mid != R_TEAR && n > 1 { kind_mid } else { R_TEAR };
            nodes.push(if kind == R_MIRROR {
                let (hx, hy) = along(0, 0, px, py, 20000);
                RiftSpec { chain: true, ..RiftSpec::line(R_MIRROR, p.0 - hx, p.1 - hy, p.0 + hx, p.1 + hy) }
            } else {
                RiftSpec::node(kind, p.0, p.1)
            });
        }
        // the real links, Vesper to the exit
        let mut links = vec![];
        let mut prev = (t.me.x, t.me.y);
        for s in &nodes {
            links.push(dist(prev.0, prev.1, s.x, s.y));
            prev = (s.x, s.y);
        }
        Some((nodes, chain_bp(&links), n))
    }
    fn wall_between(&self, t: &Tick, a: (i64, i64), b: (i64, i64)) -> bool {
        let steps = (dist(a.0, a.1, b.0, b.1) / 16000).max(1);
        (1..steps).any(|i| (t.wall)(a.0 + (b.0 - a.0) * i / steps, a.1 + (b.1 - a.1) * i / steps))
    }
    fn safe_back(&self, v: &View, t: &Tick, len: i64) -> (i64, i64) {
        let (dx, dy) = (v.ally_centre.0 - v.enemy_centre.0, v.ally_centre.1 - v.enemy_centre.1);
        let (dx, dy) = if dx == 0 && dy == 0 { (t.me.x - v.enemy_centre.0, t.me.y - v.enemy_centre.1) } else { (dx, dy) };
        clamp_map(along(t.me.x, t.me.y, dx, dy, len).0, along(t.me.x, t.me.y, dx, dy, len).1)
    }
    /// Where a pulled enemy should land: just in front of her frontliner (the ally nearest the enemies), so it drops
    /// into her team; when that is beyond TELEPORT_MAX, as far toward it as the drop reaches. None without allies.
    fn drop_spot(&self, v: &View, from: (i64, i64)) -> Option<(i64, i64)> {
        let front = v.allies.iter().min_by_key(|a| (dist2(a.x, a.y, v.enemy_centre.0, v.enemy_centre.1), a.id))?;
        let spot = along(front.x, front.y, from.0 - front.x, from.1 - front.y, 12000);
        let d = dist(spot.0, spot.1, from.0, from.1);
        Some(if d > TELEPORT_MAX { along(from.0, from.1, spot.0 - from.0, spot.1 - from.1, TELEPORT_MAX) } else { spot })
    }
    fn random_near(&mut self, x: i64, y: i64, r: i64) -> (i64, i64) {
        let (ux, uy) = DIRS[self.random(16) as usize];
        let k = self.random(r.max(1) as u64) as i64;
        clamp_map(x + ux * k / 10000, y + uy * k / 10000)
    }
    /// Candidate actions for one technique or gauntlet move. Everything can be proposed at every rank; the gates
    /// and tables only shape where it goes and how much the planner likes it.
    pub fn propose(&mut self, t: &Tick, v: &View, tech: usize) -> Vec<Action> {
        if tech >= 28 {
            return self.propose_move(t, v, tech);
        }
        let tick = t.tick;
        let me = (t.me.x, t.me.y);
        let f = v.focus.clone();
        let act = |rifts: Vec<RiftSpec>, subject: Option<usize>, uses: Vec<u32>, score: i64, habit: &'static str| Action { subject, uses, score, habit, ..Action::new(tech, rifts) };
        let (live, nodes) = (&v.live, &v.nodes);
        let chain_live = !nodes.is_empty();
        let mut out = vec![];
        match tech {
            T_CHAIN => {
                if chain_live {
                    if self.gate(GATE_SPINE) {
                        if let Some(n) = nodes.iter().find(|n| n.expires <= tick + 60) {
                            out.push(act(vec![RiftSpec { order: n.chain, ..RiftSpec::node(R_TEAR, n.x, n.y) }], None, vec![], 46, "spine refreshed"));
                        }
                    }
                } else if let Some((rifts, eff, n)) = self.plan_chain(t, &f, R_TEAR) {
                    let habit = if n >= 2 { "solved the link count" } else if self.rank >= 1 { "led target" } else { "" };
                    out.push(act(rifts, Some(f.id), vec![], 28 + eff / 300, habit));
                }
            }
            T_BANK => {
                if !chain_live {
                    let walled = self.wall_between(t, me, (f.x, f.y));
                    if let Some((mut rifts, eff, n)) = self.plan_chain(t, &f, R_MIRROR) {
                        if n == 1 {
                            // one link: the pane itself is the node
                            let p = (rifts[0].x, rifts[0].y);
                            let (ox, oy) = if self.gate(GATE_SAFETY) { along(0, 0, -(p.1 - me.1), p.0 - me.0, 20000) } else { DIRS[self.random(16) as usize] };
                            let (ox, oy) = if self.gate(GATE_SAFETY) { (ox, oy) } else { (ox * 2, oy * 2) };
                            rifts[0] = RiftSpec { chain: true, ..RiftSpec::line(R_MIRROR, p.0 - ox, p.1 - oy, p.0 + ox, p.1 + oy) };
                        }
                        let score = if self.gate(GATE_GEOMETRY) { if walled { 40 + eff / 400 } else { 0 } } else { 12 + eff / 600 };
                        out.push(act(rifts, Some(f.id), vec![], score, if walled { "bank shot around a wall" } else { "" }));
                    }
                }
            }
            T_CROSSFIRE => {
                if chain_live && live.len() >= 2 && tick >= self.crossfire_until {
                    let covered = live.iter().filter(|r| t.enemies().any(|e| dist(e.x, e.y, r.x, r.y) <= ATTACK_RANGE)).count() as i64;
                    let score = if self.gate(GATE_GEOMETRY) { 12 * covered.min(3) } else { 18 };
                    out.push(act(vec![], None, vec![], score, if covered >= 2 { "crossfire split" } else { "" }));
                }
            }
            T_RAIN => {
                let mut spot = (f.x, f.y);
                let mut count = 0i64;
                if self.gate(GATE_GEOMETRY) {
                    for e in &v.enemies {
                        let p = self.lead(e, self.delay_to(t, e.x, e.y, R_SKY));
                        let c = v.enemies.iter().filter(|o| dist(o.x, o.y, p.0, p.1) <= 26000).count() as i64;
                        if c > count {
                            count = c;
                            spot = p;
                        }
                    }
                } else {
                    count = v.enemies.iter().filter(|o| dist(o.x, o.y, spot.0, spot.1) <= 26000).count() as i64;
                }
                out.push(act(vec![RiftSpec::at(R_SKY, spot.0, spot.1)], None, vec![], 14 * count - 4, if count >= 2 { "rain on a cluster" } else { "" }));
            }
            T_VOLLEY => {
                let p = self.lead(&f, self.delay_to(t, f.x, f.y, R_SLASH));
                let (dx, dy) = if self.gate(GATE_GEOMETRY) { (-(p.1 - me.1), p.0 - me.0) } else { DIRS[self.random(16) as usize] };
                let a = along(p.0, p.1, dx, dy, 30000);
                let b = along(p.0, p.1, -dx, -dy, 30000);
                let n = v.enemies.iter().filter(|e| seg_dist(a.0, a.1, b.0, b.1, e.x, e.y) <= 22000).count() as i64;
                out.push(act(vec![RiftSpec::line(R_SLASH, a.0, a.1, b.0, b.1)], None, vec![], 10 * n + 2, ""));
            }
            T_IMPLODE => {
                if !live.is_empty() {
                    let doomed = live.iter().filter(|r| r.dwell.iter().any(|(id, n)| *id < 1_000_000 && *n >= 20)).max_by_key(|r| r.uid).cloned();
                    if let (true, Some(r)) = (self.gate(GATE_SAFETY), doomed) {
                        let n = t.enemies().filter(|e| r.reach(e.x, e.y) <= r.radius * 3 / 2).count() as i64;
                        out.push(act(vec![], None, vec![r.uid], 34 + 6 * n, "collapsed a rift before it broke"));
                    }
                    let pick = if self.gate(GATE_GEOMETRY) {
                        live.iter().max_by_key(|r| (t.enemies().filter(|e| r.reach(e.x, e.y) <= r.radius * 3 / 2).count(), r.uid)).cloned()
                    } else {
                        let i = self.random(live.len() as u64) as usize;
                        Some(live[i].clone())
                    };
                    if let Some(r) = pick {
                        let n = t.enemies().filter(|e| r.reach(e.x, e.y) <= r.radius * 3 / 2).count() as i64;
                        let low = t.enemies().any(|e| r.reach(e.x, e.y) <= r.radius * 3 / 2 && e.hp * 4 < e.max_hp);
                        let score = if self.gate(GATE_GEOMETRY) { if n >= 2 || low { 16 * n + 6 } else { -20 } } else { 10 + 8 * n };
                        out.push(act(vec![], None, vec![r.uid], score, if n >= 2 { "implode on a pile" } else { "" }));
                    }
                }
            }
            T_DOMINO => {
                let linked: Vec<&Rift> = live.iter().filter(|r| r.chain.is_some() || r.kind == R_TWIN).collect();
                if linked.len() >= 2 {
                    let total = linked.iter().map(|r| t.enemies().filter(|e| r.reach(e.x, e.y) <= r.radius * 3 / 2).count()).sum::<usize>() as i64;
                    let score = if self.gate(GATE_GEOMETRY) { if total >= 2 { 9 * total + 8 } else { -20 } } else { 12 };
                    let mut uses: Vec<u32> = linked.iter().map(|r| r.uid).collect();
                    uses.sort();
                    out.push(act(vec![], None, uses, score, if total >= 2 { "domino through the pile" } else { "" }));
                }
            }
            T_OPUNCH => {
                if self.punching && self.stress >= STRESS_CAP / 4 && self.opunch == 0 {
                    let spend = self.stress.min(STRESS_MAX_OVERLOAD);
                    let score = if self.gate(GATE_SAFETY) { spend / 3 + if self.stress >= STRESS_CAP * 7 / 10 { 12 } else { 0 } } else { 14 };
                    out.push(act(vec![], None, vec![], score, ""));
                }
            }
            T_STEP => {
                if tick >= self.self_move_until {
                    if self.gate(GATE_SURVIVAL) && v.diver {
                        let to = self.safe_back(v, t, 60000);
                        out.push(act(vec![RiftSpec::entrance(R_TEAR, me.0, me.1), RiftSpec::at(R_TEAR, to.0, to.1)], None, vec![], 55, "escaped a dive"));
                    } else {
                        let to = along(f.x, f.y, me.0 - f.x, me.1 - f.y, 15000);
                        if self.exit_ok(t, to.0, to.1) {
                            let score = if self.gate(GATE_SURVIVAL) { 0 } else { 16 };
                            out.push(act(vec![RiftSpec::entrance(R_TEAR, me.0, me.1), RiftSpec::at(R_TEAR, to.0, to.1)], None, vec![], score, ""));
                        }
                    }
                }
            }
            T_GATE | T_SKYDROP => {
                let exit_kind = if tech == T_SKYDROP { R_SKY } else { R_TEAR };
                if self.gate(GATE_TEAM) {
                    if let Some(fr) = v.front.clone() {
                        let (vx, vy) = self.velocity.get(&fr.id).copied().unwrap_or((0, 0));
                        let toward = vx as i128 * (v.enemy_centre.0 - fr.x) as i128 + vy as i128 * (v.enemy_centre.1 - fr.y) as i128 > 0;
                        let close = dist(fr.x, fr.y, v.enemy_centre.0, v.enemy_centre.1) < 170000 && t.danger(fr.x, fr.y, 45000) == 0;
                        let wait = self.table(&ALLY_WAIT) as usize;
                        let ent = self.lead(&fr, self.delay_to(t, fr.x, fr.y, R_TEAR) + wait);
                        let (dx, dy) = (v.enemy_centre.0 - v.ally_centre.0, v.enemy_centre.1 - v.ally_centre.1);
                        let behind = clamp_map(along(v.enemy_centre.0, v.enemy_centre.1, dx, dy, 18000).0, along(v.enemy_centre.0, v.enemy_centre.1, dx, dy, 18000).1);
                        let cluster = v.enemies.iter().filter(|e| dist(e.x, e.y, behind.0, behind.1) <= 40000).count() as i64;
                        let close = close && dist(ent.0, ent.1, behind.0, behind.1) <= TELEPORT_MAX;
                        let score = if toward && close { if tech == T_SKYDROP { 40 + 6 * cluster } else { 50 } } else { 8 };
                        out.push(act(vec![RiftSpec::entrance(R_TEAR, ent.0, ent.1), RiftSpec::at(exit_kind, behind.0, behind.1)], Some(fr.id), vec![], score, if toward && close { "Gate synced with the tank" } else { "" }));
                    }
                } else if !v.allies.is_empty() {
                    let i = self.random(v.allies.len() as u64) as usize;
                    let a = v.allies[i].clone();
                    out.push(act(vec![RiftSpec::entrance(R_TEAR, a.x, a.y), RiftSpec::at(exit_kind, f.x, f.y)], Some(a.id), vec![], 16, ""));
                }
            }
            T_KIDNAP => {
                let subject = f.clone();
                let front = along(subject.x, subject.y, me.0 - subject.x, me.1 - subject.y, KIDNAP_FRONT);
                // a careful Kidnapper only blinks where at most one other enemy can punish her; a team player only on a lone target
                let crowd = t.danger(front.0, front.1, 50000);
                let exposed = self.gate(GATE_SAFETY) && crowd > 2;
                let tank = self.gate(GATE_PRIORITY) && subject.id == v.tank;    // the tank is not worth a blink
                if tick >= self.kidnap_ready && dist(me.0, me.1, subject.x, subject.y) <= KIDNAP_REACH && !exposed && !tank && !(self.gate(GATE_SAFETY) && subject.cc_immune) {
                    let ent = me;
                    let exit = if self.gate(GATE_TEAM) && !v.allies.is_empty() {
                        // just in front of her frontliner: the target lands in her team and she stays in the fight
                        self.drop_spot(v, (subject.x, subject.y))
                    } else if self.gate(GATE_SAFETY) {
                        let p = self.safe_back(v, t, 45000);
                        if dist(p.0, p.1, me.0, me.1) > 25000 {
                            Some(p)
                        } else {
                            None
                        }
                    } else {
                        // a Spark kidnaps on a whim, to wherever the tear lands
                        Some(self.random_near(subject.x, subject.y, 60000))
                    };
                    if let Some(exit) = exit {
                        let score = if self.gate(GATE_TEAM) { 44 + self.prio(&subject, v) / 4 } else { 20 };
                        let habit = if self.gate(GATE_PRIORITY) && subject.id != v.tank { "Kidnapped the carry into the team" } else { "" };
                        out.push(act(vec![RiftSpec::entrance(R_TEAR, ent.0, ent.1), RiftSpec::at(R_TEAR, exit.0, exit.1)], Some(subject.id), vec![], score, habit));
                    }
                }
            }
            T_SINKTRAP => {
                let spot = if self.gate(GATE_PRIORITY) { lerp(v.enemy_centre.0, v.enemy_centre.1, v.ally_centre.0, v.ally_centre.1, 35) } else { self.lead(&f, self.delay_to(t, f.x, f.y, R_SINKHOLE)) };
                let n = v.enemies.iter().filter(|e| dist(e.x, e.y, spot.0, spot.1) <= 40000).count() as i64;
                let exit = if self.gate(GATE_TEAM) {
                    self.drop_spot(v, spot).unwrap_or(v.ally_centre)
                } else {
                    me
                };
                out.push(act(vec![RiftSpec::entrance(R_SINKHOLE, spot.0, spot.1), RiftSpec::at(R_TEAR, exit.0, exit.1)], None, vec![], 9 * n + 10, if self.gate(GATE_PRIORITY) { "Sinkhole at the chokepoint" } else { "" }));
            }
            T_SWAP => {
                if self.gate(GATE_TEAM) {
                    if let Some(fr) = v.front.clone() {
                        let a = self.lead(&f, self.delay_to(t, f.x, f.y, R_TWIN));
                        let b = self.lead(&fr, self.delay_to(t, fr.x, fr.y, R_TWIN));
                        if dist(a.0, a.1, b.0, b.1) > TELEPORT_MAX {
                            return out;
                        }
                        out.push(act(vec![RiftSpec::entrance(R_TWIN, a.0, a.1), RiftSpec::at(R_TWIN, b.0, b.1)], Some(f.id), vec![], 36 + self.prio(&f, v) / 6, "swapped the target into the team"));
                    }
                } else if !self.gate(GATE_SAFETY) || t.danger(f.x, f.y, 35000) < 3 {
                    out.push(act(vec![RiftSpec::entrance(R_TWIN, me.0, me.1), RiftSpec::at(R_TWIN, f.x, f.y)], Some(f.id), vec![], 18, ""));
                }
            }
            T_FLING => {
                let start = self.lead(&f, self.delay_to(t, f.x, f.y, R_WANDER));
                let drop = if self.gate(GATE_TEAM) { self.drop_spot(v, start) } else { None };
                let end = match drop {
                    Some(end) => end,
                    None => {
                        let (dx, dy) = if self.gate(GATE_TEAM) { (v.ally_centre.0 - start.0, v.ally_centre.1 - start.1) } else { DIRS[self.random(16) as usize] };
                        clamp_map(along(start.0, start.1, dx, dy, TELEPORT_MAX).0, along(start.0, start.1, dx, dy, TELEPORT_MAX).1)
                    }
                };
                let n = v.enemies.iter().filter(|e| dist(e.x, e.y, start.0, start.1) <= 30000).count() as i64;
                out.push(act(vec![RiftSpec::line(R_WANDER, start.0, start.1, end.0, end.1)], None, vec![], 10 * n + 4, if self.gate(GATE_TEAM) { "Fling toward the team" } else { "" }));
            }
            T_HOOK => {
                if !(self.gate(GATE_SAFETY) && f.cc_immune) {
                    let anchor = RiftSpec { host: Some(f.id), ..RiftSpec::at(R_ANCHOR, f.x, f.y) };
                    let mut rifts = vec![anchor];
                    if self.gate(GATE_TEAM) {
                        // a team player reels the target to the front of her team
                        if let Some(exit) = self.drop_spot(v, (f.x, f.y)) {
                            rifts.push(RiftSpec::at(R_TEAR, exit.0, exit.1));
                        }
                    }
                    out.push(act(rifts, Some(f.id), vec![], 22 + self.prio(&f, v) / 6, ""));
                }
            }
            T_PHASE => {
                if tick >= self.self_move_until {
                    let hurt = t.me.hp * 2 < t.me.max_hp && t.danger(me.0, me.1, 40000) >= 1;
                    let to = if self.gate(GATE_SURVIVAL) { self.safe_back(v, t, 50000) } else { self.random_near(me.0, me.1, 50000) };
                    let score = if self.gate(GATE_SURVIVAL) { if hurt { 55 } else { -10 } } else { 12 };
                    out.push(act(vec![RiftSpec::entrance(R_TEAR, me.0, me.1), RiftSpec::at(R_TEAR, to.0, to.1)], None, vec![], score, if hurt { "phased out of a burst" } else { "" }));
                }
            }
            T_CURTAIN | T_REFLECT => {
                let kind = if tech == T_CURTAIN { R_SLASH } else { R_MIRROR };
                let near = v.enemies.iter().filter(|e| dist(e.x, e.y, me.0, me.1) <= 90000).min_by_key(|e| (dist2(e.x, e.y, me.0, me.1), e.id)).cloned();
                if let Some(e) = near {
                    let mid = lerp(me.0, me.1, e.x, e.y, 45);
                    let half = RIFTS[kind].length / 2;
                    let (dx, dy) = if self.gate(if tech == T_REFLECT { GATE_SAFETY } else { GATE_SURVIVAL }) { (-(e.y - me.1), e.x - me.0) } else { DIRS[self.random(16) as usize] };
                    let a = along(mid.0, mid.1, dx, dy, half);
                    let b = along(mid.0, mid.1, -dx, -dy, half);
                    let score = if self.gate(GATE_SURVIVAL) { if v.diver { 40 } else { 0 } } else { 12 };
                    out.push(act(vec![RiftSpec::line(kind, a.0, a.1, b.0, b.1)], None, vec![], score, if v.diver { "peeled a diver" } else { "" }));
                }
            }
            T_RESCUE | T_POCKET => {
                let low = v.allies.iter().filter(|a| a.hp * 100 / a.max_hp.max(1) < if tech == T_RESCUE { 25 } else { 40 }).min_by_key(|a| (a.hp, a.id)).cloned();
                let pick = if self.gate(GATE_SURVIVAL) {
                    low
                } else if v.allies.is_empty() {
                    None
                } else {
                    let i = self.random(v.allies.len() as u64) as usize;
                    Some(v.allies[i].clone())
                };
                if let Some(a) = pick {
                    let pressed = t.danger(a.x, a.y, 40000) >= if tech == T_RESCUE { 1 } else { 2 };
                    let ent = self.lead(&a, self.delay_to(t, a.x, a.y, R_TEAR));
                    let score = if self.gate(GATE_SURVIVAL) { if pressed { 58 } else { 10 } } else { 12 };
                    let mut rifts = vec![RiftSpec::entrance(R_TEAR, ent.0, ent.1)];
                    if tech == T_RESCUE {
                        let safe = self.safe_back(v, t, 30000);
                        rifts.push(RiftSpec::at(R_TEAR, safe.0, safe.1));
                    }
                    out.push(act(rifts, Some(a.id), vec![], score, if pressed { "saved an ally" } else { "" }));
                }
            }
            T_LIFELINE => {
                if !self.rifts.iter().any(|r| !r.done && r.tech == T_LIFELINE) {
                    let close = v.enemies.iter().any(|e| dist(e.x, e.y, me.0, me.1) <= 200000);
                    let spot = if self.gate(GATE_SURVIVAL) { self.safe_back(v, t, 80000) } else { self.random_near(me.0, me.1, 80000) };
                    let score = if self.gate(GATE_SURVIVAL) { if close { 32 } else { 0 } } else { 10 };
                    out.push(act(vec![RiftSpec::at(R_ANCHOR, spot.0, spot.1)], Some(t.me.id), vec![], score, if close { "Lifeline set before the fight" } else { "" }));
                }
            }
            T_ESCORT => {
                if !chain_live {
                    let pick = if self.gate(GATE_TEAM) {
                        v.front.clone()
                    } else if v.allies.is_empty() {
                        None
                    } else {
                        let i = self.random(v.allies.len() as u64) as usize;
                        Some(v.allies[i].clone())
                    };
                    if let Some(a) = pick {
                        let diving = dist(a.x, a.y, v.enemy_centre.0, v.enemy_centre.1) < 90000;
                        let score = if self.gate(GATE_TEAM) { if diving { 42 } else { 6 } } else { 12 };
                        out.push(act(vec![RiftSpec { host: Some(a.id), chain: true, ..RiftSpec::at(R_ANCHOR, a.x, a.y) }], Some(a.id), vec![], score, if diving { "Escort on the diver" } else { "" }));
                    }
                }
            }
            T_ROTATION => {
                let far = v.allies.iter().filter(|a| dist(a.x, a.y, v.enemy_centre.0, v.enemy_centre.1) > 220000).min_by_key(|a| a.id).cloned();
                let fighting = v.allies.iter().filter(|a| dist(a.x, a.y, v.enemy_centre.0, v.enemy_centre.1) <= 120000).count();
                if let Some(a) = far {
                    let score = if self.gate(GATE_TEAM) { if fighting >= 1 { 30 } else { 4 } } else { 8 };
                    let ent = self.lead(&a, self.delay_to(t, a.x, a.y, R_TWIN));
                    let near_fight = lerp(v.enemy_centre.0, v.enemy_centre.1, me.0, me.1, 50);
                    out.push(act(vec![RiftSpec::entrance(R_TWIN, ent.0, ent.1), RiftSpec::at(R_TWIN, near_fight.0, near_fight.1)], Some(a.id), vec![], score, if fighting >= 1 { "rotated an ally into the fight" } else { "" }));
                }
            }
            T_SEEDTRAP => {
                let subject = f.clone();
                let spot = if self.rank == 0 { self.random_near(subject.x, subject.y, 60000) } else { self.forecast(&subject, self.table(&LOOK_AHEAD) * 2) };
                let mut rifts = vec![RiftSpec::at(R_SEED, spot.0, spot.1)];
                if self.gate(GATE_TEAM) {
                    if let Some(exit) = self.drop_spot(v, spot) {
                        rifts.push(RiftSpec::at(R_TEAR, exit.0, exit.1));
                    }
                }
                let score = 22 + if self.gate(GATE_PRIORITY) { 10 } else { 0 };
                out.push(act(rifts, Some(subject.id), vec![], score, if self.gate(GATE_PRIORITY) { "Seed on the carry's path" } else { "" }));
            }
            T_SWEEP => {
                let c = v.enemy_centre;
                let (dx, dy) = (-(c.1 - me.1), c.0 - me.0);
                let a = along(c.0, c.1, dx, dy, 50000);
                let b = along(c.0, c.1, -dx, -dy, 50000);
                let n = v.enemies.iter().filter(|e| seg_dist(a.0, a.1, b.0, b.1, e.x, e.y) <= 20000).count() as i64;
                out.push(act(vec![RiftSpec::line(R_WANDER, a.0, a.1, b.0, b.1)], None, vec![], 8 * n, ""));
            }
            T_INFUSE => {
                if self.hex_ready(tick) > 0 && !live.is_empty() {
                    let pick = if self.gate(GATE_PRIORITY) {
                        let frontliner = v.front.as_ref().map(|a| a.id);
                        live.iter().filter(|r| r.infused.is_none()).max_by_key(|r| (i64::from(r.host.is_some() && r.host == frontliner) * 3 + t.danger(r.x, r.y, r.radius * 2) as i64, r.uid)).cloned()
                    } else {
                        let i = self.random(live.len() as u64) as usize;
                        Some(live[i].clone())
                    };
                    if let Some(r) = pick {
                        let busy = t.danger(r.x, r.y, r.radius * 2) as i64 + t.allies().filter(|a| r.covers(a.x, a.y)).count() as i64;
                        let front = self.gate(GATE_PRIORITY) && r.host.is_some() && r.host == v.front.as_ref().map(|a| a.id);
                        let score = if self.gate(GATE_PRIORITY) { 8 * busy + if front { 20 } else { 0 } } else { 12 };
                        out.push(act(vec![], None, vec![r.uid], score, if front { "Infused the frontliner's rift" } else { "" }));
                    }
                }
            }
            T_STITCH => {
                let tears: Vec<&Rift> = live.iter().filter(|r| r.kind == R_TEAR && r.dim2.is_none()).collect();
                let mut pair = None;
                for (i, a) in tears.iter().enumerate() {
                    for b in tears.iter().skip(i + 1) {
                        if dist(a.x, a.y, b.x, b.y) <= 40000 {
                            let fresh = DIMS[a.dim].family != DIMS[b.dim].family;
                            if pair.is_none() || (fresh && self.gate(GATE_PRIORITY)) {
                                pair = Some((a.uid, b.uid, fresh));
                            }
                        }
                    }
                }
                if let Some((a, b, fresh)) = pair {
                    let score = if self.gate(GATE_PRIORITY) { if fresh { 26 } else { 4 } } else { 12 };
                    out.push(act(vec![], None, vec![a, b], score, if fresh { "stitched two worlds" } else { "" }));
                }
            }
            _ => {}
        }
        out
    }
    /// The gauntlet moves: Swipe, Shatter, Overload, Vent, Awaken, Overclock.
    fn propose_move(&mut self, t: &Tick, v: &View, m: usize) -> Vec<Action> {
        let tick = t.tick;
        let me = (t.me.x, t.me.y);
        let act = |point: Option<(i64, i64)>, subject: Option<usize>, score: i64, habit: &'static str| Action { point, subject, score, habit, ..Action::new(m, vec![]) };
        let mut out = vec![];
        let fighting = v.enemies.iter().any(|e| dist(e.x, e.y, me.0, me.1) <= 90000);
        match m {
            M_SWIPE => {
                if self.orbs_ready(tick) >= 2 {
                    let on = |p: (i64, i64)| {
                        let b = along(me.0, me.1, p.0 - me.0, p.1 - me.1, SWIPE_LEN);
                        v.enemies.iter().filter(|e| seg_dist(me.0, me.1, b.0, b.1, e.x, e.y) <= SWIPE_WIDTH).count() as i64
                    };
                    let (point, n) = if self.gate(GATE_GEOMETRY) {
                        let mut best = ((v.focus.x, v.focus.y), on((v.focus.x, v.focus.y)));
                        for e in &v.enemies {
                            let n = on((e.x, e.y));
                            if n > best.1 {
                                best = ((e.x, e.y), n);
                            }
                        }
                        best
                    } else if self.rank == 0 {
                        let p = self.random_near(me.0, me.1, SWIPE_LEN);
                        (p, on(p))
                    } else {
                        ((v.focus.x, v.focus.y), on((v.focus.x, v.focus.y)))
                    };
                    // the spine saves its swipe for lines: one target is worth less than loading an orb
                    let score = if self.gate(GATE_GEOMETRY) {
                        match n {
                            0 => -20,
                            1 if self.gate(GATE_SPINE) => 12,
                            _ => 14 * n + 4,
                        }
                    } else {
                        8 + 8 * n
                    };
                    out.push(act(Some(point), Some(v.focus.id), score, if n >= 2 { "swipe through the line" } else { "" }));
                }
            }
            M_SHATTER => {
                let held = self.pure_ready(tick) as i64;
                if held > 0 && self.open_count() > 0 {
                    let over = self.pure.iter().filter(|s| s.open && s.over && tick >= s.ready_at).count() as i64;
                    let load = (held - over) * SHATTER_LOAD + over * SHATTER_LOAD_OVER;
                    let hits = v.enemies.iter().filter(|e| self.rifts.iter().any(|r| r.live() && !r.hidden(tick) && r.reach(e.x, e.y) <= r.radius * 2)).count() as i64;
                    let breaks = self.stress + load > STRESS_CAP;
                    let score = if !self.gate(GATE_SAFETY) {
                        6 + 8 * hits + 3 * over
                    } else if breaks {
                        -60
                    } else if hits >= 2 {
                        12 * hits + 4 * held + if self.gate(GATE_SPINE) { 8 * over } else { 0 }
                    } else {
                        -10
                    };
                    let habit = if !breaks && over > 0 && hits >= 3 { "overload ridden into a Shatter" } else if !breaks && hits >= 2 { "shattered while the gauntlet held" } else { "" };
                    out.push(act(None, None, score, habit));
                }
            }
            M_OVERLOAD => {
                if !self.left_broken(tick) && self.sockets() < ORBS_MAX as usize {
                    let score = if !self.gate(GATE_SAFETY) {
                        14
                    } else if (self.sockets() >= 7 && t.danger(me.0, me.1, 60000) > 0) || self.stress + OVER_STRESS + 10 > self.table(&STRESS_LIMIT) {
                        -30
                    } else if v.enemies.iter().any(|e| dist(e.x, e.y, me.0, me.1) <= 150000) && !v.diver && self.sockets() < 8 {
                        // with headroom under the stress cap she loads up before a fight; the spine rides it harder
                        if self.gate(GATE_SPINE) { 18 } else { 16 }
                    } else {
                        4
                    };
                    let habit = if self.gate(GATE_SPINE) && score > 20 { "overloaded before the engage" } else if self.sockets() >= 7 && t.danger(me.0, me.1, 60000) > 0 { "stopped overloading near enemies" } else { "" };
                    out.push(act(None, None, score, habit));
                }
            }
            M_VENT => {
                if self.over_sockets() > 0 {
                    let score = if self.gate(GATE_SURVIVAL) && v.diver {
                        50
                    } else if self.gate(GATE_SAFETY) && self.stress >= STRESS_CAP * 3 / 4 {
                        30
                    } else {
                        2
                    };
                    out.push(act(None, None, score, if self.gate(GATE_SURVIVAL) && v.diver { "vented overload when dived" } else { "" }));
                }
            }
            M_AWAKEN => {
                if self.completion >= 100 && !self.awakened(tick) && !self.right_broken(tick) {
                    let engage = fighting && v.allies.iter().any(|a| t.danger(a.x, a.y, 60000) > 0);
                    let score = if self.gate(GATE_TEAM) {
                        if engage {
                            48
                        } else {
                            -20
                        }
                    } else if self.gate(GATE_SAFETY) {
                        if fighting {
                            26
                        } else {
                            4
                        }
                    } else {
                        30
                    };
                    out.push(act(None, None, score, if self.gate(GATE_TEAM) && engage { "Awakening timed with the engage" } else { "" }));
                }
            }
            M_OVERCLOCK => {
                if self.awakened(tick) && self.pure_ready(tick) > 0 {
                    let next = self.corruption + CORRUPT_PER_ORB;
                    let score = if !self.gate(GATE_SAFETY) {
                        20
                    } else if next >= 100 {
                        -60
                    } else if self.gate(GATE_SPINE) {
                        // three overclocks at most: corruption stays at or under 45
                        if next <= 45 && fighting {
                            18
                        } else {
                            -10
                        }
                    } else if next < 60 {
                        8
                    } else {
                        -20
                    };
                    out.push(act(None, None, score, ""));
                }
            }
            _ => {}
        }
        out
    }
    fn cost(a: &Action) -> usize {
        if a.tech >= 28 {
            return MOVE_ORBS[a.tech - 28];
        }
        a.rifts.iter().map(|s| RIFTS[s.kind].cost).sum::<usize>() + TECHS[a.tech].orbs
    }
    fn stress_of(&self, a: &Action, me: (i64, i64)) -> i64 {
        if a.tech >= 28 {
            return match a.tech {
                M_SWIPE => STRESS_TEAR,
                M_OVERLOAD => OVER_STRESS,
                _ => 0,
            };
        }
        let mut s = TECHS[a.tech].stress + STRESS_TEAR * a.rifts.len() as i64;
        let mut prev = me;
        for r in a.rifts.iter().filter(|r| r.chain) {
            if dist(prev.0, prev.1, r.x, r.y) > LINK_SCALE {
                s += STRESS_LONG;
            }
            prev = (r.x, r.y);
        }
        s
    }
    pub fn affordable(&self, a: &Action, me: (i64, i64), tick: usize) -> bool {
        let cost = Core::cost(a);
        let reserve = if a.tech >= 28 || TECHS[a.tech].group == "defense" { 0 } else { self.table(&RESERVE) as usize };
        if self.orbs_ready(tick) < cost + if cost > 0 { reserve } else { 0 } {
            return false;
        }
        if a.tech == T_INFUSE && self.hex_ready(tick) == 0 {
            return false;
        }
        if self.open_count() + a.rifts.len() + usize::from(a.tech == M_SWIPE) > MAX_RIFTS as usize {
            return false;
        }
        if a.uses.iter().any(|u| self.rift(*u).is_none()) || !self.dims_for(a, tick) {
            return false;
        }
        let buffer = if self.gate(GATE_SURVIVAL) { STRESS_BREAK } else { 0 };
        !self.gate(GATE_SAFETY) || a.tech == M_SHATTER || a.tech == M_VENT || self.stress + self.stress_of(a, me) + buffer <= self.table(&STRESS_LIMIT)
    }
    fn think(&mut self, out: &mut Vec<Cmd>, t: &Tick) {
        let tick = t.tick;
        let enemies: Vec<Fighter> = t.enemies().cloned().collect();
        if enemies.is_empty() || enemies.iter().all(|e| dist(e.x, e.y, t.me.x, t.me.y) > 320000) {
            return;
        }
        let v = self.view(t, enemies);
        let k = (self.table(&TECH_OPTIONS) as usize).min(28);
        let mut order: Vec<usize> = (0..28).collect();
        for i in 0..k {
            let j = i + self.random((28 - i) as u64) as usize;
            order.swap(i, j);
        }
        let mut pool: Vec<usize> = order[..k].to_vec();
        if tick >= self.plan_until {
            self.plan.clear();
        }
        // a Kidnap step on cooldown is skipped, not waited for
        if self.plan.front() == Some(&T_KIDNAP) && tick < self.kidnap_ready {
            self.plan.pop_front();
        }
        let next = self.plan.front().copied();
        if let Some(n) = next {
            if !pool.contains(&n) {
                pool.push(n);
            }
        }
        pool.sort();
        // the gauntlet moves are always on the table
        pool.extend(28..34);
        let noise = self.table(&NOISE_PCT);
        let me = (t.me.x, t.me.y);
        let mut best: Option<Action> = None;
        self.recent.retain(|(_, at)| tick < at + REPEAT_WINDOW as usize);
        for tech in pool {
            // variety: a technique she just used is less interesting (the gauntlet moves are her mechanics, not skills)
            let repeats = self.recent.iter().filter(|(k, _)| *k == tech).count() as i64;
            if tech < 28 && repeats >= REPEAT_MAX {
                continue;                     // at most REPEAT_MAX times in the window, combo steps included
            }
            let fatigue = if tech >= 28 {
                0
            } else {
                REPEAT_PENALTY * repeats + if self.recent.last().map(|r| r.0) == Some(tech) { REPEAT_LAST } else { 0 }
            };
            for mut a in self.propose(t, &v, tech) {
                // capped at half the play's worth: a clearly best play may repeat, close calls rotate
                a.score -= fatigue.min(a.score.max(0) / 2);
                if !self.affordable(&a, me, tick) {
                    continue;
                }
                if Some(tech) == next {
                    a.score += 30;
                }
                // deep planners like opening a whole known sequence from its first step
                let depth = self.table(&COMBO_DEPTH);
                if next.is_none() && depth >= 3 && a.score >= 20 && COMBOS.iter().any(|c| c[0] == tech) {
                    a.score += 4 * (depth - 1);
                }
                if noise > 0 {
                    a.score += self.random((2 * noise + 1) as u64) as i64 - noise;
                }
                if best.as_ref().is_none_or(|b| a.score > b.score) {
                    best = Some(a);
                }
            }
        }
        let Some(a) = best else { return };
        if a.score < 12 {
            return;
        }
        let tech = a.tech;
        if tech < 28 {
            self.recent.push((tech, tick));
        }
        self.execute(out, t, a);
        // sequencing: deeper athletes commit to the rest of a known combination
        let depth = self.table(&COMBO_DEPTH) as usize;
        if self.plan.front() == Some(&tech) {
            self.plan.pop_front();
        } else if depth > 1 && self.plan.is_empty() {
            for combo in COMBOS.iter() {
                if let Some(i) = combo.iter().position(|s| *s == tech) {
                    let rest: Vec<usize> = combo[i + 1..].iter().take(depth - 1).copied().collect();
                    if !rest.is_empty() {
                        self.plan = rest.into_iter().collect();
                        self.plan_until = tick + 360;
                        self.habit(match depth {
                            2 => "planned 2 steps ahead",
                            3 => "planned 3 steps ahead",
                            4 => "planned 4 steps ahead",
                            _ => "planned a 5-step sequence",
                        });
                        break;
                    }
                }
            }
        }
    }
    /// What a rift is for: the planner values its dimension accordingly.
    fn purpose_of(tech: usize, spec: &RiftSpec) -> Purpose {
        if spec.chain {
            Purpose::Route
        } else if (spec.entrance && matches!(tech, T_GATE | T_SKYDROP | T_ROTATION | T_RESCUE | T_STEP | T_PHASE)) || matches!(tech, T_POCKET | T_LIFELINE) {
            Purpose::Ally
        } else {
            Purpose::Enemy
        }
    }
    /// Carries out an action: spends orbs and stress, opens rifts, or acts on existing ones.
    /// The Rift Lab and the hand-placed fixtures call this directly.
    pub fn execute(&mut self, out: &mut Vec<Cmd>, t: &Tick, a: Action) {
        let tick = t.tick;
        let cost = Core::cost(&a);
        if self.orbs_ready(tick) < cost {
            return;
        }
        if a.tech >= 28 {
            let done = match a.tech {
                M_SWIPE => self.swipe(out, t, a.point.unwrap_or((t.me.x + 1000, t.me.y)), a.dim, a.subject),
                M_SHATTER => {
                    if self.pure_ready(tick) == 0 {
                        false
                    } else {
                        self.shatter(out, t, false);
                        true
                    }
                }
                M_OVERLOAD => self.overload(out, t),
                M_VENT => {
                    let any = self.over_sockets() > 0;
                    self.vent();
                    any
                }
                M_AWAKEN => self.awaken(out, t),
                _ => self.overclock(out, t),
            };
            if done {
                self.stats.move_uses[a.tech - 28] += 1;
                self.habit(a.habit);
                out.push(Cmd::Log(format!("tick {tick} rank {} {} score {} stress {} orbs {}/{}", RANKS[self.rank], MOVE_IDS[a.tech - 28], a.score, self.stress, self.pure_ready(tick), self.hex_ready(tick))));
            }
            return;
        }
        if (a.tech == T_INFUSE && self.hex_ready(tick) == 0) || self.open_count() + a.rifts.len() > MAX_RIFTS as usize || !self.dims_for(&a, tick) {
            return;
        }
        let stress = self.stress_of(&a, (t.me.x, t.me.y));
        self.stats.tech_uses[a.tech] += 1;
        if a.tech == T_KIDNAP {
            self.kidnap_ready = tick + KIDNAP_CD as usize;
        }
        self.habit(a.habit);
        out.push(Cmd::Log(format!("tick {tick} rank {} {} score {} rifts {} stress {}", RANKS[self.rank], TECHS[a.tech].name, a.score, a.rifts.len(), self.stress + stress)));
        // every technique has its own cast flourish on Vesper
        let flourish = self.tier4();
        self.fx(out, format!("tech_{}_t{flourish}", TECHS[a.tech].id), t.me.x, t.me.y, Some(t.me.id), 24);
        let hexy = self.rank >= GATE_SAFETY && matches!(TECHS[a.tech].group, "defense" | "team" | "engage");
        let mut uids = vec![];
        let mut cast = vec![];
        for spec in &a.rifts {
            let n = RIFTS[spec.kind].cost;
            let pref = if self.hand_placed {
                false
            } else if self.rank < GATE_SAFETY {
                self.random(2) == 0
            } else {
                hexy
            };
            let Some(orb) = self.take(tick, pref) else { break };
            for _ in 1..n {
                self.take(tick, orb.hex.is_some());
            }
            let subject = a.subject.and_then(|s| t.get(s)).cloned();
            let dim = spec.dim.unwrap_or_else(|| self.pick_dim(t, Core::purpose_of(a.tech, spec), subject.as_ref().filter(|s| s.team != t.me.team), (spec.x, spec.y)));
            let (_, twist) = self.activate(orb, dim, tick);
            cast.push(dim);
            if let Some(idx) = twist {
                if let Some(e) = t.enemies().filter(|e| dist(e.x, e.y, spec.x, spec.y) <= NEAR_RADIUS).min_by_key(|e| (dist2(e.x, e.y, spec.x, spec.y), e.id)).cloned() {
                    self.twist(out, t, idx, &e, 20 + t.me.attack / 2);
                }
            }
            let module = orb.hex.map(|i| self.modules[i]);
            uids.push(self.open(out, t, spec, a.tech, a.subject, dim, module));
        }
        self.commit_cast(&cast, tick);
        self.add_stress(out, t, stress);
        if uids.len() >= 2 && a.tech != T_CHAIN && a.tech != T_BANK {
            let (first, last) = (uids[0], uids[uids.len() - 1]);
            if let Some(r) = self.rift_mut(first) {
                r.peer = Some(last);
            }
            if let Some(r) = self.rift_mut(last) {
                r.peer = Some(first);
            }
        }
        if a.tech == T_HOOK && uids.len() == 2 {
            if let Some(r) = self.rift_mut(uids[0]) {
                r.expires = r.opens + 90;
            }
        }
        if matches!(a.tech, T_SEEDTRAP | T_SINKTRAP) {
            self.stats.traps += 1;
        }
        if a.tech == T_LIFELINE {
            if let Some(r) = uids.first().and_then(|u| self.rift_mut(*u)) {
                r.expires = r.opens + 1200;
            }
        }
        let tier = self.tier4();
        match a.tech {
            T_IMPLODE => {
                if let Some(&u) = a.uses.first() {
                    self.implode(out, t, u);
                }
            }
            T_DOMINO => {
                for (i, &u) in a.uses.iter().enumerate() {
                    if i == 0 {
                        self.implode(out, t, u);
                    } else {
                        self.later.push(Later::Domino { at: tick + 15 * i, uid: u });
                    }
                }
            }
            T_CROSSFIRE => {
                self.crossfire_until = tick + 180;
            }
            T_OPUNCH => {
                let spend = self.stress.min(STRESS_MAX_OVERLOAD);
                self.opunch = spend;
                self.stress -= spend;
            }
            T_INFUSE => {
                if let Some(&u) = a.uses.first() {
                    if let Some(orb) = self.take(tick, true) {
                        let module = orb.hex.map_or(self.modules[0], |i| self.modules[i]);
                        if let Some(r) = self.rift_mut(u) {
                            r.infused = Some(module);
                            let (x, y) = (r.x, r.y);
                            self.fx(out, format!("tech_infuse_t{tier}"), x, y, None, 30);
                        }
                    }
                }
            }
            T_STITCH => {
                if let (Some(&ua), Some(&ub)) = (a.uses.first(), a.uses.get(1)) {
                    let b = self.rift(ub).cloned();
                    if let (Some(b), Some(ra)) = (b, self.rift_mut(ua)) {
                        ra.x = (ra.x + b.x) / 2;
                        ra.y = (ra.y + b.y) / 2;
                        ra.radius = ra.radius * 14 / 10;
                        ra.dim2 = Some(b.dim);
                        ra.expires = ra.expires.max(b.expires);
                        let (x, y) = (ra.x, ra.y);
                        self.close(out, ub, "close");
                        self.fx(out, format!("tech_stitch_t{tier}"), x, y, None, 30);
                    }
                }
            }
            _ => {}
        }
    }
    /// Hand placement (Rift Lab and parity fixtures): the planner never runs.
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn place(&mut self, input: &Input, a: Action) -> Vec<Cmd> {
        let mut out = vec![];
        let Some(me) = input.fighters.iter().find(|f| f.id == input.me).cloned() else { return out };
        let t = Tick { tick: input.tick, me, fighters: input.fighters, wall: input.wall, attack_interval: input.attack_interval.max(1), stunned: input.stunned };
        self.fx_left = FX_PER_UPDATE as usize;
        self.execute(&mut out, &t, a);
        out
    }
    /// Vesper died: everything she owns closes, overload sockets drop, the Hexcore loses research.
    pub fn on_dead(&mut self, tick: usize) {
        if tick <= self.risky_until {
            self.stats.self_deaths += 1;
        }
        for r in self.rifts.iter_mut() {
            r.done = true;
        }
        self.rifts.clear();
        self.packets.clear();
        self.later.clear();
        self.dots.clear();
        self.marks.clear();
        self.chill.clear();
        self.exposed.clear();
        self.vent();
        self.aiming = false;
        self.punching = false;
        self.phased = false;
        self.chain_target = None;
        self.plan.clear();
        self.opunch = 0;
        self.awaken_until = self.awaken_until.min(tick);
        self.corruption = 0;
        self.completion = (self.completion - RESEARCH_DEATH).max(HEX_START);
        self.combo = 0;
        self.last_family = None;
        self.damage.clear();
        self.controls.clear();
        self.moves.clear();
        self.dim_ready.iter_mut().for_each(|t| *t = 0);
        self.family_ready.iter_mut().for_each(|t| *t = 0);
    }
}
