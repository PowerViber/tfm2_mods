//! Round 141: how many host calls one game tick costs the mod (the "game is slow, replays at 4× can't
//! slow down for highlights" report). A mock match of 10 champions (12 buffs each) and 200 other units runs
//! the per-player move path the input AI takes for every player each tick (profile scene, team guidance,
//! staging, the move pipeline's champion snapshot, map plans, smoke check, rally), plus the team planner's
//! tick. `tick_budget` (ignored) prints host calls and time per tick; the other tests pin that caching
//! never changes an answer.
use super::*;
use mod_api_stable::{
    BuffV1, CcV1, EntityHandleV1 as E, EntityKindV1 as K, LaneV1, PlayerHandleV1 as P, SimCtxV1, SimVtableV1,
    StatV1,
};
use std::{cell::Cell, ffi::c_void, mem::size_of, time::Instant};

const CHAMPS: usize = 10;
const UNITS: usize = 210;
const NAMES: [&str; CHAMPS] = [
    "tfm2_custom_gojo", "tfm2_valorant_omen", "tfm2_blockcraft_steve", "archer", "fighter",
    "tfm2_levi_levi", "tfm2_custom_magnus", "tfm2_jojo_dio", "bard", "ninja",
];

struct Host {
    calls: Cell<usize>,
    tick: Cell<usize>,
    /// buffs added during the match (the planner's clock and plan marks), per entity
    added: std::cell::RefCell<std::collections::HashMap<usize, Vec<BuffV1>>>,
    /// champion names (a fixture can put Raze or Contender on any slot)
    names: std::cell::RefCell<[&'static str; CHAMPS]>,
    /// view effects played
    effects: Cell<usize>,
}
impl Host {
    fn new(tick: usize) -> Self {
        Host { calls: Cell::new(0), tick: Cell::new(tick), added: Default::default(), names: std::cell::RefCell::new(NAMES), effects: Cell::new(0) }
    }
}
unsafe fn host<'a>(p: *const c_void) -> &'a Host {
    let h = &*(p as *const Host);
    h.calls.set(h.calls.get() + 1);
    h
}
fn id(e: E) -> usize {
    e.id().unwrap_or(usize::MAX)
}
unsafe extern "C" fn tick(p: *const c_void) -> usize { host(p).tick.get() }
unsafe extern "C" fn seed(p: *const c_void) -> u64 { host(p); 7 }
unsafe extern "C" fn count(p: *const c_void) -> usize { host(p); UNITS }
unsafe extern "C" fn at(p: *const c_void, i: usize) -> E { host(p); E::from_id(i) }
unsafe extern "C" fn valid(p: *const c_void, e: E) -> bool { host(p); id(e) < UNITS }
unsafe extern "C" fn alive(p: *const c_void, e: E) -> bool { host(p); id(e) < UNITS }
unsafe extern "C" fn targetable(p: *const c_void, e: E) -> bool { host(p); id(e) < UNITS }
unsafe extern "C" fn team(p: *const c_void, e: E) -> usize {
    host(p);
    let i = id(e);
    if i < CHAMPS { usize::from(i >= 5) } else if i < 150 { i % 2 } else { 2 }
}
unsafe extern "C" fn kind(p: *const c_void, e: E) -> u32 {
    host(p);
    match id(e) {
        i if i < CHAMPS => K::Champion.code(),
        i if i < 22 => K::Tower.code(),
        i if i < 150 => K::Minion.code(),
        i if i < 190 => K::Jungle.code(),
        190 => K::Epic.code(),
        191 => K::Serpen.code(),
        _ => K::Summon.code(),
    }
}
unsafe extern "C" fn is_champ(p: *const c_void, e: E) -> bool { host(p); id(e) < CHAMPS }
unsafe extern "C" fn is_tower(p: *const c_void, e: E) -> bool { host(p); (CHAMPS..22).contains(&id(e)) }
unsafe extern "C" fn is_minion(p: *const c_void, e: E) -> bool { host(p); (22..150).contains(&id(e)) }
unsafe extern "C" fn visible(p: *const c_void, _: usize, i: usize) -> bool { host(p); i % 7 != 3 }
unsafe extern "C" fn pos(p: *const c_void, e: E, x: *mut u64, y: *mut u64) -> bool {
    let h = host(p);
    let i = id(e) as u64;
    let t = h.tick.get() as u64;
    *x = 60_000 + (i * 37_919 + t * 300) % 840_000;
    *y = 60_000 + (i * 71_003 + t * 170) % 840_000;
    true
}
unsafe extern "C" fn hp(p: *const c_void, _: E, hp: *mut usize, max: *mut usize) -> bool {
    host(p);
    *hp = 800;
    *max = 1000;
    true
}
unsafe extern "C" fn stat(p: *const c_void, _: E, out: *mut StatV1) -> bool {
    host(p);
    *out = StatV1 { attack: 80, move_speed: 1000, hp: 1000, ..StatV1::default() };
    true
}
unsafe extern "C" fn name(p: *const c_void, e: E, buf: *mut u8, cap: usize, len: *mut usize) -> bool {
    let h = host(p);
    let i = id(e);
    let n = if i < CHAMPS { h.names.borrow()[i] } else { "unit" };
    *len = n.len();
    if !buf.is_null() {
        std::ptr::copy_nonoverlapping(n.as_ptr(), buf, n.len().min(cap));
    }
    true
}
fn base_buffs(e: usize) -> usize { if e < CHAMPS { 12 } else { 1 } }
unsafe extern "C" fn buffs(p: *const c_void, e: E) -> usize {
    let h = host(p);
    base_buffs(id(e)) + h.added.borrow().get(&id(e)).map_or(0, |v| v.len())
}
unsafe extern "C" fn buff(p: *const c_void, e: E, i: usize, out: *mut BuffV1) -> bool {
    let h = host(p);
    let base = base_buffs(id(e));
    if i < base {
        *out = BuffV1::named(&format!("b{}:{}", id(e), i));
        return true;
    }
    match h.added.borrow().get(&id(e)).and_then(|v| v.get(i - base)) {
        Some(b) => { *out = *b; true }
        None => false,
    }
}
unsafe extern "C" fn ccs(p: *const c_void, _: E) -> usize { host(p); 0 }
unsafe extern "C" fn cc(p: *const c_void, _: E, _: usize, _: *mut CcV1) -> bool { host(p); false }
unsafe extern "C" fn champ_count(p: *const c_void) -> usize { host(p); CHAMPS }
unsafe extern "C" fn champ_at(p: *const c_void, i: usize) -> usize { host(p); i }
unsafe extern "C" fn player_valid(p: *const c_void, h: P) -> bool { host(p); h.id().is_some_and(|i| i < CHAMPS) }
unsafe extern "C" fn player_at(p: *const c_void, i: usize) -> P { host(p); P::from_id(i) }
unsafe extern "C" fn player_champion(p: *const c_void, h: P) -> E { host(p); E::from_id(h.id().unwrap()) }
unsafe extern "C" fn lane(p: *const c_void, h: P) -> u32 {
    host(p);
    [LaneV1::Top, LaneV1::Jungle, LaneV1::Mid, LaneV1::Bottom, LaneV1::Support][h.id().unwrap() % 5].code()
}
unsafe extern "C" fn add_buff(p: *mut c_void, e: usize, b: *const BuffV1) {
    let h = host(p);
    let b = *b;
    let mut added = h.added.borrow_mut();
    let list = added.entry(e).or_default();
    list.retain(|x| x.name() != b.name());
    list.push(b);
}
unsafe extern "C" fn remove_buff(p: *mut c_void, e: E, name: *const u8, len: usize) -> usize {
    let h = host(p);
    let name = std::str::from_utf8(std::slice::from_raw_parts(name, len)).unwrap_or("");
    let mut added = h.added.borrow_mut();
    let Some(list) = added.get_mut(&id(e)) else { return 0 };
    let before = list.len();
    list.retain(|x| x.name() != name);
    before - list.len()
}

unsafe extern "C" fn cooldown(p: *const c_void, e: E, _: u32, out: *mut mod_api_stable::CooldownV1) -> bool {
    host(p);
    if id(e) >= CHAMPS {
        return false;
    }
    (*out).remaining = 0;
    (*out).cooltime = 480;
    (*out).max_charges = 1;
    (*out).charges = 1;
    (*out).unlocked = true;
    true
}
unsafe extern "C" fn level(p: *const c_void, _: E) -> usize { host(p); 9 }
unsafe extern "C" fn shield(p: *const c_void, _: E) -> usize { host(p); 0 }
unsafe extern "C" fn effect(
    p: *mut c_void, _: *const u8, _: usize, _: usize, _: *const mod_api_stable::InputTargetV1, _: u64, _: u64, _: u64,
) -> bool {
    let h = host(p);
    h.effects.set(h.effects.get() + 1);
    true
}

fn with_sim<T>(h: &Host, f: impl FnOnce(&mut StableSim) -> T) -> T {
    unsafe {
        let mut t: SimVtableV1 = std::mem::zeroed();
        t.size = size_of::<SimVtableV1>();
        t.tick = Some(tick);
        t.seed = Some(seed);
        t.entity_count = Some(count);
        t.entity_at = Some(at);
        t.entity_is_valid = Some(valid);
        t.entity_is_alive = Some(alive);
        t.entity_is_targetable = Some(targetable);
        t.entity_team = Some(team);
        t.entity_kind = Some(kind);
        t.entity_is_champion = Some(is_champ);
        t.entity_is_tower = Some(is_tower);
        t.entity_is_minion = Some(is_minion);
        t.is_visible = Some(visible);
        t.entity_pos = Some(pos);
        t.entity_hp = Some(hp);
        t.entity_stat = Some(stat);
        t.entity_name = Some(name);
        t.entity_buff_count = Some(buffs);
        t.entity_buff_at = Some(buff);
        t.entity_cc_count = Some(ccs);
        t.entity_cc_at = Some(cc);
        t.champion_count = Some(champ_count);
        t.champion_id_at = Some(champ_at);
        t.player_count = Some(champ_count);
        t.player_at = Some(player_at);
        t.player_is_valid = Some(player_valid);
        t.player_champion = Some(player_champion);
        t.player_lane = Some(lane);
        t.add_buff = Some(add_buff);
        t.entity_remove_buff = Some(remove_buff);
        t.entity_cooldown = Some(cooldown);
        t.entity_level = Some(level);
        t.entity_shield = Some(shield);
        t.play_view_effect = Some(effect);
        let mut raw = SimCtxV1 { size: size_of::<SimCtxV1>(), sim: &t, frame: std::ptr::null(), state: h as *const Host as *mut c_void };
        let mut sim = StableSim::from_raw(&mut raw).unwrap();
        f(&mut sim)
    }
}

/// The input AI's per-player move path that runs on a plain `StableSim` (everything `WallAi::think`
/// does for a Move order except reading the input context), returning what each step answered.
fn player_path(sim: &StableSim, player: usize) -> Vec<Option<(i64, i64)>> {
    let tick = sim.tick();
    let owner = player;
    let dest = (480_000, 480_000);
    let mut out = Vec::new();
    // athlete_play builds its scene from every champion
    let scene = crate::champions_shared(sim);
    out.push(scene.iter().find(|c| c.id == owner).map(|c| (c.x, c.y)));
    out.push(crate::universal_ai::movement(sim, owner, dest));
    out.push(crate::athlete_ai::staging_move(sim, owner, dest));
    // the move pipeline's own snapshot, then map plans, smoke check and rally
    let all = crate::champions_shared(sim);
    if let Some(me) = all.iter().find(|c| c.id == owner) {
        out.push(crate::tactics::move_goal(sim, &all, me, None, dest));
        out.push(crate::valorant::clear_smoke(sim, &all, me, dest, tick));
        out.push(crate::steve::rally_for_bench(sim, &all, me.team, tick));
    }
    out
}

/// The map plans are one global table: tests that set or read them take turns.
static MARKS_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn shipped_marks() -> Vec<crate::tactics::Mark> {
    let text = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../mods/tfm2_custom_ai/tactics.txt")).unwrap_or_default();
    crate::tactics::parse(&text)
}

/// One full tick: every player's move path and the team planner's tick. Returns (host calls, answers).
fn one_tick(h: &Host, t: usize) -> (usize, Vec<Vec<Option<(i64, i64)>>>) {
    h.tick.set(t);
    h.calls.set(0);
    let answers = with_sim(h, |sim| {
        let heroes = crate::champions(sim);
        crate::universal_ai::tick(sim, &heroes);
        (0..CHAMPS).map(|p| player_path(sim, p)).collect()
    });
    (h.calls.get(), answers)
}

/// Host calls per step of one tick (printed by `tick_budget`).
fn breakdown(h: &Host, t: usize) -> Vec<(&'static str, usize)> {
    h.tick.set(t);
    let mut out: Vec<(&'static str, usize)> = Vec::new();
    let mut note = |name: &'static str, used: usize| match out.iter_mut().find(|o| o.0 == name) {
        Some(o) => o.1 += used,
        None => out.push((name, used)),
    };
    with_sim(h, |sim| {
        let dest = (480_000, 480_000);
        let mut mark = h.calls.get();
        let mut lap = |name: &'static str, note: &mut dyn FnMut(&'static str, usize)| {
            let now = h.calls.get();
            note(name, now - mark);
            mark = now;
        };
        let heroes = crate::champions(sim);
        lap("champions (match hook)", &mut note);
        crate::universal_ai::tick(sim, &heroes);
        lap("universal_ai::tick", &mut note);
        for p in 0..CHAMPS {
            crate::champions_shared(sim);
            lap("scene snapshot", &mut note);
            crate::universal_ai::movement(sim, p, dest);
            lap("universal_ai::movement", &mut note);
            crate::athlete_ai::staging_move(sim, p, dest);
            lap("staging_move", &mut note);
            let all = crate::champions_shared(sim);
            lap("pipeline snapshot", &mut note);
            if let Some(me) = all.iter().find(|c| c.id == p) {
                crate::tactics::move_goal(sim, &all, me, None, dest);
                lap("tactics::move_goal", &mut note);
                crate::valorant::clear_smoke(sim, &all, me, dest, t);
                lap("valorant::clear_smoke", &mut note);
                crate::steve::rally_for_bench(sim, &all, me.team, t);
                lap("rally", &mut note);
            }
        }
    });
    out
}

