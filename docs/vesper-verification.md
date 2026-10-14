# Vesper and the Rift Gauntlets: verification

Versions: native 0.10.41, tfm2_custom 0.2.41. Branch `claude/wonderful-franklin-j3os0v`, following 0.10.40 (Vesper, the Rift Marksman).

This release reworks Vesper from a ranged rift marksman into a gauntlet brawler:
- pure orbs (5, overload to 9) and Hextech hexorbs, every orb able to open any of the 100 dimensions;
- 300 distinct zone, punch and swipe behaviours;
- Shatter, both gauntlet breaks, and the evolving Hexcore with Awakening and overclock;
- tech modules;
- a full art redraw: a 3/4 street-fighter stance, broken glasses, a bronze half-cloak, and six animated pose overlays.

## How to reproduce

```
python3 "Claude outputs/vesper/generate.py" --check      # catalogue → vesper_data.rs, vesper-data.js, vesper-catalogue.md
python3 "Claude outputs/vesper/art.py"                    # sheets, body, poses, previews
python3 "Claude outputs/vesper/data.py"                   # tfm2_custom_vesper.data_champion + i18n
python3 tools/verify_vesper.py                            # views, atlases, silhouette, layering, regeneration, parity
cd native/tfm2_custom_ai && cargo test --release
VESPER_REPORT=1 cargo test --release rank_fixtures_report -- --nocapture
node tools/check_vesper_parity.js
```

## Results

**Native:** 253 passed, 0 failed, 3 ignored (two diagnostic dumps and one older test). The Vesper tests cover:
- **Orbs:** cooldowns, overload to 9, overload and enemy-hit stress, far tears.
- **Attacks:** rift punches through the chain and direct, the normal jab when no orb is ready.
- **Shatter and breaks:**
  - Shatter never damages allies (they are pushed out);
  - the left break, including the forced Shatter at 100 stress;
  - the overclock right break.
- **Combos and the Hexcore:** combo and pair bonuses; Hexcore thresholds, Awakening and fusion.
- **Modules:** hexorb modules and the `vesper_modules.txt` parser.
- **Behaviours:**
  - all **300 behaviours distinct**;
  - every dimension as rift, punch and swipe within the shared limits;
  - signature punches and swipes: the Gale wall slam, Mirror reflect, Static marks, Glass Wall, Swap Space.
- **Techniques:** kidnap, gates, step and phase, implode and domino, the rift shapes, hook, Overload Punch, Stitch, walls.
- **Determinism:** determinism across clones; identical damage at every rank with hand-placed actions.
- **Mastery:** habits starting at their rank, and the rank fixtures.
- **Poses:** the overlay priority, Shatter > swipe > rift punch > jab > block > guard.
- **Adapter:** the stable-API adapter: rank layers, HUD, effects and damage.

**Builds and checks:**

| Check | Result |
|---|---|
| Linux release build | 0 warnings |
| Windows build (`x86_64-pc-windows-gnu`) | 0 warnings |
| `mods/tfm2_custom_ai/tfm2_custom_ai.dll` SHA-256 | `6bbef6dca9b918f6468def63ec78626a9d1d7d4502dcc75c1d265edceb1466ef` (matches the build output) |
| Manager | 13 passed |
| JS/native parity | 40 math rows and 523 simulation rows, 0 mismatches. The simulation rows include 100 hand punches, 100 hand swipes, a Shatter and an overclock break |

**Art and data** (`tools/verify_vesper.py`):
- **Views:** 894 views over 26 sheets. All 720 native effect names resolve, plus 173 rank, pose and HUD layers. Removed mechanics (Breach, atlas warning, charges, strain, aiming, Borrow, Overload Shot) have no views left.
- **Poses:** six pose overlays (guard, jab, block, rift punch, swipe, Shatter) × 9 looks × 8 frames. Each pose evolves per look and keeps its motion (at least 6 distinct frames).
- **Silhouette rules:**
  - waist ≤ 13 px (the old torso was 16);
  - staggered feet, both on y 58;
  - hair, skin and both lenses visible (no hood, no mask);
  - the lead gauntlet at least 16 × 18 px.
- **Layering:** the body sheet keeps one upper-body silhouette in idle and run, and **every overlay frame of every pose and look covers it**, so the unsynced body never leaks through.
- **Mastery art:** the podium fractures stay small (26 / 20 / 15 px); #1 trails her afterimage and wears the crest.
- **Atlases:** every atlas at most 2048 px; 8 frames per animation with a fixed origin.
- Smaller effect canvases are now centred before trimming. This also fixes the bonus bursts, which used to sit off-centre.
- **Regeneration** is byte-identical. Decoded RGBA estimate: **189.5 MiB**, within the 190 MiB budget.

