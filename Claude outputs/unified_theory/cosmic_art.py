"""Baked eight-frame celestial sprites; no particles or shaders run in the game.

Einstein folds star-filled space. Newton builds luminous celestial machinery.
Curie grows radioactive crystal nebulae. All functions return centred, original
pixel sprites; a fixed seed and eight discrete phases make builds reproducible.
"""
import colorsys
import math
from functools import lru_cache

from PIL import Image, ImageDraw, ImageFilter

TAU = math.tau
COLORS = [(99, 191, 255), (255, 201, 107), (99, 225, 189)]
WHITE = (238, 250, 255, 255)


def col(science, alpha=255):
    return COLORS[science] + (alpha,)


def spectral(t, alpha=255):
    return tuple(round(v*255) for v in colorsys.hsv_to_rgb(t % 1, .55, 1)) + (alpha,)


def p(x, y, r, a, flat=1):
    return round(x+r*math.cos(a)), round(y+r*math.sin(a)*flat)


def path(d, points, color, width=1):
    d.line([(round(x), round(y)) for x, y in points], fill=color, width=width)


def sparkle(d, x, y, color=WHITE, r=2):
    path(d, [(x-r,y),(x+r,y)], color)
    path(d, [(x,y-r),(x,y+r)], color)
    d.point((x,y), fill=WHITE)


def glow(im, strength=1.3):
    """Bake a restrained halo behind the solid sprite, once at asset generation."""
    halo = im.filter(ImageFilter.GaussianBlur(strength))
    halo.alpha_composite(im)
    return halo


def ellipse(d, x, y, r, science, phase, flat=.45, prismatic=False, width=2, tilt=0):
    ca, sa = math.cos(tilt), math.sin(tilt)
    for k in range(64):
        pts = []
        for a in [k*TAU/64, (k+1)*TAU/64]:
            u,v = r*math.cos(a),r*math.sin(a)*flat
            pts.append((x+u*ca-v*sa,y+u*sa+v*ca))
        color = spectral(k/64+phase*.18) if prismatic else col(science, 145+round(95*(.5+.5*math.sin(k*TAU/64-phase))))
        path(d, pts, color, width)


@lru_cache(maxsize=2048)
def singularity(radius, f, prismatic=False, flat=.72):
    """An opaque void, warped stars, and an accretion disc that passes in front."""
    size=radius*4+12;cx=cy=size//2;phase=f*TAU/8
    im=Image.new('RGBA',(size,size));pix=im.load()
    for y in range(size):
        for x in range(size):
            dx,dy=x-cx,(y-cy)/flat;r=math.hypot(dx,dy)/radius
            a=math.atan2(dy,dx)
            if r<.83:
                # True empty centre; the spiral brightens only as matter reaches its rim.
                arm=(.5+.5*math.cos(3*a+6*math.log(r+.06)-phase))**9
                edge=r**3
                pix[x,y]=(round(5+45*edge*arm),round(7+40*edge*arm),round(18+105*edge*arm),245)
            elif r<1.16:
                light=max(0,1-abs(r-.95)/.22)
                color=spectral(a/TAU+phase*.1)[:3] if prismatic else (130,180,255)
                blend=light**4*.8
                pix[x,y]=tuple(round(v+(255-v)*blend) for v in color)+(round(255*max(.12,light)),)
            elif r<1.52:
                pix[x,y]=(104,100,240,round(70*(1-(r-1.16)/.36)**2))
    d=ImageDraw.Draw(im)
    # Behind/foreground disk and its bent upper echo create lensing depth.
    for offset in [-2,0,2]:
        ellipse(d,cx,cy+offset,radius*1.55,0,phase,.17,prismatic,1, -.14)
    for k in range(32):
        a=k*TAU/32
        path(d, [p(cx,cy-radius*.16,radius*.96,a,.62),p(cx,cy-radius*.16,radius*1.04,a+.08,.62)], spectral(k/32+phase*.1,210) if prismatic else col(0,180))
    for k in range(12):
        t=(k*.618+f/8)%1;r=radius*(1.65-1.4*t);a=k*2.399+phase*.6+t*2
        x,y=p(cx,cy,r,a,flat)
        if r>radius*.55:
            path(d,[p(cx,cy,r+2,a-.06,flat),(x,y)],WHITE)
            if k%4==0:sparkle(d,x,y,spectral(k/12) if prismatic else col(0),1)
    return glow(im)


