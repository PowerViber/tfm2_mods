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
def atlas(folder,name,entries,w,h,cols=8):
    frames=sum(len(v) for v in entries.values()); sheet=Image.new('RGBA',(cols*w,math.ceil(frames/cols)*h)); anims={};n=0
    for tag, seq in entries.items():
        out=[]
        for im,duration in seq:
            x=n%cols*w;y=n//cols*h;sheet.alpha_composite(im,(x,y));out.append({'duration':duration,'data':{'x':x,'y':y,'w':w,'h':h}});n+=1
        anims[tag]={'frames':out}
    folder.mkdir(parents=True,exist_ok=True); assert max(sheet.size)<=2048,(name,sheet.size)
    sheet.save(folder/(name+'#sheet.png'));(folder/(name+'#anim.fanim')).write_text(json.dumps({'anims':anims},indent=2)+'\n')
def body(kind='idle',f=0,persona=None):
    im=Image.new('RGBA',(48,64));d=ImageDraw.Draw(im)
    # Crisp silhouette. Every persona has the same skeleton and hit silhouette.
    d.ellipse((11,56,37,60),fill=(6,14,24,100)); step=1 if kind=='run' and f%4<2 else -1 if kind=='run' else 0
    if kind=='dead':
        d.rounded_rectangle((8,46,40,55),2,fill=DARK);d.rectangle((9,43,24,49),fill='#e7ece7');d.ellipse((32,43,40,51),fill='#d6b69d');return im
    d.rectangle((17,42,22,55+step),fill='#243448');d.rectangle((26,42,31,55-step),fill='#243448')
    d.rectangle((15,55+step,23,58+step),fill=DARK);d.rectangle((25,55-step,33,58-step),fill=DARK)
    d.polygon([(16,25),(31,25),(35,45),(13,45)],fill='#e2e7dd',outline=DARK)
    d.rectangle((22,27,26,43),fill='#33465b');d.line((24,29,24,44),fill='#f4f4de');d.rectangle((14,34,19,36),fill='#6b7e8b')
    raised=kind in ('attack','skill1','skill2','ult')
    d.line((15,28,10,36 if not raised else 22),fill=DARK,width=6);d.line((15,28,10,35 if not raised else 22),fill='#e2e7dd',width=4)
    d.line((32,28,38,36 if not raised else 24),fill=DARK,width=6);d.line((32,28,38,35 if not raised else 24),fill='#e2e7dd',width=4)
    d.rectangle((8,33 if not raised else 19,12,37 if not raised else 23),fill='#d6b69d')
    d.rectangle((36,33 if not raised else 21,40,37 if not raised else 25),fill='#d6b69d')
    d.rounded_rectangle((17,9,31,26),4,fill=DARK);d.rounded_rectangle((18,10,30,25),4,fill='#d6b69d')
    d.line((19,17,22,17),fill=DARK);d.line((26,17,29,17),fill=DARK);d.point((23,21),fill='#896957')
    if persona is not None:im.alpha_composite(overlay(persona));
    return im
def overlay(science):
    im=Image.new('RGBA',(48,64));d=ImageDraw.Draw(im);c=COLORS[science]
    d.rectangle((22,28,26,31),fill=c);d.line((16,42,20,42),fill=c,width=2)
    if science==0:
        d.polygon([(16,19),(12,14),(17,12),(14,7),(20,9),(22,5),(26,8),(31,5),(31,10),(36,9),(32,14),(35,17),(31,20),(30,13),(18,13)],fill='#e9eff2',outline='#8da3b3')
        d.rectangle((21,22,27,23),fill='#f4f5ee');d.line((19,15,22,16),fill='#718394');d.line((26,16,29,15),fill='#718394')
        d.polygon([(36,31),(42,35),(36,39)],fill=c,outline=DARK)
    elif science==1:
        d.polygon([(16,27),(13,22),(15,12),(17,7),(29,7),(33,13),(34,26),(29,27),(31,20),(29,13),(19,13),(18,21),(20,26)],fill='#694633',outline=DARK)
        d.line((17,9,28,9),fill='#b89063',width=2);d.ellipse((35,31,41,37),fill=c);d.line((38,31,39,28),fill='#74bd91',width=1)
    else:
        d.ellipse((25,3,33,11),fill='#384347',outline=DARK);d.polygon([(17,15),(16,10),(19,7),(29,7),(32,12),(30,16),(28,12),(19,12)],fill='#384347')
        d.rectangle((36,29,40,38),fill=c,outline=DARK);d.rectangle((37,27,39,30),fill='#e9f4f1');d.point((38,33),fill='white')
    return im
def effects():
    entries={}
    for tag in ['cast0','cast1','cast2','shield','fizzle']+['packet'+str(i) for i in range(4)]+['field'+str(i) for i in [1,5,12,13,18,20,45,57,58,65,66]]:
        seq=[]
        for f in range(4):
            im=Image.new('RGBA',(96,96));d=ImageDraw.Draw(im);c=COLORS[2 if tag in ('field57','field58','field65','field66','packet3','packet1') else 1 if tag in ('packet2','field45','cast1') else 0]
            alpha=180+f*15;rgb=tuple(int(c[i:i+2],16) for i in (1,3,5));rgba=rgb+(alpha,)
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
            seq.append((im,.06 if not tag.startswith('field') else .4))
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
    mastery.generate(SimpleNamespace(DATA=DATA,atlas=atlas,body=body,overlay=overlay))
    print('Original body, three personas, notebooks, meter and mastery art generated')
if __name__=='__main__':generate()
