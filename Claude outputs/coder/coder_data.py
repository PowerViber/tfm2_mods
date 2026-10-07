"""Round 101: the Coder's champion data and text: mods/tfm2_custom/champion/tfm2_custom_coder.data_champion and his
entry in mods/tfm2_custom/text/champion.i18n.

Every view the native code plays is listed here (coder.rs names them '<id>_<tag>' for effects, plain for buffs); the
anims come from the sheets the other scripts write (coder_code_<lang>, coder_ui, coder_vfx). His three skills are
never cast by the game (like Scribble's): his brain runs them.

Run from the repo root after the art: python3 "Claude outputs/coder/coder_data.py"
"""
import json
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from coder_functions import FUNCS, LANGS  # noqa: E402

ROOT = os.path.abspath(os.path.join(HERE, '..', '..'))
MOD = os.path.join(ROOT, 'mods', 'tfm2_custom')
ID = 'tfm2_custom_coder'
VFX = 'asset/tfm2_custom/vfx/'


def never(action):
    return {"action_name": action, "cancelable": False, "growth_range": 0, "can_use_with_move": False, "duration": 6,
            "cooltime": 5184000, "start_timing": 3, "range": 1, "casting_type": "Targeting",
            "casting_target": "EnemyChampionInCC", "attack_type": "Skill", "effect": {"type": "Combine", "effects": []},
            "description": f"#asset/base/text/champion?description.{ID}.{action if action != 'skill1' else 'skill'}"}


def views():
    fx, buffs = [], []
    fan = lambda name: json.load(open(os.path.join(MOD, 'vfx', name + '#anim.fanim'), encoding='utf-8'))['anims']
    for lang in LANGS:
        for tag in fan(f'coder_code_{lang}'):
            fx.append({"type": "Animation", "name": ID + '_' + tag, "anim": VFX + f'coder_code_{lang}', "tag": tag, "z": 6, "is_follow": True})
    for tag in fan('coder_ui'):
        fx.append({"type": "Animation", "name": ID + '_' + tag, "anim": VFX + 'coder_ui', "tag": tag, "z": 6, "is_follow": True})
    point = ('fx_scan', 'fx_spray', 'fx_blink_out', 'fx_blink_in', 'fx_wall_')
    for tag in fan('coder_vfx'):
        if tag.startswith('fx_'):
            fx.append({"type": "Animation", "name": ID + '_' + tag, "anim": VFX + 'coder_vfx', "tag": tag,
                       "z": 2 if tag.startswith('fx_wall_') else 3, "is_follow": not tag.startswith(point)})
        elif tag != 'bit':
            z = {'shield': 3, 'lag': 5, 'oc': 4}.get(tag, 5)
            buffs.append({"type": "Animated", "name": 'cd_' + tag, "anim": VFX + 'coder_vfx', "tag": tag, "z": z})
    proj = [{"type": "Animated", "name": ID + '_bit', "anim": VFX + 'coder_vfx', "tag": 'bit', "z": 2, "repeat": True}]
    return fx, buffs, proj


def champion():
    fx, buffs, proj = views()
    return {
        "id": ID,
        "category": "Magician",
        "tags": ["AP", "Range", "Util"],
        "sprite": "asset/tfm2_custom/champions/" + ID,
        "anim_prefix": "",
        "stat": {"attack": 76, "magic_power": 45, "hp": 860, "defence": 20, "magic_resistance": 22, "move_speed": 960,
                 "hp_regen": 0, "stack": 0, "crit_chance": 0},
        "growth": {"attack": 6, "magic_power": 20, "hp": 96, "defence": 6, "magic_resistance": 4, "move_speed": 9,
                   "hp_regen": 0, "stack": 0, "crit_chance": 0},
        "attack": {"action_name": "attack", "cancelable": True, "growth_range": 0, "can_use_with_move": False,
                   "duration": 22, "cooltime": 84, "start_timing": 12, "range": 60000, "casting_type": "Targeting",
                   "casting_target": "Enemy", "attack_type": "BaseAttack",
                   "effect": {"type": "TargetProjectile", "speed": 8000, "name": ID + "_bit", "y_offset": 0,
                              "applied_target": "Enemy",
                              "applied_effects": [{"effect": {"type": "Attack", "damage": 0, "attack_ratio": 100},
                                                   "casting_type": "Targeting"}]}},
        "skill": never("skill1"),
        "skill2": never("skill2"),
        "ult": never("ult"),
        "passive": {"passive_ref": "tfm2_custom_ai:coder", "params": {}},
        "view_effects": fx,
        "view_projectiles": proj,
        "view_buffs": buffs,
    }


TEXT = {
    "name": "Coder",
    "skill": ("He fights by writing code. He picks a function and types its real code (shown over his head) at his "
              "rank's speed; every character can be a typo, more on symbols, in C++ or Rust, when hot or overclocked. "
              "A typo is a SyntaxError (the compile fails, he retypes the line) or a logic bug that compiles and "
              "misbehaves: wrong target, off-by-one, infinite loop (he freezes), null reference (it fizzles), sign flip "
              "(it heals the enemy); C++ also segfaults and leaks memory. He reviews before compiling and catches each "
              "typo at his rank's eye; Rust's compiler catches most logic bugs too, but compiles slowly. A compiled "
              "function joins his program (5 slots) and runs itself whenever its trigger holds: ping, shield, heal, "
              "scan, spray, blink, slow, cache, chain, firewall. Every rank can write every function; the rank decides "
              "how fast, how clean, and how well he judges what he can pull off (low ranks overreach). "
              "Overclock: +30% clock and +20% typing, double heat. His rig: a 3.0 GHz CPU (load; throttled above 85 C), "
              "16 GB of RAM (lasting effects hold it; over the top is Out of memory: the newest is killed), 8 storage "
              "slots (a saved function reloads in 1 s, bugs and all) and heat: at 100 C he blue-screens, stunned, and "
              "every unsaved function is lost. Ranks: Script Kiddie, Intern, Junior, Developer, Senior, Staff, "
              "Architect, Root (Top 10; #1 Zero-Day)."),
    "skill2": ("debug: every 10 s while he isn't typing he reads his program over and fixes each bug he notices "
               "(his review eye +10%)."),
    "ult": "Activate AI: Claude Max 20x, ChatGPT Pro or Gemini AI Ultra write for him (coming in the next update).",
}


if __name__ == '__main__':
    c = champion()
    with open(os.path.join(MOD, 'champion', ID + '.data_champion'), 'w', encoding='utf-8') as fh:
        fh.write(json.dumps(c, indent=2, ensure_ascii=False) + '\n')
    tp = os.path.join(MOD, 'text', 'champion.i18n')
    t = json.load(open(tp, encoding='utf-8'))
    t['en']['description'][ID] = TEXT
    raw = open(tp, encoding='utf-8').read()
    indent = 2 if raw.startswith('{\n  ') else None
    with open(tp, 'w', encoding='utf-8') as fh:
        fh.write(json.dumps(t, indent=indent, ensure_ascii=False) + ('\n' if raw.endswith('\n') else ''))
    print(ID, len(c['view_effects']), 'effects,', len(c['view_buffs']), 'buffs')
