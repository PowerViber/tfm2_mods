# Round 161: Kratos joins the game (native 0.10.96, tfm2_custom 0.2.99)

Reihan: "art and everything is done now put it in the game", and then "speed up fenrir redirect speed just like orns ult".

## What's released
- **Kratos** (God of War family), rules revision **`kratos-3`**:
  - Codex's approved body, weapons and Fenrir art (`mods/tfm2_custom/champions/tfm2_custom_kratos#*`, `vfx/kratos#*`, `vfx/kratos_weapons#*`);
  - Reihan's kratos-2 skill pass (`docs/kratos-skill-review/README.md`): a 1.8 s, 108,000 Rampage; Fenrir from up to 320,000, running through him; an aimed redirect; Dash → Guard taunts.
- **New in this round, a faster Fenrir redirect:** after the slam the wolf flies back at **4,800 a tick, twice its 2,400 charge**, so a full 320,000 return takes about **1.1 s instead of 2.2 s** (`fenrir.returnSpeed`). The hit check sweeps each tick's whole path, so the faster wolf still hits everyone on the line once.
- **Cleanup:** two dead-code warnings in the release build (`valorant_guns::basic_raw`, `Gun::upgrade`, both used only by tests) are now test-only. The Windows build has 0 warnings.

## Kratos at level 10
| | HP | Attack | Sample burst |
|---|---|---|---|
| **Kratos** | about **2,000** | about **175** | Rampage 3 swings about 510 (plus burn about 34/s) · Chain Dash about 118 · Fenrir in about 220 + slam back about 275 · full combo about **1,120** |
| Contender | about 2,000 | about 228 | |
| Rally | about 1,950 | about 198 | |
| Grimhald | about 2,080 | about 185 | |

His HP and Attack sit with the other bruisers (Mod Power `[15, 0, 5, 10, 10, 0, 5]`, no growth above theirs). His strength is control and safety, not damage.

## Flags for Reihan (your requested direction, not changed)
- **Rampage:** 1.8 s of CC immunity and 25% damage reduction while it covers 108,000 with 26,000-radius swings. That's a long, safe engage. It stays inside the shared cap (3 s of immunity in any 6 s), but if he feels unstoppable, this is the first knob (`rampage.travel` or `stageTicks`).
- **Fenrir:** a 320,000 charge plus a fast 320,000 redirect with a 1 s knock-up crosses most of a lane; watch whether it decides fights from off-screen.

## Verification
- `cargo test --release` in `native/tfm2_custom_ai`: **1,152 passed**, 0 failed, 0 warnings. The new test checks that the slammed wolf's return takes half the ticks and still hits a foe on the line once.
- The Windows DLL (`x86_64-pc-windows-gnu`, `SOURCE_DATE_EPOCH`) builds with 0 warnings and reports **0.10.96**.
- `kratos_lab` transport tests: 2 passed. `editor/kratos.wasm` was rebuilt, and a direct check of it shows a 67-tick return for 320,000 under `kratos-3`.
- Generators current: kratos, wright, thalassa, grimhald, roster.
- Mod Manager `the_repos_champions_all_load` passes.
- **Not run here:**
  - Codex's Kratos browser tests (`test-kratos-lab/art/rework/situations.cjs`) now load the installed game's battlefield (`editor/game-reference/`, extracted locally from `bundle.game_data` and not in git), so they need Reihan's PC. Their last recorded runs (kratos-2) passed.
  - The art parity verifiers read those tests' outputs.
  - `check-kratos-repro.py` rebuilds the art; on Linux that rewrote the packaged sheets byte-for-byte differently, so the approved files were restored and nothing from that run is shipped.

## Live acceptance (pending)
- **Taunt:** the actual taunt targeting on Dash → Guard.
- **Fenrir:** the wolf's alignment and the fast return in the real renderer.
- **Chain Dash:** the wall-hop presentation.
- **Heimdall's Sight:** whether healing reduction shrinks its refund.
- **Balance** overall.
