use std::io::{self, BufRead};
fn main() {
    let mut cores: Vec<steve_lab::steve_craft_core::Core> =
        (0..16).map(|_| Default::default()).collect();
    for line in io::stdin().lock().lines() {
        let line = line.unwrap();
        let mut a = line.split_whitespace();
        let id: usize = a.next().unwrap().parse().unwrap();
        let v: Vec<i64> = a.map(|n| n.parse().unwrap()).collect();
        println!(
            "{}",
            steve_lab::steve_craft_protocol::Frame::decode(&v)
                .map_or("{\"error\":\"Invalid frame\"}".into(), |f| f
                    .run(&mut cores[id]))
        );
    }
}
