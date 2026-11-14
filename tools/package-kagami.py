"""Reproduce and package PR #86's reserved 0.10.108 DLL. Never installs."""
from pathlib import Path
from concurrent.futures import ThreadPoolExecutor
import subprocess,json,os,hashlib,shutil
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/kagami-art-review'
BASE='07e5d41cfa226bd11302cff58efe8d3bbc6f503d'
def run(args,env=None):
    p=subprocess.run(['rtk','proxy',*args],cwd=ROOT,capture_output=True,text=True,encoding='utf8',errors='replace',env=env)
    if p.returncode:raise RuntimeError(p.stdout+p.stderr)
    return p.stdout.strip()
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def source_hash():
    paths=sorted((ROOT/'native/tfm2_custom_ai/src').rglob('*.rs'))
    paths += sorted((ROOT/'native/mod-api-stable/src').rglob('*.rs'))
    paths += [ROOT/'native/tfm2_custom_ai/Cargo.toml',ROOT/'native/tfm2_custom_ai/Cargo.lock',ROOT/'native/tfm2_custom_ai/mod.mod_info',ROOT/'native/mod-api-stable/Cargo.toml']
    return hashlib.sha256(b''.join(p.relative_to(ROOT).as_posix().encode()+p.read_bytes().replace(b'\r\n',b'\n') for p in paths)).hexdigest()
before=source_hash();epoch=run(['git','show','-s','--format=%ct',BASE])
env=dict(os.environ,SOURCE_DATE_EPOCH=epoch)
args=['cargo','+stable-x86_64-pc-windows-gnu','build','--locked','--release','--manifest-path','native/tfm2_custom_ai/Cargo.toml']
secondary=ROOT/'backups/kagami-art-repro-target'
with ThreadPoolExecutor(max_workers=2) as pool:
    jobs=[pool.submit(run,args,env),pool.submit(run,args,dict(env,CARGO_TARGET_DIR=str(secondary)))]
    for job in jobs:job.result()
assert before==source_hash(),'Sources changed during packaging'
built=ROOT/'native/tfm2_custom_ai/target/release/tfm2_custom_ai.dll';reproduced=secondary/'release/tfm2_custom_ai.dll'
assert sha(built)==sha(reproduced),'Builds differ'
meta=json.loads((ROOT/'native/tfm2_custom_ai/mod.mod_info').read_text())
assert meta==json.loads((ROOT/'mods/tfm2_custom_ai/mod.mod_info').read_text())
assert meta['version']=='0.10.108'
package=ROOT/'mods/tfm2_custom_ai/tfm2_custom_ai.dll';shutil.copyfile(built,package)
report=dict(native=meta['version'],champion_pack=json.loads((ROOT/'mods/tfm2_custom/mod.mod_info').read_text())['version'],
    rules=json.loads((ROOT/'rules/kagami.json').read_text())['revision'],baseCommit=BASE,sourceDateEpoch=epoch,nativeSourcesSha256=before,
    toolchain=run(['rustc','+stable-x86_64-pc-windows-gnu','--version']),
    builtDllSha256=sha(built),reproducedDllSha256=sha(reproduced),packagedDllSha256=sha(package),
    wasmSha256=sha(ROOT/'editor/kagami.wasm'),installed=False,careerMigrationApplied=False,
    liveAcceptance='Pending by user request')
(OUT/'package.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report,indent=2))
