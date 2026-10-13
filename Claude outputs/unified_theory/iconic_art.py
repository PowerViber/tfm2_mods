"""Iconic, cosmic artwork for The Unified Theory (round 125).

Each scientist is recognisable by the things they are known for:
Einstein - E=mc², pocket watch, melting time, spacetime grids, Einstein rings.
Newton   - the apple, the prism and its rainbow, orbits, the reflecting
           telescope, Principia, comets.
Curie    - glowing radium, Bohr atoms, Ra/Po element tiles, alpha/beta/gamma
           rays and her two Nobel medals.

Mastery is visual only. Every rank adds one recognisable layer; Unified Mind
#1 shows the other two scientists as translucent figures at the shoulders.
All functions are deterministic: eight discrete phases, no randomness.
"""
import math
import importlib.util
from functools import lru_cache
from pathlib import Path

from PIL import Image, ImageDraw, ImageFilter, ImageFont

TAU = math.tau
HERE = Path(__file__).resolve().parent
_spec = importlib.util.spec_from_file_location('science_compact_art_iconic', HERE/'compact_art.py')
compact = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(compact)
_cspec = importlib.util.spec_from_file_location('science_cosmic_art_iconic', HERE/'cosmic_art.py')
cosmic = importlib.util.module_from_spec(_cspec)
_cspec.loader.exec_module(cosmic)

# Persistent aura canvas. The 48 x 64 body sits at BODY so its centre matches
# the canvas centre (the engine anchors every frame at its centre).
AURA = (160, 176)
BODY = (56, 56)
CAST = 176          # activation canvas, centred on the body
FIELD = 176
PACKET = 64
IMPACT = 96
COMPLETE = 192
ACCIDENT = 96
MARK = 72

WHITE = (246, 250, 255, 255)
CHALK = (232, 236, 226, 255)
INK = (14, 18, 30, 255)
# Einstein: electric blue and violet light.
E_BLUE = (92, 178, 255, 255)
E_VIOLET = (170, 124, 255, 255)
E_PALE = (190, 226, 255, 255)
# Newton: gold, apple red and a full spectrum.
GOLD = (255, 204, 92, 255)
GOLD_DARK = (176, 116, 40, 255)
APPLE = (226, 52, 48, 255)
APPLE_DARK = (138, 24, 30, 255)
LEAF = (96, 196, 72, 255)
SPECTRUM = [(255, 64, 64), (255, 150, 40), (255, 228, 64), (86, 220, 92),
            (64, 160, 255), (92, 92, 236), (176, 92, 236)]
# Curie: radium green and its glow.
RADIUM = (118, 255, 172, 255)
RADIUM_DEEP = (40, 196, 140, 255)
CYAN = (110, 236, 236, 255)
GLASS = (196, 232, 236, 255)
ACCENT = [E_BLUE, GOLD, RADIUM]
SECOND = [E_VIOLET, APPLE, CYAN]

FONT_PATH = '/usr/share/fonts/truetype/dejavu/DejaVuSansMono-Bold.ttf'


def a(color, alpha):
    return color[:3] + (max(0, min(255, round(alpha))),)


def mix(c1, c2, t):
    return tuple(round(x + (y - x) * t) for x, y in zip(c1[:3], c2[:3])) + (255,)


def pt(x, y, r, ang, flat=1.0):
    return x + r * math.cos(ang), y + r * math.sin(ang) * flat


def line(d, pts, color, width=1):
    d.line([(round(x), round(y)) for x, y in pts], fill=color, width=width)


def dot(d, x, y, r, color, outline=None):
    d.ellipse((round(x - r), round(y - r), round(x + r), round(y + r)), fill=color, outline=outline)


def sparkle(d, x, y, color=WHITE, r=2):
    line(d, [(x - r, y), (x + r, y)], color)
    line(d, [(x, y - r), (x, y + r)], color)
    d.point((round(x), round(y)), fill=WHITE)


def orbit(d, x, y, rx, ry, color, width=1, start=0.0, end=TAU, steps=48):
    pts = [(x + rx * math.cos(start + (end - start) * k / steps),
            y + ry * math.sin(start + (end - start) * k / steps)) for k in range(steps + 1)]
    line(d, pts, color, width)


def glow(im, radius=2.0, boost=1.7):
    """A bright baked halo: blurred, alpha boosted, then the crisp sprite on top."""
    halo = im.filter(ImageFilter.GaussianBlur(radius))
    alpha = halo.getchannel('A').point(lambda v: min(255, round(v * boost)))
    halo.putalpha(alpha)
    halo.alpha_composite(im)
    return halo


@lru_cache(maxsize=8)
def font(size):
    return ImageFont.truetype(FONT_PATH, size)


def text(d, x, y, s, color, size=8):
    """Crisp, non-antialiased glyphs (chalk equations, element symbols)."""
    d.fontmode = '1'
    d.text((round(x), round(y)), s, font=font(size), fill=color)


def emc2(d, x, y, color=CHALK, size=8):
    text(d, x, y, 'E=mc', color, size)
    w = font(size).getbbox('E=mc')[2]
    text(d, x + w, y - size * .45, '2', color, max(6, size - 3))


# ---------------------------------------------------------------- Einstein --

def pocket_watch(d, x, y, r, f, color=E_BLUE):
    ph = f * TAU / 8
    dot(d, x, y, r, (22, 34, 58, 255), GOLD)
    if r >= 4:
        d.ellipse((round(x - r + 1), round(y - r + 1), round(x + r - 1), round(y + r - 1)), outline=a(color, 200))
    d.rectangle((round(x - 1), round(y - r - 2), round(x + 1), round(y - r)), fill=GOLD)
    line(d, [(x, y), pt(x, y, r * .75, ph - math.pi / 2)], WHITE)
    line(d, [(x, y), pt(x, y, r * .5, ph * .25 - math.pi / 2)], GOLD)


def melting_clock(d, x, y, r, f):
    """A soft clock drooping over itself: time slows near mass."""
    ph = f * TAU / 8
    sag = 2 + 2 * math.sin(ph)
    pts = []
    for k in range(25):
        t = k / 24 * TAU
        px, py = pt(x, y, r, t, .7)
        if math.sin(t) > 0:
            py += sag * math.sin(t) ** 2 * (1 + .6 * math.cos(t * 2))
        pts.append((px, py))
    d.polygon([(round(px), round(py)) for px, py in pts], fill=(232, 216, 160, 235), outline=GOLD_DARK)
    drip_x = x + r * .45
    line(d, [(drip_x, y + r * .5), (drip_x + 1, y + r * .5 + 3 + sag)], (232, 216, 160, 235), 2)
    for k in range(4):
        tx, ty = pt(x, y, r * .72, k * TAU / 4 - math.pi / 2, .7)
        d.point((round(tx), round(ty)), fill=INK)
    line(d, [(x, y), pt(x, y, r * .6, ph * .5 - math.pi / 2, .7)], INK)
    line(d, [(x, y), pt(x, y, r * .4, ph * .12, .7)], INK)


