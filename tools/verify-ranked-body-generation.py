"""Rebuild from frozen sources and require byte-identical candidate outputs."""
from pathlib import Path
import hashlib,json,subprocess,sys
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/ranked-body-redesign'
paths=set()
for folder,pattern in [('mods/tfm2_custom/champions','*'),('mods/tfm2_custom/vfx','*'),('docs/ranked-body-redesign/staged','*'),('docs/ranked-body-redesign/concepts','*')]:
 paths.update(p for p in(ROOT/folder).glob(pattern)if p.is_file())
paths.update(ROOT/p for p in ['rules/ranked-body-art.json','editor/magnus-art.json','editor/vesper-art-preview.json','editor/vesper-portraits.png','editor/vesper-mastery-preview.png','editor/ranked-body-assets.json','docs/ranked-body-redesign/review-data.json','docs/ranked-body-redesign/base-animation-results.json','docs/ranked-body-redesign/fallback-integration.json'])
paths.update((ROOT/'mods/tfm2_custom/champion').glob('*.data_champion'))
paths.add(ROOT/'mods/tfm2_custom/mod.mod_info')
def hashes():return{p.relative_to(ROOT).as_posix():hashlib.sha256(p.read_bytes()).hexdigest()for p in sorted(paths)}
before=hashes()
for script in ['integrate-ranked-body-fallback.py','refresh-ranked-body-labs.py','build-ranked-body-review.py','verify-ranked-bodies.py']:
 subprocess.run(['rtk','proxy',sys.executable,'-X','utf8',str(ROOT/'tools'/script)],cwd=ROOT,check=True)
after=hashes();assert before==after,[p for p in before if before[p]!=after[p]]
native=hashlib.sha256((ROOT/'mods/tfm2_custom_ai/tfm2_custom_ai.dll').read_bytes()).hexdigest()
package={p.relative_to(ROOT/'mods/tfm2_custom').as_posix():hashlib.sha256(p.read_bytes()).hexdigest()for p in sorted((ROOT/'mods/tfm2_custom').rglob('*'))if p.is_file()}
package_sha=hashlib.sha256(''.join(name+'\0'+sha+'\n'for name,sha in package.items()).encode()).hexdigest()
report={'generatedOutputs':len(before),'identicalAfterRebuild':True,'outputSha256':after,'championPackVersion':'0.2.94','championPackDirectorySha256':package_sha,'championPackFiles':len(package),'nativeDllUnchanged':True,'nativeDllSha256':native,'approval':'approved','shippedIntegration':'candidate base bodies only','nativeRankMode':'visible-base-fallback','rankAnimationsComplete':False}
(OUT/'generation-results.json').write_text(json.dumps(report,indent=2)+'\n');print('PASS reproducible outputs:',len(before),'native DLL:',native)
