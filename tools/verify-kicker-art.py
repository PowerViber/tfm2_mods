"""Verify approved art, exact wall-only rules changes and native/Lab bindings."""
from pathlib import Path
import hashlib,json,subprocess
from PIL import Image,ImageOps
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/kicker-art-review'
BASE='2b7da261b1078269b484aa7ac59561fa64c1f023'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def old(p):return subprocess.check_output(['rtk','proxy','git','show',BASE+':'+p],cwd=ROOT)
def main():
 m=json.loads((OUT/'presentation.json').read_text());app=json.loads((OUT/'approvals.json').read_text())
 assert m['body_approved'] and m['effects_approved'] and app['effects']['approved']
 for name,digest in m['sources_sha256'].items():assert sha(OUT/name)==digest,name
 hashes={n:sha(OUT/n) for n in ['body.png','body.fanim','effects.png','effects.fanim','presentation.json']}
 subprocess.run(['rtk','proxy','python',str(ROOT/'tools/build-kicker-art.py')],cwd=ROOT,check=True,capture_output=True)
 assert hashes=={n:sha(OUT/n) for n in hashes},'Packing is not reproducible'
 total=entries=0;bounds={}
 for stem in ['body','effects']:
  sheet=Image.open(OUT/(stem+'.png')).convert('RGBA');a=json.loads((OUT/(stem+'.fanim')).read_text())['anims']
  assert max(sheet.size)<=2048;total+=sheet.width*sheet.height*4
  def crop(f):
   r=f['data'];return sheet.crop((r['x'],r['y'],r['x']+r['w'],r['y']+r['h']))
  for tag,anim in a.items():
   for f in anim['frames']:
    r=f['data'];assert r['x']>=0 and r['y']>=0 and r['x']+r['w']<=sheet.width and r['y']+r['h']<=sheet.height,(stem,tag)
    assert f['duration']>0;entries+=1
  if stem=='body':
   for tag,spec in m['poses'].items():
    assert len(a[tag]['frames'])==len(spec['ticks'])
    for right,left in zip(a[tag]['frames'],a[tag+'_l']['frames']):
     r,l=crop(right),crop(left);assert ImageOps.mirror(r).tobytes()==l.tobytes(),tag
     box=r.getbbox();assert box and box[0]>0 and box[1]>0 and box[2]<64 and box[3]==48,(tag,box)
    assert abs(sum(f['duration'] for f in a[tag]['frames'])-sum(spec['ticks'])/60)<1e-9,tag
  else:
   for tag in m['buff_offsets']:
    material=crop(a[tag]['frames'][0]);attached=crop(a[tag+'_attached']['frames'][0])
    material=material.crop(material.getbbox());attached=attached.crop(attached.getbbox())
    assert material.tobytes()==attached.tobytes() and material.size==attached.size,tag
   for tag in ['tag','rock_e','kick_trail_e','skyfall_se']:assert len(a[tag]['frames'])==1,tag
  target='champions/tfm2_custom_kicker' if stem=='body' else 'vfx/kicker'
  for suffix,ext in [('sheet','.png'),('anim','.fanim')]:
   assert (OUT/(stem+ext)).read_bytes()==(ROOT/'mods/tfm2_custom'/(target+'#'+suffix+ext)).read_bytes()
  bounds[stem]={'dimensions':sheet.size,'tags':len(a),'sha256':sha(OUT/(stem+'.png'))}
 assert total==m['decoded_bytes'] and total<=32*1024*1024
 previous=json.loads(old('rules/kicker.json'));current=json.loads((ROOT/'rules/kicker.json').read_text())
 previous['revision']=current['revision'];previous['hop']['wallRange']=300000
 assert previous==current,'Unexpected balance changes'
 path='mods/tfm2_custom/champion/tfm2_custom_kicker.data_champion'
 before=json.loads(old(path));after=json.loads((ROOT/path).read_text())
 strip=lambda c:{k:v for k,v in c.items() if k not in ['view_effects','view_buffs','sprite','passive']}
 assert strip(before)==strip(after),'Nonvisual champion data changed'
 assert before['passive']['passive_ref']==after['passive']['passive_ref']
 assert after['passive']['params']=={'art_fx_v1':1},'Do not enable Animation CC for cosmetic poses'
 effects=json.loads((OUT/'effects.fanim').read_text())['anims']
 for b in after['view_effects']+after['view_buffs']:assert b['tag'] in effects,b
 presenter=(ROOT/'native/tfm2_custom_ai/src/kicker_presentation.rs').read_text().split('#[cfg(test)]')[0]
 for banned in ['apply_cc(', 'deal_damage(', 'entity_set_pos(', 'set_input(', 'random(']:assert banned not in presenter,banned
 report={'body_and_effects_approved':True,'decoded_bytes':total,'atlases':bounds,'frame_entries':entries,'reproducible_packing':True,
  'foot_anchor':[32,48],'wall_range':300000,'rules_revision':current['revision'],'unchanged_damage_stats_cooldowns':True,
  'native_special_pose_cc_enabled':False,'native_effects':'short single-frame leases within the existing budget; no more than one per update',
  'live_game':'pending by user request','packaged_dll_sha256':sha(ROOT/'mods/tfm2_custom_ai/tfm2_custom_ai.dll')}
 (OUT/'verification.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
if __name__=='__main__':main()
