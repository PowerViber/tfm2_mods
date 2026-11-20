# Round 187 — Contender 3.4

Native **0.10.122** · Champion pack **0.2.125** · Baseline [round 186, PR #102](https://github.com/PowerViber/tfm2_mods/pull/102)

## Gameplay

- Shadow Flurry delivers up to 30 punches over 1.5 seconds, one every three simulation ticks. Each deals `2 + floor(2% Attack)` physical damage before Rhythm and mitigation.
- Punches appear on their targets. Contender reappears at his current position, even if the sequence ends early. Four charges, the 12-second recharge and immediate recast eligibility follow round 186.
- The game keeps its level-three S2 slot and runs the same cast-marker path and movement decisions.

## Matched Lab comparison

Level-eight Contender at 151 Attack on the same verified game-map reference. The targets were unlimited-HP stock Knights. The combo was one Flurry, one basic and its Pact echo. HP damage uses the Lab's simulated mitigation.

| Targets | Flurry raw 3.3 → 3.4 | Combo raw | Combo HP damage | Rhythm stacks on most-stacked enemy | Shadow Break raw | Haymaker raw |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 60 → 150 | 274 → 395 | 115 → 175 | 15 → 20 | 0 → 26 | 48 → 49 |
| 3 | 60 → 150 | 260 → 350 | 108 → 153 | 8 → 15 | 0 → 0 | 45 → 45 |
| 5 | 60 → 150 | 260 → 350 | 108 → 153 | 8 → 15 | 0 → 0 | 45 → 45 |

Rhythm added zero raw damage to the individual punches at this Attack due to integer rounding. On one target the fully stacked follow-up basic gained four additional raw damage over 3.3, and triggered Break. Flurry emitted 17 → 32 presentation cues, including the vanish and reappearance effects. Both revisions earned zero recharge credit from Flurry punches.

## Native performance and checks

| Mock host configuration | 3.3 median µs/tick | 3.4 median µs/tick |
| --- | ---: | ---: |
| Idle with no targets | 0.911 | 0.942 |
| Flurry with one target | 2.696 | 2.867 |
| Flurry with three targets | 3.426 | 3.776 |
| Flurry with five targets | 4.422 | 4.401 |

The adapter microbenchmark uses seven timed samples of 30 independent casts per target count after one warm-up sample. These timings measure the native adapter on the mock host. Live match frame time and Proceed time require separate measurements.

- Windows GNU native suite passed 1,413 tests. The regular run skipped 19 tests, including opt-in benchmarks. The two ignored tick-budget checks and the Flurry microbenchmark passed when run explicitly. Windows emitted four existing dead-code warnings in the native bridge tests.
- The Lab transport passed three tests. Native and browser WASM returned identical complete state and effects across 36 scenarios, 5,508 frames and 390 Flurry punches.
- Browser checks passed for 900 registered terrain cells, final punch timing, in-place reappearance, immediate recast, recharge attribution, Lab dodge exchange, Counterpunch, Shadow Link and older recording inspection. The asset check passed for all eight directions and legacy pixels.
- All 15 Mod Manager tests passed, including champion loading. In an isolated install fixture with Rust and Git hidden from PATH, Update selected the shipped DLL. Check reported **UP TO DATE**, with 795 champion files compared and an identical installed DLL.
- Contender, roster and Lab target catalogue generation checks passed.

## Artifacts and live acceptance

| Packaged file | SHA-256 |
| --- | --- |
| `mods/tfm2_custom_ai/tfm2_custom_ai.dll` | `2223adb503fd13c4f7f926edc7de307fad61c65e35b31d862fc9e1fbb77ab872` |
| `editor/contender.wasm` | `e73f0b5ec96652e452deb8bdab019d51d8beb6edecfadb3cd695088b241ebe46` |

The Windows GNU release build used locked dependencies, `SOURCE_DATE_EPOCH=1795146379` from baseline `5c9b749`, and the GNU linker's `--no-insert-timestamp` option. The DLL contains version `0.10.122` and the WASM contains rules revision `contender-3.4`.

Actual-game acceptance is pending. In a live match, check reappearance after both a full cast and a target lost mid-cast, repeated S2 casts by the game's AI, hit readability and frame time in a crowded fight. Installation and career migration are separate.
