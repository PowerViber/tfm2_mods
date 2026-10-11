# Coder round 123 verification

Native **0.10.36**, champion mod **0.2.36**, dependency **>=0.10.36**, all dates **2026-10-11**. Base: main **1881b2e**, with PR #26 merged. Branch: `codex/coder-half-damage-saved-code`. [PR #27](https://github.com/PowerViber/tfm2_mods/pull/27) contains all commits for this revision. Local HEAD, remote branch and PR head are checked to match after the final push; the worktree is clean.

The user requested another 50% reduction to every damaging skill and a 90% saved-memory reduction while preserving coding/execution speed. The attached `coder-verification.md` is the verification reference; its older branch/version examples are historical context.

## Result and scope

The 50% multiplier applies after the previous release's burst and replay factors. A previous 100 damage becomes 50 on a fresh cast, then 5 if the code was loaded from saved storage. Integer amounts round down at the same intermediate steps as native code. Small loaded packets may round to zero; no minimum-one floor bypasses the reductions.

Saving a fresh compile and loading a saved copy are distinct. The original newly written daemon keeps the ordinary 50% output; a daemon loaded from storage retains the additional 90% penalty throughout its lifetime. Support from either stays full strength. A waiting loaded script preserves its provenance until its trigger fires. Delayed packets/walls snapshot their already-reduced amount; drones/tethers retain the original loaded/replay state while still using live AP and rig speed. Saved meta functions also penalize generated hits, and saved Kubernetes penalizes its replica while leaving a freshly written primary cast ordinary. Both source and parent being loaded applies the penalty once.

Fresh `zero_day` now deals 6% max HP; `binary_search` deals 12.5% remaining HP below its existing 20% trigger; `honeypot` reflects 10%. Loaded values are 0.6%, 1.25%, and 1%, respectively, before any existing replay factors and integer rounding. `kill9` below 8% HP uses half remaining HP (5% when loaded), replacing the old hp+max_hp lethal padding; replicas still use its ordinary-hit fallback. Thus the former guaranteed kill receives a stronger practical nerf than simply halving the padded raw number, which would still kill.

Typing/reload/compile cadence, cooldowns, packet counts and schedules, AI/RNG draws, rig behavior, support healing/shields/CC, attack-speed/attack buffs, damage-taken marks and ordinary basic attacks are unchanged. No art or animation changes. Diagnostics append `saved_reload=0/1` to scripted events and `?` to unattributed engine kill credit. The Code Lab mirrors damage formulas/provenance and its explanatory text; the arena remains a reduced lane model, not a full native match simulator.

## Meaningful damage coverage

Four new host-API tests visit all 100 functions at replay scales 0, 50, 60 and 100, requiring 30 damaging/meta/persistent routes to emit. Captured loaded hits must have exactly one tenth of their fresh counterpart, matching delivery ticks/types, with support amounts/durations preserved and saved context restored after calls. Separate tests exercise actual saved `Typing::reload` versus a newly saved compile, a pending loaded `zero_day`, and a loaded Kubernetes replica alongside unchanged healing.

Existing host tests now assert half of fixed previous-release damage baselines across AP and rig tiers, unchanged packet/daemon schedules, fixed persistent/percentage damage, actual finisher HP remaining, reflection reduction, Quantum success/miss outcomes, sign-flipped healing, live-AP delayed attribution, and presimulation behavior without extra boosts/RNG. Stock Python `ping` at AP 120 goes **63 -> 31**, then **3** when loaded. The finisher fixture at 79/1000 HP loses 39 and remains at 40 HP.

## Changed files

- Claude outputs/coder/coder_data.py
- docs/coder-half-damage-verification.md
- docs/playtest-notes.md
- editor/coderlab.js
- mods/tfm2_custom/mod.mod_info
- mods/tfm2_custom/text/champion.i18n (generated)
- mods/tfm2_custom_ai/mod.mod_info
- mods/tfm2_custom_ai/tfm2_custom_ai.dll (rebuilt)
- native/tfm2_custom_ai/mod.mod_info
- native/tfm2_custom_ai/src/coder.rs
- native/tfm2_custom_ai/src/coder_damage_tests.rs
- native/tfm2_custom_ai/src/coder_trace.rs
- native/tfm2_custom_ai/src/lib.rs
- tools/verify_coder.py

