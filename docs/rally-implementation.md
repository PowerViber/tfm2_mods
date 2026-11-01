# Rally implementation and review

This document preserves the 2.1 implementation snapshot. See [Rally 2.2 changes](rally-2.2-changes.md) for current behavior and verification.

This work is isolated on `codex/volleyball-rally`. The shipped champion definition and game artwork are **not integrated yet**: the user has requested successive preview refinements rather than giving final artwork approval. Nothing has been installed or migrated into a real career. Release versions and packaged DLL hashes remain pending. The prototype may be published for review independently of that gate; see [the review handoff](rally-review-handoff.md) for current publication scope and later confirmed changes.

## Implemented 2.1 snapshot

The later confirmed gameplay changes in the [review handoff](rally-review-handoff.md#confirmed-changes-not-implemented) supersede parts of this snapshot but have not been implemented here.

- Original full-face masked athlete: yellow base outfit, black/red after a **landed Black Thunder**. Temporary ultimate unmasking is separate. No mastery; thirteen pinned athlete attributes.
- Physical Attack-scaling kit, three charge pips, Flow, Receive, clone Self Set, separate teammate support ball and timed Run-up/Spike. **Serve replaces the old role ultimates.**
- Confirmed: Black Thunder is a separate deliberate command; accepted Set selects Setter, Receive selects Receiver, Run-up selects Spiker. Rejected commands do not change role. Manual role commands are retired with an explicit rejection.
- Prototype Run-up: +40% directional sprint for up to 1.5 seconds, no automatic chase/toss. Optional takeoff starts the clone toss and spends Self Set. WASD direction and mouse spike aim are independent. Charges now recover one per six seconds.
- Serve: 75-second cooldown, 0.4-second preparation, fixed 60-degree sector, 560,000 range, one attempt. Stops walking; applies 170% final contact height once. Set/charges/permanent power stack before this multiplier. Serve does not grant the retired role-ultimate aura/reduction/damage values.
- Black Thunder: separate 30-second prototype lockout, three charges plus at least 60 Flow, spending both. Can be armed before Serve. A perfect first Black Serve reaches 10.2 m; with Black's permanent tier it reaches 12.75 m. An ordinary perfect Set/three-charge Serve reaches 9.18 m, or 11.73 m after Black's tier.
- At 10 m actual successful Serve contact, White Storm blinks to the ball apex and uses 6x base Spike with Self Set (not multiplied again by ordinary/Black damage). No invulnerability. A landed hit replaces the permanent Black tier with prototype +2.5 m / +15pp Setter / +20pp Receiver. No repeated stacking; clears on death. White's extra values remain balance prototypes.
- Black Thunder's permanent power-up is **role bonuses only**, once per life: Spiker +1.5 m contact height, Setter aura bonuses +10 percentage points, Receiver reduction +15 percentage points. Perfect Receive remains capped at 95%. Death clears the power-up. No general stat bonus or healing.
- First Black Thunder: 6 m contact. Later empowered Black Thunders: 7.5 m; the bonus does not stack again. Their damage stays at the original 5× base-spike finisher formula, with the Self Set modifier and without ordinary Thunder/ultimate multipliers.
- Ordinary spike range/radius are **240,000 / 45,000**. Regular Black/White impact radius is **65,000**; every Serve variant uses **90,000** with a larger crater, with one 0.5-second knock-up on eligible nonimmune targets, then a four-second 30% slow crater. Objectives/buildings are not launched. These values are prototypes; warnings use the same rules. A released ball and its crater retain the committed radius after the firing window closes. The AI considers targets throughout the extended Serve range.
- Ordinary Thunder is yellow/gold with white cores; Black Thunder is black/crimson/white. The large body coil, descending ball, long coiling trail, directional impact and angular lightning strokes are distinct presentation layers.
- A successful Thunder contact holds Rally's flight for 6 ticks, or 9 for Black/White. This is a local, vulnerable contact hold, not global hit-stop. Host walking is suppressed during it; enemy control can interrupt it. These hold timings are prototype defaults. Attempt completion/interruption remasks Rally without clearing earned permanent power or resetting cooldowns.
- Self Set creates a visual-only Rally clone. The combat ball's set arc originates at the clone, not at Rally. There is no untargetable damaging summon.
- The teammate's support ball remains attached during Rally's own jump. One Self Set and one teammate Set may coexist. Current prototype timing is a three-second marker plus two seconds of stat-buff linger; death/realm loss removes the effect.

## Editor mode presets

The toolbar exposes Yellow, Black Thunder and White Storm starting forms, plus a separate armed-Black-Thunder checkbox. These are explicit practice presets: switching restarts the arena with full Flow and three charges. They use the standalone Lab fixture operation and cannot be supplied through a native gameplay Frame. Recordings save the chosen setup and replay from a fresh core without an extra hidden simulation tick. A 0.75x wide view helps inspect the longer sector.

## Sources and evidence limits

The user supplied `20261101-0522-19.0883686.mp4` (Roblox adaptation reference) and `20261101-0614-53.7892106.mp4` (Nishikawa/angular strokes). The studies in `rally-results/reference-*` are local sampled frames, not downloaded assets or proof of the source game's mechanics. The second recording contains a player pause; its held frames were not used as a gameplay timing measurement. Full-screen backgrounds and UI were not copied.

Generated source images are frozen in `rally-art-review`. Generation itself is nondeterministic. Extraction, phase derivation and atlas packing are reproducible. `packed-preview` is a review output, not installed mod art. The generators require matching approval hashes before writing game artwork.

## Architecture

- `rules/rally.json` is canonical; `tools/generate-rally.py` emits Rust constants, the editor preset and champion preview. Current rules revision: `rally-2.1`, transport v1 with explicit revision checks. The shipped roster/farming registration waits for the approved game definition; the editor's separate preview remains available.
- `rally_core.rs` is the deterministic shared native/WASM core. It owns resources, cooldowns, ball state, clone origin, role state, target-bound receipts, flight/contact holds and warnings. There is no global mutable combat state or combat-RNG consumption by presentation.
- `rally.rs` adapts host snapshots/effects. Accepted basic impacts are not rejected by another range or wall test. Native farm eligibility selects targets; it does not make incidental damage disappear from unassigned waves/camps/summons. Buildings can receive accepted basics but not spike splash.
- Supported learning slots unlock Receive/Run-up and Set. Ultimate availability includes `ult_learned`. Native actions use inert shells; no stock cast injection.
- The planner reviews bounded authored continuations: at most 64 candidates, retaining eight plans and up to three actions. This is not a claim of exhaustive counterfactual host simulation.
- Air poses use the user's approved **flight-only** hidden-animation exception. Idle/basic presentation does not renew Animation CC. Air-body effects are short phases positioned from the shared height; ground sprites retain visible roster idle frames.
- One code-defined world scale supplies game-size Lab geometry. Zoom scales characters, effects and world geometry together.

## Verification

Run from the repository root (Windows development toolchain):

```powershell
rtk proxy python tools/generate-rally.py --check
rtk proxy python tools/prepare-rally-art.py --preview-pack
rtk proxy python tools/prepare-rally-effects.py --preview-pack
rtk proxy python tools/verify-rally.py
rtk proxy cargo +stable-x86_64-pc-windows-gnu test --release --manifest-path native/rally_lab/Cargo.toml
rtk proxy cargo +stable-x86_64-pc-windows-gnu build --release --bins --manifest-path native/rally_lab/Cargo.toml
rtk proxy cargo +stable-x86_64-pc-windows-gnu build --release --lib --target wasm32-unknown-unknown --manifest-path native/rally_lab/Cargo.toml
rtk proxy node tools/test-rally-lab.cjs
rtk proxy node tools/test-rally-editor-route.cjs
rtk proxy node tools/preview-rally-timing.cjs
```

Copy the freshly built WASM to `editor/rally.wasm` before browser tests. `preview-rally-timing.cjs` uses the read-only local preview server on port 7294. Its GIFs are captures of actual shared-core Lab frames, not separately scripted animation physics.

Reports in `rally-results` separate shared-core tests, browser observations, scripted development scenarios and native-host acceptance. The development simulator has 24 development and 12 holdout seeds and five 128-unit core timing runs. Its mitigation, enemy attacks and movement are scripted: its HP loss is **not engine-attributed damage**, and it does not establish the requested 45–55% matched-role peer balance.

An early parallel native run had a nonreproducing `raze_frames_match` failure; its focused rerun and subsequent serial full suite passed. Existing Windows test-helper/dead-code warnings are reported rather than claiming zero warnings. Final integration will rerun the affected suites.

## Mandatory native acceptance still pending

1. Accepted hybrid basics, dynamic reach and exactly one primary item-proc identity in the actual game.
2. Reliable individually attributable incoming damage/shield receipts for Receiver storage.
3. Reliable ally champion-hit receipts for support-ball Flow.
4. Actual movement, contact holds, learning, parry/smoke/terrain, realm eligibility, ownership, death cleanup and game-renderer effect limits.
5. Matched current-main Contender/Raze/Steve compositions, actual damage/support/control and survival; crowded host performance and the unchanged Magnus 128-piece target.

The adapter currently **fails closed** for caught-power and ally-hit Flow (`incoming_verified = false`, `support_verified = false`). Lab receipts are explicitly simulated. Immediate damage readback is recorded internally as a provisional hit confirmation; whether the host applies damage synchronously remains a native proof, not a Lab assertion. No hidden hooks, post-damage healing workaround or untargetable damaging helper substitutes are used.

Installation, real-career migration and actual-game acceptance remain separate from publishing the PR. Artwork approval is a prerequisite to packaging, and has not been inferred from revision requests.
