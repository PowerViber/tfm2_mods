#!/usr/bin/env python3
"""Reproducible validation of canonical scientific data, runtime assets and exact lab math."""
import json, subprocess, sys, importlib.util, hashlib
from pathlib import Path
from PIL import Image
ROOT=Path(__file__).resolve().parents[1]
def script(name):
 p=ROOT/'Claude outputs/unified_theory'/name;s=importlib.util.spec_from_file_location(name,p);m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
script('generate.py').generate(check=True)
c=json.loads((ROOT/'mods/tfm2_custom/champion/tfm2_custom_unified_theory.data_champion').read_text())
assert c['passive']['passive_ref']=='tfm2_custom_ai:unified_theory'
assert c['stat']['magic_power']==45 and c['stat']['hp']==1100
views=c['view_effects']+c['view_buffs']+c['view_projectiles'];names=[v['name'] for v in views];assert len(names)==len(set(names))
sheets=set()
for v in views:
 p=ROOT/'mods'/v['anim'].removeprefix('asset/');sheets.add(p)
 meta=json.loads(Path(str(p)+'#anim.fanim').read_text());frames=meta['anims'][v['tag']]['frames']
 with Image.open(str(p)+'#sheet.png') as im:
  assert max(im.size)<=2048,(p,im.size)
  for f in frames:
   d=f['data'];assert d['x']>=0 and d['y']>=0 and d['x']+d['w']<=im.width and d['y']+d['h']<=im.height
 duration=sum(f['duration'] for f in frames)
 if v['tag'].startswith(('note','meter','allocation')):assert duration>=(8+1)/60
 if v['tag'].startswith('packet'):assert duration>=(12+1)/60
 if v['tag'].startswith('field'):assert duration>=96/60
for s in script('generate.py').DATA['skills']:
 for n in range(len(s['tokens'])+1):assert 'tfm2_custom_unified_theory_note_'+s['id']+'_'+str(n) in names
# Regeneration must reproduce exact bytes, including merged localization.
paths=list((ROOT/'mods/tfm2_custom/vfx').glob('science_*'))+list((ROOT/'mods/tfm2_custom/champions').glob('tfm2_custom_unified_theory*'))+[ROOT/'mods/tfm2_custom/champion/tfm2_custom_unified_theory.data_champion',ROOT/'mods/tfm2_custom/text/champion.i18n',ROOT/'editor/science-portraits.png']
hashfiles=lambda:{str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in paths}
before=hashfiles();script('art.py').generate();script('data.py').generate();assert before==hashfiles(),'Generated assets drifted'
subprocess.run(['node','-e',"const fs=require('fs'),l=require('./editor/unified-theorylab.js');if(!l.selfTest())process.exit(1);console.log(l.validateVectors(fs.readFileSync('native/tfm2_custom_ai/src/unified_theory_vectors.txt','utf8'))+' exact native charge vectors; all 8 ranks deterministic');"],cwd=ROOT,check=True)
print(f'Verified {len(views)} views over {len(sheets)} VFX sheets; original body <=2048; notebook lifetimes; regeneration byte-identical.')
