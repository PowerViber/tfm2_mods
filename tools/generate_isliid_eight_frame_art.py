"""Isliid's combat art, native pixels (round 88: the sword-kit step-up, the sword-crown badges, the effect logos).

The consolidated tfm2_custom mod is the source of truth; this writes its sheets, the views in Isliid's data and the
editor's manifest. Run from the repo root: python tools/generate_isliid_eight_frame_art.py [--preview DIR]

Swords (one identity each, colours = the engraving scar colours, a silhouette per sword and per rank tier):
  0 Skylight needle, 1 Terra slab, 2 Darkbringer serrated, 3 Gale curved saber, 4 Blood hooked, 5 Rift split prongs,
  6 Emperor command blade. Rank tiers: the guard grows from a bar (Bearer) to swept quillons, wings and a crown guard
  (Imperial); the blade lengthens, gains a rune fuller, an energy edge, and at Imperial a halo.

Sheets:
  swords_comet  flying swords as point effects (round 95: natively spawned projectile art doesn't render in the game;
                round 96: Spirit comets, Imperial a little solar system, tools/isliid_comet_art.py):
                <sword>_rank<r>_comet_a<h> (r 0..7, 8 = Imperial #1; 8 headings; 8 frames), played as single-frame
                _frame<k> and 2-frame _pair<k> aliases at the sword's position
  blackhole     Imperial's black hole above his head (buffs il_blackhole[1]_n<k>, k swords inside; round 97)
  wormhole      the wormholes Imperial's swords leave and arrive through (effects wormhole_out|in_<sword>|p)
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

from PIL import Image, ImageDraw, ImageFilter

sys.path.insert(0, str(Path(__file__).resolve().parent))
import isliid_comet_art as comet  # noqa: E402  (round 96: the flight forms)

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
# round 89: swords grow with mastery (Bearer 1x .. Imperial 1.6x); the engraving art comes in the swords' four tiers
SCALE = (1.0, 1.05, 1.1, 1.18, 1.26, 1.36, 1.48, 1.6)
TIER = (0, 0, 1, 1, 2, 2, 3, 3)
PLANTED_FRAMES = 12      # isliid.rs PLANTED_FRAMES
BADGE_FRAMES = 16
POP_FRAMES = 6           # isliid.rs POP_FRAMES
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


# ------------------------------------------------------------------ round 89: the high-rank sword effects

def sword_point(kind: int, rank: int, t: float, across: float, origin, ang, scale, pivot):
    """A point on the blade in image space: t 0 = the guard .. 1 = the tip, `across` in sword-space pixels."""
    L, _ = sword_parts(kind, rank)
    b0 = 10
    cx = {"mid": L / 2, "tip": L, "grip": 5}[pivot]
    return transform([(b0 + (L - b0) * t, across)], origin, ang, scale, cx)[0]


def sword_aura(im: Image.Image, kind: int, rank: int, origin, ang, scale, pivot, strength: float):
    """A flickering energy silhouette round the blade (Regent and up): the sword's mask, grown and tinted."""
    layer = Image.new("RGBA", im.size)
    draw_sword(ImageDraw.Draw(layer, "RGBA"), kind, rank, origin, ang, scale, pivot=pivot)
    mask = layer.getchannel("A").point(lambda a: 255 if a else 0)
    grown = mask.filter(ImageFilter.MaxFilter(5 if rank < 7 else 7))
    halo = grown.filter(ImageFilter.GaussianBlur(1.2))
    color = COLORS[kind][1] if rank < 7 else COLORS[kind][2]
    tint = Image.new("RGBA", im.size, color[:3] + (0,))
    tint.putalpha(halo.point(lambda a: int(a * strength)))
    im.alpha_composite(tint)


def shards(d, kind: int, rank: int, centre, rx, ry, phase: float, front: bool):
    """3-5 small crystal shards circling the sword (Regent and up); only the near (front) or far half is drawn."""
    n = (0, 0, 0, 0, 0, 3, 4, 5)[rank]
    dark, color, bright = COLORS[kind]
    for j in range(n):
        a = phase * math.tau + j * math.tau / n
        if (math.sin(a) > 0) != front:
            continue
        x, y = centre[0] + math.cos(a) * rx, centre[1] + math.sin(a) * ry
        r = 1.6 if front else 1.1
        d.polygon([(x, y - r - 1), (x + r, y), (x, y + r + 1), (x - r, y)], fill=A(bright if front else color, 255 if front else 170),
                  outline=A(INK, 200 if front else 90))


def crackle(d, start, ang, length, seed: int, color, alpha=230):
    """A short jagged lightning bolt from `start` along `ang`."""
    x, y = start
    pts = [(x, y)]
    steps = 4
    for k in range(1, steps + 1):
        j = ((seed * 37 + k * 17) % 7 - 3) * 0.9
        x = start[0] + math.cos(ang) * length * k / steps - math.sin(ang) * j
        y = start[1] + math.sin(ang) * length * k / steps + math.cos(ang) * j
        pts.append((x, y))
    d.line(pts, fill=A(color, alpha), width=1)


def blade_fx(im: Image.Image, kind: int, rank: int, origin, ang, scale, pivot, phase: float):
    """The animated layers on top of a drawn sword: a shimmer running up the edge (Swordmaster+), runes lighting in
    turn, lightning on the guard (Regent+) and a crown flare on the guard (Imperial). `phase` is 0..1 over the loop."""
    if rank < 4:
        return
    d = ImageDraw.Draw(im, "RGBA")
    dark, color, bright = COLORS[kind]
    # the shimmer: a white bar crossing the blade, running guard -> tip
    t = 0.08 + 0.88 * phase
    a = sword_point(kind, rank, t, -3.2, origin, ang, scale, pivot)
    b = sword_point(kind, rank, t, 3.2, origin, ang, scale, pivot)
    d.line([a, b], fill=(255, 255, 255, 235), width=max(1, round(scale)))
    c0 = sword_point(kind, rank, max(0.0, t - 0.06), 0, origin, ang, scale, pivot)
    d.point(c0, fill=A(bright, 220))
    # runes lighting one after another
    for k in range(4):
        lit = int(phase * 4) % 4 == k
        q = sword_point(kind, rank, 0.2 + k * 0.17, 0, origin, ang, scale, pivot)
        d.rectangle((q[0] - 0.6, q[1] - 0.6, q[0] + 0.6, q[1] + 0.6), fill=A((255, 255, 255, 255) if lit else bright, 255 if lit else 150))
    if rank >= 5:
        g = sword_point(kind, rank, -0.02, 0, origin, ang, scale, pivot)
        seed = int(phase * 16) + kind * 3
        if int(phase * 8) % 2 == 0:
            crackle(d, g, ang + math.pi / 2 + (seed % 3 - 1) * 0.4, 7 * scale, seed, bright)
        else:
            crackle(d, g, ang - math.pi / 2 + (seed % 3 - 1) * 0.4, 7 * scale, seed + 1, color)
    if rank >= 7:
        g = sword_point(kind, rank, -0.04, 0, origin, ang, scale, pivot)
        r = 2 + 2.5 * (0.5 + 0.5 * math.sin(phase * math.tau * 2))
        d.line([(g[0] - r, g[1]), (g[0] + r, g[1])], fill=(255, 250, 220, 255))
        d.line([(g[0], g[1] - r), (g[0], g[1] + r)], fill=(255, 250, 220, 255))


def halo_ring(d, centre, rx, ry, phase: float, color, bright, dots=8):
    """Imperial: a rotating ring of light (an ellipse with travelling beads)."""
    d.ellipse((centre[0] - rx, centre[1] - ry, centre[0] + rx, centre[1] + ry), outline=A(color, 170))
    for j in range(dots):
        a = phase * math.tau + j * math.tau / dots
        d.point((centre[0] + math.cos(a) * rx, centre[1] + math.sin(a) * ry), fill=A(bright, 255))


# ------------------------------------------------------------------ flying swords (projectiles, tip right)

