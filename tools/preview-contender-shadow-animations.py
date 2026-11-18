"""Contact sheet and animated whole-pose inspection from the actual packaged atlas."""
from pathlib import Path
import json
from PIL import Image,ImageDraw,ImageFont
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/contender-shadow-review';N=Image.Resampling.NEAREST
m=json.loads((OUT/'presentation.json').read_text());s=Image.open(ROOT/m['body']['file'].removeprefix('../')).convert('RGBA');anims=m['body']['anims']
font=ImageFont.truetype('C:/Windows/Fonts/arial.ttf',15)
poses=['idle','run','attack','projection','cyclone','slam','uppercut','flurry','hit','dead']
def frame(tag,tick):
    a=anims[tag];total=sum(f['duration']*60 for f in a['frames']);tick=tick%total
    for f in a['frames']:
        if tick<f['duration']*60:break
        tick-=f['duration']*60
    r=f['data'];return s.crop((r['x'],r['y'],r['x']+r['w'],r['y']+r['h']))
frames=[]
for t in range(48):
    im=Image.new('RGBA',(1700,540),'#20343e');d=ImageDraw.Draw(im)
    for i,pose in enumerate(poses):
        x=i%5*340;y=i//5*270
        d.text((x+8,y+5),pose,font=font,fill='#eee5f3')
        for left in [False,True]:
            tile=frame(pose+('_l' if left else ''),t);xx=x+45+left*170
            im.alpha_composite(tile,(xx,y+36))
            # 2x on light terrain, both facings. No resampling blur.
            d.rectangle((xx-40,y+112,xx+120,y+256),fill='#b9c5a7')
            big=tile.resize((160,128),N);im.alpha_composite(big,(xx-40,y+119))
    frames.append(im.convert('RGB'))
frames[0].save(OUT/'animation-contact-sheet.png')
frames[0].save(OUT/'animation-preview.gif',save_all=True,append_images=frames[1:],duration=[80,85]*24,loop=0,disposal=2)
print('Generated packed-animation contact sheet and 4-second GIF')
