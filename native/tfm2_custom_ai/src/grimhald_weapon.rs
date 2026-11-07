//! Optional weapon-only presentation. The complete body remains engine-owned.
//! Disabled for old/unintegrated definitions; never changes combat or control.
use crate::{grimhald_core::{Frame,Ult},grimhald_body_data as art,facing::Facing};
use mod_api_stable::StableSim;

#[derive(Clone,Default)]
pub struct Weapon {
    pub enabled:bool,
    owner:Option<usize>,
    slot:Option<String>,
    facing:Facing,
    last_pos:Option<(i64,i64)>,
    last_attack:Option<usize>,
    action:Option<(String,u64,u64)>,
    pose:String,
    pose_start:u64,
}
pub fn away(ult:Ult)->bool {matches!(ult,Ult::Flight{..}|Ult::Return{..}|Ult::Leap{..})}
fn frame(name:&str,ticks:u64)->usize {
    let durations=art::durations(name);let total=durations.iter().sum::<u64>().max(1)*60;
    let mut elapsed=ticks.saturating_mul(1000);
    elapsed=if matches!(name,"idle"|"run"){elapsed%total}else{elapsed.min(total-1)};
    for (i,d) in durations.iter().enumerate(){let d=d*60;if elapsed<d{return i;}elapsed-=d;}
    durations.len()-1
}
impl Weapon {
    pub fn pose(&mut self,name:&str,tick:u64,ticks:u64){if self.enabled {self.action=Some((name.into(),tick,tick+ticks));}}
    pub fn clear(&mut self,sim:&mut StableSim<'_>){if let (Some(id),Some(name))=(self.owner,self.slot.take()){sim.entity_remove_buff(id,&name);}}
    pub fn respawn(&mut self,sim:&mut StableSim<'_>){let enabled=self.enabled;self.clear(sim);*self=Self::default();self.enabled=enabled;}
    pub fn update(&mut self,sim:&mut StableSim<'_>,player:usize,id:usize,f:&Frame,ult:Ult){
        if !self.enabled{return;}
        self.owner=Some(id);
        if !f.alive {self.clear(sim);return;}
        let attack=sim.get_player(player).and_then(|p|p.cooldowns()).map(|c|c.0);
        if let (Some(old),Some(now))=(self.last_attack,attack) {if now>old {self.action=Some(("attack".into(),f.tick,f.tick+20));}}
        self.last_attack=attack;
        let pos=(f.pos.x,f.pos.y);let moving=self.last_pos.is_some_and(|p|p!=pos);self.last_pos=Some(pos);
        let enemy=f.enemy_champions().filter(|u|u.pos.within(f.pos,crate::facing::FACE_ENEMY)).min_by_key(|u|(u.pos.d2(f.pos),u.id)).map(|u|(u.pos.x,u.pos.y));
        self.facing.update(f.tick as usize,pos,enemy);
        if away(ult) {self.clear(sim);return;}
        if f.hard_cc_ticks>0 {self.action=None;}
        let (name,start)=if f.hard_cc_ticks>0 {("hit".to_string(),f.tick)}
            else if let Some((tag,start,_))=self.action.as_ref().filter(|(_,_,end)|f.tick<*end) {(tag.clone(),*start)}
            else {(if moving {"run"}else{"idle"}.into(),self.pose_start)};
        if self.pose!=name {self.pose=name.clone();self.pose_start=if self.action.as_ref().is_some_and(|(tag,_,_)|*tag==name){start}else{f.tick};}
        let index=frame(&name,f.tick.saturating_sub(self.pose_start));
        let slot=format!("{}{}_{}{}",art::WEAPON_PREFIX,name,if self.facing.left(){"l"}else{"r"},index);
        if self.slot.as_ref()!=Some(&slot){self.clear(sim);sim.add_buff(id,&crate::timed(&slot,5_184_000));self.slot=Some(slot);}
    }
}

#[cfg(test)]mod tests {
    use super::*;use crate::grimhald_core::Point;
    #[test]fn exact_away_lifecycle(){
        let p=Point::default();assert!(!away(Ult::Idle));assert!(!away(Ult::Windup{until:1,aim:p,target:None}));
        assert!(away(Ult::Flight{from:p,to:p,at:p,heading_mdeg:0,spent_mdeg:0,target:None,guide:None,t0:0,t1:1,last_tick:0}));
        assert!(away(Ult::Return{from:p,to:p,t0:0,t1:1}));assert!(away(Ult::Leap{from:p,to:p,t0:0,t1:1,cage_until:10}));assert!(!away(Ult::Caged{center:p,until:10}));
    }
    #[test]fn animation_clocks_are_bounded(){for name in ["idle","run","attack","skill1","skill2","ult","hit"]{for t in 0..1000{assert!(frame(name,t)<art::durations(name).len());}}assert_eq!(frame("attack",1000),3);}
    #[test]fn disabled_by_default(){assert!(!Weapon::default().enabled);}
    #[test]fn host_slot_removed_during_flight_without_touching_other_buffs(){
        use crate::glue_tests::{Host,Unit,with_sim};use mod_api_stable::{BuffV1,EntityKindV1};
        let host=Host::new(vec![Unit{kind:EntityKindV1::Champion,team:0,pos:(200000,200000),hp:1000,name:"tfm2_custom_grimhald",targetable:true}]);
        let mut w=Weapon{enabled:true,..Weapon::default()};let f=Frame{owner:0,alive:true,hp:1000,max_hp:1000,pos:Point::new(200000,200000),..Frame::default()};
        with_sim(&host,|sim|{sim.add_buff(0,&BuffV1::named("unrelated"));w.update(sim,0,0,&f,Ult::Idle)});
        assert!(host.borrow().buffs[0].iter().any(|b|b.name().starts_with(art::WEAPON_PREFIX)));
        with_sim(&host,|sim|w.update(sim,0,0,&f,Ult::Return{from:f.pos,to:f.pos,t0:0,t1:10}));
        assert!(!host.borrow().buffs[0].iter().any(|b|b.name().starts_with(art::WEAPON_PREFIX)));
        assert!(host.borrow().buffs[0].iter().any(|b|b.name()=="unrelated"));assert!(host.borrow().ccs[0].is_empty());
    }
    #[test]fn never_controls_or_hides_the_body(){let source=include_str!("grimhald_weapon.rs");let source=&source[..source.find("#[cfg(test)]").unwrap()];for call in ["apply_cc(","entity_banish(","entity_set_invisible(","set_input(","deal_damage(","entity_set_pos("]{assert!(!source.contains(call),"{call}");}}
}
