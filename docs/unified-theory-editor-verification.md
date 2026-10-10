# Scientist editor integration — round 121 verification

Native **0.10.34**, champion mod **0.2.34**, dependency **>=0.10.34**, `last_updated` **2026-10-11**. Both native metadata files match `VERSION`; `every_version_agrees` passes. Branch: `codex/scientist-editor-integration`, based on main **9b994d0** after PR #24 merged. This follow-up uses [PR #25](https://github.com/PowerViber/tfm2_mods/pull/25).

## Result

The Unified Theory now has the same common editor entry points as the other mastery champions:

- **Skill Test → Arena:** select Einstein, Newton or Curie and any ordinary rank or Unified Mind #1–#10. The outfit, cosmic aura and emblem replace the current three appearance layers. Move and basic-attack tests remain available. Direct links open Science Lab, the skill viewer and the scientist's mastery list.
- **Skill Test → Science Lab:** all 75 charged activations, individual token allocations, imbalance feedback, 50 experiment templates, reservation/cancel and deterministic eight-rank notebook comparisons.
- **Skill Test → Skill viewer (Unified Theory):** original shipped eight-frame sprites, all skills/ranks/podium positions, activation or supported travel/field animations, freely selected transformations, completion, and pause/play.
- **Skill Test → Mastery:** per-athlete points, games, wins, native rank names, Top 10 and pending result weights. The existing randomizer supports fixed ranks and selected podium athletes using `unified_theory_memory.txt` and `unified_theory_pending.txt`.
- **Skill Lab:** preserves the native passive, offers three direct navigation buttons, and lists actual recipes, per-token CU, total CU and cooldowns. Empty conventional skill slots no longer imply that those placeholder cooldowns control the native notebook.

[Skill test preview](unified-theory-editor-test.png) · [Original sprite viewer](unified-theory-editor-viewer.png) · [Mastery list preview](unified-theory-editor-mastery.png) · [Updated guide](unified-theory-guide.md).

The preview's **charge equations are exact**; its trajectory and notebook comparison remain schematic. This change does not add a browser implementation of the game's native AI planner, collisions, mitigation or item procs.

## Animation and state handling

One managed, same-origin frame reuses the existing lab and inspector; switching between them preserves notebook entries and allocations. The parent and child validate selection messages by origin, frame identity and bounded skill/rank/position values. The iframe isolates the standalone page's styles from the main editor. Embedded styling matches the common panels, with responsive controls and wrapping tabs. `/science.html` remains usable on its own.

Only the visible scientist view runs its animation loop. Leaving Skill Test suspends both; the inspector retains user pause state and pauses its elapsed time across hiding. The ordinary arena also stops stepping/rendering behind the other views. Atlas requests share in-flight loads. Scientist arena initialization loads its current appearance/basic-attack sheets, rather than preloading the complete effect set. Rank/form changes replace layer names instead of adding repeated emitters. Asynchronous champion and mastery reads cannot overwrite a newer selection.

No scientific gameplay, Coder behavior, generated art, vectors or canonical data changed. The native source change is the required release VERSION bump. Manager source/EXE is unchanged. Browser loop and layer checks do not measure actual game FPS.

## Verification commands and outputs

The attached `coder-verification.md` supplies the required shared-DLL, editor and release checks. The user request supplies the editor scope. Its historical versions/branch examples were treated as reference information. Generated files were not edited by hand; the canonical scientist verifier regenerates and checks identical bytes.

Environment: sourced `/workspace/.tfm2-env/env.sh`; configured offline Rust dependencies, MinGW linker, Python/Pillow and Chromium/Playwright. Linux/Windows builds and both suites contain **0 errors and 0 warnings**. `git diff --check` passes.

### Linux release build and full native suite

```sh
source /workspace/.tfm2-env/env.sh
cargo build --offline --release -j4 --manifest-path native/tfm2_custom_ai/Cargo.toml
cargo test --offline --release -j4 --manifest-path native/tfm2_custom_ai/Cargo.toml -- --nocapture
```

Build output:

```text
Compiling tfm2_custom_ai v0.1.0 (/workspace/tfm2_mods/native/tfm2_custom_ai)
    Finished `release` profile [optimized] target(s) in 17.30s
```

Relevant native test output, including all required named Coder checks and the full suite result:

