"""Round 77: Levi's body sheet, an original cable fighter (not a copy of any character's look).

Ash-white swept fringe, pale face, grey eyes; a dark charcoal coat with no harness or emblem; a long slate-teal
(#2C6170) scarf-cloak; a single silver gas tank on his back and silver grapple launchers on both wrists; twin narrow
steel blades. Frames 48x52, the anchor at the frame centre, feet on y = 45, a 1 px dark outline like the base sprites.

Tags: idle 8, run 8 (the airborne, diagonal cable pose: he "runs" while he flies), attack 6, skill1 8 (firing a
cable), skill2 6 (a gas burst), ult 8 (Rampage's whirling cut), hit 2, dead 6. Round 81: redrawn sharper (a stern
face, layered swept hair over a dark undercut, long coat tails, a cool rim light, reverse-grip blades) with in-sprite
effects: slash crescents, speed lines, gas puffs, the cable shooting out with its hook.

Run from the repo root:  python3 "Claude outputs/levi/levi_sprite.py" [--preview DIR]
Writes mods/tfm2_levi/champions/tfm2_levi_levi#sheet.png + #anim.fanim and returns the frames for the editor bundle.
"""
import json
import math
import os
import sys

from PIL import Image

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), '..', '..'))
OUT_DIR = os.path.join(ROOT, 'mods', 'tfm2_levi', 'champions')
NAME = 'tfm2_levi_levi'
W, H = 48, 52


def rgb(h, a=255):
    h = h.lstrip('#')
    return (int(h[0:2], 16), int(h[2:4], 16), int(h[4:6], 16), a)


OUT = rgb('#14111c')
HAIR, HAIR_D = rgb('#dfe3e8'), rgb('#a7aebb')
SKIN, SKIN_D = rgb('#f0d2bd'), rgb('#d2ab95')
EYE = rgb('#5b6470')
COAT, COAT_D, COAT_L = rgb('#2b303b'), rgb('#1d212a'), rgb('#3d4453')
SHIRT = rgb('#d8dde3')
TEAL, TEAL_D, TEAL_L = rgb('#2c6170'), rgb('#1e4552'), rgb('#3f8696')
PANTS, PANTS_D = rgb('#262a33'), rgb('#1b1e25')
BOOT, BOOT_D = rgb('#4a3427'), rgb('#33241b')
METAL, METAL_D, METAL_L = rgb('#b9c2cd'), rgb('#7c8693'), rgb('#e6edf3')
BLADE, BLADE_E = rgb('#eef5fb'), rgb('#9db1c4')


class C:
    def __init__(self):
        self.im = Image.new('RGBA', (W, H), (0, 0, 0, 0))
        self.px = self.im.load()

    def put(self, x, y, c):
        x, y = int(round(x)), int(round(y))
        if 0 <= x < W and 0 <= y < H:
            self.px[x, y] = c

    def get(self, x, y):
        return self.px[x, y] if 0 <= x < W and 0 <= y < H else (0, 0, 0, 0)

    def dot(self, x, y, r, c):
        for yy in range(int(y - r - 1), int(y + r + 2)):
            for xx in range(int(x - r - 1), int(x + r + 2)):
                if (xx - x) ** 2 + (yy - y) ** 2 <= r * r + 0.25:
                    self.put(xx, yy, c)

    def seg(self, a, b, r, c):
        n = int(max(abs(b[0] - a[0]), abs(b[1] - a[1])) * 2) + 1
        for i in range(n + 1):
            t = i / n
            self.dot(a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, r, c)

    def line(self, a, b, c):
        n = int(max(abs(b[0] - a[0]), abs(b[1] - a[1]))) + 1
        for i in range(n + 1):
            t = i / n
            self.put(a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, c)

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

    def outline(self):
        add = []
        for y in range(H):
            for x in range(W):
                if self.px[x, y][3] == 0 and any(self.get(x + dx, y + dy)[3] > 0 and self.get(x + dx, y + dy) != OUT
                                                  for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1))):
                    add.append((x, y))
        for x, y in add:
            self.px[x, y] = OUT


