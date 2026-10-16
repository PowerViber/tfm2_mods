# Magnus practical planning and reliable farming

17 October 2026 (Asia/Jakarta). Branch `codex/magnus-grandmaster-farming`, based on `5840778` (PR #35). Native **0.10.54**, champion pack **0.2.55**, dependency **>=0.10.54**, Chess Lab transport **4**.

This report supersedes the automated results in [the previous release report](magnus-verification.md). Slim A artwork, ranked appearances, compact Alchemist pieces, the 32,000-unit grid and the twelve historical adaptations are retained. No damage, HP, stun duration or draft-score bonus was added.

## Chess principles translated into the planner

These are bounded combat heuristics inspired by expert instruction, not a Chess.com engine integration or a measured human Elo rating.

| Primary reference | Combat adaptation |
| --- | --- |
| GM Bryan Smith, [Forcing Moves, Part 1](https://www.chess.com/article/view/forcing-moves-part-1) | Retain immediate attack candidates alongside checking, discovered-line and escape-covering moves. |
| IM Jeremy Silman, [Test Your Chess Understanding, Part 3](https://www.chess.com/article/view/test-your-chess-understanding-part-3) | Discount a proposed mating net if the enemy can reach a safer square or remove a vulnerable covering piece. |
| IM Jeremy Silman, [How to Evaluate a Position](https://www.chess.com/article/view/how-to-evaluate-a-position) | Preserve king safety, useful coverage and material; take practical damage or farm when a credible mating continuation is unavailable. |

The search keeps the existing one-to-four-command mastery horizon, sixteen rotating tactical pieces, sixty-four evaluated candidates and beam width eight. Each child inherits the complete preceding branch. Visible enemy motion is extrapolated through command completion, bounded to two seconds and stopped by blocked sight lines. Local counterplay is a conservative estimate, not a full simulation of every enemy ability. Terrain geometry and motion forecasts are cached only within one search.

Partial check coverage has less weight than a useful attack; completed viable mates remain valuable. Contributing pieces hold still during a valid trap warning. With no nearby champion, Magnus can target visible waves and camps. With no target, he does not spend his clock rearranging an empty field. Pawn candidates now include every legal cardinal step rather than the obsolete forward/double-step list.

## Damage and low draft activity: findings

- **Confirmed:** older embedded career definitions can lack Magnus's ultimate-learned passive, so the board never unlocks even when the champion's data pack has been updated. The migration tool adds only the missing passive. Balance patches, stats and unrelated career data are preserved.
- **Confirmed in the old planner:** stationary target assumptions and excessive partial-net value could favour repositioning over attacks. The revised search forecasts visible motion, retains damage alternatives and supports farming.
- **Confirmed measurement fault:** an immediate HP read after `deal_damage` can be zero because the shipped engine dispatches damage through a deferred path. Later uncorrelated HP loss is not an exact hit receipt. See [native evidence and confidence](magnus-native-evidence.md).
- **Observed:** the game's draft evaluation considered Magnus available, evaluating candidate 82 on 119 of the first 4,096 logged score calls, with scores from 0.42932838 to 0.74107397. This establishes eligibility, not a population pick rate or the cause of low usage. No draft score was changed.

New champions start with a flat lane profile unless a lane prior is supplied; existing match history takes over. That is a possible contributor to early draft behaviour, not a demonstrated diagnosis for this career. No career training weights were reset. [Official champion/draft API](https://raw.githubusercontent.com/teamsamoyed/TeamfightManager2Mod/main/docs/stable-api-reference.md)

Native damage still uses the engine's supported `deal_damage` pipeline, preserving mitigation, statistics, items and kill attribution. There is no raw-HP or true-damage workaround. [Official damage API](https://raw.githubusercontent.com/teamsamoyed/TeamfightManager2Mod/main/docs/stable-api-reference.md)

## Farming behaviour

The shared safe window requires at least 35% HP, no stun/push, and no visible enemy champion within 110,000 units. Candidate creeps must be living, visible, targetable enemies; towers are excluded. Selection is deterministic and bounded to 32 targets. Both teams use the same policy. New decisions yield to danger; committed effects finish normally.

| Kit | Change or audited existing behaviour |
| --- | --- |
| Magnus | Shared planner targets nearby waves/camps when no champion needs attention. |
| Scribble | Ordinary offensive recipes can target creeps; large recipes and defensive tools are reserved. |
| Coder | Ping, spray, chain and firewall can clear creeps; major scripts, escapes and copilot activation remain reserved. |
| Vesper | Ordinary attacks/techniques can target creeps. Relocation, Kidnap and major gauntlet moves are reserved; creep deaths cannot grant champion research credit. |
| Unified Theory | Direct ordinary recipes use explicit creep identities; preparation chains and expensive combinations are reserved. |
| Aegis Zero | Skill 1 accepts non-tower enemies and completes marked creep casts; skill 2 and ultimate are unchanged. |
| Isliid | An available, unempowered Bloodletter sword can make an ordinary creep-clearing stroke. It cannot steal an escort, formation or committed stroke. |
| Minato, DIO, David, V1, Vader, Frieren, Steve, Gojo | Existing broad offensive target filters and farming buff use the shared safer window. Engine-selected creep casts were observed. |
| Levi | Existing creep sweep/flight logic retained; engine-selected creep casts were observed. |
| Omen | Existing utility and escape behaviour retained. No artificial damaging farm spell was invented for a utility slot. |

Stock champions already emit skill inputs against creeps on both teams. The diagnostic observer records those inputs and returns no replacement order. It does not force or hold stock casts. Input observations do not prove that every cast hit. Native-only custom attacks are covered by their own adapters and tests.

Optional diagnostics require `mods/tfm2_custom_ai/ai_diagnostics.enabled` at startup. Draft and farming observations each have a process-wide 4,096-line limit; farming also has per-player limits and rate limiting. They change no gameplay decisions. The temporary investigation flag was removed after testing.

## Career compatibility tool

`tools/migrate-magnus-career.py SAVE` inspects without writing. `--output NEW_FILE` writes a separate validated container; `--in-place` creates and verifies a byte-for-byte backup before atomic replacement. Conflicting passives abort writing. `--farming` additionally changes only Aegis skill 1's target filter. Repeated migration is byte-identical.

Only a separate QA save was migrated and used for composition testing. The main career was never rewritten; its SHA-256 remained `93acbb6c0ec8f45afbdeeef2ba9b27e12500881c3653baca6c01ca77cae52aca`. Save files and raw logs are excluded from this PR. The original checkout's uncommitted DLL and art experiments remain untouched.

## Automated verification

- Native release suite: **365 passed, 0 failed, 3 existing diagnostic tests ignored**. After the final diagnostic-only line cap, all six farming-focused tests passed again.
- Career migration: **5 passed** (checksum validation, all embedded copies, idempotence, conflicting definitions and balance preservation).
- Native/editor parity: **9,638 complete-state frames**, including independent concurrent instances, all mastery ranks, moving targets, ultimate availability and twelve scripted historical successes. Historical near misses and duplicate/unconfirmed receipt tests remain enabled.
- JavaScript syntax checks and `git diff --check` passed. Browser fixture QA found and corrected the stale 14-field decoder; version 4 uses 17 fighter fields and rejects version 3 frames.
- Chess Lab reports predicted attack value separately from actual damage. Receipts explicitly distinguish applied, rejected, pending and unknown. Only a positive, correlated applied receipt can qualify a pawn or a historical damaging step.

### Controlled scenarios

| Set | Seeds | Mean living fraction | Checkmates | Autonomous historical completions | Raw requested piece damage |
| --- | ---: | ---: | ---: | ---: | ---: |
| Development | 24 | 0.726 | 47 | 0 | 76,715 |
| Holdout | 12 | 0.744 | 26 | 0 | 37,847 |

Full measurements: [scenarios](magnus-scenarios.json). These are synthetic combat scenarios, not balance win rates or confirmed host damage. The moving-target model changed, so the old release is not a controlled performance baseline. Autonomous historical completion remains an open tuning issue despite successful scripted fixtures.

### Release core stress on the development machine

| Living army | p95 core step | p95 planning decision | Slowest core step |
| ---: | ---: | ---: | ---: |
| 32 | 1.245 ms | 1.967 ms | 4.031 ms |
| 64 | 0.982 ms | 1.074 ms | 4.135 ms |
| 128 | 2.517 ms | 3.456 ms | 5.489 ms |

The 128-piece p95 core step meets the under-4-ms target. [Raw stress results](magnus-stress.json). These timings exclude the native host and renderer; they do not establish crowded-match FPS or a Vesper freeze fix.

## Actual game observations — partial acceptance

Game **0.6.4** loaded native **0.10.54**. A migrated QA career completed a 5v5 composition test: Aegis / Isliid / Magnus / Coder / Vesper versus Fighter / Berserker / Gambler / Archer / Frieren. Blue won **17–1**. Magnus recorded **3,988 champion damage** and **0/0/9**. This proves real attributed output in that match; it does not prove his balance or draft rate is fixed.

Replay observations at approximately 0:42, 2:02 and 3:06 showed compact pieces, PLAN displays and ongoing combat. The bounded game-log inspection found no panic, divergence or disabled-mod report. Stock and existing custom skill-to-creep inputs appeared on both teams. The main career hash remained unchanged.

These observations used intermediate DLL SHA-256 `8c4bd7f9dd1c998ae608cec49964dc0fd700b0a1a9b50492925f850f841f4065`. The final package additionally contains Isliid's ordinary farming stroke and the global diagnostic cap. Those final edits passed automated checks but were not observed in another live match.

### Remaining acceptance gates

1. The public host API still supplies no command-scoped post-damage receipt. Queued requests are **pending**, and later HP/shield changes are **unknown**, never fabricated historical/promotion credit. Native historical damaging sequences and pawn qualification remain unconfirmed until an exact supported receipt is available.
2. A full live check of all seven distinct pieces, targeted destruction, board stun/immunity, historical rewards, pawn promotion, item interactions and two-Magnus ownership remains outstanding.
3. Pick-rate improvement requires multiple controlled drafts/matches. One available candidate and one damage result do not establish it.
4. Crowded Magnus/Vesper host timings, cold versus warm effects and final-build replay acceptance remain unmeasured. No claim that the reported freezes are fixed.

Keep the PR as a draft while these acceptance gates remain open.

## Package and installation

Built, packaged and installed DLL bytes are identical:

`4d805f242371e3467ace24f1e74dcf23f21ba7b2c83421f171c86bd4e7c5b2e2`

Chess Lab WASM SHA-256:

`a36798015519544591080cb556b45135bb2d77839f562756ced06e97341b1307`

Installation occurred only after the game closed, with backups. Fourteen existing science animation differences were CRLF-only; exact normalized content was checked and the installed files backed up before synchronization. No decoded pixels, timings or repo artwork changed in this task.

The real Mod Manager check and isolated package fixture both passed: **773 champion files, zero differences, zero missing, native DLL identical**. [Installed check](magnus-farming-manager.txt) · [isolated fixture](magnus-manager-package.json) · [package metadata](magnus-package.json). The final installed build has not yet been launched, as the manager report explicitly states.
