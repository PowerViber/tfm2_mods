"""Verify Kicker's approved body, reproducible packing and unchanged native mechanics."""
from pathlib import Path
import hashlib,json,subprocess
from PIL import Image,ImageOps
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/kicker-art-review'
BASE='2b7da261b1078269b484aa7ac59561fa64c1f023'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def git(p):return subprocess.check_output(['rtk','proxy','git','show',BASE+':'+p],cwd=ROOT)
def main():
 m=json.loads((OUT/'presentation.json').read_text());assert m['body_approved'] and not m['effects_approved']
 for name,digest in m['sources_sha256'].items():assert sha(OUT/name)==digest,name
 hashes={n:sha(OUT/n) for n in ['body.png','body.fanim','effects.png','effects.fanim','presentation.json']}
 subprocess.run(['rtk','proxy','python',str(ROOT/'tools/build-kicker-art.py')],cwd=ROOT,check=True,capture_output=True)
 assert hashes=={n:sha(OUT/n) for n in hashes},'Packing is not reproducible'
 total=0;entries=0;bounds={}
 for stem in ['body','effects']:
  sheet=Image.open(OUT/(stem+'.png')).convert('RGBA');a=json.loads((OUT/(stem+'.fanim')).read_text())['anims']
  assert max(sheet.size)<=2048;total+=sheet.width*sheet.height*4
  for tag,anim in a.items():
   for f in anim['frames']:
    r=f['data'];assert r['x']>=0 and r['y']>=0 and r['x']+r['w']<=sheet.width and r['y']+r['h']<=sheet.height,(stem,tag)
    assert f['duration']>0;entries+=1
  if stem=='body':
   for tag,spec in m['poses'].items():
    assert len(a[tag]['frames'])==len(spec['ticks'])
    for right,left in zip(a[tag]['frames'],a[tag+'_l']['frames']):
     def crop(f):
      r=f['data'];return sheet.crop((r['x'],r['y'],r['x']+r['w'],r['y']+r['h']))
     r,l=crop(right),crop(left);assert ImageOps.mirror(r).tobytes()==l.tobytes(),tag
     box=r.getbbox();assert box and box[0]>0 and box[1]>0 and box[2]<64 and box[3]==48,(tag,box)
    assert abs(sum(f['duration'] for f in a[tag]['frames'])-sum(spec['ticks'])/60)<1e-9,tag
   for suffix,ext in [('sheet','.png'),('anim','.fanim')]:
    assert (OUT/('body'+ext)).read_bytes()==(ROOT/'mods/tfm2_custom/champions'/('tfm2_custom_kicker#'+suffix+ext)).read_bytes()
  bounds[stem]={'dimensions':sheet.size,'tags':len(a),'sha256':sha(OUT/(stem+'.png'))}
 assert total==m['decoded_bytes'] and total<=32*1024*1024
 unchanged=['rules/kicker.json','native/tfm2_custom_ai/src/kicker.rs','native/tfm2_custom_ai/src/kicker_core.rs','native/tfm2_custom_ai/src/kicker_data.rs','native/kicker_lab/src/lib.rs','editor/kicker-data.js','editor/kicker.wasm','mods/tfm2_custom_ai/tfm2_custom_ai.dll']
 for name in unchanged:
  new=(ROOT/name).read_bytes();old=git(name)
  assert new==old if name.endswith(('.dll','.wasm')) else new.replace(b'\r\n',b'\n')==old.replace(b'\r\n',b'\n'),name
 path='mods/tfm2_custom/champion/tfm2_custom_kicker.data_champion'
 before=json.loads(git(path));after=json.loads((ROOT/path).read_text());before['sprite']=after['sprite']
 assert before==after,'Only the body asset path may change before effect approval'
 assert after['passive']['params']=={} and not (ROOT/'mods/tfm2_custom/vfx/kicker#sheet.png').exists()
 assert after['sprite'].endswith('tfm2_custom_kicker')
 report={'approved_body':True,'effects_integrated':False,'decoded_bytes':total,'atlases':bounds,'frame_entries':entries,'reproducible_packing':True,'foot_anchor':[32,48],'whole_body_drawings':True,'unchanged_native_and_rules':unchanged,'packaged_dll_sha256':sha(ROOT/'mods/tfm2_custom_ai/tfm2_custom_ai.dll'),'live_game':'pending at user request','special_native_pose_override':'disabled to avoid adding Animation CC'}
 (OUT/'verification.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
if __name__=='__main__':main()

