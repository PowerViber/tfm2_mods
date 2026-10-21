#![allow(dead_code)]
#[path="../../tfm2_custom_ai/src/game_sense_core.rs"]pub mod game_sense_core;
#[path="../../tfm2_custom_ai/src/strategic_core.rs"]pub mod strategic_core;
#[path="../../tfm2_custom_ai/src/strategic_policy.rs"]pub mod strategic_policy;
#[path="../../tfm2_custom_ai/src/strategic_protocol.rs"]pub mod strategic_protocol;
use std::cell::RefCell;
#[derive(Default)]struct State{input:Vec<i64>,output:Vec<u8>,labs:Vec<strategic_protocol::Lab>}
thread_local!{static S:RefCell<State>=RefCell::new(Default::default());}
#[no_mangle]pub extern "C" fn universal_new()->u32{S.with(|s|{let mut s=s.borrow_mut();let id=s.labs.len();s.labs.push(Default::default());id as u32})}
#[no_mangle]pub extern "C" fn universal_input(n:u32)->*mut i64{S.with(|s|{let mut s=s.borrow_mut();if n>8192{return std::ptr::null_mut();}s.input.resize(n as usize,0);s.input.as_mut_ptr()})}
#[no_mangle]pub extern "C" fn universal_step(id:u32,n:u32)->*const u8{S.with(|s|{let mut s=s.borrow_mut();let a=s.input.get(..n as usize).unwrap_or_default().to_vec();let out=s.labs.get_mut(id as usize).map_or("{\"error\":\"Invalid session\"}".into(),|l|l.run(&a));s.output=out.into_bytes();s.output.as_ptr()})}
#[no_mangle]pub extern "C" fn universal_len()->u32{S.with(|s|s.borrow().output.len()as u32)}

#[path="../../tfm2_custom_ai/src/strategic_routes.rs"]pub mod strategic_routes;
