"""Five runs of the unchanged Magnus core fixture; output stays in a disposable directory."""
from pathlib import Path
import json,os,statistics,subprocess,tempfile
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/thalassa-art-review'
backup=ROOT/'backups';backup.mkdir(exist_ok=True)
target=Path(tempfile.mkdtemp(prefix='thalassa-magnus-benchmark-',dir=backup)).resolve()
assert target.is_relative_to(backup.resolve())
runs=[]
for n in range(5):
    subprocess.run(['rtk','proxy',str(ROOT/'native/magnus_lab/target/release/vectors.exe')],cwd=ROOT,env=dict(os.environ,MAGNUS_OUTPUT_ROOT=str(target)),capture_output=True,check=True)
    rows=json.loads((target/'docs/magnus-stress.json').read_text(encoding='utf-8'))
    assert rows[-1]['pieces']==128 and rows[-1]['p95StepUs']<4000,rows[-1]
    runs.append(rows)
report={'scope':'Unchanged Magnus release core; excludes native host, rendering and crowded-game FPS',
        'five_runs':runs,'median_128_piece_p95_us':statistics.median(r[-1]['p95StepUs'] for r in runs),'target_us':4000}
(OUT/'magnus-performance.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
print(json.dumps({'median_128_piece_p95_us':report['median_128_piece_p95_us'],'five_p95_us':[r[-1]['p95StepUs'] for r in runs],'target_us':4000}))
