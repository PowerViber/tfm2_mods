"""Check the rendered sheet against the project sprite-kit contract."""
from pathlib import Path
import json
from PIL import Image

source = Path(__file__).resolve()
root = source.parents[1]
kit = source.parents[3] / "Sprite kit"
champ = root / "champions" / "tfm2_gundam_aegis_zero#sheet.png"
meta = root / "champions" / "tfm2_gundam_aegis_zero#anim.fanim"
vfx = root / "vfx" / "gundam#sheet.png"
vfx_meta = root / "vfx" / "gundam#anim.fanim"
image = Image.open(champ).convert("RGBA")
data = json.loads(meta.read_text())["anims"]
expected = json.loads((kit / "NEW CHAMPION template.anim.json").read_text())["anims"]

assert image.size == (288, 616), image.size
assert len(image.getcolors(image.width * image.height) or []) <= 24
for name, ref in expected.items():
    assert data[name] == ref, f"{name} differs from the sprite kit"
for name, anim in data.items():
    for frame in anim["frames"]:
        r = frame["data"]
        assert r["w"] == 48 and r["h"] == 56
        assert r["x"] + 48 <= image.width and r["y"] + 56 <= image.height
        assert image.crop((r["x"], r["y"], r["x"]+48, r["y"]+56)).getbbox(), name

fx_image = Image.open(vfx).convert("RGBA")
fx = json.loads(vfx_meta.read_text())["anims"]
assert fx_image.size == (768, 19 * 96), fx_image.size
for name, anim in fx.items():
    for frame in anim["frames"]:
        r = frame["data"]
        assert r["w"] == 128 and r["h"] == 96
        assert fx_image.crop((r["x"],r["y"],r["x"]+128,r["y"]+96)).getbbox(), name

normal = image.crop((0,0,48,56))
open_wings = fx_image.crop((0,0,128,96))
assert open_wings.getbbox()[2] - open_wings.getbbox()[0] > 2 * (
    normal.getbbox()[2] - normal.getbbox()[0])
assert not (root / "source" / "aegis_zero_concept.png").exists()
print("Sprite-kit layout, frames, palette, wing span and source: OK")
