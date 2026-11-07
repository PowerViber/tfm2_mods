"""Extract six complete locomotion drawings. Never cut the body into moving parts."""
from pathlib import Path
from PIL import Image,ImageDraw,ImageOps
import hashlib,json,math
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/grimhald-art-review'
rules=json.loads((ROOT/'rules/grimhald-art.json').read_text());durations=[n/1000 for n in rules['frameMillis']['run']]
assert len(durations)==6
SCALE=39/350
def visor(cell):
    candidates=[];seen=set();p=cell.load()
    def orange(x,y):
        r,g,b,a=p[x,y];return a>=180 and r>170 and 65<g<215 and b<90
    for y in range(100,240):
        for x in range(260,420):
            if (x,y) in seen or not orange(x,y):continue
            stack=[(x,y)];pts=[];seen.add((x,y))
            while stack:
                a,b=stack.pop();pts.append((a,b))
                for dx,dy in [(1,0),(-1,0),(0,1),(0,-1)]:
                    n=(a+dx,b+dy)
                    if 260<=n[0]<420 and 100<=n[1]<240 and n not in seen and orange(*n):seen.add(n);stack.append(n)
            if len(pts)>3:
                xs=[n[0] for n in pts];ys=[n[1] for n in pts];cx=(min(xs)+max(xs))/2;cy=(min(ys)+max(ys))/2
                if max(xs)-min(xs)>=max(ys)-min(ys):candidates.append(((cx-330)**2+(cy-180)**2,cx))
    assert candidates,'visor landmark not found'
    return min(candidates)[1]
def extract(file):
    source=Image.open(OUT/file).convert('RGBA');assert source.size==(1536,1024)
    frames=[];anchors=[]
    for i in range(6):
        c=source.crop((i%3*512,i//3*512,i%3*512+512,i//3*512+512));eye=visor(c)
        # Deterministic matte cleanup removes only the generated alpha fringe.
        c.putalpha(c.getchannel('A').point(lambda a:a if a>=180 else 0))
        s=c.resize((round(512*SCALE),round(512*SCALE)),Image.Resampling.NEAREST);b=s.getbbox();assert b
        frame=Image.new('RGBA',(96,80));dx=52-round(eye*SCALE);dy=60-b[3];frame.alpha_composite(s,(dx,dy));assert frame.getbbox()[3]==60
        frames.append(frame);anchors.append({'visorSourceX':eye,'paste':[dx,dy]})
    return frames,anchors
armed,armed_anchors=extract('source-walk-v4.png');unarmed,unarmed_anchors=extract('source-walk-unarmed-v4.png')
anims={}
for row,(name,frames) in enumerate([('run',armed),('run_unarmed',unarmed)]):
    for left in [False,True]:
        entries=[]
        for i,(frame,duration) in enumerate(zip(frames,durations)):
            if left:frame=ImageOps.mirror(frame)
            x=i*98
            # 80 px cells need four rows in a 512 px atlas.
            y=(row*2+int(left))*82
            entries.append({'duration':duration,'data':{'x':x,'y':y,'w':96,'h':80}})
        anims[name+('_l' if left else '')]={'frames':entries}
atlas=Image.new('RGBA',(768,512))
for name,entry in anims.items():
    fs=unarmed if name.startswith('run_unarmed') else armed
    for frame,f in zip(fs,entry['frames']):
        if name.endswith('_l'):frame=ImageOps.mirror(frame)
        d=f['data'];atlas.alpha_composite(frame,(d['x'],d['y']))
atlas.save(OUT/'walk-v4-preview.png');(OUT/'walk-v4-preview.fanim').write_text(json.dumps({'anims':anims},indent=2)+'\n')
gifs=[]
for i in range(6):
    sheet=Image.new('RGB',(1000,560));d=ImageDraw.Draw(sheet)
    for row,fs in enumerate([armed,unarmed]):
        for col,left in enumerate([False,True]):
            x=col*500;y=row*280;bg='#b7c2ae' if left else '#19313b';d.rectangle((x,y,x+499,y+279),fill=bg)
            colour='#23313b' if left else '#efbb86';d.text((x+10,y+8),('unarmed' if row else 'armed')+(' left' if left else ' right')+' | frame '+str(i+1)+'/6',fill=colour)
            f=ImageOps.mirror(fs[i]) if left else fs[i]
            sheet.paste(f,(x+10,y+45),f);big=f.resize((288,240),Image.Resampling.NEAREST);sheet.paste(big,(x+160,y+24),big)
            d.line((x+160,y+204,x+448,y+204),fill='#638080' if not left else '#8c9a8a');d.text((x+10,y+130),'1x',fill=colour);d.text((x+160,y+220),'3x',fill=colour)
    gifs.append(sheet)
gifs[0].save(OUT/'walk-v4-preview.gif',save_all=True,append_images=gifs[1:],duration=[round(n*1000) for n in durations],loop=0,disposal=2)
sheet=Image.new('RGB',(960,300),'#19313b');d=ImageDraw.Draw(sheet)
for i,im in enumerate(armed):
    big=im.resize((192,160),Image.Resampling.NEAREST);sheet.paste(big,(i*160-16,40),big);d.text((i*160+10,220),'frame '+str(i+1),fill='#efbb86')
sheet.save(OUT/'walk-v4-contact.png')
unique=len({f.tobytes() for f in armed});assert unique==6
leg_counts={name:len({f.crop((0,50,96,60)).getchannel('A').tobytes() for f in frames}) for name,frames in [('armed',armed),('unarmed',unarmed)]}
assert all(n==6 for n in leg_counts.values())
info=json.loads((ROOT/'editor/grimhald-art.json').read_text());info['images']={};info.pop('atlas',None);info.pop('metadata',None);info.update(cycleAtlas='../docs/grimhald-art-review/walk-v4-preview.png',cycleMetadata='../docs/grimhald-art-review/walk-v4-preview.fanim',footAnchorBelowCentre=20,status='Approved six-frame walk; nominal cadence slowed to 160 ms per frame. Remaining cycles are in progress.')
(ROOT/'editor/grimhald-art.json').write_text(json.dumps(info,indent=2)+'\n')
manifest={'sourceSha256':{n:hashlib.sha256((OUT/n).read_bytes()).hexdigest() for n in ['source-walk-v4.png','source-walk-unarmed-v4.png']},'frames':6,'uniqueLowerLegSilhouettes':leg_counts,'uniqueCompleteArmedFrames':unique,'uniqueCompleteUnarmedFrames':len({f.tobytes() for f in unarmed}),'frameMillis':rules['frameMillis']['run'],'footAnchorBelowCentre':20,'armedRegistration':armed_anchors,'unarmedRegistration':unarmed_anchors,'method':'Uniform full-frame downsampling and registration; no separated torso or limb transforms','decodedBytes':atlas.width*atlas.height*4,'status':'whole-body walk approved; cadence revised by the user to match movement'}
(OUT/'walk-v4-manifest.json').write_text(json.dumps(manifest,indent=2)+'\n');print(json.dumps(manifest,indent=2))
