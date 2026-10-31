// The hook plumbing below is only reachable on Windows; elsewhere only its tests use it.
#![cfg_attr(not(windows), allow(dead_code))]
//! Contender realms, Stage 1 of the native bridge: OBSERVE ONLY.
//!
//! PR #59 found, statically, four engine functions that a real realm would have to gate (see
//! docs/contender-realms/README.md). None was ever seen running in a match. This module hooks them so that a
//! disposable match can show who calls them and with what: it records and passes through, and never changes,
//! blocks or redirects anything.
//!
//! Off unless BOTH hold: `mods/tfm2_custom_ai/realm_trace.flag` exists, and the running game is the analysed build
//! (`identity.rs`). Otherwise `status()` is "none" and no game memory is read or written. One failed hook removes
//! every hook already placed and leaves the bridge at "none".
//!
//! Layout per hook (`detour.rs`): the function's entry is replaced by a 14-byte absolute jump to a stub; the stub
//! saves every volatile register (and xmm0–5), calls `recorder`, restores them and jumps to a trampoline holding the
//! relocated original instructions, which jumps back into the function. The return address on the stack stays the
//! caller's, so the function's own unwind data still describes its frame. Stubs and trampolines are never freed, so
//! a thread still inside one when the hooks come off is safe.
pub mod detour;
pub mod identity;
pub mod trace;

use std::sync::atomic::{AtomicU64, AtomicU8, Ordering::*};
use std::sync::Mutex;

/// A hooked function of the analysed build: its RVA, the end of its unwind range (so every branch in it can be
/// checked) and the 32 entry bytes `realm-hooks.json` recorded.
pub struct Target {
    pub name: &'static str,
    pub rva: u64,
    pub end: u64,
    pub entry: &'static str,
}

pub const TARGETS: [Target; trace::HOOKS] = [
    Target { name: "visibility", rva: 0x16aa0c0, end: 0x16aa141, entry: "4883ec284c3b8150070000731d488b8148070000428b04804531c083f8ff410f" },
    Target { name: "damage", rva: 0x18c0480, end: 0x18c178b, entry: "5541574156415541545657534881ec68010000488dac248000000048c785e000" },
    Target { name: "queue", rva: 0x16ad0d0, end: 0x16ad3aa, entry: "5541574156415541545657534881ec98000000488dac248000000048c74510fe" },
    Target { name: "control", rva: 0x33d0a10, end: 0x33d0dbe, entry: "5541574156415541545657534881ecd8000000488dac248000000048c74550fe" },
];

pub fn hex(s: &str) -> Vec<u8> {
    (0..s.len() / 2).filter_map(|i| u8::from_str_radix(&s[2 * i..2 * i + 2], 16).ok()).collect()
}

const NONE: u8 = 0;
const OBSERVE: u8 = 1;
static STATUS: AtomicU8 = AtomicU8::new(NONE);
static TRIED: std::sync::Once = std::sync::Once::new();
static BASE: AtomicU64 = AtomicU64::new(0);

/// "observe" while the observe-only hooks are in place, else "none". Never "native": nothing here gates anything.
pub fn status() -> &'static str {
    if STATUS.load(Acquire) == OBSERVE { "observe" } else { "none" }
}

fn enabled() -> bool {
    crate::mod_dir().is_some_and(|d| d.join("realm_trace.flag").exists())
}

/// The recorder every stub calls (Microsoft x64 convention on every platform, matching the stub).
pub extern "win64" fn recorder(hook: u64, regs: *const u64) {
    // SAFETY: the stub passes a pointer to its 8 saved words on its own stack.
    let regs = unsafe { &*(regs as *const [u64; 8]) };
    let mut frames = [0u64; trace::FRAMES];
    let n = os::backtrace(&mut frames);
    trace::record(hook, regs, &frames[..n]);
}

static FILE: Mutex<Option<std::fs::File>> = Mutex::new(None);
const FILE_CAP: u64 = 32 * 1024 * 1024;

/// From the match hook, every tick: the first tick with the flag present places the hooks; afterwards recorded
/// calls are written out every 30 ticks.
pub fn tick(sim: &mut mod_api_stable::StableSim<'_>) {
    trace::TICK.store(sim.tick() as u64, Relaxed);
    TRIED.call_once(|| {
        if !enabled() {
            return;
        }
        let result = os::install();
        let line = match &result {
            Ok(base) => {
                BASE.store(*base, Release);
                STATUS.store(OBSERVE, Release);
                format!("native bridge: observe-only hooks placed on {} functions (visibility, damage, queue, control); nothing is gated", TARGETS.len())
            }
            Err(e) => format!("native bridge: none ({e})"),
        };
        crate::mod_log(sim, "contender_log.txt", "native_bridge", &line);
        write(&[serde_json::json!({"kind": "bridge", "status": status(), "detail": line, "engineMutations": 0}).to_string()]);
    });
    if STATUS.load(Acquire) == OBSERVE && sim.tick() % 30 == 0 {
        flush(sim.tick() % 600 == 0);
    }
}

fn flush(summary: bool) {
    let (entries, lost) = trace::drain();
    let base = BASE.load(Acquire);
    let size = identity::SIZE_OF_IMAGE as u64;
    let hexs = |v: u64| format!("{v:#x}");
    let mut lines: Vec<String> = entries
        .iter()
        .map(|e| {
            serde_json::json!({
                "kind": "call",
                "hook": TARGETS.get(e.hook as usize).map_or("?", |t| t.name),
                "tick": e.tick,
                "rcx": hexs(e.args[0]), "rdx": hexs(e.args[1]), "r8": hexs(e.args[2]), "r9": hexs(e.args[3]),
                "caller": trace::label(base, size, e.ret),
                "stack": e.frames.iter().map(|f| trace::label(base, size, *f)).collect::<Vec<_>>(),
            })
            .to_string()
        })
        .collect();
    if summary || lost > 0 {
        let calls: Vec<_> = TARGETS.iter().zip(&trace::CALLS).map(|(t, c)| serde_json::json!({"hook": t.name, "calls": c.load(Relaxed)})).collect();
        lines.push(serde_json::json!({"kind": "summary", "tick": trace::TICK.load(Relaxed), "calls": calls, "lost": lost}).to_string());
    }
    write(&lines);
}

fn write(lines: &[String]) {
    use std::io::Write;
    if lines.is_empty() {
        return;
    }
    let Ok(mut g) = FILE.try_lock() else { return };
    if g.is_none() {
        let Some(dir) = crate::mod_dir() else { return };
        *g = std::fs::OpenOptions::new().create(true).append(true).open(dir.join("realm_trace.jsonl")).ok();
    }
    let Some(f) = g.as_mut() else { return };
    if f.metadata().map_or(true, |m| m.len() >= FILE_CAP) {
        return;
    }
    for l in lines {
        let _ = writeln!(f, "{l}");
    }
}

#[cfg(windows)]
#[path = "windows.rs"]
mod os;

#[cfg(not(windows))]
mod os {
    pub fn install() -> Result<u64, String> {
        Err("the native bridge only exists on Windows".into())
    }
    pub fn backtrace(_frames: &mut [u64]) -> usize {
        0
    }
}
