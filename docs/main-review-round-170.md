# Round 170: Kicker gets down from walls (native 0.10.104, tfm2_custom 0.2.107, `kicker-1.4`)

**Reihan's clip** (2026-11-13): Kicker stands on terrain next to a nexus crystal for about 8 s and never jumps down, although he can Drop Kick ("drop in"). Reihan also noted that his wall basic applies a mark.

## Causes
1. **The nexus was a target.** The bridge marked only `is_tower()` entities as structures. The nexus (`EntityKindV1::Nexus`) read as an ordinary enemy unit, so the rooftop AI's "something in throwing range: stay and throw" held him up forever, and the 3 s idle drop never came.
2. **The game's walks were only stopped at the wall's edge** (round 168: "he never walks off a wall"). Nothing turned the game's wish to go elsewhere into his own way down, so he jumped down only for a target, a fight, or after being idle.

## Fixes
- **Structures:** towers **and the nexus** are never his targets, and never a reason to stay on a wall.
- **Following the game's walk down:**
  - **Recording it:** when the input layer stops a walk more than 30,000 past the wall's edge, it records where the game wanted him to go (`kicker::note_want`). This is a small per-match native table, because the input AI's sim view is read-only.
  - **Using it:** on a wall with nothing to fight (no enemy champion in throwing range, no Rock Rain, Skyfall or Drop Kick on a target), he **jumps down toward it** with a Drop Kick: no charge, the usual Drop-in.
- **Waves:** a wave in throwing range keeps him up only when the game doesn't want him elsewhere.
- **A Marked Dash down** (his wall basic marks):
  - **On a target:** from a wall he dashes down behind a marked enemy beyond the Drop Kick's reach, in a fight or on a pick (low or alone).
  - **The game's way:** onto a marked enemy that lies toward where the game wants him (within 45 degrees), when that lands him closer than a plain drop.
  - **The dash itself:** a jump down, so it ends behind the target with the Drop-in and the dash armor.

## Verification
- **Native tests:** **1,285 passed**, 0 warnings, including 68 Kicker tests. The new ones cover:
  - the wall basic marks, and the Marked Dash down ends behind the target with the Drop-in;
  - the AI dashes down onto a marked enemy out of Drop Kick reach;
  - he jumps down toward the game's walk, and a wave doesn't hold him then;
  - he stays when an enemy champion is in throwing range;
  - a marked enemy the other way is not chased;
  - the nexus is no reason to stay;
  - glue: the nexus reads as a structure, and a recorded wish reaches the frame and then lapses.
- **Other checks:** the `kicker_lab` tests pass and its wasm is rebuilt; the generators are current; the Mod Manager tests (15) pass.
- **DLL:** the Windows DLL reports 0.10.104.

## Live acceptance (pending)
- He no longer stands on a wall by the nexus.
- He jumps down when the game sends him off the wall (to lane, to a fight, home).
- From a wall he dashes down onto the enemies his rock basics marked.
