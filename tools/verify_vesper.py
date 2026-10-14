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
buffs = {f'vs_{pose}_r{l}' for pose in ['outfit', 'jab', 'block', 'punch', 'swipe', 'shatter'] for l in range(11)}
buffs |= {f'vs_rank{r}' for r in range(7)} | {f'vs_top{p}' for p in range(1, 11)} | {'vs_back_wound', 'vs_back_top1'}
buffs |= {f'vs_orbs{r}_{n}' for n in range(5, 10) for r in range(n + 1)} | {'vs_orbs_x', 'vs_hex_x', 'vs_core_x'}
buffs |= {f'vs_hex{r}_{n}' for n in range(2, 7) for r in range(n + 1)} | {f'vs_core{c}' for c in range(7)}
buffs |= {f'vs_core_aw{c}' for c in range(5)} | {f'vs_stress{b}' for b in range(11)}
assert buffs <= names, ('rank layers without a view', sorted(buffs - names)[:12])
assert ID + '_bolt' in names
gone = sorted(n for n in names if any(k in n for k in ('breach', 'atlas_warn', 'charges', 'strain', '_aim_r', 'tech_overload', 'tech_borrow'))
              or re.fullmatch(r'vs_back_r\d.*', n))
assert not re.search(r'vs_back_r', native_src := (ROOT / 'native/tfm2_custom_ai/src/vesper.rs').read_text()), 'native still asks for the old auras'
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
    assert len(frames) == (16 if v['tag'].startswith(('rank', 'top')) else 8), (v['tag'], len(frames))
    rects = [f['data'] for f in frames]
    assert all(d['x'] >= 0 and d['y'] >= 0 and d['x'] + d['w'] <= im.width and d['y'] + d['h'] <= im.height for d in rects), v['tag']
    assert len({(d['w'], d['h']) for d in rects}) == 1, ('animation origin wobbles', v['tag'])
    if not v['tag'].startswith(('orbs', 'hex', 'core', 'stress')) or v['tag'].startswith('core_f'):
        phases = {im.crop((d['x'], d['y'], d['x'] + d['w'], d['y'] + d['h'])).tobytes() for d in rects}
        assert len(phases) >= 6, ('animation lost its phases', v['tag'], len(phases))
assert decoded <= 190 * 2 ** 20, f'decoded atlases {decoded / 2 ** 20:.1f} MiB exceed the 190 MiB budget'

# Mastery art: one costume for every rank, the damage she does to reality tells the rank apart.
art = script('vesper_art.py')
assert art.LOOKS == 11
for pose in art.POSES:
    loops = {b''.join(art.costume(l, f, pose).tobytes() for f in range(8)) for l in range(art.LOOKS)}
    assert len(loops) == art.LOOKS, (pose, 'pose looks stopped differing')
    if pose != 'outfit':
        assert len({art.costume(2, f, pose).tobytes() for f in range(8)}) >= 6, (pose, 'pose lost its motion')
emblems = [(r, None) for r in range(7)] + [(7, p) for p in range(1, 11)]
assert len({art.emblem(r, 3, p).tobytes() for r, p in emblems}) == 17, 'every rank and Top-10 place has its own emblem'
ox, oy = art.EMBLEM[0] // 2 - 24, art.EMBLEM[1] // 2 - 32                # body origin on the emblem canvas
head = art.costume(8, 0).crop((18, 8, 36, 30)).getchannel('A')
for r, p in emblems:
    for f in (0, 5, 10):
        e = art.emblem(r, f, p)
        b = e.getbbox()
        assert b[0] - ox >= 26 and b[3] - oy <= 38 and b[2] < art.EMBLEM[0], ('the emblem must sit at her top right', r, p, b)
        hits = sum(1 for y in range(22) for x in range(18) if head.getpixel((x, y)) and (lambda q: q[3] >= 120 and sum(q[:3]) > 200)(e.getpixel((x + 18 + ox, y + 8 + oy))))
        assert hits == 0, ('the emblem covers her head', r, p, f, hits)
    assert len({art.emblem(r, f, p).tobytes() for f in range(art.EMBLEM_FRAMES)}) >= 12, ('emblem lost its animation', r, p)
