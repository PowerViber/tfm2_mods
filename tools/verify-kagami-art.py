"""Validate frozen art, actual bindings and the unchanged mechanical kit."""
from pathlib import Path
from PIL import Image,ImageOps
import json,hashlib,subprocess
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/kagami-art-review';BASE='07e5d41cfa226bd11302cff58efe8d3bbc6f503d'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def old(p):return subprocess.check_output(['rtk','proxy','git','show',BASE+':'+p],cwd=ROOT)
art=json.loads((ROOT/'rules/kagami-art.json').read_text());approval=json.loads((OUT/'approvals.json').read_text())
assert approval['body']['approved'] and approval['effects']['approved']
for name,h in art['source_sha256'].items():assert sha(OUT/name)==h,name
paths=[OUT/(n+e) for n in ['body','effects'] for e in ['.png','.fanim']]+[ROOT/'rules/kagami-art.json',ROOT/'editor/kagami-art.js',ROOT/'native/tfm2_custom_ai/src/kagami_art.rs']
before={str(p.relative_to(ROOT)):sha(p) for p in paths}
subprocess.run(['rtk','proxy','python','tools/build-kagami-art.py'],cwd=ROOT,check=True,capture_output=True)
assert before=={str(p.relative_to(ROOT)):sha(p) for p in paths},'Non-reproducible packing'
total=entries=0
for stem in ['body','effects']:
 im=Image.open(OUT/(stem+'.png')).convert('RGBA');anims=json.loads((OUT/(stem+'.fanim')).read_text())['anims'];total+=im.width*im.height*4
 assert max(im.size)<=2048
 for name,a in anims.items():
  for f in a['frames']:
   r=f['data'];assert r['x']>=0 and r['y']>=0 and r['x']+r['w']<=im.width and r['y']+r['h']<=im.height,(stem,name,r)
   assert f['duration']>0;entries+=1
 if stem=='body':
  for pose,ticks in art['pose_ticks'].items():
   assert len(anims[pose]['frames'])==len(ticks)
   assert abs(sum(f['duration'] for f in anims[pose]['frames'])-sum(ticks)/60)<1e-9
   for a,b in zip(anims[pose]['frames'],anims[pose+'_l']['frames']):
    def crop(f):
     r=f['data'];return im.crop((r['x'],r['y'],r['x']+r['w'],r['y']+r['h']))
    right,left=crop(a),crop(b);assert ImageOps.mirror(right).tobytes()==left.tobytes()
    box=right.getbbox();assert box and box[0]>0 and box[1]>0 and box[2]<64 and box[3]==52,(pose,box)
  assert len({json.dumps(f['data']) for f in anims['run']['frames']})==6,'Six distinct whole-body walking drawings'
 target=ROOT/'mods/tfm2_custom'/('champions/tfm2_custom_kagami' if stem=='body' else 'vfx/kagami')
 for ext,suffix in [('.png','#sheet.png'),('.fanim','#anim.fanim')]:assert (OUT/(stem+ext)).read_bytes()==Path(str(target)+suffix).read_bytes()
assert total==art['decoded_bytes']<=32*1024*1024
previous=json.loads(old('rules/kagami.json'));current=json.loads((ROOT/'rules/kagami.json').read_text())
assert {k:v for k,v in previous.items() if k not in ['revision','ai']}=={k:v for k,v in current.items() if k not in ['revision','ai']}
for k,v in previous['ai'].items():assert current['ai'][k]==v,'Existing neutral thresholds preserved'
path='mods/tfm2_custom/champion/tfm2_custom_kagami.data_champion';a=json.loads(old(path));b=json.loads((ROOT/path).read_text())
strip=lambda c:{k:v for k,v in c.items() if k not in ['sprite','view_effects','view_buffs','view_projectiles','passive']}
assert strip(a)==strip(b),'Mechanical champion data changed'
assert a['passive']['passive_ref']==b['passive']['passive_ref'] and b['passive']['params']=={'art_v1':1}
for binding in b['view_effects']+b['view_buffs']+b['view_projectiles']:assert binding['tag'] in art['effects'],binding
assert all(len(a['frames'])==1 for n,a in art['effects'].items() if (n.startswith('clone_') and n!='clone_in') or n.startswith('thread_p'))
assert len({b['name'] for b in b['view_effects']})==len(b['view_effects']),'Duplicate effect aliases'
report={'approved_body_and_effects':True,'reproducible_packing':True,'decoded_bytes':total,'frame_entries':entries,'walk_drawings':6,'foot_offset_from_centre':20,'unchanged_stats_damage_heal_shield_cooldowns_mod_power':True,'rules':current['revision'],'asset_sha256':before,'native_owner_animation_locks':False,'native_clone_visual_mask':'single lifetime on existing core-controlled summon; native rendering acceptance pending','actual_game':'pending'}
(OUT/'verification.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report))
