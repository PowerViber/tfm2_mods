//! Buffered diagnostics for Coder's scripted damage. No combat state or random draws live here.
//!
//! Watched simulations are recorded; known server presimulations are omitted. On older hosts with
//! no origin API, the first simulation instance for each seed/caster owns the trace, so the two
//! copies of a match do not double its damage log. Both maps are bounded to 256 entries.

use mod_api_stable::{AttackTypeV1, SimOriginKindV1, SimOriginV1, StableSim};
use std::collections::{HashMap, VecDeque};
use std::io::{self, Write};
use std::path::Path;
use std::sync::Mutex;

const MAX_STREAMS: usize = 256;
const MAX_EVENTS: usize = 512;
const MAX_BUFFER_BYTES: usize = 128 * 1024;
const FLUSH_TICKS: usize = 60;
const ROTATE_BYTES: u64 = 8 * 1024 * 1024;
const LOG_NAME: &str = "coder_damage_log.txt";
const PREVIOUS_NAME: &str = "coder_damage_log.previous.txt";

#[derive(Clone, Copy, Debug)]
pub(super) struct Source {
    pub f: usize,
    pub lang: usize,
    pub cast_tick: usize,
    pub ap: usize,
    pub power: usize,
    pub ghz: usize,
    pub scale: usize,
    pub athlete: Option<usize>,
    pub rank: Option<usize>,
    pub root: Option<usize>,
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
struct StreamKey {
    seed: u64,
    instance: usize,
    caster: usize,
}

#[derive(Clone, Copy, Debug)]
struct Context {
    key: StreamKey,
    tick: usize,
    origin: SimOriginV1,
}

impl Context {
    fn read(sim: &StableSim<'_>, caster: usize) -> Self {
        Self {
            key: StreamKey {
                seed: sim.seed(),
                instance: sim.instance(),
                caster,
            },
            tick: sim.tick(),
            origin: sim.sim_origin().unwrap_or_default(),
        }
    }

    fn is_server(self) -> bool {
        self.origin.kind == SimOriginKindV1::ServerPresim.code()
    }
}

#[derive(Clone, Copy, Debug)]
struct TargetState {
    hp: usize,
    max_hp: usize,
    alive: bool,
}

fn target_state(sim: &StableSim<'_>, target: usize) -> Option<TargetState> {
    sim.get_entity(target).map(|e| {
        let (hp, max_hp) = e.hp();
        TargetState {
            hp,
            max_hp,
            alive: e.is_alive(),
        }
    })
}

/// Send exactly the original raw AP damage through the engine, then record its whole pipeline.
/// HP loss can include item/buff procs; it is not a reimplementation of mitigation math.
pub(super) fn deal_damage(
    sim: &mut StableSim<'_>,
    caster: usize,
    target: usize,
    amount: usize,
    kind: AttackTypeV1,
    source: Source,
    delivery: &str,
    drones: usize,
) {
    let context = Context::read(sim, caster);
    if context.is_server() {
        sim.deal_damage(caster, target, 0, amount, kind);
        return;
    }
    let before = target_state(sim, target);
    let ap_now = sim.get_entity(caster).map(|e| e.stat().magic_power);
    sim.deal_damage(caster, target, 0, amount, kind);
    let after = target_state(sim, target);
    let force = after.is_none()
        || after.is_some_and(|s| !s.alive || s.hp == 0)
        || sim.get_entity(caster).is_some_and(|e| !e.is_alive());
    record(
        context,
        target,
        Some(amount),
        Some(kind.code()),
        source,
        delivery,
        drones,
        ap_now,
        before,
        after,
        force,
    );
}

/// Preserve the flipped HEAL's direct HP subtraction and its existing nonlethal floor.
pub(super) fn set_hp_damage(
    sim: &mut StableSim<'_>,
    caster: usize,
    target: usize,
    amount: usize,
    source: Source,
    drones: usize,
) {
    let before = target_state(sim, target);
    let Some(state) = before else { return };
    let context = Context::read(sim, caster);
    let ap_now = if context.is_server() {
        None
    } else {
        sim.get_entity(caster).map(|e| e.stat().magic_power)
    };
    sim.entity_set_hp(target, state.hp.saturating_sub(amount).max(1));
    if context.is_server() {
        return;
    }
    let after = target_state(sim, target);
    record(
        context,
        target,
        Some(amount),
        Some(AttackTypeV1::Skill.code()),
        source,
        "direct_hp",
        drones,
        ap_now,
        before,
        after,
        false,
    );
}

/// Engine kill credit is separate from a scripted hit: it can come from a basic attack, a
/// reflected hit or an item proc, so it deliberately does not name a function as the cause.
pub(super) fn kill_credit(sim: &StableSim<'_>, caster: usize, victim: usize, source: Source) {
    let context = Context::read(sim, caster);
    if context.is_server() {
        return;
    }
    let after = target_state(sim, victim);
    let ap_now = sim.get_entity(caster).map(|e| e.stat().magic_power);
    record(
        context,
        victim,
        None,
        None,
        source,
        "kill_credit",
        0,
        ap_now,
        None,
        after,
        true,
    );
}

struct Buffer {
    context: Context,
    source: Source,
    lines: Vec<String>,
    bytes: usize,
    last_flush: usize,
    sequence_tick: usize,
    sequence: usize,
}

impl Buffer {
    fn new(context: Context, source: Source) -> Self {
        Self {
            context,
            source,
            lines: Vec::new(),
            bytes: 0,
            last_flush: context.tick,
            sequence_tick: context.tick,
            sequence: 0,
        }
    }

