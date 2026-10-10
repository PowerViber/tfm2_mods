# One Science Lab — round 122 verification

Native **0.10.35**, champion **0.2.35**, dependency **>=0.10.35**, dates **2026-10-11**. Both native metadata files match `VERSION`, and `every_version_agrees` passes. Branch: `codex/science-lab-unified-ui`, based on main **5531753** after PR #25 merged. This follow-up uses a new PR.

## Result

Removed the separate Skill viewer tab and champion buttons. Science Lab now holds the original skill animations and charged experiment tests together. Header, arena and Skill Lab links all open the same lab; individual skill rows select the requested recipe there.

The embedded lab imports the **actual common editor stylesheet** and uses Coder/Isliid's `il-shell`, `il-head`, `il-controls`, `il-main`, `il-actions`, `il-result` and `il-side` layout. It has the same 1250 px shell, heading size and body font, checked against both labs in the browser. A compact control bar selects scientist, rank, podium position, skill, animation and preview. The large canvas and actions sit beside token charges, recipe details and imbalance feedback. Expandable panels hold transformations, all skills, the 50-template builder and ten-trial comparisons.

The Preview control switches between **Skill animation** and **Trajectory study** within Science Lab. Activate preview and Reserve & run automatically select the study. All 75 original eight-frame effects, eight ranks, Top 10 appearances, supported travel/field art, transformations and completion remain available. Charge equations are exact; trajectories and notebook comparisons are schematic rather than a browser recreation of the native combat engine.

[Combined lab preview](unified-theory-science-lab-ui.png) · [Trajectory study](unified-theory-science-lab-study.png) · [Updated guide](unified-theory-guide.md).

## State and animation cost

One persistent frame retains the notebook and pause setting across tab changes. Exactly one scientist preview runs at a time. Switching previews cancels the inactive loop; leaving the lab stops both. No duplicate native sprite/emitter is added. Controls and subject-specific Top 10 titles update while the art loop is stopped, without restarting that loop. Rank/form/skill selections still carry back to the arena, and its three current appearance layers replace cleanly.

The separate test/viewer view state and navigation were removed. The mastery list, randomizer, native file prefix and standalone Science Lab remain available. Native scientific and Coder gameplay, damage, typing/execution speed, cooldowns, art and vectors are unchanged. Native source changes only the required release VERSION. Manager source/EXE unchanged. Browser loop checks do not measure actual game FPS.

## Verification

The user's request determines the UI change. The attached `coder-verification.md` determines the required shared-DLL/editor/release checks; its historical versions and PR examples are reference material. Generated assets were not edited by hand. Both builds and both suites contain **0 errors and 0 warnings**; `git diff --check` passes. Desktop previews were visually reviewed; mobile checks use a 390 px viewport.

### Linux build and full native suite

```sh
source /workspace/.tfm2-env/env.sh
cargo build --offline --release -j4 --manifest-path native/tfm2_custom_ai/Cargo.toml
cargo test --offline --release -j4 --manifest-path native/tfm2_custom_ai/Cargo.toml -- --nocapture
```

Build output:

```text
Compiling tfm2_custom_ai v0.1.0 (/workspace/tfm2_mods/native/tfm2_custom_ai)
    Finished `release` profile [optimized] target(s) in 19.02s
```

Relevant suite output, including the checklist's named tests and the full result:

```text
   Compiling tfm2_custom_ai v0.1.0 (/workspace/tfm2_mods/native/tfm2_custom_ai)
    Finished `release` profile [optimized] target(s) in 47.13s
test coder::tests::choosing_is_cheap ... ok
test coder::tests::every_view_name_exists ... ok
test coder::tests::he_doesnt_spam_the_same_function ... ok
test coder::tests::lab_vectors ... ok
test coder::tests::scripts_keep_him_writing ... ok
test coder::tests::the_data_center_has_its_risks ... ok
test scribble::real_data::real_pending ... ignored
Cosmic crowded scene: 594 starts over 7 s, peak 30 live temporary animations, max 6 starts/update; persistent loops added once
test version_tests::every_version_agrees ... ok
test result: ok. 172 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 1.19s
```

The sole ignored test is the pre-existing `scribble::real_data::real_pending` host fixture.

### Coder verification

```sh
python tools/verify_coder.py --local
```

```text
Code lab: tables, languages, functions, models and hardware match coder.rs; 558 runs reproduced exactly
Verified 5471 Coder views over 17 sheets (each <= 2048), the rig layers, the top-rank effects, crests and every name coder.rs builds
```

### Manager suite

```sh
cargo test --offline --release -j4 --manifest-path tools/manager/Cargo.toml
```

