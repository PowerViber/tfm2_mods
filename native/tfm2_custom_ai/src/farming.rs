//! Farming eligibility shared by native kits; never injects engine input orders.
use crate::{d2, sq, Champ};
use mod_api_stable::{StableDraftContext, StableDraftDecision, StableDraftHook, StableSim};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    OnceLock,
};

/// A wave is reserved for its collector only once the collector is this close to it;
/// until then allies already on the wave keep clearing it.
pub const RESERVE_RADIUS: i64 = 60000;
/// A jungler with at least this much judgement may open an untouched Epic or Serpen
/// without a team assignment.
pub const BOLD_JUDGEMENT: i64 = 80;

/// Read-only, callback-local facts; never retained across a world mutation.
/// Team facts (champions, wave reservations, the safety window) are read once per
/// context, so a kit snapshot builds one and asks it about every unit.
pub struct Context<'a, 's> {
    sim: &'a StableSim<'s>,
    owner: usize,
    team: Option<usize>,
    jungle: std::cell::OnceCell<bool>,
    bold: std::cell::OnceCell<bool>,
    calm: std::cell::OnceCell<bool>,
    champions: std::cell::OnceCell<Vec<(usize, usize, bool, (u64, u64))>>,
    reservations: std::cell::OnceCell<Vec<(usize, (i64, i64))>>,
}
impl<'a, 's> Context<'a, 's> {
    pub fn new(sim: &'a StableSim<'s>, owner: usize) -> Self {
        Self {
            sim,
            owner,
            team: sim.get_entity(owner).map(|e| e.team()),
            jungle: std::cell::OnceCell::new(),
            bold: std::cell::OnceCell::new(),
            calm: std::cell::OnceCell::new(),
            champions: std::cell::OnceCell::new(),
            reservations: std::cell::OnceCell::new(),
        }
    }
    /// Allied collectors that hold a wave: (collector, wave point), in champion order.
    fn reservations(&self) -> &[(usize, (i64, i64))] {
        self.reservations.get_or_init(|| {
            let Some(team) = self.team else {
                return vec![];
            };
            self.champions()
                .iter()
                .filter(|a| a.1 == team && a.2)
                .filter_map(|a| {
                    crate::universal_ai::assignment(self.sim, a.0)
                        .filter(|(g, j, _, point, score)| {
                            *g == crate::strategic_core::Goal::Wave
                                && *j == crate::strategic_core::Job::CatchWave as u8
                                && *score > 0
                                && d2(a.3 .0 as i64, a.3 .1 as i64, point.0, point.1)
                                    <= sq(RESERVE_RADIUS)
                        })
                        .map(|(_, _, _, point, _)| (a.0, point))
                })
                .collect()
        })
    }
    /// The owner's safety window (`window`), read once.
    pub fn calm(&self) -> bool {
        *self.calm.get_or_init(|| {
            let heroes = crate::champions(self.sim);
            heroes
                .iter()
                .find(|c| c.id == self.owner)
                .is_some_and(|me| window(self.sim, me, &heroes))
        })
    }
    /// Eligible and inside the safety window: what a kit may farm on its own initiative.
    pub fn farmable(&self, target: usize) -> bool {
        self.eligible(target) && self.calm()
    }
    fn champions(&self) -> &[(usize, usize, bool, (u64, u64))] {
        self.champions.get_or_init(|| {
            (0..self.sim.champion_count())
                .filter_map(|i| self.sim.get_entity(self.sim.champion_id_at(i)))
                .map(|e| (e.id(), e.team(), e.is_alive(), e.pos()))
                .collect()
        })
    }
    pub fn eligible(&self, target: usize) -> bool {
        use mod_api_stable::{EntityKindV1 as K, LaneV1};
        let Some(team) = self.team else {
            return false;
        };
        let Some(e) = self.sim.get_entity(target) else {
            return false;
        };
        if e.team() == team
            || !e.is_alive()
            || !e.is_targetable()
            || !self.sim.is_visible(team, target)
        {
            return false;
        }
        let kind = e.kind();
        if kind == Some(K::Minion) {
            // One collector owns an assigned wave. Champion-targeted fighting and
            // its incidental splash keep their existing rules.
            let p = e.pos();
            let reserved = self
                .reservations()
                .iter()
                .find(|(_, point)| d2(p.0 as i64, p.1 as i64, point.0, point.1) <= sq(45000))
                .map(|r| r.0);
            if reserved.is_some_and(|id| id != self.owner) {
                return false;
            }
        }
        if !matches!(kind, Some(K::Jungle | K::Epic | K::Serpen)) {
            let fallback =
                kind.is_none() && e.is_minion() && self.champions().iter().any(|c| c.1 == e.team());
            return accepts(kind, false, false, false, fallback);
        }
        let jungle = *self.jungle.get_or_init(|| {
            (0..self.sim.player_count())
                .filter_map(|i| self.sim.player_at(i))
                .find(|p| p.champion().is_some_and(|c| c.id() == self.owner))
                .and_then(|p| p.lane())
                == Some(LaneV1::Jungle)
        });
        let active = matches!(kind, Some(K::Epic | K::Serpen))
            && (0..e.buff_count())
                .filter_map(|i| e.buff_at(i))
                .any(|b| b.name() == format!("farm_objective_active:{team}"));
        let ally = active
            && self.champions().iter().any(|a| {
                a.0 != self.owner && a.1 == team && a.2 && {
                    let (ex, ey) = e.pos();
                    d2(a.3 .0 as i64, a.3 .1 as i64, ex as i64, ey as i64) <= sq(64000)
                }
            });
        let objective = matches!(kind, Some(K::Epic | K::Serpen));
        accepts(kind, jungle, active, ally, false)
            || objective && crate::universal_ai::objective_assignment(self.sim, self.owner, target)
            || objective && jungle && *self.bold.get_or_init(|| {
                crate::universal_ai::profile(self.sim, self.owner).judgement() >= BOLD_JUDGEMENT
            })
    }
}
/// One-shot check; callers asking about many targets should keep a `Context`.
pub fn eligible(sim: &StableSim<'_>, owner: usize, target: usize) -> bool {
    Context::new(sim, owner).eligible(target)
}
pub(crate) fn accepts(
    kind: Option<mod_api_stable::EntityKindV1>,
    jungle: bool,
    active: bool,
    ally: bool,
    lane_fallback: bool,
) -> bool {
    use mod_api_stable::EntityKindV1 as K;
    match kind {
        Some(K::Minion | K::Summon) => true,
        Some(K::Jungle) => jungle,
        // Opening an untouched objective needs a team assignment (or a bold jungler,
        // see `Context::eligible`); once it is contested the jungler or a pair may join.
        Some(K::Epic | K::Serpen) => active && (jungle || ally),
        None => lane_fallback,
        _ => false,
    }
}

