"""Current whole-pose proposal checks, not acceptance of unfinished animations."""
from pathlib import Path
from PIL import Image
import hashlib,json,subprocess
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/grimhald-art-review'
files=['connected-v3-'+n+'.png' for n in ['idle','run','bash','unarmed']]+['effects-v2-preview.png','effects-v2-preview.fanim','walk-v4-preview.png','walk-v4-preview.fanim','walk-v4-preview.gif','movement-v5-preview.png','movement-v5-preview.fanim','idle-v5.png','idle-unarmed-v5.png','idle-walk-v5.gif','idle-v5-comparison.png']+['body-v6-preview.png','body-v6-preview.fanim','weapon-v6-preview.png','weapon-v6-preview.fanim','effects-v6-preview.png','effects-v6-preview.fanim']
def hashes():return {n:hashlib.sha256((OUT/n).read_bytes()).hexdigest() for n in files}
before=hashes()
for script in ['connected','walk','consistency','complete']:
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
assert all(e['tag'] in fxmeta for e in definitions['view_effects'] if e['tag']!='axe_planted')
assert all(e['tag'] in fxmeta for e in definitions['view_buffs'] if e['anim']=='asset/tfm2_custom/vfx/grimhald')
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
art=json.loads((ROOT/'rules/grimhald-art.json').read_text());bs=Image.open(OUT/'body-v6-preview.png');bm=json.loads((OUT/'body-v6-preview.fanim').read_text())['anims'];ws=Image.open(OUT/'weapon-v6-preview.png');wmeta=json.loads((OUT/'weapon-v6-preview.fanim').read_text())['anims'];fs=Image.open(OUT/'effects-v6-preview.png');fm=json.loads((OUT/'effects-v6-preview.fanim').read_text())['anims']
for name,millis in art['frameMillis'].items():
    entries=bm[name]['frames'];assert len(entries)==len(millis)
    for i,(f,ms) in enumerate(zip(entries,millis)):
        im=crop(bs,f);assert f['duration']==ms/1000
        assert ImageOps.mirror(im).tobytes()==crop(bs,bm[name+'_l']['frames'][i]).tobytes()
        assert im.tobytes()==crop(bs,bm[name+'_unarmed']['frames'][i]).tobytes()
        box=im.getbbox();assert box and 0<box[0] and 0<box[1] and box[2]<96 and box[3]==60
        tag='grh_weapon_'+name+'_r'+str(i);left='grh_weapon_'+name+'_l'+str(i)
        wi=crop(ws,wmeta[tag]['frames'][0]);assert ImageOps.mirror(wi).tobytes()==crop(ws,wmeta[left]['frames'][0]).tobytes()
        if name=='dead' or (name=='ult' and i==3):assert wi.getbbox() is None
        if name in ['idle','run']:assert im.tobytes()==crop(movement,mm[name+'_unarmed']['frames'][i]).tobytes()
    if len(entries)>1:assert len({crop(bs,f).tobytes() for f in entries})==len(entries)
for name,a in fxmeta.items():
    expected=[1/60] if name.startswith('axe_p') else [f['duration'] for f in a['frames']]
    assert expected==[f['duration'] for f in fm[name]['frames']]
    for oldf,newf in zip(a['frames'],fm[name]['frames']):assert crop(fx,oldf).tobytes()==crop(fs,newf).tobytes(),name
assert all(e['tag'] in fm for e in definitions['view_effects'])
assert all(e['tag'] in (wmeta if e['anim'].endswith('grimhald_weapon') else fm) for e in definitions['view_buffs'])
assert definitions['passive']['params']['weapon_layer']==1
for kind,stem,source in [('champions','tfm2_custom_grimhald','body-v6-preview'),('vfx','grimhald_weapon','weapon-v6-preview'),('vfx','grimhald','effects-v6-preview')]:
    for extension,suffix in [('.png','#sheet.png'),('.fanim','#anim.fanim')]:
        assert (OUT/(source+extension)).read_bytes()==(ROOT/'mods/tfm2_custom'/kind/(stem+suffix)).read_bytes()
old_def=json.loads(subprocess.check_output(['git','show','bfe6f12:mods/tfm2_custom/champion/tfm2_custom_grimhald.data_champion'],cwd=ROOT,text=True))
new_def=json.loads(json.dumps(definitions));new_def['passive']['params'].pop('weapon_layer')
for data in [old_def,new_def]:
    data.pop('view_effects');data.pop('view_buffs')
assert old_def==new_def,'nonvisual champion data changed'
decoded=sum(s.width*s.height*4 for s in [bs,ws,fs]);assert decoded<=32*1024*1024
assert all(max(s.size)<=2048 for s in [bs,ws,fs])
report.update(artRevision=6,completeFrames={n:len(v['frames']) for n,v in bm.items() if n in art['frameMillis']},legacyEffectPixelsUnchanged=True,phaseDurationCorrection='Eight single-frame flying-axe phases now last one tick, matching core emission; all other existing effect durations unchanged',gamePackIntegration='repository assets and weapon opt-in enabled; no installation',decodedBytes=decoded,completeAnimationAcceptance='All requested animation cycles completed from approved appearance; live alignment pending',nonvisualChampionDefinitionUnchanged=True,hashes=hashes())
(OUT/'art-validation.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
