"""Vesper, the Rift Marksman: body, rank outfits, aiming poses, emblems and auras.

Native pixels on the shared 48 x 64 anchor (body centre = canvas centre).
Her look goes from a scavenger with a jury-rigged rift rifle (Spark) to
someone who half-lives in another world (#1 Between Worlds). Every rank adds
one readable layer; rank never adds power. All drawing is deterministic:
eight discrete phases, no randomness (hash noise only).
"""
import json
import math
from functools import lru_cache
from pathlib import Path

from PIL import Image, ImageDraw, ImageFilter, ImageFont, ImageChops

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

PAL = [
    dict(coat=(123, 91, 62), shade=(86, 64, 44), light=(162, 124, 85), belt=(70, 50, 36)),     # Spark
    dict(coat=(110, 88, 66), shade=(76, 60, 46), light=(150, 122, 92), belt=(64, 48, 36)),     # Tear
    dict(coat=(98, 84, 70), shade=(68, 58, 48), light=(140, 122, 102), belt=(60, 46, 36)),     # Rift
    dict(coat=(62, 74, 102), shade=(43, 52, 73), light=(98, 114, 154), belt=(40, 36, 46)),     # Gate
    dict(coat=(58, 69, 96), shade=(40, 48, 68), light=(94, 108, 146), belt=(40, 36, 46)),      # Wanderer
    dict(coat=(52, 64, 90), shade=(36, 44, 64), light=(88, 104, 142), belt=(38, 34, 44)),      # Cartographer
    dict(coat=(46, 53, 80), shade=(30, 35, 56), light=(80, 92, 130), belt=(34, 30, 44)),       # Riftwalker
    dict(coat=(169, 191, 224), shade=(111, 134, 176), light=(232, 242, 255), belt=(70, 76, 112)),  # Worldbreaker
    dict(coat=(176, 196, 228), shade=(116, 138, 180), light=(255, 246, 220), belt=(120, 96, 52)),  # Between Worlds
]
for _p in PAL:
    for _k in list(_p):
        _p[_k] = _p[_k] + (255,)


def legs(d, kind='idle', f=0):
    """Shared moving legs and boots; every coat hem sits above y 50."""
    step = [1, 2, 1, 0, -1, -2, -1, 0][f] if kind == 'run' else 0
    if kind == 'attack':
        step = [0, 0, -1, -1, -1, 0, 0, 0][f]       # braced against recoil
    bdx, fdx = -step, step
    bl, fl = max(0, step), max(0, -step)
    for x0, dx, lift in ((19, bdx, bl), (25, fdx, fl)):
        d.rectangle((x0 + dx, 46, x0 + 3 + dx, 55 - lift), fill=INK)
        d.rectangle((x0 + 1 + dx, 46, x0 + 2 + dx, 54 - lift), fill=TROUSER if x0 == 19 else TROUSER_L)
        poly(d, [(x0 - 1 + dx, 55 - lift), (x0 + 3 + dx, 55 - lift), (x0 + 5 + dx, 57 - lift),
                 (x0 + 5 + dx, 58 - lift), (x0 - 1 + dx, 58 - lift)], fill=BOOT, outline=INK)
        line(d, [(x0 + dx, 56 - lift), (x0 + 3 + dx, 56 - lift)], BOOT_L)


