# Basalt artwork and real-map Lab

Work on `codex/basalt-art`, based on PR #88 head
`3790d7ddb9578f3fc051e67bfa5f8f0bbb1f200a`. Shared rules: `basalt-1.2`.

## Confirmed approvals and corrections — 16 November 2026

- Grey/orange concept rejected; Reihan requested recognisable red/yellow.
- **Appearance approved:** “Approve the red/yellow body.” Human face, yellow
  headband/sash, crimson outfit and stone-linked whip are retained.
- Later: “47 is too tall and too wide, the boulder ults are too big.”
  Body reduced proportionally: idle 39–41 visible pixels including gear; walk
  38–39. Cosmetic boulder: 60×58 rather than 88×86 pixels.
- Crushing Clap shows a cone from Basalt, like Vader's choke. Its two blocks
  now start at the **maximum-range corners**, 110,000 from Basalt, merging at
  the far centre. Hit eligibility, damage, stun and 18-tick closure are unchanged.
- **Effects approved:** “Approve these effects,” for the corrected art-4
  review. The smaller boulder, far-corner cone and stone/whip/meteor materials
  are now integrated into the game pack.

Answers: `approvals.json`. Source hashes: `body-frozen-sources.json`. Complete
animations of the approved appearance require no repeat appearance approval.

## Body completion and reproduction

Fourteen sequences: idle 4, run 6, attack 4, raise 4, punch 4, throw 4, snare 4,
swing 4, jump 4, air 2, charge 4, ultimate punch 4, hit 2 and death 6. Both
facings use connected whole-body drawings and one palette; no sliced anatomy.
Alpha-connected extraction avoids clipping whips at artificial source-grid edges.

The 160×80 canvas provides transparent lash padding. Foot anchor (80,60) stays
20 pixels below frame centre. First idle uses the approved concept at its
corrected smaller scale and remains visible on roster cards. Six-frame walking
takes 0.8 seconds at normal speed; the Lab advances it by distance travelled.
Basic drawings occupy the existing 20 ticks with contact at tick 10.

Original transparent sources were generated with built-in imagegen. Prompts:
`prompts.json`, `prompts-v2.json`, `animation-prompts.json`. Generation is not
deterministic. Extraction, palette matching and packing reproduce:

```text
rtk proxy python tools/prepare-basalt-art.py
rtk proxy python tools/build-basalt-body.py
rtk proxy python tools/build-basalt-effects.py
rtk proxy python tools/generate-basalt.py
rtk proxy python tools/verify-basalt-art.py
```

Body files: `mods/tfm2_custom/champions/tfm2_custom_basalt#*`. The review
`body.png`, `body.fanim`, `body-presentation.json` and GIFs show the same frames.
Body atlas: 1024×2048, 8 MiB decoded. Effects atlas: 2048×2048, 16 MiB.
Total: **24 MiB**, under 32 MiB; no atlas exceeds 2048 pixels. Both are packaged.

## Native presentation limits and release gates

The engine selects ordinary idle, movement, basic, hit and death poses.
Additional action drawings are inspectable in the Lab. This pass does not
claim supported selection of every owner pose during passive-only native
skills. No hiding, renewed Animation CC, idle/basic locks or cosmetic movement
restrictions were added to force a pose.

The reserved native **0.10.110** DLL and champion pack **0.2.113** are packaged.
Two independent Windows GNU builds have identical SHA-256:
`fc0ef2e63d55712f8c9b8d9140c4ddccbb36bbc7b23e1c05a6f405bfdcfee2ce`.
See `package.json` for source identity, compiler, epoch, WASM and atlas hashes.

`basalt_presentation.rs` projects accepted core state without combat writes.
Native and WASM share its immutable launch positions, charge/release clocks,
flight paths, target-following whip segments, impact selection and cleanup.
The native adapter emits only the resulting visual cues; it never adds damage,
movement, CC, vision, target scans or RNG. New definitions opt into it using
`art_fx_v1: 1`; older embedded definitions retain the existing fallback.

The cone uses 64 direction drawings (maximum angular quantisation 2.8125°),
whip strips 32 (5.625°); exact hit tests are unchanged. Short single-frame
leases prevent animation restarts. Cleanup stops renewals; already-emitted
leases expire within seven ticks because the SDK has no individual FX cancel.
One-shot impact events retain the existing token budget. Persistent updates
emit at most four cues per tick, replacing the old four stamps per wall.

