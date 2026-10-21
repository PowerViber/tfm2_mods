"""Code-native pixel artwork. Fixed local seeds never touch gameplay randomness."""
from PIL import Image, ImageDraw
import json, math, random
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
RULES=json.loads((ROOT/'rules/gojo-visuals.json').read_text())
P={k:tuple(v) for k,v in RULES['palettes'].items()}
def metadata():
    body=json.loads((ROOT/'mods/tfm2_jjk/champions/tfm2_custom_gojo#anim.fanim').read_text());poses={}
    for pose,points in RULES['poseHands'].items():
        poses[pose]=[]
        for f,(x,y) in zip(body['anims'][pose]['frames'],points):
            q=f['data'];poses[pose].append({'ticks1000':round(f['duration']*60000),'offset':[round(x-q['w']/2-7),round(y-q['h']+RULES['bodyFootPadding']+28)]})
    return {**RULES,'poseFrames':poses}
def gesture_offset(kind,frame):
    pose='skill2' if kind=='red' else 'ult' if kind=='domain' else 'skill1'
    return metadata()['poseFrames'][pose][frame]['offset']
def col(k,a=255):return (*P[k],a)
def circle(d,c,r,fill=None,outline=None,width=1):
    if r>0:d.ellipse((c[0]-r,c[1]-r,c[0]+r,c[1]+r),fill=fill,outline=outline,width=width)
def line(d,pts,color,width=1):d.line([(round(x),round(y)) for x,y in pts],fill=color,width=width)
def spiral(d,c,r,phase,color,arms=4,width=1,inward=True):
    for q in range(arms):
        pts=[]
        for j in range(38):
            t=j/37;rr=3+(r-3)*t;a=q*math.tau/arms+phase+(1-t)*2.9*(1 if inward else -1)
            pts.append((c[0]+math.cos(a)*rr,c[1]+math.sin(a)*rr*.83))
        line(d,pts,color,width)
def lightning(d,c,r,n,seed,color):
    rng=random.Random(seed)
    for q in range(n):
        a=q*math.tau/n+rng.random()*.4;pts=[c]
        for j in range(1,5):
            rr=r*j/4;aa=a+rng.uniform(-.18,.18);pts.append((c[0]+math.cos(aa)*rr,c[1]+math.sin(aa)*rr))
        line(d,pts,(12,5,25,245),5);line(d,pts,color,2)
        p=pts[2];line(d,[p,(p[0]+math.cos(a+.8)*r*.3,p[1]+math.sin(a+.8)*r*.3)],color)
