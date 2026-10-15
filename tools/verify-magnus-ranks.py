"""Verify compact Alchemist coverage, ranked poses, centered anchors and art budget."""
from pathlib import Path
import json,sys
from PIL import Image,ImageChops,ImageDraw,ImageFont
ROOT=Path(__file__).resolve().parents[1];sys.path.insert(0,str(ROOT/'Claude outputs/magnus'));import rank_art as R
MOD=ROOT/'mods/tfm2_custom'
def unpack(name,tag,index=0,size=(48,56)):
 p=MOD/'vfx'/(name+'#anim.fanim');r=json.loads(p.read_text())['anims'][tag]['frames'][index]['data'];im=Image.open(p.with_name(name+'#sheet.png')).convert('RGBA').crop((r['x'],r['y'],r['x']+r['w'],r['y']+r['h']));out=Image.new('RGBA',size);out.alpha_composite(im,((size[0]-im.width)//2,(size[1]-im.height)//2));return out
mask=R.carrier_hull();cases=0
for look in range(11):
 for kind in ['rook','bishop','knight','queen','pawn']:
  for f in range(8):
   im=unpack('magnus_pieces_'+str(look),'piece_'+kind+'_'+str(look),f,(48,56));solid=im.getchannel('A').point(lambda a:255 if a==255 else 0)
   assert not ImageChops.subtract(mask,solid).getbbox(),(look,kind,f,'Alchemist exposed')
   assert im.getbbox()[2]-im.getbbox()[0]<=32 and im.getbbox()[3]-im.getbbox()[1]<=44;cases+=1
 for pose in ['idle','run','attack','skill1','skill2','ult','hit','dead']:
  for f in range(8):
   for left in [False,True]:
    actual=unpack('magnus_outfit_'+str(look),'outfit'+str(look)+'_'+pose+('_l'if left else''),f)
    expected=R.body(pose,f,look);alpha=expected.getchannel('A');d=ImageDraw.Draw(alpha);d.rectangle((0,0,47,22),fill=0);d.rectangle((0,40,47,55),fill=0);expected.putalpha(alpha)
    if left:expected=expected.transpose(Image.Transpose.FLIP_LEFT_RIGHT)
    bg=Image.new('RGBA',(48,56),'#101827');a=bg.copy();a.alpha_composite(actual);b=bg.copy();b.alpha_composite(expected)
    assert a.tobytes()==b.tobytes(),(look,pose,f,left,'anchor / outfit')
paths=[MOD/'champions/tfm2_custom_magnus#sheet.png',*sorted((MOD/'vfx').glob('magnus*#sheet.png'))];decoded=0
for p in paths:
 im=Image.open(p);assert max(im.size)<=2048;decoded+=im.width*im.height*4
assert decoded<=32*1024*1024,decoded
F=ImageFont.truetype('C:/Windows/Fonts/segoeui.ttf',13)
im=Image.new('RGB',(1100,650),'#101827');d=ImageDraw.Draw(im);d.text((14,10),'Slim A · eleven ranked appearances · both facings · 2× pixels',font=F,fill='#e5d7b3')
for look,name in enumerate(R.NAMES):
 x=10+(look%4)*275;y=44+(look//4)*200;d.text((x+5,y),name,font=F,fill='#e5d7b3')
 for left in [False,True]:
  body=R.construction(look,0,left);b=R.body('idle',0,look);b=b.transpose(Image.Transpose.FLIP_LEFT_RIGHT)if left else b;body.alpha_composite(b,(24,20));big=body.resize((192,192),Image.Resampling.NEAREST);im.paste(big,(x+int(left)*95,y+10),big)
 b=R.piece('knight',0,look).resize((96,96),Image.Resampling.NEAREST);im.paste(b,(x+174,y+90),b)
im.save(ROOT/'docs/magnus-ranked-appearances.png')
im=Image.new('RGB',(1040,1100),'#101827');d=ImageDraw.Draw(im);d.text((12,8),'Beginner / Master / #1 · all poses, frames and facings',font=F,fill='#e5d7b3')
for row,pose in enumerate(['idle','run','attack','skill1','skill2','ult','hit','dead']):
 y=40+row*132;d.text((5,y),pose,font=F,fill='#e5d7b3')
 for j,look in enumerate([0,4,10]):
  for left in [False,True]:
   for f in range(8):
    b=R.body(pose,f,look);b=b.transpose(Image.Transpose.FLIP_LEFT_RIGHT)if left else b;im.paste(b,(76+j*320+f*39,y+left*60),b)
im.save(ROOT/'docs/magnus-ranked-poses.png')
print(f'PASS: {cases} compact frames cover Alchemist; 11 × 8 poses × 8 frames × 2 facings; {decoded/2**20:.2f} MiB decoded.')
