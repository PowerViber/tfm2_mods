//! The Lab runs the game's own core: a Mirror Step rescue on the real rules, end to end through the JSON transport.
use kagami_lab::{kagami_data as d, run, Session, REVISION};
use serde_json::json;

fn step(s: &mut Session, frame: &serde_json::Value) -> serde_json::Value {
    serde_json::from_str(&run(s, &json!({"version":1,"rules_revision":REVISION,"frame":frame}).to_string())).unwrap()
}

#[test]
fn a_mirror_step_through_the_transport_locks_on_then_swaps_and_stuns() {
    let mut s = Session::default();
    let ally = json!({"id":2,"team":0,"pos":{"x":480000,"y":400000},"hp":300,"max_hp":1000,"attack":120,"champion":true,"tower":false,"visible":true,"targetable":true});
    let foe = json!({"id":10,"team":1,"pos":{"x":490000,"y":400000},"hp":1000,"max_hp":1000,"champion":true,"tower":false,"visible":true,"targetable":true});
    let mut frame = json!({"tick":1,"owner":1,"team":0,"alive":true,"pos":{"x":400000,"y":400000},"hp":1000,"max_hp":1000,"attack":100,"ult_unlocked":false,
        "units":[ally, foe],"clone":{"id":99,"pos":{"x":380000,"y":400000},"hp":350,"max_hp":350},"profile":vec![50;13],"manual":true,
        "order":{"kind":"step_ally","aim":2}});
    let out = step(&mut s, &frame);
    assert!(out["effects"].as_array().unwrap().iter().any(|e| e["type"] == "swap_lock"), "{out}");
    let mut swapped = None;
    for tick in 2..(2 + d::SWAP_LOCK_TICKS as u64 + 2) {
        frame["tick"] = json!(tick);
        frame["order"] = json!(null);
        let out = step(&mut s, &frame);
        let fx = out["effects"].as_array().unwrap();
        if fx.iter().any(|e| e["type"] == "swap") {
            swapped = Some(tick);
            assert!(fx.iter().any(|e| e["type"] == "stun" && e["target"] == 10), "{out}");
        }
    }
    assert_eq!(swapped, Some(1 + d::SWAP_LOCK_TICKS as u64));
}
