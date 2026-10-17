"""Build twice, compare release DLL bytes, then package without installing into the game."""
from pathlib import Path
import hashlib,json,os,shutil,subprocess
ROOT=Path(__file__).resolve().parents[1]
def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def call(args,env=None):
 p=subprocess.run(['rtk',*args],cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf-8',errors='replace')
 if p.returncode:raise RuntimeError(p.stdout+p.stderr)
 return p.stdout.strip()
if __name__=='__main__':
 epoch=call(['git','show','-s','--format=%ct','a4098f15be65edc683fd62f8037d71b1e79ae9bd'])
 env=dict(os.environ,SOURCE_DATE_EPOCH=epoch)
 args=['proxy','rustup','run','stable-x86_64-pc-windows-gnu','cargo','build','--release','--manifest-path','native/tfm2_custom_ai/Cargo.toml']
 call(args,env)
 built=ROOT/'native/tfm2_custom_ai/target/release/tfm2_custom_ai.dll'
 secondary=ROOT/'backups/rework-repro-target';env2=dict(env,CARGO_TARGET_DIR=str(secondary));call(args,env2)
 reproduced=secondary/'release/tfm2_custom_ai.dll'
 if sha(built)!=sha(reproduced):raise RuntimeError('Independent release builds differ; package not changed')
 package=ROOT/'mods/tfm2_custom_ai/tfm2_custom_ai.dll';backup=ROOT/'backups/rework-original-package.dll';backup.parent.mkdir(exist_ok=True)
 if not backup.exists():shutil.copyfile(package,backup)
 shutil.copyfile(built,package);assert sha(package)==sha(built)
 meta=json.loads((ROOT/'native/tfm2_custom_ai/mod.mod_info').read_text());assert meta==json.loads((ROOT/'mods/tfm2_custom_ai/mod.mod_info').read_text())
 report={'native':meta['version'],'championPack':json.loads((ROOT/'mods/tfm2_custom/mod.mod_info').read_text())['version'],'frieren':json.loads((ROOT/'mods/tfm2_frieren/mod.mod_info').read_text())['version'],'baseCommit':'a4098f15be65edc683fd62f8037d71b1e79ae9bd','sourceDateEpoch':epoch,'toolchain':call(['proxy','rustup','run','stable-x86_64-pc-windows-gnu','rustc','--version']),'builtDllSha256':sha(built),'reproducedDllSha256':sha(reproduced),'packagedDllSha256':sha(package),'frierenWasmSha256':sha(ROOT/'editor/frieren.wasm'),'magnusWasmSha256':sha(ROOT/'editor/magnus.wasm'),'installedByThisScript':False,'liveAcceptance':'Pending; live checks deferred by user.'}
 out=ROOT/'docs/objective-stark-results/package.json';out.parent.mkdir(parents=True,exist_ok=True);out.write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
