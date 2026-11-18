"""Capture regression evidence; no install, career changes or DLL packaging."""
from pathlib import Path
import hashlib,json,re,subprocess,os
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/os.environ.get('CONTENDER_RESULTS','docs/contender-1vx-review');OUT.mkdir(exist_ok=True)
commands={
 'native':['cargo','+stable-x86_64-pc-windows-gnu','test','--release','--manifest-path','native/tfm2_custom_ai/Cargo.toml','--lib','--','--quiet'],
 'lab':['cargo','+stable-x86_64-pc-windows-gnu','test','--release','--manifest-path','native/contender_shadow_lab/Cargo.toml'],
 'manager':['cargo','+stable-x86_64-pc-windows-gnu','test','--release','--manifest-path','tools/manager/Cargo.toml'],
 'ledger':['node','tools/test-lab-damage.cjs'],
 'catalogue':['python','tools/generate-lab-targets.py','--check'],
 'generator':['python','tools/generate-contender.py','--check']}
results={}
for name,args in commands.items():
    r=subprocess.run(['rtk','proxy',*args],cwd=ROOT,capture_output=True,text=True,encoding='utf8',errors='replace');text=r.stdout+r.stderr
    (OUT/f'{name}-tests.txt').write_text(text,encoding='utf8');assert r.returncode==0,text
    results[name]={'exit_code':r.returncode,'passed':[int(v) for v in re.findall(r'test result: ok\. (\d+) passed',text)],'unused_warnings':len(re.findall(r'^warning: (?:constant|function)',text,re.M))}
    print(name,results[name])
r=subprocess.run(['rtk','proxy','git','diff','--exit-code','4c111ab18e7d04053da91e06ee512d36d586d726','--','native/tfm2_custom_ai/src','native/contender_shadow_lab/src','rules/contender.json','mods','editor/contender.wasm'],cwd=ROOT,capture_output=True,text=True);native_unchanged=r.returncode==0
results.update(rules=json.loads((ROOT/'rules/contender.json').read_text())['revision'],native_gameplay_and_packaged_assets_unchanged=native_unchanged,
    wasm_sha256=hashlib.sha256((ROOT/'editor/contender.wasm').read_bytes()).hexdigest(),packaged_dll_sha256=hashlib.sha256((ROOT/'mods/tfm2_custom_ai/tfm2_custom_ai.dll').read_bytes()).hexdigest(),
    installed=False,career_migration=False,actual_game_acceptance='separate; this is a Lab host feature')
(OUT/'regressions.json').write_text(json.dumps(results,indent=2)+'\n')