def draw_effect(tag,n):
    size=212 if tag in ['fusion','nuke'] else 96 if tag in ['blue_impact','max_blue_impact'] else 56
    im=Image.new('RGBA',(size,size));d=ImageDraw.Draw(im);c=(size//2,size//2);x,y=c;t=n/7;phase=n*math.tau/8
    if tag in ['blue_prepare','max_blue_prepare','blue_flight','max_blue_flight','blue_impact','max_blue_impact','orb']:
        maximum=tag.startswith('max_');r=(34 if 'impact' in tag else 23 if maximum else 19)
        if tag=='orb':r=15
        for rr in range(r,2,-2):circle(d,c,rr,fill=(4,12+rr,60+rr*2,60+int((r-rr)/r*150)))
        spiral(d,c,r+3,phase,col('blue',215),5 if maximum else 3,2)
        spiral(d,c,r,phase+.17,col('cyan',220),4 if maximum else 3)
        for q in range(7 if maximum else 4):
            a=q*2.399+phase*.7;rr=r+4+(q%3)*2;xx=x+round(math.cos(a)*rr);yy=y+round(math.sin(a)*rr*.8)
            d.polygon([(xx-2,yy),(xx,yy-2),(xx+2,yy+1),(xx,yy+2)],fill=(34,55,83,220))
        circle(d,c,7 if maximum else 5,fill=col('cyan'));circle(d,c,3,fill=col('white'))
        d.line((x-5,y,x+5,y),fill=col('white',210))
    elif tag.startswith('red_') or tag.startswith('max_red_'):
        maximum=tag.startswith('max');r=8+round(t*17) if 'impact' in tag else 15 if maximum else 12
        if 'flight' in tag:
            for q in range(3):line(d,[(x-28,y-7+q*7),(x-7,y-3+q*3)],col('red',70+q*35),2)
        for q in range(3 if maximum else 2):
            rr=r-q*4;d.ellipse((x-rr,y-rr*.72,x+rr,y+rr*.72),outline=col('red',225-q*40),width=2)
        circle(d,c,6,fill=(115,7,33,220));circle(d,c,3,fill=col('white'))
        if maximum:lightning(d,c,25,4,931+n,col('red',200))
        d.line((x-6,y,x+6,y),fill=col('white',200))
    elif tag.startswith('purple'):
        if tag=='purple_prepare':
            gap=round(19*(1-t));circle(d,(x-gap,y-4),6,fill=col('blue',230));circle(d,(x+gap,y+4),6,fill=col('red',230))
            spiral(d,c,24-t*6,phase,col('violet',150),3)
            circle(d,c,2+round(t*7),fill=col('white',150+round(t*100)))
        else:
            for q in range(6):
                yy=y-13+q*5;line(d,[(2,yy-3*math.sin(phase+q)),(17,yy),(x, y+(yy-y)*.4)],col('violet',75+q*20),2)
            for rr in range(18,3,-2):circle(d,c,rr,fill=(95+rr*3,35+rr,180+rr*3,100+rr*6))
            spiral(d,c,25,phase,col('cyan',170),2,2);spiral(d,c,23,-phase,col('red',190),2)
            circle(d,c,7,fill=(224,201,255,255));circle(d,(x+2,y),3,fill=col('white'))
            if tag=='purple_amplified':lightning(d,c,29,4,681+n,col('violet'))
    elif tag in ['melee','enhanced','black_flash']:
        if tag=='black_flash':
            lightning(d,c,27,7,401+n,col('red'));d.polygon([(x-9,y-2),(x-3,y-8),(x+7,y-6),(x+10,y+4),(x+1,y+9),(x-8,y+5)],fill=(6,3,15,255))
            if n<2:line(d,[(x-6,y+1),(x,y-2),(x+6,y)],col('white'),2)
        else:
            color=col('cyan') if tag=='enhanced' else (220,235,243,230)
            for q in range(3):line(d,[(5,y-8+q*8),(x-6,y-3+q*3),(x+9,y-6+q*6)],color,2)
            if tag=='enhanced':spiral(d,c,24,phase,col('blue',230),3)
            d.polygon([(x-5,y-2),(x,y-6),(x+7,y-2),(x+4,y+5),(x-3,y+4)],fill=(220,239,247,240))
    elif tag.startswith('rct'):
        for q in range(7):
            xx=x-19+q*6;yy=y+18-((n*4+q*5)%36)
            line(d,[(xx,yy+8),(xx+3*math.sin(phase+q),yy),(xx+1,yy-6)],col('cyan',110+q*15))
            d.polygon([(xx,yy-5),(xx+2,yy-2),(xx,yy+1),(xx-1,yy-2)],fill=col('white',220))
        d.arc((14,21,50,52),n*13,190+n*13,fill=col('cyan',130),width=1)
    elif tag.startswith('step'):
        # A compressed human silhouette and linear afterimages; no portal rings.
        for q in range(5):
            xx=x-22+q*7;alpha=40+q*35
            d.ellipse((xx-2,y-15,xx+2,y-9),fill=col('cyan',alpha));d.polygon([(xx-2,y-8),(xx+3,y-8),(xx+5,y+11),(xx-5,y+11)],fill=(50,100,165,alpha))
            line(d,[(xx-3,y+11),(xx-7,y+22)],col('cyan',alpha));line(d,[(xx+3,y+11),(xx+7,y+20)],col('cyan',alpha))
        for q in range(3):line(d,[(3,y-12+q*13),(57,y-15+q*13)],col('white',85))
    elif tag=='fusion':
        stage=n//3;r=92
        circle(d,c,r,outline=col('violet',210),width=2)
        gap=max(0,40-n*7);circle(d,(x-gap,y),8,fill=col('blue'));circle(d,(x+gap,y),6,fill=col('red'))
        spiral(d,c,22+n*5,phase,col('violet',130),4,2)
        for q in range(3):
            rr=14+q*5+stage*6;circle(d,c,rr,outline=col('white',80+stage*40))
        for q in range(12):
            a=q*math.tau/12;line(d,[(x+math.cos(a)*85,y+math.sin(a)*85),(x+math.cos(a)*92,y+math.sin(a)*92)],col('red' if stage==2 else 'cyan',220),2)
        if stage==2:lightning(d,c,55,6,84+n,col('violet',190))
    elif tag=='nuke':
        r=14+n*11
        # Local core only. The pressure shell is translucent and never a solid flash.
        circle(d,c,r,outline=col('violet',235-n*20),width=3);circle(d,c,max(1,r-6),outline=col('white',140-n*12))
        lightning(d,c,min(92,24+n*10),9,701+n,col('violet',225-n*15))
        circle(d,c,max(2,16-n),fill=(216,197,255,230-n*18));circle(d,c,max(1,7-n),fill=col('white',240-n*20))
    elif tag=='recoil':
        for q in [-1,1]:line(d,[(x+q*6,y-13),(x+q*14,y),(x+q*8,y+15)],col('red',200),2)
    elif tag=='infinity_contact':
        for q in range(4):
            r=15+q*3;d.arc((x-r,y-r,x+r,y+r),285+n*2,430-n*2,fill=col('cyan',210-q*35),width=2 if q==0 else 1)
        for q in range(4):line(d,[(x+16,y-12+q*8),(x+21,y-10+q*6)],col('white',170-q*25))
    elif tag=='guest':
        # Cyan open diamond and shelter arc, never the amber allied danger circle.
        line(d,[(x-7,12),(x,5),(x+7,12),(x,20),(x-7,12)],col('cyan'),2)
        d.arc((13,14,51,52),195,345,fill=col('cyan',170),width=2)
    return im
def gesture(kind,n):
    im=Image.new('RGBA',(48,56));d=ImageDraw.Draw(im);skin=(230,199,167,255);shade=(99,77,68,255)
    # Compact forearm overlays meet the existing sprite's crossed chest/hands.
    line(d,[(21,30),(29,27),(33,23)],(15,20,32,255),5);line(d,[(28,27),(33,24)],skin,3)
    if kind=='domain':
        line(d,[(28,25),(31,19),(34,24),(29,19)],shade,3);line(d,[(28,25),(31,19)],skin);line(d,[(33,25),(29,19)],skin)
    elif kind=='red':
        line(d,[(32,25),(38,23)],shade,3);line(d,[(32,24),(38,23)],skin);circle(d,(39,23),1,fill=col('white'));circle(d,(40,23),3,outline=col('red',190))
    elif kind=='blue':
        line(d,[(31,25),(35,20),(37,22),(34,26)],skin,2);d.arc((30,18,41,29),n*25,180+n*25,fill=col('cyan',190))
    else:
        line(d,[(20,29),(25,24),(32,24),(36,20)],skin,2);line(d,[(32,24),(37,24)],skin)
        circle(d,(22,24),2,fill=col('blue',200));circle(d,(36,22),2,fill=col('red',210));circle(d,(31,22),1,fill=col('white'))
    return im
def status(tag,n):
    im=Image.new('RGBA',(48,72));d=ImageDraw.Draw(im);x,y=26,17
    if tag in ['six_eyes','exhausted']:
        color='cyan' if tag=='six_eyes' else 'amber'
        for xx in [22,30]:
            d.ellipse((xx-3,y-2,xx+3,y+1),fill=(8,43,82,170));circle(d,(xx,y),2,fill=col(color,210));d.point((xx,y-1),fill=col('white'))
            if tag=='six_eyes':d.point((xx+(n%3)-1,y+1),fill=col('blue'))
        if tag=='exhausted':line(d,[(19,10),(23,14),(21,18),(26,21),(29,18)],col('amber',200))
    elif tag in ['amplified','burnout']:
        color='red' if tag=='amplified' else 'amber';d.polygon([(39,45),(43,49),(39,53),(35,49)],fill=(15,9,26,230),outline=col(color,210))
        if tag=='amplified':line(d,[(36,50),(40,47),(39,50),(42,48)],col('red'))
        else:line(d,[(36,46),(42,52)],col('amber'),2)
    else:
        color='cyan' if tag in ['infinity_on','infinity_rebuild'] else 'amber' if tag=='infinity_break' else 'blue'
        for q in range(3):
            r=17+q*3;start=n*14+q*100;d.arc((24-r,34-r,24+r,34+r),start,start+(60 if tag=='infinity_break' else 125),fill=col(color,80+q*30))
        if tag=='infinity_rebuild':line(d,[(15,43),(20,39),(24,42),(31,35)],col('white',170))
    return im
def polish_domain(im,part,n):
    if part!='field':return im
    out=im.copy();d=ImageDraw.Draw(out);w,h=out.size;cx,cy=w/2,h/2;r=min(w,h)*.43
    # Add only inside the retained opaque footprint. Original edge geometry survives.
    alpha=im.getchannel('A');rng=random.Random(9501)
    for q in range(32):
        a=q*2.399+n*.027;rr=r*math.sqrt(rng.random());x=round(cx+math.cos(a)*rr);y=round(cy+math.sin(a)*rr*.92)
        if 1<x<w-2 and 1<y<h-2 and alpha.getpixel((x,y))>100:
            brightness=115+round(90*(.5+.5*math.sin(q+n*.4)));d.point((x,y),fill=(brightness,brightness,min(255,brightness+35),alpha.getpixel((x,y))))
    return out
def windows(tag,images,total,loop=False):
    """Eight-tick slices continue the real clock across six-tick renewals."""
    out=[]
    for offset in range(total):
        run=[]
        for t in range(8):
            at=(offset+t)%total if loop else min(total-1,offset+t);index=min(7,at*8//total)
            if run and run[-1][0]==index:run[-1]=(index,run[-1][1]+1)
            else:run.append((index,1))
        for index,ticks in run:out.append((f'{tag}_d{total}_t{offset}',images[index],ticks/60))
    return out
def frames():
    out=[]
    for tag in RULES['variants']:
        for n in range(8):
            im=draw_effect(tag,n);duration=RULES['lifetimes'].get(tag,24)/480;out.append((tag,im,duration));out.append((tag+('_frame' if tag=='orb' else '')+str(n),im,.05))
            # Symmetric fields/HUD do not need duplicate mirrored pixel storage.
            if tag in ['melee','enhanced','purple_prepare','purple_flight','purple_amplified','red_flight','max_red_flight','step_prepare','step_arrive','infinity_contact']:
                mirror=im.transpose(Image.Transpose.FLIP_LEFT_RIGHT);out.extend([(tag+'_l',mirror,duration),(tag+'_l'+str(n),mirror,.05)])
    for tag in RULES['gestures']:
        for n in range(8):
            im=gesture(tag,n)
            for suffix,pixel in [('',im),('_l',im.transpose(Image.Transpose.FLIP_LEFT_RIGHT))]:
                out.extend([('gesture_'+tag+suffix,pixel,.05),('gesture_'+tag+suffix+str(n),pixel,.05)])
    for tag in RULES['statuses']:
        for n in range(8):out.append((tag,status(tag,n),.05))
    for key in ['ce','strain']:
        for value in range(21):
            im=Image.new('RGBA',(48,72));d=ImageDraw.Draw(im);y=2 if key=='ce' else 8
            d.rounded_rectangle((4,y,43,y+4),radius=1,fill=(8,17,31,220),outline=(70,99,119,230))
            if value:d.line((6,y+2,6+round(35*value/20),y+2),fill=col('cyan' if key=='ce' else 'amber'),width=2)
            out.append((key+str(value),im,.1))
    for radius in RULES['warnings']:
        size=2*radius+10;im=Image.new('RGBA',(size,size));d=ImageDraw.Draw(im);circle(d,(size//2,size//2),radius,outline=(255,180,105,200))
        out.append(('warning'+str(radius),im,.1))
    sequences={}
    for tag,im,duration in out:
        if tag in RULES['variants'] or tag.endswith('_l') and tag[:-2] in RULES['variants'] or tag.startswith('gesture_') and not any(c.isdigit() for c in tag):sequences.setdefault(tag,[]).append(im)
    combat=json.loads((ROOT/'rules/gojo-v2.json').read_text());prep=combat['preparations']
    windows_for={tag:[prep[i]] for i,tag in enumerate(['blue_prepare','max_blue_prepare','red_prepare','max_red_prepare','purple_prepare','rct_prepare','step_prepare'])}
    windows_for.update({'fusion':[combat['nuke_warning']],'blue_impact':[24],'max_blue_impact':[24]})
    for tag,totals in windows_for.items():
        for name in [tag,tag+'_l']:
            if name not in sequences:continue
            for total in totals:out.extend(windows(name,sequences[name],total,tag.endswith('impact')))
    return out
