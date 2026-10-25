"""Deterministic code-native Raze warnings, icons and compact effect animation.

Body pixels come exclusively from the separately approved image source. This tool
does not modify any existing Omen pixels or timings.
"""
from pathlib import Path
from PIL import Image,ImageDraw
import json,math,hashlib
root=Path(__file__).resolve().parents[1];mod=root/'mods/tfm2_valorant';review=root/'docs/raze-review';review.mkdir(exist_ok=True,parents=True)
atlas=Image.new('RGBA',(1024,1024));anims={};x=y=row=0
def pack(im):
 global x,y,row
 if x+im.width>1024:x=0;y+=row+1;row=0
 assert y+im.height<=1024
 r=dict(x=x,y=y,w=im.width,h=im.height);atlas.paste(im,(x,y));x+=im.width+1;row=max(row,im.height);return r
def effect(tag,size,kind,radius=0):
 frames=[]
 for k in range(8):
  im=Image.new('RGBA',size);d=ImageDraw.Draw(im);cx,cy=size[0]//2,size[1]//2;t=k/7
  if kind=='warning':
   r=radius;colour=(255,180 if k%2 else 220,69,160+k*10)
   d.ellipse((cx-r,cy-r,cx+r,cy+r),outline=colour,width=1)
   for a in range(0,360,45):
    q=math.radians(a+k*3);rr=max(2,int(r*(.25+.65*t)));px=round(cx+math.cos(q)*rr);py=round(cy+math.sin(q)*rr);d.rectangle((px,py,px+1,py+1),fill=colour)
  elif kind=='blast':
   r=max(2,round(radius*(.3+.7*t)));alpha=round(255*(1-t*.85));d.ellipse((cx-r,cy-r,cx+r,cy+r),outline=(255,178,56,alpha),width=2)
   if k<4:d.ellipse((cx-5,cy-5,cx+5,cy+5),fill=(255,238,152,alpha))
   for n in range(8):
    a=n*math.pi/4;px=round(cx+math.cos(a)*r*.8);py=round(cy+math.sin(a)*r*.8);d.rectangle((px-1,py-1,px+1,py+1),fill=(235,101,36,alpha))
  elif kind=='satchel':
   # An opaque owner-only cover for the supported Alchemist carrier. Live size is a review gate.
   d.rounded_rectangle((1,1,30,38),radius=5,fill='#263631',outline='#141e20',width=2);d.rectangle((4,7,27,30),fill='#a8873e',outline='#f8ad47',width=2);d.rectangle((14,5,17,33),fill='#34393b');d.rectangle((6,15,25,19),fill='#e37c2d');d.rectangle((20,8,23,11),fill='#83cfa5' if k%2 else '#ffd478')
  elif kind=='bot':
   d.rounded_rectangle((0,0,31,39),radius=6,fill='#22302f');d.rectangle((2,12,29,31),fill='#bf7a2e',outline='#1b2020',width=2);d.rectangle((5,16,26,22),fill='#51bca4');d.rectangle((10,5,21,12),fill='#e49b3c');d.rectangle((2,31,10,36),fill='#10191b');d.rectangle((22,31,29,36),fill='#10191b');d.rectangle((9+k%2,18,13+k%2,20),fill='#d4ffe4')
  elif kind=='grenade':d.ellipse((cx-3,cy-3,cx+3,cy+3),fill='#d09b37',outline='#182324');d.line((cx,cy-3,cx+2,cy-5),fill='#f9cf64')
  elif kind=='rocket':
   d.polygon([(4,cy-3),(15,cy-3),(19,cy),(15,cy+3),(4,cy+3)],fill='#788a63',outline='#182323');d.rectangle((4,cy-2,7,cy+2),fill='#f5b548');d.polygon([(3,cy-2),(0,cy),(3,cy+2)],fill='#ffcf76' if k%2 else '#ee7935')
  else:
   d.ellipse((cx-5,cy-5,cx+5,cy+5),outline='#f5b85e',width=1);d.line((cx-7,cy,cx+7,cy),fill='#ffd18b')
  frames.append(dict(duration=.05,data=pack(im)))
 anims[tag]=dict(frames=frames)
for tag,r in [('grenade_warning',22),('cluster_warning',12),('bot_warning',20),('rocket_path',9)]:effect(tag,(r*2+4,r*2+4),'warning',r)
for tag,r in [('satchel_blast',18),('grenade_blast',22),('cluster_blast',12),('rocket_blast',32),('landing',10)]:effect(tag,(r*2+6,r*2+6),'blast',r)
for tag in ['grenade_prep','pack_throw','bot_prep','rocket_equip','rocket_ready','rocket_release','buy']:effect(tag,(24,24),'charge')
effect('grenade_flight',(10,12),'grenade');effect('rocket_flight',(24,16),'rocket');effect('satchel',(32,40),'satchel');effect('bot',(32,40),'bot')
for direction in range(8):
 frames=[]
 for f in anims['rocket_flight']['frames']:
  r=f['data'];source=atlas.crop((r['x'],r['y'],r['x']+r['w'],r['y']+r['h']));tile=Image.new('RGBA',(32,32));tile.paste(source,(4,8));tile=tile.rotate(-45*direction,resample=Image.Resampling.NEAREST)
  frames.append(dict(duration=f['duration'],data=pack(tile)))
 anims['rocket_flight_'+str(direction)]=dict(frames=frames)