    fn next_sequence(&mut self, tick: usize) -> usize {
        if self.sequence_tick != tick {
            self.sequence_tick = tick;
            self.sequence = 0;
        }
        self.sequence += 1;
        self.sequence
    }

    fn take_batch(&mut self, tick: usize) -> Option<String> {
        self.last_flush = tick;
        if self.lines.is_empty() {
            return None;
        }
        let mut batch = header(self.context, self.source);
        for line in self.lines.drain(..) {
            batch.push_str(&line);
            batch.push('\n');
        }
        self.bytes = 0;
        Some(batch)
    }
}

#[derive(Default)]
struct Trace {
    buffers: HashMap<StreamKey, Buffer>,
    order: VecDeque<StreamKey>,
    fallback_owners: HashMap<(u64, usize), usize>,
    fallback_order: VecDeque<(u64, usize)>,
}

impl Trace {
    fn owns(&mut self, context: Context) -> bool {
        if context.is_server() {
            return false;
        }
        if context.origin.kind != SimOriginKindV1::Unknown.code() {
            return true;
        }
        let key = (context.key.seed, context.key.caster);
        if let Some(instance) = self.fallback_owners.get(&key) {
            return *instance == context.key.instance;
        }
        while self.fallback_owners.len() >= MAX_STREAMS {
            if let Some(old) = self.fallback_order.pop_front() {
                self.fallback_owners.remove(&old);
            }
        }
        self.fallback_owners.insert(key, context.key.instance);
        self.fallback_order.push_back(key);
        true
    }

    fn push(
        &mut self,
        context: Context,
        source: Source,
        line: impl FnOnce(usize) -> String,
    ) -> Vec<String> {
        if !self.owns(context) {
            return Vec::new();
        }
        let mut batches = Vec::new();
        if !self.buffers.contains_key(&context.key) {
            while self.buffers.len() >= MAX_STREAMS {
                if let Some(old) = self.order.pop_front() {
                    if let Some(mut buffer) = self.buffers.remove(&old) {
                        if let Some(batch) = buffer.take_batch(context.tick) {
                            batches.push(batch);
                        }
                    }
                }
            }
            self.buffers
                .insert(context.key, Buffer::new(context, source));
            self.order.push_back(context.key);
        }
        let buffer = self.buffers.get_mut(&context.key).unwrap();
        buffer.context = context;
        buffer.source = source;
        let sequence = buffer.next_sequence(context.tick);
        let line = line(sequence);
        buffer.bytes += line.len() + 1;
        buffer.lines.push(line);
        if buffer.lines.len() >= MAX_EVENTS || buffer.bytes >= MAX_BUFFER_BYTES {
            if let Some(batch) = buffer.take_batch(context.tick) {
                batches.push(batch);
            }
        }
        batches
    }

