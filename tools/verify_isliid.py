"""Check consolidated Isliid references, badge art and the installed native build."""
from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path
from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
MOD = ROOT / "mods" / "tfm2_custom"
GAME = Path(r"C:\Program Files (x86)\Steam\steamapps\common\Teamfight Manager2")
INSTALLED = GAME / "mods" / "tfm2_custom"
CHAMP = "tfm2_isliid_emperor"
LOCAL_ONLY = "--local" in sys.argv

data = json.loads((MOD / "champion" / f"{CHAMP}.data_champion").read_text(encoding="utf-8"))
assert data["category"] in {"Melee", "Range", "Magician", "Util", "Assassin"}
assert data["passive"]["passive_ref"] == "tfm2_custom_ai:isliid"
assert data["skill"]["effect"]["effect_ref"] == "tfm2_custom_ai:isliid_guidance"
assert data["skill2"]["effect"]["effect_ref"] == "tfm2_custom_ai:isliid_recall"
assert data["ult"]["effect"]["effect_ref"] == "tfm2_custom_ai:isliid_manifest"
assert data["skill"]["casting_type"] == data["skill2"]["casting_type"] == "Position"

for local in (MOD,) if LOCAL_ONLY else (MOD, INSTALLED):
    assert (local / "champion" / f"{CHAMP}.data_champion").read_bytes() == (MOD / "champion" / f"{CHAMP}.data_champion").read_bytes()
    body = Image.open(local / "champions" / f"{CHAMP}#sheet.png")
    assert body.size == (288, 448), (local, body.size)
    motions = json.loads((local / "champions" / f"{CHAMP}#anim.fanim").read_text(encoding="utf-8"))["anims"]
    assert all((frame["data"]["w"], frame["data"]["h"]) == (48, 56)
               for anim in motions.values() for frame in anim["frames"]), local
    for item in data["view_buffs"] + data["view_effects"] + data["view_projectiles"]:
        assert item["anim"].startswith("asset/tfm2_custom/")
        base = local / item["anim"].removeprefix("asset/tfm2_custom/")
        anim = json.loads(Path(str(base) + "#anim.fanim").read_text(encoding="utf-8"))["anims"]
        assert item["tag"] in anim, (local, item["name"], item["tag"])
        sheet = Image.open(str(base) + "#sheet.png")
        for f in anim[item["tag"]]["frames"]:
            r = f["data"]
            assert r["x"] + r["w"] <= sheet.width and r["y"] + r["h"] <= sheet.height
    # round 89: the engraving strokes come in four mastery tiers (engrave_t0..t3), plain and lit (flare)
    for tier in range(4):
        trail_anims = json.loads((local / "vfx" / f"engrave_t{tier}#anim.fanim").read_text(encoding="utf-8"))["anims"]
        trail_sheet = Image.open(local / "vfx" / f"engrave_t{tier}#sheet.png").convert("RGBA")
        effects = {effect["tag"] for effect in data["view_effects"]}
        for sword in range(7):
            for angle in range(16):
                for kind in ("scar", "flare", "scar_dim"):
                    tag = f"{kind}_{sword}_t{tier}_a{angle}"
                    assert tag in trail_anims and tag in effects, tag
        def trail_bounds(angle: int):
            r = trail_anims[f"scar_0_t{tier}_a{angle}"]["frames"][0]["data"]
            return trail_sheet.crop((r["x"], r["y"], r["x"] + r["w"], r["y"] + r["h"])).getchannel("A").getbbox()
        horizontal, vertical, diagonal = (trail_bounds(angle) for angle in (0, 8, 4))
        assert horizontal[2] - horizontal[0] > horizontal[3] - horizontal[1]
        assert vertical[3] - vertical[1] > vertical[2] - vertical[0]
        assert diagonal[2] - diagonal[0] > 15 and diagonal[3] - diagonal[1] > 15

def animation_pixels(sheet, frames):
    return [sheet.crop((r["x"], r["y"], r["x"] + r["w"], r["y"] + r["h"]))
            for frame in frames for r in [frame["data"]]]

