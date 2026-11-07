"""Grimhald body art, drawn in code (blocky pixel style, 1 px dark outline).

Frame 64x56, centre (32, 28), feet on y = 48 (20 px below the centre, like the template).
Facing right. Every pose is built from the same parts so approved pixels stay stable.
"""
from PIL import Image, ImageDraw

W, H = 64, 56
CX, CY = 32, 28
FEET = 48

PAL = {
    "out": (20, 16, 20, 255),
    "iron0": (44, 44, 52, 255),    # deep shadow
    "iron1": (70, 70, 82, 255),    # base plate
    "iron2": (104, 104, 118, 255), # light plate
    "iron3": (150, 150, 162, 255), # edge highlight
    "red0": (92, 18, 22, 255),
    "red1": (140, 28, 30, 255),
    "red2": (186, 52, 44, 255),
    "ember": (255, 140, 40, 255),
    "ember2": (255, 214, 120, 255),
    "wood0": (64, 40, 28, 255),
    "wood1": (98, 64, 40, 255),
    "steel1": (150, 156, 166, 255),
    "steel2": (196, 202, 210, 255),
    "steel3": (236, 240, 244, 255),
    "skin": (0, 0, 0, 0),
}


class Canvas:
    def __init__(self, w=W, h=H):
        self.im = Image.new("RGBA", (w, h), (0, 0, 0, 0))
        self.d = ImageDraw.Draw(self.im)

    def rect(self, x0, y0, x1, y1, c):
        self.d.rectangle((x0, y0, x1, y1), fill=PAL[c])

    def poly(self, pts, c):
        self.d.polygon(pts, fill=PAL[c])

    def px(self, x, y, c):
        if 0 <= x < self.im.width and 0 <= y < self.im.height:
            self.im.putpixel((x, y), PAL[c])

    def line(self, pts, c, w=1):
        self.d.line(pts, fill=PAL[c], width=w)

    def outline(self):
        src = self.im.copy()
        w, h = src.size
        a = src.load()
        o = self.im.load()
        for y in range(h):
            for x in range(w):
                if a[x, y][3]:
                    continue
                for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                    nx, ny = x + dx, y + dy
                    if 0 <= nx < w and 0 <= ny < h and a[nx, ny][3] and a[nx, ny] != PAL["out"]:
                        o[x, y] = PAL["out"]
                        break
        return self


def axe(c: Canvas, hx, hy, tx, ty, glow=False):
    """Haft from the hand (hx, hy) to the head end (tx, ty); a big crescent blade at the head end, facing back."""
    c.line([(hx, hy), (tx, ty)], "wood0", 3)
    c.line([(hx, hy - 1), (tx, ty - 1)], "wood1", 1)
    c.rect(hx + 1, hy + 1, hx + 3, hy + 3, "iron2")                     # pommel
    bx, by = tx, ty
    # crescent: tall curved edge on the far side, concave throat toward the haft
    c.poly([(bx + 1, by - 3), (bx - 4, by - 9), (bx - 9, by - 11), (bx - 12, by - 8), (bx - 13, by - 2),
            (bx - 13, by + 4), (bx - 11, by + 10), (bx - 7, by + 13), (bx - 3, by + 12), (bx - 5, by + 7),
            (bx - 6, by + 2), (bx - 4, by - 2)], "steel1")
    c.poly([(bx - 9, by - 9), (bx - 12, by - 6), (bx - 12, by + 5), (bx - 10, by + 9), (bx - 9, by + 2),
            (bx - 9, by - 4)], "steel2")
    c.line([(bx - 9, by - 11), (bx - 12, by - 8), (bx - 13, by - 2), (bx - 13, by + 4), (bx - 11, by + 10),
            (bx - 7, by + 13)], "steel3")
    # back spike and socket
    c.poly([(bx + 2, by - 3), (bx + 9, by - 1), (bx + 2, by + 2)], "iron2")
    c.rect(bx - 2, by - 3, bx + 2, by + 3, "red1")
    c.rect(bx - 1, by - 2, bx + 1, by - 2, "red2")
    c.px(bx, by, "ember" if glow else "red2")
    # blood-red rune groove along the blade
    for (x, y) in ((bx - 8, by - 3), (bx - 9, by), (bx - 9, by + 3), (bx - 8, by + 6)):
        c.px(x, y, "ember" if glow else "red1")


