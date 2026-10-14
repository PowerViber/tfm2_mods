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
BOOT = (222, 228, 238, 255)        # white sci-fi boots: a dark sole and one cyan running light
BOOT_L = (246, 248, 252, 255)
BOOT_D = (196, 204, 218, 255)
SOLE = (48, 54, 68, 255)
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
# Vesper in a compact 3/4 boxing guard, facing +x and turned toward the camera. Her LEFT gauntlet (pure orbs)
# leads on the far side, raised in front of her chest with the knuckles toward the opponent; her RIGHT
# gauntlet (Hexcore) is the near one, pulled back low beside her ribs. Wide staggered stance, bent front
# knee, back heel up; the torso turned about 40 degrees and leaning in; a tomboy wolf cut, black glasses with
# one cracked lens, a fitted undershirt under a sleeveless tunic and a short bronze half-cloak.
#
# Built in layers (legs, hips, torso, arms, fists, head, hair, cloak) placed by the pose rig, so every pose
# keeps her proportions. The costume is the same at every rank: mastery shows as the damage she does to
# reality (mastery_fx, back_scar) and, at #1, gold-chrome gauntlets.

HEX = (110, 240, 255, 255)          # Hextech cyan: only a few pixels per gauntlet
HEX_D = (40, 120, 150, 255)


def _steel(metal, dark, light, trim, trim_l, trim_d):
    return dict(metal=metal + (255,), dark=dark + (255,), light=light + (255,), trim=trim + (255,),
                trim_l=trim_l + (255,), trim_d=trim_d + (255,))


# mirror chrome: dimensions have stripped the old copper off her gear
STEEL = _steel((150, 160, 176), (72, 80, 96), (236, 242, 250), (196, 206, 220), (255, 255, 255), (96, 104, 122))
FRINGE_C = (110, 240, 255, 255)       # light splitting across dimensions on the chrome edges
FRINGE_M = (236, 96, 210, 255)
SHIRT = (44, 50, 68, 255)           # a dark sci-fi bodysuit under the white chest plate, cyan piping
SHIRT_L = (78, 88, 112, 255)
SHIRT_D = (32, 36, 50, 255)
SEAL = (52, 58, 74, 255)             # the dark neck seal and panel seams
HAIR = (110, 72, 52, 255)            # warm chestnut, so it never merges with the void
HAIR_L = (160, 112, 80, 255)
HAIR_D = (70, 46, 40, 255)
LENS = (132, 152, 182, 255)          # pale glass, so the black frame reads around it
LENS_FAR = (104, 176, 196, 255)     # faint cyan far lens
EYE_GLOW = (176, 120, 255, 255)      # the dimension in her, behind the crack
BRUISE = (128, 96, 128, 255)
EYE = (24, 18, 22, 255)
TUNIC = dict(coat=(240, 242, 248, 255), shade=(192, 198, 212, 255), light=(252, 252, 255, 255), belt=(40, 44, 58, 255))
CLOAK = dict(deep=(52, 58, 70, 255), warm=(86, 94, 110, 255), hi=(132, 142, 160, 255), trim=(150, 110, 230, 255), trim2=(110, 230, 240, 255))


def limb(d, p0, p1, w0, w1, fill, edge=INK):
    """A tapered limb segment as a quad (p0 width w0 -> p1 width w1), outlined."""
    dx, dy = p1[0] - p0[0], p1[1] - p0[1]
    n = math.hypot(dx, dy) or 1
    nx, ny = -dy / n, dx / n
    pts = [(p0[0] + nx * w0 / 2, p0[1] + ny * w0 / 2), (p1[0] + nx * w1 / 2, p1[1] + ny * w1 / 2),
           (p1[0] - nx * w1 / 2, p1[1] - ny * w1 / 2), (p0[0] - nx * w0 / 2, p0[1] - ny * w0 / 2)]
    poly(d, pts, fill=fill, outline=edge)
    return pts


STANCE = dict(hip_b=(21, 44), knee_b=(16, 50), ankle_b=(13, 55), hip_f=(26, 44), knee_f=(32, 49), ankle_f=(31, 55))


def legs(d, kind='idle', f=0):
    """A wide, staggered fighting stance: the front knee bent over the toe, the back leg long behind her
    with the heel up. Both feet stay on y 58."""
    s = [0, 1, 2, 1, 0, -1, -2, -1][f] if kind == 'run' else ([0, 1, 1, 1, 0, 0, 0, 0][f] if kind == 'attack' else 0)
    S = STANCE
    kb, ab = (S['knee_b'][0] - s, S['knee_b'][1]), (S['ankle_b'][0] - s * 2, S['ankle_b'][1] - max(0, s))
    kf, af = (S['knee_f'][0] + s, S['knee_f'][1]), (S['ankle_f'][0] + s * 2, S['ankle_f'][1] - max(0, -s))
    # back leg (further away, darker)
    limb(d, S['hip_b'], kb, 5, 4, TROUSER)
    limb(d, kb, ab, 4, 3, TROUSER)
    x, y = ab
    poly(d, [(x - 3, y), (x + 1, y), (x + 5, y + 2), (x + 5, y + 3), (x, y + 3), (x - 3, y + 1)], fill=BOOT_D, outline=INK)   # heel up
    line(d, [(x + 1, y + 3), (x + 4, y + 3)], SOLE)
    px(d, x - 1, y + 1, HEX_D)
    # front leg
    limb(d, S['hip_f'], kf, 6, 4, TROUSER_L)
    limb(d, kf, af, 4, 4, TROUSER_L)
    px(d, kf[0], kf[1] - 1, (128, 124, 142, 255))                           # the bent knee catches the light
    x, y = af
    poly(d, [(x - 2, y), (x + 2, y), (x + 6, y + 2), (x + 6, y + 3), (x - 2, y + 3)], fill=BOOT, outline=INK)
    line(d, [(x - 1, y + 1), (x + 2, y + 1)], BOOT_L)
    line(d, [(x - 1, y + 3), (x + 5, y + 3)], SOLE)                         # the dark sole
    px(d, x + 3, y + 1, HEX)                                                  # the running light


def hips(d, p):
    """The pelvis, tilted and turned: the belt slants down toward the front."""
    poly(d, [(19, 40), (29, 41), (31, 45), (26, 47), (19, 46)], fill=TROUSER, outline=INK)
    poly(d, [(19, 40), (29, 41), (29, 43), (19, 42)], fill=p['coat'], outline=INK)   # tunic hem over the hips
    line(d, [(20, 41), (28, 42)], p['belt'])
    px(d, 26, 42, HEX)                                                        # a cyan buckle light


def torso(d, p, look=0, f=0):
    """Turned about 40 degrees to her right and leaning in: the near shoulder (screen-left) broad and low,
    the far shoulder narrow and high, the shoulder line ahead of the hips. Sleeveless tunic, no armour."""
    # undershirt at the shoulders
    poly(d, [(17, 31), (22, 28), (29, 27), (34, 28), (34, 31), (18, 34)], fill=SHIRT, outline=INK)
    # tunic: chest leaning forward over the waist
    poly(d, [(19, 31), (24, 29), (29, 29), (32, 30), (31, 34), (29, 38), (29, 41), (20, 40), (21, 37), (19, 34)],
         fill=p['coat'], outline=INK)
    poly(d, [(27, 30), (31, 31), (30, 34), (28, 38), (28, 40), (26, 40), (26, 35)], fill=p['shade'])    # the far side turns away
    line(d, [(20, 32), (21, 35), (22, 38)], p['light'])
    poly(d, [(25, 29), (28, 29), (27, 32)], fill=SEAL)                        # the dark neck seal
    line(d, [(22, 34), (24, 36)], p['shade'])                                 # a panel seam across the turn
    line(d, [(24, 36), (24, 39)], p['shade'])
    px(d, 23, 33, HEX)                                                        # the chest plate's cyan light


