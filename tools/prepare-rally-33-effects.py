"""Unapproved contact-frame, charge-aura and geometric-crater preview. No shipped writes."""
from pathlib import Path
from PIL import Image,ImageDraw
import hashlib,json
from rally_art_packing import pack
ROOT=Path(__file__).resolve().parents[1];OLD=ROOT/'docs/rally-results/rally-3.2';OUT=ROOT/'docs/rally-results/rally-3.3'
OUT.mkdir(parents=True,exist_ok=True)
images=[];meta={};dedup={}
def add(tag,image,duration=2/60):
    key=(image.size,image.tobytes())
    if key not in dedup:dedup[key]=len(images);images.append(image)
    meta[tag]={'frames':[{'data':dict(index=dedup[key],w=image.width,h=image.height),'duration':duration}]}
# One large 300px reversed-contrast drawing at ball contact, held for six ticks.
contact=Image.open(OLD/'impact-frame-1.png').convert('RGBA')
add('impact_release_p0',contact,6/60)
contact.save(OUT/'contact-frame-art.png')
for phase in range(8):
    add('storm_bolt_p'+str(phase),Image.open(OLD/f'storm-bolt-{phase}.png').convert('RGBA'))
warning=Image.new('RGBA',(44,44));ImageDraw.Draw(warning).ellipse((3,3,40,40),outline='#c8c6ff',width=1)
for phase in range(8):add('storm_warning_p'+str(phase),warning)
dot=Image.new('RGBA',(8,8));ImageDraw.Draw(dot).rectangle((2,2,5,5),fill='#bce9f6');add('range_dot',dot)
for i,ch in enumerate('0123456789.km/h'):
    glyph=Image.new('RGBA',(8,16));ImageDraw.Draw(glyph).text((0,1),ch,fill='#fff4bb');add('hud_'+str(i),glyph)
for stage in range(7):
    size=40+stage*16;ring=Image.new('RGBA',(size,size));d=ImageDraw.Draw(ring)
    d.ellipse((2,2,size-3,size-3),outline='#eee9ff' if stage%2 else '#bfa7ed',width=1+stage//3)
    if stage>2:d.ellipse((7,7,size-8,size-8),outline='#8f82d8',width=1)
    add('charge_ring_'+str(stage),ring)
# Hand-authored pixel paths: no imagegen texture, noise, cloud or painted crater.

palettes={'normal':('#17242a','#7c9694'),'yellow':('#29251b','#d7b94d'),'black':('#25171e','#b54752'),'white':('#20303b','#c9e7ed'),'impossible':('#201d31','#bfa7ed')}

ground=[]

for style,(dark,light) in palettes.items():

    image=Image.new('RGBA',(128,128));d=ImageDraw.Draw(image)

    silhouette=[(42,44),(62,38),(79,43),(88,59),(82,78),(65,86),(45,80),(38,65)]

    if style=='normal':silhouette=[(48,56),(62,49),(78,57),(76,72),(59,77),(46,68)]

    d.polygon(silhouette,fill=dark+'b0')

    paths=[[(64,62),(78,58),(86,48),(104,44)],[(68,66),(84,72),(88,88),(105,98)],[(61,67),(55,84),(42,88),(34,105)],[(59,60),(44,54),(36,40),(20,34)]]

    if style in ['black','white','impossible']:paths += [[(65,58),(67,44),(59,35),(64,17)],[(71,64),(89,61),(98,68),(114,63)],[(63,72),(70,88),(63,99),(68,117)],[(55,66),(40,72),(27,65),(12,72)]]

    if style=='yellow':paths=paths[:3]

    if style=='normal':paths=[[(47,69),(58,63),(72,62),(83,56)],[(52,73),(63,68),(75,68)]]

    for n,line in enumerate(paths):

        d.line(line,fill=dark,width=3);d.line(line,fill=light,width=1)

        if style in ['white','impossible']:

            x,y=line[2];d.line([(x,y),(x-9,y+3),(x-12,y+10)],fill=light,width=1)

    for x,y in [(42,33),(93,45),(97,92),(37,94)]:d.rectangle((x,y,x+3,y+2),fill=light)

    if style=='impossible':d.line([(52,48),(75,45),(87,60),(81,79),(57,84),(43,65),(52,48)],fill='#eee9ff',width=1)

    add('ground_'+style,image);image.save(OUT/f'ground-{style}.png');ground.append(image)

    chip=Image.new('RGBA',(16,16));cd=ImageDraw.Draw(chip);cd.line([(1,12),(6,8),(8,3),(14,1)],fill=dark,width=3);cd.line([(1,12),(6,8),(8,3),(14,1)],fill=light,width=1);add('chip_'+style,chip)

size,spots=min((pack(images,w) for w in [512,640,768,900,1024]),key=lambda item:item[0][0]*item[0][1]);sheet=Image.new('RGBA',size)
for i,im in enumerate(images):sheet.alpha_composite(im,spots[i])
for anim in meta.values():
    for frame in anim['frames']:
        d=frame['data'];d['x'],d['y']=spots[d.pop('index')]
out=OUT/'packed-preview/vfx';out.mkdir(parents=True,exist_ok=True)
sheet.save(out/'rally-impact#sheet.png');(out/'rally-impact#anim.fanim').write_text(json.dumps({'anims':meta},separators=(',',':'))+'\n')
review=Image.new('RGB',(640,320),'#17333d')
for i,image in enumerate(ground):
    review.paste(image,(i*128,0),image);ImageDraw.Draw(review).rectangle((i*128,160,i*128+127,319),fill='#b9c5b1');review.paste(image,(i*128,160),image)
review.save(OUT/'crater-contact-sheet.png')
report={'scope':'Unapproved presentation revision; no shipped assets','impact':'One large ball-contact drawing held for 6 ticks / 0.1 seconds. No two-second sequence, map overlay or landing impact frame.',
'charge':'Five seconds; fixed-size ball, escalating compression rings and 2-8 lightning strokes.',
'craters':'Hand-authored geometric pixel paths by actual attack tier.',
'contactSourceSha256':hashlib.sha256((OLD/'impact-frame-1.png').read_bytes()).hexdigest(),
'previousImpactSource':'../rally-3.2/effect-source-manifest.json','atlasSize':sheet.size,'atlasSha256':hashlib.sha256((out/'rally-impact#sheet.png').read_bytes()).hexdigest()}
(OUT/'effect-source-manifest.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report))
