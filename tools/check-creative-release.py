"""Record actual results of this release's checks. No installs or real-career migration."""
from pathlib import Path
import subprocess,json,re
root=Path(__file__).resolve().parents[1];out=root/'docs/creative-results';out.mkdir(exist_ok=True)
commands=[
 ['cargo','+stable-x86_64-pc-windows-gnu','test','--release','--manifest-path','native/tfm2_custom_ai/Cargo.toml','--lib','--','--quiet'],
 ['cargo','+stable-x86_64-pc-windows-gnu','test','--release','--manifest-path','native/universal_lab/Cargo.toml','--lib'],
 ['python','-X','utf8','tools/test_rework_migration.py'],
 ['python','-X','utf8','tools/verify-creative-art.py'],
 ['node','tools/test-steve-lab.cjs'],
 ['node','tools/test-universal-lab.cjs'],
 ['node','tools/test-farming-catalogue.cjs'],
 ['node','tools/test-steve-v1-compat.cjs'],
]
rows=[]
for n,args in enumerate(commands):
 p=subprocess.run(['rtk','proxy',*args],cwd=root,capture_output=True,text=True,encoding='utf8',errors='replace');text=p.stdout+p.stderr
 (out/f'check-{n+1}.txt').write_text(text,encoding='utf8')
 rows.append(dict(command=args,exitCode=p.returncode,summaries=[s for s in text.splitlines()if 'test result:'in s or 'Ran 'in s or s=='OK']))
 (out/'checks.json').write_text(json.dumps({'checks':rows,'liveAcceptance':'pending','installed':False},indent=2)+'\n',encoding='utf8')
 print(json.dumps(rows[-1]),flush=True)
 if p.returncode:raise SystemExit(text)
