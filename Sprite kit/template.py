import os, json
from PIL import Image, ImageDraw, ImageFont
OUT = os.path.expanduser('~/mnt/tfm2/Sprite kit')
font = ImageFont.load_default()
W, H = 48, 56                      # one frame
ROWS = [('idle', 4, 0.18), ('run', 6, 0.08), ('attack', 4, 0.08), ('skill1', 4, 0.08), ('skill2', 4, 0.08), ('ult', 4, 0.1), ('hit', 1, 0.14), ('dead', 6, 0.1)]
cols = max(n for _, n, _ in ROWS)
blank = Image.new('RGBA', (cols * W, len(ROWS) * H), (0, 0, 0, 0))
blank.save(f'{OUT}/NEW CHAMPION template.png')
fan = {'anims': {t: {'frames': [{'duration': d, 'data': {'x': i * W, 'y': r * H, 'w': W, 'h': H}} for i in range(n)]} for r, (t, n, d) in enumerate(ROWS)}}
json.dump(fan, open(f'{OUT}/NEW CHAMPION template.anim.json', 'w'), indent=1)
# the guide: 4x, boxes, labels, the frame centre (the anchor), the feet line, and Steve's idle as a faint size reference
S = 4
g = Image.new('RGBA', (cols * W * S, len(ROWS) * H * S), (34, 36, 44, 255))
d = ImageDraw.Draw(g)
steve = Image.open(f'{OUT}/mods/tfm2_blockcraft/champions/tfm2_blockcraft_steve.png').convert('RGBA').crop((0, 0, 36, 44))
ghost = steve.copy(); ghost.putalpha(ghost.getchannel('A').point(lambda a: a * 45 // 255))
ghost = ghost.resize((36 * S, 44 * S), Image.NEAREST)
for r, (t, n, dur) in enumerate(ROWS):
    for i in range(n):
        x, y = i * W * S, r * H * S
        for yy in range(0, H * S, 8):
            for xx in range(0, W * S, 8):
                if (xx // 8 + yy // 8) % 2: d.rectangle([x + xx, y + yy, x + xx + 7, y + yy + 7], fill=(42, 44, 54, 255))
        g.alpha_composite(ghost, (x + (W - 36) * S // 2, y + (H - 44) * S // 2))
        cx, cy = x + W * S // 2, y + H * S // 2
        d.line([cx - 10, cy, cx + 10, cy], fill=(255, 220, 90, 180)); d.line([cx, cy - 10, cx, cy + 10], fill=(255, 220, 90, 180))
        feet = y + ((H - 44) // 2 + 42) * S   # the bottom edge of the ghost's boots (20 px below the centre)
        d.line([x, feet, x + W * S - 1, feet], fill=(90, 220, 255, 160))
        d.rectangle([x, y, x + W * S - 1, y + H * S - 1], outline=(150, 150, 170, 255))
        d.text((x + 3, y + 2), f'{t} {i}', fill=(255, 255, 255, 255), font=font)
        d.text((x + 3, y + H * S - 12), f'{dur:.2f}s', fill=(190, 190, 190, 255), font=font)
g.save(f'{OUT}/NEW CHAMPION template_guide.png')
print(blank.size, g.size)