def body(c: Canvas, lean=0, step=0, glow=1):
    """The bulky armoured body (no axe). lean shifts the upper body; step spreads the legs."""
    L = lean
    # ---- legs (greaves) and boots
    lx, rx = 23 - step, 34 + step
    c.rect(lx, 37, lx + 6, FEET - 2, "iron1"); c.rect(lx, 37, lx + 1, FEET - 2, "iron2")
    c.rect(rx, 37, rx + 6, FEET - 2, "iron1"); c.rect(rx, 37, rx + 1, FEET - 2, "iron2")
    c.rect(lx - 1, FEET - 2, lx + 7, FEET, "iron0"); c.rect(rx - 1, FEET - 2, rx + 8, FEET, "iron0")
    c.rect(lx + 1, 41, lx + 5, 42, "iron2"); c.rect(rx + 1, 41, rx + 5, 42, "iron2")  # knee plates
    if glow >= 2:
        c.px(lx + 3, 40, "ember"); c.px(rx + 3, 40, "ember")
    # ---- crimson tasset (cloth) with iron plates
    c.poly([(19 + L, 32), (45 + L, 32), (43, 40), (37, 38), (32, 41), (27, 38), (21, 40)], "red1")
    c.poly([(21 + L, 32), (25 + L, 32), (24, 38), (21, 40)], "red0")
    c.rect(27 + L, 32, 37 + L, 35, "iron1")
    c.rect(29 + L, 33, 35 + L, 34, "iron2")
    # ---- torso: wide chest plate
    c.poly([(18 + L, 19), (46 + L, 19), (44 + L, 33), (20 + L, 33)], "iron1")
    c.poly([(20 + L, 20), (30 + L, 20), (29 + L, 31), (21 + L, 31)], "iron2")
    c.line([(32 + L, 20), (32 + L, 32)], "iron0")             # keel
    c.line([(20 + L, 26), (44 + L, 26)], "iron0")             # plate seam
    c.rect(19 + L, 29, 45 + L, 30, "red1")                    # belt sash
    c.rect(30 + L, 28, 34 + L, 31, "iron3")                   # buckle
    c.px(32 + L, 29, "ember" if glow else "red2")
    # ---- back pauldron (behind, darker) and arm
    c.poly([(10 + L, 16), (22 + L, 13), (25 + L, 22), (13 + L, 25), (9 + L, 21)], "iron0")
    c.poly([(11 + L, 16), (21 + L, 14), (22 + L, 17), (12 + L, 19)], "iron1")
    for sx in (12, 16, 20):
        c.px(sx + L, 13 - (sx % 3 == 0), "iron2"); c.px(sx + L, 14, "iron1")
    c.rect(13 + L, 24, 18 + L, 31, "iron0")                   # back arm
    # ---- helmet: closed full helm, spiked crown ridge, glowing eye slits
    hx = 25 + L
    c.rect(hx, 6, hx + 12, 18, "iron1")
    c.rect(hx, 6, hx + 3, 18, "iron2")
    c.rect(hx + 1, 16, hx + 11, 19, "iron0")                  # gorget
    for i, sx in enumerate((hx + 1, hx + 4, hx + 7, hx + 10)):
        hgt = 3 + (i in (1, 2)) * 2
        c.poly([(sx, 6), (sx + 2, 6), (sx + 1, 6 - hgt)], "iron2" if i < 2 else "iron1")
    c.rect(hx + 5, 10, hx + 11, 11, "out")                    # visor slit (facing right)
    c.rect(hx + 7, 10, hx + 10, 10, "ember")
    c.px(hx + 11, 10, "ember2")
    c.rect(hx + 8, 13, hx + 11, 13, "iron0")                  # breath grille
    c.rect(hx + 8, 15, hx + 11, 15, "iron0")
    c.rect(hx + 12, 9, hx + 13, 14, "iron0")                  # face plate jut
    # ---- front pauldron: huge, layered, spiked
    c.poly([(36 + L, 14), (50 + L, 13), (55 + L, 20), (52 + L, 26), (38 + L, 24)], "iron1")
    c.poly([(37 + L, 15), (48 + L, 14), (50 + L, 18), (39 + L, 19)], "iron2")
    c.line([(38 + L, 21), (53 + L, 22)], "iron0")
    c.line([(37 + L, 15), (49 + L, 14)], "iron3")
    c.rect(38 + L, 24, 52 + L, 25, "red1")                    # cloth under the plate
    for sx in (40, 45, 50):
        c.poly([(sx + L, 13), (sx + 2 + L, 13), (sx + 1 + L, 9)], "iron2")
    if glow >= 1:
        c.px(44 + L, 20, "ember")
    if glow >= 3:
        c.px(25 + L, 22, "ember"); c.px(39 + L, 27, "ember"); c.px(hx + 6, 17, "ember")


