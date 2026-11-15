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
- **Effects remain unapproved.** Subsequent answers were corrections, not
  integration approval. Candidate effects stay outside `mods/`.

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
rtk proxy python tools/generate-basalt.py
rtk proxy python tools/verify-basalt-art.py
```

Body files: `mods/tfm2_custom/champions/tfm2_custom_basalt#*`. The review
`body.png`, `body.fanim`, `body-presentation.json` and GIFs show the same frames.
Atlas: 1024×2048, 8 MiB decoded. Body plus candidate effects: 9,306,112 bytes,
under 32 MiB; no atlas exceeds 2048 pixels. Effects are not packaged yet.

## Native presentation limits and release gates

The engine selects ordinary idle, movement, basic, hit and death poses.
Additional action drawings are inspectable in the Lab. This pass does not
claim supported selection of every owner pose during passive-only native
skills. No hiding, renewed Animation CC, idle/basic locks or cosmetic movement
restrictions were added to force a pose.

The PR's original packaged DLL is byte-identical. The far-corner correction is
in source and rebuilt WASM, and still requires DLL release packaging. Native
effects integration and boulder/meteor/tether bindings await effects approval.
Native alignment, moving-wall pathfinding and combat acceptance remain live
checks. Lab animation is not proof of native rendering.

## Real-map review and playable Lab

```text
rtk proxy node editor/extract-game-world.js "C:/Program Files (x86)/Steam/steamapps/common/Teamfight Manager2"
rtk proxy node tools/serve-basalt-review.cjs 7308
```

Open http://localhost:7308/editor/basalt-review.html?build=basalt-art-4 or
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
illustrative effect loop is labelled. Simulation runs at 60 ticks independently
of display refresh.

## Verified evidence

- Native: **1,336 passed, 18 ignored**, including 26 Basalt tests. Four existing
  unused-code warnings remain in `native_bridge/detour_tests.rs`.
- Lab crate: **3 passed** (two transport, one integration).
- Native/WASM: **36 fixtures, 10,800 ticks**, complete state/effect equality;
  24 development and 12 holdout inputs. This is transport parity, not balance.
- Browser: **10 checks**, no page errors; actual keyboard jump, routes/assets,
  real map, stock actors, pose/facing inspection, presentation on/off and
  manual-profile trace equality in covered fixtures.
- Nine cone boundary comparisons and maximum-range starting-block checks.
  30/60/120/144/240-Hz clock fixtures produce 600 ticks over ten seconds.
- Reproducible packing, bounds, mirrored anchors, one palette, six distinct
  walking leg drawings, timings, hashes and approved-body-only integration.
- Mod Manager: successful **898-file** comparison in a disposable repo/game
  pair. Real manager configuration, mods, game processes and careers untouched.

```text
rtk proxy cargo +stable-x86_64-pc-windows-gnu test --manifest-path native/tfm2_custom_ai/Cargo.toml --release -- --test-threads=1
rtk proxy cargo +stable-x86_64-pc-windows-gnu test --manifest-path native/basalt_lab/Cargo.toml --release
rtk proxy cargo +stable-x86_64-pc-windows-gnu build --manifest-path native/basalt_lab/Cargo.toml --release --example trace
rtk proxy node tools/test-basalt-parity.cjs
rtk proxy node tools/test-basalt-art-review.cjs
rtk proxy node tools/test-basalt-cone.cjs
rtk proxy python tools/check-basalt-art-manager.py
rtk proxy python tools/generate-basalt.py --check
```

No installation, career migration, live-game visual acceptance or balance claim.
Effects approval and release packaging remain pending.
