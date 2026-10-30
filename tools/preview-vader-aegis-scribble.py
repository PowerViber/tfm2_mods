"""Extract frozen imagegen proposals for review. Never replaces packaged artwork."""
from pathlib import Path
import importlib.util,json,hashlib,math
from PIL import Image,ImageDraw,ImageOps
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/vader-aegis-scribble-review'
spec=importlib.util.spec_from_file_location('body_tools',ROOT/'tools/original_bodies.py');body_tools=importlib.util.module_from_spec(spec);spec.loader.exec_module(body_tools)
SPECS={'vader':('Darth Vader',40),'aegis':('Aegis Zero / Destiny Gundam',46),'scribble':('Scribble / Gear 5 appearance',45)}
LABELS=['Idle','Walk','Attack A','Attack B','Cast A','Cast B','Ultimate','Hit']
def main():
    manifest={'status':'awaiting-user-preview-approval','method':'Frozen imagegen; connected alpha components, common per-character scale, foot registration and nearest-neighbour packing','gameplayChanged':False,'nativeDllChanged':False,'characters':{}}
    decisions=json.loads((OUT/'decisions.json').read_text(encoding='utf8'));sections=[];lineup=Image.new('RGB',(990,420),'#18372e');ld=ImageDraw.Draw(lineup)
    for j,(key,(title,height)) in enumerate(SPECS.items()):
        source=OUT/(key+'-source-v2.png')
        if not source.exists():source=OUT/(key+'-source.png')
        floating=key=='scribble'and(OUT/'scribble-source-floating.png').exists()
        if floating:source=OUT/'scribble-source-floating.png'
        if not source.exists():continue
        raw=body_tools.sprite_library(source,8);scale=height/raw[0].height;frames=[]
        for pose in raw:
            small=pose.resize((max(1,round(pose.width*scale)),max(1,round(pose.height*scale))),Image.Resampling.NEAREST)
            if floating:
                hair=[x for y in range(max(1,small.height//3))for x in range(small.width)if small.getpixel((x,y))[3]>=128 and min(small.getpixel((x,y))[:3])>180]
                anchor=sum(hair)/max(1,len(hair))
            else:anchor=body_tools.foot_x(small)
            f=Image.new('RGBA',(80,80));x=round(40-anchor);y=(66 if floating else 72)-small.height
            assert min(x,y)>=0 and x+small.width<=80,(key,small.size,x,y)
            f.alpha_composite(small,(x,y));frames.append(f)
        dest=OUT/key;dest.mkdir(exist_ok=True)
        for n,f in enumerate(frames):f.save(dest/f'pose-{n}.png')
        current=Image.open(OUT/(key+'-current.png')).convert('RGBA')
        labels=['Laughing hover','Glide','Gather','Weave','Release','Invoke','Hit','Hover alternate']if floating else LABELS
        sheet=Image.new('RGB',(1920,410),'#d7d5c1');d=ImageDraw.Draw(sheet)
        for n,f in enumerate(frames):
            x=n*240;d.text((x+8,8),labels[n],fill='#25354a');sheet.paste(f,(x+80,35),f)
            d.rectangle((x,135,x+239,409),fill='#18372e');z=f.resize((240,240),Image.Resampling.NEAREST);sheet.paste(z,(x,155),z)
        sheet.save(dest/'poses.png')
        compare=Image.new('RGB',(900,500),'#d7d5c1');d=ImageDraw.Draw(compare)
        for row,bg in [(0,'#d7d5c1'),(1,'#18372e')]:
            y=row*250;d.rectangle((0,y,899,y+249),fill=bg);fg='#25354a'if row==0 else'#f4ead0';d.text((16,y+12),title+' - current / proposed / mirrored / 3x',fill=fg)
            compare.paste(current,(35,y+115-current.getbbox()[3]),current);compare.paste(frames[0],(110,y+43),frames[0]);compare.paste(ImageOps.mirror(frames[0]),(200,y+43),ImageOps.mirror(frames[0]));z=frames[0].resize((240,240),Image.Resampling.NEAREST);compare.paste(z,(350,y+10),z);z=ImageOps.mirror(z);compare.paste(z,(620,y+10),z)
        compare.save(dest/'comparison.png')
        animated=[]
        for n in [0,1,0,1,2,3,4,5,6,7,0]:
            panel=Image.new('RGB',(600,300),'#18372e');d=ImageDraw.Draw(panel);d.text((12,8),title+' / '+labels[n]+' / both facings at 3x',fill='white');z=frames[n].resize((240,240),Image.Resampling.NEAREST);panel.paste(z,(20,40),z);z=ImageOps.mirror(z);panel.paste(z,(330,40),z);animated.append(panel)
        animated[0].save(dest/'keyposes.gif',save_all=True,append_images=animated[1:],duration=200,loop=0)
        x=j*330;ld.text((x+15,15),title,fill='#f4ead0');lineup.paste(current,(x+25,150-current.getbbox()[3]),current);lineup.paste(frames[0],(x+110,78),frames[0]);z=frames[0].resize((240,240),Image.Resampling.NEAREST);lineup.paste(z,(x+45,170),z)
        manifest['characters'][key]={'sourceSha256':hashlib.sha256(source.read_bytes()).hexdigest(),'visibleIdleHeight':height,'reviewCanvas':[80,80],'foot':[40,72],'approval':bool(decisions.get('revisionFeedback',{}).get(key,{}).get('appearanceApproved')),'scale':scale,'poses':8,'cosmeticHoverPixels':6 if floating else 0}
        extra='<p>30% larger existing Wings of Light; combat circles stay unchanged.</p><img src="/docs/vader-aegis-scribble-review/aegis-wings-comparison.png"><img src="/docs/vader-aegis-scribble-review/aegis-wings-preview.gif">'if key=='aegis'and(OUT/'aegis-wings-preview.gif').exists()else''
        if key=='scribble'and(OUT/'scribble-orb-presentation.png').exists():extra+='<img src="/docs/vader-aegis-scribble-review/scribble-orb-presentation.png">'
        if key=='vader'and(OUT/'vader/complete-animations.gif').exists():extra+='<p>Completed existing Vader timelines</p><img src="/docs/vader-aegis-scribble-review/vader/complete-animations.gif">'
        sections.append(f'<section><h2>{title} - '+('appearance approved'if key=='vader'else'revision for review')+f'</h2><img src="/docs/vader-aegis-scribble-review/{key}/comparison.png"><img src="/docs/vader-aegis-scribble-review/{key}/poses.png"><img src="/docs/vader-aegis-scribble-review/{key}/keyposes.gif">'+extra+'</section>')
    lineup.save(OUT/'lineup.png');(OUT/'review.json').write_text(json.dumps(manifest,indent=2)+'\n')
    html='<!doctype html><meta charset="utf-8"><title>Vader, Aegis Zero and Scribble redesign review</title><style>body{font:16px system-ui;background:#101c2c;color:#eee6d4;margin:24px auto;max-width:1320px}section{margin:32px 0;border-top:1px solid #47586e;padding-top:10px}img{image-rendering:pixelated;max-width:none;display:block;margin:16px 0}section{overflow-x:auto}a{color:#92cff0}</style><h1>Vader approved - floating Scribble and larger wings revised</h1><p>Simple blocky game-size sprites. Current and proposed bodies, both facings, light/dark backgrounds and key-pose animation. Scribble keeps his five-element / 35-spell orb kit and uses hovering gathering, weaving and invoking gestures. Aegis shows 30% larger existing wings. These previews do not replace game assets.</p>'+''.join(sections)
    (ROOT/'editor/character-redesign-review.html').write_text(html,encoding='utf8');print(json.dumps(manifest,indent=2))
if __name__=='__main__':main()
