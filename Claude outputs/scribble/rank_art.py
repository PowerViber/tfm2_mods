"""Round 72: Scribble's Top 10 badges and rank skins.

Appends to the 'scribble' VFX sheet (mods/tfm2_toon/vfx/scribble#sheet.png + #anim.fanim) and patches the editor's
bundled copy (editor/vfx2.js):

  top1 .. top10        animated Top 10 badges (48x96 frames, the badge at his top right like rank0-6)
  skin0_b / skin0_f    Grandmaster skin "Ruby": back layer (behind him) / front layer (over him)
  skin1_b / skin1_f    Archmage skin "Prism"
  skin2_b / skin2_f    Top 10 skin "Legend"

A champion's body sheet can't be swapped during a match, so a skin is two buff visuals around his own sprite. Skin
frames are 96x96, centred on him like every buff visual (his 40x52 body sits at x 28-67, y 22-73). Everything here is
symmetric, because a buff visual doesn't flip with his facing.

Run from the repo root:  python3 "Claude outputs/scribble/rank_art.py" [--preview DIR]
Re-running is safe: the sheet is cut back to the original 776 rows and these anims are rebuilt.
"""
import base64
import io
import json
import math
import os
import random
import sys

from PIL import Image

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), '..', '..'))
SHEET = os.path.join(ROOT, 'mods', 'tfm2_toon', 'vfx', 'scribble#sheet.png')
FANIM = os.path.join(ROOT, 'mods', 'tfm2_toon', 'vfx', 'scribble#anim.fanim')
BODY = os.path.join(ROOT, 'mods', 'tfm2_toon', 'champions', 'tfm2_toon_scribble#sheet.png')
BODY_ANIM = os.path.join(ROOT, 'mods', 'tfm2_toon', 'champions', 'tfm2_toon_scribble#anim.fanim')
VFX2 = os.path.join(ROOT, 'editor', 'vfx2.js')
ORIGINAL_H = 776
FRAMES = 8
DUR = 0.1
OURS = ('top', 'skin')

OUT = (18, 14, 28, 255)        # outline
WHITE = (255, 255, 255, 255)


def rgba(h, a=255):
    h = h.lstrip('#')
    return (int(h[0:2], 16), int(h[2:4], 16), int(h[4:6], 16), a)


def hsv(h, s=0.85, v=1.0, a=255):
    h = (h % 1.0) * 6
    i = int(h)
    f = h - i
    p, q, t = v * (1 - s), v * (1 - s * f), v * (1 - s * (1 - f))
    r, g, b = [(v, t, p), (q, v, p), (p, v, t), (p, q, v), (t, p, v), (v, p, q)][i % 6]
    return (int(r * 255), int(g * 255), int(b * 255), a)


class Canvas:
    def __init__(self, w, h):
        self.im = Image.new('RGBA', (w, h), (0, 0, 0, 0))
        self.px = self.im.load()
        self.w, self.h = w, h

    def put(self, x, y, c):
        x, y = int(round(x)), int(round(y))
        if 0 <= x < self.w and 0 <= y < self.h and c[3] > 0:
            if c[3] >= 255:
                self.px[x, y] = c
            else:
                base = self.px[x, y]
                a = c[3] / 255
                self.px[x, y] = tuple(int(base[i] * (1 - a) + c[i] * a) for i in range(3)) + (max(base[3], c[3]),)

    def get(self, x, y):
        if 0 <= x < self.w and 0 <= y < self.h:
            return self.px[x, y]
        return (0, 0, 0, 0)

    def rect(self, x0, y0, x1, y1, c):
        for y in range(y0, y1 + 1):
            for x in range(x0, x1 + 1):
                self.put(x, y, c)

    def disc(self, cx, cy, r, c):
        for y in range(int(cy - r - 1), int(cy + r + 2)):
            for x in range(int(cx - r - 1), int(cx + r + 2)):
                if (x - cx) ** 2 + (y - cy) ** 2 <= r * r + 0.5:
                    self.put(x, y, c)

    def poly(self, pts, c):
        ys = [p[1] for p in pts]
        for y in range(int(min(ys)), int(max(ys)) + 1):
            xs = []
            n = len(pts)
            for i in range(n):
                (x0, y0), (x1, y1) = pts[i], pts[(i + 1) % n]
                if (y0 <= y + 0.5 < y1) or (y1 <= y + 0.5 < y0):
                    xs.append(x0 + (y + 0.5 - y0) * (x1 - x0) / (y1 - y0))
            xs.sort()
            for a, b in zip(xs[::2], xs[1::2]):
                for x in range(int(math.ceil(a - 0.5)), int(math.floor(b - 0.5)) + 1):
                    self.put(x, y, c)

    def line(self, x0, y0, x1, y1, c):
        n = int(max(abs(x1 - x0), abs(y1 - y0))) + 1
        for i in range(n + 1):
            t = i / n
            self.put(x0 + (x1 - x0) * t, y0 + (y1 - y0) * t, c)

    def outline(self, c=OUT):
        """1 px outline around every opaque pixel (4-neighbour)."""
        add = []
        for y in range(self.h):
            for x in range(self.w):
                if self.px[x, y][3] == 0 and any(self.get(x + dx, y + dy)[3] > 160 for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1))):
                    add.append((x, y))
        for x, y in add:
            self.px[x, y] = c

    def paste(self, other, x=0, y=0):
        self.im.alpha_composite(other.im, (x, y))
        self.px = self.im.load()


