"""Pixel art for the equation kit: labels, live values, the charge bar, reserve icons, projectiles,
impacts and zones for the eighteen equations. About 940 game units per pixel (a 12000 zone is a
13 px radius). Every lifetime here matches the tick lifetime the native core asks for."""
import json, math
from pathlib import Path
from PIL import Image, ImageDraw, ImageFont

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
EQ = json.loads((HERE / 'equations.json').read_text(encoding='utf-8'))
SKILLS = EQ['skills']
KEYS = [s['key'] for s in SKILLS]
FONT_PATH = 'C:/Windows/Fonts/consola.ttf' if Path('C:/Windows/Fonts/consola.ttf').exists() else '/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf'
FONT = ImageFont.truetype(FONT_PATH, 9)
BOLD = ImageFont.truetype(FONT_PATH.replace('DejaVuSansMono.ttf', 'DejaVuSansMono-Bold.ttf') if 'DejaVu' in FONT_PATH else FONT_PATH, 10)
SCI = [(99, 191, 255), (255, 201, 107), (99, 225, 189)]          # Einstein, Newton, Curie
GOLD, WHITE, INK, RED = (255, 214, 120), (240, 248, 255), (12, 22, 36), (226, 52, 48)
UNITS_PER_PX = 940
# The projectile skills (core: Shape::Line / Bolt / Planet / Cradle) and the zones.
PROJECTILES = ['prism', 'orbit', 'cradle', 'calculus', 'alpha', 'beta', 'gamma', 'halflife', 'chain', 'emc2', 'photo']
ZONES = ['radium', 'timefield', 'ring', 'well']


def a(c, alpha):
    return tuple(c[:3]) + (int(alpha),)


def canvas(w, h):
    return Image.new('RGBA', (w, h))


def text_outlined(d, xy, text, font, fill, outline=INK):
    x, y = xy
    for dx in (-1, 0, 1):
        for dy in (-1, 0, 1):
            if dx or dy:
                d.text((x + dx, y + dy), text, font=font, fill=outline)
    d.text((x, y), text, font=font, fill=fill)


def centered(d, cx, y, text, font, fill):
    w = d.textlength(text, font=font)
    text_outlined(d, (round(cx - w / 2), y), text, font, fill)


# --------------------------------------------------------------------- icons (9x9-ish glyphs)