pub fn observe_objectives(sim: &mut StableSim<'_>) {
    use mod_api_stable::EntityKindV1 as K;
    let ids: Vec<_> = (0..sim.entity_count())
        .filter_map(|i| sim.entity_at(i))
        .filter(|e| matches!(e.kind(), Some(K::Epic | K::Serpen)))
        .map(|e| e.id())
        .collect();
    for id in ids { for team in 0..2 {
        if !sim.is_visible(team,id){continue;}
        let Some(e) = sim.get_entity(id) else {
            continue;
        };
        let (hp,max_hp) = e.hp();
        let prefix=format!("farm_hp:{team}:");
        let old = (0..e.buff_count())
            .filter_map(|i| e.buff_at(i))
            .find_map(|b| {
                b.name()
                    .strip_prefix(&prefix)
                    .and_then(|s| s.parse::<usize>().ok())
                    .map(|n| (b.name().to_string(), n))
            });
        let first=old.is_none();
        if let Some((name, previous)) = old {
            if hp == previous && !sim.tick().is_multiple_of(60) {
                continue;
            }
            sim.entity_remove_buff(id, &name);
            if hp < previous {
                let marker=format!("farm_objective_active:{team}");
                sim.entity_remove_buff(id, &marker);
                sim.add_buff(id, &crate::timed(&marker, 60));
            }
        }
        if hp<max_hp&&first{let marker=format!("farm_objective_active:{team}");sim.add_buff(id,&crate::timed(&marker,60));}
        sim.add_buff(id, &crate::timed(&format!("{prefix}{hp}"), 120));
    }}
}
#[cfg(test)]
pub fn safe(hp: usize, max_hp: usize, controlled: bool, nearby_visible_enemy: bool) -> bool {
    max_hp > 0
        && hp.saturating_mul(100) >= max_hp.saturating_mul(35)
        && !controlled
        && !nearby_visible_enemy
}

