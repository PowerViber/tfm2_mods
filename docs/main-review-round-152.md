# Round 152: Contender hits, Rally walls, one body, spike targets, damage and jump (native 0.10.87)

Native 0.10.87, tfm2_custom 0.2.89, Valorant 0.1.30 (both require 0.10.87). Built on main after PR #64.

## What you reported (Nov 5 test, two videos and the scoreboard)
- Contender still can't hit enemies, creeps or minions.
- Rally ("Faker") goes back and forth at a wall until she dies, while her ball sits on a wall tile.
- A move shows two Rally bodies side by side.
- Her spike locks onto the first enemy on its path, even a minion.
- Her jump looks too fast.
- **Your decisions:**
  - Remove the Impossible cap but lower her damage overall, by about 15%.
  - Slow the jump to 1.2 s.
- Raze is good now.

## Contender
**Cause:** the bridge puts a Bind on him on every tick of a punch, to hold him still, and an attack lock (`BlockAttack`) for the whole cast. The next frame read his own Bind as "rooted", and the core never sends a dash while rooted. So since 0.10.84 every Combination, Body Shot and Uppercut swung from where he stood, short of targets the planner chose expecting a 10,000–14,000 dash.

**Fixes:**
- **Named locks:** his own locks are now named `ct_motion` and `ct_lock`, and they never count as rooted or interrupted. This is the same rule as Rally's `rly_motion`.
- **Basics during a cast:** basic attacks the game accepted land even during a cast. Their tags are cleared every tick, so one skipped while busy was lost for good. Only the jab and cross pose waits for the cast to end.
- **Target check:** an accepted basic only asks whether the target is alive and on the other team. `is_targetable` is unverified on camps and minions, and it reads false when the host slot is missing.
- **Dash from a wall margin:** a dash may leave the margin of a wall cell he already stands in, as he does in camp pockets, but never enter the wall itself.
- **Expired rings:** a ring past its end, a return ledger still retrying an exact return, no longer counts as a realm. It fences nobody in.
- **Glue tests (new, `glue_tests.rs`):** a stateful mock host runs `contender.rs` itself. It records positions set, CCs applied and read back with their names, and damage calls. The tests check:
  - a punch dashes toward a champion and damages it;
  - basics land on an untargetable minion and an untargetable camp monster;
  - a basic accepted mid-cast lands.

## Rally
- **Walls:**
  - **Where the ball rests:** a grounded ball now rests where her body can stand (her radius, not the ball's).
  - **Walking to it:** she takes a straight step if it's clear. If not, she turns up to 150° and keeps to the same side, so she follows the wall round instead of flipping sides every tick.
  - **Giving up:** after 90 ticks (1.5 s) without getting 1,000 closer, the ball flies back to her hand, like a spike. This also covers danger keeping her away.
  - **Before:** she stepped straight at the wall forever while the game's AI pulled her back.
- **One body:** her form body is drawn as an overlay, which never flips, so it picks `_l` or `_r` from our facing rule. Two mistakes put it facing away from the game's own yellow sprite, mirrored and offset:
  - it turned toward the nearest visible enemy anywhere on the map, where it should only turn toward one within 90,000, as the game does and as Minato's overlay does;
  - her own move teleports counted as walking.

  Both are fixed (`Facing::placed`). With the same facing, the form body covers at least 90% of the base outline for every ground pose; `verify-rally.py` now checks this. The in-air hide is held 2 ticks, not 1.
- **Spike targets:** the ball remembers what it was aimed at. Only that target, or an enemy champion stepping into the path, stops it. Minions and camps on the way are passed. Splash at the impact is unchanged. Contact also prefers a champion in range over any minion or camp.
- **Jump:** 1.2 s instead of 0.8 s. `FLIGHT_TICKS` and `TOSS_TICKS` are both 72, and contact is at the top of the jump, ±7 ticks for imperfect athletes. Heights are unchanged, so the arc is slower.
- **Damage (rally-3.5 rules; the revision name stays `rally-3.3` because the Lab's art paths use it):**
  - Impossible is **uncapped**; the 70%-of-max-HP cap is gone. The Receive bank still lands once on top.
  - Spike 50% → **42%**.
  - Black 180% → **160%**.
  - White 220% → **190%**.
  - Impossible ×8 → **×6**.
  - **Thunder** is now 130% + height bonus, at most 150%. It used to reach 230%, more than Black and White.
- **Tests:** the pinned damage tests now carry the new values, with the old ones noted. None were deleted. Timing fixtures use `FLIGHT_TICKS / 2` instead of 24.

### Harness (scripted peer models)
| | Before (0.10.86) | Damage cut only | Damage cut + 1.2 s jump (shipped) |
|---|---|---|---|
| Duel average, verified receipts (Lab) | 56.4% | 43.4% | **35.6%** |
| Duel average, native as shipped | 44.4% | 33.3% | **24.1%** |

Per opponent at L1/L6/L10/L15 (shipped rules, verified receipts):

| Opponent | Win % (L1 / L6 / L10 / L15) |
|---|---|
| Levi | 21 / 0 / 0 / 0 |
| V1 | 100 / 0 / 0 / 0 |
| Minato | 100 / 42 / 8 / 0 |
| David | 17 / 4 / 0 / 0 |
| Raze | 100 at every level |
| Contender | 96 / 29 / 13 / 25 |

The cut and the slower jump together put her below the 45–55% target in these scripted duels. Your scoreboard had her mid-table (9,000 damage), so the live game is the judge. If she now feels weak, the cheapest knob is spike 42% → 46%, and the jump can come back to 1.0 s.

## Checks
- `cargo test --release`: 983 passed, 0 failed, 12 ignored.
- `rally_lab`: 130 passed. `contender_lab`: 49 passed. Both wasm files rebuilt (`editor/rally.wasm`, `editor/boxing.wasm`).
- `generate-rally --check`, `verify-rally.py` (with the new body-coverage check) and `generate-roster --check` pass.
- The Windows DLL reports 0.10.87.

## Needs your game
- Contender dashes in and lands punches on champions, camps and minions.
- Rally never loops at a wall, shows one body, spikes reach their target, and the 1.2 s jump feels right.
