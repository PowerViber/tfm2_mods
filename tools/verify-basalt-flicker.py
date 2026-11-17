"""Check shipped animation lifetimes, approved pixels and reproducible generation. Never installs."""
from pathlib import Path
import hashlib, json, subprocess

ROOT = Path(__file__).resolve().parents[1]
BASE = '79484f81d90aabe9ca86542a521e747fb7935b7b'
OUT = ROOT / 'docs/basalt-flicker'
OUT.mkdir(exist_ok=True)

def digest(p):
    return hashlib.sha256(p.read_bytes()).hexdigest()

def old(path):
    return subprocess.check_output(['rtk', 'proxy', 'git', 'show', BASE + ':' + path], cwd=ROOT)

def run(*args):
    subprocess.run(['rtk', 'proxy', *args], cwd=ROOT, check=True, capture_output=True)

tracked = ['docs/basalt-art-review/effects.png', 'docs/basalt-art-review/effects.fanim',
           'docs/basalt-art-review/effects-presentation.json', 'editor/basalt-effects.js',
           'native/tfm2_custom_ai/src/basalt_art_data.rs',
           'mods/tfm2_custom/vfx/basalt#sheet.png', 'mods/tfm2_custom/vfx/basalt#anim.fanim']
before = {p: digest(ROOT / p) for p in tracked}
run('python', 'tools/build-basalt-effects.py')
assert before == {p: digest(ROOT / p) for p in tracked}, 'Non-reproducible packing'
run('python', 'tools/generate-basalt.py', '--check')

meta = json.loads((ROOT / 'docs/basalt-art-review/effects-presentation.json').read_text())
anims = json.loads((ROOT / 'mods/tfm2_custom/vfx/basalt#anim.fanim').read_text())['anims']
prior = json.loads(old('mods/tfm2_custom/vfx/basalt#anim.fanim'))['anims']
assert meta['expiry'] == 'exclusive'
for tag, anim in anims.items():
    ticks = sum(f['duration'] * 60 for f in anim['frames'])
    assert abs(ticks - meta['lifetimes'][tag]) < 1e-8, tag
    if tag in prior:
        assert anim == prior[tag], ('Changed original animation', tag)
    else:
        source, suffix = tag.rsplit('_t', 1)
        assert abs(ticks - int(suffix)) < 1e-8
        assert [f['data'] for f in anim['frames']] == [f['data'] for f in prior[source]['frames']], tag

unchanged = ['rules/basalt.json', 'native/tfm2_custom_ai/src/basalt_core.rs',
             'native/tfm2_custom_ai/src/basalt_data.rs',
             'mods/tfm2_custom/vfx/basalt#sheet.png',
             'mods/tfm2_custom/champions/tfm2_custom_basalt#sheet.png',
             'mods/tfm2_custom/champions/tfm2_custom_basalt#anim.fanim']
for p in unchanged:
    assert (ROOT / p).read_bytes().replace(b'\r\n', b'\n') == old(p).replace(b'\r\n', b'\n'), p
champ_path = 'mods/tfm2_custom/champion/tfm2_custom_basalt.data_champion'
champ = json.loads((ROOT / champ_path).read_text())
previous = json.loads(old(champ_path))
assert {k:v for k,v in champ.items() if k not in ['view_effects','passive']} == {k:v for k,v in previous.items() if k not in ['view_effects','passive']}
assert all(binding['tag'] in anims for binding in champ['view_effects'])
assert len(champ['view_effects']) == len(anims)
assert champ['passive'] == {'passive_ref':'tfm2_custom_ai:basalt','params':{'art_fx_v1':1,'art_timing_v2':1}}
assert digest(ROOT / 'docs/basalt-art-review/effects.png') == digest(ROOT / 'mods/tfm2_custom/vfx/basalt#sheet.png')
assert (ROOT / 'docs/basalt-art-review/effects.fanim').read_bytes() == (ROOT / 'mods/tfm2_custom/vfx/basalt#anim.fanim').read_bytes()

report = {
    'baseline': BASE, 'art_revision': meta['revision'], 'rules_revision': 'basalt-1.4',
    'packed_animations': len(anims), 'original_animation_aliases_unchanged': len(prior),
    'duration_aliases_share_original_rectangles': True,
    'rules_core_body_and_effect_pixels_unchanged': True, 'reproducible': True,
    'expiry': 'exclusive', 'max_lease_ticks': max(meta['lease_ticks']),
    'old_wall_visibility_ticks_per_6_tick_interval': 1,
    'new_wall_visibility_ticks_per_6_tick_interval': meta['lifetimes']['square_t6'],
    'old_boulder_visibility_ticks_per_3_tick_interval': 1,
    'new_boulder_visibility_ticks_per_3_tick_interval': meta['lifetimes']['boulder_t3'],
    'decoded_art_bytes': meta['decoded_bytes'] + json.loads((ROOT / 'docs/basalt-art-review/body-presentation.json').read_text())['decoded_body_bytes'],
    'hashes': before, 'native_renderer_acceptance': 'separate disposable-game check required'
}
assert report['decoded_art_bytes'] <= 32 * 1024 * 1024
(OUT / 'assets.json').write_text(json.dumps(report, indent=2) + '\n')
print(json.dumps(report, indent=2))
