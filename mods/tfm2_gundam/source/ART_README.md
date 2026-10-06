# Aegis Zero (Destiny redesign, round 88)

Sources for Aegis Zero, who ships in the consolidated mod `mods/tfm2_custom`. Run all of these from the repo root:

```
python mods/tfm2_gundam/source/build_art.py    # body sheet + gundam vfx sheet -> mods/tfm2_custom, previews here
node   mods/tfm2_gundam/source/build.js        # champion data + his entry in tfm2_custom's champion.i18n
python mods/tfm2_gundam/source/verify_art.py   # kit layout, palette, wing span, every view tag exists
```

## Body (`champions/tfm2_gundam_aegis_zero`)
- 48 x 56 frames on the sprite kit rows (idle 4, run 6, attack 4, skill1 4, skill2 4, ult 4, hit 1, dead 6). The frame centre is his position, his feet are on y 48, and he faces right.
- A tall, athletic knight:
  - white armour, a royal-blue chest and red waist accents
  - gold V-fin, green eyes, red chin
  - large angular pauldrons, a very narrow waist, long legs and big red-soled feet
- The backpack carries folded red mechanical wings (on joints, rising above his shoulders), the Arondight hilt and the thrusters. Gundam first, wings second.
- Poses:
  - attack: a beam-saber cut
  - skill1: the charge, palm glowing cyan
  - skill2: Arondight drawn and swung
  - ult: arms spread, wings starting to open

## Effects (`vfx/gundam`, 1 px = 950 game units)
- `wings_light`: the Wings of Light. 8 frames, buff `gdm_wings`, drawn behind him.
  - Two huge upper wings in a V, built from sharp shards (pale-pink core, pink, magenta, red edges). The shards grow from the opened red mechanical wings out to a sharp contour.
  - Smaller lower projections sweep down and out.
  - About 3x his width. Wings first, Gundam at their centre.
- `wings_fade`: the same wings flickering for the last second of Zero Protection (buff `gdm_fade`).
- `deploy` / `retract`: the wings growing out and closing. One-shot effects, so only one wing visual shows at a time.
- `flare`: the S2 flash during Zero Protection.
- `palm`, `saber`, `sweep`, `charge_start`, `charge_hit`, `wall_hit`, `landing` (45000 radius), `incoming` (on the ally).
- `after_r` / `after_l`: pink afterimages in the charge and the flight.
- `zero_aura`, `ally_aura`: ground rings at foot level. `protect`: the guardian shimmer.

Previews: `pose_review.png` (every body frame, 4x), `wings_review.png` (normal vs fully deployed, 3x) and `aegis_destiny.gif` (idle, ult, deploy, wings, retract).
