"""Canonical Slim A character renderer; no machine-local preview dependencies."""

from PIL import Image, ImageDraw

OUTLINE = "#0e141f"

SKIN = "#ecd2bd"

SKIN_SHADOW = "#cba389"

SKIN_DARK = "#a87d64"

SKIN_WARM = "#dfaa93"

STUBBLE = "#8a796d"

STUBBLE_DARK = "#6e5e54"

HAIR_DEEP = "#22150d"

HAIR_DARK = "#3d2719"

HAIR_MID = "#5c3d28"

HAIR_WARM = "#7e5437"

HAIR_LIGHT = "#a6734c"

HAIR_SHEEN = "#c9956d"

SUIT_BASE = "#202c44"

SUIT_LIGHT = "#354a73"

SUIT_SHADOW = "#141d2e"

SUIT_LAPEL = "#2b3c5d"

SHIRT_WHITE = "#f4f7fb"

SHIRT_SHADOW = "#cbd5e1"

PANTS_BASE = "#182235"

SHOE_BASE = "#281b16"

BOARD_WOOD = "#593c28"

SQ_LIGHT = "#ecd9b5"

SQ_DARK = "#7d5b3d"

GOLD = "#d4b875"

def draw_body(d, x=0, y=0):
    """Body matching coder proportions."""
    # Legs
    d.rectangle((x + 20, y + 36, x + 25, y + 45), fill=PANTS_BASE, outline=OUTLINE)
    d.rectangle((x + 19, y + 45, x + 25, y + 47), fill=SHOE_BASE, outline=OUTLINE)
    d.rectangle((x + 26, y + 36, x + 31, y + 45), fill=PANTS_BASE, outline=OUTLINE)
    d.rectangle((x + 26, y + 45, x + 32, y + 47), fill=SHOE_BASE, outline=OUTLINE)

    # Torso
    d.polygon([(x + 19, y + 22), (x + 32, y + 22), (x + 33, y + 36), (x + 18, y + 36)], fill=SUIT_BASE, outline=OUTLINE)
    d.rectangle((x + 19, y + 24, x + 23, y + 35), fill=SUIT_SHADOW)
    d.line((x + 31, y + 24, x + 32, y + 35), fill=SUIT_LIGHT)
    # V-neck shirt
    d.polygon([(x + 24, y + 22), (x + 27, y + 22), (x + 26, y + 26), (x + 25, y + 26)], fill=SHIRT_WHITE)
    d.point((x + 25, y + 25), fill=SHIRT_SHADOW)
    # Lapels
    d.line((x + 23, y + 22, x + 25, y + 30), fill=SUIT_LAPEL, width=1)
    d.line((x + 28, y + 22, x + 26, y + 30), fill=SUIT_LAPEL, width=1)
    d.point((x + 25, y + 31), fill=OUTLINE)

    # Left arm
    d.line([(x + 20, y + 24), (x + 23, y + 31)], fill=SUIT_BASE, width=3)

    # Chessboard
    d.polygon([(x + 24, y + 29), (x + 38, y + 29), (x + 36, y + 36), (x + 22, y + 36)], fill=BOARD_WOOD, outline=OUTLINE)
    for row in range(3):
        for col in range(5):
            px = x + 25 + col * 2
            py = y + 30 + row
            if x + 24 <= px <= x + 37 and y + 30 <= py <= y + 34:
                fill_c = SQ_LIGHT if (row + col) % 2 == 0 else SQ_DARK
                d.point((px, py), fill=fill_c)
    d.point((x + 27, y + 30), fill="#fffdf5")
    d.point((x + 31, y + 31), fill="#fffdf5")
    d.point((x + 35, y + 30), fill="#231a14")

    # Right forearm
    d.line([(x + 30, y + 24), (x + 35, y + 30)], fill=SUIT_BASE, width=3)
    d.line([(x + 29, y + 24), (x + 34, y + 30)], fill=SUIT_LIGHT, width=1)
    d.polygon([(x + 34, y + 29), (x + 37, y + 29), (x + 36, y + 32)], fill=SKIN, outline=OUTLINE)