```text
Finished `release` profile [optimized] target(s) in 0.00s
     Running unittests src/main.rs (tools/manager/target/release/deps/tfm2_manager-756fc6bf9fcfdf44)

running 13 tests
test core::tests::build_outputs_reset_before_pull ... ok
test core::tests::champion_data_errors_name_the_bad_tag ... ok
test core::tests::check_reports_mismatch ... ok
test core::tests::log_scan_finds_load_errors ... ok
test core::tests::perf_log_is_read ... ok
test core::tests::repo_root_found_from_exe_dir ... ok
test core::tests::install_copies_and_keeps_plans ... ok
test core::tests::mods_json_enable ... ok
test core::tests::native_version_meets_the_champions_requirement ... ok
test core::tests::stale_duplicates_go_to_backup ... ok
test core::tests::stamps_are_dates ... ok
test core::tests::vdf_and_manifest_parse ... ok
test core::tests::the_repos_champions_all_load ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

### JavaScript syntax, self-test and every Coder rank

```sh
node --check editor/coderlab.js
node -e 'const l=require("./editor/coderlab.js");console.log(l.selfTest());for(const [r,p]of l.LADDER)l.simulateSkirmish(r,p,1);'
```

```text
true
```

No crash at any rank. `node --check` passes for all changed JS files (`app`, `skills`, `skilltest`, `server`, `science-editor`, `science-embed`, `science-art`) and `tools/verify_science_editor_browser.cjs`; syntax success emits no output.

### Windows DLL, copy and SHA256

```sh
cargo build --offline --release --target x86_64-pc-windows-gnu -j4 --manifest-path native/tfm2_custom_ai/Cargo.toml
cp native/tfm2_custom_ai/target/x86_64-pc-windows-gnu/release/tfm2_custom_ai.dll mods/tfm2_custom_ai/tfm2_custom_ai.dll
sha256sum native/tfm2_custom_ai/target/x86_64-pc-windows-gnu/release/tfm2_custom_ai.dll mods/tfm2_custom_ai/tfm2_custom_ai.dll 'TFM2 Mod Manager.exe'
```

The sourced setup supplies the MinGW linker through `CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER`.

```text
Compiling tfm2_custom_ai v0.1.0 (/workspace/tfm2_mods/native/tfm2_custom_ai)
    Finished `release` profile [optimized] target(s) in 16.30s
