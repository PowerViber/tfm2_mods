"""Reproducible extraction/packing of imagegen proposals. No game assets are replaced."""
from pathlib import Path
import hashlib,json,re
from PIL import Image,ImageOps,ImageDraw,ImageFont

ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/'docs/basalt-art-review'
NEAREST=Image.Resampling.NEAREST
CW,CH,FX,FY=128,80,64,60

def digest(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def crisp(im,size,colours=24):
    im=im.resize(size,Image.Resampling.BOX)
    im.putalpha(im.getchannel('A').point(lambda a:255 if a>=150 else 0))
    return im.quantize(colors=colours,method=Image.Quantize.FASTOCTREE,dither=Image.Dither.NONE).convert('RGBA')

# Crops separate the lash reaching beyond its source-grid cell from the next pose.
# Foot/head landmarks retain body scale independently of raised arms and the whip.
source=Image.open(OUT/'body-source-v2.png').convert('RGBA')
specs=[
 ('front',(0,0,400,443),(205,390),44),
 ('idle',(444,0,887,443),(212,392),46),
 ('walk_a',(887,0,1330,443),(218,405),54),
 ('walk_b',(1330,0,1774,443),(225,409),54),
 ('raise',(0,444,400,887),(210,401),73),
 ('whip',(400,444,975,887),(170,401),98),
 ('charge',(975,444,1330,887),(160,403),102),
 ('air_punch',(1330,444,1774,887),(235,375),52),
]
body=Image.new('RGBA',(CW*len(specs),CH*2));frames={};poses={}
for i,(name,rect,foot,hair) in enumerate(specs):
    crop=source.crop(rect)
    scale=44/(foot[1]-hair)
    size=(round(crop.width*scale),round(crop.height*scale))
    sprite=crisp(crop,size)
    x=FX-round(foot[0]*scale);y=FY-round(foot[1]*scale)
    assert x>=0 and y>=0 and x+sprite.width<=CW and y+sprite.height<=CH,(name,size,x,y)
    pose=Image.new('RGBA',(CW,CH));pose.alpha_composite(sprite,(x,y));poses[name]=pose
    for left in [False,True]:
        key=name+('_l' if left else '')
        tile=ImageOps.mirror(pose) if left else pose
        body.alpha_composite(tile,(i*CW,CH if left else 0))
        frames[key]={'x':i*CW,'y':CH if left else 0,'w':CW,'h':CH,'foot':[FX,FY]}
body.save(OUT/'body-proposals.png')

fx_source=Image.open(OUT/'effects-source-v1.png').convert('RGBA')
effect_specs=[
 ('block',(90,120,222,260),(16,18)),
 ('square',(350,62,582,279),(32,36)),
 ('rise',(643,58,934,302),(64,54)),
 ('break',(966,54,1252,301),(64,54)),
 ('dust',(68,394,267,560),(28,24)),
 ('slam',(314,336,641,598),(120,96)),
 ('wall_slam',(652,368,957,578),(64,44)),
 ('leap',(960,410,1260,590),(48,29)),
 ('land',(8,627,317,913),(64,58)),
 ('dash',(348,654,616,854),(52,39)),
 ('rock',(715,648,883,822),(24,25)),
 ('boulder',(951,593,1257,890),(88,86)),
 ('meteor',(0,882,330,1190),(62,58)),
 ('meteor_impact',(322,934,638,1186),(64,52)),
 ('whip_link',(648,954,950,1160),(48,33)),
 ('punch_impact',(956,917,1268,1207),(104,96)),
]
FW,FH=128,112;effects=Image.new('RGBA',(FW*4,FH*4));eframes={}
for i,(name,rect,size) in enumerate(effect_specs):
    im=crisp(fx_source.crop(rect),size,20)
    px=(FW-size[0])//2;py=(FH-size[1])//2
    effects.alpha_composite(im,(i%4*FW+px,i//4*FH+py))
    eframes[name]={'x':i%4*FW+px,'y':i//4*FH+py,'w':size[0],'h':size[1]}
effects.save(OUT/'effect-proposals.png')

meta={'revision':'basalt-art-proposal-2','review_only':True,'approved':False,
 'rules_revision':'basalt-1.1','base_commit':'3790d7ddb9578f3fc051e67bfa5f8f0bbb1f200a',
 'body_file':'../docs/basalt-art-review/body-proposals.png','effects_file':'../docs/basalt-art-review/effect-proposals.png',
 'body_canvas':[CW,CH],'feet':[FX,FY],'visible_idle_body_height':44,'world_units_per_pixel':1000,
 'wall_footprint':[32000,32000],'body_frames':frames,'effect_frames':eframes,
 'sources':{p.name:digest(p) for p in [OUT/'body-source-v1.png',OUT/'body-source-v2.png',OUT/'effects-source-v1.png']},
 'outputs':{p.name:digest(p) for p in [OUT/'body-proposals.png',OUT/'effect-proposals.png']},
 'decoded_review_atlas_bytes':body.width*body.height*4+effects.width*effects.height*4,
 'generation_deterministic':False,'packing_deterministic':True,
 'notes':['Appearance concepts only; complete walk/cast/death cycles follow approval.',
          'Native frames and gameplay unchanged. These files live outside mods/.']}
rules=json.loads((ROOT/'rules/basalt.json').read_text())
core=(ROOT/'native/tfm2_custom_ai/src/basalt_core.rs').read_text()
section=core[core.index('fn crushing_clap'):core.index('fn run_claps')]
inner=int(re.search(r'r0:\s*([0-9_]+)',section)[1].replace('_',''))
slack=int(re.search(r'r >= c.r0 - ([0-9_]+)',section)[1].replace('_',''))
meta['clap_geometry']={'cone_deg':rules['clap']['coneDeg'],'nominal_reach':rules['clap']['reach'],'near':max(0,inner-slack),'far':rules['clap']['reach']+slack,'close_ticks':rules['clap']['closeTicks'],'source':'Current canonical rules and unchanged crushing_clap hit test'}
(OUT/'manifest.json').write_text(json.dumps(meta,indent=2)+'\n')
(ROOT/'editor/basalt-art.js').write_text('window.BASALT_ART='+json.dumps(meta,separators=(',',':'))+';\n')

def paste(im,p,xy):im.paste(p,xy,p)
board=Image.new('RGB',(1560,8*200));draw=ImageDraw.Draw(board)
for row,(name,_,_,_) in enumerate(specs):
    for col,bg in enumerate(['#203b44','#b9c5a9']):
        x=col*780;y=row*200
        draw.rectangle((x,y,x+779,y+199),fill=bg)
        draw.text((x+12,y+10),name+' | right + left | native 1x and 3x',fill='#f4e8d7' if col==0 else '#152b34')
        for left in [False,True]:
            pose=ImageOps.mirror(poses[name]) if left else poses[name]
            # Native-size pair and tight 3x pair, all aligned to a foot baseline.
            box=pose.getbbox();tight=pose.crop(box)
            xx=x+14+(100 if left else 0)
            paste(board,tight,(xx,y+110-tight.height))
            big=tight.resize((tight.width*3,tight.height*3),NEAREST)
            bx=x+225+(280 if left else 0)
            paste(board,big,(bx,y+188-big.height))
board.save(OUT/'body-comparison.png')

compare=Image.new('RGB',(1160,390));draw=ImageDraw.Draw(compare)
for col,bg in enumerate(['#203b44','#b9c5a9']):
    start=col*580;draw.rectangle((start,0,start+579,389),fill=bg)
    ink='#f4e8d7' if col==0 else '#142a33'
    draw.text((start+12,12),'Identical display scale | stock / placeholder / Basalt',fill=ink)
    items=[]
    for name in ['knight','berserker']:
        a=json.loads((ROOT/f'Claude outputs/ref2/{name}_anim.json').read_text())['anims']['idle']['frames'][0]['data']
        im=Image.open(ROOT/f'Claude outputs/ref2/{name}_sheet.png').convert('RGBA')
        crop=im.crop((int(a['x']),int(a['y']),int(a['x']+a['w']),int(a['y']+a['h'])))
        items.append((name.title(),crop.crop(crop.getbbox())))
    a=json.loads((ROOT/'mods/tfm2_custom/champions/tfm2_custom_grimhald#anim.fanim').read_text())['anims']['idle']['frames'][0]['data']
    im=Image.open(ROOT/'mods/tfm2_custom/champions/tfm2_custom_grimhald#sheet.png').convert('RGBA')
    old=im.crop((a['x'],a['y'],a['x']+a['w'],a['y']+a['h']))
    items.extend([('Placeholder',old.crop(old.getbbox())),('Basalt',poses['idle'].crop(poses['idle'].getbbox()))])
    for i,(name,im) in enumerate(items):
        xx=start+18+i*140;draw.text((xx,45),name,fill=ink)
        paste(compare,im,(xx,130-im.height))
        big=im.resize((im.width*3,im.height*3),NEAREST)
        paste(compare,big,(xx,340-big.height))
        draw.text((xx,355),f'{im.height}px incl. gear',fill=ink)
compare.save(OUT/'stock-comparison.png')

fxboard=Image.new('RGB',(1280,1040));draw=ImageDraw.Draw(fxboard)
for i,(name,_,_) in enumerate(effect_specs):
    x=i%4*320;y=i//4*260;bg='#203b44' if (i%4+i//4)%2==0 else '#b9c5a9'
    draw.rectangle((x,y,x+319,y+259),fill=bg)
    draw.text((x+10,y+10),name+' | 1x + 2x',fill='#f6ead8' if bg=='#203b44' else '#142a33')
    r=eframes[name];im=effects.crop((r['x'],r['y'],r['x']+r['w'],r['y']+r['h']))
    paste(fxboard,im,(x+12,y+52))
    big=im.resize((im.width*2,im.height*2),NEAREST);paste(fxboard,big,(x+320-big.width-10,y+250-big.height))
fxboard.save(OUT/'effects-comparison.png')
print(json.dumps({'review_only':True,'body_poses':len(frames),'effects':len(eframes),'decoded_bytes':meta['decoded_review_atlas_bytes'],'native_assets_changed':False}))
