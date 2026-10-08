"""Round 103: the Coder's high-rank looks: sheet 'coder_rank' (at most 2048 x 2048).

From Senior up every rank adds a rig around him, as Levi's mantles do (lv_skin): buffs centred on him, drawn behind
(cd_rig<k>, z -1) and in front of him (cd_rigf<k>, z 4). They float around him, so they never fight his animation.

  rig1  Senior     a holo-monitor over his shoulder (violet), a keyboard glow on the ground
  rig2  Staff      two monitors (a graph and code, gold), a gold ring at his feet, data sparks orbiting
  rig3  Architect  four monitors in an arc (cyan), beams to his hands, a circuit-board floor with traces lighting up
  rig4  Root       a monitor wall (red and green), Matrix code rain behind him, a ring of nodes around his hood, a pulse
                   ring underfoot
  rig5  Zero-Day   six black-and-gold monitors showing every colour, rainbow code rain, a crown of { } glyphs orbiting
                   his head, a 0day sigil underfoot, glitch tears flickering over him

Effects that grow with rank (Architect and up, coder.rs HI_FX): fx_<name>_hi, the effect at twice the size with a
gold trim, a turning ring of hex digits, sparks and a red / cyan ghost.

Every canvas is centred on him: rig layers 96 x 112 (his head at (48, 46), his feet at y 75).

Run from the repo root: python3 "Claude outputs/coder/coder_ranks.py" [--preview]
"""
import json
import math
import os
import random
import sys

from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, '..', 'levi'))
sys.path.insert(0, HERE)
from levi_vfx import Cv, rgba, hsv, glow, star4  # noqa: E402
import coder_vfx as V  # noqa: E402
from coder_vfx import glyph  # noqa: E402

ROOT = os.path.abspath(os.path.join(HERE, '..', '..'))
OUT = os.path.join(ROOT, 'mods', 'tfm2_custom', 'vfx')

RW, RH = 96, 112
HX, HY = 48, 46          # his head
FY = 75                  # his feet
N = 8                    # frames a loop

# rig palettes: rim, rim light, content
VIOLET = (rgba('#a77bff'), rgba('#efe2ff'))
GOLD = (rgba('#ffd25a'), rgba('#fff4c8'))
TEAL = (rgba('#55e0f0'), rgba('#e0fcff'))
RED = rgba('#ff4d6d')
GREEN = rgba('#6eb9ff')        # round 106: the monitors' text is the neutral UI blue (the old name kept)
GREEN_L = rgba('#e4f2ff')
WHITE = (255, 255, 255, 255)
RAIN = '01{}<>/;=#$*&[]()'


def a(c, al):
    return c[:3] + (int(max(0, min(255, al))),)


# ------------------------------------------------------------------ parts

