//! The Lab runs the game's own core: a Crushing Clap through the JSON transport.
use basalt_lab::{basalt_data as d, run, Session, REVISION};
use serde_json::json;

#[test]
fn a_crushing_clap_through_the_transport_stuns_inside_the_v() {
    let mut s = Session::default();
    let foe = json!({"id":10,"team":1,"pos":{"x":400000,"y":340000},"hp":1000,"max_hp":1000,"champion":true,"tower":false,"visible":true,"targetable":true});
    let frame = json!({"tick":1,"owner":1,"team":0,"alive":true,"pos":{"x":400000,"y":400000},"hp":1000,"max_hp":1000,"attack":100,"ult_unlocked":true,
        "units":[foe],"profile":vec![50;13],"manual":true,"order":{"kind":"s3","aim":{"x":400000,"y":300000}}});
    let out: serde_json::Value = serde_json::from_str(&run(&mut s, &json!({"version":1,"rules_revision":REVISION,"frame":frame}).to_string())).unwrap();
    let fx = out["effects"].as_array().unwrap();
    assert!(fx.iter().any(|e| e["type"] == "stun" && e["target"] == 10), "{out}");
    assert_eq!(s.core.walls.len(), 2);
    let _ = d::CLAP_REACH;
}
