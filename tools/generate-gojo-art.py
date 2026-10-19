"""Reproducible pixel effects and lossless packing. Gojo's body pixels are preserved."""
from pathlib import Path
from PIL import Image,ImageDraw
import copy,hashlib,json,math,random,subprocess
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'mods/tfm2_jjk';DOC=ROOT/'docs/gojo-results'
def packed(sheet,meta):
    meta=copy.deepcopy(meta);unique={};images=[];keys=[]
    for anim in meta['anims'].values():
        for f in anim['frames']:
            r=f['data'];im=sheet.crop((int(r['x']),int(r['y']),int(r['x']+r['w']),int(r['y']+r['h'])));key=(im.size,im.tobytes())
            if key not in unique:unique[key]=len(images);images.append(im)
            keys.append((f,unique[key]))
    candidates=[]
    # Deterministic best-fit shelves: tall frames no longer strand large gaps
    # beside HUD cells. Full frame rectangles and one-pixel gutters are retained.
    order=sorted(range(len(images)),key=lambda i:(-images[i].height,-images[i].width,i))
    for width in [256,512,768,1024,1280,1536,1792,2048]:
        shelves=[];positions=[None]*len(images);height=0
        for i in order:
            im=images[i];w,h=im.size
            if w>width:break
            choices=[(width-x-w,j) for j,(y,rh,x) in enumerate(shelves) if h<=rh and x+w<=width]
            if choices:
                j=min(choices)[1];y,rh,x=shelves[j];positions[i]=(x,y);shelves[j]=(y,rh,x+w+1)
            else:
                positions[i]=(0,height);shelves.append((height,h,w+1));height+=h+1
        else:
            height-=1
            if height<=2048:candidates.append((width*height,width,height,positions))
    # Mixed cosmic fields and small overlays also benefit from rectangle packing.
    # Split all intersecting free rectangles, then prune contained rectangles.
    for width in [1024,1280,1536,1792,2048]:
        free=[(0,0,width+1,2049)];positions=[None]*len(images);used=0
        for i in sorted(range(len(images)),key=lambda i:(-max(images[i].size),-images[i].width*images[i].height,i)):
            w,h=images[i].width+1,images[i].height+1
            options=[(min(fw-w,fh-h),max(fw-w,fh-h),fy,fx) for fx,fy,fw,fh in free if w<=fw and h<=fh]
            if not options:break
            _,_,y,x=min(options);positions[i]=(x,y);used=max(used,y+h-1);new=[]
            for fx,fy,fw,fh in free:
                if x>=fx+fw or x+w<=fx or y>=fy+fh or y+h<=fy:new.append((fx,fy,fw,fh));continue
                if fx<x:new.append((fx,fy,x-fx,fh))
                if x+w<fx+fw:new.append((x+w,fy,fx+fw-x-w,fh))
                if fy<y:new.append((fx,fy,fw,y-fy))
                if y+h<fy+fh:new.append((fx,y+h,fw,fy+fh-y-h))
            unique_rects=list(dict.fromkeys(new));free=[a for j,a in enumerate(unique_rects) if not any(j!=k and a[0]>=b[0] and a[1]>=b[1] and a[0]+a[2]<=b[0]+b[2] and a[1]+a[3]<=b[1]+b[3] for k,b in enumerate(unique_rects))]
        else:candidates.append((width*used,width,used,positions))
    if not candidates:raise ValueError(f'Atlas exceeds 2048: {len(images)} unique frames, {sum(im.width*im.height*4 for im in images)} decoded bytes')
    _,w,h,positions=min(candidates);atlas=Image.new('RGBA',(w,h))
    for im,p in zip(images,positions):atlas.paste(im,p)
    for f,n in keys:f['data']['x'],f['data']['y']=positions[n]
    return atlas,meta
def frame_hashes(sheet,meta):
    out=[]
    for tag,a in meta['anims'].items():
        for f in a['frames']:
            r=f['data'];im=sheet.crop((int(r['x']),int(r['y']),int(r['x']+r['w']),int(r['y']+r['h'])));m=copy.deepcopy(f);m['data'].pop('x');m['data'].pop('y');out.append((tag,m,hashlib.sha256(im.tobytes()).hexdigest()))
    return out
def save_packed(sheet,meta,stem):
    expected=frame_hashes(sheet,meta);atlas,meta=packed(sheet,meta);assert frame_hashes(atlas,meta)==expected
    atlas.save(str(stem)+'#sheet.png',optimize=True);Path(str(stem)+'#anim.fanim').write_text(json.dumps(meta,separators=(',',':'))+'\n');return atlas,meta
