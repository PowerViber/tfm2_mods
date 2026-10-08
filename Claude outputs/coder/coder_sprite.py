"""Round 101: the Coder's body sheet, an original hoodie hacker.

A deep-violet hoodie with the hood up, the face in shadow but for two glowing green lenses; dark jeans, white
sneakers; a translucent green holo-keyboard hovers at his hands (he types on it while standing and casting).
Frames 48x52, the anchor at the frame centre, feet on y = 45, a 1 px dark outline like the base sprites. Drawn with
the same rig as Levi's (Claude outputs/levi/levi_sprite.py: its canvas, outline and joint helpers).

Tags: idle 8 (typing, the keys lighting up), run 8 (the keyboard folds to a glowing tablet under his arm), attack 6 (a
data packet flicked from his hand), skill1 8 (overclock: the hood vents heat, sparks), skill2 6 (debug: a scanning
lens), ult 8 (the AI: a glowing orb summoned between his hands), hit 2, dead 6.

Run from the repo root: python3 "Claude outputs/coder/coder_sprite.py" [--preview]
"""
import json
import math
import os
import sys

from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, '..', 'levi'))
import levi_sprite as L  # noqa: E402
from levi_sprite import C, FX, at, rgb, W, H  # noqa: E402

ROOT = os.path.abspath(os.path.join(HERE, '..', '..'))
OUT_DIR = os.path.join(ROOT, 'mods', 'tfm2_custom', 'champions')
NAME = 'tfm2_custom_coder'

OUT = L.OUT
HOOD, HOOD_D, HOOD_L = rgb('#4a3a7a'), rgb('#33285a'), rgb('#6a58a8')
SHADOW = rgb('#141020')
LENS, LENS_L = rgb('#3cff8a'), rgb('#c8ffd9')
JEANS, JEANS_D = rgb('#2c3a5c'), rgb('#1f2944')
SHOE, SHOE_D = rgb('#e8eaf0'), rgb('#a9afbd')
SKIN, SKIN_D = rgb('#e9c4a8'), rgb('#c79d82')
STRING = rgb('#d8d8e8')
KEY, KEY_L = (60, 255, 140, 150), (200, 255, 220, 230)
WHITE = (255, 255, 255, 255)


def keyboard(c, hx, hy, phase, lit=True):
    """The holo-keyboard: a flat translucent slab in front of his hands, a few keys lit by the phase."""
    fx = FX(c)
    for row in range(3):
        y = hy + 1 + row
        for k in range(9):
            x = hx - 6 + k + row * 0.5
            a = 120 if (k + row) % 2 else 90
            fx.add(x, y, KEY, a)
    fx.add(hx - 7, hy + 1, KEY_L, 200)
    fx.add(hx + 4, hy + 3, KEY_L, 200)
    if lit:
        for j in range(2):
            k = (phase * 3 + j * 4) % 9
            row = (phase + j) % 3
            fx.add(hx - 6 + k + row * 0.5, hy + 1 + row, KEY_L, 255)
    # the glow it throws up on his hoodie
    for dx in (-3, 0, 3):
        fx.add(hx + dx, hy - 1, KEY, 60)


