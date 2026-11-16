"""Extract complete, approved Basalt drawings; never compose sliced anatomy.

Only frozen raster sources, alpha cleanup, palette matching, scale, mirror and packing.
Image generation is not deterministic. Running this extraction is reproducible.
"""
from pathlib import Path
from collections import defaultdict
import hashlib,json,math,shutil
import numpy as np
from PIL import Image,ImageOps,ImageDraw

ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/'docs/basalt-art-review'
CW,CH=160,80
FOOT=(80,60)
N=Image.Resampling.NEAREST

def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def save(p,v):p.write_text(json.dumps(v,indent=2)+'\n',encoding='utf-8')

def components(im):
    """8-connected alpha runs, so a wide whip is not cut at an artificial grid edge."""
    mask=np.asarray(im.getchannel('A'))>=150
    parent=[];runs=[];prev=[]
    def root(i):
        while parent[i]!=i:parent[i]=parent[parent[i]];i=parent[i]
        return i
    for y,row in enumerate(mask):
        edges=np.diff(np.r_[False,row,False].astype(np.int8))
        current=[]
        for a,b in zip(np.where(edges==1)[0],np.where(edges==-1)[0]):
            i=len(parent);parent.append(i);runs.append((y,int(a),int(b),i))
            for pa,pb,pi in prev:
                if pb>=a and pa<=b:parent[root(i)]=root(pi)
            current.append((a,b,i))
        prev=current
    groups=defaultdict(list)
    for y,a,b,i in runs:groups[root(i)].append((y,a,b))
    result=[]
    for rows in groups.values():
        area=sum(b-a for _,a,b in rows)
        if area<1000:continue
        box=(min(a for _,a,_ in rows),min(y for y,_,_ in rows),max(b for _,_,b in rows),max(y for y,_,_ in rows)+1)
        alpha=Image.new('L',im.size)
        d=ImageDraw.Draw(alpha)
        for y,a,b in rows:d.line((a,y,b-1,y),fill=255)
        sprite=im.copy();sprite.putalpha(alpha)
        result.append((box,sprite.crop(box)))
    return result

def grouped(file,rows,cols):
    im=Image.open(OUT/file).convert('RGBA');parts=components(im)
    assert len(parts)==rows*cols,(file,len(parts),[r[0] for r in parts])
    # Group by row centre; sprites may cross their nominal grid cell.
    parts.sort(key=lambda q:(q[0][1]+q[0][3])/2)
    ordered=[]
    for r in range(rows):ordered.extend(sorted(parts[r*cols:(r+1)*cols],key=lambda q:(q[0][0]+q[0][2])/2))
    return ordered

