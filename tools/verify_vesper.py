#!/usr/bin/env python3
"""Vesper and the Rift Gauntlets: generated data, every native view name, atlas limits and byte-identical regeneration."""
import hashlib
import importlib.util
import json
import math
import re
import subprocess
from pathlib import Path

from PIL import Image, ImageChops, ImageDraw

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
assert c['attack']['range'] == 45000 and c['attack']['effect']['name'] == ID + '_bolt'
assert all(c[k]['cooltime'] >= 5184000 for k in ['skill', 'skill2', 'ult']), 'Vesper has no active skills and no ultimate'
views = c['view_effects'] + c['view_buffs'] + c['view_projectiles']
names = [v['name'] for v in views]
assert len(names) == len(set(names))
names = set(names)

# Every name the native passive can format resolves to a view.
kinds = [r['id'] for r in cat['rifts']]
techs = [t['id'] for t in cat['techniques']] + ['maw']
assert 'opunch' in techs and 'infuse' in techs and 'overload' not in techs and 'borrow' not in techs
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
expected |= {f'hole_l{l}' for l in range(9)} | {f'punch_t{t}' for t in range(5)} | {f'shatter_t{t}' for t in range(5)}
expected |= {f'swipe_t{t}_a{h}' for t in range(5) for h in range(4)} | {f'awaken_t{t}' for t in range(5)}
expected |= {f'gbreak_l_t{t}' for t in range(4)} | {f'overload_t{t}' for t in range(4)} | {'gbreak_r', 'gfix_l', 'corrupt'}
expected |= {f'mod{m}_t{t}' for m in range(len(cat['modules'])) for t in range(4)}
missing = sorted(t for t in expected if f'{ID}_{t}' not in names)
assert not missing, ('effects without a view', missing[:12])
buffs = {f'vs_{pose}_r{l}' for pose in ['outfit', 'jab', 'block', 'punch', 'swipe', 'shatter'] for l in range(9)}
buffs |= {f'vs_rank{r}' for r in range(7)} | {f'vs_top{p}' for p in range(1, 11)}
buffs |= {f'vs_back_r{r}' for r in range(7)} | {f'vs_back_r7_p{p}' for p in range(1, 5)}
buffs |= {f'vs_orbs{r}_{n}' for n in range(5, 10) for r in range(n + 1)} | {'vs_orbs_x', 'vs_hex_x', 'vs_core_x'}
buffs |= {f'vs_hex{r}_{n}' for n in range(2, 7) for r in range(n + 1)} | {f'vs_core{c}' for c in range(7)}
buffs |= {f'vs_core_aw{c}' for c in range(5)} | {f'vs_stress{b}' for b in range(11)}
assert buffs <= names, ('rank layers without a view', sorted(buffs - names)[:12])
assert ID + '_bolt' in names
gone = sorted(n for n in names if any(k in n for k in ('breach', 'atlas_warn', 'charges', 'strain', '_aim_r', 'tech_overload', 'tech_borrow')))
assert not gone, ('removed mechanics still have views', gone[:8])
assert all(v['name'].startswith('vs_') for v in c['view_buffs']) and not any(v['name'].startswith(ID + '_core_') and v['tag'][5:6].isdigit() for v in c['view_effects'])

# Native tag formats still match the names checked above.
native = (ROOT / 'native/tfm2_custom_ai/src/vesper_core.rs').read_text() + (ROOT / 'native/tfm2_custom_ai/src/vesper.rs').read_text()
assert re.search(r'format!\("vs_\{\}_r\{l\}"', native), 'the native names its pose overlays'
formats = set(re.findall(r'format!\("([a-z_]+)', native)) | set(re.findall(r'"(gbreak_r|gfix_l|corrupt)"\.into', native))
prefixes = {t.rstrip('0123456789') for t in expected} | {n.removesuffix(n.lstrip('vs_abcdefghijklmnopqrstuvwxyz_')) for n in buffs}
not_views = {'seed', 'tick', 'vsd_', 'vsm_', 'vs', '_l'}   # log lines, behaviour buffs, owner prefix, {why}_l{look} (open/close/break)
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
    if not v['tag'].startswith(('orbs', 'hex', 'core', 'stress', 'rank')) or v['tag'].startswith('core_f'):
        phases = {im.crop((d['x'], d['y'], d['x'] + d['w'], d['y'] + d['h'])).tobytes() for d in rects}
        assert len(phases) >= 6, ('animation lost its phases', v['tag'], len(phases))
