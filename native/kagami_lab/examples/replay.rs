//! Replays one Lab frame through the core (debugging a Lab report): `cargo run --example replay -- '<frame json>'`.
fn main() {
    let frame = std::env::args().nth(1).expect("a frame");
    let mut s = kagami_lab::Session::default();
    let input = format!(r#"{{"version":1,"rules_revision":"{}","frame":{frame}}}"#, kagami_lab::REVISION);
    println!("{}", kagami_lab::run(&mut s, &input));
}
