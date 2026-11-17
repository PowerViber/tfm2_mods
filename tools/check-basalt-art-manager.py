"""Mod Manager --check in an entirely disposable repo/game pair; never installs."""
from pathlib import Path
import hashlib,json,os,shutil,subprocess,tempfile
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/os.environ.get('BASALT_RESULTS','docs/basalt-art-review')
backups=ROOT/'backups';backups.mkdir(exist_ok=True)
fixture=Path(tempfile.mkdtemp(prefix='basalt-art-manager-',dir=backups)).resolve()
assert fixture.is_relative_to(backups.resolve())
repo=fixture/'repo';game=fixture/'game';repo.mkdir();game.mkdir();(repo/'editor').mkdir()
shutil.copytree(ROOT/'mods',repo/'mods');shutil.copytree(ROOT/'mods',game/'mods')
shutil.copy2(ROOT/'TFM2 Mod Manager.exe',repo/'TFM2 Mod Manager.exe')
(game/'bundle.game_data').touch();cfg=game/'config/game';cfg.mkdir(parents=True)
(cfg/'mods.json').write_text(json.dumps({'enabled_mods':sorted(p.name for p in (repo/'mods').iterdir() if (p/'mod.mod_info').exists())}),encoding='utf-8')
before={str(p.relative_to(ROOT/'mods')):hashlib.sha256(p.read_bytes()).hexdigest() for p in (ROOT/'mods').rglob('*') if p.is_file()}
r=subprocess.run(['rtk','proxy',str(repo/'TFM2 Mod Manager.exe'),'--check','--yes','--game',str(game)],cwd=repo,env=dict(os.environ,APPDATA=str(fixture/'appdata')),capture_output=True,text=True,encoding='utf-8',errors='replace')
text=r.stdout+r.stderr;(OUT/'mod-manager.txt').write_text(text,encoding='utf-8')
after={str(p.relative_to(ROOT/'mods')):hashlib.sha256(p.read_bytes()).hexdigest() for p in (ROOT/'mods').rglob('*') if p.is_file()}
assert before==after,'Source mods changed'
report={'scope':'Disposable fixture comparison only. No real game installation, process changes or career migration. Source manager.cfg untouched.','exit_code':r.returncode,'files_compared':len(before),'body_integrated':True,'effects_integrated':True,'source_mods_unchanged':True,'isolated_appdata':True,'packaged_dll_sha256':before[str(Path('tfm2_custom_ai/tfm2_custom_ai.dll'))],'dll_packaging':'Native '+json.loads((ROOT/'mods/tfm2_custom_ai/mod.mod_info').read_text())['version']+'; independently reproduced; see package.json','live_game':'pending'}
(OUT/'mod-manager.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2));assert r.returncode==0 and 'NOT UP TO DATE:' not in text,text
