"""Verify canonical generation is deterministic, including animation and editor metadata."""
from pathlib import Path
import hashlib,subprocess,sys
ROOT=Path(__file__).resolve().parents[1]
files=[*sorted((ROOT/'mods/tfm2_custom/vfx').glob('magnus_*#*')),
 *sorted((ROOT/'mods/tfm2_custom/champions').glob('tfm2_custom_magnus#*')),
 ROOT/'mods/tfm2_custom/champion/tfm2_custom_magnus.data_champion',
 ROOT/'mods/tfm2_custom/text/champion.i18n',ROOT/'native/tfm2_custom_ai/src/magnus_data.rs',
 ROOT/'editor/magnus-art.json',ROOT/'editor/magnus-data.js',ROOT/'docs/magnus-art-preview.png']
def hashes():return {str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest() for p in files}
before=hashes()
subprocess.run([sys.executable,str(ROOT/'Claude outputs/magnus/generate.py')],cwd=ROOT,check=True)
after=hashes();assert before==after,[p for p in before if before[p]!=after[p]]
print('PASS:',len(files),'generated files reproduce byte for byte.')
