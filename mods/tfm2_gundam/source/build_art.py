"""Hand-authored Aegis Zero pixel art on the project's NEW CHAMPION sprite kit.

Every sprite is drawn at 1x. The first eight animation rows and their frame
durations come directly from the kit; flight, dive and landing extend it.
No generated image or existing champion pixels are used in the artwork.
"""
from pathlib import Path
import json
import math
from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parents[1]
KIT = Path(__file__).resolve().parents[3] / "Sprite kit"
ID = "tfm2_gundam_aegis_zero"
S = {
    "outline": (23, 28, 44, 255),
    "joint": (57, 66, 82, 255),
    "joint_hi": (101, 111, 125, 255),
    "navy": (30, 44, 83, 255),
    "blue": (52, 82, 140, 255),
    "blue_hi": (92, 134, 188, 255),
    "white_dark": (118, 137, 160, 255),
    "white_mid": (198, 211, 222, 255),
    "white": (245, 245, 234, 255),
    "red_dark": (119, 40, 53, 255),
    "red": (204, 61, 65, 255),
    "red_hi": (246, 113, 88, 255),
    "gold_dark": (157, 109, 30, 255),
    "gold": (253, 202, 72, 255),
    "energy_dark": (24, 126, 151, 255),
    "energy": (89, 239, 222, 255),
    "energy_hi": (220, 255, 247, 255),
}

def poly(d, points, color, edge="outline"):
    d.polygon(points, fill=S[color], outline=S[edge])

def ln(d, points, color, width=1):
    d.line(points, fill=S[color], width=width)

def px(d, x, y, color):
    d.point((x, y), fill=S[color])

def armor_leg(d, x, hip_y, knee_dx, foot_dx, foot_y, front):
    kx = x + knee_dx
    fx = x + foot_dx
    poly(d, [(x-4,hip_y),(x+3,hip_y-1),(kx+4,hip_y+8),
             (kx+2,hip_y+14),(kx-4,hip_y+14),(kx-5,hip_y+8)], "white_mid")
    poly(d, [(x-2,hip_y+1),(x+2,hip_y+1),(kx+2,hip_y+7),
             (kx-2,hip_y+9),(kx-4,hip_y+7)], "white")
    poly(d, [(kx-4,hip_y+13),(kx+2,hip_y+13),(kx+4,hip_y+17),
             (kx-3,hip_y+18)], "joint")
    ln(d, [(kx-3,hip_y+14),(kx+2,hip_y+14)], "joint_hi")
    poly(d, [(kx-4,hip_y+17),(kx+3,hip_y+17),(fx+5,foot_y-5),
             (fx+3,foot_y-1),(fx-5,foot_y-1),(fx-6,foot_y-5)],
         "white_dark")
    poly(d, [(kx-2,hip_y+18),(kx+2,hip_y+18),(fx+2,foot_y-6),
             (fx-4,foot_y-5)], "white")
    poly(d, [(fx-5,foot_y-4),(fx+4,foot_y-4),(fx+7,foot_y-1),
             (fx+6,foot_y+1),(fx-5,foot_y+1),(fx-7,foot_y-1)],
         "red" if front else "navy")
    ln(d, [(fx-4,foot_y-3),(fx+3,foot_y-3)], "white_mid")
    ln(d, [(fx-5,foot_y),(fx+6,foot_y)], "outline")

def draw_folded_wings(d, bob, loosen=0):
    # Four compact armored winglets packed behind the shoulders/backpack.
    poly(d, [(15,15+bob),(11-loosen,18+bob),(8-loosen,30+bob),
             (13,36+bob),(18,31+bob),(20,21+bob)], "navy")
    poly(d, [(14,17+bob),(10-loosen,21+bob),(10-loosen,29+bob),
             (14,28+bob),(17,22+bob)], "white_mid")
    ln(d, [(11-loosen,21+bob),(14,19+bob)], "white")
    poly(d, [(17,27+bob),(12,31+bob),(11,38+bob),(16,36+bob),
             (20,31+bob)], "white_dark")
    poly(d, [(32,16+bob),(37+loosen,19+bob),(39+loosen,30+bob),
             (36,35+bob),(32,31+bob),(29,21+bob)], "navy")
    poly(d, [(34,18+bob),(37+loosen,22+bob),(37+loosen,29+bob),
             (34,28+bob),(31,22+bob)], "white_mid")
    poly(d, [(31,27+bob),(36,31+bob),(38,38+bob),(33,36+bob),
             (29,31+bob)], "white_dark")
    for x,y in [(16,23+bob),(32,23+bob)]:
        px(d,x,y,"energy_dark")

