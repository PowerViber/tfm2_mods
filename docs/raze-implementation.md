# Raze: purchased utility, momentum and thirteen athlete attributes

Raze is an AP fighter with purchased guns, grenades, a destructible Boom Bot and two paid satchels. Showstopper is a fixed-flight rocket that can be fired during a satchel manoeuvre. There is no mastery system. The supported champion category is `Range`, with AP/Range/CC tags; the internal identity is `tfm2_valorant_raze`.

Implemented on `codex/raze-air-strafe`, from main `ba8734b`. Native **0.10.68**, champion pack **0.2.68**, Valorant **0.1.17**. The original checkout's uncommitted Magnus work is preserved. Publishing this package does not install it or change any real career.

## Mechanics and economy

`rules/raze.json` generates native constants, editor rules, champion data, localisation and the preset. Omen and Raze share a host-free gun table. Omen's existing basic definition, modifiers, prices, weapon tiers and artwork remain unchanged. Raze retains the engine-owned ordinary attack identity; she does not inject stock attack/cast inputs.

| Utility | Prototype behaviour |
|---|---|
| Blast Pack | 200 credits each, two inventory slots, 40 HP, five-second lifetime. Throw within 24,000 units; detonation launches Raze within 18,000. Armed after one second: 20 + 15% AP and enemy displacement. Destruction/expiry never explodes. |
| Paint Shells | 300 credits, one inventory slot, eight-second reuse limit. 0.35-second preparation, 80,000 range. 65 + 55% AP main blast, then four 15 + 12.5% AP child blasts; at most two child hits per target. |
| Boom Bot | 300 credits, one inventory slot, eighteen-second reuse limit. One real targetable carrier, 25% of Raze's max HP, five-second lifetime. Visible forward-cone acquisition, terrain-aware travel and 45 + 45% AP explosion. |
| Showstopper | Ultimate unlock, 0.7-second equip, six-second firing window, seventy-second base cooldown from acceptance. 160,000 range at 400,000 units/second. 180 + 120% AP central blast, declining to 40% at 32,000 radius. One fixed collision, no homing. |

Grenade and rocket self-damage is 25% before normal mitigation. Area damage resolves against current damageable entities, including concealed enemies caught after commitment. Visibility is required for planning and acquisition. Towers/nexuses are excluded from explosive damage. Friendly champions are unaffected by blasts.

Weapons are Classic (free), Sheriff (800), Spectre (1,600), Judge (1,850), Vandal (2,900), Operator (4,700; purchased at 5,500 available credits). Gun-first purchasing, armour prices, 1,000 initial credits, 9,000 cap and Omen's income rules are retained. Utility must be bought; it never regenerates. Weapons and unused inventory survive death; armour and active effects do not.

One conservative API adaptation: Raze's purchased armour adds only missing shielding toward its purchased cap. It does not delete unrelated allied shields. The public API cannot remove only a particular shield contribution.

The initial launch adds 160,000 units/second radial velocity, capped at 240,000 after chained impulses. Air acceleration is limited to 180,000 units/second squared. Countersteering brakes rather than instantly reversing direction. Flight lasts 39 ticks, extendable to at most 66 with the second satchel. Physics runs every tick; the native movement hook does not overwrite airborne velocity. Grounded Raze retains wall navigation. Terrain sweeps, roots, stun, resources and unlock checks apply equally to all athletes.

## Athlete decisions

The existing version-2 pinned profile supplies all thirteen values. Missing profiles retain the existing reported neutral fallback (50 each). No new attributes, progression, mastery or stat bonuses are introduced. Decision hashing is deterministic and separate from combat RNG. Manual inputs bypass these biases/errors.

| Attribute | Applied responsibility |
|---|---|
| Ego | Danger cost and willingness to commit alone |
| Judgement | Value estimates, bounded ±20-to-zero estimation error |
| Positioning | Launch/landing geometry and firing access |
| Control Speed | First response to a changed observed situation, 24–6 ticks |
| Skill Accuracy | 30–100% observed motion lead, 6,000-to-zero placement error |
| Skill Dodge | Reaction to visible hazards, ground evasion or escape launch |
| Aggressive | Offensive entry preference, separate from danger tolerance |
| Order | Nearby support and team assignment weighting |
| Roaming | Willingness to rotate |
| Macro | Missed-wave/resource cost, farming commitments and assignments |
| Micro | Steering every 12–3 ticks, one sampled timing error up to ±6 ticks per detonation intention |
| Map Awareness | Last-sighting uncertainty and exposure review, without hidden current positions |
| Strategy | One-to-three-action continuation search |

Every level considers shooting, grenade, bot, single launch, double launch, rocket and escape candidates when legal. The authored double launch is available even with a one-action horizon. Search is bounded to 64 evaluated candidates, beam width eight and three macro actions. Queued stages revalidate visibility, resources and geometry before acceptance. Accepted projectiles keep their committed trajectories. Ordinary camp ability use follows jungler eligibility; other roles use waves and accepted major-objective plans.

The shared movement hook skips its generic athlete/team movement rewrite for Raze, avoiding duplicate personality and reaction penalties. It still performs wall navigation on the ground. Airborne guidance enters the steering controller rather than replacing velocity. Low HP has a ground retreat option even after both satchels are spent.

## Runtime, artwork and Lab

`raze_core.rs` owns purchases, inventory, casts, stable effect IDs, momentum, damage commands, observations and actual chosen plans. `raze.rs` adapts supported simulation snapshots to that core. Each passive clone owns its state; there is no shared mutable combat state. Credits and carrier receipts are deduplicated. Failed carrier allocation restores the failed summon inventory/cooldown rather than creating invisible damage.

