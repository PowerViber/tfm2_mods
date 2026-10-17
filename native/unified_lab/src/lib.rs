//! Equation Lab transport: the game's own unified_core.rs compiled to WASM.
//! Input per tick is a flat i64 frame; output is the core's state and commands as JSON.
#![allow(dead_code)]
#[path = "../../tfm2_custom_ai/src/unified_core.rs"]
pub mod unified_core;
#[path = "../../tfm2_custom_ai/src/unified_eq_data.rs"]
pub mod unified_eq_data;
#[path = "../../tfm2_custom_ai/src/unified_theory_math.rs"]
mod unified_theory_math;
use std::cell::RefCell;
use std::collections::BTreeSet;
use std::fmt::Write;
use unified_core::{Action, Aim, Body, Cmd, Core, Fighter, Input};

#[derive(Default)]
struct Session {
    cores: Vec<Option<Core>>,
    input: Vec<i64>,
    output: Vec<u8>,
}
thread_local! {static SESSION: RefCell<Session> = RefCell::new(Session::default());}

#[no_mangle]
pub extern "C" fn unified_new(seed: u32, owner: u32, rank: u32) -> u32 {
    SESSION.with(|s| {
        let mut s = s.borrow_mut();
        let core = Some(Core::new(seed as u64, owner as usize, rank as usize));
        if let Some(n) = s.cores.iter().position(Option::is_none) {
            s.cores[n] = core;
            n as u32
        } else {
            s.cores.push(core);
            (s.cores.len() - 1) as u32
        }
    })
}
#[no_mangle]
pub extern "C" fn unified_drop(id: u32) {
    SESSION.with(|s| {
        if let Some(c) = s.borrow_mut().cores.get_mut(id as usize) {
            *c = None;
        }
    })
}
#[no_mangle]
pub extern "C" fn unified_input(n: u32) -> *mut i64 {
    SESSION.with(|s| {
        let mut s = s.borrow_mut();
        if n > 20000 {
            return std::ptr::null_mut();
        }
        s.input.resize(n as usize, 0);
        s.input.as_mut_ptr()
    })
}
#[no_mangle]
pub extern "C" fn unified_output_len() -> u32 {
    SESSION.with(|s| s.borrow().output.len() as u32)
}

/// Frame: tick, me, automatic, ult_unlocked, stunned, action (0 none, 1 reserve, 2 cast, 3 ult), skill,
/// aim kind (0 unit, 1 point), aim a, aim b, fighter count, fighters (12 values each: id team x y hp max_hp
/// ad ap champion visible targetable cc_immune), wall cell count, wall cells (x y on a 16000 grid).
pub fn decode(a: &[i64]) -> Option<(usize, usize, bool, bool, bool, Option<Action>, Vec<Fighter>, BTreeSet<(i64, i64)>)> {
    let head = a.get(..11)?;
    let aim = if head[7] == 0 { Aim::Unit(head[8].max(0) as usize) } else { Aim::Point(head[8], head[9]) };
    let skill = head[6].clamp(0, 17) as usize;
    let action = match head[5] {
        1 => Some(Action::Reserve(skill, aim)),
        2 => Some(Action::Cast(skill, aim)),
        3 => Some(Action::Ult(aim)),
        _ => None,
    };
    let n = head[10].clamp(0, 64) as usize;
    let body = a.get(11..11 + n * 12)?;
    let fighters = body
        .chunks(12)
        .map(|u| Fighter { id: u[0] as usize, team: u[1] as usize, x: u[2], y: u[3], hp: u[4], max_hp: u[5].max(1), ad: u[6], ap: u[7], champion: u[8] != 0, visible: u[9] != 0, targetable: u[10] != 0, cc_immune: u[11] != 0 })
        .collect();
    let rest = a.get(11 + n * 12..)?;
    let w = rest.first().copied().unwrap_or(0).clamp(0, 2000) as usize;
    let walls = rest.get(1..1 + w * 2).unwrap_or(&[]).chunks(2).map(|c| (c[0], c[1])).collect();
    Some((head[0].max(0) as usize, head[1].max(0) as usize, head[2] != 0, head[3] != 0, head[4] != 0, action, fighters, walls))
}

