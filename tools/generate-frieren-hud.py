"""Reproducible compact mana HUD and eight directional spell heads; body artwork is retained."""
from pathlib import Path
import json
from PIL import Image,ImageDraw
ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/'mods/tfm2_frieren/vfx'
sheet=Image.new('RGBA',(256,512));anims={};x=y=0;row=0
def add(tag,im,anchor):
 global x,y,row
 if x+im.width>256:x=0;y+=row+1;row=0
 sheet.paste(im,(x,y));anims[tag]={'frames':[{'duration':.05,'data':{'x':x,'y':y,'w':im.width,'h':im.height}}]};x+=im.width+1;row=max(row,im.height)
for cell in range(21):
 im=Image.new('RGBA',(40,48));d=ImageDraw.Draw(im);d.rectangle((3,0,36,4),fill='#192c32',outline='#718f8f');d.rectangle((4,1,4+int(31*cell/20),3),fill='#a4cdd0');add('mana'+str(cell),im,(20,44))
for direction in range(8):
 im=Image.new('RGBA',(21,9));d=ImageDraw.Draw(im);d.line((0,4,18,4),fill='#80cccb',width=5);d.line((7,4,20,4),fill='#e7fff2',width=3);d.point((20,4),fill='white');im=im.rotate(-45*direction,expand=True,resample=Image.Resampling.NEAREST);add('ray'+str(direction),im,(im.width//2,im.height//2))
sheet=sheet.crop((0,0,256,y+row))
import importlib.util
spec=importlib.util.spec_from_file_location('lossless',ROOT/'tools/repack-frieren-lossless.py');lossless=importlib.util.module_from_spec(spec);spec.loader.exec_module(lossless)
sheet,_=lossless.pack(sheet,{'anims':anims})
sheet.save(OUT/'frieren_rework#sheet.png',optimize=True);(OUT/'frieren_rework#anim.fanim').write_text(json.dumps({'anims':anims},separators=(',',':'))+'\n')
p=ROOT/'mods/tfm2_frieren/champion/tfm2_frieren_frieren.data_champion';c=json.loads(p.read_text())
c['view_buffs']=[b for b in c['view_buffs'] if not b['name'].startswith('fr_mana')]
for cell in range(21):c['view_buffs'].append({'type':'Animated','name':'fr_mana'+str(cell),'anim':'asset/tfm2_frieren/vfx/frieren_rework','tag':'mana'+str(cell),'z':3})
c['view_effects']=[b for b in c['view_effects'] if not b['name'].startswith('tfm2_frieren_ray')]
for direction in range(8):c['view_effects'].append({'type':'Animation','name':'tfm2_frieren_ray'+str(direction),'anim':'asset/tfm2_frieren/vfx/frieren_rework','tag':'ray'+str(direction),'z':2,'is_follow':False})
p.write_text(json.dumps(c,indent=2)+'\n')
print('Frieren HUD:',sheet.width,'x',sheet.height,',',len(anims),'shared compact layers')
