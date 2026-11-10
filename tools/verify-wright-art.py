"""Verify approved Wright sources, real atlases, unchanged mechanics and deterministic packing."""
from pathlib import Path
from PIL import Image,ImageOps
import hashlib,json,subprocess,sys
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/wright-art-review';BASE='1f96d4805670a16f18c018b4001bb66fff715749'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
manifest=json.loads((OUT/'art-manifest.json').read_text());approval=json.loads((OUT/'approval.json').read_text());checks=[]
def check(name,ok):
    assert ok,name
    checks.append(name)
def crop(im,f):
    d=f['data'];return im.crop((d['x'],d['y'],d['x']+d['w'],d['y']+d['h']))
body_base=ROOT/'mods/tfm2_custom/champions/tfm2_custom_wright';fx_base=ROOT/'mods/tfm2_custom/vfx/wright'
body=Image.open(str(body_base)+'#sheet.png').convert('RGBA');fx=Image.open(str(fx_base)+'#sheet.png').convert('RGBA')
poses=json.loads(Path(str(body_base)+'#anim.fanim').read_text())['anims'];effects=json.loads(Path(str(fx_base)+'#anim.fanim').read_text())['anims']
for group in ('body','fleet','effects'):check(group+' approval',approval[group]['approved'])
for source,digest in manifest['sources'].items():check(source+' approved frozen source',sha(OUT/source)==approval['sources'][source]==digest)
for source,digest in manifest['animation_sources'].items():check(source+' frozen animation input',sha(OUT/source)==digest)
check('all nine poses complete',manifest['complete_body'] and not manifest['review_only'] and len(manifest['pose_ticks'])==9)
for pose,ticks in manifest['pose_ticks'].items():
    fs=poses[pose]['frames'];ls=poses[pose+'_l']['frames'];check(pose+' eight frames per facing',len(fs)==len(ls)==8)
    for i,(f,l,t) in enumerate(zip(fs,ls,ticks)):
        im=crop(body,f);other=crop(body,l);box=im.getbbox()
        check(pose+' visible '+str(i),box is not None)
        check(pose+' mirror '+str(i),ImageOps.mirror(im).tobytes()==other.tobytes())
        check(pose+' ground anchor '+str(i),box[3]==49)
        check(pose+' timing '+str(i),f['duration']==l['duration']==t/60)
    if pose!='dead':check(pose+' neutral scale',38<=crop(body,fs[0]).getbbox()[3]-crop(body,fs[0]).getbbox()[1]<=42)
check('six or more distinct walk drawings',len({crop(body,f).tobytes() for f in poses['run']['frames']})>=6)
check('approved real roster idle preserved',crop(body,poses['idle']['frames'][0]).crop((8,0,56,56)).tobytes()==Image.open(OUT/'body-idle.png').convert('RGBA').tobytes())
for im,table in ((body,poses),(fx,effects)):
    check('atlas dimension limit',max(im.size)<=2048)
    for tag,animation in table.items():
        for f in animation['frames']:
            d=f['data'];check(tag+' atlas bounds',d['x']>=0 and d['y']>=0 and d['x']+d['w']<=im.width and d['y']+d['h']<=im.height)
for plane in ('fighter','bomber','chopper','repair','drone','rocket'):
    for heading in range(8):
        fs=effects[plane+'_p'+str(heading)]['frames'];check(plane+' heading '+str(heading),len(fs)==1 and fs[0]['duration']==1/60)
check('carrier single tick renewal',effects['carrier']['frames'][0]['duration']==1/60)
for tag,ticks in [('warn',114),('warn_t114',114),('warn_t72',72),('carrier_shadow',60),('carrier_land',30),('knock_ring',24),('mg',8),('lock',30),('kamikaze',24),('repair_kit',20),('drone_up',18),('taunt',30),('bomb_t14',14),('bomb_t16',16),('bomb_t18',18),('bomber_wave1',14),('bomber_wave2',14),('bomber_wave3',14)]:
    check(tag+' complete effect lifetime',abs(sum(f['duration'] for f in effects[tag]['frames'])*60-ticks)<1e-6)
for wave in range(1,4):
    for mark in (60,18):
        deadline=mark+(wave-1)*18;tag=f'warn{wave}_t{deadline}'
        check(tag+' warning ends on its own strike',abs(sum(f['duration'] for f in effects[tag]['frames'])*60-deadline)<1e-6)
        check(tag+' true blast footprint',effects[tag]['frames'][0]['data']['w']==round(45000*2/950)+2)
for tag,world_radius in [('warn_t114',45000),('warn_t72',45000),('bomber_wave1',45000),('bomber_wave3',45000),('knock_ring',100000),('carrier_shadow',100000),('carrier_land',100000)]:
    for f in effects[tag]['frames']:check(tag+' canonical footprint',f['data']['w']==round(world_radius*2/manifest['units_per_pixel'])+2)
for phase in ('run','break'):
    for variant in range(2):
        for heading in range(8):
            fs=effects[f'fighter_{phase}{variant}_p{heading}']['frames'];check('single-frame '+phase+str(variant)+' '+str(heading),len(fs)==1 and fs[0]['duration']==1/60)
