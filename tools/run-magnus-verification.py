"""Persist native parity vectors and the 24 development / 12 holdout reports."""
from pathlib import Path
import gzip,json,subprocess
ROOT=Path(__file__).resolve().parents[1]
exe=ROOT/"native/magnus_lab/target/x86_64-pc-windows-gnu/release/vectors.exe"
v=subprocess.run([str(exe)],check=True,capture_output=True).stdout
frames=[json.loads(l) for l in v.splitlines()]
(ROOT/"editor/magnus-vectors.jsonl.gz").write_bytes(gzip.compress(v,mtime=0))
print("Saved",len(frames),"full-state native frames.",flush=True)
s=subprocess.run([str(exe),"--scenarios"],check=True,capture_output=True).stdout
scenarios=json.loads(s)
assert len(scenarios)==36 and sum(x["holdout"] for x in scenarios)==12
(ROOT/"docs/magnus-scenarios.json").write_text(json.dumps(scenarios,indent=2)+"\n",encoding="utf-8")
for holdout in [False,True]:
    group=[x for x in scenarios if x["holdout"]==holdout]
    print("Holdout" if holdout else "Development",len(group),"scenarios; mean survival",round(sum(x["pieceSurvival"] for x in group)/len(group),3),"damage",sum(x["rawDamage"] for x in group),"checkmates",sum(x["checkmates"] for x in group),"recalls",sum(x["challengeCompletions"] for x in group),"max step",max(x["maxStepUs"] for x in group),"us",flush=True)
