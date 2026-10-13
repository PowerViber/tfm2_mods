# Unified Theory redesign — verification and installation

Release: native **0.10.38**, champions **0.2.38**, dated **2026-10-13**.

Implemented three shared charges with independent 300-tick recovery, one reservation/commitment per experiment, cancellation and death handling, and release-time aiming from current host positions. Individual recipe cooldowns and the CU resource gate are removed; recipe weights, preparation cadence, material/momentum, mastery records and damage/control limits remain. The planner evaluates the 125 authored choices with range, prerequisites, resources and output limits, prioritizes actionable defense and prepared goals, and rejects redundant or zero-output opportunities. All 75 skill handlers and 50 combinations remain.

Historical forms now have distinct instruments and tailoring, plus 72 eight-frame preparation/release/recovery body animations through one replacement layer. Fixed field borders improve area readability. The 57×12 HUD contains the existing scientist orb and three independently filling charge cells. Science Lab provides matching charge math, a schematic timeline and local native-log import with selection, target, origin/aim, charge and HP events. Root-editor and iframe integrations both load the new charge model.

## Checks

| Check | Result |
|---|---|
| Full Windows GNU native release suite | **208 passed, 0 failed, 1 pre-existing ignored** |
| Mod Manager tests | **13 passed** |
| Charge replay parity | **2,400 exact Rust/JavaScript states**, including reservations, cancellation, exhaustion and staggered recovery |
| Recipe arithmetic | **375 native vectors**, all eight ranks deterministic |
| Native regression scenarios | Moving caster/target at delayed release; stale aim; reflection before launch; fixed heading afterward; field placement; invalid targets; one charge for repeated stages; deadline retained across death; actionable defense and legal heavy-attack range |
| Asset validation | **6,464 registered views / 41 VFX sheets**, every atlas ≤2048, eight distinct frames, fixed origins, lifetimes, crystal coverage and byte-identical regeneration |
| Browser checks | Standalone lab, all skill/mastery/Top 10 animations, integrated editor, shared-charge timeline, native-log import, cancellation/recovery and mobile layout pass |
| Crowded simulations | Native fixtures pass for five scientists and four scientists plus Coder, including deaths, invalid IDs, failed spawns and bounded effect lifetimes |
| Formatting / patch checks | Changed Rust files formatted; `git diff --check` passes |

Estimated decoded scientific VFX atlas storage is **134.35 MiB**. This is an atlas-size estimate, not actual game memory or an FPS measurement. Existing ownership and effect-start bounds remain.

Key commands used:

```powershell
rtk proxy rustup run stable-x86_64-pc-windows-gnu cargo test --release --manifest-path native/tfm2_custom_ai/Cargo.toml
rtk proxy rustup run stable-x86_64-pc-windows-gnu cargo build --release --manifest-path native/tfm2_custom_ai/Cargo.toml
rtk proxy rustup run stable-x86_64-pc-windows-gnu cargo test --manifest-path tools/manager/Cargo.toml
rtk proxy node tools/verify_science_charges.cjs
rtk proxy python -X utf8 tools/verify_unified_theory.py
rtk proxy node tools/verify_unified_theory_browser.cjs
rtk proxy node tools/verify_science_art_browser.cjs
rtk proxy node tools/verify_science_editor_browser.cjs
rtk proxy node tools/verify_science_redesign_browser.cjs
```

Browser checks use the bundled Playwright packages through `NODE_PATH`, the installed Chrome executable through `CHROMIUM_PATH`, and an isolated editor server. Generator writes use atomic replacement to accommodate mapped files on Windows; UTF-8 is explicit. Vector line endings are normalized for Windows parity checks without changing arithmetic.

## Installed result

With the game closed, **89 Unified Theory/native files** were copied into:

`C:\Program Files (x86)\Steam\steamapps\common\Teamfight Manager2`

Every installed file was verified against the workspace using SHA-256. The installed metadata reports native 0.10.38 and champions 0.2.38. Deployment updates the scientist's dependencies, champion description/package metadata and shared native DLL; other champion files, tactics and mastery data are retained.

Complete pre-install package backup:

`C:\Games\tfm2_mods\backups\game_mods_unified_redesign_20261013_033937`

Its `installed_manifest.json` records the 89 verified file hashes. The original locally modified workspace DLL is preserved in `backups/unified-redesign-20261013/local-before.dll`. The existing `editor/gamedata.js` remains byte-identical to the initial backup.

## Visual review and remaining game check

[Casting motions](unified-theory-casting.gif) show stored runtime sprites, slowed threefold for review. [Three-cell HUD](unified-theory-hud.gif), [Top 10 forms](unified-theory-top10.png), and [native diagnostics](unified-theory-native-diagnostics.png) provide additional review artifacts.

Automated host and browser checks validate math, ownership, timing, data and artwork. **A live game match and real renderer/FPS acceptance were not performed** because native game UI control is unavailable in this environment. On the next game launch, check visible five-second recovery, moving-target accuracy, crystal rendering and crowded-fight readability. Science Lab can inspect the resulting `unified_theory_damage_log.txt`; imported native events remain separate from schematic trajectories.
