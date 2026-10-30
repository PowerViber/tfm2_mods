"""Verify generated playtest data/art and repeatable extraction without invoking imagegen."""
from pathlib import Path
import json,hashlib,subprocess
from PIL import Image
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/contender-results'
paths=[ROOT/'mods/tfm2_custom/champion/tfm2_custom_contender.data_champion',ROOT/'editor/boxing-preset.js']+list((ROOT/'mods/tfm2_custom/champions').glob('tfm2_custom_contender#*'))+list((ROOT/'mods/tfm2_custom/vfx').glob('contender#*'))
def hashes():return {p.relative_to(ROOT).as_posix():hashlib.sha256(p.read_bytes()).hexdigest() for p in paths}
before=hashes()
for script in ['prepare-contender-art.py','generate-contender.py','generate-roster.py']:
    subprocess.run(['rtk','proxy','python',str(ROOT/'tools'/script)],cwd=ROOT,check=True,capture_output=True)
assert before==hashes(),'Generation is not reproducible'
decoded=0
for p in paths:
    if p.suffix!='.fanim':continue
    sheet=Image.open(str(p).replace('#anim.fanim','#sheet.png')).convert('RGBA');assert max(sheet.size)<=2048;decoded+=sheet.width*sheet.height*4
    animations=json.loads(p.read_text())['anims']
    for name,anim in animations.items():
        for f in anim['frames']:
            r=f['data'];assert r['x']>=0 and r['y']>=0 and r['x']+r['w']<=sheet.width and r['y']+r['h']<=sheet.height
            assert f['duration']>0
            assert sheet.crop((r['x'],r['y'],r['x']+r['w'],r['y']+r['h'])).getbbox(),name
assert decoded<=32*1024*1024
d=json.loads(paths[0].read_text());assert d['attack']['effect']['effect_ref']=='tfm2_custom_ai:contender_basic'
anims=json.loads((ROOT/'mods/tfm2_custom/champions/tfm2_custom_contender#anim.fanim').read_text())['anims']
for name in ['idle','run','attack','combination','bodyShot','uppercut','slip','guard','mainEvent','hit','dead']:
    assert name in anims and name+'_l' in anims
fx=json.loads((ROOT/'mods/tfm2_custom/vfx/contender#anim.fanim').read_text())['anims']
for v in d['view_effects']+d['view_buffs']:assert v['tag'] in fx
report={'reproducibleExtractionAndPacking':True,'aiGenerationDeterministic':False,'approved':'boxer and ring; user approved both','decodedBytes':decoded,'atlasMaximum':2048,'defaultBodyVisible':True,'bothFacings':True,'generatedHashes':before,'nativeVisualAcceptance':'pending'}
OUT.mkdir(exist_ok=True,parents=True);(OUT/'art-verification.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