assert decoded <= 190 * 2 ** 20, f'decoded atlases {decoded / 2 ** 20:.1f} MiB exceed the 190 MiB budget'

# Rank art: every look differs, the #1 look is its own construction, and rank layers animate.
art = script('vesper_art.py')
assert len({art.outfit(l, 2).tobytes() for l in range(9)}) == 9, 'rank outfits stopped evolving'
for pose in art.POSES:
    assert len({art.costume(l, 3, pose).tobytes() for l in range(9)}) == 9, (pose, 'pose stopped evolving per rank')
    if pose != 'outfit':
        assert len({art.costume(2, f, pose).tobytes() for f in range(8)}) >= 6, (pose, 'pose lost its motion')

# The new silhouette: a waist narrower than the old 16 px torso, staggered bent legs with both feet on y 58,
# no hood or mask (hair, skin and black glasses instead), both gauntlets nearly torso-sized.
def width(im, y):
    xs = [x for x in range(im.width) if im.getpixel((x, y))[3]]
    return xs[-1] - xs[0] + 1 if xs else 0
for look in range(9):
    t = art.torso_only(look)
    assert max(width(t, y) for y in range(37, 40)) <= 13, ('waist wider than 82% of the old torso', look)
legs = Image.new('RGBA', (48, 64))
art.legs(ImageDraw.Draw(legs), 'idle', 0)
feet = [x for x in range(48) if legs.getpixel((x, 58))[3]]
assert feet and max(y for y in range(64) for x in range(48) if legs.getpixel((x, y))[3]) == 58, 'feet left the anchor'
runs = [k for k in range(1, len(feet)) if feet[k] != feet[k - 1] + 1]
assert runs, 'the stance lost its stagger: one block of feet'
head = art.costume(0, 0).crop((18, 12, 33, 28))
cols = {px[:3] for px in head.getdata() if px[3]}
assert art.HAIR[:3] in cols and art.SKIN[:3] in cols and art.LENS_CYAN[:3] in cols, 'hair, face and glasses must show'
lead, _ = art.gauntlet(0, 0, 'L', 'up', 4, big=True)
b = lead.getbbox()
assert b[2] - b[0] >= 16 and b[3] - b[1] >= 18, ('the lead gauntlet shrank', b)
# Layering: every overlay frame covers the body sheet's upper body (the body never changes above the legs).
mask = art.body_mask()
bodies = {art.body(k, f).crop((0, 0, 48, 42)).getchannel('A').point(lambda v: 255 if v else 0).tobytes()
          for k in ('idle', 'run') for f in range(8)}   # attack and hit only add effects around her
assert len(bodies) == 1, 'the body sheet moved above the legs'
for pose in art.POSES:
    for look in range(9):
        for f in range(8):
            al = art.costume(look, f, pose).getchannel('A').point(lambda v: 255 if v == 255 else 0)
            assert not ImageChops.subtract(mask, al).getbbox(), ('overlay leaks the body below', pose, look, f)

# Regeneration reproduces exact bytes, including merged localization and previews.
paths = list((ROOT / 'mods/tfm2_custom/vfx').glob('vesper_*')) + list((ROOT / 'mods/tfm2_custom/champions').glob(ID + '*'))
paths += [ROOT / f'mods/tfm2_custom/champion/{ID}.data_champion', ROOT / 'mods/tfm2_custom/text/champion.i18n']
paths += [ROOT / 'editor' / n for n in ['vesper-mastery-preview.png', 'vesper-art-preview.json', 'vesper-portraits.png']]
paths += [ROOT / 'docs' / n for n in ['vesper-mastery.png', 'vesper-mastery.gif', 'vesper-gauntlets.png']]
hashes = lambda: {str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in paths}
before = hashes()
script('art.py').generate()
script('data.py').generate()
assert before == hashes(), 'Generated assets drifted'
assert sorted(map(str, (ROOT / 'mods/tfm2_custom/vfx').glob('vesper_*'))) == sorted(p for p in before if '/vfx/' in p), 'sheet set changed'
subprocess.run(['node', 'tools/check_vesper_parity.js'], cwd=ROOT, check=True)
print(f'Verified {len(views)} views over {len(sheets)} sheets: {len(expected)} native effect names and {len(buffs)} rank / gauntlet HUD layers resolve; '
      f'every atlas <= 2048 px; 8 frames each; regeneration byte-identical; decoded RGBA {decoded / 2 ** 20:.1f} MiB (budget 190).')
