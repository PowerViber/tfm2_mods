"""Record per-file hashes and a deterministic directory-manifest digest, not a DLL rebuild."""
from pathlib import Path
import hashlib,json
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/vader-aegis-scribble-review'
report={'algorithm':'SHA256 of sorted UTF-8 relative-path + NUL + lowercase file SHA256 + LF records; hashes describe the exact working-tree package bytes','packs':{}}
for name in ['tfm2_custom','tfm2_starwars','tfm2_toon']:
 folder=ROOT/'mods'/name;files={p.relative_to(folder).as_posix():hashlib.sha256(p.read_bytes()).hexdigest()for p in sorted(folder.rglob('*'))if p.is_file()};assert not any('__pycache__'in p for p in files)
 payload=''.join(p+'\0'+v+'\n'for p,v in files.items()).encode();report['packs'][name]={'version':json.loads((folder/'mod.mod_info').read_text())['version'],'manifestSha256':hashlib.sha256(payload).hexdigest(),'files':files}
report['nativeDllSha256']=hashlib.sha256((ROOT/'mods/tfm2_custom_ai/tfm2_custom_ai.dll').read_bytes()).hexdigest();(OUT/'package-hashes.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps({k:{s:v for s,v in p.items()if s!='files'}for k,p in report['packs'].items()},indent=2))
