# Kicker artwork and actual-map Lab

Based on open PR #80 head `2b7da261b1078269b484aa7ac59561fa64c1f023` (kicker-1.0, native 0.10.102, champion pack 0.2.105).

## Approved body

Reihan approved revision 2 on 2026-11-13: **“Approve the revised appearance.”** The approved body has a cobalt/lime jacket, coral headband, broad ivory foot wraps and a jade spray can. The standing idle is 22×44 visible pixels on a 64×56 canvas. World centre is (32,28); feet are (32,48). The rejected rust-red 39-pixel proposal remains historical evidence and is not used by the game.

Complete connected drawings cover idle, eight-frame jogging, basic snap kick, leap, throw, spray-tag, hit, defeat, rooftop perch, short/long hop, special kick and dive. Both facings and compatibility aliases are packed. One palette derived from the approved idle prevents idle/movement lighting changes. Jogging is 48 ticks per cycle; basic contact begins at the canonical ninth tick. Foot and world anchors stay fixed.

The real approved body sheet and fanim are integrated under `mods/tfm2_custom/champions/tfm2_custom_kicker#*`. The only champion-definition change is its sprite path. Rules, native controller/core, WASM, athlete profiles and packaged DLL remain byte-identical to the PR baseline.

**Native pose limitation:** leave `art_v1` disabled. Its legacy Animation CC route would add control merely to display a pose, especially during Rock Throw. Native retains the visible engine-owned idle/movement/basic body. Hop, perch and native technique drawings can be inspected in the Lab; a safe native pose override is not claimed. No body hiding or renewed idle/basic CC was added.

## Effects — separate approval pending

Original takeoff/landing dust, 15,000-radius Drop-in ring, directional rocks, impacts, kick trails, wall slams, Skyfall streaks, Second Wind tag/return and status marks are previewed in `effects-comparison.png` and the animated review. They remain outside the game package. Body approval does not approve these effects.

A single versioned art manifest, `rules/kicker-art.json`, drives extraction, frame timing and Lab presentation. The builder produces the native body fanim and the shared presentation manifest. Atlas sizes are body 1024×512 and effects 512×512; together they decode to **3,145,728 bytes**, below 32 MiB. Both are below the 2048-pixel limit.

## Reproduction

Sources were generated with the built-in imagegen tool. AI generation itself is not deterministic. Exact prompts are saved in `prompts.json`, `revision-v2-request.json`, `animation-prompts.json`, `supplement-prompt.json` and `effects-prompt.json`. Approved/frozen source hashes are in `approvals.json` and `frozen-sources.json`.

Run:

```powershell
rtk proxy python tools/build-kicker-art.py
rtk proxy python tools/generate-kicker.py
rtk proxy python tools/verify-kicker-art.py
```

The verifier rebuilds packing twice, checks hashes, bounds, timing, mirrored pixels, foot anchors, body integration, absent unapproved FX and unchanged native mechanics/DLL. The original concept extractor remains for reproducing historical comparisons; the complete-art builder owns production outputs.

## Playable Lab and review

Run `tools/start-kicker-review.ps1` for port 7306.

- `editor/kicker-review.html`: every complete pose/facing/frame, native-size/3× comparisons, real-map inspection and proposed effect sequences.
- `editor/kicker.html`: extracted installed map, all 900 wall cells at 960,000-unit map width, stock champions/minions/camps/objectives/towers, and eight mid-game scenario fixtures.
- Controls: focus arena, WASD movement, Space basic, Q wall mount/hop, E rock, C kick/Skyfall, Shift+C Flash kick, F marked dash, R Second Wind. Blur clears held input.
- Pause, stepping, collider edits, full-map/follow views, thirteen athlete controls, artwork toggle and modelled opponent behaviour.
- Fixed 60 Hz simulation is independent of display refresh. Sprite frame selection uses simulation time, not repeated effect renewal.
- Native/WASM core remains authoritative. Browser movement, mitigation, visibility, shields, opponent basic attacks and turret behaviour are labelled models. Stock ability AI and actual game targeting/attribution are not simulated authoritatively.
- Extracted game-reference files stay in the ignored local cache; they are not redistributed as original project art.

## Evidence

Fresh checks for the completed approved body:

| Check | Result |
|---|---|
| Native GNU release suite | 1,251 passed, 18 ignored; four inherited unused-helper warnings in native_bridge/detour_tests.rs |
| Kicker Lab | Three tests passed |
| Browser | 20 checks passed, eight scenario fixtures, 55 actors with stock sprite keys |
| Native/WASM | 213 complete output frames identical |
| Artwork enabled/disabled | Identical core inputs, outputs and model combat state |
| Packing | Reproducible hashes; 285 animation-frame entries checked |
| Mod Manager tests | 15 passed, including the repository champion-load check |
| Isolated Mod Manager comparison | 888 files identical; no installation or migration |

Machine-readable results are in `automated-results.json`, `browser-complete.json`, `verification.json` and `mod-manager.json`. Historical v1/v2 evidence remains separate.

## Pending gates

Reihan explicitly deferred real-game checks to focus on artwork. Native wall standing/targetability/safe landing, kick/carry placement, Second Wind against towers/true damage with no kill, and real Flash cooldown use remain pending.

Effects approval, safe native special-pose presentation, release packaging, actual-game visual acceptance, installation and backed-up career migration remain separate. No version or DLL was changed by this artwork pass.
