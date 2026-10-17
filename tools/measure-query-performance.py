"""Five matched release runs of frozen-policy and batching benchmarks (synthetic host)."""
from pathlib import Path
import json,re,statistics,subprocess
ROOT=Path(__file__).resolve().parents[1]
runs=[]
for n in range(5):
    p=subprocess.run(['rtk','proxy','rustup','run','stable-x86_64-pc-windows-gnu','cargo','test','--release','--manifest-path','native/tfm2_custom_ai/Cargo.toml','--lib','benchmark','--','--ignored','--nocapture','--test-threads','1'],cwd=ROOT,capture_output=True,text=True,encoding='utf-8',errors='replace')
    if p.returncode:raise RuntimeError(p.stdout+p.stderr)
    result={k:json.loads(v) for k,v in re.findall(r'(QUERY_BENCH|BUSH_BENCH) (\{[^\n]+\})',p.stdout)}
    assert len(result)==2,p.stdout;runs.append(result);print('Measured release run',n+1)
summary={}
for name in runs[0]:
    summary[name]={side:{key:statistics.median(r[name][side][key] for r in runs) for key in runs[0][name][side]} for side in ['baseline','candidate']}
    old,new=summary[name]['baseline']['p95Ns'],summary[name]['candidate']['p95Ns'];summary[name]['p95ReductionPercent']=round((old-new)*100/old,2)
    assert new<=old*1.05,(name,'p95 regression',summary[name])
out=ROOT/'docs/objective-stark-results/query-performance.json';out.parent.mkdir(parents=True,exist_ok=True);out.write_text(json.dumps({'scope':'Synthetic stable-host callbacks and full-map bush search; not engine frame times','runs':runs,'medianOfFiveRuns':summary},indent=2)+'\n');print(json.dumps(summary,indent=2))
