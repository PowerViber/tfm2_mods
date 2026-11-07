"""Mechanically register complete imagegen drawings to the frozen world anchors.

Does not rig/slice body anatomy. Source grids contain authored complete poses.
Default writes review staging only. --integrate replaces the four body assets.
"""
from __future__ import annotations
import argparse, hashlib, json, math, statistics
from pathlib import Path
from PIL import Image, ImageOps

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / 'docs/ranked-body-redesign'
POSES = ['idle', 'run', 'attack', 'skill1', 'skill2', 'ult', 'hit', 'dead']


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def write(path, obj):
    Path(path).write_text(json.dumps(obj, indent=2, ensure_ascii=False)+'\n', encoding='utf8')


def grid(path, rows):
    im = Image.open(path).convert('RGBA')
    im.putalpha(im.getchannel('A').point(lambda a: 255 if a >= 180 else 0))
    result = []
    for row in range(rows):
        frames = []
        for col in range(8):
            cell = im.crop((round(col*im.width/8), round(row*im.height/rows),
                            round((col+1)*im.width/8), round((row+1)*im.height/rows)))
            bounds = cell.getbbox()
            if not bounds:
                raise ValueError(f'Empty source cell {path} {row}/{col}')
            # Raw extraction is explicit; never repair a clipped body by painting.
            frames.append((cell, bounds))
        result.append(frames)
    return result


def registered(cell, bounds, scale, old, offset, fallen=False):
    x0,y0,x1,y1 = bounds
    crop = cell.crop(bounds)
    if fallen:
        pivot = min(crop.width*.2, (y1-y0)*.45)
    else:
        head = crop.getchannel('A').crop((0,0,crop.width,max(1,round(crop.height*.22)))).getbbox()
        pivot = (head[0]+head[2])/2 if head else crop.width/2
    crop = crop.resize((max(1,round(crop.width*scale)), max(1,round(crop.height*scale))),Image.Resampling.NEAREST)
    pivot *= scale
    left_extent = math.ceil(pivot)+2
    right_extent = math.ceil(crop.width-pivot)+2
    w = max(old['w'], 2*max(left_extent,right_extent))
    # Preserve centre-to-foot vector even when the canvas needs padding.
    h = max(old['h'], 2*math.ceil(crop.height-offset+2), 2*math.ceil(offset+2))
    w,h = math.ceil(w/2)*2,math.ceil(h/2)*2
    dest = Image.new('RGBA',(w,h))
    x,y = round(w/2-pivot),round(h/2+offset-crop.height)
    if x < 0 or y < 0 or x+crop.width>w or y+crop.height>h:
        raise ValueError('Registration would clip a complete body')
    dest.alpha_composite(crop,(x,y))
    return dest


def pack(images, metadata):
    # Deterministic shelf packing with exact-pixel deduplication.
    unique, refs, locations = [], {}, {}
    for key, im in images.items():
        signature = (im.size,im.tobytes())
        if signature not in refs:
            refs[signature]=len(unique); unique.append(im)
        locations[key]=refs[signature]
    minimum=math.ceil((max(im.width for im in unique)+1)/32)*32
    width = minimum if all(im.getbbox() is None for im in unique) else max(512,minimum)
    if width>2048:raise ValueError('Frame exceeds atlas width limit')
    while True:
        placements=[];x=y=row_h=0
        for im in unique:
            if x+im.width+1 > width:x=0;y+=row_h+1;row_h=0
            placements.append((x,y));x+=im.width+1;row_h=max(row_h,im.height)
        height=y+row_h
        if height<=2048:break
        if width>=2048:raise ValueError('Atlas exceeds 2048')
        width=min(2048,width*2)
    atlas=Image.new('RGBA',(width,height))
    for im,(x,y) in zip(unique,placements):atlas.alpha_composite(im,(x,y))
    meta=json.loads(json.dumps(metadata))
    for pose,anim in meta['anims'].items():
        for i,frame in enumerate(anim['frames']):
            im=images[(pose,i)];x,y=placements[locations[(pose,i)]]
            frame['data']={'x':x,'y':y,'w':im.width,'h':im.height}
    return atlas,meta,len(unique)


