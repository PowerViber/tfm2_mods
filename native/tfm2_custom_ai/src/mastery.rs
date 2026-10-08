//! Round 101: one champion's mastery book, the same mechanics Scribble, Levi and Isliid each carry a copy of, written
//! once and parameterised by its file name (the Coder is the first to use it).
//!
//! Points per athlete live in `<file>_memory.txt`; games played this launch go to `<file>_pending.txt` (merged at the
//! next launch, then appended to `<file>_history.txt`). A match is pinned by its seed so the game's precomputed
//! "server" simulation and the live one see the same memory. Ranks and the Top 10 come from Scribble's `Memory`.

use crate::scribble::Memory;
use crate::{d2, sq, Champ};
use mod_api_stable::{SimOriginV1, StableSim};
use std::collections::{HashMap, HashSet, VecDeque};
use std::io::Write;
use std::sync::{Arc, Mutex, OnceLock};

struct Session {
    lines: Vec<String>,
    pins: HashMap<u64, Arc<Memory>>,
    order: VecDeque<u64>,
    written: HashSet<String>,
}

pub struct Book {
    file: &'static str,
    memory: OnceLock<Memory>,
    session: Mutex<Option<Session>>,
    athletes: Mutex<Option<HashMap<(u64, usize), usize>>>,
}

impl Book {
    pub const fn new(file: &'static str) -> Book {
        Book { file, memory: OnceLock::new(), session: Mutex::new(None), athletes: Mutex::new(None) }
    }

    fn with_session<R>(&self, f: impl FnOnce(&mut Session) -> R) -> Option<R> {
        let mut g = self.session.lock().ok()?;
        let s = g.get_or_insert_with(|| Session { lines: Vec::new(), pins: HashMap::new(), order: VecDeque::new(), written: HashSet::new() });
        Some(f(s))
    }

    /// The memory of this game launch (the saved one plus last launch's pending games, merged once).
    pub fn memory(&self) -> &Memory {
        self.memory.get_or_init(|| {
            let Some(dir) = crate::mod_dir() else { return Memory::default() };
            let mem_p = dir.join(format!("{}_memory.txt", self.file));
            let pend_p = dir.join(format!("{}_pending.txt", self.file));
            let mut m = Memory::parse(&std::fs::read_to_string(&mem_p).unwrap_or_default());
            if let Ok(p) = std::fs::read_to_string(&pend_p) {
                if !p.trim().is_empty() {
                    m.merge(&p);
                    m.meta.clear();
                    if std::fs::write(&mem_p, m.render()).is_ok() {
                        let _ = std::fs::OpenOptions::new().create(true).append(true).open(dir.join(format!("{}_history.txt", self.file)))
                            .and_then(|mut f| f.write_all(p.as_bytes()));
                        let _ = std::fs::write(&pend_p, "");
                    }
                }
            }
            m
        })
    }

    /// The memory this match plays with, pinned by its seed.
    pub fn pinned(&self, seed: u64) -> Arc<Memory> {
        let base = self.memory();
        self.with_session(|s| {
            if let Some(m) = s.pins.get(&seed) { return m.clone(); }
            let mut m = base.clone();
            if !s.lines.is_empty() { m.merge(&s.lines.join("\n")); }
            let m = Arc::new(m);
            s.pins.insert(seed, m.clone());
            s.order.push_back(seed);
            while s.order.len() > 256 {
                if let Some(o) = s.order.pop_front() { s.pins.remove(&o); }
            }
            m
        })
        .unwrap_or_else(|| Arc::new(base.clone()))
    }

    /// Lines for the pending file, each written once (both simulations write the same lines).
    pub fn emit(&self, lines: Vec<String>) {
        let fresh: Vec<String> = self.with_session(|s| {
            if s.written.len() > 200_000 { s.written.clear(); }
            let fresh: Vec<String> = lines.into_iter().filter(|l| s.written.insert(l.clone())).collect();
            s.lines.extend(fresh.iter().cloned());
            fresh
        })
        .unwrap_or_default();
        if fresh.is_empty() { return; }
        let Some(dir) = crate::mod_dir() else { return };
        if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(dir.join(format!("{}_pending.txt", self.file))) {
            let _ = f.write_all((fresh.join("\n") + "\n").as_bytes());
        }
    }

