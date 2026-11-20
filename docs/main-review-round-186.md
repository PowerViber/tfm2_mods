# Round 186: Contender buff (native 0.10.121, tfm2_custom 0.2.124, `contender-3.3`)

**Reihan:**
> "ai was good, but he is quite weak now, buff the shadow recharge and flurry charges … and make the skill 15 hits 1.5s"

Then: "keep the teleport at last hit, but make the can use s2 again no cd, so no waiting for 0.75".

## Changes
| | 3.2 | 3.3 |
|---|---|---|
| Shadow dodge recharge | one every 15 s | **one every 10 s** |
| Flurry charges | 3, one back every 16 s | **4, one back every 12 s** |
| Flurry | 10 punches over 1 s | **15 punches over 1.5 s** (one every 6 ticks) |
| Wait before the next Flurry | 0.75 s | **none**; the engine slot's cooltime is 1 tick |

- **Unchanged:** he still reappears beside the last enemy hit, and punch damage stays 1 + 2% Attack.

## Tests
- **Codex's tests hard-coded the old numbers:** 961, 933 and 901 ticks, "2 charges left", the 60-tick end and the 106 cast tick.
- **Fix:** those tests now read the generated constants (`d::FLURRY_*`, `d::CHARGES_*`), so they follow `rules/contender.json`.
- **Lab transport test:** its ult order is moved past the longer Flurry.
- **Results:**
  - **Native tests:** 1,414 passed, 18 ignored.
  - **Lab and Mod Manager:** 3 Lab transport tests and 15 Mod Manager tests pass.
  - **Generators:** current.
  - **Builds:** the DLL contains 0.10.121, and the wasm is rebuilt.
