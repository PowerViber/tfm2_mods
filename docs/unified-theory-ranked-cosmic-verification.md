# Cosmic ranked scientists and all skills — round 118 verification

Native **0.10.31**, champion mod **0.2.31**, dependency **>=0.10.31**. Both native metadata copies match `VERSION`; dates are **2026-10-10**. This updates the open [PR #24](https://github.com/PowerViber/tfm2_mods/pull/24) on `codex/scientist-compact-forms`.

## Result

All 24 rank outfits now change their full material and contain eight distinct animation frames: Einstein's ivory becomes folded blue star-fabric; Newton keeps silver curls, burgundy tailoring and an apple around brass celestial machinery; Curie's ivory coat and dark skirt acquire luminous mineral fabric. Heads, forward gaze, three-quarter stance, hand poses, coat hems and shared feet retain the accepted compact anatomy. Actual idle body dimensions remain **18–23 × 37–38 px**. All 17 rank/Top 10 badges remain **<=18 × 18 px**. All 48 direct eight-frame transformations retain exact outfit endpoints and fixed feet over 24 ticks.

Every rank also has a visible cosmic aura. Student begins with a floating seed and ground orbit; Lab Assistant gains crossed atom loops; Researcher opens a portal, orrery or reactor. Scientist and Professor add spatial, mechanical or mineral structures; Fellow and Laureate acquire folded galaxy ribbons, orbital engines, crystal wings and constellations. Top 10 adds a prismatic planetary halo. #3 has three scientific worlds; #2 opens paired wormholes. **#1 assembles the Orrery of Everything:** a suspended void crown, impossible triangular machine, counter-rotating orbits, circulating celestial bodies, crystal matter and a prismatic ground gate. Each complete construction is a single eight-frame background animation within a **96 × 112 px** anchor around the compact body. The face, hands, feet and badge render in front. The three forms each have seven ordinary plus four distinct podium aura variants: **33 loops total**.

All 75 skills retain four eight-frame mastery stages, with distinct scientific actions: prism photons, synchronized clocks, compressed/sheared space, vector projection/addition, collision/trajectory machinery, separation, neutralization and chained decay. Filled surfaces animate galaxies, engraved brass or luminous minerals. Higher stages build solid orbiting frames, separated planes and unified structures. Eleven field types, fourteen moving packet types across eight headings, seven combinations of preparation modifiers and completion/podium constructions follow the same cosmic treatment. Every skill, field, directional packet and preparation modifier has eight distinct frames. Runtime view names/counts remain compatible.

[All ranks](unified-theory-mastery.png) · [Animated ranks](unified-theory-ranked-cosmic.gif) · [Animated #1 auras](unified-theory-auras.gif) · [Eight original aura sprites](unified-theory-aura-frames.png) · [Cosmic scene](unified-theory-cosmic.gif) · [Skill sprite strips](unified-theory-cosmic-frames.png) · [Einstein's 25](unified-theory-skills-einstein.png) · [Newton's 25](unified-theory-skills-newton.png) · [Curie's 25](unified-theory-skills-curie.png).

## References inspected

Inspected Levi's source `Claude outputs/levi/levi_wormhole.py` and actual Apex preview: solid night-sky portal throats, bright rotating rims and spatial ribbons. Inspected Isliid's `tools/isliid_comet_art.py`, native black-hole/aura lifecycle and the shipped `blackhole1_n7`, `wormhole_out_p` and `skylight_rank8_comet_a0` frames: a prismatic void crown, lensing disk and celestial bodies. Scientist structures use those material/depth cues with their own three-discipline machinery. Reference champions and their runtime art are unchanged.

## Animation cost and preserved behavior

The entire cosmic aura is **one** persistent background buff, added once and replaced only when its displayed form, rank or podium changes. It follows the current transformation's destination while the morph is busy; newer notebook selections cannot repeatedly replace it. Missing layers restore individually, stale aura variants are removed, and death clears all cosmetic layers. The body/transformation, badge, aura and optional preparation marks total **at most four persistent loops**. Front equipment tags remain empty. The existing non-overlapping cast/completion channels, three-item pending queue, explicit replay deadlines, coincident field/packet coalescing and six-temporary-start limit per caster/update remain.

Glows, mineral veins, galaxy surfaces and all circulating bodies are baked during asset generation. No runtime particle emitters, shaders or extra independent loops per aura component are introduced. All **36 scientist VFX sheets remain <=2048 × 2048**. The crowded native fixture emits **594 temporary starts over seven seconds**, peaks at **30 temporary live animations**, never exceeds six temporary starts/update, and adds its four persistent layers once. Existing temporary-animation bounds remain in force.

The sum of sheet dimensions at four bytes per pixel changes from **167.55 MiB** to **180.38 MiB** (**+12.83 MiB**) compared with the preceding branch commit. This is a decoded RGBA atlas estimate, not measured game memory or FPS. Trimmed indexed PNGs and packet-pair pixel reuse remain.

Combat handlers, charge allocations, activation speed, cooldowns, damage, RNG consumption and notebook timing are unchanged. Native changes only select the one cosmetic aura within the existing layer lifecycle. Coder source/art/data/vectors/lab and manager source/EXE are unchanged. The attached `coder-verification.md` guides the shared DLL, metadata, manager and editor regression checks; its historical branch/version examples do not change this authorized scientist artwork scope.

## Verification

Visually inspected all ranked outfits and auras, all three 25-skill contact sheets, Top 10 comparisons, original aura/skill frame strips and the runtime browser inspector. The art verifier retains body/badge footprint, base coverage, fixed-foot and exact-morph-endpoint assertions. It adds eight unique rank costumes per scientist, eight distinct stored outfit frames, unchanged head/gaze pixels, bounded eight-phase background auras, distinct podium structures and empty front layers. Runtime assets and every preview regenerate byte-identically. All **375 native charge vectors** still match the lab.

The new native test `cosmic_auras_replace_once_follow_the_morph_and_never_stack` checks all eight ranks, podium replacement, repeated idle updates, rapid selections during a busy morph, missing-aura restoration and death cleanup. Existing cosmetic/RNG parity, layer replacement/restoration, direct-transition and crowded-animation tests remain. Browser checks cover all skills/ranks, all 33 aura manifests, each destination aura during transformations, pause/play, cooldown, all ten leaderboard positions, travel/fields, completion and desktop/mobile. Morphs use a body-only preview tile before drawing the correct destination aura, preventing a stale Student aura from showing through.

Linux, tests and Windows compilation: **0 warnings, 0 errors**. `every_version_agrees` passes. The sole ignored native test remains the pre-existing `scribble::real_data::real_pending` real-host fixture.

### Linux release build

`source /workspace/.tfm2-env/env.sh; cargo build --offline --release -j4 --manifest-path native/tfm2_custom_ai/Cargo.toml`

```text
Compiling tfm2_custom_ai v0.1.0 (/workspace/tfm2_mods/native/tfm2_custom_ai)
    Finished `release` profile [optimized] target(s) in 18.04s
```

### Full native suite

`cargo test --offline --release -j4 --manifest-path native/tfm2_custom_ai/Cargo.toml -- --nocapture`

```text
test unified_theory::integration::all_six_form_changes_are_direct_and_finish_without_stacking ... ok
test unified_theory::integration::cosmic_auras_replace_once_follow_the_morph_and_never_stack ... ok
Cosmic crowded scene: 594 starts over 7 s, peak 30 live temporary animations, max 6 starts/update; persistent loops added once
test unified_theory::integration::visual_layers_are_replaced_restored_and_cleared_without_stacking ... ok
test unified_theory::integration::mastery_art_does_not_change_combat_resources_or_rng ... ok
test version_tests::every_version_agrees ... ok
test result: ok. 171 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.67s
```

### Scientist assets and exact parity

`python tools/verify_unified_theory.py`

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

`python tools/verify_coder.py --local`

```text
Code lab: tables, languages, functions, models and hardware match coder.rs; 558 runs reproduced exactly
Verified 5471 Coder views over 17 sheets (each <= 2048), the rig layers, the top-rank effects, crests and every name coder.rs builds
```

### Manager suite

`cargo test --offline --release -j4 --manifest-path tools/manager/Cargo.toml`

```text
Finished `release` profile [optimized] target(s) in 0.03s
     Running unittests src/main.rs (tools/manager/target/release/deps/tfm2_manager-756fc6bf9fcfdf44)

running 13 tests
test core::tests::build_outputs_reset_before_pull ... ok
test core::tests::champion_data_errors_name_the_bad_tag ... ok
test core::tests::log_scan_finds_load_errors ... ok
test core::tests::native_version_meets_the_champions_requirement ... ok
test core::tests::mods_json_enable ... ok
test core::tests::perf_log_is_read ... ok
test core::tests::install_copies_and_keeps_plans ... ok
test core::tests::stamps_are_dates ... ok
test core::tests::repo_root_found_from_exe_dir ... ok
test core::tests::check_reports_mismatch ... ok
test core::tests::stale_duplicates_go_to_backup ... ok
test core::tests::the_repos_champions_all_load ... ok
test core::tests::vdf_and_manifest_parse ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

### Coder syntax, self-test and every rank

```sh
node -c editor/coderlab.js
node -e "const l=require('./editor/coderlab.js'); console.log(l.selfTest()); for (const [r,p] of l.LADDER) l.simulateSkirmish(r,p,1);"
```

```text
true
```

### Windows release build

`cargo build --offline --release --target x86_64-pc-windows-gnu -j4 --manifest-path native/tfm2_custom_ai/Cargo.toml`

```text
Compiling tfm2_custom_ai v0.1.0 (/workspace/tfm2_mods/native/tfm2_custom_ai)
    Finished `release` profile [optimized] target(s) in 16.55s
```

The sourced environment exports `CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc`. The DLL was copied from the Windows release target into `mods/tfm2_custom_ai/`.

### Science art browser

`NODE_PATH=/workspace/.tfm2-env/browser/node_modules SCIENCE_LAB_URL=http://127.0.0.1:19362/science.html node tools/verify_science_art_browser.cjs`

```text
PASS: 75 actual skill animations across all 8 ranks, 33 eight-frame auras, 10 Top 10 positions, 25 travel/field animations, all six direct transformations with the destination aura, cooldown, same-form guard, paused morph, moving frames, pause/play, completion, desktop/mobile; no script or asset-loading errors.
```

### Science Lab browser

`NODE_PATH=/workspace/.tfm2-env/browser/node_modules SCIENCE_LAB_URL=http://127.0.0.1:19362/science.html node tools/verify_unified_theory_browser.cjs`

```text
PASS: 375 native charge vectors, 75 skill cards, search and role filters, charge imbalance, reservation/cancel, all 50 recipes, 8-rank comparison, desktop/mobile without errors or horizontal overflow.
```

### Coder browser

`NODE_PATH=/workspace/.tfm2-env/browser/node_modules node /tmp/science-art-coder-browser.cjs`

```text
PASS: browser parity, 100 rows, language filtering, picker/rewrite, 4 rank arenas, 10-run comparison, no page errors/NaN.
```

The Coder comparison returned nine numeric rank rows, ascending shipped/distinct counts and no NaN. Science art JavaScript and its browser verification script also pass `node -c`.

### Shipped binary hashes

`sha256sum <Windows build DLL> <shipped DLL> "TFM2 Mod Manager.exe"`

```text
b56c26b1585d0efb389ea4e714d017139ae4250c0a065fe416755c445cda2127  native/tfm2_custom_ai/target/x86_64-pc-windows-gnu/release/tfm2_custom_ai.dll
b56c26b1585d0efb389ea4e714d017139ae4250c0a065fe416755c445cda2127  mods/tfm2_custom_ai/tfm2_custom_ai.dll
01780422f34e0b040eb3e299ee0f6a9f6f1227462728f402aa0f093383852e91  TFM2 Mod Manager.exe
```

Build and shipped DLL hashes match. The manager EXE retains its preceding hash and needs no rebuild because manager source is unchanged. All six automated-check groups in the attached Coder checklist pass. `git diff --check` passes.

## To check in game

The cloud has no game installation, so actual game rendering, team mirroring, layer placement and FPS remain unverified. Test the PR branch before merging, or Update/Check in Mod Manager after merge and confirm native **0.10.31**. Check all three scientists at Student, Scientist, Fellow and Unified Mind, including podium #1: both facings, body/prop coverage while moving or attacking, aura alignment, badges/numerals, all direct swaps and respawn cleanup. Try preparations, fields, orbital/reflected packets, all three capstones and #1 completion. Use a crowded match with two scientists plus Coder, Levi and Isliid #1 to check readability, notebook flicker, FPS and load/performance logs. No in-game FPS result is claimed.

## Changed files

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
- `docs/unified-theory-ranked-cosmic-verification.md`
- `docs/unified-theory-ranked-cosmic.gif`
- `docs/unified-theory-scale.png`
- `docs/unified-theory-skills-curie.png`
- `docs/unified-theory-skills-einstein.png`
- `docs/unified-theory-skills-newton.png`
- `docs/unified-theory-static-cards.png`
- `docs/unified-theory-static-curie.png`
- `docs/unified-theory-static-einstein.png`
- `docs/unified-theory-static-newton.png`
- `docs/unified-theory-top10.png`
- `docs/unified-theory-transform-frames.png`
- `docs/unified-theory-transforms.gif`
- `editor/science-art-preview.json`
- `editor/science-art.js`
- `editor/science-mastery-preview.png`
- `editor/science-portraits.png`
- `editor/science.html`
- `mods/tfm2_custom/champions/tfm2_custom_unified_theory#sheet.png`
- `mods/tfm2_custom/mod.mod_info`
- `mods/tfm2_custom/vfx/science_completion#sheet.png`
- `mods/tfm2_custom/vfx/science_equipment0#anim.fanim`
- `mods/tfm2_custom/vfx/science_equipment0#sheet.png`
- `mods/tfm2_custom/vfx/science_equipment1#anim.fanim`
- `mods/tfm2_custom/vfx/science_equipment1#sheet.png`
- `mods/tfm2_custom/vfx/science_equipment2#anim.fanim`
- `mods/tfm2_custom/vfx/science_equipment2#sheet.png`
- `mods/tfm2_custom/vfx/science_fields_t1#anim.fanim`
- `mods/tfm2_custom/vfx/science_fields_t1#sheet.png`
- `mods/tfm2_custom/vfx/science_fields_t2#anim.fanim`
- `mods/tfm2_custom/vfx/science_fields_t2#sheet.png`
- `mods/tfm2_custom/vfx/science_fields_t3#anim.fanim`
- `mods/tfm2_custom/vfx/science_fields_t3#sheet.png`
- `mods/tfm2_custom/vfx/science_modifiers#sheet.png`
- `mods/tfm2_custom/vfx/science_outfits#sheet.png`
- `mods/tfm2_custom/vfx/science_packets_t0#sheet.png`
- `mods/tfm2_custom/vfx/science_packets_t1#sheet.png`
- `mods/tfm2_custom/vfx/science_packets_t2#anim.fanim`
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