def front_arm(c: Canvas, hand, L=0):
    """Thick gauntleted arm from the shoulder (~45, 24) to the hand position."""
    sx, sy = 45 + L, 24
    c.line([(sx, sy), hand], "iron1", 6)
    c.line([(sx - 1, sy - 1), (hand[0] - 1, hand[1] - 2)], "iron2", 2)
    c.rect(hand[0] - 3, hand[1] - 3, hand[0] + 3, hand[1] + 3, "iron0")   # gauntlet
    c.rect(hand[0] - 2, hand[1] - 2, hand[0] + 1, hand[1] - 1, "iron2")


def aura(im, tier):
    """Bloodrage tier ring: a crimson rim one pixel outside the outline (tier 2 partial, tier 3 full)."""
    if tier < 2:
        return im
    out = im.copy(); a = im.load(); o = out.load()
    col = (200, 30, 30, 150) if tier == 2 else (255, 60, 30, 210)
    for y in range(im.height):
        for x in range(im.width):
            if a[x, y][3]:
                continue
            if tier == 2 and (x + y) % 3:
                continue
            for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                nx, ny = x + dx, y + dy
                if 0 <= nx < im.width and 0 <= ny < im.height and a[nx, ny] == PAL["out"]:
                    o[x, y] = col
                    break
    return out


def idle(glow=1, breathe=0):
    c = Canvas()
    # axe on the shoulder: haft from the front hand up and back over the head, blade high behind him
    body(c, lean=0, glow=glow)
    axe(c, 47, 34 + breathe, 17, 12 + breathe, glow=glow >= 2)
    front_arm(c, (46, 31 + breathe))
    return aura(c.outline().im, glow)


def concept_sheet(out_path, compare=None):
    """Static concept: idle at x6, the three Bloodrage tiers, and actual size next to a stock champion."""
    tiers = [idle(1), idle(2), idle(3)]
    big = tiers[0].resize((W * 6, H * 6), Image.NEAREST)
    small = [t.resize((W * 3, H * 3), Image.NEAREST) for t in tiers]
    sheet = Image.new("RGBA", (W * 6 + W * 3 + 60, H * 6 + 40), (54, 74, 60, 255))
    sheet.alpha_composite(big, (10, 20))
    for i, s in enumerate(small):
        sheet.alpha_composite(s, (W * 6 + 40, 20 + i * (H * 2)) if i < 2 else (W * 6 + 40, 20 + H * 4))
    if compare is not None:
        x = W * 6 + 40
        actual = Image.new("RGBA", (W * 2 + 10, H), (0, 0, 0, 0))
        actual.alpha_composite(tiers[0], (0, 0))
        actual.alpha_composite(compare, (W + 10, CY - compare.height // 2))
        actual = actual.resize((actual.width * 2, actual.height * 2), Image.NEAREST)
        sheet = sheet.crop((0, 0, sheet.width, sheet.height + actual.height + 20))
        bg = Image.new("RGBA", sheet.size, (54, 74, 60, 255)); bg.alpha_composite(sheet); sheet = bg
        sheet.alpha_composite(actual, (10, H * 6 + 40))
    sheet.save(out_path)


if __name__ == "__main__":
    import sys
    from pathlib import Path
    root = Path(__file__).resolve().parents[1]
    contender = Image.open(root / "mods/tfm2_custom/champions/tfm2_custom_contender#sheet.png").convert("RGBA").crop((0, 0, 56, 48))
    out = root / "docs/grimhald-art-review"
    out.mkdir(parents=True, exist_ok=True)
    idle(1).save(out / "idle-1x.png")
    concept_sheet(out / "concept.png", contender)
    print("wrote", out / "concept.png")
