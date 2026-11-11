"""Frozen sources, packing, presentation bounds and complete native/WASM parity."""
from pathlib import Path
import hashlib,json,subprocess
from PIL import Image,ImageOps
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/akane-art-review'
BASE='7b61b8fae020ea7ce5a4dbd1d9c2e2f59b4f1a83'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def old(p):return subprocess.check_output(['rtk','proxy','git','show',BASE+':'+p],cwd=ROOT)
def pixels(sheet,f):
    r=f['data'];return sheet.crop((r['x'],r['y'],r['x']+r['w'],r['y']+r['h']))
def main():
    m=json.loads((OUT/'art-manifest.json').read_text());rules=json.loads((ROOT/'rules/akane.json').read_text());art=json.loads((ROOT/'rules/akane-art.json').read_text())
    assert m['body_approved'] and m['effects_approved'] and not m['review_only']
    frozen=json.loads((OUT/'frozen-sources.json').read_text())
    assert frozen==m['source_sha256']
    for name,digest in frozen.items():assert sha(OUT/name)==digest,name
    total=entries=0;atlases={}
    for kind,stem in m['atlases'].items():
        im=Image.open(OUT/(stem+'.png')).convert('RGBA');anims=json.loads((OUT/(stem+'.fanim')).read_text())['anims'];total+=im.width*im.height*4;assert max(im.size)<=2048
        for name,a in anims.items():
            for f in a['frames']:
                r=f['data'];assert min(r.values())>=0 and r['x']+r['w']<=im.width and r['y']+r['h']<=im.height;assert f['duration']>0;entries+=1
            if kind=='effects' and name.startswith('orb_p'):assert len(a['frames'])==1
        if kind=='body':
            for pose,ts in art['pose_ticks'].items():
                a=anims[pose];assert len(a['frames'])==8;assert abs(sum(f['duration'] for f in a['frames'])*60-sum(ts))<1e-8
                for r,l in zip(a['frames'],anims[pose+'_l']['frames']):
                    right,left=pixels(im,r),pixels(im,l);assert ImageOps.mirror(right).tobytes()==left.tobytes()
                    box=right.getbbox();assert box and box[0]>0 and box[1]>0 and box[2]<right.width and box[3]==right.height//2+21,(pose,box)
            assert len(set(pixels(im,f).tobytes() for f in anims['run']['frames']))>=6
        atlases[kind]={'size':im.size,'animations':len(anims),'sha256':sha(OUT/(stem+'.png'))}
        target='champions/tfm2_custom_akane' if kind=='body' else 'vfx/akane'
        for ext,suffix in [('.png','#sheet.png'),('.fanim','#anim.fanim')]:assert (OUT/(stem+ext)).read_bytes()==(ROOT/'mods/tfm2_custom'/(target+suffix)).read_bytes()
    assert total==m['decoded_bytes'] and total<=32*1024*1024
    champ=json.loads((ROOT/'mods/tfm2_custom/champion/tfm2_custom_akane.data_champion').read_text());previous=json.loads(old('mods/tfm2_custom/champion/tfm2_custom_akane.data_champion'))
    strip=lambda c:{k:v for k,v in c.items() if k not in ('view_effects','view_buffs','passive')}
    # Rules-driven range changes are verified separately after the art-only checkpoint.
    if rules['revision']=='akane-1':assert strip(champ)==strip(previous)
    assert champ['passive']=={'passive_ref':'tfm2_custom_ai:akane','params':{'art_v1':1}}
    mappings=json.loads((OUT/'effects.fanim').read_text())['anims']
    for binding in champ['view_effects']+champ['view_buffs']:assert binding['tag'] in mappings
    unchanged=['mods/tfm2_custom_ai/tfm2_custom_ai.dll']
    if rules['revision']=='akane-1':unchanged+=['rules/akane.json','native/tfm2_custom_ai/src/akane_core.rs','native/tfm2_custom_ai/src/akane_data.rs','native/akane_lab/src/lib.rs','editor/akane.wasm']
    for name in unchanged:
        current=(ROOT/name).read_bytes();before=old(name);assert current==before if name.endswith(('.dll','.wasm')) else current.replace(b'\r\n',b'\n')==before.replace(b'\r\n',b'\n'),name
    traces=json.loads((ROOT/'backups/akane-art-tests/parity.json').read_text())
    result=subprocess.run([str(ROOT/'native/akane_lab/target/release/examples/trace.exe')],input='\n'.join(json.dumps(t['input']) for t in traces)+'\n',text=True,capture_output=True,check=True)
    actual=[json.loads(s) for s in result.stdout.splitlines()];assert len(actual)==len(traces)
    for i,(got,trace) in enumerate(zip(actual,traces)):assert got==trace['output'],('Native/WASM mismatch',i)
    files=[OUT/(stem+ext) for stem in ('body','effects') for ext in ('.png','.fanim')]+[OUT/'presentation.json',OUT/'body-comparison.png',OUT/'effects-comparison.png']
    before={p.name:sha(p) for p in files}
    subprocess.run(['rtk','proxy','python',str(ROOT/'tools/build-akane-art.py')],cwd=ROOT,check=True,capture_output=True)
    assert before=={p.name:sha(p) for p in files},'Non-reproducible extraction/packing'
    report={'revision':rules['revision'],'decoded_bytes':total,'atlases':atlases,'animation_frame_entries':entries,'complete_native_wasm_outputs_equal':len(traces),'reproducible_packing':True,'unchanged_from_pr_head':unchanged,'artwork_approvals':'verified','release_dll_sha256':sha(ROOT/unchanged[0]),'native_game_acceptance':'pending','release_packaging':'pending'}
    (OUT/'verification.json').write_text(json.dumps(report,indent=2)+'\n')
    if rules['revision']=='akane-1':(OUT/'art-only-verification.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps(report,indent=2))
if __name__=='__main__':main()
