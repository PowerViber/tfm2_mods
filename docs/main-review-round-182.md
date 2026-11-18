# Round 182: Contender 2.7 (native 0.10.117, tfm2_custom 0.2.120)

**Reihan, after 2.6:**
> "its still too strong - dodges was part of it he cant be easily hit haha and also the passive stacks at 20 comboes with merge echo ult easily kills someone, also i want the ult to make him invisible when the flurry starts until its done so enemies dont hit him"

Then: "nerf his passive buffs".

## Changes (`contender-2.7`)
| | 2.6 | 2.7 |
|---|---|---|
| Dodges | 3, one every 10 s, any champion hit | **1, every 12 s, enemy basic attacks only** (skills and ults always land) |
| Basics speed the dodge recharge | 0.5 s each | **no** |
| Merge dodges | +1 | **none** |
| Ult | 40 banked stacks, no cooldown | **120 s cooldown**; every hit outside the flurry (echoes and copies included) takes **0.01 s** off |
| A merged Flurry | every punch echoed | **no echoes** |
| Flurry punches on a full target | Shadow Break | **no Break** (it holds at 20; his next normal hit Breaks) |
| During the Flurry | 3-tick invisibility + 100% damage reduction, renewed | **banished** (untargetable, unseen, input-locked), renewed every tick, with the old lease kept as a backstop |
| Rhythm per stack | +1% attack speed, +0.8% move speed, +1% damage taken | **+0.5%, +0.4%, +0.5%** (+10% / +8% / +10% at 20) |
| Shadow Break | 4% max HP + 20% Attack, buffs held 5 s | **3% + 15%, buffs held 3 s** |
| Kill linger | 5 s | **3 s** |
| Echo damage | 15% | **10%** |
| Merge move speed | +15% | **+10%** |

### How "basic attacks only" works
The passive's `on_damaged` hook doesn't say what kind of hit it was, so the bridge infers it:
- **Skill:** a hit lands within 1.5 s of the attacker casting one of its engine skills (Skill, Skill2 or Ult). It is treated as a skill and is never dodged.
- **Basic attack:** everything else, which can be dodged.
- **Limitation:** native champions cast their skills outside the engine slots, so their skills still read as basics and can be dodged.

## Measured (his own AI, 1v2, 20 s, enemies basic-attacking every 0.9 s)
| | 2.6 | 2.7 |
|---|---|---|
| Damage, level 3 (× a plain champion's basics) | 1.78× | **1.55×** |
| Damage, level 9 (with one ult) | 1.91× | **1.69×** |
| Enemy hits dodged | 32% | **9%** |
| Basics alone (merged) | 1.19× (1.27×) | **1.11× (1.15×)** |

## Checks
- **Native tests:** 1,420 passed, 18 ignored. New or changed tests:
  - **Dodge:** his one dodge recharges in 12 s.
  - **Ult cooldown:** 120 s, and each hit cuts 0.01 s; flurry punches never cut it.
  - **Flurry and Breaks:** flurry punches never Break, and his next hit after the flurry does. A merged flurry has no echoes.
  - **Glue tests:** a skill hit (the attacker cast within 1.5 s) is never dodged and a basic is; the vanish calls `entity_banish`.
  - **Guards, tightened:**
    - the AI 1v2 fight at ≤ 1.8× and ≤ 15% dodged (it measures 1.58× / 1.72× and 9%);
    - basics within 1.0–1.2×.
  - **Mock host:** it gained an `entity_cooldown` hook.
- **Lab, driven directly:** the ult casts at once, is refused at 119 s, and casts again at 120 s.
- **Other checks:** 3 Lab transport tests and 15 Mod Manager tests pass; the generators are current; the wasm and the DLL are rebuilt.
- **For Codex (Windows):** check in game that the banished flurry can't be targeted or hit and that his body is hidden. Refresh parity. The bank pips are now unused, and the dodge pips are 0/1.

## `contender-2.8`: lower damage, the same dodge (Reihan)
"can I have a lower final damage but keep the dodges?" Target: about 1.3× a plain champion's basics. The dodge stays exactly as in 2.7.

| | 2.7 | 2.8 |
|---|---|---|
| Echoes | 10% | **3%** (mostly crowd control now) |
| Tether copies (flurry) | 10% (5%) | **3% (2%)** |
| Tether yank, step-out cross | 20 + 25% Attack | **12 + 15%** |
| Cyclone clone, 4 hits | 10 + 12% | **3 + 4%** |
| Cyclone recast, 3 hits | 12 + 15% | **4 + 5%** |
| Flurry punch | 8 + 10% | **3 + 4%** |
| Haymaker | 40 + 50% | **15 + 20%** |
| Shadow Break | 3% max HP + 15% Attack | **1% + 5%** |

- **His skills now:** mostly utility (slows, pulls, knock-ups, the yank and the Flurry's knock-ups); his damage comes from his basic attacks and stacks.
- **Small echoes:** echoes and copies of small skill hits round down to 0 damage, but still carry their crowd control.

**Measured (AI 1v2, 20 s):**

| | 2.7 | 2.8 |
|---|---|---|
| Level 3 | 1.55× | **1.26×** |
| Level 9 (with one ult) | 1.69× | **1.32×** |
| Basics alone | 1.11× | **1.04×** |
| Enemy hits dodged | 9% | **9%** (unchanged) |

**Checks:**
- **Native tests:** 1,420 passed. The fight guard is tightened to ≤ 1.4×, and two tests use a higher Attack so the small echoes and copies aren't rounded to 0.
- **Lab and Mod Manager:** 3 Lab transport tests and 15 Mod Manager tests pass.
- **Generators:** current.
- **Builds:** the wasm and the DLL are rebuilt.
