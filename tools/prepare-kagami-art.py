"""Review-only extraction of original whole-body and mirror-effect concepts."""
from pathlib import Path
from PIL import Image,ImageOps,ImageDraw
import json,hashlib
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/kagami-art-review';N=Image.Resampling.NEAREST
if json.loads((OUT/'approvals.json').read_text())['body']['approved']:
 raise SystemExit('Appearance already approved. Use tools/build-kagami-art.py for shipped poses.')

def cells(file,cols,rows):
 im=Image.open(OUT/file).convert('RGBA');result=[]
 for i in range(cols*rows):
  f=im.crop((i%cols*im.width//cols,i//cols*im.height//rows,(i%cols+1)*im.width//cols,(i//cols+1)*im.height//rows))
  f.putalpha(f.getchannel('A').point(lambda a:255 if a>=220 else 0));result.append(f.crop(f.getbbox()))
 return result
body=cells('body-source-v3.png',4,2);scale=44/body[0].height
sheet=Image.new('RGBA',(512,128));clone=Image.new('RGBA',sheet.size);frames={}
names=['idle','run_a','run_b','attack','heal','step','hit','dead']
for i,(name,f) in enumerate(zip(names,body)):
 f=f.resize((round(f.width*scale),round(f.height*scale)),Image.Resampling.NEAREST);f.putalpha(f.getchannel('A').point(lambda a:255 if a>=128 else 0))
 f=f.quantize(colors=16,method=Image.Quantize.FASTOCTREE,dither=Image.Dither.NONE).convert('RGBA')
 assert f.width<62 and f.height<52,(name,f.size)
 p=Image.new('RGBA',(64,64));p.alpha_composite(f,((64-f.width)//2,52-f.height))
 c=p.copy();pix=c.load()
 for y in range(64):
  for x in range(64):
   r,g,b,a=pix[x,y]
   if a:
    grey=(r+g+b)//3;pix[x,y]=(min(255,grey//2+85),min(255,grey//2+96),min(255,grey//2+125),180)
 for left in [False,True]:
  x=i*64;y=64 if left else 0
  sheet.alpha_composite(ImageOps.mirror(p) if left else p,(x,y));clone.alpha_composite(ImageOps.mirror(c) if left else c,(x,y))
  frames[name+('_l' if left else '')]={'x':x,'y':y,'w':64,'h':64}
sheet.save(OUT/'concepts.png');clone.save(OUT/'clone-concepts.png')
rawfx=cells('effects-source-v1.png',4,3)
fxdefs={'shard':(0,14,8),'shard_hit':(1,24,24),'thread':(2,64,12),'swap':(3,40,40),'stun_burst':(4,100,100),'clone_in':(5,38,52),'heal':(6,32,34),'charm':(7,18,18),'angel':(8,100,90),'seal':(9,48,72),'revive':(10,58,58),'revived':(11,38,54)}
fxsheet=Image.new('RGBA',(512,384));fxframes={}
for i,(name,(idx,w,h)) in enumerate(fxdefs.items()):
 f=rawfx[idx].resize((w,h),Image.Resampling.BOX);f.putalpha(f.getchannel('A').point(lambda a:255 if a>=160 else 0));f=f.quantize(colors=12,method=Image.Quantize.FASTOCTREE,dither=Image.Dither.NONE).convert('RGBA')
 x=i%4*128+(128-w)//2;y=i//4*128+(128-h)//2;fxsheet.alpha_composite(f,(x,y));fxframes[name]={'x':x,'y':y,'w':w,'h':h}
fxsheet.save(OUT/'effects-concepts.png')
meta={'revision':'kagami-art-concept-1','review_only':True,'canvas':[64,64],'feet':[32,52],'units_per_pixel':1000,'frames':frames,'effects':fxframes,'source_sha256':{n:hashlib.sha256((OUT/n).read_bytes()).hexdigest() for n in ['body-source-v3.png','effects-source-v1.png']}}
(OUT/'concepts.json').write_text(json.dumps(meta,indent=2)+'\n');(ROOT/'editor/kagami-art.js').write_text('window.KAGAMI_ART='+json.dumps(meta,separators=(',',':'))+';\n')
board=Image.new('RGB',(1120,8*220));d=ImageDraw.Draw(board)
for i,name in enumerate(names):
 for col,bg in enumerate(['#203747','#bec9b0']):
  x=col*560;y=i*220;d.rectangle((x,y,x+559,y+219),fill=bg);d.text((x+8,y+6),name+' | native size + 3x | both facings',fill='#f8edff' if not col else '#24223a')
  for j,left in enumerate([False,True]):
   r=frames[name+('_l' if left else '')];p=sheet.crop((r['x'],r['y'],r['x']+64,r['y']+64));board.paste(p,(x+8+j*64,y+100),p);b=p.resize((192,192),N);board.paste(b,(x+150+j*195,y+18),b)
board.save(OUT/'body-comparison.png')
board=Image.new('RGB',(1120,6*340));d=ImageDraw.Draw(board)
for i,(name,r) in enumerate(fxframes.items()):
 x=i%2*560;y=i//2*340;bg='#203747' if i%2==0 else '#bec9b0';d.rectangle((x,y,x+559,y+339),fill=bg);d.text((x+8,y+5),name+' | 1x / 3x',fill='#ede7ff' if i%2==0 else '#24223a')
 f=fxsheet.crop((r['x'],r['y'],r['x']+r['w'],r['y']+r['h']));board.paste(f,(x+12,y+70),f);b=f.resize((f.width*3,f.height*3),N)
 board.paste(b,(x+160,y+25),b)
board.save(OUT/'effects-comparison.png')
world=json.loads((ROOT/'editor/game-reference/world.json').read_text());compare=Image.new('RGB',(1000,270),'#203747');d=ImageDraw.Draw(compare);d.text((12,8),'Stock Priest / Thalassa placeholder / Kagami candidate / mirror clone | 1x + 3x',fill='#f8edff')
sprites=[]
s=world['sprites']['champ-priest'];r=s['anims']['idle']['frames'][0]['data'];im=Image.open(ROOT/'editor/game-reference'/s['file']);sprites.append(('Priest',im.crop((r['x'],r['y'],r['x']+r['w'],r['y']+r['h']))))
s=json.loads((ROOT/'mods/tfm2_custom/champions/tfm2_custom_thalassa#anim.fanim').read_text())['anims']['idle']['frames'][0]['data'];im=Image.open(ROOT/'mods/tfm2_custom/champions/tfm2_custom_thalassa#sheet.png');sprites.append(('Thalassa',im.crop((s['x'],s['y'],s['x']+s['w'],s['y']+s['h']))))
sprites += [('Kagami',sheet.crop((0,0,64,64))),('Reflection',clone.crop((0,0,64,64)))]
for i,(name,f) in enumerate(sprites):
 x=20+i*245;d.text((x,32),name,fill='#f8edff');compare.paste(f,(x,112-f.height),f);b=f.resize((f.width*3,f.height*3),N);compare.paste(b,(x+45,253-b.height),b)
compare.save(OUT/'stock-comparison.png')
print(json.dumps({'review_only':True,'body_frames':16,'effects':12,'body_height':44,'native_assets_added':False}))
