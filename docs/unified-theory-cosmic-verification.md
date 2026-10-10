# Cosmic Unified Experiment — round 113 verification

Native **0.10.26**, champion mod **0.2.26**, dependency **>=0.10.26**. Both native metadata copies match `VERSION`; release dates are **2026-10-10**. This updates the existing, open Unified Theory PR #23.

The scientist now uses solid celestial structures and eight-frame motion: Einstein's dark accretion wells, folded tunnels and travelling portal rings; Newton's rotating brass keystones, stellar cores and planetary machinery; Curie's growing faceted crystals, radioactive reactors and nebulae. Every one of the 75 skills has four mastery variants. High-rank equipment becomes an orbiting universe around the same recognizable body; #1 has a three-discipline cosmic crown. Large activations and completion render behind the body.

[Animated scene](unified-theory-cosmic.gif) · [Eight original sprites](unified-theory-cosmic-frames.png) · [Top 10](unified-theory-top10.png) · [All ranks](unified-theory-mastery.png) · [Skill stages](unified-theory-effects.png) · [Science Lab inspector](unified-theory-art-inspector.png).

## Reference inspection and lifecycle fix

Inspected Levi's `levi_wormhole.py`, Apex sprite preview, native Levi visuals, and Isliid's #1 comet/black-hole artwork plus `update_visuals`, `comet_tag`, `visual_for` and crowded-fight tests. Isliid uses short frame pairs with replay deadlines instead of repeatedly spawning full loops.

The prior scientist field animation lasted 1.68 seconds: anchors could replay every 12 ticks (0.2 seconds), and fields every 36 ticks (0.6 seconds). That stacked full animations across updates even with six requests per tick.

- Fields now play eight frames in 48 ticks, once per replay deadline. Final playback is clipped to the remaining object lifetime; coincident field/anchor surfaces share one visual. Recently replaced surfaces retain their deadline, with a bounded 24-entry cache.
- Moving packets cycle through four pairs of their eight sprites, one 12-tick pair at a time. Eight headings match Isliid's approach; the physics direction remains unchanged. Coincident split packets coalesce visually while retaining every combat packet and payload.
- Cast and completion channels each have at most one active animation. A three-item pending queue bounds delayed artwork; one busy channel cannot block the other.
- Notebook and shadow redraw lifetimes equal their eight-tick refresh cadence. Persistent outfit/equipment/emblem/preparation loops remain in place instead of restarting each tick.
- Glows, nebulae and particles are baked into sprites. Symmetric per-animation trimming keeps all frames centred; indexed PNGs and gutters reduce atlas overhead without adding runtime shaders or emitters.

All 35 scientist VFX sheets remain at most 2048 × 2048. Estimated decoded RGBA texture footprint: **175.6 MiB**, down from **188.3 MiB** at `54a049d`; PNG files total **5.7 MiB**. This is an asset-size estimate, not a measurement of game GPU allocation or FPS.

## Automated evidence

Four new native integration tests exercise actual renderer calls: crowded world expiry/replay overlap, rapid cast channel overlap, eight identical split packets sharing one full cycle, and live animation counts in a full cosmic scene. Existing tests still verify visual-loop retention/replacement/restoration/death cleanup, all art names, orbital direction, completion rules, and unchanged combat/resources/RNG between podium positions. Champion data outside the view tables is byte-equivalent as parsed JSON; Coder sources, data, locale, vectors, art, lab and manager sources/EXE are unchanged.

The verifier checks eight distinct **stored** sprite frames after palette conversion and trimming, stable frame origins, all pair aliases referencing the original eight rectangles, replay durations against native constants, all atlas bounds, and byte-identical regeneration. The 375 scientific charge vectors still match the lab.

### Linux native build

`cargo build --offline --release -j4 --manifest-path native/tfm2_custom_ai/Cargo.toml`

```text
Finished `release` profile [optimized] target(s) in 0.00s
```

### Native tests

`cargo test --offline --release -j4 --manifest-path native/tfm2_custom_ai/Cargo.toml`

```text
test version_tests::every_version_agrees ... ok
test result: ok. 168 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.71s
```

### Crowded scene

`cargo test --offline --release -j4 --manifest-path native/tfm2_custom_ai/Cargo.toml full_cosmic_scene -- --nocapture`

```text
Cosmic crowded scene: 594 starts over 7 s, peak 30 live temporary animations, max 6 starts/update; persistent loops added once
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 168 filtered out; finished in 0.00s
```

