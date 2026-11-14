//! Optimization pass: every native passive on the glue mock host in a crowded 5v5 lane, timed per tick.
//! `cargo test --release whole_roster_tick_cost -- --ignored --nocapture`. The mock host is slower than the game's,
//! so read the numbers relative to each other.
use crate::glue_tests::{with_sim, Host, Unit};
use mod_api_stable::{EntityKindV1 as K, StablePassive};

fn roster() -> Vec<(&'static str, &'static str, Box<dyn StablePassive>)> {
    vec![
        ("tfm2_custom_akane", r#"{"art_v1":1}"#, Box::new(crate::akane::Akane::default())),
        ("tfm2_custom_kicker", "{}", Box::new(crate::kicker::Kicker::default())),
        ("tfm2_custom_kagami", "{}", Box::new(crate::kagami::Kagami::default())),
        ("tfm2_custom_thalassa", r#"{"r165_art":1}"#, Box::new(crate::thalassa::Thalassa::default())),
        ("tfm2_custom_wright", r#"{"r165_art":1}"#, Box::new(crate::wright::Wright::default())),
        ("tfm2_custom_kratos", r#"{"weapon_layer":1}"#, Box::new(crate::kratos::Kratos::default())),
        ("tfm2_custom_grimhald", r#"{"weapon_layer":1}"#, Box::new(crate::grimhald::Grimhald::default())),
        ("tfm2_custom_rally", "{}", Box::new(crate::rally::Rally::default())),
        ("tfm2_custom_contender", "{}", Box::new(crate::contender::Contender::default())),
        ("tfm2_blockcraft_steve", "{}", Box::new(crate::steve_craft::Steve::default())),
        ("tfm2_custom_gojo", "{}", Box::new(crate::gojo_v2::Gojo::default())),
        ("tfm2_ultrakill_v1", "{}", Box::new(crate::batch2::V1::default())),
        ("tfm2_starwars_vader", "{}", Box::new(crate::batch2::Vader::default())),
        ("tfm2_cyberpunk_david", "{}", Box::new(crate::batch2::David::default())),
        ("tfm2_jojo_dio", "{}", Box::new(crate::batch2::Dio::reworked())),
        ("tfm2_valorant_omen", "{}", Box::new(crate::valorant::Omen::reworked())),
        ("tfm2_valorant_raze", "{}", Box::new(crate::raze::Raze::default())),
        ("tfm2_toon_scribble", "{}", Box::new(crate::scribble::Scribble::default())),
        ("tfm2_levi_levi", "{}", Box::new(crate::levi::Levi::default())),
        ("tfm2_isliid_emperor", "{}", Box::new(crate::isliid::Isliid::default())),
        ("tfm2_gundam_aegis_zero", r#"{"balance_hp_r140":1}"#, Box::new(crate::gundam::Gundam::default())),
        ("tfm2_custom_unified_theory", r#"{"balance_hp_r140":1}"#, Box::new(crate::unified_theory::UnifiedTheory::default())),
        ("tfm2_frieren_frieren", "{}", Box::new(crate::frieren::Frieren::default())),
        ("tfm2_custom_coder", r#"{"balance_creative_v1":1}"#, Box::new(crate::coder::Coder::default())),
        ("tfm2_custom_vesper", "{}", Box::new(crate::vesper::Vesper::default())),
        ("tfm2_custom_magnus", "{}", Box::new(crate::magnus::Magnus::default())),
        ("tfm2_custom_minato", "{}", Box::new(crate::minato_body::MinatoBody::default())),
    ]
}

/// A walled map like the game's 30x30 grid: the outer ring, river banks and jungle pillars (about 170 wall cells),
/// so wall checks cost what they do in a match.
fn walled_map() -> std::sync::MutexGuard<'static, ()> {
    let guard = crate::walls::TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let n = crate::walls::N;
    let mut cells = vec![false; (n * n) as usize];
    for y in 0..n {
        for x in 0..n {
            let ring = x == 0 || y == 0 || x == n - 1 || y == n - 1;
            let pillar = (x % 5 == 2 && y % 5 == 2) || (x % 7 == 3 && y % 6 == 4);
            let bank = (x + y == 29 || x + y == 31) && (x % 4 != 0);
            if ring || pillar || bank {
                cells[(y * n + x) as usize] = true;
            }
        }
    }
    crate::walls::set(cells);
    guard
}

fn lane(name: &'static str) -> Vec<Unit> {
    let (x, y) = crate::glue_tests::open_spot(40_000);
    let c = |team, dx: i64, dy: i64, name| Unit { kind: K::Champion, team, pos: ((x as i64 + dx) as u64, (y as i64 + dy) as u64), hp: 2000, name, targetable: true };
    let mut u = vec![c(0, 0, 0, name)];
    for i in 0..4 { u.push(c(0, -8_000, i * 6_000 - 9_000, "knight")); }
    for i in 0..5 { u.push(c(1, 40_000 + i * 4_000, i * 5_000 - 10_000, "archer")); }
    for i in 0..12 {
        u.push(Unit { kind: K::Minion, team: (i % 2) as usize, pos: ((x as i64 + 15_000 + (i % 6) * 4_000) as u64, (y as i64 + (i / 6) * 8_000) as u64), hp: 500, name: "minion", targetable: true });
    }
    u.push(Unit { kind: K::Tower, team: 1, pos: (x + 120_000, y), hp: 5000, name: "tower", targetable: true });
    u
}

#[test]
#[ignore]
fn whole_roster_tick_cost() {
    let ticks: usize = std::env::var("BENCH_TICKS").ok().and_then(|v| v.parse().ok()).unwrap_or(1_800);
    let only = std::env::var("BENCH_ONLY").unwrap_or_default();
    let _map = walled_map();
    let mut rows = vec![];
    for (name, params, mut p) in roster() {
        if !only.is_empty() && !name.contains(only.as_str()) { continue; }
        let host = Host::new(lane(name));
        p.configure(params);
        with_sim(&host, |sim| p.on_spawn(sim, 0, 0));
        let t0 = std::time::Instant::now();
        for t in 1..=ticks {
            host.borrow_mut().tick = t;
            with_sim(&host, |sim| p.on_update(sim, t as u64, 0, 0));
        }
        let ns = t0.elapsed().as_nanos() / ticks as u128;
        let fx = host.borrow().view_effects.len();
        rows.push((ns, name, fx));
    }
    rows.sort_by(|a, b| b.0.cmp(&a.0));
    for (ns, name, fx) in rows { println!("BENCH {name:28} {ns:>9} ns/tick  fx {fx}"); }
}

/// Ten modded champions in one 5v5 (a background match during Proceed), each passive ticking every tick.
fn mixed_match(names: [&'static str; 10]) -> (u128, usize) {
    let ticks: usize = std::env::var("BENCH_TICKS").ok().and_then(|v| v.parse().ok()).unwrap_or(1_800);
    let all = roster();
    let (x, y) = crate::glue_tests::open_spot(40_000);
    let mut units = vec![];
    let mut passives = vec![];
    for (i, n) in names.iter().enumerate() {
        let team = i / 5;
        let dx = if team == 0 { 0 } else { 40_000 } + (i % 5) as i64 * 3_000;
        units.push(Unit { kind: K::Champion, team, pos: ((x as i64 + dx) as u64, (y as i64 + (i % 5) as i64 * 6_000 - 12_000) as u64), hp: 2000, name: n, targetable: true });
    }
    for i in 0..12 {
        units.push(Unit { kind: K::Minion, team: (i % 2) as usize, pos: ((x as i64 + 15_000 + (i % 6) * 4_000) as u64, (y as i64 + (i / 6) * 8_000) as u64), hp: 500, name: "minion", targetable: true });
    }
    units.push(Unit { kind: K::Tower, team: 1, pos: (x + 120_000, y), hp: 5000, name: "tower", targetable: true });
    let host = Host::new(units);
    for n in names {
        let (_, params, _) = all.iter().find(|r| r.0 == n).unwrap();
        let mut p = roster().into_iter().find(|r| r.0 == n).unwrap().2;
        p.configure(params);
        passives.push(p);
    }
    for (i, p) in passives.iter_mut().enumerate() { with_sim(&host, |sim| p.on_spawn(sim, i, i)); }
    let t0 = std::time::Instant::now();
    for t in 1..=ticks {
        host.borrow_mut().tick = t;
        for (i, p) in passives.iter_mut().enumerate() { with_sim(&host, |sim| p.on_update(sim, t as u64, i, i)); }
    }
    let fx = host.borrow().view_effects.len();
    (t0.elapsed().as_nanos() / ticks as u128, fx)
}

pub(crate) const MIXED: [[&str; 10]; 3] = [
    ["tfm2_custom_akane", "tfm2_custom_thalassa", "tfm2_custom_wright", "tfm2_custom_kratos", "tfm2_custom_grimhald",
     "tfm2_valorant_raze", "tfm2_custom_rally", "tfm2_custom_gojo", "tfm2_blockcraft_steve", "tfm2_custom_magnus"],
    ["tfm2_custom_vesper", "tfm2_toon_scribble", "tfm2_custom_unified_theory", "tfm2_custom_contender", "tfm2_frieren_frieren",
     "tfm2_custom_coder", "tfm2_valorant_omen", "tfm2_isliid_emperor", "tfm2_jojo_dio", "tfm2_levi_levi"],
    ["tfm2_cyberpunk_david", "tfm2_gundam_aegis_zero", "tfm2_starwars_vader", "tfm2_ultrakill_v1", "tfm2_custom_minato",
     "tfm2_custom_akane", "tfm2_custom_thalassa", "tfm2_valorant_raze", "tfm2_custom_rally", "tfm2_custom_magnus"],
];

#[test]
#[ignore]
fn mixed_match_tick_cost() {
    let _map = walled_map();
    let only: Option<usize> = std::env::var("BENCH_MIXED").ok().and_then(|v| v.parse().ok());
    for (i, m) in MIXED.iter().enumerate() {
        if only.is_some_and(|o| o != i) { continue; }
        let (ns, fx) = mixed_match(*m);
        println!("BENCH mixed match {i}: {ns:>9} ns/tick  fx {fx}");
    }
}