def head(d, rank, f, aim=False):
    dx = 1 if aim else 0          # cheek weld when aiming
    d.rectangle((23 + dx, 30, 25 + dx, 33), fill=SKIN_D)
    if rank >= 1:                  # Tear: the hood
        hood = (59, 70, 82, 255) if rank < 7 else (150, 170, 210, 255)
        hood_l = (86, 102, 122, 255) if rank < 7 else (232, 240, 255, 255)
        poly(d, [(18, 22), (21, 18), (26, 18), (29 + dx, 20), (30 + dx, 24), (27 + dx, 22), (23 + dx, 23),
                 (22 + dx, 28), (22 + dx, 33), (18, 32), (16, 27)], fill=hood, outline=INK)
        line(d, [(21, 19), (26, 19)], hood_l)
        line(d, [(18, 24), (17, 28)], hood_l)
    else:
        poly(d, [(19, 22), (22, 20), (26, 20), (28 + dx, 22), (28 + dx, 25), (23 + dx, 26), (22 + dx, 31),
                 (19, 30), (18, 26)], fill=HAIR, outline=INK)
        poly(d, [(18, 24), (16, 26), (15, 30), (17, 31), (19, 28)], fill=HAIR, outline=INK)
        line(d, [(21, 21), (25, 21)], HAIR_L)
    poly(d, [(23 + dx, 23), (27 + dx, 23), (29 + dx, 25), (29 + dx, 28), (28 + dx, 30), (26 + dx, 31),
             (23 + dx, 31), (22 + dx, 27)], fill=SKIN, outline=INK)
    line(d, [(23 + dx, 22), (28 + dx, 22)], HAIR if rank == 0 else INK)
    px(d, 28 + dx, 29, SKIN_D)
    # one-lens goggles; the lens is cracked at Spark, repaired and glowing later
    line(d, [(19, 25), (24 + dx, 25)], (88, 64, 40, 255))
    d.rectangle((25 + dx, 24, 28 + dx, 27), fill=BRASS, outline=INK)
    lens = CRYSTAL if rank == 0 else (RIFT_PALE if rank < 7 else GOLD_WHITE if rank == 8 else (220, 236, 255, 255))
    d.rectangle((26 + dx, 25, 27 + dx, 26), fill=lens)
    if rank == 0:
        px(d, 26 + dx, 25, INK)                    # the crack
        px(d, 27 + dx, 26, (90, 140, 170, 255))
    elif rank >= 3:
        px(d, 27 + dx, 25, WHITE)                  # ring reticle glint


def torso(d, p, hem):
    poly(d, [(18, 33), (27, 33), (30, 36), (31, 42), (30, hem), (25, hem + 1), (23, hem - 1), (21, hem + 1),
             (16, hem), (16, 42), (17, 36)], fill=p['coat'], outline=INK)
    poly(d, [(26, 34), (29, 36), (30, 42), (29, hem - 1), (26, hem - 1), (25, 40)], fill=p['shade'])
    line(d, [(18, 35), (17, 42), (17, hem - 2)], p['light'])
    line(d, [(17, 45), (30, 45)], p['belt'])
    px(d, 23, 45, BRASS)
    line(d, [(21, 33), (25, 33)], p['light'])


def rifle_rest(d, rank, f):
    """Port arms: the crossbow-rifle crosses the chest, muzzle up-right."""
    poly(d, [(15, 47), (18, 44), (20, 46), (17, 49)], fill=WOOD, outline=INK)
    line(d, [(19, 45), (28, 36)], INK, 2)
    line(d, [(19, 44), (28, 35)], METAL)
    if rank >= 7:          # Worldbreaker: the rift lance
        line(d, [(28, 35), (35, 28)], (220, 232, 255, 255) if rank == 7 else GOLD_WHITE)
        line(d, [(29, 35), (35, 29)], INK)
        tip = GOLD if rank == 8 else RIFT_PALE
        line(d, [(36, 25 + f % 2), (37, 28), (36, 31 - f % 2)], tip)
        px(d, 36, 28, WHITE)
    else:
        line(d, [(28, 35), (32, 31)], METAL_L)
        line(d, [(27, 32), (28, 33), (30, 34), (31, 36)], WOOD_L)       # crossbow limbs
        if rank >= 5:
            for k in range(3):
                px(d, 20 + k * 2 + (f % 2), 43 - k * 2, GOLD)             # engraved barrel
        if rank >= 3:
            d.ellipse((31, 31, 33, 33), outline=GOLD)                      # ring sight


def crystal(d, rank, f, x=22, y=41):
    """The taped crystal: sputters at Spark, glows steadily from Tear."""
    col = CRYSTAL if rank < 8 else GOLD_WHITE
    if rank == 0:
        col = mix(CRYSTAL, (70, 90, 110), [0, .5, .1, .7, 0, .3, .6, .2][f])
    poly(d, [(x, y - 2), (x + 2, y), (x, y + 2), (x - 2, y)], fill=col, outline=INK)
    if rank == 0:
        line(d, [(x - 2, y + 1), (x + 1, y - 2)], TAPE)
    else:
        px(d, x, y - 1, WHITE)


