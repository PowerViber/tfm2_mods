"""Steve's original body is read only. New compact voxel overlays and craft objects."""
from pathlib import Path
from PIL import Image,ImageDraw,ImageFont,ImageOps
import json,math,hashlib
ROOT=Path(__file__).resolve().parents[1];MOD=ROOT/'mods/tfm2_blockcraft';OUT=MOD/'vfx';DOC=ROOT/'docs/steve-results';DOC.mkdir(parents=True,exist_ok=True)
INK='#142329';STONE='#49545c';LIGHT='#b2c0c4';TEAL='#47e0cf';GOLD='#e2bf67';RED='#df4c54';PURPLE='#be83ef'
def img(w,h):return Image.new('RGBA',(w,h))
def object_art(r,k):
 size=(60,40)if r==0 else (40,40)if r==4 else (36,36)if r in [2,5]else(32,40)
 im=img(*size);d=ImageDraw.Draw(im);w,h=size;c=w//2
 if r in [2,4,5]:
  col=('#cde6eb',100)if r==2 else('#42bddd',100)if r==4 else('#f17535',150)
  color=(*tuple(bytes.fromhex(col[0][1:])),col[1]);d.ellipse((1,h//4,w-2,h-2),fill=color,outline=col[0],width=1)
  if r==2:
   for a in range(8):q=a*math.tau/8;d.line((c,h//2,c+int((c-2)*math.cos(q)),h//2+int((h//2-2)*math.sin(q))),fill='#dfeaed')
   for inset in [7,12]:d.ellipse((inset,inset,w-inset,h-inset),outline='#b7c6ce')
  else:
   for y in range(12,h-3,7):
    for x in range(5,w-5,9):d.line((x,y,x+4,y+int(math.sin((x+k*2)/4))),fill='#c0ffff'if r==4 else'#ffce64')
  return im
 # Opaque pedestal entirely covers the supported carrier's visible footprint.
 d.rounded_rectangle((1,1,w-2,h-1),radius=4,fill=INK,outline='#7b939d')
 if r==0:
  for y in range(3,h-2,8):
   for x in range(3+(4 if y%16 else 0),w-5,11):d.rectangle((x,y,min(x+9,w-4),y+6),fill=STONE,outline='#71808c')
 elif r==1:
  d.rectangle((4,5,w-5,h-5),fill='#88663b',outline='#d5b97b');d.rectangle((7,10,w-8,25),fill=STONE,outline=LIGHT);d.line((10,15,21,15),fill=TEAL,width=2);d.line((21,15,18,12),fill=TEAL)
 elif r==3:
  d.rectangle((5,7,w-6,h-6),fill='#ba323e',outline='#ff7d74');d.rectangle((5,17,w-6,25),fill='#ece5d5');d.text((7,17),'TNT',fill=INK,font=ImageFont.load_default(size=8));d.line((c,7,c+2,3),fill=GOLD);d.point((c+2,2),fill='#ffffff'if k%2 else GOLD)
 elif r==6:
  d.rectangle((4,30,w-5,36),fill=STONE,outline=LIGHT);d.polygon([(c,4),(w-6,17),(c,30),(5,17)],fill='#623c92',outline=PURPLE);d.line((c,6,c-4,17,c,27),fill='#e4cbff',width=2);d.line((c,9,c+4,18),fill=TEAL)
 elif r==7:
  d.rectangle((4,9,w-5,h-5),fill='#332d49',outline=PURPLE)
  for x in [8,18]:d.rectangle((x,15,x+4,21),fill=TEAL if (k+x)%3 else PURPLE)
  d.rectangle((6,28,25,33),fill='#687189');d.line((8,29,23,29),fill='#c2e9f0')
 return im
def effect(tag,k):
 if tag.startswith('object'):return object_art(int(tag[6:]),k)
 if tag.startswith('warn'):
  radius=int(tag[4:]);w=radius*2+4;im=img(w,w);d=ImageDraw.Draw(im);d.ellipse((2,2,w-3,w-3),fill=(220,75,90,15),outline=(255,160+10*(k%4),100,220),width=1);return im
 if tag in ['eyes','eyes_l','cast_l','cast_r']:
  im=img(36,44);d=ImageDraw.Draw(im)
  if tag.startswith('eyes'):
   for x in [21,24]:d.rectangle((x,13,x,13),fill='#ffffff');d.point((x,12),fill=(180,237,255,50+k*12))
  else:
   d.rectangle((24,23,27,28),fill='#9a6b49');d.line((24,22,30,16),fill=GOLD,width=2);d.rectangle((28,14,32,18),fill=TEAL,outline=INK)
  return ImageOps.mirror(im)if tag.endswith('_l')else im
 if tag.startswith('hud'):
  _,form,ready,reserved=tag.split('_');im=img(36,64);d=ImageDraw.Draw(im)
  for n in range(3):x=6+n*8;col=TEAL if n<int(ready)else GOLD if n<int(ready)+int(reserved)else STONE;d.rectangle((x,2,x+5,6),fill=col,outline=INK)
  if form=='1':d.line((9,10,26,10),fill='#eefcff')
  return im
 if tag=='wall':
  im=img(18,28);d=ImageDraw.Draw(im);d.rectangle((0,3,17,25),fill='#27303c',outline='#8294ac')
  for y in [5,13,21]:d.line((1,y,16,y),fill='#4f6278');d.line((8+(4 if y==13 else 0),y,8+(4 if y==13 else 0),y+7),fill='#85919e')
  d.line((1,26,16,26),fill='#9bd8ee');return im
 if tag=='boat':
  im=img(32,20);d=ImageDraw.Draw(im);d.polygon([(2,4),(29,4),(24,17),(8,17)],fill='#855d3b',outline=GOLD);d.rectangle((8,5,24,9),fill='#3a2e2a');return im
 if tag.startswith('fx'):
  im=img(68,68);d=ImageDraw.Draw(im);r=8+k*3;d.ellipse((34-r,34-r,34+r,34+r),outline=PURPLE if tag in ['fx6','fx7','fx20']else GOLD,width=2)
  for n in range(8):a=n*math.tau/8;d.rectangle((34+int(r*math.cos(a)),34+int(r*math.sin(a)),36+int(r*math.cos(a)),36+int(r*math.sin(a))),fill='#ecfbff')
  return im
 im=img(20,20);d=ImageDraw.Draw(im)
 if tag in ['tool8']:d.line((4,3,13,3,13,14,8,17,5,12),fill=LIGHT,width=2)
 elif tag in ['tool9']:d.ellipse((4,4,15,15),fill='#185c60',outline=TEAL);d.rectangle((8,6,10,8),fill='#c3faf4')
 elif tag=='tool12':d.line((2,10,17,10),fill='#e9e1bf',width=2);d.line((13,7,17,10,13,13),fill=GOLD)
 elif tag=='tool14':return effect('boat',k)
 else:d.rectangle((4,4,15,16),fill=RED,outline=INK)
 return im
def generate():
 tags=[f'object{r}'for r in [0,1,2,3,4,5,6,7]]+[f'tool{r}'for r in [3,8,9,12,14]]+['eyes','eyes_l','cast_l','cast_r','wall','boat']+[f'warn{r}'for r in [10,16,18,20,28,32]]+[f'fx{r}'for r in [3,6,7,20]]+[f'hud_{f}_{a}_{b}'for f in range(2)for a in range(4)for b in range(4-a)]
 atlas=img(1024,1024);anims={};x=y=row=0;dedup={}
 for tag in tags:
  frames=[]
  for k in range(8):
   im=effect(tag,k);key=(im.size,im.tobytes())
   if key in dedup:rect=dedup[key]
   else:
    w,h=im.size
    if x+w>1024:x=0;y+=row+1;row=0
    if y+h>2048:raise ValueError('Atlas exceeds limit')
    if y+h>atlas.height:big=img(1024,2048);big.paste(atlas,(0,0));atlas=big
    atlas.paste(im,(x,y));rect=dict(x=x,y=y,w=w,h=h);dedup[key]=rect;x+=w+1;row=max(row,h)
   frames.append(dict(duration=0.08,data=rect))
  anims[tag]=dict(frames=frames)
 atlas=atlas.crop((0,0,1024,y+row+1));atlas.save(OUT/'steve_craft#sheet.png');(OUT/'steve_craft#anim.fanim').write_text(json.dumps(dict(anims=anims),separators=(',',':')))
 data_path=MOD/'champion/tfm2_blockcraft_steve.data_champion';data=json.loads(data_path.read_text());base='asset/tfm2_blockcraft/vfx/steve_craft'
 data['passive']={'passive_ref':'tfm2_custom_ai:steve_craft','params':{}};data['passive_ult']=None;data['attack']['effect']={'type':'Native','name':'tfm2_custom_ai:steve_basic','params':{}}
 # Match the repository's exact Native effect schema.
 gojo=json.loads((ROOT/'mods/tfm2_jjk/champion/tfm2_custom_gojo.data_champion').read_text());data['attack']['effect']=json.loads(json.dumps(gojo['attack']['effect']).replace('gojo_basic_v2','steve_basic'))
 for slot in ['skill','skill2','ult']:
  data[slot].update(duration=6,cooltime=5184000,start_timing=3,range=1,casting_type='Targeting',casting_target='EnemyChampionInCC',cancelable=True,can_use_with_move=False,effect={'type':'Combine','effects':[]})
 data['view_effects']=[v for v in data.get('view_effects',[])if not v['name'].startswith('tfm2_blockcraft_craft_')]+[dict(type='Animation',name='tfm2_blockcraft_craft_'+tag,anim=base,tag=tag,z=-1 if tag.startswith('warn')or tag in ['object2','object4','object5']else 3,is_follow=False)for tag in tags if not tag.startswith('hud')]
 data['view_buffs']=[v for v in data.get('view_buffs',[])if not v['name'].startswith('sc_hud')]+[dict(type='Animated',name='sc_'+tag,anim=base,tag=tag,z=4)for tag in tags if tag.startswith('hud')]
 data_path.write_text(json.dumps(data,indent=2)+'\n');(ROOT/'editor/steve-preset.js').write_text('window.STEVE_PRESET='+json.dumps(data,separators=(',',':'))+';\n')
 text_path=MOD/'text/champion.i18n';text=json.loads(text_path.read_text())
 desc=dict(name='Steve',skill='Build: wall, piston, web, TNT, water or lava. Prepare a crystal or anchor using two reserved charges, then deploy and detonate it. Three shared charges recover separately in five seconds. Heavy blasts can hurt Steve and are blocked by walls.',skill2='Utility: fishing rod, ender pearl, golden apple, shield, crossbow, axe or short boat. Each recipe has its own cooldown and spends one shared charge. Steve improves healing, shields and newly built object health. Herobrine turns a confirmed champion utility hit into one target-bound melee follow-up.',ult='Boat Wall: commit a warned path, then ride while the wall rises. Steve builds straight; Herobrine can build open curves. Jump off early while the empty boat finishes. Ice speeds allies. Change Form is a separate native action: 0.4 second preparation, 12 second base cooldown after ultimate unlock. It never spends this ultimate.')
 text.setdefault('en',{}).setdefault('description',{})['tfm2_blockcraft_steve']=desc;text_path.write_text(json.dumps(text,indent=2,ensure_ascii=False)+'\n')
 # Original pixels and original animation metadata are included verbatim in review.
 body=Image.open(MOD/'champions/tfm2_blockcraft_steve#sheet.png').convert('RGBA').crop((0,0,36,44));sheet=Image.new('RGB',(720,480),'#18232e');d=ImageDraw.Draw(sheet);font=ImageFont.load_default(size=13)
 for n,tag in enumerate(['eyes','cast_r','object0','object1','object2','object3','object4','object5','object6','object7','tool8','tool9','tool12','boat','wall','warn32']):
  x=(n%4)*180;y=(n//4)*120;im=effect(tag,2);preview=img(100,70)
  if tag in ['eyes','cast_r']:preview.alpha_composite(body,(32,20));preview.alpha_composite(im,(32,20))
  else:preview.alpha_composite(im,((100-im.width)//2,(70-im.height)//2))
  sheet.paste(preview.resize((140,98),Image.Resampling.NEAREST),(x+20,y+18),preview.resize((140,98),Image.Resampling.NEAREST));d.text((x+12,y+4),tag,fill='white',font=font)
 sheet.save(DOC/'craft-contact-sheet.png')
 frames=[]
 for k in range(8):
  im=Image.new('RGBA',(288,176),'#18232e')
  for n,tag in enumerate(['eyes','cast_r','object6','object7']):
   cell=img(36,44)
   if n<2:cell.alpha_composite(body)
   art=effect(tag,k);cell.alpha_composite(art,((36-art.width)//2,44-art.height));im.alpha_composite(cell.resize((72,88),Image.Resampling.NEAREST),(n*72,44))
  frames.append(im.convert('RGB'))
 frames[0].save(DOC/'craft-preview.gif',save_all=True,append_images=frames[1:],duration=80,loop=0)
 decoded=sum(Image.open(p).width*Image.open(p).height*4 for p in MOD.rglob('*.png'));assert decoded<32*1024*1024
 report=dict(atlas=list(atlas.size),decodedBytes=decoded,uniqueFrames=len(dedup),bodySha256=hashlib.sha256((MOD/'champions/tfm2_blockcraft_steve#sheet.png').read_bytes()).hexdigest(),bodyMetadataSha256=hashlib.sha256((MOD/'champions/tfm2_blockcraft_steve#anim.fanim').read_bytes()).hexdigest());(DOC/'art.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report))
if __name__=='__main__':generate()
