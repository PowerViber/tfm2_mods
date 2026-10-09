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
    # round 106: the IDE windows and status lines of every rank theme, placed over him as point effects (coder.rs show_term)
    for tag in fan('coder_theme'):
        z = 5 if tag.startswith('tw_') else 7
        fx.append({"type": "Animation", "name": ID + '_' + tag, "anim": VFX + 'coder_theme', "tag": tag, "z": z, "is_follow": False})
    point = ('fx_scan', 'fx_spray', 'fx_blink_out', 'fx_blink_in', 'fx_wall_')
    for tag in fan('coder_vfx'):
        if tag.startswith('fx_'):
            fx.append({"type": "Animation", "name": ID + '_' + tag, "anim": VFX + 'coder_vfx', "tag": tag,
                       "z": 2 if tag.startswith('fx_wall_') else 3, "is_follow": not tag.startswith(point)})
        elif tag != 'bit':
            z = {'shield': 3, 'lag': 5, 'oc': 4}.get(tag, 5)
            buffs.append({"type": "Animated", "name": 'cd_' + tag, "anim": VFX + 'coder_vfx', "tag": tag, "z": z})
    # round 108: the new functions' effects and marks, and the power / GPU meters
    for tag in fan('coder_vfx2'):
        if tag.startswith('fx_'):
            fx.append({"type": "Animation", "name": ID + '_' + tag, "anim": VFX + 'coder_vfx2', "tag": tag,
                       "z": 1 if tag == 'fx_trap' else 3, "is_follow": tag in ('fx_lock', 'fx_beam', 'fx_docker')})
        else:
            z = 4 if tag.startswith(('pow', 'gpu')) else -1 if tag in ('cdn', 'gate', 'bloom') else 5
            buffs.append({"type": "Animated", "name": 'cd_' + tag, "anim": VFX + 'coder_vfx2', "tag": tag, "z": z})
    # round 103: the high-rank rig (behind him, in front of him) and the top-rank effects
    for tag in fan('coder_rank'):
        if tag.startswith('fx_'):
            fx.append({"type": "Animation", "name": ID + '_' + tag, "anim": VFX + 'coder_rank', "tag": tag, "z": 3,
                       "is_follow": not tag.startswith(point)})
        else:
            buffs.append({"type": "Animated", "name": 'cd_' + tag, "anim": VFX + 'coder_rank', "tag": tag,
                          "z": 3 if tag.startswith('kb') else 2 if tag.startswith('fit') else -1 if tag.startswith('rig') and not tag.startswith('rigf') else 4})
    proj = [{"type": "Animated", "name": ID + '_bit', "anim": VFX + 'coder_vfx', "tag": 'bit', "z": 2, "repeat": True}]
    return fx, buffs, proj


def champion():
    fx, buffs, proj = views()
    return {
        "id": ID,
        "category": "Magician",
        "tags": ["AP", "CC", "Range"],
        "sprite": "asset/tfm2_custom/champions/" + ID,
        "anim_prefix": "",
        "stat": {"attack": 82, "magic_power": 55, "hp": 1250, "defence": 30, "magic_resistance": 30, "move_speed": 960,
                 "hp_regen": 4, "stack": 0, "crit_chance": 0},
        "growth": {"attack": 6, "magic_power": 24, "hp": 135, "defence": 8, "magic_resistance": 7, "move_speed": 9,
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
    "skill": ("He fights by writing code. He picks a function and types its code (shown over his head in an IDE "
              "window themed by his rank) in one of 13 languages: Python, JavaScript, TypeScript, C++, Rust, Go, Java, "
              "C#, Lua, Haskell, Bash, SQL or Assembly. Each has its feel: Rust and Haskell catch most logic bugs but "
              "compile slowly, Lua and Python are quick and light, Go daemons cost less CPU, Java eats RAM, Bash picks "
              "wrong targets, Assembly hits hardest and breaks easiest. Every character can be a typo: a SyntaxError "
              "(he retypes the line) or a logic bug that compiles and misbehaves (wrong target, off-by-one, infinite "
              "loop, null reference, sign flip, segfault, leak). He reviews before compiling and catches typos at his "
              "rank's eye. 100 functions in two kinds: scripts fire the moment they compile (no slot), daemons join "
              "his program and run whenever their trigger holds; the program has 4 slots for a Script Kiddie up to 10 "
              "for an Architect, 11 at Root, 12 for Zero-Day, so he keeps coding all fight. The functions span "
              "networking, cloud and DevOps (docker, kubernetes, cron, terraform), git (revert, blame, push --force, "
              "stash, rebase, hotfix), security (sql_injection, ransomware, keylogger, honeypot, botnet, zero_day), "
              "GPU and AI (cuda_kernel, ray_tracing, train_model, llm_agent, diffusion), data (sharding, backup, "
              "deadlock, map_reduce, blockchain) and algorithms (dijkstra, binary_search, quicksort, regex). He "
              "rotates what he writes and runs instead of repeating the same function. His rig: CPU (load; throttled "
              "above 85 C), RAM (lasting effects hold it; over the top is Out of memory), storage (a saved function "
              "reloads, bugs and all; a saved script at once) and heat: at 100 C he blue-screens and loses every "
              "unsaved function. He mines Bitcoin and earns it from functions that land, kills and assists, and buys "
              "parts with it, each in five tiers up to a workstation and a data center (2 TB ECC RAM, a SAN, a RAM "
              "disk, an immersion tank, dual EPYC CPUs, an H100 rack); the pros start with theirs. A data center has "
              "risks: drawing too much power trips the breaker (a 3 s outage that loses unsaved daemons), its parts "
              "bill Bitcoin every second (unpaid, he's throttled to consumer speed), and the immersion pump can fail. "
              "Ranks: Script Kiddie, Intern, Junior, Developer, Senior, Staff, Architect, Root (Top 10; #1 Zero-Day), "
              "each with its own editor theme, outfit and keyboard. "
              "Function damage is reduced to 65%, with buffer_overflow, cuda_kernel, overfit and quantum at 52%. "
              "Replicas and reruns deal half their previous damage; support effects keep their strength. "
              "zero_day deals 12% of maximum HP; binary_search removes 25% of remaining HP below 20%; "
              "kill9 executes only below 8% HP (reruns use an ordinary hit). honeypot reflects 20%. "
              "Typing and execution speed are unchanged."),
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
