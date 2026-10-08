"""Round 103: check the Coder: his data's views against the sheets, every view name coder.rs builds, and that the Code
lab (editor/coderlab.js) runs on the native tables and writes code run for run like coder.rs.

    python3 tools/verify_coder.py [--local]

--local skips the installed game copy and the deployed DLL.
"""
from __future__ import annotations

import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
MOD = ROOT / "mods" / "tfm2_custom"
SRC = ROOT / "native" / "tfm2_custom_ai" / "src"
GAME = Path(r"C:\Program Files (x86)\Steam\steamapps\common\Teamfight Manager2")
CHAMP = "tfm2_custom_coder"
LOCAL_ONLY = "--local" in sys.argv or not GAME.exists()

data = json.loads((MOD / "champion" / f"{CHAMP}.data_champion").read_text(encoding="utf-8"))
assert data["passive"]["passive_ref"] == "tfm2_custom_ai:coder"
# round 104: anything else and the game skips him ("data_champion load error: unknown variant ...")
assert data["category"] in {"Melee", "Range", "Magician", "Util", "Assassin"}, data["category"]
assert set(data["tags"]) <= {"AD", "AP", "Heal", "Shield", "Dot", "CC", "Range", "Melee", "Tank", "Magic"}, data["tags"]
views = data["view_buffs"] + data["view_effects"] + data["view_projectiles"]
names = [v["name"] for v in views]
assert len(names) == len(set(names)), "duplicate view names"

# every view's tag is in its sheet's fanim, inside the sheet; every sheet at most 2048 x 2048
sheets = {}
for v in views:
    base = MOD / v["anim"].removeprefix("asset/tfm2_custom/")
    if base not in sheets:
        im = Image.open(str(base) + "#sheet.png")
        assert im.width <= 2048 and im.height <= 2048, (base, im.size)
        sheets[base] = (im.size, json.loads(Path(str(base) + "#anim.fanim").read_text(encoding="utf-8"))["anims"])
    (w, h), anims = sheets[base]
    assert v["tag"] in anims, (v["name"], v["tag"])
    for f in anims[v["tag"]]["frames"]:
        r = f["data"]
        assert r["x"] + r["w"] <= w and r["y"] + r["h"] <= h, (v["name"], r)
    # round 105: the game plays an effect's animation to its end whatever life the native code gives it, so a line or
    # status line re-placed every 6 ticks must last about that long (6 s frames left a trail of every copy)
    if v["tag"].startswith(("ln_", "ov_", "tw_")):
        total = sum(f["duration"] for f in anims[v["tag"]]["frames"])
        assert total <= 0.2, (v["name"], total)

# every name coder.rs builds
rust = (SRC / "coder.rs").read_text(encoding="utf-8")
code = (SRC / "coder_code.rs").read_text(encoding="utf-8")
P = CHAMP + "_"
fx = {v["name"] for v in data["view_effects"]}
buffs = {v["name"] for v in data["view_buffs"]}
STEPS = int(re.search(r"const STEPS: usize = (\d+);", rust).group(1))
THEMES = int(re.search(r"pub const THEMES: usize = (\d+);", rust).group(1))
langs = re.search(r'pub const LANGS: \[&str; 13\] = \[(.*?)\];', code).group(1).replace('"', "").replace(" ", "").split(",")
funcs = re.findall(r'\("(\w+)", (\d), \[(.*?)\]\),\n', code)
assert len(funcs) == 100, len(funcs)   # round 107: 100 functions
for name, _, per in funcs:
    # round 107: each function is written in a subset of the 13 languages (empty slice elsewhere)
    for lang, lines in zip(langs, re.findall(r"&\[(.*?)\]", per)):
        n = len(re.findall(r"\(\d+, \d+\)", lines))
        for line in range(n):
            for step in range(1, STEPS + 1):
                assert f"{P}ln_{lang}_{name}_{line}_{step}" in fx, (name, lang, line, step)
