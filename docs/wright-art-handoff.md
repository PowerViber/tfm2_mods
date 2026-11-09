# Wright: art handoff for Codex

Wright is a **modern aircraft engineer** and summoner (original design: no franchise's carrier or characters). Rules: `rules/wright.json`; kit: `docs/wright-guide.md`. The kit, Lab and tests are built; the game skips him until his body sheet exists.

## Body (`mods/tfm2_custom/champions/tfm2_custom_wright#sheet.png` + `#anim.fanim`)
- Stock melee height, 48×56 template, feet 20 px below the frame centre; 8 frames per animation unless noted; `_l` mirrored copies.
- Look: an engineer: work jacket or flight suit, goggles or headset, tool belt, a tablet or remote for his planes; a rivet gun for his basic attack.
- Animations: `idle`, `run`, `attack` (rivet gun), `skill1` (launch gesture), `skill2` (signals a bomber), `skill3` (tablet: calls repairs), `ult` (calls the carrier skyward), `hit`, `dead`.

## Effects (`mods/tfm2_custom/vfx/wright#sheet.png`, anim `asset/tfm2_custom/vfx/wright`)
Re-sent every tick, so **single frame each** (`_p0`–`_p7` = 8 headings, 0 = east, clockwise):

| tag | what |
|---|---|
| `fighter_p0`–`p7` | a small jet fighter (it banks and boosts in dogfights; an afterburner glow reads well on attack runs) |
| `bomber_p0`–`p7` | a heavier bomber (larger) |
| `chopper_p0`–`p7` | a repair helicopter |
| `repair_p0`–`p7` | a repair plane |
| `drone_p0`–`p7` | a small scout drone |
| `rocket_p0`–`p7` | a homing rocket with a trail |
| `carrier` | the hovering carrier over Wright, top-down (about 60,000 × 18,000 world units = ~63 × 19 px) |

Timed effects (frames add up to the time):

| tag | what | ground? |
|---|---|---|
| `carrier_shadow` | its shadow before the drop (1 s) | ground |
| `carrier_land` | the drop impact | |
| `knock_ring` | the shockwave ring (100,000 radius) | ground |
| `warn` | the bomber's ready-up circle (45,000 radius, 1 s, held through the 3 waves) | ground |
| `bomb` | a bomb blast (10,000 radius) | |
| `kamikaze` | a fighter's crash blast (15,000 radius) | |
| `mg` | machine-gun tracers/impacts on a target | |
| `lock` | the lock-on reticle (0.5 s) | |
| `repair_kit` | a repair kit landing on an ally | |
| `drone_up` | the drone taking off from him | |
| `taunt` | the drone's taunt pulse | |

View buffs: `burn` (`wrt_burn`, the kamikaze burn on a target) and `repair_shield` (`wrt_shielded`). Projectile: `rivet` (his basic attack).

## After the art lands
1. `python3 tools/generate-wright.py --check` passes (update its effect list if a tag is renamed).
2. Mod Manager `the_repos_champions_all_load` passes.
3. Release: the next unused native / tfm2_custom versions in all four places, the DLL, and a review doc.

## Note for Codex after the `wright-2` Lab review
- The rules changed: the bomber's area is now `bomber.radius` 45,000 (replacing `orbitRadius` 40,000 and `bombRadius` 10,000), and the carrier's knockback ring is `carrier.knockRadius` 100,000 (was 70,000). The carrier now hovers over Wright.
- `tools/build-wright-art.py` still reads `bomber.bombRadius` and `bomber.orbitRadius`. The shipped sheets were built with the old sizes, so `warn`, `bomb` and `knock_ring` are drawn smaller than the new areas. Please update the script to the new keys and rebuild those effects.

