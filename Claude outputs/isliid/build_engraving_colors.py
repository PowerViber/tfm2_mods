"""Build seven original 32px pixel-art sword marks for the game atlas."""
from pathlib import Path
import json
from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parents[2]
VFX = ROOT / "mods" / "tfm2_isliid" / "vfx"
CHAMP = ROOT / "mods" / "tfm2_isliid" / "champion" / "tfm2_isliid_emperor.data_champion"
COLORS = ["#f6edaa", "#b79769", "#a479d1", "#8de8d9", "#e87283", "#75a7fa", "#ffd166"]

sheet = Image.new("RGBA", (32 * 14, 32), (0, 0, 0, 0))
draw = ImageDraw.Draw(sheet)
anims = {}
for sword, color in enumerate(COLORS):
    frames = []
    rgb = tuple(bytes.fromhex(color[1:]))
    shade = tuple(max(0, c * 45 // 100) for c in rgb)
    shine = tuple(min(255, c + (255 - c) * 55 // 100) for c in rgb)
    for phase in range(2):
        x = (sword * 2 + phase) * 32
        # Two staggered pixel clusters form a carved groove, with chips of light.
        draw.rectangle((x + 2, 20, x + 29, 22), fill=(*shade, 220))
        draw.rectangle((x + 3, 17, x + 28, 19), fill=(*rgb, 240))
        draw.rectangle((x + 7, 16, x + 24, 17), fill=(*shine, 255))
        draw.rectangle((x + 5 + phase, 23, x + 14 + phase, 23), fill=(*rgb, 130))
        draw.rectangle((x + 19, 14 - phase, x + 22, 15 - phase), fill=(*shine, 220))
        draw.rectangle((x + 4, 14 + phase, x + 5, 15 + phase), fill=(*rgb, 200))
        frames.append({"duration": 0.12, "data": {"x": x, "y": 0, "w": 32, "h": 32}})
    anims[f"scar_{sword}"] = {"frames": frames}

VFX.mkdir(parents=True, exist_ok=True)
sheet.save(VFX / "engraving_colors#sheet.png")
(VFX / "engraving_colors#anim.fanim").write_text(json.dumps({"anims": anims}, indent=2), encoding="utf-8")
champ = json.loads(CHAMP.read_text(encoding="utf-8"))
champ["view_effects"] = [v for v in champ["view_effects"] if not v["name"].startswith("tfm2_isliid_emperor_scar_")]
for sword in range(7):
    champ["view_effects"].append({"type": "Animation", "name": f"tfm2_isliid_emperor_scar_{sword}",
        "anim": "asset/tfm2_isliid/vfx/engraving_colors", "tag": f"scar_{sword}", "z": 2, "is_follow": False})
CHAMP.write_text(json.dumps(champ, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
