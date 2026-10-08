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
    # round 102: the code lines are point effects placed over him (coder.rs TERM_DY), over as many sheets as they take
    import glob
    for path in sorted(glob.glob(os.path.join(MOD, 'vfx', 'coder_code_*#anim.fanim'))):
        sheet = os.path.basename(path).split('#')[0]
        for tag in fan(sheet):
            fx.append({"type": "Animation", "name": ID + '_' + tag, "anim": VFX + sheet, "tag": tag, "z": 6, "is_follow": False})
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
    # round 103: the high-rank rig (behind him, in front of him) and the top-rank effects
    for tag in fan('coder_rank'):
        if tag.startswith('fx_'):
            fx.append({"type": "Animation", "name": ID + '_' + tag, "anim": VFX + 'coder_rank', "tag": tag, "z": 3,
                       "is_follow": not tag.startswith(point)})
        else:
            buffs.append({"type": "Animated", "name": 'cd_' + tag, "anim": VFX + 'coder_rank', "tag": tag,
                          "z": -1 if tag.startswith('rig') and not tag.startswith('rigf') else 4})
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
              "rank's speed, in Python, JavaScript, C++, Rust or Assembly; every character can be a typo, more on "
              "symbols, in harder languages, when hot, overclocked or mining. A typo is a SyntaxError (the compile "
              "fails, he retypes the line) or a logic bug that compiles and misbehaves: wrong target, off-by-one, "
              "infinite loop (he freezes), null reference (it fizzles), sign flip (it heals the enemy); C++ and "
              "Assembly also crash him, C++ leaks memory. He reviews before compiling and catches each typo at his "
              "rank's eye; Rust's compiler catches most logic bugs, but compiles slowly; Assembly hits hardest and "
              "breaks easiest. A compiled function joins his program (5 slots) and runs itself whenever its trigger "
              "holds. 24 functions: ping, shield, heal, scan, spray, blink, slow, cache, chain, firewall, ddos, "
              "cleanse, boost, fork (drones), swap (an ally in trouble trades places with the diver), sort (lines "
              "enemies up by health, marks the weakest), encrypt, ddos_all, kill9 (executes), rollback (an enemy back "
              "to where he was 3 s ago, an ally's health back), recurse, inject (stuns), gc, deploy. Every rank can "
              "write every function; the rank decides how fast, how clean, and how well he judges what he can pull "
              "off (low ranks overreach). Overclock: +0.9 GHz and +20% typing, double heat. His rig: CPU (load; "
              "throttled above 85 C), RAM (lasting effects hold it; over the top is Out of memory: the newest is "
              "killed), storage (a saved function reloads, bugs and all) and heat: at 100 C he blue-screens, stunned, "
              "and every unsaved function is lost. He mines Bitcoin (a trickle; more with the miner on, which loads "
              "and heats his CPU and slows his typing), and earns it from functions that land, kills (25) and assists "
              "(10). He buys parts with it: RAM 32/64 GB (60/140), storage 16/32 slots (40/100), SSD/NVMe (50/120: "
              "faster reloads and compiles), air/liquid cooling (50/130), CPU 3.6/4.2 GHz (80/180). Installing away "
              "from base stuns him 2 s. Good judgement buys what his problems call for; poor judgement buys anything, "
              "anywhere. Ranks: Script Kiddie, Intern, Junior, Developer, Senior, Staff, Architect, Root (Top 10; #1 "
              "Zero-Day). From Senior up a rig floats around him: monitors, then a circuit floor, code rain and a crown; "
              "from Architect up his effects grow."),
    "skill2": ("debug: every 10 s while he isn't typing he reads his program over and fixes each bug he notices "
               "(his review eye +10%)."),
    "ult": ("Activate AI (every 40 s, for 12 s): an AI writes his functions for him. Claude Max 20x: the fewest "
            "syntax errors, best on long code, the smallest usage pool. ChatGPT Pro: the fewest logic bugs and the "
            "best pick of what to write, but thinks first. Gemini AI Ultra: three functions a prompt and a perfect "
            "read of the fight, but more hallucinated syntax; the biggest pool. A pool under 60% gives the lite model "
            "(Claude Haiku, GPT mini, Gemini Flash): faster, buggier. His prompts set how buggy any model is, and he "
            "still reviews its code himself; good judgement switches to a provider whose flagship is up (1 s)."),
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
