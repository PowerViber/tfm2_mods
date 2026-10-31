"""Preview a 30% larger copy of the existing approved Wings of Light. No new effect drawing."""
from pathlib import Path
from PIL import Image,ImageDraw
import json,hashlib,subprocess,io
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/vader-aegis-scribble-review'
BASE='00fe08dbbf2f09c560121e11a1c972dff8d92ad3'
def original(path):return subprocess.check_output(['rtk','proxy','git','show',BASE+':'+path],cwd=ROOT)
source=original('mods/tfm2_custom/vfx/gundam#sheet.png');sheet=Image.open(io.BytesIO(source)).convert('RGBA');meta=json.loads(original('mods/tfm2_custom/vfx/gundam#anim.fanim'))['anims']
body=Image.open(OUT/'aegis/pose-6.png').convert('RGBA')
def image(n,scale):
    r=meta['wings_light']['frames'][n]['data'];wing=sheet.crop((r['x'],r['y'],r['x']+r['w'],r['y']+r['h']))
    wing=wing.resize((round(wing.width*scale),round(wing.height*scale)),Image.Resampling.NEAREST)
    # Keep the shoulder joint fixed (legacy wing root: centre, eight pixels up).
    im=Image.new('RGBA',(260,220));anchor=(130,120);root=(130,96)
    im.alpha_composite(wing,(root[0]-round(72*scale),root[1]-round(56*scale)))
    im.alpha_composite(body,(anchor[0]-40,anchor[1]-72));return im
comparison=Image.new('RGB',(1040,500),'#17382e');d=ImageDraw.Draw(comparison)
for i,scale in enumerate([1.,1.3]):
    im=image(2,scale);x=i*520;d.text((x+15,12),'Current wings'if i==0 else'Proposed wings: +30% width and height',fill='#fff0d4');comparison.paste(im,(x,20),im);large=im.resize((520,440),Image.Resampling.NEAREST);comparison.paste(large,(x,100),large)
comparison.save(OUT/'aegis-wings-comparison.png')
frames=[]
for n in range(8):
    p=Image.new('RGB',(1040,500),'#17382e');d=ImageDraw.Draw(p)
    for i,scale in enumerate([1.,1.3]):
        im=image(n,scale).resize((520,440),Image.Resampling.NEAREST);p.paste(im,(i*520,40),im);d.text((i*520+15,15),'Current'if i==0 else'+30% Wings of Light',fill='white')
    frames.append(p)
frames[0].save(OUT/'aegis-wings-preview.gif',save_all=True,append_images=frames[1:],duration=70,loop=0)
(OUT/'aegis-wings.json').write_text(json.dumps({'requested':'Larger ultimate wings','approvedScale':1.3,'baseline':BASE,'source':'mods/tfm2_custom/vfx/gundam#sheet.png','method':'Nearest-neighbour scaling of existing wing pixels, fixed shoulder roots; no effect redraw','warningHitboxesChanged':False,'integrated':True,'sourceSha256':hashlib.sha256(source).hexdigest()},indent=2)+'\n')
print('Larger-wing static and animated comparisons generated')
