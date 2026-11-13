"""Pack frozen, approved whole-body drawings. Effects remain review-only until approved.
Uses built-in imagegen sources; extraction and packing are reproducible, generation is not.
"""
from pathlib import Path
import hashlib,json,math,shutil,sys
from collections import deque,Counter
from PIL import Image,ImageOps,ImageDraw
ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/'docs/kicker-art-review'
ART=json.loads((ROOT/'rules/kicker-art.json').read_text())
APP=json.loads((OUT/'approvals.json').read_text())
N=Image.Resampling.NEAREST
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def savejson(p,v):p.write_text(json.dumps(v,indent=2)+'\n',encoding='utf8')
def connected(im):
    im=im.copy();im.putalpha(im.getchannel('A').point(lambda a:255 if a>=220 else 0))
    mask=im.getchannel('A');pix=mask.load();seen=set();best=[]
    for y in range(im.height):
        for x in range(im.width):
            if not pix[x,y] or (x,y) in seen:continue
            group=[];todo=[(x,y)];seen.add((x,y))
            while todo:
                q=todo.pop();group.append(q)
                for dx,dy in [(1,0),(-1,0),(0,1),(0,-1),(1,1),(-1,1),(1,-1),(-1,-1)]:
                    a,b=q[0]+dx,q[1]+dy
                    if 0<=a<im.width and 0<=b<im.height and pix[a,b] and (a,b) not in seen:
                        seen.add((a,b));todo.append((a,b))
            if len(group)>len(best):best=group
    alpha=Image.new('L',im.size)
    for q in best:alpha.putpixel(q,255)
    im.putalpha(alpha)
    return im.crop(im.getbbox())