manifest = json.loads((ROOT / "editor" / "isliid-art-manifest.json").read_text(encoding="utf-8"))
names = ("skylight", "terra", "darkbringer", "gale", "blood", "rift", "emperor")
for family, source in (("fly", "swords_fly8"), ("swords", "swords8"), ("orbit", "orbit8"), ("auras", "auras8"),
                       ("fields", "aura_fields8"), ("badges", "badges8"), ("logos", "logos"),
                       ("engrave_t0", "engrave_t0"), ("engrave_t1", "engrave_t1"), ("engrave_t2", "engrave_t2"),
                       ("engrave_t3", "engrave_t3")):
    sheet = Image.open(MOD / "vfx" / f"{source}#sheet.png").convert("RGBA")
    anims = json.loads((MOD / "vfx" / f"{source}#anim.fanim").read_text(encoding="utf-8"))["anims"]
    listed = manifest["engrave"][family.removeprefix("engrave_")] if family.startswith("engrave_") else manifest[family]
    assert listed == anims, family
    for tag, anim in anims.items():
        frames = animation_pixels(sheet, anim["frames"])
        assert all(frame.getbbox() for frame in frames), (family, tag)
        looping = family in ("orbit", "auras", "badges") or (family == "fields" and "_frame" not in tag and "_pair" not in tag) or \
            (family == "swords" and tag.endswith(("_planted", "_ready")))
        if looping:
            # round 89: grounded swords loop over 12 frames, badges over 16, the rest over 8; every frame different
            n = 12 if family == "swords" else 16 if family == "badges" else 8
            assert len(frames) == n and len({hashlib.sha256(f.tobytes()).digest() for f in frames}) == n, (family, tag)
        if "_frame" in tag or family in ("fly", "logos"):
            assert len(frames) == 1, (family, tag)
    if family == "fly":
        # the engine turns projectile art to its heading: every flying sword points right (wider than tall)
        assert all(f"{s}_rank{r}_{st}_f{k}" in anims for s in names for r in range(8) for st in ("flight", "drawing") for k in range(4))
        for tag, anim in anims.items():
            r = anim["frames"][0]["data"]
            assert r["w"] > r["h"], tag
    if family == "swords":
        # round 91: grounded swords are emitted as 2-frame pairs of their 12-frame loop
        assert all(f"{s}_rank{r}_{st}_pair{k}" in anims and len(anims[f"{s}_rank{r}_{st}_pair{k}"]["frames"]) == 2
                   for s in names for r in range(8) for st in ("planted", "ready") for k in range(6))
        # round 89: swords grow with mastery
        height = lambda r: anims[f"emperor_rank{r}_planted"]["frames"][0]["data"]["h"]
        assert all(height(r) > height(r - 1) for r in range(1, 8)) and height(7) >= 1.5 * height(0)
        assert all(f"{s}_{fx}" in anims for s in names for fx in ("impact", "launch", "recall", "hit"))
    if family == "orbit":
        assert all(f"ar_{s}_rank{r}{sel}" in anims for s in names for r in range(8) for sel in ("", "_sel"))
    if family == "badges":
        for tag, anim in anims.items():
            for frame in animation_pixels(sheet, anim["frames"]):
                bbox = frame.getbbox()
                # round 89: the sigil sits right of his head (he is the frame's centre, x 36)
                assert frame.size == (72, 96) and bbox[0] >= 34 and bbox[3] <= 58, f"{tag}: badges stay upper right"
        base = [animation_pixels(sheet, anims[f"rank{i}"]["frames"][:1])[0] for i in range(7)]
        silhouettes = [hashlib.sha256(frame.getchannel("A").point(lambda a: 255 if a else 0).tobytes()).digest() for frame in base]
        assert len(set(silhouettes)) == 7, "every rank has its own silhouette"
        for i in range(1, 11):
            images = animation_pixels(sheet, anims[f"imperial{i}"]["frames"])
            assert len({frame.crop((49, 48, 62, 55)).tobytes() for frame in images}) == 1, "the Imperial number stays steady"
    if family == "logos":
        assert all(anim["frames"][0]["data"]["w"] == (32 if "_complete_f" in tag else 24) for tag, anim in anims.items())
    if family == "auras":
        assert all(f"aura_{k}_rank{r}_{side}" in anims for k in range(7) for r in range(8) for side in ("ally", "enemy"))
    if family == "fields":
        refs = {effect["tag"] for effect in data["view_effects"]}
        assert all(f"aura_field_{k}_rank{r}_frame{p}" in refs for k in range(7) for r in range(8) for p in range(8))
        # round 93: the native code emits the fields as 2-frame pairs every 12 ticks
        assert all(f"aura_field_{k}_rank{r}_pair{p}" in refs and len(anims[f"aura_field_{k}_rank{r}_pair{p}"]["frames"]) == 2
                   and anims[f"aura_field_{k}_rank{r}_pair{p}"]["frames"] == anims[f"aura_field_{k}_rank{r}"]["frames"][2 * p:2 * p + 2]
                   for k in range(7) for r in range(8) for p in range(4))
effect_tags = {e["tag"] for e in data["view_effects"]}
families = [f for f in ("damage", "bind", "pull", "push", "speed", "shred", "weaken", "guard", "attack", "burst", "cooldown", "heal", "domain")]
assert all(f"fire_{f}_t{t}_r{r}" in effect_tags and f"hitmark_{f}_t{t}" in effect_tags
           for f in families for t in range(4) for r in range(2)), "every family fires at every tier"
