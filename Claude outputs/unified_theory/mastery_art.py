"""Reproducible runtime pixel art for the Walking Unified Experiment.

All drawing uses native pixels. Buff canvases share the body's centre. Cosmic
costumes retain human silhouettes; one background loop contains each cosmic aura.
"""
import json
import math
import importlib.util
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

HERE = Path(__file__).resolve().parent
_spec = importlib.util.spec_from_file_location('science_cosmic_art', HERE/'cosmic_art.py')
cosmic = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(cosmic)
_ispec = importlib.util.spec_from_file_location('science_iconic_art', HERE/'iconic_art.py')
iconic = importlib.util.module_from_spec(_ispec)
_ispec.loader.exec_module(iconic)
# One shared construction library, so verification patches apply everywhere.
iconic.cosmic = cosmic
ROOT = HERE.parents[1]
COLORS = ['#63bfff', '#ffc96b', '#63e1bd']
WHITE = '#eef6ed'
DARK = '#101e30'
GOLD = '#f6d278'
RANKS = ['Student', 'Lab Assistant', 'Researcher', 'Scientist', 'Professor', 'Fellow', 'Laureate', 'Unified Mind']
FIELDS = [1, 5, 12, 13, 18, 20, 45, 57, 58, 65, 66]
MARKS = [26, 42, 50, 69]
PACKETS = [0, 5, 12, 13, 16, 19, 20, 29, 34, 39, 40, 45, 54, 57, 58, 59, 63, 65, 66, 71, 72, 73, 74]
FONT = ('C:/Windows/Fonts/consola.ttf' if Path('C:/Windows/Fonts/consola.ttf').exists() else '/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf')


def rgba(color, alpha=255):
    return tuple(int(color[i:i + 2], 16) for i in (1, 3, 5)) + (alpha,)


def point(cx, cy, radius, angle, flatten=1):
    return round(cx + radius * math.cos(angle)), round(cy + radius * math.sin(angle) * flatten)


def line(d, points, color, width=1):
    d.line([(round(x), round(y)) for x, y in points], fill=color, width=width)


def ring(d, cx, cy, radius, color, phase=0, sides=48, flatten=1, rough=False):
    pts = [point(cx, cy, radius + (i % 3 - 1 if rough else 0), phase + i * math.tau / sides, flatten) for i in range(sides + 1)]
    line(d, pts, color)


def diamond(d, x, y, r, color, fill=None):
    pts = [(x, y-r), (x+r, y), (x, y+r), (x-r, y)]
    d.polygon(pts, fill=fill or DARK)
    line(d, pts + [pts[0]], color)


def star(d, x, y, color, r=2):
    line(d, [(x-r, y), (x+r, y)], color)
    line(d, [(x, y-r), (x, y+r)], color)


