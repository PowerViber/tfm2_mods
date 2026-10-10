"""Compact, shared-anatomy scientist forms and a planted eight-cel morph.

Coordinates are native pixels on the existing 48 x 64 anchor. Costume layers
cover the common coat/head, leaving the host's moving legs visible.
"""
import math
import json
from PIL import Image, ImageDraw, ImageFont

INK = '#131a25'
COLORS = ['#63bfff', '#ffc96b', '#63e1bd']
SKIN = '#d8b492'


def body(kind='idle', f=0, persona=None):
    im = Image.new('RGBA', (48, 64)); d = ImageDraw.Draw(im)
    if kind == 'dead':
        d.polygon([(13,53),(17,49),(30,49),(35,54),(32,58),(14,58)], fill=INK)
        d.rectangle((16,50,27,54), fill='#e4e4d7')
        d.rectangle((29,51,33,55), fill=SKIN)
        return im
    # Shoes stay on the old ground line; the shorter body grows upwards from it.
    step = (1 if f % 4 < 2 else -1) if kind == 'run' else 0
    d.rectangle((20,44,23,55+step), fill=INK)
    d.rectangle((26,44,29,55-step), fill=INK)
    d.line((21,46,21,54+step), fill='#405066')
    d.line((27,46,27,54-step), fill='#293849')
    d.rectangle((18,55+step,23,58+step), fill=INK)
    d.rectangle((25,55-step,30,58-step), fill=INK)
    d.line((19,55+step,22,55+step), fill='#6b7380')
    d.line((26,55-step,29,55-step), fill='#505c6b')
    # Default sprite in the champion picker is Einstein, not the old bald body.
    im.alpha_composite(costume(0 if persona is None else persona, 0, f))
    return im


