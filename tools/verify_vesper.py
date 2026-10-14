#!/usr/bin/env python3
"""Vesper, the Rift Marksman: generated data, every native view name, atlas limits and byte-identical regeneration."""
import hashlib
import importlib.util
import json
import re
import subprocess
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
ID = 'tfm2_custom_vesper'


def script(name):
    p = ROOT / 'Claude outputs/vesper' / name
    s = importlib.util.spec_from_file_location('vesper_' + name.removesuffix('.py'), p)
    m = importlib.util.module_from_spec(s)
    s.loader.exec_module(m)
    return m


subprocess.run(['python3', str(ROOT / 'Claude outputs/vesper/generate.py'), '--check'], check=True)
cat = json.loads((ROOT / 'Claude outputs/vesper/catalogue.json').read_text())
c = json.loads((ROOT / f'mods/tfm2_custom/champion/{ID}.data_champion').read_text())
assert c['passive']['passive_ref'] == 'tfm2_custom_ai:vesper'
assert c['attack']['range'] == 66000 and c['attack']['effect']['name'] == ID + '_bolt'
assert all(c[k]['cooltime'] >= 5184000 for k in ['skill', 'skill2', 'ult']), 'Vesper has no active skills and no ultimate'
views = c['view_effects'] + c['view_buffs'] + c['view_projectiles']
names = [v['name'] for v in views]
assert len(names) == len(set(names))
names = set(names)

# Every name the native passive can format resolves to a view.
kinds = [r['id'] for r in cat['rifts']]
techs = [t['id'] for t in cat['techniques']] + ['maw']
expected = set()
for look in range(9):
    for kind in kinds:
        if kind in ('slash', 'mirror'):
            expected |= {f'rift_{kind}_l{look}_a{a}' for a in range(4)}
        else:
            expected.add(f'rift_{kind}_l{look}')
    expected |= {f'link_l{look}_a{a}' for a in range(8)} | {f'packet_l{look}_a{a}' for a in range(8)}
    expected |= {f'{event}_l{look}' for event in ['open', 'close', 'break', 'seedhint', 'impact']}
expected |= {f'core_f{f}_t{t}' for f in range(10) for t in range(4)}
expected |= {f'glyph{d:02}' for d in range(100)}
expected |= {f'tech_{t}_t{k}' for t in techs for k in range(5)}
expected |= {f'twist{k}' for k in range(len(cat['twist_kinds']))}
expected |= {f'breach_t{k}' for k in range(5)} | {'atlas_warn'}
missing = sorted(t for t in expected if f'{ID}_{t}' not in names)
assert not missing, ('effects without a view', missing[:12])
buffs = {f'vs_outfit_r{l}' for l in range(9)} | {f'vs_aim_r{l}' for l in range(9)}
buffs |= {f'vs_rank{r}' for r in range(7)} | {f'vs_top{p}' for p in range(1, 11)}
buffs |= {f'vs_back_r{r}' for r in range(7)} | {f'vs_back_r7_p{p}' for p in range(1, 5)}
buffs |= {f'vs_charges{n}' for n in range(6)} | {f'vs_strain{b}' for b in range(11)}
assert buffs <= names, ('rank layers without a view', sorted(buffs - names)[:12])
assert ID + '_bolt' in names

# Native tag formats still match the names checked above.
native = (ROOT / 'native/tfm2_custom_ai/src/vesper_core.rs').read_text() + (ROOT / 'native/tfm2_custom_ai/src/vesper.rs').read_text()
formats = set(re.findall(r'format!\("([a-z_]+)', native)) | set(re.findall(r'"(atlas_warn)"', native))
prefixes = {t.rstrip('0123456789') for t in expected} | {n.removesuffix(n.lstrip('vs_abcdefghijklmnopqrstuvwxyz_')) for n in buffs}
not_views = {'seed', 'tick', 'vsd_', 'vs', '_l'}   # log lines, behaviour buffs, owner prefix, {why}_l{look} (open/close/break)
unknown = sorted(f for f in formats if f not in not_views and not any(p.startswith(f) for p in prefixes))
assert not unknown, ('native formats a view name that is not checked', unknown)

