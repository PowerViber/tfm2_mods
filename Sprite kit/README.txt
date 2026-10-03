TFM2 SPRITE KIT
===============

Every champion sprite in your game and mods, ready to look at, copy from or draw over.

  ALL SPRITES overview.png        Every champion (the first idle frame, 3x), mod champions first, then the 78 base ones.

  NEW CHAMPION template.png       An empty sheet to draw a new champion on: 48 x 56 px frames, one animation per row.
  NEW CHAMPION template_guide.png The same sheet blown up 4x with every frame boxed and named. In each box:
                                    - yellow cross = the frame's centre (the point the game puts on the champion's position)
                                    - blue line    = where the feet go (20 px below the centre, under Steve's boots)
                                    - faint figure = Steve's idle, for size
                                  Rows: idle 4, run 6, attack 4, skill1 4, skill2 4, ult 4, hit 1, dead 6.
                                  The time under each box is how long that frame shows.
  NEW CHAMPION template.anim.json Where each frame sits on the sheet (keep it if you keep the layout).

  mods\<mod>\champions\           The mod champions (Steve, Minato, Gojo, DIO, David, V1, Vader, Frieren, Fern, Stark).
  mods\<mod>\vfx\                 Their effect sheets (projectiles, explosions, walls, icons...).
  base champions\                 The game's own champion sprites (reference: sizes, poses, animation names).

  For every sheet there are three files:
    <name>.png             the real sheet (draw on this one; keep the transparent background)
    <name>_guide.png       the sheet blown up with every frame boxed and named "<animation> <frame>" and its duration
    <name>.anim.json       the frame list: where each frame is on the sheet and how long it shows

DRAWING TIPS
  - Pixel art at 1x (no smoothing / anti-aliasing to the background). Aseprite, LibreSprite, Piskel or Photoshop with
    nearest-neighbour all work.
  - Keep a 1 px dark outline like the base sprites so the champion reads on any map tile.
  - Face RIGHT; the game mirrors the sprite when the champion walks left.
  - Keep the feet on the blue line in every frame, or the champion will bob up and down.
  - Effects (vfx sheets) are drawn at 1 px = 950 game units; e.g. a 26000-radius area is about 55 px across.

GETTING IT INTO THE GAME
  - Send me the finished PNG (and say which champion it's for); I'll wire it into the mod and the Skill Lab.
  - Or, for single frames: Editor -> Skill Lab -> sprite editor -> "Import PNG" draws a PNG into the selected frame.

REFRESHING THE KIT
  kit.py rebuilds everything above from the mods folder and bundle.game_data (run it again after sprites change);
  template.py rebuilds the NEW CHAMPION template. (Claude runs these for you.)
