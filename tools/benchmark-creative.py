"""Five matched release-core runs. No installation, career edits or host performance claims."""
from pathlib import Path
import subprocess,os,json,statistics
root=Path(__file__).resolve().parents[1];out=root/'docs/creative-results';out.mkdir(exist_ok=True)
def run(args,env=None):
 p=subprocess.run(['rtk','proxy',*args],cwd=root,capture_output=True,text=True,encoding='utf8',errors='replace',env=env)
 if p.returncode:raise RuntimeError(p.stdout+p.stderr)
 return p.stdout
cargo=['cargo','+stable-x86_64-pc-windows-gnu','run','--release','--manifest-path']
report={'hostMeasured':False,'renderingMeasured':False,'baseline':'Frozen pre-rework Steve core; same units, positions and 1800 ticks per run. Authorized kit/AI changes remain enabled.'}
for label,bin in [('steveBaseline','steve-bench-v1'),('steveCandidate','steve-bench')]:
 d=json.loads(run(cargo+['native/steve_lab/Cargo.toml','--bin',bin]));
 for crowd in d['crowds']:
  crowd['medianP95Ns']=statistics.median(r['p95Ns']for r in crowd['runs']);crowd['medianP99Ns']=statistics.median(r['p99Ns']for r in crowd['runs'])
 report[label]=d
report['universal']=json.loads(run(cargo+['native/universal_lab/Cargo.toml','--bin','universal-bench']))
report['magnus']=[]
for n in range(5):
 dest=root/f'backups/creative-magnus-run{n}';dest.mkdir(parents=True,exist_ok=True)
 run(cargo+['native/magnus_lab/Cargo.toml','--bin','vectors'],dict(os.environ,MAGNUS_OUTPUT_ROOT=str(dest)))
 stress=json.loads((dest/'docs/magnus-stress.json').read_text(encoding='utf8'));assert stress[-1]['p95StepUs']<4000,stress[-1]
 report['magnus'].append({'run':n,'stress':stress})
(out/'performance.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf8')
print(json.dumps({'steveP95Ms':[r['medianP95Ns']/1e6 for r in report['steveCandidate']['crowds']],'magnus128P95Ms':[r['stress'][-1]['p95StepUs']/1000 for r in report['magnus']],'universalP95Ms':[r['p95Ns']/1e6 for r in report['universal']['runs']],'host':'pending'}))
