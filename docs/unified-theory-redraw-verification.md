# Forward-gazing scientist redraw — round 117 verification

Native **0.10.30**, champion mod **0.2.30**, dependency **>=0.10.30**; both native metadata copies match `VERSION`. Release dates: **2026-10-10**. Branch: `codex/scientist-compact-forms`, updating [PR #24](https://github.com/PowerViber/tfm2_mods/pull/24).

## Result

The three scientists look slightly toward screen-right along their step. A broad near cheek and visible jaw frame a clear near eye and smaller far eye; the nose stays inside the face outline and the mouth is relaxed. Their chests turn three-quarter, with a forward shoulder, partly concealed far arm, natural elbows and one leading foot nearer the viewer. One-pixel outlines and clustered highlights/shadows follow the original Swordsman and Taoist sprites.

The static sprites were completed and visually inspected on actual-size champion cards before the transformations were adapted. Across **24 outfits × eight frames**, actual silhouettes measure **18–23 px wide and 37–38 px tall**, beside the source Swordsman (37 px) and Taoist (35 px). The 48 × 64 canvas and floor anchor remain.

- **Einstein:** soft white tufts, small moustache, short ivory coat over blue-grey clothing, thoughtful hand near his chest and a relaxed far hand. His cosmic watch is clipped inside the coat.
- **Newton:** silver curls, burgundy historical coat, ivory cuffs, split hem, dark breeches/boots and a red apple held at waist height.
- **Curie:** visible dark bun, fitted ivory laboratory coat, dark skirt/simple shoes, mint vial held ahead and below her face.

Mastery retains folded space, celestial machinery and crystal facets inside the cloth. All seventeen emblems remain <=18 × 18 px. The larger cosmic skill and completion graphics remain.

All **48 rank/route transformations** retain eight distinct frames over **24 ticks / 0.4 seconds**. Hands and held props gather inward; an atom forms; a close pale-blue, gold or mint wrap changes hair, clothing and prop; the destination settles into its natural pose. The forward gaze remains and idle feet stay planted. A single timed body layer, latest-selection behavior, death cleanup and all anti-stacking deadlines remain. No new runtime animation layers or particle emitters are added.

[Static champion cards](unified-theory-static-cards.png) · [Einstein](unified-theory-static-einstein.png) · [Newton](unified-theory-static-newton.png) · [Curie](unified-theory-static-curie.png) · [Both team facings](unified-theory-facing.png) · [Animated transformations](unified-theory-transforms.gif) · [Eight-frame strip](unified-theory-transform-frames.png).

## Verification

Visually inspected the exact source references, actual-size cards, enlarged static forms, mirrored Student/Unified Mind poses and all six transformation strips. The art verifier now compares every frame of all 24 runtime outfits against the neutral moving-leg foundation and checks every morph frame for uncovered default-body pixels. Exact endpoints, eight distinct frames, lifetime, fixed foot pixels, compact badges and deterministic generation remain checked. The four new static preview files are included in byte-identical regeneration.

The width guard now admits the requested 18-pixel Einstein silhouette while retaining the 24-pixel maximum and the 35–40-pixel height range. Exact foot equality is checked below y=55 because Curie's skirt reaches y=54; the previous y=51 region included her newly requested skirt. Whole-body foundation comparisons strengthen coverage checks across all outfit and morph frames.

The attached `coder-verification.md` supplies shared-DLL, release and editor regression checks. Its historical base/version examples do not change the user's scientist design scope. Coder source, vectors, champion data/text, sprites and editor behavior remain unchanged. Scientist combat, resources, charge math, notebook timing and native renderer logic remain. Native source changes only its release version. Manager source and EXE are unchanged.

**Zero compiler warnings** in native Linux, native tests, Windows and manager checks. `every_version_agrees` passes. The one ignored test is the pre-existing Scribble real-host fixture. Existing direct-route, rapid-selection, death cleanup and bounded-renderer tests pass in the complete native suite.

All Cargo/Python checks run after `source /workspace/.tfm2-env/env.sh`, with offline dependencies and the configured MinGW linker.

### Linux release build

`cargo build --offline --release -j4 --manifest-path native/tfm2_custom_ai/Cargo.toml`

```text
Finished `release` profile [optimized] target(s) in 0.01s
```

### Native suite

`cargo test --offline --release -j4 --manifest-path native/tfm2_custom_ai/Cargo.toml`

```text
test version_tests::every_version_agrees ... ok
test result: ok. 170 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.78s
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
Compact art: all eight frames of 24 forward-gazing costumes fit 18–24 x 35–40 px; every rank emblem fits 18 x 18 px; 48 direct transformations have eight distinct frames over 24 ticks with planted feet, covered base pixels and exact endpoints; static champion cards regenerate exactly; idle equipment has no aura.
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
Finished `release` profile [optimized] target(s) in 0.01s
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
4bae1d7f2d5d7c4997ba2efbc69101c5d982426375abb1115af85202a7acbd59  native/tfm2_custom_ai/target/x86_64-pc-windows-gnu/release/tfm2_custom_ai.dll
4bae1d7f2d5d7c4997ba2efbc69101c5d982426375abb1115af85202a7acbd59  mods/tfm2_custom_ai/tfm2_custom_ai.dll
01780422f34e0b040eb3e299ee0f6a9f6f1227462728f402aa0f093383852e91  TFM2 Mod Manager.exe
```

The Windows native DLL was rebuilt for 0.10.30 and copied to `mods/tfm2_custom_ai/tfm2_custom_ai.dll`; build and shipped hashes match. The final cached build checks confirm those same release outputs. Manager source/EXE is unchanged and did not need rebuilding. Round 117 in `docs/playtest-notes.md` records the revision. All revision files are committed and pushed together on PR #24's branch.

## To check in game

Cloud has no real game match available. Test the PR branch, or run Mod Manager Update/Check after merge. Check actual picker cards; both team facings; gaze/chest/leading-foot agreement; hand, prop, costume, skirt and boot coverage during motion/attacks; morphs and respawn; compact badges; and crowded-fight FPS/readability with scientists, Coder, Levi and Isliid #1. The card and mirrored previews show intended art; host mirroring and layer alignment require a game playtest. No measured game-FPS claim is made.

## Changed files in this revision

- `Claude outputs/unified_theory/compact_art.py`
- `docs/playtest-notes.md`
- `docs/unified-theory-art-inspector.png`
- `docs/unified-theory-cosmic.gif`
- `docs/unified-theory-facing.png`
- `docs/unified-theory-guide.md`
- `docs/unified-theory-lab.png`
- `docs/unified-theory-mastery.png`
- `docs/unified-theory-redraw-verification.md`
- `docs/unified-theory-scale.png`
- `docs/unified-theory-silhouettes.png`
- `docs/unified-theory-static-cards.png`
- `docs/unified-theory-static-curie.png`
- `docs/unified-theory-static-einstein.png`
- `docs/unified-theory-static-newton.png`
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
- `tools/verify_unified_theory.py`
