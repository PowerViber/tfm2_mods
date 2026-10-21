#![allow(dead_code)]
pub mod legacy_v1;
#[path = "../../tfm2_custom_ai/src/steve_craft_core.rs"]
pub mod steve_craft_core;
#[path = "../../tfm2_custom_ai/src/steve_craft_data.rs"]
pub mod steve_craft_data;
#[path = "../../tfm2_custom_ai/src/steve_craft_geometry.rs"]
pub mod steve_craft_geometry;

#[path = "../../tfm2_custom_ai/src/steve_craft_protocol.rs"]
pub mod steve_craft_protocol;
#[cfg(test)]
#[path = "../../tfm2_custom_ai/src/steve_craft_tests.rs"]
mod tests;
use std::cell::RefCell;
#[derive(Default)]
struct Session {
    cores: Vec<Option<steve_craft_core::Core>>,
    legacy: Vec<legacy_v1::steve_craft_core::Core>,
    input: Vec<i64>,
    output: Vec<u8>,
}
thread_local! {static SESSION:RefCell<Session>=RefCell::new(Session::default());}
#[no_mangle]
pub extern "C" fn steve_new() -> u32 {
    SESSION.with(|s| {
        let mut s = s.borrow_mut();
        if let Some(i) = s.cores.iter().position(Option::is_none) {
            s.cores[i] = Some(Default::default());
            s.legacy[i]=Default::default();
            i as u32
        } else {
            let i = s.cores.len();
            s.cores.push(Some(Default::default()));
            s.legacy.push(Default::default());
            i as u32
        }
    })
}
#[no_mangle]
pub extern "C" fn steve_drop(id: u32) {
    SESSION.with(|s| {
        if let Some(c) = s.borrow_mut().cores.get_mut(id as usize) {
            *c = None;
        }
    })
}
#[no_mangle]
pub extern "C" fn steve_input(n: u32) -> *mut i64 {
    SESSION.with(|s| {
        let mut s = s.borrow_mut();
        if n > 30000 {
            return std::ptr::null_mut();
        }
        s.input.resize(n as usize, 0);
        s.input.as_mut_ptr()
    })
}
#[no_mangle]
pub extern "C" fn steve_output_len() -> u32 {
    SESSION.with(|s| s.borrow().output.len() as u32)
}
#[no_mangle]
pub extern "C" fn steve_step(id: u32, n: u32) -> *const u8 {
    SESSION.with(|s| {
        let mut s = s.borrow_mut();
        let a = s.input.get(..n as usize).unwrap_or_default().to_vec();
        let output = if a.first()==Some(&1) {
            match (s.legacy.get_mut(id as usize),legacy_v1::steve_craft_protocol::Frame::decode(&a)){
                (Some(c),Some(f))=>f.run(c),_=>"{\"error\":\"Invalid legacy Steve frame\"}".into()
            }
        }else {match (
            s.cores.get_mut(id as usize).and_then(Option::as_mut),
            steve_craft_protocol::Frame::decode(&a),
        ) {
            (Some(c), Some(f)) => f.run(c),
            _ => "{\"error\":\"Invalid SteveCraft frame or session\"}".into(),
        }};
        s.output = output.into_bytes();
        s.output.as_ptr()
    })
}
