"""Current whole-pose proposal checks, not acceptance of unfinished animations."""
from pathlib import Path
from PIL import Image
import hashlib,json,subprocess
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/grimhald-art-review'
files=['connected-v3-'+n+'.png' for n in ['idle','run','bash','unarmed']]+['effects-v2-preview.png','effects-v2-preview.fanim']
def hashes():return {n:hashlib.sha256((OUT/n).read_bytes()).hexdigest() for n in files}
before=hashes();subprocess.run(['python','tools/prepare-grimhald-connected.py'],cwd=ROOT,check=True,stdout=subprocess.DEVNULL);assert before==hashes()
for name in files[:4]:
    im=Image.open(OUT/name);box=im.getbbox();assert box and box[0]>0 and box[1]>0 and box[2]<im.width and box[3]<im.height
    assert box[3]==im.height//2+20
fx=Image.open(OUT/'effects-v2-preview.png');assert max(fx.size)<=2048
for anim in json.loads((OUT/'effects-v2-preview.fanim').read_text())['anims'].values():
    for f in anim['frames']:
        d=f['data'];assert d['x']>=0 and d['y']>=0 and d['x']+d['w']<=fx.width and d['y']+d['h']<=fx.height
decoded=fx.width*fx.height*4+sum(Image.open(OUT/n).width*Image.open(OUT/n).height*4 for n in files[:4]);assert decoded<=32*1024*1024
old=json.loads(subprocess.check_output(['git','show','a9b73a4:rules/grimhald.json'],cwd=ROOT,text=True));now=json.loads((ROOT/'rules/grimhald.json').read_text())
allowed={'bulwark':{'length','castTicks'},'bash':{'range','knockback'},'ult':{'knockRadius','cageRadius'}}
for key,value in old.items():
    if isinstance(value,dict):
        for k,v in value.items():
            if k not in allowed.get(key,set()):assert now[key][k]==v,(key,k)
    else:assert now[key]==value,key
report={'rules':'grimhald-2','reproducibleExtractionAndPacking':True,'connectedKeyPoses':4,'keyPosesUnclipped':True,'footAnchorBelowCentre':20,'previewDecodedBytes':decoded,'unrelatedBalanceValuesUnchanged':True,'completeAnimationAcceptance':'pending appearance approval and completion','gamePackIntegration':'pending','hashes':hashes()}
(OUT/'art-validation.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
