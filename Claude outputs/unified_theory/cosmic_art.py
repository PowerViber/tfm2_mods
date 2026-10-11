"""Baked eight-frame celestial sprites; no particles or shaders run in the game.

Einstein connects quantum light with stellar creation. Newton unfolds impossible
mathematical solids. Curie assembles molecular, living matter. All functions return centred, original
pixel sprites; a fixed seed and eight discrete phases make builds reproducible.
"""
import math
import json
from functools import lru_cache
from pathlib import Path

from PIL import Image, ImageDraw, ImageFilter, ImageChops

TAU = math.tau
COLORS = [(99, 191, 255), (255, 201, 107), (99, 225, 189)]
WHITE = (238, 250, 255, 255)


def col(science, alpha=255):
    return COLORS[science] + (alpha,)


def stellar_light(t, alpha=255):
    """Ivory light on a cold surface, with brightness movement rather than hue cycling."""
    light=.5+.5*math.cos(t*TAU)
    return tuple(round(a+(b-a)*light) for a,b in zip((137,157,181),WHITE[:3]))+(alpha,)


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


def ellipse(d, x, y, r, science, phase, flat=.45, ivory=False, width=2, tilt=0):
    ca, sa = math.cos(tilt), math.sin(tilt)
    for k in range(64):
        pts = []
        for a in [k*TAU/64, (k+1)*TAU/64]:
            u,v = r*math.cos(a),r*math.sin(a)*flat
            pts.append((x+u*ca-v*sa,y+u*sa+v*ca))
        color = stellar_light(k/64+phase*.18) if ivory else col(science, 145+round(95*(.5+.5*math.sin(k*TAU/64-phase))))
        path(d, pts, color, width)


@lru_cache(maxsize=2048)
def singularity(radius, f, ivory=False, flat=.72):
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
                color=stellar_light(a/TAU+phase*.1)[:3] if ivory else (130,180,255)
                blend=light**4*.8
                pix[x,y]=tuple(round(v+(255-v)*blend) for v in color)+(round(255*max(.12,light)),)
            elif r<1.52:
                pix[x,y]=(104,100,240,round(70*(1-(r-1.16)/.36)**2))
    d=ImageDraw.Draw(im)
    # Behind/foreground disk and its bent upper echo create lensing depth.
    for offset in [-2,0,2]:
        ellipse(d,cx,cy+offset,radius*1.55,0,phase,.17,ivory,1, -.14)
    for k in range(32):
        a=k*TAU/32
        path(d, [p(cx,cy-radius*.16,radius*.96,a,.62),p(cx,cy-radius*.16,radius*1.04,a+.08,.62)], stellar_light(k/32+phase*.1,210) if ivory else col(0,180))
    for k in range(12):
        t=(k*.618+f/8)%1;r=radius*(1.65-1.4*t);a=k*2.399+phase*.6+t*2
        x,y=p(cx,cy,r,a,flat)
        if r>radius*.55:
            path(d,[p(cx,cy,r+2,a-.06,flat),(x,y)],WHITE)
            if k%4==0:sparkle(d,x,y,stellar_light(k/12) if ivory else col(0),1)
    return glow(im)


def paste(im, sprite, x, y):
    im.alpha_composite(sprite,(round(x-sprite.width/2),round(y-sprite.height/2)))


@lru_cache(maxsize=24)
def cosmic_material(science,f):
    """Moving galaxy, engraved brass, or luminous mineral, baked onto planes."""
    im=Image.new('RGBA',(128,128));pix=im.load();phase=f*TAU/8
    for y in range(128):
        for x in range(128):
            u,v=x-64,y-64
            if science==0:
                a=math.atan2(v,u);r=math.hypot(u,v)
                mist=(.5+.5*math.sin(a*3-r*.17+phase))**4
                pix[x,y]=(round(14+35*mist),round(24+62*mist),round(55+98*mist),230)
            elif science==1:
                ridge=(.5+.5*math.sin((u+v*.5)*.3-phase*.25))**6
                pix[x,y]=(round(86+87*ridge),round(48+69*ridge),round(31+33*ridge),240)
            else:
                vein=(.5+.5*math.sin((u-v*.7)*.22+phase))**6
                pix[x,y]=(round(15+25*vein),round(73+76*vein),round(77+57*vein),230)
    d=ImageDraw.Draw(im)
    for k in range(45):
        x=(k*47+11)%128;y=(k*31+17)%128
        d.point((x,y),fill=col(science,150+((k+f)%4)*30))
        if (k+f)%9==0:sparkle(d,x,y,WHITE,1)
    return im


def material_plane(im,points,science,f,tint=None):
    points=[(round(x),round(y)) for x,y in points]
    mask=Image.new('L',im.size);ImageDraw.Draw(mask).polygon(points,fill=255)
    left,top=64-im.width//2,64-im.height//2
    texture=cosmic_material(science,f).crop((left,top,left+im.width,top+im.height))
    if tint is not None:texture=Image.blend(texture,Image.new('RGBA',im.size,tint),.2)
    im.alpha_composite(Image.composite(texture,Image.new('RGBA',im.size),mask))


def quantum(d,x,y,r,f):
    """Light packets and possible paths, without an opaque gravitational centre."""
    f%=8;phase=f*TAU/8
    for k in range(3):
        pts=[(x-r+j*r/8,y+math.sin(j*.35+phase+k)*r*.3+(k-1)*r*.2) for j in range(17)]
        path(d,pts,col(0,110+k*45))
    px=x-r+((f+1)/9)*2*r
    sparkle(d,px,y+math.sin(phase)*r*.15,WHITE,max(1,round(r*.2)))


def cosmic_shell(im,x,y,r,f,genesis=False):
    """An OPEN section of star-filled space; quantum and cosmic scales coincide."""
    d=ImageDraw.Draw(im);phase=f*TAU/8
    arc=[-2.7+j*4.25/24 for j in range(25)]
    outer=[p(x,y,r,a,.85) for a in arc]
    inner=[p(x,y,r*.76,a,.85) for a in reversed(arc)]
    material_plane(im,outer+inner,0,f)
    path(d,outer,col(0),2);path(d,list(reversed(inner)),WHITE)
    # The creation web shares the same layout as the tiny quantum paths below.
    points=[(x+u*r,y+v*r) for u,v in [(-.68,-.3),(-.28,-.64),(.2,-.52),(.65,-.12),(.4,.44)]]
    expansion=[.12,.3,.58,.85,1,1,.55,.2][f] if genesis else .72
    sx,sy=x-r*.56,y+r*.78
    for k,(px,py) in enumerate(points):
        qx=sx+(px-sx)*expansion;qy=sy+(py-sy)*expansion
        if k:
            ax,ay=points[k-1]
            path(d,[(sx+(ax-sx)*expansion,sy+(ay-sy)*expansion),(qx,qy)],col(0,150))
        sparkle(d,qx,qy,WHITE if genesis and f==5 else col(0),1+(k+f)%3)
    quantum(d,sx,sy,r*.22,f)
    if genesis and f in (4,5):
        # A single shared pattern joins the smallest and largest observations.
        path(d,[(sx,sy),(x-r*.25,y+r*.1),(x+r*.2,y-r*.52)],WHITE)
        if f==5:sparkle(d,x+r*.2,y-r*.52,WHITE,4)
    for k in range(7):
        a=arc[(k*3+2)%25];px,py=p(x,y,r*.87,a,.85)
        d.point((px,py),fill=stellar_light((k+f)/11))