# Authorized mechanical resizing; every original approved frame is retained unchanged.
rules=json.loads((root/'rules/raze.json').read_text());C=rules['constants']
for tag,source_tag,scale in [('rocket_blast_large','rocket_blast',C['ROCKET_RADIUS']/32000),('rocket_impact_warning','grenade_warning',C['ROCKET_RADIUS']/22000),('satchel_blast_wide','satchel_blast',C['PACK_RADIUS']/18000),('grenade_blast_wide','grenade_blast',C['GRENADE_RADIUS']/22000),('grenade_warning_wide','grenade_warning',C['GRENADE_RADIUS']/22000),('cluster_blast_wide','cluster_blast',C['CLUSTER_RADIUS']/12000),('cluster_warning_wide','cluster_warning',C['CLUSTER_RADIUS']/12000)]:
 frames=[]
 for f in anims[source_tag]['frames']:
  r=f['data'];tile=atlas.crop((r['x'],r['y'],r['x']+r['w'],r['y']+r['h']))
  tile=tile.resize((round(tile.width*scale),round(tile.height*scale)),Image.Resampling.NEAREST)
  frames.append(dict(duration=f['duration'],data=pack(tile)))
 anims[tag]=dict(frames=frames)
# Approved compact pack. Preserve all previous approved frames and aliases.
approval=json.loads((review/'approval.json').read_text())
proposal=review/'sticky-satchel/proposed.png'
assert hashlib.sha256(proposal.read_bytes()).hexdigest()==approval['compact_satchel_sha256']
small=Image.open(proposal);sm=json.loads((review/'sticky-satchel/proposed.fanim').read_text())
for tag,ani in sm['anims'].items():
 frames=[]
 for f in ani['frames']:
  r=f['data'];frames.append(dict(duration=f['duration'],data=pack(small.crop((r['x'],r['y'],r['x']+r['w'],r['y']+r['h'])))))
 anims[tag]=dict(frames=frames)
# One-tick warning renewals select a baked progress phase, without stale tails.
for phase in range(8):
 size=2*(C['ROCKET_RADIUS']//1000)+6;im=Image.new('RGBA',(size,size));d=ImageDraw.Draw(im);c=size//2;r=C['ROCKET_RADIUS']//1000
 d.ellipse((c-r,c-r,c+r,c+r),outline='#ffb445',width=1)
 d.ellipse((c-r//3,c-r//3,c+r//3,c+r//3),outline='#ef7952',width=1)
 d.arc((c-r+2,c-r+2,c+r-2,c+r-2),start=-90,end=-90+max(1,360*(phase+1)//8),fill='#ffe29c',width=2)
 anims['rocket_impact_warning_'+str(phase)]=dict(frames=[dict(duration=1/60,data=pack(im))])
atlas=atlas.crop((0,0,1024,y+row+1));atlas.save(mod/'vfx/raze#sheet.png');(mod/'vfx/raze#anim.fanim').write_text(json.dumps(dict(anims=anims),separators=(',',':'))+'\n',encoding='utf-8')
sheet=Image.new('RGB',(800,math.ceil(len(anims)/5)*110+12),'#203235');d=ImageDraw.Draw(sheet)
for n,(tag,ani) in enumerate(anims.items()):
 r=ani['frames'][min(3,len(ani['frames'])-1)]['data'];p=atlas.crop((r['x'],r['y'],r['x']+r['w'],r['y']+r['h']));xx=12+(n%5)*158;yy=12+(n//5)*110;sheet.paste(p,(xx+(130-p.width)//2,yy),p);d.text((xx,yy+82),tag,fill='#f0dbb9')
sheet.save(review/'effects-comparison.png');sha=hashlib.sha256((mod/'vfx/raze#sheet.png').read_bytes()).hexdigest();approval=json.loads((review/'approval.json').read_text()) if (review/'approval.json').exists() else {}
approved=approval.get('effects_v3_atlas_sha256',approval.get('effects_scaled_atlas_sha256',approval.get('effects_derived_atlas_sha256',approval.get('effects_atlas_sha256'))))==sha
(review/'effects.json').write_text(json.dumps({'atlas':[atlas.width,atlas.height],'decodedBytes':atlas.width*atlas.height*4,'tags':list(anims),'approval':'approved' if approved else 'pending','approved_atlas_sha256':sha if approved else None},indent=2)+'\n',encoding='utf-8')
print(f'Raze presentation: {len(anims)} tags, {atlas.width}x{atlas.height}, {atlas.width*atlas.height*4} decoded bytes')
