use std::io::{self, BufRead};
fn main() {
    let mut cores: Vec<gojo_lab::gojo_core::Core> = (0..16).map(|_| Default::default()).collect();
    let mut labs:Vec<gojo_lab::athlete_ai_protocol::Lab>=(0..16).map(|_|Default::default()).collect();
    for line in io::stdin().lock().lines() {
        let line = line.unwrap();
        let mut a = line.split_whitespace();
        let first=a.next().unwrap();
        if first=="T" {let id:usize=a.next().unwrap().parse().unwrap();let command:u32=a.next().unwrap().parse().unwrap();let v:Vec<i64>=a.map(|n|n.parse().unwrap()).collect();println!("{}",labs[id].run(&v,command));continue}
        let id: usize = first.parse().unwrap();
        let v: Vec<i64> = a.map(|n| n.parse().unwrap()).collect();
        println!(
            "{}",
            gojo_lab::gojo_protocol::Frame::decode(&v)
                .map_or("{\"error\":\"Invalid Gojo frame\"}".into(), |f| f
                    .run(&mut cores[id]))
        );
    }
}
