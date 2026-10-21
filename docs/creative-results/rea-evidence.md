# Research and compatibility evidence

## Shipped artifact inspection

The REA skill was used for read-only provider discovery and a native inspection attempt. REA 6.7.0 had no available configured native analysis engine in this session; its objdump route rejected this Windows-native target. No decompilation or runtime training trace is claimed. Supported SDK source and bounded raw string inspection supplied the compatibility evidence below. No executable or real save was modified.

| Evidence | Observation | Limit |
|---|---|---|
| Installed `TeamfightManager2.exe` | PE64, 87,228,416 bytes; SHA-256 `a1d05045405a12e2bed182daeafcad8c8a40cf29ab68ba6ea5bfd30257439582` | Identifies the inspected artifact, not every future game build |
| Training serialization strings | `training_exp` near file offset 61,986,062; Athlete/TrainingExp field-name strings | Establishes a serialized training resource, not a completion callback or its runtime ordering |
| Strategy enum strings | Around file offset 64,363,120: ObjectBuildup Gather/Split, ObjectBattle Poking/Initiating, ObjectFinish KillPriority/BattlePriority, MinionWave WavePriority/JoinPriority | Runtime JSON acceptance remains to be observed; the adapter writes only when the current field already has a recognized string value |
| Read-only save container inspection | SHA-256 `93acbb6c0ec8f45afbdeeef2ba9b27e12500881c3653baca6c01ca77cae52aca`; gzip/container handling inspected | No personal records are published and no real-career migration was applied |
| SDK event surface | MatchFinished, TransferCompleted and SeasonRollover; no training-completed event | Training integration uses positive training-resource changes, skips result callbacks and needs live attribution validation |

The supported [Stable API reference](https://raw.githubusercontent.com/teamsamoyed/TeamfightManager2Mod/main/docs/stable-api-reference.md) documents `EntityKindV1`, management record reads, namespaced save data, authenticated commands, strategy JSON mutation and input hooks. This implementation uses input replacement only for movement, preserving the repository's restriction against injecting or holding stock casts. The API provides no general wrapper around every stock action; those attacks/casts retain engine ownership.

The same reference documents UI-map coordinates, font assets and read-only render hooks. The athlete panel uses the supported regular font, update-hook commands and render-hook drawing. Its actual appearance and controls remain a live acceptance item.

## Transcript lessons and TFM2 adaptations

These are the user-approved transcript analyses from planning, not a claim of a new full-video review. Observed analytical principles are separated from the mod's prototype decision rules.

| Reference and timestamp | Reviewed principle | TFM2 adaptation and fixture |
|---|---|---|
| [Polypuff, T1–Gen.G](https://www.youtube.com/watch?v=-0Adk2Al7LA), 3:03 | Manage waves before rotations | A single assigned wave collector; Macro prices travel/opportunity cost; `wave`, `prepare` |
| Same, 10:21 | Stalling and engaging have different requirements | Compare visible objective HP loss, arrivals, control readiness and survival; `stall`, `finish`, `lost` |
| Same, 12:55 | Objective damage and zoning are separate jobs | Damage/frontline/zone/protect/flank assignments; objective fixtures on both teams |
| Same, 16:23 | Abandon a deteriorating attempt | Withdraw before comparing a safe tower/wave trade; `lost`, `blocked`, `conversion` |
| [Caedrel, Gen.G analysis](https://www.youtube.com/watch?v=ncnSX6ugCgQ), 2:11–3:41 | Losing a contest can justify a map trade | Compare visible tower pressure with an objective that will finish before arrival; `lost`, `conversion` |
| Same, 6:43–8:22 | Mobility, ultimate economy and follow-up matter | Preserve native Omen teleport decisions and control-tool reserves; compare allied ultimate readiness; remote tactical fixtures |
| [Riot/SKT–EDG breakdown](https://www.youtube.com/watch?v=oa7KbSfAajg), 1:33–2:52 | Prepare the map before committing | Public terrain routes, wave preparation and uncertain sightings; `prepare`, `blocked`, `fog` |
| Same, 3:23–5:25 | Coordinate control and participants | Caller selection, jobs, visible teammate reinforcement and existing protected-guest staging; `reinforce`, `stall` |
| Same, 6:15–7:28 | Fight value depends on useful aftermath | Tower/objective conversion or a team reset; `conversion`, `reset` |
| [Dignitas, wave priority](https://dignitas.gg/articles/playing-with-priority-the-advantages-of-pushing-your-wave) and [teamfighting](https://dignitas.gg/articles/5-tips-to-improve-your-teamfighting-ft-team-dignitas) | Price missed waves, readiness, protection and target selection | Macro/Strategy comparisons, resource-respecting native presets and explicit team jobs |

The implementation reads TFM2 entity kinds, visible HP/trends, positions, movement speeds, public terrain and available allied ultimate flags. Objective/wave utility weights are prototype preferences; they are not imported League of Legends rewards or claims of exact future damage/gold. Missing route/mobility information remains an estimate. Hidden positions and cooldowns do not become observations when an athlete's Awareness increases.

## Evidence stages

1. **Eligibility:** target kind, role, visibility, survival context and current assignment.
2. **Selection:** the native selected continuation, or an observed engine input.
3. **Execution:** native/lab command fixtures, or later engine cooldown consumption as separately labelled evidence.
4. **Delivery:** attributed combat/support observed in an actual match. An input, target filter or unattributed HP delta does not establish this stage.

The current coverage matrix leaves unobserved engine selection, execution and delivery pending. Automated core/ABI/browser evidence is recorded separately from native game acceptance.
