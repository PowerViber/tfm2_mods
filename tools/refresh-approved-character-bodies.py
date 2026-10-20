"""Apply only reviewed body hashes, then refresh the editor's embedded sprite copies."""
from pathlib import Path
import argparse,base64,hashlib,json,shutil
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/character-art-review'
def generate(apply=False):
    approvals=json.loads((OUT/'approval.json').read_text());paths={}
    for name,pack in [('steve','tfm2_blockcraft'),('omen','tfm2_valorant')]:
        candidate=OUT/f'{name}-candidate.png';expected=approvals[name].get('integratedSha256',approvals[name]['candidateSha256']);assert hashlib.sha256(candidate.read_bytes()).hexdigest()==expected,'Candidate differs from its approval/packing-repair record'
        target=ROOT/'mods'/pack/'champions'/f'{pack}_{name}#sheet.png';paths[name]=(target,pack)
        if apply:shutil.copyfile(candidate,target)
    if apply:
        p=ROOT/'editor/sprites-data.js';text=p.read_text();at=text.index('=')+1;data=json.JSONDecoder().raw_decode(text[at:].lstrip())[0]
        for name,(target,pack)in paths.items():data[name].update(png=base64.b64encode(target.read_bytes()).decode(),fanim=json.loads(target.with_name(f'{pack}_{name}#anim.fanim').read_text()),label=name.title()+' (approved simple body)')
        p.write_text(text[:at]+' '+json.dumps(data,ensure_ascii=False)+';\n')
    print('Approved body hashes verified'+('; runtime and editor copies refreshed.'if apply else'; inspection only.'))
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--apply',action='store_true');generate(p.parse_args().apply)
