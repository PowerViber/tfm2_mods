"""Pack the approved original-arm edit; reconstruct armed Steve pixel for pixel.

The approval covers removing the original weapon only. No body pixels are drawn.
The fourth charge indicator copies the established charge-pip pixels.
"""
from pathlib import Path
from PIL import Image, ImageOps
import hashlib, json, shutil

ROOT = Path(__file__).resolve().parents[1]
REVIEW = ROOT / 'docs/creative-herobrine-review'
MOD = ROOT / 'mods/tfm2_blockcraft'
BODY = MOD / 'champions/steve_creative_body#sheet.png'
SOURCE = REVIEW / 'steve-approved-base.png'
if not SOURCE.exists():
    shutil.copyfile(MOD / 'champions/tfm2_blockcraft_steve#sheet.png', SOURCE)
original = Image.open(SOURCE).convert('RGBA')
candidate_path = REVIEW / 'weapon-removal-candidate.png'
assert hashlib.sha256(candidate_path.read_bytes()).hexdigest() == '64c80b1dcb191ab8ed6c9bd13e00d9658d937d6f9a9b30ebf2a437b9777a51e3'
body = Image.open(candidate_path).convert('RGBA')
assert original.size == body.size
assert all(b[3] == 0 or b == a for a, b in zip(original.get_flattened_data(), body.get_flattened_data()))
weapon = Image.new('RGBA', body.size)
weapon.putdata([a if a != b else (0, 0, 0, 0) for a, b in zip(original.get_flattened_data(), body.get_flattened_data())])
assert Image.alpha_composite(body, weapon).tobytes() == original.tobytes()
meta = json.loads((MOD / 'champions/tfm2_blockcraft_steve#anim.fanim').read_text(encoding='utf8'))
atlas = Image.new('RGBA', (512, 512))
animations = {}; dedup = {}; x = y = row = 0
def pack(im):
    global x, y, row
    key = (im.size, im.tobytes())
    if key in dedup:
        return dedup[key]
    if x + im.width > atlas.width:
        x = 0; y += row + 1; row = 0
    assert y + im.height <= atlas.height
    rect = dict(x=x, y=y, w=im.width, h=im.height)
    atlas.paste(im, (x, y)); dedup[key] = rect
    x += im.width + 1; row = max(row, im.height)
    return rect
for pose, anim in meta['anims'].items():
    for facing in ['r', 'l']:
        frames = []
        for f in anim['frames']:
            r = f['data']; im = weapon.crop((r['x'], r['y'], r['x']+r['w'], r['y']+r['h']))
            if facing == 'l': im = ImageOps.mirror(im)
            frames.append(dict(duration=f['duration'], data=pack(im)))
        animations[f'weapon_{pose}_{facing}'] = dict(frames=frames)
# Reuse the exact original pip colours, outline and dimensions, with four slots.
craft = Image.open(MOD / 'vfx/steve_craft#sheet.png').convert('RGBA')
cm = json.loads((MOD / 'vfx/steve_craft#anim.fanim').read_text(encoding='utf8'))['anims']
def hud(name):
    r = cm[name]['frames'][0]['data']
    return craft.crop((r['x'], r['y'], r['x']+r['w'], r['y']+r['h']))
pips = [hud(name).crop((6, 2, 12, 7)) for name in ['hud_1_0_0','hud_1_1_0','hud_1_0_1']]
for ready in range(5):
    for reserved in range(5-ready):
        im = hud('hud_1_0_0'); im.paste((0,0,0,0), (0,0,36,9))
        for n in range(4): im.paste(pips[1 if n<ready else 2 if n<ready+reserved else 0], (3+n*8,2))
        animations[f'hud_1_{ready}_{reserved}'] = dict(frames=[dict(duration=.08,data=pack(im)) for _ in range(8)])
body.save(BODY)
shutil.copyfile(MOD / 'champions/tfm2_blockcraft_steve#anim.fanim', MOD / 'champions/steve_creative_body#anim.fanim')
atlas = atlas.crop((0,0,512,y+row+1))
atlas.save(MOD / 'vfx/steve_forms#sheet.png')
(MOD / 'vfx/steve_forms#anim.fanim').write_text(json.dumps(dict(anims=animations),separators=(',',':'))+'\n',encoding='utf8')
path = MOD / 'champion/tfm2_blockcraft_steve.data_champion'
data = json.loads(path.read_text(encoding='utf8'))
data['sprite']='asset/tfm2_blockcraft/champions/steve_creative_body'
data['view_buffs'] = [v for v in data['view_buffs'] if not v['name'].startswith(('sc_weapon_', 'sc_hud_1_'))]
data['view_buffs'] += [dict(type='Animated',name='sc_'+tag,anim='asset/tfm2_blockcraft/vfx/steve_forms',tag=tag,z=4) for tag in animations]
data['view_effects']=[v for v in data['view_effects'] if not v['name'].startswith('tfm2_blockcraft_weapon_dead_')]
data['view_effects'] += [dict(type='Animation',name='tfm2_blockcraft_weapon_dead_'+f,anim='asset/tfm2_blockcraft/vfx/steve_forms',tag='weapon_dead_'+f,z=4,is_follow=True) for f in ['r','l']]
path.write_text(json.dumps(data,indent=2)+'\n',encoding='utf8')
(ROOT/'editor/steve-preset.js').write_text('window.STEVE_PRESET='+json.dumps(data,separators=(',',':'))+';\n',encoding='utf8')
review = json.loads((REVIEW/'weapon-removal-review.json').read_text(encoding='utf8'))
review.update(approval='Approve original arms without pickaxe', approvedDate='2026-10-21', runtimeUnchanged=False, armedSteveReconstructedExactly=True, originalBodySha256=hashlib.sha256(SOURCE.read_bytes()).hexdigest(), atlas=list(atlas.size))
(REVIEW/'weapon-removal-review.json').write_text(json.dumps(review,indent=2)+'\n',encoding='utf8')
print('Approved unarmed body integrated; original armed Steve reconstructs exactly; 4-charge HUD reuses original pixels.')
