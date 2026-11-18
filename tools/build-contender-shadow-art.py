"""Reproducible packing of approved imagegen drawings. Generation is not deterministic."""
from pathlib import Path
import hashlib, json, sys
from PIL import Image, ImageOps

ROOT=Path(__file__).resolve().parents[1]
DOC=ROOT/'docs/contender-shadow-review'
APP=json.loads((DOC/'approvals.json').read_text())
assert APP['body']['approved'] and APP['effects']['approved']
N=Image.Resampling.NEAREST
CW,CH,FOOT=80,64,(40,52)
def sha(p): return hashlib.sha256(p.read_bytes()).hexdigest()
def crisp(im):
    im=im.copy(); im.putalpha(im.getchannel('A').point(lambda a:255 if a>=180 else 0))
    return im.quantize(colors=32,method=Image.Quantize.FASTOCTREE,dither=Image.Dither.NONE).convert('RGBA')
def mirror(im):
    out=Image.new('RGBA',im.size);out.alpha_composite(ImageOps.mirror(im),(1,0));return out
source=Image.open(DOC/'body-animation-source-v1.png').convert('RGBA')
assert source.size==(1536,1024)
# Fixed source-cell landmarks; a single scale preserves anatomy, including death poses.
anchors=[(126,242),(127,242),(128,242),(127,242),(127,242),(127,242)]
tiles=[]
for row in range(4):
    for col in range(6):
        cell=crisp(source.crop((col*256,row*256,(col+1)*256,(row+1)*256)))
        cell=cell.resize((48,48),N)
        tile=Image.new('RGBA',(CW,CH));ax,ay=anchors[col]
        tile.alpha_composite(cell,(40-round(ax*3/16),52-round(ay*3/16)));tiles.append(tile)
sheet=Image.new('RGBA',(CW*12,CH*4))
rects=[]
for i,tile in enumerate(tiles):
    pair=[]
    for left in (False,True):
        n=i*2+int(left); x=n%12*CW;y=n//12*CH
        sheet.alpha_composite(mirror(tile) if left else tile,(x,y));pair.append(dict(x=x,y=y,w=CW,h=CH))
    rects.append(pair)
poses={
 'idle':([0,0,6,0],48),'run':([1,2,3,4,5,0],48),
 'attack':([6,6,7,8],18),'jab':([6,6,7,6],18),'cross':([6,6,8,6],18),
 'projection':([12,13,14,0],18),'cyclone':([6,11,9,11],12),
 'slam':([15,16,16,14],20),'uppercut':([9,9,10,10,0],30),
 'flurry':([12,15,16,16,0],24),'hit':([17,17,0],14.4),'dead':(list(range(18,24)),43.2),
 # Existing embedded definitions retain all names and template durations.
 'combination':([6,7,6,8,8,0],36),'bodyShot':([6,6,9,9,0],21),
 'slip':([14,14,14,0],12),'guard':([12,12,12,0],27),'mainEvent':([12,15,16,16,0],39),
 'skill1':([12,13,14,0,0,0],36),'skill2':([6,11,9,0],27),'ult':([12,15,16,16,0],39)}
anims={}
for name,(ids,ticks) in poses.items():
    for left in (False,True):
        anims[name+('_l' if left else '')]={'frames':[{'duration':ticks/60/len(ids),'data':rects[i][int(left)]} for i in ids]}
bodybase=ROOT/'mods/tfm2_custom/champions/tfm2_custom_contender'
sheet.save(str(bodybase)+'#sheet.png')
Path(str(bodybase)+'#anim.fanim').write_text(json.dumps({'anims':anims},separators=(',',':'))+'\n')

spec=json.loads((DOC/'effect-review-manifest.json').read_text())
concept=Image.open(DOC/'effects-proposals.png').convert('RGBA')
# Use extracted approved pixels; no procedural replacement drawings.
effects=spec['effects'] if 'effects' in spec else spec['frames']
parts={}
for key,r in effects.items():
    parts[key]=concept.crop((r['x'],r['y'],r['x']+r['w'],r['y']+r['h']))
fxsheet=Image.new('RGBA',(1024,2048));fxanims={};bindings=[];x=y=rowh=0
def put(im):
    global x,y,rowh
    if x+im.width>1024:x=0;y+=rowh;rowh=0
    assert y+im.height<=2048
    r=dict(x=x,y=y,w=im.width,h=im.height);fxsheet.alpha_composite(im,(x,y));x+=im.width;rowh=max(rowh,im.height);return r
def pack(name,ims,ticks,z=5,follow=False):
    fxanims[name]={'frames':[dict(duration=ticks/60/len(ims),data=put(im)) for im in ims]}
    bindings.append(dict(name='ct_'+name,tag=name,ticks=ticks,z=z,follow=follow))
