"""Verify approved art, shared definitions, asset bindings and reproducible generators."""
from pathlib import Path
from PIL import Image
import json,hashlib,subprocess
root=Path(__file__).resolve().parents[1];out=root/'docs/raze-results';out.mkdir(parents=True,exist_ok=True)
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
approval=json.loads((root/'docs/raze-review/approval.json').read_text())
assert sha(root/'docs/raze-review/raze-generated-source.png')==approval['body_source_sha256']
assert sha(root/'docs/raze-review/effects-approved-base.png')==approval['effects_atlas_sha256']
assert sha(root/'mods/tfm2_valorant/vfx/raze#sheet.png')==approval.get('effects_v32_atlas_sha256',approval.get('effects_v3_atlas_sha256',approval.get('effects_scaled_atlas_sha256',approval.get('effects_derived_atlas_sha256',approval['effects_atlas_sha256']))))
base=Image.open(root/'docs/raze-review/effects-approved-base.png');base_meta=json.loads((root/'docs/raze-review/effects-approved-base.fanim').read_text());current=Image.open(root/'mods/tfm2_valorant/vfx/raze#sheet.png');current_meta=json.loads((root/'mods/tfm2_valorant/vfx/raze#anim.fanim').read_text())
for tag,a in base_meta['anims'].items():
 for n,f in enumerate(a['frames']):
  r=f['data'];q=current_meta['anims'][tag]['frames'][n]['data'];assert base.crop((r['x'],r['y'],r['x']+r['w'],r['y']+r['h'])).tobytes()==current.crop((q['x'],q['y'],q['x']+q['w'],q['y']+q['h'])).tobytes()
body=root/'mods/tfm2_valorant/champions/tfm2_valorant_raze#sheet.png';meta=json.loads(body.with_name('tfm2_valorant_raze#anim.fanim').read_text())
im=Image.open(body);bounds=[]
for name,a in meta['anims'].items():
 for f in a['frames']:
  r=f['data'];assert (r['w'],r['h'])==(36,44);assert r['x']>=0 and r['y']>=0 and r['x']+r['w']<=im.width and r['y']+r['h']<=im.height
 bounds.append(name)
old=json.loads((root/'mods/tfm2_valorant/champions/tfm2_valorant_omen#anim.fanim').read_text())
for name,a in old['anims'].items():assert [f['duration'] for f in a['frames']]==[f['duration'] for f in meta['anims'][name]['frames']]
champ=json.loads((root/'mods/tfm2_valorant/champion/tfm2_valorant_raze.data_champion').read_text());assert champ['category']=='Range';assert champ['passive']['passive_ref']=='tfm2_custom_ai:raze';assert champ['attack']==json.loads((root/'mods/tfm2_valorant/champion/tfm2_valorant_omen.data_champion').read_text())['attack']
omen=json.loads((root/'mods/tfm2_valorant/champion/tfm2_valorant_omen.data_champion').read_text())
assert champ['view_buffs']==[v for v in omen['view_buffs'] if v['name'].startswith(('omn_gun','omn_arm','omn_cr','omn_cash'))]
raze_buy=next(v for v in champ['view_effects'] if v['name']=='tfm2_valorant_raze_buy').copy();omen_buy=next(v for v in omen['view_effects'] if v['name']=='tfm2_valorant_omen_buy').copy();raze_buy.pop('name');omen_buy.pop('name');assert raze_buy==omen_buy
for v in champ['view_effects']+champ['view_buffs']:
 a=root/'mods'/v['anim'].removeprefix('asset/');m=json.loads(a.with_name(a.name+'#anim.fanim').read_text());assert v['tag'] in m['anims'],v
decoded=0;atlases=[]
for p in (root/'mods/tfm2_valorant').rglob('*#sheet.png'):
 image=Image.open(p);assert max(image.size)<=2048,(p,image.size);decoded+=image.width*image.height*4;atlases.append({'path':p.relative_to(root).as_posix(),'size':image.size})
