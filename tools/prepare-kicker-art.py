"""Review-only extraction of Kicker's original generated whole-body concepts."""
from pathlib import Path
import json,hashlib
from PIL import Image,ImageOps,ImageDraw
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/kicker-art-review';N=Image.Resampling.NEAREST
SOURCE='body-source-v2.png' if (OUT/'body-source-v2.png').exists() else 'body-source-v1.png'
V2=SOURCE.endswith('v2.png');CW,CH,FY,HEIGHT=(64,56,48,44) if V2 else (56,48,44,39)
im=Image.open(OUT/SOURCE).convert('RGBA')
cells=[]
for i in range(8):
 f=im.crop((i%4*im.width//4,i//4*im.height//2,(i%4+1)*im.width//4,(i//4+1)*im.height//2))
 f.putalpha(f.getchannel('A').point(lambda a:255 if a>=220 else 0))
 cells.append(f.crop(f.getbbox()))
scale=HEIGHT/cells[1].height
names=['front','idle','run_a','run_b','windup','kick','perch','dive'];sheet=Image.new('RGBA',(CW*8,CH*2));frames={}
for i,(name,src) in enumerate(zip(names,cells)):
 f=src.resize((round(src.width*scale),round(src.height*scale)),Image.Resampling.BOX)
 f.putalpha(f.getchannel('A').point(lambda a:255 if a>=128 else 0));f=f.quantize(colors=16,method=Image.Quantize.FASTOCTREE,dither=Image.Dither.NONE).convert('RGBA')
 assert f.width<=CW and f.height<=FY,(name,f.size)
 pose=Image.new('RGBA',(CW,CH));pose.alpha_composite(f,((CW-f.width)//2,FY-f.height))
 for left in [False,True]:
  x=i*CW;y=CH if left else 0;sheet.alpha_composite(ImageOps.mirror(pose) if left else pose,(x,y));frames[name+('_l' if left else '')]={'x':x,'y':y,'w':CW,'h':CH}
sheet.save(OUT/'concepts.png')
meta={'revision':'kicker-art-concept-'+('2' if V2 else '1'),'review_only':True,'source_sha256':hashlib.sha256((OUT/SOURCE).read_bytes()).hexdigest(),'canvas':[CW,CH],'feet':[CW//2,FY],'units_per_pixel':1000,'frames':frames}
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
   f=frames[name+('_l' if left else '')];p=sheet.crop((f['x'],f['y'],f['x']+CW,f['y']+CH));xx=x+12+(100 if left else 0);board.paste(p,(xx,y+60),p);big=p.resize((CW*3,CH*3),N);board.paste(big,(x+175+(CW*3 if left else 0),y+20),big)
board.save(OUT/'concept-comparison.png')
compare=Image.new('RGB',(900,260),'#213843');d=ImageDraw.Draw(compare);d.text((16,10),'Original stock / current placeholder / Kicker candidate | 1x and 3x',fill='#e7e8d8')
a=json.loads((ROOT/'mods/tfm2_custom/champions/tfm2_custom_contender#anim.fanim').read_text())['anims']['idle']['frames'][0]['data'];b=Image.open(ROOT/'mods/tfm2_custom/champions/tfm2_custom_contender#sheet.png').convert('RGBA');old=b.crop((a['x'],a['y'],a['x']+a['w'],a['y']+a['h']))
for i,(name,p) in enumerate([('Knight',stock('knight')),('Monk',stock('monk')),('Contender placeholder',old),('Kicker',sheet.crop((CW,0,CW*2,CH)))]):
 x=20+i*220;d.text((x,38),name,fill='#fff1d8')
 compare.paste(p,(x,100-p.height),p);p=p.resize((p.width*3,p.height*3),N);compare.paste(p,(x+50,238-p.height),p)
compare.save(OUT/'stock-comparison.png')
print(json.dumps({'review_only':True,'frames':len(frames),'source_sha256':meta['source_sha256'],'canvas':[CW,CH],'stand_height':HEIGHT,'packaged_art_changed':False}))

# Preserve the rejected proposal as a side-by-side scale reference.
if V2 and (OUT/'concepts-v1.json').exists():
    old=json.loads((OUT/'concepts-v1.json').read_text())
    def idle_crop(path,meta):
        r=meta['frames']['idle'];image=Image.open(path).convert('RGBA')
        return image.crop((r['x'],r['y'],r['x']+r['w'],r['y']+r['h']))
    old_idle=idle_crop(OUT/'concepts-v1.png',old)
    new_idle=idle_crop(OUT/'concepts.png',meta)
    comparison=Image.new('RGB',(1120,340));draw=ImageDraw.Draw(comparison)
    for col,bg,ink in [(0,'#213843','#eff4eb'),(1,'#bbc9ad','#1b3038')]:
        origin=col*560;draw.rectangle((origin,0,origin+559,339),fill=bg)
        draw.text((origin+15,12),'Previous vs revised | same world scale | 1x and 3x',fill=ink)
        for label,sprite,spec,x in [('Previous',old_idle,old,75),('Revised',new_idle,meta,170)]:
            draw.text((origin+x-25,40),label,fill=ink)
            comparison.paste(sprite,(origin+x-spec['feet'][0],120-spec['feet'][1]),sprite)
        for label,sprite,spec,x in [('Previous 3x',old_idle,old,305),('Revised 3x',new_idle,meta,450)]:
            draw.text((origin+x-35,145),label,fill=ink)
            big=sprite.resize((sprite.width*3,sprite.height*3),N)
            comparison.paste(big,(origin+x-spec['feet'][0]*3,315-spec['feet'][1]*3),big)
    comparison.save(OUT/'before-after-v2.png')
