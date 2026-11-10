//! Wright Lab transport: the game's exact core (`wright_core.rs`) behind a JSON step for the editor page.
//! The page models the host (positions, glides, the plane carriers); the rules all come from the core.
#[path = "../../tfm2_custom_ai/src/wright_core.rs"]
pub mod wright_core;
#[path = "../../tfm2_custom_ai/src/wright_data.rs"]
pub mod wright_data;
#[path = "../../tfm2_custom_ai/src/wright_art.rs"]
pub mod wright_art;
use serde::Deserialize;
use serde_json::json;
use std::cell::RefCell;
use wright_core::*;

pub const REVISION: &str = wright_data::RULES_REVISION;

#[derive(Deserialize)]
pub struct Input {
    pub version: u8,
    pub rules_revision: String,
    pub frame: Frame,
}

#[derive(Default)]
pub struct Session {
    pub core: Core,
    last_tick: Option<u64>,
}

fn valid(f: &Frame) -> bool {
    let point = |p: Point| p.on_map();
    point(f.pos)
        && f.units.len() <= 64
        && (f.terrain.is_empty() || f.terrain.len() == 900)
        && f.profile.iter().all(|v| (0..=100).contains(v))
        && (1..=100_000_000).contains(&f.max_hp)
        && (0..=f.max_hp).contains(&f.hp)
        && f.units.iter().all(|u| u.id != f.owner && point(u.pos) && (1..=100_000).contains(&u.radius) && (1..=100_000_000).contains(&u.max_hp))
        && f.planes.len() <= 32
        && f.planes.iter().all(|p| point(p.pos) && (1..=100_000_000).contains(&p.max_hp))
}

pub fn run(session: &mut Session, text: &str) -> String {
    let Ok(input) = serde_json::from_str::<Input>(text) else { return json!({"error": "Invalid Wright Lab v1 input"}).to_string() };
    let f = input.frame;
    if input.version != 1 || input.rules_revision != REVISION || !valid(&f) || session.last_tick.is_some_and(|t| f.tick <= t) {
        return json!({"error": "Current rules, valid bounds and increasing ticks required"}).to_string();
    }
    session.last_tick = Some(f.tick);
    if !f.alive {
        session.core.respawn(f.tick);
    }
    let effects = session.core.step(&f);
    json!({"version": 1, "rules_revision": REVISION, "state": session.core, "effects": effects}).to_string()
}

#[derive(Default)]
struct Wasm {
    session: Option<Session>,
    input: Vec<u8>,
    output: Vec<u8>,
}
thread_local! { static WASM: RefCell<Wasm> = RefCell::new(Wasm::default()); }
#[no_mangle]
pub extern "C" fn wright_reset() {
    WASM.with(|w| w.borrow_mut().session = Some(Session::default()));
}
#[no_mangle]
pub extern "C" fn wright_input(n: u32) -> *mut u8 {
    WASM.with(|w| {
        if n > 2_000_000 {
            return std::ptr::null_mut();
        }
        let mut w = w.borrow_mut();
        w.input.resize(n as usize, 0);
        w.input.as_mut_ptr()
    })
}
#[no_mangle]
pub extern "C" fn wright_output_len() -> u32 {
    WASM.with(|w| w.borrow().output.len() as u32)
}
#[no_mangle]
pub extern "C" fn wright_step(n: u32) -> *const u8 {
    WASM.with(|w| {
        let mut w = w.borrow_mut();
        let input = String::from_utf8_lossy(&w.input[..(n as usize).min(w.input.len())]).into_owned();
        let out = run(w.session.get_or_insert_with(Session::default), &input);
        w.output = out.into_bytes();
        w.output.as_ptr()
    })
}

#[cfg(test)]
#[path = "../../tfm2_custom_ai/src/wright_core_tests.rs"]
mod core_tests;

#[cfg(test)]
mod transport_tests {
    use super::*;
    fn input(t: u64) -> serde_json::Value {
        json!({"version":1,"rules_revision":REVISION,"frame":{"tick":t,"owner":1,"team":0,"alive":true,"pos":{"x":300000,"y":300000},"hp":1000,"max_hp":1000,"ap":100,"stunned":false,"ult_unlocked":true,"units":[],"terrain":[],"profile":vec![50;13],"manual":true,"order":null}})
    }
    #[test]
    fn rejects_wrong_revision_and_repeated_ticks() {
        let mut s = Session::default();
        let mut bad = input(1);
        bad["rules_revision"] = json!("old");
        assert!(run(&mut s, &bad.to_string()).contains("error"));
        assert!(!run(&mut s, &input(1).to_string()).contains("error"));
        assert!(run(&mut s, &input(1).to_string()).contains("error"));
    }
    #[test]
    fn an_order_reaches_the_core() {
        let mut s = Session::default();
        let mut i = input(5);
        i["frame"]["order"] = json!({"kind":"carrier","aim":{"x":350000,"y":300000}});
        let out = run(&mut s, &i.to_string());
        assert!(s.core.carrier.is_some(), "{out}");
    }
}
