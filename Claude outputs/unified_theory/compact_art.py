"""Compact, shared-anatomy scientist forms and a planted eight-cel morph.

Coordinates are native pixels on the existing 48 x 64 anchor. Costume layers
cover the common coat/head, leaving the host's moving legs visible.
"""
import math
import json
from PIL import Image, ImageDraw, ImageFont

INK = '#12131a'
COLORS = ['#63bfff', '#ffc96b', '#63e1bd']
TRANSFORM_COLORS = ['#b5ddff', '#ffd18a', '#b6f7d2']
SKIN = '#e5b58d'

# Eight material treatments, not an increasing silhouette. Ivory cuffs, the
# historical red coat and familiar hair still identify each scientist at 1x.
COATS = [
    ['#e1d8bd','#c6cadb','#aebbd2','#849bbb','#657caa','#475b8c','#32426d','#283653'],
    ['#8c3e48','#944858','#833749','#763749','#693247','#572d42','#48293b','#38283b'],
    ['#e8e4cc','#d1e3d6','#b4d5c8','#92bcaf','#6b9e98','#4c7e81','#39666f','#29515e'],
]
SHADES = [
    ['#9196a0','#7b859e','#677892','#465b80','#35456c','#283556','#202944','#18213b'],
    ['#432631','#482b3c','#392536','#332333','#2b2230','#25202e','#201e2b','#181d2d'],
    ['#97a69d','#809e94','#678d84','#497b74','#386263','#2a4b53','#213f4a','#192f40'],
]
LIGHTS = ['#fff0d1','#f0ba88','#fff4dc']


def cosmic_cloth(im, science, rank, f, hem):
    """Opaque cosmic fabric clipped to the coat; no new idle VFX or limbs."""
    mask=Image.new('L',im.size);m=ImageDraw.Draw(mask)
    # The seed lives in the lining. Later ranks turn both coat tails into
    # folded space, an astronomical machine or a contained crystal nebula.
    if rank<2:m.polygon([(19,42),(22,41),(23,45),(21,hem-1),(18,hem-1)],fill=255)
    else:
        m.polygon([(19,39),(22,40),(23,44),(21,hem-1),(18,hem-1),(18,41)],fill=255)
        m.polygon([(26,39),(28,38),(29,41),(28,hem-1),(25,hem-1),(25,43)],fill=255)
    layer=Image.new('RGBA',im.size);d=ImageDraw.Draw(layer)
    dark=['#182441','#352139','#183a45'][science]
    d.rectangle((17,38,29,hem),fill=dark)
    for k in range(3+rank):
        x=18+(k*7+science*2)%11;y=40+(k*3+science)%(hem-39)
        d.point((x,y),fill=['#536aab','#87627c','#488880'][science])
    # Eight discrete travelling highlights fit within the same opaque material.
    route=[(19,43),(20,42),(21,42),(22,43),(22,44),(21,45),(20,45),(19,44)]
    if science==0:
        d.line([(19,41),(22,42),(20,44),(18,46)],fill='#537ac1')
        if rank>=2:
            d.polygon([(26,40),(28,42),(26,45),(25,44)],fill='#080f28')
            d.line((26,40,28,42,26,45),fill=COLORS[0])
        if rank>=4:d.line((18,45,21,44,23,45),fill='#b6deff')
    elif science==1:
        d.polygon([(20,42),(23,44),(20,47),(18,44)],outline='#bc8959')
        d.point((20,44),fill='#ffe9ac')
        if rank>=2:d.line((26,41,28,44,26,48,25,45,26,41),fill='#d8a66d')
        if rank>=4:d.line((18,48,21,49,23,47),fill='#f1c77f')
    else:
        d.polygon([(20,41),(22,43),(21,48),(18,46)],fill='#225d63',outline='#73c9b8')
        d.line((20,42,20,46,21,48),fill='#b8f5d2')
        if rank>=2:d.line((26,41,28,44,26,48,26,41),fill='#68d8c2')
        if rank>=4:d.line((18,47,21,49,23,47),fill='#c0ffe3')
    if rank>=3:
        # The illuminated measurement seam is part of the physical tailoring.
        d.line((18,40,18,hem-1),fill=COLORS[science])
    if rank>=5:
        d.line((28,40,28,hem-2,26,hem-1),fill=LIGHTS[science])
    if rank>=6:
        d.point((21,40),fill='#ffeaa4');d.point((26,42),fill='#f0f8ff')
    if rank==7:
        # Three connected physical panels, with an empty luminous centre.
        for k,color in enumerate(COLORS):d.line((18+k*4,hem-3,19+k*4,hem-1),fill=color)
        d.line((19,43,21,42,23,44,21,46,19,43),fill='#ebf8ff')
        d.point((21,44),fill='#070e20')
    d.point(route[f],fill=['#d4edff','#fff2c1','#d6ffeb'][science])
    # A hard mask prevents halos, coat panels or stars escaping the body.
    layer.putalpha(mask);im.alpha_composite(layer)