def at(p, ang, length):
    """A point `length` px from p, at `ang` degrees (0 = straight down, positive = forward / to the right)."""
    a = math.radians(ang)
    return (p[0] + math.sin(a) * length, p[1] + math.cos(a) * length)


RIM = rgb('#8fe3f0')            # the cool rim light down his front edge
HAIR_U = rgb('#454b58')         # the dark undercut under the ash-white top
GLOW, GLOW_L = rgb('#4fd2ff'), rgb('#c9f6ff')


class FX:
    """Effects drawn over the outlined figure (no outline): slash smears, speed lines, gas, the cable."""
    def __init__(self, c):
        self.c = c

    def add(self, x, y, col, a):
        x, y = int(round(x)), int(round(y))
        if not (0 <= x < W and 0 <= y < H) or a <= 0:
            return
        b = self.c.px[x, y]
        k = a / 255
        if b[3] == 0:
            self.c.px[x, y] = col[:3] + (int(a),)
        else:
            self.c.px[x, y] = tuple(int(b[i] * (1 - k) + col[i] * k) for i in range(3)) + (max(b[3], int(a)),)

    def smear(self, cx, cy, r0, r1, a0, a1, alpha=255):
        """A crescent slash from angle a0 to a1 (degrees, screen: 0 = right, 90 = down), thickest in the middle."""
        n = int(abs(a1 - a0) * 1.2) + 2
        for i in range(n + 1):
            t = i / n
            a = math.radians(a0 + (a1 - a0) * t)
            th = math.sin(t * math.pi)
            rin = r1 - (r1 - r0) * th
            for rr in range(int(rin), int(r1) + 1):
                k = (rr - rin) / max(1, r1 - rin)
                col = WHITE_ if k > 0.55 else GLOW_L if k > 0.25 else GLOW
                self.add(cx + math.cos(a) * rr, cy + math.sin(a) * rr, col, alpha * (0.35 + 0.65 * t) * (0.6 + 0.4 * th))

    def lines(self, f, n=5, y0=18, y1=44, length=(6, 12), x_end=14):
        for i in range(n):
            y = y0 + (y1 - y0) * ((i * 0.37 + f * 0.13) % 1)
            ln = length[0] + (i * 7 + f * 3) % (length[1] - length[0] + 1)
            xe = x_end - (i * 5 + f * 4) % 6
            for k in range(ln):
                self.add(xe - k, y, WHITE_, 200 * (1 - k / ln))

    def puff(self, x, y, r, alpha=230):
        for yy in range(int(y - r - 1), int(y + r + 2)):
            for xx in range(int(x - r - 1), int(x + r + 2)):
                d = math.hypot(xx - x, yy - y)
                if d <= r:
                    col = WHITE_ if d < r * 0.5 else GLOW_L
                    self.add(xx, yy, col, alpha * (1 - (d / r) ** 2))

    def wire(self, a, b, hook=True):
        n = int(max(abs(b[0] - a[0]), abs(b[1] - a[1]))) + 1
        for i in range(n + 1):
            t = i / n
            x, y = a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t
            self.add(x, y, METAL_L, 255)
            self.add(x, y + 1, GLOW, 90)
        if hook:
            ang = math.atan2(b[1] - a[1], b[0] - a[0])
            for side in (-1, 1):
                for k in range(1, 4):
                    hx = b[0] - math.cos(ang + side * 0.9) * k
                    hy = b[1] - math.sin(ang + side * 0.9) * k
                    self.add(hx, hy, METAL_L, 255)
            self.add(b[0], b[1], WHITE_, 255)

    def spark(self, x, y, s=1):
        self.add(x, y, WHITE_, 255)
        for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
            self.add(x + dx * s, y + dy * s, GLOW_L, 200)


WHITE_ = (255, 255, 255, 255)


