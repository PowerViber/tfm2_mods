"""Rerun the unchanged Magnus release core without replacing its editor assets."""
from pathlib import Path
import json,subprocess,os
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'backups/gojo-v3-magnus';OUT.mkdir(parents=True,exist_ok=True)
env=dict(os.environ,MAGNUS_OUTPUT_ROOT=str(OUT))
p=subprocess.run(['rtk','proxy','rustup','run','stable-x86_64-pc-windows-gnu','cargo','run','--release','--manifest-path','native/magnus_lab/Cargo.toml','--bin','vectors'],cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf-8',errors='replace')
assert p.returncode==0,p.stdout+p.stderr
rows=json.loads((OUT/'docs/magnus-stress.json').read_text());assert rows[-1]['p95StepUs']<4000,rows[-1]
(ROOT/'docs/gojo-results/magnus-stress.json').write_text(json.dumps(rows,indent=2)+'\n');print(json.dumps(rows))
