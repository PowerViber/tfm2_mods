"""Repack frozen sources and compare every generated atlas, clock and manifest."""
from pathlib import Path
import hashlib,json,subprocess
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/kratos-art-review'
paths=[*OUT.glob('*-preview.png'),*OUT.glob('*-preview.fanim'),*OUT.glob('animation-*.gif'),OUT/'art-manifest.json',OUT/'presentation.json',
       ROOT/'native/tfm2_custom_ai/src/kratos_art_data.rs',ROOT/'mods/tfm2_custom/champion/tfm2_custom_kratos.data_champion']
def hashes():return {p.relative_to(ROOT).as_posix():hashlib.sha256(p.read_bytes()).hexdigest() for p in paths}
before=hashes()
for script in ['tools/build-kratos-art.py','tools/generate-kratos.py']:
    subprocess.run(['rtk','proxy','python',script],cwd=ROOT,check=True,capture_output=True)
after=hashes();assert before==after,[p for p in before if before[p]!=after[p]]
report={'files_identical':len(before),'sha256':after,'scope':'Deterministic extraction, mirroring, packing, clocks and bindings from frozen sources. AI generation is not deterministic.'}
(OUT/'reproducibility.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps({'files_identical':len(before)}))
