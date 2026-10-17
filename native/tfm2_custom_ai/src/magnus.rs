//! Thin host adapter: targetable carriers, conservative damage receipts and shared CC immunity.
use crate::magnus_core::*;
use crate::mastery::{signature, Book, Record};
use crate::{champions, fx_point, fx_unit, timed, walls};
use mod_api_stable::{
    AttackTypeV1, BuffV1, CcKindV1, CcV1, StablePassive, StableSim, StatV1, UnitAttackV1,
};
use std::collections::{BTreeMap, BTreeSet};
pub const ID: &str = "tfm2_custom_magnus";
pub static BOOK: Book = Book::new("magnus");
#[derive(Clone)]
pub struct UltimateLearned;
impl StablePassive for UltimateLearned {
    fn clone_box(&self)->Box<dyn StablePassive>{Box::new(self.clone())}
    fn on_spawn(&mut self,sim:&mut StableSim<'_>,_:usize,entity:usize){
        if !has(sim,entity,"mg_ult_learned"){sim.add_buff(entity,&BuffV1::named("mg_ult_learned"));}
    }
    fn on_update(&mut self,sim:&mut StableSim<'_>,_:u64,player:usize,entity:usize){self.on_spawn(sim,player,entity);}
}
#[derive(Clone, Default)]
pub struct Magnus {
    core: Option<Core>,
    units: BTreeMap<usize, usize>,
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
    coverage: Vec<(usize, String)>,
    piece_views: BTreeMap<usize, Vec<String>>,
    cast_start: Option<usize>,
    facing: crate::facing::Facing,
    hits: Vec<Hit>,
    probes: Vec<(Hit,usize,usize)>,
    damage_logs: usize,
    probe: u8,
    last_pos: Option<(i64,i64)>,
    moving_until: usize,
    attack_until: usize,
    hit_until: usize,
    observations: BTreeMap<usize, (usize, i64, i64)>,
}
fn has(sim: &StableSim<'_>, id: usize, name: &str) -> bool {
    sim.get_entity(id).is_some_and(|e| {
        (0..e.buff_count())
            .filter_map(|i| e.buff_at(i))
            .any(|b| b.name() == name)
    })
}
impl Magnus {
    fn owned(&self, sim: &StableSim<'_>, piece: usize) -> Option<usize> {
        let id = *self.units.get(&piece)?;
        let e = sim.get_entity(id)?;
        (e.is_alive() && has(sim, id, &format!("mg_owner:{}:{piece}", self.owner))).then_some(id)
    }
    fn retire(&mut self, sim: &mut StableSim<'_>, piece: usize) {
        if let Some(id) = self.owned(sim, piece) {
            self.units.remove(&piece);
            sim.entity_set_hp(id, 0);
        }
        self.units.remove(&piece);
        self.piece_views.remove(&piece);
        self.coverage.retain(|(p, _)| *p != piece);
    }
    fn snapshot(&self, sim: &StableSim<'_>, id: usize) -> Option<(Fighter, Vec<Fighter>, bool)> {
        let me = sim.get_entity(id)?;
        let team = me.team();
        let tick = sim.tick();
        let controlled = (0..me.cc_count()).filter_map(|i| me.cc_at(i)).any(|c| {
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
        let all: Vec<_> = (0..sim.entity_count())
            .filter_map(|i| sim.entity_at(i))
            .filter(|e| e.is_alive())
            .map(|e| {
                let (x, y) = e.pos();
                let (hp, max_hp) = e.hp();
                let stat = e.stat();
                let (tank, ranged) = crate::magnus_data::tags(&e.name().unwrap_or_default());
                let marker = (0..e.buff_count())
                    .filter_map(|i| e.buff_at(i))
                    .find_map(|b| {
                        b.name()
                            .strip_prefix("mg_owner:")
                            .and_then(|raw| raw.split(':').next()?.parse::<usize>().ok())
                    });
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
                    farmable: crate::farming::eligible(sim,id,e.id()),
                    visible: e.team() == team || sim.is_visible(team, e.id()),
                    targetable: e.is_targetable(),
                    tank,
                    ranged,
                    friendly_chess: marker.is_some_and(|owner| owner != id),
                    sight_source: marker.is_none()
                        && (e.is_champion() || e.is_minion() || e.is_tower()),
                    mate_immune_until: if has(sim, e.id(), "mg_mate_immunity") {
                        tick + 1
                    } else {
                        0
                    },
                    vx: self.observations.get(&e.id()).filter(|(t,_,_)| *t + 1 == tick)
                        .map_or(0, |(_,old,_)| (x as i64 - old).clamp(-(stat.move_speed as i64),stat.move_speed as i64)),
                    vy: self.observations.get(&e.id()).filter(|(t,_,_)| *t + 1 == tick)
                        .map_or(0, |(_,_,old)| (y as i64 - old).clamp(-(stat.move_speed as i64),stat.move_speed as i64)),
                    move_speed: if (0..e.cc_count()).filter_map(|i|e.cc_at(i)).any(|c|
                        [CcKindV1::Stun,CcKindV1::Bind,CcKindV1::Airborne].iter().any(|k|c.kind==k.code())) {0} else {stat.move_speed as i64},
                }
            })
            .collect();
        Some((all.iter().find(|f| f.id == id)?.clone(), all, controlled))
    }
    fn carrier(
        &self,
        sim: &mut StableSim<'_>,
        piece: &Piece,
        x: i64,
        y: i64,
        team: usize,
    ) -> Option<usize> {
        let stat = StatV1 {
            hp: piece.max_hp.max(1) as usize,
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
        let id = sim.spawn_unit(
            "alchemist",
            self.owner,
            team,
            x.max(0) as u64,
            y.max(0) as u64,
            5_184_000,
            &stat,
            &atk,
        )?;
        sim.entity_set_hp(id, piece.hp.max(1) as usize);
        for kind in [CcKindV1::Bind, CcKindV1::BlockAttack, CcKindV1::BlockSkill] {
            sim.apply_cc(id, &CcV1::of_kind(kind, 5_184_000));
        }
        sim.add_buff(
            id,
            &timed(&format!("mg_owner:{}:{}", self.owner, piece.id), 5_184_000),
        );
        Some(id)
    }
    // Allocate every replacement before retiring any current carrier. Failure is atomic.
    fn stage_spawns(
        &self,
        sim: &mut StableSim<'_>,
        core: &Core,
        commands: &[Cmd],
        team: usize,
    ) -> Result<BTreeMap<usize, usize>, ()> {
        let mut staged = BTreeMap::new();
        for c in commands {
            if let Cmd::Spawn { piece, x, y, .. } = c {
                let Some(p) = core.pieces.get(piece) else {
                    continue;
                };
                if let Some(id) = self.carrier(sim, p, *x, *y, team) {
                    staged.insert(*piece, id);
                } else {
                    for id in staged.values() {
                        sim.entity_set_hp(*id, 0);
                    }
                    return Err(());
                }
            }
        }
        Ok(staged)
    }
    fn apply(
        &mut self,
        sim: &mut StableSim<'_>,
        core: &mut Core,
        commands: Vec<Cmd>,
        mut staged: BTreeMap<usize, usize>,
    ) {
        let tick = sim.tick();
        for c in commands {
            match c {
                Cmd::Spawn { piece, .. } => {
                    self.retire(sim, piece);
                    if let Some(id) = staged.remove(&piece) {
                        self.units.insert(piece, id);
                        BOOK.log(&format!(
                            "native {} owner {} piece {piece} spawned entity {id} HP {}",
                            crate::VERSION,
                            self.owner,
                            core.pieces[&piece].hp
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
                    command,
                    piece,
                    target,
                    amount,
                } => {
                    if self.owned(sim, piece).is_none() {
                        continue;
                    }
                    let before = sim
                        .get_entity(target)
                        .map_or(0, |e| e.hp().0.saturating_add(e.shield()));
                    sim.deal_damage(
                        self.owner,
                        target,
                        0,
                        amount.max(0) as usize,
                        AttackTypeV1::Skill,
                    );
                    let after = sim
                        .get_entity(target)
                        .map_or(0, |e| e.hp().0.saturating_add(e.shield()));
                    let actual = before.saturating_sub(after);
                    let receipt=Hit {
                        command,
                        piece,
                        target,
                        amount: actual as i64,
                        status:if actual>0{ReceiptStatus::Applied}else{ReceiptStatus::Pending},
                    };
                    self.hits.push(receipt);
                    if actual==0{self.probes.push((receipt,before,tick));}
                    if self.damage_logs<512 && sim.sim_origin().is_none_or(|o|o.kind!=mod_api_stable::SimOriginKindV1::ServerPresim.code()){
                        self.damage_logs+=1;
                        BOOK.log(&format!("tick {tick} owner {} piece {piece} target {target} requested {amount} immediateDelta {actual} receipt {:?}; queued engine damage",self.owner,receipt.status));
                    }
                }
                Cmd::Stun { target, ticks } => {
                    sim.apply_cc(target, &CcV1::of_kind(CcKindV1::Stun, ticks as u64))
                }
                Cmd::Immune { target, ticks } => {
                    sim.add_buff(target, &timed("mg_mate_immunity", ticks));
                }
                Cmd::Shield {
                    target,
                    amount,
                    ticks,
                } => {
                    let id = target
                        .and_then(|p| self.owned(sim, p))
                        .or_else(|| target.is_none().then_some(self.owner));
                    if let Some(id) = id {
                        sim.entity_add_shield(id, amount.max(0) as usize, ticks);
                    }
                }
                Cmd::Buff {
                    target,
                    kind,
                    ticks,
                } => {
                    let id = target
                        .and_then(|p| self.owned(sim, p))
                        .or_else(|| target.is_none().then_some(self.owner));
                    if let Some(id) = id {
                        let mut buff = timed(&format!("mg_reward_{kind}"), ticks);
                        match kind {
                            4 => buff.damaged_reduce = 25,
                            5 => buff.move_speed_mult = 20,
                            9 => {
                                buff.defence = 20;
                                buff.magic_resistance = 20;
                            }
                            _ => {}
                        }
                        sim.add_buff(id, &buff);
                    }
                }
                Cmd::OwnerPos { x, y } => {
                    sim.entity_set_pos(self.owner, x.max(0) as u64, y.max(0) as u64);
                }
                Cmd::Appearance { piece } => {
                    if let (Some(id), Some(p)) = (self.owned(sim, piece), core.pieces.get(&piece)) {
                        if let Some(old)=self.piece_views.remove(&piece){for name in old{sim.entity_remove_buff(id,&name);}}
                        let max = sim.get_entity(id).map_or(1, |e| e.hp().1);
                        let mut buff = BuffV1::named("mg_promotion");
                        buff.hp =
                            (p.max_hp - max as i64).clamp(i32::MIN as i64, i32::MAX as i64) as i32;
                        sim.add_buff(id, &buff);
                        sim.entity_set_hp(id, p.hp as usize);
                        self.piece_views.remove(&piece);
                        for kind in [
                            Kind::Rook,
                            Kind::Bishop,
                            Kind::Knight,
                            Kind::Queen,
                            Kind::Pawn,
                        ] {
                            sim.entity_remove_buff(id, &format!("mg_piece_{}", kind.name()));
                        }
                    }
                }
                Cmd::Fx { tag, x, y, ticks } => {
                    let name = format!("{ID}_{tag}");
                    if tag.starts_with("label") || tag.starts_with("game") || tag == "brilliant" {
                        fx_unit(sim, &name, self.owner, self.owner, ticks as u64);
                    } else {
                        fx_point(sim, &name, self.owner, x, y, ticks as u64);
                        if tag.starts_with("board") {
                            let frame=format!("{ID}_board_frame{}{}",appearance(self.rank,core.top_position),if tag.ends_with("_fade"){"_fade"}else{""});
                            fx_point(sim,&frame,self.owner,x,y,ticks as u64);
                        }
                    }
                }
                Cmd::Label { code } => BOOK.log(&format!(
                    "tick {tick} owner {} label {code} challenge {:?}",
                    self.owner,
                    core.challenge
                        .as_ref()
                        .map(|c| (c.id, c.progress, c.done, c.failed))
                )),
                Cmd::Failure { reason } => BOOK.log(&format!(
                    "tick {tick} owner {} command failed {reason}",
                    self.owner
                )),
            }
        }
    }
    fn sync(&mut self, sim: &mut StableSim<'_>, core: &Core, input: &Input) {
        let tick = sim.tick();
        let look=appearance(self.rank,core.top_position);
        let badge=if self.rank<7{self.rank}else{6+core.top_position.unwrap_or(10).clamp(1,10)};
        let side=if self.facing.left(){"_l"}else{""};
        let mut want = vec![format!("mg_rank{badge}"),"mg_rank_motion".into()];
        if core.pending.as_ref().is_none_or(|p|matches!(p.action,Action::Deploy{..}|Action::Dismiss)){
            let pose=if tick<self.attack_until{"attack"}else if tick<self.hit_until{"hit"}else if tick<self.moving_until{"run"}else{"idle"};
            want.push(format!("mg_outfit{look}_{pose}{side}"));
        }
        if look>=4{want.push(format!("mg_construction{look}{side}"));}
        if !core.plans.is_empty(){
            let center=core.plan_target.and_then(|id|input.fighters.iter().find(|f|f.id==id&&f.visible&&f.hp>0)).map_or(input.owner.cell(),Fighter::cell);
            let mut marks=BTreeSet::new();
            for(branch,path)in core.plans.iter().take(if look==10{3}else{1}).enumerate(){
                for(step,action)in path.iter().take(4).enumerate(){
                    let to=match action{Action::Move{to,..}|Action::Summon{to,..}=>Some(*to),Action::Castle{piece}=>core.pieces.get(piece).map(|p|p.cell),_=>None};
                    if let Some(to)=to{
                        let(x,y)=(to.x-center.x+2,3-(to.y-center.y+2));let clipped=!(0..4).contains(&x)||!(0..4).contains(&y);let cell=y.clamp(0,3)*4+x.clamp(0,3);
                        marks.insert(format!("mg_plan_mark{}_{}_{}{side}",usize::from(branch>0),step,cell));
                        if clipped{marks.insert(format!("mg_plan_edge_{cell}{side}"));}
                    }
                }
            }
            if !marks.is_empty(){want.push(format!("mg_plan_panel{side}"));want.extend(marks);}
        }
        if let Some(c) = &core.challenge {
            if !c.done && !c.failed {
                want.push(format!("mg_clue{}", c.id));
            }
        }
        if let Some(b) = core.board {
            for p in &core.projections {
                if let Some(f) = input
                    .fighters
                    .iter()
                    .find(|f| f.id == p.id && f.hp > 0 && f.visible && b.inside(f.x, f.y))
                {
                    let cell = b.cell(f.x, f.y);
                    want.push(format!(
                        "mg_mini_{}_{}_{}",
                        if self.facing.left() { "l" } else { "r" },
                        p.role.code(),
                        cell.y * 8 + cell.x
                    ));
                }
            }
        }
        let cast = core.pending.as_ref().map(|p| p.start);
        if cast != self.cast_start {
            for n in &self.visuals {
                if n.starts_with("mg_cast") {
                    sim.entity_remove_buff(self.owner, n);
                }
            }
            self.visuals.retain(|n| !n.starts_with("mg_cast"));
            self.cast_start = cast;
        }
        if let Some(p) = &core.pending {
            if !matches!(p.action, Action::Deploy { .. } | Action::Dismiss) {
                let queen = matches!(p.action,Action::Move{piece,..}if core.pieces.get(&piece).is_some_and(|p|p.kind==Kind::Queen));
                want.push(format!(
                    "mg_cast{look}{}{}{}",
                    if queen { "_queen" } else { "" },
                    if p.end - p.start == 24 || p.end - p.start == 36 {
                        "_fast"
                    } else {
                        ""
                    },
                    if self.facing.left() { "_l" } else { "" }
                ));
            }
        }
        diff_buffs(sim, self.owner, &mut self.visuals, want);
        let icons = core
            .projections
            .iter()
            .filter(|p| {
                input
                    .fighters
                    .iter()
                    .any(|f| f.id == p.id && f.hp > 0 && f.visible)
            })
            .map(|p| {
                (
                    p.id,
                    format!("mg_enemy{}_{}", self.player % 16, p.role.code()),
                )
            })
            .collect::<Vec<_>>();
        for (id, n) in &self.icons {
            if !icons.contains(&(*id, n.clone())) {
                sim.entity_remove_buff(*id, n);
            }
        }
        for (id, n) in &icons {
            if !self.icons.contains(&(*id, n.clone())) {
                sim.add_buff(*id, &timed(n, 5_184_000));
            }
        }
        self.icons = icons;
        for p in core.pieces.values() {
            if let Some(id) = self.owned(sim, p.id) {
                let (hp, max) = sim.get_entity(id).map_or((0, 1), |e| e.hp());
                let mut tags = vec![
                    format!("mg_piece_{}_{look}", p.kind.name()),
                    format!("mg_hp{}", (hp * 10 / max.max(1)).min(10)),
                    format!("mg_identity{}", p.slot.unwrap_or(7)),
                ];
                if core
                    .board
                    .is_none_or(|b| !b.inside(p.cell.world().0, p.cell.world().1))
                {
                    tags.push("mg_weakened".into());
                }
                if tick < core.glow_until && core.glow.contains(&p.id) {
                    tags.push("mg_glow".into());
                }
                let old = self.piece_views.entry(p.id).or_default();
                diff_buffs(sim, id, old, tags);
            }
        }
        let mut coverage = vec![];
        if let Some(b) = core.board {
            for n in 0..64 {
                let c = b.grid(Cell::new(n % 8, n / 8));
                if let Some(p) = core
                    .pieces
                    .values()
                    .find(|p| core.threatens(p.id, c, input))
                {
                    coverage.push((
                        p.id,
                        format!("mg_cov_{}_{}", c.x - p.cell.x + 7, c.y - p.cell.y + 7),
                    ));
                }
            }
        }
        for (p, n) in &self.coverage {
            if !coverage.contains(&(*p, n.clone())) {
                if let Some(id) = self.owned(sim, *p) {
                    sim.entity_remove_buff(id, n);
                }
            }
        }
        for (p, n) in &coverage {
            if !self.coverage.contains(&(*p, n.clone())) {
                if let Some(id) = self.owned(sim, *p) {
                    sim.add_buff(id, &timed(n, 5_184_000));
                }
            }
        }
        self.coverage = coverage;
    }
    fn sync_zones(&mut self, sim: &mut StableSim<'_>, core: &Core) {
        let tick = sim.tick();
        let until = core.pending.as_ref().map_or_else(
            || core.mate_warning.map_or(tick, |t| t + MATE_WARN),
            |p| p.end,
        );
        let want = core
            .warning_cells()
            .iter()
            .map(|c| format!("mgw:{}:{}:{until}", c.x, c.y))
            .collect();
        diff_buffs(sim, self.owner, &mut self.zones, want);
        if !self.zones.is_empty() {
            crate::steve::note_map_buff(sim.seed());
        }
    }
}
fn diff_buffs(sim: &mut StableSim<'_>, id: usize, old: &mut Vec<String>, want: Vec<String>) {
    for name in old.iter() {
        if !want.contains(name) {
            sim.entity_remove_buff(id, name);
        }
    }
    for name in &want {
        if !old.contains(name) {
            sim.add_buff(id, &timed(name, 5_184_000));
        }
    }
    *old = want;
}
impl StablePassive for Magnus {
    fn clone_box(&self) -> Box<dyn StablePassive> {
        Box::new(self.clone())
    }
    fn on_attack(&mut self,sim:&mut StableSim<'_>,_:usize,_:usize,_:usize,_:&mut usize){self.attack_until=sim.tick()+12;}
    fn on_damaged(&mut self,sim:&mut StableSim<'_>,_:usize,_:usize,_:usize,damage:usize){if damage>0{self.hit_until=sim.tick()+12;}}
    fn configure(&mut self, params: &str) {
        let p = params
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect::<String>();
        self.probe = if p.contains("\"carrier_probe\":1") {
            3
        } else if p.contains("\"knight_probe\":1") {
            1
        } else if p.contains("\"pawn_probe\":1") {
            2
        } else {
            0
        };
    }
    fn on_dead(&mut self, sim: &mut StableSim<'_>, _: usize) {
        if self.probe > 0 {
            BOOK.log(&format!(
                "probe owner {} death cleanup: {} owned carriers",
                self.owner,
                self.units.len()
            ));
        }
        if let Some(mut core) = self.core.take() {
            let commands = core.on_dead();
            self.apply(sim, &mut core, commands, BTreeMap::new());
            self.core = Some(core);
        }
        for n in self.visuals.drain(..).chain(self.zones.drain(..)) {
            sim.entity_remove_buff(self.owner, &n);
        }
        for (id, n) in self.icons.drain(..) {
            sim.entity_remove_buff(id, &n);
        }
        for n in ["mg_reward_5", "mg_reward_9"] {
            sim.entity_remove_buff(self.owner, n);
        }
        self.hits.clear();
        self.probes.clear();
        self.observations.clear();
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
        self.observations = all.iter().filter(|f| f.visible)
            .map(|f| (f.id,(tick,f.x,f.y))).collect();
        if self.last_pos.is_some_and(|(x,y)|(me.x-x).abs().max((me.y-y).abs())>200){self.moving_until=tick+6;}
        self.last_pos=Some((me.x,me.y));
        self.athlete = BOOK.athlete_of(sim.seed(), player);
        let (rank,top)=BOOK.pinned(sim.seed()).rank_for(self.athlete);
        self.rank=rank.min(7);
        let enemy = all
            .iter()
            .filter(|f| f.team != me.team && f.champion && f.visible)
            .min_by_key(|f| ((f.x - me.x).pow(2) + (f.y - me.y).pow(2), f.id))
            .map(|f| (f.x, f.y));
        self.facing.update(tick, (me.x, me.y), enemy);
        let mut core = self
            .core
            .take()
            .unwrap_or_else(|| Core::new(sim.seed(), entity, self.rank));
        core.rank = self.rank;
        core.top_position=top;
        core.owner = entity;
        let mut hp = vec![];
        let mut captures = vec![];
        for (piece, host) in &self.units {
            if let Some(id) = self.owned(sim, *piece) {
                hp.push((*piece, sim.get_entity(id).map_or(0, |e| e.hp().0) as i64));
            } else if core.pieces.contains_key(piece) {
                captures.push(Capture {
                    piece: *piece,
                    by: None,
                    combat: sim.get_entity(*host).is_none_or(|e| !e.is_alive()),
                });
            }
        }
        let champs = champions(sim);
        let dynamic = crate::steve::walls_up(&champs, tick);
        let smokes = crate::valorant::smokes_up(&champs, tick);
        let wall = |x: i64, y: i64| {
            walls::wall_at(x, y)
                || dynamic.iter().any(|w| {
                    crate::steve::crosses(w, tick, x - 1000, y, x + 1000, y)
                        || crate::steve::crosses(w, tick, x, y - 1000, x, y + 1000)
                })
        };
        let sight = |ax: i64, ay: i64, bx: i64, by: i64| {
            let steps = (((bx - ax).abs().max((by - ay).abs()) / 2000) + 1).max(1);
            (0..=steps).all(|n| {
                let (x, y) = (ax + (bx - ax) * n / steps, ay + (by - ay) * n / steps);
                !wall(x, y)
                    && !smokes.iter().any(|s| {
                        (x - s.x).pow(2) + (y - s.y).pow(2) <= crate::valorant::SMOKE_R.pow(2)
                    })
            })
        };
        let mut receipts = std::mem::take(&mut self.hits);
        for (mut hit,before,at) in std::mem::take(&mut self.probes){
            // The engine drains its damage queue after the callback. A later HP
            // delta can include unrelated attacks/healing and is not a hit receipt.
            if tick>at {
                let after=sim.get_entity(hit.target).map_or(before,|e|e.hp().0.saturating_add(e.shield()));
                hit.amount=before.saturating_sub(after) as i64;
                hit.status=ReceiptStatus::Unknown;
                if self.damage_logs<512 && sim.sim_origin().is_none_or(|o|o.kind!=mod_api_stable::SimOriginKindV1::ServerPresim.code()){
                    self.damage_logs+=1;
                    BOOK.log(&format!("tick {tick} command {} owner {} piece {} target {} observedDelta {} receipt Unknown (uncorrelated)",hit.command,self.owner,hit.piece,hit.target,hit.amount));
                }
                receipts.push(hit);
            } else {self.probes.push((hit,before,at));}
        }
        let mut action = None;
        if self.probe > 0 && core.pending.is_none() {
            let kind = if self.probe == 1 {
                Kind::Knight
            } else {
                Kind::Pawn
            };
            if core.stats.summons < if self.probe == 3 { 2 } else { 1 } {
                let kind = if self.probe == 3 && core.stats.summons == 0 {
                    Kind::Knight
                } else {
                    kind
                };
                let offset = if core.stats.summons == 0 { 1 } else { -1 };
                action = Some(Action::Summon {
                    kind,
                    to: Cell::new(me.cell().x + offset, me.cell().y),
                    direction: 0,
                });
            } else if let Some(p) = core
                .pieces
                .values()
                .nth((tick / 30) % core.pieces.len().max(1))
            {
                let target = all
                    .iter()
                    .filter(|f| f.team != me.team && f.visible && f.champion)
                    .min_by_key(|f| p.cell.distance(f.cell()))
                    .map_or(Cell::new(me.cell().x, me.cell().y + 2), Fighter::cell);
                action = core
                    .destinations(
                        p.id,
                        &Input {
            geometry: None,
                            tick,
                            owner: &me,
                            fighters: &all,
                            wall: &wall,
                            sight: &sight,
                            captures: &captures,
                            hits: &receipts,
                            piece_hp: &hp,
                            controlled,
                            automatic: false,
                            ult_unlocked: has(sim, entity, "mg_ult_learned"),
                            action: None,
                        },
                    )
                    .into_iter()
                    .min_by_key(|to| (to.distance(target), *to))
                    .map(|to| Action::Move { piece: p.id, to });
            } else if core.stats.summons == 0 {
                action = Some(Action::Summon {
                    kind,
                    to: Cell::new(me.cell().x + 1, me.cell().y),
                    direction: 0,
                });
            }
        }
        let input = Input {
            geometry: None,
            tick,
            owner: &me,
            fighters: &all,
            wall: &wall,
            sight: &sight,
            captures: &captures,
            hits: &receipts,
            piece_hp: &hp,
            controlled,
            automatic: self.probe == 0,
            ult_unlocked: has(sim, entity, "mg_ult_learned"),
            action,
        };
        let before = core.clone();
        let commands = core.step(&input);
        match self.stage_spawns(sim, &core, &commands, me.team) {
            Ok(staged) => self.apply(sim, &mut core, commands, staged),
            Err(()) => {
                core = before;
                core.pending = None;
                let recovery = core.step(&Input {
            geometry: None,
                    automatic: false,
                    action: None,
                    ..input
                });
                self.apply(sim, &mut core, recovery, BTreeMap::new());
                core.last_failure = 3;
                BOOK.log("Spawn capacity failure: kept the previous board and army; summon/deployment cooldown refunded.");
            }
        }
        self.sync_zones(sim, &core);
        if self.probe > 0 && tick.is_multiple_of(60) {
            for (piece, host) in &self.units {
                if let Some(e) = sim.get_entity(*host) {
                    BOOK.log(&format!("probe tick {tick} piece {piece} kind {:?} host {host} HP {:?} pos {:?} targetable {} alive {}",core.pieces.get(piece).map(|p|p.kind),e.hp(),e.pos(),e.is_targetable(),e.is_alive()));
                }
            }
        }
        if tick.is_multiple_of(6) {
            self.sync(sim, &core, &input);
        }
        if self.sig.is_none() && tick >= 1800 {
            self.sig = Some(signature(&BOOK, sim, &champs, self.athlete));
        }
        if let (Some(sig), Some(a)) = (&self.sig, self.athlete) {
            if tick.is_multiple_of(30) {
                self.record.track(&BOOK, sim, sig, me.team, a, tick);
            }
        }
        self.core = Some(core);
    }
}
/// Only the shared movement hook consumes these warnings; it never replaces casts or attacks.
pub fn avoid(
    all: &[crate::Champ],
    me: &crate::Champ,
    dest: (i64, i64),
    tick: usize,
) -> Option<(i64, i64)> {
    let mut dangerous = BTreeSet::new();
    for foe in all.iter().filter(|f| f.team != me.team) {
        for buff in &foe.buffs {
            if let Some(raw) = buff.name().strip_prefix("mgw:") {
                let a: Vec<_> = raw.split(':').collect();
                if a.len() == 3 {
                    if let (Ok(x), Ok(y), Ok(until)) =
                        (a[0].parse(), a[1].parse(), a[2].parse::<usize>())
                    {
                        if tick < until {
                            dangerous.insert(Cell::new(x, y));
                        }
                    }
                }
            }
        }
    }
    let from = Cell::at(me.x, me.y);
    let to = Cell::at(dest.0, dest.1);
    if !dangerous.contains(&from) && !dangerous.contains(&to) {
        return None;
    }
    let center = if dangerous.contains(&from) { from } else { to };
    (-2..=2)
        .flat_map(|x| (-2..=2).map(move |y| Cell::new(center.x + x, center.y + y)))
        .filter(|c| !dangerous.contains(c))
        .map(Cell::world)
        .filter(|(x, y)| !walls::wall_at(*x, *y))
        .min_by_key(|(x, y)| ((x - dest.0).pow(2) + (y - dest.1).pow(2), *x, *y))
}