/// A nearby opponent is contextual danger, rather than an unconditional PvE lock.
pub(crate) fn window(sim:&StableSim<'_>,me:&Champ,heroes:&[Champ])->bool {
    if me.max_hp==0||me.hp*100<me.max_hp*35||me.stunned||me.pushed{return false;}
    let enemies=heroes.iter().filter(|e|e.team!=me.team&&sim.is_visible(me.team,e.id)&&d2(me.x,me.y,e.x,e.y)<=sq(60000)).count();
    let allies=heroes.iter().filter(|a|a.team==me.team&&a.id!=me.id&&a.hp*100>=a.max_hp*35&&!a.stunned&&d2(me.x,me.y,a.x,a.y)<=sq(70000)).count();
    enemies==0 || crate::universal_ai::assignment(sim,me.id).is_some_and(|(g,_,_,_,score)|score>0&&matches!(g,crate::strategic_core::Goal::Start|crate::strategic_core::Goal::Finish|crate::strategic_core::Goal::Contest)&&enemies<=allies+1)
}

pub(crate) fn nearby(sim: &StableSim<'_>, me: &Champ, heroes: &[Champ], range: i64) -> Vec<Champ> {
    if !window(sim,me,heroes) {
        return vec![];
    }
    let context = Context::new(sim, me.id);
    let mut targets: Vec<_> = crate::units(sim)
        .into_iter()
        .filter(|(id, team, x, y)| {
            *team != me.team && d2(me.x, me.y, *x, *y) <= sq(range) && context.eligible(*id)
        })
        .filter_map(|(id, team, x, y)| {
            let e = sim.get_entity(id)?;
            if !e.is_targetable() {
                return None;
            }
            let (hp, max_hp) = e.hp();
            Some(Champ {
                id,
                team,
                x,
                y,
                hp,
                max_hp,
                attack: e.stat().attack,
                name: e.name().unwrap_or_default(),
                buffs: (0..e.buff_count()).filter_map(|i| e.buff_at(i)).collect(),
                stunned: false,
                pushed: false,
            })
        })
        .collect();
    let objective = preferred_objective(sim, me.id);
    targets.sort_by_key(|e| (Some(e.id) != objective, d2(me.x, me.y, e.x, e.y), e.hp, e.id));
    targets.truncate(32);
    targets
}

/// The objective this champion was told to contest or finish; it goes before nearer units.
pub(crate) fn preferred_objective(sim: &StableSim<'_>, owner: usize) -> Option<usize> {
    use crate::strategic_core::Goal;
    crate::universal_ai::assignment(sim, owner)
        .filter(|(g, j, _, _, score)| *score > 0 && *j <= 1 && matches!(g, Goal::Contest | Goal::Finish))
        .map(|a| a.2)
}

pub(crate) fn for_owner(sim: &StableSim<'_>, owner: usize, range: i64) -> Vec<Champ> {
    let heroes = crate::champions(sim);
    heroes
        .iter()
        .find(|c| c.id == owner)
        .map_or_else(Vec::new, |me| nearby(sim, me, &heroes, range))
}

/// Finish a cast the engine has already committed; do not re-run farming eligibility.
pub(crate) fn marked_enemy(sim: &StableSim<'_>, me: &Champ, marker: &str) -> Option<Champ> {
    crate::units(sim)
        .into_iter()
        .filter(|(id, team, _, _)| *team != me.team && sim.is_visible(me.team, *id))
        .filter_map(|(id, team, x, y)| {
            let e = sim.get_entity(id)?;
            if !e.is_targetable() {
                return None;
            }
            let buffs: Vec<_> = (0..e.buff_count()).filter_map(|i| e.buff_at(i)).collect();
            if !buffs.iter().any(|b| b.name() == marker) {
                return None;
            }
            let (hp, max_hp) = e.hp();
            Some(Champ {
                id,
                team,
                x,
                y,
                hp,
                max_hp,
                attack: e.stat().attack,
                name: e.name().unwrap_or_default(),
                buffs,
                stunned: false,
                pushed: false,
            })
        })
        .min_by_key(|e| (d2(me.x, me.y, e.x, e.y), e.id))
}