def icon(d, key, x, y, s=1.0, f=0):
    """The skill's symbol centred at (x, y); `s` scales it, `f` animates small details."""
    k = KEYS.index(key)
    c = SCI[SKILLS[k]['science']]
    r = 4 * s
    if key == 'apple':
        d.ellipse((x - r, y - r + 1, x + r, y + r + 1), fill=RED, outline=(138, 24, 30))
        d.line((x, y - r, x + 1, y - r - 2), fill=(90, 60, 30))
        d.ellipse((x + 1, y - r - 2, x + 3, y - r), fill=(86, 180, 70))
    elif key == 'prism':
        d.polygon([(x, y - r), (x + r, y + r), (x - r, y + r)], fill=a(WHITE, 200), outline=c)
        for i, col in enumerate([(255, 80, 80), (90, 230, 120), (90, 140, 255)]):
            d.line((x + r, y + i - 1, x + r + 3 * s, y + i * 2 - 2), fill=col)
    elif key == 'orbit':
        d.ellipse((x - r, y - r * .55, x + r, y + r * .55), outline=c)
        d.ellipse((x - 2 * s, y - 2 * s, x + 2 * s, y + 2 * s), fill=(214, 160, 90))
        ang = f * math.pi / 4
        d.point((x + r * math.cos(ang), y + r * .55 * math.sin(ang)), fill=WHITE)
    elif key == 'cradle':
        d.line((x - r, y - r, x + r, y - r), fill=c)
        for i in (-1, 0, 1):
            bx = x + i * r * .7
            d.line((bx, y - r, bx, y + 1), fill=a(WHITE, 160))
            d.ellipse((bx - 1.6 * s, y + 1 - 1.6 * s, bx + 1.6 * s, y + 1 + 1.6 * s), fill=(200, 200, 210))
    elif key == 'thirdlaw':
        d.line((x - r, y - 1, x - 1, y - 1), fill=c, width=2)
        d.polygon([(x - r - 2, y - 1), (x - r + 1, y - 3), (x - r + 1, y + 1)], fill=c)
        d.line((x + 1, y + 2, x + r, y + 2), fill=WHITE, width=2)
        d.polygon([(x + r + 2, y + 2), (x + r - 1, y), (x + r - 1, y + 4)], fill=WHITE)
    elif key == 'calculus':
        text_outlined(d, (x - 4 * s, y - 6 * s), '∂', FONT, c)
    elif key == 'radium':
        d.ellipse((x - r - 1, y - r - 1, x + r + 1, y + r + 1), fill=a((118, 255, 172), 70))
        d.ellipse((x - 2 * s, y - 2 * s, x + 2 * s, y + 2 * s), fill=(118, 255, 172), outline=WHITE)
    elif key == 'alpha':
        d.ellipse((x - r, y - r, x + r, y + r), fill=(255, 120, 60), outline=WHITE)
        d.ellipse((x - 1, y - 1, x + 1, y + 1), fill=WHITE)
    elif key == 'beta':
        d.line([(x - r + i, y + math.sin(i * 1.2 + f) * 2) for i in range(int(2 * r) + 1)], fill=(110, 170, 255), width=2)
    elif key == 'gamma':
        d.line((x - r, y, x + r, y), fill=(200, 140, 255), width=2)
        d.line((x - r, y - 2, x + r, y - 2), fill=a((200, 140, 255), 120))
    elif key == 'halflife':
        for i, rr in enumerate([3, 2, 1.2, .7]):
            px = x - r + i * 2.6 * s
            d.ellipse((px - rr, y - rr, px + rr, y + rr), fill=a((118, 255, 172), 255 - i * 50))
    elif key == 'chain':
        pts = [(x - r, y + 2), (x - 1, y - 2), (x + 1, y + 1), (x + r, y - 3)]
        d.line(pts, fill=(255, 240, 140), width=1)
        for p in pts:
            d.point(p, fill=WHITE)
    elif key == 'timefield':
        d.ellipse((x - r, y - r, x + r, y + r), fill=a(INK, 160), outline=c)
        ang = f * math.pi / 4 - math.pi / 2
        d.line((x, y, x + r * .8 * math.cos(ang), y + r * .8 * math.sin(ang)), fill=WHITE)
        d.line((x, y, x, y - r * .5), fill=c)
    elif key == 'emc2':
        text_outlined(d, (x - 3 * s, y - 6 * s), 'E', BOLD, (255, 236, 120))
    elif key == 'ring':
        d.ellipse((x - r, y - r, x + r, y + r), outline=(180, 220, 255), width=2)
        d.ellipse((x - 1, y - 1, x + 1, y + 1), fill=(255, 220, 140))
    elif key == 'twin':
        d.polygon([(x - r * .7, y - r), (x + r * .7, y - r), (x, y), (x + r * .7, y + r), (x - r * .7, y + r), (x, y)], outline=c, fill=a(c, 70))
    elif key == 'well':
        for i, rr in enumerate([r, r * .65, r * .3]):
            d.ellipse((x - rr, y - rr * .6, x + rr, y + rr * .6), outline=a(c, 230 - i * 40))
        d.ellipse((x - 1, y - 1, x + 1, y + 1), fill=INK)
    elif key == 'photo':
        d.line([(x - r, y - 1), (x - 1, y - 2), (x, y + 1), (x + r, y)], fill=(255, 250, 170), width=2)
        d.point((x + r, y), fill=WHITE)


# --------------------------------------------------------------------- labels and numbers

# Labels are point effects drawn above the head (the adapter adds the height), so each canvas
# is centred on its own text and stays tight.
LABEL = (176, 16)
LABEL_Y = 2


def label(text, color, f, frames, banner=False):
    im = canvas(*LABEL)
    d = ImageDraw.Draw(im)
    fade = min(1, (f + 1) / 2) * (1 if f < frames - 1 else .55)
    if banner:
        w = d.textlength(text, font=BOLD)
        d.rounded_rectangle((LABEL[0] / 2 - w / 2 - 4, LABEL_Y - 2, LABEL[0] / 2 + w / 2 + 4, LABEL_Y + 11), 3, fill=a(INK, 200 * fade), outline=a(color, 255 * fade))
        centered(d, LABEL[0] / 2, LABEL_Y - 1, text, BOLD, a(color, 255 * fade))
    else:
        centered(d, LABEL[0] / 2, LABEL_Y, text, FONT, a(color, 255 * fade))
    return im


