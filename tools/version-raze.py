"""Version the isolated Raze release and synchronize the native dependency."""
from pathlib import Path
import json
root=Path(__file__).resolve().parents[1]
versions={'tfm2_custom_ai':'0.10.69','tfm2_custom':'0.2.69','tfm2_valorant':'0.1.18'}
for pack,version in versions.items():
 p=root/f'mods/{pack}/mod.mod_info';d=json.loads(p.read_text(encoding='utf-8'));d['version']=version;d['last_updated']='2026-10-25'
 for dep in d.get('dependencies',[]):
  if dep['mod_id']=='tfm2_custom_ai':dep['version']='>=0.10.69'
 if pack=='tfm2_valorant':d['description']='Omen and Raze: shared credits, purchased guns and armour; shadow control and AP explosives. Raze uses two paid satchels, momentum-based air strafing and all 13 pinned athlete attributes, without mastery.'
 if pack=='tfm2_custom_ai' and 'Raze uses' not in d['description']:d['description']+=' Raze uses purchased Omen weapons, paid utility, deterministic air strafing and all thirteen pinned athlete attributes; the same Rust core powers Raze Lab v2.'
 d["description"]=d["description"].replace("Raze Lab v1","Raze Lab v2");text=json.dumps(d,ensure_ascii=False,indent=2)+'\n';p.write_text(text,encoding='utf-8')
 if pack=='tfm2_custom_ai':(root/'native/tfm2_custom_ai/mod.mod_info').write_text(text,encoding='utf-8')
p=root/'native/tfm2_custom_ai/src/lib.rs';s=p.read_text(encoding='utf-8');s=s.replace('const VERSION: &str = "0.10.68"','const VERSION: &str = "0.10.69"');s=s.replace('Vesper, Magnus + map plans','Vesper, Magnus, Raze + map plans');p.write_text(s,encoding='utf-8')
print(json.dumps(versions))
