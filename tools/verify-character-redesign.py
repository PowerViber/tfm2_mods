"""Raster/metadata equivalence and actual runtime sprite inspection for the art-only batch."""
from pathlib import Path
from PIL import Image,ImageDraw,ImageOps
import importlib.util,json,hashlib,base64
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/vader-aegis-scribble-review'
s=importlib.util.spec_from_file_location('art','tools/integrate-character-redesign.py');art=importlib.util.module_from_spec(s);s.loader.exec_module(art)
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def frames(stem):
 im=Image.open(ROOT/(stem+'#sheet.png')).convert('RGBA');a=json.loads((ROOT/(stem+'#anim.fanim')).read_text())['anims'];return im,a
report={'base':art.BASE,'nativeDllByteIdentical':sha(ROOT/'mods/tfm2_custom_ai/tfm2_custom_ai.dll')==hashlib.sha256(art.original('mods/tfm2_custom_ai/tfm2_custom_ai.dll')).hexdigest(),'bodies':{},'noInstallation':True,'liveVisualAcceptance':'pending'}
assert report['nativeDllByteIdentical']
allowed={'mods/tfm2_custom/mod.mod_info','mods/tfm2_toon/mod.mod_info','mods/tfm2_starwars/mod.mod_info'}
for key,(pack,id,source,height)in art.SPECS.items():
 stem=f'mods/{pack}/champions/{id}';sheet,a=frames(stem);old=art.old_json(stem+'#anim.fanim')['anims'];assert set(a)==set(old);assert max(sheet.size)<=2048
 for tag,an in a.items():
  assert [f['duration']for f in an['frames']]==[f['duration']for f in old[tag]['frames']]
  for f in an['frames']:
   r=f['data'];assert r['x']>=0 and r['y']>=0 and r['x']+r['w']<=sheet.width and r['y']+r['h']<=sheet.height
 assert art.crop(sheet,a['idle']['frames'][0]).getbbox(),'Actual runtime idle must be visible'
 definition=f'mods/{pack}/champion/{id}.data_champion';assert(ROOT/definition).read_bytes().replace(b'\r\n',b'\n')==art.original(definition).replace(b'\r\n',b'\n')
 for ext in ['#sheet.png','#anim.fanim']:allowed.add(stem+ext)
 # Actual atlas frames, rather than an editor thumbnail substitution.
 tags=[t for t in ['idle','run','attack','skill1','skill2','ult','hit','dead']if t in a];images=[]
 contact=Image.new('RGB',(len(tags)*260,270),'#17382e');draw=ImageDraw.Draw(contact)
 for j,tag in enumerate(tags):
  draw.text((j*260+8,8),key+' / '+tag,fill='white')
  for n,f in enumerate(a[tag]['frames']):
   im=art.crop(sheet,f);panel=Image.new('RGB',(600,260),'#17382e');d=ImageDraw.Draw(panel);d.text((8,8),key+' / actual runtime '+tag,fill='white');z=im.resize((im.width*3,im.height*3),Image.Resampling.NEAREST);panel.paste(z,(140-z.width//2,140-z.height//2),z);left=ImageOps.mirror(z);panel.paste(left,(450-z.width//2,140-z.height//2),left);images.append((panel,max(10,round(f['duration']*1000))))
  im=art.crop(sheet,a[tag]['frames'][0]);z=im.resize((im.width*3,im.height*3),Image.Resampling.NEAREST);contact.paste(z,(j*260+130-z.width//2,140-z.height//2),z)
 contact.save(OUT/key/'runtime-contact.png');images[0][0].save(OUT/key/'runtime-animations.gif',save_all=True,append_images=[p for p,t in images[1:]],duration=[t for p,t in images],loop=0)
 report['bodies'][key]={'animations':len(a),'frames':sum(len(v['frames'])for v in a.values()),'actualIdleVisible':True,'frameCountsAndDurationsPreserved':True,'atlas':list(sheet.size),'bodyDecodedBytes':sheet.width*sheet.height*4,'definitionByteIdentical':True}
 # Separate effects in Vader's body atlas remain frozen.
 if key=='vader':
  legacy=art.old_image(stem+'#sheet.png')
  for tag in ['ult_area','skill1_effect']:
   for f,of in zip(a[tag]['frames'],old[tag]['frames']):assert art.crop(sheet,f).tobytes()==art.crop(legacy,of).tobytes()
stem='mods/tfm2_custom/vfx/gundam';sheet,a=frames(stem);old=art.old_json(stem+'#anim.fanim')['anims'];legacy=art.old_image(stem+'#sheet.png');changed=set(json.loads((OUT/'integration.json').read_text())['effectChanges']);frozen=0
assert set(a)==set(old) and max(sheet.size)<=2048
for tag,an in a.items():
 assert [f['duration']for f in an['frames']]==[f['duration']for f in old[tag]['frames']]
 for f in an['frames']:
  r=f['data'];assert r['x']>=0 and r['y']>=0 and r['x']+r['w']<=sheet.width and r['y']+r['h']<=sheet.height
 if tag in changed:continue
 for f,of in zip(an['frames'],old[tag]['frames']):
  im=art.crop(sheet,f);before=art.crop(legacy,of);restored=Image.new('RGBA',before.size);assert(before.width-im.width)%2==0 and(before.height-im.height)%2==0;restored.alpha_composite(im,((before.width-im.width)//2,(before.height-im.height)//2));assert restored.tobytes()==before.tobytes(),tag;frozen+=1
allowed|={stem+'#sheet.png',stem+'#anim.fanim'}
# Inspect the actual packaged flight figures and enlarged wings in both facings.
phases=['deploy','ascend','sky_r','dive','wings_light','retract'];film=[]
contact=Image.new('RGB',(len(phases)*300,400),'#17382e');draw=ImageDraw.Draw(contact)
for j,tag in enumerate(phases):
 draw.text((j*300+8,8),'Aegis / '+tag,fill='white')
 for n,f in enumerate(a[tag]['frames']):
  im=art.crop(sheet,f);panel=Image.new('RGB',(800,420),'#17382e');panel.paste(Image.new('RGB',(400,420),'#cccbb7'),(400,0));d=ImageDraw.Draw(panel);d.text((8,8),tag+' / '+str(n),fill='white')
  for x,pose in [(200,im),(600,ImageOps.mirror(im))]:panel.paste(pose,(x-pose.width//2,220-pose.height//2),pose)
  film.append((panel,max(10,round(f['duration']*1000))))
  if n==len(a[tag]['frames'])//2:contact.paste(im,(j*300+150-im.width//2,210-im.height//2),im)
contact.save(OUT/'aegis/runtime-flight-contact.png');film[0][0].save(OUT/'aegis/runtime-flight.gif',save_all=True,append_images=[p for p,t in film[1:]],duration=[t for p,t in film],loop=0)
baseline=json.loads((OUT/'integration-baseline.json').read_text());modified=[]
for name,digest in baseline['files'].items():
 p=ROOT/name
 if sha(p)!=digest:
  normalized=name.replace('\\','/');assert normalized in allowed,normalized;modified.append(normalized)
for key,(pack,id,source,height)in art.SPECS.items():
 body=report['bodies'][key]['bodyDecodedBytes'];owned=ROOT/('mods/tfm2_custom/vfx/gundam#sheet.png'if key=='aegis'else'mods/tfm2_starwars/vfx/starwars#sheet.png'if key=='vader'else'mods/tfm2_toon/vfx/scribble#sheet.png');im=Image.open(owned);total=body+im.width*im.height*4;assert total<=32*1024*1024;report['bodies'][key]['ownedDecodedBytes']=total
text=(ROOT/'editor/sprites-data.js').read_text(encoding='utf8');data=json.loads(text[text.index('{'):].rstrip().removesuffix(';'))
for key,(pack,id,source,height)in art.SPECS.items():assert base64.b64decode(data[key]['png'])==(ROOT/f'mods/{pack}/champions/{id}#sheet.png').read_bytes()
sourceHashes={p.name:sha(p)for p in OUT.glob('*source*.png')}
report.update(modFilesCompared=len(baseline['files']),modifiedModFiles=modified,frozenGundamEffectFrames=frozen,atlasLimit=2048,legacyEffectAliasesPreserved=True,frozenSourceHashes=sourceHashes,nativeDllSha256=sha(ROOT/'mods/tfm2_custom_ai/tfm2_custom_ai.dll'))
(OUT/'final-verification.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
