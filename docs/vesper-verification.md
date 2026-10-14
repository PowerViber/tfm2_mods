# Vesper and the Rift Gauntlets: verification

Versions: native 0.10.46, tfm2_custom 0.2.47. Branch `claude/vesper-scifi-variety`.

0.10.46 fixes two bugs found by the tests in Volume 2 of the champion guide (see `champion-guide-addendum.md`):
- **Rewind:** a full-strength Rewind punch looked 180 ticks back while the core kept only 120 ticks of history, so it silently did nothing. The history is now 180 ticks. A teleport now only zeroes the forecast velocity, so Rewind can still send a target to before its blink. New test: `rewind_punch_at_full_strength_moves_the_target`.
- **Plating:** the text said "nearest ally"; the module shields the lowest-health ally near the rift. The text now says so.

0.10.45:
- **She turns around.** The game mirrors only a champion's own body sprite, never buff overlays, and Vesper's and the Unified Theory's bodies are overlays. Both now ship mirrored `_l` copies of every body layer and pick them with a shared facing rule (`facing.rs`, DIO's rule: the nearest enemy within 90000, else the walk; standing still keeps the facing).
- **Lag.**
  - Effects come from a token bucket (12, one back every 2 ticks, ambient visuals only above a reserve of 4).
  - The look-dependent sheets are split per look or tier, so a match decodes only its own art.
  - The adapter no longer clones her whole state every tick, and the planner gathers rifts once per think.
- **Kidnap**:
  - she blinks in front of the target and arms a 0.5 s stun field;
  - caught: the target is stunned and both drop through to the exit;
  - dodged: nothing happens and she stays where she blinked;
  - 20 s cooldown.
- **Punches, not arrows.** Chrome fists travel through the rifts and burst out of the last one. The marksman-era muzzles and shot lines are gone.
- **Rifts are dangerous to the game's AI.** Rifts and Kidnap fields are published as `vsr:`/`vsk:` buffs. The shared input AI keeps her enemies from walking into them and walks them out of armed fields (moves only). Minato and the other mod AIs read them too.

