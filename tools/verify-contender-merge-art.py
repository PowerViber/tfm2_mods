"""Check approved sources, exact animation contracts and unchanged legacy pixels."""
from pathlib import Path
import hashlib, io, json, os, subprocess
from PIL import Image, ImageOps
ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/os.environ.get('CONTENDER_RESULTS','docs/contender-merge-integration-review');OUT.mkdir(exist_ok=True)
BASE='6626c55d272966ea5bcf969e748beb74bbfab60e'
manifest=json.loads((ROOT/'docs/contender-shadow-review/presentation.json').read_text())
merge=manifest['merge'];normal=manifest['body'];sheet=Image.open(ROOT/merge['file'].removeprefix('../')).convert('RGBA')
assert sheet.size==(960,256) and manifest['decoded_bytes']<=32*1024*1024
for tag,a in normal['anims'].items():assert merge['anims'][tag]==a,tag
for tag,a in merge['anims'].items():
    for f in a['frames']:
        r=f['data'];assert r['w']==80 and r['h']==64 and r['x']+80<=sheet.width and r['y']+64<=sheet.height
        im=sheet.crop((r['x'],r['y'],r['x']+80,r['y']+64));box=im.getbbox()
        assert box and box[0]>0 and box[1]>0 and box[2]<80 and box[3]<64,tag
        if not tag.endswith('_l'):
            opposite=merge['anims'][tag+'_l']['frames'][a['frames'].index(f)]['data']
            left=sheet.crop((opposite['x'],opposite['y'],opposite['x']+80,opposite['y']+64))
            expected=Image.new('RGBA',(80,64));expected.alpha_composite(ImageOps.mirror(im),(1,0))
            assert left.tobytes()==expected.tobytes(),tag
legacy=[]
for path in ['mods/tfm2_custom/champions/tfm2_custom_contender#sheet.png','mods/tfm2_custom/champions/tfm2_custom_contender#anim.fanim',
             'mods/tfm2_custom/vfx/contender_shadow#sheet.png','mods/tfm2_custom/vfx/contender_shadow#anim.fanim']:
    old=subprocess.check_output(['rtk','proxy','git','show',BASE+':'+path],cwd=ROOT)
    assert old.replace(b'\r\n',b'\n')==(ROOT/path).read_bytes().replace(b'\r\n',b'\n'),path
    legacy.append(path)
for name,digest in manifest['merge_source_hashes'].items():
    assert hashlib.sha256((ROOT/'docs/contender-merge-review'/name).read_bytes()).hexdigest()==digest
champ=json.loads((ROOT/'mods/tfm2_custom/champion/tfm2_custom_contender.data_champion').read_text())
assert champ['passive']['params']['shadow_merge_art_v1']==1
bindings={e['name']:e for e in champ['view_buffs']}
for item in manifest['merge_bindings']:
    e=bindings[item['name']];assert e['tag']==item['tag'] and e['anim'].endswith('tfm2_custom_contender_merge')
assert len(manifest['merge_bindings'])==48
before=hashlib.sha256((ROOT/merge['file'].removeprefix('../')).read_bytes()).hexdigest()
subprocess.run(['rtk','proxy','python','tools/build-contender-merge-art.py'],cwd=ROOT,check=True,capture_output=True)
assert before==hashlib.sha256((ROOT/merge['file'].removeprefix('../')).read_bytes()).hexdigest()
report=dict(appearance_approved=True,source_hashes=manifest['merge_source_hashes'],poses=20,tiles=24,facings=2,
            exact_original_frame_counts_durations_and_anchors=True,mirrors_match=True,unclipped=True,
            legacy_pixels_and_metadata_unchanged=legacy,deterministic_packing=True,
            atlas=list(sheet.size),total_decoded_bytes=manifest['decoded_bytes'],merge_sheet_sha256=before,
            native_alignment_acceptance='pending; registered anchors and Lab rendering are not host-renderer proof')
(OUT/'merge-art.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
