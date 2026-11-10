# Kratos: art handoff for Codex

Kratos is the **God of War** (a crossover champion). Draw **original pixel art** in this mod's style, inspired by the character (a pale, ash-grey Spartan with a red tattoo, a beard, the Leviathan Axe and the Blades of Chaos on chains); don't trace or rip game assets. Rules: `rules/kratos.json`; kit: `docs/kratos-guide.md`. The kit, Lab and tests are built; the game skips him until his body sheet exists.

## Body (`mods/tfm2_custom/champions/tfm2_custom_kratos#sheet.png` + `#anim.fanim`)
- Stock melee height, 48×56 template, feet 20 px below the frame centre; 8 frames per animation unless noted; `_l` mirrored copies.
- **Two stances** (he holds one weapon at a time): the **axe** (frost-blue glow) and the **Blades** (fire-orange, chains). If one sheet can't hold both, draw the axe stance and add a blades overlay effect; tell me and I'll wire it.
- Animations: `idle`, `run`, `attack` (axe chop / Blades lash), `skill1` (Heimdall's Sight: an eye glow), `skill2` (Rampage swings / Guard raise / chain throw), `ult` (calls Fenrir; the leap-slam), `hit`, `dead`.

## Effects (`mods/tfm2_custom/vfx/kratos#sheet.png`, anim `asset/tfm2_custom/vfx/kratos`)
Re-sent every tick, single frame each (`_p0`–`_p7` = 8 headings, 0 = east, clockwise):

| tag | what |
|---|---|
| `fenrir_p0`–`p7` | Fenrir, a huge wolf running (about 24,000 wide in world units = ~25 px at gameplay scale) |

Timed effects (frames add up to the time; world sizes at gameplay scale):

| tag | what | ground? |
|---|---|---|
| `cleave`, `cleave_l` | the axe cleave arc (radius 16,000), frost | |
| `sweep`, `sweep_l` | the Blades sweep arc (radius 16,000), fire | |
| `smash` | the frost-and-fire ground smash (radius 22,000) | ground |
| `chain` | the Blades' chain throw (a line up to 55,000 / 90,000) | |
| `chain_hit` | the chain hooking a target | |
| `dash_trail` | the dash arrival | |
| `guard` | Spartan Guard raised (3 s) | |
| `sight` | Heimdall's Sight eye flash / a dodge | |
| `swap_axe`, `swap_blades` | the weapon swap (with the shield's shimmer) | |
| `freeze` | a frozen target (0.6 s) | |
| `knock_up` | a knock-up puff | |
| `fenrir_spawn` | Fenrir appearing | ground |
| `fenrir_slam` | Kratos slamming the wolf back | |
| `fenrir_fade` | Fenrir fading | |

View buffs: `burn` (`krt_burn`), `frozen` (`krt_frozen`), `frost1`, `frost2` (`krt_frost:1`, `krt_frost:2`, the stacks), `sight_aura` (`krt_sight`), `guard_aura` (`krt_guard`).

When the art is in, give an `docs/kratos-art-review/art-manifest.json` like Wright's (`review_only: false` switches the generator to it).
