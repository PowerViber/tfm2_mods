# Objective health, Stark leap and performance release

This release doubles objective HP and HP growth, restores the ultimate companion's
Stark leap, and reduces repeated mod queries and decoded texture storage. The
game executable is unchanged. Work starts at main `a4098f15be65edc683fd62f8037d71b1e79ae9bd`,
including the eighteen-equation Unified kit.

Versions: native **0.10.57**, custom pack **0.2.58**, Frieren **0.1.19**.
The implementation uses branch `codex/objective-hp-stark-performance` in a separate
worktree. Original local DLL/art changes and career files are preserved. The
candidate is packaged in this worktree, **not installed into the Steam game**.
Live acceptance remains pending as requested.

## Objective rule and migration

| Objective | Shipped base HP → candidate | Shipped growth → candidate |
|---|---:|---:|
| Morgard / Epic | 10,000 → 20,000 | 1,500 → 3,000 |
| Serpen | 5,000 → 10,000 | 1,000 → 2,000 |

The authoritative management-server extension reads the loaded career's
GameSetting through the supported API. It doubles its four current HP values,
preserving customized settings rather than resetting them to factory defaults.
One atomic settings write changes only `epic_jungle.stat.hp`,
`epic_jungle.growth.hp`, `serpen_jungle.stat.hp` and `serpen_jungle.growth.hp`.
Attack, defences, rewards, timers and ordinary camps remain unchanged.

Before the write, `objective-hp/v1` stores a verified original-settings backup,
before/after values and a phase journal in the career's native-mod namespace.
Prepared/applied/restoring/restored phases recover interrupted writes and prevent
repeated multiplication. If settings are not available during server startup,
management callbacks retry until settled. This does not run in simulation ticks.
Existing snapshots/replays retain their recorded rules; newly created matches
use the updated career settings.

Restore checks all four output values first and reports conflicts rather than
overwriting later HP edits. Unrelated settings changed after migration survive.
A restored career remains restored on later normal loads. Corrupt records,
overflow, failed backup verification and rejected API writes fail without an
unbacked settings mutation.

Objective tactics now use SDK `Epic`/`Serpen` identities instead of HP thresholds.
This fixes the case where a grown or doubled Serpen looks like Morgard. Unknown
entity kinds are not guessed from health.

### Existing-career workflow

The native extension performs the settings migration on load for new and existing
careers. To keep an additional **complete career-file backup** before first load:

```powershell
rtk python -X utf8 tools/prepare-objective-hp-career.py <career.data>
rtk python -X utf8 tools/prepare-objective-hp-career.py <career.data> --apply
```

Inspection writes nothing. `--apply` validates the container and creates/verifies
a complete backup; it does **not** rewrite unknown binary settings records. The
supported native API applies the journalled change when the career loads.

For restore, close the game, prepare the backup and stage the flag in the actual
installed native mod folder:

```powershell
rtk python -X utf8 tools/prepare-objective-hp-career.py <career.data> --restore --native-dir <game/mods/tfm2_custom_ai>
```

Load the intended career, check the `objective-hp/v1` result in the native log,
then remove `objective_hp_restore.flag`. While that flag exists it requests
restoration for every career loaded; a career without a journal is left alone.
The supported mod commands `objective_hp.inspect` and `objective_hp.restore`
also expose inspection/restoration. Stale Frieren ability/passive/view definitions
can be repaired separately with the existing `migrate-champion-reworks.py`
inspection/output/in-place-with-backup workflow. No user career was rewritten
during development.

## Stark's restored ultimate leap

Skill 2 remains Defensive Magic. The targetable Stark from Journey's Companions
can leap for his twelve-second lifetime, once per committed enemy champion, with
a 120-tick cooldown. He selects the nearest visible targetable enemy champion
within 90,000 of both himself and his guarded ally, with entity ID tie-breaking.
The walkable landing point is fixed when committed and must remain inside the
map and current guard leash.

The fifty-tick warning drives ascending/descending art and the real carrier's
ground-plane movement. Enemies can leave the point. On landing, each eligible
enemy is hit at most once: `45 + 60% AP` magic damage and fifty ticks airborne
within 20,000; a 35% slow for 105 ticks within 40,000. Incidental minions, summons
and objectives can be affected; allies, towers and nexuses are excluded by the
native snapshot. Damage uses Frieren's supported attributed skill-damage route.

