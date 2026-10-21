"""Run Mod Manager against a complete, isolated installation fixture."""
from pathlib import Path
import hashlib,json,shutil,subprocess,tempfile
ROOT=Path(__file__).resolve().parents[1];backup=ROOT/'backups';backup.mkdir(exist_ok=True)
fixture=Path(tempfile.mkdtemp(prefix='gojo-manager-',dir=backup)).resolve();assert fixture.is_relative_to(backup.resolve());(fixture/'bundle.game_data').touch();shutil.copytree(ROOT/'mods',fixture/'mods');config=fixture/'config/game';config.mkdir(parents=True);(config/'mods.json').write_text(json.dumps({'enabled_mods':['tfm2_custom','tfm2_custom_ai','tfm2_jjk']}))
cfg=ROOT/'manager.cfg';before=cfg.read_bytes() if cfg.exists() else None
try:result=subprocess.run(['rtk','proxy',str(ROOT/'TFM2 Mod Manager.exe'),'--check','--yes','--game',str(fixture)],cwd=ROOT,capture_output=True,text=True,encoding='utf-8',errors='replace')
finally:
 if before is not None:cfg.write_bytes(before)
 elif cfg.exists():cfg.unlink()
output=result.stdout+result.stderr;out=ROOT/'docs/gojo-results';out.mkdir(parents=True,exist_ok=True);(out/'mod-manager.txt').write_text(output,encoding='utf-8')
lines=[s.strip() for s in output.splitlines() if any(x in s for x in ['champion files compared','tfm2_custom_ai','tfm2_jjk','UP TO DATE:'])]
report={'scope':'Isolated full-package installation fixture. Installed game and careers unchanged.','exitCode':result.returncode,'packagedDllSha256':hashlib.sha256((fixture/'mods/tfm2_custom_ai/tfm2_custom_ai.dll').read_bytes()).hexdigest(),'result':lines};(out/'mod-manager.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2));assert result.returncode==0,output;assert 'NOT UP TO DATE:' not in output,output
