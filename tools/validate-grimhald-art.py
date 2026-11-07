"""Current whole-pose proposal checks, not acceptance of unfinished animations."""
from pathlib import Path
from PIL import Image
import hashlib,json,subprocess
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/grimhald-art-review'
files=['connected-v3-'+n+'.png' for n in ['idle','run','bash','unarmed']]+['effects-v2-preview.png','effects-v2-preview.fanim','walk-v4-preview.png','walk-v4-preview.fanim','walk-v4-preview.gif','movement-v5-preview.png','movement-v5-preview.fanim','idle-v5.png','idle-unarmed-v5.png','idle-walk-v5.gif','idle-v5-comparison.png']
def hashes():return {n:hashlib.sha256((OUT/n).read_bytes()).hexdigest() for n in files}
before=hashes()
for script in ['connected','walk','consistency']:
    subprocess.run(['python','tools/prepare-grimhald-'+script+'.py'],cwd=ROOT,check=True,stdout=subprocess.DEVNULL)
assert before==hashes()
for name in files[:4]:
    im=Image.open(OUT/name);box=im.getbbox();assert box and box[0]>0 and box[1]>0 and box[2]<im.width and box[3]<im.height
    assert box[3]==im.height//2+20
fx=Image.open(OUT/'effects-v2-preview.png');assert max(fx.size)<=2048
for anim in json.loads((OUT/'effects-v2-preview.fanim').read_text())['anims'].values():
    for f in anim['frames']:
        d=f['data'];assert d['x']>=0 and d['y']>=0 and d['x']+d['w']<=fx.width and d['y']+d['h']<=fx.height
walk=Image.open(OUT/'walk-v4-preview.png');wm=json.loads((OUT/'walk-v4-preview.fanim').read_text())['anims']
assert hashlib.sha256((OUT/'walk-v4-preview.png').read_bytes()).hexdigest()=='7ff8d016ad1138013aa5b3a5d633f924199ebd2aa9b39545e9bbd3c0fa57b927','approved walk pixels changed'
from PIL import ImageOps
for name in ['run','run_unarmed']:
    fs=wm[name]['frames'];ls=wm[name+'_l']['frames'];assert len(fs)==len(ls)==6
    images=[];legs=[]
    for f,l in zip(fs,ls):
        d=f['data'];ld=l['data'];im=walk.crop((d['x'],d['y'],d['x']+d['w'],d['y']+d['h']));left=walk.crop((ld['x'],ld['y'],ld['x']+ld['w'],ld['y']+ld['h']))
        b=im.getbbox();assert b and b[0]>0 and b[1]>0 and b[2]<im.width and b[3]<im.height;assert f['duration']==.16;assert b[3]==im.height//2+20;assert ImageOps.mirror(im).tobytes()==left.tobytes();images.append(im.tobytes());legs.append(im.crop((0,50,96,60)).getchannel('A').tobytes())
    assert len(set(images))==len(set(legs))==6
definitions=json.loads((ROOT/'mods/tfm2_custom/champion/tfm2_custom_grimhald.data_champion').read_text())
fxmeta=json.loads((OUT/'effects-v2-preview.fanim').read_text())['anims']
assert all(e['tag'] in fxmeta for e in definitions['view_effects'])
assert all(e['tag'] in fxmeta for e in definitions['view_buffs'])
flat=json.loads((OUT/'effects-v2-preview.fanim').read_text())['anims']['fissure_d0']['frames'][1]['data']
floor=fx.crop((flat['x'],flat['y'],flat['x']+flat['w'],flat['y']+flat['h']));b=floor.getbbox();assert b[2]-b[0]>=4*(b[3]-b[1]),'fissure is not flat'
movement=Image.open(OUT/'movement-v5-preview.png');mm=json.loads((OUT/'movement-v5-preview.fanim').read_text())['anims']
def crop(atlas,f):
    d=f['data'];return atlas.crop((d['x'],d['y'],d['x']+d['w'],d['y']+d['h']))
for name,count in [('idle',4),('idle_unarmed',4),('run',6),('run_unarmed',6)]:
    fs=mm[name]['frames'];assert len(fs)==count
    for i,f in enumerate(fs):
        im=crop(movement,f);b=im.getbbox();assert b and b[0]>0 and b[1]>0 and b[2]<96 and b[3]==60
        assert ImageOps.mirror(im).tobytes()==crop(movement,mm[name+'_l']['frames'][i]).tobytes()
        assert f['duration']==(.16 if name.startswith('run') else .18)
        if name.startswith('run'):
            assert im.tobytes()==crop(walk,wm[name]['frames'][i]).tobytes(),'approved walk drawing changed'
    assert len({crop(movement,f).tobytes() for f in fs})==count
# The unarmed images isolate physical body height from the raised axe.
idle_height=[60-crop(movement,f).getbbox()[1] for f in mm['idle_unarmed']['frames']]
run_height=[60-crop(movement,f).getbbox()[1] for f in mm['run_unarmed']['frames']]
import statistics
assert abs(statistics.median(idle_height)-statistics.median(run_height))<=1
assert all(abs(h-statistics.median(idle_height))<=2 for h in run_height),(idle_height,run_height)
def steel_luma(name):
    pixels=[p for f in mm[name]['frames'] for p in crop(movement,f).get_flattened_data()
            if p[3]>=180 and 40<=p[0]<=220 and max(p[:3])-min(p[:3])<35]
    import statistics
    return statistics.median((p[0]*2126+p[1]*7152+p[2]*722)/10000 for p in pixels)
idle_luma=steel_luma('idle_unarmed');run_luma=steel_luma('run_unarmed')
assert abs(idle_luma-run_luma)<18,(idle_luma,run_luma)
decoded=movement.width*movement.height*4+fx.width*fx.height*4+2*96*80*4;assert decoded<=32*1024*1024
baseline='7426636d2e97993e64850fa9a9a9316b75c0eba7'
old=json.loads(subprocess.check_output(['git','show',baseline+':rules/grimhald.json'],cwd=ROOT,text=True));now=json.loads((ROOT/'rules/grimhald.json').read_text());assert old==now
report={'rules':'grimhald-2','artRevision':5,'reproducibleExtractionAndPacking':True,'idleFramesPerState':4,'walkFramesPerState':6,'walkStates':['armed','unarmed'],'walkPixelsUnchanged':True,'mirrorsPixelExact':True,'groundFissureFlat':True,'footAnchorBelowCentre':20,'bodyHeights':{'idle':idle_height,'walk':run_height},'steelLuma':{'idle':idle_luma,'walk':run_luma},'walkStrideWorld':60000,'nominalWalkCycleSeconds':.96,'previewDecodedBytes':decoded,'gameplayRulesUnchangedFrom':baseline,'completeAnimationAcceptance':'Walk and ground correction approved; idle consistency revised; remaining action/death cycles pending','gamePackIntegration':'pending','hashes':hashes()}
(OUT/'art-validation.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