# round 106: every rank theme has its IDE window per language; round 107: one generic run line per theme
for th in range(THEMES):
    for lang in langs:
        assert f"{P}tw_t{th}_{lang}" in fx, (th, lang)
    assert f"{P}ov_run_t{th}" in fx, ("ov_run", th)
for k in range(9):
    assert f"cd_fit{k}" in buffs and f"cd_kb{k}" in buffs, k   # round 106: the rank outfits and keyboards
hi = re.search(r"const HI_FX: \[&str; \d+\] = \[(.*?)\];", rust).group(1).replace('"', "").replace(" ", "").split(",")
for tag in hi:
    assert P + tag in fx and P + tag + "_hi" in fx, tag
for k in range(1, 6):
    assert f"cd_rig{k}" in buffs and f"cd_rigf{k}" in buffs, k
for r in range(7):
    assert f"cd_rank{r}" in buffs
for p in range(1, 11):
    assert f"cd_root{p}" in buffs
for lit in set(re.findall(r'"(cd_[a-z_]+\d*)"', rust)):
    # a whole name, or a prefix the code numbers (cd_heat + 0..10)
    assert lit in buffs or any(re.fullmatch(re.escape(lit) + r"\d+", b) for b in buffs), lit
for lit in set(re.findall(r'"((?:ov|fx)_[a-z0-9_]+)"', rust)):
    if lit.endswith("_"):
        continue
    if lit.startswith("ov_") and lit != "ov_bsod":   # status lines come in every theme
        for th in range(THEMES):
            assert f"{P}{lit}_t{th}" in fx, (lit, th)
    else:
        assert P + lit in fx, lit

# the Code lab: the native tables, the code lengths, and the same runs
lab = json.loads(subprocess.run(
    ["node", "-e", "const l=require(process.argv[1]);const c=require(process.argv[2]);"
     "console.log(JSON.stringify({N:l.NATIVE,ok:l.selfTest(),V:l.vectors(),lens:c.FUNCS.map(f=>[f.name,f.tier,c.LANGS.map(g=>f.lens[g]||[])])}))",
     str(ROOT / "editor" / "coderlab.js"), str(ROOT / "editor" / "coder-code.js")],
    capture_output=True, text=True, check=True).stdout)
N = lab["N"]


def table(name):
    m = re.search(rf"const {name}: \[[^\]]+\] = \[([^\]]+)\];", rust)
    assert m, name
    return [int(v.replace("_", "")) for v in m.group(1).split(",") if v.strip()]


def scalar(name):
    m = re.search(rf"const {name}: \w+ = ([\d_]+);", rust)
    assert m, name
    return int(m.group(1).replace("_", ""))


for name in ("CPS100", "TYPO", "NOTICE", "AWARE", "CLOCK", "READ", "IQ", "OC_OFF", "OC_LAG", "PROMPT"):
    assert N[name] == table(name), f"lab {name} differs from coder.rs"
    assert N[name + "_TOP"] == scalar(name + "_TOP"), f"lab {name}_TOP differs from coder.rs"
for name in ("RAM_MB", "STORAGE", "RELOAD", "COMPILE_PCT", "COOLING", "GHZ", "POOL", "REFILL"):
    assert N[name] == table(name), f"lab {name} differs from coder.rs"
for name in ("AI_TICKS", "AI_COOLDOWN", "BSOD_SHY", "HOT_SKIP", "HI_RANK", "ROOT"):
    assert N[name] == scalar(name), f"lab {name} differs from coder.rs"
