"""Two independent release builds, package comparison and isolated manager check."""
from pathlib import Path
from concurrent.futures import ThreadPoolExecutor
import subprocess,json,os,hashlib,shutil,tempfile
root=Path(__file__).resolve().parents[1]
out=root/'docs/raze-results';out.mkdir(parents=True,exist_ok=True)
def run(args,env=None):
 p=subprocess.run(['rtk','proxy',*args],cwd=root,capture_output=True,text=True,encoding='utf8',errors='replace',env=env)
 if p.returncode:raise RuntimeError(p.stdout+p.stderr)
 return p.stdout.strip()
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
sources=sorted((root/'native/tfm2_custom_ai/src').glob('*.rs'))+[root/'native/tfm2_custom_ai/Cargo.toml',root/'native/tfm2_custom_ai/Cargo.lock']
def source_hash():return hashlib.sha256(b''.join(p.relative_to(root).as_posix().encode()+p.read_bytes()for p in sources)).hexdigest()
before=source_hash();base='6415d0b'
epoch=run(['git','show','-s','--format=%ct',base]);env=dict(os.environ,SOURCE_DATE_EPOCH=epoch)
args=['cargo','+stable-x86_64-pc-windows-gnu','build','--release','--manifest-path','native/tfm2_custom_ai/Cargo.toml'];secondary=root/'backups/raze-repro-target'
with ThreadPoolExecutor(max_workers=2) as pool:
 for job in [pool.submit(run,args,env),pool.submit(run,args,dict(env,CARGO_TARGET_DIR=str(secondary)))]:job.result()
assert before==source_hash(),'Sources changed during packaging'
built=root/'native/tfm2_custom_ai/target/release/tfm2_custom_ai.dll';reproduced=secondary/'release/tfm2_custom_ai.dll'
assert sha(built)==sha(reproduced),'Independent DLL builds differ'
package=root/'mods/tfm2_custom_ai/tfm2_custom_ai.dll';shutil.copyfile(built,package)
meta=json.loads((root/'native/tfm2_custom_ai/mod.mod_info').read_text(encoding='utf8'))
assert meta==json.loads((root/'mods/tfm2_custom_ai/mod.mod_info').read_text(encoding='utf8'))
report=dict(native=meta['version'],baseCommit=base,sourceDateEpoch=epoch,nativeSourcesSha256=before,toolchain=run(['rustc','+stable-x86_64-pc-windows-gnu','--version']),builtDllSha256=sha(built),reproducedDllSha256=sha(reproduced),packagedDllSha256=sha(package),labWasmSha256=sha(root/'editor/raze.wasm'),packs={p.parent.name:json.loads(p.read_text(encoding='utf8'))['version']for p in sorted((root/'mods').glob('*/mod.mod_info'))},installedByThisScript=False,careerMigrationApplied=False,liveAcceptance='Pending native carrier targeting, overlay coverage, damage/items/attribution and crowded host observations.')
(out/'package.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf8')
print('Reproduced packaged DLL: '+sha(package),flush=True)
backup=root/'backups';backup.mkdir(exist_ok=True)
fixture=Path(tempfile.mkdtemp(prefix='raze-manager-',dir=backup)).resolve()
assert fixture.is_relative_to(backup.resolve())
(fixture/'bundle.game_data').touch();shutil.copytree(root/'mods',fixture/'mods')
compared=[]
for source in sorted((root/'mods').rglob('*')):
 if not source.is_file():continue
 relative=source.relative_to(root/'mods')
 assert sha(source)==sha(fixture/'mods'/relative),str(relative)
 compared.append(relative.as_posix())
valorant=[p for p in compared if p.startswith('tfm2_valorant/')]
assert 'tfm2_valorant/champion/tfm2_valorant_raze.data_champion'in valorant
config=fixture/'config/game';config.mkdir(parents=True)
(config/'mods.json').write_text(json.dumps({'enabled_mods':['tfm2_custom','tfm2_custom_ai','tfm2_valorant']}))
cfg=root/'manager.cfg';old=cfg.read_bytes()if cfg.exists()else None
try:
 p=subprocess.run(['rtk','proxy',str(root/'TFM2 Mod Manager.exe'),'--check','--yes','--game',str(fixture)],cwd=root,capture_output=True,text=True,encoding='utf8',errors='replace')
finally:
 if old is not None:cfg.write_bytes(old)
 elif cfg.exists():cfg.unlink()
output=p.stdout+p.stderr
(out/'mod-manager.txt').write_text(output,encoding='utf8')
result={'scope':'Isolated fixture. Manager compares tfm2_custom and the native DLL; supplementary SHA-256 comparison covers every copied pack including Valorant. Installed game/careers unchanged.','allPackFilesCompared':len(compared),'valorantFilesCompared':len(valorant),'allPackDifferences':0,'exitCode':p.returncode,'packagedDllSha256':sha(fixture/'mods/tfm2_custom_ai/tfm2_custom_ai.dll'),'result':[s.strip()for s in output.splitlines()if any(x in s for x in ['champion files compared','tfm2_custom_ai','tfm2_valorant','UP TO DATE:'])]}
(out/'mod-manager.json').write_text(json.dumps(result,indent=2)+'\n',encoding='utf8')
assert p.returncode==0,output
assert 'NOT UP TO DATE:'not in output,output
print(json.dumps(result,indent=2))
