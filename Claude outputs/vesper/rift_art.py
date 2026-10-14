"""Vesper's rift effects: shapes, family cores, dimension glyphs, link threads,
packets, impacts, open / close / break, the 29 technique animations, the 15
bonus (twist) bursts, Breach and the atlas drift warning.

Scale: about 950 world units per pixel (Tear r18000 ~ 40 px across).
Rift shape layers stay neutral white-violet so the family core (z2, drawn on
top) supplies the colour. Every loop has eight distinct deterministic frames.
Headings follow the native heading8 (screen y = world y): 0:+x 1:(+x,+y)
2:+y 3:(-x,+y) 4:-x 5:(-x,-y) 6:-y 7:(+x,-y).
"""
import re
import math
import importlib.util
from functools import lru_cache
from pathlib import Path

from PIL import Image, ImageDraw, ImageChops

HERE = Path(__file__).resolve().parent
_spec = importlib.util.spec_from_file_location('vesper_art_common', HERE / 'vesper_art.py')
V = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(V)

TAU = math.tau
a, mix, pt, line, poly, dot, px, sparkle, ring, dotted_ring, glow, hsh = (
    V.a, V.mix, V.pt, V.line, V.poly, V.dot, V.px, V.sparkle, V.ring, V.dotted_ring, V.glow, V.hsh)
WHITE, RIM, RIFT, RIFT_PALE, RIFT_DEEP, VOID = V.WHITE, V.RIM, V.RIFT, V.RIFT_PALE, V.RIFT_DEEP, V.VOID
GOLD, GOLD_WHITE, CRYSTAL, FAMILY, INK = V.GOLD, V.GOLD_WHITE, V.CRYSTAL, V.FAMILY, V.INK
EMBER = (255, 236, 170, 255)

CANVAS = 128          # rift shapes (trimmed per tag in the atlas)
CORE = 40
GLYPH = 40
LINK = 112
PACKET = 48
EXIT = 56
IMPACT = 64
OPEN = 80
CLOSE = 64
BREAK = 96
HINT = 32
TECH = 96
TWIST = 64
BREACH = 128
BOLT = 32

SHAPES = ['tear', 'sinkhole', 'sky', 'twin', 'wander', 'anchor', 'seed', 'maw']
LINES = ['slash', 'mirror']
TECHS = ['chain', 'crossfire', 'bank', 'rain', 'volley', 'implode', 'domino', 'opunch', 'step', 'gate', 'skydrop',
         'kidnap', 'sinktrap', 'swap', 'fling', 'hook', 'phase', 'curtain', 'reflect', 'rescue', 'pocket', 'lifeline',
         'escort', 'rotation', 'seedtrap', 'sweep', 'infuse', 'stitch', 'maw']
SLOW_TECHS = {'crossfire', 'opunch', 'implode', 'phase', 'pocket', 'seedtrap', 'maw', 'stitch'}
TWISTS = [k['name'] for k in V.DATA['twist_kinds']]

H8 = [(1, 0), (1, 1), (0, 1), (-1, 1), (-1, 0), (-1, -1), (0, -1), (1, -1)]


def unit(h):
    x, y = H8[h % 8]
    n = math.hypot(x, y)
    return x / n, y / n


EDGE, MIDV, VOIDC = V.SCAR_EDGE, V.SCAR_MID, V.SCAR_VOID     # reality is glass: fractured edge, violet, void


def rim_color(look):
    return GOLD_WHITE if look == 8 else EDGE


# ----------------------------------------------------------- other worlds --

@lru_cache(maxsize=256)
def otherworld(w, h, f, full=False):
    """The world on the far side: a dark sky, a far horizon, drifting stars."""
    im = Image.new('RGBA', (w, h), VOID)
    d = ImageDraw.Draw(im)
    ph = f * TAU / 8
    if full:
        for k in range(3):
            yb = h * (.25 + .18 * k) + math.sin(ph + k) * 2
            col = [(80, 220, 200), (190, 120, 255), (255, 200, 120)][k]
            line(d, [(x, yb + math.sin(x * .4 + ph + k) * 1.5) for x in range(0, w + 1, 2)], a(col, 120))
    hy = h * .68
    pts = [(x, hy - 2 * math.sin(x * .35 + 1) - (1 if x % 7 < 3 else 0)) for x in range(0, w + 1)]
    poly(d, pts + [(w, h), (0, h)], fill=(30, 70, 84, 255))
    line(d, pts, (90, 190, 190, 255))
    for k in range(6):
        sx = (hsh(11, k) * w + f * (1 + k % 2)) % w
        sy = hsh(12, k) * hy * .9
        px(d, sx, sy, WHITE if (k + f) % 3 else RIFT_PALE)
    if full:
        dot(d, w * .7 + math.cos(ph) * 1.5, h * .3, max(1, min(w, h) * .09), GOLD_WHITE)
    return im


def paste_into(im, src_fn, pts):
    box = [min(p[0] for p in pts), min(p[1] for p in pts), max(p[0] for p in pts), max(p[1] for p in pts)]
    w, h = max(2, round(box[2] - box[0]) + 2), max(2, round(box[3] - box[1]) + 2)
    V.paste_masked(im, src_fn(w, h), box[0] - 1, box[1] - 1, pts)


# ------------------------------------------------------------- rip engine --

W_FRAC = [0, .2, .5, .55, .7, .7, .72, 1.0, .9]
JAG = [1.6, 1.0, 1.2, .8, .8, .7, .8, .6, .5]


def lens(p0, p1, w, seed, jag=1.0, taper=.75):
    (x0, y0), (x1, y1) = p0, p1
    L = math.hypot(x1 - x0, y1 - y0)
    n = max(6, int(L / 3))
    ux, uy = (x1 - x0) / L, (y1 - y0) / L
    nx, ny = -uy, ux
    left, right, spine = [], [], []
    for i in range(n + 1):
        t = i / n
        bx, by = x0 + (x1 - x0) * t, y0 + (y1 - y0) * t
        j = (hsh(seed, i) - .5) * 2 * jag if 0 < i < n else 0
        ww = w * (math.sin(math.pi * t) ** taper)
        wl = ww * (1 + (hsh(seed, i, 1) - .5) * .6 * jag)
        wr = ww * (1 + (hsh(seed, i, 2) - .5) * .6 * jag)
        spine.append((bx + nx * j, by + ny * j))
        left.append((bx + nx * (j + wl), by + ny * (j + wl)))
        right.append((bx + nx * (j - wr), by + ny * (j - wr)))
    return left, right, spine, (nx, ny)


def rune_ring(d, cx, cy, r, look, f, flat=1.0):
    """No rings: the glass of reality cracks around the rift, more the higher she ranks."""
    n = [0, 2, 3, 4, 4, 5, 5, 6, 6][look]
    col = GOLD_WHITE if look == 8 else EDGE
    for k, pth in enumerate(V.cracks(900 + look + int(r), cx, cy, n, r * .55, TAU, .3, look >= 5)):
        pts = [(cx + (x - cx) * (1 + r / max(1, math.hypot(x - cx, y - cy) + r)), cy + (y - cy) * flat) for x, y in pth]
        pts = [(cx + (x - cx) * 1.0, y) for x, y in pts]
        start = int(len(pts) * .45)
        line(d, pts[start:], a(col, 150 if (k + f) % 4 else 230))
    if look == 7:
        for k in range(3):
            x, y = pt(cx, cy, r + 2, k * TAU / 3 + f * .1, flat)
            V.shard(d, x, y, 1.1, EDGE, outline=MIDV)


def rip(im, p0, p1, wmax, look, f, seed, interior='light', taper=.75):
    """A rip along p0 -> p1 in the look's style. `interior`: light | void."""
    d = ImageDraw.Draw(im)
    ph = f * TAU / 8
    rim = rim_color(look)
    if look == 0:   # Spark: a flickering hairline crack spitting sparks
        flick = [255, 150, 230, 90, 255, 190, 70, 220][f]
        _, _, S, _ = lens(p0, p1, 0, seed, jag=1.6)
        line(d, S, a(rim, flick))
        for k in range(3):
            i = int(hsh(seed, k, f) * len(S))
            x, y = S[i]
            ang = hsh(seed, k, f, 1) * TAU
            r = 2 + (f + k) % 3 * 2
            px(d, *pt(x, y, r, ang), a(EMBER, 230))
            if (f + k) % 2:
                px(d, *pt(x, y, r + 2, ang), a(CRYSTAL, 170))
        return S
    w = max(1.0, wmax * W_FRAC[look])
    if look == 7:
        taper = .45
    if look == 6:   # Riftwalker: a second, deeper cut a pixel behind
        L2, R2, _, (nx, ny) = lens(p0, p1, w * 1.5, seed + 5, JAG[look])
        off = math.sin(ph) * 1.5
        outer = [(x + nx * off, y + ny * off) for x, y in L2 + R2[::-1]]
        poly(d, outer, fill=a(MIDV, 110), outline=a(EDGE, 120))
    L, R, S, (nx, ny) = lens(p0, p1, w, seed, JAG[look], taper)
    outline = L + R[::-1]
    poly(d, outline, fill=MIDV if look > 1 else a(MIDV, 190))
    if look >= 2 or interior == 'void':      # the near-black void at the heart of the cut
        core_w = .5 if look < 5 else .62
        inner = [(sx + (lx - sx) * core_w, sy + (ly - sy) * core_w) for (sx, sy), (lx, ly) in zip(S, L)]
        inner += [(sx + (rx - sx) * core_w, sy + (ry - sy) * core_w) for (sx, sy), (rx, ry) in zip(S[::-1], R[::-1])]
        if look == 8:   # #1: the cut opens onto another world
            paste_into(im, lambda ww, hh: otherworld(ww, hh, f, True), inner)
            d = ImageDraw.Draw(im)
        else:
            poly(d, inner, fill=VOIDC)
    line(d, outline + [outline[0]], a(rim, 170 if look == 1 else 255), 2 if look == 8 else 1)
    if look == 1:   # frayed edges
        for side, sgn in ((L, 1), (R, -1)):
            for i in range(1, len(side) - 1, 2):
                x, y = side[i]
                ln = 1 + (i + f) % 2
                line(d, [(x, y), (x + nx * sgn * ln + (hsh(seed, i) - .5), y + ny * sgn * ln)], a(EDGE, 120))
    if look in (2, 6):   # glass grit falling out of the cut
        for k in range(3 + (look == 6)):
            i = 1 + int(hsh(seed, k) * (len(S) - 2))
            x, y = S[i]
            t = (f + k * 3) % 8
            sgn = 1 if k % 2 else -1
            px(d, x + nx * sgn * (w + 2 + t), y + ny * sgn * (w + 2 + t) + t * .4, a(EDGE, 230 - t * 25))
    if look in (3, 4, 5):   # fractures branch off the lips of the cut
        for k in range(look - 1):
            i = 1 + int(hsh(seed, k, 7) * (len(S) - 2))
            side = L if k % 2 else R
            x, y = side[i]
            sgn = 1 if k % 2 else -1
            line(d, [(x, y), (x + nx * sgn * 3 + (hsh(seed, k) - .5) * 2, y + ny * sgn * 3), (x + nx * sgn * 5, y + ny * sgn * 5 + 1)], a(EDGE, 200))
    if look == 7:   # shattered-glass lips
        for side, sgn in ((L, 1), (R, -1)):
            for i in range(1, len(side) - 1, 2):
                x, y = side[i]
                tip = (x + nx * sgn * 3, y + ny * sgn * 3)
                poly(d, [(x - (ny * 1.2), y + nx * 1.2), tip, (x + ny * 1.2, y - nx * 1.2)], fill=a(EDGE, 210), outline=a(WHITE, 255))
    if look == 8:   # gold-white fractures run out from both ends
        for pth in V.cracks(seed, *S[0], 3, 10, TAU, ph * .1, False) + V.cracks(seed + 1, *S[-1], 3, 10, TAU, -ph * .1, False):
            line(d, pth[:4], a(GOLD_WHITE, 200))
        i = (f * 2) % len(outline)
        sparkle(d, *outline[i], GOLD_WHITE, 1)
    i = 1 + f % max(1, len(S) - 2)
    px(d, *S[i], a(WHITE, 220))
    return S


# ------------------------------------------------------------ rift shapes --

