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
  Ok(f) if valid(&f)=>{let effects=core.step(&f);serde_json::json!({"version":1,"rules_revision":rally_core::data::RULES_REVISION,"state":core,"effects":effects}).to_string()},
  _=>"{\"error\":\"Invalid Rally Lab v1 frame; original rules required for other recordings\"}".into()
 }
}
fn valid(f: &Frame) -> bool {
    let point = |p: rally_core::Point| (0..=960000).contains(&p.x) && (0..=960000).contains(&p.y);
    let rect = |r: rally_core::Rect| {
        point(r.min) && point(r.max) && r.max.x > r.min.x && r.max.y > r.min.y
    };
    f.version == 1
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
    let Ok(value) = serde_json::from_slice::<serde_json::Value>(input) else {
        return false;
    };
    let Some(fields) = value.as_object() else {
        return false;
    };
    if fields.keys().any(|k| {
        !matches!(
            k.as_str(),
            "flow" | "charges" | "role" | "transformed" | "form" | "black_ready"
        )
    }) {
        return false;
    }
    let flow = match fields.get("flow") {
        Some(v) => match v.as_i64() {
            Some(n) => Some(n.clamp(0, 100)),
            None => return false,
        },
        None => None,
    };
    let charges = match fields.get("charges") {
        Some(v) => match v.as_i64() {
            Some(n) => Some(n.clamp(0, 3)),
            None => return false,
        },
        None => None,
    };
    let role = match fields.get("role") {
        Some(v) => Some(match v.as_str() {
            Some("spiker") => rally_core::Role::Spiker,
            Some("setter") => rally_core::Role::Setter,
            Some("receiver") => rally_core::Role::Receiver,
            _ => return false,
        }),
        None => None,
    };
    let transformed = match fields.get("transformed") {
        Some(v) => match v.as_bool() {
            Some(b) => Some(b),
            None => return false,
        },
        None => None,
    };
    let form = match fields.get("form") {
        Some(v) => match v.as_str() {
            Some("base") => Some(0),
            Some("black_thunder") => Some(1),
            Some("white_storm") => Some(2),
            _ => return false,
        },
        None => None,
    };
    let black_ready = match fields.get("black_ready") {
        Some(v) => match v.as_bool() {
            Some(b) => Some(b),
            None => return false,
        },
        None => None,
    };
    if (form.is_some() && transformed.is_some())
        || ((form.is_some() || black_ready.is_some()) && core.busy())
    {
        return false;
    }
    if let Some(n) = form {
        core.transformed = n > 0;
        core.white_storm = n == 2;
    }
    if let Some(ready) = black_ready {
        core.black_until = if ready {
            core.last_tick.unwrap_or(0) + rally_core::data::BLACK_READY_TICKS as u64
        } else {
            0
        };
        core.charge_progress = 0;
        core.awakened = ready || core.serve.is_some();
    }
    if let Some(b) = transformed {
        core.transformed = b;
        if !b {
            core.white_storm = false;
        }
    }
    if let Some(n) = flow {
        core.flow = n;
    }
    if let Some(n) = charges {
        core.charges = n;
        core.charge_progress = 0;
    }
    if let Some(role) = role {
        core.role = role;
    }
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
    fn rejected_transport_and_practice_inputs_do_not_partially_mutate_state() {
        let mut c = Core::default();
        let before = serde_json::to_string(&c).unwrap();
        for text in [
            "not json",
            r#"{"version":2}"#,
            r#"{"version":1,"profile":[999]}"#,
            r#"{"flow":100}"#,
        ] {
            assert!(run(&mut c, text).contains("error"));
            assert_eq!(serde_json::to_string(&c).unwrap(), before);
        }
        for text in [
            r#"{"flow":100,"role":"not a role"}"#,
            r#"{"charges":"bad"}"#,
            r#"{"flow":100,"damage":99999}"#,
        ] {
            assert!(!practice(&mut c, text.as_bytes()));
            assert_eq!(serde_json::to_string(&c).unwrap(), before);
        }
    }
    #[test]
    fn practice_forms_and_armed_start_are_explicit_and_do_not_spend_cooldowns() {
        let mut c = Core::default();
        for (form, black, white) in [
            ("black_thunder", true, false),
            ("white_storm", true, true),
            ("base", false, false),
        ] {
            let text = serde_json::json!({"form":form,"flow":100,"charges":3});
            assert!(practice(&mut c, text.to_string().as_bytes()));
            assert_eq!((c.transformed, c.white_storm), (black, white));
            assert_eq!(c.cds, [0; 5]);
        }
        assert!(practice(&mut c, br#"{"black_ready":true}"#));
        assert_eq!(c.black_until, 480);
        assert!(c.awakened);
        assert!(practice(&mut c, br#"{"black_ready":false}"#));
        assert_eq!(c.black_until, 0);
        assert!(!c.awakened);
        let before = serde_json::to_string(&c).unwrap();
        for text in [
            br#"{"flow":0,"form":"invalid"}"#.as_slice(),
            br#"{"charges":0,"black_ready":1}"#.as_slice(),
            br#"{"form":"base","transformed":true}"#.as_slice(),
        ] {
            assert!(!practice(&mut c, text));
            assert_eq!(serde_json::to_string(&c).unwrap(), before);
        }
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
