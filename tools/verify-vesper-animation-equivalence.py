"""Pixel- and timing-exact audit across atlas repacking and rank splits."""
from pathlib import Path
from PIL import Image
import argparse,json,hashlib,struct
ROOT=Path(__file__).resolve().parents[1]
p=argparse.ArgumentParser();p.add_argument('--record-baseline',action='store_true');args=p.parse_args()
def inspect():
    tags={};atlases={}
    files=[ROOT/'mods/tfm2_custom/champions/tfm2_custom_vesper#anim.fanim',*sorted((ROOT/'mods/tfm2_custom/vfx').glob('vesper*#anim.fanim'))]
    for path in files:
        sheet=path.with_name(path.name.replace('#anim.fanim','#sheet.png'));im=Image.open(sheet).convert('RGBA');atlases[str(sheet.relative_to(ROOT))]=im.width*im.height*4
        for tag,anim in json.loads(path.read_text(encoding='utf-8'))['anims'].items():
            digest=hashlib.sha256()
            for fr in anim['frames']:
                r=fr['data'];x,y,w,h=[int(r[k])for k in('x','y','w','h')]
                assert x>=0 and y>=0 and x+w<=im.width and y+h<=im.height,(path,tag)
                digest.update(struct.pack('<IId',w,h,fr['duration']));digest.update(im.crop((x,y,x+w,y+h)).tobytes())
            assert tag not in tags,tag
            tags[tag]={'hash':digest.hexdigest(),'frames':len(anim['frames']),'duration':sum(f['duration']for f in anim['frames'])}
    return {'tags':tags,'atlases':atlases,'decodedBytes':sum(atlases.values())}
baseline=ROOT/'docs/vesper-animation-baseline.json';current=inspect()
if args.record_baseline:
    assert not baseline.exists(),'Baseline already exists; do not replace the original reference.'
    baseline.write_text(json.dumps({'referenceCommit':'30f2969',**current},indent=2)+'\n',encoding='utf-8');print('Recorded',len(current['tags']),'original animations.')
else:
    before=json.loads(baseline.read_text());assert current['tags']==before['tags'],[k for k in current['tags'].keys()|before['tags'].keys()if current['tags'].get(k)!=before['tags'].get(k)][:12]
    assert current['decodedBytes']<=before['decodedBytes']
    report={'animations':len(current['tags']),'allPixelsAndTimingsIdentical':True,'beforeDecodedBytes':before['decodedBytes'],'afterDecodedBytes':current['decodedBytes'],'savedDecodedBytes':before['decodedBytes']-current['decodedBytes'],'largestAtlasBytes':max(current['atlases'].values()),'scope':'Static assets, not a live FPS measurement'}
    (ROOT/'docs/vesper-animation-equivalence.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report))
