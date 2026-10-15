//! Host adapter: real targetable pieces, integer snapshots and owner-scoped visuals.
use crate::magnus_core::{Action, Board, Capture, Cell, Cmd, Core, Fighter, Input, Kind};
use crate::mastery::{signature, Book, Record};
use crate::{champions, fx_point, fx_unit, timed, walls};
use mod_api_stable::{
    AttackTypeV1, CcKindV1, CcV1, StablePassive, StableSim, StatV1, UnitAttackV1,
};

pub const ID: &str = "tfm2_custom_magnus";
pub static BOOK: Book = Book::new("magnus");
#[derive(Clone, Default)]
pub struct Magnus {
    core: Option<Core>,
    units: [Option<usize>; 7],
    owner: usize,
    player: usize,
    last_tick: Option<usize>,
    rank: usize,
    athlete: Option<usize>,
    record: Record,
    sig: Option<String>,
    visuals: Vec<String>,
    zones: Vec<String>,
    icons: Vec<(usize, String)>,
    coverage_buffs: Vec<(usize, String)>,
    cast_start: Option<usize>,
    facing: crate::facing::Facing,
    probe: bool,
    probe_phase: usize,
    probe_next: usize,
}
impl Magnus {
    fn owned(&self, sim: &StableSim<'_>, i: usize) -> Option<usize> {
        let id = self.units[i]?;
        let e = sim.get_entity(id)?;
        let marker = format!("mg_owner:{}:{i}", self.owner);
        (e.is_alive()
            && !e.is_champion()
            && (0..e.buff_count())
                .filter_map(|k| e.buff_at(k))
                .any(|b| b.name() == marker))
        .then_some(id)
    }
    fn retire(&mut self, sim: &mut StableSim<'_>, i: usize) {
        if let Some(id) = self.owned(sim, i) {
            sim.entity_set_hp(id, 0);
        }
        self.units[i] = None;
        self.coverage_buffs.retain(|(piece, _)| *piece != i);
    }
    fn snapshot(&self, sim: &StableSim<'_>, me: usize) -> Option<(Fighter, Vec<Fighter>, bool)> {
        let e = sim.get_entity(me)?;
        let team = e.team();
        let controlled = (0..e.cc_count()).filter_map(|i| e.cc_at(i)).any(|c| {
            [
                CcKindV1::Stun,
                CcKindV1::BlockSkill,
                CcKindV1::Airborne,
                CcKindV1::Fear,
                CcKindV1::Charm,
                CcKindV1::Taunt,
            ]
            .iter()
            .any(|k| c.kind == k.code())
        });
        let all = (0..sim.entity_count())
            .filter_map(|i| sim.entity_at(i))
            .filter(|e| e.is_alive() && !e.is_tower())
            .map(|e| {
                let (x, y) = e.pos();
                let (hp, max_hp) = e.hp();
                let stat = e.stat();
                let (tank, ranged) = crate::magnus_data::tags(&e.name().unwrap_or_default());
                Fighter {
                    id: e.id(),
                    team: e.team(),
                    x: x as i64,
                    y: y as i64,
                    hp: hp as i64,
                    max_hp: max_hp as i64,
                    attack: stat.attack as i64,
                    ap: stat.magic_power as i64,
                    champion: e.is_champion(),
                    visible: e.team() == team || sim.is_visible(team, e.id()),
                    targetable: e.is_targetable(),
                    tank,
                    ranged,
                    friendly_chess: (0..e.buff_count()).filter_map(|i| e.buff_at(i)).any(|b| {
                        b.name()
                            .strip_prefix("mg_owner:")
                            .and_then(|v| v.split(':').next()?.parse::<usize>().ok())
                            .is_some_and(|id| id != me)
                    }),
                }
            })
            .collect::<Vec<_>>();
        Some((all.iter().find(|f| f.id == me)?.clone(), all, controlled))
    }
    fn apply_fx(&self, sim: &mut StableSim<'_>, tag: &str, x: i64, y: i64, ticks: usize) {
        let name = format!("{ID}_{tag}");
        if tag.starts_with("label") || tag.starts_with("game") || tag == "brilliant" {
            fx_unit(sim, &name, self.owner, self.owner, ticks as u64);
        } else {
            fx_point(sim, &name, self.owner, x, y, ticks as u64);
        }
    }
    fn apply(&mut self, sim: &mut StableSim<'_>, core: &mut Core, commands: Vec<Cmd>) {
        let tick = sim.tick();
        for c in commands {
            match c {
                Cmd::Spawn { piece, x, y, hp } => {
                    self.retire(sim, piece);
                    let stat = StatV1 {
                        hp: core.pieces[piece].max_hp.max(1) as usize,
                        defence: 15,
                        magic_resistance: 15,
                        move_speed: 1,
                        ..StatV1::default()
                    };
                    let atk = UnitAttackV1 {
                        attack: 0,
                        attack_ratio: 0,
                        range: 1,
                        cooltime: 100_000,
                        duration: 10,
                        start_timing: 5,
                        cancelable: true,
                        attack_type: AttackTypeV1::BaseAttack.code(),
                        ..UnitAttackV1::default()
                    };
                    // As with UT's crystal, a known base name avoids the renderer's custom-unit-name panic.
                    if let Some(id) = sim.spawn_unit(
                        "bombardier",
                        self.owner,
                        self.snapshot(sim, self.owner).map_or(0, |s| s.0.team),
                        x.max(0) as u64,
                        y.max(0) as u64,
                        5_184_000,
                        &stat,
                        &atk,
                    ) {
                        sim.entity_set_hp(id, hp.max(1) as usize);
                        for k in [CcKindV1::Bind, CcKindV1::BlockAttack, CcKindV1::BlockSkill] {
                            sim.apply_cc(id, &CcV1::of_kind(k, 5_184_000));
                        }
                        sim.add_buff(
                            id,
                            &timed(&format!("mg_owner:{}:{piece}", self.owner), 5_184_000),
                        );
                        sim.add_buff(
                            id,
                            &timed(
                                &format!("mg_piece_{}", core.pieces[piece].kind.name()),
                                5_184_000,
                            ),
                        );
                        sim.add_buff(id, &timed(&format!("mg_identity{piece}"), 5_184_000));
                        self.units[piece] = Some(id);
                        BOOK.log(&format!("native {} tick {tick} owner {} piece {piece} {} spawned entity {id} HP {hp} at {x},{y}{}",crate::VERSION,self.owner,core.pieces[piece].kind.name(),if self.probe{" KNIGHT PROBE"}else{""}));
                    } else {
                        core.pieces[piece].hp = 0;
                        core.pieces[piece].ready_at = tick + 60;
                        BOOK.log(&format!(
                            "native {} owner {} piece {piece} spawn FAILED; excluded from coverage",
                            crate::VERSION,
                            self.owner
                        ));
                    }
                }
                Cmd::Despawn { piece } => self.retire(sim, piece),
                Cmd::Move { piece, x, y } => {
                    if let Some(id) = self.owned(sim, piece) {
                        sim.entity_set_pos(id, x.max(0) as u64, y.max(0) as u64);
                    }
                }
                Cmd::Damage {
                    piece,
                    target,
                    amount,
                } => {
                    if self.owned(sim, piece).is_none() {
                        continue;
                    }
                    let before = sim.get_entity(target).map_or(0, |e| e.hp().0);
                    sim.deal_damage(
                        self.owner,
                        target,
                        0,
                        amount.max(0) as usize,
                        AttackTypeV1::Skill,
                    );
                    let after = sim.get_entity(target).map_or(0, |e| e.hp().0);
                    BOOK.log(&format!("native {} tick {tick} owner {} piece {piece} target {target} raw {amount} HP {before}->{after}",crate::VERSION,self.owner));
                }
                Cmd::Stun { target, ticks } => {
                    sim.apply_cc(target, &CcV1::of_kind(CcKindV1::Stun, ticks as u64))
                }
                Cmd::Shield { amount, ticks } => {
                    sim.entity_add_shield(self.owner, amount.max(0) as usize, ticks);
                }
                Cmd::OwnerPos { x, y } => {
                    sim.entity_set_pos(self.owner, x.max(0) as u64, y.max(0) as u64);
                }
                Cmd::Fx { tag, x, y, ticks } => self.apply_fx(sim, tag, x, y, ticks),
                Cmd::Label { code } => {
                    BOOK.log(&format!(
                        "native {} tick {tick} owner {} label {code} challenge {:?}",
                        crate::VERSION,
                        self.owner,
                        core.challenge
                            .as_ref()
                            .map(|c| (c.id, c.progress, c.done, c.failed))
                    ));
                }
            }
        }
    }
    fn visuals(
        &mut self,
        sim: &mut StableSim<'_>,
        core: &mut Core,
        _me: &Fighter,
        all: &[Fighter],
    ) {
        let handles: [Option<usize>; 7] = std::array::from_fn(|i| self.owned(sim, i));
        let mut coverage = vec![];
        if let Some(b) = core.board {
            for n in 0..64 {
                if let Some(i) =
                    (0..7).find(|&i| handles[i].is_some() && core.coverage[i] & (1u64 << n) != 0)
                {
                    let p = core.pieces[i].cell;
                    let (dx, dy) = ((n % 8) as i8 - p.x, (n / 8) as i8 - p.y);
                    let (dx, dy) = match b.heading % 4 {
                        0 => (dx, dy),
                        1 => (dy, -dx),
                        2 => (-dx, -dy),
                        _ => (-dy, dx),
                    };
                    coverage.push((i, format!("mg_cov_{}_{}", dx + 7, dy + 7)));
                }
            }
        }
        for (i, name) in &self.coverage_buffs {
            if !coverage.contains(&(*i, name.clone())) {
                if let Some(id) = handles[*i] {
                    sim.entity_remove_buff(id, name);
                }
            }
        }
        for (i, name) in &coverage {
            if !self.coverage_buffs.contains(&(*i, name.clone())) {
                if let Some(id) = handles[*i] {
                    sim.add_buff(id, &timed(name, 5_184_000));
                }
            }
        }
        self.coverage_buffs = coverage;
        let mut want = vec![format!("mg_rank{}", self.rank.min(7))];
        if let Some(b) = core.board {
            for projection in &core.projections {
                if let Some(f) = all
                    .iter()
                    .find(|f| f.id == projection.id && f.visible && f.hp > 0 && b.inside(f.x, f.y))
                {
                    let cell = b.cell(f.x, f.y);
                    let name = format!(
                        "mg_mini_{}_{}_{}",
                        if self.facing.left() { "l" } else { "r" },
                        projection.role.code(),
                        cell.index()
                    );
                    if !want.contains(&name) {
                        want.push(name);
                    }
                }
            }
        }
        let cast_start = core.pending.as_ref().map(|p| p.start);
        if cast_start != self.cast_start {
            self.visuals.retain(|n| {
                if n.starts_with("mg_cast") {
                    sim.entity_remove_buff(self.owner, n);
                    false
                } else {
                    true
                }
            });
            self.cast_start = cast_start;
        }
        if let Some(c) = &core.challenge {
            want.push(format!("mg_clue{}", c.id));
        }
        if let Some(p) = &core.pending {
            if !matches!(p.action, Action::Deploy { .. } | Action::Dismiss) {
                let left = self.facing.left();
                let queen = matches!(p.action,Action::Move{piece,..}if core.pieces[piece].kind==Kind::Queen);
                want.push(format!(
                    "mg_cast{}{}{}",
                    if queen { "_queen" } else { "" },
                    if p.end - p.start == 24 || p.end - p.start == 36 {
                        "_fast"
                    } else {
                        ""
                    },
                    if left { "_l" } else { "" }
                ));
            }
        }
        for old in &self.visuals {
            if !want.contains(old) {
                sim.entity_remove_buff(self.owner, old);
            }
        }
        for name in &want {
            if !self.visuals.contains(name) {
                sim.add_buff(self.owner, &timed(name, 5_184_000));
            }
        }
        self.visuals = want;
        let slot = self.player % 16;
        let icons = core
            .projections
            .iter()
            .filter(|p| {
                all.iter().any(|f| {
                    f.id == p.id
                        && f.visible
                        && f.hp > 0
                        && core.board.is_some_and(|b| b.inside(f.x, f.y))
                })
            })
            .map(|p| (p.id, format!("mg_enemy{slot}_{}", p.role.code())))
            .collect::<Vec<_>>();
        for (id, name) in &self.icons {
            if !icons.contains(&(*id, name.clone())) {
                sim.entity_remove_buff(*id, name);
            }
        }
        for (id, name) in &icons {
            if !self.icons.contains(&(*id, name.clone())) {
                sim.add_buff(*id, &timed(name, 5_184_000));
            }
        }
        self.icons = icons;
        // Engine unit HP is authoritative. The extra bar mirrors it; it never restores damage.
        for i in 0..7 {
            if let Some(id) = self.owned(sim, i) {
                let glow = core.glow_mask & (1 << i) != 0 && sim.tick() < core.glow_until;
                if glow {
                    let has = sim.get_entity(id).is_some_and(|e| {
                        (0..e.buff_count())
                            .filter_map(|k| e.buff_at(k))
                            .any(|b| b.name() == "mg_glow")
                    });
                    if !has {
                        sim.add_buff(id, &timed("mg_glow", 75));
                    }
                } else {
                    sim.entity_remove_buff(id, "mg_glow");
                }
                let (hp, max) = sim.get_entity(id).map_or((0, 1), |e| e.hp());
                let bar = (hp * 10 / max.max(1)).min(10);
                for k in 0..=10 {
                    if k != bar {
                        sim.entity_remove_buff(id, &format!("mg_hp{k}"));
                    }
                }
                let name = format!("mg_hp{bar}");
                if sim.get_entity(id).is_some_and(|e| {
                    !(0..e.buff_count())
                        .filter_map(|k| e.buff_at(k))
                        .any(|b| b.name() == name)
                }) {
                    sim.add_buff(id, &timed(&name, 5_184_000));
                }
            }
        }
        self.sync_zones(sim, core);
    }
    fn sync_zones(&mut self, sim: &mut StableSim<'_>, core: &mut Core) {
        let tick = sim.tick();
        let mut want = vec![];
        if let Some(b) = core.board {
            let mask = core.warning_cells();
            if mask != 0 {
                let until = core
                    .pending
                    .as_ref()
                    .map_or_else(|| core.mate_warning.unwrap_or(tick) + 24, |p| p.end);
                let name = format!("mgz:{}:{}:{}:{mask:x}:{until}", b.x, b.y, b.heading);
                want.push(name.clone());
            }
        }
        if !want.is_empty() {
            crate::steve::note_map_buff(sim.seed());
        }
        for n in &self.zones {
            if !want.contains(n) {
                sim.entity_remove_buff(self.owner, n);
            }
        }
        for n in &want {
            if !self.zones.contains(n) {
                sim.add_buff(self.owner, &timed(n, 90));
            }
        }
        self.zones = want;
    }
}
impl StablePassive for Magnus {
    fn clone_box(&self) -> Box<dyn StablePassive> {
        Box::new(self.clone())
    }
    fn configure(&mut self, params: &str) {
        self.probe = params
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect::<String>()
            .contains("\"knight_probe\":1");
    }
    fn on_dead(&mut self, sim: &mut StableSim<'_>, _: usize) {
        let Some(mut core) = self.core.take() else {
            return;
        };
        let cmds = core.on_dead();
        self.apply(sim, &mut core, cmds);
        for n in self.visuals.drain(..).chain(self.zones.drain(..)) {
            sim.entity_remove_buff(self.owner, &n);
        }
        for (id, n) in self.icons.drain(..) {
            sim.entity_remove_buff(id, &n);
        }
        if self.probe {
            self.probe_phase = 0;
            self.probe_next = sim.tick() + 120;
        }
        self.core = Some(core);
    }
    fn on_update(&mut self, sim: &mut StableSim<'_>, _: u64, player: usize, entity: usize) {
        let tick = sim.tick();
        if self.last_tick == Some(tick) {
            return;
        }
        self.last_tick = Some(tick);
        self.owner = entity;
        self.player = player;
        let Some((me, all, controlled)) = self.snapshot(sim, entity) else {
            self.on_dead(sim, player);
            return;
        };
        let enemy = all
            .iter()
            .filter(|f| {
                f.champion
                    && f.team != me.team
                    && f.visible
                    && (f.x - me.x).pow(2) + (f.y - me.y).pow(2) <= crate::facing::FACE_ENEMY.pow(2)
            })
            .min_by_key(|f| ((f.x - me.x).pow(2) + (f.y - me.y).pow(2), f.id))
            .map(|f| (f.x, f.y));
        self.facing.update(tick, (me.x, me.y), enemy);
        self.athlete = BOOK.athlete_of(sim.seed(), player);
        self.rank = BOOK.pinned(sim.seed()).rank_for(self.athlete).0.min(7);
        let mut core = self
            .core
            .take()
            .unwrap_or_else(|| Core::new(sim.seed(), entity, self.rank));
        core.owner = entity;
        core.rank = self.rank;
        let mut captures = vec![];
        let mut hp = [None; 7];
        for i in 0..7 {
            if let Some(id) = self.units[i] {
                if let Some(uid) = self.owned(sim, i) {
                    hp[i] = sim.get_entity(uid).map(|e| e.hp().0 as i64);
                } else if core.pieces[i].hp > 0 {
                    // Only an externally destroyed live unit counts; our resets/death take its handle first.
                    let combat = sim.get_entity(id).is_none_or(|e| !e.is_alive());
                    captures.push(Capture {
                        piece: i,
                        by: None,
                        combat,
                    });
                    self.units[i] = None;
                    BOOK.log(&format!(
                        "native {} tick {tick} owner {entity} piece {i} externally captured",
                        crate::VERSION
                    ));
                }
            }
        }
        let dynamic = crate::steve::walls_up(&champions(sim), tick);
        let wall = |x: i64, y: i64| {
            walls::wall_at(x, y)
                || dynamic.iter().any(|w| {
                    crate::steve::crosses(w, tick, x - 1000, y, x + 1000, y)
                        || crate::steve::crosses(w, tick, x, y - 1000, x, y + 1000)
                })
        };
        let mut action = None;
        if self.probe && core.pending.is_none() && tick >= self.probe_next {
            action = if core.board.is_none() {
                Some(Action::Deploy { heading: 0 })
            } else {
                let p = core.pieces[4].cell;
                [(1, 2), (-1, 2), (2, 1), (-2, 1), (1, -2), (-1, -2)]
                    .iter()
                    .map(|(x, y)| Cell::new(p.x + x, p.y + y))
                    .find(|c| core.legal(4, *c, core.board.unwrap().cell(me.x, me.y), &wall))
                    .map(|to| Action::Move { piece: 4, to })
            };
            self.probe_next = tick + 120;
        }
        let input = Input {
            tick,
            owner: &me,
            fighters: &all,
            wall: &wall,
            captures: &captures,
            piece_hp: hp,
            controlled,
            automatic: !self.probe,
            action,
        };
        let mut commands = core.step(&input);
        if self.probe && core.board.is_some() && self.probe_phase == 0 {
            for i in 0..7 {
                if i != 4 {
                    core.pieces[i].hp = 0;
                    core.pieces[i].ready_at = usize::MAX / 2;
                    commands.retain(|c| !matches!(c,Cmd::Spawn{piece,..}if *piece==i));
                }
            }
            core.challenge = None;
            self.probe_phase = 1;
        }
        self.apply(sim, &mut core, commands);
        if sim.get_entity(entity).is_none_or(|e| !e.is_alive()) {
            self.core = Some(core);
            self.on_dead(sim, player);
            return;
        }
        self.visuals(sim, &mut core, &me, &all);
        if self.probe && tick.is_multiple_of(120) {
            if let Some(id) = self.owned(sim, 4) {
                let e = sim.get_entity(id).unwrap();
                BOOK.log(&format!("KNIGHT PROBE native {} tick {tick} entity {id} HP {:?} position {:?} targetable {} controlled {} cell {:?}; no autonomous damage requested",crate::VERSION,e.hp(),e.pos(),e.is_targetable(),e.cc_count(),core.pieces[4].cell));
            }
        }
        if self.sig.is_none() && tick >= 1800 {
            self.sig = Some(signature(&BOOK, sim, &champions(sim), self.athlete));
        }
        if let (Some(sig), Some(a)) = (&self.sig, self.athlete) {
            if tick.is_multiple_of(30) {
                self.record.track(&BOOK, sim, sig, me.team, a, tick);
            }
        }
        self.core = Some(core);
    }
}

