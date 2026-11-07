"""Integrate visible approved base bodies and retire incompatible clothing pixels.

Keeps the native DLL, every buff/command identity and the complete FANIM timing
contracts unchanged. Legacy clothing aliases become transparent presentation
layers; they do not hide the actual body. Independent FX and keyboards retain
identical decoded frame pixels. No game installation or save migration.
"""
from __future__ import annotations
import hashlib,io,json,re,subprocess,sys
from pathlib import Path
from PIL import Image,ImageDraw

sys.path.insert(0,str(Path(__file__).resolve().parent))
from importlib.util import spec_from_file_location,module_from_spec
spec=spec_from_file_location('rankpack',Path(__file__).with_name('pack-ranked-bodies.py'))
packer=module_from_spec(spec);spec.loader.exec_module(packer)
ROOT=packer.ROOT;OUT=packer.OUT
MANIFEST=ROOT/'rules/ranked-body-art.json'


def original(rel):
    base=json.loads(MANIFEST.read_text(encoding='utf8'))['baseline']
    return subprocess.check_output(['rtk','proxy','git','show',base+':'+rel],cwd=ROOT)


def retire(base, predicate):
    rel=base.relative_to(ROOT).as_posix()
    meta=json.loads(original(rel+'#anim.fanim'))
    source=Image.open(io.BytesIO(original(rel+'#sheet.png'))).convert('RGBA')
    images={};retired=[];preserved=[]
    for tag,anim in meta['anims'].items():
        for i,f in enumerate(anim['frames']):
            r=f['data'];im=source.crop((r['x'],r['y'],r['x']+r['w'],r['y']+r['h']))
            if predicate(tag):
                im=Image.new('RGBA',im.size);retired.append([tag,i])
            else:
                entry={'tag':tag,'frame':i,'sha256':hashlib.sha256(im.tobytes()).hexdigest()}
                if base.name=='coder_rank' and re.fullmatch(r'kb[0-8]',tag):
                    shifted=Image.new('RGBA',im.size);shifted.paste(im,(0,-4))
                    restored=Image.new('RGBA',im.size);restored.paste(shifted,(0,4))
                    assert restored.tobytes()==im.tobytes(),'Keyboard shift clips pixels'
                    im=shifted;entry['translation']=[0,-4]
                preserved.append(entry)
            images[(tag,i)]=im
    atlas,packed,_=packer.pack(images,meta)
    atlas.save(str(base)+'#sheet.png');packer.write(str(base)+'#anim.fanim',packed)
    for p in preserved:
        r=packed['anims'][p['tag']]['frames'][p['frame']]['data']
        im=atlas.crop((r['x'],r['y'],r['x']+r['w'],r['y']+r['h']))
        if 'translation'in p:
            restore=Image.new('RGBA',im.size);restore.paste(im,tuple(-n for n in p['translation']));im=restore
        assert hashlib.sha256(im.tobytes()).hexdigest()==p['sha256']
    for tag,anim in meta['anims'].items():
        for a,b in zip(anim['frames'],packed['anims'][tag]['frames']):
            assert a['duration']==b['duration'] and [a['data'][k] for k in ['w','h']]==[b['data'][k] for k in ['w','h']]
    return {'asset':rel,'retiredBodyFrames':len(retired),'preservedFrames':preserved,
            'decodedBytes':atlas.width*atlas.height*4,'sha256':packer.digest(str(base)+'#sheet.png')}


