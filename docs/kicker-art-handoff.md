# Kicker art hand-off (for Codex)

**Theme:** an original rooftop free-runner street fighter. Streetwear (hoodie or track jacket, cargo pants), **taped bare feet** or wrapped ankles, and a tag-spray can at the hip. Not based on any existing character.

**Until this art exists:** he uses Contender's body sheet. Once his own sheet is in place:
- the generator picks up `mods/tfm2_custom/champions/tfm2_custom_kicker#sheet.png` and `mods/tfm2_custom/vfx/kicker#sheet.png` automatically;
- set `art_v1: 1` in his passive params to turn on his own poses.

## Body (`tfm2_custom_kicker#anim.fanim`)
- **Standard:** idle, move, attack (a snap kick), skill1, skill2, ult, dead.
- **Native poses** (Animation CCs named `kk_<pose>`):
  - `kk_hop_low` (a short, quick leap) and `kk_hop_high` (a long, slower arc, with a ground shadow below);
  - `kk_kick` (wind-up plus kick, about 0.3 s);
  - `kk_throw` (a rock throw or kick).
- **Wall stance:** a crouched perch for standing on a wall (shown while `kk_roof` is on).

## Effects (`kicker#anim.fanim`; the names the generator binds)
- **Ground and landings:**
  - `hop` (a take-off puff);
  - `land` (a dust ring);
  - `drop_in` (a 15,000-radius shock ring on landing);
  - `dash` (a Marked Dash streak).
- **Rocks:** `rock` (a thrown rock) and `rock_hit` (a small burst).
- **Kicks:**
  - `kick` (an impact star);
  - `kick_trail` (the kicked body's flight trail);
  - `wall_slam` (a dent and dust on the wall, for the stun);
  - `skyfall` (a long dive streak).
- **Ult:** `tag` (the Second Wind graffiti tag on the ground, 8 s) and `return` (the snap back to the tag).
- **Buffs on units:**
  - `mark` (the dash mark);
  - `roof` (a faint high-ground outline);
  - `second_wind` (an aura while the tag holds);
  - `bruised0` to `bruised5` (a bruise mark with 0 to 5 pips).

## Budget
- **Atlases:** each at most 2048 px.
- **Memory:** decoded RGBA under 32 MiB for his whole set.
