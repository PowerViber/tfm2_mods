# Round 142: jungle Levi clears the jungle again

Native 0.10.68, champion pack 0.2.68.

## What you reported

Jungle Levi doesn't clear the jungle like base Levi did. He shows up in a random lane, top or bot.

## Cause

Base Levi jungled through a chain of three steps:

1. The game walks him to a camp.
2. His brain's Travel flies him along that walking path.
3. His Sweep clears each camp on arrival.

Round 141 broke that chain in five places:

1. **Roam and order points became his destination.** For a jungler, roam fires almost all the time: no enemy is near him while he's in his jungle. It always picks a laner who is trading, so a single flight carried him deep into top or bot lane.
2. **The lane point stuck.** At a camp the game sends attacks, not moves, so his destination never refreshed. After clearing, he flew back to the lane.
3. **The takeoff check favoured lanes.** It compared directions in a straight line, so camps behind walls were refused and straight-line lane points passed.
4. **His idle fallback skipped small camps** and pointed at lanes.
5. **Camp plans set the minion-wave strategy to Wave priority,** which may route junglers to waves.

## Fixes

| Fix | Where |
|---|---|
| A jungle Levi always takes the game's own move order. A retreat from the athlete profile still applies. A lane Levi is unchanged. | `steve.rs` Levi block, `levi::feed` |
| A jungler's destination lapses 120 ticks after the last fresh order, and when he sweeps a camp. | `levi.rs` (`dest_at`, `DEST_STALE`) |
| Escapes, sweeps, steals and flights toward his walking destination always pass the takeoff check. | `levi.rs` (`takeoff_allowed`) |
| With nothing to do, a jungler heads for the nearest visible small camp that no teammate is clearing. | `levi.rs` (`jungle_camp`) |
| A camp plan only writes `object_buildup=Split` and leaves the coach's wave setting. | `strategic_policy.rs` |

## Verification

- **Native tests:** `cargo test --release` 614 passed, 0 failed, 0 warnings. The new tests cover:
  - jungler versus laner destination feed;
  - stale destination expiry;
  - free-camp choice;
  - the takeoff check;
  - camp-plan strategy.
- **Universal lab:** tests pass, and its wasm is unchanged.
- **Windows DLL:** rebuilt.

## In game

Jungle Levi should clear camps between ganks. If he still lane-hops, flip these switches one at a time in `mods/tfm2_custom_ai/settings.json`:

| Switch | Off, he jungles again | Then the cause is |
|---|---|---|
| `athlete_moves` | Yes | A profile rule still reaches him. |
| `team_strategy` | Yes | The engine's wave setting routes him to lanes. |
