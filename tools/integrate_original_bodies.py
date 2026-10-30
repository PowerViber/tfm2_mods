"""Mechanical integration of approved imagegen sources; never installs into the game.

Run original_bodies.py complete first. All reference frames come from frozen main,
so rebuilding this file never feeds a generated derivative back into itself.
"""
from original_bodies import *
import base64, re

CHANGED = {}

def baseline(stem):
    rel=stem.relative_to(ROOT).as_posix()
    read=lambda ext:subprocess.check_output(['rtk','proxy','git','show',BASE+':'+rel+ext],cwd=ROOT)
    return Image.open(io.BytesIO(read('#sheet.png'))).convert('RGBA'),json.loads(read('#anim.fanim'))

def staged(name):
    folder=OUT/name
    return Image.open(folder/'complete#sheet.png').convert('RGBA'),json.loads((folder/'complete#anim.fanim').read_text())

def approved(name):
    a=json.loads((OUT/'approval.json').read_text())['bodies'][name]
    assert a['approved'] and a['sourceSha256']==sha(OUT/name/'proposal-source.png')

def save(stem,images,meta,changed):
    atlas,packed=atlas_pack(images,meta)
    atlas.save(str(stem)+'#sheet.png')
    Path(str(stem)+'#anim.fanim').write_text(json.dumps(packed,separators=(',',':'))+'\n')
    CHANGED[stem.relative_to(ROOT).as_posix()]=changed
    return atlas,packed

def recolour(after,before):
    """Retain old afterimage tint and maximum opacity, using the new silhouette."""
    data=np.asarray(before);valid=data[:,:,3]>0
    if not valid.any():return Image.new('RGBA',after.size)
    colours,counts=np.unique(data[:,:,:3][valid],axis=0,return_counts=True)
    tint=tuple(int(v) for v in colours[counts.argmax()]);opacity=int(data[:,:,3].max())
    result=Image.new('RGBA',after.size,(*tint,0))
    result.putalpha(after.getchannel('A').point(lambda a:round(a*opacity/255)))
    return result

def replace_from_body(before,body,tag,index,mirror=False,tinted=False):
    im,meta=staged(body);frames=meta['anims'][tag]['frames'];pose=crop_frame(im,frames[index%len(frames)])
    # Already registered world feet in the staged body. Derivatives retain their
    # original foot anchor and the same global size, including carried weapons.
    if mirror:pose=ImageOps.mirror(pose)
    result,_=fit_pose(pose.crop(pose.getbbox()),before,1)
    return recolour(result,before) if tinted else result

def world():
    approved('the_world');stem=ROOT/'mods/tfm2_jojo/vfx/jojo';old,meta=baseline(stem)
    poses=sprite_library(OUT/'the_world/proposal-source.png',8)
    ref=crop_frame(old,meta['anims']['stand_idler0']['frames'][0]);scale=(ref.getbbox()[3]-ref.getbbox()[1])/poses[0].height
    images={};changed=[]
    for tag,anim in meta['anims'].items():
        body=tag.startswith(('stand_idle','stand_punch','stand_bar','stand_worldr','stand_worldl','stand_lunge','stand_grab','stand_ghost')) or tag in ['stand_world_ult','stand_pop']
        for n,f in enumerate(anim['frames']):
            before=crop_frame(old,f);after=before
            if body:
                index=6+n%2 if 'world' in tag else 4+n%2 if any(s in tag for s in ['punch','bar','lunge','grab','ghost','pop']) else n%2
                if tag[-1:].isdigit():index+=int(tag[-1])%2
                index=min(index,7);pose=poses[index]
                left=bool(re.search(r'(?:idle|punch|bar|world|lunge|grab)l',tag)) or tag=='stand_ghostl'
                if left:pose=ImageOps.mirror(pose)
                after,_=fit_pose(pose,before,scale)
                if 'ghost' in tag:after=recolour(after,before)
                else:
                    opacity=before.getchannel('A').getextrema()[1]
                    if opacity<255:after.putalpha(after.getchannel('A').point(lambda a:round(a*opacity/255)))
                # Preserve the separate fist-ring detail in legacy pop/world
                # composites. It lies outside the old body at x >= 39.
                if tag in ['stand_world_ult','stand_pop']:
                    fx=before.copy();fx.paste((0,0,0,0),(0,0,39,fx.height))
                    px=(after.width-before.width)//2;py=(after.height-before.height)//2
                    after.alpha_composite(fx,(px,py))
                changed.append([tag,n])
            images[(tag,n)]=after
    save(stem,images,meta,changed)

