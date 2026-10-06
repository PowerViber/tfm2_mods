"""Generate original Isliid mastery badges.

The badge layer is deliberately separate from the hero and sword sheets, like
the Levi/Scribble mastery VFX. Frames are authored at the editor's native
48x28 pixel size and are meant to be displayed with nearest-neighbour scaling.

Ranks 0..6 are seven different constructions: they do not contain a rank
number and are not recolours of one template. Imperial is a separate crest
family; only its ten progression variants contain a number in the central
command core.
"""
from __future__ import annotations

import json
from pathlib import Path

from PIL import Image, ImageDraw


HERE = Path(__file__).resolve().parent
EDITOR = HERE.parents[1] / "editor"
FW, FH = 48, 28
PHASES = 2
IMPERIAL_LEVELS = 10
NAMES = ["Bearer", "Squire", "Engraver", "Tactician", "Swordmaster", "Regent", "Sovereign"]

# Every tier has its own palette as well as its own silhouette. These are
# visual accents, not a substitute for the construction changes below.
PALETTES = [
    ("#111827", "#7d8b9b", "#dce8ed", "#4d596b", "#7de2dd"),
    ("#171525", "#a98258", "#f1c78b", "#684334", "#a9e7c7"),
    ("#15152b", "#7d62bc", "#d5c4ff", "#453b79", "#f0c981"),
    ("#101c31", "#3f79bd", "#acd9ff", "#244267", "#f3d36b"),
    ("#201326", "#bd4f73", "#ffb7c7", "#5d244d", "#6de5e1"),
    ("#11182d", "#4b67b7", "#c5d5ff", "#293463", "#f5dd8a"),
    ("#141927", "#d2a83e", "#fff1aa", "#6b4d2e", "#69e9e1"),
]
IMPERIAL = ("#111425", "#f4c75d", "#fff4b0", "#7b3f90", "#ff6bd6")
INK = "#0b1020"
WHITE = "#eef9ff"


def new_badge() -> tuple[Image.Image, ImageDraw.ImageDraw]:
    im = Image.new("RGBA", (FW, FH), (0, 0, 0, 0))
    return im, ImageDraw.Draw(im)


def line(d: ImageDraw.ImageDraw, pts, fill, width=1) -> None:
    d.line([(int(x), int(y)) for x, y in pts], fill=fill, width=width)


def poly(d: ImageDraw.ImageDraw, pts, fill, outline=INK) -> None:
    pts = [(int(x), int(y)) for x, y in pts]
    d.polygon(pts, fill=fill)
    line(d, pts + [pts[0]], outline)


def diamond(d: ImageDraw.ImageDraw, cx, cy, rx, ry, fill, outline=INK) -> None:
    poly(d, [(cx, cy - ry), (cx + rx, cy), (cx, cy + ry), (cx - rx, cy)], fill, outline)


def node(d: ImageDraw.ImageDraw, x, y, fill, outline=INK, r=2) -> None:
    d.rectangle((x - r, y - r, x + r, y + r), fill=outline)
    d.rectangle((x - r + 1, y - r + 1, x + r - 1, y + r - 1), fill=fill)


def sparkle(d: ImageDraw.ImageDraw, x, y, col, phase, size=1) -> None:
    if phase:
        line(d, [(x - size, y), (x + size, y)], col)
        line(d, [(x, y - size), (x, y + size)], col)
    else:
        d.point((x, y), fill=col)


def common_ground(d: ImageDraw.ImageDraw, pal) -> None:
    d.rectangle((21, 25, 26, 26), fill=pal[0])


def bearer(phase: int) -> Image.Image:
    im, d = new_badge()
    edge, fill, hi, shade, accent = PALETTES[0]
    common_ground(d, PALETTES[0])
    d.rectangle((22, 3, 25, 5), fill=edge)
    d.rectangle((23, 2, 24, 3), fill=hi)
    diamond(d, 24, 13, 7, 9, fill, edge)
    diamond(d, 24, 13, 4, 6, shade, edge)
    line(d, [(24, 7), (24, 20)], hi)
    line(d, [(18, 13), (21, 13)], accent)
    line(d, [(27, 13), (30, 13)], accent)
    if phase:
        d.point((24, 10), fill=WHITE)
    return im


def squire(phase: int) -> Image.Image:
    im, d = new_badge()
    edge, fill, hi, shade, accent = PALETTES[1]
    common_ground(d, PALETTES[1])
    poly(d, [(9, 8), (17, 5), (22, 9), (24, 15), (20, 22), (14, 18)], fill, edge)
    poly(d, [(39, 8), (31, 5), (26, 9), (24, 15), (28, 22), (34, 18)], fill, edge)
    diamond(d, 24, 14, 5, 8, hi, edge)
    d.rectangle((22, 9, 25, 15), fill=shade)
    line(d, [(11, 8), (17, 8), (21, 12)], accent)
    line(d, [(37, 8), (31, 8), (27, 12)], accent)
    if phase:
        sparkle(d, 16, 5, hi, phase)
    return im


