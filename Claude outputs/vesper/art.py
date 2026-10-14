"""Vesper, the Rift Marksman: regenerate every runtime sheet, then the editor previews and docs images.

    python "Claude outputs/vesper/art.py"     then     python "Claude outputs/vesper/data.py"

Drawing lives in vesper_art.py (body, outfits, aiming poses, emblems, auras) and rift_art.py (rifts, cores,
glyphs, links, packets, techniques, bonuses, Breach). This file packs them into <=2048 px atlases the same way
The Unified Theory does, adds the HUD bars, and replaces files atomically so re-runs are byte-identical.
"""
import importlib.util
import json
import math
import os
import tempfile
from pathlib import Path
from types import SimpleNamespace

from PIL import Image, ImageDraw

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
MOD = ROOT / 'mods/tfm2_custom'


def load(name, file):
    spec = importlib.util.spec_from_file_location(name, HERE / file)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


V = load('vesper_art_common', 'vesper_art.py')
RA = load('vesper_rift_art', 'rift_art.py')


def atlas(folder, name, entries, w, h, cols=16, trim=False, palette=False):
    """One sheet + fanim; with trim, every animation gets one shared symmetric crop (no wobble)."""
    frames = sum(len(v) for v in entries.values())
    anims = {}
    n = 0
    sheet = Image.new('RGBA', (2048, 2048) if trim else (cols * w, math.ceil(frames / cols) * h))
    cx = cy = row = used = 0
    for tag, seq in entries.items():
        fw, fh = w, h
        if trim:
            boxes = [im.getbbox() for im, _ in seq if im.getbbox()]
            if boxes:
                fw = min(w, 2 * math.ceil(max(max(w / 2 - b[0], b[2] - w / 2) for b in boxes) + 1))
                fh = min(h, 2 * math.ceil(max(max(h / 2 - b[1], b[3] - h / 2) for b in boxes) + 1))
            else:
                fw = fh = 2
        out = []
        for im, duration in seq:
            if trim:
                if cx + fw + 2 > 2048:
                    cx = 0
                    cy += row
                    row = 0
                x, y = cx + 1, cy + 1
                assert y + fh + 1 <= 2048, (name, tag)
                im = im.crop(((w - fw) // 2, (h - fh) // 2, (w + fw) // 2, (h + fh) // 2))
                cx += fw + 2
                row = max(row, fh + 2)
                used = max(used, cx)
            else:
                x, y = n % cols * w, n // cols * h
            sheet.alpha_composite(im, (x, y))
            out.append({'duration': round(duration, 6), 'data': {'x': x, 'y': y, 'w': fw, 'h': fh}})
            n += 1
        anims[tag] = {'frames': out}
    if trim:
        sheet = sheet.crop((0, 0, max(used, 1), max(cy + row, 1)))
    if palette:
        sheet = sheet.quantize(colors=256, method=Image.Quantize.FASTOCTREE, dither=Image.Dither.NONE)
    folder.mkdir(parents=True, exist_ok=True)
    assert max(sheet.size) <= 2048, (name, sheet.size)
    sheet.save(folder / (name + '#sheet.png'))
    (folder / (name + '#anim.fanim')).write_text(json.dumps({'anims': anims}, indent=2) + '\n')


def bounded(name, entries, w, h, cols=16):
    """Pack into one trimmed 2048 px sheet, splitting into a/b halves when it does not fit."""
    folder = MOD / 'vfx'
    try:
        atlas(folder, name, entries, w, h, cols, trim=True, palette=True)
        return [name]
    except AssertionError:
        for suffix in ['#anim.fanim', '#sheet.png']:
            (folder / (name + suffix)).unlink(missing_ok=True)
        items = list(entries.items())
        half = len(items) // 2
        assert half, (name, 'single animation exceeds 2048 px')
        return bounded(name + 'a', dict(items[:half]), w, h, cols) + bounded(name + 'b', dict(items[half:]), w, h, cols)


# ------------------------------------------------------------------ HUD (buff layers above her head)
HUD = (48, 96)          # centred on the body; the bars sit above her hood


def charges(n, f):
    im = Image.new('RGBA', HUD)
    d = ImageDraw.Draw(im)
    for k in range(5):
        x = 11 + k * 5
        ready = k < n
        d.rectangle((x, 9, x + 3, 12), fill=V.INK)
        if ready:
            pulse = 200 + (40 if (f + k) % 8 < 2 else 0)
            d.rectangle((x + 1, 10, x + 2, 11), fill=V.a(V.RIFT_PALE, pulse))
        else:
            d.point((x + 1, 11), fill=V.a(V.RIFT_DEEP, 255))
    return im


def strain(b, f):
    im = Image.new('RGBA', HUD)
    d = ImageDraw.Draw(im)
    d.rectangle((10, 14, 37, 16), fill=V.INK)
    fill = min(26, round(26 * b / 10))
    col = (255, 93, 93, 255) if b >= 9 else (255, 184, 77, 255) if b >= 7 else (127, 209, 255, 255)
    if b >= 9 and f % 2:
        col = (255, 220, 200, 255)
    if fill:
        d.rectangle((11, 15, 10 + fill, 15), fill=col)
    if b >= 7:
        for k in range(b - 6):
            x = 14 + k * 6 + (f % 2)
            d.line((x, 13, x + 1, 17), fill=V.a(V.WHITE, 220))
    return im


def _generate():
    for old in (MOD / 'vfx').glob('vesper_*#*'):
        old.unlink()
    # the registered body: rank-0 look, eight frames per animation
    entries = {kind: [(V.body(kind, f), .1) for f in range(8)] for kind in ['idle', 'run', 'attack', 'skill1', 'skill2', 'ult', 'hit', 'dead']}
    atlas(MOD / 'champions', 'tfm2_custom_vesper', entries, 48, 64)
    vfx = MOD / 'vfx'
    # rank layers (view buffs): outfits and aiming poses for every look (0..7 ranks, 8 = #1)
    layers = {f'outfit_r{l}': [(V.outfit(l, f), .12) for f in range(8)] for l in range(9)}
    layers.update({f'aim_r{l}': [(V.aim(l, f), .12) for f in range(8)] for l in range(9)})
    atlas(vfx, 'vesper_outfits', layers, 48, 64, 16)
    badges = {f'rank{r}': [(V.badge(r, f), .12) for f in range(8)] for r in range(7)}
    badges.update({f'top{p}': [(V.badge(7, f, p), .12) for f in range(8)] for p in range(1, 11)})
    atlas(vfx, 'vesper_badges', badges, *V.BADGE, 16)
    backs = {f'back_r{r}': [(V.back(r, f), .12) for f in range(8)] for r in range(7)}
    backs.update({f'back_r7_p{p}': [(V.back(7, f, p), .12) for f in range(8)] for p in range(1, 5)})
    bounded('vesper_backs', backs, *V.AURA)
    hud = {f'charges{n}': [(charges(n, f), .12) for f in range(8)] for n in range(6)}
    hud.update({f'strain{b}': [(strain(b, f), .12) for f in range(8)] for b in range(11)})
    atlas(vfx, 'vesper_hud', hud, *HUD, 16)
    RA.generate(SimpleNamespace(MOD=MOD, bounded=bounded, atlas=atlas))
    V.previews(SimpleNamespace(MOD=MOD))
    print('Vesper body, rank layers, HUD, rifts, techniques, bonuses and previews generated')


def generate():
    """Replace every file atomically (Windows previews can memory-map old atlases)."""
    original, original_text = Image.Image.save, Path.write_text

    def atomic_text(destination, data, encoding=None, errors=None, newline=None):
        fd, temporary = tempfile.mkstemp(dir=destination.parent, suffix=destination.suffix)
        os.close(fd)
        try:
            result = original_text(Path(temporary), data, encoding=encoding or 'utf-8', errors=errors, newline='\n')
            os.replace(temporary, destination)
            return result
        finally:
            Path(temporary).unlink(missing_ok=True)

    def atomic_save(im, destination, format=None, **options):
        if not isinstance(destination, (str, Path)):
            return original(im, destination, format=format, **options)
        destination = Path(destination)
        fd, temporary = tempfile.mkstemp(dir=destination.parent, suffix=destination.suffix)
        os.close(fd)
        try:
            original(im, temporary, format=format, **options)
            os.replace(temporary, destination)
        finally:
            Path(temporary).unlink(missing_ok=True)

    Image.Image.save, Path.write_text = atomic_save, atomic_text
    try:
        _generate()
    finally:
        Image.Image.save, Path.write_text = original, original_text


if __name__ == '__main__':
    generate()