def cells(name,cols,rows,body=False):
    im=Image.open(OUT/name).convert('RGBA');out=[]
    for i in range(cols*rows):
        f=im.crop((i%cols*im.width//cols,i//cols*im.height//rows,(i%cols+1)*im.width//cols,(i//cols+1)*im.height//rows))
        if body:f=connected(f)
        else:
            f.putalpha(f.getchannel('A').point(lambda a:255 if a>=180 else 0));f=f.crop(f.getbbox())
        assert f.getbbox(),(name,i)
        out.append(f)
    return out
def pack(stem,anims,width=1024):
    x=y=row=0;unique={};metadata={}
    for tag,seq in anims.items():
        output=[]
        for im,ticks in seq:
            key=(im.size,im.tobytes())
            if key not in unique:
                if x+im.width>width:x=0;y+=row+2;row=0
                assert y+im.height<=2048,(stem,tag)
                rect=dict(x=x,y=y,w=im.width,h=im.height);unique[key]=(im,rect)
                x+=im.width+2;row=max(row,im.height)
            output.append(dict(duration=ticks/60,data=unique[key][1]))
        metadata[tag]={'frames':output}
    height=2**math.ceil(math.log2(max(2,y+row+2)))
    atlas=Image.new('RGBA',(width,height))
    for im,r in unique.values():atlas.alpha_composite(im,(r['x'],r['y']))
    atlas.save(OUT/(stem+'.png'));savejson(OUT/(stem+'.fanim'),{'anims':metadata})
    return atlas,metadata
def bodies():
    approved=Image.open(OUT/'concepts.png').convert('RGBA').crop((64,0,128,56))
    # Freeze one palette from the approved idle: no separate idle/run lighting.
    pixels=approved.get_flattened_data() if hasattr(approved,'get_flattened_data') else approved.getdata()
    colours=sorted({c[:3] for c in pixels if c[3]})
    palette=Image.new('P',(1,1));palette.putpalette([v for c in (colours+[(0,0,0)]*(256-len(colours))) for v in c])
    raw={}
    for name,spec in ART['source_groups'].items():
        fs=cells(name,spec['cols'],spec['rows'],True);scale=spec['height']/fs[spec['reference']].height
        raw[name]=[]
        for f in fs:
            f=f.resize((max(1,round(f.width*scale)),max(1,round(f.height*scale))),Image.Resampling.BOX)
            a=f.getchannel('A').point(lambda a:255 if a>=128 else 0)
            f=f.convert('RGB').quantize(palette=palette,dither=Image.Dither.NONE).convert('RGBA');f.putalpha(a)
            raw[name].append(f)
    poses={};anims={};metrics={}
    for tag,spec in ART['poses'].items():
        frames=[]
        for n in spec['frames']:
            f=raw[spec['source']][n];cw,ch=ART['canvas'];fy=ART['foot_anchor'][1]
            assert f.width<=cw-4 and f.height<=fy-2,(tag,n,f.size)
            im=Image.new('RGBA',(cw,ch));im.alpha_composite(f,((cw-f.width)//2,fy-f.height));frames.append(im)
        if tag=='idle':frames[0]=approved
        poses[tag]=frames
        metrics[tag]=[{'bbox':f.getbbox(),'sha256':hashlib.sha256(f.tobytes()).hexdigest()} for f in frames]
        for left in [False,True]:
            anims[tag+('_l' if left else '')]=[(ImageOps.mirror(f) if left else f,t) for f,t in zip(frames,spec['ticks'])]
    for alias,tag in ART['aliases'].items():
        for suffix in ['', '_l']:anims[alias+suffix]=anims[tag+suffix]
    savejson(OUT/'body-frame-metrics.json',metrics)
    return poses,pack('body',anims)
def effects():
    mats=cells('effects-source.png',4,4);anims={}
    for name,(index,w,h,ticks,z) in ART['effects'].items():
        im=mats[index].resize((w,h),Image.Resampling.BOX);im.putalpha(im.getchannel('A').point(lambda a:255 if a>=100 else 0))
        im=im.quantize(colors=12,method=Image.Quantize.FASTOCTREE,dither=Image.Dither.NONE).convert('RGBA')
        if ticks==1:seq=[(im,1)]
        else:
            seq=[]
            for i in range(4):
                fac=[.65,.9,1,.8][i];s=im.resize((max(1,round(w*fac)),max(1,round(h*fac))),N)
                f=Image.new('RGBA',(w,h));f.alpha_composite(s,((w-s.width)//2,(h-s.height)//2));seq.append((f,ticks//4+(i<ticks%4)))
        anims[name]=seq
        if name in ['rock','kick_trail','skyfall','dash']:
            for n,heading in enumerate(ART['headings']):
                rotated=[]
                for f,t in seq:
                    # Material skyfall points SE; other materials point E.
                    angle=(-n*45)+(45 if name=='skyfall' else 0)
                    s=f.rotate(angle,N,expand=True);side=max(s.width,s.height)+4
                    canvas=Image.new('RGBA',(side,side));canvas.alpha_composite(s,((side-s.width)//2,(side-s.height)//2));rotated.append((canvas,t))
                anims[name+'_'+heading]=rotated
    for n in range(6):
        icon=anims['bruised'][0][0];im=Image.new('RGBA',(20,20));im.alpha_composite(icon,(4,0));d=ImageDraw.Draw(im)
        for pip in range(n):d.rectangle((pip*4,15,pip*4+2,17),fill='#ff7055')
        anims['bruised'+str(n)]=[(im,1)]
    # Same approved material pixels, padded around the world origin for buffs.
    # Native and Lab use these exact attachment tags; no separate cursor offsets.
    for tag,(dx,dy) in ART['buff_offsets'].items():
        src=anims[tag][0][0];w=src.width+abs(dx)*2;h=src.height+abs(dy)*2
        canvas=Image.new('RGBA',(w,h));canvas.alpha_composite(src,((w-src.width)//2+dx,(h-src.height)//2+dy))
        anims[tag+'_attached']=[(canvas,1)]
    return pack('effects',anims,512)
def reviews(poses,atlas,metadata):
    rows=list(poses);board=Image.new('RGB',(1120,len(rows)*190));d=ImageDraw.Draw(board)
    for row,tag in enumerate(rows):
        for col,bg in enumerate(['#203747','#bec9b0']):
            x=col*560;y=row*190;d.rectangle((x,y,x+559,y+189),fill=bg);d.text((x+8,y+5),tag+' | whole body, both facings | 1x + 3x',fill='#fff5dc' if not col else '#203747')
            f=poses[tag][min(2,len(poses[tag])-1)]
            for j,s in enumerate([f,ImageOps.mirror(f)]):
                board.paste(s,(x+8+j*65,y+100),s);big=s.resize((s.width*3,s.height*3),N);board.paste(big,(x+150+j*195,y+20),big)
    board.save(OUT/'body-comparison.png')
    for tag,fs in poses.items():
        seq=[]
        for f in fs:
            b=Image.new('RGB',(560,210));d=ImageDraw.Draw(b)
            for col,bg in enumerate(['#203747','#bec9b0']):
                x=col*280;d.rectangle((x,0,x+279,209),fill=bg);d.text((x+4,4),tag+' | 1x, 3x',fill='#fff5dc' if not col else '#203747')
                for left,xx in [(False,8),(True,72)]:
                    s=ImageOps.mirror(f) if left else f;b.paste(s,(x+xx,50),s)
                big=f.resize((192,168),N);b.paste(big,(x+85,34),big)
            seq.append(b)
        seq[0].save(OUT/(tag+'.gif'),save_all=True,append_images=seq[1:],duration=[max(20,round(t*1000/60)) for t in ART['poses'][tag]['ticks']],loop=0,disposal=2)
    names=list(ART['effects']);board=Image.new('RGB',(1120,math.ceil(len(names)/4)*190));d=ImageDraw.Draw(board)
    for i,name in enumerate(names):
        x=i%4*280;y=i//4*190;bg='#203747' if i%2==0 else '#bec9b0';d.rectangle((x,y,x+279,y+189),fill=bg);d.text((x+8,y+5),name+' | 1x + 3x',fill='#fff5dc' if i%2==0 else '#203747')
        r=metadata[name]['frames'][min(1,len(metadata[name]['frames'])-1)]['data'];f=atlas.crop((r['x'],r['y'],r['x']+r['w'],r['y']+r['h']))
        board.paste(f,(x+14,y+85),f);s=f.resize((f.width*3,f.height*3),N);board.paste(s,(x+85,y+30),s)
    board.save(OUT/'effects-comparison.png')
def main():
    assert APP['body']['approved']
    assert sha(OUT/APP['body']['source'])==APP['body']['source_sha256']
    frozen_path=OUT/'frozen-sources.json'
    sources={name:sha(OUT/name) for name in ['body-source-v2.png',*ART['source_groups'],'effects-source.png']}
    if frozen_path.exists():
        for name,digest in json.loads(frozen_path.read_text()).items():
            assert sources.get(name)==digest,('Frozen source changed',name)
    savejson(frozen_path,sources)
    poses,(bs,bm)=bodies();fs,fm=effects();reviews(poses,fs,fm)
    fxok=APP['effects'].get('approved',False)
    if fxok:assert sha(OUT/APP['effects']['source'])==APP['effects']['source_sha256']
    for stem,ok,target in [('body',True,'champions/tfm2_custom_kicker'),('effects',fxok,'vfx/kicker')]:
        if ok:
            for suffix,ext in [('sheet','.png'),('anim','.fanim')]:shutil.copyfile(OUT/(stem+ext),ROOT/'mods/tfm2_custom'/(target+'#'+suffix+ext))
    manifest={**ART,'body_approved':True,'effects_approved':fxok,'sources_sha256':sources,
        'body_anims':bm,'effect_anims':fm,'decoded_bytes':bs.width*bs.height*4+fs.width*fs.height*4,
        'atlas_sha256':{n:sha(OUT/(n+'.png')) for n in ['body','effects']}}
    assert manifest['decoded_bytes']<=32*1024*1024
    savejson(OUT/'presentation.json',manifest)
    (ROOT/'editor/kicker-art.js').write_text('window.KICKER_ART='+json.dumps(manifest,separators=(',',':'))+';\n')
    native=['//! Generated from rules/kicker-art.json by build-kicker-art.py.',
        'pub const HEADINGS: [&str; 8] = ['+', '.join(json.dumps(h) for h in ART['headings'])+'];',
        'pub const MAX_RENEW_TICKS: u64 = '+str(ART['native']['maxRenewTicks'])+';']
    native_path=ROOT/'native/tfm2_custom_ai/src/kicker_art_data.rs';native_text='\n'.join(native)+'\n'
    if not native_path.exists() or native_path.read_text()!=native_text:native_path.write_text(native_text)
    print(json.dumps({'body_approved':True,'effects_approved':fxok,'poses':len(poses),'body_tags':len(bm),'effects':len(fm),'decoded_bytes':manifest['decoded_bytes']}))
if __name__=='__main__':main()

