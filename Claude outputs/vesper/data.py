"""Vesper's champion data and views. Run after art.py. Never edit generated champion data."""
import importlib.util
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
MOD = ROOT / 'mods/tfm2_custom'
ID = 'tfm2_custom_vesper'
VFX = 'asset/tfm2_custom/vfx/'
spec = importlib.util.spec_from_file_location('coder_data', ROOT / 'Claude outputs/coder/coder_data.py')
coder = importlib.util.module_from_spec(spec)
spec.loader.exec_module(coder)

BUFFS = ('outfit_', 'aim_', 'rank', 'top', 'back_', 'charges', 'strain')
FOLLOW = ('rift_anchor', 'core_', 'glyph', 'tech_', 'breach_', 'atlas_warn')


def write(path, text):
    temporary = path.with_name(path.name + '.tmp')
    temporary.write_text(text, encoding='utf-8', newline='\n')
    temporary.replace(path)


def buff_z(tag):
    return -1 if tag.startswith('back_') else 6 if tag.startswith(('rank', 'top')) else 7 if tag.startswith(('charges', 'strain')) else 4


def effect_z(tag):
    if tag.startswith(('rift_sinkhole', 'rift_seed', 'seedhint', 'atlas_warn')):
        return -1
    if tag.startswith(('rift_', 'link_')):
        return 1
    if tag.startswith(('core_', 'open_', 'close_', 'break_')):
        return 2
    if tag.startswith(('glyph', 'packet_', 'impact_')):
        return 3
    if tag.startswith(('tech_', 'twist')):
        return 4
    return 5   # breach, bolt


def generate():
    c = coder.champion()
    c['id'] = ID
    c['category'] = 'Range'
    c['tags'] = ['AD', 'CC', 'Range']
    c['sprite'] = 'asset/tfm2_custom/champions/' + ID
    c['stat'].update(attack=85, magic_power=0, hp=950, defence=22, magic_resistance=24)
    c['growth'].update(attack=7, magic_power=0, hp=95, defence=5, magic_resistance=5)
    c['attack'].update(range=66000, cooltime=70)
    c['attack']['effect']['name'] = ID + '_bolt'
    c['passive'] = {'passive_ref': 'tfm2_custom_ai:vesper', 'params': {}}
    for key in ['skill', 'skill2', 'ult']:
        c[key] = coder.never(c[key]['action_name'])
        c[key]['description'] = c[key]['description'].replace('tfm2_custom_coder', ID)
    c['view_effects'] = []
    c['view_buffs'] = []
    for file in sorted((MOD / 'vfx').glob('vesper_*#anim.fanim')):
        sheet = file.name.split('#')[0]
        for tag in json.loads(file.read_text(encoding='utf-8'))['anims']:
            if tag.startswith(BUFFS):
                c['view_buffs'].append({'type': 'Animated', 'name': 'vs_' + tag, 'anim': VFX + sheet, 'tag': tag, 'z': buff_z(tag)})
            elif tag != 'bolt':
                c['view_effects'].append({'type': 'Animation', 'name': ID + '_' + tag, 'anim': VFX + sheet, 'tag': tag, 'z': effect_z(tag), 'is_follow': tag.startswith(FOLLOW)})
    bolt = next(f.name.split('#')[0] for f in sorted((MOD / 'vfx').glob('vesper_*#anim.fanim')) if 'bolt' in json.loads(f.read_text(encoding='utf-8'))['anims'])
    c['view_projectiles'] = [{'type': 'Animated', 'name': ID + '_bolt', 'anim': VFX + bolt, 'tag': 'bolt', 'z': 2, 'repeat': True}]
    write(MOD / 'champion' / f'{ID}.data_champion', json.dumps(c, indent=2) + '\n')
    text = {
        'name': 'Vesper',
        'skill': 'Vesper, the Rift Marksman, has no active skills and no ultimate: she tears space. Five rift charges each recover in 5 s, and a tear lives about 5 s. She can tear anywhere on the map, but far tears open later (the rip travels from her) and every link loses damage, so a long shot is chained through several tears. While a chain is open, every basic attack is rerouted through it as a packet that hops from tear to tear. The arena is a 10 x 10 atlas of 100 dimensions in 10 families (Control, Ruin, Motion, Haven, Passage, Time, Veil, Matter, Hunger, Chaos); each tear takes the dimension of its cell, the atlas drifts every 30 s, and the last two families a packet passes through add one of 55 family-pair twists.',
        'skill2': 'Ten rift types (Tear, Slash, Sinkhole, Sky Rift, Mirror Pane, Twin Rift, Wandering Rift, Anchor Rift, Seed Rift, Maw) and 28 techniques: Chain Fire, Crossfire, Bank Shot, Rift Rain, Slash Volley, Implode, Domino, Overload Shot, Rift Step, Ally Gate, Sky Drop, Kidnap, Sinkhole Trap, Swap, Fling, Hook, Phase Out, Rift Curtain, Reflect Wall, Rescue, Pocket, Lifeline, Escort, Rotation Gate, Seed Trap, Sweep, Borrow and Stitch. Enemies standing on a tear for 1 s break it. Every tear, long link and break builds rift strain; at 100 a Breach erupts on her, hitting both teams, costing 6% maximum HP (never lethal) and collapsing every tear.',
        'ult': 'Mastery: Spark, Tear, Rift, Gate, Wanderer, Cartographer, Riftwalker, Worldbreaker (Top 10); #1 is Between Worlds. Every rift type and technique is available at rank 0, and rank never adds raw power: it improves prediction, safety, chain geometry, survival, team engages, target priority and five-step sequences. A Spark opens Maws on one target and breaches; a Worldbreaker seeds the healer’s path, sky-drops the tank onto it and implodes a domino through the right cells. Each rank adds a visible layer, and her tears change look per rank. Emitted damage is limited to 35% of target maximum HP over two seconds. Open the Rift Lab in the editor for the atlas, all 100 dimensions and every technique.',
    }
    p = MOD / 'text/champion.i18n'
    raw = p.read_text(encoding='utf-8')
    t = json.loads(raw)
    t['en']['description'][ID] = text
    write(p, json.dumps(t, indent=2 if raw.startswith('{\n  ') else None, ensure_ascii=False) + ('\n' if raw.endswith('\n') else ''))
    print(ID, len(c['view_effects']), 'effects', len(c['view_buffs']), 'rank / HUD overlays')


if __name__ == '__main__':
    generate()
