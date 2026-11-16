"""Reproduce PR #88's reserved 0.10.110 DLL twice and package it. Never installs."""
from pathlib import Path
from concurrent.futures import ThreadPoolExecutor
import subprocess,json,os,hashlib,shutil,tempfile
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/basalt-art-review'
BASE='3790d7ddb9578f3fc051e67bfa5f8f0bbb1f200a'
def run(args,env=None):
    p=subprocess.run(['rtk','proxy',*args],cwd=ROOT,capture_output=True,text=True,encoding='utf8',errors='replace',env=env)
    if p.returncode:raise RuntimeError(p.stdout+p.stderr)
    return p.stdout.strip()
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def source_hash():
    paths=sorted((ROOT/'native/tfm2_custom_ai/src').rglob('*.rs'))+sorted((ROOT/'native/mod-api-stable/src').rglob('*.rs'))
    paths += [ROOT/'native/tfm2_custom_ai/Cargo.toml',ROOT/'native/tfm2_custom_ai/Cargo.lock',ROOT/'native/mod-api-stable/Cargo.toml']
    return hashlib.sha256(b''.join(p.relative_to(ROOT).as_posix().encode()+p.read_bytes().replace(b'\r\n',b'\n') for p in paths)).hexdigest()
before=source_hash();epoch=run(['git','show','-s','--format=%ct',BASE]);env=dict(os.environ,SOURCE_DATE_EPOCH=epoch)
args=['cargo','+stable-x86_64-pc-windows-gnu','build','--locked','--release','--manifest-path','native/tfm2_custom_ai/Cargo.toml']
backups=ROOT/'backups';backups.mkdir(exist_ok=True);secondary=Path(tempfile.mkdtemp(prefix='basalt-repro-',dir=backups)).resolve();assert secondary.is_relative_to(backups.resolve())
with ThreadPoolExecutor(max_workers=2) as pool:
    jobs=[pool.submit(run,args,env),pool.submit(run,args,dict(env,CARGO_TARGET_DIR=str(secondary)))]
    for job in jobs:job.result()
assert before==source_hash(),'Sources changed during packaging'
built=ROOT/'native/tfm2_custom_ai/target/release/tfm2_custom_ai.dll';reproduced=secondary/'release/tfm2_custom_ai.dll'
assert sha(built)==sha(reproduced),'Independent DLL builds differ'
meta=json.loads((ROOT/'native/tfm2_custom_ai/mod.mod_info').read_text());assert meta==json.loads((ROOT/'mods/tfm2_custom_ai/mod.mod_info').read_text());assert meta['version']=='0.10.110'
package=ROOT/'mods/tfm2_custom_ai/tfm2_custom_ai.dll';shutil.copyfile(built,package)
report={'native':meta['version'],'champion_pack':json.loads((ROOT/'mods/tfm2_custom/mod.mod_info').read_text())['version'],'rules':json.loads((ROOT/'rules/basalt.json').read_text())['revision'],'baseline_commit':BASE,'source_date_epoch':epoch,'native_sources_sha256':before,'toolchain':run(['rustc','+stable-x86_64-pc-windows-gnu','--version']),'built_dll_sha256':sha(built),'reproduced_dll_sha256':sha(reproduced),'packaged_dll_sha256':sha(package),'wasm_sha256':sha(ROOT/'editor/basalt.wasm'),'body_sha256':sha(ROOT/'mods/tfm2_custom/champions/tfm2_custom_basalt#sheet.png'),'effects_sha256':sha(ROOT/'mods/tfm2_custom/vfx/basalt#sheet.png'),'installed':False,'career_migration_applied':False,'live_acceptance':'pending'}
(OUT/'package.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
