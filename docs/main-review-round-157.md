# Round 157: Thalassa joins the game (native 0.10.92, tfm2_custom 0.2.95)

You designed a mermaid who fights with a harp: a mage carry for mid or bot who can also support. Her water mode comes from pools she makes herself. Her twin copies her skills and rescues stunned allies, and her trident crosses the whole map. We fixed five review findings, Codex drew the art and tuned the playback, and this round puts her in the game.

## Thalassa, the mermaid harpist
She is an Original-family mage with magic damage, no mastery and Mod Power `[0, 25, 5, 5, 10, 0, 10]`. She keeps Flash and farms with Tide Pool and her harp bolts. The full kit is in `docs/thalassa-guide.md`; every number lives in `rules/thalassa.json` (revision `thalassa-1.1`).

| | Skill | In short |
|---|---|---|
| Passive | Tidebound | In water: +20% move speed and range, harp bolts bounce to a second enemy, 1% max HP regen per second. At full HP the regen becomes a shield, up to 10% of her HP. No penalty on land. |
| S1 | Tide Pool | On land, a pool (radius 35,000) forms below her. In water, her pool surges up to 60,000 into a whirlpool that slows, pulls and damages. 2+ enemy champions caught bump and are stunned 1 s, and a new whirlpool forms below her. |
| S2 | Dive | In water, she opens a pool up to 75,000 away and dives across. Into one of her own pools (the trident's too) she dives at any distance. On land, a short slide that leaves her wet. |
| S3 | Tide Twin | A 12 s twin that can be killed. It copies Tide Pool, Dive and her bolts to the same point (not the ult). An ally (or she) stunned near it gets a rescue pool: cleanse, shield, CC immunity. Spawned on water, she can swap with it once when engaged, leaving a whirlpool. |
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

## Shipped this round
- Native **0.10.92** (DLL rebuilt for Windows; it reports 0.10.92 and carries her passive) and **tfm2_custom 0.2.95**, which needs 0.10.92 or later.
- 0.2.94 is left for PR #69. Other packs are unchanged.
- README row and pack description.

## Verification
- `cargo test --release` in `native/tfm2_custom_ai`: 1058 passed, 0 failed, 0 warnings.
  - This includes 29 Thalassa tests: core, mock host, V1 parry, overheal, placement.
  - One run had a Rally tick-budget comparison fail once. It passed alone and on two full reruns. It looks like a flaky test sharing global test state; nothing in it touches Thalassa.
- `native/thalassa_lab` tests: 25 passed. `node tools/test-thalassa-lab.cjs`: 8 browser checks passed, including the dive into the trident's far pool.
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