Death, expiry, carrier replacement, invalidated terrain/leash or interrupting
control cancels the pending impact. Spawning alone never substitutes damage for
a failed carrier. Ordinary walking/melee pauses in flight and no additional
melee hit occurs on the landing tick. Normal attack values and cooldowns remain.

The core/native adapter/lab share the same leap state. Frieren Lab transport v2
includes carrier ID, HP, interruption state, leap progress and target history;
v1 is rejected. Existing art supplies the jump poses. The lab has interruption,
capture, moving-target, wall, fog and spawn-failure controls.

Native interruption observation handles stun, airborne, fear, charm, taunt and
forced movement. The adapter excludes Bind/BlockAttack/BlockSkill because these
are its own persistent carrier-control guards; the public CC view does not
identify their caster. Enemy Bind cannot be distinguished reliably from that
owned guard. Actual carrier/overlay rendering, native targeting and item/kill
attribution remain live acceptance checks.

## Performance and equivalence

- Callback-local farming contexts cache owner team, assigned role and champion
  facts lazily. Magnus, Frieren, Unified and shared farming queries reuse them
  without carrying snapshots across mutations, owners or simulations.
- Tactical mark evaluation collects the typed objective list once per evaluation.
- Bush-area searches read the bush grid once instead of locking once per cell.
  Coordinates, wall checks and nearest-first ordering remain identical.
- Frieren reuses an emptied owner-local unit buffer. A genuinely unreferenced
  Coder test fixture helper is removed. Magnus's serializer remains because the
  lab still calls it. Registered legacy Frieren handlers are retained for saved
  definitions; lack of direct Rust calls alone is not treated as dead code.
- New opt-in adapter profiling distinguishes seed, simulation instance, origin,
  owner and phase. Samples/logs are bounded, cloned contexts rebind independently,
  and timings never affect planning or RNG. Existing Vesper effect/asset profiling
  remains, with independent context rebinding. Initial-owner windows are labelled
  separately; they are not measurements of the renderer's texture cache.

Five paired release benchmark runs, median of each run's statistic:

| Synthetic helper | Baseline p95 | Candidate p95 | Reduction |
|---|---:|---:|---:|
| 300 farming eligibility queries | 22.1 µs | 13.0 µs | 41.18% |
| Full-map bush search | 15.0 µs | 10.7 µs | 28.67% |

Farming host calls fell from 5,652 to 2,398 in that fixture. These are helper/mock
host measurements, **not game frame times**. The complete five-run median/p95/p99
records are in [query-performance.json](objective-stark-results/query-performance.json).

All **387 referenced frames across six Frieren atlases** retain identical RGBA
pixels, dimensions, non-rectangle metadata, order and durations. Decoded storage
falls from **8,888,864 to 4,154,368 bytes**, saving **4,734,496 bytes (53.26%)**.
Every resulting atlas is at most 2048 pixels per side. No new quantization,
effects, frame removal, palette reduction or animation retiming is used.
The HUD generator includes the same deterministic packing pass.

## Verification results and limits

- Native release suite: **347 passed, 0 failed, 6 ignored**. Four ignored cases
  predate this release; two are explicit performance benchmarks run separately.
- Objective journal tests cover exact four-field updates, repeated loads,
  customized values, interrupted apply/restore, failed backups and conflicts.
  Three file-backup tests and two existing champion-migration tests pass.
- Frieren: **25,920 complete native/WASM frames**, 24 development and 12 holdout
  scenarios. A separate **25,920-frame baseline comparison** with leaps disabled
  matches every original public/private state field and command.
- Farming policy equivalence compares frozen main against cached queries across
  roles, visibility, entity kinds and independent worlds. Bush queries match the
  original locations/order at boundaries and across radii.
- Magnus: **9,819 complete parity frames**, 36 scenarios, concurrent instances
  and 32/64/128-piece stress. Latest 128-piece core-step p95 is **0.865 ms**,
  planning p95 1.111 ms, maximum step 6.770 ms. Host/render time is excluded.
