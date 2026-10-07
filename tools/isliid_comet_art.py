"""Round 96: Isliid's swords in flight as Spirit comets (Imperial: a little solar system). Round 97: each sword is its
own celestial body (Skylight a twinkling star, Terra a cratered planet, Darkbringer a black hole, Gale an icy comet,
Blood a red giant, Rift a spiral wormhole, Emperor a sun), and the centre of its solar system at Imperial.

A flying sword dissolves into a compact body of its own colour with a rippling tail, drawn at one of 8
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
HALF = (13, 13, 14, 14, 16, 18, 20, 24, 24)            # half canvas, px
CORE = (3.9, 4.0, 4.2, 4.3, 4.4, 4.6, 4.8, 5.0, 5.2)   # glyph radius, px
TAIL = (10, 10, 11, 12, 13, 14, 16, 17, 18)              # tail length, px
MOTES = (0, 0, 0, 1, 2, 2, 3, 0, 0)

def _a(c, a):
    return c[:3] + (max(0, min(255, int(a))),)


def _mix(c1, c2, t):
    return tuple(int(c1[k] + (c2[k] - c1[k]) * t) for k in range(3)) + (255,)


def celestial(d: ImageDraw.ImageDraw, kind: int, cx: float, cy: float, r: float, f: int, colors) -> None:
    """Round 97: sword `kind` as a celestial body of radius `r` (canvas px) at (cx, cy), frame `f` of an 8-frame loop.
    Whatever turns moves a whole share of its symmetry per frame, so the loop closes."""
    dark, color, bright = colors[kind]
    ph = f * math.tau / FRAMES
    W = max(1, round(r / 5))                         # outline width
    disc = lambda rr, **kw: d.ellipse((cx - rr, cy - rr, cx + rr, cy + rr), **kw)
    if kind == 0:   # Skylight: a twinkling star, the long cross and the short diagonals trade lengths
        for k in range(8):
            a = k * math.pi / 4
            long_ = k % 2 == 0
            ln = r * ((2.3 + 0.5 * math.sin(ph)) if long_ else (1.3 + 0.5 * math.sin(ph + math.pi)))
            wd = r * (0.32 if long_ else 0.22)
            tip = (cx + math.cos(a) * ln, cy + math.sin(a) * ln)
            sx, sy = -math.sin(a) * wd, math.cos(a) * wd
            d.polygon([(cx + sx, cy + sy), tip, (cx - sx, cy - sy)], fill=_a(bright if long_ else color, 235))
        disc(r * 0.75, fill=_a(color, 255))
        disc(r * 0.5, fill=(255, 255, 255, 255))
    elif kind == 1:   # Terra: a rocky planet turning (its craters drift across), lit from the upper left
        disc(r, fill=_a(color, 255), outline=_a(dark, 255), width=W)
        for k, (u, v, cr) in enumerate(((0.0, -0.35, 0.28), (0.45, 0.3, 0.22), (0.2, 0.05, 0.16), (0.7, -0.15, 0.18))):
            x = ((u + f / FRAMES) % 1.0) * 2.4 - 1.2          # wraps round the planet once per loop
            if abs(x) > 0.8 - cr:
                continue
            px, py, rr = cx + x * r, cy + v * r, cr * r * (1 - abs(x) * 0.45)
            d.ellipse((px - rr, py - rr * 0.85, px + rr, py + rr * 0.85), fill=_a(dark, 200))
            d.ellipse((px - rr * 0.6 - rr * 0.25, py - rr * 0.55, px + rr * 0.6 - rr * 0.25, py + rr * 0.35), fill=_a(bright, 120))
        d.chord((cx - r, cy - r, cx + r, cy + r), 20, 200, fill=_a(dark, 110))          # night side
        d.ellipse((cx - r * 0.55, cy - r * 0.6, cx - r * 0.1, cy - r * 0.2), fill=_a(bright, 150))   # sunlit cap
    elif kind == 2:   # Darkbringer: a black hole, its tilted accretion disk turning (a hot spot circles once a loop)
        rx, ry = r * 1.75, r * 0.6
        def disk(front):
            for k in range(36):
                a0 = k * 10
                if (math.sin(math.radians(a0 + 5)) > 0) != front:
                    continue
                hot = (math.cos(math.radians(a0) - ph) + 1) / 2
                col = _mix(color, bright, hot)
                pts = [(cx + math.cos(math.radians(a)) * rx, cy + math.sin(math.radians(a)) * ry) for a in (a0, a0 + 10)]
                d.line(pts, fill=_a(col, 160 + 90 * hot), width=max(1, round(r * 0.42)))
        disk(False)
        disc(r * 0.95, fill=_a(bright, 255))                 # the photon ring
        disc(r * 0.78, fill=(6, 4, 14, 255))                 # the event horizon
        disk(True)
    elif kind == 3:   # Gale: an icy comet nucleus tumbling (45 degrees a frame) in a pale coma
        disc(r * 1.3, fill=_a(bright, 70))
        disc(r * 0.95, fill=_a(bright, 90))
        spin = f * math.pi / 4
        pts = []   # a lumpy peanut of ice, longer than wide
        for k in range(16):
            a = k * math.tau / 16
            rr = (1.0 if math.cos(a) > 0 else 0.8) * (0.62 + 0.38 * abs(math.cos(a))) * (1 + 0.12 * math.sin(3 * a))
            x, y = math.cos(a) * rr * r * 0.95, math.sin(a) * rr * r * 0.95
            pts.append((cx + x * math.cos(spin) - y * math.sin(spin), cy + x * math.sin(spin) + y * math.cos(spin)))
        d.polygon(pts, fill=_a(color, 255), outline=_a(dark, 255), width=W)
        hx, hy = cx + math.cos(spin) * r * 0.35, cy + math.sin(spin) * r * 0.35
        d.ellipse((hx - r * 0.28, hy - r * 0.22, hx + r * 0.28, hy + r * 0.22), fill=(255, 255, 255, 235))
        dx_, dy_ = cx - math.cos(spin) * r * 0.3, cy - math.sin(spin) * r * 0.3
        d.ellipse((dx_ - r * 0.14, dy_ - r * 0.14, dx_ + r * 0.14, dy_ + r * 0.14), fill=_a(dark, 200))
    elif kind == 4:   # Blood: a red giant, pulsing, its granules boiling
        rr = r * (1.08 + 0.12 * math.sin(ph))
        disc(rr * 1.18, fill=_a(color, 70))                      # its swollen outer layer
        disc(rr, fill=_mix(dark, color, 0.55))                     # limb darkening
        disc(rr * 0.82, fill=_a(color, 255))
        for k in range(5):   # granules boiling up and sinking
            a = k * math.tau / 5 + ph / 5
            q = 0.42 + 0.18 * math.sin(ph * 2 + k * 1.7)
            px, py, g_ = cx + math.cos(a) * rr * q, cy + math.sin(a) * rr * q, rr * (0.13 + 0.05 * math.sin(ph + k))
            d.ellipse((px - g_, py - g_, px + g_, py + g_), fill=_a(bright, 140))
        disc(rr * 0.3, fill=_a(bright, 240))
    elif kind == 5:   # Rift: a spiral wormhole, three arms turning 15 degrees a frame (the pattern repeats every 120)
        disc(r * 1.15, fill=_a(dark, 150))
        for arm in range(3):
            pts = []
            for k in range(14):
                t = k / 13
                a = arm * math.tau / 3 + f * math.radians(15) + t * 2.6
                rr = r * (0.25 + 0.9 * t)
                pts.append((cx + math.cos(a) * rr, cy + math.sin(a) * rr))
            d.line(pts, fill=_a(color, 255), width=max(1, round(r * 0.3)))
            d.line(pts[:8], fill=_a(bright, 255), width=max(1, round(r * 0.15)))
        disc(r * 0.3, fill=(4, 6, 20, 255))
    else:   # Emperor: a golden sun, six flares rising and settling round its rim, turning 7.5 degrees a frame
        for k in range(6):
            a = k * math.tau / 6 + f * math.radians(7.5)
            h = r * (1.35 + 0.25 * math.sin(ph * 2 + k * 2))
            d.polygon([(cx + math.cos(a - 0.3) * r * 0.9, cy + math.sin(a - 0.3) * r * 0.9),
                       (cx + math.cos(a) * h, cy + math.sin(a) * h),
                       (cx + math.cos(a + 0.3) * r * 0.9, cy + math.sin(a + 0.3) * r * 0.9)], fill=_a(color, 230))
        disc(r, fill=_a(color, 255), outline=_a(dark, 255), width=W)
        disc(r * 0.62, fill=_a(bright, 255))
        disc(r * 0.28, fill=(255, 255, 255, 255))


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
    length, width = TAIL[rank] * (1.4 if kind == 3 else 1.0), 1.9 + 0.15 * min(rank, 7)   # Gale's comet tail is longer
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
    # --- the sword's celestial body (the centre of its solar system at Imperial)
    celestial(d, kind, c, c, g * (1.25 if imperial else 1.0), f, colors)
    for b in bodies:
        if b[0] >= 0:
            body(*b)
    return im.resize((2 * H, 2 * H), Image.Resampling.LANCZOS)


def orbit_body_frame(kind: int, phase: int, selected: bool, colors, ring) -> Image.Image:
    """Round 97: at Imperial a sword guarding an ally circles it as its celestial body (128 x 128 like the arsenal
    ring's frames; `ring` = the generator's (SLOT_RX, SLOT_RY, RING_Y)): it sways round its slot with a short arc of
    light behind, the far side of the ring smaller and dimmer."""
    slot_rx, slot_ry, ring_y = ring
    S = 128 * SS
    im = Image.new("RGBA", (S, S))
    d = ImageDraw.Draw(im, "RGBA")
    dark, color, bright = colors[kind]
    sway = math.sin((phase / 8) * math.tau + kind * 0.9) * 0.22 * (math.tau / 7)
    a = -math.pi / 2 + kind * math.tau / 7 + sway
    x = (64 + math.cos(a) * slot_rx) * SS
    y = (64 + ring_y + math.sin(a) * slot_ry + 1.6 * math.cos((phase / 8) * math.tau + kind * 0.9)) * SS
    back = math.sin(a) < -0.2
    r = (3.6 if back else 4.6) * SS
    for j in range(6):   # the arc of light along its path
        aa = a - (j + 1) * 0.08
        px, py = (64 + math.cos(aa) * slot_rx) * SS, (64 + ring_y + math.sin(aa) * slot_ry) * SS
        q = (1.6 - j * 0.2) * SS
        d.ellipse((px - q, py - q, px + q, py + q), fill=_a(color, (150 - j * 24) * (0.6 if back else 1)))
    glow = Image.new("RGBA", (S, S))
    ImageDraw.Draw(glow, "RGBA").ellipse((x - r * 2, y - r * 2, x + r * 2, y + r * 2), fill=_a(bright if selected else color, 110))
    im = Image.alpha_composite(im, glow.filter(ImageFilter.GaussianBlur(SS * 2)))
    body = Image.new("RGBA", (S, S))
    celestial(ImageDraw.Draw(body, "RGBA"), kind, x, y, r, phase, colors)
    if back:
        body.putalpha(body.getchannel("A").point(lambda v: v * 165 // 255))
    im = Image.alpha_composite(im, body)
    return im.resize((128, 128), Image.Resampling.LANCZOS)


# ------------------------------------------------------------------ round 97: Imperial's black hole and the wormholes

HOLE_W, HOLE_H = 72, 128          # a buff frame centred on him (taller than the badges, so the hole sits above his head)
HOLE_C = (30, 27)                 # the event horizon's centre: above his head, left of the mastery sigil
HOLE_FRAMES = 16
RAINBOW = ((255, 90, 90), (255, 170, 70), (255, 235, 90), (110, 230, 120), (90, 200, 255), (130, 120, 255), (220, 110, 255))


def _rainbow(t, a=255):
    t = t % 1.0 * 7
    k = int(t)
    c1, c2 = RAINBOW[k], RAINBOW[(k + 1) % 7]
    return tuple(int(c1[j] + (c2[j] - c1[j]) * (t - k)) for j in range(3)) + (int(a),)


def blackhole_frame(inside: int, phase: int, colors, prismatic: bool = False) -> Image.Image:
    """Round 97: the black hole Imperial keeps above his head, his swords inside it. A gravitational-lens halo, a tilted
    accretion disk whose hot spot circles once per loop (the near side over the horizon, the far side lensed up over its
    top), a photon ring, the black horizon, and `inside` glints in the sword colours spiralling in the disk. #1's disk
    is prismatic. HOLE_FRAMES frames, everything turning a whole share per frame."""
    S = SS
    im = Image.new("RGBA", (HOLE_W * S, HOLE_H * S))
    cx, cy = HOLE_C[0] * S, HOLE_C[1] * S
    t = phase / HOLE_FRAMES
    spin = t * math.tau
    rx, ry, horizon = 16.5 * S, 5.2 * S, 5.2 * S
    disk_col = lambda a, heat: (_rainbow(a / math.tau + t, 255) if prismatic else
                                _mix(_mix((150, 70, 230, 255), (255, 160, 90, 255), heat), (255, 245, 220, 255), heat ** 3))
    # the lensing halo and a faint swirl of light round it
    glow = Image.new("RGBA", im.size)
    gd = ImageDraw.Draw(glow, "RGBA")
    gd.ellipse((cx - 13 * S, cy - 11 * S, cx + 13 * S, cy + 11 * S), fill=(120, 70, 220, 70))
    gd.ellipse((cx - rx, cy - ry * 1.4, cx + rx, cy + ry * 1.4), fill=(255, 170, 120, 60))
    im = Image.alpha_composite(im, glow.filter(ImageFilter.GaussianBlur(4 * S)))
    d = ImageDraw.Draw(im, "RGBA")

    def disk(front: bool, lensed: bool = False):
        for k in range(72):
            a0 = k * math.tau / 72
            if lensed:
                if math.sin(a0) > 0:
                    continue
            elif (math.sin(a0 + math.pi / 72) > 0) != front:
                continue
            heat = (math.cos(a0 - spin) + 1) / 2                  # the hot spot circles once a loop
            doppler = 0.75 + 0.25 * math.cos(a0)                  # the side coming toward us is brighter
            for ring, w in ((1.0, 2.4), (0.78, 1.6)):
                pts = []
                for a in (a0, a0 + math.tau / 72 + 0.01):
                    x = cx + math.cos(a) * rx * ring
                    y = cy + math.sin(a) * ry * ring
                    if lensed:   # the far side of the disk bent up over the top of the hole
                        y = cy - abs(math.sin(a)) * horizon * 1.9 * ring - 0.6 * S
                        x = cx + math.cos(a) * horizon * 1.55 * ring
                    pts.append((x, y))
                a_ = (150 if lensed else 255) * doppler * (0.8 if ring < 1 else 1)
                d.line(pts, fill=_a(disk_col(a0, heat * (1 if ring == 1.0 else 1.15)), a_), width=round(w * S))

    disk(False)
    disk(False, lensed=True)
    d.ellipse((cx - horizon - S, cy - horizon - S, cx + horizon + S, cy + horizon + S),
              outline=(255, 235, 200, 230) if not prismatic else _rainbow(t, 230), width=S)     # the photon ring
    d.ellipse((cx - horizon, cy - horizon, cx + horizon, cy + horizon), fill=(3, 2, 8, 255))  # the event horizon
    # the swords inside: glints in their colours spiralling round the disk (22.5 degrees a frame)
    glints = []
    for j in range(inside):
        a = j * math.tau / max(inside, 1) + spin + j * 0.4
        q = 0.62 + 0.3 * ((math.sin(spin * 2 + j * 1.3) + 1) / 2)
        x, y = cx + math.cos(a) * rx * q, cy + math.sin(a) * ry * q
        glints.append((math.sin(a), x, y, colors[j % 7]))
    for depth, x, y, (dk, c, b) in sorted(glints):
        if depth >= 0:
            continue
        s = 1.2 * S
        d.ellipse((x - s, y - s, x + s, y + s), fill=_a(c, 150))
    disk(True)
    for depth, x, y, (dk, c, b) in sorted(glints):
        if depth < 0:
            continue
        s = 1.6 * S
        d.ellipse((x - s * 1.8, y - s * 1.8, x + s * 1.8, y + s * 1.8), fill=_a(c, 90))
        d.ellipse((x - s, y - s, x + s, y + s), fill=_a(b, 255))
    return im.resize((HOLE_W, HOLE_H), Image.Resampling.LANCZOS)


WORM = 64             # wormhole effect canvas, px, centred on the point
WORM_FRAMES = 14


def wormhole_frame(kind: int | None, k: int, arriving: bool, colors) -> Image.Image:
    """Round 97: a wormhole opening where a sword leaves (`arriving` False) or arrives (True), in the sword's colour
    (`kind` None = prismatic, Imperial #1). WORM_FRAMES frames:
      0-1   a pinpoint of light flashes,
      2-5   a tilted portal tears open: a dark throat, a bright rim, three spiral arms winding in, star specks pulled
            along them,
      6-8   leaving: the body bursts out in a flare and a lensing ring ripples away;
            arriving: streaks of light are dragged in and the body is swallowed into the throat,
      9-11  the portal winds shut, spinning faster,
      12-13 it collapses to a spark (leaving) or implodes in a ring of light (arriving)."""
    S = SS
    im = Image.new("RGBA", (WORM * S, WORM * S))
    c = WORM * S / 2
    if kind is None:
        dark, color, bright = (40, 10, 70, 255), _rainbow(k / WORM_FRAMES), (255, 255, 255, 255)
    else:
        dark, color, bright = colors[kind]
    glow = Image.new("RGBA", im.size)
    gd = ImageDraw.Draw(glow, "RGBA")
    d = ImageDraw.Draw(im, "RGBA")
    open_ = {0: 0.0, 1: 0.12, 2: 0.35, 3: 0.6, 4: 0.85, 5: 1.0, 6: 1.0, 7: 1.0, 8: 0.95, 9: 0.75, 10: 0.5, 11: 0.28,
             12: 0.08, 13: 0.0}[k]
    rx = 15 * S * open_
    ry = rx * 0.58
    spin = k * 0.55 + (k - 8) * 0.35 * (k > 8)                    # winds faster as it shuts
    # 0-1: the pinpoint flash
    if k <= 2:
        s = (1.5 + 2.0 * k) * S
        g_ = s * (2.5 if k < 2 else 1.6)
        gd.ellipse((c - g_, c - g_, c + g_, c + g_), fill=_a(color, 200 if k < 2 else 120))
        for a in range(4):
            ang = a * math.pi / 2 + math.pi / 4 * (k % 2)
            ln = (5 + 4 * k) * S
            d.line([(c - math.cos(ang) * ln, c - math.sin(ang) * ln), (c + math.cos(ang) * ln, c + math.sin(ang) * ln)],
                   fill=_a(bright, 230), width=S)
        d.ellipse((c - s * 0.6, c - s * 0.6, c + s * 0.6, c + s * 0.6), fill=(255, 255, 255, 255))
    if rx > 0.5 * S:
        # the outer glow and the throat
        gd.ellipse((c - rx * 1.5, c - ry * 1.6, c + rx * 1.5, c + ry * 1.6), fill=_a(color, 150))
        d.ellipse((c - rx, c - ry, c + rx, c + ry), fill=_a(_mix(dark, (0, 0, 0, 255), 0.6), 245))
        # three spiral arms winding in
        for arm in range(3):
            pts = []
            for j in range(16):
                t = j / 15
                a = arm * math.tau / 3 + spin + t * 3.2
                pts.append((c + math.cos(a) * rx * (1 - 0.85 * t), c + math.sin(a) * ry * (1 - 0.85 * t)))
            d.line(pts, fill=_a(color, 230), width=max(1, round(S * 1.4)))
            d.line(pts[2:9], fill=_a(bright, 220), width=S)
        # star specks pulled along the arms
        for j in range(7):
            t = ((j * 0.37 + k * 0.09) % 1.0)
            a = j * 0.9 + spin * 1.3 + t * 3.0
            x, y = c + math.cos(a) * rx * (1.25 - t), c + math.sin(a) * ry * (1.25 - t)
            s = 0.8 * S
            d.ellipse((x - s, y - s, x + s, y + s), fill=(255, 255, 255, int(255 * (1 - t))))
        # the rim
        d.ellipse((c - rx, c - ry, c + rx, c + ry), outline=_a(bright, 255), width=max(1, round(S * 1.3)))
        d.ellipse((c - rx * 1.12, c - ry * 1.12, c + rx * 1.12, c + ry * 1.12), outline=_a(color, 160), width=S)
        d.ellipse((c - S, c - S, c + S, c + S), fill=(0, 0, 0, 255))
    if not arriving and 6 <= k <= 8:
        # the body bursts out: a flare, and a lensing ring rippling away
        e = k - 6
        s = (5 - e * 1.2) * S
        gd.ellipse((c - s * 2.6, c - s * 2.6, c + s * 2.6, c + s * 2.6), fill=_a(bright, 240 - 50 * e))
        d.ellipse((c - s, c - s, c + s, c + s), fill=(255, 255, 255, 255 - 60 * e))
        for a in range(8):
            ang = a * math.pi / 4 + 0.3
            r0, r1 = (6 + 4 * e) * S, (11 + 7 * e) * S
            d.line([(c + math.cos(ang) * r0, c + math.sin(ang) * r0 * 0.7), (c + math.cos(ang) * r1, c + math.sin(ang) * r1 * 0.7)],
                   fill=_a(bright, 220 - 60 * e), width=S)
    if not arriving and 7 <= k <= 11:
        e = (k - 7) / 4
        R = (14 + 14 * e) * S
        d.ellipse((c - R, c - R * 0.62, c + R, c + R * 0.62), outline=_a(bright, 200 * (1 - e)), width=max(1, round(S * (1.5 - e))))
    if arriving and 5 <= k <= 9:
        # streaks dragged in, the body swallowed into the throat
        e = (k - 5) / 4
        for a in range(10):
            ang = a * math.tau / 10 + spin * 0.6
            r0 = (28 - 20 * e) * S
            r1 = r0 * (0.55 + 0.25 * e)
            d.line([(c + math.cos(ang) * r0, c + math.sin(ang) * r0 * 0.62), (c + math.cos(ang) * r1, c + math.sin(ang) * r1 * 0.62)],
                   fill=_a(bright, 120 + 100 * e), width=S)
        s = (4.5 - 3.5 * e) * S
        gd.ellipse((c - s * 2.2, c - s * 2.2, c + s * 2.2, c + s * 2.2), fill=_a(color, 220))
        d.ellipse((c - s, c - s, c + s, c + s), fill=(255, 255, 255, 255))
    if k >= 12:
        if arriving:   # implodes in a ring of light
            e = k - 12
            R = (6 + 9 * e) * S
            d.ellipse((c - R, c - R * 0.62, c + R, c + R * 0.62), outline=_a(bright, 230 - 90 * e), width=max(1, round(S * 1.6)))
            gd.ellipse((c - 3 * S, c - 3 * S, c + 3 * S, c + 3 * S), fill=_a(color, 200 - 80 * e))
        else:          # collapses to a final spark
            s = (2.2 - 0.9 * (k - 12)) * S
            d.line([(c - s * 3, c), (c + s * 3, c)], fill=_a(bright, 230), width=S)
            d.line([(c, c - s * 3), (c, c + s * 3)], fill=_a(bright, 230), width=S)
            d.ellipse((c - s, c - s, c + s, c + s), fill=(255, 255, 255, 240))
    im = Image.alpha_composite(glow.filter(ImageFilter.GaussianBlur(2.5 * S)), im)
    return im.resize((WORM, WORM), Image.Resampling.LANCZOS)
