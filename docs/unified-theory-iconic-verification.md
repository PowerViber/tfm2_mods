# Round 125 verification: iconic cosmic redesign and mastery difficulty

Native 0.10.39, tfm2_custom 0.2.39. Branch `claude/wonderful-franklin-j3os0v` ([PR #29](https://github.com/PowerViber/tfm2_mods/pull/29)), ported onto the 0.10.38 three-charge redesign from `main` (f4427f9).

## What changed on top of 0.10.38

`main`'s three shared charges, planner bands, release-time aim, cast poses, fixed field boundaries and playable Science Lab are kept. Round 125 adds:

- **Gameplay** (`native/tfm2_custom_ai/src/unified_theory.rs`):
  - chain payoff of +20% per earlier successful stage, up to ×1.8, with the budget cap scaled the same way;
  - Unified resonance: a three-science chain releases a combined `40 + 0.2·max(AP,AD)` strike on a 6 s cooldown;
  - non-lethal lab accidents on critical failure (4% max HP, 30-tick stagger);
  - a mastery bonus inside the existing planner bands;
  - visible setup marks (force vector, forecast, sample/isotope);
  - `accident` and `resonance` diagnostics in the damage log.

  Base outputs and the 35%-max-HP per 2 s window are unchanged.
- **Art** (`Claude outputs/unified_theory/iconic_art.py`, routed through `mastery_art.py`):
  - iconic auras that grow with each rank, and at #1 the other two scientists appear at the shoulders;
  - larger activations whose gestures (chalk E=mc², apple toss, vial) sit on the poses' release hand;
  - headline reskins, larger packets, impacts and completions;
  - field overlays inside the fixed domain boundary;
  - resonance, accident and setup-mark sheets. Oversized atlases split automatically.
- **Database Editor:**
  - `science-combat.js` mirrors chain, accident and resonance, and the renderer draws resonance, accidents and setup marks;
  - lab math and the rank comparison gain accidents, landed chains, payoff and resonance columns;
  - the art inspector previews resonance, accidents and marks;
  - Skill Test plays activation, resonance and accident on the hero;
  - the native log importer shows accident and resonance events;
  - Skill Lab and catalogue text are updated.
- **Fixes along the way:**
  - the scientist tabs in the standalone Science Lab respond to clicks again (`verify_unified_theory_browser` also failed on `main` at this step);
  - `verify_science_charges.cjs` and `verify_science_combat.cjs` fall back to the host toolchain when the Windows `rtk`/GNU toolchain is absent.

## Commands and results

```text
cargo test --offline --release -j4 --manifest-path native/tfm2_custom_ai/Cargo.toml
  test result: ok. 219 passed; 0 failed; 1 ignored (pre-existing Scribble)
rustfmt --check (changed UT files) -> clean; git diff --check -> clean
cargo build --release (Linux) and --target x86_64-pc-windows-gnu -> 0 warnings each
  sha256 f5e17fd99b7b76b3191a278c3805b51b8f16c7ce12a43a56ce64479499296ebb (build == mods/tfm2_custom_ai/tfm2_custom_ai.dll)
cargo test tools/manager -> 13 passed; python3 tools/verify_coder.py -> 558 exact runs, 5500 views / 17 sheets
node tools/verify_science_charges.cjs -> 2400 Rust/JS charge states; 7 diagnostic event types incl. accident/resonance
node tools/verify_science_combat.cjs -> 150 native/JS handler traces; round-125 chain (150/180/210), accident, resonance parity; 50 combinations
python3 tools/verify_unified_theory.py -> 375 charge vectors; 6490 views / 43 sheets <= 2048; byte-identical regeneration;
  decoded RGBA estimate 189.42 MiB (0.10.38: 134.35); 33 growing auras with #1 shoulder figures; resonance/accident/mark sheets
python3 tools/verify_science_runtime_assets.py -> 3471 runtime bindings in 40 atlases
Browser (NODE_PATH=/opt/node-tools/node_modules, CHROMIUM_PATH=/opt/pw-browsers/chromium, editor on 7272,
SCIENCE_LAB_URL=http://127.0.0.1:7272/science.html): verify_unified_theory_browser, verify_science_art_browser,
verify_science_editor_browser, verify_science_playable_browser, verify_science_redesign_browser -> all PASS
```

## Balance on the three-charge tempo

60 s duels, one 1000-HP dummy, AP 400 / AD 300 (`mastery_is_the_power_source_and_never_one_shots`):

| Distance | Student raw (accidents) | Unified Mind raw (resonances) | 0.10.38 Student / Unified Mind |
|---:|---:|---:|---:|
| 20,000 | 3,404 (9) | 3,830 (7) | 3,147 / 2,410 |
| 40,000 | 2,375 (8) | 3,290 (6) | 2,388 / 2,020 |
| 60,000 | 2,445 (6) | 2,690 (6) | 1,873 / 1,770 |
| 80,000 | 1,196 (13) | 2,340 (6) | 1,013 / 1,620 |

- Before this change, Unified Mind dealt less than Student at three of four distances. Now it deals 1.1–2.0× Student, which is 1.44–1.63× its 0.10.38 total; Student changes by 0.99–1.31×.
- This is above the ~1.25× soft target I set, because 0.10.38's Unified Mind spent about 11 s preparing a single long recipe. The knobs are `CHAIN_STEP` and `resonance_output`.
- Every worst 2 s window stays at or below 350 (35%).
- Planner: Unified Mind chose three-science recipes in 599 of 599 picks, Student in 0.
- Crowded host: 5 UT gave 4,592 temporary starts / 20 s with peak 140 live; 4 UT + Coder gave 4,020 with peak 118.

## Known limits

- The decoded memory is an atlas estimate, not measured renderer memory or FPS.
- There is no game installation in the cloud, so loading, real collisions, item procs and crowded-match FPS remain manual checks.
- The Coder comparison relies on damage caps (UT 35% per 2 s vs Coder 30% per 1 s); there is no full Coder-vs-UT simulation.
