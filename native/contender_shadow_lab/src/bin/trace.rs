//! Read-only JSON-lines parity runner: one independent session per process.
use std::io::{self, BufRead};
fn main() {
    let mut session = contender_shadow_lab::Session::default();
    for line in io::stdin().lock().lines() {
        println!("{}", contender_shadow_lab::run(&mut session, &line.expect("input")));
    }
}
