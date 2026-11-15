//! Basalt Lab transport: the game's exact core (`basalt_core.rs`) behind a JSON step for the editor page.
//! The page models the host (positions, glides, his walls, simple enemies); the rules and his AI come from the core.
#[path = "../../tfm2_custom_ai/src/basalt_core.rs"]
pub mod basalt_core;
#[path = "../../tfm2_custom_ai/src/basalt_data.rs"]
pub mod basalt_data;
use basalt_core::*;
use serde::Deserialize;
use serde_json::json;
use std::cell::RefCell;

pub const REVISION: &str = basalt_data::RULES_REVISION;

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
    let grid = |g: &Vec<bool>| g.is_empty() || g.len() == (CELLS * CELLS) as usize;
    point(f.pos)
        && f.units.len() <= 64
        && f.other_walls.len() <= 32
        && grid(&f.terrain)
        && f.profile.iter().all(|v| (0..=100).contains(v))
        && (1..=100_000_000).contains(&f.max_hp)
        && (0..=f.max_hp).contains(&f.hp)
        && f.units.iter().all(|u| u.id != f.owner && point(u.pos) && (1..=100_000_000).contains(&u.max_hp))
}

pub fn run(session: &mut Session, text: &str) -> String {
    let Ok(input) = serde_json::from_str::<Input>(text) else { return json!({"error": "Invalid Basalt Lab v1 input"}).to_string() };
    let f = input.frame;
    if input.version != 1 || input.rules_revision != REVISION || !valid(&f) || session.last_tick.is_some_and(|t| f.tick <= t) {
        return json!({"error": "Current rules, valid bounds and increasing ticks required"}).to_string();
    }
    session.last_tick = Some(f.tick);
    let effects = session.core.step(&f);
    json!({"version": 1, "rules_revision": REVISION, "state": session.core, "effects": effects, "rejection": session.core.rejection}).to_string()
}

#[derive(Default)]
struct Wasm {
    session: Option<Session>,
    input: Vec<u8>,
    output: Vec<u8>,
}
thread_local! { static WASM: RefCell<Wasm> = RefCell::new(Wasm::default()); }
#[no_mangle]
pub extern "C" fn basalt_reset() {
    WASM.with(|w| w.borrow_mut().session = Some(Session::default()));
}
#[no_mangle]
pub extern "C" fn basalt_input(n: u32) -> *mut u8 {
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
pub extern "C" fn basalt_output_len() -> u32 {
    WASM.with(|w| w.borrow().output.len() as u32)
}
#[no_mangle]
pub extern "C" fn basalt_step(n: u32) -> *const u8 {
    WASM.with(|w| {
        let mut w = w.borrow_mut();
        let input = String::from_utf8_lossy(&w.input[..(n as usize).min(w.input.len())]).into_owned();
        let out = run(w.session.get_or_insert_with(Session::default), &input);
        w.output = out.into_bytes();
        w.output.as_ptr()
    })
}

#[cfg(test)]
mod transport_tests {
    use super::*;
    fn input(t: u64) -> serde_json::Value {
        json!({"version":1,"rules_revision":REVISION,"frame":{"tick":t,"owner":1,"team":0,"alive":true,"pos":{"x":400000,"y":400000},"hp":1000,"max_hp":1000,"attack":100,"ult_unlocked":true,"units":[],"profile":vec![50;13],"manual":true,"order":null}})
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
    fn every_order_reaches_the_core_and_every_effect_serializes() {
        let mut s = Session::default();
        for (t, o) in [
            (1, json!({"kind":"s1","aim":{"x":400000,"y":300000}})),
            (2, json!({"kind":"jump","aim":{"x":400000,"y":350000}})),
            (3, json!({"kind":"s3","aim":{"x":400000,"y":300000}})),
            (4, json!({"kind":"ult","aim":{"x":800000,"y":100000}})),
        ] {
            let mut i = input(t);
            i["frame"]["order"] = o;
            let out = run(&mut s, &i.to_string());
            assert!(!out.contains("error"), "{out}");
        }
        assert!(s.core.charging.is_some());
    }
}
