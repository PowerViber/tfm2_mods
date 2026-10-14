# Vesper and the Rift Gauntlets: verification

Versions: native 0.10.50, tfm2_custom 0.2.51. Branch `claude/vesper-scifi-variety`.

0.10.50 completes the four releases: 20 families and 200 dimensions.
- **Resonance, Polarity, Circuit, Oath, Genesis:**
  - 10 dimensions each, one signature each;
  - 150 new distinct behaviours (600 in all);
  - 90 new twists (210 in all).
- **Five new primitives**, in the core, the JS port, the planner's valuation and the cooldown classes:
  - `RESONATE`: the next punch or swipe inside a 1.5–3.5 s window deals a share again;
  - `POLE`: two polarised enemies within range snap together and both take damage;
  - `CIRCUIT`: modules relay to the nearest other live rift with a bigger radius; punches and swipes arc into the nearest rift;
  - `OATH`: a shield that strikes the nearest enemy back on the first hit, only near where it was sworn;
  - `GENESIS`: seed, bloom and storm over a third of the rift's life each, or half a second apart on a punch.
- **Planner:**
  - POLE is valued low with no partner polarised, and CIRCUIT with no other rift holding an enemy;
  - rank 7 now looks ahead 84 ticks and weighs 24 techniques a think, like rank 6. Over-leading targets cost her about 10 wins in 192 fights at the new reach.
- **Fixtures:** 192 seeds by default. At 200 dimensions the Worldbreaker's margin over the win bar is a few fights in a hundred, so 48 or 96 seeds pass or fail on luck.
- **Art:**
  - new glyph frames (a wave ribbon, a bow-tie, a cross, a crown, a leaf) and core colours;
  - the second glyph sheet holds dimensions 100–199;
  - 177.9 MiB.

0.10.49 is the third of the four releases: five new families, 150 dimensions.
- **Counts generalised.** Nothing assumes 10 families or 100 dimensions any more:
  - the generator emits `N_DIMS`, `N_FAMILIES`, `N_TWISTS` and the per-family index ranges;
  - Dice and Kaleidoscope draw from those;
  - `twist_index` takes the family count;
  - the tests, vector loops, art, verifier, Rift Lab and art preview read them.
  - With the old 100 dimensions this step was proven neutral: the simulation vectors stayed byte-identical.
- **Gravity, Frost, Momentum, Pressure, Radiance:** 10 dimensions each (6 straightforward, 3 tactical, 1 signature), 150 distinct new behaviours, at the 0.10.48 scale.
  - **Six new primitives:**
    - `WELL` and `REPEL`: pull or push on every pulse;
    - `CHILL`: stacks to a freeze, fading 1 s after the last chill;
    - `MOMENTUM`: damage for distance moved, bounded;
    - `PRESSURE`: charges that burst on close, implode or break, or a delayed burst on a punch;
    - `EXPOSE`: a mark the next punch, swipe or implode detonates.
  - Each is in the core, the JS port, the planner's valuation and the cooldown classes.
  - **65 new twists**, one per new family pair, their kind set by the two aspects (120 in all).
- **Art:** each new family has its own glyph frame and core colour. The 50 new glyphs have their own sheet (first use stays bounded).
- **Rift Lab:** the dimension selects are grouped by family, the twist grid is 15×15, and the art preview shows any family's glyphs.
- **Planner:** CHILL is valued as a slow until it freezes, and MOMENTUM as nothing to a planted Vesper (it was over-picked by the aiming top ranks).

0.10.48 is the second of the four releases: more casts, less per cast. The full tables are in the guide's "More casts, less per cast" section.
- **Cheaper and faster orbs:**
  - tear strain 8, long link 5, overload 8 and +1 per 30 ticks, technique strain −25%, 15 corruption an overclock;
  - pure orbs recover in 4 s, overload orbs 6 s, hexorbs 2.5 / 2.25 s.
