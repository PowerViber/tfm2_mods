"""Reproducibly extract imagegen concepts. Review files only; no game assets."""
from pathlib import Path
import hashlib
import json
from PIL import Image, ImageDraw, ImageFont, ImageOps

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / 'docs/kratos-art-review'
FONT = ImageFont.truetype('C:/Windows/Fonts/segoeui.ttf', 15)
NEAREST = Image.Resampling.NEAREST
POSES = ('front', 'axe-idle', 'blades-idle', 'movement', 'cleave', 'sweep', 'guard', 'smash')


def crop_cells(path, columns, rows):
    source = Image.open(path).convert('RGBA')
    pieces = []
    for i in range(columns * rows):
        cell = source.crop((i % columns * source.width // columns,
                            i // columns * source.height // rows,
                            (i % columns + 1) * source.width // columns,
                            (i // columns + 1) * source.height // rows))
        cell.putalpha(cell.getchannel('A').point(lambda a: 255 if a >= 180 else 0))
        pieces.append(cell.crop(cell.getbbox()))
    return pieces


def put(board, image, x, y, scale=1):
    image = image.resize((image.width * scale, image.height * scale), NEAREST)
    board.paste(image, (x, y), image)


def stock(name):
    folder = ROOT / 'Sprite kit/base champions'
    frame = json.loads((folder / (name + '.anim.json')).read_text())['anims']['idle']['frames'][0]['data']
    source = Image.open(folder / (name + '.png')).convert('RGBA')
    return source.crop((frame['x'], frame['y'], frame['x'] + frame['w'], frame['y'] + frame['h']))


def build():
    pieces = crop_cells(OUT / 'body-source-v1.png', 4, 2)
    scale = 40 / pieces[0].height
    poses = {}
    for name, piece in zip(POSES, pieces):
        image = piece.resize((round(piece.width * scale), round(piece.height * scale)), NEAREST)
        image.putalpha(image.getchannel('A').point(lambda a: 255 if a >= 180 else 0))
        canvas = Image.new('RGBA', (64, 56))
        canvas.alpha_composite(image, ((64 - image.width) // 2, 49 - image.height))
        canvas.save(OUT / ('body-' + name + '.png'))
        ImageOps.mirror(canvas).save(OUT / ('body-' + name + '-left.png'))
        poses[name] = canvas
    board = Image.new('RGB', (1040, 1830), '#172936')
    draw = ImageDraw.Draw(board)
    draw.text((18, 12), 'KRATOS / original appearance proposal / 1x and 3x / review only', font=FONT, fill='#eef4fa')
    for row, (name, pose) in enumerate(poses.items()):
        y = 44 + row * 190
        draw.text((18, y), name.upper(), font=FONT, fill='#ffb65e')
        for side in range(2):
            x = 160 + side * 430
            draw.rectangle((x, y, x + 415, y + 178), fill='#b6c4a4' if side else '#203747')
            image = ImageOps.mirror(pose) if side else pose
            put(board, image, x + 12, y + 101)
            put(board, image, x + 125, y + 5, 3)
            draw.line((x, y + 149, x + 415, y + 149), fill='#708b88')
    draw.text((18, 1585), 'STOCK SCALE / Knight, Berserker and Kratos (1x / 3x)', font=FONT, fill='#eef4fa')
    for i, (name, pose) in enumerate([('Knight', stock('knight')), ('Berserker', stock('berserker')), ('Kratos', poses['axe-idle'])]):
        x = 18 + i * 335
        draw.text((x, 1620), name, font=FONT, fill='#b8d1df')
        put(board, pose, x + 15, 1728 - pose.height)
        put(board, pose, x + 105, 1790 - pose.height * 3, 3)
    board.save(OUT / 'body-comparison.png')

    wolves = crop_cells(OUT / 'fenrir-source-v1.png', 2, 2)
    wolf_scale = 25 / wolves[0].height
    board = Image.new('RGB', (1040, 520), '#172936')
    draw = ImageDraw.Draw(board)
    draw.text((18, 12), 'FENRIR / game-width reference 25 px / 1x and 3x / review only', font=FONT, fill='#eef4fa')
    for i, (name, piece) in enumerate(zip(('east', 'west', 'front', 'stride'), wolves)):
        image = piece.resize((round(piece.width * wolf_scale), round(piece.height * wolf_scale)), NEAREST)
        image.save(OUT / ('fenrir-' + name + '.png'))
        y = 45 + i * 110
        draw.text((18, y + 12), name.upper(), font=FONT, fill='#ffb65e')
        for side in range(2):
            x = 160 + side * 430
            draw.rectangle((x, y, x + 415, y + 100), fill='#b6c4a4' if side else '#203747')
            put(board, poses['axe-idle'], x + 5, y + 40)
            put(board, image, x + 80, y + 62 - image.height)
            put(board, image, x + 165, y + (100 - image.height * 3) // 2, 3)
    board.save(OUT / 'fenrir-comparison.png')
    approvals=json.loads((OUT/'approvals.json').read_text(encoding='utf-8')) if (OUT/'approvals.json').exists() else {}
    report = {'status': 'approved_appearance_concepts' if all(approvals.get(g,{}).get('approved') for g in ('body','fenrir')) else 'review_only_awaiting_appearance_approval', 'game_assets_written': False,
              'source_scale': scale, 'concept_canvas': [64, 56], 'foot_anchor': [32, 48],
              'sources': {p.name: hashlib.sha256(p.read_bytes()).hexdigest()
                          for p in sorted(OUT.glob('*-source-v1.png'))}}
    (OUT / 'proposal.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    images = {'body-' + name: 'body-' + name + '.png' for name in POSES}
    images.update({'fenrir-' + name: 'fenrir-' + name + '.png' for name in ('east', 'west', 'front', 'stride')})
    manifest = {'version': 1, 'revision': 'kratos-art-1', 'review_only': True,
                'complete_body': False, 'units_per_pixel': 950, 'images': images,
                'anchors': {'body-' + name: [32, 48] for name in POSES},
                'body': {name: [{'image': 'body-' + name, 'ticks': 60}] for name in POSES}}
    if not (OUT/'art-manifest.json').exists():
        (OUT / 'presentation.json').write_text(json.dumps(manifest, indent=2) + '\n', encoding='utf-8')
    print(json.dumps(report))


if __name__ == '__main__':
    build()
