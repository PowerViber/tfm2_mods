"""Register imagegen concepts and inspect live package frames without changing them.

Only mechanical alpha cleanup, whole-sprite cropping, nearest-neighbour sizing,
mirroring and packing occur here. New anatomy is authored through imagegen.
The approval gate intentionally prevents producing replacement game assets.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont, ImageOps

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / 'rules/ranked-body-art.json'
OUT = ROOT / 'docs/ranked-body-redesign'


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def write_json(path: Path, value: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, ensure_ascii=False) + '\n', encoding='utf8')


def asset_info(base: str, include_frames: bool = True) -> dict:
    path = ROOT / base
    fanim = Path(str(path) + '#anim.fanim')
    sheet = Path(str(path) + '#sheet.png')
    meta = json.loads(fanim.read_text(encoding='utf8'))
    with Image.open(sheet) as im:
        size = list(im.size)
    result = {'base': base, 'png': sheet.relative_to(ROOT).as_posix(),
            'animation': fanim.relative_to(ROOT).as_posix(), 'sheetSize': size,
            'sha256': sha(sheet), 'animationSha256': sha(fanim)}
    if include_frames:
        result['anims'] = meta['anims']
    return result


def referenced_assets(value: object) -> set[str]:
    if isinstance(value, str):
        return {'mods/' + value[6:]} if value.startswith('asset/') else set()
    if isinstance(value, dict):
        return set().union(*(referenced_assets(v) for v in value.values()))
    if isinstance(value, list):
        return set().union(*(referenced_assets(v) for v in value))
    return set()


def read_frame(asset: dict, pose: str = 'idle', index: int = 0) -> Image.Image:
    r = asset['anims'][pose]['frames'][index]['data']
    with Image.open(ROOT / asset['png']) as im:
        return im.convert('RGBA').crop(tuple(int(v) for v in
            (r['x'], r['y'], r['x'] + r['w'], r['y'] + r['h'])))


def split_concepts(char: dict, generated: Path) -> list[dict]:
    with Image.open(ROOT / char['source']) as source:
        source = source.convert('RGBA')
        # Imagegen source contains three full figures in equal horizontal cells.
        # Alpha fringe removal is deterministic; no body part is redrawn/rigged.
        source.putalpha(source.getchannel('A').point(lambda a: 255 if a >= 180 else 0))
        results = []
        for i, look in enumerate(char['conceptLooks']):
            x0, x1 = round(source.width * i / 3), round(source.width * (i + 1) / 3)
            cell = source.crop((x0, 0, x1, source.height))
            bounds = cell.getbbox()
            if not bounds:
                raise ValueError(f"{char['key']} concept {i} is empty")
            crop = cell.crop(bounds)
            scale = char['visibleHeight'] / crop.height
            crop = crop.resize((round(crop.width * scale), char['visibleHeight']), Image.Resampling.NEAREST)
            w, h = char['frame']
            fx, fy = char['feet']
            left, top = round(fx - crop.width / 2), fy - crop.height
            if min(left, top) < 0 or left + crop.width > w or top + crop.height > h:
                raise ValueError(f"{char['key']} requires explicit transparent padding, not clipping")
            frame = Image.new('RGBA', (w, h))
            frame.alpha_composite(crop, (left, top))
            paths = {}
            for facing, image in [('right', frame), ('left', ImageOps.mirror(frame))]:
                dest = generated / f"{char['key']}-look-{look}-{facing}.png"
                image.save(dest, optimize=False)
                paths[facing] = dest.relative_to(ROOT).as_posix()
            results.append({'look': look, 'name': char['looks'][look], 'files': paths,
                            'rawCell': [x0, 0, x1, source.height], 'rawBounds': list(bounds),
                            'frame': [w, h], 'feet': [fx, fy],
                            'sha256': {k: sha(ROOT / v) for k, v in paths.items()}})
    return results


def comparison(char: dict, concepts: list[dict], body: dict, stock: Image.Image) -> None:
    # Matched scale: existing body is never resized to make proposals look larger.
    images = [('Current', read_frame(body)), ('Knight', stock)]
    images += [(p['name'], Image.open(ROOT / p['files']['right']).convert('RGBA')) for p in concepts]
    canvas = Image.new('RGB', (1000, 360), '#14232c')
    d = ImageDraw.Draw(canvas)
    font_path = Path('C:/Windows/Fonts/segoeui.ttf')
    font = ImageFont.truetype(str(font_path), 14) if font_path.exists() else ImageFont.load_default()
    d.text((18, 8), char['name'] + ' · 1× above, 3× below · proposals, approval pending', font=font, fill='white')
    for i, (name, im) in enumerate(images):
        cx = 100 + 200 * i
        d.text((cx - 86, 39), name, font=font, fill='#d3e1ee')
        canvas.paste(im, (cx - im.width // 2, 125 - im.height), im)
        large = im.resize((im.width * 3, im.height * 3), Image.Resampling.NEAREST)
        canvas.paste(large, (cx - large.width // 2, 335 - large.height), large)
    canvas.save(OUT / f"{char['key']}-comparison.png")


def build(capture: bool) -> None:
    manifest = json.loads(MANIFEST.read_text(encoding='utf8'))
    generated = OUT / 'concepts'
    generated.mkdir(parents=True, exist_ok=True)
    baseline = {'baseline': manifest['baseline'], 'characters': {}, 'assets': {}}
    frozen_path = OUT / 'baseline-contract.json'
    frozen = json.loads(frozen_path.read_text(encoding='utf8')) if frozen_path.exists() else None
    review = {'version': manifest['version'], 'revision': manifest['revision'],
              'status': manifest['status'], 'approval': manifest['approval'],
              'references': manifest['references'], 'characters': []}
    stock_base = ROOT / 'Sprite kit/base champions/knight'
    stock_meta = json.loads(stock_base.with_suffix('.anim.json').read_text())
    r = stock_meta['anims']['idle']['frames'][0]['data']
    with Image.open(stock_base.with_suffix('.png')) as s:
        stock = s.convert('RGBA').crop(tuple(int(v) for v in (r['x'], r['y'], r['x']+r['w'], r['y']+r['h'])))
    for char in manifest['characters']:
        body = asset_info(char['base'])
        proposals = split_concepts(char, generated)
        path = ROOT / 'mods/tfm2_custom/champion' / (char['id'] + '.data_champion')
        data = json.loads(path.read_text(encoding='utf8'))
        assets = {}
        for base in sorted(referenced_assets(data)):
            if Path(str(ROOT / base) + '#sheet.png').exists():
                assets[base] = asset_info(base, include_frames=False)
        decoded = sum(a['sheetSize'][0] * a['sheetSize'][1] * 4 for a in assets.values())
        baseline['characters'][char['key']] = {
            'definition': path.relative_to(ROOT).as_posix(), 'definitionSha256': sha(path),
            'body': body, 'declaredDecodedBytes': decoded}
        baseline['assets'].update(assets)
        review['characters'].append({**char, 'body': body, 'proposals': proposals,
            'sourceSha256': sha(ROOT / char['source']),
            'declaredDecodedBytes': decoded, 'declaredSheets': len(assets),
            'largestSheet': max((max(a['sheetSize']) for a in assets.values()), default=0)})
        comparison(char, proposals, body, stock)
    dll = ROOT / 'mods/tfm2_custom_ai/tfm2_custom_ai.dll'
    if dll.exists():
        baseline['nativeDllSha256'] = sha(dll)
    target = OUT / 'baseline-contract.json'
    if capture:
        if target.exists():
            raise SystemExit('Baseline already exists; refusing to overwrite the frozen audit')
        write_json(target, baseline)
    elif target.exists():
        before = json.loads(target.read_text(encoding='utf8'))
        # At proposal stage every shipped asset and champion definition must stay identical.
        if manifest['optIn']['integrated']:
            allowed={c['base'] for c in manifest['characters']}
            allowed.update('mods/tfm2_custom/vfx/vesper_outfits_r'+str(i) for i in range(11))
            allowed.update('mods/tfm2_custom/vfx/magnus_outfit_'+str(i) for i in range(11))
            allowed.update('mods/tfm2_custom/vfx/magnus_casts_'+str(i) for i in range(11))
            allowed.add('mods/tfm2_custom/vfx/coder_rank')
            assert before['nativeDllSha256']==baseline['nativeDllSha256'],'Native DLL changed'
            assert before['assets'].keys()==baseline['assets'].keys(),'Asset aliases changed'
            for name,asset in before['assets'].items():
                if name not in allowed:assert asset==baseline['assets'][name],name+' independent FX changed'
            for char in manifest['characters']:
                key=char['key'];info=baseline['characters'][key];data=json.loads((ROOT/info['definition']).read_text(encoding='utf8'))
                assert data['passive']['params'].pop('rank_art_v2')==1
                old=json.loads(subprocess.check_output(['rtk','proxy','git','show',manifest['baseline']+':'+info['definition']],cwd=ROOT))
                assert data==old,key+' combat definition changed'
                for pose,anim in before['characters'][key]['body']['anims'].items():
                    assert [f['duration'] for f in anim['frames']]==[f['duration'] for f in info['body']['anims'][pose]['frames']],key+' timing changed'
            assert baseline['characters']['magnus']['declaredDecodedBytes']<=manifest['budget']['magnusCompletePresentationBytes']
        elif before != baseline:
            raise SystemExit('Live package/source contract changed; inspect before regenerating')
    review['presentation']=manifest.get('presentation',{'nativeMode':'current-package'})
    runtime={'revision':manifest['revision'],'nativeMode':review['presentation']['nativeMode'],'characters':{}}
    for c in review['characters']:
        c['baselineDecodedBytes']=frozen['characters'][c['key']]['declaredDecodedBytes'] if frozen else c['declaredDecodedBytes']
        runtime['characters'][c['key']]={'asset':c['base'].replace('mods/','asset/',1),'png':c['body']['png'],'anims':c['body']['anims']}
    write_json(ROOT/'editor/ranked-body-assets.json',runtime)
    write_json(OUT / 'review-data.json', review)
    print(f"Review rebuilt: {sum(len(c['proposals']) for c in review['characters'])} approved concepts; production contract verified")
    for c in review['characters']:
        print(f"{c['name']}: {len(c['looks'])} looks reserved, inherited declared art {c['declaredDecodedBytes']/2**20:.2f} MiB")


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--capture-baseline', action='store_true')
    build(parser.parse_args().capture_baseline)
