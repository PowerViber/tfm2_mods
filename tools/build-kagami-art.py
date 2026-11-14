"""Reproducible packing from frozen approved imagegen materials (generation is not deterministic)."""
from pathlib import Path
from PIL import Image,ImageOps,ImageDraw
import json,hashlib,math,shutil
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/kagami-art-review';N=Image.Resampling.NEAREST
APP=json.loads((OUT/'approvals.json').read_text())
assert APP['body']['approved'] and APP['effects']['approved'],'Artwork approval required'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def cells(name,cols,rows):
 im=Image.open(OUT/name).convert('RGBA');result=[]
 for i in range(cols*rows):
  f=im.crop((i%cols*im.width//cols,i//cols*im.height//rows,(i%cols+1)*im.width//cols,(i//cols+1)*im.height//rows))
  f.putalpha(f.getchannel('A').point(lambda a:255 if a>=220 else 0));assert f.getbbox(),(name,i);result.append(f.crop(f.getbbox()))
 return result
concept=cells('body-source-v3.png',4,2);anim=cells('animation-source-v1.png',6,4)
# One palette for every pose. Opaque ink survives nearest-neighbour extraction.
palette=Image.open(OUT/'body-source-v3.png').convert('RGB').quantize(colors=16,method=Image.Quantize.MEDIANCUT,dither=Image.Dither.NONE)
def pixel(f,scale):
 f=f.resize((max(1,round(f.width*scale)),max(1,round(f.height*scale))),N)
 alpha=f.getchannel('A');f=f.convert('RGB').quantize(palette=palette,dither=Image.Dither.NONE).convert('RGBA');f.putalpha(alpha)
 assert f.width<=62 and f.height<=51,f.size
 p=Image.new('RGBA',(64,64));p.alpha_composite(f,((64-f.width)//2,52-f.height));return p
c=[pixel(f,44/concept[0].height) for f in concept];a=[pixel(f,44/anim[0].height) for f in anim]
poses={'idle':[c[0]]*4,'run':a[:6],'attack':a[6:10],'skill1':a[12:16],
       'skill2':a[16:20],'ult':a[16:20],'hit':[a[10]],'dead':[a[11],*a[20:24],c[7]]}
ticks={'idle':[18]*4,'run':[8]*6,'attack':[4,5,3,10],'skill1':[4,5,5,4],'skill2':[4,5,5,4],'ult':[4,5,5,4],'hit':[8],'dead':[8]*6}
def pack(stem,anims,width=1024):
 unique={};x=y=row=0;meta={}
 for name,seq in anims.items():
  fs=[]
  for im,t in seq:
   key=(im.size,im.tobytes())
   if key not in unique:
    if x+im.width>width:x=0;y+=row+2;row=0
    rect={'x':x,'y':y,'w':im.width,'h':im.height};unique[key]=(im,rect);x+=im.width+2;row=max(row,im.height)
   fs.append({'duration':t/60,'data':unique[key][1]})
  meta[name]={'frames':fs}
 h=2**math.ceil(math.log2(max(2,y+row+2)));assert h<=2048 and width<=2048
 sheet=Image.new('RGBA',(width,h))
 for im,r in unique.values():sheet.alpha_composite(im,(r['x'],r['y']))
 sheet.save(OUT/(stem+'.png'));(OUT/(stem+'.fanim')).write_text(json.dumps({'anims':meta},indent=2)+'\n')
 return sheet,meta
body={}
for pose,fs in poses.items():
 for left in [False,True]:body[pose+('_l' if left else '')]=[(ImageOps.mirror(f) if left else f,t) for f,t in zip(fs,ticks[pose])]
body_sheet,body_anims=pack('body',body)
board=Image.new('RGB',(1120,len(poses)*220));draw=ImageDraw.Draw(board)
for i,(pose,fs) in enumerate(poses.items()):
 for col,bg in enumerate(['#203747','#bec9b0']):
  x=col*560;y=i*220;draw.rectangle((x,y,x+559,y+219),fill=bg);draw.text((x+8,y+4),pose+' | packed pixels | 1x + 3x | both facings',fill='#f8edff' if not col else '#24223a')
  for j,f in enumerate([fs[0],ImageOps.mirror(fs[0])]):
   board.paste(f,(x+8+j*66,y+110),f);large=f.resize((192,192),N);board.paste(large,(x+154+j*195,y+18),large)
board.save(OUT/'body-comparison.png')
# Mirror presentation follows a known, targetable base carrier. No global sprite override.
def tint(im):
 f=im.copy();p=f.load()
 for y in range(f.height):
  for x in range(f.width):
   r,g,b,alpha=p[x,y]
   if alpha:
    lum=(r+g+b)//3
    p[x,y]=((12,20,38,215) if lum<45 else (min(255,lum//2+75),min(255,lum//2+90),min(255,lum//2+125),185))
 return f
fx={}
for pose,fs in poses.items():
 for left in [False,True]:
  for i,f in enumerate(fs):
   im=tint(ImageOps.mirror(f) if left else f);fx['clone_'+pose+'_'+str(i)+('_l' if left else '')]=[(im,3)]
materials=cells('effects-source-v1.png',4,3)
defs={'shard':(0,14,8,12),'shard_hit':(1,24,24,12),'thread':(2,64,12,4),'swap':(3,40,40,24),'stun_burst':(4,100,100,30),'clone_in':(5,38,52,24),'heal':(6,32,34,30),'charm':(7,18,18,24),'angel':(8,100,100,36),'seal':(9,48,72,150),'revive':(10,58,58,30),'revived':(11,38,54,240)}
for name,(idx,w,h,total) in defs.items():
 material=materials[idx].resize((w,h),N).quantize(colors=12,method=Image.Quantize.FASTOCTREE,dither=Image.Dither.NONE).convert('RGBA')
 seq=[]
 for i in range(4):
  scale=([.94,1,.97,1] if name in ['seal','revived','charm','shard','thread'] else [.7,.9,1,.8])[i]
  f=material.resize((max(1,round(w*scale)),max(1,round(h*scale))),N);p=Image.new('RGBA',(w+4,h+4));p.alpha_composite(f,((p.width-f.width)//2,(p.height-f.height)//2));seq.append((p,total//4+(i<total%4)))
 fx[name]=seq
 # Persistent leases choose an immutable birth-clock phase; renewals never restart a sequence.
 if name in ['thread','seal']:
  for i,(f,_) in enumerate(seq):fx[name+'_p'+str(i)]=[(f,3)]
fx['sealed']=fx['seal']
fx_sheet,fx_anims=pack('effects',fx,1024)
board=Image.new('RGB',(1120,6*340));draw=ImageDraw.Draw(board)
for i,name in enumerate(defs):
 x=i%2*560;y=i//2*340;draw.rectangle((x,y,x+559,y+339),fill='#203747' if i%2==0 else '#bec9b0');draw.text((x+8,y+5),name+' | packed pixels | 1x + 3x',fill='#ece2ff' if i%2==0 else '#24223a')
 r=fx_anims[name]['frames'][2]['data'];f=fx_sheet.crop((r['x'],r['y'],r['x']+r['w'],r['y']+r['h']));board.paste(f,(x+10,y+90),f);large=f.resize((f.width*3,f.height*3),N);board.paste(large,(x+160,y+22),large)
board.save(OUT/'effects-comparison.png')
art={'version':1,'revision':'kagami-art-1','approved':True,'foot_anchor':[32,52],
     'pose_ticks':ticks,'body_anims':body_anims,'effects':fx_anims,'source_sha256':{n:sha(OUT/n) for n in ['body-source-v3.png','animation-source-v1.png','effects-source-v1.png']},
     'native_clone':'Known base priest carrier; lifetime cosmetic animation mask, no Bind; following single-frame leases',
     'decoded_bytes':(body_sheet.width*body_sheet.height+fx_sheet.width*fx_sheet.height)*4}
assert art['decoded_bytes']<=32*1024*1024
(ROOT/'rules/kagami-art.json').write_text(json.dumps(art,indent=2)+'\n')
for name,dest in [('body','mods/tfm2_custom/champions/tfm2_custom_kagami'),('effects','mods/tfm2_custom/vfx/kagami')]:
 for ext,suffix in [('png','#sheet.png'),('fanim','#anim.fanim')]:shutil.copy2(OUT/(name+'.'+ext),ROOT/(dest+suffix))
# Generate the native frame clock from the same manifest used by the Lab.
lines=['//! Generated by tools/build-kagami-art.py. Presentation only.','pub const RENEW: usize = 3;','pub fn frame(pose: &str, elapsed: usize) -> usize {','    let ticks: &[usize] = match pose {']
for pose,ts in ticks.items():lines.append('        "'+pose+'" => &'+str(ts)+',')
lines+=['        _ => &[18,18,18,18],','    };','    let total:usize=ticks.iter().sum();','    let mut time=if matches!(pose,"idle"|"run") {elapsed%total} else {elapsed.min(total-1)};','    for (i,t) in ticks.iter().enumerate(){if time<*t{return i;}time-=t;}','    ticks.len()-1','}','']
(ROOT/'native/tfm2_custom_ai/src/kagami_art.rs').write_text('\n'.join(lines))
(ROOT/'editor/kagami-art.js').write_text('window.KAGAMI_ART='+json.dumps(art,separators=(',',':'))+';\n')
# Whole-body GIFs show real distinct walking/casting drawings, never sliced anatomy.
for pose,fs in poses.items():
 frames=[]
 for f in fs:
  b=Image.new('RGB',(520,220),'#203747')
  for x,im in [(12,f),(80,ImageOps.mirror(f))]:b.paste(im,(x,100),im)
  for x,im in [(155,f),(345,ImageOps.mirror(f))]:
   im=im.resize((192,192),N);b.paste(im,(x,15),im)
  frames.append(b)
 frames[0].save(OUT/(pose+'.gif'),save_all=True,append_images=frames[1:],duration=[max(20,t*1000//60) for t in ticks[pose]],loop=0,disposal=2)
report={'approved':True,'atlases':{n:{'size':list(Image.open(OUT/(n+'.png')).size),'sha256':sha(OUT/(n+'.png'))} for n in ['body','effects']},'decoded_bytes':art['decoded_bytes'],'source_sha256':art['source_sha256'],'native_game':'pending','generation':'AI generation not deterministic; frozen source extraction and packing reproducible'}
(OUT/'art-verification.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report))
