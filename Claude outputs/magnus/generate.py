"""Reproducible Magnus data, catalogue bindings, pixel sprites and runtime views."""
import json
import math
import importlib.util
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

def write(p, s):
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(s, encoding="utf-8", newline="\n")


# Exact frames are shared after one symmetric crop per animation.
spec=importlib.util.spec_from_file_location("magnus_packing",HERE/"packing.py")
packing=importlib.util.module_from_spec(spec);spec.loader.exec_module(packing)
atlas=packing.atlas
TILE_UNITS=CAT["rules"]["tileUnits"]
spec=importlib.util.spec_from_file_location("magnus_rank_art",HERE/"rank_art.py")
R=importlib.util.module_from_spec(spec);spec.loader.exec_module(R)

# Body and cast sheets share the reviewed Slim A renderer.
spec=importlib.util.spec_from_file_location("magnus_character",HERE/"character.py")
character=importlib.util.module_from_spec(spec);spec.loader.exec_module(character)
def body(pose,frame):return character.frame(pose,frame)


def piece(kind,frame,dark=False,health=10,look=0):return R.piece(kind,frame,look,dark)


def font(size=10):
    p=Path("C:/Windows/Fonts/consolab.ttf")
    return ImageFont.truetype(str(p),size) if p.exists() else ImageFont.load_default()


