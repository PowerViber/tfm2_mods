"""Vesper and the Rift Gauntlets: body, rank outfits, rift-punch poses, emblems and auras.

Native pixels on the shared 48 x 64 anchor (body centre = canvas centre).
Her look goes from a scavenger with taped orbs in salvaged gauntlets (Spark)
to someone who half-lives in another world (#1 Between Worlds). Every rank adds
one readable layer; rank never adds power. All drawing is deterministic:
eight discrete phases, no randomness (hash noise only).
"""
import json
import math
from functools import lru_cache
from pathlib import Path

from PIL import Image, ImageDraw, ImageFilter, ImageFont, ImageChops, ImageOps

TAU = math.tau
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
DATA = json.loads((HERE / 'catalogue.json').read_text(encoding='utf-8'))
FONT_PATH = ('C:/Windows/Fonts/consola.ttf' if Path('C:/Windows/Fonts/consola.ttf').exists()
             else '/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf')
BOLD_PATH = ('C:/Windows/Fonts/consolab.ttf' if Path('C:/Windows/Fonts/consolab.ttf').exists()
             else '/usr/share/fonts/truetype/dejavu/DejaVuSansMono-Bold.ttf')

AURA = (160, 176)      # background aura canvas; the body sits at BODY
BODY = (56, 56)
BADGE = (64, 96)       # emblem canvas, centred on the body
RANKS = DATA['ranks']
TOP_NAME = DATA['top_name']


def hexc(h, alpha=255):
    return tuple(int(h[i:i + 2], 16) for i in (1, 3, 5)) + (alpha,)


FAMILY = [hexc(f['color']) for f in DATA['families']]
FAMILY_NAMES = [f['name'] for f in DATA['families']]

# ------------------------------------------------------------------ colours
INK = (21, 18, 26, 255)
WHITE = (248, 248, 255, 255)
RIM = (232, 228, 255, 255)          # neutral rift light
RIFT = (178, 128, 255, 255)         # Vesper's own violet rift light
RIFT_PALE = (214, 196, 255, 255)
RIFT_DEEP = (70, 40, 128, 255)
VOID = (18, 14, 38, 255)
GOLD = (255, 214, 120, 255)
GOLD_WHITE = (255, 246, 214, 255)
SILVER = (206, 216, 232, 255)
BRONZE = (214, 146, 86, 255)
CRYSTAL = (159, 226, 255, 255)
SKIN = (226, 180, 140, 255)
SKIN_D = (178, 126, 96, 255)
HAIR = (58, 40, 62, 255)
HAIR_L = (98, 72, 100, 255)
BOOT = (58, 40, 30, 255)
BOOT_L = (104, 76, 54, 255)
TROUSER = (60, 56, 70, 255)
TROUSER_L = (88, 84, 100, 255)
WOOD = (122, 82, 48, 255)
WOOD_L = (166, 118, 70, 255)
METAL = (98, 104, 118, 255)
METAL_L = (162, 170, 186, 255)
BRASS = (200, 160, 80, 255)
TAPE = (204, 192, 150, 255)


def a(color, alpha):
    return tuple(color[:3]) + (max(0, min(255, round(alpha))),)


def mix(c1, c2, t):
    return tuple(round(x + (y - x) * t) for x, y in zip(c1[:3], c2[:3])) + (255,)


def pt(x, y, r, ang, flat=1.0):
    return x + r * math.cos(ang), y + r * math.sin(ang) * flat


def line(d, pts, color, width=1):
    d.line([(round(x), round(y)) for x, y in pts], fill=color, width=width)


def poly(d, pts, fill=None, outline=None):
    d.polygon([(round(x), round(y)) for x, y in pts], fill=fill, outline=outline)


def dot(d, x, y, r, color, outline=None):
    d.ellipse((round(x - r), round(y - r), round(x + r), round(y + r)), fill=color, outline=outline)


def px(d, x, y, color):
    d.point((round(x), round(y)), fill=color)


def sparkle(d, x, y, color=WHITE, r=2):
    line(d, [(x - r, y), (x + r, y)], color)
    line(d, [(x, y - r), (x, y + r)], color)
    px(d, x, y, WHITE)


def ring(d, x, y, r, color, flat=1.0, steps=None, start=0.0, end=TAU, width=1):
    steps = steps or max(12, int(r * 2.5))
    pts = [pt(x, y, r, start + (end - start) * k / steps, flat) for k in range(steps + 1)]
    line(d, pts, color, width)


def dotted_ring(d, x, y, r, color, flat=1.0, n=24, phase=0.0, skip=2):
    for k in range(n):
        if k % skip == 0:
            px(d, *pt(x, y, r, phase + k * TAU / n, flat), color)


def glow(im, radius=1.5, boost=1.6):
    """A baked halo: blurred, alpha boosted, then the crisp sprite on top."""
    halo = im.filter(ImageFilter.GaussianBlur(radius))
    halo.putalpha(halo.getchannel('A').point(lambda v: min(255, round(v * boost))))
    halo.alpha_composite(im)
    return halo


def hsh(*k):
    """Deterministic hash noise in [0, 1)."""
    x = 2166136261
    for v in k:
        x = ((x ^ (int(v) & 0xffffffff)) * 16777619) & 0xffffffff
        x ^= x >> 13
        x = (x * 1274126177) & 0xffffffff
    x ^= x >> 16
    return x / 4294967296


@lru_cache(maxsize=16)
def font(size, bold=False):
    return ImageFont.truetype(BOLD_PATH if bold else FONT_PATH, size)


DIGITS = {
    '0': ['111', '101', '101', '101', '111'], '1': ['010', '110', '010', '010', '111'],
    '2': ['111', '001', '111', '100', '111'], '3': ['111', '001', '111', '001', '111'],
    '4': ['101', '101', '111', '001', '001'], '5': ['111', '100', '111', '001', '111'],
    '6': ['111', '100', '111', '101', '111'], '7': ['111', '001', '010', '010', '010'],
    '8': ['111', '101', '111', '101', '111'], '9': ['111', '101', '111', '001', '111'],
}


def digits(d, x, y, s, color):
    for n, ch in enumerate(s):
        for row, bits in enumerate(DIGITS[ch]):
            for col, bit in enumerate(bits):
                if bit == '1':
                    d.point((x + n * 4 + col, y + row), fill=color)


def shard(d, x, y, s, color, f=0, outline=INK, hi=WHITE):
    """A small floating rift-glass shard (diamond)."""
    pts = [(x, y - s * 1.5), (x + s, y), (x, y + s * 1.2), (x - s * .8, y)]
    poly(d, pts, fill=color, outline=outline)
    px(d, x, y - s * .6, hi)


def negative(im):
    """The other world's palette: inverted colour, same alpha."""
    r, g, b, al = im.split()
    inv = ImageChops.invert(Image.merge('RGB', (r, g, b)))
    out = inv.convert('RGBA')
    out.putalpha(al)
    return out


# ------------------------------------------------------- the other worlds --

