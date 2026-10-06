"""Author original, native-pixel eight-frame Isliid combat art.

The consolidated tfm2_custom mod is the source of truth. No reference-image
pixels enter these sheets.
"""
from __future__ import annotations

import json
import math
import re
import shutil
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parents[1]
MOD = ROOT / "mods" / "tfm2_custom"
EDITOR = ROOT / "editor"
SWORDS = ("skylight", "terra", "darkbringer", "gale", "blood", "rift", "emperor")
INK = "#0a1024"
METAL = "#e7edf4"
COLORS = (
    ("#285b92", "#8eeaff", "#ffffff"),
    ("#875129", "#ffc775", "#fff0b1"),
    ("#523681", "#b982ff", "#f1d9ff"),
    ("#087e8b", "#6af4ea", "#e8ffff"),
    ("#8c224c", "#ff669c", "#ffcfdf"),
    ("#255293", "#7db4ff", "#d9e8ff"),
    ("#9e6a1d", "#ffde77", "#fff8ce"),
)
STATES = ("orbit", "flight", "planted", "drawing", "ready")
PHASES = ("planned", "drawing", "complete", "cancelled")
EFFECTS = ("DAMAGE", "BIND", "PULL", "PUSH", "SPEED", "SHRED", "WEAKEN",
           "GUARD", "ATTACK", "BURST", "COOLDOWN", "HEAL", "DOMAIN")
SOLO_EFFECTS = ("REVEAL", "SLOW", "DEFENCE DOWN", "SPEED UP", "LEECH", "DISPLACE", "AMPLIFY")


