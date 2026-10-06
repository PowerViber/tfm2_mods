"""Generate the reworked Emperor Isliid sprite sheet and editor previews.

The run animation is a levitating glide: his feet never alternate or touch down.
All art is drawn as original pixel art with Pillow from the user's design notes.
"""
from __future__ import annotations

import json
from pathlib import Path

from PIL import Image, ImageDraw

HERE = Path(__file__).resolve().parent
EDITOR = HERE.parents[1] / "editor"
FW, FH, COLS = 64, 72, 6
ANIMS = [
    ("idle", 4, .20), ("run", 6, .09), ("attack", 4, .08), ("skill1", 4, .09),
    ("skill2", 4, .09), ("ult", 4, .11), ("hit", 1, .14), ("dead", 6, .11),
]
C = {
    "ink": "#111626", "navy": "#273354", "blue": "#425982",
    "white": "#fffaf0", "ivory": "#e3e7e4", "shade": "#a9bdc9",
    "gold": "#ddb458", "lightgold": "#ffe5a0", "skin": "#ead3bd",
    "skinshade": "#bb9d91", "hair": "#171d31", "hairlight": "#485575",
    "cyan": "#41d9e5", "ice": "#b4ffff", "steel": "#adc2d7",
    "blood": "#d65c70", "violet": "#8f79c6",
}
def poly(d, points, color, outline=None):
    d.polygon(points, fill=C.get(color, color), outline=C.get(outline, outline) if outline else None)


def aura(d, phase=0, intense=False):
    gold = (255, 226, 157, 155 if intense else 70)
    pale = (207, 244, 249, 150 if intense else 50)
    d.arc((5, 3, 59, 67), 188 + phase * 4, 345 + phase * 4, fill=gold, width=1)
    d.arc((7, 5, 57, 65), 9 + phase * 3, 177 + phase * 3, fill=pale, width=1)
    d.ellipse((23, 64, 43, 67), outline=(255, 230, 159, 60), width=1)
    for x, y in ((13, 65), (17, 68), (48, 66), (53, 69)):
        d.point((x + phase % 2, y), fill=(205, 225, 230, 105))


