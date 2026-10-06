"""Back up and deploy the tested Isliid art plus the Levi/Isliid native helper."""
from __future__ import annotations

import csv
import hashlib
import io
import shutil
import subprocess
from datetime import datetime
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
GAME = Path(r"C:\Program Files (x86)\Steam\steamapps\common\Teamfight Manager2")
MOD = ROOT / "mods" / "tfm2_custom"
INSTALLED = GAME / "mods" / "tfm2_custom"
DLL = ROOT / "native" / "tfm2_custom_ai" / "target" / "release" / "tfm2_custom_ai.dll"
ASSETS = ["champion/tfm2_isliid_emperor.data_champion"]
ASSETS += [f"vfx/{name}#{part}" for name in ("swords8", "orbit8", "auras8", "aura_fields8", "badges8", "flags")
           for part in ("sheet.png", "anim.fanim")]

def digest(path: Path) -> bytes:
    return hashlib.sha256(path.read_bytes()).digest()

def main() -> None:
    tasklist = subprocess.check_output(["tasklist", "/FO", "CSV"], text=True, errors="replace")
    running = [row[0] for row in csv.reader(io.StringIO(tasklist)) if row and "teamfight" in row[0].lower()]
    if running:
        raise RuntimeError(f"Close the game before replacing its loaded DLL: {running}")
    if not DLL.is_file() or not INSTALLED.is_dir():
        raise FileNotFoundError("Expected release DLL or installed custom mod is missing")
    stamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    backup = ROOT / "backups" / f"isliid_eight_frame_{stamp}"
    copies = [(MOD / rel, INSTALLED / rel) for rel in ASSETS]
    copies += [(DLL, ROOT / "mods" / "tfm2_custom_ai" / "tfm2_custom_ai.dll"),
               (DLL, GAME / "mods" / "tfm2_custom_ai" / "tfm2_custom_ai.dll")]
    for source, target in copies:
        if not source.is_file() or not target.parent.is_dir():
            raise FileNotFoundError((source, target.parent))
    for source, target in copies:
        if target.is_file():
            saved = backup / target.relative_to(ROOT) if target.is_relative_to(ROOT) else backup / "steam" / target.relative_to(GAME)
            saved.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(target, saved)
        shutil.copy2(source, target)
        assert digest(source) == digest(target), target
        print(f"Installed {target}")
    print(f"Previous installed files backed up at {backup}")

if __name__ == "__main__": main()
