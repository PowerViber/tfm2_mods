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

Revision 3 supplies **complete connected imagegen key poses** with slightly
narrower armour: standing, run, shoulder-first headbutt and unarmed. They are
review-only, pending appearance approval and full animation completion. The Lab
clearly labels these four static key poses. Do not count them as completed walk,
attack, cast or death cycles. The older cutout generator must not be reused.

The current review is `editor/grimhald-review.html`. Frozen sources, prompts,
hashes and actual-size comparisons are in `docs/grimhald-art-review`.
Image generation is not deterministic; extraction and packing are reproducible.

The axe is absent from the Lab body during outgoing/return flight and the leap.
Native unarmed presentation still needs the approved full-body animation set
and a separate weapon presentation layer. The SDK cannot remove a baked-in axe
from the old base sprite. Do not hide the champion globally, renew idle Animation
CC or claim an overlay covers the old weapon. A visible unarmed base plus a
pose-aligned weapon accessory is the supported implementation direction.

## Authorized gameplay revision (prototype defaults)

| Feature | Original PR head | Current candidate |
|---|---:|---:|
| Bulwark range | 38,000 | 54,000 |
| Preparation | 18 ticks | 36 ticks |
| Fissure | Instant full line | 18-tick advancing sweep, one hit per target |
| Pillar | With impact | 12 ticks after fissure arrival |
| Bash range | 22,000 | 40,000 |
| Bash knockback | 24,000 | 36,000 |
| Ultimate impact radius | 36,000 | 40,000 |
| Cage radius | 55,000 | 62,000 |
| Axe steering | None | 10 degrees total turn budget, 0.25 degrees/tick |

Damage coefficients, cooldowns, HP, growth, Mod Power, slow/stun/airborne durations
and AI weights remain unchanged. This later user request explicitly supersedes
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

## Verification and handoff

Commands (Windows GNU toolchain):

```text
python tools/generate-grimhald.py --check
python tools/prepare-grimhald-connected.py
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
