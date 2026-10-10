# Compact scientist forms — round 114 verification

Native **0.10.27**, champion mod **0.2.27**, native dependency **>=0.10.27**; both metadata copies and `VERSION` agree. Dates: **2026-10-10**. Based on merged PR #23 on `main`, with a fresh `codex/scientist-compact-forms` branch.

The wide generic scientist is replaced by Einstein in the champion picker and three matching compact forms in matches. Visible idle bounds are **25–26 × 46–47 pixels** on the existing 48 × 64 anchor. Faces are smaller, shoulders narrow, arms relaxed. Einstein wears a short light coat with white hair, moustache and watch; Newton has silver shoulder-length hair, a dark historical split coat and apple; Curie has a bun, fitted lab coat and green vial. All eight ranks retain these identities, adding confined tailoring details. The large idle laboratory/aura is removed; numbered mastery emblems and cosmic combat effects remain.

[Actual-size reference comparison](unified-theory-scale.png) · [Silhouettes](unified-theory-silhouettes.png) · [Eight-frame strip](unified-theory-transform-frames.png) · [All six transitions animated](unified-theory-transforms.gif) · [Updated inspector](unified-theory-art-inspector.png).

## Transformation and efficiency

One common drawing routine produces all six direct source/target transitions at all eight ranks: **48 animations**, each **eight distinct frames × 50 ms = 0.4 s / 24 ticks**. Hands gather, an atom forms, target-coloured light wraps the body, then the chosen hair/clothes/prop settle. Start/end sprites match the outfits exactly; the shared idle feet do not move between frames. A native transition replaces the outfit rather than adding a second body layer. It starts once, has a timed expiry, and a rapid selection retains only the latest destination. Death clears its state and layer. The athlete chooses the scientist for the next notebook stage; Science Lab allows arbitrary source/target selections. There is no mandatory form cycle or human form hotkey added to the management game.

The 24-tick visual cooldown does not delay notebook tokens/commits, skill cooldowns or resource use. Existing combat replay deadlines, coalescing, six temporary VFX requests per update and bounded cast/completion channels remain. High-rank idle graphics now have three persistent named layers when preparation marks are active (body, emblem, marks), instead of five. The second same-update costume synchronization after commitment was removed: the regular update owns the visual change.

All **36 VFX sheets** fit <=2048 × 2048. Estimated decoded RGBA footprint is **167.5 MiB**, compared with **175.6 MiB** before this change; PNGs total **5.51 MiB**. These are asset-size estimates, not game FPS/GPU measurements. Legacy gear view names retain empty, tiny atlas frames, so view references stay valid without requesting idle gear.

## Verification

Two new native tests cover all six routes, one active body layer, retained loops, exact 24-tick completion, unchanged charge/RNG/cooldowns, rapid selections and death cleanup. Existing all-skill/all-recipe, combat/resource/RNG, renderer overlap/expiry, completion, version and Coder tests pass. The art verifier checks actual stored sprite sizes, fixed idle feet, eight distinct transformation frames, exact endpoints, original-base coverage and byte-identical regeneration. Browser tests exercise all six choices, same-form rejection, cooldown, paused transformations and final destination.

The attached `coder-verification.md` is used for the shared native DLL, version files, assets and editor regression checks. Its historical version/PR examples do not override this authorized scientist redesign. Coder sources, vectors, art, champion mechanics and localization remain unchanged. Parsed scientist champion data outside its view lists also remains unchanged.

All commands use the environment installed during onboarding (`source /workspace/.tfm2-env/env.sh`). Compiler warnings: **0** for Linux, tests and Windows. The ignored native test is the existing Scribble real-host fixture.

### Native Linux build

`cargo build --offline --release -j4 --manifest-path native/tfm2_custom_ai/Cargo.toml`

```text
Compiling tfm2_custom_ai v0.1.0 (/workspace/tfm2_mods/native/tfm2_custom_ai)
    Finished `release` profile [optimized] target(s) in 18.95s
```

### Native tests

`cargo test --offline --release -j4 --manifest-path native/tfm2_custom_ai/Cargo.toml`

```text
test result: ok. 170 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.93s
```

### Crowded renderer

`cargo test --offline --release -j4 --manifest-path native/tfm2_custom_ai/Cargo.toml full_cosmic_scene -- --nocapture`

```text
Cosmic crowded scene: 594 starts over 7 s, peak 30 live temporary animations, max 6 starts/update; persistent loops added once
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 170 filtered out; finished in 0.00s
```

### Scientist art and parity

`python tools/verify_unified_theory.py`

```text
375 exact native charge vectors; all 8 ranks deterministic
75 unique skills; 50 bounded recipes; generated files current
Cosmic Unified Experiment: 17 badges, 24 outfits, 300 eight-frame casts, 44 eight-frame fields, 448 eight-frame packet loops with shared pair aliases
Original body, three personas, notebooks, meter and mastery art generated
tfm2_custom_unified_theory 3349 effects 186 persona / mastery overlays
Verified 3536 views over 36 VFX sheets; original body <=2048; notebook lifetimes; regeneration byte-identical.
Art: 300 casts, 44 fields and 448 directional packet loops have eight distinct frames; 1792 pair aliases share their atlas pixels; lifetimes match native replay deadlines; cosmic podium previews verified.
Compact art: 24 costumes fit 24–30 x 40–48 px; 48 direct transformations have eight distinct frames over 24 ticks with planted feet and exact endpoints; idle equipment has no aura.
```

