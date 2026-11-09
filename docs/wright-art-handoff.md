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
| `lock` | the lock-on reticle (0.5 s), **no longer used** since `wright-4` (jets fly sorties, no lock-on); kept registered so the art pipeline stays valid | |
| `repair_kit` | a repair kit landing on an ally | |
| `drone_up` | the drone taking off from him | |
| `taunt` | the drone's taunt pulse | |

View buffs: `burn` (`wrt_burn`, the kamikaze burn on a target) and `repair_shield` (`wrt_shielded`). Projectile: `rivet` (his basic attack).

## After the art lands
1. `python3 tools/generate-wright.py --check` passes (update its effect list if a tag is renamed).
2. Mod Manager `the_repos_champions_all_load` passes.
3. Release: the next unused native / tfm2_custom versions in all four places, the DLL, and a review doc.

## Resolved after the `wright-2` Lab review
- The rules changed: the bomber's area is now `bomber.radius` 45,000 (replacing `orbitRadius` 40,000 and `bombRadius` 10,000), and the carrier's knockback ring is `carrier.knockRadius` 100,000 (was 70,000). The carrier now hovers over Wright.
- The `wright-art-2` generator now reads the current keys. Bomber warning variants cover both 60-tick and 18-tick marks and retain the 45,000 footprint through all waves. Each wave uses one centred composite with seven decorative blasts, replacing seven separately positioned effects; damage and control are unchanged. Compact `bomb` aliases remain for single-target rocket hits and shot-down planes.
- Carrier shadow, drop and knockback ring now use the 100,000 radius. The descent warning follows Wright, matching the authoritative drop centre.
- Boosted and banking fighter variants reuse approved pixels/palette, select phases from the authoritative plane state and advance from its stable birth tick. They replace the ordinary one-tick aircraft layer; no duplicate aircraft, extra emission or new movement rule is introduced.
- The body, numerical `wright-2` rules, 15% fighter HP and carrier guard model are unchanged. Callback-stage calibration, turret targeting, DLL packaging and live acceptance remain separate pending gates.

## Superseding bomber correction: wright-3 / wright-art-3

Reihan clarified that “like Nana's ult” means **three large, overlapping explosions advancing forward in a line**, not three repeated hits at one point. The earlier seven-small-blast composite is retired for current casts. Each strike now uses one large approved explosion drawing at its own committed centre.

- Aim selects the middle/direction; nominal spacing is 40,000, radius 45,000, and the furthest centre remains within the existing 110,000 range. Map-edge clipping shortens spacing along the original ray; it never folds the line into repeated clamped centres. A zero-length invalid line spends no resources.
- Separate ground markers warn strikes 1/2/3 from acceptance. Normal deadlines are 60/78/96 ticks; ready-carrier bomber deadlines are 18/36/54. Native aliases and Lab selection share the generated manifest. Legacy warning/bomb aliases remain for older presentation.
- The first two explosions retain slow; the third retains stun. Per-strike damage, radius, timings, costs and all other skills are unchanged. This is an intentional bomber geometry change, so old bomber combat traces are not claimed identical.
- Current rules revision is `wright-3`; Lab transport remains v1. Old recordings require their original rules. The Lab review includes an actual-core animated sequence and the three independent footprints.
- The user-confirmed advancing pattern is the design authority. [Nana's published guide](https://www.oneesports.gg/mobile-legends/nana-best-build-guide/) describes three strikes; our spacing, range handling and final-strike stun are TFM2 adaptations, not a claim of exact MLBB mechanics or reviewed video footage.

