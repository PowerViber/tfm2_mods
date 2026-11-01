"""Read-only validation of the current proposal; preview validation is not art approval."""
from pathlib import Path
from PIL import Image, ImageOps
import json, hashlib, subprocess
ROOT=Path(__file__).resolve().parents[1]
PREVIEW=ROOT/'docs/rally-art-review/packed-preview'
REPORT=ROOT/'docs/rally-results/art-validation.json'
subprocess.run(['rtk','proxy','python',str(ROOT/'tools/generate-rally.py'),'--check'],check=True,cwd=ROOT)
rules=json.loads((ROOT/'rules/rally.json').read_text(encoding='utf8'))['constants']
assert rules['HEIGHT_WORLD_PER_M']==rules['WORLD_UNITS_PER_PIXEL']*rules['HEIGHT_PIXELS_PER_M']
families={};decoded=0;rows=[]
for folder,stem in [('champions','tfm2_custom_rally'),('vfx','rally'),('vfx','rally-strokes'),('vfx','rally-white')]:
    sheet=PREVIEW/folder/(stem+'#sheet.png');meta=PREVIEW/folder/(stem+'#anim.fanim')
    im=Image.open(sheet).convert('RGBA');anims=json.loads(meta.read_text(encoding='utf8'))['anims']
    assert max(im.size)<=2048,(sheet,im.size)
    decoded+=im.width*im.height*4
    for tag,anim in anims.items():
        assert anim['frames'],tag
        for f in anim['frames']:
            d=f['data'];assert d['x']>=0 and d['y']>=0 and d['w']>0 and d['h']>0
            assert d['x']+d['w']<=im.width and d['y']+d['h']<=im.height,(tag,d)
            assert f['duration']>0
    families[stem]=(im,anims)
    rows.append({'name':sheet.name,'size':im.size,'sha256':hashlib.sha256(sheet.read_bytes()).hexdigest(),'animations':len(anims)})
assert decoded<=32*1024*1024,decoded
for name,key in [('warning','SPIKE_RADIUS'),('warning_black','BLACK_RADIUS'),('warning_serve','SERVE_RADIUS')]:
    d=families['rally'][1][name]['frames'][0]['data'];assert d['w']>=2*round(rules[key]/rules['WORLD_UNITS_PER_PIXEL'])+4
def frame(folder,tag):
    key='tfm2_custom_rally'if folder=='champions'else folder
    im,anims=families[key];f=anims[tag]['frames'][0]['data'];return im.crop((f['x'],f['y'],f['x']+f['w'],f['y']+f['h']))
body,anims=families['tfm2_custom_rally']
assert frame('champions','idle').size==(64,64)
assert frame('champions','idle').getbbox(), 'Roster idle must remain visible'
assert not frame('champions','air_hidden').getbbox()
for colour in ['yellow','thunder','white']:
    for form in ['masked','unmasked']:
        for pose in ['idle','run','attack','skill1','skill2','ult','hit','dead','spike','set']:
            right=frame('champions',f'{colour}_{form}_{pose}_r');left=frame('champions',f'{colour}_{form}_{pose}_l')
            assert ImageOps.mirror(right).tobytes()==left.tobytes(),(colour,form,pose)
            assert len(anims[f'{colour}_{form}_{pose}_r']['frames'])==8
        for facing in ['r','l']:
            for phase in range(8):
                f=anims[f'air_{colour}_{form}_{facing}_p{phase}']['frames']
                assert len(f)==1 and abs(f[0]['duration']-2/60)<1e-9
champ=json.loads((ROOT/'editor/rally-champion.json').read_text(encoding='utf8'))
for view in champ['view_effects']+champ['view_buffs']:
    key=view['anim'].rsplit('/',1)[-1]
    assert view['tag'] in families[key][1],(view['name'],view['tag'])
assert champ['id']=='tfm2_custom_rally' and champ['category']=='Melee'
assert champ['attack']['effect']['effect_ref']=='tfm2_custom_ai:rally_basic'
for slot in ['skill','skill2','ult']:assert not champ[slot]['effect']['effects']
report={'scope':'Unapproved preview packing, bindings, facings, anchors and artwork limits; not native renderer acceptance','atlases':rows,'decodedMiB':decoded/1048576,'rosterIdleVisible':True,'mirroredFacings':True,'allPresentationBindingsResolved':True,'approvalRequired':True}
REPORT.write_text(json.dumps(report,indent=2)+'\n',encoding='utf8');print(json.dumps(report,indent=2))
