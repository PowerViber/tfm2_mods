"""Check the actual packaged assets and reproducibility of approved Thalassa art."""
from pathlib import Path
from PIL import Image,ImageOps
import hashlib,json,subprocess,sys,re

ROOT=Path(__file__).resolve().parents[1]; REVIEW=ROOT/'docs/thalassa-art-review'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
approval=json.loads((REVIEW/'approval.json').read_text(encoding='utf-8'))
manifest=json.loads((REVIEW/'art-manifest.json').read_text(encoding='utf-8'))
body_base=ROOT/'mods/tfm2_custom/champions/tfm2_custom_thalassa'
fx_base=ROOT/'mods/tfm2_custom/vfx/thalassa'
link_base=ROOT/'mods/tfm2_custom/vfx/thalassa_links'
body=Image.open(str(body_base)+'#sheet.png').convert('RGBA');fx=Image.open(str(fx_base)+'#sheet.png').convert('RGBA')
anims=json.loads(Path(str(body_base)+'#anim.fanim').read_text(encoding='utf-8'))['anims']
effects=json.loads(Path(str(fx_base)+'#anim.fanim').read_text(encoding='utf-8'))['anims']
link=Image.open(str(link_base)+'#sheet.png').convert('RGBA')
links=json.loads(Path(str(link_base)+'#anim.fanim').read_text(encoding='utf-8'))['anims']
checks=0
def check(ok,msg):
    global checks
    assert ok,msg
    checks+=1
def crop(im,f):
    d=f['data'];return im.crop((d['x'],d['y'],d['x']+d['w'],d['y']+d['h']))
for kind,name in [('appearance','body-source-v1.png'),('effects','effects-source-v1.png')]:
    check(approval[kind]['approved'] and sha(REVIEW/name)==approval[kind]['source_sha256'],kind+' approved source')
counts={'idle':4,'run':6,'attack':4,'skill1':4,'skill2':4,'skill3':4,'ult':4,'hit':1,'dead':6}
for pose,n in counts.items():
    fs=anims[pose]['frames'];ls=anims[pose+'_l']['frames'];check(len(fs)==len(ls)==n,pose+' frame count')
    for i,(f,l) in enumerate(zip(fs,ls)):
        im=crop(body,f);mir=crop(body,l)
        check(im.getbbox() is not None,pose+' visible '+str(i))
        check(ImageOps.mirror(im).tobytes()==mir.tobytes(),pose+' mirror '+str(i))
        check(im.getbbox()[3]==49,pose+' tail ground anchor '+str(i))
        check(f['duration']==l['duration'] and f['duration']>0,pose+' timing '+str(i))
check(crop(body,anims['idle']['frames'][0]).tobytes()==Image.open(REVIEW/'body-idle.png').convert('RGBA').tobytes(),'approved roster idle pixels unchanged')
check(len({crop(body,f).tobytes() for f in anims['run']['frames']})==6,'six distinct whole-body movement drawings')
for im,table in [(body,anims),(fx,effects),(link,links)]:
    check(max(im.size)<=2048,'atlas dimension limit')
    for tag,ani in table.items():
        for f in ani['frames']:
            d=f['data'];check(d['x']>=0 and d['y']>=0 and d['x']+d['w']<=im.width and d['y']+d['h']<=im.height,tag+' frame bounds')
for tag in ('pool','whirlpool'):
    for i in range(8):check(len(effects[tag+'_p'+str(i)]['frames'])==1,tag+' single renewal phase')
for tag,ani in effects.items():
    if re.fullmatch(r'twin_(?:p\d+|(?:idle|run|attack|skill[123]|ult|hit|dead)_\d+(?:_l)?)',tag) or tag.startswith('trident_p'):
        check(len(ani['frames'])==1 and ani['frames'][0]['duration']==1/60,tag+' one-tick body/flight lifetime')
champ=json.loads((ROOT/'mods/tfm2_custom/champion/tfm2_custom_thalassa.data_champion').read_text(encoding='utf-8'))
for group in ('view_effects','view_buffs','view_projectiles'):
    for entry in champ[group]:check(entry['tag'] in (links if entry.get('anim','').endswith('thalassa_links') else effects),entry['name']+' binding')
