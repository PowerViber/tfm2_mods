"""Reproducible diagnostic package only. Never installs or enables a mod."""
import argparse,hashlib,json,os,shutil,subprocess
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2];CRATE=ROOT/'experiments/contender-realm-observer';REPORT=ROOT/'docs/contender-realms/windows'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--repeat',action='store_true');a=p.parse_args();REPORT.mkdir(parents=True,exist_ok=True)
    epoch=subprocess.check_output(['rtk','proxy','git','show','-s','--format=%ct','5c7a9fc'],cwd=ROOT,text=True).strip()
    sources=[CRATE/'Cargo.toml',CRATE/'Cargo.lock',*sorted((CRATE/'src').glob('*.rs')),*sorted((ROOT/'native/mod-api-stable/src').glob('*.rs'))]
    before={p.relative_to(ROOT).as_posix():sha(p)for p in sources};hashes=[]
    for run in range(2 if a.repeat else 1):
        target=CRATE/'target'/('observer-repro-'+str(run));cmd=['rtk','proxy','cargo','+stable-x86_64-pc-windows-gnu','build','--release','--locked','--manifest-path',str(CRATE/'Cargo.toml'),'--target-dir',str(target),'--features','observe-runtime']
        r=subprocess.run(cmd,cwd=ROOT,env=dict(os.environ,SOURCE_DATE_EPOCH=epoch),capture_output=True,text=True,encoding='utf8',errors='replace')
        (REPORT/f'observer-build-{run}.txt').write_text(r.stdout+r.stderr,encoding='utf8')
        if r.returncode:raise RuntimeError(r.stdout+r.stderr)
        dll=target/'release/tfm2_realm_observer.dll';hashes.append(sha(dll))
        if run==0:
            package=CRATE/'target/packages/tfm2_realm_observer';package.mkdir(parents=True,exist_ok=True);shutil.copy2(dll,package/dll.name)
            (package/'mod.mod_info').write_text(json.dumps({'name':'Realm observer - disposable only','author':'TFM2 realm investigation','version':'0.1.0','mod_type':'native','contains_code':True,'description':'Observation only. No realm hooks or champion changes. Requires realm_probe.flag and the exact verified executable.','dependencies':[{'mod_id':'base','version':'>=0.7.0'}]},indent=2)+'\n')
    assert len(set(hashes))==1,'Independent builds differ'
    assert before=={p.relative_to(ROOT).as_posix():sha(p)for p in sources},'Sources changed during builds'
    report={'version':1,'productionRelease':False,'sourceDateEpoch':epoch,'independentBuilds':len(hashes),'identical':True,'dllSha256':hashes[0],'sources':before,'installationPerformed':False}
    (REPORT/'observer-builds.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({k:v for k,v in report.items()if k!='sources'},indent=2))
if __name__=='__main__':main()