### Coder regression

`python tools/verify_coder.py --local`

```text
Code lab: tables, languages, functions, models and hardware match coder.rs; 558 runs reproduced exactly
Verified 5471 Coder views over 17 sheets (each <= 2048), the rig layers, the top-rank effects, crests and every name coder.rs builds
```

### Manager tests

`cargo test --offline --release -j4 --manifest-path tools/manager/Cargo.toml`

```text
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

### Windows DLL

`cargo build --offline --release --target x86_64-pc-windows-gnu -j4 --manifest-path native/tfm2_custom_ai/Cargo.toml`

```text
Compiling tfm2_custom_ai v0.1.0 (/workspace/tfm2_mods/native/tfm2_custom_ai)
    Finished `release` profile [optimized] target(s) in 18.03s
```

### Science art browser

`NODE_PATH=/workspace/.tfm2-env/browser/node_modules SCIENCE_LAB_URL=http://127.0.0.1:19362/science.html node tools/verify_science_art_browser.cjs`

```text
PASS: 75 actual skill animations across all 8 ranks, 10 Top 10 positions, 25 travel/field animations, all six direct transformations, cooldown, same-form guard, paused morph, moving frames, pause/play, completion, desktop/mobile; no script or asset-loading errors.
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

### JavaScript syntax and Coder ladder

`node -c editor/coderlab.js`; `node -c editor/science-art.js`; `node -e "const l=require('./editor/coderlab.js'); console.log(l.selfTest()); for (const [r,p] of l.LADDER) l.simulateSkirmish(r,p,1);"`

```text
true
```

### Binary hashes

`sha256sum` for the Windows build, shipped copy and unchanged manager EXE:

```text
68c9142755d7b081d3d6588104065e0da0f30cbbb38ff131807c4154d55ca591  native/tfm2_custom_ai/target/x86_64-pc-windows-gnu/release/tfm2_custom_ai.dll
68c9142755d7b081d3d6588104065e0da0f30cbbb38ff131807c4154d55ca591  mods/tfm2_custom_ai/tfm2_custom_ai.dll
01780422f34e0b040eb3e299ee0f6a9f6f1227462728f402aa0f093383852e91  TFM2 Mod Manager.exe
```

The manager source and EXE are unchanged, so no manager rebuild is required.

## Real-game checks still required

No game match is available in cloud. Check champion-picker scale and Einstein default; both facings; head/coat/prop coverage during moving and attacking; all six direct swaps and their ground position; cooldown/rapid selections; death/respawn layer cleanup; emblem placement; notebook readability and FPS in a crowded match with two scientists, Coder, Levi and Isliid #1. Test this PR branch before merging, or use Mod Manager Update after it is merged; confirm Check reports UP TO DATE and no dependency/load warnings, then inspect scientist/Coder/performance logs. The existing large combat effects are retained, so measured game FPS remains a playtest result.

## Changed files

- `Claude outputs/unified_theory/art.py`
- `Claude outputs/unified_theory/compact_art.py`
- `Claude outputs/unified_theory/data.py`
- `Claude outputs/unified_theory/mastery_art.py`
- `docs/playtest-notes.md`
- `docs/unified-theory-art-inspector.png`
- `docs/unified-theory-compact-verification.md`
- `docs/unified-theory-cosmic.gif`
- `docs/unified-theory-guide.md`
- `docs/unified-theory-lab.png`
- `docs/unified-theory-mastery.png`
- `docs/unified-theory-scale.png`
- `docs/unified-theory-silhouettes.png`
- `docs/unified-theory-top10.png`
- `docs/unified-theory-transform-frames.png`
- `docs/unified-theory-transforms.gif`
- `editor/science-art-preview.json`
- `editor/science-art.js`
- `editor/science-mastery-preview.png`
- `editor/science-portraits.png`
- `editor/science.html`
- `mods/tfm2_custom/champion/tfm2_custom_unified_theory.data_champion`
- `mods/tfm2_custom/champions/tfm2_custom_unified_theory#sheet.png`
- `mods/tfm2_custom/mod.mod_info`
- `mods/tfm2_custom/vfx/science_equipment0#anim.fanim`
- `mods/tfm2_custom/vfx/science_equipment0#sheet.png`
- `mods/tfm2_custom/vfx/science_equipment1#anim.fanim`
- `mods/tfm2_custom/vfx/science_equipment1#sheet.png`
- `mods/tfm2_custom/vfx/science_equipment2#anim.fanim`
- `mods/tfm2_custom/vfx/science_equipment2#sheet.png`
- `mods/tfm2_custom/vfx/science_outfits#sheet.png`
- `mods/tfm2_custom/vfx/science_personas#sheet.png`
- `mods/tfm2_custom/vfx/science_transforms#anim.fanim`
- `mods/tfm2_custom/vfx/science_transforms#sheet.png`
- `mods/tfm2_custom_ai/mod.mod_info`
- `mods/tfm2_custom_ai/tfm2_custom_ai.dll`
- `native/tfm2_custom_ai/mod.mod_info`
- `native/tfm2_custom_ai/src/lib.rs`
- `native/tfm2_custom_ai/src/unified_theory.rs`
- `native/tfm2_custom_ai/src/unified_theory_tests.rs`
- `tools/verify_science_art_browser.cjs`
- `tools/verify_unified_theory.py`
