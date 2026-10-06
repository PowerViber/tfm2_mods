"""Check consolidated Isliid references, badge art and the installed native build."""
from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path
from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
MOD = ROOT / "mods" / "tfm2_custom"
GAME = Path(r"C:\Program Files (x86)\Steam\steamapps\common\Teamfight Manager2")
INSTALLED = GAME / "mods" / "tfm2_custom"
CHAMP = "tfm2_isliid_emperor"
LOCAL_ONLY = "--local" in sys.argv

data = json.loads((MOD / "champion" / f"{CHAMP}.data_champion").read_text(encoding="utf-8"))
assert data["category"] in {"Melee", "Range", "Magician", "Util", "Assassin"}
assert data["passive"]["passive_ref"] == "tfm2_custom_ai:isliid"
assert data["skill"]["effect"]["effect_ref"] == "tfm2_custom_ai:isliid_guidance"
assert data["skill2"]["effect"]["effect_ref"] == "tfm2_custom_ai:isliid_recall"
assert data["ult"]["effect"]["effect_ref"] == "tfm2_custom_ai:isliid_manifest"
assert data["skill"]["casting_type"] == data["skill2"]["casting_type"] == "Position"

for local in (MOD,) if LOCAL_ONLY else (MOD, INSTALLED):
    assert (local / "champion" / f"{CHAMP}.data_champion").read_bytes() == (MOD / "champion" / f"{CHAMP}.data_champion").read_bytes()
    body = Image.open(local / "champions" / f"{CHAMP}#sheet.png")
    assert body.size == (288, 448), (local, body.size)
    motions = json.loads((local / "champions" / f"{CHAMP}#anim.fanim").read_text(encoding="utf-8"))["anims"]
    assert all((frame["data"]["w"], frame["data"]["h"]) == (48, 56)
               for anim in motions.values() for frame in anim["frames"]), local
    for item in data["view_buffs"] + data["view_effects"]:
        assert item["anim"].startswith("asset/tfm2_custom/")
        base = local / item["anim"].removeprefix("asset/tfm2_custom/")
        anim = json.loads(Path(str(base) + "#anim.fanim").read_text(encoding="utf-8"))["anims"]
        assert item["tag"] in anim, (local, item["name"], item["tag"])
        sheet = Image.open(str(base) + "#sheet.png")
        for f in anim[item["tag"]]["frames"]:
            r = f["data"]
            assert r["x"] + r["w"] <= sheet.width and r["y"] + r["h"] <= sheet.height
    trail_anims = json.loads((local / "vfx" / "engraving_colors#anim.fanim").read_text(encoding="utf-8"))["anims"]
    trail_sheet = Image.open(local / "vfx" / "engraving_colors#sheet.png").convert("RGBA")
    for sword in range(7):
        for angle in range(16):
            tag = f"scar_{sword}_a{angle}"
            assert tag in trail_anims
            assert any(effect["tag"] == tag for effect in data["view_effects"])
    def trail_bounds(angle: int):
        r = trail_anims[f"scar_0_a{angle}"]["frames"][0]["data"]
        return trail_sheet.crop((r["x"], r["y"], r["x"] + r["w"], r["y"] + r["h"])).getchannel("A").getbbox()
    horizontal, vertical, diagonal = (trail_bounds(angle) for angle in (0, 8, 4))
    assert horizontal[2] - horizontal[0] > horizontal[3] - horizontal[1]
    assert vertical[3] - vertical[1] > vertical[2] - vertical[0]
    assert diagonal[2] - diagonal[0] > 15 and diagonal[3] - diagonal[1] > 15

def animation_pixels(sheet, frames):
    return [sheet.crop((r["x"], r["y"], r["x"] + r["w"], r["y"] + r["h"]))
            for frame in frames for r in [frame["data"]]]