def art():
    poses=["idle","run","attack","skill1","skill2","ult","hit","dead"]
    atlas(MOD/"champions"/ID,{p:[(body(p,f),0.1 if p in ("idle","run") else 0.05) for f in range(8)] for p in poses})
    for look in range(11):
        entries={"piece_"+k+"_"+str(look):[(piece(k,f,look=look),0.5)for f in range(8)]for k in("rook","bishop","knight","queen","pawn")}
        atlas(MOD/"vfx"/("magnus_pieces_"+str(look)),entries)
        outfits={}
        for pose in poses:
            for left in(False,True):
                frames=[]
                for f in range(8):
                    im=R.body(pose,f,look)
                    # Ordinary outfits replace the torso only; the base head and animated feet remain visible.
                    alpha=im.getchannel("A");d=ImageDraw.Draw(alpha);d.rectangle((0,0,47,22),fill=0);d.rectangle((0,40,47,55),fill=0);im.putalpha(alpha)
                    if left:im=im.transpose(Image.Transpose.FLIP_LEFT_RIGHT)
                    frames.append((im,0.1 if pose in("idle","run")else 0.05))
                outfits["outfit"+str(look)+"_"+pose+("_l"if left else"")]=frames
        atlas(MOD/"vfx"/("magnus_outfit_"+str(look)),outfits)
    # Original piece sheet is obsolete; all skins keep the same small carrier.
    for suffix in("#sheet.png","#anim.fanim"):(MOD/"vfx"/("magnus_pieces"+suffix)).unlink(missing_ok=True)
    fx={}
    for h in range(4):
        frames=[]
        for f in range(8):
            size=2*4+round(8*TILE_UNITS/950);im=Image.new("RGBA",(size,size));d=ImageDraw.Draw(im)
            # Board centre is between cells; pixels derive from the shared world-unit scale.
            for y in range(8):
                for x in range(8):
                    a=4+round(x*TILE_UNITS/950);b=4+round(y*TILE_UNITS/950)
                    z=4+round((x+1)*TILE_UNITS/950);w=4+round((y+1)*TILE_UNITS/950)
                    d.rectangle((a,b,z,w),fill=(39,56,76,29) if (x+y)%2 else (193,182,143,22),outline=(214,197,151,37))
            d.rectangle((2,2,size-3,size-3),outline=(213,183,112,110),width=2)
            # Symmetric square grid: headings alias the same pixels.
            frames.append((im,30/60/8))
        fx["board"+str(h)]=frames
        faded=[]
        for original,duration in frames:
            copy=original.copy();copy.putalpha(copy.getchannel("A").point(lambda a:a//2));faded.append((copy,duration))
        fx["board"+str(h)+"_fade"]=faded
    atlas(MOD/"vfx"/"magnus_board",fx,cols=8)
    fx={}
    for tag,ticks,color in (("warn30",18,(231,174,73)),("warn45",27,(231,174,73)),("warn24",24,(226,91,104)),("checkmate",90,(226,91,104)),("brilliant",90,(90,215,167)),("recovery",248,(120,145,185)),("impact",18,(255,235,186))):
        frames=[]
        for f in range(8):
            im=Image.new("RGBA",(40,40));d=ImageDraw.Draw(im)
            alpha=max(25,190-f*12) if tag=="impact" else 110+f*12
            d.rectangle((3,3,36,36),fill=(*color,25),outline=(*color,alpha),width=2)
            if tag=="recovery": d.arc((9,9,23,23),-90,f*45-90,fill=(*color,210),width=2)
            if tag=="impact": d.line((8,8,24,24),fill=(*color,alpha),width=2);d.line((24,8,8,24),fill=(*color,alpha),width=2)
            frames.append((im,ticks/60/8))
        fx[tag]=frames
    atlas(MOD/"vfx"/"magnus_events",fx,cols=8)
    # Captured pieces have no recovery timer in army rules v2.
    for suffix in ("#sheet.png","#anim.fanim"):
        (MOD/"vfx"/("magnus_recovery"+suffix)).unlink(missing_ok=True)
    # Persistent layers change with coverage; each square chooses one contributor.
    def coverage(dx,dy):
        w=2*math.ceil(abs(dx)*TILE_UNITS/950)+28;h=2*math.ceil(abs(dy)*TILE_UNITS/950)+28
        im=Image.new("RGBA",(w,h));d=ImageDraw.Draw(im)
        x=w/2+dx*TILE_UNITS/950;y=h/2+dy*TILE_UNITS/950
        d.rectangle((round(x-TILE_UNITS/1900),round(y-TILE_UNITS/1900),round(x+TILE_UNITS/1900),round(y+TILE_UNITS/1900)),fill=(97,179,148,12),outline=(133,197,169,22))
        d.ellipse((round(x-1),round(y-1),round(x+1),round(y+1)),fill=(145,214,182,45))
        return ("cov_"+str(dx+7)+"_"+str(dy+7),[(im,1.0)])
    atlas(MOD/"vfx"/"magnus_cover_h",dict(coverage(dx,0) for dx in range(-7,8) if dx),cols=4)
    atlas(MOD/"vfx"/"magnus_cover_v",dict(coverage(0,dy) for dy in range(-7,8) if dy),cols=14)
    for n in range(1,8):
        atlas(MOD/"vfx"/("magnus_cover_d"+str(n)),dict(coverage(dx*n,dy*n) for dx,dy in ((1,1),(1,-1),(-1,1),(-1,-1))),cols=4)
    atlas(MOD/"vfx"/"magnus_cover_n",dict(coverage(dx,dy) for dx,dy in ((1,2),(2,1),(-1,2),(-2,1),(1,-2),(2,-1),(-1,-2),(-2,-1))),cols=4)
    overlays={}
    for badge in range(17):
        frames=[]
        for f in range(16):
            im=Image.new("RGBA",(48,56));d=ImageDraw.Draw(im)
            d.rounded_rectangle((33,1,46,14),3,fill=OUTLINE,outline=GOLD)
            label=str(badge+1)if badge<7 else str(badge-6)
            if badge>=7:d.point((35,3),fill=WHITE);d.point((44,3),fill=WHITE)
            d.text((40,3),label,font=font(8),fill=WHITE,anchor="mt")
            perimeter=[(35,1),(38,1),(41,1),(44,1),(46,3),(46,6),(46,9),(46,12),(44,14),(41,14),(38,14),(35,14),(33,12),(33,9),(33,6),(33,3)]
            frames.append((im,.1))
        overlays["rank"+str(badge)]=frames
    atlas(MOD/"vfx"/"magnus_ranks",overlays)
    sparkle=[]
    for f in range(16):
        im=Image.new("RGBA",(48,56));d=ImageDraw.Draw(im);d.point(perimeter[f],fill="#fff5d2");sparkle.append((im,.1))
    atlas(MOD/"vfx"/"magnus_rank_motion",{"rank_motion":sparkle})
    for look in range(4,11):atlas(MOD/"vfx"/("magnus_construction_"+str(look)),{"construction"+str(look)+("_l"if left else""):[(R.construction(look,f,left),.2)for f in range(8)]for left in(False,True)})
    atlas(MOD/"vfx"/"magnus_plan",{"plan_panel"+("_l"if left else""):[(R.panel(left),1.0)]for left in(False,True)})
    for alternate in(False,True):
        marks={}
        for left in(False,True):
            for step in range(4):
                for cell in range(16):marks["plan_mark"+str(int(alternate))+"_"+str(step)+"_"+str(cell)+("_l"if left else"")]=[(R.marker(step,cell,alternate,left),1.0)]
        # Partition by step so first-use decode remains small.
        for step in range(4):atlas(MOD/"vfx"/("magnus_plan_"+str(int(alternate))+"_"+str(step)),{k:v for k,v in marks.items()if k.split("_")[2]==str(step)})
    atlas(MOD/"vfx"/"magnus_plan_edges",{"plan_edge_"+str(cell)+("_l"if left else""):[(R.marker(0,cell,False,left,True),1.0)]for left in(False,True)for cell in range(16)if cell%4 in(0,3)or cell//4 in(0,3)})
    frames={}
    size=2*4+round(8*TILE_UNITS/950)
    for look in range(11):
        for fade in(False,True):
            im=Image.new("RGBA",(size,size));d=ImageDraw.Draw(im);col=(189,164,111,45 if fade else 115)
            for x,y,sx,sy in((4,4,1,1),(size-5,4,-1,1),(4,size-5,1,-1),(size-5,size-5,-1,-1)):
                length=6+look;d.line((x+sx*length,y,x,y,x,y+sy*length),fill=col,width=1)
                if look>=4:d.point((x+sx*3,y+sy*3),fill=col)
                if look>=7:d.line((x+sx*5,y+sy*2,x+sx*2,y+sy*5),fill=col)
            frames["board_frame"+str(look)+("_fade"if fade else"")]=[(im,.0625)for _ in range(8)]
    atlas(MOD/"vfx"/"magnus_board_frames",frames,cols=4)
    overlays={}
    for role in range(5):
        frames=[]
        for f in range(1):
            im=Image.new("RGBA",(48,56));small=piece(("king","queen","rook","bishop","knight")[role],f,True).resize((14,14),Image.Resampling.NEAREST)
            im.alpha_composite(small,(17,0));frames.append((im,0.5))
        overlays["role"+str(role)]=frames
    atlas(MOD/"vfx"/"magnus_roles",overlays)
    # Sixteen owner-scoped names share the same five stored images.
    roles_path=MOD/"vfx"/"magnus_roles#anim.fanim"
    roles_data=json.loads(roles_path.read_text())
    roles_data["anims"]={"enemy"+str(slot)+"_"+str(role):roles_data["anims"]["role"+str(role)] for slot in range(16) for role in range(5)}
    write(roles_path,json.dumps(roles_data,separators=(",",":"))+"\n")
    mini={}
    shapes=[[(1,0),(0,1),(1,1),(2,1),(1,2)],[(0,0),(2,0),(0,1),(1,1),(2,1)],[(0,0),(1,0),(2,0),(0,1),(2,1),(0,2),(1,2),(2,2)],[(2,0),(1,1),(0,2),(1,2)],[(0,0),(1,0),(0,1),(1,2),(2,2)]]
    for left in (False,True):
      for role in range(5):
       for n in range(64):
        x,y=n%8,n//8;im=Image.new("RGBA",(32,24));d=ImageDraw.Draw(im)
        px=25+x*1.3-y*.18;py=33+y*.45
        if left:px=47-px
        px=round(16+px-24)-1;py=round(12+py-28)-1
        for dx,dy in shapes[role]:d.point((px+dx,py+dy),fill="#f1b9a8" if role>1 else GOLD)
        mini["mini_"+("l_" if left else "r_")+str(role)+"_"+str(n)]=[(im,1.0)]
    atlas(MOD/"vfx"/"magnus_mini",mini,cols=32)
    overlays={}
    for hp in range(11):
        im=Image.new("RGBA",(48,48));d=ImageDraw.Draw(im);d.rectangle((11,44,36,46),fill=OUTLINE)
        if hp:d.rectangle((12,44,12+int(23*hp/10),45),fill="#7fcda9")
        overlays["hp"+str(hp)]=[(im,0.5) for _ in range(8)]
    atlas(MOD/"vfx"/"magnus_health",overlays)
    im=Image.new("RGBA",(48,48));d=ImageDraw.Draw(im)
    d.line((12,43,35,43),fill="#9ca9bb",width=2)
    d.polygon([(21,41),(27,41),(24,44)],fill="#9ca9bb")
    atlas(MOD/"vfx"/"magnus_field",{"weakened":[(im,1.0)]})
    glow=[]
    for f in range(8):
        im=Image.new("RGBA",(48,48));d=ImageDraw.Draw(im);d.rounded_rectangle((8,3,39,44),5,outline=(242,202,120,150+f*12),width=3);glow.append((im,.1))
    atlas(MOD/"vfx"/"magnus_glow",{"glow":glow})
    identities={}
    for i,label in enumerate(("R1","R2","B1","B2","N1","N2","Q","+")):
        im=Image.new("RGBA",(48,48));d=ImageDraw.Draw(im)
        d.text((24,0),label,font=font(8),fill=GOLD,anchor="mt",stroke_width=1,stroke_fill=OUTLINE)
        identities["identity"+str(i)]=[(im,1.0)]
    atlas(MOD/"vfx"/"magnus_identities",identities)
    for look in range(11):
        casts={}
        for queen in(False,True):
          for fast in(False,True):
           for left in(False,True):
            ticks=(45 if queen else 30)*(.8 if fast else 1)
            tag="cast"+str(look)+("_queen"if queen else"")+("_fast"if fast else"")+("_l"if left else"")
            casts[tag]=[((R.body("skill1",f,look).transpose(Image.Transpose.FLIP_LEFT_RIGHT)if left else R.body("skill1",f,look)),ticks/60/8)for f in range(8)]
        atlas(MOD/"vfx"/("magnus_casts_"+str(look)),casts)
    for suffix in("#sheet.png","#anim.fanim"):(MOD/"vfx"/("magnus_casts"+suffix)).unlink(missing_ok=True)
    clues={}
    for i,g in enumerate(CAT["games"]):
        im=Image.new("RGBA",(224,96));d=ImageDraw.Draw(im)
        d.text((112,0),g["title"],font=font(10),fill=GOLD,anchor="mt",stroke_width=1,stroke_fill=OUTLINE)
        # A compact clue is held above the miniature board, away from health bars.
        short=g["clue"][:42]
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
        d.text((128,17),str(g["year"])+(" · STUDY ADAPTATION" if g.get("reference_kind")=="study" else " · GAME ADAPTATION"),font=font(9),fill=WHITE,anchor="mt")
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
       "skill":never("skill1"),"skill2":never("skill2"),"ult":never("ult"),"passive":{"passive_ref":"tfm2_custom_ai:magnus","params":{}},"passive_ult":{"passive_ref":"tfm2_custom_ai:magnus_ult","params":{}},"view_buffs":[],"view_effects":[]}
    for path in sorted((MOD/"vfx").glob("magnus_*#anim.fanim")):
        sheet=path.name.split("#")[0]
        for tag in json.loads(path.read_text())["anims"]:
            if tag.startswith(("piece_","rank","enemy","hp","cast","clue","glow","identity","cov_","mini_","weakened","outfit","construction","plan_")):
                c["view_buffs"].append({"type":"Animated","name":"mg_"+tag,"anim":"asset/tfm2_custom/vfx/"+sheet,"tag":tag,"z":-1 if tag.startswith(("cov_","construction")) else 10 if tag.startswith("mini_") else 8 if tag.startswith(("rank","enemy","hp","cast","clue","glow","identity","weakened","outfit","construction","plan_")) else 4})
            else:
                c["view_effects"].append({"type":"Animation","name":ID+"_"+tag,"anim":"asset/tfm2_custom/vfx/"+sheet,"tag":tag,"z":-2 if tag.startswith("board") else 7,"is_follow":tag.startswith(("label","game","brilliant"))})
    for key in ("skill","skill2","ult"):c[key]["description"]="#asset/base/text/champion?description."+ID+"."+key
    write(MOD/"champion"/(ID+".data_champion"),json.dumps(c,indent=2)+"\n")
    text_path=MOD/"text"/"champion.i18n";t=json.loads(text_path.read_text(encoding="utf-8"))
    t["en"]["description"][ID]={'name': 'Magnus, the Grandmaster', 'skill': 'Pawn Deployment: summon in allied sight anywhere. Command 0.5 s; cooldown 6 s. Pawn: 15% owner HP, 20 + 25% AP damage. One-square cardinal travel cannot immediately return; diagonals attack. Travel does no damage. Adapted en passant lasts 1 s. Six new forward ranks plus champion damage unlock promotion.', 'skill2': 'Develop a Piece: summon a persistent rook, bishop or knight. Command 0.5 s; cooldown 10 s. All pieces share one clock. Above seven living pieces, damage divides by 1 + 0.08 per extra piece. Field hits deal half damage. Captured pieces never recover automatically.', 'ult': 'Grandmaster Board: requires learned ultimate. Deploy in 1 s; cooldown 24 s. Rebuild seven slots across an 8x8 board (32,000 per square); queen beside Magnus. Leave for 5 s to dismiss it; pieces persist. Trap warning: 0.4 s. Board mate stuns 1.25 s; field mate 0.75 s, with 12 s target immunity. Castling shields. Twelve historical combinations grant distinct rewards. Full rules: Chess Lab.'}
    write(text_path,json.dumps(t,ensure_ascii=False,indent=2)+"\n")
    write(ROOT/"editor"/"magnus-data.js","/* Generated from the canonical catalogue. */\nwindow.MagnusData = "+json.dumps(CAT,ensure_ascii=False)+";\n")
    s=["// Generated by Claude outputs/magnus/generate.py. Do not edit.",
       "#[derive(Clone,Copy,Debug,PartialEq,Eq)]","pub enum Pattern { Birthday, Marathon, Opera, Century, Immortal, Hunt, DoubleBishop, Overload, PassedPawn, RoyalMarch, Underpromotion, Reti }",
       "#[allow(dead_code)]","pub struct HistoricGame { pub id:&'static str,pub title:&'static str,pub players:&'static str,pub year:u32,pub source:&'static str,pub moves:&'static [&'static str],pub clue:&'static str,pub steps:&'static [&'static str],pub pattern:Pattern,pub pieces:[usize;5],pub queen_target:bool,pub stages:usize,pub window:usize }","pub const GAMES: [HistoricGame;12] = ["]
    for g in CAT["games"]:
        quote=lambda x:json.dumps(x,ensure_ascii=False)
        r=g["matching"]
        s.append("HistoricGame{id:"+quote(g["id"])+",title:"+quote(g["title"])+",players:"+quote(g["players"])+",year:"+str(g["year"])+",source:"+quote(g["source"])+",moves:&["+",".join(quote(m) for m in g["moves"])+"],clue:"+quote(g["clue"])+",steps:&["+",".join(quote(x) for x in g["steps"])+"],pattern:Pattern::"+r["pattern"]+",pieces:"+str(r["pieces"])+",queen_target:"+str(r["queen_target"]).lower()+",stages:"+str(r["stages"])+",window:"+str(g["window_ticks"])+"},")
    s+=["];","pub const TILE_UNITS:i64="+str(TILE_UNITS)+";","pub fn tags(name:&str)->(bool,bool) { match name {"]
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
    print("Magnus v3: ranked Slim A, compact army, tactical preview, shared geometry and twelve references generated.")
