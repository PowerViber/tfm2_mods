"""Verify body art, frozen effects, anchors, data and presentation-only native scope."""
from integrate_original_bodies import *

report={'baseline':BASE,'bodyFrames':0,'preservedFrames':0,'atlases':[],'dataDefinitions':0,'liveAcceptance':'pending; no installation or real-career migration'}
manifest=json.loads((OUT/'integration.json').read_text())['changedFrames']
for rel,changed in manifest.items():
    stem=ROOT/rel;im=Image.open(str(stem)+'#sheet.png').convert('RGBA');meta=json.loads(Path(str(stem)+'#anim.fanim').read_text())
    assert max(im.size)<=2048,(rel,im.size)
    report['atlases'].append({'asset':rel,'size':list(im.size),'decodedBytes':im.width*im.height*4})
    if rel.endswith(('minato_forms','minato_presenter_base')):continue
    try:old,original=baseline(stem)
    except subprocess.CalledProcessError:continue # newly added presentation atlases
    changed={tuple(v) for v in changed}
    assert set(meta['anims'])==set(original['anims']),rel
    for tag,a in original['anims'].items():
        assert len(a['frames'])==len(meta['anims'][tag]['frames']),(rel,tag)
        for n,f in enumerate(a['frames']):
            new=meta['anims'][tag]['frames'][n];before=crop_frame(old,f);after=crop_frame(im,new)
            assert f['duration']==new['duration'],(rel,tag,n,'duration')
            if (tag,n) in changed:
                assert (after.width-before.width)%2==0 and (after.height-before.height)%2==0,(rel,tag,n,'padding')
                if before.getbbox() and after.getbbox():
                    assert 2*after.getbbox()[3]-after.height==2*before.getbbox()[3]-before.height,(rel,tag,n,'foot')
                report['bodyFrames']+=1
            else:
                assert before.size==after.size and before.tobytes()==after.tobytes(),(rel,tag,n,'effect changed')
                report['preservedFrames']+=1

for name in BODY_TAGS:approved(name)
for p in (ROOT/'mods').rglob('*.data_champion'):
    rel=p.relative_to(ROOT).as_posix()
    before=json.loads(subprocess.check_output(['rtk','proxy','git','show',BASE+':'+rel],cwd=ROOT))
    after=json.loads(p.read_text())
    if after['id']=='tfm2_custom_minato':
        for k in ['sprite','passive','view_buffs','view_effects']:before.pop(k,None);after.pop(k,None)
    assert before==after,rel
    report['dataDefinitions']+=1

# All gameplay source and rules must remain byte identical. The only runtime
# source edit outside the new presenter is its module and registration.
changed=subprocess.check_output(['rtk','proxy','git','diff','--name-only',BASE],cwd=ROOT,text=True).splitlines()
for rel in changed:
    if rel.startswith('native/tfm2_custom_ai/src/'):
        assert rel.rsplit('/',1)[1] in ['lib.rs','minato_body.rs','minato_body_data.rs'],rel
lib=ROOT/'native/tfm2_custom_ai/src/lib.rs';old=subprocess.check_output(['rtk','proxy','git','show',BASE+':native/tfm2_custom_ai/src/lib.rs'],cwd=ROOT).decode()
now=lib.read_text(encoding='utf8');now=now.replace('mod minato_body;\n','').replace('mod minato_body_data;\n','').replace('    decl.add_native_passive(format!("{MOD_ID}:minato_body"), minato_body::MinatoBody::default());\n','')
assert now==old.replace('\r\n','\n'),'Existing native gameplay changed'
native=(ROOT/'native/tfm2_custom_ai/src/minato_body.rs').read_text()
calls=set(re.findall(r'sim\.([a-z_]+)\(',native))
assert calls<={'tick','get_entity','get_player','player_count','player_at','is_visible','add_buff','entity_remove_buff'},calls
assert 'rng_seed' not in native
report['presenterHostCalls']=sorted(calls)

# Old replay/career sprite aliases and every non-body pack remain present.
base=json.loads((OUT/'baseline.json').read_text())
for rel,digest in base['files'].items():
    p=ROOT/rel;assert p.exists(),rel
    if rel.startswith('mods/tfm2_starwars/'):assert sha(p)==digest,rel

# Budget is per champion and owned presentation, not the unrelated whole pack.
budgets={
 'gojo':['tfm2_jjk'], 'frieren_party':['tfm2_frieren'],'dio':['tfm2_jojo'],
 'david':['tfm2_cyberpunk'],'v1':['tfm2_ultrakill']}
report['decodedBytes']={}
for name,packs in budgets.items():
    total=sum(Image.open(p).width*Image.open(p).height*4 for pack in packs for p in (ROOT/'mods'/pack).rglob('*#sheet.png'))
    assert total<=32*1024*1024,(name,total);report['decodedBytes'][name]=total
minato=[p for p in (ROOT/'mods/tfm2_custom/champions').glob('*minato*#sheet.png')]+list((ROOT/'mods/tfm2_custom/vfx').glob('minato*#sheet.png'))
report['decodedBytes']['minato']=sum(Image.open(p).width*Image.open(p).height*4 for p in minato)
assert report['decodedBytes']['minato']<=32*1024*1024

# Explicit mirrored frame slots are exact mirrors, including asymmetric clothing.
p=ROOT/'mods/tfm2_custom/champions/minato_forms';im=Image.open(str(p)+'#sheet.png').convert('RGBA');meta=json.loads(Path(str(p)+'#anim.fanim').read_text())
for tag,a in meta['anims'].items():
    m=re.fullmatch(r'(.+)_r(\d+)',tag)
    if m:
        left=meta['anims'][m[1]+'_l'+m[2]]
        assert ImageOps.mirror(crop_frame(im,a['frames'][0])).tobytes()==crop_frame(im,left['frames'][0]).tobytes()

# Embedded editor art must be current, not the retired recolours.
s=(ROOT/'editor/sprites-data.js').read_text();data=json.JSONDecoder().raw_decode(s[s.index('{'):])[0]
for name in BODY_TAGS:
    entry=data[name]
    assert base64.b64decode(entry['png'])==(OUT/name/'complete#sheet.png').read_bytes(),name
    assert entry['fanim']==json.loads((OUT/name/'complete#anim.fanim').read_text()),name
    if name!='minato_kcm':
        pack,body,_=SPECS[name]
        for ext in ['#sheet.png','#anim.fanim']:
            assert (ROOT/'mods'/pack/'champions'/(body+ext)).read_bytes()==(OUT/name/('complete'+ext)).read_bytes(),(name,ext)
report['unchangedLabWasm']={p.name:sha(p) for p in (ROOT/'editor').glob('*.wasm')}
for filename,digest in report['unchangedLabWasm'].items():
    old=subprocess.check_output(['rtk','proxy','git','show',BASE+':editor/'+filename],cwd=ROOT)
    assert hashlib.sha256(old).hexdigest()==digest,filename

(OUT/'verification.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report,indent=2))