DIGIT = (48, 14)


def digit(pos, ch, f, frames):
    """One digit of the live value, `= 1234` read left to right around the anchor."""
    im = canvas(*DIGIT)
    d = ImageDraw.Draw(im)
    fade = 1 if f < frames - 1 else .55
    x = DIGIT[0] / 2 - 9 + pos * 6
    if pos == 0:
        text_outlined(d, (x - 9, 1), '=', FONT, a(WHITE, 220 * fade))
    text_outlined(d, (x, 1), ch, FONT, a(GOLD, 255 * fade))
    return im


# --------------------------------------------------------------------- HUD

PIP = 7


def hud(ready, reserved, progress):
    """Five pips: ready (bright), reserved (gold ring), recovering (dark; the next one fills by quarters)."""
    im = canvas(5 * PIP + 2, PIP + 2)
    d = ImageDraw.Draw(im)
    for i in range(5):
        x = 1 + i * PIP
        box = (x, 1, x + PIP - 2, PIP - 1)
        if i < ready:
            d.ellipse(box, fill=(150, 220, 255), outline=WHITE)
        elif i < ready + reserved:
            d.ellipse(box, fill=a(GOLD, 120), outline=GOLD)
            d.point((x + 2, 3), fill=WHITE)
        else:
            d.ellipse(box, fill=(20, 32, 48), outline=(70, 90, 110))
            if i == ready + reserved and progress > 0:
                h = (PIP - 3) * progress / 4
                d.rectangle((x + 1, PIP - 2 - h, x + PIP - 3, PIP - 2), fill=(90, 150, 200))
    return im


RESERVE = (48, 14)


def reserve_icon(slot, key, f):
    """A reserved trait above the head: slot 0..3 left to right."""
    im = canvas(*RESERVE)
    d = ImageDraw.Draw(im)
    x = RESERVE[0] / 2 - 15 + slot * 10
    y = 7
    c = SCI[SKILLS[KEYS.index(key)]['science']]
    d.rounded_rectangle((x - 5, y - 5, x + 5, y + 5), 2, fill=a(INK, 190), outline=a(c, 230))
    icon(d, key, x, y, .85, f)
    return im


# --------------------------------------------------------------------- moving art

PK = 48


def projectile(key, f):
    """Drawn heading east; rotated for the other seven headings."""
    im = canvas(PK, PK)
    d = ImageDraw.Draw(im)
    cx, cy = PK / 2, PK / 2
    c = SCI[SKILLS[KEYS.index(key)]['science']]
    trail = 10 + f * 2
    if key in ('prism', 'photo', 'gamma', 'beta', 'alpha', 'emc2', 'calculus', 'halflife', 'chain'):
        col = {'prism': WHITE, 'photo': (255, 250, 170), 'gamma': (200, 140, 255), 'beta': (110, 170, 255), 'alpha': (255, 120, 60),
               'emc2': (255, 236, 120), 'calculus': c, 'halflife': (118, 255, 172), 'chain': (255, 240, 140)}[key]
        width = {'alpha': 4, 'gamma': 2, 'photo': 2}.get(key, 3)
        d.line((cx - trail, cy, cx + 4, cy), fill=a(col, 110), width=width + 2)
        d.line((cx - trail * .6, cy, cx + 5, cy), fill=col, width=width)
        if key == 'prism':
            for i, rgb in enumerate([(255, 80, 80), (90, 230, 120), (90, 140, 255)]):
                d.line((cx - trail, cy - 2 + i * 2, cx - 2, cy - 2 + i * 2), fill=a(rgb, 160))
    icon(d, key, cx + 4, cy, 1.0, f)
    return im


def rotated(im, h):
    return im.rotate(45 * h, resample=Image.Resampling.NEAREST)


def split_art(f):
    im = canvas(PK, PK)
    d = ImageDraw.Draw(im)
    for i, rgb in enumerate([(255, 80, 80), (90, 230, 120), (90, 140, 255)]):
        ang = (i - 1) * .35
        L = 10 + f * 6
        d.line((PK / 2, PK / 2, PK / 2 + L * math.cos(ang), PK / 2 + L * math.sin(ang)), fill=rgb, width=2)
    return im


