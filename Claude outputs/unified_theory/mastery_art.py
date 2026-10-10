"""Reproducible runtime pixel art for the Walking Unified Experiment.

All drawing uses native pixels. Buff canvases share the body's centre; floating
equipment stays clear of the face and never depends on the host's facing.
"""
import json
import math
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

HERE = Path(__file__).resolve().parent
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
    im=Image.new('RGBA',(96,112));d=ImageDraw.Draw(im);c=COLORS[science]
    # Body is placed at (24,24), i.e. face at y=40 and feet at y=82.
    cy=61;phase=f*math.tau/8
    if front:
        if rank<5:return im
        for k in range(3 if rank>=5 else 1):
            a=phase+k*math.tau/3
            if math.sin(a)<0:continue
            x,y=point(48,cy,29,a,.4)
            line(d,[(x-3,y),(x+3,y)],rgba(COLORS[k],130))
            diamond(d,x,y,2,COLORS[k])
        if rank>=6:
            ring(d,48,83,24,rgba(c,100),phase,36,.18)
        return im
    if rank==0:return im
    if rank<=2:
        instrument(d,science,77,55+f%2,f,rank==2)
        if rank==2:
            line(d,[(72,70),(85,67),(85,78),(72,79),(72,70)],rgba(c,130))
            line(d,[(74,74),(78,71),(82,75)],c)
        return im
    n=1 if rank==3 else 2 if rank==4 else 3
    for k in range(n):
        a=phase*.25+k*math.tau/n-.5
        x,y=point(48,cy,28,a,.62)
        instrument(d,(science+k)%3 if rank>=5 else science,x,y,f,True)
        if rank>=4:
            nx,ny=point(48,cy,28,a+math.tau/n,.62)
            line(d,[(x,y),(nx,ny)],rgba(c,70))
    if rank>=5:
        pts=[point(48,cy,35,phase*.25+k*math.tau/3,.6) for k in range(4)]
        line(d,pts,rgba(c,80))
    if rank>=6:
        line(d,[(14,79),(23,70),(48,74),(73,70),(82,79),(48,91),(14,79)],rgba(c,130))
        for k in range(3):
            x,y=point(48,81,19,k*math.tau/3+phase,.3)
            diamond(d,x,y,3,COLORS[k],rgba(COLORS[k],45))
    if rank==7:
        # Space bends, mechanics stays rigid, matter crystallizes: three different panels.
        for k in range(3):
            a=k*math.tau/3+phase*.22+science*.35
            x,y=point(48,72,21,a,.48)
            if k==0:
                bend=round(math.sin(phase)*2)
                pts=[(x-5,y),(x+4,y-2),(x+7+bend,y+11),(x,y+15),(x-7+bend,y+11),(x-5,y)]
                d.polygon(pts,fill='#152d48');line(d,pts,COLORS[0]);ring(d,x,y+7,4,COLORS[0],phase,16,.5)
            elif k==1:
                pts=[(x-5,y),(x+5,y),(x+5,y+13),(x-5,y+13),(x-5,y)]
                d.polygon(pts,fill='#3a3024');line(d,pts,COLORS[1]);line(d,[(x-3,y+10),(x+3,y+3)],GOLD)
            else:
                growth=(f if f<4 else 7-f)
                pts=[(x,y-2-growth),(x+6,y+4),(x+3,y+11+growth),(x-4,y+11),(x-6,y+4),(x,y-2-growth)]
                d.polygon(pts,fill='#153930');line(d,pts,COLORS[2]);line(d,[(x,y-2),(x,y+10),(x+3,y+14)],WHITE)
                if f>=4:star(d,x+7,y-f,COLORS[2],1)
        if podium==3:
            for k in range(3):prism(d,*point(48,32,24,k*math.tau/3+phase*.15),3,COLORS[k],f)
        elif podium==2:
            ring(d,48,25,13,COLORS[0],phase,24,.55)
            ring(d,48,25,8,COLORS[2],-phase,6,.8)
        elif podium==1:
            knot(d,48,24,16,f)
            for k in range(3):star(d,*point(48,24,20,k*math.tau/3+phase*.2),COLORS[k],2)
    return im


