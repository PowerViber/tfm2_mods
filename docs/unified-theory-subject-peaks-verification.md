# Three mastery peaks — round 120 verification

Native **0.10.33**, champion mod **0.2.33**, dependency **>=0.10.33**; dates **2026-10-10**. Both native metadata files match `VERSION`, and `every_version_agrees` passes. This revision belongs to [PR #24](https://github.com/PowerViber/tfm2_mods/pull/24), branch `codex/scientist-compact-forms`.

## Result and visual review

Each scientist now develops a different frontier across all eight ranks, four skill-art stages and the Top 10 variants:

| Subject | Mastery progression | #1 construction and eight-frame sequence | Actual #1 bounds |
|---|---|---|---|
| Einstein — physics | Quantum light paths, stellar observations and open cosmic cross-sections | **Cosmic Genesis:** a quantum seed expands into possible paths and a stellar web; stars emerge, quantum and cosmic patterns align for one pulse, then the constellation contracts. A folded star-filled cross-section gives #1 its own paused silhouette. | 67 × 78 px |
| Newton — mathematics | Measured solid faces, higher-dimensional geometry and nested infinite interiors | **The Unprovable Shape:** a solid triangular construction uses cyclic occlusion; an interior dimensional face grows, folds edge-on and emerges inverted, while its shadow changes dimensional shape. | 61 × 85 px |
| Marie Curie — chemistry | Mineral seeds, molecular bonds, double strands, membranes and living assembly | **Genesis Vessel:** mineral becomes bonds, a molecular ladder, a membrane and a branching alien organism; a sensory branch reaches toward Curie's held sample, then folds back into a seed with a heartbeat. Paired biological lobes make the #1 apparatus distinct. | 47–51 × 76–77 px |

Every #1 construction has **eight distinct frames at 200 ms each (1.6 seconds)**, within the existing 96 × 112 px anchor. Ordinary rank and podium silhouettes differ by subject, and each #1 differs from ordinary Top 10 even when paused and colour is removed. The palette retains midnight surfaces, ivory highlights and one active blue, antique-gold or teal accent.

All **75 skills** retain four eight-frame stages with recognizable actions. Physics uses prisms, quantum paths, clocks, folded coordinates and stellar webs. Mathematics uses vectors, divided geometry, intersecting curves, nested steps and solid higher-dimensional objects. Chemistry uses mineral scaffolds, bonded molecules, membranes and transforming life. Fields and directional packets follow the same subjects. **Gravity Well and Horizon Ring are the only skills that use the black-hole helper.** Rank auras, other casts, completion and echoes do not use it.

There are now **21 subject-specific completion animations**, chosen from the final successfully committed stage of a combination. They retain the existing 48-tick (0.8-second) channel. The three #1 imprints are a stellar/quantum trace, dimensional shadow or molecular trace; their existing delayed eight-tick lifetime remains.

Clothing motifs follow the three frontiers. The accepted faces, forward gaze, near/far arms, small props and planted feet retain their geometry. The compact idle body remains **18–23 × 37–38 px**, checked within the existing 18–24 × 35–40 px guard. All 17 emblems remain <=18 × 18 px. All 48 rank/route transformations retain eight distinct frames over 24 ticks (0.4 seconds), exact endpoints and fixed feet; all six direct scientist routes remain available.

Reviewed the stored sprite strips, Top 10 comparison and individual browser inspectors. Previews read the shipped palette-quantized atlas pixels; the scientist remains in front of its background construction.

[Watch the three #1 appearances](unified-theory-auras.gif) · [Eight original frames per scientist](unified-theory-aura-frames.png) · [Podium comparison](unified-theory-top10.png) · [All ranks](unified-theory-ranked-cosmic.gif) · [Einstein inspector](unified-theory-peak-einstein.png) · [Newton inspector](unified-theory-peak-newton.png) · [Curie inspector](unified-theory-peak-curie.png).

[Einstein's 25 skills](unified-theory-skills-einstein.png) · [Newton's 25 skills](unified-theory-skills-newton.png) · [Curie's 25 skills](unified-theory-skills-curie.png) · [Original skill frames](unified-theory-cosmic-frames.png) · [Design guide](unified-theory-guide.md).

## Animation cost and preserved behavior

The complete rank aura remains **one persistent background loop**, added once and replaced only when its displayed form, rank or podium changes. Body or morph, badge, aura and optional preparation marks total **at most four persistent loops**. No separate emitter or runtime loop is added per star, face, molecule or organism. Missing layers restore individually; busy morphs bind to their destination aura; death clears all cosmetics.

Existing cast/completion non-overlap, the three-item pending queue, coalescing and replay deadlines remain. The crowded native fixture reports **594 temporary starts over seven seconds, peak 30 live temporary animations, maximum six starts/update**, with persistent loops added once. Fields retain their 48-tick replay deadline; moving packets retain four 12-tick frame-pair slices; notebook refresh/expiry remains eight ticks.

All **3550 scientist views over 36 VFX sheets** fit <=2048 × 2048. Splitting seven generic completion variants into 21 subject variants adds 14 views. Estimated decoded RGBA atlas storage changes from **179.85 to 184.38 MiB**, an increase of **4.53 MiB**. This estimate measures sheet dimensions, not actual game memory, CPU cost or FPS. The animation-count limits are unchanged; actual crowded-match performance remains a manual check.

The native behavior change routes completion art to the final successful stage's subject. It does not change damage, charge, resources, notebook execution, combat cooldowns, collision size or RNG. Coder source, art, vectors, champion data and lab behavior are untouched. Manager source/EXE is unchanged. The attached `coder-verification.md` supplies the shared-DLL/editor/release checks; the user's scientist request supplies the design scope. Historical versions and branch examples in that document are not new implementation instructions.

## Verification evidence

Canonical scientist generators produced all affected art/data. Generated atlases and champion view lists were not edited by hand. Coder generation and vectors were untouched.

The asset verifier retains coverage, planted feet, compact bounds, exact endpoints, centred frames, frame uniqueness, lifetimes and byte-identical regeneration. It checks different subject alpha silhouettes at every rank/frame and a different #1 silhouette from ordinary Top 10 at every frame. It replaces the prior design's dark-centre requirement with the requested subject silhouettes and verifies that unrelated animations cannot call the black-hole helper. All 21 completion animations have eight distinct stored frames and their original lifetime; all three echoes retain theirs.

The new native test `completion_art_follows_the_subject_of_the_final_committed_stage` runs real two-stage commitments ending in each subject, checking one completion channel, correct identity and the original 48-tick lifetime. Existing success/failure, rapid-cast, anti-stacking and crowded-scene tests remain. The sole ignored test is the pre-existing `scribble::real_data::real_pending` host fixture.

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
    Finished `release` profile [optimized] target(s) in 16.86s
```

### Full native suite

Relevant output, including the complete suite result:

```sh
cargo test --offline --release -j4 --manifest-path native/tfm2_custom_ai/Cargo.toml -- --nocapture
```

```text
test coder::tests::choosing_is_cheap ... ok
test coder::tests::every_view_name_exists ... ok
test coder::tests::he_doesnt_spam_the_same_function ... ok
test coder::tests::lab_vectors ... ok
test coder::tests::scripts_keep_him_writing ... ok
test coder::tests::the_data_center_has_its_risks ... ok
test scribble::real_data::real_pending ... ignored
test unified_theory::integration::all_six_form_changes_are_direct_and_finish_without_stacking ... ok
test unified_theory::integration::completion_art_follows_the_subject_of_the_final_committed_stage ... ok
test unified_theory::integration::cosmic_auras_replace_once_follow_the_morph_and_never_stack ... ok
Cosmic crowded scene: 594 starts over 7 s, peak 30 live temporary animations, max 6 starts/update; persistent loops added once
test unified_theory::integration::mastery_art_does_not_change_combat_resources_or_rng ... ok
test unified_theory::integration::visual_layers_are_replaced_restored_and_cleared_without_stacking ... ok
test version_tests::every_version_agrees ... ok
test result: ok. 172 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.59s
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
tfm2_custom_unified_theory 3363 effects 186 persona / mastery overlays
Verified 3550 views over 36 VFX sheets; original body <=2048; notebook lifetimes; regeneration byte-identical.
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
Finished `release` profile [optimized] target(s) in 0.00s
     Running unittests src/main.rs (tools/manager/target/release/deps/tfm2_manager-756fc6bf9fcfdf44)

running 13 tests
test core::tests::check_reports_mismatch ... ok
test core::tests::log_scan_finds_load_errors ... ok
test core::tests::mods_json_enable ... ok
test core::tests::native_version_meets_the_champions_requirement ... ok
test core::tests::perf_log_is_read ... ok
test core::tests::repo_root_found_from_exe_dir ... ok
test core::tests::stale_duplicates_go_to_backup ... ok
test core::tests::stamps_are_dates ... ok
test core::tests::install_copies_and_keeps_plans ... ok
test core::tests::build_outputs_reset_before_pull ... ok
test core::tests::vdf_and_manifest_parse ... ok
test core::tests::champion_data_errors_name_the_bad_tag ... ok
test core::tests::the_repos_champions_all_load ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

### Syntax, self-test and every Coder rank

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
    Finished `release` profile [optimized] target(s) in 15.80s
```

### Science artwork browser

```sh
NODE_PATH=/workspace/.tfm2-env/browser/node_modules SCIENCE_LAB_URL=http://127.0.0.1:19362/science.html node tools/verify_science_art_browser.cjs
```

```text
PASS: 75 actual skill animations across all 8 ranks, 33 eight-frame auras, three distinct #1 subject titles, 21 subject completions, 10 Top 10 positions, 25 travel/field animations, all six direct transformations with the destination aura, cooldown, same-form guard, paused morph, moving frames, pause/play, completion, desktop/mobile; no script or asset-loading errors.
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

Linux and Windows release builds: **0 errors, 0 warnings**. The sourced setup selects `x86_64-w64-mingw32-gcc` for the Windows target. The built DLL was copied into the mod. Build and shipped SHA256 match. Manager source did not change, so its EXE was not rebuilt and its hash remains unchanged.

```sh
cp native/tfm2_custom_ai/target/x86_64-pc-windows-gnu/release/tfm2_custom_ai.dll mods/tfm2_custom_ai/tfm2_custom_ai.dll
sha256sum native/tfm2_custom_ai/target/x86_64-pc-windows-gnu/release/tfm2_custom_ai.dll mods/tfm2_custom_ai/tfm2_custom_ai.dll 'TFM2 Mod Manager.exe'
```

```text
b79c4cbcce56646f0e00b2640b8bba944c3363379ea8109ca22595f57188b27c  native/tfm2_custom_ai/target/x86_64-pc-windows-gnu/release/tfm2_custom_ai.dll
b79c4cbcce56646f0e00b2640b8bba944c3363379ea8109ca22595f57188b27c  mods/tfm2_custom_ai/tfm2_custom_ai.dll
01780422f34e0b040eb3e299ee0f6a9f6f1227462728f402aa0f093383852e91  TFM2 Mod Manager.exe
```

## Changed files (96)

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
- `docs/unified-theory-peak-curie.png`
- `docs/unified-theory-peak-einstein.png`
- `docs/unified-theory-peak-newton.png`
- `docs/unified-theory-ranked-cosmic.gif`
- `docs/unified-theory-scale.png`
- `docs/unified-theory-skills-curie.png`
- `docs/unified-theory-skills-einstein.png`
- `docs/unified-theory-skills-newton.png`
- `docs/unified-theory-static-cards.png`
- `docs/unified-theory-static-curie.png`
- `docs/unified-theory-static-newton.png`
- `docs/unified-theory-subject-peaks-verification.md`
- `docs/unified-theory-top10.png`
- `docs/unified-theory-transform-frames.png`
- `docs/unified-theory-transforms.gif`
- `editor/science-art-preview.json`
- `editor/science-art.js`
- `editor/science-mastery-preview.png`
- `editor/science-portraits.png`
- `editor/science.html`
- `mods/tfm2_custom/champion/tfm2_custom_unified_theory.data_champion`
- `mods/tfm2_custom/mod.mod_info`
- `mods/tfm2_custom/vfx/science_completion#anim.fanim`
- `mods/tfm2_custom/vfx/science_completion#sheet.png`
- `mods/tfm2_custom/vfx/science_echo#anim.fanim`
- `mods/tfm2_custom/vfx/science_echo#sheet.png`
- `mods/tfm2_custom/vfx/science_equipment0#anim.fanim`
- `mods/tfm2_custom/vfx/science_equipment0#sheet.png`
- `mods/tfm2_custom/vfx/science_equipment1#anim.fanim`
- `mods/tfm2_custom/vfx/science_equipment1#sheet.png`
- `mods/tfm2_custom/vfx/science_equipment2#anim.fanim`
- `mods/tfm2_custom/vfx/science_equipment2#sheet.png`
- `mods/tfm2_custom/vfx/science_fields_t0#anim.fanim`
- `mods/tfm2_custom/vfx/science_fields_t0#sheet.png`
- `mods/tfm2_custom/vfx/science_fields_t1#anim.fanim`
- `mods/tfm2_custom/vfx/science_fields_t1#sheet.png`
- `mods/tfm2_custom/vfx/science_fields_t2#anim.fanim`
- `mods/tfm2_custom/vfx/science_fields_t2#sheet.png`
- `mods/tfm2_custom/vfx/science_fields_t3#anim.fanim`
- `mods/tfm2_custom/vfx/science_fields_t3#sheet.png`
- `mods/tfm2_custom/vfx/science_outfits#sheet.png`
- `mods/tfm2_custom/vfx/science_packets_t0#anim.fanim`
- `mods/tfm2_custom/vfx/science_packets_t0#sheet.png`
- `mods/tfm2_custom/vfx/science_packets_t1#anim.fanim`
- `mods/tfm2_custom/vfx/science_packets_t1#sheet.png`
- `mods/tfm2_custom/vfx/science_packets_t2#sheet.png`
- `mods/tfm2_custom/vfx/science_packets_t3#anim.fanim`
- `mods/tfm2_custom/vfx/science_packets_t3#sheet.png`
- `mods/tfm2_custom/vfx/science_personas#sheet.png`
- `mods/tfm2_custom/vfx/science_skills0_t0#anim.fanim`
- `mods/tfm2_custom/vfx/science_skills0_t0#sheet.png`
- `mods/tfm2_custom/vfx/science_skills0_t1#anim.fanim`
- `mods/tfm2_custom/vfx/science_skills0_t1#sheet.png`
- `mods/tfm2_custom/vfx/science_skills0_t2#anim.fanim`
- `mods/tfm2_custom/vfx/science_skills0_t2#sheet.png`
- `mods/tfm2_custom/vfx/science_skills0_t3#anim.fanim`
- `mods/tfm2_custom/vfx/science_skills0_t3#sheet.png`
- `mods/tfm2_custom/vfx/science_skills1_t0#anim.fanim`
- `mods/tfm2_custom/vfx/science_skills1_t0#sheet.png`
- `mods/tfm2_custom/vfx/science_skills1_t1#anim.fanim`
- `mods/tfm2_custom/vfx/science_skills1_t1#sheet.png`
- `mods/tfm2_custom/vfx/science_skills1_t2#anim.fanim`
- `mods/tfm2_custom/vfx/science_skills1_t2#sheet.png`
- `mods/tfm2_custom/vfx/science_skills1_t3#anim.fanim`
- `mods/tfm2_custom/vfx/science_skills1_t3#sheet.png`
- `mods/tfm2_custom/vfx/science_skills2_t0#anim.fanim`
- `mods/tfm2_custom/vfx/science_skills2_t0#sheet.png`
- `mods/tfm2_custom/vfx/science_skills2_t1#anim.fanim`
- `mods/tfm2_custom/vfx/science_skills2_t1#sheet.png`
- `mods/tfm2_custom/vfx/science_skills2_t2#anim.fanim`
- `mods/tfm2_custom/vfx/science_skills2_t2#sheet.png`
- `mods/tfm2_custom/vfx/science_skills2_t3#anim.fanim`
- `mods/tfm2_custom/vfx/science_skills2_t3#sheet.png`
- `mods/tfm2_custom/vfx/science_transforms#sheet.png`
- `mods/tfm2_custom_ai/mod.mod_info`
- `mods/tfm2_custom_ai/tfm2_custom_ai.dll`
- `native/tfm2_custom_ai/mod.mod_info`
- `native/tfm2_custom_ai/src/lib.rs`
- `native/tfm2_custom_ai/src/unified_theory.rs`
- `native/tfm2_custom_ai/src/unified_theory_tests.rs`
- `tools/verify_science_art_browser.cjs`
- `tools/verify_unified_theory.py`

## In-game checks still required

No real game match is available in this cloud environment. Run Mod Manager **1 Update** and **2 Check**, confirm the new version and clean dependency loading, then check:

- Student through Unified Mind for each subject; distinct #1 appearance at actual game scale and while paused, compared with #2/#3/ordinary Top 10.
- Both team facings; forward gaze, natural stance, visible hand/prop and outfit coverage during movement and attack; aura and compact badge placement/numerals.
- The three eight-frame #1 sequences over 1.6 seconds; the organism reaching Curie's sample; Newton's inversion/shadow; Einstein's creation/alignment.
- All six direct swaps, planted feet, busy selections, respawn, missing-layer restoration and death cleanup without stacked persistent loops.
- All subject skill effects; black holes restricted to Gravity Well and Horizon Ring; final-subject combination completion and delayed #1 imprint.
- Coder's two IDE rows, underscores, rank theme, status/HUD, coding variety, loading and notebook flicker alongside the scientist.
- Crowded fights with scientists, Coder, Levi and Isliid #1; inspect actual FPS, readability and game/performance logs. Automated animation bounds do not establish game FPS.

Round **120** is appended to `docs/playtest-notes.md`. Source, generated assets, previews, release metadata and rebuilt DLL are kept together on `codex/scientist-compact-forms` for PR #24. The PR remains open; merging and real-game installation are separate from the cloud verification above.