- **Less per cast**, already scaled in the catalogue by a one-time migration (`catalogue.json` `balance`):
  - damage −20% everywhere, combo +8%/step up to 32%;
  - damage allowance 30% per 2 s;
  - hard control −20%, control allowance 45 ticks per 3 s;
  - heals and shields −20%, and a new heal allowance of 20% max HP per 3 s per target;
  - radii −15%;
  - immunity and Undying −25% with a 25 s cooldown, Ghost 10 s, self-invisibility 4 s.
  - The literals that bypassed the catalogue (Rain, Volley, the search radii, technique banishes and stuns, the Tithe shield, Wild Echo) are now catalogue constants.
- **Shorter reach:** Kidnap 90000, direct punch 56000, swipe 50000, Step/Phase 18000. `land()` caps every teleport at 90000.
- **Cooldowns:**
  - every dimension rests 10, 14, 20, 24 or 36 s, classed by the generator from its rift's main effect (one spectacular dimension per family takes 36 s);
  - its family rests 2 s;
  - both gauntlets share the timers;
  - one action's rifts and a fused punch's two orbs lock together.
  - With every dimension resting she jabs.
  - The Rift Lab greys out resting dimensions with their timers, refuses resting casts with the reason, and has an **Ignore cooldowns** box.
- **Planner retuned to the new numbers:**
  - pulls land just in front of her frontliner, or as far toward it as 90000 reaches;
  - Gate only syncs when the exit is in reach, and Fling and Swap respect the cap;
  - Kidnap, Swap and Hook lean less on the target's priority;
  - the spine overclocks at most three times and overloads at 18, not 24;
  - the spine saves the swipe for lines;
  - the Worldbreaker's strain limit is 162, like ranks 2–6.
