"""Pack the hand-pixelled Isliid assets into game-sized animation sheets."""
from __future__ import annotations

import json
import shutil
from pathlib import Path
from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parents[1]
SRC = ROOT / "Claude outputs" / "isliid"
DST = ROOT / "mods" / "tfm2_isliid"
CHAMP = DST / "champions"
VFX = DST / "vfx"
CHAMP.mkdir(parents=True, exist_ok=True)
VFX.mkdir(parents=True, exist_ok=True)

shutil.copyfile(SRC / "isliid#sheet.png", CHAMP / "tfm2_isliid_emperor#sheet.png")
shutil.copyfile(SRC / "isliid#anim.fanim", CHAMP / "tfm2_isliid_emperor#anim.fanim")
shutil.copyfile(SRC / "isliid_swords#sheet.png", VFX / "swords#sheet.png")
shutil.copyfile(SRC / "isliid_swords#anim.fanim", VFX / "swords#anim.fanim")

def frame(src: Image.Image, entry: dict) -> Image.Image:
    r = entry["data"]
    return src.crop((r["x"], r["y"], r["x"] + r["w"], r["y"] + r["h"]))

def save(name: str, canvas: Image.Image, anims: dict) -> None:
    canvas.save(VFX / f"{name}#sheet.png", optimize=True)
    (VFX / f"{name}#anim.fanim").write_text(json.dumps({"anims": anims}, indent=2), encoding="utf-8")

badge_src = Image.open(SRC / "isliid_ranks#sheet.png").convert("RGBA")
badge_data = json.loads((SRC / "isliid_ranks#anim.fanim").read_text(encoding="utf-8"))["anims"]
badge_sheet = Image.new("RGBA", (8 * 64, 5 * 96))
badge_anims = {}
for idx, (tag, anim) in enumerate(badge_data.items()):
    frames = []
    for sub, source_frame in enumerate(anim["frames"]):
        f = frame(badge_src, source_frame)
        placed = Image.new("RGBA", (64, 96))
        placed.alpha_composite(f, (23, 15))
        n = idx * 2 + sub
        x, y = (n % 8) * 64, (n // 8) * 96
        badge_sheet.alpha_composite(placed, (x, y))
        frames.append({"duration": source_frame["duration"], "data": {"x": x, "y": y, "w": 64, "h": 96}})
    badge_anims[tag] = {"frames": frames}
save("badges", badge_sheet, badge_anims)

sword_src = Image.open(SRC / "isliid_swords#sheet.png").convert("RGBA")
sword_data = json.loads((SRC / "isliid_swords#anim.fanim").read_text(encoding="utf-8"))["anims"]
orbit_sheet = Image.new("RGBA", (8 * 128, 7 * 128))
orbit_anims = {}
places = [(26, 25), (52, 9), (86, 23), (13, 59), (98, 59), (30, 88), (80, 90)]
for idx, (tag, anim) in enumerate(sword_data.items()):
    sword = idx % 7
    frames = []
    for sub, source_frame in enumerate(anim["frames"]):
        f = frame(sword_src, source_frame).resize((22, 44), Image.Resampling.NEAREST)
        placed = Image.new("RGBA", (128, 128))
        placed.alpha_composite(f, places[sword])
        n = idx * 2 + sub
        x, y = (n % 8) * 128, (n // 8) * 128
        orbit_sheet.alpha_composite(placed, (x, y))
        frames.append({"duration": source_frame["duration"], "data": {"x": x, "y": y, "w": 128, "h": 128}})
    orbit_anims["ar_" + tag] = {"frames": frames}
save("orbit", orbit_sheet, orbit_anims)

selector_sheet = Image.new("RGBA", (7 * 128, 128))
selector_anims = {}
for sword, name in enumerate(("skylight", "terra", "darkbringer", "gale", "blood", "rift", "emperor")):
    placed = Image.new("RGBA", (128, 128))
    d = ImageDraw.Draw(placed)
    x, y = places[sword]
    # One crisp four-corner bracket around the selected floating blade.
    glow, light = "#246c92", "#c6fbff"
    for cx, cy, sx, sy in ((x-4,y+4,1,1),(x+26,y+4,-1,1),
                           (x-4,y+40,1,-1),(x+26,y+40,-1,-1)):
        d.line(((cx,cy),(cx+sx*5,cy)),fill=glow,width=2)
        d.line(((cx,cy),(cx,cy+sy*5)),fill=glow,width=2)
        d.point((cx,cy),fill=light)
    selector_sheet.alpha_composite(placed, (sword * 128, 0))
    selector_anims[name] = {"frames": [{"duration": 0.2, "data": {"x": sword * 128, "y": 0, "w": 128, "h": 128}}]}
save("selector", selector_sheet, selector_anims)

fx_sheet = Image.new("RGBA", (4 * 32, 32))
fx_anims = {}
for idx, tag in enumerate(("scar", "seal")):
    frames = []
    for sub in range(2):
        im = Image.new("RGBA", (32, 32))
        d = ImageDraw.Draw(im)
        if tag == "scar":
            d.line(((3, 16), (28, 16)), fill="#081b35", width=5)
            d.line(((4, 15), (27, 15)), fill="#0b6992", width=3)
            d.line(((6, 14), (25, 14)), fill="#9deaff" if sub else "#4cc6ec", width=1)
            d.point((7 + sub * 4, 13), fill="#f4fcff")
        else:
            d.polygon(((16, 1), (30, 15), (16, 30), (1, 15)), fill="#061228", outline="#f6c56b")
            d.polygon(((16, 5), (26, 15), (16, 26), (5, 15)), fill="#164270", outline="#78e7ff")
            d.line(((16, 7), (16, 23)), fill="#fff0ad", width=3)
            d.line(((11, 15), (21, 15)), fill="#fff0ad", width=3)
            d.point((2 + sub, 3), fill="#b1faff")
        x = (idx * 2 + sub) * 32
        fx_sheet.alpha_composite(im, (x, 0))
        frames.append({"duration": 0.12, "data": {"x": x, "y": 0, "w": 32, "h": 32}})
    fx_anims[tag] = {"frames": frames}
save("engraving", fx_sheet, fx_anims)

print("Packed Isliid body, seven sword families, seven unique badges, Imperial 1-10, orbit and engraving effects")
