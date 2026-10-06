"""Give flying, planted and engraving visuals lifetimes that match their repaint ticks."""
from __future__ import annotations

import copy
import json
from pathlib import Path
import shutil

ROOT = Path(__file__).resolve().parents[1]
BACKUP = ROOT / "backups" / "consolidate_20261005_161555" / "visual_timing"
SWORDS = ("skylight", "terra", "darkbringer", "gale", "blood", "rift", "emperor")
GAME = Path(r"C:\Program Files (x86)\Steam\steamapps\common\Teamfight Manager2")


def save_json(path: Path, data):
    path.write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def update(mod: Path, asset_namespace: str, backup_name: str):
    champion = mod / "champion" / "tfm2_isliid_emperor.data_champion"
    swords = mod / "vfx" / "swords#anim.fanim"
    scars = mod / "vfx" / "engraving_colors#anim.fanim"
    for path in (champion, swords, scars):
        if not path.exists():
            raise FileNotFoundError(path)
        destination = BACKUP / backup_name / path.relative_to(mod)
        if not destination.exists():
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(path, destination)
    art = json.loads(swords.read_text(encoding="utf-8"))
    data = json.loads(champion.read_text(encoding="utf-8"))
    effects = {effect["name"]: effect for effect in data["view_effects"]}
    for sword in SWORDS:
        for tier in range(4):
            base = f"{sword}_tier{tier}"
            frame = art["anims"][base]["frames"][0]
            for kind, ticks in (("flight", 1), ("anchor", 7)):
                tag = f"{base}_{kind}"
                art["anims"][tag] = {"frames": [{"duration": ticks / 60,
                    "data": copy.deepcopy(frame["data"])}]}
                name = f"tfm2_isliid_emperor_{tag}"
                effects[name] = {"type": "Animation", "name": name,
                    "anim": f"asset/{asset_namespace}/vfx/swords", "tag": tag,
                    "z": 4, "is_follow": False}
    data["view_effects"] = list(effects.values())
    color = json.loads(scars.read_text(encoding="utf-8"))
    for sword in range(7):
        for angle in range(16):
            for frame in color["anims"][f"scar_{sword}_a{angle}"]["frames"]:
                frame["duration"] = 2 / 60
    save_json(swords, art)
    save_json(scars, color)
    save_json(champion, data)
    for effect in data["view_effects"]:
        if effect["anim"] == f"asset/{asset_namespace}/vfx/swords":
            assert effect["tag"] in art["anims"], effect["name"]
    print(mod, "28 flight tags, 28 anchor tags, 112 directional colored scars")


def main():
    update(ROOT / "mods" / "tfm2_custom", "tfm2_custom", "repo_custom")
    update(GAME / "mods" / "tfm2_custom", "tfm2_custom", "installed_custom")


if __name__ == "__main__":
    main()
