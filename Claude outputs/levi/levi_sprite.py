"""Round 77: Levi's body sheet, an original cable fighter (not a copy of any character's look).

Ash-white swept fringe, pale face, grey eyes; a dark charcoal coat with no harness or emblem; a long slate-teal
(#2C6170) scarf-cloak; a single silver gas tank on his back and silver grapple launchers on both wrists; twin narrow
steel blades. Frames 48x52, the anchor at the frame centre, feet on y = 45, a 1 px dark outline like the base sprites.

Tags: idle 4, run 6 (the airborne, diagonal cable pose: he "runs" while he flies), attack 5, skill1 4 (firing a
cable), skill2 4 (a gas burst), ult 6 (a spinning cut), hit 1, dead 6.

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


def figure(P):
    """Draws one pose. P: hip, lean (deg, + = forward), legs [(thigh, shin)] (front, back), arms [(upper, fore)]
    (front, back), blades [deg] (front, back; 0 = right, 90 = down), cloak (length, lift deg, wave), gas (0..3)."""
    c = C()
    hip = P['hip']
    lean = P.get('lean', 0)
    sh = at(hip, 180 + lean, 9)                    # shoulders, up the torso
    head = at(sh, 180 + lean * 0.8, 6)
    back = -1                                       # he faces right: "back" is to the left
    # ---- the scarf-cloak, behind everything
    clen, clift, cwave = P.get('cloak', (12, 20, 0))
    base_a = at(sh, 180 + lean, 1)
    root_l = (base_a[0] - 2, base_a[1])
    root_r = (base_a[0] + 1, base_a[1] + 1)
    ang = 180 - clift if clift < 90 else 180 - clift   # trailing back (to the left) and up/down by lift
    tip_ang = -90 - (90 - clift)                     # degrees in our "down = 0" system: -90 = straight left
    pts = [root_l]
    n = 5
    for i in range(1, n + 1):
        t = i / n
        p = at(root_l, -40 - clift * t * 0.9 + math.sin(t * 3.2 + cwave) * 8 * t, clen * t)
        pts.append(p)
    tip = pts[-1]
    tail2 = at(tip, -10 - clift * 0.6, 3)
    lower = [at(pp, 0, 2 + 3 * (i / n)) for i, pp in enumerate(pts[1:], 1)]
    cloak = pts + [tail2] + list(reversed(lower)) + [root_r]
    c.poly(cloak, TEAL)
    for i, pp in enumerate(pts[1:-1], 1):
        c.put(pp[0], pp[1] + 1, TEAL_L)
    for pp in lower[:-1]:
        c.put(pp[0], pp[1] - 1, TEAL_D)
    # ---- back arm + blade
    (bu, bf) = P['arms'][1]
    b_el = at(sh, bu, 4.5)
    b_hd = at(b_el, bf, 4.5)
    c.seg(sh, b_el, 1.1, COAT_D)
    c.seg(b_el, b_hd, 1.0, COAT_D)
    blade_b = P['blades'][1]
    if blade_b is not None:
        tip_b = (b_hd[0] + math.cos(math.radians(blade_b)) * 10, b_hd[1] + math.sin(math.radians(blade_b)) * 10)
        c.line(b_hd, tip_b, BLADE_E)
    c.dot(b_hd[0], b_hd[1], 1.0, METAL_D)
    # ---- gas tank on his back
    tk = at(hip, 180 + lean, 5)
    tk = (tk[0] - 3.5 * math.cos(math.radians(lean)), tk[1] - 3.5 * math.sin(math.radians(lean)) * 0.3)
    c.seg(at(tk, 180 + lean, 2), at(tk, lean, 2), 1.6, METAL_D)
    c.seg(at(tk, 180 + lean, 2), at(tk, lean, 1.5), 0.8, METAL)
    gas = P.get('gas', 0)
    if gas:
        g0 = at(tk, lean, 3)
        for k in range(gas * 2):
            c.put(g0[0] - 1 - k * 0.8, g0[1] + k * 0.6, rgb('#bff6ff') if k % 2 == 0 else rgb('#6fdcff'))
    # ---- back leg
    (tb, sb) = P['legs'][1]
    kb = at(hip, tb, 6)
    fb = at(kb, sb, 6)
    c.seg(hip, kb, 1.3, PANTS_D)
    c.seg(kb, at(kb, sb, 3), 1.2, PANTS_D)
    c.seg(at(kb, sb, 3), fb, 1.2, BOOT_D)
    c.put(fb[0] + 1, fb[1], BOOT_D)
    # ---- torso: coat, shirt line, coat tail
    w_sh, w_hip = 4.0, 3.3
    nx, ny = math.cos(math.radians(lean)), math.sin(math.radians(lean))
    torso = [(sh[0] - w_sh * nx, sh[1] - w_sh * ny), (sh[0] + w_sh * nx, sh[1] + w_sh * ny),
             (hip[0] + w_hip * nx, hip[1] + w_hip * ny), (hip[0] - w_hip * nx, hip[1] - w_hip * ny)]
    c.poly(torso, COAT)
    tail = at(hip, -20 + lean * 0.3 - P.get('tail', 0), 4)
    c.poly([(hip[0] - w_hip * nx, hip[1] - w_hip * ny), (hip[0] + 1, hip[1]), tail], COAT_D)
    c.line(at(sh, lean, 1), at(hip, 180 + lean, 1), SHIRT)          # the open shirt down the front
    c.line((sh[0] - w_sh * nx + 0.5, sh[1] - w_sh * ny), (hip[0] - w_hip * nx + 0.5, hip[1] - w_hip * ny), COAT_D)
    c.put(sh[0] + 1.5 * nx, sh[1] + 1.5 * ny + 1, COAT_L)
    # ---- front leg
    (tf, sf) = P['legs'][0]
    kf = at(hip, tf, 6)
    ff = at(kf, sf, 6)
    c.seg(hip, kf, 1.3, PANTS)
    c.seg(kf, at(kf, sf, 3), 1.2, PANTS)
    c.seg(at(kf, sf, 3), ff, 1.2, BOOT)
    c.put(ff[0] + 1, ff[1], BOOT)
    # ---- scarf wrap at the neck
    c.seg(at(sh, lean - 90, 2.5), at(sh, lean + 90, 2.5), 1.1, TEAL)
    c.put(sh[0] + 1, sh[1] - 1, TEAL_L)
    # ---- head: face, eye, swept fringe
    hx, hy = head
    c.poly([(hx - 3, hy - 3), (hx + 3, hy - 3), (hx + 3, hy + 2), (hx + 1, hy + 4), (hx - 3, hy + 3)], SKIN)
    c.put(hx - 3, hy + 2, SKIN_D)
    c.put(hx + 2, hy, EYE)
    c.put(hx + 2, hy - 1, OUT)
    c.put(hx + 1, hy + 3, SKIN_D)
    hair = [(hx - 4, hy - 1), (hx - 4, hy - 4), (hx - 2, hy - 6), (hx + 2, hy - 6), (hx + 4, hy - 4),
            (hx + 5, hy - 2), (hx + 3, hy - 2), (hx + 2, hy - 3), (hx, hy - 2), (hx - 2, hy - 2), (hx - 2, hy + 1)]
    c.poly(hair, HAIR)
    c.put(hx - 3, hy - 4, HAIR_D)
    c.put(hx - 3, hy - 2, HAIR_D)
    c.put(hx + 1, hy - 5, rgb('#ffffff'))
    # ---- front arm + blade + launcher
    (fu, ffo) = P['arms'][0]
    f_el = at(sh, fu, 4.5)
    f_hd = at(f_el, ffo, 4.5)
    c.seg(sh, f_el, 1.1, COAT)
    c.seg(f_el, f_hd, 1.0, COAT)
    blade_f = P['blades'][0]
    if blade_f is not None:
        tip_f = (f_hd[0] + math.cos(math.radians(blade_f)) * 11, f_hd[1] + math.sin(math.radians(blade_f)) * 11)
        c.line(f_hd, tip_f, BLADE)
        c.line((f_hd[0], f_hd[1] + 1), (tip_f[0], tip_f[1] + 1), BLADE_E)
    c.dot(f_hd[0], f_hd[1], 1.0, METAL)
    c.put(f_hd[0], f_hd[1], METAL_L)
    # ---- the grapple line when firing (skill1)
    if P.get('wire'):
        c.line(f_hd, P['wire'], METAL_L)
    c.outline()
    if P.get('wire'):
        c.line(f_hd, P['wire'], METAL_L)   # the wire stays thin (no outline)
    return c.im


def pose(**k):
    base = dict(hip=(24, 34), lean=0, legs=[(4, 0), (-6, 0)], arms=[(30, -10), (-25, 10)], blades=[60, 110],
                cloak=(12, 15, 0), gas=0, tail=0)
    base.update(k)
    return base


def frames():
    F = {}
    # idle: compact, blades lowered, cloak calm, a slight breath
    F['idle'] = [figure(pose(hip=(24, 34 + (1 if i in (1, 2) else 0)), lean=4, legs=[(6, 2), (-6, -2)],
                             arms=[(40, 25), (-35, -25)], blades=[55, 125], cloak=(10, 8 + i, i * 0.8)))
                 for i in range(4)]
    # run = flight: body angled ~35° forward, front knee bent up, back leg trailing, blades in an X, cloak long
    F['run'] = [figure(pose(hip=(23, 31 + (0, -1, -1, 0, 1, 1)[i]), lean=38, legs=[(70, -20), (-35 - (i % 3) * 3, -50)],
                            arms=[(95, 70), (-70, -110)], blades=[-30, 205], cloak=(17, 55 + (i % 3) * 6, i * 1.1),
                            gas=1 + (i % 2), tail=10))
                for i in range(6)]
    # attack: a fast forward cut, back blade sweeping through
    atk = [(-40, -10, 120), (60, 40, -60), (100, 90, -10), (110, 100, 20), (60, 30, 80)]
    F['attack'] = [figure(pose(lean=15 + i * 3, legs=[(25, 0), (-15, 0)], arms=[(a, b), (-30, 0)], blades=[bl, 130],
                               cloak=(12, 20 + i * 4, i)))
                   for i, (a, b, bl) in enumerate(atk)]
    # skill1: thrust the launcher forward and up, the wire shoots out
    F['skill1'] = [figure(pose(lean=10, legs=[(15, 0), (-12, 0)], arms=[(120 + i * 10, 140 + i * 8), (-25, 10)],
                               blades=[None, 110], cloak=(12, 20, i), wire=((30, 18), (38, 10), (44, 5), (46, 2))[i] if i else None))
                   for i in range(4)]
    # skill2: crouch, then a gas burst pushes him forward
    F['skill2'] = [figure(pose(hip=(24, 35 - (i > 1)), lean=10 + i * 8, legs=[(30 + i * 5, -10), (-25 - i * 6, -10)],
                               arms=[(60, 40), (-60, -40)], blades=[10, 190], cloak=(12 + i * 2, 25 + i * 10, i), gas=i))
                   for i in range(4)]
    # ult: a spinning cut, blades around him
    F['ult'] = [figure(pose(lean=(0, 20, 0, -15, 0, 20)[i], legs=[(25, -5), (-25, 5)],
                            arms=[(90 + i * 60, 90 + i * 60), (-90 + i * 60, -90 + i * 60)],
                            blades=[(i * 60) % 360, (i * 60 + 180) % 360], cloak=(15, 40 + (i % 2) * 20, i)))
                for i in range(6)]
    F['hit'] = [figure(pose(lean=-12, legs=[(5, 0), (-10, 5)], arms=[(40, 60), (-50, -30)], blades=[40, 140], cloak=(11, 5, 1)))]
    dead = []
    for i in range(6):
        if i < 3:
            dead.append(figure(pose(hip=(24, 34 + i), lean=-20 - i * 20, legs=[(10, 10), (-10, 10)],
                                    arms=[(-60, -40), (-80, -60)], blades=[200, 230], cloak=(10, 0, i))))
        else:
            im = figure(pose(hip=(24, 38), lean=-80, legs=[(80, 90), (70, 85)], arms=[(-100, -100), (-120, -110)],
                             blades=[None, None], cloak=(9, -20, 0)))
            if i >= 4:
                im.putalpha(im.getchannel('A').point(lambda a: int(a * (0.7 if i == 4 else 0.4))))
            dead.append(im)
    F['dead'] = dead
    return F


DUR = {'idle': 0.18, 'run': 0.07, 'attack': 0.06, 'skill1': 0.05, 'skill2': 0.07, 'ult': 0.07, 'hit': 0.14, 'dead': 0.1}


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
