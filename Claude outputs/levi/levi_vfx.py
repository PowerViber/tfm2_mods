"""Round 77: Levi's VFX sheet 'levi' (mods/tfm2_levi/vfx/levi#sheet.png + #anim.fanim).

  cable_<d>_<b>     a cable: d = 0..15 direction (d x 11.25 degrees, a line looks the same both ways), b = 1..6 length
                    (16 px per step, 950 world units a px); drawn centred on the cable's midpoint
  hook              the spark where a cable bites
  whiff             a cable that found nothing (a short frayed line burst)
  trail<t>_<d>      what streams behind him while he flies, d = 0..15 the way he is flying (d x 22.5 degrees);
                    t0 low speed (short scarf, a few gas puffs), t1 fast (long scarf, cyan gas jet),
                    t2 Stormcutter (a jagged storm streak), t3 Comet (a ball of light over him with a long tail),
                    t4 Apex (an aurora ribbon and a shock cone ahead of him)
  after_r / after_l a fading cyan afterimage of his flying pose (at full chain)
  pips<n>           chain pips over his HP bar, n = 0..8 lit (n = 0: none shown)
  gas<n>            his gas bar, n = 0..10
  slice, slice_big  a blade cut (passive), the Rampage cut
  crash             the slam into a wall
  dash_gas          the off-cable gas burst
  starburst         a cable bite at Comet speed
  apex_ring         a cable bite at Apex speed
  rampage           the Rampage aura (behind him)
  rank<r>           rank badges 0-6 (Grounded, Tethered, Swinger, Glider, Skyrunner, Stormcutter, Comet)
  apex<p>           Top 10 ("Apex") badges #1-#10

Run from the repo root: python3 "Claude outputs/levi/levi_vfx.py" [--preview DIR]
"""
import json
import math
import os
import random
import sys

from PIL import Image

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), '..', '..'))
OUT_DIR = os.path.join(ROOT, 'mods', 'tfm2_levi', 'vfx')
BODY = os.path.join(ROOT, 'mods', 'tfm2_levi', 'champions', 'tfm2_levi_levi#sheet.png')
BODY_ANIM = os.path.join(ROOT, 'mods', 'tfm2_levi', 'champions', 'tfm2_levi_levi#anim.fanim')
OUT = (18, 14, 28, 255)
WHITE = (255, 255, 255, 255)


def rgba(h, a=255):
    h = h.lstrip('#')
    return (int(h[0:2], 16), int(h[2:4], 16), int(h[4:6], 16), a)


def hsv(h, s=0.8, v=1.0, a=255):
    h = (h % 1.0) * 6
    i = int(h)
    f = h - i
    p, q, t = v * (1 - s), v * (1 - s * f), v * (1 - s * (1 - f))
    r, g, b = [(v, t, p), (q, v, p), (p, v, t), (p, q, v), (t, p, v), (v, p, q)][i % 6]
    return (int(r * 255), int(g * 255), int(b * 255), a)


