use super::*;
#[derive(Clone, Default)]
pub(super) struct Context {
    pub theory: Theory,
    pub anchors: Vec<Anchor>,
    pub target: usize,
    pub mask: u8,
    pub commits: usize,
    pub until: usize,
    pub parents: Vec<u64>,
    pub consumed: bool,
}
#[derive(Clone)]
pub(super) struct Overcharge {
    pub experiment: u64,
    pub slot: usize,
    pub committed: bool,
}
#[derive(Clone)]
pub(super) struct Grand {
    pub target: usize,
    pub x: i64,
    pub y: i64,
    pub at: usize,
    pub mask: u8,
    pub payloads: Vec<(u64, bool, usize)>,
}
impl UnifiedTheory {
    pub(super) fn field_compatible(&self, f: &Field) -> bool {
        self.contexts.is_empty()
            || self
                .active_context
                .is_some_and(|id| self.family(id).contains(&f.experiment))
    }
    pub(super) fn family(&self, id: u64) -> Vec<u64> {
        let mut ids = vec![id];
        if let Some(c) = self.contexts.get(&id) {
            ids.extend(c.parents.iter().copied());
        }
        ids.sort_unstable();
        ids.dedup();
        ids
    }
    pub(super) fn activate_context(&mut self, id: u64) {
        if let Some(c) = self.contexts.get(&id) {
            self.theory = c.theory.clone();
            self.anchors = c.anchors.clone();
            self.active_context = Some(id);
        }
    }
    pub(super) fn park_context(&mut self, id: u64) {
        if let Some(c) = self.contexts.get_mut(&id) {
            c.theory = self.theory.clone();
            c.anchors = self.anchors.clone();
        }
    }
    pub(super) fn new_context(&mut self, id: u64, target: usize, tick: usize) {
        if self.contexts.contains_key(&id) {
            return;
        }
        let parent = self.target_contexts.get(&target).copied().filter(|p| {
            self.contexts
                .get(p)
                .is_some_and(|c| !c.consumed && c.until > tick)
        });
        let mut parents = parent.map_or_else(Vec::new, |p| self.family(p));
        parents.truncate(8);
        let c = Context {
            theory: self.theory.clone(),
            anchors: self.anchors.clone(),
            target,
            mask: 0,
            commits: 0,
            until: tick + 1200,
            parents,
            consumed: false,
        };
        self.contexts.insert(id, c);
        self.target_contexts.insert(target, id);
        self.active_context = Some(id);
    }
    pub(super) fn execute(
        &mut self,
        sim: &mut StableSim<'_>,
        m: &Champ,
        target: &Champ,
        p: &Experiment,
        s: usize,
        power: u32,
        instability: u32,
    ) -> bool {
        self.new_context(p.id, target.id, sim.tick());
        self.activate_context(p.id);
        let family = self.family(p.id);
        // Packet transformations are scoped to the bound experiment and its explicit prerequisites.
        let mut foreign = Vec::new();
        let mut own = Vec::new();
        for q in std::mem::take(&mut self.packets) {
            if family.contains(&q.experiment) {
                own.push(q)
            } else {
                foreign.push(q)
            }
        }
        self.packets = own;
        let old_limit = self.delivery_limit;
        self.delivery_limit = OWNED_DELIVERY_LIMIT.saturating_sub(foreign.len());
        let accepted = self.execute_inner(sim, m, target, p, s, power, instability);
        self.delivery_limit = old_limit;
        self.packets.extend(foreign);
        if accepted {
            if let Some(c) = self.contexts.get_mut(&p.id) {
                c.mask |= 1 << SKILLS[s].science;
                c.commits += 1;
                c.until = sim.tick() + 1200;
            }
        }
        self.park_context(p.id);
        accepted
    }
    pub(super) fn begin_overcharge(&mut self, tick: usize) -> bool {
        let Some(id) = self.plan.as_ref().map(|p| p.id) else {
            return false;
        };
        if self.overcharge.is_some() {
            return false;
        }
        let Some(slot) = self.charges.reserve(tick) else {
            return false;
        };
        self.overcharge = Some(Overcharge {
            experiment: id,
            slot,
            committed: false,
        });
        if let Some(st) = self.plan.as_mut().and_then(|p| p.stages.front_mut()) {
            st.ready += OVERCHARGE_TICKS;
        }
        if let Some(p) = self.plan.as_ref() {
            let nominal: usize = p
                .stages
                .iter()
                .enumerate()
                .filter(|(_, s)| offense(s.skill))
                .map(|(i, s)| {
                    base_output(
                        s.skill,
                        if physical(s.skill) {
                            p.physical
                        } else {
                            p.power
                        },
                    ) * chain_pct(i)
                        / 100
                })
                .sum();
            let cap = (160 + p.power.max(p.physical) * 80 / 100)
                * chain_pct(p.stages.len().saturating_sub(1))
                / 100;
            self.budgets
                .insert(id, nominal.min(cap) * OVERCHARGE_PERCENT as usize / 100);
        }
        self.last_reason = "Overcharge: one extra charge, longer exposed preparation".into();
        true
    }
    pub(super) fn update_grand(
        &mut self,
        sim: &mut StableSim<'_>,
        m: &Champ,
        enemies: &[Champ],
        unlocked: bool,
    ) {
        let tick = sim.tick();
        if let Some(g) = self.grand.clone() {
            let name = format!(
                "utg:{}:{}:{}:{}",
                g.x,
                g.y,
                if g.mask == 7 { 32000 } else { 24000 },
                g.at
            );
            if m.stunned
                || !sim.is_visible(m.team, g.target)
                || !enemies.iter().any(|e| e.id == g.target && e.hp > 0)
            {
                self.grand = None;
                sim.entity_remove_buff(m.id, &name);
                self.last_reason =
                    "Grand Unification interrupted; committed payloads consumed".into();
                return;
            }
            if tick < g.at {
                return;
            }
            sim.entity_remove_buff(m.id, &name);
            let radius = if g.mask == 7 {
                GR_THREE_RADIUS
            } else {
                GR_TWO_RADIUS
            };
            let targets: Vec<_> = enemies
                .iter()
                .filter(|e| {
                    !self.farm_targets.contains(&e.id)
                        && e.hp > 0
                        && sim.is_visible(m.team, e.id)
                        && d2(g.x, g.y, e.x, e.y) <= sq(radius)
                })
                .map(|e| e.id)
                .collect();
            for (exp, physical, amount) in &g.payloads {
                for (i, id) in targets.iter().enumerate() {
                    self.emit(
                        sim,
                        m,
                        *id,
                        amount / targets.len() + usize::from(i < amount % targets.len()),
                        *physical,
                        *exp,
                        if *physical { 40 } else { 73 },
                    );
                }
            }
            if g.mask == 7 {
                for id in targets {
                    let n =
                        self.controls
                            .entry(id)
                            .or_default()
                            .allow(tick, 180, 60, GR_BIND_TICKS);
                    if n > 0 {
                        sim.apply_cc(id, &CcV1::of_kind(CcKindV1::Bind, n as u64));
                    }
                }
            }
            for (exp, _, _) in &g.payloads {
                self.budgets.remove(exp);
            }
            self.grand = None;
            self.resonances += 1;
            self.last_reason = "Grand Unification resolved; linked payloads consumed".into();
            self.queue_art(
                format!(
                    "{ID}_complete_unified_t{}_p{}",
                    art_tier(self.rank),
                    podium(self.rank, self.top_pos)
                ),
                48,
            );
            return;
        }
        if !unlocked || tick < self.grand_at || m.stunned {
            return;
        }
        let candidate = self.contexts.iter().rev().find_map(|(id, c)| {
            if c.consumed || c.until < tick {
                return None;
            }
            let family = self.family(*id);
            let mask = family
                .iter()
                .filter_map(|p| self.contexts.get(p))
                .fold(0, |mask, c| mask | c.mask);
            let funded = |id: &u64| self.budgets.get(id).copied().unwrap_or(0) > 0;
            let pending =
                self.packets.iter().any(|q| {
                    family.contains(&q.experiment) && q.payload > 0 && funded(&q.experiment)
                }) || self.deliveries.iter().any(|q| {
                    family.contains(&q.experiment) && q.payload > 0 && funded(&q.experiment)
                }) || self.fields.iter().any(|f| {
                    family.contains(&f.experiment) && f.payload > 0 && funded(&f.experiment)
                });
            if mask.count_ones() < 2 || !pending {
                return None;
            }
            enemies
                .iter()
                .find(|e| {
                    e.id == c.target
                        && !self.farm_targets.contains(&e.id)
                        && sim.is_visible(m.team, e.id)
                        && d2(m.x, m.y, e.x, e.y) <= sq(120_000)
                })
                .map(|e| (*id, e.id, e.x, e.y, mask))
        });
        let Some((experiment, target, x, y, mask)) = candidate else {
            return;
        };
        let family = self.family(experiment);
        let mut pools: BTreeMap<(u64, bool), usize> = BTreeMap::new();
        for q in &self.packets {
            if family.contains(&q.experiment) {
                *pools.entry((q.experiment, q.physical)).or_default() += q.payload;
            }
        }
        for q in &self.deliveries {
            if family.contains(&q.experiment) {
                *pools.entry((q.experiment, physical(q.skill))).or_default() += q.payload;
            }
        }
        for f in &self.fields {
            if family.contains(&f.experiment) {
                *pools.entry((f.experiment, physical(f.kind))).or_default() += f.payload;
            }
        }
        let totals = pools
            .iter()
            .fold(BTreeMap::<u64, usize>::new(), |mut sums, ((exp, _), n)| {
                *sums.entry(*exp).or_default() += n;
                sums
            });
        let payloads = pools
            .into_iter()
            .map(|((exp, physical), pool)| {
                let limit = self.budgets.get(&exp).copied().unwrap_or(0);
                let total = totals[&exp];
                (
                    exp,
                    physical,
                    if total > limit {
                        pool * limit / total
                    } else {
                        pool
                    },
                )
            })
            .filter(|(_, _, n)| *n > 0)
            .collect::<Vec<_>>();
        if payloads.is_empty() {
            return;
        }
        // Commitment atomically retires the sources, then warns before delivering the conserved pool.
        self.packets.retain(|q| !family.contains(&q.experiment));
        self.deliveries.retain(|q| !family.contains(&q.experiment));
        let mut keep = vec![];
        for f in std::mem::take(&mut self.fields) {
            if family.contains(&f.experiment) {
                if let Some(id) = f.entity {
                    if sim.get_entity(id).is_some_and(|e| e.is_alive()) {
                        sim.entity_set_hp(id, 0);
                    }
                }
            } else {
                keep.push(f)
            }
        }
        self.fields = keep;
        if self.plan.as_ref().is_some_and(|p| family.contains(&p.id)) {
            self.cancel();
        }
        for id in family {
            if let Some(c) = self.contexts.get_mut(&id) {
                c.consumed = true;
                c.theory = Theory::default();
                c.anchors.clear();
            }
        }
        self.grand_at = tick + GR_COOLDOWN_TICKS;
        self.grand = Some(Grand {
            target,
            x,
            y,
            at: tick + GR_WARNING_TICKS,
            mask,
            payloads,
        });
        sim.add_buff(
            m.id,
            &timed(
                &format!(
                    "utg:{x}:{y}:{}:{}",
                    if mask == 7 { 32000 } else { 24000 },
                    tick + GR_WARNING_TICKS
                ),
                36,
            ),
        );
        self.last_reason = "Grand Unification: 0.6-second escape warning".into();
        self.vfxp(
            sim,
            &format!("{ID}_field_E13_t{}", art_tier(self.rank)),
            m.id,
            x,
            y,
            36,
        );
    }
    pub(super) fn react_displacement(
        &mut self,
        sim: &mut StableSim<'_>,
        m: &Champ,
        enemies: &[Champ],
    ) {
        let tick = sim.tick();
        let pending = std::mem::take(&mut self.displacements);
        for (id, (exp, at, x, y)) in pending {
            let Some(target) = enemies
                .iter()
                .find(|e| e.id == id && sim.is_visible(m.team, id))
            else {
                continue;
            };
            if tick > at + 60 {
                continue;
            }
            if d2(x, y, target.x, target.y) < sq(100) {
                self.displacements.insert(id, (exp, at, x, y));
                continue;
            }
            if !self
                .contexts
                .get(&exp)
                .is_some_and(|c| c.target == id && c.theory.label)
            {
                continue;
            }
            let family = self.family(exp);
            let mut packets = std::mem::take(&mut self.packets);
            for q in &mut packets {
                if family.contains(&q.experiment)
                    && SKILLS[q.skill].science == 2
                    && q.release > tick
                    && q.payload > 0
                {
                    self.emit(sim, m, id, q.payload, q.physical, q.experiment, q.skill);
                    q.payload = 0;
                    q.expires = tick;
                }
            }
            self.packets = packets.into_iter().filter(|q| q.expires > tick).collect();
        }
    }
}
