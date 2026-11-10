# Kratos art and Lab verification — 2026-11-10

**Map/situations follow-up:** the [installed battlefield review](../kratos-game-world-review/README.md) supersedes the earlier backdrop/placement assumptions below. Levi's wall grid agrees, but the corrected source registration uses a 160-pixel border and 1,000 world units per art pixel. Original stock actors and eight mid-game situations are now included; 23 additional browser checks pass. Earlier measurements below describe the original art/fixture workload, not the new full scenario host.

Implementation began at PR #74 head `740a8e9`. Current main `ad6dda3` was merged as `ef036bc` to preserve round-160 balance changes and resolve the generated roster conflicts. Kratos’s rules/core/data/WASM are unchanged from the original PR head. The new native code selects presentation only.

## Automated results

| Check | Result |
|---|---|
| Full native suite, GNU Windows toolchain, after merging main | **1,145 passed, 13 ignored, 0 failed** |
| Native Kratos subset, before main merge | 30 passed: 26 existing kit/glue tests and four presentation tests |
| Lab transport | 2 passed |
| Existing kit browser checks | 10 passed |
| Artwork/Lab browser checks | **27 passed** |
| Presentation independence | 24 development + 12 holdout scenarios, identical complete shared-core inputs/outputs with artwork enabled/disabled |
| Manual athlete independence | Each of 13 values at 20/80, across three fixtures; identical mechanics |
| Native/WASM parity | **405 complete outputs equal** under identical saved inputs |
| Recording replay | Every output in a 100-step recording matches; incompatible rules explicitly rejected |
| Playback | 30/60/120/144/240 Hz each produces exactly 600 simulation ticks in ten seconds; pause/background resets and single-step checked |
| Packed art | 1,211 animation-frame entries checked; all bounds, mirrored body pixels, foot anchors, pose counts and durations pass |
| Reproducibility | 24 generated atlases/metadata/clocks/bindings/animations byte-identical after rebuilding frozen sources |
| Canonical generators | Kratos and roster checks pass; 84 champions, 269 skills |
| Mod Manager champion loader | `the_repos_champions_all_load` passes |
| Isolated Manager fixture | 878 files identical; 765 champion files compared, 0 different or missing; exit 0 |

The GNU Windows native test build reports **four inherited dead-code warnings** in Linux-only detour-test helpers (`F2_HELPER`, `f3`, `f4`, `f5`). Do not describe this Windows run as warning-free. Thirteen platform/live-dependent tests remain ignored.

Reproduction commands are in the review README and the corresponding tools. Browser tests use the installed headless Edge and the bundled Playwright module. The unchanged shared core was compiled as the native trace example; no new release DLL was copied into the package.

## Presentation and performance

- Kratos: twelve whole-body poses, eight frames each, both facings; Axe/Blades accessory variants and three Rampage stages. The visible roster idle is the genuine resting body with empty hands. No body hiding or new cosmetic movement/control restrictions.
- Fenrir: eight running drawings in all eight headings. Every native renewal remains one frame and one tick, selected from a stable three-tick phase clock. No extra emissions.
- Chain artwork uses the accepted 55,000/90,000 reach, heading and committed midpoint. Frost/fire footprints use canonical swing/smash sizes.
- Actual map artwork/walls, body/effect scale, square canvas pixels, full-map/follow views and focus-safe keyboard input are checked. Map art is no longer vertically stretched beside the inspector.
- Native body + weapon + FX atlas allocation: **16 MiB**; maximum atlas edge **2048**. The offline assembled-preview atlas is not shipped or loaded by the Lab.
- Five local browser cold navigation/art loads: approximately **208–244 ms**. Five sustained 12-body + Fenrir render runs: **0.2 ms p95** at this timer’s resolution. These are local browser measurements, not native host performance or Magnus’s 128-piece acceptance.

## Separate release and live gates

The repository includes approved assets and native presentation source. **The packaged DLL is inherited from current main, not a Kratos release build.** Its existing SHA-256 is `400836d0c21fc719f3d5580ed20bd1aae51593cbe7d1f0968558c2704da3bb6e`; this is provenance only, not a newly reproducible Kratos DLL claim. Versions remain main’s native 0.10.95 / champion pack 0.2.98 until a separate release chooses unused versions and builds/synchronises dependencies.

Pending actual-game checks: roster visibility; hand/weapon alignment under host facing/pose clocks; both weapon stances; interruption/death/respawn and crowded combat; Sight callback timing, shields and healing reduction; Chain Dash’s glide-plus-hop wall presentation; cage interaction; Fenrir target eligibility and cleanup. Lab damage/HP/shield results are simulated host observations and do not prove engine acceptance.

Sight still refunds damage through healing natively; this pass does not turn it into genuine pre-hit avoidance. The Lab labels its idealised avoidance model. Balance, the level-10 release comparison, a new packaged DLL/hash, installation and backed-up career migration remain separate. No real game or career was modified.
