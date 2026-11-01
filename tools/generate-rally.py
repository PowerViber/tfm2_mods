"""Generate Rally constants from the canonical rules. --check never rewrites outputs."""
import argparse,json,copy
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
p=argparse.ArgumentParser();p.add_argument('--check',action='store_true');p.add_argument('--integrate',action='store_true');p.add_argument('--register',action='store_true');args=p.parse_args()
r=json.loads((ROOT/'rules/rally.json').read_text(encoding='utf8'))
c=r['constants'];id=r['id'];champ=copy.deepcopy(json.loads((ROOT/'mods/tfm2_custom/champion/tfm2_custom_contender.data_champion').read_text(encoding='utf8')))
champ.update(id=id,category=r['category'],tags=r['tags'],sprite='asset/tfm2_custom/champions/'+id,anim_prefix='')
for key in ['stat','growth']:
 champ[key]={k:0 for k in champ[key]};champ[key].update({('magic_resistance'if k=='resistance'else k):v for k,v in r[key].items()})
champ['attack'].update(action_name='attack',duration=18,start_timing=9,cooltime=c['BASIC_CD'],range=c['PALM_RANGE'],effect={'type':'Native','effect_ref':'tfm2_custom_ai:rally_basic'})
for slot in ['skill','skill2','ult']:champ[slot]['description']='#asset/base/text/champion?description.'+id+'.'+slot
champ['passive']={'passive_ref':'tfm2_custom_ai:rally','params':{'rules_revision':r['revision']}}
champ['passive_ult']={'passive_ref':'tfm2_custom_ai:ult_learned','params':{}}
champ['view_projectiles']=[];champ['view_effects']=[];champ['view_buffs']=[]
aliases={'ball':'ball_p0','receive':'receive','setter_ready':'set','setter':'helper','toss':'set','serve_ready':'set','serve_impact_ring':'serve_ring','white_release':'body_white_p0','white_blink':'white_blink','white_impact':'white_impact','impossible_charge':'body_white_p0','impossible_release':'body_white_p0','impossible_impact':'white_impact','overpressure_release':'white_blink','overpressure_impact':'white_impact','black_release':'body_black_p0','spike_release':'body_thunder_p0','contact':'ball_p0','impact':'impact','black_impact':'black_impact'}
for name,tag in aliases.items():champ['view_effects'].append(dict(type='Animation',name='rly_'+name,anim='asset/tfm2_custom/vfx/'+('rally-white' if name.startswith(('white_','impossible_','overpressure_')) else 'rally'),tag=tag,z=5,is_follow=False))
for name in ['stroke_contact','stroke_impact','stroke_black_contact','stroke_black_impact']:
 champ['view_effects'].append(dict(type='Animation',name='rly_'+name,anim='asset/tfm2_custom/vfx/rally-strokes',tag=name,z=4,is_follow=False))
for tag in ['range_dot']+[f'hud_{i}' for i in range(15)]+[f'impact_release_p{p}' for p in range(c['IMPACT_DRAWINGS'])]+[f'{name}_p{p}' for name in ['storm_bolt','storm_warning'] for p in range(8)]:
 champ['view_effects'].append(dict(type='Animation',name='rly_'+tag,anim='asset/tfm2_custom/vfx/rally-impact',tag=tag,z=7 if tag=='range_dot' or tag.startswith('storm_warning') else 4,is_follow=False))
if r['revision']=='rally-3.3':
 for tag in [f'charge_ring_{n}' for n in range(7)]+[f'ground_{style}' for style in ['normal','yellow','black','white','impossible']]+[f'chip_{style}' for style in ['normal','yellow','black','white','impossible']]:
  champ['view_effects'].append(dict(type='Animation',name='rly_'+tag,anim='asset/tfm2_custom/vfx/rally-impact',tag=tag,z=3 if tag.startswith(('ground_','chip_')) else 5,is_follow=False))
for phase in range(8):
 for name in ['ball','thunder','black','support','warning','body_thunder','body_black','warning_black','serve_edge','body_white','white','trail_white','crater','crater_serve','warning_serve']:
  champ['view_effects'].append(dict(type='Animation',name=f'rly_{name}_p{phase}',anim='asset/tfm2_custom/vfx/'+('rally-white' if name in ['body_white','white','trail_white','crater','crater_serve'] else 'rally'),tag=f'{name}_p{phase}',z=7 if name in ['warning','warning_black','warning_serve','serve_edge'] else 2 if name in ['body_thunder','body_black','body_white'] else 4,is_follow=False))
 for name in ['thunder','black','trail_thunder','trail_black','white','trail_white']:
  for heading in range(8):champ['view_effects'].append(dict(type='Animation',name=f'rly_{name}_a{heading}_p{phase}',anim='asset/tfm2_custom/vfx/'+('rally-white' if name in ['white','trail_white'] else 'rally'),tag=f'{name}_a{heading}_p{phase}',z=4,is_follow=False))
for prefix,count in [('pace',21),('role',3)]:
 for n in range(count):champ['view_buffs'].append(dict(type='Animated',name=f'rly_{prefix}{n}',anim='asset/tfm2_custom/vfx/rally',tag=f'{prefix}{n}',z=6))