def costume(science, rank=0, f=0, hands=0, prop=True):
    im = Image.new('RGBA', (48,64)); d = ImageDraw.Draw(im)
    c = COLORS[science]
    coat = ['#e4e4d7', '#303342', '#d2dfd6'][science]
    shade = ['#a4b5b7', '#202330', '#8ea9a4'][science]
    light = ['#f7f4df', '#727987', '#edf3e6'][science]
    # Slim shoulders, small head, long dark legs; all forms share their anchors.
    d.rectangle((22,27,26,31), fill=SKIN)
    hem = 50 if science == 1 else 46
    d.polygon([(19,30),(28,30),(31,35),(31,hem),(27,hem),(24,43),(21,hem),(17,hem),(17,35)], fill=INK)
    d.polygon([(19,31),(23,32),(23,42),(20,hem-1),(18,hem-1),(18,35)], fill=coat)
    d.polygon([(26,31),(29,34),(30,hem-1),(27,hem-1),(25,42),(25,33)], fill=shade)
    d.rectangle((23,32,25,43), fill='#2a3546')
    d.line((19,32,22,35,21,43), fill=light)
    d.line((27,32,25,35), fill=light)
    for y in (36,39,42): d.point((24,y), fill='#bcc5c4')
    # Relaxed arms sit against the coat; gathering hands moves within this envelope.
    d.polygon([(18,32),(16,34),(15,42),(17,45),(19,42),(19,35)], fill=INK)
    d.polygon([(17,34),(16,41),(18,42),(18,35)], fill=coat)
    d.polygon([(29,33),(32,34),(34,42),(32,45),(30,42)], fill=INK)
    d.line((31,35,33,41), fill=shade, width=2)
    d.rectangle((16,42,18,44), fill=SKIN)
    d.rectangle((32,42,34,44), fill=SKIN)
    if hands:
        # Opaque sleeves cover the original relaxed hands before gathering inward.
        d.rectangle((16,41,18,44), fill=coat)
        d.rectangle((32,41,34,44), fill=shade)
        y = 41 - min(hands,2)*2
        d.line((17,39,21,y), fill=shade, width=3)
        d.line((32,39,27,y), fill=shade, width=3)
        d.rectangle((21,y-1,23,y+1), fill=SKIN)
        d.rectangle((26,y-1,28,y+1), fill=SKIN)
    d.rectangle((19,38,21,40), fill=shade)
    # A ten-pixel face, shaded on the right, with a consistent three-quarter gaze.
    d.polygon([(19,19),(28,19),(29,23),(28,28),(25,30),(20,28),(18,24)], fill=INK)
    d.polygon([(20,20),(27,20),(28,23),(27,27),(25,29),(21,27),(19,24)], fill=SKIN)
    d.line((27,21,27,26), fill='#ad886e')
    d.point((21,24), fill=INK); d.point((26,24), fill=INK)
    d.point((24,26), fill='#9b755d')
    # The common scalp footprint is covered by every form, avoiding ghost hair.
    scalp = [(16,20),(17,16),(20,14),(23,13),(29,14),(32,17),(32,21),(28,22),(27,20),(20,20),(18,23)]
    if science == 0:
        d.polygon(scalp, fill=INK)
        d.polygon([(17,19),(17,17),(20,17),(19,15),(23,16),(23,14),(26,16),(29,15),(28,18),(31,17),(30,20),(27,20),(25,19),(20,19),(18,22)], fill='#f1f0e8')
        d.line((20,18,24,17), fill='#c0cbd0')
        d.line((21,27,26,27), fill='#f8f4e7')
        d.point((23,28), fill='#d0d5ca')
    elif science == 1:
        d.polygon([(16,20),(17,16),(20,14),(23,13),(29,14),(32,17),(33,32),(29,34),(28,28),(28,21),(20,20),(20,28),(18,34),(15,32)], fill=INK)
        d.polygon([(17,19),(18,16),(22,15),(28,15),(31,18),(31,31),(29,32),(29,20),(26,18),(21,18),(19,21),(19,30),(17,32)], fill='#bdc6d1')
        d.line((19,16,26,16), fill='#edf0ed')
        d.line((17,22,17,29), fill='#76899b')
        d.line((31,21,31,29), fill='#e2e5e4')
    else:
        d.ellipse((25,12,32,18), fill=INK)
        d.ellipse((27,13,31,16), fill='#53616a')
        d.polygon(scalp, fill=INK)
        d.polygon([(17,19),(18,16),(21,15),(28,15),(31,18),(31,21),(28,21),(26,18),(21,19),(18,22)], fill='#303c45')
        d.line((20,16,26,16), fill='#667780')
    # Mastery changes tailoring and stitching, never widens the anatomy.
    if rank >= 1: d.point((21,38), fill=c)
    if rank >= 2: d.line((18,hem-2,21,hem-2), fill=c)
    if rank >= 3:
        d.rectangle((27,34,29,36), fill=INK); d.point((28,35), fill=c)
    if rank >= 4: d.line((26,40,28,hem-2), fill=c)
    if rank >= 5:
        d.line((17,40,18,40), fill=c); d.point((30,40), fill=c)
    if rank >= 6: d.line((19,32,21,33), fill='#e5c778')
    if rank == 7:
        for k,color in enumerate(COLORS): d.point((26+k,hem-2), fill=color)
        d.point((28,35), fill=COLORS[f//3 % 3])
    if prop:
        if science == 0:
            d.line((34,42,36,41), fill='#8b795b')
            d.ellipse((35,39,39,43), fill=INK)
            d.ellipse((36,40,38,42), fill='#e3c279')
            d.point((37,41), fill=c)
        elif science == 1:
            d.ellipse((35,39,40,44), fill=INK)
            d.point((35,40), fill=INK); d.point((36,39), fill=INK)
            d.rectangle((36,40,39,43), fill='#ce6151')
            d.point((36,40), fill='#f6ae73')
            d.line((37,39,38,37), fill='#8fb879')
        else:
            d.rectangle((35,39,39,44), fill=INK)
            d.rectangle((36,40,38,43), fill='#48ad93')
            d.rectangle((36,37,38,39), fill='#c9ddd7')
            d.point((37,41-f%2), fill='#e5fff0')
    return im


def transformation(source, target, rank, f):
    # Source -> target, never a cyclic intermediate. Feet remain in the base body.
    science = source if f < 4 else target
    im = costume(science, rank, f, hands=[0,1,2,2,2,2,1,0][f])
    d = ImageDraw.Draw(im); color = COLORS[target]
    if 1 <= f <= 6:
        # Small atom between the gathered hands, followed by a close body wrap.
        radius = [0,2,4,5,5,4,2,0][f]
        for tilt in (-1,1):
            pts = [(24+round(radius*math.cos(k*math.tau/16)),36+round(radius*.5*math.sin(k*math.tau/16)+tilt*radius*.35*math.cos(k*math.tau/16))) for k in range(17)]
            d.line(pts, fill=color)
        d.point((24,36), fill='#f8fff2')
        if 3 <= f <= 5:
            alpha = {3:170,4:220,5:110}[f]
            rgb = tuple(int(color[i:i+2],16) for i in (1,3,5))
            wrap = Image.new('RGBA', im.size); w = ImageDraw.Draw(wrap)
            w.line([(18,42),(15,33),(19,24),(17,16),(25,12),(32,19),(30,27),(35,35),(31,44)], fill=rgb+(alpha,), width=2)
            # Cover the face/coat at the instant the scientist changes.
            if f == 4: w.polygon([(19,18),(28,18),(31,28),(30,45),(18,45),(17,31)], fill=rgb+(135,))
            im.alpha_composite(wrap)
    return im


def previews(root):
    """Use shipped pixels, including base/outfit compositing, not concept art."""
    mod = root/'mods/tfm2_custom'; folder = root/'docs'
    def frame(sheet, tag, f=0):
        p = mod/sheet
        meta = json.loads(p.with_name(p.name+'#anim.fanim').read_text())
        rect = meta['anims'][tag]['frames'][f]['data']
        with Image.open(p.with_name(p.name+'#sheet.png')) as im:
            x,y,w,h = [int(rect[k]) for k in ('x','y','w','h')]
            return im.convert('RGBA').crop((x,y,x+w,y+h))
    base = frame('champions/tfm2_custom_unified_theory','idle')
    forms = []
    for science in range(3):
        im = base.copy(); im.alpha_composite(frame('vfx/science_outfits',f'outfit{science}_r0'))
        forms.append(im)
    refs=[]
    for name in ('swordman','taoist'):
        p=root/'Sprite kit/base champions'/name
        rect=json.loads(p.with_suffix('.anim.json').read_text())['anims']['idle']['frames'][0]['data']
        x,y,w,h=[int(rect[k]) for k in ('x','y','w','h')]
        with Image.open(p.with_suffix('.png')) as im:refs.append(im.convert('RGBA').crop((x,y,x+w,y+h)))
    labels=['SWORDSMAN','TAOIST','EINSTEIN','NEWTON','MARIE CURIE']
    sprites=refs+forms
    fontpath='/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf'
    def text(d,xy,value,size=13):d.text(xy,value,font=ImageFont.truetype(fontpath,size),fill='#dcecf4')
    scale=Image.new('RGBA',(800,510),'#20232c');d=ImageDraw.Draw(scale)
    text(d,(20,16),'UNIFIED THEORY / SHARED SCALE AND FOOT POSITION',19)
    text(d,(20,47),'Actual pixels (1x) above; nearest-neighbour enlargement (4x) below.')
    d.line((20,144,780,144),fill='#617484')
    for k,im in enumerate(sprites):
        box=im.getbbox();crop=im.crop(box);x=80+k*155
        scale.alpha_composite(crop,(x-crop.width//2,144-crop.height))
        scale.alpha_composite(crop.resize((crop.width*4,crop.height*4),Image.Resampling.NEAREST),(x-crop.width*2,423-crop.height*4))
        text(d,(x-55,157),labels[k],11)
        text(d,(x-48,442),f'{crop.width} x {crop.height} px',12)
    scale.convert('RGB').save(folder/'unified-theory-scale.png')
    sil=Image.new('RGBA',(600,370),'#dce7e5');d=ImageDraw.Draw(sil)
    d.text((18,15),'SILHOUETTES / ACTUAL PIXELS + 4x',font=ImageFont.truetype(fontpath,16),fill=INK)
    for s,im in enumerate(forms):
        solid=Image.new('RGBA',im.size,INK);solid.putalpha(im.getchannel('A'))
        x=100+s*200
        sil.alpha_composite(solid,(x-24,50))
        sil.alpha_composite(solid.resize((192,256),Image.Resampling.NEAREST),(x-96,100))
    sil.convert('RGB').save(folder/'unified-theory-silhouettes.png')
    rows=[(s,t) for s in range(3) for t in range(3) if s!=t]
    strip=Image.new('RGBA',(1000,710),'#20232c');d=ImageDraw.Draw(strip)
    text(d,(18,14),'DIRECT FORM CHANGES / EIGHT FRAMES / 0.4 SECONDS',19)
    text(d,(18,43),'Hands gather > atom forms > light wraps > chosen form settles. Feet stay planted.')
    animation=[]
    for f in range(8):
        scene=Image.new('RGBA',(800,390),'#20232c');sd=ImageDraw.Draw(scene)
        text(sd,(20,12),'ANY FORM > EITHER OTHER FORM',19)
        for k,(source,target) in enumerate(rows):
            im=base.copy();im.alpha_composite(frame('vfx/science_transforms',f'transform{source}_{target}_r0',f))
            x=k%3*260+130;y=k//3*170+72
            scene.alpha_composite(im.resize((96,128),Image.Resampling.NEAREST),(x-48,y))
            text(sd,(x-106,y-20),labels[source+2]+' > '+labels[target+2],11)
            strip.alpha_composite(im.resize((96,128),Image.Resampling.NEAREST),(182+f*100,76+k*102))
        animation.append(scene.convert('RGB'))
    for k,(source,target) in enumerate(rows):
        text(d,(12,108+k*102),labels[source+2],12)
        text(d,(12,125+k*102),'> '+labels[target+2],12)
    strip.convert('RGB').save(folder/'unified-theory-transform-frames.png')
    # Hold the final pose for review; the eight active frames total 400 ms.
    animation[0].save(folder/'unified-theory-transforms.gif',save_all=True,append_images=animation[1:],duration=[50]*7+[850],loop=0,disposal=2)
