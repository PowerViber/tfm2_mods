"""Slim A tailoring and compact chess carvings. All motion is baked, with no runtime particles."""
from pathlib import Path
from PIL import Image,ImageDraw,ImageFont
import importlib.util,json,math
HERE=Path(__file__).parent;ROOT=HERE.parents[1]
spec=importlib.util.spec_from_file_location('slim_a',HERE/'character.py');C=importlib.util.module_from_spec(spec);spec.loader.exec_module(C)
NAVY='#202c44';IVORY='#e4dcc5';GOLD='#bda46f';INK='#101724'
NAMES=['Beginner','Club Player','Tactician','Expert','Master','Grandmaster','Super GM','World Top 10','Three Files','Interlocked Position','Impossible Calculation']
def font(n=8):return ImageFont.truetype('C:/Windows/Fonts/consolab.ttf',n)
def body(pose,f,look=0):
    im=C.frame(pose,f)
    if look==0:return im
    # Replace the original cloth palette in every pose, retaining all original head/limb pixels.
    main=['#202c44','#223049','#22334c','#1c2941','#17253b','#18243a','#18283e','#131f33','#152139','#101e34','#122036'][look]
    light=['#354a73','#3d5277','#465c7d','#445975','#4a5e78','#56657c','#60718a','#68778e','#758297','#8490a1','#a7adad'][look]
    colors={C.SUIT_BASE:main,C.SUIT_LIGHT:light,C.SUIT_LAPEL:light}
    pixels=im.load()
    rgba={tuple(bytes.fromhex(a.lstrip('#')))+(255,):tuple(bytes.fromhex(b.lstrip('#')))+(255,)for a,b in colors.items()}
    for y in range(im.height):
        for x in range(im.width):
            if pixels[x,y]in rgba:pixels[x,y]=rgba[pixels[x,y]]
    if pose=='dead'and f>=1:return im
    d=ImageDraw.Draw(im);bob=1 if pose=='idle'and f in(2,3,4)else(f%2 if pose=='run'else 0);rx=[0,1,2,2,1,0,0,0][f]if pose=='hit'else 0
    x=-rx;y=2+bob
    if look>=1:
        d.rectangle((x+20,y+24,x+21,y+25),fill=IVORY);d.point((x+21,y+25),fill=GOLD)
        d.line((x+20,y+31,x+23,y+32),fill=light)
    if look>=2:
        d.line((x+19,y+26,x+21,y+30),fill=GOLD)
        d.line((x+30,y+23,x+28,y+29),fill=IVORY)
        d.point((x+20,y+28),fill=INK)
    if look>=3:
        d.line((x+21,y+33,x+24,y+33),fill=IVORY)
        d.line((x+29,y+31,x+31,y+31),fill=IVORY)
    if look>=4:
        d.line((x+19,y+23,x+21,y+22),fill=GOLD)
        d.line((x+29,y+22,x+32,y+23),fill=GOLD)
        d.point((x+31,y+34),fill=GOLD)
    if look>=5:
        d.polygon([(x+24,y+25),(x+26,y+28),(x+28,y+25),(x+28,y+35),(x+24,y+35)],fill='#ddd9c9',outline=INK)
        d.line((x+24,y+27,x+26,y+31),fill=GOLD);d.line((x+28,y+27,x+26,y+31),fill=GOLD)
        d.point((x+26,y+33),fill=INK)
    if look>=6:
        d.line((x+19,y+34,x+22,y+36,x+24,y+34),fill=IVORY)
        d.line((x+29,y+34,x+31,y+36,x+33,y+34),fill=GOLD)
    if look>=7:
        d.line((x+21,y+23,x+24,y+26),fill=IVORY,width=2);d.line((x+29,y+23,x+27,y+26),fill=IVORY,width=2)
        d.line((x+20,y+35,x+22,y+35),fill=GOLD)
    if look==8:d.line((x+19,y+28,x+21,y+28),fill=IVORY);d.point((x+19,y+30),fill=IVORY)
    if look==9:d.line((x+30,y+28,x+32,y+30,x+30,y+32),fill=GOLD)
    if look==10:
        d.line((x+19,y+24,x+21,y+24,x+21,y+26),fill=IVORY)
        d.line((x+31,y+24,x+29,y+24,x+29,y+26),fill=GOLD)
    # The held board gets new material details without changing its hand anchor.
    if pose not in('ult',):
        d.line((x+24,y+29,x+37,y+29),fill=GOLD if look>=4 else '#8e7151')
        if look>=3:d.point((x+35,y+34),fill=IVORY)
    return im

