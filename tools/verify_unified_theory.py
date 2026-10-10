#!/usr/bin/env python3
"""Reproducible validation of canonical scientific data, runtime assets and exact lab math."""
import json, subprocess, sys, importlib.util, hashlib, math, re
from pathlib import Path
from PIL import Image
from unittest.mock import patch
ROOT=Path(__file__).resolve().parents[1]
def script(name):
 p=ROOT/'Claude outputs/unified_theory'/name;s=importlib.util.spec_from_file_location(name,p);m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
script('generate.py').generate(check=True)
c=json.loads((ROOT/'mods/tfm2_custom/champion/tfm2_custom_unified_theory.data_champion').read_text())
assert c['passive']['passive_ref']=='tfm2_custom_ai:unified_theory'
assert c['stat']['magic_power']==45 and c['stat']['hp']==1100
views=c['view_effects']+c['view_buffs']+c['view_projectiles'];names=[v['name'] for v in views];assert len(names)==len(set(names))
sheets=set()
native=(ROOT/'native/tfm2_custom_ai/src/unified_theory.rs').read_text()
ticks={key:int(re.search(r'const '+key+r': \w+ = (\d+);',native)[1]) for key in ['PACKET_ART_TICKS','FIELD_ART_TICKS','CAST_ART_TICKS','TRANSFORM_ART_TICKS']}
animation_frames={}
decoded_path=None;decoded=None
for v in views:
 p=ROOT/'mods'/v['anim'].removeprefix('asset/');sheets.add(p)
 meta=json.loads(Path(str(p)+'#anim.fanim').read_text());frames=meta['anims'][v['tag']]['frames']
 with Image.open(str(p)+'#sheet.png') as im:
  assert max(im.size)<=2048,(p,im.size)
  for f in frames:
   d=f['data'];assert d['x']>=0 and d['y']>=0 and d['x']+d['w']<=im.width and d['y']+d['h']<=im.height
 duration=sum(f['duration'] for f in frames)
 tag=v['tag'];animation_frames[tag]=frames
 if tag.startswith(('note','meter','allocation')):assert duration>=8/60
 if tag.startswith('packet_'):
  if '_pair' in tag:
   assert len(frames)==2 and math.isclose(duration,ticks['PACKET_ART_TICKS']/60),('packet replay overlap/gap',tag)
  else:assert len(frames)==8 and math.isclose(duration,4*ticks['PACKET_ART_TICKS']/60)
 if tag.startswith('field') and '_t' in tag:assert len(frames)==8 and math.isclose(duration,ticks['FIELD_ART_TICKS']/60),('field replay overlap/gap',tag)
 if tag.startswith('skill_'):
  assert len(frames)==8 and math.isclose(duration,ticks['CAST_ART_TICKS']/60),('cast lifecycle',tag)
  assert v['z']==-1 and v['is_follow'],'Cosmic activation must follow behind the body'
 if tag.startswith('complete'):assert len(frames)==8 and math.isclose(duration,.8),('completion lifecycle',tag)
 if tag.startswith('echo'):assert len(frames)==8 and math.isclose(duration,8/60),('echo lifecycle',tag)
 if tag.startswith('transform'):
  assert len(frames)==8 and math.isclose(duration,ticks['TRANSFORM_ART_TICKS']/60)
  assert v['type']=='Animated' and v['z']==4,'Morph must replace the outfit body layer'
 if tag.startswith('gearback'):
  assert len(frames)==8 and math.isclose(duration,1.6 if tag.endswith('_r7_p1') else .96),('aura cadence',tag)
  assert v['type']=='Animated' and v['z']==-1,'Cosmic aura must be one background buff'
 if (tag.startswith(('skill_','field','packet_','complete_')) and '_pair' not in tag and '_t' in tag) or tag.startswith('gearback'):
  if decoded_path!=p:
   decoded=Image.open(str(p)+'#sheet.png').convert('RGBA');decoded_path=p
  rectangles=[f['data'] for f in frames]
  assert len({(d['w'],d['h']) for d in rectangles})==1,('animation origin wobbles',tag)
  pixels={hashlib.sha256(decoded.crop((d['x'],d['y'],d['x']+d['w'],d['y']+d['h'])).tobytes()).digest() for d in rectangles}
  assert len(pixels)==8,('stored sprite cycle lost frames after palette/cropping',tag)
