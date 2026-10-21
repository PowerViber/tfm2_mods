"""Assert current canonical generators do not change checked-in combat/art definitions."""
from pathlib import Path
import hashlib,subprocess,json
root=Path(__file__).resolve().parents[1]
files=list((root/'mods').glob('*/champion/*.data_champion'))+list((root/'mods').glob('*/text/*.i18n'))
files += [root/p for p in ['native/tfm2_custom_ai/src/balance_data.rs','native/tfm2_custom_ai/src/rework_data.rs','native/tfm2_custom_ai/src/steve_craft_data.rs','native/tfm2_custom_ai/src/farming_data.rs','native/tfm2_custom_ai/src/universal_data.rs','editor/steve-preset.js','editor/native-tactics-presets.js','editor/universal-data.js','editor/farming-data.js']]
def digest():return {str(p.relative_to(root)):hashlib.sha256(p.read_bytes()).hexdigest()for p in files}
before=digest()
for tool in ['generate-balance-creative.py','generate-rework-data.py','generate-steve-rules.py','generate-universal-rules.py','generate-steve-art.py']:
 subprocess.run(['rtk','proxy','python','-X','utf8',str(root/'tools'/tool)],cwd=root,check=True,capture_output=True)
subprocess.run(['rtk','proxy','node',str(root/'tools/generate-farming-capabilities.cjs')],cwd=root,check=True,capture_output=True)
after=digest();changed=[p for p in before if before[p]!=after[p]]
assert not changed,changed
out=root/'docs/creative-results/generation.json';out.write_text(json.dumps({'reproducible':True,'definitionsChecked':len(files),'legacyEffectsRedrawn':False},indent=2)+'\n',encoding='utf8');print(out.read_text())
