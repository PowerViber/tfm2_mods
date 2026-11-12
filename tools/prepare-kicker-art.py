"""Review-only extraction of Kicker's original generated whole-body concepts."""
from pathlib import Path
import json,hashlib
from PIL import Image,ImageOps,ImageDraw
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/kicker-art-review';N=Image.Resampling.NEAREST
im=Image.open(OUT/'body-source-v1.png').convert('RGBA')
cells=[]
for i in range(8):
 f=im.crop((i%4*im.width//4,i//4*im.height//2,(i%4+1)*im.width//4,(i//4+1)*im.height//2))
 f.putalpha(f.getchannel('A').point(lambda a:255 if a>=220 else 0))
 cells.append(f.crop(f.getbbox()))
scale=39/cells[1].height
names=['front','idle','run_a','run_b','windup','kick','perch','dive'];sheet=Image.new('RGBA',(56*8,48*2));frames={}
for i,(name,src) in enumerate(zip(names,cells)):
 f=src.resize((round(src.width*scale),round(src.height*scale)),Image.Resampling.BOX)
 f.putalpha(f.getchannel('A').point(lambda a:255 if a>=128 else 0));f=f.quantize(colors=16,method=Image.Quantize.FASTOCTREE,dither=Image.Dither.NONE).convert('RGBA')
 assert f.width<=56 and f.height<=44,(name,f.size)
 pose=Image.new('RGBA',(56,48));pose.alpha_composite(f,((56-f.width)//2,44-f.height))
 for left in [False,True]:
  x=i*56;y=48 if left else 0;sheet.alpha_composite(ImageOps.mirror(pose) if left else pose,(x,y));frames[name+('_l' if left else '')]={'x':x,'y':y,'w':56,'h':48}
sheet.save(OUT/'concepts.png')
meta={'revision':'kicker-art-concept-1','review_only':True,'source_sha256':hashlib.sha256((OUT/'body-source-v1.png').read_bytes()).hexdigest(),'canvas':[56,48],'feet':[28,44],'units_per_pixel':1000,'frames':frames}
(OUT/'concepts.json').write_text(json.dumps(meta,indent=2)+'\n')
(ROOT/'editor/kicker-art.js').write_text('window.KICKER_ART='+json.dumps(meta,separators=(',',':'))+';\n')
world=json.loads((ROOT/'editor/game-reference/world.json').read_text())
def stock(name):
 s=world['sprites']['champ-'+name];d=s['anims']['idle']['frames'][0]['data'];a=Image.open(ROOT/'editor/game-reference'/s['file']).convert('RGBA');return a.crop((d['x'],d['y'],d['x']+d['w'],d['y']+d['h']))
board=Image.new('RGB',(1120,8*190));d=ImageDraw.Draw(board)
for i,name in enumerate(names):
 for col,bg in enumerate(['#213843','#bac6ae']):
  x=col*560;y=i*190;d.rectangle((x,y,x+559,y+189),fill=bg);d.text((x+8,y+6),name+' | 1x + 3x | candidate',fill='#f6dcc3' if not col else '#172d37')
  for left in [False,True]:
   f=frames[name+('_l' if left else '')];p=sheet.crop((f['x'],f['y'],f['x']+56,f['y']+48));xx=x+12+(100 if left else 0);board.paste(p,(xx,y+60),p);big=p.resize((168,144),N);board.paste(big,(x+222+(168 if left else 0),y+32),big)
board.save(OUT/'concept-comparison.png')
compare=Image.new('RGB',(900,260),'#213843');d=ImageDraw.Draw(compare);d.text((16,10),'Original stock / current placeholder / Kicker candidate | 1x and 3x',fill='#e7e8d8')
a=json.loads((ROOT/'mods/tfm2_custom/champions/tfm2_custom_contender#anim.fanim').read_text())['anims']['idle']['frames'][0]['data'];b=Image.open(ROOT/'mods/tfm2_custom/champions/tfm2_custom_contender#sheet.png').convert('RGBA');old=b.crop((a['x'],a['y'],a['x']+a['w'],a['y']+a['h']))
for i,(name,p) in enumerate([('Knight',stock('knight')),('Monk',stock('monk')),('Contender placeholder',old),('Kicker',sheet.crop((56,0,112,48)))]):
 x=20+i*220;d.text((x,38),name,fill='#fff1d8')
 compare.paste(p,(x,100-p.height),p);p=p.resize((p.width*3,p.height*3),N);compare.paste(p,(x+50,238-p.height),p)
compare.save(OUT/'stock-comparison.png')
print(json.dumps({'review_only':True,'frames':len(frames),'source_sha256':meta['source_sha256'],'canvas':[56,48],'stand_height':39,'packaged_art_changed':False}))

