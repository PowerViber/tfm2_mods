"""Reproducibly extract approved frozen imagegen sources; AI generation is not deterministic."""
from pathlib import Path
from collections import deque
from PIL import Image, ImageDraw, ImageOps
import json, hashlib, statistics, math
ROOT=Path(__file__).resolve().parents[1]; OUT=ROOT/'docs/contender-art-review'; MOD=ROOT/'mods/tfm2_custom'

def figure(cell):
    mask=cell.getchannel('A').point(lambda a:255 if a>=180 else 0); pixels=mask.load(); seen=set(); biggest=[]
    for y in range(cell.height):
        for x in range(cell.width):
            if not pixels[x,y] or (x,y) in seen:continue
            q=deque([(x,y)]);seen.add((x,y));component=[]
            while q:
                p=q.popleft();component.append(p)
                for a,b in [(p[0]-1,p[1]),(p[0]+1,p[1]),(p[0],p[1]-1),(p[0],p[1]+1)]:
                    if 0<=a<cell.width and 0<=b<cell.height and pixels[a,b] and (a,b) not in seen:seen.add((a,b));q.append((a,b))
            if len(component)>len(biggest):biggest=component
    clean=Image.new('RGBA',cell.size);src=cell.load();dst=clean.load()
    for p in biggest:dst[p]=(*src[p][:3],255)
    return clean.crop(clean.getbbox())

