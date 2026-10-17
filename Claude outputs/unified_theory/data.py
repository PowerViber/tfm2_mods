"""Champion data and views. Run after art.py. Never edit generated champion data."""
import json, importlib.util
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]; MOD=ROOT/'mods/tfm2_custom'; ID='tfm2_custom_unified_theory';VFX='asset/tfm2_custom/vfx/'
# Above-head labels, values, charge bar and reserve icons; effects that ride on a unit.
LABELS=('eq','sig','uni','ch_','rv','n0_','n1_','n2_','n3_')
FOLLOW=('cast','shield','fizzle','insight','dot','lawshield','ult')
spec=importlib.util.spec_from_file_location('coder_data',ROOT/'Claude outputs/coder/coder_data.py');coder=importlib.util.module_from_spec(spec);spec.loader.exec_module(coder)

def write(path, text):
    temporary = path.with_name(path.name + '.tmp')
    temporary.write_text(text, encoding='utf-8', newline='\n')
    temporary.replace(path)
def generate():
    c=coder.champion();c['id']=ID;c['sprite']='asset/tfm2_custom/champions/'+ID
    c['stat'].update(attack=70,magic_power=45,hp=1100,defence=25,magic_resistance=28)
    c['growth'].update(attack=5,magic_power=18,hp=120,defence=7,magic_resistance=7)
    c['passive']={'passive_ref':'tfm2_custom_ai:unified_theory','params':{}}
    for key in ['skill','skill2','ult']:c[key]=coder.never(c[key]['action_name']);c[key]['description']=c[key]['description'].replace('tfm2_custom_coder',ID)
    c['attack']['effect']['name']=ID+'_bit';c['view_effects']=[];c['view_buffs']=[]
    for file in sorted((MOD/'vfx').glob('science_*#anim.fanim')):
        sheet=file.name.split('#')[0]
        for tag in json.loads(file.read_text(encoding='utf-8'))['anims']:
            if tag=='crystal_body' or tag.startswith(('pose','transform','persona','rank','top','outfit','gear','modifiers')):
                c['view_buffs'].append({'type':'Animated','name':'ut_'+tag,'anim':VFX+sheet,'tag':tag,'z':-1 if tag.startswith('gearback') else 6 if tag.startswith(('rank','top')) else 4})
            else:c['view_effects'].append({'type':'Animation','name':ID+'_'+tag,'anim':VFX+sheet,'tag':tag,'z':7 if tag.startswith(LABELS) else -1 if tag.startswith(('zn','puddle')) else 2,'is_follow':tag.startswith(FOLLOW) and tag!='ultwarn'})
    c['view_projectiles']=[{'type':'Animated','name':ID+'_bit','anim':VFX+'science_vfx','tag':'packet0','z':2,'repeat':True}]
    write(MOD/'champion'/f'{ID}.data_champion',json.dumps(c,indent=2)+'\n')
    text={"name": "Unified", "skill": "Einstein, Newton and Marie Curie share one body and 18 equations, six each; every number in an equation is its effect. Newton: Apple (E = m\u00b7g\u00b7h, more damage from higher), Prism (refracts into three rays), Orbit (F \u221d 1/r\u00b2, strongest point-blank), Cradle (momentum passes to the last enemy in line), Third Law (the reaction equals the damage taken), Calculus (dHP/dt: the next hit adds what the target just lost). Curie: Radium (a zone whose dose halves every 2 s), Alpha, Beta (curves), Gamma (through walls), Half-life (a, a/2, a/4... = 2a) and Chain Reaction (k = 1 inside Radium). Einstein: Time Field (slows by \u03b3: fast movers most), E = mc\u00b2 (% max HP as true damage), Einstein Ring (bends Unified's rays), Twin Paradox (an ally returns to 3 s ago), Spacetime Well (pulls \u221d 1/r\u00b2) and Photoelectric (only frees energy above two marks). Each cast writes its equation and value above Unified.", "skill2": "Five charges, each refilling on its own 6 s timer. Every skill is either cast or reserved: a reserved skill spends a charge, becomes a trait of the next cast and lends it 80% of its power (Prism \u25b8 Spacetime Well: the well fires lasers; Spacetime Well \u25b8 Prism: the rays pull). Up to 4 reserved + 1 cast make a 5-step combo; each link adds 25%, a step within 0.4 s is an Insight (+15%), and a reserve left 4 s fizzles. 36 named signatures add a bonus and their own effect (Criticality, Newton's Garden, Curie's Notebook, Annus Mirabilis...). A combo with all three scientists fires a Unification chosen by their order: Burst, Field, Life, Shield, Proof or Motion. Cooldowns start when the combo is cast. Skill damage is limited to 35% of a target's max HP per 2 s.", "ult": "Theory of Everything: after a 0.6 s warning, replays the last combo at 125% without spending charges, plus anything reserved now (Apple \u25b8 Prism \u25b8 Orbit \u25b8 Radium \u25b8 ultimate is its own signature). Cooldown 60 s; requires the ultimate. Mastery: Student, Lab Assistant, Researcher, Scientist, Professor, Fellow, Laureate, Unified Mind (Top 10). Rank never adds power: it decides how well the equations are read, how far aim leads, how long the combos get, which signatures are known and how often a step lands as an Insight. A Student fires Photoelectric with no marks; a Unified Mind chains five-step signatures."}
    p=MOD/'text/champion.i18n';raw=p.read_text(encoding='utf-8');t=json.loads(raw);t['en']['description'][ID]=text;write(p,json.dumps(t,indent=2 if raw.startswith('{\n  ')else None,ensure_ascii=False)+ ('\n'if raw.endswith('\n')else''))
    print(ID,len(c['view_effects']),'effects',len(c['view_buffs']),'persona / mastery overlays')
if __name__=='__main__':generate()
