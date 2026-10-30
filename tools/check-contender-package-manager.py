"""Check the UNCHANGED production package in a disposable Mod Manager fixture.

This deliberately does not install/enable the experimental probe or certify it in the game.
"""
from pathlib import Path
import hashlib
import json
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / 'docs/contender-host-proof'

def main():
    backup = ROOT / 'backups'
    backup.mkdir(exist_ok=True)
    fixture = Path(tempfile.mkdtemp(prefix='contender-manager-', dir=backup)).resolve()
    assert fixture.is_relative_to(backup.resolve())
    (fixture / 'bundle.game_data').touch()
    shutil.copytree(ROOT / 'mods', fixture / 'mods')
    config = fixture / 'config/game'
    config.mkdir(parents=True)
    enabled = sorted(p.name for p in (ROOT / 'mods').iterdir() if (p / 'mod.mod_info').is_file())
    assert 'tfm2_contender_probe' not in enabled
    (config / 'mods.json').write_text(json.dumps({'enabled_mods': enabled}), encoding='utf-8')
    cfg = ROOT / 'manager.cfg'
    before = cfg.read_bytes() if cfg.exists() else None
    try:
        result = subprocess.run(['rtk', 'proxy', str(ROOT / 'TFM2 Mod Manager.exe'), '--check', '--yes', '--game', str(fixture)], cwd=ROOT, capture_output=True, text=True, encoding='utf-8', errors='replace')
    finally:
        if before is not None:
            cfg.write_bytes(before)
        elif cfg.is_file():
            cfg.unlink()
    text = result.stdout + result.stderr
    OUT.mkdir(parents=True, exist_ok=True)
    (OUT / 'mod-manager.txt').write_text(text, encoding='utf-8')
    different = []
    count = 0
    for source in (ROOT / 'mods').rglob('*'):
        if source.is_file():
            count += 1
            target = fixture / source.relative_to(ROOT)
            if not target.is_file() or target.read_bytes() != source.read_bytes():
                different.append(source.relative_to(ROOT).as_posix())
    report = {'scope': 'Unchanged production mods in an isolated fixture; probe NOT installed or certified; no actual game/career changes',
              'exitCode': result.returncode, 'filesCompared': count, 'differentOrMissing': different,
              'productionDllSha256': hashlib.sha256((fixture / 'mods/tfm2_custom_ai/tfm2_custom_ai.dll').read_bytes()).hexdigest()}
    (OUT / 'mod-manager.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    print(json.dumps(report, indent=2))
    assert result.returncode == 0 and not different and 'NOT UP TO DATE:' not in text, text

if __name__ == '__main__':
    main()
