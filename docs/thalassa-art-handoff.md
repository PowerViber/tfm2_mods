# Thalassa art handoff (for Codex)

Thalassa's native kit and Lab now have approved original artwork. On November 9, 2026, Reihan approved the pearl-and-seafoam body and water effects. The complete 37-frame body, twin, atlases and actual-asset Lab/review are documented in [the art review](thalassa-art-review/README.md).

> **Release packaging and actual-game acceptance remain pending.** Both sheets now exist; the packaged native DLL still needs the separate release build/version gate before installation.

## Design brief (Reihan, Nov 8 2026)

- **Who she is:** an original **mermaid** who fights with a **harp**. She is a mage carry who can also support.
- **Theme:** water, music, the sea.
- **Her moves:**
  - She makes pools and whirlpools.
  - She dives between pools.
  - She summons a translucent **water twin** of herself.
  - Her ultimate throws a **trident**.
- **Codex body rules (Notion TFM2 07/08):**
  - simple, blocky pixels with a 1 px dark outline;
  - game-size height and a visible idle (roster card);
  - feet 20 px below the frame centre;
  - she faces right, and the game mirrors her.
- On land she still moves (a tail slither or hop). In water she swims; the passive's `water_aura` buff can carry the water look.

## Body sheet

| File | |
|---|---|
| `mods/tfm2_custom/champions/tfm2_custom_thalassa#sheet.png` + `#anim.fanim` | atlas ≤ 2048 px |

| name | frames | used for |
|---|---|---|
| `idle` | 4 | floating, harp held |
| `run` | 6 | slither or swim |
| `attack` | 4 | harp pluck (the bolt leaves the strings) |
| `skill1` | 4 | Tide Pool: a strum, water rising (native Pose, 0.2 s) |
| `skill2` | 4 | Dive (native Pose for the dive or slide) |
| `skill3` | 4 | Tide Twin: summoning her reflection |
| `ult` | 4 | Trident throw wind-up (native Pose, 0.4 s) |
| `hit` | 1 | |
| `dead` | 6 | |

## Effects sheet

`mods/tfm2_custom/vfx/thalassa#sheet.png` + `#anim.fanim`. The view effects are `tls_<tag>`, as listed in the generated data. Scale is 1 px = 950 world units.

| tag | what | size (world units) / note |
|---|---|---|
| `pool` | her water pool (ground, z −2) | radius 25,000 (about 53 px across) |
| `whirlpool` | a spinning whirlpool (ground) | radius 35,000 |
| `pool_rise`, `surge`, `bump` | pool appearing; surge arriving; champions bumping | |
| `bolt` | harp-bolt projectile (view projectile) | |
| `bounce` | a bolt bouncing | |
| `dive_in`, `dive_out`, `slide` | splashes | |
| `twin_spawn`, `twin_fly`, `twin_pool`, `rescue` | the twin's appearance, its flight to a stunned ally, the rescue pool | |
| `twin_p0`–`twin_p7` | **the twin's body**, translucent water Thalassa, **single frame each** | renewed every 6 ticks |
| `tide_link` | the link between her and the twin (ground) | drawn at the midpoint |
| `swap` | the tide-link swap | |
| `trident_p0`–`trident_p7` | the flying trident, **single frame each** | re-sent every tick |
| `trident_hit`, `trident_land` | impact and the pool it leaves | |

| view buff | what |
|---|---|
| `water_aura` (`tls_water`) | her water-mode look (wet sheen, droplets) |
| `rescue_shield` (`tls_rescued`) | a bubble shield on a rescued ally |

## After the art lands
1. `python3 tools/generate-thalassa.py --check` passes. If a tag is renamed, update the generator's `effects` list and regenerate.
2. Mod Manager `the_repos_champions_all_load` passes.
3. Release: native 0.10.92 / tfm2_custom 0.2.95 (0.2.94 is taken by PR #69), the DLL, and the review doc.
