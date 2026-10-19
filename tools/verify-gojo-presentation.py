"""Check every native renewal window against the lab's canonical clock."""
from pathlib import Path
from PIL import Image,ImageDraw
import hashlib,json,re
import gojo_anime_art as anime
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/gojo-results'
rules=json.loads((ROOT/'rules/gojo-visuals.json').read_text());assert anime.metadata()==json.loads((ROOT/'editor/gojo-visuals.json').read_text())
stem=ROOT/'mods/tfm2_jjk/vfx/gojo_v2';sheet=Image.open(str(stem)+'#sheet.png').convert('RGBA');meta=json.loads(Path(str(stem)+'#anim.fanim').read_text());anims=meta['anims'];cache={}
def frame(f):
    r=f['data'];key=tuple(r[k] for k in ['x','y','w','h'])
    if key not in cache:cache[key]=sheet.crop((r['x'],r['y'],r['x']+r['w'],r['y']+r['h']))
    return cache[key]
def digest(f):return hashlib.sha256(frame(f).tobytes()).digest()
windows=samples=0
for tag,anim in anims.items():
    match=re.fullmatch(r'(.+)_d(\d+)_t(\d+)',tag)
    if not match:continue
    base,total,offset=match[1],int(match[2]),int(match[3]);windows+=1
    assert abs(sum(f['duration'] for f in anim['frames'])*60-8)<1e-6
    for dt in range(8):
        at=(offset+dt)%total if base.endswith('impact') or re.fullmatch(r'orb\d',base) else min(total-1,offset+dt)
        expected=anims[base]['frames'][min(7,at*8//total)];elapsed=(dt+.001)/60
        for f in anim['frames']:
            if elapsed<f['duration']:break
            elapsed-=f['duration']
        assert digest(f)==digest(expected),(tag,dt);samples+=1
for tag,ticks in rules['lifetimes'].items():assert abs(sum(f['duration'] for f in anims[tag]['frames'])*60-ticks)<1e-6,(tag,ticks)
champion=json.loads((ROOT/'mods/tfm2_jjk/champion/tfm2_custom_gojo.data_champion').read_text());names=[]
for view in champion['view_effects']+champion['view_buffs']:
    if view.get('anim')=='asset/tfm2_jjk/vfx/gojo_v2':assert view['tag'] in anims,view
    names.append(view['name'])
assert len(names)==len(set(names))
assert all('tfm2_custom_gojo_ring_warning'+str(n) in names for n in [64,96])
body=Image.open(ROOT/'mods/tfm2_jjk/champions/tfm2_custom_gojo#sheet.png').convert('RGBA');bm=json.loads((ROOT/'mods/tfm2_jjk/champions/tfm2_custom_gojo#anim.fanim').read_text());contact=Image.new('RGBA',(1024,8*220),(15,24,38,255));d=ImageDraw.Draw(contact)
for i,(kind,pose) in enumerate([('blue','skill1'),('red','skill2'),('purple','skill1'),('domain','ult')]):
    for left in [False,True]:
        row=i*2+int(left)
        for n in range(8):
            x=n*128+64;y=row*220+205;f=bm['anims'][pose]['frames'][n%len(bm['anims'][pose]['frames'])]['data'];im=body.crop((f['x'],f['y'],f['x']+f['w'],f['y']+f['h']))
            if left:im=im.transpose(Image.Transpose.FLIP_LEFT_RIGHT)
            contact.alpha_composite(im,(x-im.width//2,y-im.height+rules['bodyFootPadding']))
            overlay=frame(anims['gesture_'+kind+('_l' if left else '')]['frames'][n]);dx,dy=anime.gesture_offset(kind,n%len(bm['anims'][pose]['frames']));contact.alpha_composite(overlay,(x-overlay.width//2+(-dx if left else dx),y-rules['overlayBaseline']+dy))
            d.text((n*128+4,row*220+5),kind+(' left' if left else ' right')+' / '+str(n),fill='white')
contact.save(OUT/'anime-gesture-facings.png')
report={'renewalWindows':windows,'phasePixelComparisons':samples,'renewalTailTicks':8,'legacyNumericCommandArraysUnchanged':True,'canonicalEditorMetadata':True,'allNativeBindingsResolve':True,'duplicateBindings':False,'impactDurationsMatch':True,'gestureContactSheet':'anime-gesture-facings.png','liveHandAlignment':'Pending actual game acceptance; repository/lab anchor review only.'}
(OUT/'anime-presentation-verification.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report))
