"""Install the Isliid data mod and updated native helper into the Steam game.

Stage data while the game is open. Run --finish after closing it to replace
the loaded DLL and enable Isliid. Keep every extra DLL outside game/mods.
"""
from __future__ import annotations

import argparse
import json
import shutil
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
GAME = Path(r"C:\Program Files (x86)\Steam\steamapps\common\Teamfight Manager2")
MODS = GAME / "mods"
SOURCE = ROOT / "mods" / "tfm2_isliid"
TARGET = MODS / "tfm2_isliid"
NATIVE_SOURCE = ROOT / "native" / "tfm2_custom_ai" / "target" / "release" / "tfm2_custom_ai.dll"
NATIVE_SOURCE_MOD = ROOT / "mods" / "tfm2_custom_ai" / "tfm2_custom_ai.dll"
NATIVE_TARGET = MODS / "tfm2_custom_ai" / "tfm2_custom_ai.dll"
NATIVE_STAGE = ROOT / "native" / "tfm2_custom_ai" / "target" / "release" / "tfm2_custom_ai.isliid-pending.dll"
OLD_STAGE = MODS / "tfm2_custom_ai" / "tfm2_custom_ai.isliid-pending.dll"
NATIVE_BACKUP = ROOT / "native" / "tfm2_custom_ai" / "target" / "backups" / "tfm2_custom_ai.pre-isliid.dll"
MOD_CONFIG = GAME / "config" / "game" / "mods.json"

def check_paths() -> None:
    if not (GAME / "TeamfightManager2.exe").is_file():
        raise SystemExit(f"Game executable missing: {GAME}")
    if not SOURCE.is_dir() or not NATIVE_SOURCE.is_file():
        raise SystemExit("Build the Isliid mod and native DLL first")
    if TARGET.resolve().parent != MODS.resolve():
        raise SystemExit("Unsafe mod destination")

def game_running() -> bool:
    result = subprocess.run(["tasklist", "/FI", "IMAGENAME eq TeamfightManager2.exe"],
                            capture_output=True, text=True, check=True)
    return "TeamfightManager2.exe" in result.stdout

def stage() -> None:
    check_paths()
    TARGET.mkdir(parents=True, exist_ok=True)
    for item in SOURCE.rglob("*"):
        if item.is_file():
            dest = TARGET / item.relative_to(SOURCE)
            if not dest.resolve().is_relative_to(TARGET.resolve()):
                raise SystemExit(f"Unsafe asset destination: {dest}")
            dest.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(item, dest)
    shutil.copy2(NATIVE_SOURCE, NATIVE_SOURCE_MOD)
    if OLD_STAGE.exists():
        if OLD_STAGE.resolve().parent != (MODS / "tfm2_custom_ai").resolve():
            raise SystemExit("Unexpected old stage path")
        NATIVE_STAGE.parent.mkdir(parents=True, exist_ok=True)
        shutil.move(OLD_STAGE, NATIVE_STAGE)
    shutil.copy2(NATIVE_SOURCE, NATIVE_STAGE)
    print(f"Staged Emperor Isliid in {TARGET}")
    print(f"Staged updated native helper outside the game mods folder: {NATIVE_STAGE}")

def finish() -> None:
    check_paths()
    if game_running():
        raise SystemExit("Close Teamfight Manager 2 before installing the DLL and enabling Isliid")
    if not NATIVE_STAGE.is_file():
        stage()
    if OLD_STAGE.exists():
        raise SystemExit("Extra DLL still inside game mods folder; run staging cleanup first")
    NATIVE_BACKUP.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(NATIVE_TARGET, NATIVE_BACKUP)
    shutil.copy2(NATIVE_STAGE, NATIVE_TARGET)
    shutil.copy2(ROOT / "mods" / "tfm2_custom_ai" / "mod.mod_info", NATIVE_TARGET.parent / "mod.mod_info")
    data = json.loads(MOD_CONFIG.read_text(encoding="utf-8")) if MOD_CONFIG.exists() else {}
    enabled = data.setdefault("enabled_mods", [])
    for mod_id in ("tfm2_custom_ai", "tfm2_isliid"):
        if mod_id not in enabled:
            enabled.append(mod_id)
    if MOD_CONFIG.exists():
        shutil.copy2(MOD_CONFIG, MOD_CONFIG.with_suffix(".pre-isliid.json"))
    MOD_CONFIG.parent.mkdir(parents=True, exist_ok=True)
    MOD_CONFIG.write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print("Installed native helper 0.9.1 and enabled Emperor Isliid")

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--finish", action="store_true")
    args = parser.parse_args()
    finish() if args.finish else stage()
