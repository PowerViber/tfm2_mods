"""Install Levi, Isliid, and Gundam in tfm2_custom without changing their IDs.

Only asset namespace strings are rewritten. Existing champion balance in the
current save/database is left untouched. Run with the game closed.
"""
from __future__ import annotations

import binascii
import copy
import gzip
import json
import os
from pathlib import Path
import shutil
import struct
import tempfile
from datetime import datetime

ROOT = Path(__file__).resolve().parents[1]
GAME = Path(r"C:\Program Files (x86)\Steam\steamapps\common\Teamfight Manager2")
MODS = (GAME / "mods").resolve()
DATA = Path(os.environ["APPDATA"]) / "TeamSamoyed" / "TeamfightManager2" / "data"
SOURCES = ("tfm2_levi", "tfm2_isliid", "tfm2_gundam")
IDS = {"tfm2_levi_levi", "tfm2_isliid_emperor", "tfm2_gundam_aegis_zero"}
BACKUP = ROOT / "backups" / ("consolidate_" + datetime.now().strftime("%Y%m%d_%H%M%S"))


def new_asset(value):
    if isinstance(value, str):
        for source in SOURCES:
            old = f"asset/{source}/"
            if value.startswith(old):
                return "asset/tfm2_custom/" + value[len(old):]
        return value
    if isinstance(value, list):
        return [new_asset(v) for v in value]
    if isinstance(value, dict):
        return {k: new_asset(v) for k, v in value.items()}
    return value


def atomic_bytes(path: Path, data: bytes):
    path.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.NamedTemporaryFile(dir=path.parent, delete=False) as handle:
        temporary = Path(handle.name)
        handle.write(data)
    temporary.replace(path)


def backup_file(path: Path, relative: Path):
    if path.exists():
        target = BACKUP / relative
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(path, target)


