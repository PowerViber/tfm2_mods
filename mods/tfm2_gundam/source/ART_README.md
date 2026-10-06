# Aegis Zero sprite-kit art

`build_art.py` draws the character at native pixel resolution. It uses the
project's `Sprite kit/NEW CHAMPION template.png` and copies the first eight
animations and their timing from `NEW CHAMPION template.anim.json`. It does not
read a generated image or another champion's pixels.

## Champion sheet

- Frame: **48 × 56 px**, facing right; the game mirrors leftward movement.
- Anchor: frame center **(24, 28)**; feet settle at **y = 48**.
- Standard rows: idle 4, run 6, attack 4, skill1 4, skill2 4, ult 4, hit 1,
  dead 6, exactly as in the kit.
- Extra rows: `ult_flight` 4, `ult_dive` 4, `ult_land` 4.
- Palette: 15 opaque colors plus transparency, with a one-pixel dark outline.
- The body and folded wings fit the kit frame. The large open wings are a
  separate 128 × 96 px VFX layer so they can persist while ordinary body
  animations play.

## Wing states

The wing VFX contains four distinct mechanical structures: left and right
outer wings above, left and right inner wings below. `deploy` opens them in six
frames; `wings_open` breathes subtly during flight and Zero Protection;
`flare` increases the span during S2; `wings_retract` closes them in six
frames. The native champion state decides which layer is visible.

`pose_review.png` compares normal, open and S2-flare forms at 4× nearest-neighbor
scale. `silhouette_check.png` compares normal and empowered silhouettes without
color. Both are generated from the actual sprite frames, not concept art.

Run from the repository root:

```powershell
python mods/tfm2_gundam/source/build_art.py
```
