"""Approved body validation. Effects deliberately remain outside the game pack."""
from pathlib import Path
import hashlib,json,subprocess
from PIL import Image,ImageOps
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/basalt-art-review'
BASE='3790d7ddb9578f3fc051e67bfa5f8f0bbb1f200a'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def old(p):return subprocess.check_output(['rtk','proxy','git','show',BASE+':'+p],cwd=ROOT)
def main():
    files=['body.png','body.fanim','body-presentation.json','body-frame-metrics.json','body-proposals.png','effect-proposals.png','manifest.json']
    before={n:sha(OUT/n) for n in files}
    for script in ['prepare-basalt-art.py','build-basalt-body.py']:
        subprocess.run(['rtk','proxy','python',str(ROOT/'tools'/script)],cwd=ROOT,check=True,capture_output=True)
    assert before=={n:sha(OUT/n) for n in files},'Non-reproducible packing'
    meta=json.loads((OUT/'body-presentation.json').read_text());sheet=Image.open(OUT/'body.png').convert('RGBA')
    assert meta['body_approved'] and not meta['effects_approved']
    assert max(sheet.size)<=2048 and sheet.width*sheet.height*4<=32*1024*1024
    for n,h in meta['source_hashes'].items():assert sha(OUT/n)==h,n
    anims=meta['body_anims'];metrics={};entries=0
    def crop(f):
        r=f['data'];assert 0<=r['x']<r['x']+r['w']<=sheet.width and 0<=r['y']<r['y']+r['h']<=sheet.height
        assert f['duration']>0
        return sheet.crop((r['x'],r['y'],r['x']+r['w'],r['y']+r['h']))
    for tag,seq in anims.items():
        for f in seq['frames']:crop(f);entries+=1
    for tag,count in meta['pose_frames'].items():
        right=anims[tag]['frames'];left=anims[tag+'_l']['frames'];assert len(right)==len(left)==count
        bounds=[]
        for f,l in zip(right,left):
            im=crop(f);box=im.getbbox();assert box and box[0]>0 and box[1]>0 and box[2]<160 and box[3]<80,(tag,box)
            mirrored=Image.new('RGBA',im.size);mirrored.alpha_composite(ImageOps.mirror(im),(1,0));assert mirrored.tobytes()==crop(l).tobytes(),tag
            bounds.append(box)
        metrics[tag]=bounds
    run=[crop(f) for f in anims['run']['frames']]
    assert len({f.tobytes() for f in run})==6
    assert len({f.crop((0,46,160,62)).getchannel('A').tobytes() for f in run})==6,'Walk legs are not distinct'
    assert abs(sum(f['duration'] for f in anims['run']['frames'])-.8)<1e-9
    assert abs(sum(f['duration'] for f in anims['attack']['frames'])-20/60)<1e-9
    assert max(b[3]-b[1] for b in metrics['idle'])<=42
    assert max(b[3]-b[1] for b in metrics['run'])<=42
    palette=lambda frames:{p[:3] for f in frames for p in f.get_flattened_data() if p[3]}
    base_palette=palette([crop(anims['idle']['frames'][0])])
    assert palette(run).issubset(base_palette),'Separate movement lighting palette'
    target=ROOT/'mods/tfm2_custom/champions/tfm2_custom_basalt'
    assert (OUT/'body.png').read_bytes()==Path(str(target)+'#sheet.png').read_bytes()
    assert (OUT/'body.fanim').read_bytes()==Path(str(target)+'#anim.fanim').read_bytes()
    prior=json.loads(old('mods/tfm2_custom/champion/tfm2_custom_basalt.data_champion'));current=json.loads((ROOT/'mods/tfm2_custom/champion/tfm2_custom_basalt.data_champion').read_text())
    assert current['sprite']=='asset/tfm2_custom/champions/tfm2_custom_basalt'
    assert {k:v for k,v in prior.items() if k!='sprite'}=={k:v for k,v in current.items() if k!='sprite'},'Unexpected nonvisual champion change'
    rules=json.loads((ROOT/'rules/basalt.json').read_text());previous=json.loads(old('rules/basalt.json'));previous['revision']=rules['revision'];assert previous==rules,'Balance value changed'
    fx=Image.open(OUT/'effect-proposals.png');manifest=json.loads((OUT/'manifest.json').read_text());assert manifest['effect_frames']['boulder']['w']==60
    assert not (ROOT/'mods/tfm2_custom/vfx/basalt#sheet.png').exists(),'Unapproved effects integrated'
    total=sheet.width*sheet.height*4+fx.width*fx.height*4;assert total<=32*1024*1024
    report={'body_approved':True,'effects_approved':False,'body_visible_pixels':{'idle':[b[3]-b[1] for b in metrics['idle']],'run':[b[3]-b[1] for b in metrics['run']]},'body_atlas':sheet.size,'frame_entries':entries,'unique_pose_count':len(meta['pose_frames']),'decoded_body_and_candidate_effect_bytes':total,'reproducible_packing':True,'fixed_foot_anchor':meta['foot_anchor'],'foot_offset_below_frame_centre':20,'shared_palette':True,'six_distinct_walk_drawings':True,'walk_cycle_seconds':.8,'body_integrated':True,'effects_integrated':False,'balance_unchanged':True,'native_dll_unchanged':sha(ROOT/'mods/tfm2_custom_ai/tfm2_custom_ai.dll')==hashlib.sha256(old('mods/tfm2_custom_ai/tfm2_custom_ai.dll')).hexdigest(),'live_game':'pending'}
    (OUT/'body-verification.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
if __name__=='__main__':main()
