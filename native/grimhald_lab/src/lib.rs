//! Transport around the game's exact core. Host guidance is explicitly a Lab model.
#[path="../../tfm2_custom_ai/src/grimhald_core.rs"]
pub mod grimhald_core;
#[path="../../tfm2_custom_ai/src/grimhald_data.rs"]
pub mod grimhald_data;
#[path="../../tfm2_custom_ai/src/grimhald_presentation.rs"]
pub mod grimhald_presentation;
use grimhald_core::*;
use serde::Deserialize;
use serde_json::json;
use std::{cell::RefCell,collections::BTreeSet};
pub const REVISION:&str=grimhald_data::RULES_REVISION;
#[derive(Deserialize)]
pub struct Input {pub version:u8,pub rules_revision:String,pub frame:Frame}
#[derive(Default)]
pub struct Session {pub core:Core,last_tick:Option<u64>,cage:Option<Cage>,members:BTreeSet<usize>}
fn valid(f:&Frame)->bool {
    let point=|p:Point|(0..=MAP).contains(&p.x)&&(0..=MAP).contains(&p.y);
    let order=match f.order {Some(Order::Ghost(p)|Order::Bulwark(p)|Order::Relocate(p)|Order::Bash(p)|Order::Ult(p)|Order::Steer(p))=>point(p),_=>true};
    order&&point(f.pos)&&f.tick<1_000_000_000&&f.units.len()<=128&&f.walls.len()<=256
    &&(f.terrain.is_empty()||f.terrain.len()==900)&&f.profile.iter().all(|v|(0..=100).contains(v))
    &&(1..=100_000_000).contains(&f.max_hp)&&(0..=f.max_hp).contains(&f.hp)
    &&(0..=1_000_000).contains(&f.attack)&&(0..=100_000).contains(&f.move_speed)
    &&f.hard_cc_ticks<1_000_000&&f.owner<=1_000_000
    &&f.units.iter().map(|u|u.id).collect::<BTreeSet<_>>().len()==f.units.len()
    &&f.units.iter().all(|u|u.id!=f.owner&&u.id<=1_000_000&&point(u.pos)&&u.velocity.x.unsigned_abs()<=100_000&&u.velocity.y.unsigned_abs()<=100_000&&(1..=100_000).contains(&u.radius)&&(1..=100_000_000).contains(&u.max_hp)&&(0..=u.max_hp).contains(&u.hp))
    &&f.walls.iter().all(|s|point(s.a)&&point(s.b))
    &&f.cage.is_none_or(|c|point(c.center)&&(1..=200_000).contains(&c.radius)&&c.until<1_000_000_000)
}
pub fn run(session:&mut Session,text:&str)->String {
    let Ok(input)=serde_json::from_str::<Input>(text) else{return json!({"error":"Invalid Grimhald Lab v1 input"}).to_string()};
    let f=input.frame;
    if input.version!=1||input.rules_revision!=REVISION||!valid(&f)||session.last_tick.is_some_and(|t|f.tick<=t) {
        return json!({"error":"Original rules, valid bounds and increasing ticks required"}).to_string()
    }
    session.last_tick=Some(f.tick);
    if !f.alive {session.core.respawn(f.tick);session.cage=None;session.members.clear();}
    let effects=session.core.step(&f);
    for effect in &effects {
        match effect {
            Effect::CageUp{cage}=>{session.cage=Some(*cage);session.members=BTreeSet::from([f.owner]);for u in &f.units {if u.champion&&u.hp>0&&inside(cage.center,cage.radius,u.pos,0){session.members.insert(u.id);}}},
            Effect::CageDown=>{session.cage=None;session.members.clear()},_=>{}
        }
    }
    if session.cage.is_some_and(|c|f.tick>=c.until){session.cage=None;session.members.clear();}
    let mut placements=vec![];
    if let Some(c)=session.cage {
        for (id,pos,champion,radius) in std::iter::once((f.owner,f.pos,true,10_000)).chain(f.units.iter().filter(|u|u.hp>0).map(|u|(u.id,u.pos,u.champion,u.radius))) {
            if id==f.owner&&matches!(session.core.ult,Ult::Leap{..}){continue;}
            let next=if session.members.contains(&id){clamp_inside(c.center,c.radius,pos,11_000)}
                else if !inside(c.center,c.radius,pos,-(11_000+if champion{0}else{30_000+radius})){pos}
                else {push_outside(c.center,c.radius,pos,11_000+if champion{0}else{30_000+radius})};
            if next!=pos {placements.push(json!({"target":id,"pos":next}));}
        }
    }
    let vertices=session.cage.map(|c|pentagon(c.center,c.radius));
    let presentation=grimhald_presentation::cues(&session.core,f.tick,f.alive);
    let effect_tags:Vec<_>=effects.iter().map(|e|match e {Effect::Fx{name,..}=>grimhald_presentation::effect_tag(name,&session.core),_=>None}).collect();
    json!({"version":1,"rules_revision":REVISION,"state":session.core,"effects":effects,"effect_tags":effect_tags,"presentation":presentation,"host_model":{"simulated":true,"cage":session.cage,"members":session.members,"placements":placements,"vertices":vertices}}).to_string()
}
#[derive(Default)]
struct Wasm {sessions:Vec<Option<Session>>,input:Vec<u8>,output:Vec<u8>}
thread_local!{static WASM:RefCell<Wasm>=RefCell::new(Wasm::default());}
#[no_mangle] pub extern "C" fn grimhald_new()->u32 {WASM.with(|w|{let mut w=w.borrow_mut();if let Some(i)=w.sessions.iter().position(Option::is_none){w.sessions[i]=Some(Session::default());i as u32}else if w.sessions.len()<128 {let i=w.sessions.len();w.sessions.push(Some(Session::default()));i as u32}else{u32::MAX}})}
#[no_mangle] pub extern "C" fn grimhald_drop(id:u32){WASM.with(|w|{if let Some(s)=w.borrow_mut().sessions.get_mut(id as usize){*s=None}})}
#[no_mangle] pub extern "C" fn grimhald_input(n:u32)->*mut u8 {WASM.with(|w|{if n>2_000_000{return std::ptr::null_mut()};let mut w=w.borrow_mut();w.input.resize(n as usize,0);w.input.as_mut_ptr()})}
#[no_mangle] pub extern "C" fn grimhald_output_len()->u32 {WASM.with(|w|w.borrow().output.len() as u32)}
#[no_mangle] pub extern "C" fn grimhald_step(id:u32,n:u32)->*const u8 {WASM.with(|w|{let mut w=w.borrow_mut();let input=String::from_utf8_lossy(&w.input[..(n as usize).min(w.input.len())]).into_owned();let out=match w.sessions.get_mut(id as usize).and_then(Option::as_mut){Some(s)=>run(s,&input),None=>json!({"error":"Invalid session"}).to_string()};w.output=out.into_bytes();w.output.as_ptr()})}
#[cfg(test)] #[path="../../tfm2_custom_ai/src/grimhald_core_tests.rs"] mod core_tests;

