# Round 178: Contender reworked into the Shadow Boxer (native 0.10.113, tfm2_custom 0.2.116, `contender-2.0`)

**Reihan:** "rework how contender works, i want him to be a shadow boxer in the liter means of shadow boxer … remove the whole kit from skills to ult". The kit was settled over several plan revisions: charges become recasts, Shadow Pact becomes a passive, Projection is S1, Cyclone and Slam & Uppercut back it, and the ult is a Hayabusa-style flurry. The full kit is in `docs/contender-guide.md`.

## What changed
- **The new kit:**
  - **Passives:** Shadows (5 charges, recasts, the ult's cooldown cut by every hit), Shadow Pact (echoes) and Shadow Rhythm (stacks).
  - **Skills:** S1 Shadow Projection, S2 Shadow Cyclone, S3 Slam & Uppercut.
  - **Ult:** Shadow Flurry.
- **Code:**
  - a new deterministic core, `contender_shadow_core.rs`, generated from `rules/contender.json`;
  - a rewritten bridge, `contender.rs`;
  - a new Lab crate, `native/contender_shadow_lab`, and `editor/contender.html`.
- **Retired:**
  - **The old kit:** stamina, Combination, Slip / Guard and Main Event.
  - **Its code:** `contender_core.rs` and `contender_arena.rs` remain for the realm code that shares their types; they read `rules/contender-1-legacy.json`, and no ring is opened.
  - **Old tests:** the old bridge's glue tests and one test that read the old bridge's source were removed.
  - **Old careers:** they keep their round-176 data. Its `contender_basic` and `contender_ultimate` refs still resolve, as a plain basic and the shared ult marker.
- **The champion template:** the other generators read `tools/native-champion-template.json` (a frozen copy of the old data shell) instead of Contender's data file. All their outputs are unchanged (`--check`).

## Merged with main
- **Codex's Basalt continuity fix:** PR #90 (Basalt effect continuity) merged into `main` first and took 0.10.112 / 0.2.115, so this round moved to 0.10.113 / 0.2.116.
- **The DLL:** rebuilt from the merged source, so it carries both #90's fix and this rework.

## Checks
- **Native tests:** 1,388 passed, 18 ignored, including 24 new Contender tests (core, AI and glue). Contender's tick cost is about 4.7 µs.
- **Lab:** the Lab crate's transport tests pass. A headless Chromium run of `editor/contender.html` (projection, priming, slam and uppercut, the swap, and the flurry) showed no console errors.
- **Other checks:** the generators are current; the DLL is built; the Mod Manager tests (15) pass.
- **Not run here:** the real-map Lab (it needs the extracted game reference) and Codex's Windows checks.
