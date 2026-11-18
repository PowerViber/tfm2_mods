# Contender: approved Shadow Boxer artwork and real-map Lab

**Current extension:** [Contender 2.2](../contender-flurry-review/README.md) adds eight incoming Flurry directions using the approved fist material. Current decoded artwork is 5.18 MiB; the original 2.0 measurements and approvals below remain historical evidence.

18 November 2026. Based on PR #91 head `dc0e0b7b2de8a326e80ff9d7fd5d226a9fd423a1`.
Reserved versions remain native **0.10.113**, champion pack **0.2.116** and rules **contender-2.0**. Main still uses 0.10.112 / 0.2.115 at publication.

The user explicitly requested replacing the old ring-boxer appearance with a shadow boxer. The body approval was **“Approve this shadow-boxer appearance”**; the separate effects approval was **“Approve these shadow effects”**. Both are recorded with frozen source hashes in [approvals.json](approvals.json). No repeated appearance approval is needed for these completed animations.

## Delivered presentation

- Original hooded indigo/plum boxer, violet sash, ivory wraps and shadow fists; connected whole-body drawings in both facings. Six distinct movement drawings, punches, projection, Cyclone, slam/uppercut, recoil and death.
- Visible real roster idle. All existing animation names remain, including legacy aliases. Canvas padding is 80×64, with feet at (40,52): still 20 pixels below frame centre. Whole-body drawings retain one extraction scale as they fall.
- The run cycle is 0.8 seconds. The four-frame primary attack, Jab and Cross now last the native 18 ticks, with the primary's extended punch at the tick-nine hit boundary. These are presentation timings only; native attack timing and cadence are unchanged. Other legacy animation counts/durations remain.
- Violet/ivory launch, swap, punches, Cyclone, slam, uppercut, echoes, vanish/reappearance, Flurry and haymaker materials. Five shadow-charge marks bind to the existing `ct_charges:<n>` buffs.
- New numeric `shadow_art_v1: 1` opts fresh definitions into the new bindings. Legacy definitions keep the existing fallback. No career definitions were migrated.
- Projection body leases follow the **real targetable carrier**, rather than leaving copies at old points. Cyclone draws its own clone while the ordinary cleave is live, then stops renewing on recast. Short, single-frame ten-tick leases adjoin; one-shot impacts play their packaged duration once. No new owner hiding, idle/basic Animation CC, control or movement restrictions were introduced. The pre-existing mechanical carrier hiding and ultimate/projection restrictions remain.
- The original danger-zone publication remains independent of drawing. No combat core, AI, stats, damage, cooldown, terrain, Mod Power or RNG changes.

Decoded presentation: **3,072,000 bytes / 2.93 MiB**, including the 960×256 body and 1024×510 effects atlases. Both are below 2048 pixels. Only durations actually emitted by this kit are packed; compatibility effect aliases remain.

Native presentation APIs do not provide arbitrary cancellation of a point effect. On interruption, a just-issued clone lease may finish its remaining **at most ten ticks**; no further copies are issued. Follow-target effect cleanup and carrier visibility still need actual-game observation. Do not describe the automated tests as renderer acceptance.

## Review and playable Lab

Run from the repository root:

```powershell
rtk proxy node editor/extract-game-world.js
rtk proxy node tools/serve-contender-shadow-review.cjs
```

- Review: <http://localhost:7309/editor/contender-shadow-review.html>
- Playable Lab: <http://localhost:7309/editor/contender.html>
- Both also work without `/editor/` through this server.

The extracted installed-game reference is a local ignored cache, not redistributed stock game art. It registers **900/900 collision cells**, a 960,000-square map with 32,000-unit cells, and 1,000 units per sprite pixel. If absent, the Lab reports how to extract it; it never substitutes an estimated map.

The sandbox includes stock champions, real minion/camp/objective/structure art and settings, and eight situations: laning, gank, countergank, assigned camp, Morgard, Serpen, tower defence and dive. Geometry and sprites share one scale. Core skills and AI run through the existing WASM transport v1. Host movement, armour, visibility, farming eligibility, enemy attacks and tower behaviour are **labelled browser simulations**.

