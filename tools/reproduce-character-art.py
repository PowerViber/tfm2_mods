"""Re-run deterministic extraction/packing and hash all generated runtime bindings."""
from pathlib import Path
import subprocess,sys,hashlib,json
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/vader-aegis-scribble-review'
files=[*ROOT.glob('mods/tfm2_starwars/champions/tfm2_starwars_vader#*'),*ROOT.glob('mods/tfm2_toon/champions/tfm2_toon_scribble#*'),*ROOT.glob('mods/tfm2_custom/champions/tfm2_gundam_aegis_zero#*'),*ROOT.glob('mods/tfm2_custom/vfx/gundam#*'),ROOT/'editor/sprites-data.js',ROOT/'editor/vfx2.js']
def hashes():return {p.relative_to(ROOT).as_posix():hashlib.sha256(p.read_bytes()).hexdigest()for p in files}
before=hashes();subprocess.run(['rtk','proxy',sys.executable,'tools/integrate-character-redesign.py'],cwd=ROOT,check=True);after=hashes();assert before==after
report={'baseline':json.loads((OUT/'integration.json').read_text())['base'],'identicalSecondGeneration':True,'scope':'Frozen-source extraction, frame selection, atlas packing and editor bindings; imagegen itself is not deterministic','sha256':after}
(OUT/'reproducibility.json').write_text(json.dumps(report,indent=2)+'\n');print('PASS:',len(files),'runtime art/binding files reproduce byte-for-byte')
