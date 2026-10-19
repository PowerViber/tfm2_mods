"""Restore the retained cosmic Domain. Nearest-neighbour scale, no body changes."""
from pathlib import Path
from PIL import Image,ImageDraw
import importlib.util,json,math
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'mods/tfm2_jjk/vfx';DOC=ROOT/'docs/gojo-results'
spec=importlib.util.spec_from_file_location('pack',ROOT/'tools/generate-gojo-art.py');pack=importlib.util.module_from_spec(spec);spec.loader.exec_module(pack)
def generate():
    old=Image.open(OUT/'gojo#sheet.png').convert('RGBA');meta=json.loads((OUT/'gojo#anim.fanim').read_text(encoding='utf-8'));frames=[]
    timelines=[]
    for mode,width,source in [('full',140,'slow'),('quick',210,'fast')]:
        for part,tag in [('field','void_'+source),('ring','void_ring_'+source),('cast','void_cast_'+source)]:
            durations=[round(f['duration']*60000) for f in meta['anims'][tag]['frames']]
            timelines.append(f'        ("{mode}","{part}") => &{durations!r},')
            for f in meta['anims'][tag]['frames']:
                r=f['data'];im=old.crop((r['x'],r['y'],r['x']+r['w'],r['y']+r['h']))
                if part!='cast':im=im.resize((width,width),Image.Resampling.NEAREST)
                frames.append((f'void_{mode}_{part}',im,f['duration']))
                frames.append((f'void_{mode}_{part}{len([x for x in frames if x[0]==f"void_{mode}_{part}"])-1}',im,f['duration']))
    groups={}
    for kind in [14,15,16,17]:
        group=[]
        for n in range(8):
            size=48 if kind in [14,17] else 212;im=Image.new('RGBA',(size,size));d=ImageDraw.Draw(im);c=size//2
            if kind==14:
                d.ellipse((c-8,c-8,c+8,c+8),fill=(5,35,110,230),outline=(150,240,255,255),width=2)
                d.ellipse((c-3,c-3,c+3,c+3),fill=(225,255,255,255))
                for q in range(3):
                    a=q*math.tau/3+n*.15;x=c+round(math.cos(a)*17);y=c+round(math.sin(a)*17);d.arc((5,5,42,42),q*120+n*8,q*120+n*8+50,fill=(55,145,255,210),width=2)
            elif kind==17:
                d.ellipse((10,18,38,30),outline=(225,165,70,230),width=2);d.line((12,13,36,35),fill=(255,100,80,255),width=3)
            else:
                radius=96 if kind==15 else 12+n*12
                d.ellipse((c-radius,c-radius,c+radius,c+radius),outline=(165,90,255,220),width=2 if kind==15 else 5)
                if kind==15:
                    d.text((c-22,c-6),'FUSION',fill=(245,230,255,255))
                    for q in range(12):
                        a=q*math.tau/12+n*.03;x=c+round(math.cos(a)*radius);y=c+round(math.sin(a)*radius);d.line((c+(x-c)*.82,c+(y-c)*.82,x,y),fill=(235,150,255,255),width=2)
                else:
                    for q in range(8):
                        a=q*math.tau/8+n*.1;x=c+round(math.cos(a)*radius);y=c+round(math.sin(a)*radius);d.line((c,c,x,y),fill=(140,65,230,180),width=3)
                    d.ellipse((c-8,c-8,c+8,c+8),fill=(250,230,255,255))
            group.append(im);frames.append((f'fx{kind}',im,.075))
        groups[kind]=group
    # A readable six-second timer uses the same orb artwork and compact digits.
    for seconds in range(1,7):
        for im in groups[14]:
            im=im.copy();ImageDraw.Draw(im).text((20,36),str(seconds),fill=(190,240,255,255));frames.append((f'orb{seconds}',im,.075))
    source=Image.new('RGBA',(2048,8192));anims={};x=y=row=0
    for tag,im,duration in frames:
        if x+im.width>2048:y+=row+1;x=row=0
        source.paste(im,(x,y));anims.setdefault(tag,{'frames':[]})['frames'].append({'duration':duration,'data':{'x':x,'y':y,'w':im.width,'h':im.height}});x+=im.width+1;row=max(row,im.height)
    atlas,packed=pack.save_packed(source.crop((0,0,2048,y+row)),{'anims':anims},OUT/'gojo_v2')
    rust='''// Generated from retained pre-PR40 animation metadata. Units: 1/1000 tick.
pub fn frame(mode:&str,part:&str,elapsed:usize,expand:usize)->usize {
    let durations:&[usize]=match (mode,part) {
'''+ '\n'.join(timelines)+'''
        _=>return 0,
    };
    let total=durations.iter().sum::<usize>();
    let mut time=if part=="field" {elapsed.saturating_sub(expand)*1000%total}
        else {elapsed.min(expand)*total/expand.max(1)};
    for (n,d) in durations.iter().enumerate() {if time<*d {return n;} time-=d;}
    durations.len()-1
}
'''
    (ROOT/'native/tfm2_custom_ai/src/gojo_v2_art.rs').write_text(rust,encoding='utf-8')
    paths=[ROOT/'mods/tfm2_jjk/champions/tfm2_custom_gojo#sheet.png',OUT/'gojo#sheet.png',OUT/'gojo_rework#sheet.png',OUT/'gojo_v2#sheet.png'];total=sum(Image.open(p).width*Image.open(p).height*4 for p in paths);assert total<=32*1024*1024,total
    contact=Image.new('RGBA',(840,440),(12,22,38,255));d=ImageDraw.Draw(contact)
    for i,tag in enumerate(['void_full_field','void_quick_field','fx14','fx15','fx16','fx17']):
        f=packed['anims'][tag]['frames'][len(packed['anims'][tag]['frames'])//2]['data'];im=atlas.crop((f['x'],f['y'],f['x']+f['w'],f['y']+f['h']));x=(i%3)*280+(280-im.width)//2;y=(i//3)*220+8;contact.alpha_composite(im,(x,y));d.text(((i%3)*280+24,(i//3)*220+205),tag,fill='white')
    contact.save(DOC/'v2-contact-sheet.png');animation=[]
    for n in range(8):
        im=Image.new('RGBA',(636,244),(12,22,38,255));d=ImageDraw.Draw(im)
        for i,k in enumerate([14,15,16]):im.alpha_composite(groups[k][n],(i*212+(212-groups[k][n].width)//2,0));d.text((i*212+15,222),['Maximum Blue orb','Fusion warning','Hollow Nuke'][i],fill='white')
        animation.append(im.convert('RGB'))
    animation[0].save(DOC/'v2-skills.gif',save_all=True,append_images=animation[1:],duration=75,loop=0)
    report={'decodedBytesIncludingV1Compatibility':total,'v2Atlas':atlas.size,'cosmicSource':'Retained pre-PR40 void/ring/cast tags, nearest-neighbour scaling','bodyPixelsChanged':False};(DOC/'v2-art.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8');print(json.dumps(report))
if __name__=='__main__':generate()
