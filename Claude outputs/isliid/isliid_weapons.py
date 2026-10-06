"""Seven independent Isliid sword sprites with four mastery art tiers.

Each sword keeps the same index in the sheet, flight and ground anchor. The art
tiers change its ornamentation; they do not change its identity or BA property.
"""
from __future__ import annotations

import json
from pathlib import Path

from PIL import Image, ImageDraw

HERE = Path(__file__).resolve().parent
EDITOR = HERE.parents[1] / "editor"
FW, FH = 32, 64
SWORDS = [
    ("skylight", "#d8e9ee", "#7ceaf1"),
    ("terra", "#d7a968", "#e8cf8b"),
    ("darkbringer", "#765692", "#bb91e2"),
    ("gale", "#53c9b9", "#abfff0"),
    ("blood", "#c7435e", "#ff99a6"),
    ("rift", "#4e8ce0", "#a4c7ff"),
    ("emperor", "#f2c65e", "#fff4b1"),
]
INK = "#13182a"
GOLD = "#f3cc77"
LIGHT = "#fffbed"
SHAPES = [
    [(13, 19), (19, 19), (18, 46), (16, 58), (14, 46)],
    [(9, 19), (23, 19), (23, 42), (20, 48), (16, 56), (12, 48), (9, 42)],
    [(11, 19), (19, 19), (22, 27), (18, 32), (21, 39), (16, 58),
     (12, 45), (9, 42), (12, 34), (9, 28)],
    [(14, 19), (20, 19), (19, 31), (22, 39), (17, 58), (13, 49),
     (16, 34)],
    [(10, 19), (22, 19), (20, 29), (23, 37), (16, 57), (9, 37),
     (12, 29)],
    [(10, 19), (14, 19), (14, 40), (16, 55), (18, 40), (18, 19),
     (22, 19), (21, 43), (16, 58), (11, 43)],
    [(11, 20), (21, 20), (20, 38), (16, 53), (12, 38)],
]


def sprite(index: int, tier: int, phase: int) -> Image.Image:
    im = Image.new("RGBA", (FW, FH))
    d = ImageDraw.Draw(im)
    name, metal, glow = SWORDS[index]
    if tier == 2:
        halo = (255, 232, 152, 160 if phase else 90)
        d.arc((2, 10, 30, 62), 188, 352, fill=halo, width=1)
        d.arc((4, 12, 28, 60), 8, 172, fill=(129, 239, 255, 125), width=1)
        d.point((2 + phase, 33), fill=glow)
        d.point((29 - phase, 42), fill=glow)
    elif tier >= 3:
        # Imperial tier: the blade is carrying a small dimensional storm.
        d.arc((-2, 4, 34, 68), 178 + phase * 12, 354 + phase * 12,
              fill=(255, 239, 155, 220), width=2)
        d.arc((1, 7, 31, 65), 0 + phase * 10, 180 + phase * 10,
              fill=(111, 241, 255, 200), width=1)
        for px, py in ((1, 23), (30, 29), (4, 52), (28, 55)):
            d.line([(px, py), (px + (2 if phase else -2), py - 4)],
                   fill=glow, width=1)
    elif tier == 1 and phase:
        d.line([(5, 24), (5, 49)], fill=(247, 217, 139, 90), width=1)

    shape = SHAPES[index]
    d.polygon(shape, fill=metal, outline=INK)
    if index == 5:  # Rift really has a visible spatial split.
        d.line([(16, 25), (16, 47)], fill=INK, width=2)
        d.point((16, 39), fill=glow)
    elif index == 2:
        d.line([(15, 23), (16, 44)], fill=glow, width=1)
        d.line([(19, 30), (13, 40)], fill=INK, width=1)
    elif index == 3:
        d.line([(17, 22), (18, 46)], fill=LIGHT, width=1)
        d.line([(21, 30), (25, 26)], fill=glow, width=1)
    elif index == 4:
        d.line([(16, 23), (16, 47)], fill=INK, width=2)
        d.point((16, 32), fill=glow)
    else:
        d.line([(16, 22), (16, 47)], fill=LIGHT, width=1)

    if index == 0:  # needle guard
        guard = [(7, 17), (16, 19), (25, 17), (24, 21), (16, 20), (8, 21)]
    elif index == 1:  # squared, heavy shoulders
        guard = [(5, 15), (11, 16), (16, 19), (21, 16), (27, 15),
                 (25, 23), (20, 20), (12, 20), (7, 23)]
    elif index == 2:  # dark hooked crossguard
        guard = [(6, 14), (12, 18), (16, 19), (22, 16), (27, 14),
                 (24, 23), (18, 21), (12, 21), (8, 23)]
    elif index == 3:  # wind swept
        guard = [(5, 20), (14, 18), (22, 15), (27, 12), (24, 20),
                 (16, 21), (9, 23)]
    elif index == 4:  # hooked red quillons
        guard = [(6, 13), (10, 17), (16, 20), (22, 17), (26, 13),
                 (25, 23), (19, 21), (13, 21), (7, 23)]
    elif index == 5:  # twin prongs
        guard = [(7, 14), (13, 19), (16, 18), (19, 19), (25, 14),
                 (24, 23), (18, 21), (14, 21), (8, 23)]
    else:  # imperial winged crown
        guard = [(3, 12), (9, 14), (16, 19), (23, 14), (29, 12),
                 (25, 22), (19, 21), (16, 23), (13, 21), (7, 22)]
    d.polygon(guard, fill=INK)
    d.line([(guard[0][0]+1, guard[0][1]+2), (16, 19),
            (guard[4][0]-1, guard[4][1]+2)], fill=GOLD, width=2)
    d.rectangle((14, 7, 18, 17), fill=INK)
    d.rectangle((15, 8, 17, 15), fill=metal)
    d.ellipse((13, 4, 19, 9), fill=INK, outline=GOLD)
    d.point((16, 18), fill=glow)

    if tier >= 1:
        # Engravings and reinforced fins remain unique to each base shape.
        d.point((13, 28), fill=GOLD)
        d.point((19, 35), fill=GOLD)
        d.line([(10, 23), (8, 29)], fill=GOLD, width=1)
        d.line([(22, 23), (24, 29)], fill=GOLD, width=1)
        d.point((16, 12), fill=glow)
    if tier >= 2:
        # Sovereign/Imperial: a bright crown and energy tracery.
        d.line([(6, 9), (10, 13)], fill=glow, width=1)
        d.line([(26, 9), (22, 13)], fill=glow, width=1)
        d.line([(16, 26), (16, 38)], fill=glow, width=1)
        for px, py in ((4, 36), (27, 32), (7, 53)):
            d.point((px + phase, py), fill=glow)
    if tier >= 3:
        # The top rank gets a second crown, split guard and lightning fins.
        d.line([(8, 16), (12, 9), (16, 15), (20, 9), (24, 16)],
               fill=(255, 245, 176), width=2)
        d.point((16, 6), fill=(255, 255, 220))
        d.line([(8, 27), (3, 32), (9, 34)], fill=glow, width=1)
        d.line([(24, 27), (29, 32), (23, 34)], fill=glow, width=1)
        d.line([(13, 42), (10, 47), (14, 46), (11, 52)], fill=glow, width=1)
        d.line([(19, 42), (22, 47), (18, 46), (21, 52)], fill=glow, width=1)
    return im


