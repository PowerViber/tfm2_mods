"""Isliid's combat art, native pixels (round 88: the sword-kit step-up, the sword-crown badges, the effect logos).

The consolidated tfm2_custom mod is the source of truth; this writes its sheets, the views in Isliid's data and the
editor's manifest. Run from the repo root: python tools/generate_isliid_eight_frame_art.py [--preview DIR]

Swords (one identity each, colours = the engraving scar colours, a silhouette per sword and per rank tier):
  0 Skylight needle, 1 Terra slab, 2 Darkbringer serrated, 3 Gale curved saber, 4 Blood hooked, 5 Rift split prongs,
  6 Emperor command blade. Rank tiers: the guard grows from a bar (Bearer) to swept quillons, wings and a crown guard
  (Imperial); the blade lengthens, gains a rune fuller, an energy edge, and at Imperial a halo.

Sheets:
  swords_fly8   projectiles, tip pointing right (the engine turns projectile art to its heading):
                <sword>_rank<r>_flight_f<k> / _drawing_f<k>, 4 single-frame aliases (the native code re-spawns a short
                segment every 3 ticks with frame (tick/6)%4, so the smear animates without restarting)
  swords8       grounded swords at their point: <sword>_rank<r>_planted / _ready (8 frames) and _frame<k> aliases;
                <sword>_impact (plant), _launch, _recall (snap), _hit (melee slash)
  orbit8        the arsenal ring on its holder: ar_<sword>_rank<r> and ar_<sword>_rank<r>_sel (selected), small blades
                swaying round their slots, the back of the ring dimmer
  badges8       mastery: a fan of blades that gains one sword per rank (1 -> 7), iron -> silver -> gold -> gemmed;
                Imperial #10..#1 crown the seven-blade fan, pixel digits, gems from #3, prismatic rays at #1
  logos         24 x 24 effect logos replacing the text flags: logo_<family>_<phase>, logo_solo<sword>_<phase>
  auras8, aura_fields8 (unchanged)
"""
from __future__ import annotations

import json
import math
import re
import shutil
import sys
from pathlib import Path

from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parents[1]
MOD = ROOT / "mods" / "tfm2_custom"
EDITOR = ROOT / "editor"
SWORDS = ("skylight", "terra", "darkbringer", "gale", "blood", "rift", "emperor")
INK = (10, 16, 36, 255)
STEEL = (231, 237, 244, 255)
STEEL_D = (150, 162, 182, 255)
# (dark, colour, bright): the colour is the sword's engraving scar colour (tools/rework_isliid_art.py COLORS)
HEX = (
    ("#5d6f8f", "#f6edaa", "#ffffff"),   # Skylight: cream light on slate
    ("#6e4a2a", "#b79769", "#f6deb0"),   # Terra: earth
    ("#3a2463", "#a479d1", "#ead8ff"),   # Darkbringer: violet
    ("#0b6b6e", "#8de8d9", "#e8fffb"),   # Gale: sea-wind
    ("#7a1c34", "#e87283", "#ffd0da"),   # Blood
    ("#1f3f8c", "#75a7fa", "#dbe7ff"),   # Rift: deep blue
    ("#8a5a14", "#ffd166", "#fff4c9"),   # Emperor: gold
)


def rgb(h, a=255):
    h = h.lstrip("#")
    return (int(h[0:2], 16), int(h[2:4], 16), int(h[4:6], 16), a)


COLORS = tuple(tuple(rgb(c) for c in t) for t in HEX)
PHASES = ("planned", "drawing", "complete", "cancelled")
FAMILIES = ("damage", "bind", "pull", "push", "speed", "shred", "weaken", "guard", "attack", "burst", "cooldown",
            "heal", "domain")


def A(c, a):
    return c[:3] + (int(a),)


# ------------------------------------------------------------------ the sword model (pommel at x 0, tip at x L)

def sword_parts(kind: int, rank: int):
    """Polygons in sword space: x along the blade (0 pommel .. L tip), y across. Returns (L, parts) where parts are
    (role, points). Roles: grip, pommel, guard, blade, edge (a light line), fuller, gem."""
    tier = (0, 0, 1, 1, 2, 2, 3, 3)[rank]
    L = 34 + rank          # longer with rank
    g = 9                  # guard position
    w = 1.0 + rank * 0.12  # blade width factor
    parts = [("grip", [(2, -1.2), (g - 1, -1.2), (g - 1, 1.2), (2, 1.2)]),
             ("pommel", [(0, -2), (2.5, -2), (2.5, 2), (0, 2)])]
    # guards by tier: bar, swept quillons, wings, crown
    if tier == 0:
        parts.append(("guard", [(g - 1, -4), (g + 1, -4), (g + 1, 4), (g - 1, 4)]))
    elif tier == 1:
        parts.append(("guard", [(g - 3, -6), (g + 1, -3), (g + 1, 3), (g - 3, 6), (g - 1, 2.5), (g - 1, -2.5)]))
    elif tier == 2:
        parts.append(("guard", [(g - 4, -8), (g, -5), (g + 2, -2.5), (g + 2, 2.5), (g, 5), (g - 4, 8), (g - 1, 3), (g - 1, -3)]))
    else:
        parts.append(("guard", [(g - 5, -9), (g - 1, -6), (g, -9.5), (g + 2, -4), (g + 2, 4), (g, 9.5), (g - 1, 6), (g - 5, 9), (g - 1, 3), (g - 1, -3)]))
    b0 = g + 1
    if kind == 0:      # Skylight: a needle
        blade = [(b0, -1.6 * w), (L - 6, -1.2 * w), (L, 0), (L - 6, 1.2 * w), (b0, 1.6 * w)]
    elif kind == 1:    # Terra: a broad slab, chisel tip
        blade = [(b0, -4.2 * w), (L - 3, -4.2 * w), (L, -1), (L, 1.5), (L - 4, 4.2 * w), (b0, 4.2 * w)]
    elif kind == 2:    # Darkbringer: serrated back edge
        blade = [(b0, -3 * w)]
        for k in range(4):
            x = b0 + 4 + k * (L - b0 - 10) / 4
            blade += [(x, -3 * w), (x + 2, -4.6 * w), (x + 3.5, -3 * w)]
        blade += [(L - 4, -2.5 * w), (L, 0), (L - 5, 2.6 * w), (b0, 2.8 * w)]
    elif kind == 3:    # Gale: a curved saber (single edge sweeping up)
        blade = []
        n = 8
        for k in range(n + 1):
            x = b0 + (L - b0) * k / n
            c = -0.012 * (x - b0) ** 2
            blade.append((x, c - (2.6 * w if k < n else 0)))
        for k in range(n, -1, -1):
            x = b0 + (L - b0) * k / n
            c = -0.012 * (x - b0) ** 2
            blade.append((x, c + (2.0 * w if k < n else 0)))
    elif kind == 4:    # Blood: a hook near the tip
        blade = [(b0, -2.8 * w), (L - 8, -2.8 * w), (L - 5, -6 * w), (L - 3, -5 * w), (L - 4, -2.5 * w), (L, 0),
                 (L - 6, 2.8 * w), (b0, 2.8 * w)]
    elif kind == 5:    # Rift: two prongs split down the middle
        m = (b0 + L) / 2 - 2
        blade = [(b0, -3.4 * w), (L, -2.4 * w), (L - 3, -1.2 * w), (m, -0.6), (m, 0.6), (L - 3, 1.2 * w), (L, 2.4 * w), (b0, 3.4 * w)]
    else:              # Emperor: a wide command blade tapering to a long point
        blade = [(b0, -4.6 * w), (b0 + 6, -3.6 * w), (L - 9, -2.8 * w), (L, 0), (L - 9, 2.8 * w), (b0 + 6, 3.6 * w), (b0, 4.6 * w)]
    parts.append(("blade", blade))
    if kind not in (5,):
        parts.append(("fuller", [(b0 + 2, 0), (L - 7, 0)]))
    if rank >= 2:
        parts.append(("gem", [(g - 0.5, -1.4), (g + 1.5, 0), (g - 0.5, 1.4), (g - 2, 0)]))
    return L, parts


