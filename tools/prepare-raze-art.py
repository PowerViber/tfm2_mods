"""Pack the generated Raze source into fixed game-size cells; never repaint source pixels.

The default writes review artifacts only. --integrate requires the reviewed source hash.
"""
from pathlib import Path
from PIL import Image, ImageOps, ImageDraw
import argparse,hashlib,json,shutil
root=Path(__file__).resolve().parents[1]
p=argparse.ArgumentParser();p.add_argument('--source');p.add_argument('--integrate',action='store_true');p.add_argument('--approved-sha256');a=p.parse_args()
review=root/'docs/raze-review';review.mkdir(parents=True,exist_ok=True)
def save_png(im,path):
 # Keep the approved file bytes when the pixels are unchanged (PNG encoders differ between Pillow builds).
 if path.exists():
  old=Image.open(path).convert('RGBA')
  if old.size==im.size and old.tobytes()==im.convert('RGBA').tobytes():return
 im.save(path)
source=review/'raze-generated-source.png'
if a.source:shutil.copyfile(a.source,source)
sha=hashlib.sha256(source.read_bytes()).hexdigest();im=Image.open(source).convert('RGBA');w,h=im.size
names=['idle','move','attack','skill1','skill2','airborne','ult','hit']
atlas=Image.new('RGBA',(36*4,44*4));frames=[]
for n,name in enumerate(names):
 x=(n%4)*w//4;y=(n//4)*h//2
 cell=im.crop((x,y,x+w//4,y+h//2));box=cell.getchannel('A').point(lambda v:255 if v>200 else 0).getbbox();assert box
 piece=cell.crop(box);piece=piece.resize((max(1,round(piece.width*.105)),max(1,round(piece.height*.105))),Image.Resampling.NEAREST)
 assert piece.width<=36 and piece.height<=44,(name,piece.size)
 frame=Image.new('RGBA',(36,44));frame.alpha_composite(piece,((36-piece.width)//2,43-piece.height))
 atlas.alpha_composite(frame,((n%4)*36,(n//4)*44));frames.append(frame)
save_png(atlas,review/'raze-candidate.png')
old=Image.open(root/'mods/tfm2_valorant/champions/tfm2_valorant_omen#sheet.png').convert('RGBA').crop((0,0,36,44))
sheet=Image.new('RGB',(880,400),'#18252b');d=ImageDraw.Draw(sheet)
d.text((16,12),'RAZE / Native 36 x 44 cells, then 4x inspection. Omen scale reference at right.',fill='#e6eee8')
for n,frame in enumerate(frames):
 x=16+(n%4)*172;y=44+(n//4)*172
 d.rectangle((x,y,x+153,y+153),fill='#a6b18c' if n%2 else '#203035')
 sheet.paste(frame,(x,y),frame);large=frame.resize((108,132),Image.Resampling.NEAREST);sheet.paste(large,(x+42,y+5),large)
 d.text((x,y+156),names[n],fill='#e6eee8')
sheet.paste(old,(756,60),old);large=old.resize((108,132),Image.Resampling.NEAREST);sheet.paste(large,(730,115),large)
d.text((729,270),'Omen / same scale',fill='#d9e5d8');save_png(sheet,review/'raze-comparison.png')
anim=[]
for frame in frames:
 bg=Image.new('RGBA',(216,264),'#203035');bg.alpha_composite(frame.resize((216,264),Image.Resampling.NEAREST));anim.append(bg.convert('RGB'))
anim[0].save(review/'raze-poses.gif',save_all=True,append_images=anim[1:],duration=350,loop=0)
meta={'source_sha256':sha,'native_canvas':[36,44],'foot_baseline':43,'poses':names,'approval':'pending','decoded_bytes':atlas.width*atlas.height*4}
(review/'review.json').write_text(json.dumps(meta,indent=2)+'\n',encoding='utf-8')
if a.integrate:
 assert a.approved_sha256==sha,'Body integration requires approval of this source hash'
 target=root/'mods/tfm2_valorant/champions'
 # Preserve the established animation names/counts/timings and anchors; pose artwork is shared.
 oldmeta=json.loads((target/'tfm2_valorant_omen#anim.fanim').read_text(encoding='utf-8'))
 for name,ani in oldmeta['anims'].items():
  index=names.index('move') if name=='run' else names.index(name) if name in names else names.index('hit') if name=='dead' else 0
  for frame in ani['frames']:frame['data']={'x':index%4*36,'y':index//4*44,'w':36,'h':44}
 # Airborne body follows the real carrier with a visual height offset. Original body is blank during flight.
 # Round 143: the airborne poses are renewed every tick, so each plays exactly one tick (no stacked ghosts).
 blank=dict(x=0,y=88,w=36,h=44);right=dict(x=36,y=44,w=36,h=44)
 flipped=ImageOps.mirror(frames[5]);atlas.alpha_composite(flipped,(36,88))
 oldmeta['anims']['air_hidden']={'frames':[{'duration':.05,'data':blank}]}
 oldmeta['anims']['airborne_r']={'frames':[{'duration':1/60,'data':right}]}
 oldmeta['anims']['airborne_l']={'frames':[{'duration':1/60,'data':dict(x=36,y=88,w=36,h=44)}]}
 for j,(tag,index) in enumerate([('rocket',6),('grenade',3),('pack',4)]):
  xx=(j+2)%4*36;yy=88+(j+2)//4*44;atlas.alpha_composite(ImageOps.mirror(frames[index]),(xx,yy))
  oldmeta['anims']['airborne_'+tag+'_r']={'frames':[{'duration':1/60,'data':dict(x=index%4*36,y=index//4*44,w=36,h=44)}]}
  oldmeta['anims']['airborne_'+tag+'_l']={'frames':[{'duration':1/60,'data':dict(x=xx,y=yy,w=36,h=44)}]}
 save_png(atlas,target/'tfm2_valorant_raze#sheet.png')
 (target/'tfm2_valorant_raze#anim.fanim').write_text(json.dumps(oldmeta,indent=1)+'\n',encoding='utf-8')
 meta['approval']='approved';(review/'review.json').write_text(json.dumps(meta,indent=2)+'\n',encoding='utf-8')
print(json.dumps(meta))
