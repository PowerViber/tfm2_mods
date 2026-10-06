"""Aegis Zero, redrawn after Destiny Gundam (round 88, Rian's brief).

Body (champions/tfm2_gundam_aegis_zero): 48 x 56 frames on the sprite kit's rows (frame centre = his position, feet on
y 48, facing right). A tall, athletic white / royal-blue / red knight with a gold V-fin and green eyes; folded red
mechanical wings, the long Arondight hilt and a folded cannon on the backpack. "Gundam first, wings second."

Effects (vfx/gundam): the Wings of Light, a separate layer behind him while the ult runs: hundreds of sharp energy
shards, pale-pink core -> pink -> magenta -> red edges, a huge V of two upper wings with smaller lower projections, the
mechanical red wings visible at their roots ("wings first, Gundam at their centre"), about 3x his width. Plus the
deploy / retract, the cyan palm blast, the beam-saber slash, the S2 sweep, the charge burst and afterimages, the landing
shockwave and the ground rings (1 px = 950 game units).

Run from the repo root: python3 mods/tfm2_gundam/source/build_art.py  (writes into mods/tfm2_custom, previews here)
"""
from pathlib import Path
import json
import math
import random
from PIL import Image, ImageDraw

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
KIT = ROOT / "Sprite kit"
OUT = ROOT / "mods" / "tfm2_custom"
ID = "tfm2_gundam_aegis_zero"

P = {
    "ink": (20, 22, 38, 255),
    "frame": (46, 52, 70, 255),
    "frame_hi": (86, 94, 112, 255),
    "white": (246, 247, 250, 255),
    "white_mid": (205, 212, 224, 255),
    "white_dark": (140, 150, 170, 255),
    "blue": (40, 74, 168, 255),
    "blue_hi": (78, 118, 214, 255),
    "blue_dark": (26, 42, 104, 255),
    "red": (214, 38, 52, 255),
    "red_hi": (250, 92, 92, 255),
    "red_dark": (128, 20, 38, 255),
    "gold": (255, 206, 64, 255),
    "gold_dark": (190, 132, 30, 255),
    "eye": (120, 255, 170, 255),
    "cyan": (110, 230, 255, 255),
    "cyan_hi": (220, 252, 255, 255),
    "pink": (255, 120, 196, 255),
    "pink_hi": (255, 214, 236, 255),
}


def C(name, a=None):
    c = P[name]
    return c if a is None else c[:3] + (a,)


def poly(d, pts, fill, edge="ink"):
    d.polygon([(round(x), round(y)) for x, y in pts], fill=C(fill), outline=C(edge) if edge else None)


def line(d, pts, color, w=1):
    d.line([(round(x), round(y)) for x, y in pts], fill=C(color) if isinstance(color, str) else color, width=w)


def dot(d, x, y, color):
    d.point((round(x), round(y)), fill=C(color) if isinstance(color, str) else color)


def quad(a, b, wa, wb):
    """A limb segment from a to b, wa wide at a and wb at b."""
    (x0, y0), (x1, y1) = a, b
    l = math.hypot(x1 - x0, y1 - y0) or 1
    nx, ny = -(y1 - y0) / l, (x1 - x0) / l
    return [(x0 + nx * wa / 2, y0 + ny * wa / 2), (x1 + nx * wb / 2, y1 + ny * wb / 2),
            (x1 - nx * wb / 2, y1 - ny * wb / 2), (x0 - nx * wa / 2, y0 - ny * wa / 2)]


def at(p, ang, length):
    a = math.radians(ang)
    return (p[0] + math.cos(a) * length, p[1] + math.sin(a) * length)


# ------------------------------------------------------------------ the body

def folded_wings(d, sx, sy, spread=0.0, glow=0):
    """The backpack: dense dark machinery behind the shoulders and two folded red wing assemblies on rotating joints,
    standing up behind the shoulders above head height. spread 0 folded .. 1 half open (the ult wind-up)."""
    poly(d, [(sx - 7, sy - 1), (sx + 7, sy - 1), (sx + 6, sy + 9), (sx - 6, sy + 9)], "frame")
    for side in (-1, 1):
        jx, jy = sx + side * 6, sy
        a0 = -90 + side * (16 + 40 * spread)           # the main spar
        tip = at((jx, jy), a0, 17 + 2 * spread)
        knee = at((jx, jy), a0 + side * 8, 9)
        # the long red wing blade along the spar, and a shorter second blade outside it
        poly(d, [(jx - side, jy), knee, tip, at((jx, jy), a0 + side * 20, 12), at((jx, jy), a0 + side * 34, 7)], "red")
        poly(d, [at((jx, jy), a0 + side * 34, 7), at((jx, jy), a0 + side * 52, 12), at((jx, jy), a0 + side * 60, 8)], "red_dark")
        line(d, [(jx, jy), tip], "frame")
        line(d, [knee, at((jx, jy), a0 + side * 20, 12)], "red_dark")
        dot(d, *tip, "white")
        d.ellipse((jx - 2, jy - 2, jx + 2, jy + 2), fill=C("frame"), outline=C("ink"))
        dot(d, jx, jy, "frame_hi")
        if glow:   # thruster glow under the wing roots
            for k in range(glow):
                dot(d, jx + side * 2, jy + 8 + k, "pink" if k == glow - 1 else "pink_hi")


