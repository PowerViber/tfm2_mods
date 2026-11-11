"""Read-only Mod Manager comparison against an isolated disposable installation."""
from pathlib import Path
import hashlib,json,os,shutil,subprocess,tempfile
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/round-165-art-review'
backup=ROOT/'backups';backup.mkdir(exist_ok=True)
fixture=Path(tempfile.mkdtemp(prefix='round-165-art-manager-',dir=backup)).resolve()
assert fixture.is_relative_to(backup.resolve())
(fixture/'bundle.game_data').touch();shutil.copytree(ROOT/'mods',fixture/'mods')
config=fixture/'config/game';config.mkdir(parents=True)
enabled=sorted(p.name for p in (ROOT/'mods').iterdir() if (p/'mod.mod_info').is_file())
(config/'mods.json').write_text(json.dumps({'enabled_mods':enabled}),encoding='utf-8')
cfg=ROOT/'manager.cfg';before=cfg.read_bytes() if cfg.exists() else None
try:
    r=subprocess.run(['rtk','proxy',str(ROOT/'TFM2 Mod Manager.exe'),'--check','--yes','--game',str(fixture)],cwd=ROOT,env=dict(os.environ,APPDATA=str(fixture/'appdata')),capture_output=True,text=True,encoding='utf-8',errors='replace')
finally:
    if before is not None:cfg.write_bytes(before)
    elif cfg.is_file():cfg.unlink()
text=r.stdout+r.stderr;(OUT/'mod-manager.txt').write_text(text,encoding='utf-8')
files=[p for p in (ROOT/'mods').rglob('*') if p.is_file()]
different=[p.relative_to(ROOT).as_posix() for p in files if (fixture/p.relative_to(ROOT)).read_bytes()!=p.read_bytes()]
report={'scope':'Disposable fixture comparison only; no real game installation or career migration','exit_code':r.returncode,'files_compared':len(files),'different_or_missing':different,'isolated_appdata':True,
        'existing_packaged_dll_sha256':hashlib.sha256((ROOT/'mods/tfm2_custom_ai/tfm2_custom_ai.dll').read_bytes()).hexdigest(),
        'release_packaging':'Reproduced DLL packaged in the repository; fixture validation does not prove live registration/rendering'}
(OUT/'mod-manager.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report));assert r.returncode==0 and not different and 'NOT UP TO DATE:' not in text,text