class Cv:
    def __init__(self, w, h):
        self.im = Image.new('RGBA', (w, h), (0, 0, 0, 0))
        self.px = self.im.load()
        self.w, self.h = w, h

    def put(self, x, y, c):
        x, y = int(round(x)), int(round(y))
        if 0 <= x < self.w and 0 <= y < self.h and c[3] > 0:
            b = self.px[x, y]
            if c[3] >= 255 or b[3] == 0:
                self.px[x, y] = c
            else:
                a = c[3] / 255
                self.px[x, y] = tuple(int(b[i] * (1 - a) + c[i] * a) for i in range(3)) + (max(b[3], c[3]),)

    def get(self, x, y):
        return self.px[x, y] if 0 <= x < self.w and 0 <= y < self.h else (0, 0, 0, 0)

    def line(self, x0, y0, x1, y1, c):
        n = int(max(abs(x1 - x0), abs(y1 - y0))) + 1
        for i in range(n + 1):
            t = i / n
            self.put(x0 + (x1 - x0) * t, y0 + (y1 - y0) * t, c)

    def disc(self, cx, cy, r, c):
        for y in range(int(cy - r - 1), int(cy + r + 2)):
            for x in range(int(cx - r - 1), int(cx + r + 2)):
                if (x - cx) ** 2 + (y - cy) ** 2 <= r * r + 0.3:
                    self.put(x, y, c)

    def ring(self, cx, cy, r, c, width=1.0):
        for y in range(int(cy - r - 2), int(cy + r + 3)):
            for x in range(int(cx - r - 2), int(cx + r + 3)):
                if abs(math.hypot(x - cx, y - cy) - r) <= width / 2 + 0.2:
                    self.put(x, y, c)

    def poly(self, pts, c):
        ys = [p[1] for p in pts]
        for y in range(int(min(ys)), int(max(ys)) + 1):
            xs = []
            for i in range(len(pts)):
                (x0, y0), (x1, y1) = pts[i], pts[(i + 1) % len(pts)]
                if (y0 <= y + 0.5 < y1) or (y1 <= y + 0.5 < y0):
                    xs.append(x0 + (y + 0.5 - y0) * (x1 - x0) / (y1 - y0))
            xs.sort()
            for a, b in zip(xs[::2], xs[1::2]):
                for x in range(int(math.ceil(a - 0.5)), int(math.floor(b - 0.5)) + 1):
                    self.put(x, y, c)

    def outline(self, c=OUT, thresh=150):
        add = []
        for y in range(self.h):
            for x in range(self.w):
                if self.px[x, y][3] == 0 and any(self.get(x + dx, y + dy)[3] > thresh for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1))):
                    add.append((x, y))
        for x, y in add:
            self.px[x, y] = c


def star4(cv, x, y, c, big=False):
    cv.put(x, y, WHITE)
    for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
        cv.put(x + dx, y + dy, c)
    if big:
        for dx, dy in ((2, 0), (-2, 0), (0, 2), (0, -2)):
            cv.put(x + dx, y + dy, c[:3] + (140,))


SILVER, SILVER_GLOW = rgba('#f2f6fa'), rgba('#9fc6da', 130)
TEAL, TEAL_D, TEAL_L = rgba('#2c6170'), rgba('#1e4552'), rgba('#3f8696')
CYAN, CYAN_L = rgba('#6fdcff'), rgba('#d6f7ff')


# ------------------------------------------------------------------ cables

def cable(d, b):
    ln = 16 * b
    side = ln + 6
    cv = Cv(side, side)
    a = math.radians(d * 11.25)
    cx = cy = side / 2
    dx, dy = math.cos(a) * ln / 2, math.sin(a) * ln / 2
    nx, ny = -math.sin(a), math.cos(a)
    cv.line(cx - dx + nx, cy - dy + ny, cx + dx + nx, cy + dy + ny, SILVER_GLOW)
    cv.line(cx - dx, cy - dy, cx + dx, cy + dy, SILVER)
    return cv.im


def hook(f):
    cv = Cv(16, 16)
    r = [1, 3, 5][f]
    for k in range(6):
        a = k * math.pi / 3 + f * 0.3
        cv.line(8 + math.cos(a) * (r - 1), 8 + math.sin(a) * (r - 1), 8 + math.cos(a) * r * 1.4, 8 + math.sin(a) * r * 1.4, SILVER if k % 2 else CYAN_L)
    cv.disc(8, 8, 1.2, WHITE)
    return cv.im


def whiff(f):
    cv = Cv(24, 24)
    rnd = random.Random(31 + f)
    for k in range(5):
        a = rnd.uniform(0, 2 * math.pi)
        r0, r1 = 2 + f * 2, 5 + f * 3
        cv.line(12 + math.cos(a) * r0, 12 + math.sin(a) * r0, 12 + math.cos(a) * r1, 12 + math.sin(a) * r1, rgba('#c9d3dc', 230 - f * 60))
    return cv.im


# ------------------------------------------------------------------ trails (96 x 96, he is at the centre, flying along d)

