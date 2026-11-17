"""Build native 0.10.112 twice, compare and package. No installation or migration."""
from pathlib import Path
from concurrent.futures import ThreadPoolExecutor
import hashlib, json, os, shutil, subprocess, tempfile
ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/'docs/basalt-flicker';OUT.mkdir(exist_ok=True)
BASE='79484f81d90aabe9ca86542a521e747fb7935b7b'
def run(args,env=None):
    p=subprocess.run(['rtk','proxy',*args],cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf8',errors='replace')
    if p.returncode:raise RuntimeError(p.stdout+p.stderr)
    return p.stdout.strip()
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def sources():
    paths=sorted((ROOT/'native/tfm2_custom_ai/src').rglob('*.rs'))+sorted((ROOT/'native/mod-api-stable/src').rglob('*.rs'))
    paths += [ROOT/'native/tfm2_custom_ai/Cargo.toml',ROOT/'native/tfm2_custom_ai/Cargo.lock',ROOT/'native/mod-api-stable/Cargo.toml']
    return hashlib.sha256(b''.join(p.relative_to(ROOT).as_posix().encode()+p.read_bytes().replace(b'\r\n',b'\n') for p in paths)).hexdigest()
before=sources();epoch=run(['git','show','-s','--format=%ct',BASE]);env=dict(os.environ,SOURCE_DATE_EPOCH=epoch)
backups=ROOT/'backups';backups.mkdir(exist_ok=True)
secondary=Path(tempfile.mkdtemp(prefix='basalt-continuity-repro-',dir=backups)).resolve()
assert secondary.is_relative_to(backups.resolve())
args=['cargo','+stable-x86_64-pc-windows-gnu','build','--locked','--release','--manifest-path','native/tfm2_custom_ai/Cargo.toml']
with ThreadPoolExecutor(max_workers=2) as pool:
    jobs=[pool.submit(run,args,env),pool.submit(run,args,dict(env,CARGO_TARGET_DIR=str(secondary)))]
    for i,job in enumerate(jobs):(OUT/f'dll-build-{i+1}.txt').write_text(job.result()+'\n')
assert before==sources(),'Native source changed while building'
built=ROOT/'native/tfm2_custom_ai/target/release/tfm2_custom_ai.dll'
reproduced=secondary/'release/tfm2_custom_ai.dll'
assert sha(built)==sha(reproduced),'Independent builds differ'
meta=json.loads((ROOT/'native/tfm2_custom_ai/mod.mod_info').read_text())
assert meta==json.loads((ROOT/'mods/tfm2_custom_ai/mod.mod_info').read_text()) and meta['version']=='0.10.112'
package=ROOT/'mods/tfm2_custom_ai/tfm2_custom_ai.dll';shutil.copyfile(built,package)
report={'native':meta['version'],'champion_pack':json.loads((ROOT/'mods/tfm2_custom/mod.mod_info').read_text())['version'],
        'rules':'basalt-1.4','art_revision':'basalt-art-6','baseline':BASE,'source_date_epoch':epoch,
        'source_sha256':before,'toolchain':run(['rustc','+stable-x86_64-pc-windows-gnu','--version']),
        'built_dll_sha256':sha(built),'reproduced_dll_sha256':sha(reproduced),'packaged_dll_sha256':sha(package),
        'wasm_sha256':sha(ROOT/'editor/basalt.wasm'),'effects_fanim_sha256':sha(ROOT/'mods/tfm2_custom/vfx/basalt#anim.fanim'),
        'effects_png_sha256':sha(ROOT/'mods/tfm2_custom/vfx/basalt#sheet.png'),
        'installed':False,'career_migration':False,'live_acceptance':'separate disposable-game evidence'}
(OUT/'package.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
