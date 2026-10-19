"""Native artwork bounds, actual committed geometry and frozen-v2 byte identity."""
from pathlib import Path
import json,hashlib,subprocess
from PIL import Image
import gojo_legacy_art as art
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/gojo-results'
sheet=Image.open(ROOT/'mods/tfm2_jjk/vfx/gojo_v2#sheet.png').convert('RGBA');meta=json.loads((ROOT/'mods/tfm2_jjk/vfx/gojo_v2#anim.fanim').read_text())
def frame(tag,n=0):
 r=meta['anims'][tag]['frames'][n]['data'];return sheet.crop((r['x'],r['y'],r['x']+r['w'],r['y']+r['h']))
def extent(im):
 b=im.getbbox();return max(b[2]-b[0],b[3]-b[1]) if b else 0
rows=[]
for mode,expand,radius in [('full',60,64000),('quick',12,96000)]:
 for tick in range(expand):
  im=frame(f'void_{mode}_expand{tick}');expected=round(2*(8000+(radius-8000)*tick//expand)/960)
  assert extent(im)<=expected+1,(mode,tick,extent(im),expected)
  rows.append({'mode':mode,'tick':tick,'diameterPixels':expected,'visiblePixels':extent(im)})
for tag,diameter in [('blue_impact',round(64000/960)),('max_blue_impact',75),('red_impact',round(40000/960)),('max_red_impact',round(52000/960)),('purple_flight',round(52000/960))]:
 for n in range(8):assert extent(frame(tag,n))<=diameter+1,(tag,n)
base='d0b08660813e5bc8178c8a7cb88f0f9528eb51b2'
expected=subprocess.check_output(['rtk','proxy','git','show',base+':editor/gojo.wasm'],cwd=ROOT)
assert expected==(ROOT/'editor/gojo-v2.wasm').read_bytes()
report={'nativeDomainExpansionFrames':len(rows),'rows':rows,'projectileAndImpactBounds':True,'frozenV2WasmSha256':hashlib.sha256(expected).hexdigest(),'scope':'Native pixel bounds at 960 world units/pixel. Actual host projection, anchoring and crowded rendering remain unobserved.'}
(OUT/'v3-geometry.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({k:v for k,v in report.items() if k!='rows'}))
