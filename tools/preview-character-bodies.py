"""Pack frozen image-generation proposals for review. Never writes runtime assets.

Nearest-neighbour packing and frame-anchor registration are mechanical operations;
the approved AI source remains in docs for reproducibility and later approval.
"""
from pathlib import Path
from PIL import Image, ImageOps, ImageDraw, ImageFont
import hashlib, json, io, subprocess
ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/'docs/character-art-review'
FONT=ImageFont.load_default(size=12)
report={}
def component(im):
    """Discard detached packing noise, never repaint the proposed character."""
    pixels=im.load();seen=set();groups=[]
    for y in range(im.height):
        for x in range(im.width):
            if (x,y)in seen or pixels[x,y][3]<100:continue
            group=set();pending=[(x,y)];seen.add((x,y))
            while pending:
                q=pending.pop();group.add(q)
                for dy in [-1,0,1]:
                    for dx in [-1,0,1]:
                        n=(q[0]+dx,q[1]+dy)
                        if n not in seen and 0<=n[0]<im.width and 0<=n[1]<im.height and pixels[n[0],n[1]][3]>=100:
                            seen.add(n);pending.append(n)
            groups.append(group)
    keep=max(groups,key=len,default=set());result=Image.new('RGBA',im.size)
    for x,y in keep:result.putpixel((x,y),pixels[x,y])
    return result
for name,pack in [('omen','tfm2_valorant'),('steve','tfm2_blockcraft')]:
    prefix=ROOT/'mods'/pack/'champions'/f'{pack}_{name}'
    relative=str(prefix.relative_to(ROOT)).replace('\\','/')+'#sheet.png'
    old=Image.open(io.BytesIO(subprocess.check_output(['rtk','proxy','git','show','234caadbc29f49e7aaa66a2de04cb7f4f0998139:'+relative],cwd=ROOT))).convert('RGBA')
    source=Image.open(OUT/f'{name}-proposal-source.png').convert('RGBA')
    proposal=source.resize(old.size, Image.Resampling.NEAREST)
    meta=json.loads(Path(str(prefix)+'#anim.fanim').read_text())['anims']
    packed=Image.new('RGBA',old.size)
    # Register the proposed foot baseline to the original frame. Preserve empty cells.
    repairs=[]
    for tag,anim in meta.items():
        previous=None
        for number,frame in enumerate(anim['frames']):
            r=frame['data'];box=(r['x'],r['y'],r['x']+r['w'],r['y']+r['h'])
            a=old.crop(box);b=component(proposal.crop(box));ab=a.getbbox();bb=b.getbbox()
            if ab and bb:
                aligned=Image.new('RGBA',a.size)
                aligned.alpha_composite(b,(0,ab[3]-bb[3]))
                packed.alpha_composite(aligned,(r['x'],r['y']))
                previous=b
            elif ab and previous is not None:
                # A generator may omit a terminal fade cell. Reuse the approved
                # preceding pose, mechanically packed into the original bounds.
                aligned=Image.new('RGBA',a.size)
                fade=previous.crop(previous.getbbox()).resize((ab[2]-ab[0],ab[3]-ab[1]),Image.Resampling.NEAREST)
                aligned.alpha_composite(fade,(ab[0],ab[1]));packed.alpha_composite(aligned,(r['x'],r['y']))
                repairs.append({'pose':tag,'frame':number,'repair':'Reuse approved preceding hood at original fade bounds'})
    packed.save(OUT/f'{name}-candidate.png')
    sheet=Image.new('RGB',(1000,60+len(meta)*110),'#18232e');d=ImageDraw.Draw(sheet)
    d.text((12,12),f'{name.upper()} BODY REVIEW - original / approved simpler body / mirrored, 1x and 3x.',font=FONT,fill='white')
    for row,(tag,anim)in enumerate(meta.items()):
        y=50+row*110;d.text((10,y),tag,font=FONT,fill='#dbc58f')
        r=anim['frames'][len(anim['frames'])//2]['data'];box=(r['x'],r['y'],r['x']+r['w'],r['y']+r['h'])
        for col,(atlas,left,bg)in enumerate([(old,False,'#283a3c'),(packed,False,'#283a3c'),(packed,True,'#283a3c'),(packed,False,'#c8cdb3'),(packed,True,'#c8cdb3')]):
            x=120+col*172;d.rectangle((x,y,x+162,y+104),fill=bg);im=atlas.crop(box)
            if left:im=ImageOps.mirror(im)
            sheet.paste(im,(x+5,y+52),im);big=im.resize((72,88),Image.Resampling.NEAREST);sheet.paste(big,(x+60,y+10),big)
    sheet.save(OUT/f'{name}-comparison.png')
    frames=[];durations=[]
    for tag,anim in meta.items():
        for f in anim['frames']:
            canvas=Image.new('RGB',(600,185),'#17232c');dr=ImageDraw.Draw(canvas);dr.text((10,10),f'{name} / {tag}: original, candidate, mirrored — actual size + 3x',font=FONT,fill='white')
            r=f['data'];box=(r['x'],r['y'],r['x']+r['w'],r['y']+r['h'])
            for i,(atlas,flip)in enumerate([(old,False),(packed,False),(packed,True)]):
                im=atlas.crop(box)
                if flip:im=ImageOps.mirror(im)
                x=10+i*198;dr.rectangle((x,40,x+185,180),fill='#253b38'if i<2 else '#c8cdb3')
                canvas.paste(im,(x+5,120),im);big=im.resize((108,132),Image.Resampling.NEAREST);canvas.paste(big,(x+65,45),big)
            frames.append(canvas);durations.append(max(20,round(f['duration']*1000)))
    frames[0].save(OUT/f'{name}-animation.gif',save_all=True,append_images=frames[1:],duration=durations,loop=0,disposal=2)
    report[name]={'runtimeUnchanged':True,'sourceSha256':hashlib.sha256((OUT/f'{name}-proposal-source.png').read_bytes()).hexdigest(),'candidateSha256':hashlib.sha256((OUT/f'{name}-candidate.png').read_bytes()).hexdigest(),'canvas':[36,44],'frames':sum(len(a['frames'])for a in meta.values()),'frameRepairs':repairs,'approval':'pending'}
(OUT/'review.json').write_text(json.dumps(report,indent=2)+'\n')
if (OUT/'approval.json').exists():
    approved=json.loads((OUT/'approval.json').read_text())
    for name,row in report.items():
        if row['candidateSha256'] in [approved.get(name,{}).get('candidateSha256'),approved.get(name,{}).get('integratedSha256')]:row['approval']=approved[name]['approval']
    (OUT/'review.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report))
