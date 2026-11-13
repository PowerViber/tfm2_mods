# Round 168: Kicker, the rooftop street fighter (native 0.10.102, tfm2_custom 0.2.105)

**Reihan's brief:** a melee fighter who engages and makes picks by kicking an enemy champion out from under its tower or into a teammate's gank. He stands and runs on walls; a K'Sante-style carry through walls; a Phoenix-style "run it back" ult. Built in one round (Reihan: "do all in one round"). The full kit is in `docs/kicker-guide.md`.

**Reihan's answers:**
- **S3 charges:** hold one for picks.
- **Franchise:** normal rules (a V1 parry cancels the kick and the carry; a cage or Main Event ring blocks them).
- **Personality:** none fixed (the athlete's values drive him; he takes the 1v1 a Skyfall creates).
- **Balance peers:** Kratos (and the Contender reference).
- **Mod Power:** `[15, 0, 5, 10, 10, 0, 5]`.
- **Farming:** S1 and S2 only.

## Kicker at level 10

| | HP | Attack | Sample burst |
|---|---|---|---|
| **Kicker** | about **1,920** | about **172** | Rock 160 · Skyfall 262 (+ Drop-in knock-up) · 2 basics about 344 · Homecoming Kick 259 · Bruised +15% on later hits · full combo about **1,025** |
| Kratos | about 2,000 | about 175 | full combo about 1,120 |
| Contender | about 2,000 | about 228 | |

**How he compares with Kratos:** slightly less HP and burst, but faster (1,150 move speed at level 10). His strength is displacement and picks, not damage.

**Guardrails:**
- the kick is one force-move per cast;
- CC on secondary targets is capped at 2;
- no CC immunity;
- Mod Power HP is 5%;
- his skills never touch towers.

## What's in the round
- **Data:** `rules/kicker.json` and `tools/generate-kicker.py` (`--check`) write `kicker_data.rs`, `tfm2_custom_kicker.data_champion`, the tooltips, `editor/kicker-data.js` and the roster entry (86 champions).
- **Core:** `kicker_core.rs` covers the walls, Drop-in, Marked Dash, Bruised, S1, S2, S3 (both forms) with the Flash combo, Second Wind and his AI.
- **Bridge:** `kicker.rs`.
- **Shared code:**
  - `flash::kit_blink` (a kit's own Flash, with the shared cooldown and blink; the match hook's blink moved into one helper);
  - `steve::WallAi` reads `kk_go:` steers (walks only);
  - `motion_guard_tests` allows one teleport (the ult's return).
- **Lab:** the `native/kicker_lab` crate builds `editor/kicker.wasm`, and `editor/kicker.html` is the Lab page.
- **Bench:** Kicker is in `perf_bench_tests.rs`, at about 18 µs per tick alone on the mock host (Contender 17, Steve 19).
- **Art:** the brief for Codex is `docs/kicker-art-handoff.md`. Until it arrives his body is Contender's sheet and he has no effects drawn.

## Verification
- **Native tests:** `cargo test --release` passes with 0 warnings, including 36 Kicker core and glue tests: walls, Drop-in, marks, Bruised stacks, every kick case, Flash re-aim, Skyfall through walls, parry, Second Wind, and the AI's farming, gank, ult and escape.
- **Lab crates:** `kicker_lab` has 3 tests. In Chromium the Lab page loads with no errors, and the AI throws, dashes and kicks toward a teammate.
- **Generators:** kicker and roster are current. The Mod Manager load test passes.
- **DLL:** the Windows DLL reports 0.10.102.

## Live acceptance (pending)
1. **Walls:**
   - he stays on a wall cell standing still with `ignore_wall`;
   - enemies can target him there;
   - a CC or a drop always lands him on open ground.
2. **Kicks:**
   - the kick and the Skyfall carry move the target as planned;
   - walls stop the kick, and the carry passes through them.
3. **Second Wind:** `undying` holds against towers and true damage, no kill is credited, and the return looks right.
4. **Flash kick:** the Flash + kick combo spends Flash's real cooldown.

## Follow-up after Codex's art (`kicker-1.2`)
Codex integrated the approved body and effects and moved the wall-to-wall hop to **300,000** (`kicker-1.1`). Reihan keeps that range.

**Fixes (Reihan):**
- **He never walks off a wall.**
  - **The bug:** `kicker-1.1` removed the walk-off check, so he kept the wall stance (`ignore_wall`, hidden, 15% reduction) standing on the ground.
  - **Input:** while he holds `kk_roof`, the input layer (`steve::WallAi`) stops any walk at the wall's edge (`kicker::roof_walk`). Attacks and casts still pass through.
  - **Core leash:** any step off a wall is glided straight back onto the nearest wall point within 64,000. Farther than that, the stance ends.
- **Every Marked Dash ends behind its target.** That includes the escape dash to a minion.
- **Every dash gives armor:** Marked Dash, the kick's dash-in, Skyfall, Drop Kick, mount and hops all give a 4% max HP shield for 2 s, not stacked.
- **Flash is not spent carelessly.** The AI's Flash kick needs all of these:
  - a real pick (the target at 40% HP or less, a peel, or a teammate at the landing);
  - his own HP at 40% or more;
  - no tower next to the Flash spot;
  - at most one other enemy near the spot;
  - Judgement 35+.

  Otherwise he holds the kick.

**Codex's art tools:**
- `verify-kicker-art.py` and `package-kicker.py` now read the gameplay revision from the rules (no balance numbers changed).
- `verify-kicker-art.py` re-packs the art, and its reproducibility check only matches on Codex's Windows set-up (Pillow/zlib), so I didn't run it here.

**Verification:**
- **Native tests:** **1,265 passed**, 0 warnings, including 48 Kicker tests. The new ones cover the wall leash, the edge stop, dash armor without stacking, Flash discipline, and the dash ending behind.
- **Other checks:** the `kicker_lab` tests pass and its wasm is rebuilt; the generators are current; the Mod Manager tests (15) pass.
- **DLL:** the Windows DLL reports 0.10.102.
- **Not run here:** the Lab page needs the local game reference.
