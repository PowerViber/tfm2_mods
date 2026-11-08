"""Mechanically extract imagegen proposals and build review-only comparisons.

No game assets are written. Full animation authoring waits for appearance approval.
"""
from pathlib import Path
from PIL import Image, ImageDraw, ImageFont, ImageOps
import argparse, hashlib, json, shutil

ROOT = Path(__file__).resolve().parents[1]
REVIEW = ROOT / 'docs/thalassa-art-review'
NAMES = ['idle', 'movement', 'harp_attack', 'trident_cast']
CELL = (64, 56)
ANCHOR = (32, 48)
EFFECTS = [
    ('pool',(53,53)),('whirlpool',(74,74)),('surge',(64,32)),('bump',(38,28)),
    ('bolt',(18,8)),('bounce',(32,18)),('dive_in',(44,32)),('dive_out',(44,40)),
    ('twin_spawn',(36,48)),('rescue',(48,32)),('rescue_shield',(64,64)),('tide_link',(64,12)),
    ('swap',(64,32)),('trident',(48,18)),('trident_hit',(40,40)),('water_aura',(48,48)),
]

def font(size=14):
    return ImageFont.truetype('C:/Windows/Fonts/segoeui.ttf', size)

def approval_status(kind,digest):
    path=REVIEW/'approval.json'
    record=json.loads(path.read_text(encoding='utf-8')).get(kind,{}) if path.exists() else {}
    return 'approved' if record.get('approved') and record.get('source_sha256')==digest else 'pending'

