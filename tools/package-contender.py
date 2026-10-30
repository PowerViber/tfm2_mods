"""Reproduce/package the playtest DLL; never install or migrate a career."""
from pathlib import Path
import hashlib, json, subprocess, os, shutil
ROOT=Path(__file__).resolve().parents[1]; OUT=ROOT/'docs/contender-results';OUT.mkdir(parents=True,exist_ok=True)
BASE='6688f74cb5e931816d710925e778f97fd069ec81'
def run(args,env=None):
    r=subprocess.run(['rtk','proxy',*args],cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf8',errors='replace')
    if r.returncode:raise RuntimeError(r.stdout+r.stderr)
    return r.stdout.strip()
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
inputs=sorted((ROOT/'native/tfm2_custom_ai/src').glob('*.rs'))+sorted((ROOT/'rules').glob('*.json'))+[ROOT/'native/tfm2_custom_ai/Cargo.lock']
def sourcehash():return hashlib.sha256(b''.join(p.relative_to(ROOT).as_posix().encode()+p.read_bytes() for p in inputs)).hexdigest()
before=sourcehash();epoch=run(['git','show','-s','--format=%ct',BASE]);env=dict(os.environ,SOURCE_DATE_EPOCH=epoch)
args=['cargo','+stable-x86_64-pc-windows-gnu','build','--release','--manifest-path','native/tfm2_custom_ai/Cargo.toml']
run(args,env);run(args,dict(env,CARGO_TARGET_DIR=str(ROOT/'backups/contender-repro-target')))
a=ROOT/'native/tfm2_custom_ai/target/release/tfm2_custom_ai.dll';b=ROOT/'backups/contender-repro-target/release/tfm2_custom_ai.dll'
assert before==sourcehash(),'Source changed while building'
assert sha(a)==sha(b),'Independent DLLs differ'
package=ROOT/'mods/tfm2_custom_ai/tfm2_custom_ai.dll';shutil.copyfile(a,package)
report={'base':BASE,'scope':'Playable Contender candidate; native acceptance pending','native':'0.10.80','champions':'0.2.80','sourceHash':before,'sourceDateEpoch':epoch,'toolchain':run(['rustc','+stable-x86_64-pc-windows-gnu','--version']),'builtSha256':sha(a),'reproducedSha256':sha(b),'packagedSha256':sha(package),'labWasmSha256':sha(ROOT/'editor/boxing.wasm'),'installed':False,'careerMigrationApplied':False,'liveAcceptance':'pending'}
(OUT/'package.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