def paste(im, sprite, x, y):
    im.alpha_composite(sprite,(round(x-sprite.width/2),round(y-sprite.height/2)))


def crystal(d,x,y,r,f,science=2,angle=0):
    # Each faceted shard has a dark face, translucent green face and white spine.
    pts=[(0,-r),(r*.43,-r*.1),(r*.28,r*.65),(0,r),(-r*.38,r*.5),(-r*.45,-r*.08)]
    ca,sa=math.cos(angle),math.sin(angle)
    points=[(round(x+u*ca-v*sa),round(y+u*sa+v*ca)) for u,v in pts]
    d.polygon(points,fill=(12,67,66,225),outline=col(science))
    d.polygon([points[0],points[1],points[2],points[3]],fill=col(science,140+f*8))
    path(d,[points[0],(x,y),points[3]],WHITE,2 if r>12 else 1)
    path(d,[points[5],(x,y),points[1]],col(science,210))


def nebula(im,x,y,r,f,science=2):
    layer=Image.new('RGBA',im.size);d=ImageDraw.Draw(layer);phase=f*TAU/8
    for k in range(7):
        a=k*TAU/7+phase*.2;px,py=p(x,y,r*.6,a,.6)
        q=r*(.3+.07*math.sin(phase+k))
        d.ellipse((px-q,py-q,px+q,py+q),fill=col(science,45+k*5))
    im.alpha_composite(layer.filter(ImageFilter.GaussianBlur(2)))


def orrery(im,x,y,r,f,prismatic=False,variant=0):
    d=ImageDraw.Draw(im);phase=f*TAU/8
    # Rotating projected cube/tesseract, with filled brass plates and a stellar core.
    for n in range(2):
        points=[p(x,y,r*(1 if n==0 else .56),phase*.25+TAU*k/4+math.pi/4,.72) for k in range(4)]
        d.polygon(points,fill=(51,30,18,80 if n==0 else 175))
        path(d,points+[points[0]],col(1),2)
        if n==0:outer=points
        else:
            for a,b in zip(outer,points):path(d,[a,b],WHITE)
    for k in range(3):ellipse(d,x,y,r*(.72+k*.17),1,phase+k,.42,prismatic,1,k*TAU/3+.2)
    for k in range(5):
        a=phase+k*TAU/5;px,py=p(x,y,r*.88,a,.6)
        q=3+((k+variant)%3)
        d.ellipse((px-q,py-q,px+q,py+q),fill=(105,57,32,255),outline=col(1))
        d.arc((px-q,py-q,px+q,py+q),190,300,fill=WHITE,width=2)
        if k%2:sparkle(d,px,py,col(1),1)
    # Solid mechanical keystones make this a machine, not a wireframe diagram.
    if r>=18:
        for k in range(4):
            a=phase*.25+k*TAU/4+math.pi/4
            points=[p(x,y,r*s,a+da,.72) for s,da in [(1.08,-.1),(.82,-.17),(.75,.06),(1.04,.12)]]
            d.polygon(points,fill=(141,82,33,245),outline=col(1))
            path(d,[points[0],points[1]],WHITE,2)
    q=r*.19
    d.ellipse((x-q,y-q,x+q,y+q),fill=col(1),outline=WHITE,width=2)
    sparkle(d,x,y,WHITE,round(q+3))


def reactor(im,x,y,r,f,prismatic=False,variant=0):
    nebula(im,x,y,r*1.2,f)
    d=ImageDraw.Draw(im);phase=f*TAU/8
    ellipse(d,x,y,r,2,phase,.55,prismatic,2)
    for k in range(6):
        a=k*TAU/6+phase*.3
        px,py=p(x,y,r*.7,a,.68)
        crystal(d,px,py,r*(.26+.04*((k+variant)%3)),f,angle=a+math.pi/2)
        path(d,[(x,y),p(x,y,r*.36,a+.13,.68),(px,py)],col(2,170))
    crystal(d,x,y,r*.65,f)
    for k in range(16):
        a=k*2.399+phase*.5;rr=r*(.35+.065*((k+f)%12));px,py=p(x,y,rr,a,.7)
        sparkle(d,px,py,spectral(k/16,210) if prismatic else col(2,210),1)


