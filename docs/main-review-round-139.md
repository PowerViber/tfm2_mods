# Round 139: review of main through PR #42, fixes, the champion roster and athlete stats

Native 0.10.65, champion pack 0.2.65, Frieren 0.1.22, Valorant 0.1.15. This branch builds on PR #39 after merging main into it.

## What was asked

1. Verify the changes since PR #38. The two big ones:
   - every champion can use skills on waves, camps and objectives;
   - athletes have new per-player stats.

   Then answer: do the stats actually change gameplay?
2. Make every champion readable as an object: **champion – family – skills – basic attack**. Each skill says what it is (dash, projectile…) and what it is used for (clear, engage, escape…).
3. Check the Omen teleport changes and the Steve rework, merge PR #39, and fix every bug found. Then optimize.
4. Modernise the editor: fewer dropdowns, a modern style.
5. Make all nine profile values affect every champion. Compare the stats with documented real games (the user suggested YouTube reviews).

## Main as found (332b4d4)

- **Health:** builds with 0 warnings, and all 517 tests pass. Every PR #38 fix survived.
- **Wave, camp and objective skill use:**
  - New with PRs #40–#42: Steve/Herobrine and Gojo v2.
  - Frieren, Coder, Vesper, Isliid, Scribble, Magnus and Unified already farmed through `farming.rs`.
  - DIO's knives and the data-driven kits farm through their "enemy except tower" skills.
  - **Omen could not farm at all.**
- **Per-player stats:**
  - The four game-sense ratings (Macro, Micro, Map Awareness, Strategy) reached every champion, but only through the team planner and `WallAi` move redirects.
  - The nine profile values (ego, judgement, positioning, control_speed, skill_hit, skill_avoid, aggressive, order, roaming) were read only by DIO's ultimate, Gojo and Omen.
  - Damage and stats were never affected.

## Bugs found and fixed

| Area | Bug | Fix |
|---|---|---|
| Team planner | It re-read strategy fields it had already overwritten, so its own last choice earned the coach's bonus and locked the first plan in (`universal_ai.rs`). | The coach's choices are recorded once per match on the nexus (`ua_coach0:`) and scored from there. Test with a mock host. |
| Frieren | Her basic attack skipped towers and the nexus, while `expected_damage` still claimed full damage. | It hits structures at the party's 80%. Test added. |
| Omen | The tooltip said a 50,000 teleport, but the code uses 75,000. | Text fixed in the game and in the editor copy. |
| Omen | Taunt, Bind and move-block didn't cancel his channels. | They do now. The API can't say who applied a Bind, so a Bind longer than his own 2-tick self-root counts as an enemy's. |
| Omen | CC immunity defeated his self-root, so he could drift mid-channel. | While immune, he is put back at the channel's start each tick. |
| Omen | Step and ultimate could land inside a Steve wall or an enemy danger zone. | Those destinations are rejected. |
| Omen | No farming. | Paranoia clears waves along a line, inside `farming::window`: 3+ lane units or one monster, never through an ally. The damage (40 + 60% AD) is new and **needs balance review**. |
| Profiles | A replay without a pin fell back to a version-1 default. Unbound profiles weren't cached, and the client process had an empty registry. | Pins are keyed by what the simulation knows (`match:set:seed:athlete`). The default is version 2. Clients load pins from the save and journal, and profiles are cached per set. |
| Saves | Pins and the journal grew forever, and participation entries were never pruned. An unreadable `game_sense_v1` entry was silently reset, the participation JSON was rewritten every tick, and the calendar date could be spoofed. | The newest 64 sets are kept and the journal is compacted. Participation is pruned after 14 days. Unreadable entries are backed up, JSON is written only on change, and implausible dates are ignored. |
| Farming | A jungler could open an Epic or Serpen alone. Waves were reserved before the collector arrived, kits took the nearest minion over the assigned objective, and Gojo and Steve skipped `farming::window`. | A jungler opens an untouched objective only with a job or judgement of 80+. A wave is reserved from 60,000 out. The assigned Contest/Finish objective sorts first. Gojo, Gojo v2 and Steve go through `Context::farmable()`. |
| Steve | The planner cloned the whole core per candidate, and `events` were cloned every tick. Cooldowns were rolled back by group only by coincidence. | Planning branches copy only what planning reads. The unread `events` copy is gone. Cooldowns use one `cooldown_slot()`. |
| Gojo and tactics labs | Both labs rejected every frame because the pages sent profile version 1 after the native side moved to 2. | The pages send version 2, and both browser tests pass again. |

**Checked and left alone:** a reviewer suspected `steve_craft_core::release` of reading the recipe from the cooldown slot. It reads `previous_cd.0`, which stores the recipe id itself, so it is correct.

