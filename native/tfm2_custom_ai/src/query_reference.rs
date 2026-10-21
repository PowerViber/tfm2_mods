// Frozen reference from main 332b4d4 (wave reservation, per-team objective marker, assigned objectives), test-only.
use crate::farming::accepts;
use crate::{d2, sq};
/// Role policy for intentional native farming. Incidental area damage is unchanged.
pub fn reference_eligible(sim: &StableSim<'_>, owner: usize, target: usize) -> bool {
    use mod_api_stable::{EntityKindV1 as K, LaneV1};
    let Some(me) = sim.get_entity(owner) else {
        return false;
    };
    let Some(e) = sim.get_entity(target) else {
        return false;
    };
    if e.team() == me.team()
        || !e.is_alive()
        || !e.is_targetable()
        || !sim.is_visible(me.team(), target)
    {
        return false;
    }
    let kind = e.kind();
    if kind==Some(K::Minion){
        let p=e.pos();
        // One collector owns an assigned wave. Champion-targeted fighting and
        // its incidental splash keep their existing rules.
        let reserved=(0..sim.champion_count()).filter_map(|n|sim.get_entity(sim.champion_id_at(n))).filter(|a|a.team()==me.team()&&a.is_alive()).find_map(|a|{
            crate::universal_ai::assignment(sim,a.id()).filter(|(g,j,_,point,score)|*g==crate::strategic_core::Goal::Wave&&*j==crate::strategic_core::Job::CatchWave as u8&&*score>0&&d2(p.0 as i64,p.1 as i64,point.0,point.1)<=sq(45000)).map(|_|a.id())
        });
        if reserved.is_some_and(|id|id!=owner){return false;}
    }
    if !matches!(kind, Some(K::Jungle | K::Epic | K::Serpen)) {
        let lane_fallback = kind.is_none()
            && e.is_minion()
            && (0..sim.champion_count())
                .filter_map(|i| sim.get_entity(sim.champion_id_at(i)))
                .any(|c| c.team() == e.team());
        return accepts(kind, false, false, false, lane_fallback);
    }
    let jungle = (0..sim.player_count())
        .filter_map(|i| sim.player_at(i))
        .find(|p| p.champion().is_some_and(|c| c.id() == owner))
        .and_then(|p| p.lane())
        == Some(LaneV1::Jungle);
    let active = matches!(kind, Some(K::Epic | K::Serpen))
        && (0..e.buff_count())
            .filter_map(|i| e.buff_at(i))
            .any(|b| b.name() == format!("farm_objective_active:{}",me.team()));
    let ally = active
        && (0..sim.champion_count())
            .filter_map(|i| sim.get_entity(sim.champion_id_at(i)))
            .any(|a| {
                a.id() != owner && a.team() == me.team() && a.is_alive() && {
                    let (x, y) = a.pos();
                    let (ex, ey) = e.pos();
                    d2(x as i64, y as i64, ex as i64, ey as i64) <= sq(64000)
                }
            });
    accepts(kind, jungle, active, ally, false)
        || matches!(kind,Some(K::Epic|K::Serpen))&&crate::universal_ai::objective_assignment(sim,owner,target)
}