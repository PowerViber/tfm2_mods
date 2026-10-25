# Round 143: no more back-and-forth, and Raze fixed and strengthened

Native 0.10.73, champion pack 0.2.73, Valorant 0.1.22.

## What you reported

- Every player still walks back and forth.
- Raze's animations don't match the editor, and the cluster animation is duplicated.
- Raze paths strangely and never uses her ultimate in a crowd.
- Raze is very weak and dies early. She should have Omen's buy passive, and her strafe should be lowered by 25°.

## Back-and-forth: the movement layers ship switched off

Before PR #42 the game pathed every champion itself, and nobody went back and forth. Two rounds of tuning the movement layers added since then didn't remove it. Those layers are:

- athlete moves;
- team guidance;
- team strategy writes;
- staging.

So they now ship **off**. That applies both in `mods/tfm2_custom_ai/settings.json` and in the built-in defaults when the file or a field is missing. Staging now follows `team_guidance`; before, it ignored the switches. Every champion walks the game's own path again.

**Unchanged:**

- The stats still shape the kits that decide for themselves: Gojo, Omen, DIO and Raze's planner.
- The team planner still makes plans.
- The older redirects stay on: Steve walls and detours, zone avoidance, Omen smoke, rallies and Map-tab plans.

**Diagnostics.** If you create an `ai_diagnostics.enabled` file, a `pathing_redirects` line is logged per player every 600 ticks. It shows the share of moves each redirect source changed, and how often the player flipped between the engine's path and a redirect. If anyone still goes back and forth, one match of that log names the cause.

## Raze

| Problem | Cause | Fix |
|---|---|---|
| Duplicate clusters and stacked effects | The game always plays a view effect's full animation (0.4 s), but grenades, bomblets, rockets, satchels, Boom Bot and her airborne body were re-sent every tick, so up to 24 copies stacked | Re-sent effects use single-frame `_p0`–`_p7` phases (1/60 s, existing frames only), as the rocket marker already did. One sprite per object, as in the Lab. |
| Animations differ from the editor | Different pose choices, effect lifetimes and body anchor | Rocket-armed and Boom Bot poses match the Lab. The Lab's effect lifetimes match the game's 24-tick plays. The Lab draws the body at its in-game height. |
| Never ults in crowds | Her planner's situation fingerprint changed every few ticks (bullet ids, CC), so she never decided. The crowd penalty punished the rocket and rewarded running. | Decisions are forced at least every reaction + 12 ticks. The rocket gets +25 per champion in its blast and +60 to fire once armed, and no penalty for the danger she stands in. The cooldown starts when it fires, so an interrupted equip costs nothing. A lost target is re-picked. |
| Strange pathing | Her movement guide expired after 18 ticks. She launched away from where the engine walked her. Guides near walls were dropped. | The guide lasts 60 ticks and is re-sent every tick. Launches more than 90° from the engine's destination are blocked, except escapes. Guides near walls are pulled back to free ground. |
| Strafe | Full-circle air turns | The turn is capped at 155° from her flight heading (180° − 25°). |
| Very weak, dies early | No Mod Power, armour about half a stock ranged champion's, entries spent her only escape, retreat at 15% HP, utility never refilled | Mod Power (+20% HP, +30% AP, +10% armour and MR). Armour 25 (+6 per level), MR growth 4, HP growth 90, attack growth 8. Retreats at 30% HP (more with low ego). Entries that would use every pack with no ally near are penalised. |
| Buy passive | Every skill had to be bought and never refilled | Omen's model: income, gun and armour buying exactly Omen's. Each skill has one free charge that comes back on its cooldown: Paint Shells 8 s, Boom Bot 18 s, Blast Pack 6 s. A second charge is bought for 300 / 300 / 200. Bought charges are lost on death, and free ones return at once. |

**Damage:**

| Skill | Before | After |
|---|---|---|
| Paint Shells | 65 + 55% AP | 85 + 75% AP |
| Clusters | 15 + 12.5% | 20 + 20% |
| Boom Bot | 45 + 45% | 70 + 70% |
| Showstopper | 180 + 120%, 60 s | 240 + 150%, 50 s |
| Blast Pack | 20 + 15%, arms after 1 s | 30 + 25%, arms after 0.5 s |
| Self-damage | 25% | 15% |

## Verification

- **Native tests:** `cargo test --release` 712 passed, 0 failed, 0 warnings. New tests cover:
  - assist layers off by default, with an ordinary laner's move passing through;
  - one effect phase per bomblet per tick;
  - nothing re-sent every tick plays longer than 1/60 s;
  - ultimate decisions in a crowd, with refunds and re-targeting;
  - the guide hold, the takeoff gate and wall snapping;
  - the strafe cap;
  - Omen/Raze income parity, free and bought charges.
- **Raze lab:** 88 tests pass and `raze.wasm` is rebuilt. Browser tests `test-raze-lab` (6,480 native/WASM frames), `v32`, `v3`, `shop` and `keyboard` all pass. `verify-raze.py` passes, and the approved pixels are unchanged.
- **Other checks:** the editor stats-changer and theme checks pass, the roster is regenerated, and the 13 Manager tests pass.
- **Not run:** `check-raze-release.py` needs the Windows toolchain and Edge.
- **Windows DLL:** rebuilt.

## For you in game

- Nobody goes back and forth. If someone does, enable diagnostics and send me the `pathing_redirects` lines.
- Raze shows one cluster sprite per bomblet and her poses match the Lab.
- She fires Showstopper into groups, has an escape left after entries, and lasts longer.