def jump_art(f):
    im = canvas(PK, PK)
    d = ImageDraw.Draw(im)
    pts = [(PK / 2 - 14, PK / 2), (PK / 2 - 6, PK / 2 - 4 + f * 2), (PK / 2 + 2, PK / 2 + 3 - f * 2), (PK / 2 + 14, PK / 2)]
    d.line(pts, fill=(255, 240, 140), width=2)
    d.line(pts, fill=WHITE, width=1)
    return im


IMP = 64


def impact(key, f, n):
    im = canvas(IMP, IMP)
    d = ImageDraw.Draw(im)
    c = SCI[SKILLS[KEYS.index(key)]['science']]
    t = f / max(1, n - 1)
    r = 6 + 14 * t
    alpha = 255 * (1 - t * .8)
    d.ellipse((IMP / 2 - r, IMP / 2 - r, IMP / 2 + r, IMP / 2 + r), outline=a(c, alpha), width=2)
    for i in range(6):
        ang = i * math.pi / 3 + t
        d.line((IMP / 2 + r * .5 * math.cos(ang), IMP / 2 + r * .5 * math.sin(ang), IMP / 2 + r * math.cos(ang), IMP / 2 + r * math.sin(ang)), fill=a(WHITE, alpha))
    icon(d, key, IMP / 2, IMP / 2, 1.3 - .4 * t, f)
    return im


def zone(key, f):
    """The zone at its radius (units -> px), an 8-frame loop."""
    s = SKILLS[KEYS.index(key)]
    r = s['radius'] / UNITS_PER_PX
    size = int(2 * r + 12)
    im = canvas(size, size)
    d = ImageDraw.Draw(im)
    cx = cy = size / 2
    c = SCI[s['science']]
    ph = f * math.pi / 4
    if key == 'radium':
        d.ellipse((cx - r, cy - r, cx + r, cy + r), fill=a((118, 255, 172), 40 + 15 * math.sin(ph)), outline=a((118, 255, 172), 200))
        for i in range(6):
            ang = i * math.pi / 3 + ph * .3
            rr = r * (.35 + .5 * ((i * 37 + f) % 8) / 8)
            d.point((cx + rr * math.cos(ang), cy + rr * math.sin(ang)), fill=WHITE)
        icon(d, 'radium', cx, cy, 1.1, f)
    elif key == 'timefield':
        d.ellipse((cx - r, cy - r, cx + r, cy + r), fill=a(c, 35), outline=a(c, 210))
        for i in range(12):
            ang = i * math.pi / 6
            d.line((cx + (r - 3) * math.cos(ang), cy + (r - 3) * math.sin(ang), cx + r * math.cos(ang), cy + r * math.sin(ang)), fill=a(WHITE, 200))
        d.line((cx, cy, cx + r * .7 * math.cos(ph - math.pi / 2), cy + r * .7 * math.sin(ph - math.pi / 2)), fill=WHITE, width=2)
        d.line((cx, cy, cx + r * .4 * math.cos(ph / 12 - math.pi / 2), cy + r * .4 * math.sin(ph / 12 - math.pi / 2)), fill=c, width=2)
    elif key == 'ring':
        d.ellipse((cx - r, cy - r, cx + r, cy + r), outline=a((180, 220, 255), 230), width=2)
        d.ellipse((cx - r * .55, cy - r * .55, cx + r * .55, cy + r * .55), outline=a((180, 220, 255), 120))
        for i in range(4):
            ang = i * math.pi / 2 + ph
            d.arc((cx - r * .8, cy - r * .8, cx + r * .8, cy + r * .8), math.degrees(ang), math.degrees(ang) + 30, fill=a(WHITE, 200), width=2)
        d.ellipse((cx - 2, cy - 2, cx + 2, cy + 2), fill=(255, 220, 140))
    elif key == 'well':
        for i in range(5):
            rr = r * (1 - ((i / 5 + f / 40) % 1))
            d.ellipse((cx - rr, cy - rr * .62, cx + rr, cy + rr * .62), outline=a(c, 70 + 150 * rr / r))
        for i in range(8):
            ang = i * math.pi / 4 + ph * .2
            d.line((cx + r * math.cos(ang), cy + r * .62 * math.sin(ang), cx + r * .25 * math.cos(ang + .4), cy + r * .2 * math.sin(ang + .4)), fill=a(c, 120))
        d.ellipse((cx - 3, cy - 2, cx + 3, cy + 2), fill=INK, outline=c)
    return im


