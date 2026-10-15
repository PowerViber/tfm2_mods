from pathlib import Path
from PIL import Image,ImageDraw,ImageFont
import json,subprocess,sys
root=Path(__file__).resolve().parents[1];sys.path.insert(0,str(root/'Claude outputs/magnus'));import rank_art as R
font=ImageFont.truetype('C:/Windows/Fonts/segoeui.ttf',13)
im=Image.new('RGB',(850,260),'#101827');d=ImageDraw.Draw(im);d.text((12,10),'Game-size silhouettes Â· prior piece / compact Alchemist pieces / Magnus',font=font,fill='#e5d7b3')
prefix='mods/tfm2_custom/vfx/magnus_pieces';old=Image.open(__import__('io').BytesIO(subprocess.check_output(['rtk','proxy','git','show','30f2969:'+prefix+'#sheet.png']))).convert('RGBA');meta=json.loads(subprocess.check_output(['rtk','proxy','git','show','30f2969:'+prefix+'#anim.fanim']));r=next(iter(meta['anims'].values()))['frames'][0]['data'];old=old.crop((r['x'],r['y'],r['x']+r['w'],r['y']+r['h']));im.paste(old,(20,65),old)
for i,k in enumerate(['rook','bishop','knight','queen','pawn']):
 b=R.piece(k,0,4);im.paste(b,(150+i*105,65),b);d.text((143+i*105,135),k,font=font,fill='#e5d7b3')
b=R.body('idle',0,4);im.paste(b,(720,65),b);d.text((700,135),'Magnus',font=font,fill='#e5d7b3');d.text((14,185),'Board: 256,000 world units per side. Visible piece footprint: up to 32Ã—44 pixels, including carrier coverage.',font=font,fill='#b7c6d7');im.save(root/'docs/magnus-size-comparison.png')
meta=json.loads((root/'mods/tfm2_custom/vfx/magnus_ranks#anim.fanim').read_text())['anims'];sheet=Image.open(root/'mods/tfm2_custom/vfx/magnus_ranks#sheet.png').convert('RGBA');motion=json.loads((root/'mods/tfm2_custom/vfx/magnus_rank_motion#anim.fanim').read_text())['anims']['rank_motion']['frames'];spark=Image.open(root/'mods/tfm2_custom/vfx/magnus_rank_motion#sheet.png').convert('RGBA')
im=Image.new('RGB',(1040,460),'#101827');d=ImageDraw.Draw(im);d.text((12,8),'17 animated emblems Â· sixteen independent clock frames Â· text stays readable in either facing',font=font,fill='#e5d7b3')
def layer(canvas,source,r):
 b=source.crop((r['x'],r['y'],r['x']+r['w'],r['y']+r['h']));canvas.alpha_composite(b,((48-b.width)//2,(56-b.height)//2))
for badge in range(17):
 y=35+badge*24;d.text((8,y),'Rank '+str(badge+1)if badge<7 else'Top #'+str(badge-6),font=font,fill='#e5d7b3')
 for f in range(16):
  b=Image.new('RGBA',(48,56));layer(b,sheet,meta['rank'+str(badge)]['frames'][f]['data']);layer(b,spark,motion[f]['data']);b=b.crop((31,0,48,16)).resize((34,32),Image.Resampling.NEAREST);im.paste(b,(105+f*57,y-7),b)
im.save(root/'docs/magnus-emblems.png')
print('Size comparison and all 17 emblem clocks generated.')