for s in script('generate.py').DATA['skills']:
 for n in range(len(s['tokens'])+1):assert 'tfm2_custom_unified_theory_note_'+s['id']+'_'+str(n) in names
# All ordinary badges differ structurally, animate, and omit ordinary-rank digits.
art=script('mastery_art.py')
badge_pixels=[art.badge(r,2).tobytes() for r in range(7)]
assert len(set(badge_pixels))==7
for r in range(7):assert len({art.badge(r,f).tobytes() for f in range(8)})>1
assert len({art.badge(7,2,p).tobytes() for p in range(1,11)})==10
for r in range(8):
 for p in (range(1,11) if r==7 else [None]):
  for f in range(8):
   box=art.badge(r,f,p).getbbox()
   assert box[2]-box[0]<=18 and box[3]-box[1]<=18,('oversized rank emblem',r,p,f,box)
skills=script('generate.py').DATA['skills']
for tier in range(4):
 assert len({art.skill_effect(s,tier,2,skills).tobytes() for s in range(75)})==75,'Skills lost their visual signatures'
 for s in skills:assert f'tfm2_custom_unified_theory_skill_{s["id"]}_t{tier}' in names
 for k in art.FIELDS:assert f'tfm2_custom_unified_theory_field{k}_t{tier}' in names
 for s in art.PACKETS:
  for angle in range(8):
   tag=f'packet_{skills[s]["id"]}_t{tier}_a{angle}'
   assert f'tfm2_custom_unified_theory_{tag}' in names
   # The four pairs reference exactly the eight original atlas rectangles.
   assert sum([animation_frames[tag+f'_pair{p}'] for p in range(4)],[])==animation_frames[tag]
for s in range(3):
 for rank in range(8):assert f'ut_outfit{s}_r{rank}' in names
 for p in range(1,5):assert f'ut_gearback{s}_r7_p{p}' in names
# Compact forms share the engine anchor and planted feet. Validate actual shipped
# body/outfit composition so leftover base hair cannot escape the target shape.
compact=script('compact_art.py')
body=Image.open(ROOT/'mods/tfm2_custom/champions/tfm2_custom_unified_theory#sheet.png').convert('RGBA').crop((0,0,48,64))
outfit_sheet=Image.open(ROOT/'mods/tfm2_custom/vfx/science_outfits#sheet.png').convert('RGBA')
transform_sheet=Image.open(ROOT/'mods/tfm2_custom/vfx/science_transforms#sheet.png').convert('RGBA')
def crop(sheet,tag,frame=0):
 d=animation_frames[tag][frame]['data'];return sheet.crop((d['x'],d['y'],d['x']+d['w'],d['y']+d['h']))
