"""Approved sources -> identical packed outputs; exact effect lifetime contract."""
from pathlib import Path
from PIL import Image
import hashlib,json,subprocess,re,os
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/contender-shadow-review'
paths=list((ROOT/'mods/tfm2_custom/champions').glob('tfm2_custom_contender#*'))+list((ROOT/'mods/tfm2_custom/champions').glob('tfm2_custom_contender_merge#*'))+list((ROOT/'mods/tfm2_custom/vfx').glob('contender_shadow#*'))+[ROOT/'editor/contender-presentation.js',OUT/'presentation.json']
def hashes():return {p.relative_to(ROOT).as_posix():hashlib.sha256(p.read_bytes()).hexdigest() for p in paths}
before=hashes();subprocess.run(['rtk','proxy','python','tools/build-contender-shadow-art.py'],cwd=ROOT,check=True);assert before==hashes()
manifest=json.loads((OUT/'presentation.json').read_text());decoded=0
for group in ['body','effects','merge']:
    p=ROOT/manifest[group]['file'].removeprefix('../');sheet=Image.open(p).convert('RGBA');assert max(sheet.size)<=2048;decoded+=sheet.width*sheet.height*4
    for tag,a in manifest[group]['anims'].items():
        for f in a['frames']:
            r=f['data'];assert min(r.values())>=0 and r['x']+r['w']<=sheet.width and r['y']+r['h']<=sheet.height and f['duration']>0,tag
            im=sheet.crop((r['x'],r['y'],r['x']+r['w'],r['y']+r['h']))
            assert im.getbbox() or tag=='charges0',tag
assert decoded==manifest['decoded_bytes'] and decoded<=32*1024*1024
champ=json.loads((ROOT/'mods/tfm2_custom/champion/tfm2_custom_contender.data_champion').read_text())
for e in manifest['bindings']:
    a=manifest['effects']['anims'][e['tag']];assert abs(sum(f['duration']*60 for f in a['frames'])-e['ticks'])<1e-7,e
for e in champ['view_effects']+champ['view_buffs']:
    group='merge' if e['anim'].endswith('tfm2_custom_contender_merge') else 'effects'
    assert e['tag'] in manifest[group]['anims']
assert champ['passive']['params']['shadow_art_v1']==1 and champ['passive']['params']['shadow_directions_v1']==1
# Keep all legacy body aliases. Four connected attack frames now align with the native 18-tick basic.
assert abs(sum(f['duration']*60 for f in manifest['body']['anims']['attack']['frames'])-18)<1e-7
assert len({json.dumps(f['data'],sort_keys=True) for f in manifest['body']['anims']['run']['frames']})==6
for source,digest in manifest['source_hashes'].items():assert hashlib.sha256((OUT/source).read_bytes()).hexdigest()==digest
comparison=subprocess.run(['rtk','proxy','git','diff','--exit-code',os.environ.get('CONTENDER_BASE','dc0e0b7b2de8a326e80ff9d7fd5d226a9fd423a1'),'--','rules/contender.json','native/tfm2_custom_ai/src/contender_shadow_core.rs','native/tfm2_custom_ai/src/contender_shadow_data.rs'],cwd=ROOT,capture_output=True)
report={'deterministic_packing':True,'generation_deterministic':False,'effect_lifetimes_match':True,'six_unique_movement_frames':True,'combat_core_and_rules_unchanged':comparison.returncode==0,'decoded_bytes':decoded,'hashes':before,'actual_game_acceptance':'pending'}
results=ROOT/os.environ.get('CONTENDER_RESULTS','docs/contender-shadow-review');results.mkdir(exist_ok=True)
(results/'art-verification.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
