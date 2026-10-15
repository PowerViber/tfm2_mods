//! Checked version 2 transport around the exact native combat core.
#![allow(dead_code)]
#[path = "../../tfm2_custom_ai/src/magnus_core.rs"]
mod magnus_core;
#[path = "../../tfm2_custom_ai/src/magnus_data.rs"]
mod magnus_data;
#[path = "../../tfm2_custom_ai/src/magnus_fixtures.rs"]
mod magnus_fixtures;
#[path = "../../tfm2_custom_ai/src/magnus_protocol.rs"]
mod magnus_protocol;
use magnus_core::*;
use magnus_protocol::Frame;
use std::cell::RefCell;
#[derive(Default)]
struct Session {
    cores: Vec<Option<Core>>,
    frames: std::collections::BTreeMap<u32,Frame>,
    input: Vec<i64>,
    output: Vec<u8>,
}
thread_local! {static SESSION:RefCell<Session>=RefCell::new(Session::default());}
#[no_mangle]
pub extern "C" fn magnus_version() -> u32 {
    2
}
fn emit(s: &mut Session, value: String) -> *const u8 {
    s.output = value.into_bytes();
    s.output.as_ptr()
}
#[no_mangle]
pub extern "C" fn magnus_new(lo: u32, hi: u32, owner: u32, rank: u32) -> u32 {
    SESSION.with(|s| {
        let mut s = s.borrow_mut();
        let core = Core::new((hi as u64) << 32 | lo as u64, owner as usize, rank as usize);
        if let Some(i) = s.cores.iter().position(Option::is_none) {
            s.cores[i] = Some(core);
            s.frames.remove(&(i as u32));
            i as u32
        } else {
            let n = s.cores.len();
            s.cores.push(Some(core));
            n as u32
        }
    })
}
#[no_mangle]
pub extern "C" fn magnus_drop(handle: u32) {
    SESSION.with(|s| {
        let mut session=s.borrow_mut();session.frames.remove(&handle);
        if let Some(c) = session.cores.get_mut(handle as usize) {
            *c = None;
        }
    })
}
#[no_mangle]
pub extern "C" fn magnus_input(len: u32) -> *mut i64 {
    SESSION.with(|s| {
        let mut s = s.borrow_mut();
        if len > 2_000_000 {
            return std::ptr::null_mut();
        }
        s.input.resize(len as usize, 0);
        s.input.as_mut_ptr()
    })
}
#[no_mangle]
pub extern "C" fn magnus_output_len() -> u32 {
    SESSION.with(|s| s.borrow().output.len() as u32)
}
#[no_mangle]
pub extern "C" fn magnus_step(handle: u32, len: u32) -> *const u8 {
    SESSION.with(|s| {
        let mut s = s.borrow_mut();
        let output = (|| {
            let frame = Frame::decode(s.input.get(..len as usize)?)?;
            let core = s.cores.get_mut(handle as usize)?.as_mut()?;
            if core.owner != frame.owner.id {
                return None;
            }
            let output=frame.run(core).0;
            s.frames.insert(handle,frame);
            Some(output)
        })()
        .unwrap_or_else(|| "{\"error\":\"Invalid version 2 practice frame or session\"}".into());
        emit(&mut s, output)
    })
}
#[no_mangle]
pub extern "C" fn magnus_fixture(handle: u32, id: u32) -> *const u8 {
    SESSION.with(|s| {
        let mut s = s.borrow_mut();
        let output = if let Some(slot) = s.cores.get_mut(handle as usize) {
            if let Some(f) = magnus_fixtures::fixture(id as usize) {
                let output = f.json();
                *slot = Some(f.core);
                s.frames.insert(handle,f.frame);
                output
            } else {
                "{\"error\":\"Unknown historical fixture\"}".into()
            }
        } else {
            "{\"error\":\"Invalid session\"}".into()
        };
        emit(&mut s, output)
    })
}
#[no_mangle]
pub extern "C" fn magnus_piece(handle: u32, id: u32, x: i32, y: i32, hp: i32) -> u32 {
    if !(0..40).contains(&x) || !(0..40).contains(&y) { return 0; }
    SESSION.with(|s| {
        let mut s = s.borrow_mut();
        let Some(core) = s.cores.get_mut(handle as usize).and_then(Option::as_mut) else {
            return 0;
        };
        let Some(p) = core.pieces.get_mut(&(id as usize)) else {
            return 0;
        };
        p.cell = Cell::new(x, y);
        p.hp = (hp as i64).clamp(1, p.max_hp);
        core.challenge = None;
        core.pending = None;
        core.has_move = false;
        core.reindex();
        1
    })
}
#[no_mangle]
pub extern "C" fn magnus_add(handle: u32, kind: u32, x: i32, y: i32, direction: u32) -> u32 {
    if !(0..40).contains(&x) || !(0..40).contains(&y) { return u32::MAX; }
    SESSION.with(|s| {
        let mut s = s.borrow_mut();
        let Some(core) = s.cores.get_mut(handle as usize).and_then(Option::as_mut) else {
            return u32::MAX;
        };
        let Some(kind) = Kind::from_code(kind as usize) else {
            return u32::MAX;
        };
        core.challenge = None;
        core.pending = None;
        core.has_move = false;
        core.add_piece(
            kind,
            Cell::new(x, y),
            800 * kind.hp_pct() / 100,
            None,
            direction as u8,
        ) as u32
    })
}
#[no_mangle]
pub extern "C" fn magnus_challenge(handle: u32, id: u32) -> u32 {
    SESSION.with(|s| {
        let mut s = s.borrow_mut();
        let Some(frame) = s.frames.get(&handle).cloned() else { return 0; };
        let Some(core) = s.cores.get_mut(handle as usize).and_then(Option::as_mut) else {
            return 0;
        };
        if id as usize >= magnus_data::GAMES.len() {
            return 0;
        }
        (frame.inspection(core, Some(id as usize), None) == "true") as u32
    })
}
#[no_mangle]
pub extern "C" fn magnus_coverage(handle: u32, piece: u32) -> *const u8 {
    SESSION.with(|s| {
        let mut s = s.borrow_mut();
        let frame = s.frames.get(&handle).cloned();
        let output = match (frame, s.cores.get_mut(handle as usize).and_then(Option::as_mut)) {
            (Some(frame), Some(core)) => frame.inspection(core, None, Some(piece as usize)),
            _ => "[]".into(),
        };
        emit(&mut s, output)
    })
}