#[cfg(test)] mod transport_tests {
    use super::*;
    fn input(t:u64)->serde_json::Value {json!({"version":1,"rules_revision":REVISION,"frame":{"tick":t,"owner":1,"team":0,"alive":true,"pos":{"x":100000,"y":100000},"hp":1000,"max_hp":1000,"attack":100,"move_speed":1000,"hard_cc_ticks":0,"ult_unlocked":true,"units":[],"terrain":[],"walls":[],"profile":(vec![50;13]),"danger":false,"cage":null,"manual":true,"order":null}})}
    #[test] fn version_and_bounds_do_not_mutate(){let mut s=Session::default();for key in ["version","rules_revision"] {let mut i=input(1);i[key]=json!(0);assert!(run(&mut s,&i.to_string()).contains("error"));assert!(s.last_tick.is_none());}let mut i=input(1);i["frame"]["profile"][0]=json!(101);assert!(run(&mut s,&i.to_string()).contains("error"));assert!(s.last_tick.is_none());}
    #[test] fn duplicate_tick_is_atomic(){let mut s=Session::default();let i=input(10).to_string();assert!(!run(&mut s,&i).contains("error"));let before=serde_json::to_string(&s.core).unwrap();assert!(run(&mut s,&i).contains("error"));assert_eq!(before,serde_json::to_string(&s.core).unwrap());}
    #[test] fn sessions_remain_independent(){let mut a=Session::default();let mut b=Session::default();let mut i=input(1);i["frame"]["order"]=json!({"kind":"ghost","aim":{"x":130000,"y":100000}});run(&mut a,&i.to_string());run(&mut b,&input(1).to_string());assert!(a.core.ghost.is_some());assert!(b.core.ghost.is_none());}
    #[test] fn death_cleans_lab_host_state_not_cooldowns(){let mut s=Session::default();s.core.ready=[50,60,70,80];s.cage=Some(Cage{owner:1,center:Point::new(100000,100000),radius:55000,until:100});s.members.insert(1);let mut i=input(2);i["frame"]["alive"]=json!(false);i["frame"]["hp"]=json!(0);run(&mut s,&i.to_string());assert!(s.cage.is_none());assert!(s.members.is_empty());assert_eq!(s.core.ready,[50,60,70,80]);}
}
