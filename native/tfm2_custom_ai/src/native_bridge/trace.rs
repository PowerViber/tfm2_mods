//! What an observe-only hook records, and how it reaches `realm_trace.jsonl` without risk to the game thread.
//!
//! The recorder runs inside the engine's own call, on whatever thread called it, possibly re-entrantly. So it takes
//! no lock, allocates nothing and opens no file: it claims a slot in a fixed ring with one atomic add, fills it with
//! raw register values and return addresses, and publishes it with a sequence number. The match hook drains the ring
//! to the file (`flush`), under a `try_lock` so a busy writer is simply skipped. Arguments are raw values only: they
//! are never dereferenced.
use std::sync::atomic::{AtomicU64, Ordering::*};

pub const FRAMES: usize = 16;
const CAP: usize = 4096;
/// Records kept per hook per process; calls beyond it are only counted.
pub const PER_HOOK: u64 = 3000;
pub const HOOKS: usize = 4;

struct Slot {
    /// 0 while empty or being written; otherwise the claim index + 1.
    seq: AtomicU64,
    hook: AtomicU64,
    tick: AtomicU64,
    args: [AtomicU64; 4],
    ret: AtomicU64,
    frames: [AtomicU64; FRAMES],
}

#[allow(clippy::declare_interior_mutable_const)]
const EMPTY: Slot = Slot {
    seq: AtomicU64::new(0),
    hook: AtomicU64::new(0),
    tick: AtomicU64::new(0),
    args: [const { AtomicU64::new(0) }; 4],
    ret: AtomicU64::new(0),
    frames: [const { AtomicU64::new(0) }; FRAMES],
};
static RING: [Slot; CAP] = [EMPTY; CAP];
static NEXT: AtomicU64 = AtomicU64::new(0);
static READ: AtomicU64 = AtomicU64::new(0);
pub static CALLS: [AtomicU64; HOOKS] = [const { AtomicU64::new(0) }; HOOKS];
static KEPT: [AtomicU64; HOOKS] = [const { AtomicU64::new(0) }; HOOKS];
/// The tick the match hook last saw (several simulations may run: it's a label, not a key).
pub static TICK: AtomicU64 = AtomicU64::new(0);

/// The stub's saved-register block: [r11, r10, r9, r8, rdx, rcx, rax, return address].
pub fn record(hook: u64, regs: &[u64; 8], frames: &[u64]) {
    let h = hook as usize;
    if h >= HOOKS {
        return;
    }
    CALLS[h].fetch_add(1, Relaxed);
    if KEPT[h].fetch_add(1, Relaxed) >= PER_HOOK {
        return;
    }
    let n = NEXT.fetch_add(1, AcqRel);
    let s = &RING[n as usize % CAP];
    s.seq.store(0, Release);
    s.hook.store(hook, Relaxed);
    s.tick.store(TICK.load(Relaxed), Relaxed);
    for (a, v) in s.args.iter().zip([regs[5], regs[4], regs[3], regs[2]]) {
        a.store(v, Relaxed);
    }
    s.ret.store(regs[7], Relaxed);
    for (k, f) in s.frames.iter().enumerate() {
        f.store(frames.get(k).copied().unwrap_or(0), Relaxed);
    }
    s.seq.store(n + 1, Release);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub hook: u64,
    pub tick: u64,
    pub args: [u64; 4],
    pub ret: u64,
    pub frames: Vec<u64>,
}

/// Everything published since the last drain (entries overwritten by a full ring are skipped, and counted).
pub fn drain() -> (Vec<Entry>, u64) {
    let end = NEXT.load(Acquire);
    let mut from = READ.load(Relaxed);
    let mut lost = 0;
    if end > from + CAP as u64 {
        lost = end - from - CAP as u64;
        from = end - CAP as u64;
    }
    let mut out = Vec::new();
    for n in from..end {
        let s = &RING[n as usize % CAP];
        if s.seq.load(Acquire) != n + 1 {
            // still being written (picked up next time) or already overwritten
            if s.seq.load(Acquire) == 0 {
                READ.store(n, Relaxed);
                return (out, lost);
            }
            lost += 1;
            continue;
        }
        let e = Entry {
            hook: s.hook.load(Relaxed),
            tick: s.tick.load(Relaxed),
            args: [0, 1, 2, 3].map(|k| s.args[k].load(Relaxed)),
            ret: s.ret.load(Relaxed),
            frames: s.frames.iter().map(|f| f.load(Relaxed)).take_while(|f| *f != 0).collect(),
        };
        if s.seq.load(Acquire) == n + 1 {
            out.push(e);
        } else {
            lost += 1;
        }
    }
    READ.store(end, Relaxed);
    (out, lost)
}

/// An address as an RVA of the game image when it falls inside it, else as an absolute label.
pub fn label(base: u64, size: u64, a: u64) -> String {
    if a >= base && a < base + size {
        format!("rva:{:#x}", a - base)
    } else {
        format!("abs:{a:#x}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn records_drain_in_order_and_stop_at_the_cap() {
        let _ = drain();
        let regs = [11, 10, 9, 8, 2, 1, 0, 0x1400_1234];
        record(3, &regs, &[0x1400_0010, 0x1400_0020]);
        record(9, &regs, &[]); // unknown hook: ignored
        let (got, lost) = drain();
        assert_eq!(lost, 0);
        let mine: Vec<_> = got.iter().filter(|e| e.hook == 3).collect();
        assert_eq!(mine.len(), 1);
        assert_eq!(mine[0].args, [1, 2, 8, 9], "rcx, rdx, r8, r9");
        assert_eq!(mine[0].ret, 0x1400_1234);
        assert_eq!(mine[0].frames, vec![0x1400_0010, 0x1400_0020]);
        assert_eq!(label(0x1400_0000, 0x10_0000, 0x1400_1234), "rva:0x1234");
        assert_eq!(label(0x1400_0000, 0x10_0000, 0x10), "abs:0x10");
        for _ in 0..PER_HOOK + 10 {
            record(2, &regs, &[]);
        }
        let (got, _) = drain();
        assert!(got.iter().filter(|e| e.hook == 2).count() as u64 <= PER_HOOK);
        assert!(CALLS[2].load(Relaxed) >= PER_HOOK + 10, "calls beyond the cap are still counted");
    }
}