def puddle(f):
    r = 7000 / UNITS_PER_PX
    size = int(2 * r + 6)
    im = canvas(size, size)
    d = ImageDraw.Draw(im)
    cx = cy = size / 2
    d.ellipse((cx - r, cy - r * .7, cx + r, cy + r * .7), fill=a((118, 255, 172), 50 + 20 * math.sin(f * math.pi / 4)), outline=a((118, 255, 172), 160))
    d.point((cx + (f % 3) - 1, cy), fill=WHITE)
    return im


def drop(f, n):
    """Apple falling: shadow grows on the ground while the apple descends."""
    im = canvas(48, 64)
    d = ImageDraw.Draw(im)
    t = f / max(1, n - 1)
    sr = 3 + 6 * t
    d.ellipse((24 - sr, 52 - sr * .45, 24 + sr, 52 + sr * .45), fill=a(INK, 120 + 80 * t), outline=a(SCI[1], 160))
    icon(d, 'apple', 24, 8 + 40 * t * t, 1.0, f)
    return im


def small(kind, f, n):
    im = canvas(64, 64)
    d = ImageDraw.Draw(im)
    t = f / max(1, n - 1)
    cx = cy = 32
    if kind == 'insight':
        for i in range(4):
            ang = i * math.pi / 2 + t
            r = 8 + 10 * t
            d.line((cx + 4 * math.cos(ang), cy - 22 + 4 * math.sin(ang), cx + r * math.cos(ang), cy - 22 + r * math.sin(ang)), fill=a(GOLD, 255 * (1 - t * .7)), width=2)
        centered(d, cx, cy - 34, '!', BOLD, a(GOLD, 255 * (1 - t * .5)))
    elif kind == 'dot':
        for i in range(3):
            y = cy - 6 - ((t * 20 + i * 7) % 20)
            d.ellipse((cx - 6 + i * 6 - 1.5, y - 1.5, cx - 6 + i * 6 + 1.5, y + 1.5), fill=a((118, 255, 172), 220 - i * 50))
    elif kind == 'lawshield':
        d.polygon([(cx, cy - 22), (cx + 16, cy - 15), (cx + 13, cy + 8), (cx, cy + 18), (cx - 13, cy + 8), (cx - 16, cy - 15)], fill=a(SCI[1], 40 + 30 * math.sin(t * math.pi)), outline=a(SCI[1], 220))
        icon(d, 'thirdlaw', cx, cy - 4, 1.2, f)
    elif kind == 'twin':
        r = 6 + 10 * t
        d.ellipse((cx - r, cy - r, cx + r, cy + r), outline=a(SCI[0], 255 * (1 - t)), width=2)
        icon(d, 'twin', cx, cy, 1.3, f)
    elif kind == 'ultwarn':
        r = 26 - 16 * t
        d.ellipse((cx - r, cy - r, cx + r, cy + r), outline=a(WHITE, 140 + 100 * t), width=2)
        for i, c in enumerate(SCI):
            ang = i * 2 * math.pi / 3 + t * 3
            d.ellipse((cx + r * math.cos(ang) - 3, cy + r * math.sin(ang) - 3, cx + r * math.cos(ang) + 3, cy + r * math.sin(ang) + 3), fill=c)
    elif kind == 'ult':
        for i, c in enumerate(SCI):
            r = 8 + 22 * t
            ang = i * 2 * math.pi / 3 + t * 2
            d.ellipse((cx - r, cy - r, cx + r, cy + r), outline=a(c, 255 * (1 - t)), width=2)
            d.line((cx, cy, cx + r * math.cos(ang), cy + r * math.sin(ang)), fill=a(c, 220))
        centered(d, cx, cy - 6, 'ToE', BOLD, a(WHITE, 255 * (1 - t * .6)))
    return im


# --------------------------------------------------------------------- sheets

