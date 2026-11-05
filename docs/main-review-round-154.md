# Round 154: heavy Rally nerf (native 0.10.89, tfm2_custom 0.2.91, Valorant 0.1.32)

You asked for a heavy nerf to Rally: range, area, ball speed and, if needed, damage, with a check on her damage output and snowballing.

## What the numbers showed (0.10.88)
`rally_damage_report` (new; run with `--ignored`) gives one spike at the centre as a share of an average scripted peer's max HP (armour ignored).

| Level | Normal | Thunder | Black | White | Impossible |
|---|---|---|---|---|---|
| 1 | 16% | 39% | 55% | **83%** | **288%** |
| 6 | 13% | 33% | 46% | **70%** | **243%** |
| 10 | 12% | 30% | 42% | **64%** | **223%** |
| 15 | 11% | 28% | 39% | **60%** | **207%** |

- **Reach:** spike 240,000 (a quarter of the map), Serve 560,000, Impossible 400,000. A Serve Impossible was map-wide.
- **Blasts:** spike 45,000, Black and White 65,000, Serve 90,000, Impossible 120,000 (144,000 with overpressure).
- **Ball speed:** 180,000/s for a normal spike, up to 840,000/s for White.
- **Snowball:** forms last until death. White added +40 power, 15 power per metre of height and a ×2.35 multiplier, with a bigger blast and faster ball. So once White, every play was a 60–80% hit in a large area from a quarter of the map away.

## Changes (rules/rally.json)
| Rule | 0.10.88 | 0.10.89 |
|---|---|---|
| Spike range | 240,000 | **110,000** |
| Serve range | 560,000 | **260,000** |
| Impossible range | 400,000 (Serve: whole map) | **200,000** (Serve: 260,000) |
| Spike radius | 45,000 | **25,000** |
| Black / White radius | 65,000 | **35,000** |
| Serve radius | 90,000 | **50,000** |
| Impossible radius / local | 120,000 / 80,000 | **70,000 / 50,000** |
| Overpressure radius | +20% | **+10%** |
| Storm patch radius | 18,000 | **12,000** |
| Ball speed (per tick) normal / Thunder / Black / White / volley | 3,000 / 6,000 / 10,000 / 14,000 / 5,000 | **1,800 / 3,200 / 5,000 / 7,000 / 3,500** |
| Speed cap, overpressure threshold | 5,120,000, 2,560,000 | **2,560,000, 900,000** |
| Form power Black / White | +25 / +40 | **+10 / +20** |
| Power per metre of height | 15 | **8** |
| Multipliers Thunder / Black / White | 155–185 / 195 / 235 | **125–140 / 150 / 175** |
| Spike % of Attack | 65 | **55** |
| Impossible multiplier, charge power | ×6, +200 | **×3, +100** |

- **Unchanged:** palm and volley. A new `PALM_PERCENT` knob, set to 100, is left in place.
- **Overpressure:** the threshold moved with the slower balls, so a fully charged Impossible and a five-bar White spike still reach it.

## After (0.10.89)
| Level | Normal | Thunder | Black | White | Impossible |
|---|---|---|---|---|---|
| 1 | 14% | 24% | 29% | 41% | 86% |
| 6 | 11% | 19% | 24% | 34% | 71% |
| 10 | 10% | 18% | 22% | 31% | 65% |
| 15 | 9% | 16% | 20% | 28% | 59% |

- **Speed:** a normal spike travels 108,000/s and a White one 420,000/s, so they can be dodged.
- **Harness duel average:** 33.7% in the Lab (53.1% before) and 22.2% as shipped (37.5%).
- **Snowball check:** in the level-15, 3-minute alone fights she now has fewer kills than deaths against every opponent except Raze (even).
- **Lane:** she still kills the stock ranged laner within a minute. Her basic attacks are untouched (palm 42% of her champion damage).
- If she now feels too weak in game, the knobs in order:
  1. spike % (55 → 60);
  2. Black and White multipliers;
  3. `PALM_PERCENT`, if her early lane is still too strong.

## Checks
- `cargo test --release`: 983 passed, 0 failed.
- Pinned tests now use the rule constants where they test geometry, and carry the new values with the old ones noted where they test damage. None were deleted.
- `rally_lab`: 130 passed, and `editor/rally.wasm` is rebuilt.
- `generate-rally --check`, `generate-roster --check` and `verify-rally.py` pass.
- The DLL reports 0.10.89.
