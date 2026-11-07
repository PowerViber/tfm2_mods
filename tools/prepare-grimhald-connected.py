"""Review-only complete poses and revised volcanic materials. No sliced body rig."""
from pathlib import Path
from PIL import Image,ImageDraw,ImageOps
import json,math,hashlib
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/grimhald-art-review'
rules=json.loads((ROOT/'rules/grimhald.json').read_text());PIX=rules['presentation']['unitsPerPixel']
body=Image.open(OUT/'source-connected-v3.png').convert('RGBA');names=['idle','run','bash','unarmed'];poses=[]
scale=39/(body.height*.375)
for i,name in enumerate(names):
    x,y=i%2,i//2;im=body.crop((x*body.width//2,y*body.height//2,(x+1)*body.width//2,(y+1)*body.height//2));im=im.crop(im.getbbox())
    im=im.resize((round(im.width*scale),round(im.height*scale)),Image.Resampling.NEAREST)
    frame=Image.new('RGBA',(96,80));frame.alpha_composite(im,(48-im.width//2,60-im.getbbox()[3]));frame.save(OUT/f'connected-v3-{name}.png');poses.append(frame)
sheet=Image.new('RGB',(1200,600));d=ImageDraw.Draw(sheet)
for row,bg in enumerate(['#19313b','#b7c2ae']):
    d.rectangle((0,row*300,1200,row*300+300),fill=bg)
    for i,(name,f) in enumerate(zip(names,poses)):
        x=i*300;d.text((x+12,row*300+10),name+' | whole-body 1x / 3x',fill='#efbb86' if row==0 else '#23313b')
        sheet.paste(f,(x+12,row*300+40),f);big=f.resize((288,240),Image.Resampling.NEAREST);sheet.paste(big,(x+6,row*300+100),big)
sheet.save(OUT/'connected-v3-review.png')
# Reuse the prior reviewed materials except the four requested replacements.
old=Image.open(OUT/'v1/effects-preview.png').convert('RGBA');oldmeta=json.loads((OUT/'v1/effects-preview.fanim').read_text())['anims'];effects={}
for name,a in oldmeta.items():
    effects[name]=[(old.crop((f['data']['x'],f['data']['y'],f['data']['x']+f['data']['w'],f['data']['y']+f['data']['h'])),f['duration']) for f in a['frames']]
source=Image.open(OUT/'source-effects-v2.png').convert('RGBA');cells=[]
for i in range(4):
    x,y=i%2,i//2;im=source.crop((x*source.width//2,y*source.height//2,(x+1)*source.width//2,(y+1)*source.height//2))
    # The solid object supplies the crop, with surrounding transparent margin.
    box=im.getchannel('A').point(lambda a:255 if a>180 else 0).getbbox() or im.getbbox()
    box=(max(0,box[0]-12),max(0,box[1]-12),min(im.width,box[2]+12),min(im.height,box[3]+12));cells.append(im.crop(box))
def fit(index,w,h):
    im=cells[index];k=min(w/im.width,h/im.height);s=im.resize((max(1,round(im.width*k)),max(1,round(im.height*k))),Image.Resampling.NEAREST)
    o=Image.new('RGBA',(w,h));o.alpha_composite(s,((w-s.width)//2,(h-s.height)//2));return o
def phases(im):
    fs=[]
    for k in range(4):
        o=Image.new('RGBA',im.size);a=im.copy();a.putalpha(a.getchannel('A').point(lambda v:v*[180,255,200,80][k]//255));o.alpha_composite(a);fs.append((o,.08))
    return fs
# A true flat top-down crack, not the previous raised stone silhouette.
floor=Image.open(OUT/'source-ground-fissure-v3.png').convert('RGBA')
floor.putalpha(floor.getchannel('A').point(lambda a:a if a>=180 else 0));floor=floor.crop(floor.getbbox())
floor=floor.resize((40,max(1,round(floor.height*40/floor.width))),Image.Resampling.NEAREST)
flat=Image.new('RGBA',(48,24));flat.alpha_composite(floor,((48-floor.width)//2,(24-floor.height)//2))
effects['fissure']=phases(flat);
for direction in range(rules['presentation']['trailDirections']):
    rotated=flat.rotate(-direction*360/rules['presentation']['trailDirections'],Image.Resampling.NEAREST,expand=True)
    effects['fissure_d'+str(direction)]=phases(rotated)
pillar=fit(1,40,56)
# Transparent padding aligns the standing pillar's base with the ground anchor.
base=pillar.getbbox()[3];grounded=Image.new('RGBA',(40,base*2));grounded.alpha_composite(pillar,(0,0))
effects['pillar']=[(grounded,.1)];effects['pillar_rise']=phases(fit(1,40,56));effects['bash_dust']=phases(fit(2,40,28));effects['slam']=phases(fit(3,64,64))
# Resize the old full pentagon to the newly authorized radius, preserving its material.
old_radius=55000;new_radius=rules['ult']['cageRadius'];ratio=new_radius/old_radius
for name in [n for n in effects if n.startswith('cage_')]:
    effects[name]=[(im.resize((round(im.width*ratio),round(im.height*ratio)),Image.Resampling.NEAREST),dur) for im,dur in effects[name]]
r=rules['ult']['knockRadius']/PIX;w=2*math.ceil(r)+8
mark=Image.new('RGBA',(w,w));d=ImageDraw.Draw(mark);d.ellipse((w/2-r,w/2-r,w/2+r,w/2+r),outline='#e29a66',width=1);d.line([(w/2-3,w/2),(w/2+3,w/2)],fill='#e29a66');d.line([(w/2,w/2-3),(w/2,w/2+3)],fill='#e29a66');effects['ult_marker']=[(mark,.1)]
atlas=Image.new('RGBA',(1024,2048));metadata={'anims':{}};x=y=row=0;seen={}
for name,fs in effects.items():
    entries=[]
    for im,dur in fs:
        key=(im.size,im.tobytes())
        if key not in seen:
            if x+im.width>1024:x=0;y+=row+2;row=0
            assert y+im.height<=2048
            atlas.alpha_composite(im,(x,y));seen[key]={'x':x,'y':y,'w':im.width,'h':im.height};x+=im.width+2;row=max(row,im.height)
        entries.append({'duration':dur,'data':seen[key]})
    metadata['anims'][name]={'frames':entries}
height=2**math.ceil(math.log2(y+row+2));atlas=atlas.crop((0,0,1024,height));atlas.save(OUT/'effects-v2-preview.png');(OUT/'effects-v2-preview.fanim').write_text(json.dumps(metadata,indent=2)+'\n')
sheet=Image.new('RGB',(800,260),'#19313b');d=ImageDraw.Draw(sheet)
for i,name in enumerate(['fissure','pillar','bash_dust','slam']):
    im=effects[name][0][0];a=im.resize((im.width*3,im.height*3),Image.Resampling.NEAREST);sheet.paste(a,(i*200+20,50),a);d.text((i*200+20,20),name,fill='#efbb86')
sheet.save(OUT/'effects-v2-review.png')
p=ROOT/'editor/grimhald-art.json';info=json.loads(p.read_text());info.update(effects='../docs/grimhald-art-review/effects-v2-preview.png',effectMetadata='../docs/grimhald-art-review/effects-v2-preview.fanim');p.write_text(json.dumps(info,indent=2)+'\n')
manifest={'rules_revision':rules['revision'],'status':'review only; new body and material approval pending','sourceHashes':{n:hashlib.sha256((OUT/n).read_bytes()).hexdigest() for n in ['source-connected-v3.png','source-effects-v2.png','source-ground-fissure-v3.png']},'wholeBodyPoses':4,'decodedEffectsBytes':atlas.width*atlas.height*4}
(OUT/'connected-v3-manifest.json').write_text(json.dumps(manifest,indent=2)+'\n');print(json.dumps(manifest))
