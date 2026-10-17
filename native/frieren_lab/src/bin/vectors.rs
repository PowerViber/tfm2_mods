use frieren_lab::{frieren_core::Core, frieren_protocol::Frame};
use std::io::{self, BufRead};
fn main() {
    let mut c = Core::default();
    for line in io::stdin().lock().lines() {
        let line = line.unwrap();
        if line == "reset" {
            c = Core::default();
            continue;
        }
        let input: Vec<i64> = line
            .split_whitespace()
            .map(|n| n.parse().unwrap())
            .collect();
        println!(
            "{}",
            Frame::decode(&input).expect("valid frame").run(&mut c)
        );
    }
}
