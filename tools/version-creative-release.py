"""Advance only affected packs, preserving the separate objective/Stark PR's reserved versions."""
from pathlib import Path
import json
root=Path(__file__).resolve().parents[1]
versions={'tfm2_custom_ai':'0.10.63','tfm2_custom':'0.2.63','tfm2_blockcraft':'0.1.22','tfm2_frieren':'0.1.20','tfm2_jojo':'0.1.19','tfm2_valorant':'0.1.14'}
for pack,version in versions.items():
 p=root/f'mods/{pack}/mod.mod_info';d=json.loads(p.read_text(encoding='utf8'));d['version']=version;d['last_updated']='2026-10-21'
 for dep in d.get('dependencies',[]):
  if dep['mod_id']=='tfm2_custom_ai':dep['version']='>=0.10.63'
 if pack=='tfm2_custom_ai':d['description']+=' Universal team intentions and four persistent athlete game-sense ratings; role-aware farming observations. Creative Herobrine has 45 catalogue entries, HP Tank Steve scaling and paid redstone networks. Frieren/Coder/Dio balance reductions and rooted 75000-range Omen Shadow Step.'
 if pack=='tfm2_blockcraft':d['description']='HP Tank Steve and persistent Creative Herobrine: 45 choices, four recovering slots, paid redstone, straight/curved Boat Wall and approved original short-arm artwork.'
 text=json.dumps(d,indent=2)+'\n';p.write_text(text,encoding='utf8')
 if pack=='tfm2_custom_ai':(root/'native/tfm2_custom_ai/mod.mod_info').write_text(text,encoding='utf8')
p=root/'native/tfm2_custom_ai/src/lib.rs';s=p.read_text(encoding='utf8').replace('const VERSION: &str = "0.10.62"','const VERSION: &str = "0.10.63"');p.write_text(s,encoding='utf8')
print(versions)
