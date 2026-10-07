"""Round 96: Isliid's swords in flight as Spirit comets (Imperial: a little solar system).

A flying sword dissolves into a compact glyph of its own shape and colour with a rippling tail, drawn at one of 8
headings (0 right, 2 down, 4 left, 6 up; pixel y down like the scars) in an 8-frame loop. The art grows by rank:

  0 Bearer       glyph + short tail
  1 Squire       + soft glow
  2 Engraver     + sparks shed from the tail
  3 Tactician    + 1 orbiting mote
  4 Swordmaster  + 2 motes, longer tail
  5 Regent       + a rotating dashed rune ring
  6 Sovereign    + 3 motes, a brighter double tail
  7 Imperial     a solar system: the glyph is a sun with turning corona rays, the seven sword colours circle it as
                 planets on three tilted orbits (inner faster), the tail behind
  8 Imperial #1  the same with a prismatic corona and a rainbow tail

Everything that turns moves a whole share of its period per frame, so the 8 frames loop seamlessly. The sword's
position is the canvas centre. Drawn 3x and reduced, so the small shapes stay clean.
"""
from __future__ import annotations

import math

from PIL import Image, ImageDraw, ImageFilter

SS = 3                       # supersampling
HEADINGS = 8
FRAMES = 8
RANKS = 9                    # 0..7, plus 8 = Imperial #1
HALF = (11, 12, 13, 14, 16, 18, 20, 24, 24)            # half canvas, px
CORE = (3.9, 4.0, 4.2, 4.3, 4.4, 4.6, 4.8, 5.0, 5.2)   # glyph radius, px
TAIL = (7, 8, 10, 11, 13, 14, 16, 17, 18)              # tail length, px
MOTES = (0, 0, 0, 1, 2, 2, 3, 0, 0)

# local glyph outlines: x forward (along the heading), y across, in units of the core radius
GLYPHS = {
    0: [(1.7, 0), (0.3, 0.3), (0, 1.0), (-0.3, 0.3), (-1.0, 0), (-0.3, -0.3), (0, -1.0), (0.3, -0.3)],      # star needle
    1: [(1.25, 0), (0.1, 0.95), (-1.0, 0.15), (-0.75, -0.7), (0.15, -0.9)],                                # rock
    2: [(1.5, 0), (0.55, 0.55), (0.2, 0.28), (-0.15, 0.75), (-0.45, 0.32), (-1.0, 0.55), (-0.75, 0),
        (-1.0, -0.55), (-0.45, -0.32), (-0.15, -0.75), (0.2, -0.28), (0.55, -0.55)],                       # serrated
    4: [(1.5, 0), (0.55, 0.62), (-0.2, 0.8), (-0.85, 0.5), (-1.0, -0.05), (-0.7, -0.6), (-0.1, -0.75),
        (0.55, -0.6)],                                                                                         # droplet
    6: [(1.3, 0), (0.0, 0.78), (-0.75, 0.55), (-1.15, 0.85), (-1.0, 0.3), (-1.3, 0), (-1.0, -0.3),
        (-1.15, -0.85), (-0.75, -0.55), (0.0, -0.78)],                                                        # crowned
}


def _crescent():
    outer = [(math.cos(math.radians(a)), math.sin(math.radians(a))) for a in range(-120, 121, 15)]
    inner = [(-0.5 + 0.85 * math.cos(math.radians(a)), 0.85 * math.sin(math.radians(a))) for a in range(110, -111, -15)]
    return outer + inner


GLYPHS[3] = _crescent()                                                                                       # crescent
GLYPHS[5] = [(1.4, 0.18), (-0.9, 0.85), (-0.6, 0.18), (-0.6, -0.18), (-0.9, -0.85), (1.4, -0.18), (0.2, 0)]  # prongs


def _a(c, a):
    return c[:3] + (max(0, min(255, int(a))),)


def _mix(c1, c2, t):
    return tuple(int(c1[k] + (c2[k] - c1[k]) * t) for k in range(3)) + (255,)


