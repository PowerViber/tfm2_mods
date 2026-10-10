# The Theory Holds — round 119 verification

Native **0.10.32**, champion mod **0.2.32**, dependency **>=0.10.32**; dates **2026-10-10**. Both native metadata files match `VERSION`, and `every_version_agrees` passes. This updates the open [PR #24](https://github.com/PowerViber/tfm2_mods/pull/24) on `codex/scientist-compact-forms`.

## Result and visual review

#1 has its own construction: a dark suspended singularity held by three separated, solid scientific instrument fragments. It replaces the ordinary Top 10 aura. The palette is midnight, ivory and the active form’s pale blue, antique gold or teal. A measuring key enters the opaque void and emerges at another elevation. Nearby observations bend, the fragments settle and hold, one contained alignment pulse occurs, then the mechanism releases. The complete sequence uses **eight frames at 200 ms each (1.6 seconds)**. Its opaque centre stays empty; the actual silhouette measures **67–73 × 86–87 px**, inside the existing 96 × 112 anchor. The compact scientist remains in front of the construction.

Ordinary Top 10 has one quiet orbit with small instruments; #3 displays three scientific worlds and #2 paired wormholes. Cosmic skill highlights move through ivory brightness rather than rainbow hues, retaining each discipline’s scientific material and accent. All 75 activations retain four distinct eight-frame stages and their individual actions; fields, packets and preparation modifiers retain their eight phases. #1 completion briefly joins the three scientific materials and collapses into one captured horizon. Its existing delayed shadow remains.

Unified Mind coat panels follow the active discipline. The accepted human faces, forward gaze, three-quarter pose, scientist props and shared feet retain their geometry. All costume frames fit the existing **18–24 × 35–40 px** guard (actual accepted idle body **18–23 × 37–38 px**), badges fit **18 × 18 px**, and all 48 morphs retain eight distinct frames, exact endpoints and fixed feet over 24 ticks.

Science Lab draws the original aura and outfit atlas animations independently, including during morphs. The slower #1 cadence therefore matches native assets without slowing body animation. Inspected the original aura frame strip, Top 10 comparison and runtime browser inspector at normal size. Previews use the actual stored palette-quantized atlas pixels.

[Watch #1](unified-theory-auras.gif) · [Original eight aura sprites](unified-theory-aura-frames.png) · [Podium comparison](unified-theory-top10.png) · [Rank progression](unified-theory-ranked-cosmic.gif) · [Einstein’s skills](unified-theory-skills-einstein.png) · [Newton’s skills](unified-theory-skills-newton.png) · [Curie’s skills](unified-theory-skills-curie.png).

## Animation cost and preserved behavior

One persistent background aura is retained, added once and replaced only when the visible form/rank/podium changes. The body or morph, badge, aura and optional preparation marks still total **at most four persistent loops**. No runtime particles, shaders, extra loops per instrument or new game objects are introduced. Existing restoration, busy-morph destination binding and death cleanup remain.

The existing crowded native fixture reports **594 temporary starts in seven seconds, peak 30 live temporary animations, maximum six starts/update**. Existing non-overlap channels, coalescing and replay deadlines remain. All **36 scientist VFX sheets fit <=2048 × 2048**, with **3536 views** and unchanged names/counts. Estimated decoded RGBA sheet storage changes from **180.38 to 179.85 MiB**, a **0.52 MiB reduction** calculated before rounding. This estimate does not measure game memory, CPU cost or FPS.

Combat, charge, damage, notebook timing, RNG, activation/cooldown timings and renderer logic are unchanged. The native source change is the required release version. Coder source/art/data/vectors/lab and manager source/EXE are unchanged. The attached `coder-verification.md` supplies regression/release checks; the user’s request supplies this scientist design scope. Its historical versions and branch examples are not new implementation instructions.

## Verification evidence

The art verifier keeps the existing footprint, exact coverage, fixed-foot, frame uniqueness, endpoint, centred-atlas, lifecycle and regeneration assertions. New #1 assertions require a bounded silhouette, a genuinely opaque dark centre, a single active discipline accent, eight unique frames and the 1.6-second cadence. Browser checks require independent outfit animation assets and native aura cadence. No tests were skipped or weakened. The native ignored test is the pre-existing `scribble::real_data::real_pending` host fixture.

Generated only the affected scientist art/data using canonical scripts. Coder generation and vectors were untouched.

```sh
source /workspace/.tfm2-env/env.sh
python 'Claude outputs/unified_theory/art.py'
python 'Claude outputs/unified_theory/data.py'
```

### Linux release build

```sh
cargo build --offline --release -j4 --manifest-path native/tfm2_custom_ai/Cargo.toml
```

```text
Compiling tfm2_custom_ai v0.1.0 (/workspace/tfm2_mods/native/tfm2_custom_ai)
    Finished `release` profile [optimized] target(s) in 19.22s
```

### Full native suite

```sh
cargo test --offline --release -j4 --manifest-path native/tfm2_custom_ai/Cargo.toml -- --nocapture
```

```text
test unified_theory::integration::all_six_form_changes_are_direct_and_finish_without_stacking ... ok
test unified_theory::integration::cosmic_auras_replace_once_follow_the_morph_and_never_stack ... ok
Cosmic crowded scene: 594 starts over 7 s, peak 30 live temporary animations, max 6 starts/update; persistent loops added once
test unified_theory::integration::mastery_art_does_not_change_combat_resources_or_rng ... ok
test unified_theory::integration::visual_layers_are_replaced_restored_and_cleared_without_stacking ... ok
test version_tests::every_version_agrees ... ok
test result: ok. 171 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 1.05s
```

### Scientist assets and exact parity

```sh
python tools/verify_unified_theory.py
```

```text
375 exact native charge vectors; all 8 ranks deterministic
75 unique skills; 50 bounded recipes; generated files current
Cosmic Unified Experiment: 17 badges, 24 outfits, 300 eight-frame casts, 44 eight-frame fields, 448 eight-frame packet loops with shared pair aliases
Original body, three personas, notebooks, meter and mastery art generated
tfm2_custom_unified_theory 3349 effects 186 persona / mastery overlays
Verified 3536 views over 36 VFX sheets; original body <=2048; notebook lifetimes; regeneration byte-identical.
Art: 300 casts, 44 fields and 448 directional packet loops have eight distinct frames; 1792 pair aliases share their atlas pixels; lifetimes match native replay deadlines; cosmic podium previews verified.
Compact art: all eight frames of 24 forward-gazing costumes fit 18–24 x 35–40 px; every rank emblem fits 18 x 18 px; 48 direct transformations have eight distinct frames over 24 ticks with planted feet, covered base pixels and exact endpoints; static champion cards regenerate exactly.
Cosmic ranks: eight unique costumes per scientist, eight distinct frames per costume and preparation modifier; accepted heads/gaze unchanged; all 75 skill-stage cards and the ranked animation regenerate exactly.
Cosmic auras: 33 distinct eight-frame background loops fit 96 x 112 px, including four podium constructions per scientist; stored frames retain their phases; front layers stay empty; aura frame strips/GIF regenerate exactly.
```

### Coder regression

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
Finished `release` profile [optimized] target(s) in 0.01s
     Running unittests src/main.rs (tools/manager/target/release/deps/tfm2_manager-756fc6bf9fcfdf44)

running 13 tests
test core::tests::build_outputs_reset_before_pull ... ok
test core::tests::champion_data_errors_name_the_bad_tag ... ok
test core::tests::install_copies_and_keeps_plans ... ok
test core::tests::log_scan_finds_load_errors ... ok
test core::tests::check_reports_mismatch ... ok
test core::tests::mods_json_enable ... ok
test core::tests::perf_log_is_read ... ok
test core::tests::native_version_meets_the_champions_requirement ... ok
test core::tests::stamps_are_dates ... ok
test core::tests::vdf_and_manifest_parse ... ok
test core::tests::stale_duplicates_go_to_backup ... ok
test core::tests::repo_root_found_from_exe_dir ... ok
test core::tests::the_repos_champions_all_load ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

### Coder syntax, self-test and every rank

```sh
node -c editor/coderlab.js
node -c editor/science-art.js
node -e "const l=require('./editor/coderlab.js'); console.log(l.selfTest()); for (const [r,p] of l.LADDER) l.simulateSkirmish(r,p,1);"
```

```text
true
```

### Windows release build

```sh
cargo build --offline --release --target x86_64-pc-windows-gnu -j4 --manifest-path native/tfm2_custom_ai/Cargo.toml
```

```text
Compiling tfm2_custom_ai v0.1.0 (/workspace/tfm2_mods/native/tfm2_custom_ai)
    Finished `release` profile [optimized] target(s) in 16.82s
```

### Science art browser

```sh
NODE_PATH=/workspace/.tfm2-env/browser/node_modules SCIENCE_LAB_URL=http://127.0.0.1:19362/science.html node tools/verify_science_art_browser.cjs
```

```text
PASS: 75 actual skill animations across all 8 ranks, 33 eight-frame auras, 10 Top 10 positions, 25 travel/field animations, all six direct transformations with the destination aura, cooldown, same-form guard, paused morph, moving frames, pause/play, completion, desktop/mobile; no script or asset-loading errors.
```

### Science mechanics browser

```sh
NODE_PATH=/workspace/.tfm2-env/browser/node_modules SCIENCE_LAB_URL=http://127.0.0.1:19362/science.html node tools/verify_unified_theory_browser.cjs
```

```text
PASS: 375 native charge vectors, 75 skill cards, search and role filters, charge imbalance, reservation/cancel, all 50 recipes, 8-rank comparison, desktop/mobile without errors or horizontal overflow.
```

### Coder browser checklist

```sh
NODE_PATH=/workspace/.tfm2-env/browser/node_modules node /tmp/science-art-coder-browser.cjs
```

```text
PASS: browser parity, 100 rows, language filtering, picker/rewrite, 4 rank arenas, 10-run comparison, no page errors/NaN.
[{"rank":"Script Kiddie","shipped":0.5,"distinct":0.5,"dps":0.20666666666666667},{"rank":"Intern","shipped":1.5,"distinct":1.5,"dps":1.7866666666666666},{"rank":"Junior","shipped":3.7,"distinct":3.7,"dps":9.47},{"rank":"Developer","shipped":6.6,"distinct":6.6,"dps":58.17666666666666},{"rank":"Senior","shipped":8.6,"distinct":8.6,"dps":53.230000000000004},{"rank":"Staff","shipped":9.3,"distinct":9.3,"dps":34.766666666666666},{"rank":"Architect","shipped":10,"distinct":10,"dps":65.46333333333334},{"rank":"Root #10","shipped":11.7,"distinct":11.7,"dps":82.28},{"rank":"Root #1 Zero-Day","shipped":14.8,"distinct":14.8,"dps":104.1}]
```

### Shipped binaries

Linux/Windows builds: **0 errors, 0 warnings**. The sourced setup selects `x86_64-w64-mingw32-gcc` for the Windows target. Copied the built DLL to `mods/tfm2_custom_ai/tfm2_custom_ai.dll`; build and shipped SHA256 match. Manager source did not change, so its EXE was not rebuilt and its SHA256 remains unchanged.

```text
a2ccbb7b87f7735efcfc240c5710e70d42ed1d7510f6062aff821d55e7689c23  native/tfm2_custom_ai/target/x86_64-pc-windows-gnu/release/tfm2_custom_ai.dll
a2ccbb7b87f7735efcfc240c5710e70d42ed1d7510f6062aff821d55e7689c23  mods/tfm2_custom_ai/tfm2_custom_ai.dll
01780422f34e0b040eb3e299ee0f6a9f6f1227462728f402aa0f093383852e91  TFM2 Mod Manager.exe
```

## Changed files (54)

- `Claude outputs/unified_theory/compact_art.py`
- `Claude outputs/unified_theory/cosmic_art.py`
- `Claude outputs/unified_theory/mastery_art.py`
- `docs/playtest-notes.md`
- `docs/unified-theory-art-inspector.png`
- `docs/unified-theory-aura-frames.png`
- `docs/unified-theory-auras.gif`
- `docs/unified-theory-cosmic-frames.png`
- `docs/unified-theory-cosmic.gif`
- `docs/unified-theory-effects.png`
- `docs/unified-theory-facing.png`
- `docs/unified-theory-guide.md`
- `docs/unified-theory-lab.png`
- `docs/unified-theory-mastery.png`
- `docs/unified-theory-ranked-cosmic.gif`
- `docs/unified-theory-restrained-cosmic-verification.md`
- `docs/unified-theory-skills-curie.png`
- `docs/unified-theory-skills-einstein.png`
- `docs/unified-theory-skills-newton.png`
- `docs/unified-theory-top10.png`
- `editor/science-art-preview.json`
- `editor/science-art.js`
- `editor/science-mastery-preview.png`
- `editor/science.html`
- `mods/tfm2_custom/mod.mod_info`
- `mods/tfm2_custom/vfx/science_completion#anim.fanim`
- `mods/tfm2_custom/vfx/science_completion#sheet.png`
- `mods/tfm2_custom/vfx/science_echo#sheet.png`
- `mods/tfm2_custom/vfx/science_equipment0#anim.fanim`
- `mods/tfm2_custom/vfx/science_equipment0#sheet.png`
- `mods/tfm2_custom/vfx/science_equipment1#anim.fanim`
- `mods/tfm2_custom/vfx/science_equipment1#sheet.png`
- `mods/tfm2_custom/vfx/science_equipment2#anim.fanim`
- `mods/tfm2_custom/vfx/science_equipment2#sheet.png`
- `mods/tfm2_custom/vfx/science_fields_t0#sheet.png`
- `mods/tfm2_custom/vfx/science_fields_t1#sheet.png`
- `mods/tfm2_custom/vfx/science_fields_t2#sheet.png`
- `mods/tfm2_custom/vfx/science_fields_t3#sheet.png`
- `mods/tfm2_custom/vfx/science_modifiers#sheet.png`
- `mods/tfm2_custom/vfx/science_outfits#sheet.png`
- `mods/tfm2_custom/vfx/science_packets_t3#sheet.png`
- `mods/tfm2_custom/vfx/science_skills0_t0#sheet.png`
- `mods/tfm2_custom/vfx/science_skills0_t1#sheet.png`
- `mods/tfm2_custom/vfx/science_skills0_t2#sheet.png`
- `mods/tfm2_custom/vfx/science_skills0_t3#sheet.png`
- `mods/tfm2_custom/vfx/science_skills1_t3#sheet.png`
- `mods/tfm2_custom/vfx/science_skills2_t3#sheet.png`
- `mods/tfm2_custom/vfx/science_transforms#sheet.png`
- `mods/tfm2_custom_ai/mod.mod_info`
- `mods/tfm2_custom_ai/tfm2_custom_ai.dll`
- `native/tfm2_custom_ai/mod.mod_info`
- `native/tfm2_custom_ai/src/lib.rs`
- `tools/verify_science_art_browser.cjs`
- `tools/verify_unified_theory.py`

## In-game checks still required

No game match is available in this cloud environment. Check Mod Manager Update/Check and dependency loading, #1 recognition at game scale, both team facings, body/prop/aura/badge alignment during movement, eight-frame transit/hold/pulse, completion and delayed shadow, swaps/death/respawn cleanup, Coder notebook readability, and crowded-match FPS with scientists, Coder, Levi and Isliid #1. Automated animation bounds are not an in-game FPS measurement.

Round **119** is appended to `docs/playtest-notes.md`. The revision and generated assets are kept together on `codex/scientist-compact-forms` for PR #24; merge/install checks are pending in-game review.
