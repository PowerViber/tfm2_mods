# Akane: art handoff for Codex

Akane is an **original** assassin. She's slim and quick, crimson and black, with a katana; a light assassin's outfit (no samurai armour), a mask or scarf, and red accents. Rules: `rules/akane.json`; kit: `docs/akane-guide.md`. The kit, Lab and tests are built; the game skips her until her body sheet exists.

## Body (`mods/tfm2_custom/champions/tfm2_custom_akane#sheet.png` + `#anim.fanim`)
- Stock melee height, 48×56 template, feet 20 px below the frame centre; 8 frames per animation unless noted; `_l` mirrored copies.
- Animations:
  - `idle`, `run` (fast, low), `attack` (a quick katana slash);
  - `skill1` (the kunai throw and backflip, and the dash-slash);
  - `skill2` (the Crimson Edge stance and the finale slash);
  - `ult` (calls the night; the night dash-slash);
  - `hit`, `dead`.

## Effects (`mods/tfm2_custom/vfx/akane#sheet.png`, anim `asset/tfm2_custom/vfx/akane`)
| tag | what | ground? |
|---|---|---|
| `orb` | a Soul Orb (crimson soul flame) waiting 15 s; about 7,000 across | ground |
| `orb_absorb` | the orb flowing into her | |
| `dismiss` | she dissolves into crimson mist (1.5 s invisible) | |
| `flip` | the backflip | |
| `kunai` | the kunai in flight (a short line) | |
| `kunai_hit` | the kunai striking | |
| `dash_slash` | the recast dash's slash | |
| `burst` | the crimson burst (18,000 radius) | |
| `finale` | the 5th hit: a vanish, a slash behind the target, a ring (20,000) | |
| `knock_up` | a knock-up puff | |
| `night` | Crimson Night: a dark-red veil around her (4 s) | ground |
| `night_slash` | the night dash's slash | |

View buffs: `mark` (`akn_mark`, the kunai mark), `bleed1`–`bleed4` (`akn_bleed:N`), `crimson` (`akn_crimson`), `vulnerable` (`akn_vulnerable`).

When the art is in, give a `docs/akane-art-review/art-manifest.json` like Wright's (`review_only: false` switches the generator to it).