**Browser** (Chromium, editor server on port 7272):
- **Rift Lab:**
  - shows "RIFT EQUATIONS MATCH NATIVE";
  - 9 sockets, 6 moves and 100 dimensions;
  - Overload reaches 9 sockets;
  - a Shatter with allies near breaks the left gauntlet and **damages no ally**;
  - Awaken, then overclock, breaks the right gauntlet;
  - with "Let her plan", the arena shows the guard, jab, rift punch and Shatter poses live, with punches and hexorbs counted. Turning on the planner now unfreezes the arena; frozen units never walked into reach;
  - no horizontal overflow at 390 px.
- **Art inspector:** 902 runtime animations; the new Pose select draws every pose at every rank, including #1.
- **Mastery page modules:** a module preset saves to `vesper_modules.txt` (`3 0 2 4 5 6 7`), duplicates are refused ("Six different modules (0..7) are needed"), and clearing returns to the derived preset. Tested against a scratch game folder.
- **Remaining error:** one 404, the browser's automatic `/favicon.ico` request.

## Rank gap: 12 fixture 5v5 teamfights per rank (seeds 1–12; enemy line-up at +55% HP and +30% attack; 60 s cap)

Same seeds, same opponents, same gauntlets; only the planner's rank changes.

| Rank | Damage | Punches | Swipe hits / swipes | Useful Shatters | Left breaks | Awaken (well-timed / all) | Fused | Rifts broken | Useful Kidnaps | Wins |
|---|---|---|---|---|---|---|---|---|---|---|
| 0 Spark | 36265 | 29 | 26 / 64 | 16 / 43 | 39 | 1 / 1 | 0 | 57 | 5 / 42 | 0 / 12 |
| 1 Tear | 33762 | 20 | 37 / 27 | 15 / 34 | 29 | 0 / 1 | 0 | 44 | 1 / 39 | 0 / 12 |
| 2 Rift | 54057 | 158 | 71 / 56 | 37 / 51 | 6 | 6 / 6 | 5 | 37 | 4 / 34 | 1 / 12 |
| 3 Gate | 63744 | 130 | 134 / 63 | 41 / 42 | 2 | 8 / 8 | 11 | 16 | 4 / 18 | 3 / 12 |
| 4 Wanderer | 78344 | 170 | 213 / 94 | 57 / 57 | 0 | 10 / 12 | 16 | 22 | 1 / 13 | 7 / 12 |
| 5 Cartographer | 78609 | 162 | 192 / 92 | 55 / 56 | 1 | 13 / 13 | 15 | 25 | 17 / 24 | 7 / 12 |
| 6 Riftwalker | 81508 | 174 | 194 / 90 | 69 / 72 | 3 | 15 / 15 | 24 | 22 | 26 / 39 | 11 / 12 |
| 7 Worldbreaker | 88005 | 171 | 170 / 59 | 61 / 61 | 0 | 17 / 17 | 15 | 26 | 31 / 42 | 12 / 12 |

The right gauntlet broke 0 times at every rank. Overclock is rare, and the habit stops it in time.

What this shows:
- **Damage:** about **2.4×** Spark → Worldbreaker, the same 35%-per-2-s window applying to both.
- **Gauntlet discipline:** left breaks 39 → 0, useful Shatters 37% → 100%, swipe hits per swipe 0.4 → 2.9.
- **Team play:** useful Kidnaps 12% → 74%; Awakenings all timed with the engage from rank 2.
- **Wins:** 0/12 → 12/12 against the strengthened fixture line-up. With the old line-up every rank won every fight, so the fixture was tuned.

The fixture test asserts:
- Worldbreaker damage ≥ 1.8× Spark;
- at most a fifth of Spark's left breaks, with Spark breaking at least 12 times;
- Worldbreaker wins ≥ Spark + 8;
- a Spark still uses at least 20 techniques and 4 gauntlet moves.

## To check in game

- In the Manager, use Update/Check and confirm 0.10.41 / 0.2.41.
- Watch a Spark and a Worldbreaker Vesper. Check:
  - that the pose overlays (jab on attacks, block on hits, rift punch, swipe, Shatter) switch at the right moments;
  - that the HUD stack (orbs, hexorbs, Hexcore, stress) reads at game zoom.
- Check FPS in a 5v5 with Vesper. She is capped at 5 rifts, 16 packets and 6 effect starts per update.
- If the game's own AI breaks the left gauntlet far more or less often than the fixtures, tune `hit_stress_pct`, `over_stress` or `shatter_load` in `catalogue.json` and regenerate.