def engraver(phase: int) -> Image.Image:
    im, d = new_badge()
    edge, fill, hi, shade, accent = PALETTES[2]
    common_ground(d, PALETTES[2])
    poly(d, [(8, 20), (13, 7), (19, 4), (27, 10), (21, 24)], fill, edge)
    poly(d, [(19, 4), (29, 7), (40, 19), (30, 23), (24, 14)], shade, edge)
    d.rectangle((19, 10, 28, 18), fill=edge)
    d.rectangle((21, 11, 26, 16), fill=hi)
    line(d, [(11, 21), (16, 17), (20, 12), (27, 8)], accent)
    line(d, [(29, 21), (34, 16), (39, 13)], accent)
    if phase:
        d.point((35, 7), fill=hi)
    return im


def tactician(phase: int) -> Image.Image:
    im, d = new_badge()
    edge, fill, hi, shade, accent = PALETTES[3]
    common_ground(d, PALETTES[3])
    poly(d, [(24, 2), (28, 9), (38, 5), (35, 14), (42, 18), (30, 20),
             (24, 26), (18, 20), (6, 22), (13, 14), (7, 7), (20, 9)], fill, edge)
    diamond(d, 24, 14, 5, 5, hi, edge)
    d.rectangle((22, 12, 26, 16), fill=shade)
    d.point((24, 14), fill=accent)
    line(d, [(24, 3), (24, 10)], accent)
    line(d, [(35, 14), (29, 14)], accent)
    if phase:
        sparkle(d, 13, 5, hi, phase)
        sparkle(d, 36, 22, accent, phase)
    return im


def swordmaster(phase: int) -> Image.Image:
    im, d = new_badge()
    edge, fill, hi, shade, accent = PALETTES[4]
    common_ground(d, PALETTES[4])
    poly(d, [(7, 5), (12, 4), (33, 23), (29, 25)], shade, edge)
    poly(d, [(41, 5), (36, 4), (15, 23), (19, 25)], fill, edge)
    line(d, [(10, 7), (36, 23)], hi)
    line(d, [(38, 7), (12, 23)], accent)
    poly(d, [(24, 6), (31, 11), (29, 20), (24, 24), (19, 20), (17, 11)], hi, edge)
    d.rectangle((21, 11, 27, 18), fill=fill)
    line(d, [(24, 8), (24, 22)], accent)
    if phase:
        d.point((10, 4), fill=hi)
        d.point((38, 4), fill=accent)
    return im


def regent(phase: int) -> Image.Image:
    im, d = new_badge()
    edge, fill, hi, shade, accent = PALETTES[5]
    common_ground(d, PALETTES[5])
    poly(d, [(7, 7), (13, 10), (17, 4), (24, 10), (31, 4), (35, 10), (41, 7),
             (36, 15), (31, 13), (27, 24), (24, 27), (21, 24), (17, 13), (12, 15)], fill, edge)
    line(d, [(12, 9), (17, 11), (24, 7), (31, 11), (36, 9)], hi)
    diamond(d, 24, 13, 5, 7, hi, edge)
    d.rectangle((22, 11, 26, 16), fill=shade)
    line(d, [(24, 17), (24, 23)], accent)
    node(d, 8, 18, accent, edge, 2)
    node(d, 40, 18, accent, edge, 2)
    if phase:
        sparkle(d, 15, 3, hi, phase)
        sparkle(d, 33, 3, accent, phase)
    return im


def sovereign(phase: int) -> Image.Image:
    im, d = new_badge()
    edge, fill, hi, shade, accent = PALETTES[6]
    common_ground(d, PALETTES[6])
    line(d, [(4, 17), (7, 8), (15, 3), (24, 1), (33, 3), (41, 8), (44, 17)], edge, 2)
    line(d, [(6, 18), (10, 22), (17, 25)], accent, 1)
    line(d, [(42, 18), (38, 22), (31, 25)], accent, 1)
    poly(d, [(5, 12), (14, 7), (21, 11), (18, 20), (11, 21)], fill, edge)
    poly(d, [(43, 12), (34, 7), (27, 11), (30, 20), (37, 21)], fill, edge)
    poly(d, [(24, 4), (32, 10), (30, 20), (24, 25), (18, 20), (16, 10)], hi, edge)
    d.rectangle((21, 10, 27, 18), fill=shade)
    line(d, [(24, 6), (24, 22)], accent)
    node(d, 8, 8, hi, edge, 2)
    node(d, 40, 8, hi, edge, 2)
    if phase:
        sparkle(d, 4, 4, hi, phase)
        sparkle(d, 44, 4, accent, phase)
    return im


DIGITS = {
    "0": ("111", "101", "101", "101", "111"),
    "1": ("010", "110", "010", "010", "111"),
    "2": ("110", "001", "010", "100", "111"),
    "3": ("110", "001", "010", "001", "110"),
    "4": ("101", "101", "111", "001", "001"),
    "5": ("111", "100", "110", "001", "110"),
    "6": ("011", "100", "111", "101", "111"),
    "7": ("111", "001", "010", "010", "010"),
    "8": ("111", "101", "111", "101", "111"),
    "9": ("111", "101", "111", "001", "110"),
}


