"""Verify reproducibility, atlas bounds, unchanged body and original legacy frame storage."""
from pathlib import Path
import hashlib,json,subprocess
from PIL import Image
ROOT=Path(__file__).resolve().parents[1]
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
files=[ROOT/p for p in ['native/tfm2_custom_ai/src/steve_craft_data.rs','editor/steve-data.js','editor/steve-preset.js','mods/tfm2_blockcraft/champion/tfm2_blockcraft_steve.data_champion','mods/tfm2_blockcraft/vfx/steve_craft#sheet.png','mods/tfm2_blockcraft/vfx/steve_craft#anim.fanim']]
before={str(p):sha(p)for p in files}
for tool in ['generate-steve-rules.py','generate-steve-art.py']:subprocess.run(['rtk','proxy','python','-X','utf8',str(ROOT/'tools'/tool)],cwd=ROOT,check=True)
assert before=={str(p):sha(p)for p in files},'Generated files differ'
for name in ['champions/tfm2_blockcraft_steve#anim.fanim','vfx/blockcraft#sheet.png','vfx/blockcraft#anim.fanim']:
 rel='mods/tfm2_blockcraft/'+name;old=subprocess.run(['rtk','proxy','git','show','0161dfa:'+rel],capture_output=True,cwd=ROOT,check=True).stdout;assert (json.loads(old)==json.loads((ROOT/rel).read_bytes()) if rel.endswith('.fanim') else old==(ROOT/rel).read_bytes()),rel
approved=json.loads((ROOT/'docs/character-art-review/approval.json').read_text());assert sha(ROOT/'mods/tfm2_blockcraft/champions/tfm2_blockcraft_steve#sheet.png')==approved['steve']['candidateSha256']
im=Image.open(ROOT/'mods/tfm2_blockcraft/vfx/steve_craft#sheet.png');data=json.loads((ROOT/'mods/tfm2_blockcraft/vfx/steve_craft#anim.fanim').read_text());assert max(im.size)<=2048
for tag,a in data['anims'].items():
 assert len(a['frames'])==8
 for f in a['frames']:
  r=f['data'];assert 0<=r['x']<r['x']+r['w']<=im.width;assert 0<=r['y']<r['y']+r['h']<=im.height;assert im.crop((r['x'],r['y'],r['x']+r['w'],r['y']+r['h'])).getbbox(),tag
for tag in ['eyes','cast_r']:
 right=data['anims'][tag]['frames'][0]['data'];left=data['anims']['eyes_l'if tag=='eyes'else'cast_l']['frames'][0]['data'];crop=lambda r:im.crop((r['x'],r['y'],r['x']+r['w'],r['y']+r['h']));assert crop(right).transpose(Image.Transpose.FLIP_LEFT_RIGHT).tobytes()==crop(left).tobytes()
report={'reproducible':True,'legacyMetadataAndVfxBytesPreserved':True,'mirroredOverlaysVerified':True,'atlas':list(im.size),'frameTags':len(data['anims'])};(ROOT/'docs/steve-results/art-verification.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report))
