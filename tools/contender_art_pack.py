"""Shared baking for Contender's view-effect art (the grab in round 190, Smother and the clones in round 192).

The game can't stretch, rotate or fade a view effect at runtime, so each tool bakes its Canvas preview into fixed frames:
`heading` turns a right-facing sprite (left headings are mirrored, never upside down), `faded` scales alpha, `pack`
shelf-packs every frame into one sheet plus its `.fanim`, and `write_or_check` writes the files or, with `--check`,
compares the decoded pixels and the `.fanim` JSON (PNG bytes differ between Pillow/zlib builds, e.g. on Windows).
"""
import io
import json
from pathlib import Path

from PIL import Image

TICK = 1 / 60


def heading(img, h):
    """Turn a right-facing sprite to heading h (clockwise eighth-turns from east, screen y down)."""
    theta = 45 * h
    if 90 < theta < 270:
        return img.transpose(Image.FLIP_LEFT_RIGHT).rotate(180 - theta, resample=Image.NEAREST, expand=True)
    return img.rotate(-theta, resample=Image.NEAREST, expand=True)


def faded(img, alpha):
    r, g, b, a = img.split()
    return Image.merge("RGBA", (r, g, b, a.point(lambda v: int(v * alpha + 0.5))))


def pack(anims, width=512):
    """anims: tag -> [(image, seconds)]. Returns (sheet image, fanim dict)."""
    frames = [(tag, img, d) for tag, seq in anims.items() for img, d in seq]
    x = y = row = 0
    placed = []
    for tag, img, d in frames:
        if x + img.width > width:
            x, y, row = 0, y + row + 1, 0
        placed.append((tag, img, d, x, y))
        x += img.width + 1
        row = max(row, img.height)
    sheet = Image.new("RGBA", (width, y + row))
    out = {}
    for tag, img, d, px, py in placed:
        sheet.paste(img, (px, py))
        out.setdefault(tag, {"frames": []})["frames"].append(
            {"duration": round(d, 4), "data": {"x": px, "y": py, "w": img.width, "h": img.height}})
    return sheet, {"anims": out}


def write_or_check(sheet_path: Path, fanim_path: Path, sheet, fanim, check: bool, what: str):
    text = json.dumps(fanim, indent=2) + "\n"
    if check:
        if not sheet_path.exists() or not fanim_path.exists():
            raise SystemExit(f"{what} art is missing: {sheet_path.name}")
        have = Image.open(sheet_path).convert("RGBA")
        if have.size != sheet.size or have.tobytes() != sheet.tobytes():
            raise SystemExit(f"{what} art is stale (pixels differ): {sheet_path}")
        if json.loads(fanim_path.read_text(encoding="utf-8")) != fanim:
            raise SystemExit(f"{what} art is stale (animation data differs): {fanim_path}")
        print(f"{what} art is current")
        return
    buf = io.BytesIO()
    sheet.save(buf, format="PNG", optimize=False)
    sheet_path.write_bytes(buf.getvalue())
    fanim_path.write_bytes(text.encode())
    print(f"{what} art packed")