def star4(cv, x, y, c, big=False):
    cv.put(x, y, WHITE if big else c)
    for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
        cv.put(x + dx, y + dy, c)
    if big:
        for dx, dy in ((2, 0), (-2, 0), (0, 2), (0, -2)):
            cv.put(x + dx, y + dy, c[:3] + (150,))


# ------------------------------------------------------------------ Top 10 badges

DIGITS = {
    '0': ['111', '101', '101', '101', '111'], '1': ['010', '110', '010', '010', '111'],
    '2': ['111', '001', '111', '100', '111'], '3': ['111', '001', '111', '001', '111'],
    '4': ['101', '101', '111', '001', '001'], '5': ['111', '100', '111', '001', '111'],
    '6': ['111', '100', '111', '101', '111'], '7': ['111', '001', '010', '010', '010'],
    '8': ['111', '101', '111', '101', '111'], '9': ['111', '101', '111', '001', '111'],
}


def number(cv, text, cx, cy, c, shadow=None):
    w = len(text) * 4 - 1
    x0, y0 = cx - w // 2, cy - 2
    for k, ch in enumerate(text):
        for j, row in enumerate(DIGITS[ch]):
            for i, b in enumerate(row):
                if b == '1':
                    if shadow:
                        cv.put(x0 + k * 4 + i, y0 + j + 1, shadow)
                    cv.put(x0 + k * 4 + i, y0 + j, c)


def flame(cv, cx, base_y, h, outer, inner):
    """A small flame tongue, h px tall, standing on base_y."""
    for j in range(h):
        y = base_y - j
        half = max(0, round((1 - j / h) * 2.2 - 0.3))
        for dx in range(-half, half + 1):
            cv.put(cx + dx, y, inner if (abs(dx) < half and j < h - 2) else outer)


def crown(cv, cx, top, gold, gem, bob=0):
    t = top + bob
    cv.rect(cx - 5, t + 3, cx + 5, t + 5, gold)
    for px in (cx - 5, cx, cx + 5):
        cv.put(px, t, gold)
        cv.put(px, t + 1, gold)
        cv.put(px, t + 2, gold)
    for px in (cx - 3, cx + 3):
        cv.put(px, t + 2, gold)
    cv.put(cx, t + 4, gem)
    cv.put(cx - 3, t + 4, gem)
    cv.put(cx + 3, t + 4, gem)