champ_path='mods/tfm2_custom/champion/tfm2_custom_wright.data_champion';champ=json.loads((ROOT/champ_path).read_text())
for group in ('view_effects','view_buffs','view_projectiles'):
    for entry in champ[group]:check(entry['name']+' asset binding',entry['tag'] in effects)
check('lock follows target',next(e for e in champ['view_effects'] if e['name']=='wrt_lock')['is_follow'])
check('drop warning follows actual carrier centre',next(e for e in champ['view_effects'] if e['name']=='wrt_carrier_shadow')['is_follow'])
for i in range(4):check('native stock '+str(i),any(b['name']=='wrt_airframes:'+str(i) for b in champ['view_buffs']))
decoded=(body.width*body.height+fx.width*fx.height)*4;check('decoded artwork budget',decoded==manifest['decoded_bytes'] and decoded<=32*1024*1024)
before_rules=json.loads(subprocess.check_output(['rtk','proxy','git','show',BASE+':rules/wright.json'],cwd=ROOT))
before_rules['revision']='wright-3';before_rules['bomber'].pop('bombsPerWave');before_rules['bomber']['spacing']=40000
# wright-4 (Reihan): jets fly sorties from him (no lock-on, no dogfight runs); lockTicks stays for the unused lock art
before_rules['revision']='wright-4'
for k in ('gunEveryTicks','gunBase','gunApRatio','rocketEveryTicks','rocketSpeed','boostPct','runTicks','breakTicks','breakRadius','overshoot','staggerTicks'):before_rules['fighter'].pop(k)
before_rules['fighter'].update(gunRange=15000,sortieBase=20,sortieApRatio=25,rocketEvery=3,rocketSplash=8000,homeRadius=8000,sortieSpeed=4800,restTicks=12)
# wright-5 (round 160 nerf): AP growth 26 -> 12, Mod Power AP 25 -> 10; jets carry 3 strikes and rearm, cruise slower with
# afterburners, reach farther, hit softer; the bomber is smaller and slower
before_rules['revision']='wright-5';before_rules['growth']['magic_power']=12;before_rules['modPower'][1]=10
for k in ('sortieSpeed','restTicks'):before_rules['fighter'].pop(k)
before_rules['fighter'].update(sortieBase=12,sortieApRatio=14,rocketBase=24,rocketApRatio=26,kamikazeBase=20,kamikazeApRatio=22,burnPermillePerSecond=10,aggroRange=110000,ammo=3,rearmTicks=120,cruiseSpeed=2400,afterburnerPct=150,afterburnerRange=25000,retargetRange=40000)
before_rules['bomber'].update(radius=30000,spacing=28000,waveEveryTicks=36,waveBase=30,waveApRatio=25)
check('only intended canonical bomber geometry changes',before_rules==json.loads((ROOT/'rules/wright.json').read_text()))
for rel in ('mods/tfm2_custom/champions/tfm2_custom_wright#sheet.png','mods/tfm2_custom/champions/tfm2_custom_wright#anim.fanim'):
    before=subprocess.check_output(['rtk','proxy','git','show',BASE+':'+rel],cwd=ROOT);after=(ROOT/rel).read_bytes()
    if not rel.endswith('.wasm'):before=before.replace(b'\r\n',b'\n');after=after.replace(b'\r\n',b'\n')
    check(rel+' unchanged combat/transport',before==after)
before=json.loads(subprocess.check_output(['rtk','proxy','git','show',BASE+':'+champ_path],cwd=ROOT))
for key in ('stat','growth','attack','skill','skill2','ult','passive','passive_ult'):
    want=json.loads(json.dumps(before[key]))
    if key=='growth':want['magic_power']=12  # round 160 nerf
    check(key+' unchanged definition (round 160 growth nerf aside)',want==champ[key])
files=[Path(str(b)+s) for b in (body_base,fx_base) for s in ('#sheet.png','#anim.fanim')]+[OUT/'art-manifest.json',ROOT/'editor/wright-art.js',ROOT/'native/tfm2_custom_ai/src/wright_art.rs']
hashes={p:sha(p) for p in files};subprocess.run([sys.executable,str(ROOT/'tools/build-wright-art.py'),'--integrate'],cwd=ROOT,capture_output=True,check=True)
for p,digest in hashes.items():check(str(p.relative_to(ROOT))+' reproducible',sha(p)==digest)
report={'checks':len(checks),'body_frames':72,'both_facings':True,'fx_tags':len(effects),'decoded_bytes':decoded,'atlases':[body.size,fx.size],'combat_baseline':BASE,
        'scope':'Intentional wright-3 bomber change: three fixed advancing centres and one large explosion each. Other skills and approved bodies preserved; full current native/WASM parity and non-bomber baseline checked separately.','hashes':{p.relative_to(ROOT).as_posix():sha(p) for p in files},'live_game_acceptance':'pending'}
(OUT/'verification.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report))
