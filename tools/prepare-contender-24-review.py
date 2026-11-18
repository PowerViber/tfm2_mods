"""Reproduce approved 2.4 effects and code-native indicators from frozen sources."""
from pathlib import Path
import hashlib,json
from PIL import Image,ImageDraw
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/contender-24-art-review';OUT.mkdir(exist_ok=True)
N=Image.Resampling.NEAREST
atlas=Image.new('RGBA',(512,2048));anims={};x=y=rowh=0;cache={}
def place(im):
    global x,y,rowh
    key=(im.size,hashlib.sha256(im.tobytes()).digest())
    if key in cache:return cache[key]
    if x+im.width>512:x=0;y+=rowh;rowh=0
    assert y+im.height<=2048
    r=dict(x=x,y=y,w=im.width,h=im.height);atlas.alpha_composite(im,(x,y));x+=im.width;rowh=max(rowh,im.height);cache[key]=r;return r
def pack(tag,images,ticks):
    anims[tag]={'frames':[dict(duration=ticks/60/len(images),data=place(im)) for im in images]}
def crisp(im):
    im=im.copy();im.putalpha(im.getchannel('A').point(lambda a:255 if a>=180 else 0));return im.quantize(colors=32,method=Image.Quantize.FASTOCTREE,dither=Image.Dither.NONE).convert('RGBA')