def carrier_hull():
    meta=json.loads((ROOT/'Sprite kit/base champions/alchemist.anim.json').read_text());sheet=Image.open(ROOT/'Sprite kit/base champions/alchemist.png').convert('RGBA');mask=Image.new('L',(48,56))
    from PIL import ImageChops,ImageFilter
    for tag in('idle','run','attack','hit','skill','skill1','skill2'):
        for fr in meta['anims'].get(tag,{}).get('frames',[]):
            r=fr['data'];x,y,w,h=[int(r[k])for k in('x','y','w','h')];im=Image.new('RGBA',(48,56));im.alpha_composite(sheet.crop((x,y,x+w,y+h)),(round((48-w)/2),round((56-h)/2)))
            alpha=im.getchannel('A');mask=ImageChops.lighter(mask,alpha);mask=ImageChops.lighter(mask,alpha.transpose(Image.Transpose.FLIP_LEFT_RIGHT))
    return mask.filter(ImageFilter.MaxFilter(3))

def piece(kind,f=0,look=0,dark=False):
    im=Image.new('RGBA',(48,56));im.paste('#18263a',(0,0),carrier_hull().point(lambda a:255 if a else 0));d=ImageDraw.Draw(im)
    d.rounded_rectangle((9,4,38,43),5,fill='#18263a',outline='#6b7681')
    fill='#aab2c4'if dark else IVORY;hi='#f6eed8';shade='#b4ab94'
    d.polygon([(13,35),(16,31),(32,31),(35,35)],fill=fill,outline=INK);d.rectangle((12,35,35,38),fill=fill,outline=INK)
    if kind=='pawn':
        d.ellipse((20,12,28,20),fill=fill,outline=INK);d.polygon([(18,31),(22,21),(26,21),(30,31)],fill=fill,outline=INK)
    elif kind=='rook':
        d.polygon([(18,30),(19,18),(17,17),(17,11),(21,11),(21,14),(23,14),(23,11),(26,11),(26,14),(28,14),(28,11),(31,11),(31,17),(29,18),(30,30)],fill=fill,outline=INK)
        d.line((21,20,21,28),fill=hi)
    elif kind=='bishop':
        d.polygon([(18,30),(22,22),(20,19),(20,16),(25,10),(29,17),(29,20),(26,23),(31,30)],fill=fill,outline=INK);d.line((23,18,27,13),fill=INK)
    elif kind=='knight':
        d.polygon([(18,31),(21,24),(17,22),(16,19),(21,14),(21,10),(25,13),(28,11),(32,17),(33,31)],fill=fill,outline=INK);d.point((23,17),fill=INK);d.line((29,17,31,25),fill=hi)
    elif kind=='king':
        d.polygon([(18,31),(22,23),(19,18),(21,15),(23,15),(23,10),(26,10),(26,15),(29,15),(31,19),(27,23),(31,31)],fill=fill,outline=INK);d.line((21,12,28,12),fill=fill)
    else:
        d.polygon([(18,31),(22,23),(18,16),(17,12),(23,17),(25,11),(28,17),(32,12),(30,21),(27,24),(31,31)],fill=fill,outline=INK)
        for x,y in((17,11),(25,10),(32,11)):d.point((x,y),fill=GOLD)
    d.line((16,36,31,36),fill=GOLD if look>=4 else shade)
    if look>=1:d.point((15,34),fill=hi);d.point((32,34),fill=hi)
    if look>=2:d.line((15,37,17,37),fill=NAVY);d.line((30,37,32,37),fill=NAVY)
    if look>=3:d.line((20,29,28,29),fill=NAVY)
    if look>=5:d.point((23,27),fill=GOLD);d.point((26,27),fill=GOLD)
    if look>=6:d.line((15,33,17,32),fill=GOLD);d.line((31,32,33,33),fill=GOLD)
    if look>=7:d.line((13,38,34,38),fill=GOLD)
    if look==8:
        for x in(20,24,28):d.point((x,37),fill=INK)
    if look==9:d.line((20,36,22,37,24,36,26,37,28,36),fill=INK)
    if look==10:d.line((19,35,23,34,27,35,30,34),fill=hi)
    d.point((20+f%4,36),fill=hi)
    return im