def figure(P, effects=None):
    """One pose. P: hip, lean (deg, + = forward), legs [(thigh, shin)] (front, back), arms [(upper, fore)] (front,
    back), blades [deg or None] (front, back; screen degrees: 0 = right, 90 = down), cloak (length, lift deg, wave),
    coat (tail length, lift), sway (hair), eye ('open' | 'shut'), gas (0..3). effects(fx, joints) draws the extras."""
    c = C()
    hip = P['hip']
    lean = P.get('lean', 0)
    sh = at(hip, 180 + lean, 10)                   # shoulders, up the torso
    head = at(sh, 180 + lean * 0.7, 7)
    nx, ny = math.cos(math.radians(lean)), math.sin(math.radians(lean))   # across the torso
    wave = P.get('wave', 0.0)
    # ---- the scarf-cloak, behind everything: wide, two-tone, a lighter hem
    clen, clift = P.get('cloak', (14, 20))
    root = at(sh, 180 + lean, 0.5)
    top, bot = [(root[0] - 1, root[1] - 1)], [(root[0] + 1.5, root[1] + 1.5)]
    for i in range(1, 7):
        t = i / 6
        ang = -45 - clift * t * 0.95 + math.sin(t * 3.4 + wave) * 9 * t
        spine = at(root, ang, clen * t)
        wdt = 1.5 + 3.8 * t
        nrm = math.radians(ang + 90)
        top.append((spine[0] + math.sin(nrm) * wdt * 0.4, spine[1] + math.cos(nrm) * wdt * 0.4))
        bot.append((spine[0] - math.sin(nrm) * wdt, spine[1] - math.cos(nrm) * wdt))
    c.poly(top + list(reversed(bot)), TEAL)
    for pp in bot[2:]:
        c.put(pp[0], pp[1] - 0.5, TEAL_D)
    for pp in top[2:-1]:
        c.put(pp[0], pp[1] + 0.6, TEAL_L)
    # ---- back arm + blade
    (bu, bf) = P['arms'][1]
    b_el = at(sh, bu, 4.8)
    b_hd = at(b_el, bf, 4.6)
    c.seg(sh, b_el, 1.3, COAT_D)
    c.seg(b_el, b_hd, 1.2, COAT_D)
    blade_b = P['blades'][1]
    if blade_b is not None:
        tip_b = (b_hd[0] + math.cos(math.radians(blade_b)) * 12, b_hd[1] + math.sin(math.radians(blade_b)) * 12)
        c.line(b_hd, tip_b, BLADE_E)
        c.line((b_hd[0], b_hd[1] - 1), (tip_b[0], tip_b[1] - 1), METAL_D)
    c.dot(b_hd[0], b_hd[1], 1.2, METAL_D)
    # ---- gas tank on his back, a faint glow at the nozzle
    tk = at(hip, 180 + lean, 5)
    tk = (tk[0] - 4.0 * nx, tk[1] - 4.0 * ny)
    c.seg(at(tk, 180 + lean, 3), at(tk, lean, 2.5), 1.8, METAL_D)
    c.seg(at(tk, 180 + lean, 2.5), at(tk, lean, 2), 0.8, METAL)
    c.put(*at(tk, 180 + lean, 2.5), METAL_L)
    # ---- back leg
    (tb, sb) = P['legs'][1]
    kb = at(hip, tb, 6.2)
    fb = at(kb, sb, 6.2)
    c.seg(hip, kb, 1.5, PANTS_D)
    c.seg(kb, at(kb, sb, 3.2), 1.4, PANTS_D)
    c.seg(at(kb, sb, 3.2), fb, 1.4, BOOT_D)
    c.seg(fb, (fb[0] + 1.5, fb[1]), 1.0, BOOT_D)
    # ---- coat tails, two flaps fluttering behind the hips
    tl, tlift = P.get('coat', (7, 10))
    for k, off in enumerate((0.0, 1.6)):
        r0 = (hip[0] - w_hip_(nx) + off * nx, hip[1] - 3.0 * ny)
        tip = at(r0, -15 - tlift - k * 8 + math.sin(wave + k) * 6, tl - k)
        mid = at(r0, -8 - tlift * 0.6 - k * 5, (tl - k) * 0.55)
        c.poly([r0, (r0[0] + 2.2, r0[1]), (mid[0] + 1.2, mid[1]), tip, (mid[0] - 1, mid[1])], COAT_D if k else COAT)
    # ---- torso: coat, belt, open shirt, launcher boxes at the hips
    w_sh, w_hip = 4.3, 3.4
    torso = [(sh[0] - w_sh * nx, sh[1] - w_sh * ny), (sh[0] + w_sh * nx, sh[1] + w_sh * ny),
             (hip[0] + w_hip * nx, hip[1] + w_hip * ny), (hip[0] - w_hip * nx, hip[1] - w_hip * ny)]
    c.poly(torso, COAT)
    c.line((sh[0] - w_sh * nx + 0.5, sh[1] - w_sh * ny), (hip[0] - w_hip * nx + 0.5, hip[1] - w_hip * ny), COAT_D)
    c.line(at(sh, lean, 1), at(hip, 180 + lean, 2), SHIRT)          # the open shirt down the front
    belt = at(hip, 180 + lean, 1.5)
    c.line((belt[0] - w_hip * nx, belt[1] - w_hip * ny), (belt[0] + w_hip * nx, belt[1] + w_hip * ny), PANTS_D)
    box = (hip[0] - 1.5 * nx, hip[1] - 1.5 * ny + 1)
    c.seg((box[0] - 1.5, box[1]), (box[0] + 1.5, box[1]), 1.2, METAL_D)
    c.put(box[0] + 1, box[1] - 1, METAL_L)
    # the rim light down his front edge
    c.line((sh[0] + w_sh * nx - 0.3, sh[1] + w_sh * ny + 1), (hip[0] + w_hip * nx - 0.3, hip[1] + w_hip * ny - 1), COAT_L)
    # ---- front leg
    (tf, sf) = P['legs'][0]
    kf = at(hip, tf, 6.2)
    ff = at(kf, sf, 6.2)
    c.seg(hip, kf, 1.5, PANTS)
    c.seg(kf, at(kf, sf, 3.2), 1.4, PANTS)
    c.seg(at(kf, sf, 3.2), ff, 1.4, BOOT)
    c.seg(ff, (ff[0] + 1.8, ff[1]), 1.0, BOOT)
    c.put(kf[0] + 1, kf[1], PANTS_D)
    # ---- the scarf wrapped high at the neck
    c.seg(at(sh, lean - 90, 3), at(sh, lean + 90, 3), 1.4, TEAL)
    c.put(sh[0] + 1.5 * nx, sh[1] - 1, TEAL_L)
    c.put(sh[0] - 2 * nx, sh[1] + 1, TEAL_D)
    # ---- head: a sharp jaw, a stern eye, ash-white swept fringe over a dark undercut
    hx, hy = head
    sway = P.get('sway', 0.0)
    c.poly([(hx - 3.5, hy - 3.5), (hx + 3.8, hy - 3.5), (hx + 4.6, hy + 0.8), (hx + 3.2, hy + 3.6),
            (hx + 0.5, hy + 4.8), (hx - 3.0, hy + 3.5)], SKIN)
    c.line((hx - 3, hy + 2.5), (hx - 0.5, hy + 4.3), SKIN_D)        # jaw shadow
    c.put(hx + 3, hy + 2.6, SKIN_D)                                 # the mouth line
    c.put(hx + 2, hy + 2.6, SKIN_D)
    c.poly([(hx - 4.4, hy + 1.5), (hx - 4.6, hy - 2), (hx - 3.0, hy - 2), (hx - 3.0, hy + 1.0)], HAIR_U)   # undercut
    hair = [(hx - 4.5, hy - 1.5), (hx - 4.2, hy - 4.5), (hx - 2.0, hy - 6.5), (hx + 1.5, hy - 7.0), (hx + 4.5, hy - 5.2),
            (hx + 5.8 + sway, hy - 2.4), (hx + 4.4 + sway, hy - 2.9), (hx + 4.8 + sway, hy - 1.6), (hx + 2.6 + sway * 0.6, hy - 2.8),
            (hx + 1.5, hy - 1.4), (hx + 0.2, hy - 3.0), (hx - 1.6, hy - 2.2), (hx - 2.6, hy - 1.0)]
    c.poly(hair, HAIR)
    c.line((hx - 3.2, hy - 4.0), (hx + 1.0, hy - 5.8), rgb('#ffffff'))  # shine
    # the eye, after the hair so the fringe never swallows it: a short stern brow, a dark eye, a grey glint
    if P.get('eye', 'open') == 'open':
        c.line((hx + 0.0, hy - 0.6), (hx + 2.0, hy - 0.1), HAIR_U)
        c.put(hx + 1.0, hy + 0.9, OUT)
        c.put(hx + 2.0, hy + 0.9, rgb('#8a99ad'))
    else:
        c.line((hx + 0.0, hy + 0.9), (hx + 2.0, hy + 0.9), OUT)
    c.put(hx - 3.6, hy - 2.4, HAIR_D)
    c.put(hx + 3.2 + sway * 0.5, hy - 3.6, HAIR_D)
    c.put(hx - 0.5, hy - 3.2, HAIR_D)
    # ---- front arm + blade + launcher on the forearm
    (fu, ffo) = P['arms'][0]
    f_el = at(sh, fu, 4.8)
    f_hd = at(f_el, ffo, 4.6)
    c.seg(sh, f_el, 1.3, COAT)
    c.seg(f_el, f_hd, 1.2, COAT)
    lm = (f_el[0] * 0.4 + f_hd[0] * 0.6, f_el[1] * 0.4 + f_hd[1] * 0.6)
    c.dot(lm[0], lm[1], 1.1, METAL)
    c.put(lm[0], lm[1] - 1, METAL_L)
    blade_f = P['blades'][0]
    if blade_f is not None:
        tip_f = (f_hd[0] + math.cos(math.radians(blade_f)) * 13, f_hd[1] + math.sin(math.radians(blade_f)) * 13)
        c.line(f_hd, tip_f, BLADE)
        c.line((f_hd[0], f_hd[1] + 1), (tip_f[0], tip_f[1] + 1), BLADE_E)
    c.dot(f_hd[0], f_hd[1], 1.2, SKIN)
    c.put(f_hd[0], f_hd[1] + 1, SKIN_D)
    c.outline()
    # the rim light over the outline on his front: the cool edge
    for y in range(H):
        for x in range(W - 1, 0, -1):
            if c.px[x, y][3] and c.px[x, y] != OUT:
                if c.px[x, y][:3] in (COAT[:3], TEAL[:3]):
                    c.px[x, y] = RIM[:3] + (255,)
                break
    j = dict(hip=hip, sh=sh, head=head, f_hd=f_hd, b_hd=b_hd, tank=tk, nx=nx, ny=ny)
    if P.get('gas'):
        fx = FX(c)
        g0 = at(tk, lean, 3)
        for k in range(P['gas']):
            fx.puff(g0[0] - 2 - k * 2.2, g0[1] + 1 + k * 1.2, 1.6 + k * 0.6, 200 - k * 40)
    if effects:
        effects(FX(c), j)
    return c.im


