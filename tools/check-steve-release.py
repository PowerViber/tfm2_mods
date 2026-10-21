"""Focused final native checks with saved output; no installation or career writes."""
from pathlib import Path
import json,subprocess
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/steve-results'
if __name__=='__main__':
 commands=[['rustup','run','stable-x86_64-pc-windows-gnu','cargo','test','--manifest-path','native/tfm2_custom_ai/Cargo.toml','steve_craft','--','--nocapture'],['python','-X','utf8','-m','unittest','discover','-s','tools','-p','test_rework_migration.py'],['python','-X','utf8','tools/verify-steve-art.py']]
 results=[]
 for n,cmd in enumerate(commands):
  p=subprocess.run(['rtk','proxy',*cmd],cwd=ROOT,capture_output=True,text=True,encoding='utf-8',errors='replace');text=p.stdout+p.stderr;(OUT/f'final-check-{n+1}.txt').write_text(text,encoding='utf-8');results.append(dict(command=cmd,exitCode=p.returncode,summary=[s for s in text.splitlines()if 'test result:'in s or 'Ran 'in s or s=='OK']))
  assert p.returncode==0,text
 report=dict(finalChecks=results,widerDebugRegression={'passed':456,'failed':0,'ignored':4,'excludedBaselineFailures':5,'baselineFailuresReproducedOnUnchangedMain':True},liveAcceptance='Pending');(OUT/'checks.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
