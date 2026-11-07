"""Deterministic pose rig and packing from the approved, frozen imagegen pixels.

Writes review assets only. It never installs art or touches careers. The imagegen
source is not deterministic; its frozen hash is the input to this reproducible step.
"""
from pathlib import Path
from PIL import Image, ImageDraw, ImageOps
import hashlib, json, math, shutil

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / 'docs/grimhald-art-review'
SRC = OUT / 'proposal-v2-right.png'
# Same frozen source and approved scale, with horizontal transparent padding.
source_body=Image.open(OUT/'source-imagegen-v2.png').convert('RGBA')
crop=source_body.crop((source_body.width//2,0,source_body.width,source_body.height));box=crop.getbbox();crop=crop.crop(box)
scale=39/(box[3]-source_body.height*.225)
small=crop.resize((round(crop.width*scale),round(crop.height*scale)),Image.Resampling.NEAREST)
foot=crop.getchannel('A').crop((0,crop.height-16,crop.width,crop.height)).getbbox()
foot_x=(foot[0]+foot[2])/2*scale
body=Image.new('RGBA',(80,64));body.alpha_composite(small,(round(40-foot_x),52-small.getbbox()[3]))
assert hashlib.sha256((OUT/'source-imagegen-v2.png').read_bytes()).hexdigest() == 'b1e231c6722601ef54c30b0ac25c35c39d555cfae026f9b271f51b8e05253b91'
fx_source = OUT/'source-effects-v1.png'
if not fx_source.exists():
    shutil.copyfile(Path('C:/Users/arian/.codex/generated_images/01a13ba6-e432-77d3-b82f-597615070100/exec-61a29dde-0b44-41dc-9a1c-1ca9c26dce9f.png'), fx_source)

def blank(w=160,h=112): return Image.new('RGBA',(w,h))
def paste(im, part, xy): im.alpha_composite(part, tuple(map(round,xy)))

# Cut the approved weapon and its supporting gauntlet as one rigid assembly.
# This preserves the corrected hand-on-haft connection throughout the swing.
weapon=Image.new('RGBA',body.size); torso=body.copy()
for y in range(64):
    for x in range(80):
        r,g,b,a=body.getpixel((x,y))
        wood = 43<=x<=78 and abs(y-(16+(x-44)*7/29))<=3 and r>g*1.15 and g>b*1.08 and a>0
        blade=(x<=50 and y<14) or (x<=44 and y<24)
        hand=69<=x<=76 and 21<=y<28
        if blade or hand or wood:
            weapon.putpixel((x,y),body.getpixel((x,y)));torso.putpixel((x,y),(0,0,0,0))

# The helmet occluded the middle of the haft in the idle source. Complete that
# hidden wooden section beneath the frozen weapon pixels before rotating it.
haft=Image.new('RGBA',body.size);hd=ImageDraw.Draw(haft)
hd.line([(44,16),(73,23)],fill='#302322',width=3)
hd.line([(44,15),(73,22)],fill='#604033',width=1)
haft.alpha_composite(weapon);weapon=haft

def standing(lean=0,bob=0,step=0):
    im=Image.new('RGBA',(96,80))
    paste(im,body.crop((0,0,80,38)),(8+lean,8+bob))
    paste(im,body.crop((0,38,45,64)),(8-step,46))
    paste(im,body.crop((45,38,80,64)),(53+step,46-abs(step)//2))
    return im

def swing(angle,lean=0,bob=0):
    im=blank();paste(im,torso,(40+lean,24+bob))
    # Articulated sleeve connects the approved shoulder and rotating gauntlet.
    rad=math.radians(angle);dx,dy=11,-1
    hand=(98+lean+math.cos(rad)*dx+math.sin(rad)*dy,48+bob-math.sin(rad)*dx+math.cos(rad)*dy)
    draw=ImageDraw.Draw(im);draw.line([(96+lean,53+bob),hand],fill='#302d35',width=5)
    draw.line([(96+lean,52+bob),hand],fill='#635963',width=2)
    layer=blank();paste(layer,weapon,(40+lean,24+bob))
    # The shoulder and gauntlet turn together; no detached hand follows the blade.
    layer=layer.rotate(angle,Image.Resampling.NEAREST,center=(98+lean,48+bob))
    im.alpha_composite(layer)
    return im

anims={
 'idle':[(standing(bob=b),.18) for b in [0,0,-1,0]],
 'run':[(standing(lean=1,bob=b,step=s),.08) for s,b in [(0,0),(2,-1),(1,0),(0,0),(-2,-1),(-1,0)]],
 'attack':[(swing(a,l,b),.08) for a,l,b in [(0,0,0),(-45,-1,-1),(-100,2,1),(-150,3,2)]],
 'skill1':[(swing(a,l,b),.08) for a,l,b in [(0,0,0),(-60,-1,-1),(-130,2,1),(-170,3,2)]],
 'skill2':[(standing(l,b,s),.08) for l,b,s in [(1,1,0),(3,2,1),(4,3,2),(2,1,0)]],
 'ult':[(swing(a,l,b),.10) for a,l,b in [(30,-2,0),(10,-1,-1),(-65,2,0),(-115,3,1)]],
 'hit':[(standing(-2,1),.14)],
 'dead':[]}
for i,a in enumerate([0,12,30,55,75,90]):
    layer=blank();paste(layer,body,(40,24));layer=layer.rotate(a,Image.Resampling.NEAREST,center=(80,76));anims['dead'].append((layer,.10))

def pack(groups,prefix):
    atlas=Image.new('RGBA',(1024,2048));x=y=row=0;data={'anims':{}};seen={}
    for name,frames in groups.items():
        entry={'frames':[]};data['anims'][name]=entry
        for im,dur in frames:
            w,h=im.size
            assert w<=2048 and h<=2048 and im.getbbox(),name
            key=(im.size,im.tobytes())
            if key in seen:
                entry['frames'].append({'duration':dur,'data':dict(seen[key])});continue
            if x+w+2>1024:x=0;y+=row+2;row=0
            assert y+h+2<=2048,name
            atlas.alpha_composite(im,(x,y))
            frame={'x':x,'y':y,'w':w,'h':h};seen[key]=frame
            entry['frames'].append({'duration':dur,'data':frame})
            x+=w+2;row=max(row,h)
    height=2**math.ceil(math.log2(y+row+2));atlas=atlas.crop((0,0,1024,height))
    atlas.save(OUT/(prefix+'.png'));(OUT/(prefix+'.fanim')).write_text(json.dumps(data,indent=2)+'\n')
    return atlas,data

both=dict(anims)
both.update({name+'_l':[(ImageOps.mirror(im),dur) for im,dur in fs] for name,fs in anims.items()})
atlas,metadata=pack(both,'body-preview')
source=Image.open(fx_source).convert('RGBA');cells=[]
for i in range(8):
    cell=source.crop(((i%4)*source.width//4,(i//4)*source.height//2,(i%4+1)*source.width//4,(i//4+1)*source.height//2))
    # Remove only alpha fringes, retaining every visible solid source shape.
    cell.putalpha(cell.getchannel('A').point(lambda a:255 if a>=160 else 0));cells.append(cell.crop(cell.getbbox()))
def fit(i,w,h):
    s=cells[i];k=min(w/s.width,h/s.height);s=s.resize((max(1,round(s.width*k)),max(1,round(s.height*k))),Image.Resampling.NEAREST)
    im=Image.new('RGBA',(w,h));paste(im,s,((w-s.width)//2,(h-s.height)//2));return im
def phases(im,n=4,dur=.08,grow=True):
    result=[]
    for i in range(n):
        k=(.65+.35*i/(n-1)) if grow else 1
        s=im.resize((max(1,round(im.width*k)),max(1,round(im.height*k))),Image.Resampling.NEAREST);o=Image.new('RGBA',im.size)
        if i==n-1:s.putalpha(s.getchannel('A').point(lambda a:a//2))
        paste(o,s,((im.width-s.width)//2,(im.height-s.height)//2));result.append((o,dur))
    return result
effects={}
for tag,index,w,h in [('ghost',0,56,56),('ghost_end',1,64,64),('pillar_rise',2,40,56),('pillar_break',4,56,56),('slash',5,72,72),('bash_dust',6,36,24),('slam',7,80,80),('land',7,80,80),('leap',6,40,32)]:effects[tag]=phases(fit(index,w,h))
effects['slash_l']=[(ImageOps.mirror(im),dur) for im,dur in effects['slash']]
effects['pillar']=[(fit(2,40,56),.1)]
effects['fissure']=phases(fit(3,24,16),4,.08,False)
cfg=json.loads((ROOT/'rules/grimhald.json').read_text())['presentation']
for angle in range(cfg['trailDirections']):
    rad=angle*2*math.pi/cfg['trailDirections']
    for length in range(1,cfg['trailMaxSteps']+1):
        im=Image.new('RGBA',(56,56))
        for i in range(length+1):
            off=(i/length-.5)*length*cfg['trailLengthStep']/cfg['unitsPerPixel']
            tile=fit(3,12,8);paste(im,tile,(22+math.cos(rad)*off,24+math.sin(rad)*off))
        effects['trail_d'+str(angle)+'_l'+str(length)]=[(im,.1)]

effects['trail']=[(fit(3,20,14),.1)]
# Full pentagon uses the same radius/orientation as grimhald_core::pentagon.
# Imagegen supplies material; this compositor supplies exact gameplay geometry.
rules=json.loads((ROOT/'rules/grimhald.json').read_text());r=rules['ult']['cageRadius']/950
for phase in range(4):
    im=Image.new('RGBA',(160,160));d=ImageDraw.Draw(im);vs=[(80+math.cos(-math.pi/2+k*2*math.pi/5)*r,80+math.sin(-math.pi/2+k*2*math.pi/5)*r) for k in range(5)]
    d.line(vs+[vs[0]],fill='#271f24',width=15);d.line(vs+[vs[0]],fill='#79615a',width=11);d.line(vs+[vs[0]],fill='#441d21',width=3)
    for a,b in zip(vs,vs[1:]+vs[:1]):
        for t in [0,.25,.5,.75]:
            p=(round(a[0]+(b[0]-a[0])*t),round(a[1]+(b[1]-a[1])*t));tile=fit(2,15,19)
            paste(im,tile,(p[0]-7,p[1]-12))
        d.line([(a[0],a[1]-2),(b[0],b[1]-2)],fill='#ed783e',width=1)
    if phase<3:
        alpha=im.getchannel('A').point(lambda a:a*(phase+1)//4);im.putalpha(alpha)
    effects['cage_phase'+str(phase)]=[(im,.1)]
effects['cage_hold']=effects['cage_phase3'];effects['cage_rise']=effects['cage_phase0'];effects['cage_block']=effects['cage_phase3']
for i in range(8):
    # The projectile contains only the blade and wood, never the supporting hand.
    flying=Image.new('RGBA',body.size);fd=ImageDraw.Draw(flying)
    fd.line([(44,16),(73,23)],fill='#302322',width=3);fd.line([(44,15),(73,22)],fill='#604033',width=1)
    for y in range(24):
        for x in range(51):
            if (x<=50 and y<14) or (x<=44 and y<24):
                pixel=body.getpixel((x,y))
                if pixel[3]:flying.putpixel((x,y),pixel)
    im=Image.new('RGBA',(128,128));paste(im,flying,(24,32));im=im.rotate(i*45,Image.Resampling.NEAREST,center=(64,64));effects['axe_p'+str(i)]=[(im,.1)]
for tier in range(1,4):
    fs=[]
    for phase in range(8):
        im=Image.new('RGBA',(64,64));d=ImageDraw.Draw(im)
        for k in range(tier*3):
            x=20+(k*13+phase*2)%34;y=51-(phase*3+k*5)%35;d.rectangle((x,y,x+1,y+2),fill=['#a43429','#d15b2d','#f99b39'][tier-1])
        fs.append((im,.1))
    effects['rage'+str(tier)]=fs
fxatlas,fxmeta=pack(effects,'effects-preview')
# A readable contact sheet and an animated full pose review, both facings/terrain.
sheet=Image.new('RGB',(1024,720),'#172d38');d=ImageDraw.Draw(sheet)
for row,(name,frames) in enumerate(anims.items()):
    d.text((8,row*90+4),name,fill='#efc090')
    for i,(im,dur) in enumerate(frames):
        s=im.resize((im.width*1,im.height*1),Image.Resampling.NEAREST);sheet.paste(s,(95+i*145,row*90-5),s)
sheet.save(OUT/'animation-contact.png')
gifs=[]
for name,fs in anims.items():
    for im,duration in fs:
        s=Image.new('RGB',(1440,460),'#17333d');dr=ImageDraw.Draw(s);dr.rectangle((720,0,1440,460),fill='#b7c2ae');dr.text((12,12),name+' | 1x and 3x | mirrored',fill='#f4c788')
        for left in [False,True]:
            a=ImageOps.mirror(im) if left else im;off=720 if left else 0
            for scale,x,y in [(1,off+16,90),(3,off+208,65)]:
                a2=a.resize((a.width*scale,a.height*scale),Image.Resampling.NEAREST);s.paste(a2,(x,y),a2)
        gifs.append((s,round(duration*1000)))
gifs[0][0].save(OUT/'animation-preview.gif',save_all=True,append_images=[g[0] for g in gifs[1:]],duration=[g[1] for g in gifs],loop=0,disposal=2)
sheet=Image.new('RGB',(960,640),'#233840');d=ImageDraw.Draw(sheet)
for i,(name,fs) in enumerate((p for p in effects.items() if not p[0].startswith('trail_d'))):
    x=(i%8)*120;y=(i//8)*120;im=fs[-1][0];im.thumbnail((110,90),Image.Resampling.NEAREST);sheet.paste(im,(x+(120-im.width)//2,y+18),im);d.text((x+4,y+3),name,fill='#edcba3')
sheet.save(OUT/'effects-contact.png')
info={'status':'Approved appearance and accepted skill direction; corrected preview extraction/layout. Installation and live acceptance pending.','images':{'right':'../docs/grimhald-art-review/proposal-v2-right.png','left':'../docs/grimhald-art-review/proposal-v2-left.png'},'atlas':'../docs/grimhald-art-review/body-preview.png','metadata':'../docs/grimhald-art-review/body-preview.fanim','effects':'../docs/grimhald-art-review/effects-preview.png','effectMetadata':'../docs/grimhald-art-review/effects-preview.fanim'}
(ROOT/'editor/grimhald-art.json').write_text(json.dumps(info,indent=2)+'\n')
manifest={'appearance':'approved corrected grip','effects':'direction accepted with preview correction; corrected extraction and layout verified','generator':'tools/animate-grimhald.py','decodedBytes':atlas.width*atlas.height*4+fxatlas.width*fxatlas.height*4,'sourceHashes':{str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest() for p in [SRC,fx_source]},'outputs':{p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in [OUT/(n+s) for n in ['body-preview','effects-preview'] for s in ['.png','.fanim']]},'frameCounts':{k:len(v) for k,v in anims.items()},'anchors':'Every standing frame: feet 20 px below centre. Padding only for swing/death extents.'}
assert manifest['decodedBytes']<=32*1024*1024
(OUT/'animation-manifest.json').write_text(json.dumps(manifest,indent=2)+'\n');print(json.dumps(manifest,indent=2))