fn json_str(s: &str) -> String {
    let mut o = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            c if (c as u32) < 32 => {
                let _ = write!(o, "\\u{:04x}", c as u32);
            }
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

pub fn state(c: &Core, tick: usize, cmds: &[Cmd]) -> String {
    let mut s = String::new();
    let _ = write!(s, "{{\"tick\":{tick},\"rank\":{},\"charges\":{:?},\"cooldown\":{:?},\"reserve\":{:?},\"persona\":{},\"reason\":{},\"ultAt\":{},\"unifyAt\":{},",
        c.rank, c.charges, c.cooldown, c.reserve.iter().map(|r| r.skill).collect::<Vec<_>>(), c.persona, json_str(c.reason), c.ult_at, c.unify_at);
    let st = &c.stats;
    let _ = write!(s, "\"stats\":{{\"casts\":{},\"steps\":{},\"combos\":{},\"signatures\":{},\"unifications\":{},\"insights\":{},\"fizzles\":{},\"ults\":{},\"damage\":{},\"healing\":{},\"misplays\":{},\"efficiency\":{},\"measured\":{},\"longest\":{}}},",
        st.casts, st.steps, st.combos, st.signatures, st.unifications, st.insights, st.fizzles, st.ults, st.damage, st.healing, st.misplays, st.efficiency, st.measured, st.longest);
    s.push_str("\"plan\":[");
    for (i, p) in c.plan.iter().enumerate() {
        let _ = write!(s, "{}[{},{}]", if i > 0 { "," } else { "" }, p.skill, u8::from(p.cast));
    }
    s.push_str("],\"marks\":{");
    for (i, (id, m)) in c.marks.iter().enumerate() {
        let _ = write!(s, "{}\"{id}\":{}", if i > 0 { "," } else { "" }, m.count(tick));
    }
    s.push_str("},\"live\":[");
    let mut first = true;
    for l in &c.live {
        let item = match &l.body {
            Body::Ray { x, y, dx, dy, .. } => format!("{{\"k\":\"ray\",\"s\":{},\"x\":{x},\"y\":{y},\"dx\":{dx},\"dy\":{dy}}}", l.skill),
            Body::Lob { x, y, land, radius, .. } => format!("{{\"k\":\"lob\",\"s\":{},\"x\":{x},\"y\":{y},\"r\":{radius},\"t\":{}}}", l.skill, land.saturating_sub(tick)),
            Body::Planet { x, y, cx, cy, radius, .. } => format!("{{\"k\":\"planet\",\"s\":{},\"x\":{x},\"y\":{y},\"cx\":{cx},\"cy\":{cy},\"r\":{radius}}}", l.skill),
            Body::Zone { x, y, radius, start, until, .. } => format!("{{\"k\":\"zone\",\"s\":{},\"x\":{x},\"y\":{y},\"r\":{radius},\"open\":{},\"left\":{}}}", if l.mods.secondary && l.skill == 6 { 99 } else { l.skill }, u8::from(tick >= *start), until.saturating_sub(tick)),
            Body::Dot { target, n, .. } => format!("{{\"k\":\"dot\",\"s\":{},\"t\":{target},\"n\":{n}}}", l.skill),
            Body::Reaction { until, taken } => format!("{{\"k\":\"reaction\",\"s\":{},\"left\":{},\"taken\":{taken}}}", l.skill, until.saturating_sub(tick)),
            _ => continue,
        };
        let _ = write!(s, "{}{item}", if first { "" } else { "," });
        first = false;
    }
    s.push_str("],\"commands\":[");
    for (i, cmd) in cmds.iter().enumerate() {
        let item = match cmd {
            Cmd::Damage { target, amount, kind } => format!("[\"damage\",{target},{amount},\"{kind:?}\"]"),
            Cmd::Heal { target, amount } => format!("[\"heal\",{target},{amount}]"),
            Cmd::Shield { target, amount, ticks } => format!("[\"shield\",{target},{amount},{ticks}]"),
            Cmd::Root { target, ticks } => format!("[\"root\",{target},{ticks}]"),
            Cmd::Speed { target, pct, ticks } => format!("[\"speed\",{target},{pct},{ticks}]"),
            Cmd::SetPos { target, x, y } => format!("[\"pos\",{target},{x},{y}]"),
            Cmd::Fx { tag, x, y, unit, ticks } => format!("[\"fx\",{},{x},{y},{},{ticks}]", json_str(tag), unit.map_or(-1, |u| u as i64)),
            Cmd::Log(t) => format!("[\"log\",{}]", json_str(t)),
        };
        let _ = write!(s, "{}{item}", if i > 0 { "," } else { "" });
    }
    s.push_str("]}");
    s
}

#[no_mangle]
pub extern "C" fn unified_step(id: u32, n: u32) -> *const u8 {
    SESSION.with(|s| {
        let mut s = s.borrow_mut();
        let frame: Vec<i64> = s.input.get(..n as usize).unwrap_or(&[]).to_vec();
        let text = match (decode(&frame), s.cores.get_mut(id as usize).and_then(|c| c.as_mut())) {
            (Some((tick, me, automatic, ult, stunned, action, fighters, walls)), Some(core)) => {
                let wall = |x: i64, y: i64| walls.contains(&(x.div_euclid(16000), y.div_euclid(16000)));
                let cmds = core.step(&Input { tick, me, fighters: &fighters, automatic, action, ult_unlocked: ult, stunned, wall: &wall });
                state(core, tick, &cmds)
            }
            _ => "{\"error\":\"bad frame or session\"}".into(),
        };
        s.output = text.into_bytes();
        s.output.as_ptr()
    })
}
