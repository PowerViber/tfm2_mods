# Round 171: Kicker stops sitting on walls and goes to help (native 0.10.105, tfm2_custom 0.2.108, `kicker-1.5`)

**Reihan's clip** (2026-11-13, 15.5 s): Kicker ("Lv.3 Kuzan") stands on a wall cell next to a minion wave for the whole clip while his teammate Faker is dived and killed nearby. He has a marked minion and an enemy within reach, but never dashes or drops to help. Reihan says most teamfights go like this, and that he also stays stuck on walls when retreating or doing nothing, even with charges.

## Causes (in `kicker_core.rs` and the `steve.rs` input branch)
1. **The wave held him.** "A wave in throwing range: stay and clear it" returned early and reset his idle timer, so with a lane by the wall he farmed from it forever.
2. **The fight "wasn't on" for him.** `fight_on` needs an enemy champion within 90,000 of *him*, so a teammate dived a bit farther away counted for nothing.
3. **No dash toward a fight.** On a wall he Marked-Dashed only onto his chosen target, and only when it was marked. A marked minion on the way to the fight was never used; on the ground, the same.
4. **The game's wish rarely arrived.**
   - **No walks:** while he was throwing at minions, the game sent attacks, not walks.
   - **The threshold:** walks only counted when they ended more than 30,000 past the wall's edge.
   - **Recall:** it was never counted.
5. **Retreats climbed walls.** The escape rule mounted a wall on the ground and hopped wall to wall on a wall.
6. **Gank climbs repeated,** even when nothing would come within reach of the wall.

## Fixes
- **Helping a teammate:**
  - **Seen from farther:** a teammate within **200,000** with an enemy champion on it is a fight to join (`help_target`).
  - **Getting there:** a Marked Dash on **anything marked** that brings him closer: minions, creeps or enemies, landing behind it at least 20,000 closer (`dash_toward`). From a wall that's a jump down with the Drop-in and armor.
  - **Otherwise:** on a wall, a hop to a wall by the fight or a Drop Kick down toward it; on the ground, a walk there.
- **The wall is a perch, not a home:**
  - minions never hold him up or reset the idle timer;
  - only enemy champions within reach do;
  - after 3 s with no champion to hit, he comes down toward his teammates.
- **The game's wish:** any walk the game wants off the wall, and any recall (toward home), makes him jump down that way. It wins over an enemy in throwing range when he's under 40% HP.
- **Retreats go down:**
  - **On the ground:** a Marked Dash away from danger, otherwise the game's own retreat. No more climbing to flee.
  - **On a wall:** when hurt, or low with an enemy near, a Marked Dash away or a Drop Kick down toward his teammates or home.
- **Gank climbs:** only when an enemy champion is within Rock Rain range (90,000) of the wall, with no fight to join, and not for 15 s after leaving a wall idle.

## Verification
- **Native tests:** **1,292 passed**, 0 warnings, including 75 Kicker tests. The new ones cover:
  - Reihan's clip: he dashes to the marked minion toward the dived teammate, or drops toward the fight with nothing marked;
  - on the ground, the dash or a walk toward a teammate's fight;
  - waves never holding him;
  - coming down when hurt;
  - retreats never climbing, and never hopping wall to wall;
  - climbs only for walls with someone to hit from them, and the rest after leaving;
  - the recall home point.
- **Bench:** about 5 µs per tick.
- **Other checks:** the `kicker_lab` tests pass and its wasm is rebuilt; the generators are current; the Mod Manager tests (15) pass.
- **DLL:** the Windows DLL reports 0.10.105.

## Live acceptance (pending)
- When a teammate is dived nearby, he comes: by dash, hop, drop or walk.
- He doesn't farm from walls while a fight is on.
- He retreats on the ground and recalls normally.
- He climbs walls only when there's someone to hit from them.
