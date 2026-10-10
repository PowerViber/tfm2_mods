# Scientist profiles and cosmic costumes — round 115 verification

Native **0.10.28**, champion mod **0.2.28**, dependency **>=0.10.28**; both metadata copies match `VERSION`. Release dates: **2026-10-10**. This updates the open PR #24 on `codex/scientist-compact-forms`.

## Result

The scientists face into the battlefield, with a single visible eye, profile nose, rear hair, asymmetric lapels and forward-pointing shoes. Their height falls from 46–47 px to **39–40 px**, close to the source Swordsman (**37 px**) and Taoist (**35 px**) idle frames. Idle width is **23–24 px** across ranks. The shared 48 × 64 anchor and ground line remain.

- **Einstein:** windswept white hair, projecting moustache, worn cream jacket and blue relativity scarf; animated pocket singularity and a folded, star-filled coat seam at higher ranks.
- **Newton:** silver curls, white cravat, historical tailcoat and a small Principia book; red apple with an amber orbit, brass arm details and a celestial machine inside the coat.
- **Curie:** rear bun, fitted light jacket over a dark teal dress; contained crystal vial and a crystalline coat panel at higher ranks.

Cosmic motion is drawn into the existing eight-frame costume/prop atlas. It adds no runtime particles or new animation channels. All seventeen rank/Top 10 emblems are redrawn at native pixels, fitting **18 × 18 px or smaller** in every frame. Three-by-five-pixel numerals remain legible, including #10. Podium signatures and #1's small crown are retained within that bound.

The common eight-frame transformation is redrawn around the smaller profiles for all six routes and eight ranks. It remains 24 ticks / 0.4 s, with exact outfit endpoints and planted idle feet. The existing single timed body layer, latest-selection rule, death cleanup and anti-stacking renderer are unchanged. Large cosmic skill/completion artwork remains.

[Scale comparison](unified-theory-scale.png) · [Both facings and cosmic progression](unified-theory-facing.png) · [Silhouettes](unified-theory-silhouettes.png) · [All six transformations](unified-theory-transforms.gif) · [Frame strip](unified-theory-transform-frames.png).

## Verification

Visually inspected the repository's Swordsman/Taoist idle frames and both mirrored scientist poses at Student and Unified Mind. The art verifier checks the revised compact bounds for all 24 costumes, every emblem's footprint across all frames/positions, base-art coverage, all 48 eight-frame morphs, fixed idle feet, exact endpoints and deterministic regeneration. The height assertion follows the user's revised request to match other champions; the previous 40–48 px target is superseded.

Scientist champion data is unchanged as parsed JSON. Coder sources, vectors, data, text, sprites, editor code and manager source/EXE remain unchanged. The attached `coder-verification.md` is used for shared-DLL, metadata and editor regressions; its historical PR/version examples do not alter the authorized design scope. All below commands run after `source /workspace/.tfm2-env/env.sh`.

Compiler warnings: **0** in native Linux, tests and Windows builds. `every_version_agrees` passes. The one ignored test remains the pre-existing Scribble real-host fixture. All renderer and direct-transition lifecycle tests remain in the full native suite.

### Linux build

`cargo build --offline --release -j4 --manifest-path native/tfm2_custom_ai/Cargo.toml`

```text
Compiling tfm2_custom_ai v0.1.0 (/workspace/tfm2_mods/native/tfm2_custom_ai)
    Finished `release` profile [optimized] target(s) in 21.20s
```

### Native suite

`cargo test --offline --release -j4 --manifest-path native/tfm2_custom_ai/Cargo.toml`

```text
test result: ok. 170 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.56s
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

### Windows build

`cargo build --offline --release --target x86_64-pc-windows-gnu -j4 --manifest-path native/tfm2_custom_ai/Cargo.toml`

```text
Compiling tfm2_custom_ai v0.1.0 (/workspace/tfm2_mods/native/tfm2_custom_ai)
    Finished `release` profile [optimized] target(s) in 15.87s
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

### Coder syntax, self-test and every ladder rank

`node -c editor/coderlab.js`; `node -e "const l=require('./editor/coderlab.js'); console.log(l.selfTest()); for (const [r,p] of l.LADDER) l.simulateSkirmish(r,p,1);"`

```text
true
```

### Shipped binary hashes

`sha256sum` for the Windows build, shipped copy and unchanged manager:

```text
2f97bc75dc08a855906c0460afb26662d644622cbf45189a88790760fb25cb47  native/tfm2_custom_ai/target/x86_64-pc-windows-gnu/release/tfm2_custom_ai.dll
2f97bc75dc08a855906c0460afb26662d644622cbf45189a88790760fb25cb47  mods/tfm2_custom_ai/tfm2_custom_ai.dll
01780422f34e0b040eb3e299ee0f6a9f6f1227462728f402aa0f093383852e91  TFM2 Mod Manager.exe
```

The Windows DLL was rebuilt and copied; its hash matches. Manager source/EXE is unchanged, so no manager rebuild is needed.

## To check in game

No real game match is available in cloud. Test the PR branch before merging, or use Mod Manager Update after merge. Check the picker portrait, both team facings, costume/prop coverage during motion and attacks, all direct transformations and their foot anchor, respawn cleanup, smaller badge placement/digit readability, and FPS/readability in crowded fights with scientists, Coder, Levi and Isliid #1. The mirrored preview demonstrates the intended art orientation; actual host mirroring and runtime layer alignment require a game playtest. No game FPS measurement is claimed.

## Changed files

- `Claude outputs/unified_theory/compact_art.py`
- `Claude outputs/unified_theory/mastery_art.py`
- `docs/playtest-notes.md`
- `docs/unified-theory-art-inspector.png`
- `docs/unified-theory-cosmic.gif`
- `docs/unified-theory-facing-verification.md`
- `docs/unified-theory-facing.png`
- `docs/unified-theory-guide.md`
- `docs/unified-theory-lab.png`
- `docs/unified-theory-mastery.png`
- `docs/unified-theory-scale.png`
- `docs/unified-theory-silhouettes.png`
- `docs/unified-theory-top10.png`
- `docs/unified-theory-transform-frames.png`
- `docs/unified-theory-transforms.gif`
- `editor/science-mastery-preview.png`
- `editor/science-portraits.png`
- `mods/tfm2_custom/champions/tfm2_custom_unified_theory#sheet.png`
- `mods/tfm2_custom/mod.mod_info`
- `mods/tfm2_custom/vfx/science_badges#sheet.png`
- `mods/tfm2_custom/vfx/science_outfits#sheet.png`
- `mods/tfm2_custom/vfx/science_personas#sheet.png`
- `mods/tfm2_custom/vfx/science_transforms#sheet.png`
- `mods/tfm2_custom_ai/mod.mod_info`
- `mods/tfm2_custom_ai/tfm2_custom_ai.dll`
- `native/tfm2_custom_ai/mod.mod_info`
- `native/tfm2_custom_ai/src/lib.rs`
- `tools/verify_unified_theory.py`
