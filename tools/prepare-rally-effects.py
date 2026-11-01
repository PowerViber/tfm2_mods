"""Extract approved-image effects mechanically; previews do not enter mods without approval."""
from pathlib import Path
from PIL import Image,ImageDraw,ImageChops,ImageOps
import argparse,hashlib,json,shutil
ROOT=Path(__file__).resolve().parents[1];RULES=json.loads((ROOT/'rules/rally.json').read_text(encoding='utf8'));OUT=ROOT/'docs/rally-art-review';OUT.mkdir(parents=True,exist_ok=True)
p=argparse.ArgumentParser();p.add_argument('--source',type=Path);p.add_argument('--thunder-source',type=Path);p.add_argument('--integrate',action='store_true');p.add_argument('--preview-pack',action='store_true');a=p.parse_args();source=OUT/'effects-source.png'
if a.source:shutil.copyfile(a.source,source)
newsource=OUT/'effects-thunder-source.png'
if a.thunder_source:shutil.copyfile(a.thunder_source,newsource)
newim=Image.open(newsource).convert('RGBA') if newsource.exists() else None
im=Image.open(source).convert('RGBA');names=['ball','receive','set','support','body_thunder','thunder','trail_thunder','impact','body_black','black','trail_black','black_impact'];sizes=[16,40,56,24,56,32,56,48,60,36,60,56];tiles={};records={}
# Frozen source cells have unequal spacing; explicit bounds avoid neighbouring fragments.
boxes=[(40,45,275,290),(310,45,665,300),(685,40,1200,300),(1205,45,1536,305),(0,305,345,628),(350,330,665,635),(680,340,1225,620),(1230,320,1536,635),(0,635,345,990),(350,670,665,990),(680,675,1230,990),(1230,650,1536,1000)]
for i,(name,size) in enumerate(zip(names,sizes)):
 box=boxes[i];src=im
 if i>=4 and newim:
  src=newim
  ni=i-4;row=ni//4;col=ni%4
  bx=[(0,370),(380,665),(680,1080),(1190,1536)][col];by=(0,510)if row==0 else(515,1024)
  box=(bx[0],by[0],bx[1],by[1])
 cut=src.crop(box);alpha=cut.getchannel('A').point(lambda n:255 if n>=128 else 0);bbox=alpha.getbbox();assert bbox,name;cut=cut.crop(bbox);cut.putalpha(cut.getchannel('A').point(lambda n:n if n>=128 else 0));scale=size/max(cut.size);cut=cut.resize((max(1,round(cut.width*scale)),max(1,round(cut.height*scale))),Image.Resampling.NEAREST);frame=Image.new('RGBA',(64,64));frame.alpha_composite(cut,((64-cut.width)//2,(64-cut.height)//2));tiles[name]=frame;frame.save(OUT/('effect-'+name+'.png'));records[name]={'cell':box,'crop':bbox,'size':cut.size,'sha256':hashlib.sha256(frame.tobytes()).hexdigest()}
white_source=OUT/'effects-white-source.png';white_im=Image.open(white_source).convert('RGBA')
for i,name in enumerate(['body_white','white','trail_white','white_impact','white_blink','crater']):
 box=(i%3*white_im.width//3,i//3*white_im.height//2,(i%3+1)*white_im.width//3,(i//3+1)*white_im.height//2)
 cut=white_im.crop(box);alpha=cut.getchannel('A').point(lambda n:255 if n>=128 else 0);bbox=alpha.getbbox();assert bbox,name
 cut=cut.crop(bbox);cut.putalpha(cut.getchannel('A').point(lambda n:n if n>=128 else 0))
 scale=(60 if name!='crater' else 126)/max(cut.size);cut=cut.resize((max(1,round(cut.width*scale)),max(1,round(cut.height*scale))),Image.Resampling.NEAREST)
 side=136 if name=='crater' else 64;tile=Image.new('RGBA',(side,side));tile.alpha_composite(cut,((side-cut.width)//2,(side-cut.height)//2))
 tiles[name]=tile;records[name]={'cell':box,'crop':bbox,'size':tile.size,'sha256':hashlib.sha256(tile.tobytes()).hexdigest()}
for name in list(tiles):
 tile=tiles[name]
 if name.startswith('body_')or name.startswith('trail_'):tile=tile.resize((192,192),Image.Resampling.NEAREST)
 elif name in ['impact','black_impact','white_impact']:
  bolt=tile.crop((0,0,64,50)).resize((256,232),Image.Resampling.NEAREST);ring_width=round(2*RULES['constants']['BLACK_RADIUS' if name!='impact' else 'SPIKE_RADIUS']/RULES['constants']['WORLD_UNITS_PER_PIXEL']);ring=tile.crop((0,50,64,64)).resize((ring_width,round(ring_width*9/40)),Image.Resampling.NEAREST)
  tile=Image.new('RGBA',(256,2*232+ring.height));tile.alpha_composite(bolt,(0,0));tile.alpha_composite(ring,((256-ring.width)//2,232))
 if name in ['impact','black_impact','white_impact']:tile=tile.crop((0,0,tile.width,tile.height-212))
 tiles[name]=tile;tile.save(OUT/('effect-'+name+'.png'));records[name]['size']=tile.size;records[name]['sha256']=hashlib.sha256(tile.tobytes()).hexdigest()
d=ImageDraw.Draw(tiles['crater']);side=tiles['crater'].width;d.ellipse((0,0,side-1,side-1),outline='#b27487ff',width=1);tiles['crater'].save(OUT/'effect-crater.png');records['crater']['sha256']=hashlib.sha256(tiles['crater'].tobytes()).hexdigest()
side=round(tiles['crater'].width*RULES['constants']['SERVE_RADIUS']/RULES['constants']['BLACK_RADIUS']);tiles['crater_serve']=tiles['crater'].resize((side,side),Image.Resampling.NEAREST);tiles['crater_serve'].save(OUT/'effect-crater_serve.png')
side=round(2*RULES['constants']['SERVE_RADIUS']/RULES['constants']['WORLD_UNITS_PER_PIXEL'])+8;center=side//2;ring=Image.new('RGBA',(side,side));d=ImageDraw.Draw(ring);d.ellipse((4,4,side-4,side-4),outline='#fff0c2ff',width=2);tiles['serve_ring']=ring;ring.save(OUT/'effect-serve_ring.png')
review=Image.new('RGB',(1440,1110));d=ImageDraw.Draw(review)
for i,name in enumerate(names):
 x=i%4*360;y=i//4*370;d.rectangle((x,y,x+359,y+369),fill='#24333c'if i<4 else'#bdb6a2');d.text((x+8,y+6),name+' (game pixels)',fill='white'if i<4 else'#10232a');frame=tiles[name];review.paste(frame,(x+(360-frame.width)//2,y+32),frame)
review.save(OUT/'effect-contact-sheet.png')
stroke_source=OUT/'effects-strokes-source.png';stroke_im=Image.open(stroke_source).convert('RGBA');stroke_names=['stroke_contact','stroke_impact','stroke_black_contact','stroke_black_impact']
for i,name in enumerate(stroke_names):
 box=(i%2*stroke_im.width//2,i//2*stroke_im.height//2,(i%2+1)*stroke_im.width//2,(i//2+1)*stroke_im.height//2);cut=stroke_im.crop(box);alpha=cut.getchannel('A').point(lambda a:255 if a>=128 else 0);bbox=alpha.getbbox();assert bbox;cut=cut.crop(bbox);cut.putalpha(cut.getchannel('A').point(lambda a:a if a>=128 else 0));scale=300/max(cut.size);cut=cut.resize((round(cut.width*scale),round(cut.height*scale)),Image.Resampling.NEAREST);tile=Image.new('RGBA',(320,320));tile.alpha_composite(cut,((320-cut.width)//2,(320-cut.height)//2));tiles[name]=tile;tile.save(OUT/('effect-'+name+'.png'));records[name]={'sourceCell':box,'size':tile.size,'sha256':hashlib.sha256(tile.tobytes()).hexdigest()}
manifest={'sourceSha256':hashlib.sha256(white_source.read_bytes()+source.read_bytes()+(newsource.read_bytes()if newsource.exists()else b'')+stroke_source.read_bytes()).hexdigest(),'sources':{'base':hashlib.sha256(source.read_bytes()).hexdigest(),'thunder':hashlib.sha256(newsource.read_bytes()).hexdigest()if newsource.exists()else None},'frames':records,'whiteSourceSha256':hashlib.sha256(white_source.read_bytes()).hexdigest(),'strokesSourceSha256':hashlib.sha256(stroke_source.read_bytes()).hexdigest()}
manifest['renderSha256']=hashlib.sha256(json.dumps(records,sort_keys=True).encode()).hexdigest()
(OUT/'effects-manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
def phases(name,tile):
 result=[]
 for phase in range(8):
  image=tile.copy()
  if name.startswith('stroke_'):
   image.putalpha(image.getchannel('A').point(lambda a:a*[255,255,210,180,120,80,40,0][phase]//255))
  elif name.startswith('body_'):image=ImageChops.offset(image,0,-phase*2)
  elif name.startswith('trail_')and phase%2:image=ImageOps.flip(image)
  elif name in ['impact','black_impact','white_impact']:
   full=Image.new('RGBA',(tile.width,tile.height+212));full.alpha_composite(tile,(0,0));scale=.4+.6*phase/7;small=full.resize((round(full.width*scale),round(full.height*scale)),Image.Resampling.NEAREST);image=Image.new('RGBA',full.size);anchor=full.height//2+20;image.alpha_composite(small,((tile.width-small.width)//2,round(anchor*(1-scale))));image.putalpha(image.getchannel('A').point(lambda v:v*max(0,255-phase*25)//255));assert not image.getchannel('A').crop((0,tile.height,full.width,full.height)).getbbox();image=image.crop((0,0,tile.width,tile.height))
  result.append(image)
 return result
for name,tile in tiles.items():
 for n,image in enumerate(phases(name,tile)):image.save(OUT/f'effect-{name}_p{n}.png')
if a.integrate or a.preview_pack:
 if a.integrate:
  approval=json.loads((OUT/'approval.json').read_text());assert approval.get('effectsSourceSha256')==manifest['sourceSha256'],'Approve these exact effects first'
 frames=[];anims={};dedup={}
 def add(tag,images,duration=.1):
  refs=[]
  for image in images:
   key=(image.size,image.tobytes())
   if key not in dedup:dedup[key]=len(frames);frames.append(image)
   i=dedup[key];refs.append({'data':{'index':i,'w':image.width,'h':image.height},'duration':duration})
  anims[tag]={'frames':refs}
 # Existing approved body used as a brief visual-only setter echo, never a summon.
 tiles['helper']=Image.open(OUT/'yellow_masked_set.png').convert('RGBA')
 for name,key in [('warning','SPIKE_RADIUS'),('warning_black','BLACK_RADIUS'),('warning_serve','SERVE_RADIUS')]:
  radius=round(RULES['constants'][key]/RULES['constants']['WORLD_UNITS_PER_PIXEL']);side=2*radius+8;center=side//2;warning=Image.new('RGBA',(side,side));d=ImageDraw.Draw(warning)
  d.ellipse((center-radius,center-radius,center+radius,center+radius),outline='#ffb362ff',width=2);d.ellipse((center-6,center-6,center+6,center+6),outline='#ff825bff',width=1);tiles[name]=warning
 edge=Image.new('RGBA',(8,8));ImageDraw.Draw(edge).polygon([(4,0),(7,4),(4,7),(0,4)],fill='#f4df9bff');tiles['serve_edge']=edge
 for name,tile in tiles.items():
  if name.startswith('stroke_') or name in ['body_white','white','trail_white','white_impact','white_blink','crater','crater_serve']:continue
  images=phases(name,tile);add(name,images,2/60)
  for phase,image in enumerate(images):add(name+'_p'+str(phase),[image],2/60)
 for name in ['thunder','black','trail_thunder','trail_black']:
  for heading in range(8):
   for phase,image in enumerate(phases(name,tiles[name])):
    rot=image.rotate(-heading*45,resample=Image.Resampling.NEAREST);add(f'{name}_a{heading}_p{phase}',[rot],2/60)
 # Existing-style code-native resource bars, pips and timing UI are presentation geometry, not new body art.
 for value in range(11):
  image=Image.new('RGBA',(64,64));d=ImageDraw.Draw(image);d.rectangle((18,0,45,4),fill='#07131cff');d.rectangle((19,1,19+round(25*value/10),3),fill='#5ee7ffff');add('flow'+str(value),[image]*8)
 for value in range(4):
  image=Image.new('RGBA',(64,64));d=ImageDraw.Draw(image)
  for k in range(3):d.rectangle((23+k*6,6,26+k*6,9),fill='#fff0a8ff'if k<value else'#172e3cff')
  add('charges'+str(value),[image]*8)
 for value,color in enumerate(['#71e8ff','#eacf8e','#c5a0fa']):
  image=Image.new('RGBA',(64,64));ImageDraw.Draw(image).rectangle((28,11,35,13),fill=color);add('role'+str(value),[image]*8)
 for height in range(2000,16001,100):
  label=Image.new('RGBA',(64,16));d=ImageDraw.Draw(label);d.rectangle((11,1,53,14),fill='#10212bcf');d.text((14,2),f'{height/1000:.1f} m',fill='#fff4bbff');add(f'height_{height}',[label],2/60);add(f'contact_height_{height}',[label]*8,.1)
 # Pack equal-height rows first; references retain each frame's own canvas/anchor.
 positions={};x=y=row_height=0
 for i in sorted(range(len(frames)),key=lambda i:(-frames[i].height,-frames[i].width,i)):
  image=frames[i]
  if x+image.width>2048:x=0;y+=row_height;row_height=0
  positions[i]=(x,y);x+=image.width;row_height=max(row_height,image.height)
 sheet=Image.new('RGBA',(2048,y+row_height));assert sheet.height<=2048,sheet.size
 for i,image in enumerate(frames):sheet.alpha_composite(image,positions[i])
 for anim in anims.values():
  for frame in anim['frames']:
   data=frame['data'];i=data.pop('index');data.update(x=positions[i][0],y=positions[i][1])
 target=ROOT/'mods/tfm2_custom/vfx'if a.integrate else OUT/'packed-preview/vfx';target.mkdir(parents=True,exist_ok=True);sheet.save(target/'rally#sheet.png');(target/'rally#anim.fanim').write_text(json.dumps({'anims':anims},separators=(',',':'))+'\n')
if a.integrate or a.preview_pack:
 stroke_frames=[];stroke_anims={};stroke_dedup={}
 for name in stroke_names:
  refs=[]
  for image in phases(name,tiles[name]):
   if not image.getchannel('A').getbbox():image=Image.new('RGBA',image.size)
   k=(image.size,image.tobytes())
   if k not in stroke_dedup:stroke_dedup[k]=len(stroke_frames);stroke_frames.append(image)
   i=stroke_dedup[k];refs.append({'data':{'index':i,'w':320,'h':320},'duration':2/60})
  stroke_anims[name]={'frames':refs}
  for phase,ref in enumerate(refs):stroke_anims[name+'_p'+str(phase)]={'frames':[ref]}
 import math
 cols=math.ceil(math.sqrt(len(stroke_frames)));stroke_sheet=Image.new('RGBA',(cols*320,math.ceil(len(stroke_frames)/cols)*320))
 for i,image in enumerate(stroke_frames):stroke_sheet.alpha_composite(image,(i%cols*320,i//cols*320))
 for anim in stroke_anims.values():
  for ref in anim['frames']:
   d=ref['data']
   if 'index' in d:
    i=d.pop('index');d.update(x=i%cols*320,y=i//cols*320)
 assert max(stroke_sheet.size)<=2048
 stroke_sheet.save(target/'rally-strokes#sheet.png');(target/'rally-strokes#anim.fanim').write_text(json.dumps({'anims':stroke_anims},separators=(',',':'))+'\n')
 frames=[];anims={};dedup={}
 for name in ['body_white','white','trail_white','white_impact','white_blink','crater','crater_serve']:
  images=phases(name,tiles[name]);add(name,images,2/60)
  for phase,image in enumerate(images):add(name+'_p'+str(phase),[image],2/60)
 for name in ['white','trail_white']:
  for heading in range(8):
   for phase,image in enumerate(phases(name,tiles[name])):
    add(f'{name}_a{heading}_p{phase}',[image.rotate(-heading*45,resample=Image.Resampling.NEAREST)],2/60)
 positions={};x=y=row_height=0
 for i in sorted(range(len(frames)),key=lambda i:(-frames[i].height,-frames[i].width,i)):
  image=frames[i]
  if x+image.width>2048:x=0;y+=row_height;row_height=0
  positions[i]=(x,y);x+=image.width;row_height=max(row_height,image.height)
 sheet=Image.new('RGBA',(2048,y+row_height));assert sheet.height<=2048
 for i,image in enumerate(frames):sheet.alpha_composite(image,positions[i])
 for anim in anims.values():
  for frame in anim['frames']:
   data=frame['data'];i=data.pop('index');data.update(x=positions[i][0],y=positions[i][1])
 sheet.save(target/'rally-white#sheet.png');(target/'rally-white#anim.fanim').write_text(json.dumps({'anims':anims},separators=(',',':'))+'\n')
print('Rally effect review prepared'+('; approved effects packed'if a.integrate else '; not integrated'))