def apple(d,cx,cy,rank=0,f=0):
    dx,dy=cx-12,cy-41
    offset=lambda pts:[(x+dx,y+dy) for x,y in pts]
    d.polygon(offset([(11,39),(12,39),(13,40),(14,39),(15,40),(15,42),(14,44),(11,44),(10,42),(10,40)]),fill=INK)
    d.polygon(offset([(11,40),(12,40),(13,41),(14,40),(14,42),(13,43),(11,43)]),fill='#c84b44')
    d.point((11+dx,41+dy),fill='#f98766')
    d.line((12+dx,39+dy,13+dx,37+dy),fill='#697941');d.point((14+dx,37+dy),fill='#9cab62')
    if rank>=2:
        angle=f*math.tau/8
        d.point((cx+round(math.cos(angle)*2),cy+round(math.sin(angle)*2)),fill='#ffcf82')


def vial(d,cx,cy,rank=0,f=0):
    d.rectangle((cx-1,cy-2,cx+2,cy+3),fill=INK)
    d.rectangle((cx,cy-1,cx+1,cy+2),fill='#4faa91')
    d.rectangle((cx-1,cy-3,cx+2,cy-2),fill='#e1ddc9')
    d.line((cx,cy,cx,cy+2),fill='#acffd4')
    d.point((cx+1,cy-1+f%3),fill=['#dcfff0','#affde7','#b4eada'][f//3])


def foundation(kind='idle', f=0):
    im = Image.new('RGBA', (48,64)); d = ImageDraw.Draw(im)
    if kind == 'dead':
        d.polygon([(14,53),(18,50),(29,50),(34,54),(31,58),(15,58)],fill=INK)
        d.rectangle((17,51,27,54),fill='#e1d8bd');d.rectangle((29,52,32,55),fill=SKIN)
        return im
    step=(1 if f%4<2 else -1) if kind=='run' else 0
    # Short trousers, separated knees and one foot closer to the viewer.
    # Shared moving feet stay below the scientist-specific coat/skirt hems.
    d.polygon([(26,46),(29,47),(30,50),(29,54-step),(26,54-step),(25,50)],fill=INK)
    d.polygon([(27,47),(28,48),(29,50),(28,53-step),(26,52)],fill='#343842')
    d.polygon([(26,53-step),(29,53-step),(29,55-step),(31,55-step),(31,57-step),(26,57-step)],fill=INK)
    d.line((27,55-step,30,55-step),fill='#73777b')
    d.polygon([(19,46),(25,46),(24,50),(23,53+step),(23,56+step),(19,56+step),(18,52)],fill=INK)
    d.polygon([(20,47),(24,47),(23,50),(21,53+step),(22,55+step),(20,55+step),(19,52)],fill='#515868')
    d.polygon([(20,54+step),(23,54+step),(23,56+step),(25,56+step),(25,58+step),(19,58+step),(19,56+step)],fill=INK)
    d.line((20,56+step,24,56+step),fill='#949592')
    return im


def body(kind='idle', f=0, persona=None):
    im=foundation(kind,f)
    if kind!='dead':im.alpha_composite(costume(0 if persona is None else persona,0,f))
    return im


def costume(science, rank=0, f=0, hands=0):
    im=Image.new('RGBA',(48,64));d=ImageDraw.Draw(im);c=COLORS[science]
    coat=COATS[science][rank]
    shade=SHADES[science][rank]
    light=LIGHTS[science]
    hem=[47,51,50][science]
    # The rear shoulder recedes. Its sleeve is a separate dark shape, and a
    # warm hand clears the cuff rather than merging into an ivory coat.
    d.polygon([(27,35),(30,36),(32,40),(30,44),(28,45),(27,40)],fill=INK)
    d.polygon([(29,37),(30,39),(31,40),(29,43)],fill=shade)
    d.line((29,43,31,43),fill=light)
    d.rectangle((29,44,31,46),fill=SKIN);d.point((31,46),fill='#b47f64')
    if science==2:
        d.polygon([(19,46),(28,46),(30,54),(17,54),(18,50)],fill=INK)
        d.polygon([(20,47),(27,47),(29,53),(18,53)],fill=['#293b3b','#293f43','#284950','#244650','#223b4e','#23374c','#263349','#282e49'][rank])
        d.line((21,48,20,52),fill='#596c65');d.line((26,48,28,52),fill='#192629')
        if rank>=3:
            d.line((19,52,21,50,23,53,25,50,28,52),fill='#4d968d')
            d.point((20+f%8,52),fill='#8bd9bb')
    d.rectangle((22,32,26,36),fill=SKIN)
    d.polygon([(18,34),(25,34),(29,36),(30,40),(29,hem),(25,hem),(24,45),(22,hem),(17,hem),(17,38)],fill=INK)
    d.polygon([(19,35),(23,35),(24,39),(23,44),(21,hem-1),(18,hem-1),(18,38)],fill=coat)
    d.polygon([(26,36),(28,37),(29,41),(28,hem-1),(25,hem-1),(25,42)],fill=shade)
    d.rectangle((23,37,25,44),fill=['#506275','#292b37','#294843'][science])
    d.line((19,36,23,39,22,43),fill=light)
    d.line((26,36,25,39),fill=light)
    d.line((18,44,20,43,19,hem-1),fill=shade)
    for y in (40,43):d.point((25,y),fill='#f1ce8d' if science==1 else light)
    if science==0:
        d.polygon([(22,35),(24,35),(24,38),(22,39),(21,37)],fill='#80a6bc')
        d.line((23,40,23,44),fill='#7596ab')
        # The watch is clipped inside the coat; Einstein's raised hand is free.
        d.ellipse((25,43,28,46),fill=INK)
        d.ellipse((26,44,27,45),fill='#8cafc1')
        d.point((26+f%2,44),fill=c)
    elif science==1:
        d.polygon([(23,35),(26,35),(25,37),(26,39),(24,40),(23,37)],fill='#f6e6c5')
        d.line((24,36,24,38),fill='#bea987')
        d.line((26,46,27,49),fill='#b76b65')
    else:
        d.line((24,36,24,44),fill='#486c62')
        d.rectangle((27,40,28,42),fill='#74958a')
        d.point((27,40),fill='#f6f1d9')
    cosmic_cloth(im,science,rank,f,hem)
    if rank>=1:d.point((26,38),fill=c)
    if rank>=2:d.point((21,36),fill=c)
    if rank>=5:d.point((19,36),fill='#f3d99d')
    # The near shoulder sits higher and comes forward. Each scientist has a
    # different forearm pose, with an explicit cuff and exposed warm fingers.
    if hands:
        # Keep the planted elbow footprint opaque over the default body, then
        # bend both forearms inward. Lower sleeve pixels cover the old hand.
        d.rectangle((29,44,31,46),fill=shade)
        d.polygon([(18,35),(20,36),(19,39),(21,37),(23,37),(23,40),(18,42),(15,41),(15,39),(16,37)],fill=INK)
        d.line((17,37,17,40),fill=coat,width=2)
        d.rectangle((21,37,23,39),fill=coat)
        near_x,near_y=([(22,40),(18,41),(25,40)][science] if hands==1 else (22,39))
        far_x,far_y=(28,41) if hands==1 else (26,39)
        d.line((17,40,near_x,near_y),fill=coat,width=3)
        d.line((31,42,far_x,far_y),fill=shade,width=3)
        d.line((near_x-1,near_y,near_x+1,near_y),fill=light)
        d.rectangle((near_x,near_y-1,near_x+1,near_y+1),fill=SKIN)
        d.rectangle((far_x,far_y-1,far_x+1,far_y+1),fill=SKIN)
        if science==1:apple(d,near_x-2,near_y-2,rank,f)
        elif science==2:vial(d,near_x+3,near_y-1,rank,f)
    elif science==0:
        d.polygon([(18,35),(20,36),(19,39),(21,37),(23,37),(23,40),(18,42),(15,41),(15,39),(16,37)],fill=INK)
        d.line((17,37,17,40,19,39),fill=coat,width=2)
        d.line((18,39,20,39),fill=light)
        d.rectangle((21,37,23,39),fill=SKIN);d.point((21,39),fill='#b47f64')
    elif science==1:
        d.polygon([(18,35),(20,37),(19,40),(17,42),(17,44),(14,44),(14,41),(16,37)],fill=INK)
        d.line((18,37,17,40,15,42),fill=coat,width=2)
        d.line((15,42,17,42),fill=light)
        d.rectangle((14,43,16,44),fill=SKIN)
        apple(d,12,41,rank,f)
    else:
        # Curie's near forearm crosses the coat and brings the sample ahead
        # along her gaze; it stays well below her face.
        d.polygon([(18,35),(20,36),(19,39),(22,41),(27,39),(29,39),(29,43),(21,45),(17,43),(15,41),(15,39),(16,37)],fill=INK)
        d.line((17,37,17,40,21,43,27,41),fill=coat,width=2)
        d.line((25,42,27,41),fill=light)
        d.rectangle((27,40,29,42),fill=SKIN)
        vial(d,31,39,rank,f)
    # A broad cheek and a receding far cheek turn the head gently right.
    # Soft hair frames the jaw; the small nose stays inside the face outline.
    if science==0:
        hair=[(18,27),(17,25),(19,24),(18,23),(21,23),(22,22),(24,23),(27,22),(29,24),(30,25),(29,28),(27,27),(25,27),(23,26),(21,28),(19,29)]
        d.polygon(hair,fill=INK)
        d.polygon([(18,25),(20,25),(19,24),(22,24),(22,23),(24,24),(27,23),(28,25),(29,25),(28,27),(25,26),(23,25),(21,27),(19,28)],fill='#f1eee0')
        d.line((19,26,21,25),fill='#aeb7bd');d.line((25,24,27,25),fill='#c5ced0')
    elif science==1:
        d.polygon([(17,25),(18,23),(22,22),(27,22),(30,24),(30,30),(32,34),(30,37),(27,37),(28,31),(28,26),(25,25),(21,27),(21,32),(23,36),(20,38),(16,36),(16,29)],fill=INK)
        d.polygon([(18,25),(19,24),(22,23),(27,23),(29,25),(29,30),(31,34),(29,36),(28,35),(29,32),(27,27),(25,26),(21,28),(20,31),(21,34),(22,36),(19,37),(17,35),(17,30)],fill='#bdc2cd')
        d.line((19,25,23,24,27,24),fill='#f4ecdf')
        d.line((18,28,17,31,19,33,18,35,20,36),fill='#737d94')
        d.line((29,28,28,31,30,33,29,35),fill='#ece9df')
    else:
        d.ellipse((16,22,21,28),fill=INK);d.ellipse((17,23,20,26),fill='#566064')
        d.polygon([(19,24),(19,22),(24,21),(28,22),(30,24),(30,28),(29,30),(28,27),(25,26),(23,25),(21,27),(19,31),(17,28),(17,25)],fill=INK)
        d.polygon([(20,24),(20,23),(24,22),(27,23),(29,25),(29,28),(28,27),(25,25),(23,24),(21,26),(20,29),(18,28),(18,26)],fill='#303f43')
        d.line((21,24,24,23,27,24),fill='#738180');d.point((18,24),fill='#94a49b')
    face=[(22,26),(24,25),(28,26),(30,28),(30,31),(28,33),(25,34),(22,33),(20,31),(20,28)]
    d.polygon(face,fill=INK)
    d.polygon([(22,27),(24,26),(27,27),(29,28),(29,31),(27,32),(25,33),(23,32),(21,30),(21,28)],fill=SKIN)
    d.line((21,29,22,31,24,32),fill='#c28c6d')
    d.point((26,30),fill='#ffdbad');d.point((28,30),fill='#b98569');d.point((28,29),fill='#ffdbad')
    d.rectangle((23,28,24,29),fill='#fff0d2')
    d.point((24,28),fill=INK);d.point((24,29),fill='#604232')
    d.point((28,28),fill='#604232')
    d.point((24,27),fill='#817063');d.point((28,27),fill='#a48770')
    if science==0:
        d.line((25,31,28,31),fill='#f5eee0');d.point((26,32),fill='#c8c9bc')
    else:d.line((26,32,27,32),fill='#a56d63' if science==2 else '#986f62')
    return im


def transformation(source,target,rank,f):
    science=source if f<4 else target
    im=costume(science,rank,f,hands=[0,1,2,2,2,2,1,0][f])
    d=ImageDraw.Draw(im);color=TRANSFORM_COLORS[target]
    if 1<=f<=6:
        radius=[0,2,4,5,5,4,2,0][f]
        for tilt in (-1,1):
            pts=[(24+round(radius*math.cos(k*math.tau/16)),39+round(radius*.5*math.sin(k*math.tau/16)+tilt*radius*.35*math.cos(k*math.tau/16))) for k in range(17)]
            d.line(pts,fill=color)
        d.point((24,39),fill='#f8fff2')
        if 3<=f<=5:
            alpha={3:170,4:220,5:110}[f];rgb=tuple(int(color[i:i+2],16) for i in (1,3,5))
            wrap=Image.new('RGBA',im.size);w=ImageDraw.Draw(wrap)
            w.line([(18,51),(16,43),(17,35),(19,29),(17,24),(24,21),(30,24),(30,33),(33,42),(30,51),(25,53)],fill=rgb+(alpha,),width=2)
            if f==4:w.polygon([(20,25),(28,25),(30,34),(30,51),(19,53),(17,35)],fill=rgb+(135,))
            im.alpha_composite(wrap)
    return im


def static_previews(root, sprites):
    """Inspect completed poses on actual-size cards before drawing the morph."""
    refs=[]
    for name in ('swordman','taoist'):
        p=root/'Sprite kit/base champions'/name
        rect=json.loads(p.with_suffix('.anim.json').read_text())['anims']['idle']['frames'][0]['data']
        x,y,w,h=[int(rect[k]) for k in ('x','y','w','h')]
        with Image.open(p.with_suffix('.png')) as im:refs.append(im.convert('RGBA').crop((x,y,x+w,y+h)))
    fontpath='/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf'
    font=lambda size:ImageFont.truetype(fontpath,size)
    canvas=Image.new('RGBA',(800,490),'#20232c');d=ImageDraw.Draw(canvas)
    d.text((18,18),'STATIC SCIENTISTS / DIRECT CHAMPION REFERENCES',font=font(18),fill='#e3e7ed')
    d.text((18,48),'Actual-size champion cards above; nearest-neighbour closeups below.',font=font(12),fill='#aebccd')
    labels=['SWORDSMAN','TAOIST','EINSTEIN','NEWTON','MARIE CURIE']
    for k,im in enumerate(refs+sprites):
        x=10+k*156
        d.rounded_rectangle((x,78,x+145,224),radius=7,fill='#252932',outline='#616977')
        d.text((x+9,86),['Melee','Support','Mage','Mage','Mage'][k],font=font(12),fill='#e0e1e8')
        crop=im.crop(im.getbbox());canvas.alpha_composite(crop,(x+72-crop.width//2,184-crop.height))
        d.text((x+72,204),labels[k],anchor='mm',font=font(12),fill='#e0e1e8')
        canvas.alpha_composite(crop.resize((crop.width*4,crop.height*4),Image.Resampling.NEAREST),(x+72-crop.width*2,433-crop.height*4))
        d.text((x+72,463),f'{crop.width} x {crop.height} px',anchor='mm',font=font(12),fill='#aebccd')
    canvas.convert('RGB').save(root/'docs/unified-theory-static-cards.png')
    for s,name in enumerate(('einstein','newton','curie')):
        sprites[s].save(root/f'docs/unified-theory-static-{name}.png')


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
    static_previews(root,forms)
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
    text(d,(18,15),'FORWARD-GAZING SCIENTISTS / THREE-QUARTER STANCE',20)
    text(d,(18,44),'Student above; Unified Mind below. Distinct hands, outfits and forward steps.',13)
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