def monitor(cv, cx, cy, w, h, rim, f, kind='code', seed=0, content=None, alpha=1.0):
    """A floating holo-monitor: a dark glass panel, a lit rim and title bar, its content scrolling with the frame."""
    x0, y0 = int(cx - w / 2), int(cy - h / 2)
    rnd = random.Random(seed)
    for y in range(h):
        for x in range(w):
            cv.add(x0 + x, y0 + y, (8, 12, 20, int(165 * alpha)))
    for x in range(w):
        cv.add(x0 + x, y0, a(rim[0], 255 * alpha))
        cv.add(x0 + x, y0 + h - 1, a(rim[0], 150 * alpha))
    for y in range(h):
        cv.add(x0, y0 + y, a(rim[0], 200 * alpha))
        cv.add(x0 + w - 1, y0 + y, a(rim[0], 200 * alpha))
    cv.add(x0 + w - 3, y0 + 1, a(rim[1], 230 * alpha))
    inner = (x0 + 2, y0 + 2, w - 4, h - 4)
    if kind == 'code':
        # lines of code scrolling up: coloured runs, indents
        rows = inner[3] // 2
        for r in range(rows):
            k = r + f
            rr = random.Random(seed * 101 + k)
            ind = rr.choice((0, 0, 2, 2, 4))
            ln = rr.randint(2, max(3, inner[2] - ind))
            col = content(k) if content else rr.choice((GREEN, rgba('#6ee0ff'), rgba('#ffb45a'), rgba('#ffe678')))
            for x in range(ln):
                if rr.random() < 0.85:
                    cv.add(inner[0] + ind + x, inner[1] + r * 2, a(col, 220 * alpha))
    elif kind == 'graph':
        pts = []
        for x in range(inner[2]):
            v = math.sin((x + f * 2) * 0.55 + seed) * 0.35 + math.sin((x + f * 2) * 0.21) * 0.3 + 0.5
            pts.append((inner[0] + x, inner[1] + inner[3] - 1 - v * (inner[3] - 1)))
        for (x0_, y0_), (x1_, y1_) in zip(pts, pts[1:]):
            cv.line(x0_, y0_, x1_, y1_, a(content(0) if content else rim[1], 240 * alpha))
        for x in range(0, inner[2], 3):
            cv.add(inner[0] + x, inner[1] + inner[3] - 1, a(rim[0], 90 * alpha))
    elif kind == 'term':
        # a prompt and a blinking block
        for r in range(inner[3] // 3):
            rr = random.Random(seed * 7 + r + f // 2)
            cv.add(inner[0], inner[1] + r * 3, a(GREEN, 230 * alpha))
            for x in range(2, rr.randint(3, inner[2])):
                cv.add(inner[0] + x, inner[1] + r * 3, a(GREEN_L, 160 * alpha))
        if f % 2 == 0:
            for y in range(2):
                cv.add(inner[0] + 3, inner[1] + inner[3] - 2 + y, a(GREEN_L, 255 * alpha))
    # a scanline sweeping down, and the glow it throws
    sy = y0 + 1 + (f * 2 + seed) % max(1, h - 2)
    for x in range(1, w - 1):
        cv.add(x0 + x, sy, a(rim[1], 70 * alpha))
    glow(cv, cx, cy + h / 2 + 1, w / 2, rim[0], int(45 * alpha))
    _ = rnd


def ground(cv, f, col, rx=17, ry=4.5, front=True, ticks=24, spin=1):
    """A ring at his feet; only the half in front of him (front) or behind him (not front)."""
    for j in range(96):
        t = j / 96 * math.tau
        s = math.sin(t)
        if (s >= 0) != front:
            continue
        lit = (j * ticks // 96 + f * spin) % 4 == 0
        cv.add(HX + math.cos(t) * rx, FY + s * ry, a(col, 255 if lit else 150))


def orbit(cv, f, ch_cols, rx=13, ry=3.5, cy=None, front=True, speed=1.0, glyphs=True):
    """Glyphs (or dots) circling his hood; the near half drawn in front of him."""
    cy = HY - 9 if cy is None else cy
    n = len(ch_cols)
    for i, (ch, col) in enumerate(ch_cols):
        t = (i / n + f / N / n * speed) * math.tau
        s = math.sin(t)
        if (s >= 0) != front:
            continue
        x, y = HX + math.cos(t) * rx, cy + s * ry
        al = 255 if front else 150
        if glyphs:
            glyph(cv, ch, x - 2, y - 4, col, al)
        else:
            cv.disc(x, y, 1.0, a(col, al))
            cv.add(x, y - 1, a(WHITE, al * 0.7))


def rain(cv, f, cols, colour, x0=2, x1=92, top=40, span=56, seed=5, alpha=1.0):
    """Matrix code rain behind him: columns of glyphs falling, a bright head and a fading trail; loops over N frames."""
    rnd = random.Random(seed)
    xs = sorted(rnd.sample([x for x in range(x0, x1, 6) if abs(x + 2 - HX) > 13], cols))
    period = span
    for c, x in enumerate(xs):
        off = rnd.randrange(period)
        step = period // N
        head = (off + f * step) % period
        trail = rnd.randint(3, 5)
        for k in range(trail + 1):
            y = head - k * 9
            if y < 0:
                y += period
            if y > span - 8:
                continue
            ch = RAIN[(rnd.randrange(len(RAIN)) + (f if k == 0 else 0) + k * 3) % len(RAIN)]
            col = WHITE if k == 0 else colour(c, k)
            al = (255 if k == 0 else 190 - k * 35) * alpha
            glyph(cv, ch, x, top + y, col, al)


def beam(cv, p0, p1, col, al):
    n = int(max(abs(p1[0] - p0[0]), abs(p1[1] - p0[1]))) + 1
    for i in range(n + 1):
        t = i / n
        if i % 2 == 0:
            cv.add(p0[0] + (p1[0] - p0[0]) * t, p0[1] + (p1[1] - p0[1]) * t, a(col, al))


def skull(cv, cx, cy, col):
    """A 7 x 7 pixel skull."""
    rows = ['.#####.', '#######', '#..#..#', '#######', '.##.##.', '.#####.', '.#.#.#.']
    for y, row in enumerate(rows):
        for x, b in enumerate(row):
            if b == '#':
                cv.put(cx - 3 + x, cy - 3 + y, col)


def tears(cv, f, rnd):
    """Glitch tears over his silhouette (a red / cyan torn scanline pair)."""
    for k in range(3):
        y = HY - 8 + rnd.randint(0, 34)
        w = rnd.randint(8, 18)
        x = HX - w // 2 + rnd.randint(-4, 4)
        for i in range(w):
            cv.add(x + i - 1, y, (255, 60, 90, 150))
            cv.add(x + i + 1, y + 1, (60, 230, 255, 150))


# ------------------------------------------------------------------ the rigs

def rig(k, f, front):
    cv = Cv(RW, RH)
    if k == 1:    # Senior
        if not front:
            monitor(cv, 25, 52, 16, 12, VIOLET, f, 'code', seed=1)
            beam(cv, (33, 56), (44, 61), VIOLET[0], 90)
            ground(cv, f, VIOLET[0], front=False, rx=14, ry=3.5)
        else:
            for j in range(3):   # the key grid glowing on the ground
                for i in range(9):
                    lit = (i * 3 + j + f) % 7 == 0
                    cv.add(HX - 9 + i * 2 + j, FY + 1 + j, a(VIOLET[1] if lit else VIOLET[0], 200 if lit else 90))
            ground(cv, f, VIOLET[0], front=True, rx=14, ry=3.5)
    elif k == 2:  # Staff
        if not front:
            monitor(cv, 22, 52, 18, 13, GOLD, f, 'graph', seed=2, content=lambda _: GREEN)
            monitor(cv, 74, 52, 18, 13, GOLD, f, 'code', seed=3)
            ground(cv, f, GOLD[0], front=False)
            orbit(cv, f, [('*', GOLD[1]), ('*', GOLD[0])], rx=16, ry=4, cy=HY + 10, front=False, glyphs=False)
        else:
            ground(cv, f, GOLD[0], front=True)
            orbit(cv, f, [('*', GOLD[1]), ('*', GOLD[0])], rx=16, ry=4, cy=HY + 10, front=True, glyphs=False)
            if f % 4 == 0:
                star4(cv, HX + 15, FY - 3, GOLD[1])
    elif k == 3:  # Architect
        if not front:
            monitor(cv, 22, 51, 18, 13, TEAL, f, 'code', seed=4)
            monitor(cv, 74, 51, 18, 13, TEAL, f, 'graph', seed=5, content=lambda _: TEAL[1])
            monitor(cv, 9, 66, 12, 9, TEAL, f, 'term', seed=6, alpha=0.85)
            monitor(cv, 87, 66, 12, 9, TEAL, f, 'code', seed=7, alpha=0.85)
            beam(cv, (31, 56), (44, 61), TEAL[0], 110)
            beam(cv, (65, 56), (54, 61), TEAL[0], 110)
            circuit(cv, f, front=False)
        else:
            circuit(cv, f, front=True)
    elif k == 4:  # Root #10..#2
        if not front:
            # round 106: Root's theme is red on black (no green)
            rain(cv, f, 8, lambda c, k_: rgba('#e8eef8') if (c + k_) % 4 else RED, alpha=0.7)
            rr = (RED, rgba('#ffc0cc'))
            gg = (rgba('#c8d2e0'), rgba('#ffffff'))
            monitor(cv, 21, 50, 18, 12, rr, f, 'term', seed=8)
            monitor(cv, 75, 50, 18, 12, gg, f, 'code', seed=9, content=lambda _: rgba('#ffc0cc'))
            monitor(cv, 12, 65, 16, 11, gg, f, 'graph', seed=10, content=lambda _: RED, alpha=0.9)
            monitor(cv, 84, 65, 16, 11, rr, f, 'code', seed=11, content=lambda _: RED, alpha=0.9)
            orbit(cv, f, [('', RED if i % 2 else rgba('#ffffff')) for i in range(10)], front=False, glyphs=False)
            pulse(cv, f, lambda t: RED if t < 0.5 else rgba('#ffd0d8'), front=False)
        else:
            orbit(cv, f, [('', RED if i % 2 else rgba('#ffffff')) for i in range(10)], front=True, glyphs=False)
            pulse(cv, f, lambda t: RED if t < 0.5 else rgba('#ffd0d8'), front=True)
    elif k == 5:  # Zero-Day (#1)
        rnd = random.Random(500 + f)
        if not front:
            rain(cv, f, 10, lambda c, k_: hsv((c / 10 + f / N) % 1, 0.6, 1.0), seed=13, alpha=0.85)
            gold = (rgba('#ffd25a'), rgba('#fff4c8'))
            for i, (x, y, w, h) in enumerate(((21, 50, 18, 12), (75, 50, 18, 12), (12, 65, 16, 11), (84, 65, 16, 11),
                                              (8, 81, 12, 8), (88, 81, 12, 8))):
                monitor(cv, x, y, w, h, gold, f, ('code', 'graph', 'term')[i % 3], seed=20 + i,
                        content=lambda kk, i=i: hsv((kk * 0.13 + i * 0.17 + f / N) % 1, 0.65, 1.0), alpha=0.95 if i < 4 else 0.7)
            orbit(cv, f, crown(f), rx=15, ry=4, front=False, speed=2)
            sigil(cv, f, front=False)
        else:
            orbit(cv, f, crown(f), rx=15, ry=4, front=True, speed=2)
            sigil(cv, f, front=True)
            if f % 4 == 1:
                tears(cv, f, rnd)
    return cv.im


def crown(f):
    chs = '{}{}{}'
    return [(chs[i], hsv((i / 6 + f / N) % 1, 0.55, 1.0)) for i in range(6)]


def circuit(cv, f, front):
    """Architect's floor: a cyan blueprint ring with traces running out to nodes that light in turn."""
    ground(cv, f, TEAL[0], front=front, rx=18, ry=5, ticks=12)
    for i in range(8):
        t = i / 8 * math.tau + 0.2
        s = math.sin(t)
        if (s >= 0) != front:
            continue
        x0, y0 = HX + math.cos(t) * 18, FY + s * 5
        x1, y1 = HX + math.cos(t) * 25, FY + s * 7
        cv.line(x0, y0, x1, y0, a(TEAL[0], 170))
        cv.line(x1, y0, x1, y1, a(TEAL[0], 170))
        on = (i + f) % 4 == 0
        cv.disc(x1, y1, 1.0, a(TEAL[1] if on else TEAL[0], 255 if on else 180))
        if on:
            glow(cv, x1, y1, 3, TEAL[0], 120)


def pulse(cv, f, col, front):
    """Root's pulse: rings growing out from his feet."""
    for j in range(2):
        tt = ((f + j * N / 2) % N) / N
        rx, ry = 8 + tt * 18, 2 + tt * 4.5
        for q in range(72):
            t = q / 72 * math.tau
            s = math.sin(t)
            if (s >= 0) != front:
                continue
            cv.add(HX + math.cos(t) * rx, FY + s * ry, a(col(tt), 230 * (1 - tt)))


def sigil(cv, f, front):
    """Zero-Day's 0day sigil: a turning ring with the letters around it, every colour."""
    for q in range(120):
        t = q / 120 * math.tau
        s = math.sin(t)
        if (s >= 0) != front:
            continue
        cv.add(HX + math.cos(t) * 20, FY + s * 5.5, hsv((q / 120 + f / N) % 1, 0.7, 1.0)[:3] + (220,))
        if q % 3 == 0:
            cv.add(HX + math.cos(t) * 15, FY + s * 4, hsv((q / 120 - f / N) % 1, 0.5, 1.0)[:3] + (140,))
    for i, ch in enumerate('0day'):
        t = (i / 4 + f / N / 4) * math.tau
        s = math.sin(t)
        if (s >= 0) != front:
            continue
        glyph(cv, ch, HX + math.cos(t) * 24 - 2, FY + s * 6 - 4, hsv((i / 4 + f / N) % 1, 0.5, 1.0), 255 if front else 140)


# ------------------------------------------------------------------ effects for the top ranks

HI = {   # fx tag: (the base frames, frame time)
    'fx_ping': (V.ping, 6, 0.05),
    'fx_shield': (V.shield_on, 6, 0.05),
    'fx_heal': (V.heal, 6, 0.06),
    'fx_chain': (V.chain, 5, 0.05),
    'fx_ddos': (V.ddos_fx, 5, 0.05),
    'fx_kill9': (V.kill9_fx, 6, 0.05),
    'fx_inject': (V.inject_fx, 6, 0.06),
    'fx_rollback': (V.rollback_fx, 6, 0.05),
}
TRIM, TRIM_L = rgba('#ffd25a'), rgba('#fff4c8')


def hi(fn, n, f):
    """The effect at twice the size with a gold trim, hex digits turning round it, sparks and a red / cyan ghost."""
    base = fn(f)
    big = base.resize((base.width * 2, base.height * 2), Image.NEAREST)
    W = big.width + 16
    H = big.height + 16
    cv = Cv(W, H)
    cx, cy = W / 2, H / 2
    life = 1 - f / n
    r = min(W, H) / 2 - 6
    # the ghost: the big effect offset red one way and cyan the other
    al = big.getchannel('A')
    for dx, col in ((-2, (255, 60, 90)), (2, (60, 230, 255))):
        g = Image.new('RGBA', big.size, col + (0,))
        g.putalpha(al.point(lambda v: int(v * 0.35)))
        cv.im.alpha_composite(g, (8 + dx, 8))
    cv.im.alpha_composite(big, (8, 8))
    cv.px = cv.im.load()
    rr = r * (0.55 + 0.45 * f / max(1, n - 1))
    cv.ring(cx, cy, rr, a(TRIM, 230 * life), 1.2)
    for i in range(10):
        t = i / 10 * math.tau + f * 0.35
        glyph(cv, V.HEX[(i * 7 + f) % 16], cx + math.cos(t) * (rr + 4) - 2, cy + math.sin(t) * (rr + 4) - 4, TRIM_L, 220 * life)
    rnd = random.Random(70 + f)
    for i in range(4):
        t = rnd.uniform(0, math.tau)
        star4(cv, cx + math.cos(t) * rr * 0.8, cy + math.sin(t) * rr * 0.8, TRIM_L, big=i % 2 == 0)
    return cv.im


# ------------------------------------------------------------------ the sheet

# ------------------------------------------------------------------ round 106: the rank outfits

FW, FH = 48, 52             # the body's frame: the outfit buff is centred on him like the body
FAX = 23.5                  # the frame's mirror axis (buffs don't flip when he faces left: outfits are symmetric)
HY = 16                     # his head's centre (coder_sprite.py: head and torso fixed in every frame)
FITS = ['Script Kiddie', 'Intern', 'Junior', 'Developer', 'Senior', 'Staff', 'Architect', 'Root', 'Zero-Day']
KY = 32                     # the keyboard's top row (his hands rest just above it)


def mirror_union(cv):
    from PIL import ImageOps
    im = cv.im
    out = im.copy()
    out.alpha_composite(ImageOps.mirror(im))
    return out


def arc(cv, cx, cy, rx, ry, a0, a1, col, w=1.0, n=60):
    for i in range(n + 1):
        t = math.radians(a0 + (a1 - a0) * i / n)
        x, y = cx + math.cos(t) * rx, cy + math.sin(t) * ry
        cv.put(x, y, col)
        if w > 1:
            cv.put(x + math.cos(t) * 0.8, y + math.sin(t) * 0.8, col)


def fill_ellipse(cv, cx, cy, rx, ry, col, y0=-99, y1=99, x1=99):
    for y in range(int(cy - ry) - 1, int(cy + ry) + 2):
        for x in range(int(cx - rx) - 1, int(cx + rx) + 2):
            if y0 <= y <= y1 and x <= x1 and ((x + 0.5 - cx) / rx) ** 2 + ((y + 0.5 - cy) / ry) ** 2 <= 1:
                cv.put(x, y, col)


def visor(cv, col, f, glint):
    for y in (15, 16):
        for x in range(18, 24):
            cv.put(x, y, col[:3] + ((235 if y == 15 else 200),))
    cv.put(17, 15, rgba('#1a1d24'))
    cv.put(17, 16, rgba('#1a1d24'))
    gx = 18 + (f * 2) % 6
    cv.put(gx, 15, glint)


def lapels(cv, cloth, edge):
    cv.poly([(18.5, 21.5), (23.5, 22.0), (23.5, 28.5), (21.0, 25.0)], cloth)
    cv.line(18.5, 21.5, 21.0, 25.0, edge)
    cv.line(21.0, 25.0, 23.5, 28.5, edge)


def fit(k, f):
    """Outfit k over his head, collar and shoulders (never where his arms move); drawn on the left half and mirrored."""
    cv = Cv(FW, FH)
    if k == 0:      # Script Kiddie: the hood up, a green trim
        for r, col in ((7.4, rgba('#1b2620')), (6.8, rgba('#24332a')), (6.3, rgba('#24332a'))):
            arc(cv, FAX, HY + 0.5, r, r + 0.8, 180, 270, col, w=2)
        arc(cv, FAX, HY + 0.5, 7.9, 8.7, 185, 270, rgba('#3cff8a')[:3] + (150 + 60 * (f % 2),))
        for y in range(19, 23):
            cv.put(16.4, y, rgba('#1b2620'))
        for y in range(24, 29):
            cv.put(21.5, y, rgba('#9fe8b8'))
    elif k == 1:    # Intern: a lanyard and a badge
        cv.line(20.0, 21.5, 23.5, 27.0, rgba('#4a8cff'))
        for y in range(27, 32):
            for x in range(21, 24):
                cv.put(x, y, rgba('#f2f5fa') if y > 27 else rgba('#4a8cff'))
        cv.put(22, 29, rgba('#9aa6b8'))
    elif k == 2:    # Junior: a beanie with a pompom
        fill_ellipse(cv, FAX, HY - 1.2, 6.2, 5.6, rgba('#ff8a3a'), y1=HY - 3, x1=FAX)
        for x in range(17, 24):
            cv.put(x, HY - 3, rgba('#cf6326'))
            cv.put(x, HY - 2, rgba('#cf6326'))
        for x in range(19, 24, 2):
            cv.put(x, HY - 6, rgba('#ffb27a'))
        cv.disc(FAX, HY - 7.6, 1.4, rgba('#ffd9bd'))
    elif k in (3, 4):   # Developer / Senior: over-ear headphones (blue / violet); Senior adds a jacket collar
        cup = rgba('#4aa3ff') if k == 3 else rgba('#a77bff')
        arc(cv, FAX, HY - 0.5, 6.4, 7.4, 180, 270, rgba('#2b2f3a'), w=2)
        arc(cv, FAX, HY - 0.8, 6.0, 7.0, 200, 260, rgba('#5b6274'))
        for y in range(HY - 2, HY + 3):
            for x in (16, 17):
                cv.put(x, y, cup if x == 16 else rgba('#1d2028'))
        cv.put(16, HY - 2, rgba('#e4f2ff'))
        if k == 4:
            cv.poly([(19.0, 21.5), (23.0, 21.5), (21.0, 25.5)], rgba('#3a2f5c'))
            cv.line(19.0, 21.5, 21.0, 25.5, rgba('#7d6bc0'))
    elif k == 5:    # Staff: a gold-trimmed jacket, a gold tie, epaulettes
        lapels(cv, rgba('#26262b'), rgba('#ffd25a'))
        for y in range(23, 29):
            cv.put(23, y, rgba('#c99a2e') if y > 23 else rgba('#ffd25a'))
        for x in range(18, 21):
            cv.put(x, 22, rgba('#ffd25a'))
    elif k == 6:    # Architect: a navy blazer, a white collar, a cyan visor
        lapels(cv, rgba('#1d2c48'), rgba('#6aa8ff'))
        cv.poly([(21.0, 21.2), (23.5, 21.2), (23.5, 24.0)], rgba('#eef4ff'))
        visor(cv, rgba('#6ee0ff'), f, rgba('#ffffff'))
    elif k == 7:    # Root: a black trench collar turned up, a red visor
        cv.poly([(17.5, 19.0), (20.5, 18.0), (22.0, 23.0), (18.5, 25.0)], rgba('#0c0c10'))
        cv.line(17.5, 19.0, 18.5, 25.0, rgba('#ff4d6d'))
        lapels(cv, rgba('#111116'), rgba('#5a1a2a'))
        visor(cv, rgba('#ff4d6d'), f, rgba('#ffd0d8'))
    else:           # Zero-Day: a white suit with gold lapels, a gold chain, a gold halo floating over his head
        # (round 106: no mask, and nothing framing his face: a dark collar under a crown read as a hijab, Rian)
        lapels(cv, rgba('#eceef4'), rgba('#e8b84a'))
        cv.line(18.5, 22.0, 20.0, 22.0, rgba('#e8b84a'))
        for k, (x, y) in enumerate(((20.5, 23.5), (21.5, 25.0), (22.5, 26.0), (23.5, 26.5))):   # the chain
            cv.put(x, y, rgba('#ffd25a') if k % 2 else rgba('#c99a2e'))
        for i in range(24):   # the halo: a thin gold ellipse above his hair, a glint running round it
            a = math.pi + i / 24 * math.pi
            x, y = FAX + math.cos(a) * 5.0, HY - 9.5 + math.sin(a) * 1.4
            on = (i + f * 3) % 12 < 2
            cv.put(x, y, rgba('#fff4c8') if on else rgba('#ffd25a'))
        for i in range(24):
            a = i / 24 * math.pi
            cv.put(FAX + math.cos(a) * 5.0, HY - 9.5 + math.sin(a) * 1.4, rgba('#c99a2e'))
    return mirror_union(cv)


def keyrow(cv, x0, y, n, w, cols, gap=1, lit=None):
    """A row of n keys w px wide; `cols` (cap, edge); `lit` the key indices lit up."""
    for k in range(n):
        c = cols[2] if lit and k in lit else cols[0]
        for x in range(w):
            cv.put(x0 + k * (w + gap) + x, y, c)


def kb(k, f):
    """Round 106: his rank's keyboard, floating at his hands (Rian: "make his keyboard change design per rank").
    Centred and symmetric (a buff doesn't flip when he faces left), 8 frames of keys being hit."""
    cv = Cv(FW, FH)
    rnd = random.Random(k * 31 + f)
    hits = {rnd.randrange(7), rnd.randrange(7)}
    x0 = 17
    if k == 0:      # Script Kiddie: a cheap black membrane keyboard, green-lit keys
        for y in range(KY, KY + 4):
            for x in range(x0, x0 + 14):
                cv.put(x, y, rgba('#121614'))
        for r in range(3):
            keyrow(cv, x0 + 1 + (r % 2), KY + r, 6, 1, (rgba('#2f6b45'), None, rgba('#5aff96')), lit=hits if r == 1 else None)
    elif k == 1:    # Intern: a silver laptop, the screen's back to us, a sticker on it
        for y in range(KY - 6, KY):
            for x in range(x0 + 1, x0 + 13):
                cv.put(x, y, rgba('#b8c0cc') if y > KY - 6 else rgba('#e2e8f0'))
        cv.put(FAX, KY - 3, rgba('#ff8a3a'))
        for y in range(KY, KY + 2):
            for x in range(x0, x0 + 14):
                cv.put(x, y, rgba('#cfd6e0') if y == KY else rgba('#8a93a2'))
    elif k == 2:    # Junior: a chunky mechanical keyboard, orange caps
        for y in range(KY, KY + 5):
            for x in range(x0 - 1, x0 + 15):
                cv.put(x, y, rgba('#2a2420'))
        for r in range(3):
            keyrow(cv, x0 + (r % 2), KY + 1 + r, 7, 1, (rgba('#ff8a3a') if r != 2 else rgba('#e6e0d8'), None, rgba('#ffd2a8')), lit=hits)
    elif k == 3:    # Developer: a split ergonomic keyboard, blue backlight
        for half in (0, 1):
            hx = x0 - 1 + half * 9
            for y in range(KY, KY + 4):
                for x in range(hx, hx + 6):
                    cv.put(x, y, rgba('#1d2230'))
            for r in range(3):
                keyrow(cv, hx + 1, KY + r, 2, 1, (rgba('#4aa3ff'), None, rgba('#d8ecff')), lit=hits)
            glow(cv, hx + 3, KY + 2, 4, rgba('#4aa3ff'), 50)
    elif k == 4:    # Senior: a 60% keyboard, a violet RGB wave running over it
        for y in range(KY, KY + 4):
            for x in range(x0, x0 + 14):
                cv.put(x, y, rgba('#24202e'))
        for r in range(3):
            for kk in range(6):
                hue = 0.72 + 0.12 * math.sin((kk + r + f) * 0.9)
                cv.put(x0 + 1 + kk * 2 + (r % 2), KY + r, hsv(hue, 0.55, 1.0))
    elif k == 5:    # Staff: a low-profile keyboard in a gold frame
        for y in range(KY, KY + 4):
            for x in range(x0 - 1, x0 + 15):
                edge = y in (KY, KY + 3) or x in (x0 - 1, x0 + 14)
                cv.put(x, y, rgba('#ffd25a') if edge else rgba('#26262b'))
        keyrow(cv, x0 + 1, KY + 1, 6, 1, (rgba('#e8e2d0'), None, rgba('#ffd25a')), lit=hits)
        keyrow(cv, x0 + 2, KY + 2, 6, 1, (rgba('#bdb6a4'), None, rgba('#ffd25a')), lit=hits)
    elif k == 6:    # Architect: a projected holo keyboard, a cyan grid of light and its projector beam
        for y in range(KY, KY + 4):
            for x in range(x0 - 1, x0 + 15):
                if (x + y) % 2 == 0 or y in (KY, KY + 3):
                    cv.add(x, y, rgba('#6ee0ff')[:3] + (150,))
        for kk in hits:
            cv.add(x0 + kk * 2, KY + 1, rgba('#e0fcff'))
        for t in range(5):
            cv.add(FAX - 0.5, KY + 4 + t, rgba('#6ee0ff')[:3] + (120 - t * 20,))
    elif k == 7:    # Root: two black keyboards stacked, red-lit
        for y0, w in ((KY - 1, 12), (KY + 2, 14)):
            xs = FAX - w / 2 + 0.5
            for y in range(y0, y0 + 2):
                for x in range(int(xs), int(xs) + w):
                    cv.put(x, y, rgba('#0c0c10'))
            keyrow(cv, int(xs) + 1, y0, w // 2 - 1, 1, (rgba('#7a1626'), None, rgba('#ff4d6d')), lit=hits)
    else:           # Zero-Day: no keyboard at all, gold keycaps floating in a ring, the hit ones flaring
        for kk in range(10):
            a = kk / 10 * math.tau + f / 8 * math.tau / 5
            x, y = FAX + math.cos(a) * 8, KY + 1 + math.sin(a) * 2.2
            col = hsv(0.12 + 0.06 * math.sin(kk + f), 0.6, 1.0) if kk % 5 not in hits else rgba('#ffffff')
            cv.put(x, y, col)
            cv.put(x + 1, y, col[:3] + (180,))
    return mirror_union(cv) if k not in (4, 8) else cv.im


def all_anims():
    A = {}
    for k in range(len(FITS)):
        A[f'fit{k}'] = ([fit(k, f) for f in range(4)], 0.15)
        A[f'kb{k}'] = ([kb(k, f) for f in range(8)], 0.1)
    for k in range(1, 6):
        A[f'rig{k}'] = ([rig(k, f, False) for f in range(N)], 0.1)
        A[f'rigf{k}'] = ([rig(k, f, True) for f in range(N)], 0.1)
    for tag, (fn, n, dur) in HI.items():
        A[tag + '_hi'] = ([hi(fn, n, f) for f in range(n)], dur)
    return A


def preview_fits(A, folder):
    """Every outfit over every body frame (idle, run, attack, skill1, skill2, ult, hit), facing right and mirrored."""
    from PIL import ImageOps
    body = Image.open(os.path.join(ROOT, 'mods', 'tfm2_custom', 'champions', 'tfm2_custom_coder#sheet.png'))
    fan = json.load(open(os.path.join(ROOT, 'mods', 'tfm2_custom', 'champions', 'tfm2_custom_coder#anim.fanim')))['anims']
    picks = [('idle', 0), ('idle', 4), ('run', 0), ('run', 3), ('attack', 3), ('skill1', 1), ('skill2', 2), ('ult', 3), ('hit', 1)]
    bg = (52, 70, 58, 255)
    out = Image.new('RGBA', (FW * len(picks) * 2 + 8, FH * len(FITS)), bg)
    for r in range(len(FITS)):
        for i, (tag, fi) in enumerate(picks):
            d = fan[tag]['frames'][fi]['data']
            b = body.crop((d['x'], d['y'], d['x'] + d['w'], d['y'] + d['h']))
            for side, im in ((0, b), (1, ImageOps.mirror(b))):
                cell = im.copy()
                cell.alpha_composite(A[f'fit{r}'][0][fi % 4])
                cell.alpha_composite(A[f'kb{r}'][0][fi % 8])
                out.alpha_composite(cell, (side * (FW * len(picks) + 8) + i * FW, r * FH))
    out.resize((out.width * 3, out.height * 3), Image.NEAREST).save(os.path.join(folder, 'outfits.png'))


def preview(A, folder):
    """Every rank: the rig behind, his idle body, the rig in front, his crest (Script Kiddie .. Zero-Day)."""
    body = Image.open(os.path.join(ROOT, 'mods', 'tfm2_custom', 'champions', 'tfm2_custom_coder#sheet.png'))
    VA = V.all_anims()
    bg = (52, 70, 58, 255)
    cols = [(r, None) for r in range(7)] + [(7, 10), (7, 1)]
    out = Image.new('RGBA', (RW * len(cols), RH * 2), bg)
    for row, f in enumerate((0, 3)):
        for i, (r, p) in enumerate(cols):
            k = {4: 1, 5: 2, 6: 3}.get(r, 0) if r < 7 else (5 if p == 1 else 4)
            cell = Image.new('RGBA', (RW, RH), (0, 0, 0, 0))
            if k:
                cell.alpha_composite(A[f'rig{k}'][0][f])
            b = body.crop((f * 48, 0, f * 48 + 48, 52))
            kk = r if r < 7 else (8 if p == 1 else 7)
            b.alpha_composite(A[f'fit{kk}'][0][f % 4])
            b.alpha_composite(A[f'kb{kk}'][0][f % 8])
            cell.alpha_composite(b, (RW // 2 - 24, RH // 2 - 26))
            if k:
                cell.alpha_composite(A[f'rigf{k}'][0][f])
            crest = VA[f'rank{r}' if r < 7 else f'root{p}'][0][f]
            cell.alpha_composite(crest, (RW // 2 - crest.width // 2, RH // 2 - 48))
            heat = VA['heat4'][0][0]
            cell.alpha_composite(heat, (RW // 2 - 24, RH // 2 - 48))
            out.alpha_composite(cell, (i * RW, row * RH))
    out.resize((out.width * 3, out.height * 3), Image.NEAREST).save(os.path.join(folder, 'ranks.png'))
    # the top-rank effects next to the normal ones
    keys = list(HI)
    rows = []
    for k in keys:
        n = HI[k][1]
        ims = [VA[k][0][min(1, n - 1)], A[k + '_hi'][0][1], A[k + '_hi'][0][n // 2]]
        w = sum(i.width + 4 for i in ims)
        h = max(i.height for i in ims)
        r_ = Image.new('RGBA', (w, h), bg)
        x = 0
        for im in ims:
            r_.alpha_composite(im, (x, (h - im.height) // 2))
            x += im.width + 4
        rows.append(r_)
    W = sum(r_.width + 6 for r_ in rows[:4])
    H = max(r_.height for r_ in rows) * 2 + 6
    fx = Image.new('RGBA', (max(W, sum(r_.width + 6 for r_ in rows[4:])), H), bg)
    x = 0
    for j, r_ in enumerate(rows):
        if j == 4:
            x = 0
        fx.alpha_composite(r_, (x, 0 if j < 4 else H // 2 + 3))
        x += r_.width + 6
    fx.resize((fx.width * 2, fx.height * 2), Image.NEAREST).save(os.path.join(folder, 'ranks_fx.png'))


if __name__ == '__main__':
    A = all_anims()
    sheet, fan = V.pack(A)
    if '--preview' in sys.argv:
        preview(A, os.path.join(HERE, 'preview'))
        preview_fits(A, os.path.join(HERE, 'preview'))
    if '--dry' not in sys.argv:
        sheet.save(os.path.join(OUT, 'coder_rank#sheet.png'), optimize=True)
        with open(os.path.join(OUT, 'coder_rank#anim.fanim'), 'w', encoding='utf-8') as fh:
            json.dump(fan, fh, separators=(',', ':'))
    print('coder_rank', sheet.size, len(fan['anims']), 'anims')