champ['view_buffs'].append(dict(type='Animated',name='rly_support_orb',anim='asset/tfm2_custom/vfx/rally',tag='support',z=5))
for colour in ['yellow','thunder','white']:
 for form in ['masked','unmasked']:
  for facing in ['r','l']:
   for pose in ['idle','run','attack','set','receive']:
    champ['view_buffs'].append(dict(type='Animated',name=f'rly_body_{colour}_{form}_{pose}_{facing}',anim=champ['sprite'],tag=f'{colour}_{form}_{pose}_{facing}',z=3))
   for phase in range(8):
    for action in ['air','helper']:champ['view_effects'].append(dict(type='Animation',name=f'rly_{action}_{colour}_{form}_{facing}_p{phase}',anim=champ['sprite'],tag=f'{action}_{colour}_{form}_{facing}_p{phase}',z=3,is_follow=False))
for height in range(2000,c['MAX_HEIGHT_LABEL']+1,100):
 for prefix in ['height','contact_height']:champ['view_effects'].append(dict(type='Animation',name=f'rly_{prefix}_{height}',anim='asset/tfm2_custom/vfx/rally',tag=f'{prefix}_{height}',z=6,is_follow=False))
text={'name':'Rally',
 'skill':'Receive (8 s): Attack-scaling shield and timed protection, capped at 95%. Verified force banks up to 300% of an ordinary spike for the next spike or volley; native receipts remain gated. Run-up: directional sprint and optional timed takeoff, then contact to spike.',
 'skill2':'Set (4 s): self setup adds height and Power; teammate Set adds Attack/AP and movement. Choose a 1/3/5-bar Aura Burst for one play. Burst Set and Burst Spike cannot stack; every Burst play ends with 1.5 s exhaustion.',
 'ult':'Serve (75 s): aim in any direction, long-range wall-passing shot and +70% prepared height. Impossible requires prior White, five-bar funding and perfect contact; outside Serve contact must reach 10 m and range is 400,000. Serve Impossible is global within the realm. Charge 5 s (+200 Power), lock the final 1 s; slower harmless flight, x8 endpoint blast with distance falloff and warned lightning around visible enemy champions; one large 0.1-second contact impact frame. Height adds 15 Power per metre above 2 m. No vision reveal.'}

outputs={ROOT/'native/tfm2_custom_ai/src/rally_data.rs':'// Generated by tools/generate-rally.py. Edit rules/rally.json.\n'+f'pub const RULES_REVISION: &str = {json.dumps(r["revision"])};\n'+''.join(f'pub const {k}: i64 = {v};\n' for k,v in c.items())+'pub const MOD_POWER: [i32; 7] = '+json.dumps(r['modPower'])+';\n',ROOT/'editor/rally-data.js':'// Generated; edit rules/rally.json.\nwindow.RALLY_RULES='+json.dumps(r,separators=(',',':'))+';\n',ROOT/'editor/rally-preset.js':'window.RALLY_PRESET='+json.dumps(champ,separators=(',',':'))+';\nwindow.RALLY_TEXT='+json.dumps(text,separators=(',',':'))+';\n',ROOT/'editor/rally-champion.json':json.dumps(champ,indent=2)+'\n'}
if args.integrate:
 approval=json.loads((ROOT/'docs/rally-art-review/approval.json').read_text());assert approval.get('bodySourceSha256')and approval.get('effectsSourceSha256'),'Approve body and effects first'
 outputs[ROOT/'mods/tfm2_custom/champion'/f'{id}.data_champion']=json.dumps(champ,indent=2)+'\n'
 locpath=ROOT/'mods/tfm2_custom/text/champion.i18n';loc=json.loads(locpath.read_text(encoding='utf8'));loc['en']['description'][id]=text;outputs[locpath]=json.dumps(loc,ensure_ascii=False,indent=2)+'\n'
if args.register and not args.integrate and not (ROOT/'mods/tfm2_custom/champion'/f'{id}.data_champion').exists():
 raise SystemExit('Rally roster registration waits for the approved game definition; use the separate Lab preview meanwhile')
if args.integrate or args.register:
 rosterpath=ROOT/'rules/champions.json';roster=json.loads(rosterpath.read_text(encoding='utf8'));roster['champions']=[x for x in roster['champions']if x['id']!=id];roster['champions'].append(dict(id=id,name='Rally',family='original',role='Melee',tags=r['tags'],native=True,mastery=False,basicAttack=dict(name='Palm / Volley',kind='hybrid',range=c['VOLLEY_RANGE']),skills=[dict(slot='skill',name='Receive / Run-up → Spike',kinds=['Shield','Buff','Dash','Projectile','Aoe'],intents=['Protect','Clear','Poke','Engage','Objective'],summary=text['skill']),dict(slot='skill2',name='Self / Teammate Set',kinds=['Buff'],intents=['Protect','Engage'],summary=text['skill2']),dict(slot='ult',name='Serve / Impossible Spike',kinds=['Buff','Projectile','Aoe','Cc'],intents=['Engage','Objective'],summary=text['ult'])]));outputs[rosterpath]=json.dumps(roster,ensure_ascii=False,indent=1)+'\n'
bad=[]
for path,text in outputs.items():
 if args.check:
  if not path.exists() or path.read_text(encoding='utf8')!=text:bad.append(str(path.relative_to(ROOT)))
 else:path.parent.mkdir(parents=True,exist_ok=True);path.write_text(text,encoding='utf8',newline='\n')
if bad:raise SystemExit('Stale generated Rally data: '+', '.join(bad))
print('Rally canonical data '+('verified' if args.check else 'generated'))
