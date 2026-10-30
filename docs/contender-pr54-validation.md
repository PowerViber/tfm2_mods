# Validation of PR #54: Contender host-proof prototype

Checked on main `62fa0a6` (PR #54 merged after PR #53), Oct 30.

## What PR #54 is

It is a **prototype that tests whether the game can support Contender, not the champion**.

- **Contender is not playable.** No champion definition, Mod Power line, art, Lab, migration or version bump is in production.
- **The kit is frozen** in `rules/contender.json` with `releaseStatus: blocked-host-proof`.
- **A separate crate, `experiments/contender-host-probe`, builds four DLLs:**
  - an inert default that registers nothing;
  - three opt-in probes (`off-map-probe`, `dodge-probe`, `shield-probe`), each measuring one thing the engine must support. The game itself was never launched.
- **Release stays held** until 8 checks pass in the real game: fighting off the map, isolating the duel, enforcing the ring boundary, exact return, reserving the return spots, camera and replay, full dodge, and shield-hit identity.

## Results

| Check | Result |
|---|---|
| Inert / off-map / dodge / shield test suites | 8 / 20 / 12 / 12 passed, 0 warnings (matches the PR) |
| Two probes enabled together | Fails to compile, as intended |
| Reproducible Windows build (off-map probe, two clean builds) | Identical bytes. The hash differs from `builds.json` because this was built with a different toolchain (Linux GNU cross-compile), which is expected. |
| Production touched by the PR | No. Its diff contains nothing under `mods/` or `native/`. Production changes on main since `777204f` all come from PR #53 (Minato). |
| `tools/check-contender-host.py` on main | **Failed (exit 1) before this fix**; exit 0 after it. `--require-release` exits 3. |

## Findings

1. **Fixed: the release-gate checker broke as soon as main moved on.**
   - The cause: it required five production files to be byte-identical to the PR's base `777204f`. PR #53 then released native 0.10.79, so `productionFilesUnchanged` became false and the checker exited 1 for a reason unrelated to Contender.
   - Every future release would have broken it again.
   - Now it checks that the experiment stays out of production:
     - no Contender or probe id anywhere under `mods/` or `native/tfm2_custom_ai/src`;
     - no Contender entry in `rules/champions.json`;
     - no `_contender` Mod Power line.
   - Production hashes and the native version are still recorded, as information only.
   - The check no longer needs `rtk` or `git show`.
   - It catches a leak: a test file mentioning `tfm2_custom_contender` under `mods/` made it exit 1.
2. **Fixed: the original map edge was hard-coded.**
   - `960_000` was written four times in `transaction.rs`.
   - It is now `arena.mapLimit` in `rules/contender.json`, generated through `build.rs` like the other arena values, and a test pins it.
   - Behaviour is unchanged.
3. **Sound: the exact-return transaction.**
   - Entry is accepted only after the position reads back correctly, and a rejected second transfer rolls the first back.
   - A failed return keeps its record and retries every tick, and never settles for a nearby point.
   - Dead fighters are never moved.
   - The shared record is deduplicated, so one pair returns once.
   - The worst-case record (about 44 bytes) fits the 64-byte buff-name limit, and the limit is checked.
   - A fighter escaping the ring, a death, the match ending or the timeout all trigger the return.
4. **Sound: the probes stay within their scope.**
   - The dodge probe removes only its own buff.
   - Logging is bounded: a tick window, 2 MB per file, 64 KB per line. It never feeds back into the simulation.
   - Ordinary engine attacks and casts are left alone, and no damage is added.
5. **Kit sanity (rules only):**
   - Combination and Body Shot share one cooldown, and Slip and Guard share another.
   - Main Event: 75 s cooldown, 8 s duration, 65,000 range. Ordinary cooldowns recover 4/3 as fast inside, so they are 25% shorter.
   - Mod Power is `[30, 0, 5, 10, 10, 0, 10]`.
   - No inconsistencies were found. Balance can't be judged until a controller exists.

## Still unproven (needs your Windows game)

None of the 8 gates can be checked from the repository. Follow the README's live procedure in a throwaway install and match, with one probe at a time:

1. Do fighters move and fight off the map?
2. Does the camera follow them, and does replay seek work?
3. Do effects stay isolated from the rest of the match?
4. Are the exact-return edge cases handled?
5. What exactly does the dodge window block, at its boundaries?
6. Can a fully shield-absorbed hit be traced to its attacker?

Until those pass, Contender correctly remains unreleased.
