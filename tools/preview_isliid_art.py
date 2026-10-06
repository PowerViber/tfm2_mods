"""Create a review sheet from the exact native-size game frames."""
from pathlib import Path
import json
from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parents[1]
VFX = ROOT / "mods" / "tfm2_custom" / "vfx"
OUT = ROOT / "editor" / "isliid-progression-preview.png"
BADGE_OUT = ROOT / "editor" / "isliid-badge-progression.png"
SWORDS = ("skylight", "terra", "darkbringer", "gale", "blood", "rift", "emperor")
RANKS = ("BEARER", "SQUIRE", "ENGRAVER", "TACTICIAN", "SWORDMASTER", "REGENT", "SOVEREIGN", "IMPERIAL")

def load(name):
    sheet = Image.open(VFX / f"{name}#sheet.png").convert("RGBA")
    anims = json.loads((VFX / f"{name}#anim.fanim").read_text(encoding="utf-8"))["anims"]
    return sheet, anims

def frame(source, tag, phase=0):
    sheet, anims = source
    r = anims[tag]["frames"][phase]["data"]
    return sheet.crop((r["x"], r["y"], r["x"]+r["w"], r["y"]+r["h"]))

def main():
    sword = load("swords8")
    badge = load("badges8")
    aura = load("auras8")
    w, h = 8*112+120, 7*94+160
    canvas = Image.new("RGBA", (w,h), "#16202d")
    draw = ImageDraw.Draw(canvas)
    font = ImageFont.load_default()
    draw.text((15,9), "ISLIID  /  EIGHT MASTERY TIERS  /  GAME-SIZE PIXEL ART", font=font, fill="#fff1c7")
    for rank,name in enumerate(RANKS):
        x=120+rank*112
        draw.text((x+5,31),name[:13],font=font,fill="#b9dced")
        btag=f"rank{rank}" if rank<7 else "imperial1"
        canvas.alpha_composite(frame(badge,btag,0),(x+24,44))
        for i,s in enumerate(SWORDS):
            y=145+i*94
            canvas.alpha_composite(frame(sword,f"{s}_rank{rank}_orbit",0),(x+12,y))
            canvas.alpha_composite(frame(aura,f"aura_{i}_rank{rank}_ally",4),(x+45,y))
    for i,s in enumerate(SWORDS):
        draw.text((10,174+i*94),s.upper(),font=font,fill="#f2dda7")
    draw.text((14,h-38),"Each sword and aura plays eight authored frames. Imperial 1-10 use numbered badge art.",font=font,fill="#9fb7c9")
    canvas.convert("RGB").save(OUT)
    gallery=Image.new("RGBA",(920,330),"#16202d")
    gd=ImageDraw.Draw(gallery)
    gd.text((16,8),"ISLIID MASTERY  /  NATIVE 48x96 PIXEL FRAMES  /  4x DETAIL",font=font,fill="#fff1c7")
    labels=list(RANKS[:7])+[f"IMPERIAL {n}" for n in range(10,0,-1)]
    tags=[f"rank{i}" for i in range(7)]+[f"imperial{n}" for n in range(10,0,-1)]
    for j,(label,tag) in enumerate(zip(labels,tags)):
        row=j//9;col=j%9;x=12+col*101;y=35+row*130
        sample=frame(badge,tag,0)
        gallery.alpha_composite(sample,(x,y+13))
        gallery.alpha_composite(sample.resize((96,192),Image.Resampling.NEAREST).crop((48,0,96,100)),(x+40,y+9))
        gd.text((x,y+113),label[:14],font=font,fill="#b9dced")
    gd.text((16,310),"Imperial #1 has the fullest crown and prismatic engraving rays.",font=font,fill="#e8c9f4")
    gallery.convert("RGB").save(BADGE_OUT)
    print(OUT, BADGE_OUT)

if __name__ == "__main__": main()
