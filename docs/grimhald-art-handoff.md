# Grimhald art handoff (for Codex)

Grimhald's kit is built and tested (native code, data, tooltips, roster), but he has **no art yet**. Reihan is
redesigning him with Codex. This page lists exactly what the code and the generated data already expect, so the art
can drop in without touching gameplay.

> **Until both sheets below exist, don't install or merge this branch.** `tfm2_custom_grimhald.data_champion`
> points at them; a champion whose assets are missing is skipped by the game.

## Design brief (Reihan, Nov 7 2026)

- An original heavy axe-warden. The feel draws on three champions, but nothing is copied (no designs, logos or exact colours):
  - **Darius:** a cool axe and armour. One huge crescent-bladed axe on a long haft.
  - **K'Sante:** a big, bulky, top-heavy body with huge pauldrons, about 1.3× the width of a stock melee body.
  - **Mordekaiser:** cool armour and a closed helm with a spiked crown and glowing eye slits.
- Follow the Codex body rules (Notion TFM2 07/08, PRs #51/#53/#57):
  - simple blocky pixels with a 1 px dark outline;
  - the original game-size visible height;
  - transparent padding that keeps the foot anchor;
  - a visible idle on the base sprite, so the roster card isn't blank.
- Claude's concept (`docs/grimhald-art-review/concept.png`, `tools/grimhald_art.py`) is only a reference. It was not approved, and it can be replaced entirely.

## Body sheet

| File | Purpose |
|---|---|
| `mods/tfm2_custom/champions/tfm2_custom_grimhald#sheet.png` | atlas (≤ 2048 px a side) |
| `mods/tfm2_custom/champions/tfm2_custom_grimhald#anim.fanim` | frames: `{"anims": {name: {"frames": [{"duration", "data": {x, y, w, h}}]}}}` |

- **Animations:** the same names and frame counts as the Sprite kit template (`Sprite kit/NEW CHAMPION template.anim.json`):

  | name | frames | used for |
  |---|---|---|
  | `idle` | 4 | standing (axe on shoulder) |
  | `run` | 6 | walking (heavy footfalls) |
  | `attack` | 4 | basic axe swing (melee, range 24,000) |
  | `skill1` | 4 | **Stone Bulwark** slam (native Pose, 0.3 s) |
  | `skill2` | 4 | **Skull Bash** shoulder charge (native Pose, 0.23 s) |
  | `ult` | 4 | **Warden's Verdict** axe throw wind-up (native Pose, 0.35 s) |
  | `hit` | 1 | hurt |
  | `dead` | 6 | death |

- **Pose names:** the native code plays `skill1`, `skill2` and `ult` by name, so keep these names exactly.
- **Frame rules:**
  - Use one symmetric crop per animation, the same for all its frames.
  - The frame centre sits on the champion's position, and the feet go 20 px below it.
  - Face right; the game mirrors the base sprite. No `_l` copies are needed, because the body is not a buff overlay.

## Effects sheet

| File | Purpose |
|---|---|
| `mods/tfm2_custom/vfx/grimhald#sheet.png` + `#anim.fanim` | every effect and buff layer below |

The view effects are named `grh_<tag>` in the data; the tag is the animation name in the fanim. Scale is 1 px = 950 world units.

| tag | what | size (world units) | note |
|---|---|---|---|
| `axe_p0`–`axe_p7` | flying axe, 8 spin phases | ~20,000 | **single frame each**; re-sent every tick |
| `land` | axe landing shockwave | knock-up radius 36,000 | |
| `cage_block` | pentagon wall block | wall ~16,000 thick | |
| `cage_rise` | pentagon edge rising | edge ~64,600 long (radius 55,000) | |
| `fissure` | fissure crack piece | 8,000 wide; pieces every 7,600 along 38,000 | ground (z −2) |
| `pillar` | rock pillar | 30,000 wide, 2 blocks | |
| `pillar_rise` | pillar rising / re-rising | | |
| `pillar_break` | pillar shatter | shatter radius 22,000 | |
| `trail` | relocation fissure trail piece | 8,000 wide | ground (z −2) |
| `bash_dust` | Skull Bash dust | | |
| `slam` | wall-slam impact | | |
| `ghost` | Iron Ghost smoke | | |
| `ghost_end` | Iron Ghost reappear puff | | |
| `leap` | leap take-off | | |
| `rage1`–`rage3` | Bloodrage aura buff layers (HP ≤ 80%, 55%, 30%) | around the body | view **buffs**, not effects |

- **Lifetime:** an effect lives for the sum of its frame durations. Multi-frame effects are fine except for `axe_p*`.
- **Budget:** keep the decoded RGBA of all sheets under 32 MiB.

## After the art lands

1. Run `python3 tools/generate-grimhald.py --check`, which must report current. If an effect is added or renamed, change the `effects` list in that tool and regenerate.
2. Run the Mod Manager test `the_repos_champions_all_load` with `cargo test --manifest-path tools/manager/Cargo.toml`.
3. Do the release in one round: versions 0.10.90 / 0.2.92 (or the next unused), the Windows DLL, `docs/main-review-round-N.md`, and the viability harness against Contender, Raze and Rally.
4. Remind Reihan to run `tools/migrate-champion-reworks.py` on careers, then play.
