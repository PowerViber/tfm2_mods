# Round 148: Contender lands his hits, punches more, clears camps, and dashes

Native 0.10.84, champion pack 0.2.85.

## What you reported

- Contender's basic attacks are weak.
- He rarely uses skills.
- As a jungler he can't clear camps.
- You asked that most of his skills be dashes, like Paquito in Mobile Legends.

## Causes

All four are in how our native code handles his attacks. His numbers aren't the problem.

| # | Cause | Where |
|---|---|---|
| 1 | His basic attack's data effect only leaves a `ct_hit` tag; the core then re-checked a hit the game had already accepted. It required a centre-to-centre distance ≤ 23,000, but the game measures range edge to edge. It also required a clear line through coarse 32,000-unit wall cells. Most hits on big units and jungle camps (which sit in walled pockets) dealt **no damage**. | `contender_core.rs` basics |
| 2 | Every basic attack sent its jab/cross pose as a 15-tick Animation CC. That is a real CC to the game, as we found with Minato, so it locked him after every hit. | `contender.rs` `Effect::Pose` |
| 3 | Skill reach was measured centre to centre (Combination 26,000, Body Shot and Uppercut 23,000). The game parks him at its edge-to-edge attack range, so every punch scored "out of reach" and only Slip was picked. | `contender_core.rs` `plan` |
| 4 | Skill hits used the same coarse wall-cell line, so near walls and in the jungle they hit nothing. | `capsule_target` |

## Fixes

1. **Accepted basic attacks always land** on their target, when it is alive and on the same side of any ring. The Boxing Lab, which has no host, keeps its own check.
2. **Basic poses send no CC.** The game's own attack animation shows instead. Skill poses still hold for the cast's length only.
3. **Reach is edge to edge** (`edge_distance`). Guard is used within 25,000 of edge distance and Slip avoids within 6,000.
4. **New hit line of sight (`clear_line`):** only wall cells strictly between the two bodies block a hit. The cells under the striker or the target don't. Slip's movement and the ring bounds still use the strict check.
5. **Dash punches (`rules/contender.json` `dashTravel`):**

   | Skill | Dash | Then |
   |---|---|---|
   | Combination | 14,000 over its first 8 ticks | Both punches (the second still re-aims) |
   | Body Shot | 10,000 over its wind-up | The body blow |
   | Uppercut | 12,000 over its wind-up | The launch |

   - **Stopping:** each dash ends 1,500 short of the target's edge and is cut short before walls.
   - **Rooted:** no dash while rooted.
   - **Interrupted:** stunned or airborne ends the cast, and the dash with it.
   - **Reach:** punches now reach targets up to 33,000–37,000 away, edge to edge.
   - **Unchanged:** Slip (already a dash), Guard, damage, stamina, cooldowns and Main Event.

## Verification

- `cargo test --release`: 845 passed, 0 failed. New tests:
  - a host-accepted basic attack on a camp with an 18,000 radius, at edge range with wall cells under both bodies, lands 5 of 5;
  - the Lab still rejects a basic attack from 60,000 away;
  - a wall strictly between still blocks;
  - at edge range against a 15,000-radius target he punches, not Slips, at profiles 0, 50 and 100;
  - each dash covers its full travel, then stops short of the target's edge;
  - no dash while rooted;
  - a basic pose has no cast, so no CC;
  - a farmable camp at edge distance 30,000 gets punched.
- The duel approach test now starts beyond dash reach (edge 50,000). At the old 48,000 centre distance he now dash-punches.
- `contender_lab`: 49 passed, and `editor/boxing.wasm` is rebuilt.
- Boxing Lab browser checks pass (8 of 8, including 42 frames of native/WASM parity), run with Linux Chromium and the Linux lab binary.
- `check-contender-host.py` exits 0. `generate-roster --check` is up to date.
- The Windows DLL reports 0.10.84; only the 2 existing `valorant_guns` warnings appear.

## Not verified live

- Basic attack damage, skill frequency and jungle clearing in a real match.
- How the dashes look in game.
- In game, his basic attacks show the game's `attack` animation; the alternating jab/cross only shows in the Lab now.
