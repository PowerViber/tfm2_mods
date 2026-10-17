// Frozen reference from main a4098f1, test-only.
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
            .any(|b| b.name() == "farm_objective_active");
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
}
