"""Mod Manager check of only the diagnostic package in an isolated fixture."""
import hashlib,json,os,shutil,subprocess,tempfile
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
OUT=ROOT/'docs/contender-realms/windows'

def main():
    package=ROOT/'experiments/contender-realm-observer/target/packages/tfm2_realm_observer'
    if not(package/'tfm2_realm_observer.dll').exists():raise SystemExit('Build the diagnostic package first')
    fixture=Path(tempfile.mkdtemp(prefix='tfm2-realm-manager-'))
    (fixture/'bundle.game_data').touch()
    # The repository's manager compares its whole production pack, rather than
    # offering a generic single-mod verifier. Supply the unchanged baseline too.
    shutil.copytree(ROOT/'mods',fixture/'mods')
    shutil.copytree(package,fixture/'mods/tfm2_realm_observer')
    config=fixture/'config/game';config.mkdir(parents=True)
    enabled=['base','tfm2_realm_observer',*[p.name for p in(ROOT/'mods').iterdir()if(p/'mod.mod_info').is_file()]]
    (config/'mods.json').write_text(json.dumps({'enabled_mods':sorted(set(enabled))}))
    cfg=ROOT/'manager.cfg';before=cfg.read_bytes()if cfg.exists()else None
    try:
        result=subprocess.run(['rtk','proxy',str(ROOT/'TFM2 Mod Manager.exe'),'--check','--yes','--game',str(fixture)],
            cwd=ROOT,env=dict(os.environ,APPDATA=str(fixture/'appdata')),capture_output=True,text=True,encoding='utf8',errors='replace')
    finally:
        if before is not None:cfg.write_bytes(before)
        elif cfg.is_file():cfg.unlink()
    OUT.mkdir(parents=True,exist_ok=True)
    (OUT/'mod-manager.txt').write_text(result.stdout+result.stderr,encoding='utf8')
    files=[p for p in package.rglob('*')if p.is_file()]
    unchanged=all((fixture/'mods/tfm2_realm_observer'/p.relative_to(package)).read_bytes()==p.read_bytes()for p in files)
    production=[p for p in(ROOT/'mods').rglob('*')if p.is_file()]
    production_unchanged=all((fixture/'mods'/p.relative_to(ROOT/'mods')).read_bytes()==p.read_bytes()for p in production)
    report={'scope':'Manager checks unchanged production baseline in an isolated fixture; diagnostic files are separately compared. Not diagnostic ABI or live host certification.',
        'exitCode':result.returncode,'filesCompared':len(files),'packageUnchanged':unchanged,
        'productionFilesCompared':len(production),'productionUnchanged':production_unchanged,
        'dllSha256':hashlib.sha256((package/'tfm2_realm_observer.dll').read_bytes()).hexdigest(),
        'installedGameModified':False,'realCareerModified':False,'nativeCodePermissionGranted':False}
    (OUT/'mod-manager.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
    if result.returncode or not unchanged or not production_unchanged:raise SystemExit('Diagnostic package check failed; inspect mod-manager.txt')
if __name__=='__main__':main()
