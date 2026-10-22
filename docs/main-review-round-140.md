# Round 140: confused players, Steve, Gojo, modded HP and the stats changer

Native 0.10.66, champion pack 0.2.66, JJK and Blockcraft packs bumped with their dependency.

## What you reported

- Players look confused, maybe because of the new stats.
- Steve moves back and forth oddly and underperforms.
- Gojo's ultimate doesn't stun, and Gojo is a bit too strong.
- Modded champions have too much HP, apart from a few that are already balanced.
- Add the new stats changer to the editor.

## Confused players (my round-139 bug)

### The cause

Round 139's `athlete_play.rs` rewrote the engine's inputs for every player in four harmful ways:

1. **It held and injected casts and attacks.** The repo's own confirmed rule forbids this: a player AI must never hold or inject casts, or the engine's planner breaks.
2. **Ground-aimed casts never fired.** The cast delay re-armed every tick whenever the target moved, so about half of all athletes stood still until their target stopped.
3. **No rule had hysteresis.** Retreating, roaming, waiting for allies, kiting and chasing each switched on and off at a fixed distance, so players walked back and forth across it.
4. **The server and live simulations shared memory.** Both are keyed by the same seed, so each read the other's state.

My unit tests only ever checked one frame at a time, which is why none of this showed up.

### The fix

| What | Now |
|---|---|
| Inputs it can change | **Moves only.** Casts and attacks always pass through, and a retreat never uses an escape skill. |
| Cast-related values | control_speed, skill_hit and judgement act only inside kits that cast for themselves (Gojo, Omen, DIO). |
| Commitment | A started move is kept for 45 ticks or until it arrives. The same rule can't fire again for 60 ticks; waiting for allies and roaming wait 300. |
| Separate start and stop distances | ego: retreat starts with an enemy within 50,000 and stops 75,000 away with 10% more HP. roaming: starts when no enemy is within 100,000, stops below 70,000. order: holds until allies are 90,000 away. Kiting: at most one step back per 30 ticks. |
| Low aggression | Only gives up chasing a hurt enemy (below 50% HP). |
| skill_avoid | Rolled once per zone. |
| Memory | Kept per simulation and player. |
| Team planner | The running plan gets +15. Enemy visibility alone no longer triggers a re-plan. Job guidance hands back to the engine at 24,000 from the job point and takes over again only beyond 40,000. |

New tests play 600 ticks of the engine and the rewrite reacting to each other, across 4 champions, 3 scenarios, all 9 values and 4 levels each. They check three things:

- no rule starts more than 3 times;
- nobody stands still for more than 30 ticks while the engine asks them to move;
- an average athlete is never rewritten.

**What the stats still do** (rollout, 96 seeds; team A at 20/80 against an average team B):

| Value | Effect |
|---|---|
| ego 20 | Objective share falls from 0.48 to 0.37. |
| aggressive 80 | More fights started with numbers (0.52), fewer objectives (0.42). |
| order 80, roaming 80 | Small shifts. |

Table: `docs/round-139-results/athlete-stats.json`.

## Steve

Round 139 made recipe cooldowns start and check by recipe **group**. The 5 beacons, 6 splash potions, 2 lingering potions and the armour family each ended up sharing one cooldown, which is why he underperformed.

- Each recipe has its own cooldown again, as before round 139.
- The real bug was that a failed placement restored the group's cooldown. It now restores the recipe's own.
- `editor/steve.wasm` was rebuilt, and the Steve lab passes 12,960 parity frames.

## Gojo

### Ultimate stun

The stun code was correct. It just rarely landed:

- the AI mostly picks the Quick domain (one 1 s stun);
- the Full domain's 1 s warning let everyone walk out of a 64,000 radius.

Now:

- every enemy inside is stunned for 1 s when the domain opens, for both Full and Quick;
- enemies inside are slowed 40% during the Full warning;
- the Full warning is 45 ticks instead of 60.

Tests check that an enemy walking straight out at full speed is still caught.

### Nerf

| Lever | Before | After |
|---|---|---|
| Mod Power AP | +35% | +15% |
| Mod Power HP | +10% | 0 |
| Infinity skill-damage cut | 75% | 55% (basic attacks stay at 30%) |
| Infinity shield | 200 + 50% AP | 160 + 40% AP |
| Six Eyes cooldown recovery | ×2 | ×1.5 (ultimate 30 s → 40 s) |

The tooltip and the Gojo lab were updated to match.

## Modded HP

The native Mod Power bonus had never been lowered. Changes:

| Champion | Mod Power HP | Data HP |
|---|---|---|
| Aegis Zero | 15% → 0 | 1180/120 → 1080/110 |
| Coder | 30% → 10% | – |
| Steve | 15% → 5% | – |
| Vader | 20% → 10% | – |
| DIO | 15% → 5% | – |
| David | 10% → 5% | – |
| Minato | 10% → 5% | – |
| Gojo | 10% → 0 | – |
| Unified | – | 1100/120 → 1000/105 |

- **Unchanged (already close to balanced):** Levi, Frieren, V1, Scribble, Vesper and Omen.
- **Below average, also unchanged:** Isliid and Magnus.
- **Existing careers:**
  - Mod Power reaches them at once, because it is native.
  - The Aegis and Unified data changes reach them through `tools/migrate-champion-reworks.py`, which applies them once per career.

## Stats changer in the editor

**Player drawer, Match behaviour**

- The nine values, each with a live line saying what it does in a match.
- Presets: Shot-caller, Duelist, Lurker, Anchor, Mechanic, Average.
- A preview of the game-sense ratings a fresh athlete starts with.

**Game sense**

- The athlete's exported Macro, Micro, Map Awareness, Strategy and training focus, editable with sliders.
- Changes are staged in the Changes tab, then sent with **Apply to career**. This writes `game_sense_overrides_v1.json`.
- The mod applies the file once, on the next management tick, and only to the matching career.

**Bulk edit**

New groups: all 9 profile values, all 4 play-style values, and each game-sense rating (or all four).

A native test checks that every threshold in the editor's descriptions matches `athlete_play.rs`.

## Verification

- **Native:** `cargo test --release` 578 passed, 0 failed, 0 warnings. The Windows DLL builds.
- **Labs and parity:**
  - Steve lab: 48 tests and 12,960 parity frames;
  - Gojo lab: 83 tests and 13 browser checks;
  - Frieren parity: 25,920 frames;
  - Vesper parity: 0 mismatches;
  - Unified verifier passes.
- **Editor:** `verify-stats-changer.cjs` and `verify-editor-theme.cjs` (24 pages, light and dark, 390 px) pass.
- **Tools:** migration tests and the 13 Manager tests pass.

## Still open

- **Live checks:** none were possible here. Please watch for:
  - players no longer moving back and forth;
  - casts not freezing anyone;
  - Gojo's domain stunning on open;
  - Steve's output.
- **Applying game-sense ratings:** this needs the career to have run once with the mod, so the export exists. Changes apply on the next management tick.
- **Micro at 80** still trades objectives for fights in the rollout.