def sequences():
    """tag -> [(image, seconds)], grouped by sheet."""
    sheets = {'science_eq_labels': {}, 'science_eq_hud': {}, 'science_eq_fx': {}}
    L, H, F = sheets['science_eq_labels'], sheets['science_eq_hud'], sheets['science_eq_fx']
    # Labels: 60 ticks for an equation and its value; 90 for a signature or Unification.
    for i, s in enumerate(SKILLS):
        L[f'eq{i}'] = [(label(s['equation'], SCI[s['science']], f, 4), 15 / 60) for f in range(4)]
    for i, g in enumerate(EQ['signatures']):
        L[f'sig{i}'] = [(label(g['name'], GOLD, f, 6, banner=True), 15 / 60) for f in range(6)]
    for i, u in enumerate(EQ['unification']):
        L[f'uni{i}'] = [(label(u['name'], WHITE, f, 6, banner=True), 15 / 60) for f in range(6)]
    for pos in range(4):
        for ch in '0123456789':
            L[f'n{pos}_{ch}'] = [(digit(pos, ch, f, 4), 15 / 60) for f in range(4)]
    # HUD: refreshed every 8 ticks; reserve icons every 30.
    for ready in range(6):
        for reserved in range(min(4, 5 - ready) + 1):
            for p in range(4):
                H[f'ch_{ready}_{reserved}_{p}'] = [(hud(ready, reserved, p), 8 / 60)]
    for slot in range(4):
        for k in KEYS:
            H[f'rv{slot}_{KEYS.index(k)}'] = [(reserve_icon(slot, k, 0), 8 / 60)]
    # Moving art: 2 frames over 4 ticks, redrawn every 4 ticks as it travels.
    for k in PROJECTILES:
        base = [projectile(k, f) for f in range(2)]
        for h in range(8):
            F[f'pk{KEYS.index(k)}_a{h}'] = [(rotated(im, h), 2 / 60) for im in base]
    for h in range(8):
        F[f'split_a{h}'] = [(rotated(split_art(f), h), 4 / 60) for f in range(3)]
        F[f'jump_a{h}'] = [(rotated(jump_art(f), h), 4 / 60) for f in range(2)]
    for i, k in enumerate(KEYS):
        F[f'im{i}'] = [(impact(k, f, 6), 4 / 60) for f in range(6)]
    for k in ZONES:
        F[f'zn{KEYS.index(k)}'] = [(zone(k, f), 6 / 60) for f in range(8)]
    F['puddle'] = [(puddle(f), 6 / 60) for f in range(8)]
    F['drop'] = [(drop(f, 6), 6 / 60) for f in range(6)]
    for kind, n, ticks in [('insight', 4, 20), ('dot', 6, 60), ('lawshield', 6, 90), ('twin', 6, 30), ('ultwarn', 6, 36), ('ult', 8, 48)]:
        F[kind] = [(small(kind, f, n), ticks / n / 60) for f in range(n)]
    return sheets


def generate(atlas, mod):
    for name, entries in sequences().items():
        w, h = max_size(entries)
        # The atlas crops every image around the centre of a shared w x h canvas.
        padded = {tag: [(pad(im, w, h), sec) for im, sec in seq] for tag, seq in entries.items()}
        atlas(mod / 'vfx', name, padded, w, h, cols=16, trim=True, palette=True)
    return sum(len(e) for e in sequences().values())


def pad(im, w, h):
    out = canvas(w, h)
    out.alpha_composite(im, ((w - im.width) // 2, (h - im.height) // 2))
    return out


def max_size(entries):
    w = max(im.width for seq in entries.values() for im, _ in seq)
    h = max(im.height for seq in entries.values() for im, _ in seq)
    return w, h


def preview(path):
    """A contact sheet of the kit for the docs."""
    sheets = sequences()
    F, L = sheets['science_eq_fx'], sheets['science_eq_labels']
    im = Image.new('RGBA', (900, 520), '#0b1422')
    d = ImageDraw.Draw(im)
    d.text((16, 10), 'UNIFIED THEORY / THE EQUATION KIT', font=BOLD, fill=WHITE)
    for i, s in enumerate(SKILLS):
        x, y = 16 + (i % 6) * 146, 40 + (i // 6) * 150
        d.rounded_rectangle((x, y, x + 138, y + 140), 6, fill='#142338', outline=SCI[s['science']])
        tile = Image.new('RGBA', (64, 64))
        tile.alpha_composite(F[f'im{i}'][1][0].resize((64, 64), Image.Resampling.NEAREST))
        im.alpha_composite(tile.resize((96, 96), Image.Resampling.NEAREST), (x + 21, y + 4))
        d.text((x + 6, y + 102), s['name'], font=BOLD, fill=SCI[s['science']])
        d.text((x + 6, y + 120), s['equation'][:22], font=FONT, fill=WHITE)
    im.save(path)