def arms_rest(d, p):
    poly(d, [(26, 33), (29, 34), (30, 37), (28, 38), (26, 36)], fill=p['coat'], outline=INK)
    d.rectangle((27, 37, 28, 38), fill=SKIN)
    poly(d, [(17, 38), (20, 40), (20, 43), (18, 43)], fill=p['shade'], outline=INK)
    d.rectangle((19, 43, 20, 44), fill=SKIN)


HEM = [49, 49, 49, 53, 53, 53, 53, 53, 53]


def figure(look, f):
    """The opaque rest silhouette in a rank palette (head, coat, arms, rifle)."""
    im = Image.new('RGBA', (48, 64))
    d = ImageDraw.Draw(im)
    p = PAL[look]
    torso(d, p, HEM[0])
    rifle_rest(d, 0, f)
    arms_rest(d, p)
    head(d, 0, f)
    return im


@lru_cache(maxsize=None)
def base_mask():
    """Union of every rest-pose pixel the body sheet draws above the legs."""
    m = Image.new('L', (48, 64))
    for f in range(8):
        m = ImageChops.lighter(m, figure(0, f).getchannel('A'))
    return m.point(lambda v: 255 if v else 0)


def shards_orbit(d, n, f, front, ox=0, oy=0, rx=17, ry=5, cy=40, size=1.4, look=7):
    """n small rift shards orbiting the waist. Front half in the outfit, back half in the aura."""
    for k in range(n):
        ang = (f / 8) * TAU / n + k * TAU / n + .3
        s = math.sin(ang)
        if (s > 0) != front:
            continue
        x, y = 24 + rx * math.cos(ang) + ox, cy + ry * s + oy - 2 * math.cos(ang * 2)
        col = (196, 206, 255, 255) if look < 7 else (222, 236, 255, 255)
        shard(d, x, y, size if front else size * .9, col if front else mix(col, VOID, .25))


