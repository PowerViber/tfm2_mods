"""Pack frozen original whole-body art and rigid weapons; no anatomy slicing.

Review outputs always build. Approved groups alone are copied into mod assets.
"""
from pathlib import Path
import hashlib
import json
import math
import shutil
from PIL import Image, ImageDraw, ImageOps

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / 'docs/kratos-art-review'
ART = json.loads((ROOT / 'rules/kratos-art.json').read_text())
RULES = json.loads((ROOT / 'rules/kratos.json').read_text())
APPROVAL = json.loads((OUT / 'approvals.json').read_text())
N = Image.Resampling.NEAREST


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def opaque(im):
    im = im.convert('RGBA')
    im.putalpha(im.getchannel('A').point(lambda a: 255 if a >= 180 else 0))
    return im


def largest(im):
    """Discard extraction specks outside the complete connected source drawing."""
    im = opaque(im)
    a = im.getchannel('A')
    # Pillow floodfill in mask space keeps original colour pixels untouched.
    mask = a.copy()
    groups = []
    while box := mask.getbbox():
        seed = next((x, box[1]) for x in range(box[0], box[2]) if mask.getpixel((x, box[1])))
        part = Image.new('L', im.size)
        ImageDraw.floodfill(mask, seed, 128)
        part = mask.point(lambda p: 255 if p == 128 else 0)
        groups.append((sum(part.histogram()[1:]), part))
        mask.paste(0, (0, 0), part)
    if groups:
        keep = max(groups, key=lambda p: p[0])[1]
        im.putalpha(keep)
    return im


