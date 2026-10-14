"""Reproducible Magnus data, catalogue bindings, pixel sprites and runtime views."""
import json
import math
import importlib.util
from functools import lru_cache
from pathlib import Path
from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parents[2]
MOD = ROOT / "mods/tfm2_custom"
HERE = Path(__file__).parent
ID = "tfm2_custom_magnus"
CAT = json.loads((HERE / "catalogue.json").read_text(encoding="utf-8"))
OUTLINE = "#101724"
WHITE = "#e4dcc5"
GOLD = "#d4b875"

@lru_cache(maxsize=1)
def carrier_hull():
    # Reuse the repository's verified base-carrier footprint and anchor margins.
    path=ROOT/"Claude outputs/unified_theory/cosmic_art.py"
    spec=importlib.util.spec_from_file_location("magnus_carrier_footprint",path)
    module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
    original=[(x+8,y) for x,y in module.crystal_formation_hull()]
    points=sorted(set(original+[(63-x,y) for x,y in original]))
    def cross(o,a,b):return (a[0]-o[0])*(b[1]-o[1])-(a[1]-o[1])*(b[0]-o[0])
    lower=[];upper=[]
    for q in points:
        while len(lower)>=2 and cross(lower[-2],lower[-1],q)<=0:lower.pop()
        lower.append(q)
    for q in reversed(points):
        while len(upper)>=2 and cross(upper[-2],upper[-1],q)<=0:upper.pop()
        upper.append(q)
    return lower[:-1]+upper[:-1]


def write(p, s):
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(s, encoding="utf-8", newline="\n")


