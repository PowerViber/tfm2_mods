//! Read-only presentation of completed core state. No combat commands or RNG.
use crate::grimhald_core::{Core, Point, Ult,ult_projection};
use crate::grimhald_data as d;
use serde::Serialize;

pub const RENEW_TICKS:u64=d::PRESENTATION_RENEW_TICKS as u64;
#[derive(Clone,Debug,PartialEq,Serialize)]
pub struct Cue {pub tag:String,pub at:Point,pub ticks:u64,pub start_tick:u64}

/// Optional visual alias. The combat event and its identity stay unchanged.
pub fn effect_tag(name:&str,core:&Core)->Option<String> {
    if name!="grh_fissure" {return None;}
    let r=core.rupture.as_ref()?;
    let a=((r.to.y-r.from.y) as f64).atan2((r.to.x-r.from.x) as f64);
    let n=d::PRESENTATION_TRAIL_DIRECTIONS;
    let direction=((a/std::f64::consts::TAU*n as f64).round() as i64).rem_euclid(n);
    Some(format!("grh_fissure_d{direction}"))
}

/// One full pentagon replaces the legacy five overlapping midpoint drawings.
/// Persistent structures use single-frame phases; renewing them never restarts an animation.
pub fn cues(core:&Core,tick:u64,alive:bool)->Vec<Cue> {
    if !alive{return vec![]}
    let mut out=vec![];
    let mut add=|tag:String,at:Point,until:u64,start_tick:u64|{
        if tick<until {out.push(Cue{tag,at,ticks:RENEW_TICKS.min(until-tick),start_tick});}
    };
    if let Some((at,until))=ult_projection(core.ult) {
        let start=match core.ult {Ult::Windup{until,..}=>until.saturating_sub(d::ULT_WINDUP_TICKS as u64),Ult::Flight{t0,..}=>t0.saturating_sub(d::ULT_WINDUP_TICKS as u64),_=>tick};
        add("ult_marker".into(),at,until+1,start);
    }
    if let Some(p)=core.pillar {
        let start=p.until.saturating_sub(d::BULWARK_PILLAR_TICKS as u64);
        add("pillar".into(),p.center,p.until,start);
    }
    for t in &core.trails {
        let start=t.until.saturating_sub(d::BULWARK_TRAIL_TICKS as u64);
        // One oriented composite keeps all persistent structures inside the
        // existing token budget, rather than renewing four separate tiles.
        let angle=((t.b.y-t.a.y) as f64).atan2((t.b.x-t.a.x) as f64);
        let direction=((angle/std::f64::consts::TAU*d::PRESENTATION_TRAIL_DIRECTIONS as f64).round() as i64).rem_euclid(d::PRESENTATION_TRAIL_DIRECTIONS);
        let n=(t.a.dist(t.b)/d::PRESENTATION_TRAIL_LENGTH_STEP as f64).ceil() as i64;
        let n=n.clamp(1,d::PRESENTATION_TRAIL_MAX_STEPS);
        add(format!("trail_d{direction}_l{n}"),t.a.lerp(t.b,1,2),t.until,start);
    }
    let cage=match core.ult {
        Ult::Leap{to,cage_until,..}=>Some((to,cage_until)),
        Ult::Caged{center,until}=>Some((center,until)),_=>None
    };
    if let Some((center,until))=cage {
        let start=until.saturating_sub((d::ULT_CAGE_TICKS+d::ULT_CAGE_RISE_TICKS) as u64);
        let age=tick.saturating_sub(start);
        let phase=(age*4/d::ULT_CAGE_RISE_TICKS as u64).min(3);
        add(format!("cage_phase{phase}"),center,until,start);
    }
    out
}

#[cfg(test)] mod tests {
    use super::*;
    use crate::grimhald_core::{Pillar,Seg,Trail};
    #[test] fn lifecycle_and_expiry(){
        let mut c=Core::default();let p=Point::new(100000,100000);
        c.pillar=Some(Pillar{center:p,seg:Seg{a:p,b:p},until:240,relocated:false});
        assert_eq!(cues(&c,239,true)[0].ticks,1);assert!(cues(&c,240,true).is_empty());assert!(cues(&c,10,false).is_empty());
        c.pillar=None;c.trails.push(Trail{a:p,b:Point::new(125000,100000),until:180});
        assert_eq!(cues(&c,20,true).len(),1);assert_eq!(cues(&c,20,true)[0].tag,"trail_d0_l4");assert!(cues(&c,180,true).is_empty());
    }
    #[test] fn complete_cage_stable_clock(){
        let mut c=Core::default();c.ult=Ult::Caged{center:Point::new(300000,300000),until:100+(d::ULT_CAGE_TICKS+d::ULT_CAGE_RISE_TICKS) as u64};
        assert_eq!(cues(&c,100,true)[0].tag,"cage_phase0");assert_eq!(cues(&c,110,true)[0].tag,"cage_phase2");
        let a=cues(&c,130,true);let b=cues(&c,136,true);assert_eq!(a[0].start_tick,b[0].start_tick);assert_eq!(a[0].tag,"cage_phase3");assert!(cues(&c,100+(d::ULT_CAGE_TICKS+d::ULT_CAGE_RISE_TICKS) as u64,true).is_empty());
    }
    #[test] fn presentation_leaves_state_identical(){
        let c=Core::default();let before=serde_json::to_string(&c).unwrap();for t in 0..1000 {let _=cues(&c,t,true);}assert_eq!(before,serde_json::to_string(&c).unwrap());
    }
    #[test] fn fissure_alias_follows_ground_direction_without_state_changes(){
        use crate::grimhald_core::Rupture;use std::collections::BTreeSet;
        let mut c=Core::default();c.rupture=Some(Rupture{from:Point::new(100000,100000),to:Point::new(154000,100000),start:0,arrival:18,pillar_at:30,last:Point::new(100000,100000),hits:BTreeSet::new()});
        let before=serde_json::to_string(&c).unwrap();assert_eq!(effect_tag("grh_fissure",&c).as_deref(),Some("grh_fissure_d0"));assert_eq!(serde_json::to_string(&c).unwrap(),before);
        c.rupture.as_mut().unwrap().to=Point::new(100000,154000);assert_eq!(effect_tag("grh_fissure",&c).as_deref(),Some("grh_fissure_d8"));assert!(effect_tag("grh_land",&c).is_none());
    }
}
