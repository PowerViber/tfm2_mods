//! The Lab runs the game's own core: a kick on the real rules, end to end through the JSON transport.
use kicker_lab::{kicker_core::*, kicker_data as d, run, Session, REVISION};
use serde_json::json;

fn step(s: &mut Session, frame: serde_json::Value) -> serde_json::Value {
    serde_json::from_str(&run(s, &json!({"version":1,"rules_revision":REVISION,"frame":frame}).to_string())).unwrap()
}

#[test]
fn a_kick_through_the_transport_sends_the_target_away_and_spends_a_charge() {
    let mut s = Session::default();
    let unit = json!({"id":10,"team":1,"pos":{"x":412000,"y":400000},"hp":1000,"max_hp":1000,"radius":3000,"champion":true,"tower":false,"visible":true,"targetable":true});
    let mut frame = json!({"tick":1,"owner":1,"team":0,"alive":true,"pos":{"x":400000,"y":400000},"hp":1500,"max_hp":1500,"attack":150,"stunned":false,"ult_unlocked":true,
        "units":[unit],"profile":vec![50;13],"manual":true,"order":{"kind":"kick","aim":[10,{"x":300000,"y":400000}]}});
    step(&mut s, frame.clone());
    assert_eq!(s.core.kicks, d::KICK_CHARGES - 1);
    let mut knocked = None;
    for tick in 2..60 {
        frame["tick"] = json!(tick);
        frame["order"] = json!(null);
        if let Some(m) = &s.core.moving { frame["pos"] = json!({"x": m.land.x, "y": m.land.y}); }
        let out = step(&mut s, frame.clone());
        for e in out["effects"].as_array().unwrap() {
            if e["kind"] == "knock" { knocked = Some(e["data"]["to"]["x"].as_i64().unwrap()); }
        }
    }
    let x = knocked.expect("a knock effect");
    assert!(x < 412_000 - d::KICK_DISTANCE + 3_000, "{x}");
    let _ = Point::new(0, 0);
}