#[test]
#[ignore]
fn tick_budget() {
    let _marks = MARKS_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    {
        crate::tactics::set_marks_for_test(shipped_marks());
        let h = Host::new(0);
        for (name, calls) in breakdown(&h, 900) {
            println!("TICK_BUDGET re-plan tick, step {name}: {calls}");
        }
        for (name, calls) in breakdown(&h, 901) {
            println!("TICK_BUDGET ordinary tick, step {name}: {calls}");
        }
    }
    crate::tactics::set_marks_for_test(shipped_marks());
    let h = Host::new(0);
    let mut calls = Vec::new();
    let started = Instant::now();
    for t in 0..600 {
        calls.push(one_tick(&h, 1000 + t).0);
    }
    let elapsed = started.elapsed();
    calls.sort_unstable();
    println!(
        "TICK_BUDGET host calls per tick: median {} p95 {} max {}; {:.1} us per tick",
        calls[calls.len() / 2],
        calls[calls.len() * 95 / 100],
        calls[calls.len() - 1],
        elapsed.as_secs_f64() * 1e6 / 600.0
    );
    crate::tactics::set_marks_for_test(Vec::new());
}

#[test]
fn only_move_plans_turn_off_the_input_ai_shortcut() {
    let _marks = MARKS_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    // the shipped plan holds smoke and wall marks only: those belong to Omen's and Steve's passives
    let shipped = shipped_marks();
    crate::tactics::set_marks_for_test(shipped.clone());
    let shipped_only = crate::tactics::has_marks();
    let mut with_avoid = shipped;
    let avoid = crate::tactics::parse("avoid all always 0 3 100000 100000 100000 100000 20000 all");
    assert_eq!(avoid.len(), 1);
    with_avoid.extend(avoid);
    crate::tactics::set_marks_for_test(with_avoid);
    let with_move_plan = crate::tactics::has_marks();
    crate::tactics::set_marks_for_test(Vec::new());
    assert!(!shipped_only, "smoke/wall marks alone must keep the shortcut");
    assert!(with_move_plan, "an avoid area is a move plan");
}

