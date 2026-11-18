"""Reproduce the reserved PR91 DLL; package only. Never install or migrate."""
from pathlib import Path
from concurrent.futures import ThreadPoolExecutor
import hashlib,json,os,shutil,subprocess,tempfile
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/os.environ.get('CONTENDER_RESULTS','docs/contender-shadow-review');OUT.mkdir(exist_ok=True)
BASE=os.environ.get('CONTENDER_BASE','dc0e0b7b2de8a326e80ff9d7fd5d226a9fd423a1')
def run(args,env=None):
    p=subprocess.run(['rtk','proxy',*args],cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf8',errors='replace')
    if p.returncode:raise RuntimeError(p.stdout+p.stderr)
    return (p.stdout+p.stderr).strip()
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def sources():
    ps=sorted((ROOT/'native/tfm2_custom_ai/src').rglob('*.rs'))+sorted((ROOT/'native/mod-api-stable/src').rglob('*.rs'))
    ps += [ROOT/'native/tfm2_custom_ai/Cargo.toml',ROOT/'native/tfm2_custom_ai/Cargo.lock',ROOT/'native/mod-api-stable/Cargo.toml']
    return hashlib.sha256(b''.join(p.relative_to(ROOT).as_posix().encode()+p.read_bytes().replace(b'\r\n',b'\n') for p in ps)).hexdigest()
before=sources();epoch=run(['git','show','-s','--format=%ct',BASE]);env=dict(os.environ,SOURCE_DATE_EPOCH=epoch)
backups=ROOT/'backups';backups.mkdir(exist_ok=True);second=Path(tempfile.mkdtemp(prefix='contender-shadow-repro-',dir=backups)).resolve();assert second.is_relative_to(backups.resolve())
args=['cargo','+stable-x86_64-pc-windows-gnu','build','--locked','--release','--manifest-path','native/tfm2_custom_ai/Cargo.toml']
with ThreadPoolExecutor(max_workers=2) as pool:
    jobs=[pool.submit(run,args,env),pool.submit(run,args,dict(env,CARGO_TARGET_DIR=str(second)))]
    for i,j in enumerate(jobs):(OUT/f'dll-build-{i+1}.txt').write_text(j.result()+'\n')
assert before==sources(),'Sources changed during build'
built=ROOT/'native/tfm2_custom_ai/target/release/tfm2_custom_ai.dll';repro=second/'release/tfm2_custom_ai.dll';assert sha(built)==sha(repro),'Reproduction differs'
meta=json.loads((ROOT/'mods/tfm2_custom_ai/mod.mod_info').read_text());assert meta['version']=='0.10.113'
assert json.loads((ROOT/'mods/tfm2_custom/mod.mod_info').read_text())['version']=='0.2.116'
package=ROOT/'mods/tfm2_custom_ai/tfm2_custom_ai.dll';shutil.copyfile(built,package)
report={'native':'0.10.113','champion_pack':'0.2.116','rules':json.loads((ROOT/'rules/contender.json').read_text())['revision'],'baseline':BASE,'source_date_epoch':epoch,'source_sha256':before,
 'built_dll_sha256':sha(built),'reproduced_dll_sha256':sha(repro),'packaged_dll_sha256':sha(package),'wasm_sha256':sha(ROOT/'editor/contender.wasm'),
 'installed':False,'career_migration':False,'actual_game_acceptance':'pending','toolchain':run(['rustc','+stable-x86_64-pc-windows-gnu','--version'])}
(OUT/'package.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
