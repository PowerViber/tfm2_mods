"""Reproduce and package Gojo's release DLL twice; never install or modify careers."""
from pathlib import Path
import hashlib,json,os,shutil,subprocess
ROOT=Path(__file__).resolve().parents[1]
BASE='a4098f15be65edc683fd62f8037d71b1e79ae9bd'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def call(args,env=None):
 p=subprocess.run(['rtk',*args],cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf-8',errors='replace')
 if p.returncode:raise RuntimeError(p.stdout+p.stderr)
 return p.stdout.strip()
if __name__=='__main__':
 epoch=call(['proxy','git','show','-s','--format=%ct',BASE]);env=dict(os.environ,SOURCE_DATE_EPOCH=epoch)
 args=['proxy','rustup','run','stable-x86_64-pc-windows-gnu','cargo','build','--release','--manifest-path','native/tfm2_custom_ai/Cargo.toml'];call(args,env)
 built=ROOT/'native/tfm2_custom_ai/target/release/tfm2_custom_ai.dll';secondary=ROOT/'backups/gojo-repro-target';call(args,dict(env,CARGO_TARGET_DIR=str(secondary)))
 reproduced=secondary/'release/tfm2_custom_ai.dll';assert sha(built)==sha(reproduced),'Independent DLL builds differ'
 package=ROOT/'mods/tfm2_custom_ai/tfm2_custom_ai.dll';shutil.copyfile(built,package);assert sha(package)==sha(built)
 meta=json.loads((ROOT/'native/tfm2_custom_ai/mod.mod_info').read_text());assert meta==json.loads((ROOT/'mods/tfm2_custom_ai/mod.mod_info').read_text())
 report={'native':meta['version'],'jjk':json.loads((ROOT/'mods/tfm2_jjk/mod.mod_info').read_text())['version'],'championPack':json.loads((ROOT/'mods/tfm2_custom/mod.mod_info').read_text())['version'],'baseCommit':BASE,'sourceDateEpoch':epoch,'toolchain':call(['proxy','rustup','run','stable-x86_64-pc-windows-gnu','rustc','--version']),'builtDllSha256':sha(built),'reproducedDllSha256':sha(reproduced),'packagedDllSha256':sha(package),'gojoWasmSha256':sha(ROOT/'editor/gojo.wasm'),'installedByThisScript':False,'liveAcceptance':'Pending: user chose to finish the PR without live game checks.','separateObjectiveStarkWorkIncluded':False}
 out=ROOT/'docs/gojo-results/package.json';out.parent.mkdir(parents=True,exist_ok=True);out.write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
