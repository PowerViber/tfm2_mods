//! Browser transport around the same deterministic core used by the game.
#![allow(dead_code)]
#[path = "../../tfm2_custom_ai/src/frieren_core.rs"]
pub mod frieren_core;
#[path = "../../tfm2_custom_ai/src/frieren_protocol.rs"]
pub mod frieren_protocol;
#[path = "../../tfm2_custom_ai/src/rework_data.rs"]
mod rework_data;
#[path = "../../tfm2_custom_ai/src/unified_theory_math.rs"]
mod unified_theory_math;
use frieren_core::Core;
use std::cell::RefCell;
#[derive(Default)]
struct Session {
    cores: Vec<Option<Core>>,
    input: Vec<i64>,
    output: Vec<u8>,
}
thread_local! {static SESSION:RefCell<Session>=RefCell::new(Session::default());}
#[no_mangle]
pub extern "C" fn frieren_new() -> u32 {
    SESSION.with(|s| {
        let mut s = s.borrow_mut();
        if let Some(n) = s.cores.iter().position(Option::is_none) {
            s.cores[n] = Some(Core::default());
            n as u32
        } else {
            let n = s.cores.len();
            s.cores.push(Some(Core::default()));
            n as u32
        }
    })
}
#[no_mangle]
pub extern "C" fn frieren_drop(id: u32) {
    SESSION.with(|s| {
        if let Some(c) = s.borrow_mut().cores.get_mut(id as usize) {
            *c = None;
        }
    })
}
#[no_mangle]
pub extern "C" fn frieren_input(n: u32) -> *mut i64 {
    SESSION.with(|s| {
        let mut s = s.borrow_mut();
        if n > 20000 {
            return std::ptr::null_mut();
        }
        s.input.resize(n as usize, 0);
        s.input.as_mut_ptr()
    })
}
#[no_mangle]
pub extern "C" fn frieren_output_len() -> u32 {
    SESSION.with(|s| s.borrow().output.len() as u32)
}
#[no_mangle]
pub extern "C" fn frieren_step(id: u32, n: u32) -> *const u8 {
    SESSION.with(|s| {
        let mut s = s.borrow_mut();
        let input = s.input.get(..n as usize).unwrap_or_default().to_vec();
        let out = match (
            s.cores.get_mut(id as usize).and_then(Option::as_mut),
            frieren_protocol::Frame::decode(&input),
        ) {
            (Some(c), Some(f)) => f.run(c),
            _ => "{\"error\":\"Invalid Frieren frame or session\"}".into(),
        };
        s.output = out.into_bytes();
        s.output.as_ptr()
    })
}