### Scientific art and charge parity

`python tools/verify_unified_theory.py`

```text
375 exact native charge vectors; all 8 ranks deterministic
75 unique skills; 50 bounded recipes; generated files current
Cosmic Unified Experiment: 17 badges, 24 outfits, 300 eight-frame casts, 44 eight-frame fields, 448 eight-frame packet loops with shared pair aliases
Original body, three personas, notebooks, meter and mastery art generated
tfm2_custom_unified_theory 3349 effects 138 persona / mastery overlays
Verified 3488 views over 35 VFX sheets; original body <=2048; notebook lifetimes; regeneration byte-identical.
Art: 300 casts, 44 fields and 448 directional packet loops have eight distinct frames; 1792 pair aliases share their atlas pixels; lifetimes match native replay deadlines; cosmic podium previews verified.
```

### Coder checklist

`python tools/verify_coder.py --local; node -c editor/coderlab.js; node selfTest and every LADDER rank`

```text
Code lab: tables, languages, functions, models and hardware match coder.rs; 558 runs reproduced exactly
Verified 5471 Coder views over 17 sheets (each <= 2048), the rig layers, the top-rank effects, crests and every name coder.rs builds
true
```

### Manager tests

`cargo test --offline --release -j4 --manifest-path tools/manager/Cargo.toml`

```text
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
```

### Windows native build

`cargo build --offline --release --target x86_64-pc-windows-gnu -j4 --manifest-path native/tfm2_custom_ai/Cargo.toml`

```text
Compiling tfm2_custom_ai v0.1.0 (/workspace/tfm2_mods/native/tfm2_custom_ai)
    Finished `release` profile [optimized] target(s) in 15.65s
```

### Browser checks

`verify_science_art_browser.cjs; verify_unified_theory_browser.cjs; existing Coder browser checks on port 19362`

```text
PASS: 75 actual skill animations across all 8 ranks, 10 Top 10 positions, 25 travel/field animations, moving frames, pause/play, completion, desktop/mobile; no script or asset-loading errors.
PASS: 375 native charge vectors, 75 skill cards, search and role filters, charge imbalance, reservation/cancel, all 50 recipes, 8-rank comparison, desktop/mobile without errors or horizontal overflow.
PASS: browser parity, 100 rows, language filtering, picker/rewrite, 4 rank arenas, 10-run comparison, no page errors/NaN.
```

Linux/Windows builds and native/manager tests have **zero compiler warnings**. The single ignored native test is the existing Scribble pending real-data test. JavaScript syntax checks pass. Browser checks use Chromium and the checkout's own runtime PNG route; there are no script errors, failed asset loads or mobile overflow.

## Shipped binary and changed files

Built and shipped DLL SHA256 (identical):

```text
f326e0b7dd7a942cd60a4bf72dcae7d6595c9671cab71560645157d734fcf115
```

The manager EXE was not rebuilt because its source did not change. Its SHA256 remains:

```text
01780422f34e0b040eb3e299ee0f6a9f6f1227462728f402aa0f093383852e91
```

- Asset generators: `Claude outputs/unified_theory/cosmic_art.py` (new), `mastery_art.py`, `art.py`, `data.py`; regenerated scientist animation metadata/PNGs and view tables.
- Native renderer and tests: `unified_theory.rs`, `unified_theory_tests.rs`; `lib.rs` version and both native `mod.mod_info` copies; champion mod version/dependency; rebuilt shipped DLL.
- Science Lab: `science-art.js`, `science.html`, `science.css`, generated art manifest/composite/portraits; extended browser and asset verifiers.
- Documentation: current player guide, round 113 playtest notes, this report, runtime artwork previews and new animated GIF/eight-frame strips.

## Still requires an actual game match

No real TFM2 match is available in cloud. Check mod loading/version compatibility; both facings and badge placement; casts behind moving bodies; scientist swaps, respawns and loop cleanup; visible eight-frame motion; curved/reflected/orbiting packet headings; crystal destruction and short cosmetic tails after geometry disappears; real Top 10 position and #1 crown/completion/shadow. Check notebook readability and FPS in a crowded match with two scientists, Isliid #1, Levi and Coder. The fixture's 30-animation peak is not an in-game FPS claim. No damage, charge, resource, mastery, targeting, or execution-speed balance changes are part of this follow-up.
