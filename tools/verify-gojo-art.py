"""Compare retained frame pixels/metadata to the branch baseline and repeat generation."""
from pathlib import Path
from PIL import Image,ImageDraw
from io import BytesIO
import hashlib,importlib.util,json,subprocess
ROOT=Path(__file__).resolve().parents[1]
spec=importlib.util.spec_from_file_location('art',ROOT/'tools/generate-gojo-art.py');a=importlib.util.module_from_spec(spec);spec.loader.exec_module(a)
def blob(name):return subprocess.check_output(['rtk','proxy','git','show','a4098f15be65edc683fd62f8037d71b1e79ae9bd:'+name],cwd=ROOT)
rows=[]
for name in ['champions/tfm2_custom_gojo','vfx/gojo']:
 p='mods/tfm2_jjk/'+name;before=a.frame_hashes(Image.open(BytesIO(blob(p+'#sheet.png'))).convert('RGBA'),json.loads(blob(p+'#anim.fanim')));after=a.frame_hashes(Image.open(ROOT/(p+'#sheet.png')).convert('RGBA'),json.loads((ROOT/(p+'#anim.fanim')).read_text()));assert before==after;rows.append({'asset':name,'frames':len(before),'equalToMainBaseline':True})
files=[*list((ROOT/'mods/tfm2_jjk').rglob('*#sheet.png')),*list((ROOT/'mods/tfm2_jjk').rglob('*#anim.fanim')),ROOT/'mods/tfm2_jjk/champion/tfm2_custom_gojo.data_champion',ROOT/'editor/gojo-preset.js',ROOT/'editor/gojo-data.js']
before={str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in files}
for script in ['generate-gojo-rules.py','generate-gojo-art.py','generate-gojo-champion.py']:subprocess.run(['rtk','proxy','python','tools/'+script],cwd=ROOT,check=True,capture_output=True)
assert before=={str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in files}
old=json.loads(blob('mods/tfm2_jjk/champion/tfm2_custom_gojo.data_champion'));new=json.loads((ROOT/'mods/tfm2_jjk/champion/tfm2_custom_gojo.data_champion').read_text());assert all(old[k]==new[k] for k in ['id','stat','growth','sprite','anim_prefix'])
assert new['passive']['passive_ref']=='tfm2_custom_ai:gojo';assert new['attack']['effect']=={'type':'Native','effect_ref':'tfm2_custom_ai:gojo_basic'}
body=Image.open(ROOT/'mods/tfm2_jjk/champions/tfm2_custom_gojo#sheet.png').convert('RGBA');bm=json.loads((ROOT/'mods/tfm2_jjk/champions/tfm2_custom_gojo#anim.fanim').read_text());fx=Image.open(ROOT/'mods/tfm2_jjk/vfx/gojo_rework#sheet.png').convert('RGBA');fm=json.loads((ROOT/'mods/tfm2_jjk/vfx/gojo_rework#anim.fanim').read_text())
def frame(sheet,meta,tag):
 f=meta['anims'][tag]['frames'][0]['data'];return sheet.crop((f['x'],f['y'],f['x']+f['w'],f['y']+f['h']))
contact=Image.new('RGBA',(768,400),(12,22,38,255));draw=ImageDraw.Draw(contact)
for facing in range(2):
 for k in range(6):
  x=64+128*k;y=100+180*facing;im=frame(body,bm,'skill1' if k<3 else 'skill2');im=im.transpose(Image.Transpose.FLIP_LEFT_RIGHT) if facing else im;contact.alpha_composite(im,(x-im.width//2,y-im.height//2));effect=frame(fx,fm,'fx'+str(k+1));effect=effect.transpose(Image.Transpose.FLIP_LEFT_RIGHT) if facing else effect;contact.alpha_composite(effect,(x-effect.width//2,y-effect.height//2));draw.text((x-42,y+55),('Left' if facing else 'Right')+' / '+str(k+1),fill='white')
contact.save(ROOT/'docs/gojo-results/pose-facings.png')
report={'baselineFrames':rows,'generationReproducible':True,'statsGrowthAndBodyIdentityUnchanged':True,'singleNativeBasicEffect':True};(ROOT/'docs/gojo-results/art-verification.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report))