def crop_stock(name):
    base = ROOT / 'Sprite kit/base champions'
    meta = json.loads((base / (name + '.anim.json')).read_text(encoding='utf-8'))
    data = meta['anims']['idle']['frames'][0]['data']
    x,y,w,h = (int(data[k]) for k in ('x','y','w','h'))
    im = Image.open(base / (name + '.png')).convert('RGBA').crop((x,y,x+w,y+h))
    canvas = Image.new('RGBA', CELL)
    # The original frame centre is the host position, not the opaque bounding box.
    canvas.alpha_composite(im, (CELL[0]//2-w//2, CELL[1]//2-h//2))
    return canvas

def body(source):
    target = REVIEW / 'body-source-v1.png'
    if Path(source).resolve()!=target.resolve():shutil.copyfile(source, target)
    im = Image.open(target).convert('RGBA')
    width, height = im.size
    pieces = []
    bounds = []
    for i in range(4):
        x,y = i%2*width//2, i//2*height//2
        part = im.crop((x,y,x+width//2,y+height//2))
        box = part.getchannel('A').point(lambda p:255 if p >= 180 else 0).getbbox()
        if not box:
            raise ValueError('Empty proposal cell')
        bounds.append(box)
        pieces.append(part.crop(box))
    # One scale for every pose. Never shrink a wide weapon pose separately.
    scale = 40 / pieces[0].height
    frames = []
    for i,piece in enumerate(pieces):
        native = piece.resize((round(piece.width*scale),round(piece.height*scale)),Image.Resampling.NEAREST)
        native.putalpha(native.getchannel('A').point(lambda a:255 if a >= 128 else 0))
        # Remove isolated extraction specks, including neighbouring-cell edge pixels.
        pixels=native.load(); seen=set()
        for yy in range(native.height):
            for xx in range(native.width):
                if (xx,yy) in seen or pixels[xx,yy][3]==0:
                    continue
                stack=[(xx,yy)]; component=[]; seen.add((xx,yy))
                while stack:
                    px,py=stack.pop(); component.append((px,py))
                    for dx,dy in ((-1,-1),(0,-1),(1,-1),(-1,0),(1,0),(-1,1),(0,1),(1,1)):
                        nx,ny=px+dx,py+dy
                        if 0<=nx<native.width and 0<=ny<native.height and (nx,ny) not in seen and pixels[nx,ny][3]:
                            seen.add((nx,ny)); stack.append((nx,ny))
                if len(component)<=6:
                    for px,py in component:pixels[px,py]=(0,0,0,0)
        if native.width > CELL[0] or native.height > ANCHOR[1]+1:
            raise ValueError((NAMES[i],native.size))
        frame = Image.new('RGBA', CELL)
        # The tail's ground contact is fixed at y=48 for every pose.
        frame.alpha_composite(native, ((CELL[0]-native.width)//2,ANCHOR[1]+1-native.height))
        frame.save(REVIEW / ('body-'+NAMES[i]+'.png'))
        ImageOps.mirror(frame).save(REVIEW / ('body-'+NAMES[i]+'-left.png'))
        frames.append(frame)
    refs = [(name,crop_stock(name)) for name in ('bard','archer','knight')]
    sheet = Image.new('RGB',(1080,690),'#132730')
    draw = ImageDraw.Draw(sheet)
    draw.text((20,12),'THALASSA / appearance proposal — actual 1x and nearest-neighbour 3x',font=font(19),fill='#e7f5ee')
    draw.text((20,40),'Four concept poses, both facings. Same scale and fixed tail anchor. Full animations await approval.',font=font(),fill='#b6d4d4')
    for i,(name,frame) in enumerate(zip(NAMES,frames)):
        col,row = i%2,i//2
        x,y = 20+col*530,78+row*220
        draw.text((x,y),name.replace('_',' ').upper(),font=font(16),fill='#f4d591')
        for facing in range(2):
            sprite = frame if facing==0 else ImageOps.mirror(frame)
            xx = x+facing*255
            bg = '#aec3a2' if (i+facing)%2 else '#203a43'
            draw.rectangle((xx,y+27,xx+245,y+199),fill=bg)
            ground = y+27+ANCHOR[1]*3
            draw.line((xx,ground,xx+245,ground),fill='#6a9089')
            sheet.paste(sprite,(xx+3,ground-ANCHOR[1]),sprite)
            big=sprite.resize((192,168),Image.Resampling.NEAREST)
            sheet.paste(big,(xx+65,y+27),big)
    y=542
    draw.text((20,y),'STOCK SCALE REFERENCES / original frame centres retained',font=font(16),fill='#e7f5ee')
    for i,(name,sprite) in enumerate(refs):
        x=20+i*350
        draw.text((x,y+30),name,font=font(),fill='#e7f5ee')
        sheet.paste(sprite,(x+75,y+30),sprite)
        big=sprite.resize((128,112),Image.Resampling.NEAREST)
        sheet.paste(big,(x+145,y+30),big)
    sheet.save(REVIEW/'body-comparison.png')
    meta={'status':approval_status('appearance',hashlib.sha256(target.read_bytes()).hexdigest()),'source_sha256':hashlib.sha256(target.read_bytes()).hexdigest(),
          'source_size':list(im.size),'source_cells':bounds,'shared_scale':scale,'canvas':list(CELL),'ground_anchor':list(ANCHOR),
          'poses':NAMES,'body_visible_height':40,'game_assets_written':False,
          'pixel_processing':'uniform nearest-neighbour reduction; alpha threshold; remove isolated components <=6 native pixels; no repainting',
          'stock_visible_bounds':{n:s.getbbox() for n,s in refs}}
    (REVIEW/'body-proposal.json').write_text(json.dumps(meta,indent=2)+'\n',encoding='utf-8')
    print(json.dumps(meta))

def effects(source):
    target=REVIEW/'effects-source-v1.png'
    if Path(source).resolve()!=target.resolve():shutil.copyfile(source,target)
    source_im=Image.open(target).convert('RGBA')
    sheet=Image.new('RGB',(1080,890),'#132730')
    draw=ImageDraw.Draw(sheet)
    draw.text((20,12),'THALASSA / effect proposals — native size and 2x inspection',font=font(19),fill='#e7f5ee')
    draw.text((20,41),'Pool: 25,000 radius / whirlpool: 35,000 radius. Still images; full animation waits for effect approval.',font=font(),fill='#b6d4d4')
    metadata=[]
    for i,(name,size) in enumerate(EFFECTS):
        x0,y0=i%4*source_im.width//4,i//4*source_im.height//4
        part=source_im.crop((x0,y0,(i%4+1)*source_im.width//4,(i//4+1)*source_im.height//4))
        box=part.getchannel('A').point(lambda a:255 if a>=220 else 0).getbbox()
        if not box:raise ValueError(name)
        part=part.crop(box).resize(size,Image.Resampling.NEAREST)
        part.putalpha(part.getchannel('A').point(lambda a:255 if a>=200 else 0))
        part.save(REVIEW/('effect-'+name+'.png'))
        x,y=20+i%4*265,78+i//4*202
        draw.text((x,y),name.replace('_',' '),font=font(15),fill='#f4d591')
        bg='#aec3a2' if i%2 else '#203a43'
        draw.rectangle((x,y+26,x+250,y+188),fill=bg)
        sheet.paste(part,(x+9,y+40),part)
        big=part.resize((size[0]*2,size[1]*2),Image.Resampling.NEAREST)
        sheet.paste(big,(x+90+(148-big.width)//2,y+32+(148-big.height)//2),big)
        metadata.append({'tag':name,'size':list(size),'cell':i,'source_bounds':box,'sha256':hashlib.sha256((REVIEW/('effect-'+name+'.png')).read_bytes()).hexdigest()})
    sheet.save(REVIEW/'effects-comparison.png')
    meta={'status':approval_status('effects',hashlib.sha256(target.read_bytes()).hexdigest()),'source_sha256':hashlib.sha256(target.read_bytes()).hexdigest(),
          'source_size':list(source_im.size),'effects':metadata,'game_assets_written':False,
          'pixel_processing':'nearest-neighbour fit to declared native footprint; alpha threshold; no repainting'}
    (REVIEW/'effects-proposal.json').write_text(json.dumps(meta,indent=2)+'\n',encoding='utf-8')
    print(json.dumps({'effect_count':len(metadata),'source_sha256':meta['source_sha256'],'game_assets_written':False}))

if __name__=='__main__':
    parser=argparse.ArgumentParser()
    parser.add_argument('--body-source')
    parser.add_argument('--effect-source')
    args=parser.parse_args()
    REVIEW.mkdir(parents=True,exist_ok=True)
    if args.body_source:body(args.body_source)
    if args.effect_source:effects(args.effect_source)
    if not args.body_source and not args.effect_source:parser.error('Provide a generated source')