def body(d, pose, frame):
    glide = pose == "run"
    bob = (0, -1, -2, -1, 0, 1)[frame % 6] if glide else (0, -1, 0, 1)[frame % 4] if pose == "idle" else 0
    lean = 1 if pose == "attack" and frame >= 2 else 0
    x, y = 32 + lean, bob
    trail = (0, 2, 4, 5, 4, 2)[frame % 6] if glide else (0, 1, 2, 1)[frame % 4]

    # Two long narrow split cape panels, navy outside with pale interiors.
    poly(d, [(x - 9, 25+y), (x - 14, 27+y), (x - 17-trail, 58+y),
             (x - 13-trail, 62+y), (x - 8, 37+y)], "ink")
    poly(d, [(x - 10, 27+y), (x - 13, 29+y), (x - 15-trail, 57+y),
             (x - 13-trail, 59+y), (x - 9, 35+y)], "navy")
    d.line([(x - 12, 30+y), (x - 14-trail, 56+y)], fill=C["ivory"], width=1)
    poly(d, [(x + 9, 25+y), (x + 13, 27+y), (x + 17-trail, 58+y),
             (x + 13-trail, 62+y), (x + 8, 37+y)], "ink")
    poly(d, [(x + 10, 27+y), (x + 12, 29+y), (x + 15-trail, 57+y),
             (x + 13-trail, 59+y), (x + 9, 35+y)], "navy")
    d.line([(x + 12, 30+y), (x + 14-trail, 56+y)], fill=C["ivory"], width=1)
    d.point((x - 13-trail, 60+y), fill=C["gold"])
    d.point((x + 13-trail, 60+y), fill=C["gold"])

    # Long straight legs and pointed white boots. No stepping frames.
    poly(d, [(x - 5, 43+y), (x - 1, 43+y), (x - 2, 60+y),
             (x - 3, 65+y), (x - 7, 65+y), (x - 7, 60+y)], "ink")
    poly(d, [(x + 1, 43+y), (x + 5, 43+y), (x + 7, 60+y),
             (x + 7, 65+y), (x + 3, 65+y), (x + 2, 60+y)], "ink")
    d.rectangle((x - 6, 47+y, x - 3, 62+y), fill=C["white"])
    d.rectangle((x + 3, 47+y, x + 6, 62+y), fill=C["white"])
    d.line([(x - 6, 61+y), (x - 5, 65+y)], fill=C["gold"], width=1)
    d.line([(x + 6, 61+y), (x + 5, 65+y)], fill=C["gold"], width=1)

    # Narrow white coat and pointed tabard with navy, turquoise and gold.
    poly(d, [(x - 8, 25+y), (x + 8, 25+y), (x + 7, 41+y),
             (x + 9, 57+y), (x + 5, 61+y), (x, 52+y),
             (x - 5, 61+y), (x - 9, 57+y), (x - 7, 41+y)], "ink")
    poly(d, [(x - 7, 26+y), (x + 7, 26+y), (x + 6, 41+y),
             (x + 7, 56+y), (x + 5, 58+y), (x, 50+y),
             (x - 5, 58+y), (x - 7, 56+y), (x - 6, 41+y)], "ivory")
    poly(d, [(x - 3, 38+y), (x + 3, 38+y), (x + 3, 54+y),
             (x, 61+y), (x - 3, 54+y)], "navy")
    d.line([(x - 6, 28+y), (x - 5, 54+y), (x - 4, 56+y)], fill=C["gold"], width=1)
    d.line([(x + 6, 28+y), (x + 5, 54+y), (x + 4, 56+y)], fill=C["gold"], width=1)
    d.point((x, 55+y), fill=C["cyan"])

    # Fitted chest and geometric imperial piping at a narrow waist.
    poly(d, [(x - 7, 24+y), (x + 7, 24+y), (x + 5, 39+y),
             (x - 5, 39+y)], "white", "ink")
    poly(d, [(x - 2, 26+y), (x + 2, 26+y), (x + 2, 38+y),
             (x, 41+y), (x - 2, 38+y)], "navy")
    d.line([(x - 6, 27+y), (x - 2, 31+y), (x, 29+y), (x + 2, 31+y),
            (x + 6, 27+y)], fill=C["gold"], width=1)
    d.line([(x - 5, 34+y), (x, 37+y), (x + 5, 34+y)], fill=C["gold"], width=1)
    d.rectangle((x - 4, 38+y, x + 4, 40+y), fill=C["ink"])
    d.rectangle((x - 2, 38+y, x + 2, 41+y), fill=C["gold"])
    d.point((x, 39+y), fill=C["cyan"])
    d.rectangle((x - 1, 28+y, x + 1, 31+y), fill=C["cyan"])
    d.point((x, 27+y), fill=C["ice"])

    # Hands stay down during idle, run and spell poses. A basic attack swings
    # the right arm through four readable positions instead of leaving him in
    # the command pose.
    def lowered(side):
        sx, sy = x + side * 8, 25 + y
        ex, ey = x + side * 13, 40 + y
        d.line([(sx, sy), (ex, ey)], fill=C["ink"], width=5)
        d.line([(sx, sy), (ex, ey)], fill=C["white"], width=3)
        d.line([(ex - side * 2, ey - 2), (ex + side * 2, ey + 2)], fill=C["gold"], width=2)
        d.point((ex + side * 2, ey + 4), fill=C["skin"])
        d.point((ex + side * 3, ey + 3), fill=C["skinshade"])

    if pose == "attack":
        lowered(-1)
        routes = [
            [(x + 8, 25+y), (x + 11, 33+y), (x + 13, 40+y)],
            [(x + 8, 25+y), (x + 14, 29+y), (x + 20, 27+y)],
            [(x + 8, 25+y), (x + 15, 25+y), (x + 24, 20+y)],
            [(x + 8, 25+y), (x + 13, 30+y), (x + 18, 35+y)],
        ][frame % 4]
        d.line(routes, fill=C["ink"], width=5)
        d.line(routes, fill=C["white"], width=3)
        ex, ey = routes[-1]
        d.line([(ex - 2, ey - 2), (ex + 2, ey + 2)], fill=C["gold"], width=2)
        d.point((ex + 2, ey + 4), fill=C["skin"])
        d.point((ex + 3, ey + 3), fill=C["skinshade"])
    else:
        lowered(-1)
        lowered(1)
    # Military epaulettes with short tassels.
    poly(d, [(x - 9, 22+y), (x - 5, 21+y), (x - 5, 25+y),
             (x - 11, 26+y)], "gold", "ink")
    poly(d, [(x + 9, 22+y), (x + 5, 21+y), (x + 5, 25+y),
             (x + 11, 26+y)], "gold", "ink")
    for side in (-1, 1):
        for k in (7, 9, 11):
            d.point((x + side*k, 27+y), fill=C["lightgold"])

    # Young face, layered short navy hair and bright cyan eyes.
    poly(d, [(x - 6, 9+y), (x + 6, 9+y), (x + 7, 19+y),
             (x + 3, 23+y), (x - 4, 22+y), (x - 7, 19+y)], "ink")
    poly(d, [(x - 5, 12+y), (x + 5, 12+y), (x + 5, 19+y),
             (x + 2, 21+y), (x - 4, 20+y)], "skin")
    poly(d, [(x - 8, 13+y), (x - 7, 7+y), (x - 4, 9+y),
             (x - 2, 5+y), (x + 2, 8+y), (x + 6, 6+y),
             (x + 8, 13+y), (x + 5, 12+y), (x + 2, 15+y),
             (x - 1, 13+y), (x - 5, 16+y)], "hair", "ink")
    d.line([(x - 5, 8+y), (x + 2, 9+y), (x + 6, 7+y)],
           fill=C["hairlight"], width=1)
    d.rectangle((x - 4, 16+y, x - 2, 17+y), fill=C["cyan"])
    d.rectangle((x + 2, 16+y, x + 4, 17+y), fill=C["cyan"])
    d.point((x - 2, 16+y), fill=C["ice"])
    d.point((x + 4, 16+y), fill=C["ice"])
    d.line([(x - 1, 20+y), (x + 2, 20+y)], fill=C["skinshade"], width=1)
    # Collar gemstone.
    d.line([(x - 4, 22+y), (x, 25+y), (x + 4, 22+y)], fill=C["gold"], width=1)
    d.point((x, 25+y), fill=C["cyan"])


