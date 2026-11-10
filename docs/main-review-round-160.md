# Round 160: heavy nerfs for Thalassa, Wright and Grimhald (native 0.10.95, tfm2_custom 0.2.98)

You played them and found that Thalassa and Wright dominate with huge damage, and that Grimhald hits hard while also being very tanky. You also asked for Grimhald's cage walls to drop when no enemy is inside.

## Why they were so strong
- **Thalassa and Wright:**
  - They gained **26 AP per level**, while Magnus, our other custom mage, gains 10. Mod Power added **+25% AP** on top. At level 10 they had about **368 AP** (Magnus: 140), and every skill scales on it.
  - Thalassa's harp bolt **bounce preferred champions**: a hit on a minion bounced onto a nearby champion for 50%, so waves were free poke.
- **Grimhald:**
  - Mod Power gave him **+20% HP**, the most of any custom champion (round 140 cut every other champion to 10% or less), and +20% attack.
  - His base HP and growth (1,180 + 110 per level) were above Contender's and Rally's.
  - In the cage he stacked damage *and* sustain: +25% damage, +80% attack speed, 12% lifesteal and 15% damage reduction. Bloodrage added up to +35% damage when low.

## What changed

### Thalassa (`thalassa-1.5`)
| | Before | After |
|---|---|---|
| AP growth per level | 26 | **12** |
| Mod Power AP | +25% | **+10%** |
| Bounce | 50%, preferring champions | **30%, same type only:** a minion hit only bounces to minions and creeps, a champion hit only to champions |
| Trident | 150 + 90% AP | **120 + 70% AP** |
| Tide Twin copies | 50% | **40%** |
| Trident clone damage | 50% | **40%** |

### Wright (`wright-5`)
| | Before | After |
|---|---|---|
| AP growth per level | 26 | **12** |
| Mod Power AP | +25% | **+10%** |
| Jets | one strike per trip home | **its own unit with 3 strikes of ammo**: it picks enemies near itself, then flies home to **rearm for 2 s** |
| Jet speed | 4,800 | **2,400 cruise**, **afterburners 1.5×** on the final 25,000 of an approach |
| Jet reach from him | 80,000 | **110,000** (you asked for a bigger range) |
| Gun strike | 20 + 25% AP | **12 + 14% AP** |
| Rocket (every 3rd strike) | 35 + 40% AP | **24 + 26% AP** |
| Kamikaze | 30 + 35% AP, burn 2% max HP/s | **20 + 22% AP, burn 1% max HP/s** |
| Bomber explosions | 45,000 radius, 40,000 apart, 0.3 s apart | **30,000 radius, 28,000 apart, 0.6 s apart** |
| Bomber wave damage | 45 + 40% AP | **30 + 25% AP** |

The Sky Carrier keeps everything it does, as you said; its jets follow the same new rules.

### Grimhald (`grimhald-3`)
| | Before | After |
|---|---|---|
| Mod Power | `[20, 0, 20, 15, 15, 0, 5]` | **`[15, 0, 5, 10, 10, 0, 5]`** |
| Base HP / growth | 1,180 / 110 | **1,080 / 100** |
| Bloodrage max damage | +35% | **+24%** |
| Cage damage | +25% | **+15%** |
| Cage attack speed | +80% | **+50%** |
| Cage lifesteal | 12% | **8%** |
| Cage damage reduction | 15% | **10%** |
| Cage walls | fell only on the timer | **drop at once when no living enemy champion is inside** |

When the cage empties early, his unarmed buff ends with it. He keeps only the extra time his kills earned (2 s each).

## Where they land at level 10
| | HP | AP / Attack | Sample burst |
|---|---|---|---|
| Thalassa before → after | 1,873 → 1,873 | AP 368 → **185** | bolt + bounce + Tide Pool + trident: about 1,160 → **about 610** (−48%) |
| Wright before → after | 1,793 → 1,793 | AP 368 → **185** | jet strike 112 → **37**; full bomber line 576 → **228** |
| Grimhald before → after | 2,604 → **2,079** | Attack 193 → **185** | lower damage bonuses and much less sustain in the cage |
| Contender (for reference) | about 2,000 | Attack about 228 | |
| Rally (for reference) | about 1,950 | Attack about 198 | |
| Magnus (for reference) | 1,565 | AP 140 | |

## Verification
- `cargo test --release` in `native/tfm2_custom_ai`: 1117 passed, 0 failed, 0 warnings. New tests cover:
  - the cut numbers;
  - same-type bounces (a minion hit never reaches a closer champion);
  - 3 strikes per trip, then home to rearm;
  - cruise and afterburner speeds;
  - the cage dropping its five walls the moment the last enemy inside dies, and holding while one is alive;
  - every bomber warning at the new timings mapping to registered art.
- Lab crates: Thalassa 44, Wright 28 and Grimhald 40 tests passed.
- **Browser tests:**
  - Thalassa: 10 checks passed.
  - Wright: 12 checks passed.
  - Grimhald: 36 checks passed, with 36 native-vs-wasm scenarios and 4,716 matching frames. It ran in Chromium here; the test itself is written for Edge on Windows. It also used to pin the old rules revision, which I fixed.
- The Thalassa and Wright art verifiers accept the new rules.
- Mod Manager `the_repos_champions_all_load` and all generator checks pass.

## Not proven yet (please check in play)
- **Live acceptance:** none of this has been seen in a real match yet.
- **Grimhald's tankiness:** whether he still feels too tanky. His HP at level 10 is now about Contender's.
- **Wright's bomber warning art:** Codex drew the warnings for the old 45,000 radius and 0.3 s timing. Until Codex rebuilds them, the second and third warnings reuse the closest existing art, so they look slightly off in size and length.

## How to get it
1. Merge the PR.
2. Close the game completely, run `TFM2 Mod Manager.exe`, and choose **1 Update everything**.

Careers keep their own copy of champion data: run `tools/migrate-champion-reworks.py` (inspect first, then `--in-place`) so existing careers get the new stats.
