# Round 141: pathing, placement and per-tick cost

Native 0.10.67; champion pack 0.2.67; JJK, Blockcraft and Valorant packs bumped with their dependency.

## What you reported

1. Since the new stats, pathing is horrendous for most champions, and Levi is the worst.
2. Check everything that could cause weird placement.
3. The game runs slowly all the time, not just in spikes. A 4× replay doesn't drop to 1× on highlights.

## Why pathing broke

Before PR #42, the mod's per-player input layer (`steve::WallAi`) returned early for almost every champion, so the engine pathed them untouched. Since then, three things changed:

- **Every walking order is rewritten.** Each one goes through athlete_play, the team planner's job guidance and staging.
- **Whole teams are steered.** The planner also rewrites the engine's own team strategy settings.
- **Units get snapped back.** Several abilities move units every tick without noticing that something else already moved them.

Four audits (team guidance, Levi, the other redirects, and every teleport) found the causes below, worst first. All are fixed.

| # | Cause | Fix |
|---|---|---|
| 1 | When no detour around a Steve or BlockCraft wall was found, the champion was sent to its own position and **froze** | No route now means no override (fallback detour or the engine's order). Never a move within 3,000 of yourself. |
| 2 | The planner rewrote team strategy (Gather/Split, Wave/Join) every re-plan, even for weak plans, and never restored the coach's choice | Only plans scoring 40 or more write, and only after leading two re-plans. Each field is held for 600 ticks. All four fields are written, the coach's values fill the rest, and they come back when no plan qualifies. |
| 3 | Levi's cables flew toward the engine's destination while his walk was redirected elsewhere, so he flew back and forth | Levi skips the walking rewrites; his profile feeds his brain (retreat, destination) instead. He only takes off toward where he is walking. His flight accepts knockbacks and pushes. His memory is kept per simulation. |
| 4 | The whole team was sent to every plan. Defend fired on one enemy, Reinforce on any trade. High Macro sent players more often. Hand-off ping-pong at 24k/40k. Farm moves were overridden. | Only players who can arrive in time are sent (4 s for fights, 10 s for objectives). Laners stay with a live wave. Defend and Reinforce need two enemies. The Macro gate is conservative. Guidance takes over once per job and never overrides farming or attacks. Hurt players withdraw home; healthy ones step back. |
| 5 | Gojo's domain pushed allies and pushed into walls, and snapped caught units back from anywhere | It pushes only enemies who aren't immune, along a legal path. The snap-back is capped at 40,000 and released on death. Displacement is adopted instead of undone. |
| 6 | Rallies pulled every teammate within 220,000 to an enemy centroid or a wall line | The rally point sits on the friendly side, only nearby or assigned teammates answer, and it is held without ping-pong. |
| 7 | athlete_play's order and roam moves aimed at stale ally positions and could cross walls | They use the ally's current position, stop at walls, and drop on teleports. |
| 8 | Omen's smoke check made enemies circle the smoke | A smoke counts as checked on entering it. |
| 9 | Coder's rollback and swaps put units in walls and sent respawned enemies back to where they died | All go through a safe-placement check. Immune enemies are skipped. Rollback history is at most 200 ticks old and cleared on death. |
| 10 | Steve's boat wall teleported him through terrain, and his wall push-out fought other movers | Over-far or blocked releases cancel. Reels stop when he is controlled. The push-out needs open ground and leaves units that something else just moved alone. |
| 11 | Omen could land on a tower, or on a wall that rose after he cast | He aims 20,000 from the tower on his side, and the landing is re-checked on arrival. |
| 12 | Steve's "never walk alone" sent him across the map | The search is capped at 120,000, measured from the enemy group. |
| 13 | Avoidance exit points sat on the far side of zones; BlockCraft and Magnus zones rerolled every tile | Exits are on your own side, and those zones count as zones. |
| 14 | Gundam's charge, Isliid's pulls, Scribble's follow and Gojo v1's push ignored walls | All go through safe placement. Scribble only moves nearby allies. |

**Checked and fine:** the terrain grid, coordinate frames, farming (it writes no movement), and the teleports of Vesper, Unified, Magnus, Minato, DIO and Flash.

### In-game switches

`mods/tfm2_custom_ai/settings.json` holds three switches, read when the server starts:

```json
{ "assist": { "athlete_moves": true, "team_guidance": true, "team_strategy": true } }
```

Turning one off gives that layer back to the engine. If anything still looks off, turn them off one at a time and tell me which one fixes it.

## Slowness and the 4× replay

The mod can't control replay speed or highlights; that behaviour belongs to the engine. Replays re-run the whole match, including every line of mod gameplay code, so a heavy simulation can't keep up at 4× (240 ticks a second).

The audit found no per-tick disk writes by default. The cost was game-state reads.

**The biggest single regression** (Oct 18): `tactics::marks_for` scanned every entity 5–6 times per player per tick, even though the shipped map plan (smoke and wall marks only) never needed it. It now returns early. It also no longer turns off the idle shortcut that lets most players skip the movement pipeline.

**Other fixes:**
- One champion snapshot per simulation and tick is shared by the input layer, athlete_play and staging. It is checked against a fingerprint of each champion, so it is never stale.
- Planner scans are smaller: a farm scan only when it matters, objectives found once per tick, buff names filtered before copying.
- Client pins are re-read only when the journal changes. The export comparison is in memory.

**Measured** with a mock match (10 champions with 12 buffs each, 210 units, the shipped map plan, 600 ticks):

| Host calls per tick | Before | After |
|---|---:|---:|
| Median | 74,015 | 4,390 |
| Re-plan ticks | 83,149 | 13,026 |
| Map-plan step | 63,259 | 0 |

That is about **17× fewer**, and cached and uncached runs give identical moves over 600 ticks. Mock wall time fell from about 738 to 150 µs per tick; real calls cost more than the mock's.

**Please time a 4× replay** and tell me whether highlights slow down to 1× again.

## Stats rollout

Two average teams split objectives about evenly again (0.535). An intermediate version of this round had dropped it to 0.37, because objectives couldn't gather the team; that is fixed.

| Value | Effect |
|---|---|
| ego 20 | Starts fights with more players far less often (0.19) and dies more (64 deaths a game against 39). |
| aggressive 80 | Fewer objectives (0.465). |
| order 80 | Fewer objectives (0.502). |
| Micro | Slightly fewer objectives at 80; within noise. |

Deaths per game are higher than before round 141 (about 39 against 29). The rollout has no engine retreat, so laners who now stay in lane fight longer. A real match retreats them.

Table: `docs/round-139-results/athlete-stats.json`.

## Verification

- **Native tests:** `cargo test --release` 609 passed, 0 failed, 0 warnings. They include:
  - 600-tick pathing tests: at most 10% of moves guided, at most 3 flips, no freezes, no guided step through a wall, farm moves kept;
  - Levi takeoff tests;
  - safe-placement tests;
  - the cached-versus-uncached equivalence test.
- **Windows DLL:** builds.
- **Parity:**
  - Gojo: lab tests and browser checks pass, parity 51,840 frames;
  - Steve: lab tests pass (its browser test needs a Windows vector binary, not run here);
  - Frieren: 25,920 frames;
  - Vesper: 0 mismatches;
  - Universal lab: rebuilt.
- **Editor:** the stats-changer and theme checks pass.
- **Other:** migration tests and 13 Manager tests pass.

## Still open

- **Live checks:** pathing near Steve walls, Levi, team movement, and replay speed on highlights.
- **Not done this round:**
  - Gojo v2's per-tick observation still builds its own snapshot.
  - The management tick still parses every athlete's JSON.
  - The planner's wall grid and routes are rebuilt each re-plan (CPU only).
  - DIO's staging open time is unchanged; its meaning differs from Gojo's.
