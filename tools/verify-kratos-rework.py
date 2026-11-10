"""Approved kratos-2 effects, frozen body pixels and complete native/WASM parity."""
from pathlib import Path
import hashlib,json,subprocess
from PIL import Image
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/kratos-art-review';REPORT=ROOT/'docs/kratos-skill-review'
def digest(b):return hashlib.sha256(b).hexdigest()
def main():
    m=json.loads((OUT/'presentation.json').read_text());r=json.loads((ROOT/'rules/kratos.json').read_text())
    assert m['body_approved'] and m['effects_approved'] and not m['review_only']
    for n,v in {**m['source_sha256'],**m['candidate_source_sha256']}.items():assert digest((OUT/n).read_bytes())==v,n
    total=0;frames=0;atlases={}
    for group,stem in [('body','tfm2_custom_kratos'),('weapons','kratos_weapons'),('effects','kratos')]:
        image=Image.open(OUT/(group+'-preview.png'));assert max(image.size)<=2048
        total+=image.width*image.height*4;meta=json.loads((OUT/(group+'-preview.fanim')).read_text())
        for name,a in meta['anims'].items():
            for f in a['frames']:
                d=f['data'];assert d['x']>=0 and d['y']>=0 and d['x']+d['w']<=image.width and d['y']+d['h']<=image.height and f['duration']>0;frames+=1
            if name.startswith('fenrir_p'):assert len(a['frames'])==1
        folder='champions' if group=='body' else 'vfx'
        for ext,suffix in [('png','#sheet.png'),('fanim','#anim.fanim')]:assert (OUT/(group+'-preview.'+ext)).read_bytes()==(ROOT/'mods/tfm2_custom'/folder/(stem+suffix)).read_bytes()
        atlases[group]={'size':image.size,'sha256':digest((OUT/(group+'-preview.png')).read_bytes())}
    assert total==m['decoded_bytes'] and total<=32*1024*1024
    for name in ['cleave','sweep','smash']:
        assert sum(m['pose_ticks'][name])==r['rampage']['stageTicks']
        assert sum(m['pose_ticks'][name][:5])==r['rampage']['stageTicks']//2
    for f in ['mods/tfm2_custom/champions/tfm2_custom_kratos#sheet.png','mods/tfm2_custom/vfx/kratos_weapons#sheet.png']:
        before=subprocess.check_output(['rtk','proxy','git','show','ef6b275:'+f],cwd=ROOT);assert (ROOT/f).read_bytes()==before,('approved body pixels changed',f)
    traces=[]
    for p in ['backups/kratos-art-tests/parity.json','backups/kratos-skill-tests/parity.json']:
        trace=json.loads((ROOT/p).read_text());text='\n'.join(json.dumps(t['input']) for t in trace)+'\n'
        output=subprocess.run([str(ROOT/'native/kratos_lab/target/release/examples/trace.exe')],input=text,text=True,capture_output=True,cwd=ROOT,check=True)
        actual=[json.loads(s) for s in output.stdout.splitlines()];assert len(actual)==len(trace)
        for i,(a,b) in enumerate(zip(actual,trace)):assert a==b['output'],(p,i,'native/WASM mismatch')
        traces.append({'fixture':p,'complete_outputs_equal':len(trace)})
    report={'rules':r['revision'],'decoded_bytes':total,'frame_entries_checked':frames,'atlases':atlases,'body_and_weapon_pixels_unchanged':True,'approved_effects_integrated':True,'native_wasm':traces,'packaged_dll':'unchanged baseline; separate release build pending','actual_game_acceptance':'pending'}
    (REPORT/'verification.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
if __name__=='__main__':main()
