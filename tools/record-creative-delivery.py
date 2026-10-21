"""Write review notes from measured results, without turning pending host checks into passes."""
from pathlib import Path
import json,statistics
root=Path(__file__).resolve().parents[1];out=root/'docs/creative-results'
read=lambda p:json.loads((root/p).read_text(encoding='utf8'))
package=read('docs/creative-results/package.json');perf=read('docs/creative-results/performance.json');checks=read('docs/creative-results/checks.json');steve=read('docs/steve-results/lab-tests.json');universal=read('docs/creative-results/universal-lab.json')
table='\n'.join(f"| {a['units']} units | {a['medianP95Ns']/1e6:.4f} ms | {b['medianP95Ns']/1e6:.4f} ms | {b['medianP99Ns']/1e6:.4f} ms |"for a,b in zip(perf['steveBaseline']['crowds'],perf['steveCandidate']['crowds']))
native=checks['checks'][0]['summaries'][0]
notes=f'''# Universal AI, balance and Creative Herobrine

Native **{package['native']}**, Blockcraft **0.1.22**, champion pack **0.2.63**, Frieren **0.1.20**, JoJo **0.1.19**, Valorant **0.1.14**. Based on main `0365335`, isolated branch `codex/balance-creative-herobrine`. Frieren 0.1.19 remains reserved for the separate objective/Stark PR #39; that PR is not merged into this change.

## Result

- Frieren's basic, spells, Fern and Stark raw damage use 80% once. Coder base HP/growth are 1000/108 and created shields use 75% once after support bonuses. Dio's normal/farming knives use 75%, rounded down before mitigation. Omen Step reaches 75000; both teleports root their owner during their existing channels and keep fixed destinations.
- Steve keeps his original base stats. His basic and recipes scale with accepted maximum HP, including the Epic/Serpen HP-component cap. Ultimate toggles persistent forms after 0.4s, with an 18s accepted-switch cooldown. Steve spends three of four independently recovering slots; Herobrine may spend all four. Switching does not heal or refill anything. Boat Wall is a separate three-charge, 80s signature.
- All 45 catalogue entries are available in the lab. Steve retains Apple, Shield Guard, repair and rod rescue. Herobrine has 43 selectable entries, empty hands, the approved white-eye overlay, reduced defences and 30 Creative choices. Helpful Creative protection excludes its original caster. Same-name periodic effects select the strongest contribution, including overlapping sources from the same owner.
- Redstone has paid single-use payloads, bounded acyclic links and delayed signals. Destruction cancels downstream activation; editing/rotation preserves object identity, health, lifetime and paid ammunition. Death clears owned constructions without cleanup explosions. Granted shields finish normally.
- Macro, Micro, Map Awareness and Strategy are saved in the mod namespace by career/athlete. Initial offsets use a separate identity hash. Development is idempotent by day/set and capped at four qualifying sets/day; only server processing writes career progress. Profiles v2 pin current ratings/stats per match/set. V1 profiles decode with neutral additions and retain their old decision arithmetic.
- Both teams receive bounded team intentions, a Strategy/Judgement caller, explicit jobs, wave reservations and conservative public-terrain arrival estimates. The planner compares waves, assigned camps, objective preparation/contests/stalls/withdrawal, tower trades, visible teammate reinforcement and team resets. It reads visible HP/trends and timestamped sightings; hidden enemies add uncertainty. It writes only recognized supported strategy fields, respects later coach edits, and changes only ordinary movement inputs. Stock attacks and casts remain engine-owned.

## Approved artwork

The rejected long-arm proposal is not integrated. The approved edit retains Steve's original short arm/hand pixels and removes the pickaxe. The original armed atlas remains byte-identical for legacy definitions. New definitions use a separate unarmed base plus an extracted original weapon layer in Steve form; Herobrine uses the existing white-eye layer. Composition reconstructs every original armed frame exactly. All 31 frames, both facings, original timing and 36x44 canvas are checked. Original skill-effect PNG bytes and animation metadata are preserved; the fourth pip copies existing pip artwork. Decoded Blockcraft art is **3,547,136 bytes**, below 32 MiB; the added atlas is 512x220.

![Approved original arms](creative-herobrine-review/weapon-removal-comparison.png)

## Automated evidence

- Native release: **{native}**. Four pre-existing ignored tests remain explicitly ignored; benchmark suites run separately.
- Steve shared core: 46 tests, including every new entry, form/charge restrictions, snapshots/caps, redstone payment, overlap deduplication, capture/cleanup and rotation. Frieren shared lab: 16 tests. Gojo shared lab: 81 tests; kit mechanics remain unchanged.
- Steve: **{steve['parityFrames']} complete native/WASM frames**, 24 development + 12 holdout scenarios; every one of the 45 recipes accepts/resolves/replays. The frozen original v1 WASM matches the current v1 reader and native vector runner for 140 frames.
- Team planner: **{universal['parityFrames']} native/WASM frames** over 72 scenarios: 24 development + 12 holdout seeds on each team. Fixtures show different actual choices/assignments when changing each new attribute independently, plus daily/result deduplication and independent sessions.
- Nine backed-up migration tests; 36 canonical definitions reproduce without changes. Both-facing art, legacy effects, atlas limits and approved body composition pass.
- Catalogue: 60 stock + 17 custom champions, 231 slots and 11,550 metadata/team/role/target combinations. This is **catalogue/policy coverage, not 11,550 observed casts**. See [the coverage matrix](universal-farming-coverage.md) and [stage explanations](creative-results/farming-catalogue.json).
- Isolated Mod Manager package fixture: **717 files compared, 0 differ, 0 missing; native DLL identical**. The installed game was not used as this fixture.

## Five-run performance

Matched Steve scenes use the frozen original core and the candidate with the same positions, unit counts and 1800 ticks. Authorized rework/decision changes remain enabled. Each value below is the median of five p95/p99 measurements; these small changes are not presented as an optimization win.

| Scene | Original p95 | Candidate p95 | Candidate p99 |
|---|---:|---:|---:|
{table}

Magnus 128-piece release-core p95 across five runs: **{', '.join(f"{r['stress'][-1]['p95StepUs']/1000:.3f}"for r in perf['magnus'])} ms**, all below 4 ms. New team planning with ten actors and 32 targets, including public-terrain routing: p95 **{min(r['p95Ns']for r in perf['universal']['runs'])/1e6:.3f}–{max(r['p95Ns']for r in perf['universal']['runs'])/1e6:.3f} ms** per evaluation. Cold and sustained samples are separate in [performance.json](creative-results/performance.json). Native host calls and rendering are excluded. No freeze fix or real-match clear-time improvement is claimed.

## Compatibility, migration and controls

- F8 opens the in-game athlete panel; F9 selects the next athlete, with the player's roster first; F10 changes an owned athlete's development focus. The editor's **Athlete Game Sense** lab exposes ratings, assignments, reasons and practice development. Practice buttons do not award career progress.
- Run `rtk proxy python -X utf8 tools/migrate-champion-reworks.py PATH` to inspect an existing career/export. Inspection does not write. `--output NEW_PATH` creates a separate migrated file; `--in-place` requires and verifies a complete backup before atomic replacement. The update preserves career records, history and items. Coder's customized embedded HP/growth are multiplied by 80% exactly once using a version marker; other champion stats are preserved. Steve visual bindings/sprite and Frieren's attributed basic are refreshed explicitly.
- Older Steve definitions retain their original armed base asset. Frozen v1 lab/recording code remains separate from the new runtime. Transfers preserve athlete identity and development. Missing recorded profiles are reported and use a neutral fallback.
- Optional `ai_diagnostics.enabled` records eligibility, selected engine inputs and subsequent cooldown-consumption evidence separately; HP changes are labelled **unattributed**, never proof of delivery. Optional `perf.flag` adds bounded simulation/origin/owner-stage timings. Neither observer injects casts or consumes combat randomness.

## Pending live acceptance and practical limits

No installation or real-career migration was performed. Live checks remain pending: stock skill selection/accepted casts/delivered contribution across roles and targets; native carrier targeting, item/kill attribution, animations and crowded fights; actual Omen channel immobility; in-game panel and training attribution; accepted strategy enum shapes and coach transitions.

The SDK provides no general wrapper around all stock actions. Strategy/movement advice cannot guarantee a stock skill will be cast. Native cast fixtures and broad target filters are not substituted for that evidence. The catalogue marks selection, execution and delivery as pending where they have not been observed.

The host exposes no explicit training-completed event. The management bridge uses a positive server-observed change in the athlete's `training_exp` (or its rollover with a stat gain), an authenticated calendar date, and excludes callbacks carrying MatchFinished. A date message alone cannot award progress. This integration is conservative and needs live confirmation; missed/unavailable evidence grants nothing. Objective values are prototype utility weights; routes and future contribution remain estimates rather than hidden-information forecasts.

The separate objective HP/Stark leap work stays in PR #39. This release scales current-main Stark damage; it does not claim that unmerged leap restoration is present.

## Reproduction and package identity

Run `rtk proxy python -X utf8 tools/deliver-creative-checks.py` with the local lab server on port 7284. The pipeline builds native/WASM labs, runs tests and browser parity, repeats five-run benchmarks, builds the DLL in two separate target directories and checks an isolated package with Mod Manager. It does not install or migrate a career. Canonical generation is checked separately by `tools/verify-creative-generation.py`.

Toolchain: {package['toolchain']}. Two builds and the packaged DLL have SHA-256:

`{package['packagedDllSha256']}`

Native source/Cargo fingerprint: `{package['nativeSourcesSha256']}`. Complete hashes and actual command outputs are in [creative-results](creative-results/package.json).
'''
(root/'docs/universal-ai-creative-implementation.md').write_text(notes,encoding='utf8')
body=f'''Steve now switches between HP Tank Survival play and persistent Creative Herobrine, with 45 paid recipes, four recovering charge slots, repair, owner-scoped redstone and editable construction. This also adds saved athlete game-sense development, shared team intentions/farming context for both teams, and the requested Frieren/Coder/Dio nerfs and rooted 75000-range Omen Step.

### Changes
- Preserve Steve's original short arms/hands. Herobrine removes the pickaxe and reuses the existing white eyes; original weapon pixels restore armed Steve exactly. Legacy body/effect pixels and timings remain intact.
- Native-authoritative resources, HP snapshots/Epic caps, 43 Herobrine choices, caster exclusions, strongest same-name effects, single-paid payloads, death cleanup and a separate straight/curved Boat Wall.
- Persistent Macro/Micro/Map Awareness/Strategy, bounded development, pinned v2 profiles with v1 readers, an in-game F8 panel and editor lab. No combat-stat bonuses.
- Bounded visible-information team plans, public-terrain ETA, jobs, wave reservations, reinforcement/reset decisions and supported strategy/movement advice. Stock attacks/casts remain engine-owned.
- Explicit backed-up migration defaults to inspection. Native {package['native']}, Blockcraft 0.1.22, custom 0.2.63, Frieren 0.1.20, JoJo 0.1.19 and Valorant 0.1.14. PR #39's objective/Stark changes remain separate.

### Automated checks
- **517 native release tests pass**, zero failures, four existing ignored tests. Steve/Frieren/Gojo shared lab suites pass; all 45 recipes resolve in browser fixtures.
- **{steve['parityFrames']} Steve native/WASM frames**, 24 development + 12 holdout scenarios; 140 original v1 replay frames match the frozen WASM and current native reader.
- **{universal['parityFrames']} planner parity frames**, 72 scenarios across both teams; independent decision changes for all four attributes, plus training/result deduplication.
- Nine migration tests; reproducible canonical definitions; approved 31-frame body composition, mirrors, original effects and atlas limits pass.
- Catalogue covers 77 champions / 231 slots / 11,550 metadata combinations. These are not claimed as observed casts.
- Five matched benchmark runs: Steve 128-unit median p95 {perf['steveBaseline']['crowds'][-1]['medianP95Ns']/1e6:.4f} → {perf['steveCandidate']['crowds'][-1]['medianP95Ns']/1e6:.4f} ms. Magnus 128-piece p95 stays below 4 ms in all runs. Host/rendering performance remains pending.
- Two separate release builds produce the same DLL. Isolated Mod Manager: **717 files, 0 differ, 0 missing; DLL identical**.

DLL SHA-256: `{package['packagedDllSha256']}`

### Review and live gate
[Implementation, measurements, migration and limitations](docs/universal-ai-creative-implementation.md) · [REA evidence](docs/creative-results/rea-evidence.md) · [Coverage matrix](docs/universal-farming-coverage.md) · [Package hashes](docs/creative-results/package.json)

**Draft — live acceptance remains pending.** This checkout is packaged only; nothing was installed and no real career was migrated. The SDK cannot force every stock cast through a supported general wrapper. Actual stock casts/contribution, native targeting/items, Omen movement locks, panel/training attribution and crowded host performance need game observation. Training uses observed training-exp changes because no completion event is exposed; a date alone never awards progress.

![Approved original arms](https://raw.githubusercontent.com/PowerViber/tfm2_mods/codex/balance-creative-herobrine/docs/creative-herobrine-review/weapon-removal-comparison.png)
![Animated original-arm preview](https://raw.githubusercontent.com/PowerViber/tfm2_mods/codex/balance-creative-herobrine/docs/creative-herobrine-review/weapon-removal-animation.gif)
![Creative Lab](https://raw.githubusercontent.com/PowerViber/tfm2_mods/codex/balance-creative-herobrine/docs/steve-results/lab-curved-wall.png)
![Game Sense Lab](https://raw.githubusercontent.com/PowerViber/tfm2_mods/codex/balance-creative-herobrine/docs/creative-results/universal-lab.png)
'''
body=body.replace('](docs/','](https://github.com/PowerViber/tfm2_mods/blob/codex/balance-creative-herobrine/docs/')
(out/'pr-body.md').write_text(body,encoding='utf8')
print('Recorded measured results and pending live gates.')
