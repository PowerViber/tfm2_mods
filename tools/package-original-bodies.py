"""Reproducible native presentation build and isolated Mod Manager verification.

Never installs or migrates a career. The manager sees only a temporary fixture.
"""
from pathlib import Path
from concurrent.futures import ThreadPoolExecutor
import subprocess,json,os,hashlib,shutil,tempfile,argparse
parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument('--base',default='02dba54');parser.add_argument('--out',default='docs/original-body-review')
parser.add_argument('--fixture-prefix',default='original-body-manager-')
options=parser.parse_args()
root=Path(__file__).resolve().parents[1];out=root/options.out;out.mkdir(parents=True,exist_ok=True)
def run(args,env=None):
 p=subprocess.run(['rtk','proxy',*args],cwd=root,capture_output=True,text=True,encoding='utf8',errors='replace',env=env)
 if p.returncode:raise RuntimeError(p.stdout+p.stderr)
 return p.stdout.strip()
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
sources=sorted((root/'native/tfm2_custom_ai/src').glob('*.rs'))+[root/'native/tfm2_custom_ai/Cargo.toml',root/'native/tfm2_custom_ai/Cargo.lock']
def source_hash():return hashlib.sha256(b''.join(p.relative_to(root).as_posix().encode()+p.read_bytes()for p in sources)).hexdigest()
before=source_hash();epoch=run(['git','show','-s','--format=%ct',options.base]);env=dict(os.environ,SOURCE_DATE_EPOCH=epoch)
args=['cargo','+stable-x86_64-pc-windows-gnu','build','--release','--manifest-path','native/tfm2_custom_ai/Cargo.toml']
secondary=root/'backups/original-bodies-repro-target'
with ThreadPoolExecutor(max_workers=2) as pool:
 for job in [pool.submit(run,args,env),pool.submit(run,args,dict(env,CARGO_TARGET_DIR=str(secondary)))]:job.result()
assert before==source_hash(),'Sources changed during packaging'
built=root/'native/tfm2_custom_ai/target/release/tfm2_custom_ai.dll';reproduced=secondary/'release/tfm2_custom_ai.dll'
assert sha(built)==sha(reproduced),'Independent DLL builds differ'
package=root/'mods/tfm2_custom_ai/tfm2_custom_ai.dll';shutil.copyfile(built,package)
assert (root/'native/tfm2_custom_ai/mod.mod_info').read_bytes()==(root/'mods/tfm2_custom_ai/mod.mod_info').read_bytes()
report={'baseCommit':options.base,'sourceDateEpoch':epoch,'nativeSourcesSha256':before,'toolchain':run(['rustc','+stable-x86_64-pc-windows-gnu','--version']),'builtDllSha256':sha(built),'reproducedDllSha256':sha(reproduced),'packagedDllSha256':sha(package),'versions':{p.parent.name:json.loads(p.read_text(encoding='utf8'))['version']for p in sorted((root/'mods').glob('*/mod.mod_info'))},'installed':False,'careerMigrationApplied':False,'liveAcceptance':'Pending actual-game visual inspection; no installed game was changed.'}
(out/'package.json').write_text(json.dumps(report,indent=2)+'\n')
print('Two independent DLL builds match: '+sha(package),flush=True)
backup=root/'backups';backup.mkdir(exist_ok=True)
fixture=Path(tempfile.mkdtemp(prefix=options.fixture_prefix,dir=backup)).resolve();assert fixture.is_relative_to(backup.resolve())
(fixture/'bundle.game_data').touch();shutil.copytree(root/'mods',fixture/'mods')
compared=[]
for source in sorted((root/'mods').rglob('*')):
 if source.is_file():
  relative=source.relative_to(root/'mods');assert sha(source)==sha(fixture/'mods'/relative),str(relative);compared.append(relative.as_posix())
config=fixture/'config/game';config.mkdir(parents=True)
(config/'mods.json').write_text(json.dumps({'enabled_mods':['tfm2_custom','tfm2_custom_ai','tfm2_jjk','tfm2_frieren','tfm2_jojo','tfm2_cyberpunk','tfm2_ultrakill']}))
cfg=root/'manager.cfg';old=cfg.read_bytes()if cfg.exists()else None
try:
 p=subprocess.run(['rtk','proxy',str(root/'TFM2 Mod Manager.exe'),'--check','--yes','--game',str(fixture)],cwd=root,capture_output=True,text=True,encoding='utf8',errors='replace')
finally:
 if old is not None:cfg.write_bytes(old)
 elif cfg.exists():cfg.unlink()
output=p.stdout+p.stderr;(out/'mod-manager.txt').write_text(output,encoding='utf8')
result={'scope':'Isolated fixture. Supplementary SHA-256 comparison covers every copied pack. Installed game and careers unchanged.','allPackFilesCompared':len(compared),'allPackDifferences':0,'exitCode':p.returncode,'result':[s.strip()for s in output.splitlines()if any(x in s for x in ['files compared','tfm2_custom_ai','UP TO DATE:'])]}
(out/'mod-manager.json').write_text(json.dumps(result,indent=2)+'\n')
assert p.returncode==0,output
assert 'NOT UP TO DATE:'not in output,output
print(json.dumps(result,indent=2))
