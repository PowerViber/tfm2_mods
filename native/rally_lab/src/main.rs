use std::io::{self, BufRead};
fn main() {
    let mut core = rally_lab::rally_core::Core::default();
    let args=std::env::args().collect::<Vec<_>>();
    if args.get(1).is_some_and(|s|s=="--practice-fixture") && !args.get(2).is_some_and(|s|rally_lab::practice(&mut core,s.as_bytes())) {
        eprintln!("Invalid explicit practice fixture");std::process::exit(2);
    }
    for line in io::stdin().lock().lines() {
        match line {
            Ok(s) => println!("{}", rally_lab::run(&mut core, &s)),
            Err(_) => break,
        }
    }
}
