"""Deploy the tested Isliid helper and corrected tooltip into the existing mod."""

from pathlib import Path
import shutil
from datetime import datetime


ROOT = Path(__file__).resolve().parents[1]
GAME = Path(r"C:\Program Files (x86)\Steam\steamapps\common\Teamfight Manager2")
BUILT = ROOT / "native/tfm2_custom_ai/target/x86_64-pc-windows-gnu/release/tfm2_custom_ai.dll"
TEXT = ROOT / "mods/tfm2_custom/text/champion.i18n"


def main():
    if not (GAME / "mods/tfm2_custom_ai/tfm2_custom_ai.dll").is_file():
        raise RuntimeError("Expected installed native helper is missing")
    if not (GAME / "mods/tfm2_custom/text/champion.i18n").is_file():
        raise RuntimeError("Expected installed custom champion text is missing")
    if not BUILT.is_file() or not TEXT.is_file():
        raise RuntimeError("Built helper or source text is missing")
    backup = ROOT / "backups" / ("isliid_build_" + datetime.now().strftime("%Y%m%d_%H%M%S"))
    backup.mkdir(parents=True, exist_ok=False)
    outputs = [
        (BUILT, ROOT / "mods/tfm2_custom_ai/tfm2_custom_ai.dll", "repo_helper.dll"),
        (BUILT, GAME / "mods/tfm2_custom_ai/tfm2_custom_ai.dll", "installed_helper.dll"),
        (TEXT, GAME / "mods/tfm2_custom/text/champion.i18n", "installed_champion.i18n"),
    ]
    for source, dest, saved in outputs:
        if dest.is_file():
            shutil.copy2(dest, backup / saved)
        shutil.copy2(source, dest)
        print(f"Updated {dest}")
    print(f"Backup {backup}")


if __name__ == "__main__":
    main()