## Optimization

`farming::Context` caches team facts once per kit snapshot. Measured on the paired benchmark: farming query p95 went from 45.1 to 37.9 µs, and host calls from 2,986 to 2,587. Steve's core step is within noise at 32/64/128 units; the benchmark never fills the fields the lighter copy skips.

Sharing one snapshot across kits within a tick was **rejected on purpose**. A minion killed by one kit earlier in the tick would still look eligible to the next, and job assignments are rewritten mid-tick.

## The champion roster (object model)

- **Data:** `rules/champions.json` describes all 77 champions (17 custom, 60 base game) as champion – family – skills – basic attack. `tools/generate-roster.py` writes three outputs from it:
  - `roster_data.rs`, the native table;
  - `editor/roster.js`, for the editor's **Champion Roster** page;
  - `docs/champions.md`.
- **Kinds:** Dash, Blink, Projectile, Line, Aoe, Zone, Summon, Heal, Shield, Cc, Buff, Construct.
- **Intents:** Clear, Engage, Escape, Protect, Objective, Poke, Vision.
- **Labels:** custom kits are labelled by hand (Unified lists all 18 equations). Base-game labels are derived from the game's own skill text, so treat them as a first pass.
- **Code:** `roster.rs` adds:
  - `def(name)` (reworked variants resolve to their champion);
  - `can(name, Intent)`, `skills_for(name, Intent)` and `champions_with(Kind)`;
  - `ChampionDef::line()`, which prints e.g. "Gojo – Jujutsu Kaisen – Blue [Projectile, Cc | Clear, Engage] – …";
  - the `Kit` trait, with `ready(sim, me, intent)` reading the slot's real cooldown and a generic implementation for every champion.

  Casting stays in each kit, because native kits own their actions and the engine owns base-game casts.
- **Families:** 14 families. The five cross-franchise rules that already existed are formalised:
  - Sandevistan outruns Infinity;
  - V1 parries any strike, teleport strikes included;
  - Steve's walls block every path;
  - Omen's smoke blocks sight;
  - ZA WARUDO stops everyone.

  No new rules were invented.
- **Already used:** tactics' map-changing champions now come from the roster (anyone with a Construct skill or a Vision zone), not a hard-coded list.

## All nine profile values, for every champion

`athlete_play.rs` sits in the per-player input layer, which sees the engine's next input for every player, base game and custom. It makes small, deterministic changes to that input. **Damage and stats are untouched, and an average athlete (50) plays exactly as before.**

| Value | What changes |
|---|---|
| control_speed | Below 50, a fresh cast waits up to 18 ticks (the profile's reaction time beyond an average athlete's). |
| skill_hit | A ground-aimed cast leads a moving enemy above 50, or trails it below 50. |
| skill_avoid | Below 50, the step out of a telegraphed zone is sometimes missed (50% of half-seconds at 0). |
| positioning | Above 55, a ranged champion steps back between its own shots. |
| judgement | Above 55, an ultimate waits up to 3 s until it catches two enemies or a low target. |
| ego | Below 50, a champion retreats earlier when hurt with an enemy close. |
| aggressive | Above 60, an enemy in reach is attacked. Below 40, nobody is chased out of range. |
| order | From 65, a champion waits for allies before attacking into two or more enemies. |
| roaming | Above 60, an idle champion joins a nearby ally's fight. |

Each value has a paired test (20 against 80) on three champions: a base-game ranged champion, a native kit and a dash champion. Kits that drive their own casts (Gojo, Omen, DIO, Unified…) keep reading these values inside their own controllers.

## Do the stats matter? Measured

**The model.** There is no whole-match engine here, so `athlete_stats_fixture.rs` plays a deterministic macro rollout:
- 5v5 for 20 minutes on a mirrored three-lane map, with waves, camps, an epic and a serpent;
- both teams use the **real team planner**;
- fights and idle moves go through `athlete_play::decide`;
- team A plays at 20, 50 or 80 in one value against an average team B, over 96 seeds with sides alternating.

`python3 tools/evaluate-game-sense.py` reruns it and writes `docs/round-139-results/athlete-stats.json`.

**Noise floor.** Two average teams split objectives 0.483 to 0.517, so differences under about 0.03 are noise.

