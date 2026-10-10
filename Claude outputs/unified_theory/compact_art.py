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
    im = Image.new('RGBA', (48,64)); d = ImageDraw.Draw(im)
    if kind == 'dead':
        d.polygon([(14,53),(18,50),(29,50),(34,54),(31,58),(15,58)],fill=INK)
        d.rectangle((17,51,27,54),fill='#e4e4d7');d.rectangle((29,52,32,55),fill=SKIN)
        return im
    step=(1 if f%4<2 else -1) if kind=='run' else 0
    d.rectangle((20,46,23,55+step),fill=INK);d.rectangle((26,46,29,55-step),fill=INK)
    d.line((21,48,21,54+step),fill='#485061');d.line((27,48,27,54-step),fill='#293442')
    # Toe caps point into the battlefield, matching the head and coat front.
    d.rectangle((20,55+step,25,58+step),fill=INK);d.rectangle((26,55-step,31,58-step),fill=INK)
    d.line((21,55+step,24,55+step),fill='#81878d');d.line((27,55-step,30,55-step),fill='#536274')
    im.alpha_composite(costume(0 if persona is None else persona,0,f))
    return im


def costume(science, rank=0, f=0, hands=0):
    im=Image.new('RGBA',(48,64));d=ImageDraw.Draw(im);c=COLORS[science]
    coat=['#e7e4d0','#262d42','#d1ded4'][science]
    shade=['#829ead','#141e32','#708e8b'][science]
    light=['#fff3de','#9098b3','#f0f6e5'][science]
    hem=50 if science==1 else 47
    # Rear shoulder to the left, chest/lapels to the right: a real side-facing
    # construction rather than two symmetrical halves of a portrait.
    d.rectangle((24,32,27,36),fill=SKIN)
    outline=[(21,34),(27,34),(30,38),(30,hem),(26,hem),(24,44),(21,hem),(17,hem),(18,38)]
    d.polygon(outline,fill=INK)
    d.polygon([(21,35),(24,36),(24,43),(21,hem-1),(18,hem-1),(19,38)],fill=shade)
    d.polygon([(25,35),(28,37),(29,hem-1),(26,hem-1),(25,44)],fill=coat)
    d.line((25,37,26,44),fill='#1b2b44',width=2)
    d.line((26,36,28,38,27,41),fill=light)
    for y in (40,43):d.point((28,y),fill='#e5cc94' if science==1 else light)
    # A low far arm; the near forearm is bent forward around the instrument.
    d.polygon([(19,36),(17,39),(17,44),(20,46),(22,43),(21,39)],fill=INK)
    d.line((19,38,19,43),fill=coat,width=2);d.rectangle((19,44,21,45),fill=SKIN)
    d.polygon([(27,36),(30,38),(31,42),(33,43),(32,45),(28,44),(26,39)],fill=INK)
    d.line((28,38,29,42,31,43),fill=coat,width=2)
    d.rectangle((31,42,33,44),fill=SKIN)
    if hands:
        d.rectangle((19,43,21,45),fill=shade)
        d.rectangle((31,42,33,44),fill=coat)
        y=43-min(hands,2)*2
        d.line((20,42,25,y),fill=shade,width=3);d.line((30,41,28,y),fill=coat,width=3)
        d.rectangle((25,y-1,26,y+1),fill=SKIN);d.rectangle((28,y-1,29,y+1),fill=SKIN)
    # One visible eye, an extended nose and a receding jaw face right. Hair
    # occupies the rear of the head, while the face stays open at the front.
    face=[(23,24),(28,24),(29,27),(30,28),(30,30),(29,30),(29,32),(26,34),(23,32),(22,28)]
    d.polygon(face,fill=INK)
    d.polygon([(24,25),(27,25),(28,28),(29,28),(29,29),(28,30),(28,32),(26,33),(24,31),(23,28)],fill=SKIN)
    d.line((24,28,24,31),fill='#a97f65');d.point((27,27),fill=INK)
    d.point((29,31),fill='#947158')
    scalp=[(15,26),(16,23),(18,21),(22,20),(27,21),(29,23),(29,26),(26,26),(25,24),(22,25),(22,29),(18,31),(16,29)]
    if science==0:
        d.polygon(scalp,fill=INK)
        # Windswept white tufts behind the ear and a projecting moustache.
        d.polygon([(16,25),(17,23),(19,24),(19,22),(22,23),(22,21),(25,23),(27,22),(28,24),(26,25),(23,24),(21,26),(21,29),(18,30),(17,28),(19,27)],fill='#f3f2e9')
        d.line((18,25,20,25),fill='#b8c7d1');d.point((22,28),fill='#c7d0cd')
        d.line((27,30,30,30),fill='#fcf8e9');d.point((27,31),fill='#dddcd0')
        # Relativity scarf and bent blue space-seam in an otherwise worn coat.
        d.polygon([(22,35),(24,35),(25,38),(23,39),(21,37)],fill='#4f90b9')
        d.line((22,38,20,42,21,45),fill='#376080')
        if rank>=3:
            d.polygon([(19,40),(22,39),(23,43),(20,46),(18,46)],fill='#193756')
            d.line((19,40,22,41,20,44),fill=c)
    elif science==1:
        # Curled silver hair and the white cravat of a seventeenth-century scholar.
        d.polygon(scalp,fill=INK)
        d.polygon([(16,25),(18,22),(23,21),(27,22),(28,24),(25,25),(24,23),(21,25),(21,31),(23,33),(22,37),(19,38),(17,36),(15,33)],fill='#b5bfce')
        d.line((18,23,23,22,26,23),fill='#f0eee9')
        d.line((17,27,16,31,18,33,17,35,20,36),fill='#71869a')
        d.line((20,27,20,31,22,34,21,36),fill='#e7e8e4')
        d.polygon([(25,35),(28,35),(27,37),(28,39),(26,40),(25,37)],fill='#ede7d4')
        d.line((27,41,27,45),fill='#b89554')
        d.rectangle((18,43,21,46),fill='#583f2d');d.line((18,43,21,43),fill='#ecd7a0')
        if rank>=3:
            d.line((18,38,20,40,18,43),fill='#c99e60')
            d.line((28,40,30,42),fill=c)
    else:
        # The bun is behind the head, with a smooth forward hairline and cheek.
        d.ellipse((15,19,21,25),fill=INK);d.ellipse((16,20,20,23),fill='#44525d')
        d.polygon(scalp,fill=INK)
        d.polygon([(16,25),(18,22),(22,21),(27,22),(28,24),(25,25),(23,24),(21,27),(21,29),(18,30),(17,28)],fill='#2a3544')
        d.line((18,24,21,23,25,23),fill='#697989')
        d.line((25,32,27,32),fill='#bb796b')
        d.line((24,36,24,44),fill='#244f51',width=2)
        d.line((27,37,29,39),fill='#f6f6e6')
        d.rectangle((18,42,20,44),fill='#32615c')
        if rank>=3:
            d.polygon([(19,43),(22,41),(23,44),(21,46),(18,46)],fill='#17484f')
            d.line((19,43,22,42,21,45),fill=c)
    # Clear the forward forehead so the single visible eye reads apart from hair.
    d.point((27,25),fill=SKIN);d.point((27,26),fill=SKIN);d.point((27,27),fill=INK)
    # Baked, sparse cosmic motion lives inside the costume, growing with mastery.
    if rank>=1:d.point((26,38),fill=c)
    if rank>=2:d.line((18,hem-2,20,hem-2),fill=c)
    if rank>=4:
        for k in range(3):
            d.point((19+(k+f//2)%3,41+k),fill=['#a9dcff','#ffe1a1','#afffdb'][science])
    if rank>=5:
        if science==0:
            d.polygon([(20,37),(23,39),(20,44),(18,46),(19,42)],fill='#0e213e')
            d.line((20,37,22,39,20,43,18,46),fill=c)
            d.point((20,40+f%3),fill='#e4f6ff')
        elif science==1:
            d.polygon([(21,39),(24,42),(21,45),(18,42)],fill='#151e36',outline='#b18a48')
            angle=f*math.tau/8
            d.point((21+round(math.cos(angle)*2),42+round(math.sin(angle)*2)),fill='#ffeab5')
            d.point((21,42),fill=c)
        else:
            d.polygon([(19,39),(22,41),(23,44),(20,47),(18,44)],fill='#104553',outline=c)
            d.line((19,40,20,45,22,43),fill='#91cdd3')
            d.point((20,41+f%4),fill='#d2ffee')
        d.line((18,39,19,41),fill=c);d.line((28,44,28,hem-2),fill=c)
    if rank>=6:
        d.point((21,36),fill='#f8d994');d.point((26,hem-2),fill=c)
    if rank==7:
        for k,color in enumerate(COLORS):d.point((18+k,hem-1),fill=color)
        d.line((21,44,23,46),fill=COLORS[(f//2)%3])
    # Pocket singularity, planetary apple and contained radioactive crystal.
    # Shared opaque footprints prevent remnants of the base prop on a swap.
    d.ellipse((33,40,37,44),fill=INK)
    if science==0:
        d.line((32,43,34,42),fill='#8e7955')
        d.ellipse((34,41,36,43),fill='#233b61')
        dx,dy=[(0,-1),(1,-1),(1,0),(1,1),(0,1),(-1,1),(-1,0),(-1,-1)][f%8]
        d.point((35+dx,42+dy),fill=c);d.point((35,42),fill='#0c152c')
        d.point((35-dx,42-dy),fill='#e0d19b')
    elif science==1:
        d.rectangle((34,41,36,43),fill='#cf5747');d.point((34,41),fill='#ffc185')
        d.line((35,40,36,38),fill='#87b47c')
        if rank>=2:
            angle=f*math.tau/8
            d.point((35+round(math.cos(angle)*3),42+round(math.sin(angle)*3)),fill='#ffcf82')
            d.point((35,44),fill='#e1a153')
    else:
        d.rectangle((33,39,37,45),fill=INK);d.rectangle((34,40,36,44),fill='#245f65')
        d.rectangle((34,38,36,39),fill='#d4ded7')
        d.line((35,40,36,42,35,44,34,42,35,40),fill=c)
        d.point((35,41+f%3),fill='#d5ffee')
    return im


def transformation(source,target,rank,f):
    science=source if f<4 else target
    im=costume(science,rank,f,hands=[0,1,2,2,2,2,1,0][f])
    d=ImageDraw.Draw(im);color=COLORS[target]
    if 1<=f<=6:
        radius=[0,2,4,5,5,4,2,0][f]
        for tilt in (-1,1):
            pts=[(27+round(radius*math.cos(k*math.tau/16)),39+round(radius*.5*math.sin(k*math.tau/16)+tilt*radius*.35*math.cos(k*math.tau/16))) for k in range(17)]
            d.line(pts,fill=color)
        d.point((27,39),fill='#f8fff2')
        if 3<=f<=5:
            alpha={3:170,4:220,5:110}[f];rgb=tuple(int(color[i:i+2],16) for i in (1,3,5))
            wrap=Image.new('RGBA',im.size);w=ImageDraw.Draw(wrap)
            w.line([(18,46),(16,36),(18,30),(16,23),(24,20),(30,27),(31,35),(34,42),(29,47)],fill=rgb+(alpha,),width=2)
            if f==4:w.polygon([(22,24),(28,24),(30,33),(29,47),(19,47),(18,34)],fill=rgb+(135,))
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
    facing=Image.new('RGBA',(960,590),'#20232c');d=ImageDraw.Draw(facing)
    text(d,(18,15),'BATTLEFIELD-FACING SCIENTISTS / COMPACT COSMIC FORMS',20)
    text(d,(18,44),'Student above; Unified Mind below. Mirrored poses face the opposing team.',13)
    for science in range(3):
        x=science*320
        for row,rank in enumerate([0,7]):
            im=base.copy();im.alpha_composite(frame('vfx/science_outfits',f'outfit{science}_r{rank}',2))
            for side in range(2):
                sprite=im if side==0 else im.transpose(Image.Transpose.FLIP_LEFT_RIGHT)
                facing.alpha_composite(sprite.resize((144,192),Image.Resampling.NEAREST),(x+8+side*155,80+row*238))
            if rank==7:
                emblem=frame('vfx/science_badges','top1',2);emblem=emblem.crop(emblem.getbbox())
                facing.alpha_composite(emblem.resize((emblem.width*3,emblem.height*3),Image.Resampling.NEAREST),(x+135,337))
        text(d,(x+35,284),labels[science+2],14)
        text(d,(x+70,547),'UNIFIED MIND / #1',12)
    facing.convert('RGB').save(folder/'unified-theory-facing.png')

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