def cells(path, columns, rows, clean=False):
    source = opaque(Image.open(path))
    out = []
    for i in range(columns * rows):
        cell = source.crop((i % columns * source.width // columns, i // columns * source.height // rows,
                            (i % columns + 1) * source.width // columns, (i // columns + 1) * source.height // rows))
        out.append(largest(cell) if clean else cell)
    return out


def pack(animations, width=1024):
    x = y = row = 0
    unique = {}
    metadata = {}
    for name, frames in animations.items():
        entries = []
        for image, ticks in frames:
            key = (image.size, image.tobytes())
            if key not in unique:
                if x + image.width > width:
                    x = 0
                    y += row + 2
                    row = 0
                assert y + image.height <= 2048, name
                rect = dict(x=x, y=y, w=image.width, h=image.height)
                unique[key] = (image, rect)
                x += image.width + 2
                row = max(row, image.height)
            entries.append({'duration': ticks / 60, 'data': unique[key][1]})
        metadata[name] = {'frames': entries}
    height = 2 ** math.ceil(math.log2(max(2, y + row + 2)))
    sheet = Image.new('RGBA', (width, height))
    for image, rect in unique.values():
        sheet.alpha_composite(image, (rect['x'], rect['y']))
    return sheet, {'anims': metadata}


def write_atlas(stem, animations, width=1024):
    sheet, metadata = pack(animations, width)
    sheet.save(OUT / (stem + '.png'))
    (OUT / (stem + '.fanim')).write_text(json.dumps(metadata, indent=2) + '\n')
    return sheet, metadata


def body_frames():
    result = {}
    for group, poses in ART['source_groups'].items():
        frames = cells(OUT / (group + '-source-v1.png'), 8, 3, True)
        # Every sheet begins with the same standing reference. One scale for
        # that sheet preserves crouches and falls rather than inflating them.
        first = frames[0].getbbox()
        scale = 40 / (first[3] - first[1])
        for row, pose in enumerate(poses):
            result[pose] = []
            for i, frame in enumerate(frames[row * 8:row * 8 + 8]):
                crop = frame.crop(frame.getbbox())
                small = opaque(crop.resize((max(1, round(crop.width * scale)), max(1, round(crop.height * scale))), N))
                canvas = Image.new('RGBA', tuple(ART['body_canvas']))
                canvas.alpha_composite(small, ((canvas.width - small.width) // 2, ART['body_anchor'][1] + 1 - small.height))
                box = canvas.getbbox()
                assert box and box[0] > 0 and box[1] > 0 and box[2] < canvas.width and box[3] == 53, (pose, i, box)
                result[pose].append(canvas)
    return result


def props():
    result = {}
    for name, size, cell in zip(('axe', 'blade', 'shield'), ((14, 30), (8, 20), (18, 18)), cells(OUT / 'props-source-v1.png', 3, 1, True)):
        im = cell.crop(cell.getbbox())
        im.thumbnail(size, N)
        result[name] = opaque(im)
    return result


# Each entry is a closed hand location in a complete 64x64 drawing, and the
# rotation of a rigid prop. This never rotates or cuts a torso, arm or leg.
RIG = {
    'idle': [(24,34,-115),(24,34,-115),(24,34,-115),(24,34,-115),(24,34,-115),(24,34,-115),(24,34,-115),(24,34,-115)],
    'run': [(25,34,-110),(24,33,-110),(26,32,-100),(25,33,-110),(24,35,-120),(25,35,-120),(26,34,-110),(25,34,-110)],
    'attack': [(24,34,-110),(29,33,30),(26,31,55),(23,30,75),(25,29,120),(27,35,200),(28,35,225),(24,34,-110)],
    'sight': [(24,34,-115),(24,34,-115),(24,34,-115),(25,33,-115),(24,34,-115),(23,35,-115),(24,34,-115),(24,34,-115)],
    'guard': [(24,34,-115),(24,33,-110),(24,34,-110),(24,34,-110),(24,34,-110),(24,34,-110),(24,34,-110),(24,34,-115)],
    'chain': [(24,34,-115),(25,31,65),(37,31,95),(40,29,100),(36,35,115),(36,38,125),(27,35,-110),(24,34,-115)],
    'cleave': [(24,34,-115),(30,36,-25),(31,31,20),(31,23,30),(32,17,35),(33,29,100),(33,36,150),(33,42,170)],
    'sweep': [(24,34,-115),(22,35,-90),(21,32,-60),(19,31,-30),(18,32,0),(22,35,60),(27,37,130),(28,39,160)],
    'smash': [(24,34,-115),(22,31,-20),(22,21,20),(23,14,25),(30,25,65),(33,44,175),(32,49,175),(32,45,160)],
    'ult': [(24,34,-115),(24,34,-115),(24,34,-115),(24,34,-115),(24,34,-115),(24,34,-115),(24,34,-115),(24,34,-115)],
    'hit': [(24,34,-115)] * 8,
    'dead': [None] * 8,
}


def weapon_frame(pose, index, blades, material):
    image = Image.new('RGBA', tuple(ART['weapon_canvas']))
    spec = RIG[pose][index]
    if spec is None:
        return image
    x, y, angle = spec
    if pose in ('idle','sight','ult','hit'):
        x, y, angle = (24 if blades else 41), 35, 180
    elif pose == 'guard':
        x, y, angle = 23, 35, 180
    elif pose == 'run' and not blades:
        x, y, angle = 40, y, 170
    weapon = material['blade' if blades else 'axe']
    grip = (weapon.width // 2, round(weapon.height * (.73 if blades else .72)))
    for hand, rotation in ([(x, angle), (64 - x, -angle)] if blades else [(x, angle)]):
        pivot = Image.new('RGBA', (96, 96))
        pivot.alpha_composite(weapon, (48 - grip[0], 48 - grip[1]))
        pivot = pivot.rotate(rotation, N, center=(48,48))
        image.alpha_composite(pivot, (hand + 16 - 48, y + 16 - 48))
    return image


def effects():
    mats = cells(OUT / 'effects-source-v1.png', 4, 2)
    specs = {
        'cleave': (0, 34, 34, 18, 6, False), 'sweep': (1, 34, 34, 18, 6, False),
        'smash': (2, 47, 47, 18, -2, False), 'sight': (3, 24, 24, 12, 5, True),
        'guard': (4, 24, 24, 180, 5, True), 'freeze': (5, 24, 32, 36, 4, False),
        'fenrir_spawn': (6, 40, 26, 24, -2, False), 'fenrir_slam': (7, 42, 32, 18, 6, False),
        'chain_hit': (1, 20, 20, 12, 6, False), 'dash_trail': (1, 22, 16, 12, 3, False),
        'swap_axe': (0, 22, 22, 12, 5, True), 'swap_blades': (1, 22, 22, 12, 5, True),
        'knock_up': (7, 24, 28, 18, 4, False), 'fenrir_fade': (6, 40, 26, 18, 2, False),
        'burn': (1, 12, 16, 60, 4, True), 'frozen': (5, 24, 32, 36, 4, True),
        'sight_aura': (3, 24, 24, 90, 5, True), 'guard_aura': (4, 24, 24, 60, 5, True),
    }
    result = {}
    bindings = {}
    for name, (source, w, h, total, z, follow) in specs.items():
        crop = mats[source].crop(mats[source].getbbox())
        frames = []
        for i in range(6):
            phase = i / 5
            size = (.5 + .5 * min(1, phase * 2)) if name not in ('guard','guard_aura','frozen','sight_aura') else 1
            small = opaque(crop.resize((max(1,round(w * size)),max(1,round(h * size))),N))
            small.putalpha(small.getchannel('A').point(lambda a: round(a*(1-.75*phase))))
            canvas = Image.new('RGBA',(w+4,h+4))
            canvas.alpha_composite(small,((canvas.width-small.width)//2,(canvas.height-small.height)//2))
            frames.append((canvas,total/6))
        result[name] = frames
        bindings[name] = {'kind': 'buff' if name in ('burn','frozen','sight_aura','guard_aura') else 'effect', 'z':z,'follow':follow}
        if name in ('cleave','sweep'):
            result[name+'_l'] = [(ImageOps.mirror(im),t) for im,t in frames]
            bindings[name+'_l'] = dict(bindings[name])
    # Ground geometry markers and the chain use canonical code-native geometry.
    # Artwork does not add damage, displacement or collision.
    for count in (1,2):
        im=Image.new('RGBA',(20,12));d=ImageDraw.Draw(im)
        for i in range(count):d.polygon([(5+i*8,1),(8+i*8,5),(5+i*8,9),(2+i*8,5)],fill='#97dcec',outline='#324e65')
        result['frost'+str(count)]=[(im,60)]
        bindings['frost'+str(count)]={'kind':'buff','z':4,'follow':True}
    im=Image.new('RGBA',(24,10));d=ImageDraw.Draw(im)
    for x in range(0,21,5):d.ellipse((x,3,x+5,7),outline='#c89d61',width=1)
    result['chain']=[(im,12)];bindings['chain']={'kind':'effect','z':6,'follow':False}
    for mode,reach in [('short',RULES['dash']['range']),('long',RULES['dash']['linkedRange'])]:
        length=round(reach/ART['units_per_pixel']);size=length+10
        for heading in range(8):
            chain=Image.new('RGBA',(size,size));d=ImageDraw.Draw(chain);y=size//2;x0=(size-length)//2
            for x in range(x0,x0+length-3,4):d.ellipse((x,y-1,x+5,y+1),outline='#c89d61')
            chain=chain.rotate(-45*heading,N,center=(size//2,size//2))
            tag='chain_'+mode+'_p'+str(heading);result[tag]=[(chain,12)];bindings[tag]={'kind':'effect','z':6,'follow':False}
    wolf_path=OUT/'fenrir-run-source-v1.png'
    runs=cells(wolf_path,4,2,True) if wolf_path.exists() else cells(OUT/'fenrir-source-v1.png',2,2,True)[:1]
    wolf_scale=25/(runs[0].getbbox()[3]-runs[0].getbbox()[1])
    for heading in range(8):
        for phase,cell in enumerate(runs):
            wolf=cell.crop(cell.getbbox());wolf=opaque(wolf.resize((round(wolf.width*wolf_scale),round(wolf.height*wolf_scale)),N))
            canvas=Image.new('RGBA',(80,80));canvas.alpha_composite(wolf,((80-wolf.width)//2,(80-wolf.height)//2))
            canvas=canvas.rotate(-45*heading,N,center=(40,40))
            tag='fenrir_p'+str(heading)+('_f'+str(phase) if phase else '')
            result[tag]=[(canvas,1)]
            bindings[tag]={'kind':'effect','z':5,'follow':False}
    return result,bindings


def main():
    body=body_frames(); material=props(); base={};accessory={};preview={}
    for pose,frames in body.items():
        for left in (False,True):
            suffix='_l' if left else ''
            base[pose+suffix]=[(ImageOps.mirror(im) if left else im,t) for im,t in zip(frames,ART['pose_ticks'][pose])]
            for blades in (False,True):
                stance='blades' if blades else 'axe'
                assembled=[]
                for i,(im,t) in enumerate(zip(frames,ART['pose_ticks'][pose])):
                    # Rampage's three stages show their real weapon independent
                    # of the selected basic-attack stance.
                    fire=pose=='sweep' or (blades and pose not in ('cleave','smash','guard'))
                    weapon=weapon_frame(pose,i,fire,material)
                    if pose=='smash':weapon.alpha_composite(weapon_frame(pose,i,True,material))
                    tag=ART['weapon_prefix']+stance+'_'+pose+('_l' if left else '_r')+str(i)
                    accessory[tag]=[(ImageOps.mirror(weapon) if left else weapon,1)]
                    whole=weapon.copy();whole.alpha_composite(im,(16,16));whole=ImageOps.mirror(whole) if left else whole
                    assembled.append((whole,t))
                preview[stance+'_'+pose+suffix]=assembled
    for alias,pose in ART['aliases'].items():
        for suffix in ('','_l'):base[alias+suffix]=base[pose+suffix]
    bs,bm=write_atlas('body-preview',base);ws,wm=write_atlas('weapons-preview',accessory)
    fspec,bindings=effects();fs,fm=write_atlas('effects-preview',fspec)
    ps,pm=write_atlas('assembled-preview',preview,2048)
    gallery=Image.new('RGB',(1024,len(body)*140),'#19313b');draw=ImageDraw.Draw(gallery)
    for row,pose in enumerate(body):
        draw.text((8,row*140+4),pose,fill='#f5bb82')
        for i in range(8):
            frame=preview['axe_'+pose][i][0];small=frame.resize((128,128),N)
            gallery.paste(small,(i*128,row*140+12),small)
    gallery.save(OUT/'animation-contact.png')
    for pose in body:
        panels=[]
        for i in range(8):
            panel=Image.new('RGB',(1040,350));d=ImageDraw.Draw(panel)
            for col,(stance,left) in enumerate([('axe',False),('axe',True),('blades',False),('blades',True)]):
                x=col*260;d.rectangle((x,0,x+259,349),fill='#b6c4a4' if left else '#203747')
                d.text((x+8,8),f'{stance} {pose} {i+1}/8',fill='#293d43' if left else '#ffc17e')
                im=preview[stance+'_'+pose+('_l' if left else '')][i][0]
                panel.paste(im,(x+20,35),im);big=im.resize((288,288),N);panel.paste(big,(x-14,85),big)
            panels.append(panel)
        panels[0].save(OUT/('animation-'+pose+'.gif'),save_all=True,append_images=panels[1:],duration=[round(t*1000/60) for t in ART['pose_ticks'][pose]],loop=0,disposal=2)
    board=Image.new('RGB',(1040,980),'#172936');d=ImageDraw.Draw(board)
    for row,(name,frames) in enumerate([(n,f) for n,f in fspec.items() if not n.startswith('fenrir_p')][:20]):
        x=(row%4)*260;y=(row//4)*190;d.text((x+8,y+8),name,fill='#ffbf81')
        im=frames[1][0];board.paste(im,(x+10,y+50),im);big=im.resize((im.width*3,im.height*3),N);board.paste(big,(x+90,y+35),big)
    board.save(OUT/'effects-comparison.png')
    all_sources={p.name:sha(p) for p in sorted(OUT.glob('*-source-v1.png'))}
    approved_body=all(APPROVAL.get(g,{}).get('approved',False) for g in ('body','fenrir')) and all(sha(OUT/n)==v for n,v in APPROVAL['source_sha256'].items())
    approved_effects=APPROVAL.get('effects',{}).get('approved',False) and APPROVAL.get('effects',{}).get('source_sha256')==sha(OUT/'effects-source-v1.png')
    manifest={**ART,'review_only':not approved_effects,'complete_body':True,'body_approved':approved_body,'effects_approved':approved_effects,'source_sha256':all_sources,'effects':bindings,
              'atlases':{'body':'body-preview','weapons':'weapons-preview','effects':'effects-preview','assembled':'assembled-preview'},
              'decoded_bytes':sum(im.width*im.height*4 for im in (bs,ws,fs)),
              'frame_counts':{p:len(f) for p,f in body.items()},'method':'Original whole-body imagegen sources; alpha extraction, rigid props and deterministic atlas packing. AI generation itself is not deterministic.'}
    manifest['selection']={'fenrir_phase_ticks':3,'fenrir_variants':len(runs) if 'runs' in locals() else (8 if (OUT/'fenrir-run-source-v1.png').exists() else 1),'chain_split_range':RULES['dash']['range']+1000}
    assert manifest['decoded_bytes']<=32*1024*1024
    (OUT/'art-manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
    (OUT/'presentation.json').write_text(json.dumps(manifest,indent=2)+'\n')
    # Per-frame sources are frozen above; game integration is group-gated.
    if approved_body:
        for folder,stem,source in [('champions',RULES['id'],'body-preview'),('vfx','kratos_weapons','weapons-preview')]:
            for ext,suffix in [('png','#sheet.png'),('fanim','#anim.fanim')]:
                target=ROOT/'mods/tfm2_custom'/folder/(stem+suffix);shutil.copy2(OUT/(source+'.'+ext),target)
    if approved_effects:
        for ext,suffix in [('png','#sheet.png'),('fanim','#anim.fanim')]:shutil.copy2(OUT/('effects-preview.'+ext),ROOT/'mods/tfm2_custom/vfx'/('kratos'+suffix))
    print(json.dumps({'body_approved':approved_body,'effects_approved':approved_effects,'decoded_bytes':manifest['decoded_bytes'],'frames':manifest['frame_counts']}))


if __name__=='__main__':main()
