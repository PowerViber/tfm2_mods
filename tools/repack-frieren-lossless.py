"""Deduplicate and compact Frieren atlases; verify every animation against main's frozen baseline."""
from pathlib import Path
import argparse, copy, hashlib, io, json, subprocess
from PIL import Image
ROOT=Path(__file__).resolve().parents[1]
BASE='a4098f15be65edc683fd62f8037d71b1e79ae9bd'
def original(path):
    p=subprocess.run(['rtk','proxy','git','show',BASE+':'+path.relative_to(ROOT).as_posix()],cwd=ROOT,capture_output=True,check=True)
    return p.stdout
def signature(sheet,doc):
    result=[]
    for name,a in doc['anims'].items():
        for frame in a['frames']:
            f=copy.deepcopy(frame);r=f.pop('data');metadata={k:v for k,v in r.items() if k not in ['x','y','w','h']}
            crop=sheet.crop((r['x'],r['y'],r['x']+r['w'],r['y']+r['h']))
            assert crop.size==(r['w'],r['h'])
            result.append((name,f,metadata,crop.size,hashlib.sha256(crop.tobytes()).hexdigest()))
    return result
def pack(sheet,doc):
    unique={};frames=[]
    for a in doc['anims'].values():
        for f in a['frames']:
            r=f['data'];crop=sheet.crop((r['x'],r['y'],r['x']+r['w'],r['y']+r['h']));key=(crop.size,crop.tobytes())
            if key not in unique:unique[key]=[crop,None]
            frames.append((r,key))
    best=None
    for width in [256,512,1024,2048]:
        if max(im.width for im,_ in unique.values())>width:continue
        x=y=row=0;places={}
        for key,(im,_) in sorted(unique.items(),key=lambda kv:(-kv[1][0].height,-kv[1][0].width,hashlib.sha256(kv[0][1]).hexdigest())):
            if x+im.width>width:x=0;y+=row+1;row=0
            places[key]=(x,y);x+=im.width+1;row=max(row,im.height)
        height=y+row
        if height>2048:continue
        if best is None or width*height<best[0]:best=(width*height,width,height,places)
    if best is None:raise ValueError('No valid compact atlas under 2048 pixels')
    _,w,h,places=best;out=Image.new('RGBA',(w,h))
    for key,(im,_) in unique.items():out.paste(im,places[key])
    for r,key in frames:r['x'],r['y']=places[key]
    return out,len(frames)-len(unique)
if __name__=='__main__':
    ap=argparse.ArgumentParser(description=__doc__);ap.add_argument('--apply',action='store_true');a=ap.parse_args();report=[]
    for anim in sorted((ROOT/'mods/tfm2_frieren').rglob('*#anim.fanim')):
        path=anim.with_name(anim.name.replace('#anim.fanim','#sheet.png'))
        baseline=Image.open(io.BytesIO(original(path))).convert('RGBA');old=json.loads(original(anim));sheet=Image.open(path).convert('RGBA');doc=json.loads(anim.read_text())
        expected=signature(baseline,old);assert signature(sheet,doc)==expected,str(anim)
        if a.apply:
            packed,duplicates=pack(sheet,doc)
            if packed.width*packed.height<=sheet.width*sheet.height:
                assert signature(packed,doc)==expected
                packed.save(path,optimize=True);anim.write_text(json.dumps(doc,separators=(',',':'))+'\n');sheet=packed
        assert max(sheet.size)<=2048,str(path)
        report.append({'atlas':path.relative_to(ROOT).as_posix(),'beforeBytes':baseline.width*baseline.height*4,'afterBytes':sheet.width*sheet.height*4,'framesVerified':len(expected),'width':sheet.width,'height':sheet.height})
    out=ROOT/'docs/objective-stark-results/animation-equivalence.json';out.parent.mkdir(parents=True,exist_ok=True);out.write_text(json.dumps({'baseline':BASE,'atlases':report,'beforeBytes':sum(r['beforeBytes'] for r in report),'afterBytes':sum(r['afterBytes'] for r in report),'pixelsAnchorsOrderAndDurationsIdentical':True},indent=2)+'\n')
    print(out.read_text())