def w_hip_(nx):
    return 3.4 * nx


def pose(**k):
    base = dict(hip=(24, 33), lean=0, legs=[(4, 0), (-6, 0)], arms=[(30, -10), (-25, 10)], blades=[60, 110],
                cloak=(14, 15), coat=(7, 10), wave=0.0, sway=0.0, eye='open', gas=0)
    base.update(k)
    return base


def frames():
    F = {}
    T = math.tau
    # idle (8): a slow breath, cloak and coat tails drifting, reverse-grip blades held low and back
    F['idle'] = [figure(pose(hip=(24, 33 + (0, 0, 1, 1, 1, 1, 0, 0)[i]), lean=4, legs=[(8, 2), (-7, -1)],
                             arms=[(25, 10), (-30, -15)], blades=[150, 160], cloak=(13, 10 + 4 * math.sin(i / 8 * T)),
                             coat=(7, 6 + 3 * math.sin(i / 8 * T + 1)), wave=i / 8 * T, sway=0.6 * math.sin(i / 8 * T)))
                 for i in range(8)]

    # run = flight (8): body at ~40°, knees tucked, the front blade swept back, cloak and coat streaming, speed lines
    def run_fx(i):
        def fx(e, j):
            e.lines(i, n=5, y0=16, y1=44, x_end=13)
            if i % 2 == 0:
                g = j['tank']
                e.puff(g[0] - 4, g[1] + 3, 1.8, 170)
        return fx
    F['run'] = [figure(pose(hip=(24, 30 + (0, -1, -1, 0, 1, 1, 0, -1)[i]), lean=40, legs=[(72, -15), (-30 - (i % 4) * 2, -55)],
                            arms=[(100, 85), (-65, -100)], blades=[195, 200], cloak=(19, 62 + 6 * math.sin(i / 8 * T)),
                            coat=(9, 40 + 6 * math.sin(i / 8 * T + 1)), wave=i / 8 * T * 2, sway=1.2), run_fx(i))
                for i in range(8)]

    # attack (6): coil, a fast forward cut with a crescent, the back blade follows through, recover
    atk = [  # (front arm, fore), blade, back arm, back blade, lean, smear
        ((-40, -20), 200, (-20, 0), 160, 0, None),
        ((-70, -60), 230, (-30, -10), 170, -5, None),
        ((70, 60), -20, (-40, -20), 170, 18, (24, 22, 9, 15, -100, 40)),
        ((110, 100), 30, (40, 60), -40, 24, (26, 24, 8, 14, -40, 90)),
        ((90, 80), 60, (80, 100), 10, 16, (27, 25, 9, 13, 20, 110)),
        ((40, 20), 120, (-10, 10), 150, 8, None),
    ]
    def atk_fx(sm):
        def fx(e, j):
            if sm:
                e.smear(sm[0], sm[1], sm[2], sm[3], sm[4], sm[5], 235)
        return fx
    F['attack'] = [figure(pose(lean=le, legs=[(25, 0), (-18, 0)], arms=[fa, ba], blades=[bl, bb], cloak=(14, 25 + i * 6),
                               coat=(7, 15 + i * 5), wave=i * 0.9, sway=0.8), atk_fx(sm))
                   for i, (fa, bl, ba, bb, le, sm) in enumerate(atk)]

    # skill1 (8): aim the launcher up and forward, the cable shoots out with its hook, a kick back, it reels in
    tips = [None, None, (34, 18), (41, 12), (46, 7), (47, 5), (41, 11), None]
    def s1_fx(i):
        def fx(e, j):
            t = tips[i]
            if t:
                e.wire((j['f_hd'][0] + 1, j['f_hd'][1] - 1), t, hook=i < 6)
            if i == 2:
                e.spark(j['f_hd'][0] + 2, j['f_hd'][1] - 2, 2)
            if i in (4, 5):
                e.spark(t[0], t[1], 1)
        return fx
    F['skill1'] = [figure(pose(lean=(8, 6, 2, -4, -6, -4, 0, 6)[i], legs=[(18, 0), (-14, 0)],
                               arms=[(112 + (0, 4, 10, 14, 14, 14, 9, 3)[i], 118 + (0, 4, 8, 12, 12, 12, 7, 2)[i]), (-30, 0)],
                               blades=[None, 150], cloak=(14, 22 + i * 3), coat=(7, 14 + i * 2), wave=i * 0.8, sway=0.4), s1_fx(i))
                   for i in range(8)]

    # skill2 (6): crouch, the gas bursts out behind in big puffs, he shoots forward low
    def s2_fx(i):
        def fx(e, j):
            g = j['tank']
            for k in range(min(i, 4)):
                e.puff(g[0] - 3 - k * 3.5, g[1] + 2 + k * 1.5, 2.2 + k * 0.9, 220 - k * 35)
            if i >= 3:
                e.lines(i, n=3, y0=22, y1=40, x_end=12)
        return fx
    F['skill2'] = [figure(pose(hip=(24, (34, 35, 33, 32, 32, 33)[i]), lean=(5, 10, 30, 38, 34, 20)[i],
                               legs=[(35 + i * 6, -15), (-25 - i * 7, -15)], arms=[(70, 50), (-70, -50)], blades=[190, 195],
                               cloak=(14 + i * 1.5, 25 + i * 9), coat=(7 + i * 0.4, 15 + i * 6), wave=i * 1.1, sway=0.8), s2_fx(i))
                   for i in range(6)]

    # ult (8): Rampage, a whirling double cut: the blades sweep round him, a full ring of crescents
    def ult_fx(i):
        def fx(e, j):
            a0 = i * 45
            e.smear(24, 28, 11, 17, a0 - 90, a0 + 20, 240)
            e.smear(24, 28, 11, 17, a0 + 90, a0 + 200, 200)
            if i % 2:
                e.spark(24 + math.cos(math.radians(a0)) * 18, 28 + math.sin(math.radians(a0)) * 18, 2)
        return fx
    F['ult'] = [figure(pose(lean=(0, 15, 25, 15, 0, -10, -15, -5)[i], legs=[(30, -5), (-28, 5)],
                            arms=[(80 + i * 45, 80 + i * 45), (-100 + i * 45, -100 + i * 45)],
                            blades=[(i * 45) % 360, (i * 45 + 180) % 360], cloak=(16, 45 + 15 * math.sin(i / 8 * T)),
                            coat=(8, 30 + 10 * math.sin(i / 8 * T)), wave=i / 8 * T * 2, sway=1.0), ult_fx(i))
                for i in range(8)]
    # hit (2): knocked back, eyes shut
    F['hit'] = [figure(pose(lean=-14 - i * 4, legs=[(8, 0), (-12, 6)], arms=[(45, 70), (-55, -30)], blades=[60, 150],
                            cloak=(12, 6), coat=(6, 4), eye='shut', sway=-0.8)) for i in range(2)]
    dead = []
    for i in range(6):
        if i < 3:
            dead.append(figure(pose(hip=(24, 33 + i * 1.5), lean=-20 - i * 22, legs=[(10, 10), (-10, 10)],
                                    arms=[(-60, -40), (-80, -60)], blades=[200, 230], cloak=(12, 0), coat=(6, 0), eye='shut')))
        else:
            im = figure(pose(hip=(24, 39), lean=-82, legs=[(80, 90), (70, 85)], arms=[(-100, -100), (-120, -110)],
                             blades=[None, None], cloak=(10, -20), coat=(6, -10), eye='shut'))
            if i >= 4:
                im.putalpha(im.getchannel('A').point(lambda a: int(a * (0.7 if i == 4 else 0.4))))
            dead.append(im)
    F['dead'] = dead
    return F


