#!/usr/bin/env python3
"""Reproducible validation of canonical scientific data, runtime assets and exact lab math."""
import json, subprocess, sys, importlib.util, hashlib, math, re
from pathlib import Path
from PIL import Image
from unittest.mock import patch
ROOT=Path(__file__).resolve().parents[1]
def script(name):
 p=ROOT/'Claude outputs/unified_theory'/name;s=importlib.util.spec_from_file_location(name,p);m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
subprocess.run([sys.executable,'tools/generate-unified-equations.py','--check'],cwd=ROOT,check=True)
c=json.loads((ROOT/'mods/tfm2_custom/champion/tfm2_custom_unified_theory.data_champion').read_text())
assert c['passive']['passive_ref']=='tfm2_custom_ai:unified_theory'
assert c['stat']['magic_power']==45 and c['stat']['hp']==1100
views=c['view_effects']+c['view_buffs']+c['view_projectiles'];names=[v['name'] for v in views];assert len(names)==len(set(names))
sheets=set()
core=(ROOT/'native/tfm2_custom_ai/src/unified_core.rs').read_text()
animation_frames={}
for v in views:
 p=ROOT/'mods'/v['anim'].removeprefix('asset/');sheets.add(p)
 meta=json.loads(Path(str(p)+'#anim.fanim').read_text());frames=meta['anims'][v['tag']]['frames']
 with Image.open(str(p)+'#sheet.png') as im:
  assert max(im.size)<=2048,(p,im.size)
  for f in frames:
   d=f['data'];assert d['x']>=0 and d['y']>=0 and d['x']+d['w']<=im.width and d['y']+d['h']<=im.height
 animation_frames[v['tag']]=frames
# The equation kit: every lifetime matches the ticks the core asks for, every name the core formats exists.
EQ=json.loads((ROOT/'Claude outputs/unified_theory/equations.json').read_text(encoding='utf-8'))
def seconds(tag):return sum(f['duration'] for f in animation_frames[tag])
eq_art=script('equation_art.py')
for i in range(18):
 assert math.isclose(seconds(f'eq{i}'),1.0) and math.isclose(seconds(f'im{i}'),.4),i
for i in range(36):assert math.isclose(seconds(f'sig{i}'),1.5),i
for i in range(6):assert math.isclose(seconds(f'uni{i}'),1.5),i
for pos in range(4):
 for ch in '0123456789':assert math.isclose(seconds(f'n{pos}_{ch}'),1.0)
for ready in range(6):
 for reserved in range(min(4,5-ready)+1):
  for p in range(4):assert math.isclose(seconds(f'ch_{ready}_{reserved}_{p}'),8/60)
for slot in range(4):
 for k in range(18):assert math.isclose(seconds(f'rv{slot}_{k}'),8/60)
for k in eq_art.PROJECTILES:
 for h in range(8):assert math.isclose(seconds(f'pk{eq_art.KEYS.index(k)}_a{h}'),4/60)
for k in eq_art.ZONES:assert math.isclose(seconds(f'zn{eq_art.KEYS.index(k)}'),.8)
for tag,ticks in [('insight',20),('dot',60),('lawshield',90),('twin',30),('ultwarn',36),('ult',48),('drop',36),('puddle',48)]:
 assert math.isclose(seconds(tag),ticks/60),tag
 assert f'"{tag}".into()' in core,tag
for pattern in ['"eq{skill}"','"sig{s}"','"uni{k}"','"n{pos}_{ch}"','"ch_{}_{}_{}"','"rv{slot}_{}"','"pk{}_a{}"','"zn{}"','"im{skill}"','"split_a{}"','"jump_a{}"']:assert pattern in core,pattern
labels=[v for v in views if v['tag'].startswith(('eq','sig','uni','ch_','rv','n0_','n1_','n2_','n3_'))]
assert all(v['z']==7 and not v['is_follow'] for v in labels),'labels are point overlays above the head'
assert len(labels)==18+36+6+40+80+72
# Zones are drawn at their game radius (about 940 units per pixel).
for k in eq_art.ZONES:
 s_=EQ['skills'][eq_art.KEYS.index(k)];w=animation_frames[f'zn{eq_art.KEYS.index(k)}'][0]['data']['w']
 assert abs(w-2*s_['radius']/940)<=14,(k,w)
