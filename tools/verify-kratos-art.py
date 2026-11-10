"""Verify frozen art, gameplay invariants, native/WASM traces and atlas bounds."""
from pathlib import Path
import hashlib
import json
import subprocess
from PIL import Image,ImageOps

ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/'docs/kratos-art-review'
BASE='740a8e930aed88c93647b1a46a14012979cedc61'


def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()


def unpack(sheet,entry):
    d=entry['data'];return sheet.crop((d['x'],d['y'],d['x']+d['w'],d['y']+d['h']))


def main():
    manifest=json.loads((OUT/'art-manifest.json').read_text())
    assert manifest['body_approved'] and manifest['effects_approved'] and not manifest['review_only']
    for name,value in manifest['source_sha256'].items():assert sha(OUT/name)==value,name
    total=0;frame_count=0;atlas_info={}
    for kind,stem in manifest['atlases'].items():
        im=Image.open(OUT/(stem+'.png')).convert('RGBA');anims=json.loads((OUT/(stem+'.fanim')).read_text())['anims']
        assert max(im.size)<=2048,(stem,im.size)
        if kind!='assembled':total+=im.width*im.height*4
        for name,animation in anims.items():
            for f in animation['frames']:
                d=f['data'];assert d['x']>=0 and d['y']>=0 and d['x']+d['w']<=im.width and d['y']+d['h']<=im.height
                assert f['duration']>0;frame_count+=1
            if kind=='effects' and name.startswith('fenrir_p'):assert len(animation['frames'])==1 and abs(animation['frames'][0]['duration']-1/60)<1e-10
        if kind=='body':
            for pose,durations in manifest['pose_ticks'].items():
                assert len(anims[pose]['frames'])==len(durations)==8
                assert abs(sum(f['duration'] for f in anims[pose]['frames'])-sum(durations)/60)<1e-9
                for a,b in zip(anims[pose]['frames'],anims[pose+'_l']['frames']):
                    right=unpack(im,a);left=unpack(im,b)
                    assert ImageOps.mirror(right).tobytes()==left.tobytes()
                    box=right.getbbox();assert box and 0<box[0] and 0<box[1] and box[2]<64 and box[3]==53,(pose,box)
        atlas_info[kind]={'dimensions':im.size,'animations':len(anims),'sha256':sha(OUT/(stem+'.png'))}
    assert total==manifest['decoded_bytes'] and total<=32*1024*1024
    for folder,stem,source in [('champions','tfm2_custom_kratos','body-preview'),('vfx','kratos_weapons','weapons-preview'),('vfx','kratos','effects-preview')]:
        for ext,suffix in [('png','#sheet.png'),('fanim','#anim.fanim')]:assert (OUT/(source+'.'+ext)).read_bytes()==(ROOT/'mods/tfm2_custom'/folder/(stem+suffix)).read_bytes()
    champion=json.loads((ROOT/'mods/tfm2_custom/champion/tfm2_custom_kratos.data_champion').read_text())
    previous=json.loads(subprocess.check_output(['rtk','proxy','git','show',BASE+':mods/tfm2_custom/champion/tfm2_custom_kratos.data_champion'],cwd=ROOT))
    strip=lambda c:{k:v for k,v in c.items() if k not in ('view_effects','view_buffs','passive')}
    assert strip(champion)==strip(previous)
    assert champion['passive']['passive_ref']==previous['passive']['passive_ref'] and champion['passive']['params']=={'weapon_layer':1}
    unchanged=['rules/kratos.json','native/tfm2_custom_ai/src/kratos_core.rs','native/tfm2_custom_ai/src/kratos_data.rs','editor/kratos-data.js','editor/kratos.wasm','native/kratos_lab/src/lib.rs']
    for p in unchanged:
        current=(ROOT/p).read_bytes();old=subprocess.check_output(['rtk','proxy','git','show',BASE+':'+p],cwd=ROOT)
        assert current==old if p.endswith('.wasm') else current.replace(b'\r\n',b'\n')==old.replace(b'\r\n',b'\n'),p
    mappings=json.loads((OUT/'effects-preview.fanim').read_text())['anims']
    for effect in champion['view_effects']:assert effect['tag'] in mappings,effect['tag']
    weapons=json.loads((OUT/'weapons-preview.fanim').read_text())['anims']
    for buff in champion['view_buffs']:assert buff['tag'] in (weapons if buff['name'].startswith(manifest['weapon_prefix']) else mappings),buff['tag']
    source=(ROOT/'native/tfm2_custom_ai/src/kratos_presentation.rs').read_text().split('#[cfg(test)]')[0]
    for banned in ['apply_cc(', 'entity_banish(', 'entity_set_invisible(', 'set_input(', 'deal_damage(', 'entity_set_pos(']:assert banned not in source,banned
    trace=json.loads((ROOT/'backups/kratos-art-tests/parity.json').read_text())
    result=subprocess.run([str(ROOT/'native/kratos_lab/target/release/examples/trace.exe')],input='\n'.join(json.dumps(f['input']) for f in trace)+'\n',capture_output=True,text=True,cwd=ROOT,check=True)
    actual=[json.loads(line) for line in result.stdout.splitlines()]
    assert len(actual)==len(trace)
    for i,(got,expected) in enumerate(zip(actual,trace)):assert got==expected['output'],('native/WASM mismatch',i)
    report={'decoded_bytes':total,'atlas_info':atlas_info,'animation_frame_entries_checked':frame_count,'native_wasm_complete_outputs_equal':len(trace),
            'unchanged_from_pr_head':unchanged,'body_and_effect_approvals':'verified','gameplay_data_unchanged':True,
            'live_game_acceptance':'pending','release_packaging':'pending; existing packaged DLL not rebuilt or installed'}
    (OUT/'verification.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps(report,indent=2))


if __name__=='__main__':main()