def canvas(im,size=(80,80)):
    out=Image.new('RGBA',size);out.alpha_composite(im,((size[0]-im.width)//2,(size[1]-im.height)//2));return out
def spectral(im):
    out=im.copy();p=out.load()
    for yy in range(out.height):
        for xx in range(out.width):
            r,g,b,a=p[xx,yy]
            if a:p[xx,yy]=(round(r*.3+93),round(g*.3+63),round(b*.3+140),round(a*.8))
    return out
parts['shadow']=canvas(spectral(tiles[11]),(80,64))
lifetimes={'launch':[18],'swap':[18],'cross':[10,18],'cleave':[12,14],'slam':[20],
 'uppercut':[20],'echo':[12],'vanish':[24],'appear':[12,18],'flurry':[8],'haymaker':[18],'shadow':[54]}
for name in lifetimes:
    base=parts[name]
    if name=='cleave':base=base.resize((50,50),N)
    for left in (False,True):
        im=ImageOps.mirror(base) if left else base
        for ticks in lifetimes[name]:
            # One accepted event: material expands once, then fades. Never renewed.
            seq=[]
            for scale,alpha in [(0.75,.8),(1,1),(1,.75),(1,.3)]:
                frame=Image.new('RGBA',im.size);s=im.resize((max(1,round(im.width*scale)),max(1,round(im.height*scale))),N)
                s.putalpha(s.getchannel('A').point(lambda a:round(a*alpha)))
                frame.alpha_composite(s,((im.width-s.width)//2,(im.height-s.height)//2));seq.append(frame)
            if name=='shadow':
                seq=[spectral(mirror(tiles[i]) if left else tiles[i]) for i in [6,11,9,8,10,11]]
            pack(f'{name}_t{ticks}'+('_l' if left else ''),seq,ticks,-2 if name=='vanish' else 5)
    # Compatibility alias for original round-178 names.
    pack(name,[base],{'shadow':54,'vanish':24,'flurry':8}.get(name,18),-2 if name=='vanish' else 5)
# Ten-tick, following carrier leases: stable pose index chosen by simulation clock.
for pose,ids in [('idle',[0]),('run',[1,2,3,4,5,0]),('punch',[7,8])]:
    for j,i in enumerate(ids):
        for left in (False,True):
            pack(f'body_{pose}{j}'+('_l' if left else ''),[spectral(mirror(tiles[i]) if left else tiles[i])],10,4,True)
for count in range(6):
    marks=Image.new('RGBA',(40,64));p=parts['charges'].resize((30,20),N)
    # Approved five-pip strip; reveal a proportional number of pips.
    marks.alpha_composite(p.crop((0,0,round(p.width*count/5),p.height)),(5,0))
    pack(f'charges{count}',[marks],60,6,True)
# The approved fist travels into the contact point from eight directions. Its
# contact landmark stays at the same world point; no full-body sprite is rotated.
for heading in range(8):
    seq=[]
    for travel,alpha in [(-12,.65),(-6,1),(0,1),(0,.25)]:
        frame=Image.new('RGBA',(128,128));fist=parts['flurry'].copy()
        fist.putalpha(fist.getchannel('A').point(lambda a:round(a*alpha)))
        frame.alpha_composite(fist,(64-48+travel,64-15))
        seq.append(frame.rotate(-heading*45,resample=N))
    pack(f'flurry_d{heading}_t8',seq,8,5)
fxsheet=fxsheet.crop((0,0,1024,y+rowh))
fxbase=ROOT/'mods/tfm2_custom/vfx/contender_shadow'
fxsheet.save(str(fxbase)+'#sheet.png');Path(str(fxbase)+'#anim.fanim').write_text(json.dumps({'anims':fxanims},separators=(',',':'))+'\n')
manifest={'revision':'contender-shadow-art-2','rules':'contender-2.2','flurry_headings':8,'canvas':[CW,CH],'foot_anchor':list(FOOT),
 'body':{'file':'../mods/tfm2_custom/champions/tfm2_custom_contender#sheet.png','anims':anims},
 'effects':{'file':'../mods/tfm2_custom/vfx/contender_shadow#sheet.png','anims':fxanims},'bindings':bindings,
 'source_hashes':{p.name:sha(p) for p in [DOC/'body-source-v1.png',DOC/'body-animation-source-v1.png',DOC/'effects-source-v1.png']},
 'decoded_bytes':sheet.width*sheet.height*4+fxsheet.width*fxsheet.height*4,'generation_deterministic':False,'packing_deterministic':True}
assert manifest['decoded_bytes']<=32*1024*1024
(DOC/'presentation.json').write_text(json.dumps(manifest,indent=2)+'\n')
(ROOT/'editor/contender-presentation.js').write_text('window.CONTENDER_PRESENTATION='+json.dumps(manifest,separators=(',',':'))+';\n')
print(json.dumps({'body':sheet.size,'fx':fxsheet.size,'decoded_bytes':manifest['decoded_bytes'],'animations':len(fxanims)}))
