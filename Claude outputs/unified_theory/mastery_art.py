"""Reproducible runtime pixel art for the Walking Unified Experiment.

All drawing uses native pixels. Buff canvases share the body's centre; floating
equipment stays clear of the face and never depends on the host's facing.
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
ROOT = HERE.parents[1]
COLORS = ['#63bfff', '#ffc96b', '#63e1bd']
WHITE = '#eef6ed'
DARK = '#101e30'
GOLD = '#f6d278'
RANKS = ['Student', 'Lab Assistant', 'Researcher', 'Scientist', 'Professor', 'Fellow', 'Laureate', 'Unified Mind']
FIELDS = [1, 5, 12, 13, 18, 20, 45, 57, 58, 65, 66]
PACKETS = [0, 16, 19, 29, 34, 39, 40, 54, 59, 63, 71, 72, 73, 74]
FONT = '/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf'


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


def badge(rank, f, position=None):
    im=Image.new('RGBA', (64, 96)); d=ImageDraw.Draw(im)
    x,y=(44 if rank==7 else 47),24; c=[COLORS[0],COLORS[2],COLORS[0],COLORS[2],COLORS[1],COLORS[0],GOLD,GOLD][rank]
    if rank==0:
        d.polygon([(36,18),(45,16),(47,19),(49,16),(58,18),(58,31),(49,29),(47,32),(45,29),(36,31)],fill='#d9d4b1',outline=DARK)
        line(d,[(47,19),(47,30)], '#778c9c')
        line(d,[(39,22),(42,22),(40,25),(44,25)], '#577286')
        line(d,[(51,24),(55,21-f%3)],c,2)
    elif rank==1:
        line(d,[(36,16),(36,34),(58,34),(58,16)],'#8da3b3',2)
        line(d,[(41,13),(44,13),(44,20),(40,27),(42,31),(53,31),(55,27),(51,20),(51,13),(54,13)],c)
        d.polygon([(41,27),(44,24),(51,24),(54,27),(52,30),(43,30)],fill=rgba(c,160))
        d.point((47,28-f%5),fill=WHITE)
        line(d,[(36,20),(41,20)],GOLD,2)
    elif rank==2:
        pts=[(36,29),(46,13),(59,29)]
        line(d,pts+[pts[0]],'#617f9d')
        for i,(a,b) in enumerate(zip(pts,pts[1:]+pts[:1])):
            diamond(d,*a,3,COLORS[i])
            if f%3==i:
                t=(f%4+1)/5; star(d,round(a[0]+(b[0]-a[0])*t),round(a[1]+(b[1]-a[1])*t),WHITE,1)
    elif rank==3:
        gap=2+f%3
        for side in [-1,1]:
            pts=[(x+side*gap,y-11),(x+side*(gap+7),y-6),(x+side*(gap+7),y+7),(x+side*gap,y+12)]
            line(d,pts,c,2)
        diamond(d,x,y,3,WHITE,rgba(c,130))
        line(d,[(x,y-17),(x,y-14)],GOLD)
    elif rank==4:
        d.polygon([(47,8),(57,33),(47,29),(37,33)],fill='#263647',outline=GOLD)
        line(d,[(47,10),(42,31),(39,34)],WHITE)
        line(d,[(47,10),(53,30),(57,33)],c,2)
        ring(d,47,25,12,rgba(c,160),-.7+f*.04,12,.5)
        diamond(d,47,10,2,WHITE)
    elif rank==5:
        for k,color in enumerate(COLORS):
            a=k*math.tau/3+f*.1
            cx,cy=point(x,y,7,a)
            pts=[point(cx,cy,7,a+j*math.tau/3) for j in range(4)]
            d.polygon(pts,fill=rgba(color,70));line(d,pts,color)
        diamond(d,x,y,2,WHITE)
    elif rank==6:
        for k in range(6):
            a=k*math.tau/6+f*.07
            pts=[point(x,y,r,a+o) for r,o in [(11,-.23),(15,-.2),(15,.2),(11,.23)]]
            d.polygon(pts,fill='#9d753b');line(d,pts+[pts[0]],GOLD)
        prism(d,x,y,6,COLORS[2],f)
        star(d,47,6,WHITE,2)
    else:
        knot(d,x,y,15,f)
        d.rectangle((x-6,19,x+6,28),fill=DARK)
        s=str(position or 10); font=ImageFont.truetype(FONT,8)
        w=d.textbbox((0,0),s,font=font)[2];d.text((x-w/2,19),s,font=font,fill=WHITE)
        if position and position<=3:
            for k in range(3):
                px,py=point(x,y,19,k*math.tau/3-math.pi/2)
                if position==3:diamond(d,px,py,2,COLORS[k])
                elif position==2:ring(d,px,py,3,COLORS[k],f*.15,12,.6)
                else:star(d,px,py,COLORS[k],3)
        if position==1:
            line(d,[(x-6,5),(x-3,9),(x,3),(x+3,9),(x+6,5),(x+6,11),(x-6,11),(x-6,5)],GOLD)
    return im


def outfit(art, science, rank, f):
    # Torso and tails cover only the neutral body: its arms, legs and face animate normally.
    im=Image.new('RGBA',(48,64));d=ImageDraw.Draw(im);c=COLORS[science]
    outer=['#e2e7dd','#d5e4df','#c7dae0','#b9cbd6','#7f97ad','#617f9d','#263648','#182b41'][rank]
    edge='#476379' if rank<6 else c
    coat=[(16,26),(31,26),(35,45),(28,46),(24,40),(20,46),(13,45)]
    if rank==0:coat=[(14,26),(33,26),(37,47),(28,48),(24,42),(19,48),(11,47)]
    if rank>=2:coat=[(16,26),(31,26),(35,48),(29,47),(27,40),(24,43),(21,40),(19,48),(12,47)]
    if rank>=4:coat=[(16,25),(31,25),(36,51),(29,49),(26,40),(24,43),(21,40),(18,51),(11,49)]
    if rank==7:
        # Floating tails are drawn by the equipment layer, so the body has a cutaway jacket.
        coat=[(16,25),(31,25),(34,41),(28,43),(24,38),(20,43),(13,41)]
    d.polygon(coat,fill=outer,outline=DARK)
    if rank>=5:
        # The coat contains a moving night sky, confined to its existing silhouette.
        mask=Image.new('L',im.size);ImageDraw.Draw(mask).polygon(coat,fill=255)
        for n in range(12):
            x=12+(n*7+f)%24;y=29+(n*5+f//2)%21
            if mask.getpixel((x,y)):d.point((x,y),fill=cosmic.spectral(n/12+f/32,210))
    d.polygon([(18,27),(24,32),(30,27),(27,40),(21,40)],fill='#25374b')
    line(d,[(18,28),(22,34),(20,42)],edge)
    line(d,[(30,28),(26,34),(28,42)],edge)
    d.rectangle((23,29,25,35),fill=c)
    if rank==0:
        d.rectangle((14,35,19,39),fill='#8fa5b0');d.rectangle((29,34,33,37),fill='#b29f84')
        d.rectangle((12,39,19,44),fill='#465d70',outline=DARK);line(d,[(13,40),(18,40)],'#dfd7b4')
    if rank>=1:
        line(d,[(15,39),(32,39)],'#746142',2)
        for x in [16,29]:d.rectangle((x,38,x+2,42),fill=GOLD)
        # Goggles sit above the eyes, keeping each head's recognizable hair silhouette.
        line(d,[(19,13),(29,13)],'#576d7b');d.rectangle((20,12,23,14),outline=c);d.rectangle((26,12,29,14),outline=c)
    if rank>=2:
        for x in [15,32]:line(d,[(x,42),(x+(1 if x<24 else -1),46)],c)
    if rank>=3:
        if science==0:prism(d,33,27,3,c,f)
        elif science==1:line(d,[(14,26),(18,23),(18,30),(14,26)],GOLD)
        else:diamond(d,33,27,3,c)
    if rank>=4:
        for y in [42,45,48]:line(d,[(13,y),(17,y-1)],rgba(c,150));line(d,[(30,y-1),(34,y)],rgba(c,150))
        d.rectangle((17,25,20,27),fill=GOLD)
    if rank>=5:
        d.polygon([(29,28),(34,29),(35,37),(31,36)],fill='#243a52',outline=c)
        line(d,[(30,31),(33,32),(31,34)],WHITE)
    if rank>=6:
        line(d,[(15,28),(13,41),(17,46)],c,2);line(d,[(32,28),(34,41),(31,46)],c,2)
        for k in range(3):d.point((15+k,44+f%2),fill=COLORS[k])
    # Hair overlays go over the goggles' edges; eyes and moustache remain unobscured.
    head=art.overlay(science)
    head.paste((0,0,0,0),(0,27,48,64))
    im.alpha_composite(head)
    return im


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
    if rank>=3:return cosmic.equipment(science,rank,f,front,podium)
    im=Image.new('RGBA',(96,112));d=ImageDraw.Draw(im)
    if front or rank==0:return im
    instrument(d,science,77,55+f%2,f,rank==2)
    if rank==2:
        line(d,[(72,70),(85,67),(85,78),(72,79),(72,70)],rgba(COLORS[science],130))
        line(d,[(74,74),(78,71),(82,75)],COLORS[science])
    return im


def field(kind,tier,f,size=128):
    return cosmic.field(kind,tier,f,size)


def skill_effect(skill,tier,f,skills):
    return cosmic.skill_effect(skill,tier,f,skills)


def packet(skill,tier,f,heading,skills):
    return cosmic.packet(skill,tier,f,heading,skills)


def modifier(mask,tier,f):
    im=Image.new('RGBA',(64,80));d=ImageDraw.Draw(im)
    for k in range(3):
        if not mask&(1<<k):continue
        x=13+k*19;y=65
        if k==0:
            line(d,[(x-5,y),(x,y),(x+5,y-5)],COLORS[1]);line(d,[(x,y),(x+5,y+5)],COLORS[1])
        elif k==1:
            diamond(d,x,y,5,COLORS[2]);diamond(d,x,y,2,GOLD)
        else:
            ring(d,x,y,5,COLORS[2],f*.2,12);line(d,[(x,y),(x+2,y-3)],WHITE)
        if tier>=2:star(d,x,y-8,WHITE,1)
    return im


def completion(tier,podium,f):
    return cosmic.completion(tier,podium,f)


def generate(art):
    mod=ROOT/'mods/tfm2_custom/vfx';skills=art.DATA['skills']
    # Separate bounded atlases rather than relying on an engine-sized monolithic sheet.
    badges={f'rank{r}':[(badge(r,f),.12) for f in range(8)] for r in range(7)}
    badges.update({f'top{p}':[(badge(7,f,p),.12) for f in range(8)] for p in range(1,11)})
    art.atlas(mod,'science_badges',badges,64,96,16)
    outfits={f'outfit{s}_r{r}':[(outfit(art,s,r,f),.12) for f in range(8)] for s in range(3) for r in range(8)}
    art.atlas(mod,'science_outfits',outfits,48,64,16)
    for s in range(3):
        gear={}
        for r in range(8):
            for podium in ([1,2,3,4] if r==7 else [4]):
                suffix=f'{s}_r{r}'+(f'_p{podium}' if r==7 else '')
                for front in [False,True]:
                    tag=('gearfront' if front else 'gearback')+suffix
                    gear[tag]=[(equipment(s,r,f,front,podium),.12) for f in range(8)]
        art.atlas(mod,f'science_equipment{s}',gear,96,112,16,trim=True,palette=True)
        for tier in range(4):
            casts={f'skill_{skills[i]["id"]}_t{tier}':[(skill_effect(i,tier,f,skills),.075) for f in range(8)] for i in range(s*25,s*25+25)}
            art.atlas(mod,f'science_skills{s}_t{tier}',casts,128,128,16,trim=True,palette=True)
    for tier in range(4):
        fields={f'field{k}_t{tier}':[(field(k,tier,f),.1) for f in range(8)] for k in FIELDS}
        art.atlas(mod,f'science_fields_t{tier}',fields,128,128,16,trim=True,palette=True)
        # Eight unique sprites per loop. Pair aliases reuse those pixels; the
        # moving renderer plays only one 12-tick pair at a time, like Isliid.
        name=f'science_packets_t{tier}'
        packets={f'packet_{skills[s]["id"]}_t{tier}_a{h}':[(packet(s,tier,f,h,skills),.1) for f in range(8)] for s in PACKETS for h in range(8)}
        art.atlas(mod,name,packets,64,64,32,trim=True,palette=True)
        path=mod/(name+'#anim.fanim');meta=json.loads(path.read_text())
        for tag,anim in list(meta['anims'].items()):
            for pair in range(4):meta['anims'][tag+f'_pair{pair}']={'frames':anim['frames'][pair*2:pair*2+2]}
        path.write_text(json.dumps(meta,indent=2)+'\n')
        for suffix in ['#anim.fanim','#sheet.png']:
            obsolete=mod/(name+'_b'+suffix)
            if obsolete.exists():obsolete.unlink()
    misc={f'modifiers{t}_{mask}':[(modifier(mask,t,f),.12) for f in range(8)] for t in range(4) for mask in range(1,8)}
    art.atlas(mod,'science_modifiers',misc,64,80,16)
    complete={f'complete_t{t}_p{p}':[(completion(t,p,f),.1) for f in range(8)] for t in range(4) for p in ([1,2,3,4] if t==3 else [4])}
    art.atlas(mod,'science_completion',complete,128,128,16,trim=True,palette=True)
    echoes={}
    for s in range(3):
        seq=[]
        for f in range(8):
            im=Image.new('RGBA',(96,112));d=ImageDraw.Draw(im)
            pts=[point(48,83,24,k*math.tau/3,.25) for k in range(4)]
            line(d,pts,rgba(COLORS[s],180-f*15),2)
            cosmic.paste(im,cosmic.singularity(12,f,True,.25),48,83)
            seq.append((im,1/60))
        echoes[f'echo{s}']=seq
    art.atlas(mod,'science_echo',echoes,96,112,16,trim=True,palette=True)
    previews(art,skills)
    print('Cosmic Unified Experiment: 17 badges, 24 outfits, 300 eight-frame casts, 44 eight-frame fields, 448 eight-frame packet loops with shared pair aliases')


def previews(art,skills):
    folder=ROOT/'docs';editor=ROOT/'editor';bg='#0b1422';font=ImageFont.truetype(FONT,13)
    runtime={};decoded={}
    for file in sorted((ROOT/'mods/tfm2_custom/vfx').glob('science_*#anim.fanim')):
        name=file.name.split('#')[0]
        for tag,anim in json.loads(file.read_text())['anims'].items():runtime[tag]={'sheet':name,'frames':anim['frames']}
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
        im=original('gearback'+suffix,f,96,112)
        base=art.body('idle',f);base.alpha_composite(original(f'outfit{s}_r{r}',f,48,64))
        im.alpha_composite(base,(24,24));im.alpha_composite(original('gearfront'+suffix,f,96,112))
        return im
    im=Image.new('RGBA',(1440,1178),bg);d=ImageDraw.Draw(im)
    title(d,24,18,'THE WALKING UNIFIED EXPERIMENT / EIGHT MASTERY RANKS',24)
    title(d,24,54,'Actual runtime layers, enlarged with nearest-neighbour scaling. All three forms share the same body.',13)
    for s,name in enumerate(['EINSTEIN / SPACE','NEWTON / MATHEMATICS','MARIE CURIE / MATTER']):
        y=97+s*350;title(d,24,y,name,19,COLORS[s])
        for r in range(8):
            x=12+r*178;d.rounded_rectangle((x,y+32,x+168,y+333),8,fill='#142338')
            sprite=composed(s,r).resize((144,168),Image.Resampling.NEAREST)
            im.alpha_composite(sprite,(x+12,y+60))
            b=badge(r,2,10 if r==7 else None);b=b.crop(b.getbbox());b=b.resize((b.width*2,b.height*2),Image.Resampling.NEAREST)
            im.alpha_composite(b,(x+84-b.width//2,y+252-b.height//2))
            parts=RANKS[r].split(' ')
            for k,p in enumerate(parts):title(d,x+8,y+290+k*15,p,12)
    im.convert('RGB').save(folder/'unified-theory-mastery.png')
    # Browser-friendly composite frames for an actual-asset animated rank inspector.
    tiles={};sheet=Image.new('RGBA',(8*96,3*11*112));n=0
    for s in range(3):
        for r in range(8):
            for podium in ([1,2,3,4] if r==7 else [4]):
                tag=f'{s}_{r}_{podium}';frames=[]
                for f in range(8):
                    x=f*96;y=n*112;sheet.alpha_composite(composed(s,r,f,podium),(x,y));frames.append({'x':x,'y':y,'w':96,'h':112})
                tiles[tag]=frames;n+=1
    sheet.save(editor/'science-mastery-preview.png')
    assets={tag:asset for tag,asset in runtime.items() if tag.startswith(('skill_','rank','top','complete','echo','field','packet_')) and '_pair' not in tag}
    (editor/'science-art-preview.json').write_text(json.dumps({'outfits':tiles,'assets':assets,'frames':8,'frameSeconds':.12},indent=2)+'\n')
    # Keep the existing portrait coordinates used by the trajectory study.
    portrait=Image.new('RGBA',(960,360),bg);d=ImageDraw.Draw(portrait)
    for s,name in enumerate(['EINSTEIN','NEWTON','MARIE CURIE']):
        x=s*320;d.rounded_rectangle((x+12,12,x+308,348),12,fill='#142338',outline=COLORS[s],width=2)
        body=art.body('idle',2);body.alpha_composite(outfit(art,s,5,2))
        portrait.alpha_composite(body.resize((192,256),Image.Resampling.NEAREST),(x+64,42))
        title(d,x+25,318,name,18,COLORS[s])
    portrait.save(editor/'science-portraits.png')
    im=Image.new('RGBA',(1152,960),bg);d=ImageDraw.Draw(im)
    title(d,24,18,'TOP 10 / WALKING UNIFIED EXPERIMENT',24)
    for s,name in enumerate(['EINSTEIN','NEWTON','MARIE CURIE']):
        y=70+s*290;title(d,18,y,name,16,COLORS[s])
        for j,p in enumerate([4,3,2,1]):
            x=12+j*285;d.rounded_rectangle((x,y+27,x+272,y+273),8,fill='#142338')
            im.alpha_composite(composed(s,7,3,p).resize((192,224),Image.Resampling.NEAREST),(x+40,y+32))
            title(d,x+12,y+251,'#4-10' if p==4 else f'#{p}'+(' / THE THEORY HOLDS' if p==1 else ''),12)
    im.convert('RGB').save(folder/'unified-theory-top10.png')
    show=[0,1,5,12,13,18,20,34,40,45,57,58,65,66,73,27,61,70]
    im=Image.new('RGBA',(1096,84+len(show)*166),bg);d=ImageDraw.Draw(im)
    title(d,24,16,'SKILL EFFECTS / ORIGINAL RUNTIME FRAMES',22)
    for t,name in enumerate(['COSMIC SEED','CELESTIAL ENGINE','IMPOSSIBLE LAB','UNIFIED UNIVERSE']):title(d,284+t*200,53,name,12,COLORS[t%3])
    for row,s in enumerate(show):
        y=85+row*166;title(d,12,y+65,skills[s]['name'][:26],12,COLORS[skills[s]['science']])
        for t in range(4):
            tile=original(f'skill_{skills[s]["id"]}_t{t}',3,128,128).resize((144,144),Image.Resampling.NEAREST)
            im.alpha_composite(tile,(270+t*200,y))
    im.convert('RGB').save(folder/'unified-theory-effects.png')
    # Eight-frame strips and a looping scene use exactly the runtime functions.
    show=[(12,'EINSTEIN / ACCRETION WELL'),(18,'EINSTEIN / FOLDED TUNNEL'),
          (34,'NEWTON / CELESTIAL MOMENTUM'),(49,'NEWTON / PRINCIPIA MACHINE'),
          (57,'CURIE / CRYSTAL CATHEDRAL'),(74,'CURIE / RADIOACTIVE NEBULA')]
    strip=Image.new('RGBA',(1056,60+len(show)*164),bg);d=ImageDraw.Draw(strip)
    title(d,16,14,'EIGHT ORIGINAL SPRITES / COSMIC UNIFIED EXPERIMENT',20)
    for row,(skill,label) in enumerate(show):
        y=60+row*164;title(d,16,y,label,13,COLORS[skills[skill]['science']])
        for f in range(8):
            strip.alpha_composite(original(f'skill_{skills[skill]["id"]}_t3',f,128,128),(16+f*128,y+23))
            title(d,20+f*128,y+144,str(f+1),10)
    strip.convert('RGB').save(folder/'unified-theory-cosmic-frames.png')
    movie=[]
    for f in range(8):
        scene=Image.new('RGBA',(960,560),bg);d=ImageDraw.Draw(scene)
        title(d,20,16,'THE UNIFIED THEORY / A WALKING UNIVERSE',24)
        for s,label in enumerate(['FOLDED SPACE','CELESTIAL MACHINERY','CRYSTAL NEBULA']):
            x=s*320;title(d,x+20,66,label,16,COLORS[s])
            scene.alpha_composite(composed(s,7,f,1).resize((192,224),Image.Resampling.NEAREST),(x+64,96))
            sprite=original(f'skill_{skills[[12,49,57][s]]["id"]}_t3',f,128,128).resize((256,256),Image.Resampling.NEAREST)
            scene.alpha_composite(sprite,(x+32,290))
        movie.append(scene.convert('RGB').quantize(colors=256))
    movie[0].save(folder/'unified-theory-cosmic.gif',save_all=True,append_images=movie[1:],duration=100,loop=0,disposal=2)
