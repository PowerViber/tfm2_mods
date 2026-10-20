"""Contact sheets at 1x/2x, every original pose, both facings and terrain contrasts."""
from pathlib import Path
from PIL import Image,ImageDraw,ImageOps,ImageFont
import json
ROOT=Path(__file__).resolve().parents[1];MOD=ROOT/'mods/tfm2_blockcraft';OUT=ROOT/'docs/steve-results'
body=Image.open(MOD/'champions/tfm2_blockcraft_steve#sheet.png').convert('RGBA');poses=json.loads((MOD/'champions/tfm2_blockcraft_steve#anim.fanim').read_text())['anims'];fx=Image.open(MOD/'vfx/steve_eyes#sheet.png').convert('RGBA');effects=json.loads((MOD/'vfx/steve_eyes#anim.fanim').read_text())['anims'];font=ImageFont.load_default(size=12)
def crop(im,r):return im.crop((r['x'],r['y'],r['x']+r['w'],r['y']+r['h']))
sheet=Image.new('RGB',(1024,80+len(poses)*130),'#15232e');d=ImageDraw.Draw(sheet);d.text((16,14),'Approved simpler body / Herobrine overlay / both facings / 1x and 2x',font=font,fill='white');d.text((16,36),'Original anchors and animation timings preserved. Native animation synchronisation needs live observation.',font=font,fill='#9db9c2')
for row,(name,pose)in enumerate(poses.items()):
 y=70+row*130;d.text((12,y+3),name+' / '+str(len(pose['frames']))+' frames',font=font,fill='#d5b878')
 for col,(bg,left,overlay)in enumerate([('#182e35',False,False),('#182e35',False,True),('#182e35',True,True),('#d0cfb4',False,True),('#d0cfb4',True,True)]):
  x=210+col*160;d.rectangle((x,y,x+148,y+119),fill=bg);frame=crop(body,pose['frames'][len(pose['frames'])//2]['data']);
  if overlay:frame.alpha_composite(crop(fx,effects[name+'_r']['frames'][len(pose['frames'])//2]['data']))
  if left:frame=ImageOps.mirror(frame)
  sheet.paste(frame,(x+12,y+62-frame.height//2),frame);large=frame.resize((72,88),Image.Resampling.NEAREST);sheet.paste(large,(x+65,y+18),large)
sheet.save(OUT/'pose-facing-review.png');print('Reviewed',len(poses),'poses in both facings on light/dark backgrounds')
