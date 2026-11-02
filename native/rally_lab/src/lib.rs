#[path = "../../tfm2_custom_ai/src/rally_core.rs"]
pub mod rally_core;
use rally_core::{Core, Frame};
use std::cell::RefCell;
#[cfg(test)]
#[path = "../../tfm2_custom_ai/src/rally_tests.rs"]
mod tests;
pub fn run(core: &mut Core, text: &str) -> String {
    let parsed = serde_json::from_str::<Frame>(text);
    match parsed {
  Ok(f) if valid(&f)=>{let effects=core.step(&f);serde_json::json!({"version":2,"rules_revision":rally_core::data::RULES_REVISION,"state":core,"effects":effects}).to_string()},
  _=>"{\"error\":\"Invalid Rally Lab v2 frame; original rules required for other recordings\"}".into()
 }
}
fn valid(f: &Frame) -> bool {
    let point = |p: rally_core::Point| (0..=960000).contains(&p.x) && (0..=960000).contains(&p.y);
    let rect = |r: rally_core::Rect| {
        point(r.min) && point(r.max) && r.max.x > r.min.x && r.max.y > r.min.y
    };
    f.version == 2
        && f.tick < 1_000_000_000
        && f.units.len() <= 512
        && f.blocks.len() <= 2048
        && f.smoke.len() <= 2048
        && f.smoke_disks.len() <= 128
        && f.smoke_disks
            .iter()
            .all(|d| point(d.pos) && (1..=960000).contains(&d.radius))
        && rect(f.bounds)
        && f.profile.iter().all(|v| (0..=100).contains(v))
        && f.me().is_some()
        && f.units.iter().all(|u| {
            point(u.pos)
                && (1..=100000).contains(&u.radius)
                && (1..=100000000).contains(&u.max_hp)
                && (0..=100000000).contains(&u.hp)
                && (0..=1000000).contains(&u.attack)
                && u.velocity.x.abs() <= 960000
                && u.velocity.y.abs() <= 960000
        })
        && f.blocks.iter().chain(&f.smoke).all(|r| rect(*r))
        && f.units
            .iter()
            .map(|u| u.id)
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            == f.units.len()
        && (1..=10000).contains(&f.move_step)
        && f.receipts.len() <= 128
        && f.basics.len() <= 128
        && f.hits.len() <= 128
        && f.support_receipts.len() <= 128
        && f.receipts
            .iter()
            .all(|r| (0..=100000000).contains(&r.delivered) && (0..=95).contains(&r.reduction))
        && f.action.as_ref().is_none_or(|a| point(a.aim))
        && f.aim.is_none_or(point)
        && [1, 3, 5].contains(&f.burst_bars)
        && f.receipts.iter().all(|r| {
            r.incoming_force
                .is_none_or(|v| (0..=100000000).contains(&v))
        })
        && f.steer.x.abs() <= 1
        && f.steer.y.abs() <= 1
}
#[derive(Default)]
struct Session {
    cores: Vec<Option<Core>>,
    input: Vec<u8>,
    output: Vec<u8>,
}
thread_local! {static SESSION:RefCell<Session>=RefCell::new(Session::default());}
#[no_mangle]
pub extern "C" fn rally_new() -> u32 {
    SESSION.with(|s| {
        let mut s = s.borrow_mut();
        if let Some(i) = s.cores.iter().position(Option::is_none) {
            s.cores[i] = Some(Core::default());
            i as u32
        } else {
            let i = s.cores.len();
            s.cores.push(Some(Core::default()));
            i as u32
        }
    })
}
#[no_mangle]
pub extern "C" fn rally_drop(id: u32) {
    SESSION.with(|s| {
        if let Some(c) = s.borrow_mut().cores.get_mut(id as usize) {
            *c = None
        }
    })
}
#[no_mangle]
pub extern "C" fn rally_input(n: u32) -> *mut u8 {
    SESSION.with(|s| {
        let mut s = s.borrow_mut();
        if n > 2_000_000 {
            return std::ptr::null_mut();
        }
        s.input.resize(n as usize, 0);
        s.input.as_mut_ptr()
    })
}
#[no_mangle]
pub extern "C" fn rally_output_len() -> u32 {
    SESSION.with(|s| s.borrow().output.len() as u32)
}
/// Explicit practice-only setup, never part of Frame and never compiled into the native adapter.
pub fn practice(core: &mut Core, input: &[u8]) -> bool {
    let Ok(v) = serde_json::from_slice::<serde_json::Value>(input) else {
        return false;
    };
    let Some(o) = v.as_object() else {
        return false;
    };
    if core.busy()
        || o.keys()
            .any(|k| !["pace", "role", "form", "bank_percent"].contains(&k.as_str()))
    {
        return false;
    }
    let mut candidate = core.clone();
    for (k, v) in o {
        match k.as_str() {
            "pace" => {
                let Some(n) = v.as_i64().filter(|n| (0..=100).contains(n)) else {
                    return false;
                };
                candidate.pace = n;
            }
            "bank_percent" => {
                let Some(n) = v.as_i64().filter(|n| (0..=300).contains(n)) else {
                    return false;
                };
                candidate.bank_base = 194;
                candidate.caught = 194 * n / 100;
                candidate.caught_until = candidate.last_tick.unwrap_or(0) + 480;
            }
            "role" => {
                candidate.role = match v.as_str() {
                    Some("spiker") => rally_core::Role::Spiker,
                    Some("setter") => rally_core::Role::Setter,
                    Some("receiver") => rally_core::Role::Receiver,
                    _ => return false,
                };
            }
            "form" => {
                let Some(n) = v.as_str() else {
                    return false;
                };
                if !["base", "yellow_thunder", "black_thunder", "white_storm"].contains(&n) {
                    return false;
                }
                candidate.thunder_earned = n != "base";
                candidate.transformed = n == "black_thunder" || n == "white_storm";
                candidate.white_storm = n == "white_storm";
            }
            _ => return false,
        }
    }
    *core = candidate;
    true
}

