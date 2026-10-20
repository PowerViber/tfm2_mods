"""Register existing white-eye material to every approved body frame, both facings."""
from pathlib import Path
from PIL import Image,ImageOps,ImageDraw
import json,shutil
ROOT=Path(__file__).resolve().parents[1];MOD=ROOT/'mods/tfm2_blockcraft';OUT=MOD/'vfx'
def generate():
    meta=json.loads((MOD/'champions/tfm2_blockcraft_steve#anim.fanim').read_text())['anims']
    body=Image.open(MOD/'champions/tfm2_blockcraft_steve#sheet.png').convert('RGBA')
    anchors=json.loads((ROOT/'rules/steve-eye-anchors.json').read_text());atlas=Image.new('RGBA',(12*36,8*44));anims={}
    review=Image.new('RGB',(6*144,8*176),'#27393d');anim=[];durations=[]
    for row,(pose,data)in enumerate(meta.items()):
        assert len(data['frames'])==len(anchors[pose]);right=[];left=[]
        for n,(frame,points)in enumerate(zip(data['frames'],anchors[pose])):
            r=frame['data'];im=body.crop((r['x'],r['y'],r['x']+36,r['y']+44));overlay=Image.new('RGBA',(36,44))
            for x,y in points:
                assert im.getpixel((x,y))[3]>0,(pose,n,x,y)
                overlay.putpixel((x,y),(255,255,255,255))
                for dy in [-1,0,1]:
                    for dx in [-1,0,1]:
                        q=(x+dx,y+dy)
                        if 0<=q[0]<36 and 0<=q[1]<44:
                            red,green,blue,alpha=im.getpixel(q)
                            if alpha and blue>red*1.03 and blue>green*1.03:overlay.putpixel(q,(255,255,255,255))
            for col,part,frames in [(n,overlay,right),(6+n,ImageOps.mirror(overlay),left)]:
                atlas.alpha_composite(part,(36*col,44*row));frames.append(dict(duration=frame['duration'],data=dict(x=36*col,y=44*row,w=36,h=44)))
            im.alpha_composite(overlay);big=im.resize((144,176),Image.Resampling.NEAREST);review.paste(big,(144*n,176*row),big)
            cell=Image.new('RGB',(360,170),'#25393a');draw=ImageDraw.Draw(cell);draw.text((8,8),pose,fill='white')
            cell.paste(im,(12,110),im);cell.paste(big,(72,0),big);flip=ImageOps.mirror(big);cell.paste(flip,(216,0),flip);anim.append(cell);durations.append(round(frame['duration']*1000))
        anims[pose+'_r']=dict(frames=right);anims[pose+'_l']=dict(frames=left)
    atlas.save(OUT/'steve_eyes#sheet.png');(OUT/'steve_eyes#anim.fanim').write_text(json.dumps(dict(anims=anims),separators=(',',':')))
    data_path=MOD/'champion/tfm2_blockcraft_steve.data_champion';champ=json.loads(data_path.read_text());champ['view_buffs']=[v for v in champ['view_buffs']if not v['name'].startswith('sc_eyes_')]+[dict(type='Animated',name='sc_eyes_'+tag,anim='asset/tfm2_blockcraft/vfx/steve_eyes',tag=tag,z=4)for tag in anims]
    data_path.write_text(json.dumps(champ,indent=2)+'\n');(ROOT/'editor/steve-preset.js').write_text('window.STEVE_PRESET='+json.dumps(champ,separators=(',',':'))+';\n')
    for tag in ['dead_r','dead_l']:
        name='tfm2_blockcraft_eye_'+tag
        champ['view_effects']=[v for v in champ['view_effects']if v['name']!=name]+[dict(type='Animation',name=name,anim='asset/tfm2_blockcraft/vfx/steve_eyes',tag=tag,z=4,is_follow=True)]
    data_path.write_text(json.dumps(champ,indent=2)+'\n');(ROOT/'editor/steve-preset.js').write_text('window.STEVE_PRESET='+json.dumps(champ,separators=(',',':'))+';\n')
    for src,dest in [('steve_eyes#sheet.png','steve-eyes.png'),('steve_eyes#anim.fanim','steve-eyes.fanim')]:shutil.copyfile(OUT/src,ROOT/'editor'/dest)
    review.save(ROOT/'docs/character-art-review/herobrine-all-frames.png');anim[0].save(ROOT/'docs/character-art-review/herobrine-animation.gif',save_all=True,append_images=anim[1:],duration=durations,loop=0,disposal=2)
    print('Registered 31 eye frames, mirrored with original body timings.')
if __name__=='__main__':generate()