/// The shared input AI uses only imminent warnings, preserving the game's casts and attacks.
pub fn avoid(
    all: &[crate::Champ],
    me: &crate::Champ,
    dest: (i64, i64),
    tick: usize,
) -> Option<(i64, i64)> {
    for foe in all.iter().filter(|c| c.team != me.team) {
        for buff in &foe.buffs {
            let Some(raw) = buff.name().strip_prefix("mgz:") else {
                continue;
            };
            let a = raw.split(':').collect::<Vec<_>>();
            if a.len() != 5 {
                continue;
            }
            let (Ok(x), Ok(y), Ok(h), Ok(mask), Ok(until)) = (
                a[0].parse(),
                a[1].parse(),
                a[2].parse(),
                u64::from_str_radix(a[3], 16),
                a[4].parse::<usize>(),
            ) else {
                continue;
            };
            if tick >= until {
                continue;
            }
            let b = Board {
                x,
                y,
                heading: h,
                serial: 0,
            };
            let from = b.cell(me.x, me.y);
            let to = b.cell(dest.0, dest.1);
            if from.bit() & mask == 0 && to.bit() & mask == 0 {
                continue;
            }
            let center = if from.bit() & mask != 0 { from } else { to };
            let best = (-1..=1)
                .flat_map(|dx| (-1..=1).map(move |dy| Cell::new(center.x + dx, center.y + dy)))
                .filter(|c| c.bit() & mask == 0)
                .map(|c| b.world(c))
                .filter(|p| !walls::wall_at(p.0, p.1))
                .min_by_key(|p| ((p.0 - dest.0).abs() + (p.1 - dest.1).abs(), p.0, p.1));
            if best.is_some() {
                return best;
            }
        }
    }
    None
}
