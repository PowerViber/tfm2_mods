"""Reproduce and package this checkout. Never installs mods or migrates careers."""
from pathlib import Path
from concurrent.futures import ThreadPoolExecutor
import subprocess,json,os,hashlib,shutil
root=Path(__file__).resolve().parents[1];out=root/'docs/creative-results';out.mkdir(exist_ok=True)
def run(args,env=None):
 p=subprocess.run(['rtk','proxy',*args],cwd=root,capture_output=True,text=True,encoding='utf8',errors='replace',env=env)
 if p.returncode:raise RuntimeError(p.stdout+p.stderr)
 return p.stdout.strip()
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
sources=sorted((root/'native/tfm2_custom_ai/src').glob('*.rs'))+[root/'native/tfm2_custom_ai/Cargo.toml',root/'native/tfm2_custom_ai/Cargo.lock']
def source_hash():return hashlib.sha256(b''.join(p.relative_to(root).as_posix().encode()+p.read_bytes()for p in sources)).hexdigest()
before=source_hash();epoch=run(['git','show','-s','--format=%ct','0365335']);env=dict(os.environ,SOURCE_DATE_EPOCH=epoch)
args=['cargo','+stable-x86_64-pc-windows-gnu','build','--release','--manifest-path','native/tfm2_custom_ai/Cargo.toml'];secondary=root/'backups/creative-repro-target'
with ThreadPoolExecutor(max_workers=2)as pool:
 jobs=[pool.submit(run,args,env),pool.submit(run,args,dict(env,CARGO_TARGET_DIR=str(secondary)))]
 for job in jobs:job.result()
assert before==source_hash(),'Native source changed during packaging'
built=root/'native/tfm2_custom_ai/target/release/tfm2_custom_ai.dll';reproduced=secondary/'release/tfm2_custom_ai.dll';assert sha(built)==sha(reproduced),'Reproduced DLL differs'
package=root/'mods/tfm2_custom_ai/tfm2_custom_ai.dll';shutil.copyfile(built,package)
meta=json.loads((root/'native/tfm2_custom_ai/mod.mod_info').read_text(encoding='utf8'));assert meta==json.loads((root/'mods/tfm2_custom_ai/mod.mod_info').read_text(encoding='utf8'))
report=dict(native=meta['version'],baseCommit='0365335',sourceDateEpoch=epoch,nativeSourcesSha256=before,toolchain=run(['rustc','+stable-x86_64-pc-windows-gnu','--version']),builtDllSha256=sha(built),reproducedDllSha256=sha(reproduced),packagedDllSha256=sha(package),labs={name:sha(root/f'editor/{name}.wasm')for name in ['steve','universal','frieren','gojo']},packs={p.parent.name:json.loads(p.read_text(encoding='utf8'))['version']for p in sorted((root/'mods').glob('*/mod.mod_info'))},installedByThisScript=False,careerMigrationApplied=False,liveAcceptance='Pending native targeting, carrier/overlay synchronization, items, training attribution, stock casts, strategy acceptance, and crowded host observations.')
(out/'package.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf8');print(json.dumps(report,indent=2))
