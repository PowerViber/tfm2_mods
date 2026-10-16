import binascii
import gzip
import importlib.util
import json
from pathlib import Path
import struct
import unittest

spec = importlib.util.spec_from_file_location("migration", Path(__file__).with_name("migrate-magnus-career.py"))
migration = importlib.util.module_from_spec(spec)
spec.loader.exec_module(migration)


def record(entry):
    raw = json.dumps(entry).encode()
    return b"1HCMmeta" + struct.pack("<Q", len(raw)) + raw


def container(payload):
    header = bytearray(b"TFM2" + bytes(41))
    compressed = gzip.compress(payload, mtime=0)
    struct.pack_into("<Q", header, 13, len(compressed))
    struct.pack_into("<I", header, 21, binascii.crc32(compressed) & 0xFFFFFFFF)
    return bytes(header) + compressed


class MigrationTests(unittest.TestCase):
    def test_farming_migration_is_opt_in_and_preserves_aegis_balance(self):
        old={"id":"tfm2_gundam_aegis_zero","passive":{"passive_ref":"tfm2_custom_ai:gundam"},"skill":{"casting_target":"EnemyChampion","cooltime":999,"range":12345},"stat":{"attack":777}}
        data=container(record(old));self.assertEqual(migration.migrate(data)[0],data)
        updated,report=migration.migrate(data,True)
        self.assertEqual(report["farmingChanged"],1)
        expected=dict(old,skill=dict(old["skill"],casting_target="EnemyWithoutTower"))
        self.assertEqual(json.loads(migration.unpack(updated)[1][16:]),expected)
        self.assertEqual(migration.migrate(updated,True)[0],updated)
    def champion(self):
        return {"id": migration.CHAMPION, "stat": {"attack": 777}, "growth": {"hp": 93},
                "passive": {"passive_ref": "tfm2_custom_ai:magnus", "params": {"custom": 42}},
                "passive_ult": None, "career_patch": ["retain", 3]}

    def test_only_missing_passive_changes_and_repeat_is_byte_identical(self):
        old = self.champion()
        other = record({"id": "archer", "career_patch": [1, 2]})
        prefix, suffix = b"career-state\x00", b"\xffother-state"
        data = container(prefix + record(old) + other + suffix)
        updated, report = migration.migrate(data)
        expected = dict(old, passive_ult=migration.PASSIVE)
        raw = json.dumps(expected, ensure_ascii=False, separators=(",", ":")).encode()
        self.assertEqual(migration.unpack(updated)[1], prefix + b"1HCMmeta" + struct.pack("<Q", len(raw)) + raw + other + suffix)
        self.assertEqual(report["changed"], 1)
        again, report = migration.migrate(updated)
        self.assertEqual(again, updated)
        self.assertEqual(report["current"], 1)
        self.assertEqual(migration.unpack(updated)[0][:13], migration.unpack(data)[0][:13])

    def test_all_embedded_copies_and_false_markers(self):
        data = container(b"1HCM" + bytes(12) + record(self.champion()) + record(self.champion()))
        updated, report = migration.migrate(data)
        self.assertEqual(report["changed"], 2)
        self.assertEqual(migration.migrate(updated)[1]["current"], 2)

    def test_conflicting_passive_is_preserved(self):
        old = self.champion()
        old["passive_ult"] = {"passive_ref": "other_mod:custom"}
        data = container(record(old))
        updated, report = migration.migrate(data)
        self.assertEqual(updated, data)
        self.assertEqual(report["conflicts"], 1)

    def test_bad_checksum_is_rejected(self):
        data = bytearray(container(record(self.champion())))
        data[21] ^= 1
        with self.assertRaisesRegex(ValueError, "checksum"):
            migration.migrate(bytes(data))


if __name__ == "__main__":
    unittest.main()
