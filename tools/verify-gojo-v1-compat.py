"""Compare frozen v1 combat sources and recording runtime with the published PR head."""
from pathlib import Path
import hashlib,json,subprocess
ROOT=Path(__file__).resolve().parents[1];BASE='290038b28395f01c92556a4a79a7b44138004a16'
def blob(path):return subprocess.check_output(['rtk','proxy','git','show',BASE+':'+path],cwd=ROOT)
def text(b):return b.replace(b'\r\n',b'\n')
paths=['native/tfm2_custom_ai/src/gojo_core.rs','native/tfm2_custom_ai/src/gojo_data.rs','native/tfm2_custom_ai/src/gojo_protocol.rs','rules/gojo.json']
checks={p:text(blob(p))==text((ROOT/p).read_bytes()) for p in paths}
checks['v1AdapterOnlyExposesGeometryHelper']=text(blob('native/tfm2_custom_ai/src/gojo.rs')).replace(b'fn domain_zones(',b'pub(super) fn domain_zones(')==text((ROOT/'native/tfm2_custom_ai/src/gojo.rs').read_bytes())
old=blob('editor/gojo.wasm');checks['frozenWasmBytes']=old==(ROOT/'editor/gojo-v1.wasm').read_bytes()
checks['frozenReader']=text(blob('editor/gojo-lab.js')).replace(b'/gojo.wasm',b'/gojo-v1.wasm')==text((ROOT/'editor/gojo-v1-lab.js').read_bytes())
assert all(checks.values()),checks
report={'baseline':BASE,'checks':checks,'v1WasmSha256':hashlib.sha256(old).hexdigest(),'textComparison':'Only line endings normalized; WASM bytes compared exactly.'}
(ROOT/'docs/gojo-results/v1-compatibility.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8');print(json.dumps(report))