def main():
    approval=json.loads((OUT/'approvals.json').read_text())
    assert approval['body']['approved'] and sha(OUT/approval['body']['source'])==approval['body']['sha256']
    sources={n:sha(OUT/n) for n in ['body-source-v2.png','animation-locomotion-source.png','animation-actions-source.png','animation-whip-source.png','animation-ultimate-death-source.png']}
    frozen=OUT/'body-frozen-sources.json'
    if frozen.exists():assert json.loads(frozen.read_text())==sources,'Frozen source changed'
    save(frozen,sources)
    groups={
        'locomotion':grouped('animation-locomotion-source.png',2,6),
        'actions':grouped('animation-actions-source.png',3,4),
        'whip':grouped('animation-whip-source.png',3,4),
        'ultimate':grouped('animation-ultimate-death-source.png',4,4),
    }
    # One palette from the approved concept, shared by every action and facing.
    concept=Image.new('RGBA',(CW,CH));concept.alpha_composite(Image.open(OUT/'body-proposals.png').convert('RGBA').crop((128,0,256,80)),(16,0))
    colours=sorted({p[:3] for p in concept.get_flattened_data() if p[3]})
    pal=Image.new('P',(1,1));pal.putpalette([v for c in (colours+[colours[0]]*(256-len(colours))) for v in c])
    # Native pixels per source pixel. Fixed per sheet/locomotion family, never
    # stretched to make a crouching/fallen figure as tall as a standing figure.
    scales={'locomotion':38/317,'actions':38/273,'whip':38/273,'ultimate':38/256}
    frame_metrics={};poses={};anims={}
    spec={
        'idle':('locomotion',[0,1,2,3],[18]*4),
        'run':('locomotion',list(range(6,12)),[8]*6),
        'attack':('actions',[0,1,2,3],[5]*4),
        'raise':('actions',[4,5,6,7],[5]*4),
        'punch':('actions',[8,9,10,11],[5]*4),
        'throw':('whip',[0,1,2,3],[6]*4),
        'snare':('whip',[4,5,6,7],[5]*4),
        'swing':('whip',[8,9,10,11],[6]*4),
        'jump':('ultimate',[4,5,6,7],[6]*4),
        'air':('ultimate',[5,6],[12]*2),
        'charge':('ultimate',[0,1,2,3],[12]*4),
        'ult_punch':('ultimate',[4,5,6,7],[5]*4),
        'hit':('ultimate',[8,9],[4,4]),
        'dead':('ultimate',[10,11,12,13,14,15],[8]*6),
    }
    for tag,(group,ids,ticks) in spec.items():
        frames=[];metrics=[]
        for i in ids:
            box,f=groups[group][i];sc=scales[group]
            if group=='locomotion' and i>=6:sc=38/302
            pixels=np.array(f);a=pixels[:,:,3]>150
            red=(pixels[:,:,0]>90)&(pixels[:,:,0]>pixels[:,:,1]*1.4)&(pixels[:,:,0]>pixels[:,:,2]*1.15)&a
            ys,xs=np.where(red)
            # Foot midpoint from the two red boots, excluding the coiled whip.
            bottom=int(ys.max())+3
            boots=red & (np.indices(red.shape)[0]>=max(0,bottom-24))
            by,bx=np.where(boots)
            anchor_x=(int(bx.min())+int(bx.max()))/2 if len(bx) else f.width/2
            if tag=='dead':anchor_x=f.width/2;bottom=f.height
            size=(max(1,round(f.width*sc)),max(1,round(f.height*sc)))
            f=f.resize(size,Image.Resampling.BOX);alpha=f.getchannel('A').point(lambda x:255 if x>=128 else 0)
            f=f.convert('RGB').quantize(palette=pal,dither=Image.Dither.NONE).convert('RGBA');f.putalpha(alpha)
            tile=Image.new('RGBA',(CW,CH));x=FOOT[0]-round(anchor_x*sc);y=FOOT[1]-round(bottom*sc)
            assert x>=2 and y>=2 and x+f.width<CW-1 and y+f.height<CH-1,(tag,i,size,x,y)
            tile.alpha_composite(f,(x,y));frames.append(tile)
            metrics.append({'source':group,'index':i,'source_rect':box,'bbox':tile.getbbox(),'sha256':hashlib.sha256(tile.tobytes()).hexdigest()})
        # The first native idle is the exact approved concept pixels.
        if tag=='idle':frames[0]=concept;metrics[0]['sha256']=hashlib.sha256(concept.tobytes()).hexdigest();metrics[0]['bbox']=concept.getbbox();metrics[0]['source']='body-proposals.png';metrics[0]['source_rect']=[128,0,256,80]
        poses[tag]=frames;frame_metrics[tag]=metrics
        for left in [False,True]:
            seq=[]
            for f,t in zip(frames,ticks):
                if left:
                    m=Image.new('RGBA',(CW,CH));m.alpha_composite(ImageOps.mirror(f),(1,0));f=m
                seq.append((f,t))
            anims[tag+('_l' if left else '')]=seq
    aliases={'walk':'run','skill1':'raise','skill':'raise','skill2':'throw','skill3':'snare','ult':'charge','air_punch':'ult_punch','whip':'attack','walk_a':'run','walk_b':'run'}
    for alias,tag in aliases.items():
        for suffix in ['', '_l']:anims[alias+suffix]=anims[tag+suffix]
    atlas=Image.new('RGBA',(1024,2048));x=y=0;unique={};meta={}
    for tag,seq in anims.items():
        output=[]
        for f,t in seq:
            key=f.tobytes()
            if key not in unique:
                if x+CW>1024:x=0;y+=CH+2
                assert y+CH<=2048
                unique[key]={'x':x,'y':y,'w':CW,'h':CH};atlas.alpha_composite(f,(x,y));x+=CW+2
            output.append({'duration':t/60,'data':unique[key]})
        meta[tag]={'frames':output}
    height=2**math.ceil(math.log2(y+CH));atlas=atlas.crop((0,0,1024,height))
    atlas.save(OUT/'body.png');save(OUT/'body.fanim',{'anims':meta});save(OUT/'body-frame-metrics.json',frame_metrics)
    target=ROOT/'mods/tfm2_custom/champions/tfm2_custom_basalt'
    for source,suffix in [('body.png','#sheet.png'),('body.fanim','#anim.fanim')]:shutil.copyfile(OUT/source,str(target)+suffix)
    presentation={'revision':'basalt-art-5','rules_revision':json.loads((ROOT/'rules/basalt.json').read_text())['revision'],'body_approved':True,'effects_approved':approval['effects']['approved'],'body_file':'../docs/basalt-art-review/body.png','canvas':[CW,CH],'foot_anchor':FOOT,'body_anims':meta,'source_hashes':sources,'atlas_sha256':sha(OUT/'body.png'),'decoded_body_bytes':atlas.width*atlas.height*4,'pose_frames':{tag:len(fs) for tag,fs in poses.items()},'generation_deterministic':False,'packing_deterministic':True}
    save(OUT/'body-presentation.json',presentation)
    (ROOT/'editor/basalt-body.js').write_text('window.BASALT_BODY='+json.dumps(presentation,separators=(',',':'))+';\n',encoding='utf-8')
    # Actual packed frames at 1x/3x, on both light and dark backgrounds.
    for tag,fs in poses.items():
        frames=[]
        for i,f in enumerate(fs):
            out=Image.new('RGB',(960,300));d=ImageDraw.Draw(out)
            for col,bg in enumerate(['#203b44','#b9c5a9']):
                xx=col*480;d.rectangle((xx,0,xx+479,299),fill=bg);ink='#fff0d2' if not col else '#142a33';d.text((xx+10,10),f'{tag} | frame {i+1}/{len(fs)} | 1x and 3x',fill=ink)
                for left in [False,True]:
                    key=tag+('_l' if left else '');r=meta[key]['frames'][i]['data'];s=atlas.crop((r['x'],r['y'],r['x']+CW,r['y']+CH))
                    out.paste(s,(xx+8+(128 if left else 0),50),s)
                    tight=s.crop(s.getbbox());big=tight.resize((tight.width*3,tight.height*3),N);out.paste(big,(xx+24+(230 if left else 0),280-big.height),big)
            frames.append(out)
        frames[0].save(OUT/(tag+'.gif'),save_all=True,append_images=frames[1:],duration=[round(t*1000/60) for t in spec[tag][2]],loop=0,disposal=2)
    contact=Image.new('RGB',(1040,90*len(poses)),'#203b44');d=ImageDraw.Draw(contact)
    for row,(tag,fs) in enumerate(poses.items()):
        d.text((2,row*90+4),tag,fill='#ffda75')
        for i,f in enumerate(fs):contact.paste(f,(60+i*160,row*90),f)
    contact.save(OUT/'all-body-frames.png')
    print(json.dumps({'poses':len(poses),'unique_frames':len(unique),'tags':len(meta),'atlas':atlas.size,'decoded_bytes':presentation['decoded_body_bytes'],'body_integrated':True,'effects_integrated':approval['effects']['approved']}))

if __name__=='__main__':main()
