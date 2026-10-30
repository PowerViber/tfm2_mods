"""Regress the actual sprite used by the game roster, without editor aliases."""
from pathlib import Path
import sys,json,subprocess,hashlib,re,io
from PIL import Image,ImageDraw,ImageOps
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/minato-portrait-fix';OUT.mkdir(exist_ok=True)
BASE='777204f'
def blob(rel):return subprocess.check_output(['rtk','proxy','git','show',BASE+':'+rel],cwd=ROOT)
def frame(sheet,f):
 r=f['data'];return sheet.crop((r['x'],r['y'],r['x']+r['w'],r['y']+r['h']))
def asset(stem):return Image.open(str(stem)+'#sheet.png').convert('RGBA'),json.loads(Path(str(stem)+'#anim.fanim').read_text())
rel='mods/tfm2_custom/champion/tfm2_custom_minato.data_champion'
champ=json.loads((ROOT/rel).read_text())
def same_except_kcm_layer_z(d):
    # 0.10.81: the golden KCM layers draw above the visible base body (z 1); nothing else may change.
    d=json.loads(json.dumps(d))
    for b in d['view_buffs']:
        if b['name'].startswith('mn_body_kcm_'): b['z']=0
    return d
assert same_except_kcm_layer_z(champ)==json.loads(blob(rel)),'Combat definition changed'
stem=ROOT/'mods'/champ['sprite'].removeprefix('asset/')
sheet,meta=asset(stem);normal,nm=asset(ROOT/'mods/tfm2_custom/champions/tfm2_custom_minato')
rules=json.loads((ROOT/'rules/minato-body-presentation.json').read_text());hidden=rules['hiddenTag']
assert max(sheet.size)<=2048
visible=0
for tag,a in nm['anims'].items():
 assert len(a['frames'])==len(meta['anims'][tag]['frames']),tag
 for n,f in enumerate(a['frames']):
  expected=frame(normal,f);new=meta['anims'][tag]['frames'][n];actual=frame(sheet,new)
  assert f['duration']==new['duration'] and actual.size==expected.size and actual.tobytes()==expected.tobytes(),(tag,n)
  visible+=actual.getbbox() is not None
assert set(meta['anims'])==set(nm['anims'])|{hidden}
assert frame(sheet,meta['anims'][hidden]['frames'][0]).getbbox() is None
idle=frame(sheet,meta['anims']['idle']['frames'][0]);assert idle.getchannel('A').getextrema()[1]>0,'Roster is blank'
before_sheet=Image.open(io.BytesIO(blob(str(stem.relative_to(ROOT)).replace('\\','/')+'#sheet.png'))).convert('RGBA')
before_meta=json.loads(blob(str(stem.relative_to(ROOT)).replace('\\','/')+'#anim.fanim'))
before=frame(before_sheet,before_meta['anims']['idle']['frames'][0]);assert before.getbbox() is None,'Regression fixture no longer demonstrates blank portrait'

preview=Image.new('RGB',(600,250),'#203532');draw=ImageDraw.Draw(preview)
for col,(im,label)in enumerate([(before,'Before: actual base idle was empty'),(idle,'After: actual base idle is visible')]):
 draw.text((col*300+12,12),label,fill='white')
 preview.paste(im,(col*300+30-im.width//2,135-im.height//2),im)
 large=im.resize((im.width*3,im.height*3),Image.Resampling.NEAREST);preview.paste(large,(col*300+170-large.width//2,135-large.height//2),large)
preview.save(OUT/'portrait-before-after.png')

source=(ROOT/'native/tfm2_custom_ai/src/minato_body.rs').read_text(encoding='utf8')
calls=set(re.findall(r'sim\.([a-z_]+)\(',source))
assert calls<={'tick','get_entity','get_player','player_count','player_at','is_visible','add_buff','entity_remove_buff'},calls
assert 'apply_cc(' not in source[:source.index('#[cfg(test)]')]  # never pins him with a per-tick CC
assert 'fx_point' not in source,'Visible fallback already owns the corpse'
assert 'entity_clear_cc' not in source and 'entity_set_invisible' not in source
assert 'minato_presenter_base\'] = out' not in (ROOT/'editor/server.js').read_text()
bytes_total=sum(Image.open(p).width*Image.open(p).height*4 for p in [*(ROOT/'mods/tfm2_custom/champions').glob('*minato*#sheet.png'),*(ROOT/'mods/tfm2_custom/vfx').glob('minato*#sheet.png')])
assert bytes_total<=32*1024*1024
report={'baseline':BASE,'sprite':champ['sprite'],'beforeIdleTransparent':True,'afterIdleVisible':True,'visibleFrames':visible,'fallbackEqualsApprovedNormalBody':True,'hiddenTag':hidden,'hiddenFrameTransparent':True,'combatDefinitionUnchanged':True,'presenterWritesOnlyVisualBuffs':True,'noUnrelatedCcCleared':True,'noDuplicateCorpseEffect':True,'editorUsesActualAsset':True,'decodedBytes':bytes_total,'atlasSize':list(sheet.size),'liveAcceptance':'Pending game roster and match inspection after installing this build; no installation performed.'}
(OUT/'verification.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