- **Fixtures:**
  - the line-up is re-centred to +45% HP / +20% attack so a mid-rank fight is even again;
  - the sample is 48 seeds;
  - the damage bar compares rift damage (the Spark's jab is not part of the kit and was not scaled).

0.10.47 removes the activation hitches and the planner's per-pick cost. It is the first of four releases (more casts, less per cast; 20 families and 200 dimensions). Nothing about how she fights changes: the simulation vectors are byte-identical to 0.10.46 and so are the fixtures.
- **No sheet an effect uses decodes more than 4.5 MiB the first time it plays** (was up to 7.6 MiB). The verifier now checks this.
  - **Gauntlet art** (5.1 MiB) is split per tier, plus a 0.5 MiB common sheet.
  - **Modules** (3.9 MiB) are split per tier, about 0.9 MiB each.
  - **The misc sheet** (4.75 MiB) is gone. Her jab's bolt now has its own 0.02 MiB sheet; the old sheet decoded on her first jab. The twists get a 1.9 MiB sheet, and Shatter one per tier (0.4 MiB).
  - **Technique flourishes** are split into two halves per tier: 3.4 MiB each instead of 7.3.
  - **Rifts** are split per look into the common kinds (tear, twin, anchor, wander, slash) and the rest: 3.8–4.1 MiB each instead of 7.6–8.3.
- **Packing:** atlases search their width for the smallest area, so the total decoded drops from 189.4 to **174.3 MiB**, even with 28 more sheets.
- **The planner reads the fight once per pick.** It used to scan every fighter twice for each of the 100 dimensions.
  - Each dimension still draws its noise in order, so the random stream is unchanged.
  - A dimension whose upper bound plus noise can't beat the best so far is skipped without exact scoring.
  - The result is exactly the old full scan's.
  - A pick takes 1–5 µs, down from 17.5 µs on average.
  - An `available_dims` hook is ready for Release B's cooldowns.
- **Manager:** `STALE_FILES` deletes the 17 replaced sheets (34 files). `TFM2 Mod Manager.exe` is rebuilt.

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

**Native:** 272 passed, 0 failed, 3 ignored (two diagnostic dumps and one older test). The Vesper tests cover:
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
- **0.10.50:** resonance pays the next hit inside its window and nothing after it; polarity needs two marks in range; circuit relays a module and arcs a punch; an oath strikes back once and ends out of range; genesis grows seed, bloom, storm in order; every dimension's three behaviours distinct (600).
- **0.10.49:** gravity wells draw in and repulsors push out; frost stacks freeze and fade when the target steps out; pressure caps its charges and bursts on close; momentum is zero standing still and bounded; an exposed target detonates on the next punch; every dimension's three behaviours distinct (450).
- **0.10.48:** dimension cooldowns and family recovery (refusals, one cast's rifts locking together, the classes, death clears them); a resting dimension leaves her the plain jab; the heal allowance; no teleport past 90000.
- **Adapter:** the stable-API adapter: rank layers, HUD, effects and damage.

**Builds and checks:**

| Check | Result |
|---|---|
| Linux release build | 0 warnings |
| Windows build (`x86_64-pc-windows-gnu`) | 0 warnings |
| `mods/tfm2_custom_ai/tfm2_custom_ai.dll` SHA-256 | `984b4e4d45781ac3f16fc556e3df3c2b3091124141979bd12381e2aecfa2c245` (matches the build output) |
| Manager | 13 passed. Rebuilt `TFM2 Mod Manager.exe` (SHA-256 `9b0e2286…bd35`): it deletes Vesper's replaced sheets from the game folder (84 stale files in all) |
| JS/native parity | 40 math rows and 954 simulation rows, 0 mismatches (every one of the 200 dimensions punched and swiped by hand). The rows cover the cooldown refusals, the heal allowance and the teleport cap through the planner teamfights and the hand-placed punches and swipes. The rows include full planner teamfights at ranks 0, 3, 5 and 7, effects started and Kidnap dodges |
| Unified Theory verifier | passes, with every `_l` body layer an exact mirror; decoded estimate 196.3 MiB (the mirrors add 6.7 MiB) | The simulation rows include 100 hand punches, 100 hand swipes, a Shatter and an overclock break |

**Art and data** (`tools/verify_vesper.py`):
- **Views:** 1124 views over 95 sheets. All 879 native effect names resolve, plus 244 pose, emblem, scar and HUD layers. Every `_l` pose and back scar is the exact mirror of its right-facing copy. No old aura, Breach, atlas, charge or strain views are left.
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
- **Regeneration** is byte-identical. Decoded RGBA: **177.9 MiB** (budget 190), with tallest-first shelf packing at the width that gives the smallest atlas.
- **First use:** the largest sheet any effect plays from decodes **4.1 MiB** (limit 4.5). Only the body overlays, decoded when she spawns, are larger.

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

## Rank gap: 192 fixture 5v5 teamfights per rank (seeds 1–192; enemy line-up at +45% HP and +20% attack; 60 s cap)

Same seeds, same opponents, same gauntlets; only the planner's rank changes. The enemies never walk into her rifts and walk out of armed Kidnap fields.

| Rank | Damage | Rift damage | Punches | Swipe hits / swipes | Useful Shatters | Left breaks | Awaken (well-timed / all) | Rifts broken | Useful pulls | Wins | Top technique | Effects/s (peak) | Dimensions a fight |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 0 Spark | 611973 | 331202 | 868 | 572 / 1390 | 309 / 831 | 529 | 74 / 78 | 971 | 65 / 594 | 11 / 192 | 9% | 15 (41) | 35 |
| 1 Tear | 725512 | 449859 | 760 | 1463 / 1200 | 347 / 811 | 533 | 79 / 83 | 1314 | 84 / 579 | 36 / 192 | 12% | 16 (42) | 39 |
| 2 Rift | 1005849 | 858763 | 3513 | 1893 / 1501 | 818 / 1096 | 39 | 240 / 247 | 540 | 53 / 406 | 98 / 192 | 19% | 15 (41) | 48 |
| 3 Gate | 1157762 | 954992 | 3104 | 2633 / 1472 | 1040 / 1063 | 28 | 229 / 237 | 423 | 58 / 345 | 124 / 192 | 19% | 16 (41) | 49 |
| 4 Wanderer | 1246765 | 1008240 | 3266 | 2594 / 1367 | 1125 / 1126 | 1 | 240 / 262 | 327 | 33 / 230 | 141 / 192 | 20% | 17 (42) | 51 |
| 5 Cartographer | 1146840 | 868810 | 3112 | 2412 / 1253 | 1178 / 1184 | 8 | 273 / 273 | 441 | 290 / 522 | 151 / 192 | 18% | 18 (42) | 52 |
| 6 Riftwalker | 1146147 | 875284 | 3197 | 2129 / 1074 | 1284 / 1289 | 5 | 288 / 288 | 319 | 362 / 555 | 147 / 192 | 18% | 18 (42) | 53 |
| 7 Worldbreaker | 1137091 | 865020 | 3226 | 2088 / 875 | 1371 / 1376 | 5 | 316 / 316 | 241 | 368 / 568 | 145 / 192 | 18% | 18 (42) | 48 |

"Useful pulls" counts every technique that lands an enemy somewhere (Hook, Sinkhole, Seed, Swap, Fling and the Kidnap blink): useful when it lands among her team.

Why the sample is 192 fights: the Worldbreaker clears the win bar by about 6 fights in 192. At 48 or 96 seeds a fixed seed set can land either side of it; the planner retunes were judged at 192.

With every payoff a fifth lower the line-up came down from +55%/+30%; at the old line-up every rank lost most fights and the ladder could not be read. The ranks that play for the team (from Cartographer) win the most; the Worldbreaker deals a little less than the middle ranks because she spends orbs on pulls, gates and her Hexcore, and wins three fights in four.

The right gauntlet broke 0 times at every rank. Overclock is rare, and the habit stops it in time.

What this shows:
- **Damage:** rift damage 2.6× Spark → Worldbreaker (331202 → 865020). Total damage is only 1.9×, because most of a Spark's damage is the plain jab.
- **Gauntlet discipline:** left breaks 529 → 5, useful Shatters 37% → 100%, swipe hits per swipe 0.41 → 2.4.
- **Team play:** pulls landing in her team 11% → 65%; Awakenings all timed with the engage from rank 5.
- **Variety:** 35 to 53 different dimensions a fight, out of 200.
- **Wins:** 6% → 76% (11 → 145 of 192).
- **Open issue:** the Worldbreaker no longer wins the most. Cartographer (151) and Riftwalker (147) edge her out: she spends more on pulls and the Hexcore, and swipes less (875 against 1253). Worth a planner pass of its own.

The fixture test asserts:
- Worldbreaker rift damage ≥ 2× Spark's;
- at most a fifth of Spark's left breaks, with Spark breaking at least 12 times;
- Worldbreaker wins ≥ Spark + 8 per 12 fights (192 seeds by default; `VESPER_SEEDS` changes it, `VS_FOE` recalibrates the line-up);
- Worldbreaker swipes hit at least 2 per swipe and 3× a Spark's rate;
- at least half of a Worldbreaker's pulls land in her team, at most 30% of a Spark's;
- the effect budget holds at every rank;
- a Spark still uses at least 20 techniques and 4 gauntlet moves;
- no technique takes more than 20% of her plays at Cartographer and above, or 25% below;
- a Worldbreaker opens at least 8 different dimensions a fight;
- a dimension pick takes at most 250 µs at every rank (it takes 1–5).

A Spark picks at random, so it has no pick time.

## To check in game

- In the Manager, use Update/Check and confirm 0.10.50 / 0.2.51.
- **Facing:** walk Vesper and the Unified Theory left and right. The overlay should turn with the base sprite. If it is the wrong way round compared to the legs, the game's mirror rule differs from DIO's; tell me which way it was.
- **Kidnap:** the blink, the cracked ring, a caught enemy dropping with her, and a dodge leaving her where she blinked.
- **Rift danger:** enemies stop at the edge of her rifts and walk out of Kidnap rings.
- **Lag:** with several rifts opening and closing. The Manager's Performance log (menu 7) shows the effects per second and the longest gap between ticks.
- Watch a Spark and a Worldbreaker Vesper. Check:
  - that the pose overlays (jab on attacks, block on hits, rift punch, swipe, Shatter) switch at the right moments;
  - that the HUD stack (orbs, hexorbs, Hexcore, stress) reads at game zoom.
- Check FPS in a 5v5 with Vesper. She is capped at 5 rifts and 16 punches in flight, and her effects come from a 12-token bucket (one back every 2 ticks, at most 6 per update).
- If the game's own AI breaks the left gauntlet far more or less often than the fixtures, tune `hit_stress_pct`, `over_stress` or `shatter_load` in `catalogue.json` and regenerate.