def derivatives():
    groups=[('tfm2_frieren','frieren'),('tfm2_cyberpunk','cyberpunk'),('tfm2_custom','minato')]
    for pack,name in groups:
        stem=ROOT/'mods'/pack/'vfx'/name;old,meta=baseline(stem);images={};changed=[]
        for tag,anim in meta['anims'].items():
            for n,f in enumerate(anim['frames']):
                before=crop_frame(old,f);after=before
                if name=='frieren':
                    if tag in ['fern_s1','fern_side']:
                        after=replace_from_body(before,'fern','skill1' if tag=='fern_s1' else 'idle',n)
                    elif tag in ['stark_up','stark_down']:
                        poses=sprite_library(OUT/'stark/motion-source.png',16);r=json.loads((OUT/'stark/complete.json').read_text())
                        after,_=fit_pose(poses[8 if tag=='stark_up' else 9],before,r['scales']['m'])
                    else:
                        m=re.fullmatch(r'stark_([ira])([rl])(\d+)',tag)
                        if m:after=replace_from_body(before,'stark',{'i':'idle','r':'run','a':'attack'}[m[1]],int(m[3]),m[2]=='l')
                elif tag.startswith('after_'):
                    body='david' if name=='cyberpunk' else 'minato'
                    after=replace_from_body(before,body,'run',n,tag.endswith('_l'),True)
                if after is not before:changed.append([tag,n])
                images[(tag,n)]=after
        save(stem,images,meta,changed)
    stem=ROOT/'mods/tfm2_frieren/champions/stark_ghoul';old,meta=baseline(stem);images={};changed=[]
    for tag,anim in meta['anims'].items():
        for n,f in enumerate(anim['frames']):
            original=crop_frame(old,f);pose=tag.removeprefix('berserk_')
            assert pose in BODY_TAGS['stark'],pose
            images[(tag,n)]=replace_from_body(original,'stark',pose,n)
            changed.append([tag,n])
    save(stem,images,meta,changed)

def minato_layers():
    # Static frame slots let the native presenter change form/facing mid-pose
    # without resetting the pose clock. The original visible alias is retained.
    images={};meta={'anims':{}};clock={}
    for form,name in [('normal','minato'),('kcm','minato_kcm')]:
        sheet,m=staged(name)
        for tag in BODY_TAGS[name]:
            frames=m['anims'][tag]['frames'];clock[tag]=[max(1,round(f['duration']*60000)) for f in frames]
            for side in ['r','l']:
                for n,f in enumerate(frames):
                    key=f'{form}_{tag}_{side}{n}';im=crop_frame(sheet,f)
                    if side=='l':im=ImageOps.mirror(im)
                    images[(key,0)]=im;meta['anims'][key]={'frames':[{'duration':f['duration'],'data':{}}]}
    stem=ROOT/'mods/tfm2_custom/champions/minato_forms';save(stem,images,meta,list(meta['anims']))
    for form,name in [('normal','minato'),('kcm','minato_kcm')]:
        sheet,m=staged(name)
        for side in ['r','l']:
            key=f'{form}_dead_{side}';meta['anims'][key]=copy.deepcopy(m['anims']['dead'])
            for n,f in enumerate(m['anims']['dead']['frames']):
                im=crop_frame(sheet,f);images[(key,n)]=ImageOps.mirror(im) if side=='l' else im
    save(stem,images,meta,list(meta['anims']))
    body=json.loads((ROOT/'mods/tfm2_custom/champion/tfm2_custom_minato.data_champion').read_text())
    body['sprite']='asset/tfm2_custom/champions/minato_presenter_base'
    body['passive']={'passive_ref':'tfm2_custom_ai:minato_body','params':{}}
    body['view_buffs']=[b for b in body['view_buffs'] if not b['name'].startswith('mn_body_')]
    for key in meta['anims']:
        if re.fullmatch(r'(normal|kcm)_dead_[rl]',key):continue
        body['view_buffs'].append({'type':'Animated','name':'mn_body_'+key,'anim':'asset/tfm2_custom/champions/minato_forms','tag':key,'z':0})
    body['view_effects']=[v for v in body['view_effects'] if not v['name'].startswith('mn_death_')]
    for form in ['normal','kcm']:
        for side in ['r','l']:body['view_effects'].append({'type':'Animation','name':f'mn_death_{form}_{side}','anim':'asset/tfm2_custom/champions/minato_forms','tag':f'{form}_dead_{side}','z':0,'is_follow':False})
    (ROOT/'mods/tfm2_custom/champion/tfm2_custom_minato.data_champion').write_text(json.dumps(body,indent=2)+'\n')
    # Roster and draft screens render this base without running passives. Keep
    # every normal pose visible; the active match presenter alone selects the
    # separate empty animation, and otherwise this is a fail-visible fallback.
    sheet,base=staged('minato')
    visible={(tag,n):crop_frame(sheet,f) for tag,a in base['anims'].items() for n,f in enumerate(a['frames'])}
    hidden_tag='mn_body_hidden'
    base['anims'][hidden_tag]={'frames':[{'duration':.1,'data':{'x':0,'y':0,'w':1,'h':1}}]}
    visible[(hidden_tag,0)]=Image.new('RGBA',(1,1))
    save(ROOT/'mods/tfm2_custom/champions/minato_presenter_base',visible,base,list(base['anims']))
    actions={k:{'tag':body[k]['action_name'],'ticks':body[k]['duration']} for k in ['attack','skill','skill2','ult']}
    rules={'version':2,'scope':'presentation only; visible roster/fallback body, one-tick match-only empty-animation override','hiddenTag':hidden_tag,'framesMilliTicks':clock,'actions':actions}
    (ROOT/'rules/minato-body-presentation.json').write_text(json.dumps(rules,indent=2)+'\n')
    rust='// Generated by tools/integrate_original_bodies.py. Presentation only.\n'
    rust+='pub const HIDDEN_TAG: &str = "'+hidden_tag+'";\n'
    rust+='pub const ACTIONS: [(&str, usize); 4] = ['+','.join(f'("{v["tag"]}",{v["ticks"]})' for v in actions.values())+'];\n'
    rust+='pub fn frames(tag: &str) -> &\'static [usize] { match tag {\n'
    for tag,frames in clock.items():rust+='"'+tag+'" => &'+str(frames)+',\n'
    rust+='_ => &[100], } }\n'
    (ROOT/'native/tfm2_custom_ai/src/minato_body_data.rs').write_text(rust)
    # The simplified Skill Test interpreter does not run native passives. It
    # uses visible form atlases while preserving its existing action clock.
    sheet,m=staged('minato_kcm')
    sheet.save(ROOT/'mods/tfm2_custom/champions/minato_kcm#sheet.png')
    (ROOT/'mods/tfm2_custom/champions/minato_kcm#anim.fanim').write_text(json.dumps(m,separators=(',',':'))+'\n')

