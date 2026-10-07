"""Round 77: Levi's VFX sheet 'levi' (mods/tfm2_levi/vfx/levi#sheet.png + #anim.fanim).

  cable_<d>_<b>     a cable: d = 0..15 direction (d x 11.25 degrees, a line looks the same both ways), b = 1..6 length
                    (16 px per step, 950 world units a px); drawn centred on the cable's midpoint
  hook              the spark where a cable bites
  whiff             a cable that found nothing (a short frayed line burst)
  trail<t>_<d>      what streams behind him while he flies, d = 0..15 the way he is flying (d x 22.5 degrees);
                    t0 low speed (short scarf, a few gas puffs), t1 fast (long scarf, cyan gas jet),
                    t2 Stormcutter (a jagged storm streak), t3 Comet (a ball of light over him with a long tail),
                    t4 Apex (round 99: the wormhole tunnel he leaves, levi_wormhole.py)
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
OUT_DIR = os.path.join(ROOT, 'mods', 'tfm2_custom', 'vfx')
BODY = os.path.join(ROOT, 'mods', 'tfm2_custom', 'champions', 'tfm2_levi_levi#sheet.png')
BODY_ANIM = os.path.join(ROOT, 'mods', 'tfm2_custom', 'champions', 'tfm2_levi_levi#anim.fanim')
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

    def add(self, x, y, c):
        """Proper 'over' compositing, so overlapping glows build up."""
        x, y = int(round(x)), int(round(y))
        if 0 <= x < self.w and 0 <= y < self.h and c[3] > 0:
            b = self.px[x, y]
            ac, ab = c[3] / 255, b[3] / 255
            ao = ac + ab * (1 - ac)
            self.px[x, y] = tuple(int((c[i] * ac + b[i] * ab * (1 - ac)) / ao) for i in range(3)) + (int(ao * 255),)

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

CABLE_PH = 4   # cable phases: the game picks one every 2 ticks, so the wire seems to hum and the light runs along it


def cable(d, b, ph=0):
    """A taut steel cable 16*b px long at angle d*11.25: a bright core over a soft glow, a faint vibration, and two
    light pulses running along it (phase ph of CABLE_PH)."""
    ln = 16 * b
    side = ln + 8
    cv = Cv(side, side)
    a = math.radians(d * 11.25)
    cx = cy = side / 2
    ux, uy = math.cos(a), math.sin(a)
    nx, ny = -uy, ux
    n = int(ln) + 1
    pulses = [((ph + 0.5) / CABLE_PH) % 1, ((ph + 0.5) / CABLE_PH + 0.5) % 1]
    for i in range(n + 1):
        t = i / n
        vib = math.sin(t * math.pi * (2 + b * 0.5) + ph * math.pi / 2) * 0.55 * math.sin(t * math.pi)
        x = cx + ux * (t - 0.5) * ln + nx * vib
        y = cy + uy * (t - 0.5) * ln + ny * vib
        cv.put(x + nx, y + ny, SILVER_GLOW)
        cv.put(x - nx, y - ny, (159, 198, 218, 60))
        cv.put(x, y, SILVER)
        for pk in pulses:
            dd = abs(t - pk) * ln
            if dd < 3.5:
                k = 1 - dd / 3.5
                cv.put(x, y, (255, 255, 255, 255))
                cv.put(x + nx, y + ny, (111, 220, 255, int(220 * k)))
                cv.put(x - nx, y - ny, (111, 220, 255, int(160 * k)))
    return cv.im


def hook(f):
    """The bite (6 frames): the claws snap open into the wall, sparks fly, a ring of grit, then it settles."""
    cv = Cv(24, 24)
    c0 = 12
    open_ = (1.0, 3.0, 4.2, 4.0, 3.6, 3.4)[f]
    for k in range(3):
        a = k * math.tau / 3 + 0.4
        tip = (c0 + math.cos(a) * open_ * 1.3, c0 + math.sin(a) * open_ * 1.3)
        cv.line(c0, c0, tip[0], tip[1], SILVER)
        cv.put(tip[0] + math.cos(a + 1.2), tip[1] + math.sin(a + 1.2), SILVER)
    cv.disc(c0, c0, 1.3, rgba('#7c8693'))
    cv.put(c0, c0, WHITE)
    if 1 <= f <= 4:
        r = 3 + f * 2
        cv.ring(c0, c0, r, (214, 247, 255, 200 - f * 40), 1)
        rnd = random.Random(70 + f)
        for _ in range(6):
            a = rnd.uniform(0, math.tau)
            d = r - 1 + rnd.uniform(-1, 2)
            star4(cv, c0 + math.cos(a) * d, c0 + math.sin(a) * d, CYAN) if rnd.random() < 0.3 else cv.put(c0 + math.cos(a) * d, c0 + math.sin(a) * d, (255, 240, 200, 230))
    if f >= 2:   # grit
        rnd = random.Random(9)
        for _ in range(5):
            a = rnd.uniform(0, math.tau)
            d = 2 + f * 1.4 + rnd.uniform(0, 2)
            cv.put(c0 + math.cos(a) * d, c0 + math.sin(a) * d + f * 0.5, (150, 140, 120, 230 - f * 30))
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

TW = 80   # trail frames are 80 x 80, centred on him


def back_axis(d):
    a = math.radians(d * 22.5)
    return (-math.cos(a), -math.sin(a)), (math.sin(a), -math.cos(a))   # behind him, and the side normal


def ribbon(cv, d, length, f, width=2.0, color=None, hue=None, wave=2.5):
    (bx, by), (nx, ny) = back_axis(d)
    c0 = cv.w / 2
    for i in range(int(length)):
        t = i / length
        w = math.sin(t * 7 + f * 1.7) * wave * t
        x, y = c0 + bx * (4 + i) + nx * w, c0 + by * (4 + i) + ny * w + 2
        half = max(0.6, width * (1 - t * 0.7))
        c = hsv(hue + t * 0.6 + f * 0.08, 0.55, 1.0, int(255 * (1 - t * 0.85))) if hue is not None else color[:3] + (int(255 * (1 - t * 0.6)),)
        for s_ in range(-int(half), int(half) + 1):
            cv.put(x + nx * s_, y + ny * s_, c)


def glow(cv, x, y, r, col, a):
    """A soft round glow: alpha falls off to the edge."""
    for yy in range(int(y - r - 1), int(y + r + 2)):
        for xx in range(int(x - r - 1), int(x + r + 2)):
            d = math.hypot(xx - x, yy - y)
            if d <= r:
                cv.add(xx, yy, col[:3] + (int(a * (1 - d / r) ** 1.4),))


def bolt(cv, pts, core, halo, a=255):
    for (x0, y0), (x1, y1) in zip(pts, pts[1:]):
        n = int(max(abs(x1 - x0), abs(y1 - y0))) + 1
        for i in range(n + 1):
            t = i / n
            x, y = x0 + (x1 - x0) * t, y0 + (y1 - y0) * t
            for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                cv.put(x + dx, y + dy, halo[:3] + (int(a * 0.55),))
    for (x0, y0), (x1, y1) in zip(pts, pts[1:]):
        cv.line(x0, y0, x1, y1, core[:3] + (a,))


def trail(t, d, f):
    """Low tiers: a short streak redrawn every 2 ticks (2 frames)."""
    cv = Cv(TW, TW)
    (bx, by), (nx, ny) = back_axis(d)
    c0 = TW / 2
    ribbon(cv, d, 16 if t == 0 else 28, f, width=1.6 if t == 0 else 2.2, color=TEAL)
    if t == 0:
        for k in range(3):
            s_ = 8 + k * 6 + f * 2
            cv.disc(c0 + bx * s_ + nx * (k - 1) * 2, c0 + by * s_ + ny * (k - 1) * 2 + 3, 1.3, CYAN_L[:3] + (190 - k * 50,))
    else:
        for i in range(26):
            s_ = 6 + i
            a = int(255 * (1 - i / 30))
            cv.put(c0 + bx * s_, c0 + by * s_ + 3, CYAN_L[:3] + (a,))
            cv.put(c0 + bx * s_ + nx, c0 + by * s_ + ny + 3, CYAN[:3] + (a // 2,))
            cv.put(c0 + bx * s_ - nx, c0 + by * s_ - ny + 3, CYAN[:3] + (a // 2,))
    return cv.im


EMIT = 6   # frames in a high-tier trail burst; played every 2 ticks, the copies overlap into one long, living trail


def band(cv, d, s0, s1, r0, r1, col, a0, a1, wave=0.0, phase=0.0):
    """A soft glowing band behind him from s0 to s1 px, radius r0 -> r1, alpha a0 -> a1."""
    (bx, by), (nx, ny) = back_axis(d)
    c0 = cv.w / 2
    n = max(2, int(s1 - s0))
    for i in range(0, n, 2):
        t = i / n
        w = math.sin(t * 6 + phase) * wave * t
        glow(cv, c0 + bx * (s0 + i) + nx * w, c0 + by * (s0 + i) + ny * w, r0 + (r1 - r0) * t, col, int(a0 + (a1 - a0) * t))


def emit_storm(d, k):
    """Stormcutter: an ion stream with a jagged bolt torn off behind him that drifts back and dies, sparks flung sideways."""
    cv = Cv(TW, TW)
    (bx, by), (nx, ny) = back_axis(d)
    c0 = TW / 2
    life = 1 - k / EMIT
    rnd = random.Random(900 + d * 13 + k)
    if k < 3:   # the ion stream under the bolts
        band(cv, d, 2 + k * 3, 30 + k * 3, 6, 2.5, rgba('#2fb8ff'), int(90 * life), 0)
    drift = 4 + k * 3
    pts = [(c0 + bx * drift, c0 + by * drift)]
    for i in range(1, 8):
        s_ = drift + i * 4.5
        j = rnd.uniform(-5, 5) * (0.4 + i / 7)
        pts.append((c0 + bx * s_ + nx * j, c0 + by * s_ + ny * j))
    bolt(cv, pts, WHITE, rgba('#4fd2ff'), int(255 * life))
    if k < 4:   # forks
        for _ in range(2 if k < 2 else 1):
            i0 = rnd.randint(2, 5)
            fx_, fy_ = pts[i0]
            side = rnd.choice((-1, 1))
            bolt(cv, [(fx_, fy_), (fx_ + bx * 5 + nx * side * 6, fy_ + by * 5 + ny * side * 6), (fx_ + bx * 8 + nx * side * 11, fy_ + by * 8 + ny * side * 11)],
                 rgba('#e8fbff'), rgba('#2fb8ff'), int(210 * life))
    for i in range(9):
        rr = random.Random(77 + d * 5 + i)
        side = rr.uniform(-1.8, 1.8)
        spd = rr.uniform(1.5, 3.4)
        x = c0 + bx * (6 + k * spd * 1.5) + nx * side * k * spd
        y = c0 + by * (6 + k * spd * 1.5) + ny * side * k * spd
        cv.put(x, y, rgba('#dffaff', int(255 * life)))
        cv.put(x + bx, y + by, rgba('#6fdcff', int(170 * life)))
    if k == 0:
        glow(cv, c0, c0, 14, rgba('#7fe3ff'), 120)
    return cv.im


def emit_comet(d, k):
    """Comet: a long soft tail of light that cools from white to blue, embers shed off it."""
    cv = Cv(TW, TW)
    (bx, by), (nx, ny) = back_axis(d)
    c0 = TW / 2
    life = 1 - k / EMIT
    if k < 3:
        band(cv, d, 2, 38, 12 - k * 2, 3, rgba('#3f9fff'), int(70 * life), 0)
        band(cv, d, 2, 30, 7 - k, 1.5, rgba('#e6faff'), int(110 * life), 0)
    s_ = 8 + k * 5
    r = 5.5 * life + 1.2
    ex, ey = c0 + bx * s_, c0 + by * s_
    glow(cv, ex, ey, r * 1.8, rgba('#4fb8ff'), int(140 * life))
    glow(cv, ex, ey, r * 1.1, rgba('#e9fbff'), int(230 * life))
    for i in range(4):
        rr = random.Random(400 + d * 3 + i)
        ang = rr.uniform(0, 2 * math.pi)
        dist = 4 + k * rr.uniform(1.5, 3.0)
        x, y = ex + math.cos(ang) * dist, ey + math.sin(ang) * dist
        if k % 2 == i % 2:
            star4(cv, x, y, rgba('#fffbe6', int(255 * life)))
        else:
            cv.put(x, y, rgba('#ffffff', int(230 * life)))
    return cv.im


AURORA = ['#3dffa8', '#38e8ff', '#7d6bff', '#ff5fd2']


def emit_apex(d, k):
    """Apex: an aurora wake - two wide curtains of light rippling off his back in shifting colour, a white core
    streak, prism glints; a shock cone ahead of him on the newest copy."""
    cv = Cv(TW, TW)
    (bx, by), (nx, ny) = back_axis(d)
    c0 = TW / 2
    life = 1 - k / EMIT
    for strand, off in ((1, 0), (-1, 2)):
        col = rgba(AURORA[(k + off) % 4])
        for i in range(0, 30, 2):
            s_ = 4 + k * 3 + i
            w = strand * (2 + i * 0.32) * (0.6 + 0.4 * math.sin(k * 1.1 + i * 0.25 + d))
            a = int(120 * life * (1 - i / 34))
            glow(cv, c0 + bx * s_ + nx * w, c0 + by * s_ + ny * w, 4.0 + i * 0.15, col, a)
            if i % 4 == 0:
                cv.put(c0 + bx * s_ + nx * w, c0 + by * s_ + ny * w, (255, 255, 255, int(200 * life)))
    if k < 2:   # the white core streak
        for i in range(26):
            s_ = 3 + i
            cv.put(c0 + bx * s_, c0 + by * s_, (255, 255, 255, int(255 * (1 - i / 26))))
    for i in range(3):
        rr = random.Random(610 + d * 7 + i)
        ang = rr.uniform(-0.9, 0.9) + math.atan2(by, bx)
        dist = 8 + k * rr.uniform(2.5, 4.5)
        star4(cv, c0 + math.cos(ang) * dist, c0 + math.sin(ang) * dist, hsv(i / 3 + k * 0.1, 0.35, 1.0, int(255 * life)), big=(k == 1))
    if k == 0:
        fx_, fy_ = -bx, -by
        for side in (-1, 1):
            for i in range(13):
                x = c0 + fx_ * (16 - i) + nx * side * i * 0.9
                y = c0 + fy_ * (16 - i) + ny * side * i * 0.9
                cv.put(x, y, (255, 255, 255, 235 - i * 14))
    return cv.im


def emitter(t, d, k):
    if t == 4:   # round 99: the path opens a wormhole (levi_wormhole.py)
        import levi_wormhole
        return levi_wormhole.trail(d, k, EMIT)
    return {2: emit_storm, 3: emit_comet, 4: emit_apex}[t](d, k)


# ------------------------------------------------------------------ forms (looping buff visuals that follow him)

FW = 64


def form(t, f):
    if t == 4:   # round 99: warped space round him
        import levi_wormhole
        return levi_wormhole.form(f, 96)   # round 100: the void sphere, drawn over him
    cv = Cv(FW, FW)
    c0 = FW / 2
    if t == 2:
        # Stormcutter: arcs jumping around his body, a cyan shimmer
        glow(cv, c0, c0, 22, rgba('#3fc4ff'), 45 + (f % 2) * 20)
        rnd = random.Random(1200 + f)
        for _ in range(3 if f % 2 == 0 else 2):
            a0 = rnd.uniform(0, 2 * math.pi)
            a1 = a0 + rnd.uniform(0.8, 1.8)
            pts = []
            for i in range(6):
                a = a0 + (a1 - a0) * i / 5
                rr_ = rnd.uniform(13, 19)
                pts.append((c0 + math.cos(a) * rr_ * 0.75, c0 + math.sin(a) * rr_))
            bolt(cv, pts, WHITE, rgba('#4fd2ff'))
        for _ in range(4):
            a = rnd.uniform(0, 2 * math.pi)
            star4(cv, c0 + math.cos(a) * rnd.uniform(16, 24) * 0.8, c0 + math.sin(a) * rnd.uniform(16, 24), rgba('#9feeff'))
    elif t == 3:
        # Comet: he is a ball of light; corona rays turn, sparkles orbit
        pulse = (0, 1, 2, 1, 0, -1, -2, -1)[f]
        glow(cv, c0, c0, 28 + pulse, rgba('#5fc8ff'), 110)
        glow(cv, c0, c0, 22 + pulse * 0.5, rgba('#bff0ff'), 200)
        cv.disc(c0, c0, 15 + pulse * 0.4, (232, 251, 255, 245))
        cv.disc(c0, c0, 11, (255, 255, 255, 255))
        for i in range(8):
            a = i * math.pi / 4 + f * 0.1
            ln = 27 if i % 2 == 0 else 22
            for rr_ in range(17, ln):
                cv.put(c0 + math.cos(a) * rr_, c0 + math.sin(a) * rr_, (235, 250, 255, int(220 * (1 - (rr_ - 17) / (ln - 17)))))
        for i in range(3):
            a = f * 0.78 + i * 2.09
            star4(cv, c0 + math.cos(a) * 25, c0 + math.sin(a) * 25, rgba('#e6f9ff'), big=(f + i) % 3 == 0)
    else:
        # Apex: an aurora glow, a turning rainbow halo, prism shards orbiting, glints
        glow(cv, c0, c0, 26, rgba(AURORA[(f // 2) % 4]), 70)
        glow(cv, c0, c0, 16, rgba('#ffffff'), 60)
        for y in range(FW):
            for x in range(FW):
                d_ = math.hypot((x - c0) / 0.82, y - c0)
                if 21.5 <= d_ <= 24:
                    a = math.atan2(y - c0, x - c0) / (2 * math.pi) + f / 8
                    cv.put(x, y, hsv(a, 0.6, 1.0, 230))
                elif 20 <= d_ < 21.5:
                    a = math.atan2(y - c0, x - c0) / (2 * math.pi) + f / 8
                    cv.put(x, y, hsv(a, 0.4, 1.0, 90))
        for i in range(3):
            a = f * math.pi / 4 + i * 2.09
            sx, sy = c0 + math.cos(a) * 26 * 0.82, c0 + math.sin(a) * 26
            col = hsv(i / 3 + f / 16, 0.5, 1.0)
            lay = Cv(FW, FW)
            lay.poly([(sx, sy - 4), (sx + 2.5, sy), (sx, sy + 4), (sx - 2.5, sy)], col)
            lay.put(sx, sy - 1, WHITE)
            lay.outline(rgba('#2a1f3f'))
            cv.im.alpha_composite(lay.im)
            cv.px = cv.im.load()
        star4(cv, c0 + (f % 4 - 1.5) * 9, c0 - 26, (255, 255, 255, 255), big=f % 2 == 0)
    return cv.im


# ------------------------------------------------------------------ helpers for the mantle

def layer(cv, draw):
    lay = Cv(cv.w, cv.h)
    draw(lay)
    cv.im.alpha_composite(lay.im)
    cv.px = cv.im.load()


# ------------------------------------------------------------------ the mantle (round 84): dragged by the wind

STW = 168      # the streaming mantle while he flies or dashes, played on him in 16 directions x 4 ripple phases
MNW = 112      # the hanging mantle at rest (symmetric: a buff doesn't flip)
ST_PH = 4


def mantle_shape(c0x, c0y, back, nrm, length, w0, w1, ph, ripple=4.0, segs=12):
    """Left / right edges of a cape from (c0x, c0y) streaming along `back`, flaring from w0 to w1, rippling."""
    (bx, by), (nx, ny) = back, nrm
    left, right, spine = [], [], []
    for i in range(segs + 1):
        t = i / segs
        wv = math.sin(t * 4.2 - ph * math.pi / 2) * ripple * t
        sx, sy = c0x + bx * length * t + nx * wv, c0y + by * length * t + ny * wv
        wd = w0 + (w1 - w0) * t ** 0.8
        edge = math.sin(t * 6.5 - ph * math.pi / 2 + 1.3) * 1.6 * t
        left.append((sx + nx * (wd + edge), sy + ny * (wd + edge)))
        right.append((sx - nx * (wd - edge), sy - ny * (wd - edge)))
        spine.append((sx, sy))
    return left, right, spine


def paint_mantle(cv, t, left, right, spine, ph, rnd, big=True):
    if t == 4:   # round 99: a strip of night sky
        import levi_wormhole
        return levi_wormhole.paint_night(cv, left, right, spine, ph, rnd, ST_PH, big)
    segs = len(spine) - 1
    for i in range(segs):
        u = i / segs
        quad = [left[i], left[i + 1], right[i + 1], right[i]]
        inner = [((left[i][0] + spine[i][0]) / 2, (left[i][1] + spine[i][1]) / 2), ((left[i + 1][0] + spine[i + 1][0]) / 2, (left[i + 1][1] + spine[i + 1][1]) / 2),
                 ((right[i + 1][0] + spine[i + 1][0]) / 2, (right[i + 1][1] + spine[i + 1][1]) / 2), ((right[i][0] + spine[i][0]) / 2, (right[i][1] + spine[i][1]) / 2)]
        fade = 1 - u * 0.75
        if t == 2:      # storm: a translucent electric cape
            layer(cv, lambda l: l.poly(quad, (40, 150, 225, int(105 * fade))))
            layer(cv, lambda l: l.poly(inner, (140, 225, 255, int(95 * fade))))
        elif t == 3:    # comet: layered fire
            layer(cv, lambda l: l.poly(quad, (255, 168, 66, int(205 * fade))))
            layer(cv, lambda l: l.poly(inner, (255, 236, 150, int(240 * fade))))
        else:           # apex: aurora, the colour running along it and turning with the ripple
            hue = (u * 0.55 + ph / ST_PH * 0.25) % 1.0
            layer(cv, lambda l: l.poly(quad, hsv(hue, 0.6, 1.0, int(170 * fade))))
            layer(cv, lambda l: l.poly(inner, hsv((hue + 0.08) % 1, 0.3, 1.0, int(150 * fade))))
    if t == 2:          # lightning veins down the cape, a crackling hem
        for v in (-0.5, 0.0, 0.5):
            pts = []
            for i in range(0, segs + 1, 2):
                l_, r_ = left[i], right[i]
                q = (v + 1) / 2
                pts.append((l_[0] * (1 - q) + r_[0] * q + rnd.uniform(-1.5, 1.5), l_[1] * (1 - q) + r_[1] * q + rnd.uniform(-1.5, 1.5)))
            bolt(cv, pts, WHITE, rgba('#3fc4ff'), 210)
        for i in range(2, segs + 1, 3):
            star4(cv, left[i][0], left[i][1], rgba('#bff4ff')) if rnd.random() < 0.5 else star4(cv, right[i][0], right[i][1], rgba('#bff4ff'))
    elif t == 3:        # a white-hot core and embers torn off the end
        layer(cv, lambda l: [l.line(spine[i][0], spine[i][1], spine[i + 1][0], spine[i + 1][1], (255, 252, 235, 255)) for i in range(segs // 2)])
        for e in range(7 if big else 3):
            k = rnd.randint(segs // 2, segs)
            x, y = spine[k]
            cv.put(x + rnd.uniform(-8, 8), y + rnd.uniform(-8, 8), (255, 210, 120, rnd.randint(150, 255)))
    else:               # bright strands and glints
        for i in range(segs):
            cv.put(left[i][0], left[i][1], (255, 255, 255, 200))
            cv.put(right[i][0], right[i][1], (255, 255, 255, 160))
        for e in range(4 if big else 2):
            k = rnd.randint(2, segs)
            star4(cv, spine[k][0] + rnd.uniform(-6, 6), spine[k][1] + rnd.uniform(-6, 6), hsv(rnd.random(), 0.4, 1.0))


def stream(t, d, ph):
    """The mantle streaming behind him as he flies along direction d (22.5° steps), ripple phase ph: a body that
    flares and waves, tearing into three tongues that whip in the wind."""
    cv = Cv(STW, STW)
    a = math.radians(d * 22.5)
    back = (-math.cos(a), -math.sin(a))
    nrm = (math.sin(a), -math.cos(a))
    c0x, c0y = STW / 2 + back[0] * 3, STW / 2 - 4 + back[1] * 3
    rnd = random.Random(9000 + t * 100 + d * 7 + ph)
    left, right, spine = mantle_shape(c0x, c0y, back, nrm, 50, 6, 15, ph, ripple=6.0, segs=20)
    paint_mantle(cv, t, left, right, spine, ph, rnd)
    # three tongues torn off the end, each whipping on its own beat
    ex, ey = spine[-1]
    for k, q in enumerate((-0.65, 0.0, 0.65)):
        sx = ex + nrm[0] * 13 * q
        sy = ey + nrm[1] * 13 * q
        ln = (26, 34, 24)[k] + 4 * math.sin(ph * math.pi / 2 + k)
        l2, r2, s2 = mantle_shape(sx, sy, back, nrm, ln, 5.5, 0.6, ph + k * 1.3, ripple=7.5, segs=10)
        paint_mantle(cv, t, l2, r2, s2, ph, rnd, big=False)
    # bright edges down the body so it reads at a glance
    if t == 4:      # round 99: the hem is already prismatic; the wormhole mouth he tears open ahead of him
        import levi_wormhole
        levi_wormhole.stream_extras(cv, d, ph, ST_PH, STW)
        return cv.im
    edge = {2: (190, 240, 255, 230), 3: (255, 236, 170, 230)}[t]
    layer(cv, lambda l: [l.line(e[i][0], e[i][1], e[i + 1][0], e[i + 1][1], edge) for e in (left, right) for i in range(len(e) - 1)])
    return cv.im


def mantle(t, f):
    """At rest: the same mantle hanging from his shoulders, big, stirring a little (8 frames)."""
    cv = Cv(MNW, MNW)
    rnd = random.Random(9500 + t * 10 + f)
    for side in (-1, 1):   # two halves so it stays symmetric
        sway = math.sin(f / 8 * math.tau) * 0.12
        back = (side * 0.22 + sway * side, 1.0)
        L = math.hypot(*back)
        back = (back[0] / L, back[1] / L)
        nrm = (-back[1] * side, back[0] * side)
        left, right, spine = mantle_shape(MNW / 2 + side * 5, MNW / 2 - 12, back, nrm, 46, 4, 12, f / 2, ripple=2.0, segs=10)
        paint_mantle(cv, t, left, right, spine, f / 2, rnd, big=False)
    if t == 3:   # a little crown of stars
        for i in range(5):
            q = i / 5 * math.tau + f / 8 * math.tau * 0.25
            star4(cv, MNW / 2 + math.cos(q) * 8, MNW / 2 - 32 + math.sin(q) * 2.2, rgba('#fff2c0'), big=(i + f) % 5 == 0)
    if t == 4:   # round 99: a tiny galaxy over his head
        import levi_wormhole
        levi_wormhole.skin_extras(cv, f, MNW)
    if t == 2 and f % 2 == 0:
        star4(cv, MNW / 2 + rnd.uniform(-14, 14), MNW / 2 + rnd.uniform(-10, 20), rgba('#bff4ff'), big=True)
    return cv.im


# ------------------------------------------------------------------ ignite: the moment he reaches his form

IW = 128


def ignite(t, f):
    if t == 4:   # round 99: space cracks, the mouth snaps open
        import levi_wormhole
        return levi_wormhole.ignite(f, 160)   # round 100: bigger
    cv = Cv(IW, IW)
    c0 = IW / 2
    life = 1 - f / 8
    f = f * 1.35   # round 83: the burst reaches a third further
    if t == 2:
        glow(cv, c0, c0, 14 + f * 3, rgba('#7fe3ff'), int(200 * life))
        rnd = random.Random(2000)
        for i in range(9):
            a = i * 2 * math.pi / 9 + rnd.uniform(-0.2, 0.2)
            ln = 10 + f * 5
            pts = [(c0, c0)]
            for j in range(1, 6):
                r_ = ln * j / 5
                pts.append((c0 + math.cos(a) * r_ + rnd.uniform(-2.5, 2.5), c0 + math.sin(a) * r_ + rnd.uniform(-2.5, 2.5)))
            bolt(cv, pts, WHITE, rgba('#3fc4ff'), int(255 * life))
    elif t == 3:
        cv.ring(c0, c0, 6 + f * 5, (220, 248, 255, int(255 * life)), 2.5)
        glow(cv, c0, c0, 30 - f * 2, rgba('#ffffff'), int(255 * life))
        for i in range(12):
            a = i * math.pi / 6
            r0, r1 = 8 + f * 3, 16 + f * 5
            cv.line(c0 + math.cos(a) * r0, c0 + math.sin(a) * r0, c0 + math.cos(a) * r1, c0 + math.sin(a) * r1, (235, 250, 255, int(230 * life)))
    else:
        r_ = 8 + f * 5
        for i in range(int(2 * math.pi * r_) + 8):
            a = i / r_
            for w in range(3):
                cv.put(c0 + math.cos(a) * (r_ - w), c0 + math.sin(a) * (r_ - w), hsv(a / (2 * math.pi) + f * 0.08, 0.55, 1.0, int((255 - w * 60) * life)))
        glow(cv, c0, c0, 16, rgba('#ffffff'), int(220 * max(0, 1 - f / 3)))
        for i in range(8):
            a = i * math.pi / 4 + 0.3
            dist = 6 + f * 6
            sx, sy = c0 + math.cos(a) * dist, c0 + math.sin(a) * dist
            cv.poly([(sx, sy - 3), (sx + 2, sy), (sx, sy + 3), (sx - 2, sy)], hsv(i / 8, 0.5, 1.0, int(255 * life)))
    return cv.im


def storm_hook(f):
    cv = Cv(32, 32)
    rnd = random.Random(3000 + f)
    life = 1 - f / 5
    for _ in range(4):
        a = rnd.uniform(0, 2 * math.pi)
        pts = [(16, 16)]
        for j in range(1, 4):
            pts.append((16 + math.cos(a) * j * (3 + f) + rnd.uniform(-2, 2), 16 + math.sin(a) * j * (3 + f) + rnd.uniform(-2, 2)))
        bolt(cv, pts, WHITE, rgba('#3fc4ff'), int(255 * life))
    glow(cv, 16, 16, 7, rgba('#9feeff'), int(200 * life))
    return cv.im


def afterimage(face_left, f, tint=1):
    body = Image.open(BODY).convert('RGBA')
    with open(BODY_ANIM, encoding='utf-8') as fh:
        d = json.load(fh)['anims']['run']['frames'][2]['data']
    im = body.crop((d['x'], d['y'], d['x'] + d['w'], d['y'] + d['h']))
    if face_left:
        im = im.transpose(Image.FLIP_LEFT_RIGHT)
    a_k = [0.6, 0.4, 0.22][f]
    if tint == 4:   # round 99: dissolving into stars
        import levi_wormhole
        return levi_wormhole.afterimage(im, f, a_k)
    px = im.load()
    for y in range(im.height):
        for x in range(im.width):
            r, g, b, a = px[x, y]
            if a:
                if tint == 2:     # Stormcutter: electric blue-white
                    px[x, y] = (int(r * 0.2 + 150), int(g * 0.2 + 225), 255, int(a * a_k))
                elif tint == 3:   # Comet: white-gold light
                    px[x, y] = (255, int(g * 0.15 + 235), int(b * 0.2 + 190), int(a * a_k))
                elif tint == 4:   # Apex: a rainbow down his body
                    c = hsv(y / im.height + f * 0.15, 0.55, 1.0)
                    px[x, y] = (c[0], c[1], c[2], int(a * a_k))
                else:
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
    import levi_wormhole   # round 99: a small wormhole opens and swallows itself
    return levi_wormhole.apex_ring(f, 64)
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


def crest_shape(cv, cx, cy, fill, rim, rim_d, wide=6, tall=7):
    """A pointed shield crest centred on (cx, cy): rim, fill, and a dark lower edge."""
    pts = [(cx - wide, cy - tall + 2), (cx - wide + 2, cy - tall), (cx + wide - 2, cy - tall), (cx + wide, cy - tall + 2),
           (cx + wide, cy + 1), (cx, cy + tall + 1), (cx - wide, cy + 1)]
    cv.poly(pts, rim)
    inner = [(cx - wide + 1.2, cy - tall + 2.4), (cx - wide + 2.6, cy - tall + 1.2), (cx + wide - 2.6, cy - tall + 1.2),
             (cx + wide - 1.2, cy - tall + 2.4), (cx + wide - 1.2, cy + 0.6), (cx, cy + tall - 0.6), (cx - wide + 1.2, cy + 0.6)]
    cv.poly(inner, fill)
    cv.line(cx - wide + 1, cy + 1.5, cx, cy + tall + 0.5, rim_d)
    cv.line(cx, cy + tall + 0.5, cx + wide - 1, cy + 1.5, rim_d)
    return pts


def glint(cv, cx, cy, f, n=8, wide=7, tall=9, a=170):
    """A diagonal shine sweeping across whatever is drawn at the crest, over the cycle."""
    g = -wide - tall + (f % n) * (2 * (wide + tall) / (n - 1))
    for y in range(int(cy - tall), int(cy + tall + 1)):
        for x in range(int(cx - wide), int(cx + wide + 1)):
            d = (x - cx) + (y - cy) - g
            if 0 <= d < 2 and cv.get(x, y)[3] > 200 and cv.get(x, y)[:3] != OUT[:3]:
                cv.put(x, y, (255, 255, 255, a))


TIERS = [  # fill, rim, rim dark
    ('#3a3f47', '#9aa3ad', '#5d646e'),   # Grounded: stone
    ('#3b2a20', '#d08a52', '#8a5530'),   # Tethered: bronze
    ('#28323d', '#dfe7ef', '#8d9aa8'),   # Swinger: silver
    ('#3a2c12', '#ffcf4a', '#b48a1c'),   # Glider: gold
    ('#14324a', '#9fe0ff', '#4f9cc8'),   # Skyrunner: sky platinum
    ('#0c2a40', '#4fd2ff', '#1f7fb8'),   # Stormcutter: electric
    ('#141a33', '#fff6d6', '#e0c67a'),   # Comet: white gold on night blue
]


def badge(r, f=0):
    """Round 81: every rank a framed crest that moves (8 frames), the effects growing with the rank."""
    cv = Cv(48, 96)
    lay = Cv(48, 96)
    fill, rim, rim_d = (rgba(c) for c in TIERS[r])
    cx, cy = BX, BY
    crest_shape(lay, cx, cy, fill, rim, rim_d)
    em = Cv(48, 96)
    steel, steel_d, light = rgba('#e6edf3'), rgba('#8d9aa8'), rgba('#ffffff')
    T = math.tau
    if r == 0:      # a coiled cable
        em.ring(cx, cy - 1, 3.2, steel_d, 1.4)
        em.ring(cx, cy - 1, 1.4, steel, 1.0)
    elif r == 1:    # a hook, swinging a little
        sw = math.sin(f / 8 * T) * 1.2
        em.line(cx + sw * 0.4, cy - 5, cx + sw, cy + 1, steel)
        for i in range(7):
            a = math.pi * i / 6
            em.put(cx + sw - 2 + math.cos(a) * 2.2, cy + 1 + math.sin(a) * 2.2, steel)
        em.put(cx + sw - 4, cy, steel)
    elif r == 2:    # two crossed cables, one flicks
        em.line(cx - 4, cy - 4, cx + 4, cy + 3, steel)
        k = (0, 1, 2, 1, 0, 0, 0, 0)[f % 8]
        em.line(cx + 4, cy - 4, cx - 4 + k, cy + 3 - k, steel_d if k else steel)
        em.disc(cx, cy - 0.5, 1.0, rgba('#ffcf4a'))
    elif r == 3:    # a feathered wing that beats
        up = (0, -1, -2, -1, 0, 1, 1, 0)[f % 8]
        root = (cx - 3.5, cy + 3.5)
        tips = [(cx + 3.5, cy - 5 + up), (cx + 4.5, cy - 2.5 + up * 0.7), (cx + 4.5, cy + 0 + up * 0.4), (cx + 3, cy + 2.5)]
        em.poly([root] + tips, rgba('#fff3c4'))
        for k, t in enumerate(tips):
            em.line(root[0], root[1], t[0], t[1], rgba('#d4b25c') if k else rgba('#ffffff'))
    elif r == 4:    # wings and a chevron, wind rising
        for side in (-1, 1):
            em.poly([(cx, cy + 1), (cx + side * 5, cy - 3), (cx + side * 4.5, cy + 1), (cx + side * 2, cy + 3)], rgba('#e2f6ff'))
        em.line(cx - 2, cy - 2, cx, cy - 4, light)
        em.line(cx, cy - 4, cx + 2, cy - 2, light)
    elif r == 5:    # a blade with a bolt through it
        em.line(cx - 4, cy + 4, cx + 4, cy - 4, steel)
        em.poly([(cx + 1, cy - 5), (cx - 2, cy), (cx, cy), (cx - 1.5, cy + 5), (cx + 2.5, cy - 1), (cx + 0.5, cy - 1)], rgba('#bff4ff'))
    elif r == 6:    # a comet streaking across a night sky
        hx, hy = cx + 2, cy - 2.5
        for i in range(9):
            t = i / 8
            em.disc(hx - t * 6.5, hy + t * 5.5, max(0.3, 1.3 * (1 - t)), (255, 214, 120, int(255 - t * 170)))
        em.disc(hx, hy, 1.6, (255, 240, 190, 255))
        em.put(hx, hy, WHITE)
        em.put(cx - 3, cy - 4, (255, 255, 255, 200))
        em.put(cx + 3.5, cy + 2, (255, 255, 255, 160))
    lay.im.alpha_composite(em.im)
    lay.px = lay.im.load()
    lay.outline()
    cv.im.alpha_composite(lay.im)
    cv.px = cv.im.load()
    glint(cv, cx, cy, f)
    # the moving extras
    if r == 4:      # wind streaks rising beside the crest
        for k in range(2):
            y = cy + 6 - ((f * 2 + k * 7) % 14)
            for side in (-1, 1):
                cv.put(cx + side * 8, y, (226, 246, 255, 200))
                cv.put(cx + side * 8, y + 1, (226, 246, 255, 110))
    if r == 5:      # crackling arcs around the crest
        rnd = random.Random(500 + f)
        for _ in range(2):
            a0 = rnd.uniform(0, T)
            pts = []
            for i in range(4):
                a = a0 + i * 0.35
                rr = rnd.uniform(8.5, 10.5)
                pts.append((cx + math.cos(a) * rr * 0.85, cy + math.sin(a) * rr))
            bolt(cv, pts, WHITE, rgba('#4fd2ff'), 230)
    if r == 6:      # a spark orbiting the crest with a little tail
        for k in range(4):
            a = (f - k * 0.35) / 8 * T
            x, y = cx + math.cos(a) * 9, cy + math.sin(a) * 10
            cv.put(x, y, (255, 240, 190, 255 - k * 60))
        star4(cv, cx + math.cos(f / 8 * T) * 9, cy + math.sin(f / 8 * T) * 10, rgba('#ffe7a8'))
    if r >= 3 and f % 4 == 1:
        star4(cv, cx + 6, cy - 7, rgba('#ffffff'))
    return cv.im


DIG = {
    '0': ['111', '101', '101', '101', '111'], '1': ['010', '110', '010', '010', '111'], '2': ['111', '001', '111', '100', '111'],
    '3': ['111', '001', '111', '001', '111'], '4': ['101', '101', '111', '001', '001'], '5': ['111', '100', '111', '001', '111'],
    '6': ['111', '100', '111', '101', '111'], '7': ['111', '001', '010', '010', '010'], '8': ['111', '101', '111', '101', '111'],
    '9': ['111', '101', '111', '001', '111'],
}


def apex_badge(p, f):
    """Apex #p: an aurora crest with the position. The rim's colours turn; #1 wears a crown and aurora wings, #2-#3
    small wings, #4-#10 the turning rim and sparkles."""
    cv = Cv(48, 96)
    lay = Cv(48, 96)
    cx, cy = BX, BY
    T = math.tau
    if p <= 3:      # wings behind the crest
        span = 8 if p == 1 else 6
        flap = math.sin(f / 8 * T) * 1.2
        for side in (-1, 1):
            for k in range(3):
                hue = (f / 8 + k * 0.12 + (0.5 if side < 0 else 0)) % 1
                col = hsv(hue, 0.45, 1.0)
                tip = (cx + side * (span + 2 - k), cy - 5 + k * 2.5 + flap * (1 - k * 0.3))
                lay.poly([(cx + side * 4, cy - 2 + k * 2), tip, (cx + side * (span - 1 - k), cy + k * 2.6)], col)
    fill = rgba('#ffcf3f') if p == 1 else rgba('#1b2440')
    crest_shape(lay, cx, cy, fill, rgba('#ffffff'), rgba('#9aa7c0'))
    lay.outline()
    cv.im.alpha_composite(lay.im)
    cv.px = cv.im.load()
    # the rim in turning aurora colours
    pts = []
    for y in range(cy - 8, cy + 10):
        for x in range(cx - 7, cx + 8):
            if cv.get(x, y)[:3] == (255, 255, 255) and cv.get(x, y)[3] == 255:
                pts.append((x, y))
    for (x, y) in pts:
        a = math.atan2(y - cy, x - cx) / T
        cv.put(x, y, hsv(a + f / 8, 0.6, 1.0))
    txt = str(p)
    w = len(txt) * 4 - 1
    for i, ch in enumerate(txt):
        for j, row in enumerate(DIG[ch]):
            for k, b in enumerate(row):
                if b == '1':
                    cv.put(cx - w // 2 + i * 4 + k, cy - 3 + j, rgba('#2a1606') if p == 1 else WHITE)
    if p == 1:      # a crown, bobbing
        bob = (0, 0, -1, -1, 0, 0, 1, 1)[f % 8] * 0.5
        crown = Cv(48, 96)
        top = cy - 12 + bob
        crown.poly([(cx - 4, top + 4), (cx - 4, top + 1), (cx - 2, top + 2.5), (cx, top), (cx + 2, top + 2.5), (cx + 4, top + 1), (cx + 4, top + 4)], rgba('#ffd84a'))
        crown.put(cx, top + 2.5, rgba('#ff5fd2'))
        crown.outline()
        cv.im.alpha_composite(crown.im)
        cv.px = cv.im.load()
    glint(cv, cx, cy, f, a=150)
    # sparkles drifting around it
    for k in range(2 if p > 3 else 3):
        a = (f / 8 + k / 3) * T
        star4(cv, cx + math.cos(a) * 10, cy + math.sin(a) * 11, hsv(f / 8 + k / 3, 0.4, 1.0), big=(f + k) % 4 == 0)
    return cv.im


# ------------------------------------------------------------------ the sheet

def all_anims():
    A = {}
    for d in range(16):
        for b in range(1, 7):
            for ph in range(CABLE_PH):
                # 2 ticks: replayed every 2 ticks with the next phase, so one copy of each cable is on screen, humming
                A[f'cable_{d}_{b}_{ph}'] = ([cable(d, b, ph)], 0.034)
    A['hook'] = ([hook(f) for f in range(6)], 0.045)
    A['whiff'] = ([whiff(f) for f in range(3)], 0.06)
    for t in range(5):
        for d in range(16):
            if t < 2:
                A[f'trail{t}_{d}'] = ([trail(t, d, f) for f in range(2)], 0.034)
            else:
                A[f'trail{t}_{d}'] = ([emitter(t, d, k) for k in range(EMIT)], 0.04)
    A['after_r'] = ([afterimage(False, f) for f in range(3)], 0.05)
    A['after_l'] = ([afterimage(True, f) for f in range(3)], 0.05)
    for t in (2, 3, 4):
        A[f'after{t}_r'] = ([afterimage(False, f, t) for f in range(3)], 0.05)
        A[f'after{t}_l'] = ([afterimage(True, f, t) for f in range(3)], 0.05)
        A[f'form{t}'] = ([form(t, f) for f in range(8)], 0.06)
        A[f'ignite{t}'] = ([ignite(t, f) for f in range(8)], 0.045)
        A[f'skin{t}'] = ([mantle(t, f) for f in range(8)], 0.12)
        for d in range(16):
            for ph in range(ST_PH):
                # 2 ticks: replayed every 2 ticks on him with the next phase, so the mantle ripples as it streams
                A[f'stream{t}_{d}_{ph}'] = ([stream(t, d, ph)], 0.034)
    A['storm_hook'] = ([storm_hook(f) for f in range(5)], 0.04)
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
        A[f'rank{r}'] = ([badge(r, f) for f in range(8)], 0.1)
    for p in range(1, 11):
        A[f'apex{p}'] = ([apex_badge(p, f) for f in range(8)], 0.1)
    return A


def pack(items):
    x, y, row_h = 0, 0, 0
    placed = []
    for name, (ims, dur) in items:
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
    return sheet, {'anims': anims}


def build():
    """Two sheets: 'levi' (everything else) and 'levi_cape' (round 84: the streaming mantles, kept apart so neither
    sheet gets too tall)."""
    A = all_anims()
    # round 100: the trails on a sheet of their own ('levi_trail'), so no sheet is taller than 4096
    main, fan = pack([(k, v) for k, v in A.items() if not k.startswith(('stream', 'trail'))])
    cape, cfan = pack([(k, v) for k, v in A.items() if k.startswith('stream')])
    trl, tfan = pack([(k, v) for k, v in A.items() if k.startswith('trail')])
    return (main, fan, cape, cfan, trl, tfan), A


def flight(A, t, folder, ticks=60, speed=5.0):
    """Simulate a flight the way the game draws it: a trail copy dropped every 2 ticks where he is, each copy
    playing its own frames out, the form buff riding on him and the ignite burst when he reaches the form.
    Writes an animated GIF and returns its last frame."""
    bg = (54, 74, 60, 255)
    body = Image.open(BODY).convert('RGBA')
    with open(BODY_ANIM, encoding='utf-8') as fh:
        fr = json.load(fh)['anims']['run']['frames'][1]['data']
    me = body.crop((fr['x'], fr['y'], fr['x'] + fr['w'], fr['y'] + fr['h']))
    W, H = 300, 200
    path = []
    x, y, hd = 30.0, 160.0, -0.6
    for k in range(ticks):
        hd += 0.012 if k < ticks * 0.5 else 0.035   # a swing: arc up, then curl
        x += math.cos(hd) * speed
        y += math.sin(hd) * speed
        path.append((x, y, hd))
    trail_ims, trail_dur = A[f'trail{t}_0'][0], A[f'trail{t}_0'][1]
    form_ims, form_dur = (A[f'form{t}'] if t >= 2 else (None, 1))
    ign_ims, ign_dur = (A[f'ignite{t}'] if t >= 2 else (None, 1))
    spawned = []   # (tick, x, y, d)
    frames = []
    for k in range(0, ticks, 2):
        x, y, hd = path[k]
        d = int(round(math.degrees(hd) % 360 / 22.5)) % 16
        spawned.append((k, x, y, d))
        im = Image.new('RGBA', (W, H), bg)
        ox, oy = 210 - x, 110 - y   # the camera follows him
        for (k0, sx, sy, sd) in spawned:
            age = (k - k0) / 60
            ims, dur = A[f'trail{t}_{sd}']
            f = int(age / dur)
            if f >= len(ims):
                continue
            fi = ims[f]
            im.alpha_composite(fi, (int(sx + ox - fi.width / 2), int(sy + oy - fi.height / 2)))
        fl = me if math.cos(hd) >= 0 else me.transpose(Image.FLIP_LEFT_RIGHT)
        if t == 4:   # round 99: the night-sky mantle and the wormhole mouth ahead of him (stream4, every 3 ticks)
            fi = A[f'stream4_{d}_{(k // 3) % ST_PH}'][0][0]
            im.alpha_composite(fi, (int(x + ox - fi.width / 2), int(y + oy - fi.height / 2)))
        if t == 4:   # round 100: no cable line; the black hole on the anchor he holds, and the pull on him toward it
            import levi_wormhole
            j = min(len(path) - 1, (k // 20) * 20 + 32)
            axx, ayy, ahd = path[j]
            side = 1 if (k // 20) % 2 else -1
            ax_, ay_ = axx + math.sin(ahd) * 55 * side, ayy - math.cos(ahd) * 55 * side
            ph = (k // 3) % 4
            h = levi_wormhole.hole(ph)
            im.alpha_composite(h, (int(ax_ + ox - h.width / 2), int(ay_ + oy - h.height / 2)))
            pd = int(round(math.degrees(math.atan2(ay_ - y, ax_ - x)) % 360 / 22.5)) % 16
            pl = levi_wormhole.pull(pd, ph)
            im.alpha_composite(pl, (int(x + ox - pl.width / 2), int(y + oy - pl.height / 2)))
        im.alpha_composite(fl, (int(x + ox - 24), int(y + oy - 26)))
        if t in (2, 3, 4):   # round 100: Apex's void sphere is drawn over him (he's swallowed)
            fi = form_ims[int(k / 60 / form_dur) % len(form_ims)]
            im.alpha_composite(fi, (int(x + ox - fi.width / 2), int(y + oy - fi.height / 2)))
        if ign_ims is not None:
            f = int(k / 60 / ign_dur)
            if f < len(ign_ims):
                fi = ign_ims[f]
                ix, iy = path[0][0], path[0][1]
                im.alpha_composite(fi, (int(ix + ox - fi.width / 2), int(iy + oy - fi.height / 2)))
        frames.append(im.resize((W * 2, H * 2), Image.NEAREST).convert('RGB'))
    frames[0].save(os.path.join(folder, f'levi_flight{t}.gif'), save_all=True, append_images=frames[1:], duration=33, loop=0)
    return frames


def preview(A, folder):
    bg = (54, 74, 60, 255)
    names = {0: 'slow', 1: 'fast', 2: 'Stormcutter', 3: 'Comet', 4: 'Apex'}
    sheets = []
    for t in range(5):
        fr = flight(A, t, folder)
        sheets.append(fr[-1])
    out = Image.new('RGB', (sheets[0].width, sheets[0].height * 5), bg[:3])
    for i, s_ in enumerate(sheets):
        out.paste(s_, (0, i * s_.height))
    out.save(os.path.join(folder, 'levi_flights.png'))
    # the bits on their own: form loops, ignites, tinted afterimages, storm hook
    cell = 100
    parts = Image.new('RGBA', (cell * 8, cell * 4), bg)
    for r, t in enumerate((2, 3, 4)):
        for f in range(0, 8, 2):
            parts.alpha_composite(A[f'form{t}'][0][f], ((f // 2) * cell + 18, r * cell + 18))
        for f in (1, 3, 5, 7):
            im = A[f'ignite{t}'][0][f]
            parts.alpha_composite(im.resize((cell - 4, cell - 4)), (4 * cell + (f // 2) * cell, r * cell))
    for i, t in enumerate((1, 2, 3, 4)):
        nm = 'after_r' if t == 1 else f'after{t}_r'
        parts.alpha_composite(A[nm][0][0], (i * cell + 20, 3 * cell + 20))
    for f in range(5):
        parts.alpha_composite(A['storm_hook'][0][f], (4 * cell + f * 40, 3 * cell + 30))
    parts.resize((parts.width * 2, parts.height * 2), Image.NEAREST).save(os.path.join(folder, 'levi_parts.png'))


if __name__ == '__main__':
    (sheet, fanim, cape, cfanim, trl, tfanim), A = build()
    if '--preview' in sys.argv:
        preview(A, sys.argv[sys.argv.index('--preview') + 1])
    if '--dry' not in sys.argv:
        os.makedirs(OUT_DIR, exist_ok=True)
        for img_, fan_, name in ((sheet, fanim, 'levi'), (cape, cfanim, 'levi_cape'), (trl, tfanim, 'levi_trail')):
            img_.save(os.path.join(OUT_DIR, name + '#sheet.png'), optimize=True)
            with open(os.path.join(OUT_DIR, name + '#anim.fanim'), 'w', encoding='utf-8') as fh:
                json.dump(fan_, fh, separators=(',', ':'))
    print('sheets', sheet.size, len(fanim['anims']), 'anims;', cape.size, len(cfanim['anims']), 'cape anims;', trl.size, len(tfanim['anims']), 'trail anims')
