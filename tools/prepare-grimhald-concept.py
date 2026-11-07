"""Mechanical extraction of frozen imagegen proposal; no shipped assets or animation completion."""
from pathlib import Path
from PIL import Image, ImageDraw, ImageOps
import hashlib,json,shutil,argparse
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/grimhald-art-review';OUT.mkdir(exist_ok=True)
p=argparse.ArgumentParser();p.add_argument('--source',type=Path);p.add_argument('--version',default='v1',choices=['v1','v2']);args=p.parse_args()
version=args.version;source=OUT/f'source-imagegen-{version}.png'
if args.source:
    if source.exists():assert source.read_bytes()==args.source.read_bytes()
    else:shutil.copyfile(args.source,source)
im=Image.open(source).convert('RGBA');assert im.getchannel('A').getextrema()[0]==0
frames=[]
for index,name in enumerate(['front','right']):
    crop=im.crop((index*im.width//2,0,(index+1)*im.width//2,im.height));box=crop.getbbox();crop=crop.crop(box)
    # The crown starts around 23% down the original image; the blade extends higher.
    scale=39/(box[3]-im.height*.225)
    size=(round(crop.width*scale),round(crop.height*scale));small=crop.resize(size,Image.Resampling.NEAREST)
    foot=crop.getchannel('A').crop((0,max(0,crop.height-16),crop.width,crop.height)).getbbox()
    foot_x=(foot[0]+foot[2])/2*scale
    frame=Image.new('RGBA',(64,64));frame.alpha_composite(small,(round(32-foot_x),52-small.height))
    assert frame.getbbox()[1]>=0;frame.save(OUT/f'proposal-{version}-{name}.png');frames.append(frame)
ImageOps.mirror(frames[1]).save(OUT/f'proposal-{version}-left.png')
stock=[]
for name in ['knight','berserker']:
    base=ROOT/'Sprite kit/base champions'/name;d=json.loads(base.with_suffix('.anim.json').read_text())['anims']['idle']['frames'][0]['data']
    s=Image.open(base.with_suffix('.png')).convert('RGBA').crop((d['x'],d['y'],d['x']+d['w'],d['y']+d['h']));s=s.crop(s.getbbox())
    stock.append(s)
sheet=Image.new('RGB',(1200,1400),'#14252d');d=ImageDraw.Draw(sheet)
for row,bg in enumerate(['#17333d','#b7c2ae']):
    d.rectangle((0,row*700,1199,row*700+699),fill=bg)
    d.text((16,row*700+10),f'GRIMHALD concept {version} - 1x / 3x / 6x; appearance approval pending',fill='#f7d797' if row==0 else '#15232b')
    for i,f in enumerate(frames+[ImageOps.mirror(frames[1])]):
        x=20+i*400;y=row*700+32
        for scale_,dx,dy in [(1,0,0),(3,90,0),(6,0,220)]:
            if scale_==6:dx,dy=0,220
            a=f.resize((64*scale_,64*scale_),Image.Resampling.NEAREST)
            sheet.paste(a,(x+dx,y+dy),a)
    # Actual size stock comparisons are in a separate native-scale strip below.
sheet.save(OUT/f'concept-imagegen-{version}.png')
strip=Image.new('RGB',(440,110),'#20333b');d=ImageDraw.Draw(strip)
for x,f,label in [(20,frames[1],'Grimhald'),(130,stock[0],'Knight'),(240,stock[1],'Berserker')]:
    strip.paste(f,(x,64-(52 if label=='Grimhald' else f.height)),f);d.text((x,76),label,fill='#eee0bd')
strip.save(OUT/f'size-imagegen-{version}.png')
manifest={'status':'appearance-approval-pending','sourceSha256':hashlib.sha256(source.read_bytes()).hexdigest(),'frame':[64,64],'footBaseline':52,'anchorBelowCentre':20,'bodyVisibleHeightTarget':39,'method':'Frozen original imagegen source, uniform nearest-neighbour extraction, transparent padding; generation itself is not deterministic','references':['https://www.leagueoflegends.com/en-us/champions/darius/','https://www.leagueoflegends.com/en-gb/champions/ksante/','https://www.leagueoflegends.com/en-us/champions/mordekaiser/']}
(OUT/f'concept-{version}-manifest.json').write_text(json.dumps(manifest,indent=2)+'\n');print(json.dumps(manifest))