#[test]
fn shared_snapshots_never_change_an_answer() {
    let _marks = MARKS_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    // the same 600 ticks with the per-tick snapshot on and off answer identically
    let h = Host::new(0);
    let run = |cached: bool| {
        crate::set_snapshot_cache_for_test(cached);
        let out: Vec<_> = (0..600).map(|t| one_tick(&h, 2000 + t).1).collect();
        crate::set_snapshot_cache_for_test(true);
        out
    };
    let off = run(false);
    let on = run(true);
    assert_eq!(on, off);
}

#[test]
fn a_snapshot_is_never_reused_across_ticks_or_sims() {
    let h = Host::new(5);
    let a = with_sim(&h, |sim| crate::champions_shared(sim));
    h.tick.set(6);
    let b = with_sim(&h, |sim| crate::champions_shared(sim));
    assert_ne!(a.iter().map(|c| (c.x, c.y)).collect::<Vec<_>>(), b.iter().map(|c| (c.x, c.y)).collect::<Vec<_>>());
    // another sim instance at the same tick builds its own
    let other = Host::new(6);
    let before = other.calls.get();
    with_sim(&other, |sim| crate::champions_shared(sim));
    assert!(other.calls.get() > before + CHAMPS * 5, "a different sim must not reuse the first sim's snapshot");
}

