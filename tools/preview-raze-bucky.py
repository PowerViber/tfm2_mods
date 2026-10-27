"""Review-only Bucky HUD icon; never writes packaged artwork or approval."""
from pathlib import Path
from PIL import Image,ImageDraw
import json,hashlib
root=Path(__file__).resolve().parents[1];out=root/'docs/raze-review/bucky';out.mkdir(parents=True,exist_ok=True)
im=Image.new('RGBA',(40,80));d=ImageDraw.Draw(im)
white='#dde8e5';shade='#829b9a'
d.polygon([(12,3),(15,3),(17,1),(21,1),(21,2),(28,2),(28,3),(21,3),(21,4),(18,4),(17,6),(15,6),(15,4),(12,5)],fill=white)
d.line((20,4,24,4),fill=shade);d.point((27,1),fill=white)
im.save(out/'proposed.png');(out/'proposed.fanim').write_text(json.dumps({'anims':{'gun_bucky':{'frames':[{'duration':.2,'data':{'x':0,'y':0,'w':40,'h':80}}]}}},separators=(',',':'))+'\n')
atlas=Image.open(root/'mods/tfm2_valorant/vfx/valorant#sheet.png').convert('RGBA');meta=json.loads((root/'mods/tfm2_valorant/vfx/valorant#anim.fanim').read_text())['anims']
sheet=Image.new('RGB',(740,250),'#17282b');s=ImageDraw.Draw(sheet);s.text((12,10),'BUCKY HUD ICON - proposal; existing Omen icons retained',fill='#e9e4ce')
for row,bg in enumerate(['#415953','#202f34']):
 yy=32+row*105;s.rectangle((12,yy,726,yy+94),fill=bg)
 for col,(tag,name) in enumerate([('gun_0','Classic'),('gun_bucky','Bucky'),('gun_3','Judge'),('gun_4','Vandal')]):
  x=26+col*176;s.text((x,yy+8),name,fill='#e9e4ce')
  if tag=='gun_bucky':tile=im
  else:
   r=meta[tag]['frames'][0]['data'];tile=atlas.crop((r['x'],r['y'],r['x']+r['w'],r['y']+r['h']))
  box=tile.getbbox();small=tile.crop(box);sheet.paste(small,(x,yy+40),small);big=small.resize((small.width*4,small.height*4),Image.Resampling.NEAREST);sheet.paste(big,(x+35,yy+32),big)
  s.text((x,yy+73),'1x + 4x',fill='#b9cbc5')
sheet.save(out/'comparison.png')
report={'sha256':hashlib.sha256((out/'proposed.png').read_bytes()).hexdigest(),'canvas':[40,80],'visible':list(im.getbbox()),'approval':'pending'}
approval=root/'docs/raze-review/approval.json'
if approval.exists() and json.loads(approval.read_text()).get('bucky_icon_sha256')==report['sha256']:report['approval']='approved'
(out/'review.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report))