def generate():
    DOC.mkdir(parents=True,exist_ok=True);reports=[]
    for relative in ['champions/tfm2_custom_gojo','vfx/gojo']:
        png=OUT/(relative+'#sheet.png');anim=OUT/(relative+'#anim.fanim');im=Image.open(png).convert('RGBA');meta=json.loads(anim.read_text());before=frame_hashes(im,meta);atlas,meta=save_packed(im,meta,OUT/relative);assert before==frame_hashes(atlas,meta);reports.append({'asset':relative,'frames':len(before),'identicalPixelsAndMetadata':True,'atlas':atlas.size})
    frames=[];anims={};previews=[];gifs=[]
    colors=[(130,230,255,220),(30,170,255,230),(235,32,70,255),(190,255,245,220),(100,220,255,210),(35,145,255,230),(255,75,90,230),(165,75,255,230),(70,140,255,220),(255,95,80,220),(165,95,230,160),(140,245,255,230)]
    for k in range(12):
        group=[]
        for n in range(8):
            im=Image.new('RGBA',(96,96));d=ImageDraw.Draw(im);c=colors[k];r=12+n*2
            if k==2:
                rng=random.Random(8100+n)
                for a in range(8):
                    angle=a*math.pi/4;pts=[(48,48),(48+int(math.cos(angle)*15)+rng.randrange(-7,8),48+int(math.sin(angle)*15)+rng.randrange(-7,8)),(48+int(math.cos(angle)*34),48+int(math.sin(angle)*34))]
                    d.line(pts,fill=(30,5,20,255),width=7);d.line(pts,fill=c,width=2)
                d.ellipse((43,43,53,53),fill=(255,235,225,255))
            elif k==0:
                d.ellipse((31,40,65,56),outline=c,width=2);d.ellipse((44,43,52,53),fill=c)
            elif k==3:
                for a in range(4):
                    x=28+((a*17+n*5)%40);y=26+((a*11+n*3)%45);d.line((x-4,y,x+4,y),fill=c,width=2);d.line((x,y-4,x,y+4),fill=c,width=2)
                d.arc((25,24,71,72),n*15,n*15+210,fill=(135,240,225,160),width=2)
            elif k==1:
                d.polygon([(18,53),(34,50),(39,37),(48,34),(58,37),(62,43),(59,53),(43,60),(18,61)],fill=(13,55,90,220),outline=(175,240,255,255))
                for q in range(4):d.line((39+q*5,37,38+q*5,44),fill=c,width=2)
                for q in range(3):d.arc((26-q*7,22-q*4,70+q*7,70+q*4),90+n*9,220+n*9,fill=c,width=2)
                d.line((7,53,25,50),fill=(100,205,255,180),width=2)
            elif k==4:
                for x,phase in [(30,0),(63,180)]:
                    d.arc((x-13,21,x+13,76),phase+n*9,phase+n*9+210,fill=c,width=3)
                for q in range(5):d.line((28+q*2,28+q*9,56+q*2,25+q*9),fill=(120,215,255,110),width=1)
            elif k==5:
                d.ellipse((35,35,61,61),fill=(10,45,100,230),outline=(180,240,255,255),width=2)
                d.ellipse((42,42,54,54),fill=(230,255,255,255))
                for q in range(3):
                    a=n*.24+q*2*math.pi/3;x=48+int(math.cos(a)*28);y=48+int(math.sin(a)*21);d.ellipse((x-4,y-4,x+4,y+4),fill=c);d.arc((17,22,79,74),q*120+n*10,q*120+n*10+65,fill=c,width=2)
            elif k in [6,9]:
                for q in range(8):
                    a=q*math.pi/4;inner=11 if k==6 else 9+n;outer=25+n if k==6 else 27+n;cx=48+int(math.cos(a)*inner);cy=48+int(math.sin(a)*inner);tx=48+int(math.cos(a)*outer);ty=48+int(math.sin(a)*outer);d.line((cx,cy,tx,ty),fill=c,width=3)
                d.ellipse((38,38,58,58),fill=(255,220,210,240),outline=c,width=3)
                if k==9:d.ellipse((17-n,17-n,79+n,79+n),outline=(255,100,85,150),width=2)
            elif k==7:
                for q in range(5,0,-1):
                    rr=7+q*4;d.ellipse((48-rr,48-rr,48+rr,48+rr),fill=(95+q*12,45+q*7,205+q*7,45+q*28))
                d.arc((20,20,76,76),n*14,n*14+145,fill=(60,180,255,230),width=3);d.arc((20,20,76,76),180+n*14,325+n*14,fill=(255,80,130,230),width=3);d.ellipse((42,42,54,54),fill=(250,230,255,255))
            elif k==8:
                for q in range(6):
                    a=q*math.pi/3+n*.08;x=48+int(math.cos(a)*(32-n));y=48+int(math.sin(a)*(32-n));d.line((x,y,48+int(math.cos(a)*11),48+int(math.sin(a)*11)),fill=c,width=2)
                d.ellipse((35,35,61,61),fill=(5,15,45,245),outline=(165,225,255,240),width=3)
            elif k==10:
                for q in range(14):
                    a=q*2.399+n*.04;rr=12+q%5*6;x=48+int(math.cos(a)*rr);y=48+int(math.sin(a)*rr);d.point((x,y),fill=(205,220,255,180))
                d.arc((12,12,84,84),n*10,260+n*10,fill=c,width=2);d.ellipse((18,18,78,78),outline=(110,105,180,100),width=1)
            else:
                for q in range(3):
                    rr=r-q*5
                    if rr>0:d.ellipse((48-rr,48-rr,48+rr,48+rr),outline=(*c[:3],max(45,c[3]-q*55)),width=2)
                if k in [5,6,7]:d.ellipse((43,43,53,53),fill=c)
                if k==11:d.polygon([(48,14),(54,24),(48,34),(42,24)],fill=c)
            group.append(im);frames.append((f'fx{k}',im,.075))
        previews.append(group[3]);gifs.append(group)
    for key,color in [('ce',(95,215,255,255)),('strain',(244,171,66,255))]:
        for n in range(21):
            im=Image.new('RGBA',(42,72));d=ImageDraw.Draw(im);y=0 if key=='ce' else 8;d.rectangle((2,y,39,y+5),fill=(10,20,35,220),outline=(95,120,140,255));
            if n:d.rectangle((3,y+1,3+int(35*n/20),y+4),fill=color)
            frames.append((key+str(n),im,.1))
    for tag,k in [('six_eyes',0),('amplified',2),('burnout',7),('infinity_on',0),('infinity_paused',7),('infinity_break',9)]:
        for im in gifs[k]:frames.append((tag,im,.1))
    legacy=Image.open(OUT/'vfx/gojo#sheet.png').convert('RGBA');lm=json.loads((OUT/'vfx/gojo#anim.fanim').read_text())
    for tag in ['blue','red','purple']:
        for n,f in enumerate(lm['anims'][tag]['frames']):
            q=f['data'];im=legacy.crop((q['x'],q['y'],q['x']+q['w'],q['y']+q['h']));frames.append((f'flight_{tag}{n}',im,f['duration']))
    for tag,radius in [(f'domain{n}',8+round(71*n/7)) for n in range(8)]+[(f'warning{n}',n) for n in [3,20,26,32,36,76]]:
        for phase in range(8):
            size=2*radius+10;im=Image.new('RGBA',(size,size));d=ImageDraw.Draw(im);c=(150,185,255,150) if tag.startswith('domain') else (255,180,105,200)
            d.ellipse((4,4,size-5,size-5),outline=c,width=1)
            for q in range(8):
                a=q*math.pi/4+phase*.025;cx=size/2+math.cos(a)*radius;cy=size/2+math.sin(a)*radius;d.rectangle((int(cx)-1,int(cy)-1,int(cx)+1,int(cy)+1),fill=c)
            frames.append((tag,im,.075))
    sheet=Image.new('RGBA',(1024,8192));x=y=row=0
    for tag,im,dur in frames:
        if x+im.width>1024:y+=row+1;x=row=0
        sheet.paste(im,(x,y));anims.setdefault(tag,{'frames':[]})['frames'].append({'duration':dur,'data':{'x':x,'y':y,'w':im.width,'h':im.height}});x+=im.width+1;row=max(row,im.height)
    atlas,meta=save_packed(sheet.crop((0,0,1024,y+row)),{'anims':anims},OUT/'vfx/gojo_rework')
    sheet=Image.new('RGBA',(768,420),(12,22,38,255));d=ImageDraw.Draw(sheet);labels=['Six Eyes','Blue punches','Black Flash','RCT','Spatial Step','Blue charge','Red charge','Purple','Blue collapse','Red burst','Domain','One guest']
    for k,im in enumerate(previews):
        x=(k%6)*128;y=(k//6)*180;sheet.alpha_composite(im,(x+16,y+24));d.text((x+8,y+127),labels[k],fill='white')
    d.text((12,375),'Existing Gojo body retained. Cyan CE / amber strain. No ranked designs.',fill='white');sheet.save(DOC/'skill-contact-sheet.png')
    animation=[]
    for n in range(8):
        im=Image.new('RGBA',(768,360),(12,22,38,255));d=ImageDraw.Draw(im)
        for k in range(12):x=(k%6)*128;y=(k//6)*180;im.alpha_composite(gifs[k][n],(x+16,y+20));d.text((x+8,y+125),labels[k],fill='white')
        animation.append(im.convert('RGB'))
    animation[0].save(DOC/'skills.gif',save_all=True,append_images=animation[1:],duration=75,loop=0)
    total=sum(Image.open(p).width*Image.open(p).height*4 for p in [OUT/'champions/tfm2_custom_gojo#sheet.png',OUT/'vfx/gojo#sheet.png',OUT/'vfx/gojo_rework#sheet.png']);assert total<=32*1024*1024
    report={'legacy':reports,'newAtlas':atlas.size,'decodedBytes':total,'bodyRedesigned':False,'rankVariants':False};(DOC/'art.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report))
if __name__=='__main__':generate()
