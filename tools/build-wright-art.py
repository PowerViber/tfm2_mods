"""Reproducible review/pack from frozen imagegen sources; integration requires approvals."""
from pathlib import Path
from PIL import Image,ImageOps,ImageDraw,ImageFont
import json,math,hashlib,sys,shutil
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/wright-art-review'
RULES=json.loads((ROOT/'rules/wright.json').read_text());UPP=950
POSE_TICKS={'idle':[9]*8,'run':[6]*8,'attack':[3,3,3,3,3,3,2,2],'skill1':[2,2,2,3,3,2,2,2],'skill2':[2,2,2,3,3,2,2,2],'skill3':[2,2,2,3,3,2,2,2],'ult':[3]*8,'hit':[1]*8,'dead':[6]*8}
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def binary(im):
    im=im.convert('RGBA');im.putalpha(im.getchannel('A').point(lambda a:255 if a>=180 else 0));return im
def fit(im,w,h):
    im=im.copy();im.thumbnail((w,h),Image.Resampling.NEAREST);out=Image.new('RGBA',(w,h));out.alpha_composite(im,((w-im.width)//2,(h-im.height)//2));return out
def material(name,size):
    src=Image.open(OUT/(name+'-source-v1.png')).convert('RGBA');cells=[]
    for i in range(8):
        cell=src.crop((i%4*src.width//4,i//4*src.height//2,(i%4+1)*src.width//4,(i//4+1)*src.height//2))
        box=cell.getchannel('A').point(lambda a:255 if a>=180 else 0).getbbox();cells.append(cell.crop(box) if box else Image.new('RGBA',(1,1)))
    scale=min(size[0]/max(c.width for c in cells),size[1]/max(c.height for c in cells))
    out=[]
    for i,c in enumerate(cells):
        c=binary(c.resize((max(1,round(c.width*scale)),max(1,round(c.height*scale))),Image.Resampling.NEAREST));frame=Image.new('RGBA',size);frame.alpha_composite(c,((size[0]-c.width)//2,(size[1]-c.height)//2));out.append(frame)
    return out
def geometric(kind,size,frames=8):
    out=[]
    for i in range(frames):
        im=Image.new('RGBA',size);d=ImageDraw.Draw(im);w,h=size;cx,cy=w//2,h//2;t=i/max(1,frames-1)
        if kind=='warn':
            d.ellipse((1,1,w-2,h-2),fill=(231,147,61,20),outline=(245,179,89,200),width=1)
            r=round((w//2-3)*t);d.ellipse((cx-r,cy-r,cx+r,cy+r),outline=(255,193,100,120),width=1)
        elif kind=='knock_ring':
            r=round((w//2-2)*(.25+.75*t));d.ellipse((cx-r,cy-r,cx+r,cy+r),outline=(229,224,208,round(220*(1-t*.8))),width=2)
        elif kind=='lock':
            gap=round(3*(1-t));a=3-gap;b=w-4+gap
            for x,y,sx,sy in ((a,a,1,1),(b,a,-1,1),(a,b,1,-1),(b,b,-1,-1)):
                d.line((x,y,x+sx*5,y),fill='#ff874b',width=1);d.line((x,y,x,y+sy*5),fill='#ff874b',width=1)
            d.ellipse((cx-1,cy-1,cx+1,cy+1),fill='#ffcf8c')
        elif kind=='shield':
            d.polygon([(cx,2),(w-4,7),(w-7,h-8),(cx,h-2),(6,h-8),(3,7)],outline=(95,225,208,180),width=1)
        elif kind=='taunt':
            r=round(3+(w//2-5)*t);d.ellipse((cx-r,cy-r,cx+r,cy+r),outline=(255,172,67,200),width=1)
            d.line((cx,cy-6,cx,cy),fill='#ffcf85',width=2);d.point((cx,cy+3),fill='#ffcf85')
        elif kind=='mg':
            d.line((cx-4+i%3,cy+2,cx+3,cy-2),fill='#ffcf7a',width=1);d.line((cx,cy-4,cx,cy+3),fill='#fff0ba')
        elif kind=='stock':
            for j in range(3):
                x=15+j*12;col='#ffae50' if j<i else '#455364';d.line((x,4,x+8,4),fill=col,width=1);d.line((x+4,1,x+4,7),fill=col,width=1);d.line((x+1,7,x+7,7),fill=col)
        out.append(im)
    return out
body={};complete=all((OUT/('animation-'+p+'-v1.png')).exists() for p in POSE_TICKS)
concept={'idle':'idle','run':'movement','attack':'attack','skill1':'command','skill2':'command','skill3':'command','ult':'command','hit':'idle','dead':'idle'}
if complete:
    # Eight connected whole-body drawings per animation. A shared scale comes
    # from standing idle; wide gestures get padding, never a smaller body.
    sheets={p:Image.open(OUT/('animation-'+p+'-v1.png')).convert('RGBA') for p in POSE_TICKS};pieces={}
    for pose,src in sheets.items():
        pieces[pose]=[]
        for i in range(8):
            cell=src.crop((i%4*src.width//4,i//4*src.height//2,(i%4+1)*src.width//4,(i//4+1)*src.height//2));box=cell.getchannel('A').point(lambda a:255 if a>=220 else 0).getbbox()
            assert box,(pose,i);pieces[pose].append(cell.crop(box))
    scales={p:40/images[0].height for p,images in pieces.items()}
    for pose,images in pieces.items():
        body[pose]=[]
        for i,im in enumerate(images):
            im=binary(im.resize((round(im.width*scales[pose]),round(im.height*scales[pose])),Image.Resampling.NEAREST));assert im.width<=64 and im.height<=49,(pose,i,im.size)
            frame=Image.new('RGBA',(64,56));frame.alpha_composite(im,((64-im.width)//2,49-im.height));body[pose].append(frame)
    # Freeze the approved appearance's actual idle; it must remain visible in draft cards.
    frame=Image.new('RGBA',(64,56));frame.alpha_composite(Image.open(OUT/'body-idle.png'),(8,0));body['idle'][0]=frame
else:
    body={p:[Image.open(OUT/('body-'+c+'.png')).convert('RGBA')] for p,c in concept.items()}
body_anims={};body_manifest={};bw=512;bh=math.ceil(sum(len(v) for v in body.values())*2/8)*56;body_atlas=Image.new('RGBA',(bw,bh));idx=0
for left in (False,True):
    for pose,images in body.items():
        fs=[]
        for i,im in enumerate(images):
            if left:im=ImageOps.mirror(im)
            canvas=Image.new('RGBA',(64,56));canvas.alpha_composite(im,((64-im.width)//2,0));x,y=idx%8*64,idx//8*56;body_atlas.alpha_composite(canvas,(x,y));idx+=1
            fs.append({'duration':(POSE_TICKS[pose][i] if complete else sum(POSE_TICKS[pose]))/60,'data':{'x':x,'y':y,'w':64,'h':56}})
        tag=pose+('_l' if left else '');body_anims[tag]={'frames':fs};body_manifest[tag]=fs
body_atlas.save(OUT/'candidate-body.png')
images=[];rectangles=[];anims={};meta={};COLS=12;CELL=160
def add(tag,frames,ticks,kind='effect',z=6,follow=False):
    fs=[]
    for image,t in zip(frames,ticks):
        assert max(image.size)<=CELL,(tag,image.size)
        rect={'x':0,'y':0,'w':image.width,'h':image.height};rectangles.append(rect);images.append(image.copy())
        fs.append({'duration':t/60,'data':rect})
    anims[tag]={'frames':fs};meta[tag]={'frames':fs,'kind':kind,'z':z,'follow':follow}
def durations(total,n=8):return [total//n+(i<total%n) for i in range(n)]
for name in ('fighter','bomber','chopper','repair','drone','rocket'):
    for i in range(8):add(name+'_p'+str(i),[Image.open(OUT/(name+'-heading-'+str(i)+'.png')).convert('RGBA')],[1])
add('carrier',[Image.open(OUT/'carrier-native.png').convert('RGBA')],[1])
radius=lambda n:round(n*2/UPP)+2
blast=material('blast',(radius(RULES['bomber']['bombRadius']),radius(RULES['bomber']['bombRadius'])))
for total in (14,16,18):add('bomb_t'+str(total),blast,durations(total))
add('bomb',blast,durations(18))
add('kamikaze',material('blast',(radius(RULES['fighter']['kamikazeRadius']),radius(RULES['fighter']['kamikazeRadius']))),durations(24))
add('carrier_land',material('pressure',(radius(RULES['carrier']['knockRadius']),96)),durations(30))
add('knock_ring',geometric('knock_ring',(radius(RULES['carrier']['knockRadius']),)*2),durations(24),z=-2)
add('warn',geometric('warn',(radius(RULES['bomber']['orbitRadius']),)*2),durations(RULES['bomber']['warnTicks']),z=-2)
shadow=[]
for i in range(8):
    im=Image.new('RGBA',(150,150));d=ImageDraw.Draw(im);d.ellipse((2,2,147,147),outline=(160,168,181,160),width=1)
    ship=Image.open(OUT/'carrier-native.png').convert('RGBA');mask=ship.getchannel('A').point(lambda a:round(a*(.2+.6*i/7)));silhouette=Image.new('RGBA',ship.size,(8,15,24));silhouette.putalpha(mask);im.alpha_composite(silhouette,((150-ship.width)//2,(150-ship.height)//2))
    height=round(65*(1-i/8)**1.4);im.alpha_composite(ship,((150-ship.width)//2,(150-ship.height)//2-height));shadow.append(im)
add('carrier_shadow',shadow,durations(RULES['carrier']['warnTicks']),z=-2)
add('lock',geometric('lock',(24,24)),durations(RULES['fighter']['lockTicks']),follow=True)
add('mg',geometric('mg',(12,12)),[1]*8)
add('repair_kit',material('repairkit',(28,36)),durations(20))
add('repair_shield',geometric('shield',(38,48)),[6]*8,kind='buff',z=4)
add('burn',material('blast',(18,24)),[6]*8,kind='buff',z=4)
drone=Image.open(OUT/'drone-native.png').convert('RGBA');add('drone_up',[drone]*8,durations(18))
add('taunt',geometric('taunt',(radius(RULES['drone']['tauntRadius']),)*2),durations(30))
rivet=Image.new('RGBA',(9,3));ImageDraw.Draw(rivet).line((1,1,7,1),fill='#e2e8ed',width=1);add('rivet',[rivet],[4],kind='projectile',z=2)
for count in range(4):add('airframes'+str(count),[geometric('stock',(64,64),4)[count]],[60],kind='buff',z=7)
x=y=row_h=0
for i in sorted(range(len(images)),key=lambda i:(-images[i].height,-images[i].width,i)):
    im=images[i]
    if x+im.width>1920:y+=row_h;x=0;row_h=0
    rectangles[i].update(x=x,y=y);x+=im.width;row_h=max(row_h,im.height)
atlas=Image.new('RGBA',(1920,math.ceil((y+row_h)/64)*64));assert max(atlas.size)<=2048,atlas.size
for im,r in zip(images,rectangles):atlas.alpha_composite(im,(r['x'],r['y']))
atlas.save(OUT/'candidate-fx.png');decoded=(body_atlas.width*body_atlas.height+atlas.width*atlas.height)*4;assert decoded<=32*1024*1024
manifest={'schema':1,'art_revision':'wright-art-1','complete_body':complete,'review_only':True,'units_per_pixel':UPP,'canvas':[64,56],'world_anchor':[32,28],'ground_anchor':[32,48],'pose_ticks':POSE_TICKS,'body':body_manifest,'effects':meta,'decoded_bytes':decoded,'sources':{p.name:sha(p) for p in sorted(OUT.glob('*-source-v1.png'))}}
manifest['animation_sources']={p.name:sha(p) for p in sorted(OUT.glob('animation-*-v1.png'))}
if complete:manifest['extraction_scales']=scales
(OUT/'art-manifest.json').write_text(json.dumps(manifest,indent=2)+'\n');(ROOT/'editor/wright-art.js').write_text('window.WRIGHT_ART='+json.dumps(manifest,separators=(',',':'))+';\n')
rust=['//! Generated by tools/build-wright-art.py. Presentation only.',
      '#![allow(dead_code)]','pub const ART_REVISION: &str = "wright-art-1";',
      'pub fn frame(pose: &str, tick: usize, looping: bool) -> usize {',
      '    let durations: &[usize] = match pose {']
for pose,ticks in POSE_TICKS.items():rust.append('        '+json.dumps(pose)+' => &'+str([n*1000 for n in ticks])+',')
rust+=['        _ => &[9000;8],','    };','    let total: usize=durations.iter().sum();',
       '    let mut elapsed=if looping {tick.saturating_mul(1000)%total}else{tick.saturating_mul(1000).min(total-1)};',
       '    for (i,d) in durations.iter().enumerate(){if elapsed<*d{return i;}elapsed-=d;}','    durations.len()-1','}',
       'pub fn effect_name(name: &str,ticks: u64) -> String {',
       '    if name=="wrt_bomb" && matches!(ticks,14|16|18){format!("wrt_bomb_t{ticks}")}else{name.to_owned()}','}']
(ROOT/'native/tfm2_custom_ai/src/wright_art.rs').write_text('\n'.join(rust)+'\n')
font=ImageFont.truetype('C:/Windows/Fonts/segoeui.ttf',14)
tags=['bomb','kamikaze','carrier_land','knock_ring','warn','carrier_shadow','lock','mg','repair_kit','repair_shield','burn','drone_up','taunt','airframes3']
board=Image.new('RGB',(1200,math.ceil(len(tags)/3)*230),'#192b36');d=ImageDraw.Draw(board)
for i,tag in enumerate(tags):
    x,y=i%3*400,i//3*230;d.text((x+12,y+8),tag+' / native + enlarged',font=font,fill='#edc080');spec=meta[tag];f=spec['frames'][min(3,len(spec['frames'])-1)]['data'];im=atlas.crop((f['x'],f['y'],f['x']+f['w'],f['y']+f['h']));box=im.getbbox();im=im.crop(box) if box else im
    board.paste(im,(x+12,y+50),im);big=im.resize((im.width*2,im.height*2),Image.Resampling.NEAREST)
    if big.width>220 or big.height>180:big.thumbnail((220,180),Image.Resampling.NEAREST)
    board.paste(big,(x+165,y+40),big)
board.save(OUT/'effects-comparison.png')
if complete:
    contact=Image.new('RGB',(1200,len(body)*140+40),'#203747');d=ImageDraw.Draw(contact)
    d.text((12,8),'WRIGHT / complete whole-body poses / 2x / constant body scale',font=font,fill='#eef4fa')
    for row,(pose,frames) in enumerate(body.items()):
        d.text((12,40+row*140),pose,font=font,fill='#ffb65e');animation=[]
        for i,im in enumerate(frames):
            large=im.resize((128,112),Image.Resampling.NEAREST);contact.paste(large,(120+i*132,40+row*140),large)
            frame=Image.new('RGBA',(384,168),'#203747');ImageDraw.Draw(frame).rectangle((192,0,383,167),fill='#b6c4a4')
            frame.alpha_composite(im.resize((192,168),Image.Resampling.NEAREST),(0,0));frame.alpha_composite(ImageOps.mirror(im).resize((192,168),Image.Resampling.NEAREST),(192,0));animation.append(frame.convert('RGB'))
        animation[0].save(OUT/(pose+'.gif'),save_all=True,append_images=animation[1:],duration=[round(t*1000/60) for t in POSE_TICKS[pose]],loop=0)
    contact.save(OUT/'complete-poses.png')
if '--integrate' in sys.argv:
    approval=json.loads((OUT/'approval.json').read_text());assert complete
    for group in ('body','fleet','effects'):assert approval[group]['approved'],group+' not approved'
    for name,digest in manifest['sources'].items():assert approval['sources'][name]==digest,('unapproved source',name)
    manifest['review_only']=False
    dest=ROOT/'mods/tfm2_custom';shutil.copyfile(OUT/'candidate-body.png',dest/'champions/tfm2_custom_wright#sheet.png');(dest/'champions/tfm2_custom_wright#anim.fanim').write_text(json.dumps({'anims':body_anims},separators=(',',':'))+'\n')
    shutil.copyfile(OUT/'candidate-fx.png',dest/'vfx/wright#sheet.png');(dest/'vfx/wright#anim.fanim').write_text(json.dumps({'anims':anims},separators=(',',':'))+'\n')
    (OUT/'art-manifest.json').write_text(json.dumps(manifest,indent=2)+'\n');(ROOT/'editor/wright-art.js').write_text('window.WRIGHT_ART='+json.dumps(manifest,separators=(',',':'))+';\n')
print(json.dumps({'complete_body':complete,'fx_tags':len(meta),'body_atlas':body_atlas.size,'fx_atlas':atlas.size,'decoded_bytes':decoded,'integrated':'--integrate' in sys.argv}))
