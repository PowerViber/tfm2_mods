"""Extract review-only Akane concepts. Native assets remain approval-gated."""
from pathlib import Path
import json,hashlib
from PIL import Image,ImageDraw,ImageOps
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/akane-art-review';N=Image.Resampling.NEAREST
def solid(im):
    im=im.convert('RGBA');im.putalpha(im.getchannel('A').point(lambda a:255 if a>=180 else 0));return im
def connected(im):
    im=solid(im);mask=im.getchannel('A');parts=[]
    while box:=mask.getbbox():
        seed=next((x,box[1]) for x in range(box[0],box[2]) if mask.getpixel((x,box[1])))
        ImageDraw.floodfill(mask,seed,128);part=mask.point(lambda p:255 if p==128 else 0);parts.append((sum(part.histogram()[1:]),part));mask.paste(0,(0,0),part)
    if parts:im.putalpha(max(parts,key=lambda p:p[0])[1])
    return im
def main():
    source=OUT/'concept-source-v1.png';im=Image.open(source);poses={};scale=None
    for i,name in enumerate(['idle','run','attack','skill1']):
        tile=connected(im.crop((i%2*im.width//2,i//2*im.height//2,(i%2+1)*im.width//2,(i//2+1)*im.height//2)))
        box=tile.getbbox();assert box,name
        if scale is None:scale=40/(box[3]-box[1])
        crop=tile.crop(box);small=crop.resize((round(crop.width*scale),round(crop.height*scale)),Image.Resampling.BOX)
        small.putalpha(small.getchannel('A').point(lambda a:255 if a>=80 else 0))
        small=small.quantize(colors=16,method=Image.Quantize.FASTOCTREE,dither=Image.Dither.NONE).convert('RGBA')
        width=max(48,((small.width+7)//8)*8);canvas=Image.new('RGBA',(width,56));canvas.alpha_composite(small,((width-small.width)//2,49-small.height));poses[name]=canvas
    frames={};sheet=Image.new('RGBA',(512,128));x=0
    for name,im in poses.items():
        for left in [False,True]:
            frame=ImageOps.mirror(im) if left else im;sheet.alpha_composite(frame,(x,0));frames[name+('_l' if left else '')]={'frames':[{'duration':.2,'data':{'x':x,'y':0,'w':frame.width,'h':56}}]};x+=frame.width+2
    sheet.save(OUT/'body-preview.png');(OUT/'body-preview.fanim').write_text(json.dumps({'anims':frames},indent=2)+'\n')
    board=Image.new('RGB',(1120,840));d=ImageDraw.Draw(board)
    w=json.loads((ROOT/'editor/game-reference/world.json').read_text());stocks=[]
    for id in ['champ-ninja','champ-knight','champ-archer']:
        spec=w['sprites'][id];r=spec['anims']['idle']['frames'][0]['data'];stocks.append((id[6:],Image.open(ROOT/'editor/game-reference'/spec['file']).crop((r['x'],r['y'],r['x']+r['w'],r['y']+r['h']))))
    for row,(name,im) in enumerate(poses.items()):
        y=row*210
        for col,bg in enumerate(['#203747','#bec9b0']):
            x=col*560;d.rectangle((x,y,x+559,y+209),fill=bg);ink='#ffd5da' if col==0 else '#213b3a';d.text((x+8,y+5),name+' | 1x + 3x | both facings',fill=ink)
            for j,frame in enumerate([im,ImageOps.mirror(im)]):
                board.paste(frame,(x+8+j*58,y+106),frame);big=frame.resize((frame.width*3,168),N);board.paste(big,(x+126+j*150,y+24),big)
            for j,(title,stock) in enumerate(stocks):board.paste(stock,(x+430+j*42,y+117),stock);d.text((x+426+j*42,y+174),title[:6],fill=ink)
    board.save(OUT/'body-comparison.png')
    manifest={'version':1,'revision':'akane-art-1','review_only':True,'body_approved':False,'effects_approved':False,'complete_body':False,'units_per_pixel':1000,'body_canvas':[48,56],'world_anchor':[24,28],'foot_anchor':[24,48],'atlases':{'body':'body-preview'},'poses':list(poses),'source_sha256':{source.name:hashlib.sha256(source.read_bytes()).hexdigest()},'proposal_note':'Four appearance concepts only; remaining animation frames await appearance approval.'}
    (OUT/'presentation.json').write_text(json.dumps(manifest,indent=2)+'\n');(OUT/'art-manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
    if not (OUT/'approvals.json').exists():(OUT/'approvals.json').write_text(json.dumps({'body':{'approved':False},'effects':{'approved':False}},indent=2)+'\n')
    print(json.dumps({'source':manifest['source_sha256'],'poses':list(poses),'native_integration':False}))
if __name__=='__main__':main()