def figure(P, effects=None):
    """P: hip, lean, legs [(thigh, shin)] (front, back), arms [(upper, fore)] (front, back), kb (keyboard phase or
    None), lens (glasses brightness 0..1), hood_tilt."""
    c = C()
    hip = P['hip']
    lean = P.get('lean', 0)
    sh = at(hip, 180 + lean, 10)
    head = at(sh, 180 + lean * 0.7, 7)
    nx, ny = math.cos(math.radians(lean)), math.sin(math.radians(lean))
    # back arm
    (bu, bf) = P['arms'][1]
    b_el = at(sh, bu, 4.8)
    b_hd = at(b_el, bf, 4.4)
    c.seg(sh, b_el, 1.5, HOOD_D)
    c.seg(b_el, b_hd, 1.4, HOOD_D)
    c.dot(b_hd[0], b_hd[1], 1.1, SKIN_D)
    # back leg
    (tb, sb) = P['legs'][1]
    kb = at(hip, tb, 6.2)
    fb = at(kb, sb, 6.2)
    c.seg(hip, kb, 1.6, JEANS_D)
    c.seg(kb, fb, 1.5, JEANS_D)
    c.seg(fb, (fb[0] + 2, fb[1]), 1.1, SHOE_D)
    # torso: a loose hoodie, wider at the hem, a kangaroo pocket
    w_sh, w_hip = 4.6, 4.2
    torso = [(sh[0] - w_sh * nx, sh[1] - w_sh * ny), (sh[0] + w_sh * nx, sh[1] + w_sh * ny),
             (hip[0] + (w_hip + 0.8) * nx, hip[1] + (w_hip + 0.8) * ny + 1), (hip[0] - w_hip * nx, hip[1] - w_hip * ny + 1)]
    c.poly(torso, HOOD)
    pk = at(hip, 180 + lean, 2.5)
    c.line((pk[0] - 2.5 * nx, pk[1] - 2.5 * ny), (pk[0] + 2.5 * nx, pk[1] + 2.5 * ny), HOOD_D)
    c.line((sh[0] - w_sh * nx + 0.5, sh[1] - w_sh * ny), (hip[0] - w_hip * nx + 0.5, hip[1] - w_hip * ny), HOOD_D)
    # drawstrings
    for s in (-1, 1):
        a0 = (sh[0] + s * 1.2 * nx, sh[1] + s * 1.2 * ny)
        c.line(a0, at(a0, lean, 4), STRING)
    # front leg
    (tf, sf) = P['legs'][0]
    kf = at(hip, tf, 6.2)
    ff = at(kf, sf, 6.2)
    c.seg(hip, kf, 1.6, JEANS)
    c.seg(kf, ff, 1.5, JEANS)
    c.seg(ff, (ff[0] + 2.2, ff[1]), 1.1, SHOE)
    c.put(ff[0] + 2, ff[1] + 1, SHOE_D)
    # head: the hood up, the face in shadow, two green lenses
    hx, hy = head
    tilt = P.get('hood_tilt', 0.0)
    hood = [(hx - 5.2, hy + 3.5), (hx - 5.6, hy - 1.5), (hx - 3.8, hy - 5.8), (hx + 0.5, hy - 7.2 + tilt),
            (hx + 4.6, hy - 5.4 + tilt), (hx + 6.0, hy - 1.0), (hx + 5.2, hy + 3.8), (hx + 1.0, hy + 5.0)]
    c.poly(hood, HOOD)
    face = [(hx - 2.0, hy - 2.8), (hx + 4.6, hy - 2.6), (hx + 4.8, hy + 2.6), (hx + 1.2, hy + 4.0), (hx - 1.6, hy + 2.6)]
    c.poly(face, SHADOW)
    c.line((hx - 3.5, hy - 4.6), (hx + 2.5, hy - 6.0), HOOD_L)          # the hood's lit rim
    c.line((hx + 0.4, hy + 3.4), (hx + 3.2, hy + 2.8), SKIN_D)          # a sliver of chin
    lens = P.get('lens', 1.0)
    lc = tuple(int(LENS[i] * lens + SHADOW[i] * (1 - lens)) for i in range(3)) + (255,)
    for ex in (hx + 0.6, hx + 3.4):
        c.put(ex, hy - 0.4, lc)
        c.put(ex + 1, hy - 0.4, lc)
    c.put(hx + 2.4, hy - 0.4, rgb('#2a6a48'))                            # the bridge
    # front arm
    (fu, fo) = P['arms'][0]
    f_el = at(sh, fu, 4.8)
    f_hd = at(f_el, fo, 4.4)
    c.seg(sh, f_el, 1.5, HOOD)
    c.seg(f_el, f_hd, 1.4, HOOD)
    c.put(f_el[0], f_el[1] - 1, HOOD_L)
    c.dot(f_hd[0], f_hd[1], 1.1, SKIN)
    c.outline()
    # the lenses glow over the outline a touch
    fx = FX(c)
    if lens > 0.5:
        for ex in (hx + 1.1, hx + 3.9):
            fx.add(ex, hy - 1.4, LENS, 70 * lens)
    j = dict(hip=hip, sh=sh, head=head, f_hd=f_hd, b_hd=b_hd)
    if P.get('kb') is not None:
        keyboard(c, (f_hd[0] + b_hd[0]) / 2 + 1, max(f_hd[1], b_hd[1]) + 1, P['kb'])
    if effects:
        effects(FX(c), j)
    return c.im


