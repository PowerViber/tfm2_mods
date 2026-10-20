"""Verify approved bodies, unchanged legacy art and registered eye animation metadata."""
from pathlib import Path
from PIL import Image,ImageOps
import hashlib,io,json,subprocess
ROOT=Path(__file__).resolve().parents[1];BASE='234caadbc29f49e7aaa66a2de04cb7f4f0998139';OUT=ROOT/'docs/character-tactics-results';OUT.mkdir(exist_ok=True)
def old(rel):return subprocess.check_output(['rtk','proxy','git','show',BASE+':'+rel],cwd=ROOT)
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
approval=json.loads((ROOT/'docs/character-art-review/approval.json').read_text());report={}
for name,pack in [('steve','tfm2_blockcraft'),('omen','tfm2_valorant')]:
    prefix=f'mods/{pack}/champions/{pack}_{name}';p=ROOT/(prefix+'#sheet.png');assert sha(p)==approval[name].get('integratedSha256',approval[name]['candidateSha256']);meta=ROOT/(prefix+'#anim.fanim');assert json.loads(meta.read_bytes())==json.loads(old(str(meta.relative_to(ROOT)).replace('\\','/')))
    before=Image.open(io.BytesIO(old(prefix+'#sheet.png'))).convert('RGBA');after=Image.open(p).convert('RGBA');data=json.loads(meta.read_text())['anims'];assert before.size==after.size
    for pose in data.values():
        for f in pose['frames']:
            r=f['data'];assert(r['w'],r['h'])==(36,44);box=(r['x'],r['y'],r['x']+36,r['y']+44);a=before.crop(box).getbbox();b=after.crop(box).getbbox();assert b and a[3]==b[3]
    decoded=0
    for sheet in (ROOT/'mods'/pack).rglob('*.png'):
        im=Image.open(sheet);assert max(im.size)<=2048;decoded+=im.width*im.height*4
    assert decoded<32*1024*1024;report[name]={'sha256':sha(p),'canvas':[36,44],'frames':sum(len(v['frames'])for v in data.values()),'anchorsAndTimingsPreserved':True,'decodedBytes':decoded}
for rel in ['mods/tfm2_blockcraft/vfx/blockcraft#sheet.png','mods/tfm2_blockcraft/vfx/blockcraft#anim.fanim','mods/tfm2_blockcraft/vfx/steve_craft#sheet.png','mods/tfm2_blockcraft/vfx/steve_craft#anim.fanim']:
    current=(ROOT/rel).read_bytes();baseline=old(rel)
    assert (json.loads(current)==json.loads(baseline) if rel.endswith('.fanim') else current==baseline),rel
eye=Image.open(ROOT/'mods/tfm2_blockcraft/vfx/steve_eyes#sheet.png').convert('RGBA');em=json.loads((ROOT/'mods/tfm2_blockcraft/vfx/steve_eyes#anim.fanim').read_text())['anims'];body=json.loads((ROOT/'mods/tfm2_blockcraft/champions/tfm2_blockcraft_steve#anim.fanim').read_text())['anims']
for pose in body:
    assert [f['duration']for f in em[pose+'_r']['frames']]==[f['duration']for f in body[pose]['frames']]
    for a,b in zip(em[pose+'_r']['frames'],em[pose+'_l']['frames']):
        def crop(f):r=f['data'];return eye.crop((r['x'],r['y'],r['x']+r['w'],r['y']+r['h']))
        assert crop(a).getbbox();assert ImageOps.mirror(crop(a)).tobytes()==crop(b).tobytes()
report['legacyEffectPixelsAndMetadata']='PNG bytes identical; metadata values identical';report['eyeFrames']=31;report['bodyApproval']='Approved the simpler bodies, 2026-10-20';report['liveGame']='pending'
(OUT/'art.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
