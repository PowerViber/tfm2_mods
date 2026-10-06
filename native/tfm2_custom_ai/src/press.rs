//! Round 88: which data-slot presses a champion's native brain would use, per tick, for the input AI (steve.rs
//! WallAi): a press the brain wouldn't use becomes a basic attack on the same target instead of a wasted cast (Aegis
//! Zero mid-ult, Isliid between plans). Same scheme as Levi's (levi.rs note_press / press_flags): written per tick and
//! read for the tick before, keyed by (seed, player), so the precomputed and the live simulation of a match (same seed,
//! one possibly far ahead) read the same values.
use std::collections::{BTreeMap, HashMap};
use std::sync::Mutex;

pub const S1: u8 = 1;
pub const S2: u8 = 2;
/// Round 91: the champion must not attack now (Aegis Zero rising / flying): the input AI never turns a press into a
/// basic attack while this is set (the press passes through and the native CC blocks it).
pub const HOLD: u8 = 4;

type Presses = HashMap<(u64, usize), BTreeMap<usize, u8>>;
static PRESSES: Mutex<Option<Presses>> = Mutex::new(None);

/// How far back flags are kept (the parallel sims can be far apart; a whole match is ~100k ticks of small entries).
const KEEP: usize = 200_000;

pub fn note(seed: u64, player: usize, tick: usize, flags: u8) {
    if let Ok(mut g) = PRESSES.lock() {
        let m = g.get_or_insert_with(HashMap::new);
        if m.len() > 64 { m.clear(); }
        let e = m.entry((seed, player)).or_default();
        e.insert(tick, flags);
        if e.len() > KEEP {
            let cut = tick.saturating_sub(KEEP);
            *e = e.split_off(&cut);
        }
    }
}

/// The flags set on exactly `tick` (None when the brain didn't run then: leave the press alone).
pub fn get(seed: u64, player: usize, tick: usize) -> Option<u8> {
    PRESSES.lock().ok()?.as_ref()?.get(&(seed, player))?.get(&tick).copied()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn flags_are_read_by_exact_tick() {
        note(0xFEED, 2, 10, S1);
        note(0xFEED, 2, 11, S1 | S2);
        assert_eq!(get(0xFEED, 2, 10), Some(S1));
        assert_eq!(get(0xFEED, 2, 11), Some(S1 | S2));
        assert_eq!(get(0xFEED, 2, 12), None);
        assert_eq!(get(0xFEED, 3, 10), None);
    }
}
