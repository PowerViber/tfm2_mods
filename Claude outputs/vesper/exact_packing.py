"""Share quantized frames without changing any pixel, duration, or frame-centre anchor."""
import json,re,hashlib,os,io
from pathlib import Path
from zipfile import ZipFile,ZIP_DEFLATED
from PIL import Image
REFERENCE=Path(__file__).with_name('quantization-reference.zip')
def quantize(name,sheet):
    """Lock the shipped quantization when the rendered source is unchanged.

    Pillow's octree palette choices differ between runtimes. The reference is a
    hashed source cache, not a replacement drawing pipeline: changed source pixels
    invalidate it, and the equivalence audit then requires an explicit art update.
    """
    raw=sheet.convert('RGBA');digest=hashlib.sha256(raw.tobytes()).hexdigest()
    source=os.environ.get('VESPER_REFERENCE_SOURCE')
    if source:
        original=Path(source)/'mods/tfm2_custom/vfx'/(name+'#sheet.png')
        assert original.is_file(),original
        prior=Image.open(original).convert('RGBA');assert prior.size==raw.size,(name,prior.size,raw.size)
        with ZipFile(REFERENCE,'a',compression=ZIP_DEFLATED)as z:
            filename=name+'.png';assert filename not in z.namelist(),name
            z.writestr(filename,original.read_bytes());z.writestr(name+'.json',json.dumps({'sourceRgbaHash':digest,'size':raw.size,'reference':'30f2969'}))
        return prior
    if REFERENCE.exists():
        with ZipFile(REFERENCE)as z:
            key=name+'.json'
            if key in z.namelist()and json.loads(z.read(key))['sourceRgbaHash']==digest:
                return Image.open(io.BytesIO(z.read(name+'.png'))).convert('RGBA')
    return sheet.quantize(colors=256,method=Image.Quantize.FASTOCTREE,dither=Image.Dither.NONE)
def pack(folder,name,sheet,anims):
    source=sheet.convert('RGBA');groups={}
    for tag,anim in anims.items():
        if name=='vesper_outfits':group=name+'_r'+re.search(r'_r(\d+)',tag).group(1)
        elif name=='vesper_hud':group=name+'_'+('orbs'if tag.startswith('orbs')else'hex'if tag.startswith('hex')else'core'if tag.startswith('core')else'stress')
        elif name=='vesper_marks':group=name+'_'+('back'if tag.startswith('back_')else'top'if tag.startswith('top')else'rank')
        else:group=name
        groups.setdefault(group,{})[tag]=anim
    for group,tags in groups.items():
        unique=[];lookup={};refs={}
        for tag,anim in tags.items():
            refs[tag]=[]
            for fr in anim['frames']:
                r=fr['data'];x,y,w,h=[int(r[k])for k in('x','y','w','h')];im=source.crop((x,y,x+w,y+h));key=(w,h,im.tobytes())
                if key not in lookup:lookup[key]=len(unique);unique.append(im)
                refs[tag].append((lookup[key],fr['duration']))
        order=sorted(range(len(unique)),key=lambda i:(-unique[i].height,-unique[i].width,i))
        best=None
        for limit in range(max(im.width for im in unique),2049,16):
            x=y=row=used=0;positions={}
            for i in order:
                im=unique[i]
                if x+im.width>limit:x=0;y+=row;row=0
                positions[i]=(x,y);x+=im.width;used=max(used,x);row=max(row,im.height)
            height=y+row
            if height<=2048 and(best is None or used*height<best[1]*best[2]):best=(positions,used,height)
        assert best is not None,(group,len(unique))
        positions,w,h=best;out=Image.new('RGBA',(w,h))
        # paste, not alpha_composite: retain exact RGB values beneath partially transparent pixels.
        for i,im in enumerate(unique):out.paste(im,positions[i])
        metadata={tag:{'frames':[{'duration':duration,'data':{'x':positions[i][0],'y':positions[i][1],'w':unique[i].width,'h':unique[i].height}}for i,duration in frames]}for tag,frames in refs.items()}
        out.save(folder/(group+'#sheet.png'))
        (folder/(group+'#anim.fanim')).write_text(json.dumps({'anims':metadata},indent=2)+'\n',encoding='utf-8')