def field(kind,tier,f,size=96):
    im=Image.new('RGBA',(size,size));d=ImageDraw.Draw(im);c=COLORS[2 if kind>=50 else 1 if kind==45 else 0]
    cx=cy=size//2;phase=f*math.tau/8;rough=tier==0
    if kind==1:
        line(d,[(21,70),(72,70),(66,66)],c)
        line(d,[(28,75),(28,22),(24,29)],c)
        line(d,[(28,70),(67,32),(63,33)],c)
        diamond(d,28,70,3,WHITE)
        if tier>=1:
            for k in range(4):line(d,[(37+k*8,69),(37+k*8,72)],c)
        if tier>=2:
            line(d,[(28,22),(72,22),(72,70)],rgba(c,110))
    elif kind==5:
        for i,r in enumerate([18,28,37]):
            ring(d,cx,cy,r,rgba(c,190-i*35),phase/(i+1),36,.72,rough)
            x,y=point(cx,cy,r,phase/(i+1)-math.pi/2,.72);line(d,[(cx,cy),(x,y)],rgba(c,190-i*35))
            if tier>=1:
                for k in range(12):d.point(point(cx,cy,r,k*math.tau/12,.72),fill=WHITE)
    elif kind==12:
        d.ellipse((38,39,58,53),fill='#050b16')
        for i in range(5):
            y=24+i*10;pts=[]
            for x in range(18,79,3):
                bend=round(14*math.exp(-((x-48)**2+(y-48)**2)/300))
                pts.append((x,y+bend))
            line(d,pts,rgba(c,110+i*20))
        if tier>=1:
            for x in [27,39,57,69]:line(d,[(x,22),(48+(x-48)*.7,52),(x,76)],rgba(c,110))
        ring(d,48,47,10,WHITE,phase,24,.6)
    elif kind==13:
        d.ellipse((36,18,60,78),fill=rgba(c,25),outline=c)
        for i in range(3):
            y=34+i*14;line(d,[(13,y),(36,y),(47,y+(i-1)*6),(60,48),(83,48+(i-1)*8)],COLORS[i] if tier>=2 else c)
        line(d,[(48,18),(43,48),(48,78)],WHITE)
    elif kind in (18,20):
        if kind==20:d.ellipse((24,32,72,64),fill='#060d18')
        for i in range(3):
            pts=[point(cx,cy,6+k*.42,phase+i*math.tau/3+k*.085,.55) for k in range(75)]
            line(d,pts,rgba(c,180-i*35),2 if tier==3 else 1)
        if kind==20:ring(d,cx,cy,29,WHITE,phase,36,.5)
    elif kind==45:
        line(d,[(18,71),(18,24),(15,30)],rgba(c,140));line(d,[(15,69),(78,69),(72,65)],rgba(c,140))
        pts=[(x,round(65-(x-20)*.5)) for x in range(20,79)]
        curve=[(x,round(28+(x-48)**2/60+f%3)) for x in range(20,79)]
        line(d,pts,c);line(d,curve,WHITE)
        diamond(d,48+f%3,51,4,GOLD)
    elif kind==57:
        for k in range(6):
            a=k*math.tau/6; r=18+(f+k)%4
            x,y=point(cx,cy,r,a,.7)
            pts=[(x,y-12),(x+6,y-1),(x+3,y+11),(x-5,y+6),(x-6,y-1),(x,y-12)]
            d.polygon(pts,fill=rgba(c,45));line(d,pts,c);line(d,[(x,y-12),(x,y+6),(x+3,y+11)],WHITE)
            line(d,[(cx,cy),(x,y)],rgba(c,140))
    elif kind==58:
        pts=[(24+x*12,24+y*12) for y in range(5) for x in range(5)]
        for x,y in pts:
            if x<72:line(d,[(x,y),(x+12,y)],rgba(c,150))
            if y<72:line(d,[(x,y),(x,y+12)],rgba(c,150))
            diamond(d,x,y,2+(f+x//12+y//12)%2,c,rgba(c,90))
    elif kind==65:
        for i in range(36):
            a=i*2.399+phase*.2;r=math.sqrt(i)*5+f*.7
            x,y=point(cx,cy,r,a,.7)
            d.ellipse((x-2,y-2,x+2,y+2),fill=rgba(c,210-i*4))
        if tier>=2:ring(d,cx,cy,34,rgba(c,80),phase,24,.7)
    elif kind==66:
        for y in range(22,79,8):
            line(d,[(45,y),(45,y+4)],rgba(c,200));line(d,[(51,y),(51,y+4)],rgba(c,200))
            for i in range(3):
                x=20+i*13+(f*3)%13;d.point((x,y+((i+f)%3)),fill=c)
        line(d,[(57,48),(75,48),(69,43)],WHITE,2)
        if tier>=2:
            for k in range(6):diamond(d,48,25+k*9,2,c)
    if tier==0:
        for x in range(18,79,11):line(d,[(x,83),(x+3,81)],rgba(c,135))
    else:
        line(d,[(19,83),(77,83)],rgba(c,140))
        for x in range(20,78,14):line(d,[(x,80),(x,85)],rgba(c,170))
    if tier>=2:
        for i in range(3):
            x,y=point(cx,cy,39,phase*.25+i*math.tau/3,.72)
            diamond(d,x,y,2,c)
    if tier==3:
        # A constructed ground plane unifies the final tier without replacing each skill's shape.
        line(d,[(12,64),(48,82),(84,64)],rgba(c,100))
        for k in range(3):star(d,*point(cx,cy,38,phase*.25+k*math.tau/3,.7),COLORS[k],2)
    return im


def skill_effect(skill,tier,f,skills):
    science=skills[skill]['science'];c=COLORS[science]
    if skill in FIELDS:return field(skill,tier,f)
    im=Image.new('RGBA',(96,96));d=ImageDraw.Draw(im);phase=f*math.tau/6
    role=skills[skill]['role'];local=skill%25
    if skill in (0,73):
        prism(d,32,48,10,c,f)
        line(d,[(13,48),(30,48),(83,48)],WHITE,2)
        for k in range(3):line(d,[(39,48),(81,43+k*5)],COLORS[k] if skill==0 else rgba(c,180-k*30))
        if skill==73:
            for x in [54,68,80]:line(d,[(x,39),(x,57)],c)
    elif skill==34:
        for k in range(3):
            x=23+k*11;d.rectangle((x,41-k*3,x+7,55+k*3),fill=rgba(c,80+k*40),outline=c)
        line(d,[(23,48),(79,48),(70,41)],WHITE,2);line(d,[(79,48),(70,55)],WHITE,2)
    elif skill==40:
        pts=[(x,round(64-(x-48)**2/65)) for x in range(15,82)]
        for i,p in enumerate(pts):
            if i%5==0:d.point(p,fill=rgba(c,140))
        x=18+f*11;y=round(64-(x-48)**2/65);diamond(d,x,y,4,c,WHITE)
        line(d,[(18,76),(82,76)],rgba(c,90))
    elif skill==27:
        line(d,[(20,48),(43,48)],c,2)
        for k in range(3):
            y=29+k*19;line(d,[(43,48),(66,y),(81,y)],c);diamond(d,77,y,3,WHITE)
    elif skill==61:
        pts=[point(48,48,22,k*math.tau/6) for k in range(7)];line(d,pts,c)
        for k,p in enumerate(pts[:-1]):diamond(d,*p,3,c)
        diamond(d,48,48,7,GOLD,rgba(c,90));line(d,[(26,48),(41,48),(55,48),(70,48)],WHITE)
    elif skill in (70,74):
        for k in range(3 if skill==74 else 2):
            x=25+k*23;ring(d,x,48,8,c,phase,16);line(d,[(x,48),(x+4,43)],WHITE)
            if k<2:line(d,[(x+9,48),(x+15,48)],rgba(c,170))
            for n in range(max(1,4-k)):d.point(point(x,48,12,n*math.tau/4+phase),fill=c)
    elif role in ('Attack','Heavy'):
        for k in range(3):
            y=36+k*12;line(d,[(18,y),(44+k*8,y),(70,y+(k-1)*4)],rgba(c,160+k*30))
        if science==1:instrument(d,1,49,47,f,True)
        else:prism(d,49,48,10,c,f)
        star(d,75,48,WHITE,3)
    elif role=='Protect':
        pts=[(48,18),(70,28),(67,58),(48,75),(29,58),(26,28),(48,18)]
        d.polygon(pts,fill=rgba(c,25));line(d,pts,c)
        if science==2:prism(d,48,47,11,c,f)
        else:line(d,[(36,48),(48,34),(60,48),(48,62),(36,48)],WHITE)
    elif role=='Move':
        for k in range(3):
            x=22+k*19;line(d,[(x,34),(x+10,48),(x,62)],rgba(c,90+k*60),2)
        line(d,[(17,68),(77,68)],rgba(c,130))
    elif role=='Observe':
        ring(d,46,45,20,c,phase,32,.6,tier==0)
        line(d,[(61,59),(73,72)],WHITE,2)
        for k in range(3):diamond(d,*point(46,45,12,phase+k*math.tau/3),2,c)
    elif role=='Capstone':
        knot(d,48,48,27,f)
        for k in range(3):instrument(d,k,*point(48,48,32,k*math.tau/3+phase*.15),f,True)
    else:
        # Distinct recipe topology: nodes + bonds encode the skill's tokens and position in its notebook.
        n=len(skills[skill]['tokens']);pts=[point(48,48,22,phase*.1+k*math.tau/n+local*.13) for k in range(n)]
        line(d,pts+[pts[0]],rgba(c,170))
        for k,p in enumerate(pts):
            diamond(d,*p,2+(local+k)%3,c)
            if (local>>k)&1:line(d,[p,(48,48)],rgba(c,130))
        instrument(d,science,48,48,f,local%2==0)
    if tier==0:
        for k in range(3):line(d,[(16+k*8,79),(19+k*8,77)],rgba(c,130))
    elif tier>=1:
        for k in range(4):line(d,[(20+k*15,76),(20+k*15,79)],rgba(c,140))
    if tier>=2:
        for k in range(3):diamond(d,*point(48,48,34,phase*.2+k*math.tau/3),2,c)
    if tier==3:
        line(d,[(14,68),(48,84),(82,68)],rgba(c,130))
        prism(d,48,22,4,GOLD,f)
    # Unique measurement signature for all 75 casts, without putting text into combat VFX.
    for k in range(5):
        if (local+1)&(1<<k):d.rectangle((36+k*5,85,38+k*5,86),fill=c)
    return im


def packet(skill,tier,f,heading,skills):
    im=Image.new('RGBA',(64,64));d=ImageDraw.Draw(im);c=COLORS[skills[skill]['science']]
    def p(x,y):
        a=heading*math.tau/16
        return round(32+x*math.cos(a)-y*math.sin(a)),round(32+x*math.sin(a)+y*math.cos(a))
    if skill==73:
        line(d,[p(-23,0),p(21,0)],WHITE,2)
        for y in [-3,3]:line(d,[p(-20,y),p(15,y)],rgba(c,190))
        for x in [-12,0,12]:line(d,[p(x,-6),p(x,6)],c)
    elif skill in (34,39):
        for k in range(3):
            x=-12+k*8;pts=[p(x,-5-k),p(x+5,-5-k),p(x+5,5+k),p(x,5+k)];d.polygon(pts,fill=rgba(c,120+k*40),outline=c)
        line(d,[p(9,-6),p(19,0),p(9,6)],WHITE,2)
    elif skill==40:
        line(d,[p(-23,8),p(-16,3),p(-6,0),p(10,0)],rgba(c,180))
        d.polygon([p(9,-4),p(19,0),p(9,4)],fill=c)
        for x in [-22,-17,-12]:d.point(p(x,9-(x+22)//2),fill=WHITE)
    elif skill in (54,59,63):
        pts=[p(10,-6),p(20,0),p(10,6),p(3,0)]
        d.polygon(pts,fill=rgba(c,180),outline=WHITE)
        for k in range(5):
            x=-18+k*5;y=((f+k)%3-1)*3;d.ellipse((*p(x,y),*p(x,y)),fill=c)
        line(d,[p(-20,0),p(7,0)],rgba(c,130))
    elif skill in (71,72,74):
        for k in range(3):
            x=-8+k*8;ring(d,*p(x,0),3,rgba(c,140+k*30),f*.2,12)
        line(d,[p(-23,0),p(13,0)],WHITE)
        if skill==74:
            for y in [-5,5]:line(d,[p(-15,y),p(-3,y)],c)
    else:
        line(d,[p(-23,0),p(8,0)],rgba(c,190),2)
        pts=[p(8,-5),p(18,0),p(8,5),p(4,0)]
        d.polygon(pts,fill=c,outline=WHITE)
        if skill==0:
            for k,col in enumerate(COLORS):line(d,[p(-17,4+k*2),p(-5,2+k)],rgba(col,150))
    if tier>=1:line(d,[p(-18,-8),p(-5,-8)],rgba(c,130))
    if tier>=2:
        for y in [-9,9]:diamond(d,*p(-7,y),2,c)
    if tier==3:star(d,*p(0,0),WHITE,2)
    return im


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
    im=Image.new('RGBA',(96,112));d=ImageDraw.Draw(im);phase=f*math.tau/8
    r=12+min(f,5)*4
    knot(d,48,50,r,f)
    for k in range(3):
        x,y=point(48,50,min(r+6,27),k*math.tau/3+phase*.1,.8)
        instrument(d,k,x,y,f,tier>=2)
        line(d,[(x,y),(48,50)],rgba(COLORS[k],140))
    if tier==3:
        if podium==1:
            # Offset front/back triangles assemble the impossible machine in perspective.
            a=[point(48,47,24,k*math.tau/3) for k in range(4)]
            b=[(x+5,y+8) for x,y in a]
            line(d,a,WHITE,2);line(d,b,GOLD)
            for x,y in a[:-1]:line(d,[(x,y),(x+5,y+8)],COLORS[f%3])
        elif podium==2:
            ring(d,48,50,12,COLORS[0],phase,24,.45);ring(d,48,50,8,COLORS[2],-phase,6)
        elif podium==3:
            for k in range(3):prism(d,*point(48,50,23,k*math.tau/3),4,COLORS[k],f)
    return im


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
        art.atlas(mod,f'science_equipment{s}',gear,96,112,16)
        for tier in range(4):
            casts={f'skill_{skills[i]["id"]}_t{tier}':[(skill_effect(i,tier,f,skills),.06) for f in range(6)] for i in range(s*25,s*25+25)}
            art.atlas(mod,f'science_skills{s}_t{tier}',casts,96,96,16)
    for tier in range(4):
        fields={f'field{k}_t{tier}':[(field(k,tier,f),.21) for f in range(8)] for k in FIELDS}
        art.atlas(mod,f'science_fields_t{tier}',fields,96,96,16)
        packets={f'packet_{skills[s]["id"]}_t{tier}_a{h}':[(packet(s,tier,f,h,skills),.06) for f in range(4)] for s in PACKETS for h in range(16)}
        art.atlas(mod,f'science_packets_t{tier}',packets,64,64,32)
    misc={f'modifiers{t}_{mask}':[(modifier(mask,t,f),.12) for f in range(8)] for t in range(4) for mask in range(1,8)}
    art.atlas(mod,'science_modifiers',misc,64,80,16)
    complete={f'complete_t{t}_p{p}':[(completion(t,p,f),.1) for f in range(8)] for t in range(4) for p in ([1,2,3,4] if t==3 else [4])}
    art.atlas(mod,'science_completion',complete,96,112,16)
    echoes={}
    for s in range(3):
        seq=[]
        for f in range(4):
            im=Image.new('RGBA',(96,112));d=ImageDraw.Draw(im)
            pts=[point(48,83,24,k*math.tau/3,.25) for k in range(4)]
            line(d,pts,rgba(COLORS[s],145-f*20),2)
            seq.append((im,.04))
        echoes[f'echo{s}']=seq
    art.atlas(mod,'science_echo',echoes,96,112,16)
    previews(art,skills)
    print('Walking Unified Experiment: 17 animated badges, 24 outfits, 3 laboratories, 300 skill casts, 896 directional packet tags, 44 fields, modifiers and podium completions')


def previews(art,skills):
    folder=ROOT/'docs';editor=ROOT/'editor';bg='#0b1422';font=ImageFont.truetype(FONT,13)
    def title(d,x,y,text,size=13,color='#dcecf4'):
        d.text((x,y),text,font=ImageFont.truetype(FONT,size),fill=color)
    def composed(s,r,f=2,podium=4):
        im=equipment(s,r,f,False,podium)
        base=art.body('idle',f);base.alpha_composite(outfit(art,s,r,f))
        im.alpha_composite(base,(24,24));im.alpha_composite(equipment(s,r,f,True,podium))
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
    assets={}
    for file in sorted((ROOT/'mods/tfm2_custom/vfx').glob('science_*#anim.fanim')):
        name=file.name.split('#')[0]
        for tag,anim in json.loads(file.read_text())['anims'].items():
            if tag.startswith(('skill_','rank','top','complete','echo')):
                assets[tag]={'sheet':name,'frames':anim['frames']}
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
    for t,name in enumerate(['CHALK','INSTRUMENT','LABORATORY','UNIFIED MIND']):title(d,284+t*200,53,name,12,COLORS[t%3])
    for row,s in enumerate(show):
        y=85+row*166;title(d,12,y+65,skills[s]['name'][:26],12,COLORS[skills[s]['science']])
        for t in range(4):
            tile=skill_effect(s,t,3,skills).resize((144,144),Image.Resampling.NEAREST)
            im.alpha_composite(tile,(270+t*200,y))
    im.convert('RGB').save(folder/'unified-theory-effects.png')