def draw_head_slim_a(d, x, y):
    """
    Slim head option A — closest to coder proportions.
    Compact oval face, wavy hair clusters, subtle Magnus features.
    Face: rows 9-19, ~10px wide.
    """
    # Neck
    d.rectangle((x + 24, y + 19, x + 27, y + 22), fill=SKIN, outline=OUTLINE)
    d.point((x + 24, y + 20), fill=SKIN_SHADOW)

    # Face — lean oval, tapered chin
    d.polygon([
        (x + 22, y + 10),  # left temple
        (x + 25, y + 9),   # forehead
        (x + 29, y + 10),  # right temple
        (x + 30, y + 13),  # right cheek
        (x + 29, y + 17),  # right jaw
        (x + 26, y + 19),  # chin
        (x + 23, y + 17),  # left jaw
        (x + 21, y + 13),  # left cheek
    ], fill=SKIN, outline=OUTLINE)

    # Cheek shadow
    d.point((x + 29, y + 14), fill=SKIN_SHADOW)
    # Chin shadow
    d.point((x + 25, y + 18), fill=SKIN_DARK)
    d.point((x + 26, y + 18), fill=SKIN_DARK)

    # Ear
    d.polygon([(x + 20, y + 12), (x + 21, y + 12), (x + 21, y + 15), (x + 20, y + 14)], fill=SKIN_WARM, outline=OUTLINE)

    # Brow — dark, heavy
    d.line([(x + 23, y + 12), (x + 29, y + 12)], fill=HAIR_DARK, width=1)

    # Eyes
    d.point((x + 24, y + 13), fill="#ffffff")
    d.point((x + 25, y + 13), fill=OUTLINE)
    d.point((x + 27, y + 13), fill="#ffffff")
    d.point((x + 28, y + 13), fill=OUTLINE)

    # Nose
    d.point((x + 26, y + 14), fill=SKIN_SHADOW)
    d.point((x + 27, y + 15), fill=SKIN_DARK)

    # Mouth
    d.line([(x + 24, y + 17), (x + 27, y + 17)], fill=SKIN_DARK)

    # Stubble
    for px, py in [(23, 17), (24, 18), (25, 18), (26, 18), (27, 17), (28, 16)]:
        d.point((x + px, y + py), fill=STUBBLE_DARK if (px+py) % 2 == 0 else STUBBLE)

    # Hair — compact wavy swept-back
    d.polygon([
        (x + 20, y + 11), (x + 20, y + 8), (x + 23, y + 6),
        (x + 27, y + 6), (x + 30, y + 8), (x + 30, y + 11),
        (x + 27, y + 10), (x + 23, y + 10),
    ], fill=HAIR_MID, outline=OUTLINE)

    # Wavy tufts
    d.polygon([(x + 22, y + 6), (x + 24, y + 4), (x + 26, y + 5), (x + 25, y + 7)], fill=HAIR_DARK, outline=OUTLINE)
    d.polygon([(x + 26, y + 5), (x + 28, y + 4), (x + 30, y + 6), (x + 29, y + 7)], fill=HAIR_WARM, outline=OUTLINE)
    # Highlight
    d.point((x + 27, y + 5), fill=HAIR_SHEEN)
    d.point((x + 24, y + 5), fill=HAIR_LIGHT)
    # Dark depth
    d.point((x + 23, y + 8), fill=HAIR_DEEP)

    # Sideburn
    d.point((x + 21, y + 11), fill=HAIR_DARK)
    d.point((x + 21, y + 12), fill=HAIR_DARK)


def frame(pose, index):
    im=Image.new("RGBA",(48,56)); d=ImageDraw.Draw(im)
    f=index%8
    if pose=="dead" and f>=1:
        sink=[0,7,11,15,15,15,15,15][f]
        d.ellipse((7,47,42,52),fill=(10,15,25,65))
        d.polygon([(12,43),(28,42),(35,49),(10,49)],fill=PANTS_BASE,outline=OUTLINE)
        d.polygon([(14,26+sink),(29,24+sink),(28,46),(12,47)],fill=SUIT_BASE,outline=OUTLINE)
        draw_head_slim_a(d,0,2+sink)
        d.line((24,15+sink,25,15+sink),fill=SKIN_DARK)
        d.line((27,15+sink,28,15+sink),fill=SKIN_DARK)
        d.line((24,19+sink,28,19+sink),fill=SKIN_DARK)
        d.polygon([(29,45),(42,44),(40,49),(28,49)],fill=BOARD_WOOD,outline=OUTLINE)
        return im
    bob=1 if pose=="idle" and f in (2,3,4) else (f%2 if pose=="run" else 0)
    recoil=[0,1,2,2,1,0,0,0][f] if pose=="hit" else 0
    d.ellipse((13,48,37,52),fill=(10,15,25,60))
    draw_body(d,-recoil,2+bob)
    # Feet remain at the established y=49 baseline in every standing pose.
    d.rectangle((15,38,34,51),fill=(0,0,0,0))
    stride=[-2,-1,0,1,2,1,0,-1][f] if pose=="run" else 0
    d.rectangle((20-stride-recoil,38,25-stride-recoil,47),fill=PANTS_BASE,outline=OUTLINE)
    d.rectangle((19-stride-recoil,47,25-stride-recoil,49),fill=SHOE_BASE,outline=OUTLINE)
    d.rectangle((26+stride-recoil,38,31+stride-recoil,47),fill=PANTS_BASE,outline=OUTLINE)
    d.rectangle((26+stride-recoil,47,32+stride-recoil,49),fill=SHOE_BASE,outline=OUTLINE)
    draw_head_slim_a(d,-recoil,2+bob)
    if pose in ("attack","skill1","skill2","ult"):
        lift=[0,1,2,3,3,2,1,0][f]
        reach=[0,1,2,4,3,2,1,0][f] if pose=="attack" else 0
        hand=(36+reach,29-lift) if pose!="ult" else (34,16-lift)
        d.line([(31,26),hand],fill=SUIT_BASE,width=3)
        d.line([(30,26),(hand[0]-1,hand[1])],fill=SUIT_LIGHT,width=1)
        d.rectangle((hand[0]-1,hand[1]-1,hand[0]+1,hand[1]+1),fill=SKIN,outline=OUTLINE)
        if pose=="ult":
            d.polygon([(33,11-lift),(37,11-lift),(35,7-lift)],fill=GOLD,outline=OUTLINE)
            d.point((35,6-lift),fill="#fffdf5")
    if pose=="hit" and f in (1,2,3):
        d.line((35,21,39,17),fill=GOLD)
    return im

