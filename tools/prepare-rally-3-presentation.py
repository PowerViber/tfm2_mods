"""Preview-only extension of frozen Rally 2.2 pixels. Never writes shipped assets."""
from pathlib import Path
from PIL import Image,ImageDraw
import hashlib,json,shutil
ROOT=Path(__file__).resolve().parents[1]
rules=json.loads((ROOT/'rules/rally.json').read_text())
assert rules['revision']=='rally-3.0'
old=ROOT/'docs/rally-art-review/packed-preview/vfx'
out=ROOT/'docs/rally-results/rally-3.0/packed-preview/vfx'
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
 if tag.startswith(('flow','charges')):continue
 images=[]
 for f in animation['frames']:
  d=f['data'];images.append(sheet.crop((d['x'],d['y'],d['x']+d['w'],d['y']+d['h'])))
 add(tag,images,[f['duration'] for f in animation['frames']])
for n in range(6):
 image=Image.new('RGBA',(64,64));draw=ImageDraw.Draw(image)
 for i in range(5):
  draw.rectangle((13+i*8,0,19+i*8,5),fill='#07131cff')
  draw.rectangle((14+i*8,1,18+i*8,4),fill='#ffe16aff' if i<n else '#253844ff')
 add('pace'+str(n),[image]*8,[.1]*8)
for height in range(16100,rules['constants']['MAX_HEIGHT_LABEL']+1,100):
 image=Image.new('RGBA',(64,16));draw=ImageDraw.Draw(image)
 draw.rectangle((11,1,53,14),fill='#10212bcf');draw.text((14,2),f'{height/1000:.1f} m',fill='#fff4bbff')
 add('height_'+str(height),[image],[2/60])
 add('contact_height_'+str(height),[image]*8,[.1]*8)
positions={};x=y=row=0
for i in sorted(range(len(frames)),key=lambda i:(-frames[i].height,-frames[i].width,i)):
 im=frames[i]
 if x+im.width>2048:x=0;y+=row;row=0
 positions[i]=(x,y);x+=im.width;row=max(row,im.height)
packed=Image.new('RGBA',(2048,y+row));assert packed.height<=2048
for i,im in enumerate(frames):packed.alpha_composite(im,positions[i])
for animation in anims.values():
 for f in animation['frames']:
  d=f['data'];i=d.pop('index');d.update(x=positions[i][0],y=positions[i][1])
# Frozen effects, anchors, durations and all old height labels must be pixel-identical.
compared=0
for tag,animation in original.items():
 if tag.startswith(('flow','charges')):continue
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
report={'scope':'Preview only; no shipped art modified or approval implied','preservedFrames':compared,'removedLegacyHud':['flow','charges'],'newPaceBars':5,'maxHeightLabelMetres':19,'sheetSize':packed.size,'sha256':hashlib.sha256((out/'rally#sheet.png').read_bytes()).hexdigest()}
(out.parent.parent/'presentation-packing.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report))
