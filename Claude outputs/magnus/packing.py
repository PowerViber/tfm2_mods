"""Deterministic centred, per-animation crops and exact-frame sharing."""
import json,math
from pathlib import Path
from PIL import Image
def atlas(path,entries,cols=8):
    unique=[];lookup={};tags={}
    for tag,frames in entries.items():
        w=max(im.width for im,_ in frames);h=max(im.height for im,_ in frames)
        canvases=[]
        for im,duration in frames:
            c=Image.new('RGBA',(w,h));c.alpha_composite(im,((w-im.width)//2,(h-im.height)//2));canvases.append((c,duration))
        boxes=[im.getbbox()for im,_ in canvases if im.getbbox()]
        cw=min(w,2*math.ceil(max([max(w/2-b[0],b[2]-w/2)for b in boxes]or[1])+1))
        ch=min(h,2*math.ceil(max([max(h/2-b[1],b[3]-h/2)for b in boxes]or[1])+1))
        tags[tag]=[]
        for im,duration in canvases:
            im=im.crop(((w-cw)//2,(h-ch)//2,(w+cw)//2,(h+ch)//2));key=(im.size,im.tobytes())
            if key not in lookup:lookup[key]=len(unique);unique.append(im)
            tags[tag].append((lookup[key],duration))
    order=sorted(range(len(unique)),key=lambda n:(-unique[n].height,-unique[n].width,n))
    def layout(limit):
        x=y=row=used=0;positions={}
        for n in order:
            im=unique[n]
            if x+im.width>limit:x=0;y+=row;row=0
            positions[n]=(x,y);x+=im.width;row=max(row,im.height);used=max(used,x)
        return positions,used,y+row
    minimum=max(im.width for im in unique);best=None
    for limit in sorted(set([minimum,2048]+list(range(max(64,minimum),2049,16)))):
        pos,w,h=layout(limit)
        if max(w,h)<=2048 and(best is None or w*h<best[1]*best[2]):best=(pos,w,h)
    assert best is not None,(str(path),'atlas overflow',len(unique))
    pos,w,h=best;sheet=Image.new('RGBA',(w,h))
    for n,im in enumerate(unique):sheet.alpha_composite(im,pos[n])
    anims={tag:{'frames':[{'duration':round(float(duration),7),'data':{'x':pos[n][0],'y':pos[n][1],'w':unique[n].width,'h':unique[n].height}}for n,duration in frames]}for tag,frames in tags.items()}
    path=Path(path);path.parent.mkdir(parents=True,exist_ok=True);sheet.save(str(path)+'#sheet.png',optimize=True)
    Path(str(path)+'#anim.fanim').write_text(json.dumps({'anims':anims},separators=(',',':'))+'\n',encoding='utf-8')
    return anims
