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
 if tag.startswith(('orb_','bar_')):
  assert math.isclose(duration,8/60),('HUD refresh lifetime',tag)
  assert v['z']==7 and not v['is_follow'],'HUD must remain a small positioned overlay'
  assert all((f['data']['w'],f['data']['h'])==((12,12) if tag.startswith('orb_') else (42,5)) for f in frames),('oversized HUD',tag)
  assert len(frames)==(8 if tag.startswith('orb_') else 1)
 if tag.startswith('packet_'):
  if '_pair' in tag:
   assert len(frames)==2 and math.isclose(duration,ticks['PACKET_ART_TICKS']/60),('packet replay overlap/gap',tag)
  else:assert len(frames)==8 and math.isclose(duration,4*ticks['PACKET_ART_TICKS']/60)
 if tag.startswith('field') and '_t' in tag:assert len(frames)==8 and math.isclose(duration,ticks['FIELD_ART_TICKS']/60),('field replay overlap/gap',tag)
 if tag.startswith('skill_'):
  assert len(frames)==8 and math.isclose(duration,ticks['CAST_ART_TICKS']/60),('cast lifecycle',tag)
  assert v['z']==-1 and v['is_follow'],'Cosmic activation must follow behind the body'
 if tag.startswith('impact_'):
  assert len(frames)==8 and math.isclose(duration,.4),('impact lifecycle',tag)
  assert v['z']==2 and not v['is_follow'],'Contact artwork belongs at the impact point'
 if tag.startswith('complete'):assert len(frames)==8 and math.isclose(duration,.8),('completion lifecycle',tag)
 if tag.startswith('accident'):
  assert len(frames)==8 and math.isclose(duration,int(re.search(r'const ACCIDENT_ART_TICKS: \w+ = (\d+);',native)[1])/60),('accident lifecycle',tag)
  assert v['z']==5 and v['is_follow'],'Lab accidents play in front of the scientist'
 if tag.startswith('mark'):
  assert len(frames)==8 and math.isclose(duration,ticks['FIELD_ART_TICKS']/60),('setup mark replay',tag)
  assert v['is_follow']==tag.startswith(('mark50','mark69')),'target marks follow; floor marks stay put'
 if tag.startswith('echo'):assert len(frames)==8 and math.isclose(duration,8/60),('echo lifecycle',tag)
 if tag.startswith('transform'):
  assert len(frames)==8 and math.isclose(duration,ticks['TRANSFORM_ART_TICKS']/60)
  assert v['type']=='Animated' and v['z']==4,'Morph must replace the outfit body layer'
 if tag.startswith('gearback'):
  assert len(frames)==8 and math.isclose(duration,1.6 if tag.endswith('_r7_p1') else .96),('aura cadence',tag)
  assert v['type']=='Animated' and v['z']==-1,'Cosmic aura must be one background buff'
 if (tag.startswith(('skill_','impact_','field','packet_','complete_')) and '_pair' not in tag and '_t' in tag) or tag.startswith('gearback'):
  if decoded_path!=p:
   decoded=Image.open(str(p)+'#sheet.png').convert('RGBA');decoded_path=p
  rectangles=[f['data'] for f in frames]
  assert len({(d['w'],d['h']) for d in rectangles})==1,('animation origin wobbles',tag)
  pixels={hashlib.sha256(decoded.crop((d['x'],d['y'],d['x']+d['w'],d['y']+d['h'])).tobytes()).digest() for d in rectangles}
  assert len(pixels)==8,('stored sprite cycle lost frames after palette/cropping',tag)
# Notebook details remain in Science Lab. The match renders exactly two small
# independently indexed overlays instead of the old five wide HUD components.
assert not any(v['tag'].startswith(('note','meter','allocation')) for v in views)
hud=script('hud_art.py')
for science in range(3):
 for progress in range(9):
  for status in range(4):
   tag=f'orb_{science}_{progress}_{status}'
   assert f'tfm2_custom_unified_theory_{tag}' in names
   assert len({hud.orb(science,progress,status,f).tobytes() for f in range(8)})==8,('orb repeats frames',tag)
for cell_a in range(12):
 for cell_b in range(12):
  for cell_c in range(12):
   assert f'tfm2_custom_unified_theory_bar_{cell_a}_{cell_b}_{cell_c}' in names
   assert hud.composed(0,0,0,cell_a,cell_b,cell_c,0).size==(57,12)
assert len([v for v in views if v['tag'].startswith('orb_')])==108
assert len([v for v in views if v['tag'].startswith('bar_')])==1728
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
skills=script('generate.py').DATA['skills']
for tier in range(4):
 assert len({art.skill_effect(s,tier,2,skills).tobytes() for s in range(75)})==75,'Skills lost their visual signatures'
 for s in skills:assert f'tfm2_custom_unified_theory_skill_{s["id"]}_t{tier}' in names
 for s in skills:assert f'tfm2_custom_unified_theory_impact_{s["id"]}_t{tier}' in names
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
for s in range(75):assert len({art.skill_effect(s,t,2,skills).tobytes() for t in range(4)})==4,'Mastery effects stopped evolving'
for tier in range(4):
 for s in range(75):assert len({art.skill_effect(s,tier,f,skills).tobytes() for f in range(8)})==8,('cast repeats frames',s,tier)
 for s in range(75):assert len({art.impact(s,tier,f,skills).tobytes() for f in range(8)})==8,('impact repeats frames',s,tier)
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
assert art.iconic.cosmic is art.cosmic,'the black-hole guard must cover the iconic artwork'
# Round 125 sheets: resonance, accidents and setup marks resolve and animate.
for tier in range(4):
 for podium in ([1,2,3,4] if tier==3 else [4]):
  tag=f'complete_unified_t{tier}_p{podium}';assert f'tfm2_custom_unified_theory_{tag}' in names
  assert len({art.iconic.unified(tier,podium,f).tobytes() for f in range(8)})==8,('resonance lost phases',tag)
 for kind in art.MARKS:
  assert f'tfm2_custom_unified_theory_mark{kind}_t{tier}' in names
  assert len({art.iconic.mark(kind,tier,f).tobytes() for f in range(8)})==8,('mark lost phases',kind,tier)