# Chrome, chestnut hair, real glasses, and broken more with every rank.
assert all(abs(c[0] - c[2]) < 40 for c in art.STEEL.values()), 'copper or bronze is back in the gauntlets'
assert art.HAIR[0] > art.HAIR[2] + 30 and sum(art.HAIR[:3]) > 180, 'the hair must be brown, not near-black'
guard = art.costume(0, 0)
assert sum(1 for x in range(22, 36) for y in range(20, 24) if guard.getpixel((x, y))[:3] == art.WHITE[:3]) >= 2, 'the visor lost its crack'
assert sum(1 for x in range(22, 36) if guard.getpixel((x, 21))[:3] in (art.VISOR[:3], art.VISOR_L[:3])) >= 8, 'the visor must wrap her eyes'
assert any(guard.getpixel((x, 26))[:3] == art.SKIN[:3] for x in range(24, 34)), 'the lower face must stay bare'
assert any(guard.getpixel((x, 17))[:3] in (art.STEEL['metal'][:3], art.STEEL['light'][:3]) for x in range(24, 33)), 'the brow plate is missing'
# The white sci-fi outfit: a white chest plate and boots over a dark bodysuit, with cyan running lights.
white = lambda c: min(c[:3]) >= 190
assert all(white(c) for c in (art.TUNIC['coat'], art.TUNIC['light'], art.BOOT, art.BOOT_L, art.BOOT_D)), 'the chest plate and boots must be white'
assert max(art.SHIRT[:3]) < 90, 'the bodysuit stays dark so the white plate reads against the chrome'
plate = sum(1 for x in range(17, 35) for y in range(28, 42) if guard.getpixel((x, y))[:3] in (art.TUNIC['coat'][:3], art.TUNIC['light'][:3]))
assert plate >= 12, ('the white chest plate must show in the guard', plate)
feet = [art.body('idle', 0).getpixel((x, y))[:3] for x in range(6, 42) for y in range(54, 60)]   # legs live on the body sheet
assert sum(1 for c in feet if c in (art.BOOT[:3], art.BOOT_L[:3], art.BOOT_D[:3])) >= 10 and art.SOLE[:3] in feet and art.HEX[:3] in feet, 'white boots with a dark sole and a cyan light'
def damage(look):
    cols = {art.SCAR_EDGE[:3], art.SCAR_MID[:3], art.SCAR_VOID[:3], art.FRINGE_C[:3], art.FRINGE_M[:3]}
    return sum(1 for f in (0, 2, 4, 6) for px in art.costume(look, f).get_flattened_data() if px[3] and px[:3] in cols)
order = [0, 3, 6, 7, 8]
counts = [damage(l) for l in order]
assert all(a < b for a, b in zip(counts, counts[1:])), ('she must look more broken with every rank', counts)
area = lambda im: (lambda b: (b[2] - b[0]) * (b[3] - b[1]))(im.getbbox())
assert area(art.back_scar('wound', 2)) < area(art.back_scar('top1', 2)), '#1 tear is the biggest scar'
for name in ('back', 'badge', 'shards_orbit', 'hexes'):
    assert not hasattr(art, name), ('old aura code is still there', name)

# Anatomy (the guard, rank 0): a wide staggered stance with a bent front knee and the back heel up, a waist
# that is not pinched, both elbows showing, the lead knuckles facing the opponent, glasses that are frames
# a visor that stays dark glass with only a crack and a signal lit, and the cloak trailing behind the torso.
def width(im, y):
    xs = [x for x in range(im.width) if im.getpixel((x, y))[3]]
    return xs[-1] - xs[0] + 1 if xs else 0