# Atlases: every rectangle inside its <=2048 sheet, 8 frames per animation, animated effects keep their phases.
sheets, decoded = {}, 0
for v in views:
    p = ROOT / 'mods' / v['anim'].removeprefix('asset/')
    frames = json.loads(Path(str(p) + '#anim.fanim').read_text())['anims'][v['tag']]['frames']
    if p not in sheets:
        sheets[p] = Image.open(str(p) + '#sheet.png').convert('RGBA')
        assert max(sheets[p].size) <= 2048, (p, sheets[p].size)
        decoded += sheets[p].width * sheets[p].height * 4
    im = sheets[p]
    assert len(frames) == 8, (v['tag'], len(frames))
    rects = [f['data'] for f in frames]
    assert all(d['x'] >= 0 and d['y'] >= 0 and d['x'] + d['w'] <= im.width and d['y'] + d['h'] <= im.height for d in rects), v['tag']
    assert len({(d['w'], d['h']) for d in rects}) == 1, ('animation origin wobbles', v['tag'])
    if not v['tag'].startswith(('charges', 'strain', 'rank')):
        phases = {im.crop((d['x'], d['y'], d['x'] + d['w'], d['y'] + d['h'])).tobytes() for d in rects}
        assert len(phases) >= 6, ('animation lost its phases', v['tag'], len(phases))
assert decoded <= 190 * 2 ** 20, f'decoded atlases {decoded / 2 ** 20:.1f} MiB exceed the 190 MiB budget'

# Rank art: every look differs, the #1 look is its own construction, and rank layers animate.
art = script('vesper_art.py')
assert len({art.outfit(l, 2).tobytes() for l in range(9)}) == 9, 'rank outfits stopped evolving'
assert len({art.aim(l, 2).tobytes() for l in range(9)}) == 9, 'aiming poses stopped evolving'
assert len({art.badge(r, 2).tobytes() for r in range(7)}) == 7
assert len({art.badge(7, 2, p).tobytes() for p in range(1, 11)}) == 10
assert len({art.back(7, 2, p).tobytes() for p in range(1, 5)}) == 4, 'podium auras lost their distinct constructions'
area = lambda im: (lambda b: (b[2] - b[0]) * (b[3] - b[1]))(im.getbbox())
assert area(art.back(0, 2)) < area(art.back(6, 2)) <= area(art.back(7, 2, 1)), 'mastery spectacle does not grow'

# Regeneration reproduces exact bytes, including merged localization and previews.
paths = list((ROOT / 'mods/tfm2_custom/vfx').glob('vesper_*')) + list((ROOT / 'mods/tfm2_custom/champions').glob(ID + '*'))
paths += [ROOT / f'mods/tfm2_custom/champion/{ID}.data_champion', ROOT / 'mods/tfm2_custom/text/champion.i18n']
paths += [ROOT / 'editor' / n for n in ['vesper-mastery-preview.png', 'vesper-art-preview.json', 'vesper-portraits.png']]
paths += [ROOT / 'docs' / n for n in ['vesper-mastery.png', 'vesper-mastery.gif', 'vesper-aiming.png']]
hashes = lambda: {str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in paths}
before = hashes()
script('art.py').generate()
script('data.py').generate()
assert before == hashes(), 'Generated assets drifted'
assert sorted(map(str, (ROOT / 'mods/tfm2_custom/vfx').glob('vesper_*'))) == sorted(p for p in before if '/vfx/' in p), 'sheet set changed'
subprocess.run(['node', 'tools/check_vesper_parity.js'], cwd=ROOT, check=True)
print(f'Verified {len(views)} views over {len(sheets)} sheets: {len(expected)} native effect names and {len(buffs)} rank / HUD layers resolve; '
      f'every atlas <= 2048 px; 8 frames each; regeneration byte-identical; decoded RGBA {decoded / 2 ** 20:.1f} MiB (budget 190).')
