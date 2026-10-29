"""Run an identical combat harness against merged main and the candidate."""
from pathlib import Path
import subprocess,json,hashlib,statistics
root=Path(__file__).resolve().parents[1];fixture=root/'backups/raze-main-comparison';src=fixture/'src';src.mkdir(parents=True,exist_ok=True)
base='73c4274c7918bfb00a5998f60fa204ef28d464be'
def run(args):
 p=subprocess.run(['rtk','proxy',*args],cwd=root,capture_output=True,text=True,encoding='utf8',errors='replace')
 if p.returncode:raise RuntimeError(p.stdout+p.stderr)
 return p.stdout
hashes={}
for name in ['raze_core.rs','raze_data.rs','valorant_guns.rs','valorant_gun_data.rs']:
 text=run(['git','show',base+':native/tfm2_custom_ai/src/'+name]);hashes[name]=hashlib.sha256(text.encode()).hexdigest()
 (src/name).write_text(text,encoding='utf8')
(src/'lib.rs').write_text('#![allow(dead_code)]\npub mod raze_core;\npub mod raze_data;\npub mod valorant_guns;\n')
(src/'main.rs').write_text((root/'native/raze_lab/src/bench.rs').read_text(),encoding='utf8')
(fixture/'Cargo.toml').write_text('[package]\nname="raze_lab"\nversion="0.1.0"\nedition="2021"\n[dependencies]\nserde={version="1.0",features=["derive"]}\nserde_json="1.0"\n')
print('Building merged-main baseline with the shared harness',flush=True)
baseline=json.loads(run(['cargo','+stable-x86_64-pc-windows-gnu','run','--release','--manifest-path',str(fixture/'Cargo.toml')]))
candidate=json.loads(run([str(root/'native/raze_lab/target/release/raze-bench.exe')]))
keys=['requestedRaw','labEstimatedDelivered','exposureTicks','missedWaveObservations','objectiveEstimatedDamage','deaths','weaponSpent','armourSpent','utilitySpent','followupDamage','entriesWithFollowup','airborneTicks','landingGuardTicks','airDamagePreventedEstimate']
def summary(d):
 result={k:sum(c[k] for c in d['scenarios'])for k in keys}
 result.update(launches=sum(c['core']['launches']for c in d['scenarios']),wastedPreparation=sum(c['core']['wasted_preparation']for c in d['scenarios']),medianP95Ms=statistics.median(c['p95Ms']for c in d['runs']))
 return result
report=dict(baselineCommit=base,baselineSourcesSha256=hashes,matchedPairs=len(candidate['scenarios']),developmentSeeds=24,holdoutSeeds=12,profiles=[20,50,80],teams=[0,1],repetitions=5,harnessSha256=hashlib.sha256((src/'main.rs').read_bytes()).hexdigest(),baseline=summary(baseline),candidate=summary(candidate),scope='Identical armour/MR harness with allied basic follow-up and current Mod Power. Not engine-attributed game damage or a win-rate study. Baseline combat, economy and AI sources are unchanged; the identical harness observes results.')
report['economy']={'baseline':baseline['economy'],'candidate':candidate['economy']}
dest=root/'docs/raze-results';dest.mkdir(parents=True,exist_ok=True)
(dest/'comparison.json').write_text(json.dumps(report,indent=2)+'\n');(fixture/'baseline.json').write_text(json.dumps(baseline));print(json.dumps(report,indent=2))