Satchels and Boom Bot use targetable Alchemist carriers with autonomous movement/attacks/skills suppressed. Native effects cover the carrier; actual coverage and targetability remain a live acceptance requirement. Gun hits use the engine; utility damage uses normal Raze attribution. Existing bounded adapter profiling separates snapshot, planning and host work.

The approved body and effect answers, exact source hashes and mechanically rotated rocket frames are recorded in [approval.json](raze-review/approval.json). The 36×44 canvas, foot baseline, original animation names, frame counts and timing are preserved. Eight approved source poses are reused across those frame sequences; this is not a new set of unique drawings for every frame. Mirrored airborne rifle/pack/grenade/rocket variants are included. Existing Omen pixels remain unchanged. The full Valorant pack uses **4,191,872 decoded RGBA bytes**; the largest Raze atlas is **1024×344**, below both limits.

[Body comparison](raze-review/raze-comparison.png) · [Animated poses](raze-review/raze-poses.gif) · [Effects comparison](raze-review/effects-comparison.png) · [Lab screenshot](raze-results/raze-lab.png)

Raze Lab v1 and Skill Test expose all thirteen controls, pinned profiles, manual utility, purchases, steering, reaction timers, intended/actual detonation times, selected continuations, alternatives and rejection reasons. The Lab uses the same Rust core compiled to WASM and the approved effect atlas. Presentation uses bounded retained visual records, independent of simulation randomness. Recording import/export verifies complete state and command replay hashes.

Lab ordinary attacks and incoming attacks are a simplified host model. The displayed post-mitigation damage assumes twenty resistance and is explicitly labelled an estimate. Lab estimates, requested raw damage and real engine-attributed damage are not interchangeable.

## Automated evidence

- Full native release suite: **648 passed, zero failed, eight pre-existing ignored tests**. No compiler warnings in this run.
- Shared Raze Lab suite: **40 passed**. This includes transport validation, manual profile independence, geometry, resource/death boundaries, spawn failures, concealment, fixed rocket collision, chained impulses and replay isolation.
- All thirteen attributes have three command-trace comparisons at **20 versus 80**, holding the other twelve at 50. Actual fixture seeds are in [athlete-pairs.json](raze-results/athlete-pairs.json). These demonstrate distinct decisions/inputs, not guaranteed superiority from high Ego or every all-high profile.
- **24 development + 12 holdout** browser scenarios, **6,480 native/WASM complete-state and command comparisons**, identical recording replay, independent sessions, thirteen controls and 390-pixel light/dark layouts. No browser errors.
- Canonical generation, approved frame pixels, anchors/timings and atlas bounds checked. Farming metadata: **78 champions, 234 slots**, 11,700 target/role/team combinations; this is metadata validation, not evidence of 11,700 executed game casts.
- Five identical-input Raze release-core timing runs: median p95 **0.0045 ms**, median p99 **0.0171 ms**, 194,400 steps per run. Magnus 128-piece p95: **0.851 / 0.764 / 0.821 / 0.839 / 0.906 ms**. Native host and rendering are not included.

The benchmark uses 36 scenario seeds, three uniform profile levels and both teams. It records requested raw damage, simplified delivered damage, launches, exposure, deaths, missed-wave observations, objective damage and wasted preparation separately. The current short harness produced zero deaths, wave kills and missed-wave observations; those zeroes limit its value as a balance evaluation. Uniform 80 did not dominate damage: all-high personality values also change commitment. Full measurements are in [performance.json](raze-results/performance.json).

## Pending live acceptance and API limits

No actual match was observed for this release. Required checks remain: carrier targeting/HP/overlay coverage, body/height alignment, terrain at native map edges, firing during flight, interruptions, item procs, self-damage interactions, kill/assist attribution, bought utility over a match, warnings/counterplay and crowded host/rendering performance. The PR stays draft for this reason.

The projectile API exposes public snapshots but not a general per-projectile visibility query or collision shape. Native dodge guidance therefore uses nearby public projectiles from visible casters, conservative clear unsmoked lines, an estimated hazard radius and observed motion. This does not establish exact engine projectile parity. Hidden enemy current positions/resources do not enter planning; committed area attacks still query actual impact victims.

## Reproduction and delivery

Run these from the implementation checkout, with the editor served at `http://localhost:7286`:

1. `rtk proxy python -X utf8 tools/build-raze.py`
2. `rtk proxy python -X utf8 tools/check-raze-release.py`
3. `rtk proxy python -X utf8 tools/package-raze.py`

The final script builds the DLL in two independent target directories, compares hashes, packages the tested result, then runs Mod Manager against a complete temporary fixture under ignored backups. It never installs into Steam or migrates a real career. Manager reported **717 custom-pack files compared, zero different, zero missing; DLL identical; UP TO DATE**. Its file comparison covers `tfm2_custom` and the native DLL; the release script supplements that with SHA-256 comparisons of every copied pack, including every Valorant/Raze file. Counts are in [mod-manager.json](raze-results/mod-manager.json). This is package consistency evidence, not live acceptance.

Native **0.10.68** DLL SHA-256 (both builds and package):

`cab733f8be88994509ee56cd2c6c8dde680abdacecf39f8b82b855686a1491e6`

Toolchain, source fingerprint, WASM hash and synchronized pack versions are in [package.json](raze-results/package.json). Raze is a newly added champion, so no existing champion or career definition migration is performed by this PR.
