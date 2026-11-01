"""Frozen imagegen proposals: mechanical extraction and packing, with an approval gate."""
from pathlib import Path
from PIL import Image,ImageDraw,ImageOps
from collections import deque
import argparse,hashlib,json,shutil
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/rally-art-review';OUT.mkdir(parents=True,exist_ok=True)
p=argparse.ArgumentParser();p.add_argument('--yellow',type=Path);p.add_argument('--thunder',type=Path);p.add_argument('--white',type=Path);p.add_argument('--integrate',action='store_true');p.add_argument('--preview-pack',action='store_true');args=p.parse_args()
poses={};sources={};records={};names=['idle','run','spike','receive','set']
for colour in ['yellow','thunder','white']:
 source=OUT/('body-'+colour+'-source.png')
 if getattr(args,colour):shutil.copyfile(getattr(args,colour),source)
 sources[colour]=hashlib.sha256(source.read_bytes()).hexdigest()
 im=Image.open(source).convert('RGBA');w,h=im.size;a=im.getchannel('A').tobytes();seen=bytearray(w*h);components=[]
 for start,alpha in enumerate(a):
  if alpha<128 or seen[start]:continue
  q=deque([start]);seen[start]=1;pixels=[]
  while q:
   n=q.popleft();x=n%w;y=n//w;pixels.append(n)
   for v in ([n-1]if x else[])+([n+1]if x+1<w else[])+([n-w]if y else[])+([n+w]if y+1<h else[]):
    if not seen[v]and a[v]>=128:seen[v]=1;q.append(v)
  if len(pixels)>5000:components.append(pixels)
 def box(c):return min(n%w for n in c),min(n//w for n in c),max(n%w for n in c)+1,max(n//w for n in c)+1
 components=sorted(sorted(components,key=len,reverse=True)[:10],key=lambda c:(box(c)[1]+box(c)[3])//2);assert len(components)==10
 rows=[sorted(components[:5],key=lambda c:box(c)[0]),sorted(components[5:],key=lambda c:box(c)[0])]
 for row,form in zip(rows,['masked','unmasked']):
  b=box(row[0]);scale=44/(b[3]-b[1])
  for name,pixels in zip(names,row):
   b=box(pixels);cut=Image.new('RGBA',(b[2]-b[0],b[3]-b[1]))
   for n in pixels:cut.putpixel((n%w-b[0],n//w-b[1]),im.getpixel((n%w,n//w))[:3]+(255,))
   cut=cut.resize((max(1,round(cut.width*scale)),max(1,round(cut.height*scale))),Image.Resampling.NEAREST);assert cut.width<=64 and cut.height<=52,(colour,form,name,cut.size)
   cut=cut.quantize(colors=32,method=Image.Quantize.FASTOCTREE).convert('RGBA');frame=Image.new('RGBA',(64,64));frame.alpha_composite(cut,((64-cut.width)//2,52-cut.height));key=f'{colour}_{form}_{name}';poses[key]=frame;frame.save(OUT/(key+'.png'));records[key]={'sourceBox':b,'sha256':hashlib.sha256(frame.tobytes()).hexdigest()}
   if colour=='yellow':frame.save(OUT/(form+'_'+name+'.png'))
combined=hashlib.sha256(json.dumps(sources,sort_keys=True).encode()).hexdigest()
(OUT/'source-manifest.json').write_text(json.dumps({'sourceSha256':combined,'sources':sources,'generator':'imagegen (not deterministic); extraction and packing deterministic','canvas':[64,64],'footBaseline':52,'poses':records},indent=2)+'\n')
review=Image.new('RGB',(1100,1080));d=ImageDraw.Draw(review)
for row,(colour,form)in enumerate([(c,f)for c in ['yellow','thunder','white']for f in ['masked','unmasked']]):
 for col,name in enumerate(names):
  x=col*220;y=row*180;bg,fg=('#203039','white')if row%2==0 else('#c8c1a8','#16252b');d.rectangle((x,y,x+219,y+179),fill=bg);d.text((x+8,y+5),f'{colour} {form} / {name}',fill=fg);frame=poses[f'{colour}_{form}_{name}'];review.paste(frame,(x+6,y+48),frame);z=frame.resize((128,128),Image.Resampling.NEAREST);review.paste(z,(x+78,y+28),z)
review.save(OUT/'body-contact-sheet.png')
frames=[]
for f in range(16):
 preview=Image.new('RGB',(960,160),'#203039'if f<8 else'#c8c1a8')
 for i,c in enumerate(['yellow','thunder','white']):
  for j,m in enumerate(['masked','unmasked']):
   image=poses[f'{c}_{m}_{names[(f//2)%5]}']
   if f>=8:image=ImageOps.mirror(image)
   preview.paste(image,(i*320+j*150+10,55),image)
 frames.append(preview)
frames[0].save(OUT/'body-animation.gif',save_all=True,append_images=frames[1:],duration=160,loop=0,disposal=2)
if args.integrate or args.preview_pack:
 if args.integrate:
  approval=json.loads((OUT/'approval.json').read_text());assert approval.get('bodySourceSha256')==combined,'Approve this exact proposal first'
 unique=[];dedup={};anims={}
 def add(tag,image,duration=.1,count=8):
  k=image.size,image.tobytes()
  if k not in dedup:dedup[k]=len(unique);unique.append(image)
  idx=dedup[k];anims[tag]={'frames':[{'data':{'x':idx%16*64,'y':idx//16*64,'w':image.width,'h':image.height},'duration':duration}for _ in range(count)]}
 add('air_hidden',Image.new('RGBA',(64,64)),1/60)
 for colour in ['yellow','thunder','white']:
  for form in ['masked','unmasked']:
   for facing in ['r','l']:
    for name,srcname in [('idle','idle'),('run','run'),('attack','receive'),('skill','receive'),('skill1','receive'),('skill2','set'),('ult','set'),('hit','receive'),('receive','receive'),('dead','receive'),('death','receive'),('approach','run'),('spike','spike'),('set','set')]:
     image=poses[f'{colour}_{form}_{srcname}']
     if facing=='l':image=ImageOps.mirror(image)
     add(f'{colour}_{form}_{name}_{facing}',image)
     if colour=='yellow'and form=='masked'and facing=='r':add(name,image)
     if colour=='yellow'and form=='masked'and facing=='l':add(name+'_l',image)
    for phase in range(8):
     for action,srcname in [('air','spike'),('helper','set')]:
      image=poses[f'{colour}_{form}_{srcname}']
      if facing=='l':image=ImageOps.mirror(image)
      add(f'{action}_{colour}_{form}_{facing}_p{phase}',image,2/60,1)
 sheet=Image.new('RGBA',(1024,((len(unique)+15)//16)*64));assert sheet.height<=2048,sheet.size
 for i,image in enumerate(unique):sheet.alpha_composite(image,(i%16*64,i//16*64))
 target=ROOT/'mods/tfm2_custom/champions'if args.integrate else OUT/'packed-preview/champions';target.mkdir(parents=True,exist_ok=True);sheet.save(target/'tfm2_custom_rally#sheet.png');(target/'tfm2_custom_rally#anim.fanim').write_text(json.dumps({'anims':anims},separators=(',',':'))+'\n')
print('Prepared six-form review'+('; approved bodies packed'if args.integrate else'; not integrated'))
