# Contender Lab: enemy Shadow Rhythm stacks

The playable Lab now shows a compact `count/10` badge above each living, visible enemy with active Shadow Rhythm stacks. Counts come directly from the shared core's per-target `state.stacks` entries, using the canonical `tick < until` expiry. Gold indicates ten stacks. The badge follows the target and remains readable above map foreground artwork. Allies, hidden/dead enemies, expired entries and the owner's lingering buff do not produce enemy badges.

This is a Lab HUD change under `contender-2.2`, transport v1. Rules, native/WASM binaries, gameplay, packaged artwork and release versions are unchanged. Nothing was installed or career-migrated. Native-game stack presentation is outside this change.

## Verification

- `browser-observations.json`: actual shared-core Flurry produced separate **6/10** and **5/10** counts; both reached **10/10**. Drawing preserved core state. Checks covered movement tracking, hidden/dead/allied exclusions, exact expiry, reset cleanup and no browser errors.
- `browser-results.json`: all **eight existing artwork/Lab checks** passed, including equal complete combat traces with artwork on/off and manual profiles 20/80; exact 60 Hz simulation at 30/60/120/144/240 Hz; focused keyboard input and focus-loss cleanup.
- `1vx-browser.json`: all **twelve existing 1vX checks** passed, covering actual target assets, independent damage counters, finite/unlimited HP, setup preservation, mixed targets and layout.
- Native suites were not rerun for this UI-only change. Their earlier evidence remains in `docs/contender-flurry-review/`; these browser observations do not establish actual-game acceptance.

![Independent enemy stack badges](enemy-stacks.png)

Re-run the existing checks without overwriting prior evidence by setting `CONTENDER_RESULTS=docs/contender-rhythm-hud-review` and running `node tools/test-contender-1vx.cjs` and `node tools/test-contender-shadow-art.cjs`. Start the local server with `node tools/serve-contender-shadow-review.cjs`; use `/editor/contender.html?build=contender-2.2-rhythm`.