def arondight_stowed(d, sx, sy):
    """The long sword folded along the backpack: its hilt and guard stick up over his right shoulder."""
    line(d, [(sx + 4, sy + 6), (sx + 9, sy - 9)], "ink", 3)
    line(d, [(sx + 4, sy + 6), (sx + 9, sy - 9)], "white_dark", 1)
    line(d, [(sx + 6, sy - 4), (sx + 11, sy - 2)], "frame", 2)
    dot(d, sx + 9, sy - 9, "red")


def leg(d, hip, knee, ankle, front):
    thigh = "white" if front else "white_mid"
    poly(d, quad(hip, knee, 6, 5), thigh)
    poly(d, quad(knee, ankle, 5, 6), thigh)
    # sharp knee armour, blue calf panel, red ankle
    kx, ky = knee
    poly(d, [(kx - 2, ky - 2), (kx + 3, ky - 1), (kx + 2, ky + 2), (kx - 2, ky + 2)], "white_mid" if front else "white_dark")
    dot(d, kx + 2, ky - 1, "white")
    mid = ((knee[0] + ankle[0]) / 2, (knee[1] + ankle[1]) / 2 + 1)
    line(d, [(mid[0] - 1, mid[1]), (mid[0] + 1, mid[1] + 1)], "blue" if front else "blue_dark")
    ax, ay = ankle
    # the big angular foot, toe forward (right), red sole and toe
    poly(d, [(ax - 3, ay - 1), (ax + 2, ay - 1), (ax + 6, ay + 1), (ax + 6, 48), (ax - 4, 48)], "white_mid" if front else "white_dark")
    line(d, [(ax - 4, 48), (ax + 6, 48)], "red" if front else "red_dark")
    dot(d, ax + 5, 47, "red" if front else "red_dark")


def arm(d, shoulder, elbow, hand, front, palm=0, fist_open=False):
    upper = "white" if front else "white_mid"
    poly(d, quad(shoulder, elbow, 4, 4), "frame")
    poly(d, quad(elbow, hand, 5, 6), upper)
    line(d, [elbow, ((elbow[0] + hand[0]) / 2, (elbow[1] + hand[1]) / 2)], "blue" if front else "blue_dark")
    hx, hy = hand
    d.rectangle((hx - 1, hy - 1, hx + 1, hy + 1), fill=C("frame_hi"), outline=C("ink"))
    if fist_open:
        dot(d, hx + 2, hy - 1, "frame_hi")
        dot(d, hx + 2, hy + 1, "frame_hi")
    if palm:
        d.ellipse((hx - palm, hy - palm, hx + palm, hy + palm), outline=C("cyan"))
        dot(d, hx, hy, "cyan_hi")
        dot(d, hx + 1, hy, "cyan")


def pauldron(d, cx, cy, front):
    poly(d, [(cx - 5, cy + 1), (cx - 3, cy - 4), (cx + 4, cy - 4), (cx + 6, cy), (cx + 4, cy + 4), (cx - 4, cy + 4)],
         "white" if front else "white_mid")
    poly(d, [(cx - 3, cy - 4), (cx + 4, cy - 4), (cx + 3, cy - 2), (cx - 2, cy - 2)], "blue" if front else "blue_dark")
    line(d, [(cx - 4, cy + 3), (cx + 4, cy + 3)], "red" if front else "red_dark")


def torso(d, sx, sy, hx, hy):
    # chest: royal blue core in white plating, red collar points, gold vents; tapering to a very narrow red waist
    poly(d, [(sx - 8, sy), (sx + 8, sy), (sx + 6, sy + 6), (hx + 3, hy - 3), (hx - 3, hy - 3), (sx - 6, sy + 6)], "white")
    poly(d, [(sx - 5, sy + 1), (sx + 5, sy + 1), (sx + 4, sy + 6), (sx, sy + 7), (sx - 4, sy + 6)], "blue")
    line(d, [(sx - 4, sy + 2), (sx + 4, sy + 2)], "blue_hi")
    poly(d, [(sx - 8, sy), (sx - 5, sy - 1), (sx - 4, sy + 2)], "red")
    poly(d, [(sx + 8, sy), (sx + 5, sy - 1), (sx + 4, sy + 2)], "red")
    for vx in (sx - 3, sx + 2):
        line(d, [(vx, sy + 4), (vx + 1, sy + 4)], "gold")
    poly(d, [(hx - 3, hy - 4), (hx + 3, hy - 4), (hx + 2, hy - 1), (hx - 2, hy - 1)], "red")
    # skirt: overlapping white plates with red trim, front plate slightly forward
    poly(d, [(hx - 6, hy - 1), (hx + 6, hy - 1), (hx + 7, hy + 4), (hx + 2, hy + 3), (hx, hy + 5), (hx - 2, hy + 3), (hx - 7, hy + 4)], "white")
    line(d, [(hx - 6, hy + 3), (hx - 3, hy + 2)], "red")
    line(d, [(hx + 3, hy + 2), (hx + 6, hy + 3)], "red")
    dot(d, hx, hy + 1, "gold")


