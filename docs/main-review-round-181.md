# Round 181: Contender 2.6, a heavy nerf to his skills and dodges (native 0.10.116, tfm2_custom 0.2.119)

**Reihan, after 2.5:** "its still too strong, like really really strong". He chose to nerf **both** his damage and his survivability, **heavily**.

## Measured, with the real core and his own AI
**Setup:** a 1v2 for 20 s, with the enemies basic-attacking him every 0.9 s. "× plain" compares his total damage (basics, skills, echoes, copies, Breaks and the ult) with a plain champion's basic attacks at the same Attack.

| | 2.5 | 2.6 |
|---|---|---|
| Level 3: damage | **2.84×** plain | **1.78×** |
| Level 3: enemy hits dodged | **68%** | **32%** |
| Level 9: damage | **3.63×** | **1.91×** |
| Level 9: enemy hits dodged | **36%** | **32%** |
| Basics only (unchanged since 2.5) | 1.24× (1.34× merged) | 1.19× (1.27× merged) |

At level 9 the rest is mostly his area skills hitting both enemies, which a single-target baseline doesn't credit.

**Why 2.5 was still so strong:**
- **The ult bank:** it refilled in about 10 s in a fight, because echoes and Tether copies banked too, so he cast the Flurry over and over. The Flurry alone was 2,855 of the level-9 damage.
- **Dodges:** 5 of them, each undoing a whole champion hit. Merge doubled the recharge and added 2 more, and the Flurry refilled them all.
- **Shadow Cyclone:** its 7-hit recast.
- **Echoes:** recasts echoed twice.
- **Tether:** copies were 20%.

## Changes (`contender-2.6`)
| | 2.5 | 2.6 |
|---|---|---|
| Dodges | 5, one every 6 s | **3, one every 10 s** |
| Merge dodges | +2, recharge ×2, 0.25 s gap | **+1, normal recharge and gap** |
| The Flurry refills dodges | yes | **no** |
| What banks toward the ult | every hit outside the flurry, echoes and copies included | **only his own direct hits** (basics, skills, recasts) |
| Echo damage | 20% | **15%** |
| Recast echoes | twice | **once** (merged hits still echo twice) |
| Cyclone clone (4 hits) | 15 + 18% Attack | **10 + 12%** |
| Cyclone recast | 7 hits of 18 + 22% | **3 hits of 12 + 15%** |
| Tether copies (flurry) | 20% (10%) | **10% (5%)** |
| Tether yank | 30 + 35% | **20 + 25%** |
| Merge step-out cross | 30 + 40% | **20 + 25%** |
| Flurry punch | 8 + 10% | **6 + 8%** |
| Haymaker | 60 + 80% | **40 + 50%** |

- **Unchanged:** basics, Rhythm stacks and Breaks, cooldowns, crowd control, the 40-stack bank size, and the level unlocks.
- **Charge pips:** they now top out at 3 (`ct_charges:0..3`).

## Checks
- **Native tests:** 1,417 passed, 18 ignored.
  - **New guard:** his own AI in the same 1v2 must stay **≤ 2.0×** plain basics and **≤ 35%** dodged, at levels 3 and 9. It measures 1.85× / 1.97× and 32% in the Rust harness.
  - **Updated tests:** recasts echo once; the Flurry doesn't refill dodges; 5% flurry copies; skill farming banks.
- **Lab and Mod Manager:** 3 Lab transport tests and 15 Mod Manager tests pass.
- **Generators:** current.
- **Builds:** the wasm and the DLL are rebuilt.
- **For Codex:** refresh parity and the Windows checks against 2.6. The charge pips use only frames 0–3 now.
