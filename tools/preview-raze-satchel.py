"""Review-only compact satchel assets. Never writes a game asset or approval."""
from pathlib import Path
from PIL import Image, ImageDraw
import hashlib, json

root=Path(__file__).resolve().parents[1]
out=root/'docs/raze-review/sticky-satchel'
out.mkdir(parents=True,exist_ok=True)
atlas=Image.new('RGBA',(12*8,10*5))
anims={}
for row,tag in enumerate(['ground','north','east','south','west']):
 frames=[]
 for k in range(8):
  im=Image.new('RGBA',(12,10));d=ImageDraw.Draw(im)
  d.rectangle((1,1,10,8),fill='#141e20')
  d.rectangle((2,2,9,7),fill='#a8873e')
  d.line((2,2,9,2),fill='#f8ad47')
  d.rectangle((4,2,5,7),fill='#34393b')
  d.line((2,5,9,5),fill='#e37c2d')
  d.point((8,3),fill='#83cfa5' if k<4 else '#ffd478')
  if tag=='north':d.line((2,1,9,1),fill='#d4d9b2')
  if tag=='south':d.line((2,8,9,8),fill='#d4d9b2')
  if tag=='east':d.line((10,2,10,7),fill='#d4d9b2')
  if tag=='west':d.line((1,2,1,7),fill='#d4d9b2')
  atlas.paste(im,(k*12,row*10))
  frames.append(dict(duration=.05,data=dict(x=k*12,y=row*10,w=12,h=10)))
 anims['satchel_'+tag]=dict(frames=frames)
atlas.save(out/'proposed.png')
(out/'proposed.fanim').write_text(json.dumps(dict(anims=anims),separators=(',',':'))+'\n')
body=Image.open(root/'mods/tfm2_valorant/champions/tfm2_valorant_raze#sheet.png').convert('RGBA')
meta=json.loads((root/'mods/tfm2_valorant/champions/tfm2_valorant_raze#anim.fanim').read_text())
r=meta['anims']['idle']['frames'][0]['data'];raze=body.crop((r['x'],r['y'],r['x']+r['w'],r['y']+r['h']))
frames=[]
for k in range(8):
 sheet=Image.new('RGB',(860,320),'#182529');d=ImageDraw.Draw(sheet)
 d.text((12,8),'COMPACT SATCHEL - proposed, not integrated',fill='#f4e7c5')
 for row,bg in enumerate(['#c8c7a4','#263a34']):
  yy=32+row*142
  for col,tag in enumerate(['ground','north','east','south','west']):
   xx=12+col*168;d.rectangle((xx,yy,xx+158,yy+132),fill=bg)
   d.text((xx+6,yy+5),tag,fill='#121e20' if row==0 else '#f4e7c5')
   sprite=atlas.crop((k*12,col*10,k*12+12,col*10+10))
   sheet.paste(raze,(xx+6,yy+43),raze);sheet.paste(sprite,(xx+49,yy+71),sprite)
   enlarged=sprite.resize((48,40),Image.Resampling.NEAREST);sheet.paste(enlarged,(xx+81,yy+48),enlarged)
   d.text((xx+6,yy+111),'1x + 4x detail',fill='#121e20' if row==0 else '#f4e7c5')
 frames.append(sheet)
frames[0].save(out/'comparison.png')
frames[0].save(out/'comparison.gif',save_all=True,append_images=frames[1:],duration=50,loop=0)
report=dict(canvas=[12,10],visible=[10,8],atlasSha256=hashlib.sha256((out/'proposed.png').read_bytes()).hexdigest(),approval='pending',nativeCarrierAcceptance='Pending actual host check of owner-only animation pose; no entity invisibility or changes to original Alchemist frames')
approval=root/'docs/raze-review/approval.json'
if approval.exists() and json.loads(approval.read_text()).get('compact_satchel_sha256')==report['atlasSha256']:
 report['approval']='approved: Approve the smaller satchel'
(out/'review.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report))