def prism(d, x, y, r, color, f=0):
    pts = [(x, y-r), (x+r, y+r//2), (x-r, y+r//2), (x, y-r)]
    d.polygon(pts, fill=rgba(color, 35))
    line(d, pts, color)
    line(d, [(x, y-r), (x, y+r//2)], WHITE)
    if f % 2: star(d, x+r, y+r//2, WHITE, 1)


def knot(d, x, y, r, f=0):
    # Alternating bridges give the three loops an over/under construction.
    for k, color in enumerate(COLORS):
        a=k*math.tau/3
        cx,cy=point(x,y,r*.22,a)
        pts=[point(cx,cy,r*.78,a+j*math.tau/3+math.pi/6) for j in range(4)]
        line(d, pts, rgba(color, 210), 2)
        bx, by=point(x, y, r*.38, a+f*.08)
        d.rectangle((bx-2, by-1, bx+2, by+1), fill=DARK)
        line(d, [(bx-2, by), (bx+2, by)], color)


DIGITS = {
    '0':['111','101','101','101','111'], '1':['010','110','010','010','111'],
    '2':['111','001','111','100','111'], '3':['111','001','111','001','111'],
    '4':['101','101','111','001','001'], '5':['111','100','111','001','111'],
    '6':['111','100','111','101','111'], '7':['111','001','010','010','010'],
    '8':['111','101','111','101','111'], '9':['111','101','111','001','111'],
}


def badge(rank,f,position=None):
    # Original compact shapes, never a resampled large badge. The position glyph
    # stays 3x5 pixels so #10 remains legible inside the smaller cosmic crest.
    im=Image.new('RGBA',(64,96));d=ImageDraw.Draw(im);x,y=49,28
    c=[COLORS[0],COLORS[2],COLORS[0],COLORS[0],COLORS[1],COLORS[2],GOLD,GOLD][rank]
    if rank==0:
        d.polygon([(x-6,y-3),(x-1,y-4),(x,y-2),(x+1,y-4),(x+6,y-3),(x+6,y+4),(x+1,y+3),(x,y+5),(x-1,y+3),(x-6,y+4)],fill='#d9d6bd',outline=DARK)
        line(d,[(x,y-2),(x,y+3)],'#597289')
        line(d,[(x-4,y),(x-2,y+f%2)],c)
    elif rank==1:
        line(d,[(x-5,y-5),(x-5,y+5),(x+5,y+5),(x+5,y-5)],'#647f91')
        d.polygon([(x-1,y-5),(x+1,y-5),(x+1,y-1),(x+4,y+3),(x+3,y+4),(x-3,y+4),(x-4,y+3),(x-1,y-1)],fill='#23534f',outline=c)
        d.point((x,y+2-f%3),fill=WHITE)
    elif rank==2:
        pts=[(x-5,y+4),(x,y-5),(x+5,y+4)]
        line(d,pts+[pts[0]],'#527e9e')
        for k,p in enumerate(pts):diamond(d,*p,1,COLORS[k])
        d.point(point(x,y,3,f*math.tau/8),fill=WHITE)
    elif rank==3:
        gap=2+f%2
        line(d,[(x-gap,y-5),(x-gap-3,y-2),(x-gap-3,y+2),(x-gap,y+5)],c)
        line(d,[(x+gap,y-5),(x+gap+3,y-2),(x+gap+3,y+2),(x+gap,y+5)],c)
        d.ellipse((x-1,y-1,x+1,y+1),fill='#0b152b',outline=WHITE)
    elif rank==4:
        d.polygon([(x,y-6),(x+4,y+4),(x,y+2),(x-4,y+4)],fill='#263647',outline=GOLD)
        line(d,[(x,y-5),(x-3,y+4)],WHITE)
        d.point(point(x,y+1,5,f*math.tau/8,0.5),fill=c)
    elif rank==5:
        for k,color in enumerate(COLORS):
            pts=[point(x,y,5,k*math.tau/3+j*math.tau/3+f*.03) for j in range(4)]
            line(d,pts,color)
        d.point(point(x,y,3,f*math.tau/8),fill=WHITE)
    elif rank==6:
        for k in range(6):
            a=k*math.tau/6
            d.point(point(x,y,5,a),fill=GOLD)
            d.point(point(x,y,6,a+f*.04),fill='#a48248')
        diamond(d,x,y,2,COLORS[2]);d.point((x,y),fill=WHITE)
    else:
        for k,color in enumerate(COLORS):
            a=k*math.tau/3
            cx,cy=point(x,y,1,a)
            pts=[point(cx,cy,6,a+j*math.tau/3+math.pi/6) for j in range(4)]
            line(d,pts,color)
        # Animation is confined to the crest, rather than rays or an oversized crown.
        d.point(point(x,y,6,f*math.tau/8),fill=WHITE)
        label=str(position or 10);left=x-(len(label)*4-1)//2
        d.rectangle((left-1,y-3,left+len(label)*4-1,y+3),fill=DARK)
        for n,digit in enumerate(label):
            for row,bits in enumerate(DIGITS[digit]):
                for col,bit in enumerate(bits):
                    if bit=='1':d.point((left+n*4+col,y-2+row),fill=WHITE)
        if position and position<=3:
            d.point((x-5,y-6),fill=COLORS[0]);d.point((x+5,y-6),fill=COLORS[2])
            if position==3:d.point((x,y-7),fill=COLORS[1])
            elif position==2:line(d,[(x-1,y-7),(x+1,y-7)],WHITE)
            else:line(d,[(x-2,y-7),(x,y-9),(x+2,y-7)],GOLD)
    return im


def outfit(art, science, rank, f):
    return art.overlay(science, rank, f)


def instrument(d, science, x, y, f, advanced=False):
    c=COLORS[science];phase=f*math.tau/8
    if science==0:
        ring(d,x,y,6,'#95acbf');line(d,[(x,y),point(x,y,4,phase)],WHITE)
        line(d,[(x,y),point(x,y,3,phase*.25)],GOLD)
        d.rectangle((x-2,y-9,x+2,y-7),fill=GOLD)
        if advanced:
            prism(d,x+10,y+4,5,c,f)
            bend=round(math.sin(phase)*2)
            line(d,[(x-6,y+7),(x,y+9+bend),(x+6,y+8),(x+12,y+5)],rgba(c,150))
            line(d,[(x-7,y-10),(x+6,y-12),(x+7,y-5)],rgba(c,100))
            d.ellipse((x-5,y+8,x+7,y+11),fill=rgba(c,35))
            line(d,[(x,y+9),point(x,y+9,4,phase*.4)],rgba(c,95))
    elif science==1:
        line(d,[(x,y-7),(x-6,y+7),(x,y+3),(x+6,y+7),(x,y-7)],GOLD)
        ax,ay=point(x+10,y,4,phase,.5) if advanced else (x+10,y)
        if advanced:ring(d,x+10,y,4,rgba(GOLD,100),0,16,.5)
        d.ellipse((ax-3,ay-4,ax+3,ay+2),fill='#db9b53',outline=DARK)
        line(d,[(ax,ay-4),(ax+1,ay-7)],COLORS[2])
        if advanced:
            nx,ny=point(x-10,y,4,phase*.5)
            line(d,[(x-15,y+5),(nx,ny),(x-14,y-5)],c)
            diamond(d,nx,ny,2,WHITE)
    else:
        d.rectangle((x-5,y-4,x+5,y+7),fill='#233b45',outline='#9ababa')
        d.rectangle((x-2,y-8,x+2,y-4),fill='#bedbd2')
        d.rectangle((x-3,y+1,x+3,y+5),fill=rgba(c,180))
        d.point((x+(f%3-1),y+3-f%4),fill=WHITE)
        if advanced:
            growth=3+(f if f<4 else 7-f)
            prism(d,x+12,y,growth,c,f)
            ring(d,x+12,y,7,rgba(c,140),f*.1,8,.6)
            if f>=4:
                for k in range(3):d.point((x+9+k*3,y-4-(f-4)*2),fill=rgba(c,150))


def equipment(science, rank, f, front=False, podium=4):
    # One entire background construction, never a separate loop per component.
    # Front tags remain empty: the face, hands, badge and feet stay readable.
    return iconic.equipment(science,rank,f,front,podium)


def field(kind,tier,f,size=None):
    size=size or iconic.FIELD
    im=iconic.field(kind,tier,f,size)
    # Match the existing domain-art scale (about 950 world units per pixel).
    # Keep the boundary fixed
    # while the scientific construction animates inside it.
    if kind in (5,12,13,18,20,45,58,65,66):
        d=ImageDraw.Draw(im);science=2 if kind in (58,65,66) else 1 if kind==45 else 0
        center=size//2;radius=19 if kind==13 else 32
        ring(d,center,center,radius,rgba(COLORS[science],155))
        if kind==20:ring(d,center,center,25,rgba(COLORS[science],110))
    return im


def skill_effect(skill,tier,f,skills):
    return iconic.cast(skill,tier,f,skills)


def packet(skill,tier,f,heading,skills):
    return iconic.packet(skill,tier,f,heading,skills)


def impact(skill,tier,f,skills):
    return iconic.impact(skill,tier,f,skills)


def modifier(mask,tier,f):
    return cosmic.modifier(mask,tier,f)


def completion(tier,podium,f,science=0):
    return iconic.completion(tier,podium,f,science)


def bounded(art,mod,name,entries,w,h,cols=16):
    """Pack into one 2048 px sheet, splitting into a/b halves when needed."""
    try:
        art.atlas(mod,name,entries,w,h,cols,trim=True,palette=True)
        return [name]
    except AssertionError:
        for suffix in ['#anim.fanim','#sheet.png']:
            stale=mod/(name+suffix)
            if stale.exists():stale.unlink()
        items=list(entries.items());half=len(items)//2
        assert half,(name,'single animation exceeds 2048 px')
        return bounded(art,mod,name+'a',dict(items[:half]),w,h,cols)+bounded(art,mod,name+'b',dict(items[half:]),w,h,cols)


def generate(art):
    mod=ROOT/'mods/tfm2_custom/vfx';skills=art.DATA['skills']
    # Separate bounded atlases rather than relying on an engine-sized monolithic sheet.
    badges={f'rank{r}':[(badge(r,f),.12) for f in range(8)] for r in range(7)}
    badges.update({f'top{p}':[(badge(7,f,p),.12) for f in range(8)] for p in range(1,11)})
    art.atlas(mod,'science_badges',badges,64,96,16)
    outfits={f'outfit{s}_r{r}':[(outfit(art,s,r,f),.12) for f in range(8)] for s in range(3) for r in range(8)}
    art.atlas(mod,'science_outfits',outfits,48,64,16)
    # Facing left: the game never mirrors buff overlays, so every body layer ships a mirrored `_l` copy on its own
    # sheet (same anchor). The gear aura behind stays as drawn: its formulas would read backwards.
    mirror=lambda entries:{tag+'_l':[(im.transpose(Image.Transpose.FLIP_LEFT_RIGHT),sec) for im,sec in seq] for tag,seq in entries.items()}
    art.atlas(mod,'science_outfits_l',mirror(outfits),48,64,16,trim=True)
    transforms={f'transform{s}_{t}_r{r}':[(art.transform(s,t,r,f),.05) for f in range(8)] for s in range(3) for t in range(3) if s!=t for r in range(8)}
    art.atlas(mod,'science_transforms',transforms,48,64,16)
    art.atlas(mod,'science_transforms_l',mirror(transforms),48,64,16,trim=True)
    for s in range(3):
        gear={}
        for r in range(8):
            for podium in ([1,2,3,4] if r==7 else [4]):
                suffix=f'{s}_r{r}'+(f'_p{podium}' if r==7 else '')
                for front in [False,True]:
                    tag=('gearfront' if front else 'gearback')+suffix
                    seconds=.2 if r==7 and podium==1 else .12
                    gear[tag]=[(equipment(s,r,f,front,podium),seconds) for f in range(8)]
        bounded(art,mod,f'science_equipment{s}',gear,*iconic.AURA)
        for tier in range(4):
            casts={f'skill_{skills[i]["id"]}_t{tier}':[(skill_effect(i,tier,f,skills),.075) for f in range(8)] for i in range(s*25,s*25+25)}
            bounded(art,mod,f'science_skills{s}_t{tier}',casts,iconic.CAST,iconic.CAST)
    for tier in range(4):
        fields={f'field{k}_t{tier}':[(field(k,tier,f),.1) for f in range(8)] for k in FIELDS}
        bounded(art,mod,f'science_fields_t{tier}',fields,iconic.FIELD,iconic.FIELD)
        # Eight unique sprites per loop. Pair aliases reuse those pixels; the
        # moving renderer plays one four-tick pair at a time. Short lifetimes
        # improve visible travel without accumulating copies behind the object.
        name=f'science_packets_t{tier}'
        packets={f'packet_{skills[s]["id"]}_t{tier}_a{h}':[(packet(s,tier,f,h,skills),2/60) for f in range(8)] for s in PACKETS for h in range(8)}
        for sheet in bounded(art,mod,name,packets,iconic.PACKET,iconic.PACKET,32):
            path=mod/(sheet+'#anim.fanim');meta=json.loads(path.read_text(encoding='utf-8'))
            for tag,anim in list(meta['anims'].items()):
                for pair in range(4):meta['anims'][tag+f'_pair{pair}']={'frames':anim['frames'][pair*2:pair*2+2]}
            path.write_text(json.dumps(meta,indent=2)+'\n')
        impacts={f'impact_{skills[s]["id"]}_t{tier}':[(impact(s,tier,f,skills),.05) for f in range(8)] for s in range(75)}
        bounded(art,mod,f'science_impacts_t{tier}',impacts,iconic.IMPACT,iconic.IMPACT,32)
    for science in range(3):
        poses={f'pose{science}_r{r}_p{p}':[(art.pose(science,r,p,f),1/40) for f in range(8)] for r in range(8) for p in range(3)}
        art.atlas(mod,f'science_poses{science}',poses,48,64,16)
        art.atlas(mod,f'science_poses{science}_l',mirror(poses),48,64,16,trim=True)
    art.atlas(mod,'science_crystal_body',{'crystal_body':[(cosmic.crystal_body(f),.1) for f in range(8)]},48,64,8)
    misc={f'modifiers{t}_{mask}':[(modifier(mask,t,f),.12) for f in range(8)] for t in range(4) for mask in range(1,8)}
    art.atlas(mod,'science_modifiers',misc,64,80,16)
    complete={f'complete_t{t}_p{p}_s{s}':[(completion(t,p,f,s),.1) for f in range(8)] for s in range(3) for t in range(4) for p in ([1,2,3,4] if t==3 else [4])}
    # Unified resonance: all three scientists, one combined strike.
    complete.update({f'complete_unified_t{t}_p{p}':[(iconic.unified(t,p,f),.1) for f in range(8)] for t in range(4) for p in ([1,2,3,4] if t==3 else [4])})
    bounded(art,mod,'science_completion',complete,iconic.COMPLETE,iconic.COMPLETE)
    # Visible lab accidents: 30 ticks, in front of the body.
    accidents={f'accident{s}':[(iconic.accident(s,f),.5/8) for f in range(8)] for s in range(3)}
    bounded(art,mod,'science_accidents',accidents,iconic.ACCIDENT,iconic.ACCIDENT)
    # Prepared vectors, forecasts, samples and isotope labels.
    marks={f'mark{k}_t{t}':[(iconic.mark(k,t,f),.1) for f in range(8)] for k in MARKS for t in range(4)}
    bounded(art,mod,'science_marks',marks,iconic.MARK,iconic.MARK)
    echoes={}
    for s in range(3):
        seq=[]
        for f in range(8):
            im=cosmic.echo(s,f)
            seq.append((im,1/60))
        echoes[f'echo{s}']=seq
    art.atlas(mod,'science_echo',echoes,96,112,16,trim=True,palette=True)
    previews(art,skills)
    art.compact_previews()
    print('Iconic Unified Theory: 33 ranked auras, 300 casts and 300 contacts, 44 fields, 736 directional packet loops, 28 completions incl. Unified resonance, 3 lab accidents, 16 setup marks and 72 cast poses')


def previews(art,skills):
    folder=ROOT/'docs';editor=ROOT/'editor';bg='#0b1422'
    AW,AH=iconic.AURA;C=iconic.CAST;K=iconic.COMPLETE
    runtime={};decoded={}
    for file in sorted((ROOT/'mods/tfm2_custom/vfx').glob('science_*#anim.fanim')):
        name=file.name.split('#')[0]
        for tag,anim in json.loads(file.read_text(encoding='utf-8'))['anims'].items():runtime[tag]={'sheet':name,'frames':anim['frames']}
    def original(tag,f,w,h):
        asset=runtime[tag];name=asset['sheet']
        if name not in decoded:decoded[name]=Image.open(ROOT/'mods/tfm2_custom/vfx'/f'{name}#sheet.png').convert('RGBA')
        rect=asset['frames'][f]['data'];x,y,fw,fh=[rect[k] for k in ['x','y','w','h']]
        im=Image.new('RGBA',(w,h));im.alpha_composite(decoded[name].crop((x,y,x+fw,y+fh)),((w-fw)//2,(h-fh)//2))
        return im
    def title(d,x,y,text,size=13,color='#dcecf4'):
        d.text((x,y),text,font=ImageFont.truetype(FONT,size),fill=color)
    def composed(s,r,f=2,podium=4):
        suffix=f'{s}_r{r}'+(f'_p{podium}' if r==7 else '')
        im=original('gearback'+suffix,f,AW,AH)
        base=art.body('idle',f);base.alpha_composite(original(f'outfit{s}_r{r}',f,48,64))
        im.alpha_composite(base,iconic.BODY);im.alpha_composite(original('gearfront'+suffix,f,AW,AH))
        return im
    def big(im,k=2):return im.resize((im.width*k,im.height*k),Image.Resampling.NEAREST)
    NAMES=['EINSTEIN / SPACETIME, LIGHT AND E=mc²','NEWTON / THE APPLE, THE PRISM AND THE ORBITS','MARIE CURIE / RADIUM, ATOMS AND TWO NOBELS']
    PEAKS=['SPACETIME FABRIC','PRINCIPIA UNIVERSE','STELLAR FORGE']
    def rank_preview(f):
        cw,ch=AW*2+16,AH*2+64
        im=Image.new('RGBA',(16+11*cw,90+3*(ch+40)),bg);d=ImageDraw.Draw(im)
        title(d,24,18,'THE UNIFIED THEORY / ICONIC COSMIC MASTERY',26)
        title(d,24,56,'Every rank adds one recognisable layer. Unified Mind #1 brings the other two scientists to the shoulders. 2x, actual runtime frames.',13)
        variants=[(r,4) for r in range(7)]+[(7,4),(7,3),(7,2),(7,1)]
        labels=RANKS[:7]+['Unified Mind #4-10','#3','#2','#1']
        for s in range(3):
            y=90+s*(ch+40);title(d,24,y,NAMES[s],19,COLORS[s])
            for j,(r,p) in enumerate(variants):
                x=16+j*cw;d.rounded_rectangle((x,y+30,x+cw-8,y+30+ch),8,fill='#142338')
                im.alpha_composite(big(composed(s,r,f,p)),(x,y+34))
                title(d,x+8,y+ch+8,labels[j] if not (r==7 and p==1) else '#1 '+PEAKS[s],12)
        return im.convert('RGB')
    rank_preview(2).save(folder/'unified-theory-mastery.png')
    ranked=[rank_preview(f).quantize(colors=256) for f in range(8)]
    ranked[0].save(folder/'unified-theory-ranked-cosmic.gif',save_all=True,append_images=ranked[1:],duration=120,loop=0,disposal=2)
    # Browser-friendly composite frames for the editor's rank inspector.
    tiles={};sheet=Image.new('RGBA',(8*AW,(3*11+1)*AH));n=0
    for s in range(3):
        for r in range(8):
            for podium in ([1,2,3,4] if r==7 else [4]):
                tag=f'{s}_{r}_{podium}';frames=[]
                for f in range(8):
                    x=f*AW;y=n*AH;sheet.alpha_composite(composed(s,r,f,podium),(x,y));frames.append({'x':x,'y':y,'w':AW,'h':AH})
                tiles[tag]=frames;n+=1
    base=Image.new('RGBA',(AW,AH));base.alpha_composite(art.body('idle',0),iconic.BODY);sheet.alpha_composite(base,(0,n*AH))
    sheet.save(editor/'science-mastery-preview.png')
    assets={tag:asset for tag,asset in runtime.items() if tag.startswith(('pose','outfit','transform','gearback','skill_','impact_','orb_','bar_','rank','top','complete','echo','field','packet_','accident','mark')) and '_pair' not in tag}
    (editor/'science-art-preview.json').write_text(json.dumps({'outfits':tiles,'base':{'x':0,'y':n*AH,'w':AW,'h':AH},'body':list(iconic.BODY),'assets':assets,'frames':8,'frameSeconds':.12},indent=2)+'\n')
    portrait=Image.new('RGBA',(960,360),bg);d=ImageDraw.Draw(portrait)
    for s,name in enumerate(['EINSTEIN','NEWTON','MARIE CURIE']):
        x=s*320;d.rounded_rectangle((x+12,12,x+308,348),12,fill='#142338',outline=COLORS[s],width=2)
        body=art.body('idle',2);body.alpha_composite(outfit(art,s,5,2))
        portrait.alpha_composite(body.resize((192,256),Image.Resampling.NEAREST),(x+64,42))
        title(d,x+25,318,name,18,COLORS[s])
    portrait.save(editor/'science-portraits.png')
    im=Image.new('RGBA',(16+4*(AW*2+16),80+3*(AH*2+60)),bg);d=ImageDraw.Draw(im)
    title(d,24,18,'TOP 10 / THREE MINDS, ONE BODY',24)
    for s,name in enumerate(['EINSTEIN','NEWTON','MARIE CURIE']):
        y=60+s*(AH*2+60);title(d,18,y,name,16,COLORS[s])
        for j,p in enumerate([4,3,2,1]):
            x=16+j*(AW*2+16);d.rounded_rectangle((x,y+24,x+AW*2+8,y+AH*2+52),8,fill='#142338')
            im.alpha_composite(big(composed(s,7,3,p)),(x+4,y+28))
            title(d,x+10,y+AH*2+30,'#4-10' if p==4 else f'#{p}'+(' / '+PEAKS[s] if p==1 else ''),12)
    im.convert('RGB').save(folder/'unified-theory-top10.png')
    show=[0,4,5,13,24,34,39,40,49,27,73,74,63,57,65]
    im=Image.new('RGBA',(300+4*C,84+len(show)*C),bg);d=ImageDraw.Draw(im)
    title(d,24,16,'SKILL ACTIVATIONS / FOUR MASTERY TIERS, RUNTIME FRAME 4',22)
    for t,name in enumerate(['SEED','CONSTRUCTION','FRONTIER','PEAK']):title(d,300+t*C+40,53,name,12,COLORS[t%3])
    for row,s in enumerate(show):
        y=80+row*C;title(d,12,y+C//2,skills[s]['name'][:30],13,COLORS[skills[s]['science']])
        for t in range(4):im.alpha_composite(original(f'skill_{skills[s]["id"]}_t{t}',3,C,C),(300+t*C,y))
    im.convert('RGB').save(folder/'unified-theory-effects.png')
    for s,name in enumerate(['einstein','newton','curie']):
        tw=C//2+8;grid=Image.new('RGBA',(12+5*(2*tw+16),82+5*(2*tw+70)),bg);d=ImageDraw.Draw(grid)
        title(d,18,16,name.upper()+' / ALL 25 SKILLS',22,COLORS[s])
        title(d,18,48,'Four mastery tiers per card: seed, construction, frontier, peak. Runtime frame 4, half size.',12)
        for local in range(25):
            skill=skills[s*25+local];x=12+(local%5)*(2*tw+16);y=82+(local//5)*(2*tw+70)
            d.rounded_rectangle((x,y,x+2*tw+8,y+2*tw+60),8,fill='#142338')
            title(d,x+8,y+8,skill['id']+' '+skill['name'][:22],12,COLORS[s])
            for t in range(4):
                tile=original(f'skill_{skill["id"]}_t{t}',3,C,C).resize((tw,tw),Image.Resampling.BOX)
                grid.alpha_composite(tile,(x+4+(t%2)*tw,y+28+(t//2)*tw))
        grid.convert('RGB').save(folder/f'unified-theory-skills-{name}.png')
    show=[(0,'EINSTEIN / PHOTON PULSE (E=mc²)'),(5,'EINSTEIN / TIME DILATION'),
          (34,'NEWTON / MOMENTUM STRIKE (THE APPLE)'),(49,'NEWTON / PRINCIPIA'),
          (73,'CURIE / GAMMA BEAM'),(74,'CURIE / DECAY CHAIN')]
    strip=Image.new('RGBA',(32+8*C,60+len(show)*(C+30)),bg);d=ImageDraw.Draw(strip)
    title(d,16,14,'EIGHT ORIGINAL SPRITES / PEAK ACTIVATIONS',20)
    for row,(skill,label) in enumerate(show):
        y=50+row*(C+30);title(d,16,y,label,13,COLORS[skills[skill]['science']])
        for f in range(8):strip.alpha_composite(original(f'skill_{skills[skill]["id"]}_t3',f,C,C),(16+f*C,y+20))
    strip.convert('RGB').save(folder/'unified-theory-cosmic-frames.png')
    movie=[]
    for f in range(8):
        scene=Image.new('RGBA',(3*(AW*2+40),AH*2+120),bg);d=ImageDraw.Draw(scene)
        title(d,20,16,'THE UNIFIED THEORY / #1',24)
        for s in range(3):
            x=s*(AW*2+40)+20;title(d,x,60,PEAKS[s],16,COLORS[s])
            hero=composed(s,7,f,1);hero.alpha_composite(original('top1',f,64,96),(AW//2-14,26))
            scene.alpha_composite(big(hero),(x,90))
        movie.append(scene.convert('RGB').quantize(colors=256))
    movie[0].save(folder/'unified-theory-auras.gif',save_all=True,append_images=movie[1:],duration=200,loop=0,disposal=2)
    movie[0].save(folder/'unified-theory-cosmic.gif',save_all=True,append_images=movie[1:],duration=200,loop=0,disposal=2)
    strip=Image.new('RGBA',(32+8*AW,60+3*(AH+40)),bg);d=ImageDraw.Draw(strip)
    title(d,16,14,'#1 / EIGHT ORIGINAL AURA SPRITES',20)
    for s in range(3):
        y=50+s*(AH+40);title(d,16,y,PEAKS[s],13,COLORS[s])
        for f in range(8):strip.alpha_composite(composed(s,7,f,1),(16+f*AW,y+20))
    strip.convert('RGB').save(folder/'unified-theory-aura-frames.png')
    peaks=[]
    for f in range(8):
        scene=Image.new('RGBA',(3*(K*2+20)+20,K*2+90),bg);d=ImageDraw.Draw(scene)
        title(d,18,16,'#1 / SUCCESSFUL COMBINATION',24)
        for s in range(3):
            x=20+s*(K*2+20);title(d,x,56,['EINSTEIN','NEWTON','MARIE CURIE'][s],16,COLORS[s])
            sprite=original(f'complete_t3_p1_s{s}',f,K,K)
            sprite.alpha_composite(composed(s,7,f,1),((K-AW)//2,(K-AH)//2))
            scene.alpha_composite(big(sprite),(x,80))
        peaks.append(scene.convert('RGB'))
    peaks[3].save(folder/'unified-theory-peak-completions.png')
    peaks[0].save(folder/'unified-theory-peak-completions.gif',save_all=True,append_images=peaks[1:],duration=100,loop=0,disposal=2)
    # Round 125: the three-science resonance and the three lab accidents.
    frames=[]
    for f in range(8):
        scene=Image.new('RGBA',(K*2+3*(iconic.ACCIDENT*2+20)+60,K*2+90),bg);d=ImageDraw.Draw(scene)
        title(d,18,16,'UNIFIED RESONANCE / LAB ACCIDENTS',24)
        sprite=original('complete_unified_t3_p1',f,K,K)
        body=art.body('idle',f);body.alpha_composite(original('outfit1_r7',f,48,64))
        sprite.alpha_composite(body,((K-48)//2,(K-64)//2));scene.alpha_composite(big(sprite),(20,70))
        for s in range(3):
            A=iconic.ACCIDENT;x=K*2+40+s*(A*2+20)
            title(d,x,56,['CHALK BLAST','THE APPLE','BOILING VIAL'][s],14,COLORS[s])
            card=Image.new('RGBA',(A,A));body=art.body('idle',f);body.alpha_composite(original(f'outfit{s}_r0',f,48,64))
            card.alpha_composite(body,((A-48)//2,(A-64)//2));card.alpha_composite(original(f'accident{s}',f,A,A))
            scene.alpha_composite(big(card),(x,90))
        frames.append(scene.convert('RGB').quantize(colors=256))
    frames[0].save(folder/'unified-theory-resonance-accidents.gif',save_all=True,append_images=frames[1:],duration=100,loop=0,disposal=2)
    frames[4].convert('RGB').save(folder/'unified-theory-resonance-accidents.png')