def math_object(im,x,y,r,f,dimension=3):
    """Solid faces projected through a fourth axis, rather than planetary rings."""
    f%=8;d=ImageDraw.Draw(im);phase=f*TAU/8
    # A 4D x/w rotation changes which surface is inside, with no 3D yaw.
    vertices=[]
    for n in range(16 if dimension>=3 else 8):
        u,v,z=[1 if n&(1<<k) else -1 for k in range(3)]
        w=(1 if n&8 else -1) if dimension>=3 else 0
        q=u*math.cos(phase)+w*math.sin(phase)
        depth=w*math.cos(phase)-u*math.sin(phase)
        scale=1/(2.7-depth*.55)
        vertices.append((round(x+(q+z*.55)*r*scale),round(y+(v-z*.42)*r*scale)))
    faces=[(0,1,3,2),(0,2,6,4),(0,1,5,4)]
    if dimension>=3:faces+=[(8,9,11,10),(8,10,14,12)]
    for k,face in enumerate(faces):
        pts=[vertices[n] for n in face]
        d.polygon(pts,fill=[(61,43,48,245),(130,108,83,250),(220,210,180,245)][(k+f//4)%3],outline=col(1))
        path(d,[pts[0],pts[1]],WHITE,2)
    for n,a in enumerate(vertices):
        for axis in range(4 if dimension>=3 else 3):
            nxt=n^(1<<axis)
            if n<nxt and nxt<len(vertices):path(d,[a,vertices[nxt]],col(1,185))
    # A detached face folds edge-on, then emerges on the object's other side.
    offset=[-.8,-.58,-.3,0,.25,.48,.3,-.4][f]*r
    width=[.4,.34,.18,.04,.2,.45,.3,.4][f]*r
    pts=[(x+offset-width,y-r*.55),(x+offset+width,y-r*.65),(x+offset+width,y+r*.1),(x+offset-width,y+r*.2)]
    d.polygon([(round(a),round(b)) for a,b in pts],fill=(181,156,114,230),outline=WHITE)
    if dimension>=3:
        # The shadow can describe a logically different object from the solid.
        shadow=[p(x+3,y+r*.85,r*.65,k*TAU/(3 if f==5 else 4),.22) for k in range(4 if f==5 else 5)]
        path(d,shadow,col(1,90))
    sparkle(d,*vertices[(f*3)%len(vertices)],WHITE,1)


def infinite_stairs(d,x,y,r,f,depth=4):
    for k in range(depth):
        rr=r*(.58**k);lift=(f%4)*rr*.035
        pts=[(x-rr,y+rr*.6),(x-rr,y-rr*.4-lift),(x+rr*.55,y-rr*.4-lift),(x+rr*.55,y+rr*.2),(x-rr*.2,y+rr*.2)]
        path(d,pts,col(1,235-k*25),2 if k==0 else 1)
        path(d,[(x-rr,y-rr*.4-lift),(x-rr+rr*.2,y-rr*.58-lift),(x+rr*.75,y-rr*.58-lift)],WHITE)


def unprovable_shape(im,x,y,r,f):
    """Cyclic solid occlusion: A covers B, B covers C, C covers A.

    A dimensional face passes through the opening and emerges inside-out. The
    contradiction is visible in a still sprite; movement reveals the extra axis.
    """
    d=ImageDraw.Draw(im)
    vertices=[(x,y-r),(x+r*.9,y+r*.58),(x-r*.9,y+r*.58)]
    shades=[(224,211,176,255),(150,125,88,255),(94,71,63,255)]
    def beam(a,b,k,short=False):
        dx,dy=b[0]-a[0],b[1]-a[1];length=math.hypot(dx,dy)
        nx,ny=-dy/length*r*.12,dx/length*r*.12
        if short:b=(a[0]+dx*.24,a[1]+dy*.24)
        pts=[(a[0]-nx,a[1]-ny),(b[0]-nx,b[1]-ny),(b[0]+nx,b[1]+ny),(a[0]+nx,a[1]+ny)]
        pts=[(round(u),round(v)) for u,v in pts]
        d.polygon(pts,fill=shades[k],outline=col(1))
        path(d,[pts[0],pts[1]],WHITE,2)
        path(d,[(a[0],a[1]),(b[0],b[1])],col(1,160))
    for k in range(3):beam(vertices[k],vertices[(k+1)%3],k)
    for k in range(3):beam(vertices[k],vertices[(k+1)%3],k,True)
    # Opening -> inner solid -> edge-on fold -> external surface -> restored.
    scale=[.25,.38,.58,.12,.72,.83,.42,.25][f]
    flip=-1 if f in (4,5) else 1
    pts=[(x+u*r*scale,y+v*r*scale*flip) for u,v in [(-.55,.45),(0,-.6),(.55,.45),(0,.2)]]
    d.polygon([(round(u),round(v)) for u,v in pts],fill=shades[0 if f>=4 else 2],outline=WHITE)
    path(d,[pts[0],pts[2]],col(1))
    # One corner remains shifted after the proof returns; no spinning halo.
    vx,vy=vertices[(f//3)%3]
    sparkle(d,vx+(2 if f==7 else 0),vy,WHITE,1)


def molecule(d,x,y,r,f,count=5):
    points=[p(x,y,r,k*TAU/count+f*TAU/64,.7) for k in range(count)]
    path(d,points+[points[0]],col(2,185))
    for k,(px,py) in enumerate(points):
        q=2 if r>=8 else 1
        d.ellipse((px-q,py-q,px+q,py+q),fill=WHITE if k==f%count else col(2),outline=(30,78,76,255))


def helix(d,x,y,r,f):
    phase=f*TAU/8;strands=[]
    for sign in [-1,1]:
        pts=[(x+sign*math.sin(j*.52-phase)*r*.38,y-r+j*r/8) for j in range(17)]
        strands.append(pts);path(d,pts,WHITE if sign==1 else col(2),2 if r>15 else 1)
    for j in range(0,17,2):
        path(d,[strands[0][j],strands[1][j]],col(2,200))
        if (j+f)%4==0:d.ellipse((strands[0][j][0]-1,strands[0][j][1]-1,strands[0][j][0]+1,strands[0][j][1]+1),fill=WHITE)


def living_cell(d,x,y,r,f):
    phase=f*TAU/8
    pts=[p(x,y,r*(.94+.06*math.sin(k*3+phase)),k*TAU/24,.76) for k in range(25)]
    d.polygon(pts,fill=(29,91,83,155),outline=(189,233,206,235))
    path(d,[p(x,y,r*.78,k*TAU/24,.76) for k in range(25)],col(2,180))
    q=max(2,r*.18);nx=x+math.sin(phase)*r*.22
    d.ellipse((nx-q,y-q,nx+q,y+q),fill=(111,198,145,235),outline=WHITE)
    for k in range(3):
        px,py=p(x,y,r*.53,phase*.25+k*TAU/3,.6)
        path(d,[(nx,y),(px,py)],col(2,170));d.point((round(px),round(py)),fill=WHITE)


def genesis_vessel(im,x,y,r,f,peak=False,level=3):
    """A mineral becomes bonded matter, a membrane and a reaching alien organism."""
    d=ImageDraw.Draw(im);phase=f*TAU/8
    pts=[(x-r*.58,y-r),(x+r*.35,y-r*.9),(x+r*.72,y-r*.24),(x+r*.56,y+r*.82),(x-r*.34,y+r),(x-r*.72,y+r*.2)]
    pts=[(round(a),round(b)) for a,b in pts]
    d.polygon(pts,fill=(12,34,37,70),outline=(203,229,215,235))
    path(d,[pts[0],pts[1]],WHITE,2);path(d,[pts[3],pts[4],pts[5]],col(2),2)
    # Stable cage, changing life inside: the #1 silhouette never blinks away.
    if peak:
        if f in (0,7):
            crystal(d,x,y+r*.54,r*.2,f)
            if f==7:sparkle(d,x,y+r*.42,col(2),2)
        elif f==1:molecule(d,x,y+r*.24,r*.4,f,6)
        elif f==2:helix(d,x,y,r*.65,f)
        elif f==3:living_cell(d,x,y,r*.65,f);helix(d,x,y,r*.42,f)
        else:
            growth=[0,0,0,0,1,1,.55,0][f]
            stem=[(x,y+r*.62),(x-r*.12,y+r*.13),(x+r*.1,y-r*.2)]
            path(d,stem,(190,235,200,245),2)
            living_cell(d,x+r*.06,y-r*.1,r*.35*growth+2,f)
            for k in [-1,1]:
                tip=(x+k*r*.5*growth,y-r*.5*growth)
                path(d,[(x,y),(x+k*r*.26,y-r*.27),tip],col(2),2)
                living_cell(d,*tip,r*.17*growth+1,f)
            if f==5:
                # The left sensory branch reaches toward Curie's visible hand.
                path(d,[(x-r*.25,y-r*.15),(x-r*.55,y+r*.35),(x-r*.4,y+r*.65)],WHITE,2)
                sparkle(d,x-r*.4,y+r*.65,col(2),2)
    elif level<=1:molecule(d,x,y,r*.48,f,4+level)
    elif level==2:helix(d,x,y,r*.62,f)
    else:
        living_cell(d,x,y,r*.6,f)
        if level>=4:helix(d,x,y,r*.52,f)
        if level>=5:
            for k in [-1,1]:living_cell(d,x+k*r*.42,y+r*.45,r*.21,f+k)
    # Atomic observations move only within the one baked vessel animation.
    px,py=p(x,y,r*.9,phase,.68);d.point((px,py),fill=col(2))


def architecture(im,x,y,r,science,tier,f):
    """Separate subject silhouettes surround the skill's own scientific action."""
    if not tier:return
    d=ImageDraw.Draw(im)
    if science==0:
        quantum(d,x,y+r*1.15,r*.35,f)
        if tier>=2:cosmic_shell(im,x,y,r*1.28,f)
    elif science==1:
        for k in [-1,1]:
            math_object(im,x+k*r*.98,y-r*.65,r*(.18+tier*.07),f,dimension=tier)
        if tier>=2:infinite_stairs(d,x,y+r*.5,r*1.2,f,tier+1)
    else:
        for k in [-1,1]:
            px,py=x+k*r*.98,y+r*.55
            molecule(d,px,py,4+tier,f+k,4+tier)
            if tier>=2:living_cell(d,px,y-r*.7,5+tier*2,f+k)
        if tier==3:helix(d,x+r*1.24,y,r*.8,f)


def experiment(im,skill,tier,f,science):
    """A scientific action for every activation; coordinates share a 128px cap."""
    d=ImageDraw.Draw(im);x=y=64;r=[17,23,29,34][tier];phase=f*TAU/8
    local=skill%25;c=col(science);dark=[(12,19,48,240),(60,32,28,240),(13,53,56,240)][science]
    def poly(pts,fill=dark,outline=c):
        points=[(round(x+u*r),round(y+v*r)) for u,v in pts]
        material_plane(im,points,science,f,fill)
        d.polygon(points,outline=outline)
    def line(pts,color=c,width=2):path(d,[(x+u*r,y+v*r) for u,v in pts],color,width)
    def star(u,v,size=3,color=WHITE):sparkle(d,x+u*r,y+v*r,color,size)
    def clock(u,v,offset=0):
        cx,cy=x+u*r,y+v*r;rr=r*.45
        ellipse(d,cx,cy,rr,science,phase+offset,.8,False,2)
        path(d,[(cx,cy),p(cx,cy,rr*.8,phase+offset,.8)],WHITE)
    def node(u,v,size=.18):
        cx,cy=x+u*r,y+v*r
        if science==2:molecule(d,cx,cy,r*size,f,4)
        else:
            q=r*size;d.ellipse((cx-q,cy-q,cx+q,cy+q),fill=dark,outline=c,width=2)
            sparkle(d,cx,cy,WHITE,1)
    def arrow(a,b,color=c):
        line([a,b],color);dx=b[0]-a[0];dy=b[1]-a[1];length=math.hypot(dx,dy)
        if length:
            ux,uy=dx/length,dy/length
            line([(b[0]-.24*ux+.16*uy,b[1]-.24*uy-.16*ux),b,
                  (b[0]-.24*ux-.16*uy,b[1]-.24*uy+.16*ux)],color)
    if science==0:
        if local==0:
            poly([(-.4,-.8),(.15,0),(-.4,.8)],(35,75,115,235),WHITE)
            line([(-1.2,0),(-.15,0),(.3,0),(1.35,0)],WHITE,3)
            for k in [-1,1]:line([(-.15,0),(1.2,k*.6)],stellar_light(.55+k*.1+f/80),2)
            star(.8+f/30,0,4)
        elif local==2:clock(-.65,0);clock(.65,0,.8);line([(-.2,0),(.2,0)],WHITE)
        elif local==3:
            poly([(-1,-.8),(-.1,-.65),(-.1,.7),(-1,.85)])
            poly([(.1,-.7),(1,-.5),(1,.85),(.1,.7)])
            arrow((-.9,.2+math.sin(phase)*.15),(.8,.2),WHITE)
        elif local==4:
            clock(0,0);ellipse(d,x,y,r,0,phase*.25,.72,False,2);star(0,-1)
        elif local==6:
            gap=.3+.2*math.sin(phase)
            for u in [-gap,gap]:poly([(u-.1,-1),(u+.1,-.8),(u+.1,.9),(u-.1,1)])
            arrow((-1,0),(-gap,0));arrow((1,0),(gap,0))
        elif local==7:tunnel(im,x+10,y,r,f,0,tier==3);star(.6,0,4)
        elif local==8:
            clock(-.7,0);clock(.7,0)
            line([(-.7,-.55),(.7,-.55)],WHITE);star(0,-.55,2+f%3)
        elif local==9:
            poly([(0,-1.2),(-.8,-.15),(.8,-.15)],(28,48,97,140))
            poly([(0,1.2),(-.8,.15),(.8,.15)],(28,48,97,140))
            line([(-1,0),(1,0)],WHITE);star(0,0,4)
        elif local==10:
            for u in [-.8,.8]:quantum(d,x+u*r,y,r*.25,f)
            line([(-.65,0),(-.2,-.3),(.2,.3),(.65,0)],WHITE)
        elif local==11:
            for k in range(5):
                line([(-1.2+j*.12,(k-2)*.26+.35*math.sin(j*.2+phase*.2)) for j in range(21)],col(0,130),1)
            line([(-1.2,.7),(-.65,.1),(0,-.35),(.7,-.1),(1.2,.4)],WHITE)
            star(-1+f*.28,-.35*math.sin(f*math.pi/7),3)
        elif local in (14,15):
            shift=(235,102,149,255) if local==14 else (144,200,255,255)
            wave=1.6 if local==14 else 3.2
            line([(-1.2+j*.08,.32*math.sin(j*wave*.22-phase)) for j in range(31)],shift,3)
            for u in [-.5,.5]:poly([(u,-.8),(u+.15,-.5),(u+.15,.5),(u,.8)],outline=shift)
        elif local==16:
            for k in range(4):
                u=-.8+k*.42+(f%4)*.05
                line([(u-.25,-.8),(u,.0),(u-.25,.8)],WHITE if k==3 else c)
            arrow((-.8,0),(1.15,0))
        elif local==17:
            for k in range(4):
                a=k*TAU/4+phase*.3;u,v=math.cos(a)*.8,math.sin(a)*.8
                poly([(u-.2,v-.2),(u+.2,v-.2),(u+.2,v+.2),(u-.2,v+.2)])
                line([(u,v),(0,0)],WHITE,1)
            star(0,0,5+f%3)
        elif local==19:
            gap=.16+.18*math.sin(phase)
            poly([(-1,-1),(-gap,-.8),(-gap,.8),(-1,1)])
            poly([(gap,-.8),(1,-1),(1,1),(gap,.8)])
            line([(0,-1.2),(0,1.2)],WHITE,3)
            for k in range(4):star(0,-.8+k*.5,2+(f+k)%2)
        elif local==21:
            poly([(0,-1.1),(.95,-.5),(.75,.6),(0,1.1),(-.75,.6),(-.95,-.5)])
            line([(-.85,-.4),(0,0),(.85,-.4),(0,.95),(-.85,-.4)],WHITE)
            clock(0,0)
        elif local==22:
            for k in range(4):line([(-1,-.8+k*.4),(0,-.55+k*.3),(1,-.8+k*.4)],col(0,170),1)
            arrow((-.7,-1),(-.7,.65),WHITE)
            quantum(d,x+r*.4,y,r*.4,f)
        elif local==23:
            poly([(-1,0),(-.4,-.5),(.4,-.5),(1,0),(.4,.5),(-.4,.5)])
            clock(0,0,.4);line([(1,0),(1.35,-.6)],WHITE);star(1.35,-.6,3)
        elif local==24:
            cosmic_shell(im,x,y,r*1.2,f,genesis=True)
    elif science==1:
        if local==0:
            for a,b in [((-1,0),(1,0)),((0,1),(0,-1)),((-.7,.7),(.7,-.7))]:arrow(a,b)
            node(0,0,.28)
        elif local==1:
            poly([(-.7,.65),(-.7,-.35),(.7,-.35),(.7,.65)],(45,28,28,200))
            arrow((-.7,.6),(.8,-.75),WHITE);node(.8,-.75)
        elif local==2:
            node(-.8,0,.3)
            for v in [-.8,0,.8]:line([(-.5,0),(.1,v),(.8,v)],WHITE);node(.8,v)
        elif local==3:
            arrow((-.9,.7),(-.9,-.7));arrow((-.9,-.7),(.8,-.7));arrow((-.9,.7),(.8,-.7),WHITE)
        elif local==4:
            arrow((-.9,.5),(.9,-.5));arrow((-.9,-.5),(.9,-.5),WHITE)
            line([(.15,-.5),(.15,.15)],col(1,130),1);star(.9,-.5,4)
        elif local==5:
            arrow((-1,0),(1,0));arrow((0,1),(0,-1));poly([(-.35,.35),(.35,.35),(.35,-.35),(-.35,-.35)],outline=WHITE)
        elif local in (6,7,10):
            for k in range(3):
                u=-1+k*.55+math.sin(phase)*.12
                poly([(u,-.4),(u+.35,-.4),(u+.35,.4),(u,.4)])
            arrow((-.75,.7),(1,.7),WHITE)
            if local==7:arrow((-.5,-.65),(1,-.65),WHITE)
            if local==10:line([(.8,-.7),(1.1,0),(.8,.7)],WHITE,3)
        elif local==8:
            node(0,0,.3);arrow((-.3,0),(-1,0),WHITE);arrow((.3,0),(1,0),WHITE)
        elif local==9:
            for k in range(3):
                u=-1+k*.4
                poly([(u,-.55),(u+.35,-.35),(u+.35,.35),(u,.55)])
            arrow((-.2,0),(1.1,0),WHITE);star(1.1,0,6)
        elif local in (11,12):
            sep=.35+abs(math.sin(phase))*.45
            node(-sep,0,.3);node(sep,0,.3)
            arrow((-1,-.6),(0,-.6),WHITE)
            arrow((1,.6),(0,.6) if local==11 else (1.3,.6),WHITE)
            if local==11:poly([(-.3,-.3),(.3,-.3),(.3,.3),(-.3,.3)],outline=WHITE)
        elif local in (13,14):
            ellipse(d,x,y,r,1,phase,.68,False,2);node(0,0,.25)
            u,v=math.cos(phase),math.sin(phase)*.68;node(u,v,.2)
            if local==14:arrow((u,v),(u+math.sin(phase)*.45,v-math.cos(phase)*.4),WHITE)
            else:line([(0,0),(u,v)],WHITE,1)
        elif local==15:
            curve=[(-1+j*.1,-.8+(.1*j-1)**2*1.25) for j in range(21)]
            line(curve,WHITE);star(*curve[(f*3)%21],4)
            for k in range(4):line([(-1+k*.5,.95),(-1+k*.5,.8)],c,1)
        elif local==16:
            poly([(-.6,-1),(.6,-.7),(.8,.7),(0,1),(-.8,.7)],outline=WHITE)
            line([(-1,-.6),(0,0),(-1,.6)],c);star(0,0,4+f%2)
        elif local in (17,23):
            line([(-1,.8),(-.6,.35),(0,-.2),(.7,-.55),(1,-.5)],WHITE)
            arrow((-.6,.4),(.7,-.7));node(.35,-.5)
            if local==23:arrow((0,.5),(.7,.3),WHITE)
        elif local==18:
            poly([(-1,.75),(-1,-.15),(-.4,-.65),(.2,-.4),(.9,-.75),(.9,.75)],(77,48,21,190))
            for k in range(6):line([(-.9+k*.3,.7),(-.9+k*.3,-.2-k*.07)],col(1,130),1)
            star(-.7+f*.2,-.45,3)
        elif local==19:
            for k in range(3):ellipse(d,x,y+k*4,r*(1-k*.23),1,phase,.35,False,1)
            arrow((-.9,-.7),(.2,.45),WHITE);node(.2,.45)
        elif local==21:
            infinite_stairs(d,x,y,r,f,3+tier)
        elif local==22:
            poly([(-.9,-.7),(.9,-.7),(.9,.7),(-.9,.7)],(45,28,28,210))
            for k in range(5):line([(-.75+k*.35,-.7),(-.75+k*.35,.7)],col(1,110),1)
            arrow((-.7,-.4),(.4,-.4),WHITE);arrow((.4,-.4),(-.3,.3),WHITE)
        elif local==24:
            math_object(im,x,y,r*1.4,f,dimension=1+tier)
            infinite_stairs(d,x,y+r*.6,r*.55,f,2+tier)
    else:
        if local==0:
            for k in range(5):node(-.8+k*.4,-.4+math.sin(phase+k)*.2,.12)
            poly([(-.5,.2),(.5,.2),(.35,.9),(-.35,.9)],outline=WHITE)
            line([(-.5,.2),(.5,.2)],WHITE)
        elif local==1:
            for k in range(3):
                u=-.75+k*.75
                poly([(u-.2,-.7),(u+.2,-.7),(u+.32,.7),(u-.32,.7)])
                node(u,.35,.16)
            line([(-.75,-.9),(0,-1),(.75,-.9)],WHITE)
        elif local==2:
            poly([(-.5,0),(.5,0),(.35,.85),(-.35,.85)],outline=WHITE)
            node(0,-.85+f*.08,.14);line([(-.65,.1),(.65,.1)],c)
        elif local in (3,17):
            poly([(0,-1.1),(.8,-.6),(.8,.6),(0,1.1),(-.8,.6),(-.8,-.6)],outline=WHITE)
            for k in range(5):node(-.6+k*.3,math.sin(phase+k)*.6,.1)
            if local==17:line([(-.1,-1),(-.1,1)],WHITE);line([(.1,-1),(.1,1)],WHITE)
        elif local in (4,5):
            for k in range(5):
                u=-1+k*.42;v=math.sin(phase+k)*.23
                crystal(d,x+u*r,y+v*r,r*(.15+k*.035),f,angle=math.pi/2)
            line([(-1,0),(1,0)],WHITE if local==4 else c,3)
            if local==5:ellipse(d,x,y,r,2,phase,.65,False,2)
        elif local==6:
            node(-.8,0,.25);node(.8,0,.25);arrow((-.55,0),(0,0));arrow((.55,0),(0,0))
            crystal(d,x,y,r*.45,f)
        elif local in (9,10):
            crystal(d,x,y,r*.7,f)
            for k in range(5):
                a=k*TAU/5+phase*.2;u,v=math.cos(a),math.sin(a)*.8
                if local==9:arrow((u*.5,v*.5),(u,v),WHITE)
                else:arrow((u,v),(u*.5,v*.5),WHITE)
        elif local==11:
            points=[(-.8,.6),(0,-.75),(.8,.6)]
            line(points+[points[0]],WHITE)
            for u,v in points:node(u,v,.23)
            living_cell(d,x,y,r*.4,f)
        elif local==12:
            for k in range(6):
                a=k*TAU/6;u,v=math.cos(a),math.sin(a)*.8
                crystal(d,x+u*r,y+v*r,r*.22,f,angle=a)
            line([(-1,-.7),(1,.7)],WHITE,3);line([(-1,.7),(1,-.7)],WHITE,3)
        elif local==13:
            reactor(im,x,y,r*.75,f,tier==3,local)
            for k in range(8):
                a=k*TAU/8+phase*.2
                arrow((math.cos(a)*.5,math.sin(a)*.4),(math.cos(a)*1.25,math.sin(a)),WHITE)
        elif local==14:
            for k in range(3):
                u=-.8+k*.8;node(u,math.sin(phase+k)*.25,.2)
                if k<2:arrow((u+.2,0),(u+.6,0),WHITE)
        elif local==18:
            ellipse(d,x,y,r,2,phase,.68,False,1)
            star(math.cos(phase),math.sin(phase)*.68,5,c)
            line([(-.5,.9),(.7,-.6)],WHITE,1)
        elif local==19:
            molecule(d,x,y,r*.6,f,6)
            for k in range(3):node(math.cos(phase+k*TAU/3),math.sin(phase+k*TAU/3)*.8,.2)
            star(0,0,4)
        elif local==20:
            for k in range(3):
                u=-.8+k*.8;crystal(d,x+u*r,y,r*(.5-k*.12),f)
            line([(-1,-.75),(1,-.75)],WHITE);clock(0,.85)
        elif local==21:
            crystal(d,x,y,r,f,angle=math.pi/2)
            line([(-1.2,0),(1.2,0)],WHITE,3);star(1.2,0,5)
        elif local==22:
            for v in [-.65,0,.65]:
                line([(-1,0),(.25,v*.5),(1.15,v)],WHITE,2)
                crystal(d,x+r,y+v*r,r*.22,f,angle=math.pi/2)
        elif local==23:
            line([(-1.3,0),(1.3,0)],WHITE,3)
            for k in range(5):
                u=-1+k*.5
                line([(u,-.35),(u+.2,.35)],c);star(u+.1,0,2+(f+k)%2)
        elif local==24:
            points=[p(x,y,r*.9,k*TAU/5+phase*.1,.8) for k in range(5)]
            path(d,points+[points[0]],WHITE,2)
            for k,(cx,cy) in enumerate(points):
                if f>=k:living_cell(d,cx,cy,r*(.24-k*.025),f+k)
                else:molecule(d,cx,cy,r*.15,f+k,4)
            genesis_vessel(im,x,y,r*.75,f,peak=tier>=2,level=2+tier)
    d=ImageDraw.Draw(im)
    sparkle(d,*p(x,y,r*1.08,phase+local*.13,.85),c,2)
    architecture(im,x,y,r,science,tier,f)


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


def orrery(im,x,y,r,f,ivory=False,variant=0):
    d=ImageDraw.Draw(im);phase=f*TAU/8
    # Rotating projected cube/tesseract, with filled brass plates and a stellar core.
    for n in range(2):
        points=[p(x,y,r*(1 if n==0 else .56),phase*.25+TAU*k/4+math.pi/4,.72) for k in range(4)]
        d.polygon(points,fill=(51,30,18,80 if n==0 else 175))
        path(d,points+[points[0]],col(1),2)
        if n==0:outer=points
        else:
            for a,b in zip(outer,points):path(d,[a,b],WHITE)
    for k in range(3):ellipse(d,x,y,r*(.72+k*.17),1,phase+k,.42,ivory,1,k*TAU/3+.2)
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


def reactor(im,x,y,r,f,ivory=False,variant=0):
    nebula(im,x,y,r*1.2,f)
    d=ImageDraw.Draw(im);phase=f*TAU/8
    ellipse(d,x,y,r,2,phase,.55,ivory,2)
    for k in range(6):
        a=k*TAU/6+phase*.3
        px,py=p(x,y,r*.7,a,.68)
        crystal(d,px,py,r*(.26+.04*((k+variant)%3)),f,angle=a+math.pi/2)
        path(d,[(x,y),p(x,y,r*.36,a+.13,.68),(px,py)],col(2,170))
    crystal(d,x,y,r*.65,f)
    for k in range(16):
        a=k*2.399+phase*.5;rr=r*(.35+.065*((k+f)%12));px,py=p(x,y,rr,a,.7)
        sparkle(d,px,py,stellar_light(k/16,210) if ivory else col(2,210),1)


def tunnel(im,x,y,r,f,science=0,ivory=False):
    d=ImageDraw.Draw(im);phase=f*TAU/8
    # A portal throat recedes into a second plane, with a moving folded interior.
    for k in range(5,0,-1):
        depth=((k-1)/5+f/8)%1
        cx=x-depth*28;cy=y-depth*18;q=r*(1-depth*.75)
        d.ellipse((cx-q,cy-q*.48,cx+q,cy+q*.48),fill=(10,9,32,160))
        ellipse(d,cx,cy,q,science,phase+k*.3,.48,ivory,1)
    quantum(d,x,y,r*.5,f)


def field(kind,tier,f,size=128):
    im=Image.new('RGBA',(size,size));x=y=size//2;r=[17,24,32,37][tier];phase=f*TAU/8
    d=ImageDraw.Draw(im);top=tier==3
    if kind in (12,18,20):
        if kind==18:
            cosmic_shell(im,x,y,r,f)
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
                a=k*TAU/3-.3;path(d,[(x,y),p(x,y,r*1.34,a,.7)],col(0),2)
                sparkle(d,*p(x,y,r*1.34,a,.7),WHITE,2)
        else:
            for k in range(3):
                color=stellar_light(k/3+f/24) if top else col(0,230)
                path(d,[(12,y-12+k*12),(x-20,y-12+k*12),(x,y+(k-1)*3),(size-14,y+(k-1)*17)],color,2)
    elif kind==5:
        nebula(im,x,y,r,f,0);d=ImageDraw.Draw(im)
        d.ellipse((x-r,y-r*.78,x+r,y+r*.78),fill=(13,18,46,160))
        for k in range(3):
            rr=r*(.6+k*.24);ellipse(d,x,y,rr,0,phase/(k+1),.78,top,2)
            path(d,[(x,y),p(x,y,rr,phase/(k+1)-math.pi/2,.78)],WHITE)
            for j in range(12):sparkle(d,*p(x,y,rr,j*TAU/12,.78),col(0),1)
        quantum(d,x,y,r*.28,f)
    elif kind==45:
        # Two curves converge on a measured root; higher ranks open abstraction.
        for k in [-1,1]:
            pts=[(x-r+j*r/12,y+k*math.sin(j*.2+phase*.08)*r*.6) for j in range(25)]
            path(d,pts,WHITE if k==1 else col(1),2)
        sparkle(d,x+math.sin(phase*.08)*r*.3,y,col(1),3)
        if tier>=2:math_object(im,x,y+r*.45,r*.6,f,tier)
    elif kind in (57,58):
        nebula(im,x,y,r,f);d=ImageDraw.Draw(im)
        if kind==57:
            for k in [-2,-1,0,1,2]:
                crystal(d,x+k*r*.35,y+abs(k)*6,r*(1-.24*abs(k))*(.83+.17*math.sin(phase+k)),f,angle=k*.22+.06*math.sin(phase))
                if tier>=2 and f>=3:living_cell(d,x+k*r*.35,y-r*(.3+.1*(f-3)),r*.13,f+k)
        else:
            points=[p(x,y,r,phase*.12+k*TAU/6,.72) for k in range(6)]
            for k,(px,py) in enumerate(points):
                for tx,ty in points[k+1:]:path(d,[(px,py),(tx,ty)],col(2,100),1)
                crystal(d,px,py,r*.32,f,angle=k*TAU/6)
            crystal(d,x,y,r*.65,f)
        ellipse(d,x,y+12,r*1.12,2,phase,.4,top,2)
    elif kind==65:
        nebula(im,x,y,r*1.1,f)
        d=ImageDraw.Draw(im)
        for k in range(28):
            a=k*2.399+phase*.24;rr=r*(.42+((k*5+f)%19)/22)
            px,py=p(x,y,rr,a,.7);q=1+k%3
            d.ellipse((px-q,py-q,px+q,py+q),fill=col(2,185-k*3))
        for k in range(3):
            px,py=p(x,y,r*.55,k*TAU/3,.6)
            if f>=3:living_cell(d,px,py,r*(.14+.02*f),f+k)
            else:molecule(d,px,py,r*.16,f+k,5)
    else:
        # Osmotic Draw: a porous membrane and directed molecular flow.
        d=ImageDraw.Draw(im)
        for dx in [-r*.12,r*.12]:path(d,[(x+dx,y-r),(x+dx,y+r)],WHITE,2)
        for k in range(7):d.rectangle((x-2,y-r+k*r/3,x+2,y-r+k*r/3+2),fill=col(2))
        for k in range(3):
            path(d,[(x-r,y+(k-1)*14),(x,y+(k-1)*4),(x+r,y+(k-1)*3)],col(2),2)
            molecule(d,x-r+(f/8)*2*r,y+(k-1)*12,r*.13,f+k,4)
    architecture(im,x,y,r,0 if kind<25 else (1 if kind<50 else 2),tier,f)
    # Baked emission only, one draw call for the complete scientific construction.
    return glow(im)


def watch(d,x,y,r,f):
    """A pocket watch, with a visible crown and two independent hands."""
    phase=f*TAU/8
    d.ellipse((x-r,y-r,x+r,y+r),fill=(17,32,49,245),outline=col(0),width=1)
    d.rectangle((x-1,y-r-3,x+1,y-r-1),fill=(225,211,169,255))
    for a in range(4):d.point(p(x,y,r-2,a*TAU/4),fill=WHITE)
    path(d,[(x,y),p(x,y,r*.72,phase-.5)],WHITE)
    path(d,[(x,y),p(x,y,r*.5,-phase*.5)],col(0))


def apple(d,x,y,r,f):
    d.ellipse((x-r,y-r*.68,x,y+r*.75),fill=(160,46,48,255),outline=(42,26,31,255))
    d.ellipse((x-1,y-r*.68,x+r,y+r*.75),fill=(198,62,50,255),outline=(42,26,31,255))
    path(d,[(x,y-r*.45),(x+1,y-r)],(170,150,91,255))
    path(d,[(x+1,y-r),(x+r*.55,y-r*.8)],(165,176,96,255))
    d.point((round(x-r*.5),round(y-r*.3)),fill=(249,153,111,255))
    d.point(p(x,y,r*.9,f*TAU/8,.7),fill=col(1))


def capsule(d,x,y,r,f,kind=0):
    """Sealed sample glass, distinguishable from a bare magic projectile."""
    path(d,[(x-r*.7,y-r*.5),(x+r*.35,y-r*.5),(x+r*.8,y),
            (x+r*.35,y+r*.5),(x-r*.7,y+r*.5),(x-r*.7,y-r*.5)],WHITE)
    d.rectangle((round(x-r*.72),round(y-r*.5),round(x-r*.4),round(y+r*.5)),fill=(66,87,86,255))
    path(d,[(x-r*.25,y),(x+r*.48,y)],col(2),max(1,round(r*.4)))
    if kind%2:crystal(d,x+r*.15,y,r*.43,f,angle=math.pi/2)
    else:molecule(d,x+r*.12,y,max(2,r*.35),f,3+kind%4)


def delivery_object(im,skill,tier,f,x,y,r):
    """The moving object has a subject and a use, not a shared orbiting aura."""
    d=ImageDraw.Draw(im);science=skill//25;phase=f*TAU/8
    if skill==0:
        pts=[(x-r,y),(x-r*.12,y-r*.42),(x+r,y),(x-r*.12,y+r*.42)]
        d.polygon([(round(a),round(b)) for a,b in pts],fill=(35,95,148,255),outline=WHITE)
        path(d,[(x-r*.65,y),(x+r*.72,y)],col(0),2)
    elif skill==1:
        for a in (0,-math.pi/2,math.pi*.75):
            q=p(x,y,r,a,.8);path(d,[(x,y),q],col(0));sparkle(d,*q,WHITE,1)
        d.rectangle((x-1,y-1,x+1,y+1),fill=WHITE)
    elif skill==5:watch(d,x,y,r,f)
    elif skill==4:
        watch(d,x,y,r*.7,f)
        path(d,[p(x,y,r,-2+j*.2,.8) for j in range(14)],col(0))
    elif skill in (12,20):
        # Only the genuinely gravitational objects carry a dark centre.
        d.ellipse((x-r*.6,y-r*.6,x+r*.6,y+r*.6),fill=(8,14,31,255),outline=col(0))
        ellipse(d,x,y,r,0,phase,.45,False,1,phase*.08)
        sparkle(d,*p(x,y,r,phase,.45),WHITE,1)
        if skill==20:ellipse(d,x,y,r*1.12,0,phase,.7,True,1)
        else:path(d,[(x-r,y-r*.55),(x,y),(x+r*.65,y+r*.7)],col(0))
    elif skill==13:
        pts=[(x-r*.65,y-r),(x+r*.75,y),(x-r*.65,y+r),(x-r*.65,y-r)]
        d.polygon([(round(a),round(b)) for a,b in pts],fill=(32,67,103,160),outline=WHITE)
        path(d,[(x-r*.3,y-r*.6),(x+r*.3,y),(x-r*.3,y+r*.6)],col(0))
    elif skill==10:
        for k in (-1,1):sparkle(d,x+k*r*.7,y+k*r*.2,col(0),2)
        path(d,[(x-r*.7,y-r*.2),(x,y-r*.5),(x+r*.7,y+r*.2)],WHITE)
    elif skill==11:
        for k in (-1,0,1):
            path(d,[(x-r+j*r/6,y+k*r*.3+math.sin(j*.3+phase*.1)*r*.25) for j in range(13)],col(0,190))
        sparkle(d,x+r*.5,y,WHITE,2)
    elif skill==16:
        for k in range(3):
            xx=x-r+k*r*.6
            path(d,[(xx,y-r*.6),(xx+r*.3,y),(xx,y+r*.6)],WHITE if k==2 else col(0))
    elif skill==18:
        for k in range(3):
            path(d,[p(x,y,r*(.18+j*.025),phase*.1+k*TAU/3+j*.2,.7) for j in range(24)],col(0,180))
        path(d,[(x-r,y-r*.55),(x+r,y-r*.25),(x+r*.6,y+r*.65)],WHITE)
    elif skill==19:
        for sign in (-1,1):
            pts=[(x+sign*r*.22,y-r),(x+sign*r*.75,y-r*.7),(x+sign*r*.75,y+r*.7),(x+sign*r*.22,y+r)]
            path(d,pts,WHITE,1)
        path(d,[(x,y-r),(x,y+r)],col(0),2)
    elif skill==34:
        # A constructed gauntlet: segmented knuckles, cuff, and a leading fist.
        pts=[(x-r,y-r*.5),(x-r*.1,y-r*.7),(x+r*.7,y-r*.45),
             (x+r,y),(x+r*.65,y+r*.55),(x-r,y+r*.5)]
        d.polygon([(round(a),round(b)) for a,b in pts],fill=(117,82,47,255),outline=col(1))
        for k in range(3):path(d,[(x+r*.1+k*r*.25,y-r*.4),(x+r*.1+k*r*.25,y+r*.25)],WHITE)
        path(d,[(x-r*.8,y-r*.35),(x-r*.8,y+r*.35)],WHITE,2)
    elif skill==40:apple(d,x,y,r*.65,f)
    elif skill==25:
        for xx,yy in ((r,0),(0,-r),(-r*.6,r*.6)):
            path(d,[(x,y),(x+xx,y+yy)],col(1));sparkle(d,x+xx,y+yy,WHITE,1)
        d.ellipse((x-2,y-2,x+2,y+2),fill=(114,82,47,255),outline=WHITE)
    elif skill==26:
        path(d,[(x-r,y+r*.6),(x+r,y-r*.6),(x+r*.6,y-r*.6)],col(1),2)
        path(d,[(x+r,y-r*.6),(x+r*.6,y-r*.2)],WHITE)
    elif skill==38:
        ellipse(d,x,y,r,1,phase,.55,False,1)
        apple(d,*p(x,y,r,phase,.55),max(2,r*.2),f)
        path(d,[(x,y),p(x,y,r,phase,.55)],WHITE)
    elif skill==47:
        pts=[(x-r,y-r*.65),(x+r,y-r*.65),(x+r,y+r*.65),(x-r,y+r*.65)]
        path(d,pts+[pts[0]],col(1))
        path(d,[(x-r*.65,y),(x+r*.65,y)],WHITE)
    elif skill in (39,29):
        math_object(im,x,y,r,f,dimension=1+tier)
        d=ImageDraw.Draw(im)
        if skill==29:
            path(d,[(x-r*.8,y+r*.5),(x+r*.8,y-r*.5)],WHITE)
            sparkle(d,x+r*.7,y-r*.45,col(1),2)
        else:
            path(d,[p(x,y,r*1.05,-2+j*.16,.65) for j in range(13)],col(1))
            path(d,[(x+r*.4,y-r*.6),(x+r*1.25,y-r*.6)],WHITE)
    elif skill==45:
        # Travelling curves converge while the object approaches its root.
        gap=[.85,.7,.55,.4,.25,.15,.08,0][f]*r
        for sign in (-1,1):
            pts=[(x-r+j*r/6,y+sign*(gap+math.sin(j*.27)*r*.2)) for j in range(13)]
            path(d,pts,WHITE if sign==1 else col(1),1)
        sparkle(d,x+r*.55,y,col(1),1+f//3)
    elif skill in (57,58,65,66):
        capsule(d,x,y,r,f,skill-57)
        if skill==57:crystal(d,x,y,r*.5,f)
        elif skill==58:
            path(d,[(x-r*.5,y-r*.75),(x,y),(x+r*.5,y-r*.75)],col(2))
        elif skill==65:
            for k in range(3):d.point(p(x,y,r*.8,phase+k*TAU/3),fill=col(2))
        else:path(d,[(x+r*.4,y-r*.7),(x+r*.4,y+r*.7)],WHITE)
    elif skill==54:
        # Acid remains a liquid stream, not a line of crystalline shards.
        for k in range(5):
            xx=x-r+k*r*.38;yy=y+math.sin(phase+k*.7)*r*.18
            q=max(1,r*(.08+k*.035));d.ellipse((xx-q,yy-q,xx+q,yy+q),fill=col(2,180+k*15))
        path(d,[(x-r,y),(x+r*.9,y)],WHITE,1)
    elif skill==59:
        molecule(d,x,y,r*.65,f,5)
        path(d,[(x-r*.9,y),(x-r*.4,y-r*.45),(x+r*.75,y)],col(2))
    elif skill==63:
        capsule(d,x,y,r,f,3)
        for k in range(3):d.point(p(x,y,r*1.15,phase+k*TAU/3),fill=WHITE)
    elif skill==71:
        crystal(d,x,y,r,f,angle=math.pi/2)
        path(d,[(x-r,y),(x+r,y)],WHITE,2)
    elif skill==72:
        for dy in (-r*.45,r*.45):
            d.ellipse((x-r*.4,y+dy-2,x+r*.4,y+dy+2),fill=col(2),outline=WHITE)
        path(d,[(x-r,y),(x,y-r*.45)],col(2))
        path(d,[(x-r,y),(x,y+r*.45)],col(2))
    elif skill==73:
        path(d,[(x-r*1.2,y),(x+r*1.2,y)],WHITE,2)
        for k in (-1,0,1):path(d,[(x+k*r*.65,y-r*.55),(x+k*r*.65+r*.2,y+r*.55)],col(2))
    elif skill==74:
        crystal(d,x,y,r,f)
        # Three clearly separated doses, contained in the same moving object.
        for k in range(3):
            xx=x-r*.62+k*r*.62
            path(d,[(xx,y-r*.3),(xx+r*.12,y+r*.32)],WHITE)
        sparkle(d,*p(x,y,r*.65,phase),col(2),1)
    elif science==0:quantum(d,x,y,r,f)
    elif science==1:math_object(im,x,y,r,f,dimension=1+tier)
    else:capsule(d,x,y,r,f,skill%7)
    if tier>=2:
        # A single extra measurement follows the object. It is baked into the
        # same sprite, and never creates another orbit/particle runtime loop.
        path(ImageDraw.Draw(im),[(x-r*.8,y-r*1.18),(x+r*.5,y-r*1.18)],col(science,150))
        sparkle(ImageDraw.Draw(im),x+r*.65,y-r*1.18,WHITE,1)


def skill_effect(skill,tier,f,skills):
    """Three preparation cels, release, then a short equipment recoil/settle."""
    im=Image.new('RGBA',(128,128));science=skills[skill]['science'];role=skills[skill]['role']
    d=ImageDraw.Draw(im);phase=f*TAU/8;r=6+tier*2
    # The origin is the scientist's casting hand, just below the body's centre.
    origin=(61,70);travel=[0,0,0,7,15,23,27,29][f]
    y=68-[0,1,2,3,4,2,1,0][f];x=67+travel
    if role in ('Attack','Heavy','Field','Setup') or skill in (13,45,57):
        grow=[.32,.6,.85,1,1,.8,.5,.2][f]
        delivery_object(im,skill,tier,f,x,y,max(2,r*grow))
        if f>=3:
            path(d,[(origin[0]+4,origin[1]),(x-r*.7,y)],col(science,130),1)
    else:
        # Modifiers, protection and observation unfold from the held instrument.
        # Keep each canonical construction, but confine it to the active hand.
        core=Image.new('RGBA',(128,128));experiment(core,skill,0,f,science)
        size=round((28+tier*6)*[.5,.7,.9,1,1,.85,.65,.45][f])
        paste(im,core.resize((size,size),Image.Resampling.NEAREST),73,65)
        if role=='Move':
            path(d,[(50,77),(62,74),(72,76),(82,72)],col(science,160))
        elif role=='Protect':
            path(d,[(77,48),(85,58),(82,77)],col(science),1+tier//2)
        elif role=='Observe':
            path(d,[(60,59),(82,59),(82,66)],WHITE)
    # A real instrument opens before release; no ghost limbs or giant area stamp.
    d=ImageDraw.Draw(im)
    if f<=3:
        if science==0:watch(d,60,70,3+min(f,2),f)
        elif science==1:
            path(d,[(59,73),(62,66),(66,73)],(211,174,104,255),2)
            d.point((62,65),fill=WHITE)
        else:capsule(d,60,71,3+min(f,2),f,skill%7)
    # A travelling white measurement is small and survives indexed packing, so
    # all eight phases remain distinct even when the object has settled.
    d.point((57+f,78),fill=col(science))
    return glow(im,.55)


def packet(skill,tier,f,heading,skills):
    science=skills[skill]['science'];r=[5,7,9,10][tier]
    im=Image.new('RGBA',(64,64));d=ImageDraw.Draw(im);phase=f*TAU/8
    # Each frame keeps the actual collision object at the origin. A short tail
    # records motion while directions are baked; no per-particle animations.
    path(d,[(12+f%3,32),(23,32+math.sin(phase)*2),(32-r*.6,32)],col(science,130),1)
    if skill==40:
        path(d,[(12,39),(21,35),(30,32)],col(1,190))
    elif skill==45:
        for sign in (-1,1):path(d,[(12,32+sign*6),(22,32+sign*3),(30,32)],col(1,160))
    elif skill==73:
        path(d,[(9,32),(51,32)],WHITE,2)
    delivery_object(im,skill,tier,f,32,32,r)
    d=ImageDraw.Draw(im)
    # Phase mark survives palette quantisation even for symmetric molecules.
    d.point((18+f,34),fill=col(science))
    return glow(im).rotate(-heading*360/8,resample=Image.Resampling.NEAREST)


def impact(skill,tier,f,skills):
    """Contact is separate from preparation/travel and lasts exactly 24 ticks."""
    im=Image.new('RGBA',(64,64));d=ImageDraw.Draw(im);science=skills[skill]['science']
    phase=f*TAU/8;life=[.25,.5,.85,1,.9,.65,.4,.15][f];r=(8+tier*4)*life
    if skill in (12,20):
        # A seed settles into the persistent field without another huge halo.
        delivery_object(im,skill,tier,f,32,32,max(2,r*.55))
    elif skill==5:
        watch(d,32,32,max(3,r*.55),f)
        for k in (-1,1):path(d,[p(32,32,r,k*.6+j*.13) for j in range(12)],col(0,190))
    elif skill==13:
        delivery_object(im,skill,tier,f,32,32,max(3,r*.65))
        for k in (-1,1):path(d,[(21,32),(32,32),(32+r,32+k*r*.5)],col(0))
    elif skill in (34,29,33,36):
        # Force has a front and a direction; the shards recoil from the fist.
        path(d,[(25,32-r*.6),(38,32),(25,32+r*.6)],WHITE,2)
        for k in range(3):
            yy=32+(k-1)*r*.55
            path(d,[(37,yy),(37+r*.8,yy+(k-1)*r*.2)],col(1),2)
    elif skill==45:
        for sign in (-1,1):
            path(d,[(13,32+sign*r*.6),(24,32+sign*r*.2),(32,32),(45,32-sign*r*.5)],WHITE if sign==1 else col(1))
        sparkle(d,32,32,WHITE,max(1,round(r*.3)))
    elif skill in (57,58):
        for k in range(3):
            xx=32+(k-1)*r*.48
            crystal(d,xx,32+abs(k-1)*3,max(2,r*(.75-.15*abs(k-1))),f,angle=(k-1)*.22)
            if skill==58:path(d,[(xx,32),(32,32-r*.8)],col(2))
    elif skill in (54,59,63,65,66):
        for k in range(5):
            xx,yy=p(32,32,r,phase*.08+k*TAU/5,.7)
            q=max(1,r*.15);d.ellipse((xx-q,yy-q,xx+q,yy+q),fill=col(2,200))
        if skill==66:path(d,[(32,32-r),(32,32+r)],WHITE)
        elif skill==63:crystal(d,32,32,max(2,r*.45),f)
    elif skill==73:
        path(d,[(8,32),(54,32)],WHITE,max(1,3-f//3))
        for k in (-1,1):path(d,[(32+k*r*.5,32-r*.4),(32+k*r*.65,32+r*.4)],col(2))
    elif skill==74:
        for k in range(3):crystal(d,32+(k-1)*r*.65,32+(k%2)*r*.25,max(2,r*.35),f+k)
    else:
        # Keep all 75 skills recognisable through their existing scientific
        # construction, condensed into a short contact response.
        core=Image.new('RGBA',(128,128));experiment(core,skill,0,f,science)
        size=max(8,round((26+tier*5)*life));paste(im,core.resize((size,size),Image.Resampling.NEAREST),32,32)
    d=ImageDraw.Draw(im)
    # Distinct contact fragments carry each recipe without a circular stamp.
    k=skill%5
    path(d,[(32-r,33+k),(30-r*.5,30+k)],col(science,200))
    d.point((24+f,48-k),fill=col(science))
    return glow(im,.55)


CRYSTAL_HOST_TAGS=('idle','run','attack','hit','skill1','skill2','ult','ult_pre','skill2_aim')


@lru_cache(maxsize=1)
def crystal_host_frames():
    """Known host sprite footprints; no global animation replacement is needed.

    The summoned structure is permanently crowd-controlled and never enters
    Bombardier's deployed form. Its body buff ends with the unit, before death
    animation. Cover regular active poses as well as every idle frame.
    """
    root=Path(__file__).resolve().parents[2]
    sources=[(root/'Sprite kit/base champions/bombardier.anim.json',root/'Sprite kit/base champions/bombardier.png'),
             (root/'mods/tfm2_valorant/champions/shadow_bombardier#anim.fanim',root/'mods/tfm2_valorant/champions/shadow_bombardier#sheet.png')]
    frames=[]
    for metadata_path,sheet_path in sources:
        metadata=json.loads(metadata_path.read_text())
        sheet=Image.open(sheet_path).convert('RGBA')
        for tag in CRYSTAL_HOST_TAGS:
            for frame in metadata['anims'][tag]['frames']:
                rect=frame['data'];x,y,w,h=[int(rect[k]) for k in ('x','y','w','h')]
                im=Image.new('RGBA',(48,64))
                im.alpha_composite(sheet.crop((x,y,x+w,y+h)),(round((48-w)/2),round((64-h)/2)))
                frames.append((metadata_path.stem+':'+tag,im))
    return frames


@lru_cache(maxsize=1)
def crystal_formation_hull():
    mask=Image.new('L',(48,64))
    for _,im in crystal_host_frames():mask=ImageChops.lighter(mask,im.getchannel('A'))
    # Include a pixel on every side for odd-sized sprite anchor rounding.
    mask=mask.filter(ImageFilter.MaxFilter(3))
    points=sorted((x,y) for y in range(64) for x in range(48) if mask.getpixel((x,y)))
    def cross(o,a,b):return (a[0]-o[0])*(b[1]-o[1])-(a[1]-o[1])*(b[0]-o[0])
    lower=[];upper=[]
    for q in points:
        while len(lower)>=2 and cross(lower[-2],lower[-1],q)<=0:lower.pop()
        lower.append(q)
    for q in reversed(points):
        while len(upper)>=2 and cross(upper[-2],upper[-1],q)<=0:upper.pop()
        upper.append(q)
    return lower[:-1]+upper[:-1]


def crystal_body(f):
    """Opaque, bounded structure covering the known unit's active silhouettes."""
    im=Image.new('RGBA',(48,64));d=ImageDraw.Draw(im);hull=crystal_formation_hull()
    d.polygon(hull,fill=(21,52,55,255),outline=(91,180,158,255))
    # Solid facets cover the placeholder even between brighter crystal spines.
    for k in range(0,len(hull),2):
        a=hull[k];b=hull[(k+2)%len(hull)]
        d.polygon([a,b,(26,29)],fill=[(30,86,82,255),(37,109,97,255),(25,65,66,255)][k%3])
        path(d,[a,(26,29),b],col(2,255))
    crystal(d,24,25,17,f)
    crystal(d,36,34,10,f,angle=.3)
    crystal(d,16,35,9,f,angle=-.3)
    path(d,[(24,10),(24,39)],WHITE)
    d.point((20+f,33),fill=WHITE)
    # Keep this formation opaque. The field channel owns all glow and movement.
    alpha=Image.new('L',im.size);ImageDraw.Draw(alpha).polygon(hull,fill=255)
    im.putalpha(alpha)
    # Clear hidden RGB outside the opaque hull, matching atlas compositing.
    stored=Image.new('RGBA',im.size);stored.alpha_composite(im)
    return stored


def verify_crystal_coverage():
    """Verify each stored phase covers every regular active host pixel."""
    frames=crystal_host_frames()
    for f in range(8):
        cover=crystal_body(f).getchannel('A')
        for tag,host in frames:
            for flip in (False,True):
                host_alpha=host.getchannel('A')
                mask=cover
                if flip:
                    host_alpha=host_alpha.transpose(Image.Transpose.FLIP_LEFT_RIGHT)
                    mask=mask.transpose(Image.Transpose.FLIP_LEFT_RIGHT)
                assert all(not a or mask.getpixel((x,y))==255
                           for y in range(64) for x in range(48)
                           for a in [host_alpha.getpixel((x,y))]),('uncovered crystal host',tag,f,flip)
    return len(frames)*8*2


def modifier(mask,tier,f):
    """Alter a single prepared object beside the hand, instead of floor icons."""
    im=Image.new('RGBA',(64,80));d=ImageDraw.Draw(im);phase=f*TAU/8;r=5+tier
    x,y=39,46
    pts=[(x-r,y),(x,y-r*.7),(x+r,y),(x,y+r*.7)]
    d.polygon([(round(a),round(b)) for a,b in pts],fill=(17,34,46,235),outline=(167,197,189,255))
    if mask&1:
        # Scalar Division changes the held construction into two branches.
        gap=1+f%3
        path(d,[(x-r,y),(x-gap,y),(x+r,y-r*.7)],col(1),1)
        path(d,[(x-gap,y),(x+r,y+r*.7)],col(1),1)
        for dy in (-r*.7,r*.7):sparkle(d,x+r,y+dy,col(1),1)
    if mask&2:
        # A catalyst inserts a reaction node into the same structure.
        crystal(d,x,y,r*.45,f)
        path(d,[(x-r,y),(x,y),(x+r,y)],col(2))
    if mask&4:
        # The decay clock pulses along its edges; it doesn't summon a new aura.
        edge=[pts[k] for k in range(4)]+[pts[0]]
        path(d,edge,col(2,170+f*10))
        path(d,[(x,y),p(x,y,r*.65,phase,.7)],WHITE)
    sparkle(d,*p(x,y,r,phase,.7),WHITE,1)
    return im


def universe_window(im,x,y,r,f):
    """A framed cross-section of space, with stars inside instead of a void."""
    shift=[0,.08,.14,.2,.14,.08,0,-.06][f]*r
    pts=[(x-r,y-r*.65),(x+r*.6,y-r*.9),(x+r,y+r*.5),(x-r*.7,y+r*.8)]
    material_plane(im,pts,0,f)
    d=ImageDraw.Draw(im);path(d,pts+[pts[0]],col(0),1)
    path(d,[pts[0],pts[1]],WHITE,2)
    path(d,[(x-r*.7,y+r*.8),(x-r*.5+shift,y),(x+r*.6,y-r*.9)],col(0,150))
    for k,(u,v) in enumerate([(-.6,-.3),(-.12,.28),(.4,-.45),(.6,.2)]):
        sparkle(d,x+u*r+shift*.3,y+v*r,WHITE if k==f%4 else col(0),1)


def equipment(science,rank,f,front=False,podium=4):
    """One compact hand-held experiment; only completion reveals the full peak."""
    im=Image.new('RGBA',(96,112))
    if front:return im
    d=ImageDraw.Draw(im);phase=f*TAU/8;peak=rank==7 and podium==1
    # Accepted body occupies x=34..57/y=45..82. These instruments stay near
    # the hands; no permanent crown, giant floor ring, or laboratory silhouette.
    r=[3,4,5,6,7,8,9,10][rank]
    if science==0:
        x,y=37,64
        if rank<=1:
            watch(d,x,y,r,f)
            if rank==1:path(d,[(x+3,y),(x+8,y-3)],col(0,150))
        elif peak:
            universe_window(im,x-1,y-2,12,f)
            d=ImageDraw.Draw(im);watch(d,x+8,y+8,3,f)
            path(d,[(x+4,y+5),(x+8,y+8)],(210,191,140,255))
        else:
            cosmic_shell(im,x,y-1,r,f)
            d=ImageDraw.Draw(im);watch(d,x+6,y+6,3,f)
            if rank>=5:quantum(d,x-6,y+7,4,f)
        if rank==7 and podium in (2,3):
            d=ImageDraw.Draw(im)
            for k in range(4-podium):sparkle(d,x-7+k*5,y-12,col(0),1)
    elif science==1:
        x,y=33,67
        if rank<=1:
            path(d,[(x-r,y+r),(x,y-r),(x+r,y+r)],col(1),1)
            path(d,[(x,y-r),p(x,y-r,r+1,phase,.7)],WHITE)
        elif peak:unprovable_shape(im,x,y-3,12,f)
        else:math_object(im,x,y-2,r*1.15,f,dimension=1+rank//3)
        d=ImageDraw.Draw(im)
        if rank>=3:
            ax,ay=p(x,y+2,8+rank//3,phase*.5,.4)
            apple(d,ax,ay,2,f)
            if rank>=5:path(d,[(x-8,y+2),(x,y+5),(x+8,y+2)],col(1,135))
        if rank==7 and podium in (2,3):
            sides=3 if podium==3 else 4
            path(d,[p(x,y-3,13,k*TAU/sides,.6) for k in range(sides+1)],col(1,140))
    else:
        x,y=65,65
        if rank<=1:capsule(d,x,y,3+rank,f,rank)
        elif peak:
            # Idle #1 contains a living seed; its full atom-to-life sequence
            # happens once on completed combinations rather than looping huge.
            genesis_vessel(im,x,y-1,11,f,peak=False,level=3)
            d=ImageDraw.Draw(im);crystal(d,x,y+4,3,f)
        else:
            genesis_vessel(im,x,y,r,f,peak=False,level=max(1,rank-2))
        if rank>=5:
            d=ImageDraw.Draw(im);path(d,[(57,65),(59,70),(65,73)],col(2,150))
        if rank==7 and podium in (2,3):
            d=ImageDraw.Draw(im)
            for k in range(4-podium):molecule(d,x+12,y-5+k*8,2,f+k,3)
    # Every phase has an actual moving measurement, inside the same baked loop.
    d=ImageDraw.Draw(im)
    d.point((round(x-r)+f,round(y+r+3)),fill=col(science,210))
    return glow(im,.45)


def completion(tier,podium,f,science=0):
    """The concluding subject owns its completion; no generic black-hole finale."""
    im=Image.new('RGBA',(128,128));life=[.35,.62,.85,1,1,.85,.6,.3][f]
    r=[17,24,30,35][tier]*life
    peak=tier==3 and podium==1
    if science==0:
        if peak:
            # The watch opens a universe, bends its light around the scientist,
            # and finally contains it again. The last cel is visibly a watch.
            window=[0,10,22,31,29,20,9,0][f]
            if window:universe_window(im,65,59,window,f)
            d=ImageDraw.Draw(im);watch(d,58,78,4,f)
            if 2<=f<=5:
                path(d,[(42,83),(35,65),(41,45),(66,35),(88,49),(92,71),(79,85)],col(0,160),1)
                quantum(d,58,78,8,f)
            if f in (0,7):sparkle(d,63,75,col(0),2)
        else:cosmic_shell(im,64,64,r,f,genesis=True)
    elif science==1:
        if peak:
            unprovable_shape(im,64,60,r,f)
            d=ImageDraw.Draw(im)
            # The apple disappears through an inner face and emerges outside.
            ax,ay=[(53,75),(47,60),(55,48),(65,54),(74,53),(85,67),(73,76),(57,78)][f]
            apple(d,ax,ay,3,f)
        else:math_object(im,64,60,r*1.18,f,dimension=1+tier)
        infinite_stairs(ImageDraw.Draw(im),64,81,r*.6,f,2+tier)
    else:genesis_vessel(im,64,64,r,f,peak=True,level=2+tier)
    d=ImageDraw.Draw(im)
    if tier==3 and podium<=3:
        # One small podium signature, inside the same baked completion channel.
        for k in range(4-podium):sparkle(d,57+k*7,102,WHITE,1+(f+k)%2)
    return glow(im,.8)


def echo(science,f):
    """Eight-tick delayed subject imprint, sharing the existing short echo channel."""
    im=Image.new('RGBA',(96,112));d=ImageDraw.Draw(im)
    if science==0:
        pts=[(27,86),(39,79),(49,84),(61,77),(70,84)]
        path(d,pts,col(0,175))
        for k,(x,y) in enumerate(pts):sparkle(d,x,y,col(0),1+(k+f)%2)
        quantum(d,48,86,14,f)
    elif science==1:math_object(im,48,83,20,f,3)
    else:helix(d,48,83,10,f);molecule(d,48,83,15,f,6)
    im.putalpha(im.getchannel('A').point(lambda alpha:round(alpha*(8-f)/8)))
    return im
