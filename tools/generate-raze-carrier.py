"""Add an owner-requested blank pose; retain every original Alchemist frame.

No entity invisibility, targeting change, global body replacement, or sheet
override. Only Raze's satchel entities request the additional animation tag.
Actual host pose/targeting acceptance is reported separately from source checks.
"""
from pathlib import Path
from PIL import Image
import json
root=Path(__file__).resolve().parents[1]
source=root/'Sprite kit/base champions/alchemist.anim.json'
original=json.loads(source.read_text())
assert Image.open(root/'Sprite kit/base champions/alchemist.png').getpixel((0,0))[3]==0
out=json.loads(json.dumps(original))
out['anims']['rz_satchel_hidden']={'frames':[{'duration':.05,'data':{'x':0,'y':0,'w':1,'h':1}}]}
assert all(out['anims'][tag]==ani for tag,ani in original['anims'].items())
p=root/'mods/tfm2_valorant/champions/raze_carrier#anim.fanim'
p.write_text(json.dumps(out,separators=(',',':'))+'\n')
p=root/'mods/tfm2_valorant/mod.override_info';d=json.loads(p.read_text())
d['asset/base/aseprite_resources/champions/alchemist#anim']={'remapping':'asset/tfm2_valorant/champions/raze_carrier#anim','type':'override'}
p.write_text(json.dumps(d,indent=2)+'\n')
print('Alchemist original animations preserved; added owner-only satchel pose. Live acceptance pending.')