Controls: focus arena; WASD moves the body or steers its projection; Q Projection/recast, E Cyclone/recast, C Slam/Uppercut, R Flurry, F End projection, Space basic. Keyboard input starts playback. Pause/step, full-map view, pan/zoom, range outlines, ultimate learning gate, opponent toggle and thirteen athlete values are exposed. Focus loss clears held inputs. Playback is fixed at 60 simulation ticks/second, independent of refresh rate. Recording export keeps the last 1,200 frames; import inspects frame/state/effects and explains that old rules are required for replay.

The review inspector plays the **actual packaged frames**, with pose/frame/facing, light/dark terrain, stock comparisons and one-shot effects. [animation-preview.gif](animation-preview.gif) and [animation-contact-sheet.png](animation-contact-sheet.png) are derived from those same packed frames. Frozen original concept comparisons remain for the approval record.

## Reproduction

Sources were generated through the built-in imagegen tool. See the three `*-prompt-v1.txt` files and `*-source-v1.png` files. AI generation is **not deterministic**; extraction, mirroring and packing are reproducible from the frozen sources.

```powershell
rtk proxy python tools/prepare-contender-shadow-review.py
rtk proxy python tools/prepare-contender-shadow-effects.py
rtk proxy python tools/build-contender-shadow-art.py
rtk proxy python tools/generate-contender.py
rtk proxy python tools/verify-contender-shadow-art.py
rtk proxy python tools/preview-contender-shadow-animations.py
rtk proxy cargo +stable-x86_64-pc-windows-gnu build --release --target wasm32-unknown-unknown --manifest-path native/contender_shadow_lab/Cargo.toml
rtk proxy cargo +stable-x86_64-pc-windows-gnu build --release --bin trace --manifest-path native/contender_shadow_lab/Cargo.toml
rtk proxy node tools/test-contender-shadow-art.cjs
rtk proxy node tools/test-contender-shadow-parity.cjs
rtk proxy node tools/benchmark-contender-shadow-art.cjs
rtk proxy python tools/package-contender-shadow-art.py
rtk proxy python tools/check-contender-shadow-manager.py
```

Copy the resulting WASM library to `editor/contender.wasm` when rebuilding it. `presentation.json` is the versioned manifest used to generate champion bindings and Lab selection. The original-body reference is frozen from the PR baseline, so regenerating approval comparisons cannot silently substitute the new body for the old one.

## Evidence and limits

- **1,392 native tests passed, 18 ignored**, including **82 targeted Contender tests**. Four existing unused-code warnings in `native_bridge/detour_tests.rs` remain. Earlier PR counts are separate historical reports; these are the observed Windows GNU-toolchain results.
- **2 Lab transport tests**, **15 Mod Manager tests**, and **8 browser checks** pass.
- **24 development + 12 holdout fixtures / 5,400 frames:** complete native/WASM core state and effects equal. See [parity.json](parity.json).
- Same manual inputs with artwork enabled/disabled produce equal complete Lab traces. Profiles 20/80 leave manual mechanics identical. A native mock-host comparison also checks core state, damage, movement, CC, buffs and unit state with the new art flag off/on.
- Five seconds produces exactly 300 ticks at 30/60/120/144/240 Hz; pause and focus-loss input checks pass. Packaged effects stop at their exact duration, rather than holding their final drawing.
- Repacking reproduces hashes, all rectangles fit, both facings and legacy aliases exist, and emitted effect durations match. See [art-verification.json](art-verification.json).
- Five isolated local headless Edge loads: approximately 123–336 ms to readiness; warm 56-actor drawing about 0.06–0.15 ms per submitted draw. These are Lab figures, **not Proceed/day-advance or actual-game measurements**. See [performance.json](performance.json).
- Two independently built DLLs and the packaged DLL share SHA-256 **`9f2c881fd30485ed3edd67b3454b7b61139cd0a3f485755295c7ca5f6cb7af17`**. WASM hash and source fingerprint are in [package.json](package.json).
- Isolated Mod Manager comparison passes for **902 files**; source mods and real manager/game configuration remain untouched. See [mod-manager.json](mod-manager.json).

**Actual-game acceptance remains pending:** following/visibility of the carrier body, moving/attacking facings, lease boundaries on interruption/death, crowded Flurry readability and roster/real-career presentation. Installation and backed-up career migration remain separate; neither was performed.
