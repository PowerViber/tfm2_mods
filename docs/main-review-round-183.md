# Round 183: Contender 2.9 (native 0.10.118, tfm2_custom 0.2.121)

**Reihan:** "can the ult cooldown be less but have much more punches but much less damage, also keep the 5 shadows".

**His answers:**
- **Dodges:** 5 dodges that undo any champion hit, skills too, recharging one at a time every 15 s.
- **Flurry:** a 60 s cooldown and 72 punches at a steady medium speed (no slow-to-fast).

## Changes (`contender-2.9`)
| | 2.8 | 2.9 |
|---|---|---|
| Dodges | 1, every 12 s, basic attacks only | **5, one back every 15 s, any champion hit** (still at most one per 0.4 s) |
| Ult cooldown | 120 s | **60 s** (still −0.01 s per hit) |
| Flurry punches | 36, slowing from 14 to 2 ticks apart | **72, steadily 4 ticks apart** (all within the 5 s) |
| Flurry punch | 3 + 4% Attack | **1 + 2%** |

Unchanged:
- the banish during the Flurry;
- no merged echoes and no Breaks from the Flurry;
- a knock-up every second;
- the 2.8 skill damage and the level unlocks.

## Measured
**One Flurry on a dummy** (level 9, Attack 161):

| | 2.8 | 2.9 |
|---|---|---|
| Punches | 36 | **72** |
| Gaps between punches (ticks) | 14 down to 2 | **all 4** |
| Damage | 324 | **288** |
| Ready again | at 120 s | **at 60 s** |

**His own AI in a 1v2 for 20 s** (× a plain champion's basic attacks):

| | 2.8 | 2.9 |
|---|---|---|
| Damage, level 3 | 1.26× | **1.26×** |
| Damage, level 9 | 1.32× | **1.33×** |
| Enemy hits dodged, level 3 | 9% | **27%** |
| Enemy hits dodged, level 9 | 9% | **23%** |

## Checks
- **Native tests:** 1,420 passed, 18 ignored.
- **Updated tests:**
  - five dodges recharge one every 15 s, and basics don't speed them up;
  - a skill hit is dodged too;
  - Merge adds no dodges, and the 0.4 s gap holds;
  - the Flurry is 72 steady punches;
  - the fight guard: ≤ 1.4× damage, and its dodge cap is relaxed to ≤ 30% (it measures 27% / 23%).
- **Lab and Mod Manager:** 3 Lab transport tests and 15 Mod Manager tests pass.
- **Generators:** current.
- **Builds:** the wasm and the DLL are rebuilt.
- **For Codex:** the dodge pips show 0–5 again, and the Flurry plays 72 cues, one every 4 ticks. Refresh parity.