def pixel_text(d: ImageDraw.ImageDraw, text: str, x: int, y: int, color: str) -> None:
    for char in text:
        for row, bits in enumerate(DIGITS[char]):
            for col, bit in enumerate(bits):
                if bit == "1":
                    d.point((x + col, y + row), fill=color)
        x += 4


def imperial(level: int, phase: int) -> Image.Image:
    im, d = new_badge()
    edge, fill, hi, shade, accent = IMPERIAL
    rainbow = ["#ff6bd6", "#b983ff", "#62d9ff", "#69f2b1", "#ffe36e"]
    # Imperial is a different family: a crowned command halo around a numbered core.
    line(d, [(4, 18), (4, 9), (10, 5), (17, 2), (24, 4), (31, 2), (38, 5), (44, 9), (44, 18)], edge, 2)
    line(d, [(8, 22), (14, 25), (20, 26)], rainbow[(level + 1) % len(rainbow)])
    line(d, [(40, 22), (34, 25), (28, 26)], rainbow[(level + 3) % len(rainbow)])
    poly(d, [(13, 8), (17, 2), (21, 8), (24, 1), (27, 8), (31, 2), (35, 8),
             (31, 12), (17, 12)], fill, edge)
    poly(d, [(24, 8), (32, 12), (30, 22), (24, 26), (18, 22), (16, 12)], hi, edge)
    d.rectangle((19, 13, 29, 20), fill=shade)
    pixel_text(d, str(level), 22 if level < 10 else 20, 14, WHITE)
    line(d, [(19, 11), (24, 9), (29, 11)], accent)
    node(d, 8, 8, rainbow[level % len(rainbow)], edge, 2)
    node(d, 40, 8, rainbow[(level + 2) % len(rainbow)], edge, 2)
    for i in range(min(4, (level - 1) // 3 + 1)):
        x = 8 + i * 10
        sparkle(d, x, 3 if i % 2 else 24, rainbow[(level + i) % len(rainbow)], phase, 1)
    if phase:
        sparkle(d, 4, 4, hi, phase)
        sparkle(d, 44, 4, accent, phase)
    return im


def frame_for(rank: int, imperial_level: int = 1, phase: int = 0) -> Image.Image:
    if rank < 7:
        return [bearer, squire, engraver, tactician, swordmaster, regent, sovereign][rank](phase)
    return imperial(max(1, min(IMPERIAL_LEVELS, imperial_level)), phase)


def main() -> None:
    families = 7 + IMPERIAL_LEVELS
    sheet = Image.new("RGBA", (FW * families * PHASES, FH))
    anims = {}
    family_names = [f"rank{i}" for i in range(7)] + [f"imperial{i}" for i in range(1, IMPERIAL_LEVELS + 1)]
    for family, name in enumerate(family_names):
        rank = family if family < 7 else 7
        level = 1 if family < 7 else family - 6
        frames = []
        for phase in range(PHASES):
            x = (family * PHASES + phase) * FW
            sheet.alpha_composite(frame_for(rank, level, phase), (x, 0))
            frames.append({"duration": 0.18, "data": {"x": x, "y": 0, "w": FW, "h": FH}})
        anims[name] = {"frames": frames}

    sheet.save(HERE / "isliid_ranks#sheet.png")
    sheet.save(EDITOR / "isliid-ranks.png")
    (HERE / "isliid_ranks#anim.fanim").write_text(json.dumps({"anims": anims}, indent=2), encoding="utf-8")

    # Review board: seven distinct tiers, then Imperial 1, followed by all
    # Imperial subdivisions. Labels sit outside the sprites.
    cell_w, scale = FW * 4, 4
    board = Image.new("RGBA", (9 * cell_w, 2 * (FH * scale + 22) + 28), "#1d2635")
    bd = ImageDraw.Draw(board)
    bd.text((8, 7), "ISLIID / ORIGINAL MASTERY BADGES", fill="#ffe5a0")
    labels = NAMES + ["Imperial 1", "Imperial 2"]
    for i, label in enumerate(labels):
        x = i * cell_w
        rank = 7 if i >= 7 else i
        level = i - 6 if i >= 7 else 1
        board.alpha_composite(frame_for(rank, level, 0).resize((FW * scale, FH * scale), Image.Resampling.NEAREST), (x, 24))
        bd.text((x + 3, 24 + FH * scale + 2), label, fill="#dbe7f2")
    for j, level in enumerate(range(2, IMPERIAL_LEVELS + 1)):
        x = j * cell_w
        board.alpha_composite(imperial(level, 0).resize((FW * scale, FH * scale), Image.Resampling.NEAREST), (x, FH * scale + 54))
        bd.text((x + 3, FH * scale + 54 + FH * scale + 2), f"Imperial {level}", fill="#dbe7f2")
    board.save(HERE / "isliid_ranks_review.png")
    print("original badge families", families, "sheet", sheet.size, "base tiers", 7, "imperial levels", IMPERIAL_LEVELS)


if __name__ == "__main__":
    main()