## Environment and canonical regeneration

Commands use the existing prepared environment via `source /workspace/.tfm2-env/env.sh`. Linux release builds run with four jobs. Windows linking uses `/workspace/.tfm2-env/mingw/usr/bin/x86_64-w64-mingw32-gcc-posix`, set by that environment file. No toolchain reinstall or manager-source change was needed.

Canonical description regeneration, run last among asset/data generators:

```bash
cd 'Claude outputs/coder'
python coder_data.py
```

```text
tfm2_custom_coder 5321 effects, 149 buffs
```

Only the generated description changed; the champion view list, sheets, function code, language tables and typing vectors remain byte-identical. No vectors were regenerated.

## Required automated checks

All six attached-checklist command groups exited **0**, with **0 warnings** in native/manager build/test logs.

1. Native Linux release build:

```bash
cargo build --offline --release -j4 --manifest-path native/tfm2_custom_ai/Cargo.toml
```

```text
Compiling tfm2_custom_ai v0.1.0 (/workspace/tfm2_mods/native/tfm2_custom_ai)
    Finished `release` profile [optimized] target(s) in 17.07s
```

2. Full native release tests:

```bash
cargo test --offline --release -j4 --manifest-path native/tfm2_custom_ai/Cargo.toml -- --nocapture
```

```text
test coder::damage_tests::a_waiting_saved_script_keeps_its_penalty_until_the_trigger_fires ... ok
test coder::damage_tests::diagnostic_presim_suppression_preserves_damage_and_does_not_consume_extra_boosts_or_rng ... ok
test coder::damage_tests::daemon_and_cooldown_schedules_are_unchanged ... ok
test coder::damage_tests::diagnostics_attribute_chain_packets_and_live_scaled_persistent_hits ... ok
test coder::damage_tests::diagnostics_preserve_support_and_report_nonlethal_direct_hp_and_engine_kill_credit ... ok
test coder::damage_tests::packets_keep_their_count_schedule_and_are_not_nerfed_twice ... ok
test coder::damage_tests::percentage_attacks_and_execute_boundaries_cannot_bypass_replay_nerfs ... ok
test coder::damage_tests::immediate_damage_and_burst_are_reduced_across_rigs ... ok
test coder::damage_tests::replay_damage_retains_both_reductions_without_halving_support_and_quantum_stays_a_coin_flip ... ok
test coder::damage_tests::saved_kubernetes_only_penalizes_its_replica_and_support_stays_full_strength ... ok
test coder::damage_tests::saving_a_fresh_compile_and_loading_a_saved_compile_have_distinct_damage ... ok
test coder::damage_tests::replays_retain_damage_scaling_in_drones_tethers_and_walls ... ok
test coder::damage_tests::support_and_sign_flipped_healing_keep_their_original_strength ... ok
test coder::damage_tests::loaded_code_reduces_every_emitted_hit_and_preserves_support_and_delivery_times ... ok
test coder::tests::choosing_is_cheap ... ok
test coder::tests::every_view_name_exists ... ok
test coder::tests::he_doesnt_spam_the_same_function ... ok
test coder::tests::lab_vectors ... ok
test coder::tests::scripts_keep_him_writing ... ok
test coder::tests::the_data_center_has_its_risks ... ok
test version_tests::every_version_agrees ... ok
test result: ok. 176 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.63s
```

The one pre-existing ignored test is `scribble::real_data::real_pending`; no test was added to the ignore list or removed.

3. Native/lab arithmetic, typing and asset parity:

```bash
python tools/verify_coder.py --local
```

```text
Code lab: tables, languages, functions, models and hardware match coder.rs; 558 runs reproduced exactly
Verified 5471 Coder views over 17 sheets (each <= 2048), the rig layers, the top-rank effects, crests and every name coder.rs builds
```

Damage arithmetic now checks 160 combinations (four replay scales, four amounts, five function/burst categories, two saved states), with ordinary, percentage and persistent output checked for each. All existing native/lab tables still match and the 558 typing runs reproduce exactly.

4. Manager release tests:

```bash
cargo test --offline --release -j4 --manifest-path tools/manager/Cargo.toml
```