def populate_custom():
    dest = ROOT / "mods" / "tfm2_custom"
    text = json.loads((dest / "text" / "champion.i18n").read_text(encoding="utf-8"))
    for source in SOURCES:
        folder = ROOT / "mods" / source
        for sub in ("champion", "champions", "vfx"):
            for item in (folder / sub).iterdir():
                if not item.is_file():
                    continue
                target = dest / sub / item.name
                if target.exists() and target.read_bytes() != item.read_bytes():
                    raise RuntimeError(f"asset collision: {target}")
                if sub == "champion":
                    champion = json.loads(item.read_text(encoding="utf-8"))
                    if champion["id"] not in IDS:
                        raise RuntimeError(f"unexpected champion: {item}")
                    target.write_text(json.dumps(new_asset(champion), ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
                else:
                    shutil.copy2(item, target)
        localized = json.loads((folder / "text" / "champion.i18n").read_text(encoding="utf-8"))
        for lang, groups in localized.items():
            for group, entries in groups.items():
                existing = text.setdefault(lang, {}).setdefault(group, {})
                for key, value in entries.items():
                    if key in existing and existing[key] != value:
                        raise RuntimeError(f"translation collision: {lang}/{group}/{key}")
                    existing[key] = value
    (dest / "text" / "champion.i18n").write_text(json.dumps(text, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    info = json.loads((dest / "mod.mod_info").read_text(encoding="utf-8"))
    info.update(name="PowerViber Custom Champions", version="0.2.0",
                description="Minato, Levi, Emperor Isliid and Aegis Zero with native mastery and combat rules.",
                last_updated=datetime.now().strftime("%Y-%m-%d"))
    info["dependencies"] = [{"mod_id": "base", "version": ">=0.4.14"},
                            {"mod_id": "tfm2_custom_ai", "version": ">=0.10.0"}]
    (dest / "mod.mod_info").write_text(json.dumps(info, indent=2) + "\n", encoding="utf-8")
    return dest


def patch_container(path: Path):
    raw = path.read_bytes()
    if raw[:4] != b"TFM2":
        raise RuntimeError(f"unknown format: {path}")
    length = struct.unpack_from("<Q", raw, 13)[0]
    start = len(raw) - length
    compressed = raw[start:]
    if binascii.crc32(compressed) & 0xffffffff != struct.unpack_from("<I", raw, 21)[0]:
        raise RuntimeError(f"CRC mismatch: {path}")
    payload = gzip.decompress(compressed)
    output = bytearray()
    offset = 0
    replacements = {}
    while True:
        index = payload.find(b"1HCM", offset)
        if index < 0:
            output += payload[offset:]
            break
        size = struct.unpack_from("<Q", payload, index + 8)[0]
        end = index + 16 + size
        try:
            champion = json.loads(payload[index + 16:end])
        except (ValueError, UnicodeDecodeError):
            output += payload[offset:index + 4]
            offset = index + 4
            continue
        if not isinstance(champion, dict) or champion.get("id") not in IDS:
            output += payload[offset:end]
        else:
            updated = new_asset(champion)
            if champion == updated:
                output += payload[offset:end]
            else:
                # Assert the only change is the intended asset path namespace.
                assert new_asset(champion) == updated
                encoded = json.dumps(updated, ensure_ascii=False, separators=(",", ":")).encode("utf-8")
                output += payload[offset:index + 8]
                output += struct.pack("<Q", len(encoded)) + encoded
                replacements[champion["id"]] = replacements.get(champion["id"], 0) + 1
        offset = end
    if replacements:
        backup_file(path, Path("career_data") / path.name)
        packed = gzip.compress(bytes(output), compresslevel=6)
        header = bytearray(raw[:start])
        struct.pack_into("<Q", header, 13, len(packed))
        struct.pack_into("<I", header, 21, binascii.crc32(packed) & 0xffffffff)
        result = bytes(header) + packed
        assert gzip.decompress(result[len(header):]) == bytes(output)
        atomic_bytes(path, result)
    return replacements


def install_custom(repo_custom: Path):
    installed = MODS / "tfm2_custom"
    if not installed.is_dir():
        raise RuntimeError(f"missing installed custom mod: {installed}")
    for item in repo_custom.rglob("*"):
        if not item.is_file():
            continue
        relative = item.relative_to(repo_custom)
        target = installed / relative
        if target.exists() and target.read_bytes() != item.read_bytes():
            backup_file(target, Path("installed_custom") / relative)
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(item, target)
    return installed


def disable_old_mods():
    config = GAME / "config" / "game" / "mods.json"
    backup_file(config, Path("config") / "mods.json")
    settings = json.loads(config.read_text(encoding="utf-8"))
    settings["enabled_mods"] = [m for m in settings["enabled_mods"] if m not in SOURCES]
    for source in SOURCES:
        old = (MODS / source).resolve()
        if old.parent != MODS or not old.is_dir():
            raise RuntimeError(f"unsafe old mod path: {old}")
        archive = BACKUP / "disabled_mods" / source
        archive.parent.mkdir(parents=True, exist_ok=True)
        shutil.move(str(old), str(archive))
    atomic_bytes(config, (json.dumps(settings, ensure_ascii=False) + "\n").encode("utf-8"))
    return settings["enabled_mods"]


def main():
    if not GAME.is_dir() or not DATA.is_dir():
        raise RuntimeError("game or career data missing")
    repo_custom = populate_custom()
    installed = install_custom(repo_custom)
    changes = {}
    for path in sorted(DATA.glob("*.data")) + sorted(DATA.glob("*.tfm2db")):
        changes[path.name] = patch_container(path)
    enabled = disable_old_mods()
    champion_files = list((installed / "champion").glob("*.data_champion"))
    ids = [json.loads(p.read_text(encoding="utf-8"))["id"] for p in champion_files]
    assert IDS.issubset(ids) and len(ids) == len(set(ids))
    for path in champion_files:
        if json.loads(path.read_text(encoding="utf-8"))["id"] in IDS:
            assert not any(f"asset/{s}/" in path.read_text(encoding="utf-8") for s in SOURCES)
    print("backup:", BACKUP)
    print("installed champion IDs:", ids)
    print("career asset updates:", changes)
    print("enabled:", enabled)


if __name__ == "__main__":
    main()