def top_badge(pos, f):
    """Top 10 badge for position pos (1 = best), frame f. 48x96, badge centred at (40, 26) like the rank badges."""
    cv = Canvas(48, 96)
    cx, cy, r = 40, 26, 7
    if pos == 1:
        disc, rim, num, numsh = rgba('#ffcf3f'), None, rgba('#2a1606'), rgba('#c98a12')
    elif pos == 2:
        disc, rim, num, numsh = rgba('#1c2238'), rgba('#d9e4f5'), rgba('#f2f7ff'), rgba('#5d6c8a')
    elif pos == 3:
        disc, rim, num, numsh = rgba('#2a1a14'), rgba('#e08a3c'), rgba('#ffc58a'), rgba('#7a3f16')
    else:
        disc, rim, num, numsh = rgba('#1b1830'), rgba('#e7b545'), rgba('#ffdf7a'), rgba('#6d5418')
    body = Canvas(48, 96)
    # flames behind the badge for the podium (#1-#3): they flicker
    if pos <= 3:
        fl_o, fl_i = {1: (rgba('#ffb21f'), rgba('#fff3b0')), 2: (rgba('#58c6ff'), rgba('#e6f8ff')), 3: (rgba('#ff6a1f'), rgba('#ffd27a'))}[pos]
        hs = [(4, 6, 5), (5, 4, 6), (6, 5, 4), (5, 6, 5), (4, 5, 6), (6, 4, 5), (5, 5, 4), (4, 6, 6)][f]
        for k, dx in enumerate((-4, 0, 4)):
            flame(body, cx + dx, cy - r + 1, hs[k] + (2 if dx == 0 else 0), fl_o, fl_i)
    body.disc(cx, cy, r, disc)
    if rim:
        for y in range(cy - r - 1, cy + r + 2):
            for x in range(cx - r - 1, cx + r + 2):
                d = math.hypot(x - cx, y - cy)
                if r - 1.0 <= d <= r + 0.4:
                    body.put(x, y, rim)
    else:
        # #1: a rainbow rim that turns
        for y in range(cy - r - 1, cy + r + 2):
            for x in range(cx - r - 1, cx + r + 2):
                d = math.hypot(x - cx, y - cy)
                if r - 1.0 <= d <= r + 0.4:
                    a = math.atan2(y - cy, x - cx) / (2 * math.pi) + f / FRAMES
                    body.put(x, y, hsv(a, 0.7, 1.0))
    if pos == 1:
        crown(body, cx, cy - r - 6, rgba('#ffd24a'), rgba('#ff3b6b'), bob=(0 if f % 4 < 2 else -1))
    body.outline()
    cv.paste(body)
    number(cv, str(pos), cx, cy, num, numsh)
    # the glint: a diagonal light sweeping across the badge (every badge)
    g = -10 + f * 3
    for y in range(cy - r, cy + r + 1):
        for x in range(cx - r, cx + r + 1):
            if (x - cx) ** 2 + (y - cy) ** 2 <= r * r and (x - cx) + (y - cy) in (g, g + 1):
                cv.put(x, y, (255, 255, 255, 120))
    # an orbiting spark from #6 up (two for the podium)
    if pos <= 6:
        for k in range(2 if pos <= 3 else 1):
            a = 2 * math.pi * (f / FRAMES + k / 2)
            star4(cv, cx + round(math.cos(a) * 10), cy + round(math.sin(a) * 10), rgba('#fff1a8'), big=(f % 2 == 0))
    return cv.im


# ------------------------------------------------------------------ skins (96x96, he stands at x 28-67, y 22-73)

CX = 48


def ellipse_ring(cv, cx, cy, rx, ry, color_at, step=1.0):
    n = int(2 * math.pi * max(rx, ry) / step) + 8
    for i in range(n):
        a = 2 * math.pi * i / n
        cv.put(cx + math.cos(a) * rx, cy + math.sin(a) * ry, color_at(a))


