"""Record release checks without installing mods or altering a career."""
from pathlib import Path
import json, subprocess, os, statistics, sys, re

root=Path(__file__).resolve().parents[1]
out=root/'docs/raze-results'
out.mkdir(parents=True,exist_ok=True)
def run(label,args,env=None,record=True):
 p=subprocess.run(['rtk','proxy',*args],cwd=root,capture_output=True,text=True,encoding='utf8',errors='replace',env=env)
 text=p.stdout+p.stderr
 if record:(out/f'{label}.txt').write_text(text,encoding='utf8')
 if p.returncode:raise RuntimeError(f'{label}: {text[-6000:]}')
 return text

if '--performance-only'not in sys.argv:
 for label,args in [
  ('art-verification',['python','-X','utf8','tools/verify-raze.py']),
  ('farming-catalogue',['node','tools/test-farming-catalogue.cjs']),
  ('lab-tests',['python','-X','utf8','tools/build-raze.py']),
 ]:
  run(label,args)
  print(label+': passed',flush=True)
 pairs=run('athlete-pairs',['cargo','+stable-x86_64-pc-windows-gnu','test','--release','--manifest-path','native/raze_lab/Cargo.toml','all_thirteen_stats_have_three_independent_behavior_pairs','--','--nocapture'])
 pairs=[{'attribute':name,'pairedFixtureSeeds':json.loads(seeds),'low':20,'high':80,'otherTwelve':50}for name,seeds in re.findall(r'(?m)^(.+): paired fixture seeds (\[.+\])',pairs)]
 assert len(pairs)==13
 (out/'athlete-pairs.json').write_text(json.dumps(pairs,indent=2)+'\n',encoding='utf8')
jobs=[
 ('native-tests',['cargo','+stable-x86_64-pc-windows-gnu','test','--release','--manifest-path','native/tfm2_custom_ai/Cargo.toml']),
 ('browser',['node','tools/test-raze-lab.cjs']),
 ('keyboard',['node','tools/test-raze-keyboard.cjs']),
 ('v3-browser',['node','tools/test-raze-v3.cjs']),
 ('shop',['node','tools/test-raze-shop.cjs']),
 ('migration',['python','-X','utf8','-m','unittest','discover','-s','tools','-p','test_rework_migration.py']),
]
if '--performance-only'in sys.argv:jobs=[]
# These browser suites share generated screenshots and report paths, and Windows
# can hold mapped files while builds run. Run them serially for reliable evidence.
for label,args in jobs:
 text=run(label,args)
 print(label+': '+next((s for s in text.splitlines()if 'test result: ok.' in s),text.splitlines()[-1])[:350],flush=True)

data=json.loads(run('raze-bench',[str(root/'native/raze_lab/target/release/raze-bench.exe')],record=False))
data['medianP95Ms']=statistics.median(r['p95Ms']for r in data['runs'])
data['medianP99Ms']=statistics.median(r['p99Ms']for r in data['runs'])
# One matched case set is retained; the five repetitions time identical inputs.
data['aggregates']={}
for level in [20,50,80]:
 cases=[s for s in data['scenarios']if s['level']==level]
 data['aggregates'][str(level)]={k:sum(c[k]for c in cases)for k in ['requestedRaw','labEstimatedDelivered','exposureTicks','missedWaveObservations','waveUnitsDefeated','objectiveEstimatedDamage','deaths','controlRequests']}
 data['aggregates'][str(level)]['casts']=sum(c['core']['casts']for c in cases)
 data['aggregates'][str(level)]['launches']=sum(c['core']['launches']for c in cases)
 data['aggregates'][str(level)]['wastedPreparation']=sum(c['core']['wasted_preparation']for c in cases)
(out/'performance.json').write_text(json.dumps(data,indent=2)+'\n',encoding='utf8')
print('Raze core p95/p99 ms: '+str((data['medianP95Ms'],data['medianP99Ms'])),flush=True)
magnus=[]
for n in range(5):
 dest=root/f'backups/raze-magnus-run{n}'
 dest.mkdir(parents=True,exist_ok=True)
 run(f'magnus-{n}',['cargo','+stable-x86_64-pc-windows-gnu','run','--release','--manifest-path','native/magnus_lab/Cargo.toml','--bin','vectors'],dict(os.environ,MAGNUS_OUTPUT_ROOT=str(dest)))
 stress=json.loads((dest/'docs/magnus-stress.json').read_text(encoding='utf8'))
 assert stress[-1]['pieces']==128 and stress[-1]['p95StepUs']<4000,stress[-1]
 magnus.append({'run':n+1,'stress':stress})
(out/'magnus-performance.json').write_text(json.dumps(magnus,indent=2)+'\n',encoding='utf8')
print('Magnus 128-piece p95 ms: '+str([r['stress'][-1]['p95StepUs']/1000 for r in magnus]))