```text
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

5. JavaScript syntax, self-test and every rank:

```bash
node --check editor/coderlab.js
node -e 'const l=require("./editor/coderlab.js"); console.log(l.selfTest()); for (const [r,p] of l.LADDER) l.simulateSkirmish(r,p,1);'
```

```text
true
```

No syntax error or rank simulation crash.

6. Windows release build and shipped DLL:

```bash
cargo build --offline --release --target x86_64-pc-windows-gnu -j4 --manifest-path native/tfm2_custom_ai/Cargo.toml
cp native/tfm2_custom_ai/target/x86_64-pc-windows-gnu/release/tfm2_custom_ai.dll mods/tfm2_custom_ai/tfm2_custom_ai.dll
sha256sum native/tfm2_custom_ai/target/x86_64-pc-windows-gnu/release/tfm2_custom_ai.dll mods/tfm2_custom_ai/tfm2_custom_ai.dll
```

```text
Compiling tfm2_custom_ai v0.1.0 (/workspace/tfm2_mods/native/tfm2_custom_ai)
    Finished `release` profile [optimized] target(s) in 17.01s
fb208508de5941cb14c7c07971c4c5c13c24ff93905fcbf05ca9c18474afeedf  native/tfm2_custom_ai/target/x86_64-pc-windows-gnu/release/tfm2_custom_ai.dll
fb208508de5941cb14c7c07971c4c5c13c24ff93905fcbf05ca9c18474afeedf  mods/tfm2_custom_ai/tfm2_custom_ai.dll
```

Build and shipped DLL are also byte-identical. Both native metadata files, VERSION, champion version, dependency and dates agree; `every_version_agrees` passed. Manager EXE was **not rebuilt** because manager source is unchanged. Its unchanged SHA-256 is `01780422f34e0b040eb3e299ee0f6a9f6f1227462728f402aa0f093383852e91`.

## Editor checklist

The prepared local editor server at port 19366 serves the changed repository. Headless Chromium:

```bash
NODE_PATH=/workspace/.tfm2-env/browser/node_modules DB_EDITOR_URL=http://127.0.0.1:19366/ node tools/verify_coder_browser.cjs
```

```text
PASS: browser parity, 100 rows, language filtering, picker/rewrite, 4 rank arenas, 10-run comparison, no page errors/NaN.
[{"rank":"Script Kiddie","shipped":0.5,"distinct":0.5,"dps":0.1},{"rank":"Intern","shipped":1.5,"distinct":1.5,"dps":0.8766666666666667},{"rank":"Junior","shipped":3.6,"distinct":3.6,"dps":4.666666666666667},{"rank":"Developer","shipped":6.1,"distinct":6.1,"dps":29.259999999999998},{"rank":"Senior","shipped":8.6,"distinct":8.6,"dps":28.11},{"rank":"Staff","shipped":9.2,"distinct":9.2,"dps":18.083333333333332},{"rank":"Architect","shipped":11.8,"distinct":11.8,"dps":28.533333333333335},{"rank":"Root #10","shipped":13,"distinct":13,"dps":43.1},{"rank":"Root #1 Zero-Day","shipped":14.8,"distinct":14.8,"dps":49.53333333333333}]
```

All nine comparison rows are finite; shipped and distinct-function counts increase with rank. The table has 100 function rows; kubernetes/regex/zero_day selection, available-language filters and rewrite work. Script Kiddie, Developer, Architect and Root #1 arenas run without page errors. NATIVE PARITY passes. Damage-per-second need not grow monotonically in this reduced lane model because ranks choose different support functions and have different survival/equipment outcomes.

## Remaining real-game checks

Round 123 in `playtest-notes.md` records the request, implementation, automated results and manual follow-up. Close the game, Update/Check, then confirm native 0.10.36 and champion 0.2.36 load without dependency warnings. Replay xartE versus Binary Tigers game 1 with comparable stats and equipment; compare scripted raw damage and observed HP loss in `coder_damage_log.txt`, using `saved_reload` to identify loaded effects. Confirm typing/reload cadence, support and delayed effects after subsequent casts. Inspect notebook readability and crowded-match FPS.

Actual installation/game loading, armor/items/buff procs/collisions and real-match performance cannot be verified in this cloud workspace. The emitted damage reductions do not guarantee that every possible combined hit or engine proc is nonlethal. Neither the old IDE art nor the Science Lab/mastery UI is altered by this damage revision.
