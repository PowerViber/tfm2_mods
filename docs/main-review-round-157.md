# Round 157: Thalassa joins the game (native 0.10.92, tfm2_custom 0.2.95)

You designed a mermaid who fights with a harp: a mage carry for mid or bot who can also support. Her water mode comes from pools she makes herself. Her twin copies her skills and rescues stunned allies, and her trident crosses the whole map. We fixed five review findings, Codex drew the art and tuned the playback, and this round puts her in the game.

## Thalassa, the mermaid harpist
She is an Original-family mage with magic damage, no mastery and Mod Power `[0, 25, 5, 5, 10, 0, 10]`. She keeps Flash and farms with Tide Pool and her harp bolts. The full kit is in `docs/thalassa-guide.md`; every number lives in `rules/thalassa.json` (revision `thalassa-1.2`).

| | Skill | In short |
|---|---|---|
| Passive | Tidebound | In water: +20% move speed and range, harp bolts bounce to a second enemy, 1% max HP regen per second. At full HP the regen becomes a shield, up to 10% of her HP. No penalty on land. |
| S1 | Tide Pool | On land, a pool (radius 35,000) forms below her. In water, her pool surges up to 60,000 into a whirlpool that slows, pulls and damages. 2+ enemy champions caught bump and are stunned 1 s, and a new whirlpool forms below her. |
| S2 | Dive | In water, she opens a pool up to 75,000 away and dives across. Into one of her own pools (the trident's too) she dives at any distance. On land, a short slide that leaves her wet. |
| S3 | Tide Twin | A 12 s twin that can be killed. It walks and fires its own bolts (50%) like someone playing her, and copies Tide Pool and Dive only when she casts them (not the ult). An ally (or she) stunned near it gets a rescue pool: cleanse, shield, CC immunity. Spawned on water, it rescues Thalassa at any range instantly, and she can swap with it once when engaged, leaving a whirlpool. |
| Ult | Trident of the Deep | A slow trident with global range, steerable 25° in total. It stuns the first champion hit 3 s (4 s from water) and leaves a pool where it lands. V1 can parry it. |

## Review fixes (since the first draft)
1. **Overheal:** only her own shield that she still has counts toward the 10% cap. A broken shield gets refilled, and it fades about 2 s after she leaves the water.
2. **Far-pool Dive:** she reaches her own pools at any distance, so she can follow the trident. The AI does this when enemies stand in that pool.
3. **V1 parry:** a parried trident deals no stun and no damage to V1. Its splash and pool still happen.
4. **Safe placement:** the twin, its flights and the tide-link swap never land on terrain or walls.
5. **Twin movement:** its flights move through the engine (the round 156 rule). Only the swap is an instant teleport.

## Codex's art and tuning
- Approved pearl-and-seafoam body: 9 animations, both facings. The effects sheet and a live Tide Link that follows her and the twin every tick.
- The twin now copies her exact Dive destination and timing.
- Retune (`thalassa-1.1`):

  | | Was | Now |
  |---|---|---|
  | Pool radius | 25,000 | 35,000 (same as a whirlpool) |
  | Whirlpool scale | 140% | 100% |
  | Dive range | 50,000 | 75,000 |
  | Trident speed | 6,000 | 2,000 (a slower trident) |

## Your follow-up: the twin plays like her (`thalassa-1.2`)
You asked whether the copy moves (it didn't: it stood where it was cast) and for a global self-cleanse from S3 on water.
- **It plays on its own now.** It walks at her move speed (+20% in water) and fires its own harp bolts at 50% on its own cooldown. It targets the nearest enemy champion it can see (a farmable unit when none is near), walks into range, backs off when an enemy gets close, and returns near her when alone. It never strays more than 60,000 from her. Its steps are engine glides (the round 156 rule).
- **Skills still follow her:** it copies Tide Pool and Dive only when she casts them, to the same point, and never the ult. It no longer echoes her bolts; its own attacks replace that.
- **Global self-rescue:** a twin spawned on water rescues Thalassa anywhere on the map, at once: the rescue pool, cleanse, shield and CC immunity appear under her, and the twin is used up. Allies keep the 40,000 range and the flight; a twin spawned on land keeps 40,000 for her too.

## Shipped this round
- Native **0.10.92** (DLL rebuilt for Windows; it reports 0.10.92 and carries her passive) and **tfm2_custom 0.2.95**, which needs 0.10.92 or later.
- 0.2.94 is left for PR #69. Other packs are unchanged.
- README row and pack description.

## Verification
- `cargo test --release` in `native/tfm2_custom_ai`: 1065 passed, 0 failed, 0 warnings.
  - This includes 36 Thalassa tests (7 for the follow-up: own bolts, no echo, kiting, leash and return, skills only with her, the global self-rescue, allies' range): core, mock host, V1 parry, overheal, placement.
  - One run had a Rally tick-budget comparison fail once. It passed alone and on two full reruns. It looks like a flaky test sharing global test state; nothing in it touches Thalassa.
- `native/thalassa_lab` tests: 32 passed. `node tools/test-thalassa-lab.cjs`: 9 browser checks passed, including the dive into the trident's far pool and the twin walking and attacking with no input.
- Codex's art verifier: 1406 checks passed, covering:
  - approved source hashes;
  - frame counts and mirrored facings;
  - every one of her 347 view effects and 2 view buffs bound to a shipped animation;
  - the 32 MiB budget;
  - only the approved rule changes.
  
  Its byte-for-byte repack step needs a font from Codex's machine, so Codex's committed run covers that step. The same goes for the Python Playwright checks in `docs/thalassa-art-review/`.
- Mod Manager `the_repos_champions_all_load`: passed. `generate-thalassa.py --check` and `generate-roster.py --check`: current.

## Not proven yet (please check in play)
- **Live acceptance:** nothing here has been seen in the real game yet.
- **Balance:** no viability harness yet. Tell me how she feels; damage goes down first if she's too strong.
- **The twin:** it's a hidden carrier unit drawn with effect frames. Its look and its glides need checking live.
- **Grimhald's axe** still doesn't ask V1 to parry (separate task).

## How to get her
1. Merge PR #70.
2. Close the game completely, run `TFM2 Mod Manager.exe`, and choose **1 Update everything**.
3. Start the game.

Thalassa appears in exhibition matches and **new careers**. Existing careers keep the champion list they started with. `tools/migrate-champion-reworks.py` updates champions a career already has (inspect first, then `--in-place`), but it can't add new ones.