assert not any(v['tag'].startswith(('packet_','skill_','impact_','field','orb_','bar_','complete','accident','mark','echo','modifiers')) for v in views),'old recipe art left behind'
for science in range(3):
 for rank in range(8):
  for phase in range(3):
   tag=f'pose{science}_r{rank}_p{phase}'
   assert f'ut_{tag}' in names
   assert len(animation_frames[tag])==8
# All ordinary badges differ structurally, animate, and omit ordinary-rank digits.
art=script('mastery_art.py')
assert 'ut_crystal_body' in names
assert art.cosmic.verify_crystal_coverage()>0,'crystal must cover the registered base unit'
crystal_view=next(v for v in c['view_buffs'] if v['name']=='ut_crystal_body')
with Image.open(str(ROOT/'mods'/crystal_view['anim'].removeprefix('asset/'))+'#sheet.png') as crystal_sheet:
 crystal_sheet=crystal_sheet.convert('RGBA')
 for frame,rect in enumerate(animation_frames['crystal_body']):
  d=rect['data'];stored=crystal_sheet.crop((d['x'],d['y'],d['x']+d['w'],d['y']+d['h']))
  assert stored.tobytes()==art.cosmic.crystal_body(frame).tobytes(),'shipped crystal pixels differ from coverage proof'
badge_pixels=[art.badge(r,2).tobytes() for r in range(7)]
assert len(set(badge_pixels))==7
for r in range(7):assert len({art.badge(r,f).tobytes() for f in range(8)})>1
assert len({art.badge(7,2,p).tobytes() for p in range(1,11)})==10
for r in range(8):
 for p in (range(1,11) if r==7 else [None]):
  for f in range(8):
   box=art.badge(r,f,p).getbbox()
   assert box[2]-box[0]<=18 and box[3]-box[1]<=18,('oversized rank emblem',r,p,f,box)
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
   assert all(im.size==art.iconic.AURA and im.getbbox() is not None for im in frames),'aura escaped its bounded anchor'
   if rank==7 and podium==1:
    for f,im in enumerate(frames):
     x,y,right,bottom=im.getbbox()
     assert right-x>=100,'#1 must read as the largest spectacle'
     # Both other scientists stand at the shoulders: their translucent
     # figures add pixels on each side of the body.
     plain=im.getchannel('A');bx,by=art.iconic.BODY
     assert plain.crop((bx-12,by+20,bx+12,by+56)).getbbox() and plain.crop((bx+36,by+20,bx+60,by+56)).getbbox(),('#1 lost its shoulder ghosts',science,f)
  if rank==7:
   assert len({art.equipment(science,7,2,False,p).tobytes() for p in range(1,5)})==4,('podium auras lost their distinct constructions',science)
  # Every rank adds a layer: the aura grows from Student to #1.
  if rank==7:
   area=lambda r,p:(lambda b:(b[2]-b[0])*(b[3]-b[1]))(art.equipment(science,r,2,False,p).getbbox())
   sizes=[area(r,4) for r in range(7)]+[area(7,4),area(7,1)]
   assert sizes[0]<sizes[3]<sizes[6]<=sizes[8] and sizes[7]<sizes[8],('mastery spectacle does not grow',science,sizes)
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
# The requested peaks must differ in shape, even with their colours removed.
for rank in range(8):
 for f in range(8):
  assert len({art.equipment(s,rank,f,False,1 if rank==7 else 4).getchannel('A').tobytes() for s in range(3)})==3,('subjects became recolours of one construction',rank,f)
for science in range(3):
 for f in range(8):
  assert art.equipment(science,7,f,False,1).getchannel('A').tobytes()!=art.equipment(science,7,f,False,4).getchannel('A').tobytes(),('#1 must have its own silhouette at every frame',science,f)
# Fail if the gravitational helper leaks into any unrelated subject/skill/frame.
assert art.iconic.cosmic is art.cosmic,'the black-hole guard must cover the iconic artwork'
with patch.object(art.cosmic,'singularity',side_effect=AssertionError('black hole outside the auras')):
 for science in range(3):
  for rank in range(8):
   for podium in ([1,2,3,4] if rank==7 else [4]):
    for f in range(8):art.equipment(science,rank,f,False,podium)
