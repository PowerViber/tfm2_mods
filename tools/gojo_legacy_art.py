"""Pre-PR40 pixels are the canonical style. New states use the same compact pixel grammar."""
from pathlib import Path
from PIL import Image,ImageDraw
import json,math
ROOT=Path(__file__).resolve().parents[1]
SOURCE=Image.open(ROOT/'mods/tfm2_jjk/vfx/gojo#sheet.png').convert('RGBA')
META=json.loads((ROOT/'mods/tfm2_jjk/vfx/gojo#anim.fanim').read_text())
def original(tag,n):
    fs=META['anims'][tag]['frames'];r=fs[n%len(fs)]['data']
    return SOURCE.crop((r['x'],r['y'],r['x']+r['w'],r['y']+r['h']))
def fit(im,diameter):
    box=im.getbbox()
    if not box:return im
    # Scale the visible silhouette rather than transparent atlas padding.
    cx,cy=im.width/2,im.height/2
    half=math.ceil(max(cx-box[0],box[2]-cx,cy-box[1],box[3]-cy))
    im=im.crop((round(cx-half),round(cy-half),round(cx+half),round(cy+half)))
    return im.resize((diameter,diameter),Image.Resampling.NEAREST)
def effect(tag,n):
    mappings={'blue_flight':('blue',18),'max_blue_flight':('blue',22),'blue_impact':('blue_hole',round(64000/960)),'max_blue_impact':('blue_hole',round(72000/960)),'red_flight':('red',round(22000/960)),'max_red_flight':('red',round(22000/960)),'red_impact':('red_burst',round(40000/960)),'max_red_impact':('red_burst',round(52000/960)),'purple_flight':('purple',round(52000/960)),'purple_amplified':('purple',round(52000/960)),'step_prepare':('blink',36),'step_arrive':('blink',36),'blue_prepare':('mini_blue',9),'max_blue_prepare':('blue',18),'red_prepare':('mini_red',9),'max_red_prepare':('red',20),'purple_prepare':('purple',26)}
    if tag in mappings:
        name,size=mappings[tag];return fit(original(name,n),size)
    size=202 if tag in ['fusion','nuke'] else 48
    im=Image.new('RGBA',(size,size));d=ImageDraw.Draw(im);c=size//2
    if tag in ['melee','enhanced','black_flash']:
        color=(235,242,255,255) if tag=='melee' else (80,185,255,255)
        if tag=='black_flash':
            d.ellipse((c-7,c-7,c+7,c+7),fill=(10,4,18,255));color=(215,20,50,255)
        for a in range(0,360,60):
            a=math.radians(a+n*5);r=7+n*2
            points=[(c+round(math.cos(a)*4),c+round(math.sin(a)*4)),(c+round(math.cos(a+.25)*r*.6),c+round(math.sin(a+.25)*r*.6)),(c+round(math.cos(a)*r),c+round(math.sin(a)*r))]
            d.line(points,fill=color,width=2)
    elif tag in ['rct','rct_prepare','recoil','infinity_contact','guest']:
        color=(140,230,255,230) if tag!='recoil' else (170,110,240,240)
        r=8+n*2;d.ellipse((c-r,c-r,c+r,c+r),outline=color,width=1)
        for q in range(4):
            x=c-12+q*8;y=c+12-(n*3+q*5)%28;d.rectangle((x,y,x+1,y+2),fill=color)
    elif tag=='orb':
        im.alpha_composite(fit(original('blue',n),17),(c-9,c-9))
        d.ellipse((c-13,c-13,c+13,c+13),outline=(45,105,225,190))
    elif tag in ['fusion','nuke']:
        # Exact 96,000-unit boundary at 960 units/pixel. Decoration stays within it.
        r=100 if tag=='fusion' else min(100,14+n*14)
        d.ellipse((c-r,c-r,c+r,c+r),outline=(165,85,240,235),width=2)
        if tag=='fusion':
            for q,name in enumerate(['mini_blue','mini_red']):
                orb=original(name,n);x=c-28+q*56+round((n/7)*28)*(1 if q==0 else -1);im.alpha_composite(orb,(x-orb.width//2,c-orb.height//2))
        else:
            burst=fit(original('purple_hit',n),max(12,r*2-8));im.alpha_composite(burst,(c-burst.width//2,c-burst.height//2))
            d.ellipse((c-100,c-100,c+100,c+100),outline=(165,85,240,150),width=1)
    return im
def install(module):
    module.draw_effect=effect
    previous=module.status
    def status(tag,n):
        if not tag.startswith('infinity_'):return previous(tag,n)
        im=Image.new('RGBA',(48,72));source=original('infinity',n)
        if tag in ['infinity_paused','infinity_break']:
            source.putalpha(source.getchannel('A').point(lambda a:a//3))
        im.alpha_composite(source,((48-source.width)//2,54-source.height))
        d=ImageDraw.Draw(im)
        if tag=='infinity_paused':d.rectangle((39,37,40,43),fill=(200,165,70,240));d.rectangle((43,37,44,43),fill=(200,165,70,240))
        if tag=='infinity_break':d.line((12,30,19,37,17,42,28,49),fill=(225,165,70,230),width=1)
        return im
    module.status=status
    # Existing poses already contain the original hands and technique gestures.
    module.gesture=lambda tag,n:Image.new('RGBA',(48,56))