def comet_frame(kind: int, rank: int, heading: int, f: int, colors) -> Image.Image:
    """One frame of sword `kind` flying at `heading` (0..7) for art rank 0..8. `colors` = the generator's COLORS."""
    dark, color, bright = colors[kind]
    H = HALF[rank]
    S = 2 * H * SS
    c = S / 2
    ang = heading * math.tau / HEADINGS
    ux, uy = math.cos(ang), math.sin(ang)
    nx, ny = -uy, ux
    ph = f * math.tau / FRAMES                      # the loop phase
    pulse = 1 + 0.12 * math.sin(ph)
    g = CORE[rank] * SS * pulse                     # glyph radius, canvas px
    imperial = rank >= 7
    rainbow = rank == 8

    def P(x, y):                                   # local (forward, across) in art px -> canvas
        return (c + (ux * x + nx * y) * SS, c + (uy * x + ny * y) * SS)

    def orbit_point(radius, squash, tilt, deg):
        a = math.radians(deg)
        x, y = math.cos(a) * radius, math.sin(a) * radius * squash
        tr = math.radians(tilt)
        return (c + (x * math.cos(tr) - y * math.sin(tr)) * SS, c + (x * math.sin(tr) + y * math.cos(tr)) * SS,
                math.sin(a))                         # depth: > 0 is in front of the core

    # --- the glow layer (blurred): the halo (Squire up) and Imperial's corona rays
    glow = Image.new("RGBA", (S, S))
    gd = ImageDraw.Draw(glow, "RGBA")
    if rank >= 1:
        r = g * (1.7 + 0.2 * rank / 8)
        gd.ellipse((c - r, c - r, c + r, c + r), fill=_a(color, 45 + 30 * (1 + math.sin(ph)) / 2 + 5 * rank))
    if imperial:   # 8 rays turning 5.625 degrees a frame (the pattern repeats every 45), long and short, pulsing
        for k in range(8):
            a = math.radians(k * 45 + f * 5.625)
            r0, r1 = g * 1.15, g * ((2.1 if k % 2 == 0 else 1.6) + 0.25 * math.sin(ph + k))
            col = colors[k % 7][2] if rainbow else bright
            gd.line([(c + math.cos(a) * r0, c + math.sin(a) * r0), (c + math.cos(a) * r1, c + math.sin(a) * r1)],
                    fill=_a(col, 210), width=SS)
    im = glow.filter(ImageFilter.GaussianBlur(SS * (1.2 if rank < 7 else 1.6)))
    d = ImageDraw.Draw(im, "RGBA")
    if imperial:   # the rays again, crisp, over their own glow
        for k in range(8):
            a = math.radians(k * 45 + f * 5.625)
            r0, r1 = g * 1.3, g * ((2.0 if k % 2 == 0 else 1.55) + 0.25 * math.sin(ph + k))
            col = colors[k % 7][2] if rainbow else bright
            d.line([(c + math.cos(a) * r0, c + math.sin(a) * r0), (c + math.cos(a) * r1, c + math.sin(a) * r1)],
                   fill=_a(col, 190), width=max(1, SS * 2 // 3))

    # --- the tail: a tapered ribbon behind, rippling; Sovereign doubles it, Imperial #1 is a rainbow
    length, width = TAIL[rank], 1.9 + 0.15 * min(rank, 7)
    steps = int(length * SS)
    for off in ([-1.3, 1.3] if rank == 6 else [0.0]):
        for j in range(steps, 0, -1):
            t = j / steps
            wob = math.sin(t * 7 - ph) * (0.4 + 0.9 * t) + off * (1 - 0.4 * t)
            px, py = P(-(CORE[rank] * 0.6 + t * length), wob)
            half = (width * (1 - t) ** 0.8 + 0.25) * SS
            if rainbow:
                k = int(t * 7) % 7
                col = _mix(colors[k][1], colors[(k + 1) % 7][1], t * 7 % 1)
            else:
                col = _mix(bright, color, min(1, t * 1.6)) if t < 0.6 else _mix(color, dark, (t - 0.6) / 0.4)
            d.ellipse((px - half, py - half, px + half, py + half), fill=_a(col, 235 * (1 - t) ** 1.25))
    # --- sparks shed from the tail (Engraver up), drifting back over the loop
    if rank >= 2:
        for k in range(2 + min(rank, 7) // 2):
            t = (k * 0.37 + f / FRAMES) % 1
            px, py = P(-(CORE[rank] + t * length * 0.9), (1 if k % 2 else -1) * (0.8 + 1.6 * t + (k % 3) * 0.4))
            s = (0.7 if t > 0.5 else 0.9) * SS
            d.ellipse((px - s, py - s, px + s, py + s), fill=_a((255, 255, 255, 255) if t < 0.5 else bright, 240 * (1 - t)))
    # --- the rune ring (Regent, Sovereign): 8 dashes turning 5.625 degrees a frame
    if rank in (5, 6):
        radius = CORE[rank] + 6.5
        for k in range(8):
            seg = [orbit_point(radius, 0.62, -25, k * 45 + f * 5.625 + s_)[:2] for s_ in range(0, 23, 3)]
            d.line(seg, fill=_a(bright, 150), width=SS)

    # orbiting bodies, sorted so the far side passes behind the core: (depth, x, y, radius, (dark, colour, bright))
    bodies = []
    n = MOTES[rank]
    for k in range(n):   # motes (Tactician, Swordmaster, Sovereign): 45 degrees a frame
        x, y, depth = orbit_point(CORE[rank] + 4.5, 0.6, -25, k * 360 / n + f * 45)
        bodies.append((depth, x, y, 1.1 * SS, (color, color, bright)))
    if imperial:   # three tilted orbits, the seven sword colours as planets, inner ones faster (135 / 90 / 45 a frame)
        orbits = ((8.0, (0, 1), 3), (13.0, (2, 3), 2), (18.5, (4, 5, 6), 1))
        for radius, _, _ in orbits:
            pts = [orbit_point(radius, 0.5, -22, a)[:2] for a in range(0, 361, 6)]
            d.line(pts, fill=_a(bright, 60), width=max(1, SS // 2))
        for radius, planets, step in orbits:
            for j, p in enumerate(planets):
                x, y, depth = orbit_point(radius, 0.5, -22, j * 360 / len(planets) + p * 17 + f * 45 * step)
                bodies.append((depth, x, y, (1.25 + 0.15 * (radius > 10)) * SS, colors[p]))

    def body(depth, x, y, s, cols):
        bd, bc, bb = cols
        dim = 0.65 if depth < 0 else 1.0
        d.ellipse((x - s * 1.7, y - s * 1.7, x + s * 1.7, y + s * 1.7), fill=_a(bc, 70 * dim))
        d.ellipse((x - s, y - s, x + s, y + s), fill=_a(_mix(bd, bc, dim), 255), outline=_a(bd, 255))
        h = s * 0.4
        d.ellipse((x - s * 0.3 - h, y - s * 0.3 - h, x - s * 0.3 + h, y - s * 0.3 + h), fill=_a(bb, 255 * dim))

    bodies.sort(key=lambda b: b[0])
    for b in bodies:
        if b[0] < 0:
            body(*b)
    # --- the glyph: the sword's own shape pointing along the heading (on a sun at Imperial)
    if imperial:
        r = g * 1.25
        d.ellipse((c - r, c - r, c + r, c + r), fill=_a(color, 255), outline=_a(dark, 255), width=SS)
    k_ = g / SS
    d.polygon([P(x * k_, y * k_) for x, y in GLYPHS[kind]], fill=_a(color, 255), outline=_a(dark, 255), width=SS)
    d.polygon([P(x * k_ * 0.45 + 0.15 * CORE[rank], y * k_ * 0.45) for x, y in GLYPHS[kind]],
              fill=_a(bright, 230 + 25 * math.sin(ph)))
    hot = (0.6 + 0.25 * math.sin(ph)) * SS
    hx, hy = P(0.35 * CORE[rank], 0)
    d.ellipse((hx - hot, hy - hot, hx + hot, hy + hot), fill=(255, 255, 255, 255))
    for b in bodies:
        if b[0] >= 0:
            body(*b)
    return im.resize((2 * H, 2 * H), Image.Resampling.LANCZOS)
