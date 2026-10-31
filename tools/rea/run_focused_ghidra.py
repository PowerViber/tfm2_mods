"""Focused headless fallback after recording REA whole-program startup limits."""
import argparse,hashlib,json,os,subprocess,tempfile,shutil
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--ghidra',type=Path,required=True)
    p.add_argument('--exe',type=Path,required=True);p.add_argument('--java-home',type=Path,required=True)
    p.add_argument('--reports',type=Path,default=ROOT/'docs/contender-realms/windows');a=p.parse_args()
    a.reports=a.reports.resolve();a.reports.mkdir(parents=True,exist_ok=True)
    before=hashlib.sha256(a.exe.read_bytes()).hexdigest()
    # A failed script must not leave an older successful report looking current.
    (a.reports/'ghidra-focused.json').unlink(missing_ok=True)
    # Ghidra project URLs reject any path component starting with a dot.
    workspace=Path(tempfile.mkdtemp(prefix='tfm2-realm-focused-'))
    target=workspace/'target.exe';shutil.copy2(a.exe,target)
    for name in ['home','tmp','config','data']:(workspace/name).mkdir()
    command=['rtk','proxy','cmd','/d','/c',str(a.ghidra.resolve()/'support/analyzeHeadless.bat'),str(workspace),
        'realm-focused','-import',str(target),'-noanalysis','-scriptPath',str(ROOT/'tools/rea'),
        '-postScript','RealmSeedAnalysis.java',str(a.reports/'realm-hooks.json'),str(a.reports/'ghidra-focused.json'),
        '-deleteProject','-max-cpu','2']
    output=a.reports/'ghidra-focused.log'
    with output.open('w',encoding='utf8')as log:
        env=dict(os.environ,JAVA_HOME=str(a.java_home.resolve()),APPDATA=str(workspace/'config'),
            LOCALAPPDATA=str(workspace/'data'),JAVA_TOOL_OPTIONS=f'-Duser.home={workspace / "home"} -Djava.io.tmpdir={workspace / "tmp"}')
        r=subprocess.run(command,cwd=ROOT,env=env,stdout=log,stderr=subprocess.STDOUT,text=True)
    after=hashlib.sha256(a.exe.read_bytes()).hexdigest()
    project_files=[str(p.relative_to(workspace))for p in workspace.glob('realm-focused*')]
    report={'exitCode':r.returncode,'exeSha256':after,'exeBeforeSha256':before,
        'scriptSha256':hashlib.sha256((ROOT/'tools/rea/RealmSeedAnalysis.java').read_bytes()).hexdigest(),
        'globalAutoAnalysis':False,'projectDeletedByHeadless':not project_files,
        'remainingProjectFiles':project_files,'executableChanged':before!=after}
    if(a.reports/'ghidra-focused.json').exists():
        report['outputSha256']=hashlib.sha256((a.reports/'ghidra-focused.json').read_bytes()).hexdigest()
    (a.reports/'ghidra-focused-run.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
    if r.returncode or before!=after or not(a.reports/'ghidra-focused.json').exists():raise SystemExit('Focused analysis failed; inspect its log')
if __name__=='__main__':main()
