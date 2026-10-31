//! Opt-in callback-local diagnostics. Never feeds timings back into gameplay.
use mod_api_stable::StableSim;
use std::{collections::BTreeMap, io::Write, time::Instant};
#[derive(Clone, Default)]
pub struct Profile {
    identity: Option<(usize, u64, usize)>,
    samples: BTreeMap<&'static str, Vec<u64>>,
    last_flush: Option<usize>,
    initial: bool,
}
/// Round 145: tests time the phases without a perf.flag file.
#[cfg(test)]
pub(crate) static FORCE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
fn on() -> bool {
    #[cfg(test)]
    if FORCE.load(std::sync::atomic::Ordering::Relaxed) {
        return true;
    }
    crate::perf::enabled()
}
impl Profile {
    /// Median nanoseconds per phase so far (tests).
    #[cfg(test)]
    pub fn medians(&self) -> Vec<(&'static str, u64)> {
        self.samples
            .iter()
            .map(|(k, v)| {
                let mut v = v.clone();
                v.sort_unstable();
                (*k, v.get(v.len() / 2).copied().unwrap_or(0))
            })
            .collect()
    }
    pub fn bind(&mut self, sim: &StableSim, owner: usize) {
        if !on() {
            return;
        }
        let identity = (sim.instance(), sim.seed(), owner);
        if self.identity != Some(identity) {
            *self = Self {
                identity: Some(identity),
                initial: true,
                ..Self::default()
            };
        }
    }
    pub fn start(&self) -> Option<Instant> {
        on().then(Instant::now)
    }
    pub fn finish(&mut self, phase: &'static str, start: Option<Instant>) {
        if let Some(at) = start {
            let a = self.samples.entry(phase).or_default();
            if a.len() < 2048 {
                a.push(at.elapsed().as_nanos().min(u64::MAX as u128) as u64);
            }
        }
    }
    pub fn flush(&mut self, sim: &StableSim, owner: usize, name: &str) {
        if !crate::perf::enabled(){return;}
        let tick = sim.tick();
        if tick == 0
            || !tick.is_multiple_of(600)
            || self.last_flush == Some(tick)
        {
            return;
        }
        self.last_flush = Some(tick);
        let Some(dir) = crate::mod_dir() else {
            return;
        };
        let mut report=format!("{name} seed {} instance {} origin {:?} owner {owner} tick {tick} initial_window {} (not renderer texture-cache state)\n",sim.seed(),sim.instance(),sim.sim_origin().map(|o|o.kind),self.initial);
        self.initial = false;
        for (phase, mut a) in std::mem::take(&mut self.samples) {
            a.sort_unstable();
            if a.is_empty() {
                continue;
            }
            let p = |n| a[(a.len() - 1) * n / 100] as f64 / 1e6;
            report += &format!(
                "  {phase}: n={} p95_ms={:.5} p99_ms={:.5} max_ms={:.5}\n",
                a.len(),
                p(95),
                p(99),
                p(100)
            );
        }
        let path = dir.join("adapter_perf_log.txt");
        if std::fs::metadata(&path).is_ok_and(|m| m.len() > 8 * 1024 * 1024) {
            let _ = std::fs::rename(&path, path.with_extension("previous.txt"));
        }
        let _ = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .and_then(|mut f| f.write_all(report.as_bytes()));
    }
}
