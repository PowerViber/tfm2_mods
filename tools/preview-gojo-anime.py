"""Before/after comparisons from the pinned preceding PR40 build."""
from pathlib import Path
from io import BytesIO
from PIL import Image,ImageDraw
import json,subprocess
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/gojo-results';BASE='ab44870bb25994db0b1ac4af096f41d74b95d245'
def assets(stem,old=False):
    def read(s):return subprocess.check_output(['rtk','proxy','git','show',BASE+':'+s],cwd=ROOT) if old else (ROOT/s).read_bytes()
    return Image.open(BytesIO(read(stem+'#sheet.png'))).convert('RGBA'),json.loads(read(stem+'#anim.fanim'))
def frame(asset,tag,n):
    sheet,meta=asset;frames=meta['anims'][tag]['frames'];r=frames[n%len(frames)]['data'];return sheet.crop((int(r['x']),int(r['y']),int(r['x']+r['w']),int(r['y']+r['h'])))
legacy=assets('mods/tfm2_jjk/vfx/gojo_rework',True);previous=assets('mods/tfm2_jjk/vfx/gojo_v2',True);current=assets('mods/tfm2_jjk/vfx/gojo_v2');body=assets('mods/tfm2_jjk/champions/tfm2_custom_gojo')
rows=[('Six Eyes',legacy,'fx0','six_eyes'),('Blue',legacy,'fx8','blue_impact'),('Red',legacy,'fx9','red_impact'),('Purple preparation',legacy,'fx7','purple_prepare'),('Black Flash',legacy,'fx2','black_flash'),('Full Void',previous,'void_full_field','void_full_field'),('Quick Void',previous,'void_quick_field','void_quick_field')]
animation=[]
for n in range(16):
    left=n>=8;canvas=Image.new('RGBA',(1000,7*238+42),(15,24,38,255));d=ImageDraw.Draw(canvas);d.text((20,14),'Previous PR40 / ab44870',fill='white');d.text((520,14),'Anime upgrade / actual pixels + enlarged detail',fill='white')
    for row,(title,old,oldtag,newtag) in enumerate(rows):
        y=42+row*238;bg=(179,177,151,255) if row%2 else (15,24,38,255);d.rectangle((0,y,999,y+237),fill=bg)
        for col,(asset,tag) in enumerate([(old,oldtag),(current,newtag)]):
            im=frame(asset,tag,n+4 if tag.startswith('void') else n);sprite=frame(body,'idle',n)
            if left:im=im.transpose(Image.Transpose.FLIP_LEFT_RIGHT);sprite=sprite.transpose(Image.Transpose.FLIP_LEFT_RIGHT)
            x=col*500+110;canvas.alpha_composite(sprite,(x-sprite.width//2,y+110-sprite.height//2));canvas.alpha_composite(im,(x-im.width//2,y+110-im.height//2))
            if not tag.startswith('void'):zoom=im.resize((im.width*2,im.height*2),Image.Resampling.NEAREST);canvas.alpha_composite(zoom,(col*500+340-zoom.width//2,y+110-zoom.height//2))
        d.text((14,y+220),title+(' / left' if left else ' / right'),fill='white' if row%2==0 else (15,24,38,255))
    animation.append(canvas.convert('RGB'))
animation[3].save(OUT/'anime-before-after.png');animation[0].save(OUT/'anime-before-after.gif',save_all=True,append_images=animation[1:],duration=110,loop=0)
print('Actual-size before/after PNG and animated GIF written')
