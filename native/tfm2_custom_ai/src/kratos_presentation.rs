//! Optional weapon-only presentation. The host keeps the visible whole body.
//! This module never moves, hides, damages, retargets or controls a champion.
use crate::{facing::Facing, kratos_art_data as art, kratos_core::{Core, Frame, Weapon}};
use mod_api_stable::StableSim;

#[derive(Clone, Default)]
pub struct Presentation {
    pub enabled: bool,
    owner: Option<usize>,
    slot: Option<String>,
    facing: Facing,
    last_pos: Option<(i64,i64)>,
    last_attack: Option<usize>,
    action: Option<(String,u64,u64)>,
    pose: String,
    start: u64,
}

pub fn pose(core: &Core, fallback: &'static str) -> &'static str {
    if let Some(r) = &core.rampage { return ["cleave","sweep","smash"][r.stage.min(2)]; }
    match core.last_skill {"sight"=>"sight", "guard"=>"guard", "dash"=>"chain", "fenrir"=>"ult", _=>fallback}
}

pub fn frame(name: &str, elapsed: u64) -> usize {
    let durations=art::durations(name); let total=durations.iter().sum::<u64>().max(1);
    let mut t=if matches!(name,"idle"|"run"){elapsed%total}else{elapsed.min(total-1)};
    for (i,d) in durations.iter().enumerate(){if t<*d{return i;}t-=d;}
    durations.len()-1
}

impl Presentation {
    pub fn action(&mut self, name:&str, tick:u64, ticks:u64) {
        if self.enabled {self.action=Some((name.into(),tick,tick+ticks));}
    }
    pub fn clear(&mut self,sim:&mut StableSim<'_>) {
        if let (Some(id),Some(name))=(self.owner,self.slot.take()){sim.entity_remove_buff(id,&name);}
    }
    pub fn respawn(&mut self,sim:&mut StableSim<'_>){let enabled=self.enabled;self.clear(sim);*self=Self::default();self.enabled=enabled;}
    pub fn update(&mut self,sim:&mut StableSim<'_>,player:usize,f:&Frame,core:&Core){
        if !self.enabled{return;}
        self.owner=Some(f.owner);
        if !f.alive{self.clear(sim);return;}
        let attack=sim.get_player(player).and_then(|p|p.cooldowns()).map(|c|c.0);
        if let (Some(old),Some(now))=(self.last_attack,attack){if now>old {self.action=Some(("attack".into(),f.tick,f.tick+crate::kratos_data::BASIC_DURATION as u64));}}
        self.last_attack=attack;
        let p=(f.pos.x,f.pos.y);let moving=self.last_pos.is_some_and(|old|old!=p);self.last_pos=Some(p);
        let enemy=f.enemy_champions().filter(|u|u.pos.within(f.pos,crate::facing::FACE_ENEMY)).min_by_key(|u|(u.pos.d2(f.pos),u.id)).map(|u|(u.pos.x,u.pos.y));
        self.facing.update(f.tick as usize,p,enemy);
        if f.stunned {self.action=None;}
        let (name,start)=if f.stunned{("hit".to_string(),self.start)}
            else if let Some((name,start,_))=self.action.as_ref().filter(|(_,_,end)|f.tick<*end){(name.clone(),*start)}
            else{(if moving{"run"}else{"idle"}.into(),self.start)};
        if name!=self.pose {self.pose=name.clone();self.start=if self.action.as_ref().is_some_and(|(p,_,_)|*p==name){start}else{f.tick};}
        let index=frame(&name,f.tick.saturating_sub(self.start));
        let stance=if core.weapon==Weapon::Blades{"blades"}else{"axe"};
        let slot=format!("{}{stance}_{name}_{}{index}",art::WEAPON_PREFIX,if self.facing.left(){"l"}else{"r"});
        if self.slot.as_ref()!=Some(&slot){self.clear(sim);sim.add_buff(f.owner,&crate::timed(&slot,5_184_000));self.slot=Some(slot);}
    }
}

#[cfg(test)] mod tests {
    use super::*;
    use crate::kratos_core::Point;
    #[test]fn stable_clocks_and_exact_action_durations(){
        for name in ["idle","run","attack","sight","guard","chain","cleave","sweep","smash","ult","hit","dead"]{
            for tick in 0..1000{assert!(frame(name,tick)<8);}
        }
        assert_eq!(art::durations("attack").iter().sum::<u64>(),18);
        assert_eq!(art::durations("cleave").iter().sum::<u64>(),crate::kratos_data::RAMPAGE_STAGE_TICKS as u64);
        assert_eq!(frame("attack",999),7);assert_eq!(frame("run",48),0);
    }
    #[test]fn three_rampage_phases_override_stance_without_touching_it(){
        let mut c=Core::default();c.last_skill="guard";assert_eq!(pose(&c,"skill2"),"guard");
        c.last_skill="dash";assert_eq!(pose(&c,"skill2"),"chain");
        for stage in 0..3 {c.rampage=Some(crate::kratos_core::Rampage{start:0,stage,swung:false,aim:Point::default(),at:Point::default(),hit:Default::default()});assert_eq!(pose(&c,"ult"),["cleave","sweep","smash"][stage]);assert_eq!(c.weapon,Weapon::Axe);}
    }
    #[test]fn weapon_switch_death_and_respawn_preserve_unrelated_state(){
        use crate::glue_tests::{Host,Unit,with_sim};use mod_api_stable::{BuffV1,EntityKindV1};
        let host=Host::new(vec![Unit{kind:EntityKindV1::Champion,team:0,pos:(200000,200000),hp:1000,name:"tfm2_custom_kratos",targetable:true}]);
        let mut p=Presentation{enabled:true,..Default::default()};let mut c=Core::default();let mut f=Frame{owner:0,alive:true,hp:1000,max_hp:1000,pos:Point::new(200000,200000),..Default::default()};
        with_sim(&host,|s|{s.add_buff(0,&BuffV1::named("unrelated"));p.update(s,0,&f,&c);});
        assert!(host.borrow().buffs[0].iter().any(|b|b.name().contains("weapon_axe_idle")));
        c.weapon=Weapon::Blades;f.tick=1;with_sim(&host,|s|p.update(s,0,&f,&c));
        assert!(host.borrow().buffs[0].iter().any(|b|b.name().contains("weapon_blades_idle")));
        assert_eq!(host.borrow().buffs[0].iter().filter(|b|b.name().starts_with(art::WEAPON_PREFIX)).count(),1);
        f.alive=false;with_sim(&host,|s|p.update(s,0,&f,&c));
        assert!(!host.borrow().buffs[0].iter().any(|b|b.name().starts_with(art::WEAPON_PREFIX)));
        with_sim(&host,|s|p.respawn(s));assert!(p.enabled);assert!(host.borrow().ccs[0].is_empty());
        assert!(host.borrow().buffs[0].iter().any(|b|b.name()=="unrelated"));
    }
    #[test]fn disabled_for_legacy_definitions(){assert!(!Presentation::default().enabled);}
}
