#[path = "../../tfm2_custom_ai/src/contender_core.rs"]
pub mod contender_core;
use contender_core::{Core, Frame};
use std::cell::RefCell;
#[derive(Default)]
struct Session {
    cores: Vec<Option<Core>>,
    input: Vec<u8>,
    output: Vec<u8>,
}
thread_local! {static SESSION:RefCell<Session>=RefCell::new(Session::default());}
pub fn run(core: &mut Core, text: &str) -> String {
    match serde_json::from_str::<Frame>(text) {
        Ok(f) if valid(&f) => {
            let effects = core.step(&f);
            serde_json::json!({"version":1,"rules_revision":"contender-1","state":core,"effects":effects}).to_string()
        }
        _ => "{\"error\":\"Invalid Boxing Lab v1 frame\"}".into(),
    }
}
fn valid(f: &Frame) -> bool {
    let point = |p: contender_core::Point| {
        (-30_000_000..=30_000_000).contains(&p.x) && (-30_000_000..=30_000_000).contains(&p.y)
    };
    let rect = |r: contender_core::Rect| {
        point(r.min) && point(r.max) && r.max.x - r.min.x >= 1 && r.max.y - r.min.y >= 1
    };
    // Validate arithmetic inputs before contains/clamp can add radii to coordinates.
    if !rect(f.bounds)
        || f.units
            .iter()
            .any(|u| u.id > 4096 || !(1..=100_000).contains(&u.radius))
    {
        return false;
    }
    f.version == 1
        && f.units.len() <= 128
        && f.blocks.len() <= 2048
        && f.profile.iter().all(|x| (0..=100).contains(x))
        && f.me().is_some_and(|u| f.bounds.contains(u.pos, u.radius))
        && rect(f.bounds)
        && f.units.iter().all(|u| {
            f.bounds.max.x - f.bounds.min.x >= 2 * u.radius
                && f.bounds.max.y - f.bounds.min.y >= 2 * u.radius
        })
        && f.blocks.iter().all(|r| rect(*r))
        && f.action.as_ref().is_none_or(|a| point(a.aim))
        && f.receipts.len() <= 128
        && f.basics.len() <= 128
        && f.units
            .iter()
            .map(|u| u.id)
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            == f.units.len()
        && f.tick < 1_000_000_000
        && f.units.iter().all(|u| {
            point(u.pos)
                && point(u.velocity)
                && u.max_hp > 0
                && u.max_hp <= 100_000_000
                && (0..=100_000_000).contains(&u.hp)
                && (0..=1_000_000).contains(&u.attack)
                && (1..=100_000).contains(&u.radius)
        })
}
#[no_mangle]
pub extern "C" fn contender_new() -> u32 {
    SESSION.with(|s| {
        let mut s = s.borrow_mut();
        if let Some(n) = s.cores.iter().position(Option::is_none) {
            s.cores[n] = Some(Core::default());
            n as u32
        } else {
            let n = s.cores.len();
            s.cores.push(Some(Core::default()));
            n as u32
        }
    })
}
#[no_mangle]
pub extern "C" fn contender_drop(id: u32) {
    SESSION.with(|s| {
        if let Some(c) = s.borrow_mut().cores.get_mut(id as usize) {
            *c = None
        }
    })
}
#[no_mangle]
pub extern "C" fn contender_input(n: u32) -> *mut u8 {
    SESSION.with(|s| {
        if n > 2_000_000 {
            return std::ptr::null_mut();
        }
        let mut s = s.borrow_mut();
        s.input.resize(n as usize, 0);
        s.input.as_mut_ptr()
    })
}
#[no_mangle]
pub extern "C" fn contender_output_len() -> u32 {
    SESSION.with(|s| s.borrow().output.len() as u32)
}
#[no_mangle]
pub extern "C" fn contender_step(id: u32, n: u32) -> *const u8 {
    SESSION.with(|s| {
        let mut s = s.borrow_mut();
        let input =
            String::from_utf8_lossy(&s.input[..(n as usize).min(s.input.len())]).into_owned();
        let output = match s.cores.get_mut(id as usize).and_then(Option::as_mut) {
            Some(c) => run(c, &input),
            None => "{\"error\":\"Invalid session\"}".into(),
        };
        s.output = output.into_bytes();
        s.output.as_ptr()
    })
}
#[cfg(test)]
#[path = "../../tfm2_custom_ai/src/contender_tests.rs"]
mod tests;

#[cfg(test)]
mod protocol_tests {
    #[test]
    fn malformed_extremes_are_rejected_without_mutating_core() {
        let mut c = super::Core::default();
        let before = serde_json::to_string(&c).unwrap();
        for text in [
            "{}",
            "null",
            r#"{"version":1,"tick":0,"owner":1,"units":[],"blocks":[],"bounds":{"min":{"x":9223372036854775807,"y":0},"max":{"x":0,"y":0}},"profile":[50,50,50,50,50,50,50,50,50,50,50,50,50],"manual":true,"action":null,"skill1":true,"skill2":true,"ultimate":true,"interrupted":false,"rooted":false}"#,
        ] {
            assert!(super::run(&mut c, text).contains("error"));
            assert_eq!(serde_json::to_string(&c).unwrap(), before);
        }
    }
}
