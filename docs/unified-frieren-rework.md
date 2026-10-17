# Unified and Frieren rework — verification record

This candidate changes how Unified assembles and delivers experiments and replaces
Frieren's old Fern/Stark skill cycle with mana, Spellcraft, Defensive Magic and
Journey's Companions. Magnus keeps his numerical balance and receives the shared
role-aware farming policy. Display names become **Unified** and **Magnus**;
champion IDs, mastery records, base stats, growth and ranked body artwork remain.

**Status: reviewable draft; real-match acceptance remains pending.** No candidate
files were installed into the Steam game and no career was migrated during this
verification. Steam and direct launch attempts did not expose a usable game
window. The package fixture check below is not observed gameplay acceptance.

## Release and isolation

| Component | Baseline | Candidate |
|---|---:|---:|
| Native rules | 0.10.54 | 0.10.55 |
| Custom champion pack | 0.2.55 | 0.2.56 |
| Frieren | 0.1.16 | 0.1.17 |

Base: `920f80b346cb892b83564971499698cb8e7d60dc` (main, PR #36).
Branch: `codex/unified-frieren-magnus-roles`, in a separate managed worktree.
The original checkout's uncommitted DLL and artwork were preserved. The original
career save's SHA-256 was checked before and after; it was unchanged. Raw career
files and game logs are not included in this PR.

The vendored official stable SDK advances to ABI 10 for entity kinds, unlock
queries and unit-kill observations. Its existing local simulation-identity helper
remains diagnostic only. Native metadata matches the packaged metadata, and both
data packs require native `>=0.10.55`.

## Implemented behaviour

### Unified

- All **75 spell identities and 50 authored template identities** are retained.
  Their original catalogue definitions remain unchanged.
- Experiment contexts own target bindings, preparation, instruments, packets,
  fields and remaining budgets. Scientist and target switches restore the
  appropriate context. World effects use stable owner-local identities.
- Compatible owned lenses redirect once with finite travel; split payloads are
  conserved. Time fields shorten reaction delivery without multiplying payload.
  A marked enemy's observed displacement can release its remaining queued payload
  at its new visible position, once.
- Grand Unification requires ultimate unlock and at least two sciences. Commitment
  consumes linked producers, starts a 60-second cooldown and warns for 0.6 seconds.
  It redistributes remaining physical/magic payload within 24,000 units; all three
  sciences use 32,000 units and a bounded 0.6-second bind. Death/interruption cannot
  leave a second damaging copy. The AI does not intentionally use it for farming.
- Explicit overcharge reserves an extra charge, adds 0.5 seconds of preparation
  and funds at most 125% ordinary payload including allocation. Committed
  disruption causes 4% maximum-HP nonlethal backlash. Ordinary formula mistakes
  can fizzle or waste committed resources without causing self-damage.
- A bounded chooser enumerates legal catalogue choices and reviews 8–128 branches
  by rank, up to five stages. It considers delivery time, exposure, resource
  recovery, defence and existing preparation. Rank changes accuracy/review and
  prediction, without changing base damage, speed, charge recovery or control.
- Career-owned profiles persist six personal styles and two distinct authored
  signatures per athlete. Match-start copies pin preferences for simulation and
  replay. Bias is capped at 10% within a tactical priority; manual practice bypasses
  it. Science Lab exposes styles, signatures, actual submitted/selected lines,
  cancellation, overcharge and the warned ultimate.

### Frieren

- Zoltraak basic: `20 + 45% AP`, 1.2-second interval, original range retained.
- Mana: 100, integer recovery at 8/second while alive, 12/second during revealed
  mana. Dead time does not regenerate mana on the next live tick.
- Spellcraft: quick `50 + 70% AP`, 0.35-second preparation/15 mana; charged
  `85 + 100% AP`, 1-second preparation/35 mana. Shared four-second cooldown and
  mana are paid when preparation begins; interruption cancels the shot.
- Defensive Magic: breakable `90 + 45% AP` shield for 1.5 seconds within 60,000,
  30 mana/10-second cooldown. AI can protect an endangered ally.
- Journey's Companions: learned ultimate, 60-second cooldown, eight-second existing
  empowerment. Normal skills remain available. Fern fires every three seconds;
  one targetable Stark per owner guards Frieren or the currently protected ally
  for twelve seconds, using actual HP and a 90,000-unit pursuit leash.
- Host damage is attributed to Frieren. Stark's autonomous attacks are suppressed;
  failed carrier spawning creates no invisible replacement damage source.
- A single deterministic Rust core runs both native rules and Frieren Lab.
  The lab supports manual/AI casting, mana, fog, walls, moving targets, incoming
  attacks, spawn failures, interruption, death and single stepping.
- Existing bodies/rank art remain. The generated HUD/ray atlas adds **224,256
  decoded bytes**, 256×219 pixels, with 29 compact layers.

### Farming and Magnus

The native farming helper accepts visible targetable lane minions/summons for all
roles, ordinary camps only for assigned junglers, and major-objective assistance
for non-junglers only after observed objective HP loss with a nearby living ally.
Unknown neutral entity kinds fail closed. Low HP (below 35%), control or a visible
enemy champion within 110,000 suppresses new farming decisions. An untargetable
visible champion still counts as danger.

Magnus carries farm eligibility through Chess Lab transport v4 and selects a real
wave target when no champion target exists. This adds no damage, health or command
speed. Stock AI input is observed for diagnostics, never injected or held by this
change. Existing champion behaviour still requires live evaluation before balance
buffs or claims about pick rate and win rate.

## Automated verification

| Check | Observed result |
|---|---|
| Native release suite | **391 passed, 0 failed, 3 existing ignored** |
| Stable SDK | 2 tests passed; 2 documentation examples ignored |
| Frieren full native/WASM parity | **25,920 state-and-effect frames**, 24 development + 12 holdout scenarios |
| Unified native/JS handler parity | **150 traces**, commit plus 24-tick world state for every one of 75 spells |
| Unified authored templates | All 50 accepted/completed in prepared fixtures, plus interaction/conservation tests |
| Magnus full native/editor parity | **9,819 frames**, independent concurrent instances |
| Magnus scenarios | 24 development + 12 holdout scenarios; existing complete historical trajectories retained |
| Browser labs | Casting/damage/mana, ultimate, interruption/death; six Unified styles, overcharge and retained artwork load |
| Migration | 2 tests passed; inspection, backup path, idempotence, conflict/checksum rejection and unrelated record preservation |
| Data/art | Deterministic regeneration, valid atlas bounds, unchanged base stats/growth/IDs and unchanged Magnus champion data |
| Packaging | Two release target directories produced byte-identical DLLs; packaged DLL matched |
| Mod Manager | Exit 0; **773 files compared, 0 differ, 0 missing**, native DLL identical, isolated fixture |

The native suite includes independent owners, manual rank/style invariance,
visibility/cancellation, charge recovery, overcharge disruption, payload
conservation, companion spawn failure/death and role/safety boundaries. Unified's
150 handler traces do **not** establish complete native/editor planner-state parity.

Latest Magnus stress run (release core on the development Windows machine):

| Living pieces | Core step p95 | Planning p95 | Maximum step |
|---:|---:|---:|---:|
| 32 | 1.373 ms | 2.191 ms | 3.822 ms |
| 64 | 1.075 ms | 1.136 ms | 4.495 ms |
| 128 | **2.925 ms** | 3.904 ms | 7.298 ms |

The under-4-ms **p95 core-step** target passes. Host calls, rendering and crowded
real-match frame times are excluded. These measurements do not establish a
maximum-frame-time guarantee. Scenario reports record raw requested damage,
piece survival, command variety and trap outcomes separately. Their unscripted
historical completion count is zero; scripted historical success fixtures are a
different check.

## Controlled comparison and limits

The baseline and candidate use identical 60-second mock-host fixtures, seed,
positions, 400 AP/300 attack and a stationary 1,000-HP target. There is no enemy AI,
mitigation, item processing or career draft. The baseline has its former automatic
resonance; the candidate requires the learned ultimate, which is locked in this
comparison. Numbers are **requested raw spell damage**, not engine-attributed
damage or win rates.

| Target distance | Baseline Student | Candidate Student | Baseline Unified Mind | Candidate Unified Mind |
|---:|---:|---:|---:|---:|
| 20,000 | 3,404 | 3,138 | 3,830 | 4,039 |
| 40,000 | 2,375 | 3,850 | 3,290 | 5,470 |
| 60,000 | 2,445 | 3,393 | 2,690 | 3,124 |
| 80,000 | 1,196 | 3,393 | 2,340 | 3,124 |

Candidate Unified Mind totals 15,757 versus baseline 12,150 (+29.7%) across these
four positions. It exceeds candidate Student's aggregate by 14.4%, but deals
about 8% less in the two longer-distance fixtures. Higher mastery is not proven
uniformly stronger in every fight. Ordinary-error backlash disappears as intended;
the explicit overcharge path is tested separately.

Remaining acceptance work: matched game compositions measuring actual credited
damage, useful control, survival, experiment completion and wasted preparation;
visible/readable Grand warnings and counterplay; companion targeting and HP,
kill attribution, items and farming; match/background/replay agreement; crowded
Magnus/Frieren/Vesper host performance. No new Vesper freeze-fix claim is made.

## Reproduction

Run from the repository root with the installed GNU Rust toolchain, wasm target,
Python/Pillow and Node. Commands use the workspace's RTK prefix.

```powershell
rtk proxy rustup run stable-x86_64-pc-windows-gnu cargo test --release --manifest-path native/tfm2_custom_ai/Cargo.toml --lib --quiet
rtk proxy rustup run stable-x86_64-pc-windows-gnu cargo test --manifest-path native/mod-api-stable/Cargo.toml
rtk node tools/verify-frieren-parity.cjs
rtk node tools/verify_science_combat.cjs
rtk python -X utf8 tools/verify-magnus-rework.py
rtk python -X utf8 tools/test_rework_migration.py
rtk python -X utf8 tools/verify-rework-data.py
rtk node tools/verify-rework-browser.cjs
rtk python -X utf8 tools/package-champion-reworks.py
rtk python -X utf8 tools/check-rework-package-manager.py
```

The browser check uses `playwright-core` and installed Edge. Set `NODE_PATH` to
your dependency directory if it is not already discoverable. It uses a temporary
fixture and port 19421, leaving an already-open editor untouched. Comparison:
`rtk python -X utf8 tools/record-rework-comparison.py --baseline <baseline-checkout>`.
Use commit `c5d7e99`/native 0.10.54 for that baseline.

The package script uses `rustc 1.98.1 (48a229cea 2026-09-01)`, GNU Windows,
`SOURCE_DATE_EPOCH=1792175594`, and two separate target directories. Reproduction
was verified on this machine, not across different toolchains/machines.

**Packaged DLL SHA-256:**
`e539ddc8289316ac8e32141d3a65995fb4965a3be3f2afe34101edb59751d1d9`

Detailed reports and the WASM hashes are in [rework-results](rework-results/).
Lab previews: [Frieren](rework-results/frieren-lab.png),
[Unified](rework-results/unified-lab.png). These are editor captures.

## Existing-career migration and installation

New careers read the candidate data definitions. Existing careers may embed old
champion definitions; copying a DLL alone does not rewrite them. The explicit
migration tool inspects by default:

```powershell
rtk python -X utf8 tools/migrate-champion-reworks.py <career.data>
rtk python -X utf8 tools/migrate-champion-reworks.py <career.data> --output <new-qa-copy.data>
```

Use a separate QA copy for acceptance. `--in-place` is available for an explicitly
chosen migration and creates/verifies a timestamped backup before atomic
replacement. Conflicting native controllers or nonstandard Frieren basic attacks
are rejected. Base/growth/balance patches and unrelated career/history records
are preserved. Inspection of the existing career found nine embedded definitions
needing updates and no conflicts; it did not write the save.

Close the game before an eventual installation. This PR deliberately ships the
reviewable source/package and leaves the live-game gate open.