def main():
    raw=[]
    for name in ['body','motion']:
        im=Image.open(OUT/(name+'-source.png')).convert('RGBA')
        for i in range(8):raw.append(figure(im.crop(((i%4)*384,(i//4)*512,(i%4+1)*384,(i//4+1)*512))))
    frames=[]
    for i,crop in enumerate(raw):
        scale=38/crop.height if i not in (11,12) else 38/statistics.median(x.height for x in raw[8:11])
        crop=crop.resize((max(1,round(crop.width*scale)),max(1,round(crop.height*scale))),Image.Resampling.NEAREST)
        feet=[x for y in range(max(0,crop.height-5),crop.height) for x in range(crop.width) if crop.getpixel((x,y))[3]]
        foot=round(statistics.mean(feet)) if feet else crop.width//2
        frame=Image.new('RGBA',(56,48));offset=(28-foot,45-crop.height)
        assert offset[0]>=0 and offset[0]+crop.width<=56,(i,crop.size,offset)
        frame.alpha_composite(crop,offset);frames.append(frame)
    strip=Image.new('RGBA',(448,48))
    for i,f in enumerate(frames[:8]):strip.alpha_composite(f,(56*i,0))
    strip.save(OUT/'body-poses.png')
    preview=Image.new('RGB',(1120,640),'#183b2d');d=ImageDraw.Draw(preview)
    labels=['Idle','Step','Jab','Cross','Body','Uppercut','Slip','Guard','Walk A','Walk B','Hit','KO kneel','KO ground','Challenge','Victory','Idle B']
    for i,f in enumerate(frames):
        x=(i%8)*140;y=(i//8)*320;d.rectangle((x,y,x+139,y+159),fill='#d4d6bd');d.text((x+6,y+8),labels[i],fill='#192234')
        preview.paste(f,(x+42,y+60),f);z=f.resize((112,96),Image.Resampling.NEAREST);preview.paste(z,(x+14,y+185),z)
    preview.save(OUT/'body-actual-size.png')
    atlas=Image.new('RGBA',(896,96));rects=[]
    for i,f in enumerate(frames):
        atlas.alpha_composite(f,(i*56,0));atlas.alpha_composite(ImageOps.mirror(f),(i*56,48));rects.append(dict(x=i*56,y=0,w=56,h=48))
    sequences={'idle':([0,0,15,0],.2),'run':([8,0,9,0,8,9],.08),'attack':([0,2,2,0],.06),'jab':([0,2,2,0],.06),'cross':([0,3,3,0],.06),'combination':([0,2,0,3,3,0],.1),'bodyShot':([0,0,4,4,0],.07),'uppercut':([0,4,5,5,0],.1),'slip':([6,6,6,0],.05),'guard':([7,7,7,0],.1125),'mainEvent':([0,13,13,13,0],.13),'skill1':([0,2,0,3,3,0],.1),'skill2':([7,7,7,0],.1125),'ult':([0,13,13,13,0],.13),'hit':([10,10,0],.08),'dead':([10,11,12,12,12,12],.12)}
    anims={}
    for name,(seq,duration) in sequences.items():
        for suffix,y in [('',0),('_l',48)]:anims[name+suffix]={'frames':[dict(duration=duration,data={**rects[i],'y':y}) for i in seq]}
    dest=MOD/'champions';dest.mkdir(exist_ok=True,parents=True);atlas.save(dest/'tfm2_custom_contender#sheet.png');(dest/'tfm2_custom_contender#anim.fanim').write_text(json.dumps({'anims':anims},separators=(',',':'))+'\n')
    gif=[]
    for name in ['idle','run','attack','combination','bodyShot','uppercut','slip','guard','mainEvent','hit','dead']:
        for i in sequences[name][0]:
            panel=Image.new('RGB',(448,192),'#17392c');a=frames[i].resize((168,144),Image.Resampling.NEAREST);b=ImageOps.mirror(a);panel.paste(a,(24,32),a);panel.paste(b,(248,32),b);ImageDraw.Draw(panel).text((8,5),name+' / both facings',fill='white');gif.append(panel)
    gif[0].save(OUT/'body-animations.gif',save_all=True,append_images=gif[1:],duration=100,loop=0)
    ring=Image.open(OUT/'ring-source.png').convert('RGBA');upper=ring.crop((0,0,ring.width,990));upper=upper.crop(upper.getchannel('A').point(lambda a:255 if a>=180 else 0).getbbox()).resize((208,208),Image.Resampling.NEAREST);upper.save(OUT/'ring-proposal.png')
    # The shared ivory rectangle supplies the precise usable boundary over the decorative ropes.
    fx=Image.new('RGBA',(1024,344));fx.alpha_composite(upper,(0,0));ImageDraw.Draw(fx).rectangle((21,21,187,187),outline='#efe7c4',width=1)
    vfx={'ring':{'frames':[dict(duration=5/60,data=dict(x=0,y=0,w=208,h=208))]}}
    for n in range(11):
        im=Image.new('RGBA',(40,14));draw=ImageDraw.Draw(im);draw.rectangle((1,3,38,9),fill='#152033',outline='#e9dfba');draw.rectangle((3,5,3+round(32*n/10),7),fill='#62cea6');fx.alpha_composite(im,(n*40,220));vfx['stamina'+str(n)]={'frames':[dict(duration=.1,data=dict(x=n*40,y=220,w=40,h=14))]}
    for i,name in enumerate(['impact','counter','guard_arc','return_red','return_blue']):
        cell=ring.crop((i*ring.width//5,995,(i+1)*ring.width//5,ring.height));b=cell.getchannel('A').point(lambda a:255 if a>=180 else 0).getbbox()
        if not b:continue
        im=cell.crop(b);im.thumbnail((32,32),Image.Resampling.NEAREST);frame=Image.new('RGBA',(40,40));frame.alpha_composite(im,((40-im.width)//2,40-im.height));fx.alpha_composite(frame,(i*40,250));vfx[name]={'frames':[dict(duration=.1,data=dict(x=i*40,y=250,w=40,h=40))]}
    for n in range(8):
        im=Image.new('RGBA',(48,48));draw=ImageDraw.Draw(im);dx=12*math.cos(n*math.pi/4);dy=12*math.sin(n*math.pi/4)
        a=(24-dx,24-dy);b=(24+dx,24+dy);draw.line((a,b),fill='#d7aa6280',width=12)
        for x,y in [a,b]:draw.ellipse((x-6,y-6,x+6,y+6),fill='#d7aa6280',outline='#f5d498')
        fx.alpha_composite(im,(n*48,294));vfx['warning'+str(n)]={'frames':[dict(duration=2/60,data=dict(x=n*48,y=294,w=48,h=48))]}
    v=MOD/'vfx';v.mkdir(exist_ok=True,parents=True);fx.save(v/'contender#sheet.png');(v/'contender#anim.fanim').write_text(json.dumps({'anims':vfx},separators=(',',':'))+'\n')
    compare=Image.new('RGB',(832,450),'#183b2d');compare.paste(upper,(10,20),upper);large=upper.resize((416,416),Image.Resampling.NEAREST);compare.paste(large,(400,15),large);compare.paste(frames[0],(88,98),frames[0]);compare.save(OUT/'ring-actual-size.png')
    manifest={'status':'approved-both-by-user','canvas':[56,48],'visibleHeight':38,'footBaseline':45,'ringCanvas':[208,208],'sources':{p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(OUT.glob('*-source.png'))},'decodedBytes':atlas.width*atlas.height*4+fx.width*fx.height*4,'method':'Frozen imagegen sources; largest alpha component, foot alignment, nearest-neighbour extraction and deterministic packing.'}
    (OUT/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n',encoding='utf8');print(json.dumps(manifest,indent=2))
if __name__=='__main__':main()
