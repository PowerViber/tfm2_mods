# Round 172: Kicker's kicks lead to hits, no 1v4 dives, wall escapes back (native 0.10.106, tfm2_custom 0.2.109, `kicker-1.6`)

**Reihan, from the latest games:**
- he "just pushes the enemy and doesn't hit them";
- he rushes into 4 people and dies often;
- when retreating he just runs, without getting on walls.

**Keep:** the engage, where he climbs a wall first and then uses skills on the enemy.

## Causes
- **Pushing:** the kick's "toward home" and "into a wall" directions counted even with nobody near where the target lands. A kick sent it 75,000 away from him, nobody could hit it, and nothing made him follow.
- **1v4:** joining a teammate's fight and the Ego 60+ fearless tag never compared numbers.
- **Running on the ground:** round 171 removed wall escapes because of the earlier "stuck on walls" reports. Those came from rules that round 171 fixed separately.

## Fixes
- **Kicks lead to hits:**
  - **Toward a teammate or toward home:** only with a teammate along the flight or within 40,000 of the landing.
  - **Into a wall:** only when he can follow (a dash charge, or a landing within 45,000).
  - **No re-kick:** the target he just kicked isn't kicked away again (except to peel).
  - **Follow-up:** after a kick or a carry he goes after the target. While it's flying he walks to where it lands; then he Marked-Dashes onto it beyond 25,000, or walks to it, and the game swings his basics.
- **No 1v4:** `outnumbered()` counts enemy champions within 60,000 of a spot against his team there, him included (two more than his side, or three with fewer than two of his). It blocks:
  - joining a teammate's fight;
  - choosing a target there (tagged or not);
  - the Ego 60+ fearless tag.
- **Retreats use walls again:**
  - **On the ground:** a Marked Dash away, or a climb up a wall close by on the side away from the danger.
  - **On a wall:** a hop along walls (300,000) to one farther from the danger and toward home or his teammates; otherwise a drop toward them.
  - **Once safe:** the perch rules bring him down.
- **Unchanged:** the wall-first engage, helping teammates (now numbers-checked), the perch rules, the game's wish, Second Wind and the careful Flash.

## Verification
- **Native tests:** **1,295 passed**, 0 warnings, including 78 Kicker tests. The new ones cover:
  - no pushback kicks (nobody at the landing, or a wall he can't follow to);
  - a kick toward a teammate beside the target still goes;
  - the follow-up dash or walk;
  - no joining a 4v2 (a 2v2 still pulls him in);
  - no Ego 80 tag into a 1v3;
  - retreat climbs on the safe side, hops along walls, and drops;
  - the updated held-charge, tag-group and Flash tests.
- **Bench:** about 9 µs per tick alone on the mock host (Akane 10, Contender 13).
- **Other checks:** the `kicker_lab` tests pass and its wasm is rebuilt; the generators are current; the Mod Manager tests (15) pass.
- **DLL:** the Windows DLL reports 0.10.106.

## Live acceptance (pending)
- His kicks set up hits: he follows the kicked target, and teammates are where it lands.
- No more 1v4 dives.
- He retreats onto and along walls, then comes down once safe.
- The wall-first engage is unchanged.
