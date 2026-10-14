//! An effect budget. Every `play_view_effect` is a new animated sprite for the game; bursts of them (and giant
//! shared VFX sheets decoded on first use) are what made Isliid and Vesper lag. The repo's pattern
//! (`vesper_core.rs` `fx` / `fx_low`):
//! - a token bucket: TOKENS, one back every REFILL ticks, and at most PER_UPDATE per tick;
//! - important effects (opens, closes, hits, casts) spend any token;
//! - ambient re-spawned visuals (looping layers, projectiles in flight) only spend above RESERVE, and when refused
//!   retry next tick instead of skipping a whole loop.
//!
//! Also: split atlases per rank/look so a match decodes only its own art, keep view-definition counts in the low
//! thousands (Isliid's 9,514 views were cut to 5,034), and never deep-clone big state every tick.

pub const TOKENS: usize = 12;
pub const REFILL: usize = 2;
pub const RESERVE: usize = 4;
pub const PER_UPDATE: usize = 6;

#[derive(Clone, Debug)]
pub struct Budget {
    tokens: usize,
    left: usize,
    pub started: usize,
}

impl Default for Budget {
    fn default() -> Self {
        Self { tokens: TOKENS, left: PER_UPDATE, started: 0 }
    }
}

impl Budget {
    /// Call once at the start of every update.
    pub fn tick(&mut self, tick: usize) {
        self.left = PER_UPDATE;
        if tick % REFILL == 0 && self.tokens < TOKENS {
            self.tokens += 1;
        }
    }
    /// An effect that must show. Returns whether it may be started.
    pub fn fx(&mut self) -> bool {
        if self.left > 0 && self.tokens > 0 {
            self.left -= 1;
            self.tokens -= 1;
            self.started += 1;
            return true;
        }
        false
    }
    /// An ambient effect: only from the tokens above the reserve.
    pub fn fx_low(&mut self) -> bool {
        self.tokens > RESERVE && self.fx()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_busiest_second_is_bounded_and_important_effects_keep_a_reserve() {
        let mut b = Budget::default();
        let mut per_tick = vec![];
        for t in 0..600 {
            b.tick(t);
            let before = b.started;
            while b.fx_low() {}     // a flood of ambient effects
            per_tick.push(b.started - before);
        }
        let peak = per_tick.windows(60).map(|w| w.iter().sum::<usize>()).max().unwrap();
        assert!(peak <= 60 / REFILL + TOKENS, "peak {peak} per second");
        assert!(b.fx(), "an important effect still has a token after the ambient flood");
    }
}