| Value | 20 | 80 | Reading |
|---|---|---|---|
| Macro | obj 0.463, ahead 0.469 | obj 0.483, ahead 0.499 | Slightly better objectives and fights. |
| Micro | obj 0.490, ahead 0.490 | obj **0.430**, ahead 0.531 | Fights more and wins more fights, but **takes fewer objectives**. |
| Map Awareness | obj 0.502 | obj 0.485 | No measurable effect, as suspected last round. |
| Strategy | obj 0.434 | obj 0.447 | Within noise of each other, both below even. |
| ego | obj **0.380**, roams 19.6/10 min | obj 0.489 | Low ego retreats early and loses objectives. |
| aggressive | ahead 0.511 | obj 0.436, ahead 0.495 | High aggression takes fights and gives up objectives. |
| order | – | obj 0.421, ahead **0.607** | Waiting for allies wins fights and costs objectives. |
| roaming | – | obj 0.498, roams 30.6 | Small gain. |
| judgement | – | – | No macro effect: it acts on ultimates, which this model doesn't simulate. |
| control_speed, skill_hit, skill_avoid, positioning | – | – | Act inside a single exchange, finer than the one-second model. The unit tests show the behaviour. |

"obj" is team A's objective share and "ahead" is its share of fights started with more players.

**Answer.**
- **Before this round:** the four ratings changed the planner's choices, but outcomes moved less than about 5 points. Only three kits read the nine profile values.
- **Now:** ego, order and aggression clearly change how a team plays (−10 points of objectives, +15 points of fights with numbers).
- **Wrong way:** Micro at 80 is the only rating that points the wrong way beyond noise. It trades objectives for fights.

  I did not retune it. Without reference games, a single model is not enough to say which is right.

**Comparison with real games.**
- `www.youtube.com` is blocked by this environment's network policy (Network access → Allowed domains), and video can't be read as text anyway.
- Instead, `tools/reference-games/` defines a JSON format for a documented game's macro events: objectives, fights with numbers, roams, deaths to unseen enemies, towers. There is a template and a README.
- The evaluator reports, per metric, which level of each value plays most like the stronger side.
- **No games are filled in yet.** With timestamps from the suggested video (or access to YouTube, gol.gg or Leaguepedia), the calibration table fills itself.

## Editor

- **Theme:** `editor/theme.css` holds design tokens for light and dark (system default, with a remembered toggle), card surfaces and focus rings. Every page imports it, and game arenas stay dark in both themes.
- **Controls:** `editor/ui-kit.js` turns dropdowns into segmented controls, chip groups, rank chips and one searchable picker. The original `<select>` stays underneath, so scripts and tests keep working.
- **Dropdowns:** visible dropdowns across 23 pages and views went from **176 to 0**. The 238 selects now render as:
  - 57 segmented controls;
  - 70 chip groups;
  - 8 rank ladders;
  - 81 searchable pickers.
- **New page:** **Champion Roster** (`roster.html`) offers search, intent and kind filters ("who can Engage with a Dash": 22), and the family rules.
- **Fixed on the way:** the Skill Test crashed on a preset with no text, and the Game Sense page scrolled sideways on phones.

## PR #39

Main (through #42) was merged into `codex/objective-hp-stark-performance`:
- `set_server_extension` keeps only the last registration, so `servers.rs` composes the objective rules and the athlete profile server.
- Farming keeps both sides' rules.
- Stark's leap uses main's 80% damage scale.
- The Frieren regression baseline moved to main.

All verifiers passed; a comment on the PR has the details.

## Verification

- **Native:** `cargo test --release` 572 passed, 0 failed, 7 ignored; 0 warnings; the Windows DLL builds.
- **Generators:** `generate-roster.py --check` and `generate-unified-equations.py --check` are up to date.
- **Frieren:** native/WASM parity 25,920 frames; unchanged-with-leap-disabled 25,920 frames.
- **Others:**
  - Vesper parity: 0 mismatches;
  - Unified verifier: passes;
  - objective clears: 16 models pass.
- **Migration tests:** rework, objective HP and Magnus career all pass.
- **Manager:** 13 tests pass.
- **Browser:**
  - `verify-editor-theme.cjs` covers 23 pages and views, light and dark, at 390 px, with no page errors and no sideways scroll;
  - the Gojo lab, tactics lab, Unified lab and Coder checks all pass;
  - the roster page was checked in both themes.

## Still open

- **Live game checks:** none were possible here; every result above comes from tests, labs and the macro model.
- **Omen:** Paranoia's wave-clear damage needs balance review. A Bind of 2 ticks or less from an enemy won't cancel his channel.
- **Base-game roster labels** are heuristic; review `docs/champions.md` and edit `rules/champions.json`.
- **Micro at 80** costs objectives in the model. Decide with reference games before retuning.
- **Lab tests on Linux:** `test-steve-lab.cjs` and `test-universal-lab.cjs` still need Edge and a Windows `.exe` for their parity steps.