def skin_ruby(f):
    """Grandmaster: a ruby sigil under him, ruby wisps rising behind, a floating gold crown, ruby sparks in front."""
    back, front = Canvas(96, 96), Canvas(96, 96)
    ruby, ruby_d, gold = rgba('#e0234f'), rgba('#7d0f2c'), rgba('#ffcf4a')
    # sigil under his feet, gold dashes turning around a ruby ring
    ellipse_ring(back, CX, 77, 19, 4.5, lambda a: gold if int((a / (2 * math.pi)) * 16 + f * 2) % 4 == 0 else ruby)
    ellipse_ring(back, CX, 77, 14, 3, lambda a: ruby_d)
    # wisps rising on both sides
    for side in (-1, 1):
        for k in range(3):
            t = ((f + k * FRAMES / 3) % FRAMES) / FRAMES
            y = 70 - t * 44
            x = CX + side * (23 + 2 * math.sin(t * 6 + k))
            a = int(255 * min(1.0, (1 - t) * 1.4))
            for dx in (-1, 0, 1):
                back.put(x + dx, y + 1, ruby_d[:3] + (a,))
                back.put(x + dx, y, ruby[:3] + (a,))
            back.put(x, y - 1, rgba('#ff8fa6', a))
            back.put(x, y - 2, rgba('#ffd0da', a // 2))
    # the crown, floating behind and above his hair
    crown(back, CX, 11, gold, ruby, bob=(0 if f % 4 < 2 else -1))
    crown_layer = Canvas(96, 96)
    crown(crown_layer, CX, 11, gold, ruby, bob=(0 if f % 4 < 2 else -1))
    crown_layer.outline()
    back.paste(crown_layer)
    # sparks twinkling around his outline
    spots = [(25, 34), (71, 30), (23, 56), (73, 60), (30, 18), (66, 20)]
    for k, (x, y) in enumerate(spots):
        phase = (f + k * 3) % FRAMES
        if phase < 3:
            star4(front, x, y, rgba('#ff5c7f') if k % 2 else gold, big=(phase == 1))
    return back.im, front.im


def page(cv, x, y, color=None):
    c = color or rgba('#fbfbff')
    cv.rect(x - 2, y - 2, x + 2, y + 2, c)
    cv.put(x - 1, y - 1, rgba('#8fb6ff'))
    cv.put(x, y - 1, rgba('#8fb6ff'))
    cv.put(x - 1, y + 1, rgba('#8fb6ff'))
    cv.put(x + 1, y + 1, rgba('#8fb6ff'))


def skin_prism(f):
    """Archmage: a turning rainbow halo, pages orbiting him (behind on the far side, in front on the near side) and
    rainbow sparkles."""
    back, front = Canvas(96, 96), Canvas(96, 96)
    # the halo: a ring behind his head and shoulders, its rainbow turning
    for y in range(8, 64):
        for x in range(20, 77):
            d = math.hypot(x - CX, y - 34)
            if 19.2 <= d <= 21.2:
                a = math.atan2(y - 34, x - CX) / (2 * math.pi) + f / FRAMES
                back.put(x, y, hsv(a, 0.65, 1.0, 235))
            elif 18.2 <= d < 19.2:
                a = math.atan2(y - 34, x - CX) / (2 * math.pi) + f / FRAMES
                back.put(x, y, hsv(a, 0.5, 1.0, 90))
    # orbiting pages: an ellipse around his waist; the far half behind him, the near half in front
    for k in range(3):
        a = 2 * math.pi * (f / FRAMES + k / 3)
        x, y = CX + math.cos(a) * 30, 52 + math.sin(a) * 7
        layer = front if math.sin(a) > 0 else back
        pg = Canvas(96, 96)
        page(pg, round(x), round(y))
        pg.outline()
        layer.paste(pg)
    # sparkles
    for k in range(5):
        phase = (f + k * 2) % FRAMES
        if phase < 2:
            ang = 2 * math.pi * k / 5 + 0.4
            star4(front, CX + round(math.cos(ang) * 27), 40 + round(math.sin(ang) * 26), hsv(k / 5 + f / FRAMES, 0.6, 1.0), big=phase == 0)
    return back.im, front.im


def wing(cv, f, side):
    """One ink wing with gold trim, rooted at his shoulder blade."""
    flap = [0, -1, -2, -3, -2, -1, 0, 1][f]
    rx, ry = CX + side * 7, 40
    tips = [(43, 14 + flap), (45, 25 + flap), (43, 37 + flap // 2), (37, 48)]
    ink, ink_l, trim, trim_d = rgba('#17131f'), rgba('#2b2440'), rgba('#f2c14e'), rgba('#9c7420')
    layer = Canvas(96, 96)
    for k, (tx, ty) in enumerate(tips):
        tip = (CX + side * tx, ty)
        w = 5.2 - k * 0.6
        # a long feather: root -> tip, widest near the middle
        ang = math.atan2(tip[1] - ry, tip[0] - rx)
        nx, ny = -math.sin(ang), math.cos(ang)
        mid = ((rx + tip[0]) / 2, (ry + tip[1]) / 2)
        pts = [(rx, ry), (mid[0] + nx * w, mid[1] + ny * w), tip, (mid[0] - nx * w, mid[1] - ny * w)]
        layer.poly(pts, ink if k % 2 == 0 else ink_l)
    # covert feathers near the root
    layer.poly([(rx, ry - 6), (CX + side * 26, 22 + flap), (CX + side * 30, 34 + flap // 2), (rx, ry + 4)], ink_l)
    # gold trim along the top edge of each column
    for x in range(96):
        for y in range(96):
            if layer.get(x, y)[3] and not layer.get(x, y - 1)[3]:
                layer.put(x, y, trim)
            elif layer.get(x, y)[3] and not layer.get(x + side, y)[3] and (x - CX) * side > 18:
                layer.put(x, y, trim_d)
    layer.outline()
    cv.paste(layer)


def bolt(cv, x0, y0, dx, rnd, length, core, glow):
    x, y = x0, y0
    for _ in range(length):
        nx = x + dx * rnd.choice((1, 2, 2, 3))
        ny = y + rnd.choice((-3, -2, -1, 1, 2, 3))
        cv.line(x, y, nx, ny, glow)
        cv.put(x, y, core)
        x, y = nx, ny


def skin_legend(f):
    """Top 10: ink wings with gold trim, a gold halo with a star, a turning gold sigil under him, gold lightning."""
    back, front = Canvas(96, 96), Canvas(96, 96)
    gold, gold_d, pale = rgba('#f2c14e'), rgba('#9c7420'), rgba('#fff2b3')
    # sigil: two rings turning opposite ways, with rune ticks
    ellipse_ring(back, CX, 77, 25, 6, lambda a: gold if int((a / (2 * math.pi)) * 24 + f * 3) % 3 else gold_d)
    ellipse_ring(back, CX, 77, 17, 4, lambda a: pale if int((a / (2 * math.pi)) * 12 - f * 2) % 4 == 0 else gold_d)
    for k in range(8):
        a = 2 * math.pi * (k / 8 + f / (FRAMES * 4))
        x, y = CX + math.cos(a) * 21, 77 + math.sin(a) * 5
        back.put(x, y, pale)
        back.put(x, y - 1, gold)
    wing(back, f, 1)
    wing(back, f, -1)
    # halo with a star on top
    for y in range(8, 36):
        for x in range(32, 65):
            d = math.hypot((x - CX) / 1.0, (y - 21) / 0.45)
            if 10.3 <= d <= 11.6:
                back.put(x, y, gold if (x + f) % 5 else pale)
    star4(back, CX, 10, gold, big=(f % 2 == 0))
    # gold lightning crackling at his sides (in front)
    rnd = random.Random(7200 + f)
    if f % 2 == 0 or f == 3:
        bolt(front, CX - 19, 36 + rnd.randint(-6, 8), -1, rnd, 5, WHITE, gold)
    if f % 2 == 1 or f == 6:
        bolt(front, CX + 19, 36 + rnd.randint(-6, 8), 1, rnd, 5, WHITE, gold)
    # embers rising
    for k in range(4):
        t = ((f + k * 2) % FRAMES) / FRAMES
        x = CX + (-1) ** k * (12 + 4 * k)
        front.put(x, 74 - t * 50, gold[:3] + (int(255 * (1 - t)),))
    return back.im, front.im


SKINS = [skin_ruby, skin_prism, skin_legend]


# ------------------------------------------------------------------ sheet packing

def build():
    sheet = Image.open(SHEET).convert('RGBA').crop((0, 0, 2048, ORIGINAL_H))
    with open(FANIM, encoding='utf-8-sig') as fh:
        fanim = json.load(fh)
    anims = {k: v for k, v in fanim['anims'].items() if not k.startswith(OURS)}
    new = {}
    for pos in range(1, 11):
        new[f'top{pos}'] = [top_badge(pos, f) for f in range(FRAMES)]
    for k, fn in enumerate(SKINS):
        frames = [fn(f) for f in range(FRAMES)]
        new[f'skin{k}_b'] = [b for b, _ in frames]
        new[f'skin{k}_f'] = [fr for _, fr in frames]
    # pack below the old content, row by row, 1 px apart
    x, y, row_h = 0, ORIGINAL_H + 1, 0
    placed = []
    for name, frames in new.items():
        rects = []
        for im in frames:
            if x + im.width > 2048:
                x, y, row_h = 0, y + row_h + 1, 0
            rects.append((x, y, im))
            x += im.width + 1
            row_h = max(row_h, im.height)
        placed.append((name, rects))
    height = y + row_h
    out = Image.new('RGBA', (2048, height), (0, 0, 0, 0))
    out.paste(sheet, (0, 0))
    for name, rects in placed:
        for (fx, fy, im) in rects:
            out.paste(im, (fx, fy))
        anims[name] = {'frames': [{'duration': DUR, 'data': {'x': fx, 'y': fy, 'w': im.width, 'h': im.height}} for fx, fy, im in rects]}
    return out, {'anims': anims}, new


def write(out, fanim):
    out.save(SHEET, optimize=True)
    with open(FANIM, 'w', encoding='utf-8') as fh:
        json.dump(fanim, fh, separators=(',', ':'))
    # the editor's bundled copy
    s = open(VFX2, encoding='utf-8').read()
    pre = 'window.TFM2_VFX = Object.assign(window.TFM2_VFX || {}, '
    i = s.index(pre) + len(pre)
    obj, end = json.JSONDecoder().raw_decode(s[i:])
    buf = io.BytesIO()
    out.save(buf, format='PNG', optimize=True)
    obj['scribble'] = {'png': base64.b64encode(buf.getvalue()).decode('ascii'), 'fanim': fanim}
    with open(VFX2, 'w', encoding='utf-8') as fh:
        fh.write(s[:i] + json.dumps(obj, separators=(', ', ': ')) + s[i + end:])


def preview(new, folder):
    """Each skin over his idle and run frames, and the ten badges, at 4x."""
    os.makedirs(folder, exist_ok=True)
    body = Image.open(BODY).convert('RGBA')
    with open(BODY_ANIM, encoding='utf-8-sig') as fh:
        ba = json.load(fh)['anims']
    poses = ba['idle']['frames'][:4] + ba['run']['frames'][:4]
    bg = (54, 74, 60, 255)
    rows = []
    for k in range(3):
        row = Image.new('RGBA', (97 * FRAMES, 96), bg)
        for f in range(FRAMES):
            cell = Image.new('RGBA', (96, 96), (0, 0, 0, 0))
            cell.alpha_composite(new[f'skin{k}_b'][f])
            d = poses[f]['data']
            cell.alpha_composite(body.crop((d['x'], d['y'], d['x'] + d['w'], d['y'] + d['h'])), (48 - d['w'] // 2, 48 - d['h'] // 2))
            cell.alpha_composite(new[f'skin{k}_f'][f])
            row.alpha_composite(cell, (f * 97, 0))
        rows.append(row)
    sheet = Image.new('RGBA', (97 * FRAMES, 97 * 3), bg)
    for k, r in enumerate(rows):
        sheet.alpha_composite(r, (0, k * 97))
    sheet.resize((sheet.width * 3, sheet.height * 3), Image.NEAREST).save(os.path.join(folder, 'skins.png'))
    badges = Image.new('RGBA', (49 * FRAMES, 40 * 10), bg)
    for p in range(1, 11):
        for f in range(FRAMES):
            badges.alpha_composite(new[f'top{p}'][f].crop((0, 6, 48, 46)), (f * 49, (p - 1) * 40))
    badges.resize((badges.width * 3, badges.height * 3), Image.NEAREST).save(os.path.join(folder, 'top_badges.png'))


if __name__ == '__main__':
    out, fanim, new = build()
    if '--preview' in sys.argv:
        preview(new, sys.argv[sys.argv.index('--preview') + 1])
    if '--dry' not in sys.argv:
        write(out, fanim)
    print(f'sheet {out.size[0]}x{out.size[1]}, {len(fanim["anims"])} anims')
