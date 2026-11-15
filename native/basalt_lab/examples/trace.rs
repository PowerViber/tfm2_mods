//! Read-only native transport runner for exact WASM comparison. No game process.
use std::io::{self, BufRead};
fn main() {
    let mut session=basalt_lab::Session::default();
    for line in io::stdin().lock().lines() {
        let line=line.expect("read fixture");
        if line=="reset" {session=basalt_lab::Session::default();println!("{{\"reset\":true}}");}
        else {println!("{}",basalt_lab::run(&mut session,&line));}
    }
}
