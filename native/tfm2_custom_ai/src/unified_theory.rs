//! The Unified Theory. One body, three notebooks, one conserved experimental budget.
//! All decisions use integer arithmetic and a private seeded RNG. Rendering and I/O never advance it.
use crate::mastery::{signature, Book, Record};
use crate::unified_theory_data::{COMBOS, SKILLS};
use crate::unified_theory_math::{instability, strength, unit, Pool, Window};
use crate::{champions, d2, fx_point, fx_unit, sq, timed, walls, Champ};
use mod_api_stable::{AttackTypeV1, BuffV1, CcV1, StablePassive, StableSim, StatV1, UnitAttackV1};
use std::collections::{BTreeMap, VecDeque};
use std::io::Write;
pub static BOOK: Book = Book::new("unified_theory");
const ID: &str = "tfm2_custom_unified_theory";
pub const RANKS: [&str; 8] = [
    "Student",
    "Lab Assistant",
    "Researcher",
    "Scientist",
    "Professor",
    "Fellow",
    "Laureate",
    "Unified Mind",
];
#[derive(Clone)]
struct Stage {
    skill: usize,
    charge: Vec<u32>,
    token: usize,
    ready: usize,
    started: usize,
    error: bool,
}
#[derive(Clone)]
struct Experiment {
    id: u64,
    target: usize,
    stages: VecDeque<Stage>,
    power: usize,
    physical: usize,
}
#[derive(Clone)]
struct Anchor {
    x: i64,
    y: i64,
    expires: usize,
}
#[derive(Clone)]
struct Field {
    kind: usize,
    x: i64,
    y: i64,
    expires: usize,
    experiment: u64,
    payload: usize,
    entity: Option<usize>,
}
#[derive(Clone)]
struct Packet {
    skill: usize,
    experiment: u64,
    x: i64,
    y: i64,
    dx: i64,
    dy: i64,
    payload: usize,
    physical: bool,
    class: usize,
    expires: usize,
    release: usize,
    age: usize,
    split: usize,
    curve: i64,
    orbit: bool,
    hits: Vec<usize>,
}
#[derive(Clone, Default)]
struct Theory {
    target: Option<usize>,
    until: usize,
    origin: Option<(i64, i64)>,
    vector: (i64, i64),
    velocity: (i64, i64),
    acceleration: (i64, i64),
    aim: Option<(i64, i64)>,
    sync: usize,
    proper: bool,
    cone: bool,
    link: bool,
    curve: i64,
    blue: bool,
    pressure: bool,
    exchange: bool,
    drag: i64,
    scalar: usize,
    orbit: bool,
    elastic: bool,
    root: bool,
    boundary: bool,
    work: usize,
    series: usize,
    impulse: usize,
    sample: bool,
    purity: usize,
    acid: bool,
    base: bool,
    catalyst: bool,
    order: usize,
    label: bool,
    half: usize,
}
#[derive(Clone)]
pub struct UnifiedTheory {
    key: u64,
    owner: usize,
    started: bool,
    last_tick: Option<usize>,
    pool: Pool,
    material: u32,
    momentum: usize,
    rank: usize,
    athlete: Option<usize>,
    rank_set: bool,
    rng: u64,
    cooldown: Vec<usize>,
    plan: Option<Experiment>,
    next_id: u64,
    budgets: BTreeMap<u64, usize>,
    packets: Vec<Packet>,
    fields: Vec<Field>,
    anchors: Vec<Anchor>,
    theory: Theory,
    last_pos: Option<(i64, i64)>,
    motion: (i64, i64),
    vfx_left: usize,
    observations: BTreeMap<usize, ((i64, i64), (i64, i64))>,
    damage: BTreeMap<usize, Window>,
    controls: BTreeMap<usize, Window>,
    moves: BTreeMap<usize, Window>,
    support: Window,
    last_offense: usize,
    cursor: usize,
    goal: Option<usize>,
    persona: usize,
    shown: usize,
    record: Record,
    sig: Option<String>,
    trace: Vec<String>,
    dropped: usize,
    casts: usize,
    failures: usize,
}
impl Default for UnifiedTheory {
    fn default() -> Self {
        Self {
            key: 0,
            owner: 0,
            started: false,
            last_tick: None,
            pool: Pool::default(),
            material: 8,
            momentum: 0,
            rank: 0,
            athlete: None,
            rank_set: false,
            rng: 1,
            cooldown: vec![0; 75],
            plan: None,
            next_id: 0,
            budgets: BTreeMap::new(),
            packets: vec![],
            fields: vec![],
            anchors: vec![],
            theory: Theory::default(),
            last_pos: None,
            motion: (0, 0),
            vfx_left: 6,
            observations: BTreeMap::new(),
            damage: BTreeMap::new(),
            controls: BTreeMap::new(),
            moves: BTreeMap::new(),
            support: Window::default(),
            last_offense: 0,
            cursor: 0,
            goal: None,
            persona: 0,
            shown: usize::MAX,
            record: Record::default(),
            sig: None,
            trace: vec![],
            dropped: 0,
            casts: 0,
            failures: 0,
        }
    }
}
fn offense(s: usize) -> bool {
    matches!(
        s,
        0 | 16 | 19 | 29 | 34 | 39 | 40 | 54 | 59 | 63 | 65 | 71 | 72 | 73 | 74
    )
}
fn heavy(s: usize) -> bool {
    matches!(s, 34 | 63 | 71 | 73 | 74)
}
fn physical(s: usize) -> bool {
    matches!(s, 16 | 29 | 34 | 39 | 40)
}
fn manipulates(s: usize) -> bool {
    matches!(s, 27 | 37 | 38 | 39 | 46)
}
fn material_cost(s: usize) -> u32 {
    if s < 50 {
        0
    } else if matches!(s, 63 | 71 | 73 | 74) {
        2
    } else if matches!(s, 54 | 57 | 58 | 59 | 65 | 66 | 72) {
        1
    } else {
        0
    }
}
fn cost(a: &[u32]) -> u32 {
    a.iter().sum::<u32>() * 100
}
impl UnifiedTheory {
    fn vfxp(&mut self, sim: &mut StableSim<'_>, tag: &str, me: usize, x: i64, y: i64, ticks: u64) {
        if self.vfx_left > 0 {
            self.vfx_left -= 1;
            fx_point(sim, tag, me, x, y, ticks);
        }
    }
    fn vfxu(&mut self, sim: &mut StableSim<'_>, tag: &str, me: usize, target: usize, ticks: u64) {
        if self.vfx_left > 0 {
            self.vfx_left -= 1;
            fx_unit(sim, tag, me, target, ticks);
        }
    }
    fn random(&mut self, n: u64) -> u64 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 7;
        self.rng ^= self.rng << 17;
        self.rng % n.max(1)
    }
    fn log(&mut self, s: String) {
        if self.trace.len() < 64 {
            self.trace.push(s)
        } else {
            self.dropped = self.dropped.saturating_add(1);
        }
    }
    fn flush(&mut self, sim: &StableSim<'_>, me: usize, force: bool) {
        if self.trace.is_empty() || (!force && !sim.tick().is_multiple_of(120)) {
            return;
        }
        let mut lines = std::mem::take(&mut self.trace);
        if self.dropped > 0 {
            lines.push(format!(
                "{} additional diagnostic events omitted by the bounded trace buffer",
                self.dropped
            ));
            self.dropped = 0;
        }
        if sim
            .sim_origin()
            .is_some_and(|o| o.kind == mod_api_stable::SimOriginKindV1::ServerPresim.code())
        {
            return;
        }
        // Watched origins keep separate traces; presimulation never suppresses the live view.
        let key = format!("ut-trace:{}:{me}:{}", sim.seed(), sim.tick());
        let key = format!("{key}:{}", sim.sim_origin().map_or(0, |o| o.kind));
        if !BOOK.first(key) {
            return;
        }
        if let Some(dir) = crate::mod_dir() {
            let _ = append_trace(
                &dir.join("unified_theory_damage_log.txt"),
                &lines,
                8 * 1024 * 1024,
            );
        }
    }

    fn cancel(&mut self) {
        if let Some(p) = self.plan.take() {
            let n = p.stages.iter().map(|s| cost(&s.charge)).sum();
            self.pool.refund(n);
        }
    }
    fn valid(&self, s: usize) -> bool {
        let t = &self.theory;
        match s {
            7 | 24 => !self.anchors.is_empty(),
            19 => self.anchors.len() >= 2,
            10 => t.cone,
            11 => t.origin.is_some(),
            17 => self.material > 0,
            22 => t.impulse > 0,
            27 | 28 | 29 | 30 | 32 | 33 => t.vector != (0, 0),
            34 => self.momentum >= 10,
            37 | 38 | 39 | 46 => !self.packets.is_empty(),
            43 => self.momentum > 0,
            44 | 45 => t.aim.is_some(),
            47 => self.fields.iter().any(|f| f.kind == 57),
            49 => t.origin.is_some() && t.vector != (0, 0),
            51 | 52 | 69 => t.sample,
            56 => t.acid && t.base,
            62 => self.fields.iter().any(|f| matches!(f.kind, 58 | 65 | 66)),
            63 => t.sample && t.label,
            70 => t.label,
            74 => t.label && t.half > 0,
            _ => true,
        }
    }
    /// Templates may require a previous experiment (crystal, measurement, isotope, etc.).
    /// Prepare missing prerequisites separately; their charge is never hidden or free.
    fn preparation(&self, steps: &[usize]) -> Option<usize> {
        let mut seen = Vec::new();
        for &s in steps {
            let has = |id| seen.contains(&id);
            let packet = self.packets.len() > 0
                || seen
                    .iter()
                    .any(|s| matches!(s, 0 | 16 | 40 | 54 | 59 | 63 | 72 | 73 | 74));
            let needed = match s {
                7 | 24 if self.anchors.is_empty() && !has(1) => Some(1),
                19 if self.anchors.len() + seen.iter().filter(|s| **s == 1).count() < 2 => Some(1),
                10 if !self.theory.cone && !has(9) && !has(48) => Some(9),
                11 | 49 if self.theory.origin.is_none() && !has(1) && !has(25) => Some(25),
                22 if self.theory.impulse == 0 && !has(32) && !has(49) => Some(32),
                27 | 28 | 29 | 30 | 32 | 33 | 49 if self.theory.vector == (0, 0) && !has(26) => {
                    Some(26)
                }
                34 if self.momentum < 10 && !has(32) => Some(32),
                37 | 38 | 39 | 46 if !packet => Some(0),
                43 if self.momentum == 0 && !has(32) => Some(32),
                44 | 45
                    if self.theory.aim.is_none() && !has(42) && !has(23) && !has(3) && !has(48) =>
                {
                    Some(42)
                }
                47 if !self.fields.iter().any(|f| f.kind == 57) && !has(57) => Some(57),
                51 | 52 | 63 | 69 if !self.theory.sample && !has(50) => Some(50),
                56 if !self.theory.acid && !has(54) => Some(54),
                56 if !self.theory.base && !has(55) => Some(55),
                62 if !self.fields.iter().any(|f| matches!(f.kind, 58 | 65 | 66))
                    && !has(58)
                    && !has(65)
                    && !has(66) =>
                {
                    Some(65)
                }
                63 | 70 | 74 if !self.theory.label && !has(69) => Some(69),
                74 if self.theory.half == 0 && !has(70) => Some(70),
                _ => None,
            };
            if let Some(n) = needed {
                return self.preparation(&[n]).or(Some(n));
            }
            seen.push(s);
        }
        None
    }
    fn pick(&mut self, sim: &StableSim<'_>, m: &Champ, enemies: &[Champ]) {
        let target = enemies
            .iter()
            .filter(|c| d2(m.x, m.y, c.x, c.y) <= sq(100000))
            .min_by_key(|c| {
                (
                    usize::from(self.rank >= 3 && c.name.is_empty()),
                    d2(m.x, m.y, c.x, c.y),
                    c.id,
                )
            });
        let Some(target) = target else { return };
        if self.theory.target != Some(target.id) {
            self.theory = Theory {
                target: Some(target.id),
                origin: self.anchors.last().map(|a| (a.x, a.y)),
                ..Theory::default()
            };
        }
        if let Some(goal) = self.goal {
            let wanted = if goal < 75 {
                vec![goal]
            } else {
                COMBOS[goal - 75].1.to_vec()
            };
            let required = if let Some(prep) = self.preparation(&wanted) {
                cost(SKILLS[prep].charge)
            } else {
                wanted.iter().map(|s| cost(SKILLS[*s].charge)).sum()
            };
            // Keep a prepared goal while charge recovers. Cheap filler casts would starve 96-CU recipes forever.
            if required > self.pool.free {
                return;
            }
        }
        let comfort = [1, 1, 2, 2, 3, 4, 5, 5][self.rank];
        let mut best: Option<(i64, Vec<usize>, Option<usize>)> = None;
        // A bounded rotating window explores all 125 choices. No combinatorial search.
        for j in 0..32 {
            let n = if j == 0 {
                self.goal.unwrap_or(self.cursor)
            } else {
                (self.cursor + j) % 125
            };
            let wanted = if n < 75 {
                vec![n]
            } else {
                COMBOS[n - 75].1.to_vec()
            };
            let prep = self.preparation(&wanted);
            let steps = if let Some(s) = prep { vec![s] } else { wanted };
            let total: u32 = steps.iter().map(|s| cost(SKILLS[*s].charge)).sum();
            let material: u32 = steps.iter().map(|s| material_cost(*s)).sum();
            if total > self.pool.free
                || material > self.material
                || steps.iter().any(|s| self.cooldown[*s] > sim.tick())
                || !self.valid(steps[0])
            {
                continue;
            }
            if steps.contains(&71) && d2(m.x, m.y, target.x, target.y) > sq(30000) {
                continue;
            }
            let attack = steps.iter().any(|s| offense(*s));
            let protect = steps.iter().any(|s| SKILLS[*s].role == "Protect");
            let mut score = if attack { 100 } else { 40 };
            if protect && m.hp * 2 < m.max_hp {
                score += 110;
            }
            if steps.len() <= comfort {
                score += self.rank as i64 * 10;
            } else {
                score -= ((steps.len() - comfort) * 25) as i64;
            }
            // Long plans are possible even for Students. Rank changes selection, never token speed.
            score += self.random(if self.rank == 7 {
                3
            } else {
                (8 - self.rank) * 12
            } as u64) as i64;
            score -= j as i64 / 4;
            if self.goal == Some(n) {
                score += 500;
            }
            if prep.is_some() {
                score += if self.rank >= 3 { 65 } else { 20 };
            }
            if best.as_ref().is_none_or(|(v, _, _)| score > *v) {
                best = Some((score, steps, prep.map(|_| n)));
            }
        }
        self.cursor = (self.cursor + 32) % 125;
        let Some((_, steps, goal)) = best else { return };
        self.goal = goal;
        let mut stages = VecDeque::new();
        let mut reserved = 0;
        for s in steps {
            let jitter = [40, 30, 22, 16, 10, 6, 3, 1][self.rank];
            let mut a: Vec<u32> = SKILLS[s]
                .charge
                .iter()
                .map(|w| {
                    let delta = self.random((jitter * 2 + 1) as u64) as i64 - jitter;
                    ((*w as i64 * (100 + delta) + 50) / 100).max(1) as u32
                })
                .collect();
            // Review corrects charges, without changing the common activation speed.
            if self.random(100) < [15, 25, 40, 55, 70, 85, 96, 100][self.rank] {
                a = SKILLS[s].charge.to_vec();
            }
            reserved += cost(&a);
            stages.push_back(Stage {
                skill: s,
                charge: a,
                token: 0,
                ready: sim.tick() + 6,
                started: sim.tick(),
                error: false,
            });
        }
        if !self.pool.reserve(reserved) {
            return;
        }
        self.next_id += 1;
        let st = sim.get_entity(m.id).map(|e| e.stat()).unwrap_or_default();
        let nominal: usize = stages
            .iter()
            .filter(|s| offense(s.skill))
            .map(|s| {
                let stat = if physical(s.skill) {
                    st.attack
                } else {
                    st.magic_power
                };
                let n = if heavy(s.skill) {
                    90 + stat * 45 / 100
                } else {
                    50 + stat / 4
                };
                n * strength(SKILLS[s.skill].charge, &s.charge) as usize / 100
            })
            .sum();
        let cap = 160 + st.attack.max(st.magic_power) * 80 / 100;
        self.budgets.insert(self.next_id, nominal.min(cap));
        self.plan = Some(Experiment {
            id: self.next_id,
            target: target.id,
            stages,
            power: st.magic_power,
            physical: st.attack,
        });
    }
    fn emit(
        &mut self,
        sim: &mut StableSim<'_>,
        m: &Champ,
        target: usize,
        amount: usize,
        physical: bool,
        exp: u64,
        skill: usize,
    ) {
        let Some((alive, hp, max)) = sim
            .get_entity(target)
            .map(|e| (e.is_alive(), e.hp().0, e.hp().1))
        else {
            return;
        };
        if !alive {
            return;
        }
        let budget = self.budgets.entry(exp).or_default();
        let n = amount.min(*budget);
        let n = self
            .damage
            .entry(target)
            .or_default()
            .allow(sim.tick(), 120, max * 35 / 100, n);
        if n == 0 {
            return;
        }
        *budget -= n;
        self.last_offense = sim.tick();
        sim.deal_damage(
            m.id,
            target,
            if physical { n } else { 0 },
            if physical { 0 } else { n },
            AttackTypeV1::Skill,
        );
        let after = sim.get_entity(target).map_or(hp, |e| e.hp().0);
        self.log(format!("seed {} instance {} tick {} caster {} target {} skill {} experiment {exp} raw {n} type {} HP {hp}->{after} budget {}",sim.seed(),self.key,sim.tick(),m.id,target,SKILLS[skill].id,if physical{"physical"}else{"magic"},self.budgets[&exp]));
    }
    fn shield(&mut self, sim: &mut StableSim<'_>, m: &Champ, n: usize) {
        let n = self.support.allow(sim.tick(), 180, m.max_hp / 4, n);
        if n > 0 {
            sim.entity_add_shield(m.id, n, 180);
            self.vfxu(sim, &format!("{ID}_shield"), m.id, m.id, 18);
        }
    }
    fn cc(&mut self, sim: &mut StableSim<'_>, id: usize, ticks: usize) {
        if sim.get_entity(id).is_some_and(|e| {
            (0..e.buff_count())
                .filter_map(|i| e.buff_at(i))
                .any(|b| b.cc_immune)
        }) {
            return;
        }
        let n = self
            .controls
            .entry(id)
            .or_default()
            .allow(sim.tick(), 180, 60, ticks);
        if n > 0 {
            sim.apply_cc(id, &CcV1::stun(n as u64));
        }
    }
    fn push(&mut self, sim: &mut StableSim<'_>, m: &Champ, id: usize, pull: bool) {
        if self
            .moves
            .entry(id)
            .or_default()
            .allow(sim.tick(), 120, 2, 1)
            > 0
        {
            let n = self
                .controls
                .entry(id)
                .or_default()
                .allow(sim.tick(), 180, 60, 12);
            if n == 0 {
                return;
            }
            if pull {
                sim.entity_pull(m.id, id, 1000, n);
            } else {
                sim.entity_knockback(m.id, id, 1000, n);
            }
        }
    }
    fn buff(
        &self,
        sim: &mut StableSim<'_>,
        id: usize,
        tag: &str,
        ticks: usize,
        f: impl FnOnce(&mut BuffV1),
    ) {
        let name = format!("ut{}_{}", self.owner, tag);
        sim.entity_remove_buff(id, &name);
        let mut b = timed(&name, ticks.min(360));
        f(&mut b);
        sim.add_buff(id, &b);
    }
    fn field(
        &mut self,
        sim: &mut StableSim<'_>,
        m: &Champ,
        s: usize,
        x: i64,
        y: i64,
        exp: u64,
        n: usize,
    ) {
        let limit = match s {
            13 | 57 => 2,
            _ => 1,
        };
        if self.fields.iter().filter(|f| f.kind == s).count() >= limit {
            if let Some(i) = self.fields.iter().position(|f| f.kind == s) {
                let old = self.fields.remove(i);
                if let Some(id) = old.entity {
                    sim.entity_set_hp(id, 0);
                }
            }
        }
        if matches!(s, 58 | 65 | 66)
            && self
                .fields
                .iter()
                .filter(|f| matches!(f.kind, 58 | 65 | 66))
                .count()
                >= 2
        {
            if let Some(i) = self
                .fields
                .iter()
                .position(|f| matches!(f.kind, 58 | 65 | 66))
            {
                let old = self.fields.remove(i);
                if let Some(id) = old.entity {
                    sim.entity_set_hp(id, 0);
                }
            }
        }
        if self.fields.len() >= 12 {
            return;
        }
        let entity = if s == 57 {
            sim.spawn_unit(
                &format!("{ID}_crystal"),
                m.id,
                m.team,
                x.max(0) as u64,
                y.max(0) as u64,
                360,
                &StatV1 {
                    hp: 120,
                    move_speed: 0,
                    ..StatV1::default()
                },
                &UnitAttackV1 {
                    attack: 0,
                    attack_ratio: 0,
                    ..UnitAttackV1::default()
                },
            )
        } else {
            None
        };
        self.fields.push(Field {
            kind: s,
            x,
            y,
            expires: sim.tick() + 360,
            experiment: exp,
            payload: n,
            entity,
        });
    }
    fn packet(
        &mut self,
        sim: &StableSim<'_>,
        m: &Champ,
        target: &Champ,
        s: usize,
        exp: u64,
        n: usize,
        physical: bool,
    ) {
        let t = &self.theory;
        let aim = t.aim.unwrap_or((target.x, target.y));
        let (dx, dy) = unit(aim.0 - m.x, aim.1 - m.y);
        let split = if s == 72 {
            3
        } else {
            t.scalar.max(t.series).clamp(1, 4)
        };
        let available = 8usize.saturating_sub(self.packets.len());
        let split = split.min(available);
        if split == 0 {
            return;
        }
        for i in 0..split {
            let payload = n / split + usize::from(i < n % split);
            self.packets.push(Packet {
                skill: s,
                experiment: exp,
                x: m.x,
                y: m.y,
                dx,
                dy,
                payload,
                physical,
                class: if physical {
                    2
                } else if matches!(s, 54 | 59 | 63) {
                    3
                } else if matches!(s, 71 | 72 | 74) {
                    1
                } else {
                    0
                },
                expires: sim.tick() + 180,
                release: sim.tick() + t.sync + t.order * i,
                age: 0,
                split,
                curve: if s == 40 { 250 } else { t.curve },
                orbit: t.orbit,
                hits: vec![],
            });
        }
    }
    fn execute(
        &mut self,
        sim: &mut StableSim<'_>,
        m: &Champ,
        target: &Champ,
        p: &Experiment,
        s: usize,
        strength: u32,
        instability: u32,
    ) -> bool {
        if !self.valid(s) || material_cost(s) > self.material {
            return false;
        }
        self.material -= material_cost(s);
        let pending_manipulation = p.stages.iter().any(|s| manipulates(s.skill))
            || self.goal.is_some_and(|g| {
                if g < 75 {
                    manipulates(g)
                } else {
                    COMBOS[g - 75].1.iter().any(|s| manipulates(*s))
                }
            });
        if pending_manipulation && matches!(s, 0 | 16 | 40 | 54 | 59 | 63 | 72 | 73 | 74) {
            self.theory.sync = self.theory.sync.max(12);
        }

        self.persona = SKILLS[s].science;
        self.theory.target = Some(target.id);
        self.theory.until = sim.tick() + 1200; // Committed calculations persist 20s; unfinished notebook stages still expire at 8s.
        let stat = if physical(s) { p.physical } else { p.power };
        let base = if heavy(s) {
            90 + stat * 45 / 100
        } else {
            50 + stat / 4
        };
        // Strength is applied once; no effect, packet or child may multiply it again.
        let n = base * strength as usize / 100;
        let n = if physical(s) {
            let work = self.theory.work.min(20);
            self.theory.work -= work;
            n + work
        } else {
            n
        }; // Still constrained by the experiment budget.
        let (dx, dy) = unit(target.x - m.x, target.y - m.y);
        if instability > 20 {
            let drift = if instability > 40 { 2 } else { 1 };
            let divisor = if instability > 40 { 1 } else { 2 };
            let aim = self.theory.aim.unwrap_or((target.x, target.y));
            self.theory.aim = Some((
                (aim.0 - dy * drift / divisor).clamp(0, 960000),
                (aim.1 + dx * drift / divisor).clamp(0, 960000),
            ));
        }
        match s {
            0 => self.packet(sim, m, target, s, p.id, n, false),
            1 => {
                if self.anchors.len() >= 3 {
                    self.anchors.remove(0);
                }
                self.anchors.push(Anchor {
                    x: m.x,
                    y: m.y,
                    expires: sim.tick() + 960,
                });
                self.theory.origin = Some((m.x, m.y));
            }
            2 => {
                self.theory.sync = 30;
                let at = sim.tick() + 30;
                for q in &mut self.packets {
                    q.release = at;
                    q.orbit = false;
                }
            }
            3 => {
                let v = self
                    .observations
                    .get(&target.id)
                    .map_or((0, 0), |(_, v)| *v);
                self.theory.velocity = (v.0 - self.motion.0, v.1 - self.motion.1);
                self.theory.aim = Some(forecast(target, self.theory.velocity, (0, 0), 12, 0));
            }
            4 => self.theory.proper = true,
            5 => self.field(sim, m, s, target.x, target.y, p.id, 0),
            6 => self.buff(sim, m.id, "contract", 180, |b| b.radius_mult = -20),
            7 => {
                let a = self.anchors.last().unwrap();
                let (x, y) = walls::clip(
                    m.x,
                    m.y,
                    (a.x + dx).clamp(m.x - 20000, m.x + 20000),
                    (a.y + dy).clamp(m.y - 20000, m.y + 20000),
                );
                sim.entity_set_pos(m.id, x.max(0) as u64, y.max(0) as u64);
                self.last_pos = Some((x, y));
            }
            8 => {
                for q in &mut self.packets {
                    q.release = sim.tick() + 12;
                    q.orbit = false;
                }
                self.theory.sync = 12;
            }
            9 => {
                self.theory.cone = true;
                self.theory.aim = Some(forecast(target, self.theory.velocity, (0, 0), 18, 0));
            }
            10 => self.theory.link = true,
            11 => self.theory.curve = 180,
            12 => self.field(sim, m, s, target.x, target.y, p.id, 0),
            13 => self.field(
                sim,
                m,
                s,
                (m.x + target.x) / 2,
                (m.y + target.y) / 2,
                p.id,
                0,
            ),
            14 => {
                let incoming = (0..sim.projectile_count().min(256))
                    .filter_map(|i| sim.projectile_at(i))
                    .filter(|q| {
                        q.team != m.team
                            && !q.is_end
                            && d2(m.x, m.y, q.x as i64, q.y as i64) < sq(45000)
                    })
                    .count()
                    .min(2);
                self.shield(sim, m, (35 + p.power / 5 + incoming * 15).min(120));
                // Foreign projectile classes are unavailable: protective filtering only.
            }
            15 => self.theory.blue = true, // Removes packet range attrition, never amplifies damage.
            16 => {
                self.packet(sim, m, target, s, p.id, n, true);
                self.theory.pressure = true;
            }
            17 => {
                self.material -= 1;
                self.theory.exchange = true;
                self.shield(sim, m, 40 + p.power / 4);
            }
            18 => {
                self.theory.drag = 120;
                self.field(sim, m, s, m.x, m.y, p.id, 0);
            }
            19 => {
                let a = self.anchors.first().unwrap();
                let b = self.anchors.last().unwrap();
                let ids: Vec<_> = champions(sim)
                    .into_iter()
                    .filter(|e| {
                        e.team != m.team
                            && sim.is_visible(m.team, e.id)
                            && segment_near(a.x, a.y, b.x, b.y, e.x, e.y, 12000)
                    })
                    .map(|e| e.id)
                    .collect();
                let len = ids.len().max(1);
                for (i, id) in ids.into_iter().enumerate() {
                    self.emit(
                        sim,
                        m,
                        id,
                        n / len + usize::from(i < n % len),
                        false,
                        p.id,
                        s,
                    );
                }
            }
            20 => self.field(sim, m, s, target.x, target.y, p.id, 0),
            21 => {
                let approach = self.theory.aim.map_or(true, |(x, y)| {
                    d2(m.x, m.y, x, y) < d2(m.x, m.y, target.x, target.y)
                });
                if approach {
                    self.shield(sim, m, 60 + p.power / 3);
                }
            }
            22 => {
                self.field(sim, m, 12, m.x + dx, m.y + dy, p.id, 0);
                self.theory.impulse = 0;
            }
            23 => {
                let v = self.theory.velocity;
                let a = self.theory.acceleration;
                self.theory.aim = Some(forecast(target, v, a, 12, 36));
            }
            24 => {
                let count = self.anchors.len().min(3);
                self.theory.curve = 100 * count as i64;
                self.theory.cone = true;
                self.theory.sync = 6;
                self.theory.aim = Some((target.x, target.y));
            }
            25 => self.theory.origin = Some((m.x, m.y)),
            26 => {
                self.theory.vector = if m.hp * 5 < m.max_hp * 2 {
                    (-dx, -dy)
                } else {
                    (dx, dy)
                }
            }
            27 => {
                self.theory.scalar = 2;
                split_existing(&mut self.packets, 2);
            }
            28 => {
                let (x, y) = self.theory.vector;
                self.theory.vector = unit(x + dx, y + dy);
            }
            29 => {
                if d2(m.x, m.y, target.x, target.y) > sq(60000) {
                    return false;
                }
                let v = self.theory.vector;
                let alignment = ((v.0 * dx + v.1 * dy) / 10000).clamp(0, 10000) as usize;
                self.emit(sim, m, target.id, n * alignment / 10000, true, p.id, s);
            }
            30 => {
                let (x, y) = self.theory.vector;
                self.theory.vector = (-y, x);
                self.theory.curve = -250;
            }
            31 => self.buff(sim, m.id, "inertia", 180, |b| b.move_speed_mult = 10),
            32 => {
                self.theory.impulse = self.momentum.min(30);
                self.momentum = (self.momentum + 10).min(100);
                self.buff(sim, m.id, "force", 90, |b| b.move_speed_mult = 15);
            }
            33 => {
                self.push(sim, m, target.id, false);
                self.buff(sim, m.id, "recoil", 60, |b| b.move_speed_mult = -10);
            }
            34 => {
                if d2(m.x, m.y, target.x, target.y) > sq(30000) {
                    return false;
                }
                let usable = self.momentum.min(50);
                self.momentum -= usable;
                self.emit(sim, m, target.id, n * usable / 50, true, p.id, s);
            }
            35 => {
                let (dx, dy) = if m.hp * 5 < m.max_hp * 2 {
                    (-dx, -dy)
                } else if self.theory.vector != (0, 0) {
                    self.theory.vector
                } else {
                    (dx, dy)
                };
                let (x, y) = walls::clip(m.x, m.y, m.x + dx, m.y + dy);
                sim.entity_set_pos(m.id, x.max(0) as u64, y.max(0) as u64);
                self.last_pos = Some((x, y));
                self.momentum = self.momentum.saturating_sub(10);
            }
            36 => {
                self.momentum = self.momentum.saturating_sub(20);
                self.buff(sim, target.id, "inelastic", 90, |b| b.move_speed_mult = -15);
            }
            37 => {
                self.theory.elastic = true;
                for q in &mut self.packets {
                    q.dx = -q.dx;
                    q.dy = -q.dy;
                    q.hits.clear();
                }
            }
            38 => {
                for q in &mut self.packets {
                    q.orbit = true;
                }
                self.theory.orbit = true;
            }
            39 => {
                for q in &mut self.packets {
                    q.orbit = false;
                    q.release = sim.tick();
                    let (u, v) = unit(q.x - m.x, q.y - m.y);
                    q.dx = -v;
                    q.dy = u;
                }
                self.theory.orbit = false;
                self.packet(sim, m, target, s, p.id, n, true);
            }
            40 => self.packet(sim, m, target, s, p.id, n, true),
            41 => {
                let incoming = (0..sim.projectile_count().min(256))
                    .filter_map(|i| sim.projectile_at(i))
                    .any(|q| {
                        q.team != m.team
                            && !q.is_end
                            && d2(m.x, m.y, q.x as i64, q.y as i64) <= sq(45000)
                    });
                if incoming {
                    self.shield(sim, m, 75 + p.physical / 4);
                }
            }
            42 => {
                if let Some((_, v)) = self.observations.get(&target.id) {
                    self.theory.acceleration =
                        (v.0 - self.theory.velocity.0, v.1 - self.theory.velocity.1);
                    self.theory.velocity = *v;
                }
                self.theory.aim = Some(forecast(target, self.theory.velocity, (0, 0), 10, 0));
            }
            43 => {
                self.theory.work = (self.theory.work + self.momentum / 4).min(40);
                self.momentum = self.momentum.saturating_sub(10);
            }
            44 => {
                let a = self.theory.aim.unwrap();
                self.theory.aim = Some(((a.0 + target.x) / 2, (a.1 + target.y) / 2));
                self.theory.curve /= 2;
            }
            45 => {
                self.theory.root = true;
                self.field(sim, m, s, target.x, target.y, p.id, 0);
            }
            46 => {
                self.theory.series = 3;
                split_existing(&mut self.packets, 3);
            }
            47 => self.theory.boundary = true,
            48 => {
                let v = self.theory.velocity;
                let a = self.theory.acceleration;
                self.theory.aim = Some(forecast(target, v, a, 15, 60));
                self.theory.cone = true;
            }
            49 => {
                self.theory.vector = (dx, dy);
                self.theory.impulse = 20;
                self.theory.work = 20;
                self.buff(sim, m.id, "principia", 180, |b| b.move_speed_mult = 10);
            }
            50 => {
                self.theory.sample = true;
                self.theory.acid = target.buffs.iter().any(|b| b.name().ends_with("_acid"));
            }
            51 => {
                if self.material > 0 {
                    self.material -= 1;
                    self.theory.purity = 100;
                }
            }
            52 => {
                let dose = target.max_hp.saturating_sub(target.hp).min(100);
                self.shield(sim, m, 35 + dose / 3);
            }
            53 => self.buff(sim, m.id, "buffer", 180, |b| {
                b.skill_damaged_reduce = 10;
                b.toughness = 15;
            }),
            54 => {
                self.theory.acid = true;
                self.buff(sim, target.id, "acid", 180, |b| b.defence_mult = -8);
                self.packet(sim, m, target, s, p.id, n, false);
            }
            55 => {
                self.theory.base = true;
                clear_chemical(sim, m.id, "acid");
                self.shield(sim, m, 30 + p.power / 6);
            }
            56 => {
                self.theory.acid = false;
                self.theory.base = false;
                clear_chemical(sim, target.id, "acid");
                sim.heal(m.id, m.id, (25 + p.power / 8).min(80));
            }
            57 => self.field(sim, m, s, m.x + dx * 2, m.y + dy * 2, p.id, 0),
            58 => self.field(sim, m, s, target.x, target.y, p.id, 0),
            59 => {
                self.buff(sim, target.id, "oxidize", 180, |b| b.heal_reduce = 15);
                self.packet(sim, m, target, s, p.id, n, false);
            }
            60 => {
                clear_chemical(sim, m.id, "oxidize");
                self.buff(sim, m.id, "reduce", 180, |b| b.magic_resistance_mult = 10);
            }
            61 => self.theory.catalyst = true,
            62 => {
                self.fields.retain(|f| !matches!(f.kind, 58 | 65 | 66));
                self.material = (self.material + 1).min(12); // valid() required an owned field; removing it consumes its source exactly once.
            }
            63 => {
                let ratio = if self.theory.acid && self.theory.base {
                    100
                } else {
                    70
                };
                self.packet(sim, m, target, s, p.id, n * ratio / 100, false);
            }
            64 => self.theory.order = 20,
            65 => self.field(sim, m, s, target.x, target.y, p.id, n),
            66 => self.field(sim, m, s, target.x, target.y, p.id, 0),
            67 => {
                self.shield(sim, m, 80 + p.power / 4);
                self.buff(sim, m.id, "membrane", 120, |b| b.skill_damaged_reduce = 5);
            }
            68 => {
                self.buff(sim, target.id, "tracer", 360, |_| {});
                self.theory.aim = Some((target.x, target.y));
            }
            69 => self.theory.label = true,
            70 => self.theory.half = 3,
            71 => {
                if d2(m.x, m.y, target.x, target.y) <= sq(30000) {
                    self.emit(sim, m, target.id, n, false, p.id, s);
                }
            }
            72 => self.packet(sim, m, target, s, p.id, n, false),
            73 => self.packet(sim, m, target, s, p.id, n, false),
            74 => {
                let available = 8 - self.packets.len();
                let parts = self.theory.half.min(available);
                let old_scalar = self.theory.scalar;
                let old_series = self.theory.series;
                self.theory.scalar = 1;
                self.theory.series = 1;
                for i in 0..parts {
                    self.packet(
                        sim,
                        m,
                        target,
                        s,
                        p.id,
                        n / parts + usize::from(i < n % parts),
                        false,
                    );
                    if let Some(q) = self.packets.last_mut() {
                        q.release = sim.tick() + i * 30;
                    }
                }
                self.theory.scalar = old_scalar;
                self.theory.series = old_series;
            }
            _ => unreachable!("catalogue bounds"),
        }
        self.vfxu(sim, &format!("{ID}_cast{}", self.persona), m.id, m.id, 15);
        true
    }
    fn step_notebook(&mut self, sim: &mut StableSim<'_>, m: &Champ, enemies: &[Champ]) {
        let Some(mut p) = self.plan.take() else {
            return;
        };
        let Some(target) = enemies.iter().find(|e| {
            e.id == p.target
                && d2(m.x, m.y, e.x, e.y) <= sq(120000)
                && sim.get_entity(e.id).is_some_and(|e| e.is_alive())
                && sim.is_visible(m.team, e.id)
        }) else {
            self.plan = Some(p);
            self.cancel();
            return;
        };
        // Reserve an owned output for an already-funded manipulation, without extending world expiry.
        if p.stages.iter().any(|s| manipulates(s.skill)) {
            let ready = p.stages.front().map_or(sim.tick(), |s| s.ready) + 12;
            for q in &mut self.packets {
                if !q.orbit {
                    q.release = q.release.max(ready);
                }
            }
        }
        let Some(st) = p.stages.front_mut() else {
            return;
        };
        if sim.tick().saturating_sub(st.started) > 480 {
            self.plan = Some(p);
            self.cancel();
            return;
        }
        self.persona = SKILLS[st.skill].science;
        if sim.tick() < st.ready || m.stunned {
            self.plan = Some(p);
            return;
        }
        if st.token < SKILLS[st.skill].tokens.len() {
            if self.random(10000) < [1200, 800, 600, 400, 200, 100, 30, 10][self.rank] {
                st.error = true;
            }
            st.token += 1;
            st.ready = sim.tick()
                + if st.token == SKILLS[st.skill].tokens.len() {
                    12
                } else if self.theory.catalyst && SKILLS[st.skill].science == 2 {
                    8
                } else {
                    10
                };
            self.plan = Some(p);
            return;
        }
        let st = p.stages.pop_front().unwrap();
        let s = st.skill;
        self.pool.commit(cost(&st.charge));
        self.cooldown[s] = sim.tick() + SKILLS[s].cooldown;
        let error = st.error && self.random(100) >= [15, 25, 40, 55, 70, 85, 96, 100][self.rank];
        let instability = instability(SKILLS[s].charge, &st.charge).unwrap_or(100);
        let instability = if SKILLS[s].science == 2 && self.theory.purity == 100 {
            self.theory.purity = 0;
            instability.saturating_sub(10)
        } else {
            instability
        };
        let accepted = !error
            && instability <= 70
            && self.execute(
                sim,
                m,
                target,
                &p,
                s,
                strength(SKILLS[s].charge, &st.charge),
                instability,
            );
        self.casts += usize::from(accepted);
        self.failures += usize::from(!accepted);
        self.log(format!("seed {} tick {} caster {} {} ({}) charge {:?} instability {instability} accepted {accepted} CU {} reserved {} material {} momentum {}",sim.seed(),sim.tick(),m.id,SKILLS[s].id,SKILLS[s].name,st.charge,self.pool.free,self.pool.reserved,self.material,self.momentum));
        if !accepted {
            self.vfxu(sim, &format!("{ID}_fizzle"), m.id, m.id, 15);
            self.plan = Some(p);
            self.cancel();
            return;
        }
        if let Some(next) = p.stages.front_mut() {
            next.started = sim.tick();
            next.ready = sim.tick() + 6;
            self.plan = Some(p);
        }
    }
    fn step_world(&mut self, sim: &mut StableSim<'_>, m: &Champ, enemies: &[Champ]) {
        let tick = sim.tick();
        self.anchors.retain(|a| a.expires > tick);
        self.fields.retain(|f| {
            f.expires > tick
                && f.entity
                    .is_none_or(|id| sim.get_entity(id).is_some_and(|e| e.is_alive()))
        });
        if self.theory.until < tick {
            self.theory = Theory {
                ..Theory::default()
            };
        }
        let mut packets = std::mem::take(&mut self.packets);
        for q in &mut packets {
            if q.expires <= tick || q.release > tick {
                continue;
            }
            if q.orbit {
                let (x, y) = unit(q.x - m.x, q.y - m.y);
                let (x, y) = if x == 0 && y == 0 { (10000, 0) } else { (x, y) };
                q.x = m.x + x * 2 - y / 5;
                q.y = m.y + y * 2 + x / 5;
                if tick.is_multiple_of(12) {
                    self.vfxp(sim, &format!("{ID}_packet{}", q.class), m.id, q.x, q.y, 15);
                }
                continue;
            }
            let slow = self
                .fields
                .iter()
                .any(|f| f.kind == 5 && d2(q.x, q.y, f.x, f.y) <= sq(35000));
            let speed = if slow {
                1800
            } else if self.theory.proper {
                4000
            } else {
                3600
            };
            let old = (q.x, q.y);
            let turn = q.curve
                + self
                    .fields
                    .iter()
                    .filter(|f| f.kind == 18 && d2(q.x, q.y, f.x, f.y) < sq(40000))
                    .count() as i64
                    * 120;
            if turn != 0 {
                let (u, v) = unit(q.dx - q.dy * turn / 10000, q.dy + q.dx * turn / 10000);
                q.dx = u;
                q.dy = v;
            }
            if q.class == 0 {
                for f in self.fields.iter().filter(|f| f.kind == 13) {
                    if d2(q.x, q.y, f.x, f.y) < sq(18000) {
                        let (u, v) = unit(q.dx + (f.x - q.x) / 4, q.dy + (f.y - q.y) / 4);
                        q.dx = u;
                        q.dy = v;
                    }
                }
            }
            let desired = (q.x + q.dx * speed / 10000, q.y + q.dy * speed / 10000);
            let clipped = walls::clip(q.x, q.y, desired.0, desired.1);
            let surface = self.fields.iter().any(|f| {
                f.kind == 57 && segment_near(q.x, q.y, desired.0, desired.1, f.x, f.y, 8000)
            });
            if clipped != desired || (surface && self.theory.boundary) {
                if self.theory.elastic || self.theory.boundary {
                    q.dx = -q.dx;
                    q.dy = -q.dy;
                } else {
                    q.expires = tick;
                    continue;
                }
            }
            q.x = clipped.0;
            q.y = clipped.1;
            q.age += 1;
            if !self.theory.blue && q.age.is_multiple_of(30) {
                q.payload = q.payload * 95 / 100;
            }
            let mut hit: Vec<_> = enemies
                .iter()
                .filter(|e| {
                    !q.hits.contains(&e.id) && segment_near(old.0, old.1, q.x, q.y, e.x, e.y, 10000)
                })
                .map(|e| e.id)
                .collect();
            hit.sort_unstable();
            // Piercing gamma distributes its remaining original payload; it does not copy it per target.
            let count = if q.skill == 73 { hit.len().max(1) } else { 1 };
            for id in hit.into_iter().take(count) {
                let n = q.payload / count;
                self.emit(sim, m, id, n, q.physical, q.experiment, q.skill);
                q.hits.push(id);
                q.payload = q.payload.saturating_sub(n);
                if self.theory.pressure {
                    self.push(sim, m, id, false);
                }
                if self.theory.link {
                    self.theory.link = false;
                    self.cc(sim, id, 12);
                }
                if q.skill != 73 {
                    q.expires = tick;
                    break;
                }
            }
            if q.payload == 0 {
                q.expires = tick;
            }
            if tick.is_multiple_of(12) {
                self.vfxp(sim, &format!("{ID}_packet{}", q.class), m.id, q.x, q.y, 15);
            }
        }
        self.packets = packets
            .into_iter()
            .filter(|q| q.expires > tick)
            .take(8)
            .collect();
        if tick.is_multiple_of(12) {
            let anchors = self.anchors.clone();
            for a in anchors {
                self.vfxp(sim, &format!("{ID}_field1"), m.id, a.x, a.y, 96);
            }
            let fields = self.fields.clone();
            for (i, f) in fields.iter().enumerate() {
                if i % 3 == (tick / 12) % 3 {
                    self.vfxp(sim, &format!("{ID}_field{}", f.kind), m.id, f.x, f.y, 96);
                }
                for e in enemies
                    .iter()
                    .filter(|e| d2(e.x, e.y, f.x, f.y) <= sq(30000))
                {
                    match f.kind {
                        5 => self.buff(sim, e.id, "dilation", 24, |b| b.move_speed_mult = -12),
                        12 | 66 => {
                            if !e.buffs.iter().any(|b| b.cc_immune)
                                && self.moves.entry(e.id).or_default().allow(tick, 120, 2, 1) > 0
                            {
                                let duration = self
                                    .controls
                                    .entry(e.id)
                                    .or_default()
                                    .allow(tick, 180, 60, 10);
                                if duration == 0 {
                                    continue;
                                }
                                let (dx, dy) = unit(f.x - e.x, f.y - e.y);
                                let (x, y) = walls::clip(e.x, e.y, e.x + dx, e.y + dy);
                                sim.apply_cc(
                                    e.id,
                                    &mod_api_stable::CcV1 {
                                        kind: mod_api_stable::CcKindV1::ForceMove.code(),
                                        tick: duration as u64,
                                        dx: x - e.x,
                                        dy: y - e.y,
                                        speed: 1000,
                                        ..CcV1::default()
                                    },
                                );
                            }
                        }
                        20 => {
                            if d2(e.x, e.y, f.x, f.y) >= sq(24000) {
                                self.buff(sim, e.id, "horizon", 24, |b| b.move_speed_mult = -15);
                            }
                        }
                        45 => {
                            if self.theory.root {
                                self.cc(sim, e.id, 18);
                                self.theory.root = false;
                            }
                        }
                        58 => {
                            if self.theory.label || self.theory.sample {
                                self.cc(sim, e.id, 12);
                            }
                        }
                        65 => {
                            if f.payload > 0 {
                                self.emit(
                                    sim,
                                    m,
                                    e.id,
                                    (f.payload / 10).max(1),
                                    false,
                                    f.experiment,
                                    65,
                                );
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        // Drop exhausted or unreachable experiments; bounded by current plan, packets and fields.
        self.budgets.retain(|id, _| {
            self.plan.as_ref().is_some_and(|p| p.id == *id)
                || self.packets.iter().any(|q| q.experiment == *id)
                || self.fields.iter().any(|f| f.experiment == *id)
        });
    }
    fn show(&mut self, sim: &mut StableSim<'_>, m: &Champ) {
        if self.shown != self.persona {
            for s in 0..3 {
                sim.entity_remove_buff(m.id, &format!("ut_persona{s}"));
            }
            sim.add_buff(
                m.id,
                &timed(&format!("ut_persona{}", self.persona), 5184000),
            );
            for r in 0..8 {
                sim.entity_remove_buff(m.id, &format!("ut_rank{r}"));
            }
            sim.add_buff(m.id, &timed(&format!("ut_rank{}", self.rank), 5184000));
            self.shown = self.persona;
        }
        if !sim.tick().is_multiple_of(8) {
            return;
        }
        let mut rows = [None, None, None];
        if let Some(p) = &self.plan {
            for s in &p.stages {
                if rows[SKILLS[s.skill].science].is_none() {
                    rows[SKILLS[s.skill].science] = Some(s);
                }
            }
        }
        let rows: Vec<_> = rows
            .iter()
            .enumerate()
            .map(|(i, s)| (i, s.map(|s| (s.skill, s.token))))
            .collect();
        for (i, s) in rows.iter() {
            let tag = s.map_or_else(
                || format!("note_blank{i}"),
                |s| format!("note_{}_{}", SKILLS[s.0].id, s.1),
            );
            self.vfxp(
                sim,
                &format!("{ID}_{tag}"),
                m.id,
                m.x,
                m.y - 72000 - *i as i64 * 25000,
                11,
            );
        }
        if let Some(st) = self.plan.as_ref().and_then(|p| p.stages.front()) {
            let a = st.charge.iter().sum::<u32>();
            let score = instability(SKILLS[st.skill].charge, &st.charge).unwrap_or(100);
            let class = if score <= 20 {
                0
            } else if score <= 40 {
                1
            } else if score <= 70 {
                2
            } else {
                3
            };
            self.vfxp(
                sim,
                &format!("{ID}_allocation_{}_{}", a.min(100), class),
                m.id,
                m.x,
                m.y - 174000,
                11,
            );
        }
        self.vfxp(
            sim,
            &format!("{ID}_meter{}", self.pool.free / 1000),
            m.id,
            m.x,
            m.y - 150000,
            11,
        );
    }
}
fn compatible_chemical(name: &str, tag: &str) -> bool {
    name.strip_prefix("ut")
        .and_then(|n| n.strip_suffix(&format!("_{tag}")))
        .is_some_and(|owner| !owner.is_empty() && owner.bytes().all(|b| b.is_ascii_digit()))
}
fn clear_chemical(sim: &mut StableSim<'_>, target: usize, tag: &str) {
    let names = sim
        .get_entity(target)
        .map(|e| {
            (0..e.buff_count())
                .filter_map(|i| e.buff_at(i))
                .filter(|b| compatible_chemical(b.name(), tag))
                .map(|b| b.name().to_owned())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    for name in names {
        sim.entity_remove_buff(target, &name);
    }
}
fn append_trace(path: &std::path::Path, lines: &[String], limit: u64) -> std::io::Result<()> {
    if std::fs::metadata(path).is_ok_and(|m| m.len() >= limit) {
        let previous = path.with_extension("previous.txt");
        if previous.exists() {
            std::fs::remove_file(&previous)?;
        }
        std::fs::rename(path, previous)?;
    }
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    writeln!(f, "{}", lines.join("\n"))
}
fn forecast(target: &Champ, v: (i64, i64), a: (i64, i64), h: i64, k: i64) -> (i64, i64) {
    (
        (target.x + (v.0 * h + a.0 * k).clamp(-30000, 30000)).clamp(0, 960000),
        (target.y + (v.1 * h + a.1 * k).clamp(-30000, 30000)).clamp(0, 960000),
    )
}
fn segment_near(ax: i64, ay: i64, bx: i64, by: i64, x: i64, y: i64, r: i64) -> bool {
    let (dx, dy) = ((bx - ax) as i128, (by - ay) as i128);
    let len = dx * dx + dy * dy;
    if len == 0 {
        return d2(ax, ay, x, y) <= sq(r);
    }
    let dot = (((x - ax) as i128) * dx + ((y - ay) as i128) * dy).clamp(0, len);
    d2(
        x,
        y,
        ax + (dx * dot / len) as i64,
        ay + (dy * dot / len) as i64,
    ) <= sq(r)
}
fn split_existing(packets: &mut Vec<Packet>, parts: usize) {
    let originals = std::mem::take(packets);
    for q in originals {
        let n = parts.min(8 - packets.len()).max(1);
        for i in 0..n {
            let mut child = q.clone();
            child.payload = q.payload / n + usize::from(i < q.payload % n);
            child.curve += (i as i64 - (n / 2) as i64) * 100;
            child.split = n;
            packets.push(child);
            if packets.len() == 8 {
                break;
            }
        }
        if packets.len() == 8 {
            break;
        }
    }
}
impl StablePassive for UnifiedTheory {
    fn clone_box(&self) -> Box<dyn StablePassive> {
        Box::new(self.clone())
    }
    fn on_dead(&mut self, sim: &mut StableSim<'_>, player: usize) {
        self.cancel();
        self.packets.clear();
        for f in &self.fields {
            if let Some(id) = f.entity {
                sim.entity_set_hp(id, 0);
            }
        }
        self.fields.clear();
        self.anchors.clear();
        self.budgets.clear();
        self.theory = Theory::default();
        self.goal = None;
        self.momentum = 0;
        self.last_pos = None;
        self.shown = usize::MAX;
        if let Some(id) = sim
            .get_player(player)
            .and_then(|p| p.champion())
            .map(|e| e.id())
        {
            for i in 0..3 {
                sim.entity_remove_buff(id, &format!("ut_persona{i}"));
            }
            for i in 0..8 {
                sim.entity_remove_buff(id, &format!("ut_rank{i}"));
            }
        }
        self.flush(sim, self.owner, true);
    }
    fn on_kill(&mut self, sim: &mut StableSim<'_>, _: usize, entity: usize, victim: usize) {
        self.log(format!("seed {} tick {} caster {entity} engine kill credit victim {victim}; associated with engine pipeline, not inferred from a last packet",sim.seed(),sim.tick()));
    }
    fn on_update(&mut self, sim: &mut StableSim<'_>, _: u64, player: usize, entity: usize) {
        let key = crate::match_key(sim);
        self.key = key;
        self.owner = entity;
        let tick = sim.tick();
        self.vfx_left = 6;
        if self.last_tick == Some(tick) {
            return;
        }
        self.last_tick = Some(tick);
        if !self.started {
            self.started = true;
            self.rng =
                (sim.seed() ^ (entity as u64).wrapping_mul(0x9e3779b97f4a7c15) ^ 0x534349454e4345)
                    .max(1);
        }
        if !self.rank_set && tick >= 60 {
            self.athlete = BOOK.athlete_of(sim.seed(), player);
            self.rank = BOOK.pinned(sim.seed()).rank_for(self.athlete).0.min(7);
            self.rank_set = true;
            self.shown = usize::MAX;
        }
        let all = champions(sim);
        let Some(m) = all.iter().find(|c| c.id == entity).cloned() else {
            return;
        };
        let mut enemies: Vec<_> = all
            .iter()
            .filter(|e| e.team != m.team && sim.is_visible(m.team, e.id))
            .cloned()
            .collect();
        let mut wave = crate::units(sim);
        wave.sort_by_key(|u| (d2(m.x, m.y, u.2, u.3), u.0));
        for (id, team, x, y) in wave
            .into_iter()
            .filter(|u| u.1 != m.team && d2(m.x, m.y, u.2, u.3) <= sq(100000))
            .take(32)
        {
            if !sim.is_visible(m.team, id) {
                continue;
            }
            if let Some(e) = sim.get_entity(id) {
                let (hp, max_hp) = e.hp();
                enemies.push(Champ {
                    id,
                    team,
                    x,
                    y,
                    hp,
                    max_hp,
                    attack: e.stat().attack,
                    name: String::new(),
                    buffs: vec![],
                    stunned: false,
                    pushed: false,
                });
            }
        }
        enemies.sort_by_key(|e| e.id);
        if self.sig.is_none() && tick >= 1800 {
            self.sig = Some(signature(&BOOK, sim, &all, self.athlete));
        }
        if let (Some(sig), Some(a)) = (&self.sig, self.athlete) {
            if tick.is_multiple_of(30) {
                self.record.track(&BOOK, sim, sig, m.team, a, tick);
            }
        }
        if tick.is_multiple_of(3600) {
            let sig = self.sig.as_deref().unwrap_or("warmup");
            if BOOK.first(format!("ut-summary:{}:{entity}:{tick}", sim.seed())) {
                BOOK.log(&format!("game {sig} native {} minute {} athlete {:?} rank {} ({}) casts {} failures {} CU {} reserved {} material {} momentum {} packets {} fields {}",crate::VERSION,tick/3600,self.athlete,self.rank,RANKS[self.rank],self.casts,self.failures,self.pool.free,self.pool.reserved,self.material,self.momentum,self.packets.len(),self.fields.len()));
            }
        }
        self.pool.regen(
            tick.saturating_sub(self.last_offense) < 300
                || enemies.iter().any(|e| d2(m.x, m.y, e.x, e.y) < sq(100000)),
        );
        if tick.is_multiple_of(180) {
            self.material = (self.material + 1).min(12);
        }
        if let Some((x, y)) = self.last_pos {
            self.motion = (m.x - x, m.y - y);
            let dist = d2(x, y, m.x, m.y);
            if dist > sq(200) && dist <= sq(3000) {
                self.momentum = (self.momentum + 1).min(100);
            } else if tick.is_multiple_of(30) {
                self.momentum = self.momentum.saturating_sub(1);
            }
        }
        self.last_pos = Some((m.x, m.y));
        if tick.is_multiple_of(6) {
            for e in &enemies {
                let old = self.observations.get(&e.id).map_or((e.x, e.y), |(p, _)| *p);
                self.observations
                    .insert(e.id, ((e.x, e.y), ((e.x - old.0) / 6, (e.y - old.1) / 6)));
            }
            self.observations
                .retain(|id, _| enemies.iter().any(|e| e.id == *id));
        }
        self.show(sim, &m);
        self.step_world(sim, &m, &enemies);
        if self.plan.is_none() && tick.is_multiple_of(6) && !m.stunned {
            self.pick(sim, &m, &enemies);
        }
        self.step_notebook(sim, &m, &enemies);
        self.flush(sim, entity, false);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn catalogue_integrity() {
        assert_eq!(SKILLS.len(), 75);
        assert_eq!(COMBOS.len(), 50);
        for s in SKILLS {
            assert_eq!(s.tokens.len(), s.charge.len());
            assert_eq!(instability(s.charge, s.charge), Some(0));
            assert!(!s.name.is_empty());
            assert!(!s.role.is_empty());
        }
        for (_, steps) in COMBOS {
            assert!(steps.len() <= 5);
            assert!(steps.iter().map(|s| cost(SKILLS[*s].charge)).sum::<u32>() <= 10000);
        }
    }
    #[test]
    fn cancel_refunds_only_uncommitted() {
        let mut u = UnifiedTheory::default();
        u.pool.reserve(3400);
        u.pool.commit(800);
        u.plan = Some(Experiment {
            id: 1,
            target: 2,
            power: 45,
            physical: 70,
            stages: VecDeque::from([
                Stage {
                    skill: 13,
                    charge: vec![4, 5, 3],
                    token: 1,
                    ready: 10,
                    started: 0,
                    error: false,
                },
                Stage {
                    skill: 73,
                    charge: vec![4, 6, 4],
                    token: 0,
                    ready: 10,
                    started: 0,
                    error: false,
                },
            ]),
        });
        u.cancel();
        assert_eq!(u.pool.free, 9200);
        assert_eq!(u.pool.reserved, 0);
    }
    #[test]
    fn collision_segment() {
        assert!(segment_near(0, 0, 100000, 0, 50000, 5000, 10000));
        assert!(!segment_near(0, 0, 100000, 0, 150000, 0, 10000));
    }
    #[test]
    fn children_conserve_payload() {
        let mut q = Vec::new();
        let u = UnifiedTheory::default();
        let _ = u;
        q.push(Packet {
            skill: 0,
            experiment: 1,
            x: 0,
            y: 0,
            dx: 10000,
            dy: 0,
            payload: 101,
            physical: false,
            class: 0,
            expires: 180,
            release: 0,
            age: 0,
            split: 1,
            curve: 0,
            orbit: false,
            hits: vec![],
        });
        split_existing(&mut q, 3);
        assert_eq!(q.len(), 3);
        assert_eq!(q.iter().map(|p| p.payload).sum::<usize>(), 101);
        assert!(q.iter().all(|p| p.experiment == 1));
    }
}
#[cfg(test)]
#[path = "unified_theory_tests.rs"]
mod integration;