0.10.44:
- **Stress cap 100 → 180.** The thresholds scale with it: unstable rifts are avoided past 3/5, Overload Punch from 1/4 and pushed past 7/10, vent at 3/4, the break resets to 2/5 (72). She now overloads before fights at every rank from Rift up.
- **Variety.** Repeats lose score (8 per repeat in the last 10 s, +10 if it was the last technique, capped at half the play's worth). At most 4 uses of a technique per 10 s, combo steps included. Gauntlet moves are exempt.
- **White sci-fi outfit.** A white chest plate with grey seams and a cyan light, a dark bodysuit with cyan piping, and white boots with dark soles and cyan running lights.
- The editor's stress bars scale to the cap.

0.2.44 swapped her glasses for a broken dimensional visor and brow plate (art only).

0.10.43 builds on 0.10.42's 3/4 boxing guard and "reality is glass" rifts. She now has chestnut hair, framed glasses with a cracked lens, a hardened face, a gunmetal cloak and mirror-chrome gauntlets. Rank shows as how broken she is, plus an animated rank emblem at her top right (Isliid/Scribble style, 16 frames).

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

**Native:** 258 passed, 0 failed, 3 ignored (two diagnostic dumps and one older test). The Vesper tests cover:
- **Orbs:** cooldowns, overload to 9, overload and enemy-hit stress, far tears.
- **Attacks:** rift punches through the chain and direct, the normal jab when no orb is ready.
- **Shatter and breaks:**
  - Shatter never damages allies (they are pushed out);
  - the left break, including the forced Shatter at the 180 stress cap;
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
- **0.10.45:**
  - Kidnap caught (stun, both drop through), dodged (no stun, she stays) and immune;
  - rifts published, read back, and steering enemies but not allies;
  - the facing rule for Vesper and the Unified Theory;
  - the effect budget in the fixtures: at most one bucket plus refill in any second, at most 30/s on average.
- **Poses:** the overlay priority, Shatter > swipe > rift punch > jab > block > guard; the pose looks (#1 → 8, #2 → 10, #3 → 9) and the rank tags (no tag below Top 10, the wound from Riftwalker, the massive tear for #1).
- **Adapter:** the stable-API adapter: rank layers, HUD, effects and damage.

**Builds and checks:**

| Check | Result |
|---|---|
| Linux release build | 0 warnings |
| Windows build (`x86_64-pc-windows-gnu`) | 0 warnings |
| `mods/tfm2_custom_ai/tfm2_custom_ai.dll` SHA-256 | `a01563d14a66e598d1271dd22ebecd3aab9ad04fab007f309b3ee54945e62451` (matches the build output) |
| Manager | 13 passed. Rebuilt `TFM2 Mod Manager.exe` (SHA-256 `489c2d1f…d204f`): it deletes Vesper's old combined sheets from the game folder |
| JS/native parity | 40 math rows and 674 simulation rows, 0 mismatches. The rows include full planner teamfights at ranks 0, 3, 5 and 7, effects started and Kidnap dodges |
| Unified Theory verifier | passes, with every `_l` body layer an exact mirror; decoded estimate 196.3 MiB (the mirrors add 6.7 MiB) | The simulation rows include 100 hand punches, 100 hand swipes, a Shatter and an overclock break |

**Art and data** (`tools/verify_vesper.py`):
- **Views:** 984 views over 66 sheets. All 739 native effect names resolve, plus 244 pose, emblem, scar and HUD layers. Every `_l` pose and back scar is the exact mirror of its right-facing copy. No old aura, Breach, atlas, charge or strain views are left.
- **Poses:** six pose overlays × 11 pose looks × 8 frames. Every look's loop differs, and every pose keeps its motion.
- **Emblems:** 17 (ranks 0–6, Top 10 #1–#10), 16 frames each, each at least 12 distinct frames. They sit at her top right, below her chest line, and never cover her head with a visible pixel.
- **Look:**
  - the gauntlet palette is chrome (no copper or bronze);
  - the hair is brown, not near-black;
  - the visor wraps her eyes (at least 8 visor pixels across), carries at least 2 white crack pixels, and stays dark glass;
  - the brow plate is present and the lower face is bare;
  - the chest plate and boots are white, the bodysuit dark; the plate shows in the guard; the boots have a dark sole and a cyan light;
  - the cloak starts at least 6 px wide at the shoulder and ends above the rear elbow;
  - the damage colours (scar and chromatic fringe) grow strictly from Spark through Gate, Riftwalker and Worldbreaker to #3.
- **Anatomy:** stance width, bent front knee, lifted back heel, waist, visible elbows, knuckles toward the opponent, frames rather than a visor.
- **Layering:** every overlay frame covers the body sheet.
- **Rifts:** the scar palette is present; no atlas grid.
- **Atlases:** at most 2048 px; 8 frames per animation (16 for the emblems); fixed origins.
- **Regeneration** is byte-identical. Decoded RGBA: **189.4 MiB** (budget 190), with the mirrored poses paid for by tallest-first shelf packing, a shorter link thread and per-look sheets.

**Browser** (Chromium, editor server on port 7272):
- **Rift Lab:**
  - shows "RIFT EQUATIONS MATCH NATIVE";
  - 9 sockets, 6 moves and 100 dimensions;
  - Overload reaches 9 sockets;
  - a Shatter with allies near breaks the left gauntlet and **damages no ally**;
  - Awaken, then overclock, breaks the right gauntlet;
  - with "Let her plan", the arena shows the guard, jab, rift punch and Shatter poses live, with punches and hexorbs counted. Turning on the planner now unfreezes the arena; frozen units never walked into reach;
  - no horizontal overflow at 390 px.
- **Art inspector:** 905 runtime animations; every pose draws at Spark, Riftwalker, #3 and #1 with the emblem and no layer missing. The arena shows the emblem, the scars and the HUD together.
- **Mastery page modules:** a module preset saves to `vesper_modules.txt` (`3 0 2 4 5 6 7`), duplicates are refused ("Six different modules (0..7) are needed"), and clearing returns to the derived preset. Tested against a scratch game folder.
- **Remaining error:** one 404, the browser's automatic `/favicon.ico` request.

## Rank gap: 24 fixture 5v5 teamfights per rank (seeds 1–24; enemy line-up at +55% HP and +30% attack; 60 s cap)

Same seeds, same opponents, same gauntlets; only the planner's rank changes. The enemies follow the game's new rule: they never walk into her rifts and they walk out of armed Kidnap fields.

| Rank | Damage | Punches | Swipe hits / swipes | Useful Shatters | Left breaks | Awaken (well-timed / all) | Rifts broken | Useful pulls | Kidnap dodges | Wins | Top technique | Effects/s (peak) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 0 Spark | 66660 | 83 | 54 / 120 | 36 / 86 | 61 | 3 / 3 | 97 | 9 / 83 | 3 | 0 / 24 | 10% | 13 (39) |
| 1 Tear | 82265 | 61 | 143 / 108 | 39 / 84 | 64 | 5 / 6 | 123 | 6 / 74 | 3 | 2 / 24 | 16% | 14 (41) |
| 2 Rift | 117496 | 388 | 205 / 146 | 88 / 109 | 3 | 22 / 23 | 55 | 7 / 62 | 3 | 5 / 24 | 24% | 12 (40) |
| 3 Gate | 148820 | 370 | 358 / 168 | 118 / 120 | 2 | 21 / 24 | 57 | 7 / 35 | 3 | 11 / 24 | 24% | 13 (39) |
| 4 Wanderer | 136699 | 350 | 329 / 146 | 122 / 123 | 1 | 21 / 22 | 47 | 3 / 24 | 2 | 5 / 24 | 25% | 12 (41) |
| 5 Cartographer | 149264 | 208 | 563 / 214 | 148 / 148 | 0 | 28 / 28 | 49 | 23 / 44 | 4 | 12 / 24 | 19% | 14 (41) |
| 6 Riftwalker | 155815 | 244 | 459 / 202 | 156 / 156 | 0 | 34 / 34 | 53 | 27 / 61 | 3 | 13 / 24 | 17% | 16 (41) |
| 7 Worldbreaker | 152717 | 223 | 460 / 168 | 165 / 165 | 0 | 31 / 31 | 22 | 37 / 71 | 3 | 16 / 24 | 14% | 15 (41) |

"Useful pulls" counts every technique that lands an enemy somewhere (Hook, Sinkhole, Seed, Swap, Fling and the Kidnap blink): useful when it lands among her team.

Why the sample is 24 fights, not 12: with Kidnap rarer, a few fights flipped on a single pull and moved the 12-seed win count by two. Over 36 seeds, the new Kidnap changed Worldbreaker's wins by one (25 vs 26 of 36) and cost about 6% of her damage. That is the price of her following the target to the exit.

Stepping around rifts makes every rank's fights harder. The mid ranks now win about half; the Worldbreaker two thirds; the Spark none.

The right gauntlet broke 0 times at every rank. Overclock is rare, and the habit stops it in time.

What this shows:
- **Damage:** about **2.3×** Spark → Worldbreaker, the same 35%-per-2-s window applying to both.
- **Gauntlet discipline:** left breaks 61 → 0, useful Shatters 42% → 100%, swipe hits per swipe 0.45 → 2.7.
- **Team play:** pulls landing in her team 11% → 52%; Awakenings all timed with the engage from rank 2.
- **Wins:** 0/24 → 16/24 against the strengthened fixture line-up.

The fixture test asserts:
- Worldbreaker damage ≥ 1.8× Spark;
- at most a fifth of Spark's left breaks, with Spark breaking at least 12 times;
- Worldbreaker wins ≥ Spark + 8 per 12 fights;
- Worldbreaker swipes hit at least 2 per swipe and 3× a Spark's rate;
- at least half of a Worldbreaker's pulls land in her team, at most 30% of a Spark's;
- the effect budget holds at every rank;
- a Spark still uses at least 20 techniques and 4 gauntlet moves;
- no technique takes more than 20% of her plays at Cartographer and above, or 25% below.

## To check in game

- In the Manager, use Update/Check and confirm 0.10.46 / 0.2.47.
- **Facing:** walk Vesper and the Unified Theory left and right. The overlay should turn with the base sprite. If it is the wrong way round compared to the legs, the game's mirror rule differs from DIO's; tell me which way it was.
- **Kidnap:** the blink, the cracked ring, a caught enemy dropping with her, and a dodge leaving her where she blinked.
- **Rift danger:** enemies stop at the edge of her rifts and walk out of Kidnap rings.
- **Lag:** with several rifts opening and closing. The Manager's Performance log (menu 7) shows the effects per second and the longest gap between ticks.
- Watch a Spark and a Worldbreaker Vesper. Check:
  - that the pose overlays (jab on attacks, block on hits, rift punch, swipe, Shatter) switch at the right moments;
  - that the HUD stack (orbs, hexorbs, Hexcore, stress) reads at game zoom.
- Check FPS in a 5v5 with Vesper. She is capped at 5 rifts and 16 punches in flight, and her effects come from a 12-token bucket (one back every 2 ticks, at most 6 per update).
- If the game's own AI breaks the left gauntlet far more or less often than the fixtures, tune `hit_stress_pct`, `over_stress` or `shatter_load` in `catalogue.json` and regenerate.