for science in range(3):
 assert len({crop(outfit_sheet,f'outfit{science}_r{r}',0).tobytes() for r in range(8)})==8,('rank costumes stopped evolving',science)
 for rank in range(8):
  assert len({crop(outfit_sheet,f'outfit{science}_r{rank}',f).tobytes() for f in range(8)})==8,('cosmic outfit repeats frames',science,rank)
  for f in range(8):
   outfit=crop(outfit_sheet,f'outfit{science}_r{rank}',f)
   assert outfit.crop((0,0,48,34)).tobytes()==crop(outfit_sheet,f'outfit{science}_r0',0).crop((0,0,48,34)).tobytes(),('cosmic rank changed accepted head/gaze',science,rank,f)
   im=body.copy();im.alpha_composite(outfit)
   expected=compact.foundation();expected.alpha_composite(outfit)
   assert im.tobytes()==expected.tobytes(),('base hair/prop escapes the selected costume',science,rank,f)
   x,y,right,bottom=im.getbbox()
   assert 18<=right-x<=24 and 35<=bottom-y<=40,('compact silhouette',science,rank,f,im.getbbox())
   # Curie's skirt and Newton's split hem can cover the shins. Foot pixels
   # below those hems must retain the exact common stance and floor anchor.
   assert im.crop((0,55,48,64)).tobytes()==body.crop((0,55,48,64)).tobytes(),'idle feet moved'
  assert art.equipment(science,rank,0,True).getbbox() is None,'aura obscures the scientist with a front layer'
  podiums=[1,2,3,4] if rank==7 else [4]
  for podium in podiums:
   frames=[art.equipment(science,rank,f,False,podium) for f in range(8)]
   assert len({im.tobytes() for im in frames})==8,('cosmic aura repeats frames',science,rank,podium)
   assert all(im.size==(96,112) and im.getbbox() is not None for im in frames),'aura escaped its bounded anchor'
   if rank==7 and podium==1:
    for im in frames:
     x,y,right,bottom=im.getbbox()
     assert right-x<=76 and bottom-y<=92,'#1 lost its bounded subject silhouette'
     for _,(red,green,blue,alpha) in im.getcolors(96*112):
      rgb=(red,green,blue)
      if alpha>128 and max(rgb)>180 and max(rgb)-min(rgb)>70:
       assert rgb.index(max(rgb))==[2,0,1][science],'#1 aura cycles into another discipline colour'
  if rank==7:
   assert len({art.equipment(science,7,2,False,p).tobytes() for p in range(1,5)})==4,('podium auras lost their distinct constructions',science)
  for target in range(3):
   if target==science:continue
   tag=f'transform{science}_{target}_r{rank}';assert f'ut_{tag}' in names
   frames=[crop(transform_sheet,tag,f) for f in range(8)]
   assert len({im.tobytes() for im in frames})==8,('morph lost a frame',tag)
   assert frames[0].tobytes()==crop(outfit_sheet,f'outfit{science}_r{rank}',0).tobytes()
   assert frames[-1].tobytes()==crop(outfit_sheet,f'outfit{target}_r{rank}',7).tobytes()
   for f in frames:
    im=body.copy();im.alpha_composite(f)
    expected=compact.foundation();expected.alpha_composite(f)
    assert im.tobytes()==expected.tobytes(),('base sprite escapes the morph',tag)
    assert im.crop((0,55,48,64)).tobytes()==body.crop((0,55,48,64)).tobytes(),'morph feet moved'
for s in range(75):assert len({art.skill_effect(s,t,2,skills).tobytes() for t in range(4)})==4,'Mastery effects stopped evolving'
for tier in range(4):
 for s in range(75):assert len({art.skill_effect(s,tier,f,skills).tobytes() for f in range(8)})==8,('cast repeats frames',s,tier)
 for k in art.FIELDS:assert len({art.field(k,tier,f).tobytes() for f in range(8)})==8,('field repeats frames',k,tier)
 for s in art.PACKETS:assert len({art.packet(s,tier,f,0,skills).tobytes() for f in range(8)})==8,('packet repeats frames',s,tier)
 for mask in range(1,8):assert len({art.modifier(mask,tier,f).tobytes() for f in range(8)})==8,('modifier repeats frames',mask,tier)
# The requested peaks must differ in shape, even with their colours removed.
for rank in range(8):
 for f in range(8):
  assert len({art.equipment(s,rank,f,False,1 if rank==7 else 4).getchannel('A').tobytes() for s in range(3)})==3,('subjects became recolours of one construction',rank,f)
for science in range(3):
 for f in range(8):
  assert art.equipment(science,7,f,False,1).getchannel('A').tobytes()!=art.equipment(science,7,f,False,4).getchannel('A').tobytes(),('#1 must have its own silhouette at every frame',science,f)
