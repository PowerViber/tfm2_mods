# Round 169: Kicker plays the fight (native 0.10.103, tfm2_custom 0.2.106, `kicker-1.3`)

**Reihan, after watching him in mid:**
- He locks on to one enemy, even inside tower range and with others around.
- He rarely gets on walls.
- On a wall he rarely uses anything and just drops down, not contributing to the fight.
- He never casts Second Wind, which Reihan rates the strongest ult implemented (a death cancel, and fearless engages).
- **Idea:** on a wall, his basic attack becomes a mid-range rock throw.

## Causes
- **The lock-on:** a `Steer` toward the target every tick he didn't cast.
- **Tower dives:** a tower penalty that dropped whenever he had a kick charge.
- **Target choice:** the lowest-HP% enemy, however far.
- **Walls:** he only mounted with a wall within 40,000 and no target.
- **On a wall:** he acted only on an Ego-gated target; Rock Rain only on unmarked targets; Skyfall only with both charges; otherwise idle, then home after 10 s.
- **Second Wind:** it needed a fight around the target *and* no enemy within 50,000 of him.

## Changes (AI; the kit's numbers are unchanged except the new rock basic)
- **Targets:** a `pick()` score re-read every tick (nearer, lower, alone, fighting his team).
  - **Under a tower:** only with a **tower kick-out ready** (a charge, a safe way out, in reach, HP 50%+).
  - **Steering:** with a purpose only.
  - **Joining his team's fights:** whatever his Ego.
- **Walls:**
  - **Getting up:** he walks to a wall within 120,000 between fights and climbs it, and climbs a wall standing between him and a target.
  - **Up there:** he never idles: Rock Rain on any enemy in range, a single-charge Skyfall in fights, a Drop Kick into fights, basics at anything in range, and a 300,000 hop toward a far target.
  - **Leaving:** after 3 s idle, toward his teammates.
- **Rock basic (Reihan):** on a wall his basic reaches **45,000** (`kk_roof.range` +23,000, the additive field Kratos's blades use) and plays Codex's rock flight and hit effects. On the ground it's still the 22,000 kick.
- **Second Wind:**
  - **Death cancel:** at `30 - Ego/5` % HP with an enemy close, real damage, or an enemy projectile closing in. The bridge watches projectiles tick to tick (`Frame.shots_near`). Then he goes all in.
  - **Fearless engage:** the tag goes down the tick before a committed engage, scaled by Ego (60+: any engage; 35-59: picks and fights; under 35: teamfights of 3+), never from under an enemy tower.
  - **Teamfight:** at the start of one.
  - **While tagged:** he never escapes, may spend the held kick charge, and takes targets inside groups.
- **Cost:** a cheap reach check before the kick-out direction scan. The bench shows about 4.5 µs per tick alone on the mock host (was about 18).

## Verification
- **Native tests:** **1,280 passed**, 0 warnings, including 63 Kicker tests. The new ones cover:
  - no lock-on, and no tower targets without a kick-out;
  - the target score, and joining teamfights at Ego 10;
  - the rooftop rain, Skyfall, staying to throw, the long hop and the idle drop;
  - walking to a wall and climbing a wall in the way;
  - the death cancel by Ego, Flash and projectiles, and going all in after it;
  - the fearless engage by Ego, the teamfight tag, and targets in groups while tagged;
  - the rock basic's range and effects;
  - the projectile watch.
- **Other checks:** the `kicker_lab` tests pass and its wasm is rebuilt; the generators are current; the Mod Manager tests (15) pass.
- **DLL:** the Windows DLL reports 0.10.103.
- **Mock host:** it now models projectiles, for the projectile-watch test.

## Live acceptance (pending)
In mid:
1. **No more lock-on:** he no longer follows one enemy under its tower, and switches to better targets.
2. **Walls:** he climbs walls between fights and fights from them (rocks, Skyfall, Drop Kick, rock basics).
3. **Second Wind:** a death cancel at low HP, and a tag before his engages, more often at a high Ego.