def transform(points, origin, ang, scale, L_center=None):
    """Sword space -> image: rotate by ang (radians, 0 = tip pointing +x) around the sword's own centre (or a given x)."""
    cx = L_center
    c, s = math.cos(ang), math.sin(ang)
    out = []
    for x, y in points:
        x -= cx
        out.append((origin[0] + (x * c - y * s) * scale, origin[1] + (x * s + y * c) * scale))
    return out


def draw_sword(d: ImageDraw.ImageDraw, kind: int, rank: int, origin, ang, scale=1.0, alpha=255, pivot="mid", glow=0.0):
    dark, color, bright = COLORS[kind]
    L, parts = sword_parts(kind, rank)
    cx = {"mid": L / 2, "tip": L, "grip": 5}[pivot]
    T = lambda pts: transform(pts, origin, ang, scale, cx)
    for role, pts in parts:
        P = T(pts)
        if role == "grip":
            d.polygon(P, fill=A((48, 40, 52, 255), alpha), outline=A(INK, alpha))
        elif role == "pommel":
            d.polygon(P, fill=A(dark, alpha), outline=A(INK, alpha))
        elif role == "guard":
            d.polygon(P, fill=A(color if rank >= 4 else STEEL_D, alpha), outline=A(INK, alpha))
        elif role == "blade":
            d.polygon(P, fill=A(STEEL if kind in (0, 3) else dark, alpha), outline=A(INK, alpha))
            if rank >= 4:   # an energy edge in the sword's colour
                d.line(P + [P[0]], fill=A(color, alpha * (0.75 + 0.25 * glow)), width=1)
        elif role == "fuller":
            d.line(P, fill=A(color if kind not in (0, 3) else (170, 200, 220, 255), alpha), width=max(1, round(scale)))
            if rank >= 3:   # runes along the fuller
                for k in range(1, 4):
                    q = (P[0][0] + (P[1][0] - P[0][0]) * k / 4, P[0][1] + (P[1][1] - P[0][1]) * k / 4)
                    d.point(q, fill=A(bright, alpha))
        elif role == "gem":
            d.polygon(P, fill=A(bright if rank >= 6 else color, alpha), outline=A(INK, alpha))
    return L


# ------------------------------------------------------------------ flying swords (projectiles, tip right)

def flight_frame(kind: int, rank: int, k: int, drawing: bool) -> Image.Image:
    W, H = 72, 24
    im = Image.new("RGBA", (W, H))
    d = ImageDraw.Draw(im, "RGBA")
    dark, color, bright = COLORS[kind]
    cx, cy = W / 2 + 10, H / 2          # the projectile position: the sword's centre, a little forward
    # the wake: a tapered ribbon behind the sword in its colour (brighter and longer while engraving)
    length = 30 + (14 if drawing else 0) + 3 * math.sin(k * math.pi / 2)
    tail_x = cx - 17
    for j in range(int(length)):
        t = j / length
        half = (3.2 if drawing else 2.4) * (1 - t) ** 0.8 + 0.3
        wob = math.sin(j * 0.5 + k * 1.6) * 0.6 * t
        a = 220 * (1 - t) ** 1.2
        x = tail_x - j
        d.line([(x, cy - half + wob), (x, cy + half + wob)], fill=A(color, a))
        if j % 2 == 0:
            d.point((x, cy + wob), fill=A(bright, a))
    # speed lines
    for j in range(3):
        y = cy + (-6, 5, -2)[j] + (k + j) % 2
        x0 = tail_x - 6 - ((k * 5 + j * 9) % 18)
        d.line([(x0, y), (x0 - 6 - j * 2, y)], fill=A(bright, 150))
    if drawing:   # sparks thrown off the engraving
        for j in range(4):
            x = tail_x - ((k * 7 + j * 11) % 30)
            y = cy + ((-1) ** j) * (3 + (j + k) % 3)
            d.point((x, y), fill=A((255, 255, 255, 255), 230))
    draw_sword(d, kind, rank, (cx, cy), 0.0, 1.0, glow=k / 3)
    if rank == 7:
        d.point((cx + 18, cy - 3 + k % 3), fill=(255, 255, 255, 255))
    return im


# ------------------------------------------------------------------ grounded swords and one-shot effects

