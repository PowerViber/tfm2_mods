"""Build twice and package only. Never installs or edits careers."""
from pathlib import Path
import hashlib,json,os,shutil,subprocess,statistics
from concurrent.futures import ThreadPoolExecutor
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/steve-results';BASE='0161dfa'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def call(args,env=None):
 p=subprocess.run(['rtk',*args],cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf-8',errors='replace')
 if p.returncode:raise RuntimeError(p.stdout+p.stderr)
 return p.stdout.strip()
if __name__=='__main__':
 epoch=call(['proxy','git','show','-s','--format=%ct',BASE]);env=dict(os.environ,SOURCE_DATE_EPOCH=epoch)
 source_files=sorted((ROOT/'native/tfm2_custom_ai/src').glob('*.rs'))
 fingerprint=lambda:hashlib.sha256(b''.join(p.name.encode()+p.read_bytes()for p in source_files)).hexdigest()
 source_hash=fingerprint()
 args=['proxy','rustup','run','stable-x86_64-pc-windows-gnu','cargo','build','--release','--manifest-path','native/tfm2_custom_ai/Cargo.toml'];built=ROOT/'native/tfm2_custom_ai/target/release/tfm2_custom_ai.dll';secondary=ROOT/'backups/steve-repro-target'
 with ThreadPoolExecutor(max_workers=2)as pool:
  jobs=[pool.submit(call,args,env),pool.submit(call,args,dict(env,CARGO_TARGET_DIR=str(secondary)))]
  for job in jobs:job.result()
 assert source_hash==fingerprint(),'Native sources changed during packaging'
 reproduced=secondary/'release/tfm2_custom_ai.dll';assert sha(built)==sha(reproduced),'Reproduced DLL differs';package=ROOT/'mods/tfm2_custom_ai/tfm2_custom_ai.dll';shutil.copyfile(built,package)
 native=json.loads((ROOT/'native/tfm2_custom_ai/mod.mod_info').read_text());assert native==json.loads((ROOT/'mods/tfm2_custom_ai/mod.mod_info').read_text())
 report=dict(native=native['version'],blockcraft=json.loads((ROOT/'mods/tfm2_blockcraft/mod.mod_info').read_text())['version'],baseCommit=BASE,sourceDateEpoch=epoch,nativeSourceSha256=source_hash,toolchain=call(['proxy','rustup','run','stable-x86_64-pc-windows-gnu','rustc','--version']),builtDllSha256=sha(built),reproducedDllSha256=sha(reproduced),packagedDllSha256=sha(package),steveWasmSha256=sha(ROOT/'editor/steve.wasm'),installedByThisScript=False,liveAcceptance='Pending real-match carrier, item, targeting, warnings and crowded host checks.')
 (OUT/'package.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
 bench=json.loads(call(['proxy','rustup','run','stable-x86_64-pc-windows-gnu','cargo','run','--release','--bin','steve-bench','--manifest-path','native/steve_lab/Cargo.toml']))
 for crowd in bench['crowds']:
  crowd['medianP95Ns']=statistics.median(r['p95Ns']for r in crowd['runs']);crowd['medianP99Ns']=statistics.median(r['p99Ns']for r in crowd['runs'])
 (OUT/'performance.json').write_text(json.dumps(bench,indent=2)+'\n')
