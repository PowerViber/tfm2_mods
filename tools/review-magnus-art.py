"""Reproducible Slim A pose/facing contact sheet and prior-release comparison."""
from pathlib import Path
from io import BytesIO
import json, subprocess
from PIL import Image, ImageDraw, ImageFont
ROOT=Path(__file__).resolve().parents[1]
PREFIX='mods/tfm2_custom/champions/tfm2_custom_magnus'
new=Image.open(ROOT/(PREFIX+'#sheet.png')).convert('RGBA')
meta=json.loads((ROOT/(PREFIX+'#anim.fanim')).read_text())
old=Image.open(BytesIO(subprocess.check_output(['git','show','af600c8:'+PREFIX+'#sheet.png'],cwd=ROOT))).convert('RGBA')
previous=json.loads(subprocess.check_output(['git','show','af600c8:'+PREFIX+'#anim.fanim'],cwd=ROOT))
def frame(sheet,record):
    r=record['data'];return sheet.crop((r['x'],r['y'],r['x']+r['w'],r['y']+r['h']))
for tag,a in meta['anims'].items():
    b=previous['anims'][tag]
    assert len(a['frames'])==len(b['frames']),tag
    assert [f['duration'] for f in a['frames']]==[f['duration'] for f in b['frames']],tag
    assert all(f['data']['w']==48 and f['data']['h']==56 for f in a['frames']),tag
font=ImageFont.truetype('C:/Windows/Fonts/segoeui.ttf',14)
tags=list(meta['anims'])
im=Image.new('RGB',(1050,64+len(tags)*268),'#111b2c');d=ImageDraw.Draw(im)
d.text((18,12),'SLIM A · Every pose / facing · established 48 × 56 canvas',fill='#f0d49c',font=font)
for row,tag in enumerate(tags):
    y=54+row*268;d.text((12,y),tag,fill='#ded8cb',font=font)
    for i,f in enumerate(meta['anims'][tag]['frames']):
        body=frame(new,f)
        for flip in (False,True):
            tile=body.transpose(Image.Transpose.FLIP_LEFT_RIGHT) if flip else body
            tile=tile.resize((96,112),Image.Resampling.NEAREST)
            im.paste(tile,(170+i*104,y+20+flip*118),tile)
im.save(ROOT/'docs/magnus-poses.png')
compare=Image.new('RGB',(720,300),'#111b2c');d=ImageDraw.Draw(compare)
for i,(sheet,m,label) in enumerate([(old,previous,'Previous main art'),(new,meta,'Slim A · main art')]):
    d.text((25+i*350,16),label,fill='#f0d49c',font=font)
    tile=frame(sheet,m['anims']['idle']['frames'][0]).resize((192,224),Image.Resampling.NEAREST)
    compare.paste(tile,(40+i*350,50),tile)
compare.save(ROOT/'docs/magnus-slim-comparison.png')
print('PASS: all animation names, frame counts, timings and canvases preserved; pose and comparison sheets written.')
