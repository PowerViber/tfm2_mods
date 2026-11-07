"""Register complete idle drawings to the approved walk; never slice anatomy."""
from pathlib import Path
from PIL import Image, ImageDraw, ImageOps
import hashlib, json, statistics

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / 'docs/grimhald-art-review'
rules = json.loads((ROOT / 'rules/grimhald-art.json').read_text())

def cells(name):
    source = Image.open(OUT / name).convert('RGBA')
    assert source.width == source.height and source.width % 2 == 0
    side = source.width // 2
    result = []
    for i in range(4):
        cell = source.crop((i % 2 * side, i // 2 * side, (i % 2 + 1) * side, (i // 2 + 1) * side))
        cell.putalpha(cell.getchannel('A').point(lambda a: a if a >= 180 else 0))
        result.append(cell)
    return result

armed = cells('source-idle-v5.png')
unarmed = cells('source-idle-unarmed-v5.png')
# One scale for the complete sheet and both weapon states. Fallen/crouched
# poses must never be individually inflated to a target bounding-box height.
scale = 39 / statistics.median(c.getbbox()[3] - c.getbbox()[1] for c in unarmed)
registrations = {}
frames = {}
for name, source in [('idle', armed), ('idle_unarmed', unarmed)]:
    output, anchors = [], []
    for i, cell in enumerate(source):
        # The eye strip registers the same helmet landmark as the walk sheet.
        side = cell.width
        eye = [(x, y) for y in range(round(side * .24), round(side * .36))
               for x in range(round(side * .49), round(side * .68))
               if (lambda p: p[3] >= 180 and p[0] > 190 and 75 < p[1] < 225 and p[2] < 100)(cell.getpixel((x, y)))]
        assert eye, (name, i, 'eye landmark missing')
        eye_x = statistics.mean(x for x, _ in eye)
        small = cell.resize((round(side * scale), round(side * scale)), Image.Resampling.NEAREST)
        box = small.getbbox()
        frame = Image.new('RGBA', (96, 80))
        offset = (52 - round(eye_x * scale), 60 - box[3])
        frame.alpha_composite(small, offset)
        assert frame.getbbox()[3] == 60
        output.append(frame)
        anchors.append({'eyeX': eye_x, 'paste': list(offset)})
    frames[name] = output
    registrations[name] = anchors

walk = Image.open(OUT / 'walk-v4-preview.png').convert('RGBA')
walk_meta = json.loads((OUT / 'walk-v4-preview.fanim').read_text())['anims']
for name in ['run', 'run_unarmed']:
    frames[name] = []
    for entry in walk_meta[name]['frames']:
        d = entry['data']
        frames[name].append(walk.crop((d['x'], d['y'], d['x'] + d['w'], d['y'] + d['h'])))

atlas = Image.new('RGBA', (1024, 512))
meta = {'anims': {}}
index = 0
for name, drawings in frames.items():
    duration = rules['frameMillis']['run' if name.startswith('run') else 'idle']
    for left in [False, True]:
        entries = []
        for i, im in enumerate(drawings):
            x, y = index % 10 * 98, index // 10 * 82
            atlas.alpha_composite(ImageOps.mirror(im) if left else im, (x, y))
            entries.append({'duration': duration[i] / 1000, 'data': {'x': x, 'y': y, 'w': 96, 'h': 80}})
            index += 1
        meta['anims'][name + ('_l' if left else '')] = {'frames': entries}
atlas.save(OUT / 'movement-v5-preview.png')
(OUT / 'movement-v5-preview.fanim').write_text(json.dumps(meta, indent=2) + '\n')
frames['idle'][0].save(OUT / 'idle-v5.png')
frames['idle_unarmed'][0].save(OUT / 'idle-unarmed-v5.png')

def panel(items, title, left=False):
    sheet = Image.new('RGB', (1040, 300))
    d = ImageDraw.Draw(sheet)
    for i, (label, im) in enumerate(items):
        x = i * 260
        bg = '#b7c2ae' if left else '#19313b'
        d.rectangle((x, 0, x + 259, 299), fill=bg)
        colour = '#23313b' if left else '#efbb86'
        d.text((x + 12, 10), label, fill=colour)
        if left: im = ImageOps.mirror(im)
        sheet.paste(im, (x + 2, 36), im)
        big = im.resize((288, 240), Image.Resampling.NEAREST)
        # Full canvas may overlap panel margins, but the visible body never does.
        sheet.paste(big, (x - 20, 70), big)
        d.line((x + 30, 250, x + 245, 250), fill='#6c8180')
    return sheet

old = Image.open(OUT / 'connected-v3-idle.png').convert('RGBA')
panel([('Old idle (superseded)', old), ('Matched idle', frames['idle'][0]),
       ('Approved walk', frames['run'][0]), ('Matched unarmed', frames['idle_unarmed'][0])], '').save(OUT / 'idle-v5-comparison.png')

# Four seconds of the actual distance-driven timing: idle -> walk -> idle.
# The original six walk drawings are unchanged; the new nominal cycle is .96s.
gif = []
nominal_speed = 1030 * 60
for tick in range(0, 240, 6):
    if 60 <= tick < 180:
        phase = (tick - 60) / 60 * nominal_speed / rules['walkStrideWorld']
        a = frames['run'][int(phase * 6) % 6]
        u = frames['run_unarmed'][int(phase * 6) % 6]
        label = 'Walking / same light and scale'
    else:
        a = frames['idle'][int(tick / 60 / .18) % 4]
        u = frames['idle_unarmed'][int(tick / 60 / .18) % 4]
        label = 'Idle / same light and scale'
    gif.append(panel([(label, a), ('Unarmed', u), ('Left / light', a), ('Unarmed / left', u)], ''))
    # Exact mirrors on the two light-background panels.
    d = ImageDraw.Draw(gif[-1])
    for col, im in [(2, a), (3, u)]:
        x = col * 260
        d.rectangle((x, 0, x + 259, 299), fill='#b7c2ae')
        d.text((x + 12, 10), 'Left / light terrain', fill='#23313b')
        im = ImageOps.mirror(im)
        gif[-1].paste(im, (x + 2, 36), im)
        big = im.resize((288, 240), Image.Resampling.NEAREST)
        gif[-1].paste(big, (x - 20, 70), big)
        d.line((x + 30, 250, x + 245, 250), fill='#8c9a8a')
gif[0].save(OUT / 'idle-walk-v5.gif', save_all=True, append_images=gif[1:], duration=100, loop=0, disposal=2)

info = json.loads((ROOT / 'editor/grimhald-art.json').read_text())
info.update(revision=5, cycleAtlas='../docs/grimhald-art-review/movement-v5-preview.png',
            cycleMetadata='../docs/grimhald-art-review/movement-v5-preview.fanim',
            walkStrideWorld=rules['walkStrideWorld'],
            status='Matched idle/walk palette and body scale; walking follows distance travelled. Remaining action/death cycles and game-pack integration are pending.')
info['candidateImages'] = {'idle': '../docs/grimhald-art-review/idle-v5.png',
                           'unarmed': '../docs/grimhald-art-review/idle-unarmed-v5.png'}
(ROOT / 'editor/grimhald-art.json').write_text(json.dumps(info, indent=2) + '\n')
manifest = {'sources': {name: hashlib.sha256((OUT / name).read_bytes()).hexdigest()
                        for name in ['source-idle-v5.png', 'source-idle-unarmed-v5.png']},
            'scale': scale, 'registrations': registrations, 'footBelowCentre': 20,
            'walkStrideWorld': rules['walkStrideWorld'], 'approvedWalkPixelsUnchanged': True,
            'method': 'Whole-frame nearest-neighbour registration; no anatomy slicing, relighting or per-frame body resizing',
            'remaining': 'Attack, casts, hit and death are not finished; generic idle fallback is explicitly identified.'}
(OUT / 'movement-v5-manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
print(json.dumps(manifest, indent=2))
