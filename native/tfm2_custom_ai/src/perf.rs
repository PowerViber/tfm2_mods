//! Round 93: optional timing of the mod's hooks, to see in a real game where the time goes. Off unless a file named
//! `perf.flag` sits in mods/tfm2_custom_ai when the game starts; then every 600 ticks (10 s of game) the time spent in
//! each passive, the match hook and the input AI is appended to `perf_log.txt` next to it.
//!
//! The wrappers forward every hook unchanged (same calls, same order, same arguments) and only read the clock, so
//! turning this on or off can't change a match.
//!
//! Round 99 (Rian's video: 200-330 ms freezes every ~240 ms in big fights, with no native code anywhere near that slow):
//! the log also counts the view effects the mod plays (per champion, and the most on one tick) and the longest wall-time
//! gap between two match ticks, so it shows whether a freeze is our code, our effects or the game itself.

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

/// Round 99: effects played since the last report: per caster entity, and (tick, count on it, most on one tick).
#[derive(Default)]
struct Fx { by_caster: BTreeMap<usize, u64>, tick: usize, on_tick: u64, peak: u64 }
static FX: Mutex<Option<Fx>> = Mutex::new(None);

/// Round 99: wall time between match ticks since the last report: (last tick's instant, longest gap ns, gaps over 100 ms).
static GAPS: Mutex<(Option<Instant>, u128, u64)> = Mutex::new((None, 0, 0));

/// Round 99: count a view effect played by `caster` on `tick` (lib.rs fx_point / fx_unit). Off without perf.flag.
pub(crate) fn note_fx(caster: usize, tick: usize) {
    if !enabled() { return; }
    let Ok(mut g) = FX.lock() else { return };
    let fx = g.get_or_insert_with(Fx::default);
    *fx.by_caster.entry(caster).or_insert(0) += 1;
    if fx.tick != tick { fx.tick = tick; fx.on_tick = 0; }
    fx.on_tick += 1;
    fx.peak = fx.peak.max(fx.on_tick);
}

fn note_gap() {
    if !enabled() { return; }
    let now = Instant::now();
    if let Ok(mut g) = GAPS.lock() {
        if let Some(last) = g.0 {
            let gap = now.duration_since(last).as_nanos();
            g.1 = g.1.max(gap);
            if gap > 100_000_000 { g.2 += 1; }
        }
        g.0 = Some(now);
    }
}

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
    // round 99: the effects we played and the game's own stalls
    let fx = FX.lock().ok().and_then(|mut g| g.take()).unwrap_or_default();
    let all: u64 = fx.by_caster.values().sum();
    let mut casters: Vec<(usize, u64)> = fx.by_caster.into_iter().collect();
    casters.sort_by_key(|c| std::cmp::Reverse(c.1));
    let who = |id: usize| sim.get_entity(id).and_then(|e| e.name()).unwrap_or_else(|| format!("entity {id}"));
    text += &format!("  effects played: {all} ({:.0} a second), at most {} on one tick\n", all as f64 / 10.0, fx.peak);
    for (id, n) in casters.into_iter().take(4) { text += &format!("    {:<34} {n:>6}\n", who(id)); }
    if let Ok(mut g) = GAPS.lock() {
        text += &format!("  longest gap between two match ticks: {:.0} ms ({} over 100 ms)\n", g.1 as f64 / 1e6, g.2);
        g.1 = 0; g.2 = 0;
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
        note_gap();
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
    fn effect_counts_are_off_without_the_flag() {
        // round 99: counting is a no-op unless perf.flag exists (tests have no mod folder), so play can't change
        super::note_fx(3, 10);
        assert!(super::FX.lock().unwrap().is_none());
        // and every helper that plays a view effect counts it when on
        for (file, src) in [("lib.rs", include_str!("lib.rs")), ("batch2.rs", include_str!("batch2.rs")),
                            ("scribble.rs", include_str!("scribble.rs")), ("gundam.rs", include_str!("gundam.rs"))] {
            for (i, _) in src.match_indices("fn fx") {
                let body = &src[i..i + src[i..].find("\n}").unwrap_or(src.len() - i)];
                if body.contains("play_view_effect") { assert!(body.contains("note_fx"), "{file}: {}", body.lines().next().unwrap_or("")); }
            }
        }
    }

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