def construction(look,f,left=False):
    im=Image.new('RGBA',(96,96));d=ImageDraw.Draw(im)
    if look<4:return im
    phase=f/8*math.tau;x,y=68,29
    if look<7:
        d.line((x-11,y+9,x-11,y-8,x+8,y-8),fill=(189,164,111,65+look*12))
        if look>=5:d.line((x-7,y+5,x+9,y-11),fill=(228,220,197,80))
        if look>=6:d.polygon([(x-12,y-12),(x+12,y-9),(x+9,y+12),(x-15,y+8)],outline=(189,164,111,110))
    else:
        planes=3 if look==8 else 2 if look==9 else 1
        for n in range(planes):
            ox=(n-1)*4;oy=(n-1)*3
            d.polygon([(x-16+ox,y-14+oy),(x+10+ox,y-10+oy),(x+13+ox,y+14+oy),(x-13+ox,y+10+oy)],fill=(16,28,44,40),outline=(189,164,111,95+n*25))
        if look==10:
            # A single restrained framing loop; actual branch markers are supplied by the planner.
            for n in range(3):
                h=y-9+n*8;d.line((x-13,h,x-7,h,x-3,y+round(math.sin(phase)*2)),fill=(189,164,111,80+n*25))
            d.point((x-3,y+round(math.sin(phase)*2)),fill=(246,238,216,180))
    if left:im=im.transpose(Image.Transpose.FLIP_LEFT_RIGHT)
    return im

def panel(left=False):
    im=Image.new('RGBA',(96,96));d=ImageDraw.Draw(im);offset=-52 if left else 0
    d.rounded_rectangle((62+offset,8,85+offset,34),2,fill=(15,25,41,220),outline=(189,164,111,150));d.text((64+offset,8),'PLAN',font=font(7),fill=IVORY)
    for n in range(5):d.line((66+offset+n*4,16,66+offset+n*4,32),fill=(189,164,111,30));d.line((66+offset,16+n*4,82+offset,16+n*4),fill=(189,164,111,30))
    return im

def marker(step,cell,alternate=False,left=False,edge=False):
    im=Image.new('RGBA',(96,96));d=ImageDraw.Draw(im);x=67+(cell%4)*4;y=17+(cell//4)*4
    if left:x-=52
    color=(169,187,208,115)if alternate else(243,213,144,255)
    if edge:d.polygon([(x-1,y+2),(x+1,y),(x+3,y+2)],fill=color)
    else:
        # Four distinct 3x3 patterns keep order legible without writing tiny fake coordinates.
        shapes=[[(0,0)],[(0,0),(2,2)],[(0,0),(1,1),(2,2)],[(0,0),(2,0),(0,2),(2,2)]]
        for dx,dy in shapes[step]:d.point((x+dx,y+dy),fill=color)
    return im

def contact(path):
    im=Image.new('RGBA',(840,310),'#111a2b');d=ImageDraw.Draw(im)
    for i,look in enumerate((0,4,10)):
        x=20+i*280;d.text((x,12),NAMES[look],font=font(13),fill=IVORY)
        for side in range(2):
            sprite=body('idle',0,look);sprite=sprite if not side else sprite.transpose(Image.Transpose.FLIP_LEFT_RIGHT)
            im.alpha_composite(sprite,(x+20+side*85,50));im.alpha_composite(sprite.resize((144,168),Image.Resampling.NEAREST),(x+side*130,120))
        im.alpha_composite(piece('knight',0,look),(x+190,60))
    im.save(path)
