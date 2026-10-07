"""Round 99: Levi's Apex look, the path opens a wormhole (Rian: "he travels through a wormhole, not on the ground, but
like the path opened a wormhole, crazy animation is okay").

Only the Apex tier changes (the rainbow one); the names, frame counts and timings stay, so the game plays exactly as
many effects as before. levi_vfx.py and levi_wire.py call these for tier 4:

  trail4_<d>      the tunnel he leaves: a funnel of prismatic portal rings behind him, narrowing into the distance,
                  a starfield rushing up it toward him; the copies dropped along his path chain into one tube that
                  twists (the rings' colours and beads turn copy to copy) and collapses behind him as it ages
  lv_form4        warped space around him: a lensing ring with broken accretion arcs, stars spiralling in
  stream4_<d>_<p> his mantle as a strip of night sky with a prismatic hem, and the wormhole mouth he tears open
                  just ahead of him (a dark throat, a rainbow rim, spiral arms turning inward)
  lv_skin4        the night-sky mantle hanging at rest, a tiny galaxy turning over his head
  ignite4         reaching Apex speed: space cracks round him, the mouth snaps open, a shockwave and shards
  after4_l|r      his afterimage, dissolving into stars
  apex_ring       a cable bite at Apex speed: a small wormhole opens and swallows itself
  wire4_*         the cable as a space-time thread: a rainbow seam with a dark rim, star beads racing along it
  bite4           the wall punched into a wormhole: a flash, a portal that opens, turns and collapses to a star
  spin4           cosmic rifts: three slits of night sky slash round him and seal shut in sparks
  cut4            a rift slash on the enemy that seals in sparks
"""
import math
import random

from PIL import Image

from levi_vfx import Cv, hsv, glow, star4, WHITE

TAU = math.tau
SPACE = (10, 6, 30)          # the throat
SPACE_V = (62, 24, 120)      # violet near the rim
NIGHT = (24, 18, 66)         # the mantle's sky
NIGHT_L = (58, 40, 132)


def mix(a, b, t):
    t = max(0.0, min(1.0, t))
    return tuple(int(a[i] + (b[i] - a[i]) * t) for i in range(3))


def portal(cv, cx, cy, rx, ry, ang, ph, life=1.0, arms=3, stars=7, seed=0, lens=True, rim=1.0):
    """A wormhole mouth: an ellipse (semi-axes rx along `ang`, ry across it) with a dark throat, spiral arms turning
    inward with phase ph (0..1 = one turn of the pattern), a rainbow rim with a white inner edge, a faint lensing halo,
    and stars spiralling into it."""
    if rx < 1 or ry < 1 or life <= 0:
        return
    ca, sa = math.cos(ang), math.sin(ang)
    R = max(rx, ry) * (1.5 if lens else 1.15) + 2
    for y in range(int(cy - R), int(cy + R) + 2):
        for x in range(int(cx - R), int(cx + R) + 2):
            dx, dy = x - cx, y - cy
            u, v = dx * ca + dy * sa, -dx * sa + dy * ca
            r = math.hypot(u / rx, v / ry)
            th = math.atan2(v / ry, u / rx)
            if r < 0.86:
                base = mix(SPACE, SPACE_V, (r / 0.86) ** 1.6)
                arm = 0.5 + 0.5 * math.cos(arms * th + 5.0 * math.log(r + 0.06) - ph * TAU)
                arm = arm ** 5 * min(1.0, r / 0.35)
                col = tuple(min(255, int(base[i] + arm * (150, 110, 255)[i] * 0.9)) for i in range(3))
                cv.add(x, y, col + (int(240 * life),))
            elif r < 1.08:
                q = (r - 0.86) / 0.22
                c = hsv(th / TAU + ph * 0.5, 0.7, 1.0)
                if q < 0.35:
                    c = mix(c, (255, 255, 255), 0.65)
                a = 255 * life * rim * (1 - max(0.0, q - 0.55) * 1.6)
                cv.add(x, y, c[:3] + (max(0, int(a)),))
            elif lens and r < 1.5:
                q = (r - 1.08) / 0.42
                a = 80 * life * (1 - q) ** 1.5 * (0.55 + 0.45 * math.cos(2 * th - ph * TAU))
                cv.add(x, y, (205, 225, 255, max(0, int(a))))
    rnd = random.Random(seed)
    for i in range(stars):
        t0, th0 = rnd.random(), rnd.uniform(0, TAU)
        k = (t0 + ph) % 1.0                 # 0 far out .. 1 swallowed
        rr = 1.45 * (1 - k) + 0.1
        th = th0 + 2.6 * k
        u, v = math.cos(th) * rr * rx, math.sin(th) * rr * ry
        x, y = cx + u * ca - v * sa, cy + u * sa + v * ca
        a = int(255 * life * min(1.0, (1 - k) * 2.2))
        if a > 20:
            if rr > 1.1 and i % 3 == 0:
                star4(cv, x, y, hsv(th0 / TAU, 0.4, 1.0, a))
            else:
                cv.add(x, y, (255, 255, 255, a))


