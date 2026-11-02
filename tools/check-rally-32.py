"""Repeatable build/test/performance evidence. No package, installation or career access."""
from pathlib import Path
import hashlib,json,os,re,shutil,statistics,subprocess
ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/'docs/rally-results/rally-3.2';OUT.mkdir(parents=True,exist_ok=True)
def run(args,name,cwd=ROOT,env=None):
    p=subprocess.run(['rtk','proxy',*args],cwd=cwd,env=env,capture_output=True,text=True,encoding='utf-8',errors='replace')
    text=p.stdout+p.stderr;(OUT/(name+'.log')).write_text(text,encoding='utf-8')
    if p.returncode:raise RuntimeError(name+' failed: '+text[-1600:])
    return p.stdout
prefix=['cargo','+stable-x86_64-pc-windows-gnu']
run(['python','tools/generate-rally.py','--check'],'canonical')
native=run(prefix+['test','--release','--manifest-path','native/tfm2_custom_ai/Cargo.toml','--','--test-threads','1'],'native-tests-serial')
lab=run(prefix+['test','--release','--manifest-path','native/rally_lab/Cargo.toml'],'lab-tests')
run(prefix+['build','--release','--manifest-path','native/rally_lab/Cargo.toml','--bin','rally_runner'],'runner-build')
run(prefix+['build','--release','--target','wasm32-unknown-unknown','--manifest-path','native/rally_lab/Cargo.toml','--lib'],'wasm-build')
wasm=ROOT/'native/rally_lab/target/wasm32-unknown-unknown/release/rally_lab.wasm'
shutil.copyfile(wasm,ROOT/'editor/rally.wasm')
env=dict(os.environ,CARGO_TARGET_DIR=str(ROOT/'backups/rally-3.2-repro'))
run(prefix+['build','--release','--target','wasm32-unknown-unknown','--manifest-path','native/rally_lab/Cargo.toml','--lib'],'wasm-repro',env=env)
other=ROOT/'backups/rally-3.2-repro/wasm32-unknown-unknown/release/rally_lab.wasm'
assert wasm.read_bytes()==other.read_bytes(),'Independent WASM build differs'
run(['node','tools/test-rally-lab.cjs'],'browser')
run(['node','tools/test-rally-editor-route.cjs'],'editor-route')
run(['python','tools/verify-rally.py'],'art')
scenarios=json.loads(run(prefix+['run','--release','--quiet','--manifest-path','native/rally_lab/Cargo.toml','--bin','scenarios'],'scenarios'))
(OUT/'scenarios.json').write_text(json.dumps(scenarios,indent=2)+'\n')
vectors=ROOT/'native/magnus_lab/target/release/vectors.exe'
if not vectors.exists():run(prefix+['build','--release','--manifest-path','native/magnus_lab/Cargo.toml','--bin','vectors'],'magnus-build')
magnus=[]
for n in range(5):
    target=ROOT/'backups/rally-3.2-magnus'/str(n);target.mkdir(parents=True,exist_ok=True)
    run([str(vectors)],'magnus-'+str(n),env=dict(os.environ,MAGNUS_OUTPUT_ROOT=str(target)))
    rows=json.loads((target/'docs/magnus-stress.json').read_text());assert rows[-1]['pieces']==128 and rows[-1]['p95StepUs']<4000
    magnus.append(rows)
(OUT/'magnus-performance.json').write_text(json.dumps({'scope':'Existing Magnus release core; native host/rendering excluded','fiveRuns':magnus,'median128PieceP95StepUs':statistics.median(r[-1]['p95StepUs']for r in magnus)},indent=2)+'\n')
counts=lambda text:dict(zip(['passed','failed','ignored'],map(int,re.search(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored',text).groups())))
report={'rules':'rally-3.2','nativeTests':counts(native),'labTests':counts(lab),
    'wasmSha256':hashlib.sha256(wasm.read_bytes()).hexdigest(),'wasmReproducible':True,
    'development':24,'holdout':12,
    'rally128UnitMedianP95Us':statistics.median(r['p95Us'] for r in scenarios['release128UnitStepFiveRuns']),
    'rallyActiveStorm128UnitMedianP95Us':statistics.median(r['p95Us'] for r in scenarios['activeImpossibleStorm128UnitFiveRuns']),
    'magnus128PieceMedianP95Us':statistics.median(r[-1]['p95StepUs']for r in magnus),
    'scope':'Automated core/Lab evidence only; no native receipt calibration, shipped-art integration, DLL package, installation or live acceptance'}
(OUT/'automated-checks.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
