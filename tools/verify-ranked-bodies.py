"""Check visible bodies, exact timing/anchors, legacy aliases and frozen FX."""
from pathlib import Path
import hashlib,io,json,subprocess
from PIL import Image,ImageOps
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/ranked-body-redesign'
sha=lambda b:hashlib.sha256(b).hexdigest()
read=lambda p:json.loads(p.read_text(encoding='utf8'))
manifest=read(ROOT/'rules/ranked-body-art.json');frozen=read(OUT/'baseline-contract.json')
proof=read(OUT/'base-animation-results.json');fallback=read(OUT/'fallback-integration.json')
changed={r['asset']for r in fallback['retired']}|{c['base']for c in manifest['characters']}
frame_hashes={};body_bytes={};count=0;frozen_count=0;preserved_count=0
for key,c in [(c['key'],c)for c in manifest['characters']]:
 p=ROOT/c['base'];image=Image.open(str(p)+'#sheet.png').convert('RGBA');meta=read(Path(str(p)+'#anim.fanim'))['anims']
 old=frozen['characters'][key]['body']['anims'];assert meta.keys()==old.keys()
 assert sha(Path(str(p)+'#sheet.png').read_bytes())==proof['characters'][key]['pngSha256']
 body_bytes[key]=image.width*image.height*4;assert max(image.size)<=2048 and body_bytes[key]<=32*2**20
 offsets=set();frame_hashes[key]={}
 for tag,anim in meta.items():
  assert [f['duration']for f in anim['frames']]==[f['duration']for f in old[tag]['frames']]
  frame_hashes[key][tag]=[]
  for i,f in enumerate(anim['frames']):
   r=f['data'];assert r['x']>=0 and r['y']>=0 and r['x']+r['w']<=image.width and r['y']+r['h']<=image.height
   im=image.crop((r['x'],r['y'],r['x']+r['w'],r['y']+r['h']));b=im.getbbox();assert b,(key,tag,i,'blank body')
   assert b[0]>=1 and b[1]>=1 and b[2]<im.width and b[3]<im.height,(key,tag,i,'clipping')
   offsets.add(b[3]-im.height/2);count+=1
   frame_hashes[key][tag].append({'size':list(im.size),'right':sha(im.tobytes()),'left':sha(ImageOps.mirror(im).tobytes())})
  assert len(set(f['right']for f in frame_hashes[key]['run']))==len(meta['run']['frames']) if tag=='run' else True
 assert offsets=={proof['characters'][key]['footOffset']},(key,offsets)
 data=read(ROOT/'mods/tfm2_custom/champion'/f"{c['id']}.data_champion")
 assert data['sprite']==c['base'].replace('mods/','asset/',1)
 assert data['passive']['params'].pop('rank_art_v2')==1
 before=json.loads(subprocess.check_output(['rtk','proxy','git','show',manifest['baseline']+':mods/tfm2_custom/champion/'+c['id']+'.data_champion'],cwd=ROOT))
 assert data==before,(key,'combat definition')
for base,a in frozen['assets'].items():
 if base in changed:continue
 assert sha((ROOT/a['png']).read_bytes())==a['sha256'],base
 assert sha((ROOT/a['animation']).read_bytes())==a['animationSha256'],base
 frozen_count+=1
for item in fallback['retired']:
 p=ROOT/item['asset'];im=Image.open(str(p)+'#sheet.png').convert('RGBA');anims=read(Path(str(p)+'#anim.fanim'))['anims']
 assert max(im.size)<=2048
 original=json.loads(subprocess.check_output(['rtk','proxy','git','show',manifest['baseline']+':'+item['asset']+'#anim.fanim'],cwd=ROOT))['anims']
 for tag,anim in anims.items():
  assert [f['duration']for f in anim['frames']]==[f['duration']for f in original[tag]['frames']]
  for i,f in enumerate(anim['frames']):
   r=f['data'];a=im.crop((r['x'],r['y'],r['x']+r['w'],r['y']+r['h']))
   if item['asset'].endswith('coder_rank') and not tag.startswith('fit'):continue
   assert not a.getbbox(),(item['asset'],tag,i,'legacy body leakage')
 for p in item['preservedFrames']:
  r=anims[p['tag']]['frames'][p['frame']]['data'];a=im.crop((r['x'],r['y'],r['x']+r['w'],r['y']+r['h']))
  if 'translation'in p:
   restore=Image.new('RGBA',a.size);restore.paste(a,tuple(-n for n in p['translation']));a=restore
  assert sha(a.tobytes())==p['sha256'],('independent frame',p['tag']);preserved_count+=1
dll_sha=sha((ROOT/'mods/tfm2_custom_ai/tfm2_custom_ai.dll').read_bytes());assert dll_sha==frozen['nativeDllSha256']
unchanged=subprocess.check_output(['rtk','proxy','git','diff',manifest['baseline'],'--name-only','--','native','editor/*.wasm'],cwd=ROOT).decode().strip();assert not unchanged,unchanged
review=read(OUT/'review-data.json');magnus=next(c for c in review['characters']if c['key']=='magnus');assert magnus['declaredDecodedBytes']<=32*2**20
(OUT/'frame-pixel-oracle.json').write_text(json.dumps(frame_hashes,separators=(',',':'))+'\n')
report={'bodyFrames':count,'bothFacings':True,'fixedFootOffsets':True,'allRunFramesDistinct':True,'unchangedWholeAssets':frozen_count,'preservedMixedAtlasFrames':preserved_count,'decodedBodyBytes':body_bytes,'magnusCompleteBytes':magnus['declaredDecodedBytes'],'nativeDllSha256':dll_sha,'nativeAndWasmSourcesUnchanged':True,'championMechanicsUnchanged':True,'rankAnimationsComplete':False,'nativeRankMode':'visible-base-fallback','liveAcceptance':'pending'}
(OUT/'art-verification.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