assert decoded<=32*1024*1024
outputs=[root/p for p in ['native/tfm2_custom_ai/src/raze_data.rs','native/tfm2_custom_ai/src/valorant_gun_data.rs','editor/raze-data.js','editor/raze-preset.js','mods/tfm2_valorant/champion/tfm2_valorant_raze.data_champion','mods/tfm2_valorant/champions/tfm2_valorant_raze#sheet.png','mods/tfm2_valorant/champions/tfm2_valorant_raze#anim.fanim','mods/tfm2_valorant/vfx/raze#sheet.png','mods/tfm2_valorant/vfx/raze#anim.fanim','rules/champions.json']]
outputs += [root/'mods/tfm2_valorant/champions/raze_carrier#anim.fanim',root/'mods/tfm2_valorant/mod.override_info']
original=json.loads((root/'Sprite kit/base champions/alchemist.anim.json').read_text())
carrier=json.loads((root/'mods/tfm2_valorant/champions/raze_carrier#anim.fanim').read_text())
assert all(carrier['anims'][tag]==a for tag,a in original['anims'].items())
assert carrier['anims']['rz_satchel_hidden']['frames'][0]['data']==dict(x=0,y=0,w=1,h=1)
assert Image.open(root/'Sprite kit/base champions/alchemist.png').getpixel((0,0))[3]==0
assert sha(root/'docs/raze-review/sticky-satchel/proposed.png')==approval['compact_satchel_sha256']
assert sha(root/'docs/raze-review/round-bot/proposed.png')==approval['round_bot_sha256']
assert carrier['anims']['rz_bot_hidden']==carrier['anims']['rz_satchel_hidden']
proposed=Image.open(root/'docs/raze-review/round-bot/proposed.png')
proposed_meta=json.loads((root/'docs/raze-review/round-bot/proposed.fanim').read_text())
for tag,a in proposed_meta['anims'].items():
 for n,f in enumerate(a['frames']):
  r=f['data'];actual=current_meta['anims'][tag]['frames'][n];q=actual['data']
  assert (q['w'],q['h'])==(18,18) and actual['duration']==f['duration']
  assert proposed.crop((r['x'],r['y'],r['x']+r['w'],r['y']+r['h'])).tobytes()==current.crop((q['x'],q['y'],q['x']+q['w'],q['y']+q['h'])).tobytes()
for tag in ['satchel_ground','satchel_north','satchel_east','satchel_south','satchel_west']:
 assert all((f['data']['w'],f['data']['h'])==(12,10) for f in current_meta['anims'][tag]['frames'])
before={str(p):sha(p) for p in outputs}
for args in [['tools/generate-raze.py'],['tools/generate-raze-presentation.py'],['tools/generate-raze-carrier.py'],['tools/prepare-raze-art.py','--integrate','--approved-sha256',approval['body_source_sha256']],['tools/generate-roster.py','--check']]:
 subprocess.run(['rtk','proxy','python','-X','utf8',*args],cwd=root,check=True,capture_output=True)
after={str(p):sha(p) for p in outputs}
assert before==after,'Generation changed outputs: '+', '.join(p for p in before if before[p]!=after[p])
baseline=subprocess.run(['rtk','proxy','git','show','ba8734b:mods/tfm2_valorant/vfx/valorant#sheet.png'],cwd=root,capture_output=True,check=True).stdout
assert hashlib.sha256(baseline).hexdigest()==sha(root/'mods/tfm2_valorant/vfx/valorant#sheet.png'),'Existing Omen pixels changed'
report={'approved':True,'bodySourceSha256':approval['body_source_sha256'],'framesAndTimingPreserved':True,'nativeCanvas':[36,44],'poses':bounds,'decodedValorantBytes':decoded,'atlases':atlases,'reproducible':True,'existingOmenPixelsUnchanged':True,'liveAcceptance':'Pending carrier coverage, body/height alignment, warnings, targeting and attribution in the actual game.'}
(out/'art.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8');print(json.dumps(report))
