"""Extract original shadow material proposals at canonical sizes. Review only."""
from pathlib import Path
import json,hashlib
from PIL import Image,ImageDraw,ImageFont
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/contender-shadow-review';N=Image.Resampling.NEAREST
source=OUT/'effects-source-v1.png';im=Image.open(source).convert('RGBA');assert im.size==(1448,1086)
specs=[('launch',(0,0,380,360),(72,28)),('swap',(380,0,685,360),(40,40)),('cross',(690,0,1060,360),(48,32)),('cleave',(1060,0,1448,360),(54,54)),
 ('slam',(0,365,380,725),(34,26)),('uppercut',(380,365,690,725),(36,60)),('echo',(720,365,1060,725),(22,22)),('vanish',(1060,365,1448,725),(44,26)),
 ('appear',(0,726,330,1086),(40,44)),('flurry',(330,726,687,1086),(52,30)),('haymaker',(688,726,1168,1086),(64,42)),('charges',(1170,726,1448,1086),(30,20))]
sheet=Image.new('RGBA',(512,384));frames={};comparison=Image.new('RGBA',(1100,780),'#152c36');d=ImageDraw.Draw(comparison);font=ImageFont.truetype('C:/Windows/Fonts/arial.ttf',14)
for index,(name,rect,size) in enumerate(specs):
    tile=im.crop(rect);box=tile.getchannel('A').point(lambda p:255 if p>=128 else 0).getbbox();assert box
    tile=tile.crop(box).resize(size,N);x=(index%4)*128;y=(index//4)*128;sheet.alpha_composite(tile,(x,y));frames[name]={'x':x,'y':y,'w':size[0],'h':size[1]}
    cx=(index%4)*275;cy=(index//4)*260;d.rectangle((cx,cy,cx+274,cy+259),fill='#2b414a' if index%2==0 else '#bec9ae');ink='#fff0df' if index%2==0 else '#182939'
    d.text((cx+12,cy+10),name+' / 1× and 3×',font=font,fill=ink);comparison.alpha_composite(tile,(cx+42-tile.width//2,cy+130-tile.height//2));big=tile.resize((tile.width*3,tile.height*3),N);comparison.alpha_composite(big,(cx+170-big.width//2,cy+130-big.height//2))
    if name=='cleave':d.text((cx+12,cy+230),'25,000 radius + edge accents',font=font,fill=ink)
    if name=='slam':d.text((cx+12,cy+230),'15,000 ground impact radius',font=font,fill=ink)
sheet.save(OUT/'effects-proposals.png');comparison.convert('RGB').save(OUT/'effects-comparison.png')
meta={'revision':'contender-shadow-effects-1','approved':json.loads((OUT/'approvals.json').read_text())['effects']['approved'],'source':'effects-source-v1.png','source_sha256':hashlib.sha256(source.read_bytes()).hexdigest(),'file':'../docs/contender-shadow-review/effects-proposals.png','frames':frames,'sizes':{n:list(s) for n,_,s in specs},'world_units_per_pixel':1000,'generation_deterministic':False,'extraction_deterministic':True}
(OUT/'effect-review-manifest.json').write_text(json.dumps(meta,indent=2)+'\n');(ROOT/'editor/contender-effect-concepts.js').write_text('window.CONTENDER_EFFECT_CONCEPTS='+json.dumps(meta,separators=(',',':'))+';\n')
print(json.dumps({'review_only':True,'materials':len(frames),'source_sha256':meta['source_sha256']}))
