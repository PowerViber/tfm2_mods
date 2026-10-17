//! Farming eligibility shared by native kits; never injects engine input orders.
use crate::{d2, sq, Champ};
use mod_api_stable::{StableDraftContext, StableDraftDecision, StableDraftHook, StableSim};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    OnceLock,
};

pub const THREAT_RANGE: i64 = 110_000;
/// Role policy for intentional native farming. Incidental area damage is unchanged.
pub fn eligible(sim: &StableSim<'_>, owner: usize, target: usize) -> bool {
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
fn accepts(
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
        Some(K::Epic | K::Serpen) => jungle || (active && ally),
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
    for id in ids {
        let Some(e) = sim.get_entity(id) else {
            continue;
        };
        let hp = e.hp().0;
        let old = (0..e.buff_count())
            .filter_map(|i| e.buff_at(i))
            .find_map(|b| {
                b.name()
                    .strip_prefix("farm_hp:")
                    .and_then(|s| s.parse::<usize>().ok())
                    .map(|n| (b.name().to_string(), n))
            });
        if let Some((name, previous)) = old {
            if hp == previous && !sim.tick().is_multiple_of(60) {
                continue;
            }
            sim.entity_remove_buff(id, &name);
            if hp < previous {
                sim.entity_remove_buff(id, "farm_objective_active");
                sim.add_buff(id, &crate::timed("farm_objective_active", 60));
            }
        }
        sim.add_buff(id, &crate::timed(&format!("farm_hp:{hp}"), 120));
    }
}
pub fn safe(hp: usize, max_hp: usize, controlled: bool, nearby_visible_enemy: bool) -> bool {
    max_hp > 0
        && hp.saturating_mul(100) >= max_hp.saturating_mul(35)
        && !controlled
        && !nearby_visible_enemy
}

pub(crate) fn nearby(sim: &StableSim<'_>, me: &Champ, heroes: &[Champ], range: i64) -> Vec<Champ> {
    let threatened = heroes.iter().any(|e| {
        e.team != me.team
            && sim.is_visible(me.team, e.id)
            && d2(me.x, me.y, e.x, e.y) <= sq(THREAT_RANGE)
    });
    if !safe(me.hp, me.max_hp, me.stunned || me.pushed, threatened) {
        return vec![];
    }
    let mut targets: Vec<_> = crate::units(sim)
        .into_iter()
        .filter(|(id, team, x, y)| {
            *team != me.team && d2(me.x, me.y, *x, *y) <= sq(range) && eligible(sim, me.id, *id)
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
    targets.sort_by_key(|e| (d2(me.x, me.y, e.x, e.y), e.hp, e.id));
    targets.truncate(32);
    targets
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
pub struct DraftObserver;
/// Read-only observations of the engine's own farming casts on either team.
/// Never creates, holds, or replaces an input order.
#[derive(Clone, Default)]
pub struct FarmObserver {
    last: Option<usize>,
    lines: usize,
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
                            jungle || (active && ally)
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