for science in range(3):
 assert f'tfm2_custom_unified_theory_accident{science}' in names
 assert len({art.iconic.accident(science,f).tobytes() for f in range(8)})==8,('accident lost phases',science)
# Native names resolve: every formatted round-125 tag has a view.
for pattern in ['_accident{science}','_complete_unified_t{}_p{}','_mark{kind}_t{}']:assert pattern.split('{')[0] in native,pattern
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
  for podium in ([1,2,3,4] if tier==3 else [4]):
   for f in range(8):art.iconic.unified(tier,podium,f)
  for kind in art.MARKS:
   for f in range(8):art.iconic.mark(kind,tier,f)
 for science in range(3):
  for f in range(8):art.iconic.accident(science,f)
# Regeneration must reproduce exact bytes, including merged localization.
paths=list((ROOT/'mods/tfm2_custom/vfx').glob('science_*'))+list((ROOT/'mods/tfm2_custom/champions').glob('tfm2_custom_unified_theory*'))+[ROOT/'mods/tfm2_custom/champion/tfm2_custom_unified_theory.data_champion',ROOT/'mods/tfm2_custom/text/champion.i18n',ROOT/'editor/science-portraits.png']
paths += [ROOT/'editor/science-mastery-preview.png',ROOT/'editor/science-art-preview.json']+[ROOT/'docs'/f'unified-theory-{name}.png' for name in ['mastery','top10','effects']]
paths += [ROOT/'docs/unified-theory-cosmic-frames.png',ROOT/'docs/unified-theory-cosmic.gif']
paths += [ROOT/'docs/unified-theory-ranked-cosmic.gif']+[ROOT/'docs'/f'unified-theory-skills-{name}.png' for name in ['einstein','newton','curie']]
paths += [ROOT/'docs/unified-theory-aura-frames.png',ROOT/'docs/unified-theory-auras.gif']
paths += [ROOT/'docs'/name for name in ['unified-theory-scale.png','unified-theory-silhouettes.png','unified-theory-transform-frames.png','unified-theory-transforms.gif','unified-theory-facing.png']]
paths += [ROOT/'docs'/f'unified-theory-static-{name}.png' for name in ['cards','einstein','newton','curie']]
paths += [ROOT/'docs/unified-theory-hud.png',ROOT/'docs/unified-theory-hud.gif']
paths += [ROOT/'docs/unified-theory-peak-completions.png',ROOT/'docs/unified-theory-peak-completions.gif']
paths += [ROOT/'docs/unified-theory-resonance-accidents.png',ROOT/'docs/unified-theory-resonance-accidents.gif']
hashfiles=lambda:{str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in paths}
before=hashfiles();script('art.py').generate();script('data.py').generate();assert before==hashfiles(),'Generated assets drifted'
subprocess.run(['node','-e',"const fs=require('fs'),l=require('./editor/unified-theorylab.js');if(!l.selfTest())process.exit(1);console.log(l.validateVectors(fs.readFileSync('native/tfm2_custom_ai/src/unified_theory_vectors.txt','utf8'))+' exact native charge vectors; all 8 ranks deterministic');"],cwd=ROOT,check=True)
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
print(f'Art: 300 casts, 300 impacts, {len(art.FIELDS)*4} fields and {len(art.PACKETS)*32} directional packet loops have eight distinct frames; pair aliases share their atlas pixels; lifetimes match native replay deadlines; cosmic podium previews verified.')
print('HUD: 108 eight-frame 12x12 scientist orbs and 1728 three-cell 42x5 charge states; total footprint 57x12 px; obsolete wide notebook and allocation views removed.')
print('Compact art: all eight frames of 24 forward-gazing costumes fit 18–24 x 35–40 px; every rank emblem fits 18 x 18 px; 48 direct transformations have eight distinct frames over 24 ticks with planted feet, covered base pixels and exact endpoints; static champion cards regenerate exactly.')
print('Cosmic ranks: eight unique costumes per scientist, eight distinct frames per costume and preparation modifier; accepted heads/gaze unchanged; all 75 skill-stage cards and the ranked animation regenerate exactly.')
print('Iconic auras: 33 distinct eight-frame background loops fit 160 x 176 px and grow from Student to #1; #1 shows the other two scientists at the shoulders; four podium constructions per scientist; front layers stay empty; aura frame strips/GIF regenerate exactly.')
print('Round 125: 7 Unified resonance loops, 3 lab accidents and 16 setup marks have eight distinct frames and native-matching lifetimes; black holes stay limited to Gravity Well and Horizon Ring.')