assert 10 <= max(width(art.torso_only(0), y) for y in range(36, 39)) <= 13, 'waist width'
legs = Image.new('RGBA', (48, 64))
art.legs(ImageDraw.Draw(legs), 'idle', 0)
ground = [x for x in range(48) if legs.getpixel((x, 58))[3]]
assert ground and max(y for y in range(64) for x in range(48) if legs.getpixel((x, y))[3]) == 58, 'feet left the anchor'
S = art.STANCE
assert S['ankle_f'][0] - S['ankle_b'][0] >= 13, 'stance too narrow'
assert S['knee_f'][0] > S['hip_f'][0] + 3 and S['knee_f'][0] > S['ankle_f'][0], 'front knee not bent forward'
heel = [y for y in range(64) if legs.getpixel((S['ankle_b'][0] - 3, y))[3]]
assert heel and max(heel) <= 57, 'back heel not lifted'
G = art.costume(0, 0)
assert art.G_LEAD['orient'] == 'right' and art.G_REAR['orient'] == 'right', 'guard knuckles face the opponent'
for spec in (art.G_LEAD, art.G_REAR):
    ex, ey = spec['elbow']
    assert G.getpixel((ex, ey))[3], ('an elbow is hidden', spec['elbow'])
face = G.crop((22, 19, 34, 25))
bright = sum(1 for px in face.get_flattened_data() if px[3] and min(px[:3]) >= 230)
assert 1 <= bright <= 3, ('the visor glass must stay dark: a crack, not a light bar', bright)
cloak_cols = {art.CLOAK['deep'][:3], art.CLOAK['warm'][:3], art.CLOAK['hi'][:3]}
cl = [(x, y) for y in range(64) for x in range(48) if G.getpixel((x, y))[:3] in cloak_cols and G.getpixel((x, y))[3]]
assert cl and max(x for x, _ in cl) <= 25 and min(x for x, _ in cl) <= 6 and max(y for _, y in cl) <= 39, 'the cloak must trail behind, short of the rear elbow'
mantle = [x for x in range(48) if G.getpixel((x, 29))[:3] in cloak_cols]
assert len(mantle) >= 6, ('the cloak must start wide at the shoulder', len(mantle))
# Layering: every overlay frame covers the body sheet's upper body (the body never changes above the legs).
mask = art.body_mask()
bodies = {art.body(k, f).crop((0, 0, 48, 42)).getchannel('A').point(lambda v: 255 if v else 0).tobytes()
          for k in ('idle', 'run') for f in range(8)}   # attack and hit only add effects around her
assert len(bodies) == 1, 'the body sheet moved above the legs'
for pose in art.POSES:
    for look in range(art.LOOKS):
        for f in range(8):
            al = art.costume(look, f, pose).getchannel('A').point(lambda v: 255 if v == 255 else 0)
            assert not ImageChops.subtract(mask, al).getbbox(), ('overlay leaks the body below', pose, look, f)
# Reality is glass: rift shapes carry the scar palette, and no rune rings or atlas grid remain.
rift_src = (ROOT / 'Claude outputs/vesper/rift_art.py').read_text()
assert 'GRID' not in rift_src, 'the atlas grid is back in the rift art'
ra = script('rift_art.py')
for kind in ('tear', 'maw', 'sinkhole'):
    cols = {px[:3] for px in ra.rift_shape(kind, 4, 2).get_flattened_data() if px[3] == 255}
    assert art.SCAR_MID[:3] in cols and art.SCAR_VOID[:3] in cols, (kind, 'lost the scar layers')

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
print(f'Verified {len(views)} views over {len(sheets)} sheets: {len(expected)} native effect names and {len(buffs)} pose / rank / HUD layers resolve; '
      f'every atlas <= 2048 px; 8 frames each; regeneration byte-identical; decoded RGBA {decoded / 2 ** 20:.1f} MiB (budget 190).')