def atlas(path, entries, cols=8):
    """All frames in a tag share one centred canvas; no drifting crops."""
    count = sum(len(frames) for frames in entries.values())
    w = max(im.width for frames in entries.values() for im, _ in frames)
    h = max(im.height for frames in entries.values() for im, _ in frames)
    sheet = Image.new("RGBA", (cols * w, math.ceil(count / cols) * h))
    anims = {}
    n = 0
    for tag, frames in entries.items():
        rects = []
        for im, seconds in frames:
            x, y = n % cols * w, n // cols * h
            sheet.alpha_composite(im, (x + (w-im.width)//2, y + (h-im.height)//2))
            rects.append({"duration": round(seconds, 7), "data": {"x": x, "y": y, "w": w, "h": h}})
            n += 1
        anims[tag] = {"frames": rects}
    assert max(sheet.size) <= 2048, (path, sheet.size)
    path.parent.mkdir(parents=True, exist_ok=True)
    sheet.save(str(path) + "#sheet.png", optimize=True)
    write(Path(str(path) + "#anim.fanim"), json.dumps({"anims": anims}, separators=(",", ":")) + "\n")
    return anims


def body(pose, frame):
    im = Image.new("RGBA", (48, 56))
    d = ImageDraw.Draw(im)
    bob = (frame // 2) % 2 if pose in ("idle", "run") else 0
    y = bob
    d.ellipse((10, 45, 40, 51), fill=(8, 12, 22, 65))
    if pose == "dead":
        d.rounded_rectangle((8, 35, 36, 46), 3, fill="#26334b", outline=OUTLINE)
        d.rectangle((33, 34, 43, 43), fill="#c9977e", outline=OUTLINE)
        return im
    stride = [-2,-1,0,1,2,1,0,-1][frame] if pose == "run" else 0
    d.rectangle((17+stride, 35+y, 24+stride, 48), fill="#1a2537", outline=OUTLINE)
    d.rectangle((27-stride, 35+y, 33-stride, 48), fill="#233047", outline=OUTLINE)
    d.rectangle((15+stride, 47, 25+stride, 49), fill=OUTLINE)
    d.rectangle((27-stride, 47, 36-stride, 49), fill=OUTLINE)
    d.polygon([(17,22+y),(29,21+y),(34,37+y),(16,38+y),(12,30+y)], fill="#263b59", outline=OUTLINE)
    d.polygon([(20,24+y),(24,25+y),(23,37+y),(18,36+y)], fill="#5b7391")
    d.line((27,24+y,31,35+y), fill="#708cad", width=2)
    d.rectangle((22,19+y,27,25+y), fill="#c9977e")
    # Three-quarter face, beard and swept brown hair.
    d.rounded_rectangle((18,7+y,32,21+y), 3, fill="#d8ad93", outline=OUTLINE)
    d.rectangle((28,10+y,33,17+y), fill="#e7bca0")
    d.polygon([(18,17+y),(22,18+y),(29,17+y),(32,19+y),(28,23+y),(21,22+y)], fill="#785343")
    d.line((23,20+y,28,20+y), fill="#ba8670")
    d.rectangle((29,12+y,31,13+y), fill=OUTLINE)
    d.rectangle((28,10+y,31,10+y), fill="#64493d")
    d.polygon([(16,12+y),(17,6+y),(20,4+y),(27,4+y),(32,7+y),(30,9+y),(22,8+y),(20,13+y)], fill="#76533e", outline=OUTLINE)
    d.line((20,5+y,27,5+y), fill="#aa7b54", width=2)
    reach = 3 if pose in ("attack", "skill1", "skill2", "ult") and 2 <= frame <= 5 else 0
    d.line((30,26+y,34+reach,31+y), fill="#263b59", width=5)
    d.ellipse((32+reach,28+y,36+reach,33+y), fill="#d8ad93", outline=OUTLINE)
    d.line((17,28+y,22,34+y), fill="#314d6d", width=5)
    # Handheld board, with seven pieces and a clock rather than combat pawns.
    d.polygon([(23,31+y),(39,33+y),(35,39+y),(20,37+y)], fill="#56452f", outline=OUTLINE)
    for yy in range(4):
        for xx in range(4):
            d.rectangle((23+xx*3-yy,32+yy+y,25+xx*3-yy,32+yy+y), fill=WHITE if (xx+yy)%2==0 else "#574a3f")
    for xx in (23,26,29,32,35):
        d.point((xx,32+y), fill="#f5eedc")
    if pose in ("skill1","ult") and frame in (3,4):
        d.line((33,27+y,29,32+y), fill="#e7bca0", width=2)
    if pose == "hit":
        d.line((13,11,10,8), fill="#e4a68c")
    return im


def piece(kind, frame, dark=False, health=10):
    im = Image.new("RGBA", (64, 64))
    d = ImageDraw.Draw(im)
    d.polygon(carrier_hull(),fill="#1a2538",outline="#647389")
    # Opaque carved backing deliberately covers the base carrier's visible body.
    d.rounded_rectangle((9,7,54,58), 12, fill="#1a2538", outline="#647389", width=2)
    d.ellipse((6,48,58,62), fill=(6,10,17,140))
    fill = "#aab2c4" if dark else WHITE
    hi = "#e6eaf4" if dark else "#fff7df"
    d.polygon([(15,52),(18,46),(47,46),(50,52)], fill=fill, outline=OUTLINE)
    d.rectangle((13,52,51,56), fill=fill, outline=OUTLINE)
    d.rectangle((18,54,47,54), fill=GOLD)
    if kind == "rook":
        d.polygon([(23,44),(24,23),(22,19),(22,11),(27,11),(27,16),(31,16),(31,11),(36,11),(36,16),(40,16),(40,11),(45,11),(45,19),(41,23),(42,44)], fill=fill, outline=OUTLINE)
        d.rectangle((27,23,37,41), fill=hi)
    elif kind == "bishop":
        d.polygon([(22,44),(28,30),(25,27),(24,22),(29,12),(33,8),(38,16),(41,22),(39,28),(36,30),(42,44)], fill=fill, outline=OUTLINE)
        d.line((30,20,36,14), fill=OUTLINE, width=2)
        d.line((30,31,29,41), fill=hi, width=2)
    elif kind == "knight":
        d.polygon([(22,45),(25,35),(22,30),(18,30),(17,25),(24,15),(24,10),(29,14),(34,11),(43,19),(46,32),(44,45)], fill=fill, outline=OUTLINE)
        d.line((35,18,40,32), fill=hi, width=2)
        d.rectangle((24,20,26,22), fill=OUTLINE)
        d.line((18,28,25,28), fill=OUTLINE)
    elif kind == "king":
        d.polygon([(23,45),(28,29),(24,25),(25,17),(29,15),(29,7),(35,7),(35,15),(40,17),(41,25),(37,29),(42,45)], fill=fill, outline=OUTLINE)
        d.line((26,10,39,10), fill=fill, width=3)
    else:
        d.polygon([(23,45),(28,28),(23,24),(20,12),(28,18),(32,10),(37,18),(44,12),(41,24),(37,28),(42,45)], fill=fill, outline=OUTLINE)
        for x,y in ((20,10),(32,8),(44,10)):
            d.ellipse((x-2,y-2,x+2,y+2), fill=GOLD, outline=OUTLINE)
        d.line((28,26,38,26), fill=GOLD, width=2)
    d.rectangle((14,60,49,62), fill=OUTLINE)
    if health:
        d.rectangle((15,60,15+int(33*health/10),61), fill="#7fcda9")
    d.point((17+(frame%4),53), fill=hi)
    return im


def font(size=10):
    p=Path("C:/Windows/Fonts/consolab.ttf")
    return ImageFont.truetype(str(p),size) if p.exists() else ImageFont.load_default()


def art():
    poses=["idle","run","attack","skill1","skill2","ult","hit","dead"]
    atlas(MOD/"champions"/ID,{p:[(body(p,f),0.1 if p in ("idle","run") else 0.05) for f in range(8)] for p in poses})
    entries={}
    for k in ("rook","bishop","knight","queen"):
        entries["piece_"+k]=[(piece(k,f),0.5) for f in range(8)]
    # Per-piece bodies have the same fixed anchor and a shared small atlas.
    atlas(MOD/"vfx"/"magnus_pieces",entries)
    fx={}
    for h in range(4):
        frames=[]
        for f in range(8):
            im=Image.new("RGBA",(208,208));d=ImageDraw.Draw(im)
            # Board centre is between cells; 25.263px approximates the exact world tile size.
            for y in range(8):
                for x in range(8):
                    a=3+round(x*24_000/950);b=3+round(y*24_000/950)
                    z=3+round((x+1)*24_000/950);w=3+round((y+1)*24_000/950)
                    d.rectangle((a,b,z,w),fill=(39,56,76,29) if (x+y)%2 else (193,182,143,22),outline=(214,197,151,37))
            d.rectangle((2,2,205,205),outline=(213,183,112,110+(f%4)*6),width=2)
            im=im.rotate(h*90)
            frames.append((im,31/60))
        fx["board"+str(h)]=frames
    atlas(MOD/"vfx"/"magnus_board",fx,cols=8)
    fx={}
    for tag,ticks,color in (("warn30",18,(231,174,73)),("warn45",27,(231,174,73)),("warn24",24,(226,91,104)),("checkmate",90,(226,91,104)),("brilliant",90,(90,215,167)),("recovery",248,(120,145,185)),("impact",18,(255,235,186))):
        frames=[]
        for f in range(8):
            im=Image.new("RGBA",(32,32));d=ImageDraw.Draw(im)
            alpha=max(25,190-f*12) if tag=="impact" else 110+f*12
            d.rectangle((3,3,28,28),fill=(*color,25),outline=(*color,alpha),width=2)
            if tag=="recovery": d.arc((9,9,23,23),-90,f*45-90,fill=(*color,210),width=2)
            if tag=="impact": d.line((8,8,24,24),fill=(*color,alpha),width=2);d.line((24,8,8,24),fill=(*color,alpha),width=2)
            frames.append((im,ticks/60/8))
        fx[tag]=frames
    atlas(MOD/"vfx"/"magnus_events",fx,cols=8)
    recovery={}
    for i,label in enumerate(("R1","R2","B1","B2","N1","N2","Q")):
      for seconds in range(1,15):
        im=Image.new("RGBA",(32,32));d=ImageDraw.Draw(im)
        d.rounded_rectangle((3,3,28,28),5,fill=(20,31,47,150),outline=(138,155,177,190))
        d.text((16,5),label,font=font(8),fill=GOLD,anchor="mt")
        d.text((16,16),str(seconds)+"s",font=font(9),fill="#b4c2d2",anchor="mt")
        recovery["recover_"+str(i)+"_"+str(seconds)]=[(im,1.1)]
    atlas(MOD/"vfx"/"magnus_recovery",recovery)
    # Persistent layers change with coverage; each square chooses one contributor.
    def coverage(dx,dy):
        w=2*math.ceil(abs(dx)*24000/950)+28;h=2*math.ceil(abs(dy)*24000/950)+28
        im=Image.new("RGBA",(w,h));d=ImageDraw.Draw(im)
        x=w/2+dx*24000/950;y=h/2+dy*24000/950
        d.rectangle((round(x-12),round(y-12),round(x+12),round(y+12)),fill=(97,179,148,12),outline=(133,197,169,22))
        d.ellipse((round(x-1),round(y-1),round(x+1),round(y+1)),fill=(145,214,182,45))
        return ("cov_"+str(dx+7)+"_"+str(dy+7),[(im,1.0)])
    atlas(MOD/"vfx"/"magnus_cover_h",dict(coverage(dx,0) for dx in range(-7,8) if dx),cols=4)
    atlas(MOD/"vfx"/"magnus_cover_v",dict(coverage(0,dy) for dy in range(-7,8) if dy),cols=14)
    for n in range(1,8):
        atlas(MOD/"vfx"/("magnus_cover_d"+str(n)),dict(coverage(dx*n,dy*n) for dx,dy in ((1,1),(1,-1),(-1,1),(-1,-1))),cols=4)
    atlas(MOD/"vfx"/"magnus_cover_n",dict(coverage(dx,dy) for dx,dy in ((1,2),(2,1),(-1,2),(-2,1),(1,-2),(2,-1),(-1,-2),(-2,-1))),cols=4)
    overlays={}
    for r in range(8):
        frames=[]
        for f in range(16):
            im=Image.new("RGBA",(48,56));d=ImageDraw.Draw(im)
            d.ellipse((33,1,46,14),fill=OUTLINE,outline=GOLD)
            d.text((36,2),str(r+1),font=font(9),fill=WHITE)
            d.point((35+f%8,3),fill="#f9e5ae")
            frames.append((im,0.1))
        overlays["rank"+str(r)]=frames
    atlas(MOD/"vfx"/"magnus_ranks",overlays)
    overlays={}
    for slot in range(16):
      for role in range(5):
        frames=[]
        for f in range(1):
            im=Image.new("RGBA",(48,56));small=piece(("king","queen","rook","bishop","knight")[role],f,True).resize((14,14),Image.Resampling.NEAREST)
            im.alpha_composite(small,(17,0));frames.append((im,0.5))
        overlays["enemy"+str(slot)+"_"+str(role)]=frames
    atlas(MOD/"vfx"/"magnus_roles",overlays)
    overlays={}
    for hp in range(11):
        im=Image.new("RGBA",(64,64));d=ImageDraw.Draw(im);d.rectangle((14,60,49,62),fill=OUTLINE)
        if hp:d.rectangle((15,60,15+int(33*hp/10),61),fill="#7fcda9")
        overlays["hp"+str(hp)]=[(im,0.5) for _ in range(8)]
    atlas(MOD/"vfx"/"magnus_health",overlays)
    glow=[]
    for f in range(8):
        im=Image.new("RGBA",(64,64));d=ImageDraw.Draw(im);d.rounded_rectangle((7,5,56,61),12,outline=(242,202,120,150+f*12),width=3);glow.append((im,.1))
    atlas(MOD/"vfx"/"magnus_glow",{"glow":glow})
    identities={}
    for i,label in enumerate(("R1","R2","B1","B2","N1","N2","Q")):
        im=Image.new("RGBA",(64,64));d=ImageDraw.Draw(im)
        d.text((32,0),label,font=font(8),fill=GOLD,anchor="mt",stroke_width=1,stroke_fill=OUTLINE)
        identities["identity"+str(i)]=[(im,1.0)]
    atlas(MOD/"vfx"/"magnus_identities",identities)
    casts={}
    for queen in (False,True):
      for fast in (False,True):
       for left in (False,True):
        ticks=(45 if queen else 30)*(0.8 if fast else 1)
        tag="cast"+("_queen" if queen else "")+("_fast" if fast else "")+("_l" if left else "")
        frames=[]
        for f in range(8):
            im=body("skill1",f)
            if left:im=im.transpose(Image.Transpose.FLIP_LEFT_RIGHT)
            frames.append((im,ticks/60/8))
        casts[tag]=frames
    atlas(MOD/"vfx"/"magnus_casts",casts)
    clues={}
    for i,g in enumerate(CAT["games"]):
        im=Image.new("RGBA",(224,96));d=ImageDraw.Draw(im)
        d.text((112,0),g["title"],font=font(10),fill=GOLD,anchor="mt",stroke_width=1,stroke_fill=OUTLINE)
        # A compact clue is held above the miniature board, away from health bars.
        short=("Q check > Q offer > R check","R > N > R: protect knight","B guards R > Q offer > mate","B check > Q offer > N discovery","Two knights > Q offer > B mate","R offer > other R > Q hunt")[i]
        d.text((112,13),short,font=font(8),fill=WHITE,anchor="mt",stroke_width=1,stroke_fill=OUTLINE)
        clues["clue"+str(i)]=[(im,1.0)]
    atlas(MOD/"vfx"/"magnus_clues",clues,cols=6)
    labels=["","BOARD READY","CHECK","CHECKMATE","FORK","DISCOVERED ATTACK","SACRIFICE","STALEMATE","BRILLIANT RECALL","INTERRUPTED","CASTLING","PIN"]
    words={}
    for i,label in enumerate(labels):
        im=Image.new("RGBA",(192,72));d=ImageDraw.Draw(im)
        if label:d.text((96,7),label,font=font(10),fill=GOLD,anchor="mt",stroke_width=1,stroke_fill=OUTLINE)
        words["label"+str(i)]=[(im,90/60/8) for _ in range(8)]
    for i,g in enumerate(CAT["games"]):
        im=Image.new("RGBA",(256,72));d=ImageDraw.Draw(im)
        d.text((128,2),g["title"],font=font(11),fill=GOLD,anchor="mt",stroke_width=1,stroke_fill=OUTLINE)
        d.text((128,17),str(g["year"])+" · ADAPTATION",font=font(9),fill=WHITE,anchor="mt")
        d.text((128,30),g["players"],font=font(8),fill=WHITE,anchor="mt")
        words["game"+str(i)]=[(im,90/60/8) for _ in range(8)]
    atlas(MOD/"vfx"/"magnus_labels",words,cols=8)
    im=Image.new("RGBA",(6*96,180),"#111a2b");d=ImageDraw.Draw(im)
    for i,k in enumerate(("rook","bishop","knight","queen","king")):
        im.alpha_composite(piece(k,0),(i*96+16,38));d.text((i*96+48,112),k.upper(),font=font(11),fill=WHITE,anchor="mt")
    im.alpha_composite(body("idle",0).resize((96,112),Image.Resampling.NEAREST),(5*96,15));d.text((5*96+48,140),"MAGNUS",font=font(11),fill=GOLD,anchor="mt")
    (ROOT/"docs").mkdir(exist_ok=True);im.save(ROOT/"docs"/"magnus-art-preview.png")


def data():
    never=lambda action:{"action_name":action,"cancelable":False,"growth_range":0,"can_use_with_move":False,"duration":6,"cooltime":5184000,"start_timing":3,"range":1,"casting_type":"Targeting","casting_target":"EnemyChampionInCC","attack_type":"Skill","effect":{"type":"Combine","effects":[]}}
    stat={"attack":45,"magic_power":50,"hp":800,"defence":18,"magic_resistance":22,"move_speed":960,"hp_regen":0,"stack":0,"crit_chance":0}
    growth={"attack":4,"magic_power":10,"hp":85,"defence":3,"magic_resistance":4,"move_speed":8,"hp_regen":0,"stack":0,"crit_chance":0}
    c={"id":ID,"category":"Magician","tags":["AP","CC","Range"],"sprite":"asset/tfm2_custom/champions/"+ID,"anim_prefix":"","stat":stat,"growth":growth,
       "attack":{"action_name":"attack","cancelable":True,"growth_range":0,"can_use_with_move":False,"duration":24,"cooltime":90,"start_timing":12,"range":55000,"casting_type":"Targeting","casting_target":"Enemy","attack_type":"BaseAttack",
                 "effect":{"type":"ApAttack","damage":20,"attack_ratio":35,"attack_effect_type":"Target"}},
       "skill":never("skill1"),"skill2":never("skill2"),"ult":never("ult"),"passive":{"passive_ref":"tfm2_custom_ai:magnus","params":{}},"view_buffs":[],"view_effects":[]}
    for path in sorted((MOD/"vfx").glob("magnus_*#anim.fanim")):
        sheet=path.name.split("#")[0]
        for tag in json.loads(path.read_text())["anims"]:
            if tag.startswith(("piece_","rank","enemy","hp","cast","clue","glow","identity","cov_")):
                c["view_buffs"].append({"type":"Animated","name":"mg_"+tag,"anim":"asset/tfm2_custom/vfx/"+sheet,"tag":tag,"z":-1 if tag.startswith("cov_") else 8 if tag.startswith(("rank","enemy","hp","cast","clue","glow","identity")) else 4})
            else:
                c["view_effects"].append({"type":"Animation","name":ID+"_"+tag,"anim":"asset/tfm2_custom/vfx/"+sheet,"tag":tag,"z":-2 if tag.startswith("board") else 7,"is_follow":tag.startswith(("label","game","brilliant"))})
    for key in ("skill","skill2","ult"):c[key]["description"]="#asset/base/text/champion?description."+ID+"."+key
    write(MOD/"champion"/(ID+".data_champion"),json.dumps(c,indent=2)+"\n")
    text_path=MOD/"text"/"champion.i18n";t=json.loads(text_path.read_text(encoding="utf-8"))
    t["en"]["description"][ID]={
        "name":"Magnus, the Grandmaster",
        "skill":"Magnus commands two rooks, two bishops, two knights and a queen on an 8×8 chessboard. Pieces are destructible and move on squares; champions move freely. One command at a time: 0.5 s, or 0.75 s for the queen. Basic attack: 20 + 35% AP every 1.5 s. Board deployment takes 1 s, cooldown 16 s. Each deployment places a rook and queen beside Magnus, randomising the rest with bishops on opposite colours. Reset fully restores that adjacent rook and queen; other pieces keep HP and recovery.",
        "skill2":"Rooks: 35 + 35% AP, 30% owner HP, recover in 10 s. Bishops: 30 + 40% AP, 20% HP, 8 s. Knights: 45 + 35% AP, 25% HP, 8 s. Queen: 65 + 55% AP, 35% HP, 14 s. All pieces have 15 armour and MR. Friendly pieces block sliders, knights jump. Each command hits each enemy once. Castling swaps Magnus with a living rook on a clear row or column within three squares, shielding for 80 + 20% AP for 1 s; cooldown 12 s.",
        "ult":"Tactical checkmate: attack the enemy king and cover every adjacent traversable escape square using at least two pieces for a 0.4 s warning. Stuns enemies inside the board for 1.25 s; once per deployment and a shared 20 s cooldown. Six adapted famous-game challenges grant Brilliant Recall: 20% shorter command durations for 5 s (0.4 s / queen 0.6 s), reward cooldown 20 s. Reset and death do not bypass these cooldowns. All pieces and challenges are available at every mastery rank. Mastery improves planning, not numeric power. Open Chess Lab to practise."
    }
    write(text_path,json.dumps(t,ensure_ascii=False,indent=2)+"\n")
    write(ROOT/"editor"/"magnus-data.js","/* Generated from the canonical catalogue. */\nwindow.MagnusData = "+json.dumps(CAT,ensure_ascii=False)+";\n")
    s=["// Generated by Claude outputs/magnus/generate.py. Do not edit.",
       "#[derive(Clone,Copy,Debug,PartialEq,Eq)]","pub enum Pattern { Birthday, Marathon, Opera, Century, Immortal, Hunt }",
       "#[allow(dead_code)]","pub struct HistoricGame { pub id:&'static str,pub title:&'static str,pub players:&'static str,pub year:u32,pub source:&'static str,pub moves:&'static [&'static str],pub clue:&'static str,pub steps:&'static [&'static str],pub pattern:Pattern,pub pieces:[usize;4],pub queen_target:bool,pub stages:usize }","pub const GAMES: [HistoricGame;6] = ["]
    for g in CAT["games"]:
        quote=lambda x:json.dumps(x,ensure_ascii=False)
        r=g["matching"]
        s.append("HistoricGame{id:"+quote(g["id"])+",title:"+quote(g["title"])+",players:"+quote(g["players"])+",year:"+str(g["year"])+",source:"+quote(g["source"])+",moves:&["+",".join(quote(m) for m in g["moves"])+"],clue:"+quote(g["clue"])+",steps:&["+",".join(quote(x) for x in g["steps"])+"],pattern:Pattern::"+r["pattern"]+",pieces:"+str(r["pieces"])+",queen_target:"+str(r["queen_target"]).lower()+",stages:"+str(r["stages"])+"},")
    s+=["];","pub fn recovery_tag(piece:usize,ticks:usize)->&'static str {","const TAGS:[[&str;14];7]=["]
    for i in range(7):s.append("["+",".join(json.dumps("recover_"+str(i)+"_"+str(sec)) for sec in range(1,15))+"],")
    s+=["];TAGS[piece.min(6)][ticks.div_ceil(60).clamp(1,14)-1]","}","pub fn tags(name:&str)->(bool,bool) { match name {"]
    raw=(ROOT/"editor"/"gamedata.js").read_text(encoding="utf-8")
    gamedata=json.loads(raw.split("window.TFM2_GAMEDATA = ",1)[1].strip().rstrip(";"))
    tags={n:v.get("tags",[]) for n,v in gamedata["info"].items()}
    for f in (ROOT/"mods").glob("*/champion/*.data_champion"):
        j=json.loads(f.read_text(encoding="utf-8"));tags[j["id"]]=j.get("tags",[])
    for n,ts in sorted(tags.items()):
        if "Tank" in ts or "Range" in ts:s.append(json.dumps(n)+ "=>("+str("Tank" in ts).lower()+","+str("Range" in ts).lower()+"),")
    s+=["_ => (false,false),","}}"]
    write(ROOT/"native/tfm2_custom_ai/src/magnus_data.rs","\n".join(s)+"\n")
    # A read-only artwork manifest makes the lab and asset verifier independent of filename guesses.
    manifest={}
    for f in [MOD/"champions"/(ID+"#anim.fanim"),*sorted((MOD/"vfx").glob("magnus_*#anim.fanim"))]:
        manifest[f.name.split("#")[0]]=json.loads(f.read_text(encoding="utf-8"))["anims"]
    write(ROOT/"editor"/"magnus-art.json",json.dumps(manifest,separators=(",",":"))+"\n")


if __name__=="__main__":
    art();data()
    print("Magnus: champion, seven-piece art, six historical references, native data and editor manifest generated.")