Native alignment, moving-wall pathfinding and combat acceptance remain live
checks. Lab animation is not proof of actual-game rendering. No release or
career installation occurred.

## Real-map review and playable Lab

```text
rtk proxy node editor/extract-game-world.js "C:/Program Files (x86)/Steam/steamapps/common/Teamfight Manager2"
rtk proxy node tools/serve-basalt-review.cjs 7308
```

Open http://localhost:7308/editor/basalt-review.html?build=basalt-art-5 or
http://localhost:7308/editor/basalt.html?art=proposal . Root and /editor/ routes
work; this review server has no installation or career endpoints.

The game-exported collision map has 900/900 artwork-registration matches.
Extracted stock art stays in ignored `editor/game-reference/`, outside the mod.
The same 1,000 world units/pixel applies to bodies, blocks and ranges. The Lab
offers nine stock champions, minions, camps, towers and objectives, with mid-game
laning, gank, countergank, objective and tower scenarios, plus all thirteen stats.

Skills and decisions run in shared Rust/WASM. Host movement, opponents, damage
mitigation and visibility are explicitly simulated. Cone fill uses the real hit
test, including its existing point-blank gap and edge tolerance. The separate
illustrative effect loop is labelled. The playable Lab draws the same packed
effect frames and shared Rust cues as native gameplay. The inspector also
shows every direction/phase from the actual atlas. Simulation runs at 60 ticks
independently of display refresh.

## Verified evidence

- Native: **1,342 passed, 18 ignored**, including 32 Basalt tests. Four existing
  unused-code warnings remain in `native_bridge/detour_tests.rs`.
- Lab crate: **7 passed**, including four presentation tests.
- Native/WASM: **36 fixtures, 10,800 ticks**, complete state/effect equality;
  24 development and 12 holdout inputs. This is transport parity, not balance.
- Browser: **13 checks**, no page errors; actual keyboard jump, routes/assets,
  real map, stock actors, pose/facing inspection, presentation on/off and
  manual-profile trace equality in covered fixtures, shared charge/flight/
  impact/cone cues, emission limits and owner-death cleanup.
- Nine cone boundary comparisons and maximum-range starting-block checks.
  30/60/120/144/240-Hz clock fixtures produce 600 ticks over ten seconds.
- Reproducible packing, bounds, mirrored anchors, one palette, six distinct
  walking leg drawings, timings, hashes and approved body/effects integration.
- Mod Manager: **15 unit tests**, and successful **900-file** comparison in a disposable repo/game
  pair. Real manager configuration, mods, game processes and careers untouched.
- Native adapter art-on/off traces match complete core, damage, movement,
  control, buffs and shields. Presentation tests cover launch/impact clocks,
  owner independence, cleanup and binding positions.
- Five matched synthetic core/projection runs per case: median **0.069 µs**
  inactive, **0.880 µs** with five blocks, **1.328 µs** with synthetic charge
  plus five blocks. SDK queries, rendering and actual Proceed time are excluded.
- Five fresh browser contexts: ready in 182–402 ms; median warm model-step plus
  draw ~0.397 ms. These are Lab observations, not native game performance.

```text
rtk proxy cargo +stable-x86_64-pc-windows-gnu test --manifest-path native/tfm2_custom_ai/Cargo.toml --release -- --test-threads=1
rtk proxy cargo +stable-x86_64-pc-windows-gnu test --manifest-path native/basalt_lab/Cargo.toml --release
rtk proxy cargo +stable-x86_64-pc-windows-gnu build --manifest-path native/basalt_lab/Cargo.toml --release --example trace
rtk proxy node tools/test-basalt-parity.cjs
rtk proxy node tools/test-basalt-art-review.cjs
rtk proxy node tools/test-basalt-cone.cjs
rtk proxy python tools/check-basalt-art-manager.py
rtk proxy python tools/generate-basalt.py --check
rtk proxy python tools/package-basalt.py
```

No installation, career migration, live-game visual acceptance or balance claim.
Artwork approval, integration and reproducible DLL packaging are complete.