decoded=(body.width*body.height+fx.width*fx.height+link.width*link.height)*4
check(decoded==manifest['decoded_bytes'] and decoded<=32*1024*1024,'decoded art budget')
baseline='68f3d8048bcbed7500921743f740e3821b580c60'
# The follow-up intentionally changes pool/Dive range, clone destination and trident speed.
# Preserve the art-only evidence from 3849991 separately; don't assert old gameplay equality.
rel='native/thalassa_lab/src/lib.rs'
original=subprocess.check_output(['rtk','proxy','git','show',baseline+':'+rel],cwd=ROOT)
check(original.replace(b'\r\n',b'\n')==(ROOT/rel).read_bytes().replace(b'\r\n',b'\n'),'transport and profile format unchanged')
original_rules=json.loads(subprocess.check_output(['rtk','proxy','git','show',baseline+':rules/thalassa.json'],cwd=ROOT))
rules=json.loads((ROOT/'rules/thalassa.json').read_text())
expected=json.loads(json.dumps(original_rules));expected['revision']='thalassa-1.1'
expected['water']['poolRadius']=35000;expected['tidePool']['whirlScalePct']=100
expected['dive']['range']=75000;expected['ult']['speed']=2000
# thalassa-1.2 (round 157 follow-up): the twin plays on its own and a water twin rescues her at any range
expected['revision']='thalassa-1.2';expected['twin'].update(walkSpeed=990,leash=60000,aggroRange=70000,kitePct=60,selfRescueGlobal=1)
# thalassa-1.3: the trident leaves a clone of her that fights on its own (no far-pool Dive)
expected['revision']='thalassa-1.3';expected['ult'].update(echoTicks=480,echoHpPct=25,echoDamagePct=50,echoLeash=80000,echoLowHpPct=40)
# thalassa-1.4: whirlpools pull once on the surge, swirl 2 s, bump at most 2; every pool slows 20%
expected['revision']='thalassa-1.4';[expected['tidePool'].pop(k) for k in ('pullStep','pullEveryTicks')];expected['tidePool'].update(bumpMaxChampions=2,whirlTicks=120);expected['water'].update(poolSlowPct=20)
check(rules==expected,'only authorized canonical rule changes')
for tag,ani in links.items():check(len(ani['frames'])==1 and ani['frames'][0]['duration']==1/60,tag+' one-tick tracked link')
check(Image.open(REVIEW/'effect-pool.png').size==Image.open(REVIEW/'effect-whirlpool.png').size==(74,74),'equal pool and whirlpool footprints')
original=json.loads(subprocess.check_output(['rtk','proxy','git','show',baseline+':mods/tfm2_custom/champion/tfm2_custom_thalassa.data_champion'],cwd=ROOT))
for key in ('stat','growth','attack','skill','skill2','ult','passive','passive_ult'):
    check(champ[key]==original[key],key+' unchanged gameplay definition')
paths=[Path(str(body_base)+'#sheet.png'),Path(str(body_base)+'#anim.fanim'),Path(str(fx_base)+'#sheet.png'),Path(str(fx_base)+'#anim.fanim'),Path(str(link_base)+'#sheet.png'),Path(str(link_base)+'#anim.fanim'),REVIEW/'art-manifest.json',ROOT/'editor/thalassa-art.js',ROOT/'native/tfm2_custom_ai/src/thalassa_art.rs']
before={p:sha(p) for p in paths}
subprocess.run([sys.executable,str(ROOT/'tools/pack-thalassa-art.py')],cwd=ROOT,check=True,capture_output=True)
for p in paths:check(before[p]==sha(p),'reproduction '+str(p.relative_to(ROOT)))
report={'checks':checks,'body_frames':37,'both_facings':True,'decoded_bytes':decoded,'atlases':[body.size,fx.size,link.size],'comparison_baseline':baseline,'rules_revision':rules['revision'],'authorized_followup':['pool 35000 equal to existing whirlpool','Dive range 75000, unchanged travel formula','clone copies committed Dive endpoint/clock','trident 2000/2600 units per tick','live segmented link','60 Hz real-time Lab clock'],
        'hashes':{p.relative_to(ROOT).as_posix():sha(p) for p in paths},'live_game_acceptance':'pending'}
(REVIEW/'verification.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
print(json.dumps(report))