def tunnel(im,x,y,r,f,science=0,prismatic=False):
    d=ImageDraw.Draw(im);phase=f*TAU/8
    # A portal throat recedes into a second plane, with a moving folded interior.
    for k in range(5,0,-1):
        depth=((k-1)/5+f/8)%1
        cx=x-depth*28;cy=y-depth*18;q=r*(1-depth*.75)
        d.ellipse((cx-q,cy-q*.48,cx+q,cy+q*.48),fill=(10,9,32,160))
        ellipse(d,cx,cy,q,science,phase+k*.3,.48,prismatic,1)
    paste(im,singularity(max(5,round(r*.5)),f,prismatic,.7),x,y)


def field(kind,tier,f,size=128):
    im=Image.new('RGBA',(size,size));x=y=size//2;r=[17,24,32,37][tier];phase=f*TAU/8
    d=ImageDraw.Draw(im);top=tier==3
    if kind in (12,18,20):
        if kind==18:
            tunnel(im,x+8,y+4,r,f,0,top)
            d=ImageDraw.Draw(im)
            for k in range(3):
                path(d,[p(x,y,r*(.2+j*.013),phase+k*TAU/3+j*.12,.65) for j in range(65)],col(0,180),2)
        else:
            paste(im,singularity(r,f,top,.67 if kind==12 else 1),x,y)
            if kind==20:
                ellipse(ImageDraw.Draw(im),x,y,r*1.28,0,phase,.55,top,2,phase*.12)
    elif kind in (1,13):
        tunnel(im,x+10,y+5,r,f,0,top)
        d=ImageDraw.Draw(im)
        if kind==1:
            for k in range(3):
                a=k*TAU/3-.3;path(d,[(x,y),p(x,y,r*1.34,a,.7)],col(k),2)
                sparkle(d,*p(x,y,r*1.34,a,.7),WHITE,2)
        else:
            for k in range(3):
                color=spectral(k/3+f/24) if top else col(0,230)
                path(d,[(12,y-12+k*12),(x-20,y-12+k*12),(x,y+(k-1)*3),(size-14,y+(k-1)*17)],color,2)
    elif kind==5:
        nebula(im,x,y,r,f,0);d=ImageDraw.Draw(im)
        d.ellipse((x-r,y-r*.78,x+r,y+r*.78),fill=(13,18,46,160))
        for k in range(3):
            rr=r*(.6+k*.24);ellipse(d,x,y,rr,0,phase/(k+1),.78,top,2)
            path(d,[(x,y),p(x,y,rr,phase/(k+1)-math.pi/2,.78)],WHITE)
            for j in range(12):sparkle(d,*p(x,y,rr,j*TAU/12,.78),col(0),1)
        paste(im,singularity(max(5,round(r*.25)),f,top),x,y)
    elif kind==45:
        orrery(im,x,y,r,f,top,kind);d=ImageDraw.Draw(im)
        path(d,[(x-r,y+r),(x,y),(x+r,y-r)],WHITE,2)
        path(d,[(x-r,y-r),(x,y),(x+r,y+r)],col(1),2)
    elif kind in (57,58):
        nebula(im,x,y,r,f);d=ImageDraw.Draw(im)
        if kind==57:
            for k in [-2,-1,0,1,2]:
                crystal(d,x+k*r*.35,y+abs(k)*6,r*(1-.24*abs(k))*(.83+.17*math.sin(phase+k)),f,angle=k*.22+.06*math.sin(phase))
        else:
            points=[p(x,y,r,phase*.12+k*TAU/6,.72) for k in range(6)]
            for k,(px,py) in enumerate(points):
                for tx,ty in points[k+1:]:path(d,[(px,py),(tx,ty)],col(2,100),1)
                crystal(d,px,py,r*.32,f,angle=k*TAU/6)
            crystal(d,x,y,r*.65,f)
        ellipse(d,x,y+12,r*1.12,2,phase,.4,top,2)
    elif kind==65:
        nebula(im,x,y,r*1.35,f);reactor(im,x,y,r*.66,f,top,kind)
        d=ImageDraw.Draw(im)
        for k in range(28):
            a=k*2.399+phase*.24;rr=r*(.42+((k*5+f)%19)/22)
            px,py=p(x,y,rr,a,.7);q=1+k%3
            d.ellipse((px-q,py-q,px+q,py+q),fill=col(2,185-k*3))
    else:
        reactor(im,x,y,r,f,top,kind)
        d=ImageDraw.Draw(im)
        for k in range(3):
            path(d,[(x-r,y+(k-1)*14),(x,y+(k-1)*4),(x+r,y+(k-1)*3)],col(2),2)
    # Baked emission only, one draw call for the complete scientific construction.
    return glow(im)


