use std::io::{self, BufRead};
fn main() {
    let mut core = raze_lab::raze_core::Core::default();
    for line in io::stdin().lock().lines() {
        let s = line.unwrap();
        if s == "reset" {
            core = Default::default();
            println!("{{\"reset\":true}}");
        } else {
            println!("{}", raze_lab::run(&mut core, &s));
        }
    }
}