def pack(animations: dict[str, list[Image.Image]], width: int, height: int,
         columns: int, duration: float) -> tuple[Image.Image, dict]:
    total = sum(map(len, animations.values()))
    sheet = Image.new("RGBA", (columns * width, math.ceil(total / columns) * height))
    metadata = {}
    index = 0
    for tag, frames in animations.items():
        entries = []
        for frame in frames:
            x, y = (index % columns) * width, (index // columns) * height
            sheet.alpha_composite(frame, (x, y))
            entries.append({"duration": duration, "data": {"x": x, "y": y, "w": width, "h": height}})
            index += 1
        metadata[tag] = {"frames": entries}
    return sheet, {"anims": metadata}


def save(name: str, animations: dict[str, list[Image.Image]], size: tuple[int, int],
         columns: int, duration: float) -> dict:
    sheet, data = pack(animations, *size, columns, duration)
    target = MOD / "vfx" / name
    sheet.save(str(target) + "#sheet.png", optimize=True)
    (MOD / "vfx" / f"{name}#anim.fanim").write_text(json.dumps(data, separators=(",", ":")), encoding="utf-8")
    shutil.copyfile(str(target) + "#sheet.png", EDITOR / f"isliid-{name}-8.png")
    return data["anims"]


def sword_frame(kind: int, rank: int, phase: int, state: str) -> Image.Image:
    im = Image.new("RGBA", (32, 64))
    d = ImageDraw.Draw(im)
    dark, color, bright = COLORS[kind]
    cx = 16
    widen = rank // 2
    # Seven independent silhouettes: needle, slab, serration, sweep, hook,
    # split edge, and imperial command blade.
    if kind == 0:
        blade = [(16, 4), (18+widen//2, 12), (18+widen//2, 45), (16, 53),
                 (14-widen//2, 45), (14-widen//2, 12)]
    elif kind == 1:
        blade = [(12-widen, 11), (20+widen, 11), (22+widen, 42), (16, 54), (10-widen, 42)]
    elif kind == 2:
        blade = [(15, 7), (23+widen, 16), (19+widen, 27), (24+widen, 33),
                 (17, 55), (12-widen, 34), (9-widen, 30), (13, 20)]
    elif kind == 3:
        blade = [(13, 7), (22+widen, 13), (20+widen, 31), (24+widen, 37),
                 (14, 54), (15, 33), (10-widen, 21)]
    elif kind == 4:
        blade = [(11-widen, 12), (20, 9), (24+widen, 16), (20, 24),
                 (22+widen, 38), (15, 55), (11, 38), (14, 26)]
    elif kind == 5:
        blade = [(10-widen, 11), (14, 7), (15, 35), (16, 47), (17, 35),
                 (18, 7), (22+widen, 11), (21, 39), (16, 56), (11, 39)]
    else:
        blade = [(16, 4), (20+widen, 14), (23+widen, 19), (20, 30),
                 (21+widen, 43), (16, 55), (11-widen, 43), (12, 30), (9-widen, 19), (12, 14)]
    d.polygon(blade, fill=dark, outline=INK)
    d.line([(16, 10), (16, 49)], fill=METAL if kind in (0, 5) else color, width=2)
    d.line([(14, 17), (14, 40)], fill=bright if kind in (0, 3) else color)
    guard = 5 + rank // 2
    d.line([(cx-guard, 16), (cx+guard, 16)], fill=INK, width=4)
    d.line([(cx-guard+1, 15), (cx+guard-1, 15)], fill=color, width=2)
    d.rectangle((14, 10, 18, 14), fill=INK)
    d.rectangle((15, 11, 17, 13), fill=bright)
    d.line([(16, 4), (16, 8)], fill=color, width=2)
    if rank >= 1:
        d.line([(cx-guard, 15), (cx-guard-2, 12-rank//3)], fill=color)
        d.line([(cx+guard, 15), (cx+guard+2, 12-rank//3)], fill=color)
    if rank >= 2:
        for y in (23, 30, 37):
            d.point((16+(1 if (y//7)%2 else -1), y), fill=bright)
    if rank >= 3:
        d.polygon([(8-rank//3, 12), (12, 17), (10, 23), (6-rank//3, 20)], fill=dark, outline=color)
        d.polygon([(24+rank//3, 12), (20, 17), (22, 23), (26+rank//3, 20)], fill=dark, outline=color)
    if rank >= 4:
        d.line([(11, 29), (9-rank//4, 34), (12, 40)], fill=bright)
        d.line([(21, 29), (23+rank//4, 34), (20, 40)], fill=bright)
    if rank >= 5:
        d.ellipse((10, 6, 22, 20), outline=color)
        d.point((16, 7), fill=bright)
    if rank >= 6:
        for side in (-1, 1):
            d.polygon([(16+side*9, 22), (16+side*13, 27), (16+side*10, 36)],
                      fill=dark, outline=color)
    if rank == 7:
        d.arc((3, 3, 29, 29), 205, 335, fill=bright, width=2)
        d.arc((3, 3, 29, 29), 25, 155, fill=color, width=2)
    # Hand-placed motion accents: eight actual frames, not duplicated frames.
    glint_y = 20 + (phase * 4) % 29
    d.point((16, glint_y), fill="#ffffff")
    d.point((15, glint_y+1), fill=bright)
    for j in range(1 + rank // 2):
        x = 4 + ((phase*3 + j*7 + kind*5) % 24)
        y = 8 + ((phase*5 + j*13 + kind*3) % 44)
        if abs(x-16) > 5:
            d.point((x,y), fill=color if j%2 else bright)
    if state == "flight":
        d.line([(7-(phase%3), 44), (5-(phase%3), 54)], fill=color)
        d.line([(24+(phase%3), 36), (26+(phase%3), 48)], fill=bright)
    elif state == "drawing":
        d.line([(4, 50-phase%4), (12, 53)], fill=color, width=2)
        d.line([(20, 53), (28, 48+phase%4)], fill=bright)
    elif state == "planted":
        d.arc((7, 49, 25, 61), 0, 180, fill=dark, width=2)
        d.point((8+phase*2, 57), fill=color)
    elif state == "ready":
        d.ellipse((6, 50, 26, 60), outline=color, width=2)
        d.point((8+phase*2, 55), fill=bright)
    return im


def aura_frame(kind: int, rank: int, phase: int, hostile: bool) -> Image.Image:
    im = Image.new("RGBA", (64, 64))
    d = ImageDraw.Draw(im)
    dark, color, bright = COLORS[kind]
    # An ellipse at the feet. Seven unique moving sectors compose one ring.
    angle0 = kind * 360/7 + phase * (1.2 if hostile else -1.2)
    angle1 = angle0 + 360/7 - 5
    box = (8, 36, 56, 56)
    d.arc(box, int(angle0), int(angle1), fill=dark, width=4)
    d.arc(box, int(angle0), int(angle1), fill=color, width=2)
    t = math.radians(angle0 + 8 + (phase * 5) % 28)
    x, y = int(32+24*math.cos(t)), int(46+10*math.sin(t))
    d.point((x,y),fill=bright)
    if hostile:
        d.line([(x,y-4),(x,y)],fill=bright)
    else:
        d.line([(x,y),(x,y+4)],fill=bright)
    motif = kind % 7
    if motif == 0: d.line([(x-2,y),(x+2,y)],fill=bright)
    elif motif == 1: d.polygon([(x,y-3),(x+2,y),(x,y+2),(x-2,y)],outline=bright)
    elif motif == 2: d.line([(x-2,y-2),(x,y),(x+2,y-1)],fill=bright)
    elif motif == 3: d.line([(x-3,y+1),(x,y-1),(x+3,y+1)],fill=bright)
    elif motif == 4: d.ellipse((x-1,y-2,x+1,y+1),fill=bright)
    elif motif == 5: d.line([(x-2,y+1),(x+2,y-1)],fill=bright)
    else: d.polygon([(x,y-3),(x+3,y),(x,y+3),(x-3,y)],outline=bright)
    # One extra outer rune at every rank gives the aura a visible progression.
    for j in range(rank):
        a=math.radians(angle0+10+j*12+phase*2)
        q=(int(32+28*math.cos(a)),int(46+13*math.sin(a)))
        d.point(q,fill=color)
        if j>=3:
            d.point((q[0],q[1]-1),fill=bright)
    if rank>=6:
        d.arc((5,33,59,59),int(angle0+phase*2),int(angle0+18+phase*2),fill=bright)
    if rank==7:
        d.point((x+3,y-3),fill="#ffffff")
    return im


def badge_frame(rank: int, phase: int, imperial: int | None = None) -> Image.Image:
    """Original narrow sword-and-engraving insignia, drawn at 48x96 native pixels."""
    im=Image.new("RGBA",(48,96)); d=ImageDraw.Draw(im)
    gold="#e6bb62"; light="#fff0ae"; blue="#78dcea"; navy="#17233f"
    cx=38
    # Each tier has a different silhouette, independent of hue or a rank digit.
    if rank==0:  # a plain carried blade
        d.polygon([(38,17),(41,22),(40,37),(38,43),(36,37),(35,22)],fill="#48566c",outline=INK)
        d.line((38,20,38,38),fill="#b6c9d2",width=2)
        d.line((33,25,43,25),fill=light,width=2)
    elif rank==1:  # a forged X
        for blade in [[(29,17),(33,19),(44,38),(42,42),(39,38)],
                      [(45,17),(47,20),(35,42),(32,40),(34,35)]]:
            d.polygon(blade,fill="#677c98",outline=INK)
        d.line((31,18,43,39),fill=blue,width=2);d.line((45,18,34,39),fill=light,width=2)
        d.rectangle((35,27,41,32),fill=gold,outline=INK)
    elif rank==2:  # one blade inscribed in a triangle
        d.polygon([(38,12),(47,39),(29,39)],fill=navy,outline=gold)
        d.line((38,16,38,42),fill=blue,width=3)
        d.line((34,26,42,26),fill=light,width=2)
        d.polygon([(38,41),(35,35),(41,35)],fill=light)
    elif rank==3:  # a four-point engraved compass
        d.polygon([(38,11),(41,21),(47,27),(42,32),(38,44),(34,32),(29,27),(35,21)],
                  fill=navy,outline=gold)
        d.polygon([(38,16),(40,26),(38,38),(36,26)],fill=blue,outline=INK)
        d.line((30,27,46,27),fill=light,width=2)
        for x,y in ((38,11),(29,27),(47,27),(38,44)):d.point((x,y),fill=light)
    elif rank==4:  # outward-reaching twin wing blades
        d.polygon([(38,17),(31,12),(28,19),(32,29),(29,38),(37,34),(38,44),
                   (39,34),(47,38),(44,29),(47,19),(42,12)],fill=navy,outline=gold)
        d.line((32,17,36,34),fill=blue,width=2);d.line((44,17,40,34),fill=blue,width=2)
        d.polygon([(38,18),(41,27),(38,38),(35,27)],fill=light)
    elif rank==5:  # split laurel around a blade and gemstone
        d.polygon([(38,11),(43,16),(46,32),(42,43),(38,47),(34,43),(30,32),(33,16)],
                  fill=navy,outline=gold)
        d.line((38,15,38,43),fill=light,width=2)
        for side in (-1,1):
            for j in range(3):
                y=20+j*7;x=38+side*(5+j%2)
                d.line((x,y,x-side*3,y-4),fill=gold,width=2)
        d.polygon([(38,23),(41,27),(38,31),(35,27)],fill=blue,outline=INK)
    elif rank==6:  # seven pointed sovereign star, no numeral
        pts=[]
        for j in range(14):
            a=-math.pi/2+j*math.pi/7;r=17 if j%2==0 else 10
            pts.append((round(cx+math.cos(a)*r*.55),round(28+math.sin(a)*r)))
        d.polygon(pts,fill=navy,outline=gold)
        d.ellipse((33,22,43,34),fill="#294663",outline=light)
        d.polygon([(38,18),(41,28),(38,39),(35,28)],fill=blue,outline=INK)
        for j in range(7):
            a=-math.pi/2+j*math.tau/7
            d.point((round(cx+math.cos(a)*9),round(28+math.sin(a)*14)),fill=light)
    else:  # seven-blade imperial aureole; leaderboard #1 receives the richest art
        top=max(1,min(10,imperial or 10)); prestige=11-top
        d.ellipse((28,13,47,46),fill=navy,outline=gold,width=2)
        for j in range(7):
            a=-math.pi/2+j*math.tau/7
            x=round(cx+math.cos(a)*10);y=round(29+math.sin(a)*17)
            tip=(round(cx+math.cos(a)*13),round(29+math.sin(a)*21))
            d.line((x,y,*tip),fill=light if j%2 else blue,width=2)
            d.point(tip,fill="#ff8fd0" if prestige>=5 else light)
        d.ellipse((32,23,44,36),fill="#27365e",outline=light)
        d.rectangle((34,25,42,33),fill="#183051")
        # This glyph is placed identically in all eight frames.
        font=ImageFont.load_default()
        digits=str(top); box=d.textbbox((0,0),digits,font=font)
        d.text((38-(box[2]-box[0])//2,26),digits,font=font,fill="#fff6c5")
        if prestige>=4:
            d.arc((26,11,49,48),190,350,fill="#d6a4f5",width=2)
        if prestige>=7:
            d.line((28,18,27,12),fill=blue,width=2)
            d.line((47,18,47,9),fill="#ff9dc9",width=2)
        if prestige>=8:
            d.line((29,40,26,45),fill="#9cefff",width=2)
            d.line((45,41,47,47),fill="#f6a8ed",width=2)
            d.point((29,16),fill=light);d.point((46,14),fill=light)
        if prestige>=9:
            d.arc((25,9,47,49),165,195,fill="#b5eeff",width=2)
            d.point((27,28),fill="#fff6c5");d.point((47,40),fill="#fff6c5")
        if prestige>=10:
            d.polygon([(38,6),(40,10),(38,13),(36,10)],fill="#fff1ae",outline="#985ed9")
            d.point((26,34),fill="#fff1ae");d.point((47,32),fill="#fff1ae")
            d.line([(31,15),(25,19),(29,23),(24,27)],fill="#81f1ff",width=2)
            d.line([(44,15),(47,18),(44,24),(47,28)],fill="#ff8fd0",width=2)
            d.line([(31,42),(26,47),(31,47),(35,53)],fill="#a8f6ff",width=2)
            d.line([(44,42),(47,47),(43,47),(40,53)],fill="#ffb9e7",width=2)
            for x,y in ((26,11),(45,7),(25,36),(46,36),(38,53)):
                d.point((x,y),fill="#fff5c8")
    # Animated light travels along the crest; fixed silhouettes and digits stay still.
    trace=[(31,16),(34,13),(38,10),(43,13),(46,18),(45,36),(40,45),(33,41)]
    x,y=trace[phase]
    d.point((x,y),fill="#ffffff" if rank>=5 else blue)
    if rank>=4:d.point((x,y+1),fill=light)
    if rank>=7:
        a=phase*math.tau/8
        d.point((round(38+math.cos(a)*11),round(28+math.sin(a)*22)),fill="#ff8fd0")
    return im


def badge_frames() -> dict[str, list[Image.Image]]:
    result={f"rank{rank}":[badge_frame(rank,p) for p in range(8)] for rank in range(7)}
    result.update({f"imperial{n}":[badge_frame(7,p,n) for p in range(8)] for n in range(1,11)})
    return result


def aura_base_frame(rank: int, phase: int, hostile: bool) -> Image.Image:
    im=Image.new("RGBA",(64,64));d=ImageDraw.Draw(im)
    tone="#806a7e" if hostile else "#66899c"
    d.arc((8,36,56,56),phase*45,phase*45+240,fill=tone,width=1)
    d.arc((10,38,54,54),(phase*45+180)%360,(phase*45+300)%360,
          fill="#a986a0" if hostile else "#9dc3cb",width=1)
    return im


def aura_field_frame(kind: int, rank: int, phase: int) -> Image.Image:
    im=Image.new("RGBA",(96,96));d=ImageDraw.Draw(im)
    dark,color,bright=COLORS[kind]
    # 40-pixel horizontal radius mirrors 40,000 map units at native scale.
    for j in range(4):
        start=(kind*51+phase*11+j*90)%360
        d.arc((8,29,88,67),start,start+46,fill=color if j%2 else dark,width=2)
    d.ellipse((44,45,52,51),outline=bright)
    a=phase*math.tau/8
    x=round(48+40*math.cos(a));y=round(48+19*math.sin(a))
    d.point((x,y),fill=bright)
    if rank>=3:d.point((x,y-2),fill=color)
    if rank>=6:
        d.arc((6,27,90,69),int(phase*45),int(phase*45+18),fill=bright,width=2)
    if rank==7:d.point((48,28),fill="#fff8e5")
    return im


def flag_frame(name: str, effect: str, phase: str) -> Image.Image:
    im=Image.new("RGBA",(120,48))
    d=ImageDraw.Draw(im)
    border={"planned":"#80cbe4","drawing":"#f6d27b","complete":"#fff1b1","cancelled":"#ff6480"}[phase]
    d.polygon([(8,2),(112,2),(112,32),(10,32),(8,35)],fill="#111c33",outline=border)
    d.line([(9,2),(9,46)],fill="#d6bd83",width=2)
    font=ImageFont.load_default()
    title=name.upper()
    if len(title)>18:title=title[:17]+"."
    d.text((13,4),title,font=font,fill="#f5f5eb")
    d.text((13,17),effect,font=font,fill=border)
    if phase=="drawing":d.line([(12,30),(12+((len(name)*7)%93),30)],fill=border,width=2)
    if phase=="complete":d.polygon([(103,23),(106,26),(112,17),(110,16),(106,22)],fill="#8ef7cb")
    if phase=="cancelled":
        d.line([(18,3),(101,31)],fill="#ff3758",width=5)
        d.line([(101,3),(18,31)],fill="#ff3758",width=5)
        d.line([(18,3),(101,31)],fill="#ffe5e9")
        d.line([(101,3),(18,31)],fill="#ffe5e9")
    return im


def main() -> None:
    sword_anims={}
    orbit_anims={}
    positions=((24,20),(49,8),(82,20),(8,55),(92,55),(28,64),(76,64))
    for rank in range(8):
        for kind,name in enumerate(SWORDS):
            for state in STATES:
                tag=f"{name}_rank{rank}_{state}"
                frames=[sword_frame(kind,rank,phase,state) for phase in range(8)]
                sword_anims[tag]=frames
                if state=="orbit":
                    placed=[]
                    for frame in frames:
                        canvas=Image.new("RGBA",(128,128))
                        canvas.alpha_composite(frame,positions[kind])
                        placed.append(canvas)
                    orbit_anims[f"ar_{name}_rank{rank}"]=placed
    swords=save("swords8",sword_anims,(32,64),32,0.10)
    orbit=save("orbit8",orbit_anims,(128,128),8,0.10)
    aura_anims={}
    for rank in range(8):
        for side in ("ally","enemy"):
            aura_anims[f"aura_base_rank{rank}_{side}"]=[aura_base_frame(rank,p,side=="enemy") for p in range(8)]
        for kind in range(7):
            for side in ("ally","enemy"):
                aura_anims[f"aura_{kind}_rank{rank}_{side}"]=[aura_frame(kind,rank,p,side=="enemy") for p in range(8)]
    auras=save("auras8",aura_anims,(64,64),16,0.10)
    field_anims={f"aura_field_{kind}_rank{rank}":
                 [aura_field_frame(kind,rank,p) for p in range(8)]
                 for rank in range(8) for kind in range(7)}
    fields=save("aura_fields8",field_anims,(96,96),16,0.10)
    # Phase aliases share source pixels and allow moving world-point effects to
    # retain their global animation phase without restarting on every step.
    for tag,meta in list(fields.items()):
        for phase,entry in enumerate(meta["frames"]):
            fields[f"{tag}_frame{phase}"]={"frames":[entry]}
    (MOD/"vfx"/"aura_fields8#anim.fanim").write_text(
        json.dumps({"anims":fields},separators=(",",":")),encoding="utf-8")
    badges=save("badges8",badge_frames(),(48,96),8,0.12)
    rust=(ROOT/"native"/"tfm2_custom_ai"/"src"/"isliid.rs").read_text(encoding="utf-8")
    patterns=re.findall(r'Pattern\{name:"([^"]+)",swords:\d+,style:\d+,effect:(\d+)\}',rust)
    assert len(patterns)==30
    flags={}
    for i,(name,effect) in enumerate(patterns):
        for phase in PHASES:
            flags[f"flag_pattern_{i}_{phase}"]=[flag_frame(name,EFFECTS[int(effect)],phase)]
    for i,name in enumerate(SWORDS):
        for phase in PHASES:
            flags[f"flag_solo_{i}_{phase}"]=[flag_frame(name,SOLO_EFFECTS[i],phase)]
    flag_meta=save("flags",flags,(120,48),8,0.1)

    data_path=MOD/"champion"/"tfm2_isliid_emperor.data_champion"
    data=json.loads(data_path.read_text(encoding="utf-8"))
    data["attack"]["cooltime"]=72
    prefix="tfm2_isliid_emperor_"
    old_sword=tuple(prefix+n+"_" for n in SWORDS)
    data["view_effects"]=[v for v in data["view_effects"] if not v["name"].startswith(old_sword)
                          and not v["name"].startswith(prefix+"aura_")
                          and not v["name"].startswith(prefix+"flag_")]
    data["view_buffs"]=[v for v in data["view_buffs"] if not v["name"].startswith(
        ("il_ar_","il_rank","il_imperial","il_aura_base_","il_aura_visual_"))]
    for tag in swords:
        data["view_effects"].append({"type":"Animation","name":prefix+tag,
            "anim":"asset/tfm2_custom/vfx/swords8","tag":tag,"z":3,"is_follow":False})
    for tag in orbit:
        data["view_buffs"].append({"type":"Animated","name":"il_"+tag,
            "anim":"asset/tfm2_custom/vfx/orbit8","tag":tag,"z":2})
    for tag in auras:
        name=("il_"+tag) if tag.startswith("aura_base_") else ("il_aura_visual_"+tag[5:])
        data["view_buffs"].append({"type":"Animated","name":name,
            "anim":"asset/tfm2_custom/vfx/auras8","tag":tag,"z":-1})
    for tag in fields:
        if "_frame" in tag:
            data["view_effects"].append({"type":"Animation","name":prefix+tag,
                "anim":"asset/tfm2_custom/vfx/aura_fields8","tag":tag,"z":-2,"is_follow":False})
    for tag in badges:
        name="il_"+tag
        data["view_buffs"].append({"type":"Animated","name":name,
            "anim":"asset/tfm2_custom/vfx/badges8","tag":tag,"z":4})
    for tag in flag_meta:
        data["view_effects"].append({"type":"Animation","name":prefix+tag,
            "anim":"asset/tfm2_custom/vfx/flags","tag":tag,"z":5,"is_follow":False})
    data_path.write_text(json.dumps(data,indent=2,ensure_ascii=False)+"\n",encoding="utf-8")
    manifest={"swords":swords,"orbit":orbit,"auras":auras,"fields":fields,"badges":badges,"flags":flag_meta}
    (EDITOR/"isliid-art-manifest.json").write_text(json.dumps(manifest,separators=(",",":")),encoding="utf-8")
    print(f"Generated {len(swords)} sword, {len(orbit)} orbit, {len(auras)} recipient aura, {len(fields)} field, {len(badges)} badge, {len(flags)} flag animations")


if __name__=="__main__":main()