def back_axis(d):
    a = math.radians(d * 22.5)
    return (-math.cos(a), -math.sin(a)), (math.sin(a), -math.cos(a))   # behind him, and the side normal


def ribbon(cv, d, length, f, width=2.0, color=None, hue=None, wave=2.5):
    (bx, by), (nx, ny) = back_axis(d)
    for i in range(int(length)):
        t = i / length
        w = math.sin(t * 7 + f * 1.7) * wave * t
        x, y = 48 + bx * (4 + i) + nx * w, 48 + by * (4 + i) + ny * w + 2
        half = max(0.6, width * (1 - t * 0.7))
        c = hsv(hue + t * 0.6 + f * 0.08, 0.55, 1.0, int(255 * (1 - t * 0.85))) if hue is not None else color[:3] + (int(255 * (1 - t * 0.6)),)
        for s in range(-int(half), int(half) + 1):
            cv.put(x + nx * s, y + ny * s, c)


def trail(t, d, f):
    cv = Cv(96, 96)
    (bx, by), (nx, ny) = back_axis(d)
    if t in (0, 1, 2, 4):
        ribbon(cv, d, 16 if t == 0 else 30, f, width=1.6 if t == 0 else 2.2, color=TEAL if t != 4 else None, hue=0.45 if t == 4 else None)
    if t == 0:
        for k in range(3):
            s = 8 + k * 6 + f * 2
            cv.disc(48 + bx * s + nx * (k - 1) * 2, 48 + by * s + ny * (k - 1) * 2 + 3, 1.3, CYAN_L[:3] + (190 - k * 50,))
    if t >= 1 and t != 3:
        # the gas jet: a cyan stream with a white core
        for i in range(26 if t == 1 else 34):
            s = 6 + i
            a = int(255 * (1 - i / 34))
            cv.put(48 + bx * s, 48 + by * s + 3, CYAN_L[:3] + (a,))
            cv.put(48 + bx * s + nx, 48 + by * s + ny + 3, CYAN[:3] + (a // 2,))
            cv.put(48 + bx * s - nx, 48 + by * s - ny + 3, CYAN[:3] + (a // 2,))
    if t == 2:
        # Stormcutter: a jagged storm streak with sparks
        rnd = random.Random(500 + d * 7 + f)
        px_, py_ = 48 + bx * 6, 48 + by * 6
        for k in range(8):
            s = 6 + (k + 1) * 5
            j = rnd.choice((-4, -3, 3, 4))
            x, y = 48 + bx * s + nx * j, 48 + by * s + ny * j
            cv.line(px_, py_, x, y, rgba('#bff4ff'))
            cv.put(x, y, WHITE)
            px_, py_ = x, y
        for k in range(4):
            s = rnd.randint(8, 36)
            star4(cv, 48 + bx * s + nx * rnd.randint(-8, 8), 48 + by * s + ny * rnd.randint(-8, 8), rgba('#8fe8ff'))
    if t == 3:
        # Comet: a ball of light over him and a long tapering tail
        for i in range(46, 0, -1):
            s = 6 + i
            r = 9 * (1 - i / 50)
            a = int(200 * (1 - i / 46))
            cv.disc(48 + bx * s, 48 + by * s, max(0.6, r), (150, 230, 255, a))
        cv.disc(48, 48, 13, (120, 220, 255, 90))
        cv.disc(48, 48, 11, (190, 240, 255, 200))
        cv.disc(48, 48, 9 + (f % 2) * 0.5, (235, 252, 255, 245))
        cv.disc(48, 48, 6, WHITE)
        for k in range(5):
            a = k * 1.256 + f * 0.6
            star4(cv, 48 + math.cos(a) * 15, 48 + math.sin(a) * 15, rgba('#c8f3ff'))
    if t == 4:
        # Apex: the shock cone ahead of him
        fx_, fy_ = -bx, -by
        for side in (-1, 1):
            for i in range(14):
                x = 48 + fx_ * (16 - i) + nx * side * i * 0.8
                y = 48 + fy_ * (16 - i) + ny * side * i * 0.8
                cv.put(x, y, (255, 255, 255, 230 - i * 12))
        for k in range(3):
            star4(cv, 48 + fx_ * (18 + f * 2) + nx * (k - 1) * 6, 48 + fy_ * (18 + f * 2) + ny * (k - 1) * 6, hsv(k / 3 + f * 0.1, 0.4))
    return cv.im


def afterimage(face_left, f):
    body = Image.open(BODY).convert('RGBA')
    with open(BODY_ANIM, encoding='utf-8') as fh:
        d = json.load(fh)['anims']['run']['frames'][2]['data']
    im = body.crop((d['x'], d['y'], d['x'] + d['w'], d['y'] + d['h']))
    if face_left:
        im = im.transpose(Image.FLIP_LEFT_RIGHT)
    px = im.load()
    a_k = [0.6, 0.4, 0.22][f]
    for y in range(im.height):
        for x in range(im.width):
            r, g, b, a = px[x, y]
            if a:
                px[x, y] = (int(r * 0.3 + 120), int(g * 0.3 + 210), int(b * 0.3 + 240), int(a * a_k))
    return im


# ------------------------------------------------------------------ HUD (48 x 96, centred on him like every buff)

def pips(n):
    cv = Cv(48, 96)
    for k in range(8):
        x = 24 - 11 + k * 3
        c = rgba('#d8f2ff') if k < n else rgba('#2b3b48', 220)
        cv.put(x, 9, c)
        cv.put(x + 1, 9, c)
        cv.put(x, 10, c if k < n else rgba('#1d2832', 220))
        cv.put(x + 1, 10, c if k < n else rgba('#1d2832', 220))
    if n == 0:
        return Cv(48, 96).im
    return cv.im


def gasbar(n):
    cv = Cv(48, 96)
    for x in range(14, 34):
        cv.put(x, 12, OUT)
        cv.put(x, 15, OUT)
    for y in range(12, 16):
        cv.put(13, y, OUT)
        cv.put(34, y, OUT)
    fill = 14 + n * 2
    for x in range(14, 34):
        for y in (13, 14):
            cv.put(x, y, (rgba('#7fe3ff') if y == 13 else rgba('#3fb6e0')) if x < fill else rgba('#1b2630'))
    return cv.im


# ------------------------------------------------------------------ hits and bursts

def slice_fx(f, big=False):
    s = 48 if big else 32
    cv = Cv(s, s)
    c = s / 2
    r = (s / 2 - 3) * (0.6 + f * 0.13)
    a0 = -2.3 + f * 0.5
    for i in range(30):
        a = a0 + i * 0.07
        w = 1 if i in (0, 29) else 2
        col = (255, 255, 255, 255 - f * 50) if not big else ((200, 245, 255, 255 - f * 45) if i % 2 else (255, 255, 255, 255 - f * 45))
        for k in range(w):
            cv.put(c + math.cos(a) * (r - k), c + math.sin(a) * (r - k), col)
    return cv.im


def crash(f):
    cv = Cv(32, 32)
    for k in range(8):
        a = k * math.pi / 4 + 0.2
        r0, r1 = 3 + f * 2, 7 + f * 3
        cv.line(16 + math.cos(a) * r0, 16 + math.sin(a) * r0, 16 + math.cos(a) * r1, 16 + math.sin(a) * r1, rgba('#fff2b0', 255 - f * 55))
    cv.disc(16, 16, max(0.5, 4 - f), rgba('#ffffff', 230 - f * 50))
    return cv.im


def dash_gas(f):
    cv = Cv(32, 32)
    for k in range(7):
        a = k * 0.9 + f * 0.4
        r = 3 + f * 3
        cv.disc(16 + math.cos(a) * r, 16 + math.sin(a) * r, 2.2 - f * 0.4, CYAN_L[:3] + (230 - f * 55,))
    return cv.im


def starburst(f):
    cv = Cv(32, 32)
    for k in range(8):
        a = k * math.pi / 4 + (0.39 if k % 2 else 0)
        r = (6 if k % 2 else 10) + f * 2
        cv.line(16, 16, 16 + math.cos(a) * r, 16 + math.sin(a) * r, (230, 250, 255, 240 - f * 55))
    cv.disc(16, 16, 2.5, WHITE)
    return cv.im


def apex_ring(f):
    cv = Cv(64, 64)
    r = 6 + f * 6
    for i in range(int(2 * math.pi * r) + 6):
        a = i / r
        cv.put(32 + math.cos(a) * r, 32 + math.sin(a) * r, hsv(a / (2 * math.pi) + f * 0.1, 0.5, 1.0, 250 - f * 45))
    return cv.im


def rampage(f):
    cv = Cv(64, 64)
    for k in range(3):
        a0 = f * 0.8 + k * 2.09
        for i in range(14):
            a = a0 + i * 0.08
            cv.put(32 + math.cos(a) * 20, 34 + math.sin(a) * 10, (255, 120, 120, 220 - i * 12))
            cv.put(32 + math.cos(a) * 18, 34 + math.sin(a) * 9, (220, 245, 255, 200 - i * 12))
    return cv.im


# ------------------------------------------------------------------ badges (48 x 96, at his top right like Scribble's)

BX, BY = 40, 26


def badge(r, f=0):
    cv = Cv(48, 96)
    lay = Cv(48, 96)
    steel, steel_d, gold = rgba('#cfd8e2'), rgba('#7d8896'), rgba('#ffcf4a')
    if r == 0:      # Grounded: a coiled cable
        lay.ring(BX, BY, 5, steel_d, 2)
        lay.ring(BX, BY, 2.5, steel, 1.4)
    elif r == 1:    # Tethered: a single hook
        lay.line(BX, BY - 6, BX, BY + 2, steel)
        lay.line(BX + 1, BY - 6, BX + 1, BY + 2, steel_d)
        for i in range(7):
            a = math.pi * i / 6
            lay.put(BX - 2 + math.cos(a) * 3, BY + 2 + math.sin(a) * 3, steel)
        lay.put(BX - 5, BY + 1, steel)
    elif r == 2:    # Swinger: a carabiner
        for i in range(40):
            a = 2 * math.pi * i / 40
            lay.put(BX + math.cos(a) * 4, BY + math.sin(a) * 6, rgba('#e8b84a'))
        lay.line(BX + 3, BY - 3, BX + 4, BY + 2, steel)
    elif r == 3:    # Glider: a feather
        lay.poly([(BX - 5, BY + 5), (BX + 4, BY - 6), (BX + 5, BY - 5), (BX - 3, BY + 5)], rgba('#e9f1f8'))
        lay.line(BX - 5, BY + 6, BX + 4, BY - 5, steel_d)
    elif r == 4:    # Skyrunner: a pair of wings
        for side in (-1, 1):
            lay.poly([(BX, BY + 1), (BX + side * 7, BY - 5), (BX + side * 6, BY + 1), (BX + side * 3, BY + 4)], rgba('#bfe6ff'))
    elif r == 5:    # Stormcutter: a lightning bolt through a blade
        lay.line(BX - 6, BY + 6, BX + 6, BY - 6, steel)
        lay.poly([(BX + 1, BY - 7), (BX - 3, BY), (BX, BY), (BX - 2, BY + 7), (BX + 3, BY - 1), (BX, BY - 1)], rgba('#7fe3ff'))
    elif r == 6:    # Comet: a glowing comet (animated)
        for i in range(10):
            lay.disc(BX - 2 - i * 0.7, BY + 2 + i * 0.5, max(0.5, 2.5 - i * 0.25), (150, 230, 255, 200 - i * 18))
        lay.disc(BX + 2, BY - 2, 3.2, (220, 248, 255, 255))
        lay.disc(BX + 2, BY - 2, 1.6, WHITE)
    lay.outline()
    cv.im.alpha_composite(lay.im)
    if r == 6:
        star4(cv, BX + 2 + (f % 2) * 4 - 2, BY - 7 + (f // 2), rgba('#c8f3ff'))
    return cv.im


DIG = {
    '0': ['111', '101', '101', '101', '111'], '1': ['010', '110', '010', '010', '111'], '2': ['111', '001', '111', '100', '111'],
    '3': ['111', '001', '111', '001', '111'], '4': ['101', '101', '111', '001', '001'], '5': ['111', '100', '111', '001', '111'],
    '6': ['111', '100', '111', '101', '111'], '7': ['111', '001', '010', '010', '010'], '8': ['111', '101', '111', '101', '111'],
    '9': ['111', '101', '111', '001', '111'],
}


def apex_badge(p, f):
    """An arrowhead star with the position; #1 gold with a turning glint, the rest steel-blue."""
    cv = Cv(48, 96)
    lay = Cv(48, 96)
    body = rgba('#ffcf3f') if p == 1 else rgba('#2a3f57')
    rim = rgba('#fff2b8') if p == 1 else rgba('#9fd7ff')
    pts = []
    for k in range(10):
        a = -math.pi / 2 + k * math.pi / 5
        rr = 8 if k % 2 == 0 else 4
        pts.append((BX + math.cos(a) * rr, BY + math.sin(a) * rr + 1))
    lay.poly(pts, body)
    for k in range(0, 10, 2):
        lay.put(*pts[k], rim)
    lay.outline()
    cv.im.alpha_composite(lay.im)
    txt = str(p)
    w = len(txt) * 4 - 1
    for i, ch in enumerate(txt):
        for j, row in enumerate(DIG[ch]):
            for k, b in enumerate(row):
                if b == '1':
                    cv.put(BX - w // 2 + i * 4 + k, BY - 1 + j, rgba('#2a1606') if p == 1 else WHITE)
    # glint
    g = -9 + f * 3
    for y in range(BY - 8, BY + 9):
        for x in range(BX - 8, BX + 9):
            if (x - BX) + (y - BY) in (g, g + 1) and cv.get(x, y)[3]:
                cv.put(x, y, (255, 255, 255, 120))
    return cv.im


# ------------------------------------------------------------------ the sheet

def all_anims():
    A = {}
    for d in range(16):
        for b in range(1, 7):
            A[f'cable_{d}_{b}'] = ([cable(d, b)], 0.1)
    A['hook'] = ([hook(f) for f in range(3)], 0.05)
    A['whiff'] = ([whiff(f) for f in range(3)], 0.06)
    for t in range(5):
        for d in range(16):
            A[f'trail{t}_{d}'] = ([trail(t, d, f) for f in range(2)], 0.06)
    A['after_r'] = ([afterimage(False, f) for f in range(3)], 0.05)
    A['after_l'] = ([afterimage(True, f) for f in range(3)], 0.05)
    for n in range(9):
        A[f'pips{n}'] = ([pips(n)], 0.1)
    for n in range(11):
        A[f'gas{n}'] = ([gasbar(n)], 0.1)
    A['slice'] = ([slice_fx(f) for f in range(4)], 0.04)
    A['slice_big'] = ([slice_fx(f, True) for f in range(4)], 0.04)
    A['crash'] = ([crash(f) for f in range(4)], 0.06)
    A['dash_gas'] = ([dash_gas(f) for f in range(4)], 0.05)
    A['starburst'] = ([starburst(f) for f in range(4)], 0.05)
    A['apex_ring'] = ([apex_ring(f) for f in range(5)], 0.05)
    A['rampage'] = ([rampage(f) for f in range(6)], 0.06)
    for r in range(7):
        A[f'rank{r}'] = ([badge(r, f) for f in range(4)] if r == 6 else [badge(r)], 0.12)
    for p in range(1, 11):
        A[f'apex{p}'] = ([apex_badge(p, f) for f in range(8)], 0.1)
    return A


def build():
    A = all_anims()
    x, y, row_h = 0, 0, 0
    placed = []
    for name, (ims, dur) in A.items():
        rects = []
        for im in ims:
            if x + im.width > 2048:
                x, y, row_h = 0, y + row_h + 1, 0
            rects.append((x, y, im))
            x += im.width + 1
            row_h = max(row_h, im.height)
        placed.append((name, rects, dur))
    sheet = Image.new('RGBA', (2048, y + row_h), (0, 0, 0, 0))
    anims = {}
    for name, rects, dur in placed:
        for fx, fy, im in rects:
            sheet.paste(im, (fx, fy))
        anims[name] = {'frames': [{'duration': dur, 'data': {'x': fx, 'y': fy, 'w': im.width, 'h': im.height}} for fx, fy, im in rects]}
    return sheet, {'anims': anims}, A


def preview(A, folder):
    bg = (54, 74, 60, 255)
    body = Image.open(BODY).convert('RGBA')
    with open(BODY_ANIM, encoding='utf-8') as fh:
        fr = json.load(fh)['anims']['run']['frames'][1]['data']
    me = body.crop((fr['x'], fr['y'], fr['x'] + fr['w'], fr['y'] + fr['h']))
    out = Image.new('RGBA', (97 * 8, 97 * 6), bg)
    # five trail tiers flying right (d = 0) and up-right (d = 14), with two cables
    for t in range(5):
        for k, d in enumerate((0, 14)):
            cell = Image.new('RGBA', (96, 96), bg)
            if t != 3:
                cell.alpha_composite(A[f'trail{t}_{d}'][0][0])
                cell.alpha_composite(me, (48 - 24, 48 - 26))
            else:
                cell.alpha_composite(me, (48 - 24, 48 - 26))
                cell.alpha_composite(A[f'trail{t}_{d}'][0][0])
            out.alpha_composite(cell, ((t % 4) * 194 + k * 97, (t // 4) * 97))
    hud = Image.new('RGBA', (96, 96), bg)
    hud.alpha_composite(me, (48 - 24, 48 - 26))
    big = Image.new('RGBA', (48, 96), (0, 0, 0, 0))
    big.alpha_composite(A['pips5'][0][0])
    big.alpha_composite(A['gas7'][0][0])
    big.alpha_composite(A['rank5'][0][0])
    hud.alpha_composite(big, (24, 0))
    out.alpha_composite(hud, (194, 97))
    cab = Image.new('RGBA', (194, 97), bg)
    c1 = A['cable_13_5'][0][0]
    c2 = A['cable_2_4'][0][0]
    cab.alpha_composite(c1, (20, 0))
    cab.alpha_composite(c2, (100, 20))
    out.alpha_composite(cab, (388, 97))
    x = 0
    for r in range(7):
        out.alpha_composite(A[f'rank{r}'][0][0].crop((24, 10, 48, 42)), (x, 2 * 97))
        x += 26
    for p in (1, 2, 3, 10):
        out.alpha_composite(A[f'apex{p}'][0][0].crop((24, 10, 48, 42)), (x, 2 * 97))
        x += 26
    x = 0
    for n in ('slice', 'slice_big', 'crash', 'dash_gas', 'starburst', 'apex_ring', 'rampage', 'hook'):
        im = A[n][0][1]
        out.alpha_composite(im, (x, 3 * 97))
        x += im.width + 4
    out.alpha_composite(A['after_r'][0][0], (0, 4 * 97))
    out.resize((out.width * 3, out.height * 3), Image.NEAREST).save(os.path.join(folder, 'levi_vfx.png'))


if __name__ == '__main__':
    sheet, fanim, A = build()
    if '--preview' in sys.argv:
        preview(A, sys.argv[sys.argv.index('--preview') + 1])
    if '--dry' not in sys.argv:
        os.makedirs(OUT_DIR, exist_ok=True)
        sheet.save(os.path.join(OUT_DIR, 'levi#sheet.png'), optimize=True)
        with open(os.path.join(OUT_DIR, 'levi#anim.fanim'), 'w', encoding='utf-8') as fh:
            json.dump(fanim, fh, separators=(',', ':'))
    print('sheet', sheet.size, len(fanim['anims']), 'anims')