/// Round 145 (the "slow since Raze" report): one tick of the whole mock match plus every Raze / Contender passive in
/// it. Returns (host calls, effects played, Raze/Contender host calls).
fn kit_tick(h: &Host, kits: &mut [(usize, Kit)], t: usize) -> (usize, usize, usize) {
    let (calls, _) = one_tick(h, t);
    let before = h.calls.get();
    h.effects.set(0);
    with_sim(h, |sim| {
        for (id, kit) in kits.iter_mut() {
            match kit {
                Kit::Raze(r) => mod_api_stable::StablePassive::on_update(r.as_mut(), sim, 0, *id, *id),
                Kit::Contender(c) => mod_api_stable::StablePassive::on_update(c.as_mut(), sim, 0, *id, *id),
            }
        }
    });
    (calls + h.calls.get() - before, h.effects.get(), h.calls.get() - before)
}
enum Kit {
    Raze(Box<crate::raze::Raze>),
    Contender(Box<crate::contender::Contender>),
}
fn kits(h: &Host, raze: &[usize], contender: &[usize]) -> Vec<(usize, Kit)> {
    let mut names = NAMES;
    let mut out = Vec::new();
    for &i in raze {
        names[i] = crate::raze::ID;
        out.push((i, Kit::Raze(Box::default())));
    }
    for &i in contender {
        names[i] = "tfm2_custom_contender";
        out.push((i, Kit::Contender(Box::default())));
    }
    *h.names.borrow_mut() = names;
    with_sim(h, |sim| {
        for (id, kit) in out.iter_mut() {
            match kit {
                Kit::Raze(r) => mod_api_stable::StablePassive::on_spawn(r.as_mut(), sim, *id, *id),
                Kit::Contender(c) => mod_api_stable::StablePassive::on_spawn(c.as_mut(), sim, *id, *id),
            }
        }
    });
    out
}