def world(family, w, h, f, bright=1.0):
    """A tiny animated world vignette for one dimension family (w x h px)."""
    im = Image.new('RGBA', (w, h))
    d = ImageDraw.Draw(im)
    c = FAMILY[family]
    bg = mix(VOID, c, .28)
    d.rectangle((0, 0, w - 1, h - 1), fill=bg)
    ph = f * TAU / 8
    cx, cy = (w - 1) / 2, (h - 1) / 2
    if family == 0:      # Control: chains in fog
        for y in range(1, h, 3):
            line(d, [(0, y + (f + y) % 2), (w - 1, y)], a(SILVER, 50))
        for k in range(3):
            yy = 1 + (k * 3 + f // 2) % max(1, h - 2)
            d.rectangle((round(cx) - 1, yy, round(cx) + 1, yy + 1), outline=a(SILVER, 220))
    elif family == 1:    # Ruin: embers
        line(d, [(0, h - 1), (w - 1, h - 1)], a((255, 120, 60), 255))
        for k in range(4):
            ex = (k * 3 + 1) % w
            ey = h - 1 - ((f + k * 2) % h)
            px(d, ex, ey, mix(c, GOLD, k / 4))
    elif family == 2:    # Motion: wind streaks
        for k in range(3):
            y = 1 + k * max(1, (h - 2) // 3)
            x0 = (f + k * 3) % (w + 3) - 3
            line(d, [(x0, y), (x0 + 3, y)], a(WHITE, 220))
    elif family == 3:    # Haven: flowers
        line(d, [(0, h - 2), (w - 1, h - 2)], a((60, 140, 60), 255))
        for k in range(2):
            fx_ = 1 + k * (w - 3)
            px(d, fx_, h - 3, a((255, 236, 160), 255 if (f + k) % 4 < 3 else 160))
        px(d, cx, h - 4 - f % 2, (255, 190, 220, 255))
    elif family == 4:    # Passage: a door
        d.rectangle((round(cx) - 1, 1, round(cx) + 1, h - 1), outline=a(WHITE, 220))
        px(d, cx, 2 + f % max(1, h - 3), GOLD)
    elif family == 5:    # Time: hourglass
        poly(d, [(cx - 2, 1), (cx + 2, 1), (cx, cy), (cx + 2, h - 2), (cx - 2, h - 2), (cx, cy)], outline=a(GOLD, 255))
        px(d, cx, cy + 1 + f % 3, WHITE)
    elif family == 6:    # Veil: crescent moon in fog
        dot(d, cx, cy, min(w, h) / 3, a(WHITE, 220))
        dot(d, cx + 1 + (f % 2), cy - 1, min(w, h) / 3, bg)
        line(d, [((f * 2) % w - 2, h - 2), ((f * 2) % w + 2, h - 2)], a(SILVER, 120))
    elif family == 7:    # Matter: crystal
        poly(d, [(cx, 1), (cx + 2, cy), (cx, h - 2), (cx - 2, cy)], fill=a(c, 255), outline=a(WHITE, 200))
        px(d, cx + (1 if f % 4 < 2 else -1), 2 + f % 3, WHITE)
    elif family == 8:    # Hunger: teeth
        for x in range(0, w, 2):
            px(d, x, 1 + ((x // 2 + f) % 2), a(WHITE, 230))
            px(d, x + 1, h - 2 - ((x // 2 + f) % 2), a(WHITE, 230))
    else:                # Chaos: tumbling dice pips
        for k in range(1 + f % 5):
            px(d, 1 + (k * 3 + f) % max(1, w - 2), 1 + (k * 5 + f * 3) % max(1, h - 2), WHITE)
    px(d, (f * 3) % w, 0, a(WHITE, 200 * bright))
    return im


def paste_masked(im, src, x, y, mask_poly):
    """Paste `src` at (x, y) inside a polygon (in im coordinates)."""
    m = Image.new('L', im.size)
    ImageDraw.Draw(m).polygon([(round(u), round(v)) for u, v in mask_poly], fill=255)
    layer = Image.new('RGBA', im.size)
    layer.alpha_composite(src, (round(x), round(y)))
    layer.putalpha(ImageChops.multiply(layer.getchannel('A'), m))
    im.alpha_composite(layer)


# ------------------------------------------------------------------- body --
# The Rift Gauntlet fighter. Facing +x in an orthodox stance: the LEFT gauntlet (pure orbs) is the lead
# fist on screen-right, the RIGHT gauntlet (Hexcore) is the rear fist on screen-left. The fists are the
# silhouette (13 x 13 px plus a 2 px dark rim, nearly the size of her torso); the body stays simple.

PAL = [   # brown coat throughout, the lining deepens to violet as she ranks up
    dict(coat=(123, 91, 62), shade=(86, 64, 44), light=(162, 124, 85), belt=(70, 50, 36), lining=(70, 46, 96)),      # Spark
    dict(coat=(118, 86, 58), shade=(82, 60, 42), light=(158, 120, 80), belt=(66, 48, 34), lining=(76, 46, 108)),     # Tear
    dict(coat=(112, 80, 55), shade=(78, 56, 40), light=(152, 114, 76), belt=(62, 44, 32), lining=(82, 48, 118)),     # Rift
    dict(coat=(106, 74, 52), shade=(72, 50, 37), light=(146, 108, 74), belt=(58, 42, 32), lining=(86, 50, 128)),     # Gate
    dict(coat=(100, 69, 50), shade=(68, 47, 36), light=(140, 102, 72), belt=(54, 40, 32), lining=(90, 52, 136)),     # Wanderer
    dict(coat=(94, 64, 48), shade=(64, 44, 35), light=(134, 98, 70), belt=(50, 38, 32), lining=(94, 54, 144)),       # Cartographer
    dict(coat=(86, 58, 46), shade=(58, 40, 34), light=(126, 92, 70), belt=(46, 36, 32), lining=(100, 58, 156)),      # Riftwalker
    dict(coat=(72, 48, 42), shade=(46, 32, 30), light=(150, 128, 170), belt=(40, 32, 36), lining=(122, 76, 196)),    # Worldbreaker
    dict(coat=(150, 110, 74), shade=(104, 74, 50), light=(255, 230, 180), belt=(120, 96, 52), lining=(200, 160, 90)),  # Between Worlds
]
for _p in PAL:
    for _k in list(_p):
        _p[_k] = _p[_k] + (255,)

HEX = (110, 240, 255, 255)          # Hextech cyan: only a few pixels per gauntlet
HEX_D = (40, 120, 150, 255)


def _steel(metal, dark, light, trim, trim_l, trim_d):
    return dict(metal=metal + (255,), dark=dark + (255,), light=light + (255,), trim=trim + (255,),
                trim_l=trim_l + (255,), trim_d=trim_d + (255,))


STEEL = _steel((84, 90, 104), (50, 54, 66), (146, 154, 170), (170, 110, 60), (216, 142, 92), (108, 66, 36))
GLASS = _steel((128, 146, 192), (76, 84, 132), (228, 238, 255), (176, 150, 232), (255, 255, 255), (96, 76, 150))
GILT = _steel((222, 198, 142), (150, 118, 68), (255, 246, 214), (255, 214, 120), (255, 255, 255), (160, 120, 50))


def steel(look):
    return GILT if look == 8 else GLASS if look == 7 else STEEL


def legs(d, kind='idle', f=0):
    """Long legs (3 px longer than the marksman's) and boots; feet stay on y 58."""
    step = [1, 2, 1, 0, -1, -2, -1, 0][f] if kind == 'run' else 0
    if kind == 'attack':
        step = [0, 1, 1, 1, 0, 0, 0, 0][f]          # the jab steps in
    bdx, fdx = -step, step
    bl, fl = max(0, step), max(0, -step)
    for x0, dx, lift in ((20, bdx, bl), (25, fdx, fl)):
        d.rectangle((x0 + dx, 43, x0 + 3 + dx, 55 - lift), fill=INK)
        d.rectangle((x0 + 1 + dx, 43, x0 + 2 + dx, 54 - lift), fill=TROUSER if x0 == 20 else TROUSER_L)
        poly(d, [(x0 - 1 + dx, 55 - lift), (x0 + 3 + dx, 55 - lift), (x0 + 5 + dx, 57 - lift),
                 (x0 + 5 + dx, 58 - lift), (x0 - 1 + dx, 58 - lift)], fill=BOOT, outline=INK)
        line(d, [(x0 + dx, 56 - lift), (x0 + 3 + dx, 56 - lift)], BOOT_L)


def head(d, look, f):
    rank = min(look, 7)
    d.rectangle((23, 27, 25, 30), fill=SKIN_D)                      # neck
    if rank >= 1:                  # Tear: the hood
        hood = (59, 52, 74, 255) if rank < 7 else (96, 80, 140, 255) if look == 7 else (150, 120, 70, 255)
        hood_l = (90, 80, 112, 255) if rank < 7 else (200, 186, 255, 255) if look == 7 else GOLD_WHITE
        poly(d, [(18, 19), (21, 15), (26, 15), (29, 17), (30, 21), (27, 19), (23, 20),
                 (22, 25), (22, 30), (18, 29), (16, 24)], fill=hood, outline=INK)
        line(d, [(21, 16), (26, 16)], hood_l)
        line(d, [(18, 21), (17, 25)], hood_l)
    else:
        poly(d, [(19, 19), (22, 17), (26, 17), (28, 19), (28, 22), (23, 23), (22, 28),
                 (19, 27), (18, 23)], fill=HAIR, outline=INK)
        poly(d, [(18, 21), (16, 23), (15, 27), (17, 28), (19, 25)], fill=HAIR, outline=INK)
        line(d, [(21, 18), (25, 18)], HAIR_L)
    poly(d, [(23, 20), (27, 20), (29, 22), (29, 25), (28, 27), (26, 28), (23, 28), (22, 24)], fill=SKIN, outline=INK)
    line(d, [(23, 19), (28, 19)], HAIR if rank == 0 else INK)
    line(d, [(25, 22), (27, 22)], INK)                              # brow
    eye = WHITE if rank < 3 else HEX if look < 7 else (230, 220, 255, 255) if look == 7 else GOLD_WHITE
    px(d, 27, 23, eye)
    if rank < 3:                   # a cloth half-mask over nose and mouth
        poly(d, [(22, 25), (29, 24), (29, 26), (27, 28), (23, 29), (22, 27)], fill=(74, 52, 96, 255), outline=INK)
        line(d, [(24, 25), (28, 25)], (110, 84, 136, 255))
    else:                          # Gate: a riveted steel half-mask with one cyan seam
        S = steel(look)
        poly(d, [(22, 24), (29, 24), (30, 26), (28, 28), (23, 29), (22, 27)], fill=S['metal'], outline=INK)
        line(d, [(23, 25), (29, 25)], S['light'])
        px(d, 26, 27, HEX if look < 8 else GOLD)
        px(d, 23, 27, S['dark'])


def torso(d, p, hem):
    """Defined shoulders, a waist about 18% narrower than the marksman's, coat skirt to the hem."""
    poly(d, [(19, 30), (28, 30), (31, 32), (30, 35), (28, 39), (29, 42), (30, hem), (25, hem + 1), (23, hem - 1),
             (21, hem + 1), (18, hem), (19, 42), (19, 39), (17, 35), (16, 32)], fill=p['coat'], outline=INK)
    poly(d, [(26, 31), (29, 33), (28, 37), (27, 40), (28, hem - 1), (26, hem - 1), (25, 40)], fill=p['shade'])
    line(d, [(18, 32), (18, 35), (20, 39)], p['light'])
    # the coat opens below the belt onto the dark violet lining
    poly(d, [(23, 42), (25, 42), (26, hem - 1), (24, hem), (22, hem - 1)], fill=p['lining'])
    line(d, [(19, 41), (29, 41)], p['belt'])
    px(d, 24, 41, BRASS)
    line(d, [(21, 30), (26, 30)], p['light'])


def gear(d, x, y, S, f):
    d.ellipse((x - 2, y - 2, x + 2, y + 2), fill=S['trim'], outline=INK)
    px(d, x, y, INK)
    tooth = [(0, -3), (2, -2), (3, 0), (2, 2), (0, 3), (-2, 2), (-3, 0), (-2, -2)]
    for k in range(2):
        tx, ty = tooth[(k * 4 + f) % 8]
        px(d, x + tx, y + ty, S['trim_d'])


ARM = (18, 26)       # one gauntlet on its own canvas: housing on top, fist below (left-gauntlet orientation)


def gauntlet(look, f, side):
    """One Rift Gauntlet, drawn hanging: a tapered forearm housing (overlapping plates, piston rod, gear)
    over a 13 x 13 fist seen from the front: bronze cuff, four thick fingers in two segments, a row of
    square knuckle plates as the striking face and a copper thumb plate across. The right gauntlet is the
    mirror image and carries the hexagonal Hexcore; a few cyan pixels at the seams. 2 px dark rim."""
    S = steel(look)
    rank = min(look, 7)
    im = Image.new('RGBA', ARM)
    d = ImageDraw.Draw(im)
    o = 1                                                   # room for the rim
    X = lambda x: x + o
    # housing (rows 0..10): narrower and darker than the fist, so the fist reads as a mallet
    poly(d, [(X(3), X(0)), (X(11), X(0)), (X(10), X(10)), (X(4), X(10))], fill=S['dark'], outline=INK)
    line(d, [(X(4), X(1)), (X(10), X(1))], S['metal'])
    line(d, [(X(4), X(5)), (X(10), X(5))], INK)             # overlapping plates
    line(d, [(X(5), X(6)), (X(9), X(6))], S['metal'])
    gear(d, X(6), X(8), S, f)
    line(d, [(X(10), X(2)), (X(10), X(9))], S['light'])     # piston rod along the outer edge
    px(d, X(10), X(2 + (f * 3) % 8), WHITE)
    if side == 'L':     # five pure-orb sockets; four overload sockets from rank 2
        for k, row in enumerate((1, 2, 3, 6, 7)):
            col = FAMILY[(k * 2 + f // 4) % 10] if rank else (RIFT_PALE if (k + f) % 4 else (90, 80, 120, 255))
            px(d, X(8), X(row), col)
        if rank >= 2:
            for k in range(3):
                px(d, X(5 + k), X(3), (255, 154, 90, 255) if (k + f) % 6 == 0 else S['metal'])
        if rank == 0:   # Spark: the orbs are taped in
            line(d, [(X(7), X(2)), (X(9), X(4))], TAPE)
    else:               # the Hexcore, set into the housing above the wrist
        hx, hy = X(7), X(3)
        for dy, (x0, x1) in zip(range(-2, 3), ((-1, 1), (-2, 2), (-2, 2), (-2, 2), (-1, 1))):
            line(d, [(hx + x0, hy + dy), (hx + x1, hy + dy)], S['trim'])
        for dy in (-1, 0, 1):
            line(d, [(hx - 1, hy + dy), (hx + 1, hy + dy)], VOID)
        core = HEX if look < 7 else (236, 246, 255, 255) if look == 7 else GOLD_WHITE
        pulse = [core, core, mix(core, HEX_D, .4), mix(core, HEX_D, .6), mix(core, HEX_D, .4), core, WHITE, core][f]
        px(d, hx, hy, pulse)
        px(d, hx, hy - 1 if f % 2 else hy + 1, a(core, 190))
    # fist (rows 10..22): domed back of the hand, bronze cuff, fingers, square knuckles stepping out below
    poly(d, [(X(3), X(10)), (X(11), X(10)), (X(13), X(12)), (X(13), X(20)), (X(1), X(20)), (X(1), X(12))],
         fill=S['metal'], outline=INK)
    line(d, [(X(3), X(11)), (X(11), X(11))], S['trim_l'])                   # bronze cuff
    line(d, [(X(2), X(12)), (X(12), X(12))], S['trim'])
    line(d, [(X(2), X(13)), (X(12), X(13))], INK)
    line(d, [(X(2), X(14)), (X(9), X(14))], S['light'])                     # back-of-hand plate
    px(d, X(3), X(15), S['light'])
    px(d, X(11), X(15), S['dark'])
    line(d, [(X(1), X(17)), (X(13), X(17))], INK)                           # finger joint line
    for k, c0 in enumerate((2, 5, 8, 11)):
        d.rectangle((X(c0), X(18), X(c0 + 1), X(19)), fill=S['metal'])     # finger segment
        px(d, X(c0), X(18), S['light'])
        # the square knuckle plate, stepping 1 px out of the silhouette
        d.rectangle((X(c0 - 1), X(20), X(c0 + 2), X(23)), fill=INK)
        d.rectangle((X(c0), X(20), X(c0 + 1), X(22)), fill=S['trim'])
        line(d, [(X(c0), X(20)), (X(c0 + 1), X(20))], S['trim_l'])
        px(d, X(c0 + 1), X(22), S['trim_d'])
        if c0 < 11:
            line(d, [(X(c0 + 2), X(17)), (X(c0 + 2), X(19))], INK)
    if side == 'R':     # the right gauntlet's knuckles are heavier: a bronze bar across them
        line(d, [(X(2), X(19)), (X(12), X(19))], S['trim_d'])
    # the thumb plate crossing from the inner side
    poly(d, [(X(0), X(14)), (X(3), X(14)), (X(6), X(16)), (X(5), X(18)), (X(0), X(18))], fill=S['trim_d'], outline=INK)
    line(d, [(X(1), X(15)), (X(3), X(15))], S['trim'])
    # Hextech: a few cyan pixels in the finger joints
    glow_px = HEX if look < 8 else GOLD_WHITE
    px(d, X(10), X(17), glow_px)
    if rank >= 3:
        px(d, X(7), X(17), glow_px)
    if rank == 0 and side == 'L':
        line(d, [(X(8), X(14)), (X(11), X(16))], TAPE)                      # Spark: a taped crack
    if side == 'R':
        im = ImageOps.mirror(im)
    return rim(im)


def rim(im):
    """The second pixel of dark outline that keeps the fists readable at game scale."""
    m = im.getchannel('A').filter(ImageFilter.MaxFilter(3))
    edge = Image.new('RGBA', im.size, INK)
    edge.putalpha(ImageChops.subtract(m, im.getchannel('A')).point(lambda v: 255 if v else 0))
    edge.alpha_composite(im)
    return edge


def sleeve(d, p, side):
    if side == 'R':
        poly(d, [(13, 30), (17, 30), (19, 32), (18, 35), (13, 35)], fill=p['shade'], outline=INK)
    else:
        poly(d, [(29, 30), (33, 30), (35, 32), (35, 35), (30, 35)], fill=p['coat'], outline=INK)


REAR = (1, 32)       # where each gauntlet canvas sits at rest (fist about x 3..15 and x 31..43)
LEAD = (29, 32)
PUNCH_AT = (22, 24)  # the lead gauntlet turned to punch along +x


def arm(look, f, side, pose='rest'):
    """One arm as its own 48 x 64 layer: sleeve cap and gauntlet."""
    im = Image.new('RGBA', (48, 64))
    d = ImageDraw.Draw(im)
    p = PAL[look]
    if side == 'L' and pose == 'punch':
        poly(d, [(27, 29), (31, 29), (32, 33), (28, 35)], fill=p['coat'], outline=INK)
        im.alpha_composite(gauntlet(look, f, 'L').rotate(90, expand=True), PUNCH_AT)
        return im
    sleeve(d, p, side)
    im.alpha_composite(gauntlet(look, f, side), REAR if side == 'R' else LEAD)
    return im


HEM = [46, 46, 46, 50, 50, 50, 50, 50, 50]


def figure(look, f):
    """The opaque rest silhouette in a rank palette (head, coat, gauntlets)."""
    im = Image.new('RGBA', (48, 64))
    d = ImageDraw.Draw(im)
    torso(d, PAL[look], HEM[0])
    head(d, 0, f)
    im.alpha_composite(arm(look, f, 'R'))
    im.alpha_composite(arm(look, f, 'L'))
    return im


@lru_cache(maxsize=None)
def lead_mask():
    """Every pixel the rank-0 body's resting lead arm can cover; the punch pose must hide it."""
    m = Image.new('L', (48, 64))
    for f in range(8):
        m = ImageChops.lighter(m, arm(0, f, 'L').getchannel('A'))
    return m.point(lambda v: 255 if v else 0)


def torso_only(look=0):
    """For the verifier: the bare torso of a look, without head, arms or legs."""
    im = Image.new('RGBA', (48, 64))
    torso(ImageDraw.Draw(im), PAL[look], HEM[look])
    return im


def shards_orbit(d, n, f, front, ox=0, oy=0, rx=19, ry=5, cy=38, size=1.4, look=7, families=False):
    """n small rift shards orbiting the waist. Front half in the outfit, back half in the aura."""
    for k in range(n):
        ang = (f / 8) * TAU / n + k * TAU / n + .3
        s = math.sin(ang)
        if (s > 0) != front:
            continue
        x, y = 24 + rx * math.cos(ang) + ox, cy + ry * s + oy - 2 * math.cos(ang * 2)
        col = FAMILY[k % 10] if families else (196, 206, 255, 255) if look < 7 else (222, 236, 255, 255)
        shard(d, x, y, size if front else size * .9, col if front else mix(col, VOID, .25))


def tear_portal(im, look, f):
    """Punch pose: the tear her lead fist is thrust into. Opaque, so it hides the resting arm below."""
    d = ImageDraw.Draw(im)
    rank = min(look, 7)
    jag = [0, 1, 0, -1, 0, 1, -1, 0]
    pts = [(39, 17), (41, 22), (44 + jag[f], 24), (44, 28), (47, 31), (46, 36), (47, 41), (45, 44), (47, 49),
           (44 + jag[(f + 3) % 8], 53), (42, 58), (37, 56), (34, 58), (31, 54), (29, 50), (31 + jag[(f + 5) % 8], 46),
           (29, 42), (31, 38), (30, 33), (33, 29), (34 + jag[(f + 1) % 8], 23), (37, 22)]
    rim_col = (236, 222, 176, 255) if rank == 0 else GOLD_WHITE if look == 8 else (228, 236, 255, 255) if look == 7 else RIFT_PALE
    cover = Image.new('RGBA', (48, 64), VOID)
    cover.putalpha(lead_mask())
    im.alpha_composite(cover)
    d = ImageDraw.Draw(im)
    poly(d, pts, fill=VOID, outline=rim_col)
    inner = [(38, 23), (42, 30), (44, 39), (43, 49), (39, 54), (34, 51), (33, 41), (34, 30)]
    poly(d, inner, fill=RIFT_DEEP if look != 8 else (90, 70, 30, 255))
    for k in range(5):       # the dimension cracks outward from the fist's entry point
        ang = math.pi / 2 + (k - 2) * .5 + hsh(31, k) * .2
        q0 = pt(43, 34, 2, ang)
        q1 = pt(43, 34, 9 + (k + f) % 4, ang)
        line(d, [q0, q1], mix(rim_col, RIFT_DEEP, .25))
    for k in range(6):       # the other world's stars
        sx = 34 + (k * 5 + f) % 10
        sy = 26 + (k * 9 + f * 3) % 28
        px(d, sx, sy, FAMILY[(k * 3 + rank) % 10] if rank else (200, 190, 160, 255))
    for k in range(3):       # light runs along the rim
        q = pts[(f + k * 4) % len(pts)]
        px(d, q[0], q[1], WHITE)
    if rank == 0 and f % 3 == 1:   # Spark: the tear sputters
        line(d, [pts[4], (44, 44)], (236, 222, 176, 255))
    return im


def costume(look, f, punch=False):
    """Rank layers on top of the shared silhouette (look 0..8). punch=True: the rift-punch pose."""
    rank = min(look, 7)
    im = Image.new('RGBA', (48, 64))
    d = ImageDraw.Draw(im)
    p = PAL[look]
    S = steel(look)
    hem = HEM[look]
    if rank >= 4:      # Wanderer: the travel cloak hangs behind, its violet lining full of stars
        poly(d, [(19, 30), (15, 35), (14, 44), (15, hem + 1), (19, hem), (18, 33)], fill=(36, 24, 62, 255), outline=INK)
        for k in range(4):
            sx = 15 + (k * 2 + f) % 3
            sy = 35 + (k * 5 + f * 2) % max(1, hem - 36)
            px(d, sx, sy, (230, 236, 255, 255) if (k + f) % 3 else RIFT_PALE)
    if punch:
        tear_portal(im, look, f)
        d = ImageDraw.Draw(im)
        poly(d, [(15, 56), (20, 56), (21, 58), (15, 58)], fill=BOOT, outline=INK)     # braced, wider stance
        poly(d, [(27, 56), (32, 56), (33, 58), (27, 58)], fill=BOOT, outline=INK)
    torso(d, p, hem)
    if rank == 0:      # Spark: patches on the coat
        d.rectangle((26, 43, 28, 45), fill=(95, 107, 72, 255), outline=INK)
        px(d, 27, 44, (130, 140, 100, 255))
    if rank >= 1:      # Tear: cyan seams stitched down the coat edge
        seam = HEX if look != 8 else GOLD
        for k, q in enumerate([(22, 43), (22, 45), (26, 44), (26, 46)]):
            px(d, *q, WHITE if k == f % 4 else seam)
    if rank >= 2:      # Rift: stress gauge on the belt
        d.ellipse((19, 40, 22, 43), fill=(30, 26, 36, 255), outline=BRASS)
        needle = [(20, 41), (21, 41), (21, 42), (21, 42), (22, 42), (21, 42), (21, 41), (20, 41)][f]
        line(d, [(20, 42), needle], (255, 120, 90, 255))
    if rank >= 3:      # Gate: long coat with gate-ring sigils
        for sx, sy in ((20, 47), (27, 47)):
            d.ellipse((sx - 1, sy - 1, sx + 1, sy + 1), outline=GOLD if look != 8 else GOLD_WHITE)
        px(d, 21 + (f % 4 == 0), 47, WHITE)
    if rank >= 5:      # Cartographer: an orb bandolier across the chest, one spare orb per family pair
        line(d, [(19, 31), (28, 40)], (64, 44, 34, 255), 2)
        for k in range(5):
            gx, gy = 20 + k * 2, 32 + k * 2
            px(d, gx, gy, WHITE if (k + f) % 5 == 0 else FAMILY[(k * 2 + 1) % 10])
    if rank >= 6:      # Riftwalker: the hem dissolves into rift fragments
        for k in range(5):
            fx_ = 18 + k * 3
            fy = hem + 1 + (f + k * 3) % 5
            col = (190, 170, 255, 255) if look != 8 else GOLD_WHITE
            if fy <= 60:
                d.rectangle((fx_, fy, fx_ + (k % 2), fy), fill=a(col, 255 - (fy - hem) * 30))
    if rank >= 7:      # Worldbreaker: rift-glass facets on the coat
        for (x0, y0, x1, y1) in [(20, 32, 22, 37), (26, 33, 27, 38), (20, 43, 22, 48), (28, 43, 27, 48)]:
            line(d, [(x0, y0), (x1, y1)], (236, 228, 255, 255))
    head(d, look, f)
    # the gauntlets
    rear = arm(look, f, 'R')
    lead = arm(look, f, 'L', 'punch' if punch else 'rest')
    for layer, side in ((rear, 'R'), (lead, 'L')):
        ld = ImageDraw.Draw(layer)
        if rank >= 4:      # Wanderer: copper coil piping from the housing to the shoulder
            if side == 'R':
                line(ld, [(6, 33), (9, 30), (14, 29)], S['trim_l'])
            elif not punch:
                line(ld, [(42, 33), (39, 30), (34, 29)], S['trim_l'])
        if rank >= 6:      # Riftwalker: rift-light cracks through the plates
            cracks_ = [(5, 44), (6, 46), (7, 47)] if side == 'R' else ([(36, 31), (37, 33), (38, 34)] if punch else [(34, 44), (35, 46), (34, 48)])
            for k, q in enumerate(cracks_):
                px(ld, *q, WHITE if k == f % 3 else (190, 170, 255, 255))
        if rank >= 7:      # Worldbreaker: rift-glass gauntlets, white facets over the plates
            base = (4, 42) if side == 'R' else ((35, 29) if punch else (33, 42))
            for (x0, y0, x1, y1) in [(0, 0, 3, 4), (1, 6, 4, 9)]:
                line(ld, [(base[0] + x0, base[1] + y0), (base[0] + x1, base[1] + y1)], WHITE)
        im.alpha_composite(layer)
    d = ImageDraw.Draw(im)
    if rank >= 2:      # Rift: a bronze pauldron over the lead shoulder
        poly(d, [(27, 29), (31, 29), (34, 31), (34, 33), (29, 33)], fill=S['trim'], outline=INK)
        line(d, [(28, 30), (32, 30)], S['trim_l'])
    if punch:          # the rim of the tear closes over the knuckles: the fist is going through
        rim_col = (236, 222, 176, 255) if rank == 0 else GOLD_WHITE if look == 8 else (228, 236, 255, 255) if look == 7 else RIFT_PALE
        line(d, [(44, 27 + f % 2), (45, 33), (44, 39 - f % 2)], rim_col)
        px(d, 46, 33, WHITE)
        px(d, 47, 30 + f % 6, HEX)
    if look < 7 and rank >= 3:
        shards_orbit(d, 2 if rank < 6 else 4, f, True, look=look)
    if look == 7:
        shards_orbit(d, 6, f, True, look=7)
    if look == 8:
        shards_orbit(d, 10, f, True, families=True)
    if rank >= 7:      # crown of floating shards
        for k in range(5):
            x = 19 + k * 2.5
            y = 13 - (2 if k == 2 else 1 if k in (1, 3) else 0) + (1 if (k + f) % 4 < 2 else 0)
            shard(d, x, y, 1, GOLD_WHITE if look == 8 else (220, 232, 255, 255), outline=INK if look < 8 else (120, 90, 40, 255))
    # Hextech sparks: sputtering at Spark, steady later
    cx, cy = (45, 34) if punch else (44, 50)
    if rank == 0:
        for k in range(2):
            if (f + k) % 3 != 2:
                sx, sy = pt(cx, cy, 2 + (f + k * 2) % 3, hsh(7, f, k) * TAU)
                px(d, sx, sy, (255, 236, 160, 255) if k else HEX)
    elif look < 8:
        px(d, *pt(10, 36, 4, f * TAU / 8), a(HEX, 200))                       # the Hexcore hums
    if punch:          # anything translucent over the tear still sits on the void, never on the resting arm
        under = Image.new('RGBA', (48, 64), VOID)
        under.putalpha(lead_mask())
        under.alpha_composite(im)
        im = under
    if look == 8:
        # Between Worlds: the right half lives in the other world's palette
        left = im.crop((0, 0, 24, 64))
        right = negative(im.crop((24, 0, 48, 64)))
        im = Image.new('RGBA', (48, 64))
        im.alpha_composite(left, (0, 0))
        im.alpha_composite(right, (24, 0))
        d = ImageDraw.Draw(im)
        alpha = im.getchannel('A')
        for y in range(64):
            if alpha.getpixel((24, y)) or alpha.getpixel((23, y)):
                bright = (y + f * 4) % 16
                px(d, 24, y, GOLD_WHITE if bright < 3 else a(GOLD, 220) if bright < 8 else a(WHITE, 160))
    return im


def outfit(look, f):
    return costume(look, f, False)


def punch(look, f):
    return costume(look, f, True)


def body(kind='idle', f=0):
    """The registered body sheet: rank-0 look, eight frames per animation."""
    im = Image.new('RGBA', (48, 64))
    d = ImageDraw.Draw(im)
    if kind == 'dead':
        poly(d, [(12, 54), (16, 50), (32, 50), (35, 53), (32, 58), (14, 58)], fill=INK)
        d.rectangle((16, 51, 31, 55), fill=PAL[0]['coat'])
        d.rectangle((17, 54, 30, 55), fill=PAL[0]['shade'])
        poly(d, [(32, 49), (37, 48), (39, 51), (36, 54), (32, 54)], fill=SKIN, outline=INK)
        poly(d, [(33, 48), (37, 47), (39, 49), (34, 50)], fill=HAIR)
        # the gauntlets lie heavy on the ground; the Hexcore fades
        im.alpha_composite(gauntlet(0, 0, 'R').rotate(-90, expand=True), (-5, 42))
        im.alpha_composite(gauntlet(0, 0, 'L').rotate(90, expand=True), (27, 44))
        d = ImageDraw.Draw(im)
        dim = [255, 200, 150, 110, 80, 55, 35, 20][f]
        px(d, 13, 51, a(HEX_D, 255))
        px(d, 13, 51, a(HEX, dim))
        if f < 6:
            px(d, 12 + (f % 3), 46 - f // 2, a((255, 236, 160), 200 - f * 30))
        return im
    legs(d, kind, f)
    im.alpha_composite(outfit(0, f))
    d = ImageDraw.Draw(im)
    if kind == 'attack':       # the normal jab: a knuckle flash and a cyan shock off the lead fist
        if 1 <= f <= 4:
            r = [0, 2, 4, 3, 1][f]
            for k in range(3):
                line(d, [(45, 43 + k * 3), (45 + r, 43 + k * 3)], a(HEX, 230))
            ring(d, 46, 47, 2 + r, a(WHITE, 220), .9, 10, -1.2, 1.2)
        else:
            px(d, 46, 46 + f % 3, a((200, 220, 230), 160))
    elif kind == 'skill1':     # orbs ignite down the left housing
        for k in range(1 + f // 2):
            px(d, 39 + k % 2, 33 + k, FAMILY[(k * 2 + f) % 10])
        if f >= 3:
            px(d, *pt(44, 36, 3 + f % 3, f), HEX)
    elif kind == 'skill2':     # the swipe: a tearing arc in front of her
        for k in range(f + 1):
            q = pt(30, 40, 16, -1.2 + k * .3)
            px(d, *q, RIFT_PALE if k < f else WHITE)
    elif kind == 'ult':        # the Hexcore surges
        for k in range(6):
            line(d, [pt(10, 36, 3 + f, k * TAU / 6 + f * .2), pt(10, 36, 5 + f, k * TAU / 6 + f * .2)], a(HEX, 255 - f * 20))
    elif kind == 'hit':
        if f < 4:
            m = im.getchannel('A').filter(ImageFilter.MaxFilter(3))
            edge = ImageChops.subtract(m, im.getchannel('A'))
            flash = Image.new('RGBA', (48, 64), (255, 255, 255, 255))
            flash.putalpha(edge.point(lambda v: v * (4 - f) // 4))
            im.alpha_composite(flash)
            d = ImageDraw.Draw(im)
        px(d, 31 + f, 33 - f // 2, a((255, 200, 160), 255 - f * 28))
    elif kind == 'idle':
        px(d, 14 + f, 61, a(HEX, 60))
    elif kind == 'run':
        px(d, 13 - f % 4, 59 - f // 4, a((150, 130, 110), 120))
    return im


# ------------------------------------------------------------------ emblems --

def badge(rank, f, position=None):
    """Compact emblem above-right of the head (<= 18 x 18 px)."""
    im = Image.new('RGBA', BADGE)
    d = ImageDraw.Draw(im)
    x, y = 49, 28
    ph = f * TAU / 8
    if rank == 0:          # Spark: a small, dull spark
        dull = (168, 154, 122, 255)
        poly(d, [(x, y - 4), (x + 1, y - 1), (x + 4, y), (x + 1, y + 1), (x, y + 4), (x - 1, y + 1), (x - 4, y), (x - 1, y - 1)],
             fill=dull, outline=INK)
        tips = [(x, y - 4), (x + 4, y), (x, y + 4), (x - 4, y)]
        if f % 2 == 0:
            px(d, *tips[f // 2], (255, 236, 170, 255))
        px(d, x, y, (220, 210, 170, 255))
    elif rank == 1:        # Tear: a slit
        poly(d, [(x, y - 6), (x + 2, y - 2), (x + 2, y + 2), (x, y + 6), (x - 2, y + 2), (x - 2, y - 2)], fill=VOID, outline=RIFT_PALE)
        g = [120, 160, 210, 250, 210, 160, 120, 90][f]
        line(d, [(x, y - 4), (x, y + 4)], a(RIFT, g))
        px(d, x, y - 3 + f % 7, WHITE)
    elif rank == 2:        # Rift: a jagged rip
        pts = [(x - 1, y - 7), (x + 2, y - 4), (x + 1, y - 2), (x + 3, y), (x + 1, y + 2), (x + 2, y + 5), (x, y + 7),
               (x - 2, y + 4), (x - 1, y + 2), (x - 3, y), (x - 1, y - 2), (x - 2, y - 4)]
        poly(d, pts, fill=RIFT_DEEP, outline=RIM)
        line(d, [(x, y - 4), (x, y + 4)], RIFT)
        px(d, x + 4 + f % 3, y - 3 + f // 2, a(RIM, 220 - f * 20))
    elif rank == 3:        # Gate: a ring
        d.ellipse((x - 5, y - 5, x + 5, y + 5), outline=GOLD)
        d.ellipse((x - 3, y - 3, x + 3, y + 3), fill=VOID)
        for k in range(4):
            px(d, *pt(x, y, 5, ph / 4 + k * TAU / 4), WHITE)
        px(d, x, y, RIFT_PALE)
        line(d, [(x, y - 2), (x, y + 2)], RIFT)
    elif rank == 4:        # Wanderer: a compass
        d.ellipse((x - 5, y - 5, x + 5, y + 5), fill=(32, 28, 40, 255), outline=BRASS)
        for k in range(4):
            px(d, *pt(x, y, 4, k * TAU / 4), (230, 220, 190, 255))
        nx, ny = pt(x, y, 3.5, ph - TAU / 4)
        line(d, [(x, y), (nx, ny)], RIFT_PALE)
        line(d, [(x, y), pt(x, y, 2, ph + TAU / 4)], (255, 120, 90, 255))
    elif rank == 5:        # Cartographer: a grid
        d.rectangle((x - 5, y - 5, x + 5, y + 5), fill=(30, 34, 52, 255), outline=(227, 211, 166, 255))
        for k in (-2, 1):
            line(d, [(x + k, y - 4), (x + k, y + 4)], (90, 200, 220, 255))
            line(d, [(x - 4, y + k), (x + 4, y + k)], (90, 200, 220, 255))
        cells = [(-4, -4), (-1, -4), (2, -4), (2, -1), (2, 2), (-1, 2), (-4, 2), (-4, -1)]
        cx_, cy_ = cells[f]
        d.rectangle((x + cx_, y + cy_, x + cx_ + 1, y + cy_ + 1), fill=WHITE)
    elif rank == 6:        # Riftwalker: fractured
        poly(d, [(x, y - 6), (x + 5, y), (x, y + 6), (x - 5, y)], fill=(60, 50, 110, 255), outline=RIM)
        line(d, [(x - 2, y - 3), (x + 1, y), (x - 1, y + 3)], WHITE)
        line(d, [(x + 1, y), (x + 3, y - 1)], WHITE)
        for k in range(2):
            t = (f + k * 4) % 8
            px(d, x + 6 + t // 2, y - 5 + k * 9 - t // 3, a(RIFT_PALE, 240 - t * 25))
    else:                  # Worldbreaker: numbered shard
        col = GOLD if position == 1 else SILVER if position == 2 else BRONZE if position == 3 else (190, 206, 240, 255)
        poly(d, [(x - 4, y - 7), (x + 4, y - 7), (x + 7, y), (x + 4, y + 7), (x - 4, y + 7), (x - 7, y)],
             fill=(26, 24, 44, 255), outline=col)
        if position == 1:
            # split: right half of the crest in the other world's palette
            poly(d, [(x + 1, y - 6), (x + 4, y - 6), (x + 6, y), (x + 4, y + 6), (x + 1, y + 6)], fill=(70, 60, 30, 255))
            line(d, [(x, y - 7), (x, y + 7)], GOLD_WHITE)
        label = str(position or 10)
        left = x - (len(label) * 4 - 1) // 2
        digits(d, left, y - 2, label, WHITE)
        px(d, *pt(x, y, 7.2, ph), WHITE)
        if position and position <= 3:
            for k in range(4 - position):
                px(d, x - 2 + k * 2, y - 9, col)
    return im


# -------------------------------------------------------------- the auras --

def hexagon(d, x, y, r, color, width=1):
    pts = [pt(x, y, r, k * TAU / 6) for k in range(6)]
    line(d, pts + [pts[0]], color, width)


def hexes(d, cx, cy, w, h, r, color):
    """A honeycomb of outlined hexagons over a w x h half-extent; returns the cell centres."""
    cells = []
    dx, dy = r * 1.5, r * math.sqrt(3)
    for i in range(-int(w / dx) - 1, int(w / dx) + 2):
        for j in range(-int(h / dy) - 1, int(h / dy) + 2):
            x, y = cx + i * dx, cy + j * dy + (dy / 2 if i % 2 else 0)
            if abs(x - cx) <= w and abs(y - cy) <= h:
                hexagon(d, x, y, r, color)
                cells.append((x, y))
    return cells


def cracks(seed, cx, cy, n, length, spread=TAU, base=0.0, branch=True):
    """Deterministic crack polylines radiating from (cx, cy)."""
    out = []
    for k in range(n):
        ang = base + (k + .5) * spread / n + (hsh(seed, k) - .5) * .5
        pts = [(cx, cy)]
        x, y = cx, cy
        seg = max(3, length / 5)
        for j in range(5):
            ang += (hsh(seed, k, j) - .5) * .9
            x, y = x + seg * math.cos(ang), y + seg * math.sin(ang)
            pts.append((x, y))
            if branch and j == 2:
                b = ang + (.7 if hsh(seed, k, 99) > .5 else -.7)
                out.append([(x, y), (x + seg * 1.4 * math.cos(b), y + seg * 1.4 * math.sin(b))])
        out.append(pts)
    return out


def back(rank, f, podium=4):
    """One baked background loop behind the body (z -1). Each rank adds a layer."""
    im = Image.new('RGBA', AURA)
    d = ImageDraw.Draw(im)
    bx, by = BODY
    cx, cy = bx + 24, by + 32        # body centre
    ph = f * TAU / 8
    top = rank >= 7
    gold = top and podium == 1
    if top:
        col = GOLD_WHITE if podium == 1 else SILVER if podium == 2 else BRONZE if podium == 3 else (196, 214, 245, 255)
        if podium <= 3:
            # cracked sky: #1 largest, #2 silver, #3 bronze and smaller
            size = {1: 62, 2: 46, 3: 34}[podium]
            sky = Image.new('RGBA', AURA)
            sd = ImageDraw.Draw(sky)
            hub = (cx, cy - 46)
            paths = cracks(41 + podium, *hub, {1: 9, 2: 7, 3: 5}[podium], size, TAU, ph * .05)
            for pth in paths:
                line(sd, pth, a(col, 230), 2 if podium == 1 else 1)
            if podium == 1:
                # a Hextech lattice glows through the cracks
                grid = Image.new('RGBA', AURA)
                gd = ImageDraw.Draw(grid)
                hexes(gd, hub[0], hub[1], 70, 60, 8, a((127, 240, 255), 255))
                near = sky.getchannel('A').filter(ImageFilter.MaxFilter(9)).point(lambda v: 110 if v else 0)
                grid.putalpha(ImageChops.multiply(grid.getchannel('A'), near))
                im.alpha_composite(grid)
                d = ImageDraw.Draw(im)
            im.alpha_composite(glow(sky, 1.6, 1.3))
            d = ImageDraw.Draw(im)
            # light pulses along a crack
            pth = paths[f % len(paths)]
            q = pth[min(len(pth) - 1, 1 + f % 4)]
            sparkle(d, q[0], q[1], col, 2)
            # worlds seen through the sky: 2 for #2, 1 for #3, 10 shards for #1
            if podium in (2, 3):
                fams = [1, 7] if podium == 2 else [4]
                for k, fam in enumerate(fams):
                    wx, wy = hub[0] + (-26 + k * 52 if podium == 2 else 0), hub[1] + (6 if podium == 2 else -14)
                    vig = world(fam, 11, 9, f)
                    pts = [pt(wx, wy, 7, j * TAU / 10, .8) for j in range(10)]
                    paste_masked(im, vig, wx - 5, wy - 4, pts)
                    d = ImageDraw.Draw(im)
                    line(d, pts + [pts[0]], col)
        # shard halo / cracked-glass halo behind the head
        hx, hy = cx, cy - 22
        r = 17 if podium == 4 else 19
        for k in range(12):
            ang = k * TAU / 12 + ph / 12
            x0, y0 = pt(hx, hy, r, ang)
            x1, y1 = pt(hx, hy, r + 4 + (k % 2) * 2, ang + .12)
            poly(d, [(x0, y0), (x1, y1), pt(hx, hy, r + 1, ang + .26)], fill=a(col, 120 if k % 3 else 200), outline=a(col, 230))
        for pth in cracks(7, hx, hy, 6, 9, TAU, .4, False):
            line(d, pth[:3], a(col, 140))
        ring(d, hx, hy, r - 2, a(col, 150 + 40 * math.sin(ph)))
        if podium == 1:
            # 10 orbiting family shards, each holding a tiny world
            for k in range(10):
                ang = (f / 8) * TAU / 10 + k * TAU / 10
                sx, sy = pt(cx, cy - 2, 62, ang, .42)
                sy += -6 * math.sin(ang * 2 + ph * .1)
                pts = [(sx, sy - 7), (sx + 5, sy), (sx, sy + 7), (sx - 5, sy)]
                paste_masked(im, world(k, 9, 13, f), sx - 4, sy - 6, pts)
                d = ImageDraw.Draw(im)
                line(d, pts + [pts[0]], FAMILY[k])
                px(d, sx, sy - 5, GOLD_WHITE)
            # gold-white threads between neighbouring shards
            for k in range(0, 10, 2):
                ang = (f / 8) * TAU / 10 + k * TAU / 10
                p0 = pt(cx, cy - 2, 62, ang, .42)
                p1 = pt(cx, cy - 2, 62, ang + TAU / 10, .42)
                line(d, [p0, p1], a(GOLD_WHITE, 70))
        else:
            for k in range(6):
                ang = (f / 8) * TAU / 6 + k * TAU / 6 + .3
                if math.sin(ang) <= 0:
                    x, y = bx + 24 + 17 * math.cos(ang), by + 40 + 5 * math.sin(ang) - 2 * math.cos(ang * 2)
                    shard(d, x, y, 1.3, (170, 184, 230, 255))
        out = glow(im, 1.4 if gold else 1.1, 1.7 if gold else 1.4)
        ImageDraw.Draw(out).point((round(cx - 20 + f), round(cy + 30)), fill=a(col, 220))
        return out
    if rank == 0:
        # Spark: a few dull sparks dropping from the crystal
        for k in range(3):
            t = (f + k * 3) % 8
            sx = bx + 22 + (k - 1) * 3 + (t % 2)
            sy = by + 42 + t * 2
            if t < 7:
                px(d, sx, sy, a((255, 220, 150), 200 - t * 25))
    if rank >= 1:
        # Tear: a dim slit of rift light behind the shoulder
        g = 70 + 30 * math.sin(ph) if rank == 1 else 50
        line(d, [(bx + 12, by + 14), (bx + 11, by + 30), (bx + 12, by + 44)], a(RIFT, g), 2 if rank == 1 else 1)
    if rank >= 2:
        # Rift: a jagged rip with a little debris
        rx, ry = bx + 8, by + 30
        pts = [(rx, ry - 14), (rx + 3, ry - 8), (rx + 1, ry - 3), (rx + 4, ry + 3), (rx + 1, ry + 9), (rx, ry + 15),
               (rx - 2, ry + 8), (rx - 1, ry + 2), (rx - 3, ry - 4), (rx - 1, ry - 9)]
        poly(d, pts, fill=a(RIFT_DEEP, 180), outline=a(RIM, 200))
        line(d, [(rx, ry - 10), (rx, ry + 10)], a(RIFT, 200))
        for k in range(3):
            t = (f + k * 3) % 8
            px(d, rx - 4 - t, ry - 6 + k * 6 + (t % 3) - 1, a(RIM, 220 - t * 25))
    if rank >= 3:
        # Gate: a rune ring + the back half of the orbiting shards
        ring(d, cx, cy + 4, 27, a(GOLD, 90))
        for k in range(8):
            ang = ph / 8 + k * TAU / 8
            x, y = pt(cx, cy + 4, 27, ang)
            d.rectangle((round(x) - 1, round(y) - 1, round(x), round(y)), fill=a(GOLD, 200))
        shards_orbit(d, 2 if rank < 6 else 4, f, False, bx, by)
    if rank >= 4:
        # Wanderer: occasional star motes
        for k in range(6):
            t = (f + k * 5) % 8
            sx = cx + (hsh(3, k) - .5) * 90
            sy = cy + (hsh(4, k) - .5) * 90
            if t < 5:
                bright = [90, 200, 255, 200, 90][t]
                if t == 2:
                    sparkle(d, sx, sy, a(WHITE, bright), 1)
                else:
                    px(d, sx, sy, a(WHITE, bright))
    if rank >= 5:
        # Cartographer: a faint Hextech lattice behind her, one cell lit at a time
        gx, gy = cx + 14, cy - 18
        cells = hexes(d, gx, gy, 26, 26, 6, a((127, 240, 255), 30))
        q = cells[(f * 5) % len(cells)]
        hexagon(d, q[0], q[1], 6, a((127, 240, 255), 170))
    if rank >= 6:
        # Riftwalker: a slow parallax rift behind her
        rx, ry = cx - 30, cy - 6
        for layer, (dx, alpha, w) in enumerate(((math.sin(ph) * 2, 90, 9), (math.sin(ph) * 4, 170, 5))):
            pts = []
            for j in range(9):
                t = j / 8
                pts.append((rx + dx + w * math.sin(math.pi * t) + (2 if j % 2 else -1), ry - 30 + t * 60))
            for j in range(8, -1, -1):
                t = j / 8
                pts.append((rx + dx - w * math.sin(math.pi * t) + (1 if j % 2 else -2), ry - 30 + t * 60))
            poly(d, pts, fill=a(VOID if layer else RIFT_DEEP, alpha), outline=a(RIM, alpha))
        for k in range(4):
            t = (f + k * 2) % 8
            px(d, rx + 6 + t * 2, ry - 20 + k * 12 - t, a(RIM, 230 - t * 26))
            px(d, rx + 3 + k, ry - 8 + (k * 7 + f) % 30, a(WHITE, 200))
    # A moving measurement keeps every phase distinct after palette packing.
    ImageDraw.Draw(im).point((round(cx - 20 + f), round(cy + 30)), fill=a(RIFT_PALE, 200))
    return glow(im, 1.0, 1.4)


# ----------------------------------------------------------------- previews --

def previews(ctx):
    """Editor manifest, mastery sheet, portrait card and docs ladders, built from shipped frames."""
    vfx = ROOT / 'mods/tfm2_custom/vfx'
    docs, editor = ROOT / 'docs', ROOT / 'editor'
    bg = '#0b1020'
    AW, AH = AURA
    runtime, decoded = {}, {}
    for file in sorted(vfx.glob('vesper_*#anim.fanim')):
        name = file.name.split('#')[0]
        for tag, anim in json.loads(file.read_text(encoding='utf-8'))['anims'].items():
            runtime[tag] = {'sheet': name, 'frames': anim['frames']}

    def original(tag, f, w, h):
        asset = runtime[tag]
        name = asset['sheet']
        if name not in decoded:
            decoded[name] = Image.open(vfx / f'{name}#sheet.png').convert('RGBA')
        frames = asset['frames']
        rect = frames[f % len(frames)]['data']
        x, y, fw, fh = rect['x'], rect['y'], rect['w'], rect['h']
        im = Image.new('RGBA', (w, h))
        im.alpha_composite(decoded[name].crop((x, y, x + fw, y + fh)), ((w - fw) // 2, (h - fh) // 2))
        return im

    def title(d, x, y, s, size=13, color='#dcecf4'):
        d.text((x, y), s, font=font(size), fill=color)

    def big(im, k=2):
        return im.resize((im.width * k, im.height * k), Image.Resampling.NEAREST)

    def composed(r, f, podium=4, punching=False, emblem=False):
        look = 8 if r == 7 and podium == 1 else r
        btag = f'back_r7_p{podium}' if r == 7 else f'back_r{r}'
        im = original(btag, f, AW, AH)
        base = body('idle', f)
        base.alpha_composite(original(('punch_r' if punching else 'outfit_r') + str(look), f, 48, 64))
        im.alpha_composite(base, BODY)
        if emblem:
            tag = f'top{1 if podium == 1 else 2 if podium == 2 else 3 if podium == 3 else 10}' if r == 7 else f'rank{r}'
            im.alpha_composite(original(tag, f, *BADGE), (BODY[0] + 24 - BADGE[0] // 2, BODY[1] + 32 - BADGE[1] // 2))
        return im

    variants = [(r, 4) for r in range(7)] + [(7, 4), (7, 3), (7, 2), (7, 1)]
    labels = RANKS[:7] + ['Worldbreaker #4-10', '#3', '#2', '#1 ' + TOP_NAME]
    # editor manifest + browser-friendly composite sheet
    tiles = {}
    sheet = Image.new('RGBA', (8 * AW, (len(variants) + 1) * AH))
    for n, (r, p) in enumerate(variants):
        frames = []
        for f in range(8):
            x, y = f * AW, n * AH
            sheet.alpha_composite(composed(r, f, p), (x, y))
            frames.append({'x': x, 'y': y, 'w': AW, 'h': AH})
        tiles[f'{r}_{p}'] = frames
    n = len(variants)
    base = Image.new('RGBA', (AW, AH))
    base.alpha_composite(body('idle', 0), BODY)
    sheet.alpha_composite(base, (0, n * AH))
    sheet.save(editor / 'vesper-mastery-preview.png')
    prefixes = ('outfit_', 'punch_', 'rank', 'top', 'back_', 'orbs', 'hex', 'core', 'stress', 'rift_', 'glyph', 'link_',
                'packet_', 'open_', 'close_', 'break_', 'seedhint_', 'impact_', 'tech_', 'twist', 'hole_', 'swipe_',
                'shatter_', 'gbreak_', 'gfix_', 'awaken_', 'corrupt', 'overload_', 'mod', 'bolt')
    assets = {tag: asset for tag, asset in runtime.items() if tag.startswith(prefixes)}
    (editor / 'vesper-art-preview.json').write_text(json.dumps(
        {'outfits': tiles, 'labels': dict(zip(tiles, labels)), 'base': {'x': 0, 'y': n * AH, 'w': AW, 'h': AH},
         'body': list(BODY), 'assets': assets, 'frames': 8, 'frameSeconds': .12}, indent=2) + '\n')
    # portrait card: Spark, Riftwalker, Between Worlds
    portrait = Image.new('RGBA', (960, 360), bg)
    d = ImageDraw.Draw(portrait)
    cards = [(0, 4, 'SPARK', RIFT_PALE), (6, 4, 'RIFTWALKER', (190, 170, 255, 255)), (7, 1, 'BETWEEN WORLDS', GOLD)]
    for i, (r, p, name, col) in enumerate(cards):
        x = i * 320
        d.rounded_rectangle((x + 12, 12, x + 308, 348), 12, fill='#151a33', outline=col[:3], width=2)
        scene = composed(r, 2, p, punching=(i == 1), emblem=True).crop((BODY[0] - 24, BODY[1] - 28, BODY[0] + 72, BODY[1] + 76))
        portrait.alpha_composite(scene.resize((288, 312), Image.Resampling.NEAREST), (x + 16, 14))
        title(d, x + 25, 318, name, 18, col[:3])
    portrait.save(editor / 'vesper-portraits.png')
    # docs: rank ladder
    cw, ch = AW * 2 + 16, AH * 2 + 40
    im = Image.new('RGBA', (16 + 6 * cw, 90 + 2 * (ch + 40)), bg)
    d = ImageDraw.Draw(im)
    title(d, 24, 18, 'VESPER AND THE RIFT GAUNTLETS / MASTERY LADDER', 26)
    title(d, 24, 56, 'Every rank adds one readable layer; rank never adds power. Back aura + body + outfit + emblem, 2x, runtime frame 2.', 13)
    for j, ((r, p), label) in enumerate(zip(variants, labels)):
        row, col = divmod(j, 6)
        x, y = 16 + col * cw, 90 + row * (ch + 40)
        d.rounded_rectangle((x, y, x + cw - 8, y + ch), 8, fill='#151a33')
        im.alpha_composite(big(composed(r, 2, p, emblem=True)), (x, y + 4))
        title(d, x + 8, y + ch - 22, f'{r} {label}' if r < 7 else label, 12, '#ffd678' if p == 1 else '#dcecf4')
    im.convert('RGB').save(docs / 'vesper-mastery.png')
    # docs: rift-punch poses per rank
    im = Image.new('RGBA', (16 + 9 * 112, 80 + 140), bg)
    d = ImageDraw.Draw(im)
    title(d, 24, 16, 'RIFT PUNCH / THE LEFT GAUNTLET THRUST THROUGH A TEAR (2x, frame 3)', 18)
    for look in range(9):
        x = 16 + look * 112
        base = body('idle', 3)
        base.alpha_composite(original(f'punch_r{look}', 3, 48, 64))
        im.alpha_composite(big(base), (x + 6, 56))
        title(d, x + 4, 196, (RANKS + ['#1'])[look][:12], 11)
    im.convert('RGB').save(docs / 'vesper-gauntlets.png')
    # docs: animated ladder gif
    frames = []
    for f in range(8):
        sc = Image.new('RGBA', (4 * (AW * 2 + 8) + 16, AH * 2 + 70), bg)
        d = ImageDraw.Draw(sc)
        title(d, 16, 12, 'SPARK  /  GATE  /  RIFTWALKER  /  #1 BETWEEN WORLDS', 18)
        for i, (r, p) in enumerate([(0, 4), (3, 4), (6, 4), (7, 1)]):
            sc.alpha_composite(big(composed(r, f, p, emblem=True)), (16 + i * (AW * 2 + 8), 50))
        frames.append(sc.convert('RGB').quantize(colors=256))
    frames[0].save(docs / 'vesper-mastery.gif', save_all=True, append_images=frames[1:], duration=120, loop=0, disposal=2)
    return original
