//! Round 93: optional timing of the mod's hooks, to see in a real game where the time goes. Off unless a file named
//! `perf.flag` sits in mods/tfm2_custom_ai when the game starts; then every 600 ticks (10 s of game) the time spent in
//! each passive, the match hook and the input AI is appended to `perf_log.txt` next to it.
//!
//! The wrappers forward every hook unchanged (same calls, same order, same arguments) and only read the clock, so
//! turning this on or off can't change a match.

use mod_api_stable::{InputV1, StableAiContext, StableMatchHook, StablePassive, StablePlayerAi, StableSim};
use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

fn enabled() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| crate::mod_dir().is_some_and(|d| d.join("perf.flag").exists()))
}

/// (nanoseconds, calls) per hook since the last report.
static TOTALS: Mutex<BTreeMap<&'static str, (u128, u64)>> = Mutex::new(BTreeMap::new());

fn measure<T>(name: &'static str, f: impl FnOnce() -> T) -> T {
    if !enabled() { return f(); }
    let start = Instant::now();
    let out = f();
    let spent = start.elapsed().as_nanos();
    if let Ok(mut t) = TOTALS.lock() {
        let e = t.entry(name).or_insert((0, 0));
        e.0 += spent;
        e.1 += 1;
    }
    out
}

/// Every 600 ticks: one line per hook (ms over those 10 s of game, calls, µs a call), slowest first.
fn report(sim: &StableSim<'_>) {
    if !enabled() || sim.tick() == 0 || !sim.tick().is_multiple_of(600) { return; }
    let Ok(mut t) = TOTALS.lock() else { return };
    let mut rows: Vec<(&'static str, (u128, u64))> = std::mem::take(&mut *t).into_iter().collect();
    drop(t);
    rows.sort_by_key(|r| std::cmp::Reverse(r.1 .0));
    let total: u128 = rows.iter().map(|r| r.1 .0).sum();
    let mut text = format!("game {:x} tick {}: {:.2} ms in the mod over the last 600 ticks (both simulations)\n",
        sim.seed(), sim.tick(), total as f64 / 1e6);
    for (name, (ns, calls)) in rows {
        text += &format!("  {name:<28} {:>8.2} ms {:>7} calls {:>8.1} us/call\n", ns as f64 / 1e6, calls,
            ns as f64 / 1e3 / calls.max(1) as f64);
    }
    if let Some(dir) = crate::mod_dir() {
        use std::io::Write;
        let _ = std::fs::OpenOptions::new().create(true).append(true).open(dir.join("perf_log.txt"))
            .and_then(|mut f| f.write_all(text.as_bytes()));
    }
}

#[derive(Clone)]
pub(crate) struct Timed<P> { pub name: &'static str, pub inner: P }

impl<P: StablePassive + Clone> StablePassive for Timed<P> {
    fn clone_box(&self) -> Box<dyn StablePassive> { Box::new(self.clone()) }
    fn configure(&mut self, params_json: &str) { self.inner.configure(params_json) }
    fn on_spawn(&mut self, sim: &mut StableSim<'_>, player: usize, entity: usize) {
        measure(self.name, || self.inner.on_spawn(sim, player, entity))
    }
    fn on_attack(&mut self, sim: &mut StableSim<'_>, player: usize, entity: usize, target: usize, damage: &mut usize) {
        measure(self.name, || self.inner.on_attack(sim, player, entity, target, damage))
    }
    fn on_damaged(&mut self, sim: &mut StableSim<'_>, player: usize, entity: usize, attacker: usize, damage: usize) {
        measure(self.name, || self.inner.on_damaged(sim, player, entity, attacker, damage))
    }
    fn on_kill(&mut self, sim: &mut StableSim<'_>, player: usize, entity: usize, victim: usize) {
        measure(self.name, || self.inner.on_kill(sim, player, entity, victim))
    }
    fn on_update(&mut self, sim: &mut StableSim<'_>, rng_seed: u64, player: usize, entity: usize) {
        measure(self.name, || self.inner.on_update(sim, rng_seed, player, entity))
    }
    fn on_base_attack(&mut self, sim: &mut StableSim<'_>, rng_seed: u64, player: usize, entity: usize) {
        measure(self.name, || self.inner.on_base_attack(sim, rng_seed, player, entity))
    }
    fn on_assist(&mut self, sim: &mut StableSim<'_>, player: usize, entity: usize) {
        measure(self.name, || self.inner.on_assist(sim, player, entity))
    }
    fn on_dead(&mut self, sim: &mut StableSim<'_>, player: usize) {
        measure(self.name, || self.inner.on_dead(sim, player))
    }
}

pub(crate) struct TimedHook<H>(pub H);

impl<H: StableMatchHook> StableMatchHook for TimedHook<H> {
    fn on_match_start(&self, sim: &mut StableSim<'_>) { measure("match hook", || self.0.on_match_start(sim)) }
    fn on_match_tick(&self, sim: &mut StableSim<'_>, rng_seed: u64) {
        measure("match hook", || self.0.on_match_tick(sim, rng_seed));
        report(sim);
    }
    fn check_match_end(&self, sim: &mut StableSim<'_>) -> Option<bool> { self.0.check_match_end(sim) }
}

#[derive(Clone)]
pub(crate) struct TimedAi<A>(pub A);

impl<A: StablePlayerAi + Clone> StablePlayerAi for TimedAi<A> {
    fn clone_box(&self) -> Box<dyn StablePlayerAi> { Box::new(self.clone()) }
    fn id(&self) -> String { self.0.id() }
    fn priority(&self) -> i32 { self.0.priority() }
    fn matches(&self, init: &mod_api_stable::StableAiInit) -> bool { self.0.matches(init) }
    fn think(&mut self, ctx: &mut StableAiContext<'_>, base_input: Option<InputV1>) -> Option<InputV1> {
        measure("input AI", || self.0.think(ctx, base_input))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn wrappers_forward_every_hook() {
        // the passive, match hook and input AI traits: every method the SDK declares is forwarded
        let sdk = include_str!("../../mod-api-stable/src/traits.rs");
        let me = include_str!("perf.rs");
        for (start, end) in [("pub trait StablePassive", "\n}"), ("pub trait StableMatchHook", "\n}"), ("pub trait StablePlayerAi", "\n}")] {
            let body = &sdk[sdk.find(start).unwrap()..];
            let body = &body[..body.find(end).unwrap()];
            for line in body.lines().map(str::trim).filter(|l| l.starts_with("fn ")) {
                let name = &line[3..line.find('(').unwrap()];
                assert!(me.contains(&format!("    fn {name}(")), "{start}: {name} not forwarded");
            }
        }
    }
}
