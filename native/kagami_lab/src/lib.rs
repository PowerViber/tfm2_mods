//! Kagami Lab transport: the game's exact core (`kagami_core.rs`) behind a JSON step for the editor page.
//! The page models the host (positions, glides, swaps, seals, hits); the rules and her AI all come from the core.
#[path = "../../tfm2_custom_ai/src/kagami_core.rs"]
pub mod kagami_core;
#[path = "../../tfm2_custom_ai/src/kagami_ai.rs"]
pub mod kagami_ai;
#[path = "../../tfm2_custom_ai/src/kagami_data.rs"]
pub mod kagami_data;
use kagami_core::*;
use serde::Deserialize;
use serde_json::json;
use std::cell::RefCell;

pub const REVISION: &str = kagami_data::RULES_REVISION;

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
        && f.units.len() <= 128
        && (f.terrain.is_empty() || f.terrain.len() == 900)
        && f.hazards.len() <= 16
        && f.hazards.iter().all(|h| point(h.at) && (0..=960_000).contains(&h.radius))
        && f.profile.iter().all(|v| (0..=100).contains(v))
        && (1..=100_000_000).contains(&f.max_hp)
        && (0..=f.max_hp).contains(&f.hp)
        && f.clone.is_none_or(|c| point(c.pos) && c.id != f.owner && (1..=100_000_000).contains(&c.max_hp))
        && f.units.iter().all(|u| u.id != f.owner && point(u.pos) && (1..=100_000_000).contains(&u.max_hp))
}

pub fn run(session: &mut Session, text: &str) -> String {
    let Ok(input) = serde_json::from_str::<Input>(text) else { return json!({"error": "Invalid Kagami Lab v1 input"}).to_string() };
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
pub extern "C" fn kagami_reset() {
    WASM.with(|w| w.borrow_mut().session = Some(Session::default()));
}
#[no_mangle]
pub extern "C" fn kagami_input(n: u32) -> *mut u8 {
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
pub extern "C" fn kagami_output_len() -> u32 {
    WASM.with(|w| w.borrow().output.len() as u32)
}
#[no_mangle]
pub extern "C" fn kagami_step(n: u32) -> *const u8 {
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
        json!({"version":1,"rules_revision":REVISION,"frame":{"tick":t,"owner":1,"team":0,"alive":true,"pos":{"x":300000,"y":300000},"hp":1000,"max_hp":1000,"attack":100,"stunned":false,"ult_unlocked":true,"units":[],"profile":vec![50;13],"manual":true,"order":null}})
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
    fn the_first_step_asks_for_the_clone() {
        let mut s = Session::default();
        let out = run(&mut s, &input(1).to_string());
        assert!(out.contains("spawn_clone"), "{out}");
    }
}

#[cfg(test)]
mod effect_tests {
    use super::*;
    /// every effect must serialize for the page (an internally tagged newtype of a string can't)
    #[test]
    fn every_effect_serializes() {
        let p = Point::new(1, 2);
        for e in [
            Effect::SpawnClone { at: p, hp: 1 },
            Effect::CloneMove { to: p, ticks: 1 },
            Effect::CloneShot { target: 1, amount: 1, from: p },
            Effect::Heal { target: 1, amount: 1, from: p },
            Effect::Charm { target: 1, ticks: 1 },
            Effect::SwapLock { a: 1, b: 2, ticks: 1 },
            Effect::Swap { a: 1, b: 2 },
            Effect::Stun { target: 1, ticks: 1 },
            Effect::Knockback { target: 1 },
            Effect::Shield { target: 1, amount: 1, ticks: 1 },
            Effect::Ward { target: 1, ticks: 1 },
            Effect::SelfWard { ticks: 1 },
            Effect::Seal { target: 1, ticks: 1 },
            Effect::Revive { target: 1, hp: 1, blessed: true },
            Effect::Steer { to: p },
            Effect::Fx { name: "x".into(), at: p, ticks: 1 },
            Effect::Log { text: "x".into() },
        ] {
            assert!(serde_json::to_value(&e).is_ok(), "{e:?}");
        }
    }
}
