use std::io::{self, BufRead};
fn main() {
    let mut core = contender_lab::contender_core::Core::default();
    for line in io::stdin().lock().lines() {
        match line {
            Ok(s) => println!("{}", contender_lab::run(&mut core, &s)),
            Err(_) => break,
        }
    }
}