def head(d, cx, cy, look=1):
    # small white helmet, navy crown, gold V-fin, angular faceplate, red chin, narrow green eyes (turned right)
    poly(d, [(cx - 3, cy - 3), (cx + 3, cy - 3), (cx + 4, cy + 1), (cx + 2, cy + 4), (cx - 2, cy + 4), (cx - 4, cy + 1)], "white")
    poly(d, [(cx - 2, cy - 4), (cx + 2, cy - 4), (cx + 3, cy - 2), (cx - 3, cy - 2)], "blue_dark")
    line(d, [(cx, cy - 3), (cx - 5, cy - 8)], "gold")
    line(d, [(cx, cy - 3), (cx + 6, cy - 8)], "gold")
    dot(d, cx - 5, cy - 8, "gold_dark")
    dot(d, cx + 6, cy - 8, "gold_dark")
    dot(d, cx, cy - 3, "red")
    line(d, [(cx - 2 + look, cy), (cx + look, cy)], "eye")
    dot(d, cx + 2 + look, cy, "eye")
    line(d, [(cx - 1, cy + 3), (cx + 1, cy + 3)], "red")


def draw(pose):
    """pose: dict of joint positions (see POSES). Draws one 48 x 56 frame."""
    im = Image.new("RGBA", (48, 56))
    d = ImageDraw.Draw(im)
    by = pose.get("bob", 0)
    lean = pose.get("lean", 0)
    sx, sy = 24 + lean, 20 + by          # shoulder line centre
    hx, hy = 24 + lean // 2, 31 + by     # waist
    folded_wings(d, sx - 1, sy, pose.get("spread", 0.0), pose.get("glow", 0))
    arondight_stowed(d, sx + 3, sy - 2)
    # back limbs
    bl = pose["back_leg"]
    leg(d, (hx - 3, hy + 2), bl[0], bl[1], False)
    ba = pose["back_arm"]
    arm(d, (sx - 8, sy + 2), ba[0], ba[1], False)
    pauldron(d, sx - 9, sy + 1, False)
    torso(d, sx, sy, hx, hy)
    fl = pose["front_leg"]
    leg(d, (hx + 3, hy + 2), fl[0], fl[1], True)
    head(d, sx + 1, sy - 6 + pose.get("head_dy", 0), pose.get("look", 1))
    pauldron(d, sx + 9, sy + 1, True)
    fa = pose["front_arm"]
    if pose.get("blade"):                # a beam saber (attack) or Arondight (skill2) in the front hand
        (x0, y0), (x1, y1) = fa[1], pose["blade"]
        col = "pink" if pose.get("blade_kind") == "beam" else "white_mid"
        line(d, [(x0, y0), (x1, y1)], "ink", 3)
        line(d, [(x0, y0), (x1, y1)], col, 1 if pose.get("blade_kind") == "beam" else 2)
        if pose.get("blade_kind") == "beam":
            line(d, [(x0, y0), (x1, y1)], (255, 214, 236, 255), 1)
    arm(d, (sx + 8, sy + 2), fa[0], fa[1], True, pose.get("palm", 0), pose.get("open", False))
    return im


