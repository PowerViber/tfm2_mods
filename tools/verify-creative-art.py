"""Verify the approved edit, exact armed composition, mirrors and unchanged legacy effects."""
from pathlib import Path
from PIL import Image,ImageOps
import json,hashlib,subprocess
root=Path(__file__).resolve().parents[1];mod=root/'mods/tfm2_blockcraft';review=root/'docs/creative-herobrine-review'
def load(p):return Image.open(p).convert('RGBA')
base=load(mod/'champions/steve_creative_body#sheet.png');old=load(review/'steve-approved-base.png')
assert (mod/'champions/tfm2_blockcraft_steve#sheet.png').read_bytes()==(review/'steve-approved-base.png').read_bytes()
assert (mod/'champions/steve_creative_body#anim.fanim').read_bytes()==(mod/'champions/tfm2_blockcraft_steve#anim.fanim').read_bytes()
atlas=load(mod/'vfx/steve_forms#sheet.png');meta=json.loads((mod/'vfx/steve_forms#anim.fanim').read_text(encoding='utf8'))['anims']
body=json.loads((mod/'champions/tfm2_blockcraft_steve#anim.fanim').read_text(encoding='utf8'))['anims']
def crop(im,r):return im.crop((r['x'],r['y'],r['x']+r['w'],r['y']+r['h']))
frames=0
for pose,a in body.items():
 for n,f in enumerate(a['frames']):
  r=f['data'];b=crop(base,r);weapon=meta['weapon_'+pose+'_r']['frames'][n];left=meta['weapon_'+pose+'_l']['frames'][n]
  assert f['duration']==weapon['duration']==left['duration']
  w=crop(atlas,weapon['data']);assert Image.alpha_composite(b,w).tobytes()==crop(old,r).tobytes()
  assert ImageOps.mirror(w).tobytes()==crop(atlas,left['data']).tobytes()
  assert all(y[3]==0 or x==y for x,y in zip(crop(old,r).get_flattened_data(),b.get_flattened_data()))
  frames+=1
baseline='0365335'
legacy=['champions/tfm2_blockcraft_steve#anim.fanim','vfx/blockcraft#sheet.png','vfx/blockcraft#anim.fanim','vfx/steve_craft#sheet.png','vfx/steve_craft#anim.fanim','vfx/steve_eyes#sheet.png','vfx/steve_eyes#anim.fanim']
for name in legacy:
 p=mod/name;original=subprocess.check_output(['rtk','proxy','git','show',baseline+':'+str(p.relative_to(root)).replace('\\','/')],cwd=root)
 assert (json.loads(p.read_bytes())==json.loads(original) if name.endswith('.fanim') else p.read_bytes()==original),name
files=[mod/'champions/steve_creative_body#sheet.png',mod/'vfx/steve_forms#sheet.png',mod/'vfx/steve_forms#anim.fanim',mod/'champion/tfm2_blockcraft_steve.data_champion',root/'editor/steve-preset.js']
before=[p.read_bytes()for p in files]
subprocess.run(['rtk','proxy','python','-X','utf8',str(root/'tools/generate-steve-form-layers.py')],cwd=root,check=True)
assert before==[p.read_bytes()for p in files]
decoded=0
for p in mod.rglob('*.png'):
 im=Image.open(p);assert max(im.size)<=2048;decoded+=im.width*im.height*4
assert decoded<32*1024*1024
report=dict(approval='Approve original arms without pickaxe',frames=frames,originalArmedCompositionExact=True,newBodyPixels=0,legacyPixelBytesAndAnimationMetadataPreserved=legacy,bothFacings=True,reproducible=True,decodedBytes=decoded,atlas=list(atlas.size),livePoseSynchronization='pending game observation')
(review/'verification.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf8');print(json.dumps(report))