def planted_frame(kind: int, rank: int, phase: int, ready: bool) -> Image.Image:
    """The sword stuck in the ground, tip at the image centre (= its point), hilt up; the ground cracked around it."""
    W, H = 40, 96
    im = Image.new("RGBA", (W, H))
    d = ImageDraw.Draw(im, "RGBA")
    dark, color, bright = COLORS[kind]
    cx, cy = W / 2, H / 2
    pulse = 0.5 + 0.5 * math.sin(phase / 8 * math.tau)
    # the ground: a crack and a glow pool (a rune circle when armed)
    if ready:
        d.ellipse((cx - 15, cy - 4, cx + 15, cy + 6), outline=A(color, 140 + 100 * pulse), width=1)
        for k in range(6):
            a = k * math.tau / 6 + phase * math.tau / 24
            d.point((cx + math.cos(a) * 15, cy + 1 + math.sin(a) * 5), fill=A(bright, 255))
        d.ellipse((cx - 9, cy - 2, cx + 9, cy + 4), outline=A(bright, 90 + 120 * pulse))
    else:
        d.ellipse((cx - 9, cy - 2, cx + 9, cy + 4), fill=A(color, 40 + 40 * pulse))
    for a, l in ((200, 8), (330, 7), (20, 6), (150, 5)):
        r = math.radians(a)
        d.line([(cx, cy + 1), (cx + math.cos(r) * l, cy + 1 + math.sin(r) * l * 0.45)], fill=A((40, 30, 24, 255), 200))
    # the sword, tip slightly in the ground
    draw_sword(d, kind, rank, (cx, cy + 3), math.pi / 2, 1.0, pivot="tip", glow=pulse)
    # motes rising, more with rank
    for j in range(1 + rank // 2):
        y = cy - ((phase * 4 + j * 11) % 34)
        x = cx + ((j * 7 + phase) % 9) - 4
        d.point((x, y), fill=A(bright if j % 2 else color, 120 + 120 * (1 - (cy - y) / 34)))
    return im


def impact_frame(kind: int, f: int, n: int = 6) -> Image.Image:
    """A sword plants: dust thrown out, the ground cracks, a ring in the sword's colour."""
    im = Image.new("RGBA", (56, 40))
    d = ImageDraw.Draw(im, "RGBA")
    dark, color, bright = COLORS[kind]
    cx, cy = 28, 22
    t = f / (n - 1)
    a = 255 * (1 - t) + 30
    r = 4 + t * 20
    d.ellipse((cx - r, cy - r * 0.45, cx + r, cy + r * 0.45), outline=A(color, a), width=2)
    if f < 2:
        d.ellipse((cx - 6, cy - 4, cx + 6, cy + 4), fill=A(bright, 230))
    for k in range(7):
        ang = k * math.tau / 7 + 0.3
        dd = 3 + t * 16
        x, y = cx + math.cos(ang) * dd, cy + math.sin(ang) * dd * 0.45 - t * 6 * abs(math.sin(ang * 3))
        d.ellipse((x - 2 + t, y - 2 + t, x + 2 - t, y + 2 - t), fill=A((176, 160, 136, 255), a * 0.8))
    for ang, l in ((190, 10), (320, 12), (30, 9), (140, 8)):
        rr = math.radians(ang)
        d.line([(cx, cy), (cx + math.cos(rr) * l * min(1, t * 3), cy + math.sin(rr) * l * 0.45 * min(1, t * 3))], fill=A((40, 30, 24, 255), 220 - 120 * t))
    return im


def launch_frame(kind: int, f: int, n: int = 4) -> Image.Image:
    im = Image.new("RGBA", (32, 32))
    d = ImageDraw.Draw(im, "RGBA")
    dark, color, bright = COLORS[kind]
    t = f / (n - 1)
    r = 3 + t * 11
    d.ellipse((16 - r, 16 - r, 16 + r, 16 + r), outline=A(color, 255 * (1 - t) + 30), width=2)
    if f == 0:
        d.ellipse((12, 12, 20, 20), fill=A(bright, 240))
    for k in range(4):
        ang = k * math.pi / 2 + math.pi / 4
        d.line([(16 + math.cos(ang) * r * 0.5, 16 + math.sin(ang) * r * 0.5), (16 + math.cos(ang) * (r + 3), 16 + math.sin(ang) * (r + 3))], fill=A(bright, 220 * (1 - t) + 30))
    return im


def recall_frame(kind: int, f: int, n: int = 5) -> Image.Image:
    """The recall snap: light collapses inward and blinks out."""
    im = Image.new("RGBA", (40, 40))
    d = ImageDraw.Draw(im, "RGBA")
    dark, color, bright = COLORS[kind]
    t = f / (n - 1)
    r = 16 * (1 - t) + 2
    for k in range(8):
        ang = k * math.pi / 4 + t
        d.line([(20 + math.cos(ang) * r, 20 + math.sin(ang) * r), (20 + math.cos(ang) * (r + 5), 20 + math.sin(ang) * (r + 5))], fill=A(color, 230 * (1 - t * 0.6)))
    d.ellipse((20 - r * 0.5, 20 - r * 0.5, 20 + r * 0.5, 20 + r * 0.5), outline=A(bright, 220))
    if f == n - 1:
        d.ellipse((17, 17, 23, 23), fill=A((255, 255, 255, 255), 255))
    return im


def hit_frame(kind: int, f: int, n: int = 5) -> Image.Image:
    """A melee hit: a slash arc in the sword's colour and a spark (no second sword on the target)."""
    im = Image.new("RGBA", (40, 40))
    d = ImageDraw.Draw(im, "RGBA")
    dark, color, bright = COLORS[kind]
    t = f / (n - 1)
    span = 50 + 130 * min(1, t * 2)
    start = -160 + 40 * t
    a = 255 if t < 0.6 else 255 * (1 - (t - 0.6) / 0.4) + 30
    for w, c in ((0, (255, 255, 255, 255)), (1, bright), (2, color), (3, dark)):
        rr = 15 - w
        d.arc((20 - rr, 20 - rr, 20 + rr, 20 + rr), start, start + span, fill=A(c, a), width=1)
    if f < 3:
        for k in range(4):
            ang = math.radians(start + span) + k * 0.6
            d.point((20 + math.cos(ang) * (8 + f * 3), 20 + math.sin(ang) * (8 + f * 3)), fill=A(bright, 255))
    return im


# ------------------------------------------------------------------ the arsenal ring (a buff on the holder)

SLOT_RX, SLOT_RY, RING_Y = 22, 12, 2     # the ring round his middle (his position is the image centre)


def orbit_frame(kind: int, rank: int, phase: int, selected: bool) -> Image.Image:
    """One blade of the arsenal halo: its hilt on a ring round him, the blade pointing outward, swaying round its slot
    (each sword is its own buff, so slots never cross); the far side of the ring smaller and dimmer."""
    im = Image.new("RGBA", (128, 128))
    d = ImageDraw.Draw(im, "RGBA")
    dark, color, bright = COLORS[kind]
    sway = math.sin((phase / 8) * math.tau + kind * 0.9) * 0.22 * (math.tau / 7)
    a = -math.pi / 2 + kind * math.tau / 7 + sway
    x = 64 + math.cos(a) * SLOT_RX
    y = 64 + RING_Y + math.sin(a) * SLOT_RY + 1.6 * math.cos((phase / 8) * math.tau + kind * 0.9)   # a bob a quarter turn off the sway
    back = math.sin(a) < -0.2
    scale = 0.46 if back else 0.56
    alpha = 165 if back else 255
    out = math.atan2(math.sin(a) * SLOT_RY / SLOT_RX * 1.6, math.cos(a))   # outward on the flattened ring
    if selected:
        tip = (x + math.cos(out) * 20, y + math.sin(out) * 20)
        d.line([(x, y), tip], fill=A(bright, 120), width=5)
    draw_sword(d, kind, rank, (x, y), out, scale, alpha, pivot="grip", glow=0.5)
    # a short arc of light along its path
    for j in range(5):
        aa = a - (j + 1) * 0.07
        d.point((64 + math.cos(aa) * SLOT_RX, 64 + RING_Y + math.sin(aa) * SLOT_RY), fill=A(color, (160 - j * 30) * alpha / 255))
    return im


# ------------------------------------------------------------------ mastery badges: the sword crown

IRON, IRON_D = (148, 158, 172, 255), (86, 94, 110, 255)
SILVER, SILVER_D = (226, 232, 242, 255), (150, 160, 178, 255)
GOLD, GOLD_D, GOLD_L = (255, 204, 84, 255), (178, 118, 30, 255), (255, 240, 170, 255)
GEMS = ((120, 220, 255, 255), (255, 110, 170, 255), (170, 120, 255, 255), (120, 255, 190, 255))
DIGITS = {  # 3 x 5 pixel digits
    "0": ["111", "101", "101", "101", "111"], "1": ["010", "110", "010", "010", "111"],
    "2": ["111", "001", "111", "100", "111"], "3": ["111", "001", "011", "001", "111"],
    "4": ["101", "101", "111", "001", "001"], "5": ["111", "100", "111", "001", "111"],
    "6": ["111", "100", "111", "101", "111"], "7": ["111", "001", "010", "010", "010"],
    "8": ["111", "101", "111", "101", "111"], "9": ["111", "101", "111", "001", "111"],
}


def badge_blade(d, cx, cy, ang, length, metal, metal_d, light=None):
    """One small blade of the fan, from the hub at (cx, cy) outward along ang."""
    ux, uy = math.cos(ang), math.sin(ang)
    nx, ny = -uy, ux
    tip = (cx + ux * length, cy + uy * length)
    base = (cx + ux * 4, cy + uy * 4)
    w = 1.6
    pts = [(base[0] + nx * w, base[1] + ny * w), (tip[0] - ux * 3 + nx * w, tip[1] - uy * 3 + ny * w), tip,
           (tip[0] - ux * 3 - nx * w, tip[1] - uy * 3 - ny * w), (base[0] - nx * w, base[1] - ny * w)]
    d.polygon([(round(x), round(y)) for x, y in pts], fill=metal, outline=INK)
    d.line([(round(base[0]), round(base[1])), (round(tip[0] - ux * 2), round(tip[1] - uy * 2))], fill=metal_d)
    if light is not None:   # the sweep of light along this blade
        q = (base[0] + (tip[0] - base[0]) * light, base[1] + (tip[1] - base[1]) * light)
        d.point((round(q[0]), round(q[1])), fill=(255, 255, 255, 255))
        d.point((round(q[0] + nx), round(q[1] + ny)), fill=GOLD_L)


def badge_digits(d, text, cx, top, color=(255, 246, 200, 255), shadow=INK):
    w = 4 * len(text) - 1
    x0 = cx - w // 2
    for k, ch in enumerate(text):
        for r, row in enumerate(DIGITS[ch]):
            for c, bit in enumerate(row):
                if bit == "1":
                    d.point((x0 + k * 4 + c + 1, top + r + 1), fill=shadow)
                    d.point((x0 + k * 4 + c, top + r), fill=color)


def badge_frame(rank: int, phase: int, imperial: int | None = None) -> Image.Image:
    """48 x 96 (centred on him like every buff); the badge sits top right, clear of his head (x >= 28)."""
    im = Image.new("RGBA", (48, 96))
    d = ImageDraw.Draw(im)
    cx, cy = 38, 30                     # the hub, under the blades
    n = 7 if rank >= 7 else rank + 1    # one blade per rank
    metal, metal_d = ((IRON, IRON_D), (IRON, IRON_D), (SILVER, SILVER_D), (SILVER, SILVER_D),
                      (GOLD, GOLD_D), (GOLD, GOLD_D), (GOLD, GOLD_D), (GOLD, GOLD_D))[min(rank, 7)]
    # the fan must stay inside x 28..47 (the narrow upper-right area Levi's badges use too): a tight sheaf
    length = 13 if n == 1 else 9.6 + min(rank, 7) * 0.05
    spread = math.radians(min(160, 27 * (n - 1)))
    sweep = phase / 8                   # the light runs out along every blade
    for k in range(n):
        ang = -math.pi / 2 + (0 if n == 1 else -spread / 2 + spread * k / (n - 1))
        badge_blade(d, cx, cy, ang, length, metal, metal_d, light=(sweep + k * 0.12) % 1.0)
    # the hub: a shield boss that grows with rank, a gem from Swordmaster up
    hr = 3 + (rank >= 2) + (rank >= 4)
    d.ellipse((cx - hr, cy - hr + 1, cx + hr, cy + hr + 1), fill=metal_d, outline=INK)
    d.ellipse((cx - hr + 1, cy - hr + 2, cx + hr - 1, cy + hr), fill=metal)
    if rank >= 4:
        gem = GEMS[min(rank - 4, 3)] if rank < 7 else (120, 220, 255, 255)
        glow = 1 if phase in (2, 3, 4) else 0
        d.polygon([(cx, cy - 1 - glow), (cx + 2 + glow, cy + 1), (cx, cy + 3 + glow), (cx - 2 - glow, cy + 1)], fill=gem, outline=INK)
        d.point((cx - 1, cy), fill=(255, 255, 255, 255))
    if rank < 7:
        # a ribbon of pips under the hub: rank+1 notches, so the tier reads even without colour
        for k in range(rank + 1):
            x = cx - rank * 1.5 + k * 3
            d.rectangle((x - 1, cy + hr + 3, x, cy + hr + 4), fill=metal, outline=INK)
        return im
    # Imperial: a crown over the full fan, the number below; more jewels and light toward #1
    top = max(1, min(10, imperial or 10))
    prestige = 11 - top
    crown_y = cy - length - 4
    pts = [(cx - 9, crown_y + 7), (cx - 9, crown_y + 1), (cx - 5, crown_y + 4), (cx - 2, crown_y - 1), (cx, crown_y + 3),
           (cx + 2, crown_y - 1), (cx + 5, crown_y + 4), (cx + 9, crown_y + 1), (cx + 9, crown_y + 7)]
    d.polygon(pts, fill=GOLD, outline=INK)
    d.line([(cx - 8, crown_y + 6), (cx + 8, crown_y + 6)], fill=GOLD_D)
    jewels = 1 + (prestige >= 4) * 2 + (prestige >= 8) * 2
    for k in range(jewels):
        x = cx + (0, -4, 4, -7, 7)[k]
        jc = GEMS[(k + prestige) % 4] if prestige >= 8 else (255, 110, 170, 255)
        d.point((x, crown_y + 4), fill=jc)
        if prestige >= 8:
            d.point((x, crown_y + 3), fill=jc)
    # the number plate under the hub
    plate_y = cy + hr + 3
    d.rectangle((cx - 7, plate_y, cx + 7, plate_y + 8), fill=(24, 30, 58, 255), outline=GOLD if prestige < 8 else (GEMS[0] if prestige == 8 else GEMS[1] if prestige == 9 else GEMS[phase % 4]))
    badge_digits(d, str(top), cx, plate_y + 2)
    if prestige >= 10:  # #1: a big crown gem and prismatic rays
        for k in range(6):
            ang = phase * math.pi / 16 + k * math.pi / 3
            p = (cx + math.cos(ang) * 22, cy - 6 + math.sin(ang) * 22)
            if 28 <= p[0] < 48 and 0 <= p[1] < cy + hr:   # above the hub only: the number plate stays still
                d.point((round(p[0]), round(p[1])), fill=GEMS[k % 4])
        d.polygon([(cx, crown_y - 5), (cx + 2, crown_y - 3), (cx, crown_y - 1), (cx - 2, crown_y - 3)], fill=GEMS[phase % 4], outline=INK)
        d.point((cx, crown_y - 4), fill=(255, 255, 255, 255))
    return im


def badge_frames():
    result = {f"rank{r}": [badge_frame(r, p) for p in range(8)] for r in range(7)}
    result.update({f"imperial{n}": [badge_frame(7, p, n) for p in range(8)] for n in range(1, 11)})
    return result


# ------------------------------------------------------------------ effect logos (24 x 24)

FAMILY_HUE = {"damage": (255, 96, 96), "bind": (170, 140, 255), "pull": (110, 180, 255), "push": (255, 170, 90),
              "speed": (120, 240, 200), "shred": (190, 120, 255), "weaken": (200, 150, 210), "guard": (140, 200, 255),
              "attack": (255, 140, 90), "burst": (255, 220, 100), "cooldown": (120, 230, 255), "heal": (130, 255, 150),
              "domain": (255, 210, 120)}
PHASE_RING = {"planned": (140, 200, 230), "drawing": (255, 210, 110), "complete": (150, 250, 190), "cancelled": (255, 80, 110)}


def glyph(d, family: str, c):
    """A 12 x 12 pictogram at (6..18, 6..18)."""
    C = c + (255,)
    if family == "damage":
        d.line([(7, 17), (17, 7)], fill=C, width=2); d.line([(7, 7), (17, 17)], fill=C, width=2)
    elif family == "bind":
        d.ellipse((7, 7, 13, 13), outline=C); d.ellipse((11, 11, 17, 17), outline=C)
    elif family == "pull":
        for k in range(3):
            d.line([(6 + k * 3, 6 + k * 3), (12, 12)], fill=C)
        d.polygon([(12, 12), (16, 10), (16, 14)], fill=C); d.polygon([(18, 12), (14, 9), (14, 15)], fill=C)
    elif family == "push":
        d.ellipse((10, 10, 14, 14), fill=C)
        for ang in range(0, 360, 90):
            a = math.radians(ang); d.line([(12 + math.cos(a) * 3, 12 + math.sin(a) * 3), (12 + math.cos(a) * 6, 12 + math.sin(a) * 6)], fill=C)
    elif family == "speed":
        for k in range(3):
            d.line([(6 + k * 4, 8), (10 + k * 4, 12), (6 + k * 4, 16)], fill=C)
    elif family == "shred":
        for k in range(3):
            d.line([(7 + k * 4, 6), (5 + k * 4, 18)], fill=C)
    elif family == "weaken":
        d.line([(12, 6), (12, 16)], fill=C, width=2); d.polygon([(8, 13), (16, 13), (12, 18)], fill=C)
    elif family == "guard":
        d.polygon([(7, 7), (17, 7), (17, 12), (12, 18), (7, 12)], outline=C)
        d.line([(12, 9), (12, 15)], fill=C)
    elif family == "attack":
        d.line([(7, 17), (16, 8)], fill=C, width=2); d.line([(14, 6), (18, 10)], fill=C); d.line([(8, 14), (10, 16)], fill=C)
    elif family == "burst":
        for k in range(8):
            a = k * math.pi / 4; r = 6 if k % 2 == 0 else 3
            d.line([(12, 12), (12 + math.cos(a) * r, 12 + math.sin(a) * r)], fill=C)
    elif family == "cooldown":
        d.arc((7, 7, 17, 17), 40, 320, fill=C, width=2); d.polygon([(16, 5), (18, 9), (14, 9)], fill=C)
    elif family == "heal":
        d.rectangle((11, 7, 13, 17), fill=C); d.rectangle((7, 11, 17, 13), fill=C)
    else:  # domain
        d.polygon([(12, 5), (19, 12), (12, 19), (5, 12)], outline=C); d.rectangle((10, 10, 14, 14), fill=C)


def logo_frame(family: str | None, phase: str, sword: int | None = None) -> Image.Image:
    im = Image.new("RGBA", (24, 24))
    d = ImageDraw.Draw(im)
    ring = PHASE_RING[phase]
    d.ellipse((1, 1, 22, 22), fill=(16, 22, 44, 235), outline=INK)
    if phase == "planned":      # a dashed ring
        for k in range(0, 360, 45):
            d.arc((2, 2, 21, 21), k, k + 25, fill=ring + (255,))
    elif phase == "drawing":    # gold progress ticks
        for k in range(0, 360, 30):
            a = math.radians(k - 90)
            d.point((11.5 + math.cos(a) * 9.5, 11.5 + math.sin(a) * 9.5), fill=ring + (255,))
        d.arc((2, 2, 21, 21), -90, 90, fill=ring + (255,))
    else:
        d.ellipse((2, 2, 21, 21), outline=ring + (255,))
    if sword is not None:       # solo: the sword's own colour and a tiny blade
        c = COLORS[sword][1][:3]
        d.polygon([(12, 5), (14, 8), (13, 16), (12, 18), (11, 16), (10, 8)], fill=c + (255,), outline=INK)
        d.line([(8, 15), (16, 15)], fill=c + (255,))
    else:
        glyph(d, family, FAMILY_HUE[family])
    if phase == "complete":
        d.polygon([(15, 18), (17, 20), (21, 15), (20, 14), (17, 18), (16, 17)], fill=(150, 250, 190, 255))
    if phase == "cancelled":
        d.line([(5, 19), (19, 5)], fill=(255, 60, 90, 255), width=2)
    return im


# ------------------------------------------------------------------ unchanged: the auras

def aura_frame(kind: int, rank: int, phase: int, hostile: bool) -> Image.Image:
    im = Image.new("RGBA", (64, 64))
    d = ImageDraw.Draw(im)
    dark, color, bright = COLORS[kind]
    angle0 = kind * 360 / 7 + phase * (1.2 if hostile else -1.2)
    angle1 = angle0 + 360 / 7 - 5
    box = (8, 36, 56, 56)
    d.arc(box, int(angle0), int(angle1), fill=dark, width=4)
    d.arc(box, int(angle0), int(angle1), fill=color, width=2)
    t = math.radians(angle0 + 8 + (phase * 5) % 28)
    x, y = int(32 + 24 * math.cos(t)), int(46 + 10 * math.sin(t))
    d.point((x, y), fill=bright)
    d.line([(x, y - 4), (x, y)] if hostile else [(x, y), (x, y + 4)], fill=bright)
    for j in range(rank):
        a = math.radians(angle0 + 10 + j * 12 + phase * 2)
        q = (int(32 + 28 * math.cos(a)), int(46 + 13 * math.sin(a)))
        d.point(q, fill=color)
        if j >= 3:
            d.point((q[0], q[1] - 1), fill=bright)
    if rank >= 6:
        d.arc((5, 33, 59, 59), int(angle0 + phase * 2), int(angle0 + 18 + phase * 2), fill=bright)
    if rank == 7:
        d.point((x + 3, y - 3), fill=(255, 255, 255, 255))
    return im


def aura_base_frame(rank: int, phase: int, hostile: bool) -> Image.Image:
    im = Image.new("RGBA", (64, 64))
    d = ImageDraw.Draw(im)
    tone = "#806a7e" if hostile else "#66899c"
    d.arc((8, 36, 56, 56), phase * 45, phase * 45 + 240, fill=tone, width=1)
    d.arc((10, 38, 54, 54), (phase * 45 + 180) % 360, (phase * 45 + 300) % 360, fill="#a986a0" if hostile else "#9dc3cb", width=1)
    return im


def aura_field_frame(kind: int, rank: int, phase: int) -> Image.Image:
    im = Image.new("RGBA", (96, 96))
    d = ImageDraw.Draw(im)
    dark, color, bright = COLORS[kind]
    for j in range(4):
        start = (kind * 51 + phase * 11 + j * 90) % 360
        d.arc((8, 29, 88, 67), start, start + 46, fill=color if j % 2 else dark, width=2)
    d.ellipse((44, 45, 52, 51), outline=bright)
    a = phase * math.tau / 8
    x = round(48 + 40 * math.cos(a)); y = round(48 + 19 * math.sin(a))
    d.point((x, y), fill=bright)
    if rank >= 3: d.point((x, y - 2), fill=color)
    if rank >= 6: d.arc((6, 27, 90, 69), int(phase * 45), int(phase * 45 + 18), fill=bright, width=2)
    if rank == 7: d.point((48, 28), fill="#fff8e5")
    return im


# ------------------------------------------------------------------ packing and the data

def shelf_pack(anims: dict[str, list[Image.Image]], durations: dict[str, float], width: int = 2048):
    frames = [(tag, i, im) for tag, ims in anims.items() for i, im in enumerate(ims)]
    order = sorted(range(len(frames)), key=lambda k: (-frames[k][2].height, -frames[k][2].width))
    x = y = row_h = 0
    pos = {}
    for k in order:
        tag, i, im = frames[k]
        if x + im.width > width:
            x, y, row_h = 0, y + row_h + 1, 0
        pos[(tag, i)] = (x, y)
        x += im.width + 1
        row_h = max(row_h, im.height)
    sheet = Image.new("RGBA", (width, y + row_h))
    meta = {}
    for tag, ims in anims.items():
        meta[tag] = {"frames": []}
        for i, im in enumerate(ims):
            px, py = pos[(tag, i)]
            sheet.alpha_composite(im, (px, py))
            meta[tag]["frames"].append({"duration": durations[tag], "data": {"x": px, "y": py, "w": im.width, "h": im.height}})
    return sheet, meta


def save(name: str, anims: dict[str, list[Image.Image]], durations, aliases: tuple[str, ...] = ()) -> dict:
    """Write mods/tfm2_custom/vfx/<name>; for tags starting with any of `aliases`, add <tag>_frame<k> single-frame
    aliases sharing the pixels (moving world effects keep their phase without restarting)."""
    if isinstance(durations, (int, float)):
        durations = {t: durations for t in anims}
    sheet, meta = shelf_pack(anims, durations)
    for tag in list(meta):
        if tag.startswith(aliases) if aliases else False:
            for k, entry in enumerate(meta[tag]["frames"]):
                meta[f"{tag}_frame{k}"] = {"frames": [entry]}
    target = MOD / "vfx" / name
    sheet.save(str(target) + "#sheet.png", optimize=True)
    (MOD / "vfx" / f"{name}#anim.fanim").write_text(json.dumps({"anims": meta}, separators=(",", ":")), encoding="utf-8")
    shutil.copyfile(str(target) + "#sheet.png", EDITOR / f"isliid-{name}-8.png")
    return meta


def main(preview: str | None = None) -> None:
    P = "tfm2_isliid_emperor_"
    # swords in flight (projectiles)
    fly = {}
    for r in range(8):
        for k, s in enumerate(SWORDS):
            for state in ("flight", "drawing"):
                for f in range(4):
                    fly[f"{s}_rank{r}_{state}_f{f}"] = [flight_frame(k, r, f, state == "drawing")]
    fly_meta = save("swords_fly8", fly, 0.1)
    # grounded swords and the one-shot effects
    ground, dur = {}, {}
    for r in range(8):
        for k, s in enumerate(SWORDS):
            for state in ("planted", "ready"):
                tag = f"{s}_rank{r}_{state}"
                ground[tag] = [planted_frame(k, r, p, state == "ready") for p in range(8)]
                dur[tag] = 0.1
    for k, s in enumerate(SWORDS):
        for tag, frames, d_ in ((f"{s}_impact", [impact_frame(k, f) for f in range(6)], 0.05),
                                (f"{s}_launch", [launch_frame(k, f) for f in range(4)], 0.04),
                                (f"{s}_recall", [recall_frame(k, f) for f in range(5)], 0.035),
                                (f"{s}_hit", [hit_frame(k, f) for f in range(5)], 0.035)):
            ground[tag] = frames
            dur[tag] = d_
    swords = save("swords8", ground, dur, aliases=tuple(f"{s}_rank" for s in SWORDS))
    # the arsenal ring
    orbit = {}
    for r in range(8):
        for k, s in enumerate(SWORDS):
            orbit[f"ar_{s}_rank{r}"] = [orbit_frame(k, r, p, False) for p in range(8)]
            orbit[f"ar_{s}_rank{r}_sel"] = [orbit_frame(k, r, p, True) for p in range(8)]
    orbit_meta = save("orbit8", orbit, 0.1)
    # auras (unchanged look)
    aura_anims = {}
    for r in range(8):
        for side in ("ally", "enemy"):
            aura_anims[f"aura_base_rank{r}_{side}"] = [aura_base_frame(r, p, side == "enemy") for p in range(8)]
        for k in range(7):
            for side in ("ally", "enemy"):
                aura_anims[f"aura_{k}_rank{r}_{side}"] = [aura_frame(k, r, p, side == "enemy") for p in range(8)]
    auras = save("auras8", aura_anims, 0.1)
    fields = save("aura_fields8", {f"aura_field_{k}_rank{r}": [aura_field_frame(k, r, p) for p in range(8)]
                                    for r in range(8) for k in range(7)}, 0.1, aliases=("aura_field_",))
    badges = save("badges8", badge_frames(), 0.12)
    # logos
    rust = (ROOT / "native" / "tfm2_custom_ai" / "src" / "isliid.rs").read_text(encoding="utf-8")
    patterns = re.findall(r'Pattern\{name:"([^"]+)",swords:\d+,style:\d+,effect:(\d+)\}', rust)
    assert len(patterns) == 30
    logos = {}
    for fam in FAMILIES:
        for ph in PHASES:
            logos[f"logo_{fam}_{ph}"] = [logo_frame(fam, ph)]
    for k in range(7):
        for ph in PHASES:
            logos[f"logo_solo{k}_{ph}"] = [logo_frame(None, ph, k)]
    logo_meta = save("logos", logos, 0.1)

    # the data: replace every sword / orbit / badge / flag view, keep the rest
    data_path = MOD / "champion" / "tfm2_isliid_emperor.data_champion"
    data = json.loads(data_path.read_text(encoding="utf-8"))
    old_sword = tuple(P + s + "_" for s in SWORDS)
    data["view_effects"] = [v for v in data["view_effects"] if not v["name"].startswith(old_sword)
                            and not v["name"].startswith(P + "aura_") and not v["name"].startswith(P + "flag_")
                            and not v["name"].startswith(P + "logo_")]
    data["view_buffs"] = [v for v in data["view_buffs"] if not v["name"].startswith(
        ("il_ar_", "il_rank", "il_imperial", "il_aura_base_", "il_aura_visual_", "il_selected_"))]
    data["view_projectiles"] = [{"type": "Animated", "name": P + tag, "anim": "asset/tfm2_custom/vfx/swords_fly8",
                                 "tag": tag, "z": 3, "repeat": True} for tag in fly_meta]
    for tag in swords:
        if re.search(r"_rank\d_(planted|ready)$", tag):
            continue   # only the frame aliases are played (emitted every 3 ticks)
        follow = tag.endswith("_hit")
        data["view_effects"].append({"type": "Animation", "name": P + tag, "anim": "asset/tfm2_custom/vfx/swords8",
                                     "tag": tag, "z": 3, "is_follow": follow})
    for tag in orbit_meta:
        data["view_buffs"].append({"type": "Animated", "name": "il_" + tag, "anim": "asset/tfm2_custom/vfx/orbit8", "tag": tag, "z": 2})
    for tag in auras:
        name = ("il_" + tag) if tag.startswith("aura_base_") else ("il_aura_visual_" + tag[5:])
        data["view_buffs"].append({"type": "Animated", "name": name, "anim": "asset/tfm2_custom/vfx/auras8", "tag": tag, "z": -1})
    for tag in fields:
        if "_frame" in tag:
            data["view_effects"].append({"type": "Animation", "name": P + tag, "anim": "asset/tfm2_custom/vfx/aura_fields8",
                                         "tag": tag, "z": -2, "is_follow": False})
    for tag in badges:
        data["view_buffs"].append({"type": "Animated", "name": "il_" + tag, "anim": "asset/tfm2_custom/vfx/badges8", "tag": tag, "z": 4})
    for tag in logo_meta:
        data["view_effects"].append({"type": "Animation", "name": P + tag, "anim": "asset/tfm2_custom/vfx/logos",
                                     "tag": tag, "z": 5, "is_follow": False})
    data_path.write_text(json.dumps(data, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    manifest = {"fly": fly_meta, "swords": swords, "orbit": orbit_meta, "auras": auras, "fields": fields,
                "badges": badges, "logos": logo_meta,
                "patterns": [{"name": n, "family": FAMILIES[int(e)]} for n, e in patterns]}
    (EDITOR / "isliid-art-manifest.json").write_text(json.dumps(manifest, separators=(",", ":")), encoding="utf-8")
    for stale in ("flags", "selector", "swords", "orbit", "badges"):
        for ext in ("#sheet.png", "#anim.fanim"):
            p = MOD / "vfx" / f"{stale}{ext}"
            if p.exists():
                p.unlink()
    for stale in ("isliid-flags-8.png",):
        p = EDITOR / stale
        if p.exists():
            p.unlink()
    print(f"Generated {len(fly_meta)} flight, {len(swords)} ground, {len(orbit_meta)} orbit, {len(auras)} aura, "
          f"{len(fields)} field, {len(badges)} badge, {len(logo_meta)} logo animations; "
          f"{len(data['view_projectiles'])} projectile, {len(data['view_effects'])} effect, {len(data['view_buffs'])} buff views")
    if preview:
        previews(Path(preview))


def previews(folder: Path) -> None:
    folder.mkdir(parents=True, exist_ok=True)
    bg = (40, 52, 46, 255)
    # badges: every rank and Imperial, frame 0 and an animated strip
    cells = [badge_frame(r, 0) for r in range(7)] + [badge_frame(7, 0, n) for n in range(10, 0, -1)]
    sheet = Image.new("RGBA", (24 * len(cells), 56), bg)
    for i, c in enumerate(cells):
        sheet.alpha_composite(c.crop((24, 0, 48, 56)), (i * 24, 0))
    sheet.resize((sheet.width * 5, sheet.height * 5), Image.Resampling.NEAREST).save(folder / "isliid_badges.png")
    gif = []
    for p in range(8):
        im = Image.new("RGBA", (24 * 4, 56), bg)
        for i, (r, n) in enumerate(((3, None), (6, None), (7, 4), (7, 1))):
            im.alpha_composite(badge_frame(r, p, n).crop((24, 0, 48, 56)), (i * 24, 0))
        gif.append(im.resize((im.width * 5, im.height * 5), Image.Resampling.NEAREST).convert("RGB"))
    gif[0].save(folder / "isliid_badges.gif", save_all=True, append_images=gif[1:], duration=120, loop=0)
    # swords: each sword at ranks 0 / 3 / 5 / 7, planted
    sh = Image.new("RGBA", (40 * 7 * 4 // 2, 96 * 2), bg)
    for k in range(7):
        for j, r in enumerate((0, 3, 5, 7)):
            sh.alpha_composite(planted_frame(k, r, 0, False), ((k * 2 + j % 2) * 40 // 1, (j // 2) * 96))
    sh.resize((sh.width * 3, sh.height * 3), Image.Resampling.NEAREST).save(folder / "isliid_swords.png")
    # flight, impact, hit, recall, orbit ring and logos
    strip = Image.new("RGBA", (72 * 2 + 56 + 40 * 2 + 128, 128), bg)
    frames = []
    for f in range(8):
        im = strip.copy()
        im.alpha_composite(flight_frame(3, 5, f % 4, False), (0, 10))
        im.alpha_composite(flight_frame(6, 7, f % 4, True), (0, 40))
        im.alpha_composite(flight_frame(2, 2, f % 4, False), (72, 10))
        im.alpha_composite(flight_frame(4, 6, f % 4, True), (72, 40))
        im.alpha_composite(impact_frame(1, min(f, 5)), (144, 10))
        im.alpha_composite(hit_frame(5, min(f, 4)), (200, 10))
        im.alpha_composite(recall_frame(0, min(f, 4)), (240, 10))
        ring = Image.new("RGBA", (128, 128))
        for k in range(7):
            ring.alpha_composite(orbit_frame(k, 5, f, k == 3))
        im.alpha_composite(ring, (280, 0))
        frames.append(im.resize((im.width * 3, im.height * 3), Image.Resampling.NEAREST).convert("RGB"))
    frames[0].save(folder / "isliid_sword_kit.gif", save_all=True, append_images=frames[1:], duration=100, loop=0)
    lg = Image.new("RGBA", (26 * 4, 26 * (len(FAMILIES) + 7)), bg)
    for i, fam in enumerate(list(FAMILIES) + [None] * 7):
        for j, ph in enumerate(PHASES):
            lg.alpha_composite(logo_frame(fam, ph, None if fam else i - len(FAMILIES)), (j * 26, i * 26))
    lg.resize((lg.width * 3, lg.height * 3), Image.Resampling.NEAREST).save(folder / "isliid_logos.png")


if __name__ == "__main__":
    main(sys.argv[sys.argv.index("--preview") + 1] if "--preview" in sys.argv else None)