    /// True the first time `key` is seen this launch.
    pub fn first(&self, key: String) -> bool {
        self.with_session(|s| s.written.insert(key)).unwrap_or(false)
    }

    /// A line in `<file>_log.txt`.
    pub fn log(&self, line: &str) {
        let Some(dir) = crate::mod_dir() else { return };
        if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(dir.join(format!("{}_log.txt", self.file))) {
            let _ = f.write_all(format!("{line}\n").as_bytes());
        }
    }

    /// Called by the input AI, the one place the game says which athlete plays a champion.
    pub fn note_athlete(&self, seed: u64, player: usize, athlete: usize) {
        if let Ok(mut g) = self.athletes.lock() {
            let m = g.get_or_insert_with(HashMap::new);
            if m.len() > 4096 { m.clear(); }
            m.insert((seed, player), athlete);
        }
    }

    pub fn athlete_of(&self, seed: u64, player: usize) -> Option<usize> {
        self.athletes.lock().ok().and_then(|g| g.as_ref().and_then(|m| m.get(&(seed, player)).copied()))
    }
}

/// A game's signature (the match id, or "x" for a scrim, and a hash of the line-up and positions), and the "g" line
/// that credits the athlete with the game.
pub fn signature(book: &Book, sim: &StableSim<'_>, all: &[Champ], athlete: Option<usize>) -> String {
    let origin = sim.sim_origin().unwrap_or_default();
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let mut eat = |v: u64| {
        for b in v.to_le_bytes() {
            h ^= b as u64;
            h = h.wrapping_mul(0x100_0000_01b3);
        }
    };
    eat(sim.seed());
    let mut v: Vec<&Champ> = all.iter().collect();
    v.sort_by_key(|c| c.id);
    for c in v {
        eat(c.id as u64);
        eat(c.x as u64);
        eat(c.y as u64);
        for b in c.name.bytes() { eat(b as u64); }
    }
    let mid = if origin.match_id == SimOriginV1::NONE { "x".to_string() } else { format!("{}", origin.match_id) };
    let sig = format!("{mid}.{h:x}");
    if let Some(a) = athlete { book.emit(vec![format!("g {sig} {a}")]); }
    sig
}

fn nexus_pos(team: usize) -> (i64, i64) {
    if team == 0 { (96_000, 864_000) } else { (864_000, 96_000) }
}

/// The structure record of a game (towers, nexus, score), written when it changes or every 10 s; the last one of a
/// game decides the win.
#[derive(Clone, Default)]
pub struct Record {
    last: Option<(i64, i64, i64, i64, i64)>,
    at: usize,
}

impl Record {
    pub fn track(&mut self, book: &Book, sim: &StableSim<'_>, sig: &str, team: usize, athlete: usize, tick: usize) {
        let mut towers = [0i64; 2];
        let mut nexus = [100i64; 2];
        let mut best = [i128::MAX; 2];
        for i in 0..sim.tower_count() {
            let id = sim.tower_id_at(i);
            let Some(e) = sim.get_entity(id) else { continue };
            let t = e.team().min(1);
            let (x, y) = e.pos();
            let alive = e.is_alive();
            if alive { towers[t] += 1; }
            let (nx, ny) = nexus_pos(t);
            let d = d2(x as i64, y as i64, nx, ny);
            if d <= sq(70_000) && d < best[t] {
                best[t] = d;
                let (hp, mx) = e.hp();
                nexus[t] = if !alive || mx == 0 { 0 } else { (hp * 100 / mx) as i64 };
            }
        }
        let me = team.min(1);
        let rec = (towers[me], towers[1 - me], nexus[me], nexus[1 - me], sim.score_diff(team) as i64);
        if Some(rec) != self.last || tick >= self.at + 600 {
            self.last = Some(rec);
            self.at = tick;
            book.emit(vec![format!("r {sig} {athlete} {tick} {} {} {} {} {}", rec.0, rec.1, rec.2, rec.3, rec.4)]);
        }
    }
}
