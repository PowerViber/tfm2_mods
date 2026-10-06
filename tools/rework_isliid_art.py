"""Repack Isliid at champion scale and draw angle-aware pixel engraving scars."""
from __future__ import annotations

import json
import math
from pathlib import Path
import re
import shutil

from PIL import Image, ImageDraw


ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "Claude outputs" / "isliid"
BACKUP = ROOT / "backups" / "isliid_sprite_trails_20261005"
MODS = (ROOT / "mods" / "tfm2_isliid", ROOT / "mods" / "tfm2_custom")
COLORS = ("#f6edaa", "#b79769", "#a479d1", "#8de8d9", "#e87283", "#75a7fa", "#ffd166")
OLD_W, OLD_H = 64, 72
NEW_W, NEW_H = 48, 56


def remember(path: Path) -> None:
    target = BACKUP / path.relative_to(ROOT)
    if path.exists() and not target.exists():
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(path, target)


def put_image(path: Path, image: Image.Image) -> None:
    remember(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    image.save(path, optimize=True)


def put_json(path: Path, data: dict) -> None:
    remember(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def resize_character() -> None:
    original_sheet = BACKUP / "Claude outputs" / "isliid" / "isliid#sheet.png"
    original_anim = BACKUP / "Claude outputs" / "isliid" / "isliid#anim.fanim"
    remember(SOURCE / "isliid#sheet.png")
    remember(SOURCE / "isliid#anim.fanim")
    old = Image.open(original_sheet).convert("RGBA")
    metadata = json.loads(original_anim.read_text(encoding="utf-8"))
    if old.size != (OLD_W * 6, OLD_H * 8):
        raise ValueError(f"Unexpected original Isliid sheet size: {old.size}")
    canvas = Image.new("RGBA", (NEW_W * 6, NEW_H * 8))
    for row in range(8):
        for col in range(6):
            original = old.crop((col * OLD_W, row * OLD_H, (col + 1) * OLD_W, (row + 1) * OLD_H))
            frame = original.resize((NEW_W, 54), Image.Resampling.NEAREST)
            canvas.alpha_composite(frame, (col * NEW_W, row * NEW_H + 1))
    for anim in metadata["anims"].values():
        for frame in anim["frames"]:
            data = frame["data"]
            if (data["w"], data["h"]) != (OLD_W, OLD_H):
                raise ValueError(f"Unexpected frame size: {data}")
            data["x"] = data["x"] // OLD_W * NEW_W
            data["y"] = data["y"] // OLD_H * NEW_H
            data["w"], data["h"] = NEW_W, NEW_H
    put_image(SOURCE / "isliid#sheet.png", canvas)
    put_json(SOURCE / "isliid#anim.fanim", metadata)
    for mod in MODS:
        put_image(mod / "champions" / "tfm2_isliid_emperor#sheet.png", canvas)
        put_json(mod / "champions" / "tfm2_isliid_emperor#anim.fanim", metadata)
    put_image(ROOT / "editor" / "isliid-sprite-sheet.png", canvas)
    put_image(ROOT / "editor" / "isliid-sprite.png", canvas.crop((0, 0, NEW_W, NEW_H)))


def trail_frame(color: str, angle: int, phase: int) -> Image.Image:
    frame = Image.new("RGBA", (32, 32))
    draw = ImageDraw.Draw(frame)
    radians = math.pi * angle / 16
    ux, uy = math.cos(radians), math.sin(radians)
    a = (round(15.5 - ux * 12), round(15.5 - uy * 12))
    b = (round(15.5 + ux * 12), round(15.5 + uy * 12))
    draw.line((*a, *b), fill="#091321", width=5)
    draw.line((*a, *b), fill=color, width=3)
    draw.line((round(15.5 - ux * 9), round(15.5 - uy * 9),
               round(15.5 + ux * 9), round(15.5 + uy * 9)), fill="#fff7d8" if phase else color, width=1)
    t = -.45 if phase == 0 else .45
    draw.point((round(15.5 + ux * 10 * t), round(15.5 + uy * 10 * t)), fill="#ffffff")
    return frame


def directional_trails() -> None:
    sheet = Image.new("RGBA", (16 * 32, 14 * 32))
    anims: dict[str, dict] = {}
    for sword, color in enumerate(COLORS):
        for angle in range(16):
            frames = []
            for phase in range(2):
                cell = (sword * 16 + angle) * 2 + phase
                x, y = (cell % 16) * 32, (cell // 16) * 32
                sheet.alpha_composite(trail_frame(color, angle, phase), (x, y))
                frames.append({"duration": 2 / 60, "data": {"x": x, "y": y, "w": 32, "h": 32}})
            anims[f"scar_{sword}_a{angle}"] = {"frames": frames}
    for mod in MODS:
        put_image(mod / "vfx" / "engraving_colors#sheet.png", sheet)
        put_json(mod / "vfx" / "engraving_colors#anim.fanim", {"anims": anims})
        champion = mod / "champion" / "tfm2_isliid_emperor.data_champion"
        if not champion.exists():
            continue
        data = json.loads(champion.read_text(encoding="utf-8"))
        effects = [effect for effect in data["view_effects"] if not re.fullmatch(
            r"tfm2_isliid_emperor_scar_\d(?:_a\d+)?", effect["name"])]
        namespace = data["sprite"].split("/")[1]
        for tag in anims:
            effects.append({"type": "Animation", "name": f"tfm2_isliid_emperor_{tag}",
                            "anim": f"asset/{namespace}/vfx/engraving_colors", "tag": tag,
                            "z": 2, "is_follow": False})
        data["view_effects"] = effects
        put_json(champion, data)
    put_image(ROOT / "editor" / "isliid-trails.png", sheet)


if __name__ == "__main__":
    resize_character()
    directional_trails()
    print("Isliid: 48x56 character frames; 7 colors x 16 pixel-art scar angles x 2 frames")
