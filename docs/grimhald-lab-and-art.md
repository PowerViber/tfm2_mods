# Grimhald Lab and complete connected-body animation

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

Revision 6 completes **all eight cycles**: idle 4, run 6, attack 4,
Bulwark 4, Bash 4, throw 4, hit 1 and death 6. Every source pose is a complete
connected drawing. The axe is a separate rigid accessory; only that accessory is
rotated and translated. Body anatomy is never sliced into a rig.

The repository now includes the actual base body, weapon and VFX sheets under
mods/tfm2_custom, with identical packed preview pixels in the Lab.
New definitions opt into the weapon presenter through weapon_layer=1.
The always-visible engine-owned body is unarmed, including on draft/roster
screens; the in-match accessory supplies the axe. Older embedded definitions
without that opt-in retain a visible unarmed fallback and need a separately
authorized definition migration before they show the accessory.

The axe disappears during flight, return, leap and planted-axe combat. The throw
ends empty-handed; death clears the accessory and uses the engine-owned complete
six-frame corpse sequence. A local read-only alias gives the planted axe a
dedicated ground-aligned phase without changing its raw core event identity or
emission cadence. Existing VFX frame pixels are unchanged. The eight flying-axe phases now last one tick to match their per-tick core emission; other existing effect durations are preserved.

Final native body/accessory clock and facing alignment still require actual-game
observation. The Lab's travel-driven gait is not proof of engine-owned playback.
No idle/basic animation CC, body hiding or injected inputs were added.

The fissure is now a flat top-down ground crack. Native and Lab choose its
directional alias through the same read-only resolver; the raw combat event name,
damage, geometry, timing and RNG are unchanged. The Lab projects body feet onto
the same ground plane as the crack. Only the endpoint raises a pillar. The older
cutout generator must not be reused.

The current review is `editor/grimhald-review.html`. Frozen sources, prompts,
hashes and actual-size comparisons are in `docs/grimhald-art-review`.
Image generation is not deterministic; extraction and packing are reproducible.

The current complete review is available at
http://localhost:7296/editor/grimhald-review.html?build=body-6.
Every animation has an armed/unarmed, both-facing GIF at actual size and 3x.
Sources, prompts and hashes are frozen under docs/grimhald-art-review.

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

The gameplay core and rules remain unchanged from PR head **bfe6f12**.
Only presentation bindings, the opt-in visual accessory and its read-only effect
alias changed. The shared Lab/core suite passes **35** tests. Browser checks pass
**35** checks over **24 development + 12 holdout scenarios**, including complete
pose coverage, weapon travel/return, planted state, death freeze, replay and
full native/WASM trace parity. Current exact results live in lab-results.json.

Art validation reproduces whole-frame extraction and packing, verifies every
frame/facing/anchor/duration, exact idle/walk source-body pixels, unchanged existing
VFX pixels, one-tick flying phases and matching game/Lab sheets. The three final atlases decode to
**16 MiB**, each at most **2048 pixels**. The actual base idle has nonzero alpha,
so it remains visible without a native presenter on roster cards.

The Mod Manager champion-load test passes. The art validator additionally checks
real asset paths, animation tags, weapon bindings and exact copied pack files.
Neither establishes actual-game rendering acceptance. Asset and WASM hashes are
in asset-hashes-v6.json; no release DLL/package hash is claimed.

## Verification and handoff

Commands (Windows GNU toolchain):

```text
python tools/generate-grimhald.py --check
python tools/prepare-grimhald-connected.py
python tools/prepare-grimhald-walk.py
python tools/prepare-grimhald-consistency.py
python tools/prepare-grimhald-complete.py
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