    fn flush(&mut self, context: Context, force: bool) -> Option<String> {
        if !self.owns(context) {
            return None;
        }
        let buffer = self.buffers.get_mut(&context.key)?;
        if force || context.tick.saturating_sub(buffer.last_flush) >= FLUSH_TICKS {
            buffer.take_batch(context.tick)
        } else {
            None
        }
    }
}

fn origin_label(kind: u32) -> &'static str {
    match kind {
        1 => "server_presim",
        2 => "live",
        3 => "spectate",
        4 => "replay",
        5 => "tool",
        _ => "unknown(first-instance)",
    }
}

fn optional(n: Option<usize>) -> String {
    n.map_or_else(|| "?".to_string(), |n| n.to_string())
}

fn origin_id(n: u64) -> String {
    if n == SimOriginV1::NONE {
        "?".to_string()
    } else {
        n.to_string()
    }
}

fn header(context: Context, source: Source) -> String {
    let rank_name = source
        .rank
        .and_then(|rank| super::RANK_NAMES.get(rank))
        .copied()
        .unwrap_or("?");
    let root = source.root.map_or(String::new(), |r| format!(" #{r}"));
    format!(
        "== Coder damage native {} game {:x} match {} set {} origin {} caster {} athlete {} rank {} ({}{}); hp_loss includes the engine pipeline and item/buff procs; kill_credit has no guessed function attribution\n",
        crate::VERSION,
        context.key.seed,
        origin_id(context.origin.match_id),
        origin_id(context.origin.set_index),
        origin_label(context.origin.kind),
        context.key.caster,
        optional(source.athlete),
        optional(source.rank),
        rank_name,
        root,
    )
}

static TRACE: Mutex<Option<Trace>> = Mutex::new(None);

#[cfg(test)]
#[derive(Default)]
struct TestCapture {
    lines: Vec<String>,
    tick: Option<usize>,
    sequence: usize,
}

#[cfg(test)]
thread_local! {
    static TEST_CAPTURE: std::cell::RefCell<TestCapture> = std::cell::RefCell::new(TestCapture::default());
}

/// Host fixtures collect their own thread's diagnostic events without touching real log files or
/// sharing first-instance ownership with another simultaneously running fixture.
#[cfg(test)]
pub(super) fn take_test_lines() -> Vec<String> {
    TEST_CAPTURE.with(|capture| {
        let mut capture = capture.borrow_mut();
        capture.tick = None;
        capture.sequence = 0;
        std::mem::take(&mut capture.lines)
    })
}

fn record(
    context: Context,
    target: usize,
    amount: Option<usize>,
    kind: Option<u32>,
    source: Source,
    delivery: &str,
    drones: usize,
    ap_now: Option<usize>,
    before: Option<TargetState>,
    after: Option<TargetState>,
    force: bool,
) {
    let format_line = |sequence| {
        let language = super::LANGS.get(source.lang).copied().unwrap_or("?");
        let function = super::FUNCS.get(source.f);
        let scripted = function.is_some() && delivery != "kill_credit";
        let effect = function.map_or_else(
            || "engine_credit".to_string(),
            |f| format!("{}.{}", f.0, language),
        );
        let hp_loss = match (before, after) {
            (Some(b), Some(a)) => b.hp.saturating_sub(a.hp).to_string(),
            _ => "?".to_string(),
        };
        let killed = delivery == "kill_credit"
            || (before.is_some_and(|s| s.alive) && after.is_some_and(|s| !s.alive || s.hp == 0));
        format!(
            "game {:x} tick {} event {} caster {} athlete {} rank {} target {}: {} delivery={} cast_tick={} replay_scale={} power_pct={} ghz_x100={} cast_ap={} ap_now={} drones={} raw_ap={} attack_type={} hp={}->{} max_hp={} hp_loss={} killed={}",
            context.key.seed,
            context.tick,
            sequence,
            context.key.caster,
            optional(source.athlete),
            optional(source.rank),
            target,
            effect,
            delivery,
            optional(scripted.then_some(source.cast_tick)),
            optional(scripted.then_some(source.scale)),
            optional(scripted.then_some(source.power)),
            source.ghz,
            optional(scripted.then_some(source.ap)),
            optional(ap_now),
            optional(scripted.then_some(drones)),
            optional(amount),
            optional(kind.map(|k| k as usize)),
            optional(before.map(|s| s.hp)),
            optional(after.map(|s| s.hp)),
            optional(before.map(|s| s.max_hp).or(after.map(|s| s.max_hp))),
            hp_loss,
            killed,
        )
    };
    #[cfg(test)]
    {
        let _ = force;
        TEST_CAPTURE.with(|capture| {
            let mut capture = capture.borrow_mut();
            if capture.tick != Some(context.tick) {
                capture.tick = Some(context.tick);
                capture.sequence = 0;
            }
            capture.sequence += 1;
            let sequence = capture.sequence;
            capture.lines.push(format_line(sequence));
        });
    }
    #[cfg(not(test))]
    {
        let Ok(mut guard) = TRACE.lock() else { return };
        let trace = guard.get_or_insert_with(Trace::default);
        let mut batches = trace.push(context, source, format_line);
        if force {
            if let Some(batch) = trace.flush(context, true) {
                batches.push(batch);
            }
        }
        write_batches(batches);
    }
}

