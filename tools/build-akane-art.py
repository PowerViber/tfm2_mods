"""Reproducibly extract frozen imagegen whole-body drawings and VFX materials.

No anatomy slicing. Approval gates native copies; review outputs always build.
"""
from pathlib import Path
import hashlib,json,math,shutil,runpy
from PIL import Image,ImageDraw,ImageOps

ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/'docs/akane-art-review'
ART=json.loads((ROOT/'rules/akane-art.json').read_text())
APP=json.loads((OUT/'approvals.json').read_text())
N=Image.Resampling.NEAREST
connected=runpy.run_path(str(ROOT/'tools/prepare-akane-art.py'))['connected']

def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def cells(path,cols,rows,body=False):
    im=Image.open(path).convert('RGBA');out=[]
    for i in range(cols*rows):
        c=im.crop((i%cols*im.width//cols,i//cols*im.height//rows,(i%cols+1)*im.width//cols,(i//cols+1)*im.height//rows))
        if body:c=connected(c)
        else:c.putalpha(c.getchannel('A').point(lambda a:255 if a>=180 else 0))
        assert c.getbbox(),(path,i)
        out.append(c)
    return out

def lowres(im,size):
    a=im.resize(size,Image.Resampling.BOX);a.putalpha(a.getchannel('A').point(lambda a:255 if a>=80 else 0))
    return a

def pack(stem,anims,width=1024):
    x=y=row=0;unique={};meta={}
    for name,frames in anims.items():
        result=[]
        for im,ticks in frames:
            key=(im.size,im.tobytes())
            if key not in unique:
                if x+im.width>width:x=0;y+=row+2;row=0
                assert y+im.height<=2048,(stem,name)
                rect=dict(x=x,y=y,w=im.width,h=im.height);unique[key]=(im,rect);x+=im.width+2;row=max(row,im.height)
            result.append({'duration':ticks/60,'data':unique[key][1]})
        meta[name]={'frames':result}
    height=2**math.ceil(math.log2(max(2,y+row+2)));sheet=Image.new('RGBA',(width,height))
    for im,rect in unique.values():sheet.alpha_composite(im,(rect['x'],rect['y']))
    sheet.save(OUT/(stem+'.png'));(OUT/(stem+'.fanim')).write_text(json.dumps({'anims':meta},indent=2)+'\n')
    return sheet,meta

def bodies():
    raw={}
    for group,poses in ART['source_groups'].items():
        frames=cells(OUT/(group+'-source-v1.png'),8,3,True);b=frames[0].getbbox();scale=40/(b[3]-b[1])
        for row,pose in enumerate(poses):
            if not pose:continue
            raw[pose]=[]
            for f in frames[row*8:row*8+8]:
                c=f.crop(f.getbbox());raw[pose].append(lowres(c,(max(1,round(c.width*scale)),max(1,round(c.height*scale)))))
    # One shared palette for all poses prevents lighting/colour jumps.
    swatch=Image.new('RGB',(512,512),(26,22,29));x=y=0
    for frames in raw.values():
        for im in frames:
            if x+im.width>=512:x=0;y+=64
            swatch.paste(im,(x,y),im);x+=im.width+2
    palette=swatch.quantize(colors=24,method=Image.Quantize.MEDIANCUT,dither=Image.Dither.NONE)
    result={}
    for pose,frames in raw.items():
        w=max(48,math.ceil((max(f.width for f in frames)+6)/8)*8)
        h=max(56,math.ceil((max(f.height for f in frames)-16+2)/4)*8)
        # world origin stays frame centre; ground contact is always +20 px.
        result[pose]=[]
        for f in frames:
            a=f.getchannel('A');f=f.convert('RGB').quantize(palette=palette,dither=Image.Dither.NONE).convert('RGBA');f.putalpha(a)
            im=Image.new('RGBA',(w,h));im.alpha_composite(f,((w-f.width)//2,h//2+21-f.height));box=im.getbbox()
            assert box and box[0]>0 and box[1]>0 and box[2]<w and box[3]==h//2+21,(pose,box,w,h)
            result[pose].append(im)
    anims={}
    for pose,frames in result.items():
        for left in (False,True):anims[pose+('_l' if left else '')]=[(ImageOps.mirror(im) if left else im,t) for im,t in zip(frames,ART['pose_ticks'][pose])]
    # Additional tags select already approved whole drawings; no control changes.
    for alias,pose in ART['aliases'].items():
        total=14 if alias=='night_dash' else 12
        ts=[total//8+(i<total%8) for i in range(8)]
        for left in (False,True):anims[alias+('_l' if left else '')]=[(ImageOps.mirror(im) if left else im,t) for im,t in zip(result[pose],ts)]
    return result,pack('body',anims)

def effects():
    mats=cells(OUT/'effects-source-v1.png',4,3);anims={};bindings={}
    for name,(index,w,h,ticks,z,kind) in ART['effects'].items():
        mat=mats[index];mat=lowres(mat.crop(mat.getbbox()),(w,h)).quantize(colors=12,method=Image.Quantize.FASTOCTREE,dither=Image.Dither.NONE).convert('RGBA')
        frames=[]
        for i in range(4):
            # Whole material silhouettes expand/settle. Persistent visuals pulse;
            # their selected phase uses the original birth clock in native code.
            factor=([.85,1,.94,.88] if kind=='buff' or name=='orb' else [.65,.9,1,.8])[i]
            scaled=mat.resize((max(1,round(w*factor)),max(1,round(h*factor))),N)
            im=Image.new('RGBA',(w+8,h+8));im.alpha_composite(scaled,((im.width-scaled.width)//2,(im.height-scaled.height)//2))
            frames.append((im,ticks//4+(i<ticks%4)))
        anims[name]=frames;bindings[name]={'kind':kind,'z':z,'follow':kind=='buff','ticks':ticks,'geometry_pixels':[w,h]}
        if name=='orb':
            for i,(im,_) in enumerate(frames):anims['orb_p'+str(i)]=[(im,1)];bindings['orb_p'+str(i)]={'kind':'effect','z':z,'follow':False,'ticks':1}
        if name in ('kunai','dash_slash','night_slash'):
            for i,heading in enumerate(ART['headings']):
                side=max(w,h)+12;seq=[]
                for im,t in frames:
                    c=Image.new('RGBA',(side,side));c.alpha_composite(im,((side-im.width)//2,(side-im.height)//2));seq.append((c.rotate(-i*45,N,expand=False),t))
                tag=name+'_'+heading;anims[tag]=seq;bindings[tag]={'kind':'effect','z':z,'follow':False,'ticks':ticks}
                if name=='kunai':
                    tag='kunai_flight_'+heading;anims[tag]=[(seq[2][0],1)];bindings[tag]={'kind':'effect','z':z,'follow':False,'ticks':1}
    return pack('effects',anims,512),bindings

def reviews(body,fx,metadata):
    board=Image.new('RGB',(1120,840));d=ImageDraw.Draw(board)
    for row,(name,idx) in enumerate([('idle',0),('run',2),('attack',4),('skill1',4)]):
        im=body[name][idx]
        for col,bg in enumerate(['#203747','#bec9b0']):
            x=col*560;y=row*210;d.rectangle((x,y,x+559,y+209),fill=bg);ink='#ffd5da' if col==0 else '#213b3a';d.text((x+8,y+5),name+' | 1x + 3x | both facings',fill=ink)
            for j,f in enumerate([im,ImageOps.mirror(im)]):board.paste(f,(x+10+j*58,y+110),f);b=f.resize((f.width*3,f.height*3),N);board.paste(b,(x+130+j*155,y+25),b)
            world=json.loads((ROOT/'editor/game-reference/world.json').read_text())
            for j,id in enumerate(['champ-ninja','champ-knight','champ-archer']):
                s=world['sprites'][id];r=s['anims']['idle']['frames'][0]['data'];f=Image.open(ROOT/'editor/game-reference'/s['file']).crop((r['x'],r['y'],r['x']+r['w'],r['y']+r['h']));board.paste(f,(x+430+j*40,y+120),f)
    board.save(OUT/'body-comparison.png')
    board=Image.new('RGB',(1120,600));d=ImageDraw.Draw(board)
    for i,name in enumerate(ART['effects']):
        x=i%5*224;y=i//5*150;d.rectangle((x,y,x+223,y+149),fill='#203747' if i%2==0 else '#bcc7b0');d.text((x+8,y+4),name,fill='#e17993' if i%2==0 else '#421f30')
        r=metadata[name]['frames'][1]['data'];im=fx.crop((r['x'],r['y'],r['x']+r['w'],r['y']+r['h']));board.paste(im,(x+8,y+60),im);big=im.resize((im.width*3,im.height*3),N);board.paste(big,(x+76,y+20),big)
    board.save(OUT/'effects-comparison.png')
    for pose in ('idle','run','attack','skill1','skill2','finale','ult','hit','dead'):
        frames=[]
        for im in body[pose]:
            b=Image.new('RGB',(520,210),'#203747')
            for x,f in [(12,im),(75,ImageOps.mirror(im))]:b.paste(f,(x,90),f)
            for x,f in [(150,im),(325,ImageOps.mirror(im))]:f=f.resize((f.width*3,f.height*3),N);b.paste(f,(x,15),f)
            frames.append(b)
        frames[0].save(OUT/(pose+'.gif'),save_all=True,append_images=frames[1:],duration=[max(20,round(t/60*1000)) for t in ART['pose_ticks'][pose]],loop=0,disposal=2)

def main():
    frozen=json.loads((OUT/'frozen-sources.json').read_text())
    for name,digest in frozen.items():assert sha(OUT/name)==digest,('Frozen source changed',name)
    assert sha(OUT/'concept-source-v1.png')==APP['body']['source_sha256']
    if APP['effects'].get('approved'):assert sha(OUT/'effects-source-v1.png')==APP['effects']['source_sha256']
    body,(bs,bm)=bodies();(fs,fm),bindings=effects();reviews(body,fs,fm)
    body_ok=APP['body'].get('approved',False);effects_ok=APP['effects'].get('approved',False)
    for group,ok in [('body',body_ok),('effects',effects_ok)]:
        if ok:
            stem='champions/tfm2_custom_akane' if group=='body' else 'vfx/akane'
            for suffix,ext in [('sheet','.png'),('anim','.fanim')]:shutil.copyfile(OUT/(group+ext),ROOT/'mods/tfm2_custom'/(stem+'#'+suffix+ext))
    manifest={'version':1,'revision':ART['revision'],'review_only':not(body_ok and effects_ok),'body_approved':body_ok,'effects_approved':effects_ok,'complete_body':True,'units_per_pixel':1000,'body_canvas':[48,56],'world_anchor':[24,28],'foot_anchor':[24,48],'atlases':{'body':'body','effects':'effects'},'poses':list(body),'effects':bindings,'headings':ART['headings'],'source_sha256':{p.name:sha(p) for p in sorted(OUT.glob('*source-v1.png'))},'atlas_sha256':{n:sha(OUT/(n+'.png')) for n in ['body','effects']},'decoded_bytes':bs.width*bs.height*4+fs.width*fs.height*4,'phase_clock':'orb phase = (tick - birth) / 15 modulo 4; all other animations use committed start, not renewal'}
    assert manifest['decoded_bytes']<=32*1024*1024
    for name in ['presentation.json','art-manifest.json']:(OUT/name).write_text(json.dumps(manifest,indent=2)+'\n')
    print(json.dumps({'poses':len(body),'body_frames':len(bm)*8,'effect_tags':len(fm),'decoded_bytes':manifest['decoded_bytes'],'body_integrated':body_ok,'effects_integrated':effects_ok}))
if __name__=='__main__':main()

# Preserve the approved Round 165 overlays and compact legacy layout.
_r165=Path(__file__).resolve().parents[1]/'docs/round-165-art-review/approval.json'
if _r165.exists() and json.loads(_r165.read_text())['effects'].get('approved'):
    import subprocess
    subprocess.run(['rtk','proxy','python',str(Path(__file__).with_name('build-round-165-art.py')),'--integrate'],check=True)
