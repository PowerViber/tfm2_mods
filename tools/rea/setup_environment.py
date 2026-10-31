"""Acquire pinned analysis tools into a private cache; never edits PATH/MCP/game files."""
import argparse,hashlib,json,os,subprocess,sys,urllib.request,venv,zipfile
from pathlib import Path
NAME='ghidra_12.1.4_PUBLIC_20260921.zip'
URL='https://github.com/NationalSecurityAgency/ghidra/releases/download/Ghidra_12.1.4_build/'+NAME
DIGEST='ddac49f903da9d5bac833e5cc79395098b9c33cfd3279be5f31bd00387d2d4db'
def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--cache',type=Path,default=Path.home()/'.cache/tfm2-rea');a=p.parse_args()
    a.cache.mkdir(parents=True,exist_ok=True);archive=a.cache/NAME
    if not archive.exists():
        temporary=a.cache/(NAME+'.download');urllib.request.urlretrieve(URL,temporary)
        if hashlib.file_digest(temporary.open('rb'),'sha256').hexdigest()!=DIGEST:raise RuntimeError('Official archive digest mismatch; retained download for inspection')
        temporary.rename(archive)
    if hashlib.file_digest(archive.open('rb'),'sha256').hexdigest()!=DIGEST:raise RuntimeError('Cached Ghidra digest mismatch')
    ghidra=a.cache/'ghidra_12.1.4_PUBLIC'
    if not(ghidra/'support/analyzeHeadless.bat').exists():
        with zipfile.ZipFile(archive)as z:
            if not all((a.cache/n).resolve().is_relative_to(a.cache.resolve())for n in z.namelist()):raise RuntimeError('Unsafe archive member')
            z.extractall(a.cache)
    python=a.cache/'python/Scripts/python.exe'
    if not python.exists():venv.create(a.cache/'python',with_pip=True)
    subprocess.run(['rtk','proxy',str(python),'-m','pip','install','iced-x86==1.21.0'],check=True)
    report={'ghidraVersion':'12.1.4','url':URL,'archiveSha256':DIGEST,'ghidraDirectory':str(ghidra),
        'python':str(python),'decoder':'iced-x86==1.21.0','persistentEnvironmentChanged':False,'mcpConfigChanged':False}
    (a.cache/'environment.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
if __name__=='__main__':main()