DUR = {'idle': 0.12, 'run': 0.06, 'attack': 0.05, 'skill1': 0.04, 'skill2': 0.05, 'ult': 0.06, 'hit': 0.08, 'dead': 0.1}


def build():
    F = frames()
    cols = 8
    rows = len(F)
    sheet = Image.new('RGBA', (W * cols, H * rows), (0, 0, 0, 0))
    anims = {}
    for r, (tag, ims) in enumerate(F.items()):
        out = []
        for i, im in enumerate(ims):
            sheet.paste(im, (i * W, r * H))
            out.append({'duration': DUR[tag], 'data': {'x': i * W, 'y': r * H, 'w': W, 'h': H}})
        anims[tag] = {'frames': out}
    return sheet, {'anims': anims}, F


def preview(F, folder):
    bg = (60, 82, 66, 255)
    rows = []
    for tag, ims in F.items():
        row = Image.new('RGBA', (W * 8, H), bg)
        for i, im in enumerate(ims):
            row.alpha_composite(im, (i * W, 0))
        rows.append(row)
    out = Image.new('RGBA', (W * 8, H * len(rows)), bg)
    for k, r in enumerate(rows):
        out.alpha_composite(r, (0, k * H))
    out.resize((out.width * 5, out.height * 5), Image.NEAREST).save(os.path.join(folder, 'levi_body.png'))


if __name__ == '__main__':
    sheet, fanim, F = build()
    if '--preview' in sys.argv:
        preview(F, sys.argv[sys.argv.index('--preview') + 1])
    if '--dry' not in sys.argv:
        os.makedirs(OUT_DIR, exist_ok=True)
        sheet.save(os.path.join(OUT_DIR, NAME + '#sheet.png'), optimize=True)
        with open(os.path.join(OUT_DIR, NAME + '#anim.fanim'), 'w', encoding='utf-8') as fh:
            json.dump(fanim, fh, separators=(',', ':'))
    print('sheet', sheet.size)
