# Round 159: Wright joins the game (native 0.10.94, tfm2_custom 0.2.97)

You wanted a summoner who launches planes like a carrier spawning aircraft, designed as an engineer who builds them. Over two Lab reviews you shaped his jets, his bomber and his carrier, and Codex drew him and his fleet. This round puts him in the game.

## Wright, the aircraft engineer
He is an Original-family summoner mage for mid who can jungle, with magic damage, no mastery and Mod Power `[0, 25, 5, 5, 10, 0, 10]`. He keeps Flash. The full kit is in `docs/wright-guide.md`; every number lives in `rules/wright.json` (revision `wright-4`).

| | Skill | In short |
|---|---|---|
| Passive | Workshop | An airframe every 4 s (up to 3); each skill uses one. Every 14 s a scout drone (a real unit, 15% of his HP) flies to the nearest enemy and taunts enemies near it once. |
| S1 | Fighter Wing | 2 jets for 20 s (up to 4), real units with 15% of his HP. They **fly sorties from him**: commit to where the nearest enemy is (champions, then minions and creeps), strike once (every 3rd sortie a rocket) and fly back. No homing, and **the closer he is, the faster they attack**. They kamikaze (knockback, burn) at the end or when nearly shot down. |
| S2 | Carpet Bomber | Three big, overlapping explosions advance along a committed line after a ready-up warning; the first ones slow and the last stuns. |
| S3 | Repair Wing | A repair chopper around him and a repair plane to the most hurt ally anywhere: heal and shield kits. |
| Ult | Sky Carrier | Drops on him (no damage, a huge knockback, the closer the farther), follows him for 10 s taking 80% of his damage from its pool, launches jets and choppers, and readies bombers his S2 uses with no airframe or cooldown. |

His summons clear minions and creeps like normal, **never damage turrets**, and turrets can shoot his jets and drone.

## Your changes along the way
- **First Lab review:** the jets became dogfighters that stay targetable. S2 became Nana-style (a ready-up, then waves; the last stuns). The carrier drops on him, follows him and guards him, and its bombers stack up ready for S2. The "jets hit friendlies" report didn't reproduce: the ally took 0 damage, and the Lab now marks every hit.
- **This round:** the jets **return to him after every hit** and attack enemy champions, creeps and minions; **the closer he is, the faster they attack**; **no lock-on** (you found that too strong).
  - Each sortie commits to the spot where its target stood, so a moving target can dodge. Someone else on the spot takes the hit instead.
  - The lock-on rocket and its reticle are gone; every 3rd sortie is an unguided rocket.
- **Codex:** the approved engineer, the fleet in 8 headings and the full animation set; then bigger areas, the bomber as three advancing warned explosions, and dogfight art variants (still used for the outbound and return legs).

## Shipped this round
- Native **0.10.94** (DLL rebuilt for Windows; it reports 0.10.94) and **tfm2_custom 0.2.97**, which needs 0.10.94 or later. 0.2.94 is still held by PR #69.
- The README row and the pack description.

## Verification
- `cargo test --release` in `native/tfm2_custom_ai`: 1109 passed, 0 failed, 0 warnings, Codex's tests included. The sortie tests cover:
  - one strike per sortie, then home;
  - more strikes on a close target than a far one;
  - a target that moves away is missed, while someone on the spot is hit;
  - every 3rd sortie is a rocket;
  - no lock reticle;
  - minions and creeps but never turrets.
- `native/wright_lab`: 25 passed. `node tools/test-wright-lab.cjs`: 12 browser checks passed.
- Codex's art verifier: every check up to its rebuild step passes, including the exact rules for `wright-4`. Its byte-for-byte rebuild needs a font from Codex's machine.
- Mod Manager `the_repos_champions_all_load`, `generate-wright.py --check` and `generate-roster.py --check`: pass.

## Not proven yet (please check in play)
- **Live acceptance:** none of this has been seen in a real match yet.
- **Sortie pacing:** the jets' pacing at real distances, and whether 20 + 25% AP per strike is right.
- **Guard pool:** the carrier's guard assumes `on_damaged` reports damage after the 80% reduction. Watch whether the pool drains too fast.
- **Turrets:** that turrets target the jets and drone (the game decides turret targeting).
- **Balance.**

## How to get him
1. Merge PR #72.
2. Close the game completely, run `TFM2 Mod Manager.exe`, and choose **1 Update everything**.

Wright appears in exhibition matches and **new careers**. Existing careers can't add new champions.
