"""Keep the last transport-v2 lab playable without adding installed mod artwork."""
from pathlib import Path
import subprocess, json, hashlib
ROOT=Path(__file__).resolve().parents[1]
BASE='d0b08660813e5bc8178c8a7cb88f0f9528eb51b2'
def blob(path):
    return subprocess.check_output(['rtk','proxy','git','show',BASE+':'+path],cwd=ROOT)
def main():
    replacements={'gojo.wasm':'gojo-v2.wasm','gojo-lab.js':'gojo-v2-lab.js','gojo-data.js':'gojo-v2-data.js','gojo-visuals.json':'gojo-v2-visuals.json','gojo-v2-fx':'gojo-v2-archive-fx'}
    for old,new in [('gojo.html','gojo-v2.html'),('gojo-lab.js','gojo-v2-lab.js'),('gojo-data.js','gojo-v2-data.js'),('gojo-visuals.json','gojo-v2-visuals.json')]:
        text=blob('editor/'+old).decode('utf-8-sig')
        for a,b in replacements.items(): text=text.replace(a,b)
        text=text.replace('Gojo Lab','Gojo Lab · archived v2')
        (ROOT/'editor'/new).write_text(text,encoding='utf-8')
    for old,new in [('editor/gojo.wasm','editor/gojo-v2.wasm'),('mods/tfm2_jjk/vfx/gojo_v2#sheet.png','editor/gojo-v2-archive-fx.png'),('mods/tfm2_jjk/vfx/gojo_v2#anim.fanim','editor/gojo-v2-archive-fx.fanim')]:
        (ROOT/new).write_bytes(blob(old))
    print('Frozen transport v2 at '+BASE)
if __name__=='__main__':main()
