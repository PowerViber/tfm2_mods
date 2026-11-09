# Round 158: calmer whirlpools for Thalassa (native 0.10.93, tfm2_custom 0.2.96)

You played Thalassa (round 157) and saw two problems: bumps happened too often, and the whirlpool pulled so hard that it held enemies in place and made her too strong.

## Why it happened
- Every whirlpool force-moved every enemy inside it toward its center 6 times a second. Each force-move is a real engine crowd control, so enemies were nearly permanently held and interrupted. The pull also ignored walls.
- Every surge (hers, the Tide Twin's and the trident clone's) could bump-stun every champion it caught, so stuns chained.

## What changed (rules `thalassa-1.4`)
- **Bumps:** a surge still needs 2+ champions caught, but it stuns **at most 2**, the closest to its center. Its damage still hits everyone caught.
- **Pull:** **once, on the surge**, and never through walls, terrain or cages. Whirlpools no longer keep pulling.
- **Whirlpools:** they swirl for **2 s**, then calm into a normal pool for the rest of their 10 s. Other AIs read them as a pool again.
- **Every pool slows enemies 20%**, and a swirling whirlpool 30%. An enemy gets one slow (the strongest), never stacked.

## Shipped this round
- Native **0.10.93** (Windows DLL rebuilt) and **tfm2_custom 0.2.96**, which needs 0.10.93 or later. 0.2.94 is still held by PR #69.
- The tooltips and the roster describe the new whirlpools.

## Verification
- `cargo test --release` in `native/tfm2_custom_ai`: 1075 passed, 0 failed, 0 warnings. This includes 46 Thalassa tests; 4 are new:
  - a surge stuns at most 2 champions, the closest;
  - it pulls once and never after, never onto terrain (this caught a bug where the wall check stepped back toward the whirlpool instead of the enemy);
  - a whirlpool swirls 2 s, then becomes a pool;
  - every pool slows once, and whirlpools slow more.
- `native/thalassa_lab` tests: 42 passed. `node tools/test-thalassa-lab.cjs`: 10 browser checks passed, including whirlpools calming after 2 s.
- The art verifier (1406 checks), the Mod Manager `the_repos_champions_all_load` test, `generate-thalassa.py --check` and `generate-roster.py --check` all pass.

## Not proven yet (please check in play)
- How the new whirlpools feel in a real match: whether 2 bumps per surge and a 20% pool slow are the right strength.

## How to get it
1. Merge the PR.
2. Close the game completely, run `TFM2 Mod Manager.exe`, and choose **1 Update everything**.
