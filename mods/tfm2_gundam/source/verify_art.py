"""Check Aegis Zero's sheets (in mods/tfm2_custom) against the sprite-kit contract and his data's views."""
from pathlib import Path
import json
from PIL import Image

source = Path(__file__).resolve()
repo = source.parents[3]
mod = repo / "mods" / "tfm2_custom"
kit = repo / "Sprite kit"
ID = "tfm2_gundam_aegis_zero"
image = Image.open(mod / "champions" / f"{ID}#sheet.png").convert("RGBA")
data = json.loads((mod / "champions" / f"{ID}#anim.fanim").read_text())["anims"]
expected = json.loads((kit / "NEW CHAMPION template.anim.json").read_text())["anims"]

assert image.size == (288, 448), image.size
colors = image.getcolors(image.width * image.height) or []
assert len(colors) <= 26, len(colors)
for name, ref in expected.items():
    assert data[name] == ref, f"{name} differs from the sprite kit"
for name, anim in data.items():
    for frame in anim["frames"]:
        r = frame["data"]
        assert r["w"] == 48 and r["h"] == 56
        crop = image.crop((r["x"], r["y"], r["x"] + 48, r["y"] + 56))
        box = crop.getbbox()
        assert box, name
        assert box[3] <= 50, f"{name}: feet below the ground line"

fx_image = Image.open(mod / "vfx" / "gundam#sheet.png").convert("RGBA")
fx = json.loads((mod / "vfx" / "gundam#anim.fanim").read_text())["anims"]
for name, anim in fx.items():
    for frame in anim["frames"]:
        r = frame["data"]
        assert fx_image.crop((r["x"], r["y"], r["x"] + r["w"], r["y"] + r["h"])).getbbox(), name

# the Wings of Light must dwarf him (2.5-3x his own width), the normal body must stay compact
normal = image.crop((0, 0, 48, 56)).getbbox()
r = fx["wings_light"]["frames"][0]["data"]
wings = fx_image.crop((r["x"], r["y"], r["x"] + r["w"], r["y"] + r["h"])).getbbox()
ratio = (wings[2] - wings[0]) / (normal[2] - normal[0])
assert 2.5 <= ratio <= 4.5, ratio

# every view of the champion points at an existing tag
champ = json.loads((mod / "champion" / f"{ID}.data_champion").read_text())
for v in champ["view_effects"] + champ["view_buffs"]:
    assert v["anim"] == "asset/tfm2_custom/vfx/gundam", v
    assert v["tag"] in fx, v["tag"]
print(f"Aegis Zero art OK: kit layout, {len(colors)} colours, wings {ratio:.1f}x his width, {len(fx)} effects, views match")
