"""Assemble review GIFs from the actual packaged frames; never draws body pixels."""
from pathlib import Path
import json
from PIL import Image, ImageDraw, ImageFont
ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/'docs/contender-merge-integration-review';OUT.mkdir(exist_ok=True)
m=json.loads((ROOT/'docs/contender-shadow-review/presentation.json').read_text())
sheets={k:Image.open(ROOT/m[k]['file'].removeprefix('../')).convert('RGBA') for k in ['body','merge']}
font=ImageFont.truetype('C:/Windows/Fonts/arial.ttf',14)
for pose in ['idle','run','attack','projection']:
    frames=[];durations=[];clock=0
    for n,f in enumerate(m['body']['anims'][pose]['frames']):
        out=Image.new('RGBA',(1000,580));d=ImageDraw.Draw(out)
        for row in range(2):
            d.rectangle((0,row*290,1000,(row+1)*290),fill='#bac5a6' if row else '#29454a')
            for col,group in enumerate(['body','merge']):
                d.text((col*500+20,row*290+12),('Normal' if col==0 else 'Merged')+' / '+pose,fill='#152b35' if row else '#f4e9d4',font=font)
                r=m[group]['anims'][pose+('_l' if row else '')]['frames'][n]['data'];tile=sheets[group].crop((r['x'],r['y'],r['x']+r['w'],r['y']+r['h']))
                for scale in [1,3]:
                    im=tile.resize((80*scale,64*scale),Image.Resampling.NEAREST)
                    out.alpha_composite(im,(col*500+(120 if scale==1 else 330)-im.width//2,row*290+155-im.height//2))
        frames.append(out.convert('RGB'))
        end=clock+f['duration']*6000
        durations.append(round(end)*10-round(clock)*10);clock=end
    frames[0].save(OUT/f'merge-{pose}.gif',save_all=True,append_images=frames[1:],duration=durations,loop=0,disposal=2)
print('Packaged idle/run/attack/Merge-cast animation previews; GIF timing rounded to 10 ms, Lab clocks remain exact.')