def stand(by=0, lean=0, stride=0, lift=0):
    """Legs for standing / running: stride shifts feet, lift raises the back foot."""
    hx = 24 + lean // 2
    return {
        "back_leg": ((hx - 3 - stride // 2, 40 + by - (lift > 0)), (hx - 4 - stride, 46 - lift)),
        "front_leg": ((hx + 3 + stride // 2, 40 + by), (hx + 3 + stride, 46)),
    }


def arms_rest(by=0, lean=0, swing=0):
    sx, sy = 24 + lean, 20 + by
    return {"back_arm": ((sx - 10 - swing // 2, sy + 9), (sx - 10 - swing, sy + 14)),
            "front_arm": ((sx + 10 + swing // 2, sy + 9), (sx + 11 + swing, sy + 14))}


def P_(**kw):
    by, lean = kw.get("bob", 0), kw.get("lean", 0)
    p = {"bob": by, "lean": lean}
    p.update(stand(by, lean, kw.get("stride", 0), kw.get("lift", 0)))
    p.update(arms_rest(by, lean, kw.get("swing", 0)))
    for k in ("spread", "glow", "palm", "blade", "blade_kind", "head_dy", "look", "open"):
        if k in kw:
            p[k] = kw[k]
    for k in ("back_arm", "front_arm", "back_leg", "front_leg"):
        if k in kw:
            p[k] = kw[k]
    return p


def poses():
    A = {}
    A["idle"] = [P_(bob=b, glow=g) for b, g in ((0, 1), (0, 2), (1, 2), (1, 1))]
    run = []
    for i in range(6):
        ph = i / 6 * math.tau
        st = round(math.sin(ph) * 4)
        run.append(P_(bob=abs(round(math.sin(ph) * 1)), lean=2, stride=st, swing=-st // 2, lift=1 if st < 0 else 0, glow=2))
    A["run"] = run
    # basic attack: a beam-saber slash (windup, swing, follow-through, recover)
    A["attack"] = [
        P_(lean=-1, front_arm=((33, 22), (30, 16)), blade=(24, 8), blade_kind="beam"),
        P_(lean=2, front_arm=((36, 24), (41, 22)), blade=(47, 12), blade_kind="beam"),
        P_(lean=3, front_arm=((36, 28), (40, 32)), blade=(46, 42), blade_kind="beam"),
        P_(lean=1, front_arm=((35, 28), (37, 33))),
    ]
    # skill1, the charge: thrusters flare, he leans in, the open palm glows cyan (Palma Fiocina)
    A["skill1"] = [
        P_(lean=1, spread=0.25, glow=3, front_arm=((35, 27), (34, 32)), open=True, palm=1),
        P_(lean=4, spread=0.4, glow=4, front_arm=((38, 22), (43, 22)), open=True, palm=2, stride=2),
        P_(lean=5, spread=0.45, glow=4, front_arm=((39, 22), (45, 22)), open=True, palm=3, stride=3, lift=1),
        P_(lean=3, spread=0.3, glow=3, front_arm=((38, 23), (43, 23)), open=True, palm=2, stride=2),
    ]
    # skill2: Arondight drawn over the head, then a wide cut
    A["skill2"] = [
        P_(lean=-1, front_arm=((32, 13), (30, 8)), back_arm=((19, 14), (26, 8)), blade=(14, 2), blade_kind="steel", head_dy=0),
        P_(lean=1, front_arm=((35, 15), (38, 10)), back_arm=((20, 16), (30, 12)), blade=(47, 0), blade_kind="steel"),
        P_(lean=3, front_arm=((37, 22), (42, 22)), back_arm=((21, 22), (31, 22)), blade=(48, 30), blade_kind="steel"),
        P_(lean=2, front_arm=((36, 28), (39, 33)), back_arm=((20, 28), (28, 32)), blade=(46, 44), blade_kind="steel"),
    ]
    # ult: arms out, chest up, the wings start to open (the light wings take over on top of this)
    A["ult"] = [
        P_(spread=0.2, glow=2, head_dy=-1),
        P_(spread=0.5, glow=3, head_dy=-1, front_arm=((36, 19), (41, 16)), back_arm=((14, 19), (9, 16)), open=True),
        P_(spread=0.8, glow=4, head_dy=-1, front_arm=((37, 17), (43, 13)), back_arm=((13, 17), (7, 13)), open=True),
        P_(spread=1.0, glow=4, head_dy=-1, front_arm=((37, 17), (43, 13)), back_arm=((13, 17), (7, 13)), open=True),
    ]
    A["hit"] = [P_(lean=-3, head_dy=1, front_arm=((33, 27), (33, 31)))]
    return A


def body_frame(tag, i, A):
    if tag == "dead":
        base = draw(A["hit"][0])
        ang = (0, -12, -30, -52, -76, -90)[i]
        r = base.rotate(ang, resample=Image.Resampling.NEAREST, expand=True)
        r = r.crop(r.getbbox())
        out = Image.new("RGBA", (48, 56))
        out.alpha_composite(r, (max(0, (48 - r.width) // 2), 49 - r.height))
        return out
    return draw(A[tag][i])


# ------------------------------------------------------------------ effects

def lerp(a, b, t):
    return tuple(int(a[k] + (b[k] - a[k]) * t) for k in range(4))


WING_CORE, WING_MID, WING_EDGE, WING_DEEP = (255, 236, 246, 255), (255, 120, 196, 255), (236, 40, 140, 255), (190, 20, 70, 255)


def shard(im, root, ang, length, width, alpha=1.0, bright=1.0):
    """One sharp energy feather: a long thin triangle with a pale core and magenta / red edges."""
    d = ImageDraw.Draw(im, "RGBA")
    a = math.radians(ang)
    ux, uy = math.cos(a), math.sin(a)
    nx, ny = -uy, ux
    tip = (root[0] + ux * length, root[1] + uy * length)
    for layer, (w, col) in enumerate(((width, WING_DEEP), (width * 0.7, WING_EDGE), (width * 0.42, WING_MID), (width * 0.18, WING_CORE))):
        if w < 0.35:
            continue
        base = (root[0] + ux * length * 0.08 * layer, root[1] + uy * length * 0.08 * layer)
        pts = [(base[0] + nx * w, base[1] + ny * w), tip, (base[0] - nx * w, base[1] - ny * w)]
        c = col[:3] + (int(min(255, col[3] * alpha * (0.75 + 0.25 * bright) * (0.75 if layer == 0 else 1.0))),)
        d.polygon(pts, fill=c)


def wings_of_light(f=0, scale=1.0, alpha=1.0, flicker=False):
    """The Wings of Light: 144 x 128, his position at the centre, the wing joints 8 px above it (behind the shoulders).
    Each upper wing: shards grow from the red mechanical spar out to a sharp contour (a huge V, about 3x his width);
    smaller lower projections sweep down and out."""
    W, H = 144, 128
    im = Image.new("RGBA", (W, H))
    cx, cy = W / 2, H / 2 - 8
    rnd = random.Random(1000 + f * 7)
    scale *= 0.82   # about 3x his width (Rian's brief: 2.5-3x)
    flap = math.sin(f / 8 * math.tau)
    S = lambda x, y, side: (cx + side * x * scale, cy + y * scale)
    d = ImageDraw.Draw(im, "RGBA")
    for side in (-1, 1):
        joint = S(5, 0, side)
        spar = [S(5, 0, side), S(14, -12 - flap, side), S(22, -26 - flap * 1.5, side)]
        # the outer contour the shards reach for: top of the V, the wing tip, then back down to the shoulder
        contour = [S(18, -60 - flap * 2, side), S(40, -62 - flap * 3, side), S(64, -46 - flap * 2, side),
                   S(68, -24 - flap, side), S(54, -6, side), S(34, 4, side)]
        # a translucent wing body first, so the shards read as one wing
        n = 30
        for k in range(n):
            u = rnd.uniform(0.15, 1.0)
            a_ = spar[0] if u < 0.5 else spar[1]
            b_ = spar[1] if u < 0.5 else spar[2]
            t = (u * 2) % 1 if u < 0.5 else (u - 0.5) * 2
            start = (a_[0] + (b_[0] - a_[0]) * t, a_[1] + (b_[1] - a_[1]) * t)
            v = rnd.random() * (len(contour) - 1)
            i0 = int(v)
            q = v - i0
            end = (contour[i0][0] + (contour[i0 + 1][0] - contour[i0][0]) * q, contour[i0][1] + (contour[i0 + 1][1] - contour[i0][1]) * q)
            reach = rnd.uniform(0.82, 1.04) * (0.7 if flicker and rnd.random() < 0.35 else 1.0)
            length = math.hypot(end[0] - start[0], end[1] - start[1]) * reach
            ang = math.degrees(math.atan2(end[1] - start[1], end[0] - start[0]))
            tipness = 1 - abs(v - 2) / 3     # the shards toward the wing tip are the longest and brightest
            shard(im, start, ang, length, max(1.2, (3.6 + 2.2 * (1 - tipness) + rnd.uniform(-0.6, 0.6)) * scale), alpha * (0.85 + 0.15 * tipness), 0.5 + 0.5 * tipness)
        # lower projections
        low = [S(16, 34, side), S(34, 30, side), S(44, 18, side)]
        for k in range(11):
            v = rnd.random() * 2
            i0 = int(v)
            q = v - i0
            end = (low[i0][0] + (low[i0 + 1][0] - low[i0][0]) * q, low[i0][1] + (low[i0 + 1][1] - low[i0][1]) * q)
            start = S(6 + rnd.uniform(0, 4), 6 + rnd.uniform(0, 3), side)
            length = math.hypot(end[0] - start[0], end[1] - start[1]) * rnd.uniform(0.8, 1.0)
            ang = math.degrees(math.atan2(end[1] - start[1], end[0] - start[0]))
            shard(im, start, ang, length, max(1.0, 2.8 * scale), alpha * 0.8, 0.4)
        # the red mechanical wings, opened, at the roots
        d.polygon([joint, spar[1], spar[2], S(26, -20, side), S(18, -4, side)], fill=C("red", int(255 * alpha)), outline=C("red_dark", int(255 * alpha)))
        d.polygon([joint, S(20, -2, side), S(30, -8, side), S(22, 4, side)], fill=C("red_dark", int(255 * alpha)), outline=C("ink", int(200 * alpha)))
        d.line([joint, spar[1], spar[2]], fill=C("frame", int(255 * alpha)))
        d.point(spar[2], fill=(255, 255, 255, int(255 * alpha)))
    # glints drifting off the tips
    for _ in range(12):
        side = rnd.choice((-1, 1))
        x, y = S(rnd.uniform(30, 70), rnd.uniform(-66, -10), side)
        d.point((x, y), fill=(255, 255, 255, int(235 * alpha)))
        d.point((x + side, y + 1), fill=WING_MID[:3] + (int(170 * alpha),))
    return im


def ring_img(w, h, cx, cy, r, color, width=2, alpha=255, dash=0):
    im = Image.new("RGBA", (w, h))
    d = ImageDraw.Draw(im, "RGBA")
    if dash:
        for k in range(0, 360, dash * 2):
            d.arc((cx - r, cy - r, cx + r, cy + r), k, k + dash, fill=color[:3] + (alpha,), width=width)
    else:
        d.ellipse((cx - r, cy - r, cx + r, cy + r), outline=color[:3] + (alpha,), width=width)
    return im


def palm_blast(f, n=6, size=56):
    """Palma Fiocina: a cyan burst from the open palm."""
    im = Image.new("RGBA", (size, size))
    d = ImageDraw.Draw(im, "RGBA")
    c = size / 2
    t = f / (n - 1)
    r = 4 + t * (size / 2 - 5)
    a = max(40, int(255 * (1 - t) ** 0.7))
    d.ellipse((c - r, c - r, c + r, c + r), outline=(110, 230, 255, a), width=2)
    if f < 3:
        g = 9 - f * 2
        d.ellipse((c - g, c - g, c + g, c + g), fill=(220, 252, 255, 230))
        d.ellipse((c - g + 2, c - g + 2, c + g - 2, c + g - 2), fill=(255, 255, 255, 255))
    for k in range(8):
        ang = k * math.pi / 4 + f * 0.3
        r0, r1 = r * 0.4, r * 1.05
        d.line([(c + math.cos(ang) * r0, c + math.sin(ang) * r0), (c + math.cos(ang) * r1, c + math.sin(ang) * r1)], fill=(170, 245, 255, a), width=1)
    return im


def slash(f, n=5, size=48, col=(255, 120, 196), core=(255, 236, 246), r_frac=0.42, thick=4):
    im = Image.new("RGBA", (size, size))
    d = ImageDraw.Draw(im, "RGBA")
    c = size / 2
    r = size * r_frac
    t = f / (n - 1)
    span = 40 + 150 * min(1, t * 2)
    start = -150 + 30 * t
    a = max(40, int(255 * (1 - max(0, t - 0.4) / 0.6)))
    for k in range(thick):
        rr = r - k
        d.arc((c - rr, c - rr, c + rr, c + rr), start, start + span, fill=(col if k else core)[:3] + (a,), width=1)
    return im


def sweep(f, n=6):
    """S2: Arondight's wide cut, radius 37000 (39 px)."""
    size = 96
    im = Image.new("RGBA", (size, size))
    d = ImageDraw.Draw(im, "RGBA")
    c = size / 2
    t = f / (n - 1)
    a = max(40, int(255 * (1 - max(0, t - 0.5) / 0.5)))
    span = 360 * min(1, t * 1.6)
    for k, col in enumerate(((255, 255, 255), (255, 214, 236), (255, 120, 196), (236, 40, 140), (190, 20, 70))):
        rr = 39 - k
        d.arc((c - rr, c - rr, c + rr, c + rr), -90, -90 + span, fill=col + (a,), width=1)
    rnd = random.Random(f)
    for _ in range(10):
        ang = math.radians(-90 + rnd.uniform(0, span))
        rr = rnd.uniform(30, 44)
        d.point((c + math.cos(ang) * rr, c + math.sin(ang) * rr), fill=(255, 255, 255, a))
    return im


def charge_burst(f, n=5):
    im = Image.new("RGBA", (64, 64))
    d = ImageDraw.Draw(im, "RGBA")
    t = f / (n - 1)
    a = max(40, int(240 * (1 - t)))
    for k in range(9):
        ang = math.radians(180 + (k - 4) * 12)
        r0, r1 = 6 + t * 10, 14 + t * 22
        d.line([(32 + math.cos(ang) * r0, 32 + math.sin(ang) * r0), (32 + math.cos(ang) * r1, 32 + math.sin(ang) * r1)],
               fill=(255, 150, 210, a), width=1)
    d.ellipse((26 - 4 * t, 26 - 4 * t, 38 + 4 * t, 38 + 4 * t), outline=(110, 230, 255, a))
    return im


def afterimage(body, face_left=False, k=0):
    """A pink copy of his body, fading (the Wings of Light leave these behind at speed)."""
    im = body.copy()
    if face_left:
        im = im.transpose(Image.Transpose.FLIP_LEFT_RIGHT)
    px = im.load()
    fade = (0.55, 0.35, 0.18)[k]
    for y in range(im.height):
        for x in range(im.width):
            r, g, b, a = px[x, y]
            if a:
                px[x, y] = (255, int(120 + g * 0.3), int(190 + b * 0.2), int(a * fade))
    return im


def landing(f, n=8):
    """The landing shockwave: LAND_R 45000 = 47 px; pink feathers and a cyan core."""
    size = 112
    im = Image.new("RGBA", (size, size))
    d = ImageDraw.Draw(im, "RGBA")
    c = size / 2
    t = f / (n - 1)
    r = 6 + t * 46
    a = max(40, int(255 * (1 - t) ** 0.8))
    d.ellipse((c - r, c - r, c + r, c + r), outline=(255, 120, 196, a), width=3)
    d.ellipse((c - r * 0.7, c - r * 0.7, c + r * 0.7, c + r * 0.7), outline=(110, 230, 255, int(a * 0.8)), width=1)
    if f < 3:
        g = 14 - f * 3
        d.ellipse((c - g, c - g, c + g, c + g), fill=(255, 236, 246, 230))
    rnd = random.Random(77)
    for k in range(16):
        ang = rnd.uniform(0, math.tau)
        rr = r * rnd.uniform(0.6, 1.1)
        x, y = c + math.cos(ang) * rr, c + math.sin(ang) * rr
        shard(im, (x, y), math.degrees(ang), 7 * (1 - t) + 2, 1.6, (1 - t), 0.6)
    return im


def ground_ring(r_px, color, f, n=8, alpha=150, w=None, h=None, offset=20):
    """A ring on the ground at his feet (20 px below his position), pulsing; w x h with his position at the centre."""
    w = w or int(r_px * 2 + 8)
    h = h or int(r_px * 2 + 8 + offset * 2)
    im = Image.new("RGBA", (w, h))
    d = ImageDraw.Draw(im, "RGBA")
    cx, cy = w / 2, h / 2 + offset
    pulse = 0.5 + 0.5 * math.sin(f / n * math.tau)
    rr = r_px * 0.5   # flattened onto the ground plane (the game is drawn top-down at a slant)
    d.ellipse((cx - r_px, cy - rr, cx + r_px, cy + rr), outline=color[:3] + (int(alpha * (0.6 + 0.4 * pulse)),), width=2)
    for k in range(0, 360, 30):
        ang = math.radians(k + f * 6)
        d.point((cx + math.cos(ang) * r_px, cy + math.sin(ang) * rr), fill=(255, 255, 255, int(200 * pulse)))
    return im


def protect_shimmer(f, n=6):
    im = Image.new("RGBA", (48, 64))
    d = ImageDraw.Draw(im, "RGBA")
    a = int(90 + 70 * math.sin(f / n * math.tau))
    d.arc((8, 10, 40, 58), 200, 340, fill=(110, 230, 255, a), width=1)
    d.point((24, 11 + f % 3), fill=(220, 252, 255, 220))
    return im


def incoming(f, n=8):
    """On the ally he's coming for: a pink wing sigil coming down over them."""
    im = Image.new("RGBA", (48, 80))
    t = f / (n - 1)
    y = 6 + t * 18
    for side in (-1, 1):
        for k in range(5):
            shard(im, (24 + side * 2, y + 8), -90 - side * (30 + k * 14), 12 - k, 1.6, 0.9, 0.7)
    d = ImageDraw.Draw(im, "RGBA")
    d.ellipse((14, 56, 34, 64), outline=(255, 120, 196, int(200 * (0.5 + 0.5 * t))))
    return im


def wall_hit(f, n=5):
    im = Image.new("RGBA", (40, 40))
    d = ImageDraw.Draw(im, "RGBA")
    t = f / (n - 1)
    a = max(40, int(255 * (1 - t)))
    for k in range(10):
        ang = k * math.tau / 10
        d.line([(20 + math.cos(ang) * (3 + t * 6), 20 + math.sin(ang) * (3 + t * 6)), (20 + math.cos(ang) * (8 + t * 12), 20 + math.sin(ang) * (8 + t * 12))],
               fill=(255, 236, 180, a))
    d.ellipse((16, 16, 24, 24), fill=(255, 255, 255, a))
    return im


# ------------------------------------------------------------------ build

def pack(anims):
    """anims: {tag: (frames, duration)} -> sheet + fanim (rows, 2048 wide)."""
    x = y = row_h = 0
    pos = []
    for tag, (frames, dur) in anims.items():
        for i, im in enumerate(frames):
            if x + im.width > 2048:
                x, y, row_h = 0, y + row_h + 1, 0
            pos.append((tag, i, x, y, im, dur))
            x += im.width + 1
            row_h = max(row_h, im.height)
    sheet = Image.new("RGBA", (2048, y + row_h))
    meta = {}
    for tag, i, px_, py, im, dur in pos:
        sheet.alpha_composite(im, (px_, py))
        meta.setdefault(tag, {"frames": []})["frames"].append({"duration": dur, "data": {"x": px_, "y": py, "w": im.width, "h": im.height}})
    return sheet, {"anims": meta}


def build():
    A = poses()
    template_meta = json.loads((KIT / "NEW CHAMPION template.anim.json").read_text())
    template = Image.open(KIT / "NEW CHAMPION template.png").convert("RGBA")
    sheet = Image.new("RGBA", template.size)
    meta = template_meta["anims"]
    frames = {}
    for tag, desc in meta.items():
        for i, f in enumerate(desc["frames"]):
            im = body_frame(tag, i, A)
            frames[(tag, i)] = im
            sheet.alpha_composite(im, (f["data"]["x"], f["data"]["y"]))
    (OUT / "champions").mkdir(parents=True, exist_ok=True)
    sheet.save(OUT / "champions" / f"{ID}#sheet.png")
    (OUT / "champions" / f"{ID}#anim.fanim").write_text(json.dumps({"anims": meta}, indent=2) + "\n")

    run0 = frames[("skill1", 2)]
    fx = {
        "wings_light": ([wings_of_light(f) for f in range(8)], 0.07),
        "wings_fade": ([wings_of_light(f, alpha=0.75 if f % 2 else 0.4, flicker=True) for f in range(8)], 0.06),
        "deploy": ([wings_of_light(0, scale=s, alpha=0.6 + 0.4 * s) for s in (0.15, 0.3, 0.48, 0.66, 0.84, 1.0)], 0.06),
        "retract": ([wings_of_light(0, scale=s, alpha=s) for s in (0.9, 0.7, 0.5, 0.32, 0.18, 0.08)], 0.07),
        "flare": ([wings_of_light(f, scale=1.12, alpha=1.0) for f in range(4)], 0.06),
        "palm": ([palm_blast(f) for f in range(6)], 0.04),
        "saber": ([slash(f) for f in range(5)], 0.035),
        "sweep": ([sweep(f) for f in range(6)], 0.045),
        "charge_start": ([charge_burst(f) for f in range(5)], 0.04),
        "charge_hit": ([palm_blast(f, size=40) for f in range(6)], 0.04),
        "wall_hit": ([wall_hit(f) for f in range(5)], 0.05),
        "after_r": ([afterimage(run0, False, k) for k in range(3)], 0.05),
        "after_l": ([afterimage(run0, True, k) for k in range(3)], 0.05),
        "landing": ([landing(f) for f in range(8)], 0.05),
        "incoming": ([incoming(f) for f in range(8)], 0.08),
        "zero_aura": ([ground_ring(55, (255, 120, 196), f) for f in range(8)], 0.1),
        "ally_aura": ([ground_ring(14, (255, 120, 196), f, alpha=200) for f in range(8)], 0.1),
        "protect": ([protect_shimmer(f) for f in range(6)], 0.1),
    }
    vsheet, vmeta = pack(fx)
    (OUT / "vfx").mkdir(parents=True, exist_ok=True)
    vsheet.save(OUT / "vfx" / "gundam#sheet.png", optimize=True)
    (OUT / "vfx" / "gundam#anim.fanim").write_text(json.dumps(vmeta, separators=(",", ":")))
    return A, frames, fx


def previews(frames, fx):
    bg = (54, 74, 60, 255)
    # every body frame, 4x
    rows = ["idle", "run", "attack", "skill1", "skill2", "ult", "hit", "dead"]
    sheet = Image.new("RGBA", (48 * 6, 56 * len(rows)), bg)
    for r, tag in enumerate(rows):
        i = 0
        while (tag, i) in frames:
            sheet.alpha_composite(frames[(tag, i)], (i * 48, r * 56))
            i += 1
    sheet.resize((sheet.width * 4, sheet.height * 4), Image.Resampling.NEAREST).save(HERE / "pose_review.png")
    # normal vs wings deployed, 3x
    w = Image.new("RGBA", (144 * 2 + 16, 128), bg)
    w.alpha_composite(frames[("idle", 0)], (72 - 24, 64 - 28))
    w.alpha_composite(fx["wings_light"][0][0], (144 + 16, 0))
    w.alpha_composite(frames[("ult", 3)], (144 + 16 + 72 - 24, 64 - 28))
    w.resize((w.width * 3, w.height * 3), Image.Resampling.NEAREST).save(HERE / "wings_review.png")
    # an animated GIF: idle -> ult -> wings deploy -> loop -> retract
    gif = []
    for k in range(40):
        im = Image.new("RGBA", (144, 128), bg)
        if 8 <= k < 14:
            im.alpha_composite(fx["deploy"][0][k - 8])
        elif 14 <= k < 32:
            im.alpha_composite(fx["wings_light"][0][k % 8])
        elif 32 <= k < 38:
            im.alpha_composite(fx["retract"][0][k - 32])
        body = frames[("ult", min(3, k // 2))] if 4 <= k < 12 else frames[("idle", (k // 3) % 4)]
        im.alpha_composite(body, (72 - 24, 64 - 28))
        gif.append(im.resize((im.width * 3, im.height * 3), Image.Resampling.NEAREST).convert("RGB"))
    gif[0].save(HERE / "aegis_destiny.gif", save_all=True, append_images=gif[1:], duration=80, loop=0)


if __name__ == "__main__":
    A, frames, fx = build()
    previews(frames, fx)
    print("built", ID, "body + gundam vfx", len(fx), "effects")