def main():
    manifest=json.loads(MANIFEST.read_text(encoding='utf8'))
    assert manifest['approval']['status']=='approved'
    packer.main(integrate=True)
    retired=[]
    vfx=ROOT/'mods/tfm2_custom/vfx'
    for i in range(11):
        retired.append(retire(vfx/f'vesper_outfits_r{i}',lambda tag: True))
        retired.append(retire(vfx/f'magnus_outfit_{i}',lambda tag: True))
        # These legacy cast sheets contain only the old full-body drawing.
        # Command/geometry effects and PLAN are separate assets and stay intact.
        retired.append(retire(vfx/f'magnus_casts_{i}',lambda tag: True))
    retired.append(retire(vfx/'coder_rank',lambda tag: re.fullmatch(r'fit[0-8]',tag) is not None))
    for c in manifest['characters']:
        p=ROOT/'mods/tfm2_custom/champion'/f"{c['id']}.data_champion"
        # Preserve any explicit current definition edits; this art pipeline adds
        # its marker without restoring older combat numbers from the audit ref.
        data=json.loads(p.read_text(encoding='utf8'))
        data['passive'].setdefault('params',{})['rank_art_v2']=1
        packer.write(p,data)
    manifest['revision']='rank-art-2.0-base-fallback-1'
    manifest['status']='base-integrated-rank-animation-gated'
    manifest['optIn']['integrated']=True
    manifest['presentation']={'nativeMode':'visible-base-fallback','rankAnimations':'gated',
        'reason':'Stable API provides no current body animation frame/facing; do not infer exact layer alignment',
        'legacyClothing':'transparent aliases with original dimensions, frame counts and durations',
        'rankBadgesAndFunctionalDisplays':'unchanged','nativeDll':'unchanged'}
    packer.write(MANIFEST,manifest)
    packer.write(OUT/'fallback-integration.json',{'retired':retired,'nativeDllUnchanged':True,
        'reason':manifest['presentation']['reason'],'rankAnimationsComplete':False,
        'installationPerformed':False,'careerMigrationPerformed':False,'actualGameAcceptance':'pending'})
    # Header preview must show the actual candidate body, not an old stock-derived thumbnail.
    preview_path=ROOT/'editor/vesper-portraits.png'
    size=Image.open(preview_path).size
    preview=Image.new('RGBA',size,(22,25,46,255));draw=ImageDraw.Draw(preview)
    body=Image.open(ROOT/'mods/tfm2_custom/champions/tfm2_custom_vesper#sheet.png').convert('RGBA')
    r=json.loads((ROOT/'mods/tfm2_custom/champions/tfm2_custom_vesper#anim.fanim').read_text())['anims']['idle']['frames'][0]['data']
    frame=body.crop((r['x'],r['y'],r['x']+r['w'],r['y']+r['h']))
    definition=json.loads((ROOT/'mods/tfm2_custom/champion/tfm2_custom_vesper.data_champion').read_text())
    for i,(label,name) in enumerate([('Spark','vs_rank0'),('Riftwalker','vs_rank6'),('#1 / base fallback','vs_top1')]):
        cell=size[0]//3;cx=round((i+.5)*cell)
        art=frame.resize((frame.width*2,frame.height*2),Image.Resampling.NEAREST)
        preview.alpha_composite(art,(cx-art.width//2,round(size[1]*.67)-art.height))
        entry=next(e for e in definition['view_buffs']if e['name']==name)
        stem=ROOT/entry['anim'].replace('asset/','mods/',1)
        rr=json.loads(Path(str(stem)+'#anim.fanim').read_text())['anims'][entry['tag']]['frames'][0]['data']
        badge=Image.open(str(stem)+'#sheet.png').convert('RGBA').crop((rr['x'],rr['y'],rr['x']+rr['w'],rr['y']+rr['h']))
        badge=badge.crop(badge.getbbox());preview.alpha_composite(badge,(cx+art.width//3,10))
        draw.text((i*cell+8,size[1]-20),label,fill=(210,219,240,255))
    preview.save(preview_path)
    # Refresh the legacy inspector's composed preview cells, preserving its
    # historical lookup layout and label strips. The cells show the real fallback.
    oldmeta=json.loads(original('editor/vesper-art-preview.json'))
    sheet=Image.open(io.BytesIO(original('editor/vesper-mastery-preview.png'))).convert('RGBA')
    for name,rects in oldmeta['outfits'].items():
        rank,top=map(int,name.split('_'))
        badge_name=f'vs_rank{rank}'if rank<7 else f'vs_top{top}'
        entry=next(e for e in definition['view_buffs']if e['name']==badge_name)
        stem=ROOT/entry['anim'].replace('asset/','mods/',1)
        badge_im=Image.open(str(stem)+'#sheet.png').convert('RGBA')
        badge_frames=json.loads(Path(str(stem)+'#anim.fanim').read_text())['anims'][entry['tag']]['frames']
        native_frames=json.loads((ROOT/'mods/tfm2_custom/champions/tfm2_custom_vesper#anim.fanim').read_text())['anims']['idle']['frames']
        for i,rr in enumerate(rects):
            cell=Image.new('RGBA',(rr['w'],rr['h']))
            b=native_frames[i%len(native_frames)]['data'];actual=body.crop((b['x'],b['y'],b['x']+b['w'],b['y']+b['h']))
            cell.alpha_composite(actual,(round(rr['w']/2-actual.width/2),round(rr['h']/2-actual.height/2)))
            b=badge_frames[i%len(badge_frames)]['data'];badge=badge_im.crop((b['x'],b['y'],b['x']+b['w'],b['y']+b['h']))
            cell.alpha_composite(badge,(round(rr['w']/2-badge.width/2),round(rr['h']/2-badge.height/2)))
            sheet.paste(cell,(rr['x'],rr['y']))
    sheet.save(ROOT/'editor/vesper-mastery-preview.png')
    print('Visible base bodies integrated; legacy clothing aliases safe; rank animations remain gated')


if __name__=='__main__':main()
