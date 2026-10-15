"""Run the real Mod Manager against an isolated full-package installation fixture."""
from pathlib import Path
import json,shutil,subprocess
ROOT=Path(__file__).resolve().parents[1]
fixture=ROOT/'backups/magnus-manager-fixture';fixture.mkdir(parents=True,exist_ok=True)
(fixture/'bundle.game_data').touch()
shutil.copytree(ROOT/'mods',fixture/'mods',dirs_exist_ok=True)
config=fixture/'config/game';config.mkdir(parents=True,exist_ok=True)
(config/'mods.json').write_text(json.dumps({'enabled_mods':['tfm2_custom','tfm2_custom_ai']}))
cfg=ROOT/'manager.cfg';before=cfg.read_bytes() if cfg.exists() else None
try:
    result=subprocess.run([str(ROOT/'TFM2 Mod Manager.exe'),'--check','--yes','--game',str(fixture)],cwd=ROOT,capture_output=True,text=True)
finally:
    if before is not None:cfg.write_bytes(before)
    elif cfg.exists():cfg.unlink()
lines=[s.strip() for s in result.stdout.splitlines() if any(x in s for x in ['champion files compared','native (tfm2_custom_ai)','champions (tfm2_custom)','UP TO DATE:'])]
report={'scope':'Isolated full-package installation fixture; no game was run from this directory','exitCode':result.returncode,'result':lines}
(ROOT/'docs/magnus-manager-package.json').write_text(json.dumps(report,indent=2)+'\n')
print('\n'.join(lines))
assert result.returncode==0,result.stdout+result.stderr
