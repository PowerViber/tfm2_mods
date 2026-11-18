"""Reproducible extraction of the unapproved Merge concept; no shipped assets."""
from pathlib import Path
import hashlib
import json
from PIL import Image, ImageDraw, ImageFont, ImageOps

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / 'docs/contender-merge-review'
source = OUT / 'merge-source-v1.png'
im = Image.open(source).convert('RGBA')
assert im.size == (2172, 724)
scale = 42 / 519
anchors = [(245, 665), (220, 665), (210, 665), (248, 665)]
poses = ['idle', 'run', 'straight', 'uppercut']
sheet = Image.new('RGBA', (640, 64))
frames = []
rects = {}
for i, (pose, (ax, ay)) in enumerate(zip(poses, anchors)):
    crop = im.crop((543*i, 80, 543*(i+1), 690))
    crop = crop.resize((round(crop.width*scale), round(crop.height*scale)), Image.Resampling.NEAREST)
    tile = Image.new('RGBA', (80, 64))
    tile.alpha_composite(crop, (40-round(ax*scale), 52-round((ay-80)*scale)))
    for left in [False, True]:
        frame = tile
        if left:
            frame = Image.new('RGBA', tile.size)
            frame.alpha_composite(ImageOps.mirror(tile), (1, 0))
        x = len(frames)*80
        sheet.alpha_composite(frame, (x, 0))
        frames.append(frame)
        rects[pose+('_l' if left else '')] = dict(x=x, y=0, w=80, h=64)
sheet.save(OUT / 'merge-proposals.png')
approval_path = OUT/'approvals.json'
approval = json.loads(approval_path.read_text())['appearance'] if approval_path.exists() else {}
if approval.get('approved'):
    assert approval['source_sha256'] == hashlib.sha256(source.read_bytes()).hexdigest()
manifest = dict(revision='contender-merge-proposal-1', approved=approval.get('approved', False),
                file='../docs/contender-merge-review/merge-proposals.png',
                source_sha256=hashlib.sha256(source.read_bytes()).hexdigest(),
                sheet_sha256=hashlib.sha256((OUT/'merge-proposals.png').read_bytes()).hexdigest(),
                canvas=[80, 64], foot_anchor=[40, 52], frames=rects,
                note='Four appearance poses, not a completed animation set. Review only.')
(OUT/'review.json').write_text(json.dumps(manifest, indent=2)+'\n', encoding='utf8')
(ROOT/'editor/contender-merge-data.js').write_text('window.CONTENDER_MERGE_PREVIEW='+json.dumps(manifest, separators=(',', ':'))+';\n', encoding='utf8')

old = Image.open(ROOT/'docs/contender-shadow-review/body-proposals.png').convert('RGBA')
font = ImageFont.truetype('C:/Windows/Fonts/arial.ttf', 15)
review = Image.new('RGBA', (1024, 660), '#11232d')
d = ImageDraw.Draw(review)
d.text((18, 12), 'SHADOW MERGE — one fused body / appearance proposal, not approved', font=font, fill='#edf2f1')
for row, (bg, ink) in enumerate([('#29454a', '#f4e9d4'), ('#bac5a6', '#152b35')]):
    y = 46+row*300
    d.rectangle((12, y, 1012, y+285), fill=bg)
    for col, pose in enumerate(poses):
        x = 20+col*250
        d.text((x, y+10), pose+' / '+('left' if row else 'right'), font=font, fill=ink)
        normal = old.crop(((col*2+row)*80, 0, (col*2+row+1)*80, 64))
        merged = frames[col*2+row]
        for j, tile in enumerate([normal, merged]):
            centre = x+65+j*115
            review.alpha_composite(tile, (centre-40, y+45))
            review.alpha_composite(tile.resize((240, 192), Image.Resampling.NEAREST), (centre-120, y+105))
        d.text((x+36, y+275), 'normal       merged  |  1x / 3x', font=font, fill=ink)
review.convert('RGB').save(OUT/'merge-comparison.png')
print(json.dumps(manifest))
