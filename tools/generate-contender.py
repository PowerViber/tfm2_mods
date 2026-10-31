"""Generate the playtest champion and editor preset from the canonical boxing rules."""
from pathlib import Path
import json, copy
ROOT=Path(__file__).resolve().parents[1]
rules=json.loads((ROOT/'rules/contender.json').read_text())
champ=copy.deepcopy(json.loads((ROOT/'mods/tfm2_custom/champion/tfm2_custom_magnus.data_champion').read_text()))
id=rules['id'];champ.update(id=id,category=rules['category'],tags=rules['tags'],sprite='asset/tfm2_custom/champions/'+id,anim_prefix='')
for key in ['stat','growth']:
    champ[key]={k:0 for k in champ[key]};champ[key].update(rules[key])
champ['attack'].update(action_name='attack',duration=18,cooltime=rules['basic']['cooldownTicks'],start_timing=9,range=rules['basic']['range'],effect={'type':'Native','effect_ref':'tfm2_custom_ai:contender_basic'})
for slot in ['skill','skill2','ult']:champ[slot]['description']='#asset/base/text/champion?description.'+id+'.'+slot
champ['passive']={'passive_ref':'tfm2_custom_ai:contender','params':{}}
champ['passive_ult']={'passive_ref':'tfm2_custom_ai:contender_ultimate','params':{}}
champ['view_projectiles']=[];champ['view_effects']=[];champ['view_buffs']=[]
for tag in ['ring','impact','counter','guard_arc','return_red','return_blue']+['warning'+str(i) for i in range(8)]:
    champ['view_effects'].append(dict(type='Animation',name='ct_'+tag,anim='asset/tfm2_custom/vfx/contender',tag=tag,z=-2 if tag=='ring' else 5,is_follow=False))
for i in range(11):champ['view_buffs'].append(dict(type='Animated',name='ct_stamina'+str(i),anim='asset/tfm2_custom/vfx/contender',tag='stamina'+str(i),z=6))
text={'name':'Contender','skill':'Combination: dashes up to 14000 at the target, then two warned physical punches, 20 + 55% Attack then 30 + 75% Attack; 22 stamina, 4 s shared cooldown. Body Shot: lunges up to 10000, then 60 + 100% Attack, -15% attack speed for 1.5 s; 28 stamina on the same cooldown. Uppercut: dashes up to 12000, then 55 + 85% Attack and 0.4 s airborne, 26 stamina, 8 s cooldown. Conditioning: 100 stamina, recovering 6/s or 12/s after 3 s without champion damage. Mod Power: +30% Attack, +5% HP, +10% armour/MR and attack speed. No mastery.',
'skill2':'Slip: travel up to 16000 over 0.2 s, avoiding targeting, damage and new control during the window; existing debuffs continue. Costs 20 stamina. Guard: 100 + 35% Attack shielding for 1.25 s, 40% damage reduction and half movement for 0.45 s; costs 24 stamina. Shared 6 s cooldown. The first confirmed champion impact during Guard readies a 2 s attacker-bound Counterpunch: next basic adds 25 + 35% Attack in its primary packet.',
'ult':'PLAYTEST: Main Event spends 40 stamina and starts a 75 s cooldown. After a 0.65 s warning, transfer Contender and one enemy within 65000 into a separate 160000-square off-map ring for 8 s; return survivors to their exact pre-transfer positions. Ordinary cooldowns recover 4/3 as fast inside. No healing or reset. Native off-map navigation, global-effect isolation, camera/replay, summons and full Slip avoidance still require actual-game acceptance; this is not a certified isolated realm.'}
(ROOT/'mods/tfm2_custom/champion'/f'{id}.data_champion').write_text(json.dumps(champ,indent=2)+'\n')
p=ROOT/'mods/tfm2_custom/text/champion.i18n';loc=json.loads(p.read_text(encoding='utf8'));loc['en']['description'][id]=text;p.write_text(json.dumps(loc,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
(ROOT/'editor/boxing-preset.js').write_text('window.BOXING_PRESET='+json.dumps(champ,separators=(',',':'))+';\nwindow.BOXING_TEXT='+json.dumps(text,separators=(',',':'))+';\n')
p=ROOT/'rules/champions.json';r=json.loads(p.read_text(encoding='utf8'));r['champions']=[c for c in r['champions'] if c['id']!=id];r['champions'].append(dict(id=id,name='Contender (playtest)',family='original',role='Melee',native=True,basicAttack=dict(name='Jab / Cross',kind='melee',range=23000),skills=[dict(slot='skill',name='Combination / Body Shot / Uppercut',kinds=['Aoe','Cc'],intents=['Clear','Poke','Objective'],summary=text['skill']),dict(slot='skill2',name='Slip / Guard',kinds=['Dash','Shield'],intents=['Escape','Protect'],summary=text['skill2']),dict(slot='ult',name='Main Event (unverified host)',kinds=['Cc'],intents=['Engage','Protect'],summary=text['ult'])]));p.write_text(json.dumps(r,ensure_ascii=False,indent=1)+'\n',encoding='utf8')
print('Contender data, localisation and editor preset generated')