assert all(f"shatter_t{t}" in effect_tags for t in range(4)) and "crown_flash" in effect_tags
assert all(e["is_follow"] for e in data["view_effects"] if e["tag"].startswith("hitmark_"))
projectiles = {p["name"]: p for p in data["view_projectiles"]}
assert all(p["type"] == "Animated" and p["repeat"] and p["anim"] == "asset/tfm2_custom/vfx/swords_fly8" for p in projectiles.values())
assert len(projectiles) == len(manifest["fly"])
assert not any(e["name"].startswith(tuple(f"{CHAMP}_{n}_" for n in names)) and
               ("_flight" in e["name"] or "_drawing" in e["name"] or e["name"].endswith("_orbit")) for e in data["view_effects"]), \
    "flying swords are projectiles now, never effects"
assert all(f"il_rank{i}" in [b["name"] for b in data["view_buffs"]] for i in range(7))
assert all(f"il_imperial{i}" in [b["name"] for b in data["view_buffs"]] for i in range(1, 11))
buffs={b["name"] for b in data["view_buffs"]}
assert all(f"il_aura_base_rank{rank}_{side}" in buffs for rank in range(8) for side in ("ally","enemy"))
assert all(f"il_aura_visual_{sword}_rank{rank}_{side}" in buffs
           for sword in range(7) for rank in range(8) for side in ("ally","enemy"))

# round 88: the Engraving lab must run on the native tables (constants, grades, pattern names and order)
import re
import subprocess
rust = (ROOT / "native" / "tfm2_custom_ai" / "src" / "isliid.rs").read_text(encoding="utf-8")
def table(name):
    m = re.search(rf"const {name}: \[[^\]]+\] = \[([^\]]+)\];", rust)
    assert m, name
    return [float(v.replace("_", "")) for v in m.group(1).split(",") if v.strip()]
native = {k: table(k) for k in ("SPEED", "THINK_TICKS", "LOOK_AHEAD", "PATTERN_BUDGET", "WOBBLE", "ESCORTS", "REASSESS",
                                "IDLE_RETURN", "STRIKE_GAP", "SOLO_QUALITY")}
grades = [(n, float(a), int(m)) for n, a, m in re.findall(r'\("(\w+)", ([\d.]+), (\d+)\)', re.search(r"const GRADES[^=]+= \[(.*?)\];", rust, re.S).group(1))]
threat = float(re.search(r"const THREAT_R: i64 = ([\d_]+);", rust).group(1).replace("_", ""))
pattern_names = re.findall(r'Pattern\{name:"([^"]+)"', rust)
lab = json.loads(subprocess.run(["node", "-e", "const l=require(process.argv[1]);console.log(JSON.stringify({N:l.NATIVE,P:Object.values(l.PATTERNS).map(p=>p.name),ok:l.selfTest()}))",
                                 str(ROOT / "editor" / "isliidlab.js")], capture_output=True, text=True, check=True).stdout)
for k, v in native.items():
    assert [float(x) for x in lab["N"][k]] == v, f"lab {k} differs from isliid.rs"
assert [(g[0], float(g[1]), int(g[2])) for g in lab["N"]["GRADES"]] == grades, "lab GRADES differ"
assert float(lab["N"]["THREAT_R"]) == threat
# round 93: the distance falloff of the strike gap; round 94: the airtime speed ramp
for name in ("FULL_R", "FAR_R", "STRIKE_FAR_PCT", "LAUNCH_SPEED", "RAMP_TICKS", "TOP_PCT"):
    m = re.search(rf"const {name}: \w+ = ([\d_]+);", rust)
    assert m and float(lab["N"][name]) == float(m.group(1).replace("_", "")), f"lab {name} differs from isliid.rs"
assert set(lab["P"]) <= set(pattern_names), "every lab formation is a native pattern (logo lookup by name)"
assert lab["ok"], "the lab's grade / wobble vectors differ from the native tests"

built = ROOT / "native" / "tfm2_custom_ai" / "target" / "release" / "tfm2_custom_ai.dll"
if not LOCAL_ONLY:
    for deployed in (ROOT / "mods" / "tfm2_custom_ai" / "tfm2_custom_ai.dll", GAME / "mods" / "tfm2_custom_ai" / "tfm2_custom_ai.dll"):
        assert hashlib.sha256(built.read_bytes()).digest() == hashlib.sha256(deployed.read_bytes()).digest(), deployed
    assert sorted(p.name for p in (GAME / "mods" / "tfm2_custom_ai").glob("*.dll")) == ["tfm2_custom_ai.dll"]
print("Engraving lab tables, grades and aim error match isliid.rs")
print(f"Verified 48x56 Isliid art, 4 tiers of 224 directional strokes and the engraving bursts, {len(data['view_buffs']) + len(data['view_effects']) + len(data['view_projectiles'])} visual references (flying swords as {len(projectiles)} projectiles), looping grounded swords, the arsenal ring, badges, logos, Imperial 1-10" + (" and deployed native DLL" if not LOCAL_ONLY else ""))
