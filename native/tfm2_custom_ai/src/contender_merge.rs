//! Opaque whole-body Merge presentation. Only artwork buffs are written here.
//! No control, movement, combat or random state is changed.
use crate::contender_shadow_core::{Core, Effect, Frame, Point};
use mod_api_stable::{StableSim};

#[derive(Clone, Default)]
pub struct Presenter {
    facing: crate::facing::Facing,
    last_pos: Option<Point>,
    attack_cd: Option<usize>,
    attack_start: Option<u64>,
    cast_start: Option<u64>,
    hit_until: u64,
    pose: &'static str,
    pose_start: u64,
    slot: Option<(usize, String)>,
}

pub fn tile(pose: &str, elapsed: u64, looping: bool) -> usize {
    let seq = crate::contender_merge_data::sequence(pose);
    let total = seq.iter().map(|(_, ticks)| ticks).sum::<u64>().max(1);
    let mut at = elapsed.saturating_mul(1000);
    at = if looping { at % total } else { at.min(total - 1) };
    for &(id, ticks) in seq {
        if at < ticks { return id; }
        at -= ticks;
    }
    seq.last().map_or(0, |x| x.0)
}

impl Presenter {
    pub fn clear(&mut self, sim: &mut StableSim<'_>) {
        if let Some((owner, name)) = self.slot.take() { sim.entity_remove_buff(owner, &name); }
    }
    pub fn hit(&mut self, tick: u64) { self.hit_until = tick + 6; }
    pub fn basic(&mut self, tick: u64) { self.attack_start = Some(tick); }
    fn observe(&mut self, f: &Frame, effects: &[Effect], cd: Option<usize>) -> (&'static str, usize) {
        let moving = self.last_pos.is_some_and(|p| p != f.pos);
        self.last_pos = Some(f.pos);
        if let (Some(old), Some(now)) = (self.attack_cd, cd) {
            if now > old { self.basic(f.tick); }
        }
        self.attack_cd = cd;
        if effects.iter().any(|e| matches!(e, Effect::Merge { ticks, .. } if *ticks > 1)) {
            self.cast_start = Some(f.tick);
        }
        let (pose, start) = if f.stunned || f.tick < self.hit_until {
            self.attack_start = None;
            self.cast_start = None;
            ("hit", self.pose_start)
        } else if let Some(start) = self.attack_start.filter(|s| f.tick < *s + crate::contender_shadow_data::BASIC_DURATION as u64) {
            ("attack", start)
        } else if let Some(start) = self.cast_start.filter(|s| f.tick < *s + 18) {
            ("projection", start)
        } else {
            (if moving { "run" } else { "idle" }, self.pose_start)
        };
        if self.pose != pose {
            self.pose = pose;
            self.pose_start = if matches!(pose, "attack" | "projection") { start } else { f.tick };
        } else if matches!(pose, "attack" | "projection") { self.pose_start = start; }
        (pose, tile(pose, f.tick.saturating_sub(self.pose_start), matches!(pose, "idle" | "run")))
    }
    pub fn sync(&mut self, sim: &mut StableSim<'_>, owner: usize, f: &Frame, core: &Core, effects: &[Effect], cd: Option<usize>) {
        let (_, index) = self.observe(f, effects, cd);
        let enemy = f.units.iter().filter(|u| u.team != f.team && u.visible && u.hp > 0 && u.targetable)
            .map(|u| (u.pos.d2(f.pos), u.id, u.pos)).filter(|(d, _, _)| *d <= (crate::facing::FACE_ENEMY as i128).pow(2))
            .min_by_key(|(d, id, _)| (*d, *id)).map(|(_, _, p)| (p.x, p.y));
        self.facing.update(f.tick as usize, (f.pos.x, f.pos.y), enemy);
        if !f.alive || !core.merged(f.tick) || core.flurrying() { self.clear(sim); return; }
        let name = format!("ct_mb_{index}{}", if self.facing.left() { "_l" } else { "" });
        if self.slot.as_ref().is_some_and(|(id, old)| *id == owner && *old == name) { return; }
        self.clear(sim);
        sim.add_buff(owner, &crate::timed(&name, core.merge_until.saturating_sub(f.tick).max(1) as usize));
        self.slot = Some((owner, name));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn frames_follow_exact_existing_body_clocks_without_restarting() {
        assert_eq!(tile("run", 0, true), 1);
        assert_eq!(tile("run", 8, true), 2);
        assert_eq!(tile("run", 48, true), 1);
        assert_eq!(tile("attack", 0, false), 6);
        assert_eq!(tile("attack", 9, false), 7);
        assert_eq!(tile("attack", 17, false), 8);
        assert_eq!(tile("attack", 1000, false), 8);
    }
    #[test]
    fn presenter_never_changes_gameplay_or_hides_the_base() {
        let src=include_str!("contender_merge.rs");let src=&src[..src.find("#[cfg(test)]").unwrap()];
        for call in ["apply_cc","deal_damage","entity_set_pos","entity_set_invisible","set_input","heal"] {
            assert!(!src.contains(&format!(".{call}(")),"{call}");
        }
    }
    #[test]
    fn whole_body_slots_replace_immediately_and_cleanup_without_combat_side_effects() {
        use crate::glue_tests::{Host, Unit, open_spot, with_sim};
        use mod_api_stable::EntityKindV1;
        let _guard=crate::walls::TEST_LOCK.lock();
        let (x,y)=open_spot(90_000);
        let host=Host::new(vec![Unit{kind:EntityKindV1::Champion,team:0,pos:(x,y),hp:2000,name:"tfm2_custom_contender",targetable:true}]);
        let mut adapter=crate::contender::Contender::default();
        let mut f=with_sim(&host,|sim|adapter.frame(sim,0));
        let mut core=Core::default();core.merge_until=400;
        let before=serde_json::to_value(&core).unwrap();let mut p=Presenter::default();
        for tick in 100..400 {
            f.tick=tick;f.pos.x+=1000;
            if tick==120 {p.basic(tick);}
            with_sim(&host,|sim|p.sync(sim,0,&f,&core,&[],None));
            let h=host.borrow();
            assert_eq!(h.buffs[0].iter().filter(|b|b.name().starts_with("ct_mb_")).count(),1);
            if tick==129 {assert!(h.buffs[0].iter().any(|b|b.name()=="ct_mb_7"));}
            assert!(h.damage.is_empty()&&h.moves.is_empty()&&h.ccs[0].is_empty()&&h.heals.is_empty());
        }
        f.tick=400;with_sim(&host,|sim|p.sync(sim,0,&f,&core,&[],None));
        assert!(!host.borrow().buffs[0].iter().any(|b|b.name().starts_with("ct_mb_")));
        assert_eq!(serde_json::to_value(&core).unwrap(),before);
    }
    #[test]
    fn owners_and_clones_are_independent_and_death_clears_the_layer() {
        use crate::glue_tests::{Host, Unit, open_spot, with_sim};
        use mod_api_stable::EntityKindV1;
        let _guard=crate::walls::TEST_LOCK.lock();let (x,y)=open_spot(90_000);
        let unit=|pos|Unit{kind:EntityKindV1::Champion,team:0,pos,hp:2000,name:"tfm2_custom_contender",targetable:true};
        let host=Host::new(vec![unit((x,y)),unit((x+10_000,y))]);
        let mut adapter=crate::contender::Contender::default();let mut f=with_sim(&host,|sim|adapter.frame(sim,0));f.tick=100;
        let mut core=Core::default();core.merge_until=300;
        let mut a=Presenter::default();let mut b=a.clone();
        with_sim(&host,|sim|{a.sync(sim,0,&f,&core,&[],None);b.sync(sim,1,&f,&core,&[],None);});
        f.alive=false;with_sim(&host,|sim|a.sync(sim,0,&f,&core,&[],None));
        assert!(host.borrow().buffs[0].is_empty());
        assert!(host.borrow().buffs[1].iter().any(|b|b.name().starts_with("ct_mb_")));
        with_sim(&host,|sim|b.clear(sim));assert!(host.borrow().buffs[1].is_empty());
    }
}