def flight_frame(kind: int, rank: int, k: int, drawing: bool) -> Image.Image:
    sc = SCALE[rank]
    L = sword_parts(kind, rank)[0] * sc
    front = 10 * sc + L / 2 + 8
    W = round(2 * front)
    H = round(28 * sc) + (10 if rank >= 5 else 4)
    im = Image.new("RGBA", (W, H))
    d = ImageDraw.Draw(im, "RGBA")
    dark, color, bright = COLORS[kind]
    cx, cy = W / 2 + 10 * sc, H / 2          # the projectile position: the sword's centre, a little forward
    phase = k / 4
    # the wake: a tapered ribbon behind the sword in its colour (brighter and longer while engraving, wider with rank)
    length = (30 + (14 if drawing else 0)) * (0.8 + 0.2 * sc) + 3 * math.sin(k * math.pi / 2)
    tail_x = cx - 17 * sc
    width = (3.2 if drawing else 2.4) * (0.7 + 0.3 * sc)
    for j in range(int(length)):
        t = j / length
        half = width * (1 - t) ** 0.8 + 0.3
        wob = math.sin(j * 0.5 + k * 1.6) * 0.6 * t
        a = 220 * (1 - t) ** 1.2
        x = tail_x - j
        if x < 0:
            break
        d.line([(x, cy - half + wob), (x, cy + half + wob)], fill=A(color, a))
        if j % 2 == 0:
            d.point((x, cy + wob), fill=A(bright, a))
    # speed lines
    for j in range(3 + (rank >= 5) * 2):
        y = cy + (-6, 5, -2, 8, -9)[j] * (0.7 + 0.3 * sc) + (k + j) % 2
        x0 = tail_x - 6 - ((k * 5 + j * 9) % 18)
        d.line([(x0, y), (max(0, x0 - 6 - j * 2), y)], fill=A(bright, 150))
    if drawing:   # sparks thrown off the engraving
        for j in range(4 + rank // 2):
            x = tail_x - ((k * 7 + j * 11) % 30)
            y = cy + ((-1) ** j) * (3 + (j + k) % 3) * (0.8 + 0.2 * sc)
            d.point((x, y), fill=A((255, 255, 255, 255), 230))
    # Sovereign leaves one afterimage, Imperial two
    for n in range((rank >= 6) + (rank >= 7)):
        ghost = Image.new("RGBA", im.size)
        draw_sword(ImageDraw.Draw(ghost, "RGBA"), kind, rank, (cx - (11 + 11 * n) * sc, cy), 0.0, sc, 110 - 45 * n)
        im.alpha_composite(ghost)
    if rank >= 5:
        sword_aura(im, kind, rank, (cx, cy), 0.0, sc, "mid", 0.45 + 0.35 * (k % 2))
        shards(d, kind, rank, (cx, cy), L * 0.45, 5 * sc, phase, front=False)
    draw_sword(d, kind, rank, (cx, cy), 0.0, sc, glow=k / 3)
    blade_fx(im, kind, rank, (cx, cy), 0.0, sc, "mid", phase)
    if rank >= 5:
        shards(d, kind, rank, (cx, cy), L * 0.45, 5 * sc, phase, front=True)
    if rank == 7:
        halo_ring(d, (cx - L / 2 + 8 * sc, cy), 3 * sc, 7 * sc, phase, color, bright, dots=4)
    return im


# ------------------------------------------------------------------ grounded swords and one-shot effects

def planted_frame(kind: int, rank: int, phase: int, ready: bool) -> Image.Image:
    """The sword stuck in the ground, tip at the image centre (= its point), hilt up; the ground cracked around it."""
    sc = SCALE[rank]
    L = sword_parts(kind, rank)[0] * sc
    W = round(40 * sc) + (12 if rank >= 5 else 0)
    H = round(2 * (L + 16))
    W += W % 2
    im = Image.new("RGBA", (W, H))
    d = ImageDraw.Draw(im, "RGBA")
    dark, color, bright = COLORS[kind]
    cx, cy = W / 2, H / 2
    ph = phase / PLANTED_FRAMES
    pulse = 0.5 + 0.5 * math.sin(ph * math.tau)
    gr = 15 * sc
    # the ground: a crack and a glow pool (a rune circle when armed; Regent and up a turning rune circle always)
    if ready or rank >= 5:
        d.ellipse((cx - gr, cy - 4 * sc, cx + gr, cy + 6 * sc), outline=A(color, 140 + 100 * pulse), width=1)
        runes = 6 + 2 * TIER[rank]
        for k in range(runes):
            a = k * math.tau / runes + ph * math.tau / (2 if ready else 4)
            d.point((cx + math.cos(a) * gr, cy + 1 + math.sin(a) * 5 * sc), fill=A(bright, 255))
        d.ellipse((cx - 9 * sc, cy - 2, cx + 9 * sc, cy + 4), outline=A(bright, 90 + 120 * pulse))
        if rank >= 7:   # an outer counter-turning ring
            for k in range(10):
                a = k * math.tau / 10 - ph * math.tau / 2
                d.point((cx + math.cos(a) * (gr + 4), cy + 1 + math.sin(a) * (5 * sc + 2)), fill=A(color, 220))
    if not ready:
        d.ellipse((cx - 9 * sc, cy - 2, cx + 9 * sc, cy + 4), fill=A(color, 40 + 40 * pulse + 20 * TIER[rank]))
    for a, l in ((200, 8), (330, 7), (20, 6), (150, 5)):
        r = math.radians(a)
        d.line([(cx, cy + 1), (cx + math.cos(r) * l * sc, cy + 1 + math.sin(r) * l * 0.45 * sc)], fill=A((40, 30, 24, 255), 200))
    mid = (cx, cy + 3 - L / 2)
    if rank >= 5:
        sword_aura(im, kind, rank, (cx, cy + 3), math.pi / 2, sc, "tip", 0.35 + 0.4 * pulse)
        shards(d, kind, rank, mid, 9 * sc, 4 * sc, ph, front=False)
    if rank >= 7:
        halo_ring(d, (cx, cy + 3 - L + 6 * sc), 8 * sc, 3 * sc, ph, color, bright)
    # the sword, tip slightly in the ground
    draw_sword(d, kind, rank, (cx, cy + 3), math.pi / 2, sc, pivot="tip", glow=pulse)
    blade_fx(im, kind, rank, (cx, cy + 3), math.pi / 2, sc, "tip", ph)
    if rank >= 5:
        shards(d, kind, rank, mid, 9 * sc, 4 * sc, ph, front=True)
    # motes rising, more and higher with rank
    rise = 34 * sc
    for j in range(1 + rank // 2 + (rank >= 5) * 2):
        y = cy - ((phase * rise / PLANTED_FRAMES * 2 + j * 11) % rise)
        x = cx + ((j * 7 + phase) % 9 - 4) * sc
        d.point((x, y), fill=A(bright if j % 2 else color, 120 + 120 * (1 - (cy - y) / rise)))
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
    grow = 1 + (SCALE[rank] - 1) * 0.7          # round 89: the ring's blades grow with rank too (x1.42 at Imperial)
    scale = (0.46 if back else 0.56) * grow
    alpha = 165 if back else 255
    out = math.atan2(math.sin(a) * SLOT_RY / SLOT_RX * 1.6, math.cos(a))   # outward on the flattened ring
    if selected:
        tip = (x + math.cos(out) * 20 * grow, y + math.sin(out) * 20 * grow)
        d.line([(x, y), tip], fill=A(bright, 120), width=5)
    if rank >= 5 and not back:
        sword_aura(im, kind, rank, (x, y), out, scale, "grip", 0.3 + 0.3 * (phase % 2))
    draw_sword(d, kind, rank, (x, y), out, scale, alpha, pivot="grip", glow=0.5)
    if not back:
        blade_fx(im, kind, rank, (x, y), out, scale, "grip", ((phase / 8) + kind / 7) % 1.0)
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


BADGE_W, BADGE_H = 72, 96          # centred on him like every buff
SIGIL = (BADGE_W // 2 + 19, BADGE_H // 2 - 19)   # top right of his head
SIGIL_R = 7                # the medallion; the blade wheel reaches SIGIL_R + 9


def sigil_blade(d, cx, cy, ang, r0, r1, metal, metal_d):
    """A small blade of the sigil's wheel: from radius r0 to r1 along `ang`, tip outward."""
    ux, uy = math.cos(ang), math.sin(ang)
    nx, ny = -uy, ux
    base = (cx + ux * r0, cy + uy * r0)
    tip = (cx + ux * r1, cy + uy * r1)
    w = 1.5
    pts = [(base[0] + nx * w, base[1] + ny * w), (tip[0] - ux * 2 + nx * w, tip[1] - uy * 2 + ny * w), tip,
           (tip[0] - ux * 2 - nx * w, tip[1] - uy * 2 - ny * w), (base[0] - nx * w, base[1] - ny * w)]
    d.polygon(pts, fill=metal, outline=INK)
    d.line([(base[0] - nx * 2.6, base[1] - ny * 2.6), (base[0] + nx * 2.6, base[1] + ny * 2.6)], fill=INK, width=2)   # the guard
    d.line([(base[0] + ux * 1.5, base[1] + uy * 1.5), (tip[0] - ux * 2.5, tip[1] - uy * 2.5)], fill=(255, 255, 255, 140))


def badge_frame(rank: int, phase: int, imperial: int | None = None) -> Image.Image:
    """Round 89: the mastery sigil, BADGE_FRAMES frames that loop seamlessly. A rune ring turns round a pulsing core
    gem; one small blade per rank wheels round the core (faster with rank); sparks shed off the ring. Iron -> silver ->
    gold -> gemmed. Imperial: a bobbing crown, turning prismatic rays, the #number plate, lightning between the blades
    from #3."""
    im = Image.new("RGBA", (BADGE_W, BADGE_H))
    d = ImageDraw.Draw(im, "RGBA")
    cx, cy = SIGIL
    R = SIGIL_R
    rank = min(rank, 7)
    t = phase / BADGE_FRAMES                     # 0..1 over the loop
    n = 7 if rank >= 7 else rank + 1             # blades
    metal, metal_d = ((IRON, IRON_D), (IRON, IRON_D), (SILVER, SILVER_D), (SILVER, SILVER_D),
                      (GOLD, GOLD_D), (GOLD, GOLD_D), (GOLD, GOLD_D), (GOLD, GOLD_D))[rank]
    gem = (GEMS[0], GEMS[0], GEMS[0], GEMS[0], GEMS[0], GEMS[1], GEMS[2], (120, 220, 255, 255))[rank]
    top = max(1, min(10, imperial or 10))
    prestige = 11 - top if rank == 7 else 0
    if rank == 7 and prestige >= 10:
        gem = GEMS[phase // 4 % 4]
    pulse = 0.5 + 0.5 * math.sin(t * math.tau * 2)
    # Imperial: prismatic rays turning behind everything (one ray-step per loop, so it loops)
    if rank == 7:
        rays = 8
        for k in range(rays):
            a = t * math.tau / rays * (1 + prestige // 4) + k * math.tau / rays
            col = GEMS[k % 4] if prestige >= 8 else GOLD_L
            p1 = (cx + math.cos(a) * (R + 4), cy + math.sin(a) * (R + 4))
            p2 = (cx + math.cos(a) * (R + 10 + 2 * pulse), cy + math.sin(a) * (R + 10 + 2 * pulse))
            d.line([p1, p2], fill=A(col, 150 + 80 * pulse))
    # the back glow and the disc
    glow = (metal[:3] if rank < 4 else gem[:3])
    G = R + 4 + 2 * pulse
    d.ellipse((cx - G, cy - G, cx + G, cy + G), fill=glow + (int(30 + 40 * pulse) if rank >= 2 else 0,))
    d.ellipse((cx - R, cy - R, cx + R, cy + R), fill=(18, 24, 50, 245), outline=INK)
    d.ellipse((cx - R + 1, cy - R + 1, cx + R - 1, cy + R - 1), outline=metal_d)
    d.ellipse((cx - R + 2, cy - R + 2, cx + R - 2, cy + R - 2), outline=metal)
    # the rune ring: notches travelling round (bright ones every third), one notch-step per loop at Bearer, more higher
    notches = 8
    turn = (1 + rank // 2) * math.tau / notches
    for k in range(notches):
        a = t * turn + k * math.tau / notches
        x, y = cx + math.cos(a) * (R - 1.5), cy + math.sin(a) * (R - 1.5)
        bright = k % 3 == 0
        d.point((round(x), round(y)), fill=(255, 255, 255, 255) if bright and rank >= 2 else metal_d if not bright else metal)
    # the blades wheeling round the core: a whole number of blade-steps per loop (seamless), faster with rank
    spin = (1 + rank // 3) * math.tau / n
    for k in range(n):
        a = -math.pi / 2 + t * spin + k * math.tau / n
        sigil_blade(d, cx, cy, a, R + 1, R + 9, metal, metal_d)
    # Imperial #3..#1: lightning arcs jumping between neighbouring blade tips
    if rank == 7 and prestige >= 8:
        for k in range(n):
            if (k + phase) % (4 if prestige == 8 else 3 if prestige == 9 else 2):
                continue
            a0 = -math.pi / 2 + t * spin + k * math.tau / n
            a1 = a0 + math.tau / n
            p0 = (cx + math.cos(a0) * (R + 7), cy + math.sin(a0) * (R + 7))
            p1 = (cx + math.cos(a1) * (R + 7), cy + math.sin(a1) * (R + 7))
            m = ((p0[0] + p1[0]) / 2 + (phase % 3 - 1), (p0[1] + p1[1]) / 2 - (phase % 2))
            d.line([p0, m, p1], fill=A(GEMS[(k + phase) % 4] if prestige >= 10 else (200, 240, 255, 255), 255))
    # the core: a boss that pulses, a gem from Swordmaster
    hr = 2 + (rank >= 4) * 0.5 + 0.8 * pulse
    d.ellipse((cx - hr, cy - hr, cx + hr, cy + hr), fill=metal_d, outline=INK)
    if rank >= 4:
        g = 1.6 + pulse
        d.polygon([(cx, cy - g - 0.5), (cx + g, cy), (cx, cy + g + 0.5), (cx - g, cy)], fill=gem, outline=INK)
        if phase % 8 < 2:
            d.point((cx - 1, cy - 1), fill=(255, 255, 255, 255))
    else:
        d.ellipse((cx - hr + 1, cy - hr + 1, cx + hr - 1, cy + hr - 1), fill=metal)
    # sparks shedding off the ring (more with rank), each living a quarter of the loop
    for k in range(1 + rank):
        life = ((t * 4 + k * 0.37) % 1.0)
        a = k * 2.399 + math.floor(t * 4 + k * 0.37) * 1.3
        r = R + 9 + life * 5
        x, y = cx + math.cos(a) * r, cy + math.sin(a) * r - life * 2
        if 0 <= x < BADGE_W and 0 <= y < BADGE_H:
            d.point((round(x), round(y)), fill=A(gem if rank >= 4 else metal, 255 * (1 - life)))
    if rank < 7:
        # a ribbon of pips under the sigil: rank+1 notches, so the tier reads even without colour
        for k in range(rank + 1):
            x = cx - rank * 1.5 + k * 3
            lit = k == phase % (rank + 1) and rank >= 2
            d.rectangle((x - 1, cy + R + 11, x, cy + R + 12), fill=(255, 255, 255, 255) if lit else metal, outline=INK)
        return im
    # Imperial: a crown bobbing over the sigil, the number below
    bob = round(math.sin(t * math.tau) * 1.5)
    cy0 = cy - R - 16 + bob
    pts = [(cx - 8, cy0 + 6), (cx - 8, cy0 + 1), (cx - 4, cy0 + 4), (cx - 2, cy0 - 1), (cx, cy0 + 3),
           (cx + 2, cy0 - 1), (cx + 4, cy0 + 4), (cx + 8, cy0 + 1), (cx + 8, cy0 + 6)]
    d.polygon(pts, fill=GOLD, outline=INK)
    d.line([(cx - 7, cy0 + 5), (cx + 7, cy0 + 5)], fill=GOLD_D)
    jewels = 1 + (prestige >= 4) * 2 + (prestige >= 8) * 2
    for k in range(jewels):
        x = cx + (0, -4, 4, -6, 6)[k]
        jc = GEMS[(k + phase // 2) % 4] if prestige >= 8 else (255, 110, 170, 255)
        d.point((x, cy0 + 3), fill=jc)
    # the crown's glint runs left to right once a loop
    gx = cx - 8 + round(t * 16)
    if cx - 8 <= gx <= cx + 8:
        d.point((gx, cy0 + 1), fill=(255, 255, 255, 255))
    if prestige >= 10:
        d.polygon([(cx, cy0 - 6), (cx + 2, cy0 - 4), (cx, cy0 - 2), (cx - 2, cy0 - 4)], fill=GEMS[phase // 2 % 4], outline=INK)
    plate_y = cy + R + 11
    rim = GOLD if prestige < 8 else (GEMS[0] if prestige == 8 else GEMS[1] if prestige == 9 else GEMS[phase // 2 % 4])
    d.rectangle((cx - 7, plate_y, cx + 7, plate_y + 8), fill=(24, 30, 58, 255), outline=A(rim, 255 if pulse > 0.3 or prestige < 8 else 170))
    badge_digits(d, str(top), cx, plate_y + 2)
    return im


def badge_frames():
    result = {f"rank{r}": [badge_frame(r, p) for p in range(BADGE_FRAMES)] for r in range(7)}
    result.update({f"imperial{n}": [badge_frame(7, p, n) for p in range(BADGE_FRAMES)] for n in range(1, 11)})
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


def logo_pop_frame(family: str | None, f: int, sword: int | None = None) -> Image.Image:
    """Round 89: a completed logo pops: a light ring bursts out of it and the logo flashes, then settles (32 x 32, the
    24 x 24 logo centred)."""
    im = Image.new("RGBA", (32, 32))
    d = ImageDraw.Draw(im, "RGBA")
    t = f / (POP_FRAMES - 1)
    hue = (COLORS[sword][1][:3] if sword is not None else FAMILY_HUE[family])
    r = 9 + t * 7
    d.ellipse((16 - r, 16 - r, 16 + r, 16 + r), outline=hue + (int(255 * (1 - t)),), width=2 if t < 0.5 else 1)
    for k in range(8):
        a = k * math.pi / 4 + t
        q = (16 + math.cos(a) * (r + 1), 16 + math.sin(a) * (r + 1))
        d.point(q, fill=(255, 255, 255, int(255 * (1 - t))))
    im.alpha_composite(logo_frame(family, "complete", sword), (4, 4))
    if f < 2:   # the flash
        flash = Image.new("RGBA", (32, 32))
        ImageDraw.Draw(flash).ellipse((5, 5, 26, 26), fill=(255, 255, 255, 150 - 70 * f))
        im.alpha_composite(flash)
    return im


# ------------------------------------------------------------------ round 89: engravings that visibly fire, by tier

# the world is about 900 units per pixel (playtest notes: 750-950)
UPX = 900


def ground_ring(d, c, r, squash, color, alpha, width=1, dash=0, turn=0.0, ticks=0, bright=None):
    """A ring on the ground (an ellipse squashed to the floor); optional dashes, rune ticks turning by `turn` radians."""
    box = (c[0] - r, c[1] - r * squash, c[0] + r, c[1] + r * squash)
    if dash:
        step = 360 / dash
        for k in range(dash):
            a0 = k * step + math.degrees(turn)
            d.arc(box, a0, a0 + step * 0.55, fill=A(color, alpha), width=width)
    else:
        d.ellipse(box, outline=A(color, alpha), width=width)
    for k in range(ticks):
        a = turn + k * math.tau / ticks
        x, y = c[0] + math.cos(a) * r, c[1] + math.sin(a) * r * squash
        d.rectangle((x - 1, y - 1, x + 1, y + 1), fill=A(bright or color, alpha))


SCAR_STEP = 30_000        # isliid.rs SCAR_STEP: one sprite per this much stroke, centred on its piece
SCAR_PHASES = 4            # x 3 ticks = SCAR_HOT_EVERY (12): the shimmer loops exactly once per emission
COOL_SECONDS = 1.0         # isliid.rs SCAR_COOL_EVERY (60 ticks)


def scar_frame(kind: int, tier: int, angle: int, phase: int, lit: bool, cool: bool = False) -> Image.Image:
    """One piece of an engraved stroke at one of 16 angles (isliid.rs trail_angle: 0..pi in 16 steps), covering
    SCAR_STEP of it with a little overlap. t0 a thin cut, t1 a glowing groove, t2 runes flicker along it, t3 a luminous
    channel with crackling energy; a light runs along it over the 4 phases. Lit (its formation just fired): white-hot,
    wider and glowing. Cool (10 s old): the settled groove, one still frame, no glow."""
    h = SCAR_STEP / UPX / 2 + 2 + tier * 0.5            # half length in pixels
    S = int(2 * h + 8 + (4 if tier >= 3 else 0) + (6 if lit else 0))
    im = Image.new("RGBA", (S, S))
    d = ImageDraw.Draw(im, "RGBA")
    dark, color, bright = COLORS[kind]
    if cool:   # the colour settles toward the dark tone
        color = tuple(int(color[k] * 0.6 + dark[k] * 0.4) for k in range(3)) + (255,)
        bright = tuple(int(bright[k] * 0.55 + dark[k] * 0.45) for k in range(3)) + (255,)
    c = (S - 1) / 2
    ang = math.pi * angle / 16
    ux, uy = math.cos(ang), math.sin(ang)
    nx, ny = -uy, ux
    P = lambda k, j=0.0: (c + ux * k + nx * j, c + uy * k + ny * j)
    if lit or (tier >= 3 and not cool):
        glow = Image.new("RGBA", (S, S))
        g = ImageDraw.Draw(glow, "RGBA")
        g.line([P(-h), P(h)], fill=A(bright if lit else color, 150 if lit else 70), width=(9 + 2 * tier) if lit else 7)
        im.alpha_composite(glow.filter(ImageFilter.GaussianBlur(1.6 if lit else 1.0)))
    widths = ((3, 1, 0), (5, 3, 1), (5, 3, 1), (6, 4, 2))[tier]
    d.line([P(-h), P(h)], fill=A(INK, 200 if cool else 230), width=widths[0])
    d.line([P(-h), P(h)], fill=A(color, 210 if cool else 245), width=widths[1])
    if (widths[2] and not cool) or lit:
        core = (255, 255, 255, 255) if lit else (bright if phase % 2 else color)
        d.line([P(-h + 3), P(h - 3)], fill=core, width=max(1, widths[2] + (1 if lit else 0)))
    if not cool:   # the light running along the stroke over the loop
        run = -h + 3 + (phase + 0.5) * (2 * h - 6) / SCAR_PHASES
        d.line([P(run - 2), P(run + 2)], fill=A((255, 255, 255, 255), 230 if tier >= 1 or lit else 170), width=1)
    if tier >= 2:   # runes: small cross-ticks along the stroke, lighting in turn (still and dim when cool)
        for n, k in enumerate((-h * 0.6, 0.0, h * 0.6)):
            on = cool or lit or (n + phase) % 3 != 0
            if on:
                col = bright if not lit else (255, 255, 255, 255)
                d.line([P(k, -2.5), P(k, 2.5)], fill=A(col, 150 if cool else 255))
                if not cool:
                    d.point(P(k + 1, 2.5 if n % 2 else -2.5), fill=A(bright, 255))
    if tier >= 3 and not cool:   # crackling energy and sparks
        pts = [P(-h + 2 + k * (2 * h - 4) / 8, ((k + phase) % 3 - 1) * 2.4) for k in range(9)]
        d.line(pts, fill=A((255, 255, 255, 255) if lit else bright, 230))
        for k in range(4):
            q = P(-h + 4 + ((k * 9 + phase * 5) % int(2 * h - 8)), (-4 if k % 2 else 4) - phase % 2)
            d.point(q, fill=A(bright, 255))
    if lit:   # sparkles thrown off
        for k in range(3 + tier):
            q = P(-h + (k * 7 + phase * 5) % (2 * h), (-1) ** k * (4 + tier + phase % 2))
            d.point(q, fill=(255, 255, 255, 255))
    return im


def fire_size(tier: int, big: bool) -> int:
    """The burst canvas: its formation's reach (plan radius + 15000 -> about 55 or 78 px) plus room for the tier art."""
    base = 156 if big else 116
    return base + 16 * tier + (16 if tier == 3 else 0)


FIRE_FRAMES = 9


def burst_motif(d, fam: str, c, R: float, t: float, hue, tier: int):
    """The family's motif at progress t (0..1). R is the formation's reach in pixels."""
    H = hue + (255,)
    W1, W2 = 1 + tier // 2, 2 + (tier + 1) // 2     # stroke widths grow with the tier
    env = min(1.0, t * 5) * (1 - 0.85 * max(0.0, (t - 0.65) / 0.35))   # fade in fast, out over the last third
    a = int(255 * env)
    sq = 0.5
    if fam in ("damage", "burst"):
        # a blade storm: blades converge on the centre and slash; burst adds a star of rays
        n = 6 + 2 * tier
        for k in range(n):
            ang = k * math.tau / n + t * 1.5
            r = R * (1 - min(1.0, t * 1.8))
            tip = (c[0] + math.cos(ang) * r * 0.25, c[1] + math.sin(ang) * r * 0.25 * sq - 4)
            tail = (c[0] + math.cos(ang) * (r + 14), c[1] + math.sin(ang) * (r + 14) * sq - 4)
            if t < 0.55:
                d.line([tail, tip], fill=A((235, 240, 250), a), width=W2)
                d.line([tail, tip], fill=A(hue, a), width=W1)
        if t >= 0.45:
            for k in range(3 + tier):
                ang = k * math.pi / (3 + tier) + 0.4
                L = R * 0.8 * min(1.0, (t - 0.45) * 4)
                d.line([(c[0] - math.cos(ang) * L, c[1] - math.sin(ang) * L * sq), (c[0] + math.cos(ang) * L, c[1] + math.sin(ang) * L * sq)],
                       fill=A((255, 255, 255), a), width=W2)
        if fam == "burst":
            for k in range(12):
                ang = k * math.tau / 12
                L = R * (0.3 + 0.7 * min(1.0, t * 2.5)) * (1 if k % 2 else 0.6)
                d.line([c, (c[0] + math.cos(ang) * L, c[1] + math.sin(ang) * L * sq)], fill=A(hue, a * 0.8), width=W1)
    elif fam == "bind":
        # rune chains closing in, then locking in a ring
        r = R * (1 - 0.45 * min(1.0, t * 2))
        links = 12 + 4 * tier
        for k in range(links):
            ang = k * math.tau / links + t * 0.8
            x, y = c[0] + math.cos(ang) * r, c[1] + math.sin(ang) * r * sq
            if k % 2:
                d.ellipse((x - 3, y - 1.5, x + 3, y + 1.5), outline=A(H, a))
            else:
                d.ellipse((x - 1.5, y - 2.5, x + 1.5, y + 2.5), outline=A((230, 220, 255), a))
        if t > 0.5:
            for k in range(4):
                ang = k * math.pi / 2 + math.pi / 4
                d.line([c, (c[0] + math.cos(ang) * r, c[1] + math.sin(ang) * r * sq)], fill=A(H, a * 0.7), width=W1)
    elif fam == "pull":
        # a vortex: three spiral arms winding inward
        for arm in range(3 + (tier >= 2)):
            pts = []
            for j in range(24):
                u = j / 23
                rr = R * (1 - u) * (1 - 0.3 * t)
                ang = arm * math.tau / (3 + (tier >= 2)) + u * 3.5 - t * 6
                pts.append((c[0] + math.cos(ang) * rr, c[1] + math.sin(ang) * rr * sq))
            d.line(pts, fill=A(H, a), width=W2 if tier >= 2 else W1 + 1)
        d.ellipse((c[0] - 4, c[1] - 2, c[0] + 4, c[1] + 2), fill=A((255, 255, 255), a))
    elif fam == "push":
        # shockwave rings rushing outward
        for k in range(2 + (tier >= 2)):
            u = min(1.0, t * 1.6 - k * 0.18)
            if u <= 0:
                continue
            ground_ring(d, c, R * u, sq, hue, int(a * (1 - u * 0.6)), width=W2 + 1 - k)
        for k in range(8):
            ang = k * math.pi / 4
            r0, r1 = R * min(1.0, t * 1.6) * 0.6, R * min(1.0, t * 1.6) * 0.85
            d.line([(c[0] + math.cos(ang) * r0, c[1] + math.sin(ang) * r0 * sq), (c[0] + math.cos(ang) * r1, c[1] + math.sin(ang) * r1 * sq)],
                   fill=A((255, 255, 255), a), width=W1)
    elif fam in ("shred", "weaken"):
        # debuffs: shards falling onto the ground and a cracked ring
        ground_ring(d, c, R * 0.85, sq, hue, a, width=2, dash=8, turn=t)
        for k in range(10 + 2 * tier):
            x = c[0] + ((k * 37) % 100 - 50) / 50 * R * 0.7
            y0 = c[1] - R * 0.6 + ((k * 23) % 30)
            y = y0 + t * R * 0.9
            if y < c[1] + R * 0.35:
                d.line([(x, y - 5), (x, y)], fill=A(H if k % 2 else (255, 255, 255, 255), a), width=W1)
        if fam == "weaken":
            d.polygon([(c[0] - 6, c[1] - 10), (c[0] + 6, c[1] - 10), (c[0], c[1] - 1)], fill=A(H, a))
    elif fam == "domain":
        # a big seal spinning on the ground
        for k, (rr, spin) in enumerate(((R, 1.2), (R * 0.72, -1.8), (R * 0.45, 2.4))):
            ground_ring(d, c, rr * min(1.0, t * 3), sq, hue if k != 1 else (255, 240, 200), a, width=W2 if k == 0 else W1,
                        ticks=8 + 4 * k, turn=t * spin * math.tau / 4, bright=(255, 255, 255))
        rr = R * 0.72 * min(1.0, t * 3)
        ang0 = t * math.tau / 3
        pts = [(c[0] + math.cos(ang0 + k * math.pi / 2) * rr, c[1] + math.sin(ang0 + k * math.pi / 2) * rr * sq) for k in range(4)]
        d.polygon(pts, outline=A(H, a))
    else:
        # ally buffs: a rising pillar of light with the family glyph floating up
        w = R * 0.28
        top = c[1] - R * 0.9 * min(1.0, t * 2.5)
        for j in range(int(w)):
            al = a * (1 - j / w) * 0.6
            d.line([(c[0] - j, top), (c[0] - j, c[1])], fill=A(hue, al))
            d.line([(c[0] + j, top), (c[0] + j, c[1])], fill=A(hue, al))
        d.line([(c[0], top), (c[0], c[1])], fill=A((255, 255, 255), a), width=W2)
        ground_ring(d, c, R * 0.8 * min(1.0, t * 3), sq, hue, a, width=W2)
        for k in range(6 + 2 * tier):
            x = c[0] + ((k * 41) % 60 - 30) / 30 * R * 0.6
            y = c[1] - ((t * 1.4 + k * 0.13) % 1.0) * R * 0.9
            d.point((x, y), fill=A((255, 255, 255) if k % 2 else hue, a))


def fire_frame(fam: str, tier: int, big: bool, f: int) -> Image.Image:
    """A formation (or solo stroke) firing: the family motif, then a layer per tier: t1 a rune circle, t2 a second
    counter-turning ring and light pillars round it, t3 sword phantoms slamming into the centre, a wide flash ring
    and embers."""
    D = fire_size(tier, big)
    im = Image.new("RGBA", (D, D))
    d = ImageDraw.Draw(im, "RGBA")
    c = ((D - 1) / 2, (D - 1) / 2)
    R = (70_000 if big else 50_000) / UPX
    t = f / (FIRE_FRAMES - 1)
    hue = FAMILY_HUE[fam]
    env = min(1.0, t * 5) * (1 - 0.85 * max(0.0, (t - 0.65) / 0.35))
    a = int(255 * env)
    # the ground flash at the start
    if f < 3:
        fl = Image.new("RGBA", (D, D))
        ImageDraw.Draw(fl).ellipse((c[0] - R, c[1] - R * 0.5, c[0] + R, c[1] + R * 0.5), fill=hue + (110 - 35 * f,))
        im.alpha_composite(fl.filter(ImageFilter.GaussianBlur(3)))
    if tier >= 1:
        ground_ring(d, c, R * 1.02, 0.5, hue, a, width=1, ticks=12, turn=t * math.tau / 6, bright=(255, 255, 255))
    if tier >= 2:
        ground_ring(d, c, R * 0.86, 0.5, (255, 240, 200), int(a * 0.8), dash=10, turn=-t * math.tau / 4)
        for k in range(4):
            ang = k * math.pi / 2 + math.pi / 4 + t * 0.6
            x, y = c[0] + math.cos(ang) * R, c[1] + math.sin(ang) * R * 0.5
            hgt = R * 0.7 * min(1.0, t * 3) * (1 - max(0.0, (t - 0.7) / 0.3))
            d.line([(x, y), (x, y - hgt)], fill=A(hue, a * 0.8), width=3)
            d.line([(x, y), (x, y - hgt)], fill=A((255, 255, 255), a), width=1)
    if tier >= 3:
        # the flash ring rushing out past the formation
        u = min(1.0, t * 1.4)
        ground_ring(d, c, R * (0.4 + 0.7 * u), 0.5, (255, 255, 255), int(220 * (1 - u)), width=2)
        # sword phantoms falling from above into the centre in the first half
        if t < 0.5:
            for k in range(4):
                ang = k * math.pi / 2 + 0.3
                drop = (1 - t * 2) * R * 0.9
                x = c[0] + math.cos(ang) * R * 0.35
                y = c[1] + math.sin(ang) * R * 0.18 - drop
                ghost = Image.new("RGBA", (D, D))
                draw_sword(ImageDraw.Draw(ghost, "RGBA"), (k * 2) % 7, 7, (x, y), math.pi / 2, 0.9, 170, pivot="tip")
                im.alpha_composite(ghost)
        # embers drifting up in the second half
        for k in range(16):
            if t < 0.35:
                break
            x = c[0] + ((k * 53) % 100 - 50) / 50 * R * 0.9
            y = c[1] + ((k * 31) % 20 - 10) - (t - 0.35) * R * (0.8 + (k % 3) * 0.2)
            d.point((x, y), fill=A((255, 220, 140) if k % 2 else hue, a))
    burst_motif(d, fam, c, R * 0.85, t, hue, tier)
    # bloom: a blurred, brighter copy underneath so the burst glows and reads from a distance
    bloom = im.filter(ImageFilter.GaussianBlur(2 + tier))
    bloom.putalpha(bloom.getchannel("A").point(lambda v: min(255, int(v * (1.8 + 0.4 * tier)))))
    out = Image.new("RGBA", im.size)
    out.alpha_composite(bloom)
    out.alpha_composite(im)
    return out


def hitmark_frame(fam: str, tier: int, f: int, n: int = 8) -> Image.Image:
    """On every champion a formation touched: a ring snapping onto them and the family glyph flashing over them."""
    S = 40 + 8 * tier
    im = Image.new("RGBA", (S, S))
    d = ImageDraw.Draw(im, "RGBA")
    c = (S - 1) / 2
    hue = FAMILY_HUE[fam]
    t = f / (n - 1)
    a = int(255 * (1 - 0.85 * max(0.0, (t - 0.5) / 0.5)))
    r = (S / 2 - 2) * (1 - 0.55 * min(1.0, t * 2.5))
    ground_ring(d, (c, c + 6), r, 0.5, hue, a, width=2 if tier >= 1 else 1)
    if tier >= 2:
        ground_ring(d, (c, c + 6), r * 0.7, 0.5, (255, 255, 255), a, dash=6, turn=t * 3)
    g = Image.new("RGBA", (24, 24))
    glyph(ImageDraw.Draw(g), fam, hue)
    rise = int(t * 6)
    gl = g if tier < 3 else g.resize((30, 30), Image.Resampling.NEAREST)
    gl.putalpha(gl.getchannel("A").point(lambda v: v * a // 255))
    im.alpha_composite(gl, (round(c - gl.width / 2), round(c - gl.height / 2 - 10 - rise)))
    if tier >= 2 and f < 4:
        for k in range(6):
            ang = k * math.tau / 6 + f
            d.point((c + math.cos(ang) * (6 + f * 3), c - 10 + math.sin(ang) * (6 + f * 3)), fill=(255, 255, 255, 255))
    return im


def shatter_frame(tier: int, f: int, n: int = 8) -> Image.Image:
    """A formation too far off to take cracks apart: a broken ring and shards flung out, greyer at the low tiers."""
    S = 72 + 16 * tier
    im = Image.new("RGBA", (S, S))
    d = ImageDraw.Draw(im, "RGBA")
    c = (S - 1) / 2
    t = f / (n - 1)
    a = int(255 * (1 - t * 0.8))
    col = ((170, 170, 180), (200, 150, 170), (230, 110, 140), (255, 80, 110))[tier]
    r = S * 0.3
    for k in range(6):
        a0 = k * 60 + 8 + t * 20
        off = t * 6
        ang = math.radians(a0 + 22)
        box = (c - r + math.cos(ang) * off, c - r * 0.5 + math.sin(ang) * off * 0.5,
               c + r + math.cos(ang) * off, c + r * 0.5 + math.sin(ang) * off * 0.5)
        d.arc(box, a0, a0 + 40, fill=A(col, a), width=2)
    for k in range(8 + 4 * tier):
        ang = k * math.tau / (8 + 4 * tier) + 0.3
        dist = 4 + t * S * 0.42
        x, y = c + math.cos(ang) * dist, c + math.sin(ang) * dist * 0.6 - math.sin(t * math.pi) * 6
        d.polygon([(x, y - 2), (x + 1.5, y), (x, y + 2), (x - 1.5, y)], fill=A((240, 240, 250), a), outline=A(INK, a))
    if f < 2:
        d.line([(c - 8, c - 8), (c + 8, c + 8)], fill=A((255, 60, 90), 255), width=2)
        d.line([(c - 8, c + 8), (c + 8, c - 8)], fill=A((255, 60, 90), 255), width=2)
    return im


def crown_flash_frame(f: int, n: int = 10) -> Image.Image:
    """Imperial: a gold crown sigil flashes over every formation he completes."""
    im = Image.new("RGBA", (64, 48))
    d = ImageDraw.Draw(im, "RGBA")
    t = f / (n - 1)
    a = int(255 * (1 - 0.85 * max(0.0, (t - 0.6) / 0.4)))
    s = 0.6 + 0.6 * min(1.0, t * 3)
    cx, cy = 32, 26 - t * 6
    for k in range(10):
        ang = k * math.pi / 5 + t
        L = 14 + 8 * math.sin(t * math.pi)
        d.line([(cx + math.cos(ang) * 8, cy + math.sin(ang) * 8), (cx + math.cos(ang) * L, cy + math.sin(ang) * L)], fill=A(GOLD_L, a * 0.7))
    pts = [(-9, 6), (-9, -2), (-5, 2), (-2, -5), (0, 0), (2, -5), (5, 2), (9, -2), (9, 6)]
    d.polygon([(cx + x * s, cy + y * s) for x, y in pts], fill=A(GOLD, a), outline=A(INK, a))
    for k, x in enumerate((-5, 0, 5)):
        d.point((cx + x * s, cy + 3 * s), fill=A(GEMS[(k + f) % 4], a))
    return im


def engraving_sheet(tier: int) -> tuple[dict, dict]:
    """Every engraving visual of one tier: scars, flares, bursts, hit markers and the shatter (and Imperial's crown)."""
    anims, dur = {}, {}
    for k in range(7):
        for ang in range(16):
            for kind, lit in (("scar", False), ("flare", True)):
                tag = f"{kind}_{k}_t{tier}_a{ang}"
                anims[tag] = [scar_frame(k, tier, ang, ph, lit) for ph in range(SCAR_PHASES)]
                dur[tag] = 3 / 60
            tag = f"scar_dim_{k}_t{tier}_a{ang}"
            anims[tag] = [scar_frame(k, tier, ang, 0, False, cool=True)]
            dur[tag] = COOL_SECONDS
    for fam in FAMILIES:
        for big in (False, True):
            tag = f"fire_{fam}_t{tier}_r{int(big)}"
            anims[tag] = [fire_frame(fam, tier, big, f) for f in range(FIRE_FRAMES)]
            dur[tag] = 0.07
        tag = f"hitmark_{fam}_t{tier}"
        anims[tag] = [hitmark_frame(fam, tier, f) for f in range(8)]
        dur[tag] = 0.06
    anims[f"shatter_t{tier}"] = [shatter_frame(tier, f) for f in range(8)]
    dur[f"shatter_t{tier}"] = 0.06
    if tier == 3:
        anims["crown_flash"] = [crown_flash_frame(f) for f in range(10)]
        dur["crown_flash"] = 0.06
    return {tag: trim_centred(frames) for tag, frames in anims.items()}, dur


def trim_centred(frames: list[Image.Image]) -> list[Image.Image]:
    """Crop an effect's frames to their joint content, symmetric round the centre (the effect's anchor), so the sheet
    holds no empty margins while the art stays where it was."""
    W, H = frames[0].size
    cx, cy = W / 2, H / 2
    dx = dy = 1.0
    for f in frames:
        b = f.getbbox()
        if b:
            dx = max(dx, cx - b[0], b[2] - cx)
            dy = max(dy, cy - b[1], b[3] - cy)
    w, h = min(W, 2 * math.ceil(dx)), min(H, 2 * math.ceil(dy))
    box = (round(cx - w / 2), round(cy - h / 2), round(cx - w / 2) + w, round(cy - h / 2) + h)
    return [f.crop(box) for f in frames]


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


def save(name: str, anims: dict[str, list[Image.Image]], durations, aliases: tuple[str, ...] = (), colors: int = 0,
         editor: bool = True, pairs: tuple[str, ...] = (), game: bool = True) -> dict:
    """Write mods/tfm2_custom/vfx/<name>; for tags starting with any of `aliases`, add <tag>_frame<k> single-frame
    aliases sharing the pixels (moving world effects keep their phase without restarting)."""
    if isinstance(durations, (int, float)):
        durations = {t: durations for t in anims}
    sheet, meta = shelf_pack(anims, durations)
    for tag in list(meta):
        if tag.startswith(aliases) if aliases else False:
            for k, entry in enumerate(meta[tag]["frames"]):
                meta[f"{tag}_frame{k}"] = {"frames": [entry]}
        if tag.startswith(pairs) if pairs else False:   # round 91: 2-frame pieces of a loop, one emission each
            fr = meta[tag]["frames"]
            for k in range(len(fr) // 2):
                meta[f"{tag}_pair{k}"] = {"frames": [fr[2 * k], fr[2 * k + 1]]}
    target = MOD / "vfx" / name if game else EDITOR / f"isliid-{name}-8"
    if colors:   # round 89: the bloomed engraving sheets keep RGBA but at most `colors` colours (a third the size)
        # alpha in steps of 8 with empty pixels kept exactly empty (quantizing RGBA together could make them faintly
        # opaque), colours to a palette, and no colour left under empty pixels
        alpha = sheet.getchannel("A").point(lambda a: 0 if a == 0 else min(255, max(8, (a + 4) // 8 * 8)))
        empty = alpha.point(lambda a: 255 if a == 0 else 0)
        rgb = sheet.convert("RGB")
        rgb.paste((0, 0, 0), mask=empty)
        sheet = rgb.quantize(colors=colors, method=Image.Quantize.MEDIANCUT, dither=Image.Dither.NONE).convert("RGBA")
        sheet.paste((0, 0, 0), mask=empty)
        sheet.putalpha(alpha)
    if not game:   # round 95: an editor-only sheet (the editor reads the manifest, not the .fanim)
        sheet.save(str(target) + ".png", optimize=True)
        return meta
    sheet.save(str(target) + "#sheet.png", optimize=True)
    (MOD / "vfx" / f"{name}#anim.fanim").write_text(json.dumps({"anims": meta}, separators=(",", ":")), encoding="utf-8")
    if editor:
        shutil.copyfile(str(target) + "#sheet.png", EDITOR / f"isliid-{name}-8.png")
    return meta


def main(preview: str | None = None) -> None:
    P = "tfm2_isliid_emperor_"
    # swords in flight: the comets (round 96; the editor draws them too, round 97)
    comets = save("swords_comet", {f"{s}_rank{r}_comet_a{h}": trim_centred([comet.comet_frame(k, r, h, f, COLORS)
                                                                            for f in range(comet.FRAMES)])
                                   for r in range(comet.RANKS) for k, s in enumerate(SWORDS)
                                   for h in range(comet.HEADINGS)},
                  0.05, aliases=tuple(SWORDS), pairs=tuple(SWORDS), colors=256)
    # round 97: Imperial's black hole (a buff above his head, by how many swords are inside; #1 prismatic) and the
    # wormholes his swords leave and arrive through
    holes = save("blackhole", {f"blackhole{v}_n{n}": trim_centred([comet.blackhole_frame(n, p, COLORS, v == "1")
                                                                  for p in range(comet.HOLE_FRAMES)])
                               for v in ("", "1") for n in range(8)}, 0.06)
    worms = save("wormhole", {f"wormhole_{way}_{s}": trim_centred([comet.wormhole_frame(k, f, way == "in", COLORS)
                                                                   for f in range(comet.WORM_FRAMES)])
                              for way in ("out", "in") for k, s in enumerate(SWORDS)} |
                 {f"wormhole_{way}_p": trim_centred([comet.wormhole_frame(None, f, way == "in", COLORS)
                                                     for f in range(comet.WORM_FRAMES)]) for way in ("out", "in")}, 0.04)
    # grounded swords and the one-shot effects
    ground, dur = {}, {}
    for r in range(8):
        for k, s in enumerate(SWORDS):
            for state in ("planted", "ready"):
                tag = f"{s}_rank{r}_{state}"
                if r == 7:   # round 98: at Imperial a waiting sword is its celestial body hovering over its point
                    ground[tag] = [comet.grounded_body_frame(k, p, PLANTED_FRAMES, state == "ready", COLORS)
                                   for p in range(PLANTED_FRAMES)]
                else:
                    ground[tag] = [planted_frame(k, r, p, state == "ready") for p in range(PLANTED_FRAMES)]
                dur[tag] = 0.1
    for k, s in enumerate(SWORDS):
        for tag, frames, d_ in ((f"{s}_impact", [impact_frame(k, f) for f in range(6)], 0.05),
                                (f"{s}_launch", [launch_frame(k, f) for f in range(4)], 0.04),
                                (f"{s}_recall", [recall_frame(k, f) for f in range(5)], 0.035),
                                (f"{s}_hit", [hit_frame(k, f) for f in range(5)], 0.035),
                                (f"{s}_hit_cosmic", [comet.cosmic_hit_frame(k, f, COLORS) for f in range(5)], 0.035)):
            ground[tag] = frames
            dur[tag] = d_
    swords = save("swords8", ground, dur, pairs=tuple(f"{s}_rank" for s in SWORDS))
    # the arsenal ring
    orbit = {}
    for r in range(8):
        for k, s in enumerate(SWORDS):
            if r == 7:   # round 97: at Imperial a sword guarding an ally circles it as its celestial body
                ring = (SLOT_RX, SLOT_RY, RING_Y)
                orbit[f"ar_{s}_rank{r}"] = [comet.orbit_body_frame(k, p, False, COLORS, ring) for p in range(8)]
                orbit[f"ar_{s}_rank{r}_sel"] = [comet.orbit_body_frame(k, p, True, COLORS, ring) for p in range(8)]
                continue
            orbit[f"ar_{s}_rank{r}"] = [orbit_frame(k, r, p, False) for p in range(8)]
            orbit[f"ar_{s}_rank{r}_sel"] = [orbit_frame(k, r, p, True) for p in range(8)]
    # round 90: each frame cropped to its content round the anchor (the 128 x 128 frames held one small blade)
    orbit_meta = save("orbit8", {tag: trim_centred(fr) for tag, fr in orbit.items()}, 0.1)
    # auras (unchanged look)
    aura_anims = {}
    for r in range(8):
        for side in ("ally", "enemy"):
            aura_anims[f"aura_base_rank{r}_{side}"] = [aura_base_frame(r, p, side == "enemy") for p in range(8)]
        for k in range(7):
            for side in ("ally", "enemy"):
                aura_anims[f"aura_{k}_rank{r}_{side}"] = [aura_frame(k, r, p, side == "enemy") for p in range(8)]
    auras = save("auras8", {tag: trim_centred(fr) for tag, fr in aura_anims.items()}, 0.1)
    fields = save("aura_fields8", {f"aura_field_{k}_rank{r}": trim_centred([aura_field_frame(k, r, p) for p in range(8)])
                                    for r in range(8) for k in range(7)}, 0.1, aliases=("aura_field_",),
                  pairs=("aura_field_",))   # round 93: emitted as 2-frame pairs every 12 ticks
    badges = save("badges8", badge_frames(), 0.08)
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
    # round 89: the completed logo pops (single-frame aliases _f0.._f5, picked by the native code every 6 ticks)
    for fam in FAMILIES:
        for f in range(POP_FRAMES):
            logos[f"logo_{fam}_complete_f{f}"] = [logo_pop_frame(fam, f)]
    for k in range(7):
        for f in range(POP_FRAMES):
            logos[f"logo_solo{k}_complete_f{f}"] = [logo_pop_frame(None, f, k)]
    logo_meta = save("logos", logos, 0.1)
    # round 89: the engravings by tier (scars, flares, bursts, hit markers, shatter; the crown flash in t3)
    engrave = {}
    for tier in range(4):
        anims, dur = engraving_sheet(tier)
        engrave[tier] = save(f"engrave_t{tier}", anims, dur, colors=256, editor=False)
    # round 98: Imperial's strokes are constellation lines (tier 4: strokes only; its bursts and markers stay tier 3)
    const = {}
    for k in range(7):
        for ang in range(16):
            for kind, lit in (("scar", False), ("flare", True)):
                const[f"{kind}_{k}_t4_a{ang}"] = trim_centred([comet.constellation_frame(k, ang, ph, lit, False, SCAR_STEP / UPX, COLORS)
                                                               for ph in range(SCAR_PHASES)])
            const[f"scar_dim_{k}_t4_a{ang}"] = trim_centred([comet.constellation_frame(k, ang, 0, False, True, SCAR_STEP / UPX, COLORS)])
    engrave[4] = save("engrave_t4", const, {t: (COOL_SECONDS if t.startswith("scar_dim") else 3 / 60) for t in const},
                      colors=256, editor=False)
    # round 98: Imperial's formation fire: the dominant sword's body falling on the engraving (radius 35000 / 55000;
    # _p = Imperial #1, prismatic)
    falls = save("falls", {f"fall_{s}_r{big}{v}": trim_centred([comet.fall_frame(k, f, (55_000 if big else 35_000) / 950,
                                                                                    v == "_p", COLORS)
                                                                 for f in range(comet.FALL_FRAMES)])
                           for k, s in enumerate(SWORDS) for big in (0, 1) for v in ("", "_p")}, 0.045, colors=256)

    # the data: replace every sword / orbit / badge / flag view, keep the rest
    data_path = MOD / "champion" / "tfm2_isliid_emperor.data_champion"
    data = json.loads(data_path.read_text(encoding="utf-8"))
    old_sword = tuple(P + s + "_" for s in SWORDS)
    data["view_effects"] = [v for v in data["view_effects"] if not v["name"].startswith(old_sword)
                            and not v["name"].startswith(P + "aura_") and not v["name"].startswith(P + "flag_")
                            and not v["name"].startswith(P + "logo_")
                            and not re.match(r"(scar|scar_dim|flare)_\d_", v["name"].removeprefix(P))
                            and not v["name"].removeprefix(P).startswith(("fire_", "hitmark_", "shatter_", "crown_flash",
                                                                          "wormhole_", "fall_"))]
    data["view_buffs"] = [v for v in data["view_buffs"] if not v["name"].startswith(
        ("il_ar_", "il_rank", "il_imperial", "il_aura_base_", "il_aura_visual_", "il_selected_", "il_blackhole"))]
    data["view_projectiles"] = []   # round 95: flying swords are point effects (swords_dir), not projectiles
    for tag in comets:   # only the aliases are played (single frames every 3 ticks, pairs every 6)
        if "_frame" in tag or "_pair" in tag:
            data["view_effects"].append({"type": "Animation", "name": P + tag, "anim": "asset/tfm2_custom/vfx/swords_comet",
                                         "tag": tag, "z": 3, "is_follow": False})
    for tag in swords:
        if re.search(r"_rank\d_(planted|ready)$", tag):
            continue   # only the frame aliases are played (emitted every 3 ticks)
        follow = tag.endswith(("_hit", "_hit_cosmic"))
        data["view_effects"].append({"type": "Animation", "name": P + tag, "anim": "asset/tfm2_custom/vfx/swords8",
                                     "tag": tag, "z": 3, "is_follow": follow})
    for tag in holes:
        data["view_buffs"].append({"type": "Animated", "name": "il_" + tag, "anim": "asset/tfm2_custom/vfx/blackhole", "tag": tag, "z": 4})
    for tag in falls:
        data["view_effects"].append({"type": "Animation", "name": P + tag, "anim": "asset/tfm2_custom/vfx/falls",
                                     "tag": tag, "z": 4, "is_follow": False})
    for tag in worms:
        data["view_effects"].append({"type": "Animation", "name": P + tag, "anim": "asset/tfm2_custom/vfx/wormhole",
                                     "tag": tag, "z": 4, "is_follow": False})
    for tag in orbit_meta:
        data["view_buffs"].append({"type": "Animated", "name": "il_" + tag, "anim": "asset/tfm2_custom/vfx/orbit8", "tag": tag, "z": 2})
    for tag in auras:
        name = ("il_" + tag) if tag.startswith("aura_base_") else ("il_aura_visual_" + tag[5:])
        data["view_buffs"].append({"type": "Animated", "name": name, "anim": "asset/tfm2_custom/vfx/auras8", "tag": tag, "z": -1})
    for tag in fields:
        if "_frame" in tag or "_pair" in tag:
            data["view_effects"].append({"type": "Animation", "name": P + tag, "anim": "asset/tfm2_custom/vfx/aura_fields8",
                                         "tag": tag, "z": -2, "is_follow": False})
    for tag in badges:
        data["view_buffs"].append({"type": "Animated", "name": "il_" + tag, "anim": "asset/tfm2_custom/vfx/badges8", "tag": tag, "z": 4})
    for tag in logo_meta:
        data["view_effects"].append({"type": "Animation", "name": P + tag, "anim": "asset/tfm2_custom/vfx/logos",
                                     "tag": tag, "z": 5, "is_follow": False})
    for tier, meta in engrave.items():
        for tag in meta:
            data["view_effects"].append({"type": "Animation", "name": P + tag, "anim": f"asset/tfm2_custom/vfx/engrave_t{tier}",
                                         "tag": tag, "z": 4 if tag.startswith("hitmark_") else 2,
                                         "is_follow": tag.startswith("hitmark_")})
    data_path.write_text(json.dumps(data, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    manifest = {"comets": comets, "blackhole": holes, "wormholes": worms, "falls": falls, "swords": swords, "orbit": orbit_meta, "auras": auras, "fields": fields,
                "badges": badges, "logos": logo_meta, "engrave": {f"t{t}": m for t, m in engrave.items()},
                "patterns": [{"name": n, "family": FAMILIES[int(e)]} for n, e in patterns]}
    (EDITOR / "isliid-art-manifest.json").write_text(json.dumps(manifest, separators=(",", ":")), encoding="utf-8")
    for stale in ("flags", "selector", "swords", "orbit", "badges", "engraving_colors", "swords_fly8", "swords_dir"):
        for ext in ("#sheet.png", "#anim.fanim"):
            p = MOD / "vfx" / f"{stale}{ext}"
            if p.exists():
                p.unlink()
    for stale in ("isliid-flags-8.png", "isliid-swords_fly8-8.png"):
        p = EDITOR / stale
        if p.exists():
            p.unlink()
    print(f"Generated {len(comets)} comet flight, {len(swords)} ground, {len(orbit_meta)} orbit, {len(auras)} aura, "
          f"{len(fields)} field, {len(badges)} badge, {len(logo_meta)} logo, "
          f"{sum(len(m) for m in engrave.values())} engraving animations; "
          f"{len(data['view_projectiles'])} projectile, {len(data['view_effects'])} effect, {len(data['view_buffs'])} buff views")
    if preview:
        previews(Path(preview))


def _row(cells, bg, pad=4, bottom=True):
    W = sum(c.width for c in cells) + pad * (len(cells) + 1)
    H = max(c.height for c in cells) + 2 * pad
    im = Image.new("RGBA", (W, H), bg)
    x = pad
    for c in cells:
        im.alpha_composite(c, (x, H - pad - c.height if bottom else (H - c.height) // 2))
        x += c.width + pad
    return im


def _gif(frames, path, scale, duration):
    out = [f.resize((f.width * scale, f.height * scale), Image.Resampling.NEAREST).convert("RGB") for f in frames]
    out[0].save(path, save_all=True, append_images=out[1:], duration=duration, loop=0)


def previews(folder: Path) -> None:
    folder.mkdir(parents=True, exist_ok=True)
    bg = (40, 52, 46, 255)
    # badges: every rank and Imperial #10..#1, the sigil cropped round its corner of the buff frame
    box = (SIGIL[0] - 17, 0, SIGIL[0] + 17, 60)
    combos = [(r, None) for r in range(7)] + [(7, n) for n in range(10, 0, -1)]
    strip = lambda p: _row([badge_frame(r, p, n).crop(box) for r, n in combos], bg, pad=0, bottom=False)
    s0 = strip(0)
    s0.resize((s0.width * 4, s0.height * 4), Image.Resampling.NEAREST).save(folder / "isliid_badges.png")
    _gif([strip(p) for p in range(BADGE_FRAMES)], folder / "isliid_badges.gif", 4, 80)
    # swords: Skylight, Darkbringer, Rift and Emperor at ranks 0 / 3 / 5 / 7, planted
    sh = _row([planted_frame(k, r, 0, False) for k in (0, 2, 5, 6) for r in (0, 3, 5, 7)], bg)
    sh.resize((sh.width * 3, sh.height * 3), Image.Resampling.NEAREST).save(folder / "isliid_swords.png")
    # the sword kit at Swordmaster / Regent / Imperial: planted, armed, flying, engraving, and the ring
    kit = []
    for f in range(PLANTED_FRAMES):
        top = _row([planted_frame(k, r, f, k == 5) for r in (4, 5, 7) for k in (3, 5)], bg)
        mid = _row([flight_frame(k, r, f % 4, k == 6) for r in (4, 5, 7) for k in (4, 6)], bg, bottom=False)
        ring = Image.new("RGBA", (128, 128))
        for k in range(7):
            ring.alpha_composite(orbit_frame(k, 7, f % 8, k == 3))
        im = Image.new("RGBA", (max(top.width, mid.width + 136), top.height + max(mid.height, 128)), bg)
        im.alpha_composite(top, (0, 0)); im.alpha_composite(mid, (0, top.height)); im.alpha_composite(ring, (mid.width + 8, top.height))
        kit.append(im)
    _gif(kit, folder / "isliid_sword_kit.gif", 3, 100)
    # the engravings firing, by tier: lit strokes then the burst, for four families
    fams = ("damage", "bind", "pull", "heal")
    D = fire_size(3, True)
    frames = []
    for f in range(FIRE_FRAMES + 4):
        im = Image.new("RGBA", (D * len(fams), D * 4), bg)
        for t in range(4):
            for j, fam in enumerate(fams):
                c = (j * D + D // 2, t * D + D // 2)
                for k in range(3):   # a triangle of lit legs
                    for q in range(5):
                        a0 = k * math.tau / 3 - math.pi / 2
                        a1 = a0 + math.tau / 3
                        u = q / 4
                        x = c[0] + (math.cos(a0) * (1 - u) + math.cos(a1) * u) * 45
                        y = c[1] + (math.sin(a0) * (1 - u) + math.sin(a1) * u) * 22
                        ang = round(math.atan2(math.sin(a1) * 22 - math.sin(a0) * 22, math.cos(a1) * 45 - math.cos(a0) * 45) % math.pi * 16 / math.pi) % 16
                        sc = scar_frame((j * 2 + k) % 7, t, ang, f % 2, f < 9)
                        im.alpha_composite(sc, (round(x - sc.width / 2), round(y - sc.height / 2)))
                if f < FIRE_FRAMES:
                    b = fire_frame(fam, t, True, f)
                    im.alpha_composite(b, (c[0] - b.width // 2, c[1] - b.height // 2))
        frames.append(im)
    _gif(frames, folder / "isliid_engravings.gif", 2, 70)
    # logos: every family / solo sword in its four phases, then the completion pop
    rows = list(FAMILIES) + [None] * 7
    lg = Image.new("RGBA", (34 * (4 + POP_FRAMES), 34 * len(rows)), bg)
    for i, fam in enumerate(rows):
        sword = None if fam else i - len(FAMILIES)
        for j, ph in enumerate(PHASES):
            lg.alpha_composite(logo_frame(fam, ph, sword), (j * 34 + 5, i * 34 + 5))
        for f in range(POP_FRAMES):
            lg.alpha_composite(logo_pop_frame(fam, f, sword), ((4 + f) * 34 + 1, i * 34 + 1))
    lg.resize((lg.width * 3, lg.height * 3), Image.Resampling.NEAREST).save(folder / "isliid_logos.png")


if __name__ == "__main__":
    main(sys.argv[sys.argv.index("--preview") + 1] if "--preview" in sys.argv else None)
