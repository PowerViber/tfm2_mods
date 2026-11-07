# Round 156: dashes and knockbacks move through the engine (native 0.10.91, tfm2_custom 0.2.93, Valorant 0.1.33)

## What you reported
Grimhald's Skull Bash and the enemy it knocks back, Raze's satchel jump, Levi, Aegis Zero and Rally all "stop animating and teleport" instead of dashing along their path. Base-game dashes look smooth at every speed, even 0.5×.

## Cause
These moves set the unit's position by hand every tick (`entity_set_pos`), and Raze's satchel push did it in a single tick. That call only writes the coordinates:
- the game never knows the unit is moving, so the screen shows a jump;
- the unit keeps its standing pose, so the animation looks frozen.

Base-game dashes and knockbacks are smooth because the engine moves the unit itself.

## Fix
A new shared helper, `native/tfm2_custom_ai/src/motion.rs`, moves units through the engine's own forced movement (a direction and a speed) instead of by hand.

| Champion | Converted |
|---|---|
| Grimhald | Skull Bash charge, the knocked-back enemy, punches, pillar nudge; the leap is one engine glide |
| Raze | Satchel jump flight; the satchel push now travels over 0.2 s instead of landing at once |
| Levi | Cable flight and gas dash |
| Aegis Zero | Arondight charge and the carried enemy |
| Rally | Her jumps and dashes |

What stays as it was:
- Real teleports stay instant: Flash, Minato, Omen, pearls, Grimhald's invisible Iron Ghost walk, and Aegis Zero's banished flight.
- The cage and domain edges stay deliberate snaps.
- Damage, ranges and AI are unchanged.

Each kit's own checks ignore its own dash, so a dash doesn't read as being crowd-controlled. Levi's flight, Raze's flight and Rally's checks were updated, and so was the shared AI.

## For future champions
- `motion_guard_tests.rs` fails the build if any file gains a hand-set position call that isn't on its allowlist of real teleports.
- The rule is in `docs/champion-guide-addendum.md` §12 and in Notion (TFM2 02, 03 and 04).

## Switch to compare
- **`motion.flag`:** put an empty file with this name in `mods/tfm2_custom_ai/` (next to the DLL) to get the old movement back after a game restart.
- **`motion_trace.flag`:** logs every motion to `motion_log.txt`.

## Verification
- **Tests:** `cargo test --release`: 1029 passed, 0 failed. This includes the guard test and an engine-motion test (forced move, named `mv_self`, correct direction and speed, old path with the flag). The Grimhald Lab: 38 passed, with the WASM rebuilt.
- **Build:** Windows DLL rebuilt; it reports 0.10.91.

## Not proven yet
- **On screen:** whether the game draws these forced moves as smoothly as base dashes. It's the engine's own movement, so it should, but please check a Skull Bash, a Raze satchel jump and push, a Levi flight, an Aegis Zero charge and a Rally jump. If something still jumps, send `motion_log.txt` (with `motion_trace.flag`) and compare with `motion.flag`.
- **Grimhald's leap and cc_immune:** the leap is issued before his mid-air immunity starts. If the game cancels it anyway, he'll land at the start, and that would show at once.
- **Steve's boat, Isliid's sword nudges and Scribble's moves** still set positions by hand. They're listed in the guard for a later round.