def skill_effect(skill,tier,f,skills):
    if skill in [1,5,12,13,18,20,45,57,58,65,66]:return field(skill,tier,f)
    im=Image.new('RGBA',(128,128));x=y=64;r=[15,22,29,35][tier]
    science=skills[skill]['science'];local=skill%25;role=skills[skill]['role'];phase=f*TAU/8;top=tier==3
    if science==0:
        if role in ('Move','Setup','Capstone'):tunnel(im,x+8,y+4,r,f,0,top)
        else:paste(im,singularity(max(8,round(r*.66)),f,top),x,y)
    elif science==1:orrery(im,x,y,r,f,top,local)
    else:reactor(im,x,y,r,f,top,local)
    d=ImageDraw.Draw(im)
    if role in ('Attack','Heavy'):
        for k in range(3):
            yy=y+(k-1)*9;xx=14+(f*7+k*4)%19
            path(d,[(xx,yy),(x+30,yy+(k-1)*3),(x+43,yy+(k-1)*5)],col(science,200),2)
        sparkle(d,x+39,y,WHITE,5+f%3)
    elif role=='Protect':
        pts=[(x,y-r*1.4),(x+r,y-r*.6),(x+r*.8,y+r*.6),(x,y+r*1.35),(x-r*.8,y+r*.6),(x-r,y-r*.6),(x,y-r*1.4)]
        path(d,pts,col(science),2)
        for k in range(4):path(d,[(x-r*.6+k*r*.4,y-r),(x-r*.6+k*r*.4,y+r)],col(science,70))
    elif role=='Move':
        for k in range(4):
            xx=x-35+k*18;path(d,[(xx,y+25),(xx+12,y+30),(xx,y+35)],col(science,150+k*20),2)
    elif role=='Observe':
        ellipse(d,x,y,r*1.3,science,phase,.7,top,1)
        sparkle(d,*p(x,y,r*1.3,phase+local*.2,.7),WHITE,3)
    elif role=='Capstone':
        for k in range(3):
            px,py=p(x,y,r*1.25,phase*.2+k*TAU/3,.75)
            if k==0:paste(im,singularity(7,f,top),px,py)
            elif k==1:orrery(im,px,py,8,f,top)
            else:reactor(im,px,py,8,f,top)
    # Recipe-specific satellites change topology/silhouette, rather than tiny ID bars.
    d=ImageDraw.Draw(im)
    for k in range(2+local%4):
        a=phase*.25+local*.37+k*TAU/(2+local%4)
        px,py=p(x,y,r*(1.15+.09*(local%3)),a,.75)
        if science==2:crystal(d,px,py,3+local%4,f,angle=a)
        elif science==1:
            pts=[p(px,py,3+local%3,a+j*TAU/3) for j in range(4)]
            d.polygon(pts,fill=(69,40,23,220));path(d,pts,WHITE)
        else:sparkle(d,px,py,col(0),2+local%3)
    return glow(im)


def packet(skill,tier,f,heading,skills):
    science=skills[skill]['science'];r=[5,7,9,11][tier]
    im=Image.new('RGBA',(64,64));d=ImageDraw.Draw(im);phase=f*TAU/8
    # Draw in a local horizontal frame, then bake all sixteen headings.
    for k in range(3):
        pts=[(5+j,32+(k-1)*4+math.sin(j*.18-phase+k)*2) for j in range(29)]
        path(d,pts,spectral(k/3+f/32,170) if tier==3 else col(science,125+k*40),2)
    if science==0:
        paste(im,singularity(r,f,tier==3,.82),38,32)
    elif science==1:
        orrery(im,38,32,r,f,tier==3,skill)
    else:reactor(im,38,32,r,f,tier==3,skill)
    d=ImageDraw.Draw(im)
    if skill==73:
        path(d,[(3,32),(58,32)],WHITE,2)
        for k in range(4):sparkle(d,10+k*12,32,col(2),2+(f+k)%2)
    elif skill==40:
        path(d,[(7,40),(19,35),(32,32)],col(1,220))
    elif skill==72:
        for yy in [24,40]:sparkle(d,22,yy,col(2),2)
    return glow(im).rotate(-heading*360/8,resample=Image.Resampling.NEAREST)


