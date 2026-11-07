# Grimhald Lab and connected-body revision

The playable Lab uses the same Rust combat core as native gameplay. Serve
`editor/grimhald.html` through the editor server, or run the read-only review
server with `node tools/serve-grimhald-review.cjs` (port 7296).

The current rules are **grimhald-2**, transport v1. Q is Ghost/recast, E is
Bulwark/relocate, C is Bash, R throws the axe; R during flight supplies a steering
intention. Space is the simulated host basic. WASD, play/pause/step, targets,
terrain, fog, incoming attacks/CC, recordings and thirteen profile controls are
available. The core directly weights Judgement, Accuracy and Aggressive; the Lab
does not invent new weights for the other ten ratings.

## Latest user correction

The first full animation attempt used separately transformed upper/lower body
regions. The user rejected its disconnected appearance and static legs. This is
an artwork failure, not an accepted animation set. The source and evidence are
retained under `docs/grimhald-art-review/v1` for the handoff.

Revision 3 supplied complete connected key poses. The user then requested a real,
more fluid walk instead of its single run drawing, and a fissure lying on the
floor rather than an upright rock silhouette.

Revision 4's complete walk and flat floor correction were approved. A later
review found the older idle too dark, visually inconsistent and differently sized.
The same review requested walking that looks like his actual travel speed.

Revision 5 replaces that older idle with four full-body drawings per weapon
state, referenced directly from the approved walking identity. The 39-pixel
standing body shares the walk's helmet landmark, steel/crimson palette and fixed
foot anchor. The six approved walk drawings are pixel-unchanged; their nominal
duration is now **160 ms/frame** (a 0.96-second cycle instead of 0.48).
The live Lab advances gait phase by actual ground travel, one cycle per
**60,000 world units**, so slowing, stopping and speed bonuses do not leave a
time-driven sprint or foot shuffle. Repeated rendering and pause do not advance
the gait; recordings reconstruct it. This does not change gameplay movement speed.

Unfinished attack/cast/hit/death drawings explicitly use the matched idle as a
placeholder rather than displaying the superseded dark concept. Their complete
cycles, final body/weapon asset integration and native alignment remain pending.
The native accessory presenter remains disabled; the Lab's distance-driven body
animation is not proof of engine-owned body playback in the game.

The fissure is now a flat top-down ground crack. Native and Lab choose its
directional alias through the same read-only resolver; the raw combat event name,
damage, geometry, timing and RNG are unchanged. The Lab projects body feet onto
the same ground plane as the crack. Only the endpoint raises a pillar. The older
cutout generator must not be reused.

The current review is `editor/grimhald-review.html`. Frozen sources, prompts,
hashes and actual-size comparisons are in `docs/grimhald-art-review`.
Image generation is not deterministic; extraction and packing are reproducible.

The axe is absent from the Lab body during outgoing/return flight, the leap and the current planted-axe state.
The native weapon presenter is implemented and tested through the mock host, but remains disabled until the approved full-body and weapon frames are integrated. The SDK cannot remove a baked-in axe
from the old base sprite. Do not hide the champion globally, renew idle Animation
CC or claim an overlay covers the old weapon. A visible unarmed base plus a
pose-aligned weapon accessory is the supported implementation direction.

## Authorized gameplay revision (prototype defaults)

| Feature | Original PR head | Current candidate |
|---|---:|---:|
| Bulwark range | 38,000 | 70,000 |
| Preparation | 18 ticks | 36 ticks |
| Fissure | Instant full line | 18-tick advancing sweep, one hit per target |
| Pillar | With impact | 12 ticks after fissure arrival |
| Bash range | 22,000 | 52,000 |
| Bash knockback | 24,000 | 50,000 |
| Ultimate impact radius | 36,000 | 40,000 |
| Cage radius | 55,000 | 62,000 |
| Axe steering | None | 10 degrees total turn budget, 0.25 degrees/tick |

The later Claude Round 2 changes (commit 7426636) include the 8-second cage, pillar nudge, unarmed fists and planted-axe aura described in grimhald-guide.md. This presentation correction preserves that entire combat core and its canonical rules; it does not restore the earlier numbers. This later user request explicitly supersedes
the earlier art-only scope. Ornn's Volcanic Rupture and Alistar's Headbutt supply
the requested attack identities; their artwork and numerical balance are not
copied. Official reference pages: [Ornn](https://www.leagueoflegends.com/en-us/champions/ornn/)
and [Alistar](https://www.leagueoflegends.com/en-us/champions/alistar/).

The core selects a visible eligible champion near the initial landing point.
Steering never reads a hidden target's current position. Each turn consumes the
remaining ten-degree budget, including reversals. Recasting changes an intention,
not range, speed, preparation or cooldown. The warning projects the completed
current heading; later steering can update it. Shared movement-assistance
settings retain their existing defaults.

## Current visual verification

The correction is based on PR head **7426636** and preserves its complete
combat core and gameplay rules. The shared Lab/native core tests pass **34**.
The latest browser count and full native/WASM trace results are recorded in
docs/grimhald-art-review/lab-results.json; earlier full-native totals in the
delivery report are explicitly earlier evidence, not a fresh full-suite claim.
Art checks reproduce extraction/packing, compare exact approved walk pixels and
mirrors, verify fixed feet/body scale and bound the idle/walk steel-luminance
difference. Current loaded preview textures total **10,547,200 decoded bytes**;
final complete assets must be budgeted again.

## Verification and handoff

Commands (Windows GNU toolchain):

```text
python tools/generate-grimhald.py --check
python tools/prepare-grimhald-connected.py
python tools/prepare-grimhald-walk.py
python tools/prepare-grimhald-consistency.py
python tools/validate-grimhald-art.py
cargo +stable-x86_64-pc-windows-gnu test --release
cargo +stable-x86_64-pc-windows-gnu build --release --lib --target wasm32-unknown-unknown
```

Run the Cargo commands in `native/grimhald_lab`; copy its lib WASM to
`editor/grimhald.wasm`. Build the native runner separately with
`cargo ... build --release --bin grimhald_lab` to avoid a WASM bin/lib filename
collision. `node tools/test-grimhald-lab.cjs` checks real keyboard/focus input,
recordings, the actual editor server, manual profile independence and complete
native/WASM traces across 24 development and 12 holdout scenarios.

Earlier evidence is explicitly archived in `v1`. That revision passed 1,005
native tests with 13 ignored when serial, 25 Lab tests, 15 browser checks and
4,716 parity frames. The parallel full suite had an unrelated Raze cache test
failure that passed individually and serially; do not describe that run as a
clean pass. Four existing Windows-only detour-test dead-code warnings remain.

Current results are in `lab-results.json`, `art-validation.json` and the delivery
report. Lab movement, basics, mitigation, visibility and cage enforcement are
host models. Native mock tests and Lab scenes do not establish live stock-engine
isolation, shield/item behaviour or actual damage acceptance.

**Release remains gated:** revised artwork approval, complete connected
animations, native weapon-layer integration, Mod Manager verification of the
final assets, balance harness, version/DLL/package work and actual-game checks.
Installation and real-career migration remain separate. Do not merge/install the
draft while the final Grimhald body/effect assets are absent.