def pose(**k):
    base = dict(hip=(24, 33), lean=0, legs=[(6, 0), (-6, 0)], arms=[(55, 95), (25, 95)], kb=0, lens=1.0, hood_tilt=0.0)
    base.update(k)
    return base


def frames():
    F = {}
    T = math.tau
    # idle: typing, a slight bob, fingers dancing (the arms twitch), keys lighting up
    F['idle'] = [figure(pose(hip=(24, 33 + (0, 0, 1, 1, 0, 0, 1, 1)[i]), lean=3,
                             arms=[(55 + (i % 2) * 6, 95 - (i % 3) * 5), (28 - (i % 2) * 5, 92 + (i % 2) * 6)],
                             kb=i, lens=0.75 + 0.25 * math.sin(i / 8 * T)))
                 for i in range(8)]

    # run: the keyboard folded to a glowing tablet under his arm
    def run_fx(i):
        def fx(e, j):
            b = j['b_hd']
            for y in range(4):
                for x in range(3):
                    e.add(b[0] - 1 + x, b[1] - 3 + y, KEY, 170 if (x + y + i) % 3 else 230)
        return fx
    F['run'] = [figure(pose(hip=(24, 32 + (0, -1, -1, 0, 0, -1, -1, 0)[i]), lean=10,
                            legs=[(40 * math.sin(i / 8 * T), -20 - 15 * max(0, math.sin(i / 8 * T))),
                                  (-40 * math.sin(i / 8 * T), -20 - 15 * max(0, -math.sin(i / 8 * T)))],
                            arms=[(-35 * math.sin(i / 8 * T), 40), (-20, -60)], kb=None), run_fx(i))
                for i in range(8)]

    # attack: flick a data packet (0s and 1s) off the fingertips
    def atk_fx(i):
        def fx(e, j):
            h = j['f_hd']
            if 2 <= i <= 4:
                for k in range(i - 1):
                    x, y = h[0] + 3 + k * 4 + (i - 2) * 3, h[1] - 1 - k
                    for dy in range(3):
                        e.add(x, y + dy, LENS, 255 if (k + dy) % 2 else 200)
                    e.add(x + 1, y, LENS_L, 255)
            if i == 2:
                e.spark(h[0] + 2, h[1], 1)
        return fx
    F['attack'] = [figure(pose(lean=(2, -3, 8, 10, 6, 3)[i], arms=[((40, 70), (10, 40), (85, 90), (95, 92), (80, 88), (60, 90))[i], (25, 90)],
                               kb=None, lens=1.0), atk_fx(i)) for i in range(6)]

    # skill1, overclock: arms spread, the hood vents heat (red-orange glow, sparks), the lenses blaze
    def oc_fx(i):
        def fx(e, j):
            hx, hy = j['head']
            for k in range(3):
                x = hx - 2 + k * 2 + math.sin(i + k) * 1.5
                for s in range(4):
                    e.add(x, hy - 8 - s - (i % 3), (255, 140 - s * 25, 60, 255), 200 - s * 45)
            if i % 2:
                e.spark(hx + 6, hy - 6 + (i % 4), 1)
                e.spark(hx - 5, hy - 4 - (i % 3), 1)
        return fx
    F['skill1'] = [figure(pose(lean=-4, arms=[(100 + (i % 2) * 5, 120), (-80, -100)], kb=i, lens=1.0, hood_tilt=-0.5), oc_fx(i))
                   for i in range(8)]

    # skill2, debug: a scanning lens sweeps over the keyboard, a little bug squashed
    def dbg_fx(i):
        def fx(e, j):
            h = j['f_hd']
            cx, cy = h[0] + 4 + math.sin(i) * 2, h[1] - 4
            for a in range(16):
                t = a / 16 * math.tau
                e.add(cx + math.cos(t) * 2.5, cy + math.sin(t) * 2.5, (255, 220, 120, 255), 230)
            e.add(cx + 2.5, cy + 2.5, (255, 220, 120, 255), 230)
            e.add(cx + 3.5, cy + 3.5, (255, 220, 120, 255), 230)
            if i >= 3:
                e.add(cx, cy, (255, 80, 80, 255), 255 - (i - 3) * 60)
        return fx
    F['skill2'] = [figure(pose(lean=4, arms=[(80, 60), (30, 95)], kb=i, lens=1.0), dbg_fx(i)) for i in range(6)]

    # ult, the AI: an orb of light between his raised hands, rings turning
    def ai_fx(i):
        def fx(e, j):
            ox, oy = (j['f_hd'][0] + j['b_hd'][0]) / 2, min(j['f_hd'][1], j['b_hd'][1]) - 5
            r = 2.5 + (i % 4) * 0.4
            for yy in range(-5, 6):
                for xx in range(-5, 6):
                    d = math.hypot(xx, yy)
                    if d <= r:
                        e.add(ox + xx, oy + yy, WHITE if d < r * 0.5 else (170, 220, 255, 255), 255 * (1 - d / (r + 1)))
            for a in range(20):
                t = a / 20 * math.tau + i * 0.4
                e.add(ox + math.cos(t) * 5, oy + math.sin(t) * 2, (120, 200, 255, 255), 180)
        return fx
    F['ult'] = [figure(pose(lean=-2, arms=[(150, 170), (-150, -170)], kb=None, lens=1.0, hood_tilt=-0.4), ai_fx(i)) for i in range(8)]

    F['hit'] = [figure(pose(lean=-12 - i * 4, arms=[(40, 60), (-40, -20)], kb=None, lens=0.2)) for i in range(2)]
    dead = []
    for i in range(6):
        if i < 3:
            dead.append(figure(pose(hip=(24, 33 + i * 1.5), lean=-20 - i * 22, legs=[(10, 10), (-10, 10)],
                                    arms=[(-60, -40), (-80, -60)], kb=None, lens=0.0)))
        else:
            im = figure(pose(hip=(24, 39), lean=-82, legs=[(80, 90), (70, 85)], arms=[(-100, -100), (-120, -110)], kb=None, lens=0.0))
            if i >= 4:
                im.putalpha(im.getchannel('A').point(lambda a: int(a * (0.7 if i == 4 else 0.4))))
            dead.append(im)
    F['dead'] = dead
    return F


DUR = {'idle': 0.1, 'run': 0.07, 'attack': 0.05, 'skill1': 0.05, 'skill2': 0.06, 'ult': 0.06, 'hit': 0.08, 'dead': 0.1}


def build():
    F = frames()
    sheet = Image.new('RGBA', (W * 8, H * len(F)), (0, 0, 0, 0))
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
    out = Image.new('RGBA', (W * 8, H * len(F)), bg)
    for r, (tag, ims) in enumerate(F.items()):
        for i, im in enumerate(ims):
            out.alpha_composite(im, (i * W, r * H))
    out.resize((out.width * 4, out.height * 4), Image.NEAREST).save(os.path.join(folder, 'coder_body.png'))


if __name__ == '__main__':
    sheet, fanim, F = build()
    if '--preview' in sys.argv:
        preview(F, os.path.join(HERE, 'preview'))
    if '--dry' not in sys.argv:
        sheet.save(os.path.join(OUT_DIR, NAME + '#sheet.png'), optimize=True)
        with open(os.path.join(OUT_DIR, NAME + '#anim.fanim'), 'w', encoding='utf-8') as fh:
            json.dump(fanim, fh, separators=(',', ':'))
    print('sheet', sheet.size)