#[test]
#[ignore]
fn raze_budget() {
    let _marks = MARKS_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    crate::tactics::set_marks_for_test(shipped_marks());
    crate::adapter_perf::FORCE.store(true, std::sync::atomic::Ordering::Relaxed);
    for (label, raze, contender) in [
        ("no Raze", &[][..], &[][..]),
        ("one Raze", &[3][..], &[][..]),
        ("Raze on each team", &[3, 8][..], &[][..]),
        ("one Contender", &[][..], &[4][..]),
    ] {
        let h = Host::new(0);
        let mut k = kits(&h, raze, contender);
        let (mut calls, mut fx, mut own) = (Vec::new(), Vec::new(), Vec::new());
        let started = Instant::now();
        for t in 0..600 {
            let (c, e, o) = kit_tick(&h, &mut k, 3000 + t);
            calls.push(c);
            fx.push(e);
            own.push(o);
        }
        let us = started.elapsed().as_secs_f64() * 1e6 / 600.0;
        for (id, kit) in &k {
            if let Kit::Raze(r) = kit {
                println!("RAZE_BUDGET {label}: Raze {id} median ns per phase {:?}", r.perf_medians());
            }
        }
        for v in [&mut calls, &mut fx, &mut own] {
            v.sort_unstable();
        }
        let q = |v: &Vec<usize>| (v[v.len() / 2], v[v.len() * 95 / 100], v[v.len() - 1]);
        println!(
            "RAZE_BUDGET {label}: host calls/tick {:?} (kits {:?}); effects/tick {:?}; {us:.1} us/tick",
            q(&calls),
            q(&own),
            q(&fx)
        );
    }
    crate::tactics::set_marks_for_test(Vec::new());
}

