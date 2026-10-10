# Three-quarter scientist stance — round 116 verification

Native **0.10.29**, champion mod **0.2.29**, dependency **>=0.10.29**; both native metadata copies match `VERSION`. Release dates: **2026-10-10**. Branch: `codex/scientist-compact-forms`, updating open [PR #24](https://github.com/PowerViber/tfm2_mods/pull/24).

## Result

Einstein, Newton and Curie use a three-quarter stance matching the reference champions: torso partly toward the viewer, head angled toward combat, a visible near hand and a far hand concealed by the coat. The near hand holds each scientist's prop. Shared short legs have bent knees, staggered feet and compact boots. Across all **24 costumes × eight frames**, their actual bounds are **20–21 px wide and 39–40 px tall**, close to Swordsman's 37 px and Taoist's 35 px heights. The 48 × 64 anchor is retained.

The transformation's hands, atom and close light wrap follow the revised chest. All **48 rank/route variants** remain eight frames over **24 ticks / 0.4 seconds**, with planted idle feet and exact outfit endpoints. The existing single timed body layer, latest-selection behavior, death cleanup and anti-stacking renderer remain. The small cosmic coat/prop animations, all seventeen emblems (<=18 × 18 px) and larger cosmic combat artwork retain their identities. This pose revision adds no animation channels or runtime particles.

[Scale comparison](unified-theory-scale.png) · [Both mirrored poses](unified-theory-facing.png) · [Silhouettes](unified-theory-silhouettes.png) · [All direct transformations](unified-theory-transforms.gif) · [Eight-frame strip](unified-theory-transform-frames.png).

## Verification

Visually inspected the original Swordsman/Taoist sprites, the actual-size and enlarged scientist forms, both mirrored Student/Unified Mind poses and all six transformation strips. The art verifier checks selected-costume coverage over the default body, compact costume/emblem bounds, every morph's eight distinct frames, exact endpoints, planted idle feet and byte-identical regeneration. An additional measurement covers every frame of all 24 outfits and confirms the actual 20–21 × 39–40 px bounds.

The attached `coder-verification.md` supplies shared-DLL, release and editor regression checks. Its historical base/version examples do not replace the user's authorized scientist design scope. Coder source, vectors, data, text, sprites and editor behavior remain unchanged. Manager source/EXE is unchanged. Scientist combat, charge, notebook timing and visual lifecycle logic remain; native code changes only its release version.

**Zero compiler warnings** in native Linux, native tests and Windows builds. `every_version_agrees` passes. The one ignored test is the pre-existing Scribble real-host fixture. Existing direct-route, rapid-selection, death cleanup and bounded-renderer tests pass in the complete suite.

All Cargo and Python checks run after `source /workspace/.tfm2-env/env.sh`; offline dependencies and the configured MinGW linker are used.

### Linux release build

`cargo build --offline --release -j4 --manifest-path native/tfm2_custom_ai/Cargo.toml`

```text
Compiling tfm2_custom_ai v0.1.0 (/workspace/tfm2_mods/native/tfm2_custom_ai)
    Finished `release` profile [optimized] target(s) in 19.94s
```

### Native suite

`cargo test --offline --release -j4 --manifest-path native/tfm2_custom_ai/Cargo.toml`

```text
test version_tests::every_version_agrees ... ok
test result: ok. 170 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.96s
```

### Scientist art and charge parity

`python tools/verify_unified_theory.py`

```text
375 exact native charge vectors; all 8 ranks deterministic
75 unique skills; 50 bounded recipes; generated files current
Cosmic Unified Experiment: 17 badges, 24 outfits, 300 eight-frame casts, 44 eight-frame fields, 448 eight-frame packet loops with shared pair aliases
Original body, three personas, notebooks, meter and mastery art generated
tfm2_custom_unified_theory 3349 effects 186 persona / mastery overlays
Verified 3536 views over 36 VFX sheets; original body <=2048; notebook lifetimes; regeneration byte-identical.
Art: 300 casts, 44 fields and 448 directional packet loops have eight distinct frames; 1792 pair aliases share their atlas pixels; lifetimes match native replay deadlines; cosmic podium previews verified.
Compact art: 24 battlefield-facing costumes fit 20–24 x 35–40 px; every rank emblem fits 18 x 18 px; 48 direct transformations have eight distinct frames over 24 ticks with planted feet and exact endpoints; idle equipment has no aura.
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
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

### Coder syntax, self-test and every ladder rank

`node -c editor/coderlab.js; node -e "const l=require('./editor/coderlab.js'); console.log(l.selfTest()); for (const [r,p] of l.LADDER) l.simulateSkirmish(r,p,1);"`

```text
true
```

### Windows release build

`cargo build --offline --release --target x86_64-pc-windows-gnu -j4 --manifest-path native/tfm2_custom_ai/Cargo.toml`

```text
Compiling tfm2_custom_ai v0.1.0 (/workspace/tfm2_mods/native/tfm2_custom_ai)
    Finished `release` profile [optimized] target(s) in 19.66s
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

### Shipped binary hashes

`sha256sum native/tfm2_custom_ai/target/x86_64-pc-windows-gnu/release/tfm2_custom_ai.dll mods/tfm2_custom_ai/tfm2_custom_ai.dll "TFM2 Mod Manager.exe"`

```text
8a620302d4e340bf3e38f3f532137581782fefbcd7c9ccdbb3ca4831d25abbef  native/tfm2_custom_ai/target/x86_64-pc-windows-gnu/release/tfm2_custom_ai.dll
8a620302d4e340bf3e38f3f532137581782fefbcd7c9ccdbb3ca4831d25abbef  mods/tfm2_custom_ai/tfm2_custom_ai.dll
01780422f34e0b040eb3e299ee0f6a9f6f1227462728f402aa0f093383852e91  TFM2 Mod Manager.exe
```

The Windows DLL was rebuilt and copied to `mods/tfm2_custom_ai/tfm2_custom_ai.dll`; the hashes match. Manager source and EXE were not changed, so no manager rebuild was needed. Round 116 in `docs/playtest-notes.md` records this revision. All revision files are committed and pushed on the same PR branch; no changes are intended after merge.

## To check in game

No real game match is available in cloud. Test the PR branch before merging, or use Mod Manager Update and Check after merge. Confirm both team facings, the hidden far hand, near-hand props, bent-leg stance during movement/attack, transformation feet/coverage, respawn cleanup, compact badges and crowded-fight FPS/readability with scientists, Coder, Levi and Isliid #1. Mirrored sprite previews show intended orientation; actual host mirroring and layer alignment still require a game playtest. No measured in-game FPS claim is made.

## Changed files in this revision

- `Claude outputs/unified_theory/compact_art.py`
- `docs/playtest-notes.md`
- `docs/unified-theory-art-inspector.png`
- `docs/unified-theory-cosmic.gif`
- `docs/unified-theory-facing.png`
- `docs/unified-theory-guide.md`
- `docs/unified-theory-lab.png`
- `docs/unified-theory-mastery.png`
- `docs/unified-theory-pose-verification.md`
- `docs/unified-theory-scale.png`
- `docs/unified-theory-silhouettes.png`
- `docs/unified-theory-top10.png`
- `docs/unified-theory-transform-frames.png`
- `docs/unified-theory-transforms.gif`
- `editor/science-mastery-preview.png`
- `editor/science-portraits.png`
- `mods/tfm2_custom/champions/tfm2_custom_unified_theory#sheet.png`
- `mods/tfm2_custom/mod.mod_info`
- `mods/tfm2_custom/vfx/science_outfits#sheet.png`
- `mods/tfm2_custom/vfx/science_personas#sheet.png`
- `mods/tfm2_custom/vfx/science_transforms#sheet.png`
- `mods/tfm2_custom_ai/mod.mod_info`
- `mods/tfm2_custom_ai/tfm2_custom_ai.dll`
- `native/tfm2_custom_ai/mod.mod_info`
- `native/tfm2_custom_ai/src/lib.rs`
