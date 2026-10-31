"""Verify the art candidate against a disposable Mod Manager game directory."""
from pathlib import Path
import hashlib,json,shutil,subprocess,tempfile,os
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/vader-aegis-scribble-review'
backup=ROOT/'backups';backup.mkdir(exist_ok=True)
fixture=Path(tempfile.mkdtemp(prefix='character-art-manager-',dir=backup)).resolve();assert fixture.is_relative_to(backup.resolve())
(fixture/'bundle.game_data').touch();shutil.copytree(ROOT/'mods',fixture/'mods');config=fixture/'config/game';config.mkdir(parents=True)
enabled=sorted(p.name for p in(ROOT/'mods').iterdir()if(p/'mod.mod_info').is_file());(config/'mods.json').write_text(json.dumps({'enabled_mods':enabled}),encoding='utf8')
cfg=ROOT/'manager.cfg';before=cfg.read_bytes()if cfg.exists()else None
try:
 result=subprocess.run(['rtk','proxy',str(ROOT/'TFM2 Mod Manager.exe'),'--check','--yes','--game',str(fixture)],cwd=ROOT,env=dict(os.environ,APPDATA=str(fixture/'appdata')),capture_output=True,text=True,encoding='utf8',errors='replace')
finally:
 if before is not None:cfg.write_bytes(before)
 elif cfg.is_file():cfg.unlink()
text=result.stdout+result.stderr;(OUT/'mod-manager.txt').write_text(text,encoding='utf8')
files=[p for p in(ROOT/'mods').rglob('*')if p.is_file()];different=[p.relative_to(ROOT).as_posix()for p in files if(fixture/p.relative_to(ROOT)).read_bytes()!=p.read_bytes()]
report={'scope':'Art candidate in an isolated fixture; no real installation or career migration','isolatedAppData':True,'liveGameLogExamined':False,'exitCode':result.returncode,'filesCompared':len(files),'differentOrMissing':different,'nativeDllSha256':hashlib.sha256((fixture/'mods/tfm2_custom_ai/tfm2_custom_ai.dll').read_bytes()).hexdigest()}
(OUT/'mod-manager.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2));assert result.returncode==0 and not different and 'NOT UP TO DATE:'not in text,text
