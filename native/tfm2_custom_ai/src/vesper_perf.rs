//! Opt-in native/host timings. All state belongs to one passive, including cloned simulations.
use mod_api_stable::StableSim;
use std::{
    collections::{BTreeMap, BTreeSet},
    io::Write,
    time::Instant,
};
include!("vesper_assets.rs");
#[derive(Clone, Default)]
pub struct Profile {
    samples: BTreeMap<&'static str, Vec<u64>>,
    starts: BTreeMap<&'static str, Instant>,
    effects: BTreeMap<String, (usize, u64, u64)>,
    assets: BTreeSet<&'static str>,
    first_bytes: usize,
    signatures: BTreeSet<(String, i64, i64, Option<usize>)>,
    effect_tick: usize,
    repeated_signatures: usize,
    peak: usize,
    count: usize,
    active: Vec<usize>,
    peak_active: usize,
}
impl Profile {
    pub fn start(&self) -> Option<Instant> {
        crate::perf::enabled().then(Instant::now)
    }
    pub fn finish(&mut self, name: &'static str, at: Option<Instant>) {
        if let Some(at) = at {
            let samples = self.samples.entry(name).or_default();
            if samples.len() < 2048 {
                samples.push(at.elapsed().as_nanos().min(u64::MAX as u128) as u64);
            }
        }
    }
    pub fn phase(&mut self, name: &'static str, start: bool) {
        if !crate::perf::enabled() {
            return;
        }
        if start {
            self.starts.insert(name, Instant::now());
        } else {
            let at = self.starts.remove(name);
            self.finish(name, at);
        }
    }
    pub fn asset(&mut self, tag: &str) {
        if !crate::perf::enabled() {
            return;
        }
        if let Some((name, bytes)) = asset_info(tag) {
            if self.assets.insert(name) {
                self.first_bytes += bytes;
            }
        }
    }
    pub fn effect(
        &mut self,
        tag: &str,
        x: i64,
        y: i64,
        unit: Option<usize>,
        tick: usize,
        ticks: usize,
        at: Option<Instant>,
    ) {
        if !crate::perf::enabled() {
            return;
        }
        self.asset(tag);
        let ns = at.map_or(0, |at| at.elapsed().as_nanos().min(u64::MAX as u128) as u64);
        let entry = self.effects.entry(tag.into()).or_default();
        entry.0 += 1;
        entry.1 += ns;
        entry.2 = entry.2.max(ns);
        if self.effect_tick != tick {
            self.effect_tick = tick;
            self.signatures.clear();
            self.count = 0;
        }
        if !self.signatures.insert((tag.into(), x, y, unit)) {
            self.repeated_signatures += 1;
        }
        self.count += 1;
        self.peak = self.peak.max(self.count);
        self.active.retain(|until| *until > tick);
        if self.active.len() < 512 {
            self.active.push(tick + ticks);
        }
        self.peak_active = self.peak_active.max(self.active.len());
    }
    pub fn flush(&mut self, sim: &StableSim<'_>, owner: usize) {
        if !crate::perf::enabled() || sim.tick() == 0 || !sim.tick().is_multiple_of(600) {
            return;
        }
        let Some(dir) = crate::mod_dir() else {
            return;
        };
        let origin = sim.sim_origin().map(|o| o.kind);
        let mut text = format!(
            "seed {} instance {} origin {:?} owner {owner} tick {}: Vesper profile\n",
            sim.seed(),
            sim.instance(),
            origin,
            sim.tick()
        );
        for (name, mut values) in std::mem::take(&mut self.samples) {
            values.sort_unstable();
            let n = values.len();
            if n == 0 {
                continue;
            }
            let percentile = |p: usize| values[((n - 1) * p / 100).min(n - 1)] as f64 / 1e6;
            let sum: u64 = values.iter().sum();
            text += &format!(
                "  {name}: n={n} total_ms={:.3} p95_ms={:.4} p99_ms={:.4} max_ms={:.4}\n",
                sum as f64 / 1e6,
                percentile(95),
                percentile(99),
                percentile(100)
            );
        }
        text+=&format!("  effect_peak_per_tick={} estimated_peak_active={} first_asset_bytes={} repeated_signatures={} (audit only; no effects suppressed)\n",self.peak,self.peak_active,self.first_bytes,self.repeated_signatures);
        let mut effects: Vec<_> = std::mem::take(&mut self.effects).into_iter().collect();
        effects.sort_by_key(|(tag, (n, _, _))| (std::cmp::Reverse(*n), tag.clone()));
        for (tag, (n, total, max)) in effects.into_iter().take(32) {
            text += &format!(
                "    {tag}: starts={n} host_total_ms={:.3} host_max_ms={:.3}\n",
                total as f64 / 1e6,
                max as f64 / 1e6
            );
        }
        let path = dir.join("vesper_perf_log.txt");
        if std::fs::metadata(&path).is_ok_and(|m| m.len() > 8 * 1024 * 1024) {
            let _ = std::fs::rename(&path, path.with_extension("previous.txt"));
        }
        let _ = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .and_then(|mut file| file.write_all(text.as_bytes()));
        self.first_bytes = 0;
        self.repeated_signatures = 0;
        self.peak = 0;
        self.peak_active = 0;
    }
}