def forearm(d, elbow, wrist, side, look, f, w0=8, w1=7):
    """The gauntlet housing between elbow and wrist at any angle: plates banded across it, a piston along
    one side, a gear on the elbow joint; the orb sockets run along the left gauntlet."""
    S = steel(look)
    pts = limb(d, elbow, wrist, w0, w1, S['dark'])
    ex, ey = elbow
    wx, wy = wrist
    dx, dy = wx - ex, wy - ey
    n = math.hypot(dx, dy) or 1
    ux, uy, nx, ny = dx / n, dy / n, -dy / n, dx / n
    for t in (.4, .75):                      # plate bands across the housing
        cx, cy = ex + dx * t, ey + dy * t
        w = (w0 + (w1 - w0) * t) / 2 - .5
        line(d, [(cx + nx * w, cy + ny * w), (cx - nx * w, cy - ny * w)], INK)
    light_side = 1 if ny < 0 or (ny == 0 and nx < 0) else -1                # the side facing the top-left light
    line(d, [(ex + nx * light_side * (w0 / 2 - 1), ey + ny * light_side * (w0 / 2 - 1)),
             (wx + nx * light_side * (w1 / 2 - 1), wy + ny * light_side * (w1 / 2 - 1))], S['light'])
    line(d, [(ex - nx * light_side * (w0 / 2 - 1.5), ey - ny * light_side * (w0 / 2 - 1.5)),
             (wx - nx * light_side * (w1 / 2 - 1.5), wy - ny * light_side * (w1 / 2 - 1.5))], S['metal'])   # piston
    if side == 'L' and n >= 4:
        for k in range(min(5, int(n) - 1)):
            t = .25 + k * .12
            px(d, ex + dx * t, ey + dy * t, FAMILY[(k * 2 + f // 4) % 10] if look else (RIFT_PALE if (k + f) % 4 else (90, 80, 120, 255)))
    gear(d, round(ex), round(ey), S, f)
    return pts


def sleeve(d, shoulder, elbow, near):
    """The upper arm in the fitted undershirt: the near arm a touch thicker."""
    limb(d, shoulder, elbow, 6 if near else 5, 5 if near else 4, SHIRT if near else SHIRT_D)
    if near:                                                                  # the bodysuit's cyan piping
        line(d, [(shoulder[0] + (elbow[0] - shoulder[0]) * .2, shoulder[1] + (elbow[1] - shoulder[1]) * .2),
                 (shoulder[0] + (elbow[0] - shoulder[0]) * .6, shoulder[1] + (elbow[1] - shoulder[1]) * .6)], HEX_D)


VISOR = (58, 40, 104, 255)           # dark violet glass
VISOR_L = (96, 74, 160, 255)


def head(d, look, f):
    """A reckless interdimensional boxer: a true 3/4 head with the lower face bare (a flat, tired mouth, a
    bruised cheek), a small chrome brow plate, and a slim visor over the eyes. The far side of the visor is
    cracked and its signal glitches cyan and magenta; behind the glass her near eye glows violet."""
    d.rectangle((26, 27, 28, 30), fill=SKIN_D)                               # neck
    poly(d, [(24, 18), (31, 18), (33, 20), (34, 23), (33, 24), (33, 26), (31, 28), (27, 29), (24, 27), (23, 22)],
         fill=SKIN, outline=INK)
    line(d, [(24, 24), (24, 26)], SKIN_D)                                     # the near cheek turns into shadow
    px(d, 34, 23, SKIN_D)                                                     # nose shadow
    line(d, [(29, 26), (31, 26)], INK)                                        # a flat, tired mouth
    px(d, 26, 25, BRUISE)                                                     # a bruise on the cheek
    px(d, 25, 25, mix(BRUISE, SKIN_D, .5))
    # the brow plate: light combat headgear, not a helmet
    poly(d, [(24, 17), (31, 16), (33, 18), (32, 19), (24, 19)], fill=STEEL['metal'], outline=INK)
    line(d, [(25, 17), (30, 16)], STEEL['light'])
    px(d, 27, 18, STEEL['dark'])                                              # a rivet
    px(d, 32, 18, FRINGE_C)
    # the visor: one slim band wrapping round the eyes
    poly(d, [(22, 20), (34, 20), (35, 21), (34, 23), (23, 23), (22, 22)], fill=VISOR, outline=INK)
    line(d, [(23, 21), (33, 21)], VISOR_L)
    scan = 23 + (f * 2) % 11
    px(d, scan, 22, a(FRINGE_C, 200))                                         # a signal line sweeping the glass
    glow = [.55, .7, .85, 1, .85, .7, .55, .45][f]
    px(d, 26, 22, mix(VISOR, EYE_GLOW, glow))                                 # the near eye glows behind it
    px(d, 31, 22, mix(VISOR, EYE_GLOW, glow * .5))
    # the far side is broken: a white crack, and the signal glitches
    px(d, 30, 21, WHITE)
    px(d, 31, 22, WHITE)
    px(d, 32, 21, mix(VISOR_L, WHITE, .5))
    if f in (2, 5, 6):
        line(d, [(33, 21), (35, 21)], FRINGE_C)
        px(d, 34, 22, FRINGE_M)
    line(d, [(20, 21), (22, 21)], INK)                                        # the strap into her hair


def hair(d, look, f):
    """Chestnut hair only at the back and sides now: a choppy mass behind the headgear, a side lock past the
    near cheek and nape flicks kicking outward; a few tufts stick up behind the brow plate."""
    sway = 1 if f in (2, 3, 4) else 0
    # the back of the head
    poly(d, [(19, 21), (20, 16), (23, 13), (27, 12), (30, 13), (31, 15), (25, 16), (23, 19), (22, 23)], fill=HAIR, outline=INK)
    poly(d, [(25, 15), (26, 11), (28, 14)], fill=HAIR, outline=INK)           # tufts behind the brow plate
    poly(d, [(28, 14), (30, 11), (31, 14)], fill=HAIR, outline=INK)
    # side lock past the near cheek
    poly(d, [(22, 21), (24, 21), (24, 24), (23, 26), (22, 24)], fill=HAIR, outline=INK)
    # nape flicks: layered ends kicking outward behind her
    poly(d, [(21, 18), (22, 24), (21, 27), (19 - sway, 28), (20, 25), (17 - sway, 26), (19, 22), (17 - sway, 21), (20, 19)],
         fill=HAIR, outline=INK)
    line(d, [(21, 24), (20, 26)], HAIR_D)                                     # the underside of the nape layer
    for q in [((21, 16), (25, 13)), ((27, 13), (29, 13)), ((20, 22), (19, 25))]:
        line(d, list(q), HAIR_L)                                              # lighter strands through it


def cloak(d, look, f):
    """A short gunmetal half-cloak: a wide mantle over the near shoulder, then three tattered tails trailing
    back, ending above the rear elbow. Its edge catches violet-cyan light; its tails sway outward only."""
    C = CLOAK
    sway = [0, 1, 1, 2, 2, 1, 1, 0][f]
    # the trailing part behind her: three tattered tails
    tails = [(18, 28), (14, 29), (9, 30), (4 - sway, 31), (8, 33), (3 - sway, 35), (7, 35), (5 - sway, 38), (10, 37),
             (14, 37), (17, 34), (19, 31)]
    poly(d, tails, fill=C['deep'], outline=INK)
    line(d, [(15, 30), (10, 32), (6 - sway, 34)], C['warm'])
    line(d, [(16, 33), (11, 35), (7 - sway, 37)], C['warm'])
    for k, q in enumerate([(4 - sway, 31), (3 - sway, 35), (5 - sway, 38)]):
        px(d, *q, C['trim'] if (k + f) % 2 else C['trim2'])                 # the iridescent edge on each tail tip


def mantle(d, look, f):
    """The cloak's mantle, on top of her near shoulder: wide where the cloak starts."""
    C = CLOAK
    poly(d, [(14, 30), (17, 27), (24, 27), (25, 29), (22, 31), (17, 33)], fill=C['warm'], outline=INK)
    line(d, [(17, 28), (23, 28)], C['hi'])
    line(d, [(16, 32), (21, 31)], C['trim'] if f % 2 else C['trim2'])


def fist(look, f, side, orient='right', big=False):
    """A mechanical fist seen from the side, knuckles forward (+x when orient='right'): a bronze cuff, the
    domed back of the hand, a copper thumb bump on top, curled steel fingers and four knuckle plates that
    bump out of the silhouette. Drawn once in the 'right' orientation (lit from the top-left) and turned
    for the others. Returns (layer, origin offset of local (0, 0))."""
    S = steel(look)
    rank = min(look, 7)
    W = 14 if big else 13
    L = 13
    o = 3
    im = Image.new('RGBA', (L + 2 * o, W + 2 * o))
    d = ImageDraw.Draw(im)
    X = lambda u: u + o
    Y = lambda v: v + o
    # cuff (narrower than the hand) and the hand's body with chamfered corners
    d.rectangle((X(0), Y(2), X(1), Y(W - 3)), fill=S['trim'])
    line(d, [(X(0), Y(2)), (X(1), Y(2))], S['trim_l'])
    poly(d, [(X(2), Y(1)), (X(4), Y(0)), (X(9), Y(0)), (X(10), Y(1)), (X(10), Y(W - 2)), (X(9), Y(W - 1)),
             (X(4), Y(W - 1)), (X(2), Y(W - 2))], fill=S['metal'])
    line(d, [(X(3), Y(1)), (X(8), Y(1))], S['light'])                        # the dome catches the light
    px(d, X(3), Y(2), S['light'])
    line(d, [(X(6), Y(1)), (X(6), Y(W - 2))], INK)                           # knuckle joint line across the hand
    # curled fingers: four bands stacked across, each with a dark crease
    bands = [(1, 3), (4, 6), (7, 9), (10, W - 2)]
    for k, (v0, v1) in enumerate(bands):
        d.rectangle((X(7), Y(v0), X(9), Y(v1)), fill=S['dark'] if k % 2 else S['metal'])
        line(d, [(X(7), Y(v0)), (X(9), Y(v0))], S['light'] if k == 0 else S['metal'])
        if k:
            line(d, [(X(7), Y(v0 - 1)), (X(10), Y(v0 - 1))], INK)
        # the knuckle plate bumps out past the fingers
        d.rectangle((X(10), Y(v0), X(11), Y(v1)), fill=S['trim'])
        line(d, [(X(11), Y(v0)), (X(11), Y(v1))], S['trim_l'])
        px(d, X(12), Y((v0 + v1) // 2), S['trim_d'])
        px(d, X(10), Y(v1), S['trim_d'])
    # the copper thumb bump on top, folded over the first finger
    poly(d, [(X(4), Y(0)), (X(5), Y(-1)), (X(8), Y(-1)), (X(9), Y(0)), (X(8), Y(2)), (X(5), Y(2))], fill=S['trim_d'])
    line(d, [(X(5), Y(-1)), (X(8), Y(-1))], S['trim'])
    # Hextech: a few cyan pixels between the fingers
    glow_px = HEX
    px(d, X(10), Y(6), glow_px)
    px(d, X(2), Y(2), FRINGE_C)                                               # light splitting on the chrome
    px(d, X(11), Y(W - 2), FRINGE_M)
    if rank >= 3:
        px(d, X(10), Y(9), glow_px)
    if side == 'R':      # the Hexcore set into the back of the hand
        hx, hy = X(4), Y(W // 2 + 1)
        for dy, (x0, x1) in zip(range(-2, 3), ((-1, 1), (-2, 2), (-2, 2), (-2, 2), (-1, 1))):
            line(d, [(hx + x0, hy + dy), (hx + x1, hy + dy)], S['trim'])
        for dy in (-1, 0, 1):
            line(d, [(hx - 1, hy + dy), (hx + 1, hy + dy)], VOID)
        core = WHITE if look == 8 else HEX
        pulse = core if look == 8 else [core, core, mix(core, HEX_D, .4), mix(core, HEX_D, .6), mix(core, HEX_D, .4), core, WHITE, core][f]
        px(d, hx, hy, pulse)
        px(d, hx, hy - 1 if f % 2 else hy + 1, a(core, 190))
    elif rank == 0:      # Spark: a taped crack across the left fist
        line(d, [(X(3), Y(W - 4)), (X(5), Y(W - 3))], TAPE)
    # outline and the thick rim
    a_ = im.getchannel('A')
    edge = ImageChops.subtract(a_.filter(ImageFilter.MaxFilter(3)), a_).point(lambda v: 255 if v else 0)
    ink = Image.new('RGBA', im.size, INK)
    ink.putalpha(edge)
    ink.alpha_composite(im)
    im = rim(ink)
    if orient == 'up':
        im = im.transpose(Image.Transpose.ROTATE_90)
    elif orient == 'down':
        im = im.transpose(Image.Transpose.ROTATE_270)
    return im, o


def fist_at(im, look, f, side, wrist, orient='right'):
    """Place a fist so its cuff sits on the wrist point."""
    big = side == 'R'
    layer, o = fist(look, f, side, orient, big)
    W = 14 if big else 13
    wx, wy = wrist
    if orient == 'right':          # local (0, W/2) is the wrist
        X, Y = wx - o, wy - W // 2 - o
    elif orient == 'up':           # rotated 90: local u runs upward
        X, Y = wx - W // 2 - o, wy - 12 - o
    else:
        X, Y = wx - (W - 1 - W // 2) - o, wy - o
    im.alpha_composite(layer, (round(X), round(Y)))


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


# ------------------------------------------------------- the dimensional scar --
# Reality is glass: every mark she leaves is a torn, irregular cut with a white-cyan fractured edge, a
# violet interior and a near-black void. Never a clean portal.
SCAR_EDGE = (206, 246, 255, 255)
SCAR_MID = (128, 74, 206, 255)
SCAR_VOID = (14, 8, 24, 255)


def scar(d, p0, p1, depth, seed, f=0, edge=SCAR_EDGE, mid=SCAR_MID, void=SCAR_VOID):
    """A jagged cut from p0 to p1. depth 0 is a hairline fracture; deeper cuts open a violet lens with a
    void core. Deterministic: the jag comes from the seed, a little flutter from the frame."""
    dx, dy = p1[0] - p0[0], p1[1] - p0[1]
    n = math.hypot(dx, dy) or 1
    nx, ny = -dy / n, dx / n
    steps = max(3, int(n / 2.5))
    spine = []
    for k in range(steps + 1):
        t = k / steps
        j = (hsh(seed, k) - .5) * 1.6 if 0 < k < steps else 0
        spine.append((p0[0] + dx * t + nx * j, p0[1] + dy * t + ny * j))
    if depth <= 0:
        line(d, spine, edge)
        return spine
    left, right = [], []
    for k, (x, y) in enumerate(spine):
        t = k / steps
        w = depth * math.sin(math.pi * t) * (.75 + .5 * hsh(seed, k, 1)) + (.3 if (k + f) % 3 == 0 and 0 < k < steps else 0)
        left.append((x + nx * w, y + ny * w))
        right.append((x - nx * w * (.8 + .3 * hsh(seed, k, 2)), y - ny * w))
    outline = left + right[::-1]
    poly(d, outline, fill=mid, outline=edge)
    if depth >= 1.6:
        inner = [(x + (lx - x) * .45, y + (ly - y) * .45) for (x, y), (lx, ly) in zip(spine, left)]
        inner += [(x + (rx - x) * .45, y + (ry - y) * .45) for (x, y), (rx, ry) in zip(spine[::-1], right[::-1])]
        poly(d, inner, fill=void)
    return spine


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


# ------------------------------------------------------------------ poses --
# Each pose frame places both arms (shoulder, elbow, wrist, fist orientation) and names its beat.
# The lead arm is her LEFT (pure orbs, far side); the rear is her RIGHT (Hexcore, near the camera).
def arm(shoulder, elbow, wrist, orient='right'):
    return dict(shoulder=shoulder, elbow=elbow, wrist=wrist, orient=orient)


G_LEAD = arm((32, 30), (28, 42), (30, 36))
G_REAR = arm((19, 32), (8, 43), (12, 45))
EXT_LEAD = arm((32, 30), (31, 33), (35, 33))
POSES = {
    'outfit': [(G_LEAD, G_REAR, 'idle')] * 8,
    'jab': [(arm((32, 30), (27, 42), (29, 37)), G_REAR, 'chamber'), (arm((32, 30), (29, 36), (33, 34)), G_REAR, 'snap'),
            (EXT_LEAD, G_REAR, 'hit'), (EXT_LEAD, G_REAR, 'hold'), (EXT_LEAD, G_REAR, 'hold'),
            (arm((32, 30), (30, 35), (33, 34)), G_REAR, 'recoil'), (arm((32, 30), (28, 40), (31, 36)), G_REAR, 'back'),
            (G_LEAD, G_REAR, 'idle')],
    'block': [(arm((32, 30), (31, 43), (36, 40), 'up'), G_REAR, 'raise')]
             + [(arm((32, 30), (31, 43), (36, 38), 'up'), G_REAR, 'brace')] * 5
             + [(arm((32, 30), (29, 42), (32, 37)), G_REAR, 'lower'), (G_LEAD, G_REAR, 'idle')],
    'punch': [(arm((32, 30), (26, 42), (28, 38)), G_REAR, 'windup'), (arm((32, 30), (25, 43), (27, 39)), G_REAR, 'windup'),
              (arm((32, 30), (30, 34), (33, 33)), G_REAR, 'thrust'), (EXT_LEAD, G_REAR, 'contact'),
              (EXT_LEAD, G_REAR, 'burst'), (EXT_LEAD, G_REAR, 'through'), (arm((32, 30), (29, 37), (32, 35)), G_REAR, 'collapse'),
              (G_LEAD, G_REAR, 'idle')],
    'swipe': [(arm((32, 30), (31, 43), (34, 29), 'up'), G_REAR, 'raise'), (arm((32, 30), (31, 43), (35, 26), 'up'), G_REAR, 'raise'),
              (arm((32, 30), (33, 30), (35, 33), 'down'), G_REAR, 'rip'), (arm((32, 30), (33, 31), (35, 36), 'down'), G_REAR, 'follow'),
              (arm((32, 30), (33, 31), (35, 36), 'down'), G_REAR, 'follow'), (arm((32, 30), (33, 31), (34, 35), 'down'), G_REAR, 'follow'),
              (arm((32, 30), (29, 40), (31, 36)), G_REAR, 'back'), (G_LEAD, G_REAR, 'idle')],
    # Shatter: she drives both gauntlets into a fracture, grabs its lips and wrenches them apart
    'shatter': [(arm((32, 30), (30, 42), (33, 33), 'up'), G_REAR, 'reach'),
                (arm((32, 30), (31, 43), (34, 31), 'up'), arm((19, 32), (8, 43), (13, 45)), 'grab'),
                (arm((32, 30), (31, 43), (34, 29), 'up'), arm((19, 32), (7, 44), (12, 46)), 'wrench'),
                (arm((32, 30), (31, 43), (35, 28), 'up'), arm((19, 32), (7, 45), (11, 47)), 'wrench'),
                (arm((32, 30), (31, 43), (35, 28), 'up'), arm((19, 32), (7, 45), (11, 47)), 'hold'),
                (arm((32, 30), (31, 43), (34, 30), 'up'), arm((19, 32), (8, 44), (12, 46)), 'hold'),
                (arm((32, 30), (29, 42), (32, 36)), G_REAR, 'settle'), (G_LEAD, G_REAR, 'idle')],
}
SECONDS = {'outfit': .12, 'jab': 24 / 60 / 8, 'block': 18 / 60 / 8, 'punch': .06, 'swipe': 24 / 60 / 8, 'shatter': 30 / 60 / 8}
# Pose looks: 0-6 ranks, 7 Worldbreaker #4-10, 8 #1 Between Worlds, 9 #3, 10 #2.
LOOKS = 11
LOOK_NAMES = RANKS[:7] + ['Worldbreaker #4-10', '#1 ' + TOP_NAME, '#3', '#2']


def steel(look):
    return STEEL


def fist_box(spec, big):
    """Screen box (x0, y0, x1, y1) of a placed fist, for effects that hang on it."""
    W = 14 if big else 13
    wx, wy = spec['wrist']
    if spec['orient'] == 'right':
        return wx, wy - W // 2, wx + 12, wy - W // 2 + W - 1
    if spec['orient'] == 'up':
        return wx - W // 2, wy - 12, wx - W // 2 + W - 1, wy
    return wx - W // 2, wy, wx - W // 2 + W - 1, wy + 12


def knuckles(spec, big):
    x0, y0, x1, y1 = fist_box(spec, big)
    if spec['orient'] == 'right':
        return x1, (y0 + y1) / 2
    if spec['orient'] == 'up':
        return (x0 + x1) / 2, y0
    return (x0 + x1) / 2, y1


def draw_arm(im, look, f, side, spec):
    d = ImageDraw.Draw(im)
    near = side == 'R'
    sleeve(d, spec['shoulder'], spec['elbow'], near)
    forearm(d, spec['elbow'], spec['wrist'], side, look, f, 8 if near else 7, 7 if near else 6)
    fist_at(im, look, f, side, spec['wrist'], spec['orient'])


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
    if style == 'ghost' and S is not None:
        # a dimensional afterimage: reality keeps an echo of her guard where the fist just was
        echo = S.load()
        for x, y in todo:
            c = echo[x, y]
            px_[x, y] = mix(c, SCAR_MID, .55) if c[3] else SCAR_VOID
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
        done = set(todo) - set(rest)
        for x, y in done:
            nd[x, y] = 0
        if not rest:
            break
        todo = rest
    return im


def mastery_fx(im, look, f, pose, frame, lead, rear):
    """The damage she does to reality, by rank (cosmetic: hitboxes, timing and damage never change)."""
    d = ImageDraw.Draw(im)
    lk = knuckles(lead, False)
    rk = knuckles(rear, True)
    lb = fist_box(lead, False)
    rb = fist_box(rear, True)
    moving = POSES[pose][frame][2] in ('snap', 'hit', 'thrust', 'contact', 'burst', 'rip', 'follow', 'wrench')
    if look == 0:      # Spark: sparks escape the gauntlet seams; no dimensional damage yet
        for k, (bx, by) in enumerate(((lb[0] + 1, lb[1] + 3), (rb[0] + 1, rb[3] - 3))):
            if (f + k) % 3 != 2:
                sx, sy = pt(bx, by, 2 + (f + k * 2) % 3, hsh(7, f, k) * TAU)
                px(d, sx, sy, (255, 236, 160, 255) if k else HEX)
    elif look == 1:    # Tear: a tiny jagged crack flickers when she flexes
        if f in (2, 6):
            scar(d, (lk[0] + 2, lk[1] - 5), (lk[0] + 4, lk[1] - 1), 0, 11 + f)
    elif look == 2:    # Rift: a thin violet scar hovers by the lead gauntlet, splitting open now and then
        depth = [0, 0, .8, 1.4, 1.8, 1.2, .5, 0][f]
        scar(d, (lb[2] + 3, lb[1] - 2), (lb[2] + 1, lb[1] + 6), depth, 21)
    elif look == 3:    # Gate: a small hole punched in the air by the lead fist, a compression ring round it
        hx, hy = lb[2] + 3, lb[1] - 3
        r = [1.5, 2, 2.5, 2.5, 2, 2, 1.5, 1.5][f]
        ring(d, hx, hy, r + 2, a(SCAR_EDGE, 120 + 30 * (f % 2)), 1, 10)
        dot(d, hx, hy, r, SCAR_VOID, SCAR_MID)
    elif look == 4:    # Wanderer: slices of the gauntlets slip behind tiny ruptures
        box = lb if f in (1, 2) else rb if f in (5, 6) else None
        if box:
            y = box[1] + 4 + (f % 2) * 3
            d.rectangle((box[0] + 2, y, box[2] - 1, y + 1), fill=SCAR_VOID)
            line(d, [(box[0] + 1, y - 1), (box[2], y - 1)], SCAR_EDGE)
            line(d, [(box[0] + 1, y + 2), (box[2], y + 2)], SCAR_MID)
    elif look == 5:    # Cartographer: tiny fractures join into a broken path round the punching arm
        cx, cy = (lb[0] + lb[2]) / 2, (lb[1] + lb[3]) / 2
        for k in range(5):
            if (k + f) % 5 == 4:
                continue
            a0, a1 = -2.4 + k * .9, -2.4 + k * .9 + .55
            scar(d, pt(cx, cy, 10, a0), pt(cx, cy, 10, a1), 0, 50 + k)
    elif look == 6:    # Riftwalker: fragments of the wound behind her drift outward past her shoulder
        for k in range(3):
            t = (f + k * 3) % 8
            shard(d, 10 - t, 24 + k * 5 - t // 2, 1, a(SCAR_EDGE, 230 - t * 25), outline=a(SCAR_MID, 230 - t * 25))
    if look >= 7:      # Worldbreaker and the podium: knuckle cracks expose the void under the metal
        for box, seed in ((lb, 70), (rb, 71)):
            x = box[2] - 2
            for k in range(2):
                y = box[1] + 3 + k * 5
                line(d, [(x - 3, y), (x - 1, y + 1), (x, y + 3)], SCAR_MID)
                px(d, x - 1, y + 1, SCAR_VOID)
            if (f + seed) % 4 == 0:
                px(d, x - 2, box[1] + 4, SCAR_EDGE)
        if moving:     # every strike leaves a short-lived fracture trail behind the fist
            scar(d, (lk[0] - 12, lk[1] + 3), (lk[0] - 3, lk[1]), .6, 80 + frame)
    if look == 9:      # #3: a persistent split rides the lead gauntlet and sometimes opens onto another world
        depth = 2.2 if f in (3, 4) else .7
        spine = scar(d, (lb[0] + 1, lb[1] - 3), (lb[2] + 2, lb[1] - 1), depth, 90)
        if f in (3, 4):
            vig = world(4, 4, 2, f)
            im.alpha_composite(vig, (round((lb[0] + lb[2]) / 2) - 2, lb[1] - 3))
    if look == 10:     # #2: both gauntlets carry their own ruptures, a displaced fragment floats between them
        scar(d, (lb[2] + 2, lb[1] + 1), (lb[2] + 1, lb[1] + 7), 1.6, 100 + (f > 3))
        scar(d, (rb[0] - 2, rb[1] + 2), (rb[0] - 1, rb[1] + 8), 1.4, 101 + (f > 3))
        fx_, fy = (lb[0] + rb[2]) // 2, (lb[3] + rb[1]) // 2 - 2 + [0, 0, -1, -1, 0, 0, 1, 1][f]
        frag = im.crop((fx_ - 6, fy - 4, fx_ - 2, fy))
        im.alpha_composite(frag, (fx_, fy - 6))
        d = ImageDraw.Draw(im)
        d.rectangle((fx_ - 1, fy - 7, fx_ + 4, fy - 1), outline=SCAR_EDGE)
    if look == 8:      # #1: her fingertips slip to the other side of reality for an instant
        for box in (lb, rb):
            if f in (2, 5):
                strip = im.crop((box[2] - 2, box[1] + 1, box[2] + 1, box[3]))
                im.paste((0, 0, 0, 0), (box[2] - 2, box[1] + 1, box[2] + 1, box[3]))
                im.alpha_composite(strip, (box[2] - 1, box[1] - 2))
                d = ImageDraw.Draw(im)
                line(d, [(box[2] - 3, box[1]), (box[2] - 3, box[3])], SCAR_EDGE)
        for k in range(5):     # and the chrome shimmers as light splits around it
            hx, hy = pt((lb[0] + lb[2]) / 2, (lb[1] + lb[3]) / 2, 10 + (k + f) % 2, k * TAU / 5 + f * .4)
            if 0 <= round(hx) < 48 and 0 <= round(hy) < 64 and im.getpixel((round(hx), round(hy)))[3] == 0:
                px(d, hx, hy, a(FRINGE_C if k % 2 else FRINGE_M, 90 + 40 * ((k + f) % 3)))


# How broken she is, by pose look: 0-6 ranks, then Worldbreaker #4-10, #3, #2 and #1 at the top.
SEVERITY = {0: 0, 1: 1, 2: 2, 3: 3, 4: 4, 5: 5, 6: 6, 7: 7, 9: 8, 10: 9, 8: 10}


def broken(im, look, f, lead, rear):
    """Rank is how broken the dimensions have left her: scars on her body and face, and gauntlets that crack,
    split their light, slip out of alignment and finally orbit apart. Each rank adds to the one before."""
    sev = SEVERITY[look]
    d = ImageDraw.Draw(im)
    lb = fist_box(lead, False)
    rb = fist_box(rear, True)
    pulse = sev >= 4 and f % 4 in (1, 2)
    hot = WHITE if pulse or sev >= 9 else SCAR_EDGE

    def mark(pts, col=None):
        line(d, pts, col or hot)
        if sev >= 6:     # the deeper scars show violet beside the light
            line(d, [(x + 1, y) for x, y in pts], SCAR_MID)
    if sev <= 2:         # a Spark still patches herself up
        line(d, [(25, 24), (26, 24)], TAPE)
    # the gauntlets: scratches, then a crack through a knuckle plate, then violet showing through
    line(d, [(lb[0] + 3, lb[1] + 3), (lb[0] + 5, lb[1] + 5)], STEEL['light'])
    if sev >= 1:
        line(d, [(rb[0] + 4, rb[1] + 4), (rb[0] + 7, rb[1] + 6)], STEEL['light'])
        mark([(rear['elbow'][0] + 2, rear['elbow'][1] - 1), (rear['wrist'][0] - 1, rear['wrist'][1] - 1)], SCAR_EDGE)  # forearm scar
    if sev >= 2:
        mark([(18, 31), (20, 32), (22, 31)])                                   # across the shoulder
        line(d, [(lb[2] - 2, lb[1] + 2), (lb[2] - 1, lb[1] + 4), (lb[2] - 2, lb[1] + 6)], INK)
    if sev >= 3:
        mark([(24, 26), (25, 24), (25, 23)])                                   # the face scar, cheek to eye
        for box in (lb, rb):
            line(d, [(box[0] + 6, box[1] + 2), (box[0] + 7, box[1] + 5), (box[0] + 6, box[1] + 8)], SCAR_MID)
    if sev >= 4:
        d.rectangle((22, 41, 23, 42), fill=SCAR_VOID)                          # the tunic torn through
        for box in (lb, rb):                                                   # a constant chromatic fringe
            line(d, [(box[0] - 1, box[1] + 3), (box[0] - 1, box[3] - 3)], FRINGE_C)
            line(d, [(box[2] + 2, box[1] + 4), (box[2] + 2, box[3] - 4)], FRINGE_M)
    if sev >= 5:
        mark([(27, 28), (28, 30)])                                             # down the neck
        mark([(lead['shoulder'][0] - 1, lead['shoulder'][1] + 2), (lead['elbow'][0], lead['elbow'][1] - 3)])
        strip = im.crop((lb[2] - 3, lb[1] + 1, lb[2] + 1, lb[1] + 5))         # a knuckle plate slips out of line
        im.alpha_composite(strip, (lb[2] - 2, lb[1]))
        d = ImageDraw.Draw(im)
    if sev >= 6:
        for q in ([(21, 34), (23, 36)], [(24, 33), (26, 35), (25, 37)], [(20, 37), (22, 39)]):
            mark(q)                                                            # a network of scars
        for k, (bx, by) in enumerate(((lb[2] + 3, lb[1] + 1), (lb[0] - 3, lb[3] - 2))):
            if (f + k) % 4 != 3:
                V_shard = shard
                V_shard(d, bx + (f % 2), by - (f // 4), 1, STEEL['metal'], outline=INK)   # plate fragments float off
    if sev >= 7:         # she glitches: a band of her slips a pixel sideways
        y = 18 + (f * 5) % 22
        band = im.crop((0, y, 47, y + 2))
        im.alpha_composite(band, (1, y))
        d = ImageDraw.Draw(im)
        for box in (lb, rb):
            line(d, [(box[0] + 2, box[1] + 6), (box[0] + 5, box[1] + 7), (box[0] + 8, box[1] + 6)], SCAR_VOID)
    if sev >= 8:         # #3: a sliver of her face is displaced through a tear
        sliver = im.crop((29, 21, 33, 25))
        im.alpha_composite(sliver, (30, 19 + (f % 2)))
        d = ImageDraw.Draw(im)
        line(d, [(29, 19), (30, 26)], SCAR_EDGE)
    if sev >= 9:         # #2: the arm scars burn, the cloak frays, the fingers slip
        for k in range(3):
            t = (f + k * 3) % 8
            px(d, 6 - t, 33 + k * 2 - t // 3, a(STEEL['light'], 230 - t * 25))
        strip = im.crop((lb[2] - 2, lb[1] + 2, lb[2] + 1, lb[3] - 2))
        im.alpha_composite(strip, (lb[2], lb[1] + 1))
        d = ImageDraw.Draw(im)
    if sev >= 10:        # #1: her whole outline splits across dimensions; the other world shows through her
        al = im.getchannel('A').point(lambda v: 255 if v else 0)
        for col, dx in ((FRINGE_C, -1), (FRINGE_M, 1)):
            ghost = Image.new('RGBA', im.size, col)
            ghost.putalpha(al.point(lambda v: v * 150 // 255))
            under = Image.new('RGBA', im.size)
            under.alpha_composite(ghost, (dx, 0))
            under.alpha_composite(im)
            im.paste(under, (0, 0))
        vig = world(8, 4, 6, f)
        im.alpha_composite(vig, (8, 31))                                       # through the cloak
        im.alpha_composite(world(3, 2, 3, f), (32, 23))                        # through her cheek
        d = ImageDraw.Draw(im)
        line(d, [(32, 22), (33, 26)], SCAR_EDGE)
        cx, cy = (lb[0] + lb[2]) / 2, (lb[1] + lb[3]) / 2
        for k in range(4):                                                     # plates orbiting the fist
            ang = k * TAU / 4 + f * TAU / 8
            shard(d, *pt(cx, cy, 10, ang, .6), 1.2, STEEL['light'], outline=INK)
        dot(d, cx + 1, cy, 1.5, SCAR_VOID, SCAR_MID)                           # the void glowing through it
    return im


def assemble_layers(look, f, pose, frame, fx=True):
    """Pose `pose` at beat `frame` for pose look `look`. No legs: the body sheet owns them."""
    lead, rear, beat = POSES[pose][frame]
    im = Image.new('RGBA', (48, 64))
    d = ImageDraw.Draw(im)
    flare = beat in ('wrench', 'hold')
    cloak(d, look, min(7, f + 2) if flare else f)
    # the far (lead) arm's shoulder sits behind the torso
    sleeve(d, lead['shoulder'], lead['elbow'], False)
    hips(d, TUNIC)
    torso(d, TUNIC, look, f)
    mantle(d, look, f)
    lead_l = Image.new('RGBA', (48, 64))
    if pose == 'punch' and beat in ('contact', 'burst', 'through'):
        reality_contact(ImageDraw.Draw(lead_l), look, f, beat, knuckles(lead, False))
    if pose == 'shatter' and beat in ('grab', 'wrench', 'hold'):
        pry_scar(ImageDraw.Draw(lead_l), look, frame, knuckles(lead, False), knuckles(rear, True))
    draw_arm(lead_l, look, f, 'L', lead)
    im.alpha_composite(lead_l)
    d = ImageDraw.Draw(im)
    hair(d, look, f + (2 if flare else 0))
    head(d, look, f)
    if pose == 'block' and beat == 'brace' and frame == 2:
        px(d, 26, 21, WHITE)                                                  # a glint off the glasses
    draw_arm(im, look, f, 'R', rear)
    d = ImageDraw.Draw(im)
    if beat in ('snap', 'thrust'):
        kx, ky = knuckles(lead, False)
        for k in range(3):
            line(d, [(kx - 14 + k, ky - 4 + k * 4), (kx - 6, ky - 4 + k * 4)], a(WHITE, 200 - k * 50))
    if pose == 'block' and beat == 'brace' and frame in (1, 2):
        ring(d, 41, 26, 4 + frame, a(HEX, 200), 1, 10, -1.3, 1.3)
    if beat == 'rip':
        kx, ky = knuckles(lead, False)
        scar(d, (kx + 4, ky - 18), (kx, ky + 2), 1.2, 300)
    if fx:
        mastery_fx(im, look, f, pose, frame, lead, rear)
        im = broken(im, look, f, lead, rear)
    return im


def reality_contact(d, look, f, beat, k):
    """The punch meeting reality: a flat white surface at the knuckles, the compression, cracks bursting
    out, and the fist tearing through."""
    kx, ky = k
    if beat == 'contact':
        line(d, [(kx + 1, ky - 6), (kx + 1, ky + 6)], WHITE)
        ring(d, kx + 1, ky, 4, a(SCAR_EDGE, 200), .5)
    elif beat == 'burst':
        for pth in cracks(500 + look, kx + 1, ky, 5, 7, TAU, .3, False):
            line(d, pth, SCAR_EDGE)
        scar(d, (kx + 1, ky - 6), (kx + 1, ky + 6), 1.6, 510)
    else:
        scar(d, (kx, ky - 8), (kx + 1, ky + 8), 2.6, 511, f)


def pry_scar(d, look, frame, lk, rk):
    """Shatter: the fracture she wrenches open between her raised lead fist and her low rear fist."""
    depth = {1: 1.2, 2: 2.6, 3: 3.6, 4: 4.0, 5: 3.2}[frame] * (1.3 if look == 8 else 1)
    scar(d, (lk[0] + 3, lk[1] + 1), (rk[0] + 3, rk[1] - 2), depth, 700 + look)


def costume(look, f, pose='outfit'):
    """The overlay for one pose at one frame, sealed over the body sheet."""
    im = assemble_layers(look, f, pose, f)
    moved = POSES[pose][f][0] is not G_LEAD or POSES[pose][f][1] is not G_REAR
    if moved and pose != 'block':
        return seal(im, 'ghost', assemble_layers(look, f, 'outfit', f, fx=False))
    return seal(im, 'near')


def outfit(look, f):
    return costume(look, f, 'outfit')


def punch(look, f):
    return costume(look, f, 'punch')


def body(kind='idle', f=0):
    """The registered body sheet: rank-0 guard. The upper body keeps one silhouette in every frame (the pose
    overlays always cover it); the legs and the effects around her carry the motion."""
    im = Image.new('RGBA', (48, 64))
    d = ImageDraw.Draw(im)
    if kind == 'dead':
        poly(d, [(12, 55), (16, 52), (32, 52), (35, 55), (32, 59), (14, 59)], fill=INK)
        d.rectangle((16, 53, 31, 57), fill=TUNIC['coat'])
        d.rectangle((17, 56, 30, 57), fill=TUNIC['shade'])
        poly(d, [(32, 51), (37, 50), (39, 53), (36, 56), (32, 56)], fill=SKIN, outline=INK)
        poly(d, [(33, 48), (38, 47), (41, 50), (38, 52), (34, 51)], fill=HAIR, outline=INK)
        line(d, [(34, 52), (37, 52)], INK)                                    # the glasses, askew
        poly(d, [(8, 52), (16, 50), (17, 54), (9, 56)], fill=CLOAK['deep'], outline=INK)
        fist_at(im, 0, 0, 'R', (2, 52), 'right')
        fist_at(im, 0, 0, 'L', (27, 53), 'right')
        d = ImageDraw.Draw(im)
        dim = [255, 200, 150, 110, 80, 55, 35, 20][f]
        px(d, 7, 53, a(HEX, dim))
        if f < 6:
            px(d, 8 + (f % 3), 47 - f // 2, a((255, 236, 160), 200 - f * 30))
        return im
    legs(d, kind, f)
    im.alpha_composite(assemble_layers(0, 0, 'outfit', 0, fx=False))
    d = ImageDraw.Draw(im)
    kx, ky = knuckles(G_LEAD, False)
    if kind == 'attack':       # the jab: a knuckle flash and a cyan shock off the lead fist
        if 1 <= f <= 4:
            r = [0, 2, 4, 3, 1][f]
            for k in range(3):
                line(d, [(kx + 3, ky - 4 + k * 4), (kx + 3 + r, ky - 4 + k * 4)], a(HEX, 230))
        else:
            px(d, kx + 4, ky + f % 3, a((200, 220, 230), 160))
    elif kind == 'skill1':
        for k in range(1 + f // 2):
            px(d, kx + 3, ky - 6 + k * 3, FAMILY[(k * 2 + f) % 10])
    elif kind == 'skill2':
        for k in range(f + 1):
            px(d, *pt(32, 34, 15, -1.3 + k * .3), RIFT_PALE if k < f else WHITE)
    elif kind == 'ult':
        for k in range(6):
            q = pt(18, 46, 3 + f, k * TAU / 6 + f * .2)
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
        px(d, 44, 22 - f // 2, a((255, 200, 160), 255 - f * 28))
    elif kind == 'idle':
        px(d, 14 + f, 61, a(HEX, 60))
    elif kind == 'run':
        px(d, 10 - f % 4, 60 - f // 4, a((150, 130, 110), 120))
    return im


def torso_only(look=0):
    """For the verifier: the bare torso, without head, arms or legs."""
    im = Image.new('RGBA', (48, 64))
    torso(ImageDraw.Draw(im), TUNIC, look, 0)
    return im


# ------------------------------------------------------------ rank emblems --
# Her rank, like Isliid's and Scribble's, sits at her top right: a chrome-rimmed medallion of cracked glass
# with her scar inside it, sixteen frames that loop seamlessly. Rank deepens the scar and adds moving parts;
# the Top 10 carry a number plate, fracture lightning from #3, and #1 a crown of floating glass.
EMBLEM = (88, 96)        # centred on her
EMBLEM_AT = (67, 28)     # +23 px right and -20 px up from her centre: clear of her hair and the HUD
EMBLEM_FRAMES = 16


def emblem(rank, f, position=None):
    im = Image.new('RGBA', EMBLEM)
    d = ImageDraw.Draw(im)
    cx, cy = EMBLEM_AT
    t = f / EMBLEM_FRAMES
    top = rank >= 7
    p = position or 10
    tier = 7 + (p <= 3) + (p <= 2) + (p == 1) if top else rank          # 0-6 ranks, 7 Top 10, 8 #3, 9 #2, 10 #1
    if p == 1 and top:
        hue = [FRINGE_C, (176, 120, 255, 255), FRINGE_M, (176, 120, 255, 255)]
        rim = mix(hue[(f // 4) % 4], hue[(f // 4 + 1) % 4], (f % 4) / 4)
    elif tier >= 4:
        rim = mix(STEEL['light'], FRINGE_C if f % 2 else FRINGE_M, .25)
    else:
        rim = STEEL['metal'] if tier < 2 else STEEL['light']
    # rotating fracture rays behind (#1), and the glow pulsing (from Rift)
    if p == 1 and top:
        for k in range(8):
            ang = k * TAU / 8 + t * TAU / 8
            reach = 16 if math.cos(ang) > -.3 else 11                         # shorter on the side facing her head
            line(d, [pt(cx, cy, 9, ang), pt(cx, cy, reach - 3, ang + .08), pt(cx, cy, reach, ang)], a(EDGE_RAY, 150 if k % 2 else 90))
    if tier >= 2:
        g = Image.new('RGBA', EMBLEM)
        dot(ImageDraw.Draw(g), cx, cy, 8 + tier * .3, a(SCAR_MID, 70 + 50 * math.sin(TAU * t)))
        im.alpha_composite(g.filter(ImageFilter.GaussianBlur(2)))
        d = ImageDraw.Draw(im)
    # the medallion: dark glass in a chrome rim
    dot(d, cx, cy, 7, (22, 20, 36, 255), INK)
    ring(d, cx, cy, 7, rim, 1, 24)
    ring(d, cx, cy, 6, mix(rim, INK, .5), 1, 20, math.pi * .2, math.pi * 1.1)
    if tier >= 4:
        px(d, cx - 7, cy, FRINGE_C)
        px(d, cx + 7, cy, FRINGE_M)
    # cracks across the glass, with light running outward along them
    n = 2 + min(tier, 8) // 2
    for k, pth in enumerate(cracks(500 + tier, cx, cy, n, 6, TAU, .4 + tier * .1, False)):
        line(d, pth[:4], (90, 96, 120, 255))
        j = int((t * 4 + k / n) * 4) % 4
        if j < len(pth):
            px(d, *pth[min(j, 3)], WHITE if tier >= 3 else STEEL['light'])
    # the scar at the centre breathes open and shut
    depth = [0, .6, 1.0, 1.3, 1.8, 2.2, 2.6, 2.8, 3.0, 3.2, 3.4][tier] * (.8 + .25 * math.sin(TAU * t))
    scar(d, (cx - 1, cy - 5), (cx + 1, cy + 5), depth if tier else 0, 520 + tier, f)
    if tier == 10:
        im.alpha_composite(world(8, 2, 4, f % 8), (cx - 1, cy - 2))             # the other world inside
        d = ImageDraw.Draw(im)
    # glass shards orbiting; a second ring at #2 and #1
    speed = 1 + min(tier, 9) // 3
    rings = [(rank + 1 if not top else 7, 10.5)] + ([(5, 13)] if tier >= 9 else [])
    tips = []
    for ri, (count, r) in enumerate(rings):
        for k in range(count):
            ang = TAU * (t * speed * (1 if ri == 0 else -1) + k / count)
            x, y = pt(cx, cy, r, ang, .9)
            tips.append((x, y))
            col = STEEL['light'] if (k + ri) % 3 else (FRINGE_C if k % 2 else FRINGE_M) if tier >= 4 else SCAR_EDGE
            shard(d, x, y, 1 if tier < 7 else 1.2, col, outline=INK)
    # fracture lightning between neighbouring shards from #3; more at #2, every colour at #1
    if tier >= 8:
        every = {8: 4, 9: 3, 10: 2}[tier]
        for k in range(len(tips) if tier < 9 else rings[0][0]):
            if (f + k) % every:
                continue
            a0, a1 = tips[k], tips[(k + 1) % rings[0][0]]
            mid = ((a0[0] + a1[0]) / 2 + (hsh(f, k) - .5) * 3, (a0[1] + a1[1]) / 2 + (hsh(k, f) - .5) * 3)
            col = [FRINGE_C, FRINGE_M, (176, 120, 255, 255)][k % 3] if tier == 10 else SCAR_EDGE
            line(d, [a0, mid, a1], col)
    # sparks shed off the rim
    for k in range(1 + min(tier, 8)):
        ph = (t * 2 + k / (1 + min(tier, 8))) % 1
        if ph < .5:
            x, y = pt(cx, cy, 8 + ph * 10, (hsh(77, k) - .5) * math.pi * 1.1)      # shed away from her head
            px(d, x, y, a(WHITE if k % 2 else SCAR_EDGE, 255 * (1 - ph * 2)))
    if not top:      # rank pips under the medallion, the lit one cycling from Rift
        for k in range(rank + 1):
            x = cx - rank * 1.5 + k * 3
            lit = rank >= 2 and k == f % (rank + 1)
            px(d, x, cy + 10, WHITE if lit else STEEL['metal'])
    else:            # the number plate
        label = '#' + str(p)
        w = 4 * len(label) + 2
        d.rectangle((cx - w // 2 - 1, cy + 9, cx + w // 2, cy + 16), fill=(28, 26, 40, 255), outline=rim)
        x0 = cx - (4 * len(label) - 1) // 2
        hashp = [(0, 1), (2, 1), (0, 3), (2, 3), (1, 0), (1, 1), (1, 2), (1, 3), (1, 4), (0, 2), (2, 2)]
        for col, off in ((INK, 1), (WHITE if p > 3 else rim if p > 1 else WHITE, 0)):
            for dx_, dy_ in hashp:
                px(d, x0 + dx_ + off, cy + 11 + dy_ + off, col)
            digits(d, x0 + 4 + off, cy + 11 + off, str(p), col)
    if p == 1 and top:   # a crown of floating glass shards, bobbing
        bob = round(math.sin(TAU * t) * 1.5)
        for k in range(5):
            x = cx - 6 + k * 3
            y = cy - 13 - (2 if k == 2 else 1 if k in (1, 3) else 0) + bob
            shard(d, x, y, 1.2, [FRINGE_C, WHITE, (176, 120, 255, 255), WHITE, FRINGE_M][k], outline=INK)
        glint = cx - 6 + (f % 8) * 1.5
        px(d, glint, cy - 15 + bob, WHITE)
    return glow(im, .8 + .05 * tier, 1.4)


EDGE_RAY = (206, 246, 255, 255)


# ---------------------------------------------------------- scars behind her --

def back_scar(kind, f):
    """The wound behind her near shoulder (Riftwalker, Worldbreakers) or the massive tear (#1)."""
    im = Image.new('RGBA', AURA)
    d = ImageDraw.Draw(im)
    bx, by = BODY
    if kind == 'wound':
        x, y = bx + 9, by + 14
        scar(d, (x + 3, y), (x - 2, y + 26), 3.2, 41, f)
        for k in range(4):
            t = (f + k * 2) % 8
            shard(d, x - 6 - t * 1.5, y + 6 + k * 5 - t, 1.2, a(SCAR_EDGE, 230 - t * 25), outline=a(SCAR_MID, 230 - t * 25))
        px(d, x + 1, y + 2 + (f * 3) % 22, WHITE)
        return glow(im, .8, 1.3)
    # #1: a massive irregular tear suspended behind her, the world beyond showing through
    cx, cy = bx + 22, by + 20
    spine = scar(d, (cx + 10, cy - 30), (cx - 14, cy + 34), 9.5, 61, f)
    vig = world(8, 10, 22, f)
    paste_masked(im, vig, cx - 6, cy - 6, [(cx - 4, cy - 10), (cx + 3, cy - 12), (cx + 2, cy + 10), (cx - 5, cy + 12)])
    d = ImageDraw.Draw(im)
    for pth in cracks(62, cx + 10, cy - 30, 3, 10, TAU * .5, -2.4, False) + cracks(63, cx - 14, cy + 34, 3, 10, TAU * .5, .8, False):
        line(d, pth, a(SCAR_EDGE, 200))
    q = spine[(f * 2) % len(spine)]
    sparkle(d, q[0], q[1], a(GOLD_WHITE, 230), 2)
    for k in range(6):
        t = (f + k) % 8
        shard(d, cx + 14 + t * 2, cy - 20 + k * 9 - t, 1.4, a(GOLD_WHITE if k % 2 else SCAR_EDGE, 230 - t * 25))
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

    def pose_look(r, podium):
        return r if r < 7 else {1: 8, 2: 10, 3: 9}.get(podium, 7)

    def composed(r, f, podium=10, pose='outfit', emblem=False):
        im = Image.new('RGBA', (AW, AH))
        if r >= 6:
            im.alpha_composite(original('back_top1' if r == 7 and podium == 1 else 'back_wound', f, AW, AH))
        base = body('idle', f)
        base.alpha_composite(original(f'{pose}_r{pose_look(r, podium)}', f, 48, 64))
        im.alpha_composite(base, BODY)
        if emblem:
            tag = f'top{podium}' if r == 7 else f'rank{r}'
            im.alpha_composite(original(tag, f, *EMBLEM), (BODY[0] + 24 - EMBLEM[0] // 2, BODY[1] + 32 - EMBLEM[1] // 2))
        return im

    variants = [(r, 10) for r in range(7)] + [(7, 10), (7, 3), (7, 2), (7, 1)]
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
    cards = [(0, 10, 'SPARK', RIFT_PALE), (6, 10, 'RIFTWALKER', (190, 170, 255, 255)), (7, 1, 'BETWEEN WORLDS', GOLD)]
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
    title(d, 24, 56, 'The higher the rank, the more broken the dimensions have left her; rank never adds power. 2x, runtime frame 2.', 13)
    for j, ((r, p), label) in enumerate(zip(variants, labels)):
        row, col = divmod(j, 6)
        x, y = 16 + col * cw, 90 + row * (ch + 40)
        d.rounded_rectangle((x, y, x + cw - 8, y + ch), 8, fill='#151a33')
        im.alpha_composite(big(composed(r, 2, p, emblem=True)), (x, y + 4))
        title(d, x + 8, y + ch - 22, f'{r} {label}' if r < 7 else label, 12, '#ffd678' if p == 1 else '#dcecf4')
    im.convert('RGB').save(docs / 'vesper-mastery.png')
    # docs: the animated poses, every frame, at Gate (rows) and the rift punch for every look
    names = [('outfit', 'GUARD'), ('jab', 'JAB'), ('block', 'BLOCK'), ('punch', 'RIFT PUNCH'), ('swipe', 'SWIPE'), ('shatter', 'SHATTER (PRY)')]
    im = Image.new('RGBA', (140 + 11 * 104, 70 + len(names) * 136 + 150), bg)
    d = ImageDraw.Draw(im)
    title(d, 24, 16, 'POSES / EIGHT FRAMES EACH (2x, Worldbreaker look); BELOW: THE GUARD FOR EVERY RANK', 18)
    for row, (pose, label) in enumerate(names):
        y = 60 + row * 136
        title(d, 16, y + 56, label, 13)
        for f in range(8):
            base = body('idle', 0)
            base.alpha_composite(original(f'{pose}_r7', f, 48, 64))
            im.alpha_composite(big(base), (140 + f * 104, y))
    y = 60 + len(names) * 136
    for look in range(LOOKS):
        base = body('idle', 0)
        base.alpha_composite(original(f'outfit_r{look}', 2, 48, 64))
        im.alpha_composite(big(base), (16 + look * 104, y))
        title(d, 20 + look * 104, y + 130, ['Spark', 'Tear', 'Rift', 'Gate', 'Wanderer', 'Cartogr.', 'Riftwalker', 'WB #4-10', '#1', '#3', '#2'][look], 11)
    im.convert('RGB').save(docs / 'vesper-gauntlets.png')
    # docs: animated ladder gif
    frames = []
    for f in range(8):
        sc = Image.new('RGBA', (4 * (AW * 2 + 8) + 16, AH * 2 + 70), bg)
        d = ImageDraw.Draw(sc)
        title(d, 16, 12, 'SPARK  /  GATE  /  RIFTWALKER  /  #1 BETWEEN WORLDS', 18)
        for i, (r, p) in enumerate([(0, 10), (3, 10), (6, 10), (7, 1)]):
            sc.alpha_composite(big(composed(r, f, p, emblem=True)), (16 + i * (AW * 2 + 8), 50))
        frames.append(sc.convert('RGB').quantize(colors=256))
    frames[0].save(docs / 'vesper-mastery.gif', save_all=True, append_images=frames[1:], duration=120, loop=0, disposal=2)
    return original