def main(integrate=False):
    manifest=json.loads((ROOT/'rules/ranked-body-art.json').read_text(encoding='utf8'))
    assert manifest['approval']['status']=='approved','Appearance approval required'
    for source,sha in manifest['approvedSources'].items():
        assert digest(ROOT/source)==sha,'Approved source was modified'
    baseline=json.loads((OUT/'baseline-contract.json').read_text(encoding='utf8'))
    staging=OUT/'staged';staging.mkdir(exist_ok=True)
    report={'scope':'Complete base bodies; rank overlays gated on safe host pose alignment',
            'integrated':integrate,'characters':{},'nativeRankLayers':'visible-base-fallback'}
    for c in manifest['characters']:
        key=c['key'];old=baseline['characters'][key]['body'];meta=old['anims']
        first=OUT/'sources/proof'/f'{key}-idle-run-attack.png'
        rest=OUT/'sources/complete'/f'{key}-skills-hit-dead.png'
        rows=grid(first,3)+grid(rest,5)
        scales=[c['visibleHeight']/statistics.median(b[3]-b[1] for _,b in rows[0]),
                c['visibleHeight']/statistics.median(b[3]-b[1] for _,b in rows[3])]
        base_h=old['anims']['idle']['frames'][0]['data']['h']
        offset=c['feet'][1]-c['frame'][1]/2
        images={};all_drawings={};source_counts={}
        for p,row in zip(POSES,rows):
            scale=scales[0 if POSES.index(p)<3 else 1]
            contract=meta[p]['frames']
            for i,(cell,bounds) in enumerate(row):
                old_frame=contract[min(i,len(contract)-1)]['data']
                all_drawings[(p,i)]=registered(cell,bounds,scale,old_frame,offset,p=='dead' and i>=4)
            # Resample authored poses, preserving the complete original timing list.
            for i,frame in enumerate(contract):
                source_i=round(i*7/max(1,len(contract)-1))
                images[(p,i)]=all_drawings[(p,source_i)]
            source_counts[p]=len({im.tobytes() for (pose,_),im in images.items() if pose==p})
        atlas,packed,count=pack(images,{'anims':meta})
        stem=staging/key
        atlas.save(str(stem)+'#sheet.png')
        write(str(stem)+'#anim.fanim',packed)
        # The reusable complete-pose reference is a regular grid, not a body rig.
        mw=max(im.width for im in all_drawings.values());mh=max(im.height for im in all_drawings.values())
        reference=Image.new('RGBA',(mw*8,mh*8))
        for (p,i),im in all_drawings.items():
            reference.alpha_composite(im,(i*mw+(mw-im.width)//2,POSES.index(p)*mh+(mh-im.height)//2))
        reference.save(staging/(key+'-master-poses.png'))
        for facing in ['right','left']:
            frames=[];dur=[]
            for p in POSES:
                for i,f in enumerate(meta[p]['frames']):
                    im=images[(p,i)]
                    if facing=='left':im=ImageOps.mirror(im)
                    bg=Image.new('RGBA',(mw,mh),(20,35,44,255));bg.alpha_composite(im,((mw-im.width)//2,(mh-im.height)//2))
                    frames.append(bg.convert('RGB').resize((mw*3,mh*3),Image.Resampling.NEAREST));dur.append(round(f['duration']*1000))
            frames[0].save(staging/(key+'-'+facing+'.gif'),save_all=True,append_images=frames[1:],duration=dur,loop=0,disposal=2)
        if integrate:
            atlas.save(str(ROOT/c['base'])+'#sheet.png');write(str(ROOT/c['base'])+'#anim.fanim',packed)
        report['characters'][key]={'sourceSha256':{first.relative_to(ROOT).as_posix():digest(first),rest.relative_to(ROOT).as_posix():digest(rest)},
            'png':stem.relative_to(ROOT).as_posix()+'#sheet.png','animation':stem.relative_to(ROOT).as_posix()+'#anim.fanim',
            'sheet':list(atlas.size),'decodedBytes':atlas.width*atlas.height*4,'uniqueFrames':count,
            'frameCount':len(images),'poseDistinctFrames':source_counts,'footOffset':offset,
            'pngSha256':digest(str(stem)+'#sheet.png'),'animationSha256':digest(str(stem)+'#anim.fanim')}
        print(key,report['characters'][key]['sheet'],len(images),'frames',source_counts)
    write(OUT/'base-animation-results.json',report)


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--integrate',action='store_true');main(p.parse_args().integrate)
