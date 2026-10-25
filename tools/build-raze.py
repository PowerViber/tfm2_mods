"""Build the same Rust Raze core for native verification and browser practice."""
from pathlib import Path
import subprocess,shutil
root=Path(__file__).resolve().parents[1]
cwd=root/'native/raze_lab'
for args in [['test','--release'],['build','--release'],['build','--release','--target','wasm32-unknown-unknown','--lib']]:
 subprocess.run(['rtk','proxy','cargo','+stable-x86_64-pc-windows-gnu',*args],cwd=cwd,check=True)
shutil.copyfile(cwd/'target/wasm32-unknown-unknown/release/raze_lab.wasm',root/'editor/raze.wasm')
