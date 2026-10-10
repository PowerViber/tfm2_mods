"""Baked eight-frame celestial sprites; no particles or shaders run in the game.

Einstein connects quantum light with stellar creation. Newton unfolds impossible
mathematical solids. Curie assembles molecular, living matter. All functions return centred, original
pixel sprites; a fixed seed and eight discrete phases make builds reproducible.
"""
import math
from functools import lru_cache

from PIL import Image, ImageDraw, ImageFilter

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


def skill_effect(skill,tier,f,skills):
    if skill in [1,5,12,13,18,20,45,57,58,65,66]:return field(skill,tier,f)
    im=Image.new('RGBA',(128,128));x=y=64;r=[15,22,29,35][tier]
    science=skills[skill]['science'];local=skill%25;role=skills[skill]['role'];phase=f*TAU/8;top=tier==3
    nebula(im,x,y,r*.8,f,science)
    experiment(im,skill,tier,f,science)
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
    # Recipe-specific satellites change topology/silhouette, rather than tiny ID bars.
    d=ImageDraw.Draw(im)
    for k in range(2+local%4):
        a=phase*.25+local*.37+k*TAU/(2+local%4)
        px,py=p(x,y,r*(1.15+.09*(local%3)),a,.75)
        if science==2:molecule(d,px,py,3+local%4,f,4+local%3)
        elif science==1:
            pts=[p(px,py,3+local%3,a+j*TAU/3) for j in range(4)]
            d.polygon(pts,fill=(69,40,23,220));path(d,pts,WHITE)
        else:sparkle(d,px,py,col(0),2+local%3)
    return glow(im)


def packet(skill,tier,f,heading,skills):
    science=skills[skill]['science'];r=[5,7,9,11][tier]
    im=Image.new('RGBA',(64,64));d=ImageDraw.Draw(im);phase=f*TAU/8
    # Draw in a local horizontal frame, then bake all eight headings.
    for k in range(3):
        pts=[(5+j,32+(k-1)*4+math.sin(j*.18-phase+k)*2) for j in range(29)]
        path(d,pts,stellar_light(k/3+f/32,170) if tier==3 else col(science,125+k*40),2)
    if science==0:
        quantum(d,38,32,r,f)
        # A photon packet visibly passes through its travelling folded prism.
        d=ImageDraw.Draw(im)
        d.polygon([(30,32-r),(39,32),(30,32+r)],fill=(24,48,87,220),outline=WHITE)
        path(d,[(25,32),(53,32)],WHITE,2)
    elif science==1:
        math_object(im,38,32,r*1.4,f,1+tier)
    else:
        molecule(d,38,32,r,f,4+(skill%3))
        if skill==74:living_cell(d,38,32,r,f)
    d=ImageDraw.Draw(im)
    if skill==73:
        path(d,[(3,32),(58,32)],WHITE,2)
        for k in range(4):sparkle(d,10+k*12,32,col(2),2+(f+k)%2)
    elif skill==40:
        path(d,[(7,40),(19,35),(32,32)],col(1,220))
    elif skill==72:
        for yy in [24,40]:sparkle(d,22,yy,col(2),2)
    elif skill==71:
        crystal(d,41,32,r+3,f,angle=math.pi/2)
        path(d,[(28,32),(56,32)],WHITE,2)
    elif skill==54:
        for k in range(4):crystal(d,20+k*8,32+math.sin(phase+k)*3,3+k,f,angle=math.pi/2)
    if tier>=1:
        ellipse(d,38,32,r*1.4,science,phase,.65,tier==3,1)
    if tier>=2:
        for yy in [32-r-3,32+r+3]:sparkle(d,35+f%4,yy,col(science),1)
    return glow(im).rotate(-heading*360/8,resample=Image.Resampling.NEAREST)


def modifier(mask,tier,f):
    im=Image.new('RGBA',(64,80));d=ImageDraw.Draw(im);phase=f*TAU/8;r=5+tier
    for k in range(3):
        if not mask&(1<<k):continue
        x=13+k*19;y=65
        if k==0:
            path(d,[(x-r,y),(x,y),(x+r,y-r)],col(1),2)
            path(d,[(x,y),(x+r,y+r)],col(1),2)
            for dy in [-r,r]:sparkle(d,x+r,y+dy,col(1),1)
        elif k==1:
            crystal(d,x,y,r,f)
            for a in [0,TAU/3,2*TAU/3]:
                path(d,[p(x,y,r,a),p(x,y,r,a+TAU/3)],col(2),1)
        else:
            ellipse(d,x,y,r,2,phase,1,False,1)
            path(d,[(x,y),p(x,y,r-1,phase,1)],WHITE)
        sparkle(d,*p(x,y,r,phase,1),WHITE,1)
        if tier>=2:d.point((x,y-r-2),fill=col(k))
    return im


