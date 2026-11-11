"""Review-only extraction of original imagegen wave and status materials."""
from pathlib import Path
import json,hashlib
from PIL import Image,ImageDraw,ImageOps
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/round-165-art-review';N=Image.Resampling.NEAREST
def cells(path):
    im=Image.open(path).convert('RGBA');out=[]
    for i in range(8):
        f=im.crop((i%4*im.width//4,i//4*im.height//2,(i%4+1)*im.width//4,(i//4+1)*im.height//2));f.putalpha(f.getchannel('A').point(lambda a:255 if a>=220 else 0));assert f.getbbox();out.append(f.crop(f.getbbox()))
    return out
def fit(im,size):
    im=im.resize(size,Image.Resampling.BOX);im.putalpha(im.getchannel('A').point(lambda a:255 if a>=100 else 0));return im.quantize(colors=12,method=Image.Quantize.FASTOCTREE,dither=Image.Dither.NONE).convert('RGBA')
def materials():
    waves=[fit(im,(66,15)) for im in cells(OUT/'tsunami-source-v2.png')];a=cells(OUT/'accents-source-v1.png');icons={}
    pip=fit(a[0],(4,7))
    for n in range(1,6):
        im=Image.new('RGBA',(48,104))
        for i in range(5):
            p=pip.copy()
            if i>=n:p.putalpha(p.getchannel('A').point(lambda x:x//4))
            im.alpha_composite(p,(10+i*6,12))
        icons['rhythm'+str(n)]=im
    for name,src,size in [('ambush',1,(32,24)),('dismiss_aura',2,(34,40))]:
        im=Image.new('RGBA',(48,64));f=fit(a[src],size);im.alpha_composite(f,((48-f.width)//2,(64-f.height)//2));icons[name]=im
    im=Image.new('RGBA',(48,112));d=ImageDraw.Draw(im);d.rectangle((9,10,38,21),fill='#12283ee8')
    for i,(idx,size) in enumerate([(4,(8,5)),(5,(7,7)),(6,(7,7))]):im.alpha_composite(fit(a[idx],size),(11+i*9,13))
    icons['carrier_empowered']=im
    return waves,icons
def main():
    waves,icons=materials();anims={};atlas=Image.new('RGBA',(512,256));x=y=0;row=0
    for name,ims,ts in [('tsunami',waves,[2,2,2,2,2,2,3,3])]+[(n,[f],[60]) for n,f in icons.items()]:
        frames=[]
        for im,t in zip(ims,ts):
            if x+im.width>512:x=0;y+=row+2;row=0
            atlas.alpha_composite(im,(x,y));frames.append({'duration':t/60,'data':dict(x=x,y=y,w=im.width,h=im.height)});x+=im.width+2;row=max(row,im.height)
        anims[name]={'frames':frames}
    atlas.save(OUT/'preview.png');(OUT/'preview.fanim').write_text(json.dumps({'anims':anims},indent=2)+'\n')
    board=Image.new('RGB',(1120,800));d=ImageDraw.Draw(board)
    for i,name in enumerate(['tsunami','rhythm5','ambush','dismiss_aura','carrier_empowered']):
        for col,bg in enumerate(['#203747','#bec9b0']):
            x=col*560;y=i*160;d.rectangle((x,y,x+559,y+159),fill=bg);d.text((x+8,y+5),name+' | 1x + 3x | visible glyph',fill='#ff8eaa' if not col else '#302d40')
            im=waves[3] if name=='tsunami' else icons[name];im=im.crop(im.getbbox());board.paste(im,(x+30,y+30),im);big=im.resize((im.width*3,im.height*3),N)
            board.paste(big,(x+170,y+15),big)
    board.save(OUT/'effects-comparison.png')
    if not (OUT/'approval.json').exists():(OUT/'approval.json').write_text(json.dumps({'effects':{'approved':False}},indent=2)+'\n')
    frozen={p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in [OUT/'tsunami-source-v2.png',OUT/'accents-source-v1.png']}
    if (OUT/'frozen-sources.json').exists():assert json.loads((OUT/'frozen-sources.json').read_text())==frozen,'Frozen source changed; obtain a new approval'
    else:(OUT/'frozen-sources.json').write_text(json.dumps(frozen,indent=2)+'\n')
    print(json.dumps({'review_only':True,'tags':list(anims),'sources':frozen}))
if __name__=='__main__':main()
