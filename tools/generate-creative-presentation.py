"""Refresh approved descriptions/presets. Does not redraw any existing skill effect."""
from pathlib import Path
import json,runpy
root=Path(__file__).resolve().parents[1]
updates={
 'tfm2_blockcraft':{'tfm2_blockcraft_steve':{
  'name':'Steve',
  'skill':'Build uses shared charges and caster maximum HP snapshotted on acceptance. Steve has three usable slots, +25% new construction health, Repair (one charge, 8s cooldown, 40000 range), and +20% Survival healing/shielding. Herobrine has four usable slots, no Survival bonuses, and 30 Creative choices: beacons, potions, gifted armour/enchantments, redstone, terrain tools, Milk, Totem, editing and tipped arrows. Helpful Creative protection excludes its original caster. Four slots recover independently in 5s; changing forms never refills them. HP-derived damage against Morgard/Serpen is capped at 200 per hit.',
  'skill2':'Utility retains rod, pearl, Apple, Shield Guard, crossbow, axe and boat/ice. Herobrine cannot use Apple, Shield Guard, Repair or rod ally rescue. Confirmed champion utility hits enable one target-bound melee follow-up. Steve basic: 100% AD + 1% max HP; Herobrine basic: 100% AD. Repair restores construction health only, never lifetime or ammunition.',
  'ult':'Change Form: 0.4s preparation, 18s cooldown from acceptance, requires ultimate learned. Herobrine persists until changed or death: empty hands, white eyes and -20% armour/MR. Switching retains HP, recovering charges and objects; interruption retains the previous form and spent cooldown. Boat Wall is a separate Build signature: three charges, 80s cooldown. Steve builds straight walls; Herobrine can build curves. Redstone networks are owner scoped, at most four nodes, two outputs/node, 32000 connection range, no cycles; loaded payloads are paid once. Death removes owned constructions without explosions.'}},
 'tfm2_frieren':{'tfm2_frieren_frieren':{'skill':'All Frieren-owned raw damage, including basic Zoltraak, Spellcraft, Fern and Stark, is multiplied by 80% once before mitigation. ', 'ult':'Stark retains his existing targetable companion. All party raw damage is multiplied by 80% once. '}},
 'tfm2_custom':{'tfm2_custom_coder':{'skill2':'Coder-created shields are multiplied by 75% once after support bonuses. Base HP 1000; HP growth 108. '}},
 'tfm2_jojo':{'tfm2_jojo_dio':{'skill':'Every knife deals 75% of its former raw damage, rounded down before mitigation; farming follows the same nerf. '}},
 'tfm2_valorant':{'tfm2_valorant_omen':{'skill2':'Shadow Step range: 75000. Omen cannot move during Shadow Step or ultimate channeling. Accepted destinations stay fixed; enemy interruptions still cancel. '}}
}
for pack,champions in updates.items():
 p=root/f'mods/{pack}/text/champion.i18n';data=json.loads(p.read_text(encoding='utf8'));descs=data.setdefault('en',{}).setdefault('description',{})
 for id,fields in champions.items():
  for key,value in fields.items():
   if pack=='tfm2_blockcraft':descs[id][key]=value
   elif not descs[id].get(key,'').startswith(value):descs[id][key]=value+descs[id].get(key,'')
 p.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
p=root/'editor/steve.html';s=p.read_text(encoding='utf8').replace('Fifteen recipes.','Forty-five recipes.').replace('transport v1','transport v2');p.write_text(s,encoding='utf8')
# Presets are generated from current champion definitions, never from retired kits.
runpy.run_path(str(root/'tools/generate-tactics-presets.py'),run_name='__main__')
