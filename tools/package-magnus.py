"""Verify assets, package a tested DLL, and optionally stage the reversible knight probe."""
from pathlib import Path
import argparse,hashlib,json,shutil,time
from PIL import Image
ROOT=Path(__file__).resolve().parents[1]
p=argparse.ArgumentParser();p.add_argument("--install-probe",type=Path);p.add_argument("--install-full",type=Path);args=p.parse_args()
built=ROOT/"native/tfm2_custom_ai/target/x86_64-pc-windows-gnu/release/tfm2_custom_ai.dll"
dest=ROOT/"mods/tfm2_custom_ai/tfm2_custom_ai.dll"
assert built.is_file(),"Build the GNU release DLL first"
digest=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
if not dest.is_file() or digest(built)!=digest(dest):
    temp=dest.with_suffix(".dll.new")
    shutil.copy2(built,temp)
    temp.replace(dest)
assert digest(built)==digest(dest)
c=json.loads((ROOT/"mods/tfm2_custom/champion/tfm2_custom_magnus.data_champion").read_text())
decoded=0;atlases=[]
for f in [ROOT/"mods/tfm2_custom/champions/tfm2_custom_magnus#sheet.png",*sorted((ROOT/"mods/tfm2_custom/vfx").glob("magnus_*#sheet.png"))]:
    with Image.open(f) as im:
        w,h=im.size;assert w<=2048 and h<=2048,(f,w,h);decoded+=w*h*4
        atlas=json.loads(f.with_name(f.name.replace("#sheet.png","#anim.fanim")).read_text())
        for tag,anim in atlas["anims"].items():
            assert anim["frames"],tag
            for fr in anim["frames"]:
                r=fr["data"];assert fr["duration"]>0 and 0<=r["x"]<=w-r["w"] and 0<=r["y"]<=h-r["h"],(tag,r)
        atlases.append({"file":str(f.relative_to(ROOT)),"width":w,"height":h})
assert decoded<=32*1024*1024,decoded
for v in c["view_buffs"]+c["view_effects"]:
    f=ROOT/"mods"/(v["anim"].removeprefix("asset/")+"#anim.fanim")
    assert v["tag"] in json.loads(f.read_text())["anims"],v
native=json.loads((ROOT/"native/tfm2_custom_ai/mod.mod_info").read_text())
assert native==json.loads((ROOT/"mods/tfm2_custom_ai/mod.mod_info").read_text())
pack=json.loads((ROOT/"mods/tfm2_custom/mod.mod_info").read_text())
assert any(d["mod_id"]=="tfm2_custom_ai" and d["version"]==">="+native["version"] for d in pack["dependencies"])
report={"nativeVersion":native["version"],"championVersion":pack["version"],"dllSha256":digest(dest),"decodedArtworkBytes":decoded,"atlases":atlases,"gameAcceptance":"Not observed"}
(ROOT/"docs/magnus-package.json").write_text(json.dumps(report,indent=2)+"\n",encoding="utf-8")
print("DLL SHA-256:",report["dllSha256"]);print("Artwork decoded:",round(decoded/1024/1024,2),"MiB,",len(atlases),"atlases")
game=args.install_probe or args.install_full
if game:
    game=game.resolve();assert (game/"bundle.game_data").is_file(),"Not a TFM2 game directory"
    import subprocess
    tasks=subprocess.run(["tasklist","/FO","CSV","/NH"],capture_output=True,text=True).stdout.lower()
    assert "teamfightmanager2" not in tasks and "teamfight manager2" not in tasks,"Close the game before staging"
    backup=ROOT/"backups"/("magnus-"+str(time.time_ns()));backup.mkdir(parents=True)
    files=[dest,ROOT/"mods/tfm2_custom_ai/mod.mod_info",ROOT/"mods/tfm2_custom/mod.mod_info",ROOT/"mods/tfm2_custom/text/champion.i18n",ROOT/"mods/tfm2_custom/champion/tfm2_custom_magnus.data_champion"]
    files += list((ROOT/"mods/tfm2_custom/champions").glob("tfm2_custom_magnus#*"))
    files += list((ROOT/"mods/tfm2_custom/vfx").glob("magnus_*#*"))
    # Mod Manager compares bytes. Normalise only files whose parsed data already
    # matches, preserving every gameplay difference and backing up the old bytes.
    for src in (ROOT/"mods").rglob("*"):
        if not src.is_file() or src in files or src.suffix not in [".fanim",".data_champion",".i18n",".mod_info"]:continue
        if "source" in src.relative_to(ROOT/"mods").parts:continue
        target=game/src.relative_to(ROOT)
        if not target.is_file() or src.read_bytes()==target.read_bytes():continue
        try:
            if json.loads(src.read_text(encoding="utf-8-sig"))==json.loads(target.read_text(encoding="utf-8-sig")):files.append(src)
        except (ValueError,UnicodeDecodeError):pass
    manifest=[]
    for src in files:
        rel=src.relative_to(ROOT);target=game/rel
        assert target.resolve().is_relative_to(game/"mods")
        old=backup/rel;entry={"path":str(rel),"existed":target.is_file()}
        if target.is_file():old.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(target,old)
        target.parent.mkdir(parents=True,exist_ok=True)
        temp_target=target.with_name(target.name+".new")
        shutil.copy2(src,temp_target);temp_target.replace(target);manifest.append(entry)
    if args.install_probe:
        c["passive"]["params"]={"knight_probe":1}
        (game/"mods/tfm2_custom/champion/tfm2_custom_magnus.data_champion").write_text(json.dumps(c,indent=2)+"\n",encoding="utf-8")
    (backup/"manifest.json").write_text(json.dumps({"game":str(game),"files":manifest,"probe":bool(args.install_probe)},indent=2)+"\n")
    print("Staged", "single-knight probe" if args.install_probe else "full Magnus", "in",game);print("Previous game files backed up:",backup)
