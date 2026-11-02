"""Freeze original imagegen proposals and mechanically extract a preview-only atlas.

Generation is not deterministic. Extraction/placement is deterministic from the
frozen sources and their recorded hashes. No shipped asset is written here.
"""
from pathlib import Path
from PIL import Image, ImageDraw
import argparse, hashlib, json, shutil

ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/'docs/rally-results/rally-3.2'
parser=argparse.ArgumentParser()
parser.add_argument('--impact-source',type=Path)
parser.add_argument('--storm-source',type=Path)
args=parser.parse_args()
OUT.mkdir(parents=True,exist_ok=True)
for name,provided in [('impact-source.png',args.impact_source),('storm-source.png',args.storm_source)]:
    if provided:
        target=OUT/name
        if target.exists():
            assert target.read_bytes()==provided.read_bytes(),'Frozen source differs; use a new named revision'
        else: shutil.copyfile(provided,target)
impact=Image.open(OUT/'impact-source.png').convert('RGBA')
storm=Image.open(OUT/'storm-source.png').convert('RGBA')
assert impact.getchannel('A').getextrema()[0]==0 and storm.getchannel('A').getextrema()[0]==0
sheet=Image.new('RGBA',(900,396));anims={};frames=[]

def tag(name,x,y,w,h,duration=2/60):
    anims[name]={'frames':[{'data':dict(x=x,y=y,w=w,h=h),'duration':duration}]}

for phase in range(3):
    x0=round(phase*impact.width/3);x1=round((phase+1)*impact.width/3)
    cell=impact.crop((x0,0,x1,impact.height))
    # The reference ball centres sit slightly below each panel's midpoint.
    square=Image.new('RGBA',(cell.width,cell.width))
    square.alpha_composite(cell,(0,round(cell.width/2-cell.height*.545)))
    image=square.resize((300,300),Image.Resampling.NEAREST)
    sheet.alpha_composite(image,(phase*300,0))
    image.save(OUT/f'impact-frame-{phase}.png')
    tag(f'impact_release_p{phase}',phase*300+86,86,128,128)
    for radius in [80000,96000,120000,144000]:
        diameter=round(2*radius/960)
        offset=(300-diameter)//2
        # Centre crops share stored pixels; no additional scaled frame copies.
        tag(f'impact_landing_{radius}_p{phase}',phase*300+offset,offset,diameter,diameter)
    frames.append(image)
for phase in range(8):
    x0=round(phase*storm.width/8);x1=round((phase+1)*storm.width/8)
    image=storm.crop((x0,0,x1,storm.height)).resize((32,96),Image.Resampling.NEAREST)
    sheet.alpha_composite(image,(phase*32,300))
    tag(f'storm_bolt_p{phase}',phase*32,300,32,96)
    image.save(OUT/f'storm-bolt-{phase}.png')
warning=Image.new('RGBA',(32,32));draw=ImageDraw.Draw(warning)
draw.ellipse((3,3,28,28),outline='#c8c6ff',width=1)
sheet.alpha_composite(warning,(300,300))
for phase in range(8):tag(f'storm_warning_p{phase}',300,300,32,32)
dot=Image.new('RGBA',(8,8));ImageDraw.Draw(dot).rectangle((2,2,5,5),fill='#bce9f6')
sheet.alpha_composite(dot,(340,300));tag('range_dot',340,300,8,8)
for i,ch in enumerate('0123456789.km/h'):
    glyph=Image.new('RGBA',(8,16));ImageDraw.Draw(glyph).text((0,1),ch,fill='#fff4bb')
    sheet.alpha_composite(glyph,(360+i*8,300));tag('hud_'+str(i),360+i*8,300,8,16)
target=OUT/'packed-preview/vfx';target.mkdir(parents=True,exist_ok=True)
sheet.save(target/'rally-impact#sheet.png')
(target/'rally-impact#anim.fanim').write_text(json.dumps({'anims':anims},separators=(',',':'))+'\n')
review=Image.new('RGB',(900,600),'#17333d')
for i,image in enumerate(frames):
    review.paste(image,(300*i,0),image)
    ImageDraw.Draw(review).rectangle((300*i,300,300*i+299,599),fill='#b9c5b1')
    review.paste(image,(300*i,300),image)
review.save(OUT/'impact-contact-sheet.png')
gif=[]
for colour in ['#17333d','#b9c5b1']:
    for image in frames:
        frame=Image.new('RGB',(300,300),colour);frame.paste(image,(0,0),image);gif.append(frame)
gif[0].save(OUT/'impact-reference-preview.gif',save_all=True,append_images=gif[1:],duration=[34,33,33]*2,loop=0,disposal=2)
manifest={
    'scope':'Unapproved original effect proposals; no shipped integration',
    'generation':'Built-in imagegen; generation itself is not deterministic',
    'extraction':'Three equal impact panels, centred vertically at 54.5%; nearest-neighbour resampling; eight equal storm panels',
    'sourceSha256':{name:hashlib.sha256((OUT/name).read_bytes()).hexdigest() for name in ['impact-source.png','storm-source.png']},
    'atlasSha256':hashlib.sha256((target/'rally-impact#sheet.png').read_bytes()).hexdigest(),
    'size':sheet.size,'impactFrameTicks':2,'impactDrawings':3,
    'references':[
        {'url':'https://www.youtube.com/watch?v=CtO_XE2Nrho&t=15s','observed':'15.2–15.8: colour to rough monochrome contours and reversed contrast'},
        {'url':'https://www.youtube.com/watch?v=jJMDwMRycd4&t=139s','observed':'Visual chapters: first drawing 0:37; sequence 1:15; finalization 2:19; polish 2:56. No transcript available.'}
    ],
    'adaptation':'Original volleyball/lightning drawings; 3 × 2 ticks at release/landing are mod timing choices, not copied source frames',
}
(OUT/'effect-source-manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
print(json.dumps(manifest))
