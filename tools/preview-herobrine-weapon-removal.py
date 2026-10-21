"""Register only the weapon-removal proposal; all retained pixels come from approved Steve.

ImageGen supplied the removal reference. This packing step forbids any added arm,
body or gesture pixels. Runtime files remain unchanged until approval.
"""
from pathlib import Path
from PIL import Image,ImageDraw,ImageFont,ImageOps
import json,hashlib
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/creative-herobrine-review'
stem=ROOT/'mods/tfm2_blockcraft/champions/tfm2_blockcraft_steve'
old=Image.open(OUT/'steve-approved-base.png').convert('RGBA');candidate=old.copy()
meta=json.loads(Path(str(stem)+'#anim.fanim').read_text(encoding='utf8'))
eye=Image.open(ROOT/'mods/tfm2_blockcraft/vfx/steve_eyes#sheet.png').convert('RGBA')
eyes=json.loads((ROOT/'mods/tfm2_blockcraft/vfx/steve_eyes#anim.fanim').read_text(encoding='utf8'))['anims']
font=ImageFont.load_default(size=12);sheet=Image.new('RGB',(850,60+len(meta['anims'])*120),'#15232d');d=ImageDraw.Draw(sheet)
d.text((12,12),'REVISED: original Steve arms and hands; pickaxe removed; existing white-eye overlay.',font=font,fill='white')
frames=[];dur=[];removed=0
for row,(name,anim)in enumerate(meta['anims'].items()):
 for n,f in enumerate(anim['frames']):
  r=f['data'];box=(r['x'],r['y'],r['x']+36,r['y']+44);a=old.crop(box);b=a.copy();erase=set()
  def region(x,y):return y<8 or (x>=26 and y<28)or(x>=24 and 20<y<28)
  def skin(p):
   red,g,blue,alpha=p
   return alpha>100 and red>110 and red>g*1.15 and g>blue*1.1 and blue>=g*.43
  for y in range(32):
   for x in range(36):
    red,g,blue,alpha=a.getpixel((x,y))
    if alpha<40 or not region(x,y):continue
    metal=g>65 and abs(red-g)<45 and abs(blue-g)<40
    handle=red>80 and blue<g*.43 and red>g*1.15
    if metal or handle:erase.add((x,y))
  for y in range(32):
   for x in range(36):
    p=a.getpixel((x,y))
    if not region(x,y)or p[3]<40 or max(p[:3])>100:continue
    near_tool=any((x+dx,y+dy)in erase for dx in [-1,0,1]for dy in [-1,0,1])
    near_skin=any(skin(a.getpixel((xx,yy)))for xx in range(max(0,x-1),min(35,x+1)+1)for yy in range(max(0,y-1),min(43,y+1)+1))
    if near_tool and not near_skin:erase.add((x,y))
  for p in erase:b.putpixel(p,(0,0,0,0))
  if name=='attack' and n>=2:
   for x in range(26,36):b.putpixel((x,19),(0,0,0,0))
   for y in range(20,23):
    for x in range(26,36):
     if max(b.getpixel((x,y))[:3])<110:b.putpixel((x,y),(0,0,0,0))
  removed+=len(erase);candidate.paste(b,(r['x'],r['y']))
  assert all(q[3]==0 or q==p for p,q in zip(a.getdata(),b.getdata())),'Added body pixels'
  # The original white eyes remain a separate, already approved overlay.
  er=eyes[name+'_r']['frames'][n]['data'];white=eye.crop((er['x'],er['y'],er['x']+36,er['y']+44));b.alpha_composite(white)
  canvas=Image.new('RGB',(650,185),'#17232c');dr=ImageDraw.Draw(canvas);dr.text((10,10),name+' / Steve / original short arms, no pickaxe / mirrored',font=font,fill='white')
  for col,(im,bg)in enumerate([(a,'#293e38'),(b,'#293e38'),(ImageOps.mirror(b),'#c8cdb3')]):
   x=10+col*210;dr.rectangle((x,35,x+200,182),fill=bg);canvas.paste(im,(x+5,126),im);big=im.resize((108,132),Image.Resampling.NEAREST);canvas.paste(big,(x+70,45),big)
  frames.append(canvas);dur.append(max(20,round(f['duration']*1000)))
  if n==len(anim['frames'])//2:
   y=50+row*120;d.text((10,y),name,font=font,fill='#d7bf82')
   for col,(im,bg)in enumerate([(a,'#293e38'),(b,'#293e38'),(ImageOps.mirror(b),'#c8cdb3')]):
    x=150+col*225;d.rectangle((x,y,x+210,y+112),fill=bg);sheet.paste(im,(x+5,y+62),im);large=im.resize((72,88),Image.Resampling.NEAREST);sheet.paste(large,(x+80,y+18),large)
candidate.save(OUT/'weapon-removal-candidate.png');sheet.save(OUT/'weapon-removal-comparison.png')
frames[0].save(OUT/'weapon-removal-animation.gif',save_all=True,append_images=frames[1:],duration=dur,loop=0,disposal=2)
(OUT/'weapon-removal-review.json').write_text(json.dumps({'approval':'Approve original arms without pickaxe','runtimeUnchanged':False,'retainedPixelsIdentical':True,'newBodyPixels':0,'removedWeaponPixels':removed,'candidateSha256':hashlib.sha256((OUT/'weapon-removal-candidate.png').read_bytes()).hexdigest(),'canvas':[36,44],'frames':len(frames)},indent=2)+'\n',encoding='utf8')
print('Original body pixels retained; no extended arms; preview only.')
