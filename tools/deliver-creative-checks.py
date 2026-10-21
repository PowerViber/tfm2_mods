"""Sequential final verification, reproducible packaging, and isolated Mod Manager check."""
from pathlib import Path
import subprocess,json,shutil
root=Path(__file__).resolve().parents[1];out=root/'docs/creative-results';out.mkdir(exist_ok=True)
for n,tool in enumerate(['build-creative-labs.py','verify-creative-generation.py','check-creative-release.py','benchmark-creative.py','package-creative-release.py','check-steve-package-manager.py']):
 p=subprocess.run(['rtk','proxy','python','-X','utf8',str(root/'tools'/tool)],cwd=root,capture_output=True,text=True,encoding='utf8',errors='replace')
 (out/f'delivery-{n+1}.txt').write_text(p.stdout+p.stderr,encoding='utf8');print(json.dumps({'tool':tool,'exitCode':p.returncode}),flush=True)
 if p.returncode:raise SystemExit(p.stdout+p.stderr)
 if tool=='check-steve-package-manager.py':
  for suffix in ['json','txt']:shutil.copyfile(root/f'docs/steve-results/mod-manager.{suffix}',out/f'mod-manager.{suffix}')