```text
Compiling tfm2_custom_ai v0.1.0 (/workspace/tfm2_mods/native/tfm2_custom_ai)
    Finished `release` profile [optimized] target(s) in 22.11s
test coder::tests::choosing_is_cheap ... ok
test coder::tests::every_view_name_exists ... ok
test coder::tests::he_doesnt_spam_the_same_function ... ok
test coder::tests::lab_vectors ... ok
test coder::tests::scripts_keep_him_writing ... ok
test coder::tests::the_data_center_has_its_risks ... ok
test scribble::real_data::real_pending ... ignored
test unified_theory::integration::cosmic_auras_replace_once_follow_the_morph_and_never_stack ... ok
Cosmic crowded scene: 594 starts over 7 s, peak 30 live temporary animations, max 6 starts/update; persistent loops added once
test unified_theory::integration::visual_layers_are_replaced_restored_and_cleared_without_stacking ... ok
test version_tests::every_version_agrees ... ok
test result: ok. 172 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.60s
```

The sole ignored test is the pre-existing host fixture `scribble::real_data::real_pending`.

### Coder/native asset parity

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
Finished `release` profile [optimized] target(s) in 0.04s
     Running unittests src/main.rs (tools/manager/target/release/deps/tfm2_manager-756fc6bf9fcfdf44)

running 13 tests
test core::tests::build_outputs_reset_before_pull ... ok
test core::tests::champion_data_errors_name_the_bad_tag ... ok
test core::tests::check_reports_mismatch ... ok
test core::tests::install_copies_and_keeps_plans ... ok
test core::tests::log_scan_finds_load_errors ... ok
test core::tests::mods_json_enable ... ok
test core::tests::perf_log_is_read ... ok
test core::tests::stamps_are_dates ... ok
test core::tests::native_version_meets_the_champions_requirement ... ok
test core::tests::repo_root_found_from_exe_dir ... ok
test core::tests::stale_duplicates_go_to_backup ... ok
test core::tests::vdf_and_manifest_parse ... ok
test core::tests::the_repos_champions_all_load ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
```

### JavaScript syntax and Coder lab

```sh
node --check editor/coderlab.js
node -e 'const l=require("./editor/coderlab.js");console.log(l.selfTest());for(const [r,p]of l.LADDER)l.simulateSkirmish(r,p,1);'
```

```text
true
```

No crashes across any Coder rank. `node --check` also passes for `app.js`, `skills.js`, `skilltest.js`, `server.js`, `science-editor.js`, `science-embed.js`, `science-art.js`, `unified-theorylab.js` and both new browser verification scripts; syntax success emits no output.

### Windows DLL and shipped hash

```sh
cargo build --offline --release --target x86_64-pc-windows-gnu -j4 --manifest-path native/tfm2_custom_ai/Cargo.toml
cp native/tfm2_custom_ai/target/x86_64-pc-windows-gnu/release/tfm2_custom_ai.dll mods/tfm2_custom_ai/tfm2_custom_ai.dll
sha256sum native/tfm2_custom_ai/target/x86_64-pc-windows-gnu/release/tfm2_custom_ai.dll mods/tfm2_custom_ai/tfm2_custom_ai.dll 'TFM2 Mod Manager.exe'
```

The sourced setup selects `x86_64-w64-mingw32-gcc-posix` through `CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER`.

```text
Compiling tfm2_custom_ai v0.1.0 (/workspace/tfm2_mods/native/tfm2_custom_ai)
    Finished `release` profile [optimized] target(s) in 17.09s
