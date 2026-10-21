"""Record final automated checks. Start the editor on port 7281 before browser checks."""
from pathlib import Path
import json,subprocess
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/gojo-results';OUT.mkdir(parents=True,exist_ok=True)
def run(name,args):
 p=subprocess.run(['rtk',*args],cwd=ROOT,capture_output=True,text=True,encoding='utf-8',errors='replace');(OUT/(name+'.txt')).write_text((p.stdout+p.stderr).rstrip()+'\n',encoding='utf-8');assert p.returncode==0,p.stdout+p.stderr;return {'name':name,'passed':True}
if __name__=='__main__':
 checks=[];prefix=['proxy','rustup','run','stable-x86_64-pc-windows-gnu','cargo']
 for name,args in [('native-tests',prefix+['test','--release','--manifest-path','native/tfm2_custom_ai/Cargo.toml','--lib','--quiet']),('shared-tests',prefix+['test','--release','--manifest-path','native/gojo_lab/Cargo.toml','--quiet']),('migration',['proxy','python','tools/test_rework_migration.py']),('art-check',['proxy','python','tools/verify-gojo-art.py']),('browser-check',['proxy','node','tools/test-gojo-lab.cjs'])]:
  checks.append(run(name,args));print(name+' passed',flush=True)
 p=subprocess.run(['rtk',*prefix,'run','--release','--quiet','--manifest-path','native/gojo_lab/Cargo.toml','--bin','benchmark'],cwd=ROOT,capture_output=True,text=True,encoding='utf-8');assert p.returncode==0,p.stderr;rows=json.loads(p.stdout);report={'scope':'Release core with 128 immutable entities. Host calls/rendering excluded. Cold is a fresh core first call, not cold assets. Fixed snapshots are a timing stress case, not a balance match.','runs':rows,'medianP95Ns':sorted(r['p95Ns'] for r in rows)[2],'medianP99Ns':sorted(r['p99Ns'] for r in rows)[2]};(OUT/'performance.json').write_text(json.dumps(report,indent=2)+'\n')
 (OUT/'verification.json').write_text(json.dumps({'checks':checks,'scope':'Automated only; real engine attribution, items and matched balance acceptance pending.'},indent=2)+'\n');print(json.dumps(report))
