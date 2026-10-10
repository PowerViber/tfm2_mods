"""Baked eight-frame celestial sprites; no particles or shaders run in the game.

Einstein folds star-filled space. Newton builds luminous celestial machinery.
Curie grows radioactive crystal nebulae. All functions return centred, original
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
    texture=cosmic_material(science,f)
    if tint is not None:texture=Image.blend(texture,Image.new('RGBA',im.size,tint),.2)
    im.alpha_composite(Image.composite(texture,Image.new('RGBA',im.size),mask))


def architecture(im,x,y,r,science,tier,f):
    """Rank changes construction as well as scale, in the same baked sprite."""
    if not tier:return
    d=ImageDraw.Draw(im);phase=f*TAU/8
    # Solid orbiting frame segments: instrumental gate -> separated planes ->
    # three-discipline impossible structure. Nothing runs as a child emitter.
    count=[0,3,4,6][tier]
    for k in range(count):
        a=k*TAU/count+phase*.16
        px,py=p(x,y,r*1.18,a,.78)
        colour=col(science)
        if science==0:
            pts=[p(px,py,5+tier,a+j*TAU/4,.55) for j in range(4)]
            material_plane(im,pts,science,f);d.polygon(pts,outline=colour)
            path(d,[pts[0],pts[2]],WHITE)
        elif science==1:
            pts=[p(px,py,5+tier,a+j*TAU/3) for j in range(3)]
            material_plane(im,pts,science,f);d.polygon(pts,outline=colour)
            path(d,[(px,py),(x+(px-x)*.7,y+(py-y)*.7)],col(1,140))
        else:crystal(d,px,py,5+tier,f,angle=a)
    if tier>=2:
        ellipse(d,x,y,r*1.3,science,phase,.66,tier==3,1, .35)
        ellipse(d,x,y,r*1.15,science,-phase*.5,.38,tier==3,1, -.65)
    if tier==3:
        vertices=[p(x,y,r*1.34,k*TAU/3-math.pi/2+phase*.06,.82) for k in range(3)]
        for k,(a,b) in enumerate(zip(vertices,vertices[1:]+vertices[:1])):
            path(d,[a,b],col(science,170),2)
            sparkle(d,*a,WHITE,2)


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
        if science==2:crystal(d,cx,cy,r*size,f)
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
            for u in [-.8,.8]:paste(im,singularity(round(r*.3),f,tier==3),x+u*r,y)
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
            paste(im,singularity(round(r*.4),f,tier==3),x+r*.4,y)
        elif local==23:
            poly([(-1,0),(-.4,-.5),(.4,-.5),(1,0),(.4,.5),(-.4,.5)])
            clock(0,0,.4);line([(1,0),(1.35,-.6)],WHITE);star(1.35,-.6,3)
        elif local==24:
            for k in range(3):
                a=k*TAU/3+phase*.08;cx,cy=p(x,y,r*.72,a,.7)
                tunnel(im,cx,cy,r*.45,f,k,tier==3)
            line([(0,-1),(.95,.6),(-.95,.6),(0,-1)],WHITE,2)
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
            for k in range(3):
                rr=(k+1)*.3
                line([(-1+j*.1,(1-k*.25)*math.sin(j*rr*.5+phase*.15)) for j in range(21)],col(1,100+k*65),2)
        elif local==22:
            poly([(-.9,-.7),(.9,-.7),(.9,.7),(-.9,.7)],(45,28,28,210))
            for k in range(5):line([(-.75+k*.35,-.7),(-.75+k*.35,.7)],col(1,110),1)
            arrow((-.7,-.4),(.4,-.4),WHITE);arrow((.4,-.4),(-.3,.3),WHITE)
        elif local==24:
            orrery(im,x,y,r,f,tier==3,local)
            for k in range(3):
                a=k*TAU/3+phase*.2;cx,cy=p(x,y,r*.9,a,.7)
                orrery(im,cx,cy,r*.32,f,tier==3,k)
            line([(-1,.8),(0,-1),(1,.8),(-1,.8)],WHITE)
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
            crystal(d,x,y,r*.5,f)
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
            reactor(im,x,y,r*.65,f,tier==3,local)
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
            for k,(cx,cy) in enumerate(points):crystal(d,cx,cy,r*(.38-k*.045),f,angle=k*TAU/5)
            reactor(im,x,y,r*.5,f,tier==3,local)
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
    paste(im,singularity(max(5,round(r*.5)),f,ivory,.7),x,y)


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
    # Draw in a local horizontal frame, then bake all eight headings.
    for k in range(3):
        pts=[(5+j,32+(k-1)*4+math.sin(j*.18-phase+k)*2) for j in range(29)]
        path(d,pts,stellar_light(k/3+f/32,170) if tier==3 else col(science,125+k*40),2)
    if science==0:
        paste(im,singularity(r,f,tier==3,.82),38,32)
        # A photon packet visibly passes through its travelling folded prism.
        d=ImageDraw.Draw(im)
        d.polygon([(30,32-r),(39,32),(30,32+r)],fill=(24,48,87,220),outline=WHITE)
        path(d,[(25,32),(53,32)],WHITE,2)
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


def theory_holds(science,f):
    """A single suspended horizon, three broken instruments and one impossible transit.

    The construction replaces the ordinary Top 10 aura. Its empty interior and
    separated silhouette are deliberate; no underlying reactor/orbits are added.
    All motion, occlusion and the brief alignment pulse live in these eight cels.
    """
    im=Image.new('RGBA',(96,112));d=ImageDraw.Draw(im)
    cx,cy=48,29;accent=col(science);phase=[0,1,2,3,4,5,5,7][f]*TAU/8
    # The machinery settles, holds its alignment, pulses once, then releases.
    drift=[-3,-2,0,2,1,0,0,-2][f]
    pulse=[0,0,0,0,0,0,1,.25][f]
    shade=[(51,62,80,255),(78,61,45,255),(42,68,67,255)][science]
    edge=WHITE if pulse else accent
    for k in range(3):
        angle=-math.pi/2+k*TAU/3+drift*.035*(1 if k%2 else -1)
        outer=26+(abs(drift) if k==1 else 0)
        points=[p(cx,cy,outer,angle+a) for a in [-.5,-.24,.26,.5]]
        points += [p(cx,cy,20,angle+a) for a in [.46,-.46]]
        d.polygon(points,fill=shade,outline=edge)
        path(d,[p(cx,cy,outer-1,angle-.43),p(cx,cy,outer-1,angle+.4)],WHITE,2)
        path(d,[p(cx,cy,22,angle-.35),p(cx,cy,22,angle+.35)],accent)
        # Instrument-specific engraving, still within the same three fragments.
        x,y=p(cx,cy,23,angle)
        if science==0:path(d,[(x-2,y+2),(x+2,y-2)],accent)
        elif science==1:d.rectangle((x-2,y-1,x+2,y+1),outline=accent)
        else:d.polygon([(x,y-3),(x+2,y),(x,y+3),(x-2,y)],outline=accent)
        for da in [-.43,.43]:
            x,y=p(cx,cy,23,angle+da);d.point((x,y),fill=WHITE)
    # A detached measuring key enters the left horizon and reappears on the
    # right at a different elevation. Drawing the void afterwards occludes it.
    x,y=[(24,31),(33,31),(45,31),(61,25),(71,25),(71,25),(71,25),(50,31)][f]
    d.polygon([(x-4,y-2),(x+3,y-2),(x+4,y+1),(x-3,y+2)],fill=shade,outline=accent)
    path(d,[(x-2,y-1),(x+2,y-1)],WHITE)
    # One opaque horizon. A tilted ivory accretion sheet bends behind its rim.
    ellipse(d,cx,cy,18,science,phase,.2,False,1,-.22)
    d.ellipse((cx-13,cy-12,cx+13,cy+12),fill=(4,7,14,255),outline=WHITE,width=1)
    # Broken bright rim and lensed observations keep the black centre empty.
    for k in range(3):
        a=k*TAU/3+phase*.12
        path(d,[p(cx,cy,14,a+j*.08,.9) for j in range(6)],accent,2)
    path(d,[p(cx,cy+8,20,j*math.pi/24,.14) for j in range(25)],WHITE)
    d.arc((cx-15,cy-14,cx+15,cy+14),210,325,fill=accent,width=1)
    for k,(x,y) in enumerate([(18,18),(76,17),(17,50),(78,48)]):
        bend=[0,1,2,3,2,2,2,1][f]*(1 if k%2 else -1)
        path(d,[(x,y),(x+bend,y+2),(x+bend+1,y+4)],stellar_light(k/4+phase/TAU/2,190))
    if pulse:
        # A short contained flash, no full-body strobe or expanding ring stack.
        d.arc((cx-16,cy-15,cx+16,cy+15),195,345,fill=(238,250,255,round(255*pulse)),width=2)
        for x in [cx-27,cx+27]:sparkle(d,x,cy,WHITE,2 if f==6 else 1)
    # The scientist's shadow points up toward the captured universe. Feet,
    # face and held props are painted by the body layer in front of this aura.
    path(d,[(29,79),(34,64),(37,56)],col(science,85))
    path(d,[(66,79),(62,65),(60,57)],col(science,85))
    d.ellipse((34,82,62,85),fill=(7,12,22,145))
    return glow(im,.65)


def top_equipment(science,f,podium):
    if podium==1:return theory_holds(science,f)
    im=Image.new('RGBA',(96,112));d=ImageDraw.Draw(im);phase=f*TAU/8
    ellipse(d,48,65,26,science,phase,.77,False,1,.15)
    if science==0:paste(im,singularity(10,f,False,.8),48,64)
    elif science==1:orrery(im,48,64,15,f)
    else:
        for x in [29,67]:crystal(d,x,65,12,f,angle=(x-48)/70)
    # Ordinary Top 10 has small instruments and a single quiet orbit. Podium
    # additions have distinct silhouettes, leaving #1's horizon crown unique.
    for k in range(3):
        x,y=p(48,64,26,phase*.25+k*TAU/3,.77)
        if podium==3:
            if k==0:paste(im,singularity(4,f),x,y)
            elif k==1:orrery(im,x,y,5,f)
            else:crystal(d,x,y,6,f)
        else:
            d.polygon([(x-3,y-2),(x+3,y-2),(x+4,y+2),(x-3,y+3)],fill=(22,31,49,255),outline=col(science))
            d.point((x,y-1),fill=WHITE)
    if podium==2:
        for x in [30,66]:paste(im,singularity(6,f,False,.85),x,31)
        path(ImageDraw.Draw(im),[(30,31),(39,25),(57,25),(66,31)],col(science,140))
    d=ImageDraw.Draw(im)
    ellipse(d,48,84,22,science,phase,.14,False,1)
    sparkle(d,*p(48,84,22,phase,.14),WHITE,1)
    return glow(im,.65)


def equipment(science,rank,f,front=False,podium=4):
    """Eight baked cels, one persistent background aura, fixed 96x112 anchor."""
    im=Image.new('RGBA',(96,112))
    if front:return im
    if rank==7:return top_equipment(science,f,podium)
    phase=f*TAU/8;r=[12,16,20,23,26,28,30,32][rank]
    top=rank==7;d=ImageDraw.Draw(im)
    # The foot anchor is shared with the compact body placed at (24,24).
    # A small orbit is visible even when the central seed sits behind the coat.
    ellipse(d,48,84,r,science,phase,.18,top,1 if rank<3 else 2)
    if rank==0:
        px,py=p(48,70,15,phase,.32)
        if science==0:paste(im,singularity(3,f),px,py)
        elif science==1:orrery(im,px,py,4,f)
        else:crystal(d,px,py,4,f)
        sparkle(d,*p(48,84,r,phase,.18),col(science),1)
        return glow(im,.7)
    nebula(im,48,64,r*.85,f,science);d=ImageDraw.Draw(im)
    if rank==1:
        ellipse(d,48,66,r,science,phase,.72,False,1,.6)
        ellipse(d,48,66,r,science,-phase,.35,False,1,-.6)
        sparkle(d,*p(48,66,r,phase,.7),WHITE,2)
    elif science==0:
        if rank==2:tunnel(im,51,64,12,f)
        else:
            paste(im,singularity(round(r*.68),f,top,.8),48,60)
            ellipse(d,48,60,r,0,phase,.76,top,2,.25)
        if rank>=4:
            for k in [-1,1]:
                cx=48+k*(r*.8);cy=62+math.sin(phase+k)*4
                paste(im,singularity(5+rank//2,f,top,.8),cx,cy)
                path(d,[(cx,cy),(48,84)],col(0,120))
        if rank>=5:
            # Space folds into two solid night-sky ribbons beside the shoulders.
            for k in [-1,1]:
                cx=48+k*(r*.75)
                points=[(cx-k*6,39),(cx+k*5,48),(cx+k*8,71),(cx-k*5,77),(cx-k*2,56)]
                mask=Image.new('L',im.size);ImageDraw.Draw(mask).polygon(points,fill=255)
                texture=cosmic_material(0,f).crop((16,8,112,120))
                im.alpha_composite(Image.composite(texture,Image.new('RGBA',im.size),mask))
                path(d,points+[points[0]],col(0) if not top else stellar_light(f/16+k/3),1)
    elif science==1:
        orrery(im,48,63,r*.82,f,top,rank)
        if rank>=3:
            ellipse(d,48,62,r,1,-phase,.8,top,2,.15)
            for k in range(3):
                a=phase*.3+k*TAU/3;cx,cy=p(48,62,r,a,.8)
                q=3+rank//2
                points=[p(cx,cy,q,a+j*TAU/4,.65) for j in range(4)]
                d.polygon(points,fill=(133,74,37,245),outline=WHITE)
                path(d,[(cx,cy),(48,64)],col(1,100))
        if rank>=5:
            for k in [-1,1]:
                cx,cy=48+k*27,65+math.sin(phase+k)*6
                orrery(im,cx,cy,6,f,top,rank)
    else:
        reactor(im,48,65,r*.77,f,top,rank)
        if rank>=3:
            for k in [-1,1]:
                cx=48+k*(r*.75)
                crystal(d,cx,57,9+rank,f,angle=k*.45)
                crystal(d,cx+k*5,77,6+rank//2,f,angle=-k*.65)
        if rank>=5:
            ellipse(d,48,63,r,2,phase,.8,top,2,-.25)
            for k in range(8):
                a=phase*.2+k*TAU/8
                sparkle(d,*p(48,64,r,a,.72),col(2,190),1)
    d=ImageDraw.Draw(im)
    if rank>=3:
        # Suspended observations become a bounded constellation, not emitters.
        for k in range(3+rank):
            a=k*2.399+phase*.12;rr=r*(.68+.07*(k%4))
            sparkle(d,*p(48,62,rr,a,.9),col(science,170+k*7),1)
    if rank>=6:
        for k in range(3):
            a=k*TAU/3+phase*.14
            path(d,[p(48,64,r,a,.7),p(48,64,r,a+TAU/3,.7)],col(science,100))
    return glow(im,.9)


def completion(tier,podium,f):
    im=Image.new('RGBA',(128,128));r=[15,22,30,37][tier]
    # Eight real phases: seed -> expansion -> unified structure -> collapse.
    life=[.35,.62,.85,1,1,.85,.6,.3][f];r=round(r*life)
    if tier==3 and podium==1:
        # Three disciplines meet once, then collapse into a captured horizon.
        # The conjunction owns the three accents; idle #1 owns just one.
        d=ImageDraw.Draw(im)
        for k in range(3):
            a=k*TAU/3-math.pi/2
            x,y=p(64,64,r*1.05,a,.85)
            pts=[p(x,y,5+life*4,a+j*TAU/4,.6) for j in range(4)]
            material_plane(im,pts,k,f);d.polygon(pts,outline=col(k))
            if f in (3,4,5):path(d,[(x,y),p(64,64,r*.5,a,.85)],col(k,180),2)
        paste(im,singularity(max(5,round(r*.55)),f,True,.8),64,64)
        if f==4:sparkle(ImageDraw.Draw(im),64,64,WHITE,5)
        return glow(im,.9)
    paste(im,singularity(max(5,r),f,tier==3,.68),64,64)
    orrery(im,64,64,r*.85,f,tier==3)
    d=ImageDraw.Draw(im)
    for k in range(3):
        x,y=p(64,64,r*1.24,k*TAU/3+f*TAU/48,.74)
        if k==2:reactor(im,x,y,max(5,r*.27),f,tier==3)
        else:sparkle(d,x,y,stellar_light(k/3+f/32),3)
    if tier==3 and podium==2:
        ellipse(d,64,64,r*1.4,0,f*TAU/8,.4,True,2)
    elif tier==3 and podium==3:
        for k in range(3):crystal(d,*p(64,64,r,k*TAU/3),max(3,r*.18),f)
    architecture(im,64,64,r,0,tier,f)
    return glow(im)
