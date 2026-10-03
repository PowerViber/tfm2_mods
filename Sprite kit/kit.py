import os, json, struct, glob, io
from PIL import Image, ImageDraw, ImageFont
ROOT = os.path.expanduser('~/mnt')
GAME = ROOT + '/Teamfight Manager2'
OUT = ROOT + '/tfm2/Sprite kit'
os.makedirs(OUT, exist_ok=True)
font = ImageFont.load_default()
COLS = [(255,90,90),(90,200,255),(120,230,120),(255,200,60),(220,120,255),(255,150,90),(90,255,210),(255,120,200),(170,170,255),(200,255,120),(255,255,255),(160,220,255)]

def guide(sheet, anims, scale):
    """The sheet blown up with every frame boxed and labelled 'tag #i'."""
    W, H = sheet.size
    bg = Image.new('RGBA', (W * scale, H * scale + 0), (34, 36, 44, 255))
    # checker so transparency shows
    d0 = ImageDraw.Draw(bg)
    for y in range(0, H * scale, 8 * scale // 2 or 8):
        for x in range(0, W * scale, 8 * scale // 2 or 8):
            if (x // max(1, 4 * scale) + y // max(1, 4 * scale)) % 2: d0.rectangle([x, y, x + 4 * scale - 1, y + 4 * scale - 1], fill=(44, 46, 56, 255))
    bg.alpha_composite(sheet.resize((W * scale, H * scale), Image.NEAREST))
    d = ImageDraw.Draw(bg)
    for ti, (tag, a) in enumerate(anims.items()):
        c = COLS[ti % len(COLS)]
        for i, f in enumerate(a['frames']):
            r = f['data']; x, y, w, h = r['x'] * scale, r['y'] * scale, r['w'] * scale, r['h'] * scale
            d.rectangle([x, y, x + w - 1, y + h - 1], outline=c + (255,), width=1)
            d.text((x + 2, y + 1), f"{tag} {i}", fill=c + (255,), font=font)
            d.text((x + 2, y + h - 11), f"{f['duration']:.2f}s", fill=(200, 200, 200, 255), font=font)
    return bg

def write_set(folder, name, png_bytes, fanim, scale):
    os.makedirs(folder, exist_ok=True)
    sheet = Image.open(io.BytesIO(png_bytes)).convert('RGBA')
    open(f'{folder}/{name}.png', 'wb').write(png_bytes)
    json.dump(fanim, open(f'{folder}/{name}.anim.json', 'w'), indent=1)
    guide(sheet, fanim['anims'], scale).save(f'{folder}/{name}_guide.png')
    return sheet

overview = []   # (label, first idle frame image)
def first_frame(sheet, fanim):
    a = fanim['anims']
    tag = 'idle' if 'idle' in a else next(iter(a))
    r = a[tag]['frames'][0]['data']
    return sheet.crop((r['x'], r['y'], r['x'] + r['w'], r['y'] + r['h']))

# 1) the mod sprites (champions + effects)
for png in sorted(glob.glob(GAME + '/mods/*/*/*#sheet.png')):
    mod = png.split('/mods/')[1].split('/')[0]
    kind = png.split('/')[-2]
    base = os.path.basename(png)[:-len('#sheet.png')]
    fan = png.replace('#sheet.png', '#anim.fanim')
    if not os.path.exists(fan): continue
    fanim = json.load(open(fan))
    sheet = write_set(f'{OUT}/mods/{mod}/{kind}', base, open(png, 'rb').read(), fanim, 4 if kind == 'champions' else 2)
    if kind == 'champions': overview.append((base.replace('tfm2_', ''), first_frame(sheet, fanim)))

# 2) the base game's champion sprites, read from bundle.game_data (reference for size, poses and tags)
pngs, fans = {}, {}
with open(GAME + '/bundle.game_data', 'rb') as f:
    n = struct.unpack('<I', f.read(4))[0]
    for _ in range(n):
        tl = struct.unpack('<I', f.read(4))[0]; typ = f.read(tl).decode()
        kl = struct.unpack('<I', f.read(4))[0]; key = f.read(kl).decode()
        sz = struct.unpack('<I', f.read(4))[0]
        if key.startswith('asset/base/aseprite_resources/champions/') and typ in ('png', 'fanim') and '#' in key:
            data = f.read(sz)
            nm, part = key.rsplit('/', 1)[1].split('#')
            (pngs if part == 'sheet' else fans)[nm] = data
        else:
            f.seek(sz, 1)
for nm in sorted(pngs):
    if nm not in fans: continue
    fanim = json.loads(fans[nm])
    sheet = write_set(f'{OUT}/base champions', nm, pngs[nm], fanim, 3)
    overview.append(('base: ' + nm, first_frame(sheet, fanim)))

# 3) one overview of every champion (first idle frame, 3x)
cell = 72 * 3; per = 10
rows = (len(overview) + per - 1) // per
ov = Image.new('RGBA', (per * cell, rows * (cell + 16)), (34, 36, 44, 255))
d = ImageDraw.Draw(ov)
for k, (label, im) in enumerate(overview):
    x, y = (k % per) * cell, (k // per) * (cell + 16)
    im3 = im.resize((im.width * 3, im.height * 3), Image.NEAREST)
    if im3.width > cell or im3.height > cell: im3.thumbnail((cell, cell), Image.NEAREST)
    ov.alpha_composite(im3, (x + (cell - im3.width) // 2, y + (cell - im3.height) // 2))
    d.text((x + 4, y + cell + 2), label[:30], fill=(230, 230, 230, 255), font=font)
ov.save(f'{OUT}/ALL SPRITES overview.png')
print(len(overview), 'champions;', len(pngs), 'base sheets')
