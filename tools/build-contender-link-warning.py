"""Pack the exact landing warning geometry for the Contender art review."""
import argparse
import io
import json
from pathlib import Path

from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parents[1]
ASSET = ROOT / "mods/tfm2_custom/vfx"
RULES = json.loads((ROOT / "rules/contender.json").read_bytes())

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--check", action="store_true")
args = parser.parse_args()

radius = RULES["shadowLink"]["radius"] // 1000
width = 2 * radius + 4
image = Image.new("RGBA", (width, width))
draw = ImageDraw.Draw(image)
box = (2, 2, width - 3, width - 3)
draw.ellipse(box, fill=(117, 71, 199, 30), outline=(79, 43, 133, 208), width=2)
for angle in range(0, 360, 15):
    draw.arc(box, angle, angle + 9, fill=(224, 185, 255, 255), width=1)
sheet = io.BytesIO()
image.save(sheet, format="PNG", optimize=False)
animation = {"anims": {"warning": {"frames": [{"duration": 0.1,
    "data": {"x": 0, "y": 0, "w": width, "h": width}}]}}}

for file, data in {
    ASSET / "contender_link_warning#sheet.png": sheet.getvalue(),
    ASSET / "contender_link_warning#anim.fanim": (json.dumps(animation, indent=2) + "\n").encode(),
}.items():
    if args.check:
        if not file.exists():
            raise SystemExit(f"Warning asset is missing: {file}")
        actual = file.read_text(encoding="utf-8").encode() if file.suffix == ".fanim" else file.read_bytes()
        if actual != data:
            raise SystemExit(f"Warning asset is stale: {file}")
    else:
        file.write_bytes(data)
