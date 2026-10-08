"""Round 101: the Coder's effects, rig HUD and rank crests: sheet 'coder_vfx' (at most 2048 x 2048).

  fx_send          the packet leaving his hand (on him)
  fx_ping          a packet landing: 0s and 1s burst off the target
  fx_heal          green pluses rising off an ally
  fx_shield        the firewall shell snapping on (the lasting shell is the cd_shield buff)
  fx_scan          a radar sweep around him
  fx_spray         hex glyphs blasting out around him
  fx_blink_out/in  glitching out and back in (scanlines)
  fx_cache         a recycling spinner (he idles to clear his CPU)
  fx_chain         a square-wave zap on each enemy the chain hits
  fx_wall_<a>      a piece of burning hex wall, a = 0..7 (22.5 degree steps over 180)
  buffs            cd_shield (hex dome), cd_lag (a buffering spinner over the slowed), cd_oc (overclock heat),
                   cd_heat0..10 (the thermometer), cd_ram0..8 (the RAM bar), cd_disk0..8 (saved functions),
                   cd_rank0..6 and cd_root1..10 (the crests: Script Kiddie .. Architect, Root #1-#10)

Run from the repo root: python3 "Claude outputs/coder/coder_vfx.py" [--preview]
"""
import json
import math
import os
import random
import sys

from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, '..', 'levi'))
sys.path.insert(0, HERE)
from levi_vfx import Cv, rgba, hsv, glow, star4, crest_shape, glint  # noqa: E402
import coder_font as F  # noqa: E402

ROOT = os.path.abspath(os.path.join(HERE, '..', '..'))
OUT = os.path.join(ROOT, 'mods', 'tfm2_custom', 'vfx')

# round 106 (Rian: "I don't really like all the green stuff"): his effects are software UI in one neutral palette (sky
# blue, white, a dark panel, amber / red alerts); the old names stay so every effect switched over at once.
UI_B, UI_W, UI_D = rgba('#6eb9ff'), rgba('#eef4ff'), rgba('#2a5fa8')
UI_PANEL, UI_EDGE = rgba('#1a2030'), rgba('#c8dcff')
GREEN, GREEN_L, GREEN_D = UI_B, UI_W, UI_D
CYAN = rgba('#a8dcff')
AMBER = rgba('#ffbe50')


def popup(cv, x, y, w, h, bar, a=255):
    """A tiny window: a dark body, a coloured title bar, a light frame."""
    for yy in range(h):
        for xx in range(w):
            edge = xx in (0, w - 1) or yy in (0, h - 1)
            c = UI_EDGE if edge else bar if yy < 3 else UI_PANEL
            cv.add(x + xx, y + yy, c[:3] + (int(a * (1.0 if edge or yy < 3 else 0.92)),))


CURSOR = ['#', '##', '#.#', '#..#', '#...#', '#..##', '##.#', '#..#']


def cursor(cv, x, y, a=255, col=None):
    """A mouse pointer with its tip at (x, y)."""
    for yy, row in enumerate(CURSOR):
        for xx, ch in enumerate(row):
            if ch == '#':
                cv.add(x + xx, y + yy, (20, 24, 34, int(a)))
            elif ch == '.':
                cv.add(x + xx, y + yy, (col or UI_W)[:3] + (int(a),))
HEX = '0123456789ABCDEF'


def glyph(cv, ch, x, y, col, a=255):
    """One font character stamped at (x, y) (its top-left)."""
    rows = F.glyphs().get(ch, F.glyphs()['?'])
    for yy, row in enumerate(rows):
        for xx, bit in enumerate(row):
            if bit == '#':
                cv.add(x + xx, y + yy, col[:3] + (int(a),))


# ------------------------------------------------------------------ effects

def send(f):
    """His hand sends a request: a click ripple and a small cursor."""
    cv = Cv(32, 32)
    life = 1 - f / 4
    cv.ring(18, 16, 2 + f * 2.5, UI_B[:3] + (int(230 * life),), 1.0)
    cursor(cv, 17 + f, 13 - f, 255 * life)
    return cv.im


def bit(f):
    """His basic attack in flight: a pointer with a light trail."""
    cv = Cv(16, 16)
    glow(cv, 7, 8, 6, UI_B, 120)
    for k in range(3):
        cv.add(3 - k + (f % 2), 9 + k, UI_W[:3] + (150 - k * 40,))
    cursor(cv, 6, 4)
    return cv.im


def ping(f):
    """A packet lands: a click, a double ripple and a little 'x' error box popping off the target."""
    cv = Cv(40, 40)
    life = 1 - f / 6
    if f <= 1:
        glow(cv, 20, 20, 8, UI_W, 220)
    cv.ring(20, 20, 3 + f * 3, UI_B[:3] + (int(240 * life),), 1.2)
    cv.ring(20, 20, 1 + f * 2, UI_W[:3] + (int(160 * life),), 1.0)
    if f >= 1:
        bx, by = 22 + f * 2, 10 - f
        popup(cv, bx, by, 9, 7, RED, 255 * life)
        cv.add(bx + 3, by + 4, (255, 255, 255, int(255 * life)))
        cv.add(bx + 5, by + 4, (255, 255, 255, int(255 * life)))
        cv.add(bx + 4, by + 5, (255, 255, 255, int(255 * life)))
    return cv.im


