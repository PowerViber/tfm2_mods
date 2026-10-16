// Bounded tactical search. The preview is the actual selected search path, never a second simulation.
pub fn appearance(rank: usize, top: Option<usize>) -> usize {
    if rank < 7 {
        rank
    } else {
        match top {
            Some(1) => 10,
            Some(2) => 9,
            Some(3) => 8,
            _ => 7,
        }
    }
}
impl Core {
    fn command_ticks(&self, action: Action, tick: usize) -> usize {
        let kind = match action {
            Action::Move { piece, .. } => self.pieces.get(&piece).map_or(Kind::Rook, |p|p.kind),
            Action::Summon { kind, .. } => kind,
            Action::Deploy { .. } => return 60,
            _ => Kind::Rook,
        };
        let ticks = if kind == Kind::Queen {45} else {30};
        if self.reward.as_ref().is_some_and(|r| tick < r.until && (r.id == 2 || r.id == 11 && kind == Kind::Pawn)) {ticks*4/5} else {ticks}
    }
    /// Constant observed motion is a hypothesis, never authoritative combat state.
    fn forecast(&self, input: &Input, elapsed: usize) -> Vec<Fighter> {
        input.fighters.iter().map(|f| {
            let mut next=f.clone();
            if f.visible && f.team != input.owner.team {
                let n=elapsed.min(120) as i64;
                let vx=f.vx.clamp(-f.move_speed.max(0),f.move_speed.max(0));
                let vy=f.vy.clamp(-f.move_speed.max(0),f.move_speed.max(0));
                for t in (1..=n).step_by(4).chain(std::iter::once(n)) {
                    let (x,y)=(f.x+vx*t,f.y+vy*t);
                    if !(input.sight)(f.x,f.y,x,y) {break;}
                    next.x=x;next.y=y;
                }
            }
            next
        }).collect()
    }
    /// Check the strongest reachable local escape, including time during the warning.
    fn counterplay_value(&self, input: &Input, elapsed: usize) -> i64 {
        let mut value=self.position_value(input);
        if value>=900_000 {
            // Counterplay estimate: an adjacent enemy may remove an injured
            // covering piece. This never alters authoritative piece health.
            if let Some(piece)=self.pieces.values().filter(|p| input.fighters.iter().any(|f|
                f.visible && f.team!=input.owner.team && f.champion && f.hp>0
                && f.cell().distance(p.cell)<=1 && f.attack>=p.hp))
                .min_by_key(|p|(p.hp,p.id)) {
                let mut broken=self.clone();broken.pieces.remove(&piece.id);broken.reindex();
                value=value.min(broken.position_value(input));
            }
        }
        let Some(k)=self.planning_target(input).filter(|f|f.champion && f.move_speed>0) else {return value};
        let budget=k.move_speed.saturating_mul((elapsed+MATE_WARN).min(180) as i64);
        let mut escape=None;
        for (dx,dy) in DIRS {
            for distance in 1..=2 {
                let cell=Cell::new(k.cell().x+dx*distance,k.cell().y+dy*distance);
                let (x,y)=cell.world();
                let travel=(x-k.x).abs().max((y-k.y).abs());
                if travel>budget || !(input.sight)(k.x,k.y,x,y) {continue;}
                let covered=self.pieces.keys().filter(|id|self.threatens(**id,cell,input)).count();
                let key=(covered,std::cmp::Reverse(distance),cell);
                if escape.as_ref().is_none_or(|(old,_)|key<*old) {escape=Some((key,(x,y)));}
            }
        }
        let Some((_,(x,y)))=escape else {return value};
        let mut replies=input.fighters.to_vec();
        if let Some(f)=replies.iter_mut().find(|f|f.id==k.id){f.x=x;f.y=y;}
        value.min(self.position_value(&Input {fighters:&replies,..*input}))
    }
    fn planning_target<'a>(&self, input: &'a Input) -> Option<&'a Fighter> {
        self.projected(Role::King)
            .and_then(|id| self.visible(input, id))
            .filter(|f| self.board.is_some_and(|b| b.inside(f.x, f.y)))
            .or_else(|| self.field_target.and_then(|id| self.visible(input, id)))
            .or_else(|| {
                input
                    .fighters
                    .iter()
                    .filter(|f| {
                        f.team != input.owner.team
                            && f.hp > 0
                            && f.visible
                            && f.targetable
                            && f.champion
                            && (input.owner.cell().distance(f.cell()) <= 8
                                || self.pieces.values().any(|p|p.cell.distance(f.cell())<=3))
                    })
                    .min_by_key(|f| {
                        (
                            (f.x - input.owner.x).pow(2) + (f.y - input.owner.y).pow(2),
                            f.id,
                        )
                    })
            })
            .or_else(|| input.fighters.iter().filter(|f| f.team!=input.owner.team && f.hp>0
                && f.visible && f.targetable && !f.champion && !f.friendly_chess
                && input.owner.cell().distance(f.cell())<=6)
                .min_by_key(|f| (input.owner.cell().distance(f.cell()),f.hp,f.id)))
    }
    fn mate_available(&self, target: &Fighter, input: &Input) -> bool {
        input.tick >= target.mate_immune_until
            && input.tick >= *self.immune.get(&target.id).unwrap_or(&0)
            && input.tick
                >= if self.board.is_some_and(|b| b.inside(target.x, target.y)) {
                    self.mate_at
                } else {
                    self.field_mate_at
                }
    }
    fn positioned_threat(&self, p: &Piece, from: Cell, to: Cell, input: &Input) -> bool {
        let (dx, dy) = (to.x - from.x, to.y - from.y);
        if dx == 0 && dy == 0 {
            return false;
        }
        if p.kind == Kind::Knight {
            return dx.abs() * dy.abs() == 2
                && dx.abs() + dy.abs() == 3
                && self.clear_segment(to, to, input);
        }
        if p.kind == Kind::Pawn {
            let (fx, fy) = forward(p.direction);
            return dx * fx + dy * fy == 1
                && (dx * fy - dy * fx).abs() == 1
                && self.clear_segment(from, to, input);
        }
        if from.distance(to) > 7
            || !match p.kind {
                Kind::Rook => dx == 0 || dy == 0,
                Kind::Bishop => dx.abs() == dy.abs(),
                Kind::Queen => dx == 0 || dy == 0 || dx.abs() == dy.abs(),
                _ => false,
            }
        {
            return false;
        }
        for n in 1..from.distance(to) {
            if self.occupied(
                Cell::new(from.x + dx.signum() * n, from.y + dy.signum() * n),
                p.id,
                input,
            ) {
                return false;
            }
        }
        self.clear_segment(from, to, input)
    }
    fn attack_value(&self, id: usize, to: Cell, input: &Input) -> i64 {
        let p = &self.pieces[&id];
        let pawn_attack = if p.kind == Kind::Pawn {
            let (fx, fy) = forward(p.direction);
            (to.x - p.cell.x) * fx + (to.y - p.cell.y) * fy == 1
                && ((to.x - p.cell.x) * fy - (to.y - p.cell.y) * fx).abs() == 1
        } else {
            true
        };
        if !pawn_attack {
            return 0;
        }
        let path = segment_cells(p.cell, to);
        input
            .fighters
            .iter()
            .filter(|f| f.team != input.owner.team && f.hp > 0 && f.visible && f.targetable)
            .filter(|f| {
                if matches!(p.kind, Kind::Knight | Kind::Pawn) {
                    f.cell() == to
                } else {
                    path.iter().skip(1).any(|c| *c == f.cell())
                }
            })
            .map(|f| {
                let territory = if self
                    .board
                    .is_some_and(|b| b.inside(to.world().0, to.world().1) && b.inside(f.x, f.y))
                {
                    100
                } else {
                    50
                };
                p.kind.damage(input.owner.ap) * territory / self.army_denominator()
                    * if f.champion { 8 } else { 2 }
            })
            .sum()
    }
    fn destination_interest(&self, id: usize, to: Cell, input: &Input) -> i64 {
        let p = &self.pieces[&id];
        let mut score = self.attack_value(id, to, input);
        if let Some(k) = self.planning_target(input) {
            if k.champion && self.mate_available(k, input) {
                for (dx, dy) in std::iter::once((0, 0)).chain(DIRS) {
                    let c = Cell::new(k.cell().x + dx, k.cell().y + dy);
                    if self.positioned_threat(p, to, c, input) {
                        score += if dx == 0 && dy == 0 { 1800 } else { 700 };
                    }
                }
            }
            score -= i64::from(to.distance(k.cell())) * 3;
        }
        // A cheap prefilter, not a proof of survival: the full position evaluates protection.
        if p.hp * 3 < p.max_hp {
            score -= input
                .fighters
                .iter()
                .filter(|f| {
                    f.team != input.owner.team
                        && f.visible
                        && f.champion
                        && to.distance(f.cell()) <= 1
                })
                .count() as i64
                * 300;
        }
        if p.kind == Kind::Pawn {
            let (fx, fy) = forward(p.direction);
            score += i64::from(
                ((to.x - p.origin.x) * fx + (to.y - p.origin.y) * fy - p.forward_frontier).max(0),
            ) * 40;
        }
        score
    }
    fn position_value(&self, input: &Input) -> i64 {
        let Some(k) = self.planning_target(input) else {
            return 0;
        };
        if !k.champion || !self.mate_available(k, input) {
            return 0;
        }
        let mut score = 0;
        let mut contributors = BTreeSet::new();
        for (dx, dy) in std::iter::once((0, 0)).chain(DIRS) {
            let c = Cell::new(k.cell().x + dx, k.cell().y + dy);
            let (x, y) = c.world();
            if (dx != 0 || dy != 0) && (input.wall)(x, y) {
                continue;
            }
            let ids: Vec<_> = self
                .pieces
                .keys()
                .filter(|id| self.threatens(**id, c, input))
                .copied()
                .collect();
            if !ids.is_empty() {
                score += if dx == 0 && dy == 0 { 600 } else { 120 };
                contributors.extend(ids);
            }
        }
        if self.has_move && self.trap(k.cell(), input).is_some() {
            score += 1_000_000;
        }
        for id in &contributors {
            let p = &self.pieces[id];
            let protected = self
                .pieces
                .keys()
                .any(|other| other != id && self.threatens(*other, p.cell, input));
            if protected {
                score += 350;
            }
            let danger = input
                .fighters
                .iter()
                .filter(|f| {
                    f.team != input.owner.team
                        && f.hp > 0
                        && f.visible
                        && f.champion
                        && p.cell.distance(f.cell()) <= 1
                })
                .count() as i64;
            score -= danger
                * if protected {
                    100
                } else {
                    if p.hp * 3 < p.max_hp {
                        1600
                    } else {
                        700
                    }
                };
        }
        score
    }
    fn simulate_action(&mut self, action: Action, input: &Input) -> Option<i64> {
        let value = match action {
            Action::Move { piece, to } => {
                if !self.legal(piece, to, input) {
                    return None;
                }
                let damage = self.attack_value(piece, to, input);
                let from = self.pieces[&piece].cell;
                let approach=self.planning_target(input).filter(|f| !f.champion).map_or(0,|f|
                    (from.distance(f.cell())-to.distance(f.cell())).max(0) as i64*80);
                let p = self.pieces.get_mut(&piece)?;
                if p.kind == Kind::Pawn {
                    p.previous = Some(from);
                    let (fx, fy) = forward(p.direction);
                    let depth = (to.x - p.origin.x) * fx + (to.y - p.origin.y) * fy;
                    p.ranks = p
                        .ranks
                        .saturating_add((depth - p.forward_frontier).max(0) as u8);
                    p.forward_frontier = p.forward_frontier.max(depth);
                }
                p.cell = to;
                p.moved = true;
                p.yielding = false;
                self.has_move |= p.kind != Kind::Pawn || damage > 0;
                damage+approach
            }
            Action::Summon {
                kind,
                to,
                direction,
            } => {
                if input.tick
                    < if kind == Kind::Pawn {
                        self.pawn_at
                    } else {
                        self.develop_at
                    }
                    || !self.summon_legal(kind, to, input)
                {
                    return None;
                }
                self.add_piece(
                    kind,
                    to,
                    (input.owner.max_hp * kind.hp_pct() / 100).max(1),
                    None,
                    direction,
                );
                if kind == Kind::Pawn {
                    self.pawn_at = input.tick + 360;
                } else {
                    self.develop_at = input.tick + 600;
                }
                50 - self.active_count().saturating_sub(7) as i64 * 10
            }
            _ => return None,
        };
        self.pending = None;
        self.reindex();
        Some(value)
    }
    fn search_actions(
        &self,
        input: &Input,
        ids: &[usize],
        per_piece: usize,
        summons: usize,
    ) -> Vec<Action> {
        let mut out = vec![];
        for id in ids.iter().filter(|id| self.pieces.contains_key(id)) {
            let mut cells = self.destinations(*id, input);
            cells.sort_by_cached_key(|to| {
                (
                    std::cmp::Reverse(self.destination_interest(*id, *to, input)),
                    *to,
                )
            });
            // Preserve a forcing/damage alternative instead of filling both slots
            // with near-identical positional moves.
            let attack=cells.iter().copied().max_by_key(|to|(self.attack_value(*id,*to,input),std::cmp::Reverse(*to)))
                .filter(|to|self.attack_value(*id,*to,input)>0);
            let mut selected:Vec<_>=cells.into_iter().take(per_piece).collect();
            if per_piece>1 {if let Some(to)=attack {if !selected.contains(&to) {selected.truncate(per_piece-1);selected.push(to);}}}
            out.extend(selected.into_iter().map(|to|Action::Move{piece:*id,to}));
        }
        if summons > 0 && (input.tick >= self.pawn_at || input.tick >= self.develop_at) {
            let mut cells = BTreeSet::new();
            for ally in std::iter::once(input.owner)
                .chain(input.fighters.iter())
                .filter(|f| {
                    f.team == input.owner.team && f.hp > 0 && f.sight_source && !f.friendly_chess
                })
            {
                for (dx, dy) in DIRS {
                    cells.insert(Cell::new(ally.cell().x + dx, ally.cell().y + dy));
                }
            }
            let target = self
                .planning_target(input)
                .map_or(input.owner.cell(), Fighter::cell);
            let mut cells: Vec<_> = cells.into_iter().collect();
            cells.sort_by_key(|c| (c.distance(target), *c));
            for to in cells.into_iter().take(summons) {
                let (dx, dy) = (target.x - to.x, target.y - to.y);
                let direction = if dx.abs() > dy.abs() {
                    if dx >= 0 {
                        1
                    } else {
                        3
                    }
                } else if dy >= 0 {
                    0
                } else {
                    2
                };
                let develop =
                    [Kind::Rook, Kind::Knight, Kind::Bishop][(self.stats.summons + self.rank) % 3];
                for kind in [Kind::Pawn, develop] {
                    if input.tick
                        >= if kind == Kind::Pawn {
                            self.pawn_at
                        } else {
                            self.develop_at
                        }
                        && self.summon_legal(kind, to, input)
                    {
                        out.push(Action::Summon {
                            kind,
                            to,
                            direction,
                        });
                    }
                }
            }
        }
        out
    }
    pub fn plan(&mut self, input: &Input) -> Option<Action> {
        let geometry=std::cell::RefCell::new(BTreeMap::new());
        let cached=Input{geometry:Some(&geometry),..*input};
        let input=&cached;
        let mut forecasts=BTreeMap::new();
        self.last_plan_candidates = 0;
        self.plan_score=0;self.plan_damage=0;self.plan_counterplay=0;
        if input.controlled || self.pending.is_some() {
            return None;
        }
        self.plans.clear();
        self.plan_target = self.planning_target(input).map(|f| f.id);
        self.plan_target_cell = self.planning_target(input).map(Fighter::cell);
        self.plan_at = input.tick;
        self.plan_reason = 0;
        if self.planning_target(input).is_none() {return None;}
        if self.mate_warning.is_some()
            && self
                .mate_target
                .and_then(|id| self.visible(input, id))
                .is_some_and(|k| self.trap(k.cell(), input).is_some())
        {
            self.plan_reason = 1;
            return None;
        }
        let foes: Vec<_> = input
            .fighters
            .iter()
            .filter(|f| f.team != input.owner.team && f.hp > 0 && f.visible && f.targetable)
            .collect();
        if input.owner.hp * 3 < input.owner.max_hp && input.tick >= self.castle_at {
            if let Some(p) = self
                .pieces
                .values()
                .filter(|p| p.kind == Kind::Rook && self.castle_legal(p.id, input))
                .find(|p| {
                    foes.iter().all(|f| {
                        !f.champion
                            || p.cell.distance(f.cell()) > input.owner.cell().distance(f.cell())
                    })
                })
            {
                let action = Action::Castle { piece: p.id };
                self.plans = vec![vec![action]];
                self.plan_reason = 2;
                return Some(action);
            }
        }
        if input.ult_unlocked
            && input.tick >= self.reset_at
            && (self.board.is_none() || self.slots.iter().flatten().count() < 4)
        {
            if let Some(foe) = foes
                .iter()
                .min_by_key(|f| {
                    (
                        (f.x - input.owner.x).pow(2) + (f.y - input.owner.y).pow(2),
                        f.id,
                    )
                })
                .filter(|f| input.owner.cell().distance(f.cell()) <= 7)
            {
                let (dx, dy) = (foe.x - input.owner.x, foe.y - input.owner.y);
                let heading = if dx.abs() > dy.abs() {
                    if dx >= 0 {
                        1
                    } else {
                        3
                    }
                } else if dy >= 0 {
                    0
                } else {
                    2
                };
                if self.placement(input, heading, self.rng).is_some() {
                    let a = Action::Deploy { heading };
                    self.plans = vec![vec![a]];
                    self.plan_reason = 3;
                    return Some(a);
                }
            }
        }
        if self
            .challenge
            .as_ref()
            .is_some_and(|c| c.id == 10 && !c.done && !c.failed)
        {
            if let Some(p) = self.pieces.values().find(|p| {
                p.kind == Kind::Pawn && p.qualified && p.ranks >= 5 && p.promotion != Kind::Rook
            }) {
                let a = Action::Promote {
                    piece: p.id,
                    kind: Kind::Rook,
                };
                self.plans = vec![vec![a]];
                return Some(a);
            }
        }
        let all: Vec<_> = self.pieces.keys().copied().collect();
        let count = all.len().min(16);
        let ids: Vec<_> = (0..count)
            .map(|i| all[(self.planner_cursor + i) % all.len()])
            .collect();
        self.planner_cursor = (self.planner_cursor + count.max(1)) % all.len().max(1);
        let horizon = 1 + self.rank * 3 / 7;
        let root_limit = 64 - (horizon - 1) * 8;
        let candidates = self.search_actions(input, &ids, 2, 4);
        // Position score dominates immediate damage. A full trap dominates every partial setup.
        let mut beam = vec![];
        for a in candidates.into_iter().take(root_limit) {
            let mut view = self.clone();
            let elapsed=self.command_ticks(a,input.tick);
            let fighters=forecasts.entry(elapsed).or_insert_with(||self.forecast(input,elapsed));
            let future=Input {tick:input.tick+elapsed,fighters:&fighters,..*input};
            if let Some(damage) = view.simulate_action(a, &future) {
                self.last_plan_candidates += 1;
                let score = view.position_value(&future) + damage - 100;
                beam.push((score, damage, vec![a], view));
            }
        }
        beam.sort_by_key(|(s, _, p, _)| (std::cmp::Reverse(*s), action_key(p[0])));
        beam.truncate(8);
        for depth in 1..horizon {
            let mut next = vec![];
            let allowance = (64 - self.last_plan_candidates) / (horizon - depth).max(1);
            let mut spent = 0;
            for (_, damage, path, view) in &beam {
                if spent >= allowance {
                    break;
                }
                let branch_elapsed=path.iter().map(|a|self.command_ticks(*a,input.tick)).sum::<usize>();
                let fighters=forecasts.entry(branch_elapsed).or_insert_with(||self.forecast(input,branch_elapsed));
                let branch_input=Input{tick:input.tick+branch_elapsed,fighters:&fighters,..*input};
                if view.has_move
                    && view
                        .planning_target(&branch_input)
                        .is_some_and(|k| view.trap(k.cell(), &branch_input).is_some())
                {
                    next.push((
                        view.position_value(&branch_input) + *damage - path.len() as i64 * 10_000,
                        *damage,
                        path.clone(),
                        view.clone(),
                    ));
                    continue;
                }
                let mut candidates = view.search_actions(&branch_input, &ids, 1, 0);
                candidates.sort_by_cached_key(|a| match *a {
                    Action::Move { piece, to } => (
                        std::cmp::Reverse(view.destination_interest(piece, to, &branch_input)),
                        action_key(*a),
                    ),
                    _ => (std::cmp::Reverse(0), action_key(*a)),
                });
                let take = ((allowance - spent) / (beam.len().max(1))).max(1).min(2);
                for a in candidates.into_iter().take(take) {
                    if spent >= allowance {
                        break;
                    }
                    let mut child = view.clone();
                    let elapsed=path.iter().map(|a|self.command_ticks(*a,input.tick)).sum::<usize>()+view.command_ticks(a,input.tick);
                    let fighters=forecasts.entry(elapsed).or_insert_with(||self.forecast(input,elapsed));
                    let future=Input {tick:input.tick+elapsed,fighters:&fighters,..*input};
                    if let Some(hit) = child.simulate_action(a, &future) {
                        self.last_plan_candidates += 1;
                        spent += 1;
                        let mut route = path.clone();
                        route.push(a);
                        let reward = *damage + hit;
                        let value = child.position_value(&future);
                        let score = value + reward
                            - route.len() as i64 * if value >= 1_000_000 { 10_000 } else { 100 };
                        next.push((score, reward, route, child));
                    }
                }
            }
            if next.is_empty() {
                break;
            }
            next.sort_by_key(|(s, _, p, _)| {
                (
                    std::cmp::Reverse(*s),
                    p.iter().map(|a| action_key(*a)).collect::<Vec<_>>(),
                )
            });
            next.truncate(8);
            beam = next;
        }
        if beam.is_empty() {
            return None;
        }
        for (score, damage, path, view) in &mut beam {
            let elapsed=path.iter().map(|a|self.command_ticks(*a,input.tick)).sum::<usize>();
            let fighters=forecasts.entry(elapsed).or_insert_with(||self.forecast(input,elapsed));
            let future=Input {tick:input.tick+elapsed,fighters:&fighters,..*input};
            let position=view.counterplay_value(&future,elapsed);
            *score=position+*damage-path.len() as i64 * if position>=1_000_000 {10_000} else {100};
        }
        beam.sort_by_key(|(s,_,p,_)|(std::cmp::Reverse(*s),p.iter().map(|a|action_key(*a)).collect::<Vec<_>>()));
        // Seeded mistakes cannot discard a proven mating line or emergency survival action.
        let mistake =
            beam[0].0 < 900_000 && self.roll(100) < [28, 22, 17, 12, 8, 5, 2, 0][self.rank];
        let index = if mistake {
            self.roll(beam.len().min(3))
        } else {
            0
        };
        let selected = beam.remove(index);
        if selected.1<=0 && selected.0<=self.position_value(input)
            && !matches!(selected.2[0],Action::Summon{..}) {
            self.plan_reason=7;
            return None;
        }
        self.plan_score=selected.0;
        let first_elapsed=self.command_ticks(selected.2[0],input.tick);
        let fighters=forecasts.entry(first_elapsed).or_insert_with(||self.forecast(input,first_elapsed));
        let first_input=Input{tick:input.tick+first_elapsed,fighters:&fighters,..*input};
        self.plan_damage=match selected.2[0] {Action::Move{piece,to}=>self.attack_value(piece,to,&first_input),_=>0};
        let elapsed=selected.2.iter().map(|a|self.command_ticks(*a,input.tick)).sum::<usize>();
        let fighters=forecasts.entry(elapsed).or_insert_with(||self.forecast(input,elapsed));
        let future=Input{tick:input.tick+elapsed,fighters:&fighters,..*input};
        self.plan_counterplay=selected.3.position_value(&future)-selected.3.counterplay_value(&future,elapsed);
        self.plan_reason = if selected.0 >= 900_000 { 4 }
            else if self.planning_target(input).is_some_and(|f|!f.champion) {8}
            else if self.plan_damage>0 {5} else {9};
        self.plans.push(selected.2);
        for (_, _, path, _) in beam.into_iter() {
            if self.plans.len() >= 3 {
                break;
            }
            if !self.plans.contains(&path) {
                self.plans.push(path);
            }
        }
        self.plans.first().and_then(|p| p.first()).copied()
    }
    fn validate_preview(&mut self, input: &Input) {
        // Validate the completed continuation against current terrain and occupancy.
        // This is bounded legality replay, not another search or a gameplay RNG call.
        let stale=self.plans.iter().any(|path|{
            let mut view=self.clone();view.pending=None;view.plans.clear();
            path.iter().any(|action|match action{
                Action::Move{..}|Action::Summon{..}=>view.simulate_action(*action,input).is_none(),
                Action::Castle{piece}=>!view.castle_legal(*piece,input),
                Action::Deploy{heading}=>!input.ult_unlocked||view.placement(input,*heading,view.rng).is_none(),
                _=>false,
            })
        });
        if stale{self.plans.clear();return;}
        if self.plan_target.is_some_and(|id|self.visible(input,id).is_none_or(|f|Some(f.cell())!=self.plan_target_cell))||self.plans.iter().flatten().any(|a|matches!(a,Action::Move{piece,..}|Action::Castle{piece}|Action::Promote{piece,..}if !self.pieces.contains_key(piece))){self.plans.clear();self.plan_target=None;self.plan_target_cell=None;return;}
        if let Some(first) = self.plans.first().and_then(|p| p.first()).copied() {
            let valid = match first {
                Action::Move { piece, to } => {
                    self.pending
                        .as_ref()
                        .is_some_and(|p| p.committed && p.action == first)
                        || self.legal(piece, to, input)
                }
                Action::Deploy { .. } => input.ult_unlocked,
                _ => true,
            };
            if !valid {
                self.plans.clear();
            }
        }
    }
}
