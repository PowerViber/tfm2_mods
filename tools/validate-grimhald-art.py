"""Current whole-pose proposal checks, not acceptance of unfinished animations."""
from pathlib import Path
from PIL import Image
import hashlib,json,subprocess
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/grimhald-art-review'
files=['connected-v3-'+n+'.png' for n in ['idle','run','bash','unarmed']]+['effects-v2-preview.png','effects-v2-preview.fanim','walk-v4-preview.png','walk-v4-preview.fanim','walk-v4-preview.gif']
def hashes():return {n:hashlib.sha256((OUT/n).read_bytes()).hexdigest() for n in files}
before=hashes();subprocess.run(['python','tools/prepare-grimhald-connected.py'],cwd=ROOT,check=True,stdout=subprocess.DEVNULL);subprocess.run(['python','tools/prepare-grimhald-walk.py'],cwd=ROOT,check=True,stdout=subprocess.DEVNULL);assert before==hashes()
for name in files[:4]:
    im=Image.open(OUT/name);box=im.getbbox();assert box and box[0]>0 and box[1]>0 and box[2]<im.width and box[3]<im.height
    assert box[3]==im.height//2+20
fx=Image.open(OUT/'effects-v2-preview.png');assert max(fx.size)<=2048
for anim in json.loads((OUT/'effects-v2-preview.fanim').read_text())['anims'].values():
    for f in anim['frames']:
        d=f['data'];assert d['x']>=0 and d['y']>=0 and d['x']+d['w']<=fx.width and d['y']+d['h']<=fx.height
walk=Image.open(OUT/'walk-v4-preview.png');wm=json.loads((OUT/'walk-v4-preview.fanim').read_text())['anims']
from PIL import ImageOps
for name in ['run','run_unarmed']:
    fs=wm[name]['frames'];ls=wm[name+'_l']['frames'];assert len(fs)==len(ls)==6
    images=[];legs=[]
    for f,l in zip(fs,ls):
        d=f['data'];ld=l['data'];im=walk.crop((d['x'],d['y'],d['x']+d['w'],d['y']+d['h']));left=walk.crop((ld['x'],ld['y'],ld['x']+ld['w'],ld['y']+ld['h']))
        b=im.getbbox();assert b and b[0]>0 and b[1]>0 and b[2]<im.width and b[3]<im.height;assert f['duration']==.08;assert b[3]==im.height//2+20;assert ImageOps.mirror(im).tobytes()==left.tobytes();images.append(im.tobytes());legs.append(im.crop((0,50,96,60)).getchannel('A').tobytes())
    assert len(set(images))==len(set(legs))==6
definitions=json.loads((ROOT/'mods/tfm2_custom/champion/tfm2_custom_grimhald.data_champion').read_text())
fxmeta=json.loads((OUT/'effects-v2-preview.fanim').read_text())['anims']
assert all(e['tag'] in fxmeta for e in definitions['view_effects'])
assert all(e['tag'] in fxmeta for e in definitions['view_buffs'])
flat=json.loads((OUT/'effects-v2-preview.fanim').read_text())['anims']['fissure_d0']['frames'][1]['data']
floor=fx.crop((flat['x'],flat['y'],flat['x']+flat['w'],flat['y']+flat['h']));b=floor.getbbox();assert b[2]-b[0]>=4*(b[3]-b[1]),'fissure is not flat'
decoded=walk.width*walk.height*4+fx.width*fx.height*4+sum(Image.open(OUT/n).width*Image.open(OUT/n).height*4 for n in files[:4]);assert decoded<=32*1024*1024
old=json.loads(subprocess.check_output(['git','show','a9b73a4:rules/grimhald.json'],cwd=ROOT,text=True));now=json.loads((ROOT/'rules/grimhald.json').read_text())
allowed={'bulwark':{'length','castTicks'},'bash':{'range','knockback'},'ult':{'knockRadius','cageRadius'}}
for key,value in old.items():
    if isinstance(value,dict):
        for k,v in value.items():
            if k not in allowed.get(key,set()):assert now[key][k]==v,(key,k)
    else:assert now[key]==value,key
report={'rules':'grimhald-2','reproducibleExtractionAndPacking':True,'connectedKeyPoses':4,'walkFramesPerState':6,'walkStates':['armed','unarmed'],'walkMirrorsPixelExact':True,'groundFissureFlat':True,'keyPosesUnclipped':True,'footAnchorBelowCentre':20,'previewDecodedBytes':decoded,'unrelatedBalanceValuesUnchanged':True,'completeAnimationAcceptance':'walk ready for review; remaining cycles pending','gamePackIntegration':'pending','hashes':hashes()}
(OUT/'art-validation.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
