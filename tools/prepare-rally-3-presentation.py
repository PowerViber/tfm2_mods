"""Preview-only extension of frozen Rally 2.2 pixels. Never writes shipped assets."""
from pathlib import Path
from PIL import Image,ImageDraw
import hashlib,json,shutil
from rally_art_packing import pack
ROOT=Path(__file__).resolve().parents[1]
rules=json.loads((ROOT/'rules/rally.json').read_text())
assert rules['revision'] in ['rally-3.0','rally-3.1','rally-3.2','rally-3.3']
old=ROOT/'docs/rally-art-review/packed-preview/vfx'
out=ROOT/'docs/rally-results'/rules['revision']/'packed-preview/vfx'
out.mkdir(parents=True,exist_ok=True)
sheet=Image.open(old/'rally#sheet.png').convert('RGBA')
original=json.loads((old/'rally#anim.fanim').read_text())['anims']
frames=[];dedup={};anims={}
def add(tag,images,durations):
 refs=[]
 for image,duration in zip(images,durations):
  key=(image.size,image.tobytes())
  if key not in dedup:dedup[key]=len(frames);frames.append(image)
  refs.append({'data':{'index':dedup[key],'w':image.width,'h':image.height},'duration':duration})
 anims[tag]={'frames':refs}
for tag,animation in original.items():
 if tag.startswith(('flow','charges','role')):continue
 images=[]
 for f in animation['frames']:
  d=f['data'];images.append(sheet.crop((d['x'],d['y'],d['x']+d['w'],d['y']+d['h'])))
 add(tag,images,[f['duration'] for f in animation['frames']])
for n in range(21):
 image=Image.new('RGBA',(64,64));draw=ImageDraw.Draw(image)
 draw.rectangle((12,0,51,5),fill='#07131cff');draw.rectangle((13,1,50,4),fill='#253844ff')
 if n:draw.rectangle((13,1,13+round(37*n/20),4),fill='#ffe16aff')
 if rules['revision'] in ['rally-3.2','rally-3.3']:
  for x in [25,38]:draw.line((x,1,x,4),fill='#07131cff',width=1)
 add('pace'+str(n),[image]*8,[.1]*8)
for n,name in enumerate(['Spiker','Setter','Receiver']):
 image=Image.new('RGBA',(64,64));draw=ImageDraw.Draw(image)
 if rules['revision'] not in ['rally-3.2','rally-3.3']:
  box=draw.textbbox((0,0),name);x=(64-(box[2]-box[0]))//2
  draw.rectangle((x-2,7,x+box[2]+2,20),fill='#07131cdd');draw.text((x,7),name,fill=['#ffe16a','#b9edff','#d6c4ff'][n])
 add('role'+str(n),[image]*8,[.1]*8)
for height in range(16100,rules['constants']['MAX_HEIGHT_LABEL']+1,100):
 image=Image.new('RGBA',(64,16));draw=ImageDraw.Draw(image)
 draw.rectangle((11,1,53,14),fill='#10212bcf');draw.text((14,2),f'{height/1000:.1f} m',fill='#fff4bbff')
 add('height_'+str(height),[image],[2/60])
 add('contact_height_'+str(height),[image]*8,[.1]*8)
size,positions=pack(frames)
packed=Image.new('RGBA',size);assert packed.height<=2048
for i,im in enumerate(frames):packed.alpha_composite(im,positions[i])
for animation in anims.values():
 for f in animation['frames']:
  d=f['data'];i=d.pop('index');d.update(x=positions[i][0],y=positions[i][1])
# Frozen effects, anchors, durations and all old height labels must be pixel-identical.
compared=0
for tag,animation in original.items():
 if tag.startswith(('flow','charges','role')):continue
 actual=anims[tag]['frames'];assert len(actual)==len(animation['frames'])
 for a,b in zip(animation['frames'],actual):
  da,db=a['data'],b['data']
  assert (da['w'],da['h'],a['duration'])==(db['w'],db['h'],b['duration'])
  ia=sheet.crop((da['x'],da['y'],da['x']+da['w'],da['y']+da['h']))
  ib=packed.crop((db['x'],db['y'],db['x']+db['w'],db['y']+db['h']))
  assert ia.tobytes()==ib.tobytes(),tag
  compared+=1
packed.save(out/'rally#sheet.png')
(out/'rally#anim.fanim').write_text(json.dumps({'anims':anims},separators=(',',':'))+'\n')
for stem in ['rally-strokes','rally-white']:
 for suffix in ['#sheet.png','#anim.fanim']:shutil.copyfile(old/(stem+suffix),out/(stem+suffix))
if rules['revision'] in ['rally-3.2','rally-3.3']:
 # Repack the existing White atlas without changing a frame, anchor or duration.
 source=Image.open(old/'rally-white#sheet.png').convert('RGBA')
 meta=json.loads((old/'rally-white#anim.fanim').read_text());images=[];locations={}
 for anim in meta['anims'].values():
  for f in anim['frames']:
   d=f['data'];key=(d['x'],d['y'],d['w'],d['h'])
   if key not in locations:
    locations[key]=len(images);images.append(source.crop((d['x'],d['y'],d['x']+d['w'],d['y']+d['h'])))
 size,spots=min((pack(images,w) for w in ([960,1024,1280,1536,1792,2048] if rules['revision']=='rally-3.3' else [2048])),key=lambda item:item[0][0]*item[0][1]);white=Image.new('RGBA',size)
 for i,im in enumerate(images):white.alpha_composite(im,spots[i])
 for anim in meta['anims'].values():
  for f in anim['frames']:
   d=f['data'];key=(d['x'],d['y'],d['w'],d['h']);i=locations[key];d['x'],d['y']=spots[i]
   assert images[i].tobytes()==white.crop((d['x'],d['y'],d['x']+d['w'],d['y']+d['h'])).tobytes()
 white.save(out/'rally-white#sheet.png');(out/'rally-white#anim.fanim').write_text(json.dumps(meta,separators=(',',':'))+'\n')
report={'scope':'Preview only; no shipped art modified or approval implied','preservedFrames':compared,'replacedHud':['flow','charges','role'],'continuousPaceSteps':21,'overheadSections':3 if rules['revision'] in ['rally-3.2','rally-3.3'] else 1,'maxHeightLabelMetres':19,'sheetSize':packed.size,'sha256':hashlib.sha256((out/'rally#sheet.png').read_bytes()).hexdigest()}
(out.parent.parent/'presentation-packing.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report))
