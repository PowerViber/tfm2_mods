"""Extract original imagegen proposals into review-only, actual-size comparisons."""
from pathlib import Path
from PIL import Image,ImageOps,ImageDraw,ImageFont
import json,hashlib
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/wright-art-review'
SIZE=(48,56);GROUND=48
FLEET={'fighter':(26,18),'bomber':(38,26),'chopper':(28,24),'repair':(32,24),'drone':(16,16),'rocket':(18,6),'carrier':(63,19)}
FONT=ImageFont.truetype('C:/Windows/Fonts/segoeui.ttf',15)
def clean(im):
    im=im.convert('RGBA');im.putalpha(im.getchannel('A').point(lambda a:255 if a>=180 else 0))
    px=im.load();seen=set()
    for y in range(im.height):
        for x in range(im.width):
            if (x,y) in seen or not px[x,y][3]:continue
            stack=[(x,y)];seen.add((x,y));group=[]
            while stack:
                a,b=stack.pop();group.append((a,b))
                for dx,dy in ((-1,0),(1,0),(0,-1),(0,1)):
                    q,r=a+dx,b+dy
                    if 0<=q<im.width and 0<=r<im.height and (q,r) not in seen and px[q,r][3]:seen.add((q,r));stack.append((q,r))
            if len(group)<=2:
                for q,r in group:px[q,r]=(0,0,0,0)
    return im
def stock(name):
    p=ROOT/'Sprite kit/base champions';d=json.loads((p/(name+'.anim.json')).read_text())['anims']['idle']['frames'][0]['data']
    im=Image.open(p/(name+'.png')).convert('RGBA').crop((d['x'],d['y'],d['x']+d['w'],d['y']+d['h']))
    out=Image.new('RGBA',SIZE);out.alpha_composite(im,((48-im.width)//2,(56-im.height)//2));return out
def put(board,im,x,y,scale=1):
    im=im.resize((im.width*scale,im.height*scale),Image.Resampling.NEAREST);board.paste(im,(x,y),im)
source=Image.open(OUT/'body-source-v1.png').convert('RGBA');pieces=[]
for i in range(4):
    cell=source.crop((i%2*source.width//2,i//2*source.height//2,(i%2+1)*source.width//2,(i//2+1)*source.height//2))
    box=cell.getchannel('A').point(lambda a:255 if a>=180 else 0).getbbox();pieces.append(cell.crop(box))
scale=40/pieces[0].height;body={}
for name,piece in zip(('idle','movement','attack','command'),pieces):
    im=clean(piece.resize((round(piece.width*scale),round(piece.height*scale)),Image.Resampling.NEAREST))
    assert im.width<=48 and im.height<=49,(name,im.size)
    out=Image.new('RGBA',SIZE);out.alpha_composite(im,((48-im.width)//2,GROUND+1-im.height));body[name]=out
    out.save(OUT/('body-'+name+'.png'));ImageOps.mirror(out).save(OUT/('body-'+name+'-left.png'))
board=Image.new('RGB',(1040,1080),'#172936');d=ImageDraw.Draw(board)
d.text((18,12),'WRIGHT / original appearance proposal / 1x and 3x / both facings',font=FONT,fill='#eef4fa')
for row,(name,im) in enumerate(body.items()):
    y=45+row*190;d.text((18,y),name.upper(),font=FONT,fill='#ffb65e')
    for facing in range(2):
        x=160+facing*430;bg='#b6c4a4' if facing else '#203747';d.rectangle((x,y,x+415,y+178),fill=bg)
        pose=ImageOps.mirror(im) if facing else im;put(board,pose,x+15,y+102,1);put(board,pose,x+120,y+6,3)
        d.line((x,y+150,x+415,y+150),fill='#708b88')
d.text((18,835),'STOCK SCALE (native pixels and 3x): Bard / Archer / Knight / Wright',font=FONT,fill='#eef4fa')
for i,(name,im) in enumerate([(n,stock(n)) for n in ('bard','archer','knight')]+[('wright',body['idle'])]):
    x=18+i*250;d.text((x,866),name,font=FONT,fill='#b8d1df');put(board,im,x+65,956);put(board,im,x+110,860,3)
board.save(OUT/'body-comparison.png')
fleet={}
for name,size in FLEET.items():
    src=Image.open(OUT/(name+'-source-v1.png')).convert('RGBA');box=src.getchannel('A').point(lambda a:255 if a>=180 else 0).getbbox();src=src.crop(box)
    if name=='carrier':src=src.resize(size,Image.Resampling.NEAREST)
    else:src.thumbnail(size,Image.Resampling.NEAREST)
    im=clean(src);fleet[name]=im;im.save(OUT/(name+'-native.png'))
    if name!='carrier':
        for i in range(8):im.rotate(-45*i,Image.Resampling.NEAREST,expand=True).save(OUT/(name+'-heading-'+str(i)+'.png'))
board=Image.new('RGB',(1040,780),'#172936');d=ImageDraw.Draw(board)
d.text((18,12),'WRIGHT / fleet appearance / 1x and 3x / actual Wright scale',font=FONT,fill='#eef4fa')
for i,(name,im) in enumerate(fleet.items()):
    y=48+i*100;d.text((18,y+12),name.upper(),font=FONT,fill='#ffb65e')
    for side in range(2):
        x=155+side*430;d.rectangle((x,y,x+415,y+92),fill='#b6c4a4' if side else '#203747')
        put(board,body['idle'],x+10,y+18);put(board,im,x+70,y+(92-im.height)//2);put(board,im,x+160,y+(92-im.height*3)//2,3)
board.save(OUT/'fleet-comparison.png')
sources={p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(OUT.glob('*-source-v1.png'))}
approval=json.loads((OUT/'approval.json').read_text()) if (OUT/'approval.json').exists() else {}
approved=all(approval.get(g,{}).get('approved',False) for g in ('body','fleet','effects')) and all(approval.get('sources',{}).get(name)==digest for name,digest in sources.items())
report={'status':'approved_frozen_sources' if approved else 'review_only_not_approved','body_canvas':SIZE,'ground_anchor':[24,48],'body_scale':scale,'fleet':{k:list(v.size) for k,v in fleet.items()},'sources':sources,'game_assets_written':False}
(OUT/'proposal.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report))