def heal(f):
    """A heal toast: a little window with a plus, a health bar filling, light rising."""
    cv = Cv(40, 48)
    life = 1 - max(0, f - 3) / 3
    popup(cv, 8, 26 - f * 2, 24, 11, rgba('#5fd6a0'), 245 * life)
    for d in range(-2, 3):
        cv.add(13 + d, 32 - f * 2, (255, 255, 255, int(255 * life)))
        cv.add(13, 32 - f * 2 + d, (255, 255, 255, int(255 * life)))
    fill = min(9, 2 + f * 2)
    for x in range(fill):
        cv.add(18 + x, 32 - f * 2, rgba('#7ef0b8')[:3] + (int(255 * life),))
        cv.add(18 + x, 33 - f * 2, rgba('#5fd6a0')[:3] + (int(255 * life),))
    for k in range(3):
        star4(cv, 10 + k * 10, 22 - f * 3 - (k % 2) * 3, UI_W[:3] + (int(230 * life),))
    return cv.im


def hexdome(cv, cx, cy, rx, ry, f, a=200):
    """A dome of hexagon cells (the firewall shell)."""
    for y in range(int(cy - ry), int(cy + ry) + 1):
        for x in range(int(cx - rx), int(cx + rx) + 1):
            d = math.hypot((x - cx) / rx, (y - cy) / ry)
            if d > 1:
                continue
            # a hex lattice: cells 4 px wide, edges where the coordinates line up
            u = (x + (y // 3) % 2 * 2 + f) % 4
            v = y % 3
            edge = u == 0 or v == 0
            if d > 0.86 or edge:
                k = 1.0 if d > 0.86 else 0.45
                cv.add(x, y, CYAN[:3] + (int(a * k * (0.6 + 0.4 * d)),))


def shield_on(f):
    cv = Cv(56, 56)
    life = 1 - f / 6
    hexdome(cv, 28, 30, 14 + f, 17 + f, f, int(240 * life))
    if f == 0:
        glow(cv, 28, 30, 14, CYAN, 200)
    return cv.im


def shield_buff(f):
    cv = Cv(48, 56)
    hexdome(cv, 24, 30, 15, 19, f, 120 + 40 * (f % 2))
    return cv.im


def scan(f):
    """A scan: a loading ring sweeping out round him, tick marks like a progress dial."""
    cv = Cv(128, 128)
    life = 1 - f / 6
    r = 10 + f * 9
    cv.ring(64, 64, r, UI_B[:3] + (int(220 * life),), 1.5)
    for k in range(24):
        a = k / 24 * math.tau
        on = k <= f * 4
        cv.add(64 + math.cos(a) * (r + 4), 64 + math.sin(a) * (r + 4), (UI_W if on else UI_D)[:3] + (int(230 * life),))
    a0 = f * 1.1
    for k in range(int(r)):
        for da in range(5):
            a = a0 - da * 0.06
            cv.add(64 + math.cos(a) * k, 64 + math.sin(a) * k, CYAN[:3] + (int(120 * life * (1 - da / 5)),))
    return cv.im


def spray(f):
    """Pop-up spam: little windows bursting out round him."""
    cv = Cv(96, 96)
    life = 1 - f / 6
    rnd = random.Random(31)
    if f <= 1:
        glow(cv, 48, 48, 14, UI_W, 200)
    for k in range(10):
        a = k * math.tau / 10 + rnd.uniform(-0.15, 0.15)
        d = 6 + f * rnd.uniform(5.5, 7.5)
        popup(cv, 48 + math.cos(a) * d - 4, 48 + math.sin(a) * d - 3, 8, 6, rnd.choice((UI_B, AMBER, RED)), 245 * life)
    cv.ring(48, 48, 8 + f * 6.5, UI_B[:3] + (int(150 * life),), 1.0)
    return cv.im


def blink(f, out):
    """A window minimising (out) or restoring (in) where he was: a frame shrinking to a bar, scanlines."""
    cv = Cv(40, 56)
    k = f / 5 if out else 1 - f / 5
    w, h = int(30 * (1 - k) + 6), int(40 * (1 - k) + 3)
    x0, y0 = 20 - w // 2, 30 - h // 2 + int(k * 14)
    a = 230 * (1 - k * 0.5)
    for x in range(w):
        cv.add(x0 + x, y0, UI_EDGE[:3] + (int(a),))
        cv.add(x0 + x, y0 + h, UI_EDGE[:3] + (int(a),))
    for y in range(h):
        cv.add(x0, y0 + y, UI_EDGE[:3] + (int(a),))
        cv.add(x0 + w - 1, y0 + y, UI_EDGE[:3] + (int(a),))
        if y % 3 == 0:
            for x in range(1, w - 1):
                cv.add(x0 + x, y0 + y, UI_B[:3] + (int(a * 0.4),))
    for x in range(1, w - 1):
        cv.add(x0 + x, y0 + 1, UI_B[:3] + (int(a),))
    return cv.im


def cache(f):
    cv = Cv(48, 48)
    for k in range(3):
        a0 = f * math.tau / 8 + k * math.tau / 3
        for j in range(14):
            a = a0 + j * 0.08
            cv.add(24 + math.cos(a) * 15, 26 + math.sin(a) * 15, GREEN[:3] + (int(240 * (1 - j / 16)),))
        tip = a0 + 14 * 0.08
        for s in (-1, 1):
            cv.add(24 + math.cos(tip) * 15 + math.cos(tip + s * 2.3) * 2, 26 + math.sin(tip) * 15 + math.sin(tip + s * 2.3) * 2, GREEN_L)
    return cv.im


def chain(f):
    cv = Cv(40, 40)
    life = 1 - f / 5
    rnd = random.Random(5 + f)
    # a square-wave zap through the target
    x, y = 4, 20
    while x < 36:
        ny = 20 + rnd.choice((-6, -3, 3, 6))
        cv.line(x, y, x, ny, CYAN[:3] + (int(255 * life),))
        cv.line(x, ny, x + 4, ny, (255, 255, 255, int(255 * life)))
        x, y = x + 4, ny
    glow(cv, 20, 20, 8, CYAN, int(160 * life))
    return cv.im


def wall_piece(a8, f):
    """A 24 px stretch of burning hex wall at angle a8 * 22.5 degrees."""
    S = 40
    cv = Cv(S, S)
    ang = math.radians(a8 * 22.5)
    ux, uy = math.cos(ang), math.sin(ang)
    rnd = random.Random(900 + a8 * 7 + f)
    for i in range(-12, 13, 4):
        bx, by = S / 2 + ux * i, S / 2 + uy * i
        h = 6 + (i + f * 3) % 5
        for k in range(h):   # hazard stripes rising off the wall
            col = AMBER[:3] if (k + i // 4 + f) % 2 else (40, 40, 48)
            cv.add(bx, by - k, col + (int(230 * (1 - k / (h + 2))),))
        if (i // 4 + f) % 3 == 0:
            popup(cv, bx - 3, by - h - 6, 7, 5, RED, 230)
    cv.line(S / 2 - ux * 12, S / 2 - uy * 12, S / 2 + ux * 12, S / 2 + uy * 12, (255, 200, 120, 255))
    return cv.im


def lag(f):
    """The buffering spinner over a slowed enemy (48 x 96 like every buff, so it sits over the head)."""
    cv = Cv(48, 96)
    for k in range(8):
        a = k * math.tau / 8
        on = (k - f) % 8
        cv.disc(24 + math.cos(a) * 5, 16 + math.sin(a) * 5, 1.1, (230, 235, 255, max(60, 255 - on * 28)))
    return cv.im


def overclock(f):
    """Heat pouring off his hood (on him, the 48 x 96 buff frame)."""
    cv = Cv(48, 96)
    rnd = random.Random(40 + f)
    for k in range(7):
        x = 18 + k * 2 + rnd.uniform(-1, 1)
        top = 22 - (k * 3 + f * 2) % 7
        for s in range(5):
            cv.add(x + math.sin(f + s) * 0.8, top - s, (255, 150 - s * 22, 50, 220 - s * 40))
    if f % 2:
        star4(cv, 16 + rnd.uniform(0, 16), 18 + rnd.uniform(0, 6), (255, 190, 90, 255))
    return cv.im



# ------------------------------------------------------------------ round 102: the new functions' effects

RED, ORANGE, GOLD = rgba('#ff5a5a'), rgba('#ffa040'), rgba('#ffd25a')


def ddos_fx(f):
    """A flood of requests: error pop-ups stacking on the target, torn lines."""
    cv = Cv(40, 40)
    life = 1 - f / 5
    rnd = random.Random(300 + f)
    for k in range(5):
        popup(cv, rnd.randint(4, 24), rnd.randint(6, 26), 10, 7, rnd.choice((RED, AMBER, UI_B)), 240 * life)
    for y in range(10, 32, 4):
        off = rnd.randint(-3, 3)
        cv.line(10 + off, y, 30 + off, y, UI_W[:3] + (int(90 * life),))
    return cv.im


def cleanse_fx(f):
    """Broken chains falling away, clean light rising."""
    cv = Cv(48, 56)
    life = 1 - f / 6
    for k in range(4):
        x = 12 + k * 8
        y = 30 + f * 3 + (k % 2) * 2
        cv.ring(x, y, 2.2, (170, 175, 190, int(230 * life)), 1.0)
        cv.ring(x + 3, y + 2, 2.2, (140, 145, 160, int(200 * life)), 1.0)
    for k in range(5):
        x = 10 + k * 7
        y = 40 - f * 5 - (k % 3) * 4
        star4(cv, x, y, (230, 255, 240, int(255 * life)))
    return cv.im


def swap_fx(f):
    """Two arrows chasing each other round a flash: places traded."""
    cv = Cv(40, 56)
    life = 1 - f / 6
    for k, col in ((0, CYAN), (1, GREEN)):
        a0 = f * 0.9 + k * math.pi
        for j in range(16):
            a = a0 + j * 0.12
            cv.add(20 + math.cos(a) * 12, 28 + math.sin(a) * 16, col[:3] + (int(230 * life * (j / 16)),))
        a = a0 + 16 * 0.12
        star4(cv, 20 + math.cos(a) * 12, 28 + math.sin(a) * 16, col)
    if f <= 1:
        glow(cv, 20, 28, 8, (255, 255, 255, 255), 200)
    return cv.im


def sort_fx(f):
    """A little bar chart rising over each sorted enemy."""
    cv = Cv(40, 40)
    life = 1 - max(0, f - 3) / 3
    for k, h in enumerate((3, 6, 9, 12)):
        hh = min(h, h * (f + 1) // 3)
        for y in range(hh):
            for x in range(3):
                cv.add(10 + k * 5 + x, 30 - y, (GOLD if k == 0 else CYAN)[:3] + (int(240 * life),))
    return cv.im


def kill9_fx(f):
    """SIGKILL: a red X stamped, the target's pixels crumbling into squares."""
    cv = Cv(48, 48)
    life = 1 - f / 6
    k = min(1.0, (f + 1) / 2)
    for i in range(int(14 * k)):
        for t in (-1, 0, 1):
            cv.add(24 - 10 + i + t * 0.5, 24 - 10 + i, RED[:3] + (int(255 * life),))
            cv.add(24 + 10 - i + t * 0.5, 24 - 10 + i, RED[:3] + (int(255 * life),))
    rnd = random.Random(90)
    for j in range(10):
        a = rnd.uniform(0, math.tau)
        d = 6 + f * rnd.uniform(2, 3.5)
        x, y = 24 + math.cos(a) * d, 24 + math.sin(a) * d
        for yy in range(2):
            for xx in range(2):
                cv.add(x + xx, y + yy, (255, 255, 255, int(200 * life)) if j % 3 else RED[:3] + (int(220 * life),))
    return cv.im


def rollback_fx(f):
    """A rewind: two triangles pointing back and a counter-clockwise ghost ring."""
    cv = Cv(48, 56)
    life = 1 - f / 6
    for j in range(24):
        a = -f * 0.6 - j * 0.22
        cv.add(24 + math.cos(a) * 14, 28 + math.sin(a) * 18, (190, 200, 255, int(200 * life * (1 - j / 24))))
    for k in range(2):
        x0 = 26 - k * 7
        for y in range(-4, 5):
            for x in range(0, 5 - abs(y)):
                cv.add(x0 - x, 28 + y, (220, 230, 255, int(240 * life)))
    return cv.im


def inject_fx(f):
    """A payload driven in: a '</>' window strikes, an install bar fills, a red ring."""
    cv = Cv(40, 40)
    life = 1 - f / 6
    x = 4 + min(f, 2) * 4
    popup(cv, x, 12, 19, 12, AMBER, 245 * life)
    for i, ch in enumerate('</>'):
        glyph(cv, ch, x + 2 + i * 5, 15, UI_W, 255 * life)
    if f >= 2:
        cv.ring(20, 20, 4 + (f - 2) * 3, RED[:3] + (int(230 * life),), 1.2)
        for k in range(min(15, (f - 1) * 5)):
            cv.add(x + 2 + k, 25, UI_B[:3] + (int(255 * life),))
    return cv.im


def gc_fx(f):
    """Garbage collection: a sweep arc, shield shards swept away."""
    cv = Cv(56, 56)
    life = 1 - f / 6
    a0 = -2.4 + f * 0.7
    for j in range(22):
        a = a0 + j * 0.07
        for r in (18, 19, 20):
            cv.add(28 + math.cos(a) * r, 28 + math.sin(a) * r, (200, 210, 220, int(220 * life * (j / 22))))
    rnd = random.Random(70 + f)
    for k in range(6):
        x, y = 28 + rnd.uniform(-14, 14) + f * 2, 28 + rnd.uniform(-14, 14)
        cv.add(x, y, CYAN[:3] + (int(230 * life),))
        cv.add(x + 1, y, CYAN[:3] + (int(160 * life),))
    return cv.im


# ------------------------------------------------------------------ round 102: buffs on units

def ddos_buff(f):
    """Static over a ddos'd head (48 x 96)."""
    cv = Cv(48, 96)
    rnd = random.Random(500 + f)
    for k in range(10):
        x, y = rnd.randint(16, 31), rnd.randint(12, 20)
        cv.add(x, y, rnd.choice((GREEN, CYAN, RED))[:3] + (210,))
        cv.add(x + 1, y, (255, 255, 255, 150))
    return cv.im


def boost_buff(f):
    """Orange arrows climbing his allies (48 x 56)."""
    cv = Cv(48, 56)
    for k in range(3):
        x = 12 + k * 12
        y = 44 - ((f * 4 + k * 9) % 30)
        for d in range(4):
            cv.add(x - d, y + d, ORANGE[:3] + (220,))
            cv.add(x + d, y + d, ORANGE[:3] + (220,))
        cv.line(x, y + 1, x, y + 7, ORANGE[:3] + (180,))
    return cv.im


def drone(cv, x, y, f):
    cv.disc(x, y, 2.2, (60, 70, 85, 255))
    cv.put(x, y, UI_W)
    for s in (-1, 1):
        w = 3 if f % 2 else 2
        cv.line(x + s * 3, y - 2, x + s * (3 + w), y - 2, (200, 210, 220, 230))
    cv.add(x, y + 3, GREEN[:3] + (150,))


def drones_buff(n, f):
    """1 or 2 drones orbiting at his shoulders (48 x 96)."""
    cv = Cv(48, 96)
    for k in range(n):
        a = f / 8 * math.tau + k * math.pi
        drone(cv, 24 + math.cos(a) * 16, 42 + math.sin(a) * 4, f)
    return cv.im


def marked_buff(f):
    """A red reticle over the weakest enemy (48 x 96)."""
    cv = Cv(48, 96)
    r = 5 + (f % 2)
    cv.ring(24, 16, r, RED, 1.0)
    for d in (r + 1, r + 2, r + 3):
        for sx, sy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
            cv.put(24 + sx * d, 16 + sy * d, RED)
    cv.put(24, 16, RED)
    return cv.im


def encrypt_buff(f):
    """A padlock over the encrypted ally, a cyan shimmer (48 x 96)."""
    cv = Cv(48, 96)
    for y in range(13, 20):
        for x in range(20, 29):
            cv.put(x, y, GOLD if (x + y + f) % 5 else (255, 240, 180, 255))
    cv.ring(24, 12, 3, (200, 200, 210, 255), 1.0)
    cv.put(24, 16, (60, 50, 20, 255))
    cv.put(24, 17, (60, 50, 20, 255))
    for k in range(3):
        a = f / 4 * math.tau + k * 2.1
        cv.add(24 + math.cos(a) * 9, 16 + math.sin(a) * 4, CYAN[:3] + (180,))
    return cv.im


def deploy_buff(f):
    """push("prod"): a green ring at his feet and rising sparks (48 x 56)."""
    cv = Cv(48, 56)
    for j in range(48):
        a = j / 48 * math.tau
        cv.add(24 + math.cos(a) * 14, 46 + math.sin(a) * 4, GREEN[:3] + (200 if (j + f * 3) % 6 else 255,))
    for k in range(4):   # and a deploy progress bar under him
        y = 44 - ((f * 5 + k * 10) % 34)
        star4(cv, 12 + k * 8, y, UI_W)
    for x in range(12, 37):
        cv.add(x, 52, UI_PANEL[:3] + (220,))
        if x - 12 < 4 + f * 4:
            cv.add(x, 52, UI_B[:3] + (255,))
    return cv.im


def mine_buff(f):
    """The miner on: a spinning fan by his shoulder, coins popping off it (48 x 96)."""
    cv = Cv(48, 96)
    cx, cy = 8, 46
    for k in range(4):
        a = f * 0.8 + k * math.pi / 2
        cv.line(cx, cy, cx + math.cos(a) * 3.5, cy + math.sin(a) * 3.5, (170, 180, 195, 255))
    cv.ring(cx, cy, 4.5, (90, 95, 110, 255), 1.0)
    cv.put(cx + 3, cy - 6 - (f % 3) * 2, GOLD)
    cv.put(cx - 2, cy - 7 - ((f + 1) % 3) * 2, GOLD)
    return cv.im


def ai_icon(p, lite, f):
    """The AI writing for him, by his terminal (48 x 96, top-left): Claude a spark, ChatGPT a knot, Gemini a star;
    the lite model smaller and greyed."""
    cv = Cv(48, 96)
    cx, cy = 8, 8
    k = 0.6 if lite else 1.0
    col = [(255, 150, 90), (140, 230, 190), (150, 160, 255)][p]
    if lite:
        col = tuple(int(c * 0.7 + 60) for c in col)
    if p == 0:      # a spark: rays of uneven length
        for j in range(8):
            a = j * math.pi / 4 + f * 0.2
            ln = (5 if j % 2 == 0 else 3) * k
            cv.line(cx, cy, cx + math.cos(a) * ln, cy + math.sin(a) * ln, col + (255,))
    elif p == 1:    # a knot: three interlaced loops
        for j in range(3):
            a = j * math.tau / 3 + f * 0.3
            cv.ring(cx + math.cos(a) * 2 * k, cy + math.sin(a) * 2 * k, 2.4 * k, col + (255,), 1.0)
    else:           # a four-point star
        for j in range(4):
            a = j * math.pi / 2 + f * 0.25
            for t in range(int(6 * k)):
                w = 1 - t / (6 * k)
                cv.add(cx + math.cos(a) * t, cy + math.sin(a) * t, col + (int(255 * w),))
        cv.disc(cx, cy, 1.2 * k, col + (255,))
    if lite:
        cv.put(cx + 5, cy + 4, (200, 200, 200, 255))
    return cv.im


def btc_digit(place, d):
    """The Bitcoin readout by his crest: a coin and three digits (48 x 96); the coin rides with the hundreds."""
    cv = Cv(48, 96)
    y = 23   # left of his head, opposite the crest
    x = {'h': 6, 't': 11, 'o': 16}[place]
    glyph(cv, str(d), x, y, GOLD)
    if place == 'h':
        cv.disc(2, y + 3, 2.2, GOLD)
        cv.put(2, y + 3, (120, 80, 10, 255))
    return cv.im

# ------------------------------------------------------------------ the rig HUD (48 x 96, centred on him)

def heat_bar(n):
    """10 segments, blue to red; drawn where Levi's gas bar sits (just under the HP bar)."""
    cv = Cv(48, 96)
    for x in range(13, 35):
        cv.put(x, 12, rgba('#14111c'))
        cv.put(x, 15, rgba('#14111c'))
    for y in range(12, 16):
        cv.put(12, y, rgba('#14111c'))
        cv.put(35, y, rgba('#14111c'))
    for k in range(10):
        c = hsv(0.62 - 0.62 * k / 9, 0.85, 1.0) if k < n else rgba('#1b2630')
        for x in range(13 + k * 2, 15 + k * 2):
            for y in (13, 14):
                cv.put(x, y, c)
    if n >= 9:   # about to blue-screen: a warning blink
        cv.put(37, 13, (255, 80, 80, 255))
        cv.put(37, 14, (255, 80, 80, 255))
    return cv.im


def ram_bar(n):
    cv = Cv(48, 96)
    for k in range(8):
        c = (rgba('#c47bff') if n < 7 else rgba('#ff5a5a')) if k < n else rgba('#2a2338', 220)
        cv.put(14 + k * 2.5, 17, c)
        cv.put(15 + k * 2.5, 17, c)
    return cv.im


def disk_dots(n):
    cv = Cv(48, 96)
    for k in range(8):
        cv.put(14 + k * 2.5, 19, rgba('#ffd25a') if k < n else rgba('#3a3424', 200))
    return cv.im


# ------------------------------------------------------------------ rank crests

TIERS = [  # fill, rim, rim dark, emblem, emblem colour
    ('#2b2f36', '#8a929c', '#555c66', '>_', '#3cff8a'),   # Script Kiddie: a bare prompt
    ('#3b2f24', '#c08a55', '#7a5534', '?', '#ffe2b0'),    # Intern
    ('#3b2a14', '#ffb34a', '#9a6420', '{}', '#ffe2b0'),   # Junior (round 106: amber, his theme)
    ('#1d2c48', '#6aa8ff', '#33558f', '</', '#d8ecff'),   # Developer
    ('#2e2048', '#a77bff', '#5b3f99', 'fn', '#efe2ff'),   # Senior
    ('#3d3418', '#ffd25a', '#9c7a20', '**', '#fff4c8'),   # Staff
    ('#173a40', '#55e0f0', '#24808c', '::', '#e0fcff'),   # Architect
]
BX, BY = 40, 26


def emblem(cv, text, col, cx, cy):
    w = F.CW * len(text) - 1
    for i, ch in enumerate(text):
        glyph(cv, ch, cx - w / 2 + i * F.CW, cy - 5, col)


def badge(r, f):
    if r >= 4:
        return high_badge(r, f)
    cv = Cv(48, 96)
    fill, rim, rim_d, em, ec = TIERS[r]
    wide = 6 if len(em) == 1 else 7
    crest_shape(cv, BX, BY, rgba(fill), rgba(rim), rgba(rim_d), wide=wide)
    emblem(cv, em, rgba(ec), BX, BY)
    return cv.im


# round 103: from Senior up the crests are their own constructions on a wider canvas (64 x 96, still centred on him,
# the crest at the same place: HX, BY)
HX = 48


def hexagon(cv, cx, cy, r, ry, col):
    pts = [(cx + math.cos(k * math.pi / 3) * r, cy + math.sin(k * math.pi / 3) * ry) for k in range(6)]
    cv.poly(pts, col)


def high_badge(r, f):
    cv = Cv(64, 96)
    fill, rim, rim_d, em, ec = TIERS[r]
    fill, rim, rim_d, ec = rgba(fill), rgba(rim), rgba(rim_d), rgba(ec)
    if r == 4:      # Senior: a hexagon with an inner rim, two side nodes
        hexagon(cv, HX, BY, 10, 9, rim_d)
        hexagon(cv, HX, BY, 9, 8, rim)
        hexagon(cv, HX, BY, 7.6, 6.8, fill)
        for s in (-1, 1):
            cv.disc(HX + s * 12, BY, 1.2, rim if (f // 2 + (s > 0)) % 2 else rim_d)
        emblem(cv, em, ec, HX, BY)
        glint(cv, HX, BY, f, wide=10, tall=9)
    elif r == 5:    # Staff: a gold double rim with wings, a twinkle
        for s in (-1, 1):
            for k in range(3):
                y0 = BY - 4 + k * 3
                ln = 6 - k
                flap = (1 if (f // 2) % 2 else 0) * (k == 0)
                cv.line(HX + s * 8, y0, HX + s * (8 + ln), y0 - 3 - flap, rim if k != 1 else rgba('#fff4c8'))
        crest_shape(cv, HX, BY, rim_d, rim_d, rim_d, wide=9, tall=10)
        crest_shape(cv, HX, BY, fill, rim, rim_d, wide=8, tall=9)
        emblem(cv, em, ec, HX, BY)
        glint(cv, HX, BY, f, wide=9, tall=10)
        if f % 4 == 0:
            star4(cv, HX + 7, BY - 9, ec, big=True)
        if f % 4 == 2:
            star4(cv, HX - 8, BY + 4, ec)
    else:           # Architect: a circuit-board diamond, traces to nodes that light in turn
        for k, (dx, dy) in enumerate(((1, 0), (0, 1), (-1, 0), (0, -1))):
            x1, y1 = HX + dx * 15, BY + dy * 12
            cv.line(HX + dx * 10, BY + dy * 10, x1, y1, rim_d)
            on = (f // 2) % 4 == k
            cv.disc(x1, y1, 1.3, rgba('#e0fcff') if on else rim)
            if on:
                glow(cv, x1, y1, 4, rim, 140)
        cv.poly([(HX, BY - 11), (HX + 11, BY), (HX, BY + 11), (HX - 11, BY)], rim)
        cv.poly([(HX, BY - 9), (HX + 9, BY), (HX, BY + 9), (HX - 9, BY)], fill)
        for d in (-4, 4):   # the board's own traces
            cv.line(HX - 6 + abs(d) // 2, BY + d, HX + 6 - abs(d) // 2, BY + d, rim_d)
        emblem(cv, em, ec, HX, BY)
        glint(cv, HX, BY, f, wide=11, tall=11)
    return cv.im


def window(cv, cx, cy, w, h, rim, f):
    """A terminal window: a title bar with three dots, a black body."""
    x0, y0 = int(cx - w / 2), int(cy - h / 2)
    for y in range(h):
        for x in range(w):
            edge = x in (0, w - 1) or y in (0, h - 1)
            cv.put(x0 + x, y0 + y, rim if edge else (rgba('#1a1d24') if y < 4 else rgba('#07090c')))
    for k, c in enumerate(('#ff5f56', '#ffbd2e', '#27c93f')):
        cv.put(x0 + 2 + k * 2, y0 + 2, rgba(c))
    return x0, y0


def root_badge(p, f):
    """Root #p: a terminal window with a red-green rim, the number at the prompt and a blinking cursor; #1 (Zero-Day)
    a window shimmering through every colour with a skull between brackets, torn by glitches."""
    cv = Cv(64, 96)
    if p > 1:
        rim = rgba('#ff4d6d') if (f // 2) % 2 else rgba('#ffd0d8')   # round 106: red on black, no green
        glow(cv, HX, BY, 13, rim, 70)
        x0, y0 = window(cv, HX, BY, 20, 16, rim, f)
        glyph(cv, '#', x0 + 2, y0 + 6, GREEN)
        s = str(p)
        for i, ch in enumerate(s):
            glyph(cv, ch, x0 + 7 + i * 5, y0 + 6, rgba('#ffffff'))
        if f % 2 == 0:
            for y in range(6):
                cv.put(x0 + 8 + len(s) * 5, y0 + 7 + y, GREEN)
        glint(cv, HX, BY, f, wide=10, tall=8, a=120)
        return cv.im
    rim = hsv(f / 8, 0.75, 1.0)
    glow(cv, HX, BY, 16, rim, 110)
    x0, y0 = window(cv, HX, BY, 24, 17, rim, f)
    glyph(cv, '[', x0 + 2, y0 + 6, rim)
    glyph(cv, ']', x0 + 17, y0 + 6, rim)
    skull(cv, HX, y0 + 10, hsv(f / 8 + 0.5, 0.3, 1.0))
    for k in range(3):     # a crown of three pixels over the window
        cv.put(HX - 4 + k * 4, y0 - 2 - (k == 1), hsv(f / 8 + k * 0.2, 0.6, 1.0))
        cv.put(HX - 4 + k * 4, y0 - 1, hsv(f / 8 + k * 0.2, 0.6, 1.0))
    if f % 2 == 1:         # a glitch: a slice of it torn sideways, a red / cyan fringe
        y = y0 + 3 + (f * 5) % 12
        row = [cv.get(x, y) for x in range(x0 - 2, x0 + 27)]
        for i, c in enumerate(row):
            cv.px[min(63, x0 - 2 + i + 2), y] = c if c[3] else cv.get(x0 - 2 + i + 2, y)
        cv.add(x0 - 1, y + 1, (255, 60, 90, 220))
        cv.add(x0 + 25, y - 1, (60, 230, 255, 220))
    glint(cv, HX, BY, f, wide=12, tall=9, a=140)
    return cv.im


def skull(cv, cx, cy, col):
    rows = ['.###.', '#####', '#.#.#', '#####', '.#.#.']
    for y, row in enumerate(rows):
        for x, b in enumerate(row):
            if b == '#':
                cv.put(cx - 2 + x, cy - 2 + y, col)


# ------------------------------------------------------------------ the sheet

def all_anims():
    A = {}
    A['bit'] = ([bit(f) for f in range(2)], 0.08)   # his basic attack's projectile (data's view_projectiles)
    A['fx_send'] = ([send(f) for f in range(4)], 0.05)
    A['fx_ping'] = ([ping(f) for f in range(6)], 0.05)
    A['fx_heal'] = ([heal(f) for f in range(6)], 0.06)
    A['fx_shield'] = ([shield_on(f) for f in range(6)], 0.05)
    A['fx_scan'] = ([scan(f) for f in range(6)], 0.07)
    A['fx_spray'] = ([spray(f) for f in range(6)], 0.06)
    A['fx_blink_out'] = ([blink(f, True) for f in range(6)], 0.05)
    A['fx_blink_in'] = ([blink(f, False) for f in range(6)], 0.05)
    A['fx_cache'] = ([cache(f) for f in range(8)], 0.07)
    A['fx_chain'] = ([chain(f) for f in range(5)], 0.05)
    for a8 in range(8):
        A[f'fx_wall_{a8}'] = ([wall_piece(a8, f) for f in range(4)], 0.09)
    # round 102
    A['fx_ddos'] = ([ddos_fx(f) for f in range(5)], 0.05)
    A['fx_cleanse'] = ([cleanse_fx(f) for f in range(6)], 0.06)
    A['fx_swap'] = ([swap_fx(f) for f in range(6)], 0.05)
    A['fx_sort'] = ([sort_fx(f) for f in range(6)], 0.06)
    A['fx_kill9'] = ([kill9_fx(f) for f in range(6)], 0.05)
    A['fx_rollback'] = ([rollback_fx(f) for f in range(6)], 0.05)
    A['fx_inject'] = ([inject_fx(f) for f in range(6)], 0.06)
    A['fx_gc'] = ([gc_fx(f) for f in range(6)], 0.05)
    A['ddos'] = ([ddos_buff(f) for f in range(4)], 0.08)
    A['boost'] = ([boost_buff(f) for f in range(6)], 0.08)
    A['drone1'] = ([drones_buff(1, f) for f in range(8)], 0.08)
    A['drone2'] = ([drones_buff(2, f) for f in range(8)], 0.08)
    A['marked'] = ([marked_buff(f) for f in range(2)], 0.2)
    A['encrypt'] = ([encrypt_buff(f) for f in range(4)], 0.1)
    A['deploy'] = ([deploy_buff(f) for f in range(6)], 0.08)
    A['mine'] = ([mine_buff(f) for f in range(6)], 0.08)
    for p, name in enumerate(('claude', 'gpt', 'gemini')):
        A[f'ai_{name}'] = ([ai_icon(p, False, f) for f in range(4)], 0.1)
        A[f'ai_{name}_lite'] = ([ai_icon(p, True, f) for f in range(4)], 0.1)
    for place in ('h', 't', 'o'):
        for d in range(10):
            A[f'btc_{place}{d}'] = ([btc_digit(place, d)], 0.5)
    A['shield'] = ([shield_buff(f) for f in range(4)], 0.1)
    A['lag'] = ([lag(f) for f in range(8)], 0.08)
    A['oc'] = ([overclock(f) for f in range(6)], 0.07)
    for n in range(11):
        A[f'heat{n}'] = ([heat_bar(n)], 0.1)
    for n in range(9):
        A[f'ram{n}'] = ([ram_bar(n)], 0.1)
        A[f'disk{n}'] = ([disk_dots(n)], 0.1)
    for r in range(7):
        A[f'rank{r}'] = ([badge(r, f) for f in range(8)], 0.1)
    for p in range(1, 11):
        A[f'root{p}'] = ([root_badge(p, f) for f in range(8)], 0.1)
    return A


def pack(A, width=2048):
    frames = [(name, i, im) for name, (ims, _) in A.items() for i, im in enumerate(ims)]
    frames.sort(key=lambda e: (-e[2].height, -e[2].width))
    x = y = row = 0
    pos = {}
    for name, i, im in frames:
        if x + im.width > width:
            x, y, row = 0, y + row + 1, 0
        pos[(name, i)] = (x, y)
        x += im.width + 1
        row = max(row, im.height)
    sheet = Image.new('RGBA', (width, y + row), (0, 0, 0, 0))
    anims = {}
    for name, (ims, dur) in A.items():
        fr = []
        for i, im in enumerate(ims):
            px, py = pos[(name, i)]
            sheet.paste(im, (px, py))
            fr.append({'duration': dur, 'data': {'x': px, 'y': py, 'w': im.width, 'h': im.height}})
        anims[name] = {'frames': fr}
    assert sheet.height <= 2048, sheet.size
    return sheet, {'anims': anims}


def preview(A, folder):
    bg = (54, 74, 60, 255)
    keys = ['fx_send', 'fx_ping', 'fx_heal', 'fx_shield', 'fx_spray', 'fx_blink_out', 'fx_cache', 'fx_chain', 'fx_wall_2', 'shield', 'lag', 'oc']
    rows = []
    for k in keys:
        ims = A[k][0]
        w = sum(i.width + 4 for i in ims)
        h = max(i.height for i in ims)
        r = Image.new('RGBA', (w, h), bg)
        x = 0
        for i in ims:
            r.alpha_composite(i, (x, (h - i.height) // 2))
            x += i.width + 4
        rows.append(r)
    hud = Image.new('RGBA', (48 * 9, 40), bg)
    for n in range(9):
        for k in (f'heat{min(10, n + 2)}', f'ram{n}', f'disk{n}', f'rank{min(6, n)}' if n < 7 else f'root{11 - (n - 6) * 5 if n == 7 else 1}'):
            im = A[k][0][0]
            o = (im.width - 48) // 2
            hud.alpha_composite(im.crop((o, 4, o + 48, 44)), (n * 48, 0))
    rows.append(hud)
    W = max(r.width for r in rows)
    out = Image.new('RGBA', (W, sum(r.height + 4 for r in rows)), bg)
    y = 0
    for r in rows:
        out.alpha_composite(r, (0, y))
        y += r.height + 4
    out.resize((out.width * 3, out.height * 3), Image.NEAREST).save(os.path.join(folder, 'vfx.png'))


if __name__ == '__main__':
    A = all_anims()
    sheet, fan = pack(A)
    if '--preview' in sys.argv:
        preview(A, os.path.join(HERE, 'preview'))
    if '--dry' not in sys.argv:
        sheet.save(os.path.join(OUT, 'coder_vfx#sheet.png'), optimize=True)
        with open(os.path.join(OUT, 'coder_vfx#anim.fanim'), 'w', encoding='utf-8') as fh:
            json.dump(fan, fh, separators=(',', ':'))
    print('coder_vfx', sheet.size, len(fan['anims']), 'anims')
