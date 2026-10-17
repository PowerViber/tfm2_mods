"""Record actual Unified raw-equation behaviour for enlarged objective health."""
from pathlib import Path
import os,subprocess
ROOT=Path(__file__).resolve().parents[1]
out=ROOT/'docs/objective-stark-results/percentage-hp-equations.json';out.parent.mkdir(parents=True,exist_ok=True)
env=dict(os.environ,OBJECTIVE_PERCENT_REPORT=str(out))
p=subprocess.run(['rtk','proxy','rustup','run','stable-x86_64-pc-windows-gnu','cargo','test','--release','--manifest-path','native/tfm2_custom_ai/Cargo.toml','--lib','objective_hp','--quiet'],cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf-8',errors='replace')
if p.returncode:raise RuntimeError(p.stdout+p.stderr)
assert out.is_file();print(p.stdout);print(out.read_text())