def equipment(science,rank,f,front=False,podium=4):
    im=Image.new('RGBA',(96,112));phase=f*TAU/8;r=[0,0,0,14,19,24,27,29][rank]
    if front:
        if rank<5:return im
        d=ImageDraw.Draw(im)
        ellipse(d,48,84,r,science,phase,.16,rank==7,1)
        for k in range(3):
            a=phase+k*TAU/3
            if math.sin(a)<0:continue
            x,y=p(48,73,r,a,.3)
            if k==0:paste(im,singularity(4+(rank==7),f,podium==1),x,y)
            elif k==1:orrery(im,x,y,5,f,podium==1)
            else:reactor(im,x,y,5,f,podium==1)
        return im
    if science==0:paste(im,singularity(r,f,rank==7,.7),48,45)
    elif science==1:orrery(im,48,45,r,f,rank==7)
    else:reactor(im,48,45,r,f,rank==7)
    d=ImageDraw.Draw(im)
    if rank==7:
        # Three star-filled coat panels are the same body's folded laboratory.
        for k in range(3):
            x,y=p(48,76,23,k*TAU/3+phase*.22+science*.3,.35)
            pts=[(x-6,y-3),(x+6,y-2),(x+8,y+13),(x,y+18),(x-8,y+13),(x-6,y-3)]
            d.polygon(pts,fill=(11,16,42,255),outline=col(k))
            for n in range(5):d.point((x-4+(n*5+f)%9,y+n*3),fill=spectral(n/5+f/32))
            path(d,[(x,y-1),(x+2*math.sin(phase),y+8),(x,y+15)],WHITE)
        if podium==1:
            # A three-body crown, rather than a copy of Isliid's sword dispenser.
            for k in range(3):
                x,y=p(48,22,18,phase*.25+k*TAU/3,.45)
                if k==0:paste(im,singularity(6,f,True),x,y)
                elif k==1:orrery(im,x,y,7,f,True)
                else:reactor(im,x,y,7,f,True)
            path(d,[(28,25),(48,8),(68,25),(28,25)],WHITE)
        elif podium==2:
            for x in [34,62]:paste(im,singularity(6,f,True),x,22)
        elif podium==3:
            for k in range(3):sparkle(d,*p(48,22,19,k*TAU/3+phase*.25,.4),spectral(k/3+f/32),3)
    return glow(im)


def completion(tier,podium,f):
    im=Image.new('RGBA',(128,128));r=[15,22,30,37][tier]
    # Eight real phases: seed -> expansion -> unified structure -> collapse.
    life=[.35,.62,.85,1,1,.85,.6,.3][f];r=round(r*life)
    paste(im,singularity(max(5,r),f,tier==3,.68),64,64)
    orrery(im,64,64,r*.85,f,tier==3)
    d=ImageDraw.Draw(im)
    for k in range(3):
        x,y=p(64,64,r*1.24,k*TAU/3+f*TAU/48,.74)
        if k==2:reactor(im,x,y,max(5,r*.27),f,tier==3)
        else:sparkle(d,x,y,spectral(k/3+f/32),3)
    if tier==3 and podium==1:
        pts=[p(64,64,r*1.25,k*TAU/3-math.pi/2,.85) for k in range(4)]
        path(d,pts,WHITE,3)
        path(d,[(x+5,y+7) for x,y in pts],col(1),2)
        for x,y in pts[:-1]:path(d,[(x,y),(x+5,y+7)],col(2))
    elif tier==3 and podium==2:
        ellipse(d,64,64,r*1.4,0,f*TAU/8,.4,True,2)
    elif tier==3 and podium==3:
        for k in range(3):crystal(d,*p(64,64,r,k*TAU/3),max(3,r*.18),f)
    return glow(im)
