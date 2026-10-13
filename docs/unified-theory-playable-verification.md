# Science Lab playable arena verification

2026-10-13. This correction replaces the schematic trajectory viewer in the editor. It does not change the installed native DLL, champion package, skill IDs or mastery files.

The actor uses transparent runtime body and outfit frames. Right-click moves; left-click locks a dummy or clears the lock for cursor aim; Space casts; S stops; Escape cancels preparation. Both standalone and embedded Lab default to the practice arena. The animation inspector remains available.

`ScienceCombat.Combat` provides commands, fixed 60-tick updates, snapshots and typed events without depending on the DOM, artwork or wall clock. Every skill has an explicit handler corresponding to `unified_theory.rs`. Inputs, recipe weights, prerequisites, material, momentum, three independent charges, delayed release, owned packets, fields, support and chemistry share that model. `ScienceRenderer.Renderer` reads the original transparent body and runtime atlas frames and coordinates artwork with actual simulation events.

## Verification

| Check | Result |
|---|---|
| Native release suite | 209 passed, zero failures; one pre-existing ignored test |
| Skill handler parity | 150 Rust/JavaScript traces: commitment and 24 world ticks for every one of the 75 handlers |
| Authored experiments | All 50 complete deterministically with legal live prerequisite fixtures and one charge each |
| Charge parity | 2,400 exact Rust/JavaScript states; reservation, refund, exhaustion and staggered recovery |
| Core scenarios | Cursor capture, moving origin, current target prediction, invalid launches, fixed headings, contact coordinates, placement rules, splitting, reflection, orbit, chemistry, support/control budgets, paid prerequisites, pause/step, death and respawn |
| Runtime assets | 3,445 manifest bindings in 38 atlases match native frames exactly; transparent body frames and all consumed skill/impact/packet/field/pose bindings validated |
| Real browser input | Right-click movement, S, Space, Escape, cursor/locked aim and coordinate mapping pass in standalone, embedded and actual editor iframe surfaces |
| Visual review | 192 form/rank/facing/body-phase renders; visible crystals, clouds, shielding, transparent character backgrounds and responsive layouts |
| Crowded practice models | Ten simultaneous scientists, 32 enemies each, moving targets and incoming projectiles over 720 ticks; output, artwork and event-history bounds hold |
| Existing editor checks | All skill/rank/Top 10 selectors, search, 50 recipes, transformations, native-log import, hidden-frame suspension and isolated mastery workflows pass |

The trace fixture captures native kit operations with a minimal stable host. It is not a complete game engine. Practice HP changes report raw emitted skill damage; game armor, item procs, engine collision geometry and autonomous match decisions require native diagnostics or an in-game match. Rank comparisons remain notebook studies. Cursor aim is an explicit practice override; native combat still uses enemy identities.

The practice model bounds owned packets/instruments at eight, fields at twelve, temporary effect starts at six per tick, active temporary artwork at 32 and retained timeline events at 200. Cast/body channels replace previous poses and activation artwork. Cosmetic suppression never delays combat.

## Reproduce

```powershell
rtk proxy node tools/verify_science_combat.cjs
rtk proxy node tools/verify_science_charges.cjs
rtk proxy python -X utf8 tools/verify_science_runtime_assets.py
rtk proxy node tools/verify_science_playable_browser.cjs
rtk proxy node tools/verify_science_editor_browser.cjs
rtk proxy node tools/verify_science_art_browser.cjs
rtk proxy node tools/verify_science_redesign_browser.cjs
```

Browser checks require Playwright through `NODE_PATH`, Chrome through `CHROMIUM_PATH`, and the running editor URL through `SCIENCE_LAB_URL`. The integration test uses an isolated temporary game directory and verifies its cleanup path.

[Embedded practice arena](unified-theory-playable-embedded.png) · [Standalone arena](unified-theory-playable-standalone.png) · [Native-scale pose contact sheet](unified-theory-playable-poses.png).
