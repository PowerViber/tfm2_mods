//! Complete JSON-lines native/WASM trace runner; no alternate combat implementation.
use std::io::{self,BufRead};
fn main(){let mut session=wright_lab::Session::default();for line in io::stdin().lock().lines(){let line=line.expect("fixture line");if !line.trim().is_empty(){println!("{}",wright_lab::run(&mut session,&line));}}}
