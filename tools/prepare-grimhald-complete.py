"""Pack full-body authored animation and a rigid axe accessory. No limb rig."""
from pathlib import Path
from PIL import Image, ImageOps, ImageDraw
import hashlib, json, math
import shutil

ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/'docs/grimhald-art-review'
ART=json.loads((ROOT/'rules/grimhald-art.json').read_text())
PREFIX=ART['weaponPrefix']

def clean(im):
    im=im.convert('RGBA')
    im.putalpha(im.getchannel('A').point(lambda a:a if a>=180 else 0))
    return im

def unpack(atlas,entry):
    d=entry['data'];return atlas.crop((d['x'],d['y'],d['x']+d['w'],d['y']+d['h']))

old=Image.open(OUT/'movement-v5-preview.png').convert('RGBA')
om=json.loads((OUT/'movement-v5-preview.fanim').read_text())['anims']
body={name:[unpack(old,f) for f in om[name+'_unarmed']['frames']] for name in ['idle','run']}
registration={}
for name in ['attack','skill1','skill2','ult','dead']:
    source=clean(Image.open(OUT/f'source-{name}-v6.png'))
    cols=3 if name=='dead' else 2;rows=2;cw,ch=source.width//cols,source.height//rows
    # The first drawing is the standing reference. One scale for the ENTIRE
    # sequence preserves actual crouching/falling instead of inflating it.
    scale=39/365 if name=='dead' else .09017341040462427
    first=source.crop((0,0,cw,ch));fb=first.getbbox()
    offset_x=48-round((fb[0]+fb[2])/2*scale)
    fs=[];poses=[]
    for i in range(cols*rows):
        cell=source.crop((i%cols*cw,i//cols*ch,(i%cols+1)*cw,(i//cols+1)*ch))
        small=cell.resize((round(cw*scale),round(ch*scale)),Image.Resampling.NEAREST)
        b=small.getbbox();assert b
        y=60-b[3]
        frame=Image.new('RGBA',(96,80));frame.alpha_composite(small,(offset_x,y))
        box=frame.getbbox();assert box and 0<box[0] and 0<box[1] and box[2]<96 and box[3]==60,(name,i,box)
        fs.append(frame);poses.append({'paste':[offset_x,y]})
    body[name]=fs
    registration[name]={'scale':scale,'cell':[cw,ch],'frames':poses}
body['hit']=[body['dead'][0].copy()]

# The axe is a complete rigid object. Only that object is rotated/translated;
# character anatomy always comes from a complete authored drawing above.
source_axe=clean(Image.open(OUT/'source-axe-v6.png'))
axe_box=source_axe.getbbox();source_axe=source_axe.crop(axe_box)
axe=source_axe.resize((43,round(source_axe.height*43/source_axe.width)),Image.Resampling.NEAREST)
# Grip coordinates in the isolated, downsampled accessory (wood near pommel).
grip=(33,24)
# Positions refer to complete 96x80 body cells; angles turn the whole axe only.
# Hand occlusion is provided by the unarmed body, rendered over the accessory.
rig={
 'idle':[(58,30,0),(59,30,0),(58,30,0),(59,30,0)],
 'run':[(60,29,0),(59,31,0),(60,29,0),(60,29,0),(59,32,0),(60,30,0)],
 'attack':[(59,30,0),(39,24,-100),(58,42,180),(40,42,30)],
 'skill1':[(59,30,0),(48,18,-95),(54,57,235),(54,45,205)],
 'skill2':[(59,30,0),(39,40,0),(33,42,0),(40,42,0)],
 'ult':[(59,30,0),(40,23,-90),(59,22,-150),None],
 'hit':[(57,30,0)],
 'dead':[None]*6,
}

def weapon(spec):
    canvas=Image.new('RGBA',(128,112))
    if spec is None:return canvas
    x,y,angle=spec
    # A large transparent pivot canvas avoids rotation clipping.
    pivot=Image.new('RGBA',(128,128));pivot.alpha_composite(axe,(64-grip[0],64-grip[1]))
    rotated=pivot.rotate(angle,Image.Resampling.NEAREST,center=(64,64))
    # Body and accessory frame centres share the same world point.
    canvas.alpha_composite(rotated,(x+16-64,y+16-64))
    b=canvas.getbbox();assert not b or (b[0]>0 and b[1]>0 and b[2]<128 and b[3]<112),(spec,b)
    return canvas

weapons={name:[weapon(p) for p in rig[name]] for name in ART['frameMillis']}

def pack(animations, width=1024):
    x=y=row=0;seen={};placements={}
    for name,fs in animations.items():
        entries=[]
        for im,duration in fs:
            key=(im.size,im.tobytes())
            if key not in seen:
                if x+im.width>width:x=0;y+=row+2;row=0
                assert y+im.height<=2048
                seen[key]=(im.copy(),{'x':x,'y':y,'w':im.width,'h':im.height})
                x+=im.width+2;row=max(row,im.height)
            entries.append({'duration':duration,'data':seen[key][1]})
        placements[name]={'frames':entries}
    height=2**math.ceil(math.log2(y+row+2));assert height<=2048
    sheet=Image.new('RGBA',(width,height))
    for im,d in seen.values():sheet.alpha_composite(im,(d['x'],d['y']))
    return sheet,{'anims':placements}

base={};accessory={}
for name,ms in ART['frameMillis'].items():
    assert len(body[name])==len(ms)==len(weapons[name])
    for left in [False,True]:
        suffix='_l' if left else ''
        base[name+suffix]=[(ImageOps.mirror(im) if left else im,m/1000) for im,m in zip(body[name],ms)]
        # Extra lab names alias the same complete, unarmed body pixels.
        base[name+'_unarmed'+suffix]=base[name+suffix]
        for i,im in enumerate(weapons[name]):
            tag=PREFIX+name+('_l' if left else '_r')+str(i)
            accessory[tag]=[(ImageOps.mirror(im) if left else im,.1)]

sheet,metadata=pack(base)
ws,wm=pack(accessory)
sheet.save(OUT/'body-v6-preview.png');(OUT/'body-v6-preview.fanim').write_text(json.dumps(metadata,indent=2)+'\n')
ws.save(OUT/'weapon-v6-preview.png');(OUT/'weapon-v6-preview.fanim').write_text(json.dumps(wm,indent=2)+'\n')

# Keep all prior effect pixels/durations, then append the planted-axe phase.
fx_source=Image.open(OUT/'effects-v2-preview.png').convert('RGBA')
fx_meta=json.loads((OUT/'effects-v2-preview.fanim').read_text())['anims']
fx={name:[(unpack(fx_source,f),f['duration']) for f in a['frames']] for name,a in fx_meta.items()}
for i in range(8):
    # The core emits exactly one flight phase per tick. Keep the pixel drawing
    # but expire that phase after one tick, rather than stacking six rotations.
    fx['axe_p'+str(i)]=[(im,1/60) for im,_ in fx['axe_p'+str(i)]]
standing=axe.rotate(-70,Image.Resampling.NEAREST,expand=True)
standing=standing.crop(standing.getbbox())
planted=Image.new('RGBA',(64,112));planted.alpha_composite(standing,((64-standing.width)//2,56-standing.height))
fx['axe_planted']=[(planted,.1)]
fx_sheet,fx_metadata=pack(fx)
fx_sheet.save(OUT/'effects-v6-preview.png');(OUT/'effects-v6-preview.fanim').write_text(json.dumps(fx_metadata,indent=2)+'\n')

def composite(name,i,left=False,armed=True):
    result=Image.new('RGBA',(128,112))
    if armed:result.alpha_composite(weapons[name][i])
    result.alpha_composite(body[name][i],(16,16))
    return ImageOps.mirror(result) if left else result

# Every frame is shown as an assembled body, with an actual-size copy and 3x.
contact=Image.new('RGB',(1536,8*280),'#19313b');d=ImageDraw.Draw(contact)
for row,name in enumerate(ART['frameMillis']):
    d.text((12,row*280+8),name,fill='#edbc8c')
    for i in range(len(body[name])):
        im=composite(name,i);big=im.resize((256,224),Image.Resampling.NEAREST)
        contact.paste(big,(i*256,row*280+30),big)
contact.save(OUT/'body-v6-contact.png')

for name,ms in ART['frameMillis'].items():
    frames=[]
    for i in range(len(body[name])):
        panel=Image.new('RGB',(1040,380));d=ImageDraw.Draw(panel)
        for col,(left,armed) in enumerate([(False,True),(True,True),(False,False),(True,False)]):
            x=col*260;bg='#b7c2ae' if left else '#19313b'
            d.rectangle((x,0,x+259,379),fill=bg)
            d.text((x+10,10),f'{name} {i+1}/{len(ms)} '+('armed' if armed else 'unarmed'),fill='#263b42' if left else '#edbc8c')
            im=composite(name,i,left,armed)
            panel.paste(im,(x,38),im)
            big=im.resize((384,336),Image.Resampling.NEAREST);panel.paste(big,(x-65,100),big)
            d.line((x+10,328,x+250,328),fill='#778a85')
        frames.append(panel)
    frames[0].save(OUT/f'body-v6-{name}.gif',save_all=True,append_images=frames[1:],duration=ms,loop=0,disposal=2)

info=json.loads((ROOT/'editor/grimhald-art.json').read_text())
info.update(revision=6,images={},candidateImages={},cycleAtlas='../docs/grimhald-art-review/body-v6-preview.png',
            cycleMetadata='../docs/grimhald-art-review/body-v6-preview.fanim',
            weaponAtlas='../docs/grimhald-art-review/weapon-v6-preview.png',
            weaponMetadata='../docs/grimhald-art-review/weapon-v6-preview.fanim',weaponPrefix=PREFIX,
            effects='../docs/grimhald-art-review/effects-v6-preview.png',effectMetadata='../docs/grimhald-art-review/effects-v6-preview.fanim',
            status='All eight complete full-body animation cycles, both facings and separate axe. Native alignment/live acceptance pending.')
(ROOT/'editor/grimhald-art.json').write_text(json.dumps(info,indent=2)+'\n')

sources=[f'source-{n}-v6.png' for n in ['attack','skill1','skill2','ult','dead','axe']]
manifest={'revision':6,'sourceSha256':{n:hashlib.sha256((OUT/n).read_bytes()).hexdigest() for n in sources},
          'registration':registration,'weaponGrip':grip,'weaponRig':rig,'frames':{n:len(f) for n,f in body.items()},
          'bodyCanvas':[96,80],'weaponCanvas':[128,112],'footBelowCentre':20,
          'decodedBytes':sheet.width*sheet.height*4+ws.width*ws.height*4+fx_sheet.width*fx_sheet.height*4,
          'method':'Whole-body source frames; rigid accessory transforms only; no torso/limb slicing',
          'integration':'repository assets enabled; installation and actual-game acceptance remain separate'}
(OUT/'body-v6-manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
if ART['integrated']:
    for kind,stem,source in [('champions','tfm2_custom_grimhald','body-v6-preview'),('vfx','grimhald_weapon','weapon-v6-preview'),('vfx','grimhald','effects-v6-preview')]:
        target=ROOT/'mods/tfm2_custom'/kind;target.mkdir(parents=True,exist_ok=True)
        shutil.copy2(OUT/(source+'.png'),target/(stem+'#sheet.png'))
        shutil.copy2(OUT/(source+'.fanim'),target/(stem+'#anim.fanim'))
print(json.dumps({k:v for k,v in manifest.items() if k not in ['registration','weaponRig']},indent=2))
