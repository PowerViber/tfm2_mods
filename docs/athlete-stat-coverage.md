# Which champions use the athlete's stats (audit, round 163)

Checked on 2026-11-11 at main b427163 (native 0.10.97). Re-check by grepping for `pinned_owner`, `athlete_ai::traits`, `universal_ai::profile`, the `Profile` helpers and `PROFILE_*` / `profile[i]` in each kit.

**The 13 stats:**
- **The 9 profile values:** 0 ego, 1 judgement, 2 positioning, 3 control_speed, 4 skill_hit, 5 skill_avoid, 6 aggressive, 7 order, 8 roaming. They are read from the game's athlete `stat` record.
- **The 4 game-sense ratings:** 9 Macro, 10 Micro, 11 Map Awareness, 12 Strategy. The mod derives them.
- **Not read by the mod yet:** the game's record also has `last_hit`, `concentration` and `mental`.

## Shared layers
| Layer | Stats | Default |
|---|---|---|
| Movement (`athlete_play.rs`) | ego, positioning, aggressive, order, roaming, skill_avoid | **Off** (`mods/tfm2_custom_ai/settings.json` → `athlete_moves`, since round 143) |
| Team planner (`universal_ai.rs`) | Macro, Micro, Map Awareness, Strategy | **Off** (`team_guidance`, `team_strategy`) |
| Farming (`farming.rs`) | judgement ≥ 80 lets a jungler open the Epic or Serpent alone | On |
| Global Flash (`flash.rs`) | none | — |

In a default install, the farming gate is the only stat effect the mod applies to every champion.

## Each kit's own AI
| Champion | Stats it reads | Count |
|---|---|---|
| Raze | all 13 | 13 |
| Contender | all 13 | 13 |
| Omen | control_speed, positioning, skill_hit, skill_avoid | 4 |
| Rally | control_speed, skill_hit, Micro | 3 |
| Thalassa | judgement, skill_hit, aggressive | 3 |
| Grimhald | judgement, skill_hit, aggressive | 3 |
| Kratos | positioning, aggressive | 2 |
| DIO (ultimate only) | control_speed, positioning | 2 |
| Gojo v2 | ego, control_speed | 2 |
| Wright | aggressive | 1 |
| Steve | skill_avoid (zone step, through the movement layer) | 1 |
| Levi | the movement layer's output, fed into his own brain | 0 |
| Coder (it has its own rank system), Magnus, Minato, Unified Theory, Vesper, Frieren and Stark, David, V1, Vader, Aegis Zero, Isliid, Scribble | none | 0 |
| The 60 base-game champions | only what the engine itself does with the stats (not in this repo; see Part 0 in `docs/main-review-round-163.md`) | — |

## Fixing it
- Kits with 0–2 stats can adopt Fight Sense (`docs/fight-sense-guide.md`) and give each stat a decision, with a paired 20/80 test per stat.
- Base-game casts belong to the engine and must never be held or injected from the input layer. For those champions, the way in is the engine's own formulas (Part 0).
