"""Repeat the existing unchanged Magnus release-core stress fixture, with isolated output."""
from pathlib import Path
import subprocess,os,json,statistics
ROOT=Path(__file__).resolve().parents[1];runs=[]
for n in range(5):
    target=ROOT/'backups/contender-magnus-benchmark'
    env=dict(os.environ,MAGNUS_OUTPUT_ROOT=str(target))
    subprocess.run(['rtk','proxy',str(ROOT/'native/magnus_lab/target/release/vectors.exe')],cwd=ROOT,env=env,capture_output=True,check=True)
    rows=json.loads((target/'docs/magnus-stress.json').read_text());assert rows[-1]['pieces']==128 and rows[-1]['p95StepUs']<4000,rows[-1]
    runs.append(rows)
report={'scope':'Unchanged Magnus release core; excludes native host, rendering and crowded-match FPS','fiveRuns':runs,'median128PieceP95StepUs':statistics.median(r[-1]['p95StepUs']for r in runs),'targetUs':4000}
(ROOT/'docs/contender-results/magnus-performance.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
