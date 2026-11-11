# Round 163: the global AI track, Part 1 (Fight Sense) and the athlete-stat audit

No version change: nothing runs differently in game, and no DLL was rebuilt.

## What was asked
Reihan started a standalone global-AI track driven by the 13 athlete stats:
1. **What the global Flash does for the AI, and which stats affect it.** None: `flash.rs` reads no stat. A 20 and an 80 athlete flash in exactly the same situations.
2. **Part 1, "Fight Sense":** a shared, host-free reading and scoring layer next to `athlete_ai_core.rs`. No champion is switched over; Akane will be first.
3. **Find the champions the stats don't affect,** and record it for new champions (Notion: TFM2 03 and TFM2 20).
4. **Find how the base game applies the stats to its own champions,** so new champions can reuse it. Reihan chose reverse engineering, and wants `last_hit`, `concentration` and `mental` read too.

## Done
- **Fight Sense:** `fight_sense_core.rs` (snapshot and scores, serde, no host) and `fight_sense.rs` (one build per match, tick and team, shared by every caller, with per-team memory). Six new `Profile` helpers. Guide: `docs/fight-sense-guide.md`.
- **`walls::snapshot()`** for host-free terrain.
- **Audit:** `docs/athlete-stat-coverage.md`.
  - Only Raze and Contender use all 13 stats.
  - 12 native kits use none in their own AI.
  - The shared movement and planner layers are **off by default** (round 143's `settings.json`). In a default install, the mod's only stat effect that applies to everyone is the farming judgement gate.
- **Notion:**
  - new memory *TFM2 20 · Global AI improvements (13 athlete stats)*;
  - *TFM2 03 · How to make a new champion* has a new section, "Athlete stats: make all 13 count", plus pitfall 21;
  - *TFM2 09* links the new memory.

## Not done, and why
- **Part 0 (the engine's own formulas):** the game binary isn't in this cloud session, and no computer folder is shared.
  - **Needed:** Reihan shares the Teamfight Manager 2 install folder through the Claude desktop app (Settings → Claude Code → Remote Control), or uploads the game executable. Then REA reads the formulas, which go into `docs/engine-athlete-stats.md` and `Profile::engine_*`.
  - The extra stats (`last_hit`, `concentration`, `mental`) wait for this. They need a `Profile` v3 encoding, and they only make sense with the engine's formulas.
- **Armour formula:** `effective_hp` uses the repo's viability model, `raw × 100 / (100 + resist)`. Part 0 should confirm it.
- **Labs:** the core builds alone with only `serde` and its two sibling cores (checked with a throwaway crate). The wasm target isn't installed here, so a wasm build wasn't tried. No Lab includes it yet.

## Verification
- `cargo test --release` in `native/tfm2_custom_ai`: **1,175 passed** (1,159 before, plus 16 new), 0 failed, **0 warnings**.
- Existing helpers and kits are unchanged. The new `Profile` methods are in a separate `impl` block, and nothing calls Fight Sense in game.
