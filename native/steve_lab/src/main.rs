use std::io::{self, BufRead};
fn main() {
    let mut cores: Vec<steve_lab::steve_craft_core::Core> =
        (0..16).map(|_| Default::default()).collect();
    let mut legacy:Vec<steve_lab::legacy_v1::steve_craft_core::Core>=(0..16).map(|_|Default::default()).collect();
    for line in io::stdin().lock().lines() {
        let Ok(line)=line else{break};
        let mut a = line.split_whitespace();
        let Some(id)=a.next().and_then(|s|s.parse::<usize>().ok()).filter(|n|*n<16)else{println!("{{\"error\":\"Invalid session\"}}");continue};
        let Ok(v)=a.map(str::parse::<i64>).collect::<Result<Vec<_>,_>>()else{println!("{{\"error\":\"Invalid integer\"}}");continue};
        if v.first()==Some(&1){println!("{}",steve_lab::legacy_v1::steve_craft_protocol::Frame::decode(&v).map_or("{\"error\":\"Invalid legacy frame\"}".into(),|f|f.run(&mut legacy[id])));continue;}
        println!(
            "{}",
            steve_lab::steve_craft_protocol::Frame::decode(&v)
                .map_or("{\"error\":\"Invalid frame\"}".into(), |f| f
                    .run(&mut cores[id]))
        );
    }
}
