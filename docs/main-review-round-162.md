# Round 162: Kratos's Rampage stays on target, plus engage speed (native 0.10.97, tfm2_custom 0.2.100)

Reihan, after merging round 161: Kratos is balanced overall, but **S2 usually runs into the turret** ("it always moves to the front"), and he wanted a speed buff for engaging.

## Why S2 ran into turrets
Every Rampage stage stepped a fixed 36,000 forward along a fixed direction, so 108,000 in all, **even after reaching the target**. He overshot by up to 108,000, and nothing checked for towers. The athlete also cast it from 80,000 away and on lane waves, which sit near their tower.

## Changes (rules `kratos-4`)
| | Before | After |
|---|---|---|
| Rampage advance | always 36,000 per stage | **stops at the enemy in front** (champions first), `stickGap` 12,000 short so the swing lands; full 36,000 when nobody's ahead |
| Towers | not checked | **an advance never ends inside an enemy tower's range** (`towerRange` 85,000) unless he started inside it |
| Athlete | Rampage any target in reach | **no Rampage on a target under their tower** unless he's already under it or it's a champion below 30% HP (a dive); farm Rampages skip waves under towers |
| Speed on swap | — | **+20% move speed for 2 s** with every weapon swap |
| Speed on Sight | — | **+25% move speed** for Heimdall's Sight's 1.5 s |
| Spartan Charge (new passive) | — | heading at a visible enemy champion within 60,000: **+20% for 2 s**, every 8 s |

The three speed bonuses share **one buff** (`krt_haste`): a weaker one never lowers a stronger one that's active, and they never add up. Everything is tunable under `rampage` and `speed` in `rules/kratos.json`.

## Verification
- `cargo test --release`: **1,159 passed**, 0 failed, 0 warnings. New tests:
  - the Rampage stops on a close foe and still travels 108,000 with nobody ahead;
  - it stops at the edge of a tower's range, but isn't cut if he started inside;
  - the athlete skips a healthy foe under a tower but dives a low one;
  - the speed buffs (Sight 25%, swap 20%, the strongest wins);
  - Spartan Charge triggers toward a champion, waits its cooldown and never fires walking away;
  - a mock-host test that the speed is one replaced buff.
- **The Chain Dash glue test was fixed:** it only passed because the AI's follow-up Rampage glided forward; it now checks the dash's own glide right after the cast.
- **The rebuilt `editor/kratos.wasm`,** checked directly:
  - a foe 20,000 ahead: he ends 12,000 short of it;
  - nobody ahead: the full 108,000;
  - a tower 120,000 ahead: he stops at 434,000, the range edge.
- **Other checks:** the Lab transport tests and the generators (kratos, roster) are current, and the Mod Manager load test passes.
- **The Windows DLL** builds with 0 warnings and reports **0.10.97**.
- Codex's browser tests need the local game-reference cache, so they weren't run here.

## Live acceptance (pending)
- He stops in front of his target in a real fight.
- No more tower dives from the Rampage.
- The engage speed feels right.
