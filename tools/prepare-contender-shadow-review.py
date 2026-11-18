"""Extract original generated concepts for review only; never touches game assets."""
from pathlib import Path
import hashlib,json
from PIL import Image,ImageOps,ImageDraw,ImageFont
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/contender-shadow-review';OUT.mkdir(exist_ok=True)
source=OUT/'body-source-v1.png';im=Image.open(source).convert('RGBA');N=Image.Resampling.NEAREST
assert im.size==(2172,724)
poses=['idle','run','straight','uppercut'];anchors=[(245,665),(220,665),(210,665),(248,665)]
scale=42/519;anims={};sheet=Image.new('RGBA',(80*8,64));frames=[]
for i,(pose,(ax,ay)) in enumerate(zip(poses,anchors)):
    crop=im.crop((543*i,80,543*(i+1),690));crop=crop.resize((round(crop.width*scale),round(crop.height*scale)),N)
    tile=Image.new('RGBA',(80,64));tile.alpha_composite(crop,(40-round(ax*scale),52-round((ay-80)*scale)))
    for left in [False,True]:
        frame=tile
        if left:
            frame=Image.new('RGBA',tile.size);frame.alpha_composite(ImageOps.mirror(tile),(1,0))
        index=len(frames);sheet.alpha_composite(frame,(index*80,0));frames.append(frame)
        anims[pose+('_l' if left else '')]={'x':index*80,'y':0,'w':80,'h':64}
sheet.save(OUT/'body-proposals.png')
old=Image.open(OUT/'old-body-reference.png').convert('RGBA')
world=json.loads((ROOT/'editor/game-reference/world.json').read_text())
stock=[]
for name in ['champ-knight','champ-berserker']:
    spec=world['sprites'][name];atlas=Image.open(ROOT/'editor/game-reference'/spec['file']).convert('RGBA');r=spec['anims']['idle']['frames'][0]['data']
    stock.append(atlas.crop((r['x'],r['y'],r['x']+r['w'],r['y']+r['h'])))
font=ImageFont.truetype('C:/Windows/Fonts/arial.ttf',15)
review=Image.new('RGBA',(1000,620),'#11232d');d=ImageDraw.Draw(review)
d.text((20,14),'CONTENDER — SHADOW BOXER / appearance proposal, not integrated',font=font,fill='#edf2f1')
for row,(bg,ink) in enumerate([('#29454a','#f4e9d4'),('#bac5a6','#152b35')]):
    y=48+row*275;d.rectangle((12,y,988,y+260),fill=bg)
    references=[old,*stock]
    if row:references=[ImageOps.mirror(f) for f in references]
    for j,(label,tile) in enumerate(zip(['Old Contender','Stock Knight','Stock Berserker','New idle','New movement','New straight','New uppercut'],[*references,*[frames[k+row] for k in [0,2,4,6]]])):
        x=22+j*137;d.text((x,y+9),label,font=font,fill=ink)
        review.alpha_composite(tile,(x+55-tile.width//2,y+94-tile.height//2))
        big=tile.resize((tile.width*3,tile.height*3),N);review.alpha_composite(big,(x+60-big.width//2,y+194-big.height//2))
    d.text((20,y+241),'1× at top; 3× below. '+('Left' if row else 'Right')+' facing. Fixed anchors; review only.',font=font,fill=ink)
review.convert('RGB').save(OUT/'body-comparison.png')
approval=json.loads((OUT/'approvals.json').read_text()) if (OUT/'approvals.json').exists() else {}
meta={'revision':'contender-shadow-art-1','body_approved':approval.get('body',{}).get('approved',False),'effects_approved':approval.get('effects',{}).get('approved',False),'source':'body-source-v1.png','source_sha256':hashlib.sha256(source.read_bytes()).hexdigest(),'body_file':'../docs/contender-shadow-review/body-proposals.png','body_frames':anims,'canvas':[80,64],'foot_anchor':[40,52],'visible_idle_height':42,'source_scale':scale,'source_anchors':anchors,'gameplay_revision':'contender-2.0','generation_deterministic':False,'extraction_deterministic':True}
(OUT/'review-manifest.json').write_text(json.dumps(meta,indent=2)+'\n')
(ROOT/'editor/contender-art.js').write_text('window.CONTENDER_ART='+json.dumps(meta,separators=(',',':'))+';\n')
print(json.dumps({'source':str(source),'source_sha256':meta['source_sha256'],'review_only':True,'visible_body_pixels':42}))
