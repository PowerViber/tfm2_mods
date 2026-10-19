#![allow(dead_code)]
#[path = "../../tfm2_custom_ai/src/gojo_core.rs"]
pub mod gojo_core;
#[path = "../../tfm2_custom_ai/src/gojo_data.rs"]
pub mod gojo_data;
#[path = "../../tfm2_custom_ai/src/gojo_protocol.rs"]
pub mod gojo_protocol;
#[path = "../../tfm2_custom_ai/src/unified_theory_math.rs"]
pub mod unified_theory_math;
use std::cell::RefCell;
#[derive(Default)]
struct Session {
    cores: Vec<Option<gojo_core::Core>>,
    input: Vec<i64>,
    output: Vec<u8>,
}
thread_local! {static SESSION:RefCell<Session>=RefCell::new(Session::default());}
#[no_mangle]
pub extern "C" fn gojo_new() -> u32 {
    SESSION.with(|s| {
        let mut s = s.borrow_mut();
        if let Some(i) = s.cores.iter().position(Option::is_none) {
            s.cores[i] = Some(Default::default());
            i as u32
        } else {
            let i = s.cores.len();
            s.cores.push(Some(Default::default()));
            i as u32
        }
    })
}
#[no_mangle]
pub extern "C" fn gojo_drop(id: u32) {
    SESSION.with(|s| {
        if let Some(c) = s.borrow_mut().cores.get_mut(id as usize) {
            *c = None
        }
    })
}
#[no_mangle]
pub extern "C" fn gojo_input(n: u32) -> *mut i64 {
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
pub extern "C" fn gojo_output_len() -> u32 {
    SESSION.with(|s| s.borrow().output.len() as u32)
}
#[no_mangle]
pub extern "C" fn gojo_step(id: u32, n: u32) -> *const u8 {
    SESSION.with(|s| {
        let mut s = s.borrow_mut();
        let a = s.input.get(..n as usize).unwrap_or_default().to_vec();
        let output = match (
            s.cores.get_mut(id as usize).and_then(Option::as_mut),
            gojo_protocol::Frame::decode(&a),
        ) {
            (Some(c), Some(f)) => f.run(c),
            _ => "{\"error\":\"Invalid Gojo frame or session\"}".into(),
        };
        s.output = output.into_bytes();
        s.output.as_ptr()
    })
}

#[cfg(test)]
#[path = "../../tfm2_custom_ai/src/gojo_tests.rs"]
mod gojo_tests;
