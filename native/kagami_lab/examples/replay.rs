//! JSON-lines trace oracle for native/WASM parity; no alternate combat implementation.
use std::io::{self, BufRead};
fn main() {
    if let Some(frame) = std::env::args().nth(1) {
        let input = serde_json::json!({"version":1,"rules_revision":kagami_lab::REVISION,"frame":serde_json::from_str::<serde_json::Value>(&frame).expect("frame JSON")});
        println!("{}", kagami_lab::run(&mut kagami_lab::Session::default(), &input.to_string()));
        return;
    }
    for line in io::stdin().lock().lines() {
        let line = line.expect("stdin");
        let inputs: Vec<serde_json::Value> = serde_json::from_str(&line).expect("array of transport inputs");
        let mut session = kagami_lab::Session::default();
        let results: Vec<serde_json::Value> = inputs
            .iter()
            .map(|input| serde_json::from_str(&kagami_lab::run(&mut session, &input.to_string())).expect("result"))
            .collect();
        println!("{}", serde_json::to_string(&results).unwrap());
    }
}