/// Called every update, including before frozen/dead early returns. No file writes with an empty buffer.
pub(super) fn flush(sim: &StableSim<'_>, caster: usize, force: bool) {
    let context = Context::read(sim, caster);
    if context.is_server() {
        return;
    }
    let Ok(mut guard) = TRACE.lock() else { return };
    let Some(trace) = guard.as_mut() else { return };
    if let Some(batch) = trace.flush(context, force || sim.is_end()) {
        write_batches(vec![batch]);
    }
}

fn write_batches(batches: Vec<String>) {
    if batches.is_empty() {
        return;
    }
    let Some(dir) = crate::mod_dir() else { return };
    // A failed diagnostic write never changes gameplay or grows the buffers without bound.
    let _ = append_batch(&dir, &batches.concat(), ROTATE_BYTES);
}

fn append_batch(dir: &Path, text: &str, rotate_bytes: u64) -> io::Result<()> {
    if text.is_empty() {
        return Ok(());
    }
    let current = dir.join(LOG_NAME);
    let previous = dir.join(PREVIOUS_NAME);
    let existing = std::fs::metadata(&current).map_or(0, |m| m.len());
    if existing > 0 && existing.saturating_add(text.len() as u64) > rotate_bytes {
        match std::fs::remove_file(&previous) {
            Ok(()) => {}
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(e) => return Err(e),
        }
        std::fs::rename(&current, &previous)?;
    }
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(current)?
        .write_all(text.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn context(seed: u64, instance: usize, tick: usize, kind: SimOriginKindV1) -> Context {
        Context {
            key: StreamKey {
                seed,
                instance,
                caster: 7,
            },
            tick,
            origin: SimOriginV1 {
                kind: kind.code(),
                match_id: 42,
                set_index: 1,
                ..SimOriginV1::default()
            },
        }
    }

    fn source() -> Source {
        Source {
            f: 0,
            lang: 0,
            cast_tick: 100,
            ap: 120,
            power: 140,
            ghz: 300,
            scale: 0,
            athlete: Some(976),
            rank: Some(3),
            root: None,
        }
    }

    #[test]
    fn event_lines_report_observed_hp_loss_and_do_not_guess_kill_credit_source() {
        take_test_lines();
        let ctx = context(1, 10, 100, SimOriginKindV1::ClientMatchView);
        let mut cast = source();
        cast.scale = 50;
        record(
            ctx,
            8,
            Some(65),
            Some(AttackTypeV1::Skill.code()),
            cast,
            "packet",
            3,
            Some(200),
            Some(TargetState {
                hp: 200,
                max_hp: 1000,
                alive: true,
            }),
            Some(TargetState {
                hp: 40,
                max_hp: 1000,
                alive: true,
            }),
            false,
        );
        let hit = take_test_lines().pop().unwrap();
        assert!(hit.contains("ping.py delivery=packet cast_tick=100 replay_scale=50"));
        assert!(
            hit.contains("power_pct=140 ghz_x100=300 cast_ap=120 ap_now=200 drones=3 raw_ap=65")
        );
        assert!(hit.contains("hp=200->40 max_hp=1000 hp_loss=160 killed=false"));
        cast.f = usize::MAX;
        record(
            ctx,
            9,
            None,
            None,
            cast,
            "kill_credit",
            0,
            Some(200),
            None,
            Some(TargetState {
                hp: 0,
                max_hp: 1000,
                alive: false,
            }),
            true,
        );
        let credit = take_test_lines().pop().unwrap();
        assert!(credit.contains("engine_credit delivery=kill_credit"));
        assert!(!credit.contains("engine_credit.py"));
        assert!(credit.contains("cast_tick=? replay_scale=? power_pct=?"));
        assert!(credit.contains("cast_ap=? ap_now=200 drones=?"));
        assert!(credit.contains("raw_ap=? attack_type=? hp=?->0 max_hp=1000 hp_loss=? killed=true"));
        // An engine kill callback remains authoritative if the victim has already been removed.
        record(
            ctx,
            9,
            None,
            None,
            cast,
            "kill_credit",
            0,
            Some(200),
            None,
            None,
            true,
        );
        let removed = take_test_lines().pop().unwrap();
        assert!(removed.contains("hp=?->? max_hp=? hp_loss=? killed=true"));
    }

    #[test]
    fn presim_is_skipped_but_each_watched_origin_is_recorded() {
        let mut trace = Trace::default();
        let presim = context(1, 10, 100, SimOriginKindV1::ServerPresim);
        assert!(trace
            .push(presim, source(), |_| panic!("presim must not format"))
            .is_empty());
        assert!(trace.buffers.is_empty());
        for kind in [
            SimOriginKindV1::ClientMatchView,
            SimOriginKindV1::ClientSpectate,
            SimOriginKindV1::ClientReplay,
            SimOriginKindV1::Tool,
        ] {
            let watched = context(1, kind.code() as usize, 100, kind);
            assert!(trace
                .push(watched, source(), |n| format!("event {n}"))
                .is_empty());
            let batch = trace.flush(watched, true).unwrap();
            assert!(batch.contains(origin_label(kind.code())));
            assert!(batch.ends_with("event 1\n"));
        }
    }

    #[test]
    fn unknown_host_logs_only_the_first_instance_for_each_seed_and_caster() {
        let mut trace = Trace::default();
        let first = context(1, 10, 100, SimOriginKindV1::Unknown);
        let duplicate = context(1, 11, 100, SimOriginKindV1::Unknown);
        trace.push(first, source(), |n| format!("first {n}"));
        trace.push(duplicate, source(), |_| panic!("duplicate must not format"));
        assert!(trace.flush(duplicate, true).is_none());
        assert!(trace.flush(first, true).unwrap().ends_with("first 1\n"));
        let mut other_caster = duplicate;
        other_caster.key.caster = 8;
        trace.push(other_caster, source(), |n| format!("other {n}"));
        assert!(trace
            .flush(other_caster, true)
            .unwrap()
            .ends_with("other 1\n"));
        let other_game = context(2, 11, 100, SimOriginKindV1::Unknown);
        trace.push(other_game, source(), |n| format!("game {n}"));
        assert!(trace.flush(other_game, true).unwrap().ends_with("game 1\n"));
    }

    #[test]
    fn batching_keeps_every_hit_and_waits_one_second_or_force() {
        let mut trace = Trace::default();
        let mut ctx = context(1, 10, 100, SimOriginKindV1::ClientMatchView);
        trace.push(ctx, source(), |n| format!("first {n}"));
        trace.push(ctx, source(), |n| format!("second {n}"));
        ctx.tick = 101;
        trace.push(ctx, source(), |n| format!("next_tick {n}"));
        ctx.tick = 159;
        assert!(trace.flush(ctx, false).is_none());
        ctx.tick = 160;
        let batch = trace.flush(ctx, false).unwrap();
        assert!(batch.ends_with("first 1\nsecond 2\nnext_tick 1\n"));
        assert!(batch.contains("native "));
        assert!(
            batch.contains("match 42 set 1 origin live caster 7 athlete 976 rank 3 (Developer)")
        );
        assert!(trace.flush(ctx, true).is_none());
        trace.push(ctx, source(), |n| format!("forced {n}"));
        assert!(trace.flush(ctx, true).unwrap().ends_with("forced 1\n"));
    }

    #[test]
    fn event_and_byte_limits_flush_batches_and_preserve_sequence() {
        let mut trace = Trace::default();
        let ctx = context(1, 10, 100, SimOriginKindV1::ClientMatchView);
        for _ in 0..MAX_EVENTS - 1 {
            assert!(trace.push(ctx, source(), |n| format!("hit {n}")).is_empty());
        }
        let batches = trace.push(ctx, source(), |n| format!("hit {n}"));
        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0].lines().count(), MAX_EVENTS + 1);
        assert!(batches[0].ends_with("hit 512\n"));
        trace.push(ctx, source(), |n| format!("hit {n}"));
        assert!(trace.flush(ctx, true).unwrap().ends_with("hit 513\n"));
        let batches = trace.push(ctx, source(), |_| "x".repeat(MAX_BUFFER_BYTES));
        assert_eq!(batches.len(), 1);
        let buffer = trace.buffers.get(&ctx.key).unwrap();
        assert!(buffer.lines.is_empty());
        assert_eq!(buffer.bytes, 0);
    }

    #[test]
    fn maps_are_bounded_and_evicted_pending_events_are_flushed() {
        let mut trace = Trace::default();
        for seed in 0..MAX_STREAMS as u64 {
            let ctx = context(seed, seed as usize, 100, SimOriginKindV1::Unknown);
            assert!(trace
                .push(ctx, source(), |_| format!("seed {seed}"))
                .is_empty());
        }
        let ctx = context(
            MAX_STREAMS as u64,
            MAX_STREAMS,
            101,
            SimOriginKindV1::Unknown,
        );
        let batches = trace.push(ctx, source(), |_| "newest".to_string());
        assert_eq!(batches.len(), 1);
        assert!(batches[0].ends_with("seed 0\n"));
        assert_eq!(trace.buffers.len(), MAX_STREAMS);
        assert_eq!(trace.order.len(), MAX_STREAMS);
        assert_eq!(trace.fallback_owners.len(), MAX_STREAMS);
        assert_eq!(trace.fallback_order.len(), MAX_STREAMS);
    }

    struct TempDir(std::path::PathBuf);

    impl TempDir {
        fn new() -> Self {
            static NEXT: AtomicUsize = AtomicUsize::new(0);
            let path = std::env::temp_dir().join(format!(
                "tfm2-coder-trace-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed),
            ));
            std::fs::create_dir(&path).unwrap();
            Self(path)
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn append_and_rotation_keep_two_complete_readable_logs() {
        let dir = TempDir::new();
        append_batch(&dir.0, "first\n", 16).unwrap();
        append_batch(&dir.0, "second\n", 16).unwrap();
        assert_eq!(
            std::fs::read_to_string(dir.0.join(LOG_NAME)).unwrap(),
            "first\nsecond\n"
        );
        append_batch(&dir.0, "third\n", 16).unwrap();
        assert_eq!(
            std::fs::read_to_string(dir.0.join(PREVIOUS_NAME)).unwrap(),
            "first\nsecond\n"
        );
        assert_eq!(
            std::fs::read_to_string(dir.0.join(LOG_NAME)).unwrap(),
            "third\n"
        );
        append_batch(&dir.0, "fourth line\n", 16).unwrap();
        assert_eq!(
            std::fs::read_to_string(dir.0.join(PREVIOUS_NAME)).unwrap(),
            "third\n"
        );
        assert_eq!(
            std::fs::read_to_string(dir.0.join(LOG_NAME)).unwrap(),
            "fourth line\n"
        );
        append_batch(&dir.0, "", 16).unwrap();
        assert_eq!(
            std::fs::read_to_string(dir.0.join(LOG_NAME)).unwrap(),
            "fourth line\n"
        );
    }
}