# Fail if the gravitational helper leaks into any unrelated subject/skill/frame.
with patch.object(art.cosmic,'singularity',side_effect=AssertionError('black hole outside Gravity Well/Horizon Ring')):
 for science in range(3):
  assert len({art.cosmic.echo(science,f).tobytes() for f in range(8)})==8
  for rank in range(8):
   for podium in ([1,2,3,4] if rank==7 else [4]):
    for f in range(8):art.equipment(science,rank,f,False,podium)
  for tier in range(4):
   for podium in ([1,2,3,4] if tier==3 else [4]):
    tag=f'complete_t{tier}_p{podium}_s{science}'
    assert f'tfm2_custom_unified_theory_{tag}' in names
    assert len({art.completion(tier,podium,f,science).tobytes() for f in range(8)})==8,('completion lost phases',tag)
 for tier in range(4):
  for skill in range(75):
   if skill not in (12,20):
    for f in range(8):art.skill_effect(skill,tier,f,skills)
  for skill in art.PACKETS:
   for f in range(8):art.packet(skill,tier,f,0,skills)
# Regeneration must reproduce exact bytes, including merged localization.
paths=list((ROOT/'mods/tfm2_custom/vfx').glob('science_*'))+list((ROOT/'mods/tfm2_custom/champions').glob('tfm2_custom_unified_theory*'))+[ROOT/'mods/tfm2_custom/champion/tfm2_custom_unified_theory.data_champion',ROOT/'mods/tfm2_custom/text/champion.i18n',ROOT/'editor/science-portraits.png']
paths += [ROOT/'editor/science-mastery-preview.png',ROOT/'editor/science-art-preview.json']+[ROOT/'docs'/f'unified-theory-{name}.png' for name in ['mastery','top10','effects']]
paths += [ROOT/'docs/unified-theory-cosmic-frames.png',ROOT/'docs/unified-theory-cosmic.gif']
paths += [ROOT/'docs/unified-theory-ranked-cosmic.gif']+[ROOT/'docs'/f'unified-theory-skills-{name}.png' for name in ['einstein','newton','curie']]
paths += [ROOT/'docs/unified-theory-aura-frames.png',ROOT/'docs/unified-theory-auras.gif']
paths += [ROOT/'docs'/name for name in ['unified-theory-scale.png','unified-theory-silhouettes.png','unified-theory-transform-frames.png','unified-theory-transforms.gif','unified-theory-facing.png']]
paths += [ROOT/'docs'/f'unified-theory-static-{name}.png' for name in ['cards','einstein','newton','curie']]
hashfiles=lambda:{str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in paths}
before=hashfiles();script('art.py').generate();script('data.py').generate();assert before==hashfiles(),'Generated assets drifted'
subprocess.run(['node','-e',"const fs=require('fs'),l=require('./editor/unified-theorylab.js');if(!l.selfTest())process.exit(1);console.log(l.validateVectors(fs.readFileSync('native/tfm2_custom_ai/src/unified_theory_vectors.txt','utf8'))+' exact native charge vectors; all 8 ranks deterministic');"],cwd=ROOT,check=True)
print(f'Verified {len(views)} views over {len(sheets)} VFX sheets; original body <=2048; notebook lifetimes; regeneration byte-identical.')
print('Art: 300 casts, 44 fields and 448 directional packet loops have eight distinct frames; 1792 pair aliases share their atlas pixels; lifetimes match native replay deadlines; cosmic podium previews verified.')
print('Compact art: all eight frames of 24 forward-gazing costumes fit 18–24 x 35–40 px; every rank emblem fits 18 x 18 px; 48 direct transformations have eight distinct frames over 24 ticks with planted feet, covered base pixels and exact endpoints; static champion cards regenerate exactly.')
print('Cosmic ranks: eight unique costumes per scientist, eight distinct frames per costume and preparation modifier; accepted heads/gaze unchanged; all 75 skill-stage cards and the ranked animation regenerate exactly.')
print('Cosmic auras: 33 distinct eight-frame background loops fit 96 x 112 px, including four podium constructions per scientist; stored frames retain their phases; front layers stay empty; aura frame strips/GIF regenerate exactly.')
