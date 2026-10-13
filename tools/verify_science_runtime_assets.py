"""Validate the shared renderer's exact runtime sources without regeneration."""
import json
from pathlib import Path
from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
manifest = json.loads((ROOT / 'editor/science-art-preview.json').read_text(encoding='utf-8'))
catalogue = json.loads((ROOT / 'Claude outputs/unified_theory/catalogue.json').read_text(encoding='utf-8'))
cache = {}
for tag, asset in manifest['assets'].items():
    sheet = asset['sheet']
    if sheet not in cache:
        stem = ROOT / 'mods/tfm2_custom/vfx' / sheet
        metadata = json.loads(Path(str(stem) + '#anim.fanim').read_text(encoding='utf-8'))
        with Image.open(str(stem) + '#sheet.png') as image:
            size = image.size
        cache[sheet] = metadata['anims'], size
    anims, (width, height) = cache[sheet]
    assert asset['frames'] == anims[tag]['frames'], (tag, 'manifest differs from runtime')
    for frame in asset['frames']:
        f = frame['data']
        assert 0 <= f['x'] <= width-f['w'] and 0 <= f['y'] <= height-f['h'], tag
        assert frame['duration'] > 0, tag

packets = [0,16,39,40,54,59,63,72,73,74]
instruments = [5,12,13,20,45,57,58,65,66]
for tier in range(4):
    for skill in catalogue['skills']:
        for prefix in ['skill_', 'impact_']:
            assert prefix + skill['id'] + '_t' + str(tier) in manifest['assets']
    for skill in packets + instruments:
        for heading in range(8):
            assert f"packet_{catalogue['skills'][skill]['id']}_t{tier}_a{heading}" in manifest['assets']
    for skill in instruments + [18]:
        assert f'field{skill}_t{tier}' in manifest['assets']
for scientist in range(3):
    for rank in range(8):
        for phase in range(3):
            assert f'pose{scientist}_r{rank}_p{phase}' in manifest['assets']
        assert f'outfit{scientist}_r{rank}' in manifest['assets']
body = json.loads((ROOT/'mods/tfm2_custom/champions/tfm2_custom_unified_theory#anim.fanim').read_text(encoding='utf-8'))
with Image.open(ROOT/'mods/tfm2_custom/champions/tfm2_custom_unified_theory#sheet.png') as image:
    for name in ['idle','run','dead']:
        for frame in body['anims'][name]['frames']:
            f = frame['data']
            alpha = image.crop((f['x'],f['y'],f['x']+f['w'],f['y']+f['h'])).convert('RGBA').getchannel('A')
            assert alpha.getextrema() == (0,255), (name, 'body must contain transparent and visible pixels')
print(f"PASS: {len(manifest['assets'])} exact runtime bindings in {len(cache)} atlases; all skill, impact, directional packet, field and body-pose bindings; transparent idle/run/dead frames.")