def costume(look, f, aim=False):
    """Rank layers on top of the shared silhouette (look 0..8)."""
    rank = min(look, 7)
    im = Image.new('RGBA', (48, 64))
    d = ImageDraw.Draw(im)
    p = PAL[look]
    hem = HEM[look]
    ph = f * TAU / 8
    if rank >= 4:      # Wanderer: the travel cloak hangs behind, lining shows a starfield
        poly(d, [(18, 33), (14, 38), (12, 47), (13, hem + 1), (17, hem), (17, 36)], fill=(28, 30, 58, 255), outline=INK)
        for k in range(4):
            sx = 13 + (k * 2 + f) % 4
            sy = 38 + (k * 5 + f * 2) % max(1, hem - 39)
            px(d, sx, sy, (230, 236, 255, 255) if (k + f) % 3 else RIFT_PALE)
    if aim:
        # Cover the rest-pose silhouette before drawing the planted aiming pose.
        cover = Image.new('RGBA', (48, 64), p['shade'])
        cover.putalpha(base_mask())
        im.alpha_composite(cover)
        d = ImageDraw.Draw(im)
        legs(d, 'idle', 0)
        # braced boots, planted wider than the idle stance
        poly(d, [(16, 56), (20, 56), (21, 58), (16, 58)], fill=BOOT, outline=INK)
        poly(d, [(27, 56), (31, 56), (32, 58), (27, 58)], fill=BOOT, outline=INK)
    torso(d, p, hem)
    if rank >= 7:      # Worldbreaker: rift-glass armour facets
        for (x0, y0, x1, y1) in [(19, 35, 22, 40), (22, 40, 19, 46), (26, 36, 28, 42), (20, 47, 24, 52), (27, 46, 25, 52)]:
            line(d, [(x0, y0), (x1, y1)], WHITE)
        line(d, [(18, 41), (29, 39)], p['shade'])
    if rank >= 3:      # Gate: long coat with gate-ring sigils
        for sx, sy in ((19, 49), (27, 48)):
            d.ellipse((sx - 1, sy - 1, sx + 1, sy + 1), outline=GOLD if look != 8 else GOLD_WHITE)
        px(d, 20 + (f % 4 == 0), 49, WHITE)
    if rank == 0:      # Spark: patches on the traveller's coat
        d.rectangle((18, 39, 20, 41), fill=(95, 107, 72, 255), outline=INK)
        d.rectangle((26, 46, 28, 48), fill=(140, 110, 74, 255))
        px(d, 19, 40, (130, 140, 100, 255))
    if rank >= 2:      # Rift: strain gauge on the belt
        d.ellipse((17, 44, 20, 47), fill=(30, 26, 36, 255), outline=BRASS)
        needle = [(18, 45), (19, 45), (19, 46), (19, 46), (20, 46), (19, 46), (19, 45), (18, 45)][f]
        line(d, [(18, 46), needle], (255, 120, 90, 255))
    if rank >= 4:      # Wanderer: atlas compass at the hip
        d.ellipse((27, 47, 30, 50), fill=(36, 30, 26, 255), outline=BRASS)
        nx, ny = pt(28.5, 48.5, 1.6, ph)
        line(d, [(28.5, 48.5), (nx, ny)], RIFT_PALE)
    if rank >= 5:      # Cartographer: map-scroll sash with a glowing grid
        line(d, [(18, 34), (29, 46)], (227, 211, 166, 255), 2)
        for k in range(5):
            gx, gy = 19 + k * 2, 35 + k * 2
            px(d, gx + 1, gy, (127, 240, 255, 255) if (k + f) % 5 else WHITE)
        d.rectangle((14, 42, 18, 47), fill=(227, 211, 166, 255), outline=INK)
        for gx in range(15, 18):
            for gy in range(43, 47):
                if (gx + gy) % 2 == 0:
                    px(d, gx, gy, (90, 200, 220, 255))
        px(d, 15 + f % 3, 43 + (f // 3) % 4, WHITE)
    if rank >= 6:      # Riftwalker: the hem dissolves into rift fragments
        for k in range(5):
            fx_ = 17 + k * 3
            fy = hem + 1 + (f + k * 3) % 5
            col = (190, 170, 255, 255) if look != 8 else GOLD_WHITE
            if fy <= 60:
                d.rectangle((fx_, fy, fx_ + (k % 2), fy), fill=a(col, 255 - (fy - hem) * 30))
    if rank >= 2:      # Rift: shoulder mantle with dimension-pocket patches
        mantle = (75, 63, 94, 255) if rank < 7 else (126, 146, 196, 255)
        poly(d, [(17, 34), (21, 32), (27, 32), (31, 35), (30, 38), (26, 37), (22, 38), (17, 38)], fill=mantle, outline=INK)
        for k, (mx, my) in enumerate(((20, 35), (27, 35))):
            d.rectangle((mx - 1, my - 1, mx + 1, my), fill=VOID)
            px(d, mx - 1 + (f + k) % 3, my - (f // 2 + k) % 2, FAMILY[(f + k * 3) % 10])
    # the rifle and arms
    if aim:
        poly(d, [(17, 37), (22, 36), (22, 39), (17, 40)], fill=WOOD, outline=INK)
        d.rectangle((22, 36, 34, 38), fill=INK)
        line(d, [(22, 37), (34, 37)], METAL)
        if rank >= 7:
            line(d, [(34, 37), (44, 37)], (220, 232, 255, 255) if look == 7 else GOLD_WHITE)
            line(d, [(34, 38), (43, 38)], INK)
            tip = GOLD if look == 8 else RIFT_PALE
            line(d, [(45, 34 + f % 2), (46, 37), (45, 40 - f % 2)], tip)
            px(d, 46, 37, WHITE)
        else:
            line(d, [(34, 37), (40, 37)], METAL_L)
            line(d, [(35, 33), (36, 35), (36, 39), (35, 41)], WOOD_L)       # limbs
            if rank >= 5:
                for k in range(4):
                    px(d, 24 + k * 3 + (f % 2), 37, GOLD)
            if rank >= 3:
                d.ellipse((37, 33, 39, 35), outline=GOLD)
        # the crystal housing covers the rest pose's muzzle
        d.rectangle((27, 32, 32, 35), fill=WOOD if rank < 7 else (126, 146, 196, 255), outline=INK)
        crystal(d, look, f, 29, 33)
        poly(d, [(25, 34), (28, 35), (33, 37), (33, 39), (28, 39), (25, 37)], fill=p['coat'], outline=INK)
        d.rectangle((33, 37, 34, 39), fill=SKIN)
        poly(d, [(18, 37), (22, 38), (25, 39), (24, 41), (19, 41)], fill=p['shade'], outline=INK)
        d.rectangle((24, 38, 25, 39), fill=SKIN)
    else:
        rifle_rest(d, rank, f)
        crystal(d, look, f)
        arms_rest(d, p)
    if rank >= 1:      # Tear: one glowing seam stitched down the sleeve
        seam = RIFT if look != 8 else GOLD
        pts = [(26, 34), (27, 35), (28, 36)] if not aim else [(27, 35), (29, 36), (31, 37)]
        for k, q in enumerate(pts):
            px(d, *q, WHITE if k == f % 3 else seam)
    if rank >= 6:      # Riftwalker: the left arm phased into another dimension
        arm = Image.new('RGBA', (48, 64))
        ad = ImageDraw.Draw(arm)
        if aim:
            poly(ad, [(18, 37), (22, 38), (25, 39), (24, 41), (19, 41)], fill=p['shade'], outline=INK)
            ad.rectangle((24, 38, 25, 39), fill=SKIN)
        else:
            poly(ad, [(17, 38), (20, 40), (20, 43), (18, 43)], fill=p['shade'], outline=INK)
            ad.rectangle((19, 43, 20, 44), fill=SKIN)
        im.alpha_composite(negative(arm))
        d = ImageDraw.Draw(im)
        y = 38 + f % 6
        px(d, 17 if not aim else 19, y, (255, 255, 255, 255))
    head(d, rank if look != 8 else 8, f, aim)
    if look < 7 and rank >= 3:
        shards_orbit(d, 2 if rank < 6 else 4, f, True, look=look)
    if look == 7:
        shards_orbit(d, 6, f, True, look=7)
    if rank >= 7:      # crown of floating shards
        for k in range(5):
            x = 19 + k * 2.5
            y = 16 - (2 if k == 2 else 1 if k in (1, 3) else 0) + (1 if (k + f) % 4 < 2 else 0)
            shard(d, x, y, 1, GOLD_WHITE if look == 8 else (220, 232, 255, 255), outline=INK if look < 8 else (120, 90, 40, 255))
    if aim and rank == 3:
        # Gate: a faint ring under her feet while aiming
        ring(d, 24, 58, 13, a(GOLD, 70 + 20 * (f % 2)), .3)
    if aim and rank >= 5 and look < 8:
        # Cartographer: a small atlas grid projects around her feet
        for k in range(-3, 4):
            line(d, [(24 + k * 5 - 6, 61), (24 + k * 5 + 6, 55)], a((127, 240, 255), 40))
            line(d, [(10, 58 + k * .9), (38, 58 + k * .9)], a((127, 240, 255), 18 if k % 2 else 34))
        px(d, 12 + f * 3, 58, a(WHITE, 200))
    if rank == 0:
        # sputtering sparks off the taped crystal
        cx, cy = (29, 33) if aim else (22, 41)
        for k in range(2):
            if (f + k) % 3 != 2:
                sx, sy = pt(cx, cy, 3 + (f + k * 2) % 3, hsh(7, f, k) * TAU)
                px(d, sx, sy, (255, 236, 160, 255) if k else CRYSTAL)
    elif look < 8:
        cx, cy = (29, 33) if aim else (22, 41)
        px(d, *pt(cx, cy, 3, ph), a(CRYSTAL, 200))   # steady glow
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
        if aim:
            for k in range(-3, 4):
                line(d, [(10, 58 + k * .9), (38, 58 + k * .9)], a(GOLD, 26 if k % 2 else 44))
            px(d, 12 + f * 3, 58, a(GOLD_WHITE, 220))
    return im


def outfit(look, f):
    return costume(look, f, False)


def aim(look, f):
    return costume(look, f, True)


def body(kind='idle', f=0):
    """The registered body sheet: rank-0 look, eight frames per animation."""
    im = Image.new('RGBA', (48, 64))
    d = ImageDraw.Draw(im)
    if kind == 'dead':
        poly(d, [(10, 54), (14, 50), (32, 50), (36, 53), (33, 58), (12, 58)], fill=INK)
        d.rectangle((14, 51, 31, 55), fill=PAL[0]['coat'])
        d.rectangle((15, 54, 30, 55), fill=PAL[0]['shade'])
        poly(d, [(32, 50), (37, 49), (39, 52), (36, 55), (32, 55)], fill=SKIN, outline=INK)
        poly(d, [(33, 49), (37, 48), (39, 50), (34, 51)], fill=HAIR)
        line(d, [(8, 58), (26, 58)], INK, 2)
        line(d, [(9, 57), (25, 57)], METAL)
        dim = [255, 200, 150, 110, 80, 55, 35, 20][f]
        poly(d, [(18, 55), (20, 57), (18, 59), (16, 57)], fill=a(CRYSTAL, dim), outline=INK)
        if f < 6:
            px(d, 18 + (f % 3) - 1, 53 - f // 2, a((255, 236, 160), 200 - f * 30))
        return im
    legs(d, kind, f)
    im.alpha_composite(outfit(0, f))
    d = ImageDraw.Draw(im)
    if kind == 'attack':
        if 1 <= f <= 4:
            r = [0, 2, 4, 3, 1][f]
            dot(d, 34, 30, r, a((255, 240, 190), 255))
            for k in range(4):
                line(d, [pt(34, 30, r + 1, k * TAU / 4 - TAU / 8 + f), pt(34, 30, r + 3, k * TAU / 4 - TAU / 8 + f)], CRYSTAL)
        else:
            px(d, 34 + f % 3, 29 - f % 2, a((200, 200, 210), 160))
    elif kind == 'skill1':
        for k in range(1 + f // 2):
            sx, sy = pt(22, 41, 3 + f * .7, k * TAU / 5 + f * .4)
            px(d, sx, sy, CRYSTAL if k % 2 else (255, 236, 160, 255))
        if f >= 3:
            h = (f - 2) * 3
            line(d, [(41, 52 - h), (42, 52 - h // 2), (41, 52)], RIFT_PALE)
    elif kind == 'skill2':
        for k in range(5):
            x = 33 + k * 3 + (f % 3)
            if (k + f) % 2 == 0 and x < 47:
                px(d, x, 31 - k, RIFT_PALE)
    elif kind == 'ult':
        r = 2 + f
        for k in range(6):
            line(d, [pt(22, 41, r, k * TAU / 6 + f * .2), pt(22, 41, r + 2, k * TAU / 6 + f * .2)], a(CRYSTAL, 255 - f * 20))
        line(d, [(22, 12), (24, 15), (23, 17)], a(RIFT_PALE, 120 + f * 15))
    elif kind == 'hit':
        if f < 4:
            m = im.getchannel('A').filter(ImageFilter.MaxFilter(3))
            edge = ImageChops.subtract(m, im.getchannel('A'))
            flash = Image.new('RGBA', (48, 64), (255, 255, 255, 255))
            flash.putalpha(edge.point(lambda v: v * (4 - f) // 4))
            im.alpha_composite(flash)
            d = ImageDraw.Draw(im)
        px(d, 30 + f, 36 - f // 2, a((255, 200, 160), 255 - f * 28))
    elif kind == 'idle':
        px(d, 13 + f, 60, a(CRYSTAL, 60))
    elif kind == 'run':
        px(d, 12 - f % 4, 57 - f // 4, a((150, 130, 110), 120))
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
                # the atlas grid glows through the cracks
                grid = Image.new('RGBA', AURA)
                gd = ImageDraw.Draw(grid)
                for k in range(-6, 7):
                    line(gd, [(hub[0] + k * 9, hub[1] - 60), (hub[0] + k * 9, hub[1] + 60)], a((127, 240, 255), 255))
                    line(gd, [(hub[0] - 70, hub[1] + k * 9), (hub[0] + 70, hub[1] + k * 9)], a((127, 240, 255), 255))
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
        # Cartographer: a faint atlas plane behind her
        gx, gy = cx + 14, cy - 18
        for k in range(11):
            line(d, [(gx - 25 + k * 5, gy - 25), (gx - 25 + k * 5, gy + 25)], a((127, 240, 255), 26))
            line(d, [(gx - 25, gy - 25 + k * 5), (gx + 25, gy - 25 + k * 5)], a((127, 240, 255), 26))
        cell = (f * 3) % 10, (f * 7) % 10
        d.rectangle((gx - 25 + cell[0] * 5, gy - 25 + cell[1] * 5, gx - 20 + cell[0] * 5, gy - 20 + cell[1] * 5), outline=a((127, 240, 255), 160))
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

    def composed(r, f, podium=4, aiming=False, emblem=False):
        look = 8 if r == 7 and podium == 1 else r
        btag = f'back_r7_p{podium}' if r == 7 else f'back_r{r}'
        im = original(btag, f, AW, AH)
        base = body('idle', f)
        base.alpha_composite(original(('aim_r' if aiming else 'outfit_r') + str(look), f, 48, 64))
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
    prefixes = ('outfit_', 'aim_', 'rank', 'top', 'back_', 'charges', 'strain', 'rift_', 'core_', 'glyph', 'link_',
                'packet_', 'open_', 'close_', 'break_', 'seedhint_', 'impact_', 'tech_', 'twist', 'breach_', 'atlas_warn', 'bolt')
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
        scene = composed(r, 2, p, aiming=(i == 1), emblem=True).crop((BODY[0] - 24, BODY[1] - 28, BODY[0] + 72, BODY[1] + 76))
        portrait.alpha_composite(scene.resize((288, 312), Image.Resampling.NEAREST), (x + 16, 14))
        title(d, x + 25, 318, name, 18, col[:3])
    portrait.save(editor / 'vesper-portraits.png')
    # docs: rank ladder
    cw, ch = AW * 2 + 16, AH * 2 + 40
    im = Image.new('RGBA', (16 + 6 * cw, 90 + 2 * (ch + 40)), bg)
    d = ImageDraw.Draw(im)
    title(d, 24, 18, 'VESPER, THE RIFT MARKSMAN / MASTERY LADDER', 26)
    title(d, 24, 56, 'Every rank adds one readable layer; rank never adds power. Back aura + body + outfit + emblem, 2x, runtime frame 2.', 13)
    for j, ((r, p), label) in enumerate(zip(variants, labels)):
        row, col = divmod(j, 6)
        x, y = 16 + col * cw, 90 + row * (ch + 40)
        d.rounded_rectangle((x, y, x + cw - 8, y + ch), 8, fill='#151a33')
        im.alpha_composite(big(composed(r, 2, p, emblem=True)), (x, y + 4))
        title(d, x + 8, y + ch - 22, f'{r} {label}' if r < 7 else label, 12, '#ffd678' if p == 1 else '#dcecf4')
    im.convert('RGB').save(docs / 'vesper-mastery.png')
    # docs: aiming poses per rank
    im = Image.new('RGBA', (16 + 9 * 112, 80 + 140), bg)
    d = ImageDraw.Draw(im)
    title(d, 24, 16, 'AIMING POSES / PLANTED FEET, RIFLE RAISED (2x, frame 3)', 18)
    for look in range(9):
        x = 16 + look * 112
        base = body('idle', 3)
        base.alpha_composite(original(f'aim_r{look}', 3, 48, 64))
        im.alpha_composite(big(base), (x + 6, 56))
        title(d, x + 4, 196, (RANKS + ['#1'])[look][:12], 11)
    im.convert('RGB').save(docs / 'vesper-aiming.png')
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