# round 107: program slots by rank are a table (slots()); the GPU is a sixth part
assert N["SLOTS_RANK"] == [int(v) for v in re.search(r"const S: \[usize; 8\] = \[(.*?)\];", rust).group(1).split(",")]
assert N["GPU_POWER"] == table("GPU_POWER") and N["NPARTS"] == scalar("NPARTS")
assert N["RANK_NAMES"] == re.findall(r'"([^"]+)"', re.search(r"pub const RANK_NAMES[^=]+= \[(.*?)\];", rust).group(1))
lang = re.findall(r"Lang \{ typo: (\d+), syntax: (\d+), compile: (\d+), power: (\d+), heat: (\d+), ram: (\d+), catch: (\d+), bugs: \[([\d, ]+)\] \}", rust)
assert [[int(x) for x in l[:7]] + [[int(b) for b in l[7].split(",")]] for l in lang] == \
       [[x["typo"], x["syntax"], x["compile"], x["power"], x["heat"], x["ram"], x["catch"], x["bugs"]] for x in N["LANG"]], "lab LANG differs"
spec = re.findall(r"\((\d+), (\d+), (\d+), (\d+), (\d+)\),\s+// ", re.search(r"const SPEC[^=]+= \[(.*?)\n\];", rust, re.S).group(1))
assert [[int(x) for x in s] for s in spec] == N["SPEC"], "lab SPEC differs"
# round 107: IDEAL and KIND are generated into coder_code.rs (from coder_functions.py)
gen_ideal = [int(x) for x in re.search(r"pub const IDEAL: \[usize; 100\] = \[(.*?)\];", code).group(1).split(",")]
assert gen_ideal == N["IDEAL"], "lab IDEAL differs from coder_code.rs"
gen_kind = [x.strip() == "true" for x in re.search(r"pub const KIND: \[bool; 100\] = \[(.*?)\];", code).group(1).split(",")]
assert gen_kind == N["KIND"], "lab KIND differs from coder_code.rs"
assert re.search(r"if rank >= 6 && avail\(f, ASM\) && matches!\(f, CHAIN \| DDOS \| RECURSE\) \{ ASM \}", rust), "ideal()'s Assembly list changed: update the lab"
models = re.findall(r"Model \{ cps100: (\d+), syntax: (\d+), logic: (\d+), think: (\d+), per_prompt: (\d+) \}", rust)
assert [[int(x) for x in m] for m in models] == [[m["cps100"], m["syntax"], m["logic"], m["think"], m["per_prompt"]] for p in N["MODELS"] for m in p]
prices = re.search(r"pub const PRICE: \[\[usize; 2\]; NPARTS\] = \[(.*?)\];", rust).group(1)
assert [[int(a), int(b)] for a, b in re.findall(r"\[(\d+), (\d+)\]", prices)] == N["PRICE"]
for (name, tier, per), (lname, ltier, lper) in zip(funcs, lab["lens"]):
    assert name == lname and int(tier) == ltier, name
    for lines, ll in zip(re.findall(r"&\[(.*?)\]", per), lper):
        assert [[int(a), int(b)] for a, b in re.findall(r"\((\d+), (\d+)\)", lines)] == ll, name
vectors = (SRC / "coder_vectors.txt").read_text(encoding="utf-8").split("\n")[:-1]
assert vectors == lab["V"], f"the lab's runs differ from coder.rs ({sum(a != b for a, b in zip(vectors, lab['V']))} of {len(vectors)})"
assert lab["ok"], "the lab's self-test failed"

if not LOCAL_ONLY:
    built = ROOT / "native" / "tfm2_custom_ai" / "target" / "x86_64-pc-windows-gnu" / "release" / "tfm2_custom_ai.dll"
    for deployed in (ROOT / "mods" / "tfm2_custom_ai" / "tfm2_custom_ai.dll", GAME / "mods" / "tfm2_custom_ai" / "tfm2_custom_ai.dll"):
        assert hashlib.sha256(built.read_bytes()).digest() == hashlib.sha256(deployed.read_bytes()).digest(), deployed

print(f"Code lab: tables, languages, functions, models and hardware match coder.rs; {len(vectors)} runs reproduced exactly")
print(f"Verified {len(views)} Coder views over {len(sheets)} sheets (each <= 2048), the rig layers, the top-rank effects, "
      f"crests and every name coder.rs builds" + ("" if LOCAL_ONLY else ", and the deployed DLL"))
