"""Complete approved Vader animation timelines into review artifacts, without installing."""
from pathlib import Path
import importlib.util,json,hashlib
from PIL import Image,ImageOps,ImageDraw
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/vader-aegis-scribble-review';D=OUT/'vader'
spec=importlib.util.spec_from_file_location('body_tools',ROOT/'tools/original_bodies.py');b=importlib.util.module_from_spec(spec);spec.loader.exec_module(b)
p=b.sprite_library(OUT/'vader-source.png',8);m=b.sprite_library(OUT/'vader-motion-source.png',8);scale=40/p[0].height
stem=ROOT/'mods/tfm2_starwars/champions/tfm2_starwars_vader';original=Image.open(str(stem)+'#sheet.png').convert('RGBA');old=json.loads(Path(str(stem)+'#anim.fanim').read_text());anims={};images=[];rects={};x=y=row=0
def pack(im):
    global x,y,row
    key=(im.size,im.tobytes())
    if key in rects:return rects[key]
    if x+im.width>1024:x=0;y+=row;row=0
    r=dict(x=x,y=y,w=im.width,h=im.height);images.append((r,im));rects[key]=r;x+=im.width;row=max(row,im.height);return r
def choice(tag,n,count):
    if tag=='dead':return [m[4],m[5],m[6]][min(2,n*3//count)]
    if tag=='hit':return m[4]
    if tag=='run':return [m[0],p[1],m[1],p[1]][n%4]
    if tag=='attack':return [m[2],p[2],m[3],p[0]][min(3,n*4//count)]
    if tag in ['choke','choke_seq','skill2']:return p[5]if 0<n<count-1 or count==1 else p[0]
    if tag in ['ult','ult_area']:return p[6]
    if tag.endswith('dash'):return p[1]
    if tag in ['skill1_pre','skill1_end','skill1_effect']:return p[4]
    return [p[0],m[7],m[7],p[0]][n%4]
for name,a in old['anims'].items():
    frames=[]
    for n,f in enumerate(a['frames']):
        reference=b.crop_frame(original,f);fitted,padding=b.fit_pose(choice(name,n,len(a['frames'])),reference,scale);frames.append({'duration':f['duration'],'data':pack(fitted)})
    anims[name]={'frames':frames}
sheet=Image.new('RGBA',(1024,y+row))
for r,im in images:sheet.alpha_composite(im,(r['x'],r['y']))
assert max(sheet.size)<=2048;sheet.save(D/'complete#sheet.png');(D/'complete#anim.fanim').write_text(json.dumps({'anims':anims},separators=(',',':'))+'\n')
gif=[]
for name,a in anims.items():
    for f in a['frames']:
        r=f['data'];im=sheet.crop((r['x'],r['y'],r['x']+r['w'],r['y']+r['h']));panel=Image.new('RGB',(600,270),'#17382e');d=ImageDraw.Draw(panel);d.text((12,10),name+' - completed existing timeline / both facings',fill='white');im=im.resize((im.width*3,im.height*3),Image.Resampling.NEAREST);panel.paste(im,(140-im.width//2,230-im.height),im);mirror=ImageOps.mirror(im);panel.paste(mirror,(450-im.width//2,230-im.height),mirror);gif.append(panel)
gif[0].save(D/'complete-animations.gif',save_all=True,append_images=gif[1:],duration=100,loop=0)
assert set(anims)==set(old['anims'])
for name,a in anims.items():assert [f['duration']for f in a['frames']]==[f['duration']for f in old['anims'][name]['frames']]
(D/'complete.json').write_text(json.dumps({'appearanceApproved':True,'approvalQuote':'i love vader','sourceHashes':{p.name:hashlib.sha256(p.read_bytes()).hexdigest()for p in [OUT/'vader-source.png',OUT/'vader-motion-source.png']},'animations':len(anims),'frameCount':sum(len(v['frames'])for v in anims.values()),'namesCountsTimingPreserved':True,'atlas':list(sheet.size),'decodedBytes':sheet.width*sheet.height*4,'integrated':False},indent=2)+'\n')
print('Completed',len(anims),'Vader animation timelines in review artifacts')
