//! JSON-lines runner for complete native/WASM Lab trace comparison.
use std::io::{self, BufRead};
#[allow(dead_code)]
#[path = "../../../tfm2_custom_ai/src/thalassa_art.rs"]
mod art;
fn main() {
    let mut session=thalassa_lab::Session::default();
    let links=std::env::args().any(|v|v=="--links");
    for line in io::stdin().lock().lines() {
        let line=line.expect("read fixture line");
        if !line.trim().is_empty() {
            if links {
                let points: [[i64;2];2]=serde_json::from_str(&line).expect("two endpoints");
                println!("{}",serde_json::to_string(&art::link_segments((points[0][0],points[0][1]),(points[1][0],points[1][1]))).unwrap());
            } else { println!("{}",thalassa_lab::run(&mut session,&line)); }
        }
    }
}
