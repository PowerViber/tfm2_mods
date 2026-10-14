"""Vesper's champion data and views. Run after art.py. Never edit generated champion data."""
import importlib.util
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
MOD = ROOT / 'mods/tfm2_custom'
ID = 'tfm2_custom_vesper'
VFX = 'asset/tfm2_custom/vfx/'
spec = importlib.util.spec_from_file_location('coder_data', ROOT / 'Claude outputs/coder/coder_data.py')
coder = importlib.util.module_from_spec(spec)
spec.loader.exec_module(coder)

BUFF = re.compile(r'^((outfit|jab|block|punch|swipe|shatter)_r|top\d|back_|orbs|hex(\d|_x)|core(\d|_aw|_x)|stress)')
FOLLOW = ('rift_anchor', 'core_f', 'glyph', 'tech_', 'punch_t', 'gbreak_', 'gfix_', 'awaken_', 'corrupt', 'overload_')


def write(path, text):
    temporary = path.with_name(path.name + '.tmp')
    temporary.write_text(text, encoding='utf-8', newline='\n')
    temporary.replace(path)


def buff_z(tag):
    return -1 if tag.startswith('back_') else 6 if tag.startswith('top') else 7 if tag.startswith(('orbs', 'hex', 'core', 'stress')) else 4


def effect_z(tag):
    if tag.startswith(('rift_sinkhole', 'rift_seed', 'seedhint')):
        return -1
    if tag.startswith(('rift_', 'link_', 'swipe_')):
        return 1
    if tag.startswith(('core_', 'open_', 'close_', 'break_')):
        return 2
    if tag.startswith(('glyph', 'packet_', 'impact_', 'hole_')):
        return 3
    if tag.startswith(('tech_', 'twist', 'mod')):
        return 4
    return 5   # punch, shatter, gauntlet breaks, Awakening, overload


def generate():
    c = coder.champion()
    c['id'] = ID
    c['category'] = 'Range'
    c['tags'] = ['AD', 'CC', 'Range']
    c['sprite'] = 'asset/tfm2_custom/champions/' + ID
    c['stat'].update(attack=85, magic_power=0, hp=950, defence=22, magic_resistance=24)
    c['growth'].update(attack=7, magic_power=0, hp=95, defence=5, magic_resistance=5)
    c['attack'].update(range=45000, cooltime=70)
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
            if BUFF.match(tag):
                c['view_buffs'].append({'type': 'Animated', 'name': 'vs_' + tag, 'anim': VFX + sheet, 'tag': tag, 'z': buff_z(tag)})
            elif tag != 'bolt':
                c['view_effects'].append({'type': 'Animation', 'name': ID + '_' + tag, 'anim': VFX + sheet, 'tag': tag, 'z': effect_z(tag), 'is_follow': tag.startswith(FOLLOW)})
    bolt = next(f.name.split('#')[0] for f in sorted((MOD / 'vfx').glob('vesper_*#anim.fanim')) if 'bolt' in json.loads(f.read_text(encoding='utf-8'))['anims'])
    c['view_projectiles'] = [{'type': 'Animated', 'name': ID + '_bolt', 'anim': VFX + bolt, 'tag': 'bolt', 'z': 2, 'repeat': True}]
    write(MOD / 'champion' / f'{ID}.data_champion', json.dumps(c, indent=2) + '\n')
    text = {
        'name': 'Vesper',
        'skill': 'Vesper fights with the Rift Gauntlets and has no active skills and no ultimate. Her normal jab hits minions, creeps and champions. The left gauntlet holds 5 pure orbs (overload to 9), each on its own 5 s cooldown; the right gauntlet holds hexorbs, a pure orb fused with Hextech at the Hexcore completion % (2 slots at 70%, up to 6 at 90%, faster and weaker). Every orb is a key that opens any of the 100 dimensions, and the dimension decides the effect. With an orb active she rift punches: the fist goes through a tear and comes out of the exit with that dimension\u2019s punch effect. Spending two orbs, she rift swipes, tearing the dimension along a line like pulling off a mask. Orbs activated within 1 s combo (+10% per step up to +40%), and two families in a row add one of 55 family-pair twists.',
        'skill2': 'Ten rift types and 28 techniques, including Overload Punch and Infuse (a hexorb\u2019s tech module on a tear for its whole life). Shatter crushes every pure orb she holds: every open rift rips wide and banishes and damages the enemies inside, never allies. Overloaded orbs and enemy hits build gauntlet stress; past 100 the left gauntlet breaks (offline 8 s, 6% max HP, never lethal). The Hexcore researches as she fights (+1% per orb, +3% per takedown, +4% per Shatter); at 100% it Awakens for 6 s (hexorbs at full power, fused orbs, unbreakable tears). Overclocking past that builds corruption, the only way the right gauntlet breaks. Tech modules: Plating, Coil, Stasis, Magnet, Mender, Scanner, Spark Arc, Anchor Bolt.',
        'ult': 'Mastery: Spark, Tear, Rift, Gate, Wanderer, Cartographer, Riftwalker, Worldbreaker (Top 10); #1 is Between Worlds. Every rift type, technique and gauntlet move is available at rank 0, and rank never adds raw power: it improves which dimension each orb opens, safety with overload and Shatter, combo sequencing and Awakening timing. A Spark overloads at random, shatters into empty tears and breaks her gauntlet; a Worldbreaker rewinds the diver, slams them into a wall with Gale and rides overload into a Shatter that catches the whole team. There are no auras: each rank shows more of the damage her fists do to reality (sparks, cracks, scars, a wound behind her, and at #1 gold gauntlets and a massive tear), and the Top 10 carry a small number. Emitted damage is limited to 35% of target maximum HP over two seconds. Open the Rift Lab in the editor for the Gauntlet panel, all 100 dimensions (300 behaviours) and every technique.',
    }
    p = MOD / 'text/champion.i18n'
    raw = p.read_text(encoding='utf-8')
    t = json.loads(raw)
    t['en']['description'][ID] = text
    write(p, json.dumps(t, indent=2 if raw.startswith('{\n  ') else None, ensure_ascii=False) + ('\n' if raw.endswith('\n') else ''))
    print(ID, len(c['view_effects']), 'effects', len(c['view_buffs']), 'rank / HUD overlays')


if __name__ == '__main__':
    generate()