def spacetime_grid(d, x, y, w, h, f, dent=0.0, color=E_BLUE, cells=6, alpha=170):
    """An oval perspective floor; `dent` pulls the centre down like a mass on a sheet."""
    ph = f * TAU / 8

    def warp(u, v):
        rr = u * u + v * v
        sink = dent * math.exp(-rr * 3.2)
        ripple = .7 * math.sin(math.sqrt(rr) * 9 - ph) * (1 - min(1, rr))
        return x + u * w, y + v * h + sink + ripple

    for k in range(1, cells):
        t = -1 + 2 * k / cells
        span = math.sqrt(max(0, 1 - t * t))
        fade = alpha * (.45 + .55 * span)
        for horizontal in (False, True):
            pts = []
            for j in range(17):
                q = -span + 2 * span * j / 16
                pts.append(warp(q, t) if horizontal else warp(t, q))
            line(d, pts, a(color, fade))
    rim = [warp(math.cos(j * TAU / 40), math.sin(j * TAU / 40)) for j in range(41)]
    line(d, rim, a(mix(color, WHITE, .4), alpha))


def einstein_ring(d, x, y, r, f, color=E_PALE, images=0, flat=1.0):
    ph = f * TAU / 8
    for k in range(32):
        t0, t1 = k * TAU / 32, (k + 1) * TAU / 32
        bright = .55 + .45 * math.sin(t0 * 2 - ph)
        line(d, [pt(x, y, r, t0, flat), pt(x, y, r, t1, flat)], a(color, 120 + 135 * bright), 2)
    for k in range(images):
        ix, iy = pt(x, y, r, k * TAU / images + math.pi / 4 + ph * .05, flat)
        dot(d, ix, iy, 1.5, WHITE)
        sparkle(d, ix, iy, E_PALE, 2)
    dot(d, x, y, max(1, r * .18), (255, 236, 196, 255))


def light_beam(d, x0, y0, x1, y1, color=E_PALE, width=2):
    line(d, [(x0, y0), (x1, y1)], a(color, 120), width + 2)
    line(d, [(x0, y0), (x1, y1)], color, width)
    line(d, [(x0, y0), (x1, y1)], WHITE, 1)


def star(d, x, y, r, color=WHITE):
    dot(d, x, y, max(1, r * .45), color)
    line(d, [(x - r, y), (x + r, y)], a(color, 200))
    line(d, [(x, y - r), (x, y + r)], a(color, 200))


