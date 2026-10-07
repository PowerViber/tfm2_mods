"""Round 87: Levi's rank effects, sheet 'levi_wire' (mods/tfm2_levi/vfx/levi_wire#sheet.png + #anim.fanim).

Five looks by mastery tier, wilder the higher he is (levi.rs: vfx_tier):
  0 steel    Grounded, Tethered, Swinger   (his cable stays the plain steel 'cable_' on sheet 'levi')
  1 gale     Glider, Skyrunner             wind wrapped round the wire, the wall cracks, dust and a shockwave
  2 storm    Stormcutter                   a lightning wire, a bolt strikes the anchor, the wall crackles
  3 comet    Comet                         a molten gold wire licking flame, a meteor impact with a crater and embers
  4 apex     Apex                          round 99: a space-time thread, the wall punched into a wormhole, rifts

  wire<t>_<d>_<b>_<ph>  the cable of tier t = 1..4, like 'cable_<d>_<b>_<ph>' (d x 11.25 degrees, 16*b px long, phase ph
                        of 4), cropped to its own box and drawn centred on the segment's midpoint
  bite<t>               where a cable hits the wall, t = 0..4
  spin<t>               his spin after cutting through an enemy (on him), t = 0..4
  cut<t>                the cut on each enemy he passes, t = 0..4

Run from the repo root: python3 "Claude outputs/levi/levi_wire.py" [--preview DIR] [--dry]
"""
import json
import math
import os
import random
import sys

from PIL import Image

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from levi_vfx import Cv, rgba, hsv, glow, bolt, star4, WHITE, SILVER, SILVER_GLOW, CYAN, CYAN_L, OUT_DIR  # noqa: E402

TIERS = 5
PH = 4
GOLD, GOLD_D, EMBER, FLAME_R = rgba('#ffd25a'), rgba('#e8a21a'), rgba('#ff8a2a'), rgba('#ff4a1c')
STORM, STORM_L = rgba('#3fc4ff'), rgba('#bff0ff')
CRACK, DUST = rgba('#2a2620'), rgba('#b9ab8f')


def taper(s, lead=0.35):
    """0 at both ends, 1 at `lead` (a blade is thickest just behind its leading edge)."""
    if s <= 0 or s >= 1:
        return 0.0
    return (s / lead) ** 0.6 if s < lead else ((1 - s) / (1 - lead)) ** 0.9


# ------------------------------------------------------------------ cables

