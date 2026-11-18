//! Approved 2.4 artwork leases. Reads the core; never changes combat or control.
use crate::contender_shadow_core::{Core, Effect, Frame};
use mod_api_stable::StableSim;
use std::collections::BTreeMap;

#[derive(Clone, Default)]
pub struct Presenter {
    owner: Option<usize>,
    markers: BTreeMap<usize, String>,
    aura: Option<String>,
    break_at: Option<u64>,
    rings: BTreeMap<(i64, i64, i64), u64>,
}

impl Presenter {
    pub fn clear(&mut self, sim: &mut StableSim<'_>) {
        for (id, name) in std::mem::take(&mut self.markers) { sim.entity_remove_buff(id, &name); }
        if let (Some(owner), Some(name)) = (self.owner, self.aura.take()) { sim.entity_remove_buff(owner, &name); }
        self.break_at = None;
        self.rings.clear();
    }
    pub fn sync(&mut self, sim: &mut StableSim<'_>, owner: usize, player: usize, f: &Frame, core: &Core, effects: &[Effect]) {
        // Cloned simulations retain their own identical entity IDs. Rebinding a
        // passive to another entity must not erase the original owner's leases.
        if self.owner.is_some_and(|id| id != owner) { *self = Self::default(); }
        self.owner = Some(owner);
        if !f.alive { self.clear(sim); return; }
        let mut desired = BTreeMap::new();
        // Standard matches have ten player slots. Unknown layouts retain the
        // visible base rather than sharing another owner's marker namespace.
        if player < 10 {
            for s in &core.stacks {
                if s.count > 0 && f.tick < s.until && f.unit(s.id).is_some_and(|u| u.hp > 0 && u.visible && f.skill_target(u)) {
                    desired.insert(s.id, format!("ct_rhythm_p{player}:{}", s.count.min(20)));
                }
            }
        }
        for (id, name) in &self.markers {
            if desired.get(id) != Some(name) { sim.entity_remove_buff(*id, name); }
        }
        for (id, name) in &desired {
            if self.markers.get(id) != Some(name) { sim.add_buff(*id, &mod_api_stable::BuffV1::named(name)); }
        }
        self.markers = desired;
        if effects.iter().any(|e| matches!(e, Effect::Fx { name, .. } if name == "ct_break")) { self.break_at = Some(f.tick); }
        let phase = self.break_at.filter(|start| f.tick < start.saturating_add(480))
            .map(|start| format!("ct_rhythm_full_{}", ((f.tick - start) / 12) % 4));
        if phase != self.aura {
            if let Some(name) = self.aura.take() { sim.entity_remove_buff(owner, &name); }
            if let Some(name) = &phase {
                let remaining = self.break_at.unwrap_or(f.tick).saturating_add(480).saturating_sub(f.tick);
                sim.add_buff(owner, &crate::timed(name, remaining.max(1) as usize));
            }
            self.aura = phase;
        }
        self.rings.retain(|key, _| core.cleaves.iter().any(|c| (c.at.x,c.at.y,c.radius) == *key));
        for c in &core.cleaves {
            let key = (c.at.x,c.at.y,c.radius);
            if self.rings.get(&key).is_none_or(|due| f.tick >= *due) {
                let radius = if c.recast { 35 } else { 25 };
                crate::fx_point(sim, &format!("ct_ring{radius}_t6"), owner, c.at.x, c.at.y, 6);
                self.rings.insert(key, f.tick + 6);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contender_shadow_core::Point;
    use crate::glue_tests::{Host, Unit, open_spot, with_sim};
    use mod_api_stable::EntityKindV1;
    #[test]
    fn break_aura_has_one_clock_and_markers_cleanup_by_owner_without_combat_writes() {
        let _guard = crate::walls::TEST_LOCK.lock(); let (x,y) = open_spot(90_000);
        let unit = |team, pos| Unit { kind: EntityKindV1::Champion, team, pos, hp: 2000, name: "tfm2_custom_contender", targetable: true };
        let host = Host::new(vec![unit(0,(x,y)),unit(0,(x+1000,y)),unit(1,(x+10_000,y))]);
        let mut adapter = crate::contender::Contender::default();
        let mut f = with_sim(&host,|sim| adapter.frame(sim,0)); f.tick = 100;
        let mut core = Core::default();
        core.stacks.push(crate::contender_shadow_core::Stack { id: 2, count: 20, until: 900, champion: true });
        let snapshot = serde_json::to_value(&core).unwrap();
        let cue = Effect::Fx {name:"ct_break".into(),at:Point::new(x as i64,y as i64),ticks:18,heading:None};
        let mut a=Presenter::default(); let mut b=Presenter::default();
        with_sim(&host,|sim| {a.sync(sim,0,0,&f,&core,&[cue]); b.sync(sim,1,1,&f,&core,&[]);});
        assert!(host.borrow().buffs[2].iter().any(|b|b.name()=="ct_rhythm_p0:20"));
        assert!(host.borrow().buffs[2].iter().any(|b|b.name()=="ct_rhythm_p1:20"));
        for tick in 101..580 { f.tick=tick; with_sim(&host,|sim|a.sync(sim,0,0,&f,&core,&[]));
            assert_eq!(host.borrow().buffs[0].iter().filter(|b|b.name().starts_with("ct_rhythm_full_")).count(),1); }
        f.tick=580;with_sim(&host,|sim|a.sync(sim,0,0,&f,&core,&[]));
        assert!(!host.borrow().buffs[0].iter().any(|b|b.name().starts_with("ct_rhythm_full_")));
        f.alive=false;with_sim(&host,|sim|a.sync(sim,0,0,&f,&core,&[]));
        assert!(!host.borrow().buffs[2].iter().any(|b|b.name()=="ct_rhythm_p0:20"));
        assert!(host.borrow().buffs[2].iter().any(|b|b.name()=="ct_rhythm_p1:20"));
        let h=host.borrow(); assert!(h.damage.is_empty()&&h.moves.is_empty()&&h.ccs.iter().all(|x|x.is_empty())&&h.heals.is_empty());
        assert_eq!(serde_json::to_value(&core).unwrap(),snapshot);
    }
    #[test]
    fn invisible_expired_or_reset_stacks_never_leave_a_marker() {
        let _guard = crate::walls::TEST_LOCK.lock();let (x,y)=open_spot(90_000);
        let unit=|team|Unit{kind:EntityKindV1::Champion,team,pos:(x,y),hp:2000,name:"tfm2_custom_contender",targetable:true};
        let host=Host::new(vec![unit(0),unit(1)]);let mut adapter=crate::contender::Contender::default();
        let mut f=with_sim(&host,|sim|adapter.frame(sim,0));f.tick=100;
        let mut core=Core::default();core.stacks.push(crate::contender_shadow_core::Stack{id:1,count:12,until:200,champion:true});
        let mut p=Presenter::default();with_sim(&host,|sim|p.sync(sim,0,0,&f,&core,&[]));
        assert!(!host.borrow().buffs[1].is_empty());
        f.units.iter_mut().find(|u|u.id==1).unwrap().visible=false;
        with_sim(&host,|sim|p.sync(sim,0,0,&f,&core,&[]));assert!(host.borrow().buffs[1].is_empty());
        f.units.iter_mut().find(|u|u.id==1).unwrap().visible=true;f.tick=200;
        with_sim(&host,|sim|p.sync(sim,0,0,&f,&core,&[]));assert!(host.borrow().buffs[1].is_empty());
        f.tick=150;core.stacks.clear();with_sim(&host,|sim|p.sync(sim,0,0,&f,&core,&[]));assert!(host.borrow().buffs[1].is_empty());
    }
}
