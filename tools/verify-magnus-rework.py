"""Rebuild the shared Magnus lab and repeat trajectories, 36 scenarios and army stress."""
from pathlib import Path
import gzip,json,os,shutil,subprocess
ROOT=Path(__file__).resolve().parents[1]
OUTPUT=ROOT/'backups/rework-magnus-results';OUTPUT.mkdir(parents=True,exist_ok=True)
def run(args,env=None):
 p=subprocess.run(['rtk',*args],cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf-8',errors='replace')
 if p.returncode:raise RuntimeError(p.stdout+p.stderr)
 return p.stdout
prefix=['proxy','rustup','run','stable-x86_64-pc-windows-gnu','cargo']
print(run(prefix+['build','--release','--target','wasm32-unknown-unknown','--manifest-path','native/magnus_lab/Cargo.toml']))
shutil.copyfile(ROOT/'native/magnus_lab/target/wasm32-unknown-unknown/release/magnus_lab.wasm',ROOT/'editor/magnus.wasm')
env=dict(os.environ,MAGNUS_OUTPUT_ROOT=str(OUTPUT));print(run(prefix+['run','--release','--manifest-path','native/magnus_lab/Cargo.toml','--bin','vectors'],env))
result=ROOT/'docs/rework-results';result.mkdir(parents=True,exist_ok=True)
for name in ['magnus-scenarios.json','magnus-stress.json']:shutil.copyfile(OUTPUT/'docs'/name,result/name)
with (ROOT/'editor/magnus-vectors.jsonl.gz').open('wb')as f:f.write(gzip.compress((OUTPUT/'backups/magnus-vectors-v4.jsonl').read_bytes(),mtime=0))
print(run(['node','tools/verify-magnus.js']))
stress=json.loads((result/'magnus-stress.json').read_text());assert stress[-1]['p95StepUs']<4000,stress[-1]
print('PASS: 128-piece release-core p95 below 4 ms; native host/rendering excluded.')
