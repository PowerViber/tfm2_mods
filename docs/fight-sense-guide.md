# Fight Sense: the shared read of a fight

**Status:** round 163, added but not used in game yet. No champion calls it, so nothing plays differently. Akane will be the first user, and other kits can adopt it one by one.

Fight Sense answers the questions every champion AI keeps asking:
- how much it takes to kill a target;
- what is waiting at a spot;
- where I can get out to;
- who is missing and could be here.

Each AI used to scan champions, cooldowns, towers and walls for itself. Fight Sense reads them once per tick per team, and the athlete's stats shape the answers. Combat numbers never change.

## Files
| File | What |
|---|---|
| `native/tfm2_custom_ai/src/fight_sense_core.rs` | Host-free and serde. Holds the snapshot types and every score. It needs only `athlete_ai_core.rs` and `game_sense_core.rs`, so a Lab can include it with `#[path]` the way `universal_lab` includes `game_sense_core.rs`. |
| `native/tfm2_custom_ai/src/fight_sense.rs` | The host reader. `fight_sense::read(sim, team) -> Arc<Sense>` builds at most once per (match, tick, team); every caller in that tick shares the result. |
| `native/tfm2_custom_ai/src/athlete_ai_core.rs` | New `Profile` helpers, in a separate `impl` block. The existing helpers are unchanged. |
| `lib.rs` `walls::snapshot()` | Copies of the wall and bush grids. |

## What the snapshot holds (`Sense`)
- **Units (champions):**
  - HP, max HP, shield, armour, MR, attack, magic power, move speed and attack range;
  - position, velocity (per tick, from the last position) and last seen;
  - `flash_ready`, from the `flash_cd` buff;
  - `ready` for skill, skill 2 and ult, from `player.cooldowns()` (the ult also needs to be unlocked);
  - `tools`: Dash, Blink, Cc, Heal, Shield and Damage bits per slot, from the roster kinds in `rules/champions.json`;
  - `protection`: CC-immune, damage reduction (all, skill and basic, from buffs), V1's guard (`v1_guard` without `v1_parry_cd`) and invisible;
  - stunned.
- **Fog:** an enemy out of sight keeps its last visible state, with `visible: false`. Its Flash counts as ready unless the team saw it used.
- **Invisible** is a heuristic, because the API can't tell fog from stealth:
  - an enemy is invisible when it is hidden while one of our champions is within 30,000 and it isn't in a bush;
  - an ally is invisible when it is hidden from an enemy within 30,000 and isn't in a bush.
- **Towers:** position, range 85,000, alive, damage per shot (its Attack), and minions of each team in range.
- **Creeps:** minions, jungle camps, Epic and Serpent that the team can see.
- **Terrain:** the wall and bush grids (30×30 cells of 32,000).
- **Memory, per match and team:**
  - who used Flash and when (seen going down, forgotten after 7,200 ticks);
  - who is missing and since when.
  - It only learns from what the team saw.

## Scores
| Function | What it returns |
|---|---|
| `Sense::effective_hp(target, Physical/Magic/True, Basic/Skill)` | (HP + shield) × (100 + resist) / 100 ÷ (1 − reduction). The armour formula is the repo's viability model (`raze_viability::mitigate`) and is still to be confirmed against the engine. 100% reduction is unkillable (`i64::MAX`). |
| `Sense::escape_margin(target)` | Extra burst a kill needs: +25% with Flash up, +15% with a ready dash or blink, +15% with a ready heal or shield (they add up). `not_now` if CC-immune or invisible; `no_parry_open` if V1's guard is up. |
| `Sense::lethal(burst, target, type, hit, profile)` | `burst` ≥ effective HP × (1 + margin × aggression scale). The caller supplies its own burst. |
| `sense.isolation(target, radius)` | How many of the target's allies are within `radius`. |
| `sense.danger_at(point, ticks, team, me)` | Damage, CC ticks and source count from enemies who can reach the point in time (walk, plus a dash or blink, plus Flash, plus range): their attacks, plus a burst estimate per ready damaging skill and an estimate per ready CC skill. A tower counts only when nothing else is in its range for it to shoot. |
| `sense.escape_routes(me, tools, profile)` | For each tool (`Blink`, which crosses walls; `Dash` and `Speed`, which walls stop; `Flash`), a ring of landing spots. Spots in enemy tower range are dropped. Bushes with no enemy in sight and allied towers score better; danger and nearby enemies score worse. Best first, each marked `safe` against what the athlete accepts. |
| `sense.missing_threat(point, team, profile)` | Enemies missing for less than the athlete's memory, whose walk plus range since they were last seen (capped by the athlete's threat radius) covers the point. |

The estimates (skill burst 100% attack + 80% AP, CC 60 ticks, dash 30,000, blink 40,000) are named constants at the top of the core.

## How the athlete shapes it (`Profile`)
An average athlete (every value 50) gets the base value from each helper.

| Helper | Stat | Effect |
|---|---|---|
| `lethal_margin_scale()` | aggressive | The margin is ×120% at 0 and ×80% at 100. |
| `accepted_damage(ehp)` | positioning and ego (caution = (positioning + 100 − ego) / 2) | Accepts 50% of effective HP on average, 30% at full caution, 70% with none. |
| `aim_lead(vel, travel)` | skill_hit | Leads the share of movement given by `prediction_ticks` (30% to 100%). |
| `threat_radius(base)` | Map Awareness | 50% to 150% of the base radius. |
| `missing_memory_ticks()` | Map Awareness | 3 s to 10 s. |
| `route_candidates()` | Micro | 8, 16 or 32 landing spots tried. |

`Profile::reaction_ticks()` (control_speed), `Profile::score()` (judgement and ego) and `prediction_ticks()` (skill_hit) were already there; use them alongside these.

## Adopting it in a kit
1. `let sense = crate::fight_sense::read(sim, my_team);` once per think, plus `let profile = crate::athlete_ai::pinned_owner(sim, owner, &mut self.profile);`.
2. Ask the questions above instead of scanning yourself. Pass your own burst to `lethal`.
3. For a Lab, copy the `Sense` into the frame. It is serde, so the core runs the same scores in wasm.
4. Add a paired 20/80 test for each stat the kit's decision now depends on.

## Tests
`fight_sense_core::tests` and `fight_sense::tests` (16) cover:
- the margins;
- immunity and V1's guard;
- effective HP;
- tower danger with and without tanking units;
- reach and ready skills;
- a dash stopped by a wall vs a blink over it;
- routes avoiding enemy towers and preferring allied towers and bushes;
- Flash memory and its expiry;
- missing enemies fading by Map Awareness;
- 20/80 pairs for each helper;
- a JSON round-trip;
- the one-build-per-tick cache.
