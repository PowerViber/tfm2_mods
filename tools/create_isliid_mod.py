"""Generate Emperor Isliid's game data from the seven-sword design."""
from __future__ import annotations

import json
from datetime import date
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MOD = ROOT / "mods" / "tfm2_isliid"
(MOD / "champion").mkdir(parents=True, exist_ok=True)
(MOD / "text").mkdir(exist_ok=True)

MOD_ID = "tfm2_isliid_emperor"
NATIVE = "tfm2_custom_ai"
SWORDS = ["skylight", "terra", "darkbringer", "gale", "blood", "rift", "emperor"]

def write(path: Path, value: object) -> None:
    path.write_text(json.dumps(value, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")

def action(name: str, duration: int, cooldown: int, range_: int, casting: str,
           target: str, effect: dict, start: int = 1, move: bool = True) -> dict:
    return {
        "action_name": name,
        "cancelable": name == "attack",
        "growth_range": 0,
        "can_use_with_move": move,
        "duration": duration,
        "cooltime": cooldown,
        "start_timing": start,
        "range": range_,
        "casting_type": casting,
        "casting_target": target,
        "attack_type": "BaseAttack" if name == "attack" else "Skill",
        "effect": effect,
        **({"description": f"#asset/base/text/champion?description.{MOD_ID}.{name if name != 'skill1' else 'skill'}"}
           if name != "attack" else {}),
    }

def native(name: str) -> dict:
    return {"type": "Native", "effect_ref": f"{NATIVE}:isliid_{name}"}

champion = {
    "id": MOD_ID,
    "category": "Melee",
    "tags": ["AD", "Melee"],
    "sprite": f"asset/tfm2_isliid/champions/{MOD_ID}",
    "anim_prefix": "",
    "stat": {"attack": 104, "magic_power": 0, "hp": 920, "defence": 30,
             "magic_resistance": 27, "move_speed": 1030, "hp_regen": 0, "stack": 0, "crit_chance": 0},
    "growth": {"attack": 25, "magic_power": 0, "hp": 86, "defence": 7,
               "magic_resistance": 5, "move_speed": 10, "hp_regen": 0, "stack": 0, "crit_chance": 0},
    "attack": action("attack", 22, 55, 65_000, "Targeting", "Enemy",
                     {"type": "Attack", "damage": 0, "attack_ratio": 100}, 10, False),
    "skill": action("skill1", 11, 18, 110_000, "Position", "EnemyWithoutTower", native("guidance"), 5),
    "skill2": action("skill2", 11, 16, 110_000, "Position", "EnemyWithoutTower", native("recall"), 5),
    "ult": action("ult", 24, 1800, 105_000, "Targeting", "EnemyChampion", native("manifest"), 12),
    "passive": {"passive_ref": f"{NATIVE}:isliid", "params": {}},
    "view_effects": [],
    "view_projectiles": [],
    "view_buffs": [],
}

for sword in SWORDS:
    for tier in range(4):
        tag = f"{sword}_tier{tier}"
        champion["view_effects"].append({"type": "Animation", "name": f"{MOD_ID}_{tag}",
                                          "anim": "asset/tfm2_isliid/vfx/swords", "tag": tag,
                                          "z": 3, "is_follow": False})
        champion["view_buffs"].append({"type": "Animated", "name": f"il_ar_{tag}",
                                        "anim": "asset/tfm2_isliid/vfx/orbit", "tag": f"ar_{tag}", "z": 2})
    champion["view_buffs"].append({"type": "Animated", "name": f"il_selected_{sword}",
                                    "anim": "asset/tfm2_isliid/vfx/selector", "tag": sword, "z": 3})
for tag in ("scar", "seal"):
    champion["view_effects"].append({"type": "Animation", "name": f"{MOD_ID}_{tag}",
                                      "anim": "asset/tfm2_isliid/vfx/engraving", "tag": tag,
                                      "z": 2, "is_follow": False})
for rank in range(7):
    champion["view_buffs"].append({"type": "Animated", "name": f"il_rank{rank}",
                                    "anim": "asset/tfm2_isliid/vfx/badges", "tag": f"rank{rank}", "z": 4})
for number in range(1, 11):
    champion["view_buffs"].append({"type": "Animated", "name": f"il_imperial{number}",
                                    "anim": "asset/tfm2_isliid/vfx/badges", "tag": f"imperial{number}", "z": 4})
write(MOD / "champion" / f"{MOD_ID}.data_champion", champion)

write(MOD / "mod.mod_info", {
    "name": "Emperor Isliid",
    "author": "PowerViber",
    "version": "0.1.0",
    "description": "Seven-sword floating swordsman with individual weapon effects, engraving formations and mastery badges. Requires the TFM2 native helper mod.",
    "last_updated": "2026-10-05",
    "dependencies": [{"mod_id": "base", "version": ">=0.4.14"},
                     {"mod_id": "tfm2_custom_ai", "version": ">=0.9.1"}],
})
write(MOD / "mod.override_info", {
    "asset/base/text/champion": {"remapping": "asset/tfm2_isliid/text/champion", "type": "merge"}
})

description = {
    "name": "Emperor Isliid",
    "skill": "Imperial Guidance: click near Isliid in one of seven directions to choose a sword directly (NW Skylight, N Terra, NE Darkbringer, W Gale, E Blood, SW Rift, SE Emperor). Click ground to send the selected sword there and carve a line. Click an embedded sword, then a destination, to redraw it. Mastery improves drawing speed and precision.",
    "skill2": "Imperial Recall: click near one embedded sword to return only that sword. It cuts enemies along the return path and becomes available again. The formation may break.",
    "ult": "Imperial Manifestation: activate up to three nonidentical valid sword formations. Triangles deal concentrated damage and reduce physical defence; a two-sword line also attacks. Emperor amplifies any formation it joins. Shared vertices can serve more than one formation.",
    "passive": "Imperial Arsenal: the seven swords are Skylight, Terra, Darkbringer, Gale, Blood, Rift and Emperor. Melee basic attacks swing the selected sword and keep it available; ranged basic attacks throw that exact sword and plant it at the impact. Each sword has its own effect. Bearer through Sovereign display seven distinct, numberless mastery badges; Imperial displays its subdivision number."
}
write(MOD / "text" / "champion.i18n", {"en": {"description": {MOD_ID: description}}})
print(f"Generated {MOD_ID}: 7 swords, 28 sword visual variants, 17 mastery badges")