static ENABLED: OnceLock<bool> = OnceLock::new();
static LINES: AtomicUsize = AtomicUsize::new(0);
static FARM_LINES: AtomicUsize = AtomicUsize::new(0);
fn diagnostics() -> bool {
    *ENABLED
        .get_or_init(|| crate::mod_dir().is_some_and(|p| p.join("ai_diagnostics.enabled").exists()))
}
pub fn decision(sim:&StableSim,team:usize,p:&crate::strategic_core::Plan){
 if diagnostics()&&FARM_LINES.fetch_add(1,Ordering::Relaxed)<4096{
  let origin=sim.sim_origin().unwrap_or_default();
  crate::magnus::BOOK.log(&format!("team-plan v1 match {} set {} origin {} seed {} tick {} team {team} caller {} goal {:?} target {} score {} reason {:?} jobs {:?}",origin.match_id,origin.set_index,origin.kind,sim.seed(),sim.tick(),p.caller,p.goal,p.target,p.score,p.reason,p.jobs));
 }
}
pub fn planning_timing(sim:&StableSim,caller:usize,team:usize,snapshot:u128,planning:u128,host:u128){
 static TIMINGS:AtomicUsize=AtomicUsize::new(0);
 if crate::perf::enabled()&&TIMINGS.fetch_add(1,Ordering::Relaxed)<4096{
  let origin=sim.sim_origin().unwrap_or_default();
  crate::magnus::BOOK.log(&format!("universal-perf v1 sim {} origin {} match {} set {} seed {} tick {} caller {caller} team {team} snapshot_route_ns {snapshot} planning_ns {planning} host_ns {host} visual_ns 0",sim.instance(),origin.kind,origin.match_id,origin.set_index,sim.seed(),sim.tick()));
 }
}
pub struct DraftObserver;
/// Read-only observations of the engine's own farming casts on either team.
/// Never creates, holds, or replaces an input order.
#[derive(Clone, Default)]
pub struct FarmObserver {
    last: Option<usize>,
    lines: usize,
    pending:Vec<(usize,usize,usize,mod_api_stable::SkillSlotV1,usize,usize)>,
}
impl mod_api_stable::StablePlayerAi for FarmObserver {
    fn clone_box(&self) -> Box<dyn mod_api_stable::StablePlayerAi> {
        Box::new(self.clone())
    }
    fn id(&self) -> String {
        "tfm2_custom_ai:farm_observer".into()
    }
    fn matches(&self, _: &mod_api_stable::StableAiInit) -> bool {
        diagnostics()
    }
    fn think(
        &mut self,
        ctx: &mut mod_api_stable::StableAiContext<'_>,
        base: Option<mod_api_stable::InputV1>,
    ) -> Option<mod_api_stable::InputV1> {
        // A selected input is not cast acceptance. Observe its own engine cooldown
        // in a later callback; health deltas alone cannot prove attributed delivery.
        let player_id=ctx.player_id();
        if let Some(sim)=ctx.sim(){
          if let Some(me)=sim.get_player(player_id).and_then(|p|p.champion()){
           self.pending.retain(|(start,target,before,slot,remaining,life)|{
            let now=me.cooldown(*slot).map_or(0,|c|c.remaining);
            if sim.tick()>*start&&now>*remaining{
             if FARM_LINES.fetch_add(1,Ordering::Relaxed)<4096{
              let hp=sim.get_entity(*target).map_or(0,|e|e.hp().0);
              crate::magnus::BOOK.log(&format!("farm stage=executed evidence=cooldown-consumed tick {} player {player_id} target {target} selected_tick {start} target_hp_delta_unattributed {} delivered=unverified",sim.tick(),before.saturating_sub(hp)));
             }false
            }else{sim.tick()<=*life}
           });
          }
        }
        let input = base?;
        if FARM_LINES.load(Ordering::Relaxed) >= 4096
            || self.lines >= 128
            || self.last.is_some_and(|t| ctx.tick().saturating_sub(t) < 30)
            || input.kind < mod_api_stable::InputKindV1::Attack.code()
            || input.target.kind != mod_api_stable::InputTargetKindV1::Target.code()
        {
            return None;
        }
        let (tick, player_id, name) = (ctx.tick(), ctx.player_id(), ctx.champion_name());
        let sim = ctx.sim()?;
        let player = sim.get_player(player_id)?;
        let me = player.champion()?;
        let target = sim.get_entity(input.target.target_id)?;
        if target.team() != me.team()
            && !target.is_champion()
            && !target.is_tower()
            && target.is_alive()
            && sim.is_visible(me.team(), target.id())
            && FARM_LINES.fetch_add(1, Ordering::Relaxed) < 4096
        {
            self.last = Some(tick);
            self.lines += 1;
            crate::magnus::BOOK.log(&format!("farm engine-input tick {tick} player {player_id} team {} champion {name:?} kind {} target {} minion {}",me.team(),input.kind,target.id(),target.is_minion()));
            let eligible=eligible(&sim,me.id(),target.id());
            crate::magnus::BOOK.log(&format!("farm stage=eligible result={eligible} stage=selected tick {tick} player {player_id} target {}",target.id()));
            let slot=match mod_api_stable::InputKindV1::from_code(input.kind){Some(mod_api_stable::InputKindV1::Skill)=>Some(mod_api_stable::SkillSlotV1::Skill),Some(mod_api_stable::InputKindV1::Skill2)=>Some(mod_api_stable::SkillSlotV1::Skill2),Some(mod_api_stable::InputKindV1::Ult)=>Some(mod_api_stable::SkillSlotV1::Ult),_=>None};
            if let Some(slot)=slot{if self.pending.len()<8{let cd=me.cooldown(slot).map_or(0,|c|c.remaining);self.pending.push((tick,target.id(),target.hp().0,slot,cd,tick+180));}}
        }
        None
    }
}
impl StableDraftHook for DraftObserver {
    fn id(&self) -> String {
        "tfm2_custom_ai:draft_observer".into()
    }
    fn score_pick(
        &self,
        ctx: &StableDraftContext<'_>,
        candidate: usize,
        base: f32,
    ) -> StableDraftDecision {
        if diagnostics() && LINES.fetch_add(1, Ordering::Relaxed) < 4096 {
            crate::magnus::BOOK.log(&format!("draft phase {:?} candidate {} name {:?} available {} base {} explore {} allies {:?} enemies {:?}",
                ctx.phase(),candidate,ctx.champion_name(candidate),ctx.available_champions().contains(&candidate),base,ctx.is_explore(),ctx.ally_picks(),ctx.enemy_picks()));
        }
        StableDraftDecision::Pass
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn farming_yields_to_danger_and_low_health() {
        assert!(safe(350, 1000, false, false));
        assert!(!safe(349, 1000, false, false));
        assert!(!safe(900, 1000, true, false));
        assert!(!safe(900, 1000, false, true));
        assert!(!safe(1, 0, false, false));
    }
}

#[cfg(test)]
mod role_tests {
    use super::*;
    use mod_api_stable::EntityKindV1 as K;
    #[test]
    fn lane_roles_never_start_ordinary_camps_and_objective_assistance_requires_both_signals() {
        for jungle in [false, true] {
            for active in [false, true] {
                for ally in [false, true] {
                    assert!(accepts(Some(K::Minion), jungle, active, ally, false));
                    assert!(accepts(Some(K::Summon), jungle, active, ally, false));
                    assert_eq!(
                        accepts(Some(K::Jungle), jungle, active, ally, false),
                        jungle
                    );
                    for kind in [K::Epic, K::Serpen] {
                        assert_eq!(
                            accepts(Some(kind), jungle, active, ally, false),
                            active && (jungle || ally)
                        );
                    }
                    for kind in [K::Champion, K::Tower, K::Nexus] {
                        assert!(!accepts(Some(kind), jungle, active, ally, false));
                    }
                }
            }
        }
        assert!(!accepts(None, true, true, true, false));
        assert!(accepts(None, false, false, false, true));
    }
}

#[cfg(test)]
#[path = "farming_rules_tests.rs"]
mod farming_rules_tests;