- Browser checks cover visible leap/landing, actual scene movement, interruption,
  spawn rejection and fixed-point escape. Equation Lab checks pass for eighteen
  equations, thirty-six signatures, two mastery ranks and 390-pixel layout.
- Repacked animation equivalence and deterministic generated data/HUD checks pass.
  The broader existing Unified source-art regeneration verifier stops on this
  Windows host because its hard-coded Linux DejaVuSansMono-Bold font is absent.
  Unified's shipped art, equations and combat core are unchanged; this is not
  reported as a passed source-art regeneration check.

The objective comparison contains sixteen paired unmitigated, host-free basic
and Frieren-core clear models at two party sizes and two growth stages. It does
not simulate stock AI, armour, items, aggro or retreat. The actual Unified `E=mc²`
raw formula retains its 4% maximum-HP component: at the fixture's stats, raw damage
changes from 420 on 10,000 HP to 820 on 20,000 HP (fixed part 20). Larger HP does
not promise doubled clear time against percentage-health attacks.

Real stock/custom team clear times, spawned-objective growth/respawn observation,
native jump rendering/targeting, crowded host performance and replay acceptance
are still pending. **No whole-game freeze fix is claimed.**

## REA evidence and reproduction

Artifact identities are recorded in [artifact-identities.json](objective-stark-results/artifact-identities.json).
The installed Windows x86-64 PE remains 87,228,416 bytes, SHA-256
`a1d05045405a12e2bed182daeafcad8c8a40cf29ab68ba6ea5bfd30257439582`.
REA 6.5.0 static inspection returns Rust-metadata evidence
`ev_f732eedfdff88cd390eb51531a70f3e08f4235ffe7cd7e2cc56f679bb1e2c17b` and PE-resource evidence
`ev_b891149a4df21c5b1f5a3a6857ab8bdd0cd2d6dd070934dcfb86ae2ce51fdac4`.
The compiler-path observation is an inference, not recovered source code or a
runtime profile. Matching identity permits reuse of the prior
[native-boundary investigation](magnus-native-evidence.md).

Read-only bundle inspection found the ordinary GameSetting at offset 521,030,305
(17,863 bytes), with equivalent objective defaults in the single-lane/deathmatch
settings. The supported settings API and actor-position/damage routes are traced
in the shipped SDK and repository adapters. Objective settings/identity questions
are answered; renderer stall causes and live performance remain unresolved.
Only stateless REA operations were used for this pass; no native analysis session
was opened and no executable bytes, default runtime paths or system configuration
were changed.

```powershell
rtk proxy rustup run stable-x86_64-pc-windows-gnu cargo test --release --manifest-path native/tfm2_custom_ai/Cargo.toml --lib --quiet
rtk node tools/verify-frieren-parity.cjs
rtk node tools/verify-frieren-unchanged.cjs
rtk python -X utf8 tools/verify-magnus-rework.py
rtk python -X utf8 tools/measure-query-performance.py
rtk python -X utf8 tools/record-objective-percentage.py
rtk node tools/compare-objective-clears.cjs
rtk python -X utf8 tools/test_objective_hp_prepare.py
rtk python -X utf8 tools/test_rework_migration.py
rtk python -X utf8 tools/repack-frieren-lossless.py
rtk python -X utf8 tools/verify-rework-data.py
rtk node tools/verify-stark-browser.cjs
rtk node tools/verify_unified_lab_browser.cjs
rtk python -X utf8 tools/package-champion-reworks.py
rtk python -X utf8 tools/check-rework-package-manager.py
```

Browser checks require Playwright/installed Edge; configure `NODE_PATH` when the
bundled dependencies are not on the normal module path. Use `--apply` on the
lossless packing tool to regenerate layouts. It verifies against frozen main.
Packaging uses the locked GNU Rust toolchain in two separate target directories
with `SOURCE_DATE_EPOCH=1792251129`, compares DLL bytes, and copies only into this
worktree's package. Final DLL/WASM hashes are in
[package.json](objective-stark-results/package.json); isolated Mod Manager results
are recorded separately. Neither proves that the installed game ran the candidate.

Editor previews: [leap](objective-stark-results/stark-airborne.png),
[landing](objective-stark-results/stark-landing.png).