#[no_mangle]
pub extern "C" fn rally_fixture(id: u32, n: u32) -> bool {
    SESSION.with(|s| {
        let mut s = s.borrow_mut();
        let input = s.input[..(n as usize).min(s.input.len())].to_vec();
        let Some(c) = s.cores.get_mut(id as usize).and_then(Option::as_mut) else {
            return false;
        };
        practice(c, &input)
    })
}

#[cfg(test)]
mod protocol_tests {
    use super::*;
    #[test]
    fn rejected_practice_is_atomic() {
        let mut c = Core::default();
        for text in [
            r#"{"pace":101}"#,
            r#"{"pace":50,"flow":100}"#,
            r#"{"bank_percent":1001}"#,
        ] {
            let before = serde_json::to_value(&c).unwrap();
            assert!(!practice(&mut c, text.as_bytes()));
            assert_eq!(serde_json::to_value(&c).unwrap(), before);
        }
        assert!(practice(
            &mut c,
            br#"{"pace":100,"form":"white_storm","bank_percent":300}"#
        ));
        assert!(c.white_storm);
        assert_eq!(c.pace, 100);
        assert_eq!(c.bank_percent(), 300);
    }
    #[test]
    fn old_transport_never_steps_new_physics() {
        let mut c = Core::default();
        let f = Frame {
            version: 1,
            ..Frame::default()
        };
        assert!(run(&mut c, &serde_json::to_string(&f).unwrap()).contains("error"));
        assert!(c.last_tick.is_none());
    }
}

#[no_mangle]
pub extern "C" fn rally_step(id: u32, n: u32) -> *const u8 {
    SESSION.with(|s| {
        let mut s = s.borrow_mut();
        let input =
            String::from_utf8_lossy(&s.input[..(n as usize).min(s.input.len())]).into_owned();
        let out = match s.cores.get_mut(id as usize).and_then(Option::as_mut) {
            Some(c) => run(c, &input),
            None => "{\"error\":\"Invalid session\"}".into(),
        };
        s.output = out.into_bytes();
        s.output.as_ptr()
    })
}