def draw_body(pose="idle", frame=0):
    im = Image.new("RGBA", (48,56))
    d = ImageDraw.Draw(im)
    bob = 1 if pose == "run" and frame in (1,4) else 0
    bob += 1 if pose == "idle" and frame == 2 else 0
    stride = (-3,-1,2,3,1,-2)[frame] if pose == "run" else 0
    front_stride = (2,0,-2,-3,0,3)[frame] if pose == "run" else 0
    if pose == "skill1": front_stride = (0,2,4,2)[frame]
    if pose == "skill2": front_stride = (0,1,2,0)[frame]
    foot_l = 48 - (2 if pose == "run" and frame in (1,2) else 0)
    foot_r = 48 - (2 if pose == "run" and frame in (4,5) else 0)
    if pose == "ult_land": foot_l, foot_r = 46, 48
    loosen = 1 if pose == "ult" and frame >= 2 else 0
    draw_folded_wings(d, bob, loosen)

    armor_leg(d, 20, 29+bob, stride//2, stride, foot_l, False)
    armor_leg(d, 31, 29+bob, front_stride//2, front_stride, foot_r, True)

    # Skirt plates and the narrow waist retain a readable leg/torso gap.
    poly(d, [(17,28+bob),(34,28+bob),(34,33+bob),(27,35+bob),
             (18,33+bob)], "joint")
    for pts in ([(17,29+bob),(22,30+bob),(22,36+bob),(16,34+bob)],
                [(23,30+bob),(29,30+bob),(29,35+bob),(25,37+bob),(22,34+bob)],
                [(30,29+bob),(35,29+bob),(37,34+bob),(31,36+bob)]):
        poly(d,pts,"white_mid")
    ln(d,[(22,31+bob),(27,32+bob)],"red")

    # Backpack joint, massive angular shoulders, navy chest and split plates.
    poly(d,[(19,16+bob),(31,16+bob),(35,21+bob),(33,30+bob),
            (17,30+bob),(14,22+bob)],"navy")
    poly(d,[(14,17+bob),(21,14+bob),(23,20+bob),(19,27+bob),
            (12,26+bob),(10,21+bob)],"white_dark")
    poly(d,[(13,18+bob),(20,16+bob),(21,20+bob),(17,23+bob),
            (12,22+bob)],"white")
    poly(d,[(30,15+bob),(37,17+bob),(41,22+bob),(37,27+bob),
            (32,25+bob),(28,20+bob)],"white_dark")
    poly(d,[(31,16+bob),(36,18+bob),(39,22+bob),(35,23+bob),
            (30,20+bob)],"white")
    poly(d,[(17,20+bob),(24,22+bob),(23,29+bob),(18,28+bob),
            (15,24+bob)],"white_mid")
    poly(d,[(27,22+bob),(34,19+bob),(36,24+bob),(32,29+bob),
            (27,29+bob)],"white_mid")
    ln(d,[(18,21+bob),(22,23+bob)],"white")
    ln(d,[(30,23+bob),(34,21+bob)],"white")
    poly(d,[(23,21+bob),(28,21+bob),(30,25+bob),(27,29+bob),
            (22,27+bob)],"red_dark")
    poly(d,[(24,22+bob),(28,23+bob),(27,26+bob),(24,26+bob)],"red")
    poly(d,[(24,23+bob),(27,23+bob),(27,25+bob),(24,25+bob)],
         "energy" if pose == "ult" or frame == 2 else "energy_dark")
    ln(d,[(17,25+bob),(20,26+bob)],"gold",2)
    ln(d,[(32,26+bob),(35,24+bob)],"gold",2)

    # One arm balances the saber hilt; the other carries the forward shield.
    if pose == "attack":
        hand = ((17,36),(20,31),(29,24),(21,33))[frame]
        ln(d,[(15,25+bob),hand],"outline",8)
        ln(d,[(15,25+bob),hand],"white_dark",6)
        ln(d,[(15,25+bob),hand],"white_mid",4)
        ln(d,[(15,24+bob),hand],"white",1)
        poly(d,[(hand[0]-2,hand[1]-2),(hand[0]+2,hand[1]-2),
                (hand[0]+3,hand[1]+2),(hand[0]-2,hand[1]+2)],"joint")
        ln(d,[(hand[0],hand[1]),(hand[0]+3,hand[1]-1)],"gold")
    else:
        poly(d,[(13,23+bob),(19,25+bob),(18,34+bob),(14,36+bob),
                (11,31+bob)],"joint")
        poly(d,[(12,27+bob),(18,27+bob),(18,33+bob),(14,35+bob),
                (11,31+bob)],"white_mid")
        ln(d,[(13,28+bob),(17,28+bob)],"white")
        poly(d,[(16,34+bob),(19,34+bob),(19,38+bob),(15,38+bob)],"joint")
        ln(d,[(17,36+bob),(19,36+bob)],"gold") # stored saber hilt
    poly(d,[(35,23+bob),(40,26+bob),(41,34+bob),(37,38+bob),
            (32,32+bob)],"joint")
    poly(d,[(36,25+bob),(40,27+bob),(41,33+bob),(37,35+bob),
            (34,31+bob)],"white_mid")

    # Compact mechanical face, gold V-fin and bright right-facing eyes.
    poly(d,[(20,7+bob),(24,5+bob),(31,6+bob),(35,10+bob),
            (33,17+bob),(25,19+bob),(20,15+bob)],"white_dark")
    poly(d,[(22,7+bob),(25,6+bob),(31,7+bob),(33,10+bob),
            (31,15+bob),(25,16+bob),(21,13+bob)],"white")
    poly(d,[(26,11+bob),(34,10+bob),(33,14+bob),(28,15+bob)],"navy")
    ln(d,[(30,12+bob),(33,12+bob)],"energy_hi" if pose == "ult" else "energy",2)
    poly(d,[(29,15+bob),(32,15+bob),(31,18+bob),(29,17+bob)],"red")
    poly(d,[(25,8+bob),(20,4+bob),(17,2+bob),(21,9+bob),
            (25,11+bob)],"gold")
    poly(d,[(27,8+bob),(32,3+bob),(36,2+bob),(33,10+bob),
            (28,11+bob)],"gold")
    ln(d,[(19,3+bob),(23,8+bob)],"gold_dark")
    ln(d,[(34,3+bob),(30,8+bob)],"gold_dark")

    shield_forward = pose == "skill1"
    sx = (0,0,1,0)[frame] if shield_forward else 0
    shield = [(36+sx,22+bob),(42+sx,20+bob),(47+sx,28+bob),
              (44+sx,39+bob),(39+sx,43+bob),(34+sx,36+bob)]
    poly(d,shield,"navy")
    poly(d,[(38+sx,23+bob),(42+sx,22+bob),(45+sx,29+bob),
            (42+sx,38+bob),(39+sx,40+bob),(36+sx,35+bob)],"white_mid")
    poly(d,[(39+sx,24+bob),(42+sx,23+bob),(44+sx,30+bob),
            (41+sx,35+bob)],"white")
    poly(d,[(37+sx,34+bob),(41+sx,36+bob),(40+sx,40+bob),
            (38+sx,39+bob)],"red")
    ln(d,[(40+sx,27+bob),(42+sx,30+bob)],"blue_hi")
    if shield_forward:
        ln(d,[(43+sx,24+bob),(46+sx,29+bob)],"energy")

    if pose == "skill2":
        saber = [((17,35),(8,27)),((18,35),(7,15)),
                 ((19,35),(38,10)),((18,35),(45,26))][frame]
        ln(d,saber,"energy_dark",5)
        ln(d,saber,"energy",3)
        ln(d,saber,"energy_hi",1)
    if pose == "ult":
        if frame >= 1:
            ln(d,[(11,20+bob),(13,16+bob)],"energy_dark")
            ln(d,[(38,20+bob),(36,16+bob)],"energy_dark")
        if frame >= 3:
            for x,y in ((12,30),(36,30)):
                ln(d,[(x,y),(x-2,y+5)],"energy",2)
    if pose == "hit":
        ln(d,[(13,16),(10,14)],"energy_hi",2)
    return im

def body_frame(tag, i):
    if tag == "dead":
        base = draw_body("idle",0)
        angle = (0,-12,-28,-48,-70,-85)[i]
        fallen = base.rotate(angle, resample=Image.Resampling.NEAREST,
                             center=(26,44), expand=False)
        return fallen
    if tag in ("ult_flight","ult_dive"):
        base = draw_body("ult",3)
        angles = (-42,-40,-38,-36) if tag == "ult_flight" else (-33,-23,-13,-4)
        return base.rotate(angles[i], resample=Image.Resampling.NEAREST,
                           center=(25,27), expand=False)
    if tag == "ult_land":
        base = draw_body("skill1",min(i,3))
        if i < 2:
            base = base.resize((48,52+i*2),Image.Resampling.NEAREST)
            out = Image.new("RGBA",(48,56)); out.alpha_composite(base,(0,56-base.height))
            return out
        return base
    return draw_body("idle" if tag == "hit" else tag,i)

template = Image.open(KIT / "NEW CHAMPION template.png").convert("RGBA")
template_meta = json.loads((KIT / "NEW CHAMPION template.anim.json").read_text())
extra = {"ult_flight":(4,.08), "ult_dive":(4,.08), "ult_land":(4,.10)}
body_sheet = Image.new("RGBA",(template.width,template.height+len(extra)*56))
body_sheet.alpha_composite(template)
body_meta = template_meta["anims"]
for tag, desc in body_meta.items():
    for i,f in enumerate(desc["frames"]):
        body_sheet.alpha_composite(body_frame(tag,i),(f["data"]["x"],f["data"]["y"]))
for row,(tag,(count,duration)) in enumerate(extra.items(),start=8):
    frames=[]
    for i in range(count):
        x,y=i*48,row*56
        body_sheet.alpha_composite(body_frame(tag,i),(x,y))
        frames.append({"duration":duration,"data":{"x":x,"y":y,"w":48,"h":56}})
    body_meta[tag]={"frames":frames}
out=ROOT/"champions";out.mkdir(parents=True,exist_ok=True)
body_sheet.save(out/f"{ID}#sheet.png")
(out/f"{ID}#anim.fanim").write_text(json.dumps({"anims":body_meta},indent=2)+"\n")

def wing_layer(extent=1.0, sweep=0, bright=False):
    im=Image.new("RGBA",(128,96)); d=ImageDraw.Draw(im)
    def tr(pt, mirrored=False):
        x,y=pt
        x=64+(x-64)*extent
        y=40+(y-40)*(.73+.27*extent)+sweep*(abs(x-64)/64)
        return (round(128-x if mirrored else x),round(y))
    def part(pts,color,mirrored=False):
        poly(d,[tr(p,mirrored) for p in pts],color)
    for mirror in (False,True):
        # Outer wing: articulated navy spar under three overlapping armor feathers.
        part([(60,38),(48,25),(10,3),(16,16),(37,34),(56,46)],"navy",mirror)
        part([(51,28),(33,12),(6,4),(14,16),(35,28)],"white",mirror)
        part([(54,34),(38,22),(8,19),(20,28),(41,39)],"white_mid",mirror)
        part([(57,40),(42,31),(15,35),(27,43),(53,47)],"white",mirror)
        part([(48,30),(39,27),(29,31),(41,38)],"white_dark",mirror)
        # Inner wing below the outer one, separated by a navy gap.
        part([(59,43),(45,46),(11,76),(26,70),(51,51)],"navy",mirror)
        part([(54,45),(37,48),(15,70),(29,65),(52,50)],"white",mirror)
        part([(57,43),(38,42),(18,59),(31,60),(53,50)],"white_mid",mirror)
        part([(56,42),(42,40),(29,49),(42,54),(55,48)],"white",mirror)
        part([(54,39),(59,37),(61,46),(56,48)],"joint",mirror)
        for a,b,color in [((34,14),(19,11),"white_dark"),
                          ((38,25),(22,23),"white_dark"),
                          ((44,34),(31,37),"white_dark"),
                          ((38,51),(24,63),"white_dark"),
                          ((41,44),(27,54),"white_dark")]:
            ln(d,[tr(a,mirror),tr(b,mirror)],color)
        ln(d,[tr((57,38),mirror),tr((47,31),mirror)],
           "energy_hi" if bright else "energy_dark")
        ln(d,[tr((57,44),mirror),tr((46,50),mirror)],
           "energy_hi" if bright else "energy_dark")
        part([(56,39),(59,38),(60,41),(57,42)],"gold_dark",mirror)
    return im

def ring(d,r,color="energy",width=2):
    d.ellipse((64-r,51-r//2,64+r,51+r//2),outline=S[color],width=width)

def effect(tag,i,n):
    im=Image.new("RGBA",(128,96));d=ImageDraw.Draw(im)
    if tag=="wings_open": return wing_layer((.88,.90,.92,.90)[i],bright=i==2)
    if tag in ("wings_retract","retract"):
        return wing_layer(.90-.80*(i+1)/n,bright=i<2)
    if tag=="deploy": return wing_layer(.10+.80*(i+1)/n,bright=i>=3)
    if tag=="flight":
        im=wing_layer(.92,sweep=9,bright=True);d=ImageDraw.Draw(im)
        for x in (43,64,85): ln(d,[(x,54),(x-4-i*2,82+i)],"energy",2)
        return im
    if tag=="flare":
        im=wing_layer(1.0,bright=True);d=ImageDraw.Draw(im)
        ring(d,28+i*3,"energy",2)
        return im
    if tag in ("landing","takeoff"):
        r=10+i*15;ring(d,r,"energy_hi" if i<2 else "energy",3)
        for k in range(7):
            a=k*2*math.pi/7;x=64+round(r*math.cos(a));y=51+round(r*.5*math.sin(a))
            ln(d,[(x,y),(x+round(5*math.cos(a)),y+round(3*math.sin(a)))],"white_mid",2)
    elif tag in ("sweep","saber"):
        r=(25 if tag=="sweep" else 14)+i*5
        d.arc((64-r,50-r//2,64+r,50+r//2),200+i*36,480+i*36,fill=S["energy_dark"],width=5)
        d.arc((64-r,50-r//2,64+r,50+r//2),200+i*36,480+i*36,fill=S["energy_hi"],width=2)
    elif tag in ("incoming","zero_aura","ally_aura","protect","fade"):
        ring(d,14+i%3*2,"energy_dark" if tag=="fade" else "energy",2)
        if tag=="incoming":
            poly(d,[(64,27),(58,38),(70,38)],"gold")
    elif tag in ("charge_start","charge_hit","wall_hit"):
        poly(d,[(59,42),(75+i*4,49),(59,56)],"energy")
        if tag=="wall_hit":ring(d,12+i*6,"white",2)
    elif tag=="vulcan":
        r=1+i*2;d.ellipse((65-r,47-r,65+r,47+r),fill=S["gold"])
    return im

vfx_specs={
    "wings_open":(4,.15),"wings_retract":(6,.075),"deploy":(6,.18),
    "flight":(4,.08),"landing":(5,.07),"takeoff":(4,.08),
    "sweep":(4,.08),"flare":(4,.09),"incoming":(4,.14),
    "zero_aura":(4,.14),"ally_aura":(4,.14),"protect":(4,.14),
    "fade":(4,.14),"charge_start":(3,.07),"charge_hit":(3,.07),
    "wall_hit":(4,.07),"saber":(4,.06),"vulcan":(3,.06),
    "retract":(6,.075),
}
vfx_sheet=Image.new("RGBA",(6*128,len(vfx_specs)*96))
vfx_meta={}
for row,(tag,(count,duration)) in enumerate(vfx_specs.items()):
    frames=[]
    for i in range(count):
        x,y=i*128,row*96
        vfx_sheet.alpha_composite(effect(tag,i,count),(x,y))
        frames.append({"duration":duration,"data":{"x":x,"y":y,"w":128,"h":96}})
    vfx_meta[tag]={"frames":frames}
out=ROOT/"vfx";out.mkdir(parents=True,exist_ok=True)
vfx_sheet.save(out/"gundam#sheet.png")
(out/"gundam#anim.fanim").write_text(json.dumps({"anims":vfx_meta},indent=2)+"\n")

def silhouette(im):
    a=im.getchannel("A").point(lambda v:255 if v else 0)
    b=Image.new("RGBA",im.size,(12,17,28,255));b.putalpha(a)
    return b

check=Image.new("RGBA",(280,112),(230,234,239,255))
normal=Image.new("RGBA",(128,96));normal.alpha_composite(body_frame("idle",0),(40,25))
open_=wing_layer(.90);open_.alpha_composite(body_frame("idle",0),(40,25))
check.alpha_composite(silhouette(normal),(5,8))
check.alpha_composite(silhouette(open_),(145,8))
check.save(ROOT/"source"/"silhouette_check.png")

review=Image.new("RGBA",(128*3,96),(39,43,56,255))
for x in range(0,review.width,8):
    for y in range(0,review.height,8):
        if (x//8+y//8)%2:
            ImageDraw.Draw(review).rectangle((x,y,x+7,y+7),fill=(44,48,61,255))
for col,wings in enumerate((None,wing_layer(.9),wing_layer(1.0,bright=True))):
    tile=Image.new("RGBA",(128,96))
    if wings: tile.alpha_composite(wings)
    tile.alpha_composite(body_frame("idle",0),(40,25))
    review.alpha_composite(tile,(col*128,0))
review.resize((1536,384),Image.Resampling.NEAREST).save(ROOT/"source"/"pose_review.png")
print("Built hand-authored sprite-kit body and wing/VFX sheets")
