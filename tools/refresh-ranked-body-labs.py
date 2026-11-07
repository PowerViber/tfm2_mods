"""Refresh presentation metadata from current canonical packed assets only."""
from pathlib import Path
import json
ROOT=Path(__file__).resolve().parents[1]
def read(p):return json.loads(p.read_text(encoding='utf8'))
def write(p,j):p.write_text(json.dumps(j,indent=2,ensure_ascii=False)+'\n',encoding='utf8')
path=ROOT/'editor/magnus-art.json';data=read(path)
for name in data:
 for folder in ['champions','vfx']:
  p=ROOT/'mods/tfm2_custom'/folder/(name+'#anim.fanim')
  if p.exists():data[name]=read(p)['anims'];break
path.write_text(json.dumps(data,separators=(',',':'),ensure_ascii=False)+'\n',encoding='utf8')
path=ROOT/'editor/vesper-art-preview.json';data=read(path)
for tag,entry in data['assets'].items():
 p=ROOT/'mods/tfm2_custom/vfx'/(entry['sheet']+'#anim.fanim')
 anims=read(p)['anims']
 if tag in anims:entry['frames']=anims[tag]['frames']
data['nativeBody']=read(ROOT/'mods/tfm2_custom/champions/tfm2_custom_vesper#anim.fanim')['anims']
data['nativeBodyMode']='visible-base-fallback'
write(path,data)
print('Lab presentation maps match native packed assets; no gameplay data regenerated')
