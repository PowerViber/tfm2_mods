"""Reproduce Thalassa's approved artwork atlases and native/Lab art manifest.

Imagegen sources are frozen. Only extraction, uniform resampling, palette matching,
mirroring, tinting and phase transforms happen here; no sprite anatomy is painted.
"""
from pathlib import Path
from PIL import Image, ImageOps, ImageDraw, ImageFont
import hashlib, json, math

ROOT=Path(__file__).resolve().parents[1]
REVIEW=ROOT/'docs/thalassa-art-review'
BODY=ROOT/'mods/tfm2_custom/champions'
VFX=ROOT/'mods/tfm2_custom/vfx'
APPROVAL=json.loads((REVIEW/'approval.json').read_text(encoding='utf-8'))
for kind,name in [('appearance','body-source-v1.png'),('effects','effects-source-v1.png')]:
    assert APPROVAL[kind]['approved'] and hashlib.sha256((REVIEW/name).read_bytes()).hexdigest()==APPROVAL[kind]['source_sha256']

SIZE=(64,56); GROUND=48
COUNTS={'idle':4,'run':6,'attack':4,'skill1':4,'skill2':4,'skill3':4,'ult':4,'hit':1,'dead':6}
TICKS={'idle':[10.8]*4,'run':[7.2]*6,'attack':[6,6,5,5],'skill1':[3]*4,'skill2':[6]*4,'skill3':[5,5,4,4],'ult':[6]*4,'hit':[8.4],'dead':[6]*6}

def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def binary(im):
    im=im.convert('RGBA'); im.putalpha(im.getchannel('A').point(lambda a:255 if a>=180 else 0)); return im
def strip_specks(im):
    px=im.load(); seen=set()
    for y in range(im.height):
        for x in range(im.width):
            if (x,y) in seen or not px[x,y][3]:continue
            stack=[(x,y)]; seen.add((x,y)); found=[]
            while stack:
                a,b=stack.pop(); found.append((a,b))
                for dx in (-1,0,1):
                    for dy in (-1,0,1):
                        q,r=a+dx,b+dy
                        if 0<=q<im.width and 0<=r<im.height and (q,r) not in seen and px[q,r][3]:seen.add((q,r));stack.append((q,r))
            if len(found)<=3:
                for q,r in found:px[q,r]=(0,0,0,0)
    return im

