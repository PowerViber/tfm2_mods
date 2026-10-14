"""Vesper and the Rift Gauntlets: body, rank outfits, animated pose overlays, emblems and auras.

Native pixels on the shared 48 x 64 anchor (body centre = canvas centre).
Her look goes from a scavenger with taped orbs in salvaged gauntlets (Spark)
to someone who half-lives in another world (#1 Between Worlds): a lean street
fighter with broken glasses, a bronze half-cloak and two enormous gauntlets. Every rank adds
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
# Vesper, 3/4 front-right, in a boxer's guard. Facing +x: the LEFT gauntlet (pure orbs) is the lead fist,
# held in front of her chest; the RIGHT gauntlet (Hexcore) is chambered low beside her waist, partly behind
# her. Lean and battle-worn: messy dark hair, black glasses with one cracked lens, a fitted dark undershirt
# under a sleeveless brown tunic, a short bronze half-cloak trailing off the back shoulder.
#
# Animation-ready: every pose is assembled from the same layers (cloak, rear gauntlet, torso, head, lead
# gauntlet) at the anchors in RIG, so height, hands and cloak stay consistent. The body sheet keeps one
# upper-body silhouette in every frame (only the legs move); every overlay frame covers it (seal()).

PAL = [   # the tunic: brown throughout, the seams turn violet and cyan as she ranks up
    dict(coat=(123, 91, 62), shade=(86, 64, 44), light=(162, 124, 85), belt=(70, 50, 36), lining=(70, 46, 96)),      # Spark
    dict(coat=(118, 86, 58), shade=(82, 60, 42), light=(158, 120, 80), belt=(66, 48, 34), lining=(76, 46, 108)),     # Tear
    dict(coat=(112, 80, 55), shade=(78, 56, 40), light=(152, 114, 76), belt=(62, 44, 32), lining=(82, 48, 118)),     # Rift
    dict(coat=(106, 74, 52), shade=(72, 50, 37), light=(146, 108, 74), belt=(58, 42, 32), lining=(86, 50, 128)),     # Gate
    dict(coat=(100, 69, 50), shade=(68, 47, 36), light=(140, 102, 72), belt=(54, 40, 32), lining=(90, 52, 136)),     # Wanderer
    dict(coat=(94, 64, 48), shade=(64, 44, 35), light=(134, 98, 70), belt=(50, 38, 32), lining=(94, 54, 144)),       # Cartographer
    dict(coat=(86, 58, 46), shade=(58, 40, 34), light=(126, 92, 70), belt=(46, 36, 32), lining=(100, 58, 156)),      # Riftwalker
    dict(coat=(72, 48, 42), shade=(46, 32, 30), light=(150, 128, 170), belt=(40, 32, 36), lining=(122, 76, 196)),    # Worldbreaker
    dict(coat=(110, 76, 50), shade=(76, 52, 36), light=(220, 186, 130), belt=(120, 96, 52), lining=(200, 160, 90)),  # Between Worlds
]
for _p in PAL:
    for _k in list(_p):
        _p[_k] = _p[_k] + (255,)

HEX = (110, 240, 255, 255)          # Hextech cyan: only a few pixels per gauntlet
HEX_D = (40, 120, 150, 255)
SHIRT = (46, 44, 58, 255)           # fitted dark undershirt
SHIRT_L = (74, 72, 92, 255)
HAIR = (36, 28, 38, 255)
HAIR_L = (72, 58, 76, 255)
LENS = (196, 214, 226, 255)
LENS_CYAN = (86, 196, 214, 255)
EYE = (34, 26, 30, 255)
CLOAK = dict(deep=(101, 69, 41, 255), warm=(164, 108, 56, 255), hi=(214, 164, 106, 255), trim=(185, 196, 204, 255))
CLOAK_TOP = dict(CLOAK, trim=GOLD)  # #1: gold trim


def _steel(metal, dark, light, trim, trim_l, trim_d):
    return dict(metal=metal + (255,), dark=dark + (255,), light=light + (255,), trim=trim + (255,),
                trim_l=trim_l + (255,), trim_d=trim_d + (255,))


STEEL = _steel((78, 84, 98), (46, 50, 62), (140, 148, 164), (166, 106, 56), (214, 150, 92), (104, 64, 34))
GLASS = _steel((128, 146, 192), (76, 84, 132), (228, 238, 255), (176, 150, 232), (255, 255, 255), (96, 76, 150))
GILT = _steel((214, 186, 120), (138, 106, 56), (255, 244, 206), (236, 196, 104), (255, 252, 236), (150, 108, 44))


def steel(look):
    return GILT if look == 8 else GLASS if look == 7 else STEEL


# Anchors per pose: fist placements are (orientation, x, y, housing length) for the gauntlet() frame.
#   up: knuckles up (guard)   right: knuckles forward (+x)   down: knuckles down (hanging / ripping)
RIG = {
    'guard': dict(lead=('up', 30, 26, 4), rear=('right', -4, 35, 5)),
}


def legs(d, kind='idle', f=0):
    """Staggered, bent-kneed legs: back foot left, front foot forward, both on y 58."""
    if kind == 'run':
        s = [0, 1, 2, 1, 0, -1, -2, -1][f]
    elif kind == 'attack':
        s = [0, 1, 1, 1, 0, 0, 0, 0][f]
    else:
        s = 0
    back = [(21, 42), (18 - s, 49), (18 - s * 2, 55)]
    front = [(26, 42), (30 + s, 48), (30 + s * 2, 55)]
    for pts, col, lift in ((back, TROUSER, max(0, s)), (front, TROUSER_L, max(0, -s))):
        hip, knee, ankle = pts
        ankle = (ankle[0], ankle[1] - lift)
        for a_, b_ in ((hip, knee), (knee, ankle)):
            line(d, [a_, b_], INK, 4)
        for a_, b_ in ((hip, knee), (knee, ankle)):
            line(d, [a_, b_], col, 2)
        px(d, knee[0] + 1, knee[1], TROUSER_L if col == TROUSER else (120, 116, 134, 255))
        x, y = ankle
        poly(d, [(x - 2, y), (x + 2, y), (x + 5, y + 2), (x + 5, y + 3), (x - 2, y + 3)], fill=BOOT, outline=INK)
        line(d, [(x - 1, y + 1), (x + 2, y + 1)], BOOT_L)


def hair(d, look, f):
    """Dark, messy, medium-short hair swept back: spikes trail behind her head."""
    sway = 1 if f in (2, 3, 4) else 0
    poly(d, [(19, 15), (21, 13), (25, 12), (28, 13), (31, 15), (31, 18), (28, 17), (24, 18), (23, 21),
             (22, 25), (19, 24), (17, 22), (15 - sway, 21), (17, 19), (14 - sway, 17), (17, 16)], fill=HAIR, outline=INK)
    poly(d, [(22, 12), (23, 10), (24, 12)], fill=HAIR, outline=INK)          # a messy crown spike
    line(d, [(20, 15), (24, 13), (28, 14)], HAIR_L)
    px(d, 18, 19, HAIR_L)
    px(d, 29, 17, HAIR)                                                       # a strand over the brow
    px(d, 30, 18, HAIR)


def face(d, look, f):
    d.rectangle((25, 26, 27, 29), fill=SKIN_D)                               # neck
    poly(d, [(23, 17), (29, 17), (31, 19), (32, 21), (31, 23), (31, 25), (29, 27), (25, 27), (22, 25), (21, 21)],
         fill=SKIN, outline=INK)
    line(d, [(23, 22), (23, 24)], SKIN_D)                                     # the far cheek in shadow
    px(d, 24, 26, SKIN_D)
    line(d, [(28, 25), (29, 25)], INK)                                        # a smirk
    px(d, 30, 24, INK)
    if look == 0:
        line(d, [(26, 24), (27, 23)], TAPE)                                   # Spark: a plaster on the cheek


def glasses(d, look, f):
    """Rectangular black frames: the near lens cracked, the far lens tinted cyan, both eyes visible."""
    d.rectangle((22, 19, 25, 22), outline=INK)
    d.rectangle((27, 19, 31, 22), outline=INK)
    px(d, 26, 20, INK)                                                        # the bridge
    line(d, [(20, 20), (22, 20)], INK)                                        # the temple arm into her hair
    d.rectangle((23, 20, 24, 21), fill=LENS_CYAN)
    px(d, 24, 21, mix(EYE, LENS_CYAN, .35))
    d.rectangle((28, 20, 30, 21), fill=LENS)
    px(d, 29, 21, EYE)
    crack = WHITE if look < 8 else GOLD_WHITE
    px(d, 28, 20, crack)                                                      # the crack: two bright diagonal pixels
    px(d, 30, 21, crack if f % 4 else mix(LENS, WHITE, .3))
    px(d, 29, 20, mix(LENS, WHITE, .5))


def torso(d, p, look, f):
    """Fitted dark undershirt, a sleeveless brown tunic, the slim waist; no armour."""
    # undershirt: shoulders and upper arms
    poly(d, [(18, 29), (24, 28), (30, 28), (33, 30), (33, 34), (30, 33), (19, 34), (16, 32)], fill=SHIRT, outline=INK)
    line(d, [(19, 30), (23, 29)], SHIRT_L)
    # the tunic, turned toward the front-right
    poly(d, [(20, 30), (24, 31), (29, 30), (30, 33), (29, 37), (28, 39), (29, 42), (29, 44), (25, 45), (23, 43),
             (20, 44), (20, 42), (21, 39), (20, 36), (19, 33)], fill=p['coat'], outline=INK)
    poly(d, [(26, 32), (29, 31), (28, 37), (27, 40), (28, 43), (26, 43), (25, 39)], fill=p['shade'])
    line(d, [(21, 32), (21, 36), (22, 39)], p['light'])
    poly(d, [(23, 30), (25, 30), (24, 33)], fill=SHIRT)                       # the collar V
    line(d, [(21, 40), (28, 40)], p['belt'])
    px(d, 24, 40, BRASS)


def cloak_back(d, look, f, top=False):
    """The half-cloak's trailing part, blown back behind her shoulders. Its tip only ever moves outward."""
    C = CLOAK_TOP if top else CLOAK
    sway = [0, 1, 1, 2, 2, 1, 1, 0][f]
    poly(d, [(18, 28), (23, 28), (21, 31), (18, 35), (16, 40), (13, 41 + (sway > 1)), (9 - sway, 39), (5 - sway, 35 - sway // 2),
             (8, 32), (12, 29)], fill=C['deep'], outline=INK)
    line(d, [(13, 30), (9, 33), (7 - sway, 35)], C['warm'])
    line(d, [(17, 30), (14, 34), (12, 38)], C['warm'])
    px(d, 6 - sway, 35 - sway // 2, C['trim'])
    if look == 0:     # Spark: a tattered edge
        px(d, 8 - sway, 38, (0, 0, 0, 0))
        px(d, 9 - sway, 39, (0, 0, 0, 0))


def cloak_mantle(d, look, f, top=False):
    """Over the back shoulder only, fastened at the collar; the chest and arms stay clear."""
    C = CLOAK_TOP if top else CLOAK
    poly(d, [(15, 30), (18, 27), (23, 27), (26, 29), (24, 31), (20, 32), (16, 33)], fill=C['warm'], outline=INK)
    line(d, [(18, 28), (22, 28)], C['hi'])
    line(d, [(17, 32), (20, 31), (23, 30)], C['trim'])                        # chrome trim on the edge
    px(d, 25, 29, C['trim'] if not top else GOLD_WHITE)                       # the clasp


def gauntlet(look, f, side, orient='up', hlen=10, big=False):
    """One Rift Gauntlet as a local plan (u along the arm from elbow to knuckles, v across), mapped to the
    screen for its orientation and lit from the top-left in screen space. Returns (layer, origin)
    where origin is the screen offset of local (0, 0)."""
    S = steel(look)
    rank = min(look, 7)
    W = 14 if big else 13
    L0 = 10 - hlen                     # housing rows actually drawn (foreshortened forearms skip the elbow end)
    rects = []                         # (u0, v0, u1, v1, colour, plate?)

    def R(u0, v0, u1, v1, col, plate=False):
        rects.append((u0, v0, u1, v1, col, plate))
    # housing: wide at the elbow, narrower at the wrist
    for u in range(L0, 10):
        inset = 1 + (u >= 6) + (u >= 8)
        R(u, inset, u, W - 1 - inset, S['dark'])
    if hlen >= 3:
        R(L0, 2, L0, W - 3, S['metal'])                                       # top plate edge
        R(min(9, L0 + 3), 3, min(9, L0 + 3), W - 4, INK)                      # overlapping plates
    R(10, 1, 11, W - 2, S['trim'])                                            # bronze cuff
    R(11, 1, 11, W - 2, S['trim_d'])
    # fist
    R(12, 0, 21, W - 1, S['metal'], True)
    R(12, 0, 12, W - 1, INK)
    R(15, 0, 15, W - 1, INK)                                                  # finger joint line
    fingers = [(1, 2), (4, 5), (7, 8), (10, W - 2)]
    for k, (a0, a1) in enumerate(fingers):
        R(16, a0, 17, a1, S['dark'] if k % 2 else S['metal'], True)          # finger segment
        R(18, a0 - 1, 22, a1 + 1, INK)
        R(18, a0, 21, a1, S['trim'], True)                                    # square knuckle plate
        if k < 3:
            R(13, a1 + 1, 17, a1 + 1, INK)
    R(13, 0, 17, 0, INK)
    R(13, 1, 14, 4, S['trim_d'], True)                                        # thumb plate across the first fingers
    R(15, 1, 15, 4, INK)
    # Hextech accents
    glow_px = HEX if look < 8 else GOLD_WHITE
    R(15, 9, 15, 9, glow_px)
    if rank >= 3:
        R(15, 6, 15, 6, glow_px)
    if side == 'L' and hlen >= 4:      # five pure-orb sockets up the housing
        for k in range(5):
            u = 9 - k * max(1, (hlen - 1)) // 5
            if u < L0:
                continue
            col = FAMILY[(k * 2 + f // 4) % 10] if rank else (RIFT_PALE if (k + f) % 4 else (90, 80, 120, 255))
            R(u, W - 4, u, W - 4, col)
    # map to the screen
    if orient == 'up':
        T = lambda u, v: (v, 22 - u)
    elif orient == 'down':
        T = lambda u, v: (v, u)
    else:                               # right
        T = lambda u, v: (u, v)
    size = (W + 4, 27) if orient in ('up', 'down') else (27, W + 4)
    im = Image.new('RGBA', size)
    d = ImageDraw.Draw(im)
    o = 2
    for u0, v0, u1, v1, col, plate in rects:
        x0, y0 = T(u0, v0)
        x1, y1 = T(u1, v1)
        x0, x1 = sorted((x0, x1))
        y0, y1 = sorted((y0, y1))
        d.rectangle((x0 + o, y0 + o, x1 + o, y1 + o), fill=col)
        if plate and (x1 - x0 >= 1 or y1 - y0 >= 1):
            lite = S['light'] if col in (S['metal'], S['dark']) else S['trim_l'] if col == S['trim'] else S['trim']
            line(d, [(x0 + o, y0 + o), (x1 + o, y0 + o)], lite)               # light from the top
            if x1 - x0 >= 2:
                px(d, x0 + o, y0 + o + 1, lite)                               # and a touch from the left
    # the outline: one pixel of ink round the shapes
    a_ = im.getchannel('A')
    edge = ImageChops.subtract(a_.filter(ImageFilter.MaxFilter(3)), a_).point(lambda v: 255 if v else 0)
    ink = Image.new('RGBA', im.size, INK)
    ink.putalpha(edge)
    ink.alpha_composite(im)
    im = ink
    d = ImageDraw.Draw(im)
    # gear and the Hexcore drawn after mapping so they stay round and upright
    if hlen >= 4:
        gx, gy = T(max(L0 + 1, 7), W // 2 - 3 if side == 'L' else 3)
        gear(d, gx + o, gy + o, S, f)
    if side == 'R':
        hx, hy = T(max(L0 + 2, 7), W // 2 + 2)
        hx, hy = hx + o, hy + o
        for dy, (x0, x1) in zip(range(-2, 3), ((-1, 1), (-2, 2), (-2, 2), (-2, 2), (-1, 1))):
            line(d, [(hx + x0, hy + dy), (hx + x1, hy + dy)], S['trim'])
        for dy in (-1, 0, 1):
            line(d, [(hx - 1, hy + dy), (hx + 1, hy + dy)], VOID)
        core = HEX if look < 7 else (236, 246, 255, 255) if look == 7 else WHITE
        steady = look == 8
        pulse = core if steady else [core, core, mix(core, HEX_D, .4), mix(core, HEX_D, .6), mix(core, HEX_D, .4), core, WHITE, core][f]
        px(d, hx, hy, pulse)
        px(d, hx, hy - 1 if f % 2 else hy + 1, a(core, 190))
    if rank == 0 and side == 'L':
        x, y = T(13, 9)
        line(d, [(x + o - 1, y + o), (x + o + 1, y + o + 1)], TAPE)          # Spark: a taped crack
    return rim(im), o


def gear(d, x, y, S, f):
    d.ellipse((x - 2, y - 2, x + 2, y + 2), fill=S['trim'], outline=INK)
    px(d, x, y, INK)
    tooth = [(0, -3), (2, -2), (3, 0), (2, 2), (0, 3), (-2, 2), (-3, 0), (-2, -2)]
    for k in range(2):
        tx, ty = tooth[(k * 4 + f) % 8]
        px(d, x + tx, y + ty, S['trim_d'])


def rim(im):
    """The second pixel of dark outline that keeps the fists readable at game scale."""
    m = im.getchannel('A').filter(ImageFilter.MaxFilter(3))
    edge = Image.new('RGBA', im.size, INK)
    edge.putalpha(ImageChops.subtract(m, im.getchannel('A')).point(lambda v: 255 if v else 0))
    edge.alpha_composite(im)
    return edge


def place(im, look, f, side, spec):
    orient, x, y, hlen = spec
    layer, o = gauntlet(look, f, side, orient, hlen, big=(side == 'L'))
    im.alpha_composite(layer, (x - o, y - o))


GUARD_LEAD = ('up', 30, 26, 4)
GUARD_REAR = ('right', -4, 35, 5)
EXT = ('right', 24, 27, 10)          # the lead arm fully extended
HALF = ('right', 18, 27, 7)
# Each animated overlay: per frame the lead gauntlet and a beat label for the extras.
POSES = {
    'outfit': [(GUARD_LEAD, 'idle')] * 8,
    'jab': [(('up', 29, 27, 4), 'chamber'), (HALF, 'snap'), (EXT, 'hit'), (EXT, 'hold'), (EXT, 'hold'),
            (('right', 21, 27, 10), 'recoil'), (HALF, 'back'), (GUARD_LEAD, 'idle')],
    'block': [(('up', 29, 22, 10), 'raise'), (('up', 29, 20, 10), 'brace'), (('up', 29, 20, 10), 'brace'),
              (('up', 29, 20, 10), 'brace'), (('up', 29, 20, 10), 'brace'), (('up', 29, 21, 10), 'brace'),
              (('up', 29, 24, 8), 'lower'), (GUARD_LEAD, 'idle')],
    'punch': [(('up', 29, 27, 4), 'windup'), (('up', 28, 28, 4), 'windup'), (HALF, 'thrust'), (EXT, 'impact'),
              (EXT, 'through'), (EXT, 'through'), (HALF, 'back'), (GUARD_LEAD, 'idle')],
    'swipe': [(('up', 30, 16, 10), 'raise'), (('up', 31, 13, 10), 'raise'), (('down', 31, 21, 10), 'rip'),
              (('down', 31, 24, 10), 'follow'), (('down', 31, 24, 10), 'follow'), (('down', 31, 24, 10), 'follow'),
              (('up', 30, 24, 8), 'back'), (GUARD_LEAD, 'idle')],
    'shatter': [(GUARD_LEAD, 'clench'), (GUARD_LEAD, 'crush'), (GUARD_LEAD, 'burst'), (GUARD_LEAD, 'burst'),
                (GUARD_LEAD, 'burst'), (GUARD_LEAD, 'settle'), (GUARD_LEAD, 'settle'), (GUARD_LEAD, 'idle')],
}
SECONDS = {'outfit': .12, 'jab': 24 / 60 / 8, 'block': 18 / 60 / 8, 'punch': .06, 'swipe': 24 / 60 / 8, 'shatter': 30 / 60 / 8}


def rim_color(look):
    return (236, 222, 176, 255) if look == 0 else GOLD_WHITE if look == 8 else (228, 236, 255, 255) if look == 7 else RIFT_PALE


def small_tear(d, look, f, x, y, h, w):
    """The tear a rift punch goes through: a jagged lens, void inside, the rank's rim light."""
    rc = rim_color(look)
    j = [0, 1, 0, -1, 1, 0, -1, 0]
    pts = [(x, y - h), (x + w * .6 + j[f] * .5, y - h * .45), (x + w, y), (x + w * .6, y + h * .5), (x, y + h),
           (x - w * .6, y + h * .45), (x - w, y + j[(f + 3) % 8] * .5), (x - w * .55, y - h * .5)]
    poly(d, pts, fill=VOID, outline=rc)
    line(d, [(x, y - h + 2), (x, y + h - 2)], RIFT_DEEP if look != 8 else (90, 70, 30, 255))
    for k in range(3):
        q = pts[(f + k * 3) % len(pts)]
        px(d, q[0], q[1], WHITE)


@lru_cache(maxsize=None)
def body_mask():
    """The body sheet's upper-body silhouette (the same in every body frame)."""
    return assemble_layers(0, 0, 'outfit', 0, fx=False).getchannel('A').point(lambda v: 255 if v else 0)


def seal(im, style='near', S=None):
    """Make sure the overlay covers the body sheet below it. Pixels the pose left uncovered are filled:
    'near' grows the nearest overlay colour into them; 'smear' paints a motion smear of the gauntlet."""
    need = ImageChops.subtract(body_mask(), im.getchannel('A').point(lambda v: 255 if v == 255 else 0))
    if not need.getbbox():
        return im
    px_ = im.load()
    nd = need.load()
    todo = [(x, y) for y in range(64) for x in range(48) if nd[x, y]]
    if style == 'smear':
        S = S or STEEL
        for x, y in todo:
            px_[x, y] = (S['dark'] if y % 3 else mix(S['trim'], S['dark'], .45)) if not (x + y) % 7 == 0 else S['metal']
        return im
    for _ in range(64):
        rest = []
        for x, y in todo:
            got = None
            for dx, dy in ((-1, 0), (1, 0), (0, -1), (0, 1), (-1, -1), (1, -1), (-1, 1), (1, 1)):
                X, Y = x + dx, y + dy
                if 0 <= X < 48 and 0 <= Y < 64 and px_[X, Y][3] == 255 and not nd[X, Y]:
                    got = px_[X, Y]
                    break
            if got:
                px_[x, y] = got
            else:
                rest.append((x, y))
        for x, y in todo:
            if (x, y) not in rest:
                nd[x, y] = 0
        if not rest:
            break
        todo = rest
    return im


def assemble_layers(look, f, pose, frame, fx=True):
    """Pose `pose` at beat `frame`, with the rank layers. No legs: the body sheet owns them."""
    rank = min(look, 7)
    top = look == 8
    p = PAL[look]
    S = steel(look)
    lead, beat = POSES[pose][frame]
    flare = 2 if beat in ('burst', 'crush') else 0                           # Shatter blows the cloak out
    back_l = Image.new('RGBA', (48, 64))
    d = ImageDraw.Draw(back_l)
    cloak_back(d, look, min(7, f + flare) if flare else f, top)
    if rank >= 4:      # Wanderer: the cloak's underside is full of stars
        for k in range(4):
            sx, sy = 8 + (k * 3 + f) % 8, 32 + (k * 2 + f) % 6
            if back_l.getpixel((sx, sy))[3] and back_l.getpixel((sx, sy))[:3] == CLOAK['deep'][:3]:
                px(d, sx, sy, (230, 236, 255, 255) if (k + f) % 3 else RIFT_PALE)
    if rank >= 6:      # Riftwalker: the cloak's edge dissolves into rift fragments
        for k in range(4):
            fx_, fy = 5 - (f + k) % 3 - k, 34 + k * 2
            if fx_ >= 0:
                px(d, fx_, fy, a((190, 170, 255) if look != 8 else (255, 240, 200), 230 - k * 40))
    rear = Image.new('RGBA', (48, 64))
    place(rear, look, f, 'R', GUARD_REAR)
    mid = Image.new('RGBA', (48, 64))
    d = ImageDraw.Draw(mid)
    torso(d, p, look, f)
    if rank == 0:      # Spark: a patch on the tunic
        d.rectangle((25, 35, 27, 37), fill=(95, 107, 72, 255), outline=INK)
    if rank >= 1:      # Tear: cyan seams down the tunic
        for k, q in enumerate([(22, 34), (22, 37), (27, 33), (26, 37)]):
            px(d, *q, WHITE if k == f % 4 else (HEX if look != 8 else GOLD))
    if rank >= 2:      # Rift: the stress gauge on the belt
        d.ellipse((20, 39, 23, 42), fill=(30, 26, 36, 255), outline=BRASS)
        needle = [(21, 40), (22, 40), (22, 41), (22, 41), (23, 41), (22, 41), (22, 40), (21, 40)][f]
        line(d, [(21, 41), needle], (255, 120, 90, 255))
    if rank >= 3:      # Gate: gate rings on the tunic skirt
        for sx in (22, 27):
            d.ellipse((sx - 1, 42, sx + 1, 44), outline=GOLD if look != 8 else GOLD_WHITE)
    if rank >= 5:      # Cartographer: an orb bandolier across the chest
        line(d, [(20, 31), (28, 40)], (64, 44, 34, 255), 2)
        for k in range(5):
            px(d, 21 + k * 2, 32 + k * 2, WHITE if (k + f) % 5 == 0 else FAMILY[(k * 2 + 1) % 10])
    cloak_mantle(d, look, f, top)
    if top:            # #1 regalia: a tiny world crest on the clasp
        d.rectangle((24, 28, 26, 30), fill=GOLD, outline=INK)
        px(d, 25, 29, WHITE)
    hair(d, look, f + (2 if flare else 0))
    face(d, look, f)
    glasses(d, look, f)
    if beat == 'brace' and frame == 2:
        px(d, 30, 20, WHITE)                                                  # the glasses glint behind the block
    if rank >= 7:      # crown of floating shards
        for k in range(5):
            x = 19 + k * 2.5
            y = 9 - (2 if k == 2 else 1 if k in (1, 3) else 0) + (1 if (k + f) % 4 < 2 else 0)
            shard(d, x, y, 1, GOLD_WHITE if top else (220, 232, 255, 255), outline=INK if not top else (120, 90, 40, 255))
    if top:            # Between Worlds: the right half of her (not the gauntlets) lives in the other world's palette
        right = negative(mid.crop((26, 0, 48, 64)))
        mid.paste((0, 0, 0, 0), (26, 0, 48, 64))
        mid.alpha_composite(right, (26, 0))
        md = ImageDraw.Draw(mid)
        al = mid.getchannel('A')
        for y in range(64):
            if al.getpixel((26, y)):
                bright = (y + f * 4) % 16
                px(md, 26, y, GOLD_WHITE if bright < 3 else a(GOLD, 220) if bright < 8 else a(WHITE, 160))
    front = Image.new('RGBA', (48, 64))
    d = ImageDraw.Draw(front)
    beat_tear = beat in ('impact', 'through') or (pose == 'punch' and beat == 'thrust')
    if pose == 'punch' and beat_tear:
        small_tear(d, look, f, 43, 33, [0, 0, 7, 12, 13, 13, 0, 0][frame], [0, 0, 2, 4, 4, 4, 0, 0][frame])
    place(front, look, f, 'L', lead)
    d = ImageDraw.Draw(front)
    if rank >= 4 and lead == GUARD_LEAD:      # Wanderer: copper coil piping from the lead housing to the shoulder
        line(d, [(41, 40), (36, 43), (31, 42)], S['trim_l'])
    if rank >= 6:      # Riftwalker: rift-light cracks through the rear plates
        for k, q in enumerate([(9, 38), (10, 40), (11, 41)]):
            px(ImageDraw.Draw(rear), *q, WHITE if k == f % 3 else (190, 170, 255, 255))
    if top:            # #1 quiet menace: engraved knuckles and a heat haze round both fists
        for k in range(6):
            ang = k * TAU / 6 + f * .4
            for cx, cy, r in ((37, 32, 9), (10, 41, 9)):
                hx, hy = pt(cx, cy, r + (k + f) % 2, ang)
                if 0 <= round(hx) < 48 and 0 <= round(hy) < 64 and front.getpixel((round(hx), round(hy)))[3] == 0:
                    px(d, hx, hy, a((255, 236, 190), 70 + 40 * ((k + f) % 3)))
    im = Image.new('RGBA', (48, 64))
    for layer in (back_l, rear, mid, front):
        im.alpha_composite(layer)
    d = ImageDraw.Draw(im)
    # beats: smears, flashes and shards
    if beat in ('snap', 'thrust'):
        for k in range(3):
            line(d, [(20 - k * 2, 29 + k * 5), (30 - k, 29 + k * 5)], a(WHITE, 200 - k * 50))
    if beat in ('hit', 'impact'):
        for k in range(4):
            ang = (k - 1.5) * .45
            line(d, [pt(46, 33, 1, ang), pt(46, 33, 3, ang)], WHITE)
        px(d, 47, 33, HEX)
    if beat == 'rip':
        for k in range(5):
            q0 = (44 - k, 14 + k * 7)
            line(d, [q0, (q0[0] - 2, q0[1] + 5)], (190, 170, 255, 255) if k % 2 else WHITE)
    if pose == 'block' and beat == 'brace' and frame in (1, 2):
        ring(d, 43, 25, 4 + frame, a(HEX, 200), 1, 10, -1.3, 1.3)
    if pose == 'shatter' and beat in ('crush', 'burst'):
        n = 3 + frame * 2
        for k in range(n):
            ang = -math.pi / 2 + (k / max(1, n - 1) - .5) * 2.6
            r = 6 + frame * 3 + (k % 3)
            sx, sy = pt(37, 30, r, ang)
            if 0 <= round(sx) < 48 and 0 <= round(sy) < 64 and im.getpixel((round(sx), round(sy)))[3] == 0:
                shard(d, sx, sy, 1, FAMILY[k % 10] if look else RIFT_PALE)
        if beat == 'crush':
            line(d, [(31, 27), (43, 27)], WHITE)
    if look < 7 and rank >= 3:
        shards_orbit(d, 2 if rank < 6 else 4, f, True, look=look)
    if look == 7:
        shards_orbit(d, 6, f, True, look=7)
    if top:
        shards_orbit(d, 10, f, True, families=True)
    if fx and rank == 0 and pose == 'outfit':     # Spark: the lead gauntlet sputters
        for k in range(2):
            if (f + k) % 3 != 2:
                sx, sy = pt(44, 28, 2 + (f + k * 2) % 3, hsh(7, f, k) * TAU)
                px(d, sx, sy, (255, 236, 160, 255) if k else HEX)
    return im


def costume(look, f, pose='outfit'):
    """The overlay for one pose at one frame, sealed over the body sheet."""
    im = assemble_layers(look, f, pose, f)
    moved = POSES[pose][f][0] != GUARD_LEAD
    return seal(im, 'smear' if moved and pose in ('jab', 'punch', 'swipe') else 'near', steel(look))


def outfit(look, f):
    return costume(look, f, 'outfit')


def punch(look, f):
    return costume(look, f, 'punch')


def body(kind='idle', f=0):
    """The registered body sheet: rank-0 look. The upper body keeps one silhouette in every frame (the pose
    overlays always cover it); the legs and the effects around her carry the motion."""
    im = Image.new('RGBA', (48, 64))
    d = ImageDraw.Draw(im)
    if kind == 'dead':
        poly(d, [(12, 55), (16, 52), (32, 52), (35, 55), (32, 59), (14, 59)], fill=INK)
        d.rectangle((16, 53, 31, 57), fill=PAL[0]['coat'])
        d.rectangle((17, 56, 30, 57), fill=PAL[0]['shade'])
        poly(d, [(32, 51), (37, 50), (39, 53), (36, 56), (32, 56)], fill=SKIN, outline=INK)
        poly(d, [(33, 49), (38, 48), (40, 51), (34, 51)], fill=HAIR, outline=INK)
        line(d, [(34, 52), (37, 52)], INK)                                    # the glasses, askew
        poly(d, [(8, 52), (16, 50), (17, 54), (9, 56)], fill=CLOAK['deep'], outline=INK)
        rear, o = gauntlet(0, 0, 'R', 'right', 4)
        lead, o2 = gauntlet(0, 0, 'L', 'right', 4, big=True)
        im.alpha_composite(rear, (-8 - o, 46 - o))
        im.alpha_composite(lead, (22 - o2, 47 - o2))
        d = ImageDraw.Draw(im)
        dim = [255, 200, 150, 110, 80, 55, 35, 20][f]
        px(d, 4, 50, a(HEX, dim))
        if f < 6:
            px(d, 5 + (f % 3), 45 - f // 2, a((255, 236, 160), 200 - f * 30))
        return im
    legs(d, kind, f)
    im.alpha_composite(assemble_layers(0, 0, 'outfit', 0, fx=False))
    d = ImageDraw.Draw(im)
    if kind == 'attack':       # the jab: a knuckle flash and a cyan shock off the lead fist
        if 1 <= f <= 4:
            r = [0, 2, 4, 3, 1][f]
            for k in range(3):
                line(d, [(45, 28 + k * 4), (45 + r, 28 + k * 4)], a(HEX, 230))
        else:
            px(d, 46, 31 + f % 3, a((200, 220, 230), 160))
    elif kind == 'skill1':     # orbs ignite on the lead gauntlet
        for k in range(1 + f // 2):
            px(d, 45, 26 + k * 3, FAMILY[(k * 2 + f) % 10])
    elif kind == 'skill2':     # the swipe: a tearing arc in front of her
        for k in range(f + 1):
            px(d, *pt(32, 34, 15, -1.3 + k * .3), RIFT_PALE if k < f else WHITE)
    elif kind == 'ult':        # the Hexcore surges
        for k in range(6):
            q = pt(6, 44, 3 + f, k * TAU / 6 + f * .2)
            if 0 <= q[0] < 48:
                px(d, *q, a(HEX, 255 - f * 20))
    elif kind == 'hit':
        if f < 4:
            m = im.getchannel('A').filter(ImageFilter.MaxFilter(3))
            edge = ImageChops.subtract(m, im.getchannel('A'))
            flash = Image.new('RGBA', (48, 64), (255, 255, 255, 255))
            flash.putalpha(edge.point(lambda v: v * (4 - f) // 4))
            im.alpha_composite(flash)
            d = ImageDraw.Draw(im)
        px(d, 45, 24 - f // 2, a((255, 200, 160), 255 - f * 28))
    elif kind == 'idle':
        px(d, 14 + f, 61, a(HEX, 60))
    elif kind == 'run':
        px(d, 13 - f % 4, 60 - f // 4, a((150, 130, 110), 120))
    return im


def torso_only(look=0):
    """For the verifier: the bare torso of a look, without head, arms or legs."""
    im = Image.new('RGBA', (48, 64))
    torso(ImageDraw.Draw(im), PAL[look], look, 0)
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
            # a fracture in the air behind her back shoulder: #1 the largest, #2 silver, #3 bronze. Small,
            # 1-px and jagged like real cracked glass, with a glint running along it.
            size = {1: 26, 2: 20, 3: 15}[podium]
            sky = Image.new('RGBA', AURA)
            sd = ImageDraw.Draw(sky)
            hub = (bx + 5, by + 25)
            paths = cracks(41 + podium, *hub, {1: 5, 2: 4, 3: 3}[podium], size, TAU * .7, 2.2 + ph * .01)
            for pth in paths:
                line(sd, pth, a(col, 230))
            if podium == 1:
                # a Hextech lattice glows through the fracture
                grid = Image.new('RGBA', AURA)
                gd = ImageDraw.Draw(grid)
                hexes(gd, hub[0], hub[1], 30, 30, 5, a((127, 240, 255), 255))
                near = sky.getchannel('A').filter(ImageFilter.MaxFilter(5)).point(lambda v: 120 if v else 0)
                grid.putalpha(ImageChops.multiply(grid.getchannel('A'), near))
                im.alpha_composite(grid)
                d = ImageDraw.Draw(im)
            im.alpha_composite(glow(sky, .8, 1.2))
            d = ImageDraw.Draw(im)
            pth = paths[f % len(paths)]
            q = pth[min(len(pth) - 1, 1 + f % 3)]
            px(d, q[0], q[1], WHITE)
            # worlds seen through the fracture: 2 for #2, 1 for #3
            if podium in (2, 3):
                fams = [1, 7] if podium == 2 else [4]
                for k, fam in enumerate(fams):
                    wx, wy = hub[0] + (-6 + k * 12 if podium == 2 else 0), hub[1] - 6
                    vig = world(fam, 7, 6, f)
                    pts = [pt(wx, wy, 4, j * TAU / 8, .8) for j in range(8)]
                    paste_masked(im, vig, wx - 3, wy - 3, pts)
                    d = ImageDraw.Draw(im)
                    line(d, pts + [pts[0]], col)
        if podium == 1:
            # reality bends: a dark afterimage trails 3 px behind her, and the floor cracks under her feet
            ghost = costume(8, f).getchannel('A').point(lambda v: 110 if v else 0)
            shade = Image.new('RGBA', (48, 64), (40, 20, 74, 255))
            shade.putalpha(ghost)
            im.alpha_composite(shade, (bx - 3, by))
            d = ImageDraw.Draw(im)
            foot = (cx + 1, by + 59)
            for k, pth in enumerate(cracks(77, *foot, 6, 16, TAU, 0, False)):
                flat = [(foot[0] + (x - foot[0]) * 1.3, foot[1] + (y - foot[1]) * .28) for x, y in pth]
                lit = (k + f) % 6 == 0
                line(d, flat, a(RIFT_PALE if lit else (120, 96, 160), 230 if lit else 150))
            px(d, foot[0] - 12 + f * 3, foot[1], a(GOLD_WHITE, 200))
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

    def composed(r, f, podium=4, pose='outfit', emblem=False):
        look = 8 if r == 7 and podium == 1 else r
        btag = f'back_r7_p{podium}' if r == 7 else f'back_r{r}'
        im = original(btag, f, AW, AH)
        base = body('idle', f)
        base.alpha_composite(original(f'{pose}_r{look}', f, 48, 64))
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
    prefixes = ('outfit_', 'punch_', 'jab_', 'block_', 'swipe_', 'shatter_', 'rank', 'top', 'back_', 'orbs', 'hex', 'core', 'stress', 'rift_', 'glyph', 'link_',
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
        scene = composed(r, 3 if i == 1 else 2, p, pose='punch' if i == 1 else 'outfit', emblem=True).crop((BODY[0] - 24, BODY[1] - 28, BODY[0] + 72, BODY[1] + 76))
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
    # docs: the animated poses, every frame, at Gate (rows) and the rift punch for every look
    names = [('outfit', 'GUARD'), ('jab', 'JAB'), ('block', 'BLOCK'), ('punch', 'RIFT PUNCH'), ('swipe', 'SWIPE'), ('shatter', 'SHATTER')]
    im = Image.new('RGBA', (140 + 8 * 104, 70 + len(names) * 136 + 150), bg)
    d = ImageDraw.Draw(im)
    title(d, 24, 16, 'POSES / EIGHT FRAMES EACH (2x, Gate look); BELOW: THE RIFT PUNCH FOR EVERY LOOK', 18)
    for row, (pose, label) in enumerate(names):
        y = 60 + row * 136
        title(d, 16, y + 56, label, 13)
        for f in range(8):
            base = body('idle', 0)
            base.alpha_composite(original(f'{pose}_r3', f, 48, 64))
            im.alpha_composite(big(base), (140 + f * 104, y))
    y = 60 + len(names) * 136
    for look in range(9):
        base = body('idle', 0)
        base.alpha_composite(original(f'punch_r{look}', 4, 48, 64))
        im.alpha_composite(big(base), (16 + look * 104, y))
        title(d, 20 + look * 104, y + 130, (RANKS + ['#1'])[look][:12], 11)
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