def equipment(science,rank,f,front=False,podium=4):
    """Three subject silhouettes, one eight-cel background buff per champion."""
    im=Image.new('RGBA',(96,112))
    if front:return im
    d=ImageDraw.Draw(im);phase=f*TAU/8;peak=rank==7 and podium==1
    # All appearances keep the accepted body at (24,24) and floor at y=82.
    # The three peaks deliberately have different occupied space and shadows.
    if science==0:
        if rank==0:
            quantum(d,28,69,6,f);sparkle(d,34,61,col(0),1)
        elif rank==1:
            quantum(d,48,66,18,f)
            for x in [29,67]:sparkle(d,x,62+math.sin(phase)*2,WHITE,2)
        else:
            r=[0,0,15,20,23,25,27,28][rank]
            cosmic_shell(im,51,43 if rank>=5 else 61,r,f,genesis=peak)
            if peak:
                # A folded, star-filled cross-section makes #1's silhouette
                # distinct from the ordinary crescent, even while paused.
                pts=[(24,23),(51,11),(80,22),(65,37),(45,27)]
                material_plane(im,pts,0,f)
                d=ImageDraw.Draw(im);path(d,pts+[pts[0]],col(0),1)
                path(d,[pts[0],pts[1],pts[2]],WHITE,2)
                for k,(x,y) in enumerate([(33,23),(51,18),(68,26)]):
                    sparkle(d,x,y,col(0),1+(k+f)%2)
                # Transparent spiral arms contain stars, never an opaque void.
                for k in [-1,1]:
                    path(d,[p(65,24,1+j*.19,k*math.pi+j*.25+phase*.2,.6) for j in range(22)],col(0,210))
            if rank>=4:
                d=ImageDraw.Draw(im)
                # Two observatory rail fragments measure the open cosmos.
                for k in [-1,1]:
                    path(d,[(48+k*24,65),(48+k*29,49),(48+k*21,40)],col(0),2)
                    path(d,[(48+k*24,65),(48+k*18,66)],WHITE)
            if rank==7 and podium in (2,3):
                d=ImageDraw.Draw(im)
                for k in range(2 if podium==2 else 3):
                    quantum(d,27+k*21,29+(k%2)*8,5,f+k)
            if peak:
                d=ImageDraw.Draw(im)
                path(d,[(35,65),(43,61)],col(0,180))
        d=ImageDraw.Draw(im)
        path(d,[(32,84),(48,81),(64,84)],col(0,110))
    elif science==1:
        if rank==0:
            # The floating compass draws a solid, persistent little plane.
            pts=[(26,67),(32,63),(37,68),(31,72),(26,67)]
            path(d,pts,col(1));path(d,[(31,68),p(31,68,5,phase,.7)],WHITE)
        else:
            r=[0,12,18,22,25,27,28,29][rank]
            if peak:unprovable_shape(im,48,34,27,f)
            else:math_object(im,48,34 if rank>=5 else 63,r,f,dimension=min(rank,4))
            d=ImageDraw.Draw(im)
            if rank>=3 and not peak:infinite_stairs(d,48,71,15 if rank<5 else 20,f,2 if rank<5 else 4)
            if rank==7 and podium in (2,3):
                for k in [-1,1]:math_object(im,48+k*25,57,8+(podium==2)*3,f+k,3)
            if peak:
                # A different-dimensional floor shadow remains after the fold.
                sides=3 if f==5 else 4
                shadow=[p(48,82,19,k*TAU/sides,.16) for k in range(sides+1)]
                path(d,shadow,col(1,130))
                if f==5:sparkle(d,48,75,WHITE,2)
        d=ImageDraw.Draw(im);d.point((24+f*2,79),fill=col(1,180))
    else:
        if rank==0:
            crystal(d,67,70,5,f);molecule(d,67,65,4,f,4)
        elif rank==1:molecule(d,67,63,10,f,5)
        else:
            r=[0,0,13,18,23,25,26,27][rank]
            genesis_vessel(im,66,45 if rank>=4 else 62,r,f,peak=peak,level=rank-1)
            d=ImageDraw.Draw(im)
            if peak:
                # Upper containment lobes and a molecular bridge distinguish
                # the open genesis apparatus from an ordinary sample vessel.
                for x,y in [(58,17),(82,23)]:
                    living_cell(d,x,y,6,f)
                    path(d,[(x,y+5),(66,29)],col(2,150))
                helix(d,70,76,7,f)
            if rank==7 and podium in (2,3):
                for k in range(2 if podium==2 else 3):
                    living_cell(d,29+k*12,72-k*4,5,f+k)
            if rank>=5:
                # One molecular sampling stem joins the scientist to the vessel.
                path(d,[(55,64),(62,72),(72,72)],col(2,135))
        d=ImageDraw.Draw(im)
        for k in range(3):d.point((55+k*5,84-k%2),fill=col(2,125+(k+f)%3*35))
    return glow(im,.65)


def completion(tier,podium,f,science=0):
    """The concluding subject owns its completion; no generic black-hole finale."""
    im=Image.new('RGBA',(128,128));life=[.35,.62,.85,1,1,.85,.6,.3][f]
    r=[17,24,30,35][tier]*life
    if science==0:cosmic_shell(im,64,64,r,f,genesis=True)
    elif science==1:
        if tier==3 and podium==1:unprovable_shape(im,64,60,r,f)
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