manifest = json.loads((ROOT / "editor" / "isliid-art-manifest.json").read_text(encoding="utf-8"))
names = ("skylight", "terra", "darkbringer", "gale", "blood", "rift", "emperor")
states = ("orbit", "flight", "planted", "drawing", "ready")
for family in ("swords", "orbit", "auras", "fields", "badges"):
    source = {"swords": "swords8", "orbit": "orbit8", "auras": "auras8",
              "fields": "aura_fields8", "badges": "badges8"}[family]
    sheet = Image.open(MOD / "vfx" / f"{source}#sheet.png").convert("RGBA")
    anims = json.loads((MOD / "vfx" / f"{source}#anim.fanim").read_text(encoding="utf-8"))["anims"]
    assert manifest[family] == anims
    for tag, anim in anims.items():
        frames = animation_pixels(sheet, anim["frames"])
        alias=family=="fields" and "_frame" in tag
        assert len(frames) == (1 if alias else 8), (family, tag)
        if not alias:
            assert len({hashlib.sha256(frame.tobytes()).digest() for frame in frames}) == 8, (family, tag)
        assert all(frame.getbbox() for frame in frames), (family, tag)
    if family == "badges":
        base=[animation_pixels(sheet, anims[f"rank{i}"]["frames"][:1])[0] for i in range(7)]
        assert all(frame.size==(48,96) and (bbox:=frame.getbbox()) and bbox[0]>=28 and bbox[2]<=48
                   for frame in base), "Badges must fit Levi's narrow upper-right area"
        silhouettes=[hashlib.sha256(frame.getchannel("A").point(lambda a:255 if a else 0).tobytes()).digest()
                     for frame in base]
        assert len(set(silhouettes)) == 7, "Each numberless rank needs a unique silhouette"
        for i in range(1, 11):
            assert f"imperial{i}" in anims
            images=animation_pixels(sheet,anims[f"imperial{i}"]["frames"])
            assert all(frame.size==(48,96) and (bbox:=frame.getbbox()) and bbox[0]>=24 and bbox[2]<=48
                       for frame in images)
            assert len({frame.crop((33,25,45,36)).tobytes() for frame in images})==1, "Imperial numeral must stay steady"
    if family == "swords":
        assert all(f"{sword}_rank{rank}_{state}" in anims
                   for sword in names for rank in range(8) for state in states)
    if family == "auras":
        assert all(f"aura_{sword}_rank{rank}_{side}" in anims
                   for sword in range(7) for rank in range(8) for side in ("ally", "enemy"))
        assert all(f"aura_base_rank{rank}_{side}" in anims
                   for rank in range(8) for side in ("ally","enemy"))
    if family == "fields":
        assert all(f"aura_field_{sword}_rank{rank}_frame{phase}" in anims
                   for sword in range(7) for rank in range(8) for phase in range(8))
        refs={effect["tag"] for effect in data["view_effects"]}
        assert all(f"aura_field_{sword}_rank{rank}_frame{phase}" in refs
                   for sword in range(7) for rank in range(8) for phase in range(8))
assert all(f"il_rank{i}" in [b["name"] for b in data["view_buffs"]] for i in range(7))
assert all(f"il_imperial{i}" in [b["name"] for b in data["view_buffs"]] for i in range(1, 11))
buffs={b["name"] for b in data["view_buffs"]}
assert all(f"il_aura_base_rank{rank}_{side}" in buffs for rank in range(8) for side in ("ally","enemy"))
assert all(f"il_aura_visual_{sword}_rank{rank}_{side}" in buffs
           for sword in range(7) for rank in range(8) for side in ("ally","enemy"))

built = ROOT / "native" / "tfm2_custom_ai" / "target" / "release" / "tfm2_custom_ai.dll"
if not LOCAL_ONLY:
    for deployed in (ROOT / "mods" / "tfm2_custom_ai" / "tfm2_custom_ai.dll", GAME / "mods" / "tfm2_custom_ai" / "tfm2_custom_ai.dll"):
        assert hashlib.sha256(built.read_bytes()).digest() == hashlib.sha256(deployed.read_bytes()).digest(), deployed
    assert sorted(p.name for p in (GAME / "mods" / "tfm2_custom_ai").glob("*.dll")) == ["tfm2_custom_ai.dll"]
print(f"Verified 48x56 Isliid art, 112 directional trails, {len(data['view_buffs']) + len(data['view_effects'])} visual references, eight-frame swords/recipient auras/field auras/badges, Imperial 1-10" + (" and deployed native DLL" if not LOCAL_ONLY else ""))
