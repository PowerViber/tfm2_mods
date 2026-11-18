"""Capture this continuation separately from the historical 2.3 evidence."""
from pathlib import Path
import json, os, re, subprocess
ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/'docs/contender-24-art-review';OUT.mkdir(exist_ok=True)
BASE='1648b00b8d6c610b5be472784fc04edb30d79a64'
env=dict(os.environ,CONTENDER_RESULTS=str(OUT.relative_to(ROOT)),CONTENDER_BASE=BASE)
commands={
 'native':['cargo','+stable-x86_64-pc-windows-gnu','test','--release','--manifest-path','native/tfm2_custom_ai/Cargo.toml','--lib','--','--quiet'],
 'lab':['cargo','+stable-x86_64-pc-windows-gnu','test','--release','--manifest-path','native/contender_shadow_lab/Cargo.toml'],
 'manager':['cargo','+stable-x86_64-pc-windows-gnu','test','--release','--manifest-path','tools/manager/Cargo.toml'],
 'generator':['python','tools/generate-contender.py','--check'],
 'catalogue':['python','tools/generate-lab-targets.py','--check'],
 'art':['python','tools/verify-contender-shadow-art.py'],
 'merge_art':['python','tools/verify-contender-merge-art.py']}
results={}
for name,args in commands.items():
 p=subprocess.run(['rtk','proxy',*args],cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf8',errors='replace')
 text=p.stdout+p.stderr;(OUT/f'{name}-checks.txt').write_text(text,encoding='utf8')
 assert p.returncode==0,text
 results[name]=dict(exit_code=0,passed=[int(n) for n in re.findall(r'test result: ok\. (\d+) passed',text)])
 print(name,results[name],flush=True)
# The only shared-core source change is a disambiguated presentation cue.
core='native/tfm2_custom_ai/src/contender_shadow_core.rs'
old=subprocess.check_output(['rtk','proxy','git','show',BASE+':'+core],cwd=ROOT).decode().replace('\r\n','\n')
new=(ROOT/core).read_text()
needle='out.push(Effect::Fx { name: "ct_haymaker".into(), at: u.pos, ticks: 18, heading: None });'
assert old.replace(needle,needle.replace('ct_haymaker','ct_break'),1)==new
for f in ['rules/contender.json','native/tfm2_custom_ai/src/contender_shadow_data.rs']:
 assert subprocess.check_output(['rtk','proxy','git','show',BASE+':'+f],cwd=ROOT).replace(b'\r\n',b'\n')==(ROOT/f).read_bytes().replace(b'\r\n',b'\n')
results.update(rules='contender-2.4',baseline=BASE,combat_source_unchanged_except_visual_cue=True,installed=False,actual_game_acceptance='pending')
(OUT/'windows-checks.json').write_text(json.dumps(results,indent=2)+'\n')
