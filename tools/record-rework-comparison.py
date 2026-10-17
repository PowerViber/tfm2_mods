"""Record the same stationary passive-loop fixture on baseline and candidate source.

These are raw mock-host results, not engine-attributed damage, win rates or live acceptance.
"""
from pathlib import Path
import argparse,json,re,subprocess
ROOT=Path(__file__).resolve().parents[1]
def run(root):
 p=subprocess.run(['rtk','proxy','rustup','run','stable-x86_64-pc-windows-gnu','cargo','test','--release','--manifest-path','native/tfm2_custom_ai/Cargo.toml','mastery_is_the_power_source','--','--nocapture'],cwd=root,capture_output=True,text=True,encoding='utf-8',errors='replace')
 if p.returncode:raise RuntimeError(p.stdout+p.stderr)
 rows=[]
 for distance,low,high in re.findall(r'60 s duel at (\d+) raw / casts / accidents / resonances / worst 2 s window: Student \(([^)]+)\), Unified Mind \(([^)]+)\)',p.stdout+p.stderr):
  rows.append({'distance':int(distance),'student':[int(v) for v in low.split(',')],'unifiedMind':[int(v)for v in high.split(',')]})
 assert len(rows)==4
 return rows
if __name__=='__main__':
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--baseline',type=Path,required=True);args=a.parse_args();report={'scope':__doc__.split('\n\n')[1].strip(),'fixture':'60 seconds, 400 AP / 300 attack, 1000-HP stationary enemy, identical positions and seed; no enemy AI or mitigation. Baseline has automatic resonance; candidate requires an unlocked ultimate and this fixture leaves it locked.','columns':['rawDamage','successfulStages','backlashes','automaticResonanceOrGrandResolutions','worst120TickRaw'],'baselineVersion':'0.10.54','candidateVersion':'0.10.55','baseline':run(args.baseline),'candidate':run(ROOT)}
 out=ROOT/'docs/rework-results/unified-comparison.json';out.parent.mkdir(parents=True,exist_ok=True);out.write_text(json.dumps(report,indent=2)+'\n');print('Recorded four matched baseline/candidate fixtures:',out)
