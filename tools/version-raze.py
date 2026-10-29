"""Version the isolated Raze release and synchronize the native dependency."""
from pathlib import Path
import json,re
root=Path(__file__).resolve().parents[1]
versions={'tfm2_custom_ai':'0.10.75','tfm2_custom':'0.2.75','tfm2_valorant':'0.1.24'}
for pack,version in versions.items():
 p=root/f'mods/{pack}/mod.mod_info';d=json.loads(p.read_text(encoding='utf-8'));assert tuple(map(int,d['version'].split('.')))<=tuple(map(int,version.split('.'))),'Refusing to downgrade '+pack;d['version']=version;d['last_updated']='2026-10-29'
 for dep in d.get('dependencies',[]):
  if dep['mod_id']=='tfm2_custom_ai':dep['version']='>=0.10.75'
 if pack=='tfm2_valorant':d['description']='Omen and Raze: shared credit income, purchased weapons and free-plus-bought utility. Raze starts with Bucky, protects weapon savings, deals triple Attack-scaling physical skill damage and plans supported entries or exits. All thirteen athlete attributes affect decisions and execution, without mastery.'
 if pack=='tfm2_custom_ai' and 'Raze 0.10.75:' not in d['description']:d['description']+=' Raze 0.10.75: Attack-scaling physical skills, half-strength air strafing, positional dome launches and champion-count-based 3-12-second landing defence.'
 if pack=='tfm2_custom_ai' and 'Raze uses' not in d['description']:d['description']+=' Raze uses purchased Omen weapons, paid utility, deterministic air strafing and all thirteen pinned athlete attributes; the same Rust core powers Raze Lab v2.'
 d["description"]=d["description"].replace("Raze Lab v1","Raze Lab v3").replace("Raze Lab v2","Raze Lab v3");text=json.dumps(d,ensure_ascii=False,indent=2)+'\n';p.write_text(text,encoding='utf-8')
 if pack=='tfm2_custom_ai':(root/'native/tfm2_custom_ai/mod.mod_info').write_text(text,encoding='utf-8')
p=root/'native/tfm2_custom_ai/src/lib.rs';s=p.read_text(encoding='utf-8');s=re.sub(r'const VERSION: &str = "[0-9.]+";', 'const VERSION: &str = "0.10.75";',s);p.write_text(s,encoding='utf-8')
print(json.dumps(versions))
