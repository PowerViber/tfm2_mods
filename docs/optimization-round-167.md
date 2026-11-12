# Round 167: native optimization pass for every modded champion (native 0.10.101, tfm2_custom 0.2.104)

Reihan: the game launch, and mostly **Proceed**, felt slow; optimize every modded character and add-on.

It builds on round 166 (`akane-1.5`, Akane engages while healthy).

- **Why Proceed:** it simulates the day's other matches in the background, so the per-tick cost of the native passives is paid in every background match that drafts a modded champion.
- **What doesn't change:** gameplay, AI decisions, art and data. Every test gives the same answers as before.

## How it was measured
- **New ignored benches** in `native/tfm2_custom_ai/src/perf_bench_tests.rs` run every native passive on the glue mock host:
  - **the setup:** a walled 30×30 map (the outer ring, river banks and pillars, about 170 wall cells) and a crowded lane with 10 champions, 12 minions and a tower;
  - **`whole_roster_tick_cost`:** each champion alone (`BENCH_ONLY=<name>`, `BENCH_TICKS=<n>`);
  - **`mixed_match_tick_cost`:** three 5v5s of ten modded champions, each ticking every tick (`BENCH_MIXED=<0..2>`), which is what a background match looks like.
- **Numbers are instruction counts** (valgrind callgrind), because wall-clock time on a shared machine is too noisy. The mock host's own cost is included, so read them relative to each other.

## Results (round 165 → round 167)

| Ten modded champions in one match | Round 165 | Round 167 | Change |
|---|---|---|---|
| Akane, Thalassa, Wright, Kratos, Grimhald, Raze, Rally, Gojo, Steve, Magnus | 378 M | 224 M | **−41%** |
| Vesper, Scribble, Unified, Contender, Frieren, Coder, Omen, Isliid, DIO, Levi | 224 M | 142 M | **−37%** |
| David, Aegis Zero, Vader, V1, Minato, Akane, Thalassa, Raze, Rally, Magnus | 477 M | 154 M | **−68%** |

The match numbers were measured on the tree merged with round 166, so they include `akane-1.5`.

**Each champion alone,** in thousands of instructions per tick (the harness cost is subtracted):

| Champion | Round 165 | Round 167 | Change |
|---|---|---|---|
| Raze | 2,424 | 399 | −84% |
| Rally | 347 | 234 | −33% |
| Gojo | 127 | 102 | −20% |
| Steve | 125 | 108 | −14% |
| Contender | 116 | 107 | −8% |
| Akane | 113 | 106 | −6% |
| Magnus | 101 | 96 | −4% |
| Grimhald | 100 | 77 | −23% |
| Unified | 92 | 91 | −1% |
| Thalassa | 78 | 73 | −7% |
| Kratos | 77 | 67 | −14% |
| Frieren | 63 | 59 | −5% |
| Wright | 61 | 56 | −9% |
| Omen | 57 | 47 | −17% |
| Scribble | 52 | 46 | −11% |
| Vesper | 49 | 44 | −10% |
| Coder | 46 | 43 | −7% |
| Isliid | 43 | 41 | −5% |
| Levi | 29 | 28 | −2% |
| DIO | 29 | 23 | −20% |
| David | 22 | 20 | −8% |
| Aegis Zero | 20 | 19 | −3% |
| V1 | 14 | 16 | +17% |
| Vader | 13 | 16 | +18% |
| **All 24** | **4,197** | **1,916** | **−54%** |

- **V1 and Vader alone:** they pay about 2k more instructions per tick, because the shared snapshot has nobody to share with.
- **In a real match:** the input AI already reads the snapshot for every player every tick, so it is warm and they read it for free.

## What changed
1. **One champion snapshot per tick for everyone.**
   - **Before:** about 30 places in the passives called `champions(sim)` directly. With 10 modded champions, the same snapshot (every champion's buffs and CCs) was rebuilt about 10 times a tick.
   - **Now:** they all use `champions_shared`.
   - **Exactness:**
     - the snapshot is keyed by match, tick, a fingerprint, and the SDK's new `removal_epoch` (buff removals and CC clears this thread made), so a buff swapped at the same count is never served stale;
     - the first reader of a tick builds it straight away and derives the fingerprint from what it read.
2. **The frame builders reuse it:**
   - Akane, Thalassa, Wright, Kratos and Grimhald read a champion's buffs from that snapshot, through `entity_buffs`;
   - minions, camps and towers are still read from the host.
3. **SDK:**
   - `BuffV1::name()` and the CC `name()` skip full UTF-8 validation when the name is ASCII, which nearly all are (same result);
   - `champions()` pre-sizes its vectors.
4. **Raze:**
   - **Line sweeps:** `sweep` only tests walls and surfaces near the segment (same answers).
   - **Grenade bank shots:** they sort walls with the key computed once per wall (`sort_by_cached_key`, same stable order).
   - **Open maps:** the wall list is no longer rebuilt every tick when there are no walls.
5. **Rally:** `clear` (line and run-up checks) only tests blocks near the segment (same answers).

## Tried and dropped
- **Invalidating the snapshot on every simulation write:** almost every passive writes something each tick, so it rebuilt more and cost more.
- **A per-callback buff cache inside the SDK:** its bookkeeping cost more than the reads it saved (−0% to +36%).

## Verification
- **Tests:**
  - `cargo test --release` (merged with round 166): **1,217 passed**, 0 failed, 0 warnings;
  - generators current;
  - Mod Manager tests (15) pass.
- **Lab crates:**
  - all pass: Rally 130, Raze 136, Thalassa 45, Wright 29, Grimhald 40, Akane 9, Kratos 2, Steve 50, Gojo 86, Magnus 88, Frieren 22, Unified 2, Universal 14, Contender 49;
  - `raze_lab`'s `raze-bench` binary target fails to compile on main as well (`valorant_guns::basic_raw` is configured out). It is unrelated to this change, and its library tests pass.
- **DLL:** the Windows DLL reports **0.10.101**.

## Not changed
- **Launch:** what the game reads at launch (mod data and art) is unchanged. Round 165 added only 1.4% text and 0.6% decoded art (see `main-review-round-165.md`), and the Gundam review images in `mods/tfm2_gundam/source` are never installed.
- **Add-on folders:** `tfm2_jjk`, `tfm2_valorant`, `tfm2_toon` and the rest are data-only; their champions' native passives are covered above.

## Live check (pending)
- **Timing:** time a Proceed against round 165.
- **Optional:** put `perf.flag` in `mods/tfm2_custom_ai` and send `perf_log.txt`.
