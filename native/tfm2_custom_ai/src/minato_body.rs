//! Presentation-only body replacement. Never writes input, stats, CC, HP or RNG.
//! Old embedded definitions retain their visible sprite; only new definitions opt
//! into this passive and the visible roster/fallback base. Buff slots contain artwork only.
use mod_api_stable::{CcKindV1, StablePassive, StableSim};
use crate::{facing::Facing, minato_body_data::{frames, ACTIONS}};

#[derive(Clone, Debug, Default)]
struct Clock {
    last_cooldowns: Option<[usize; 4]>,
    action: Option<(&'static str, usize, usize)>,
    last_pos: Option<(i64, i64)>,
    pose: String,
    pose_start: usize,
    hit_until: usize,
}

impl Clock {
    fn observe(&mut self, tick: usize, pos: (i64, i64), cooldowns: Option<[usize; 4]>, controlled: bool) -> (&str, usize) {
        if let (Some(old), Some(now)) = (self.last_cooldowns, cooldowns) {
            // Highest-priority newly accepted action wins if host reports several.
            for n in 0..4 {
                if now[n] > old[n] {
                    let (tag, duration) = ACTIONS[n];
                    self.action = Some((tag, tick, tick + duration));
                }
            }
        }
        self.last_cooldowns = cooldowns;
        let moving = self.last_pos.is_some_and(|p| p != pos);
        self.last_pos = Some(pos);
        if controlled { self.action = None; }
        let (pose, start) = if controlled || tick < self.hit_until {
            ("hit", self.pose_start)
        } else if let Some((tag, start, end)) = self.action.filter(|(_, _, end)| tick < *end) {
            let _ = end;
            (tag, start)
        } else {
            (if moving { "run" } else { "idle" }, self.pose_start)
        };
        if self.pose != pose {
            self.pose = pose.into();
            self.pose_start = if self.action.is_some_and(|(tag,_,end)| tag == pose && tick < end) { start } else { tick };
        }
        (&self.pose, frame(&self.pose, tick.saturating_sub(self.pose_start), matches!(pose, "idle" | "run")))
    }
}

fn frame(tag: &str, elapsed: usize, looping: bool) -> usize {
    let durations = frames(tag);
    let total = durations.iter().sum::<usize>().max(1);
    let mut at = elapsed.saturating_mul(1000);
    at = if looping { at % total } else { at.min(total - 1) };
    for (n, d) in durations.iter().enumerate() {
        if at < *d { return n; }
        at -= d;
    }
    durations.len() - 1
}

#[derive(Clone, Default)]
pub struct MinatoBody {
    clock: Clock,
    facing: Facing,
    owner: Option<usize>,
    slot: Option<String>,
}

impl MinatoBody {
    fn clear(&mut self, sim: &mut StableSim<'_>) {
        if let (Some(id), Some(name)) = (self.owner, self.slot.take()) {
            sim.entity_remove_buff(id, &name);
        }
    }
    fn show(&mut self, sim: &mut StableSim<'_>, tag: &str, index: usize, kcm: bool) {
        let name = format!("mn_body_{}_{}_{}{}", if kcm { "kcm" } else { "normal" }, tag, if self.facing.left() { "l" } else { "r" }, index);
        if self.slot.as_ref() != Some(&name) {
            self.clear(sim);
            if let Some(id) = self.owner { sim.add_buff(id, &crate::timed(&name, 5_184_000)); }
            self.slot = Some(name);
        }
    }
}

impl StablePassive for MinatoBody {
    fn clone_box(&self) -> Box<dyn StablePassive> { Box::new(self.clone()) }
    fn on_spawn(&mut self, sim: &mut StableSim<'_>, player: usize, entity: usize) {
        self.clear(sim);
        *self = Self::default();
        self.owner = Some(entity);
        self.on_update(sim, 0, player, entity);
    }
    fn on_update(&mut self, sim: &mut StableSim<'_>, _: u64, player: usize, entity: usize) {
        self.owner = Some(entity);
        let tick = sim.tick();
        let Some(e) = sim.get_entity(entity) else { self.clear(sim); return; };
        if !e.is_alive() {
            self.clear(sim); // on_dead owns the single fixed-position death animation.
            return;
        }
        let (x,y) = e.pos(); let pos = (x as i64, y as i64); let team = e.team();
        let kcm = (0..e.buff_count()).filter_map(|n| e.buff_at(n)).any(|b| b.name() == "minato_kcm");
        let controlled = (0..e.cc_count()).filter_map(|n| e.cc_at(n)).any(|c|
            [CcKindV1::Stun, CcKindV1::Airborne, CcKindV1::Fear].iter().any(|k| k.code() == c.kind));
        let enemy = (0..sim.player_count()).filter_map(|n| sim.player_at(n)).filter_map(|p| p.champion())
            .filter(|e| e.team() != team && e.is_alive() && sim.is_visible(team, e.id()))
            .map(|e| { let (x,y) = e.pos(); let p = (x as i64, y as i64); ((p.0-pos.0) as i128 * (p.0-pos.0) as i128 + (p.1-pos.1) as i128 * (p.1-pos.1) as i128, e.id(), p) })
            .filter(|(d,_,_)| *d <= (crate::facing::FACE_ENEMY as i128).pow(2)).min_by_key(|(d,id,_)| (*d,*id)).map(|(_,_,p)| p);
        self.facing.update(tick, pos, enemy);
        let cds = sim.get_player(player).and_then(|p| p.cooldowns()).map(|(a,b,c,d)| [a,b,c,d]);
        let (tag,index) = self.clock.observe(tick, pos, cds, controlled);
        let tag = tag.to_string();
        // The engine draws his visible base body (every approved normal frame) and walks it. An Animation CC is
        // still a CC: renewing one every tick (PR #53) kept him "in an animation" and pinned him at spawn. So this
        // presenter never touches CC. Normal form needs no layer; KCM draws the golden body over the base.
        if kcm { self.show(sim, &tag, index, kcm); } else { self.clear(sim); }
    }
    fn on_damaged(&mut self, sim: &mut StableSim<'_>, _: usize, _: usize, _: usize, damage: usize) {
        if damage > 0 { self.clock.hit_until = sim.tick() + 6; }
    }
    fn on_dead(&mut self, sim: &mut StableSim<'_>, _: usize) {
        self.clear(sim);
        self.clock.action = None;
        // The visible fallback owns its normal engine death animation. Do not
        // emit a second corpse; the last empty override expires after one tick.
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn presenter_never_touches_crowd_control() {
        // PR #53 renewed a one-tick Animation CC every tick, and Minato never left spawn.
        let source = include_str!("minato_body.rs");
        let code = &source[..source.find("#[cfg(test)]").unwrap()];
        for call in ["apply_cc", "remove_cc", "clear_cc", "entity_banish", "set_input"] {
            assert!(!code.contains(&format!("{call}(")), "{call}");
        }
    }
    #[test] fn cooldown_observation_selects_without_restarting_each_tick() {
        let mut c=Clock::default(); c.observe(0,(0,0),Some([0;4]),false);
        assert_eq!(c.observe(1,(0,0),Some([50,0,0,0]),false).0,"attack");
        c.observe(8,(0,0),Some([43,0,0,0]),false); assert_eq!(c.pose_start,1);
        assert_eq!(c.observe(22,(1,0),Some([29,0,0,0]),false).0,"run");
    }
    #[test] fn form_change_does_not_change_pose_clock() {
        let mut c=Clock::default(); c.observe(0,(0,0),Some([0;4]),false);
        c.observe(1,(0,0),Some([0,0,0,3600]),false);
        let mut clone=c.clone();
        assert_eq!(c.observe(12,(0,0),Some([0,0,0,3589]),false),clone.observe(12,(0,0),Some([0,0,0,3589]),false));
        assert_eq!(c.pose_start,1);
    }
    #[test] fn interruption_clears_only_presentation_intention() {
        let mut c=Clock::default(); c.observe(0,(0,0),Some([0;4]),false);
        c.observe(1,(0,0),Some([0,420,0,0]),false);
        assert_eq!(c.observe(2,(0,0),Some([0,419,0,0]),true).0,"hit");
        assert!(c.action.is_none());
    }
    #[test] fn nonlooping_death_holds_last_frame_and_loops_remain_bounded() {
        assert_eq!(frame("dead",10000,false),frames("dead").len()-1);
        for t in 0..1000 { assert!(frame("run",t,true)<frames("run").len()); }
    }
}