def frame(pose, n):
    im = Image.new("RGBA", (FW, FH), (0, 0, 0, 0))
    d = ImageDraw.Draw(im)
    aura(d, n, pose == "ult")
    if pose == "dead" and n >= 3:
        d.ellipse((21, 65, 45, 69), fill=(206, 224, 226, 80))
        poly(d, [(20, 57), (44, 59), (49, 63), (21, 65)], "ink")
        poly(d, [(23, 58), (43, 59), (46, 62), (24, 63)], "ivory")
        d.line([(30, 59), (41, 61)], fill=C["gold"], width=1)
        return im
    # The seven swords live in a separate sheet, like Levi's cape layer.
    # Keeping this frame sword-free lets the rendering state hide or plant the
    # exact blade that was used without redrawing the character animation.
    body(d, "hit" if pose == "dead" else pose, n)
    if pose == "run":
        # Horizontal drift, cloth streaming, and an unbroken air gap beneath the feet.
        d.line([(7, 40+n%2), (3, 42+n%2)], fill=C["cyan"], width=1)
        d.line([(15, 54), (8, 58)], fill=C["lightgold"], width=1)
        d.arc((23, 63, 43, 69), 190, 350, fill=C["ice"], width=1)
    elif pose == "attack":
        d.arc((36, 9+n, 62, 48+n), 210, 67, fill=C["lightgold"], width=2)
        d.line([(46, 18+n), (59, 16+n)], fill=C["cyan"], width=1)
    elif pose == "skill1":
        d.line([(37, 39), (57-n*3, 27+n*2)], fill=C["cyan"], width=2)
        d.point((57-n*3, 27+n*2), fill=C["ice"])
    elif pose == "skill2":
        d.line([(57-n*5, 31), (38, 37)], fill=C["ice"], width=1)
        d.point((57-n*5, 31), fill=C["lightgold"])
    elif pose == "ult":
        d.arc((2, 2, 62, 69), 30+n*5, 160+n*5, fill=C["lightgold"], width=1)
        d.arc((5, 5, 59, 66), 200+n*5, 330+n*5, fill=C["ice"], width=1)
    elif pose in ("hit", "dead"):
        d.line([(15, 19), (49, 45)], fill=C["blood"], width=2)
    return im


def main():
    HERE.mkdir(parents=True, exist_ok=True)
    sheet = Image.new("RGBA", (FW*COLS, FH*len(ANIMS)))
    fanim = {"anims": {}}
    for row, (name, count, duration) in enumerate(ANIMS):
        entries = []
        for i in range(count):
            sheet.alpha_composite(frame(name, i), (FW*i, FH*row))
            entries.append({"duration": duration, "data": {
                "x": FW*i, "y": FH*row, "w": FW, "h": FH}})
        fanim["anims"][name] = {"frames": entries}
    sheet.save(HERE / "isliid#sheet.png")
    (HERE / "isliid#anim.fanim").write_text(json.dumps(fanim, indent=2), encoding="utf-8")
    frame("idle", 0).save(EDITOR / "isliid-sprite.png")
    sheet.save(EDITOR / "isliid-sprite-sheet.png")

    zoom = 5
    board = Image.new("RGBA", (FW*COLS*zoom + 145, FH*len(ANIMS)*zoom + 48),
                      "#1d2635")
    bd = ImageDraw.Draw(board)
    bd.text((12, 12), "EMPEROR ISLIID  /  FLOAT REWORK", fill=C["lightgold"])
    for row, (name, _, _) in enumerate(ANIMS):
        bd.text((12, 48 + row*FH*zoom + 10),
                "RUN / FLOAT" if name == "run" else name.upper(), fill=C["ice"])
        strip = sheet.crop((0, row*FH, FW*COLS, (row+1)*FH))
        board.alpha_composite(strip.resize((FW*COLS*zoom, FH*zoom),
                                           Image.Resampling.NEAREST),
                              (145, 48 + row*FH*zoom))
    board.save(HERE / "isliid_sprite_review.png")
    frames = [frame("run", i).resize((FW*5, FH*5), Image.Resampling.NEAREST)
              for i in range(6)]
    frames[0].save(HERE / "isliid_float.gif", save_all=True,
                   append_images=frames[1:], duration=90, loop=0, disposal=2)
    idle = [frame("idle", i).resize((FW*5, FH*5), Image.Resampling.NEAREST)
            for i in range(4)]
    idle[0].save(HERE / "isliid_idle.gif", save_all=True,
                 append_images=idle[1:], duration=200, loop=0, disposal=2)
    print(HERE / "isliid#sheet.png", sheet.size)


if __name__ == "__main__":
    main()