a5cc865a232e3da6933a14827a771956bb0a7ceb0eb5e906de42d062d2513533  native/tfm2_custom_ai/target/x86_64-pc-windows-gnu/release/tfm2_custom_ai.dll
a5cc865a232e3da6933a14827a771956bb0a7ceb0eb5e906de42d062d2513533  mods/tfm2_custom_ai/tfm2_custom_ai.dll
01780422f34e0b040eb3e299ee0f6a9f6f1227462728f402aa0f093383852e91  TFM2 Mod Manager.exe
```

Build and shipped DLL hashes match. Manager source did not change, so its EXE was not rebuilt; its hash matches the prior release.

### Scientist canonical assets and exact lab math

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

### Browser coverage

Start the editor for the standalone regression scripts:

```sh
node editor/server.js --no-open --port 19363
NODE_PATH=/workspace/.tfm2-env/browser/node_modules DB_EDITOR_URL=http://127.0.0.1:19363/ node tools/verify_coder_browser.cjs
NODE_PATH=/workspace/.tfm2-env/browser/node_modules SCIENCE_LAB_URL=http://127.0.0.1:19363/science.html node tools/verify_science_art_browser.cjs
NODE_PATH=/workspace/.tfm2-env/browser/node_modules SCIENCE_LAB_URL=http://127.0.0.1:19363/science.html node tools/verify_unified_theory_browser.cjs
NODE_PATH=/workspace/.tfm2-env/browser/node_modules node tools/verify_science_editor_browser.cjs
```

The new editor test starts its own server and builds an isolated game fixture in the system temporary directory. Its memory edits, backups and generation operate on test athletes. It verifies official wins (+1.5) and exhibition wins (+0.75), correct scientist rank names, all five champion lists, a real Set write, a generator write with a selected #1, and unchanged sentinel files for all four other champions. No player save or installed mastery record was changed. Screenshots use fixture athletes.

Coder browser output:

```text
PASS: browser parity, 100 rows, language filtering, picker/rewrite, 4 rank arenas, 10-run comparison, no page errors/NaN.
[{"rank":"Script Kiddie","shipped":0.5,"distinct":0.5,"dps":0.20666666666666667},{"rank":"Intern","shipped":1.5,"distinct":1.5,"dps":1.7866666666666666},{"rank":"Junior","shipped":3.7,"distinct":3.7,"dps":9.47},{"rank":"Developer","shipped":6.6,"distinct":6.6,"dps":58.17666666666666},{"rank":"Senior","shipped":8.6,"distinct":8.6,"dps":53.230000000000004},{"rank":"Staff","shipped":9.3,"distinct":9.3,"dps":34.766666666666666},{"rank":"Architect","shipped":10,"distinct":10,"dps":65.46333333333334},{"rank":"Root #10","shipped":11.7,"distinct":11.7,"dps":82.28},{"rank":"Root #1 Zero-Day","shipped":14.8,"distinct":14.8,"dps":104.1}]
```

Scientist standalone art output:

```text
PASS: 75 actual skill animations across all 8 ranks, 33 eight-frame auras, three distinct #1 subject titles, 21 subject completions, 10 Top 10 positions, 25 travel/field animations, all six direct transformations with the destination aura, cooldown, same-form guard, paused morph, moving frames, pause/play, completion, desktop/mobile; no script or asset-loading errors.
```

Scientist standalone lab output:

```text
PASS: 375 native charge vectors, 75 skill cards, search and role filters, charge imbalance, reservation/cancel, all 50 recipes, 8-rank comparison, desktop/mobile without errors or horizontal overflow.
```

New integration output:

```text
PASS: integrated test/viewer, 75 native recipes, 375 charge vectors, 51 rank/form appearances without stacking, one persistent iframe, hidden animation suspension, preserved notebook/pause, all five mastery lists, isolated scientist writes/backups and mastery generation with a pinned #1, correct pending win points, desktop/mobile layout.
```

The 51 appearance cases cover three forms × seven ordinary ranks plus three forms × all ten Top 10 positions. Each case has exactly three current scientist appearance layers, with no stale Scribble layers. The tests also verify selections carry back to the arena, native passive/recipe/cooldown presentation, one preserved iframe, hidden-loop suspension, notebook/pause preservation, and 390 px mobile layout without document or iframe overflow. Desktop previews were visually reviewed.

## Manual checks remaining

- Close the game, run Mod Manager **1 Update**, then **2 Check**; confirm the new version and matching dependency/load popup.
- Open the editor through option **3** from the installed game. Open the actual save for athlete names, then test the scientist's mastery generator and list. Restart the game so it reloads changed mastery.
- Check both team facings, movement alignment, scientist swaps, cosmic aura/emblem clarity and all three #1 identities at actual gameplay scale.
- Run crowded matches with the scientist, Levi and Isliid #1 and one/two Coders; confirm FPS, Coder IDE readability and absence of load/runtime errors. Final collisions, mitigation, item interactions and measured game performance remain unavailable in cloud.

Round **121** in [playtest notes](playtest-notes.md) records the request, implementation, checks and manual items. All changes, rebuilt DLL and verification artifacts belong to the single branch and its new PR.

## Changed files

25 files:

- `docs/playtest-notes.md`
- `docs/unified-theory-editor-mastery.png`
- `docs/unified-theory-editor-test.png`
- `docs/unified-theory-editor-verification.md`
- `docs/unified-theory-editor-viewer.png`
- `docs/unified-theory-guide.md`
- `editor/app.js`
- `editor/index.html`
- `editor/science-art.js`
- `editor/science-editor.js`
- `editor/science-embed.js`
- `editor/science.css`
- `editor/science.html`
- `editor/server.js`
- `editor/skills.js`
- `editor/skilltest.js`
- `editor/style.css`
- `editor/unified-theorylab.js`
- `mods/tfm2_custom/mod.mod_info`
- `mods/tfm2_custom_ai/mod.mod_info`
- `mods/tfm2_custom_ai/tfm2_custom_ai.dll`
- `native/tfm2_custom_ai/mod.mod_info`
- `native/tfm2_custom_ai/src/lib.rs`
- `tools/verify_coder_browser.cjs`
- `tools/verify_science_editor_browser.cjs`
