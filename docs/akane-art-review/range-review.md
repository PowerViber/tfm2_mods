# Akane 1.1 — longer range with visible travel

Reihan requested a much farther S2 backflip and kunai, a much farther ultimate dash, and slower travel. When asked whether the kunai should also travel before dealing damage, he chose **“Yes—give the kunai visible flight time.”** The following numbers are Codex’s prototype defaults for that direction.

| Property | PR baseline | Current |
|---|---:|---:|
| Backflip distance | 15,000 | 30,000 |
| Backflip duration | 12 ticks / 0.2 s | 30 ticks / 0.5 s |
| Kunai range | 60,000 | 120,000 |
| Kunai delivery | Immediate line hit | Non-homing flight at 3,000/tick / 180,000 per second |
| Kunai recast dash speed | 4,500/tick | 3,000/tick |
| Night dash range | 120,000 | 240,000 |
| Night dash speed | 6,000/tick | 4,000/tick |

Rules revision is `akane-1.1`; transport remains v1. A kunai commits its endpoint and snapshots Attack on acceptance, sweeps each traversed segment, and applies one damage packet/three-second mark on contact. It passes over terrain, retaining the baseline rule, and ignores towers/allies. A miss cannot recast. Retreatment is still terrain-clipped. Flight is owner-scoped and cleared on death/respawn.

At 180 Attack, the unchanged throw formula is **138 raw physical damage**. A stationary champion centred 100,000 away with 3,000 radius is hit on tick 31 because the kunai has 5,000 half-width. Casts still spend cooldown on acceptance. Soul, Edge, damage formulas, Mod Power, basic attack clocks and ultimate duration/cooldown are unchanged.

The existing planner uses the new canonical range/speed values. No new personality weights, global-AI work or strength modifiers were introduced. The Kunai hold/recast test now disables Night so it continues to isolate Kunai; the expanded Night legitimately creates a second opportunity against its formerly out-of-range enemy.

## Verified

- Full Windows GNU native suite: **1,191 passed, 13 ignored**, no failures. Four inherited dead-code warnings in `native_bridge/detour_tests.rs`; no new warning claim.
- Lab Rust tests: **2 transport + 7 range/flight tests** passed.
- Existing skill browser checks: **9 passed**. Expanded artwork/map/travel browser checks: **37 passed**.
- Complete native/WASM outputs: **3,240 equal**, covering 24 development/12 holdout fixtures. Art enabled/disabled retains identical core/model combat. Thirteen-stat manual comparisons pass at 20 versus 80 across three fixtures each.
- Five matched Lab runs: core plus modeled-host step p95 **1.5–2.2 ms**, draw p95 **0.9–1.3 ms**. Cold page/assets: 509–580 ms. These are browser observations, not native game or Magnus performance measurements.
- Reproducible source extraction/packing, mirrored frames, anchors, atlas limits and generated bindings pass. Body/effects total **4 MiB**. Isolated Mod Manager fixture compared **883 files**, all matching.

`browser-results.json`, `verification.json`, `native-tests.txt` and `mod-manager.json` contain current evidence. `art-only-*` files preserve the earlier `akane-1` evidence and its unchanged core/WASM; they do not describe this gameplay revision.

## Remaining gates

Native live-game animation/facing, actual dodge/visibility/targeting, cooldown feel and balance still need observation. The Lab uses a labeled modeled stock host and cannot establish those results. Its projectile collision sees only eligible units supplied in the shared frame; this is not proof of hidden-unit collision in the stock engine.

Release versions and the new packaged DLL remain pending. The old packaged DLL SHA-256 is `c0bad5da4891e8dee321c87852eecfbfd295c37b39613944c253592e045b2d86` and has not changed; it is not represented as containing this adapter or travel revision. Installation and real-career migration have not been performed.