def rift_shape(kind, look, f, angle=0):
    im = Image.new('RGBA', (CANVAS, CANVAS))
    d = ImageDraw.Draw(im)
    c = CANVAS // 2
    ph = f * TAU / 8
    seed = 100 + SHAPES.index(kind) * 10 if kind in SHAPES else 300 + angle
    if kind == 'tear':
        rune_ring(d, c, c, 19, look, f)
        rip(im, (c, c - 19), (c, c + 19), 8, look, f, seed)
    elif kind == 'sinkhole':
        rune_ring(d, c, c, 29, look, f, .5)
        if look == 0:
            pts = [pt(c, c, 26 + (hsh(seed, k) - .5) * 3, k * TAU / 20, .42) for k in range(21)]
            line(d, pts, a(RIM, [255, 150, 230, 90, 255, 190, 70, 220][f]))
            for k in range(3):
                x, y = pts[int(hsh(seed, k, f) * 20)]
                px(d, x, y - 2 - (f + k) % 3, a(EMBER, 230))
        else:
            # a flat ground hole: an ellipse lens with a swirling interior
            rx, ry = 12 + 16 * W_FRAC[look], 5 + 7 * W_FRAC[look]
            pts = [pt(c, c, 1, k * TAU / 28) for k in range(28)]
            pts = [(c + rx * (1 + (hsh(seed, k) - .5) * .25 * JAG[look]) * math.cos(k * TAU / 28),
                    c + ry * (1 + (hsh(seed, k, 1) - .5) * .25 * JAG[look]) * math.sin(k * TAU / 28)) for k in range(28)]
            poly(d, pts, fill=MIDV)
            inner = [(c + (x - c) * .6, c + (y - c) * .6) for x, y in pts]
            if look == 8:
                paste_into(im, lambda ww, hh: otherworld(ww, hh, f, True), inner)
                d = ImageDraw.Draw(im)
            else:
                poly(d, inner, fill=VOIDC)
            for k in range(2):    # the ground caving in
                r0 = rx * (.35 + .25 * k)
                ring(d, c, c, r0, a(MIDV, 200), ry / rx, 10, ph / (k + 1) + k * 2, ph / (k + 1) + k * 2 + 2.2)
            line(d, pts + [pts[0]], a(rim_color(look), 255), 2 if look >= 7 else 1)
            if look == 1:
                for k in range(0, 28, 3):
                    x, y = pts[k]
                    px(d, x, y - 1 - (k + f) % 2, a(RIM, 120))
            if look == 7:
                for k in range(0, 28, 2):
                    x, y = pts[k]
                    poly(d, [(x - 1, y), (x, y - 3), (x + 1, y)], fill=a(EDGE, 220), outline=a(WHITE, 200))
            if look == 8:
                for pth in V.cracks(seed, c, c, 6, rx * .9, TAU, ph * .05, False):
                    line(d, [(x, c + (y - c) * .45) for x, y in pth[:4]], a(GOLD_WHITE, 180))
            if look in (2, 6):
                for k in range(3):
                    t = (f + k * 3) % 8
                    x, y = pts[(k * 9) % 28]
                    px(d, x, y - t * 1.5, a(EDGE, 230 - t * 25))
            if look >= 3:   # the ground around it cracks
                for pth in V.cracks(seed + look, c, c, 2 + look // 2, rx * .8, TAU, .2, False):
                    flat = [(c + (x - c) * 1.2, c + (y - c) * .45) for x, y in pth]
                    line(d, flat[2:], a(GOLD_WHITE if look == 8 else EDGE, 170))
            px(d, *pts[(f * 3) % 28], WHITE)
    elif kind == 'sky':
        # the shadow on the ground, then the overhead rip casting it
        d.ellipse((c - 22, c - 8, c + 22, c + 8), fill=a(VOID, 70 + 6 * look))
        d.ellipse((c - 14, c - 5, c + 14, c + 5), fill=a(VOID, 50))
        rune_ring(d, c, c, 25, look, f, .4)
        if look >= 5:   # glass grit falls out of the overhead cut
            for k in range(4):
                t = (f + k * 2) % 8
                px(d, c - 15 + k * 10, c - 28 + t * 3, a(EDGE, 230 - t * 25))
        rip(im, (c - 24, c - 32), (c + 24, c - 32), 8, look, f, seed)
        d = ImageDraw.Draw(im)
        px(d, c - 20 + f * 5, c + 6, a(RIM, 120))
    elif kind == 'twin':
        rune_ring(d, c, c, 16, look, f)
        ghost = Image.new('RGBA', (CANVAS, CANVAS))
        rip(ghost, (c + 7, c - 13), (c + 7, c + 13), 5, look, (f + 4) % 8, seed + 3)
        ghost.putalpha(ghost.getchannel('A').point(lambda v: v * 2 // 5))
        im.alpha_composite(ghost)
        rip(im, (c, c - 16), (c, c + 16), 6, look, f, seed)
        d = ImageDraw.Draw(im)
        # the pair is one fracture: a hairline crack bridges the two cuts
        bridge = [(c, c - 4), (c + 2, c - 2), (c + 4, c - 3), (c + 7, c)]
        line(d, bridge, a(rim_color(look), 230 if look else [255, 140, 220, 90, 255, 180, 70, 220][f]))
        px(d, *bridge[f % 4], WHITE)
    elif kind == 'wander':
        rune_ring(d, c, c, 16, look, f)
        for k in range(3):
            y = c - 9 + k * 9
            x0 = c - 6 - ((f * 2 + k * 3) % 8)
            line(d, [(x0 - 8 - k * 2, y), (x0, y)], a(RIM, 70 + 40 * (k == 1)))
            px(d, x0 - 10 - k * 2 - f % 2, y, a(RIM, 50))
        rip(im, (c, c - 15), (c, c + 15), 6, look, f, seed)
    elif kind == 'anchor':
        rune_ring(d, c, c, 14, look, f)
        rip(im, (c, c - 12), (c, c + 9), 5, look, f, seed)
        d = ImageDraw.Draw(im)
        col = rim_color(look)
        for k in range(3):
            px(d, c, c + 11 + k * 2 + (f % 2), a(col, 230))
        line(d, [(c, c + 17), (c, c + 21), (c + 2, c + 23), (c + 4, c + 22), (c + 5, c + 19)], a(col, 255))
        line(d, [(c + 5, c + 19), (c + 3, c + 20)], a(col, 255))
        if look >= 3:
            px(d, c + 4, c + 22 - f % 3, GOLD)
    elif kind == 'seed':
        seed_shape(d, c, look, f, 1.0)
    elif kind == 'maw':
        rune_ring(d, c, c, 42, look, f, .55)
        S = rip(im, (c - 40, c), (c + 40, c), 18, look, f, seed, interior='void', taper=.6)
        d = ImageDraw.Draw(im)
        if look > 0:
            w = max(1.0, 18 * W_FRAC[look])
            L, R, _, (nx, ny) = lens((c - 40, c), (c + 40, c), w, seed, JAG[look], .6)
            teeth = GOLD_WHITE if look == 8 else EDGE
            bite = [0, .5, 1, 1.5, 2, 1.5, 1, .5][f]
            for side, sgn in ((L, -1), (R, 1)):
                for i in range(1, len(side) - 1, 2):
                    x, y = side[i]
                    h = (2 + 2 * W_FRAC[look]) * (1 + (i % 3 == 0)) + bite
                    poly(d, [(x - 1.5, y), (x + 1.5, y), (x + nx * sgn * h, y + ny * sgn * h)], fill=a(teeth, 240), outline=a(INK, 160))
        else:
            for k in range(0, len(S), 3):
                x, y = S[k]
                px(d, x, y - 2, a(RIM, 120))
                px(d, x, y + 2, a(RIM, 120))
    elif kind == 'slash':
        ux, uy = unit(angle)
        p0, p1 = (c - ux * 31, c - uy * 31), (c + ux * 31, c + uy * 31)
        rip(im, p0, p1, 5, look, f, seed, taper=.5)
    elif kind == 'mirror':
        pane(im, angle, look, f)
    d = ImageDraw.Draw(im)
    px(d, 4 + f, CANVAS - 4, a(RIM, 40))     # keeps eight phases distinct after packing
    return glow(im, .8 + .1 * look, 1.4) if look >= 2 else im


def seed_shape(d, c, look, f, alpha=1.0):
    """Seed Rift: a hidden hairline crack in the glass of reality, glinting now and then."""
    col = GOLD_WHITE if look == 8 else EDGE
    k = alpha * (0.5 + .05 * look)
    for pth in V.cracks(81 + look, c, c, 3 + look // 3, 9 + look, TAU, .5, look >= 4):
        line(d, pth, a(col, (90 + 20 * (f % 3 == 0)) * k + 30))
    px(d, c, c, a(MIDV, 220 * k + 30))
    glint = [(c + 3, c - 2), (c - 4, c + 1), (c + 1, c + 4), (c - 2, c - 4)][f % 4]
    if f % 2 == 0:
        px(d, *glint, a(WHITE, 200 * k + 40))


def pane(im, angle, look, f):
    """Mirror Pane: a standing glass segment along the line, with reflections."""
    d = ImageDraw.Draw(im)
    c = CANVAS // 2
    ph = f * TAU / 8
    ux, uy = unit(angle)
    H = 12 + look
    B0, B1 = (c - ux * 21, c - uy * 21 + H / 2), (c + ux * 21, c + uy * 21 + H / 2)
    T0, T1 = (B0[0] - 3, B0[1] - H), (B1[0] - 3, B1[1] - H)
    quad = [B0, B1, T1, T0]
    rim = rim_color(look)
    if look == 0:
        al = [200, 120, 180, 70, 210, 150, 60, 170][f]
        line(d, quad + [quad[0]], a(rim, al))
        mid = ((B0[0] + T1[0]) / 2, (B0[1] + T1[1]) / 2)
        line(d, [mid, (mid[0] + 3, mid[1] - 4), (mid[0] + 1, mid[1] - 7)], a(rim, al))
        for k in range(2):
            q = quad[(f + k) % 4]
            px(d, q[0] + (k * 2 - 1) * (1 + f % 3), q[1] - 1, a(EMBER, 230))
        return
    if look == 6:   # a second pane at parallax depth
        off = math.sin(ph) * 1.5
        q2 = [(x + 2 + off, y - 2) for x, y in quad]
        poly(d, q2, fill=a(MIDV, 50))
        line(d, q2 + [q2[0]], a(EDGE, 100))
    if look == 8:
        paste_into(im, lambda ww, hh: otherworld(ww, hh, f, True), quad)
        d = ImageDraw.Draw(im)
        overlay = Image.new('RGBA', im.size)
        poly(ImageDraw.Draw(overlay), quad, fill=a((210, 230, 255), 70))
        im.alpha_composite(overlay)
        d = ImageDraw.Draw(im)
    else:
        poly(d, quad, fill=a((210, 230, 255), [40, 40, 70, 80, 90, 90, 100, 110][look]))
    if look >= 2:   # the pane is cracked: a star of fractures from its centre
        mid = ((B0[0] + T1[0]) / 2, (B0[1] + T1[1]) / 2)
        for pth in V.cracks(91 + look, *mid, 3 + look // 2, 6 + look, TAU, .4, look >= 5):
            line(d, pth[:4], a(GOLD_WHITE if look == 8 else EDGE, 200))
    if look >= 2:   # a reflection streak sweeping the pane
        t = f / 8
        p = (B0[0] + (B1[0] - B0[0]) * t, B0[1] + (B1[1] - B0[1]) * t)
        q = (T0[0] + (T1[0] - T0[0]) * min(1, t + .15), T0[1] + (T1[1] - T0[1]) * min(1, t + .15))
        line(d, [p, q], a(WHITE, 200), 1 + (look >= 7))
    line(d, quad + [quad[0]], a(rim, 150 if look == 1 else 255), 2 if look == 8 else 1)
    if look == 7:
        for k in range(5):
            t = (k + .5) / 5
            x, y = T0[0] + (T1[0] - T0[0]) * t, T0[1] + (T1[1] - T0[1]) * t
            poly(d, [(x - 1, y), (x, y - 3), (x + 1, y)], fill=a(EDGE, 230), outline=a(WHITE, 200))
    if look == 8:
        for q in quad:
            sparkle(d, q[0], q[1], GOLD_WHITE, 1 + (f % 2))
    if look == 1:
        px(d, *quad[f % 4], a(WHITE, 160))


# ----------------------------------------------------------- family cores --

def core(fam, tier, f):
    im = Image.new('RGBA', (CORE, CORE))
    d = ImageDraw.Draw(im)
    c = CORE // 2
    col = FAMILY[fam]
    ph = f * TAU / 8
    r = [4, 5, 6, 7][tier]
    if tier == 0:
        flick = [1, .6, .9, .45, 1, .7, .4, .85][f]
        dot(d, c, c, r, a(col, 170 * flick))
        dot(d, c, c, r - 2, a(mix(col, WHITE, .5), 230 * flick))
        for k in range(2):
            if (f + k) % 3:
                px(d, *pt(c, c, r + 2 + (f + k) % 3, hsh(fam, f, k) * TAU), a(col, 220))
    else:
        pulse = 1 + .12 * math.sin(ph)
        dot(d, c, c, r * pulse + 2, a(col, 60))
        dot(d, c, c, r * pulse, a(col, 190))
        dot(d, c, c, r * .55, a(mix(col, WHITE, .6), 255))
        motes = [0, 1, 2, 3][tier]
        for k in range(motes):
            ang = ph + k * TAU / max(1, motes)
            x, y = pt(c, c, r + 3, ang, .7)
            motif(d, fam, x, y, f, col)
        if tier == 3:
            ring(d, c, c, r + 5, a(col, 90), .7)
            px(d, *pt(c, c, r + 5, -ph, .7), WHITE)
    motif(d, fam, c, c, f, mix(col, WHITE, .8), inner=True)
    return glow(im, 1.2 + .3 * tier, 1.5)


def motif(d, fam, x, y, f, col, inner=False):
    """Family signature: a two-pixel hint of what the dimensions do."""
    if inner:
        if fam == 0:
            d.rectangle((round(x) - 1, round(y) - 1, round(x) + 1, round(y) + 1), outline=a(col, 200))
        elif fam == 1:
            px(d, x, y - 1 - f % 2, col)
        elif fam == 5:
            line(d, [(x, y), pt(x, y, 2, f * TAU / 8 - TAU / 4)], col)
        elif fam == 8:
            px(d, x - 1, y, col)
            px(d, x + 1, y, col)
        elif fam == 9:
            px(d, x + (f % 3) - 1, y + ((f // 3) % 3) - 1, col)
        else:
            px(d, x, y, col)
        return
    if fam in (1, 9):
        px(d, x, y - (f % 2), mix(col, GOLD, .5))
    elif fam == 2:
        line(d, [(x - 1, y), (x + 1, y)], col)
    elif fam == 3:
        px(d, x, y, (255, 200, 230, 255))
    else:
        px(d, x, y, col)


# -------------------------------------------------------- dimension glyphs --

PIPS = [{4}, {0, 8}, {0, 4, 8}, {0, 2, 6, 8}, {0, 2, 4, 6, 8}, {0, 2, 3, 5, 6, 8}, {1, 3, 4, 5, 7},
        {1, 3, 5, 7}, {0, 1, 2, 6, 7, 8}, {0, 1, 2, 3, 4, 5, 6, 7, 8}]


def frame_pts(fam, x, y):
    s = 5
    if fam == 0:   # Control: a notched lock square
        return [(x - s + 1, y - s), (x + s - 1, y - s), (x + s, y - s + 1), (x + s, y + s - 1), (x + s - 1, y + s),
                (x - s + 1, y + s), (x - s, y + s - 1), (x - s, y - s + 1)]
    if fam == 1:   # Ruin: a flame triangle
        return [(x, y - s - 1), (x + s, y + s), (x - s, y + s)]
    if fam == 2:   # Motion: a circle
        return [pt(x, y, s, k * TAU / 16) for k in range(16)]
    if fam == 3:   # Haven: a shield
        return [(x - s, y - s), (x + s, y - s), (x + s, y + 1), (x, y + s + 1), (x - s, y + 1)]
    if fam == 4:   # Passage: an arch
        return [(x - s, y + s)] + [pt(x, y - 1, s, math.pi + k * math.pi / 8) for k in range(9)] + [(x + s, y + s)]
    if fam == 5:   # Time: an hourglass
        return [(x - s, y - s), (x + s, y - s), (x + 1, y), (x + s, y + s), (x - s, y + s), (x - 1, y)]
    if fam == 6:   # Veil: an eye
        return [pt(x, y, s + 1, k * TAU / 16, .55) for k in range(16)]
    if fam == 7:   # Matter: a hexagon
        return [pt(x, y, s, k * TAU / 6 + TAU / 12) for k in range(6)]
    if fam == 8:   # Hunger: jaws
        return [(x - s, y - s + 1), (x - 2, y - 2), (x, y - s + 1), (x + 2, y - 2), (x + s, y - s + 1), (x + s, y + s - 1),
                (x + 2, y + 2), (x, y + s - 1), (x - 2, y + 2), (x - s, y + s - 1)]
    if fam == 9:   # Chaos: a star burst
        return [pt(x, y, s + 1 if k % 2 == 0 else s - 2, k * TAU / 16) for k in range(16)]
    if fam == 10:  # Gravity: a funnel, wide at the top
        return [(x - s - 1, y - s), (x + s + 1, y - s), (x + 2, y + s), (x - 2, y + s)]
    if fam == 11:  # Frost: a six-point crystal
        return [pt(x, y, s + 1 if k % 2 == 0 else 2, k * TAU / 12 - TAU / 4) for k in range(12)]
    if fam == 12:  # Momentum: a chevron
        return [(x - s, y - s), (x, y - s), (x + s, y), (x, y + s), (x - s, y + s), (x - 1, y)]
    if fam == 13:  # Pressure: a pinched box
        return [(x - s, y - s), (x, y - s + 2), (x + s, y - s), (x + s - 2, y), (x + s, y + s), (x, y + s - 2), (x - s, y + s), (x - s + 2, y)]
    if fam == 14:  # Radiance: a bright diamond
        return [pt(x, y, s + 1 if k % 4 == 0 else s - 1, k * TAU / 16) for k in range(16)]
    if fam == 15:  # Resonance: a wave ribbon
        top = [(x + k, y - 3 - round(1.5 * math.sin(k * math.pi / 5))) for k in range(-s, s + 1)]
        return top + [(px_, py_ + 7) for px_, py_ in reversed(top)]
    if fam == 16:  # Polarity: a bow-tie, two poles meeting
        return [(x - s, y - s), (x, y - 1), (x + s, y - s), (x + s, y + s), (x, y + 1), (x - s, y + s)]
    if fam == 17:  # Circuit: a cross
        return [(x - 2, y - s), (x + 2, y - s), (x + 2, y - 2), (x + s, y - 2), (x + s, y + 2), (x + 2, y + 2), (x + 2, y + s),
                (x - 2, y + s), (x - 2, y + 2), (x - s, y + 2), (x - s, y - 2), (x - 2, y - 2)]
    if fam == 18:  # Oath: a crown
        return [(x - s, y + s), (x - s, y - s), (x - 2, y - 1), (x, y - s - 1), (x + 2, y - 1), (x + s, y - s), (x + s, y + s)]
    # Genesis: a leaf
    return [(x, y - s - 1), (x + 3, y - 2), (x + s - 1, y + 2), (x, y + s), (x - s + 1, y + 2), (x - 3, y - 2)]


FAM_OF = [x['family'] for x in V.DATA['dimensions']]
FAM_START = {f: FAM_OF.index(f) for f in set(FAM_OF)}


def glyph(dim, f):
    im = Image.new('RGBA', (GLYPH, GLYPH))
    d = ImageDraw.Draw(im)
    fam = FAM_OF[dim]
    idx = dim - FAM_START[fam]                     # the dimension's place in its family
    col = FAMILY[fam]
    x, y = GLYPH // 2, 9           # floats above the rift centre
    pts = frame_pts(fam, x, y)
    pulse = [200, 220, 245, 255, 245, 220, 200, 190][f]
    poly(d, pts, fill=a(VOID, 190))
    line(d, pts + [pts[0]], a(col, pulse))
    for cell in PIPS[idx % 10]:
        r, q = divmod(cell, 3)
        bright = 255 if (cell + f) % 4 else 170
        px(d, x - 2 + q * 2, y - 2 + r * 2 + (1 if fam == 1 else 0), a(WHITE, bright))
    i = f * len(pts) // 8
    px(d, *pts[i % len(pts)], WHITE)
    # a fine tether toward the rift
    px(d, x, y + 8 + f % 2, a(col, 120))
    return glow(im, .8, 1.3)


# ------------------------------------------------------------ link threads --

def link(look, h, f):
    """The chain between two rifts: a hairline fracture running through the glass of reality, with a
    pulse of light travelling along it from Vesper toward the node."""
    im = Image.new('RGBA', (LINK, LINK))
    d = ImageDraw.Draw(im)
    c = LINK // 2
    ux, uy = unit(h)
    nx, ny = -uy, ux
    fade = lambda s: max(0.0, min(1.0, (42 - abs(s)) / 10))
    col = GOLD_WHITE if look == 8 else EDGE
    jag = 1.4 if look < 3 else 1.0
    spine = []
    for k in range(-14, 15):
        s = k * 3
        o = (hsh(200 + look, k) - .5) * 2 * jag if abs(k) < 14 else 0
        spine.append((s, o))
    for (s0, o0), (s1, o1) in zip(spine, spine[1:]):
        al = 255 * min(fade(s0), fade(s1))
        if look == 0 and hsh(s0, f, 2) < .35:
            continue                      # Spark: a broken, flickering hairline
        p0 = (c + ux * s0 + nx * o0, c + uy * s0 + ny * o0)
        p1 = (c + ux * s1 + nx * o1, c + uy * s1 + ny * o1)
        if look >= 4:                     # violet showing through the deeper fractures
            line(d, [(p0[0] + nx, p0[1] + ny), (p1[0] + nx, p1[1] + ny)], a(MIDV, al * .8))
        line(d, [p0, p1], a(col, al * (.7 if look < 2 else 1)))
    if look >= 2:                         # little branch cracks off the line
        for k in range(2 + look // 2):
            s = ((hsh(210 + look, k) * 80) - 40)
            o = (hsh(200 + look, int(s // 3)) - .5) * 2 * jag
            base = (c + ux * s + nx * o, c + uy * s + ny * o)
            sgn = 1 if k % 2 else -1
            tip = (base[0] + nx * sgn * 3 + ux * 2, base[1] + ny * sgn * 3 + uy * 2)
            line(d, [base, tip], a(col, 200 * fade(s)))
    pulse = (f * 12) % 96 - 48
    dot(d, c + ux * pulse, c + uy * pulse, 1, a(WHITE, 255 * fade(pulse)))
    if look >= 7:
        for k in range(-3, 4):
            s = k * 16 + (f * 2) % 16 - 4
            if fade(s) > .5 and (k + f) % 2 == 0:
                sparkle(d, c + ux * s, c + uy * s, a(WHITE if look == 7 else GOLD_WHITE, 255 * fade(s)), 1)
    return glow(im, .7, 1.4) if look >= 2 else im


# ------------------------------------------------------------------ packets --

def packet(look, h, f):
    """A rift punch in flight: her chrome gauntlet fist, knuckles first, dragging a tear through reality.
    Higher looks crack the plate, split its light and let the void show, like her own gauntlets."""
    im = Image.new('RGBA', (PACKET, PACKET))
    d = ImageDraw.Draw(im)
    c = PACKET // 2
    ux, uy = unit(h)
    nx, ny = -uy, ux
    P = lambda s, o=0: (c + ux * s + nx * o, c + uy * s + ny * o)
    S = V.STEEL
    col = GOLD if look == 8 else RIFT_PALE
    # the tear it drags: two fracture lines closing behind the cuff
    trail = 6 + look
    for s in range(trail):
        w = 2.6 * (1 - s / trail)
        wob = math.sin(s * .7 + f * TAU / 8) * .5
        al = 230 - s * (200 // trail)
        w = 4 * (1 - s / trail)
        px(d, *P(-8 - s, w + wob), a(EDGE if s % 3 else col, al))
        px(d, *P(-8 - s, -w + wob), a(EDGE if (s + 1) % 3 else col, al))
        if look >= 4 and s % 2 == 0:
            px(d, *P(-8 - s, wob * .5), a(VOIDC if look >= 7 else MIDV, al))
    for k in range(2 + look // 3):                                  # glass shards shed by the tear
        s = 8 + (k * 5 + f * 2) % (trail + 4)
        o = (hsh(look, k, h) - .5) * 8
        px(d, *P(-s, o), a(WHITE if k % 2 else CRYSTAL, 220 - s * 6))
    # the cuff, then the fist: a big chrome fist, four knuckles leading, the thumb wrapped over the side
    poly(d, [P(-7, -3.4), P(0, -4), P(0, 4), P(-7, 3.4)], fill=S['dark'], outline=INK)
    line(d, [P(-6, -2.4), P(-1, -2.8)], S['metal'])
    px(d, *P(-3.5, 0), V.HEX)
    px(d, *P(-3.5, 1), V.HEX_D)
    poly(d, [P(0, -5), P(8, -5), P(9, -4), P(9, 4), P(8, 5), P(0, 5)], fill=S['metal'], outline=INK)
    line(d, [P(1, -3.8), P(7.5, -3.8)], S['light'])
    for o in (-3.6, -1.2, 1.2, 3.6):                                 # the knuckles
        dot(d, *P(9.4, o), 1.3, S['trim_l'], outline=INK)
        line(d, [P(5, o * .9), P(8.4, o)], S['trim_d'])
    poly(d, [P(1.5, 5), P(6, 5), P(6.5, 6.5), P(2, 6.5)], fill=S['trim'], outline=INK)   # thumb over the side
    # rank: cracks, then split light, then the void through the plate
    if look >= 2:
        line(d, [P(1.5, -2.5), P(4, .3), P(6, -1)], MIDV)
    if look >= 4:
        px(d, *P(10.5, -4.6), V.FRINGE_C)
        px(d, *P(10.5, 4.6), V.FRINGE_M)
        px(d, *P(-1, -5), V.FRINGE_C)
        px(d, *P(-1, 5), V.FRINGE_M)
    if look >= 7:
        px(d, *P(4, .3), VOIDC)
        px(d, *P(5, 1.2), VOIDC)
        r = 8 + (f % 4)
        for t_ in range(0, 12, 2):                                  # the hole it punches ahead of itself
            ang = t_ * TAU / 12 + f * .2
            px(d, *P(12 + .3 * r * math.cos(ang), r * .7 * math.sin(ang)), a(GOLD if look == 8 else EDGE, 200))
    # the impact flash at the knuckles on the leading frames
    if f % 4 == 0:
        sparkle(d, *P(12), a(WHITE, 230), 1)
    return glow(im, .7, 1.3) if look >= 2 else im


def exit_(look, f):
    """The fist bursting out of the last rift toward its target: glass blown outward and a knuckle flash."""
    im = Image.new('RGBA', (EXIT, EXIT))
    d = ImageDraw.Draw(im)
    c = EXIT // 2
    life = [1, .95, .85, .7, .55, .4, .25, .1][f]
    r = 5 + f * (2 + look * .2)
    for k in range(10 + look):
        ang = k * TAU / (10 + look) + hsh(look, k) * .4
        q0, q1 = pt(c, c, r, ang), pt(c, c, r + 3 + (k % 3), ang)
        line(d, [q0, q1], a(EDGE if k % 2 else (GOLD if look == 8 else RIFT_PALE), 255 * life))
    if f < 3:
        dot(d, c, c, 5 - f, a(WHITE, 240))
        for k in range(4):                                          # four knuckle sparks
            sparkle(d, *pt(c, c, 4 + f * 2, k * TAU / 4 + TAU / 8), a(V.STEEL['trim_l'], 230), 1)
    if look >= 4:
        ring(d, c, c, r * .7, a(V.FRINGE_C if f % 2 else V.FRINGE_M, 160 * life))
    if look >= 7:
        dot(d, c, c, max(1, 3 - f // 2), a(VOIDC, 220 * life))
    return glow(im, .8, 1.4) if look >= 2 else im


# ------------------------------------------------------------- impacts etc --

def impact(look, f):
    im = Image.new('RGBA', (IMPACT, IMPACT))
    d = ImageDraw.Draw(im)
    c = IMPACT // 2
    life = [.3, .7, 1, .95, .8, .6, .4, .2][f]
    r = (5 + look * 1.6) * (.5 + f / 10)
    col = GOLD_WHITE if look == 8 else RIFT_PALE
    dot(d, c, c, max(1, r * .45 * life), a(WHITE, 255 * life))
    if look == 0:
        for k in range(5):
            ang = hsh(3, k) * TAU
            px(d, *pt(c, c, r + (f % 2), ang), a(EMBER, 255 * life))
    elif look == 1:
        line(d, [(c, c - r * 1.4), (c, c + r * 1.4)], a(col, 230 * life))
    elif look == 2:
        for k in range(6):
            ang = k * TAU / 6 + .3
            line(d, [pt(c, c, r * .5, ang), pt(c, c, r * (1.1 + .3 * (k % 2)), ang + .2)], a(col, 230 * life))
    elif look == 3:
        ring(d, c, c, r, a(V.GOLD, 230 * life))
        for k in range(4):
            px(d, *pt(c, c, r, k * TAU / 4 + f * .2), a(WHITE, 255 * life))
    elif look == 4:
        ring(d, c, c, r, a(col, 200 * life), .8)
        for k in range(5):
            sparkle(d, *pt(c, c, r * 1.2, k * TAU / 5 + f * .3, .8), a(WHITE, 220 * life), 1)
    elif look == 5:
        for pth in V.cracks(66, c, c, 5, r * 1.2, TAU, f * .1, True):
            line(d, pth[:4], a(EDGE, 220 * life))
    elif look == 6:
        ring(d, c, c, r, a(col, 220 * life))
        ring(d, c + 1, c - 1, r * .7, a(RIFT, 200 * life))
        for k in range(4):
            px(d, *pt(c, c, r * 1.4 + f, k * TAU / 4 + .7), a(RIM, 230 * life))
    else:
        for k in range(8):
            ang = k * TAU / 8 + .2
            q0, q1 = pt(c, c, r * .6, ang), pt(c, c, r * 1.5, ang)
            poly(d, [q0, pt(c, c, r * 1.1, ang + .15), q1], fill=a(col if look == 8 else (220, 236, 255), 220 * life))
        ring(d, c, c, r * .8, a(GOLD if look == 8 else WHITE, 220 * life))
    px(d, c - 8 + f * 2, c + r + 2, a(col, 120))
    return glow(im, .9, 1.6)


def open_(look, f):
    """Telegraph: cracks converge on the point where the rift will open."""
    im = Image.new('RGBA', (OPEN, OPEN))
    d = ImageDraw.Draw(im)
    c = OPEN // 2
    prog = f / 7
    rin = 32 * (1 - prog)
    col = GOLD_WHITE if look == 8 else EDGE
    n = [3, 4, 5, 6, 6, 6, 8, 8, 9][look]
    for k in range(n):
        ang = k * TAU / n + .4
        if look == 7:   # glass shards flying in
            x, y = pt(c, c, rin + 4, ang)
            V.shard(d, x, y, 1.6, (220, 236, 255, 255))
            line(d, [pt(c, c, rin + 8, ang), pt(c, c, rin + 14, ang)], a(WHITE, 120))
            continue
        pts = []
        for j in range(6):
            r = 36 - (36 - rin) * j / 5
            wob = (hsh(k, j, f) - .5) * (4 if look == 0 else 2 if look < 3 else 1)
            pts.append(pt(c, c, r, ang + wob / max(4, r)))
        al = [255, 130, 230, 100, 255, 160, 210, 120][f] if look == 0 else 230
        line(d, pts, a(col, al))
        if look >= 6:
            line(d, [(x + 1, y - 1) for x, y in pts], a(MIDV, 160))
        if look == 4:
            px(d, *pts[-1], WHITE)
    if look == 0:
        for k in range(2):
            px(d, *pt(c, c, rin + 2, hsh(9, f, k) * TAU), a(EMBER, 230))
    dot(d, c, c, .5 + prog * 2, a(WHITE, 120 + 130 * prog))
    if f == 7:      # it pops open
        V.scar(d, (c, c - 5), (c + 1, c + 5), 1.4, 33 + look)
    return glow(im, .8, 1.4) if look >= 2 else im


def close_(look, f):
    """The scar zips shut from both ends and sheds glass flakes."""
    im = Image.new('RGBA', (CLOSE, CLOSE))
    d = ImageDraw.Draw(im)
    c = CLOSE // 2
    life = 1 - f / 8
    edge = GOLD_WHITE if look == 8 else EDGE
    if look == 0:   # fizzles out
        al = [200, 160, 180, 90, 110, 50, 40, 15][f]
        line(d, [(c, c - 12), (c + 1, c - 5), (c - 1, c + 3), (c, c + 12)], a(EDGE, al))
        for k in range(3):
            px(d, c + (k - 1) * 3 + (f % 2), c - 4 + f * 2 + k, a(EMBER, max(20, 220 - f * 28)))
        return im
    h = [14, 10, 6, 3, 0, 0, 0, 0][f]
    if h:
        V.scar(d, (c, c - h), (c + 1, c + h), [3, 2.4, 1.6, .8][f] * (1 + look / 10), 44 + look, f, edge=edge)
    else:
        line(d, [(c, c - 3), (c + 1, c), (c, c + 3)], a(edge, 255 * life))
    for k in range(3 + look // 2):      # flakes of glass falling away
        t = f
        x = c - 6 + k * 3 + (hsh(look, k) - .5) * 4
        y = c - 4 + t * 2 + .3 * t * t + k
        V.shard(d, x, y, 1, a(edge, 230 * life), outline=a(MIDV, 200 * life))
    px(d, c - 10 + f * 2, c + 12, a(edge, 80))
    return glow(im, .7, 1.4)


def break_(look, f):
    """Shatter: rift-glass fragments fly out."""
    im = Image.new('RGBA', (BREAK, BREAK))
    d = ImageDraw.Draw(im)
    c = BREAK // 2
    col = GOLD_WHITE if look == 8 else EDGE
    life = [1, 1, .9, .8, .65, .5, .35, .2][f]
    n = 5 + look
    if f <= 1:
        for pth in V.cracks(55 + look, c, c, n, 10 + 6 * f, TAU, 0, False):
            line(d, pth[:4], a(col, 255))
    for k in range(n):
        ang = k * TAU / n + hsh(look, k) * .4
        r = 4 + f * (3.5 + look * .3)
        x, y = pt(c, c, r, ang)
        y += f * f * .3
        s = 2 + (k % 3)
        spin = f * .6 + k
        pts = [pt(x, y, s, spin), pt(x, y, s * .7, spin + 2.2), pt(x, y, s, spin + 4)]
        poly(d, pts, fill=a(col, 200 * life), outline=a(WHITE, 255 * life))
    if look == 0:
        for k in range(4):
            px(d, *pt(c, c, 3 + f * 2, hsh(8, k) * TAU), a(EMBER, 230 * life))
    if f in (1, 2):
        dot(d, c, c, 3, a(WHITE, 230))
    px(d, c - 12 + f * 3, c + 20, a(col, 80))
    return glow(im, .8, 1.4)


def seedhint(look, f):
    """Barely visible: a few faint shimmering pixels where a seed waits."""
    im = Image.new('RGBA', (HINT, HINT))
    d = ImageDraw.Draw(im)
    c = HINT // 2
    col = GOLD_WHITE if look == 8 else RIM
    base = [30, 40, 50, 40, 30, 25, 20, 25][f]
    for k in range(3):
        ang = k * TAU / 3 + f * TAU / 24 + look * .2
        px(d, *pt(c, c, 2 + (f + k) % 3, ang), a(col, base + 10 * (k == f % 3)))
    px(d, c, c, a(col, base + 10))
    px(d, c - 6 + f, c + 6, a(col, 18))
    return im


# --------------------------------------------------------------- techniques --

TIER = [
    dict(main=(176, 156, 208, 255), hi=(236, 222, 176, 255), dark=(62, 52, 84, 255), jit=2,
         flick=[255, 150, 255, 110, 235, 170, 255, 95], prog=[0, 1, 1, 3, 4, 4, 6, 7]),
    dict(main=V.RIFT, hi=V.RIFT_PALE, dark=V.RIFT_DEEP, jit=1,
         flick=[255, 225, 255, 205, 255, 230, 255, 215], prog=[0, 1, 2, 2, 4, 5, 6, 7]),
    dict(main=(160, 144, 255, 255), hi=(232, 224, 255, 255), dark=(52, 38, 112, 255), jit=0,
         flick=[255] * 8, prog=list(range(8))),
    dict(main=(192, 212, 255, 255), hi=WHITE, dark=(60, 72, 142, 255), jit=0,
         flick=[255] * 8, prog=list(range(8))),
    dict(main=GOLD, hi=GOLD_WHITE, dark=(124, 86, 32, 255), jit=0,
         flick=[255] * 8, prog=list(range(8))),
]


class Tech:
    def __init__(self, tier, f, seed):
        self.t, self.f, self.seed = tier, f, seed
        p = TIER[tier]
        self.main, self.hi, self.dark = p['main'], p['hi'], p['dark']
        self.al = p['flick'][f] / 255
        self.p = p['prog'][f] / 7            # stuttering progress at Spark tier
        self.jit = p['jit']

    def j(self, k=0):
        return (hsh(self.seed, self.f, k) - .5) * 2 * self.jit

    def c(self, col, alpha=255):
        return a(col, alpha * self.al)


def sil(d, x, y, col, s=1.0, out=None):
    """A tiny person silhouette (feet at y)."""
    dot(d, x, y - 9 * s, 1.6 * s, col)
    poly(d, [(x - 2 * s, y - 7 * s), (x + 2 * s, y - 7 * s), (x + 2.4 * s, y - 2 * s), (x + 1 * s, y), (x - 1 * s, y), (x - 2.4 * s, y - 2 * s)], fill=col, outline=out)


def mini_rift(d, x, y, h, w, T, alpha=255):
    """A small torn scar: fractured white-cyan edge, violet, a void line down the middle."""
    pts = [(x, y - h), (x + w, y - h * .3), (x + w * .6, y + h * .3), (x, y + h), (x - w, y + h * .2), (x - w * .6, y - h * .4)]
    poly(d, pts, fill=T.c(MIDV, alpha * .9), outline=T.c(GOLD_WHITE if T.t == 4 else EDGE, alpha))
    line(d, [(x, y - h * .7), (x + (1 if w > 2 else 0), y), (x, y + h * .7)], T.c(VOIDC, alpha))


def muzzle(d, x, y, r, T, ang=0.0, alpha=255):
    dot(d, x, y, r * .6, T.c(WHITE, alpha))
    for k in range(5):
        aa = ang + (k - 2) * .45
        line(d, [pt(x, y, r * .5, aa), pt(x, y, r * (1.2 + .3 * (k % 2)), aa)], T.c(T.hi, alpha))


def mini_fist(d, x, y, ang, T, k=1.0, alpha=255):
    """A small chrome gauntlet fist at (x, y), knuckles toward `ang`: her punches through rifts, not shots."""
    ux, uy = math.cos(ang), math.sin(ang)
    nx, ny = -uy, ux
    P = lambda s, o=0: (x + (ux * s + nx * o) * k, y + (uy * s + ny * o) * k)
    S = V.STEEL
    poly(d, [P(-4, -2.2), P(0, -2.6), P(0, 2.6), P(-4, 2.2)], fill=a(S['dark'], alpha), outline=a(INK, alpha))
    poly(d, [P(0, -3.2), P(5, -3.2), P(5.8, -2.4), P(5.8, 2.4), P(5, 3.2), P(0, 3.2)], fill=a(S['metal'], alpha), outline=a(INK, alpha))
    line(d, [P(5.6, -2.2), P(5.6, 2.2)], a(S['trim_l'], alpha))
    line(d, [P(1, -2.2), P(4.5, -2.2)], a(S['light'], alpha))
    px(d, *P(-2, 0), a(V.HEX, alpha))
    if T.t >= 2:
        px(d, *P(6.4, -2.8), T.c(T.hi, alpha))


def kidnap_warn(tier, f):
    """Kidnap's armed field (0.5 s): a cracked-glass ring closing in on the enemy, cracks pointing inward, a pulse
    on every frame so the game's players (and its AI) can read it and step out."""
    im = Image.new('RGBA', (TECH, TECH))
    d = ImageDraw.Draw(im)
    c = TECH // 2
    T = Tech(tier, f, 211)
    r = 29 - f * .5                                    # the field radius: 27200 units ~ 29 px
    d.ellipse((c - r, c - r * .55, c + r, c + r * .55), fill=T.c(MIDV, 60 + f * 8))
    ring(d, c, c, r, T.c(EDGE, 230), .55)
    ring(d, c, c, r - 1.5, T.c(T.main, 150), .55)
    for k in range(12):                                # cracks reaching in from the rim
        ang = k * TAU / 12 + hsh(tier, k) * .3
        q0 = pt(c, c, r, ang, .55)
        q1 = pt(c, c, r - 4 - (f + k) % 4 - f * .8, ang + .08, .55)
        line(d, [q0, q1], T.c(EDGE if k % 2 else T.hi, 220))
    if f % 2 == 0:                                     # the closing pulse
        ring(d, c, c, r * (1 - f / 10), T.c(WHITE, 120 + f * 15), .55)
    if f >= 6:
        dot(d, c, c, 2 + (f - 6) * 2, T.c(T.hi, 200))
    return glow(im, .6, 1.3) if tier >= 2 else im


def kidnap_miss(tier, f):
    """The field shatters on nothing: the ring breaks into falling glass."""
    im = Image.new('RGBA', (TECH, TECH))
    d = ImageDraw.Draw(im)
    c = TECH // 2
    T = Tech(tier, f, 223)
    life = 1 - f / 8
    for k in range(14):
        ang = k * TAU / 14 + hsh(tier, k, 3) * .4
        x, y = pt(c, c, 30 + f * .5, ang, .55)
        y += f * f * .35 + hsh(k, 9) * 3                # shards fall
        line(d, [(x, y), (x + 1, y + 2)], T.c(EDGE if k % 2 else T.main, 230 * life))
    return im


def shock(d, x, y, r, T, flat=.45, alpha=255):
    ring(d, x, y, r, T.c(T.hi, alpha), flat)
    if T.t >= 2:
        ring(d, x, y, r * .8, T.c(T.main, alpha * .6), flat)


def tech(name, tier, f):
    im = Image.new('RGBA', (TECH, TECH))
    d = ImageDraw.Draw(im)
    c = TECH // 2
    T = Tech(tier, f, TECHS.index(name) * 31 + 7)
    p, ph = T.p, f * TAU / 8
    jx, jy = T.j(1), T.j(2)
    life = 1 - max(0, f - 5) / 4
    if name == 'chain':
        # a fist driven into the chain's first rift, the tear it drags behind it
        x = c - 22 + 40 * min(1, p * 1.3)
        line(d, [(c - 24, c + jy), (x - 5, c + jy)], T.c(T.main, 150), 1)
        mini_rift(d, c + 22, c, 9, 3, T)
        if x < c + 20:
            mini_fist(d, x, c + jy, 0, T, 1.2)
        ring(d, c + 22, c, 6 + f % 3, T.c(T.hi, 160))
        for k in range(3):
            q = c - 14 + k * 10 + (f * 2) % 10
            ring(d, q, c, 2, T.c(T.hi, 200), .6, 8)
    elif name == 'crossfire':
        for k in range(3):
            ang = k * TAU / 3 - TAU / 4
            ex, ey = pt(c, c, 28, ang)
            mini_rift(d, ex, ey, 6, 2, T)
            q = pt(ex, ey, 6 + 16 * p, ang + math.pi)                # a fist punches out of each rift, inward
            line(d, [(ex, ey), q], T.c(T.main, 140), 1)
            mini_fist(d, *q, ang + math.pi, T)
        dot(d, c, c, 1 + 3 * p, T.c(WHITE, 200 * p))
    elif name == 'bank':
        mini_rift(d, c + 4, c - 18, 3, 12, T)
        x0, y0 = c - 30, c + 16
        mid = (c + 4, c - 16)
        x1, y1 = c + 32, c + 18
        k = min(1, p * 2)
        q = (x0 + (mid[0] - x0) * k, y0 + (mid[1] - y0) * k)
        line(d, [(x0, y0), q], T.c(T.main, 140), 1)
        if p > .5:                                                  # the fist banks off the pane and goes on
            k2 = (p - .5) * 2
            q = (mid[0] + (x1 - mid[0]) * k2, mid[1] + (y1 - mid[1]) * k2)
            line(d, [mid, q], T.c(T.main, 140), 1)
            mini_fist(d, *q, math.atan2(y1 - mid[1], x1 - mid[0]), T)
        else:
            mini_fist(d, *q, math.atan2(mid[1] - y0, mid[0] - x0), T)
        if f < 3:
            shock(d, x0 + jx, y0 + jy, 3 + f * 2, T)
        if 3 <= f <= 5:
            sparkle(d, *mid, T.c(WHITE), 3)
    elif name == 'rain':
        mini_rift(d, c, c - 34, 3, 26, T, 200)
        for k in range(6):
            x = c - 22 + k * 9 + T.j(k)
            y = c - 30 + ((f * 9 + k * 13) % 56)
            line(d, [(x, y - 6), (x, y)], T.c(T.hi), 1 + (tier >= 3))
            if y > c + 18:
                ring(d, x, c + 24, 2 + (y - c - 18) / 4, T.c(T.main, 180), .4, 10)
    elif name == 'volley':
        # a flurry: five fists out of the slash, fanning
        if f < 3:
            shock(d, c - 24 + jx, c + jy, 3 + f * 2, T)
        for k in range(5):
            ang = (k - 2) * .28
            reach = 8 + 40 * min(1, p * 1.2 + (k % 2) * .08)
            line(d, [pt(c - 24, c, 6, ang), pt(c - 24, c, reach - 4, ang)], T.c(T.main, 120), 1)
            mini_fist(d, *pt(c - 24, c, reach, ang), ang, T, .9)
    elif name == 'implode':
        rr = 30 * (1 - p) + 3
        for k in range(8):
            ang = k * TAU / 8 + .2
            line(d, [pt(c, c, rr + 10, ang, .55), pt(c, c, rr, ang + .15, .55)], T.c(T.hi), 1)
        dot(d, c, c, max(1, 6 * p), T.c(T.dark, 230))
        if f >= 4:
            for k in range(1 + tier // 2 + 1):
                shock(d, c, c, (f - 3) * (6 + k * 4), T, .5, 255 * life)
        else:
            shock(d, c, c, rr, T, .5)
    elif name == 'domino':
        for k in range(4):
            x = c - 33 + k * 22
            start = k * 1.7
            if f >= start:
                q = min(1, (f - start) / 3)
                dot(d, x + T.j(k), c + 6, 7 * q, T.c(T.dark, 220))
                ring(d, x, c + 6, 4 + 7 * q, T.c(T.hi, 255 * (1 - q * .5)), .5)
                for j in range(3):
                    px(d, x - 4 + j * 4, c + 4 - q * 6 - j, T.c(T.main))
            else:
                line(d, [(x - 3, c + 6), (x + 3, c + 6)], T.c(T.main, 120))
    elif name == 'opunch':
        # Overload Punch: stress vents into the knuckles (anticipation), a white impact, then a heavy shock
        # cone forward that eases out while sparks fall
        x = c - 6
        for k in range(4):
            d.rectangle((x - 2, c - 7 + k * 4, x + 1, c - 5 + k * 4), fill=T.c(T.main), outline=T.c(WHITE))
        if f == 0:
            for k in range(6):
                ang = k * TAU / 6
                line(d, [pt(x, c, 16, ang), pt(x, c, 10, ang)], T.c((255, 160, 90), 220))
        elif f == 1:
            dot(d, x + 4, c, 6, T.c(WHITE))
        else:
            e = [0, .5, .75, .88, .95, .98, 1, 1][f - 1]
            fade = [1, 1, 1, .85, .65, .45, .28, .12][f]
            for k in range(6 + tier * 2):
                ang = (k / (5 + tier * 2) - .5) * 1.4
                pts = [pt(x + 3, c, 4, ang)]
                for j in range(1, 4):
                    pts.append(pt(x + 3, c, 4 + j * (5 + 6 * e), ang + (hsh(k, j, f) - .5) * .4))
                line(d, pts, T.c((255, 160, 90) if k % 2 else T.hi, 255 * fade))
            ring(d, x + 3, c, 10 + 20 * e, T.c(T.hi, 220 * fade), .9, 16, -1.1, 1.1, 2)
            for k in range(4):
                t = f - 2
                px(d, x + 8 + k * 5 + t, c - 6 + k * 3 + .5 * t * t, T.c((255, 160, 90), 230 * fade))
    elif name == 'step':
        x0, x1 = c - 20, c + 20
        if f < 4:
            sil(d, x0 + jx, c + 14, T.c(T.hi, 255 * (1 - f / 4)))
            ring(d, x0, c + 4, 3 + f * 3, T.c(WHITE, 200))
        else:
            sil(d, x1 + jx, c + 14, T.c(T.hi, 255 * (f - 3) / 4))
            ring(d, x1, c + 4, 3 + (7 - f) * 3, T.c(WHITE, 200))
        if 2 <= f <= 5:
            line(d, [(x0, c + 4), (x1, c + 4)], T.c(T.main, 180), 1 + (tier >= 3))
    elif name == 'gate':
        h = 16 + 8 * min(1, p * 2)
        pts = [(c - 12, c + 20)] + [pt(c, c + 20 - h + 12, 12, math.pi + k * math.pi / 10) for k in range(11)] + [(c + 12, c + 20)]
        poly(d, pts, fill=T.c(T.dark, 200))
        line(d, pts, T.c(T.hi), 1 + (tier >= 2))
        if f >= 3:
            sil(d, c + (f - 3) * 3 + jx, c + 20, T.c(WHITE, 230), 1.2)
        for k in range(4):
            px(d, *pts[(k * 3 + f) % len(pts)], T.c(WHITE))
    elif name == 'skydrop':
        mini_rift(d, c, c - 34, 3, 18, T)
        if f < 5:
            y = c - 30 + (f / 4) ** 2 * 48
            sil(d, c + jx, y + 10, T.c(WHITE, 230), 1.1)
            line(d, [(c, c - 32), (c, y - 2)], T.c(T.main, 120))
        else:
            sil(d, c + jx, c + 20, T.c(WHITE, 230 * life), 1.1)
            for k in range(1 + tier // 2):
                shock(d, c, c + 20, (f - 4) * 8 + k * 5, T, .4, 255 * life)
    elif name == 'kidnap':
        if f < 4:
            mini_rift(d, c - 16, c, 12 * min(1, (f + 1) / 2), 5, T)
            s = 1 - f / 4
            sil(d, c - 16 + (1 - s) * 2 + jx, c + 10 * s, T.c(WHITE, 230 * s + 25), max(.3, s))
        else:
            mini_rift(d, c + 16, c, 12, 5, T)
            k = (f - 4) / 3
            sil(d, c + 16 + k * 18 + jx, c + 10 - math.sin(k * math.pi) * 8, T.c(WHITE, 230), .4 + .6 * k)
            for j in range(3):
                px(d, c + 16 + k * 18 - 4 - j * 3, c + 6 - j, T.c(T.hi))
    elif name == 'sinktrap':
        for k in range(3):
            ring(d, c, c + 6, 26 - k * 7, T.c(T.main, 200), .45, 16, ph * (k + 1) * .5 + k, ph * (k + 1) * .5 + k + 3.5)
        dot(d, c, c + 6, 8, T.c(T.dark, 200))
        im2 = Image.new('RGBA', (TECH, TECH))
        sil(ImageDraw.Draw(im2), c + jx, c + 8 + p * 10, T.c(WHITE, 230 * (1 - p * .7)))
        im2.paste((0, 0, 0, 0), (0, c + 9, TECH, TECH))
        im.alpha_composite(im2)
        d = ImageDraw.Draw(im)
        ring(d, c, c + 6, 9, T.c(T.hi), .45)
    elif name == 'swap':
        k = p
        xa, xb = c - 20 + 40 * k, c + 20 - 40 * k
        arc = math.sin(k * math.pi) * 10
        sil(d, xa + jx, c + 12 - arc, T.c(T.hi, 230))
        sil(d, xb - jx, c + 12 + arc * .4, T.c(T.main, 230))
        ring(d, c, c, 20, T.c(T.hi, 150), .45, 24, ph, ph + 2.5)
        ring(d, c, c, 20, T.c(T.main, 150), .45, 24, ph + math.pi, ph + math.pi + 2.5)
    elif name == 'fling':
        k = p
        x = c - 28 + 56 * k
        y = c + 14 - math.sin(k * math.pi) * 30
        pts = [(c - 28 + 56 * t / 10, c + 14 - math.sin(t / 10 * math.pi) * 30) for t in range(round(k * 10) + 1)]
        if len(pts) > 1:
            line(d, pts, T.c(T.main, 180))
        im2 = Image.new('RGBA', (24, 24))
        sil(ImageDraw.Draw(im2), 12, 18, T.c(WHITE, 230))
        im.alpha_composite(im2.rotate(-f * 45, resample=Image.Resampling.NEAREST), (round(x + jx) - 12, round(y) - 14))
        d = ImageDraw.Draw(im)
    elif name == 'hook':
        reel = p
        hx = c + 30 - 48 * reel * (f >= 3)
        if f < 3:
            hx = c - 18 + 48 * (f / 2)
        line(d, [(c - 30, c), (hx, c + jy)], T.c(T.main), 1)
        for k in range(0, int(abs(hx - (c - 30))), 6):
            px(d, c - 30 + k, c - 1 + (k // 6 + f) % 2, T.c(T.hi))
        line(d, [(hx, c + jy), (hx + 3, c + 3 + jy), (hx, c + 6 + jy), (hx - 2, c + 4 + jy)], T.c(WHITE), 1)
        line(d, [(hx, c + jy), (hx - 3, c - 3 + jy)], T.c(WHITE), 1)
        if f >= 3:
            sil(d, hx + 6, c + 14, T.c(T.hi, 200))
        mini_rift(d, c - 32, c, 7, 2, T)
    elif name == 'phase':
        for k in range(3):
            r = ((f + k * 2.6) % 8) * 4 + 2
            ring(d, c, c + 12, r, T.c(T.hi, 220 * (1 - r / 34)), .4)
        depth = p * 14
        im2 = Image.new('RGBA', (TECH, TECH))
        sil(ImageDraw.Draw(im2), c + jx, c + 10 + depth, T.c(WHITE, 230))
        im2.paste((0, 0, 0, 0), (0, c + 12, TECH, TECH))
        im.alpha_composite(im2)
        d = ImageDraw.Draw(im)
    elif name == 'curtain':
        for k in range(12):
            x = c - 24 + k * 4
            off = math.sin(ph + k * .6) * 2
            al = (120 + 100 * math.sin(ph * 1 + k)) * (.6 if tier == 0 and (k + f) % 3 == 0 else 1)
            line(d, [(x + off, c - 24), (x - off, c + 20)], T.c(T.hi if k % 2 else T.main, max(40, al)))
        line(d, [(c - 26, c + 20), (c + 26, c + 20)], T.c(WHITE, 200))
    elif name == 'reflect':
        poly(d, [(c - 4, c - 22), (c + 4, c - 18), (c + 4, c + 22), (c - 4, c + 18)], fill=T.c((210, 230, 255), 80), outline=T.c(T.hi))
        if f < 4:
            x = c - 30 + f * 7
            line(d, [(x - 8, c - 6), (x, c - 4)], T.c(T.main), 2)
        else:
            x = c - 6 - (f - 4) * 8
            line(d, [(c - 4, c - 2), (x, c + 2 + (f - 4) * 2)], T.c(T.main), 2)
        if 3 <= f <= 5:
            dot(d, c, c - 2, 5 - abs(f - 4) * 2, T.c(WHITE))
            sparkle(d, c, c - 2, T.c(WHITE), 6)
    elif name == 'rescue':
        y = c - 20 + min(1, p * 1.6) * 30
        ring(d, c, c + 14, 12, T.c(T.hi, 200), .4, 20, 0, math.pi)
        sil(d, c + jx, y + 2, T.c(WHITE, 230))
        dot(d, c, y - 4, 10, T.c(T.main, 50))
        if f >= 5:
            for k in range(5):
                px(d, c - 8 + k * 4, c + 12 - (f - 5) * 3 - k % 2, T.c(T.hi))
    elif name == 'pocket':
        r = 8 + 10 * min(1, p * 1.5)
        dot(d, c, c, r, T.c(T.main, 50))
        ring(d, c, c, r, T.c(T.hi), 1)
        line(d, [pt(c, c, r - 3, -2.3), pt(c, c, r - 3, -1.6)], T.c(WHITE))
        sil(d, c + jx, c + 7, T.c(WHITE, 200), .9)
        px(d, *pt(c, c, r, ph), T.c(WHITE))
    elif name == 'lifeline':
        ax, ay = c + 26, c - 14
        mini_rift(d, ax, ay, 6, 2, T)
        line(d, [(ax, ay + 7), (ax, ay + 10), (ax + 2, ay + 12)], T.c(T.hi))
        k = min(1, p * 1.4)
        pts = [(c - 20 + (ax - c + 20) * t / 12, c + 10 + (ay - c - 10) * t / 12 + math.sin(t + ph) * 1.5) for t in range(round(12 * k) + 1)]
        if len(pts) > 1:
            line(d, pts, T.c(T.hi), 1 + (tier >= 3))
        sil(d, c - 20 + jx, c + 18, T.c(WHITE, 230))
    elif name == 'escort':
        xa, xb = c - 18, c + 18
        sil(d, xa + jx, c + 14, T.c(T.hi, 230))
        sil(d, xb, c + 14, T.c(WHITE, 230))
        pts = [(xa + (xb - xa) * t / 16, c + 3 + math.sin(t * .8 - ph * 2) * 2) for t in range(17)]
        line(d, pts, T.c(T.main))
        poly(d, [(c, c - 6), (c + 3, c - 4), (c + 3, c), (c, c + 2), (c - 3, c), (c - 3, c - 4)], fill=T.c(T.dark), outline=T.c(T.hi))
        px(d, *pts[(f * 2) % 17], T.c(WHITE))
    elif name == 'rotation':
        for k in range(2):
            a0 = ph * .5 + k * math.pi
            ring(d, c, c, 26, T.c(T.hi if k else T.main), .55, 20, a0, a0 + 2.4)
            hx_, hy_ = pt(c, c, 26, a0 + 2.4, .55)
            tx, ty = pt(c, c, 26, a0 + 2.2, .55)
            ang = math.atan2(hy_ - ty, hx_ - tx)
            poly(d, [(hx_, hy_), pt(hx_, hy_, 4, ang + 2.5), pt(hx_, hy_, 4, ang - 2.5)], fill=T.c(WHITE))
        mini_rift(d, c - 26, c, 6, 2, T)
        mini_rift(d, c + 26, c, 6, 2, T)
    elif name == 'seedtrap':
        if f < 4:
            h = 2 + f * 3
            line(d, [(c, c + 12), (c, c + 12 - h)], T.c(T.main))
            for s in (-1, 1):
                poly(d, [(c, c + 12 - h), (c + s * (3 + f), c + 6 - h), (c + s * 2, c + 12 - h)], fill=T.c(T.hi, 200))
        else:
            k = (f - 4) / 3
            for s in (-1, 1):
                ang = -math.pi / 2 + s * (1 - k) * 1.1
                poly(d, [(c, c + 12), pt(c, c + 12, 14, ang - .3 * s), pt(c, c + 12, 16, ang)], fill=T.c(T.hi, 220), outline=T.c(WHITE))
            if f == 7 or (f == 6 and tier >= 2):
                sparkle(d, c, c, T.c(WHITE), 4)
        ring(d, c, c + 12, 12, T.c(T.main, 140), .4)
    elif name == 'sweep':
        x = c - 30 + 60 * p
        for k in range(10):
            px(d, x - 4 - k * 3, c + (k % 3) - 1 + T.j(k), T.c(T.hi, 230 - k * 20))
        line(d, [(c - 30, c + 4), (x, c + 4)], T.c(T.main, 120), 3)
        mini_rift(d, x, c, 8, 3, T)
        for k in range(3):
            px(d, x - 6 - (f + k * 3) % 9, c + 8 + k, T.c(V.RIM, 180))
    elif name == 'infuse':
        # Infuse: a hexorb's tech module streams into a tear and stays for its whole life
        mini_rift(d, c + 26, c + 14, 9, 3, T)
        for k in range(5):
            t = ((f / 8) + k / 5) % 1
            x, y = c - 26 + 52 * t, c - 14 + 28 * t
            pts = [pt(x, y, 2.5, j * TAU / 6) for j in range(6)]
            line(d, pts + [pts[0]], V.HEX if k % 2 else T.c(T.hi))
        hx, hy = c - 26, c - 14
        pts = [pt(hx, hy, 6, j * TAU / 6) for j in range(6)]
        poly(d, pts, fill=T.c(T.dark), outline=T.c(V.HEX))
        px(d, hx, hy, T.c(WHITE))
        ring(d, c + 26, c + 14, 6 + f % 3, T.c(V.HEX, 160), .8)
    elif name == 'stitch':
        mini_rift(d, c - 26, c - 8, 9, 3, T)
        mini_rift(d, c + 26, c + 8, 9, 3, T)
        k = p
        pts = []
        for i in range(round(16 * k) + 1):
            t = i / 16
            pts.append((c - 26 + 52 * t, c - 8 + 16 * t + (3 if i % 2 else -3)))
        if len(pts) > 1:
            line(d, pts, T.c(T.hi), 1)
        nx_, ny_ = pts[-1]
        line(d, [(nx_, ny_), (nx_ + 5, ny_ - 4)], T.c(WHITE), 1)
        px(d, nx_ + 6, ny_ - 5, T.c(WHITE))
    elif name == 'maw':
        close = [0, .1, .25, .45, .7, .9, 1, 1][f]
        open_h = 20 * (1 - close) + 1
        for s in (-1, 1):
            base = c + s * open_h
            pts = [(c - 36, c), (c - 28, base), (c + 28, base), (c + 36, c)]
            poly(d, pts, fill=T.c((220, 236, 255) if tier >= 3 else T.hi, 140 + 60 * close), outline=T.c(WHITE))
            for k in range(8):
                x = c - 24 + k * 7
                poly(d, [(x - 2, base), (x + 2, base), (x, base - s * (5 + (k % 2) * 2))], fill=T.c(WHITE))
        if f >= 6:
            sparkle(d, c, c, T.c(WHITE), 6)
    # one moving mark per frame keeps the eight phases distinct; t0 stutters
    px(d, 8 + f * 3, TECH - 8, T.c(T.main, 120))
    if tier == 0 and f in (2, 5):
        im = im.transform(im.size, Image.AFFINE, (1, 0, -1, 0, 1, 0))
    if tier >= 2:
        im = glow(im, .8 + .2 * min(3, tier), 1.5)
    return im


# ------------------------------------------------------------------ twists --

TWIST_COLORS = [(127, 168, 255), (255, 90, 90), (255, 154, 60), (255, 240, 200), (170, 150, 220), (159, 232, 255),
                (255, 120, 70), (125, 220, 106), (201, 166, 255), (94, 224, 208), (110, 230, 180), (220, 200, 255),
                (125, 220, 106), (255, 214, 107), (255, 140, 230)]


def twist(kind, f):
    im = Image.new('RGBA', (TWIST, TWIST))
    d = ImageDraw.Draw(im)
    c = TWIST // 2
    col = TWIST_COLORS[kind] + (255,)
    life = [.5, .9, 1, 1, .9, .7, .45, .2][f]
    ph = f * TAU / 8
    pop = [.6, .9, 1.1, 1, 1, 1, 1, 1][f]
    ring(d, c, c, 6 + f * 3, a(col, 160 * life))
    if kind == 0:     # Lock
        d.rectangle((c - 5, c - 1, c + 5, c + 7), fill=a(col, 230 * life), outline=a(WHITE, 255 * life))
        sh = [4, 2, 0, 0, 0, 0, 0, 0][f]
        ring(d, c, c - 2 - sh, 4, a(WHITE, 255 * life), 1, 12, math.pi, TAU)
        px(d, c, c + 3, a(INK, 255 * life))
    elif kind == 1:   # Execute Window
        r = 12 * pop
        d.rectangle((c - r, c - r * .7, c + r, c + r * .7), outline=a(col, 255 * life))
        line(d, [(c - r - 3, c), (c + r + 3, c)], a(WHITE, 220 * life))
        line(d, [(c, c - r), (c, c + r)], a(WHITE, 220 * life))
    elif kind == 2:   # Crush
        y = c - 18 + min(1, f / 3) * 14
        d.rectangle((c - 9, y - 4, c + 9, y), fill=a(col, 230 * life), outline=a(WHITE, 255 * life))
        if f >= 3:
            for k in range(5):
                line(d, [(c - 10 + k * 5, c), (c - 12 + k * 6, c + 5)], a(WHITE, 200 * life))
    elif kind == 3:   # Sanctuary
        ring(d, c, c + 6, 13 * pop, a(col, 255 * life), 1, 20, math.pi, TAU, 2)
        line(d, [(c - 13 * pop, c + 6), (c + 13 * pop, c + 6)], a(col, 200 * life))
        px(d, c, c - 7 * pop + 6, a(WHITE, 255 * life))
    elif kind == 4:   # Prison
        for k in range(5):
            x = c - 10 + k * 5
            line(d, [(x, c - 12 * pop), (x, c + 12 * pop)], a(col, 240 * life), 2 if k in (0, 4) else 1)
        line(d, [(c - 11, c - 12 * pop), (c + 11, c - 12 * pop)], a(WHITE, 220 * life))
    elif kind == 5:   # Overcharge
        pts = [(c - 4, c - 14), (c + 3, c - 2), (c - 2, c - 1), (c + 5, c + 14)]
        line(d, pts, a(WHITE, 255 * life), 2)
        line(d, [(c + 5, c + 14), (c + 20, c + 8 - f)], a(col, 230 * life))
    elif kind == 6:   # Shrapnel
        for k in range(8):
            ang = k * TAU / 8 + .2
            q = pt(c, c, 4 + f * 3, ang)
            poly(d, [q, pt(q[0], q[1], 3, ang + 2), pt(q[0], q[1], 3, ang - 2)], fill=a(col, 240 * life))
    elif kind == 7:   # Siphon
        for k in range(10):
            t = (k / 10 + f / 16) % 1
            q = pt(c, c, 20 * (1 - t), k * 2.4 + t * 6)
            px(d, *q, a(col, 255 * life))
        dot(d, c, c + 2, 3, a(col, 230 * life))
        px(d, c, c - 1, a(WHITE, 255 * life))
    elif kind == 8:   # Leak
        mini = Tech(2, f, 3)
        mini_rift(d, c, c - 8, 8, 3, mini, 255 * life)
        for k in range(3):
            y = c + ((f * 3 + k * 6) % 18)
            dot(d, c - 3 + k * 3, y, 1, a(col, 240 * life))
    elif kind == 9:   # Vortex
        for k in range(3):
            ring(d, c, c, 6 + k * 5, a(col, 230 * life), .8, 16, ph * (3 - k) + k, ph * (3 - k) + k + 4)
    elif kind == 10:  # Escort
        poly(d, [(c, c - 10), (c + 7, c - 6), (c + 6, c + 4), (c, c + 10), (c - 6, c + 4), (c - 7, c - 6)], fill=a(col, 120 * life), outline=a(WHITE, 255 * life))
        line(d, [(c - 14 + f * 2, c + 14), (c - 4 + f * 2, c + 14)], a(col, 240 * life))
        poly(d, [(c - 2 + f * 2, c + 14), (c - 5 + f * 2, c + 11), (c - 5 + f * 2, c + 17)], fill=a(col, 240 * life))
    elif kind == 11:  # Fold-throw
        k = f / 7
        x = c - 16 + 32 * k
        y = c + 8 - math.sin(k * math.pi) * 18
        poly(d, [(x - 4, y), (x, y - 4), (x + 4, y), (x, y + 2)], fill=a(col, 240 * life), outline=a(WHITE, 255 * life))
        line(d, [(c - 16, c + 8), (x, y)], a(col, 100 * life))
    elif kind == 12:  # Haven
        for s in (-1, 1):
            poly(d, [(c, c + 8), (c + s * 9 * pop, c - 2), (c + s * 4, c - 8 * pop)], fill=a(col, 220 * life), outline=a(WHITE, 200 * life))
        px(d, c, c + 2 - f % 3, a(WHITE, 255 * life))
    elif kind == 13:  # Gift
        d.rectangle((c - 7 * pop, c - 4, c + 7 * pop, c + 8), fill=a(col, 230 * life), outline=a(WHITE, 255 * life))
        line(d, [(c, c - 4), (c, c + 8)], a((255, 120, 120), 255 * life))
        ring(d, c - 3, c - 6, 3, a(WHITE, 220 * life), 1, 10)
        ring(d, c + 3, c - 6, 3, a(WHITE, 220 * life), 1, 10)
        sparkle(d, *pt(c, c, 14, ph), a(WHITE, 220 * life), 1)
    else:             # Paradox
        pts = [pt(c, c, 11 * pop, k * TAU / 3 - TAU / 4) for k in range(3)]
        line(d, pts + [pts[0]], a(col, 255 * life), 2)
        pts2 = [pt(c, c, 6 * pop, k * TAU / 3 + TAU / 4 + ph * .2) for k in range(3)]
        line(d, pts2 + [pts2[0]], a(WHITE, 220 * life))
    px(d, 6 + f * 3, TWIST - 6, a(col, 100))
    return glow(im, 1.0, 1.6)


# ------------------------------------------------------- gauntlet effects --
# Offsets from her centre (the native follows her unit): the lead fist in guard (+13, 0), the lead fist at
# the end of a rift punch (+22, +1), the Hexcore on the chambered right gauntlet (-21, +11).
# Every move is timed like a hit: an anticipation frame, a one-frame white impact, then ease-out falloff
# with debris that falls under gravity.

HEXC = V.HEX
HOLE = 64
GFX = 96
SWIPE = 112
MOD = 64
TIER_LOOK = [0, 3, 5, 7, 8]
CORRUPT = (220, 70, 170, 255)
MODULE_COLORS = [(120, 200, 255), (130, 255, 170), (170, 210, 255), (255, 120, 200), (120, 255, 140),
                 (255, 230, 120), (150, 230, 255), (255, 170, 90)]
EASE = [0, .5, .75, .88, .95, .98, 1, 1]           # ease-out progress per frame after the impact
FADE = [1, 1, 1, .85, .65, .45, .28, .12]
LEAD = (13, 0)
PUNCH_TIP = (22, 1)
HEXCORE = (-21, 11)


def hexagon(d, x, y, r, col, width=1, rot=0.0):
    pts = [pt(x, y, r, rot + k * TAU / 6) for k in range(6)]
    line(d, pts + [pts[0]], col, width)
    return pts


def debris(d, seed, x, y, n, f, speed, col, start=1, size=1.5, up=1.0):
    """Shards thrown out at the impact frame, then falling (gravity) and fading."""
    if f < start:
        return
    t = f - start
    for k in range(n):
        ang = -math.pi / 2 + (hsh(seed, k) - .5) * math.pi * 1.6 * up + (math.pi if up < 0 else 0)
        v = speed * (.6 + .6 * hsh(seed, k, 1))
        sx = x + math.cos(ang) * v * t
        sy = y + math.sin(ang) * v * t + .55 * t * t
        alpha = 255 * max(0, 1 - t / 7)
        c = col[k % len(col)] if isinstance(col, list) else col
        if size <= 1:
            px(d, sx, sy, a(c, alpha))
        else:
            spin = t * .9 + k
            poly(d, [pt(sx, sy, size, spin), pt(sx, sy, size * .6, spin + 2.2), pt(sx, sy, size, spin + 4.1)],
                 fill=a(c, alpha), outline=a(WHITE, alpha * .8))


def flash(d, x, y, r, alpha=255):
    dot(d, x, y, r, a(WHITE, alpha))
    for k in range(4):
        ang = k * TAU / 4 + TAU / 8
        line(d, [pt(x, y, r + 1, ang), pt(x, y, r + 3, ang)], a(WHITE, alpha))


def hole(look, f):
    """Where a rift punch lands: frame 0 cracks, frame 1 the fist punches through (white), then the hole
    hangs open with ripples and glass falling out of it, and snaps shut."""
    im = Image.new('RGBA', (HOLE, HOLE))
    d = ImageDraw.Draw(im)
    c = HOLE // 2
    col = GOLD_WHITE if look == 8 else (236, 222, 176, 255) if look == 0 else (228, 236, 255, 255) if look == 7 else RIFT_PALE
    open_r = [0, 10, 9, 9, 8, 7, 4, 1][f]
    paths = V.cracks(400 + look, c, c, 5 + look // 2, 6 + 10 * EASE[max(0, f - 1)] if f else 5, TAU, .3, look >= 3)
    for pth in paths:
        line(d, pth, a(col, 230 * FADE[f]))
    if f == 1:
        flash(d, c, c, 6)
    elif open_r:
        wob = [0, 0, 1, -1, 1, 0, 0, 0][f]
        pts = [pt(c, c, open_r * (1 + .22 * hsh(look, k)) + (wob if k % 2 else -wob), k * TAU / 9) for k in range(9)]
        poly(d, pts, fill=VOID, outline=col)
        if open_r >= 6:
            poly(d, [pt(c, c, open_r * .55, k * TAU / 6 + f) for k in range(6)], fill=RIFT_DEEP if look != 8 else (90, 70, 30, 255))
            px(d, c + (f % 3) - 1, c - 1, FAMILY[(f + look) % 10])
    if f == 7:
        sparkle(d, c, c, a(col, 220), 2)
    debris(d, 400 + look, c, c, 4 + look // 2, f, 3.2, [col, RIFT_PALE] if look else [EMBER, col], start=2, size=1.2)
    px(d, c - 10 + f * 3, c + 14, a(col, 120))
    return glow(im, .9, 1.5)


def punch_fx(tier, f):
    """On her, at the punching fist: sparks gather (anticipation), white impact, a shock arc and a small
    tear opening ahead of the knuckles, then sparks falling away."""
    im = Image.new('RGBA', (GFX, GFX))
    d = ImageDraw.Draw(im)
    T = Tech(tier, f, 501)
    c = GFX // 2
    x, y = c + PUNCH_TIP[0], c + PUNCH_TIP[1]
    if f == 0:
        for k in range(5):
            ang = math.pi + (k - 2) * .45
            line(d, [pt(x, y, 12, ang), pt(x, y, 7, ang)], T.c(HEXC, 200))
    elif f == 1:
        flash(d, x, y, 4)
        for k in range(3):
            line(d, [(x - 14, y - 4 + k * 4), (x - 5, y - 4 + k * 4)], T.c(WHITE, 220))
    else:
        e = EASE[f - 1]
        r = 4 + 9 * e
        ring(d, x, y, r, T.c(HEXC, 240 * FADE[f]), .9, 16, -1.25, 1.25, 1 + (tier >= 2))
        if tier >= 2:
            ring(d, x, y, r * .7, T.c(T.hi, 180 * FADE[f]), .9, 12, -1.1, 1.1)
        mini_rift(d, x + 3 + 2 * e, y, 4 + 6 * e, 2 + 2 * e, T, 255 * FADE[f])
    debris(d, 501 + tier, x, y, 5 + tier, f, 2.6, [HEXC, T.hi, WHITE], start=2, size=1, up=.8)
    if tier == 4 and f >= 2:
        for k in range(10):
            px(d, *pt(x, y, 6 + 10 * EASE[f - 1], k * TAU / 10 + f * .2), a(FAMILY[k], 230 * FADE[f]))
    px(d, 10 + f * 4, GFX - 10, T.c(T.main, 120))
    return glow(im, .8 + .2 * min(3, tier), 1.5)


def swipe(tier, h, f):
    """The swipe, like pulling off a mask: claw marks drag along the line, the rip snaps open (white),
    its edges flutter, then it zips shut from both ends."""
    im = Image.new('RGBA', (SWIPE, SWIPE))
    d = ImageDraw.Draw(im)
    c = SWIPE // 2
    look = TIER_LOOK[tier]
    T = Tech(tier, f, 600 + h)
    ux, uy = H8[h]
    n = math.hypot(ux, uy)
    ux, uy = ux / n, uy / n
    nx, ny = -uy, ux
    half = 30
    if f <= 1:      # anticipation and the drag: three claw marks race along the line
        reach = half * (.35 if f == 0 else 1)
        for k in (-1, 0, 1):
            q0 = (c - ux * half + nx * k * 5, c - uy * half + ny * k * 5)
            q1 = (c - ux * half + ux * 2 * reach + nx * k * 5, c - uy * half + uy * 2 * reach + ny * k * 5)
            line(d, [q0, q1], T.c(WHITE if f else T.hi, 230))
        if f == 1:
            for k in range(4):
                t = .2 + k * .2
                px(d, c - ux * half + ux * 2 * half * t + nx * 8, c - uy * half + uy * 2 * half * t + ny * 8, T.c(HEXC))
    else:
        close = [0, 0, 0, 0, 0, 0, .35, .75][f]       # the zip closes from both ends
        span = half * (1 - close)
        p0 = (c - ux * span, c - uy * span)
        p1 = (c + ux * span, c + uy * span)
        width = [0, 0, 11, 10, 9, 9, 7, 4][f] + ([0, 0, 0, 1, -1, 1, 0, 0][f])
        rip(im, p0, p1, width, look, f, 77 + h + tier * 5, 'void')
        d = ImageDraw.Draw(im)
        if f == 2:      # the snap: white edges and a burst along the line
            for k in range(-2, 3):
                bx, by = c + ux * k * 12, c + uy * k * 12
                line(d, [(bx + nx * 9, by + ny * 9), (bx + nx * 13, by + ny * 13)], T.c(WHITE))
                line(d, [(bx - nx * 9, by - ny * 9), (bx - nx * 13, by - ny * 13)], T.c(WHITE))
        if 3 <= f <= 5:  # the far side pulls them through: drag arrows across the tear
            for k in (-2, 0, 2):
                bx, by = c + ux * k * 9, c + uy * k * 9
                s = 9 - (f - 3) * 2
                line(d, [(bx + nx * s, by + ny * s), (bx + nx * (s - 4), by + ny * (s - 4))], T.c(T.hi, 200))
        for k in range(3):
            t = (f / 8 + k / 3) % 1
            px(d, p0[0] + (p1[0] - p0[0]) * t, p0[1] + (p1[1] - p0[1]) * t, T.c(WHITE))
    debris(d, 600 + h, c, c, 6, f, 3.0, [T.hi, HEXC], start=3, size=1)
    px(d, 8 + f * 3, SWIPE - 8, T.c(T.main, 120))
    return glow(im, .9 + .2 * min(3, tier), 1.5)


def shatter(tier, f):
    """At every open rift: the orb glows (anticipation), crush flash, orb glass bursts out under gravity and
    the rip opens to twice its size with an ease-out ring."""
    im = Image.new('RGBA', (BREACH, BREACH))
    d = ImageDraw.Draw(im)
    c = BREACH // 2
    T = Tech(min(tier, 4), f, 900 + tier)
    n = [8, 10, 13, 16, 18][tier]
    if f == 0:
        dot(d, c, c, 5, T.c(HEXC, 230))
        ring(d, c, c, 9, T.c(WHITE, 200))
        for k in range(6):
            ang = k * TAU / 6
            line(d, [pt(c, c, 16, ang), pt(c, c, 11, ang)], T.c(T.hi, 200))
    elif f == 1:
        flash(d, c, c, 9)
        for pth in V.cracks(700 + tier, c, c, n, 26, TAU, 0, True):
            line(d, pth, T.c(T.hi), 1 + (tier >= 3))
    else:
        e = EASE[f - 1]
        rr = 14 + 40 * e
        jag = [pt(c, c, rr * (1 + .1 * math.sin(k * 2.3 + f)), k * TAU / 20, .55) for k in range(20)]
        line(d, jag + [jag[0]], T.c(T.hi, 230 * FADE[f]), 1 + (tier >= 2))
        if tier >= 2:
            inner = [pt(c, c, rr * .8, k * TAU / 14 + f * .1, .55) for k in range(14)]
            line(d, inner + [inner[0]], T.c(T.main, 150 * FADE[f]))
    cols = [FAMILY[(k + tier) % 10] for k in range(n)] if tier == 4 else [T.main, T.hi, HEXC]
    debris(d, 900 + tier, c, c, n, f, 6.5, cols, start=1, size=3 + tier * .4)
    if tier == 0:
        for k in range(5):
            px(d, *pt(c, c, 10 + f * 6, hsh(4, k, f) * TAU), a(EMBER, 220 * FADE[f]))
    px(d, c - 20 + f * 5, c + 50, T.c(T.main, 100))
    return glow(im, 1.0 + tier * .2, 1.6)


def gbreak_l(tier, f):
    """The left gauntlet breaks: a crack flash, then plates and orb glass burst off the lead fist and
    fall, the cyan fizzles out and it smokes."""
    im = Image.new('RGBA', (GFX, GFX))
    d = ImageDraw.Draw(im)
    T = Tech(tier, f, 1200 + tier)
    c = GFX // 2
    x, y = c + LEAD[0], c + LEAD[1]
    if f == 0:
        for pth in V.cracks(1300 + tier, x, y, 5, 8, TAU, 0, False):
            line(d, pth, a((255, 150, 90), 230))
    elif f == 1:
        flash(d, x, y, 7)
    plates = [(84, 90, 104, 255), V.BRONZE, (146, 154, 170, 255)]
    debris(d, 1200 + tier, x, y, 6 + tier, f, 4.2, plates, start=1, size=2.6)
    debris(d, 1250 + tier, x, y, 6, f, 5.5, [HEXC, RIFT_PALE] + [FAMILY[(k * 3) % 10] for k in range(3)], start=1, size=1)
    if f >= 3:      # smoke rising
        for k in range(3):
            sy = y - (f - 3) * 3 - k * 4
            px(d, x - 2 + k * 2 + (f + k) % 2, sy, a((140, 140, 150), 170 - k * 40))
    px(d, 10 + f * 4, GFX - 10, T.c(T.main, 120))
    return glow(im, 1.0, 1.5)


def gbreak_r(f):
    """The Hexcore corrupts and the right gauntlet blows: a red-violet hex burst off the chambered fist."""
    im = Image.new('RGBA', (GFX, GFX))
    d = ImageDraw.Draw(im)
    c = GFX // 2
    x, y = c + HEXCORE[0], c + HEXCORE[1]
    if f == 0:
        hexagon(d, x, y, 4, a(CORRUPT, 255), 2)
    elif f == 1:
        flash(d, x, y, 6)
    else:
        e = EASE[f - 1]
        for k in range(3):
            hexagon(d, x, y, 4 + 16 * e + k * 4, a(CORRUPT if k % 2 else HEXC, 230 * FADE[f]), 1 + (k == 0), f * .15 + k * .3)
    debris(d, 1400, x, y, 9, f, 4.5, [CORRUPT, HEXC, WHITE], start=1, size=2.2)
    px(d, 10 + f * 4, GFX - 10, a(CORRUPT, 120))
    return glow(im, 1.2, 1.6)


def gfix_l(f):
    """The left gauntlet comes back online: a cyan ring cinches round the lead fist, the five sockets light
    one by one, then a last glint."""
    im = Image.new('RGBA', (GFX, GFX))
    d = ImageDraw.Draw(im)
    c = GFX // 2
    x, y = c + LEAD[0], c + LEAD[1]
    r = 15 - 11 * EASE[f]
    ring(d, x, y, r, a(HEXC, 230), .8)
    for k in range(min(5, f)):
        px(d, x - 4 + k * 2, y - 9, FAMILY[k * 2])
    if f == 7:
        sparkle(d, x + 6, y - 6, WHITE, 2)
    px(d, *pt(x, y, r, f * TAU / 8, .8), WHITE)
    return glow(im, .9, 1.5)


def awaken(tier, f):
    """Hexcore Awakening (a 6 s loop): white-cyan flames lick off the Hexcore, embers rise, hex rings climb
    over her."""
    im = Image.new('RGBA', (GFX, GFX))
    d = ImageDraw.Draw(im)
    c = GFX // 2
    T = Tech(tier, f, 1500 + tier)
    x, y = c + HEXCORE[0], c + HEXCORE[1]
    hot = GOLD_WHITE if tier == 4 else (236, 250, 255, 255)
    for k in range(7):
        ang = -math.pi / 2 + (k - 3) * .32
        h = 6 + ((f + k * 3) % 5) * 2 + (k == 3) * 3
        tip = pt(x, y, h, ang + math.sin(f * 1.3 + k) * .18)
        line(d, [pt(x, y, 3, ang), tip], a(HEXC if k % 2 else hot, 220))
        if (f + k) % 4 == 0:
            px(d, tip[0], tip[1] - 2 - (f % 3), a(hot, 200))                 # an ember breaking off
    hexagon(d, x, y, 4, a(hot, 255), 1, f * .2)
    for k in range(2):
        t = ((f / 8) + k / 2) % 1
        hexagon(d, c, c + 16 - t * 44, 9 + t * 7, a(HEXC, 210 * (1 - t)), 1, .5)
    for k in range(4 + tier):
        ang = k * TAU / (4 + tier) + f * .3
        px(d, *pt(c, c, 20 + (k * 5 + f) % 10, ang), a(hot, 200))
    px(d, 10 + f * 4, GFX - 10, T.c(T.main, 120))
    return glow(im, 1.1, 1.6)


def corrupt(f):
    """Each overclock step: red-violet glitch blocks tear off the Hexcore."""
    im = Image.new('RGBA', (GFX, GFX))
    d = ImageDraw.Draw(im)
    c = GFX // 2
    x, y = c + HEXCORE[0], c + HEXCORE[1]
    for k in range(7):
        gx = x - 8 + int(hsh(31, f, k) * 16)
        gy = y - 10 + int(hsh(32, f, k) * 20)
        w = 2 + k % 3 + (f == 1) * 2
        d.rectangle((gx, gy, gx + w, gy), fill=a(CORRUPT if k % 2 else HEXC, 230 * FADE[f]))
    hexagon(d, x, y, 5 + 6 * EASE[f], a(CORRUPT, 220 * FADE[f]), 1, f * .3)
    if f == 1:
        flash(d, x, y, 3)
    return glow(im, 1.0, 1.5)


def overload(tier, f):
    """An orb is forced into an overload socket: the arcs gather, strike (white), then crawl and fade."""
    im = Image.new('RGBA', (GFX, GFX))
    d = ImageDraw.Draw(im)
    c = GFX // 2
    T = Tech(tier, f, 1700 + tier)
    x, y = c + LEAD[0], c + LEAD[1] + 4
    hot = (255, 160, 90, 255)
    if f == 1:
        flash(d, x, y, 4)
    for k in range(3 + tier):
        ang = k * TAU / (3 + tier) + f * .7
        r0 = 12 if f == 0 else 4
        pts = [pt(x, y, r0, ang)]
        for j in range(1, 4):
            pts.append(pt(x, y, r0 + j * (4 if f else -2), ang + (hsh(k, j, f) - .5) * .9))
        line(d, pts, a(hot if k % 2 else RIFT_PALE, 230 * FADE[f]))
    ring(d, x, y, 9 + 4 * EASE[f], a(hot, 160 * FADE[f]), .8)
    debris(d, 1700 + tier, x, y, 4, f, 2.5, [hot, WHITE], start=2, size=1)
    px(d, 10 + f * 4, GFX - 10, T.c(T.main, 120))
    return glow(im, 1.0, 1.6)


def module(m, tier, f):
    """A hexorb's tech module firing at a point, each with its own readable icon."""
    im = Image.new('RGBA', (MOD, MOD))
    d = ImageDraw.Draw(im)
    c = MOD // 2
    col = MODULE_COLORS[m] + (255,)
    T = Tech(tier, f, 1900 + m)
    life = [.5, .9, 1, 1, .9, .7, .45, .2][f]
    pop = [.6, .9, 1.1, 1, 1, 1, 1, 1][f]
    hexagon(d, c, c, 12 * pop, a(col, 150 * life), 1, 0)
    if m == 0:      # Plating: a hex shield plate
        poly(d, [pt(c, c, 8 * pop, k * TAU / 6 + TAU / 12) for k in range(6)], fill=a(col, 90 * life), outline=a(WHITE, 255 * life))
        line(d, [(c - 4, c), (c + 4, c)], a(WHITE, 200 * life))
    elif m == 1:    # Coil: a spring and speed chevrons
        pts = [(c - 8 + k * 2, c + (3 if k % 2 else -3)) for k in range(9)]
        line(d, pts, a(col, 255 * life))
        for k in range(2):
            x = c + 10 + k * 4 + f % 2
            line(d, [(x - 2, c - 3), (x, c), (x - 2, c + 3)], a(WHITE, 220 * life))
    elif m == 2:    # Stasis: a slowed clock ring
        ring(d, c, c, 8, a(col, 255 * life))
        line(d, [(c, c), pt(c, c, 6, -math.pi / 2 + f * .2)], a(WHITE, 255 * life))
        line(d, [(c, c), pt(c, c, 4, f * .05)], a(WHITE, 200 * life))
    elif m == 3:    # Magnet: a horseshoe with inward arrows
        ring(d, c, c - 2, 6, a(col, 255 * life), 1, 12, 0, math.pi, 2)
        line(d, [(c - 6, c - 2), (c - 6, c - 7)], a(WHITE, 255 * life), 2)
        line(d, [(c + 6, c - 2), (c + 6, c - 7)], a(WHITE, 255 * life), 2)
        for k in range(4):
            ang = k * TAU / 4 + .78
            line(d, [pt(c, c, 18 - f, ang), pt(c, c, 14 - f, ang)], a(col, 220 * life))
    elif m == 4:    # Mender: a cross with rising sparks
        d.rectangle((c - 2, c - 7, c + 2, c + 7), fill=a(col, 230 * life))
        d.rectangle((c - 7, c - 2, c + 7, c + 2), fill=a(col, 230 * life))
        px(d, c, c, a(WHITE, 255 * life))
        for k in range(3):
            px(d, c - 8 + k * 8, c - 4 - (f + k * 2) % 8, a(WHITE, 200 * life))
    elif m == 5:    # Scanner: a reticle and a scan line
        ring(d, c, c, 7, a(col, 255 * life))
        for k in range(4):
            ang = k * TAU / 4
            line(d, [pt(c, c, 5, ang), pt(c, c, 10, ang)], a(col, 255 * life))
        y = c - 10 + (f * 3) % 20
        line(d, [(c - 10, y), (c + 10, y)], a(WHITE, 160 * life))
    elif m == 6:    # Spark Arc: lightning jumping to two points
        for tgt in ((c + 18, c - 10), (c + 16, c + 12)):
            pts = [(c, c)]
            for j in range(1, 4):
                t = j / 4
                pts.append((c + (tgt[0] - c) * t + (hsh(m, j, f) - .5) * 6, c + (tgt[1] - c) * t + (hsh(m, j, f, 1) - .5) * 6))
            pts.append(tgt)
            line(d, pts, a(WHITE if f % 2 else col, 255 * life))
        dot(d, c, c, 2, a(col, 255 * life))
    else:           # Anchor Bolt: a spike driven into the ground
        y = c - 14 + min(1, f / 2) * 12
        poly(d, [(c - 2, y - 8), (c + 2, y - 8), (c + 2, y), (c, y + 4), (c - 2, y)], fill=a(col, 240 * life), outline=a(WHITE, 255 * life))
        if f >= 2:
            ring(d, c, c + 4, 4 + f, a(col, 200 * life), .4)
    px(d, 6 + f * 3, MOD - 6, T.c(T.main, 100))
    return glow(im, .9, 1.5)


def bolt(f):
    """The normal attack: a cyan knuckle shock thrown off the jab, pointing +x."""
    im = Image.new('RGBA', (BOLT, BOLT))
    d = ImageDraw.Draw(im)
    c = BOLT // 2
    for k in range(3):
        line(d, [(c - 9 + k, c - 3 + k * 3), (c + 1, c - 3 + k * 3)], a(HEXC, 120 + k * 30))
    ring(d, c + 2, c, 4 + f % 2, a(WHITE, 230), 1, 10, -1.2, 1.2)
    for k in range(4):
        px(d, c + 4, c - 3 + k * 2, a(RIFT_PALE if k % 2 else HEXC, 230))
    px(d, c - 12 + f, c + (1 if f % 2 else -1), a(RIFT, 160))
    return glow(im, .7, 1.5)


# ----------------------------------------------------------------- generate --

def generate(ctx):
    """Build every vesper_* VFX atlas. ctx: atlas, bounded, MOD."""
    vfx = ctx.MOD / 'vfx'
    S = lambda ticks: ticks / 60 / 8          # seconds per frame for an 8-frame loop of `ticks`

    def split(name, entries, w, h, key):
        """One sheet per look or tier: a match only ever decodes its own rank's art (big shared sheets made
        the game stall when the first rift of a kind opened)."""
        groups = {}
        for tag, seq in entries.items():
            groups.setdefault(key(tag), {})[tag] = seq
        for k, group in sorted(groups.items()):
            ctx.bounded(f'{name}_{k}', group, w, h)
    look_of = lambda tag: re.search(r'_(l\d)', tag).group(1)
    tier_of = lambda tag: re.search(r'_(t\d)', tag).group(1)
    tier_or_common = lambda tag: (re.search(r'_(t\d)', tag) or re.search(r'()$', tag)).group(1) or 'common'
    # technique flourishes: per tier, then halved by group, so the first cast decodes about half a tier
    groups = {t['id']: t['group'] for t in V.DATA['techniques']}
    groups.update(maw='offense', kidnapwarn='engage', kidnapmiss='engage')
    half = lambda tag: tier_of(tag) + ('a' if groups[re.sub(r'^tech_|_t\d$', '', tag)] in ('engage', 'defense') else 'b')
    rifts = {}
    for kind in SHAPES:
        for look in range(9):
            rifts[f'rift_{kind}_l{look}'] = [(rift_shape(kind, look, f), S(48)) for f in range(8)]
    for kind in LINES:
        for look in range(9):
            for ang in range(4):
                rifts[f'rift_{kind}_l{look}_a{ang}'] = [(rift_shape(kind, look, f, ang), S(48)) for f in range(8)]
    # per look, then common kinds (a) apart from the rarer ones (b): the first rift decodes about 4 MiB, not 8
    common = ('tear', 'twin', 'anchor', 'wander', 'slash')
    split('vesper_rifts', rifts, CANVAS, CANVAS, lambda tag: look_of(tag) + ('a' if tag.split('_')[1] in common else 'b'))
    cores = {f'core_f{fam}_t{t}': [(core(fam, t, f), S(48)) for f in range(8)] for fam in range(len(FAMILY)) for t in range(4)}
    split('vesper_cores', cores, CORE, CORE, tier_of)
    # the first 100 glyphs keep their sheet; each further 100 get their own (a first rift decodes one sheet)
    n = len(FAM_OF)
    for k in range(0, n, 100):
        glyphs = {f'glyph{dim:02}': [(glyph(dim, f), S(48)) for f in range(8)] for dim in range(k, min(n, k + 100))}
        ctx.bounded('vesper_glyphs' + ('' if k == 0 else f'_{k // 100}'), glyphs, GLYPH, GLYPH)
    links = {f'link_l{look}_a{h}': [(link(look, h, f), S(48)) for f in range(8)] for look in range(9) for h in range(8)}
    split('vesper_links', links, LINK, LINK, look_of)
    packets = {f'packet_l{look}_a{h}': [(packet(look, h, f), S(4)) for f in range(8)] for look in range(9) for h in range(8)}
    split('vesper_packets', packets, PACKET, PACKET, look_of)
    events = {}
    for look in range(9):
        events[f'open_l{look}'] = [(open_(look, f), S(30)) for f in range(8)]
        events[f'close_l{look}'] = [(close_(look, f), S(24)) for f in range(8)]
        events[f'break_l{look}'] = [(break_(look, f), S(24)) for f in range(8)]
        events[f'seedhint_l{look}'] = [(seedhint(look, f), S(48)) for f in range(8)]
        events[f'impact_l{look}'] = [(impact(look, f), S(12)) for f in range(8)]
        events[f'exit_l{look}'] = [(exit_(look, f), S(12)) for f in range(8)]
    split('vesper_events', events, BREAK, BREAK, look_of)
    techs = {f'tech_{name}_t{t}': [(tech(name, t, f), S(30 if name in SLOW_TECHS else 24)) for f in range(8)]
             for name in TECHS for t in range(5)}
    techs.update({f'kidnapwarn_t{t}': [(kidnap_warn(t, f), S(30)) for f in range(8)] for t in range(5)})
    techs.update({f'kidnapmiss_t{t}': [(kidnap_miss(t, f), S(18)) for f in range(8)] for t in range(5)})
    split('vesper_techs', techs, TECH, TECH, half)
    # the normal-attack projectile on its own tiny sheet: it is decoded at her first jab
    ctx.bounded('vesper_bolt', {'bolt': [(bolt(f), .05) for f in range(8)]}, BREACH, BREACH)
    ctx.bounded('vesper_twists', {f'twist{k}': [(twist(k, f), S(18)) for f in range(8)] for k in range(15)}, BREACH, BREACH)
    split('vesper_shatter', {f'shatter_t{t}': [(shatter(t, f), S(30)) for f in range(8)] for t in range(5)}, BREACH, BREACH, tier_of)
    holes = {f'hole_l{look}': [(hole(look, f), S(18)) for f in range(8)] for look in range(9)}
    split('vesper_holes', holes, HOLE, HOLE, look_of)
    swipes = {f'swipe_t{t}_a{h}': [(swipe(t, h, f), S(24)) for f in range(8)] for t in range(5) for h in range(4)}
    split('vesper_swipes', swipes, SWIPE, SWIPE, tier_of)
    gauntlets = {f'punch_t{t}': [(punch_fx(t, f), S(12)) for f in range(8)] for t in range(5)}
    gauntlets.update({f'gbreak_l_t{t}': [(gbreak_l(t, f), S(36)) for f in range(8)] for t in range(4)})
    gauntlets['gbreak_r'] = [(gbreak_r(f), S(36)) for f in range(8)]
    gauntlets['gfix_l'] = [(gfix_l(f), S(18)) for f in range(8)]
    gauntlets.update({f'awaken_t{t}': [(awaken(t, f), S(48)) for f in range(8)] for t in range(5)})
    gauntlets['corrupt'] = [(corrupt(f), S(18)) for f in range(8)]
    gauntlets.update({f'overload_t{t}': [(overload(t, f), S(18)) for f in range(8)] for t in range(4)})
    split('vesper_gauntlets', gauntlets, GFX, GFX, tier_or_common)
    modules = {f'mod{m}_t{t}': [(module(m, t, f), S(18)) for f in range(8)] for m in range(8) for t in range(4)}
    split('vesper_modules', modules, MOD, MOD, tier_of)
    return vfx
