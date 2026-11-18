# Contender 2.4 — approved Rhythm presentation

Continuation from Claude's `1648b00b8d6c610b5be472784fc04edb30d79a64` in PR #91.
Reihan approved the Break burst, full-Rhythm aura and stack/bank indicators on
18 November 2026: **“Approve the 2.4 effects and indicators.”**

## Current behaviour

- A dedicated `ct_break` presentation cue plays the approved four-drawing,
  18-tick violet implosion/shatter once. The post-ultimate haymaker keeps its
  existing artwork. Older definitions without the new opt-in use the old cue.
- The owner's low foot aura starts on Break, lasts 480 ticks, and follows one
  stable phase clock. Routine stat-buff refreshes do not restart it. A later
  Break restarts the eight-second window.
- Visible eligible enemies carry the small fist logo and tiny count, up to 20;
  twenty turns gold. Zero, expiry, death and lost visibility remove it. Each
  of the ten player slots has its own aliases, so one owner cannot clear another
  owner's marker. Nonstandard player slots omit these markers safely.
- Eight bank pips hold five stacks each and turn gold at forty. They are separate
  from the existing dodge-charge display.
- Active Cyclone circles show the actual 25,000/35,000 radii. Native circles use
  adjoining six-tick single-frame leases, expiring at most six ticks after cleanup.
  They add no collision, damage or control.

Damage, stack mechanics, buffs, cooldowns, AI, Mod Power and control are unchanged
from the 2.4 baseline. The shared core's only source change disambiguates the
visual Break cue from the haymaker. There is no image-driven gameplay or damage
inference. Native presentation adds only zero-stat artwork buffs and effects.

New champion definitions opt in with `shadow_rhythm_art_v1: 1`. Existing embedded
career definitions keep their compatible presentation; a backed-up migration is
a separate operation. Normal bodies, fused Merge pixels and legacy FX remain
unchanged.

## Previews and reproduction

- Play: http://localhost:7309/editor/contender.html?build=contender-2.4
- Inspect: http://localhost:7309/editor/contender-rhythm-review.html
- [Break/aura comparison](effects-preview.png), [indicators](hud-preview.png),
  [merged/left comparison](effects-merged-left.png), [Cyclone Lab](lab-cyclone.png).
- Sources, exact prompts and approval: `generation.json`, `approvals.json`,
  `break-source-v1.png`, `aura-source-v1.png`.
- Repack: `rtk proxy python tools/build-contender-shadow-art.py`, then
  `rtk proxy python tools/generate-contender.py`.
- Start the local server with
  `rtk proxy node tools/serve-contender-shadow-review.cjs`.

Built-in imagegen generated the original frozen drawings. AI generation is not
deterministic; extraction, quantisation, mirroring and packing reproduce from
those files. The separate Rhythm atlas is 512×680, and total decoded artwork is
**7,806,976 bytes (7.45 MiB)**, below 32 MiB. All atlases remain below 2048 pixels.
`art-verification.json` records exact hashes and duration/bounds checks.

## Evidence and limits

- `windows-checks.json`: **1,406 native tests**, 18 ignored; **2 transport** and
  **15 Mod Manager tests** pass. Four inherited unused native-bridge test-helper
  warnings remain. New native tests cover stable aura expiry, independent owners,
  marker cleanup and absence of combat writes.
- `parity.json`: 24 development + 12 holdout scenarios, **5,400 frames**, complete
  current native/WASM state and effects agree.
- `1vx-browser.json`, `browser-results.json`, `flurry-browser.json`,
  `merge-browser.json`: the existing 12 1vX, 8 browser, dedicated Flurry and
  7 Merge checks pass. Paid 40-stack ultimates are funded by real core actions.
- `review-browser.json`: all 21 stack states, 41 bank states, both body forms and
  facings, Break's 18 ticks and aura's 480 ticks.
- `integration-browser.json`: authoritative Break renders, bank is visible,
  aura expires and both active Cyclone circles render from packaged assets.
- Art-enabled/disabled complete Lab traces and manual profiles 20/80 agree.
  `check-contender-24.py` also verifies rules/data are unchanged and the sole core
  delta is the visual cue name. `combat_core_and_rules_unchanged: false` in the
  older generic art report reflects that literal name change, not a numeric change.
- `performance.json`: five isolated headless Edge loads and warm draws, measured
  separately from actual-game performance and Proceed/day advancement.
- `package.json` records matching independent Windows builds, packaged DLL,
  WASM/source hashes and reserved versions **0.10.113 / 0.2.116**.
- `mod-manager.json` records isolated Mod Manager verification. Installed game
  files, real saves and careers are not modified.

Earlier 2.0–2.3 evidence remains in its original review folders. The named JSON
reports and `*-checks.txt` are the current continuation's evidence.

**Actual-game acceptance is pending:** buff-layer anchor/facing, concurrent-owner
readability, crowded fights, native effect continuity and cleanup require a live
match. The Lab's extracted map and actors do not prove engine rendering. The
existing heal-after-hit dodge limitation is unchanged. No installation, real-career
migration or balance change is included.