def constellation(d, pts, f, color=E_PALE):
    for (x0, y0), (x1, y1) in zip(pts, pts[1:]):
        line(d, [(x0, y0), (x1, y1)], a(color, 110))
    for k, (x, y) in enumerate(pts):
        star(d, x, y, 1 + (k + f) % 3 // 2, WHITE if (k + f) % 4 == 0 else color)


# ------------------------------------------------------------------ Newton --

def apple(d, x, y, r, f=0, bite=False):
    r = max(2, r)
    dot(d, x - r * .25, y, r * .85, APPLE)
    dot(d, x + r * .25, y, r * .85, APPLE)
    dot(d, x, y + r * .15, r * .8, APPLE)
    d.arc((round(x - r), round(y - r), round(x + r), round(y + r * 1.05)), 20, 160, fill=APPLE_DARK)
    dot(d, x - r * .4, y - r * .3, max(1, r * .22), (255, 170, 160, 255))
    line(d, [(x, y - r * .7), (x + r * .15, y - r * 1.25)], (96, 60, 30, 255), max(1, round(r * .18)))
    d.polygon([(round(x + r * .15), round(y - r * 1.05)), (round(x + r * .8), round(y - r * 1.35)),
               (round(x + r * .55), round(y - r * .85))], fill=LEAF)
    if bite:
        dot(d, x + r * .85, y, r * .35, (0, 0, 0, 0))


def prism(d, x, y, r, f=0, rainbow=0, angle=0.0):
    """Glass triangle; white light in from the left, a spectrum fans out."""
    pts = [pt(x, y, r, angle - math.pi / 2), pt(x, y, r, angle + math.pi / 6), pt(x, y, r, angle + 5 * math.pi / 6)]
    d.polygon([(round(px), round(py)) for px, py in pts], fill=(200, 236, 255, 120), outline=WHITE)
    if rainbow:
        light_beam(d, x - rainbow * .7, y + 1, x, y + 1, WHITE, 1)
        spread = .5
        for k, c in enumerate(SPECTRUM):
            t = angle + (k - 3) / 6 * spread + .06 * math.sin(f * TAU / 8 + k)
            ex, ey = pt(x, y, rainbow, t)
            line(d, [(x + 1, y + 1), (ex, ey)], c + (230,), 2 if rainbow > 18 else 1)


def rainbow_ring(d, x, y, r, f, flat=.55, width=1):
    for k, c in enumerate(SPECTRUM):
        orbit(d, x, y, r + k * width, (r + k * width) * flat, c + (210,), width)


def planet(d, x, y, r, color, ring=False, f=0):
    dot(d, x, y, r, color)
    dot(d, x - r * .35, y - r * .35, max(1, r * .35), mix(color, WHITE, .5))
    if ring:
        orbit(d, x, y, r * 1.8, r * .55, a(GOLD, 220))


def sun(d, x, y, r, f, color=(255, 214, 96, 255)):
    ph = f * TAU / 8
    for k in range(12):
        t = k * TAU / 12 + ph * .1
        line(d, [pt(x, y, r * 1.15, t), pt(x, y, r * (1.5 + .25 * math.sin(ph + k)), t)], a(color, 210))
    dot(d, x, y, r, color)
    dot(d, x - r * .3, y - r * .3, r * .45, (255, 248, 210, 255))


def orrery(d, x, y, r, f, planets=4, flat=.42):
    ph = f * TAU / 8
    for k in range(planets):
        rr = r * (.38 + .2 * k)
        orbit(d, x, y, rr, rr * flat, a(GOLD, 150))
    sun(d, x, y, r * .16, f)
    colors = [(160, 196, 255, 255), APPLE, (120, 220, 140, 255), (236, 170, 96, 255), E_VIOLET]
    for k in range(planets):
        rr = r * (.38 + .2 * k)
        t = ph * (1.2 - k * .2) + k * 1.7
        px, py = pt(x, y, rr, t, flat)
        planet(d, px, py, 1.5 + k * .5, colors[k % len(colors)], ring=(k == 3))


def telescope(d, x, y, f):
    """Newton's reflecting telescope: a short wide tube on a stand."""
    line(d, [(x - 8, y + 4), (x + 8, y - 4)], GOLD_DARK, 5)
    line(d, [(x - 8, y + 4), (x + 8, y - 4)], GOLD, 3)
    dot(d, x + 8, y - 4, 2.5, (40, 52, 80, 255), GOLD)
    line(d, [(x - 2, y + 1), (x - 4, y + 10)], GOLD_DARK)
    line(d, [(x - 2, y + 1), (x + 2, y + 10)], GOLD_DARK)
    sparkle(d, x + 8, y - 4, WHITE, 1 + f % 2)


def comet(d, x, y, f, angle=math.pi * .85, length=22):
    for k in range(length):
        tx, ty = pt(x, y, k, angle)
        d.point((round(tx), round(ty + .4 * math.sin(k * .5 + f))), fill=a(E_PALE, 220 - k * 9))
        if k % 3 == 0:
            d.point((round(tx), round(ty + 1)), fill=a(GOLD, 160 - k * 6))
    dot(d, x, y, 2, WHITE)


def page(d, x, y, f, angle=0.0, size=7):
    c, s = math.cos(angle), math.sin(angle)
    corners = [(-size, -size * .7), (size, -size * .7), (size, size * .7), (-size, size * .7)]
    pts = [(x + u * c - v * s, y + u * s + v * c) for u, v in corners]
    d.polygon([(round(px), round(py)) for px, py in pts], fill=(240, 226, 190, 235), outline=GOLD_DARK)
    for k in range(3):
        v = -size * .35 + k * size * .3
        u0, u1 = -size * .7, size * (.4 + .2 * (k % 2))
        line(d, [(x + u0 * c - v * s, y + u0 * s + v * c), (x + u1 * c - v * s, y + u1 * s + v * c)], (120, 96, 70, 255))


# ------------------------------------------------------------------- Curie --

def vial(d, x, y, r, f, intensity=1.0):
    ph = f * TAU / 8
    pulse = .75 + .25 * math.sin(ph)
    dot(d, x, y + r * .3, r * 1.9, a(RADIUM, 40 * intensity * pulse))
    d.rectangle((round(x - r * .55), round(y - r), round(x + r * .55), round(y + r * .9)), fill=(30, 54, 60, 230), outline=GLASS)
    d.rectangle((round(x - r * .45), round(y), round(x + r * .45), round(y + r * .8)), fill=a(RADIUM, 140 + 100 * pulse * intensity))
    d.rectangle((round(x - r * .35), round(y - r * 1.35), round(x + r * .35), round(y - r)), fill=(150, 110, 80, 255))
    d.point((round(x - r * .2), round(y + r * .2 + (f % 3))), fill=WHITE)


def bohr_atom(d, x, y, r, f, color=RADIUM, shells=3, flat=.42):
    ph = f * TAU / 8
    for k in range(shells):
        tilt = k * math.pi / shells
        pts = []
        for j in range(33):
            t = j * TAU / 32
            u, v = r * math.cos(t), r * flat * math.sin(t)
            pts.append((x + u * math.cos(tilt) - v * math.sin(tilt), y + u * math.sin(tilt) + v * math.cos(tilt)))
        line(d, pts, a(color, 190))
        t = ph * (1 + .3 * k) + k
        u, v = r * math.cos(t), r * flat * math.sin(t)
        ex, ey = x + u * math.cos(tilt) - v * math.sin(tilt), y + u * math.sin(tilt) + v * math.cos(tilt)
        dot(d, ex, ey, 1.2, WHITE)
    dot(d, x, y, max(1.5, r * .2), (255, 250, 210, 255))


def element_tile(d, x, y, symbol, number, color=RADIUM):
    d.rectangle((round(x - 7), round(y - 7), round(x + 7), round(y + 7)), fill=(18, 44, 46, 235), outline=color)
    text(d, x - 5, y - 4, symbol, WHITE, 8)
    text(d, x - 6, y - 8, str(number), color, 6)


def ray_fan(d, x, y, f, length=26):
    """Rutherford's alpha / beta / gamma separation as the classic diagram."""
    ph = f * TAU / 8
    grow = .55 + .45 * (f + 1) / 8
    L = length * grow
    line(d, [(x, y), (x + L * .9, y - L * .35)], a((255, 120, 96, 255), 230), 2)   # alpha bends a little
    pts = [(x + k / 12 * L, y + (k / 12) ** 2 * L * .65) for k in range(13)]
    line(d, pts, a(CYAN, 230), 1)                                                      # beta bends strongly
    line(d, [(x, y), (x + L * 1.15, y)], a(RADIUM, 255), 1)                            # gamma straight through
    for k in range(3):
        sparkle(d, x + L * (.35 + .25 * k) + math.sin(ph + k), y - 2 + k, WHITE, 1)


def radium_cloud(im, x, y, r, f):
    d = ImageDraw.Draw(im)
    ph = f * TAU / 8
    for k in range(18):
        t = k * 2.399 + ph * .3
        rr = r * (.3 + .7 * ((k * 7 + f) % 11) / 11)
        px, py = pt(x, y, rr, t, .75)
        c = mix(RADIUM, WHITE, .35 * (k % 3) / 2)
        if k % 3 == 0:
            sparkle(d, px, py, a(c, 210), 1)
        else:
            dot(d, px, py, .8 + k % 2 * .7, a(c, 150 + (k % 4) * 25))


def glow_pool(d, x, y, r, f):
    """Radium light pooled on the floor, pulsing outward."""
    ph = f * TAU / 8
    for k in range(3):
        rr = r * (.45 + .25 * k + .06 * math.sin(ph - k))
        orbit(d, x, y, rr, rr * .28, a(mix(RADIUM, WHITE, .2 * k), 190 - k * 45))
    for k in range(10):
        gx, gy = pt(x, y, r * (.3 + .6 * ((k * 3 + f) % 7) / 7), k * TAU / 10 + ph * .2, .28)
        d.point((round(gx), round(gy)), fill=a(WHITE, 200))


def nobel_medal(d, x, y, r, f, ribbon=True):
    if ribbon:
        d.polygon([(round(x - r * .6), round(y - r * 2.2)), (round(x), round(y - r * .8)),
                   (round(x + r * .6), round(y - r * 2.2))], fill=(60, 112, 200, 255))
    dot(d, x, y, r, GOLD, GOLD_DARK)
    dot(d, x - r * .15, y - r * .1, r * .55, (236, 180, 76, 255))
    d.point((round(x - r * .3), round(y - r * .3)), fill=WHITE)
    sparkle(d, x + r * .7, y - r * .7, WHITE, 1 + (f % 2))


def nucleus_star(im, x, y, r, f, color=RADIUM):
    d = ImageDraw.Draw(im)
    ph = f * TAU / 8
    for k in range(16):
        t = k * TAU / 16 + ph * .08
        line(d, [pt(x, y, r * 1.1, t), pt(x, y, r * (1.6 + .35 * math.sin(ph * 2 + k)), t)], a(color, 170))
    dot(d, x, y, r, a(color, 255))
    for k in range(7):
        px, py = pt(x, y, r * .45, k * TAU / 7 + ph * .2)
        dot(d, px, py, max(1, r * .22), (255, 120, 96, 255) if k % 2 else (150, 210, 255, 255))
    dot(d, x - r * .3, y - r * .3, r * .3, (240, 255, 240, 255))


# -------------------------------------------------------------- the trio ---

@lru_cache(maxsize=64)
def figure(science, f):
    """The scientist's actual body + rank-5 costume, for ghosts and resonance."""
    im = compact.body('idle', f)
    im.alpha_composite(compact.costume(science, 5, f))
    return im


def ghost(science, f, alpha=96, tint=None):
    im = figure(science, f).copy()
    tint = tint or ACCENT[science]
    r, g, b, al = im.split()
    lum = Image.merge('RGB', (r, g, b)).convert('L')
    colored = Image.merge('RGBA', (
        lum.point(lambda v: round(v * .35 + tint[0] * .65)),
        lum.point(lambda v: round(v * .35 + tint[1] * .65)),
        lum.point(lambda v: round(v * .35 + tint[2] * .65)),
        al.point(lambda v: round(v * alpha / 255))))
    return colored


# -------------------------------------------------------------- the auras --

def hand(science):
    """Hand positions on the aura canvas (body pasted at BODY)."""
    bx, by = BODY
    return [(bx + 13, by + 40), (bx + 9, by + 43), (bx + 33, by + 41)][science]


def aura(science, rank, f, podium=4):
    """One baked background loop; each rank adds one recognisable layer."""
    im = Image.new('RGBA', AURA)
    d = ImageDraw.Draw(im)
    cx, cy = AURA[0] / 2, AURA[1] / 2 + 4          # body centre
    hx, hy = hand(science)
    head = (cx + 1, cy - 20)
    feet = (cx, cy + 25)
    ph = f * TAU / 8
    top = rank == 7
    if science == 0:
        if top:
            if podium == 1:
                # Spacetime Fabric: a warped universe around him.
                spacetime_grid(d, cx, feet[1] - 2, 70, 16, f, dent=10, color=E_BLUE, cells=10, alpha=150)
                einstein_ring(d, cx, cy - 6, 50, f, E_PALE, images=0, flat=.9)
                for k in range(3):
                    t = ph * (1 - .2 * k) + k * 2.1
                    sx, sy = pt(cx, cy - 6, 50, t, .9)
                    light_beam(d, cx, cy - 6, sx, sy, a(E_PALE, 150), 1)
                    star(d, sx, sy, 3, WHITE)
                sun(d, cx + 34, cy - 46, 6, f, (255, 236, 200, 255))
                emc2(d, cx - 38, cy - 60, a(E_PALE, 200 + 55 * math.sin(ph)), 10)
                constellation(d, [(cx - 58, cy - 20), (cx - 46, cy - 32), (cx - 50, cy - 46), (cx - 36, cy - 52)], f)
                constellation(d, [(cx + 44, cy - 18), (cx + 56, cy - 4), (cx + 52, cy + 12)], f)
            elif podium == 2:
                einstein_ring(d, cx, cy - 18, 30, f, E_PALE, images=4)
                spacetime_grid(d, cx, feet[1] - 2, 50, 11, f, dent=6, color=E_BLUE, cells=8)
            elif podium == 3:
                for k in (-1, 1):
                    sx, sy = pt(cx, cy - 26, 14, ph + (k > 0) * math.pi, .5)
                    star(d, sx, sy, 3, WHITE if k > 0 else E_PALE)
                for k in range(4):
                    orbit(d, cx, cy - 26, 16 + k * 7, (16 + k * 7) * .5, a(E_VIOLET, 170 - k * 35 + 20 * math.sin(ph - k)))
                spacetime_grid(d, cx, feet[1] - 2, 48, 10, f, dent=5, color=E_BLUE, cells=8)
            else:
                # Spacetime-sheet cape with constellations.
                cape = [(cx - 12, cy - 12), (cx - 34, cy + 10), (cx - 26, cy + 26), (cx - 6, cy + 18)]
                d.polygon([(round(x), round(y)) for x, y in cape], fill=(26, 40, 96, 200), outline=E_BLUE)
                constellation(d, [(cx - 26, cy + 2), (cx - 20, cy + 14), (cx - 12, cy + 8)], f)
                einstein_ring(d, head[0], head[1] - 2, 18, f, E_PALE)
                spacetime_grid(d, cx, feet[1] - 2, 44, 9, f, dent=4, color=E_BLUE, cells=8)
        else:
            if rank >= 3:
                spacetime_grid(d, cx, feet[1] - 1, 20 + rank * 4, 5 + rank, f,
                               dent=4 if rank >= 6 else 0, color=E_BLUE, cells=6)
            if rank >= 6:
                star(d, cx + 2, feet[1] + 2, 3, (255, 236, 200, 255))
            if rank >= 5:
                einstein_ring(d, head[0], head[1] - 1, 13 + (rank - 5) * 3, f, E_PALE, images=0)
            if rank >= 4:
                melting_clock(d, cx + 22, cy - 22, 6, f)
            if rank >= 1:
                for k in range(min(3, rank)):
                    t = ph * .5 + k * TAU / 3
                    ex, ey = pt(head[0], head[1] - 4, 18, t, .35)
                    s = ['c', 'Δt', 'G'][k]
                    text(d, ex - 3, ey - 4, s, a(CHALK, 190 + 60 * math.sin(t)), 9)
            if rank >= 2:
                for k in range(5):
                    t = ph - k * .3
                    tx, ty = pt(hx - 2, hy, 9 + rank, t - 1.2, .6)
                    d.point((round(tx), round(ty)), fill=a(E_PALE, 230 - k * 40))
            pocket_watch(d, hx - 3, hy + 1, 3 + min(rank, 4) * .5, f)
            emc2(d, hx - 30, hy - 13 - (f % 4) * .5, a(CHALK, 170 + rank * 10), 9)
    elif science == 1:
        if top:
            if podium == 1:
                # Principia Universe: gold orrery, giant prism, the falling Moon.
                orrery(d, cx, cy - 4, 72, f, planets=4, flat=.4)
                rainbow_ring(d, cx, cy - 4, 58, f, .4, 1)
                prism(d, cx - 30, cy - 46, 11, f, rainbow=26, angle=-.2)
                mx, my = pt(cx, cy - 4, 46, ph * .6 + 1, .4)
                dot(d, mx, my, 3, (220, 224, 236, 255))
                line(d, [(mx, my), (mx, my + 4)], a(WHITE, 120))
                apple(d, cx + 30, cy - 52 + (f % 8) * 1.5, 3, f)
            elif podium == 2:
                orrery(d, cx, cy - 2, 54, f, planets=3, flat=.42)
                for k in range(4):
                    px, py = pt(cx, cy - 2, 38 + k * 4, ph * .7 + k * 1.6, .42)
                    page(d, px, py - 6, f, ph * .2 + k, 5)
            elif podium == 3:
                orrery(d, cx, cy - 2, 52, f, planets=3, flat=.42)
                comet(d, cx + 46 - f * 4, cy - 50 + f * 2, f)
            else:
                orrery(d, cx, cy - 2, 48, f, planets=3, flat=.42)
            if podium != 1:
                prism(d, hx - 2, hy - 2, 5, f, rainbow=12, angle=0)
        else:
            if rank >= 5:
                for k in range(2 if rank == 5 else 3):
                    rr = 22 + k * 9
                    orbit(d, cx, cy, rr, rr * .38, a(GOLD, 140))
                    px, py = pt(cx, cy, rr, ph * (1 - .2 * k) + k * 2, .38)
                    planet(d, px, py, 2 + k * .5, [(160, 196, 255, 255), (120, 220, 140, 255), (236, 170, 96, 255)][k], ring=(k == 2))
            elif rank >= 3:
                for k in range(2 if rank == 3 else 3):
                    px, py = pt(cx, cy - 4, 20, ph + k * TAU / 3, .4)
                    apple(d, px, py, 2.5, f)
                orbit(d, cx, cy - 4, 20, 8, a(GOLD, 90))
            if rank >= 4:
                for k, s in enumerate(['∫', 'dy', 'dx']):
                    t = ph * .4 + k * 2.1
                    gx, gy = pt(head[0], head[1] - 4, 20, t, .3)
                    text(d, gx - 3, gy - 4, s, a(GOLD, 200), 8 if s == '∫' else 7)
            if rank >= 6:
                telescope(d, cx + 22, cy + 8, f)
            if rank >= 2:
                prism(d, hx + 1, hy - 4, 4 + (rank >= 4), f, rainbow=8 + rank * 2, angle=math.pi)
            # The apple: held at Student, tossed and caught from Lab Assistant.
            toss = [0, 3, 6, 8, 8, 6, 3, 0][f] if rank >= 1 else 0
            apple(d, hx - 2, hy - 1 - toss, 3, f)
    else:
        if top:
            if podium == 1:
                # Stellar Forge: a sun-like nucleus forging elements.
                glow_pool(d, cx, feet[1], 46, f)
                nucleus_star(im, cx + 16, cy - 40, 12, f)
                d = ImageDraw.Draw(im)
                elems = [('H', 1), ('He', 2), ('C', 6), ('O', 8), ('Fe', 26), ('Po', 84), ('Ra', 88)]
                for k, (s, n) in enumerate(elems):
                    t = ph * .35 + k * .9
                    rr = 22 + k * 5
                    ex, ey = pt(cx + 16, cy - 40, rr, t, .55)
                    if -10 < ex < AURA[0] + 10:
                        element_tile(d, ex, ey, s, n)
                radium_cloud(im, cx, cy + 8, 46, f)
                d = ImageDraw.Draw(im)
                vial(d, hx + 2, hy, 4, f, 1.4)
            elif podium == 2:
                for k in (-1, 1):
                    sx, sy = pt(cx + 22, cy - 30, 10, ph * .7 + (k > 0) * math.pi, .6)
                    nucleus_star(im, sx, sy, 6, f + k)
                d = ImageDraw.Draw(im)
                glow_pool(ImageDraw.Draw(im), cx, feet[1], 36, f)
                radium_cloud(im, cx, cy + 8, 40, f)
                d = ImageDraw.Draw(im)
                vial(d, hx + 2, hy, 4, f, 1.2)
            elif podium == 3:
                for k in range(9):
                    t = ph * .3 + k * .7
                    rr = 8 + k * 4
                    ex, ey = pt(cx + 24, cy - 26, rr, t, .7)
                    dot(d, ex, ey, 2, (255, 120, 96, 255) if k % 2 else RADIUM)
                    if k:
                        px, py = pt(cx + 24, cy - 26, rr - 4, t - .7, .7)
                        line(d, [(px, py), (ex, ey)], a(CYAN, 150))
                glow_pool(ImageDraw.Draw(im), cx, feet[1], 36, f)
                radium_cloud(im, cx, cy + 8, 38, f)
                d = ImageDraw.Draw(im)
                vial(d, hx + 2, hy, 4, f, 1.2)
            else:
                nucleus_star(im, cx + 26, cy - 26, 9, f)
                d = ImageDraw.Draw(im)
                glow_pool(ImageDraw.Draw(im), cx, feet[1], 36, f)
                radium_cloud(im, cx, cy + 6, 34, f)
                d = ImageDraw.Draw(im)
                vial(d, hx + 2, hy, 4, f, 1.1)
        else:
            if rank >= 3:
                glow_pool(d, cx, feet[1], 12 + rank * 4, f)
            if rank >= 5:
                radium_cloud(im, cx, cy + 4, 18 + rank * 3, f)
                d = ImageDraw.Draw(im)
            if rank >= 6:
                for k in range(2):
                    t = ph * .6 + k * math.pi
                    mx, my = pt(cx, cy - 6, 24, t, .4)
                    nobel_medal(d, mx, my, 4, f)
            if rank >= 4:
                ray_fan(d, hx + 6, hy - 2, f, 18 + rank * 2)
            if rank >= 3:
                for k, (s, n) in enumerate([('Ra', 88), ('Po', 84)]):
                    ex, ey = pt(head[0] - 2, head[1] - 4, 20, ph * .25 + k * math.pi + .6, .3)
                    element_tile(d, ex, ey, s, n)
            if rank >= 2:
                bohr_atom(d, hx + 1, hy - 1, 8 + rank, f, RADIUM, shells=2 if rank < 4 else 3)
            vial(d, hx + 1, hy, 3 + min(rank, 3) * .4, f, .6 + rank * .12)
            if rank >= 1:
                # The vial's glow brightens with each rank.
                for k in range(6):
                    t = ph * .5 + k * TAU / 6
                    gx, gy = pt(hx + 1, hy, 6 + rank, t, .8)
                    d.point((round(gx), round(gy)), fill=a(mix(RADIUM, WHITE, .4), 150 + 15 * rank))
    if top and podium == 1:
        # Three minds in one body: the other two stand at the shoulders.
        others = [s for s in range(3) if s != science]
        for k, s in enumerate(others):
            g = ghost(s, f, 104 + round(24 * math.sin(ph + k * math.pi)))
            im.alpha_composite(g, (BODY[0] + (-20 if k == 0 else 20), BODY[1] - 6))
    # A moving measurement keeps every phase distinct after palette packing.
    ImageDraw.Draw(im).point((round(cx - 20 + f), round(feet[1] + 6)), fill=a(ACCENT[science], 220))
    return glow(im, 1.6 if top else 1.1, 1.8 if top else 1.4)


def equipment(science, rank, f, front=False, podium=4):
    if front:
        return Image.new('RGBA', AURA)
    return aura(science, rank, f, podium)


# ------------------------------------------------------------ activations --

SIZE = [7, 9, 12, 15]          # travelling object radius per art tier
CONSTRUCT = [32, 40, 52, 64]    # held construction size per art tier


def _shrink(im, size):
    """Downscale a 128 px construction without losing thin lines."""
    box = im.getbbox()
    if not box:
        return Image.new('RGBA', (size, size))
    return im.resize((size, size), Image.Resampling.BOX)


def iconic_object(d, im, skill, tier, f, x, y, r):
    """Headline skills drawn as what the scientist is famous for. Returns
    True when it replaced the generic construction."""
    ph = f * TAU / 8
    if skill == 0:        # Photon Pulse: an E=mc² bolt of light
        light_beam(d, x - r * 1.4, y, x + r, y, E_PALE, 2 + tier // 2)
        for k in (-1, 1):
            line(d, [(x - r * .2, y), (x + r * .9, y + k * r * .5)], a(E_VIOLET, 220), 1)
        dot(d, x + r * .8, y, r * .35, WHITE)
        if r >= 10:
            emc2(d, x - r * 1.2, y - r * .9, a(CHALK, 230), 8)
        return True
    if skill == 4:        # Proper Time: his own pocket watch, ticking fast
        pocket_watch(d, x, y, r * .7, f * 2)
        for k in range(4):
            tx, ty = pt(x, y, r * 1.1, ph - k * .35, .8)
            d.point((round(tx), round(ty)), fill=a(E_PALE, 230 - k * 50))
        return True
    if skill == 5:        # Time Dilation: the melting clock
        melting_clock(d, x, y, r * .9, f)
        return True
    if skill == 13:       # Gravitational Lens: an Einstein ring
        einstein_ring(d, x, y, r, f, E_PALE, images=4 if tier >= 2 else 0)
        return True
    if skill == 24:       # Unified Frame: three frames, one grid
        spacetime_grid(d, x, y + r * .5, r * 1.3, r * .4, f, dent=r * .2, color=E_BLUE, cells=6)
        einstein_ring(d, x, y - r * .2, r * .7, f, E_PALE)
        return True
    if skill in (34, 40):  # Momentum Strike / Parabolic Shot: Newton's apple
        apple(d, x, y, r * (.85 if skill == 34 else .6), f)
        if skill == 34:
            for k in range(3):
                line(d, [(x - r * (1.2 + k * .3), y - r * .5 + k * r * .5), (x - r * .9, y - r * .4 + k * r * .4)], a(GOLD, 200))
        return True
    if skill == 39:       # Tangential Release: a planet flung from orbit
        orbit(d, x - r * .8, y, r * .9, r * .45, a(GOLD, 160), 1, -1.2, 1.2)
        planet(d, x, y, r * .5, (120, 200, 255, 255), ring=tier >= 2)
        return True
    if skill == 49:       # Principia: an orrery slam
        orrery(d, x, y, r * 1.2, f, planets=3 + (tier >= 2))
        return True
    if skill == 27:       # Scalar Division: the prism splits the light
        prism(d, x, y, r * .6, f, rainbow=r * 1.4, angle=0)
        return True
    if skill == 73:       # Gamma Beam
        line(d, [(x - r * 1.6, y), (x + r * 1.4, y)], a(RADIUM, 120), 5 + tier)
        line(d, [(x - r * 1.6, y), (x + r * 1.4, y)], RADIUM, 2 + tier // 2)
        line(d, [(x - r * 1.6, y), (x + r * 1.4, y)], WHITE, 1)
        return True
    if skill == 74:       # Decay Chain: three glowing blasts
        for k in range(3):
            q = r * (.3 + .1 * ((k + f) % 3))
            cx_, cy_ = x - r * .8 + k * r * .8, y + (k % 2) * r * .3
            dot(d, cx_, cy_, q, a(RADIUM_DEEP, 200), RADIUM)
            dot(d, cx_, cy_, max(1, q * .4), WHITE)
        return True
    if skill == 63:       # Radioactive Burst: radium in its vial
        vial(d, x, y, r * .5, f, 1.6)
        return True
    if skill == 71:       # Alpha: heavy, short
        dot(d, x, y, r * .45, (255, 120, 96, 255), WHITE)
        dot(d, x - r * .25, y - r * .1, r * .2, (255, 200, 180, 255))
        return True
    if skill == 72:       # Beta: curved electrons
        for k in range(3):
            pts = [(x - r + j * r / 6, y + (k - 1) * r * .35 + (j / 12) ** 2 * r * .6) for j in range(13)]
            line(d, pts, a(CYAN, 230))
        return True
    return False


def cast(skill, tier, f, skills):
    """Gesture, then release: chalk writes E=mc², an apple is tossed, a
    radium vial is raised. 36 ticks, behind the body."""
    im = Image.new('RGBA', (CAST, CAST))
    d = ImageDraw.Draw(im)
    science = skills[skill]['science']
    role = skills[skill]['role']
    c = ACCENT[science]
    # The cast pose extends the right wrist to body (29-33, 39-41); the body
    # origin on this canvas is (64, 56). Props decorate that release hand.
    hx, hy = 95, 96 + [-1, 1, 0][science]
    feet_y = CAST / 2 + 29
    ph = f * TAU / 8
    # Ground ring from the construction tier, Isliid-style.
    if tier >= 2:
        rr = 14 + tier * 5 + [0, 2, 4, 6, 6, 4, 2, 0][f]
        orbit(d, CAST / 2, feet_y, rr, rr * .3, a(c, 150 + 30 * (tier - 2)), 1)
        orbit(d, CAST / 2, feet_y, rr * .7, rr * .21, a(mix(c, WHITE, .4), 120), 1)
    # The gesture (frames 0-3).
    if science == 0 and skill != 0:
        shown = 'E=mc²'[:min(5, f + 2)]
        text(d, hx - 10, hy - 22 - tier * 2, shown, a(CHALK, 240 - max(0, f - 4) * 40), 9)
        line(d, [(hx, hy), (hx - 6 + f * 2, hy - 14)], a(CHALK, 200 - f * 20), 1)
    elif science == 1:
        toss = [0, 6, 11, 14, 14, 11, 6, 0][f]
        apple(d, hx - 3, hy - 4 - toss, 3 + tier * .5, f)
        if f >= 3:
            prism(d, hx + 4, hy - 6, 4 + tier, f, rainbow=10 + tier * 5, angle=0)
    else:
        lift = [0, 3, 6, 8, 8, 8, 6, 3][f]
        vial(d, hx + 2, hy - lift, 4 + tier * .5, f, .9 + tier * .3)
        for k in range(6 + tier * 2):
            t = k * TAU / (6 + tier * 2) + ph * .3
            gx, gy = pt(hx + 2, hy - lift - 1, 8 + tier * 3, t, .8)
            d.point((round(gx), round(gy)), fill=a(mix(RADIUM, WHITE, .3), 210))
    # The scientific object (frames 3-7) or the held construction.
    r = SIZE[tier]
    # The packet sprite carries the flight; the activation only launches it.
    travel = [0, 0, 0, 4, 8, 12, 14, 15][f]
    grow = [.35, .6, .85, 1, 1, .85, .6, .3][f]
    x, y = hx + 3 + travel, hy - 4 - [0, 1, 2, 3, 4, 2, 1, 0][f]
    if role in ('Attack', 'Heavy', 'Field', 'Setup') or skill in (13, 45, 57):
        q = max(2, r * grow)
        if not iconic_object(d, im, skill, tier, f, x, y, q):
            cosmic.delivery_object(im, skill, tier, f, x, y, q)
        d = ImageDraw.Draw(im)
        if f >= 3:
            line(d, [(hx + 4, hy - 2), (x - q * .8, y)], a(c, 150), 1)
    else:
        core = Image.new('RGBA', (128, 128))
        cosmic.experiment(core, skill, min(tier + 1, 3), f, science)
        size = round(CONSTRUCT[tier] * [.5, .7, .9, 1, 1, .9, .75, .55][f])
        piece = _shrink(core, size)
        im.alpha_composite(piece, (round(hx + 4 - size / 2), round(hy - 6 - size / 2)))
        d = ImageDraw.Draw(im)
        iconic_object(d, im, skill, tier, f, hx + 4, hy - 6, CONSTRUCT[tier] * .28)
        if role == 'Protect':
            d.arc((round(hx), round(hy - 26 - tier * 3), round(hx + 30 + tier * 4), round(hy + 14 + tier * 3)), -70, 70, fill=a(c, 230), width=2)
        elif role == 'Move':
            for k in range(3):
                line(d, [(CAST / 2 - 18 - k * 5, feet_y - 4 - k * 4), (CAST / 2 - 8, feet_y - 4 - k * 4)], a(c, 180 - k * 40))
    if tier == 3 and f in (3, 4):
        for k in range(10):
            t = k * TAU / 10 + f * .2
            line(d, [pt(hx + 8, hy - 4, 8, t), pt(hx + 8, hy - 4, 18 + f * 2, t)], a(mix(c, WHITE, .5), 200))
    d.point((round(hx - 4 + f), round(hy + 6)), fill=c)
    return glow(im, 1.0, 1.6)


def packet(skill, tier, f, heading, skills):
    science = skills[skill]['science']
    r = [5, 7, 9, 11][tier]
    im = Image.new('RGBA', (PACKET, PACKET))
    d = ImageDraw.Draw(im)
    c = ACCENT[science]
    x = y = PACKET // 2
    # A longer, brighter tail per tier: the trajectory is the spectacle.
    tail = 9 + tier * 3
    for k in range(tail):
        tx = x - r * .6 - k
        ty = y + math.sin(k * .45 + f * TAU / 8) * (1 + tier * .4)
        col = SPECTRUM[(k // 2) % 7] + (220 - k * 180 // tail,) if skill in (27, 40) else a(c, 220 - k * 180 // tail)
        d.point((round(tx), round(ty)), fill=col)
        if tier >= 2 and k % 4 == f % 4:
            d.point((round(tx), round(ty) - 2), fill=a(WHITE, 160 - k * 4))
    if not iconic_object(d, im, skill, tier, f, x, y, r):
        cosmic.delivery_object(im, skill, tier, f, x, y, r)
    d = ImageDraw.Draw(im)
    if tier >= 1:
        orbit(d, x, y, r * 1.2, r * 1.2, a(c, 60 + 20 * tier), 1, f * .6, f * .6 + 2.2)
    d.point((x - 14 + f, y + r + 2), fill=c)
    return glow(im, .8, 1.7).rotate(-heading * 360 / 8, resample=Image.Resampling.NEAREST)


def impact(skill, tier, f, skills):
    im = Image.new('RGBA', (IMPACT, IMPACT))
    science = skills[skill]['science']
    x = y = IMPACT // 2
    life = [.25, .5, .85, 1, .9, .65, .4, .15][f]
    r = (8 + tier * 4) * life
    # Keep each recipe's own contact construction at the centre.
    im.alpha_composite(cosmic.impact(skill, tier, f, skills), (x - 32, y - 32))
    d = ImageDraw.Draw(im)
    c = ACCENT[science]
    if science == 0:
        dot(d, x, y, max(1, r * .35), a(WHITE, 255 * life))
        einstein_ring(d, x, y, r, f, a(E_PALE, 200), images=0, flat=.8)
        for k in range(6):
            t = k * TAU / 6 + f * .1
            line(d, [pt(x, y, r * .5, t, .8), pt(x, y, r * 1.3, t, .8)], a(E_VIOLET, 200 * life))
    elif science == 1:
        orbit(d, x, y, r, r * .55, a(GOLD, 240 * life), 2)
        for k, col in enumerate(SPECTRUM):
            t = k * TAU / 7 + f * .15
            line(d, [pt(x, y, r * .6, t, .6), pt(x, y, r * 1.2, t, .6)], col + (round(220 * life),))
        if skill in (34, 40):
            for k in range(5):
                px, py = pt(x, y, r * .9, k * TAU / 5 + .3, .6)
                dot(d, px, py, 1.5, APPLE)
    else:
        dot(d, x, y, max(1, r * .55), a(RADIUM, 120 * life))
        for k in range(8):
            t = k * TAU / 8 + f * .2
            px, py = pt(x, y, r * 1.1, t, .7)
            sparkle(d, px, py, a(mix(RADIUM, WHITE, .4), 230 * life), 1)
        orbit(d, x, y, r * .8, r * .56, a(CYAN, 200 * life), 1)
    d.point((x - 6 + f, y + 5), fill=c)
    return glow(im, .9 + tier * .2, 1.6)


# ------------------------------------------------------------------ fields --

BOUNDED_FIELDS = (5, 12, 13, 18, 20, 45, 58, 65, 66)


def field(kind, tier, f, size=FIELD):
    """Iconic overlays inside the gameplay boundary. Fields with a fixed
    domain ring (drawn by mastery_art at ~950 world units per pixel) keep every
    overlay within that radius so the drawn area matches the real area."""
    im = cosmic.field(kind, tier, f, size)
    d = ImageDraw.Draw(im)
    x = y = size // 2
    bound = 19 if kind == 13 else 32
    r = min([24, 32, 42, 54][tier], bound - 4) if kind in BOUNDED_FIELDS else [24, 32, 42, 54][tier]
    ph = f * TAU / 8
    science = 0 if kind < 25 else (1 if kind < 50 else 2)
    c = ACCENT[science]
    if kind == 5:
        spacetime_grid(d, x, y + r * .35, r * 1.1, r * .35, f, dent=0, color=E_BLUE, cells=8, alpha=130)
        melting_clock(d, x, y - r * .25, r * .45, f)
    elif kind == 1:
        line(d, [(x - r * .7, y), (x + r * .7, y)], a(CHALK, 200))
        line(d, [(x, y - r * .7), (x, y + r * .4)], a(CHALK, 200))
        text(d, x + 4, y - r * .7, 't', CHALK, 9)
        text(d, x + r * .55, y + 2, 'x', CHALK, 9)
    elif kind in (12, 20):
        spacetime_grid(d, x, y + 4, r * 1.3, r * .5, f, dent=r * .35, color=E_VIOLET, cells=8, alpha=140)
    elif kind == 13:
        einstein_ring(d, x, y, r * .85, f, E_PALE, images=4)
    elif kind == 18:
        for k in range(3):
            pts = [pt(x, y, r * (.2 + j * .03), ph * .4 + k * TAU / 3 + j * .25, .55) for j in range(28)]
            line(d, pts, a(E_BLUE, 200), 1)
    elif kind == 45:
        text(d, x - r * .9, y - r * .8, 'f(x)=0', a(GOLD, 220), 9)
    elif kind in (57, 58):
        glow_pool(d, x, y + r * .45, r, f)
    elif kind in (65, 66):
        radium_cloud(im, x, y, r * 1.1, f)
        d = ImageDraw.Draw(im)
        bohr_atom(d, x, y, r * .35, f, RADIUM, shells=2)
    if kind not in BOUNDED_FIELDS:
        rr = r * (1.05 + .05 * math.sin(ph))
        orbit(d, x, y + r * .3, rr, rr * .3, a(c, 130 + tier * 25), 1)
    d.point((x - 6 + f, y + 4), fill=c)
    return glow(im, 1.2 + tier * .3, 1.5)


def mark(kind, tier, f):
    """Visible preparation: a golden vector, a forecast crosshair, a glowing
    sample or an isotope label on the target."""
    im = Image.new('RGBA', (MARK, MARK))
    d = ImageDraw.Draw(im)
    x = y = MARK // 2
    s = 1 + tier * .25
    pulse = .7 + .3 * math.sin(f * TAU / 8)
    if kind == 26:
        orbit(d, x, y + 6, 16 * s, 5 * s, a(GOLD, 160))
        line(d, [(x - 14 * s, y + 10), (x + 14 * s, y - 4)], a(GOLD, 255 * pulse), 2)
        line(d, [(x + 14 * s, y - 4), (x + 8 * s, y - 5)], GOLD, 2)
        line(d, [(x + 14 * s, y - 4), (x + 10 * s, y + 1)], GOLD, 2)
        text(d, x - 4, y - 14, 'F', GOLD, 9)
    elif kind == 42:
        orbit(d, x, y, 12 * s, 12 * s, a(E_BLUE, 220 * pulse), 1)
        for k in range(4):
            t = k * TAU / 4
            line(d, [pt(x, y, 6 * s, t), pt(x, y, 16 * s, t)], E_PALE)
        for k in range(5):
            if (k + f) % 2 == 0:
                d.point((round(x - 26 + k * 4), round(y + 2 - k)), fill=GOLD)
    elif kind == 50:
        orbit(d, x, y + 16, 12 * s, 4 * s, a(RADIUM, 200 * pulse))
        vial(d, x, y - 20, 3 * s, f, 1.4)
    else:
        orbit(d, x, y + 16, 12 * s, 4 * s, a(RADIUM, 200 * pulse))
        element_tile(d, x, y - 20, 'Ra', 88)
    d.point((x - 8 + f, y + 22), fill=WHITE)
    return glow(im, 1.2, 1.6)


# ------------------------------------------------- completion and resonance --

def completion(tier, podium, f, science=0):
    im = Image.new('RGBA', (COMPLETE, COMPLETE))
    d = ImageDraw.Draw(im)
    x, y = COMPLETE // 2, COMPLETE // 2 - 6
    life = [.35, .62, .85, 1, 1, .85, .6, .3][f]
    r = [22, 30, 40, 50][tier] * life
    peak = tier == 3 and podium == 1
    ph = f * TAU / 8
    if science == 0:
        spacetime_grid(d, x, y + 34, r * 1.2, r * .3, f, dent=r * .25, color=E_BLUE, cells=8, alpha=150)
        einstein_ring(d, x, y - 8, r * .8, f, E_PALE, images=4 if tier >= 2 else 0)
        for k in range(8):
            t = k * TAU / 8 + ph * .1
            line(d, [pt(x, y - 8, r * .3, t), pt(x, y - 8, r * .95, t)], a(E_VIOLET, 180 * life))
        if tier >= 1:
            emc2(d, x - 20, y - r - 6, a(CHALK, 255 * life), 11 if tier >= 2 else 9)
        if peak:
            sun(d, x + r * .55, y - r * .6, 6, f, (255, 236, 200, 255))
            constellation(d, [(x - r, y - 10), (x - r * .7, y - r * .6), (x - r * .3, y - r * .85)], f)
    elif science == 1:
        prism(d, x - r * .4, y - 6, 6 + tier * 2, f, rainbow=r * 1.3, angle=-.15)
        orbit(d, x, y, r, r * .4, a(GOLD, 220 * life), 2)
        for k in range(3 + tier):
            px, py = pt(x, y, r, ph * .8 + k * TAU / (3 + tier), .4)
            apple(d, px, py, 2 + tier * .5, f)
        if peak:
            orrery(d, x, y, r * 1.2, f, planets=4)
            comet(d, x + r - f * 5, y - r * .8, f)
    else:
        nucleus_star(im, x, y - 6, max(3, r * .28), f)
        d = ImageDraw.Draw(im)
        bohr_atom(d, x, y - 6, r * .8, f, RADIUM, shells=3)
        glow_pool(d, x, y + 34, r, f)
        if tier >= 2:
            for k in range(2):
                mx, my = pt(x, y - 6, r * .95, ph * .5 + k * math.pi, .4)
                nobel_medal(d, mx, my, 4 + tier, f)
        if peak:
            for k, (s, n) in enumerate([('Ra', 88), ('Po', 84), ('He', 2)]):
                ex, ey = pt(x, y - 6, r * 1.15, ph * .3 + k * TAU / 3, .55)
                element_tile(d, ex, ey, s, n)
    if tier == 3 and podium <= 3:
        for k in range(4 - podium):
            sparkle(d, x - 8 + k * 8, y + 52, WHITE, 1 + (f + k) % 2)
    d.point((x - 30 + f, y + 60), fill=ACCENT[science])
    return glow(im, 1.6 + tier * .3, 1.7)


def unified(tier, podium, f):
    """Einstein, Newton and Curie stand together for one combined strike."""
    im = Image.new('RGBA', (COMPLETE, COMPLETE))
    x, y = COMPLETE // 2, COMPLETE // 2
    appear = [.35, .7, 1, 1, 1, .9, .6, .3][f]
    for k, s in enumerate((0, 2, 1)):
        g = ghost(s, f, round(170 * appear))
        im.alpha_composite(g, (x - 24 + [-26, 26, 0][k], y - 32 + [0, 0, -10][k]))
    d = ImageDraw.Draw(im)
    # Three beams converge ahead of the body, then one burst.
    focus = (x + 52, y - 4)
    hands = [(x - 26 + 13 - 24 + 24, y + 8), (x + 26 + 9, y + 8), (x + 9, y - 2)]
    reach = [0, .3, .6, 1, 1, 1, 1, 1][f]
    for (hx, hy), c in zip(hands, (E_BLUE, RADIUM, GOLD)):
        ex = hx + (focus[0] - hx) * reach
        ey = hy + (focus[1] - hy) * reach
        light_beam(d, hx, hy, ex, ey, c, 1 + tier // 2)
    if f >= 3:
        burst = [0, 0, 0, .5, 1, .85, .6, .3][f] * (14 + tier * 8) * (1.3 if podium == 1 else 1)
        for k in range(12):
            t = k * TAU / 12 + f * .15
            col = (E_BLUE, GOLD, RADIUM)[k % 3]
            line(d, [pt(*focus, burst * .3, t), pt(*focus, burst, t)], col, 2)
        dot(d, *focus, burst * .3, WHITE)
        if tier >= 2:
            einstein_ring(d, *focus, burst * .8, f, E_PALE)
            orbit(d, *focus, burst * 1.05, burst * .45, a(GOLD, 200), 1)
            bohr_atom(d, *focus, burst * .55, f, RADIUM, shells=2)
        if podium == 1:
            emc2(d, focus[0] - 16, focus[1] - burst - 14, CHALK, 10)
    if tier == 3 and podium <= 3:
        for k in range(4 - podium):
            sparkle(d, x - 8 + k * 8, y + 44, WHITE, 1 + (f + k) % 2)
    d.point((x - 30 + f, y + 50), fill=WHITE)
    return glow(im, 1.6 + tier * .3, 1.8)


def accident(science, f):
    """Lab accidents: chalk dust and a cracked watch; the apple lands on his
    head; the vial boils over in green smoke."""
    im = Image.new('RGBA', (ACCIDENT, ACCIDENT))
    d = ImageDraw.Draw(im)
    x, y = ACCIDENT // 2, ACCIDENT // 2
    head = (x + 1, y - 9)
    if science == 0:
        for k in range(10):
            t = k * TAU / 10 + k * .3
            rr = 4 + f * 3 + (k % 3) * 2
            px, py = pt(head[0], head[1] + 6, rr, t, .8)
            dot(d, px, py, 1.5 + (k % 2) + f * .25, a(CHALK, 210 - f * 22))
        wx, wy = x + 10 + f * 3, y + 2 - [0, 4, 7, 8, 8, 7, 4, 0][f]
        pocket_watch(d, wx, wy, 4, f)
        line(d, [(wx - 2, wy - 3), (wx + 1, wy), (wx - 1, wy + 3)], (255, 96, 96, 255))
    elif science == 1:
        if f <= 3:
            ay = 6 + [0, 9, 19, 28][f]
            apple(d, head[0], ay, 4, f)
            line(d, [(head[0], ay - 12), (head[0], ay - 6)], a(WHITE, 140))
        else:
            k = f - 4
            apple(d, head[0] + 6 + k * 5, head[1] - 10 - [6, 8, 6, 2][k], 4, f)
            for j in range(3):
                t = j * TAU / 3 + f * .7
                sx, sy = pt(head[0], head[1] - 7, 9, t, .35)
                sparkle(d, sx, sy, GOLD, 2)
        if f == 3:
            for j in range(6):
                t = j * TAU / 6
                line(d, [pt(head[0], head[1] - 6, 4, t), pt(head[0], head[1] - 6, 9, t)], WHITE)
    else:
        vx, vy = x + 10, y + 3
        vial(d, vx, vy, 4, f, 1.6)
        for k in range(7):
            rise = (f * 4 + k * 5) % 30
            px = vx + math.sin(k * 1.7 + f) * (3 + rise * .3)
            dot(d, px, vy - 6 - rise, 1.5 + rise * .12, a(mix(RADIUM, (60, 80, 70), .3), 200 - rise * 5))
        if f >= 2:
            for k in range(4):
                dot(d, vx - 4 + k * 3, vy + 6 + (f - 2) * 2, 1, RADIUM)
    if f in (2, 3):
        d.ellipse((head[0] - 12, head[1] - 12, head[0] + 12, head[1] + 12), outline=(255, 110, 96, 200))
    d.point((x - 20 + f, y + 30), fill=ACCENT[science])
    return glow(im, 1.2, 1.5)
