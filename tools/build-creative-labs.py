"""Build shared labs without installing anything in the game."""
from pathlib import Path
import subprocess,shutil
root=Path(__file__).resolve().parents[1]
for name in ['steve','universal','frieren','gojo']:
 cwd=root/f'native/{name}_lab'
 for args in [['test','--release'],['build','--release'],['build','--release','--target','wasm32-unknown-unknown','--lib']]:
  subprocess.run(['rtk','proxy','cargo','+stable-x86_64-pc-windows-gnu',*args],cwd=cwd,check=True)
 shutil.copyfile(cwd/f'target/wasm32-unknown-unknown/release/{name}_lab.wasm',root/f'editor/{name}.wasm')