e41e64e2e1accb979309627d58489a86e39d71f8027d44ccd8c45270ac558da3  native/tfm2_custom_ai/target/x86_64-pc-windows-gnu/release/tfm2_custom_ai.dll
e41e64e2e1accb979309627d58489a86e39d71f8027d44ccd8c45270ac558da3  mods/tfm2_custom_ai/tfm2_custom_ai.dll
01780422f34e0b040eb3e299ee0f6a9f6f1227462728f402aa0f093383852e91  TFM2 Mod Manager.exe
```

Built and shipped DLL hashes match. The manager EXE was not rebuilt because its source did not change; its hash matches the prior release.

### Scientist asset regeneration and exact math

```sh
python tools/verify_unified_theory.py
```

```text
375 exact native charge vectors; all 8 ranks deterministic
75 unique skills; 50 bounded recipes; generated files current
Cosmic Unified Experiment: 17 badges, 24 outfits, 300 eight-frame casts, 44 eight-frame fields, 448 eight-frame packet loops with shared pair aliases
Original body, three personas, notebooks, meter and mastery art generated
tfm2_custom_unified_theory 3363 effects 186 persona / mastery overlays
Verified 3550 views over 36 VFX sheets; original body <=2048; notebook lifetimes; regeneration byte-identical.
Art: 300 casts, 44 fields and 448 directional packet loops have eight distinct frames; 1792 pair aliases share their atlas pixels; lifetimes match native replay deadlines; cosmic podium previews verified.
Compact art: all eight frames of 24 forward-gazing costumes fit 18–24 x 35–40 px; every rank emblem fits 18 x 18 px; 48 direct transformations have eight distinct frames over 24 ticks with planted feet, covered base pixels and exact endpoints; static champion cards regenerate exactly.
Cosmic ranks: eight unique costumes per scientist, eight distinct frames per costume and preparation modifier; accepted heads/gaze unchanged; all 75 skill-stage cards and the ranked animation regenerate exactly.
Cosmic auras: 33 distinct eight-frame background loops fit 96 x 112 px, including four podium constructions per scientist; stored frames retain their phases; front layers stay empty; aura frame strips/GIF regenerate exactly.
```

### Browser checks

```sh
node editor/server.js --no-open --port 19366
NODE_PATH=/workspace/.tfm2-env/browser/node_modules DB_EDITOR_URL=http://127.0.0.1:19366/ node tools/verify_coder_browser.cjs
NODE_PATH=/workspace/.tfm2-env/browser/node_modules SCIENCE_LAB_URL=http://127.0.0.1:19366/science.html node tools/verify_science_art_browser.cjs
NODE_PATH=/workspace/.tfm2-env/browser/node_modules SCIENCE_LAB_URL=http://127.0.0.1:19366/science.html node tools/verify_unified_theory_browser.cjs
NODE_PATH=/workspace/.tfm2-env/browser/node_modules node tools/verify_science_editor_browser.cjs
```

The integration script launches its own server and isolated temporary game fixture. Memory edits, backups and generator writes use fixture athletes; sentinel memory files for the four other mastery champions remain unchanged. No player save or installed mastery record was changed. The temporary game fixture is removed after the test.

Coder output:

```text
PASS: browser parity, 100 rows, language filtering, picker/rewrite, 4 rank arenas, 10-run comparison, no page errors/NaN.
[{"rank":"Script Kiddie","shipped":0.5,"distinct":0.5,"dps":0.20666666666666667},{"rank":"Intern","shipped":1.5,"distinct":1.5,"dps":1.7866666666666666},{"rank":"Junior","shipped":3.7,"distinct":3.7,"dps":9.47},{"rank":"Developer","shipped":6.6,"distinct":6.6,"dps":58.17666666666666},{"rank":"Senior","shipped":8.6,"distinct":8.6,"dps":53.230000000000004},{"rank":"Staff","shipped":9.3,"distinct":9.3,"dps":34.766666666666666},{"rank":"Architect","shipped":10,"distinct":10,"dps":65.46333333333334},{"rank":"Root #10","shipped":11.7,"distinct":11.7,"dps":82.28},{"rank":"Root #1 Zero-Day","shipped":14.8,"distinct":14.8,"dps":104.1}]
```

Standalone scientist art output:

```text
PASS: 75 actual skill animations across all 8 ranks, 33 eight-frame auras, three distinct #1 subject titles, 21 subject completions, 10 Top 10 positions, 25 travel/field animations, all six direct transformations with the destination aura, cooldown, same-form guard, paused morph, moving frames, pause/play, completion, desktop/mobile; no script or asset-loading errors.
```

Standalone scientist lab output:

```text
PASS: 375 native charge vectors, 75 skill cards, search and role filters, charge imbalance, reservation/cancel, all 50 recipes, 8-rank comparison, desktop/mobile without errors or horizontal overflow.
```

Combined editor output:

```text
PASS: one Science Lab with no duplicate viewer, shared Coder/Isliid styling, 75 native recipes and animations, all 50 experiments, 375 charge vectors, 51 rank/form appearances without stacking, one persistent iframe, hidden animation suspension, preserved notebook/pause, all five mastery lists, isolated scientist writes/backups and mastery generation with a pinned #1, correct pending win points, desktop/mobile layout.
```

The revised test retains coverage for all five mastery lists, official and exhibition win weights, Set writes, backups, generation with a selected #1, native passive and recipe/cooldown presentation. It additionally checks no duplicate viewer tab/button; exact shared Coder/Isliid shell width, heading and font; all 75 embedded skill/form selections; all 50 embedded templates; automatic trajectory selection; stopped inactive loops; metadata updates while the art loop is stopped; transformations/comparison controls after relocation; notebook and pause preservation; 51 arena rank/form cases without stacked layers; and both preview modes on mobile without horizontal overflow.

## Manual checks remaining

After merge, close the game and run Mod Manager **1 Update**, then **2 Check**. Confirm dependency/load versions, open the editor with option **3**, and check the combined Science Lab from the installed game. Use the actual save to populate mastery athlete names. Check both team facings, swaps and cosmic art readability, Coder IDE readability, and crowded-match FPS with scientists, Levi and Isliid #1. Final native combat, collisions and item interactions require real matches.

Round **122** in [playtest notes](playtest-notes.md) records the request, change, checks and manual items. All changes and the rebuilt DLL belong to this single branch and its new PR.

## Changed files

20 files:

- `docs/playtest-notes.md`
- `docs/unified-theory-guide.md`
- `docs/unified-theory-science-lab-study.png`
- `docs/unified-theory-science-lab-ui-verification.md`
- `docs/unified-theory-science-lab-ui.png`
- `editor/app.js`
- `editor/science-art.js`
- `editor/science-editor.js`
- `editor/science-embed.css`
- `editor/science-embed.js`
- `editor/science.css`
- `editor/server.js`
- `editor/skills.js`
- `editor/skilltest.js`
- `mods/tfm2_custom/mod.mod_info`
- `mods/tfm2_custom_ai/mod.mod_info`
- `mods/tfm2_custom_ai/tfm2_custom_ai.dll`
- `native/tfm2_custom_ai/mod.mod_info`
- `native/tfm2_custom_ai/src/lib.rs`
- `tools/verify_science_editor_browser.cjs`
