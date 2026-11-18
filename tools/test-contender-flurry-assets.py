"""Directional variants reuse approved pixels; validate bounds/lifetimes and legacy art."""
from pathlib import Path
from PIL import Image
import io,json,hashlib,subprocess
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/contender-flurry-review';OUT.mkdir(exist_ok=True)
BASE='e8e99baef3d753dec9012d80fbd94be9793db5ea'
def original(path):return subprocess.check_output(['rtk','proxy','git','show',BASE+':'+path],cwd=ROOT)
png='mods/tfm2_custom/vfx/contender_shadow#sheet.png';meta='mods/tfm2_custom/vfx/contender_shadow#anim.fanim'
before=Image.open(io.BytesIO(original(png))).convert('RGBA');after=Image.open(ROOT/png).convert('RGBA');old=json.loads(original(meta))['anims'];new=json.loads((ROOT/meta).read_text())['anims']
assert max(after.size)<=2048
def crop(sheet,r):return sheet.crop((r['x'],r['y'],r['x']+r['w'],r['y']+r['h']))
for tag,a in old.items():
    assert new[tag]==a,tag
    for f in a['frames']:assert crop(before,f['data']).tobytes()==crop(after,f['data']).tobytes(),tag
hashes=[]
for h in range(8):
    tag=f'flurry_d{h}_t8';a=new[tag];assert len(a['frames'])==4 and abs(sum(f['duration']*60 for f in a['frames'])-8)<1e-7
    for f in a['frames']:
        r=f['data'];assert r['x']+r['w']<=after.width and r['y']+r['h']<=after.height
        im=crop(after,r);box=im.getchannel('A').getbbox();assert box and box[0]>0 and box[1]>0 and box[2]<im.width and box[3]<im.height,tag
    hashes.append(hashlib.sha256(crop(after,a['frames'][2]['data']).tobytes()).hexdigest())
assert len(set(hashes))==8
for suffix in ['#sheet.png','#anim.fanim']:
    path='mods/tfm2_custom/champions/tfm2_custom_contender'+suffix
    assert (ROOT/path).read_bytes().replace(b'\r\n',b'\n')==original(path).replace(b'\r\n',b'\n'),path
champ=json.loads((ROOT/'mods/tfm2_custom/champion/tfm2_custom_contender.data_champion').read_text());names={e['name']:e for e in champ['view_effects']}
assert champ['passive']['params']['shadow_directions_v1']==1
for h in range(8):assert names[f'ct_flurry_d{h}_t8']['tag']==f'flurry_d{h}_t8'
report={'eight_distinct_directions':True,'unclipped':True,'ticks_per_punch':8,'legacy_effect_pixels_and_frames_unchanged':True,'base_body_unchanged':True,'atlas':list(after.size),'contact_hashes':hashes}
(OUT/'directional-art.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
