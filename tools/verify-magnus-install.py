"""Record byte and structured-data differences without altering the game."""
from pathlib import Path
import json,hashlib,sys
ROOT=Path(__file__).resolve().parents[1];game=Path(sys.argv[1]).resolve()
rows=[]
for src in (ROOT/"mods").rglob("*"):
    if not src.is_file() or "source" in src.relative_to(ROOT/"mods").parts or src.suffix not in [".fanim",".data_champion",".png",".i18n",".mod_info",".dll"]:continue
    rel=src.relative_to(ROOT);other=game/rel
    if not other.is_file():rows.append({"path":str(rel),"status":"missing"});continue
    a=src.read_bytes();b=other.read_bytes()
    if a==b:continue
    status="content"
    if src.suffix in [".fanim",".data_champion",".i18n",".mod_info"]:
        try:
            if json.loads(a.decode("utf-8-sig"))==json.loads(b.decode("utf-8-sig")):status="formatting"
        except (ValueError,UnicodeDecodeError):pass
    rows.append({"path":str(rel),"status":status})
p=ROOT/"docs/magnus-install-comparison.json";p.write_text(json.dumps(rows,indent=2)+"\n")
print(json.dumps({s:sum(r["status"]==s for r in rows) for s in ["missing","formatting","content"]}))
for r in rows:
    if r["status"]!="formatting":print(r["status"],r["path"])
