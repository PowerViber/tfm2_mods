#![allow(dead_code)]
#[path = "../../tfm2_custom_ai/src/raze_core.rs"]
pub mod raze_core;
#[path = "../../tfm2_custom_ai/src/raze_data.rs"]
pub mod raze_data;
#[cfg(test)]
#[path = "../../tfm2_custom_ai/src/raze_tests.rs"]
mod tests;
#[path = "../../tfm2_custom_ai/src/valorant_guns.rs"]
pub mod valorant_guns;
use raze_core::{Core, Frame};
use std::cell::RefCell;
#[derive(Default)]
struct Session {
    cores: Vec<Option<Core>>,
    input: Vec<u8>,
    output: Vec<u8>,
}
thread_local! {static SESSION:RefCell<Session>=RefCell::new(Session::default());}
pub fn run(core: &mut Core, text: &str) -> String {
    let frame = serde_json::from_str::<Frame>(text);
    match frame {
        Ok(f) if valid(&f) => {
            let commands = core.step(&f);
            serde_json::json!({"version":3,"rules_revision":raze_data::RULES_REVISION,"state":core,"commands":commands}).to_string()
        }
        _ => "{\"error\":\"Invalid Raze v3 frame; older rules require their original simulator\"}"
            .into(),
    }
}
fn valid(f: &Frame) -> bool {
    use raze_core::{Action, Point};
    let point = |p: Point| p.x.unsigned_abs() <= 2000000 && p.y.unsigned_abs() <= 2000000;
    let action = match &f.action {
        Some(
            Action::Grenade { aim }
            | Action::Pack { aim }
            | Action::Bot { aim }
            | Action::FireRocket { aim },
        ) => point(*aim),
        Some(Action::Steer { input }) => {
            input.x.unsigned_abs() <= 1000 && input.y.unsigned_abs() <= 1000
        }
        Some(Action::Detonate { id }) => *id < 1000000000000,
        _ => true,
    };
    f.version == 3
        && f.tick < 1000000000000
        && f.owner <= 1000000
        && f.units.len() <= 512
        && f.blocks.len() <= 4096
        && f.surfaces.len() <= 256
        && f.surfaces.iter().all(|s| {
            s.id > 0
                && s.id < 1000000000000
                && point(s.bounds.min)
                && point(s.bounds.max)
                && s.bounds.min.x <= s.bounds.max.x
                && s.bounds.min.y <= s.bounds.max.y
        })
        && f.surfaces
            .iter()
            .map(|s| s.id)
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            == f.surfaces.len()
        && f.defenders.len() <= 64
        && f.defenders
            .iter()
            .all(|d| d.id <= 1000000 && d.profile.values.iter().all(|v| (0..=100).contains(v)))
        && f.hazards.len() <= 256
        && f.utility.len() <= 8
        && f.credits.len() <= 256
        && f.me().is_some()
        && action
        && f.units.iter().all(|u| {
            u.id <= 1000000
                && u.team <= 255
                && point(u.pos)
                && u.velocity.x.unsigned_abs() <= 1000000
                && u.velocity.y.unsigned_abs() <= 1000000
                && (0..=10000000).contains(&u.hp)
                && (1..=10000000).contains(&u.max_hp)
                && (0..=1000000).contains(&u.ap)
                && (0..=1000000).contains(&u.attack)
                && (0..=1000000).contains(&u.mr)
                && (0..=1000000).contains(&u.defence)
                && (0..=1000000).contains(&u.attack_range)
                && (0..=10000000).contains(&u.shield)
                && u.kind <= 5
        })
        && f.units
            .iter()
            .map(|u| u.id)
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            == f.units.len()
        && f.blocks
            .iter()
            .all(|b| point(b.min) && point(b.max) && b.min.x <= b.max.x && b.min.y <= b.max.y)
        && f.hazards.iter().all(|h| {
            point(h.pos)
                && (0..=1000000).contains(&h.radius)
                && (0..=1000).contains(&h.severity)
                && h.until < 1000000000000
        })
        && f.utility
            .iter()
            .all(|u| u.id < 1000000000000 && (-1..=10000000).contains(&u.hp))
        && f.credits
            .iter()
            .all(|c| c.kind <= 3 && c.id < 1000000000000)
        && f.assignment.map_or(true, point)
        && f.intent.map_or(true, point)
        && f.steering.map_or(true, |p| {
            p.x.unsigned_abs() <= 1000 && p.y.unsigned_abs() <= 1000
        })
        && (0..=10000000).contains(&f.shield)
        && (1..=30).contains(&f.level)
        && f.mastery <= 7
        && (0..=1000000).contains(&f.missed_wave)
}
#[no_mangle]
pub extern "C" fn raze_new() -> u32 {
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
pub extern "C" fn raze_drop(id: u32) {
    SESSION.with(|s| {
        if let Some(c) = s.borrow_mut().cores.get_mut(id as usize) {
            *c = None
        }
    })
}
#[no_mangle]
pub extern "C" fn raze_input(n: u32) -> *mut u8 {
    SESSION.with(|s| {
        let mut s = s.borrow_mut();
        if n > 2000000 {
            return std::ptr::null_mut();
        }
        s.input.resize(n as usize, 0);
        s.input.as_mut_ptr()
    })
}
#[no_mangle]
pub extern "C" fn raze_output_len() -> u32 {
    SESSION.with(|s| s.borrow().output.len() as u32)
}
#[no_mangle]
pub extern "C" fn raze_step(id: u32, n: u32) -> *const u8 {
    SESSION.with(|s| {
        let mut s = s.borrow_mut();
        let input =
            String::from_utf8_lossy(s.input.get(..n as usize).unwrap_or_default()).into_owned();
        let output = match s.cores.get_mut(id as usize).and_then(Option::as_mut) {
            Some(c) => run(c, &input),
            _ => "{\"error\":\"Invalid Raze session\"}".into(),
        };
        s.output = output.into_bytes();
        s.output.as_ptr()
    })
}
#[cfg(test)]
mod transport_tests {
    use super::*;
    #[test]
    fn malformed_frames_reject_without_mutating_a_session() {
        let mut c = Core::default();
        let initial = serde_json::to_string(&c).unwrap();
        for bad in [
            "{}",
            "[]",
            "{\"version\":2}",
            "{\"units\":[{\"id\":1,\"pos\":{\"x\":-9223372036854775808,\"y\":0},\"max_hp\":1}]}",
            "{\"units\":[{\"id\":1,\"max_hp\":1},{\"id\":1,\"max_hp\":1}]}",
            "{\"units\":[{\"id\":1,\"max_hp\":1,\"shield\":9223372036854775807}]}",
            "{\"units\":[{\"id\":1,\"max_hp\":1,\"shield\":-1}]}",
        ] {
            assert!(run(&mut c, bad).contains("error"));
            assert_eq!(initial, serde_json::to_string(&c).unwrap());
        }
    }
}
