"""Generate inert action shells and native basic impact; preserve Gojo body/stats/growth."""
from pathlib import Path
import copy,json
ROOT=Path(__file__).resolve().parents[1];P=ROOT/'mods/tfm2_jjk';ID='tfm2_custom_gojo'
def generate():
    path=P/'champion'/f'{ID}.data_champion';d=json.loads(path.read_text(encoding='utf-8'));d['passive']={'passive_ref':'tfm2_custom_ai:gojo_v2','params':{}}
    d['attack']['effect']={'type':'Native','effect_ref':'tfm2_custom_ai:gojo_basic_v2'}
    for k in ['skill','skill2','ult']:
        d[k].update(duration=6,cooltime=5184000,start_timing=3,range=1,casting_type='Targeting',casting_target='EnemyChampionInCC',effect={'type':'Combine','effects':[]})
    aliases=['six_eyes','enhanced','black_flash','rct','step_arrive','blue_prepare','red_prepare','purple_flight','blue_impact','red_impact','void_full_field','guest']
    d['view_effects']=[v for v in d.get('view_effects',[]) if not v['name'].startswith('tfm2_custom_gojo_new_')]+[{'type':'Animation','name':f'tfm2_custom_gojo_new_{i}','anim':'asset/tfm2_jjk/vfx/gojo_v2','tag':tag,'z':1,'is_follow':False} for i,tag in enumerate(aliases)]
    d['view_effects']=[v for v in d['view_effects'] if not v['name'].startswith('tfm2_custom_gojo_ring_')]+[{'type':'Animation','name':f'tfm2_custom_gojo_ring_{tag}','anim':'asset/tfm2_jjk/vfx/gojo_rework','tag':tag,'z':-1,'is_follow':False} for tag in [f'domain{n}' for n in range(8)]+[f'warning{n}' for n in [3,20,26,32,36,76]]]
    d['view_effects']=[v for v in d['view_effects'] if not v['name'].startswith('tfm2_custom_gojo_flight_')]+[{'type':'Animation','name':f'tfm2_custom_gojo_flight_{tag}{n}','anim':'asset/tfm2_jjk/vfx/gojo_rework','tag':f'flight_{tag}{n}','z':1,'is_follow':False} for tag,count in [('blue',4),('red',4),('purple',6)] for n in range(count)]
    d['view_buffs']=[v for v in d.get('view_buffs',[]) if not v['name'].startswith('gj_')]
    for key in ['ce','strain']:
        for i in range(21):d['view_buffs'].append({'type':'Animated','name':f'gj_{key}{i}','anim':'asset/tfm2_jjk/vfx/gojo_v2','tag':key+str(i),'z':3})
    for name,tag in [('gj_six_eyes','six_eyes'),('gj_amp','amplified'),('gj_burnout','burnout'),('gj_inf_on','infinity_on'),('gj_inf_rebuild','infinity_rebuild'),('gj_inf_paused','infinity_paused'),('gj_inf_broken','infinity_break')]:d['view_buffs'].append({'type':'Animated','name':name,'anim':'asset/tfm2_jjk/vfx/gojo_v2','tag':tag,'z':1})
    d['view_effects']=[v for v in d['view_effects'] if not v['name'].startswith('tfm2_custom_gojo_v2_')]
    vm=json.loads((P/'vfx/gojo_v2#anim.fanim').read_text(encoding='utf-8'))
    for tag in vm['anims']:
        d['view_effects'].append({'type':'Animation','name':'tfm2_custom_gojo_v2_'+tag,'anim':'asset/tfm2_jjk/vfx/gojo_v2','tag':tag,'z':-1 if tag.startswith('void_') else 1,'is_follow':False})
    # Include enlarged Domain/nuke warning sizes; retain old named bindings.
    for radius in [3,20,26,32,36,64,76,96]:
        name=f'tfm2_custom_gojo_ring_warning{radius}';d['view_effects']=[v for v in d['view_effects'] if v['name']!=name]
        d['view_effects'].append({'type':'Animation','name':name,'anim':'asset/tfm2_jjk/vfx/gojo_v2','tag':f'warning{radius}','z':-1,'is_follow':False})
    for i in range(14,18):d['view_effects'].append({'type':'Animation','name':f'tfm2_custom_gojo_new_{i}','anim':'asset/tfm2_jjk/vfx/gojo_v2','tag':f'fx{i}','z':1,'is_follow':False})
    d['view_buffs'].append({'type':'Animated','name':'gj_eye_exhausted','anim':'asset/tfm2_jjk/vfx/gojo_v2','tag':'fx17','z':1})
    path.write_text(json.dumps(d,indent=2)+'\n',encoding='utf-8');(ROOT/'editor/gojo-champion.json').write_text(json.dumps(d,separators=(',',':'))+'\n')
    text={'name':'Gojo','skill':'Six Eyes active by default: 90% less CE, twice normal technique cooldown recovery, 20% less kit damage. 100 CE; regenerates 3/s or 6/s out of champion combat. Eye strain caps at 250; exhausted eyes recover 20/s and restore at 50. Active strain recovers 4/s after 1.5s. Infinity retains its existing shield and reductions; .1 CE/s upkeep and .5 CE rebuild while eyes function. Remaining shield stays during pauses without refilling. Blue: .6 CE / 5 strain, .4s preparation; existing damage once and a .6s inward spiral. Maximum Blue: 1.2 CE / 70 strain, .9s preparation, 135% damage and one six-second orb. Maximum outputs require active Six Eyes and the eight-second Black Flash window.',
    'skill2':'Red: 1 CE / 7 strain, .4s preparation; existing direct hit and explosion. Maximum Red: 2 CE / 80 strain, 1s preparation and 135% damage. Its direct collision with your maximum Blue orb forms Hollow Nuke: .8s fixed warning, 400 + 240% AP magic damage in 96000 radius. It consumes half current HP (rounded down), bypassing shields, and adds 75 strain after impact. Body blocking prevents fusion. RCT: 2 CE / 8 strain, heals 80 + 50% AP; unavailable with exhausted eyes. Spatial Step: .8 CE / 8 strain, legal visible route up to 32000. Black Flash: two champion-bound melee hits after Blue/Red within four seconds; +30 + 35% AP, -10 strain and +20% kit damage for eight seconds, refresh without stacking. Triggering strike is resolved before amplification. Six Eyes with Black Flash gives 96% ordinary kit damage.',
    'ult':'Unlimited Void requires Six Eyes. Quick: 96000 radius, .2s warning, .2s exposure and one 1s stun; 50 strain. Full: 64000 radius, 1s warning and 4s maintained seal; 80 strain. Both cost 4 CE and share 60s base cooldown (30s with Six Eyes). One nearest living ally is the fixed protected guest; every other caught ally is controlled like an enemy. Closure causes 3s Limitless burnout. Ordinary Purple requires eyes, ultimate learned and both Blue/Red ready: 3.4 CE / 30 strain, 200 + 160% AP. Without Six Eyes only melee, ordinary Blue/Red and Spatial Step remain at full CE cost and normal cooldown recovery. Numeric rules and whirlpool/nuke behavior are gameplay adaptations; no mastery ranks.'}
    loc=json.loads((P/'text/champion.i18n').read_text(encoding='utf-8'));loc.setdefault('en',{}).setdefault('description',{})[ID]=text;(P/'text/champion.i18n').write_text(json.dumps(loc,indent=2)+'\n',encoding='utf-8')
    (ROOT/'editor/gojo-preset.js').write_text('window.GOJO_PRESET='+json.dumps({'json':d,'text':text,'vfx':'gojo','sprite':'gojo'},separators=(',',':'))+';\n')
if __name__=='__main__':generate()
