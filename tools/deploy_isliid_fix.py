"""Back up and install the tested Isliid art and native helper in the Steam mod."""
from __future__ import annotations

import hashlib
from pathlib import Path
import shutil


ROOT = Path(__file__).resolve().parents[1]
GAME = Path(r"C:\Program Files (x86)\Steam\steamapps\common\Teamfight Manager2")
BACKUP = ROOT / "backups" / "isliid_sprite_trails_20261005" / "installed"
CUSTOM_FILES = (
    "champion/tfm2_isliid_emperor.data_champion",
    "champions/tfm2_isliid_emperor#sheet.png",
    "champions/tfm2_isliid_emperor#anim.fanim",
    "vfx/engraving_colors#sheet.png",
    "vfx/engraving_colors#anim.fanim",
)
DLL = "tfm2_custom_ai.dll"


def install(source: Path, destination: Path, backup: Path) -> None:
    if not source.is_file() or not destination.parent.is_dir():
        raise FileNotFoundError((source, destination.parent))
    if destination.exists() and not backup.exists():
        backup.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(destination, backup)
    shutil.copy2(source, destination)
    assert hashlib.sha256(source.read_bytes()).digest() == hashlib.sha256(destination.read_bytes()).digest()


if __name__ == "__main__":
    built = ROOT / "native" / "tfm2_custom_ai" / "target" / "x86_64-pc-windows-gnu" / "release" / DLL
    for relative in CUSTOM_FILES:
        install(ROOT / "mods" / "tfm2_custom" / relative,
                GAME / "mods" / "tfm2_custom" / relative,
                BACKUP / "tfm2_custom" / relative)
    install(built, ROOT / "mods" / "tfm2_custom_ai" / DLL,
            BACKUP / "repository" / "tfm2_custom_ai" / DLL)
    install(built, GAME / "mods" / "tfm2_custom_ai" / DLL,
            BACKUP / "tfm2_custom_ai" / DLL)
    print("Installed Isliid character sheet, animation, 112 directional trails and native DLL")
