# Round 179: Contender emergency damage nerf (native 0.10.114, tfm2_custom 0.2.117, `contender-2.5`)

**Reihan, after installing 2.4:** "EMERGENCY NERF, HE IS SO OP … EVEN WITH NO ITEM HIS DAMAGE IS CRAZY".

## Measured, with the real core (the Lab wasm)
**Setup:** basic attacks only, level 1, no items, a 1,500 HP champion, 10 s. "Plain" is a champion with the same Attack and attack timer and no passive.

| | Total in 10 s | × plain |
|---|---|---|
| 2.4, not merged | 2,325 | **2.00×** |
| 2.4, merged | 2,670 | **2.29×** |
| 2.5, not merged | 1,290 | **1.24×** (1.11× the old 85-Attack plain baseline) |
| 2.5, merged | 1,396 | **1.34×** |

**Why 2.4 was so strong (the effects multiplied):**
- Echoes repeated 40% of every hit, and they also added Rhythm stacks, so stacks built about twice as fast.
- Full stacks gave +40% damage taken, +30% attack speed and +20% Attack.
- A Break did 6% of max HP and kept his buffs at full for 8 s.
- Merge doubled stacks per hit.
- Tether copies were 30%.
- Base Attack was 85 (+10 per level).

## Changes
| | 2.4 | 2.5 |
|---|---|---|
| Attack / growth | 85 / 10 | **76 / 8** |
| Echo damage | 40% | **20%** |
| Echoes add Rhythm stacks | yes | **no** |
| Damage taken per stack | +2% (+40% at full) | **+1% (+20%)** |
| Attack speed per stack | +1.5% (+30%) | **+1% (+20%)** |
| Attack per stack | +1% (+20%) | **none** |
| Shadow Break | 6% max HP + 30% Attack | **4% + 20%** |
| Buffs held after a Break | 8 s | **5 s** |
| Merge stacks per hit | 2 | **1** |
| Tether copies (flurry) | 30% (15%) | **20% (10%)** |

- **Unchanged:** dodges, cooldowns, crowd control, the 40-stack ult and its punches, and Cyclone.
- **Bug fix:** while merged, a basic attack queues two echoes, but only one of them was given damage; the other dealt 0. Both now deal damage, so "every hit echoes twice" is true again. The merged numbers above include the fix.

## Checks
- **Native tests:** 1,411 passed, 18 ignored. New tests:
  - echoes never add stacks;
  - both merged echoes deal damage;
  - a **DPS guard**: 10 s of basics, merged or not, must stay above 1.0× and at or below 1.4× a plain champion. It reads Attack from the rules file.
- **Lab and Mod Manager:** 2 Lab transport tests and 15 Mod Manager tests pass.
- **Generators:** current.
- **Builds:** the wasm and the DLL are rebuilt; the DLL reports 0.10.114.
- **For Codex:** `tools/package-contender-shadow-art.py` still pins 0.10.113 / 0.2.116 for its packaging record. Refresh it, the parity fixtures and the Windows checks against 2.5.
