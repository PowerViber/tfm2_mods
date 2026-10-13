# Round 125 verification: iconic cosmic redesign and mastery difficulty

Native 0.10.38, tfm2_custom 0.2.38. Branch `claude/wonderful-franklin-j3os0v`.

## What changed

- **Gameplay** (`native/tfm2_custom_ai/src/unified_theory.rs`, `unified_theory_math.rs`):
  - charge regeneration rises from 4/8 to 7/12 CU/s;
  - base outputs are retuned to `40 + 0.22·stat` (ordinary) and `75 + 0.40·stat` (heavy);
  - chain payoff adds 30% per earlier successful stage, up to ×2.2, and the budget cap scales the same way;
  - Unified resonance: a three-science chain releases a combined `70 + 0.5·max(AP,AD)` strike on a 6 s cooldown;
  - critical failures become lab accidents (4% max HP, never lethal, 30-tick stagger);
  - the planner scores long chains and three-science chains by rank;
  - setup marks show the force vector, forecast and sample/isotope. The 35%-max-HP per 2 s window is unchanged.
- **Art** (`Claude outputs/unified_theory/iconic_art.py`, routed through `mastery_art.py`):
  - iconic, brighter auras that grow with each rank;
  - the other two scientists appear at the shoulders at #1;
  - larger activations with gestures (chalk E=mc², apple toss, glowing vial);
  - headline reskins, larger packets, impacts, fields and completions;
  - new resonance, accident and mark sheets. Oversized atlases split automatically.
- **Database Editor:**
  - Science Lab math mirrors native, and its comparison adds accidents, landed chains, payoff and resonance;
  - the trajectory study reports chain payoff, accidents and resonance;
  - the art inspector adds Unified resonance, Lab accident and Setup mark previews on the new 160 × 176 auras;
  - Skill Test gains activation, resonance and accident playback;
  - Skill Lab and catalogue text are updated.

## Commands and results

```text
cargo test --offline --release -j4 --manifest-path native/tfm2_custom_ai/Cargo.toml
test result: ok. 212 passed; 0 failed; 1 ignored (pre-existing Scribble)

cargo build --offline --release -j4 --manifest-path native/tfm2_custom_ai/Cargo.toml                    -> 0 warnings
cargo build --offline --release --target x86_64-pc-windows-gnu -j4 --manifest-path native/tfm2_custom_ai/Cargo.toml -> 0 warnings
sha256 2d3243b6a2a2985bf069dc20bc97b53b7c993d4d0d6961a5a96c2d29beef4801  (build output == mods/tfm2_custom_ai/tfm2_custom_ai.dll)

cargo test --offline --release -j4 --manifest-path tools/manager/Cargo.toml  -> 13 passed (manager source/EXE unchanged)
python3 tools/verify_coder.py  -> 558 runs reproduced exactly; 5500 Coder views over 17 sheets
python3 "Claude outputs/unified_theory/generate.py" --check  -> 75 unique skills; 50 bounded recipes; generated files current
python3 tools/verify_unified_theory.py
  375 exact native charge vectors; all 8 ranks deterministic
  Verified 4756 views over 40 VFX sheets; every atlas <=2048; regeneration byte-identical; decoded RGBA estimate 182.64 MiB
  Iconic auras: 33 eight-frame loops fit 160 x 176 px and grow from Student to #1; #1 shows both shoulder figures
  Round 125: 7 Unified resonance loops, 3 lab accidents and 16 setup marks; black holes limited to Gravity Well and Horizon Ring
NODE_PATH=/opt/node-tools/node_modules CHROMIUM_PATH=/opt/pw-browsers/chromium (editor server on 7272 for the first two)
  node tools/verify_unified_theory_browser.cjs  -> PASS (round-125 parity, lab accident study, new comparison columns, desktop/mobile)
  node tools/verify_science_art_browser.cjs     -> PASS (75 skills x 8 ranks, 33 auras, resonance/accident/mark previews, desktop/mobile)
  node tools/verify_science_editor_browser.cjs  -> PASS (shared Science Lab, 51 rank/form appearances, mastery lists, desktop/mobile)
```

New native tests:
- `chain_percentages_are_bounded`;
- `later_offense_in_a_successful_chain_carries_the_payoff` (128 → 166 → 204 for a Photon Pulse after 0, 1 and 2 successes);
- `a_failed_stage_ends_the_chain_without_payoff`;
- `pick_scales_the_budget_cap_with_the_planned_chain`;
- `critical_failures_are_visible_nonlethal_lab_accidents` (all three scientists, including at 1 HP);
- `ordinary_invalid_commits_only_fizzle`;
- `three_science_chains_release_one_unified_strike_with_a_cooldown`;
- `resonance_stays_inside_the_target_damage_window`;
- `experienced_athletes_seek_three_science_chains`;
- `mastery_is_the_power_source_and_never_one_shots`.

Fixture output:

```text
three-science plans per 599 picks: Student 3, Unified Mind 387
20 s duel raw damage / casts / accidents / resonances / worst 2 s window:
  Student (0, 14, 7, 0, 0), Unified Mind (734, 18, 0, 1, 350)
5 UT 5v5 host stress: 4938 temporary starts/20 s, peak 140 live (round 124: 4566 / 125)
4 UT + Coder 5v5 host stress: 4318 temporary starts/20 s, peak 118 live (round 124: 4021 / 106)
```

The six-starts-per-caster-per-update limit and all replay deadlines are unchanged.

## Known limits

- **Student damage is 0 in the duel.** Its five attacks committed but drifted past a stationary dummy, because the drift from instability is the existing accuracy system. This makes low ranks clearly weaker, as requested. If it feels too punishing in real matches, the knobs are the drift in `execute()` and the jitter/review tables.
- **Coder comparison.** UT damage is limited to 35% of target max HP per 2 s. Coder's limit is 30% per 1 s, so UT's ceiling is lower. A direct Coder-vs-UT DPS run would need a full Coder planner fixture, which this host does not provide.
- **Cast poses** live in the activation effect layer (gesture art). The body sprite is not swapped during casts, which avoids extra persistent buff churn.
- **Untested in game:** decoded memory (182.64 MiB) is an atlas estimate, not measured renderer memory or FPS. There is no game installation in the cloud, so loading, real collisions, item procs and crowded-match FPS remain manual checks.