anchors={};source_hashes={}
for name in ['break','aura']:
    source=OUT/f'{name}-source-v1.png';source_hashes[source.name]=hashlib.sha256(source.read_bytes()).hexdigest()
    im=Image.open(source).convert('RGBA');assert im.width==im.height and im.width%2==0;cell=im.width//2;images=[];anchors[name]=[]
    for i in range(4):
        tile=crisp(im.crop((i%2*cell,i//2*cell,(i%2+1)*cell,(i//2+1)*cell)))
        frame=Image.new('RGBA',(80,80));scaled=tile.resize((64,64),N)
        if name=='break':at=(8,8);anchor=[cell/2,cell/2]
        else:
            alpha=tile.getchannel('A');pixels=list(alpha.getdata());dense=[yy for yy in range(cell) if sum(v>0 for v in pixels[yy*cell:(yy+1)*cell])>=15]
            ground=max(dense)-cell*.04;at=(8,60-round(ground*64/cell));anchor=[cell/2,ground]
        frame.alpha_composite(scaled,at);images.append(frame);anchors[name].append(anchor)
    if name=='break':
        pack('break_t18',images,18)
        pack('break_t18_l',[im.transpose(Image.Transpose.FLIP_LEFT_RIGHT) for im in images],18)
    pack('full_rhythm_t480',images*10,480) if name=='aura' else None
    if name=='aura':
        # Single phases support stable clocks if a native controller renews the buff.
        for i,frame in enumerate(images):pack(f'full_rhythm_p{i}',[frame],480)

manifest=json.loads((ROOT/'docs/contender-shadow-review/presentation.json').read_text())
effects=Image.open(ROOT/manifest['effects']['file'].removeprefix('../')).convert('RGBA');r=manifest['effects']['anims']['uppercut']['frames'][0]['data']
fist=effects.crop((r['x']+10,r['y'],r['x']+36,r['y']+26)).resize((10,10),N)
digits=['111101101101111','010110010010111','111001111100111','111001111001111','101101111001001','111100111001111','111100111101111','111001001001001','111101111101111','111101111001111']
def number(im,n,at,colour):
    draw=ImageDraw.Draw(im)
    for k,c in enumerate(str(n)):
        for j,pixel in enumerate(digits[int(c)]):
            if pixel=='1':draw.point((at[0]+k*4+j%3,at[1]+j//3),fill=colour)
for count in range(21):
    im=Image.new('RGBA',(24,100))
    if count:
        gold=count==20;colour='#ffe0a0' if gold else '#b88aef';d=ImageDraw.Draw(im)
        d.rectangle((5,6,17,18),fill='#20132f',outline=colour);im.alpha_composite(fist,(6,7));d.rectangle((13,15,23,21),fill='#20132f');number(im,count,(14,16),colour)
    pack(f'enemy_{count}',[im],240)
for radius in [25,35]:
    im=Image.new('RGBA',(80,80));d=ImageDraw.Draw(im)
    d.ellipse((40-radius,40-radius,40+radius,40+radius),fill=(130,85,190,12),outline=(194,155,246,180),width=1)
    pack(f'ring{radius}_t6',[im],6)
for count in range(41):
    im=Image.new('RGBA',(40,100));d=ImageDraw.Draw(im)
    for i in range(8):
        value=max(0,min(5,count-i*5));colour='#ffe0a0' if count==40 else '#c7a0ff';cx=6+i*4
        for j,(dx,dy) in enumerate([(0,-1),(-1,0),(0,0),(1,0),(0,1)]):d.point((cx+dx,79+dy),fill=colour if j<value else '#4b3a63')
    pack(f'bank_{count}',[im],60)
atlas=atlas.crop((0,0,512,y+rowh));atlas.save(OUT/'rhythm-proposals.png')
approval_path=OUT/'approvals.json'
approved=approval_path.exists() and json.loads(approval_path.read_text())['effects']['approved']
if approved:
    assert json.loads(approval_path.read_text())['effects']['source_hashes']==source_hashes
data=dict(revision='contender-rhythm-art-1',rules='contender-2.4',approved=approved,
          file='../docs/contender-24-art-review/rhythm-proposals.png',anims=anims,source_hashes=source_hashes,
          source_anchors=anchors,atlas=list(atlas.size),decoded_bytes=atlas.width*atlas.height*4,
          generation_deterministic=False,packing_deterministic=True)
(OUT/'review.json').write_text(json.dumps(data,indent=2)+'\n')
(ROOT/'editor/contender-rhythm-art.js').write_text('window.CONTENDER_RHYTHM_ART='+json.dumps(data,separators=(',',':'))+';\n')
contract=dict(approval_required=True,candidate_asset='asset/tfm2_custom/vfx/contender_rhythm',
              break_effect=dict(name='ct_break_t18',tag='break_t18',ticks=18,follow=False),
              full_rhythm=dict(name='ct_rhythm_full_<phase>',tag='full_rhythm_t480',ticks=480,follow=True,
                               phase_tags=[f'full_rhythm_p{i}' for i in range(4)],note='Start once on Break, not on each Rhythm refresh; stable phase variants are provided.'),
              enemy=[dict(name=f'ct_rhythm_p{p}:{n}',tag=f'enemy_{n}') for p in range(10) for n in range(1,21)],
              bank=[dict(name=f'ct_bank:{n}',tag=f'bank_{n}') for n in range(41)],
              ownership_note='Track leases per owner/simulation. Do not remove another Contender owner\'s target markers; do not infer Break from aggregate HP changes.')
(OUT/'presentation-contract.json').write_text(json.dumps(contract,indent=2)+'\n')
if approved:
    stem=ROOT/'mods/tfm2_custom/vfx/contender_rhythm'
    atlas.save(str(stem)+'#sheet.png')
    Path(str(stem)+'#anim.fanim').write_text(json.dumps({'anims':anims},separators=(',',':'))+'\n')
    data['file']='../mods/tfm2_custom/vfx/contender_rhythm#sheet.png'
    (OUT/'review.json').write_text(json.dumps(data,indent=2)+'\n')
    (ROOT/'editor/contender-rhythm-art.js').write_text('window.CONTENDER_RHYTHM_ART='+json.dumps(data,separators=(',',':'))+';\n')
    manifest['rhythm']=data
    manifest['rhythm_bindings']=[dict(name='ct_'+tag,tag=tag,ticks=18 if tag.startswith('break') else 6,z=5 if tag.startswith('break') else -1,follow=False) for tag in anims if tag.startswith(('break','ring'))]
    manifest['rhythm_buffs']=[dict(name=f'ct_rhythm_full_{i}',tag=f'full_rhythm_p{i}',z=-1) for i in range(4)]
    manifest['rhythm_buffs'] += [dict(name=f'ct_rhythm_p{p}:{n}',tag=f'enemy_{n}',z=7) for p in range(10) for n in range(1,21)]
    manifest['rhythm_buffs'] += [dict(name=f'ct_bank:{n}',tag=f'bank_{n}',z=7) for n in range(41)]
    manifest['decoded_bytes']=sum(Image.open(ROOT/manifest[k]['file'].removeprefix('../')).width*Image.open(ROOT/manifest[k]['file'].removeprefix('../')).height*4 for k in ['body','effects','merge','rhythm'])
    manifest['revision']='contender-shadow-art-4'
    (ROOT/'docs/contender-shadow-review/presentation.json').write_text(json.dumps(manifest,indent=2)+'\n')
    (ROOT/'editor/contender-presentation.js').write_text('window.CONTENDER_PRESENTATION='+json.dumps(manifest,separators=(',',':'))+';\n')
print(json.dumps(dict(atlas=list(atlas.size),decoded_bytes=data['decoded_bytes'],sources=source_hashes,review_only=not approved)))
