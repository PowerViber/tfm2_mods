"""Original pixel artwork for the scientist trio, notebooks and scientific effects."""
import json, math, importlib.util
from types import SimpleNamespace
from pathlib import Path
from PIL import Image, ImageDraw, ImageFont
ROOT=Path(__file__).resolve().parents[2]; HERE=Path(__file__).resolve().parent
DATA=json.loads((HERE/'catalogue.json').read_text()); MOD=ROOT/'mods/tfm2_custom'
COLORS=['#63bfff','#ffc96b','#63e1bd']; DARK='#101e30'; INK='#dcecf4'
FONT=ImageFont.truetype('/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf',9)
SMALL=ImageFont.truetype('/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf',8)
def atlas(folder,name,entries,w,h,cols=8,trim=False,palette=False):
    frames=sum(len(v) for v in entries.values());anims={};n=0
    sheet=Image.new('RGBA',(2048,2048) if trim else (cols*w,math.ceil(frames/cols)*h))
    cx=cy=row=used=0
    for tag, seq in entries.items():
        # One symmetric crop shared by all eight frames: their origin and scale
        # never wobble. Include every nonzero halo pixel and a transparent gutter.
        fw,fh=w,h
        if trim:
            boxes=[im.getbbox() for im,_ in seq if im.getbbox()]
            if boxes:
                fw=min(w,2*math.ceil(max(max(w/2-b[0],b[2]-w/2) for b in boxes)+1))
                fh=min(h,2*math.ceil(max(max(h/2-b[1],b[3]-h/2) for b in boxes)+1))
            else:fw=fh=2
        out=[]
        for im,duration in seq:
            if trim:
                if cx+fw+2>2048:cx=0;cy+=row;row=0
                x,y=cx+1,cy+1;assert y+fh+1<=2048,(name,tag)
                im=im.crop(((w-fw)//2,(h-fh)//2,(w+fw)//2,(h+fh)//2))
                cx+=fw+2;row=max(row,fh+2);used=max(used,cx)
            else:x=n%cols*w;y=n//cols*h
            sheet.alpha_composite(im,(x,y));out.append({'duration':duration,'data':{'x':x,'y':y,'w':fw,'h':fh}});n+=1
        anims[tag]={'frames':out}
    if trim:sheet=sheet.crop((0,0,used,cy+row))
    if palette:sheet=sheet.quantize(colors=256,method=Image.Quantize.FASTOCTREE,dither=Image.Dither.NONE)
    folder.mkdir(parents=True,exist_ok=True); assert max(sheet.size)<=2048,(name,sheet.size)
    sheet.save(folder/(name+'#sheet.png'));(folder/(name+'#anim.fanim')).write_text(json.dumps({'anims':anims},indent=2)+'\n')
_spec=importlib.util.spec_from_file_location('science_compact_art',HERE/'compact_art.py')
compact=importlib.util.module_from_spec(_spec);_spec.loader.exec_module(compact)
def body(kind='idle',f=0,persona=None):
    return compact.body(kind,f,persona)
def overlay(science,rank=0,f=0):
    return compact.costume(science,rank,f)
def effects():
    entries={}
    for tag in ['cast0','cast1','cast2','shield','fizzle']+['packet'+str(i) for i in range(4)]+['field'+str(i) for i in [1,5,12,13,18,20,45,57,58,65,66]]:
        seq=[]
        for f in range(8):
            im=Image.new('RGBA',(96,96));d=ImageDraw.Draw(im);c=COLORS[2 if tag in ('field57','field58','field65','field66','packet3','packet1') else 1 if tag in ('packet2','field45','cast1') else 0]
            alpha=min(255,180+f*10);rgb=tuple(int(c[i:i+2],16) for i in (1,3,5));rgba=rgb+(alpha,)
            if tag.startswith('packet'):
                d.line((29-f*2,48,58,48),fill=rgba,width=3);d.polygon([(58,42),(67,48),(58,54),(53,48)],fill=c);d.line((31,44,43,44),fill=rgb+(90,),width=1)
            elif tag=='field57':
                d.polygon([(48,18),(64,38),(60,68),(38,74),(30,40)],fill=rgb+(50,),outline=c);d.line((48,18,48,62,60,68),fill=rgba,width=2);d.line((30,40,64,38),fill=rgba)
            elif tag=='field58':
                for y in range(24,74,12):d.line((24,y,72,y),fill=rgba)
                for x in range(24,74,12):d.line((x,24,x,72),fill=rgba)
            elif tag in ('field65','field66'):
                for i in range(8):
                    a=i*math.pi/4+f*.1;x=48+int(math.cos(a)*(22+f*2));y=48+int(math.sin(a)*(15+f*2));d.ellipse((x-7,y-6,x+7,y+6),fill=rgb+(65,));d.point((x,y),fill=c)
            elif tag=='fizzle':
                for i in range(6):
                    a=i*math.pi/3;d.line((48+int(math.cos(a)*8),48+int(math.sin(a)*8),48+int(math.cos(a)*(22+f*3)),48+int(math.sin(a)*(22+f*3))),fill='#ff798e',width=2)
            elif tag=='shield':
                d.polygon([(48,15),(72,25),(68,59),(48,79),(28,59),(24,25)],fill=rgb+(30,),outline=c);d.line((48,22,48,64),fill=rgba,width=2)
            else:
                r=25+f*2;d.ellipse((48-r,48-r,48+r,48+r),outline=rgba,width=2)
                for i in range(6):
                    a=i*math.pi/3+f*.12;x=48+int(math.cos(a)*r);y=48+int(math.sin(a)*r);d.rectangle((x-2,y-2,x+2,y+2),fill=c)
                if tag=='field13':d.ellipse((42,15,54,81),outline=c,width=1)
                if tag=='field12':d.ellipse((36,36,60,60),fill=(6,12,22,200),outline=c)
                if tag=='field45':d.line((25,48,71,48),fill=c);d.line((48,25,48,71),fill=c)
            seq.append((im,.03125 if tag=='fizzle' else .0375 if tag=='shield' else .1 if tag.startswith('field') else .06))
        entries[tag]=seq
    return entries
def generate():
    entries={}
    for kind in ['idle','run','attack','skill1','skill2','ult','hit','dead']:
        entries[kind]=[(body(kind,f),.1) for f in range(8)]
    atlas(MOD/'champions','tfm2_custom_unified_theory',entries,48,64)
    personas={'persona'+str(i):[(overlay(i),.2)] for i in range(3)}
    atlas(MOD/'vfx','science_personas',personas,48,64)
    atlas(MOD/'vfx','science_vfx',effects(),96,96)
    for domain in range(3):
        notes={}
        for s in DATA['skills']:
            if s['science']!=domain:continue
            for k in range(len(s['tokens'])+1):
                im=Image.new('RGBA',(224,24));d=ImageDraw.Draw(im);d.rounded_rectangle((0,0,223,23),3,fill=(10,21,35,235),outline=COLORS[domain]);d.text((5,2),s['id']+' '+s['name'][:22],font=SMALL,fill=COLORS[domain]);
                token=' '.join((t[:4]+':'+str(a)) for t,a in zip(s['tokens'][:k],s['charge'][:k]));d.text((5,12),'~'+token,font=SMALL,fill=INK);d.rectangle((217,5,220,18),fill=COLORS[domain] if k==len(s['tokens']) else '#32495e');notes[f"note_{s['id']}_{k}"]=[(im,.18)]
        im=Image.new('RGBA',(224,24));d=ImageDraw.Draw(im);d.rounded_rectangle((0,0,223,23),3,fill=(10,21,35,220),outline=COLORS[domain]);d.text((6,7),['E / RELATIVITY','N / MECHANICS','C / CHEMISTRY'][domain]+'  /  READY',font=SMALL,fill=COLORS[domain]);notes['note_blank'+str(domain)]=[(im,.18)]
        atlas(MOD/'vfx','science_notebook'+str(domain),notes,224,24)
    meters={}
    for n in range(11):
        im=Image.new('RGBA',(224,18));d=ImageDraw.Draw(im);d.rounded_rectangle((0,0,223,17),3,fill=(10,21,35,235));d.text((5,3),f'CU {n*10:3}',font=SMALL,fill=INK);d.rectangle((46,5,210,12),fill='#2c4055');
        if n:d.rectangle((46,5,46+n*16,12),fill=COLORS[0]);meters['meter'+str(n)]=[(im,.18)]
    atlas(MOD/'vfx','science_meters',meters,224,18)
    allocations={}
    for n in range(101):
        for level,label in enumerate(['STABLE','STRAINED','UNSTABLE','CRITICAL']):
            im=Image.new('RGBA',(224,18));d=ImageDraw.Draw(im);d.rounded_rectangle((0,0,223,17),3,fill=(10,21,35,240));d.text((5,3),f'ALLOC {n:3} CU / {label}',font=SMALL,fill=['#63e1bd','#ffc96b','#ffac7b','#ff798e'][level]);allocations[f'allocation_{n}_{level}']=[(im,.18)]
    atlas(MOD/'vfx','science_allocations',allocations,224,18)
    # Portrait triptych shares the exact game silhouette, enlarged without smoothing.
    portrait=Image.new('RGBA',(960,360),'#0b1422');d=ImageDraw.Draw(portrait)
    for i,name in enumerate(['EINSTEIN','NEWTON','MARIE CURIE']):
        x=i*320;d.rounded_rectangle((x+12,12,x+308,348),12,fill='#142338',outline=COLORS[i],width=2)
        for r in (70,100):d.ellipse((x+160-r,160-r,x+160+r,160+r),outline='#294059',width=1)
        portrait.alpha_composite(body('idle',0,i).resize((192,256),Image.Resampling.NEAREST),(x+64,42));d.text((x+25,318),name,font=ImageFont.truetype('/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf',18),fill=COLORS[i])
    portrait.save(ROOT/'editor/science-portraits.png')
    spec=importlib.util.spec_from_file_location('science_mastery_art',HERE/'mastery_art.py')
    mastery=importlib.util.module_from_spec(spec);spec.loader.exec_module(mastery)
    mastery.generate(SimpleNamespace(DATA=DATA,atlas=atlas,body=body,overlay=overlay,transform=compact.transformation,compact_previews=lambda:compact.previews(ROOT)))
    print('Original body, three personas, notebooks, meter and mastery art generated')
if __name__=='__main__':generate()
