"""Check generated data/art reproducibility and preserved champion balance fields."""
from pathlib import Path
import hashlib
import json
import subprocess
from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
files = [
    'native/tfm2_custom_ai/src/rework_data.rs', 'editor/rework-data.js',
    'mods/tfm2_frieren/vfx/frieren_rework#sheet.png',
    'mods/tfm2_frieren/vfx/frieren_rework#anim.fanim',
    'mods/tfm2_frieren/champion/tfm2_frieren_frieren.data_champion',
]
def digest(p):
    return hashlib.sha256((ROOT / p).read_bytes()).hexdigest()

before = {p: digest(p) for p in files}
for name in ['generate-rework-data.py', 'generate-frieren-hud.py']:
    subprocess.run(['rtk', 'python', '-X', 'utf8', str(ROOT / 'tools' / name)], cwd=ROOT, check=True)
assert before == {p: digest(p) for p in files}, 'Generated files were stale or not reproducible'

for name in ['tfm2_custom_magnus', 'tfm2_custom_unified_theory', 'tfm2_frieren_frieren']:
    p = next((ROOT / 'mods').glob('*/champion/' + name + '.data_champion'))
    old = subprocess.run(['rtk', 'proxy', 'git', 'show',
        '920f80b:' + p.relative_to(ROOT).as_posix()], cwd=ROOT, check=True,
        capture_output=True, text=True, encoding='utf-8').stdout
    baseline, candidate = json.loads(old), json.loads(p.read_text())
    for key in ['id', 'stat', 'growth']:
        assert baseline.get(key) == candidate.get(key), (name, key)
    if name.endswith('magnus'):
        assert baseline == candidate, 'Magnus numerical/data balance changed'

native = json.loads((ROOT / 'native/tfm2_custom_ai/mod.mod_info').read_text())
assert native == json.loads((ROOT / 'mods/tfm2_custom_ai/mod.mod_info').read_text())
for pack in ['tfm2_custom', 'tfm2_frieren']:
    meta = json.loads((ROOT / 'mods' / pack / 'mod.mod_info').read_text())
    assert any(d['mod_id'] == 'tfm2_custom_ai' and d['version'] == '>=' + native['version']
               for d in meta['dependencies'])

sheet = Image.open(ROOT / files[2])
atlas = json.loads((ROOT / files[3]).read_text())
assert max(sheet.size) <= 2048
for anim in atlas['anims'].values():
    for frame in anim['frames']:
        d = frame['data']
        assert 0 <= d['x'] < d['x'] + d['w'] <= sheet.width
        assert 0 <= d['y'] < d['y'] + d['h'] <= sheet.height
        assert sheet.crop((d['x'], d['y'], d['x'] + d['w'], d['y'] + d['h'])).getbbox()
report = {
    'generatedFilesReproducible': True,
    'baseStatsGrowthAndIdsPreserved': True,
    'magnusDataUnchanged': True,
    'metadataDependenciesSynchronized': True,
    'frierenNewAtlas': {'width': sheet.width, 'height': sheet.height,
                       'decodedBytes': sheet.width * sheet.height * 4,
                       'layers': len(atlas['anims'])},
}
out = ROOT / 'docs/rework-results/data-art.json'
out.parent.mkdir(parents=True, exist_ok=True)
out.write_text(json.dumps(report, indent=2) + '\n')
print(json.dumps(report, indent=2))
