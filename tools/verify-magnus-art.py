"""Check stored piece artwork against the supported carrier's active poses."""
from pathlib import Path
import importlib.util,json,sys
from PIL import Image
ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/"Claude outputs/unified_theory"))
spec=importlib.util.spec_from_file_location("cosmic_art",ROOT/"Claude outputs/unified_theory/cosmic_art.py")
cosmic=importlib.util.module_from_spec(spec);spec.loader.exec_module(cosmic)
sheet=Image.open(ROOT/"mods/tfm2_custom/vfx/magnus_pieces#sheet.png").convert("RGBA")
anims=json.loads((ROOT/"mods/tfm2_custom/vfx/magnus_pieces#anim.fanim").read_text())["anims"]
cases=0;uncovered=[]
for tag,anim in anims.items():
    for fr in anim["frames"]:
        r=fr["data"];mask=sheet.crop((r["x"],r["y"],r["x"]+r["w"],r["y"]+r["h"])).getchannel("A")
        for name,host in cosmic.crystal_host_frames():
            aligned=Image.new("RGBA",(64,64));aligned.alpha_composite(host,(8,0))
            for flip in [False,True]:
                h=aligned.getchannel("A")
                if flip:h=h.transpose(Image.Transpose.FLIP_LEFT_RIGHT)
                points=[(x,y) for y in range(64) for x in range(64) if h.getpixel((x,y)) and mask.getpixel((x,y))!=255]
                if points:uncovered.append((tag,name,flip,points[:3]))
                cases+=1
assert not uncovered,uncovered[:8]
print("PASS:",cases,"carrier-frame, piece-frame and facing combinations covered.")
