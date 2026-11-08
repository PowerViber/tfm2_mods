//! JSON-lines runner for complete native/WASM Lab trace comparison.
use std::io::{self, BufRead};
fn main() {
    let mut session=thalassa_lab::Session::default();
    for line in io::stdin().lock().lines() {
        let line=line.expect("read fixture line");
        if !line.trim().is_empty() { println!("{}",thalassa_lab::run(&mut session,&line)); }
    }
}
