# Vesper, the Rift Marksman: verification

Versions: native 0.10.40, tfm2_custom 0.2.40. Branch `claude/wonderful-franklin-j3os0v`, built on `main` after PR #29.

## How to reproduce

```
python3 "Claude outputs/vesper/generate.py" --check      # catalogue → vesper_data.rs, vesper-data.js, vesper-catalogue.md
python3 "Claude outputs/vesper/art.py"                    # sheets, body, previews
python3 "Claude outputs/vesper/data.py"                   # tfm2_custom_vesper.data_champion + i18n
python3 tools/verify_vesper.py                            # views, atlases, regeneration, parity
cd native/tfm2_custom_ai && cargo test --release
VESPER_REPORT=1 cargo test --release rank_fixtures_report -- --nocapture
node tools/check_vesper_parity.js
```

## Results

**Native:**
- 245 passed, 0 failed, 3 ignored. The ignored ones are Vesper's two diagnostic dumps (`usage_dump`, `breach_trace`) plus one older test.
- 25 Vesper tests cover:
  - the atlas: a permutation, drift wrap, determinism;
  - chain-efficiency vectors, opening delay, and the charge, rift-count and expiry limits;
  - dimension imbue and all 55 bonuses;
  - breaking: 1 s dwell, chain cut, packet fizzle, +10 strain;
  - each rift type's behaviour, and every technique spending the right rifts and strain;
  - `cc_immune` and per-target cooldowns;
  - Breach: hits both teams, non-lethal, collapses everything, resets strain to 40;
  - the shared damage, control and displacement windows;
  - habits starting at their rank, and identical damage at every rank with hand-placed rifts;
  - the stable-API adapter.

**Builds and checks:**

| Check | Result |
|---|---|
| Linux release build | 0 warnings |
| Windows build (`x86_64-pc-windows-gnu`) | 0 warnings |
| `mods/tfm2_custom_ai/tfm2_custom_ai.dll` SHA-256 | `a22cae358a0cc3617a746f527ec411a6b508dc8cd4c3517cc05743667c4c57aa` (matches the build output) |
| Manager | 13 passed. "Show logs" includes `vesper_log.txt` and `vesper_damage_log.txt` |
| JS/native parity | 50 math rows and 222 simulation rows, 0 mismatches. The Rift Lab's practice arena replays native decisions exactly |

**Art and data** (`tools/verify_vesper.py`):
- 703 views over 22 sheets. All 639 effect names the native passive can format resolve, plus 63 rank and HUD layers and the bolt projectile.
- Every atlas is at most 2048 px, and every animation has 8 frames with a fixed origin.
- Regeneration is byte-identical, including the merged i18n and the editor and doc previews.
- Decoded RGBA estimate: **173.5 MiB**, within the 190 MiB budget.
- The Unified Theory and Coder verifiers still pass after the shared i18n file was rewritten.

**Browser** (Chromium, editor server on port 7272):
- **Rift Lab:**
  - shows "RIFT EQUATIONS MATCH NATIVE";
  - 100 atlas cells, 100 bonus cells, 100 dimensions and 28 technique buttons;
  - a hand-built 3-link chain shows its per-link efficiency, total and bonus;
  - a hand-placed **Kidnap lands** (kidnaps = 1);
  - the rank duel and the 8-rank comparison run;
  - no horizontal overflow at 390 px.
- **Art inspector:** 711 runtime animations load, and every layer of the chosen rank is drawn.
- **Editor:** the Rift Lab button mounts the embedded lab with the real sprites, and the Mastery page lists Vesper.
- **Fix during testing:** the Mastery page said "No athlete has played him yet"; it now says "her" for Vesper.
- **Remaining error:** one 404, the browser's automatic `/favicon.ico` request, which every lab page gets.

## Rank gap: 12 fixture 5v5 teamfights per rank (seeds 1–12; odd seeds start at close range; 60 s cap)

Same seeds, same opponents, same toolbox; only the planner's rank changes.

| Rank | Damage | Efficiency | Breaches | Broken rifts | Useful Kidnaps | Gates engaged | Trap catches | Wins | Average fight |
|---|---|---|---|---|---|---|---|---|---|
| 0 Spark | 22836 | 61.7% | 45 | 19 | 0/16 | 0/4 | 4/22 | 8/12 | 32.0 s |
| 1 Tear | 24604 | 58.3% | 58 | 6 | 0/34 | 4/7 | 3/40 | 10/12 | 32.5 s |
| 2 Rift | 34714 | 65.0% | 2 | 20 | 9/29 | 4/9 | 20/28 | 11/12 | 31.5 s |
| 3 Gate | 36668 | 64.5% | 1 | 11 | 3/27 | 5/6 | 16/36 | 10/12 | 29.3 s |
| 4 Wanderer | 36674 | 69.3% | 2 | 10 | 1/17 | 0/2 | 12/23 | 11/12 | 28.2 s |
| 5 Cartographer | 37596 | 73.2% | 0 | 5 | 26/32 | 0/0 | 1/1 | 11/12 | 29.3 s |
| 6 Riftwalker | 37013 | 72.4% | 0 | 4 | 32/40 | 0/0 | 3/9 | 12/12 | 29.0 s |
| 7 Worldbreaker | 38810 | 72.6% | 0 | 5 | 24/31 | 0/0 | 0/0 | 12/12 | 28.4 s |

What this shows, honestly:
- The gap is large in **decisions**:
  - Breaches: 45 → 0;
  - broken rifts: 19 → 5;
  - useful Kidnaps: 0% → 77%;
  - wins: 8/12 → 12/12, with shorter fights.
- It is smaller in **raw damage**: about **1.7×**, not the 3–5× hoped for in the plan. The 35%-per-2-s damage window and the fixture opponents' HP cap how much a better plan can add.
- Low ranks use more techniques: a Spark uses 25 of 28 and a Worldbreaker 17. Spark misuses the toolbox, while Worldbreaker settles on the ones that work.
- **Kidnaps** become useful from rank 5, where the team-engage habit starts sending enemies *into* her team.
- **Gates and traps:** from rank 5 she opens no Ally Gates and few traps in these fixtures. Her planner finds more value in Kidnaps and chain fire against these opponents: the gate counter falls to 0, and traps reach 0 at Worldbreaker.

The fixture test asserts:
- Worldbreaker damage ≥ 1.4× Spark;
- at least 5× fewer Breaches;
- at least 2× fewer breaks;
- useful Kidnaps ≥ 60% at Worldbreaker vs ≤ 20% at Spark;
- at least as many wins, in shorter fights.

## To check in game
- In the Manager, use Update/Check and confirm 0.10.40 / 0.2.40.
- Watch a Spark and a Worldbreaker Vesper. Check:
  - whether rifts, link threads and packets are readable at game zoom;
  - the aiming pose and the HUD bars;
  - Breach and Kidnap timing.
- Check FPS in a 5v5 with Vesper. She is capped at 5 rifts, 16 packets and 6 effect starts per update.
- If the game's own AI breaks rifts far more or less often than the fixtures, tune `break_ticks` / `break_core` in `catalogue.json` and regenerate.