def ellipse_ring(cv, cx, cy, rx, ry, ang, color_at, width=1.2, back_dim=1.0):
    """An ellipse outline; color_at(theta) gives RGBA. The half with v < 0 is multiplied by back_dim (seen through)."""
    ca, sa = math.cos(ang), math.sin(ang)
    n = int(TAU * max(rx, ry) * 2.2) + 12
    for i in range(n):
        th = i / n * TAU
        u, v = math.cos(th) * rx, math.sin(th) * ry
        c = color_at(th)
        if v < 0:
            c = c[:3] + (int(c[3] * back_dim),)
        for w in ((0.0,) if width < 1.5 else (-0.5, 0.5)):
            x = cx + (u + w * math.cos(th)) * ca - (v + w * math.sin(th)) * sa
            y = cy + (u + w * math.cos(th)) * sa + (v + w * math.sin(th)) * ca
            cv.add(x, y, c)


# ------------------------------------------------------------------ the tunnel behind him (trail4_<d>)

TW4 = 96
TUBE_L = 46     # how far back one copy's tunnel reaches (px)
TUBE_R = 17     # its mouth's half-width at him


def trail(d, k, emit):
    """Copy age k of `emit`: the wormhole tube behind him, flying along d (22.5 degree steps). Its mouth is round him,
    it narrows into the distance behind; prismatic portal rings stand across it, a starfield streams up it toward
    him. As the copy ages the tube collapses (narrower, dimmer) and the rings twist on."""
    cv = Cv(TW4, TW4)
    c0 = TW4 / 2
    a = math.radians(d * 22.5)
    bx, by = -math.cos(a), -math.sin(a)          # behind him
    nx, ny = math.sin(a), -math.cos(a)           # across his path
    life = 1 - k / emit
    shrink = 1 - 0.13 * k
    L = TUBE_L * (1 - 0.06 * k)

    def R(s):
        return TUBE_R * shrink * max(0.0, 1 - s / L) ** 0.7

    # the tube's inside: night that deepens with distance, light streaks rushing toward him
    for y in range(TW4):
        for x in range(TW4):
            dx, dy = x - c0, y - c0
            s = dx * bx + dy * by
            w = dx * nx + dy * ny
            if s < 1 or s > L:
                continue
            r = R(s)
            if r < 1 or abs(w) > r:
                continue
            depth = s / L
            edge = abs(w) / r
            col = mix(SPACE_V, SPACE, depth * 0.8 + 0.2)
            streak = (0.5 + 0.5 * math.cos(TAU * (s / 7.0 + k * 0.33) + edge * 2.0)) ** 10 * (1 - depth)
            col = tuple(min(255, int(col[i] + streak * (120, 140, 255)[i])) for i in range(3))
            alpha = (150 + 70 * edge ** 2) * life * (1 - depth * 0.35)
            cv.add(x, y, col + (int(alpha),))
            if r - abs(w) < 1.3:   # the tube's walls: prismatic, brighter near him
                cv.add(x, y, hsv(depth * 0.9 + k * 0.11 + (0.5 if w > 0 else 0.0), 0.6, 1.0, int(230 * life * (1 - depth * 0.6))))
    # the starfield rushing up the tunnel toward him (each frame they're further up it)
    rnd = random.Random(7000 + d)
    for i in range(14):
        s0, u = rnd.uniform(0, 1), rnd.uniform(-0.85, 0.85)
        s = ((s0 - k * 0.14) % 1.0) * L
        r = R(s)
        if r < 2:
            continue
        x, y = c0 + bx * s + nx * u * r, c0 + by * s + ny * u * r
        a_ = int(255 * life * (1 - s / L * 0.5))
        cv.add(x, y, (255, 255, 255, a_))
        cv.add(x + bx, y + by, (190, 210, 255, a_ // 2))   # a streak behind it
        cv.add(x + bx * 2, y + by * 2, (150, 160, 255, a_ // 4))
    # the portal rings standing across it, receding: each twists its colours and its bead with the copy's age
    ang = math.atan2(ny, nx)
    for j in range(4):
        s = 3 + j * 10.5 + k * 1.5
        r = R(s)
        if r < 2.5:
            continue
        h0 = j * 0.21 + k * 0.13
        fade = life * (1 - s / L * 0.55)
        col = lambda th, h0=h0, fade=fade: hsv(th / TAU + h0, 0.62, 1.0, int(245 * fade))
        ellipse_ring(cv, c0 + bx * s, c0 + by * s, r, r * 0.34, ang, col, width=1.6 if j == 0 else 1.0, back_dim=0.45)
        th = k * 0.95 + j * 1.9          # a bead of light runs round each ring
        u, v = math.cos(th) * r, math.sin(th) * r * 0.34
        star4(cv, c0 + bx * s + u * math.cos(ang) - v * math.sin(ang), c0 + by * s + u * math.sin(ang) + v * math.cos(ang),
              hsv(h0, 0.35, 1.0, int(255 * fade)), big=j == 0 and k < 2)
    if k == 0:   # the newest copy: the rim of the mouth he's just come through flashes white
        ellipse_ring(cv, c0 + bx * 2, c0 + by * 2, TUBE_R + 1, (TUBE_R + 1) * 0.34, ang,
                     lambda th: (255, 255, 255, 200), width=1.0, back_dim=0.5)
    return cv.im


# ------------------------------------------------------------------ warped space round him (lv_form4, a buff: no heading)

def form(f, size=64):
    cv = Cv(size, size)
    c0 = size / 2
    ph = f / 8
    # the lensing ring: a thin band of bent starlight, squashed like the old halo
    for y in range(size):
        for x in range(size):
            dd = math.hypot((x - c0) / 0.82, y - c0)
            if 22.0 <= dd <= 23.6:
                th = math.atan2(y - c0, x - c0)
                cv.add(x, y, hsv(th / TAU - ph, 0.35, 1.0, int(110 + 70 * math.cos(2 * th - ph * TAU))))
    # three broken accretion arcs, rainbow, turning
    for i in range(3):
        a0 = ph * TAU + i * TAU / 3
        for j in range(26):
            th = a0 + j * 0.045
            for rr in (18.5, 19.5, 20.5):
                x, y = c0 + math.cos(th) * rr * 0.82, c0 + math.sin(th) * rr
                cv.add(x, y, hsv(i / 3 + j / 60 + ph, 0.65, 1.0, int(235 * (1 - j / 30))))
    # stars falling in along spirals
    rnd = random.Random(31)
    for i in range(7):
        t0, th0 = rnd.random(), rnd.uniform(0, TAU)
        k = (t0 + ph) % 1.0
        rr = 29 * (1 - k) + 8
        th = th0 + 2.2 * k
        x, y = c0 + math.cos(th) * rr * 0.82, c0 + math.sin(th) * rr
        a = int(255 * min(1.0, (1 - k) * 2))
        if i % 2 == 0:
            star4(cv, x, y, hsv(th0 / TAU, 0.4, 1.0, a), big=k < 0.25)
        else:
            cv.add(x, y, (255, 255, 255, a))
    return cv.im


# ------------------------------------------------------------------ the night-sky mantle and the mouth ahead (stream4)

def paint_night(cv, left, right, spine, ph, rnd, st_ph, big=True):
    """The mantle as a strip of night sky: deep blue, stars that twinkle with the ripple, a prismatic hem."""
    from levi_vfx import layer
    segs = len(spine) - 1
    for i in range(segs):
        u = i / segs
        quad = [left[i], left[i + 1], right[i + 1], right[i]]
        inner = [tuple((left[j][c] + spine[j][c]) / 2 for c in (0, 1)) for j in (i, i + 1)] + \
                [tuple((right[j][c] + spine[j][c]) / 2 for c in (0, 1)) for j in (i + 1, i)]
        fade = 1 - u * 0.6
        layer(cv, lambda l: l.poly(quad, NIGHT + (int(225 * fade),)))
        layer(cv, lambda l: l.poly(inner, mix(NIGHT, NIGHT_L, 0.5 + 0.5 * math.sin(u * 5 - ph * TAU / st_ph)) + (int(150 * fade),)))
    # its stars (fixed in the cloth, twinkling) and a few bright ones
    for i in range(segs * (2 if big else 1)):
        k = rnd.randint(1, segs - 1)
        q = rnd.uniform(0.15, 0.85)
        x = left[k][0] * (1 - q) + right[k][0] * q
        y = left[k][1] * (1 - q) + right[k][1] * q
        tw = (i + ph) % 3
        cv.add(x, y, (255, 255, 255, 255 if tw == 0 else 150))
    for e in range(3 if big else 1):
        k = rnd.randint(2, segs)
        star4(cv, spine[k][0] + rnd.uniform(-3, 3), spine[k][1] + rnd.uniform(-3, 3), hsv(rnd.random(), 0.35, 1.0))
    # the prismatic hem
    for side in (left, right):
        for i in range(segs):
            c = hsv(i / segs * 0.8 + ph / st_ph * 0.25, 0.6, 1.0, int(235 * (1 - i / segs * 0.4)))
            cv.line(side[i][0], side[i][1], side[i + 1][0], side[i + 1][1], c)


def stream_extras(cv, d, ph, st_ph, size):
    """The mouth he tears open just ahead of him, tilted across his path, turning."""
    a = math.radians(d * 22.5)
    fx, fy = math.cos(a), math.sin(a)
    cx, cy = size / 2 + fx * 27, size / 2 - 4 + fy * 27
    portal(cv, cx, cy, 16, 6.5, a + math.pi / 2, ph / st_ph, seed=400 + d, stars=6)


def skin_extras(cv, f, size):
    """A tiny galaxy turning over his head (the halo of the other looks)."""
    cx, cy = size / 2, size / 2 - 32
    for arm in range(2):
        for j in range(18):
            t = j / 18
            th = arm * math.pi + t * 3.2 + f / 8 * TAU
            rr = 1.5 + t * 9
            cv.add(cx + math.cos(th) * rr, cy + math.sin(th) * rr * 0.35, hsv(t * 0.7 + f / 8, 0.5, 1.0, int(240 * (1 - t * 0.6))))
    cv.add(cx, cy, (255, 255, 255, 255))
    star4(cv, cx, cy, (230, 220, 255, 255))


# ------------------------------------------------------------------ ignite4: space cracks, the mouth snaps open

def ignite(f, size=128):
    cv = Cv(size, size)
    c0 = size / 2
    life = 1 - f / 8
    rnd = random.Random(4242)
    # the cracks: jagged white fractures with rainbow fringes, longest on the first frames
    cracks = []
    for i in range(9):
        a = i * TAU / 9 + rnd.uniform(-0.25, 0.25)
        pts = [(c0, c0)]
        ln = rnd.uniform(34, 52)
        for j in range(1, 7):
            r_ = ln * j / 6
            a += rnd.uniform(-0.3, 0.3)
            pts.append((c0 + math.cos(a) * r_, c0 + math.sin(a) * r_))
        cracks.append(pts)
    reach = min(1.0, (f + 1) / 2.5)
    if f <= 5:
        for i, pts in enumerate(cracks):
            n = max(2, int(len(pts) * reach))
            for (x0, y0), (x1, y1) in zip(pts[:n], pts[1:n]):
                fringe = hsv(i / 9 + f * 0.05, 0.7, 1.0, int(200 * life))
                for o in (-1, 1):
                    cv.line(x0 + o, y0, x1 + o, y1, fringe)
                cv.line(x0, y0, x1, y1, (255, 255, 255, int(255 * (1 - f / 6))))
    if f <= 1:
        glow(cv, c0, c0, 22, (255, 255, 255), 230)
    # the mouth snaps open (frames 2-5) and closes round him (6-7)
    open_ = (0, 0.35, 0.8, 1.0, 1.0, 0.9, 0.6, 0.3)[f]
    if open_ > 0:
        portal(cv, c0, c0, 30 * open_, 26 * open_, 0.0, f / 8, life=min(1.0, 0.5 + life), seed=77, stars=10, arms=4)
    # the shockwave: a rainbow ring and a faint second one
    for k, sp in ((0, 7.5), (1, 5.5)):
        rr = 10 + f * sp
        if f >= 1 + k and rr < c0 - 2:
            m = int(TAU * rr) + 10
            for i in range(m):
                th = i / m * TAU
                for w in range(2 if k == 0 else 1):
                    cv.add(c0 + math.cos(th) * (rr - w), c0 + math.sin(th) * (rr - w),
                           hsv(th / TAU + f * 0.07 + k * 0.5, 0.6, 1.0, int((250 - w * 90) * life)))
    # shards of space flung out
    g = random.Random(99)
    for i in range(10):
        a = i * TAU / 10 + g.uniform(-0.2, 0.2)
        dist = 14 + f * g.uniform(5, 7)
        if dist > c0 - 4 or f < 1:
            continue
        sx, sy = c0 + math.cos(a) * dist, c0 + math.sin(a) * dist
        r_ = a + f * 0.6
        sz = 3 + i % 3
        tri = [(sx + math.cos(r_) * sz, sy + math.sin(r_) * sz), (sx + math.cos(r_ + 2.4) * sz * 0.6, sy + math.sin(r_ + 2.4) * sz * 0.6),
               (sx + math.cos(r_ - 2.4) * sz * 0.6, sy + math.sin(r_ - 2.4) * sz * 0.6)]
        cv.poly(tri, NIGHT_L + (int(240 * life),))
        cv.line(tri[0][0], tri[0][1], tri[1][0], tri[1][1], hsv(i / 10, 0.55, 1.0, int(255 * life)))
        cv.add(tri[0][0], tri[0][1], WHITE)
    return cv.im


# ------------------------------------------------------------------ after4: his afterimage dissolving into stars

def afterimage(im, f, a_k):
    """`im` is his flying pose; frame f of 3 at alpha a_k: night-blue with a rainbow rim, crumbling into stars."""
    im = im.copy()
    px = im.load()
    w, h = im.size
    gone = (0.08, 0.4, 0.72)[f]
    rnd = random.Random(55)
    solid = [[px[x, y][3] > 0 for x in range(w)] for y in range(h)]
    sparks = []
    for y in range(h):
        for x in range(w):
            r, g, b, a = px[x, y]
            if not a:
                continue
            roll = rnd.random()
            if roll < gone:
                px[x, y] = (0, 0, 0, 0)
                if roll < gone * 0.12:
                    sparks.append((x, y))
                continue
            rim = any(not (0 <= x + dx < w and 0 <= y + dy < h and solid[y + dy][x + dx]) for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)))
            if rim:
                c = hsv(y / h + f * 0.15, 0.6, 1.0)
            else:
                c = mix(NIGHT, NIGHT_L, (r + g + b) / 765)
                if (x * 7 + y * 13 + f) % 17 == 0:
                    c = (255, 255, 255)
            px[x, y] = c[:3] + (int(a * a_k),)
    pad = 6
    out = Image.new('RGBA', (w + 2 * pad, h + 2 * pad), (0, 0, 0, 0))
    out.paste(im, (pad, pad))
    cv = Cv(out.width, out.height)
    cv.im = out
    cv.px = out.load()
    for i, (x, y) in enumerate(sparks):   # the stars it crumbles into drift off upward
        sx, sy = x + pad + (i % 3 - 1) * f, y + pad - f * 2
        if i % 4 == 0:
            star4(cv, sx, sy, hsv(i / 7, 0.4, 1.0, int(255 * a_k / 0.6)))
        else:
            cv.add(sx, sy, (255, 255, 255, int(255 * a_k / 0.6)))
    return cv.im


# ------------------------------------------------------------------ apex_ring: a cable bite at Apex speed

def apex_ring(f, size=64):
    cv = Cv(size, size)
    c0 = size / 2
    open_ = (0.3, 0.8, 1.0, 0.7, 0.25)[f]
    life = 1 - f / 6
    portal(cv, c0, c0, 15 * open_, 13 * open_, 0.0, f / 5, life=1.0, seed=12, stars=5, lens=f >= 1)
    rr = 8 + f * 5.5
    m = int(TAU * rr) + 6
    for i in range(m):
        th = i / m * TAU
        cv.add(c0 + math.cos(th) * rr, c0 + math.sin(th) * rr, hsv(th / TAU + f * 0.1, 0.5, 1.0, int(240 * life)))
    if f == 0:
        glow(cv, c0, c0, 10, (255, 255, 255), 230)
    return cv.im


# ------------------------------------------------------------------ wire4: a space-time thread

def wire(cv, P, nx, ny, n, b, q, rnd):
    """Drawn on the levi_wire canvas: P(s, off) maps 0..1 along the cable (and an offset across) to pixels; q is the
    hum phase 0..1. A dark seam either side, a rainbow core with a white heart, star beads racing along."""
    for i in range(n + 1):
        s = i / n
        x, y = P(s)
        for o in (-2, 2):
            cv.add(x + nx * o, y + ny * o, SPACE_V + (110,))
        for o in (-1, 1):
            cv.add(x + nx * o, y + ny * o, hsv(s * 1.1 - q, 0.6, 1.0, 190))
    for i in range(n + 1):
        x, y = P(i / n)
        cv.put(x, y, WHITE)
    # the beads: a star with a short comet tail, racing along with the hum (0..1 of the cable a phase step)
    beads = max(1, b // 2 + 1)
    for k in range(beads):
        s = ((k + q) / beads + 0.05) % 1.0
        if 0.06 < s < 0.94:
            x, y = P(s)
            star4(cv, x, y, hsv(s + k * 0.3, 0.4, 1.0), big=True)
            for j in range(1, 4):
                tx, ty = P(s - j * 1.6 / (16 * b))
                cv.add(tx, ty, (220, 230, 255, 200 - j * 55))
    # little ripples of bent space either side, travelling with the beads
    for k in range(b):
        s = (k / b + q / b) % 1.0
        if 0.08 < s < 0.92:
            for sg in (1, -1):
                x, y = P(s, sg * 4)
                cv.add(x, y, (190, 200, 255, 90))


# ------------------------------------------------------------------ bite4: the wall punched into a wormhole

def bite(cv, c0, f, n, claws):
    life = 1 - f / n
    if f <= 1:
        glow(cv, c0, c0, 22 - f * 6, (255, 255, 255), 255)
    open_ = (0.15, 0.55, 0.9, 1.0, 1.0, 0.95, 0.8, 0.55, 0.3, 0.1)[min(f, 9)]
    portal(cv, c0, c0, 30 * open_, 25 * open_, 0.3, f / n, life=1.0, seed=600, stars=9, arms=3)
    # fractures round the hole, lit from inside
    rnd = random.Random(4040)
    for i in range(7):
        a = i * TAU / 7 + rnd.uniform(-0.3, 0.3)
        r0 = 30 * open_ + 1
        x, y = c0 + math.cos(a) * r0 * 0.95, c0 + math.sin(a) * r0 * 0.8
        for j in range(3):
            a += rnd.uniform(-0.5, 0.5)
            x1, y1 = x + math.cos(a) * 4, y + math.sin(a) * 4
            if max(abs(x1 - c0), abs(y1 - c0)) < c0 - 2:
                cv.line(x, y, x1, y1, hsv(i / 7 + f * 0.04, 0.5, 1.0, int(220 * life)))
            x, y = x1, y1
    if f >= n - 2:   # it collapses to a star
        star4(cv, c0, c0, (255, 255, 255, 255), big=True)
    claws(cv, c0, f, 1.5)


# ------------------------------------------------------------------ rifts: spin4 and cut4

def rift(cv, x0, y0, x1, y1, wmax, prog, seal, life, seed, hue=0.0):
    """A straight slit torn in space (see rift_path)."""
    rift_path(cv, [(x0, y0), (x1, y1)], wmax, prog, seal, life, seed, hue)


def rift_path(cv, pts, wmax, prog, seal, life, seed, hue=0.0):
    """A slit torn in space along the polyline `pts`, drawn up to `prog` of its length, widest in the middle; `seal`
    0 (wide open) .. 1 (shut). Night sky inside with stars, rainbow lips, a white seam where it is shut."""
    segs = [(p, q, math.hypot(q[0] - p[0], q[1] - p[1])) for p, q in zip(pts, pts[1:])]
    ln = sum(l for _, _, l in segs)
    if ln < 1:
        return

    def at(s):
        d = s * ln
        for p, q, l in segs:
            if d <= l or (p, q, l) == segs[-1]:
                t = d / l if l else 0.0
                ux, uy = ((q[0] - p[0]) / l, (q[1] - p[1]) / l) if l else (1.0, 0.0)
                return p[0] + (q[0] - p[0]) * t, p[1] + (q[1] - p[1]) * t, ux, uy
            d -= l
    steps = int(ln * 2)
    rnd = random.Random(seed)
    x0, y0 = pts[0]
    ux, uy = 1.0, 0.0
    for i in range(int(steps * prog) + 1):
        s = i / steps
        w = wmax * math.sin(math.pi * s) ** 0.8 * (1 - seal)
        bx_, by_, ux, uy = at(s)
        nx, ny = -uy, ux
        m = int(w * 2) + 1
        for j in range(-m, m + 1):
            o = j / max(1, m) * w
            e = abs(j) / max(1, m)
            if e > 0.7:
                c = hsv(hue + s * 0.7 + (0.5 if j > 0 else 0), 0.65, 1.0, int(245 * life))
            else:
                c = mix(SPACE, SPACE_V, e) + (int(235 * life),)
                if rnd.random() < 0.05:
                    c = (255, 255, 255, int(255 * life))
            cv.add(bx_ + nx * o, by_ + ny * o, c)
        cv.add(bx_, by_, (255, 255, 255, int((120 + 135 * seal) * life)))
    if seal > 0.3:   # sparks along the seam as it shuts
        for _ in range(int(6 * seal) + 2):
            sx, sy, ux, uy = at(rnd.uniform(0.1, 0.9) * prog)
            x, y = sx - uy * rnd.uniform(-3, 3), sy + ux * rnd.uniform(-3, 3)
            star4(cv, x, y, hsv(rnd.random(), 0.4, 1.0, int(255 * life)))


def spin(cv, c0, f, n, grow):
    life = 1 - f / n
    seal = max(0.0, (f - 3) / (n - 3))
    if f <= 1:
        glow(cv, c0, c0, 18, (200, 190, 255), int(200 * (1 - f / 2)))
    for b in range(3):   # three rifts slash round him, curving like the old blades
        a0 = f * 0.9 + b * TAU / 3
        pts = []
        for i in range(9):
            t = i / 8
            a = a0 + 1.9 * t
            r = (16 + 26 * t) * grow
            pts.append((c0 + math.cos(a) * r, c0 + math.sin(a) * r))
        rift_path(cv, pts, 5.0, min(1.0, (f + 1) / 2), seal, life, 300 + b * 17 + f * 3, hue=b / 3)
    # the dust of space they leave
    g = random.Random(4100 + f)
    for k in range(9):
        a = g.uniform(0, TAU)
        d = (22 + f * 3.5 + g.uniform(-4, 4)) * grow
        if d < c0 - 3:
            star4(cv, c0 + math.cos(a) * d, c0 + math.sin(a) * d, hsv(g.random(), 0.45, 1.0, int(255 * life)), big=g.random() < 0.3)


def cut(cv, c0, f, n, thin):
    r = c0 - 3
    life = 1.0 if f < 2 else 1 - (f - 1) / (n - 1)
    prog = min(1.0, (f + 1) / 2)
    seal = 0.0 if f < 2 else (f - 1) / (n - 1)
    if f <= 1:
        glow(cv, c0, c0, 14, (230, 220, 255), 230)
    rift(cv, c0 - r * 0.85, c0 - r * 0.7, c0 + r * 0.85, c0 + r * 0.7, 5.0 * thin, prog, seal, life, 800 + f, hue=0.0)
    if f >= 1:
        rift(cv, c0 + r * 0.6, c0 - r * 0.8, c0 - r * 0.6, c0 + r * 0.8, 3.0 * thin, min(1.0, f / 2), seal, life, 820 + f, hue=0.5)
