//! Kratos Lab transport: the game's exact core (`kratos_core.rs`) behind a JSON step for the editor page.
//! The page models the host (positions, glides, the real map's walls, simple enemies); the rules all come from the core.
#[path = "../../tfm2_custom_ai/src/kratos_core.rs"]
pub mod kratos_core;
#[path = "../../tfm2_custom_ai/src/kratos_data.rs"]
pub mod kratos_data;
use kratos_core::*;
use serde::Deserialize;
use serde_json::json;
use std::cell::RefCell;

pub const REVISION: &str = kratos_data::RULES_REVISION;

#[derive(Deserialize)]
pub struct Input {
    pub version: u8,
    pub rules_revision: String,
    pub frame: Frame,
    /// a hit on Kratos this tick from (attacker, enemy champion): Heimdall's Sight decides if it's dodged
    #[serde(default)]
    pub hits_on_me: Vec<(usize, bool)>,
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
        && (f.terrain.is_empty() || f.terrain.len() == (CELLS * CELLS) as usize)
        && f.profile.iter().all(|v| (0..=100).contains(v))
        && (1..=100_000_000).contains(&f.max_hp)
        && (0..=f.max_hp).contains(&f.hp)
        && f.units.iter().all(|u| u.id != f.owner && point(u.pos) && (1..=100_000).contains(&u.radius) && (1..=100_000_000).contains(&u.max_hp))
}

pub fn run(session: &mut Session, text: &str) -> String {
    let Ok(input) = serde_json::from_str::<Input>(text) else { return json!({"error": "Invalid Kratos Lab v1 input"}).to_string() };
    let f = input.frame;
    if input.version != 1 || input.rules_revision != REVISION || !valid(&f) || session.last_tick.is_some_and(|t| f.tick <= t) {
        return json!({"error": "Current rules, valid bounds and increasing ticks required"}).to_string();
    }
    session.last_tick = Some(f.tick);
    if !f.alive {
        session.core.respawn(f.tick);
    }
    let dodged: Vec<bool> = input.hits_on_me.iter().map(|&(a, champ)| session.core.dodge(f.tick, a, champ)).collect();
    let effects = session.core.step(&f);
    json!({"version": 1, "rules_revision": REVISION, "state": session.core, "effects": effects, "dodged": dodged, "rejection": session.core.rejection}).to_string()
}

#[derive(Default)]
struct Wasm {
    session: Option<Session>,
    input: Vec<u8>,
    output: Vec<u8>,
}
thread_local! { static WASM: RefCell<Wasm> = RefCell::new(Wasm::default()); }
#[no_mangle]
pub extern "C" fn kratos_reset() {
    WASM.with(|w| w.borrow_mut().session = Some(Session::default()));
}
#[no_mangle]
pub extern "C" fn kratos_input(n: u32) -> *mut u8 {
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
pub extern "C" fn kratos_output_len() -> u32 {
    WASM.with(|w| w.borrow().output.len() as u32)
}
#[no_mangle]
pub extern "C" fn kratos_step(n: u32) -> *const u8 {
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
        json!({"version":1,"rules_revision":REVISION,"frame":{"tick":t,"owner":1,"team":0,"alive":true,"pos":{"x":300000,"y":300000},"hp":1000,"max_hp":1000,"attack":100,"stunned":false,"ult_unlocked":true,"units":[],"terrain":[],"profile":vec![50;13],"manual":true,"order":null}})
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
    fn orders_and_dodges_reach_the_core() {
        let mut s = Session::default();
        let mut i = input(5);
        i["frame"]["order"] = json!({"kind":"sight"});
        run(&mut s, &i.to_string());
        let mut i = input(6);
        i["hits_on_me"] = json!([[7, true], [7, true]]);
        let out: serde_json::Value = serde_json::from_str(&run(&mut s, &i.to_string())).unwrap();
        assert_eq!(out["dodged"], json!([true, false]));
        let mut i = input(7);
        i["frame"]["order"] = json!({"kind":"guard"});
        run(&mut s, &i.to_string());
        assert_eq!(s.core.link.map(|l| l.0), Some(Link::Guard));
    }
}
