//! Authoritative Steve/Herobrine rules. No host handles, wall clock, or gameplay RNG.
use crate::steve_craft_data::*;
use crate::steve_craft_geometry::*;
#[path="steve_creative.rs"]
mod creative;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Form {
    Steve,
    Herobrine,
}
#[derive(Clone, Debug)]
pub struct Unit {
    pub id: usize,
    pub team: usize,
    pub x: i64,
    pub y: i64,
    pub hp: i64,
    pub max_hp: i64,
    pub ad: i64,
    pub shield: i64,
    pub visible: bool,
    pub targetable: bool,
    pub champion: bool,
    pub farmable: bool,
    pub immune: bool,
    pub cc_immune: bool,
    pub tower: bool,
    pub gadget: bool,
    pub immovable: bool,
    pub epic: bool,
    pub totem_locked: bool,
    pub gifted_armour_until: usize,
    pub vx: i64,
    pub vy: i64,
    pub swing: bool,
}
impl Unit {
    pub fn pos(&self) -> Point {
        (self.x, self.y)
    }
    pub fn pct(&self) -> i64 {
        100 * self.hp / self.max_hp.max(1)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    Recipe {
        recipe: u8,
        target: Option<usize>,
        point: Point,
    },
    Prepare(u8),
    Deploy(Point),
    Detonate(u32),
    Discard,
    Stance,
    Wall {
        centre: Point,
        heading: u8,
        bend: i8,
    },
    Dismount(Point),
    Repair(u32),
    Link {from:u32,to:u32},
    Configure {node:u32,mode:u8,delay:usize},
    Arm(u32),
    Activate(u32),
    Load {node:u32,recipe:u8,target:Option<usize>,point:Point},
    Rotate {object:u32,heading:u8},
}
impl Action {
    pub fn recipe(&self) -> Option<u8> {
        match self {
            Self::Recipe { recipe, .. } => Some(*recipe),
            Self::Prepare(r) => Some(*r),
            _ => None,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Cmd {
    Damage {
        event: u64,
        target: usize,
        ad: i64,
        ap: i64,
        utility: bool,
    },
    Heal(usize, i64),
    Shield(usize, i64, usize),
    Move(usize, Point),
    Push(usize, Point, usize),
    Slow(usize, i64, usize),
    Speed(usize, i64, usize),
    Spawn {
        object: u32,
        recipe: u8,
        point: Point,
        hp: i64,
    },
    Remove(u32),
    Warning {
        point: Point,
        radius: i64,
        until: usize,
        owner_hazard: bool,
    },
    PathWarning {
        path: Vec<Point>,
        until: usize,
    },
    Lock {
        ticks: usize,
        pose: u8,
    },
    Immune(usize),
    Fx {
        kind: u8,
        point: Point,
    },
    Status {target:usize,kind:u8,power:i64,ticks:usize},
    ClearStatus {target:usize,kind:u8},
    PotionTick {event:u64,target:usize,power:i64,kind:u8},
    HealObject(u32,i64),
    RelocateObject(u32,Point),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Charge {
    Ready(usize),
    Reserved,
}
#[derive(Clone, Debug)]
pub struct Pending {
    pub action: Action,
    pub end: usize,
    pub begin: usize,
    pub bonus: bool,
    pub charges: Vec<usize>,
    pub previous_cd: Option<(usize, usize)>,
    pub path: Option<Vec<Point>>,
    pub power_hp:i64,
}
#[derive(Clone, Debug)]
pub struct Gadget {
    pub recipe: u8,
    pub charges: Vec<usize>,
}
#[derive(Clone, Debug)]
pub struct Object {
    pub id: u32,
    pub recipe: u8,
    pub point: Point,
    pub direction: Point,
    pub born: usize,
    pub end: usize,
    pub active_at: usize,
    pub blast: Option<usize>,
    pub hp: i64,
    pub confirmed: bool,
    pub paid: Vec<usize>,
    pub old_cd: usize,
    pub tick_at: usize,
    pub hit: Vec<usize>,
    pub power_hp:i64,
    pub max_hp:i64,
}
#[derive(Clone, Debug)]
pub struct Projectile {
    pub recipe: u8,
    pub from: Point,
    pub point: Point,
    pub dest: Point,
    pub begin: usize,
    pub end: usize,
    pub target: Option<usize>,
    pub hit: Vec<usize>,
    pub paid: Vec<usize>,
    pub old_cd: usize,
    pub bonus: bool,
    pub travelled: i64,
    pub power_hp:i64,
}
#[derive(Clone, Debug)]
pub struct Reel {
    pub target: usize,
    pub end: usize,
    pub ally: bool,
}
#[derive(Clone, Debug)]
pub struct Ride {
    pub path: Vec<Point>,
    pub start: usize,
    pub end: usize,
    pub wall_first: usize,
    pub rider: bool,
    pub hit: Vec<usize>,
    pub jump: Option<(Point, Point, usize)>,
    pub power_hp:i64,
}
#[derive(Clone, Debug)]
pub struct WallState {
    pub path: Vec<Point>,
    pub begins: Vec<usize>,
    pub finishes: Vec<usize>,
    pub end: usize,
    pub ultimate: bool,
    pub object: Option<u32>,
}
#[derive(Clone, Debug)]
pub struct Receipt {
    pub id: u64,
    pub target: usize,
    pub champion: bool,
    pub utility: bool,
    pub at: usize,
    pub power_hp:i64,
}
#[derive(Clone, Debug)]
pub struct Metrics {
    pub casts: [usize; 45],
    pub switches: usize,
    pub walls: usize,
    pub combos: usize,
    pub requested_damage: i64,
    pub actual_damage: i64,
    pub self_damage: i64,
    pub interrupted: usize,
    pub wasted: usize,
    pub branches: usize,
}
impl Default for Metrics {fn default()->Self{Self{casts:[0;45],switches:0,walls:0,combos:0,requested_damage:0,actual_damage:0,self_damage:0,interrupted:0,wasted:0,branches:0}}}
#[derive(Clone, Debug)]
pub struct Core {
    pub form: Form,
    pub charges: [Charge; 4],
    pub cooldowns: [usize; 45],
    pub creative:creative::State,
    pub repair_at:usize,
    pub axe_hp:i64,
    pub stance_at: usize,
    pub ult_at: usize,
    pub pending: Option<Pending>,
    pub prepared: Option<Gadget>,
    pub objects: Vec<Object>,
    pub shots: Vec<Projectile>,
    pub reels: Vec<Reel>,
    pub grapple: Option<(Point, usize)>,
    pub trails: Vec<(Point, Point, usize)>,
    pub walls: Vec<WallState>,
    pub ride: Option<Ride>,
    pub followup: Option<(usize, usize)>,
    pub followup_at: usize,
    pub followup_hp:i64,
    pub axe_until: usize,
    pub last_basic: u64,
    pub receipts: Vec<Receipt>,
    pub serial: u64,
    pub next_object: u32,
    pub last_tick: Option<usize>,
    pub reason: &'static str,
    pub selected: Option<Action>,
    pub preview: Vec<Continuation>,
    pub metrics: Metrics,
    pub last_ai: usize,
    pub last_owner: usize,
}
impl Default for Core {
    fn default() -> Self {
        Self {
            form: Form::Steve,
            charges: [Charge::Ready(0), Charge::Ready(0), Charge::Ready(0),Charge::Ready(0)],
            cooldowns: [0; 45],
            creative:Default::default(),
            repair_at:0,
            axe_hp:0,
            stance_at: 0,
            ult_at: 0,
            pending: None,
            prepared: None,
            objects: vec![],
            shots: vec![],
            reels: vec![],
            grapple: None,
            trails: vec![],
            walls: vec![],
            ride: None,
            followup: None,
            followup_at: 0,
            followup_hp:0,
            axe_until: 0,
            last_basic: 0,
            receipts: vec![],
            serial: 0,
            next_object: 1,
            last_tick: None,
            reason: "Ready",
            selected: None,
            preview: vec![],
            metrics: Metrics::default(),
            last_ai: 0,
            last_owner: 0,
        }
    }
}
#[derive(Clone)]
pub struct Input<'a> {
    pub tick: usize,
    pub me: &'a Unit,
    pub units: &'a [Unit],
    pub unlocked: [bool; 3],
    pub controlled: bool,
    pub bound: bool,
    pub automatic: bool,
    pub action: Option<Action>,
    pub basics: &'a [(u64, usize)],
    pub objects: &'a [(u32, i64)],
    pub wall: &'a dyn Fn(i64, i64) -> bool,
    pub barriers: &'a [(Point,Point)],
    pub home: Point,
    pub haste: i64,
    pub ult_haste: i64,
}
/// The cooldown a recipe starts, checks and a failed allocation restores: the recipe's own. Every
/// beacon, splash, armour piece and arrow recharges on its own timer; the recipe `group` is only
/// the family a recipe belongs to (round 139 briefly shared one timer per family, a large nerf).
pub fn cooldown_slot(recipe: u8) -> usize {
    recipe as usize
}
fn cd(ticks: usize, haste: i64) -> usize {
    (ticks as i64 * 100 / (100 + haste).max(10)).max(1) as usize
}
fn boosted(n: i64, bonus: bool) -> i64 {
    n * if bonus { STEVE_SUPPORT_PCT } else { 100 } / 100
}
#[derive(Clone, Debug)]
pub struct Continuation {
    pub actions: Vec<Action>,
    pub score: i64,
}
impl Core {
    /// Deterministic bounded beam search. Every recipe is generated at the root;
    /// utility scores only compare actions after urgent survival/protection costs.
    fn candidates(&self, i: &Input) -> Vec<(Action, i64)> {
        let mut a = vec![];
        let foe = i
            .units
            .iter()
            .filter(|u| {
                u.team != i.me.team
                    && u.hp > 0
                    && u.visible
                    && u.targetable
                    && !u.immune
                    && (u.champion || u.farmable)
            })
            .min_by_key(|u| (!u.champion, square(i.me.pos(), u.pos()), u.id));
        let ally = i
            .units
            .iter()
            .filter(|u| {
                u.team == i.me.team && u.hp > 0 && u.champion && within(i.me.pos(), u.pos(), 60000)
            })
            .min_by_key(|u| (u.pct(), u.id))
            .unwrap_or(i.me);
        let danger = i
            .units
            .iter()
            .filter(|u| {
                u.team != i.me.team
                    && u.champion
                    && u.hp > 0
                    && u.visible
                    && within(i.me.pos(), u.pos(), 90000)
            })
            .count() as i64;
        if i.unlocked[2] && i.tick >= self.stance_at {
            let want = if i.me.pct() < 45 || ally.pct() < 35 {
                Form::Steve
            } else if foe.is_some_and(|u| u.champion) && danger <= 2 {
                Form::Herobrine
            } else {
                Form::Steve
            };
            if self.form != want {
                a.push((Action::Stance, if want == Form::Steve { 230 } else { 110 }));
            }
        }
        for r in 0..15u8 {
            if self.form==Form::Herobrine&&matches!(r,10|11){continue;}
            if r == 6 || r == 7 {
                continue;
            }
            let target = if r == 10 {
                Some(ally.id)
            } else if r == 8 {
                if self.form==Form::Steve&&ally.id != i.me.id && ally.pct() < 35 {
                    Some(ally.id)
                } else {
                    foe.map(|u| u.id)
                }
            } else {
                None
            };
            let p = if r == 9 || r == 14 {
                toward(i.me.pos(), i.home, if r == 9 { 70000 } else { 80000 })
            } else if r == 10 {
                ally.pos()
            } else if r == 11 || r == 13 {
                i.me.pos()
            } else if let Some(f) = foe {
                let lead = if r == 3 { 36 } else { 21 };
                let q = (
                    f.x + f.vx * lead,
                    f.y + f.vy * lead + if r == 0 || r == 1 { 20000 } else { 0 },
                );
                toward(i.me.pos(), q, RECIPES[r as usize].range)
            } else {
                i.me.pos()
            };
            let champion = foe.is_some_and(|u| u.champion);
            let close = foe.is_some_and(|u| within(i.me.pos(), u.pos(), 32000));
            let score = match r {
                0 => {
                    if danger > 0 && ally.pct() < 50 {
                        160
                    } else {
                        -30
                    }
                }
                1 => {
                    if danger > 0 && i.me.pct() < 40 {
                        190
                    } else if close {
                        55
                    } else {
                        -50
                    }
                }
                2 => {
                    if champion {
                        100
                    } else if foe.is_some() {
                        25
                    } else {
                        -100
                    }
                }
                3 => {
                    if foe.is_some() {
                        if champion {
                            145
                        } else {
                            65
                        }
                    } else {
                        -100
                    }
                }
                4 => {
                    if self
                        .objects
                        .iter()
                        .any(|o| o.recipe == 5 && within(o.point, ally.pos(), 40000))
                    {
                        90
                    } else {
                        -80
                    }
                }
                5 => {
                    if foe.is_some() {
                        if champion {
                            100
                        } else {
                            60
                        }
                    } else {
                        -100
                    }
                }
                8 => {
                    if ally.id != i.me.id && ally.pct() < 35 {
                        300
                    } else if champion && !close {
                        180
                    } else if foe.is_some() {
                        70
                    } else {
                        -100
                    }
                }
                9 | 14 => {
                    if danger > 0 && i.me.pct() < 35 {
                        350
                    } else {
                        -100
                    }
                }
                10 => {
                    if ally.pct() < 60 {
                        320 - ally.pct() * 2
                    } else {
                        -100
                    }
                }
                11 => {
                    if danger > 0 && i.me.pct() < 55 {
                        300
                    } else {
                        -100
                    }
                }
                12 => {
                    if foe.is_some() {
                        if champion {
                            120
                        } else {
                            55
                        }
                    } else {
                        -100
                    }
                }
                13 => {
                    if close && self.axe_until < i.tick {
                        110
                    } else {
                        -100
                    }
                }
                _ => -100,
            };
            // Keep a defence charge when hurt; don't burn resources on idle construction.
            if score > 0
                && !(self.available(i.tick) <= 1 && i.me.pct() < 40 && !matches!(r, 8..=11 | 14))
            {
                a.push((
                    Action::Recipe {
                        recipe: r,
                        target,
                        point: p,
                    },
                    score,
                ));
            }
        }
        if let Some(f) = foe.filter(|u| u.champion) {
            if let Some(g) = &self.prepared {
                let q = toward(i.me.pos(), (f.x, f.y + 18000), 70000);
                let raw = if g.recipe == 6 {
                    CRYSTAL_BASE + i.me.max_hp * CRYSTAL_HP_PCT / 100
                } else {
                    ANCHOR_BASE + i.me.max_hp * ANCHOR_HP_PCT / 100
                };
                let risk = if within(i.me.pos(), q, RECIPES[g.recipe as usize].radius) {
                    raw / 2
                } else {
                    0
                };
                if i.me.hp + i.me.shield > risk + 80 {
                    a.push((Action::Deploy(q), 170 - risk));
                }
            } else if self.available(i.tick) >= 2 && i.me.pct() > 65 && danger <= 2 {
                for r in [6, 7] {
                    let grouped=i.units.iter().filter(|u|u.team!=i.me.team&&u.hp>0&&u.visible&&u.champion&&within(u.pos(),f.pos(),40000)).count();
                    a.push((Action::Prepare(r), if self.form==Form::Herobrine&&grouped>=2{230}else{80}));
                }
            }
            for o in self
                .objects
                .iter()
                .filter(|o| matches!(o.recipe, 3 | 6 | 7) && o.confirmed && o.blast.is_none())
            {
                if within(f.pos(), o.point, RECIPES[o.recipe as usize].radius)
                    && !self.occluded(i, o.point, f.pos())
                    && (!within(i.me.pos(), o.point, RECIPES[o.recipe as usize].radius)
                        || o.recipe == 3
                        || i.me.pct() > 65)
                {
                    a.push((Action::Detonate(o.id), 210));
                }
            }
            if i.unlocked[2] && i.tick >= self.ult_at && i.me.pct() > 45 && danger > 0 {
                let heading = (0..16)
                    .min_by_key(|h| {
                        let d = arc((0, 0), *h, 0);
                        let v = (d[1].0 - d[0].0, d[1].1 - d[0].1);
                        ((f.x - i.me.x) * v.0 + (f.y - i.me.y) * v.1).abs()
                    })
                    .unwrap_or(0);
                for bend in if self.form == Form::Herobrine {
                    vec![0, -2, 2, -4, 4]
                } else {
                    vec![0]
                } {
                    a.push((
                        Action::Wall {
                            centre: f.pos(),
                            heading,
                            bend,
                        },
                        if danger >= 2 { 185 } else { 75 },
                    ));
                }
            }
        }
        self.creative_candidates(i,&mut a);
        a
    }
    pub fn plan_lines(&mut self, i: &Input) -> Vec<Continuation> {
        #[derive(Clone)]
        struct Branch {
            core: Core,
            time: usize,
            line: Vec<Action>,
            score: i64,
        }
        let mut beam = vec![Branch {
            core: self.scratch(),
            time: i.tick,
            line: vec![],
            score: 0,
        }];
        let mut evaluated = 0;
        let mut completed = false;
        for depth in 0..3 {
            let mut next = vec![];
            for b in &beam {
                let frame = Input {
                    tick: b.time,
                    me: i.me,
                    units: i.units,
                    unlocked: i.unlocked,
                    controlled: i.controlled,
                    bound: i.bound,
                    automatic: false,
                    action: None,
                    basics: &[],
                    objects: i.objects,
                    wall: i.wall,
                    barriers: i.barriers,
                    home: i.home,
                    haste: i.haste,
                    ult_haste: i.ult_haste,
                };
                for (action, value) in b.core.candidates(&frame) {
                    if evaluated >= 128 {
                        break;
                    }
                    evaluated += 1;
                    let mut c = b.core.scratch();
                    let mut commands = vec![];
                    if !c.accept(&frame, action.clone(), &mut commands) {
                        continue;
                    }
                    let end = c.pending.as_ref().map_or(b.time + 1, |p| p.end);
                    if let Some(p) = c.pending.take() {
                        let release = Input {
                            tick: end,
                            ..frame.clone()
                        };
                        c.release(&release, p, &mut commands);
                    }
                    for o in &mut c.objects {
                        o.confirmed = true;
                    }
                    let mut line = b.line.clone();
                    line.push(action);
                    let score = b.score + value * 100 / (100 + depth * 40);
                    next.push(Branch {
                        core: c,
                        time: end + 1,
                        line,
                        score,
                    });
                }
            }
            next.sort_by_key(|b| std::cmp::Reverse(b.score));
            next.truncate(8);
            if next.is_empty() {
                break;
            }
            beam = next;
            completed = true;
        }
        self.metrics.branches += evaluated;
        if !completed {
            beam.clear();
        }
        beam
            .into_iter()
            .map(|b| Continuation {
                actions: b.line,
                score: b.score,
            })
            .collect()
    }
    /// What a planning branch needs: the state `candidates`, `accept` and `release` read.
    /// The preview, receipts, trails and metrics are only written there, so a branch starts
    /// them empty instead of copying them for every candidate.
    fn scratch(&self) -> Core {
        Core {
            form: self.form,
            charges: self.charges.clone(),
            cooldowns: self.cooldowns,
            creative: self.creative.clone(),
            repair_at: self.repair_at,
            axe_hp: self.axe_hp,
            stance_at: self.stance_at,
            ult_at: self.ult_at,
            pending: self.pending.clone(),
            prepared: self.prepared.clone(),
            objects: self.objects.clone(),
            shots: self.shots.clone(),
            reels: self.reels.clone(),
            grapple: self.grapple,
            trails: vec![],
            walls: self.walls.clone(),
            ride: self.ride.clone(),
            followup: self.followup,
            followup_at: self.followup_at,
            followup_hp: self.followup_hp,
            axe_until: self.axe_until,
            last_basic: self.last_basic,
            receipts: vec![],
            serial: self.serial,
            next_object: self.next_object,
            last_tick: self.last_tick,
            reason: self.reason,
            selected: self.selected.clone(),
            preview: vec![],
            metrics: Metrics::default(),
            last_ai: self.last_ai,
            last_owner: self.last_owner,
        }
    }
    fn plan(&mut self, i: &Input) -> Option<Action> {
        self.preview = self.plan_lines(i);
        self.preview
            .first()
            .and_then(|b| b.actions.first())
            .cloned()
    }
    pub fn available(&self, t: usize) -> usize {
        self.charges
            .iter().take(if self.form==Form::Steve{3}else{4})
            .filter(|c| matches!(c,Charge::Ready(at)if *at<=t))
            .count()
    }
    fn spend(&mut self, t: usize, n: usize, reserve: bool) -> Option<Vec<usize>> {
        let ids: Vec<_> = self
            .charges
            .iter().take(if self.form==Form::Steve{3}else{4})
            .enumerate()
            .filter(|(_, c)| matches!(c,Charge::Ready(at)if *at<=t))
            .take(n)
            .map(|(i, _)| i)
            .collect();
        if ids.len() != n {
            return None;
        }
        for &i in &ids {
            self.charges[i] = if reserve {
                Charge::Reserved
            } else {
                Charge::Ready(t + CHARGE_TICKS as usize)
            }
        }
        Some(ids)
    }
    fn recover(&mut self, ids: &[usize], t: usize) {
        for &i in ids {
            self.charges[i] = Charge::Ready(t + CHARGE_TICKS as usize)
        }
    }
    fn reject(&mut self, why: &'static str) -> bool {
        self.reason = why;
        false
    }
    fn enemy<'a>(i: &Input<'a>, id: usize) -> Option<&'a Unit> {
        i.units.iter().find(|u| {
            u.id == id && u.hp > 0 && u.team != i.me.team && u.visible && u.targetable && !u.immune
        })
    }
    fn aimed(&self, i: &Input, p: Point, r: i64) -> bool {
        legal(p) && within(i.me.pos(), p, r) && !(i.wall)(p.0, p.1) && !self.segments(i.tick).iter().chain(i.barriers.iter()).any(|&(a,b)|distance_segment(p,a,b)<8000)
    }
    fn occupied(i: &Input, p: Point) -> bool {
        i.units
            .iter()
            .any(|u| u.hp > 0 && within(u.pos(), p, 10000))
    }
    fn friendly(&self) -> bool {
        self.form == Form::Steve
    }
    pub fn died(&mut self, t: usize) -> Vec<Cmd> {
        let out = self.objects.iter().map(|o| Cmd::Remove(o.id)).collect();
        if let Some(p) = self.prepared.take() {
            self.recover(&p.charges, t)
        }
        if let Some(p) = self.pending.take() {
            if matches!(p.action, Action::Prepare(_)) {
                self.recover(&p.charges, t)
            }
        }
        self.form = Form::Steve;
        self.objects.clear();
        self.walls.clear();
        self.shots.clear();
        self.reels.clear();
        self.grapple = None;
        self.trails.clear();
        self.ride = None;
        self.followup = None;
        self.receipts.clear();
        self.preview.clear();
        self.selected = None;
        self.axe_until = 0;
        self.creative.clear();
        self.reason = "Death cleanup";
        out
    }
    /// A failed host allocation rolls back that object's accepted recipe. It never becomes an invisible effect.
    pub fn allocated(&mut self, id: u32, success: bool) {
        if let Some(n) = self.objects.iter().position(|o| o.id == id) {
            if success {
                self.objects[n].confirmed = true
            } else {
                let o = self.objects.remove(n);
                for k in o.paid {
                    self.charges[k] = Charge::Ready(0)
                }
                self.cooldowns[cooldown_slot(o.recipe)] = o.old_cd;
                self.walls.retain(|w| w.object != Some(id));
                self.metrics.wasted += 1;
                self.reason = "Native object capacity unavailable; charges and cooldown restored";
            }
        }
    }
    pub fn receipt(&mut self, event: u64, actual: i64, t: usize) {
        let Some(n) = self.receipts.iter().position(|r| r.id == event) else {
            return;
        };
        let r = self.receipts.remove(n);
        if actual <= 0 {
            return;
        }
        if r.target == self.last_owner {
            self.metrics.self_damage += actual;
            return;
        }
        self.metrics.actual_damage += actual;
        if self.form == Form::Herobrine
            && r.champion
            && r.utility
            && self.followup.is_none()
            && t >= self.followup_at
        {
            self.followup = Some((r.target, t + FOLLOWUP_TICKS as usize));
            self.followup_hp=r.power_hp;
        }
    }
    fn damage(&mut self, i: &Input, u: &Unit, ad: i64, ap: i64, utility: bool, out: &mut Vec<Cmd>) {
        self.damage_with_hp(i,u,ad,ap,utility,i.me.max_hp,out)
    }
    fn damage_with_hp(&mut self, i: &Input, u: &Unit, ad: i64, ap: i64, utility: bool, power_hp:i64, out: &mut Vec<Cmd>) {
        if u.hp <= 0 || !u.targetable || u.immune {
            return;
        }
        self.serial += 1;
        let id = self.serial;
        self.receipts.push(Receipt {
            id,
            target: u.id,
            champion: u.champion && u.visible,
            utility,
            at: i.tick,
            power_hp,
        });
        self.metrics.requested_damage += ad + ap;
        out.push(Cmd::Damage {
            event: id,
            target: u.id,
            ad,
            ap,
            utility,
        });
    }
    pub fn segments(&self, t: usize) -> Vec<(Point, Point)> {
        let mut s = vec![];
        for w in &self.walls {
            if t >= w.end {
                continue;
            }
            if let Some(id) = w.object {
                if !self.objects.iter().any(|o| o.id == id && o.confirmed) {
                    continue;
                }
            }
            for (n, p) in w.path.windows(2).enumerate() {
                if t < w.begins[n] {
                    continue;
                }
                let end = if t >= w.finishes[n] {
                    p[1]
                } else {
                    let l = length(p[0], p[1]);
                    toward(
                        p[0],
                        p[1],
                        l * (t - w.begins[n]) as i64 / (w.finishes[n] - w.begins[n]).max(1) as i64,
                    )
                };
                s.push((p[0], end));
            }
        }
        s
    }
    fn occluded(&self, i: &Input, a: Point, b: Point) -> bool {
        !clear(a, b, i.wall) || i.barriers.iter().any(|&(c,d)|crosses(a,b,c,d))
            || self
                .segments(i.tick)
                .iter()
                .any(|&(c, d)| crosses(a, b, c, d))
    }
    fn capacity(&self, r: u8) -> bool {
        if self.objects.len()+self.shots.iter().filter(|s|s.recipe==3).count()>=CONSTRUCTION_CAP as usize{return false;}
        if (15..=19).contains(&r){return !self.objects.iter().any(|o|(15..=19).contains(&o.recipe));}
        if (26..=27).contains(&r){return self.objects.iter().filter(|o|(26..=27).contains(&o.recipe)).count()<2;}
        if r == 6 || r == 7 {
            !self.objects.iter().any(|o| o.recipe == 6 || o.recipe == 7)
        } else {
            (self.objects.iter().filter(|o| o.recipe == r).count()
                + self
                    .shots
                    .iter()
                    .filter(|s| s.recipe == r && r == 3)
                    .count())
                < RECIPES[r as usize].cap
        }
    }
    pub fn wall_path(&self, i: &Input, c: Point, h: u8, b: i8) -> Result<Vec<Point>, &'static str> {
        if b.abs() > 4 || h > 15 {
            return Err("Invalid wall heading or bend");
        }
        if self.form == Form::Steve && b != 0 {
            return Err("Curves require Herobrine");
        }
        if !within(i.me.pos(), c, 340000) {
            return Err("Wall beyond deployment range");
        }
        let raw = arc(c, h, b);
        let mut p = vec![];
        for q in raw {
            if !legal(q) || (i.wall)(q.0, q.1) {
                break;
            }
            if let Some(&a) = p.last() {
                let end = clip(a, q, i.wall);
                if end != q {
                    if length(a, end) > 1000 {
                        p.push(end)
                    }
                    break;
                }
            }
            p.push(q);
        }
        if path_length(&p) < 60000 {
            return Err("Terrain leaves less than 60000 wall");
        }
        if b != 0 && length(p[0], *p.last().unwrap()) < 48000 {
            return Err("Curve opening is too narrow");
        }
        let seg: Vec<_> = p.windows(2).map(|w| (w[0], w[1])).collect();
        for u in i.units.iter().filter(|u| u.team == i.me.team && u.hp > 0) {
            if seg
                .iter()
                .any(|&(a, b)| distance_segment(u.pos(), a, b) < 12000)
            {
                return Err("Wall would rise under an ally");
            }
            if within(u.pos(), c, 220000) {
                let minx = p.iter().map(|p| p.0).min().unwrap() - 24000;
                let maxx = p.iter().map(|p| p.0).max().unwrap() + 24000;
                let miny = p.iter().map(|p| p.1).min().unwrap() - 24000;
                let maxy = p.iter().map(|p| p.1).max().unwrap() + 24000;
                let mut exits = vec![(minx, u.y), (maxx, u.y), (u.x, miny), (u.x, maxy)];
                if i.home.0 < minx || i.home.0 > maxx || i.home.1 < miny || i.home.1 > maxy {
                    exits.push(i.home)
                }
                exits.sort_by_key(|&q| square(q, i.home));
                if !exits.into_iter().any(|q| {
                    legal(q)
                        && !(i.wall)(q.0, q.1)
                        && detour(u.pos(), q, &seg, i.wall) != Some(u.pos())
                }) {
                    return Err("Wall leaves no allied escape route");
                }
            }
        }
        Ok(p)
    }
    pub fn accept(&mut self, i: &Input, a: Action, out: &mut Vec<Cmd>) -> bool {
        let t = i.tick;
        if i.me.hp <= 0 {
            return self.reject("Owner dead");
        }
        if let Action::Dismount(p) = a {
            if let Some(r) = self.ride.as_mut() {
                if r.rider && within(i.me.pos(), p, 60000) && !(i.wall)(p.0, p.1) && legal(p) {
                    r.jump = Some((i.me.pos(), p, t));
                    r.rider = false;
                    self.reason = "Dismounting";
                    return true;
                }
            }
            return self.reject("No legal dismount");
        }
        if matches!(a, Action::Discard) {
            if let Some(p) = self.prepared.take() {
                self.recover(&p.charges, t);
                self.reason = "Prepared gadget discarded";
                return true;
            }
            return self.reject("No prepared gadget");
        }
        if i.controlled {
            return self.reject("Controlled");
        }
        if self.pending.is_some()
            || self
                .ride
                .as_ref()
                .is_some_and(|r| r.rider || r.jump.is_some())
        {
            return self.reject("Another action is preparing");
        }
        let (mut paid, mut previous_cd, mut ticks) = (vec![], None, PREPARE_TICKS as usize);
        let mut path = None;
        match &a {
            Action::Stance => {
                if !i.unlocked[2] {
                    return self.reject("Ultimate unlock required");
                };
                if t < self.stance_at {
                    return self.reject("Form switch cooling down");
                };
                self.stance_at = t + cd(STANCE_COOLDOWN as usize, i.ult_haste);
                ticks = STANCE_TICKS as usize;
            }
            Action::Wall {
                centre,
                heading,
                bend,
            } => {
                if !i.unlocked[2] {
                    return self.reject("Ultimate locked");
                };
                if t < self.ult_at {
                    return self.reject("Wall ultimate cooling down");
                };
                let p = match self.wall_path(i, *centre, *heading, *bend) {
                    Ok(p) => p,
                    Err(s) => return self.reject(s),
                };
                if length(i.me.pos(), p[0]) > 60000 && !clear(i.me.pos(), p[0], i.wall) {
                    return self.reject("Boat approach blocked");
                };
                let Some(charges)=self.spend(t,3,false)else{return self.reject("Boat Wall requires three charges");};
                paid=charges;
                out.push(Cmd::PathWarning {
                    path: p.clone(),
                    until: t + WALL_PREPARE_TICKS as usize,
                });
                path = Some(p);
                self.ult_at = t + cd(WALL_COOLDOWN as usize, i.ult_haste);
                ticks = WALL_PREPARE_TICKS as usize;
            }
            Action::Prepare(r) => {
                if *r != 6 && *r != 7 {
                    return self.reject("Only crystals and anchors use the pocket");
                };
                if !i.unlocked[0] {
                    return self.reject("Build skill locked");
                };
                if self.prepared.is_some() {
                    return self.reject("Prepared pocket occupied");
                };
                if t < self.cooldowns[cooldown_slot(*r)] {
                    return self.reject("Recipe cooling down");
                };
                let Some(p) = self.spend(t, 2, true) else {
                    return self.reject("Two unreserved charges required");
                };
                paid = p;
                ticks = HEAVY_PREPARE_TICKS as usize;
            }
            Action::Deploy(p) => {
                let Some(g) = self.prepared.as_ref() else {
                    return self.reject("Prepare a crystal or anchor first");
                };
                let r = g.recipe;
                if !i.unlocked[0] {
                    return self.reject("Build skill locked");
                };
                if !self.aimed(i, *p, 70000) || Self::occupied(i, *p) {
                    return self.reject("Invalid or occupied gadget landing");
                };
                if !self.capacity(r) {
                    return self.reject("Heavy gadget already deployed");
                };
                if t < self.cooldowns[cooldown_slot(r)] {
                    return self.reject("Recipe cooling down");
                };
                let g = self.prepared.take().unwrap();
                paid = g.charges;
                self.recover(&paid, t);
                previous_cd = Some((r as usize, self.cooldowns[cooldown_slot(r)]));
                self.cooldowns[cooldown_slot(r)] = t + cd(RECIPES[r as usize].cooldown, i.haste);
            }
            Action::Detonate(id) => {
                let Some(o) = self.objects.iter_mut().find(|o| {
                    o.id == *id && o.confirmed && (o.recipe == 3 || o.recipe == 6 || o.recipe == 7)
                }) else {
                    return self.reject("No armed owned explosive");
                };
                if o.blast.is_some() {
                    return self.reject("Detonation already committed");
                };
                o.blast = Some(t + BLAST_WARNING_TICKS as usize);
                out.push(Cmd::Warning {
                    point: o.point,
                    radius: RECIPES[o.recipe as usize].radius,
                    until: o.blast.unwrap(),
                    owner_hazard: o.recipe >= 6,
                });
                self.reason = "Detonation warning";
                return true;
            }
            Action::Repair(_) | Action::Link {..} | Action::Configure {..} | Action::Arm(_) | Action::Activate(_) | Action::Load {..} | Action::Rotate {..} => {
                return self.creative_accept(i,a,out);
            }
            Action::Recipe {recipe:15..=44,..} => return self.creative_accept(i,a,out),
            Action::Recipe {
                recipe: r,
                target,
                point,
            } => {
                if *r >= 45 {
                    return self.reject("Unknown recipe");
                };
                let d = RECIPES[*r as usize];
                if self.form==Form::Herobrine&&matches!(*r,10|11){return self.reject("Survival-only recipe");}
                if *r == 6 || *r == 7 {
                    return self.reject("Heavy gadget requires preparation");
                };
                if !i.unlocked[d.slot] {
                    return self.reject("Skill group locked");
                };
                if t < self.cooldowns[cooldown_slot(*r)] {
                    return self.reject("Recipe cooling down");
                };
                if *r == 10 {
                    if !i.units.iter().any(|u| {
                        Some(u.id) == *target
                            && u.team == i.me.team
                            && u.hp > 0
                            && within(i.me.pos(), u.pos(), d.range)
                    }) {
                        return self.reject("Apple needs a nearby living ally");
                    }
                } else if *r == 8 {
                    let q = if let Some(id) = target {
                        let Some(u) = i.units.iter().find(|u| {
                            u.id == *id
                                && u.hp > 0
                                && u.targetable
                                && (u.team == i.me.team || u.visible)
                        }) else {
                            return self.reject("Rod needs a visible target");
                        };
                        if self.form==Form::Herobrine&&u.team==i.me.team{return self.reject("Ally rescue requires Steve");}
                        u.pos()
                    } else {
                        *point
                    };
                    if !legal(q) || !within(i.me.pos(), q, d.range) {
                        return self.reject("Rod beyond range");
                    };
                    if target.is_none() && !(i.wall)(q.0,q.1) && !self.objects.iter().any(|o|o.recipe==3&&o.confirmed&&within(o.point,q,16000)) {return self.reject("Grapple needs terrain or owned TNT");}
                    ticks =
                        (((length(i.me.pos(), q)+8000).min(154000)-70000).max(0)*60+83999).div_euclid(84000).clamp(0,60) as usize;
                    ticks = ticks.max(1);
                } else if *r != 11 && *r != 13 {
                    if !self.aimed(i, *point, d.range) {
                        return self.reject("Destination unavailable");
                    };
                    if d.cap > 0 && !self.capacity(*r) {
                        return self.reject("Recipe object capacity reached");
                    };
                    if (*r == 0 || *r == 1) && Self::occupied(i, *point) {
                        return self.reject("Construction footprint occupied");
                    };
                    if *r == 0 {
                        let p = [(point.0 - 30000, point.1), (point.0 + 30000, point.1)];
                        if !clear(p[0], p[1], i.wall) || p.iter().any(|&p| !legal(p)) {
                            return self.reject("Wall footprint blocked");
                        };
                        let seg = [(p[0], p[1])];
                        if i.units.iter().any(|u| {
                            u.team == i.me.team
                                && (distance_segment(u.pos(), p[0], p[1]) < 12000
                                    || detour(u.pos(), i.home, &seg, i.wall) == Some(u.pos()))
                        }) {
                            return self.reject("Wall blocks allied escape");
                        }
                    }
                }
                if matches!(*r, 9 | 14) && i.bound {
                    return self.reject("Movement disabled");
                }
                if let Some(id) = target {
                    if *r != 8 && *r != 10 && Self::enemy(i, *id).is_none() {
                        return self.reject("Target no longer visible or targetable");
                    }
                }
                let Some(p) = self.spend(t, 1, false) else {
                    return self.reject("No unreserved utility charge");
                };
                paid = p;
                previous_cd = Some((*r as usize, self.cooldowns[cooldown_slot(*r)]));
                self.cooldowns[cooldown_slot(*r)] = t + cd(d.cooldown, i.haste);
            }
            _ => return self.reject("Unavailable action"),
        }
        let pose = if matches!(a, Action::Wall { .. } | Action::Stance) {
            2
        } else if matches!(a, Action::Recipe { recipe: 8..=14, .. }) {
            1
        } else {
            0
        };
        self.pending = Some(Pending {
            action: a.clone(),
            end: t + ticks,
            begin: t,
            bonus: self.friendly(),
            charges: paid,
            previous_cd,
            path,
            power_hp:i.me.max_hp,
        });
        self.selected = Some(a);
        self.reason = "Preparing";
        out.push(Cmd::Lock { ticks, pose });
        true
    }
    fn object(
        &mut self,
        i: &Input,
        r: u8,
        p: Point,
        paid: Vec<usize>,
        old_cd: usize,
        bonus: bool,
        power_hp:i64,
        out: &mut Vec<Cmd>,
    ) {
        let id = self.next_object;
        self.next_object += 1;
        let hp = power_hp
            * if r == 0 {
                OBJECT_WALL_HP_PCT
            } else if r == 39 {30
            } else if (15..=19).contains(&r) {25
            } else if (33..=36).contains(&r) {15
            } else if r == 1 {
                OBJECT_PISTON_HP_PCT
            } else {
                OBJECT_HEAVY_HP_PCT
            }
            / 100
            * if bonus { STEVE_OBJECT_PCT } else { 100 }
            / 100;
        let delta = (p.0 - i.me.x, p.1 - i.me.y);
        let len = length((0, 0), delta).max(1);
        let direction = (delta.0 * 10000 / len, delta.1 * 10000 / len);
        let physical = matches!(r, 0 | 1 | 3 | 6 | 7 | 15..=19 | 33..=36 | 39);
        self.objects.push(Object {
            id,
            recipe: r,
            point: p,
            direction,
            born: i.tick,
            end: i.tick + RECIPES[r as usize].life,
            active_at: i.tick + if r == 1 { 18 } else { 0 },
            blast: None,
            hp,
            confirmed: !physical,
            paid,
            old_cd,
            tick_at: i.tick,
            hit: vec![],
            power_hp,
            max_hp:hp,
        });
        if r==1{out.push(Cmd::PathWarning{path:vec![p,(p.0+direction.0*24000/10000,p.1+direction.1*24000/10000)],until:i.tick+18});}
        if physical {
            out.push(Cmd::Spawn {
                object: id,
                recipe: r,
                point: p,
                hp,
            })
        }
        if r == 0 || r==39 {
            self.walls.push(WallState {
                path: vec![(p.0 - 30000, p.1), (p.0 + 30000, p.1)],
                begins: vec![i.tick],
                finishes: vec![i.tick],
                end: i.tick + RECIPES[r as usize].life,
                ultimate: false,
                object: Some(id),
            });
        }
    }
    fn release(&mut self, i: &Input, p: Pending, out: &mut Vec<Cmd>) {
        if self.creative_release(i,&p,out){return;}
        let t = i.tick;
        match p.action {
            Action::Stance => {
                self.form = if self.form == Form::Steve {
                    Form::Herobrine
                } else {
                    Form::Steve
                };
                self.followup = None;
                self.metrics.switches += 1;
                out.push(Cmd::Fx {
                    kind: 20,
                    point: i.me.pos(),
                });
            }
            Action::Prepare(r) => {
                self.prepared = Some(Gadget {
                    recipe: r,
                    charges: p.charges,
                });
            }
            Action::Deploy(q) => {
                let r = p.previous_cd.unwrap().0 as u8;
                if !self.aimed(i, q, 70000) || Self::occupied(i, q) || !self.capacity(r) {
                    self.metrics.wasted += 1;
                    self.reason = "Gadget commitment invalidated";
                    return;
                }
                self.object(i, r, q, p.charges, p.previous_cd.unwrap().1, p.bonus,p.power_hp, out);
                self.metrics.casts[r as usize] += 1;
            }
            Action::Wall {
                centre,
                heading,
                bend,
            } => {
                let _ = (centre, heading, bend);
                let path = p.path.unwrap();
                if path.windows(2).any(|q| !clear(q[0], q[1], i.wall)) {
                    self.reason = "Committed wall path blocked";
                    self.metrics.wasted += 1;
                    return;
                }
                let start = path[0];
                let mut ride_path = path.clone();
                let mut first = 0;
                if length(i.me.pos(), start) > 60000 && clear(i.me.pos(), start, i.wall) {
                    ride_path.insert(0, i.me.pos());
                    first = 1
                } else {
                    out.push(Cmd::Move(i.me.id, start));
                }
                let mut elapsed = if first == 1 {
                    length(i.me.pos(), start)
                } else {
                    0
                };
                let (mut begins, mut finishes) = (vec![], vec![]);
                for pair in path.windows(2) {
                    begins.push(t + (elapsed / BOAT_SPEED) as usize);
                    elapsed += length(pair[0], pair[1]);
                    finishes.push(t + (elapsed / BOAT_SPEED) as usize);
                }
                let end = t + (elapsed / BOAT_SPEED) as usize;
                self.walls.push(WallState {
                    path,
                    begins,
                    finishes,
                    end: end + WALL_LIFE as usize,
                    ultimate: true,
                    object: None,
                });
                self.ride = Some(Ride {
                    path: ride_path,
                    start: t,
                    end,
                    wall_first: first,
                    rider: true,
                    hit: vec![],
                    jump: None,
                    power_hp:p.power_hp,
                });
                self.metrics.walls += 1;
            }
            Action::Recipe {
                recipe: r,
                target,
                point: q,
            } => {
                let d = RECIPES[r as usize];
                if let Some(id) = target {
                    if r != 8 && r != 10 && Self::enemy(i, id).is_none() {
                        self.reason = "Preparation lost visible target";
                        self.metrics.wasted += 1;
                        return;
                    }
                }
                self.metrics.casts[r as usize] += 1;
                match r {
                    0..=2 | 4 | 5 => {
                        if !self.aimed(i, q, d.range)
                            || ((r == 0 || r == 1) && Self::occupied(i, q))
                        {
                            self.reason = "Construction commitment invalidated";
                            return;
                        }
                        if r == 4 {
                            let removed: Vec<_> = self
                                .objects
                                .iter()
                                .filter(|o| o.recipe == 5 && within(o.point, q, d.radius + 18000))
                                .map(|o| o.id)
                                .collect();
                            self.objects.retain(|o| !removed.contains(&o.id));
                        }
                        self.object(i, r, q, p.charges, p.previous_cd.unwrap().1, p.bonus,p.power_hp, out);
                    }
                    3 | 9 | 12 | 14 => {
                        if !self.aimed(i,q,d.range){self.reason="Projectile commitment invalidated";self.metrics.wasted+=1;return;}
                        if (r == 9 || r == 14) && i.bound {
                            self.reason = "Movement disabled at commitment";
                            return;
                        }
                        let dest = if r == 9 || r == 14 {
                            clip(i.me.pos(), q, i.wall)
                        } else {
                            q
                        };
                        let duration = match r {
                            3 => 15,
                            9 => 18,
                            14 => (length(i.me.pos(), dest) / 2500).max(1) as usize,
                            _ => (length(i.me.pos(), dest) / 7000).max(1) as usize,
                        };
                        self.shots.push(Projectile {
                            recipe: r,
                            from: i.me.pos(),
                            point: i.me.pos(),
                            dest,
                            begin: t,
                            end: t + duration,
                            target,
                            hit: vec![],
                            paid: p.charges.clone(),
                            old_cd: p.previous_cd.map_or(0, |c| c.1),
                            bonus: p.bonus,
                            travelled: 0,
                            power_hp:p.power_hp,
                        });
                        if r == 12 {
                            out.push(Cmd::PathWarning {
                                path: vec![i.me.pos(), dest],
                                until: t + duration,
                            });
                        }
                        if r == 3 {
                            out.push(Cmd::Warning {
                                point: dest,
                                radius: 32000,
                                until: t + duration + 30,
                                owner_hazard: false,
                            })
                        }
                    }
                    8 => {
                        let dest = if let Some(id) = target {
                            let Some(u) = i.units.iter().find(|u| {
                                u.id == id
                                    && u.hp > 0
                                    && u.targetable
                                    && (u.team == i.me.team || u.visible)
                            }) else {
                                self.reason = "Rod lost target";
                                return;
                            };
                            u.pos()
                        } else {
                            q
                        };
                        let reach=(70000+84000*(p.end-p.begin)as i64/60).min(154000);
                        if !within(i.me.pos(),dest,reach){self.reason="Rod left charged reach";self.metrics.wasted+=1;return;}
                        let duration = (length(i.me.pos(), dest) / 4200).max(1) as usize;
                        self.shots.push(Projectile {
                            recipe: r,
                            from: i.me.pos(),
                            point: i.me.pos(),
                            dest,
                            begin: t,
                            end: t + duration,
                            target,
                            hit: vec![],
                            paid: p.charges.clone(),
                            old_cd: p.previous_cd.map_or(0, |c| c.1),
                            bonus: p.bonus,
                            travelled: 0,
                            power_hp:p.power_hp,
                        });
                    }
                    10 => {
                        if let Some(u) = i.units.iter().find(|u| {
                            Some(u.id) == target
                                && u.team == i.me.team
                                && u.hp > 0
                                && within(i.me.pos(), u.pos(), 60000)
                        }) {
                            out.push(Cmd::Heal(
                                u.id,
                                boosted(APPLE_BASE + p.power_hp * APPLE_HP_PCT / 100, p.bonus),
                            ));
                            out.push(Cmd::Shield(
                                u.id,
                                boosted(p.power_hp * APPLE_SHIELD_PCT / 100, p.bonus),
                                240,
                            ));
                        }
                    }
                    11 => out.push(Cmd::Shield(
                        i.me.id,
                        boosted(GUARD_BASE + p.power_hp * GUARD_HP_PCT / 100, p.bonus),
                        90,
                    )),
                    13 => {self.axe_until = t + 120;self.axe_hp=p.power_hp;},
                    _ => {}
                }
            }
            _ => {}
        }
        self.reason = "Resolved";
    }
    fn explode(&mut self, i: &Input, o: &Object, segments: &[(Point, Point)], out: &mut Vec<Cmd>) {
        let r = o.recipe;
        let (base,pct) = match r {3=>(TNT_BASE,TNT_HP_PCT),6=>(CRYSTAL_BASE,CRYSTAL_HP_PCT),_=>(ANCHOR_BASE,ANCHOR_HP_PCT)};
        let radius = RECIPES[r as usize].radius;
        for u in i.units.iter().filter(|u| {
            u.hp > 0
                && !u.tower
                && !u.gadget
                && within(u.pos(), o.point, radius)
                && (u.team != i.me.team || u.id == i.me.id && r >= 6)
        }) {
            if r >= 6
                && (!clear(o.point, u.pos(), i.wall)
                    || blocked(o.point,u.pos(),segments))
            {
                continue;
            }
            let raw=creative::hp_hit(u,base,o.power_hp,pct*100);
            let damage = if u.id == i.me.id { raw / 2 } else { raw };
            self.damage_with_hp(
                i,
                u,
                if r == 3 { damage } else { 0 },
                if r == 3 { 0 } else { damage },
                u.id != i.me.id,
                o.power_hp,
                out,
            );
            if r == 3 && u.team != i.me.team && !u.immovable && !u.cc_immune {
                let delta = (u.x - o.point.0, u.y - o.point.1);
                let len = length(o.point, u.pos()).max(1);
                let dest = (u.x + delta.0 * 36000 / len, u.y + delta.1 * 36000 / len);
                out.push(Cmd::Push(u.id, clip(u.pos(), dest, i.wall), 12));
            }
        }
        out.push(Cmd::Fx {
            kind: r,
            point: o.point,
        });
        out.push(Cmd::Remove(o.id));
    }
    fn objects_step(&mut self, i: &Input, out: &mut Vec<Cmd>) {
        self.refresh_aura_winners(i);
        let mut segments = self.segments(i.tick);segments.extend_from_slice(i.barriers);
        let mut objects = std::mem::take(&mut self.objects);
        for mut o in objects.drain(..) {
            if !o.confirmed {
                self.objects.push(o);
                continue;
            }
            let hp = i.objects.iter().find(|v| v.0 == o.id).map(|v| v.1);
            if hp == Some(0) && o.recipe == 6 {
                if o.blast.is_none() {
                    o.blast = Some(i.tick + BLAST_WARNING_TICKS as usize);
                    out.push(Cmd::Warning {
                        point: o.point,
                        radius: 32000,
                        until: o.blast.unwrap(),
                        owner_hazard: true,
                    });
                }
            } else if hp == Some(0) {
                out.push(Cmd::Remove(o.id));
                continue;
            }
            if let Some(hp) = hp {
                o.hp = hp
            }
            if o.recipe == 3
                && i.units
                    .iter()
                    .any(|u| u.team == i.me.team && u.swing && within(u.pos(), o.point, 22000))
            {
                o.blast = Some(o.blast.map_or(i.tick + 12, |t| t.min(i.tick + 6)));
            }
            if o.blast.is_some_and(|t| i.tick >= t) {
                self.explode(i, &o, &segments, out);
                continue;
            }
            if i.tick >= o.end && o.blast.is_none() {
                out.push(Cmd::Remove(o.id));
                continue;
            }
            if i.tick < o.active_at {
                self.objects.push(o);
                continue;
            }
            let enemies: Vec<_> = i
                .units
                .iter()
                .filter(|u| u.team != i.me.team && u.hp > 0 && u.targetable)
                .collect();
            match o.recipe {
                1 if o.hit.is_empty() => {
                    for u in enemies.iter().filter(|u| !u.immovable && !u.cc_immune) {
                        let dx = u.x - o.point.0;
                        let dy = u.y - o.point.1;
                        let along = (dx * o.direction.0 + dy * o.direction.1) / 10000;
                        let cross = (dx * o.direction.1 - dy * o.direction.0).abs() / 10000;
                        if (0..=24000).contains(&along) && cross <= 10000 {
                            let dest = (
                                u.x + o.direction.0 * 18000 / 10000,
                                u.y + o.direction.1 * 18000 / 10000,
                            );
                            let landing = clip(u.pos(), dest, i.wall);
                            if landing != u.pos() {
                                out.push(Cmd::Move(u.id, landing));
                                self.utility_control(i, u.id);
                            }
                        }
                    }
                    o.hit.push(i.me.id);
                }
                2 => {
                    for u in enemies
                        .iter()
                        .filter(|u| !u.cc_immune && within(u.pos(), o.point, 18000))
                    {
                        out.push(Cmd::Slow(u.id, 35, 15));
                        if !o.hit.contains(&u.id) {
                            self.utility_control(i, u.id);
                            o.hit.push(u.id)
                        }
                    }
                }
                3 if o.blast.is_none() => {
                    if enemies.iter().any(|u| within(u.pos(), o.point, 20000)) {
                        o.blast = Some(i.tick + 12);
                        out.push(Cmd::Warning {
                            point: o.point,
                            radius: 32000,
                            until: i.tick + 12,
                            owner_hazard: false,
                        });
                    }
                }
                4 => {
                    for u in i
                        .units
                        .iter()
                        .filter(|u| u.team == i.me.team && within(u.pos(), o.point, 20000))
                    {
                        out.push(Cmd::Speed(u.id, 15, 15))
                    }
                }
                5 if i.tick >= o.tick_at => {
                    for u in enemies.iter().filter(|u| within(u.pos(), o.point, 18000)) {
                        self.damage_with_hp(i, u, 0, creative::hp_hit(u,LAVA_BASE,o.power_hp,LAVA_HP_PERMYRIAD), true,o.power_hp, out)
                    }
                    o.tick_at = i.tick + 30;
                }
                _ => {}
            }
            if o.recipe>=15&&i.tick.is_multiple_of(15){self.creative_object(i,&mut o,out);}
            self.objects.push(o);
        }
        let live: Vec<_> = self.objects.iter().map(|o| o.id).collect();
        self.walls
            .retain(|w| i.tick < w.end && w.object.is_none_or(|id| live.contains(&id)));
    }
    fn utility_control(&mut self, i: &Input, id: usize) {
        if self.form == Form::Herobrine
            && self.followup.is_none()
            && i.tick >= self.followup_at
            && Self::enemy(i, id).is_some_and(|u| u.champion && !u.immovable && !u.cc_immune)
        {
            self.followup = Some((id, i.tick + FOLLOWUP_TICKS as usize));self.followup_hp=i.me.max_hp;
        }
    }
    fn projectiles_step(&mut self, i: &Input, out: &mut Vec<Cmd>) {
        let shots = std::mem::take(&mut self.shots);
        for mut s in shots {
            if self.creative_shot(i,&mut s,out){if i.tick<s.end{self.shots.push(s);}continue;}
            let duration = (s.end - s.begin).max(1);
            let dt = i.tick.saturating_sub(s.begin).min(duration);
            let mut next = (
                s.from.0 + (s.dest.0 - s.from.0) * dt as i64 / duration as i64,
                s.from.1 + (s.dest.1 - s.from.1) * dt as i64 / duration as i64,
            );
            let mut done = false;
            if s.recipe == 8 {
                if let Some(u) = i
                    .units
                    .iter()
                    .find(|u| Some(u.id) == s.target && u.team == i.me.team && u.hp > 0)
                {
                    next = toward(s.point, u.pos(), 4200);
                    s.travelled += length(s.point, next);
                    if s.travelled < 154000 {
                        s.end = i.tick + 1;
                    }
                }
            }

            if s.recipe == 8 {
                if let Some(n) = self.objects.iter().position(|o| {
                    o.recipe == 3
                        && o.confirmed
                        && distance_segment(o.point, s.point, next) <= 16000
                }) {
                    let o = self.objects.remove(n);
                    out.push(Cmd::Remove(o.id));
                    if let Some(f) = i
                        .units
                        .iter()
                        .filter(|u| u.team != i.me.team && u.hp > 0 && u.visible && u.targetable)
                        .min_by_key(|u| (square(o.point, u.pos()), u.id))
                    {
                        self.shots.push(Projectile {
                            recipe: 3,
                            from: o.point,
                            point: o.point,
                            dest: f.pos(),
                            begin: i.tick,
                            end: i.tick + 14,
                            target: Some(f.id),
                            hit: vec![],
                            paid: vec![],
                            old_cd: self.cooldowns[3],
                            bonus: false,
                            travelled: 0,
                            power_hp:o.power_hp,
                        });
                    }
                    done = true;
                }
            }

            if !done && (s.recipe == 12 || s.recipe == 8) {
                if s.recipe == 8
                    && s.target
                        .and_then(|id| i.units.iter().find(|u| u.id == id))
                        .is_some_and(|u| u.team == i.me.team)
                {
                    if let Some(u) = i.units.iter().find(|u| Some(u.id) == s.target) {
                        if within(next, u.pos(), 16000) {
                            self.reels.push(Reel {
                                target: u.id,
                                end: i.tick + 35,
                                ally: true,
                            });
                            done = true
                        }
                    }
                } else {
                    let hit = i
                        .units
                        .iter()
                        .filter(|u| {
                            u.team != i.me.team
                                && u.hp > 0
                                && u.visible
                                && u.targetable
                                && (s.recipe == 8 || clear(s.point, u.pos(), i.wall))
                                && distance_segment(u.pos(), s.point, next)
                                    <= if s.recipe == 8 { 16000 } else { 4000 }
                        })
                        .min_by_key(|u| (square(s.point, u.pos()), u.id));
                    if let Some(u) = hit {
                        let damage = if s.recipe == 8 {
                            creative::hp_hit(u,ROD_BASE,s.power_hp,ROD_HP_PCT*100)
                        } else {
                            creative::hp_hit(u,CROSSBOW_BASE,s.power_hp,CROSSBOW_HP_PCT*100)
                        };
                        self.damage_with_hp(i, u, damage, 0, true,s.power_hp, out);
                        if s.recipe == 8 && !u.immovable && !u.cc_immune {
                            self.reels.push(Reel {
                                target: u.id,
                                end: i.tick + 40,
                                ally: false,
                            })
                        }
                        done = true;
                    }
                }
                if s.recipe == 12 && !clear(s.point, next, i.wall) {
                    done = true;
                }
            }
            s.point = next;
            if s.recipe == 14 {
                if i.controlled || i.bound || blocked(s.point,next,&self.segments(i.tick)) || blocked(s.point,next,i.barriers) {
                    done = true
                } else {
                    out.push(Cmd::Move(i.me.id, next));
                    self.trails.push((s.point, next, i.tick + 180));
                    for u in i.units.iter().filter(|u| {
                        u.team == i.me.team && distance_segment(u.pos(), s.from, next) < 12000
                    }) {
                        out.push(Cmd::Speed(u.id, 25, 180))
                    }
                }
            }
            if i.tick >= s.end && !done {
                match s.recipe {
                    8 => {
                        if s.target.is_none() && (i.wall)(s.dest.0,s.dest.1) {
                            let p = clip(s.from, s.dest, i.wall);
                            self.grapple = Some((p, i.tick + 60));
                        }
                    }
                    3 => {
                        self.object(i, 3, s.dest, s.paid.clone(), s.old_cd, s.bonus,s.power_hp, out);
                        if let Some(o) = self.objects.last_mut() {
                            o.active_at = i.tick + 1;
                            if i.units
                                .iter()
                                .any(|u| u.team != i.me.team && within(u.pos(), s.dest, 32000))
                            {
                                o.blast = Some(i.tick + 30);
                            }
                        }
                    }
                    9 => {
                        if !i.controlled && !i.bound && !(i.wall)(s.dest.0, s.dest.1) {
                            out.push(Cmd::Fx{kind:9,point:i.me.pos()});
                            out.push(Cmd::Move(i.me.id, s.dest));
                            out.push(Cmd::Fx{kind:9,point:s.dest});
                            for r in self.reels.iter().filter(|r| !r.ally) {
                                if let Some(u) = i.units.iter().find(|u| u.id == r.target) {
                                    let p = toward(s.dest, u.pos(), 10000);
                                    if !(i.wall)(p.0, p.1) {
                                        out.push(Cmd::Move(u.id, p))
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
                done = true;
            }
            if !done {
                self.shots.push(s)
            }
        }
        let reels = std::mem::take(&mut self.reels);
        for r in reels {
            if let Some(u) = i.units.iter().find(|u| u.id == r.target && u.hp > 0) {
                if i.tick < r.end && length(u.pos(), i.me.pos()) > 10000 {
                    let p = toward(u.pos(), i.me.pos(), if r.ally { 5000 } else { 4200 });
                    if !(i.wall)(p.0, p.1) {
                        out.push(Cmd::Move(u.id, p));
                    }
                    self.reels.push(r)
                }
            }
        }
    }
    fn travel_step(&mut self, i: &Input, out: &mut Vec<Cmd>) {
        if let Some((p, end)) = self.grapple {
            if i.tick < end && !i.controlled && !i.bound && length(i.me.pos(), p) > 3500 {
                out.push(Cmd::Move(i.me.id, toward(i.me.pos(), p, 3500)))
            } else {
                self.grapple = None;
            }
        }
        self.trails.retain(|v| i.tick < v.2);
        for u in i.units.iter().filter(|u| {
            u.team == i.me.team
                && u.hp > 0
                && self
                    .trails
                    .iter()
                    .any(|&(a, b, _)| distance_segment(u.pos(), a, b) < 12000)
        }) {
            out.push(Cmd::Speed(u.id, 25, 15));
        }
    }
    fn ride_step(&mut self, i: &Input, out: &mut Vec<Cmd>) {
        let Some(mut r) = self.ride.take() else {
            return;
        };
        let distance = i.tick.saturating_sub(r.start) as i64 * BOAT_SPEED;
        let (p, n) = at(&r.path, distance);
        if r.rider {
            out.push(Cmd::Move(i.me.id, p));
            if i.me.pct() < 35 {
                let to = toward(p, i.home, 25000);
                r.jump = Some((p, clip(p, to, i.wall), i.tick));
                r.rider = false;
            }
        }
        if let Some((a, b, t)) = r.jump {
            let end = t + 14;
            if i.tick >= end {
                out.push(Cmd::Move(i.me.id, b));
                r.jump = None;
            } else {
                out.push(Cmd::Move(
                    i.me.id,
                    toward(a, b, length(a, b) * (i.tick - t) as i64 / 14),
                ));
                out.push(Cmd::Immune(2));
            }
        }
        if n >= r.wall_first {
            for u in i
                .units
                .iter()
                .filter(|u| {
                    u.team != i.me.team
                        && u.hp > 0
                        && !u.tower
                        && within(u.pos(), p, 16000)
                        && !r.hit.contains(&u.id)
                })
                .collect::<Vec<_>>()
            {
                self.damage_with_hp(
                    i,
                    u,
                    creative::hp_hit(u,RAM_BASE,r.power_hp,RAM_HP_PCT*100),
                    0,
                    true,
                    r.power_hp,
                    out,
                );
                if !u.immovable && !u.cc_immune {
                    let a = r.path[n];
                    let b = r.path[n + 1];
                    let d = (b.0 - a.0, b.1 - a.1);
                    let l = length(a, b).max(1);
                    let side = if (u.x - p.0) * -d.1 + (u.y - p.1) * d.0 >= 0 {
                        1
                    } else {
                        -1
                    };
                    let dest = (u.x - d.1 * side * 42000 / l, u.y + d.0 * side * 42000 / l);
                    out.push(Cmd::Push(u.id, clip(u.pos(), dest, i.wall), 12))
                }
                r.hit.push(u.id);
            }
        }
        if r.rider {
            out.push(Cmd::Immune(2))
        }
        for u in i.units.iter().filter(|u| {
            u.team == i.me.team
                && self
                    .segments(i.tick)
                    .iter()
                    .any(|&(a, b)| distance_segment(u.pos(), a, b) < 18000)
        }) {
            out.push(Cmd::Speed(u.id, 25, 15))
        }
        if i.tick < r.end || r.jump.is_some() {
            self.ride = Some(r)
        }
    }
    pub fn parried(&mut self) {
        self.walls.retain(|w| !w.ultimate);
        self.ride = None;
        self.reason = "Boat parried; ultimate wall broken";
    }
    pub fn step(&mut self, i: &Input) -> Vec<Cmd> {
        let mut out = vec![];
        if self.last_tick.is_some_and(|t| i.tick <= t) {
            self.reason = "Duplicate or out-of-order tick";
            return out;
        }
        self.last_tick = Some(i.tick);
        self.last_owner = i.me.id;
        if i.me.hp <= 0 {
            return self.died(i.tick);
        }
        self.receipts.retain(|r| i.tick <= r.at + 3);
        if self
            .followup
            .is_some_and(|(id, t)| i.tick > t || Self::enemy(i, id).is_none())
        {
            self.followup = None
        }
        for &(event, target) in i.basics {
            if event <= self.last_basic {
                continue;
            }
            self.last_basic = event;
            let Some(u) = Self::enemy(i, target) else {
                continue;
            };
            if self.axe_until >= i.tick && self.axe_until > 0 {
                self.damage(i, u, creative::hp_hit(u,0,self.axe_hp,AXE_HP_PCT*100), 0, false, &mut out);
                if !u.cc_immune {
                    out.push(Cmd::Slow(target, 25, 45));
                }
                self.axe_until = 0
            }
            if self.form == Form::Herobrine
                && self
                    .followup
                    .is_some_and(|(id, end)| id == target && i.tick <= end)
                && i.tick >= self.followup_at
            {
                self.damage(
                    i,
                    u,
                    creative::hp_hit(u,FOLLOWUP_BASE,self.followup_hp,FOLLOWUP_HP_PCT*100),
                    0,
                    false,
                    &mut out,
                );
                self.followup = None;
                self.followup_at = i.tick + FOLLOWUP_COOLDOWN as usize;
                self.metrics.combos += 1;
            }
        }
        if i.controlled {
            if let Some(p) = self.pending.take() {
                if matches!(p.action, Action::Prepare(_)) {
                    self.recover(&p.charges, i.tick)
                }
                self.reason = "Interrupted";
                self.metrics.interrupted += 1;
            }
            self.shots.retain(|s| s.recipe != 9 && s.recipe != 14);
        }
        if self.pending.as_ref().is_some_and(|p| i.tick >= p.end) {
            let p = self.pending.take().unwrap();
            self.release(i, p, &mut out)
        }
        self.projectiles_step(i, &mut out);
        self.travel_step(i, &mut out);
        self.objects_step(i, &mut out);
        self.creative_step(i,&mut out);
        self.ride_step(i, &mut out);
        let segments = self.segments(i.tick);
        for u in i.units.iter().filter(|u| {
            u.team == i.me.team
                && u.hp > 0
                && segments
                    .iter()
                    .any(|&(a, b)| distance_segment(u.pos(), a, b) < 18000)
        }) {
            out.push(Cmd::Speed(u.id, 25, 15));
        }
        let action = if let Some(a) = i.action.clone() {
            self.preview.clear();
            Some(a)
        } else if i.automatic && i.tick >= self.last_ai && self.pending.is_none() {
            self.last_ai = i.tick + 10;
            self.plan(i)
        } else {
            None
        };
        if let Some(a) = action {
            self.accept(i, a, &mut out);
        }
        out
    }
}
