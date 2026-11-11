"""Validate approved supplements, exact legacy pixels and authorized rules."""
from pathlib import Path
from io import BytesIO
import hashlib,json,subprocess,copy
from PIL import Image,ImageOps
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/round-165-art-review';BASE='0d7ec96587ba576dc91d9022be810cedfc3cf4da'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def old(p):return subprocess.check_output(['rtk','proxy','git','show',BASE+':'+p],cwd=ROOT)
def crop(im,f):
    r=f['data'];return im.crop((r['x'],r['y'],r['x']+r['w'],r['y']+r['h']))
def main():
    m=json.loads((OUT/'manifest.json').read_text());approval=json.loads((OUT/'approval.json').read_text());assert m['integrated'] and approval['effects']['approved'];assert approval['effects']['source_sha256']==m['source_sha256']
    for name,h in m['source_sha256'].items():assert sha(OUT/name)==h
    unchanged=[];frames=0;budgets={}
    for name in ['akane','wright','thalassa']:
        for suffix in ['#sheet.png','#anim.fanim']:
            p='mods/tfm2_custom/champions/tfm2_custom_'+name+suffix;current=(ROOT/p).read_bytes();before=old(p);assert json.loads(current)==json.loads(before) if suffix.endswith('.fanim') else current==before,p;unchanged.append(p)
        if name!='thalassa':
            for suffix in ['#sheet.png','#anim.fanim']:
                p='mods/tfm2_custom/vfx/'+name+suffix;current=(ROOT/p).read_bytes();before=old(p);assert json.loads(current)==json.loads(before) if suffix.endswith('.fanim') else current==before,p;unchanged.append(p)
            for p in ['rules/'+name+'.json','native/tfm2_custom_ai/src/'+name+'_core.rs','editor/'+name+'.wasm']:
                current=(ROOT/p).read_bytes();before=old(p);assert current==before if p.endswith('.wasm') else current.replace(b'\r\n',b'\n')==before.replace(b'\r\n',b'\n'),p;unchanged.append(p)
        champ=json.loads((ROOT/f'mods/tfm2_custom/champion/tfm2_custom_{name}.data_champion').read_text());before=json.loads(old(f'mods/tfm2_custom/champion/tfm2_custom_{name}.data_champion'))
        strip=lambda c:{k:v for k,v in c.items() if k not in ['view_effects','view_buffs','view_projectiles','passive']};assert strip(champ)==strip(before),name+' gameplay definition changed'
        assert champ['passive']['passive_ref']==before['passive']['passive_ref']
        assets={champ['sprite']}|{b['anim'] for group in ['view_effects','view_buffs','view_projectiles'] for b in champ.get(group,[]) if 'anim' in b}
        decoded=0
        for asset in assets:
            stem=ROOT/'mods'/asset.removeprefix('asset/');im=Image.open(str(stem)+'#sheet.png');assert max(im.size)<=2048;decoded+=im.width*im.height*4
            meta=json.loads(Path(str(stem)+'#anim.fanim').read_text())['anims']
            for a in meta.values():
                for f in a['frames']:
                    r=f['data'];assert r['x']>=0 and r['y']>=0 and r['x']+r['w']<=im.width and r['y']+r['h']<=im.height and f['duration']>0;frames+=1
            for group in ['view_effects','view_buffs','view_projectiles']:
                for b in champ.get(group,[]):
                    if b.get('anim')==asset:assert b['tag'] in meta,(name,b['name'])
        assert decoded<=32*1024*1024,(name,decoded);assert decoded==m['decoded_bytes_per_owner'][name],(name,decoded,m['decoded_bytes_per_owner'][name]);budgets[name]=decoded
    # Reconstruct every original frame, including transparent RGBA, around the
    # same world centre after symmetric padding reduction.
    path='mods/tfm2_custom/vfx/thalassa';old_image=Image.open(BytesIO(old(path+'#sheet.png'))).convert('RGBA');old_meta=json.loads(old(path+'#anim.fanim'))['anims'];im=Image.open(ROOT/(path+'#sheet.png')).convert('RGBA');meta=json.loads((ROOT/(path+'#anim.fanim')).read_text())['anims'];legacy=0
    assert old_meta.keys()==meta.keys()
    for name,ani in old_meta.items():
        assert len(ani['frames'])==len(meta[name]['frames'])
        for before,after in zip(ani['frames'],meta[name]['frames']):
            a,b=crop(old_image,before),crop(im,after);assert before['duration']==after['duration'];assert (a.width-b.width)%2==0 and (a.height-b.height)%2==0
            recovered=Image.new('RGBA',a.size);recovered.paste(b,((a.width-b.width)//2,(a.height-b.height)//2));assert a.tobytes()==recovered.tobytes(),name;legacy+=1
    expected=json.loads(old('rules/thalassa.json'));expected['revision']='thalassa-1.7';expected['water']['rangePct']=50;expected['water']['cloneRangePct']=20
    rules=json.loads((ROOT/'rules/thalassa.json').read_text());assert rules==expected,'Unauthorized rules change';assert rules['basic']['cooldownTicks']==120
    new_meta=json.loads((ROOT/'mods/tfm2_custom/vfx/round_165#anim.fanim').read_text())['anims']
    for name,a in new_meta.items():
        if name.startswith('tsunami_'):assert abs(sum(f['duration'] for f in a['frames'])*60-18)<1e-8
    for key,tag in m['wright_aliases'].items():assert abs(sum(f['duration'] for f in new_meta[tag]['frames'])*60-int(key.split(':')[1]))<1e-8
    generated=(ROOT/'native/tfm2_custom_ai/src/round_165_art.rs').read_text()
    for banned in ['apply_cc(', 'deal_damage(', 'entity_set_pos(', 'entity_set_invisible(', 'heal(', 'set_input(']:assert banned not in generated
    paths=[ROOT/'mods/tfm2_custom/vfx'/('round_165'+ext) for ext in ['#sheet.png','#anim.fanim']]+[ROOT/'mods/tfm2_custom/vfx'/('thalassa'+ext) for ext in ['#sheet.png','#anim.fanim']]+[ROOT/'editor/round-165-art.js',ROOT/'native/tfm2_custom_ai/src/round_165_art.rs']
    hashes={str(p.relative_to(ROOT)):sha(p) for p in paths};subprocess.run(['rtk','proxy','python',str(ROOT/'tools/build-round-165-art.py'),'--integrate'],cwd=ROOT,check=True,capture_output=True)
    assert hashes=={str(p.relative_to(ROOT)):sha(p) for p in paths},'Non-reproducible artwork packing'
    report={'approved':True,'legacy_thalassa_frames_exact_rgba_and_timing':legacy,'atlas_frame_bounds_checked':frames,'decoded_bytes_per_owner':budgets,'unchanged_from_round_165':unchanged,'authorized_gameplay_delta':'thalassa-1.7: water-only tsunami, owner water range 82500; clone range unchanged; 120-tick cadence preserved','reproducible':True,'asset_sha256':hashes,'live_game_acceptance':'pending'}
    (OUT/'verification.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
if __name__=='__main__':main()