#[test]
fn raze_frames_match() {
    // Round 145: reading each unit once (and its name once while it lives) gives Raze the same frames, so the same
    // decisions, buffs and effects, over 600 ticks of a moving mock match with a Raze on each team.
    let _marks = MARKS_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let run = |cached: bool| {
        crate::raze::set_frame_cache_for_test(cached);
        let h = Host::new(0);
        let mut k = kits(&h, &[3, 8], &[]);
        let mut out = Vec::new();
        for t in 0..600 {
            let (_, fx, _) = kit_tick(&h, &mut k, 4000 + t);
            let cores: Vec<String> = k
                .iter()
                .filter_map(|(_, kit)| match kit {
                    Kit::Raze(r) => serde_json::to_string(&r.core).ok(),
                    Kit::Contender(_) => None,
                })
                .collect();
            let mut buffs: Vec<String> = h
                .added
                .borrow()
                .iter()
                .flat_map(|(e, v)| v.iter().map(move |b| format!("{e}:{}", b.name())))
                .collect();
            buffs.sort();
            out.push((cores, fx, buffs));
        }
        crate::raze::set_frame_cache_for_test(true);
        out
    };
    let off = run(false);
    let on = run(true);
    assert_eq!(off.len(), on.len());
    for (t, (a, b)) in off.iter().zip(&on).enumerate() {
        assert_eq!(a, b, "tick {t}");
    }
}

#[test]
fn rings_cost_nothing_where_no_ring_opened() {
    // Round 145: the planners, farming and the duel guide ask for rings several times per player per tick; a sim in
    // which no ring was ever opened answers without a host call (it used to read every champion's buffs each time).
    let h = Host::new(10);
    let before = h.calls.get();
    let rings = with_sim(&h, |sim| crate::contender_arena::rings(sim));
    assert!(rings.is_empty());
    assert_eq!(h.calls.get(), before);
}
