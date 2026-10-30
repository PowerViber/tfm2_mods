# Candidate verification — 2026-10-31

Native **0.10.80**, champion pack **0.2.80**, based on `6688f74`.

| Check | Observed result |
|---|---|
| Final native release suite | 806 passed, zero failures, 10 ignored |
| Boxing Lab crate, debug | 32 passed, zero failures |
| Browser | Seven checks pass; actual WASD, Q, Guard counter, Uppercut suppression and ring returns |
| Native/WASM | Identical complete state/effects for 43 recorded input frames |
| Development simulator | 24 development + 12 holdout cases; synthetic enemies/approach, not host observations |
| Athlete sensitivity sample | 13 attributes × three paired cases; some attributes tie in these narrow scenarios, so exhaustive athlete discrimination is not established |
| Art | Approved boxer/ring; deterministic extraction/packing; 1,753,088 decoded bytes; both facings and valid atlas/frame bounds |
| DLL reproduction | Two independent target directories produce the same tested/package hash |
| Isolated Mod Manager | Exit zero; 841 copied files identical; 728 champion files in manager comparison and identical DLL |
| Magnus regression | Five release-core stress runs; 128-piece p95 1.455–2.554 ms, median 1.734 ms; target <4 ms |
| Actual Windows game | Not run; installation and real-career migration not performed |

Packaged DLL SHA-256:

`58882909ddde74aabc3fdd5608dce4e492a93fa7b460b03b1104dc47496c4252`

The DLL hash matches both independent build outputs. `package.json` records the toolchain, source digest, epoch and WASM hash. Native release tests execute the same source as the packaged DLL; simulated host tests exercise SDK calls against a fake host and do not establish executable behavior.

The first debug sweep hit five overflow failures in Unified's unchanged planner, plus an initially stale Contender roster check. Regeneration/count correction fixed the roster. The normalized Unified source is byte-equivalent to the main baseline (`unified-baseline.json`); no unrelated Unified behavior was edited. The final release sweep passes, and the boxing debug suite passes. The whole repository's debug suite is therefore not claimed clean.

The first manager invocation also read the real game's older 0.10.79 log via APPDATA and correctly reported a version mismatch. That diagnostic is retained in `mod-manager-host-history.*`. The successful isolated rerun points APPDATA at an empty fixture, certifying only package/data integrity without fabricating a game launch or changing real logs.

`scenarios.json` contains actual simulator results and five-run timings, not promised outcomes for high-valued athletes. Its 128-**unit** Contender benchmark is separate from the existing 128-**piece** Magnus regression. Benchmarks were taken on the development machine, including background build contention; they establish the reported core target only, not a host FPS improvement.

Live acceptance is still pending for off-map stock/custom combat, visibility/pathfinding, universal interaction isolation, pre-movement rope enforcement, camera/replay seek, arena-born summons, other solid builders and complete Slip/shield-hit host behavior. Some are incomplete bridge coverage, not merely unobserved tests. See [the playtest limitations and procedure](../contender-playtest.md).

The stable-release check deliberately exits 3. The user authorized preparing a playable character before testing; this draft candidate must not be mistaken for a validated isolated-arena release.
