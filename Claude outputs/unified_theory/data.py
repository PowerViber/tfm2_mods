"""Champion data and views. Run after art.py. Never edit generated champion data."""
import json, importlib.util
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]; MOD=ROOT/'mods/tfm2_custom'; ID='tfm2_custom_unified_theory';VFX='asset/tfm2_custom/vfx/'
spec=importlib.util.spec_from_file_location('coder_data',ROOT/'Claude outputs/coder/coder_data.py');coder=importlib.util.module_from_spec(spec);spec.loader.exec_module(coder)
def generate():
    c=coder.champion();c['id']=ID;c['sprite']='asset/tfm2_custom/champions/'+ID
    c['stat'].update(attack=70,magic_power=45,hp=1100,defence=25,magic_resistance=28)
    c['growth'].update(attack=5,magic_power=18,hp=120,defence=7,magic_resistance=7)
    c['passive']={'passive_ref':'tfm2_custom_ai:unified_theory','params':{}}
    for key in ['skill','skill2','ult']:c[key]=coder.never(c[key]['action_name']);c[key]['description']=c[key]['description'].replace('tfm2_custom_coder',ID)
    c['attack']['effect']['name']=ID+'_bit';c['view_effects']=[];c['view_buffs']=[]
    for file in sorted((MOD/'vfx').glob('science_*#anim.fanim')):
        sheet=file.name.split('#')[0]
        for tag in json.loads(file.read_text())['anims']:
            if tag.startswith(('persona','rank','top','outfit','gear','modifiers')):
                c['view_buffs'].append({'type':'Animated','name':'ut_'+tag,'anim':VFX+sheet,'tag':tag,'z':-1 if tag.startswith('gearback') else 6 if tag.startswith(('rank','top')) else 4})
            else:c['view_effects'].append({'type':'Animation','name':ID+'_'+tag,'anim':VFX+sheet,'tag':tag,'z':7 if tag.startswith(('note','meter','allocation')) else -1 if tag.startswith(('echo','complete','skill_')) else 2,'is_follow':tag.startswith(('cast','skill_','complete','shield','fizzle'))})
    c['view_projectiles']=[{'type':'Animated','name':ID+'_bit','anim':VFX+'science_vfx','tag':'packet0','z':2,'repeat':True}]
    (MOD/'champion'/f'{ID}.data_champion').write_text(json.dumps(c,indent=2)+'\n')
    text={'name':'The Unified Theory','skill':'One body, three scientists. Einstein: light, relativity and geometry. Newton: vectors, momentum and calculus. Marie Curie: chemistry, reactions and radiation. Each has 25 symbolic recipes. The athlete writes tokens at a common speed, reserves Charge Units and commits stages. All 75 skills are available at every mastery rank. Good proportions matter as much as total charge.','skill2':'100 CU, regeneration 4/s in combat and 8/s after 5 seconds without combat. Curie starts with 8 of 12 material units and gains one every 3 seconds. Newton earns momentum through measured movement and valid impulses. Excess or uneven charge causes drift, malfunction or critical cancellation. Uncommitted charge is refunded; failed commitments are spent.','ult':'Mastery: Student, Lab Assistant, Researcher, Scientist, Professor, Fellow, Laureate, Unified Mind (Top 10). Mastery improves accuracy, forecasting and experiment choice. Split, orbit, piercing, clouds and decay share the original experiment damage budget; emitted skill damage is limited to 35% of target maximum HP over two seconds. Open Science Lab in the editor for all 75 recipes and 50 experiments.'}
    p=MOD/'text/champion.i18n';raw=p.read_text();t=json.loads(raw);t['en']['description'][ID]=text;p.write_text(json.dumps(t,indent=2 if raw.startswith('{\n  ')else None,ensure_ascii=False)+ ('\n'if raw.endswith('\n')else''))
    print(ID,len(c['view_effects']),'effects',len(c['view_buffs']),'persona / mastery overlays')
if __name__=='__main__':generate()