def starters():
    p=ROOT/'editor/sprites-data.js';raw=p.read_text();data=json.loads(raw[raw.index('{'):].rstrip().removesuffix(';'))
    for name in ['gojo','frieren','fern','stark','dio','david','v1','minato','minato_kcm']:
        folder=OUT/name
        entry=data.setdefault(name,{})
        entry['label']=name.replace('_',' ').title()+' (approved original body)'
        entry['png']=base64.b64encode((folder/'complete#sheet.png').read_bytes()).decode()
        # Existing starters use `fanim`; retain the schema exactly.
        entry.pop('anim',None)
        entry['fanim']=json.loads((folder/'complete#anim.fanim').read_text())
    stem=ROOT/'mods/tfm2_frieren/champions/stark_ghoul';entry=data['stark_ghoul'];entry['label']='Stark carrier (approved original body)';entry['png']=base64.b64encode(Path(str(stem)+'#sheet.png').read_bytes()).decode();entry['fanim']=json.loads(Path(str(stem)+'#anim.fanim').read_text())
    p.write_text('/* Frozen approved original bodies; packed by tools/integrate_original_bodies.py. */\nwindow.TFM2_SPRITES = '+json.dumps(data,separators=(',',':'))+';\n')
    for file,names in [('vfx.js',{'minato':'tfm2_custom'}),('vfx2.js',{'frieren':'tfm2_frieren','jojo':'tfm2_jojo','cyberpunk':'tfm2_cyberpunk'})]:
        path=ROOT/'editor'/file;s=path.read_text();start=s.index('{',s.index('window.TFM2_VFX'))
        if file=='vfx2.js':start=s.index('{',start+2)
        d,end=json.JSONDecoder().raw_decode(s[start:])
        # Object.assign's first object is the previous catalogue, second holds
        # this file's entries; the first opening brace belongs to that object.
        for name,pack in names.items():
            stem=ROOT/'mods'/pack/'vfx'/name
            if name in d:d[name].update(png=base64.b64encode(Path(str(stem)+'#sheet.png').read_bytes()).decode(),fanim=json.loads(Path(str(stem)+'#anim.fanim').read_text()))
        path.write_text(s[:start]+json.dumps(d,separators=(',',':'))+s[start+end:])

def main():
    for name in BODY_TAGS:
        approved(name);pack,body,_=SPECS[name]
        if name=='minato_kcm':continue
        stem=ROOT/'mods'/pack/'champions'/body
        for ext in ['#sheet.png','#anim.fanim']:shutil.copyfile(OUT/name/('complete'+ext),str(stem)+ext)
        CHANGED[stem.relative_to(ROOT).as_posix()]=[[r['tag'],r['frame']] for r in json.loads((OUT/name/'complete.json').read_text())['frames']]
    world();derivatives();minato_layers();starters()
    (OUT/'integration.json').write_text(json.dumps({'base':BASE,'changedFrames':CHANGED},indent=2)+'\n')
    gallery(list(SPECS))
    print(json.dumps({'updatedAtlases':len(CHANGED),'vader':'unchanged','installation':'not performed'}))

if __name__=='__main__':main()