# Regeneration must reproduce exact bytes, including merged localization.
paths=list((ROOT/'mods/tfm2_custom/vfx').glob('science_*'))+list((ROOT/'mods/tfm2_custom/champions').glob('tfm2_custom_unified_theory*'))+[ROOT/'mods/tfm2_custom/champion/tfm2_custom_unified_theory.data_champion',ROOT/'mods/tfm2_custom/text/champion.i18n',ROOT/'editor/science-portraits.png']
paths += [ROOT/'editor/science-mastery-preview.png',ROOT/'editor/science-art-preview.json']+[ROOT/'docs'/f'unified-theory-{name}.png' for name in ['mastery','top10']]+[ROOT/'docs/unified-equations.png']
paths += [ROOT/'docs/unified-theory-cosmic.gif']
paths += [ROOT/'docs/unified-theory-ranked-cosmic.gif']
paths += [ROOT/'docs/unified-theory-aura-frames.png',ROOT/'docs/unified-theory-auras.gif']
paths += [ROOT/'docs'/name for name in ['unified-theory-scale.png','unified-theory-silhouettes.png','unified-theory-transform-frames.png','unified-theory-transforms.gif','unified-theory-facing.png']]
paths += [ROOT/'docs'/f'unified-theory-static-{name}.png' for name in ['cards','einstein','newton','curie']]
hashfiles=lambda:{str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in paths}
before=hashfiles();script('art.py').generate();script('data.py').generate();assert before==hashfiles(),'Generated assets drifted'
# Facing: buff overlays never flip in game, so each body layer's `_l` copy is its exact mirror (trimmed around the
# same anchor), and the native names one for every outfit, pose and transform.
def frames_of(v):
    base=ROOT/'mods'/v['anim'].removeprefix('asset/')
    sheet=Image.open(str(base)+'#sheet.png').convert('RGBA')
    return sheet,json.loads(Path(str(base)+'#anim.fanim').read_text())['anims'][v['tag']]['frames']
by_tag={v['tag']:v for v in c['view_buffs']}
lefts=[v for v in c['view_buffs'] if v['tag'].endswith('_l')]
assert {v['tag'][:-2] for v in lefts}=={t for t in by_tag if t.startswith(('outfit','pose','transform')) and not t.endswith('_l')},'every body layer has a mirrored copy'
for v in lefts:
    (li,lf),(ri,rf)=frames_of(v),frames_of(by_tag[v['tag'][:-2]])
    for a_,b_ in zip(lf,rf):
        a_,b_=a_['data'],b_['data']
        left=li.crop((a_['x'],a_['y'],a_['x']+a_['w'],a_['y']+a_['h']))
        ox,oy=(b_['w']-a_['w'])//2,(b_['h']-a_['h'])//2
        right=ri.crop((b_['x']+ox,b_['y']+oy,b_['x']+ox+a_['w'],b_['y']+oy+a_['h'])).transpose(Image.Transpose.FLIP_LEFT_RIGHT)
        assert left.tobytes()==right.tobytes(),('the left-facing copy is not a mirror',v['tag'])
assert '{side}' in (ROOT/'native/tfm2_custom_ai/src/unified_theory.rs').read_text(),'the native picks the mirrored body layer'
decoded_bytes=0
for p in sheets:
 with Image.open(str(p)+'#sheet.png') as im: decoded_bytes+=im.width*im.height*4
print(f'Verified {len(views)} views over {len(sheets)} VFX sheets; every atlas <=2048; compact HUD lifetimes; regeneration byte-identical; decoded RGBA estimate {decoded_bytes/1048576:.2f} MiB.')
print('Equation kit: 18 equations, 36 signatures, 6 Unifications, live values, 5-pip charge bar, 72 reserve icons, 88 directional projectiles, 18 impacts and 4 zones at game radius; every lifetime matches the core and every name it formats exists.')
print('Compact art: 24 forward-gazing costumes fit 18-24 x 35-40 px; rank emblems fit 18 x 18 px; 48 morphs keep planted feet and exact endpoints.')
print('Iconic auras: 33 distinct eight-frame loops grow from Student to #1; #1 shows the other two scientists at the shoulders; front layers stay empty.')