def main() -> None:
    HERE.mkdir(parents=True, exist_ok=True)
    sheet = Image.new("RGBA", (FW * 14, FH * 4))
    tags = {}
    for tier in range(4):
        for index, (name, _, _) in enumerate(SWORDS):
            frames = []
            for phase in range(2):
                x, y = (index * 2 + phase) * FW, tier * FH
                sheet.alpha_composite(sprite(index, tier, phase), (x, y))
                frames.append({"duration": .16, "data": {
                    "x": x, "y": y, "w": FW, "h": FH}})
            tags[f"{name}_tier{tier}"] = {"frames": frames}
    sheet.save(HERE / "isliid_swords#sheet.png")
    sheet.save(EDITOR / "isliid-weapons.png")
    (HERE / "isliid_swords#anim.fanim").write_text(
        json.dumps({"anims": tags}, indent=2), encoding="utf-8")

    scale = 5
    board = Image.new("RGBA", (FW * 7 * scale + 150, FH * 4 * scale + 95),
                      "#1d2635")
    bd = ImageDraw.Draw(board)
    bd.text((12, 9), "ISLIID  /  SEVEN INDEPENDENT SWORDS", fill=GOLD)
    for tier, label in enumerate(("BASE", "ENGRAVED", "SOVEREIGN", "IMPERIAL")):
        bd.text((12, 55 + tier * FH * scale), label, fill=LIGHT)
        for index, (name, _, _) in enumerate(SWORDS):
            tile = sprite(index, tier, 0).resize(
                (FW*scale, FH*scale), Image.Resampling.NEAREST)
            board.alpha_composite(tile, (150 + index*FW*scale,
                                         40 + tier*FH*scale))
    for index, (name, _, _) in enumerate(SWORDS):
        bd.text((150 + index*FW*scale + 3, 40 + 3*FH*scale + 9),
                name.upper()[:11], fill=LIGHT)
    board.save(HERE / "isliid_swords_review.png")
    print("weapons", len(SWORDS), "tiers", 4, "sheet", sheet.size)


if __name__ == "__main__":
    main()