def wire(t, d, b, ph):
    ln = 16 * b
    a = math.radians(d * 11.25)
    ux, uy = math.cos(a), math.sin(a)
    nx, ny = -uy, ux
    pad = 7
    w = int(math.ceil(abs(ux) * ln)) + 2 * pad
    h = int(math.ceil(abs(uy) * ln)) + 2 * pad
    cv = Cv(w, h)
    cx, cy = w / 2, h / 2
    n = int(ln * 2) + 2
    rnd = random.Random(t * 100000 + d * 1000 + b * 10 + ph)
    P = lambda s, off=0.0: (cx + ux * (s - 0.5) * ln + nx * off, cy + uy * (s - 0.5) * ln + ny * off)
    q = ph / PH
    if t == 1:
        # gale: a steel core, two wind wisps winding round it, wind dashes running along
        for i in range(n + 1):
            s = i / n
            env = math.sin(math.pi * s) ** 0.4
            for sg, hue_a, pitch in ((1, 200, 1.0), (-1, 110, 1.37)):
                off = sg * 2.8 * env * math.sin(2 * math.pi * (s * (1.1 + b * 0.3) * pitch - q) + (0.0 if sg > 0 else 1.1))
                x, y = P(s, off)
                cv.add(x, y, (226, 248, 255, hue_a))
            x, y = P(s)
            cv.put(x + nx, y + ny, SILVER_GLOW)
            cv.put(x - nx, y - ny, (159, 198, 218, 70))
            cv.put(x, y, SILVER)
        for k in range(max(2, b)):
            s0 = ((k + q) / max(2, b)) % 1
            side = 4.2 if k % 2 else -4.2
            for j in range(6):
                s = s0 + j / ln
                if 0.04 < s < 0.96:
                    x, y = P(s, side)
                    cv.add(x, y, (240, 252, 255, 150 - j * 20))
    elif t == 2:
        # storm: a white-hot core in a blue halo, a lightning arc wrapped round it, sparks
        for i in range(n + 1):
            s = i / n
            x, y = P(s)
            for o, al in ((2, 50), (-2, 50), (1, 170), (-1, 170)):
                cv.add(x + nx * o, y + ny * o, STORM[:3] + (al,))
        for i in range(n + 1):
            x, y = P(i / n)
            cv.put(x, y, WHITE)
        pts = []
        steps = max(3, int(ln / 5))
        for k in range(steps + 1):
            s = k / steps
            off = rnd.uniform(-4.0, 4.0) * math.sin(math.pi * s) ** 0.3
            pts.append(P(s, off))
        bolt(cv, pts, WHITE, STORM, 235)
        for _ in range(max(1, b // 2)):
            s = rnd.uniform(0.15, 0.85)
            x0, y0 = P(s, rnd.uniform(-2, 2))
            ang = a + rnd.choice((1, -1)) * rnd.uniform(0.6, 1.3)
            pts = [(x0, y0)]
            for j in range(1, 3):
                pts.append((x0 + math.cos(ang) * j * 2.6 + rnd.uniform(-1, 1), y0 + math.sin(ang) * j * 2.6 + rnd.uniform(-1, 1)))
            bolt(cv, pts, STORM_L, STORM, 200)
        for _ in range(b):
            x, y = P(rnd.uniform(0.05, 0.95), rnd.uniform(-5, 5))
            star4(cv, x, y, STORM) if rnd.random() < 0.4 else cv.add(x, y, STORM_L[:3] + (220,))
    elif t == 3:
        # comet: molten gold, flame licking off both sides, embers
        for i in range(n + 1):
            s = i / n
            x, y = P(s)
            for o, c in ((2, EMBER[:3] + (110,)), (-2, EMBER[:3] + (110,)), (1, GOLD), (-1, GOLD_D)):
                cv.add(x + nx * o, y + ny * o, c)
        for i in range(n + 1):
            x, y = P(i / n)
            cv.put(x, y, (255, 250, 225, 255))
        # flame tongues at uneven spacing, each its own height, curling back along the wire, flickering by phase
        g = random.Random(d * 37 + b)
        s = g.uniform(0.0, 4.0) / ln
        k = 0
        while s < 0.96:
            side = g.choice((1, -1))
            hgt = g.uniform(1.5, 5.0) * (0.6 + 0.4 * abs(math.sin(k * 2.3 + ph * 1.7)))
            curl = g.uniform(0.8, 2.2)
            m = int(hgt * 3) + 1
            for j in range(m + 1):
                f = j / m
                off = side * (1.5 + f * hgt)
                along = s - (f * f * curl + math.sin(f * 4 + ph * 1.5) * 0.4) / ln
                col = (GOLD if f < 0.3 else EMBER if f < 0.65 else FLAME_R)[:3] + (int(235 * (1 - f) + 30),)
                if 0.02 < along < 0.98:
                    x, y = P(along, off)
                    cv.add(x, y, col)
                    if f < 0.4:
                        x2, y2 = P(along + 0.6 / ln, off)
                        cv.add(x2, y2, col[:3] + (col[3] // 2,))
            s += g.uniform(3.0, 9.0) / ln
            k += 1
        for _ in range(b + 1):
            x, y = P(rnd.uniform(0.05, 0.95), rnd.choice((1, -1)) * rnd.uniform(3.5, 6))
            cv.add(x, y, rnd.choice((GOLD, EMBER, FLAME_R))[:3] + (220,))
    elif t == 4:
        # round 99: a space-time thread (levi_wormhole.py)
        import levi_wormhole
        levi_wormhole.wire(cv, P, nx, ny, n, b, q, rnd)
    else:
        # (before round 99) apex: a white core in a flowing rainbow, a rainbow double helix, glints running along
        for i in range(n + 1):
            s = i / n
            x, y = P(s)
            c = hsv(s * 1.2 + q, 0.55, 1.0)
            cv.add(x + nx * 2, y + ny * 2, (255, 255, 255, 40))
            cv.add(x - nx * 2, y - ny * 2, (255, 255, 255, 40))
            cv.add(x + nx, y + ny, c[:3] + (200,))
            cv.add(x - nx, y - ny, c[:3] + (200,))
        for i in range(n + 1):
            s = i / n
            env = math.sin(math.pi * s) ** 0.35
            for sg, h0 in ((1, 0.0), (-1, 0.5)):
                off = sg * 3.4 * env * math.sin(2 * math.pi * (s * (1.2 + b * 0.3) - q))
                x, y = P(s, off)
                cv.add(x, y, hsv(s + q + h0, 0.75, 1.0, 230))
        for i in range(n + 1):
            x, y = P(i / n)
            cv.put(x, y, WHITE)
        for k in range(max(1, b // 2)):
            s = ((k + q) / max(1, b // 2) + 0.1) % 1
            if 0.06 < s < 0.94:
                x, y = P(s)
                star4(cv, x, y, hsv(s + q, 0.5, 1.0), big=True)
    return cv.im


# ------------------------------------------------------------------ bites (where the cable hits the wall)

BITE_S = [40, 56, 72, 88, 128]   # round 100: Apex's wormhole bite is bigger
BITE_N = [6, 8, 9, 10, 10]


def claws(cv, c0, f, scale=1.0):
    open_ = (1.0, 3.0, 4.2, 4.0, 3.6, 3.4)[min(f, 5)] * scale
    for k in range(3):
        a = k * math.tau / 3 + 0.4
        tip = (c0 + math.cos(a) * open_ * 1.3, c0 + math.sin(a) * open_ * 1.3)
        cv.line(c0, c0, tip[0], tip[1], SILVER)
        cv.put(tip[0] + math.cos(a + 1.2), tip[1] + math.sin(a + 1.2), SILVER)
    cv.disc(c0, c0, 1.3 * scale, rgba('#7c8693'))
    cv.put(c0, c0, WHITE)


def cracks(cv, c0, n, length, seed, col=CRACK, a=220):
    rnd = random.Random(seed)
    for k in range(n):
        ang = k * math.tau / n + rnd.uniform(-0.3, 0.3)
        x, y = c0, c0
        seg = max(2, int(length / 3))
        for j in range(seg):
            ang += rnd.uniform(-0.5, 0.5)
            nx_, ny_ = x + math.cos(ang) * 3, y + math.sin(ang) * 3
            cv.line(x, y, nx_, ny_, col[:3] + (int(a * (1 - j / seg * 0.6)),))
            if j == seg // 2 and rnd.random() < 0.6:
                b_ang = ang + rnd.choice((1, -1)) * 0.9
                cv.line(nx_, ny_, nx_ + math.cos(b_ang) * 3, ny_ + math.sin(b_ang) * 3, col[:3] + (int(a * 0.6),))
            x, y = nx_, ny_


def bite(t, f):
    S = BITE_S[t]
    N = BITE_N[t]
    cv = Cv(S, S)
    c0 = S / 2
    life = 1 - f / N
    rnd = random.Random(500 * t + f)
    if t == 0:
        cracks(cv, c0, 3, 2 + min(f, 3) * 2, 11, a=180)
        claws(cv, c0, f)
        if 1 <= f <= 4:
            r = 3 + f * 2.5
            cv.ring(c0, c0, r, (214, 247, 255, 200 - f * 40), 1)
            for _ in range(7):
                a = rnd.uniform(0, math.tau)
                d = r - 1 + rnd.uniform(-1, 2)
                p = (c0 + math.cos(a) * d, c0 + math.sin(a) * d)
                star4(cv, *p, CYAN) if rnd.random() < 0.3 else cv.put(*p, (255, 240, 200, 230))
        if f >= 2:
            g = random.Random(9)
            for _ in range(6):
                a = g.uniform(0, math.tau)
                d = 2 + f * 1.6 + g.uniform(0, 2)
                cv.put(c0 + math.cos(a) * d, c0 + math.sin(a) * d + f * 0.5, (150, 140, 120, max(0, 230 - f * 35)))
        return cv.im
    if t == 1:
        # gale: the wall cracks, a dust cloud bursts out, a shockwave ring and a whirl of wind
        cracks(cv, c0, 7, min(16, 4 + f * 4), 21)
        if f == 0:
            glow(cv, c0, c0, 8, (255, 255, 255), 230)
        for k in range(7):
            a = k * math.tau / 7 + 0.3
            d = 4 + f * 2.6
            r = 2.5 + f * 0.7
            glow(cv, c0 + math.cos(a) * d, c0 + math.sin(a) * d, r, DUST, int(190 * life))
        if f <= 5:
            cv.ring(c0, c0, 5 + f * 4, (226, 248, 255, int(230 * (1 - f / 6))), 1.5)
        g = random.Random(77)
        for k in range(9):
            a = g.uniform(0, math.tau)
            sp = g.uniform(2.2, 3.4)
            d = 3 + f * sp
            cv.disc(c0 + math.cos(a) * d, c0 + math.sin(a) * d + f * f * 0.12, 0.9, (90, 82, 70, int(240 * life)))
        for k in range(2):
            a0 = f * 0.9 + k * math.pi
            for i in range(14):
                a = a0 + i * 0.12
                rr = 9 + f * 1.5 + i * 0.3
                cv.add(c0 + math.cos(a) * rr, c0 + math.sin(a) * rr, (240, 252, 255, int(200 * life * (1 - i / 14))))
        claws(cv, c0, f, 1.2)
        return cv.im
    if t == 2:
        # storm: a bolt strikes the anchor, the wall crackles with lightning, blue rings and sparks, a scorch
        if f >= 3:
            cv.ring(c0, c0, 6, (30, 34, 48, int(200 * life)), 2.5)
        if f <= 2:
            pts = [(c0 + rnd.uniform(-6, 6), 0)]
            steps = 7
            for k in range(1, steps + 1):
                pts.append((c0 + rnd.uniform(-5, 5) * (1 - k / steps), c0 * k / steps))
            bolt(cv, pts, WHITE, STORM, 255)
            glow(cv, c0, c0, 18 - f * 3, STORM_L, 230)
        for k in range(6):
            a = k * math.tau / 6 + rnd.uniform(-0.3, 0.3)
            ln = min(c0 - 4, 8 + f * 3.5)
            pts = [(c0, c0)]
            for j in range(1, 5):
                rr = ln * j / 4
                pts.append((c0 + math.cos(a) * rr + rnd.uniform(-2.5, 2.5), c0 + math.sin(a) * rr + rnd.uniform(-2.5, 2.5)))
            if rnd.random() < 0.85 * life + 0.15:
                bolt(cv, pts, STORM_L, STORM, int(240 * life))
        for k in range(2):
            rr = 6 + f * (3.5 + k * 1.5)
            if rr < c0 - 1:
                cv.ring(c0, c0, rr, STORM[:3] + (int(220 * life),), 1.2)
        for _ in range(10):
            a = rnd.uniform(0, math.tau)
            d = rnd.uniform(4, 8 + f * 3)
            star4(cv, c0 + math.cos(a) * d, c0 + math.sin(a) * d, STORM) if rnd.random() < 0.4 else cv.add(c0 + math.cos(a) * d, c0 + math.sin(a) * d, (255, 255, 255, int(255 * life)))
        claws(cv, c0, f, 1.3)
        return cv.im
    if t == 3:
        # comet: a meteor impact: a white-gold flash, a ring of fire, flame spikes, a crater, embers and smoke
        if f >= 1:
            cv.ring(c0, c0, 7, (40, 26, 18, int(230 * min(1, life + 0.3))), 3)
            cracks(cv, c0, 8, min(22, 6 + f * 3), 31, col=(60, 30, 14, 255))
        if f <= 2:
            glow(cv, c0, c0, 22 - f * 4, (255, 246, 210), 255)
            glow(cv, c0, c0, 12, GOLD, 220)
        if 1 <= f <= 7:
            rr = 6 + f * 4.2
            for w, col in ((0, (255, 250, 220)), (1, GOLD[:3]), (2, EMBER[:3]), (3, FLAME_R[:3])):
                cv.ring(c0, c0, rr - w, col + (int(235 * life),), 1.1)
        if f <= 5:
            for k in range(10):
                a = k * math.tau / 10 + 0.15 * f
                l0, l1 = 5 + f * 2, 10 + f * 4 + (k % 2) * 4
                for j in range(int(l1 - l0) + 1):
                    s = j / max(1, l1 - l0)
                    rr = l0 + j
                    wdt = (1 - s) * 1.6
                    col = (GOLD if s < 0.4 else EMBER if s < 0.75 else FLAME_R)[:3] + (int(240 * (1 - f / 6)),)
                    for o in (-wdt, 0, wdt):
                        cv.add(c0 + math.cos(a) * rr - math.sin(a) * o, c0 + math.sin(a) * rr + math.cos(a) * o, col)
        g = random.Random(313)
        for k in range(14):
            a = g.uniform(0, math.tau)
            sp = g.uniform(2.5, 4.5)
            d = 4 + f * sp
            if d < c0 - 2:
                x, y = c0 + math.cos(a) * d, c0 + math.sin(a) * d
                cv.add(x, y, (255, 240, 170, int(255 * life)))
                cv.add(x - math.cos(a) * 1.5, y - math.sin(a) * 1.5, EMBER[:3] + (int(200 * life),))
                cv.add(x - math.cos(a) * 3, y - math.sin(a) * 3, FLAME_R[:3] + (int(120 * life),))
        if f >= 5:
            for k in range(4):
                a = k * 1.6 + 0.5
                glow(cv, c0 + math.cos(a) * 6, c0 + math.sin(a) * 4 - (f - 5) * 3, 3 + (f - 5), (120, 110, 105), int(150 * life))
        claws(cv, c0, f, 1.4)
        return cv.im
    if t == 4:   # round 99: the wall punched into a wormhole
        import levi_wormhole
        levi_wormhole.bite(cv, c0, f, N, claws)
        return cv.im
    # (before round 99) apex: the wall shatters into crystal: a flash with light rays, two rainbow rings, shards
    if f <= 2:
        glow(cv, c0, c0, 20 - f * 4, (255, 255, 255), 255)
        for k in range(8):
            a = k * math.pi / 4 + f * 0.25
            ln = c0 - 2 - (k % 2) * 10
            for j in range(int(ln)):
                cv.add(c0 + math.cos(a) * j, c0 + math.sin(a) * j, hsv(k / 8, 0.4, 1.0, int(230 * (1 - j / ln) * (1 - f / 3))))
    if 1 <= f <= 4:
        rr = 8 + f * 3
        pts = [(c0 + math.cos(k * math.pi / 3 + 0.5) * rr, c0 + math.sin(k * math.pi / 3 + 0.5) * rr) for k in range(6)]
        for p0, p1 in zip(pts, pts[1:] + pts[:1]):
            cv.line(p0[0], p0[1], p1[0], p1[1], (255, 255, 255, 230 - f * 40))
    for k, sp in ((0, 4.8), (1, 3.2)):
        rr = 5 + f * sp
        if f >= 1 + k and rr < c0 - 1:
            m = int(math.tau * rr) + 8
            for i in range(m):
                a = i / m * math.tau
                cv.add(c0 + math.cos(a) * rr, c0 + math.sin(a) * rr, hsv(a / math.tau + f * 0.08 + k * 0.5, 0.7, 1.0, int(250 * life)))
                cv.add(c0 + math.cos(a) * (rr - 1), c0 + math.sin(a) * (rr - 1), hsv(a / math.tau + f * 0.08 + k * 0.5, 0.4, 1.0, int(150 * life)))
    g = random.Random(4040)
    for k in range(12):
        a = k * math.tau / 12 + g.uniform(-0.2, 0.2)
        sp = g.uniform(3.0, 4.6)
        d = 4 + f * sp
        if d > c0 - 4:
            continue
        x, y = c0 + math.cos(a) * d, c0 + math.sin(a) * d
        rot = a + f * 0.7 * (1 if k % 2 else -1)
        sz = 2.6 + (k % 3)
        tri = [(x + math.cos(rot) * sz, y + math.sin(rot) * sz), (x + math.cos(rot + 2.4) * sz * 0.6, y + math.sin(rot + 2.4) * sz * 0.6),
               (x + math.cos(rot - 2.4) * sz * 0.6, y + math.sin(rot - 2.4) * sz * 0.6)]
        cv.poly(tri, hsv(k / 12 + f * 0.05, 0.55, 1.0, int(240 * life + 15)))
        cv.put(tri[0][0], tri[0][1], WHITE)
    for _ in range(8):
        a = rnd.uniform(0, math.tau)
        d = rnd.uniform(6, min(c0 - 3, 10 + f * 4))
        star4(cv, c0 + math.cos(a) * d, c0 + math.sin(a) * d, hsv(rnd.random(), 0.5, 1.0), big=rnd.random() < 0.4)
    claws(cv, c0, f, 1.5)
    return cv.im


# ------------------------------------------------------------------ spins (on him, after cutting through an enemy)

SPIN_S = [48, 64, 80, 96, 136]
SPIN_N = 8


def crescent(cv, c0, a0, span, r0, r1, wmax, color_at, alpha, lead=0.3):
    """A blade of light swept from angle a0 over `span`, radius r0 -> r1 (a spiral when they differ), thickest near its
    leading edge; color_at(s, k) gives the colour at progress s along it and k = 0 (outer edge) .. 1 (inner)."""
    steps = int(abs(span) * max(r0, r1) * 1.6) + 8
    for i in range(steps + 1):
        s = i / steps
        a = a0 + span * s
        r = r0 + (r1 - r0) * s
        w = wmax * taper(1 - s, lead)
        if w <= 0.2:
            continue
        m = int(w * 2) + 1
        for j in range(m + 1):
            k = j / m
            rr = r - k * w
            c = color_at(s, k)
            cv.add(c0 + math.cos(a) * rr, c0 + math.sin(a) * rr, c[:3] + (int(c[3] * alpha * (0.35 + 0.65 * (1 - s))),))


def spin(t, f):
    S = SPIN_S[t]
    cv = Cv(S, S)
    c0 = S / 2
    life = 1 - f / SPIN_N
    grow = 0.55 + 0.45 * math.sin(min(1, (f + 1) / 4) * math.pi / 2)
    rot = f * 0.95
    rnd = random.Random(900 + t * 31 + f)
    if t == 0:
        crescent(cv, c0, rot, 3.6, 12 * grow, 15 * grow, 3.2, lambda s, k: (255, 255, 255, 255) if k < 0.6 else (190, 240, 255, 230), life)
        return cv.im
    if t == 1:
        for b in range(2):
            crescent(cv, c0, rot + b * math.pi, 3.0, 16 * grow, 22 * grow, 3.8,
                     lambda s, k: (255, 255, 255, 255) if k < 0.4 else (150, 230, 255, 235), life)
        for b in range(3):
            a0 = -rot * 0.7 + b * 2.1
            for i in range(18):
                a = a0 + i * 0.09
                rr = 26 * grow + b
                cv.add(c0 + math.cos(a) * rr, c0 + math.sin(a) * rr, (235, 250, 255, int(170 * life * (1 - i / 18))))
        for _ in range(5):
            a = rnd.uniform(0, math.tau)
            d = rnd.uniform(18, 28) * grow
            glow(cv, c0 + math.cos(a) * d, c0 + math.sin(a) * d, 2.2, (200, 190, 170), int(120 * life))
        return cv.im
    if t == 2:
        if f <= 2:
            glow(cv, c0, c0, 14, STORM_L, int(200 * (1 - f / 3)))
        for b in range(3):
            crescent(cv, c0, rot + b * math.tau / 3, 2.6, 15 * grow, 30 * grow, 4.2,
                     lambda s, k: (255, 255, 255, 255) if k < 0.35 else STORM_L if k < 0.7 else STORM, life)
        pts = []
        for k in range(13):
            a = rot * 1.3 + k * math.tau / 12
            rr = 31 * grow + rnd.uniform(-3, 3)
            pts.append((c0 + math.cos(a) * rr, c0 + math.sin(a) * rr))
        if f < SPIN_N - 1:
            bolt(cv, pts, STORM_L, STORM, int(230 * life))
        for _ in range(9):
            a = rnd.uniform(0, math.tau)
            d = rnd.uniform(20, 36) * grow
            if d < c0 - 2:
                star4(cv, c0 + math.cos(a) * d, c0 + math.sin(a) * d, STORM) if rnd.random() < 0.5 else cv.add(c0 + math.cos(a) * d, c0 + math.sin(a) * d, (255, 255, 255, int(255 * life)))
        return cv.im
    if t == 3:
        cv.ring(c0, c0, 38 * grow, EMBER[:3] + (int(120 * life),), 2)
        if f <= 2:
            glow(cv, c0, c0, 16, (255, 240, 200), int(220 * (1 - f / 3)))
        flame = lambda s, k: (255, 252, 230, 255) if k < 0.25 else GOLD if k < 0.5 else EMBER if k < 0.8 else FLAME_R
        for b in range(4):
            crescent(cv, c0, rot * 1.1 + b * math.pi / 2, 2.4, 16 * grow, 36 * grow, 6.0, flame, life)
        g = random.Random(77 + f)
        for k in range(16):
            a = g.uniform(0, math.tau)
            d = (20 + f * 2.5 + g.uniform(-3, 6)) * grow
            if d >= c0 - 3:
                continue
            x, y = c0 + math.cos(a) * d, c0 + math.sin(a) * d
            ta = a + math.pi / 2   # flung off along the spin
            cv.add(x, y, (255, 240, 170, int(255 * life)))
            for j in range(1, 4):
                cv.add(x - math.cos(ta) * j, y - math.sin(ta) * j, (EMBER if j < 3 else FLAME_R)[:3] + (int(190 * life * (1 - j / 4)),))
        return cv.im
    if t == 4:   # round 99: cosmic rifts slash round him and seal in sparks
        import levi_wormhole
        levi_wormhole.spin(cv, c0, f, SPIN_N, grow)
        return cv.im
    # (before round 99) apex: an aurora vortex: six rainbow blades in a spiral, prismatic rings, shards flung out
    if f <= 3:
        glow(cv, c0, c0, 18, (255, 255, 255), int(230 * (1 - f / 4)))
        for k in range(4):
            a = k * math.pi / 2 + f * 0.4
            for j in range(int(14 * (1 - f / 4))):
                cv.add(c0 + math.cos(a) * j, c0 + math.sin(a) * j, (255, 255, 255, 220 - j * 14))
    for b in range(6):
        h0 = b / 6 + f * 0.04
        crescent(cv, c0, rot * 1.2 + b * math.tau / 6, 2.2, 14 * grow, 42 * grow, 5.0,
                 lambda s, k, h0=h0: (255, 255, 255, 255) if k < 0.25 else hsv(h0 + s * 0.3, 0.75 - k * 0.2, 1.0, 245), life)
    for k, sp in ((0, 1.0), (1, 0.8)):
        rr = (46 + k * 4) * grow * sp + f * 1.2
        if rr < c0 - 1:
            m = int(math.tau * rr) + 8
            for i in range(m):
                a = i / m * math.tau
                cv.add(c0 + math.cos(a) * rr, c0 + math.sin(a) * rr, hsv(a / math.tau - f * 0.07 + k * 0.5, 0.6, 1.0, int(200 * life)))
    g = random.Random(4100 + f)
    for k in range(10):
        a = g.uniform(0, math.tau)
        d = (24 + f * 3.5 + g.uniform(-4, 4)) * grow
        if d >= c0 - 4:
            continue
        x, y = c0 + math.cos(a) * d, c0 + math.sin(a) * d
        r_ = a + f * 0.8
        sz = 2.5 + (k % 3)
        tri = [(x + math.cos(r_) * sz, y + math.sin(r_) * sz), (x + math.cos(r_ + 2.4) * sz * 0.6, y + math.sin(r_ + 2.4) * sz * 0.6),
               (x + math.cos(r_ - 2.4) * sz * 0.6, y + math.sin(r_ - 2.4) * sz * 0.6)]
        cv.poly(tri, hsv(k / 10 + f * 0.05, 0.55, 1.0, int(230 * life + 20)))
    for _ in range(7):
        a = g.uniform(0, math.tau)
        d = g.uniform(10, 48) * grow
        if d < c0 - 3:
            star4(cv, c0 + math.cos(a) * d, c0 + math.sin(a) * d, hsv(g.random(), 0.5, 1.0), big=g.random() < 0.5)
    return cv.im


# ------------------------------------------------------------------ cuts (on each enemy he passes)

CUT_S = [32, 40, 48, 56, 64]
CUT_N = 6


def slash(cv, x0, y0, x1, y1, wmax, color_at, prog, alpha, jag=0.0, rnd=None):
    """A blade stroke from (x0, y0) toward (x1, y1), drawn up to `prog` of its length, thickest in the middle."""
    ln = math.hypot(x1 - x0, y1 - y0)
    ux, uy = (x1 - x0) / ln, (y1 - y0) / ln
    nx, ny = -uy, ux
    steps = int(ln * 2)
    off = 0.0
    for i in range(int(steps * prog) + 1):
        s = i / steps
        if jag and rnd and i % 4 == 0:
            off = rnd.uniform(-jag, jag)
        w = wmax * math.sin(math.pi * s) ** 0.7
        m = int(w * 2) + 1
        for j in range(-m, m + 1):
            k = abs(j) / max(1, m)
            o = j / max(1, m) * w + off
            c = color_at(s, k)
            cv.add(x0 + ux * s * ln + nx * o, y0 + uy * s * ln + ny * o, c[:3] + (int(c[3] * alpha),))


def cut(t, f):
    S = CUT_S[t]
    cv = Cv(S, S)
    c0 = S / 2
    prog = min(1.0, (f + 1) / 2)
    life = 1.0 if f < 2 else 1 - (f - 1) / (CUT_N - 1)
    thin = 1.0 if f < 2 else 0.65
    r = c0 - 3
    rnd = random.Random(70 + t * 13 + f)
    white = lambda s, k: (255, 255, 255, 255) if k < 0.5 else (190, 240, 255, 200)
    if t == 0:
        slash(cv, c0 - r * 0.8, c0 - r * 0.6, c0 + r * 0.8, c0 + r * 0.6, 1.8 * thin, white, prog, life)
        return cv.im
    if t == 1:
        cyan = lambda s, k: (255, 255, 255, 255) if k < 0.4 else (130, 225, 255, 230)
        slash(cv, c0 - r * 0.8, c0 - r * 0.7, c0 + r * 0.8, c0 + r * 0.7, 2.2 * thin, cyan, prog, life)
        if f >= 1:
            slash(cv, c0 + r * 0.8, c0 - r * 0.7, c0 - r * 0.8, c0 + r * 0.7, 2.2 * thin, cyan, min(1.0, f / 2), life)
        for k in range(3):
            a = k * 2.1 + f * 0.6
            glow(cv, c0 + math.cos(a) * (6 + f * 2), c0 + math.sin(a) * (6 + f * 2), 2.4, (230, 245, 255), int(140 * life))
        return cv.im
    if t == 2:
        blue = lambda s, k: (255, 255, 255, 255) if k < 0.35 else STORM_L if k < 0.7 else STORM
        slash(cv, c0 - r * 0.85, c0 - r * 0.7, c0 + r * 0.85, c0 + r * 0.7, 2.6 * thin, blue, prog, life, jag=1.4, rnd=rnd)
        if f >= 1:
            slash(cv, c0 + r * 0.85, c0 - r * 0.7, c0 - r * 0.85, c0 + r * 0.7, 2.6 * thin, blue, min(1.0, f / 2), life, jag=1.4, rnd=rnd)
        if f <= 1:
            glow(cv, c0, c0, 9, STORM_L, 200)
        for _ in range(7):
            a = rnd.uniform(0, math.tau)
            d = rnd.uniform(5, r)
            star4(cv, c0 + math.cos(a) * d, c0 + math.sin(a) * d, STORM) if rnd.random() < 0.5 else cv.add(c0 + math.cos(a) * d, c0 + math.sin(a) * d, (255, 255, 255, int(255 * life)))
        return cv.im
    if t == 3:
        fire = lambda s, k: (255, 252, 230, 255) if k < 0.3 else GOLD if k < 0.6 else EMBER if k < 0.85 else FLAME_R
        slash(cv, c0 - r * 0.85, c0 - r * 0.75, c0 + r * 0.85, c0 + r * 0.75, 3.2 * thin, fire, prog, life)
        if f >= 1:
            slash(cv, c0 + r * 0.85, c0 - r * 0.75, c0 - r * 0.85, c0 + r * 0.75, 3.2 * thin, fire, min(1.0, f / 2), life)
        if f <= 1:
            glow(cv, c0, c0, 12, (255, 236, 180), 230)
        g = random.Random(55)
        for k in range(10):
            a = g.uniform(0, math.tau)
            d = 4 + f * g.uniform(3, 5)
            if d < c0 - 2:
                x, y = c0 + math.cos(a) * d, c0 + math.sin(a) * d
                cv.add(x, y, (255, 240, 170, int(255 * life)))
                cv.add(x - math.cos(a) * 1.5, y - math.sin(a) * 1.5, EMBER[:3] + (int(180 * life),))
        return cv.im
    if t == 4:   # round 99: a rift slash that seals in sparks
        import levi_wormhole
        levi_wormhole.cut(cv, c0, f, CUT_N, thin)
        return cv.im
    # (before round 99) apex: three rainbow strokes in a star, a white flash, shards and glints
    if f <= 1:
        glow(cv, c0, c0, 16, (255, 255, 255), 240)
    for k in range(3):
        a = k * math.pi / 3 + 0.3
        h0 = k / 3
        rainbow = lambda s, kk, h0=h0: (255, 255, 255, 255) if kk < 0.35 else hsv(h0 + s * 0.6, 0.7, 1.0, 240)
        p = min(1.0, max(0.0, (f + 1 - k * 0.5) / 2))
        if p > 0:
            slash(cv, c0 - math.cos(a) * r, c0 - math.sin(a) * r, c0 + math.cos(a) * r, c0 + math.sin(a) * r, 3.0 * thin, rainbow, p, life)
    g = random.Random(808)
    for k in range(8):
        a = k * math.tau / 8 + g.uniform(-0.3, 0.3)
        d = 6 + f * g.uniform(3, 4.5)
        if d >= c0 - 4:
            continue
        x, y = c0 + math.cos(a) * d, c0 + math.sin(a) * d
        r_ = a + f
        tri = [(x + math.cos(r_) * 3, y + math.sin(r_) * 3), (x + math.cos(r_ + 2.4) * 2, y + math.sin(r_ + 2.4) * 2),
               (x + math.cos(r_ - 2.4) * 2, y + math.sin(r_ - 2.4) * 2)]
        cv.poly(tri, hsv(k / 8, 0.55, 1.0, int(230 * life + 20)))
    for _ in range(4):
        a = rnd.uniform(0, math.tau)
        d = rnd.uniform(4, r)
        star4(cv, c0 + math.cos(a) * d, c0 + math.sin(a) * d, hsv(rnd.random(), 0.5, 1.0), big=True)
    return cv.im


# ------------------------------------------------------------------ the sheet

def all_anims():
    A = {}
    # round 100: Apex has no cable line, so no wire4; a black hole on each held anchor and the pull on him instead
    import levi_wormhole
    for ph in range(PH):
        A[f'hole4_{ph}'] = ([levi_wormhole.hole(ph)], 0.05)
        for d in range(16):
            A[f'pull4_{d}_{ph}'] = ([levi_wormhole.pull(d, ph)], 0.05)
    for t in range(1, TIERS - 1):
        for d in range(16):
            for b in range(1, 7):
                for ph in range(PH):
                    A[f'wire{t}_{d}_{b}_{ph}'] = ([wire(t, d, b, ph)], 0.034)
    for t in range(TIERS):
        A[f'bite{t}'] = ([bite(t, f) for f in range(BITE_N[t])], 0.045)
        A[f'spin{t}'] = ([spin(t, f) for f in range(SPIN_N)], 0.035)
        A[f'cut{t}'] = ([cut(t, f) for f in range(CUT_N)], 0.04)
    return A


def pack(A, width=2048):
    """Shelf packing, tallest first (the order of the anims in the file doesn't matter)."""
    frames = [(name, i, im) for name, (ims, _) in A.items() for i, im in enumerate(ims)]
    frames.sort(key=lambda e: (-e[2].height, -e[2].width))
    x = y = row_h = 0
    pos = {}
    for name, i, im in frames:
        if x + im.width > width:
            x, y, row_h = 0, y + row_h + 1, 0
        pos[(name, i)] = (x, y)
        x += im.width + 1
        row_h = max(row_h, im.height)
    sheet = Image.new('RGBA', (width, y + row_h), (0, 0, 0, 0))
    anims = {}
    for name, (ims, dur) in A.items():
        fr = []
        for i, im in enumerate(ims):
            px, py = pos[(name, i)]
            sheet.paste(im, (px, py))
            fr.append({'duration': dur, 'data': {'x': px, 'y': py, 'w': im.width, 'h': im.height}})
        anims[name] = {'frames': fr}
    return sheet, {'anims': anims}


NAMES = ['steel', 'gale', 'storm', 'comet', 'apex']
RANKS_OF = ['Grounded / Tethered / Swinger', 'Glider / Skyrunner', 'Stormcutter', 'Comet', 'Apex']


def preview(A, folder):
    """A contact sheet (levi_wire_tiers.png) and an animated GIF (levi_wire_tiers.gif): per tier a cable from him to a
    wall that bites, and his spin after cutting through an enemy (with the cut on it)."""
    from PIL import ImageDraw
    os.makedirs(folder, exist_ok=True)
    bg = (54, 74, 60, 255)
    cell_w, cell_h = 330, 140
    frames = []
    T = 26
    for k in range(T):
        im = Image.new('RGBA', (cell_w, cell_h * TIERS), bg)
        dr = ImageDraw.Draw(im)
        for t in range(TIERS):
            oy = t * cell_h
            dr.rectangle((236, oy + 22, 252, oy + 118), fill=(30, 34, 40, 255))   # a wall
            wy = oy + 70
            # the cable from him (x 40) to the wall (x 236): a chain of 2 segments of b = 6 at d = 0
            key = 'cable_0_6_{}' if t == 0 else f'wire{t}_0_6_{{}}'
            if t == 4:   # round 100: no line, the black hole on the anchor pulls him
                h = A[f'hole4_{k % 4}'][0][0]
                im.alpha_composite(h, (236 - h.width // 2, wy - h.height // 2))
                pl = A[f'pull4_0_{k % 4}'][0][0]
                im.alpha_composite(pl, (32 - pl.width // 2, wy - pl.height // 2))
            for sx in ((88, 184) if t < 4 else ()):
                if t == 0:
                    seg = _steel(k % 4)
                else:
                    seg = A[key.format(k % 4)][0][0]
                im.alpha_composite(seg, (sx - seg.width // 2, wy - seg.height // 2))
            bf = A[f'bite{t}'][0]
            b = bf[min(k // 2, len(bf) - 1)] if k < 2 * len(bf) else None
            if b is not None:
                im.alpha_composite(b, (236 - b.width // 2, wy - b.height // 2))
            # the spin on him and the cut on the enemy he just passed
            sf = A[f'spin{t}'][0]
            ks = (k - 6) // 2
            dr.ellipse((26, wy - 6, 38, wy + 6), fill=(230, 230, 240, 255))
            if 0 <= ks < len(sf):
                s_ = sf[ks]
                im.alpha_composite(s_, (32 - s_.width // 2, wy - s_.height // 2))
            cf = A[f'cut{t}'][0]
            if 0 <= ks < len(cf):
                c_ = cf[ks]
                dr.ellipse((300, wy - 6, 312, wy + 6), fill=(200, 80, 80, 255))
                im.alpha_composite(c_, (306 - c_.width // 2, wy - c_.height // 2))
            else:
                dr.ellipse((300, wy - 6, 312, wy + 6), fill=(200, 80, 80, 255))
            dr.text((6, oy + 4), f'{NAMES[t]}: {RANKS_OF[t]}', fill=(255, 255, 255, 255))
        frames.append(im.resize((im.width * 2, im.height * 2), Image.NEAREST).convert('RGB'))
    frames[0].save(os.path.join(folder, 'levi_wire_tiers.gif'), save_all=True, append_images=frames[1:], duration=70, loop=0)
    # contact sheet: every frame of every bite, spin and cut, and a cable of each tier
    rows = []
    for t in range(TIERS):
        parts = [A[f'bite{t}'][0], A[f'spin{t}'][0], A[f'cut{t}'][0]]
        w = sum(sum(f.width + 2 for f in p) + 10 for p in parts)
        h = max(max(f.height for f in p) for p in parts)
        row = Image.new('RGBA', (w, h + 4), bg)
        x = 0
        for p in parts:
            for f in p:
                row.alpha_composite(f, (x, (h - f.height) // 2 + 2))
                x += f.width + 2
            x += 10
        rows.append(row)
    W = max(r.width for r in rows)
    sheet = Image.new('RGBA', (W, sum(r.height for r in rows)), bg)
    y = 0
    for r in rows:
        sheet.alpha_composite(r, (0, y))
        y += r.height
    sheet.resize((sheet.width * 2, sheet.height * 2), Image.NEAREST).save(os.path.join(folder, 'levi_wire_frames.png'))


def _steel(ph):
    from levi_vfx import cable
    return cable(0, 6, ph)


def sync_views():
    """Round 100: Levi's data views for the Apex look: hole4 / pull4 in, wire4 out, lv_form4 drawn in front of him, the
    trails on their own sheet."""
    path = os.path.join(os.path.dirname(OUT_DIR), 'champion', 'tfm2_levi_levi.data_champion')
    with open(path, encoding='utf-8') as fh:
        data = json.load(fh)
    P = 'tfm2_levi_levi_'
    fx = [e for e in data['view_effects'] if not e['name'].startswith(P + 'wire4_')
          and not e['name'].startswith((P + 'hole4_', P + 'pull4_'))]
    for ph in range(PH):
        fx.append({'type': 'Animation', 'name': f'{P}hole4_{ph}', 'anim': 'asset/tfm2_custom/vfx/levi_wire',
                   'tag': f'hole4_{ph}', 'z': 2, 'is_follow': False})
    for d in range(16):
        for ph in range(PH):
            fx.append({'type': 'Animation', 'name': f'{P}pull4_{d}_{ph}', 'anim': 'asset/tfm2_custom/vfx/levi_wire',
                       'tag': f'pull4_{d}_{ph}', 'z': 3, 'is_follow': True})
    for e in fx:   # round 100: the trails moved to their own sheet
        if e['tag'].startswith('trail'):
            e['anim'] = 'asset/tfm2_custom/vfx/levi_trail'
    data['view_effects'] = fx
    for b in data['view_buffs']:
        if b['name'] == 'lv_form4':
            b['z'] = 3
    with open(path, 'w', encoding='utf-8') as fh:
        fh.write(json.dumps(data, indent=2, ensure_ascii=False) + '\n')
    return len(fx)


if __name__ == '__main__':
    A = all_anims()
    sheet, fan = pack(A)
    if '--preview' in sys.argv:
        preview(A, sys.argv[sys.argv.index('--preview') + 1])
    if '--dry' not in sys.argv:
        os.makedirs(OUT_DIR, exist_ok=True)
        sheet.save(os.path.join(OUT_DIR, 'levi_wire#sheet.png'), optimize=True)
        with open(os.path.join(OUT_DIR, 'levi_wire#anim.fanim'), 'w', encoding='utf-8') as fh:
            json.dump(fan, fh, separators=(',', ':'))
        print('levi data views:', sync_views())
    print('levi_wire', sheet.size, len(fan['anims']), 'anims')
