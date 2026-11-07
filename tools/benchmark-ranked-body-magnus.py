"""Five existing Magnus core stress runs; no host/rendering claims."""
from pathlib import Path
import subprocess,os,json,statistics
ROOT=Path(__file__).resolve().parents[1];runs=[]
target=ROOT/'backups/ranked-body-magnus-benchmark'
for n in range(5):
 subprocess.run(['rtk','proxy',str(ROOT/'native/magnus_lab/target/release/vectors.exe')],cwd=ROOT,env=dict(os.environ,MAGNUS_OUTPUT_ROOT=str(target)),capture_output=True,check=True)
 rows=json.loads((target/'docs/magnus-stress.json').read_text());assert rows[-1]['pieces']==128 and rows[-1]['p95StepUs']<4000,rows[-1];runs.append(rows)
report={'scope':'Existing Magnus release-core fixture, excluding host and rendering','fiveRuns':runs,'median128PieceP95StepUs':statistics.median(r[-1]['p95StepUs']for r in runs),'targetUs':4000}
(ROOT/'docs/ranked-body-redesign/magnus-performance.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
