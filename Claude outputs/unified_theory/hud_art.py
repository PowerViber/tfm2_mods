"""Compact, independently indexed scientist orb and charge bar.

The orb occupies 12 native pixels, the bar 42 x 5, with a three-pixel gap.
Neither preparation nor charge state multiplies the other atlas's entries.
"""
import math
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

COLORS = [(99, 191, 255), (255, 201, 107), (99, 225, 189)]
DARK = (12, 20, 32, 245)
WHITE = (232, 241, 234, 255)


def orb(science, progress, status, frame):
    im = Image.new('RGBA', (12, 12))
    d = ImageDraw.Draw(im)
    c = COLORS[science] + (255,)
    d.ellipse((1, 1, 10, 10), fill=DARK, outline=(59, 77, 94, 255))
    points = [(6, 1), (9, 2), (10, 6), (9, 9), (6, 10), (2, 9), (1, 6), (2, 2)]
    for p in points[:progress]:
        d.point(p, fill=c)
    if science == 0:
        d.line((4, 6, 8, 6), fill=c)
        d.line((6, 4, 6, 8), fill=c)
        d.point((6, 6), fill=WHITE)
    elif science == 1:
        d.polygon([(6, 4), (8, 5), (8, 7), (6, 8), (4, 7), (4, 5)], outline=c)
        d.line((4, 5, 6, 6, 8, 5), fill=WHITE)
        d.line((6, 6, 6, 8), fill=c)
    else:
        d.line((4, 7, 6, 4, 8, 7, 4, 7), fill=(45, 126, 111, 255))
        for p in [(4, 7), (6, 4), (8, 7)]:
            d.point(p, fill=c)
        d.point((6, 6), fill=WHITE)
    # One baked glint, with eight discrete locations, supplies the entire idle
    # animation. No independently spawned particle or orbiting buff is needed.
    d.point(points[frame], fill=WHITE if status == 0 else (166, 189, 201, 255))
    if status:
        warning = (255, 201, 107, 255) if status == 1 else (255, 121, 142, 255)
        d.point((10, 2), fill=warning)
        if frame in (2, 3):
            d.line((9, 1, 11, 3), fill=warning)
        if status == 3:
            d.line((5, 3, 6, 5, 5, 7, 7, 9), fill=DARK)
            d.point((6, 5), fill=warning)
    return im


def bar(a,b,c):
    cells=(a,b,c);assert all(0<=n<=11 for n in cells)
    im=Image.new('RGBA',(42,5));d=ImageDraw.Draw(im)
    d.rectangle((0,0,41,4),fill=DARK,outline=(59,77,94,255))
    for i,n in enumerate(cells):
        left=1+i*14
        if n==11:
            for x in range(left,left+12):
                for y in range(1,4):d.point((x,y),fill=WHITE if (x+y)%3==0 else (132,127,112,255))
        elif n:
            width=12 if n==10 else max(1,n*12//10)
            d.rectangle((left,1,left+width-1,3),fill=(179,218,222,255) if n==10 else (73,130,157,255))
    return im

def composed(science,progress,status,a,b,c,frame):
    im=Image.new('RGBA',(57,12));im.alpha_composite(orb(science,progress,status,frame),(0,0))
    im.alpha_composite(bar(a,b,c),(15,4));return im


def generate(atlas, mod, root):
    orbs = {f'orb_{s}_{p}_{status}': [(orb(s, p, status, f), 1 / 60) for f in range(8)]
            for s in range(3) for p in range(9) for status in range(4)}
    bars={f'bar_{a}_{b}_{c}':[(bar(a,b,c),8/60)] for a in range(12) for b in range(12) for c in range(12)}
    atlas(mod / 'vfx', 'science_orbs', orbs, 12, 12, 32)
    atlas(mod / 'vfx', 'science_charge', bars, 42, 5, 32)
    # Retire the oversized runtime textures and their view definitions together.
    for name in ['science_notebook0', 'science_notebook1', 'science_notebook2',
                 'science_meters', 'science_allocations']:
        for suffix in ['#sheet.png', '#anim.fanim']:
            (mod / 'vfx' / (name + suffix)).unlink(missing_ok=True)
    font = ImageFont.truetype(('C:/Windows/Fonts/consola.ttf' if Path('C:/Windows/Fonts/consola.ttf').exists() else '/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf'), 14)
    examples=[(0,0,0,10,10,10,'Einstein / three ready charges'),
              (1,4,0,11,10,10,'Newton / one reserved charge'),
              (2,7,1,2,5,8,'Curie / independent five-second recovery'),
              (0,8,2,0,10,10,'Critical recipe / charge spent on commitment'),
              (2,0,3,10,10,10,'Three scientists / one shared bank')]
    frames = []
    for f in range(8):
        preview = Image.new('RGBA', (850, 430), '#0b1422')
        d = ImageDraw.Draw(preview)
        d.text((18, 16), 'IN-GAME HUD / 57 x 12 NATIVE PIXELS', font=font, fill=WHITE)
        d.text((18, 41), '12px form orb + 3px gap + 42px charge bar', font=font, fill=WHITE)
        for i, (s, p, status, a, b, c, label) in enumerate(examples):
            y = 82 + i * 68
            hud = composed(s, p, status, a, b, c, f)
            preview.alpha_composite(hud, (20, y + 18))
            preview.alpha_composite(hud.resize((228, 48), Image.Resampling.NEAREST), (100, y))
            d.text((350, y + 16), label, font=font, fill=COLORS[s] + (255,))
        frames.append(preview.convert('RGB'))
    frames[2].save(root / 'docs/unified-theory-hud.png')
    frames[0].save(root / 'docs/unified-theory-hud.gif', save_all=True,
                   append_images=frames[1:], duration=100, loop=0, disposal=2)
    print('Compact HUD: 108 eight-frame 12px orbs and 1728 three-cell 42x5 charge states')
