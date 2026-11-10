//! Compare the native shared core against saved browser/WASM inputs.
use std::io::{self, BufRead};
fn main() {
    let mut session=kratos_lab::Session::default();
    let mut previous=None;
    for line in io::stdin().lock().lines(){
        let line=line.expect("stdin");let value:serde_json::Value=serde_json::from_str(&line).expect("trace input");
        let tick=value["frame"]["tick"].as_u64().expect("tick");
        if previous.is_some_and(|p|tick<=p){session=Default::default();}
        println!("{}",kratos_lab::run(&mut session,&line));previous=Some(tick);
    }
}
