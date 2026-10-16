"""Add Magnus's missing ultimate passive without replacing career balance data.

The default is inspection only. --output writes a separate validated container;
--in-place keeps a byte-for-byte backup before atomically replacing the input.
"""
from __future__ import annotations
import argparse
import binascii
import gzip
import hashlib
import json
from pathlib import Path
import shutil
import struct
import time

CHAMPION = "tfm2_custom_magnus"
PASSIVE = {"passive_ref": "tfm2_custom_ai:magnus_ult", "params": {}}


def unpack(data: bytes) -> tuple[bytes, bytes]:
    if len(data) < 25 or data[:4] != b"TFM2":
        raise ValueError("Not a TFM2 container")
    length = struct.unpack_from("<Q", data, 13)[0]
    if not 0 < length <= len(data) - 25:
        raise ValueError("Invalid compressed payload length")
    start = len(data) - length
    compressed = data[start:]
    if binascii.crc32(compressed) & 0xFFFFFFFF != struct.unpack_from("<I", data, 21)[0]:
        raise ValueError("Container checksum mismatch")
    return data[:start], gzip.decompress(compressed)


def migrate(data: bytes, farming: bool = False) -> tuple[bytes, dict]:
    header, payload = unpack(data)
    parts, cursor = [], 0
    report = {"found": 0, "changed": 0, "current": 0, "conflicts": 0}
    if farming:
        report.update(farmingFound=0,farmingChanged=0)
    while True:
        at = payload.find(b"1HCM", cursor)
        if at < 0:
            parts.append(payload[cursor:])
            break
        if at + 16 > len(payload):
            parts.append(payload[cursor:])
            break
        size = struct.unpack_from("<Q", payload, at + 8)[0]
        end = at + 16 + size
        try:
            if end > len(payload):
                raise ValueError("Incomplete candidate record")
            entry = json.loads(payload[at + 16:end])
            if not isinstance(entry, dict):
                raise ValueError("Not a champion object")
        except (ValueError, UnicodeDecodeError):
            parts.append(payload[cursor:at + 4])
            cursor = at + 4
            continue
        parts.append(payload[cursor:at])
        record = payload[at:end]
        if entry.get("id") == CHAMPION:
            report["found"] += 1
            passive = entry.get("passive") or {}
            if passive.get("passive_ref") != "tfm2_custom_ai:magnus":
                report["conflicts"] += 1
            elif entry.get("passive_ult") == PASSIVE:
                report["current"] += 1
            elif entry.get("passive_ult") is not None:
                report["conflicts"] += 1
            else:
                entry["passive_ult"] = PASSIVE
                encoded = json.dumps(entry, ensure_ascii=False, separators=(",", ":")).encode("utf-8")
                record = payload[at:at + 8] + struct.pack("<Q", len(encoded)) + encoded
                report["changed"] += 1
        if farming and entry.get("id")=="tfm2_gundam_aegis_zero":
            report["farmingFound"]+=1
            skill=entry.get("skill") or {}
            if (entry.get("passive") or {}).get("passive_ref")!="tfm2_custom_ai:gundam" or skill.get("casting_target") not in ("EnemyChampion","EnemyWithoutTower"):
                report["conflicts"]+=1
            elif skill["casting_target"]=="EnemyChampion":
                skill["casting_target"]="EnemyWithoutTower"
                encoded=json.dumps(entry,ensure_ascii=False,separators=(",", ":")).encode("utf-8")
                record=payload[at:at+8]+struct.pack("<Q",len(encoded))+encoded
                report["farmingChanged"]+=1
        parts.append(record)
        cursor = end
    if not report["changed"] and not report.get("farmingChanged"):
        return data, report
    compressed = gzip.compress(b"".join(parts), compresslevel=6, mtime=0)
    updated_header = bytearray(header)
    struct.pack_into("<Q", updated_header, 13, len(compressed))
    struct.pack_into("<I", updated_header, 21, binascii.crc32(compressed) & 0xFFFFFFFF)
    result = bytes(updated_header) + compressed
    unpack(result)
    return result, report


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", type=Path)
    parser.add_argument("--farming",action="store_true",help="Also broaden only Aegis Zero's S1 target filter; retain every balance field")
    group = parser.add_mutually_exclusive_group()
    group.add_argument("--output", type=Path)
    group.add_argument("--in-place", action="store_true")
    args = parser.parse_args()
    original = args.source.read_bytes()
    updated, report = migrate(original,args.farming)
    report["sourceSha256"] = hashlib.sha256(original).hexdigest()
    report["resultSha256"] = hashlib.sha256(updated).hexdigest()
    if report["conflicts"]:
        raise SystemExit("Conflicting Magnus definitions; no file written: " + json.dumps(report))
    if args.output:
        if args.output.exists() or args.output.resolve() == args.source.resolve():
            raise SystemExit("Output must be a new file; use --in-place for a backed-up migration")
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_bytes(updated)
        report["output"] = str(args.output.resolve())
    elif args.in_place and updated != original:
        backup = args.source.with_name(args.source.name + ".before-magnus-ult-" + str(time.time_ns()) + ".bak")
        shutil.copy2(args.source, backup)
        if backup.read_bytes() != original:
            raise SystemExit("Backup verification failed; input unchanged")
        temporary = args.source.with_name(args.source.name + ".magnus-migration.tmp")
        with temporary.open("xb") as stream:
            stream.write(updated)
        temporary.replace(args.source)
        report["backup"] = str(backup)
    else:
        report["mode"] = "inspection; input unchanged"
    print(json.dumps(report, indent=2))


if __name__ == "__main__":
    main()
