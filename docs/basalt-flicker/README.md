# Basalt effect continuity — 17 November 2026

Based on merged main `79484f81d90aabe9ca86542a521e747fb7935b7b` (PRs #88/#89). Reihan reported that walls, smashes and ultimate rocks still flickered in the **installed game**, not just the Lab. The inspected installed package was native 0.10.111 / champion pack 0.2.114.

## Cause and correction

An Animation view effect lives for the sum of its `.fanim` frame durations. The SDK's `play_view_effect(time)` does not stretch it (the existing champion guide documents this contract). Persistent Basalt materials had one-tick animations, but walls renewed every six ticks and moving rocks every three. PR #89 increased the requested lease time and priority, without changing the actual animation length. Its interval tests therefore could not detect the missing rendered frames.

Art revision **basalt-art-6** adds duration-specific aliases sharing the exact approved atlas rectangles. Six-tick standing-wall and warning animations renew every six ticks; moving rocks/blocks/cones last three ticks. Predictable flight/charge/cone endings use shorter aliases when necessary. Lifetimes are adjoining and exclusive: a replacement begins exactly when the preceding instance ends. The priority scheduler remains bounded, without the previous 4/13-tick overlapping requests.

The 247 original animation aliases remain byte-for-byte equivalent in frames, order, rectangles and durations. New aliases bring the count to 586, with **no new texture pixels**. Body and effect atlas PNGs are byte-identical to merged main; decoded presentation remains **24 MiB**, with atlas dimensions at most 2048. `assets.json` verifies the shipped file independently against generated lifetime metadata.

Smash, shatter and landing sequences remain one-shot events. Their 12/18/24-tick aliases retain every existing drawing and order, matching the emitted duration rather than repeatedly restarting or holding their final drawing. No combat effect is added.

The Lab now expires effects using the packaged animation, removes expired instances before renewing, and draws against the tick which produced the current frame. Its old indefinite final-frame holding concealed the native mismatch. It deliberately does not deduplicate overlapping native emissions to hide errors.

## Verification

- Native suite: **1,371 passed, 18 ignored**; four inherited unused native-bridge test-helper warnings. No new warnings.
- The final suite runs with one test thread. An intermediate parallel run hit the existing `levi::tests::flights_follow_the_walking_path_and_read_walls` failure; the serialized full suite passes without any Levi changes.
- Lab crate: **13 tests passed**, including independent reading of the shipped `.fanim`, exact one-copy coverage at fractional ticks, predictable ending boundaries, one-shot expiry and owner isolation.
- Native/WASM: **24 development + 12 holdout scenarios**, 10,800 complete state/effect/presentation ticks identical. `parity.json` records the trace hash.
- Browser: 13 existing map/pose/control/combat-independence checks pass. Additional continuity checks cover **840 steady-wall subframes**, five complete one-shot sequences, legacy one-tick expiry, no doubled boulder and exactly one landing. `browser-continuity.json` records the result.
- Five fresh-context/warm-render samples are in `browser-performance.json`; they measure the Lab, not native FPS or Proceed time.
- Canonical generation, deterministic packing, unchanged rules/core/body/effect PNGs and atlas budgets are checked by `assets.json`.

Actual-game rendering is a separate gate. The disposable game reached its main menu and Mods screen. Reihan reported enabling the code mod; subsequent Computer Use was stopped with physical Escape before post-approval inspection. No match or corrected native frames were observed. The exact owned process was closed before filesystem isolation detached; all 47 real-profile files stayed identical. The staged runtime candidate preceded the final legacy-binding fallback and is distinguished from the final package in `native-attempt.json`. Automated tests and canvas observations do not establish native acceptance. A later disposable match must still verify walls, moving blocks, boulders/meteors, impact sequences, crowded scenes and cleanup.

## Reproduce

All shell examples below run from this worktree and use the user's `rtk` prefix.

```powershell
rtk proxy python tools/build-basalt-effects.py
rtk proxy python tools/generate-basalt.py
rtk proxy python tools/verify-basalt-flicker.py
rtk proxy cargo +stable-x86_64-pc-windows-gnu test --release --manifest-path native/tfm2_custom_ai/Cargo.toml -- --quiet
rtk proxy cargo +stable-x86_64-pc-windows-gnu test --release --manifest-path native/basalt_lab/Cargo.toml
rtk proxy cargo +stable-x86_64-pc-windows-gnu build --release --example trace --manifest-path native/basalt_lab/Cargo.toml
rtk proxy node editor/extract-game-world.js
rtk proxy node tools/serve-basalt-review.cjs 7308
```

The extracted game reference is an ignored local cache. Build `basalt_lab` for `wasm32-unknown-unknown` and copy its WASM to `editor/basalt.wasm` before parity/browser checks. Set `BASALT_RESULTS=docs/basalt-flicker` when running `test-basalt-parity.cjs`, `test-basalt-art-review.cjs` and `benchmark-basalt-art.cjs` to preserve earlier evidence. `test-basalt-flicker.cjs` writes here directly.

`package-basalt-continuity.py` builds the Windows DLL twice in independent target directories, compares hashes and packages matching native **0.10.112** / champion pack **0.2.115**. `package.json` records the exact hashes. Mod Manager verification uses a disposable repo/game pair, not the installed game.

Basalt's gameplay remains **basalt-1.4**: rules, core, AI, recovery, farming and approved designs are unchanged. New definitions opt into the timed aliases with art_timing_v2. Older career-embedded definitions retain their known original effect names, avoiding blank effects, but still require the existing backed-up definition migration to receive the timing fix. No career migration or real-game installation is performed by this change.