def extract(name,rows):
    source=Image.open(REVIEW/name).convert('RGBA'); pieces=[]
    for i in range(rows*4):
        cell=source.crop((i%4*source.width//4,i//4*source.height//rows,(i%4+1)*source.width//4,(i//4+1)*source.height//rows))
        box=cell.getchannel('A').point(lambda a:255 if a>=220 else 0).getbbox()
        pieces.append(cell.crop(box) if box else None)
    scale=40/pieces[0].height
    frames=[]
    for i,piece in enumerate(pieces):
        if piece is None:frames.append(None);continue
        p=strip_specks(binary(piece.resize((round(piece.width*scale),round(piece.height*scale)),Image.Resampling.NEAREST)))
        # Locate the face, not the extending tail/weapon, for horizontal registration.
        skin=[]
        for y in range(max(1,int(p.height*.48))):
            for x in range(p.width):
                r,g,b,a=p.getpixel((x,y))
                if a and r>170 and 10<r-g<80 and 8<g-b<65:skin.append(x)
        face=sum(skin)/len(skin) if skin else p.width*.58
        dx=round(35-face)
        if name.endswith('death-v1.png') and i>=4:dx=(SIZE[0]-p.width)//2
        dx=max(0,min(SIZE[0]-p.width,dx)); dy=GROUND+1-p.height
        assert p.width<=SIZE[0] and dy>=0,(name,i,p.size)
        frame=Image.new('RGBA',SIZE);frame.alpha_composite(p,(dx,dy));frames.append(frame)
    return frames,scale

a,sa=extract('animation-locomotion-v1.png',3)
b,sb=extract('animation-combat-v1.png',4)
c,sc=extract('animation-ultimate-death-v1.png',3)
frames={'idle':a[:4],'run':a[4:10],'hit':[a[10]],'attack':b[:4],'skill1':b[4:8],'skill2':b[8:12],'skill3':b[12:16],'ult':c[:4],'dead':c[4:10]}
# Preserve the approved roster idle exactly as shown, including its original pixels.
frames['idle'][0]=Image.open(REVIEW/'body-idle.png').convert('RGBA')

BODY.mkdir(parents=True,exist_ok=True);VFX.mkdir(parents=True,exist_ok=True)
atlas=Image.new('RGBA',(512,560)); body_anims={}; body_manifest={}; index=0
for facing in ('right','left'):
    for pose in COUNTS:
        fs=[]
        for i,im in enumerate(frames[pose]):
            if facing=='left':im=ImageOps.mirror(im)
            x,y=index%8*64,index//8*56;atlas.alpha_composite(im,(x,y));index+=1
            fs.append({'duration':TICKS[pose][i]/60,'data':{'x':x,'y':y,'w':64,'h':56}})
        key=pose+('_l' if facing=='left' else '')
        body_anims[key]={'frames':fs};body_manifest[key]=fs
atlas.save(BODY/'tfm2_custom_thalassa#sheet.png')
(BODY/'tfm2_custom_thalassa#anim.fanim').write_text(json.dumps({'anims':body_anims},separators=(',',':'))+'\n',encoding='utf-8')

FX_COLS=16
fx_images=[];fx_anims={};fx_buffs={};fx_meta={}
def add(tag,images,ticks,kind='effect',z=5):
    fs=[]
    for image,t in zip(images,ticks):
        i=len(fx_images);x,y=i%FX_COLS*96,i//FX_COLS*96
        frame=Image.new('RGBA',(96,96));frame.alpha_composite(image,((96-image.width)//2,(96-image.height)//2));fx_images.append(frame)
        fs.append({'duration':t/60,'data':{'x':x,'y':y,'w':96,'h':96}})
    fx_anims[tag]={'frames':fs};fx_meta[tag]={'frames':fs,'kind':kind,'z':z}
    if kind=='buff':fx_buffs[tag]=tag
def effect(tag):return Image.open(REVIEW/('effect-'+tag+'.png')).convert('RGBA')
def phase_frames(tag):
    im=effect(tag)
    if tag in ('pool','whirlpool','water_aura','rescue_shield'):
        return [im.rotate(-i*45,Image.Resampling.NEAREST,expand=False) for i in range(8)]
    out=[]
    for i in range(8):
        # An appearance/expansion tail derived from the approved artwork.
        factor=.65+.35*math.sin((i+1)/8*math.pi/2)
        q=im.resize((max(1,round(im.width*factor)),max(1,round(im.height*factor))),Image.Resampling.NEAREST)
        if i>4:q.putalpha(q.getchannel('A').point(lambda a:a*(8-i)//4))
        out.append(q)
    return out

alias={'pool_rise':'pool','surge':'surge','bump':'bump','bounce':'bounce','dive_in':'dive_in','dive_out':'dive_out','slide':'surge','twin_spawn':'twin_spawn','twin_fly':'surge','twin_pool':'rescue','tide_link':'tide_link','swap':'swap','rescue':'rescue','trident_hit':'trident_hit','trident_land':'dive_out'}
lifetimes={'pool_rise':18,'surge':20,'bump':20,'bounce':10,'dive_in':18,'dive_out':18,'slide':14,'twin_spawn':20,'twin_fly':12,'twin_pool':24,'tide_link':6,'swap':18,'rescue':24,'trident_hit':24,'trident_land':24}
for tag in ('pool','whirlpool'):
    phases=phase_frames(tag);add(tag,phases,[6]*8,z=-2)
    for i,im in enumerate(phases):add(tag+'_p'+str(i),[im],[9],z=-2)
for tag,base in alias.items():
    total=lifetimes[tag];ticks=[total//8+(i<total%8) for i in range(8)]
    # A link is renewed by the native core; use one frame per renewal.
    if tag=='tide_link':add(tag,[effect(base)],[7],z=-2)
    else:add(tag,phase_frames(base),ticks,z=-2 if tag=='twin_pool' else 5)
for tag in ('water_aura','rescue_shield'):add(tag,phase_frames(tag),[6]*8,kind='buff',z=-1 if tag=='water_aura' else 4)
add('bolt',[effect('bolt')],[4],kind='projectile',z=2)
trident=effect('trident')
for i in range(8):
    im=trident.rotate(-i*45,Image.Resampling.NEAREST,expand=True)
    add('trident_p'+str(i),[im],[1])

def water_tint(im):
    out=Image.new('RGBA',im.size); src=im.load();dst=out.load()
    colours=[(13,70,89),(28,124,145),(91,200,204),(183,246,236)]
    for y in range(im.height):
        for x in range(im.width):
            r,g,b,a=src[x,y]
            if a:
                v=(r*3+g*6+b)//10;dst[x,y]=(*colours[min(3,v//64)],185)
    return out
for pose,images in frames.items():
    for i,im in enumerate(images):
        twin=water_tint(im)
        for left in (False,True):
            # Body centres and tail ground anchors match the true body exactly.
            tag='twin_'+pose+'_'+str(i)+('_l' if left else '')
            add(tag,[ImageOps.mirror(twin) if left else twin],[1])
for i in range(8):
    add('twin_p'+str(i),[water_tint(frames['idle'][i%4])],[1])

fx_atlas=Image.new('RGBA',(FX_COLS*96,math.ceil(len(fx_images)/FX_COLS)*96))
for i,im in enumerate(fx_images):fx_atlas.alpha_composite(im,(i%FX_COLS*96,i//FX_COLS*96))
assert max(fx_atlas.size)<=2048,fx_atlas.size
fx_atlas.save(VFX/'thalassa#sheet.png')
(VFX/'thalassa#anim.fanim').write_text(json.dumps({'anims':fx_anims},separators=(',',':'))+'\n',encoding='utf-8')
# Bake rotated/stretched copies of approved thread pixels; the host cannot stretch a line.
LINK={'directions':32,'lengths_pixels':[8,16,32,64,128,256,512],'max_segments':6,'units_per_pixel':950}
link_source=effect('tide_link');pending=[]
for length in LINK['lengths_pixels']:
    for direction in range(LINK['directions']):
        image=link_source.resize((length,8),Image.Resampling.NEAREST).rotate(-direction*360/LINK['directions'],Image.Resampling.NEAREST,expand=True)
        pending.append((f'tide_link_{length}_{direction}',image))
# Deterministic MaxRects packing; occupied rectangles never overlap.
free=[(0,0,2048,2048)];link_atlas=Image.new('RGBA',(2048,2048));link_anims={}
for tag,image in sorted(pending,key=lambda v:(-max(v[1].size),-v[1].width*v[1].height,v[0])):
    w,h=image.size
    options=[(min(rw-w,rh-h),max(rw-w,rh-h),y,x,i) for i,(x,y,rw,rh) in enumerate(free) if w<=rw and h<=rh]
    assert options,('link atlas overflow',tag,image.size)
    _,_,y,x,_=min(options);new=[]
    for rx,ry,rw,rh in free:
        if x>=rx+rw or x+w<=rx or y>=ry+rh or y+h<=ry:new.append((rx,ry,rw,rh));continue
        if x>rx:new.append((rx,ry,x-rx,rh))
        if x+w<rx+rw:new.append((x+w,ry,rx+rw-x-w,rh))
        if y>ry:new.append((rx,ry,rw,y-ry))
        if y+h<ry+rh:new.append((rx,y+h,rw,ry+rh-y-h))
    free=[r for i,r in enumerate(new) if not any(j!=i and a<=r[0] and b<=r[1] and a+c>=r[0]+r[2] and b+d>=r[1]+r[3] and (r!=q or j<i) for j,q in enumerate(new) for a,b,c,d in [q])]
    link_atlas.alpha_composite(image,(x,y))
    fs=[{'duration':1/60,'data':{'x':x,'y':y,'w':w,'h':h}}]
    link_anims[tag]={'frames':fs};fx_meta[tag]={'frames':fs,'kind':'effect','z':-2,'asset':'asset/tfm2_custom/vfx/thalassa_links'}
link_atlas.save(VFX/'thalassa_links#sheet.png')
(VFX/'thalassa_links#anim.fanim').write_text(json.dumps({'anims':link_anims},separators=(',',':'))+'\n',encoding='utf-8')
decoded=(atlas.width*atlas.height+fx_atlas.width*fx_atlas.height+link_atlas.width*link_atlas.height)*4
assert decoded<=32*1024*1024,decoded
manifest={'schema':1,'art_revision':'thalassa-art-1.1','body_asset':'asset/tfm2_custom/champions/tfm2_custom_thalassa',
          'vfx_asset':'asset/tfm2_custom/vfx/thalassa','canvas':list(SIZE),'world_anchor':[32,28],'ground_anchor':[32,48],
          'world_units_per_pixel':950,'body':body_manifest,'effects':fx_meta,'decoded_bytes':decoded,
          'frame_counts':COUNTS,'pose_ticks':TICKS,'pool_renew_ticks':8,'link':LINK,'sources':{p.name:sha(p) for p in sorted(REVIEW.glob('*-v1.png'))},
          'extraction_scales':{'locomotion':sa,'combat':sb,'ultimate_death':sc}}
(REVIEW/'art-manifest.json').write_text(json.dumps(manifest,indent=2)+'\n',encoding='utf-8')
(ROOT/'editor/thalassa-art.js').write_text('window.THALASSA_ART='+json.dumps(manifest,separators=(',',':'))+';\n',encoding='utf-8')
rust=['//! Generated by tools/pack-thalassa-art.py. Presentation only.',
      'pub const ART_REVISION: &str = "thalassa-art-1.1";',
      'pub const POOL_RENEW_TICKS: usize = 8;',
      'pub fn frame(pose: &str, elapsed: usize, looping: bool) -> usize {',
      '    let durations: &[usize] = match pose {']
for pose,ts in TICKS.items():rust.append('        '+json.dumps(pose)+' => &'+str([round(t*1000) for t in ts])+',')
rust+=['        _ => &[10800,10800,10800,10800],','    };',
       '    let total: usize = durations.iter().sum();',
       '    let mut t = if looping { elapsed.saturating_mul(1000) % total } else { elapsed.saturating_mul(1000).min(total - 1) };',
       '    for (i,d) in durations.iter().enumerate() { if t < *d { return i; } t -= d; }',
       '    durations.len() - 1','}']
rust += ['pub fn link_segments(from: (i64,i64), to: (i64,i64)) -> Vec<(String,i64,i64)> {',
         '    let dx=(to.0-from.0) as f64; let dy=(to.1-from.1) as f64; let distance=dx.hypot(dy);',
         '    if distance < 1. { return vec![]; }',
         f'    let dir=((dy.atan2(dx) * {LINK["directions"]}. / std::f64::consts::TAU).round() as i64).rem_euclid({LINK["directions"]});',
         f'    let sizes: &[i64] = &{LINK["lengths_pixels"]};',
         f'    let mut remaining=distance/{LINK["units_per_pixel"]}.; let mut consumed=0.; let mut out=vec![];',
         f'    for i in 0..{LINK["max_segments"]} {{',
         '        if remaining <= 0. { break; }',
         f'        let length=if i=={LINK["max_segments"]-1} {{ sizes.iter().find(|&&v| v as f64 >= remaining).copied().unwrap_or(*sizes.last().unwrap()) }}',
         '            else { sizes.iter().rev().find(|&&v| v as f64 <= remaining).copied().unwrap_or(sizes[0]) };',
         f'        let span=remaining.min(length as f64); let t=(consumed+span/2.)*{LINK["units_per_pixel"]}./distance;',
         '        out.push((format!("tide_link_{length}_{dir}"),(from.0 as f64+dx*t).round() as i64,(from.1 as f64+dy*t).round() as i64));',
         '        remaining-=span; consumed+=span;',
         '    }', '    out', '}']
(ROOT/'native/tfm2_custom_ai/src/thalassa_art.rs').write_text('\n'.join(rust)+'\n',encoding='utf-8')

# A complete-pose contact sheet and real animation GIFs from packaged frames.
board=Image.new('RGB',(1050,9*140+55),'#17313c');draw=ImageDraw.Draw(board)
font=ImageFont.truetype('C:/Windows/Fonts/segoeui.ttf',15)
draw.text((15,10),'THALASSA / complete packaged poses / 2x / same lighting and scale',font=font,fill='#e7f5ee')
for row,pose in enumerate(COUNTS):
    y=45+row*140;draw.text((15,y),pose,font=font,fill='#f1d590')
    for i,im in enumerate(frames[pose]):
        x=110+i*150;board.paste(im.resize((128,112),Image.Resampling.NEAREST),(x,y),im.resize((128,112),Image.Resampling.NEAREST))
    gifs=[]
    for im in frames[pose]:
        bg=Image.new('RGBA',(256,224),'#203a43');bg.alpha_composite(im.resize((256,224),Image.Resampling.NEAREST));gifs.append(bg.convert('RGB'))
    gifs[0].save(REVIEW/(pose+'.gif'),save_all=True,append_images=gifs[1:],duration=[round(t*1000/60) for t in TICKS[pose]],loop=0)
board.save(REVIEW/'complete-body-contact.png')
print(json.dumps({'body_frames':sum(COUNTS.values()),'body_atlas':atlas.size,'fx_atlas':fx_atlas.size,'fx_tags':len(fx_anims),'decoded_bytes':decoded,'body_sha256':sha(BODY/'tfm2_custom_thalassa#sheet.png'),'fx_sha256':sha(VFX/'thalassa#sheet.png')}))
