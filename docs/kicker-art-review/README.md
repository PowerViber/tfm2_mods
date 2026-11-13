# Kicker artwork and actual-map Lab

Based on open PR #80 head `2b7da261b1078269b484aa7ac59561fa64c1f023` (kicker-1.0, native 0.10.102, champion pack 0.2.105).

## Approved body

Reihan approved revision 2 on 2026-11-13: **“Approve the revised appearance.”** The approved body has a cobalt/lime jacket, coral headband, broad ivory foot wraps and a jade spray can. The standing idle is 22×44 visible pixels on a 64×56 canvas. World centre is (32,28); feet are (32,48). The rejected rust-red 39-pixel proposal remains historical evidence and is not used by the game.

Complete connected drawings cover idle, eight-frame jogging, basic snap kick, leap, throw, spray-tag, hit, defeat, rooftop perch, short/long hop, special kick and dive. Both facings and compatibility aliases are packed. One palette derived from the approved idle prevents idle/movement lighting changes. Jogging is 48 ticks per cycle; basic contact begins at the canonical ninth tick. Foot and world anchors stay fixed.

The real approved body sheet and fanim are integrated under `mods/tfm2_custom/champions/tfm2_custom_kicker#*`. The approved effects are integrated under `mods/tfm2_custom/vfx/kicker#*`. The later wall-hop correction advances shared rules to `kicker-1.1`; damage, stats, cooldowns and athlete profiles retain the PR baseline values.

**Native pose limitation:** leave `art_v1` disabled. Its legacy Animation CC route would add control merely to display a pose, especially during Rock Throw. Native retains the visible engine-owned idle/movement/basic body. Hop, perch and native technique drawings can be inspected in the Lab; a safe native pose override is not claimed. No body hiding or renewed idle/basic CC was added.

## Approved effects and native presentation

Reihan separately approved the effects on 2026-11-13: **“Approve these effects.”** Original takeoff/landing dust, 15,000-radius Drop-in ring, directional rocks, impacts, kick trails, wall slams, Skyfall streaks, Second Wind tag/return and status marks are integrated. Sources and material pixels remain frozen. Attachment variants add transparent padding only; native and Lab use the same offsets.

The native presenter reads the existing frame and accepted core state. It follows each travelling rock, kicked unit and Skyfall position, and renews the Second Wind anchor only while active. One single-frame lease per update shares the existing token bucket and its critical-impact reserve. Cleanup stops renewals; a previously emitted lease expires within at most ten ticks. Idle presentation allocates no cue list, reads no additional world snapshot and writes no combat state. No hidden targets are used for trails.

A single versioned art manifest, `rules/kicker-art.json`, drives extraction, frame timing and Lab presentation. The builder produces the native body fanim and the shared presentation manifest. Atlas sizes are body 1024×512 and effects 512×512; together they decode to **3,145,728 bytes**, below 32 MiB. Both are below the 2048-pixel limit.

## Reproduction

Sources were generated with the built-in imagegen tool. AI generation itself is not deterministic. Exact prompts are saved in `prompts.json`, `revision-v2-request.json`, `animation-prompts.json`, `supplement-prompt.json` and `effects-prompt.json`. Approved/frozen source hashes are in `approvals.json` and `frozen-sources.json`.

Run:

```powershell
rtk proxy python tools/build-kicker-art.py
rtk proxy python tools/generate-kicker.py
rtk proxy python tools/verify-kicker-art.py
```

The verifier rebuilds packing twice and checks hashes, bounds, timing, mirrored pixels, foot anchors, body/FX integration, attachment pixels and unchanged damage/stats/cooldowns. The only balance-rule edits are the wall-hop range and rules revision; state-transition tests cover persistent walls and interruption. The package builder separately verifies two identical DLL builds. The original concept extractor remains for historical comparisons; the complete-art builder owns production outputs.

## Playable Lab and review

The requested wall pair is 226,274 units apart in the actual map. Wall-to-wall range is now **300,000** (previously 60,000), with unchanged charges, cooldown, speed and initial 40,000 mount range. Roof is an accepted stance: walking or a coarse ground-cell classification cannot automatically dismount him. Hard control interrupts even a long wall hop; deliberate Drop Kick/Skyfall still lands with the normal Drop-in. Death and Second Wind retain their original lifecycle rules.

The Lab canvas is square rather than stretched by the sidebar. A range guide appears while on a wall; Shift+Q issues Drop-in and a stun button exercises interruption. The host model replaces an interrupted glide instead of resuming an old trajectory after landing.

Run `tools/start-kicker-review.ps1` for port 7306.

- `editor/kicker-review.html`: every complete pose/facing/frame, native-size/3× comparisons, real-map inspection and proposed effect sequences.
- `editor/kicker.html`: extracted installed map, all 900 wall cells at 960,000-unit map width, stock champions/minions/camps/objectives/towers, and eight mid-game scenario fixtures.
- Controls: focus arena, WASD movement, Space basic, Q wall mount/hop, E rock, C kick/Skyfall, Shift+C Flash kick, F marked dash, R Second Wind. Blur clears held input.
- Pause, stepping, collider edits, full-map/follow views, thirteen athlete controls, artwork toggle and modelled opponent behaviour.
- Fixed 60 Hz simulation is independent of display refresh. Sprite frame selection uses simulation time, not repeated effect renewal.
- Native/WASM core remains authoritative. Browser movement, mitigation, visibility, shields, opponent basic attacks and turret behaviour are labelled models. Stock ability AI and actual game targeting/attribution are not simulated authoritatively.
- Extracted game-reference files stay in the ignored local cache; they are not redistributed as original project art.

## Evidence

Fresh checks for the approved artwork and wall-state correction:

| Check | Result |
|---|---|
| Native GNU release suite | 1,259 passed, 18 ignored; four inherited unused-helper warnings in native_bridge/detour_tests.rs |
| Kicker Lab | Three tests passed |
| Browser | 25 checks passed, including the actual photographed wall pair, persistent stance, deliberate Drop-in, mid-hop stun and square map scale |
| Native/WASM | 213 complete output frames identical |
| Artwork enabled/disabled | Identical core inputs, outputs and model combat state |
| Packing | Reproducible hashes; 294 animation-frame entries checked |
| Mod Manager tests | 15 passed, including the repository champion-load check |
| Isolated Mod Manager comparison | 890 files identical; no installation or migration |

Machine-readable results are in `automated-results.json`, `browser-complete.json`, `verification.json`, `package.json` and `mod-manager.json`. Historical body-only evidence remains under `body-phase-*`; v1/v2 evidence remains separate.

## Pending gates

Reihan explicitly deferred real-game checks to focus on artwork. Native wall standing/targetability/safe landing, kick/carry placement, Second Wind against towers/true damage with no kill, and real Flash cooldown use remain pending.

Safe native special-pose presentation, actual-game visual acceptance, installation and backed-up career migration remain separate. PR #80 retains its unused release reservations: native 0.10.102 and pack 0.2.105. The DLL and WASM are built from the final sources; `package.json` records two identical DLL builds and their hashes. No installation or migration is performed.
